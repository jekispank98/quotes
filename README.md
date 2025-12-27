# Stock Quote Streaming System

A real-time stock quote streaming system implemented in Rust. 
This project features a multi-threaded server that generates synthetic market data and a client application that subscribes to updates via a custom TCP/UDP protocol.

## 🏗 Project Structure (Workspace)

The project is organized as a Rust Workspace to manage shared logic and dependencies efficiently:

* **`quote_common`**: A shared library containing data structures (`Quote`), JSON serialization logic, and custom error types (`ParserError`).
* **`quote_server`**: The "Generator" application. It manages TCP commands, generates market data using a Random Walk algorithm, and streams data via UDP.
* **`quote_client`**: The CLI application. It connects to the server, sends subscription requests, and displays the live data stream.

## 🚀 Tech Stack

* **Language**: Rust (Edition 2021)
* **Networking**: `std::net` (TCP for control, UDP for data streaming)
* **Serialization**: `serde` + `serde_json` (Strict JSON format requirement)
* **Concurrency**: `std::thread`, `std::sync::mpsc`, and `crossbeam-channel` (for multi-subscriber broadcasting)
* **CLI**: `clap` (for robust command-line argument parsing)

## 🛠 Installation & Usage

### 1. Build the Workspace
cargo build --release

### 2. Run the server
The server starts the TCP listener and the price generator.
cargo run -p quote_server

### 3. Run the client
cargo run -p quote_client -- [ARGUMENTS]

Example Command:
cargo run -p quote_client --server-ip 192.168.0.10 --listen-port 55555 --path ./tickers.txt

### Data channel (UDP)
Quotes are pushed to the client in the following JSON format:
JSON
{
  "ticker": "AAPL",
  "price": 150.25,
  "volume": 1200,
  "timestamp": 1672531200
}
