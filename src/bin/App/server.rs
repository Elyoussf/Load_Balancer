use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[tokio::main]
async fn main(){
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server is listening on 127.0.0.1:8080");

    loop{
        let (mut socket,_) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut buf = [0;1024];

            let n = socket.read(&mut buf).await.unwrap();
            println!("Received : {}",String::from_utf8_lossy(&buf[..n]));
            socket.write_all(b"Hello from server!").await.unwrap();
        });
    }
}