# MQTT ESP Rust Example

This repository provides an example of using **MQTT** on **ESP32** with **Rust**, including **TLS support** for secure communication.

## Features
- Connect ESP32 to an MQTT broker using **Rust**.
- Secure communication with **TLS support**.
- Lightweight and efficient implementation.

## Requirements
- **Rust toolchain** (nightly recommended)
- **ESP32 toolchain** (esp-rs)
- **Cargo** and **cargo-espflash**
- An MQTT broker (e.g., [Mosquitto](https://mosquitto.org/), [EMQX](https://www.emqx.io/), or [HiveMQ](https://www.hivemq.com/))

## Installation & Setup
### 1. Install Rust for ESP32
Follow the official [esp-rs setup guide](https://esp-rs.github.io/book/installation/index.html) to install the necessary tools.

### 2. Clone the Repository
```sh
git clone https://github.com/TuEmb/mqtt-esp-rs-example.git
cd mqtt-esp-rs-example
```
### 3. Setup MQTT client
- put your certs at `certs`
- Update your MQTT info at `src/tasks/mod.rs`
### 4. Build & Flash
Run the following command to build and flash the firmware:
```sh
SSID="your-wifi" PASSWORD="your-wifi-password" cargo r --release
```

## Usage
Once flashed, the ESP32 will:
1. Connect to the configured Wi-Fi network.
2. Establish a **TLS-secured** MQTT connection.
3. Publish to a topic.

## Roadmap
- [ ] Add support for all ESP variants (esp32c6 for now)

## License
This project is licensed under the **MIT License**.

## Contributions
Contributions are welcome! Feel free to open an issue or submit a pull request.


