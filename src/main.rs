mod args;
mod book;
mod characters;
mod fetch_json;
mod minecraft_text;
mod script;
mod script_data;

use crate::args::Args;
use crate::book::generate_script_book;
use crate::minecraft_text::{TextColour, ToMcText};
use crate::script::{CharacterTeam, Script, ScriptCharacter};
use crate::script_data::{CharacterItem, ScriptData};
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, stdin};
use std::path::Path;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use crate::fetch_json::get_text_from_url;

fn main() {
    println!("Note: This project is unaffiliated with The Pandemonium Institute.");
    println!();

    let args = Args::parse();

    let script_id = &args.name;

    if script_id == "all" {
        println!("RUNNING FOR ALL {} SCRIPTS", script_data::script_count());
        sleep(Duration::from_secs(1));

        // Run all online scripts
        for script_name in script_data::all_script_keys() {
            let script_args = Args {
                name: script_name.to_string(),
                ..args.clone()
            };

            let mut script_dir = script_args.base_dir();
            script_dir.push("script");
            script_dir.push(format!("{}.mcfunction", script_args.name));
            if script_dir.exists() {
                println!("Skipping script {script_name}");
                continue;
            }

            println!("Running script {script_name}");
            create_script_functions(script_name, &script_args);
            println!();
        }
    } else {
        create_script_functions(script_id, &args);
    }
}

fn create_script_functions(script_id: &str, args: &Args) {
    let Some(data) = script_data::lookup_data(script_id) else {
        eprintln!(
            "Could not find data for script {script_id}. Add this script to script-data.json."
        );
        exit(1);
    };

    let script_colour = data.colour;
    let item_overrides = &data.items;

    let (version, mut script) = inputs(script_id, args, data);

    if script.meta.name.is_empty() {
        println!("Warning: No script name set for script with id {script_id}");
        script.meta.name = "Unknown".to_string();
    }

    if version.0.chars().any(|c| !matches!(c, '0'..='9' | '.')) {
        eprintln!(
            "Warning: Version string \"{}\" contains non-digit characters.",
            version.0
        );
    }

    let script_book = generate_script_book(script_colour, &script, &version);

    if let Err(e) = args.output_script_file().write_all(script_book.as_bytes()) {
        eprintln!("Error while writing script command: {e}");
        exit(1);
    }

    let role_papers = generate_role_papers(&script, item_overrides);
    if let Err(e) = args.output_roles_file().write_all(role_papers.as_bytes()) {
        eprintln!("Error while writing role papers command: {e}");
        exit(1);
    }

    println!("Written functions for script {script_id} v{}", version.0);
}

struct Version(String);

fn inputs(script_id: &str, args: &Args, data: &ScriptData) -> (Version, Script) {
    // Options for the input are:
    // 1. Input file set in command line
    // 2. Script json directly in ScriptData
    // 3. Script filepath set in ScriptData
    // 4. Fetch from URL set in ScriptData
    // 5. Fetch from URL using the script name

    // args.version MUST be set unless we are fetching from URL using script name
    let mut version = args.version.clone().or_else(|| data.version.clone());
    let script: Script;

    if let Some(file_path) = &args.input_file {
        let json = read_from_file(file_path);
        script = script_from_json(&json);
    } else if let Some(data_script) = &data.script {
        script = data_script.clone();
    } else if let Some(file_path) = &data.script_path {
        let json = read_from_file(file_path);
        script = script_from_json(&json);
    } else if let Some(data_script_url) = &data.script_url {
        let Ok(json) = get_text_from_url(data_script_url) else {
            eprintln!("Failed to fetch script from URL: {data_script_url}");
            exit(1);
        };

        script = script_from_json(&json);
    } else {
        let (web_version, web_script) = read_from_web(
            script_id,
            args.version.as_deref(),
        );
        script = web_script;
        version = Some(web_version);
    }

    let Some(version) = version else {
        eprintln!("No version set.\n --version needs to be set for offline scripts");
        exit(1);
    };

    (Version(version), script)
}

fn read_from_file(file_path: &Path) -> String {
    if file_path.as_os_str() == "-" {
        // Read from stdin
        let mut json = String::new();
        return match stdin().lock().read_to_string(&mut json) {
            Ok(_) => json,
            Err(e) => {
                eprintln!("Failed to read script from stdin: {e}");
                exit(1);
            }
        };
    }

    let mut file = match File::open(file_path) {
        Err(e) => {
            eprintln!("Failed to open file {}.", file_path.display());
            eprintln!("{e}");
            exit(1);
        }
        Ok(f) => f,
    };

    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => json,
        Err(e) => {
            eprintln!("Failed to read script from file {}", file_path.display());
            eprintln!("{e}");
            exit(1);
        }
    }
}
fn read_from_web(script_name: &str, version_override: Option<&str>) -> (String, Script) {
    println!("Loading script from online");
    let script_list = match fetch_json::OnlineScriptData::load_script_list() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Failed to get script list: {e}");
            exit(1);
        }
    };

    let json_result = match version_override {
        Some(v) => script_list
            .script_json(script_name, v)
            .map(|x| (v.to_owned(), x)),
        None => script_list.latest_script_json(script_name),
    };

    let (v, json) = match json_result {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Failed to download json for script: {e}");
            exit(1);
        }
    };

    let mut script = script_from_json(&json);
    if script.meta.name.is_empty()
        && let Ok(full_script_name) = script_list.script_name(script_name)
    {
        full_script_name.clone_into(&mut script.meta.name);
    }

    (v, script)
}
fn script_from_json(script_json: &str) -> Script {
    let script: Result<Script, _> = serde_json::from_str(script_json);
    match script {
        Ok(script) => script,
        Err(e) => {
            eprintln!("Failed to parse script.");
            eprintln!("{e}");
            exit(1);
        }
    }
}

fn team_colour(team: CharacterTeam, light: bool) -> TextColour {
    use TextColour::{Aqua, DarkBlue, DarkGreen, DarkRed, Gold, Gray};

    match team {
        CharacterTeam::townsfolk | CharacterTeam::outsider if light => Aqua,
        CharacterTeam::townsfolk | CharacterTeam::outsider => DarkBlue,
        CharacterTeam::minion | CharacterTeam::demon => DarkRed,
        CharacterTeam::traveller => Gray,
        CharacterTeam::fabled => Gold,
        CharacterTeam::loric => DarkGreen,
    }
}
fn team_text(team: CharacterTeam) -> &'static str {
    match team {
        CharacterTeam::townsfolk => "TOWNSFOLK",
        CharacterTeam::outsider => "OUTSIDER",
        CharacterTeam::minion => "MINION",
        CharacterTeam::demon => "DEMON",
        CharacterTeam::traveller => "TRAVELLER",
        CharacterTeam::fabled => "FABLED",
        CharacterTeam::loric => "LORIC",
    }
}

fn generate_role_papers(
    script: &Script,
    item_overrides: &HashMap<String, CharacterItem>,
) -> String {
    let roles = script
        .characters
        .iter()
        .filter_map(|c| generate_role_paper(c, item_overrides.get(&c.id)))
        .collect::<Vec<_>>();
    roles.join("\n")
}

fn generate_role_paper(
    c: &ScriptCharacter,
    item_override: Option<&CharacterItem>,
) -> Option<String> {
    use TextColour::{Gray, White};

    let (item_type, components) = match (c, item_override) {
        // An item override is set
        (_, Some(item)) => {
            let item_id = item.item_id.as_ref()?;
            (item_id.as_str(), item.components.as_str())
        }
        (c, None) if !should_skip_item(c) => ("paper", ""),
        _ => return None,
    };

    let colour = team_colour(c.team, true);
    let team_caps = team_text(c.team).to_ascii_uppercase();

    // Wrap text in ability to 36 chars
    let ability_lines = wrap_text(&c.ability, 36);

    let name = c.name.mc().bold().underlined().color(colour).cmd();
    let mut lore = team_caps.mc().color(White).underlined();
    lore.push("".mc());
    for l in ability_lines {
        lore.push(l.mc().color(Gray));
    }
    let lore = lore.cmd();

    let command = format!("give @s {item_type}[custom_name={name},lore={lore},{components}]");

    Some(command)
}

fn should_skip_item(c: &ScriptCharacter) -> bool {
    matches!(
        c.team,
        CharacterTeam::fabled | CharacterTeam::loric | CharacterTeam::traveller
    ) || matches!(c.id.as_str(), "drunk" | "lunatic" | "marionette")
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::with_capacity(text.len() / width + 1);
    let mut current_line = String::with_capacity(width);

    for word in text.split_ascii_whitespace() {
        // Always add the next word to an empty line
        if current_line.is_empty() {
            current_line = word.to_string();
            continue;
        }

        // Will adding the word make the line too long?
        if current_line.len() + 1 + word.len() > width {
            // Create new line
            lines.push(current_line);
            current_line = word.to_string();
            continue;
        }

        // Have space, add the word to the current line (with a space)
        current_line += " ";
        current_line += word;
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
