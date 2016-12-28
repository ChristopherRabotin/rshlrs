//#![feature(rustc_private)]
extern crate bufstream;
extern crate getopts;

use getopts::Options;
use std::env;
use std::net::{TcpListener, TcpStream};
use std::io::{Write, BufRead, Read};
use bufstream::BufStream;
use std::error::Error;
use std::process::{Command, Stdio};

struct Config {
    die: bool,
    port: String
}

fn main(){
    // argument parsing to create the initial configuration
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    let brief = format!("Usage: {} [options]", args[0].clone());
    opts.optopt("p", "port", "set binding port", "PORT");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => (m),
        Err(why) => {
            println!("{}", why);
            print!("{}", opts.usage(&brief));
            return
        }
    };
    let port;
    match matches.opt_str("p"){
        None => {
            println!("must specify a port");
            print!("{}", opts.usage(&brief));
            return;
        },
        Some(p) => port = p
    }
    let mut config = Config{die:false, port:port};
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

fn handler(config: &Config, s: &mut BufStream<TcpStream>) -> Config{
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
            "exit" => return Config{die:false, port:config.port.to_string()},
            "kill" => return Config{die:true, port:config.port.to_string()},
            "cfgport" => {
                if cmd_splt.len() == 1{
                    echo_str("[USAGE] cfgport [newport]".to_string(), s)
                }else{
                    match cmd_splt[1].parse::<i32>(){
                        Err(_) => echo_str("[USAGE] cfgport [newport] (newport must be an integer)".to_string(), s),
                        Ok(_) => {
                            let new_port = cmd_splt[1]; // just for clarity
                            let addr_string = "127.0.0.1:".to_string() + new_port;
                            match TcpListener::bind(&*addr_string) {
                                Err(_) => echo_str("[NOK] port ".to_string() + new_port + " unavailable", s),
                                Ok(_) => {
                                    echo_str("[OK] reconnect via port ".to_string() + new_port, s);
                                    return Config{die:false, port:new_port.to_string()}
                                },
                            }
                        }
                    };
                }
            },
            "cfgpwd" => echo_str("passwords not yet implemented".to_string(), s),
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
