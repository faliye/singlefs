use super::*;

// ───────────────────────────── 块键文件 ─────────────────────────────

const KEYS_FILE_HEADER_PREFIX: &str = "# E184KEYS version=1 expansion=";
const KEYS_FILE_ROW_PREFIX: &str = "# row ";
const KEYS_FILE_TRAILER_PREFIX: &str = "# keys_done blocks=";
pub(super) const FULL_KEYS_FILE_SUFFIX: &str = ".keys";
pub(super) const IN_PLACE_SUBSETS_KEYS_FILE_SUFFIX: &str = ".in_place_subsets.keys";
pub(super) const DUMP_FILE_SUFFIX: &str = ".dump";

/// 块键文件的内容写进 `writer`：头一行写展开方式，第二行抄这条流的 `name=flow` 行，之后每块一行（复用键、计划哈希、块起点、这一块的态数全是十六进制，
/// 最后是点序号），末行写块数——没有末行的文件当没写完。
fn write_keys_file_contents(
    writer: &mut impl std::io::Write,
    row: &FlowRow,
    expansion: Expansion,
    blocks: &[BlockOfAPlan],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "{KEYS_FILE_HEADER_PREFIX}{}\n{KEYS_FILE_ROW_PREFIX}{}",
        expansion.name(),
        row.body()
    )?;
    for block in blocks {
        writeln!(
            writer,
            "{} {} {:x} {:x} {}",
            hexadecimal_text(&block.key.node_input_digest.0),
            hexadecimal_text(&block.key.enumeration_plan_hash.0),
            block.key.block_start.0,
            block.states,
            block.point_index
        )?;
    }
    writeln!(writer, "{KEYS_FILE_TRAILER_PREFIX}{}", blocks.len())
}

#[cfg(test)]
pub(super) fn keys_file_text(
    row: &FlowRow,
    expansion: Expansion,
    blocks: &[BlockOfAPlan],
) -> String {
    let mut bytes = Vec::new();
    write_keys_file_contents(&mut bytes, row, expansion, blocks).expect("写进内存不会失败");
    String::from_utf8(bytes).expect("块键文件里只有 ASCII 与流名")
}

/// 块键文件写到 `path`：先写同目录的临时名（边算边写，不先拼成一整个字符串：最大的流有上千万块）、同步、再改名。
pub(super) fn write_keys_file_atomically(
    path: &Path,
    row: &FlowRow,
    expansion: Expansion,
    blocks: &[BlockOfAPlan],
) -> Result<(), String> {
    let temporary = path.with_extension("partial");
    let created = std::fs::File::create(&temporary)
        .map_err(|error| format!("建 {} 失败：{error}", temporary.display()))?;
    let mut writer = std::io::BufWriter::new(created);
    write_keys_file_contents(&mut writer, row, expansion, blocks)
        .map_err(|error| format!("写 {} 失败：{error}", temporary.display()))?;
    let flushed = writer
        .into_inner()
        .map_err(|error| format!("写 {} 失败：{error}", temporary.display()))?;
    flushed
        .sync_all()
        .map_err(|error| format!("同步 {} 失败：{error}", temporary.display()))?;
    std::fs::rename(&temporary, path).map_err(|error| {
        format!(
            "{} 改名为 {} 失败：{error}",
            temporary.display(),
            path.display()
        )
    })
}

fn digest_from_hexadecimal(text: &str) -> Result<[u8; 32], String> {
    if text.len() != 64 || !text.is_ascii() {
        return Err(format!("摘要要 64 个十六进制字符，读到 {text:?}"));
    }
    let mut digest = [0u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * index..2 * index + 2], 16)
            .map_err(|error| format!("摘要 {text:?}：{error}"))?;
    }
    Ok(digest)
}

/// 读回的块键文件。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct KeysFile {
    pub(super) expansion: Expansion,
    pub(super) row: FlowRow,
    pub(super) blocks: Vec<BlockOfAPlan>,
}

/// 块键文件读到的东西：头、`name=flow` 行、块（`keep_blocks` 是假时只数不存）、末行说的块数与实际数到的块数。
struct KeysFileContents {
    pub(super) expansion: Expansion,
    pub(super) row: FlowRow,
    pub(super) blocks: Vec<BlockOfAPlan>,
    counted_blocks: usize,
    declared_blocks: Option<usize>,
}

fn block_of_the_line(line: &str) -> Result<BlockOfAPlan, String> {
    let words: Vec<&str> = line.split(' ').collect();
    let [reuse_key, plan_hash, block_start, states, point_index] = words.as_slice() else {
        return Err(format!("块键行要 5 段，读到 {line:?}"));
    };
    Ok(BlockOfAPlan {
        key: VerdictBlockKey {
            node_input_digest: NodeInputDigest(digest_from_hexadecimal(reuse_key)?),
            enumeration_plan_hash: EnumerationPlanHash(digest_from_hexadecimal(plan_hash)?),
            block_start: BlockStartStateIndex(
                u64::from_str_radix(block_start, 16).map_err(|error| format!("块起点：{error}"))?,
            ),
        },
        states: u64::from_str_radix(states, 16).map_err(|error| format!("态数：{error}"))?,
        point_index: point_index
            .parse::<usize>()
            .map_err(|error| format!("点序号：{error}"))?,
    })
}

fn read_keys_file_contents(
    reader: impl std::io::BufRead,
    keep_blocks: bool,
) -> Result<KeysFileContents, String> {
    let mut lines = reader
        .lines()
        .map(|line| line.map_err(|error| format!("读块键文件失败：{error}")));
    let header = lines.next().ok_or("块键文件是空的")??;
    let expansion = match header.strip_prefix(KEYS_FILE_HEADER_PREFIX) {
        Some("full") => Expansion::Full,
        Some("in_place_subsets") => Expansion::InPlaceSubsets,
        Some(other) => return Err(format!("展开方式认不出：{other:?}")),
        None => return Err("头一行不是块键文件的头".to_string()),
    };
    let row_line = lines.next().ok_or("缺 name=flow 行")??;
    let row = FlowRow::from_body(
        row_line
            .strip_prefix(KEYS_FILE_ROW_PREFIX)
            .ok_or("第二行不是 # row")?,
    )?;
    let (mut blocks, mut counted_blocks, mut declared_blocks) = (Vec::new(), 0usize, None);
    for line in lines {
        let line = line?;
        if let Some(count) = line.strip_prefix(KEYS_FILE_TRAILER_PREFIX) {
            declared_blocks = Some(
                count
                    .parse::<usize>()
                    .map_err(|error| format!("末行块数：{error}"))?,
            );
            continue;
        }
        if declared_blocks.is_some() {
            return Err("末行之后还有内容".to_string());
        }
        counted_blocks += 1;
        if keep_blocks {
            blocks.push(block_of_the_line(&line)?);
        }
    }
    Ok(KeysFileContents {
        expansion,
        row,
        blocks,
        counted_blocks,
        declared_blocks,
    })
}

impl KeysFile {
    /// [`write_keys_file_contents`] 的逆。头、`name=flow` 行、末行块数与实际块数，任何一样对不上都交回说明。
    fn from_reader(reader: impl std::io::BufRead) -> Result<Self, String> {
        let contents = read_keys_file_contents(reader, true)?;
        match contents.declared_blocks {
            Some(count) if count == contents.counted_blocks => Ok(Self {
                expansion: contents.expansion,
                row: contents.row,
                blocks: contents.blocks,
            }),
            Some(count) => Err(format!(
                "末行说 {count} 块，实际读到 {}",
                contents.counted_blocks
            )),
            None => Err("没有 keys_done 末行：没写完".to_string()),
        }
    }

    #[cfg(test)]
    pub(super) fn from_text(text: &str) -> Result<Self, String> {
        Self::from_reader(text.as_bytes())
    }

    pub(super) fn read_from(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path)
            .map_err(|error| format!("读 {} 失败：{error}", path.display()))?;
        Self::from_reader(std::io::BufReader::new(file))
            .map_err(|error| format!("{}：{error}", path.display()))
    }

    /// 只看头、`name=flow` 行与末行（不存块）：文件写完了没有，说的是哪条流。
    pub(super) fn read_row_of_a_finished_file(path: &Path) -> Result<(Expansion, FlowRow), String> {
        let file = std::fs::File::open(path)
            .map_err(|error| format!("读 {} 失败：{error}", path.display()))?;
        let contents = read_keys_file_contents(std::io::BufReader::new(file), false)
            .map_err(|error| format!("{}：{error}", path.display()))?;
        match contents.declared_blocks {
            Some(count) if count == contents.counted_blocks => {
                Ok((contents.expansion, contents.row))
            }
            Some(_) | None => Err(format!("{}：没写完", path.display())),
        }
    }

    pub(super) fn state_total(&self) -> u64 {
        self.blocks.iter().map(|block| block.states).sum()
    }
}

/// `directories` 里某种后缀的文件，按文件名排；目录读不了交回说明。
pub(super) fn files_with_suffix(
    directories: &[PathBuf],
    suffix: &str,
    excluded_suffix: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for directory in directories {
        let entries = std::fs::read_dir(directory)
            .map_err(|error| format!("读目录 {} 失败：{error}", directory.display()))?;
        for entry in entries {
            let path = entry
                .map_err(|error| format!("读目录 {} 失败：{error}", directory.display()))?
                .path();
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            if name.ends_with(suffix)
                && !excluded_suffix.is_some_and(|excluded| name.ends_with(excluded))
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// 目录里的块键文件路径（`expansion` 那一种）。
pub(super) fn keys_file_paths(
    directories: &[PathBuf],
    expansion: Expansion,
) -> Result<Vec<PathBuf>, String> {
    match expansion {
        Expansion::Full => files_with_suffix(
            directories,
            FULL_KEYS_FILE_SUFFIX,
            Some(IN_PLACE_SUBSETS_KEYS_FILE_SUFFIX),
        ),
        Expansion::InPlaceSubsets => {
            files_with_suffix(directories, IN_PLACE_SUBSETS_KEYS_FILE_SUFFIX, None)
        }
    }
}

/// 读一份块键文件，展开方式要与文件名后缀对得上。
pub(super) fn read_keys_file(path: &Path, expansion: Expansion) -> Result<KeysFile, String> {
    let keys = KeysFile::read_from(path)?;
    if keys.expansion == expansion {
        Ok(keys)
    } else {
        Err(format!("{}：展开方式与文件名后缀对不上", path.display()))
    }
}

/// 目录里的全部块键文件（`expansion` 那一种）；任何一份读不回都交回说明（不当没有）。
pub(super) fn read_keys_files(
    directories: &[PathBuf],
    expansion: Expansion,
) -> Result<Vec<KeysFile>, String> {
    keys_file_paths(directories, expansion)?
        .iter()
        .map(|path| read_keys_file(path, expansion))
        .collect()
}

#[cfg(test)]
mod tests;
