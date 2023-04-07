use std::io::Error;
use std::process::{Command, exit, Stdio};
use windows::core::PCWSTR;
use crate::exit::{show};
use crate::kotlin::ScopeFunc;
use crate::std_set::{close, recovery};


type Results<T> = Result<T,Box<dyn std::error::Error>>;

fn convert(s: String) -> Results<PCWSTR> {
    let w: Vec<u16> = s.encode_utf16().chain(Some(0)).collect();
    PCWSTR::from_raw(w.as_ptr())
        .transform(Ok)
}

fn main() {
    println!("stdout");
    std_set::init().unwrap();
    println!("file stdout");
    close();
    println!("yield files"); //panic here
    recovery();
    exit::show(Error::new(std::io::ErrorKind::Other, "hello error"));
    exit::if_check_utf8();
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
            exit(show(e).unwrap().0);
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
    use std::io::{Error, Write};
    use std::{io, panic};
    use std::mem::transmute;
    use std::os::windows::io::{AsRawHandle, IntoRawHandle};
    use std::panic::catch_unwind;
    use std::path::Path;
    use std::ptr::hash;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use std::thread::Result;
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
            if org.0 ==  other.0 {
                HANDLERS_ORG.remove(i);
                HANDLERS_ORG.push((*from_std,*other));
                return Ok(*other);
            }
        }
        { // fixme: not work always
            let std = GetStdHandle(*from_std)?;
            if !std.is_invalid() {
                return  Err(system_err_or(format!("std handle {} is not invalid", from_std.0)));
            }
        }
        Ok(*other)
    }
    unsafe fn set(from_std:&STD_HANDLE, other:&HANDLE) -> Results<()> {
        let other = check(from_std, other)?;
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
                close();
                writeln!(&panic_file, "{}", info).unwrap();
                recovery();
                hook(info);
                exit::after_show(Error::last_os_error());
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
            };
        }).unwrap();
        catch_unwind(|| {
            unsafe {
                set_by_file(&STD_ERROR_HANDLE, out.err().unwrap()).unwrap();
            };
        }).unwrap();
    }) }

    pub fn close() {
        unsafe {
            for handler in HANDLERS.iter() {
                CloseHandle(*handler).unwrap();
            }
        }
    }
    pub fn recovery() {
        unsafe {
            for (from_std,handler) in HANDLERS_ORG.iter() {
                SetStdHandle(*from_std, *handler);
            }
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

    pub fn show(err:Error) -> Results<MESSAGEBOX_RESULT> {
        message_box(format!("Error: {}", err), "错误".to_string(), MB_OK | MB_ICONERROR)
    }
    pub fn after_show(err:Error) {
        show(err).unwrap();
        exit(-1);
    }
    pub fn if_check_utf8() {
        let utf8 = Command::new("chcp.com")
            .args(["65001"])
            // .stdout(Stdio::inherit())
            .output();
        if let Err(err) = utf8 {
            exit(show(err).unwrap().0);
        } else if let Ok(utf8) = utf8 {
            if !utf8.status.success() {
                exit(show(Error::last_os_error()).unwrap().0);
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
