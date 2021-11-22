# udp-app

Simple GUI app that allows you to save phone numbers by key and receive them from database by running separate server and using UDP to send/receive data.

# How to run

```bash
git clone https://github.com/playXE/IcedPhoneNumbersUDP && cd IcedPhoneNumbersUDP
cargo build 
```
Then run `cargo run --bin server` and `cargo run --bin client`. You would need to enter IpV4 addr for server to bind to when running it. And in client you would need to enter IpV4 addr for client itself and sevrer address. 