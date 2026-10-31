use crate::minecraft_text::TextColour;
use crate::script::Script;
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::LazyLock;

#[derive(Clone)]
pub struct ScriptData {
    pub colour: TextColour,
    pub items: HashMap<String, CharacterItem>,
    pub script: Option<Script>,
    pub script_path: Option<PathBuf>,
    pub version: Option<String>,
}
#[derive(Clone)]
pub struct CharacterItem {
    pub item_id: Option<String>,
    pub components: String,
}

pub fn lookup_data(script_id: &str) -> Option<&ScriptData> {
    SCRIPT_DATA.get(script_id)
}
pub fn all_script_keys() -> impl Iterator<Item = &'static str> {
    SCRIPT_DATA.keys().map(String::as_str)
}

static SCRIPT_DATA: LazyLock<HashMap<String, ScriptData>> = LazyLock::new(|| {
    fn items_to_map<T, K, V, F>(is: Vec<T>, f: F) -> HashMap<K, V>
    where
        K: Eq + Hash,
        F: Fn(T) -> (K, V),
    {
        is.into_iter().map(f).collect()
    }

    let data: Vec<ScriptDataJson> = serde_json::from_str(include_str!("../data/script-data.json"))
        .expect("Failed to parse script data");

    items_to_map(data, |d: ScriptDataJson| {
        (
            d.script_id,
            ScriptData {
                colour: d.colour,
                items: items_to_map(d.items, |d: ScriptItemJson| {
                    (
                        d.character_id,
                        CharacterItem {
                            item_id: d.item_id,
                            components: d.components,
                        },
                    )
                }),
                script: d.script,
                script_path: d.script_path,
                version: d.version,
            },
        )
    })
});

#[derive(Deserialize)]
pub struct ScriptDataJson {
    #[serde(alias = "id")]
    script_id: String,
    colour: TextColour,
    #[serde(default)]
    items: Vec<ScriptItemJson>,
    #[serde(default)]
    script: Option<Script>,
    #[serde(default)]
    script_path: Option<PathBuf>,
    #[serde(default)]
    version: Option<String>,
}

impl<'de> Deserialize<'de> for TextColour {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TextColourVisitor;
        impl Visitor<'_> for TextColourVisitor {
            type Value = TextColour;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "one of ")?;
                for col in TextColour::ALL_CONSTANTS {
                    write!(formatter, "{}, ", col.cmd())?;
                }
                write!(formatter, " or hex code of the form #RRGGBB")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                TextColour::try_parse(v).ok_or_else(|| E::invalid_value(Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_string(TextColourVisitor)
    }
}

#[derive(Deserialize)]
pub struct ScriptItemJson {
    #[serde(alias = "id")]
    character_id: String,
    #[serde(alias = "item")]
    item_id: Option<String>,
    #[serde(default)]
    components: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn script_data_loads() {
        let _ = &*SCRIPT_DATA;
    }
}
