extern crate openssl;
extern crate regex;

use std::io;
use std::io::prelude::*;
use std::net::{TcpStream,Shutdown};
use openssl::ssl::{Ssl, SslStream, SslMethod, SslContext, Error};
use regex::Regex;
use std::thread;
use std::sync::{Arc, Mutex};
use std::os::unix::io::{AsRawFd, FromRawFd};

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

fn irc_read(stream: Arc<Mutex<SslStream<TcpStream>>>) {
// fn irc_read(mut stream: SslStream<TcpStream>) {
    loop {
        let mut buf = vec![0; 2048];
        let mut stream = stream.lock().unwrap();
        let resp = stream.ssl_read(&mut buf);
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
                match e {
                    Error::WantRead(e) => {}
                    _ => {
                        println!("Read Error : {:?}", e);
                    }
                }
            }
        }
    }
}

fn irc_write(stream: Arc<Mutex<SslStream<TcpStream>>>) {
// fn irc_write(mut stream: SslStream<TcpStream>) {
    thread::sleep(Duration::new(3, 0));
    let msg = "QUIT\n";
    let mut stream = stream.lock().unwrap();
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
    let mut stream = SslStream::connect(ssl, raw_stream).unwrap();
    let _ = stream.get_mut().set_nonblocking(true);
    let stream = Arc::new(Mutex::new(stream));

    let read_stream = stream.clone();
    let read_thread = thread::spawn(move || {
        irc_read(read_stream);
    });

    let write_stream = stream.clone();
    let write_thread = thread::spawn(move || {
        irc_write(write_stream);
    });

    let _ = read_thread.join();
    let _ = write_thread.join();
}
