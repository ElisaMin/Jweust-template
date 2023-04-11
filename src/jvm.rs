use std::cmp::Ordering;
use std::env::{args, var};
use std::fmt::{Debug, Display, Formatter};
use std::fs::{File};
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Write};
use std::{io, panic, thread};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use crate::logs::{LogFile};
use crate::{logs, Results};
use crate::kotlin::ScopeFunc;
use crate::var::*;


/**
var.rs
pub const INCLUDE_JAR:bool = false;
pub const APPLICATION_TYPE_IS_NO_CMD:bool = true;
pub const WORKDIR:&str = ".";
pub const WORKDIR_IS_VARIABLE:bool = false;

pub const LOG_STDERR_PATH:Option<(Option<&'static str>,bool)> = Some((None,false));
pub const LOG_STDOUT_PATH:Option<(Option<&'static str>,bool)> = None;

pub const JAR_FILES:&[&str] = &["path/to/jar"];
pub const JAR_LAUNCHER_FILE:usize = 0;
pub const JAR_LAUNCHER_MAIN_CLASS:Option<&str> = Some("tools.heizi.ast.Main");
pub const JAR_LAUNCHER_ARGS:&[(i32,&str)] = &[(0,"-a"),(i32::MAX,"-c")];

pub const EXE_IS_INSTANCE:bool = true;
pub const EXE_IS_X86:bool = false;
pub const EXE_PERMISSION:i8 = -1;
pub const EXE_ICON_PATH:Option<&str> = Some("icon.ico");
pub const EXE_FILE_VERSION:&str = "0.0.1";
pub const EXE_PRODUCT_VERSION:&str = "0.0.0";
pub const EXE_INTERNAL_NAME:&str = "Android apk Sideload Tool From Heizi Flash Tools";
pub const EXE_FILE_DESCRIPTION:&str = "线刷APK安装工具";
pub const EXE_LEGAL_COPYRIGHT:&str = "Github/ElisaMin";
pub const EXE_COMPANY_NAME:& str = "Heizi";

pub const JRE_SEARCH_DIR:&[&str] = &["./lib/runtime"];
pub const JRE_SEARCH_ENV:&[&str] = &["JAVA_HOME"];
pub const JRE_OPTIONS:&[&str] = &[];
pub const JRE_NATIVE_LIBS:&[&str] = &[];
pub const JRE_VERSION:&[&str] = &["19.0"];
pub const JRE_PREFERRED:& str = "DefaultVM";
pub const SPLASH_SCREEN_IMAGE_PATH:Option<&'static str> = Some("path/toImage");

 */


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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JvmError::JvmNotFound(p) => {
                let p = p.to_str().unwrap_or_default();
                write!(f, "Jvm not found in path: {}", p)
            },
            JvmError::ExecuteFailed(s) => write!(f, "{}", s),
            JvmError::ExitCode(code, s) => write!(f, "jvm execute is failed; \nby code:{} reason:\n{}\n", code, s)

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


pub struct Jvm {
    path: PathBuf,
}

impl Jvm {

    pub fn create() -> Option<Self> {
        let path = File::open(Path::new(WORKDIR).join(".jvm"));
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
            File::create(Path::new(WORKDIR).join(".jvm")).unwrap().write_all(jvm.to_string_lossy().as_bytes()).unwrap();
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
        command.args(self.command_args());
        // workdir
        command.current_dir(WORKDIR);
        // set env
        // for (k,v) in JRE_ENVS {
        //     command.env(k,v);
        // }
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let command = command.spawn()?;
        self._next(command).unwrap();
        // set log
        Ok(())
    }
    fn read_till_line<F>(reader: &mut dyn BufRead, mut callback: F) -> io::Result<()>
        where F: FnMut(&[u8]) -> io::Result<()>
    {
        let mut buf = vec![];
        // let callback = &callback;
        loop {
            let size = reader.read_until(b'\n',&mut buf)?;
            if size == 0 {
                break;
            }
            callback(&buf[..size])?;
            buf.clear();
        }
        Ok(())
    }
    fn read_std<R:Read+Send>(reader: R, mut log_file:Option<File>, is_err:bool) -> io::Result<()> {
        let mut  reader = BufReader::new(reader);
        Jvm::read_till_line(&mut reader, |buf| {
            let line = String::from_utf8_lossy(buf);
            if is_err {
                eprint!("{}", line);
            } else {
                print!("{}", line);
            }
            if let Some(log) = log_file.as_mut() {
                log.write_all(line.as_bytes()).unwrap();
            }
            Ok(())
        })
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
        if let Some(out) = out {
            out.join()
                .map_err(|e| e.downcast::<Error>().unwrap())??;
        }
        if let Some(err) = err {
            err.join()
                .map_err(|e| e.downcast::<Error>().unwrap())??;
        }
        if !exit_code.status.success() {
            let reason = String::from_utf8_lossy(&exit_code.stderr);
            // let out = String::from_utf8_lossy(&exit_code.stderr);
            // if let Ok(mut f) = "exit".as_append_log_file() {
            //     let _ = f.write_all(out.as_bytes());
            //     let _ = f.write_all("\nerr\n".as_bytes());
            //     let _ = f.write_all(reason.as_bytes());
            // }
            let err = JvmError::ExitCode(exit_code.status.code().unwrap_or(-1), reason.to_string());
            Err(Box::try_from(err).unwrap())
        } else {
            Ok(())
        }
    }
}



fn test_path_if_is_jvm(path:&Path) ->bool {
    path.exists() && path.join("bin").join("java.exe").exists()
}
fn test_version_in(versions:&[&str], version:&str) ->bool {
    for v in versions {
        if version.starts_with(v) { return true }
    }
    false
}


fn test_jvm_home_version<'a>(path: &'a Path, versions: &[&str]) -> Result<&'a Path, String> {
    if let Ok(mut file) = File::open(path.join("release")).map_err(|e| format!("Failed to open release file: {}", e)) {
        let mut buf = String::new();
        file.read_to_string(&mut buf).map_err(|e| format!("Failed to read release file: {}", e))?;
        let version = buf
            .lines()
            .find_map(|line| line.strip_prefix("JAVA_VERSION=").map(|s| s.trim_matches('"')))
            .ok_or_else(|| format!("Java version not found in release file: {:?}", path.join("release")))?;
        if test_version_in(versions, version) {
            return Ok(path);
        }
    }

    let output = Command::new(path.join("bin").join("java.exe"))
        .arg("-version")
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "java -version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let output = String::from_utf8(output.stdout).map_err(|e| {
        format!(
            "Failed to parse command output as UTF-8: {}",
            String::from_utf8_lossy(&e.as_bytes())
        )
    })?;
    let output = output
        .lines()
        .next()
        .and_then(|line| line.split(' ').nth(1))
        .ok_or_else(|| format!("Failed to parse command output: {}", output))?;
    if test_version_in(versions, output) {
        Ok(path)
    } else {
        Err(format!("{} is not a JVM path", path.display()))
    }
}





fn test_all_jvm() -> Result<PathBuf, String> {

    let mut errors:Vec<String> = Vec::new();

    let dirs = JRE_SEARCH_ENV
        .iter()
        .map(|&key| {
            var(key)
                .map_err(|e| { errors.push(format!("{}: {}", key, e)); })
                .ok()
        })
        .filter_map(|s| s)
        .collect::<Vec<String>>();

    let dirs = dirs
        .iter()
        .map(|s| s.as_str());

    let dirs = JRE_SEARCH_DIR
        .iter()
        .map(|&s| s)
        .chain(dirs)
        .map(|path| Path::new(path))
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
                .map_err(|e| { errors.push(e); })
                .ok()
        ).map(|path| path.to_owned().into()).collect::<Vec<PathBuf>>();

    dirs
        .first()
        .map(|buf|buf.to_owned())
        .ok_or(format!("we cant find a jvm in the system, errors: {}", errors.join(", ")))
}