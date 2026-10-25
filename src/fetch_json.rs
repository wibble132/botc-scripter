use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

const SCRIPT_LIST_URL: &str = "https://scripts.botc.ink/api/script_ids/";

#[derive(Debug)]
pub enum FetchError {
    Http(ureq::Error),
    Json(serde_json::Error),
    ScriptNotFound(String),
    VersionNotFound { script_id: String, version: String },
}

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Http(e) => write!(f, "HTTP request failed: {e}"),
            FetchError::Json(e) => write!(f, "Failed to parse JSON response: {e}"),
            FetchError::ScriptNotFound(id) => write!(f, "No online script with id \"{id}\""),
            FetchError::VersionNotFound { script_id, version } => {
                write!(f, "Script \"{script_id}\" has no version \"{version}\"")
            }
        }
    }
}
impl std::error::Error for FetchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FetchError::Http(e) => Some(e),
            FetchError::Json(e) => Some(e),
            _ => None,
        }
    }
}
impl From<ureq::Error> for FetchError {
    fn from(e: ureq::Error) -> Self {
        FetchError::Http(e)
    }
}
impl From<serde_json::Error> for FetchError {
    fn from(e: serde_json::Error) -> Self {
        FetchError::Json(e)
    }
}

pub struct OnlineScriptData {
    scripts: Vec<OnlineScript>,
}

#[derive(Serialize, Deserialize)]
struct OnlineScriptPage {
    count: u64,
    next: Option<String>,
    previous: Option<String>,
    #[serde(alias = "results")]
    scripts: Vec<OnlineScript>,
}

#[derive(Serialize, Deserialize)]
struct OnlineScript {
    pk: u64,
    name: String,
    slug: Option<String>,
    versions: HashMap<String, String>,
    latest_version: String,
}

fn get_text_from_url(url: &str) -> Result<String, FetchError> {
    Ok(ureq::get(url).call()?.body_mut().read_to_string()?)
}

impl OnlineScriptData {
    pub fn load_script_list() -> Result<Self, FetchError> {
        let mut scripts = vec![];
        let mut next_url = Some(SCRIPT_LIST_URL.to_string());
        while let Some(url) = next_url {
            let page: OnlineScriptPage = serde_json::from_str(&get_text_from_url(&url)?)?;
            scripts.extend(page.scripts);
            next_url = page.next;
        }

        Ok(Self { scripts })
    }

    fn find_script(&self, script_id: &str) -> Result<&OnlineScript, FetchError> {
        self.scripts
            .iter()
            .find(|s| s.slug.as_deref() == Some(script_id))
            .ok_or_else(|| FetchError::ScriptNotFound(script_id.to_string()))
    }

    #[expect(dead_code)] // Not used currently, but seems useful
    pub fn get_script_versions(&self, script_id: &str) -> Result<Vec<String>, FetchError> {
        let script = self.find_script(script_id)?;
        Ok(script.versions.keys().cloned().collect())
    }

    pub fn script_json(&self, script_id: &str, version: &str) -> Result<String, FetchError> {
        let script = self.find_script(script_id)?;
        let url = script
            .versions
            .get(version)
            .ok_or_else(|| FetchError::VersionNotFound {
                script_id: script_id.to_string(),
                version: version.to_string(),
            })?;

        get_text_from_url(&format!("{url}json/"))
    }

    pub fn latest_script_json(&self, script_id: &str) -> Result<(String, String), FetchError> {
        let script = self.find_script(script_id)?;
        let url = &script.latest_version;
        let Some(version) = script
            .versions
            .iter()
            .find(|(_, u)| u == &url)
            .map(|(v, _)| v)
        else {
            return Err(FetchError::VersionNotFound {
                script_id: script_id.to_string(),
                version: "latest".to_string(),
            });
        };
        get_text_from_url(&format!("{url}json/")).map(|text| (version.clone(), text))
    }
}
