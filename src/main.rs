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

extern crate gtk;
use gtk::prelude::*;

fn gtk_path_entries() {
    gtk::init().expect("failed to initialize GTK");

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("rmenu");
    window.set_default_size(350, 140);
    window.set_modal(true);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let list_store = gtk::ListStore::new(&[gtk::Type::String]);
    let tree_view = gtk::TreeView::new_with_model(&list_store);

    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();
    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    tree_view.append_column(&column);

    let path_entries = find_path_entries().unwrap();
    for exe in path_entries.clone().keys() {
        let iter = list_store.append();
        list_store.set(&iter, &[0], &[exe]);
    }

    tree_view.connect_row_activated(move |_, tree_path, _| {
        let index = tree_path.get_indices()[0];
        let exe = path_entries.keys().nth(index as usize).unwrap();
        let exe_path = path_entries.get(exe).unwrap();
        Command::new(exe_path).spawn().unwrap();
        gtk::main_quit();
    });

    let scrolled = gtk::ScrolledWindow::new(None, None);
    scrolled.add(&tree_view);

    window.add(&scrolled);
    window.show_all();

    gtk::main();
}

fn main() {
    match env::args().nth(1).as_ref().map(String::as_ref) {
        Some("de") => dmenu_desktop_entries().unwrap(),
        Some("dp") => dmenu_path_entries().unwrap(),
        _ => gtk_path_entries(),
    }
}
