#![windows_subsystem = "windows"]
#![feature(panic_info_message)]
#![feature(iter_collect_into)]
extern crate core;

mod logs;
mod var;
mod jvm;
mod charsets;
use std::env::{args, current_dir};
use std::fs::{create_dir_all};
use std::path::PathBuf;
use std::process::{exit};
use once_cell::sync::Lazy;
use windows::core::{ HSTRING, PCWSTR};
use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK};
use crate::exit::{if_check_utf8, message_box};
use crate::jvm::Jvm;
use crate::kotlin::ScopeFunc;
use crate::logs::hook_panic;
use crate::var::{APPLICATION_WITH_OUT_CLI, EXE_IS_INSTANCE, EXE_PRODUCT_NAME, WORKDIR};

type Results<T> = Result<T,Box<dyn std::error::Error>>;
fn wstr(s: String) -> (HSTRING, PCWSTR) {
    let h = HSTRING::from(s);
    let w = PCWSTR::from_raw(h.as_ptr());
    (h, w)
}
fn workdir()->PathBuf {
    static _WORKDIR: Lazy<PathBuf> = Lazy::new(|| {
        if let Some(path) = WORKDIR {
            let path = PathBuf::from(path);
            if path.exists() || create_dir_all(&path).is_ok() {
                return path;
            }
        };
        if let Some(path) = std::env::var_os("APPDATA") {
                let path = PathBuf::from(path).join("jweust").join(EXE_PRODUCT_NAME);
                if path.exists() || create_dir_all(path.clone()).is_ok() {
                    return path;
                }
            };
        current_dir().unwrap().canonicalize().unwrap()
    });
    _WORKDIR.clone()

}
fn main() -> Results<()> {
    // set_current_dir(workdir()).unwrap();
    // None means CLI enabling
    if let Some(cli_command) = APPLICATION_WITH_OUT_CLI {
        cli_command.is_some() && args().collect::<Vec<String>>().contains(&cli_command.unwrap().to_string())
    } else {
        true
    }.transform(|e| unsafe {
        if e { AllocConsole(); }
        else { FreeConsole(); };
    });
    if EXE_IS_INSTANCE {
        exit::if_instance_exist().unwrap();
    }
     exit::if_instance_exist().unwrap();
    if_check_utf8();
    hook_panic();
    let jvm = Jvm::create();
    match jvm {
        Ok(jvm) => {
            jvm.invoke().unwrap();
            Ok(())
        }
        Err(e) => {
            let e = format!("{e}");
            message_box(
                String::from(&e),
                String::from("JVM错误！"),
                MB_OK|MB_ICONERROR
            ).unwrap();
            panic!("{e}");
        }
    }
}

mod exit {
    #![allow(unused_imports)]
    #![allow(dead_code)]

    use std::{env, process};
    use std::io::Error;
    use std::panic::catch_unwind;
    use std::path::PathBuf;
    use std::process::{Command, exit};
    use windows::imp::CloseHandle;
    use windows::Win32::Foundation::{HWND};
    use windows::Win32::System::Console::SetConsoleCP;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowExW, FlashWindow, GetWindowThreadProcessId, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, MessageBoxW, SetForegroundWindow};
    use windows::Win32::System::ProcessStatus::{EnumProcesses, GetModuleFileNameExW};
    use crate::kotlin::ScopeFunc;
    use crate::{Results, wstr};
    use crate::var::CHARSET_PAGE_CODE;


    pub fn if_instance_exist() ->Results<()> {
        let selfs =  found_process_by_path(env::current_exe()?);
        if selfs.len() > 1 {
            // if let Some((pid,_)) = {
            //     let pid = process::id();
            //     selfs.iter().find(|(p,_)|!pid.eq(p))
            // } {
            //     unsafe {
            //         if let Some(handler) = find_window_by(*pid) {
            //             FlashWindow(handler,true);
            //             SetForegroundWindow(handler);
            //         }
            //     }
            // }
            message_box(
                "本应用为单例模式，检测到已经有一个实例运行，请检查。".to_string(),
                "单例模式".to_string(),
                MB_OK | MB_ICONINFORMATION
            )?;

            exit(0);
        }
        Ok(())
    }

    pub fn show_err_(msg:String) -> Results<MESSAGEBOX_RESULT> {
        message_box(msg, "错误".to_string(), MB_OK | MB_ICONERROR)
    }
    pub fn show_(err:Error) -> Results<MESSAGEBOX_RESULT> {
        show_err_(format!("Error: {}", err))
    }
    // pub fn after_show(err:Error) {
    //     show_(err).unwrap();
    //     exit(-1);
    // }
    pub fn if_check_utf8() {
        if let Some(page_code) =  CHARSET_PAGE_CODE {
            let _=catch_unwind(|| unsafe {
                SetConsoleCP(page_code.parse::<u32>().unwrap()).unwrap();
            });
            let utf8 = Command::new("chcp.com")
                .args([page_code])
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
    fn find_window_by(pid:u32) -> Option<HWND> {
        let mut hwnd:Option<HWND> = None;
        while let Some(handler) = unsafe { FindWindowExW(None, hwnd.as_ref(), None, None).into() } {
            hwnd = Some(handler);
            let mut _pid = 0u32;
            unsafe {
                GetWindowThreadProcessId(handler, Some(&mut _pid));
            }
            if pid == _pid {
                return Some(handler)
            }
        }
        None
    }

    fn all_process_and_path(process_ids : &'_ mut [u32; 1024]) -> impl Iterator<Item = (u32, String)> + '_ {
        {
            let mut bytes_returned = 0u32;
            unsafe {
                EnumProcesses(process_ids.as_mut_ptr(), process_ids.len() as u32 * 4, &mut bytes_returned);
            }
        };
        process_ids.iter()
            .filter(|pid| **pid != 0)
            .filter_map(|&pid| {
                unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) }
                    .map(|process_handle| (pid, process_handle))
                    .ok()
            })
            .filter(|(_,process_handle)| !process_handle.is_invalid())
            .map(|(pid,process_handle)| {
                let path = {
                    let mut process_path = [0u16; 1024];
                    let size = unsafe {
                        let size = GetModuleFileNameExW(
                            process_handle,
                            None,
                            &mut process_path,
                        );
                        CloseHandle(process_handle.0);
                        size as usize
                    };
                    String::from_utf16_lossy(&process_path[..size])
                };
                (pid,path)
            })


    }
    fn found_process_by_path(path: PathBuf) -> Vec<(u32, PathBuf)> {
        let path = path.as_path().canonicalize().unwrap();
        all_process_and_path(&mut  [0u32; 1024])
            .filter_map(|(pid,process_path)|
                PathBuf::from(process_path)
                    .as_path()
                    .canonicalize()
                    .map(|process_path|
                        (pid,process_path)
                    )
                    .ok()
            )
            .filter(|(_,process_path)| process_path
                .as_path().canonicalize().unwrap() == path
            ).collect::<Vec<_>>()
    }
}


mod kotlin {
    pub trait ScopeFunc: Sized {
        #[inline]
        fn transform<F, R>(self, f: F) -> R where F: FnOnce(Self) -> R, { f(self) }
        #[inline]
        fn modify<F>(mut self, f: F) -> Self where F: FnOnce(&mut Self), { f(&mut self);self }
        #[inline]
        fn inspect<F>(self, f: F) -> Self where F: FnOnce(&Self), { f(&self);self }
    }
    impl<T> ScopeFunc for T {}
}
