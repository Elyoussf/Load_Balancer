# Load Balancer Demo in Rust

This project demonstrates a simple load balancer implemented in Rust using the Tokio asynchronous runtime. The load balancer supports multiple strategies for distributing client requests across multiple servers. Servers dynamically subscribe to the load balancer, and clients simulate multiple instances sending requests concurrently.

## Features

- **Dynamic Server Subscription**: Servers register themselves with the load balancer upon startup.
- **Multiple Load Balancing Strategies**:
  - Round Robin
  - Sticky Round Robin
  - Weighted Round Robin
  - Least Connections
- **Concurrent Clients**: Simulate multiple clients sending requests to the load balancer.
- **Clear Logging**: Detailed logs for the load balancer, servers, and clients.

## Prerequisites

- Rust and Cargo installed. If not, follow the instructions at [rustup.rs](https://rustup.rs/).
- Tokio runtime (included in `Cargo.toml`).

## Project Structure

```
src/
├── bin/
│   ├── client.rs    # Simulate multiple clients
│   ├── server.rs    # Simulate multiple servers
├── load_balancer.rs # Load balancer logic
├── main.rs          # Entry point
```

## How to Run

### 1. Clone the Repository
```bash
git clone <repository-url>
cd load-balancer-demo
```

### 2. Build the Project
```bash
cargo build
```

### 3. Start the Load Balancer

Run the load balancer with the desired strategy:
```bash
cargo run --bin load_balancer -- <strategy>
```
Replace `<strategy>` with one of the following:

- `round_robin`
- `sticky_round_robin`
- `weighted_round_robin`
- `least_connections`
- `least_time`

**Example:**
```bash
cargo run --bin load_balancer -- round_robin
```

### 4. Start the Servers

Start multiple servers on different ports. Each server will automatically subscribe to the load balancer.
```bash
cargo run --bin server -- 127.0.0.1:8000
cargo run --bin server -- 127.0.0.1:8001
```

### 5. Run the Clients

Simulate multiple clients sending requests to the load balancer:
```bash
cargo run --bin client
```

### 6. Observe the Logs

- **Load Balancer Logs**: Show which server is handling each request.
- **Server Logs**: Show received requests and responses.
- **Client Logs**: Show client connections and received responses.

---

This project serves as an educational example of implementing a load balancer in Rust. Feel free to experiment and extend its functionality! 🚀

