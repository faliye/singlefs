//! 登记的临时判定库（`src/registered_libraries.rs`）与取活循环（`src/serving_from_the_library.rs` 的 `serve_from_the_libraries`），
//! 对着测试进程里起的两三份临时判定库服务：一份当「真服务」（待判池的真库，登记认领在它里面），另几份当判段登记的临时服务
//! （各带一段键空间前缀）。不上卡：待判行照验证包的写法读写（`VerdictStore` 的表 `1D`），验证包那一侧另外三样换成替身
//! （结论字原样存一行，键前面接同一段前缀），卡与判器换成替身。
//! 每条的弄坏开关（`GPU_VERIFICATION_SERVER_BREAK=<项>`，表在 `src/break_switches.rs` 的 `BreakSwitch::red_test`）打开时那一条必须红。

mod common;

use std::collections::BTreeMap;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use common::{
    conclusion_version_of_the_crash_image_judgement, pending_block_name, pending_names,
    pending_row, put_pending_rows, stand_in_facts_bits, stand_in_verdicts_in,
    StandInVerificationPackageSide, TemporaryLibraryHandle, TemporaryLibraryService,
};
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_lease::{
    claim_a_card_lease_key_as, release_card_lease_keys_as, CardLeaseKeyAnswer, CardLeaseKeys,
    PROBE_LEASE_MILLISECONDS,
};
use gpu_verification_server::card_memory_ledger::{CardIndex, CardOfTheService};
use gpu_verification_server::card_worker::{JudgedOrdinals, JudgesOnACard};
use gpu_verification_server::cards_of_the_service::CardsOfTheService;
use gpu_verification_server::facts_verifier_family::FactsVerifierMaterial;
use gpu_verification_server::library_pending_pool::{
    claim_a_pending_block, ClaimAnswer, LibraryPendingPool, PendingBlockName,
};
use gpu_verification_server::pending_pool::StateVerdict;
use gpu_verification_server::registered_libraries::{
    OpenedRegisteredLibrary, OpeningOfARegisteredLibrary, RegisteredLibrariesSettings,
    REGISTERED_LIBRARY_RECONNECT, TEMPORARY_LIBRARY_DROPPED_LINE_PREFIX,
    TEMPORARY_LIBRARY_OPENED_LINE_PREFIX,
};
use gpu_verification_server::service_configuration::ServiceConfiguration;
use gpu_verification_server::service_process::{serving_line, ServiceProcessIdentity};
use gpu_verification_server::serving_from_the_library::{
    serve_from_the_libraries, ServingFailure, ServingSummary, BLOCK_JUDGED_LINE_PREFIX,
};
use gpu_verification_server::stream_identity::{StreamFacts, StreamKey};
use prune_server::client::RouteAddress;
use prune_server::protocol::{ClaimOutcome, Table};
use prune_server::reliable_client::{ReliableClient, ReliableClientSettings};
use singlefs_verification::crash_amplification::CardThatJudgedAPendingBlock;
use singlefs_verification::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
use singlefs_verification::crash_judge_gpu::VERDICT_WORDS;
use singlefs_verification::crash_library::{
    TemporaryLibraryRegistration, TemporaryServiceOfTheLibrary,
};
use singlefs_verification::verdict_store::{
    ForcedRejudging, GpuPendingRow, LibraryKeySpace, DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE,
};

type StandInSide = StandInVerificationPackageSide<StandInMaterials>;

/// 登记里「这一台」那一段：测试只当键里的一段文字，不经多机配置判法。
const THIS_HOST: &str = "192.0.2.10";
/// 取活循环那一半给待判池的借卡这一台（同上，文档段的地址）。
const SERVING_LOOP_HOST: &str = "192.0.2.7";
/// 认领与登记的租期：用例里不等它过期。
const LONG_LEASE_MILLISECONDS: u32 = 60_000;
/// 「登记没续」那一条登记的租期：等它过期。
const SHORT_REGISTRATION_LEASE_MILLISECONDS: u32 = 1_500;
/// 登记租期过了之后再等多久：取活循环至少又列过几回登记。
const REGISTRATION_LAPSE_SPAN: Duration = Duration::from_millis(2_300);
/// 取活循环的扫库间隔。
const SCAN_INTERVAL: Duration = Duration::from_millis(20);
/// 等取活循环做完一步的上限（替身判器一块几毫秒，扫库间隔 20 毫秒；10 秒是宽的）。
const STEP_WAIT: Duration = Duration::from_secs(10);
const STEP_POLL: Duration = Duration::from_millis(20);
/// 核「没认领、没判」时看多久：二十几个扫库间隔。
const HOLD_SPAN: Duration = Duration::from_millis(500);
/// 每条流的态数。
const STATES_OF_A_STREAM: u64 = 8;
/// 借的一方的持有者身份与旁观者的持有者身份。
const BORROWER_IDENTITY: &[u8] = b"registered-libraries-test-borrower";
/// 登记的临时库的键空间前缀：头一个字节是一次性库名字空间的头一个字节。
const PREFIX_OF_THE_TEMPORARY_LIBRARY: [u8; 4] =
    [DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE, b't', b'm', b'p'];
const PREFIX_OF_THE_OTHER_TEMPORARY_LIBRARY: [u8; 4] =
    [DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE, b'o', b't', b'h'];
/// 头一个字节不是一次性库名字空间的头一个字节的前缀。
const FOREIGN_PREFIX: [u8; 3] = [0x01, b'f', b'o'];

/// 替身判器装一条流要的：这条流的标记。
struct StandInMaterials {
    tag: u32,
}

/// 替身判出的结论：流的标记、序号的低 32 位，其余字是 0（不红）。
fn stand_in_verdict(tag: u32, ordinal: u64) -> StateVerdict {
    let mut verdict = [0u32; VERDICT_WORDS];
    verdict[0] = tag;
    verdict[1] = u32::try_from(ordinal & 0xffff_ffff).expect("低 32 位");
    verdict
}

fn stand_in_verdicts_of(tag: u32, ordinals: Range<u64>) -> Vec<StateVerdict> {
    ordinals
        .map(|ordinal| stand_in_verdict(tag, ordinal))
        .collect()
}

/// 替身判器判之前等的闸：开着才判。数一数已经开判的块。
#[derive(Default)]
struct JudgeGate {
    is_open: Mutex<bool>,
    opened: Condvar,
    blocks_started: AtomicU64,
}

impl JudgeGate {
    fn opened_at_start() -> Arc<Self> {
        let gate = Arc::new(Self::default());
        gate.open();
        gate
    }

    fn open(&self) {
        *self.is_open.lock().expect("没有线程拿着锁 panic") = true;
        self.opened.notify_all();
    }

    fn wait_until_open(&self) {
        let mut is_open = self.is_open.lock().expect("没有线程拿着锁 panic");
        while !*is_open {
            is_open = self.opened.wait(is_open).expect("没有线程拿着锁 panic");
        }
    }
}

/// 替身卡：一张，判器在闸上等。
struct StandInCards {
    gate: Arc<JudgeGate>,
}

struct StandInJudges {
    loaded: BTreeMap<StreamKey, u32>,
    gate: Arc<JudgeGate>,
}

impl JudgesOnACard for StandInJudges {
    type StreamMaterials = StandInMaterials;

    fn load_the_judge(
        &mut self,
        stream: StreamKey,
        materials: &StandInMaterials,
        _states_per_dispatch: u64,
    ) {
        assert!(
            self.loaded.insert(stream, materials.tag).is_none(),
            "{stream:?} 装了两次"
        );
    }

    fn evict_the_judge(&mut self, stream: StreamKey) {
        assert!(
            self.loaded.remove(&stream).is_some(),
            "要换出的 {stream:?} 不在卡上"
        );
    }

    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> JudgedOrdinals {
        let tag = *self
            .loaded
            .get(&stream)
            .unwrap_or_else(|| panic!("要判的 {stream:?} 不在卡上"));
        self.gate.blocks_started.fetch_add(1, Ordering::SeqCst);
        self.gate.wait_until_open();
        JudgedOrdinals {
            verdicts: stand_in_verdicts_of(tag, ordinals),
            dispatches: 1,
        }
    }

    fn verify_the_facts(
        &mut self,
        _material: &dyn FactsVerifierMaterial,
        blocks: &[Range<u64>],
    ) -> Vec<Vec<u32>> {
        blocks
            .iter()
            .map(|ordinals| stand_in_facts_bits(ordinals.clone()))
            .collect()
    }
}

impl CardsOfTheService for StandInCards {
    type Materials = StandInMaterials;
    type Judges<'cards> = StandInJudges;

    fn cards_of_the_service(&self) -> Vec<CardOfTheService> {
        vec![CardOfTheService {
            card: CardIndex(0),
            service_quota_mebibytes: MEBIBYTES_LEFT_FOR_THE_DRIVER + 4096,
        }]
    }

    fn judges_on(&self, _card: CardIndex) -> StandInJudges {
        StandInJudges {
            loaded: BTreeMap::new(),
            gate: Arc::clone(&self.gate),
        }
    }

    fn card_that_judged(&self, _card: CardIndex) -> CardThatJudgedAPendingBlock {
        CardThatJudgedAPendingBlock {
            card_index: 0,
            card_name: "stand-in card".to_string(),
            card_driver_version: "stand-in driver".to_string(),
        }
    }
}

fn tables_digest_of(tag: u8) -> [u8; 32] {
    [tag; 32]
}

/// 替身验证包一侧，连 `handle` 那份服务里 `key_prefix` 那一段键空间，登记了 `tags` 那几条流。
fn stand_in_side_with(
    handle: &TemporaryLibraryHandle,
    key_prefix: Vec<u8>,
    tags: &[u8],
) -> StandInSide {
    let mut side = StandInVerificationPackageSide::on_key_space(handle, key_prefix);
    for tag in tags {
        side.register(
            tables_digest_of(*tag),
            StreamKey {
                tables_digest: tables_digest_of(*tag),
                kernel_sources_digest: [0; 32],
            },
            StreamFacts {
                table_mebibytes: 1,
                state_count: STATES_OF_A_STREAM,
            },
            Arc::new(StandInMaterials {
                tag: u32::from(*tag),
            }),
        );
    }
    side
}

/// 真库那一份待判池（全仓那一份，键原样），服务打的行另收进内存。
fn real_pool_on(real: &TemporaryLibraryService, tags: &[u8]) -> LibraryPendingPool<StandInSide> {
    let mut pool = LibraryPendingPool::new(
        stand_in_side_with(&real.handle(), Vec::new(), tags),
        real.open_client(),
        LONG_LEASE_MILLISECONDS,
        BreakSwitches::of_the_environment(),
    );
    pool.keep_the_service_lines();
    pool
}

/// 第 `tag` 条流上的一块：名字与一行不强制的待判行。
fn block_of(
    tag: u8,
    block_number: u64,
    ordinals: Range<u64>,
    pool_entry_number: u64,
) -> (PendingBlockName, GpuPendingRow) {
    (
        pending_block_name(tag, block_number),
        pending_row(
            tables_digest_of(tag),
            ordinals,
            conclusion_version_of_the_crash_image_judgement(),
            ForcedRejudging::NotForced,
            pool_entry_number,
        ),
    )
}

fn key_space_of(prefix: &[u8]) -> LibraryKeySpace {
    LibraryKeySpace::DisposableLibrary(prefix.to_vec())
}

/// 以验证包写登记的方式（`TemporaryLibraryRegistration` 的登记键、持有者身份）在真服务里登记一份临时库
/// （判段登记也是这样：一把认领键，不写数据行），交回登记键。
fn register_a_library_at(
    registrar: &mut ReliableClient,
    socket_path: &str,
    key_space_prefix: &[u8],
    label: &str,
    lease_milliseconds: u32,
) -> Vec<u8> {
    let registration = TemporaryLibraryRegistration {
        host: THIS_HOST.to_string(),
        registrant: format!("{}-1", std::process::id()),
        service: TemporaryServiceOfTheLibrary {
            socket_path: Some(PathBuf::from(socket_path)),
            network_address: None,
            key_space_prefix: key_space_prefix.to_vec(),
        },
        label: label.to_string(),
    };
    let key = registration.claim_key().expect("登记键编得出");
    let outcomes = registrar
        .claim_as(
            &registration.holder_identity(),
            &[(Table::Conclusions, key.clone())],
            lease_milliseconds,
        )
        .unwrap_or_else(|failure| panic!("登记认领不了：{failure}"));
    assert!(
        matches!(outcomes.as_slice(), [ClaimOutcome::Claimed]),
        "登记认领到了：{outcomes:?}"
    );
    key
}

fn register_a_temporary_library(
    registrar: &mut ReliableClient,
    temporary: &TemporaryLibraryService,
    key_space_prefix: &[u8],
    label: &str,
) -> Vec<u8> {
    register_a_library_at(
        registrar,
        &temporary.socket_path().to_string_lossy(),
        key_space_prefix,
        label,
        LONG_LEASE_MILLISECONDS,
    )
}

/// 取活循环对登记的设置要的几样，连同循环里替身用的。
struct LoopSetup {
    real_socket: PathBuf,
    /// 登记里可能指到的各份服务的连法。
    handles: Vec<TemporaryLibraryHandle>,
    /// 每份登记的库的替身验证包一侧都登记这几条流。
    tags: Vec<u8>,
    /// 标签是它的那一份登记的库的替身验证包一侧读待判行时按 `reads_fail` 失败。
    reads_fail_label: &'static str,
    reads_fail: Arc<AtomicBool>,
    previous_serving_line: Option<String>,
    gate: Arc<JudgeGate>,
}

impl LoopSetup {
    fn over(
        real: &TemporaryLibraryService,
        others: &[&TemporaryLibraryService],
        tags: &[u8],
    ) -> Self {
        let mut handles = vec![real.handle()];
        handles.extend(others.iter().map(|service| service.handle()));
        Self {
            real_socket: real.socket_path().to_path_buf(),
            handles,
            tags: tags.to_vec(),
            reads_fail_label: "",
            reads_fail: Arc::new(AtomicBool::new(false)),
            previous_serving_line: None,
            gate: JudgeGate::opened_at_start(),
        }
    }
}

/// 照登记开一份登记的库（测试里的开法：连登记里的那份服务、验证包一侧是替身）。
fn open_a_registered_library_for_the_test(
    opening: &OpeningOfARegisteredLibrary,
    setup_handles: &[TemporaryLibraryHandle],
    tags: &[u8],
    reads_fail_label: &str,
    reads_fail: &Arc<AtomicBool>,
) -> Result<OpenedRegisteredLibrary<StandInSide>, String> {
    let RouteAddress::UnixSocket(socket) = &opening.route_of_the_library else {
        return Err("测试里的登记都走 unix socket".to_string());
    };
    let handle = setup_handles
        .iter()
        .find(|handle| handle.socket_path() == socket.as_path())
        .ok_or_else(|| format!("没有 {} 那份服务", socket.display()))?;
    let mut claims_settings = ReliableClientSettings::new(
        opening.route_of_the_claims.clone(),
        handle.journal_path_for_a_new_client(),
    );
    claims_settings.reconnect = REGISTERED_LIBRARY_RECONNECT;
    let claims = ReliableClient::open(claims_settings).map_err(|failure| failure.to_string())?;
    let mut side = stand_in_side_with(
        handle,
        opening.registration.service.key_space_prefix.clone(),
        tags,
    );
    if opening.registration.label == reads_fail_label {
        side.reads_fail = Arc::clone(reads_fail);
    }
    Ok(OpenedRegisteredLibrary {
        pool: LibraryPendingPool::new(
            side,
            claims,
            LONG_LEASE_MILLISECONDS,
            BreakSwitches::of_the_environment(),
        ),
        journal_directory: None,
    })
}

/// 测试那一侧结束（含 panic）时叫取活循环停、开判器的闸（闸关着，循环收尾时等手上的块判完就等不到头）。
struct StopTheLoopOnDrop {
    stop: Arc<AtomicBool>,
    gate: Arc<JudgeGate>,
}

impl Drop for StopTheLoopOnDrop {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.gate.open();
    }
}

/// 在另一条线程里对着真库与登记的库跑取活循环，`scenario` 在这一条线程里演；演完叫循环停，交回循环的结局与 `scenario` 交回的。
fn run_the_loop_over_registered_libraries<Output>(
    pool: &mut LibraryPendingPool<StandInSide>,
    setup: LoopSetup,
    scenario: impl FnOnce() -> Output,
) -> (Result<ServingSummary, ServingFailure>, Output) {
    let configuration = ServiceConfiguration {
        oldest_pending_wait_limit: Duration::ZERO,
        library_scan_interval: SCAN_INTERVAL,
        claim_lease: Duration::from_millis(u64::from(LONG_LEASE_MILLISECONDS)),
    };
    let break_switches = BreakSwitches::of_the_environment();
    let stop = Arc::new(AtomicBool::new(false));
    let gate_for_the_test = Arc::clone(&setup.gate);
    std::thread::scope(|scope| {
        let stop_for_the_loop = Arc::clone(&stop);
        let configuration = &configuration;
        let break_switches = &break_switches;
        let serving = scope.spawn(move || {
            let LoopSetup {
                real_socket,
                handles,
                tags,
                reads_fail_label,
                reads_fail,
                previous_serving_line,
                gate,
            } = setup;
            let mut open_the_cards = || -> Result<StandInCards, String> {
                Ok(StandInCards {
                    gate: Arc::clone(&gate),
                })
            };
            let mut open_a_library = |opening: &OpeningOfARegisteredLibrary| {
                open_a_registered_library_for_the_test(
                    opening,
                    &handles,
                    &tags,
                    reads_fail_label,
                    &reads_fail,
                )
            };
            let registered_libraries = RegisteredLibrariesSettings {
                open_a_library: &mut open_a_library,
                this_host: THIS_HOST.to_string(),
                real_route: RouteAddress::UnixSocket(real_socket),
                previous_serving_line,
            };
            serve_from_the_libraries(
                configuration,
                break_switches,
                pool,
                Some(registered_libraries),
                &mut open_the_cards,
                &|| stop_for_the_loop.load(Ordering::SeqCst),
            )
        });
        let stop_on_drop = StopTheLoopOnDrop {
            stop: Arc::clone(&stop),
            gate: gate_for_the_test,
        };
        let output = scenario();
        drop(stop_on_drop);
        (serving.join().expect("取活循环没 panic"), output)
    })
}

fn summary_of(result: Result<ServingSummary, ServingFailure>) -> ServingSummary {
    result.unwrap_or_else(|failure| panic!("取活循环没跑下去：{failure:?}"))
}

/// 至多等 `STEP_WAIT`（至少问一次）到 `condition` 成立，到了没成立就 panic。
fn wait_until(what: &str, condition: &mut dyn FnMut() -> bool) {
    let deadline = Instant::now() + STEP_WAIT;
    loop {
        if condition() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "{} 秒里没等到：{what}",
            STEP_WAIT.as_secs()
        );
        std::thread::sleep(STEP_POLL);
    }
}

fn lines_starting_with<'lines>(lines: &'lines [String], prefix: &str) -> Vec<&'lines String> {
    lines
        .iter()
        .filter(|line| line.starts_with(prefix))
        .collect()
}

/// 以旁观者身份、1 毫秒的租期试认领一次：认领到就是没人认领着这一块（那 1 毫秒过了自己失效）。
fn block_is_claimed(client: &mut ReliableClient, name: &PendingBlockName) -> bool {
    !matches!(
        claim_a_pending_block(client, name, PROBE_LEASE_MILLISECONDS).expect("认领得了"),
        ClaimAnswer::Claimed
    )
}

/// 登记一份临时库之后，判段把块写进那一份里，服务取走、判完、结论写回那一份里它自己的那一段名字空间，
/// 待判行在那一份里删掉；真库那一份一行都没多（既没有结论、也没有待判行）。临时服务里不接前缀的那一段（临时服务的全仓那一份）同样一行没多。
/// 弄坏开关：conclusion-written-into-the-real-library（登记的库认领与写结论的客户端连真服务）。
#[test]
fn a_registered_temporary_library_is_judged_and_written_back_into_that_library_only() {
    let real = TemporaryLibraryService::start("registered-judged-real");
    let temporary = TemporaryLibraryService::start("registered-judged-temporary");
    let block = block_of(1, 0, 0..4, 1);
    let temporary_store = temporary.open_store_in(key_space_of(&PREFIX_OF_THE_TEMPORARY_LIBRARY));
    put_pending_rows(&temporary_store, std::slice::from_ref(&block));
    let mut registrar = real.open_client();
    register_a_temporary_library(
        &mut registrar,
        &temporary,
        &PREFIX_OF_THE_TEMPORARY_LIBRARY,
        "judged-and-written-back",
    );
    let mut real_pool = real_pool_on(&real, &[1]);
    let mut temporary_raw = temporary.open_client();
    let mut real_raw = real.open_client();
    let real_store = real.open_store();
    let (result, ()) = run_the_loop_over_registered_libraries(
        &mut real_pool,
        LoopSetup::over(&real, &[&temporary], &[1]),
        || {
            wait_until(
                "登记的库里那一块判完写回、待判行删掉",
                &mut || !pending_names(&temporary_store).contains(&block.0),
            );
        },
    );
    let summary = summary_of(result);
    assert_eq!(
        stand_in_verdicts_in(
            &mut temporary_raw,
            &PREFIX_OF_THE_TEMPORARY_LIBRARY,
            &block.1
        ),
        Some(stand_in_verdicts_of(1, 0..4)),
        "结论写在登记的那一份里它自己的那一段名字空间"
    );
    assert_eq!(
        stand_in_verdicts_in(&mut temporary_raw, &[], &block.1),
        None,
        "临时服务里不接前缀的那一段没有这一块的结论"
    );
    assert_eq!(
        stand_in_verdicts_in(&mut real_raw, &PREFIX_OF_THE_TEMPORARY_LIBRARY, &block.1),
        None,
        "真服务里那一段名字空间没有这一块的结论"
    );
    assert_eq!(
        stand_in_verdicts_in(&mut real_raw, &[], &block.1),
        None,
        "真库里没有这一块的结论"
    );
    assert!(
        pending_names(&real_store).is_empty(),
        "真库里没有多出待判行"
    );
    assert_eq!(
        (
            summary.registered_libraries_opened,
            summary.blocks_written_back
        ),
        (1, 1)
    );
    assert_eq!(
        lines_starting_with(
            real_pool.kept_service_lines(),
            TEMPORARY_LIBRARY_OPENED_LINE_PREFIX
        )
        .len(),
        1,
        "打开这一份登记的库打一行"
    );
}

/// 两份库里有同名的块（同一条流的同一个块号）：入池时记着它在哪一份库，判完各写回自己的那一份，不串。
/// 弄坏开关：judged-block-matched-to-its-library-by-name（判完的块照块名找库，头一份认领着这个名字的）。
#[test]
fn blocks_with_the_same_name_in_two_libraries_are_each_written_back_into_their_own_library() {
    let real = TemporaryLibraryService::start("registered-same-name-real");
    let temporary = TemporaryLibraryService::start("registered-same-name-temporary");
    let real_block = block_of(7, 0, 0..4, 20);
    let registered_block = block_of(7, 0, 4..8, 10);
    assert_eq!(real_block.0, registered_block.0, "两块同名");
    let real_store = real.open_store();
    put_pending_rows(&real_store, std::slice::from_ref(&real_block));
    let temporary_store = temporary.open_store_in(key_space_of(&PREFIX_OF_THE_TEMPORARY_LIBRARY));
    put_pending_rows(&temporary_store, std::slice::from_ref(&registered_block));
    let mut registrar = real.open_client();
    register_a_temporary_library(
        &mut registrar,
        &temporary,
        &PREFIX_OF_THE_TEMPORARY_LIBRARY,
        "same-name",
    );
    let mut real_pool = real_pool_on(&real, &[7]);
    let mut temporary_raw = temporary.open_client();
    let mut real_raw = real.open_client();
    let (result, ()) = run_the_loop_over_registered_libraries(
        &mut real_pool,
        LoopSetup::over(&real, &[&temporary], &[7]),
        || {
            wait_until("两份库里的同名块都判完写回", &mut || {
                pending_names(&real_store).is_empty() && pending_names(&temporary_store).is_empty()
            });
        },
    );
    let summary = summary_of(result);
    assert_eq!(
        stand_in_verdicts_in(&mut real_raw, &[], &real_block.1),
        Some(stand_in_verdicts_of(7, 0..4)),
        "真库里的块写回真库，结论是它自己的态"
    );
    assert_eq!(
        stand_in_verdicts_in(
            &mut temporary_raw,
            &PREFIX_OF_THE_TEMPORARY_LIBRARY,
            &registered_block.1
        ),
        Some(stand_in_verdicts_of(7, 4..8)),
        "登记的库里的块写回登记的库，结论是它自己的态"
    );
    assert_eq!(
        stand_in_verdicts_in(&mut real_raw, &[], &registered_block.1),
        None,
        "登记的库里那一块的结论没写进真库"
    );
    assert_eq!(
        stand_in_verdicts_in(
            &mut temporary_raw,
            &PREFIX_OF_THE_TEMPORARY_LIBRARY,
            &real_block.1
        ),
        None,
        "真库里那一块的结论没写进登记的库"
    );
    assert_eq!(summary.blocks_written_back, 2);
}

/// 登记的租期过了（判段没续、也没放就没了）：下一圈丢掉那一份库；在判的块判完不写回（待判行留在那一份里），打一行丢掉的、记一笔。
/// 一个场景按次序几步：块入了池、登记过期、判完。
/// 弄坏开关：registration-lapse-ignored（登记没了也不丢）。
#[test]
fn a_registration_whose_lease_lapsed_drops_its_library_and_discards_its_blocks() {
    let real = TemporaryLibraryService::start("registered-lapse-real");
    let temporary = TemporaryLibraryService::start("registered-lapse-temporary");
    let block = block_of(2, 0, 0..4, 1);
    let temporary_store = temporary.open_store_in(key_space_of(&PREFIX_OF_THE_TEMPORARY_LIBRARY));
    put_pending_rows(&temporary_store, std::slice::from_ref(&block));
    let mut registrar = real.open_client();
    register_a_library_at(
        &mut registrar,
        &temporary.socket_path().to_string_lossy(),
        &PREFIX_OF_THE_TEMPORARY_LIBRARY,
        "lapsing",
        SHORT_REGISTRATION_LEASE_MILLISECONDS,
    );
    let mut real_pool = real_pool_on(&real, &[2]);
    let mut temporary_raw = temporary.open_client();
    let mut setup = LoopSetup::over(&real, &[&temporary], &[2]);
    setup.gate = Arc::new(JudgeGate::default());
    let gate = Arc::clone(&setup.gate);
    let (result, ()) = run_the_loop_over_registered_libraries(&mut real_pool, setup, || {
        wait_until("那一块入了池、在闸上等着判", &mut || {
            gate.blocks_started.load(Ordering::SeqCst) == 1
        });
        std::thread::sleep(REGISTRATION_LAPSE_SPAN);
        gate.open();
        std::thread::sleep(HOLD_SPAN);
    });
    let summary = summary_of(result);
    assert_eq!(
        stand_in_verdicts_in(
            &mut temporary_raw,
            &PREFIX_OF_THE_TEMPORARY_LIBRARY,
            &block.1
        ),
        None,
        "登记没了之后判完的块不写回"
    );
    assert!(
        pending_names(&temporary_store).contains(&block.0),
        "待判行留在那一份里"
    );
    assert_eq!(
        (
            summary.blocks_written_back,
            summary.blocks_discarded_with_their_library,
            summary.registered_libraries_dropped
        ),
        (0, 1, 1)
    );
    let dropped = lines_starting_with(
        real_pool.kept_service_lines(),
        TEMPORARY_LIBRARY_DROPPED_LINE_PREFIX,
    );
    assert_eq!(dropped.len(), 1, "丢掉那一份库打一行：{dropped:?}");
    assert!(
        dropped[0].contains("why=registration-lapsed"),
        "原因是登记没了：{}",
        dropped[0]
    );
}

/// 一份登记的库读写失败：只丢那一份、点名一次，服务不退出，真库里的活照判；失败的登记还在，下一次列到再试，丢掉那一行仍只打一次。
/// 一个场景按次序几步：登记的库判了一块、它读不了了、真库里来一块。
/// 弄坏开关：temporary-library-failure-ends-the-service（登记的库失败照真库那样交回）。
#[test]
fn an_unreachable_temporary_library_is_named_once_and_the_service_keeps_serving_the_real_library() {
    let real = TemporaryLibraryService::start("registered-failure-real");
    let temporary = TemporaryLibraryService::start("registered-failure-temporary");
    let registered_block = block_of(3, 0, 0..4, 1);
    let real_block = block_of(3, 1, 4..8, 2);
    let temporary_store = temporary.open_store_in(key_space_of(&PREFIX_OF_THE_TEMPORARY_LIBRARY));
    put_pending_rows(&temporary_store, std::slice::from_ref(&registered_block));
    let mut registrar = real.open_client();
    register_a_temporary_library(
        &mut registrar,
        &temporary,
        &PREFIX_OF_THE_TEMPORARY_LIBRARY,
        "failing",
    );
    let mut real_pool = real_pool_on(&real, &[3]);
    let mut real_raw = real.open_client();
    let real_store = real.open_store();
    let mut setup = LoopSetup::over(&real, &[&temporary], &[3]);
    setup.reads_fail_label = "failing";
    let reads_fail = Arc::clone(&setup.reads_fail);
    let (result, ()) = run_the_loop_over_registered_libraries(&mut real_pool, setup, || {
        wait_until("登记的库里那一块判完写回", &mut || {
            !pending_names(&temporary_store).contains(&registered_block.0)
        });
        reads_fail.store(true, Ordering::SeqCst);
        put_pending_rows(&real_store, std::slice::from_ref(&real_block));
        wait_until(
            "登记的库读不了之后真库里的块照判完写回",
            &mut || !pending_names(&real_store).contains(&real_block.0),
        );
    });
    let summary = summary_of(result);
    assert_eq!(
        stand_in_verdicts_in(&mut real_raw, &[], &real_block.1),
        Some(stand_in_verdicts_of(3, 4..8)),
        "真库里的块判完写回了真库"
    );
    assert!(
        summary.registered_libraries_dropped >= 1,
        "读不了的那一份丢了"
    );
    let failures: Vec<&String> = lines_starting_with(
        real_pool.kept_service_lines(),
        TEMPORARY_LIBRARY_DROPPED_LINE_PREFIX,
    )
    .into_iter()
    .filter(|line| line.contains("why=failed"))
    .collect();
    assert_eq!(
        failures.len(),
        1,
        "读写失败丢掉那一份只点名一次：{failures:?}"
    );
    assert!(
        failures[0].contains("label=failing"),
        "点名的是那一份：{}",
        failures[0]
    );
}

/// 登记指到真服务自己、或键空间前缀不空而头一个字节不是一次性库名字空间的头一个字节：不开，那两份库里的块不判；
/// 同时登记的合规的那一份照开、照判（这一条不是因为什么都不开才绿）。
/// 弄坏开关：real-service-registration-opened（两样都不核，照开）。
#[test]
fn a_registration_pointing_at_the_real_service_or_with_a_foreign_prefix_is_not_opened() {
    let real = TemporaryLibraryService::start("registered-guard-real");
    let temporary = TemporaryLibraryService::start("registered-guard-temporary");
    let block_at_the_real_service = block_of(4, 0, 0..4, 1);
    let block_with_a_foreign_prefix = block_of(4, 1, 4..8, 2);
    let block_that_is_opened = block_of(4, 2, 0..4, 3);
    let real_namespace_store = real.open_store_in(key_space_of(&PREFIX_OF_THE_TEMPORARY_LIBRARY));
    put_pending_rows(
        &real_namespace_store,
        std::slice::from_ref(&block_at_the_real_service),
    );
    let foreign_store = temporary.open_store_in(key_space_of(&FOREIGN_PREFIX));
    put_pending_rows(
        &foreign_store,
        std::slice::from_ref(&block_with_a_foreign_prefix),
    );
    let opened_store =
        temporary.open_store_in(key_space_of(&PREFIX_OF_THE_OTHER_TEMPORARY_LIBRARY));
    put_pending_rows(&opened_store, std::slice::from_ref(&block_that_is_opened));
    let mut registrar = real.open_client();
    register_a_library_at(
        &mut registrar,
        &real.socket_path().to_string_lossy(),
        &PREFIX_OF_THE_TEMPORARY_LIBRARY,
        "points-at-the-real-service",
        LONG_LEASE_MILLISECONDS,
    );
    register_a_temporary_library(
        &mut registrar,
        &temporary,
        &FOREIGN_PREFIX,
        "foreign-prefix",
    );
    register_a_temporary_library(
        &mut registrar,
        &temporary,
        &PREFIX_OF_THE_OTHER_TEMPORARY_LIBRARY,
        "well-formed",
    );
    let mut real_pool = real_pool_on(&real, &[4]);
    let mut real_raw = real.open_client();
    let mut temporary_raw = temporary.open_client();
    let (result, ()) = run_the_loop_over_registered_libraries(
        &mut real_pool,
        LoopSetup::over(&real, &[&temporary], &[4]),
        || {
            wait_until("合规的那一份里的块判完写回", &mut || {
                !pending_names(&opened_store).contains(&block_that_is_opened.0)
            });
            std::thread::sleep(HOLD_SPAN);
        },
    );
    let summary = summary_of(result);
    assert_eq!(
        stand_in_verdicts_in(
            &mut real_raw,
            &PREFIX_OF_THE_TEMPORARY_LIBRARY,
            &block_at_the_real_service.1
        ),
        None,
        "指到真服务的登记没开：那一段里的块没判"
    );
    assert_eq!(
        stand_in_verdicts_in(
            &mut temporary_raw,
            &FOREIGN_PREFIX,
            &block_with_a_foreign_prefix.1
        ),
        None,
        "前缀不合的登记没开：那一段里的块没判"
    );
    assert_eq!(summary.registered_libraries_opened, 1, "只开了合规的那一份");
}

/// 各份库的块合在一起按待判行的入池序号先来先判：真库的块入池序号是 10、30，登记的库的是 20、40，判完的次序就是 10、20、30、40，
/// 不是真库的先判完（真库一直有活时登记的库的活照样排得到）。同一条流的块在主循环里按入池先后派，所以判完的次序就是入池的次序。
/// 弄坏开关：real-library-claimed-first（不合起来排，真库的先）。
#[test]
fn candidates_of_every_library_are_claimed_in_pool_entry_order() {
    let real = TemporaryLibraryService::start("registered-order-real");
    let temporary = TemporaryLibraryService::start("registered-order-temporary");
    let real_store = real.open_store();
    put_pending_rows(
        &real_store,
        &[block_of(5, 0, 0..2, 10), block_of(5, 2, 4..6, 30)],
    );
    let temporary_store = temporary.open_store_in(key_space_of(&PREFIX_OF_THE_TEMPORARY_LIBRARY));
    put_pending_rows(
        &temporary_store,
        &[block_of(5, 1, 2..4, 20), block_of(5, 3, 6..8, 40)],
    );
    let mut registrar = real.open_client();
    register_a_temporary_library(
        &mut registrar,
        &temporary,
        &PREFIX_OF_THE_TEMPORARY_LIBRARY,
        "in-entry-order",
    );
    let mut real_pool = real_pool_on(&real, &[5]);
    let (result, ()) = run_the_loop_over_registered_libraries(
        &mut real_pool,
        LoopSetup::over(&real, &[&temporary], &[5]),
        || {
            wait_until("两份库里的四块都判完写回", &mut || {
                pending_names(&real_store).is_empty() && pending_names(&temporary_store).is_empty()
            });
        },
    );
    let summary = summary_of(result);
    assert_eq!(summary.blocks_written_back, 4);
    let judged_block_numbers: Vec<u64> =
        lines_starting_with(real_pool.kept_service_lines(), BLOCK_JUDGED_LINE_PREFIX)
            .into_iter()
            .map(|line| {
                line.split("block=")
                    .nth(1)
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|number| number.parse::<u64>().ok())
                    .unwrap_or_else(|| panic!("判完一块那一行里没有块号：{line}"))
            })
            .collect();
    assert_eq!(
        judged_block_numbers,
        [0, 1, 2, 3],
        "判完的次序是入池序号 10、20、30、40 那四块"
    );
}

/// 有人借卡（卡键在借的一方手里）：哪一份库都不扫、不认领，登记的库也不开；卡键放出来之后收回、开登记的库、判完写回。
/// 弄坏开关：temporary-libraries-scanned-while-lent（借出期间登记的库照扫、照认领）。
#[test]
fn no_library_is_scanned_while_the_cards_are_lent() {
    let real = TemporaryLibraryService::start("registered-lent-real");
    let temporary = TemporaryLibraryService::start("registered-lent-temporary");
    let keys = CardLeaseKeys::of_this_host(SERVING_LOOP_HOST);
    let block = block_of(6, 0, 0..4, 1);
    let temporary_store = temporary.open_store_in(key_space_of(&PREFIX_OF_THE_TEMPORARY_LIBRARY));
    put_pending_rows(&temporary_store, std::slice::from_ref(&block));
    let mut registrar = real.open_client();
    register_a_temporary_library(
        &mut registrar,
        &temporary,
        &PREFIX_OF_THE_TEMPORARY_LIBRARY,
        "while-lent",
    );
    let mut real_pool = real_pool_on(&real, &[6]);
    real_pool.lend_the_cards_of_this_host(keys.clone());
    let mut borrower = real.open_client();
    assert_eq!(
        claim_a_card_lease_key_as(
            &mut borrower,
            BORROWER_IDENTITY,
            keys.card_key(),
            LONG_LEASE_MILLISECONDS
        )
        .expect("认领得了卡键"),
        CardLeaseKeyAnswer::Claimed,
        "借的一方先持着卡键，服务起来时从借出起步"
    );
    let mut observer = temporary.open_client();
    let (result, ()) = run_the_loop_over_registered_libraries(
        &mut real_pool,
        LoopSetup::over(&real, &[&temporary], &[6]),
        || {
            std::thread::sleep(HOLD_SPAN);
            assert!(
                pending_names(&temporary_store).contains(&block.0)
                    && !block_is_claimed(&mut observer, &block.0),
                "借出期间登记的库里的块不认领"
            );
            release_card_lease_keys_as(&mut borrower, BORROWER_IDENTITY, &[keys.card_key()])
                .expect("放得了卡键");
            wait_until(
                "卡键放出来之后收回、登记的库里的块判完写回",
                &mut || !pending_names(&temporary_store).contains(&block.0),
            );
        },
    );
    let summary = summary_of(result);
    assert_eq!(summary.registered_libraries_opened, 1);
    assert_eq!(summary.blocks_written_back, 1);
}

/// 服务重起：上一个进程（「在跑」那一行记的进程号此刻在、起来的时刻对不上，就是它没了）留在登记的库里的认领，
/// 头一次打开那一份库时以它的持有者放掉，那一块随即认领得到、判完写回，不等租期。
/// 弄坏开关：previous-claims-kept-in-temporary-libraries（打开登记的库时不放）。
#[test]
fn a_restarted_service_releases_the_claims_its_previous_process_left_in_a_temporary_library() {
    let real = TemporaryLibraryService::start("registered-restart-real");
    let temporary = TemporaryLibraryService::start("registered-restart-temporary");
    let block = block_of(8, 0, 0..4, 1);
    let temporary_store = temporary.open_store_in(key_space_of(&PREFIX_OF_THE_TEMPORARY_LIBRARY));
    put_pending_rows(&temporary_store, std::slice::from_ref(&block));
    let mut previous_process_pool = LibraryPendingPool::new(
        stand_in_side_with(
            &temporary.handle(),
            PREFIX_OF_THE_TEMPORARY_LIBRARY.to_vec(),
            &[8],
        ),
        temporary.open_client(),
        LONG_LEASE_MILLISECONDS,
        BreakSwitches::none(),
    );
    let candidates = previous_process_pool.candidates().expect("扫得了库");
    assert_eq!(
        previous_process_pool
            .claim_the_candidates(candidates)
            .expect("认领得了")
            .taken
            .len(),
        1,
        "上一个进程认领了那一块、没写回就没了"
    );
    let this_process = ServiceProcessIdentity::of_this_process().expect("读得到这个进程的身份");
    let process_that_is_gone = ServiceProcessIdentity {
        process_number: this_process.process_number,
        start_ticks: this_process.start_ticks + 1,
        boot: this_process.boot,
    };
    let previous_serving_line =
        serving_line(&process_that_is_gone, 0, &previous_process_pool.holder());
    drop(previous_process_pool);
    let mut observer = temporary.open_client();
    assert!(
        block_is_claimed(&mut observer, &block.0),
        "上一个进程持着的认领此刻还在（租期没到）"
    );
    let mut registrar = real.open_client();
    register_a_temporary_library(
        &mut registrar,
        &temporary,
        &PREFIX_OF_THE_TEMPORARY_LIBRARY,
        "after-a-restart",
    );
    let mut real_pool = real_pool_on(&real, &[8]);
    let mut setup = LoopSetup::over(&real, &[&temporary], &[8]);
    setup.previous_serving_line = Some(previous_serving_line);
    let (result, ()) = run_the_loop_over_registered_libraries(&mut real_pool, setup, || {
        wait_until(
            "上一个进程的认领放掉之后那一块判完写回",
            &mut || !pending_names(&temporary_store).contains(&block.0),
        );
    });
    let summary = summary_of(result);
    assert_eq!(summary.blocks_written_back, 1);
    assert!(
        real_pool
            .kept_service_lines()
            .iter()
            .any(|line| line.starts_with(TEMPORARY_LIBRARY_OPENED_LINE_PREFIX)),
        "打开了登记的库"
    );
}
