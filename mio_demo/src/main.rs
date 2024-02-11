use mio::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::ptr::read;

const SERVER: mio::Token = mio::Token(0);
const CLIENT: mio::Token = mio::Token(1);
fn main() {
    let mut listener = TcpListener::bind("127.0.0.1:8080".parse().unwrap()).unwrap();
    let mut poll = mio::Poll::new().unwrap();
    let mut events = mio::Events::with_capacity(1024);

    let mut tcp_list = vec![];
    poll.registry()
        .register(&mut listener, SERVER, mio::Interest::READABLE)
        .unwrap();

    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                SERVER => {
                    while let Ok((mut stream, _)) = listener.accept() {
                        let token = mio::Token(tcp_list.len() + 1);
                        poll.registry()
                            .register(&mut stream, token, mio::Interest::READABLE)
                            .unwrap();
                        tcp_list.push(stream);
                    }
                }
                token => {
                    let stream = tcp_list.get_mut(token.0 - 1).unwrap();
                    let mut buf = [0; 1024];
                    match stream.read(&mut buf) {
                        Ok(0) => {
                            println!("client disconnected");
                            poll.registry().deregister(stream).unwrap();
                        }
                        Ok(n) => {
                            println!("read {} bytes", n);
                            stream.write(&buf[..n]).unwrap();
                        }
                        Err(e) => {
                            println!("Error reading from socket; err = {:?}", e);
                        }
                    }
                    if let Err(e) = stream.read(&mut buf) {
                        println!("Error writing to socket; err = {:?}", e);
                    }
                }
            }
        }
    }
}
