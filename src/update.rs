use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const LATEST: &str =
    "https://api.github.com/repos/doubletap-dave/wardogs-arty-buddy/releases/latest";
const CHECK_AFTER_SECS: f64 = 8.0;
const CHECK_EVERY_SECS: f64 = 6.0 * 60.0 * 60.0;

pub(crate) struct UpdateCheck {
    job: Option<JoinHandle<Option<PathBuf>>>,
    last_check: f64,
    started_once: bool,
}

impl UpdateCheck {
    pub(crate) fn new() -> Self {
        Self {
            job: None,
            last_check: 0.0,
            started_once: false,
        }
    }

    /// Returns true when a newer build has been downloaded and a restart is underway.
    pub(crate) fn poll(&mut self, now: f64) -> bool {
        if cfg!(debug_assertions) {
            return false;
        }
        if self.job.is_none() {
            let due = if self.started_once {
                now - self.last_check >= CHECK_EVERY_SECS
            } else {
                now >= CHECK_AFTER_SECS
            };
            if due {
                self.started_once = true;
                self.last_check = now;
                self.job = Some(thread::spawn(fetch_update));
            }
        }
        let finished = self.job.as_ref().is_some_and(|job| job.is_finished());
        if !finished {
            return false;
        }
        let job = self.job.take().expect("finished job");
        match job.join() {
            Ok(Some(path)) => relaunch(&path),
            _ => false,
        }
    }
}

fn asset_name() -> &'static str {
    if cfg!(windows) {
        "wardogs-arty-buddy-windows-x86_64.exe"
    } else {
        "wardogs-arty-buddy-linux-x86_64"
    }
}

fn fetch_update() -> Option<PathBuf> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(120))
        .build();
    let body = agent
        .get(LATEST)
        .set("User-Agent", "wardogs-arty-buddy")
        .set("Accept", "application/vnd.github+json")
        .call()
        .ok()?
        .into_string()
        .ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    let tag = json.get("tag_name")?.as_str()?;
    if !is_newer(tag, env!("CARGO_PKG_VERSION")) {
        return None;
    }
    let assets = json.get("assets")?.as_array()?;
    let url = assets.iter().find_map(|asset| {
        let name = asset.get("name")?.as_str()?;
        if name == asset_name() {
            asset
                .get("browser_download_url")?
                .as_str()
                .map(str::to_owned)
        } else {
            None
        }
    })?;
    let current = std::env::current_exe().ok()?;
    let dest = current.with_extension("update");
    let mut reader = agent
        .get(&url)
        .set("User-Agent", "wardogs-arty-buddy")
        .call()
        .ok()?
        .into_reader();
    let mut file = File::create(&dest).ok()?;
    std::io::copy(&mut reader, &mut file).ok()?;
    file.flush().ok()?;
    drop(file);
    let len = std::fs::metadata(&dest).ok()?.len();
    if len < 1_000_000 {
        let _ = std::fs::remove_file(&dest);
        return None;
    }
    Some(dest)
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(latest), Some(current)) => latest > current,
        _ => false,
    }
}

fn parse_version(text: &str) -> Option<[u32; 3]> {
    let text = text.trim().trim_start_matches('v');
    let mut parts = text.split('.');
    Some([
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ])
}

fn relaunch(downloaded: &Path) -> bool {
    let Ok(current) = std::env::current_exe() else {
        return false;
    };
    let pid = std::process::id();
    let spawned = if cfg!(windows) {
        spawn_windows_swap(pid, downloaded, &current)
    } else {
        spawn_unix_swap(pid, downloaded, &current)
    };
    spawned.is_ok()
}

fn spawn_windows_swap(pid: u32, downloaded: &Path, current: &Path) -> std::io::Result<()> {
    let new_path = ps_quote(downloaded);
    let exe_path = ps_quote(current);
    let script = format!(
        "$deadline = (Get-Date).AddSeconds(30); \
         while ((Get-Process -Id {pid} -ErrorAction SilentlyContinue) -and ((Get-Date) -lt $deadline)) {{ Start-Sleep -Milliseconds 200 }}; \
         Move-Item -LiteralPath {new_path} -Destination {exe_path} -Force; \
         Start-Process -FilePath {exe_path}"
    );
    Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .spawn()?;
    Ok(())
}

fn spawn_unix_swap(pid: u32, downloaded: &Path, current: &Path) -> std::io::Result<()> {
    let script = format!(
        "i=0; while kill -0 {pid} 2>/dev/null && [ \"$i\" -lt 150 ]; do sleep 0.2; i=$((i+1)); done; \
         mv -f '{}' '{}'; chmod +x '{}'; nohup '{}' >/dev/null 2>&1 &",
        downloaded.display(),
        current.display(),
        current.display(),
        current.display()
    );
    Command::new("sh").args(["-c", &script]).spawn()?;
    Ok(())
}

fn ps_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_release_compares_by_numbers() {
        assert!(is_newer("v0.7.0", "0.6.0"));
        assert!(is_newer("0.6.1", "0.6.0"));
        assert!(!is_newer("v0.6.0", "0.6.0"));
        assert!(!is_newer("v0.5.9", "0.6.0"));
        assert!(!is_newer("nope", "0.6.0"));
    }
}
