use std::io::{self, Read, Write};
use std::net::TcpStream;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    let mut request_id = 1;

    println!("Connected to server. Commands available:");
    println!("1. PING");
    println!("2. STORE <key> <value>");
    println!("3. RETRIEVE <key>");
    println!("4. DELETE <key>");
    println!("5. LIST");
    println!("(Press Ctrl+C to quit)");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let message = match parts[0].to_uppercase().as_str() {
            "PING" => {
                create_message(request_id, 1, b"ping")?  // OpCode::Ping
            }
            "STORE" => {
                if parts.len() < 3 {
                    println!("Usage: STORE <key> <value>");
                    continue;
                }
                let key = parts[1];
                let value = parts[2..].join(" ");
                // using your bincode serialization format
                let payload = bincode::serialize(&(key, value))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                create_message(request_id, 2, &payload)?  // OpCode::Store
            }
            "RETRIEVE" => {
                if parts.len() != 2 {
                    println!("Usage: RETRIEVE <key>");
                    continue;
                }
                let payload = bincode::serialize(&parts[1])
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                create_message(request_id, 3, &payload)?  // OpCode::Retrieve
            }
            "DELETE" => {
                if parts.len() != 2 {
                    println!("Usage: DELETE <key>");
                    continue;
                }
                let payload = bincode::serialize(&parts[1])
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                create_message(request_id, 4, &payload)?  // OpCode::Delete
            }
            "LIST" => {
                create_message(request_id, 5, &[])?  // OpCode::List
            }
            _ => {
                println!("Unknown command");
                continue;
            }
        };

        // send message
        stream.write_all(&message)?;

        // read response
        match read_response(&mut stream) {
            Ok((msg_type, resp_id, payload)) => {
                match msg_type {
                    2 => { // res
                        match String::from_utf8(payload) {
                            Ok(text) => println!("Success (ID: {}): {}", resp_id, text),
                            Err(_) => println!("Success (ID: {}): <binary data>", resp_id),
                        }
                    },
                    3 => { // error
                        println!("Error (ID: {}): {}", resp_id,
                            String::from_utf8_lossy(&payload));
                    },
                    _ => println!("Unknown response type: {}", msg_type),
                }
            }
            Err(e) => println!("Error reading response: {}", e),
        }

        request_id += 1;
    }
}

fn create_message(request_id: u32, op_code: u8, payload: &[u8]) -> io::Result<Vec<u8>> {
    let mut message = Vec::new();
    message.push(1u8);
    message.write_u32::<BigEndian>(request_id)?;
    message.push(op_code);
    message.write_u32::<BigEndian>(payload.len() as u32)?;
    message.extend_from_slice(payload);
    Ok(message)
}

fn read_response(stream: &mut TcpStream) -> io::Result<(u8, u32, Vec<u8>)> {
    let mut response_type = [0u8; 1];
    stream.read_exact(&mut response_type)?;

    let resp_id = stream.read_u32::<BigEndian>()?;

    let mut op_code = [0u8; 1];
    stream.read_exact(&mut op_code)?;

    let length = stream.read_u32::<BigEndian>()?;
    let mut response_data = vec![0u8; length as usize];
    stream.read_exact(&mut response_data)?;

    Ok((response_type[0], resp_id, response_data))
}
