
pub mod json_stuff
{
    use std::{collections::HashMap, os::fd::RawFd};

    use json::JsonValue;

    pub fn parse_json(buffer: &str) -> Option<JsonValue>
    {
        //log_it!("recv", buffer);

        match json::parse(buffer)
        {
            Ok(parsed) => Some(parsed),
            Err(_) =>
            {
                //log_it!(error, buffer);
                None
            }
        }
    }

    pub fn get_username(json: &JsonValue, parent: &str) -> Result<String, &'static str>
    {   
        if json[parent].is_object()
        {
            let user = &json[parent];
            return Ok(user["username"].to_string());
        }

        Err("Invalid JSON, cannot find username")
    }

    pub fn get_request_code(json: &JsonValue) -> Result<String, &'static str>
    {
        let key = "request";
        if json[key].is_string()
        {
            return Ok(json[key].to_string());
        }

        Err("Invalid JSON, cannot find username")
    }
    
    pub fn get_login_response(users: &HashMap<String, RawFd>) -> Result<String, json::JsonError>
    {
        let mut users_arr = json::JsonValue::new_array();
        for (username, _fd) in users
        {
            users_arr.push(username.as_str())?;
        }

        let mut json = json::JsonValue::new_object();
        json.insert("response", "login")?;
        json.insert("users", users_arr)?;

        Ok(json.dump())
    }
}
