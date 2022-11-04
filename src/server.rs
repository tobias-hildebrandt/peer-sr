use std::{net::{SocketAddr, TcpListener}, io::Read};

fn main() -> Result<(), anyhow::Error> {
    println!("starting server");

    // create tcp socket bound to localhost:8888
    let listen = TcpListener::bind("127.0.0.1:8888".parse::<SocketAddr>()?)?;

    // incoming network buffer
    let mut buffer = [0u8;2048];

    // block until a client attempts to connect
    let (mut client_conn, client_addr) = listen.accept()?;

    println!("client connected from address: {}", client_addr);

    // wait until the client sends us a message
    client_conn.read(&mut buffer)?;

    // decode bytes as utf-8
    let client_message = std::str::from_utf8(&buffer)?;

    println!("client sent: {}", client_message);

    Ok(())
}
