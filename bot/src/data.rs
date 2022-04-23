use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaveData {
    pub github_users: Vec<GitLink>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitLink {
    pub discord_id: u64,
    pub github_username: String,
}

impl GitLink {
    pub fn new(id: u64, username: String) -> Self {
        Self {
            discord_id: id,
            github_username: username,
        }
    }
}

pub fn save_data(path: String, data: SaveData) -> bool {
    let json_data = serde_json::to_string_pretty(&data).unwrap();

    match std::fs::write(&path, json_data) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Failed to save file: {}", e);
            return false;
        }
    };

    true
}

pub fn load_data(path: String) -> (bool, SaveData) {
    let mut loaded_data = SaveData {
        github_users: Vec::new(),
    };

    let file_data = match std::fs::read_to_string(&path) {
        Ok(x) => x,
        Err(_) => {
            log::error!("File not found");
            return (false, loaded_data);
        }
    };
    loaded_data = serde_json::from_str(&file_data).unwrap();

    (true, loaded_data)
}
