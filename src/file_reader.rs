use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_DIR_NAME: &str = "code_enter";
const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    pub alias: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Map {
    pub alias_path: Vec<Alias>,
}

pub fn config_file_path() -> Result<PathBuf, String> {
    if let Ok(dir) = env::var("CODE_ENTER_CONFIG_DIR") {
        return Ok(PathBuf::from(dir).join(CONFIG_FILE_NAME));
    }

    let base_dir = platform_config_base_dir()?;
    Ok(base_dir.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME))
}

fn platform_config_base_dir() -> Result<PathBuf, String> {
    if cfg!(windows) {
        if let Ok(appdata) = env::var("APPDATA") {
            return Ok(PathBuf::from(appdata));
        }

        if let Ok(user_profile) = env::var("USERPROFILE") {
            return Ok(PathBuf::from(user_profile).join("AppData").join("Roaming"));
        }
    }

    if cfg!(target_os = "macos") {
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support"));
        }
    }

    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg_config_home));
    }

    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home).join(".config"));
    }

    Err("Unable to locate a user config directory".to_string())
}

pub fn load_map() -> Result<Map, String> {
    load_map_from(&config_file_path()?)
}

pub fn save_map(map: &Map) -> Result<(), String> {
    save_map_to(&config_file_path()?, map)
}

pub fn load_map_from(path: &Path) -> Result<Map, String> {
    if !path.exists() {
        let map = Map::default();
        save_map_to(path, &map)?;
        return Ok(map);
    }

    let content = fs::read_to_string(path)
        .map_err(|err| format!("Unable to read {}: {err}", path.display()))?;

    if content.trim().is_empty() {
        return Ok(Map::default());
    }

    serde_json::from_str(&content)
        .map_err(|err| format!("Unable to parse {}: {err}", path.display()))
}

pub fn save_map_to(path: &Path, map: &Map) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Unable to create {}: {err}", parent.display()))?;
    }

    let content = serde_json::to_string_pretty(map)
        .map_err(|err| format!("Unable to serialize config: {err}"))?;
    fs::write(path, format!("{content}\n"))
        .map_err(|err| format!("Unable to write {}: {err}", path.display()))
}

pub fn add_alias(map: &mut Map, alias: &str, path: &str) -> Result<(), String> {
    if map.alias_path.iter().any(|route| route.alias == alias) {
        return Err(format!("{alias} already exists. Use `ed` to update it."));
    }

    map.alias_path.push(Alias {
        alias: alias.to_string(),
        path: path.to_string(),
    });
    Ok(())
}

pub fn edit_alias(map: &mut Map, alias: &str, path: &str) -> bool {
    if let Some(route) = map.alias_path.iter_mut().find(|route| route.alias == alias) {
        route.path = path.to_string();
        true
    } else {
        map.alias_path.push(Alias {
            alias: alias.to_string(),
            path: path.to_string(),
        });
        false
    }
}

pub fn delete_alias(map: &mut Map, alias: &str) -> bool {
    let old_len = map.alias_path.len();
    map.alias_path.retain(|route| route.alias != alias);
    map.alias_path.len() != old_len
}

pub fn find_alias<'a>(map: &'a Map, alias: &str) -> Option<&'a Alias> {
    map.alias_path.iter().find(|route| route.alias == alias)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_path(test_name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        env::temp_dir()
            .join("code_enter_tests")
            .join(format!("{test_name}_{}_{}", std::process::id(), nonce))
            .join(CONFIG_FILE_NAME)
    }

    #[test]
    fn load_map_creates_empty_config() {
        let path = temp_config_path("load_map_creates_empty_config");

        let map = load_map_from(&path).unwrap();

        assert_eq!(map, Map::default());
        assert!(path.exists());
    }

    #[test]
    fn save_and_load_map_round_trip() {
        let path = temp_config_path("save_and_load_map_round_trip");
        let map = Map {
            alias_path: vec![Alias {
                alias: "rust".to_string(),
                path: "D:\\Code\\Rust".to_string(),
            }],
        };

        save_map_to(&path, &map).unwrap();

        assert_eq!(load_map_from(&path).unwrap(), map);
    }

    #[test]
    fn add_alias_rejects_duplicates() {
        let mut map = Map::default();

        add_alias(&mut map, "rust", "D:\\Code\\Rust").unwrap();
        let result = add_alias(&mut map, "rust", "D:\\Other");

        assert!(result.is_err());
        assert_eq!(map.alias_path.len(), 1);
    }

    #[test]
    fn edit_alias_updates_existing_and_adds_missing() {
        let mut map = Map::default();

        assert!(!edit_alias(&mut map, "rust", "D:\\Code\\Rust"));
        assert!(edit_alias(&mut map, "rust", "D:\\Code\\Rust2"));

        assert_eq!(find_alias(&map, "rust").unwrap().path, "D:\\Code\\Rust2");
        assert_eq!(map.alias_path.len(), 1);
    }

    #[test]
    fn delete_alias_reports_whether_it_removed_anything() {
        let mut map = Map::default();
        add_alias(&mut map, "rust", "D:\\Code\\Rust").unwrap();

        assert!(!delete_alias(&mut map, "missing"));
        assert!(delete_alias(&mut map, "rust"));
        assert!(map.alias_path.is_empty());
    }
}
