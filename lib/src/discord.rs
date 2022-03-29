// use std::io::prelude::*;
#![allow(unused_imports)] // temp
use url::{Url, Host};
use std::io::{Read, Write};
use std::net::{SocketAddr, IpAddr, TcpListener, TcpStream, ToSocketAddrs, Shutdown};
use std::str::from_utf8;

pub fn test() {
    println!("test print");
}

pub fn create_connection(addr: String) {
    match TcpStream::connect(addr) {
        Ok(mut stream) => {
            print!("Connection Established");
            let mut data = [0 as u8; 6];
            match stream.read_exact(&mut data)
            {
                Ok(_) => {
                    let text = from_utf8(&data).unwrap();
                    println!("{}", text);
                }
                Err(e) => {
                    println!("Failed to read data {}", e);
                }
            }
        } 
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
}

pub fn run() {
    let _url = Url::parse("httpbin.org/get?project=azoth");
    // create_connection("httpbin.org/get?project=azoth".to_string());
    let host = "httpbin.org:80";
    let path = "/get?project=azoth";
    
    // establish connection
    let mut stream = TcpStream::connect(host).expect("ERORR: Could not connect to server");

    // build http message
    let mut req = String::new();
    req.push_str(format!("GET {} HTTP/1.1", path).as_str());
    req.push_str("\r\n");
    req.push_str(format!("Host: {}", host).as_str());
    req.push_str("\r\n");
    req.push_str("Connection: close");
    req.push_str("\r\n");
    req.push_str("\r\n");
    println!("request we sending is: \n\n{}\n\n", req);

    // sending the bytes
    let req_bytes: &[u8] = req.as_bytes();
    stream
        .write_all(req_bytes)
        .expect("ERROR: Writing bytes errored out");

    // read response back
    let mut res = String::new();
    stream
        .read_to_string(&mut res)
        .expect("ERROR: Failed to read response");
    println!("response received: \n\n{}\n\n", res);

    // clean up
    stream
        .shutdown(Shutdown::Both)   
        .expect("ERROR: Shutdown failed");

    test();
}
