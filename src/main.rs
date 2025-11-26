use std::io::{BufReader, Error, ErrorKind, Read, Write};
use std::io;
use std::net::TcpListener;

use crate::resp::Value;

mod resp;

enum Command {
    Ping,
}

fn parse_command<R: Read>(mut reader: BufReader<R>) -> io::Result<Command> {
    let command_line: Value = resp::decode(&mut reader)?;
    match command_line {
        Value::Array(args) => {
            match &args[0] {
                Value::String(s) if s == "ping" => {
                    Ok(Command::Ping)
                }
                _ => Err(Error::new(ErrorKind::InvalidInput, format!("Unknown command {:?}", args[0])))
            }
        }
        value => Err(Error::new(ErrorKind::InvalidInput, format!("Unknown command {:?}", value)))
    }
}


fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let reader = BufReader::new(&_stream);
                let command = parse_command(reader)?;
                match command {
                    Command::Ping => _stream.write("+pong\r\n".as_bytes())?
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        };
    }
    Ok(())
}
