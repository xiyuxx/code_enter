use crate::file_reader::{set_lang_map, set_path, Alias, Map};
use crate::{command::Commands, file_reader};
use std::env;
use std::path::PathBuf;
pub fn handle(cmd: &Commands) {
    // 文件目录
    let mut dir: PathBuf = Default::default();
    match env::current_exe(){
        Ok(path) => dir = path.parent().unwrap().to_path_buf(),
        _ => {}
    }
    // alias-path对应表
    let map = file_reader::get_lang_path(dir.clone());
    match cmd {
        Commands::Add { alias, path } => {
            add(map, alias, path, &mut dir);
        }
        Commands::Del { alias } => {
            delete(map, alias,&mut dir);
        }
        Commands::Ed { alias, path } => {
            edit(map, alias, path, &mut dir);
        }
        Commands::Jp { alias } => {
            jump(map, alias,&mut dir);
        }
        Commands::List => {
            list(map);
        }
    }
}
fn add(mut map: Map, alias: &String, path: &String,dir:&mut PathBuf) {
    let new_map = Alias {
        alias: alias.clone(),
        path: path.clone(),
    };
    map.alias_path.push(new_map);

    set_lang_map(map,dir);
    println!("{}'s path had been added~", alias);
}

fn delete(mut map: Map, alias: &String,dir:&mut PathBuf) {
    map.alias_path.retain(|lan| {
        if lan.alias == alias.as_str() {
            false
        } else {
            true
        }
    });
    set_lang_map(map,dir);
    println!("{} has been deleted success!", alias);
}

fn edit(mut map: Map, alias: &String, path: &String,dir:&mut PathBuf) {
    // 判断是否有该元素
    let mut has = false;
    map.alias_path.iter_mut().for_each(|route| {
        if route.alias == alias.as_str() {
            route.path = path.clone();
            has = true;
        }
    });
    // 若有则修改
    if has {
        set_lang_map(map,dir);
        println!("{alias}'s path has benn edited to {path}");
    }
    // 若无则调用add函数添加
    else {
        println!("Didn't find {alias}, ready to add it");
        add(map, alias, path,dir);
    }
}

fn jump(map: Map, alias: &String,dir:&mut PathBuf) {
    let key_lang = alias.as_str();
    map.alias_path.iter().for_each(|route| {
        if key_lang == route.alias {
            set_path(route,dir);
        }
    })
}

fn list(map: Map) {
    println!("Available maps are as follows~");
    if map.alias_path.len() == 0 {
        return;
    }
    map.alias_path.into_iter().for_each(|route| {
        println!("{:<6} : {}", route.alias, route.path);
    })
}
