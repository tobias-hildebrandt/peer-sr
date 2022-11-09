use lib_p2p_sr::Client;

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
