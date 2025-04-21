
use std::{io::{Read, Write}, net::TcpStream};

use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(version, about, long_about = None)]
struct Args
{
    hostname: String,
    #[arg(short, long, default_value_t = 4242)]
    port: u16,
    #[arg(short, long)]
    username: String,
}

fn convert(raw_buffer: [u8; 4096]) -> String
{
    match String::from_utf8(raw_buffer.to_vec())
    {
        Ok(s) => return s,
        Err(error) => return format!("Error while converting from utf8: {}", error)
    }
}

fn read(stream: &mut TcpStream) -> String
{
    let mut raw_buffer: [u8; 4096] = [0u8; 4096];
    match stream.read(&mut raw_buffer)
    {
        Ok(bytes_read) =>
        {
            if bytes_read > 0
            {
                return convert(raw_buffer);
            }
            else
            {
                return format!("{} bytes read from socket", bytes_read);
            }
        },
        Err(error) =>
        {
            return format!("Unable to read from socket: {}", error);
        }
    }
}

fn client_main(mut stream: TcpStream, args: Args)
{
    let login_json = r#"{"request":"login","user":{"username":""#.to_owned() + &args.username + r#""}}"#;
    if let Err(error) = stream.write_all(login_json.as_bytes())
    {
        println!("Unable to write to socket: {error}");
    }

    println!("{}", read(& mut stream));
}

fn main()
{
    let args = Args::parse();

    match TcpStream::connect(args.hostname.clone() + ":" + &args.port.to_string())
    {
        Ok(stream) => client_main(stream, args),
        Err(error) => println!("Unable to connect to {0}:{1}, {2}", args.hostname, args.port, error),
    }
}
