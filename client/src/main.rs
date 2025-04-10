
use std::{io::{BufRead, BufReader, Write}, net::TcpStream};

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

fn client_main(mut stream: TcpStream, args: Args)
{
    let login_json = r#"{"request":"login","user":{"username":""#.to_owned() + &args.username + r#""}}"#;
    if let Err(error) = stream.write_all(login_json.as_bytes())
    {
        println!("Unable to write to socket: {error}");
    }

    let buf_reader = BufReader::new(&stream);
    for line in buf_reader.lines()
    {
        match line
        {
            Ok(str) => println!("{str}"),
            Err(error) => println!("Error while reading socket: {error}")
        }
    }
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
