use std::env;
use std::fs::create_dir_all;
use std::path::PathBuf;
use crate::var::{EXE_PRODUCT_NAME, HASH_OF_INCLUDE_JAR};


// extract jar to AppData/Roaming/jweust/$EXE_PRODUCT_NAME/$hash.jar

const fn get_bytes() -> &'static [u8] {
    include_bytes!("H:\\gits\\Heizi-Flashing-Tools\\tools\\sideload-install-wizard\\build\\libs\\sideload-install-wizard-0.0.9-all.jar")
    // &[0u8; 0]
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
pub fn jar_locate(default:&str) -> String {
    let mut r = String::from(default);
    if let Some(hash) = HASH_OF_INCLUDE_JAR {
        let (d,j) = get_dir_jar(hash);
        let j = &j;
        if d.is_dir() {// delete all jar if it is not the same hash
            let files = std::fs::read_dir(&d).unwrap();
            let j = j.clone();
            for file in files {
                let path = file.unwrap().path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "jar" && path != j {
                            std::fs::remove_file(path).unwrap();
                        }
                    }
                }
            }
        } else {
            create_dir_all(d).unwrap();
        }
        if !j.exists() {
            let b = get_bytes();
            std::fs::write(&j, b).unwrap();
        }
        if !j.exists() {
            panic!("{} not found", j.display());
        }
        r = j.to_str().unwrap().to_string();
    }
    r
}
