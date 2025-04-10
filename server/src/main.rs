
use json_stuff::json_stuff;
use common::log_it;
use common::util;
use epoller::Epoller;
use clap::Parser;
use json::JsonValue;
use std::collections::HashMap;
use std::os::fd::RawFd;

#[derive(Parser, Default, Debug)]
#[command(version, about, long_about = None)]
struct Args
{
    #[arg(short, long, default_value_t = 4242)]
    port: u16,
}


struct UserConnections
{
    users: HashMap<String, RawFd>,
}

impl UserConnections
{
    pub fn new() -> Self
    {
        Self{ users: HashMap::new(), }
    }

    pub fn handle_request(&mut self, json: JsonValue, connection_id: RawFd) -> (RawFd, String)
    {
        if let Ok(req_code) = json_stuff::get_request_code(&json)
        {
            let ret = match req_code.as_str()
            {
                "login" => self.handle_login(json, connection_id),
                "message" => self.handle_message(json, connection_id),
                &_ => 
                {
                    log_it!("Invalid request", req_code);
                    (connection_id, "Invalid request".to_string())
                }
            };

            return ret;
        }

        (connection_id, "Unable to parse JSON".to_string())
    }

    fn handle_login(&mut self, json: JsonValue, connection_id: RawFd) -> (RawFd, String)
    {
        let mut ret: (RawFd, String) = (connection_id, String::new());
        match json_stuff::get_username(&json, "user")
        {
            Ok(username) =>
            {
                self.users.insert(username.to_string(), connection_id);
                ret.1 = match json_stuff::get_login_response(&self.users)
                {
                    Ok(response) => response,
                    Err(error) => format!("Unable to generate JSON for login response: {:#?}", error)
                };
            },
            Err(error) => ret.1 = error.to_string()
        }

        ret
    }

    fn handle_message(&mut self, json: JsonValue, connection_id: RawFd) -> (RawFd, String)
    {
        let mut ret: (RawFd, String) = (connection_id, String::new());
        match json_stuff::get_username(&json, "recipient")
        {
            Ok(username) =>
            {
                if let Some(recipient_conn_id) = self.users.get(&username).cloned()
                {
                    if json["payload"].is_string()
                    {
                        ret.0 = recipient_conn_id;
                        ret.1 = json["payload"].to_string();
                    }
                }
                else
                {
                    ret.1 = format!("Recipient {} does not exists", username);
                }
            }
            Err(error) => ret.1 = error.to_string()
        }

        ret
    }
}

fn main()
{
    let args = Args::parse();

    let mut users = UserConnections::new();
    let mut epoller = Epoller::new(
        args.port,
        |connection_id, input|
        {
            match String::from_utf8(input)
            {
                Ok(mut input) =>
                {
                    log_it!("length", input.len());

                    util::strip_crlf(&mut input);

                    if let Some(json) = json_stuff::parse_json(input.as_str())
                    {
                        return users.handle_request(json, connection_id);
                    }
                }
                Err(error) =>
                {
                    log_it!("Unable to convert bytes to string", error);
                }
            }

            (connection_id, "Unable to convert bytes to string".to_string())
        });

    epoller.start();
}
