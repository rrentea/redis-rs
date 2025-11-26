use std::io::BufRead;
use std::io::{BufReader, Error, ErrorKind, Read, Result};

#[derive(Debug)]
pub enum Value {
    /// Null bulk reply, `$-1\r\n`
    Null,
    /// Null array reply, `*-1\r\n`
    NullArray,
    /// For Simple Strings the first byte of the reply is "+".
    String(String),
    /// For Errors the first byte of the reply is "-".
    Error(String),
    /// For Integers the first byte of the reply is ":".
    Integer(i64),
    /// For Arrays the first byte of the reply is "*".
    Array(Vec<Value>),
}

fn parse_string(buf: &[u8]) -> Result<String> {
    String::from_utf8(buf.to_vec()).map_err(|err| Error::new(ErrorKind::InvalidInput, err))
}

fn parse_integer(buf: &[u8]) -> Result<i64> {
    let str_integer = parse_string(buf)?;
    (str_integer.parse::<i64>()).map_err(|err| Error::new(ErrorKind::InvalidInput, err))
}

fn is_crlf(a: u8, b: u8) -> bool {
    a == b'\r' && b == b'\n'
}

pub fn parse_request<R: Read>(reader: &mut BufReader<R>) -> Result<Value> {
    let mut res: Vec<u8> = Vec::new();
    reader.read_until(b'\n', &mut res)?;

    let len = res.len();
    let bytes = res[1..len - 2].as_ref();
    match res[0] {
        b'+' => parse_string(bytes).map(Value::String),
        b'-' => parse_string(bytes).map(Value::Error),
        b':' => parse_integer(bytes).map(Value::Integer),
        b'$' => {
            let length = parse_integer(bytes)?;
            if length == -1 {
                return Ok(Value::Null);
            }

            let mut buf: Vec<u8> = Vec::new();
            let length = length as usize;
            buf.resize(length + 2, 0);

            reader.read_exact(&mut buf)?;
            if !is_crlf(buf[length], buf[length + 1]) {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid CRLF: {:?}", buf),
                ));
            }

            buf.truncate(length);
            parse_string(buf.as_slice()).map(Value::String)
        }
        b'*' => {
            let length = parse_integer(bytes)?;
            if length == -1 {
                return Ok(Value::NullArray);
            }

            let mut elements: Vec<Value> = Vec::with_capacity(length as usize);
            for _ in 0..length {
                elements.push(parse_request(reader)?)
            }
            Ok(Value::Array(elements))
        }
        prefix => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Invalid RESP type: {:?}", prefix),
        )),
    }
}
