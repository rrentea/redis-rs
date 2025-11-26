use std::io;
use std::io::Write;
use std::io::Read;
use std::net::TcpListener;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let mut buffer: [u8;512] = [0;512];

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let n = _stream.read(&mut buffer)?;
                println!("Read: {:?}", &buffer[..n]);
                let written = _stream.write("+PONG\r\n".as_bytes())?;
                println!("Written: {}", written);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        };
    }
    Ok(())
}
