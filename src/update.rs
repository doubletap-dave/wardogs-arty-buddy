use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const LATEST: &str =
    "https://api.github.com/repos/doubletap-dave/wardogs-arty-buddy/releases/latest";
const CHECK_AFTER_SECS: f64 = 1.0;
const CHECK_EVERY_SECS: f64 = 15.0 * 60.0;

pub(crate) struct DownloadedUpdate {
    pub version: String,
    pub path: PathBuf,
    pub startup: bool,
}

struct FetchedUpdate {
    version: String,
    path: PathBuf,
}

pub(crate) struct UpdateCheck {
    job: Option<JoinHandle<Option<FetchedUpdate>>>,
    job_is_startup: bool,
    last_check: f64,
    checks_started: u32,
}

impl UpdateCheck {
    pub(crate) fn new() -> Self {
        Self {
            job: None,
            job_is_startup: false,
            last_check: 0.0,
            checks_started: 0,
        }
    }

    /// A finished check that found a newer build. Startup is the first check after launch.
    pub(crate) fn poll(&mut self, now: f64, paused: bool) -> Option<DownloadedUpdate> {
        if cfg!(debug_assertions) {
            return None;
        }
        if self.job.is_none() && !paused {
            let due = if self.checks_started == 0 {
                now >= CHECK_AFTER_SECS
            } else {
                now - self.last_check >= CHECK_EVERY_SECS
            };
            if due {
                self.job_is_startup = self.checks_started == 0;
                self.checks_started += 1;
                self.last_check = now;
                self.job = Some(thread::spawn(fetch_update));
            }
        }
        let finished = self.job.as_ref().is_some_and(|job| job.is_finished());
        if !finished {
            return None;
        }
        let job = self.job.take().expect("finished job");
        let startup = self.job_is_startup;
        match job.join() {
            Ok(Some(update)) => Some(DownloadedUpdate {
                version: update.version,
                path: update.path,
                startup,
            }),
            _ => None,
        }
    }
}

fn asset_name() -> &'static str {
    if cfg!(windows) {
        "WardogsArtyBuddy-Setup.exe"
    } else {
        "wardogs-arty-buddy-linux-x86_64"
    }
}

fn fetch_update() -> Option<FetchedUpdate> {
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
    ensure_maps(&agent, &json);
    let tag = json.get("tag_name")?.as_str()?;
    if !is_newer(tag, env!("CARGO_PKG_VERSION")) {
        return None;
    }
    let version = tag.trim().trim_start_matches('v').to_owned();
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
    let dest = if cfg!(windows) {
        current.with_file_name("WardogsArtyBuddy-Setup.update.exe")
    } else {
        current.with_extension("update")
    };
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
    if len < 200_000 {
        let _ = std::fs::remove_file(&dest);
        return None;
    }
    Some(FetchedUpdate {
        version,
        path: dest,
    })
}

fn ensure_maps(agent: &ureq::Agent, json: &serde_json::Value) {
    let Some(assets) = json.get("assets").and_then(|assets| assets.as_array()) else {
        return;
    };
    let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("lut")))
    else {
        return;
    };
    let _ = std::fs::create_dir_all(&dir);
    for file in ["bakurani.bin", "ozeti.bin", "zestafona.bin"] {
        let dest = dir.join(file);
        if dest.is_file() {
            continue;
        }
        let Some(url) = assets.iter().find_map(|asset| {
            let name = asset.get("name")?.as_str()?;
            if name == file {
                asset.get("browser_download_url")?.as_str()
            } else {
                None
            }
        }) else {
            continue;
        };
        let Ok(response) = agent
            .get(url)
            .set("User-Agent", "wardogs-arty-buddy")
            .call()
        else {
            continue;
        };
        let Ok(mut file) = File::create(&dest) else {
            continue;
        };
        let mut reader = response.into_reader();
        if std::io::copy(&mut reader, &mut file).is_err() {
            let _ = std::fs::remove_file(&dest);
        }
    }
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

pub(crate) fn apply(downloaded: &Path) -> bool {
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

fn spawn_windows_swap(pid: u32, downloaded: &Path, _current: &Path) -> std::io::Result<()> {
    let setup = ps_quote(downloaded);
    let script = format!(
        "$deadline = (Get-Date).AddSeconds(30); \
         while ((Get-Process -Id {pid} -ErrorAction SilentlyContinue) -and ((Get-Date) -lt $deadline)) {{ Start-Sleep -Milliseconds 200 }}; \
         Start-Process -FilePath {setup} -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART' -Wait; \
         $installed = Join-Path $env:LOCALAPPDATA 'Ghostweasel Labs\\Wardogs Arty Buddy\\wardogs-arty-buddy.exe'; \
         if (Test-Path -LiteralPath $installed) {{ Start-Process -FilePath $installed }}"
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
