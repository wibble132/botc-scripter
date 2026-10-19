use std::fmt::Write;
use std::mem;

#[derive(Clone)]
pub enum MinecraftText {
    String(String),
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
                    _ = write!(out, ",color:{c}");
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
    pub fn italic(self) -> Self {
        self.richen(|r| r.italic = true)
    }
    pub fn underlined(self) -> Self {
        self.richen(|r| r.underlined = true)
    }
    pub fn color(self, color: &str) -> Self {
        self.richen(|r| r.color = Some(color.to_string()))
    }
    fn richen<Fn>(mut self, f: Fn) -> Self
    where
        Fn: FnOnce(&mut RichText),
    {
        match &mut self {
            MinecraftText::String(s) => {
                let text = mem::take(s);
                let mut rich = RichText {
                    text,
                    ..RichText::default()
                };
                f(&mut rich);
                self = MinecraftText::Rich(rich);
            }
            MinecraftText::Array(_) => panic!("Not supported"),
            MinecraftText::Rich(r) => f(r),
        }

        self
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

#[derive(Default, Clone)]
pub struct RichText {
    text: String,
    bold: bool,
    italic: bool,
    underlined: bool,
    color: Option<String>,
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
}
