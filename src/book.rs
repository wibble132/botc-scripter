use crate::minecraft_text::JoinMcText;
use crate::minecraft_text::{MinecraftText, TextColour, ToMcText};
use crate::script::{CharacterJinx, CharacterTeam, Script, ScriptCharacter};
use crate::{team_colour, team_text, Version};
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum PageType {
    Character(CharacterTeam),
    Jinx,
}
impl PageType {
    #[allow(clippy::enum_glob_use)]
    const ALL_TYPES: [PageType; 8] = {
        use CharacterTeam::*;
        use PageType::*;
        [
            Character(townsfolk),
            Character(outsider),
            Character(minion),
            Character(demon),
            Character(traveller),
            Character(fabled),
            Character(loric),
            Jinx,
        ]
    };
}

#[derive(Clone)]
struct ContentPage {
    ty: PageType,
    title: MinecraftText,
    content: MinecraftText,
}

pub fn generate_script_book(
    script_color: TextColour,
    script: &Script,
    script_version: &Version,
) -> String {
    let script_name = &script.meta.name;

    let cover = generate_cover_page(script_name, script_color, script_version);

    let character_pages = script
        .characters
        .iter()
        // Do not create a page for the Bootlegger Loric character, the bootlegger modifiers
        // are in the meta of the script
        .filter(|c| !(c.team == CharacterTeam::loric && c.name == "Bootlegger"))
        .map(generate_character_page);

    let bootlegger_pages = script
        .meta
        .bootlegger
        .iter()
        .enumerate()
        .map(|(n, bootlegger)| {
            generate_character_page(&ScriptCharacter {
                name: format!("Bootlegger {}", n + 1),
                team: CharacterTeam::loric,
                ability: bootlegger.clone(),
                ..ScriptCharacter::default()
            })
        });

    let jinx_pages = script.characters.iter().flat_map(|c1| {
        c1.jinxes.iter().filter_map(|jinx| {
            let c2 = script.characters.iter().find(|c| c.id == jinx.id)?;
            Some(generate_jinx_page(c1, c2, jinx))
        })
    });

    let mut content_pages = vec![];
    content_pages.extend(character_pages);
    content_pages.extend(bootlegger_pages);
    content_pages.extend(jinx_pages);

    // Pages = Cover + Index + Content
    // How do I build Index?
    let index_page_count = generate_index_pages(&content_pages, 0).len();
    let index_page_count = u32::try_from(index_page_count).expect("More than a million pages??!");
    let index = generate_index_pages(&content_pages, 2 + index_page_count);

    let mut pages = vec![cover];
    pages.extend(index);
    pages.extend(content_pages.into_iter().map(|c| c.content));

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
    script_version: &Version,
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
        format!("        v{}", script_version.0)
            .mc()
            .italic()
            .color(TextColour::Gray),
        "\n\n\n\n* next to ability means Not the First Night.".mc(),
    ]
    .mc()
}
fn generate_character_page(c: &ScriptCharacter) -> ContentPage {
    let color = team_colour(c.team, false);
    let title = c.name.mc().color(color).underlined();
    let content = vec![
        "".mc(),
        "      ".mc(),
        team_title(c.team).page_link(2),
        "\n\n".mc(),
        title.clone(),
        ":\n".mc().color(color),
        c.ability.mc(),
    ]
    .mc();

    ContentPage {
        ty: PageType::Character(c.team),
        title,
        content,
    }
}
fn generate_jinx_page(
    c1: &ScriptCharacter,
    c2: &ScriptCharacter,
    jinx: &CharacterJinx,
) -> ContentPage {
    let color1 = team_colour(c1.team, false);
    let color2 = team_colour(c2.team, false);

    let title = vec![
        "".mc(),
        c1.name.mc().color(color1).underlined(),
        "/\n   ".mc().color(color1),
        c2.name.mc().color(color2).underlined(),
    ]
    .mc();

    let content = vec![
        "".mc(),
        "      ".mc(),
        jinx_title().page_link(2),
        "\n\n".mc(),
        title.clone(),
        ":\n".mc().color(color2),
        jinx.reason.mc(),
    ]
    .mc();

    ContentPage {
        ty: PageType::Jinx,
        title,
        content,
    }
}
fn team_title(team: CharacterTeam) -> MinecraftText {
    team_text(team).mc().bold().color(team_colour(team, false))
}
fn jinx_title() -> MinecraftText {
    "JINX".mc().bold().color(TextColour::Gold)
}
fn generate_index_pages(content: &[ContentPage], link_offset: u32) -> Vec<MinecraftText> {
    const LINES_PER_PAGE: usize = 12; // Not including header lines
    let header = vec!["".mc(), "      Contents\n".mc().bold()].mc();

    // Where the next item should link to
    let mut link_target = link_offset;

    // Group the pages by character type
    let mut page_types = HashMap::new();
    for c in content {
        page_types.entry(c.ty).or_insert(vec![]).push(c);
    }

    let mut book = Book::new(header.clone(), LINES_PER_PAGE);

    for ty in PageType::ALL_TYPES {
        let Some(ty_pages) = page_types.get(&ty) else {
            continue;
        };

        let entry_size = match ty {
            PageType::Character(_) => 1,
            PageType::Jinx => 2,
        };

        let ty_title = match ty {
            PageType::Character(team) => team_title(team),
            PageType::Jinx => jinx_title(),
        };

        // Don't squeeze into the bottom of a page
        // the 2 here: 1 for blank space, 1 for title
        // and also ensure space for at least one entry
        if book.current_page_space() < 2 + entry_size {
            book.start_new_page();
        }

        if book.current_page_lines > 0 {
            // If we are in the middle of a page, put an extra space between the last type's roles
            //  and this new type's
            book.blank_line();
        }

        book.add_text(ty_title, 1);

        for c in ty_pages {
            let title = c.title.clone().unbold().page_link(link_target);

            link_target += 1;

            book.add_text(title.clone(), entry_size);
        }
    }

    book.finish()
}

struct Book {
    pages: Vec<MinecraftText>,
    page_heading: MinecraftText,
    line_space: usize,

    current_page: Vec<MinecraftText>,
    current_page_lines: usize,
}
impl Book {
    pub fn new(page_heading: MinecraftText, line_space: usize) -> Self {
        let first_page = vec![page_heading.clone()];

        Self {
            pages: vec![],
            page_heading,
            line_space,

            current_page: first_page,
            current_page_lines: 0,
        }
    }
    pub fn current_page_space(&self) -> usize {
        self.line_space - self.current_page_lines
    }
    pub fn start_new_page(&mut self) {
        let page = std::mem::take(&mut self.current_page);

        self.pages.push(Vec::<MinecraftText>::join(page, "\n".mc()));

        self.current_page.push(self.page_heading.clone());
        self.current_page_lines = 0;
    }
    pub fn add_text(&mut self, line: MinecraftText, lines: usize) {
        // Can we fit?
        if lines > self.current_page_space() {
            self.start_new_page();
        }

        // Note: If `lines` is more than `self.line_space` this will never fit,
        // so we just do our best
        self.current_page.push(line);
        self.current_page_lines += lines;
    }
    pub fn blank_line(&mut self) {
        self.add_text("".mc(), 1);
    }
    pub fn finish(mut self) -> Vec<MinecraftText> {
        if self.current_page_lines > 0 {
            self.start_new_page();
        }

        self.pages
    }
}
