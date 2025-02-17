use log::warn;
use std::{
    env::var,
    path::{Path, PathBuf},
};

fn env_var(id: &str) -> Option<String> {
    var(id)
        .inspect_err(|e| {
            warn!("Could not read env var {}: {}", id, e);
        })
        .ok()
}

pub fn hostname() -> String {
    env_var("HOSTNAME").unwrap_or(String::from("0.0.0.0"))
}

pub fn port() -> String {
    env_var("PORT").unwrap_or(String::from("35963"))
}

pub fn datadir() -> PathBuf {
    env_var("DATADIR")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new("/var/lib/xnode-demo").to_path_buf())
}

pub fn reservationsdir() -> PathBuf {
    env_var("RESERVATIONSDIR")
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or(Path::new(&datadir()).join("reservation"))
}

pub fn reservationduration() -> u64 {
    env_var("RESERVATIONDURATION")
        .and_then(|s| {
            str::parse::<u64>(&s)
                .inspect_err(|e| {
                    warn!("Could not parse RESERVATIONDURATION to u64: {}", e);
                })
                .ok()
        })
        .unwrap_or(3600)
}

pub fn xnodes() -> Vec<String> {
    env_var("XNODES")
        .map(|d| d.split_whitespace().map(|s| s.to_owned()).collect())
        .unwrap_or(vec![])
}
