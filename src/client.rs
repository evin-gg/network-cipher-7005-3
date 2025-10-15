mod networking_util;

#[allow(unused_imports)]


use networking_util::{
    format_send, check_valid_ip, client_response_handler, client_arg_validation, client_connect
};

use std::time::Duration;
use::std::{process, env};

fn main() {

    // get user args
    let args: Vec<String> = env::args().collect();

    // verify args
    match client_arg_validation(&args) {
        Ok(()) => {},
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    } 

    //verify ip
    match check_valid_ip(&args[3]) {
        Ok(()) => {},
        Err(e) => {
            println!("Ip address error: {}", e);
            process::exit(1);
        }
    }

    let socket = match client_connect(&args) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };

    for _n in 0..3{
        // Send the formatted data
        match format_send(args.clone(), &socket) {
            Ok(()) => {},
            Err(e) => {
                eprintln!("[CLIENT] Error Sending Data {}", e);
                process::exit(1);
            }
        };

        
        println!("SLEEPING FOR 2 SECONDS");
        std::thread::sleep(Duration::from_secs(2));

        // Receive the response
        client_response_handler(&socket);
    }
}
