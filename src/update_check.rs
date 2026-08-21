use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::task::JoinHandle;

const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const FETCH_TIMEOUT: Duration = Duration::from_secs(2);
const WAIT_LIMIT: Duration = Duration::from_millis(500);
const DEFAULT_URL: &str = "https://api.github.com/repos/coval-ai/cli/releases/latest";

pub struct UpdateCheck {
    handle: JoinHandle<Option<String>>,
}

pub fn start() -> Option<UpdateCheck> {
    if std::env::var_os("COVAL_NO_UPDATE_CHECK").is_some() {
        return None;
    }
    if !is_due() {
        return None;
    }
    let url = std::env::var("COVAL_UPDATE_CHECK_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
    let handle = tokio::spawn(fetch_latest(url));
    Some(UpdateCheck { handle })
}

pub async fn finish(check: Option<UpdateCheck>) {
    let Some(check) = check else {
        return;
    };
    let Ok(result) = tokio::time::timeout(WAIT_LIMIT, check.handle).await else {
        return;
    };
    let Ok(latest) = result else {
        return;
    };
    touch_stamp();
    let Some(latest) = latest else {
        return;
    };
    let current = env!("CARGO_PKG_VERSION");
    if is_newer(&latest, current) {
        eprintln!(
            "A newer version of coval is available: v{latest} (installed: v{current}). {}",
            upgrade_instructions()
        );
    }
}

fn state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("coval")
        .join("update-check")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn is_due() -> bool {
    let Ok(content) = std::fs::read_to_string(state_path()) else {
        return true;
    };
    let Ok(last_check) = content.trim().parse::<u64>() else {
        return true;
    };
    now_secs().saturating_sub(last_check) >= CHECK_INTERVAL.as_secs()
}

fn touch_stamp() {
    let path = state_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, now_secs().to_string());
}

async fn fetch_latest(url: String) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .user_agent(format!("coval-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;
    let response = client.get(&url).send().await.ok()?;
    let value: serde_json::Value = response.json().await.ok()?;
    value
        .get("tag_name")
        .and_then(|tag| tag.as_str())
        .map(|tag| tag.trim_start_matches('v').to_string())
}

fn version_parts(version: &str) -> Vec<u64> {
    version
        .split('.')
        .filter_map(|part| part.parse().ok())
        .collect()
}

fn is_newer(latest: &str, current: &str) -> bool {
    version_parts(latest) > version_parts(current)
}

fn upgrade_instructions() -> String {
    let installed_via_brew = std::env::current_exe()
        .map(|exe| {
            let path = exe.to_string_lossy();
            path.contains("Cellar") || path.contains("homebrew")
        })
        .unwrap_or(false);
    if installed_via_brew {
        "Update with: brew upgrade coval-ai/tap/coval".to_string()
    } else {
        "Download: https://github.com/coval-ai/cli/releases".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn newer_patch_version() {
        assert!(is_newer("0.7.1", "0.7.0"));
    }

    #[test]
    fn newer_minor_version_with_double_digits() {
        assert!(is_newer("0.10.0", "0.9.9"));
    }

    #[test]
    fn same_version_is_not_newer() {
        assert!(!is_newer("0.7.1", "0.7.1"));
    }

    #[test]
    fn older_version_is_not_newer() {
        assert!(!is_newer("0.5.0", "0.7.1"));
    }

    #[test]
    fn shorter_version_loses_to_longer_prefix() {
        assert!(is_newer("0.7.1", "0.7"));
        assert!(!is_newer("0.7", "0.7.1"));
    }

    #[test]
    fn non_numeric_suffix_is_ignored() {
        assert!(is_newer("1.0.0-beta.1", "0.9.0"));
    }
}
