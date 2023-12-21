use anyhow::{Context as _, Result};
use std::ffi::{CString, OsStr};
use std::fs::{create_dir_all, File};
use std::io::{self, stdin};
use std::os::fd::FromRawFd as _;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let mut stdin_lock = stdin().lock();
    let mut path_template: PathBuf = dirs::cache_dir().context("Unable to detect cache dir")?;
    path_template.push("outbox");
    create_dir_all(&path_template)?;
    path_template.push("XXXXXX.eml");
    let (path, mut file) = mkstemps(path_template, 4)?;
    io::copy(&mut stdin_lock, &mut file)?;
    opener::open(path)?;
    Ok(())
}

fn mkstemps(template: impl AsRef<Path>, suffixlen: i32) -> io::Result<(PathBuf, File)> {
    let template = CString::new(template.as_ref().as_os_str().as_bytes())?.into_raw();
    let fd = unsafe { libc::mkstemps(template, suffixlen) };
    let path = PathBuf::from(OsStr::from_bytes(
        unsafe { CString::from_raw(template) }.as_bytes(),
    ));

    if fd >= 0 {
        let file = unsafe { File::from_raw_fd(fd) };
        Ok((path.to_owned(), file))
    } else {
        Err(io::Error::last_os_error())
    }
}
