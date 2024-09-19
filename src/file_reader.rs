use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
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

fn get_tool_map(dir:&PathBuf) -> String {
    let mut path = dir.clone();
    path.push(TOOL_MAP);
    path.to_string_lossy().to_string()
}

fn get_goal_path(dir:&PathBuf) -> String {
    let mut path = dir.clone();
    path.push(GOAL_PATH);
    path.to_string_lossy().to_string()
}

// 读取跳转命令和路径的对应map
pub fn get_lang_path(mut dir: PathBuf) -> Map {
    let tool_map = get_tool_map(&mut dir);
    let alias_path = fs::read_to_string(tool_map).unwrap();
    serde_json::from_str(&alias_path).unwrap()
}

// 更新map
// update the map
pub fn set_lang_map(map: Map,dir:&mut PathBuf) {
    let tool_map = get_tool_map(dir);
    let mut f = File::create(tool_map).unwrap();
    let map = serde_json::to_string(&map).unwrap();

    let goal_path = get_goal_path(&mut dir.clone());
    f.write_all(map.as_bytes()).unwrap();
    

    let f2 = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(goal_path)
        .unwrap();

    let f_read = BufReader::new(&f2);
    let mut f_write = BufWriter::new(&f2);
    if let Some(if_jump) = f_read.lines().last() {
        if if_jump.unwrap() == "true" {
            f_write.write_all("\nfalse".as_bytes()).unwrap();
        }
    }
}

// 设置跳转路径
// set the path to jump
pub fn set_path(route: &Alias,dir:&mut PathBuf) {
    let goal_path = get_goal_path(dir);
    let mut f = File::create(goal_path.clone()).unwrap();

    f.write_all(route.path.as_bytes()).unwrap();
    f.write_all("\ntrue".as_bytes()).unwrap();
}
