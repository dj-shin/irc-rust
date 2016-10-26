extern crate openssl;
extern crate regex;

use std::io::prelude::*;
use std::net::TcpStream;
use openssl::ssl::{Ssl, SslStream, SslMethod, SslContext};
use regex::Regex;
use std::thread;
use std::io::{self, BufReader, BufWriter};

fn parse(msg: &str) {
    let re = Regex::new(r"(?:^[:cntrl:]*)(?::(([^@! ]*)(?:(?:!([^@]*))?@([^ ]*))?) )?([^ ]+)((?: [^: ][^ \r\n]*){0,14})(?: :?([^\r\n]*))?[:space:]*$").unwrap();
    println!("{}", msg.trim());
    if let Some(caps) = re.captures(msg) {
        for i in 1..caps.len() {
            if let Some(w) = caps.at(i) {
                print!("{:?} ", w);
            }
        }
        println!("");
    }
    else {
        println!("Unrecognized : {}", msg);
    }
}

fn irc_read(mut stream: BufReader<SslStream<TcpStream>>) {
    loop {
        let mut buf = vec![0; 2048];
        let resp = stream.read_until('\n' as u8, &mut buf);
        match resp {
            Ok(len) => {
                if len > 0 {
                    let msg = String::from_utf8(buf).unwrap();
                    parse(&msg);
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

fn irc_write(mut stream: BufWriter<SslStream<TcpStream>>) {
    let username = "rust-bot";
    let realname = "rust-bot";
    let nick = "테스트";

    let msg = format!("USER {name} {host} {user} :{realname}\n",
                      name=username, host=username, user=username, realname=realname);
    let _ = stream.write(msg.as_bytes());
    let msg = format!("NICK {nick}\n", nick=nick);
    let _ = stream.write(msg.as_bytes());
    let msg = format!("JOIN #{channel}\n", channel="마시마로");
    let _ = stream.write(msg.as_bytes());
    let _ = stream.flush();
}

fn main() {
    let ctx = SslContext::new(SslMethod::Sslv23).unwrap();
    let ssl = Ssl::new(&ctx).unwrap();

    let raw_stream = TcpStream::connect(("irc.uriirc.org", 16664)).unwrap();
    let stream = SslStream::connect(ssl, raw_stream).unwrap();

    let reader = BufReader::new(stream.try_clone().unwrap());
    let read_thread = thread::spawn(move || {
        irc_read(reader);
    });

    let writer = BufWriter::new(stream.try_clone().unwrap());
    let write_thread = thread::spawn(move || {
        irc_write(writer);
    });

    let _ = read_thread.join();
    let _ = write_thread.join();
}
