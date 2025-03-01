use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let load_balancer_address = std::env::var("load_balancer_address").expect("Failed to load the address of the load balancer");
    let mut stream = TcpStream::connect(load_balancer_address).await.unwrap();
    stream.write_all(b"Hello from client!").await.unwrap();

    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).await.unwrap();
    println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
}

