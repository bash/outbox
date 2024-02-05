use anyhow::{Context as _, Result};
use std::fs::create_dir_all;
use std::io::{self, stdin};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut temp_file = tempfile::Builder::new()
        .suffix(".eml")
        .tempfile_in(cache_dir()?)?;
    let mut stdin_lock = stdin().lock();
    io::copy(&mut stdin_lock, &mut temp_file)?;
    let (_, path) = temp_file.keep()?;
    opener::open(path)?;
    Ok(())
}

fn cache_dir() -> Result<PathBuf> {
    let mut cache_dir = dirs::cache_dir().context("Unable to detect cache dir")?;
    cache_dir.push("outbox");
    create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}
