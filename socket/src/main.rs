// use std::io::Write;
use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) -> Option<String> {
    let script = "<script>setTimeout(() => window.close(), 3000)</script><h1>Success! Window will close automatically.</h1>";
    let length = script.len();

    let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n{script}");

    match stream.write_all(response.as_bytes()) {
        Ok(_) => return None,
        Err(e) => {
            // TODO: make this more helpful
            eprintln!("Error writing data to oauth tab: {}", e);
            return None;
        }
    }
}

fn main() {
    // println!("sup");

    // let mut file = std::fs::File::create(
    //     "/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/log/test.txt",
    // )
    // .unwrap();
    //
    // let res = file.write_all(b"Sup friend");
    //
    // println!("{res:?}");

    let possible_listener = TcpListener::bind(format!("127.0.0.1:{}", 6969));

    if let Err(listener_error) = possible_listener {
        eprintln!("Error getting listener: {}", listener_error);
        return;
    }

    let listener = possible_listener.unwrap();

    for possible_stream in listener.incoming() {
        match possible_stream {
            Ok(stream) => {
                let possible_code = handle_connection(stream);

                match possible_code {
                    Some(code) => {
                        println!("{code:?}")
                    }
                    None => {}
                }
            }
            Err(e) => {
                eprintln!("Stream encountered an unforseen error: {}", e);
                break;
            }
        }
    }
}
