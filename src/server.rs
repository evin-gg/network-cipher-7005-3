mod cipher;
mod networking_util;

// standard
use ::std::os::fd::AsRawFd;
use ::std::{env, process};
use ctrlc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

// network sockets
use nix::sys::socket::{MsgFlags, send};
use socket2::Socket;

// poll
use tokio::net::{TcpListener};
use tokio::io::{AsyncReadExt};

// other util
use cipher::split_payload;
use networking_util::{check_valid_ip, server_arg_validation, setup_server};

fn handle_signal(flag: &Arc<AtomicBool>) {
    println!("Signal received");
    flag.store(false, Ordering::SeqCst);
}

#[tokio::main]
async fn main() {
    let catch = Arc::new(AtomicBool::new(true));
    let c = catch.clone();

    ctrlc::set_handler(move || handle_signal(&c)).expect("[SERVER] Signal Handler Error");

    // args
    let args: Vec<String> = env::args().collect();

    // verify args
    match server_arg_validation(&args) {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    }

    //verify ip
    match check_valid_ip(&args[1]) {
        Ok(()) => {}
        Err(e) => {
            println!("Ip address error: {}", e);
            process::exit(1);
        }
    }

    // setup server
    let socket: Socket = match setup_server(&args) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };
    match socket.set_nonblocking(true) {
        Ok(()) => {},
        Err(e) => {
            println!("[SERVER] Failed to set socket to non-blocking: {}", e);
            process::exit(1);
        }
    };

    // convert socket2 to TCP stream
    let listener: std::net::TcpListener = socket.into();
    let tokio_listener = match TcpListener::from_std(listener) {
        Ok(listener) => {listener},
        Err(e) => {
            eprintln!("[SERVER] Failed to convert socket to a Tokio listener: {}", e);
            process::exit(1);
        }
    };


    while catch.load(Ordering::SeqCst) {

        // new accept
        let (mut clientfd, _clientaddr) = tokio_listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];

            loop {
                let _n = match clientfd.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(_n) => {
                        println!("[SERVER] Client connected"); 
                        println!("[SERVER] Payload: {}", String::from_utf8_lossy(&buf[.._n]));
                    },
                    Err(e) => {
                        eprintln!("[SERVER] Could not read from client: {}", e);
                        return;
                    }
                };

                let response = split_payload(&buf);
                send(clientfd.as_raw_fd(), response.as_bytes(), MsgFlags::empty()).expect("[SERVER] Error sending response");
            }
        });


        // accept
        // let (clientfd, _clientaddr) = match socket.accept() {
        //     Ok((fd, addr)) => {
        //         println!("[SERVER] Accepted connection");
        //         (fd, addr)
        //     },
        //     Err(e) => {
        //         eprintln!("[SERVER] Accept Error {}", e);
        //         return;
        //     }
        // };

        // // read in
        // let mut buf = [0u8; 1024];
        // let read_bytes = match recv(clientfd.as_raw_fd(), &mut buf, MsgFlags::empty()){
        //     Ok(n) => {println!("[SERVER] Received {} bytes", n); n},
        //     Err(e) => {println!("[SERVER] Error: {}", e); 0},
        // };
        // println!("[SERVER] Payload: {}", String::from_utf8_lossy(&buf[..read_bytes]));

        // // process
        // let response = split_payload(&buf);

        // //send
        // send(clientfd.as_raw_fd(), response.as_bytes(), MsgFlags::empty())
        //     .expect("[SERVER] Error sending response");
    }

    // drop(socket);
    println!("[SERVER] Socket closed. Exiting");
}
