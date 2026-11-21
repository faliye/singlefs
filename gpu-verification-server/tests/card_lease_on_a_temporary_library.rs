//! 借卡（`src/card_lease.rs` 文件头）对着测试进程里起的临时判定库服务（`prune_server::server::start_in_this_process`，不写真服务），不上卡。
//!
//! 取活循环那一半（`src/serving_from_the_library.rs` 文件头「借卡」）：待判行照验证包的写法读写，验证包另外三样与卡、判器换成替身
//! （判器在一道闸上等，测试开闸它才判完：「手上入了池的块」由它造）；借的一方由测试自己以一个持有者身份认领、放掉两把键来演。
//! 这一半直接给待判池借卡键（这一台写文档段的地址 [`SERVING_LOOP_HOST`]），不经多机配置判法。
//!
//! 借的一方那一半：起这个包编出来的二进制的 `lease take`、`lease return`，多机配置是用例目录里自己写的一份
//! （`SINGLEFS_MULTI_HOST_CONFIG`；这一台写 `127.0.0.1`，每一台的 lo 上都有，判法照真的网卡地址认它是这一台；第二台写文档段的 `192.0.2.1`，
//! 哪一台的网卡上都没有：两台跑判得一样），判定库是临时服务（经改连接地址的变量交，带临时服务的标记）。`lease take` 只借给借卡白名单里的
//! 可执行文件名（`src/card_lease.rs` 文件头「借卡白名单」）：借得到的那几条，借的进程是测试起的 `sleep`——拷进用例目录、改名成白名单里那个名字
//! （[`LISTED_EXECUTABLE_NAME`]）再起，放进它自己的进程组，用例结束时由守卫停掉、收尸；白名单拒的那几条，借的进程是这个测试进程自己
//! （cargo 测试二进制，文件名带哈希后缀）或改名成白名单里那个名字接上哈希后缀的 `sleep`。`lease take` 起的续租进程由守卫照它的借的一方
//! `lease return` 叫停（借的进程没了它自己放键、退出）。看两把键在谁手里，以一个旁观的持有者身份、1 毫秒的租期试认领一次；
//! 看这一台排着哪几张等号（`src/card_lease.rs` 文件头「先来后到」），以那个旁观的客户端按前缀列认领。先来后到那一条要让先等的那一个
//! `lease take` 停一段：对它自己的进程组发 SIGSTOP、SIGCONT（照 `PATH` 找 `kill`）。
//!
//! 每条的弄坏开关（`GPU_VERIFICATION_SERVER_BREAK=<项>`，表在 `src/break_switches.rs` 的 `BreakSwitch::red_test`）打开时那一条必须红；
//! 起二进制的那几条，开关经继承的环境交给它（它只在临时判定库服务底下认）。

mod common;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Read;
use std::ops::Range;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use common::{
    conclusion_version_of_the_crash_image_judgement, pending_block_name, pending_names,
    pending_row, put_pending_rows, StandInVerificationPackageSide, TemporaryLibraryService,
};
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_lease::{
    cards_lent_line, claim_a_card_lease_key_as, holder_text, release_card_lease_keys_as,
    CardLeaseBorrower, CardLeaseBorrowerWhitelist, CardLeaseKeyAnswer, CardLeaseKeys,
    BORROWER_OFF_THE_WHITELIST_EXIT, BORROWER_VARIABLE, CARDS_LENT_LINE_PREFIX,
    CARDS_TAKEN_BACK_LINE_PREFIX, CARD_LEASE_BORROWERS_PATH_IN_THE_REPOSITORY,
    CARD_LEASE_HELD_LINE_PREFIX, CARD_LEASE_QUEUED_LINE_PREFIX, CARD_LEASE_REFUSED_LINE_PREFIX,
    CARD_LEASE_REQUESTED_LINE_PREFIX, CARD_LEASE_RETURNING_LINE_PREFIX,
    CARD_LEASE_TAKEN_LINE_PREFIX, CARD_LEASE_WAITING_LINE_PREFIX, PROBE_LEASE_MILLISECONDS,
};
use gpu_verification_server::card_memory_ledger::{CardIndex, CardOfTheService};
use gpu_verification_server::card_worker::{JudgedOrdinals, JudgesOnACard};
use gpu_verification_server::cards_of_the_service::CardsOfTheService;
use gpu_verification_server::facts_verifier_family::FactsVerifierMaterial;
use gpu_verification_server::library_pending_pool::{
    claim_a_pending_block, ClaimAnswer, LibraryPendingPool, PendingBlockName,
};
use gpu_verification_server::pending_pool::StateVerdict;
use gpu_verification_server::service_configuration::{
    ServiceConfiguration, SERVICE_CONFIGURATION_TEXT,
};
use gpu_verification_server::service_process::ProcessLiveness;
use gpu_verification_server::serving_from_the_library::{
    serve_from_the_library, ServingSummary, CARD_PERIOD_LINE_PREFIX, CARD_SUMMARY_PERIOD,
};
use gpu_verification_server::stream_identity::{StreamFacts, StreamKey};
use prune_server::reliable_client::ReliableClient;
use singlefs_verification::crash_amplification::CardThatJudgedAPendingBlock;
use singlefs_verification::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
use singlefs_verification::crash_judge_gpu::VERDICT_WORDS;
use singlefs_verification::crash_library::{
    ExclusiveTemporaryDirectory, PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE,
    PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE,
    PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::verdict_store::{ForcedRejudging, GpuPendingRow};

const SERVER_BINARY: &str = env!("CARGO_BIN_EXE_gpu-verification-server");
/// 取活循环那一半给待判池的这一台（文档段的地址：不经多机配置判法，只当键里的那一段）。
const SERVING_LOOP_HOST: &str = "192.0.2.7";
/// 借的一方那一半的多机配置里的这一台（每一台的 lo 上都有）与第二台（文档段，哪一台的网卡上都没有）。
const THIS_HOST: &str = "127.0.0.1";
const PEER_HOST: &str = "192.0.2.1";
/// 用例写的多机配置里清场之后那一份显卡先后：这一台两项夹着第二台一项，借到的卡串只该有这一台那两项、照原来的先后。
const CARD_STRING_WITH_CARDS_OF_THIS_HOST: &str = "127.0.0.1:0:8G 192.0.2.1:0:16G 127.0.0.1:1:4G";
const CARDS_OF_THIS_HOST: &str = "127.0.0.1:0:8G 127.0.0.1:1:4G";
/// 只点名第二台的卡的那一份。
const CARD_STRING_WITHOUT_CARDS_OF_THIS_HOST: &str = "192.0.2.1:0:16G";
/// [`CARD_STRING_WITH_CARDS_OF_THIS_HOST`] 里第二台的那几项（`lease list` 那一台的卡串）。
const CARDS_OF_THE_PEER: &str = "192.0.2.1:0:16G";
/// 认领的租期：用例里不等它过期。
const LONG_LEASE_MILLISECONDS: u32 = 60_000;
/// 「借的一方没了」那一条借的一方认领两把键的租期：等它过期。
const VANISHING_BORROWER_LEASE_MILLISECONDS: u32 = 1_500;
/// 取活循环的扫库间隔（也是它探请求键的间隔）。
const SCAN_INTERVAL: Duration = Duration::from_millis(20);
/// 等取活循环做完一步的上限（替身判器一块几毫秒，扫库间隔 20 毫秒；10 秒是宽的）。
const STEP_WAIT: Duration = Duration::from_secs(10);
const STEP_POLL: Duration = Duration::from_millis(20);
/// 核「这一段里没认领、没放」时看多久：二十几个扫库间隔。
const HOLD_SPAN: Duration = Duration::from_millis(500);
/// 借的一方认领请求键之后等多久再往库里写块：几个扫库间隔，取活循环至少又探过一次请求键。
const REQUEST_SEEN_SPAN: Duration = Duration::from_millis(100);
/// 「借出期间每分钟卡账那一行照打」那一条借的一方认领两把键的租期：比那一条跑的时长长，跑完之前不过期。
const LENT_FOR_A_WHOLE_PERIOD_LEASE_MILLISECONDS: u32 = 120_000;
/// 每条流的态数。
const STATES_OF_A_STREAM: u64 = 8;
/// 旁观者的持有者身份：看两把键在谁手里时以它试认领。
const OBSERVER_IDENTITY: &[u8] = b"card-lease-test-observer";
/// 取活循环那一半里借的一方的持有者身份。
const BORROWER_IDENTITY_IN_THE_SERVING_LOOP: &[u8] = b"card-lease-test-borrower";
/// 等 `lease take` 借到、`lease return` 还完的上限：借的一方每 2 秒试一次，判法与连临时判定库各不到一秒，30 秒是宽的。
const LEASE_COMMAND_WAIT: Duration = Duration::from_secs(30);
const LEASE_COMMAND_POLL: Duration = Duration::from_millis(50);
/// 核 `lease take` 还在等时等多久：比借的一方再试的间隔（2 秒）长。
const TAKE_STILL_WAITING_SPAN: Duration = Duration::from_secs(5);
/// 借的进程不在、`lease return` 退 0 之后，两把键与续租进程至多这么久放掉、退出（`src/card_lease.rs` 文件头：续租进程一秒一拍）。
const RELEASE_LIMIT: Duration = Duration::from_secs(5);
const USAGE_EXIT: i32 = 2;
const RUN_CONDITION_EXIT: i32 = 78;
/// 「借出期间每分钟卡账那一行照打」那一条在一段卡账周期之外多跑多久：满一段那一行打出来之后再停。
const CARD_PERIOD_MARGIN: Duration = Duration::from_secs(2);
/// 借卡白名单里的那个可执行文件名（`gpu-verification-server/src/card-lease-borrowers.tsv` 此刻唯一的一项，`src/card_lease.rs` 的单测
/// `the_compiled_in_whitelist_names_only_the_e202_comparison_binary` 钉住表里恰好是它）。
const LISTED_EXECUTABLE_NAME: &str = "e202_gpu_cpu_gc_comparison";
/// 白名单里那个名字接上 cargo 测试二进制那种 `-<十六位哈希>` 后缀：逐字比对不上。
const LISTED_EXECUTABLE_NAME_WITH_A_CARGO_HASH_SUFFIX: &str =
    "e202_gpu_cpu_gc_comparison-0123456789abcdef";
/// 借的进程拷的那个长驻小程序（照 `PATH` 找）：改名之后照 `sleep 600` 跑。
const LONG_RUNNING_PROGRAM: &str = "sleep";
const LONG_RUNNING_PROGRAM_SECONDS: &str = "600";
/// 先来后到那一条里先等的那一个 `lease take` 停着的那一段：比借的一方再试的间隔（2 秒）的两倍长，不看次序的那一种在这一段里一定再试过。
const EARLIER_TAKE_PAUSED_SPAN: Duration = Duration::from_secs(5);
/// 先等的那一张等号排上之后隔多久再起后等的那一个：两张等号的开始等的时刻分得开。
const GAP_BETWEEN_THE_TWO_WAITS: Duration = Duration::from_millis(200);
/// 停、接着跑一个进程组用的程序（照 `PATH` 找）与两个信号的名字。
const SIGNAL_PROGRAM: &str = "kill";
const STOP_SIGNAL: &str = "STOP";
const CONTINUE_SIGNAL: &str = "CONT";
/// 拷完的可执行文件起不来（ETXTBSY）时再试几回、隔多久：测试进程里别的线程同一刻起子进程，fork 出来、还没 exec 的子进程照样开着
/// 拷的那一份的写端，它们 exec 之后就起得来。
const TEXT_FILE_BUSY_RETRIES: u32 = 100;
const TEXT_FILE_BUSY_PAUSE: Duration = Duration::from_millis(10);

/// 替身判器装一条流要的：这条流的标记。
struct StandInMaterials {
    tag: u32,
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

/// 替身卡：一张，判器在闸上等；放掉（丢掉）时记一笔。
struct GatedStandInCards {
    let_go: Arc<AtomicU64>,
    gate: Arc<JudgeGate>,
}

impl Drop for GatedStandInCards {
    fn drop(&mut self) {
        self.let_go.fetch_add(1, Ordering::SeqCst);
    }
}

struct GatedStandInJudges {
    loaded: BTreeMap<StreamKey, u32>,
    gate: Arc<JudgeGate>,
}

impl JudgesOnACard for GatedStandInJudges {
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

    /// 记一块开判，等闸开了再判：结论是流的标记与序号的低 32 位，其余字是 0（不红）。
    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> JudgedOrdinals {
        let tag = *self
            .loaded
            .get(&stream)
            .unwrap_or_else(|| panic!("要判的 {stream:?} 不在卡上"));
        self.gate.blocks_started.fetch_add(1, Ordering::SeqCst);
        self.gate.wait_until_open();
        JudgedOrdinals {
            verdicts: ordinals
                .map(|ordinal| {
                    let mut verdict: StateVerdict = [0u32; VERDICT_WORDS];
                    verdict[0] = tag;
                    verdict[1] = u32::try_from(ordinal & 0xffff_ffff).expect("低 32 位");
                    verdict
                })
                .collect(),
            dispatches: 1,
        }
    }

    fn verify_the_facts(
        &mut self,
        _material: &dyn FactsVerifierMaterial,
        blocks: &[Range<u64>],
    ) -> Vec<Vec<u32>> {
        panic!(
            "借卡的用例不写核对待判行，替身核对器不该被派到（{} 块）",
            blocks.len()
        )
    }
}

impl CardsOfTheService for GatedStandInCards {
    type Materials = StandInMaterials;
    type Judges<'cards> = GatedStandInJudges;

    fn cards_of_the_service(&self) -> Vec<CardOfTheService> {
        vec![CardOfTheService {
            card: CardIndex(0),
            service_quota_mebibytes: MEBIBYTES_LEFT_FOR_THE_DRIVER + 4096,
        }]
    }

    fn judges_on(&self, _card: CardIndex) -> GatedStandInJudges {
        GatedStandInJudges {
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

/// 第 `tag` 条流：判表摘要每个字节都是 `tag`。
fn tables_digest_of(tag: u8) -> [u8; 32] {
    [tag; 32]
}

/// 第 `tag` 条流上第 `block_number` 块：名字与一行不强制的待判行。
fn block_of(tag: u8, block_number: u64, ordinals: Range<u64>) -> (PendingBlockName, GpuPendingRow) {
    (
        pending_block_name(tag, block_number),
        pending_row(
            tables_digest_of(tag),
            ordinals,
            conclusion_version_of_the_crash_image_judgement(),
            ForcedRejudging::NotForced,
            block_number + 1,
        ),
    )
}

/// 借了这一台的卡的待判池：替身验证包一侧登记了第 `tag` 条流，服务打的行另收进内存。
fn lending_pool_on(
    library: &TemporaryLibraryService,
    tag: u8,
) -> LibraryPendingPool<StandInVerificationPackageSide<StandInMaterials>> {
    let mut side = StandInVerificationPackageSide::on(library);
    side.register(
        tables_digest_of(tag),
        StreamKey {
            tables_digest: tables_digest_of(tag),
            kernel_sources_digest: [0; 32],
        },
        StreamFacts {
            table_mebibytes: 1,
            state_count: STATES_OF_A_STREAM,
        },
        Arc::new(StandInMaterials {
            tag: u32::from(tag),
        }),
    );
    let mut pool = LibraryPendingPool::new(
        side,
        library.open_client(),
        LONG_LEASE_MILLISECONDS,
        BreakSwitches::of_the_environment(),
    );
    pool.keep_the_service_lines();
    pool.lend_the_cards_of_this_host(CardLeaseKeys::of_this_host(SERVING_LOOP_HOST));
    pool
}

fn serving_loop_configuration() -> ServiceConfiguration {
    ServiceConfiguration {
        oldest_pending_wait_limit: Duration::ZERO,
        library_scan_interval: SCAN_INTERVAL,
        claim_lease: Duration::from_millis(u64::from(LONG_LEASE_MILLISECONDS)),
    }
}

/// 至多等 `limit`（至少问一次）到 `condition` 成立，到了没成立就 panic。
fn wait_until(what: &str, limit: Duration, condition: &mut dyn FnMut() -> bool) {
    let deadline = Instant::now() + limit;
    loop {
        if condition() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "{} 毫秒里没等到：{what}",
            limit.as_millis()
        );
        std::thread::sleep(STEP_POLL);
    }
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

/// 取活循环跑着时测试那一侧看得到的：开了几回卡、放了几回卡、判器的闸。
struct ServingLoopWatch {
    openings: Arc<AtomicU64>,
    let_go: Arc<AtomicU64>,
    gate: Arc<JudgeGate>,
}

/// 在另一条线程里跑取活循环，`scenario` 在这一条线程里演借的一方，演完叫循环停；交回循环的账（循环没跑下去就 panic）。
fn run_the_serving_loop_through(
    pool: &mut LibraryPendingPool<StandInVerificationPackageSide<StandInMaterials>>,
    gate: Arc<JudgeGate>,
    scenario: impl FnOnce(&ServingLoopWatch),
) -> (ServingSummary, ServingLoopWatch) {
    let configuration = serving_loop_configuration();
    let break_switches = BreakSwitches::of_the_environment();
    let watch = ServingLoopWatch {
        openings: Arc::new(AtomicU64::new(0)),
        let_go: Arc::new(AtomicU64::new(0)),
        gate,
    };
    let stop = Arc::new(AtomicBool::new(false));
    let summary = std::thread::scope(|scope| {
        let stop_for_the_loop = Arc::clone(&stop);
        let openings_in_the_loop = Arc::clone(&watch.openings);
        let let_go_in_the_loop = Arc::clone(&watch.let_go);
        let gate_in_the_loop = Arc::clone(&watch.gate);
        let configuration = &configuration;
        let break_switches = &break_switches;
        let serving = scope.spawn(move || {
            let mut open_the_cards = || -> Result<GatedStandInCards, String> {
                openings_in_the_loop.fetch_add(1, Ordering::SeqCst);
                Ok(GatedStandInCards {
                    let_go: Arc::clone(&let_go_in_the_loop),
                    gate: Arc::clone(&gate_in_the_loop),
                })
            };
            serve_from_the_library(
                configuration,
                break_switches,
                pool,
                &mut open_the_cards,
                &|| stop_for_the_loop.load(Ordering::SeqCst),
            )
        });
        let stop_on_drop = StopTheLoopOnDrop {
            stop: Arc::clone(&stop),
            gate: Arc::clone(&watch.gate),
        };
        scenario(&watch);
        drop(stop_on_drop);
        serving.join().expect("取活循环没 panic")
    })
    .unwrap_or_else(|failure| panic!("取活循环没跑下去：{failure:?}"));
    (summary, watch)
}

/// 以旁观者身份、1 毫秒的租期试认领一次：谁持着这把键（没人持着是 `None`）。
fn holder_of(client: &mut ReliableClient, key: Vec<u8>) -> Option<Vec<u8>> {
    match claim_a_card_lease_key_as(client, OBSERVER_IDENTITY, key, PROBE_LEASE_MILLISECONDS)
        .expect("看得了借卡的键")
    {
        CardLeaseKeyAnswer::Claimed => None,
        CardLeaseKeyAnswer::HeldByOther { holder, .. } => Some(holder),
    }
}

/// 这一块此刻有没有人认领着：以 `client` 的身份、1 毫秒的租期试认领一次，认领到就是没人（那 1 毫秒过了自己失效）。
fn block_is_claimed(client: &mut ReliableClient, name: &PendingBlockName) -> bool {
    !matches!(
        claim_a_pending_block(client, name, PROBE_LEASE_MILLISECONDS).expect("认领得了"),
        ClaimAnswer::Claimed
    )
}

/// 这几行里以 `prefix` 起头的。
fn lines_starting_with<'lines>(lines: &'lines [String], prefix: &str) -> Vec<&'lines String> {
    lines
        .iter()
        .filter(|line| line.starts_with(prefix))
        .collect()
}

/// 有人要借：这一圈起三族候选都不认领；手上入了池的块照常判完、写回，都写回之后才放掉卡键、打借出那一行（放之前判完写回 1 块）；
/// 借出期间不认领、不放设备；请求键没人持了（借的一方还了）收回卡键、打收回那一行，接着认领。一个场景按次序几步：同一份库上先后发生。
/// 弄坏开关：card-lease-request-ignored（不探请求键：照认领）、cards-lent-before-the-blocks-in-hand-are-written-back（手上的块没写回就放卡键）、
/// devices-let-go-while-lent（借出时放设备）、card-key-not-taken-back（借的一方还了也不收回）。
#[test]
fn a_lease_request_stops_new_claims_and_the_cards_are_lent_once_the_blocks_in_hand_are_written_back(
) {
    let library = TemporaryLibraryService::start("card-lease-request");
    let keys = CardLeaseKeys::of_this_host(SERVING_LOOP_HOST);
    let mut pool = lending_pool_on(&library, 3);
    let service_holder = pool.holder();
    let store = library.open_store();
    let mut borrower = library.open_client();
    let mut observer = library.open_client();
    let in_hand = block_of(3, 0, 0..4);
    let after_the_request = block_of(3, 1, 4..8);
    let (summary, watch) =
        run_the_serving_loop_through(&mut pool, Arc::new(JudgeGate::default()), |watch| {
            wait_until("取活循环起来就开卡", STEP_WAIT, &mut || {
                watch.openings.load(Ordering::SeqCst) == 1
            });
            put_pending_rows(&store, std::slice::from_ref(&in_hand));
            wait_until(
                "头一块入了池、在闸上等着判",
                STEP_WAIT,
                &mut || watch.gate.blocks_started.load(Ordering::SeqCst) == 1,
            );
            assert_eq!(
                claim_a_card_lease_key_as(
                    &mut borrower,
                    BORROWER_IDENTITY_IN_THE_SERVING_LOOP,
                    keys.request_key(),
                    LONG_LEASE_MILLISECONDS
                )
                .expect("认领得了请求键"),
                CardLeaseKeyAnswer::Claimed,
                "借的一方认领请求键"
            );
            std::thread::sleep(REQUEST_SEEN_SPAN);
            put_pending_rows(&store, std::slice::from_ref(&after_the_request));
            std::thread::sleep(HOLD_SPAN);
            assert!(
                !block_is_claimed(&mut observer, &after_the_request.0),
                "有人要借：请求之后写进来的那一块不认领"
            );
            assert_eq!(
                holder_of(&mut observer, keys.card_key()),
                Some(service_holder.clone()),
                "手上那一块还没判完写回：卡键照持着"
            );
            watch.gate.open();
            wait_until(
                "手上那一块判完写回、待判行删掉",
                STEP_WAIT,
                &mut || !pending_names(&store).contains(&in_hand.0),
            );
            wait_until(
                "写回之后卡键放出来、借的一方认领到",
                STEP_WAIT,
                &mut || {
                    claim_a_card_lease_key_as(
                        &mut borrower,
                        BORROWER_IDENTITY_IN_THE_SERVING_LOOP,
                        keys.card_key(),
                        LONG_LEASE_MILLISECONDS,
                    )
                    .expect("认领得了卡键")
                        == CardLeaseKeyAnswer::Claimed
                },
            );
            std::thread::sleep(HOLD_SPAN);
            assert!(
                pending_names(&store).contains(&after_the_request.0)
                    && !block_is_claimed(&mut observer, &after_the_request.0),
                "借出期间照样不认领"
            );
            assert_eq!(
                (
                    watch.openings.load(Ordering::SeqCst),
                    watch.let_go.load(Ordering::SeqCst)
                ),
                (1, 0),
                "借出期间设备照持着：没放、没再开"
            );
            let released = release_card_lease_keys_as(
                &mut borrower,
                BORROWER_IDENTITY_IN_THE_SERVING_LOOP,
                &[keys.request_key(), keys.card_key()],
            )
            .expect("放得了两把键");
            assert_eq!(released.len(), 2);
            wait_until(
                "借的一方还了之后收回卡键、认领、判完写回请求之后那一块",
                STEP_WAIT,
                &mut || !pending_names(&store).contains(&after_the_request.0),
            );
        });
    let lines = pool.kept_service_lines();
    let lent = lines_starting_with(lines, CARDS_LENT_LINE_PREFIX);
    let taken_back = lines_starting_with(lines, CARDS_TAKEN_BACK_LINE_PREFIX);
    assert_eq!(
        lent,
        [&cards_lent_line(&keys, 1)],
        "借出那一行恰好一行：这一台、放之前判完写回 1 块"
    );
    assert_eq!(taken_back.len(), 1, "收回那一行恰好一行：{taken_back:?}");
    assert!(
        taken_back[0].starts_with(&format!(
            "{CARDS_TAKEN_BACK_LINE_PREFIX} host={SERVING_LOOP_HOST} seconds_lent="
        )),
        "收回那一行写这一台与借出了多少秒：{}",
        taken_back[0]
    );
    let lent_position = lines.iter().position(|line| line == lent[0]);
    let taken_back_position = lines.iter().position(|line| line == taken_back[0]);
    assert!(
        lent_position < taken_back_position,
        "先借出、后收回：{lines:?}"
    );
    assert_eq!((summary.cards_lent, summary.cards_taken_back), (1, 1));
    assert_eq!(summary.blocks_written_back, 2);
    assert_eq!(summary.card_openings, 1, "起来开一回，借出、收回都不再开");
    assert_eq!(watch.let_go.load(Ordering::SeqCst), 1, "停的时候才放掉卡");
}

/// 服务起来时卡键被别人持着（借的一方还没还、或它没了而租期没到）：卡照开，从借出那一态起步（打借出那一行，放之前判完 0 块），
/// 写进来的块不认领；卡键放出来之后收回、认领、判完写回。
/// 弄坏开关：card-key-held-by-another-at-start-ignored（起来时当自己持着，照认领）。
#[test]
fn a_service_starting_while_another_holds_the_card_key_starts_lent_with_its_cards_open() {
    let library = TemporaryLibraryService::start("card-lease-start-lent");
    let keys = CardLeaseKeys::of_this_host(SERVING_LOOP_HOST);
    let mut pool = lending_pool_on(&library, 4);
    let store = library.open_store();
    let mut borrower = library.open_client();
    let mut observer = library.open_client();
    let block = block_of(4, 0, 0..8);
    assert_eq!(
        claim_a_card_lease_key_as(
            &mut borrower,
            BORROWER_IDENTITY_IN_THE_SERVING_LOOP,
            keys.card_key(),
            LONG_LEASE_MILLISECONDS
        )
        .expect("认领得了卡键"),
        CardLeaseKeyAnswer::Claimed
    );
    let (summary, _watch) =
        run_the_serving_loop_through(&mut pool, JudgeGate::opened_at_start(), |watch| {
            wait_until("取活循环起来照开卡", STEP_WAIT, &mut || {
                watch.openings.load(Ordering::SeqCst) == 1
            });
            put_pending_rows(&store, std::slice::from_ref(&block));
            std::thread::sleep(HOLD_SPAN);
            assert!(
                pending_names(&store).contains(&block.0)
                    && !block_is_claimed(&mut observer, &block.0),
                "卡键在别人手里：不认领"
            );
            release_card_lease_keys_as(
                &mut borrower,
                BORROWER_IDENTITY_IN_THE_SERVING_LOOP,
                &[keys.card_key()],
            )
            .expect("放得了卡键");
            wait_until(
                "卡键放出来之后收回、判完写回",
                STEP_WAIT,
                &mut || !pending_names(&store).contains(&block.0),
            );
        });
    let lines = pool.kept_service_lines();
    assert_eq!(
        lines_starting_with(lines, CARDS_LENT_LINE_PREFIX),
        [&cards_lent_line(&keys, 0)],
        "起来时从借出起步：借出那一行一行、放之前判完 0 块"
    );
    assert_eq!(
        lines_starting_with(lines, CARDS_TAKEN_BACK_LINE_PREFIX).len(),
        1
    );
    assert_eq!((summary.card_openings, summary.blocks_written_back), (1, 1));
}

/// 借的一方认领了两把键之后没了、没还（续租进程也没了）：服务照借出办，租期过了之后的那一圈自己收回、接着认领。
/// 弄坏开关 card-key-not-taken-back 下这一条同样红（点名的是上面那一条）。
#[test]
fn the_cards_of_a_borrower_that_vanished_without_returning_them_are_taken_back_once_its_lease_runs_out(
) {
    let library = TemporaryLibraryService::start("card-lease-vanished");
    let keys = CardLeaseKeys::of_this_host(SERVING_LOOP_HOST);
    let mut pool = lending_pool_on(&library, 5);
    let store = library.open_store();
    let mut borrower = library.open_client();
    let block = block_of(5, 0, 0..8);
    for key in [keys.request_key(), keys.card_key()] {
        assert_eq!(
            claim_a_card_lease_key_as(
                &mut borrower,
                BORROWER_IDENTITY_IN_THE_SERVING_LOOP,
                key,
                VANISHING_BORROWER_LEASE_MILLISECONDS
            )
            .expect("认领得了"),
            CardLeaseKeyAnswer::Claimed
        );
    }
    let claimed_at = Instant::now();
    let (summary, _watch) =
        run_the_serving_loop_through(&mut pool, JudgeGate::opened_at_start(), |_watch| {
            put_pending_rows(&store, std::slice::from_ref(&block));
            wait_until(
                "租期过了之后收回、判完写回",
                STEP_WAIT,
                &mut || !pending_names(&store).contains(&block.0),
            );
            assert!(
                claimed_at.elapsed()
                    >= Duration::from_millis(u64::from(VANISHING_BORROWER_LEASE_MILLISECONDS)),
                "租期没过就收回了"
            );
        });
    assert_eq!(
        lines_starting_with(pool.kept_service_lines(), CARDS_TAKEN_BACK_LINE_PREFIX).len(),
        1
    );
    assert_eq!((summary.cards_lent, summary.cards_taken_back), (1, 1));
}

/// 借出期间每分钟卡账那一行照打，字段与平常相同、块数是 0（借出之后整整一段才停）。跑一段卡账周期多两秒。
#[test]
fn while_the_cards_are_lent_the_card_period_line_is_printed_every_minute_with_its_fields_and_no_blocks(
) {
    let library = TemporaryLibraryService::start("card-lease-period");
    let keys = CardLeaseKeys::of_this_host(SERVING_LOOP_HOST);
    let mut pool = lending_pool_on(&library, 6);
    let mut borrower = library.open_client();
    for key in [keys.request_key(), keys.card_key()] {
        claim_a_card_lease_key_as(
            &mut borrower,
            BORROWER_IDENTITY_IN_THE_SERVING_LOOP,
            key,
            LENT_FOR_A_WHOLE_PERIOD_LEASE_MILLISECONDS,
        )
        .expect("认领得了");
    }
    let (summary, _watch) =
        run_the_serving_loop_through(&mut pool, JudgeGate::opened_at_start(), |_watch| {
            std::thread::sleep(CARD_SUMMARY_PERIOD + CARD_PERIOD_MARGIN);
        });
    let period_seconds = CARD_SUMMARY_PERIOD.as_secs().to_string();
    let lines = pool.kept_service_lines();
    let period_lines = lines_starting_with(lines, CARD_PERIOD_LINE_PREFIX);
    assert_eq!(
        period_lines.len(),
        2,
        "满一段那一行与停的时候那一行：{period_lines:?}"
    );
    let full_period = period_lines[0];
    let fields: Vec<(&str, &str)> = full_period
        .split_whitespace()
        .skip(1)
        .map(|field| field.split_once('=').unwrap_or((field, "")))
        .collect();
    assert_eq!(
        fields.iter().map(|(name, _)| *name).collect::<Vec<&str>>(),
        [
            "card",
            "seconds",
            "blocks",
            "states",
            "facts_blocks",
            "facts_states",
            "busy_permille"
        ],
        "字段照平常：{full_period}"
    );
    assert_eq!(
        fields,
        [
            ("card", "0"),
            ("seconds", period_seconds.as_str()),
            ("blocks", "0"),
            ("states", "0"),
            ("facts_blocks", "0"),
            ("facts_states", "0"),
            ("busy_permille", "0"),
        ],
        "借出的这一段一块没判：{full_period}"
    );
    assert!(
        lines_starting_with(lines, CARDS_TAKEN_BACK_LINE_PREFIX).is_empty(),
        "请求键一直持着，没收回"
    );
    assert_eq!(summary.cards_lent, 1);
}

/// 用例自己的目录：一份多机配置（共享根写这个目录本身，判法要它是本机已在的目录），用例结束时删掉。
struct LeaseCase {
    library: TemporaryLibraryService,
    directory: ExclusiveTemporaryDirectory,
    keys: CardLeaseKeys,
    observer: RefCell<ReliableClient>,
}

impl LeaseCase {
    /// 临时判定库、用例目录与一份多机配置：清场之后那一份显卡先后写 `quiesced_cards`。
    fn with_the_cards(label: &str, quiesced_cards: &str) -> Self {
        let library = TemporaryLibraryService::start(label);
        let directory = ExclusiveTemporaryDirectory::create(&format!("card-lease-{label}"))
            .unwrap_or_else(|problem| panic!("{problem}"));
        let configuration = format!(
            "HOSTS={THIS_HOST} {PEER_HOST}\n\
             CARGO_BIN_DIRECTORY_BY_HOST={THIS_HOST}=/card-lease-test/this-cargo {PEER_HOST}=/card-lease-test/peer-cargo\n\
             BUILD_ROOT_BY_HOST={THIS_HOST}=/card-lease-test/this-builds {PEER_HOST}=/card-lease-test/peer-builds\n\
             SLICE_MEMORY_CAP_BY_HOST={THIS_HOST}=2G {PEER_HOST}=1G\n\
             QUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\nQUIESCE_STARTED_CHECK_COMMAND=\n\
             ENABLE_ACROSS_MACHINES=0\nENABLE_GPU=1\n\
             NFS_SHARED_ROOTS={}\n\
             GPU_CARDS_BY_PRIORITY='{PEER_HOST}:0:8G'\n\
             GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='{quiesced_cards}'\n",
            directory.path().display()
        );
        std::fs::write(directory.join("multi-host.env"), configuration)
            .unwrap_or_else(|error| panic!("写不了多机配置：{error}"));
        let observer = RefCell::new(library.open_client());
        Self {
            library,
            directory,
            keys: CardLeaseKeys::of_this_host(THIS_HOST),
            observer,
        }
    }

    fn configuration(&self) -> PathBuf {
        self.directory.join("multi-host.env")
    }

    /// 起 `gpu-verification-server lease <arguments>`：借的一方的变量写 `borrower`（`None` 是不给），放进它自己的进程组。
    fn lease_command(&self, borrower: Option<&str>, arguments: &[&str]) -> Command {
        let mut command = Command::new(SERVER_BINARY);
        command
            .arg("lease")
            .args(arguments)
            .env(
                MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
                self.configuration(),
            )
            .env(
                PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE,
                self.library.socket_path(),
            )
            .env(
                PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE,
                self.library.socket_path(),
            )
            .env_remove(PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        match borrower {
            Some(value) => command.env(BORROWER_VARIABLE, value),
            None => command.env_remove(BORROWER_VARIABLE),
        };
        command
    }

    fn start_lease(&self, borrower: Option<&str>, arguments: &[&str]) -> LeaseProcess {
        LeaseProcess {
            child: self
                .lease_command(borrower, arguments)
                .spawn()
                .expect("起得了 gpu-verification-server lease"),
        }
    }

    /// 起一条 `lease` 子命令、等它退出（到上限 panic）：交回退出状态、标准输出、标准错误。
    fn run_lease(
        &self,
        borrower: Option<&str>,
        arguments: &[&str],
    ) -> (ExitStatus, String, String) {
        let mut process = self.start_lease(borrower, arguments);
        let status = process.exit_within(LEASE_COMMAND_WAIT).unwrap_or_else(|| {
            panic!(
                "lease {arguments:?} {} 秒里没退出",
                LEASE_COMMAND_WAIT.as_secs()
            )
        });
        let (output, error) = process.output_after_the_exit();
        (status, output, error)
    }

    /// 还这个借的一方借的卡（`lease return <这一台的卡串>`），核退 0。
    fn return_the_lease(&self, borrower: &str) {
        let (status, _output, error) =
            self.run_lease(Some(borrower), &["return", CARDS_OF_THIS_HOST]);
        assert_eq!(status.code(), Some(0), "lease return 退 0：{error}");
    }

    fn holder_of(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        holder_of(&mut self.observer.borrow_mut(), key)
    }

    fn holder_identity_of(&self, borrower: &CardLeaseBorrower) -> Vec<u8> {
        borrower.holder_identity(&self.keys)
    }

    /// 可执行文件名在借卡白名单里的借的进程。
    fn listed_borrowing_process(&self) -> BorrowingProcess {
        BorrowingProcess::started_as(&self.directory, LISTED_EXECUTABLE_NAME)
    }

    /// 这一台此刻排着的等号的持有者身份，照排队的次序（旁观的客户端按前缀列认领）。
    fn queued_holders(&self) -> Vec<Vec<u8>> {
        self.observer
            .borrow_mut()
            .list_claims(&self.keys.waiting_ticket_prefix())
            .expect("列得了这一台的等号")
            .into_iter()
            .map(|ticket| ticket.holder_identity)
            .collect()
    }

    /// 以 `holder` 认领（`LONG_LEASE_MILLISECONDS`）这一台的请求键与卡键：别人先持着两把键。
    fn hold_both_keys_as(&self, client: &mut ReliableClient, holder: &[u8]) {
        for key in [self.keys.request_key(), self.keys.card_key()] {
            assert_eq!(
                claim_a_card_lease_key_as(client, holder, key, LONG_LEASE_MILLISECONDS)
                    .expect("认领得了借卡的键"),
                CardLeaseKeyAnswer::Claimed,
                "别人先持着请求键与卡键"
            );
        }
    }

    /// 以 `holder` 放掉这一台的请求键与卡键。
    fn let_both_keys_go_as(&self, client: &mut ReliableClient, holder: &[u8]) {
        release_card_lease_keys_as(
            client,
            holder,
            &[self.keys.request_key(), self.keys.card_key()],
        )
        .expect("放得了两把键");
    }
}

/// 起来的 `lease` 子命令：用例结束（含 panic）时停掉、收尸。
struct LeaseProcess {
    child: Child,
}

impl LeaseProcess {
    /// 当场杀掉、收尸（被杀的 `lease take` 不放它认领到的、不续它的等号）。
    fn kill_now(&mut self) {
        self.child.kill().expect("杀得了自己起的 lease 子命令");
        self.child.wait().expect("收得了自己起的 lease 子命令");
    }

    /// `limit` 里退出了交回退出状态，没退出交 `None`（照跑着）。
    fn exit_within(&mut self, limit: Duration) -> Option<ExitStatus> {
        let deadline = Instant::now() + limit;
        loop {
            if let Some(status) = self.child.try_wait().expect("问得到子进程的状态") {
                return Some(status);
            }
            if Instant::now() >= deadline {
                return None;
            }
            std::thread::sleep(LEASE_COMMAND_POLL);
        }
    }

    /// 退出之后读完标准输出与标准错误。
    fn output_after_the_exit(&mut self) -> (String, String) {
        let mut output = String::new();
        let mut error = String::new();
        if let Some(mut stream) = self.child.stdout.take() {
            stream.read_to_string(&mut output).expect("读得了标准输出");
        }
        if let Some(mut stream) = self.child.stderr.take() {
            stream.read_to_string(&mut error).expect("读得了标准错误");
        }
        (output, error)
    }
}

impl Drop for LeaseProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// 借到了的那一份：用例结束（含 panic）时还掉，续租进程照叫停的办法退出。
struct TakenLease<'case> {
    case: &'case LeaseCase,
    borrower: String,
    is_returned: bool,
}

impl TakenLease<'_> {
    fn return_now(&mut self) {
        self.case.return_the_lease(&self.borrower);
        self.is_returned = true;
    }
}

/// 丢掉时（含 panic 收尾的那一回）不再 panic：起不来、没退出都照放手。
impl Drop for TakenLease<'_> {
    fn drop(&mut self) {
        if self.is_returned {
            return;
        }
        let returning = self
            .case
            .lease_command(Some(&self.borrower), &["return", CARDS_OF_THIS_HOST])
            .spawn();
        if let Ok(child) = returning {
            let _ = LeaseProcess { child }.exit_within(LEASE_COMMAND_WAIT);
        }
    }
}

/// 停着的一个 `lease` 子命令（它自己的进程组整组 SIGSTOP）：丢掉时（含 panic 收尾的那一回）SIGCONT，接着跑。
struct PausedLeaseProcess {
    process_group: String,
}

impl PausedLeaseProcess {
    fn paused(process: &LeaseProcess) -> Self {
        let paused = Self {
            process_group: format!("-{}", process.child.id()),
        };
        let status = paused
            .signal(STOP_SIGNAL)
            .unwrap_or_else(|error| panic!("起不了 {SIGNAL_PROGRAM}：{error}"));
        assert!(
            status.success(),
            "停那个进程组的 {SIGNAL_PROGRAM} 退了 {status}"
        );
        paused
    }

    /// 对这个进程组发 `signal`（它自己的进程组：`lease_command` 照 `process_group(0)` 起）。
    fn signal(&self, signal: &str) -> std::io::Result<ExitStatus> {
        Command::new(SIGNAL_PROGRAM)
            .args(["-s", signal, "--", &self.process_group])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    }
}

/// 丢掉时不再 panic：发不出去的照放手（那个进程随后由它的守卫杀掉、收尸）。
impl Drop for PausedLeaseProcess {
    fn drop(&mut self) {
        let _ = self.signal(CONTINUE_SIGNAL);
    }
}

/// 编进二进制的那份服务配置里的认领租期（等号的租期取它，`src/card_lease.rs` 文件头「先来后到」）。
fn compiled_claim_lease() -> Duration {
    let configuration = ServiceConfiguration::from_the_text(SERVICE_CONFIGURATION_TEXT)
        .unwrap_or_else(|problem| panic!("编进来的服务配置解不开：{problem}"));
    Duration::from_millis(u64::from(configuration.claim_lease_milliseconds()))
}

/// `lease take` 标准错误里等的那几行里有没有一行带 `position=<position>` 这一项。
fn waiting_lines_name_the_position(error: &str, position: usize) -> bool {
    let field = format!("position={position}");
    error
        .lines()
        .filter(|line| line.starts_with(CARD_LEASE_WAITING_LINE_PREFIX))
        .any(|line| line.split_whitespace().any(|written| written == field))
}

/// `lease list` 的一行拆成两半：`lease_left_milliseconds=` 之前那一截与剩余租期（这一行不以它收尾就 panic）。
fn split_off_the_lease_left(line: &str) -> (&str, u64) {
    let (before, lease_left_written) = line
        .rsplit_once(" lease_left_milliseconds=")
        .unwrap_or_else(|| panic!("这一行不以 lease_left_milliseconds= 收尾：{line}"));
    let lease_left_milliseconds = lease_left_written
        .parse()
        .unwrap_or_else(|error| panic!("剩余租期「{lease_left_written}」认不出：{error}"));
    (before, lease_left_milliseconds)
}

/// `PATH` 里头一个 `sleep`。
fn long_running_program() -> PathBuf {
    let search_path = std::env::var_os("PATH").expect("环境里有 PATH");
    std::env::split_paths(&search_path)
        .map(|directory| directory.join(LONG_RUNNING_PROGRAM))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("PATH 里找不到 {LONG_RUNNING_PROGRAM}"))
}

/// 测试起的借的进程（拷进用例目录、改了名的 `sleep`，在它自己的进程组里）：用例结束（含 panic）时停掉、收尸。
struct BorrowingProcess {
    child: Child,
    borrower: CardLeaseBorrower,
}

impl BorrowingProcess {
    /// 把 `sleep` 拷进 `directory`、改名成 `executable_name` 起（拷过的照用）：`/proc/<进程号>/exe` 指到的文件名就是 `executable_name`。
    /// argv[0] 照写 `sleep`：多合一的 coreutils 照 argv[0] 认自己是哪个命令。
    fn started_as(directory: &ExclusiveTemporaryDirectory, executable_name: &str) -> Self {
        let executable = directory.join(executable_name);
        if !executable.exists() {
            std::fs::copy(long_running_program(), &executable).unwrap_or_else(|error| {
                panic!(
                    "拷不了 {LONG_RUNNING_PROGRAM} 到 {}：{error}",
                    executable.display()
                )
            });
        }
        let mut busy_retries = 0;
        // 迭代上界：TEXT_FILE_BUSY_RETRIES 回之后照 panic
        let child = loop {
            let spawned = Command::new(&executable)
                .arg0(LONG_RUNNING_PROGRAM)
                .arg(LONG_RUNNING_PROGRAM_SECONDS)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .process_group(0)
                .spawn();
            match spawned {
                Ok(child) => break child,
                Err(error)
                    if error.kind() == std::io::ErrorKind::ExecutableFileBusy
                        && busy_retries < TEXT_FILE_BUSY_RETRIES =>
                {
                    busy_retries += 1;
                    std::thread::sleep(TEXT_FILE_BUSY_PAUSE);
                }
                Err(error) => panic!("起不了 {}：{error}", executable.display()),
            }
        };
        let borrower = CardLeaseBorrower::of_the_running_process(child.id())
            .unwrap_or_else(|problem| panic!("读不到借的进程的启动刻：{problem}"));
        Self { child, borrower }
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for BorrowingProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 这个测试进程自己当借的进程。
fn this_process_as_the_borrower() -> CardLeaseBorrower {
    CardLeaseBorrower::of_the_running_process(std::process::id())
        .unwrap_or_else(|problem| panic!("读不到这个进程的启动刻：{problem}"))
}

/// `lease take` 打在标准错误的那一行里续租进程的 `<进程号>:<启动刻>`。
fn renewer_of_the_taken_line(error: &str) -> CardLeaseBorrower {
    let line = error
        .lines()
        .find(|line| line.starts_with(CARD_LEASE_TAKEN_LINE_PREFIX))
        .unwrap_or_else(|| {
            panic!("lease take 的标准错误里没有 {CARD_LEASE_TAKEN_LINE_PREFIX} 那一行：{error}")
        });
    let renewer = line
        .split_whitespace()
        .find_map(|field| field.strip_prefix("renewer="))
        .unwrap_or_else(|| panic!("那一行没有 renewer=：{line}"));
    CardLeaseBorrower::of_the_variable_value(Some(renewer))
        .unwrap_or_else(|problem| panic!("续租进程那一项认不出：{problem}"))
}

fn is_gone(process: &CardLeaseBorrower) -> bool {
    process.liveness() == Ok(ProcessLiveness::Gone)
}

/// `lease take` 要认领到卡键之后才打卡串：卡键被服务持着时它认领了请求键、一直等，不打；服务放了卡键它才认领到、打这一台的卡串
/// （只这一台的那几项、照原来的先后）、退 0，两把键都在借的一方名下。`lease return` 退 0 之后两把键都放了、续租进程 5 秒内退出；再还一次照退 0。
/// 一个场景按次序几步：借、还在同一份库上先后发生。
/// 弄坏开关：card-lease-printed-before-the-card-key（认领到请求键就打）、card-lease-return-leaves-the-renewer-running（还卡不叫停续租进程）。
#[test]
fn lease_take_prints_the_cards_only_once_it_holds_the_card_key_and_lease_return_lets_both_keys_go()
{
    let case = LeaseCase::with_the_cards(
        "take-after-the-card-key",
        CARD_STRING_WITH_CARDS_OF_THIS_HOST,
    );
    let mut service = case.library.open_client();
    let service_identity = service.client_identity().to_vec();
    assert_eq!(
        claim_a_card_lease_key_as(
            &mut service,
            &service_identity,
            case.keys.card_key(),
            LONG_LEASE_MILLISECONDS
        )
        .expect("认领得了卡键"),
        CardLeaseKeyAnswer::Claimed,
        "服务持着卡键"
    );
    let borrowing = case.listed_borrowing_process();
    let borrower = borrowing.borrower;
    let borrower_value = borrower.variable_value();
    let holder = case.holder_identity_of(&borrower);
    let mut taking = case.start_lease(Some(&borrower_value), &["take"]);
    let mut lease = TakenLease {
        case: &case,
        borrower: borrower_value.clone(),
        is_returned: false,
    };
    wait_until(
        "lease take 认领到请求键",
        LEASE_COMMAND_WAIT,
        &mut || case.holder_of(case.keys.request_key()) == Some(holder.clone()),
    );
    assert_eq!(
        taking.exit_within(TAKE_STILL_WAITING_SPAN),
        None,
        "卡键在服务手里：lease take 照等，不打卡串、不退出"
    );
    release_card_lease_keys_as(&mut service, &service_identity, &[case.keys.card_key()])
        .expect("服务放得了卡键");
    let status = taking
        .exit_within(LEASE_COMMAND_WAIT)
        .expect("服务放了卡键之后 lease take 借到、退出");
    let (output, error) = taking.output_after_the_exit();
    assert_eq!(status.code(), Some(0), "借到退 0：{error}");
    assert_eq!(
        output,
        format!("{CARDS_OF_THIS_HOST}\n"),
        "标准输出只有这一台的卡串"
    );
    assert_eq!(
        case.holder_of(case.keys.card_key()),
        Some(holder.clone()),
        "卡键在借的一方名下"
    );
    assert_eq!(
        case.holder_of(case.keys.request_key()),
        Some(holder.clone()),
        "请求键在借的一方名下"
    );
    let renewer = renewer_of_the_taken_line(&error);
    assert_eq!(
        renewer.liveness(),
        Ok(ProcessLiveness::Running),
        "续租进程在跑"
    );
    let returned_at = Instant::now();
    lease.return_now();
    assert_eq!(
        (
            case.holder_of(case.keys.request_key()),
            case.holder_of(case.keys.card_key())
        ),
        (None, None),
        "lease return 退 0 之后两把键都放了"
    );
    wait_until(
        "lease return 之后续租进程退出",
        RELEASE_LIMIT.saturating_sub(returned_at.elapsed()),
        &mut || is_gone(&renewer),
    );
    case.return_the_lease(&borrower_value);
}

/// 同一台两个借的一方：只有一个先借到；另一个等着（不打卡串、不退出），前一个还了它才借到。
/// 弄坏开关：card-lease-keys-held-by-another-taken-as-claimed（别人持着也当认领到：两个一起借到）。
#[test]
fn two_borrowers_on_one_host_get_the_cards_one_after_the_other() {
    let case = LeaseCase::with_the_cards("two-borrowers", CARD_STRING_WITH_CARDS_OF_THIS_HOST);
    let borrowing_processes = [
        case.listed_borrowing_process(),
        case.listed_borrowing_process(),
    ];
    let borrowers = borrowing_processes
        .each_ref()
        .map(|process| process.borrower.variable_value());
    let mut takings = [
        case.start_lease(Some(&borrowers[0]), &["take"]),
        case.start_lease(Some(&borrowers[1]), &["take"]),
    ];
    let mut leases = borrowers.clone().map(|borrower| TakenLease {
        case: &case,
        borrower,
        is_returned: false,
    });
    let deadline = Instant::now() + LEASE_COMMAND_WAIT;
    let first = loop {
        if let Some(first) =
            (0..2).find(|&position| takings[position].exit_within(Duration::ZERO).is_some())
        {
            break first;
        }
        assert!(
            Instant::now() < deadline,
            "{} 秒里两个都没借到",
            LEASE_COMMAND_WAIT.as_secs()
        );
        std::thread::sleep(LEASE_COMMAND_POLL);
    };
    let second = 1 - first;
    let (first_output, first_error) = takings[first].output_after_the_exit();
    assert_eq!(
        first_output,
        format!("{CARDS_OF_THIS_HOST}\n"),
        "先借到的打卡串：{first_error}"
    );
    assert_eq!(
        takings[second].exit_within(TAKE_STILL_WAITING_SPAN),
        None,
        "前一个还没还：后一个照等"
    );
    leases[first].return_now();
    let status = takings[second]
        .exit_within(LEASE_COMMAND_WAIT)
        .expect("前一个还了之后后一个借到");
    let (second_output, second_error) = takings[second].output_after_the_exit();
    assert_eq!(status.code(), Some(0), "后一个借到退 0：{second_error}");
    assert_eq!(second_output, format!("{CARDS_OF_THIS_HOST}\n"));
    let second_borrower =
        CardLeaseBorrower::of_the_variable_value(Some(&borrowers[second])).expect("认得回");
    assert_eq!(
        case.holder_of(case.keys.card_key()),
        Some(case.holder_identity_of(&second_borrower)),
        "卡键在后一个名下"
    );
    leases[second].return_now();
}

/// 借的进程不在了（没还）：续租进程 5 秒内放掉两把键、退出，不等租期。
/// 弄坏开关：card-lease-renewer-ignores-the-borrower（续租进程不看借的进程在不在）。
#[test]
fn both_keys_are_let_go_within_five_seconds_once_the_borrower_is_gone() {
    let case = LeaseCase::with_the_cards("borrower-gone", CARD_STRING_WITH_CARDS_OF_THIS_HOST);
    let mut borrowing = case.listed_borrowing_process();
    let borrower_value = borrowing.borrower.variable_value();
    let _lease = TakenLease {
        case: &case,
        borrower: borrower_value.clone(),
        is_returned: false,
    };
    let (status, output, error) = case.run_lease(Some(&borrower_value), &["take"]);
    assert_eq!(
        (status.code(), output.as_str()),
        (Some(0), format!("{CARDS_OF_THIS_HOST}\n").as_str()),
        "借到：{error}"
    );
    let renewer = renewer_of_the_taken_line(&error);
    let holder = case.holder_identity_of(&borrowing.borrower);
    assert_eq!(case.holder_of(case.keys.card_key()), Some(holder));
    borrowing.stop();
    wait_until(
        "借的进程没了之后两把键都放了、续租进程退出",
        RELEASE_LIMIT,
        &mut || {
            case.holder_of(case.keys.request_key()).is_none()
                && case.holder_of(case.keys.card_key()).is_none()
                && is_gone(&renewer)
        },
    );
}

/// 借的一方的变量没给或写法不对：`lease take`、`lease return` 都退 2、说明里点名那个变量，不认领；`lease return` 一个参数都没给、`lease` 后面认不出的也退 2。
/// 弄坏开关：card-lease-borrower-from-the-parent-process（变量没给时取父进程：照借）。
#[test]
fn lease_subcommands_refuse_a_missing_or_malformed_borrower_variable_with_the_usage_exit() {
    let case = LeaseCase::with_the_cards("borrower-variable", CARD_STRING_WITH_CARDS_OF_THIS_HOST);
    let borrower_value = this_process_as_the_borrower().variable_value();
    // 弄坏开关下变量没给的那一条照借（借的进程是这个测试进程）：结束时照这个身份还掉
    let _lease = TakenLease {
        case: &case,
        borrower: borrower_value.clone(),
        is_returned: false,
    };
    for written in [
        None,
        Some(""),
        Some("abc"),
        Some("4242"),
        Some("4242:"),
        Some(":5"),
        Some("0:5"),
        Some("1:2:3"),
    ] {
        for arguments in [&["take"][..], &["return", CARDS_OF_THIS_HOST][..]] {
            let (status, output, error) = case.run_lease(written, arguments);
            assert_eq!(
                (status.code(), output.as_str()),
                (Some(USAGE_EXIT), ""),
                "变量 {written:?}、lease {arguments:?}：退 2、不打卡串：{error}"
            );
            assert!(
                error.contains(BORROWER_VARIABLE),
                "说明点名 {BORROWER_VARIABLE}：{error}"
            );
        }
    }
    for arguments in [
        &["return"][..],
        &["return", CARDS_OF_THIS_HOST, CARDS_OF_THIS_HOST][..],
        &["borrow"][..],
        &[][..],
    ] {
        let (status, _output, error) = case.run_lease(Some(&borrower_value), arguments);
        assert_eq!(
            status.code(),
            Some(USAGE_EXIT),
            "lease {arguments:?} 退 2：{error}"
        );
    }
    assert_eq!(
        (
            case.holder_of(case.keys.request_key()),
            case.holder_of(case.keys.card_key())
        ),
        (None, None),
        "拒了的一条键都没认领"
    );
}

/// `lease return` 的卡串参数给了就行：借到之后 `lease return ''`（借的命令退 0 而卡串是空的、借的一方当场还的那一支）退 0、两把键都放了、
/// 续租进程退出，标准错误那一行带着空的卡串；`lease return` 一个参数都没给退 2、不还，两把键照在借的一方名下。
/// 弄坏开关：card-lease-return-refuses-an-empty-card-string（空的卡串当用法错、退 2）。
#[test]
fn lease_return_takes_an_empty_card_string_and_refuses_a_missing_one_with_the_usage_exit() {
    let case = LeaseCase::with_the_cards(
        "return-an-empty-card-string",
        CARD_STRING_WITH_CARDS_OF_THIS_HOST,
    );
    let borrowing = case.listed_borrowing_process();
    let borrower = borrowing.borrower;
    let borrower_value = borrower.variable_value();
    let holder = case.holder_identity_of(&borrower);
    let mut lease = TakenLease {
        case: &case,
        borrower: borrower_value.clone(),
        is_returned: false,
    };
    let (taken_status, taken_output, taken_error) =
        case.run_lease(Some(&borrower_value), &["take"]);
    assert_eq!(
        (taken_status.code(), taken_output.as_str()),
        (Some(0), format!("{CARDS_OF_THIS_HOST}\n").as_str()),
        "借到：{taken_error}"
    );
    let renewer = renewer_of_the_taken_line(&taken_error);
    let (bare_status, bare_output, bare_error) = case.run_lease(Some(&borrower_value), &["return"]);
    assert_eq!(
        (bare_status.code(), bare_output.as_str()),
        (Some(USAGE_EXIT), ""),
        "lease return 一个参数都没给：退 2：{bare_error}"
    );
    assert_eq!(
        (
            case.holder_of(case.keys.request_key()),
            case.holder_of(case.keys.card_key())
        ),
        (Some(holder.clone()), Some(holder)),
        "没给参数的那一条不还：两把键照在借的一方名下"
    );
    let returned_at = Instant::now();
    let (empty_status, _empty_output, empty_error) =
        case.run_lease(Some(&borrower_value), &["return", ""]);
    assert_eq!(
        empty_status.code(),
        Some(0),
        "两把键在自己名下时 lease return '' 退 0：{empty_error}"
    );
    lease.is_returned = true;
    assert!(
        empty_error
            .lines()
            .any(|line| line.starts_with(CARD_LEASE_RETURNING_LINE_PREFIX)
                && line.ends_with("card_string=\"\"")),
        "标准错误那一行带着空的卡串：{empty_error}"
    );
    assert_eq!(
        (
            case.holder_of(case.keys.request_key()),
            case.holder_of(case.keys.card_key())
        ),
        (None, None),
        "lease return '' 退 0 之后两把键都放了"
    );
    wait_until(
        "lease return '' 之后续租进程退出",
        RELEASE_LIMIT.saturating_sub(returned_at.elapsed()),
        &mut || is_gone(&renewer),
    );
}

/// 多机配置里这一台一张卡都没点名：`lease take` 退 78、不打卡串、不认领。
/// 弄坏开关：card-lease-taken-without-cards-of-this-host（照借，卡串是空的）。
#[test]
fn lease_take_refuses_with_78_when_the_configuration_names_no_card_of_this_host() {
    let case = LeaseCase::with_the_cards(
        "no-cards-of-this-host",
        CARD_STRING_WITHOUT_CARDS_OF_THIS_HOST,
    );
    let borrowing = case.listed_borrowing_process();
    let borrower_value = borrowing.borrower.variable_value();
    let _lease = TakenLease {
        case: &case,
        borrower: borrower_value.clone(),
        is_returned: false,
    };
    let (status, output, error) = case.run_lease(Some(&borrower_value), &["take"]);
    assert_eq!(
        (status.code(), output.as_str()),
        (Some(RUN_CONDITION_EXIT), ""),
        "这一台没有卡：退 78、不打卡串：{error}"
    );
    assert_eq!(
        (
            case.holder_of(case.keys.request_key()),
            case.holder_of(case.keys.card_key())
        ),
        (None, None),
        "没有卡不认领"
    );
}

/// `lease take` 照借卡白名单拒了 `borrower`（它的可执行文件名是 `executable_name`）：退白名单的码、不打卡串、两把键都没被认领；
/// 标准错误先是拒绝那一行（借的进程号、带引号的可执行文件名），再是 `✗` 一行（点名借的进程号、可执行文件名与白名单的路径）与出路一行。
fn assert_lease_take_refuses_off_the_whitelist(
    case: &LeaseCase,
    borrower: &CardLeaseBorrower,
    executable_name: &str,
) {
    let (status, output, error) = case.run_lease(Some(&borrower.variable_value()), &["take"]);
    assert_eq!(
        (status.code(), output.as_str()),
        (Some(i32::from(BORROWER_OFF_THE_WHITELIST_EXIT)), ""),
        "可执行文件名「{executable_name}」不在白名单里：退 {BORROWER_OFF_THE_WHITELIST_EXIT}、不打卡串：{error}"
    );
    assert_eq!(
        (
            case.holder_of(case.keys.request_key()),
            case.holder_of(case.keys.card_key())
        ),
        (None, None),
        "拒了的一把键都没认领"
    );
    let lines: Vec<&str> = error.lines().collect();
    assert_eq!(
        lines.first().copied(),
        Some(
            format!(
                "{CARD_LEASE_REFUSED_LINE_PREFIX} borrower_process={} executable=\"{executable_name}\"",
                borrower.process_number
            )
            .as_str()
        ),
        "标准错误头一行是拒绝那一行：{error}"
    );
    let problem_line = lines
        .iter()
        .find(|line| line.trim_start().starts_with('✗'))
        .unwrap_or_else(|| panic!("标准错误里没有 ✗ 那一行：{error}"));
    for named in [
        borrower.process_number.to_string().as_str(),
        executable_name,
        CARD_LEASE_BORROWERS_PATH_IN_THE_REPOSITORY,
    ] {
        assert!(
            problem_line.contains(named),
            "✗ 那一行点名「{named}」：{problem_line}"
        );
    }
    assert!(
        lines.iter().any(|line| line.contains("卡只归 GPU 服务")
            && line.contains(CARD_LEASE_BORROWERS_PATH_IN_THE_REPOSITORY)
            && line.contains("control install")),
        "出路那一行写卡只归 GPU 服务、往白名单加一行、重装服务：{error}"
    );
    assert!(
        !error
            .lines()
            .any(|line| line.starts_with(CARD_LEASE_TAKEN_LINE_PREFIX)),
        "拒了的不起续租进程：{error}"
    );
}

/// 借的一方是一个可执行文件名不在借卡白名单里的进程（这个测试进程自己：cargo 测试二进制，文件名带哈希后缀）：`lease take` 退白名单的码、
/// 两把键都没认领，标准错误里有拒绝那一行与 `✗`、出路两行。
/// 弄坏开关：card-lease-borrower-whitelist-unchecked（不认借的一方：照借）。
#[test]
fn lease_take_refuses_a_borrower_whose_executable_name_is_not_on_the_whitelist_and_claims_no_key() {
    let case = LeaseCase::with_the_cards(
        "borrower-off-the-whitelist",
        CARD_STRING_WITH_CARDS_OF_THIS_HOST,
    );
    let borrower = this_process_as_the_borrower();
    // 弄坏开关下照借（借的进程是这个测试进程）：结束时照这个身份还掉
    let _lease = TakenLease {
        case: &case,
        borrower: borrower.variable_value(),
        is_returned: false,
    };
    let current_executable = std::env::current_exe().expect("读得到这个进程的可执行文件");
    let executable_name = current_executable
        .file_name()
        .and_then(|name| name.to_str())
        .expect("测试二进制的文件名是 UTF-8");
    assert!(
        !CardLeaseBorrowerWhitelist::compiled_in()
            .expect("编进来的白名单解得开")
            .lets_borrow(executable_name),
        "这个测试二进制「{executable_name}」不在白名单里，这一条才判得了拒"
    );
    assert_lease_take_refuses_off_the_whitelist(&case, &borrower, executable_name);
}

/// 可执行文件名是白名单里那一项接上 cargo 测试二进制那种 `-<十六位哈希>` 后缀的进程：只比得上前缀，照拒、两把键都没认领。
#[test]
fn lease_take_refuses_a_borrower_named_like_a_cargo_test_binary_of_the_listed_executable() {
    let case = LeaseCase::with_the_cards(
        "borrower-with-a-hash-suffix",
        CARD_STRING_WITH_CARDS_OF_THIS_HOST,
    );
    let borrowing = BorrowingProcess::started_as(
        &case.directory,
        LISTED_EXECUTABLE_NAME_WITH_A_CARGO_HASH_SUFFIX,
    );
    let _lease = TakenLease {
        case: &case,
        borrower: borrowing.borrower.variable_value(),
        is_returned: false,
    };
    assert_lease_take_refuses_off_the_whitelist(
        &case,
        &borrowing.borrower,
        LISTED_EXECUTABLE_NAME_WITH_A_CARGO_HASH_SUFFIX,
    );
}

/// 借的一方是一个可执行文件名在借卡白名单里的进程（改名成 `e202_gpu_cpu_gc_comparison` 的 `sleep`）：`lease take` 借到、退 0、打这一台的卡串，
/// 两把键都在它名下，标准错误里没有拒绝那一行。
#[test]
fn a_borrower_whose_executable_name_is_on_the_whitelist_takes_the_cards() {
    let case = LeaseCase::with_the_cards(
        "borrower-on-the-whitelist",
        CARD_STRING_WITH_CARDS_OF_THIS_HOST,
    );
    assert!(
        CardLeaseBorrowerWhitelist::compiled_in()
            .expect("编进来的白名单解得开")
            .lets_borrow(LISTED_EXECUTABLE_NAME),
        "{LISTED_EXECUTABLE_NAME} 在白名单里"
    );
    let borrowing = case.listed_borrowing_process();
    let mut lease = TakenLease {
        case: &case,
        borrower: borrowing.borrower.variable_value(),
        is_returned: false,
    };
    let (status, output, error) =
        case.run_lease(Some(&borrowing.borrower.variable_value()), &["take"]);
    assert_eq!(
        (status.code(), output.as_str()),
        (Some(0), format!("{CARDS_OF_THIS_HOST}\n").as_str()),
        "白名单里的借到：{error}"
    );
    assert!(
        !error
            .lines()
            .any(|line| line.starts_with(CARD_LEASE_REFUSED_LINE_PREFIX)),
        "借到的没有拒绝那一行：{error}"
    );
    let holder = case.holder_identity_of(&borrowing.borrower);
    assert_eq!(
        (
            case.holder_of(case.keys.request_key()),
            case.holder_of(case.keys.card_key())
        ),
        (Some(holder.clone()), Some(holder)),
        "两把键都在白名单里那个借的进程名下"
    );
    lease.return_now();
}

/// 先来后到（`src/card_lease.rs` 文件头「先来后到」）：请求键与卡键先被别人（服务那一个身份）持着，两个在借卡白名单里的借的一方先后对同一台起
/// `lease take`，两个都排着：先等的那一张排第一、后等的排第二，后等的那一个等的那一行带 `position=2`。别人放掉两把键时先等的那一个
/// `lease take` 停着（整组 SIGSTOP，停 [`EARLIER_TAKE_PAUSED_SPAN`]，比两次再试的间隔长）：照先来后到后等的那一个排第二、只能等着；
/// 先等的那一个接着跑之后先借到，后等的照等，先等的还了它才借到。一个场景按次序几步：同一份库上先后发生。
/// 弄坏开关：card-lease-waiting-order-ignored（不看等号次序：先等的停着的那一段里后等的撞上请求键、先借到）。
#[test]
fn the_borrower_that_started_waiting_first_takes_the_cards_first_and_the_later_one_waits_second_in_the_queue(
) {
    let case = LeaseCase::with_the_cards("waiting-order", CARD_STRING_WITH_CARDS_OF_THIS_HOST);
    let mut service = case.library.open_client();
    let service_identity = service.client_identity().to_vec();
    case.hold_both_keys_as(&mut service, &service_identity);
    let earlier_process = case.listed_borrowing_process();
    let later_process = case.listed_borrowing_process();
    let earlier_borrower = earlier_process.borrower.variable_value();
    let later_borrower = later_process.borrower.variable_value();
    let earlier_holder = case.holder_identity_of(&earlier_process.borrower);
    let later_holder = case.holder_identity_of(&later_process.borrower);
    let mut earlier_taking = case.start_lease(Some(&earlier_borrower), &["take"]);
    let mut earlier_lease = TakenLease {
        case: &case,
        borrower: earlier_borrower,
        is_returned: false,
    };
    wait_until(
        "先等的那一张等号排上",
        LEASE_COMMAND_WAIT,
        &mut || case.queued_holders() == [earlier_holder.clone()],
    );
    std::thread::sleep(GAP_BETWEEN_THE_TWO_WAITS);
    let mut later_taking = case.start_lease(Some(&later_borrower), &["take"]);
    let mut later_lease = TakenLease {
        case: &case,
        borrower: later_borrower,
        is_returned: false,
    };
    wait_until(
        "后等的那一张排在先等的后面",
        LEASE_COMMAND_WAIT,
        &mut || case.queued_holders() == [earlier_holder.clone(), later_holder.clone()],
    );
    {
        let _earlier_paused = PausedLeaseProcess::paused(&earlier_taking);
        case.let_both_keys_go_as(&mut service, &service_identity);
        assert_eq!(
            later_taking.exit_within(EARLIER_TAKE_PAUSED_SPAN),
            None,
            "两把键放了、先等的那一个停着：后等的排第二，照等、不借"
        );
    }
    let earlier_status = earlier_taking
        .exit_within(LEASE_COMMAND_WAIT)
        .expect("先等的那一个接着跑之后借到、退出");
    let (earlier_output, earlier_error) = earlier_taking.output_after_the_exit();
    assert_eq!(
        earlier_status.code(),
        Some(0),
        "先等的借到退 0：{earlier_error}"
    );
    assert_eq!(earlier_output, format!("{CARDS_OF_THIS_HOST}\n"));
    assert_eq!(
        case.holder_of(case.keys.card_key()),
        Some(earlier_holder),
        "卡键在先等的那一个名下"
    );
    assert_eq!(
        later_taking.exit_within(TAKE_STILL_WAITING_SPAN),
        None,
        "先等的还没还：后等的照等"
    );
    earlier_lease.return_now();
    let later_status = later_taking
        .exit_within(LEASE_COMMAND_WAIT)
        .expect("先等的还了之后后等的借到");
    let (later_output, later_error) = later_taking.output_after_the_exit();
    assert_eq!(
        later_status.code(),
        Some(0),
        "后等的借到退 0：{later_error}"
    );
    assert_eq!(later_output, format!("{CARDS_OF_THIS_HOST}\n"));
    assert!(
        waiting_lines_name_the_position(&later_error, 2),
        "后等的那一个等的那一行带 position=2：{later_error}"
    );
    assert_eq!(
        (case.holder_of(case.keys.card_key()), case.queued_holders()),
        (Some(later_holder), Vec::new()),
        "卡键在后等的那一个名下，两张等号都放了"
    );
    later_lease.return_now();
}

/// 排在最前的那个借的一方的 `lease take` 被杀（它的等号没人续）：请求键与卡键先被别人持着，两个都排上之后杀掉排在最前的那一个、别人放掉两把键；
/// 后面排着的那一个照等，那一张等号照在最前，到它的租期（编进二进制的认领租期）过了后面那一个才排到最前、借到。
/// 这一条跑一个认领租期多（两分钟多）。
#[test]
fn a_borrower_queued_behind_a_killed_lease_take_takes_the_cards_once_the_ticket_of_the_killed_one_lapses(
) {
    let ticket_lease = compiled_claim_lease();
    let case = LeaseCase::with_the_cards("killed-front", CARD_STRING_WITH_CARDS_OF_THIS_HOST);
    let mut service = case.library.open_client();
    let service_identity = service.client_identity().to_vec();
    case.hold_both_keys_as(&mut service, &service_identity);
    let killed_process = case.listed_borrowing_process();
    let waiting_process = case.listed_borrowing_process();
    let killed_holder = case.holder_identity_of(&killed_process.borrower);
    let waiting_holder = case.holder_identity_of(&waiting_process.borrower);
    let waiting_borrower = waiting_process.borrower.variable_value();
    let started_before_the_killed_one = Instant::now();
    let mut killed_taking =
        case.start_lease(Some(&killed_process.borrower.variable_value()), &["take"]);
    wait_until(
        "排在最前的那一张等号排上",
        LEASE_COMMAND_WAIT,
        &mut || case.queued_holders() == [killed_holder.clone()],
    );
    std::thread::sleep(GAP_BETWEEN_THE_TWO_WAITS);
    let mut waiting_taking = case.start_lease(Some(&waiting_borrower), &["take"]);
    let mut waiting_lease = TakenLease {
        case: &case,
        borrower: waiting_borrower,
        is_returned: false,
    };
    wait_until(
        "后面那一张排在它后面",
        LEASE_COMMAND_WAIT,
        &mut || case.queued_holders() == [killed_holder.clone(), waiting_holder.clone()],
    );
    killed_taking.kill_now();
    case.let_both_keys_go_as(&mut service, &service_identity);
    assert_eq!(
        waiting_taking.exit_within(TAKE_STILL_WAITING_SPAN),
        None,
        "被杀的那一张等号还在：后面那一个照等"
    );
    assert_eq!(
        case.queued_holders(),
        [killed_holder, waiting_holder.clone()],
        "被杀的那一张等号到租期之前照在最前"
    );
    let status = waiting_taking
        .exit_within(ticket_lease + LEASE_COMMAND_WAIT)
        .expect("被杀的那一张等号租期过了之后后面那一个借到");
    let waited = started_before_the_killed_one.elapsed();
    let (output, error) = waiting_taking.output_after_the_exit();
    assert_eq!(status.code(), Some(0), "借到退 0：{error}");
    assert!(
        waited >= ticket_lease,
        "被杀的那一张等号的租期（{} 毫秒）没过就借到了：从起它算过了 {} 毫秒",
        ticket_lease.as_millis(),
        waited.as_millis()
    );
    assert_eq!(output, format!("{CARDS_OF_THIS_HOST}\n"));
    assert_eq!(
        case.holder_of(case.keys.card_key()),
        Some(waiting_holder),
        "卡键在后面那一个名下"
    );
    waiting_lease.return_now();
}

/// 没人借、没人排时 `lease list`（不给借的一方的变量）：退 0，多机配置主机数组里点名了卡的每一台照主机数组的次序各一行卡键那一行，
/// 写没人持着（`held_by=nobody holder=none`、剩余租期 0）与那一台的卡串；没有请求键与排队的行，一把键都不认领。
#[test]
fn lease_list_with_nobody_borrowing_prints_one_held_line_per_host_with_cards_and_nothing_else() {
    let case = LeaseCase::with_the_cards("list-nobody", CARD_STRING_WITH_CARDS_OF_THIS_HOST);
    let (status, output, error) = case.run_lease(None, &["list"]);
    assert_eq!(status.code(), Some(0), "lease list 退 0：{error}");
    assert_eq!(
        output,
        format!(
            "{CARD_LEASE_HELD_LINE_PREFIX} host={THIS_HOST} held_by=nobody holder=none cards=\"{CARDS_OF_THIS_HOST}\" lease_left_milliseconds=0\n\
             {CARD_LEASE_HELD_LINE_PREFIX} host={PEER_HOST} held_by=nobody holder=none cards=\"{CARDS_OF_THE_PEER}\" lease_left_milliseconds=0\n"
        ),
        "两台各一行没人持着：{error}"
    );
    assert_eq!(
        (
            case.holder_of(case.keys.request_key()),
            case.holder_of(case.keys.card_key()),
            case.queued_holders()
        ),
        (None, None, Vec::new()),
        "lease list 一把键都不认领"
    );
}

/// 服务持着卡键时 `lease list` 这一台那一行写 `held_by=service` 与服务的身份（照打给人看的写法、带引号）、剩余租期大于 0。
#[test]
fn lease_list_marks_the_card_key_held_by_the_service_as_held_by_the_service() {
    let case = LeaseCase::with_the_cards("list-service", CARD_STRING_WITH_CARDS_OF_THIS_HOST);
    let mut service = case.library.open_client();
    let service_identity = service.client_identity().to_vec();
    assert_eq!(
        claim_a_card_lease_key_as(
            &mut service,
            &service_identity,
            case.keys.card_key(),
            LONG_LEASE_MILLISECONDS
        )
        .expect("认领得了卡键"),
        CardLeaseKeyAnswer::Claimed,
        "服务持着卡键"
    );
    let (status, output, error) = case.run_lease(None, &["list"]);
    assert_eq!(status.code(), Some(0), "lease list 退 0：{error}");
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2, "两台各一行卡键那一行：{output}");
    let (held_by_the_service, lease_left_milliseconds) = split_off_the_lease_left(lines[0]);
    assert_eq!(
        held_by_the_service,
        format!(
            "{CARD_LEASE_HELD_LINE_PREFIX} host={THIS_HOST} held_by=service holder={:?} cards=\"{CARDS_OF_THIS_HOST}\"",
            holder_text(&service_identity)
        )
    );
    assert!(
        lease_left_milliseconds > 0,
        "服务的卡键剩余租期大于 0：{output}"
    );
}

/// 一个借的一方借着、另一个排着时 `lease list`：这一台的卡键那一行写借方持着（`held_by=borrower`、带引号的借着的那一个的身份、剩余租期大于 0），
/// 请求键那一行写同一个身份，排队的行恰好一行、`position=1`、是排着的那一个；第二台照写没人持着。
#[test]
fn lease_list_names_the_borrower_holding_the_cards_and_the_one_queued_first_behind_it() {
    let case = LeaseCase::with_the_cards("list-borrowed", CARD_STRING_WITH_CARDS_OF_THIS_HOST);
    let holding_process = case.listed_borrowing_process();
    let queued_process = case.listed_borrowing_process();
    let holding_borrower = holding_process.borrower.variable_value();
    let queued_borrower = queued_process.borrower.variable_value();
    let holding_holder = case.holder_identity_of(&holding_process.borrower);
    let queued_holder = case.holder_identity_of(&queued_process.borrower);
    let mut holding_lease = TakenLease {
        case: &case,
        borrower: holding_borrower.clone(),
        is_returned: false,
    };
    let (taken_status, taken_output, taken_error) =
        case.run_lease(Some(&holding_borrower), &["take"]);
    assert_eq!(
        (taken_status.code(), taken_output.as_str()),
        (Some(0), format!("{CARDS_OF_THIS_HOST}\n").as_str()),
        "借着的那一个借到：{taken_error}"
    );
    // 守卫先建、子命令后起：丢掉时先杀排着的那个 lease take，再照它的身份还
    let _queued_lease = TakenLease {
        case: &case,
        borrower: queued_borrower.clone(),
        is_returned: false,
    };
    let _queued_taking = case.start_lease(Some(&queued_borrower), &["take"]);
    wait_until(
        "排着的那一个的等号排上",
        LEASE_COMMAND_WAIT,
        &mut || case.queued_holders() == [queued_holder.clone()],
    );
    let (status, output, error) = case.run_lease(None, &["list"]);
    assert_eq!(status.code(), Some(0), "lease list 退 0：{error}");
    let lines: Vec<(&str, u64)> = output.lines().map(split_off_the_lease_left).collect();
    let holding_text = holder_text(&holding_holder);
    let queued_text = holder_text(&queued_holder);
    assert_eq!(
        lines.iter().map(|(before, _)| *before).collect::<Vec<&str>>(),
        [
            format!(
                "{CARD_LEASE_HELD_LINE_PREFIX} host={THIS_HOST} held_by=borrower holder={holding_text:?} cards=\"{CARDS_OF_THIS_HOST}\""
            )
            .as_str(),
            format!("{CARD_LEASE_REQUESTED_LINE_PREFIX} host={THIS_HOST} holder={holding_text:?}")
                .as_str(),
            format!(
                "{CARD_LEASE_QUEUED_LINE_PREFIX} host={THIS_HOST} position=1 holder={queued_text:?}"
            )
            .as_str(),
            format!(
                "{CARD_LEASE_HELD_LINE_PREFIX} host={PEER_HOST} held_by=nobody holder=none cards=\"{CARDS_OF_THE_PEER}\""
            )
            .as_str(),
        ],
        "卡键、请求键在借着的那一个名下，排着的那一个排第一，第二台没人持着：{output}"
    );
    let lease_left: Vec<u64> = lines.iter().map(|(_, left)| *left).collect();
    assert!(
        lease_left[..3].iter().all(|left| *left > 0) && lease_left[3] == 0,
        "持着的三把剩余租期大于 0，没人持着的写 0：{output}"
    );
    holding_lease.return_now();
}
