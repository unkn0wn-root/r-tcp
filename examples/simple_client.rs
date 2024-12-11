use tcp_server::client::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect("127.0.0.1:8083")?;

    println!("Sending ping...");
    let pong = client.ping()?;
    println!("Received: {}", pong);

    println!("\nStoring data...");
    client.store("hello", "world".as_bytes().to_vec())?;
    println!("Data stored successfully");

    println!("\nRetrieving data...");
    if let Some(value) = client.retrieve("hello")? {
        println!("Retrieved: {}", String::from_utf8_lossy(&value));
    }

    println!("\nListing all keys...");
    let keys = client.list()?;
    println!("Keys in storage: {:?}", keys);

    Ok(())
}
