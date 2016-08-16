extern crate xdg;

#[macro_use]
extern crate nom;

pub mod error;
pub mod key_file;
pub mod desktop_entry;

use error::Error;
use desktop_entry::*;

macro_rules!  errln(
    ($($arg:tt)*) => ({
        use std::io::Write;
        writeln!(&mut ::std::io::stderr(), $($arg)*).expect("failed to write to stderr");
    })
);

use std::io::Write;
use std::process::{Command, Stdio};
use std::collections::BTreeMap;
use std::{env, fs};
use std::path::PathBuf;

fn run_dmenu(items: Vec<&str>) -> Result<String, Error> {
    let mut cmd = try!(Command::new("dmenu")
                       .arg("-i")
                       .stdin(Stdio::piped())
                       .stdout(Stdio::piped())
                       .spawn());

    {
        let stdin = cmd.stdin.as_mut().expect("child missing stdin");

        for item in items {
            try!(writeln!(stdin, "{}", item));
        }
    }

    let output = try!(cmd.wait_with_output());
    let string = String::from_utf8(output.stdout).expect("dmenu returned bad utf8");

    Ok(string.trim().to_owned())
}

fn find_path_entries() -> Result<BTreeMap<String, PathBuf>, Error> {
    let mut exes = BTreeMap::new();
    let paths = env::var_os("PATH").expect("no path");
    for path in env::split_paths(&paths) {
        for entry in try!(fs::read_dir(path)) {
            let entry = try!(entry);
            let path = try!(entry.path().canonicalize());
            if path.is_file() {
                let name = path.file_name().unwrap().to_str().unwrap().to_owned();
                if !exes.contains_key(&name) {
                    exes.insert(name, path);
                }
            }
        }
    }
    Ok(exes)
}

fn dmenu_path_entries() -> Result<(), Error> {
    let exes = try!(find_path_entries());
    let menu_items = exes.keys().map(String::as_ref).collect();
    let choice = try!(run_dmenu(menu_items));
    if let Some(path) = exes.get(&choice) {
        try!(Command::new(path).spawn());
    }
    Ok(())
}

fn dmenu_desktop_entries() -> Result<(), Error> {
    let mut exes = BTreeMap::new();

    for path in find_desktop_files() {
        if let Ok(de) = DesktopEntry::read_file(&path) {
            if !de.visible() {
                continue;
            }
            exes.insert(de.name().to_owned(), de.exec().to_owned());
        } else {
            errln!("{}: failed to read desktop entry", path.display());
        }
    }

    let menu_items = exes.keys().map(String::as_ref).collect();
    let choice = try!(run_dmenu(menu_items));
    if let Some(exec) = exes.get(&choice) {
        let exec = try!(exec.split_whitespace().nth(0).ok_or(Error::Parse));
        try!(Command::new(exec).spawn());
    }
    Ok(())
}

fn main() {
    if env::args().nth(1) == Some("run".to_owned()) {
        dmenu_path_entries().ok();
    } else {
        dmenu_desktop_entries().ok();
    }
}
