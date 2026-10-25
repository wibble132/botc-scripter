//! Structures for building Minecraft Text Components. Only supports the
//!
//! See <https://minecraft.wiki/w/Text_component_format#Java_Edition> for more information

use std::fmt::Write;
use std::mem;

#[derive(Clone, Debug)]
pub enum MinecraftText {
    // Written as quoted text, "A" is equivalent to "{text: A}"
    String(String),
    // Written as an array. [A, B, C] is equivalent to {..A, extra: A.extra + [B,C]}
    Array(Vec<MinecraftText>),
    Rich(RichText),
}

impl MinecraftText {
    pub fn cmd(&self) -> String {
        match self {
            MinecraftText::String(s) => Self::escape_string(s),
            MinecraftText::Array(vec) => {
                let inner: Vec<_> = vec.iter().map(MinecraftText::cmd).collect();
                let inner = inner.join(",");
                format!("[{inner}]")
            }
            MinecraftText::Rich(r) => {
                let mut out = String::new();
                out += "{";
                _ = write!(out, "text:{}", Self::escape_string(&r.text));

                if r.bold {
                    out += ",bold:true";
                }
                _ = write!(out, ",italic:{}", r.italic);
                if r.underlined {
                    out += ",underlined:true";
                }

                if let Some(c) = &r.color {
                    _ = write!(out, ",color:{}", c.cmd());
                }

                if let Some(click_event) = &r.click_event {
                    match click_event {
                        ClickEvent::ChangePage { page } => {
                            _ = write!(out, ",click_event:{{action:change_page,page:{page}}}");
                        }
                    }
                }

                out += "}";
                out
            }
        }
    }
    fn escape_string(s: &str) -> String {
        serde_json::to_string(s).unwrap()
    }
    pub fn push(&mut self, text: MinecraftText) {
        // If we are already array, add to it
        if let MinecraftText::Array(vec) = self {
            vec.push(text);
            return;
        }

        // Replace self with an array, first with old self then the new item
        let array = MinecraftText::Array(vec![]);
        let old_self = mem::replace(self, array);
        let MinecraftText::Array(vec) = self else {
            unreachable!("Self was just set to Array");
        };
        vec.push(old_self);
        vec.push(text);
    }
    pub fn bold(self) -> Self {
        self.richen(|r| r.bold = true)
    }
    pub fn unbold(self) -> Self {
        self.richen(|r| r.bold = false)
    }
    pub fn italic(self) -> Self {
        self.richen(|r| r.italic = true)
    }
    pub fn underlined(self) -> Self {
        self.richen(|r| r.underlined = true)
    }
    pub fn color(self, color: TextColour) -> Self {
        self.richen(|r| r.color = Some(color))
    }
    pub fn page_link(self, page: u32) -> Self {
        self.richen(|r| r.click_event = Some(ClickEvent::ChangePage { page }))
    }
    fn richen<F>(self, f: F) -> Self
    where
        F: Fn(&mut RichText) + Copy,
    {
        match self {
            MinecraftText::String(text) => {
                let mut rich = RichText {
                    text,
                    ..RichText::default()
                };
                f(&mut rich);
                MinecraftText::Rich(rich)
            }
            MinecraftText::Array(mut v) => {
                for t in &mut v {
                    let tt = mem::replace(t, MinecraftText::String(String::new()));
                    *t = tt.richen(f);
                }

                MinecraftText::Array(v)
            }
            MinecraftText::Rich(mut r) => {
                f(&mut r);
                MinecraftText::Rich(r)
            }
        }
    }
}

impl<'a> From<&'a str> for MinecraftText {
    fn from(value: &'a str) -> Self {
        MinecraftText::String(value.to_string())
    }
}
impl From<Vec<MinecraftText>> for MinecraftText {
    fn from(value: Vec<MinecraftText>) -> Self {
        MinecraftText::Array(value)
    }
}
impl From<RichText> for MinecraftText {
    fn from(value: RichText) -> Self {
        MinecraftText::Rich(value)
    }
}

pub trait ToMcText {
    fn mc(self) -> MinecraftText;
}
impl ToMcText for &'_ str {
    fn mc(self) -> MinecraftText {
        self.into()
    }
}
impl ToMcText for Vec<MinecraftText> {
    fn mc(self) -> MinecraftText {
        self.into()
    }
}

pub(crate) trait JoinMcText {
    fn join(slice: Self, sep: MinecraftText) -> MinecraftText;
}
impl JoinMcText for Vec<MinecraftText> {
    fn join(slice: Self, sep: MinecraftText) -> MinecraftText {
        let mut iter = slice.into_iter();
        let Some(mut text) = iter.next() else {
            return "".mc();
        };

        for line in iter {
            text.push(sep.clone());
            text.push(line);
        }
        text
    }
}

#[derive(Default, Clone, Debug)]
pub struct RichText {
    text: String,
    bold: bool,
    italic: bool,
    underlined: bool,
    color: Option<TextColour>,
    click_event: Option<ClickEvent>,
}

#[derive(Copy, Clone, Debug)]
pub enum TextColour {
    Rgb { red: u8, green: u8, blue: u8 },
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl TextColour {
    pub fn try_parse(s: &str) -> Option<Self> {
        let res = match s.to_ascii_lowercase().as_str() {
            "black" => TextColour::Black,
            "dark_blue" => TextColour::DarkBlue,
            "dark_green" => TextColour::DarkGreen,
            "dark_aqua" => TextColour::DarkAqua,
            "dark_red" => TextColour::DarkRed,
            "dark_purple" => TextColour::DarkPurple,
            "gold" => TextColour::Gold,
            "gray" => TextColour::Gray,
            "dark_gray" => TextColour::DarkGray,
            "blue" => TextColour::Blue,
            "green" => TextColour::Green,
            "aqua" => TextColour::Aqua,
            "red" => TextColour::Red,
            "light_purple" => TextColour::LightPurple,
            "yellow" => TextColour::Yellow,
            "white" => TextColour::White,
            mut s => {
                if s.starts_with('#') {
                    s = &s[1..];
                }

                if s.len() != 6 {
                    return None;
                }

                let chars = s.chars().collect::<Vec<_>>();

                let &[r, g, b] = chars.chunks(2).collect::<Vec<_>>().as_slice() else {
                    // Shouldn't happen, since we are of length 6, and are collecting 3 chunks of length 2
                    return None;
                };

                let [Ok(red), Ok(green), Ok(blue)] = [r, g, b]
                    .map(|x| u8::from_str_radix(x.iter().collect::<String>().as_str(), 16))
                else {
                    // Failed to parse one of the numbers from hex
                    return None;
                };

                TextColour::Rgb { red, green, blue }
            }
        };
        Some(res)
    }
    pub fn cmd(self) -> String {
        match self {
            TextColour::Rgb { red, green, blue } => format!("\"#{red:02X}{green:02X}{blue:02X}\""),
            TextColour::Black => "black".to_string(),
            TextColour::DarkBlue => "dark_blue".to_string(),
            TextColour::DarkGreen => "dark_green".to_string(),
            TextColour::DarkAqua => "dark_aqua".to_string(),
            TextColour::DarkRed => "dark_red".to_string(),
            TextColour::DarkPurple => "dark_purple".to_string(),
            TextColour::Gold => "gold".to_string(),
            TextColour::Gray => "gray".to_string(),
            TextColour::DarkGray => "dark_gray".to_string(),
            TextColour::Blue => "blue".to_string(),
            TextColour::Green => "green".to_string(),
            TextColour::Aqua => "aqua".to_string(),
            TextColour::Red => "red".to_string(),
            TextColour::LightPurple => "light_purple".to_string(),
            TextColour::Yellow => "yellow".to_string(),
            TextColour::White => "white".to_string(),
        }
    }
    pub const ALL_CONSTANTS: [Self; 16] = [
        TextColour::Black,
        TextColour::DarkBlue,
        TextColour::DarkGreen,
        TextColour::DarkAqua,
        TextColour::DarkRed,
        TextColour::DarkPurple,
        TextColour::Gold,
        TextColour::Gray,
        TextColour::DarkGray,
        TextColour::Blue,
        TextColour::Green,
        TextColour::Aqua,
        TextColour::Red,
        TextColour::LightPurple,
        TextColour::Yellow,
        TextColour::White,
    ];
}

#[derive(Debug, Clone)]
enum ClickEvent {
    ChangePage {
        page: u32,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_text() {
        let text: MinecraftText = "wibble".into();
        let result = text.cmd();
        assert_eq!(result, r#""wibble""#);
    }
    #[test]
    fn array_text() {
        let text: MinecraftText = ["wibble", ", ", "wobble"]
            .map(str::to_string)
            .map(MinecraftText::String)
            .to_vec()
            .into();
        let result = text.cmd();
        assert_eq!(result, r#"["wibble",", ","wobble"]"#);
    }
    #[test]
    fn underline_written_only_when_true() {
        for underlined in [false, true] {
            let text = MinecraftText::Rich(RichText {
                underlined,
                ..RichText::default()
            });
            let result = text.cmd();
            assert_eq!(result.contains("underlined"), underlined);
        }
    }
    #[test]
    fn bold_written_only_when_true() {
        for bold in [false, true] {
            let text = MinecraftText::Rich(RichText {
                bold,
                ..RichText::default()
            });
            let result = text.cmd();
            assert_eq!(result.contains("bold"), bold);
        }
    }
    #[test]
    fn italic_written() {
        for italic in [false, true] {
            let text = MinecraftText::Rich(RichText {
                italic,
                ..RichText::default()
            });
            let result = text.cmd();
            let expected = format!("italic:{italic}");
            assert!(result.contains(&expected));
        }
    }
    #[test]
    fn colour_none_not_written() {
        let text = MinecraftText::Rich(RichText {
            color: None,
            ..RichText::default()
        });
        let result = text.cmd();
        assert!(!result.contains("color"));
    }
    #[test]
    fn colour_name_written() {
        use TextColour::*;
        for colour in [
            Black,
            DarkBlue,
            DarkGreen,
            DarkAqua,
            DarkRed,
            DarkPurple,
            Gold,
            Gray,
            DarkGray,
            Blue,
            Green,
            Aqua,
            Red,
            LightPurple,
            Yellow,
            White,
        ] {
            let text = MinecraftText::Rich(RichText {
                color: Some(colour),
                ..RichText::default()
            });
            let result = text.cmd();
            assert!(result.contains("color:"));
        }
    }
}
