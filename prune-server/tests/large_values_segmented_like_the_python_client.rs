//! 大值分段（`prune_server::segmented_values`）经容错客户端在临时服务上的写、读、扫：每条用例起自己的一份 prune-server（这个工程编出的服务二进制，
//! socket、状态目录与库都在这条用例独占的临时目录里），用完停服务、删目录；不连真服务。与 Python 客户端自证的 large-value-round-trip、
//! segmented-read-incomplete-refused 两格逐条对应（`scripts/prune-server/prune_server_client.py` 文件头「自证的格」），两边互读那一条让 Python 客户端的命令行
//! read 读回 Rust 客户端写下的大值（两边互读：存下的字节两边一样，Python 才拼得回）。另有一条一段长的值攒成一批、读回合计越过帧长上限的成批点读
//! （容错客户端拆键再问，`prune_server::reliable_client` 文件头「回应越过帧长上限就拆键」）。

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use prune_server::break_switches::{BreakSwitches, BREAK_VARIABLE};
use prune_server::client::{RouteAddress, SimpleClient};
use prune_server::configuration::{create_directory_holding_a_socket, MAXIMUM_SOCKET_PATH_BYTES};
use prune_server::protocol::{Table, MAXIMUM_FRAME_BODY_BYTES};
use prune_server::reliable_client::{
    ClientFailure, ReliableClient, ReliableClientSettings, OTHER_CLIENT_FAILURE_EXIT_CODE,
};
use prune_server::segmented_values::{
    frame_ranges, segment_key, version_registry_key, ScannedEntry, SegmentedScanAssembler,
    SegmentedValueFailure, SEGMENTED_FRAME_ENTRY_LIMIT, SEGMENT_BYTES, SEGMENT_MANIFEST_MAGIC,
};
use prune_server::server::LISTENING_LINE_PREFIX;
use prune_server::sha256::sha256_hexadecimal;
use prune_server::spawned::{SpawnedProcess, StandardError};

/// 用例全写进结论那一张表（Python 客户端只用它）。
const TABLE: Table = Table::Conclusions;
/// 写的帧至多这么多字节（与崩溃判定库 `PRUNE_SERVER_WRITE_FRAME_BYTES` 同一个数，远低于帧长上限）。
const WRITE_FRAME_BYTES: usize = 32 << 20;
/// 成批写的帧里每条在键值之外另占的字节（表 1、键长 4、值长 4，`protocol` 文件头）。
const WRITE_BYTES_PER_ENTRY: usize = 9;
/// 临时服务的工作线程数（与自证起的服务同一个数：至少 2 个才乱序得起来）。
const SERVER_WORKER_THREADS: &str = "4";
/// 等服务打出就绪行的上限：同机重活压着时开库也慢，给足（只是上限，起得快就不等）。
const SERVER_READY_TIMEOUT: Duration = Duration::from_secs(120);
/// 前缀扫一帧至多回这么多条（扫的前缀下只有几十条，一帧扫全）。
const SCAN_ENTRY_LIMIT: u32 = 1024;
/// Python 客户端命令行 read 的族与种类（两边互读那一条；`prune_server_client.py` 文件头「键空间」）。
const PYTHON_FAMILY: &str = "apparatus-items";
const PYTHON_KIND: &str = "segmented-values-interoperation";
const PYTHON_RELATIVE_PATH: &str = "large-value";
/// Python 客户端在临时服务底下给键加的命名空间取库目录 sha256 的前几个字符（`prune_server_client.py` 的 `NAMESPACE_DIGEST_CHARACTERS`）。
const PYTHON_NAMESPACE_DIGEST_CHARACTERS: usize = 16;

/// 临时目录名的前缀与里面 socket 的文件名。
const TEMPORARY_DIRECTORY_NAME_PREFIX: &str = "prune-segmented";
const SOCKET_FILE_NAME: &str = "server.sock";

/// 一条用例独占的临时目录：放手时删。建法见 `create_directory_holding_a_socket`，里面的 socket 全路径不超 107 字节。
struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn create() -> TemporaryDirectory {
        TemporaryDirectory::create_under(std::env::var_os("TMPDIR"))
    }

    /// `temporary_directory_variable` 是 `TMPDIR` 的值。
    fn create_under(temporary_directory_variable: Option<OsString>) -> TemporaryDirectory {
        let path = create_directory_holding_a_socket(
            temporary_directory_variable,
            TEMPORARY_DIRECTORY_NAME_PREFIX,
            SOCKET_FILE_NAME,
        )
        .unwrap_or_else(|message| panic!("{message}"));
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

/// 一条用例起的临时服务。字段次序就是放手次序：先停服务（`SpawnedProcess` 的 Drop 照进程号 SIGKILL 再收尸），再删目录。
struct TemporaryServer {
    _process: SpawnedProcess,
    unix_socket: PathBuf,
    network_address: String,
    directory: TemporaryDirectory,
}

impl TemporaryServer {
    /// 起服务：这个工程的服务二进制带 `--selftest-instance`（自证起服务的同一种起法，不过平常跑的准入），socket、状态目录、库都在这条用例的临时目录里，
    /// TCP 走回环、端口由内核给；弄坏开关的变量不传给它。
    fn start() -> TemporaryServer {
        TemporaryServer::start_in(TemporaryDirectory::create())
    }

    fn start_in(directory: TemporaryDirectory) -> TemporaryServer {
        let mut command = Command::new(env!("CARGO_BIN_EXE_prune-server"));
        command
            .arg("serve")
            .arg("--selftest-instance")
            .arg("--socket")
            .arg(directory.path.join(SOCKET_FILE_NAME))
            .arg("--state-directory")
            .arg(&directory.path)
            .arg("--library")
            .arg(directory.path.join("library"))
            .arg("--worker-threads")
            .arg(SERVER_WORKER_THREADS)
            .arg("--tcp")
            .arg("127.0.0.1:0")
            .env_remove(BREAK_VARIABLE);
        let mut process = SpawnedProcess::spawn(command, StandardError::Capture)
            .unwrap_or_else(|refusal| panic!("起不了临时 prune-server：{refusal}"));
        let ready_line = process
            .wait_for_line_starting_with(LISTENING_LINE_PREFIX, SERVER_READY_TIMEOUT)
            .unwrap_or_else(|message| {
                panic!(
                    "临时 prune-server 没起来：{message}；标准错误：{}",
                    process.standard_error_text()
                )
            });
        let unix_socket = PathBuf::from(
            field_of_the_ready_line(&ready_line, "unix")
                .unwrap_or_else(|| panic!("就绪行没有 unix=：{ready_line}")),
        );
        let network_address = field_of_the_ready_line(&ready_line, "tcp")
            .unwrap_or_else(|| panic!("就绪行没有 tcp=：{ready_line}"));
        TemporaryServer {
            _process: process,
            unix_socket,
            network_address,
            directory,
        }
    }

    fn route(&self) -> RouteAddress {
        RouteAddress::UnixSocket(self.unix_socket.clone())
    }

    /// 连它的一个容错客户端（追加日志在这条用例的临时目录里）。
    fn reliable_client(&self) -> ReliableClient {
        ReliableClient::open(ReliableClientSettings::new(
            self.route(),
            self.directory.path.join("client.journal"),
        ))
        .unwrap_or_else(|failure| panic!("连不上临时 prune-server：{failure}"))
    }
}

/// 就绪行里 `<名字>=<值>` 那一段的值。
fn field_of_the_ready_line(line: &str, name: &str) -> Option<String> {
    line.split_whitespace()
        .find_map(|field| field.strip_prefix(name)?.strip_prefix('='))
        .map(str::to_string)
}

/// 一个 `length_in_bytes` 字节、每个位置上的字节随位置与 `seed` 变的值（同长换一个字节摘要就变）。
fn patterned_value(length_in_bytes: usize, seed: usize) -> Vec<u8> {
    (0..length_in_bytes)
        .map(|position| {
            u8::try_from((position.wrapping_mul(31) + seed) % 251).expect("取模 251 放得进 u8")
        })
        .collect()
}

/// 照崩溃判定库的写法写一批：大值先分段写下，清单与别的条目随后按帧写、等确认。
fn write_with_large_values_segmented(
    client: &mut ReliableClient,
    entries: Vec<(Table, Vec<u8>, Vec<u8>)>,
) {
    let entries_to_write_last = client
        .write_segments_of_large_values(entries, WRITE_FRAME_BYTES)
        .unwrap_or_else(|failure| panic!("大值分段写不成：{failure}"));
    for frame in frame_ranges(
        &entries_to_write_last,
        |(_table, key, value)| WRITE_BYTES_PER_ENTRY + key.len() + value.len(),
        WRITE_FRAME_BYTES,
        SEGMENTED_FRAME_ENTRY_LIMIT,
    ) {
        client
            .submit_write_entries(&entries_to_write_last[frame])
            .unwrap_or_else(|failure| panic!("写不成：{failure}"));
    }
    client
        .wait_all_acknowledged()
        .unwrap_or_else(|failure| panic!("等不到写确认：{failure}"));
}

/// 原样写几条（造坏的清单与段用），不经分段。
fn write_stored_entries(client: &mut ReliableClient, entries: &[(Table, Vec<u8>, Vec<u8>)]) {
    client
        .submit_write_entries(entries)
        .unwrap_or_else(|failure| panic!("写不成：{failure}"));
    client
        .wait_all_acknowledged()
        .unwrap_or_else(|failure| panic!("等不到写确认：{failure}"));
}

/// 原样点读一把键（清单、段、版本登记键照存着的交）。
fn stored_value(client: &mut ReliableClient, key: &[u8]) -> Option<Vec<u8>> {
    client
        .read(&[(TABLE, key.to_vec())])
        .unwrap_or_else(|failure| panic!("读不成：{failure}"))
        .pop()
        .expect("问一把键回一个值")
}

/// Python 客户端 `encode_manifest` 写出的清单：开头接 `{"length":…,"segments":[…],"sha256":"…"}`，键按字典序、不带空白。
fn manifest_written_by_the_python_client(value: &[u8], segment_lengths: &[usize]) -> Vec<u8> {
    let segment_length_texts: Vec<String> = segment_lengths.iter().map(usize::to_string).collect();
    let mut manifest = SEGMENT_MANIFEST_MAGIC.to_vec();
    manifest.extend_from_slice(
        format!(
            "{{\"length\":{},\"segments\":[{}],\"sha256\":\"{}\"}}",
            value.len(),
            segment_length_texts.join(","),
            sha256_hexadecimal(value)
        )
        .as_bytes(),
    );
    manifest
}

/// large-value-round-trip：一个长过帧长上限的值（崩溃点重放三项退 3 的那种单独一行）与一条小值写得进；逐字节读回，不在的读成不在；
/// 服务里原样是 Python 那种存法：原键是清单、这一版（整值 sha256）的五段依次是四整段与余下的一个字节、版本登记键是这一版。
/// 判别力（`crates/mutations.tsv`）：点读读回清单就照原样交、不拼（读回的是清单本身），这条红。
#[test]
fn large_value_round_trip_reads_back_every_byte_and_stores_the_segments_like_the_python_client() {
    let server = TemporaryServer::start();
    let mut client = server.reliable_client();
    let frame_limit = usize::try_from(MAXIMUM_FRAME_BODY_BYTES).expect("u32 放得进 usize");
    let large_value = patterned_value(frame_limit + 1, 3);
    let large_key = b"family/kind/large".to_vec();
    let small_key = b"family/kind/small".to_vec();
    write_with_large_values_segmented(
        &mut client,
        vec![
            (TABLE, large_key.clone(), large_value.clone()),
            (TABLE, small_key.clone(), b"small".to_vec()),
        ],
    );
    let read_back = client
        .read_assembling_segmented_values(&[
            (TABLE, large_key.clone()),
            (TABLE, small_key),
            (TABLE, b"family/kind/absent".to_vec()),
        ])
        .unwrap_or_else(|failure| panic!("读不回：{failure}"));
    assert!(
        read_back[0].as_deref() == Some(large_value.as_slice()),
        "大值逐字节读回（读回 {:?} 字节，写的是 {} 字节）",
        read_back[0].as_ref().map(Vec::len),
        large_value.len()
    );
    assert_eq!(
        read_back[1..],
        [Some(b"small".to_vec()), None],
        "小值照原样、不在的读成不在"
    );
    let segment_lengths = [
        SEGMENT_BYTES,
        SEGMENT_BYTES,
        SEGMENT_BYTES,
        SEGMENT_BYTES,
        1,
    ];
    assert_eq!(
        stored_value(&mut client, &large_key),
        Some(manifest_written_by_the_python_client(
            &large_value,
            &segment_lengths
        )),
        "原键里存的是 Python 那种清单"
    );
    let large_version = sha256_hexadecimal(&large_value);
    for (segment_index, segment_length) in segment_lengths.iter().enumerate() {
        assert_eq!(
            stored_value(
                &mut client,
                &segment_key(&large_key, Some(&large_version), segment_index)
            )
            .map(|piece| piece.len()),
            Some(*segment_length),
            "段号 {segment_index} 在服务里、长度照清单"
        );
    }
    assert_eq!(
        stored_value(&mut client, &version_registry_key(&large_key)),
        Some(large_version.into_bytes())
    );
}

/// large-value-round-trip 里改写那一段：分段存过三段的键改写成一个短值：读回短值；短值照原样写进原键，旧的那一版三段这一次不动
/// （写方死在写原键之前读的照旧是旧值），版本登记键照旧登记着它；再写一次短值时写前第 2 步删掉旧的那一版三段，登记键写成空值（一版都不剩）。
/// 判别力（`crates/mutations.tsv`）：写之前读回的版本登记键不用、当没登记（旧的那一版三段永远留着），这条红。
#[test]
fn large_value_rewritten_shorter_keeps_the_old_version_until_the_next_write_reclaims_it() {
    let server = TemporaryServer::start();
    let mut client = server.reliable_client();
    let key = b"family/kind/rewritten".to_vec();
    let large_value = patterned_value(2 * SEGMENT_BYTES + 3, 5);
    let large_version = sha256_hexadecimal(&large_value);
    write_with_large_values_segmented(&mut client, vec![(TABLE, key.clone(), large_value)]);
    write_with_large_values_segmented(&mut client, vec![(TABLE, key.clone(), b"short".to_vec())]);
    assert_eq!(
        client
            .read_assembling_segmented_values(&[(TABLE, key.clone())])
            .unwrap_or_else(|failure| panic!("读不回：{failure}")),
        vec![Some(b"short".to_vec())]
    );
    assert_eq!(
        stored_value(&mut client, &key),
        Some(b"short".to_vec()),
        "短值照原样写进原键"
    );
    for segment_index in 0..3 {
        assert!(
            stored_value(
                &mut client,
                &segment_key(&key, Some(&large_version), segment_index)
            )
            .is_some(),
            "旧的那一版段号 {segment_index} 这一次不动，留给下一次写时回收"
        );
    }
    assert_eq!(
        stored_value(&mut client, &version_registry_key(&key)),
        Some(large_version.clone().into_bytes()),
        "版本登记键照旧登记着旧的那一版"
    );
    write_with_large_values_segmented(&mut client, vec![(TABLE, key.clone(), b"shorter".to_vec())]);
    for segment_index in 0..3 {
        assert!(
            stored_value(
                &mut client,
                &segment_key(&key, Some(&large_version), segment_index)
            )
            .is_none(),
            "再写一次时旧的那一版段号 {segment_index} 删掉"
        );
    }
    assert_eq!(
        stored_value(&mut client, &version_registry_key(&key)),
        Some(Vec::new()),
        "一版都不剩的版本登记键是空值"
    );
}

/// 原样写一把分两段存的键：清单照整值 helloworld（hello 与 world 两段）写，段号 0 是 hello，world 那一段写成 `world_segment`（`None` 不写）。
fn write_a_hello_world_key_with_its_world_segment_stored_as(
    client: &mut ReliableClient,
    key: &[u8],
    world_segment: Option<&[u8]>,
) {
    let version = sha256_hexadecimal(b"helloworld");
    let mut entries = vec![
        (
            TABLE,
            key.to_vec(),
            manifest_written_by_the_python_client(b"helloworld", &[5, 5]),
        ),
        (
            TABLE,
            segment_key(key, Some(&version), 0),
            b"hello".to_vec(),
        ),
        (
            TABLE,
            version_registry_key(key),
            version.clone().into_bytes(),
        ),
    ];
    if let Some(world_segment) = world_segment {
        entries.push((
            TABLE,
            segment_key(key, Some(&version), 1),
            world_segment.to_vec(),
        ));
    }
    write_stored_entries(client, &entries);
}

/// segmented-read-incomplete-refused：清单点名的段缺一段，点读拼不回、照别的错交回（退 3），不读成「不在」、不交值。
/// 判别力（`crates/mutations.tsv`）：点读来的段里缺的当空段拼（缺段读成长度不对），这条红。
#[test]
fn segmented_read_with_a_missing_segment_is_refused_and_never_read_as_absent() {
    let server = TemporaryServer::start();
    let mut client = server.reliable_client();
    let key = b"family/kind/missing-segment".to_vec();
    write_a_hello_world_key_with_its_world_segment_stored_as(&mut client, &key, None);
    match client.read_assembling_segmented_values(&[(TABLE, key)]) {
        Err(
            failure @ ClientFailure::SegmentedValue(SegmentedValueFailure::SegmentMissing {
                segment_index: 1,
                ..
            }),
        ) => assert_eq!(failure.exit_code(), OTHER_CLIENT_FAILURE_EXIT_CODE),
        other => panic!("段号 1 不在服务里，要拒（缺段）；交回的是 {other:?}"),
    }
}

/// segmented-read-incomplete-refused：一段短了一个字节，点读拼不回、照别的错退 3（不当服务不可用的 75 去重试）。
/// 判别力（`crates/mutations.tsv`）：分段拼不回照服务不可用退 75，这条红。
#[test]
fn segmented_read_with_a_short_segment_is_refused_with_the_other_failure_exit_code() {
    let server = TemporaryServer::start();
    let mut client = server.reliable_client();
    let key = b"family/kind/short-segment".to_vec();
    write_a_hello_world_key_with_its_world_segment_stored_as(&mut client, &key, Some(b"worl"));
    match client.read_assembling_segmented_values(&[(TABLE, key)]) {
        Err(
            failure @ ClientFailure::SegmentedValue(SegmentedValueFailure::SegmentLengthDiffers {
                segment_index: 1,
                ..
            }),
        ) => assert_eq!(
            failure.exit_code(),
            OTHER_CLIENT_FAILURE_EXIT_CODE,
            "拼不回是别的错（退 3），不是服务不可用"
        ),
        other => panic!("段号 1 短了一个字节，要拒（长度不对）；交回的是 {other:?}"),
    }
}

/// segmented-read-incomplete-refused：一段同长换了字节（wordl），只有整值 sha256 认得出：点读拼不回、不交值。
/// 判别力（`crates/mutations.tsv`）：拼回的整值不现算摘要、拿清单里写的当它的摘要（核了等于没核），这条红。
#[test]
fn segmented_read_with_swapped_bytes_of_the_same_length_is_refused_by_the_digest() {
    let server = TemporaryServer::start();
    let mut client = server.reliable_client();
    let key = b"family/kind/swapped-bytes".to_vec();
    write_a_hello_world_key_with_its_world_segment_stored_as(&mut client, &key, Some(b"wordl"));
    match client.read_assembling_segmented_values(&[(TABLE, key)]) {
        Err(ClientFailure::SegmentedValue(SegmentedValueFailure::DigestDiffers { .. })) => {}
        other => panic!("段号 1 同长换了字节，要拒（sha256 对不上）；交回的是 {other:?}"),
    }
}

/// large-value-round-trip 的前缀扫：前缀下两条小值与排在最后的一把分段键（两段），另有一条前缀之外的；服务里原样扫回来带着段键与版本登记键，
/// 喂给拼接器之后交出去的只有三把原键，分段那一把交拼回的整值。
/// 判别力（`crates/mutations.tsv`）：扫完时把还攒着的那把分段键丢掉（排在最后的分段键交不出去），这条红。
#[test]
fn large_value_prefix_scan_hands_out_owner_keys_with_the_segmented_value_assembled() {
    let server = TemporaryServer::start();
    let mut client = server.reliable_client();
    let large_value = patterned_value(SEGMENT_BYTES + 5, 7);
    let prefix = b"family/scan/".to_vec();
    write_with_large_values_segmented(
        &mut client,
        vec![
            (TABLE, b"family/scan/a".to_vec(), b"apple".to_vec()),
            (TABLE, b"family/scan/b".to_vec(), b"banana".to_vec()),
            (TABLE, b"family/scan/c".to_vec(), large_value.clone()),
            (TABLE, b"family/scan-not/d".to_vec(), b"outside".to_vec()),
        ],
    );
    let mut scan_client = SimpleClient::connect(&server.route(), BreakSwitches::none())
        .unwrap_or_else(|refusal| panic!("连不上临时 prune-server：{refusal}"));
    let (is_truncated, stored_entries) = scan_client
        .scan(TABLE, &prefix, SCAN_ENTRY_LIMIT)
        .unwrap_or_else(|refusal| panic!("扫不成：{refusal}"));
    assert!(!is_truncated, "前缀下几条、一帧扫全");
    assert_eq!(
        stored_entries.len(),
        6,
        "服务里原样有两条小值、清单、两段与版本登记键"
    );
    let mut assembler = SegmentedScanAssembler::new();
    let mut scanned_entries = Vec::new();
    for stored_entry in stored_entries {
        scanned_entries.extend(
            assembler
                .accept_stored_entry(stored_entry.key, stored_entry.value)
                .unwrap_or_else(|failure| panic!("清单解不开：{failure}")),
        );
    }
    scanned_entries.extend(assembler.finish().map(ScannedEntry::Segmented));
    let mut handed_out = Vec::new();
    for scanned_entry in scanned_entries {
        match scanned_entry {
            ScannedEntry::Whole { key, value } => handed_out.push((key, value)),
            ScannedEntry::Segmented(being_assembled) => {
                let key_in_the_service = being_assembled.owner_key().to_vec();
                handed_out.push(
                    client
                        .assemble_segmented_value(TABLE, &key_in_the_service, being_assembled)
                        .unwrap_or_else(|failure| panic!("拼不回：{failure}")),
                );
            }
        }
    }
    assert_eq!(
        handed_out,
        vec![
            (b"family/scan/a".to_vec(), b"apple".to_vec()),
            (b"family/scan/b".to_vec(), b"banana".to_vec()),
            (b"family/scan/c".to_vec(), large_value),
        ],
        "只交三把原键，分段那一把拼回整值"
    );
}

/// 成批点读回应越过帧长上限：五把键各存一个正好一段长的值（不分段，单键回应装得下一帧），合起来读回会越过帧长上限（服务整帧拒这种帧，
/// 说明写「这一帧读回的值合计 … 字节，回应会超过帧长上限」），夹一条小值与一把不在的键；一次成批点读交回全部，次序与问的相同，逐字节对。
/// 判别力（`crates/mutations.tsv`）：容错客户端回应越过帧长上限时不拆键（整帧拒照原样交回），这条红。
#[test]
fn batched_point_read_whose_answer_exceeds_the_frame_limit_reads_back_every_value_in_order() {
    let server = TemporaryServer::start();
    let mut client = server.reliable_client();
    let frame_limit = usize::try_from(MAXIMUM_FRAME_BODY_BYTES).expect("u32 放得进 usize");
    let segment_long_keys: Vec<Vec<u8>> = (0..5)
        .map(|key_index| format!("family/batched/segment-long-{key_index}").into_bytes())
        .collect();
    let segment_long_values: Vec<Vec<u8>> = (0..segment_long_keys.len())
        .map(|key_index| patterned_value(SEGMENT_BYTES, 13 + key_index))
        .collect();
    let answer_bytes: usize = segment_long_values.iter().map(Vec::len).sum();
    assert!(
        answer_bytes > frame_limit,
        "这一批读回的值合计 {answer_bytes} 字节，要越过帧长上限 {frame_limit} 才造得出服务整帧拒的现场"
    );
    let small_key = b"family/batched/small".to_vec();
    let mut entries: Vec<(Table, Vec<u8>, Vec<u8>)> = segment_long_keys
        .iter()
        .zip(&segment_long_values)
        .map(|(key, value)| (TABLE, key.clone(), value.clone()))
        .collect();
    entries.push((TABLE, small_key.clone(), b"small".to_vec()));
    write_with_large_values_segmented(&mut client, entries);
    for key in &segment_long_keys {
        assert!(
            stored_value(&mut client, &version_registry_key(key)).is_none(),
            "正好一段长的值不分段（没有版本登记键）：单键回应装得下一帧"
        );
    }
    let keys_to_read: Vec<(Table, Vec<u8>)> = vec![
        (TABLE, segment_long_keys[0].clone()),
        (TABLE, segment_long_keys[1].clone()),
        (TABLE, small_key),
        (TABLE, segment_long_keys[2].clone()),
        (TABLE, b"family/batched/absent".to_vec()),
        (TABLE, segment_long_keys[3].clone()),
        (TABLE, segment_long_keys[4].clone()),
    ];
    let read_back = client
        .read_assembling_segmented_values(&keys_to_read)
        .unwrap_or_else(|failure| panic!("回应越过帧长上限的成批点读读不全：{failure}"));
    let expected: Vec<Option<&[u8]>> = vec![
        Some(segment_long_values[0].as_slice()),
        Some(segment_long_values[1].as_slice()),
        Some(b"small".as_slice()),
        Some(segment_long_values[2].as_slice()),
        None,
        Some(segment_long_values[3].as_slice()),
        Some(segment_long_values[4].as_slice()),
    ];
    assert_eq!(read_back.len(), expected.len(), "问几把键回几个值");
    for (((_table, key), read_value), expected_value) in
        keys_to_read.iter().zip(&read_back).zip(&expected)
    {
        assert!(
            read_value.as_deref() == *expected_value,
            "键 {:?}：读回 {:?} 字节，该是 {:?} 字节（逐字节相同）",
            String::from_utf8_lossy(key),
            read_value.as_ref().map(Vec::len),
            expected_value.map(<[u8]>::len)
        );
    }
}

/// 两边互读：Rust 客户端分段写下一个三段长的值，Python 客户端的命令行 read（`prune_server_client.py` 文件头「命令行」）在同一份临时服务上
/// 读回、拼接、核 sha256，逐字节相同、退 0。Python 一侧照它的临时服务底下的起法连：`PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET` 与
/// `SINGLEFS_PRUNE_SERVER_SOCKET` 指这份 socket、`SINGLEFS_PRUNE_SERVER_TCP_ADDRESS` 指它的回环端口；键照 Python 的 `LibraryStore` 拼
/// （族、「@」接库目录 sha256 的前 16 个字符、种类、相对路径）。要本机有 `python3` 与仓里那份 Python 客户端，不在就红、不跳过。
/// 判别力（`crates/mutations.tsv`）：段号不补零（段键与 Python 的六位段号对不上，Rust 自己写自己读照样拼得回），这条红。
#[test]
fn python_client_reads_back_a_large_value_the_rust_client_wrote() {
    let server = TemporaryServer::start();
    let mut client = server.reliable_client();
    let library_directory = server.directory.path.join("python-library");
    std::fs::create_dir(&library_directory)
        .unwrap_or_else(|error| panic!("建不了库目录 {}：{error}", library_directory.display()));
    let canonical_library_directory = std::fs::canonicalize(&library_directory)
        .unwrap_or_else(|error| panic!("取不到库目录的真路径：{error}"));
    let namespace_digest = sha256_hexadecimal(
        canonical_library_directory
            .to_str()
            .expect("临时目录的路径是 UTF-8")
            .as_bytes(),
    );
    let key = format!(
        "{PYTHON_FAMILY}@{}/{PYTHON_KIND}/{PYTHON_RELATIVE_PATH}",
        &namespace_digest[..PYTHON_NAMESPACE_DIGEST_CHARACTERS]
    );
    let large_value = patterned_value(2 * SEGMENT_BYTES + 7, 11);
    write_with_large_values_segmented(
        &mut client,
        vec![(TABLE, key.into_bytes(), large_value.clone())],
    );
    let python_client = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("scripts")
        .join("prune-server")
        .join("prune_server_client.py");
    assert!(
        python_client.is_file(),
        "两边互读要仓里那份 Python 客户端：{} 不在",
        python_client.display()
    );
    let output = Command::new("python3")
        .arg(python_client)
        .arg("read")
        .arg(PYTHON_FAMILY)
        .arg(PYTHON_KIND)
        .arg(library_directory)
        .arg(PYTHON_RELATIVE_PATH)
        .env("PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET", &server.unix_socket)
        .env("SINGLEFS_PRUNE_SERVER_SOCKET", &server.unix_socket)
        .env("SINGLEFS_PRUNE_SERVER_TCP_ADDRESS", &server.network_address)
        .env_remove("PRUNE_SERVER_CLIENT_BREAK")
        .output()
        .unwrap_or_else(|error| panic!("起不了 python3：{error}"));
    assert!(
        output.status.success(),
        "Python 客户端读 Rust 写下的大值要退 0；退出 {:?}，标准错误：{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout == large_value,
        "Python 读回的 {} 字节要与 Rust 写的 {} 字节逐字节相同",
        output.stdout.len(),
        large_value.len()
    );
}

/// `TMPDIR` 比 socket 路径上限还长时，临时服务照样起得来、读写得通：目录退到短根底下，socket 全路径不超上限。
/// 判别力：用例的临时目录不经 `create_directory_holding_a_socket`、一律建在 `TMPDIR` 底下时，这条红（服务报 socket 路径超上限、不打就绪行）。
#[test]
fn server_starts_and_serves_when_the_temporary_root_is_longer_than_a_socket_path_may_be() {
    let long_root_parent = TemporaryDirectory::create();
    let long_root = long_root_parent
        .path
        .join("r".repeat(MAXIMUM_SOCKET_PATH_BYTES + 20));
    std::fs::create_dir_all(&long_root)
        .unwrap_or_else(|error| panic!("建不了长临时根 {}：{error}", long_root.display()));

    let server = TemporaryServer::start_in(TemporaryDirectory::create_under(Some(
        long_root.clone().into_os_string(),
    )));

    assert!(
        !server.directory.path.starts_with(&long_root),
        "临时根 {} 放不下 socket 全路径，目录该改建在短根底下，实际是 {}",
        long_root.display(),
        server.directory.path.display()
    );
    let mut client = server.reliable_client();
    let key = b"family/kind/long-temporary-root".to_vec();
    write_with_large_values_segmented(&mut client, vec![(TABLE, key.clone(), b"short".to_vec())]);
    assert_eq!(
        client
            .read_assembling_segmented_values(&[(TABLE, key)])
            .unwrap_or_else(|failure| panic!("读不回：{failure}")),
        vec![Some(b"short".to_vec())]
    );
}
