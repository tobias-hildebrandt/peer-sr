use std::{net::{SocketAddr, TcpListener}, io::{Read, Write}};

fn main() -> Result<(), anyhow::Error> {
    println!("starting server");

    // create tcp socket bound to localhost:8888
    let listen = TcpListener::bind("127.0.0.1:8888".parse::<SocketAddr>()?)?;

    // incoming network buffer
    let mut buffer = [0u8;2048];

    // block until a client attempts to connect
    let (mut client1_conn, client1_addr) = listen.accept()?;

    println!("client connected from address: {}", client1_addr);

    // wait until the client sends us a message
    let len = client1_conn.read(&mut buffer)?;

    // decode bytes as utf-8
    let client1_message = std::str::from_utf8(&buffer[0..len])?;

    println!("client sent: {}", client1_message);

    // assume the client sent us their p2p port
    let p2p_port = client1_message.parse::<u16>()?;

    // calculate their P2P port via their IP + p2p port
    let client1_p2p_socket = SocketAddr::new(
        client1_addr.ip(),
        p2p_port,
    );

    // tell client1 we are done and they will be contacted by client2 eventually
    client1_conn.write(&"".as_bytes())?;

    // now we are completely done with client1
    drop(client1_conn);
    drop(client1_addr);

    // wait for client2 to connect
    let (mut client2_conn, _client2_addr) = listen.accept()?;

    // send them client1's p2p socket address (ip + port)
    client2_conn.write(&client1_p2p_socket.to_string().as_bytes())?;

    // we are done
    println!("server done");

    Ok(())
}
