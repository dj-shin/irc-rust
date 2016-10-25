extern crate openssl;
extern crate regex;

use std::io::prelude::*;
use std::net::TcpStream;
use openssl::ssl::{Ssl, SslStream, SslMethod, SslContext};
use regex::Regex;
use std::thread;

use std::time::Duration;

fn parse(msg: &str) {
    let notice = Regex::new(r"^NOTICE (?P<nick>\S*) :(?P<text>.*)$").unwrap();
    // let privmsg = Regex::new(r"^PRIVMSG (?P<nick>\S*) :(?P<text>.*)$").unwrap();
    if let Some(caps) = notice.captures(msg) {
        println!("NOTICE : {} {}", &caps.name("nick").unwrap(), &caps.name("text").unwrap());
    } else {
        println!("Unrecognized message : {}", msg);
    }
}

fn irc_read(mut stream: SslStream<TcpStream>) {
    loop {
        let mut buf = vec![0; 2048];
        let resp = stream.ssl_read(&mut buf);
        println!("{:?}", resp);
        match resp {
            Ok(len) => {
                println!("Received({})", len);
                if len > 0 {
                    let msg = String::from_utf8(buf).unwrap();
                    for line in msg.split('\n') {
                        parse(&line);
                    }
                }
                else {
                    println!("Connection Closed");
                    break;
                }
            }
            Err(e) => {
                println!("Read Error : {:?}", e);
            }
        }
    }
}

fn irc_write(mut stream: SslStream<TcpStream>) {
    thread::sleep(Duration::new(5, 0));
    let msg = "QUIT\n";
    let res = stream.ssl_write(msg.as_bytes());
    let _ = stream.flush();
    match res {
        Ok(len) => {
            println!("Write({}) : {}", len, msg.trim());
        }
        Err(e) => {
            println!("Write Error : {:?}", e);
            return;
        }
    }
}

fn main() {
    let ctx = SslContext::new(SslMethod::Sslv23).unwrap();
    let ssl = Ssl::new(&ctx).unwrap();

    let raw_stream = TcpStream::connect(("irc.uriirc.org", 16664)).unwrap();
    let stream = SslStream::connect(ssl, raw_stream).unwrap();

    let read_stream = stream.try_clone().unwrap();
    let read_thread = thread::spawn(move || {
        irc_read(read_stream);
    });

    let write_stream = stream.try_clone().unwrap();
    let write_thread = thread::spawn(move || {
        irc_write(write_stream);
    });

    let _ = read_thread.join();
    let _ = write_thread.join();
}
