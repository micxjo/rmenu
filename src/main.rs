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
use std::collections::HashMap;

fn wrap_dmenu() -> Result<(), Error> {
    let mut apps: HashMap<String, String> = HashMap::new();

    let mut child = try!(Command::new("dmenu")
        .arg("-i")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn());

    {
        let stdin = child.stdin.as_mut().expect("child missing stdin");

        for path in find_desktop_files() {
            if let Ok(de) = DesktopEntry::read_file(&path) {
                if !de.visible() {
                    continue;
                }

                try!(writeln!(stdin, "{}", de.name()));
                apps.insert(de.name().to_owned(), de.exec().to_owned());
            } else {
                errln!("{}: failed to read desktop entry", path.display());
            }
        }
    }

    let output = try!(child.wait_with_output());
    let string = String::from_utf8(output.stdout).expect("dmenu returned bad utf8");

    if let Some(exec) = apps.get(string.trim()) {
        let exec = try!(exec.split_whitespace().nth(0).ok_or(Error::Parse));
        try!(Command::new(exec).spawn());
    }

    Ok(())
}

fn main() {
    wrap_dmenu().unwrap();
}
