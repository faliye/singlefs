//! 结果行：写进产物、从产物读回（续跑）。

use super::*;

// ---- 结果行：写、读回（续跑） ----

/// 结果行里的值不带空白：空白换成下划线。
pub(super) fn field_value(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_whitespace() {
                '_'
            } else {
                character
            }
        })
        .collect()
}

/// 产物里的一条结果行：种类（`name=` 的值）与其余的键值。
pub(super) type KindAndFieldsOfAResultLine = (String, BTreeMap<String, String>);

/// 一行结果拆成种类与键值；不是本装置的结果行交回 `None`。
pub(super) fn fields_of_a_result_line(line: &str) -> Option<KindAndFieldsOfAResultLine> {
    let mut words = line.split_whitespace();
    if words.next() != Some(RESULT_LINE_PREFIX) {
        return None;
    }
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    for word in words {
        let (key, value) = word.split_once('=')?;
        fields.insert(key.to_string(), value.to_string());
    }
    let name = fields.remove("name")?;
    Some((name, fields))
}

/// 产物：标准输出与 `--product` 那份文件上同样的结果行；文件只追加。
pub(super) struct ProductWriter {
    file: std::fs::File,
    emitted: u64,
}

impl ProductWriter {
    pub(super) fn appending_to(path: &Path) -> Result<ProductWriter, String> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("建产物目录 {}：{error}", parent.display()))?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| format!("打开产物 {}：{error}", path.display()))?;
        Ok(ProductWriter { file, emitted: 0 })
    }

    pub(super) fn emit(&mut self, name: &str, fields: &str) {
        let line = if fields.is_empty() {
            format!("{RESULT_LINE_PREFIX} name={name}")
        } else {
            format!("{RESULT_LINE_PREFIX} name={name} {fields}")
        };
        println!("{line}");
        if let Err(error) = writeln!(self.file, "{line}").and_then(|()| self.file.flush()) {
            eprintln!("写产物失败：{error}");
            std::process::exit(EXIT_RUN_CONDITION_NOT_MET);
        }
        self.emitted += 1;
    }

    pub(super) fn finish(mut self) {
        let emitted = self.emitted + 1;
        self.emit("done", &format!("emitted={emitted}"));
    }
}

/// 读回产物里已有的结果行（续跑）；文件不在交回空。
pub(super) fn result_lines_already_in_the_product(
    path: &Path,
) -> Result<Vec<KindAndFieldsOfAResultLine>, String> {
    match std::fs::File::open(path) {
        Ok(file) => std::io::BufReader::new(file)
            .lines()
            .map(|line| line.map_err(|error| format!("读产物 {}：{error}", path.display())))
            .filter_map(|line| match line {
                Ok(line) => fields_of_a_result_line(&line).map(Ok),
                Err(error) => Some(Err(error)),
            })
            .collect(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(format!("打开产物 {}：{error}", path.display())),
    }
}
