extern crate fs_extra;

use fs_extra::dir::CopyOptions;
use fs_extra::dir::{copy, remove};

use std::env;

fn main() {
    let mut pwd = env::current_dir().unwrap();
    pwd.push("view");
    
    let options = CopyOptions::new();

    if cfg!(debug_assertions) {
        remove("target/debug/view").unwrap();
        copy(&pwd, "target/debug", &options).unwrap();
    } else {
        remove("target/release/view").unwrap();
        copy(&pwd, "target/release", &options).unwrap();
    }
}
