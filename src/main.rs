extern crate bufstream;

use bufstream::BufStream;
use std::error::Error;
use std::io::{Write, BufRead, Read};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};

#[derive(Clone)]
struct Config {
    die: bool,
    port: String,
    pwd_hash : String
}

const DEFAULT_PORT: &'static str = "8000";
const DEFAULT_PWD: &'static str = "2b63faf8fbc1849334e2a63f2577e8507b2cf4cadc6214c5d64f4a36c47fc66e051f97cd9633cfd4f88bca61c49050ea1c60229e28672187a566d62dff5bf209";

fn main(){
    let mut config = Config{die:false, port:DEFAULT_PORT.to_string(), pwd_hash: DEFAULT_PWD.to_string()};
    let mut accept = true;

    while accept{
        let addr_string = "127.0.0.1:".to_string() + &config.port;
        match TcpListener::bind(&*addr_string) {
            Err(why) => println!("{}", why),
            Ok(listener) => for stream in listener.incoming() {
                match stream {
                    Err(why) => println!("accept err {}", why.description()),
                    Ok(stream) => {
                        config = handler(&config, &mut BufStream::new(stream));
                        if config.die {
                            accept = false;
                        }
                        break
                    }
                }
            }
        }
    }
}

fn echoln(string: String, s: &mut BufStream<TcpStream>){
    echo(string, s);
    match s.write_all(b"\n") {
        Err(_) => {},
        Ok(_) =>{},
    }
    match s.flush() {
        Err(_) => {},
        Ok(_) =>{},
    }
}

fn echo(string: String, s: &mut BufStream<TcpStream>){
    match s.write_all(string.as_bytes()){
        Err(why) => println!("could not reply {:?}", why),
        Ok(_) => {}
    }
}

fn handler(config: &Config, s: &mut BufStream<TcpStream>) -> Config{
    // Until the user does not request to close.
    loop{
        echo("> ".to_string(), s);
        match s.flush() {
            Err(_) => {},
            Ok(_) =>{},
        }
        let mut line = String::new();
        s.read_line(&mut line).unwrap();
        let mut it = 0;
        let mut main_cmd = String::new();
        let mut args = Vec::new();
        for cmd in line.split_whitespace(){
            if it == 0{
                main_cmd = cmd.to_string();
            }else{
                args.push(cmd)
            }
            it+=1;
        }

        // Here goes the command processing.
        match main_cmd.as_ref(){
            "exit" => return config.clone(),
            "kill" => {
                let mut clone = config.clone();
                clone.die = true;
                return clone
            },
            "cfgport" => {
                if args.len() == 0{
                    echoln("[USAGE] cfgport [port]".to_string(), s)
                }else{
                    match args[0].parse::<i32>(){
                        Err(_) => echoln("[USAGE] cfgport [port] (port must be an integer)".to_string(), s),
                        Ok(_) => {
                            let new_port = args[0]; // just for clarity
                            let addr_string = "127.0.0.1:".to_string() + new_port;
                            match TcpListener::bind(&*addr_string) {
                                Err(_) => echoln("[NOK] port ".to_string() + new_port + " unavailable", s),
                                Ok(_) => {
                                    echoln("[OK] reconnect via port ".to_string() + new_port, s);
                                    let mut clone = config.clone();
                                    clone.port = new_port.to_string();
                                    return clone
                                },
                            }
                        }
                    };
                }
            },
            "cfgpwd" => {
                if args.len() == 0{
                    echoln("[USAGE] cfgpwd [hash]".to_string(), s)
                } else {
                    if args[0].len() != 128 {
                        echoln("[USAGE] cfgpwd [hash] (hash must be a 128 character SHA512 hash)".to_string(), s);
                    } else {
                        echoln("[OK] reconnect with new password ".to_string(), s);
                        let mut clone = config.clone();
                        clone.pwd_hash = args[0].to_string();
                        return clone
                    }
                }
            },
            _ => match Command::new(main_cmd).args(&args).stdout(Stdio::piped()).spawn() {
                Err(why) => echoln(why.description().to_string(), s),
                Ok(process) => {
                    let mut stde = String::new();
                    match process.stderr {
                        Some(mut err) => {
                            match err.read_to_string(&mut stde) {
                                Err(why) => echoln(why.description().to_string(), s),
                                Ok(_) => echoln(stde.to_string(), s)
                            };
                        },
                        None => {
                            let mut stdo = String::new();
                            match process.stdout.unwrap().read_to_string(&mut stdo) {
                                Err(why) => echoln(why.description().to_string(), s),
                                Ok(_) => echoln(stdo.to_string(), s)
                            }
                        }
                    }
                },
            }
        }
    }
}
