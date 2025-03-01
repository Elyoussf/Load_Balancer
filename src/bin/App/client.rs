use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    stream.write_all(b"Hello from client!").await.unwrap();

    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).await.unwrap();
    println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
}