use super::*;

// ============================================================================================
// 第二、三段共用：片文件（一片一份，写完才在末行写 `name=shard_done`，先写临时名再改名）与片文件里的产物行读回。
// ============================================================================================

/// 片文件末行：`E7RESULT name=shard_done shard=<片名> lines=<整份行数，含这一行>`。
fn shard_done_line(shard: &str, line_count_including_this_one: usize) -> String {
    format!("E7RESULT name=shard_done shard={shard} lines={line_count_including_this_one}")
}

/// 一片的行（不含末行）连同末行写进 `<路径>.partial-<进程号>`、刷盘，再改名成 `<路径>`：被打断时片文件要么不在、要么整份在。
pub(super) fn write_shard_file_atomically(
    path: &Path,
    shard: &str,
    lines: &[String],
) -> std::io::Result<()> {
    let partial = PathBuf::from(format!("{}.partial-{}", path.display(), std::process::id()));
    {
        let mut file = std::fs::File::create(&partial)?;
        for line in lines {
            writeln!(file, "{line}")?;
        }
        writeln!(file, "{}", shard_done_line(shard, lines.len() + 1))?;
        file.sync_all()?;
    }
    std::fs::rename(&partial, path)
}

/// 片文件做完了：末行逐字等于这一片的完成行（行数对得上）。做完的交回不含末行的那些行。
pub(super) fn lines_of_a_finished_shard(text: &str, shard: &str) -> Option<Vec<String>> {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let last = lines.last()?;
    (*last == shard_done_line(shard, lines.len())).then(|| lines[..lines.len() - 1].to_vec())
}

/// 一行产物：`E7RESULT name=<名> <键>=<值> …`。第二、三段打的行值里都没有空格，按空白切、按第一个 `=` 分键值。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResultLine {
    pub(super) fields: Vec<(String, String)>,
}

impl ResultLine {
    pub(super) fn parse(line: &str) -> Option<ResultLine> {
        let body = line.strip_prefix("E7RESULT ")?;
        let fields = body
            .split_whitespace()
            .filter_map(|word| word.split_once('='))
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();
        Some(ResultLine { fields })
    }

    pub(super) fn field(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(field_key, _)| field_key == key)
            .map(|(_, value)| value.as_str())
    }

    pub(super) fn name(&self) -> Option<&str> {
        self.field("name")
    }

    pub(super) fn number(&self, key: &str) -> Option<u64> {
        self.field(key).and_then(|value| value.parse().ok())
    }

    pub(super) fn truth(&self, key: &str) -> Option<bool> {
        match self.field(key) {
            Some("true") => Some(true),
            Some("false") => Some(false),
            Some(_) | None => None,
        }
    }
}

/// 一片里叫这个名字的行。
pub(super) fn lines_named<'lines>(
    lines: &'lines [ResultLine],
    name: &str,
) -> Vec<&'lines ResultLine> {
    lines
        .iter()
        .filter(|line| line.name() == Some(name))
        .collect()
}

/// 确定性核（登记第五节「另在每段第一格多跑 1 轮核两轮逐字段相同」、停机 T4）要去掉的字段：挂钟、进程的高水位与片名、程序文件的 sha256。
fn is_a_field_left_out_of_the_determinism_check(key: &str) -> bool {
    key.ends_with("nanoseconds")
        || key == "vm_high_water_mark_kibibytes"
        || key == "shard"
        || key == "binary_sha256"
}

/// 两份片逐行、逐字段比（去掉 [`is_a_field_left_out_of_the_determinism_check`] 那几样）；交回第一处不同的行号（从 1 起），全同交回 `None`。
pub(super) fn first_line_differing_outside_the_timing_fields(
    first: &[ResultLine],
    second: &[ResultLine],
) -> Option<usize> {
    let kept = |line: &ResultLine| -> Vec<(String, String)> {
        line.fields
            .iter()
            .filter(|(key, _)| !is_a_field_left_out_of_the_determinism_check(key))
            .cloned()
            .collect()
    };
    let longest = first.len().max(second.len());
    (0..longest)
        .find(|index| match (first.get(*index), second.get(*index)) {
            (Some(left), Some(right)) => kept(left) != kept(right),
            (Some(_), None) | (None, Some(_)) | (None, None) => true,
        })
        .map(|index| index + 1)
}

/// 进程此刻的内存高水位（KiB）：`/proc/self/status` 的 VmHWM，在 Linux 上与 `getrusage` 的 `ru_maxrss` 是同一个数（登记 X7「`ru_maxrss` 附带报」）。
pub(super) fn vm_high_water_mark_kibibytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|number| number.parse().ok())
}

#[cfg(test)]
mod tests;
