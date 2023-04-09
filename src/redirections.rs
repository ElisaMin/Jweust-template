use std::fs::{File, OpenOptions};
use std::{io, panic};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use once_cell::sync::Lazy;
use crate::kotlin::ScopeFunc;
use crate::Results;
use crate::exit;

const TIME_FORMAT:&str = "%Y%m%d%H%M%S";

static FILENAME_BODY: Lazy<String> = Lazy::new(|| Local::now()
    .format(TIME_FORMAT)
    .to_string()
    .transform(|s|
        "test".to_string()
    )
);
fn get_file_name(prefix:&str) -> String {
    format!("{}{}.log", FILENAME_BODY.as_str(),prefix)
}
fn open_log_(data:&str,is_prefix:bool) -> io::Result<File> {
    let path = if is_prefix { get_file_name(data) } else { data.to_string() };
    OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
}
#[inline]
fn open_log_or_prefix(path:Option<&PathBuf>,prefix:&str) -> io::Result<File> {
    if let Some(path) = path {
        open_log_(path.to_str().unwrap(),false)
            .unwrap_or(open_log_or_prefix(None,prefix)?)
    } else {
        open_log_(get_file_name(prefix).as_str(),true)?
    }.transform(Ok)
}
#[inline]
pub fn file_err(path:Option<&PathBuf>) -> Results<File> {
    Ok(open_log_or_prefix(path,"err")?)
}
#[inline]
pub fn file_out(path:Option<&PathBuf>) -> Results<File> {
    Ok(open_log_or_prefix(path,"out")?)
}
#[inline]
pub fn file_panic(path:Option<&PathBuf>) -> Results<File> {
    Ok(open_log_or_prefix(path,"panic")?)
}

pub fn hook_panic() {
    let hook_before = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let mut file = file_panic(None).unwrap();
        let info = format!("new panic at : {}.\n info:\n {:?}\n\n",Local::now().format(TIME_FORMAT), panic_info);
        file.write(info.as_bytes()).unwrap();
        file.flush().unwrap();
        exit::show_err_(info).unwrap();
        hook_before(panic_info);
        exit(1);
    }))
}