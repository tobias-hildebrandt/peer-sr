use std::{
    io::{Write, Read},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

fn main() -> Result<(), anyhow::Error> {
    println!("starting client");

    // command line arguments
    let arguments: Vec<_> = std::env::args().collect();

    if arguments.len() < 2 {
        eprintln!("usage: client <p2p_port>");
    }

    // port number that we will use to listen for a p2p connection
    let p2p_port: u16 = arguments.get(1).unwrap().parse()?;

    // incoming network buffer
    let mut buffer = [0u8;2048];

    // connect to the server
    let mut server_connection = TcpStream::connect("127.0.0.1:8888".parse::<SocketAddr>()?)?;

    // the socket that we will listen on for our peer
    let peer_listen_socket = TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        p2p_port,
    ))?;

    // send the server our p2p port
    server_connection.write(&p2p_port.to_string().as_bytes())?;

    // wait for the server to respond
    let len = server_connection.read(&mut buffer)?;

    if len == 0 {
        // server has told us we are client1
        println!("i am client1, waiting for connection from client2");

        // wipe buffer
        buffer.fill(0);

        // wait for client2 to connect to our p2p port
        let (mut peer_connection, peer_address) = peer_listen_socket.accept()?;

        println!("connection from client2: {}", peer_address);

        peer_connection.read(&mut buffer)?;

        let message = std::str::from_utf8(&buffer)?;

        println!("got message: {}", message);

        peer_connection.write("this is a response from client1".as_bytes())?;
    } else {
        // server has told us we are client2
        let peer_address: SocketAddr = std::str::from_utf8(&buffer[0..len])?.parse()?;

        // wipe buffer
        buffer.fill(0);

        println!("i am client2, sending message to client1: {}", peer_address);

        let mut peer_connection = TcpStream::connect(peer_address)?;

        peer_connection.write(&"this is a message from client2".as_bytes())?;

        peer_connection.read(&mut buffer)?;

        let message = std::str::from_utf8(&buffer)?;

        println!("got message: {}", message);
    }

    Ok(())
}
