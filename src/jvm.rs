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

    pub fn create() -> Result<Self,JvmError> {
        let jvm_remember = workdir().join(".jvm");
        if !jvm_remember.exists() {
            let (min_j,max_j) = JRE_VERSION;
            let jvm = jvm_searches();
            let jvm = jvm_version_parsing(jvm);

            let jvm = jvm.collect::<Vec<(PathBuf,String)>>();
            
            if let Some((jvm,_)) = jvm.iter()
                .filter(|(_,v)|
                    check_jvm_version(v,(&min_j,&max_j))
                ).next() {
                let mut f = File::create(&jvm_remember).expect(format!("创建失败.jvm {jvm_remember:?}").as_str());
                f.write_all(jvm.to_string_lossy().as_bytes()).expect(format!("写入.jvm失败 {jvm_remember:?}").as_str());
            } else {
                let mut buf = String::new();
                jvm.iter()
                    .map(|(p,v)|
                        format!("{p:?} found versions {v} 's jvm\n"))
                    .collect_into(&mut buf);

                let min_j = if min_j<6 {format!("{min_j}")} else { String::from("undefined")};
                let max_j = if max_j>40 {format!("{max_j}")} else { String::from("unlimited")};

                buf.push_str(&*format!("but not in version supported jvm 's version : {}..{}",min_j,max_j));
                return Err(JvmError::JvmNotFound(buf))

            };

        }
        let mut jvm_remember = &File::open(&jvm_remember).expect(format!("打开失败 {jvm_remember:?}").as_str());
        let mut buf = String::new();

        jvm_remember.read_to_string(&mut buf).expect(format!("读取失败 {jvm_remember:?}").as_str());

        let buf = PathBuf::from(buf);

        if get_dll_if_jvm_in_(&buf).is_some() {
            return Ok(Self::new(Path::new(&buf).to_path_buf()));
        }
        panic!("end of creating jvm")
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
    let path = &PathBuf::from(var("JAVA_HOME").unwrap());
    let v = get_version_from_(path).unwrap();
    println!("{}",v);
}

pub fn get_version_from_(path: &PathBuf) -> StartJvmResult<String> {
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
fn check_jvm_version(test:&str,range:(&u8,&u8))-> bool {
    let mut test = test.split('.');
    let version = test.next().unwrap_or("0");
    let mut version = u8::from_str(version).unwrap_or(0);
    if  version<7 {
        let next = test.next().unwrap_or("0");
        version = u8::from_str(next).unwrap_or(0);
    }
    let (start,end) = range;
    version>= *start && version<= *end
}

// static ble
fn jvm_searches() -> impl Iterator<Item = PathBuf> {
    JRE_SEARCH_DIR.iter().map(|s|String::from(*s)).chain(
    JRE_SEARCH_ENV.iter().filter_map(|&key| var(key).ok())
    ).map(PathBuf::from)
}
fn jvm_version_parsing(dirs:impl Iterator<Item = PathBuf>) -> impl Iterator<Item = (PathBuf,String)> {
    dirs.filter_map(|path|
        get_version_from_(&path).map(|version|(path, version))  .ok()
    )
}

// jvm errors
// execute jvm error
pub enum JvmError {
    JvmNotFound(String),
    // ExecuteFailed(String),
    ExitCode(i32,String)
}
impl JvmError {
    // pub fn failed(s: String) -> Self {
    //     Self::ExecuteFailed(s)
    // }
    // pub fn jvm_not_found(errors: String) -> Self {
    //     Self::JvmNotFound(errors)
    // }
    pub fn exit_code(code: i32, s: String) -> Self {
        Self::ExitCode(code, s)
    }
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JvmError::JvmNotFound(str) => {
                write!(f, "{str}")
            }
            // JvmError::ExecuteFailed(s) => write!(f, "{}", s),
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
