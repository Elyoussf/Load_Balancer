use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct Server {
    address: String,
    weight: usize,
    connections: Arc<Mutex<usize>>, // For Least Connections
    response_time: Arc<Mutex<Duration>>, // For Least Time
    last_checked: Arc<Mutex<Instant>>, // When response time was last measured
}

struct LoadBalancer {
    servers: Arc<Mutex<Vec<Server>>>,
    strategy: String,
    sticky_sessions: Arc<Mutex<HashMap<String, usize>>>,
    wrr_counter: Arc<Mutex<usize>>, 
}

impl LoadBalancer {
    fn new(strategy: String) -> Self {
        LoadBalancer {
            servers: Arc::new(Mutex::new(Vec::new())),
            strategy,
            sticky_sessions: Arc::new(Mutex::new(HashMap::new())),
            wrr_counter: Arc::new(Mutex::new(0)), // Initialize counter
        }
    }

    async fn handle_client(&self, mut socket: TcpStream, client_ip: String) {
        let mut buf = [0; 1024];
        let n = socket.read(&mut buf).await.unwrap_or(0);
        
        if n == 0 {
            println!("Empty request received from {}", client_ip);
            return;
        }
        
        let request = String::from_utf8_lossy(&buf[..n]);
        println!("Load Balancer: Received request from {}: {}", client_ip, request);
        
        // Select a server - we need to clone the server before the await point
        let server = match self.select_server(&client_ip).await {
            Some(s) => s,
            None => {
                println!("No servers available to handle the request");
                let _ = socket.write_all(b"Error: No servers available").await;
                return;
            }
        };
        
        println!("Load Balancer: Forwarding request from {} to server {}", client_ip, server.address);

        // Increment connection count for Least Connections
        {
            let mut conn = server.connections.lock().unwrap();
            *conn += 1;
        }

        // Forward request to the selected server
        let start_time = Instant::now();
        
        match TcpStream::connect(&server.address).await {
            Ok(mut backend) => {
                if let Err(e) = backend.write_all(&buf[..n]).await {
                    println!("Error forwarding request: {}", e);
                    let _ = socket.write_all(b"Error forwarding request").await;
                } else {
                    // Send response back to client
                    let mut response_buf = [0; 1024];
                    match backend.read(&mut response_buf).await {
                        Ok(resp_n) if resp_n > 0 => {
                            let elapsed = start_time.elapsed();
                            
                            // Update server response time for the least_time strategy
                            {
                                let mut response_time = server.response_time.lock().unwrap();
                                let mut last_checked = server.last_checked.lock().unwrap();
                                
                                // Exponential moving average to smooth response time measurements
                                const ALPHA: f32 = 0.3; // Weight for new measurement
                                let old_time_ms = response_time.as_millis() as f32;
                                let new_time_ms = elapsed.as_millis() as f32;
                                let updated_time_ms = (ALPHA * new_time_ms) + ((1.0 - ALPHA) * old_time_ms);
                                
                                *response_time = Duration::from_millis(updated_time_ms as u64);
                                *last_checked = Instant::now();
                            }
                            
                            println!("Load Balancer: Received response from {} in {:?}", server.address, elapsed);
                            let _ = socket.write_all(&response_buf[..resp_n]).await;
                        }
                        Ok(_) => {
                            println!("Server sent empty response");
                            let _ = socket.write_all(b"Error: Empty response from server").await;
                        }
                        Err(e) => {
                            println!("Error reading response: {}", e);
                            let _ = socket.write_all(b"Error reading response from server").await;
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to connect to backend server {}: {}", server.address, e);
                let _ = socket.write_all(b"Error connecting to backend server").await;
            }
        }

        // Decrement connection count
        {
            let mut conn = server.connections.lock().unwrap();
            *conn -= 1;
        }
    }

    async fn select_server(&self, client_ip: &str) -> Option<Server> {
        //kankhdmou lock bash n holdiw data ondirou clone without risk nakhdou chi version incompatible
        let servers = self.servers.lock().unwrap();
        
        if servers.is_empty() {
            return None;
        }

        match self.strategy.as_str() {
            "round_robin" => {
                
                let server = servers[0].clone();
                
                
                drop(servers);
                
                
                let mut servers = self.servers.lock().unwrap();
                if !servers.is_empty() { // Check again in case servers changed
                    let server_to_push = servers.remove(0);
                    servers.push(server_to_push);
                }
                
                Some(server)
            }
            "sticky_round_robin" => {
                // For sticky, get the current index from sticky_sessions
                let mut sticky_sessions = self.sticky_sessions.lock().unwrap();
                let server_index = if let Some(index) = sticky_sessions.get(client_ip) {
                    *index
                } else {
                    0
                };
                
                // Safety check for index
                let server_index = server_index % servers.len();
                let server = servers[server_index].clone();
                
                // Update the sticky session for next time
                sticky_sessions.insert(client_ip.to_string(), (server_index + 1) % servers.len());
                
                Some(server)
            }
            "weighted_round_robin" => {
                let total_weight: usize = servers.iter().map(|s| s.weight).sum();
        if total_weight == 0 {
            return Some(servers[0].clone());
        }

        // Get and increment counter atomically
        let mut counter = self.wrr_counter.lock().unwrap();
        *counter = (*counter + 1) % total_weight;
        let current_pos = *counter;

        // Find server based on accumulated weights
        let mut acc_weight = 0;
        let selected_server = servers.iter()
            .find(|server| {
                acc_weight += server.weight;
                current_pos < acc_weight
            })
            .unwrap_or(&servers[0]);

        Some(selected_server.clone())
            }
            "least_connections" => {
                // Find server with least connections
                let selected = servers.iter()
                    .min_by_key(|s| {
                        let conn = s.connections.lock().unwrap();
                        *conn
                    })
                    .map(|s| s.clone());
                
                selected
            }
            "least_time" => {
                // Find server with shortest response time
                let selected = servers.iter()
                    .min_by(|a, b| {
                        let a_time = a.response_time.lock().unwrap();
                        let b_time = b.response_time.lock().unwrap();
                        
                        // Compare response times
                        a_time.cmp(&*b_time)
                    })
                    .map(|s| s.clone());
                
                selected
            }
            _ => {
                println!("Unsupported strategy: {}, falling back to round robin", self.strategy);
                if servers.is_empty() {
                    None
                } else {
                    Some(servers[0].clone())
                }
            }
        }
    }

    // Check server health periodically for least_time strategy
    async fn check_server_health(&self, server_addr: String) -> Duration {
        let start = Instant::now();
        match TcpStream::connect(&server_addr).await {
            Ok(_) => {
                // Just testing connection time
                start.elapsed()
            }
            Err(_) => {
                // If connection fails, mark as very slow
                Duration::from_secs(60)
            }
        }
    }

    async fn handle_subscription(&self, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = [0; 1024];
        let n = stream.read(&mut buf).await?;
        let message = String::from_utf8_lossy(&buf[..n]);
        let parts: Vec<&str> = message.split_whitespace().collect();
        
        if parts.len() < 3 || parts[0] != "subscribe" {
            println!("Invalid subscription message: {}", message);
            stream.write_all(b"Error: Invalid subscription format").await?;
            return Ok(());
        }
        
        let address = parts[1].to_string();
        let weight = match parts[2].parse::<usize>() {
            Ok(w) => w,
            Err(_) => {
                println!("Invalid weight in subscription: {}", parts[2]);
                stream.write_all(b"Error: Invalid weight format").await?;
                return Ok(());
            }
        };
        
        // Check server's initial response time for least_time strategy
        let initial_time = if self.strategy == "least_time" {
            self.check_server_health(address.clone()).await
        } else {
            Duration::from_millis(1000) // Default value
        };
        
        let server = Server {
            address: address.clone(),
            weight,
            connections: Arc::new(Mutex::new(0)),
            response_time: Arc::new(Mutex::new(initial_time)),
            last_checked: Arc::new(Mutex::new(Instant::now())),
        };
        
        // Add the server to our list
        {
            let mut servers = self.servers.lock().unwrap();
            servers.push(server);
        }
        
        println!("Load Balancer: New server subscribed at {} with weight {}", address, weight);
        
        // Send confirmation back to the server
        stream.write_all(b"Subscription successful").await?;
        
        Ok(())
    }

    // Start a health check loop for least_time
    async fn start_health_check_loop(self: Arc<Self>) {
        if self.strategy != "least_time" {
            return;
        }
        
        let health_check_interval = Duration::from_secs(30);
        let lb = Arc::clone(&self);
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(health_check_interval).await;
                
                // Get all servers
                let servers = {
                    let servers_guard = lb.servers.lock().unwrap();
                    servers_guard.clone()
                };
                
                // Check each server's health
                for server in servers {
                    let address = server.address.clone();
                    let response_time = lb.check_server_health(address.clone()).await;
                    
                    // Update server's response time
                    let mut time = server.response_time.lock().unwrap();
                    let mut last_checked = server.last_checked.lock().unwrap();
                    
                    *time = response_time;
                    *last_checked = Instant::now();
                    
                    println!("Health check for {}: {:?}", address, response_time);
                }
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get strategy from command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} strategy", args[0]);
        eprintln!("  @##Available strategies: round_robin, sticky_round_robin, weighted_round_robin, least_connections, least_time");
        return Ok(());
    }
    
    let strategy = args[1].clone();
    println!("Using load balancing strategy: {}", strategy);

    let lb = Arc::new(LoadBalancer::new(strategy));
    
    // Start health check loop if using least_time strategy
    if lb.strategy == "least_time" {
        let lb_clone = lb.clone();
        lb_clone.start_health_check_loop().await;
    }
    
    // Set up main client listener
    let client_listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Load balancer running for clients on 127.0.0.1:8080");
    
    // Set up subscription listener on a different port
    let subscription_listener = TcpListener::bind("127.0.0.1:8081").await?;
    println!("Server subscription service running on 127.0.0.1:8081");
    
    // Handle server subscriptions in a separate task
    let lb_sub = lb.clone();
    tokio::spawn(async move {
        println!("Waiting for server subscriptions...");
        loop {
            match subscription_listener.accept().await {
                Ok((socket, addr)) => {
                    println!("New subscription connection from {}", addr);
                    let lb = lb_sub.clone();
                    tokio::spawn(async move {
                        if let Err(e) = lb.handle_subscription(socket).await {
                            println!("Error handling subscription: {}", e);
                        }
                    });
                }
                Err(e) => {
                    println!("Error accepting subscription connection: {}", e);
                }
            }
        }
    });
    
    // Handle client connections
    println!("Waiting for client connections...");
    loop {
        match client_listener.accept().await {
            Ok((socket, addr)) => {
                println!("New client connection from {}", addr);
                let lb = lb.clone();
                let client_ip = addr.ip().to_string();
                tokio::spawn(async move {
                    lb.handle_client(socket, client_ip).await;
                });
            }
            Err(e) => {
                println!("Error accepting client connection: {}", e);
            }
        }
    }
}