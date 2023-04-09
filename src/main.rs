mod redirections;
mod var;
mod jvm;

use std::io::Error;
use std::process::{Command, exit, Stdio};
use windows::core::PCWSTR;
use windows::{s, w};
use windows::Win32::Storage::FileSystem::{CreateFileA, CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_GENERIC_WRITE, FILE_SHARE_MODE, OPEN_EXISTING};
use windows::Win32::System::Console::{SetConsoleOutputCP, WriteConsoleOutputW, WriteConsoleW};
use crate::exit::{show_};
use crate::kotlin::ScopeFunc;
use crate::redirections::get_os_file_handle;
use crate::std_set::{close, recovery};

type Results<T> = Result<T,Box<dyn std::error::Error>>;

fn convert(s: String) -> Results<PCWSTR> {
    let w: Vec<u16> = s.encode_utf16().chain(Some(0)).collect();
    PCWSTR::from_raw(w.as_ptr())
        .transform(Ok)
}

fn main() {

}

mod exit {
    use std::io::Error;
    use std::process::{Command, exit};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, MessageBoxW, wsprintfW};
    use crate::{convert, Results};
    use crate::kotlin::ScopeFunc;

    pub fn show_err_(msg:String) -> Results<MESSAGEBOX_RESULT> {
        message_box(msg, "错误".to_string(), MB_OK | MB_ICONERROR)
    }
    pub fn show_(err:Error) -> Results<MESSAGEBOX_RESULT> {
        show_err_(format!("Error: {}", err))
    }
    pub fn after_show(err:Error) {
        show_(err).unwrap();
        exit(-1);
    }
    pub fn if_check_utf8() {
        let utf8 = Command::new("chcp.com")
            .args(["65001"])
            // .stdout(Stdio::inherit())
            .output();
        if let Err(err) = utf8 {
            exit(show_(err).unwrap().0);
        } else if let Ok(utf8) = utf8 {
            if !utf8.status.success() {
                exit(show_(Error::last_os_error()).unwrap().0);
            }
        }
    }
    pub fn message_box(message:String, title:String, u_type:MESSAGEBOX_STYLE) -> Results<MESSAGEBOX_RESULT> {
        unsafe {
            let message = convert(message)?;
            println!("{:?}", message);
            MessageBoxW(
                HWND::default(),
                message,
                convert(title)?,
                u_type
            ).transform(Ok)
        }
    }

}


mod kotlin {
    pub trait ScopeFunc: Sized {
        fn transform<F, R>(self, f: F) -> R where F: FnOnce(Self) -> R, { f(self) }
        fn modify<F>(mut self, f: F) -> Self where F: FnOnce(&mut Self), { f(&mut self);self }
        fn inspect<F>(self, f: F) -> Self where F: FnOnce(&Self), { f(&self);self }
    }

    impl<T> ScopeFunc for T {}
}
