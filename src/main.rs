use base64;
use regex::bytes::Regex;
use sha1::{Digest, Sha1};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

extern crate server;
use server::thread_pool::ThreadPool;

const INDEX_HTML: &'static [u8] = include_bytes!("../index.html");

const NOT_FOUND_HTML: &'static [u8] = include_bytes!("../404.html");

fn main() {
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

    let get_index = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    let websocket = b"GET /websocket HTTP/1.1\r\n";

    if buf.starts_with(get_index) {
        let response = format!(
            "HTTP/1.1 200 OK\r\n\r\n{}",
            String::from_utf8_lossy(INDEX_HTML)
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else if buf.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        let response = format!(
            "HTTP/1.1 200 OK\r\n\r\n{}",
            String::from_utf8_lossy(INDEX_HTML)
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else if buf.starts_with(websocket) {
        let regex = Regex::new("Sec-WebSocket-Key: (.*)").unwrap();

        if let Some(caps) = regex.captures(&buf) {
            println!("opening handshake received");

            let key = String::from(String::from_utf8_lossy(caps.get(1).unwrap().as_bytes()).trim())
                + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
            let hash = base64::encode(Sha1::digest(key.as_bytes()));

            let response = format!(
                "HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: {}\r\n\r\n",
                hash
            );
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();

            loop {
                let mut msg_buf = [0; 1024];
                if let Ok(_) = stream.read(&mut msg_buf) {
                    if msg_buf[0] == 0 {
                        break;
                    }
                    println!("message received");
                } else {
                    break;
                }
            }
        } else {
            let response = format!(
                "HTTP/1.1 404 NOT FOUND\r\n\r\n{}",
                String::from_utf8_lossy(NOT_FOUND_HTML)
            );
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    } else {
        let response = format!(
            "HTTP/1.1 404 NOT FOUND\r\n\r\n{}",
            String::from_utf8_lossy(NOT_FOUND_HTML)
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
