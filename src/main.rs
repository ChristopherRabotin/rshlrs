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

fn echo_err(why: std::io::Error, s: &mut BufStream<TcpStream>)-> std::io::Result<()>{
    s.write_all(why.description().as_bytes());
    s.write_all(b"\n")
}

fn echo_str(str: std::string::String, s: &mut BufStream<TcpStream>) -> std::io::Result<()>{
    //s.write_all(str.as_bytes()).and_then(s.write_all(b"\n").and_then(s.flush()));
    s.write_all(str.as_bytes());
    s.write_all(b"\n");
    s.flush()
}

fn handler(s: &mut BufStream<TcpStream>){
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
            "exit" => break,
            _ => match Command::new(main_cmd).args(&args).stdout(Stdio::piped()).spawn() {
                Err(why) => {
                    match echo_err(why, s){
                        Err(why) => println!("could not send error {:?}", why),
                        Ok(_) =>{}
                    }
                },
                Ok(process) => {
                    let mut stde = String::new();
                    match process.stderr {
                        Some(mut err) => {
                            match err.read_to_string(&mut stde) {
                                Err(why) => match echo_err(why, s){
                                    Err(why) => println!("could not send error {:?}", why),
                                    Ok(_) =>{}
                                },
                                Ok(_) => match echo_str(stde, s){
                                    Err(why) => println!("{:?}", why),
                                    Ok(_) =>{}
                                },
                            };
                        },
                        None => {
                            let mut stdo = String::new();
                            match process.stdout.unwrap().read_to_string(&mut stdo) {
                                Err(why) => match echo_err(why, s){
                                    Err(why) => println!("could not send error {:?}", why),
                                    Ok(_) =>{}
                                },
                                Ok(_) => match echo_str(stdo, s){
                                    Err(why) => println!("{:?}", why),
                                    Ok(_) =>{}
                                },
                            }
                        }
                    }
                },
            }
        }
    }
}
