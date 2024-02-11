use bincode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub password: String,
    pub port_to_pub: u16,
    pub protocol: String,
}

impl Rule {
    pub fn new(name: String, password: String, port_to_pub: u16, protocol: String) -> Rule {
        Rule {
            name,
            password,
            port_to_pub,
            protocol,
        }
    }

    pub fn to_u8(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(&self)
    }
    pub fn from_u8(data: &Vec<u8>) -> Result<Rule, bincode::Error> {
        bincode::deserialize(&data)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
    pub fn from_json(data: String) -> serde_json::Result<Rule> {
        serde_json::from_str(&data)
    }
}
