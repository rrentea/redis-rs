use std::io;
use std::io::{BufReader, Error, ErrorKind, Read, Write};
use std::net::TcpListener;

use crate::resp::Value;

mod resp;

enum Command {
    Ping,
    Command,
}

fn parse_command<R: Read>(mut reader: BufReader<R>) -> io::Result<Command> {
    let command_line: Value = resp::decode(&mut reader)?;
    match command_line {
        Value::Array(args) => match &args[0] {
            Value::SimpleString(s) if s.to_lowercase() == "ping" => Ok(Command::Ping),
            Value::SimpleString(s) if s.to_lowercase() == "command" => Ok(Command::Command),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Unknown command {:?}", args[0]),
            )),
        },
        value => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Unknown command {:?}", value),
        )),
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
                    Command::Ping => _stream
                        .write(Value::SimpleString("pong".to_string()).encode().as_slice())?,
                    Command::Command => _stream.write(
                        Value::Array(vec![Value::Map(vec![
                            (
                                Value::BulkString("name".into()),
                                Value::BulkString("ping".into()),
                            ),
                            (Value::BulkString("arity".into()), Value::Integer(1)),
                            (
                                Value::BulkString("flags".into()),
                                Value::Array(vec![Value::BulkString("fast".into())]),
                            ),
                            (Value::BulkString("first-key".into()), Value::Integer(0)),
                            (Value::BulkString("last-key".into()), Value::Integer(0)),
                            (Value::BulkString("step".into()), Value::Integer(0)),
                        ])])
                        .encode()
                        .as_slice(),
                    )?,
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        };
    }
    Ok(())
}
