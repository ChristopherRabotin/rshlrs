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
                handler(&mut BufStream::new(stream));
            }
        }
    }
}

fn echo_err(why: std::io::Error, s: &mut BufStream<TcpStream>){
    let _ = s.write_all(why.description().as_bytes());
    let _ = s.write_all(b"\n");
}

fn echo_str(str: std::string::String, s: &mut BufStream<TcpStream>){
    let _ = s.write_all(str.as_bytes());
    let _ = s.write_all(b"\n");
    let _ = s.flush();
}

fn handler(s: &mut BufStream<TcpStream>){
    // Until the user does not request to close.
    loop{
        let _ = s.write(b"> ");
        let _ = s.flush();
        let mut cmd = std::string::String::new();
        let _ = s.read_line(&mut cmd).unwrap();
        let cmd_splt : Vec<_> = cmd.as_str().split_whitespace().collect();
        let mut args = Vec::new();
        for i in 1..cmd_splt.len() {
            args.push(cmd_splt[i]);
        }

        match Command::new(cmd_splt[0]).args(&args).stdout(Stdio::piped()).spawn() {
            Err(why) => echo_err(why, s),
            Ok(process) => {
                let mut stde = String::new();
                match process.stderr {
                    Some(mut err) => {
                        match err.read_to_string(&mut stde) {
                            Err(why) => echo_err(why, s),
                            Ok(_) => echo_str(stde, s),
                        };
                    },
                    None => {
                        let mut stdo = String::new();
                        match process.stdout.unwrap().read_to_string(&mut stdo) {
                            Err(why) => echo_err(why, s),
                            Ok(_) => echo_str(stdo, s),
                        }
                    }
                }
            },
        };
    }
}
