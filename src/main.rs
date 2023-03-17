use log::{info, warn};
use env_logger::Env;
use tokio::net::TcpStream;


use crate::client::Client;

mod client;
mod command;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let mut client = Client::new( "127.0.0.1".to_string());
    match client.run().await {
        Err(e) => {warn!("Error: {}", e);}
        _ => {}
    };
}

// start connection to server and return TCP stream to it
async fn connect(mut server: String) -> std::io::Result<TcpStream> {
    server.push_str(":8088");
    info!("Starting client...");
    let stream = TcpStream::connect(server).await;
    match stream {
        Ok(s) => {
            info!("Found chess server...");
            Ok(s)
        }
        Err(e) => {
            warn!("Failed connection.");
            Err(e)
        }
    }
}