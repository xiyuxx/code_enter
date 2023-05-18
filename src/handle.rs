use crate::file_reader::{set_lang_map, set_path, Alias, Map};
use crate::{command::Commands, file_reader};

pub fn handle(cmd: &Commands) {
    // alias-path对应表
    let map = file_reader::get_lang_path();
    match cmd {
        Commands::Add { alias, path } => {
            add(map, alias, path);
        }
        Commands::Del { alias } => {
            delete(map, alias);
        }
        Commands::Ed { alias, path } => {
            edit(map, alias, path);
        }
        Commands::Jp { alias } => {
            jump(map, alias);
        }
        Commands::List => {
            list(map);
        }
    }
}
fn add(mut map: Map, alias: &String, path: &String) {
    let new_map = Alias {
        alias: alias.clone(),
        path: path.clone(),
    };
    map.alias_path.push(new_map);

    set_lang_map(map);
    println!("{}'s path had been added~", alias);
}

fn delete(mut map: Map, alias: &String) {
    map.alias_path.retain(|lan| {
        if lan.alias == alias.as_str() {
            false
        } else {
            true
        }
    });
    set_lang_map(map);
    println!("{} has been deleted success!", alias);
}

fn edit(mut map: Map, alias: &String, path: &String) {
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
        set_lang_map(map);
        println!("{alias}'s path has benn edited to {path}");
    }
    // 若无则调用add函数添加
    else {
        println!("Didn't find {alias}, ready to add it");
        add(map, alias, path);
    }
}

fn jump(map: Map, alias: &String) {
    let key_lang = alias.as_str();
    map.alias_path.iter().for_each(|route| {
        if key_lang == route.alias {
            set_path(route);
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
