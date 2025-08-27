use std::io::{Read,Write};
use std::net::TcpStream;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian}; // Add `byteorder` crate

pub fn read_length_prefix(stream: &mut TcpStream) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let len = stream.read_u32::<BigEndian>()?;
    let mut buffer = vec![0; len as usize];
    stream.read_exact(&mut buffer)?;
    Ok(buffer)
}

pub fn write_length_prefix(stream: &mut TcpStream, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    stream.write_u32::<BigEndian>(data.len() as u32)?;
    stream.write_all(data)?;
    Ok(())
}
