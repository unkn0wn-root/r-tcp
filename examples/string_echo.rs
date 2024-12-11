use std::io::{Read, Write};
use std::net::TcpStream;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;

    let message_to_send = "Hello, Server!";
    let payload = message_to_send.as_bytes();

    let mut message = Vec::new();
    message.push(1u8);                              // Message Type (REQUEST)
    message.write_u32::<BigEndian>(1)?;            // Request ID
    message.push(2u8);                             // Op Code (STORE)
    message.write_u32::<BigEndian>(payload.len() as u32)?;  // Payload Length
    message.extend_from_slice(payload);            // Payload

    println!("Sending: {}", message_to_send);

    stream.write_all(&message)?;

    let mut response_type = [0u8; 1];
    stream.read_exact(&mut response_type)?;
    let mut op_code = [0u8; 1];
    stream.read_exact(&mut op_code)?;
    let length = stream.read_u32::<BigEndian>()?;

    let mut response_data = vec![0u8; length as usize];
    stream.read_exact(&mut response_data)?;

    println!("Received: {}", String::from_utf8_lossy(&response_data));

    Ok(())
}
