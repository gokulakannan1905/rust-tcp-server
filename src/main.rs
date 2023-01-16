use colored::Colorize;
use std::{net::TcpListener, net::TcpStream,process};
use tcp_server::{
    authenticate, edit_file, list_directory, read_from_stream, send_file, send_to_steam,
};
use threadpool::ThreadPool;

fn main() {
    // TcpListener instance is bound to a port and can be used to accept incoming connections.
    let listener = match TcpListener::bind("192.168.1.9:3333") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    // Accepting the incoming connections and storing the TcpStream in a variable
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_client(stream);
        });
    }
}

fn handle_client(mut stream: TcpStream) {
    let user = match read_from_stream(&mut stream) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };
    // dbg!(&user);
    let username = match user.split(' ').next() {
        Some(u) => u.trim(),
        None => {
            eprintln!("Error: Invalid username");
            return;
        }
    };

    let password = match user.split(' ').skip(1).next() {
        Some(p) => p.trim(),
        None => {
            eprintln!("Error: Invalid password");
            return;
        }
    };

    match authenticate(&mut stream, &username, &password) {
        Ok(true) => loop {
            let req = read_from_stream(&mut stream).unwrap().trim().to_string();
            let sub_command = req.split(' ').next().unwrap();
            match sub_command {
                "exit" => {
                    break;
                }
                "ls" => {
                    list_directory(&mut stream).unwrap();
                }
                "cat" => {
                    let filename = match req.split(" ").skip(1).next() {
                        Some(f) => f,
                        None => {
                            let err =
                                format!("{}", "💥Error: No filename provided".to_string().red());
                            send_to_steam(&mut stream, &err).unwrap();
                            continue;
                        }
                    };
                    send_file(&mut stream, &filename).unwrap();
                }
                "edit" => {
                    let filename = match req.split(" ").skip(1).next() {
                        Some(f) => f,
                        None => {
                            send_to_steam(&mut stream, "No filename provided").unwrap();
                            continue;
                        }
                    };
                    let line_number: usize = match req.split(" ").skip(2).next() {
                        Some(l) => match l.parse() {
                            Ok(n) => n,
                            Err(_) => {
                                send_to_steam(&mut stream, "Invalid line number").unwrap();
                                continue;
                            }
                        },
                        None => {
                            send_to_steam(&mut stream, "Line number not provided").unwrap();
                            continue;
                        }
                    };
                    edit_file(&mut stream, &filename, line_number).unwrap();
                }
                _ => {
                    match send_to_steam(&mut stream, "Invalid request") {
                        Ok(_) => {}
                        Err(_) => {
                            break;
                        }
                    };
                }
            }
        },
        Ok(false) => {
            return;
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    }
}
