use std::io::BufRead;
use std::io::{BufReader, Error, ErrorKind, Read, Result};

#[derive(Debug)]
pub enum Value {
    Null,
    NullArray,
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(Vec<u8>),
    BulkError(Vec<u8>),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Set(Vec<Value>),
    Bool(bool),
}

impl Value {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Value::Null => vec![b'$', b'-', b'1', b'\r', b'\n'],
            Value::NullArray => vec![b'*', b'-', b'1', b'\r', b'\n'],
            Value::SimpleString(string) => {
                let mut res = vec![b'+'];
                string.bytes().for_each(|b| res.push(b));
                res.push(b'\r');
                res.push(b'\n');
                res
            }
            Value::SimpleError(string) => {
                let mut res = vec![b'-'];
                string.bytes().for_each(|b| res.push(b));
                res.push(b'\r');
                res.push(b'\n');
                res
            }
            Value::Integer(num) => {
                let mut res = vec![b':'];
                if *num < 0 {
                    res.push(b'-');
                }
                num.to_string().bytes().for_each(|b| res.push(b));
                res.push(b'\r');
                res.push(b'\n');
                res
            }
            Value::BulkString(string) => {
                let mut res = vec![b'$'];
                string.len().to_string().bytes().for_each(|b| res.push(b));
                res.push(b'\r');
                res.push(b'\n');
                res.extend(string);
                res.push(b'\r');
                res.push(b'\n');
                res
            }
            Value::BulkError(string) => {
                let length: usize = string.len();
                let mut res = vec![b'!'];
                length.to_string().bytes().for_each(|b| res.push(b));
                res.push(b'\r');
                res.push(b'\n');
                res.extend(string);
                res.push(b'\r');
                res.push(b'\n');
                res
            }
            Value::Array(values) => {
                let mut res = vec![b'*'];
                values.len().to_string().bytes().for_each(|b| res.push(b));
                res.push(b'\r');
                res.push(b'\n');
                values
                    .iter()
                    .map(|val| val.encode())
                    .for_each(|bytes| res.extend(bytes));
                res
            }
            Value::Map(maps) => {
                let mut res = vec![b'%'];
                maps.len().to_string().bytes().for_each(|b| res.push(b));
                res.push(b'\r');
                res.push(b'\n');
                for (key, value) in maps {
                    res.extend(key.encode());
                    res.extend(value.encode());
                }
                res
            }
            Value::Set(values) => {
                let mut res = vec![b'~'];
                values.len().to_string().bytes().for_each(|b| res.push(b));
                res.push(b'\r');
                res.push(b'\n');
                values
                    .iter()
                    .map(|val| val.encode())
                    .for_each(|bytes| res.extend(bytes));
                res
            }
            Value::Bool(b) => {
                let mut res = vec![b'#'];
                if *b {
                    res.push(b't');
                } else {
                    res.push(b'f');
                }
                res.push(b'\r');
                res.push(b'\n');
                res
            },
        }
    }
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

pub fn decode<R: Read>(reader: &mut BufReader<R>) -> Result<Value> {
    let mut res: Vec<u8> = Vec::new();
    reader.read_until(b'\n', &mut res)?;

    let len = res.len();
    let bytes = res[1..len - 2].as_ref();
    match res[0] {
        b'+' => parse_string(bytes).map(Value::SimpleString),
        b'-' => parse_string(bytes).map(Value::SimpleError),
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
            parse_string(buf.as_slice()).map(Value::SimpleString)
        }
        b'*' => {
            let length = parse_integer(bytes)?;
            if length == -1 {
                return Ok(Value::NullArray);
            }

            let mut elements: Vec<Value> = Vec::with_capacity(length as usize);
            for _ in 0..length {
                elements.push(decode(reader)?)
            }
            Ok(Value::Array(elements))
        }
        prefix => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Invalid RESP type: {:?}", prefix),
        )),
    }
}
