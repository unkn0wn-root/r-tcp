use std::io::{self, Read, Write};
use std::net::TcpStream;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    let mut request_id = 1;

    println!("Connected to server. Type your messages (press Enter to send, Ctrl+C to quit):");

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // create and send message
        let mut message = Vec::new();
        message.push(1u8);  // Message Type (REQUEST)
        message.write_u32::<BigEndian>(request_id)?;  // Request ID
        message.push(2u8);  // Op Code (STORE)
        message.write_u32::<BigEndian>(input.len() as u32)?;  // Payload Length
        message.extend_from_slice(input.as_bytes());  // Payload

        stream.write_all(&message)?;

        // read response
        let mut response_type = [0u8; 1];
        stream.read_exact(&mut response_type)?;
        let _resp_id = stream.read_u32::<BigEndian>()?;
        let mut op_code = [0u8; 1];
        stream.read_exact(&mut op_code)?;
        let length = stream.read_u32::<BigEndian>()?;

        let mut response_data = vec![0u8; length as usize];
        stream.read_exact(&mut response_data)?;

        println!("Server response: {}", String::from_utf8_lossy(&response_data));

        request_id += 1;
    }
}
