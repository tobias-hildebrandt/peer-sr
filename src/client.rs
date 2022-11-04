use std::{net::{TcpStream, SocketAddr}, io::Write};

fn main() -> Result<(), anyhow::Error> {
    println!("starting client");

    // connect to the server
    let mut server_connection = TcpStream::connect("127.0.0.1:8888".parse::<SocketAddr>()?)?;

    // send the server a message
    server_connection.write(&"this is a message from the client!".as_bytes())?;

    Ok(())
}
