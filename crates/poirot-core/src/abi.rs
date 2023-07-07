use alloy_json_abi::JsonAbi;
use serde_json;
use std::{env, fs, path::Path};

pub fn load_all_jsonabis(relative_path: &str) -> Result<Vec<JsonAbi>, Box<dyn std::error::Error>> {
    let mut abis = Vec::new();

    let current_dir = env::current_dir()?;
    let dir_path = current_dir.join(relative_path);

    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            // Ensure the file is a .json file
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let json_abi: JsonAbi = serde_json::from_str(&content)?;
                abis.push(json_abi);
            }
        }
    }

    Ok(abis)
}
