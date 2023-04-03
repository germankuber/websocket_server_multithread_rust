# WebSocket Server Example in Rust

This repository contains an example of a WebSocket server implementation using Rust. The server accepts incoming WebSocket connections, receives messages from clients, and broadcasts the messages to all connected clients.

## Features

- Asynchronous WebSocket server using async_std and async_tungstenite
- Accepts incoming WebSocket connections
- Receives messages from clients
- Broadcasts messages to all connected clients
- Demonstrates usage of async channels and shared state

## Getting Started

### Prerequisites

- Rust: To install Rust, follow the instructions on the official [Rust website](https://www.rust-lang.org/tools/install).

### Running the server

1. Clone the repository:


git clone https://github.com/yourusername/websocket_server_example.git
cd websocket_server_example

2. Build the project:

cargo build


3. Run the server:

cargo run


The server will start listening on `127.0.0.1:8080`.

### Testing the server

You can use a WebSocket client, such as [websocat](https://github.com/vi/websocat), to test the server. Install websocat and run the following command to connect to the server and start sending messages:

websocat ws://127.0.0.1:8080


## Built With

- [Rust](https://www.rust-lang.org/)
- [async_std](https://docs.rs/async-std)
- [async_tungstenite](https://crates.io/crates/async-tungstenite)
- [tungstenite](https://crates.io/crates/tungstenite)
- [futures](https://crates.io/crates/futures)
- [log](https://crates.io/crates/log)
- [env_logger](https://crates.io/crates/env_logger)

