use std::{fs, collections::HashMap, path::{Path, PathBuf}, os::unix::prelude::MetadataExt, thread, time::Duration, sync::{Arc, Mutex}};

use regex::Regex;

pub struct Stat {
    pub count: u64,
    pub unique_count: u64,
    pub size: u64,
    pub unique_size: u64,
    pub curr_path: String
}

pub enum SizeState {
    Lazy(PathBuf),
    Evaled(HashMap<String, (u64, u64)>)
}

fn dive(p: &Path, excluded: &Vec<Regex>, dic: &mut HashMap<u64, SizeState>, stat: &mut Arc<Mutex<Stat>>) {
    if excluded.iter().any(|r| r.is_match(p.as_os_str().to_str().unwrap())) {
        println!("Skipped {}", p.as_os_str().to_str().unwrap());
        return;
    }
    for path in fs::read_dir(p).unwrap() {
        let path = path.unwrap().path();
        let path = path.as_path();
        let m = fs::symlink_metadata(path);
        if m.is_err() {
            println!("Error on {}", path.as_os_str().to_str().unwrap());
            continue;
        }
        let m = m.unwrap();
        if m.is_symlink() {
            continue;
        } else if m.is_dir() {
            dive(path, excluded, dic, stat);
        } else {
            let mut stat = stat.lock().unwrap();
            stat.count += 1;
            if stat.count % 50 == 0 {
                stat.curr_path = String::from(path.as_os_str().to_str().unwrap());
            }
            stat.size += m.size();


            if dic.get(&m.size()).is_none() {
                dic.insert(m.size(), SizeState::Lazy(path.to_path_buf()));
            } else {
                let hash = sha256::try_digest(path);
                if hash.is_err() {
                    println!("Error on {}", path.as_os_str().to_str().unwrap());
                    continue;
                }
                let hash = hash.unwrap();
                let mut map = dic.get_mut(&m.size()).unwrap();
                let mut map = match map {
                    SizeState::Lazy(prev_path) => {
                        let hash_old = sha256::try_digest(prev_path);
                        if hash_old.is_err() {
                            println!("Error on {}", prev_path.as_os_str().to_str().unwrap());
                            continue;
                        }
                        let mut hm = HashMap::new();
                        hm.insert(hash_old, (0, 0));
                        hm
                    },
                    SizeState::Evaled(m) => m
                };
                match map.get(&hash) {
                    Some(p) => {
                        let (size, count) = p.clone();
                        map.insert(hash, (size, count + 1));
                    }
                    None => {
                        stat.unique_count += 1;
                        stat.unique_size += m.size();
                        map.insert(hash, (m.size(), 1));
                    }
                };
            }

        }
    }
}

fn human(size: u64) -> String {
    match size {
        ..=10240 => format!("{} b", size),
        ..=10485760 => format!("{} kb", size/1024),
        ..=10737418240 => format!("{} mb", size/1024/1024),
        _ => format!("{} gb", size/1024/1024/1024),
    }
}

fn main() {
    let args : Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        panic!("Specify path as argument");
    }
    let path = args[args.len()-1].clone();
    let mut excl : Vec<Regex> = Vec::new();
    for i in 1..(args.len()-1) {
        excl.push(Regex::new(args[i].as_str()).unwrap());
    }

    let stat_init : Stat = Stat { count: 0, unique_count: 0, size: 0, unique_size: 0, curr_path: String::from("/") };
    let stat_arc = Arc::new(Mutex::new(stat_init));
    let mut stat_arc_t = stat_arc.clone();
    let t = thread::spawn(move || {
        let mut map : HashMap<String, (u64, u64)> = HashMap::new();
        dive(&Path::new(path.as_str()), &excl, &mut map, &mut stat_arc_t);
    });
    println!("FC: File Count");
    println!("S: Size");
    println!("UFC: Unique File Count");
    println!("US: Unique Size");
    println!("R: Ratio of US to S");
    loop {
        thread::sleep(Duration::from_millis(500));
        let stat = stat_arc.lock().unwrap();
        let ratio = stat.unique_size as f64 / stat.size as f64;
        println!("{}", stat.curr_path);
        println!("FC: {}; S: {}; UFC: {}; US: {}; R: {:.4}", stat.count, human(stat.size), stat.unique_count, human(stat.unique_size), ratio);
        println!("");
        if t.is_finished() {
            break;
        }
    }
}
