extern crate winresource;
mod src;

use std::env::set_var;
use std::io;
use std::path::PathBuf;
use embed_manifest::manifest::{ActiveCodePage, ExecutionLevel};
use embed_manifest::manifest::MaxVersionTested::{Windows11Version22H2};
use embed_manifest::manifest::SupportedOS::Windows10;
use embed_manifest::{embed_manifest, Error, new_manifest};
use winresource::{VersionInfo, WindowsResource};
use src::var::*;

fn convert_version(version: &str) -> (u16, u16, u16, u16) {
    let mut version = version.split('.');
    let major = version.next().unwrap_or("0").parse().unwrap();
    let minor = version.next().unwrap_or("0").parse().unwrap();
    let build = version.next().unwrap_or("0").parse().unwrap();
    let revision = version.next().unwrap_or("0").parse().unwrap();
    (major, minor, build, revision)
}
fn covert_version_to_u64(version: &str) -> u64 {
    let (major, minor, build, revision) = convert_version(version);
    (major as u64) << 48 | (minor as u64) << 32 | (build as u64) << 16 | (revision as u64)
}


fn manifest() -> Result<(), Error> {
    let builder = new_manifest("jweust");
    let p = match CHARSET_PAGE_CODE {
        Some(page) if page == "65001" => ActiveCodePage::Utf8,
        // Some(page) => ActiveCodePage::Locale(page.to_string()),
        _ => ActiveCodePage::System
    };
    let pms = match EXE_PERMISSION {
        2=> ExecutionLevel::HighestAvailable,
        1=> ExecutionLevel::RequireAdministrator,
        _ => ExecutionLevel::AsInvoker
    };
    let builder = builder.active_code_page(p)
        .max_version_tested(Windows11Version22H2)
        .requested_execution_level(pms)
        // .ui_access(true)
        .supported_os(Windows10..Windows10);
    embed_manifest(builder)
}
fn res() -> io::Result<WindowsResource> {
    manifest().unwrap();
    let mut res = WindowsResource::new();
    res
        .set_version_info(VersionInfo::PRODUCTVERSION, covert_version_to_u64(EXE_PRODUCT_VERSION))
        .set_version_info(VersionInfo::FILEVERSION, covert_version_to_u64(EXE_FILE_VERSION))
        .set("FileVersion", EXE_FILE_VERSION)
        .set("ProductVersion", EXE_PRODUCT_VERSION)
        .set("ProductName", EXE_PRODUCT_NAME)
        .set("InternalName", EXE_INTERNAL_NAME)
        .set("FileDescription", EXE_FILE_DESCRIPTION)
        .set("LegalCopyright", EXE_LEGAL_COPYRIGHT)
        .set("CompanyName", EXE_COMPANY_NAME);
    if let Some(icon) =  EXE_ICON_PATH  {
        let icon = PathBuf::from(icon).as_path().canonicalize().unwrap();
        let icon = icon.to_str().unwrap();
        res.set_icon(icon);
    }
    Ok(res)
}

fn main() {
    set_var("CARGO_CFG_TARGET_ARCH", EXE_ARCH);
    res().unwrap().compile().unwrap();
    println!("cargo:rerun-if-changed=src/var.rs");
    println!("cargo:rerun-if-changed=build.rs");
}