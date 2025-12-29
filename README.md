# Multithreaded Web Server in Rust

A bare-bones, low-level HTTP web server implementation in Rust. This project demonstrates how to build a concurrent server from scratch using the standard library (`std`), without relying on external frameworks like Actix or Tokio.

It implements a custom **Thread Pool** to manage concurrent connections efficiently and handles graceful shutdowns.

## Features

* **TCP Networking**: Listens on `127.0.0.1:7878` for incoming TCP connections.
* **Custom Thread Pool**:
  * Pre-allocates a fixed number of workers (4 by default) to prevent resource exhaustion.
  * Uses `mpsc` channels to dispatch jobs to workers.
  * Uses `Arc<Mutex<...>>` for thread-safe receiver sharing.

* **Robust Error Handling**:
  * Includes a custom `PoolError` enum for creation and dispatch errors.
  * Returns `Result` types for pool operations rather than panicking.

* **Graceful Shutdown**: Implements the `Drop` trait to ensure all threads finish their current jobs before the server closes.
* **Basic HTTP Parsing**: Manually parses `GET` requests to serve HTML content.

## Project Structure

```text
.
├── Cargo.toml
├── src
│   ├── lib.rs       # ThreadPool implementation, Worker logic, and Error types
│   └── main.rs      # Entry point, TCP listener, and HTTP request parsing
├── hello.html       # (Required) HTML file for successful 200 responses
└── 404.html         # (Required) HTML file for 404 responses

```

## Getting Started

### Prerequisites

* [Rust and Cargo](https://www.rust-lang.org/tools/install) installed.

### Setup

**Clone or Create the Project**:
Ensure your `Cargo.toml` defines the package name as `hello` (or update `src/main.rs` imports to match your package name).

### Running the Server

Run the server using Cargo:

```bash
cargo run
```

Open your web browser and navigate to:

* [http://127.0.0.1:7878](https://www.google.com/search?q=http://127.0.0.1:7878) (Serves `hello.html`)
* [http://127.0.0.1:7878/unknown](https://www.google.com/search?q=http://127.0.0.1:7878/unknown) (Serves `404.html`)

> **Note:** For demonstration purposes, the server is configured to shut down automatically after accepting **100 connections** (see `listener.incoming().take(100)` in `main.rs`).

## Testing

The `src/lib.rs` file contains unit tests for the ThreadPool logic (creation validation and execution). Run them with:

```bash
cargo test
```

## Architecture Overview

### The Request Lifecycle

1. **Main Thread**: The `TcpListener` waits for a connection.
2. **Handshake**: When a client connects, the stream is passed to the `pool.execute` method.
3. **Dispatch**: The `ThreadPool` sends the job (a closure containing `handle_connection`) down an `mpsc` channel.
4. **Worker**: A free thread (Worker) locks the shared receiver, grabs the job, and executes it.
5. **Response**: The Worker reads the file system and writes the HTTP response back to the TCP stream.

### Synchronization Primitives

To make the `mpsc::Receiver` shared and thread-safe among multiple workers, the project uses the type:

```rust
Arc<Mutex<mpsc::Receiver<Job>>>
```

* **`Arc`**: Allows multiple workers to own the receiver.
* **`Mutex`**: Ensures only one worker retrieves a job from the receiver at a time.

## License

This project is open source and available under the [MIT License](https://www.google.com/search?q=LICENSE).
