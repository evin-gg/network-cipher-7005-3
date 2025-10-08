mod networking_util;
mod cipher;

// standard
use::std::{process, env};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use::std::os::fd::AsRawFd;
use std::sync::atomic::Ordering;
use ctrlc;

// network sockets
use socket2::{Socket};
use nix::sys::socket::{
    MsgFlags, send, recv
};

// other util
use networking_util::{
    check_valid_ip, server_arg_validation, setup_server
};
use cipher::{split_payload};

fn handle_signal(flag: &Arc<AtomicBool>) {
    println!("Signal received");
    flag.store(false, Ordering::SeqCst);
}

fn main() {

    let catch = Arc::new(AtomicBool::new(true));
    let c = catch.clone();

    ctrlc::set_handler(move || handle_signal(&c)).expect("[SERVER] Signal Handler Error");

    // args
    let args: Vec<String> = env::args().collect();

    // verify args
    match server_arg_validation(&args) {
        Ok(()) => {},
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    }

    //verify ip
    match check_valid_ip(&args[1]) {
        Ok(()) => {},
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

    while catch.load(Ordering::SeqCst) {

        // accept
        let (clientfd, _clientaddr) = match socket.accept() {
            Ok((fd, addr)) => {
                println!("[SERVER] Accepted connection");
                (fd, addr)
            },
            Err(e) => {
                eprintln!("[SERVER] Accept Error {}", e);
                return;
            }
        };

        // read in
        let mut buf = [0u8; 1024];
        let read_bytes = match recv(clientfd.as_raw_fd(), &mut buf, MsgFlags::empty()){
            Ok(n) => {println!("[SERVER] Received {} bytes", n); n},
            Err(e) => {println!("[SERVER] Error: {}", e); 0},
        };
        println!("[SERVER] Payload: {}", String::from_utf8_lossy(&buf[..read_bytes]));

        // process
        let response = split_payload(&buf);

        //send
        send(clientfd.as_raw_fd(), response.as_bytes(), MsgFlags::empty()).expect("[SERVER] Error sending response");
    }

    drop(socket);
    println!("[SERVER] Socket closed. Exiting");
}
