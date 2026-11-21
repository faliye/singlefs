//! 读 `cargo metadata --format-version 1` 用的最小 JSON 读取器（不加依赖）。只读，不写。

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(String),
    Text(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    pub fn field(&self, name: &str) -> &JsonValue {
        match self {
            JsonValue::Object(fields) => fields.get(name).unwrap_or(&JsonValue::Null),
            JsonValue::Null | JsonValue::Boolean(_) | JsonValue::Number(_) | JsonValue::Text(_) | JsonValue::Array(_) => &JsonValue::Null,
        }
    }

    pub fn array_items(&self) -> &[JsonValue] {
        match self {
            JsonValue::Array(items) => items,
            JsonValue::Null | JsonValue::Boolean(_) | JsonValue::Number(_) | JsonValue::Text(_) | JsonValue::Object(_) => &[],
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            JsonValue::Text(text) => Some(text),
            JsonValue::Null | JsonValue::Boolean(_) | JsonValue::Number(_) | JsonValue::Array(_) | JsonValue::Object(_) => None,
        }
    }
}

struct JsonReader<'text> {
    characters: std::iter::Peekable<std::str::Chars<'text>>,
}

impl JsonReader<'_> {
    fn skip_whitespace(&mut self) {
        while self.characters.peek().is_some_and(|character| character.is_whitespace()) {
            self.characters.next();
        }
    }

    fn expect_character(&mut self, expected: char) {
        self.skip_whitespace();
        let found = self.characters.next();
        assert_eq!(found, Some(expected), "JSON 这里应是 {expected:?}");
    }

    fn read_text(&mut self) -> String {
        self.expect_character('"');
        let mut text = String::new();
        loop {
            let character = self.characters.next().expect("JSON 串没闭合");
            match character {
                '"' => return text,
                '\\' => {
                    let escaped = self.characters.next().expect("JSON 转义没写完");
                    match escaped {
                        'n' => text.push('\n'),
                        't' => text.push('\t'),
                        'r' => text.push('\r'),
                        'b' => text.push('\u{8}'),
                        'f' => text.push('\u{c}'),
                        'u' => {
                            let hexadecimal_digits: String = (0..4).map(|_| self.characters.next().expect("\\u 后面四位")).collect();
                            let code = u32::from_str_radix(&hexadecimal_digits, 16).expect("\\u 后面是十六进制");
                            text.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                        }
                        other => text.push(other),
                    }
                }
                other => text.push(other),
            }
        }
    }

    fn read_value(&mut self) -> JsonValue {
        self.skip_whitespace();
        match self.characters.peek().copied() {
            Some('{') => {
                self.characters.next();
                let mut fields = BTreeMap::new();
                self.skip_whitespace();
                if self.characters.peek() == Some(&'}') {
                    self.characters.next();
                    return JsonValue::Object(fields);
                }
                loop {
                    let name = self.read_text();
                    self.expect_character(':');
                    let value = self.read_value();
                    fields.insert(name, value);
                    self.skip_whitespace();
                    match self.characters.next() {
                        Some(',') => continue,
                        Some('}') => return JsonValue::Object(fields),
                        other => panic!("JSON 对象里遇到 {other:?}"),
                    }
                }
            }
            Some('[') => {
                self.characters.next();
                let mut items = Vec::new();
                self.skip_whitespace();
                if self.characters.peek() == Some(&']') {
                    self.characters.next();
                    return JsonValue::Array(items);
                }
                loop {
                    items.push(self.read_value());
                    self.skip_whitespace();
                    match self.characters.next() {
                        Some(',') => continue,
                        Some(']') => return JsonValue::Array(items),
                        other => panic!("JSON 数组里遇到 {other:?}"),
                    }
                }
            }
            Some('"') => JsonValue::Text(self.read_text()),
            Some('t') | Some('f') | Some('n') => {
                let mut word = String::new();
                while self.characters.peek().is_some_and(|character| character.is_ascii_alphabetic()) {
                    word.push(self.characters.next().expect("刚看过有"));
                }
                match word.as_str() {
                    "true" => JsonValue::Boolean(true),
                    "false" => JsonValue::Boolean(false),
                    "null" => JsonValue::Null,
                    other => panic!("认不出的 JSON 字 {other}"),
                }
            }
            _ => {
                let mut number = String::new();
                while self.characters.peek().is_some_and(|character| character.is_ascii_digit() || "+-.eE".contains(*character)) {
                    number.push(self.characters.next().expect("刚看过有"));
                }
                assert!(!number.is_empty(), "JSON 这里应是一个值");
                JsonValue::Number(number)
            }
        }
    }
}

pub fn parse_json(text: &str) -> JsonValue {
    let mut reader = JsonReader { characters: text.chars().peekable() };
    reader.read_value()
}
