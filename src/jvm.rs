use std::cmp::Ordering;
use std::env::{args, var};
use std::fs::File;
use std::{io, thread};
use std::fmt::{Debug, Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use jni::errors::StartJvmResult;
use jni::{InitArgsBuilder, JavaVM};
use jni::objects::{JString, JValue};
use crate::charsets::CharsetConverter;
use crate::kotlin::ScopeFunc;
use crate::{Results, workdir};
use crate::logs::LogFile;
use crate::var::*;

pub struct Jvm {
    path: PathBuf,
}

impl Jvm {

    pub fn create() -> Option<Self> {
        let jvm_remember = workdir().join(".jvm");
        let path = File::open(&jvm_remember );
        let mut is_get = path.is_ok();
        if let Ok(mut path) = path {
            is_get = false;
            let mut buf = String::new();
            path.read_to_string(&mut buf).unwrap();
            if test_path_if_is_jvm(Path::new(&buf)) {
                return Some(Self::new(Path::new(&buf).to_path_buf()));
            }
        }
        let jvm = test_all_jvm().ok()?;
        if !is_get {
            File::create(&jvm_remember).unwrap().write_all(jvm.to_string_lossy().as_bytes()).unwrap();
        }
        Some(Self::new(test_all_jvm().ok()?))
    }
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    // java *JRE_OPTIONS -cp *FILES -jar Files[Launcher] -a -c
    fn command_args(&self) -> Vec<String> {
        let mut command_args = Vec::new();
        // push all jre opts
        for opt in JRE_OPTIONS {
            command_args.push(opt.to_string());
        }
        if let Some(charset) = CHARSET_JVM {
            let charset = format!("-Dfile.encoding={}", charset);
            if !command_args.contains(&charset) {
                command_args.push(charset);
            }
        }
        if let Some(img) = SPLASH_SCREEN_IMAGE_PATH {
            command_args.push("-splash:".to_string()+img);
            // command_args.push(img.to_string());
        }
        // get main file and drop it out from FILES
        {
            let mut jars = JAR_FILES.to_vec();
            if jars.is_empty() && JAR_LAUNCHER_MAIN_CLASS.is_none() {
                panic!("No jar files or main class defined");
            }
            let after_commands = if let Some(main_class) = JAR_LAUNCHER_MAIN_CLASS {
                vec![String::from(main_class)]
            } else {
                vec![String::from("-jar"), String::from(jars.remove(JAR_LAUNCHER_FILE))]
            };
            command_args.push("-cp".to_string());
            command_args.push(jars.iter().map(|jar| jar.to_string()).collect::<Vec<String>>().join(";"));
            // push all
            command_args.extend(after_commands);

        }
        // push all args from system and mix up with index
        {
            // get args with index as [(i32, &str)]
            let args = args()
                .skip(1) // skip the first arg and it is the path of the executable
                // fixme console
                .enumerate()
                .map(|(i, arg)| (i as i32, arg))
                .collect::<Vec<(i32, String)>>();
            // join all args
            let mut args = args.iter()
                .map(|(i, arg)| (i, arg.as_str()))
                .chain(JAR_LAUNCHER_ARGS
                    .iter()
                    .map(|(i, arg)|
                        (i, *arg)
                    )
                ).collect::<Vec<(&i32, &str)>>();
            args.sort_by(|a, b| a.0.cmp(b.0).then(Ordering::Less));

            args
                .iter()
                .map(|(_, arg)| arg.to_string())
                .for_each(|arg| command_args.push(arg));
        }


        command_args
    }
    #[inline]
    fn get_file_by_log(data:Option<(Option<&str>,bool)>,log:&str) -> io::Result<Option<File>> {
        if let Some((name_or,overwrite)) = data {
            let s = if let Some(name) = name_or {
                PathBuf::from(name).as_log_file(overwrite)
            } else {
                log.as_log_file(overwrite)
            }.unwrap();
            s.transform(Some).transform(Ok)
        } else {
            Ok(None)
        }
    }

    pub fn invoke(&self) -> Results<()> {
        // command prepare
        let mut command = Command::new(self.path.join("bin").join("java.exe"));
        let child = {
            command.creation_flags(0x08000000);
            command.args(self.command_args());
            // workdir
            command.current_dir(&workdir());
            // set env
            // for (k,v) in JRE_ENVS {
            //     command.env(k,v);
            // }
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
            command.spawn()?

        };
        self._next(child).unwrap();
        // set log
        Ok(())
    }
    fn read_till_line<F>(reader: &mut dyn BufRead, mut callback: F) -> io::Result<()>
        where F: FnMut(&[u8]) -> io::Result<()>
    {
        let mut buf = vec![];
        // let callback = &callback;
        loop {
            if  reader.read_until(b'\n',&mut buf)? < 1 {
                break
            };
            callback(&buf[..])?;
            buf.clear();
        }
        Ok(())
    }
    fn read_std<R:Read+Send>(reader: R, mut log_file:Option<File>, is_err:bool) -> io::Result<String> {
        let mut  reader = BufReader::new(reader);
        let mut r = String::new();
        Jvm::read_till_line(&mut reader, |mut buf| {
            let line = buf.encode_from_std();
            if is_err {
                eprint!("{}", line);
            } else {
                print!("{}", line);
            }
            r+=&line;
            if let Some(log) = log_file.as_mut() {
                log.write_all(line.as_bytes()).unwrap();
            }
            Ok(())
        })?;
        Ok(r)
    }
    fn _next(&self, mut command:Child) -> Results<()> {
        let out = command.stdout.take().map(|stdout| {
            thread::spawn(move || {
                let log = Self::get_file_by_log(LOG_STDOUT_PATH, "log")?;
                Jvm::read_std(stdout, log, false)
            })
        });
        let err = command.stderr.take().map(|stderr| {
            thread::spawn(move || {
                let log = Self::get_file_by_log(LOG_STDERR_PATH, "err")?;
                Jvm::read_std(stderr, log, true)
            })
        });
        let exit_code = command.wait_with_output()?;
        let out =  out.map(|out|
            out.join().unwrap().unwrap()
        );
        let err =  err.map(|err|
            err.join().unwrap().unwrap()
        );
        if !exit_code.status.success() {
            let reason = if let Some(err) = err {
                err + out.unwrap_or_default().as_str()
            } else { String::from("no msg") };
            // let out = String::from_utf8_lossy(&exit_code.stderr);
            // if let Ok(mut f) = "exit".as_append_log_file() {
            //     let _ = f.write_all(out.as_bytes());
            //     let _ = f.write_all("\nerr\n".as_bytes());
            //     let _ = f.write_all(reason.as_bytes());
            // }
            let err = JvmError::exit_code(exit_code.status.code().unwrap_or(-1), reason);
            Err(err.into())
        } else {
            Ok(())
        }
    }
}



#[test]
fn test_java_home() {
    // hook_panic();
    // let path = &PathBuf::from(var_os("JAVA_HOME").unwrap());
    let path = &PathBuf::from("C:\\Heizi\\.jdks\\corretto-1.8.0_372");
    let v = get_version_from_(path).unwrap();
    println!("{}",v
    //     // .join().unwrap()
    );
}

pub fn get_version_from_(path:&PathBuf) -> StartJvmResult<String> {
    let jvm = {
        let i = InitArgsBuilder::default().build().unwrap();
        JavaVM::with_libjvm(i, || {
            Ok(get_dll_if_jvm_in_(path).unwrap())
        })?
    };
    let jvm = &mut jvm.attach_current_thread()?;
    // System.getProperty("java.specification.version")
    let param = {
        let tmp = jvm.find_class("java/lang/System")?;
        let param = jvm.new_string("java.specification.version")?;
        let param = &[JValue::from(&param)];
        let param = jvm.
            call_static_method(tmp, "getProperty", "(Ljava/lang/String;)Ljava/lang/String;", param )?.l()?;
        param // result
    };{ // to string
        let param = JString::from(param);
        let param = jvm.get_string(&param)?;
        param.to_string_lossy().to_string()
    }.transform(Ok)
}


fn get_dll_if_jvm_in_(path:&PathBuf) ->Option<PathBuf> {
    let p = path
        .join("bin").join("server").join("jvm.dll");
    if  p.exists() { return Some(p); }
    let p = path.join("jre")
        .join("bin").join("server").join("jvm.dll");
    if  p.exists() { return Some(p); }
    None
}
fn check_jvm_version(test:&str,range:(u8,u8))-> bool {
    let mut test = test.split('.');
    let version = test.next().unwrap_or("0");
    let mut version = u8::from_str(version).unwrap_or(0);
    if  version<7 {
        let next = test.next().unwrap_or("0");
        version = u8::from_str(next).unwrap_or(0);
    }
    let (start,end) = range;
    version>=start && version<=end
}

fn test_path_if_is_jvm(path:&Path) ->bool {
    path.exists() && path.join("bin").join("java.exe").exists() && path.join("server").join("jvm.dll").exists()
}

fn test_version_in(versions:&[&str], version:&str) ->bool {
    for v in versions {
        if version.starts_with(v) { return true }
    }
    false
}


fn test_jvm_home_version<'a>(path: &'a Path, versions: &[&str]) -> Results<&'a Path> {
    if let Ok(file) = File::open(path.join("release")) {
        let version = BufReader::new(file)
            .lines()
            .flatten()
            .find(|x| x.starts_with("JAVA_VERSION="));
        if let Some(version) = version {
            let version = version.trim_start_matches("JAVA_VERSION=").trim_matches('"');
            if test_version_in(versions, version) {
                return Ok(path)
            }
        }
    }
    let mut output = Command::new(path.join("bin").join("java.exe"))
        .arg("-version")
        .output()?;

    if !output.status.success() {
        let msg = format!(
            "err to execute the java.exe from {:?} . \ncuz:\n{}\n{} ",
            path, output.stdout.encode_from_std(), output.stderr.encode_from_std()
        );
        let msg = JvmError::failed(msg);
        return Err(msg.into());
    }
    let output = output.stdout.encode_from_std();
    let output = output
        .lines()
        .next()
        .and_then(|line| line.split(' ').nth(1));

    if test_version_in(versions, output.expect("not found version")) {
        Ok(path)
    } else {
        let err = JvmError::jvm_not_found(path.to_path_buf());
        Err(err.into())
    }
}







fn test_all_jvm() -> Result<PathBuf, String> {

    let mut errors:Vec<String> = Vec::new();

    let dirs = JRE_SEARCH_ENV
        .iter()
        .filter_map(|&key| {
            var(key)
                .map_err(|e| { errors.push(format!("{}: {}", key, e)); })
                .ok()
        })
        .collect::<Vec<String>>();

    let dirs = dirs
        .iter()
        .map(|s| s.as_str());

    let dirs = JRE_SEARCH_DIR
        .iter().copied()
        .chain(dirs)
        .map(Path::new)
        // .map(|p| {
        //
        //     println!("found {}",p.display());
        //     println!("found {}",p.exists());
        //     p
        // })
        .filter(|path| path.exists())
        .filter(|path| test_path_if_is_jvm(path))
        .filter_map(|path|
            test_jvm_home_version(path, JRE_VERSION)
                .map_err(|e| { errors.push(e.to_string()); })
                .ok()
        ).map(|path| path.to_owned()).collect::<Vec<PathBuf>>();

    dirs
        .first()
        .map(|buf|buf.to_owned())
        .ok_or(format!("we cant find a jvm in the system, errors: {}", errors.join(", ")))
}

// jvm errors
// execute jvm error
enum JvmError {
    JvmNotFound(PathBuf),
    ExecuteFailed(String),
    ExitCode(i32,String)
}
impl JvmError {
    pub fn failed(s: String) -> Self {
        Self::ExecuteFailed(s)
    }
    pub fn jvm_not_found(p: PathBuf) -> Self {
        Self::JvmNotFound(p)
    }
    pub fn exit_code(code: i32, s: String) -> Self {
        Self::ExitCode(code, s)
    }
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JvmError::JvmNotFound(p) => {
                let p = p.to_str().unwrap_or_default();
                write!(f, "Jvm not found in path: {}", p)
            },
            JvmError::ExecuteFailed(s) => write!(f, "{}", s),
            JvmError::ExitCode(code, s) => write!(f, "jvm execute is failed; \nby code:{} reason:\n{}", code, s)
        }
    }
}

impl Debug for JvmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { JvmError::fmt(self, f) }
}
impl Display for JvmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { JvmError::fmt(self, f) }
}
impl std::error::Error for JvmError {}
