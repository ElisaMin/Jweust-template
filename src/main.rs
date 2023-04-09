mod redirections;
mod var;

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
    println!("stdout");
    // exit::if_check_utf8();
    // std_set::init().unwrap();
    println!("•◘▬¨ŤlCęół♥☺☻0");
    unsafe {
        SetConsoleOutputCP(936 ).unwrap();
        println!("•◘▬¨ŤlCęół♥☺☻0");
        let h_console = CreateFileW(
            w!("CONOUT$"),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        ).unwrap();
        s!("hello").transform(|a| WriteConsoleW(h_console, a.as_bytes(), None, None).unwrap());
        w!("hello").transform(|a| WriteConsoleW(h_console, a.to_hstring().unwrap().to_string().into_bytes().as_slice(), None, None).unwrap());
        w!("hello").transform(|a| WriteConsoleW(h_console, a.to_hstring().unwrap().to_string().into_bytes().as_slice(), None, None).unwrap());
        w!("hello").transform(|a| WriteConsoleW(h_console, a.to_hstring().unwrap().to_string().into_bytes().as_slice(), None, None).unwrap());
        let text = "aaaaaaa".to_string()
            .encode_utf16().chain(Some(0))
            .map(|a| a as u8)
            .collect::<Vec<u8>>();
        WriteConsoleW(h_console, &text, None, None).unwrap()
            .transform(|a| ());
        ;
        WriteConsoleW(h_console, &text, None, None).unwrap();
        WriteConsoleW(h_console, &text, None, None).unwrap();
        WriteConsoleW(h_console, &text, None, None).unwrap();
        WriteConsoleW(h_console, &text, None, None).unwrap();
        exit(0)
    }
    close().unwrap();
    // panic!("here");
    // println!("file stdout");
    // close().unwrap();
    // println!("yield files"); //panic here
    // recovery();
    // exit::show(Error::new(std::io::ErrorKind::Other, "hello error"));
    // exit::if_check_utf8();
    println!("exit now");
    exit(0);



    println!("hello stdout");
    let out = Command::new("cmd")
        .args(&["/c" ,
            // "start", "cmd","/c",
            "ping",
            // "-t",
            "baidu.com"])
        .stdout(Stdio::inherit())
        // .output()
        // .unwrap();
        .spawn()
        .map_err(|e| {
            exit(show_(e).unwrap().0);
        })
        .unwrap();
    // out.wait().unwrap();


    panic!("hello panic");



    // let out = out.stdout.unwrap();
    // get buffer of stdout
    // let buffer = BufReader::new(out);

    // for l in buffer.lines() {
    //     println!("{}", l.unwrap());
    //     // GBK.decode(l.unwrap_or_default().as_bytes()).0
    //     //     .chars().for_each(|c| {
    //     //     print!("{}", c);
    //     // });
    // }
    // for s in buffer.split(b'\r') {
    //     encoding_rs::GBK.decode(s.unwrap().as_slice()).0
    //         .chars().for_each(|c| {
    //         print!("{}", c);
    //     });
    // }


    return;
}

mod std_set {
    use std::fs::{File, remove_file};
    use std::io::{Error, ErrorKind, Write};
    use std::{io, panic};
    use std::alloc::System;
    use std::fmt::format;
    use std::mem::transmute;
    use std::os::windows::io::{AsRawHandle, IntoRawHandle, RawHandle};
    use std::panic::catch_unwind;
    use std::path::Path;
    use std::ptr::hash;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use std::thread::Result;
    use encoding_rs::mem::check_utf8_for_latin1_and_bidi;
    use windows::h;
    use windows::Win32::Storage::FileSystem::{CREATE_ALWAYS, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_READ};
    use windows::Win32::System::Console::{GetStdHandle, SetStdHandle, STD_ERROR_HANDLE, STD_HANDLE, STD_OUTPUT_HANDLE};
    use crate::{convert, exit, Results};
    use crate::kotlin::ScopeFunc;

    static mut HANDLERS: Vec<HANDLE> = vec![];
    static mut HANDLERS_ORG: Vec<(STD_HANDLE,HANDLE)> = vec![];

    pub struct FileNames {
        pub panic: String,
        pub out: String,
        pub err: String,
        is_created:bool,
    }
    impl FileNames {

        pub fn get(prefix:&str) -> Results<String> { Ok(chrono::Local::now()
            .format("%Y%m%d%H%M%S")
            .to_string()
            .transform(|s|
                format!("{}{}.log", "test",prefix)
            )
        ) }
        pub fn new() -> Results<Self> {
            let panic = Self::get("err")?;
            let out = Self::get("out")?;
            let err = Self::get("err")?;
            Ok(Self { panic, out, err, is_created: false })
        }
        pub fn is_created(&self) -> bool {
            self.is_created
        }
        fn create_or_open(filename: &str) -> Results<File> {
            let file = Path::new(filename);
            if file.exists() {
                Ok(File::open(file)?)
            } else {
                Ok(File::create(file)?)
            }
        }
        pub fn create_all(&mut self) -> Results<()> {
            if !self.is_created {
                for filename in [&self.err,&self.out,&self.panic].iter() {
                    Self::create_or_open(filename)?;
                }
            }
            self.is_created = true;
            Ok(())
        }
        pub fn panic(&self) -> Results<File> {
            Self::create_or_open(&self.panic)
        }
        pub fn out(&self) -> Results<File> {
            Self::create_or_open(&self.out)
        }
        pub fn err(&self) -> Results<File> {
            Self::create_or_open(&self.err)
        }
    }

    pub fn system_err_or(msg: String) -> Error {
        let err = Error::last_os_error();
        if err.raw_os_error().unwrap_or(0) == 0 {
            Error::new(err.kind(), msg)
        } else {
            err
        }
    }
    unsafe fn check(from_std:&STD_HANDLE, other:&HANDLE) -> std::result::Result<HANDLE, Error> {
        for (i, (_,org)) in HANDLERS_ORG.iter().enumerate() {
            if !org.is_invalid() && !other.is_invalid() && org.0 ==  other.0 {
                HANDLERS_ORG.remove(i);
                HANDLERS_ORG.push((*from_std,*other));
                return Ok(*other);
            }
        }
        {
            let std = GetStdHandle(*from_std).unwrap();
            println!("handler {:?}",std);
            if std.is_invalid() {
                return Err(system_err_or(format!("std handle {} is not invalid", from_std.0)));
            } else {
                HANDLERS_ORG.push((*from_std,std))
            }
        }
        Ok(*other)
    }
    unsafe fn set(from_std:&STD_HANDLE, other:&HANDLE) -> Results<()> {
        let other = check(from_std, other).unwrap();
        if !SetStdHandle(*from_std, other).as_bool() {
            Err(system_err_or(format!("set std handle {} failed", from_std.0)).into())
        } else {
            HANDLERS.push(other);
            Ok(())
        }
    }
    unsafe fn set_by_file(from_std:&STD_HANDLE,file:File) -> Results<()> {
        let handler = file.into_raw_handle();
        set(from_std, &transmute(handler))
    }
    unsafe fn set_by_name(handler:STD_HANDLE, filename:String) -> Results<HANDLE> {
        CreateFileW(
            convert(filename)?,
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_READ,
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?.transform(|file|
            set(&handler,&file).map(|_|file)
        )
    }
    fn hook_panic(filename:String)-> Results<()> {
        catch_unwind(||{
            let panic_file = Path::new(&filename);
            if panic_file.exists(){
                remove_file(panic_file).unwrap();
            }
            let panic_file: File = File::create(panic_file).unwrap();

            let hook = panic::take_hook();
            panic::set_hook(Box::new(move |info| {
                println!("panichs");
                let sys_err = Error::last_os_error();
                let err_info = format!(
                    "panic by : {} \nmore:\nlast system error : {}", info,sys_err
                );
                writeln!(&panic_file, "{}", &err_info).unwrap();
                recovery();
                eprintln!("{}",&err_info);
                hook(info);
                exit::show_err_(err_info).unwrap();
                exit(-1);
            }));
        }).map_err(|e| e.downcast::<Error>().unwrap().into())
    }

    pub fn init() -> Result<()> { catch_unwind(|| {
        let out = &FileNames::new().unwrap();
        catch_unwind(|| {
            hook_panic(
                FileNames::new().unwrap()
                    .transform(|out| {
                        out.panic().unwrap();
                        out.panic
                    })
            ).unwrap();
        }).unwrap();
        catch_unwind(|| {
            unsafe {
                set_by_file(&STD_OUTPUT_HANDLE, out.out().unwrap()).unwrap();
                set_by_file(&STD_ERROR_HANDLE, out.err().unwrap()).unwrap();
            };
        }).unwrap();
    }) }
    static mut REC:bool = false;

    pub fn close() -> Results<()> {
        unsafe {
            // if !REC {
            //     return Err(Error::new(ErrorKind::PermissionDenied,
            //         "close in recovery before"
            //     ).into())
            // }
            // REC = false;
            // ramdom handler


            let mut handlers_existing:Vec<HANDLE> = vec![];
            for (std,_) in HANDLERS_ORG.iter() {
                let the_handler = GetStdHandle(*std)?;
                if !the_handler.is_invalid() {
                    handlers_existing.push(the_handler);
                }
            }
            // println!("handlers_existing: {:?}", handlers_existing);
            // println!("handlersssss: {:?}", HANDLERS);
            // let hs = hs.as_mut_slice();
            for filed in handlers_existing {
                println!("handlers: {:?} start {:?} ", HANDLERS, filed);
                for it in HANDLERS.iter() {
                    println!("inside fors");
                    catch_unwind(||{
                        println!("inside unwind,{:?}", filed);
                        let a = CloseHandle(filed);
                        println!("after close ");
                            a
                            .inspect(|it| println!("close handle: {:?}", it))
                            .as_bool()
                            .inspect(|it| println!("close handle: {:?}", it));
                    }).transform(|it| {
                        println!("inside transform: {:?}", it);
                        it.is_ok()
                    });
                    if it.0 == filed.0  {
                        HANDLERS.retain(|removes| it.0 == removes.0);
                    }
                }
                println!("handlers: {:?} after {:?} ", HANDLERS, filed);
            }
            println!("handlers end ---: {:?}", HANDLERS);
        }
        Ok(())
    }
    pub fn recovery() {
        unsafe {
            let closable =  HANDLERS_ORG.clone();
            let closable = closable
                .iter()
                .filter(|(it,to)|
                    !SetStdHandle(*it, *to).as_bool()
                );
            HANDLERS_ORG.clear();
            HANDLERS_ORG.extend(closable);
            println!("recovery, handlers: {:?} ", HANDLERS_ORG);
            REC = true;
        }
    }
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
