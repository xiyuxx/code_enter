use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};

// this file contains functions which will CRUD file

// store the list of alias and path
const LANG_MAP: &str = "D:\\Tool\\enter\\lang_map.json";

// use by the shell to check if and where will jump
const GOAL_PATH: &str = "D:\\Tool\\enter\\goal_path.txt";

#[derive(Serialize, Deserialize, Debug)]
pub struct Alias {
    pub alias: String,
    pub path: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Map {
    pub alias_path: Vec<Alias>,
}

// 读取跳转命令和路径的对应map
pub fn get_lang_path() -> Map {
    let alias_path = fs::read_to_string(LANG_MAP).unwrap();
    serde_json::from_str(&alias_path).unwrap()
}

// 更新map
// update the map
pub fn set_lang_map(map: Map) {
    let mut f = File::create(LANG_MAP).unwrap();
    let map = serde_json::to_string(&map).unwrap();
    f.write_all(map.as_bytes()).unwrap();
    

    let f2 = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(GOAL_PATH)
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
pub fn set_path(route: &Alias) {
    let mut f = File::create(GOAL_PATH).unwrap();
    f.write_all(route.path.as_bytes()).unwrap();
    f.write_all("\ntrue".as_bytes()).unwrap();
}
