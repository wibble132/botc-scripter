//! Structs for the json format of a botc script. Based on script-schema.json

use crate::characters::lookup_character;
use serde::de::value::MapAccessDeserializer;
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashSet;
use std::fmt::Formatter;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Script {
    pub meta: Meta,
    pub characters: Vec<ScriptCharacter>,
}

impl<'de> Deserialize<'de> for Script {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let items = Vec::<ScriptItem>::deserialize(deserializer)?;

        let mut meta = Option::<Meta>::None;
        let mut characters = Vec::with_capacity(items.len());

        for item in items {
            match item {
                ScriptItem::Meta(m) => {
                    if meta.replace(m).is_some() {
                        return Err(D::Error::custom("script contains multiple _meta tags"));
                    }
                }
                ScriptItem::ScriptCharacter(c) => characters.push(*c),
                ScriptItem::OfficialCharacterId(id)
                | ScriptItem::OfficialCharacter(OfficialCharacter { id }) => {
                    match lookup_character(&id) {
                        Some(c) => characters.push(c.clone()),
                        None => {
                            return Err(D::Error::custom(format!(
                                "Unknown official character id {id}"
                            )));
                        }
                    }
                }
            }
        }

        characters.sort_by_key(|c| c.team);

        // Filter the jinxes to only contain ones with both characters in play
        let character_ids = characters
            .iter()
            .map(|c| c.id.clone())
            .collect::<HashSet<_>>();
        for c in &mut characters {
            c.jinxes.retain(|jinx| character_ids.contains(&jinx.id));
        }

        Ok(Script {
            meta: meta.unwrap_or_default(),
            characters,
        })
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ScriptItem {
    /// Script Metadata
    Meta(Meta),
    /// Script Character
    ScriptCharacter(Box<ScriptCharacter>),
    /// Official character ID
    OfficialCharacterId(String),
    /// Official Character (deprecated)
    OfficialCharacter(OfficialCharacter),
}

impl<'de> Deserialize<'de> for ScriptItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ScriptItemVisitor;
        impl<'de> Visitor<'de> for ScriptItemVisitor {
            type Value = ScriptItem;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "a character or script metadata.")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ScriptItem::OfficialCharacterId(v.to_string()))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let json_map =
                    serde_json::value::Map::deserialize(MapAccessDeserializer::new(map))?;

                let Some(Value::String(id)) = json_map.get("id") else {
                    return Err(A::Error::missing_field("Expected string field \"id\""));
                };

                if id == "_meta" {
                    let meta = match Meta::deserialize(json_map) {
                        Ok(m) => m,
                        Err(e) => {
                            return Err(A::Error::custom(format!("Could not parse _meta: {e}")));
                        }
                    };
                    Ok(ScriptItem::Meta(meta))
                } else if json_map.len() == 1 {
                    Ok(ScriptItem::OfficialCharacter(OfficialCharacter {
                        id: id.clone(),
                    }))
                } else {
                    let char = match ScriptCharacter::deserialize(json_map) {
                        Ok(c) => c,
                        Err(e) => {
                            return Err(A::Error::custom(format!(
                                "Could not parse character: {e}"
                            )));
                        }
                    };
                    Ok(ScriptItem::ScriptCharacter(Box::new(char)))
                }
            }
        }

        deserializer.deserialize_any(ScriptItemVisitor)
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Clone, Eq, PartialEq, Debug, Default)]
pub struct ScriptCharacter {
    /// Character ID, alphanumeric lowercase without spaces
    pub id: String,
    /// Character name
    pub name: String,
    #[serde(skip, default)]
    pub image: (),
    /// Character team
    pub team: CharacterTeam,
    /// The edition ID
    #[serde(default)]
    pub edition: String,
    /// Character Ability
    pub ability: String,
    /// Character flavor text
    #[serde(default)]
    pub flavor: String,
    /// First night wake-up priority. 0 means this character doesn't wake up during the first night.
    #[serde(default)]
    pub firstNight: u32,
    /// First night Storyteller reminder
    #[serde(default)]
    pub firstNightReminder: Option<String>,
    /// Other nights wake-up priority. 0 means this character doesn't wake up during other nights.
    #[serde(default)]
    pub otherNight: u32,
    /// Other night Storyteller reminder
    #[serde(default)]
    pub otherNightReminder: Option<String>,
    /// Character reminder tokens
    #[serde(default)]
    pub reminders: Vec<String>,
    /// Global character reminder tokens, which will be selectable even when the character is not in play.
    #[serde(default)]
    pub remindersGlobal: Vec<String>,
    /// Whether this character has a ability that is relevant for the game setup.
    #[serde(default)]
    pub setup: bool,
    /// Jinxes with other characters on this script
    #[serde(default)]
    pub jinxes: Vec<CharacterJinx>,
    #[serde(skip, default)]
    pub special: (),
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd, Default)]
pub enum CharacterTeam {
    #[default]
    townsfolk,
    outsider,
    minion,
    demon,
    traveller,
    fabled,
    loric,
}

#[derive(Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct CharacterJinx {
    /// The ID of the other character this one is jinxed with
    pub id: String,
    /// The Jinx explanation
    pub reason: String,
}

#[derive(Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct OfficialCharacter {
    /// Official character ID
    pub id: String,
}

#[derive(Deserialize, Default, Clone, Eq, PartialEq, Debug)]
pub struct Meta {
    /// The script name
    pub name: String,
    /// The script author
    #[serde(default)]
    pub author: String,
    /// The script logo image
    #[serde(default)]
    pub logo: String,
    /// Whether the script title / author should be hidden on the app
    #[serde(default, rename = "hide_title")]
    pub hide_title: bool,
    /// The script background image
    #[serde(default)]
    pub background: String,
    /// A link to the script's almanac page
    #[serde(default)]
    pub almanac: String,
    /// A list of homebrew rules that are to be used for this script.
    #[serde(default)]
    pub bootlegger: Vec<String>,
    /// The relative First Night order of characters on this script.
    #[serde(default, rename = "firstNight")]
    pub first_night: Vec<String>,
    /// The relative Other Nights order of characters on this script.
    #[serde(default, rename = "otherNight")]
    pub other_night: Vec<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn official_characters() {
        let input = r#"["sailor", "balloonist", "soldier", "drunk", "spy", "imp"]"#;
        serde_json::from_str::<Script>(input).unwrap();
    }
    #[test]
    fn official_characters_old() {
        let input = r#"
        [
          { "id": "sailor" },
          { "id": "balloonist" },
          { "id": "soldier" },
          { "id": "drunk" },
          { "id": "spy" },
          { "id": "imp" }
        ]
        "#;

        let data = serde_json::from_str::<Script>(input).unwrap();

        let characters = ["sailor", "balloonist", "soldier", "drunk", "spy", "imp"]
            .map(lookup_character)
            .map(Option::unwrap)
            .map(Clone::clone)
            .to_vec();

        let expected = Script {
            meta: Meta::default(),
            characters,
        };

        assert_eq!(data, expected);
    }
    #[test]
    fn meta() {
        let input = r#"
        [
          { "id": "_meta", "name": "cool_script" }
        ]
        "#;

        let data = serde_json::from_str::<Script>(input).unwrap();

        let expected = Script {
            meta: Meta {
                name: "cool_script".to_string(),
                ..Meta::default()
            },
            characters: vec![],
        };

        assert_eq!(data, expected);
    }

    #[test]
    fn script_character() {
        let input = r#"
        [
          {
            "id": "washerwoman",
            "name": "Washerwoman",
            "team": "townsfolk",
            "edition": "tb",
            "ability": "You start knowing that 1 of 2 players is a particular Townsfolk.",
            "setup": false,
            "reminders": [
              "Townsfolk",
              "Wrong"
            ],
            "remindersGlobal": [],
            "firstNightReminder": "Show the Townsfolk character token. Point to both the *TOWNSFOLK* and *WRONG* players.",
            "otherNightReminder": null,
            "special": [],
            "jinxes": [],
            "flavor": "Bloodstains on a dinner jacket? No, this is cooking sherry. How careless.",
            "image": "https://cdn.botc.games/images/resources/characters/tb/washerwoman_g.webp",
            "firstNight": 14,
            "otherNight": 0
          }
        ]
        "#;

        let data = serde_json::from_str::<Script>(input).unwrap();

        let expected = Script {
            characters: vec![ScriptCharacter {
                id: "washerwoman".to_string(),
                name: "Washerwoman".to_string(),
                image: (),
                team: CharacterTeam::townsfolk,
                edition: "tb".to_string(),
                ability: "You start knowing that 1 of 2 players is a particular Townsfolk.".to_string(),
                flavor: "Bloodstains on a dinner jacket? No, this is cooking sherry. How careless.".to_string(),
                firstNight: 14,
                firstNightReminder: Some("Show the Townsfolk character token. Point to both the *TOWNSFOLK* and *WRONG* players.".to_string()),
                otherNight: 0,
                otherNightReminder: None,
                reminders: vec!["Townsfolk".to_string(), "Wrong".to_string()],
                remindersGlobal: vec![],
                setup: false,
                jinxes: vec![],
                special: (),
            }],
            meta: Meta::default(),
        };

        assert_eq!(data, expected);
    }
}
