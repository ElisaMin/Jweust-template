use std::fs::{File, OpenOptions};
use std::{io, thread};
use std::mem::transmute;
use std::ops::Deref;
use std::os::windows::io::{AsRawHandle, IntoRawHandle, RawHandle};
use std::panic::catch_unwind;
use once_cell::sync::Lazy;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{CREATE_ALWAYS, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_READ};
use windows::Win32::System::Console::{GetStdHandle, SetStdHandle, STD_HANDLE, WriteConsoleOutputW};
use crate::kotlin::ScopeFunc;
use crate::{convert, Results};

static FILENAME_BODY: Lazy<String> = Lazy::new(|| chrono::Local::now()
    .format("%Y%m%d%H%M%S")
    .to_string()
    .transform(|s|
        "test".to_string()
        // format!("{}{}.log", "test",prefix)
    )
);
fn get_file_name(prefix:&str) -> String {
    format!("{}{}.log", FILENAME_BODY.as_str(),prefix)
}
fn create(prefix:&str) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .open(get_file_name(prefix))
}
pub fn file_err() -> Results<File> {
    static mut FILE_ERR: Lazy<File> = Lazy::new(|| create("err").unwrap());
    Ok(unsafe { FILE_ERR.deref() }.try_clone()?)
}
pub fn file_out() -> Results<File> {
    static mut FILE_OUT: Lazy<File> = Lazy::new(|| create("out").unwrap());
    Ok(unsafe { FILE_OUT.deref() }.try_clone()?)
}
pub fn file_panic() -> Results<File> {
    static mut FILE_PANIC: Lazy<File> = Lazy::new(|| create("err").unwrap());
    Ok(unsafe { FILE_PANIC.deref() }.try_clone()?)
}


// ```
// set_redirections block
#[cfg(target_os = "windows")]
pub fn redirect_io_handler_to(io:&STD_HANDLE, h:&HANDLE) -> thread::Result<bool> {
    catch_unwind(|| {
        unsafe {
            SetStdHandle(*io, *h).as_bool()
        }
    })
}
#[cfg(target_os = "windows")]
pub fn redirect_io_handler_to_file(io:&STD_HANDLE, file:&File) -> Results<HANDLE> {
    let file = file.try_clone()?;
    let handle = file.into_raw_handle();
    let handle = raw_to_handler(handle);
     redirect_io_handler_to(io, &handle)
         .map(|_| handle)
         .map_err(|e| e.downcast::<io::Error>().unwrap().into())
}
#[cfg(target_os = "windows")]
pub fn get_std_handle(io:&STD_HANDLE) -> windows::core::Result<HANDLE> {
    unsafe {
        GetStdHandle(*io)
    }
}
#[cfg(target_os = "windows")]
pub fn get_os_file_handle(filename:String) -> windows::core::Result<HANDLE> {
    unsafe {
        CreateFileW(
            convert(filename).unwrap(),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_READ,
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
}

#[cfg(target_os = "windows")]
pub fn close_handle(h:&HANDLE) -> thread::Result<bool> {
    catch_unwind(||{
        unsafe {
            CloseHandle(*h).as_bool()
        }
    })
}
#[cfg(target_os = "windows")]
pub fn raw_to_handler(raw:RawHandle) -> HANDLE {
    unsafe { transmute(raw) }
}
// pub unsafe  fn write_to_console() {
//     WriteConsoleOutputW()
// }


//
//
//
//
//
//
// // static FILE_ERR =
//
// fn
//
//
// static mut HANDLERS: Vec<HANDLE> = vec![];
// static mut HANDLERS_ORG: Vec<(STD_HANDLE,HANDLE)> = vec![];
//
// pub struct FileNames {
//     pub panic: String,
//     pub out: String,
//     pub err: String,
//     is_created:bool,
// }
// impl FileNames {
//
//     pub fn get(prefix:&str) -> Results<String> { Ok(chrono::Local::now()
//         .format("%Y%m%d%H%M%S")
//         .to_string()
//         .transform(|s|
//             format!("{}{}.log", "test",prefix)
//         )
//     ) }
//     pub fn new() -> Results<Self> {
//         let panic = Self::get("err")?;
//         let out = Self::get("out")?;
//         let err = Self::get("err")?;
//         Ok(Self { panic, out, err, is_created: false })
//     }
//     pub fn is_created(&self) -> bool {
//         self.is_created
//     }
//     fn create_or_open(filename: &str) -> Results<File> {
//         let file = Path::new(filename);
//         if file.exists() {
//             Ok(File::open(file)?)
//         } else {
//             Ok(File::create(file)?)
//         }
//     }
//     pub fn create_all(&mut self) -> Results<()> {
//         if !self.is_created {
//             for filename in [&self.err,&self.out,&self.panic].iter() {
//                 Self::create_or_open(filename)?;
//             }
//         }
//         self.is_created = true;
//         Ok(())
//     }
//     pub fn panic(&self) -> Results<File> {
//         Self::create_or_open(&self.panic)
//     }
//     pub fn out(&self) -> Results<File> {
//         Self::create_or_open(&self.out)
//     }
//     pub fn err(&self) -> Results<File> {
//         Self::create_or_open(&self.err)
//     }
// }
//
// pub fn system_err_or(msg: String) -> Error {
//     let err = Error::last_os_error();
//     if err.raw_os_error().unwrap_or(0) == 0 {
//         Error::new(err.kind(), msg)
//     } else {
//         err
//     }
// }
// unsafe fn check(from_std:&STD_HANDLE, other:&HANDLE) -> std::result::Result<HANDLE, Error> {
//     for (i, (_,org)) in HANDLERS_ORG.iter().enumerate() {
//         if !org.is_invalid() && !other.is_invalid() && org.0 ==  other.0 {
//             HANDLERS_ORG.remove(i);
//             HANDLERS_ORG.push((*from_std,*other));
//             return Ok(*other);
//         }
//     }
//     {
//         let std = GetStdHandle(*from_std).unwrap();
//         println!("handler {:?}",std);
//         if std.is_invalid() {
//             return Err(system_err_or(format!("std handle {} is not invalid", from_std.0)));
//         } else {
//             HANDLERS_ORG.push((*from_std,std))
//         }
//     }
//     Ok(*other)
// }
// unsafe fn set(from_std:&STD_HANDLE, other:&HANDLE) -> Results<()> {
//     let other = check(from_std, other).unwrap();
//     if !SetStdHandle(*from_std, other).as_bool() {
//         Err(system_err_or(format!("set std handle {} failed", from_std.0)).into())
//     } else {
//         HANDLERS.push(other);
//         Ok(())
//     }
// }
// unsafe fn set_by_file(from_std:&STD_HANDLE,file:File) -> Results<()> {
//     let handler = file.into_raw_handle();
//     set(from_std, &transmute(handler))
// }
// unsafe fn set_by_name(handler:STD_HANDLE, filename:String) -> Results<HANDLE> {
//     CreateFileW(
//         convert(filename)?,
//         FILE_GENERIC_WRITE.0,
//         FILE_SHARE_READ,
//         None,
//         CREATE_ALWAYS,
//         FILE_ATTRIBUTE_NORMAL,
//         None,
//     )?.transform(|file|
//         set(&handler,&file).map(|_|file)
//     )
// }
// fn hook_panic(filename:String)-> Results<()> {
//     catch_unwind(||{
//         let panic_file = Path::new(&filename);
//         if panic_file.exists(){
//             remove_file(panic_file).unwrap();
//         }
//         let panic_file: File = File::create(panic_file).unwrap();
//
//         let hook = panic::take_hook();
//         panic::set_hook(Box::new(move |info| {
//             println!("panichs");
//             let sys_err = Error::last_os_error();
//             let err_info = format!(
//                 "panic by : {} \nmore:\nlast system error : {}", info,sys_err
//             );
//             writeln!(&panic_file, "{}", &err_info).unwrap();
//             recovery();
//             eprintln!("{}",&err_info);
//             hook(info);
//             exit::show_err_(err_info).unwrap();
//             exit(-1);
//         }));
//     }).map_err(|e| e.downcast::<Error>().unwrap().into())
// }
//
// pub fn init() -> Result<()> { catch_unwind(|| {
//     let out = &FileNames::new().unwrap();
//     catch_unwind(|| {
//         hook_panic(
//             FileNames::new().unwrap()
//                 .transform(|out| {
//                     out.panic().unwrap();
//                     out.panic
//                 })
//         ).unwrap();
//     }).unwrap();
//     catch_unwind(|| {
//         unsafe {
//             set_by_file(&STD_OUTPUT_HANDLE, out.out().unwrap()).unwrap();
//             set_by_file(&STD_ERROR_HANDLE, out.err().unwrap()).unwrap();
//         };
//     }).unwrap();
// }) }
// static mut REC:bool = false;
//
// pub fn close() -> Results<()> {
//     unsafe {
//         // if !REC {
//         //     return Err(Error::new(ErrorKind::PermissionDenied,
//         //         "close in recovery before"
//         //     ).into())
//         // }
//         // REC = false;
//         // ramdom handler
//
//
//         let mut handlers_existing:Vec<HANDLE> = vec![];
//         for (std,_) in HANDLERS_ORG.iter() {
//             let the_handler = GetStdHandle(*std)?;
//             if !the_handler.is_invalid() {
//                 handlers_existing.push(the_handler);
//             }
//         }
//         // println!("handlers_existing: {:?}", handlers_existing);
//         // println!("handlersssss: {:?}", HANDLERS);
//         // let hs = hs.as_mut_slice();
//         for filed in handlers_existing {
//             println!("handlers: {:?} start {:?} ", HANDLERS, filed);
//             for it in HANDLERS.iter() {
//                 println!("inside fors");
//                 catch_unwind(||{
//                     println!("inside unwind,{:?}", filed);
//                     let a = CloseHandle(filed);
//                     println!("after close ");
//                     a
//                         .inspect(|it| println!("close handle: {:?}", it))
//                         .as_bool()
//                         .inspect(|it| println!("close handle: {:?}", it));
//                 }).transform(|it| {
//                     println!("inside transform: {:?}", it);
//                     it.is_ok()
//                 });
//                 if it.0 == filed.0  {
//                     HANDLERS.retain(|removes| it.0 == removes.0);
//                 }
//             }
//             println!("handlers: {:?} after {:?} ", HANDLERS, filed);
//         }
//         println!("handlers end ---: {:?}", HANDLERS);
//     }
//     Ok(())
// }
// pub fn recovery() {
//     unsafe {
//         let closable =  HANDLERS_ORG.clone();
//         let closable = closable
//             .iter()
//             .filter(|(it,to)|
//                 !SetStdHandle(*it, *to).as_bool()
//             );
//         HANDLERS_ORG.clear();
//         HANDLERS_ORG.extend(closable);
//         println!("recovery, handlers: {:?} ", HANDLERS_ORG);
//         REC = true;
//     }
// }