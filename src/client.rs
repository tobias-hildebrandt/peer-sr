use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

/// Represents a p2p client before connection
struct Client {
    /// Incoming network buffer
    buffer: [u8; 2048],
    /// Listening TCP socket for incoming peer connection
    peer_listen_socket: TcpListener,
}

/// Represents a connected p2p client
struct ConnectedClient {
    /// Incoming network buffer
    buffer: [u8; 2048],
    /// TCP connection to peer
    peer_connection: TcpStream
}

impl Client {
    fn new(port: u16) -> Result<Self, anyhow::Error> {
        let peer_listen_socket = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port,
        ))?;
        Ok(Self {
            buffer: [0u8; 2048],
            peer_listen_socket,
        })
    }

    fn real_port(&self) -> Result<u16, anyhow::Error> {
        Ok(self.peer_listen_socket.local_addr()?.port())
    }

    /// Connect to server and block until peer connection is made
    fn connect(mut self) -> Result<ConnectedClient, anyhow::Error> {
        // get the port that the OS gave us
        let real_p2p_port = self.peer_listen_socket.local_addr()?.port();

        println!("my p2p port is {}", real_p2p_port);

        let mut server_connection = TcpStream::connect("127.0.0.1:8888".parse::<SocketAddr>()?)?;

        // send the server our real p2p port
        server_connection.write_all(real_p2p_port.to_string().as_bytes())?;

        // wait for the server to respond
        let len = server_connection.read(&mut self.buffer)?;

        if len == 0 {
            // server has told us we are client1
            println!("i am client1, waiting for connection from client2");

            // wipe buffer
            self.buffer.fill(0);

            // wait for client2 to connect to our p2p port
            let (peer_connection, peer_address) = self.peer_listen_socket.accept()?;

            println!("connected to {}", peer_address);

            Ok(ConnectedClient {
                buffer: self.buffer,
                peer_connection,
            })
        } else {
            // server has told us we are client2
            let peer_address: SocketAddr = std::str::from_utf8(&self.buffer[0..len])?.parse()?;

            // wipe buffer
            self.buffer.fill(0);

            println!("i am client2");

            let peer_connection = TcpStream::connect(peer_address)?;

            println!("connected to {}", peer_address);

            Ok(ConnectedClient {
                buffer: self.buffer,
                peer_connection,
            })
        }
    }
}

impl ConnectedClient {
    fn send(&mut self, message: &[u8]) -> Result<(), anyhow::Error> {
        Ok(self.peer_connection.write_all(message)?)
    }

    fn receive(&mut self) -> Result<&str, anyhow::Error> {
        let _size = self.peer_connection.read(&mut self.buffer)?;
        Ok(std::str::from_utf8(&self.buffer)?)
    }
}

fn main() -> Result<(), anyhow::Error> {
    println!("starting client");

    // command line arguments
    let arguments: Vec<_> = std::env::args().collect();

    // port number that we will use to listen for a p2p connection
    let p2p_port: u16 = match arguments.get(1) {
        // if user gives us one, parse it
        Some(port) => port.parse()?,
        // else use 0, which means "let the OS give us one"
        None => 0,
    };

    let client = Client::new(p2p_port)?;

    let real_port = client.real_port()?;

    let mut connected = client.connect()?;

    connected.send(format!("hello from {}", real_port).as_bytes())?;

    let received = connected.receive()?;

    println!("received message: {}", received);

    Ok(())
}
