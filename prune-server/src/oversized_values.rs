//! 离线清掉判定库里一把键单独一帧也读不回的整值：`prune-server drop-oversized-values [--state-directory <目录>] [--dry-run]`
//! （用法与退出码在 `src/bin/prune_server.rs` 文件头）。
//!
//! 为什么要有：大值分段（`segmented_values.rs`）之前写进来的整值，有的单个就长过成批点读一帧回得了的长度；服务对这种键整帧拒
//! （[`point_read_response_exceeds_frame_limit`]），容错客户端拆到一把键还被拒，那一族的入口永远读不出这把键。
//! 判定库不认旧写法（`.claude/rules/verification.md`「判定库只有一份」）：读路径上不加兼容，服务停着时由这里直接开库把这种行删掉，
//! 删掉的判定由各入口下次经服务现判、照键分段写回。
//!
//! 认哪几行：默认列族（库只有这一个列族，`storage.rs` 文件头）里键不是段键、计数键的形状
//! （[`has_the_shape_of_a_segment_or_count_key`]），值长让「只读这一把键」的成批点读回应超过帧长上限的行。
//! 界与服务整帧拒的判法是同一份（[`point_read_response_payload_bytes`]、[`point_read_response_exceeds_frame_limit`]），这里不另写一个数。
//! 判定向量两张表的空值行（`crates/singlefs-verification/src/verdict_store.rs` 文件头）值长 0，认不上。
//!
//! 次序：开库（只开已有的库、不建；服务开着它时 RocksDB 的 LOCK 拒开，先 `control stop`）→ 从头扫一遍认出清单 →
//! [`OversizedValueRemoval::ListOnly`]（`--dry-run`）到这里就关库，库一个字节不动；[`OversizedValueRemoval::DropAndReadBack`]
//! 把清单里的键放进一个写批删掉、同步 WAL，再逐把点读核没了。扫与删在同一次开库里，锁在这个进程手里，中间没有别的写者。

use std::path::{Path, PathBuf};

use rocksdb::{IteratorMode, WriteBatch, WriteOptions, DB};

use crate::break_switches::BreakSwitches;
use crate::command_line::report_refusal;
use crate::configuration::{default_state_directory, library_directory_in};
use crate::protocol::{
    point_read_response_exceeds_frame_limit, point_read_response_payload_bytes,
    MAXIMUM_FRAME_BODY_BYTES, RESPONSE_HEADER_BYTES,
};
use crate::segmented_values::has_the_shape_of_a_segment_or_count_key;
use crate::server::{DESCRIPTOR_LIMIT_TOO_LOW_EXIT_CODE, LIBRARY_MISMATCHED_EXIT_CODE};
use crate::storage::{
    library_options, open_single_column_family_library, LibraryOpenFailure,
    DESCRIPTOR_LIMIT_NEXT_STEP,
};

pub const DROP_OVERSIZED_VALUES_USAGE: &str =
    "用法：prune-server drop-oversized-values [--state-directory <目录>] [--dry-run]（服务停着时跑；库是状态目录下的 library）";
/// 清单里每一行的开头。
pub const OVERSIZED_VALUE_LINE_PREFIX: &str = "PRUNE_SERVER_OVERSIZED_VALUE";
/// 末行的开头。
pub const DROP_OVERSIZED_VALUES_SUMMARY_PREFIX: &str = "PRUNE_SERVER_DROP_OVERSIZED_VALUES";
/// 退出码：库打不开（被服务开着、目录不在）或扫、删、点读时 RocksDB 出错（与服务的「库打不开」同一个码）。
const LIBRARY_UNAVAILABLE_EXIT_CODE: i32 = 3;
/// 退出码：删完点读还读得到清单里的键。
const STILL_PRESENT_AFTER_DROP_EXIT_CODE: i32 = 1;
/// 开库时交给 RocksDB 的后台线程数：只扫一遍、删几把键，用不上压实并行。
const BACKGROUND_THREADS: usize = 1;

/// 认出清单之后做到哪一步。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OversizedValueRemoval {
    /// `--dry-run`：只认出清单，库一个字节不动。
    ListOnly,
    /// 删掉清单里的行、同步 WAL、逐把点读核没了。
    DropAndReadBack,
}

/// 一把键单独一帧也读不回的整值行。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OversizedValue {
    pub key: Vec<u8>,
    pub value_length_in_bytes: usize,
}

#[derive(Debug)]
pub struct OversizedValuesReport {
    /// 扫过的行数（默认列族里全部的行，记账行也算）。
    pub scanned_rows: u64,
    /// 认出的行，按键升序；`DropAndReadBack` 回来时这些行都已删掉、点读核过不在。
    pub oversized_values: Vec<OversizedValue>,
}

#[derive(Debug)]
pub enum OversizedValuesFailure {
    /// 库没开成：库一个字节没动。
    LibraryOpen(LibraryOpenFailure),
    /// 开着库扫、删、同步、点读时 RocksDB 出错。
    Storage(String),
    /// 删完、同步之后点读还读得到的键。
    StillPresentAfterDrop(Vec<Vec<u8>>),
}

/// 值长 `value_length_in_bytes` 的一把键单独一帧点读，服务回不了（整帧拒）。
#[must_use]
pub fn is_too_long_for_a_point_read_of_one_key(value_length_in_bytes: usize) -> bool {
    point_read_response_exceeds_frame_limit(point_read_response_payload_bytes([Some(
        value_length_in_bytes,
    )]))
}

/// 一把键单独一帧点读回得了的最长的值（只给末行报界用，认行照 [`is_too_long_for_a_point_read_of_one_key`]）。
fn longest_value_readable_by_a_point_read_of_one_key() -> usize {
    let frame_limit_in_bytes = usize::try_from(MAXIMUM_FRAME_BODY_BYTES - RESPONSE_HEADER_BYTES)
        .expect("这个工程只编 64 位目标，u32 放得进 usize");
    let longest = frame_limit_in_bytes - point_read_response_payload_bytes([Some(0)]);
    assert!(
        !is_too_long_for_a_point_read_of_one_key(longest)
            && is_too_long_for_a_point_read_of_one_key(longest + 1),
        "报的界与认行的判法对不上：两者都取自成批点读的回应字节数"
    );
    longest
}

/// 这一行要删：不是段键、计数键的形状，值一把键单独一帧也读不回（文件头「认哪几行」）。
fn is_an_oversized_whole_value(key: &[u8], value_length_in_bytes: usize) -> bool {
    !has_the_shape_of_a_segment_or_count_key(key)
        && is_too_long_for_a_point_read_of_one_key(value_length_in_bytes)
}

/// 开 `library_directory` 那份已有的库，认出清单，照 `removal` 只列或删掉再点读核（文件头「次序」）。
///
/// # Errors
/// 库目录不在、被别的进程开着、对不上、文件数上限不够：[`OversizedValuesFailure::LibraryOpen`]，库没动。
/// 扫、删、同步、点读出错：[`OversizedValuesFailure::Storage`]。删完还读得到：[`OversizedValuesFailure::StillPresentAfterDrop`]。
pub fn find_oversized_values_then_drop_unless_listing_only(
    library_directory: &Path,
    removal: OversizedValueRemoval,
) -> Result<OversizedValuesReport, OversizedValuesFailure> {
    if !library_directory.is_dir() {
        return Err(OversizedValuesFailure::LibraryOpen(
            LibraryOpenFailure::Unavailable(format!(
                "库目录 {} 不在（这个子命令只开已有的库、不建）",
                library_directory.display()
            )),
        ));
    }
    let database = open_single_column_family_library(
        library_directory,
        &library_options(BACKGROUND_THREADS, false),
        &BreakSwitches::none(),
    )
    .map_err(OversizedValuesFailure::LibraryOpen)?;
    let mut scanned_rows = 0_u64;
    let mut oversized_values = Vec::new();
    for item in database.iterator(IteratorMode::Start) {
        let (key, value) =
            item.map_err(|error| OversizedValuesFailure::Storage(format!("扫库失败：{error}")))?;
        scanned_rows += 1;
        if is_an_oversized_whole_value(&key, value.len()) {
            oversized_values.push(OversizedValue {
                key: key.into_vec(),
                value_length_in_bytes: value.len(),
            });
        }
    }
    match removal {
        OversizedValueRemoval::ListOnly => Ok(()),
        OversizedValueRemoval::DropAndReadBack => {
            drop_rows_then_read_them_back_absent(&database, &oversized_values)
        }
    }?;
    Ok(OversizedValuesReport {
        scanned_rows,
        oversized_values,
    })
}

/// 把 `oversized_values` 的键放进一个写批删掉、同步 WAL，再逐把点读核没了（文件头「次序」）。
fn drop_rows_then_read_them_back_absent(
    database: &DB,
    oversized_values: &[OversizedValue],
) -> Result<(), OversizedValuesFailure> {
    if !oversized_values.is_empty() {
        let mut batch = WriteBatch::default();
        for oversized in oversized_values {
            batch.delete(&oversized.key);
        }
        let mut unsynced = WriteOptions::default();
        unsynced.set_sync(false);
        database.write_opt(batch, &unsynced).map_err(|error| {
            OversizedValuesFailure::Storage(format!("删除写批写不进去：{error}"))
        })?;
        database.flush_wal(true).map_err(|error| {
            OversizedValuesFailure::Storage(format!("删除之后同步 WAL 失败：{error}"))
        })?;
    }
    let mut still_present = Vec::new();
    for oversized in oversized_values {
        let stored = database.get_pinned(&oversized.key).map_err(|error| {
            OversizedValuesFailure::Storage(format!("删除之后点读失败：{error}"))
        })?;
        if stored.is_some() {
            still_present.push(oversized.key.clone());
        }
    }
    if !still_present.is_empty() {
        return Err(OversizedValuesFailure::StillPresentAfterDrop(still_present));
    }
    Ok(())
}

/// 清单里一行的「表」：键以 ASCII 字母起头的是 Python 几家的行，写族名（第一个「/」之前那一截，带命名空间的连「@命名空间」）；
/// 别的写键头那个字节（崩溃判定库的表标签，`verdict_store.rs` 文件头那张表）。
fn table_of_the_key(key: &[u8]) -> String {
    match key.first() {
        Some(leading_byte) if leading_byte.is_ascii_alphabetic() => {
            let family_end = key
                .iter()
                .position(|byte| *byte == b'/')
                .unwrap_or(key.len());
            key[..family_end].escape_ascii().to_string()
        }
        Some(leading_byte) => format!("0x{leading_byte:02X}"),
        None => "(空键)".to_string(),
    }
}

/// `prune-server drop-oversized-values` 的入口：回退出码。
pub fn run_drop_oversized_values(arguments: &[String]) -> i32 {
    let mut state_directory = None;
    let mut removal = OversizedValueRemoval::DropAndReadBack;
    let mut position = 0;
    while position < arguments.len() {
        match arguments[position].as_str() {
            "--dry-run" => {
                removal = OversizedValueRemoval::ListOnly;
                position += 1;
            }
            "--state-directory" => {
                let Some(value) = arguments.get(position + 1) else {
                    report_refusal("--state-directory 后面缺值", DROP_OVERSIZED_VALUES_USAGE);
                    return 2;
                };
                state_directory = Some(PathBuf::from(value));
                position += 2;
            }
            other => {
                report_refusal(
                    &format!("认不得的参数「{other}」"),
                    DROP_OVERSIZED_VALUES_USAGE,
                );
                return 2;
            }
        }
    }
    let state_directory = match state_directory.map_or_else(default_state_directory, Ok) {
        Ok(directory) => directory,
        Err(message) => {
            report_refusal(&message, "设 HOME，或命令行给 --state-directory <目录>");
            return 2;
        }
    };
    let library_directory = library_directory_in(&state_directory);
    let report =
        match find_oversized_values_then_drop_unless_listing_only(&library_directory, removal) {
            Ok(report) => report,
            Err(failure) => return report_failure(&failure),
        };
    for oversized in &report.oversized_values {
        println!(
            "{OVERSIZED_VALUE_LINE_PREFIX} table={} value_bytes={} key={}",
            table_of_the_key(&oversized.key),
            oversized.value_length_in_bytes,
            oversized.key.escape_ascii()
        );
    }
    let mode = match removal {
        OversizedValueRemoval::ListOnly => "dry-run",
        OversizedValueRemoval::DropAndReadBack => "dropped",
    };
    println!(
        "{DROP_OVERSIZED_VALUES_SUMMARY_PREFIX} mode={mode} library={} scanned_rows={} oversized_rows={} longest_readable_value_bytes={}",
        library_directory.display(),
        report.scanned_rows,
        report.oversized_values.len(),
        longest_value_readable_by_a_point_read_of_one_key()
    );
    0
}

fn report_failure(failure: &OversizedValuesFailure) -> i32 {
    match failure {
        OversizedValuesFailure::LibraryOpen(LibraryOpenFailure::Unavailable(message)) => {
            report_refusal(
                message,
                "服务开着这份库就先 python3 prune-server/control stop 再跑；库目录不对就给 --state-directory（control check 打出库目录）",
            );
            LIBRARY_UNAVAILABLE_EXIT_CODE
        }
        OversizedValuesFailure::LibraryOpen(LibraryOpenFailure::Mismatched(message)) => {
            report_refusal(
                message,
                "库不是服务那一种布局：服务照样拒起（退 6），照 README「库」一节处置",
            );
            LIBRARY_MISMATCHED_EXIT_CODE
        }
        OversizedValuesFailure::LibraryOpen(LibraryOpenFailure::DescriptorLimitTooLow(message)) => {
            report_refusal(message, DESCRIPTOR_LIMIT_NEXT_STEP);
            DESCRIPTOR_LIMIT_TOO_LOW_EXIT_CODE
        }
        OversizedValuesFailure::Storage(message) => {
            report_refusal(
                message,
                "看库所在的盘与文件系统（dmesg、df），盘好了再跑一遍（删除只在一个写批里，没同步的不算删了）",
            );
            LIBRARY_UNAVAILABLE_EXIT_CODE
        }
        OversizedValuesFailure::StillPresentAfterDrop(keys) => {
            let shown: Vec<String> = keys
                .iter()
                .map(|key| key.escape_ascii().to_string())
                .collect();
            report_refusal(
                &format!("删完点读还读得到 {} 把键：{}", keys.len(), shown.join("、")),
                "再跑一遍；还读得到就停在这里交主 agent，不起服务",
            );
            STILL_PRESENT_AFTER_DROP_EXIT_CODE
        }
    }
}
