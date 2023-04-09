use std::cmp::Ordering;
use std::env::{args, var};
use std::fmt::{Debug, Display, Formatter, write};
use std::fs::{File};
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Write};
use std::panic;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, exit, Stdio};
use windows::core::{PCWSTR};
use crate::show_and_exit_with;
use crate::var::*;


/**
var.rs
pub static INCLUDE_JAR:bool = false;
pub static APPLICATION_TYPE_IS_NO_CMD:bool = true;
pub static WORKDIR:&'static str = ".";
pub static WORKDIR_IS_VARIABLE:bool = false;

pub static LOG_ERROR_PATH:Option<&'static str> = Some("error.log");
pub static LOG_ERROR_IS_OVERWRITE:bool = false;
pub static LOG_STDOUT_PATH:Option<&'static str> = None;
pub static LOG_STDOUT_IS_OVERWRITE:bool = false;

pub static JAR_FILES:&[&'static str] = &["path/to/jar"];
pub static JAR_LAUNCHER_FILE:usize = 0;
pub static JAR_LAUNCHER_MAIN_CLASS:Option<&'static str> = Some("tools.heizi.ast.Main");
pub static JAR_LAUNCHER_ARGS:&[(i32,&'static str)] = &[(0,"-a"),(i32::MAX,"-c")];
pub static EXE_IS_INSTANCE:bool = true;
pub static EXE_IS_X86:bool = false;
pub static EXE_PERMISSION:i8 = -1;
pub static EXE_ICON_PATH:Option<&'static str> = Some("icon.ico");
pub static EXE_FILE_VERSION:&'static str = "0.0.1";
pub static EXE_PRODUCT_VERSION:&'static str = "0.0.0";
pub static EXE_INTERNAL_NAME:&'static str = "Android apk Sideload Tool From Heizi Flash Tools";
pub static EXE_FILE_DESCRIPTION:&'static str = "线刷APK安装工具";
pub static EXE_LEGAL_COPYRIGHT:&'static str = "Github/ElisaMin";
pub static EXE_COMPANY_NAME:&'static str = "Heizi";

pub static JRE_SEARCH_DIR:&[&'static str] = &["./lib/runtime"];
pub static JRE_SEARCH_ENV:&[&'static str] = &["JAVA_HOME"];
pub static JRE_OPTIONS:&[&'static str] = &[];
pub static JRE_NATIVE_LIBS:&[&'static str] = &[];
pub static JRE_VERSION:&[&'static str] = &["19.0"];
pub static JRE_PREFERRED:&'static str = "DefaultVM";
pub static SPLASH_SCREEN_IMAGE_PATH:Option<&'static str> = Some("path/toImage");

 */


// jvm errors
// execute jvm error
enum JvmError {
    JvmNotFound(PathBuf),
    ExecuteFailed(String)
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
            JvmError::ExecuteFailed(s) => write!(f, "{}", s)
        }
    }
}


impl Debug for JvmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for JvmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for JvmError {}




struct Jvm {
    path: PathBuf,
}

impl Jvm {

    pub fn create() -> Option<Self> {
        let path = File::open(Path::new(WORKDIR).join(".jvm"));
        let mut is_get = path.is_ok();
        if path.is_ok() {
            is_get = false;
            let mut path = path.unwrap();
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
            let mut jars = JAR_FILES.iter().map(|&s|s).collect::<Vec<&str>>();
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

    // log
    fn log(& self,mut process: Child) {
        let process_err = panic::catch_unwind(||{
            LOG_STDOUT_PATH.map(|path| {
                let file = File::create(path).unwrap();
            });
            let file = file!("log.txt");
            ;
            process.try_wait().unwrap().unwrap()
        });

        if let Ok(code) = process_err {
            if !code.success() {
                exit()
                
            }
        }

        }
        if process_err.is_err() {
            println!("Error: {}", process_err.err().unwrap());
        }
    }

    pub fn invoke(&self) {

        // command prepare
        let mut command = std::process::Command::new(self.path.join("bin").join("java.exe"));
        command.current_dir(WORKDIR);
        command.args(JAR_FILES.iter().map(|&path| "-jar".to_string()).chain(
            JAR_FILES.iter().map(|&path| path.to_string())
        ));
    }
}


fn test_path_if_is_jvm(path:&Path) ->bool {
    return path.exists() && path.join("bin").join("java.exe").exists();
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

    let output = std::process::Command::new(path.join("bin").join("java.exe"))
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