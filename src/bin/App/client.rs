use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let load_balancer_address = "127.0.0.1:8080";
    println!("Client test against load balancer at {}", load_balancer_address);
    
    // Simulate 10 clients with different IP addresses making multiple requests
    let mut handles = vec![];
    
    for i in 0..3 {
        let address = load_balancer_address.to_string();
        let handle = tokio::spawn(async move {
            // Simulate client making 3 requests to test sticky sessions
            for request_num in 1..=3 {
                // Connect to the load balancer
                match TcpStream::connect(&address).await {
                    Ok(mut stream) => {
                        println!("Client {} connected (request {})", i + 1, request_num);
                        
                        // Send a request
                        let message = format!("Hello from client {} (request {})!", i + 1, request_num);
                        match stream.write_all(message.as_bytes()).await {
                            Ok(_) => {
                                // Receive a response
                                let mut buf = [0; 1024];
                                match stream.read(&mut buf).await {
                                    Ok(n) if n > 0 => {
                                        println!(
                                            "Client {} received: {}",
                                            i + 1,
                                            String::from_utf8_lossy(&buf[..n])
                                        );
                                    }
                                    Ok(_) => println!("Client {} received empty response", i + 1),
                                    Err(e) => println!("Client {} error reading response: {}", i + 1, e),
                                }
                            }
                            Err(e) => println!("Client {} error sending request: {}", i + 1, e),
                        }
                    }
                    Err(e) => println!("Client {} connection error: {}", i + 1, e),
                }
                
                // Add a delay between requests from the same client
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
        handles.push(handle);
        
        // Add a small delay between starting clients
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Wait for all clients to finish
    for handle in handles {
        handle.await?;
    }
    
    println!("All client tests completed");
    Ok(())
}