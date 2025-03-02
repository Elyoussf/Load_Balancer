use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::env;
use rand::Rng;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Get server address from command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <server_address>", args[0]);
        return Ok(());
    }
    
    let server_address = args[1].clone();
    
    // Generate a random weight for the server (or use the provided one)
    let weight: usize = if args.len() > 2 {
        args[2].parse().unwrap_or_else(|_| {
            let w = rand::thread_rng().gen_range(1..=10);
            println!("Invalid weight provided, using random weight: {}", w);
            w
        })
    } else {
        rand::thread_rng().gen_range(1..=10)
    };
    
    println!("Server {} starting with weight {}", server_address, weight);
    
    // Subscribe to the load balancer
    let load_balancer_address = "127.0.0.1:8081"; // Note: using the subscription port
    
    let mut stream = TcpStream::connect(load_balancer_address).await?;
    
    let subscription_message = format!("subscribe {} {}", server_address, weight);
    println!("Sending subscription: {}", subscription_message);
    
    stream.write_all(subscription_message.as_bytes()).await?;
    
    // Read confirmation response
    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).await?;
    if n > 0 {
        println!("Subscription response: {}", String::from_utf8_lossy(&buf[..n]));
    } else {
        println!("Empty response from load balancer");
    }
    
    // Start handling requests
    let listener = tokio::net::TcpListener::bind(server_address.clone()).await?;
    println!("Server {} is ready to handle requests", server_address);
    
    loop {
        let (mut socket, client_addr) = listener.accept().await?;
        println!("Server {} accepted connection from {}", server_address, client_addr);
        let server_address = server_address.clone();
        
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            match socket.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    let request = String::from_utf8_lossy(&buf[..n]);
                    println!("Server {} received: {}", server_address, request);
                    
                    // Add a small delay to simulate processing
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    
                    // Send response with server identity
                    let response = format!("Hello from server {}!", server_address);
                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        println!("Error sending response: {}", e);
                    }
                }
                Ok(_) => println!("Empty request received"),
                Err(e) => println!("Error reading request: {}", e),
            }
        });
    }
}
