use clap::Parser;
use lib_p2p_sr::Client;

/// Command line arguments for our client
#[derive(Parser)]
struct Args {
    /// port number that we will use to listen for a p2p connection
    /// defaults to 0, meaning "let the OS decide"
    #[arg(short, long, default_value_t = 0)]
    p2p_port: u16,

    /// How many messages should we expect to receive
    #[arg(short, long, default_value_t = 5)]
    receive: u32,

    /// How many messages should we sent
    #[arg(short, long, default_value_t = 5)]
    send: u32,

    /// Should we print out messages that we sent and receive?
    #[arg(short, long, default_value_t = true)]
    debug: bool
}

fn main() -> Result<(), anyhow::Error> {
    println!("starting client");

    // command line arguments
    let arguments = Args::parse();

    // make a new client
    let client = Client::new(arguments.p2p_port)?;

    let real_port = client.real_port()?;

    // connect to server then to peer
    let mut connected = client.connect()?;

    // send some messages
    for i in 0..arguments.send {
        connected.send(format!("message #{} from port {}", i, real_port))?;
        if arguments.debug {
            println!("sent message #{}", i);
        }
    }

    // wait until we receive some messages
    for i in 0..arguments.receive {
        let received = connected.receive()?;
        if arguments.debug {
            println!("received message #{}: \'{}\'", i, received);
        }
    }

    println!("done");

    Ok(())
}
