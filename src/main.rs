mod args;
mod characters;
mod minecraft_text;
mod script;

use crate::args::Args;
use crate::minecraft_text::{MinecraftText, TextColour, ToMcText};
use crate::script::{CharacterJinx, CharacterTeam, Script, ScriptCharacter};
use clap::Parser;
use std::io::Write;
use std::process::exit;

fn main() {
    println!("Note: This project is unaffiliated with The Pandemonium Institute.");
    println!();

    let args = Args::parse();

    let script: Result<Script, _> = serde_json::from_reader(args.input_stream());
    let script = match script {
        Ok(script) => script,
        Err(e) => {
            eprintln!("Failed to parse script.");
            eprintln!("{e}");
            exit(1);
        }
    };

    let Some(script_colour) = TextColour::try_parse(&args.color) else {
        eprintln!("Failed to parse colour from \"{}\".", args.color);
        let all_colour_names = TextColour::ALL_CONSTANTS.map(TextColour::cmd);
        eprintln!(
            "Supported colours are: {}, or hex codes of the form #RRGGBB.",
            all_colour_names.join(", ")
        );
        exit(1);
    };

    if args.version.chars().any(|c| matches!(c, '0'..='9' | '.')) {
        eprintln!("Warning: Version string contains non-digit characters.");
    }

    let script_book = generate_script_book(script_colour, &script, &args.version);

    if let Err(e) = args.output_script_file().write_all(script_book.as_bytes()) {
        eprintln!("Error while writing script command: {e}");
        exit(1);
    }

    let role_papers = generate_role_papers(&script);
    if let Err(e) = args.output_roles_file().write_all(role_papers.as_bytes()) {
        eprintln!("Error while writing role papers command: {e}");
        exit(1);
    }

    println!("Finished successfully!");
}

fn generate_script_book(script_color: TextColour, script: &Script, script_version: &str) -> String {
    let script_name = &script.meta.name;

    let page_count = script.characters.len() + 1 + script.meta.bootlegger.len();
    let mut pages = Vec::with_capacity(page_count);

    let cover_page = generate_cover_page(script_name, script_color, script_version);
    pages.push(cover_page);

    for c in &script.characters {
        if c.team == CharacterTeam::loric && c.name == "Bootlegger" {
            // Do not create a page for the Bootlegger Loric character, the bootlegger modifiers
            // are in the meta of the script
            continue;
        }

        pages.push(generate_character_page(c));
    }
    for (n, bootlegger) in script.meta.bootlegger.iter().enumerate() {
        pages.push(generate_character_page(&ScriptCharacter {
            name: format!("Bootlegger {}", n + 1),
            team: CharacterTeam::loric,
            ability: bootlegger.clone(),
            ..ScriptCharacter::default()
        }));
    }
    for c1 in &script.characters {
        for jinx in &c1.jinxes {
            let Some(c2) = script.characters.iter().find(|c| c.id == jinx.id) else {
                continue;
            };
            pages.push(generate_jinx_page(c1, c2, jinx));
        }
    }
    let pages_str = pages
        .iter()
        .map(MinecraftText::cmd)
        .collect::<Vec<_>>()
        .join(",");

    let book_name = format!("{script_name} Script")
        .mc()
        .color(script_color)
        .underlined()
        .bold()
        .cmd();

    let lore: MinecraftText = vec![
        "Useful role cheat sheet for ".mc().color(TextColour::Gray),
        vec![
            script_name.mc().color(script_color),
            " script.".mc().color(TextColour::Gray),
        ]
        .mc(),
    ]
    .mc();
    let lore = lore.cmd();

    // Ensure text is escaped
    let title = format!("{script_name} Roles").mc().cmd();

    format!(
        "give @s written_book[written_book_content={{pages:[{pages_str}],title:{title},author:\"\"}},custom_name={book_name},lore={lore}]"
    )
}
fn generate_cover_page(
    script_name: &str,
    script_color: TextColour,
    script_version: &str,
) -> MinecraftText {
    vec![
        "".mc(),
        "\n\n   BLOOD ON THE\n    CLOCKTOWER".mc().bold(),
        "\n".mc(),
        format!(r#"  "{script_name}""#)
            .mc()
            .italic()
            .color(script_color),
        "\n".mc(),
        "ROLE CHEAT SHEET".mc().bold(),
        "\n".mc(),
        format!("        v{script_version}")
            .mc()
            .italic()
            .color(TextColour::Gray),
        "\n\n\n\n* next to ability means Not the First Night.".mc(),
    ]
    .mc()
}
fn generate_character_page(c: &ScriptCharacter) -> MinecraftText {
    let color = team_colour(c.team, false);
    vec![
        "".mc(),
        "      ".mc(),
        team_text(c.team).mc().bold().color(color),
        "\n\n".mc(),
        c.name.mc().color(color).underlined(),
        ":\n".mc().color(color),
        c.ability.mc(),
    ]
    .mc()
}
fn generate_jinx_page(
    c1: &ScriptCharacter,
    c2: &ScriptCharacter,
    jinx: &CharacterJinx,
) -> MinecraftText {
    let color1 = team_colour(c1.team, false);
    let color2 = team_colour(c2.team, false);
    vec![
        "".mc(),
        "      ".mc(),
        "JINX".mc().bold().color(TextColour::Gold),
        "\n\n".mc(),
        c1.name.mc().color(color1).underlined(),
        "/ ".mc().color(color1),
        c2.name.mc().color(color2).underlined(),
        ":\n".mc().color(color2),
        jinx.reason.mc(),
    ]
    .mc()
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

fn generate_role_papers(script: &Script) -> String {
    let roles = script
        .characters
        .iter()
        .filter_map(generate_role_paper)
        .collect::<Vec<_>>();
    roles.join("\n")
}

fn generate_role_paper(c: &ScriptCharacter) -> Option<String> {
    use TextColour::{Gray, White};

    let (item_type, components) = item_type(c)?;

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

    let components = components.join(",");
    let command = format!("give @s {item_type}[custom_name={name},lore={lore},{components}]");

    Some(command)
}

fn item_type(c: &ScriptCharacter) -> Option<(String, Vec<String>)> {
    let mut extra = vec![];
    let res = match c.team {
        CharacterTeam::loric if c.id == "godofug" => "copper_helmet",
        CharacterTeam::fabled | CharacterTeam::loric | CharacterTeam::traveller => return None,
        _ => match c.id.as_str() {
            "drunk" | "butler_drunk" | "lunatic" | "marionette" => return None,
            "lilmonsta" => "piglin_head", // TODO: custom player head choice support
            "vizier" => {
                extra.push(
                    "trim={pattern:\"minecraft:wild\",material:\"minecraft:redstone\"}".into(),
                );
                "golden_helmet"
            }
            _ => "paper",
        },
    };

    Some((res.to_string(), extra))
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
