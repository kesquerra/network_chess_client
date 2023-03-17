use std::{io::{self, Write}, time::Duration, sync::mpsc::{self, Sender, Receiver}};
use chess_lib::{packet::Packet, opcode::Opcode, chess::fen_to_str};
use tokio::{net::TcpStream, spawn, time::sleep, task::JoinHandle};
use crate::{connect, command::Command};
use log::{info, warn};

pub struct Client {
    server: String
}

impl Client {
    pub fn new(server: String) -> Self {
        Client {
            server,
        }
    }

    // make connection to the server
    pub async fn connect(&mut self, tx: &mut Sender<Packet>) -> Result<TcpStream, String> {
        match connect(
            self.server.to_owned()
        ).await {
            Ok(itc) => {
                self.join_server(tx).await?;
                Ok(itc)
            },
            Err(e) => Err(e.to_string())
        }
    }

    // send join request to server
    pub async fn join_server(&mut self, tx: &mut Sender<Packet>) -> Result<(), String> {
        let mut msg = String::new();
        print!("Enter username to connect: ");
        io::stdout().flush().unwrap();
        match io::stdin().read_line(&mut msg) {
            Ok(_) => {
                let str = msg.trim_end();
                let join = Command::from_strings(vec!["join", str])?;
                info!("Sending join...");
                Client::run_cmd(join, tx).await
            }
            Err(e) => Err(e.to_string())
        }
    }

    // print the board from the FEN string in ascii art
    pub fn print_board(fen: String) {
        println!("{}", fen_to_str(fen))
    }

    // read the packet response and print accordingly
    pub fn process_resp(packet: Packet) {
        let str = packet.payload_str();
        match packet.op() {
            Opcode::SendMoveResp => {
                println!("Move accepted:");
                Client::print_board(str)
            },
            Opcode::RecvMove => {
                println!("Opponent played:");
                Client::print_board(str)
            },
            Opcode::ShowGameResp => {Client::print_board(str)},
            Opcode::JoinGameResp => {
                println!("Game joined:");
                Client::print_board(str)
            },
            Opcode::CreateGameResp => {
                println!("Game created:");
                Client::print_board(str)
            },
            Opcode::LeaveGameResp => {println!("Left game {}.", str)},
            Opcode::Err => {println!("Error: {}", str)}
            _ => {println!("{}", str);}
        }
    }

    // run a command input
    pub async fn run_cmd(cmd: Command, tx: &mut Sender<Packet>) -> Result<(), String> {
        let (pkt, _) = cmd.build_packet();
        tx.send(pkt).unwrap();
        Ok(())
    }

    // get a command from the string input in terminal
    pub fn get_cmd() -> Result<Command, String> {
        let mut msg = String::new();
        print!("Enter command to run: ");
        io::stdout().flush().unwrap();
        match io::stdin().read_line(&mut msg) {
            Ok(_) => {
                let strs = msg.trim_end().split(" ");
                Command::from_strings(strs.collect())
            },
            Err(e) => Err(e.to_string())
        }
    }

    // send keep alive packets continuously every 5 sec
    pub async fn keep_alive(&mut self, tx: Sender<Packet>) -> JoinHandle<()> {
        spawn(async move {loop {
            tx.send(Packet::ka()).unwrap();
            sleep(Duration::from_secs(5)).await;
        }})
    }

    // loop to get command from terminal and run the command by sending a packet to the server
    pub async fn cmd_loop(&mut self, rx: Receiver<Result<(), String>>, tx: &mut Sender<Packet>) -> Result<(), String> {
        loop {
            match rx.try_recv() {
                Ok(res) => {
                    match res {
                        Ok(_) => {},
                        Err(e) => return Err(e)
                    }
                }
                Err(_) => {}
            }
            match Client::get_cmd() {
                Ok(cmd) => {
                    match Client::run_cmd(cmd, tx).await {
                    Ok(()) => {},
                    Err(e) => {println!("Error: {}", e)}}
                }
                Err(e) => {
                    println!("Error: {}", e)
                }
            }
        }
    }

    // send and read packets from TCP stream
    pub async fn send_pkts(&mut self, tcp: TcpStream, rx: Receiver<Packet>, rrx: Sender<Result<(), String>>) -> (JoinHandle<()>, JoinHandle<()>) {
        //split read/write
        let (mut r, mut w) = tcp.into_split();
        
        // send packets on write channel
        let send = spawn(async move {
            loop {
                let res = rx.recv();
                
                if let Ok(pkt) = res {
                    let res = pkt.send_write(&mut w).await;
                    rrx.send(res).unwrap();
                }          
        }});

        // read packets on read channel
        let read = spawn(async move {
            loop {
                let res = Packet::from_tcp_read(&mut r).await;
                if let Ok(pkt) = res {
                    Client::process_resp(pkt);
                }          
        }});

        (send, read)
    }

    // run the client software
    // processes initial join to the server, starts the keepalive packets every 5 sec
    // starts command loop to allow user to send packets to server using commands
    pub async fn run(&mut self) -> Result<(), String> {
        let (ptx, prx) = mpsc::channel();
        let mut jptx = ptx.clone();
        match self.connect(&mut jptx).await {
            Ok(s) => {
                let (tx, rx) = mpsc::channel();
                let atx = ptx.clone();
                let (r, w) = self.send_pkts(s, prx, tx).await;
                let k = self.keep_alive(atx).await;
                let mut aatx = ptx.clone();
                self.cmd_loop(rx, &mut aatx).await.unwrap();
                k.await.unwrap();
                r.await.unwrap();
                w.await.unwrap();
                Ok(())
            },
            Err(e) => {
                warn!("Cannot establish join: {}", e);
                Ok(())}
        }
    }
}