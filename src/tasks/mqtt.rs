use embassy_net::{
    tcp::{ConnectError, TcpSocket},
    Ipv4Address, Stack,
};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::{RSA, SHA};
use esp_mbedtls::{asynch::Session, Certificates, Mode, Tls, TlsVersion, X509};
use esp_println::println;
use log::{error, info};

use crate::{
    dns::DnsBuilder,
    mqtt::MqttClient,
    tasks::{MQTT_CLIENT_ID, MQTT_USR_NAME, MQTT_USR_PASS, SERVERNAME},
};

use super::{MQTT_SERVERNAME, MQTT_SERVERPORT};

#[embassy_executor::task]
pub async fn mqtt_handler(stack: &'static Stack<'static>, mut sha: SHA, mut rsa: RSA) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let tls = Tls::new(&mut sha).unwrap().with_hardware_rsa(&mut rsa);

    //wait until wifi connected
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address); //dhcp IP address
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    loop {
        Timer::after(Duration::from_millis(1_000)).await;

        let remote_endpoint = if let Ok(endpoint) = dns_query(stack).await {
            endpoint
        } else {
            continue;
        };
        println!("Establish TCP connection to broker {:?}", remote_endpoint);
        let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
        socket.connect(remote_endpoint).await.unwrap();
        let certificates = Certificates {
            ca_chain: X509::pem(concat!(include_str!("../../certs/crt.pem"), "\0").as_bytes()).ok(),
            certificate: X509::pem(concat!(include_str!("../../certs/dvt.crt"), "\0").as_bytes()).ok(),
            private_key: X509::pem(concat!(include_str!("../../certs/dvt.key"), "\0").as_bytes()).ok(),
            password: None,
        };

        println!("Open TLS session");
        let session = Session::new(
            socket,
            Mode::Client {
                servername: SERVERNAME,
            },
            TlsVersion::Tls1_3,
            certificates,
            tls.reference(),
        )
        .unwrap();
        println!("Establishing MQTT client connection ...");
        let mut mqtt_client = MqttClient::new(MQTT_CLIENT_ID, session);
        mqtt_client
            .connect(
                remote_endpoint,
                60,
                Some(MQTT_USR_NAME),
                Some(&MQTT_USR_PASS),
            )
            .await
            .unwrap();
        println!("Establishing MQTT client connection OK");
        loop {
            use core::fmt::Write;
            let mut mqtt_topic: heapless::String<80> = heapless::String::new();
            writeln!(&mut mqtt_topic, "channels/{}/hello", MQTT_CLIENT_ID).unwrap();
            if let Err(e) = mqtt_client
                .publish(
                    &mqtt_topic,
                    b"Hello from MQTT client",
                    mqttrust::QoS::AtMostOnce,
                )
                .await
            {
                error!("Failed to publish MQTT packet: {:?}", e);
                break;
            }

            info!("MQTT sent OK");
            mqtt_client.poll().await;
            Timer::after_secs(1).await;
        }
    }
}

pub async fn dns_query(
    stack: &'static Stack<'static>,
) -> Result<embassy_net::IpEndpoint, ConnectError> {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
    let mut buffer = [0; 512];
    let dns_ip = Ipv4Address::new(8, 8, 8, 8);
    let remote_endpoint = embassy_net::IpEndpoint {
        addr: embassy_net::IpAddress::Ipv4(dns_ip),
        port: 53,
    };
    socket.connect(remote_endpoint).await?;
    let dns_builder = DnsBuilder::build(MQTT_SERVERNAME);
    socket.write(&dns_builder.query_data()).await.unwrap();

    let size = socket.read(&mut buffer).await.unwrap();
    let broker_ip = if size > 2 {
        if let Ok(ips) = DnsBuilder::parse_dns_response(&buffer[2..size]) {
            info!("broker IP: {}.{}.{}.{}", ips[0], ips[1], ips[2], ips[3]);
            ips
        } else {
            return Err(ConnectError::NoRoute);
        }
    } else {
        return Err(ConnectError::NoRoute);
    };

    let broker_ipv4 = Ipv4Address::new(broker_ip[0], broker_ip[1], broker_ip[2], broker_ip[3]);

    let remote_endpoint = embassy_net::IpEndpoint {
        addr: embassy_net::IpAddress::Ipv4(broker_ipv4),
        port: MQTT_SERVERPORT,
    };
    Ok(remote_endpoint)
}
