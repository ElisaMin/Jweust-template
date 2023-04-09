mod logs;
mod var;
mod jvm;


use std::process::{exit};
use windows::core::{ HSTRING, PCWSTR};
use crate::exit::{if_check_utf8};
use crate::logs::hook_panic;

type Results<T> = Result<T,Box<dyn std::error::Error>>;
fn wstr(s: String) -> (HSTRING, PCWSTR) {
    let h = HSTRING::from(s);
    let w = PCWSTR::from_raw(h.as_ptr());
    (h, w)
}

fn main() {
    if_check_utf8();
    hook_panic();
}

mod exit {
    use std::io::Error;
    use std::process::{Command, exit};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, MessageBoxW};
    use crate::kotlin::ScopeFunc;
    use crate::{Results, wstr};

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
        let title = wstr(title);
        let message = wstr(message);
        unsafe {
            MessageBoxW(
                HWND::default(),
                message.1,
                title.1,
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
