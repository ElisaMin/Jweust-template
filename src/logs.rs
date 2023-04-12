use std::fs::{File, OpenOptions, remove_file};
use std::{io, panic};
use std::io::{Write};
use std::path::{PathBuf};
use chrono::Local;
use once_cell::sync::Lazy;
use crate::kotlin::ScopeFunc;
use crate::exit;

const TIME_FORMAT:&str = "%Y%m%d%H%M%S";

static FILENAME_BODY: Lazy<String> = Lazy::new(|| Local::now()
    .format(TIME_FORMAT)
    .to_string()
    // .transform(|_|
    //     "test".to_string()
    // )
);
pub trait LogFile {
    fn as_log_file(&self, is_overwrite: bool) -> io::Result<File>;
    #[inline]
    fn as_append_log_file(&self) -> io::Result<File> {
        self.as_log_file(false)
    }
}
impl LogFile for PathBuf {
    #[inline]
    fn as_log_file(&self, is_overwrite: bool) -> io::Result<File> {
        _log_file(self.clone(), is_overwrite)
    }
}
impl LogFile for &str {
    #[inline]
    fn as_log_file(&self, is_overwrite: bool) -> io::Result<File> {
        get_file_name(self)
            .transform(PathBuf::from)
            .transform(|s| _log_file(s,is_overwrite))
    }
}
#[inline]
pub fn get_file_name(prefix:&str) -> String {
    format!("{}-{}.log", FILENAME_BODY.as_str(),prefix)
}

#[inline]
pub fn _log_file(path:PathBuf,is_overwrite:bool) ->io::Result<File> {
    let path = path.as_path();
    if is_overwrite && path.exists() {
        remove_file(path)?;
    }
    OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)
}


pub fn hook_panic() {
    let hook_before = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let mut file = "panic".as_append_log_file().unwrap();
        let info = format!("new panic:\ninfo:\ntime:{},{:?}\n\n",Local::now().format(TIME_FORMAT), panic_info);
        file.write_all(info.as_bytes()).unwrap();
        file.flush().unwrap();
        exit::show_err_(info).unwrap();
        hook_before(panic_info);
        exit(1);
    }))
}