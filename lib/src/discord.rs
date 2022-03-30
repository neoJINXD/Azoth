// use std::io::prelude::*;
use std::io::{Read, Write};
use std::net::{SocketAddr, IpAddr, TcpListener, TcpStream, ToSocketAddrs, Shutdown};
// use error_chain::error_chain;
use reqwest::{Client, Request};

pub fn test() {
    println!("test print");
}

pub fn create_connection(host: &str, path: &str) {
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
}


pub async fn run(client : &Client) -> Result<(), Box<dyn std::error::Error>> {
    let res = client
        .get("http://httpbin.org/get?project=azoth")
        .header("key", "value")
        .send()
        .await?
        .text()
        .await?;
    println!("{:?}", res);
    Ok(())
}
