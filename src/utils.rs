// Copyright © 2016 Felix Obenhuber
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use config::{Config, File};
use failure::Error;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{convert::Into, env, path::PathBuf, sync::RwLock};
use which::which_in;

lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new({
        Config::builder()
            .add_source(File::from(config_dir().join("config.toml")).required(false))
            .build()
            .unwrap_or_default()
    });
}

/// Find adb binary
pub fn adb() -> Result<PathBuf, Error> {
    which_in("adb", env::var_os("PATH"), env::current_dir()?).map_err(Into::into)
}

pub fn terminal_width() -> Option<usize> {
    match term_size::dimensions() {
        Some((width, _)) => Some(width),
        None => env::var("COLUMNS")
            .ok()
            .and_then(|e| e.parse::<usize>().ok()),
    }
}

/// Detect configuration directory
pub fn config_dir() -> PathBuf {
    directories::BaseDirs::new()
        .unwrap()
        .config_dir()
        .join("rogcat")
}

/// Read a value from the configuration file
/// `config_dir/config.toml`
pub fn config_get<'a, T: Deserialize<'a>>(key: &'a str) -> Option<T> {
    CONFIG.read().ok().and_then(|c| c.get::<T>(key).ok())
}

pub fn config_init() {
    drop(CONFIG.read().expect("Failed to get config lock"));
}

pub fn get_pids(packages: &[String]) -> Result<std::collections::HashSet<u32>, Error> {
    if packages.is_empty() {
        return Ok(std::collections::HashSet::new());
    }

    let mut command = std::process::Command::new(adb()?);
    command.arg("shell").arg("pidof");
    for pkg in packages {
        command.arg(pkg);
    }

    let output = command.stdout(std::process::Stdio::piped()).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut pids = std::collections::HashSet::new();

    for word in stdout.split_whitespace() {
        if let Ok(pid) = word.parse::<u32>() {
            pids.insert(pid);
        }
    }

    Ok(pids)
}

pub struct ProcessFilter {
    packages: Vec<String>,
    valid_pids: std::collections::HashSet<u32>,
    last_update: std::time::Instant,
}

impl ProcessFilter {
    const TTL: std::time::Duration = std::time::Duration::from_secs(2);

    pub fn new(packages: Vec<String>) -> Self {
        // Initialize with a past timestamp so it updates immediately on first use
        let last_update = std::time::Instant::now()
            .checked_sub(Self::TTL + std::time::Duration::from_secs(1))
            .unwrap_or_else(std::time::Instant::now);

        Self {
            packages,
            valid_pids: std::collections::HashSet::new(),
            last_update,
        }
    }

    pub fn should_skip_process(&mut self, pid_str: &str) -> bool {
        if self.packages.is_empty() || pid_str.is_empty() {
            return false;
        }

        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => return false,
        };

        let now = std::time::Instant::now();
        if now.duration_since(self.last_update) > Self::TTL {
            if let Ok(pids) = get_pids(&self.packages) {
                self.valid_pids = pids;
            }
            self.last_update = now;
        }

        !self.valid_pids.contains(&pid)
    }
}
