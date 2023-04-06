use std::process::{Command, exit, Stdio};
use windows::core::PCWSTR;
use crate::exit::{show};


type Results<T> = Result<T,Box<dyn std::error::Error>>;

fn convert(s: String) -> PCWSTR {
    let w: Vec<u16> = s.encode_utf16().chain(Some(0)).collect();
    PCWSTR::from_raw(w.as_ptr())
}

fn main() {
    std_set::init().unwrap();
    exit::if_check_utf8();



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
            exit(show(e).0);
        })
        .unwrap();
    // out.wait().unwrap();






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
    use std::panic;
    use std::path::Path;
    use std::process::{exit};
    use windows::Win32::Foundation::HANDLE;
    use std::thread::Result;
    use windows::Win32::Storage::FileSystem::{CREATE_ALWAYS, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_READ};
    use windows::Win32::System::Console::{SetStdHandle, STD_ERROR_HANDLE, STD_HANDLE, STD_OUTPUT_HANDLE};
    use crate::{convert, Results};
    use crate::kotlin::ScopeFunc;

    pub struct FileNames {
        pub panic: String,
        pub out: String,
        pub err: String,
    }
    impl FileNames {
        pub fn get(prefix:&str) -> Results<String> { Ok(chrono::Local::now()
            .format("%Y%m%d%H%M%S")
            .to_string()
            .transform(|s|
                format!("{}{}.log", s,prefix)
            )
        ) }
        pub fn new() -> Results<Self> {
            let panic = Self::get("err")?;
            let out = Self::get("out")?;
            let err = Self::get("err")?;
            Ok(Self { panic, out, err })
        }
    }
    unsafe fn set(handler:STD_HANDLE, filename:String) -> std::result::Result<HANDLE, Error> {
        CreateFileW(
            convert(filename),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_READ,
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?.transform(|file|
            (SetStdHandle(handler, file).as_bool(), file)
        ).transform(|(succeeds, file)| {
            if !succeeds {
                return Err(Error::last_os_error());
            } else { Ok(file) }
        })
    }
    fn hook_panic(filename:String)-> Results<()> {
        panic::catch_unwind(||{
            let panic_file = Path::new(&filename);
            if panic_file.exists(){
                remove_file(panic_file).unwrap();
            }
            let panic_file: File = File::create(panic_file).unwrap();

            let hook = panic::take_hook();
            panic::set_hook(Box::new(move |info| {
                writeln!(&panic_file, "{}", info).unwrap();
                hook(info);
                exit(-1);
            }));
        }).map_err(|e| e.downcast::<Error>().unwrap().into())
    }

    pub fn init() -> Result<()> { panic::catch_unwind(|| {
        let out = FileNames::new().unwrap();
        let (panic,out, err) = (out.panic,out.out,out.err) ;
        panic::catch_unwind(|| {
            hook_panic(panic.into()).unwrap();
        }).unwrap();
        panic::catch_unwind(|| {
            unsafe {
                set(STD_OUTPUT_HANDLE, out.into()).unwrap();
            };
        }).unwrap();
        panic::catch_unwind(|| {
            unsafe {
                set(STD_ERROR_HANDLE, err.into()).unwrap();
            };
        }).unwrap();
    }) }


}

mod exit {
    use std::io::Error;
    use std::process::{Command, exit};
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, MessageBoxW};
    use crate::convert;

    pub fn show(err:Error) -> MESSAGEBOX_RESULT {
        message_box(format!("Error: {}", err), "Error".to_string(), MB_OK|MB_ICONERROR)
    }
    pub fn after_show(err:Error) {
        show(err);
        exit(-1);
    }
    pub fn if_check_utf8() {
        let utf8 = Command::new("chcp.com")
            .args(&[ "65001"])
            // .stdout(Stdio::inherit())
            .output();
        if let Err(err) = utf8 {
            exit(show(err).0);
        } else if let Ok(utf8) = utf8 {
            if !utf8.status.success() {
                exit(show(Error::last_os_error()).0);
            }
        }
    }
    pub fn message_box(message:String, title:String, u_type:MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
        unsafe {
            let r = MessageBoxW(
                None,
                convert(message),
                convert(title),
                u_type
            );
            return r
        }
    }

}


mod kotlin {
    pub trait ScopeFunc: Sized {
        fn transform<F, R>(self, f: F) -> R where F: FnOnce(Self) -> R,
        {
            f(self)
        }
        fn modify<F>(mut self, f: F) -> Self where F: FnOnce(&mut Self), { f(&mut self);self }
        fn inspect<F>(self, f: F) -> Self where F: FnOnce(&Self), { f(&self);self }
    }

    impl<T> ScopeFunc for T {}
}
