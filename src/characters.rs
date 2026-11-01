use crate::script::{CharacterJinx, ScriptCharacter};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Deserialize)]
struct NightSheet {
    #[serde(rename = "firstNight")]
    first_night: Vec<String>,
    #[serde(rename = "otherNight")]
    other_night: Vec<String>,
}

#[derive(Deserialize)]
struct CharacterJinxes {
    id: String,
    jinx: Vec<CharacterJinx>,
}

static ROLES: LazyLock<HashMap<String, ScriptCharacter>> = LazyLock::new(|| {
    let roles: Vec<ScriptCharacter> =
        serde_json::from_str(include_str!("../data/roles.json")).unwrap();
    let night_sheet: NightSheet =
        serde_json::from_str(include_str!("../data/nightsheet.json")).unwrap();
    let jinxes: Vec<CharacterJinxes> =
        serde_json::from_str(include_str!("../data/jinxes.json")).unwrap();

    let mut roles_map = HashMap::new();
    for mut role in roles {
        let first_night = night_sheet
            .first_night
            .iter()
            .position(|i| i == &role.id)
            .unwrap_or_default();

        let other_nights = night_sheet
            .other_night
            .iter()
            .position(|i| i == &role.id)
            .unwrap_or_default();

        let jinxes = jinxes
            .iter()
            .find(|j| j.id == role.id)
            .map(|js| js.jinx.clone())
            .unwrap_or_default();

        role.firstNight = first_night as u64 as f64;
        role.otherNight = other_nights as u64 as f64;
        role.jinxes = jinxes;

        roles_map.insert(role.id.clone(), role);
    }

    roles_map
});

pub fn lookup_character(id: &str) -> Option<&ScriptCharacter> {
    ROLES.get(id)
}
