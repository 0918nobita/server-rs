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
        index_page(&mut stream);
    } else if buf.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        index_page(&mut stream);
    } else if buf.starts_with(websocket) {
        if let Some(key) = parse_ws_key(&buf) {
            send_back_handshake(&mut stream, key);
            receive_ws_messages(&mut stream);
        } else {
            not_found_page(&mut stream);
        }
    } else {
        not_found_page(&mut stream);
    }
}

fn index_page(stream: &mut TcpStream) {
    let response = format!(
        "HTTP/1.1 200 OK\r\n\r\n{}",
        String::from_utf8_lossy(INDEX_HTML)
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn not_found_page(stream: &mut TcpStream) {
    let response = format!(
        "HTTP/1.1 404 NOT FOUND\r\n\r\n{}",
        String::from_utf8_lossy(NOT_FOUND_HTML)
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn parse_ws_key<'a>(headers: &'a [u8]) -> Option<String> {
    let regex = Regex::new("Sec-WebSocket-Key: (.*)").unwrap();
    regex
        .captures(headers)
        .and_then(|caps| caps.get(1))
        .map(|m| String::from(String::from_utf8_lossy(m.as_bytes()).trim()))
}

fn send_back_handshake(stream: &mut TcpStream, key: String) {
    let key = key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let hash = base64::encode(Sha1::digest(key.as_bytes()));

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Connection: Upgrade\r\n\
         Upgrade: websocket\r\n\
         Sec-WebSocket-Accept: {}\r\n\r\n",
        hash
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn receive_ws_messages(stream: &mut TcpStream) {
    loop {
        let mut msg_buf = [0; 1024];
        if stream.read(&mut msg_buf).is_ok() {
            if msg_buf[0] == 0 {
                break;
            }
            let opcode = msg_buf[0] % 16;
            if opcode == 1 {
                let payload_length = (msg_buf[1] % 128) as usize;
                let mask: Vec<u8> = msg_buf[2..=5].to_vec();
                let mut payload = Vec::<u8>::with_capacity(payload_length);
                for i in 0..payload_length {
                    payload.push(msg_buf[6 + i] ^ mask[i % 4]);
                }
                println!("Received: {}", String::from_utf8(payload).unwrap().trim());
            } else if opcode == 9 {
                println!("Pong");
                stream.write(&[138, 0]).unwrap();
                stream.flush().unwrap();
            } else {
                eprintln!("Unsupported opcode {}; ignoring.", opcode);
            }
        } else {
            break;
        }
    }
}
