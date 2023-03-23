use std::fs::File;
use sysinfo::ProcessExt;
use sysinfo::SystemExt;
mod art;
mod notify;
mod snotify;
mod track;

use crate::snotify::Snotify;

use std::fs::remove_file;
use sysinfo::System;

use std::path::Path;

use std::env;

use log::error;
use log::info;

fn main() {
    env_logger::init();
    info!("{}", "Starting Snotify.");

    let parent_dir = "/Snotify";
    let spotify_cache_dir = check_args(env::args());

    let dir = spotify_cache_dir.to_owned() + parent_dir;
    let display = Path::new(&dir).display();
    let test_file = dir.to_owned() + &String::from("/snotify.txt");

    info!("Spotify album art directory = {:?}", dir);
    info!("Testing r/w access");

    if !Path::new(&dir).exists() {
        match std::fs::create_dir(&dir) {
            Err(e) => {
                error!("Failed to create cache directory = {}", dir);
                panic!("Couldn't create cache directory {}, {}", display, e)
            }
            Ok(_file) => {
                info!("Spotify cache directory created successfully = {:?}.", dir);
                test_rw_access(&test_file)
            }
        };
    } else {
        test_rw_access(&test_file);
    }

    // test r/w access to cache directory

    Snotify::start_loop(dir);
}

fn test_rw_access(test_file: &String) {
    let file = match File::create(&test_file) {
        Err(why) => panic!("Couldn't create file = {}, {}", test_file, why),
        Ok(_file) => remove_file(test_file),
    };

    match file {
        Ok(_) => info!("Test passed."),
        Err(e) => panic!("Test failed = {}", e),
    }
}

fn check_args(args: env::Args) -> String {
    let args: Vec<String> = args.collect();
    let mut spotify_cache_dir = "";

    if args.len() == 1 {
        panic!("Usage snotify --cache [cache directory]");
    }

    if args.len() > 0 {
        if args[1] == "--cache" {
            if args[2].len() > 0 {
                spotify_cache_dir = &args[2];
            } else {
                spotify_cache_dir = "";
            }
        }
    }

    return spotify_cache_dir.to_string();
}

fn _check_process() -> bool {
    // First check if the Spotify process is running
    info!("Checking if the spotify process is running.");

    let sys = System::new_all();

    for (pid, process) in sys.processes() {
        if process.name() == "spotify" {
            info!("Process running pid = {}", pid);
            return true;
        }
    }

    return false;
}

fn _get_current_dir() -> std::path::PathBuf {
    let path = match env::current_dir() {
        Ok(path) => path,
        Err(e) => panic!("error cwd {}", e),
    };
    path
}
