use chess_lib::{packet::Packet, opcode::Opcode};
use serde::de::IntoDeserializer;

pub struct Command {
    id: Opcode,
    arg: Option<Argument>
}

pub enum Argument {
    Bool(bool),
    Int32(u32),
    String(String)
}

pub fn check_len(str: &Vec<&str>, len: usize, cmd: &str) -> Result<(), String> {
    if str.len() != len {
        return Err(format!("Incorrect amount of arguments for {}", cmd))
    }
    Ok(())
}

impl Command {
    pub fn new(id: Opcode, arg: Argument) -> Self {
        Command {
            id,
            arg: Some(arg)
        }
    }

    pub fn new_no_arg(id: Opcode) -> Self {
        Command {
            id,
            arg: None
        }
    }

    pub fn build_packet(&self) -> (Packet, bool) {
        let pl = match &self.arg {
            None => vec![],
            Some(arg) => match arg {
                Argument::Bool(b) => if *b {vec![1]} else {vec![0]},
                Argument::Int32(i) => i.to_be_bytes().to_vec(),
                Argument::String(s) => s.as_bytes().to_vec()
            }
        };
        let mut req_resp = true;
        if self.id == Opcode::Join || self.id == Opcode::SendMsg {
            req_resp = false;
        }
        (Packet::new_prim(self.id.clone(), pl), req_resp)
    }

    pub fn from_strings(strs: Vec<&str>) -> Result<Self, String> {
        match strs[0] {
            "join" => {
                check_len(&strs, 2, "join")?;
                Ok(Command::new(Opcode::Join, Argument::String(strs[1].to_string())))
            }
            "create_game" => {
                check_len(&strs, 2, "create_game")?;
                let white = match strs[1] {
                    "white" => true,
                    _ => false
                };
                Ok(Command::new(Opcode::CreateGame, Argument::Bool(white)))
            },
            "join_game" => {
                check_len(&strs, 2, "join_game")?;
                let id = strs[1].parse::<u32>();
                match id {
                    Ok(i) => Ok(Command::new(Opcode::JoinGame, Argument::Int32(i))),
                    Err(e) => Err(e.to_string())
                }
            },
            _ => Err("Invalid command.".to_string())
        }
    }
}