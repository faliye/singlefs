//! 自证里库布局那三格与删前缀那两格（格表在 `selftest.rs` 的 `CELLS`）：
//! 服务只有自己那一份库、只有默认列族、记账行在客户端碰不到的那一段键里（`storage.rs` 文件头）；库对不上时拒起、退出码点名原因；
//! 软上限压得比库的文件数低时服务开库之前抬到硬上限、开得了，硬上限也不够时退 8 说要几个（`storage.rs` 文件头「文件数上限」）。
//!
//! 删前缀那两格（`delete-prefix-only-under-prefix`、`delete-prefix-counts-before-deleting`）：按键前缀成批删（`protocol.rs` 文件头「删前缀」），
//! 只删那个前缀底下的键、走出前缀就停；同一个请求能只数不删，数的口径和删一样。样本键用崩溃判定库表标签那样的单字节前缀，服务里没有哪张表的专门分支。
//!
//! 样本库照 `crates/singlefs-verification/src/verdict_store.rs` 的开法造与核：建库用 `library_options(LibraryCreation::CreateEmpty)`
//! 那几项（建、目录已在就拒、LZ4）加 `synced_write_options` 的同步写；只读核用 `open_existing_for_reading_only` 那几项
//! （不建、LZ4、RocksDB 的运行日志另放，`open_for_read_only` 不拿锁、只开默认列族）。行的键以表标签起头（`VerdictStoreTable::key_tag`），
//! 库布局行是键 `00`、值布局号 6（大端）。
//! 服务的弄坏开关只交这一格要证红的那几项（`BreakSwitches::only_these`），客户端不带开关。

use std::collections::BTreeMap;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use rocksdb::{
    DBCompressionType, IteratorMode, Options, WriteBatch, WriteOptions, DB,
    DEFAULT_COLUMN_FAMILY_NAME,
};

use crate::break_switches::{BreakSwitch, BreakSwitches, BREAK_VARIABLE};
use crate::client::{RouteAddress, SimpleClient};
use crate::protocol::{ClaimOutcome, Table};
use crate::selftest::{
    line_field, loopback_arguments, server_command, CellContext, CellDirectory,
    SELFTEST_WORKER_THREADS, SERVER_READY_TIMEOUT,
};
use crate::server::{
    DESCRIPTOR_LIMIT_TOO_LOW_EXIT_CODE, LIBRARY_MISMATCHED_EXIT_CODE, LISTENING_LINE_PREFIX,
};
use crate::spawned::{SpawnedProcess, StandardError};
use crate::storage::{claim_key, descriptor_limit, entries_in, CLAIM_KEY_TAG, DESCRIPTOR_HEADROOM};

/// 已生效的请求号那几行的键前缀（`storage.rs` 的同名常量；这里另写一份，是为了核服务写的前缀对不对）。
const APPLIED_REQUEST_KEY_TAG: u8 = 0xF0;
/// 崩溃判定库这一版的布局号（`verdict_store.rs` 的 `LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION`）。
const SAMPLE_LIBRARY_LAYOUT_NUMBER: u32 = 6;
/// 「选项对不上」那一份样本库用的比较器名字（与 RocksDB 缺省的按字节比较器不同名、次序相反）。
const REVERSED_COMPARATOR_NAME: &str = "prune-server-selftest.reversed-bytewise";
/// 服务收 SIGTERM 之后等它退出的上限。
const STOP_TIMEOUT: Duration = Duration::from_secs(30);
/// 拒起的服务没打就绪行之后等它退出的上限。
const REFUSAL_EXIT_TIMEOUT: Duration = Duration::from_secs(15);
/// 认领那一把键的租期：格跑完之前不会过期。
const CLAIM_LEASE_MILLISECONDS: u32 = 600_000;
/// 文件数上限那一格：服务子进程的软上限压到这么多（systemd 用户单元给的 1024 的八分之一，样本库不用造到上千个文件）。
const LOW_DESCRIPTOR_SOFT_LIMIT: u64 = 128;
/// 文件数上限那一格样本库的 SST 数：比压低的软上限多一倍有余，开库时 RocksDB 把它们全打开（`storage.rs` 文件头「文件数上限」）。
const MANY_FILES_SAMPLE_TABLE_FILES: u32 = 300;
/// 文件数上限那一格样本库里行的键的第一个字节：录入块那张表的标签（`verdict_store.rs` 的 `VerdictStoreTable::key_tag`）。
const MANY_FILES_KEY_TAG: u8 = 0x05;

/// 表标签起头、后面 `length` 个 `filler` 字节的键。
fn tagged_key(tag: u8, filler: u8, length: usize) -> Vec<u8> {
    let mut key = Vec::with_capacity(1 + length);
    key.push(tag);
    key.resize(1 + length, filler);
    key
}

/// 库布局行：键 `00`，值布局号（4 字节大端）。
fn layout_row() -> (Vec<u8>, Vec<u8>) {
    (
        vec![0x00],
        SAMPLE_LIBRARY_LAYOUT_NUMBER.to_be_bytes().to_vec(),
    )
}

/// `tag` ‖ `rest`。
fn prefixed(tag: u8, rest: &[u8]) -> Vec<u8> {
    let mut key = Vec::with_capacity(1 + rest.len());
    key.push(tag);
    key.extend_from_slice(rest);
    key
}

/// `verdict_store.rs` 的 `library_options(LibraryCreation::CreateEmpty)`。
fn creation_options() -> Options {
    let mut options = Options::default();
    options.create_if_missing(true);
    options.set_error_if_exists(true);
    options.set_compression_type(DBCompressionType::Lz4);
    options
}

/// `verdict_store.rs` 的 `synced_write_options`。
fn synced_write_options() -> WriteOptions {
    let mut write_options = WriteOptions::default();
    write_options.disable_wal(false);
    write_options.set_sync(true);
    write_options
}

/// 样本库怎么个对不上。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SampleMismatch {
    /// 与 `verdict_store.rs` 建的一样。
    Compatible,
    /// 另建一个列族。
    ExtraColumnFamily(&'static str),
    /// 比较器换成 `REVERSED_COMPARATOR_NAME`。
    ReversedComparator,
}

/// 在还不存在的 `directory` 上造一份样本库、同步写进 `rows`，然后关库。
fn build_sample_library(
    directory: &Path,
    rows: &[(Vec<u8>, Vec<u8>)],
    mismatch: SampleMismatch,
) -> Result<(), String> {
    let mut options = creation_options();
    let opened = match mismatch {
        SampleMismatch::Compatible => DB::open(&options, directory),
        SampleMismatch::ExtraColumnFamily(name) => {
            options.create_missing_column_families(true);
            DB::open_cf(&options, directory, [name])
        }
        SampleMismatch::ReversedComparator => {
            options.set_comparator(
                REVERSED_COMPARATOR_NAME,
                Box::new(|left: &[u8], right: &[u8]| right.cmp(left)),
            );
            DB::open(&options, directory)
        }
    };
    let database =
        opened.map_err(|error| format!("造不了样本库 {}：{error}", directory.display()))?;
    let mut batch = WriteBatch::default();
    for (key, value) in rows {
        batch.put(key, value);
    }
    database
        .write_opt(batch, &synced_write_options())
        .map_err(|error| format!("写不进样本库 {}：{error}", directory.display()))
}

/// 照 `verdict_store.rs` 的 `open_existing_for_reading_only` 只读开库，读出默认列族的全部行；RocksDB 的运行日志写到 `log_directory`。
fn rows_read_only(
    directory: &Path,
    log_directory: &Path,
) -> Result<BTreeMap<Vec<u8>, Vec<u8>>, String> {
    std::fs::create_dir_all(log_directory)
        .map_err(|error| format!("建不了运行日志目录 {}：{error}", log_directory.display()))?;
    let mut options = Options::default();
    options.create_if_missing(false);
    options.set_compression_type(DBCompressionType::Lz4);
    options.set_db_log_dir(log_directory);
    let database = DB::open_for_read_only(&options, directory, false)
        .map_err(|error| format!("只读开不了库 {}：{error}", directory.display()))?;
    let mut rows = BTreeMap::new();
    for item in database.iterator(IteratorMode::Start) {
        let (key, value) =
            item.map_err(|error| format!("读不出库 {} 的行：{error}", directory.display()))?;
        rows.insert(key.into_vec(), value.into_vec());
    }
    Ok(rows)
}

fn column_families_of(directory: &Path) -> Result<Vec<String>, String> {
    DB::list_cf(&Options::default(), directory)
        .map_err(|error| format!("读不出 {} 的列族清单：{error}", directory.display()))
}

/// 起一份服务（状态目录与 socket 在 `state_directory`），不等就绪行；`library` 是 `None` 时不给 `--library`，用状态目录下的缺省库。
fn spawn_server(
    context: &CellContext,
    state_directory: &Path,
    library: Option<&Path>,
    switches: &BreakSwitches,
) -> Result<SpawnedProcess, String> {
    SpawnedProcess::spawn(
        server_process_command(context, state_directory, library, switches)?,
        StandardError::Capture,
    )
}

/// `spawn_server` 起的那条命令（TCP 走回环、弄坏开关只交 `switches`）。
fn server_process_command(
    context: &CellContext,
    state_directory: &Path,
    library: Option<&Path>,
    switches: &BreakSwitches,
) -> Result<Command, String> {
    let mut command = match library {
        Some(library) => server_command(context, state_directory, library)?,
        None => {
            let mut command = Command::new(&context.executable);
            command
                .arg("serve")
                .arg("--selftest-instance")
                .arg("--socket")
                .arg(state_directory.join("server.sock"))
                .arg("--state-directory")
                .arg(state_directory)
                .arg("--worker-threads")
                .arg(SELFTEST_WORKER_THREADS);
            command
        }
    };
    command
        .args(loopback_arguments())
        .env(BREAK_VARIABLE, switches.environment_text());
    Ok(command)
}

/// 起服务、等就绪行，回服务进程、unix socket 那条路与就绪行里的 `library=`。
fn start_server(
    context: &CellContext,
    state_directory: &Path,
    library: Option<&Path>,
    switches: &BreakSwitches,
) -> Result<(SpawnedProcess, RouteAddress, String), String> {
    let mut process = spawn_server(context, state_directory, library, switches)?;
    let line = process
        .wait_for_line_starting_with(LISTENING_LINE_PREFIX, SERVER_READY_TIMEOUT)
        .map_err(|message| {
            format!(
                "服务没起来：{message}；标准错误：{}",
                process.standard_error_text()
            )
        })?;
    let socket = line_field(&line, "unix").ok_or_else(|| format!("就绪行没有 unix=：{line}"))?;
    let library_field =
        line_field(&line, "library").ok_or_else(|| format!("就绪行没有 library=：{line}"))?;
    Ok((
        process,
        RouteAddress::UnixSocket(PathBuf::from(socket)),
        library_field,
    ))
}

/// SIGTERM 停掉自己起的这一份服务、等它退出（库的锁随进程放掉）。
fn stop_server(process: &mut SpawnedProcess) -> Result<(), String> {
    process.send_signal(libc::SIGTERM)?;
    process
        .wait_for_exit(STOP_TIMEOUT)
        .ok_or("服务收了 SIGTERM 还没退出")?;
    Ok(())
}

/// 服务该拒起：没打就绪行就退出，退出码是 `expected_code`、标准错误里有 `expected_text`。
fn expect_refused(
    context: &CellContext,
    library: &Path,
    switches: &BreakSwitches,
    expected_code: i32,
    expected_text: &str,
    what: &str,
) -> Result<(), String> {
    let state = CellDirectory::create()?;
    let process = spawn_server(context, &state.path, Some(library), switches)?;
    expect_process_refused(process, expected_code, &[expected_text], what)
}

/// 已经起了的服务该拒起：没打就绪行就退出，退出码是 `expected_code`、标准错误里 `expected_texts` 每一段都有。
fn expect_process_refused(
    mut process: SpawnedProcess,
    expected_code: i32,
    expected_texts: &[&str],
    what: &str,
) -> Result<(), String> {
    if let Ok(line) =
        process.wait_for_line_starting_with(LISTENING_LINE_PREFIX, SERVER_READY_TIMEOUT)
    {
        return Err(format!("{what}：服务该拒起，却起来了：{line}"));
    }
    let status = process
        .wait_for_exit(REFUSAL_EXIT_TIMEOUT)
        .ok_or_else(|| format!("{what}：服务既没起来也没退出"))?;
    let standard_error = process.standard_error_text();
    if status.code() != Some(expected_code)
        || !expected_texts
            .iter()
            .all(|expected_text| standard_error.contains(expected_text))
    {
        return Err(format!(
            "{what}：服务退出码 {:?}（该是 {expected_code}）、标准错误该点名「{}」；标准错误：{}",
            status.code(),
            expected_texts.join("」「"),
            standard_error.trim()
        ));
    }
    Ok(())
}

pub(crate) fn cell_single_library_layout(context: &CellContext) -> Result<String, String> {
    let switches = context.break_switches.only_these(&[
        BreakSwitch::BookkeepingKeyTagDropped,
        BreakSwitch::ServiceKeysOpenToClients,
    ]);
    let directory = CellDirectory::create()?;
    let expected_library = directory.path.join("library");
    let (mut server, route, library_field) =
        start_server(context, &directory.path, None, &switches)?;
    if Path::new(&library_field) != expected_library {
        return Err(format!(
            "不带 --library 时服务开的库是 {library_field}，该是状态目录下的 {}",
            expected_library.display()
        ));
    }
    let mut client = SimpleClient::connect(&route, BreakSwitches::none())?;

    // 成批写 3 条：两条崩溃行（表标签起头）、一条 Python 那几家的 ASCII 族名起头的键，各用一张表的名义
    let written_rows: Vec<(Table, Vec<u8>, Vec<u8>)> = vec![
        (
            Table::Conclusions,
            tagged_key(0x0A, 0xE5, 105),
            b"conclusion-block".to_vec(),
        ),
        (
            Table::Recording,
            tagged_key(0x05, 0xE6, 80),
            b"recorded-block".to_vec(),
        ),
        (
            Table::Facts,
            b"verdict-library/selftest-family/key".to_vec(),
            b"python-family-row".to_vec(),
        ),
    ];
    let written = client.write(&written_rows)?;
    if written != 3 {
        return Err(format!("写确认说写了 {written} 条，该是 3 条"));
    }
    // 换一张表的名义读：三张表同一个键空间，读回的是同一行
    let reads: Vec<(Table, Vec<u8>)> = written_rows
        .iter()
        .zip([Table::Facts, Table::Conclusions, Table::Recording])
        .map(|((_, key, _), other_table)| (other_table, key.clone()))
        .collect();
    let values = client.read(&reads)?;
    for ((_, key, value), received) in written_rows.iter().zip(&values) {
        if received.as_deref() != Some(value.as_slice()) {
            return Err(format!(
                "用另一张表的名义读 {key:02x?} 回 {received:?}，该是写的值（三张表同在默认列族）"
            ));
        }
    }
    // 认领不分表：一个客户端用结论表认领，另一个用事实表认领同一把键回别人持有
    let claimed_key = tagged_key(0x0A, 0xE7, 105);
    let outcomes = client.claim(
        &[(Table::Conclusions, claimed_key.clone())],
        CLAIM_LEASE_MILLISECONDS,
    )?;
    if outcomes != [ClaimOutcome::Claimed] {
        return Err(format!("认领一把没人持有的键回 {outcomes:?}，该是认领到"));
    }
    let mut other_client = SimpleClient::connect(&route, BreakSwitches::none())?;
    let other_outcomes = other_client.claim(
        &[(Table::Facts, claimed_key.clone())],
        CLAIM_LEASE_MILLISECONDS,
    )?;
    if !matches!(
        other_outcomes.as_slice(),
        [ClaimOutcome::HeldByOther { .. }]
    ) {
        return Err(format!(
            "另一个客户端用事实表的名义认领同一把键回 {other_outcomes:?}，该是别人持有（认领不分表）"
        ));
    }
    // 空前缀扫：只回客户端写的 3 条，记账那一段不回
    let (_, scanned) = client.scan(Table::Facts, b"", 100)?;
    let scanned_keys: Vec<Vec<u8>> = scanned.iter().map(|entry| entry.key.clone()).collect();
    let mut expected_keys: Vec<Vec<u8>> =
        written_rows.iter().map(|(_, key, _)| key.clone()).collect();
    expected_keys.sort();
    if scanned_keys != expected_keys {
        return Err(format!(
            "空前缀扫回 {} 条（{:02x?}），该是客户端写的 3 条；记账那一段（0xF0 起头）不该扫到",
            scanned_keys.len(),
            scanned_keys
                .iter()
                .map(|key| key.first().copied())
                .collect::<Vec<_>>()
        ));
    }
    // 客户端写记账那一段的键：拒（状态 1），连接照常可用
    let service_key = vec![APPLIED_REQUEST_KEY_TAG, 0x01, b'x'];
    if let Ok(accepted) = client.write(&[(Table::Conclusions, service_key, b"forged".to_vec())]) {
        return Err(format!(
            "客户端写 0xF0 起头的键被收了（写确认 {accepted} 条），该回状态 1：这一段留给服务的记账"
        ));
    }
    drop(client);
    drop(other_client);
    stop_server(&mut server)?;

    // 停服务之后直接开库：只有默认列族；行恰好是写的 3 条、请求号 1 行（F0 起头、值是 3）、认领 1 行（F1 ‖ 键）
    let library_column_families = column_families_of(&expected_library)?;
    if library_column_families != [DEFAULT_COLUMN_FAMILY_NAME] {
        return Err(format!(
            "服务的库里的列族是 {library_column_families:?}，该只有 default"
        ));
    }
    let library_rows = rows_read_only(&expected_library, &directory.path.join("read-only-log"))?;
    for (_, key, value) in &written_rows {
        if library_rows.get(key) != Some(value) {
            return Err(format!(
                "写进去的键 {key:02x?} 在库的默认列族里是 {:?}，该是写的值",
                library_rows.get(key)
            ));
        }
    }
    let applied_rows: Vec<&Vec<u8>> = library_rows
        .iter()
        .filter(|(key, _)| key.first() == Some(&APPLIED_REQUEST_KEY_TAG))
        .map(|(_, value)| value)
        .collect();
    if applied_rows != [&3_u32.to_le_bytes().to_vec()] {
        return Err(format!(
            "库里 0xF0 起头的请求号行是 {applied_rows:02x?}，该恰好 1 行、值是 3（写了一帧 3 条）"
        ));
    }
    let claim_row_key = claim_key(&claimed_key);
    if claim_row_key.first() != Some(&CLAIM_KEY_TAG) || !library_rows.contains_key(&claim_row_key) {
        return Err(format!(
            "库里没有认领那一把键的认领行 {:02x?}",
            &claim_row_key[..claim_row_key.len().min(8)]
        ));
    }
    let unexpected: Vec<String> = library_rows
        .keys()
        .filter(|key| {
            key.first() != Some(&APPLIED_REQUEST_KEY_TAG)
                && **key != claim_row_key
                && !written_rows
                    .iter()
                    .any(|(_, written_key, _)| written_key == *key)
        })
        .map(|key| format!("{:02x?}", &key[..key.len().min(8)]))
        .collect();
    if !unexpected.is_empty() {
        return Err(format!(
            "库里多出了既不是写的、也不是记账的行：{}",
            unexpected.join("、")
        ));
    }
    Ok(format!(
        "不带 --library 开的是状态目录下的 library；三张表的名义互读同一行、认领不分表、空前缀扫只回写的 3 条、写 0xF0 起头的键被拒；\
         停服务直接开库：只有 default，行是写的 3 条加请求号 1 行（0xF0）、认领 1 行（0xF1），共 {} 行",
        library_rows.len()
    ))
}

pub(crate) fn cell_library_refuses_mismatched(context: &CellContext) -> Result<String, String> {
    let switches = context.break_switches.only_these(&[
        BreakSwitch::LibraryExtraColumnFamiliesAccepted,
        BreakSwitch::LibraryOpenFailureReportedAsLocked,
        BreakSwitch::LibraryWithoutCurrentFileOpened,
    ]);
    let directory = CellDirectory::create()?;
    let rows = vec![layout_row()];

    let extra_column_family = "extra-family";
    let with_extra_column_family = directory.path.join("with-extra-column-family");
    build_sample_library(
        &with_extra_column_family,
        &rows,
        SampleMismatch::ExtraColumnFamily(extra_column_family),
    )?;
    expect_refused(
        context,
        &with_extra_column_family,
        &switches,
        LIBRARY_MISMATCHED_EXIT_CODE,
        extra_column_family,
        "库里另有列族",
    )?;
    let after = column_families_of(&with_extra_column_family)?;
    if after != [DEFAULT_COLUMN_FAMILY_NAME, extra_column_family] {
        return Err(format!("拒起之后那份库的列族变成了 {after:?}"));
    }

    let with_other_comparator = directory.path.join("with-other-comparator");
    build_sample_library(
        &with_other_comparator,
        &rows,
        SampleMismatch::ReversedComparator,
    )?;
    expect_refused(
        context,
        &with_other_comparator,
        &switches,
        LIBRARY_MISMATCHED_EXIT_CODE,
        "选项与库对不上",
        "库的比较器与 verdict_store.rs 的不同",
    )?;

    let not_a_library = directory.path.join("not-a-library");
    std::fs::create_dir(&not_a_library)
        .map_err(|error| format!("建不了目录 {}：{error}", not_a_library.display()))?;
    std::fs::write(not_a_library.join("stray-file"), b"not rocksdb")
        .map_err(|error| format!("写不了 {} 里的文件：{error}", not_a_library.display()))?;
    expect_refused(
        context,
        &not_a_library,
        &switches,
        LIBRARY_MISMATCHED_EXIT_CODE,
        "CURRENT",
        "目录里有文件却不是 RocksDB 库",
    )?;
    if not_a_library.join("CURRENT").exists() {
        return Err(format!(
            "服务拒起了，却在 {} 里建了库",
            not_a_library.display()
        ));
    }

    let held = directory.path.join("held-library");
    build_sample_library(&held, &rows, SampleMismatch::Compatible)?;
    let holder_state = CellDirectory::create()?;
    let (_holder, _route, _library) =
        start_server(context, &holder_state.path, Some(held.as_path()), &switches)?;
    expect_refused(context, &held, &switches, 3, "LOCK", "库被另一个服务开着")?;
    Ok(format!(
        "库里另有列族（{extra_column_family}）退 {LIBRARY_MISMATCHED_EXIT_CODE} 点名那个列族、列族不变；比较器不同退 {LIBRARY_MISMATCHED_EXIT_CODE} 说选项与库对不上；\
         有文件没有 CURRENT 的目录退 {LIBRARY_MISMATCHED_EXIT_CODE}、没在里面建库；库被另一个服务开着退 3 点名 LOCK"
    ))
}

/// 文件数上限那一格的样本库里每一行的键：录入块那张表的标签起头、后面是行号（服务不看键里是什么，只要不落进 0xF0 起头的记账那一段）。
fn many_files_key(row: u32) -> Vec<u8> {
    prefixed(MANY_FILES_KEY_TAG, &row.to_be_bytes())
}

/// 在还不存在的 `directory` 上照 `verdict_store.rs` 的建法造一份库，每写一行落盘一次、造出 `table_files` 个 SST；
/// 关掉自动压实与 L0 的写入减速和停写，不然 SST 被压实合掉或写到一半停住。回库目录里的文件数与其中的 SST 数。
fn build_many_files_library(directory: &Path, table_files: u32) -> Result<(u64, u64), String> {
    let mut options = creation_options();
    options.set_disable_auto_compactions(true);
    options.set_level_zero_slowdown_writes_trigger(i32::MAX);
    options.set_level_zero_stop_writes_trigger(i32::MAX);
    let database = DB::open(&options, directory)
        .map_err(|error| format!("造不了样本库 {}：{error}", directory.display()))?;
    let (layout_key, layout_value) = layout_row();
    database
        .put(layout_key, layout_value)
        .map_err(|error| format!("写不进样本库 {}：{error}", directory.display()))?;
    for row in 0..table_files {
        database
            .put(many_files_key(row), row.to_be_bytes())
            .map_err(|error| format!("写不进样本库 {}：{error}", directory.display()))?;
        database
            .flush()
            .map_err(|error| format!("样本库 {} 落不了盘：{error}", directory.display()))?;
    }
    drop(database);
    let library_files = entries_in(directory)?;
    let mut table_files_found = 0_u64;
    for entry in std::fs::read_dir(directory)
        .map_err(|error| format!("读不了样本库目录 {}：{error}", directory.display()))?
    {
        let entry = entry.map_err(|error| {
            format!("读不了样本库目录 {} 里的一项：{error}", directory.display())
        })?;
        if entry.file_name().to_string_lossy().ends_with(".sst") {
            table_files_found += 1;
        }
    }
    Ok((library_files, table_files_found))
}

/// 起一份服务，子进程在 exec 之前把自己的 RLIMIT_NOFILE 设成 `limit`（软上限与硬上限一起设，非 root 只往下压得了硬上限）。
fn spawn_server_with_descriptor_limit(
    context: &CellContext,
    state_directory: &Path,
    library: &Path,
    switches: &BreakSwitches,
    limit: libc::rlimit,
) -> Result<SpawnedProcess, String> {
    let mut command = server_process_command(context, state_directory, Some(library), switches)?;
    // SAFETY: pre_exec 的闭包在 fork 之后、exec 之前的子进程里跑，只调 setrlimit 这一个异步信号安全的系统调用，
    // 读的是挪进闭包的一个结构体，不分配内存、不碰锁。
    unsafe {
        command.pre_exec(move || {
            if libc::setrlimit(libc::RLIMIT_NOFILE, &limit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    SpawnedProcess::spawn(command, StandardError::Capture)
}

/// `/proc/<进程号>/limits` 里「Max open files」那一行的软上限与硬上限。
fn descriptor_limit_of(process_identifier: u32) -> Result<(u64, u64), String> {
    let path = format!("/proc/{process_identifier}/limits");
    let text = std::fs::read_to_string(&path).map_err(|error| format!("读不了 {path}：{error}"))?;
    let label = "Max open files";
    let after_label = text
        .lines()
        .find_map(|line| line.strip_prefix(label))
        .ok_or_else(|| format!("{path} 里没有「{label}」那一行：{text}"))?;
    let numbers: Vec<u64> = after_label
        .split_whitespace()
        .filter_map(|field| field.parse().ok())
        .collect();
    match numbers.as_slice() {
        [soft, hard] => Ok((*soft, *hard)),
        _ => Err(format!(
            "{path} 的「{label}」那一行读不出软上限与硬上限：{after_label}"
        )),
    }
}

pub(crate) fn cell_library_opens_under_low_descriptor_soft_limit(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::DescriptorSoftLimitNotRaised]);
    let runner_limit = descriptor_limit()?;
    let directory = CellDirectory::create()?;
    let library = directory.path.join("many-files-library");
    let (library_files, table_files) =
        build_many_files_library(&library, MANY_FILES_SAMPLE_TABLE_FILES)?;
    if table_files < u64::from(MANY_FILES_SAMPLE_TABLE_FILES)
        || library_files <= LOW_DESCRIPTOR_SOFT_LIMIT
    {
        return Err(format!(
            "样本库造坏了：要 {MANY_FILES_SAMPLE_TABLE_FILES} 个 SST、文件数多于软上限 {LOW_DESCRIPTOR_SOFT_LIMIT}，\
             造出 {table_files} 个 SST、{library_files} 个文件，判不出软上限低时开不开得了"
        ));
    }
    // 跑格的进程的硬上限要够服务抬软上限之后开这份库（库的文件、余量、再留一个压低的软上限那么多给进程自己开着的描述符）。
    let hard_needed = library_files + DESCRIPTOR_HEADROOM + LOW_DESCRIPTOR_SOFT_LIMIT;
    if runner_limit.rlim_max < hard_needed {
        return Err(format!(
            "跑格的进程硬上限只有 {}，小于 {hard_needed}（样本库 {library_files} 个文件加余量 {DESCRIPTOR_HEADROOM}）：\
             造不出「硬上限够、软上限低」的服务子进程；在硬上限够的地方跑（统一任务服务的尝试单元软硬上限相同、都是服务那一层的硬上限）",
            runner_limit.rlim_max
        ));
    }

    // 拒起的那条输入先跑：服务开过这份库之后会把 L0 的 SST 压实合掉，库里就不再有那么多文件。
    let refused_state = CellDirectory::create()?;
    let refused = spawn_server_with_descriptor_limit(
        context,
        &refused_state.path,
        &library,
        &switches,
        libc::rlimit {
            rlim_cur: LOW_DESCRIPTOR_SOFT_LIMIT,
            rlim_max: LOW_DESCRIPTOR_SOFT_LIMIT,
        },
    )?;
    let hard_limit_text = format!("、硬上限 {LOW_DESCRIPTOR_SOFT_LIMIT}");
    expect_process_refused(
        refused,
        DESCRIPTOR_LIMIT_TOO_LOW_EXIT_CODE,
        &[
            "文件数上限不够：要 ",
            hard_limit_text.as_str(),
            "/proc/sys/fs/nr_open",
        ],
        &format!("软上限与硬上限都压到 {LOW_DESCRIPTOR_SOFT_LIMIT}"),
    )?;

    let served_state = CellDirectory::create()?;
    let mut served = spawn_server_with_descriptor_limit(
        context,
        &served_state.path,
        &library,
        &switches,
        libc::rlimit {
            rlim_cur: LOW_DESCRIPTOR_SOFT_LIMIT,
            rlim_max: runner_limit.rlim_max,
        },
    )?;
    let line = served
        .wait_for_line_starting_with(LISTENING_LINE_PREFIX, SERVER_READY_TIMEOUT)
        .map_err(|message| {
            format!(
                "软上限压到 {LOW_DESCRIPTOR_SOFT_LIMIT}、硬上限 {}、库里 {library_files} 个文件：服务没起来：{message}；标准错误：{}",
                runner_limit.rlim_max,
                served.standard_error_text()
            )
        })?;
    let (soft_after, hard_after) = descriptor_limit_of(served.process_identifier())?;
    if soft_after != runner_limit.rlim_max || hard_after != runner_limit.rlim_max {
        return Err(format!(
            "服务起来之后软上限该抬到硬上限 {}，读到软上限 {soft_after}、硬上限 {hard_after}",
            runner_limit.rlim_max
        ));
    }
    let socket = line_field(&line, "unix").ok_or_else(|| format!("就绪行没有 unix=：{line}"))?;
    let mut client = SimpleClient::connect(
        &RouteAddress::UnixSocket(PathBuf::from(socket)),
        BreakSwitches::none(),
    )?;
    let probe_row = MANY_FILES_SAMPLE_TABLE_FILES - 1;
    let read_back = client.read(&[(Table::Recording, many_files_key(probe_row))])?;
    if read_back != [Some(probe_row.to_be_bytes().to_vec())] {
        return Err(format!(
            "开了文件很多的库之后点读最后一个 SST 里的那一行，读回 {read_back:?}"
        ));
    }
    drop(client);
    stop_server(&mut served)?;
    Ok(format!(
        "样本库 {library_files} 个文件（{table_files} 个 SST）：软上限压到 {LOW_DESCRIPTOR_SOFT_LIMIT}、硬上限 {} 时服务开得了、\
         就绪后软上限是 {soft_after}、点读最后一个 SST 里的行对上；软上限与硬上限都压到 {LOW_DESCRIPTOR_SOFT_LIMIT} 时退 \
         {DESCRIPTOR_LIMIT_TOO_LOW_EXIT_CODE}、标准错误写「文件数上限不够：要 …{hard_limit_text}」与出路",
        runner_limit.rlim_max
    ))
}

/// 删前缀那两格的目标前缀只有一个字节：崩溃判定库一张表的标签（`verdict_store.rs` 的 `VerdictStoreTable::key_tag` 里的一个）。
/// 服务里没有哪张表的专门分支，这个字节和 `b"count/pages/"` 那样的 ASCII 前缀走同一条路。
const DELETE_PREFIX_TARGET_TAG: u8 = 0x1C;

/// 起一份只交 `switches` 的服务、连一条一问一答的客户端；回这一格的临时目录、服务进程与客户端（字段次序就是放手次序的反面：先放客户端、再服务、最后目录）。
fn start_delete_prefix_cell(
    context: &CellContext,
    switches: &BreakSwitches,
) -> Result<(CellDirectory, SpawnedProcess, SimpleClient), String> {
    let directory = CellDirectory::create()?;
    let (server, route, _library) = start_server(context, &directory.path, None, switches)?;
    let client = SimpleClient::connect(&route, BreakSwitches::none())?;
    Ok((directory, server, client))
}

/// 库里客户端看得到的全部键（空前缀扫，记账那一段不回），按键升序。
fn keys_in_library(client: &mut SimpleClient) -> Result<Vec<Vec<u8>>, String> {
    let (is_truncated, scanned) = client.scan(Table::Facts, b"", 1000)?;
    if is_truncated {
        return Err("空前缀扫被截断了：这一格写的键远不到 1000 把".to_string());
    }
    Ok(scanned.into_iter().map(|entry| entry.key).collect())
}

pub(crate) fn cell_delete_prefix_only_under_prefix(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::DeletePrefixOverruns]);
    let (_directory, mut server, mut client) = start_delete_prefix_cell(context, &switches)?;

    // 目标前缀 0x1C 底下 5 把键；前后各有别的前缀的键（比它小的 0x1B、比它大的 0x1D 与 ASCII 族名起头的），三张表的名义各写几把
    let targets: Vec<Vec<u8>> = (0..5_u8)
        .map(|index| tagged_key(DELETE_PREFIX_TARGET_TAG, index, 20 + usize::from(index)))
        .collect();
    let mut neighbours: Vec<Vec<u8>> = (0..2_u8)
        .map(|index| tagged_key(DELETE_PREFIX_TARGET_TAG - 1, index, 20))
        .collect();
    neighbours
        .extend((0..2_u8).map(|index| tagged_key(DELETE_PREFIX_TARGET_TAG + 1, 9 + index, 20)));
    neighbours.push(b"verdict-library/selftest-family/neighbour".to_vec());
    let mut entries: Vec<(Table, Vec<u8>, Vec<u8>)> = targets
        .iter()
        .map(|key| (Table::Conclusions, key.clone(), b"target".to_vec()))
        .collect();
    entries.extend(
        neighbours
            .iter()
            .map(|key| (Table::Recording, key.clone(), b"neighbour".to_vec())),
    );
    client.write(&entries)?;

    // 每帧至多 2 把：该是 2 把（没删完）、2 把（没删完）、1 把（前缀下没有更多了）；每帧的续扫起点是上一帧回的最后一把键
    let prefix = vec![DELETE_PREFIX_TARGET_TAG];
    let expected_frames = [
        (2_u32, false, &targets[1]),
        (2, false, &targets[3]),
        (1, true, &targets[4]),
    ];
    let mut after_key = Vec::new();
    for &(expected_keys, expected_exhausted, expected_last_key) in &expected_frames {
        let frame = client.delete_prefix_frame(&prefix, &after_key, 2, false)?;
        if frame.keys_in_frame != expected_keys
            || frame.is_exhausted != expected_exhausted
            || frame.last_key != *expected_last_key
        {
            return Err(format!(
                "从 {after_key:02x?} 之后起删前缀 {prefix:02x?}：回 {frame:?}，该是删 {expected_keys} 把、前缀下没有更多了={expected_exhausted}、\
                 最后一把键 {expected_last_key:02x?}（只删这个前缀底下的键，走出前缀就停）"
            ));
        }
        after_key = frame.last_key;
    }
    let emptied = client.delete_prefix_frame(&prefix, &[], 2, false)?;
    if emptied.keys_in_frame != 0 || !emptied.is_exhausted || !emptied.last_key.is_empty() {
        return Err(format!(
            "前缀 {prefix:02x?} 删空之后再删回 {emptied:?}，该是 0 把、没有更多了、最后一把键空"
        ));
    }

    // 删空前缀、落在记账那一段的前缀都拒，拒了之后库不动
    for (description, refused_prefix) in [
        ("空前缀", Vec::new()),
        ("落在服务记账那一段的前缀", vec![CLAIM_KEY_TAG]),
    ] {
        if let Ok(accepted) = client.delete_prefix_frame(&refused_prefix, &[], 2, false) {
            return Err(format!(
                "{description}被收了（回 {accepted:?}），该回状态 1：空前缀等于清空整个库，记账那一段留给服务"
            ));
        }
    }
    let remaining = keys_in_library(&mut client)?;
    let mut expected_remaining = neighbours.clone();
    expected_remaining.sort();
    if remaining != expected_remaining {
        return Err(format!(
            "删完前缀 {prefix:02x?} 之后库里剩 {} 把键，该是前后别的前缀的 {} 把一把不少（走出前缀的删除不许发生）：剩下 {remaining:02x?}",
            remaining.len(),
            expected_remaining.len()
        ));
    }
    drop(client);
    stop_server(&mut server)?;
    Ok(format!(
        "前缀 0x{DELETE_PREFIX_TARGET_TAG:02X} 底下 5 把键，每帧至多 2 把分三帧删完（2、2、1，最后一帧说没有更多了，续扫起点取上一帧的最后一把键）；\
         前后别的前缀的 {} 把键（三张表的名义写的）一把不少；再删回 0 把；空前缀与记账那一段的前缀被拒、库不动",
        neighbours.len()
    ))
}

pub(crate) fn cell_delete_prefix_counts_before_deleting(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only_these(&[BreakSwitch::DeletePrefixDryRunDeletes]);
    let (_directory, mut server, mut client) = start_delete_prefix_cell(context, &switches)?;

    // 前缀 count/pages/ 底下 5 把键，前面排着一把别的键；前缀之后没有别的键
    let prefix = b"count/pages/".to_vec();
    let preceding_key = b"count/a".to_vec();
    let page_keys: Vec<Vec<u8>> = (0..5)
        .map(|index| format!("count/pages/{index}").into_bytes())
        .collect();
    let mut entries = vec![(
        Table::Conclusions,
        preceding_key.clone(),
        b"preceding".to_vec(),
    )];
    entries.extend(
        page_keys
            .iter()
            .map(|key| (Table::Conclusions, key.clone(), b"page".to_vec())),
    );
    client.write(&entries)?;
    let keys_before: Vec<Vec<u8>> = entries.iter().map(|(_, key, _)| key.clone()).collect();

    // 只数不删：每帧至多 2 把，数 2、2、1，最后一帧说没有更多了；数完库里一把键不少
    let expected_frames = [
        (2_u32, false, &page_keys[1]),
        (2, false, &page_keys[3]),
        (1, true, &page_keys[4]),
    ];
    let mut after_key = Vec::new();
    for &(expected_keys, expected_exhausted, expected_last_key) in &expected_frames {
        let frame = client.delete_prefix_frame(&prefix, &after_key, 2, true)?;
        if frame.keys_in_frame != expected_keys
            || frame.is_exhausted != expected_exhausted
            || frame.last_key != *expected_last_key
        {
            return Err(format!(
                "从 {after_key:02x?} 之后起只数不删前缀 {prefix:02x?}：回 {frame:?}，该是数到 {expected_keys} 把、前缀下没有更多了={expected_exhausted}、\
                 最后一把键 {expected_last_key:02x?}"
            ));
        }
        after_key = frame.last_key;
    }
    let keys_after_counting = keys_in_library(&mut client)?;
    if keys_after_counting != keys_before {
        return Err(format!(
            "只数不删之后库里剩 {} 把键，该一把不少的 {} 把：只数不删把键删了",
            keys_after_counting.len(),
            keys_before.len()
        ));
    }

    // 先数后删：数到的总数就是随后删掉的总数，删完再数是 0，前缀之外那一把键还在
    let counted = client.delete_prefix(&prefix, 2, true)?;
    let deleted = client.delete_prefix(&prefix, 2, false)?;
    let counted_again = client.delete_prefix(&prefix, 2, true)?;
    if counted != 5 || deleted != counted || counted_again != 0 {
        return Err(format!(
            "先数后删：数到 {counted} 把（该是 5）、删掉 {deleted} 把（该与数到的相同）、删完再数 {counted_again} 把（该是 0）"
        ));
    }
    let keys_after_deleting = keys_in_library(&mut client)?;
    if keys_after_deleting != [preceding_key] {
        return Err(format!(
            "删完之后库里剩 {keys_after_deleting:02x?}，该只剩前缀之外的 count/a"
        ));
    }
    drop(client);
    stop_server(&mut server)?;
    Ok(
        "只数不删：每帧至多 2 把分三帧数到 5 把、库里一把键不少；先数后删：数到 5、删掉 5、删完再数 0，前缀之外的键还在"
            .to_string(),
    )
}
