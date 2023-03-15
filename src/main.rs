use std::{net::TcpStream, io::{self,Write}};
use chess_lib::packet::{Packet, self};
use serde::Serialize;
use log::{info, warn};
use env_logger::Env;


use crate::client::Client;

mod client;
mod command;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let mut client = Client::new("Kyle".to_string(), "127.0.0.1".to_string());
    client.run();
}


fn connect(name: String, mut server: String) -> std::io::Result<TcpStream> {
    server.push_str(":8088");
    info!("Starting client...");
    let stream = TcpStream::connect(server);
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