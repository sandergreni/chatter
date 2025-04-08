
use cs_server::epoller::Epoller;
use cs_server::log_it;
use chrono::Local;
use json::JsonValue;
use std::collections::HashMap;
use std::os::fd::RawFd;
use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(version, about, long_about = None)]
struct Args
{
    #[arg(short, long, default_value_t = 4242)]
    port: u16,
}

fn parse_json(buffer: &str) -> Option<JsonValue>
{
    log_it!("recv", buffer);

    match json::parse(buffer)
    {
        Ok(parsed) => Some(parsed),
        Err(error) =>
        {
            log_it!(error, buffer);
            None
        }
    }
}

fn strip_crlf(input: &mut String)
{
    if input.ends_with('\n') || input.ends_with('\r')
    {
        input.pop();
        if input.ends_with('\n')
        {
            input.pop();
        }
    }
}

fn get_username(json: & JsonValue, parent: &str) -> Result<String, &'static str>
{   
    if json[parent].is_object()
    {
        let user = &json[parent];
        return Ok(user["username"].to_string());
    }

    Err("Invalid JSON, cannot find username")
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
        let key = "request";
        if let Some(req_code) = json[key].as_str()
        {
            let ret = match req_code
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

    fn get_login_response(&self) -> Result<String, json::JsonError>
    {
        let mut users_arr = json::JsonValue::new_array();
        for (username, _fd) in &self.users
        {
            users_arr.push(username.as_str())?;
        }

        let mut json = json::JsonValue::new_object();
        json.insert("response", "login")?;
        json.insert("users", users_arr)?;

        Ok(json.dump())
    }

    fn handle_login(&mut self, json: JsonValue, connection_id: RawFd) -> (RawFd, String)
    {
        let mut ret: (RawFd, String) = (connection_id, String::new());
        match get_username(&json, "user")
        {
            Ok(username) =>
            {
                self.users.insert(username.to_string(), connection_id);
                ret.1 = match self.get_login_response()
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
        match get_username(&json, "recipient")
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

                    strip_crlf(&mut input);

                    if let Some(json) = parse_json(input.as_str())
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
