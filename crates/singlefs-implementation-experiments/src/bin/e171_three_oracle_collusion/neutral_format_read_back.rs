//! 中性格式（E170（O3规约执行器第一版） 登记 5.3）的文件名、读回与读回后重建镜像的摘要。

use super::*;

// ───────────────────────────── 中性格式：写出与读回重建（登记 5.3、S4） ─────────────────────────────

pub(super) const BASE_SECTORS_FILE: &str = "base-sectors.bin";
pub(super) const OPERATIONS_FILE: &str = "ops.tsv";
pub(super) const OPERATIONS_PAYLOAD_FILE: &str = "ops-payload.bin";
pub(super) const CALLS_FILE: &str = "calls.tsv";
pub(super) const TORN_FILE: &str = "torn.tsv";
pub(super) const TORN_PAYLOAD_FILE: &str = "torn-payload.bin";
pub(super) const STATES_FILE: &str = "states.tsv";
pub(super) const IMPLEMENTATION_VERDICTS_FILE: &str = "impl-verdicts.tsv";
pub(super) const IN_PLACE_SUBSETS_FILE: &str = "in-place-subsets.tsv";
pub(super) const MANIFEST_FILE: &str = "manifest.tsv";
/// manifest 记 sha256 的那几个文件，按这个次序。
pub(super) const FILES_LISTED_IN_THE_MANIFEST: [&str; 9] = [
    BASE_SECTORS_FILE,
    OPERATIONS_FILE,
    OPERATIONS_PAYLOAD_FILE,
    CALLS_FILE,
    TORN_FILE,
    TORN_PAYLOAD_FILE,
    STATES_FILE,
    IMPLEMENTATION_VERDICTS_FILE,
    IN_PLACE_SUBSETS_FILE,
];

/// 读回来的一条操作：盘身份、种类、偏移、长度、在 `ops-payload.bin` 里的起点（清零与屏障没有）。
pub(super) struct NeutralOperation {
    pub(super) device: u32,
    pub(super) kind: RecordedOperationKind,
    pub(super) offset: u64,
    pub(super) length: u64,
    pub(super) payload_start: Option<usize>,
}

/// 从导出目录读回一条流：基镜像扇区、操作、两份载荷、撕裂表。只读这几个文件，不碰内存里的 `CrashImage`。
pub(super) struct NeutralStream {
    pub(super) base_sectors: BTreeMap<(u32, u64), Vec<u8>>,
    pub(super) operations: Vec<NeutralOperation>,
    pub(super) payload: Vec<u8>,
    pub(super) torn_bytes_by_write: BTreeMap<usize, (usize, usize)>,
    pub(super) torn_payload: Vec<u8>,
}

pub(super) fn read_file(directory: &Path, file_name: &str) -> Vec<u8> {
    std::fs::read(directory.join(file_name))
        .unwrap_or_else(|error| panic!("读 {}：{error}", directory.join(file_name).display()))
}

pub(super) fn read_text_lines(directory: &Path, file_name: &str) -> Vec<String> {
    String::from_utf8(read_file(directory, file_name))
        .expect("导出的 tsv 是 UTF-8")
        .lines()
        .map(str::to_string)
        .collect()
}

pub(super) fn parse_number<Number: std::str::FromStr>(field: &str, what: &str) -> Number {
    field
        .parse()
        .unwrap_or_else(|_| panic!("{what} 解析不了：{field:?}"))
}

pub(super) fn read_neutral_stream(directory: &Path) -> NeutralStream {
    let base_bytes = read_file(directory, BASE_SECTORS_FILE);
    let record_bytes = 4 + 8 + usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
    assert!(
        base_bytes.len().is_multiple_of(record_bytes),
        "base-sectors.bin 是整条记录"
    );
    let base_sectors = base_bytes
        .chunks_exact(record_bytes)
        .map(|record| {
            let device = u32::from_le_bytes(record[0..4].try_into().expect("4 字节"));
            let sector = u64::from_le_bytes(record[4..12].try_into().expect("8 字节"));
            ((device, sector), record[12..].to_vec())
        })
        .collect();
    let operations = read_text_lines(directory, OPERATIONS_FILE)
        .iter()
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 7, "ops.tsv 一行七列：{line:?}");
            let kind = RecordedOperationKind::ALL
                .into_iter()
                .find(|candidate| candidate.name() == fields[2])
                .unwrap_or_else(|| panic!("ops.tsv 里认不出的种类 {:?}", fields[2]));
            NeutralOperation {
                device: parse_number(fields[1], "盘身份"),
                kind,
                offset: parse_number(fields[3], "偏移"),
                length: parse_number(fields[4], "长度"),
                payload_start: (fields[5] != "-").then(|| parse_number(fields[5], "载荷起点")),
            }
        })
        .collect();
    let torn_bytes_by_write = read_text_lines(directory, TORN_FILE)
        .iter()
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 4, "torn.tsv 一行四列：{line:?}");
            (
                parse_number(fields[0], "写序号"),
                (
                    parse_number(fields[1], "撕裂字节起点"),
                    parse_number(fields[2], "撕裂字节长度"),
                ),
            )
        })
        .collect();
    NeutralStream {
        base_sectors,
        operations,
        payload: read_file(directory, OPERATIONS_PAYLOAD_FILE),
        torn_bytes_by_write,
        torn_payload: read_file(directory, TORN_PAYLOAD_FILE),
    }
}

/// 从中性格式按落地向量重建一态的镜像（按流序叠：`2` 放新字节、`1` 放 `torn-payload.bin` 里那一段、`0` 不动），算镜像摘要。
pub(super) fn rebuilt_image_digest(
    stream: &NeutralStream,
    landing_vector: &str,
) -> [u8; DIGEST_BYTES] {
    let sector_bytes = usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
    let mut sectors = stream.base_sectors.clone();
    let mut landings = landing_vector.chars();
    let mut write_index = 0usize;
    for operation in &stream.operations {
        if operation.kind == RecordedOperationKind::Barrier {
            continue;
        }
        let landing = landings.next().expect("落地向量与写数一样长");
        let first_sector = operation.offset / LOCAL_SECTOR_BYTES;
        let length = usize::try_from(operation.length).expect("写长装得进 usize");
        let bytes: Option<&[u8]> = match (landing, operation.kind) {
            ('0', _) => None,
            ('2', RecordedOperationKind::WriteZeroes) => {
                let end_sector = (operation.offset + operation.length) / LOCAL_SECTOR_BYTES;
                let covered: Vec<(u32, u64)> = sectors
                    .range((operation.device, first_sector)..(operation.device, end_sector))
                    .map(|(key, _)| *key)
                    .collect();
                for key in covered {
                    sectors.remove(&key);
                }
                None
            }
            ('2', _) => {
                let start = operation.payload_start.expect("带字节的写有载荷起点");
                Some(&stream.payload[start..start + length])
            }
            ('1', _) => {
                let (start, torn_length) = stream.torn_bytes_by_write[&write_index];
                assert_eq!(torn_length, length, "撕裂那一段罩整次写");
                Some(&stream.torn_payload[start..start + torn_length])
            }
            (unknown, _) => panic!("落地向量里认不出的字符 {unknown:?}"),
        };
        if let Some(bytes) = bytes {
            for (sector_offset, chunk) in bytes.chunks_exact(sector_bytes).enumerate() {
                let sector = first_sector + u64::try_from(sector_offset).expect("扇区下标");
                sectors.insert((operation.device, sector), chunk.to_vec());
            }
        }
        write_index += 1;
    }
    assert!(landings.next().is_none(), "落地向量比写数长");
    let mut message = Vec::new();
    for ((device, sector), bytes) in &sectors {
        append_sector_to_digest_message(&mut message, *device, *sector, bytes);
    }
    sha256_digest(&message)
}

pub(super) fn root_text(root: Option<(InstanceGeneration, CheckpointTxg)>) -> String {
    match root {
        Some((instance, checkpoint_txg)) => format!("{},{}", instance.0, checkpoint_txg.0),
        None => "none".to_string(),
    }
}

/// outcome 的种类；走不下去的那一种带上失败的名字（空白换成下划线，不破坏 tsv 与结果行）。
pub(super) fn outcome_text(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::FileRead { .. } => "file_read".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("failed:{failure:?}")
            .chars()
            .map(|character| {
                if character.is_whitespace() {
                    '_'
                } else {
                    character
                }
            })
            .collect(),
    }
}

pub(super) fn list_text(items: &[String]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(",")
    }
}

pub(super) fn sha256_text(digest: &[u8; DIGEST_BYTES]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests;
