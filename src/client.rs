use std::{collections::HashMap, io::{self, Write, Read}, net::TcpStream};
use chess_lib::{packet::Packet, opcode::Opcode};
use crate::{connect, command::Command};
use log::{info, warn};

pub struct Client {
    name: String,
    server: String
}

impl Client {
    pub fn new(name: String, server: String) -> Self {
        Client {
            name,
            server,
        }
    }

    pub fn connect(&mut self) -> Result<TcpStream, String> {
        match connect(
            self.name.to_owned(),
            self.server.to_owned()
        ) {
            Ok(itc) => {
                self.join_server(&itc)?;
                Ok(itc)
            },
            Err(e) => return Err(e.to_string())
        }
    }

    pub fn join_server(&mut self, stream: &TcpStream) -> Result<(), String> {
        let mut msg = String::new();
        print!("Enter username to connect: ");
        io::stdout().flush();
        match io::stdin().read_line(&mut msg) {
            Ok(_) => {
                let str = msg.trim_end();
                let join = Command::from_strings(vec!["join", str])?;
                info!("Sending join...");
                self.run_cmd(join, &stream)
            }
            Err(e) => Err(e.to_string())
        }
    }

    pub fn run_cmd(&mut self, cmd: Command, stream: &TcpStream) -> Result<(), String> {
        let (pkt, req_resp) = cmd.build_packet();
        info!("{:?}", pkt);
        let _c = self.send_packet(stream, pkt)?;
        if req_resp {
            let p = self.get_packet(&stream)?;
            println!("{}", p.payload_str());
        }
        Ok(())
    }

    pub fn get_cmd(&self) -> Result<Command, String> {
        let mut msg = String::new();
        print!("Enter command to run: ");
        io::stdout().flush();
        match io::stdin().read_line(&mut msg) {
            Ok(_) => {
                let strs = msg.trim_end().split(" ");
                Command::from_strings(strs.collect())
            },
            Err(e) => Err(e.to_string())
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        match self.connect() {
            Ok(sout) => {
                loop {
                    match self.get_cmd() {
                        Ok(cmd) => match self.run_cmd(cmd, &sout) {
                            Ok(()) => {},
                            Err(e) => {println!("Error: {}", e)}
                        }
                        Err(e) => {
                            println!("Error: {}", e)
                        }
                    }
                }
            },
            Err(e) => {
                warn!("Cannot establish join: {}", e);
                Ok(())}
        }
    }

    pub fn send_packet(&self, mut stream: &TcpStream, pkt: Packet) -> Result<(), String> {
        info!("Sending packet {:?}", pkt);
        match stream.write(&pkt.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }

    pub fn get_packet(&mut self, stream: &TcpStream) -> Result<Packet, String> {
        let mut buf: [u8; 2] = [0, 0];
        match stream.peek(&mut buf) {
            Ok(_) => {
                let size: u64 = buf[1] as u64;
                let mut pbytes: Vec<u8> = Vec::new();
                match stream.take(size+2).read_to_end(&mut pbytes) {
                    Ok(_) => Packet::from_bytes(pbytes),
                    Err(e) => Err(e.to_string())
                }
            },
            Err(e) => {
                warn!("Error: {}", e.to_string());
                Err(e.to_string())
            }
        }
    }


}