use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
// this file contains functions which will CRUD file

// store the list of alias and path
const TOOL_MAP: &str = "lang_map.json";

// use by the shell to check if and where will jump
const GOAL_PATH: &str = "goal_path.txt";

#[derive(Serialize, Deserialize, Debug)]
pub struct Alias {
    pub alias: String,
    pub path: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Map {
    pub alias_path: Vec<Alias>,
}

fn get_tool_map(dir: &Path) -> PathBuf {
    dir.join(TOOL_MAP)
}

fn get_goal_path(dir: &Path) -> PathBuf {
    dir.join(GOAL_PATH)
}

// 读取跳转命令和路径的对应map
pub fn get_lang_path(dir: &Path) -> Map {
    let tool_map = get_tool_map(dir);
    let alias_path = match fs::read_to_string(&tool_map) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let empty_map = Map { alias_path: vec![] };
            let json = serde_json::to_string_pretty(&empty_map).unwrap();
            fs::write(&tool_map, &json).expect("Unable to initialize lang_map.json");
            json
        }
        Err(error) => panic!("Unable to read {}: {error}", tool_map.display()),
    };
    serde_json::from_str(&alias_path)
        .unwrap_or_else(|error| panic!("Unable to parse {}: {error}", tool_map.display()))
}

// 更新map
// update the map
pub fn set_lang_map(map: Map, dir: &Path) {
    let tool_map = get_tool_map(dir);
    let map = serde_json::to_string(&map).unwrap();
    fs::write(&tool_map, map)
        .unwrap_or_else(|error| panic!("Unable to write {}: {error}", tool_map.display()));

    reset_jump_state(dir);
}

// 设置跳转路径
// set the path to jump
pub fn set_path(route: &Alias, dir: &Path) {
    let goal_path = get_goal_path(dir);
    let content = format!("{}\ntrue", route.path);
    fs::write(&goal_path, content)
        .unwrap_or_else(|error| panic!("Unable to write {}: {error}", goal_path.display()));
}

fn reset_jump_state(dir: &Path) {
    let goal_path = get_goal_path(dir);
    let current_path = fs::read_to_string(&goal_path)
        .ok()
        .and_then(|content| content.lines().next().map(str::to_owned))
        .unwrap_or_default();
    let content = format!("{current_path}\nfalse");
    fs::write(&goal_path, content)
        .unwrap_or_else(|error| panic!("Unable to write {}: {error}", goal_path.display()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "code_enter_{name}_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn initializes_map_in_the_requested_directory() {
        let dir = test_dir("initialize_map");

        let map = get_lang_path(&dir);

        assert!(map.alias_path.is_empty());
        assert!(dir.join(TOOL_MAP).is_file());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn updating_map_initializes_and_resets_jump_state() {
        let dir = test_dir("reset_jump");
        let map = Map {
            alias_path: vec![Alias {
                alias: "rust".into(),
                path: r"C:\Code\Rust".into(),
            }],
        };

        set_lang_map(map, &dir);
        assert_eq!(fs::read_to_string(dir.join(GOAL_PATH)).unwrap(), "\nfalse");

        fs::write(dir.join(GOAL_PATH), "C:\\Code\\Rust\ntrue\ntrue").unwrap();
        set_lang_map(Map { alias_path: vec![] }, &dir);
        assert_eq!(
            fs::read_to_string(dir.join(GOAL_PATH)).unwrap(),
            "C:\\Code\\Rust\nfalse"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn setting_path_replaces_previous_jump_state() {
        let dir = test_dir("set_path");
        fs::write(dir.join(GOAL_PATH), "old\nfalse\nfalse").unwrap();
        let route = Alias {
            alias: "project".into(),
            path: r"D:\Code\Project".into(),
        };

        set_path(&route, &dir);

        assert_eq!(
            fs::read_to_string(dir.join(GOAL_PATH)).unwrap(),
            "D:\\Code\\Project\ntrue"
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
