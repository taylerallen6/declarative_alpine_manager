use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub packages: Vec<String>, // e.g., ["git", "vim"]
                               // pub users: Vec<User>,
                               // pub services: HashMap<String, ServiceState>, // e.g., {"sshd": ServiceState { enabled: true, running: true }}
                               // pub networking: Networking,
                               // Add more: filesystem, security, etc.
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub shell: String,
    pub groups: Vec<String>,
    // hashed_password: Option<String>, etc.
}

#[derive(Deserialize, Debug)]
pub struct ServiceState {
    pub enabled: bool,
    pub running: bool,
}

#[derive(Deserialize, Debug)]
pub struct Networking {
    pub interfaces: HashMap<String, InterfaceConfig>,
}

#[derive(Deserialize, Debug)]
pub struct InterfaceConfig {
    pub ip: String, // e.g., "192.168.1.10/24"
                    // dhcp: bool, etc.
}
