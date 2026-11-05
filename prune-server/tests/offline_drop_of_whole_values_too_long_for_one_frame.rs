//! 离线子命令 `drop-oversized-values` 的认法与删法（`prune_server::oversized_values` 文件头）：每条用例在自己独占的临时目录里用服务写库的那一条路
//! （`VerdictLibrary` 的组提交）造一份库，写一行比「一把键单独一帧读得回」长一个字节的整值、一行恰好最长读得回的整值、一份真分段存的值
//! （清单、段键、计数键，照 `plan_segmented_write` 拆）、一行形状是段键而值同样超界的行、判定向量两张表的空值行与一行小值；
//! 再照只列与删两种跑，核认出的只有超界的那一行整值、库里别的行逐把不变。不起服务、不连真服务。
//! 界不从被测模块取：照 `protocol` 文件头的回应帧格式（帧长上限减回应帧头、个数、在不在、值长）在这里另算一遍。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use prune_server::break_switches::BreakSwitches;
use prune_server::oversized_values::{
    find_oversized_values_then_drop_unless_listing_only, OversizedValue, OversizedValueRemoval,
};
use prune_server::protocol::{
    EntryInTable, Table, MAXIMUM_FRAME_BODY_BYTES, RESPONSE_HEADER_BYTES,
};
use prune_server::segmented_values::{
    plan_segmented_write, segment_count_key, segment_key, SEGMENT_MANIFEST_MAGIC,
};
use prune_server::storage::VerdictLibrary;

/// 成批点读回应里 `u32 个数` 的字节数（`protocol` 文件头「回应帧」）。
const POINT_READ_COUNT_FIELD_BYTES: usize = 4;
/// 成批点读回应里每个键 `u8 在不在` 的字节数。
const POINT_READ_PRESENCE_FIELD_BYTES: usize = 1;
/// 成批点读回应里在的键 `u32 值长` 的字节数。
const POINT_READ_VALUE_LENGTH_FIELD_BYTES: usize = 4;

/// 比最长读得回的长一个字节的整值：要认出、要删。键头 0x01 是崩溃判定库判定块那张表的表标签。
const OVERSIZED_WHOLE_VALUE_KEY: &[u8] = b"\x01oversized-whole-value";
/// 恰好最长读得回的整值：不认、不删。
const LONGEST_READABLE_WHOLE_VALUE_KEY: &[u8] = b"\x01longest-readable-whole-value";
/// 分段存的值的原键（Python 那几家的键形）；值以清单开头，照分段格式存成清单加一段（`segmented_values` 的 `needs_segments`）。
const SEGMENTED_OWNER_KEY: &[u8] =
    b"verdict-library/singlefs-formal-unit-verdicts/proof-function/keys/segmented.json";
/// 形状是段键、值同样超界的那一行的原键部分：只认原键，这一行不认、不删。
const SEGMENT_SHAPED_OWNER_KEY: &[u8] = b"\x01segment-shaped-owner";
/// 判定向量按内容那张表里绿那一行（键就是单字节 0x02，值空）。
const EMPTY_VERDICT_VECTOR_BY_CONTENT_KEY: &[u8] = b"\x02";
/// 判定向量按编号那张表里绿那一号（0x03 ‖ 编号 0，值空）。
const EMPTY_VERDICT_VECTOR_BY_NUMBER_KEY: &[u8] = b"\x03\x00\x00\x00\x00";
const SMALL_WHOLE_VALUE_KEY: &[u8] = b"unit-verdicts/singlefs-unit-verdicts/kind/records/small";
/// 造库那一批写的客户端身份与请求号（组提交顺带记一行已生效的请求号，记账行留在库里、扫时一起数）。
const FIXTURE_CLIENT_IDENTITY: &[u8] = b"offline-drop-fixture";
const FIXTURE_REQUEST_NUMBER: u64 = 1;

static TEMPORARY_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 一条用例独占的临时目录：放手时删。
struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn create() -> TemporaryDirectory {
        let path = std::env::temp_dir().join(format!(
            "prune-oversized-{}-{}",
            std::process::id(),
            TEMPORARY_DIRECTORY_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir(&path)
            .unwrap_or_else(|error| panic!("建不了临时目录 {}：{error}", path.display()));
        TemporaryDirectory { path }
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.path) {
            eprintln!("删不了用例的临时目录 {}：{error}", self.path.display());
        }
    }
}

/// 一把键单独一帧点读回得了的最长的值：帧长上限减回应帧头、个数、在不在、值长。
fn longest_value_readable_by_a_point_read_of_one_key() -> usize {
    usize::try_from(MAXIMUM_FRAME_BODY_BYTES - RESPONSE_HEADER_BYTES)
        .expect("这个工程只编 64 位目标，u32 放得进 usize")
        - POINT_READ_COUNT_FIELD_BYTES
        - POINT_READ_PRESENCE_FIELD_BYTES
        - POINT_READ_VALUE_LENGTH_FIELD_BYTES
}

/// 在 `library_directory` 造文件头说的那份库；回超界那一行整值的长度。
fn write_fixture_library(library_directory: &Path) -> usize {
    let longest_readable = longest_value_readable_by_a_point_read_of_one_key();
    let oversized_bytes = vec![0x5A_u8; longest_readable + 1];
    let mut segmented_value = SEGMENT_MANIFEST_MAGIC.to_vec();
    segmented_value
        .extend_from_slice(b"a value that starts with the manifest magic is stored as segments");
    let plan = plan_segmented_write(
        vec![(
            Table::Conclusions,
            SEGMENTED_OWNER_KEY.to_vec(),
            segmented_value,
        )],
        &[0],
    )
    .expect("一段就装得下的值，段数在段号装得下的范围里");
    assert!(
        plan.segment_and_count_entries.len() >= 2,
        "以清单开头的值要存成段与计数键，计划里却只有 {} 条",
        plan.segment_and_count_entries.len()
    );
    let segment_shaped_key = segment_key(SEGMENT_SHAPED_OWNER_KEY, 0);
    let mut entries: Vec<(Table, &[u8], &[u8])> = vec![
        (
            Table::Conclusions,
            OVERSIZED_WHOLE_VALUE_KEY,
            oversized_bytes.as_slice(),
        ),
        (
            Table::Conclusions,
            LONGEST_READABLE_WHOLE_VALUE_KEY,
            &oversized_bytes[..longest_readable],
        ),
        (
            Table::Conclusions,
            segment_shaped_key.as_slice(),
            oversized_bytes.as_slice(),
        ),
        (
            Table::Conclusions,
            EMPTY_VERDICT_VECTOR_BY_CONTENT_KEY,
            b"".as_slice(),
        ),
        (
            Table::Conclusions,
            EMPTY_VERDICT_VECTOR_BY_NUMBER_KEY,
            b"".as_slice(),
        ),
        (
            Table::Conclusions,
            SMALL_WHOLE_VALUE_KEY,
            b"small value".as_slice(),
        ),
    ];
    for (table, key, value) in plan
        .segment_and_count_entries
        .iter()
        .chain(&plan.entries_to_write_last)
    {
        entries.push((*table, key.as_slice(), value.as_slice()));
    }
    let library = VerdictLibrary::open(library_directory, 1, BreakSwitches::none())
        .unwrap_or_else(|failure| panic!("造库时开不了临时库：{failure:?}"));
    let entries_in_table: Vec<EntryInTable<'_>> = entries
        .iter()
        .map(|(table, key, value)| EntryInTable {
            table: *table,
            key,
            value,
        })
        .collect();
    let mut group = library.begin_group();
    library.add_write(
        &mut group,
        FIXTURE_CLIENT_IDENTITY,
        FIXTURE_REQUEST_NUMBER,
        &entries_in_table,
    );
    library
        .commit_group(group)
        .unwrap_or_else(|failure| panic!("造库时写不进去：{}", failure.message()));
    oversized_bytes.len()
}

/// 库里每一行的键与值长（记账行也在内），按键升序：只读打开，读完就关。
fn rows_of_the_library(library_directory: &Path) -> BTreeMap<Vec<u8>, usize> {
    let database =
        rocksdb::DB::open_for_read_only(&rocksdb::Options::default(), library_directory, false)
            .unwrap_or_else(|error| panic!("只读开不了临时库：{error}"));
    database
        .iterator(rocksdb::IteratorMode::Start)
        .map(|item| {
            let (key, value) = item.unwrap_or_else(|error| panic!("扫临时库失败：{error}"));
            (key.into_vec(), value.len())
        })
        .collect()
}

#[test]
fn listing_only_names_just_the_whole_value_one_byte_past_the_frame_bound_and_leaves_the_library_unchanged(
) {
    let directory = TemporaryDirectory::create();
    let library_directory = directory.path.join("library");
    let oversized_length = write_fixture_library(&library_directory);
    let rows_before = rows_of_the_library(&library_directory);
    let report = find_oversized_values_then_drop_unless_listing_only(
        &library_directory,
        OversizedValueRemoval::ListOnly,
    )
    .unwrap_or_else(|failure| panic!("只列那一趟失败：{failure:?}"));
    assert_eq!(
        report.oversized_values,
        vec![OversizedValue {
            key: OVERSIZED_WHOLE_VALUE_KEY.to_vec(),
            value_length_in_bytes: oversized_length,
        }],
        "只该认出比最长读得回长一个字节的那一行整值；恰好最长读得回的、段键形状的、分段存的、判定向量空值的都不该认"
    );
    assert_eq!(
        report.scanned_rows,
        u64::try_from(rows_before.len()).expect("几十行放得进 u64"),
        "扫过的行数该是库里全部的行"
    );
    assert_eq!(
        rows_of_the_library(&library_directory),
        rows_before,
        "只列（--dry-run）之后库里每一行都该原样在"
    );
}

#[test]
fn dropping_removes_just_that_row_and_keeps_segments_and_empty_verdict_vector_rows() {
    let directory = TemporaryDirectory::create();
    let library_directory = directory.path.join("library");
    let oversized_length = write_fixture_library(&library_directory);
    let rows_before = rows_of_the_library(&library_directory);
    assert!(
        rows_before.contains_key(&segment_count_key(SEGMENTED_OWNER_KEY))
            && rows_before.contains_key(&segment_key(SEGMENTED_OWNER_KEY, 0)),
        "造库之后分段存的值该有计数键与段键"
    );
    let report = find_oversized_values_then_drop_unless_listing_only(
        &library_directory,
        OversizedValueRemoval::DropAndReadBack,
    )
    .unwrap_or_else(|failure| panic!("删那一趟失败：{failure:?}"));
    assert_eq!(
        report.oversized_values,
        vec![OversizedValue {
            key: OVERSIZED_WHOLE_VALUE_KEY.to_vec(),
            value_length_in_bytes: oversized_length,
        }],
        "删的清单只该是比最长读得回长一个字节的那一行整值"
    );
    let mut rows_expected_after = rows_before;
    assert_eq!(
        rows_expected_after.remove(OVERSIZED_WHOLE_VALUE_KEY),
        Some(oversized_length),
        "删之前库里该有超界的那一行整值"
    );
    assert_eq!(
        rows_of_the_library(&library_directory),
        rows_expected_after,
        "删完库里该只少超界的那一行整值：段键形状的、分段存的清单与段与计数键、判定向量空值的、记账行都该原样在"
    );
}
