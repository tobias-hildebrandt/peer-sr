use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

/// Represents the signaling server
pub struct Server {
    /// Tcp socket that clients connect to, which causes a new socket to be spun off
    listener: TcpListener,
    /// Incoming network buffer
    buffer: [u8; 2048],
    /// Keeps track of client1's open P2P port
    client_p2p_socket: Option<SocketAddr>,
}

impl Server {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Server {
            listener: TcpListener::bind("127.0.0.1:8888".parse::<SocketAddr>()?)?,
            buffer: [0u8; 2048],
            client_p2p_socket: None,
        })
    }

    /// Blocks until a client connects, then responds to them and drops their connection
    pub fn listen(&mut self) -> Result<(), anyhow::Error> {
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
                client_conn.write_all("".as_bytes())?;
            }
            Some(sock) => {
                // we have a peer to give them!

                println!("sending and clearing stored client p2p socket ({})", sock);

                // send it to them
                client_conn.write_all(sock.to_string().as_bytes())?;

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

/// Represents a p2p client before connection
pub struct Client {
    /// Incoming network buffer
    buffer: [u8; 2048],
    /// Listening TCP socket for incoming peer connection
    peer_listen_socket: TcpListener,
}

/// Represents a connected p2p client
pub struct ConnectedClient {
    /// Incoming network buffer
    buffer: [u8; 2048],
    /// TCP connection to peer
    peer_connection: TcpStream,
}

impl Client {
    pub fn new(port: u16) -> Result<Self, anyhow::Error> {
        let peer_listen_socket = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port,
        ))?;
        Ok(Self {
            buffer: [0u8; 2048],
            peer_listen_socket,
        })
    }

    pub fn real_port(&self) -> Result<u16, anyhow::Error> {
        Ok(self.peer_listen_socket.local_addr()?.port())
    }

    /// Connect to server and block until peer connection is made
    pub fn connect(mut self) -> Result<ConnectedClient, anyhow::Error> {
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
    pub fn send(&mut self, message: &[u8]) -> Result<(), anyhow::Error> {
        Ok(self.peer_connection.write_all(message)?)
    }

    pub fn receive(&mut self) -> Result<&str, anyhow::Error> {
        let _size = self.peer_connection.read(&mut self.buffer)?;
        Ok(std::str::from_utf8(&self.buffer)?)
    }
}
