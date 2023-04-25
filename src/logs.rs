use std::fs::{File, OpenOptions, remove_file};
use std::{io, panic};
use std::env::temp_dir;
use std::io::{Write};
use std::path::{PathBuf};
use chrono::Local;
use once_cell::sync::Lazy;
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK};
use crate::kotlin::ScopeFunc;
use crate::exit;
use crate::exit::message_box;

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
            .transform(|name| temp_dir().join(name) )
            .transform(|path| _log_file(path, is_overwrite))
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
        if let Ok(mut file) = "panic".as_append_log_file() {
            let info =  format!("new panic:\ninfo:\ntime:{},{:?}\n\n",Local::now().format(TIME_FORMAT), panic_info);
            let _ = file.write_all(info.as_bytes());
            let _ = file.flush();
            hook_before(panic_info);
        };{
            let panic_type = panic_info.location().map(|it|it.file()).unwrap_or_default();
            // let panic_type = format!(" panic-type:{:?}", panic_type);
            let title = format!("错误！{:?}", panic_type);
            let mut msg = panic_info.message().map(|m| m.to_string())
                .unwrap_or_default();
            if msg.is_empty() {
                msg = panic_info.payload().downcast_ref::<&str>()
                    .unwrap_or(&"").to_string();
            }
            let _ = message_box( msg,title ,MB_OK|MB_ICONERROR);
        };
        exit(1);
    }))
}