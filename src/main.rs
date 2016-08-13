extern crate xdg;

#[macro_use]
extern crate nom;

pub mod error;
pub mod key_file;
pub mod desktop_entry;

use desktop_entry::*;

macro_rules!  errln(
    ($($arg:tt)*) => ({
        use std::io::Write;
        writeln!(&mut ::std::io::stderr(), $($arg)*).expect("failed to write to stderr");
    })
);

fn main() {
    for path in find_desktop_files() {
        if let Ok(de) = DesktopEntry::read_file(&path) {
            if de.visible() {
                println!("{}", de.name());
                println!(" - {}", de.exec());
                if let Some(wd) = de.working_dir() {
                    println!(" - {}", wd);
                }
            }
        } else {
            errln!("{}: failed to read desktop entry", path.display());
        }
    }
}
