use super::*;

// ───────────────────────────── 命令行与统计 ─────────────────────────────

/// 命令行上读出来的选项：带值的（可重复）和不带值的开关。
#[derive(Debug, Default)]
pub(super) struct ParsedOptions {
    pub(super) values: BTreeMap<String, Vec<String>>,
    flags: BTreeSet<String>,
}

impl ParsedOptions {
    /// 只许 `value_options` 与 `flag_options` 里的名字；带值的选项后面缺值、写了认不出的选项都交回说明。
    pub(super) fn parse(
        arguments: &[String],
        value_options: &[&str],
        flag_options: &[&str],
    ) -> Result<Self, String> {
        let mut parsed = Self::default();
        let mut remaining = arguments.iter();
        while let Some(option) = remaining.next() {
            if flag_options.contains(&option.as_str()) {
                parsed.flags.insert(option.clone());
            } else if value_options.contains(&option.as_str()) {
                let value = remaining
                    .next()
                    .ok_or_else(|| format!("{option} 后面缺值"))?;
                parsed
                    .values
                    .entry(option.clone())
                    .or_default()
                    .push(value.clone());
            } else {
                return Err(format!("认不出的选项 {option}"));
            }
        }
        Ok(parsed)
    }

    pub(super) fn every_value_given_for(&self, option: &str) -> &[String] {
        self.values.get(option).map_or(&[], Vec::as_slice)
    }

    /// 至少给一次。
    pub(super) fn paths(&self, option: &str) -> Result<Vec<PathBuf>, String> {
        let given = self.every_value_given_for(option);
        if given.is_empty() {
            return Err(format!("缺 {option}"));
        }
        Ok(given.iter().map(PathBuf::from).collect())
    }

    /// 恰好给一次。
    pub(super) fn single(&self, option: &str) -> Result<&str, String> {
        match self.every_value_given_for(option) {
            [only] => Ok(only.as_str()),
            [] => Err(format!("缺 {option}")),
            _ => Err(format!("{option} 只许给一次")),
        }
    }

    pub(super) fn flag(&self, option: &str) -> bool {
        self.flags.contains(option)
    }
}

/// 一组数的最小、中位、最大；空组都是 `None`。偶数个时中位取中间两个的平均。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Spread {
    pub(super) minimum: f64,
    pub(super) median: f64,
    pub(super) maximum: f64,
}

impl Spread {
    pub(super) fn of(values: &[f64]) -> Option<Self> {
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);
        let middle = sorted.len() / 2;
        let median = match sorted.len() {
            0 => return None,
            count if count % 2 == 1 => sorted[middle],
            _ => f64::midpoint(sorted[middle - 1], sorted[middle]),
        };
        Some(Self {
            minimum: sorted[0],
            median,
            maximum: sorted[sorted.len() - 1],
        })
    }

    pub(super) fn text(spread: Option<&Self>) -> String {
        spread.map_or_else(
            || "minimum=none median=none maximum=none".to_string(),
            |reading| {
                format!(
                    "minimum={} median={} maximum={}",
                    reading.minimum, reading.median, reading.maximum
                )
            },
        )
    }
}

/// 一行的写出口：行首 `E7RESULT `，数着打了几行（收尾行报它），打的同时记下来，`--out` 给了就写一份文件。
pub(super) struct Output {
    emitter: Emitter,
    pub(super) lines: Vec<String>,
}

impl Output {
    pub(super) fn new() -> Self {
        Self {
            emitter: Emitter::new(),
            lines: Vec::new(),
        }
    }

    pub(super) fn emit(&mut self, body: &str) {
        self.emitter.emit(body);
        self.lines.push(format!("E7RESULT {body}"));
    }

    pub(super) fn emit_closing_line(&mut self, name: &str, fields: &str) {
        self.emitter.emit_closing_line(name, fields);
        let emitted = self.lines.len() + 1;
        self.lines
            .push(format!("E7RESULT name={name} {fields} emitted={emitted}"));
    }

    /// 打过的全部行写进 `path`（先写临时名再改名）。
    pub(super) fn write_to(&self, path: &Path) -> Result<(), String> {
        let mut text = self.lines.join("\n");
        text.push('\n');
        write_dump::write_bytes_atomically(path, text.as_bytes())
    }
}

#[cfg(test)]
mod tests;
