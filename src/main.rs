use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    path::Path,
};

use hello::ThreadPool;

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to address: {}", e);
            return;
        }
    };

    let pool = match ThreadPool::build(4) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to create ThreadPool: {}", e);
            return;
        }
    };

    for stream in listener.incoming().take(100) {
        match stream {
            Ok(stream) => {
                let res = pool.execute(|| {
                    handle_connection(stream);
                });
                
                if let Err(e) = res {
                    eprintln!("Failed to send job to pool: {}", e);
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    
    let request_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        _ => return,
    };

    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = if Path::new(filename).exists() {
        fs::read_to_string(filename).unwrap_or_default()
    } else {
        String::from("404 Not Found (Missing File)")
    };

    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to write response to stream: {}", e);
    }
}