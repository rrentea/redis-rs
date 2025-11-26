use std::io::BufReader;
use std::io;
use std::net::TcpListener;

mod resp;
use resp::parse_request;


fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let mut reader = BufReader::new(_stream);
                match parse_request(&mut reader) {
                    Ok(value) => println!("Value: {:?}", value),
                    Err(e) => println!("{}", e)
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        };
    }
    Ok(())
}
