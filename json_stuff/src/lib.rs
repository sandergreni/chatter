
pub mod json_stuff
{
    use std::{collections::HashMap, os::fd::RawFd};
    use chrono;
    use json::JsonValue;

    pub fn parse_json(buffer: &str) -> Option<JsonValue>
    {
        common::log_it!("recv", buffer);

        match json::parse(buffer)
        {
            Ok(parsed) => Some(parsed),
            Err(error) =>
            {
                common::log_it!(error, buffer);
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
        for username in users.keys()
        {
            users_arr.push(username.as_str())?;
        }

        let mut json = json::JsonValue::new_object();
        json.insert("response", "login")?;
        json.insert("users", users_arr)?;

        Ok(json.dump())
    }
}

// ************************************************************************************************
// ********************************************* TESTS ********************************************
// ************************************************************************************************

#[cfg(test)]
mod tests
{
    use std::{collections::HashMap, os::fd::RawFd};

    use crate::json_stuff::{get_login_response, get_request_code, get_username, parse_json};

    #[test]
    fn test_parse_valid_json()
    {
        let valid_json_str = r#"{"valid":"json"}"#;
        let valid_json = parse_json(valid_json_str);
        assert!(valid_json.is_some());
        
        assert_eq!(valid_json.unwrap().to_string(), valid_json_str);
    }
    
    #[test]
    fn test_parse_invalid_json()
    {
        let valid_json_str = r#""invalid":"json""#;
        let valid_json = parse_json(valid_json_str);
        assert!(valid_json.is_none());
    }
    
    #[test]
    fn test_get_username_from_login()
    {
        let valid_json_str = r#"{"request":"login","user":{"username":"sander"}}"#;
        let valid_json = parse_json(valid_json_str);
        assert!(valid_json.is_some());

        match get_username(&valid_json.unwrap(), "user")
        {
            Ok(username) => assert_eq!(username, "sander"),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_get_username_from_message()
    {
        let valid_json_str = r#"{"request":"message","recipient":{"username":"sander"},"payload":"Hei Sander"}"#;
        let valid_json = parse_json(valid_json_str);
        assert!(valid_json.is_some());

        match get_username(&valid_json.unwrap(), "recipient")
        {
            Ok(username) => assert_eq!(username, "sander"),
            Err(_) => assert!(false),
        }
    }
    
    #[test]
    fn test_get_request_code()
    {
        let valid_json_str = r#"{"request":"message","recipient":{"username":"sander"},"payload":"Hei Sander"}"#;
        let valid_json = parse_json(valid_json_str);
        assert!(valid_json.is_some());

        match get_request_code(&valid_json.unwrap())
        {
            Ok(req_code) => assert_eq!(req_code, "message"),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_get_login_response()
    {
        let mut users: HashMap<String, RawFd> = HashMap::new();
        users.insert("sander".to_owned(), 1);
        users.insert("per".to_owned(), 2);

        match get_login_response(&users)
        {
            Ok(valid_json) => assert_eq!(r#"{"response":"login","users":["sander","per"]}"#, valid_json.as_str()),
            Err(_) => assert!(false)
        }
    }
}
