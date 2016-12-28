#![feature(loop_break_value)]
extern crate bufstream;

use std::net::{TcpListener, TcpStream};
use std::io::{Write, BufRead, Read};
use bufstream::BufStream;
use std::error::Error;
use std::process::{Command, Stdio};

fn main(){
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Err(why) => println!("accept err {}", why.description()),
            Ok(stream) => {
                if handler(&mut BufStream::new(stream)){
                    break
                }
            }
        }
    }
}

fn echo_ln(s: &mut BufStream<TcpStream>){
    match s.write_all(b"\n") {
        Err(_) => {},
        Ok(_) =>{},
    }
    match s.flush() {
        Err(_) => {},
        Ok(_) =>{},
    }
}

fn echo_str(str: std::string::String, s: &mut BufStream<TcpStream>){
    match s.write_all(str.as_bytes()){
        Err(why) => println!("could not reply {:?}", why),
        Ok(_) => {}
    }
    echo_ln(s);
}

fn handler(s: &mut BufStream<TcpStream>) -> bool{
    // Until the user does not request to close.
    loop{
        let _ = s.write(b"> ");
        let _ = s.flush();
        let mut cmd = std::string::String::new();
        s.read_line(&mut cmd).unwrap();
        let cmd_splt : Vec<_> = cmd.as_str().split_whitespace().collect();
        let mut args = Vec::new();
        for i in 1..cmd_splt.len() {
            args.push(cmd_splt[i]);
        }
        let main_cmd = cmd_splt[0];
        // Here goes the command processing.
        match main_cmd{
            "exit" => return false,
            "kill" => return true,
            "cfgport" => {},
            "cfgpwd" => {},
            _ => match Command::new(main_cmd).args(&args).stdout(Stdio::piped()).spawn() {
                Err(why) => echo_str(why.description().to_string(), s),
                Ok(process) => {
                    let mut stde = String::new();
                    match process.stderr {
                        Some(mut err) => {
                            match err.read_to_string(&mut stde) {
                                Err(why) => echo_str(why.description().to_string(), s),
                                Ok(_) => echo_str(stde.to_string(), s)
                            };
                        },
                        None => {
                            let mut stdo = String::new();
                            match process.stdout.unwrap().read_to_string(&mut stdo) {
                                Err(why) => echo_str(why.description().to_string(), s),
                                Ok(_) => echo_str(stdo.to_string(), s)
                            }
                        }
                    }
                },
            }
        }
    }
}
