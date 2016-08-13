use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use super::error::Error;
use super::key_file::KeyFile;

pub struct DesktopEntry {
    name: String,
    generic_name: Option<String>,
    exec: String,
    working_dir: Option<String>,
    hidden: bool,
    no_display: bool,
}

impl DesktopEntry {
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<DesktopEntry, Error> {
        use std::io::prelude::*;
        use std::fs::File;

        let mut file = try!(File::open(path));
        let mut vec = Vec::new();
        try!(file.read_to_end(&mut vec));

        let kf = try!(KeyFile::read_bytes(&vec[..]));

        if Some("Application") != kf.get_default_string("Desktop Entry", "Type") {
            return Err(Error::Parse);
        }

        let name = try!(kf.get_default_string("Desktop Entry", "Name")
            .ok_or(Error::Parse));
        let exec = try!(kf.get_default_string("Desktop Entry", "Exec")
            .ok_or(Error::Parse));

        let generic_name = kf.get_default_string("Desktop Entry", "GenericName")
            .map(|s| s.to_owned());
        let working_dir = kf.get_default_string("Desktop Entry", "Path")
            .map(|s| s.to_owned());

        let hidden = kf.get_boolean("Desktop Entry", "Hidden") == Some(true);
        let no_display = kf.get_boolean("Desktop Entry", "NoDisplay") == Some(true);

        Ok(DesktopEntry {
            name: name.to_owned(),
            generic_name: generic_name,
            exec: exec.to_owned(),
            working_dir: working_dir,
            hidden: hidden,
            no_display: no_display,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn generic_name(&self) -> Option<&str> {
        self.generic_name.as_ref().map(String::as_ref)
    }

    pub fn exec(&self) -> &str {
        &self.exec
    }

    pub fn working_dir(&self) -> Option<&str> {
        self.working_dir.as_ref().map(String::as_ref)
    }

    pub fn visible(&self) -> bool {
        !self.hidden && !self.no_display
    }
}

pub fn find_desktop_files() -> Vec<PathBuf> {
    if let Ok(dirs) = ::xdg::BaseDirectories::new() {
        let mut paths = dirs.list_data_files_once("applications/");
        let extension = OsStr::new("desktop");

        paths.retain(|p| p.extension() == Some(extension));

        paths
    } else {
        Vec::new()
    }
}
