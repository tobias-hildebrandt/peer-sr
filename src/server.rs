use lib_p2p_sr::Server;

fn main() -> Result<(), anyhow::Error> {
    println!("starting server");

    let mut server = Server::new()?;

    // loop forever, handling clients
    loop {
        server.listen()?;
    }
}
