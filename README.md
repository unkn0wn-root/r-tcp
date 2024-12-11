
# Simple TCP Server in Rust

My implementation of TCP server in Rust. Made two implementations:
1. A standard implementation using tokio package
2. A raw implementation using system calls

## Usage

### Running the standard server

make sure you have .env file in place and then:

```bash
make run-server
```

### Testing the server

```bash
make chat
```

Type any message and press enter. The server will echo back your message.
Type "quit" to close the connection.
