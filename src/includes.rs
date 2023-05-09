use std::env;
use std::fs::create_dir_all;
use std::io::Error;
use std::path::PathBuf;
use crate::jvm::{ErrorJvmExt, JvmError};
use crate::jvm::JvmError::JvmCacheFailed;
use crate::var::{EXE_PRODUCT_NAME, HASH_OF_INCLUDE_JAR};


// extract jar to AppData/Roaming/jweust/$EXE_PRODUCT_NAME/$hash.jar

const fn get_bytes() -> &'static [u8] {
    return if HASH_OF_INCLUDE_JAR.is_none() {
        &[0u8; 0]
    } else { //jweust-include-jar-start
        panic!("include_bytes!(here...")
    };//jweust-include-jar-end
}

fn get_dir_jar(hash_id:&str) -> (PathBuf,PathBuf) {
    let d = env::var_os("APPDATA")
        .map(PathBuf::from)
        .expect("APPDATA not found")
        .join("jweust")
        .join(EXE_PRODUCT_NAME);
    let j = d
        .join(hash_id)
        .with_extension("jar");
    (d,j)
}
pub fn jar_locate(default:&str) -> Result<String,JvmError> {
    let mut r = String::from(default);
    if let Some(hash) = HASH_OF_INCLUDE_JAR {
        let (d,j) = get_dir_jar(hash);
        let j = &j;
        if d.is_dir() {// delete all jar if it is not the same hash
            let files = std::fs::read_dir(&d)
                .cache_failed(&d, "读取失败")?;
            let j = j.clone();
            for file in files {
                let path = file
                    .cache_failed(&d, "读取失败")?
                    .path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "jar" && path != j {
                            std::fs::remove_file(&path)
                                .cache_failed(path.as_path(), "删除失败")?;
                        }
                    }
                }
            }
        } else {
            create_dir_all(d.clone())
                .cache_failed(&d, "创建失败")?;
        }
        if !j.exists() {
            let b = get_bytes();
            std::fs::write(j, b).unwrap();
        }
        if !j.exists() {
            return Err(JvmCacheFailed(j.clone(), "jar文件不存在".to_string(),Error::last_os_error()));
        }
        r = j.to_str().unwrap().to_string();
    }
    Ok(r)
}
