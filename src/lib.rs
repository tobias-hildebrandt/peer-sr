use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

/// Byte that delineates messages
const MESSAGE_END: u8 = 0;

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
    /// Create a new server
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

        println!("client sent (as their p2p port number): {}", client_message);

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
    read_buffer: [u8; 2048],
    /// Listening TCP socket for incoming peer connection
    peer_listen_socket: TcpListener,
}

/// Represents a connected p2p client
pub struct ConnectedClient {
    /// Incoming network buffer
    read_buffer: [u8; 2048],
    /// Outgoing networking buffer
    write_buffer: [u8; 2048],
    /// TCP connection to peer
    peer_connection: TcpStream,
}

impl Client {
    /// Create a new client, not yet connected to anything
    pub fn new(port: u16) -> Result<Self, anyhow::Error> {
        // bind to p2p socket
        let peer_listen_socket = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port,
        ))?;
        Ok(Self {
            read_buffer: [0u8; 2048],
            peer_listen_socket,
        })
    }

    /// Return the port on which our client is listening for P2P connections
    pub fn real_port(&self) -> Result<u16, anyhow::Error> {
        Ok(self.peer_listen_socket.local_addr()?.port())
    }

    /// Connect to server and block until peer connection is made
    pub fn connect(mut self) -> Result<ConnectedClient, anyhow::Error> {
        // get the port that the OS gave us
        let real_p2p_port = self.peer_listen_socket.local_addr()?.port();

        println!("my p2p port is {}", real_p2p_port);

        // connect to server
        let mut server_connection = TcpStream::connect("127.0.0.1:8888".parse::<SocketAddr>()?)?;

        // send the server our real p2p port
        server_connection.write_all(real_p2p_port.to_string().as_bytes())?;

        // wait for the server to respond
        let len = server_connection.read(&mut self.read_buffer)?;

        // have to do different things if we are the first or second client
        let peer_connection = if len == 0 {
            // server has told us we are client1
            println!("i am client1, waiting for connection from client2");

            // wipe buffer
            self.read_buffer.fill(0);

            // wait for client2 to connect to our p2p port
            let (peer_connection, peer_address) = self.peer_listen_socket.accept()?;

            println!("connected to {}", peer_address);

            peer_connection
        } else {
            // server has told us we are client2
            let peer_address: SocketAddr = std::str::from_utf8(&self.read_buffer[0..len])?.parse()?;

            // wipe buffer
            self.read_buffer.fill(0);

            println!("i am client2");

            // connect to peer
            let peer_connection = TcpStream::connect(peer_address)?;

            println!("connected to {}", peer_address);

            peer_connection
        };

        Ok(ConnectedClient {
            // re-use read buffer
            read_buffer: self.read_buffer,
            // new write buffer
            write_buffer: [0u8; 2048],
            // transfer our p2p connection
            peer_connection,
        })
    }
}

impl ConnectedClient {
    /// Send a message to our peer
    pub fn send(&mut self, message: String) -> Result<(), anyhow::Error> {

        let message_bytes = message.as_bytes();
        let message_bytes_len = message.len();

        // copy only message bytes into write buffer
        self.write_buffer[0..message_bytes_len].copy_from_slice(message_bytes);

        // add terminating byte
        self.write_buffer[message_bytes_len] = MESSAGE_END;

        // write *only* the message + the terminating byte
        Ok(self.peer_connection.write_all(&self.write_buffer[0..message_bytes_len+1])?)
    }

    /// Block until we receive a message from our peer
    pub fn receive(&mut self) -> Result<&str, anyhow::Error> {

        // read bytes in one by one until we encounter terminator
        let mut byte_count = 0;
        let mut current_byte = [0u8];
        loop {
            // blocks until we read in a byte
            self.peer_connection.read_exact(&mut current_byte)?;

            // the byte we just read
            let new_byte = current_byte[0];

            match new_byte {
                // we have hit the end
                MESSAGE_END => {
                    // return str representation of the message
                    return Ok(std::str::from_utf8(&self.read_buffer[0..byte_count])?);
                }
                // we have to read more
                _ => {
                    // copy the byte to our buffer
                    self.read_buffer[byte_count] = new_byte;
                    // increment our count
                    byte_count += 1;

                }
            }
        }
    }
}
