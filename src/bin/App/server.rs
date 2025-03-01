use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::env;

#[tokio::main]
async fn main() {
   
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <address>", args[0]);
        return;
    }
    let address = &args[1];

    
    let listener = TcpListener::bind(address).await.unwrap();
    println!("Server running on {}", address);

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            let n = socket.read(&mut buf).await.unwrap();
            println!("Server received: {}", String::from_utf8_lossy(&buf[..n]));

            socket.write_all(b"Hello from server!").await.unwrap();
        });
    }
}