use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
};

/// Represents the signaling server
struct Server {
    /// Tcp socket that clients connect to, which causes a new socket to be spun off
    listener: TcpListener,
    /// Incoming network buffer
    buffer: [u8; 2048],
    /// Keeps track of client1's open P2P port
    client_p2p_socket: Option<SocketAddr>,
}

impl Server {
    fn new() -> Result<Self, anyhow::Error> {
        Ok(Server {
            listener: TcpListener::bind("127.0.0.1:8888".parse::<SocketAddr>()?)?,
            buffer: [0u8; 2048],
            client_p2p_socket: None,
        })
    }

    /// Blocks until a client connects, then responds to them and drops their connection
    fn listen(&mut self) -> Result<(), anyhow::Error> {
        // block until a client attempts to connect
        let (mut client_conn, client_addr) = self.listener.accept()?;

        println!("client connected from address: {}", client_addr);

        // wait until the client sends us a message
        let len = client_conn.read(&mut self.buffer)?;

        // decode bytes as utf-8
        let client_message = std::str::from_utf8(&self.buffer[0..len])?;

        println!("client sent: {}", client_message);

        // assume the client sent us their p2p port
        let p2p_port = client_message.parse::<u16>()?;

        match self.client_p2p_socket {
            None => {
                // we don't have any other peer to give to the client,
                // so store their info and tell them to wait

                // calculate their P2P socket via their IP + p2p port
                let new_p2p_socket = SocketAddr::new(client_addr.ip(), p2p_port);

                println!("new stored client p2p socket is {:?}", new_p2p_socket);

                // store it
                self.client_p2p_socket = Some(new_p2p_socket);

                // tell client1 we are done and they will be contacted by client2 eventually
                client_conn.write(&"".as_bytes())?;
            }
            Some(sock) => {
                // we have a peer to give them!

                println!("sending and clearing stored client p2p socket ({})", sock);

                // send it to them
                client_conn.write(&sock.to_string().as_bytes())?;

                // clear it
                self.client_p2p_socket = None;
            }
        }

        // in either case, we don't need to keep their connection open
        // so let it get dropped
        println!("done with client {}\n", client_addr);

        Ok(())
    }
}

fn main() -> Result<(), anyhow::Error> {
    println!("starting server");

    let mut server = Server::new()?;

    // loop forever, handling clients
    loop {
        server.listen()?;
    }
}
