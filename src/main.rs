use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use regex::bytes::Regex;
use sha1::{Sha1, Digest};

extern crate server;
use server::thread_pool::ThreadPool;

const INDEX_HTML: &'static [u8] = include_bytes!("../index.html");

const NOT_FOUND_HTML: &'static [u8] = include_bytes!("../404.html");

fn main() {
    let result = Sha1::digest(b"Hello, world");
    println!("{:x}", result);

    let regex = Regex::new("Sec-WebSocket-Key: (.*)").unwrap();

    if let Some(caps) = regex.captures(b"Sec-WebSocket-Key: abcdefg") {
        println!("{}", String::from_utf8_lossy(caps.get(1).unwrap().as_bytes()).trim());
    }

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(4);

    // for stream in listener.incoming().take(2) {
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, contents) = if buf.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", String::from_utf8_lossy(INDEX_HTML))
    } else if buf.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", String::from_utf8_lossy(INDEX_HTML))
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", String::from_utf8_lossy(NOT_FOUND_HTML))
    };

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
