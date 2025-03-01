use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::VecDeque;
use std::env;

#[derive(Clone)]
struct Server {
    address: String,
}

#[derive(Clone)]
struct LoadBalancer {
    servers: Arc<Mutex<VecDeque<Server>>>,
    strategy: String,
}

impl LoadBalancer {
    fn new(servers: Vec<String>, strategy: String) -> Self {
        let servers = servers.into_iter().map(|addr| Server { address: addr }).collect();
        LoadBalancer {
            servers: Arc::new(Mutex::new(servers)),
            strategy,
        }
    }

    async fn handle_connection(&self, mut socket: tokio::net::TcpStream) {
        let server = self.select_server();
        println!("Forwarding request to server: {}", server.address);

        
        let mut backend = tokio::net::TcpStream::connect(&server.address).await.unwrap();
        let mut buf = [0; 1024];
        let n = socket.read(&mut buf).await.unwrap();
        backend.write_all(&buf[..n]).await.unwrap();

        let n = backend.read(&mut buf).await.unwrap();
        socket.write_all(&buf[..n]).await.unwrap();
    }

    fn select_server(&self) -> Server {
        let mut servers = self.servers.lock().unwrap();
        match self.strategy.as_str() {
            "round_robin" => {
                let server = servers.pop_front().unwrap();
                servers.push_back(server.clone());
                server
            }
            _ => panic!("Unsupported strategy"),
        }
    }
}

#[tokio::main]
async fn main() {
   
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <server_address1> <server_address2> ...", args[0]);
        return;
    }
    let servers = args[1..].to_vec();

    let lb = LoadBalancer::new(servers, "round_robin".to_string());

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Load balancer running on 127.0.0.1:8080");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let lb = lb.clone();
        tokio::spawn(async move {
            lb.handle_connection(socket).await;
        });
    }
}