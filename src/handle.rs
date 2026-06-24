use crate::file_reader::{set_lang_map, set_path, Alias, Map};
use crate::{command::Commands, file_reader};
use std::env;
use std::path::Path;

pub fn handle(cmd: &Commands) {
    let dir = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| env::current_dir().expect("Unable to determine the data directory"));

    // alias-path对应表
    let map = file_reader::get_lang_path(&dir);
    match cmd {
        Commands::Add { alias, path } => {
            add(map, alias, path, &dir);
        }
        Commands::Del { alias } => {
            delete(map, alias, &dir);
        }
        Commands::Ed { alias, path } => {
            edit(map, alias, path, &dir);
        }
        Commands::Jp { alias } => {
            jump(map, alias, &dir);
        }
        Commands::List => {
            list(map);
        }
    }
}

fn add(mut map: Map, alias: &str, path: &str, dir: &Path) {
    let new_map = Alias {
        alias: alias.to_owned(),
        path: path.to_owned(),
    };
    map.alias_path.push(new_map);

    set_lang_map(map, dir);
    println!("{}'s path had been added~", alias);
}

fn delete(mut map: Map, alias: &str, dir: &Path) {
    map.alias_path.retain(|lan| lan.alias != alias);
    set_lang_map(map, dir);
    println!("{} has been deleted success!", alias);
}

fn edit(mut map: Map, alias: &str, path: &str, dir: &Path) {
    // 判断是否有该元素
    let mut has = false;
    map.alias_path.iter_mut().for_each(|route| {
        if route.alias == alias {
            route.path = path.to_owned();
            has = true;
        }
    });
    // 若有则修改
    if has {
        set_lang_map(map, dir);
        println!("{alias}'s path has benn edited to {path}");
    }
    // 若无则调用add函数添加
    else {
        println!("Didn't find {alias}, ready to add it");
        add(map, alias, path, dir);
    }
}

fn jump(map: Map, alias: &str, dir: &Path) {
    map.alias_path.iter().for_each(|route| {
        if alias == route.alias {
            set_path(route, dir);
        }
    })
}

fn list(map: Map) {
    println!("Available maps are as follows~");
    if map.alias_path.is_empty() {
        return;
    }
    map.alias_path.into_iter().for_each(|route| {
        println!("{:<6} : {}", route.alias, route.path);
    })
}
