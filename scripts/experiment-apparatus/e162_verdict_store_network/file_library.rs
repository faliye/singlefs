//! 文件库：目录同步、块文件的内容与相对路径、头的编解码与校验、文件库的写读。

use super::*;

// ============================== 候选 F：加固的文件 ==============================

/// 一块一个文件：`<根>/<节点 8 位十六进制>/<段号 8 位十六进制>/<起点 16 位十六进制>-<指纹 32 位十六进制>.blk`。
pub(super) struct FileLibrary {
    root: PathBuf,
    known_directories: HashSet<PathBuf>,
}

fn sync_directory(directory: &Path) -> Result<(), StoreError> {
    File::open(directory)
        .and_then(|handle| handle.sync_all())
        .map_err(|error| file_error(&format!("目录 fsync {}", directory.display()), error))
}

fn format_file_contents() -> [u8; FORMAT_FILE_BYTES] {
    let mut contents = [0u8; FORMAT_FILE_BYTES];
    contents[0..8].copy_from_slice(FORMAT_MAGIC);
    contents[8..12].copy_from_slice(&FILE_FORMAT_VERSION.to_le_bytes());
    let checksum = sender_role::crc32c(&[&contents[0..12]]);
    contents[12..16].copy_from_slice(&checksum.to_le_bytes());
    contents
}

fn block_file_relative_path(key: &[u8]) -> Option<PathBuf> {
    if key.len() != BLOCK_KEY_BYTES {
        return None;
    }
    let node = u32::from_be_bytes(key[0..4].try_into().expect("4 字节"));
    let segment = u32::from_be_bytes(key[4..8].try_into().expect("4 字节"));
    let first_state = u64::from_be_bytes(key[8..16].try_into().expect("8 字节"));
    Some(
        PathBuf::from(format!("{node:08x}"))
            .join(format!("{segment:08x}"))
            .join(format!("{first_state:016x}-{}.blk", hexadecimal(&key[16..32]))),
    )
}

fn temporary_path_of(final_path: &Path) -> PathBuf {
    let mut name = final_path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

/// 文件头 48 字节：魔数｜版本 u32 小端｜块键｜块值长度 u32 小端｜CRC-32C（覆盖头的前 44 字节加整块值）。
fn block_file_header(key: &BlockKey, value: &[u8]) -> [u8; FILE_HEADER_BYTES] {
    let mut header = [0u8; FILE_HEADER_BYTES];
    header[0..4].copy_from_slice(FILE_MAGIC);
    header[4..8].copy_from_slice(&FILE_FORMAT_VERSION.to_le_bytes());
    header[8..40].copy_from_slice(key);
    let value_length = u32::try_from(value.len()).expect("块值装得进 u32");
    header[40..44].copy_from_slice(&value_length.to_le_bytes());
    let checksum = sender_role::crc32c(&[&header[..FILE_HEADER_CHECKSUMMED_BYTES], value]);
    header[44..48].copy_from_slice(&checksum.to_le_bytes());
    header
}

/// 读路径的校验：长度、魔数、版本、头里的键、长度字段、CRC 任一不对就是读报错。
fn validate_block_file(bytes: &[u8], key: &BlockKey) -> Result<Vec<u8>, String> {
    if bytes.len() < FILE_HEADER_BYTES {
        return Err(format!("文件只有 {} 字节，短于文件头", bytes.len()));
    }
    if &bytes[0..4] != FILE_MAGIC {
        return Err("魔数不对".to_string());
    }
    if bytes[4..8] != FILE_FORMAT_VERSION.to_le_bytes() {
        return Err("版本不对".to_string());
    }
    if &bytes[8..40] != key {
        return Err("头里的键与文件名不符".to_string());
    }
    let value_length = u32::from_le_bytes(bytes[40..44].try_into().expect("4 字节"));
    let value = &bytes[FILE_HEADER_BYTES..];
    if u64::from(value_length) != u64::try_from(value.len()).expect("装得进 u64") {
        return Err(format!("长度字段 {value_length} 与文件里的 {} 字节块值不符", value.len()));
    }
    let stored_checksum = u32::from_le_bytes(bytes[44..48].try_into().expect("4 字节"));
    if sender_role::crc32c(&[&bytes[..FILE_HEADER_CHECKSUMMED_BYTES], value]) != stored_checksum {
        return Err("CRC-32C 不对".to_string());
    }
    Ok(value.to_vec())
}

impl FileLibrary {
    /// 打开：根目录在、FORMAT 在且对，否则打开失败；打开时删掉残留的 `*.tmp`。
    /// allow_create：根目录不在时（写入用）先经 `<根>.creating` 建好 FORMAT、改名成根目录。
    pub(super) fn open(root: &Path, allow_create: bool) -> Result<FileLibrary, StoreError> {
        if !root.exists() {
            if !allow_create {
                return Err(StoreError::File(format!("根目录 {} 不在", root.display())));
            }
            Self::create_root_atomically(root)?;
        }
        let format_bytes = fs::read(root.join(FORMAT_FILE_NAME)).map_err(|error| file_error("读 FORMAT", error))?;
        if format_bytes[..] != format_file_contents()[..] {
            return Err(StoreError::File("FORMAT 不对".to_string()));
        }
        let mut known_directories = HashSet::new();
        for node_entry in fs::read_dir(root).map_err(|error| file_error("列根目录", error))? {
            let node_path = node_entry.map_err(|error| file_error("列根目录", error))?.path();
            if !node_path.is_dir() {
                continue;
            }
            known_directories.insert(node_path.clone());
            for segment_entry in fs::read_dir(&node_path).map_err(|error| file_error("列节点目录", error))? {
                let segment_path = segment_entry.map_err(|error| file_error("列节点目录", error))?.path();
                if !segment_path.is_dir() {
                    continue;
                }
                known_directories.insert(segment_path.clone());
                for block_entry in fs::read_dir(&segment_path).map_err(|error| file_error("列段目录", error))? {
                    let block_path = block_entry.map_err(|error| file_error("列段目录", error))?.path();
                    if block_path.extension().is_some_and(|extension| extension == "tmp") {
                        fs::remove_file(&block_path).map_err(|error| file_error("删残留的临时文件", error))?;
                    }
                }
            }
        }
        Ok(FileLibrary { root: root.to_path_buf(), known_directories })
    }

    fn create_root_atomically(root: &Path) -> Result<(), StoreError> {
        let mut creating_name = root.as_os_str().to_owned();
        creating_name.push(".creating");
        let creating = PathBuf::from(creating_name);
        if creating.exists() {
            fs::remove_dir_all(&creating).map_err(|error| file_error("删上一次没建完的根目录", error))?;
        }
        fs::create_dir_all(&creating).map_err(|error| file_error("建根目录", error))?;
        let mut format_file = File::create(creating.join(FORMAT_FILE_NAME)).map_err(|error| file_error("建 FORMAT", error))?;
        format_file.write_all(&format_file_contents()).map_err(|error| file_error("写 FORMAT", error))?;
        format_file.sync_all().map_err(|error| file_error("fsync FORMAT", error))?;
        sync_directory(&creating)?;
        fs::rename(&creating, root).map_err(|error| file_error("根目录改名", error))?;
        let parent = root.parent().expect("根目录有父目录");
        sync_directory(parent)
    }

    /// 节点与段目录这个句柄没见过就建（另一个线程刚建好、撞上 AlreadyExists 也算建好），
    /// 建完对新目录与它的父目录各 fsync：两个线程都 fsync，谁先返回都不靠另一个线程的 fsync 作保。
    fn ensure_directory(&mut self, directory: &Path) -> Result<(), StoreError> {
        if self.known_directories.contains(directory) {
            return Ok(());
        }
        let parent = directory.parent().expect("目录有父目录").to_path_buf();
        if parent != self.root {
            self.ensure_directory(&parent)?;
        }
        match fs::create_dir(directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(file_error("建目录", error)),
        }
        sync_directory(directory)?;
        sync_directory(&parent)?;
        self.known_directories.insert(directory.to_path_buf());
        Ok(())
    }

    fn final_path_of(&self, key: &BlockKey) -> PathBuf {
        self.root.join(block_file_relative_path(key).expect("块键 32 字节"))
    }

    /// 登记的写法：临时文件（一次 write_all）→ sync_all → 改名 → 目录 fsync。
    pub(super) fn put_block(&mut self, key: &BlockKey, value: &[u8]) -> Result<(), StoreError> {
        let final_path = self.final_path_of(key);
        let directory = final_path.parent().expect("块文件有父目录").to_path_buf();
        self.ensure_directory(&directory)?;
        let temporary_path = temporary_path_of(&final_path);
        let mut contents = Vec::with_capacity(FILE_HEADER_BYTES + value.len());
        contents.extend_from_slice(&block_file_header(key, value));
        contents.extend_from_slice(value);
        let mut handle = File::create(&temporary_path).map_err(|error| file_error("建临时文件", error))?;
        handle.write_all(&contents).map_err(|error| file_error("写临时文件", error))?;
        handle.sync_all().map_err(|error| file_error("fsync 临时文件", error))?;
        fs::rename(&temporary_path, &final_path).map_err(|error| file_error("改名", error))?;
        sync_directory(&directory)
    }

    pub(super) fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match fs::read(self.final_path_of(key)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => ReadOutcome::NotFound,
            Err(error) => ReadOutcome::ReadError(error.to_string()),
            Ok(bytes) => match validate_block_file(&bytes, key) {
                Ok(value) => ReadOutcome::Found(value),
                Err(reason) => ReadOutcome::ReadError(reason),
            },
        }
    }

    /// 走目录收全部 `.blk` 文件名（`*.tmp` 与别的文件不算）；键从路径拆回，块值长度 = 文件长度 − 48。
    pub(super) fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
        let mut entries = Vec::new();
        for node_entry in fs::read_dir(&self.root).map_err(|error| file_error("列根目录", error))? {
            let node_path = node_entry.map_err(|error| file_error("列根目录", error))?.path();
            let Some(node) = hexadecimal_name_as_u32(&node_path) else { continue };
            for segment_entry in fs::read_dir(&node_path).map_err(|error| file_error("列节点目录", error))? {
                let segment_path = segment_entry.map_err(|error| file_error("列节点目录", error))?.path();
                let Some(segment) = hexadecimal_name_as_u32(&segment_path) else { continue };
                for block_entry in fs::read_dir(&segment_path).map_err(|error| file_error("列段目录", error))? {
                    let block_entry = block_entry.map_err(|error| file_error("列段目录", error))?;
                    let name = block_entry.file_name().to_string_lossy().to_string();
                    let Some(stem) = name.strip_suffix(".blk") else { continue };
                    let Some((first_state_text, fingerprint_text)) = stem.split_once('-') else { continue };
                    let (Ok(first_state), Some(fingerprint)) =
                        (u64::from_str_radix(first_state_text, 16), parse_hexadecimal(fingerprint_text))
                    else {
                        continue;
                    };
                    let mut key = Vec::with_capacity(BLOCK_KEY_BYTES);
                    key.extend_from_slice(&node.to_be_bytes());
                    key.extend_from_slice(&segment.to_be_bytes());
                    key.extend_from_slice(&first_state.to_be_bytes());
                    key.extend_from_slice(&fingerprint);
                    let file_length = block_entry.metadata().map_err(|error| file_error("读文件长度", error))?.len();
                    entries.push((key, file_length.saturating_sub(u64::try_from(FILE_HEADER_BYTES).expect("48"))));
                }
            }
        }
        Ok(entries)
    }
}
