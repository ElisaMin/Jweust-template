# Jweust !!! but template.
JavaWindowsExecutableRust is a Rust-based executable launcher for Java.

# Getting Started
To use this project, you need to follow these steps:
 - Clone the repository to your local machine.  
 - Modify the var.rs file with the necessary configuration options for your project.  
 - Use ` cargo build  --release ` to compile the Rust code into an executable.    
 
to config var.rs
```rust
#![allow(dead_code)]

// pub const INCLUDE_JAR:bool = false; // not support
pub const APPLICATION_WITH_OUT_CLI:Option<Option<&'static str>> = Some(Some("-DConsolog=true"));
pub const WORKDIR:Option<(&'static str,bool)> = None;
// pub const WORKDIR_IS_VARIABLE:bool = false;

pub const LOG_STDERR_PATH:Option<(Option<&'static str>,bool)> = None;
pub const LOG_STDOUT_PATH:Option<(Option<&'static str>,bool)> = None;

pub const CHARSET_STDOUT:Option<&'static str> = Some("GBK");
pub const CHARSET_JVM:Option<&'static str> = None;
pub const CHARSET_PAGE_CODE:Option<&'static str> = None;

pub const JAR_FILES:&[&str] = &["H:\\gits\\Heizi-Flashing-Tools\\tools\\sideload-install-wizard\\build\\libs\\sideload-install-wizard-0.0.9-all.jar"];
pub const JAR_LAUNCHER_FILE:usize = 0;
pub const JAR_LAUNCHER_MAIN_CLASS:Option<&str> = None;
pub const JAR_LAUNCHER_ARGS:&[(i32,&str)] = &[];

pub const EXE_IS_INSTANCE:bool = false;
pub const EXE_PERMISSION:i8 = -1;
pub const EXE_ICON_PATH:Option<&str> = Some("D:\\Downloads\\ic_ast_ugly.ico");
pub const EXE_FILE_VERSION:&str = "0.0.0.9";
pub const EXE_PRODUCT_VERSION:&str = "0.0.9";
pub const EXE_INTERNAL_NAME:&str = "Android apk Sideload Tool From Heizi Flash Tools";
pub const EXE_FILE_DESCRIPTION:&str = "线刷APK安装工具";
pub const EXE_LEGAL_COPYRIGHT:&str = "Github/ElisaMin";
pub const EXE_COMPANY_NAME:& str = "Heizi";

// new
pub const EXE_ARCH:&str = "x86_64";
pub const EXE_PRODUCT_NAME:&str = "Android Package Sideload Tool";
pub const RUST_PROJECT_NAME:&str = "sideload-install-wizard";

pub const JRE_SEARCH_DIR:&[&str] = &["./lib/runtime"];
pub const JRE_SEARCH_ENV:&[&str] = &["JAVA_HOME"];
pub const JRE_OPTIONS:&[&str] = &[];
pub const JRE_NATIVE_LIBS:&[&str] = &[];
pub const JRE_VERSION:&[&str] = &["19.0"];
pub const JRE_PREFERRED:& str = "DefaultVM";
pub const SPLASH_SCREEN_IMAGE_PATH:Option<&'static str> = Some("H:\\tupm\\ic_ast_ugly.png");

```
