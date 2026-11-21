//! 判定库那一份待判池（`src/library_pending_pool.rs`）与取活循环（`src/serving_from_the_library.rs`），对着测试进程里起的临时判定库服务
//! （`prune_server::server::start_in_this_process`，与判定库服务自证同一种临时服务；不写真服务），不上卡：待判行照验证包的写法读写
//! （`VerdictStore` 的表 `1D`、`1E`），验证包那一侧另外三样换成替身（结论字原样存一行，判到了哪一步看那一行），卡与判器换成替身。
//! 每条的弄坏开关（`GPU_VERIFICATION_SERVER_BREAK=<项>`，表在 `src/break_switches.rs` 的 `BreakSwitch::red_test`）打开时那一条必须红。
//! 服务日志那一条：替身判器照真判器的样子往一份共享的行里打逐次派活的行，连同待判池收下的服务行一起过服务的日志过滤（`src/service_log.rs`
//! 的 `filter_the_service_log`，与服务进程里过滤线程同一个函数），核过滤之后的样子；不动这个测试进程的标准错误。
//! 核对待判行那一族（表 `1F`）：扫、重读、核过没有、编核对行那一帧都照验证包那一份，只有核对材料是替身（替身核对器照序号出结论位，
//! `common::stand_in_facts_bits`）；编好那一帧之后别的写方插进来写，由替身验证包一侧照 `common::FactsVerifierFrameInterference` 演。
//! 判表待判行那一族（表 `20`、`21`）：扫、重读、结论字在不在、编结论字那一帧都照验证包那一份，只有流是替身（照判表摘要认登记过的那一条）、
//! 判器是替身；编好那一帧之后别的写方插进来写，由替身验证包一侧照 `common::PackedTablesFrameInterference` 演。

#[path = "../common/mod.rs"]
mod common;

use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::{
    conclusion_version_of_the_crash_image_judgement, facts_verifier_block,
    facts_verifier_pending_names, facts_verifier_pending_row_in_the_library,
    facts_verifier_row_key_of, is_stand_in_conclusion_red, packed_tables_pending,
    packed_tables_pending_names, packed_tables_pending_row_in_the_library,
    packed_tables_verdict_words_in_the_library, pending_block_name, pending_names, pending_row,
    pending_row_in_the_library, put_facts_verifier_pending_rows, put_packed_tables_pending_rows,
    put_pending_rows, stand_in_facts_bits, stand_in_verdicts, store_packed_tables_verdict_words,
    store_stand_in_verdicts, verifier_row_in_the_library, write_a_raw_row,
    FactsVerifierFrameInterference, PackedTablesFrameInterference, StandInVerificationPackageSide,
    TemporaryLibraryService, STAND_IN_RED_WORD,
};
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_memory_ledger::{CardIndex, CardOfTheService};
use gpu_verification_server::card_worker::{CardAfterAPanic, JudgedOrdinals, JudgesOnACard};
use gpu_verification_server::cards_of_the_service::CardsOfTheService;
use gpu_verification_server::facts_verifier_family::{
    FactsVerifierBlockName, FactsVerifierMaterial, FACTS_BLOCK_VERIFIED_LINE_PREFIX,
    FACTS_VERIFIER_BLOCK_CLAIMED_LINE_PREFIX,
    FACTS_VERIFIER_CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX, FACTS_VERIFIER_WRITTEN_LINE_PREFIX,
};
use gpu_verification_server::library_pending_pool::{
    claim_a_facts_verifier_pending_block, claim_a_packed_tables_pending_row, claim_a_pending_block,
    ClaimAnswer, ClaimsReleased, FactsVerifierWrittenBack, LibraryPendingPool, PendingBlockName,
    PreviousProcessClaims, BLOCK_CLAIMED_LINE_PREFIX, CLAIM_HELD_BY_OTHER_LINE_PREFIX,
    CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX, CONCLUSION_WRITTEN_LINE_PREFIX,
};
use gpu_verification_server::packed_tables_family::{
    PackedTablesName, PACKED_TABLES_CLAIMED_LINE_PREFIX,
    PACKED_TABLES_CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX, PACKED_TABLES_JUDGED_LINE_PREFIX,
    PACKED_TABLES_VERDICT_WORDS_WRITTEN_LINE_PREFIX,
};
use gpu_verification_server::pending_pool::StateVerdict;
use gpu_verification_server::service_configuration::ServiceConfiguration;
use gpu_verification_server::service_log::{filter_the_service_log, is_a_dispatch_line};
use gpu_verification_server::service_process::{serving_line, ServiceProcessIdentity};
use gpu_verification_server::serving_from_the_library::{
    serve_from_the_library, ServingSummary, BLOCK_JUDGED_LINE_PREFIX,
    BLOCK_NOT_JUDGEABLE_LINE_PREFIX, CARD_FAILED_LINE_PREFIX, OUT_OF_MEMORY_CAUGHT_LINE_PREFIX,
};
use gpu_verification_server::stream_identity::{StreamFacts, StreamKey};
use singlefs_verification::crash_amplification::CardThatJudgedAPendingBlock;
use singlefs_verification::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
use singlefs_verification::crash_judge_gpu::VERDICT_WORDS;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::sha256::sha256_digest;
use singlefs_verification::verdict_store::{
    FactsVerifierPendingRow, ForcedRejudging, GpuPendingRow, GpuPendingRowPut,
    PackedTablesPendingRow,
};

mod cards_failing_in_the_serving_loop;
mod facts_verifier_family;
mod image_judgement_claims_and_write_back;
mod image_judgement_serving_loop_and_claim_release;
mod packed_tables_family;

/// 判不了那一行里流键打的头几个字节（`src/serving_from_the_library.rs` 的块那一行同一个数）。
const STREAM_KEY_HEAD_BYTES_IN_LINES: usize = 8;
/// 认领的租期：别的用例不等它过期。
const LONG_LEASE_MILLISECONDS: u32 = 60_000;
/// 「写结论那一帧被拒」那一条的租期：等它过期、让别的持有者认领走。
const SHORT_LEASE_MILLISECONDS: u32 = 300;
/// 每条流的态数。
const STATES_OF_A_STREAM: u64 = 8;
/// 等取活循环做完一步的上限（替身判器一块几毫秒，扫库间隔 20 毫秒；10 秒是宽的）。
const STEP_WAIT: Duration = Duration::from_secs(10);
const STEP_POLL: Duration = Duration::from_millis(20);
/// 取活循环的扫库间隔。
const SCAN_INTERVAL: Duration = Duration::from_millis(20);
/// 「池空着卡也持着」那一条池空着等多久再核：十来个扫库间隔。
const IDLE_SPAN: Duration = Duration::from_millis(300);

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

/// 同 [`stand_in_verdicts_of`]，头一个态记红。
fn red_stand_in_verdicts_of(tag: u32, ordinals: Range<u64>) -> Vec<StateVerdict> {
    let mut verdicts = stand_in_verdicts_of(tag, ordinals);
    verdicts[0][STAND_IN_RED_WORD] = 1;
    verdicts
}

/// 第 `tag` 条流：判表摘要每个字节都是 `tag`。
fn tables_digest_of(tag: u8) -> [u8; 32] {
    [tag; 32]
}

fn stream_key_of(tag: u8) -> StreamKey {
    StreamKey {
        tables_digest: tables_digest_of(tag),
        kernel_sources_digest: [0; 32],
    }
}

/// 替身验证包一侧，登记了 `tags` 那几条流。
fn stand_in_side_with(
    library: &TemporaryLibraryService,
    tags: &[u8],
) -> StandInVerificationPackageSide<StandInMaterials> {
    let mut side = StandInVerificationPackageSide::on(library);
    for tag in tags {
        side.register(
            tables_digest_of(*tag),
            stream_key_of(*tag),
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

fn pool_on(
    library: &TemporaryLibraryService,
    side: StandInVerificationPackageSide<StandInMaterials>,
    claim_lease_milliseconds: u32,
) -> LibraryPendingPool<StandInVerificationPackageSide<StandInMaterials>> {
    LibraryPendingPool::new(
        side,
        library.open_client(),
        claim_lease_milliseconds,
        BreakSwitches::of_the_environment(),
    )
}

/// 第 `tag` 条流上的一块：名字与一行不强制的待判行。
fn block_of(
    tag: u8,
    block_number: u64,
    ordinals: Range<u64>,
    conclusion_version: [u8; 32],
    forced: ForcedRejudging,
    pool_entry_number: u64,
) -> (PendingBlockName, GpuPendingRow) {
    (
        pending_block_name(tag, block_number),
        pending_row(
            tables_digest_of(tag),
            ordinals,
            conclusion_version,
            forced,
            pool_entry_number,
        ),
    )
}

/// 替身判这一块的那张卡。
fn stand_in_card() -> CardThatJudgedAPendingBlock {
    CardThatJudgedAPendingBlock {
        card_index: 0,
        card_name: "stand-in card".to_string(),
        card_driver_version: "stand-in driver".to_string(),
    }
}

/// 替身判器打的逐次派活的行（真判器打到标准错误的那两种，`src/service_log.rs` 的 `DISPATCH_LINE_PREFIXES`）收在这里。
type DispatchLinesOfTheStandInJudges = Arc<Mutex<Vec<String>>>;

/// 替身判器照设定出的毛病：显存不够（设备没丢，丢掉卡上全部判器）或设备丢了。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StandInCardFault {
    OutOfMemory,
    DeviceLost,
}

/// 判第几条流（键是流的标记）的时候出哪一种毛病、还出几回；各卡共用一份。
type StandInCardFaults = Arc<Mutex<BTreeMap<u32, (StandInCardFault, usize)>>>;

/// 替身卡的 PCI 总线号。
const STAND_IN_PCI_BUS_IDENTIFIER: &str = "0000:00:00.0";

/// 替身卡：一张，判器是替身（照 `faults` 出毛病）；放掉（丢掉）时记一笔。
struct StandInCards {
    let_go: Arc<AtomicU64>,
    dispatch_lines: DispatchLinesOfTheStandInJudges,
    faults: StandInCardFaults,
}

impl Drop for StandInCards {
    fn drop(&mut self) {
        self.let_go.fetch_add(1, Ordering::SeqCst);
    }
}

struct StandInJudges {
    loaded: BTreeMap<StreamKey, u32>,
    dispatch_lines: DispatchLinesOfTheStandInJudges,
    faults: StandInCardFaults,
    /// 上一回 panic 是照设定出的哪一种毛病（`after_a_panic` 照它答）。
    fault_of_the_panic: Option<StandInCardFault>,
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

    /// 一段一次派活；照真判器的样子打一行 `GPU_LIFECYCLE`、一行 `CRASH_JUDGE_ACCOUNTING_SLICE`、一行 `CRASH_JUDGE_ACCOUNTING_SLICES`。
    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> JudgedOrdinals {
        let tag = *self
            .loaded
            .get(&stream)
            .unwrap_or_else(|| panic!("要判的 {stream:?} 不在卡上"));
        let fault = {
            let mut faults = self.faults.lock().expect("没有线程拿着锁 panic");
            faults
                .get_mut(&tag)
                .filter(|(_, times)| *times > 0)
                .map(|(fault, times)| {
                    *times -= 1;
                    *fault
                })
        };
        if let Some(fault) = fault {
            self.fault_of_the_panic = Some(fault);
            panic!("替身照设定在判第 {tag} 条流时出毛病：{fault:?}");
        }
        let states = ordinals.end - ordinals.start;
        self.dispatch_lines
            .lock()
            .expect("没有线程拿着锁 panic")
            .extend([
                format!("GPU_LIFECYCLE phase=dispatch_readback_end card=0 states={ordinals:?}"),
                format!(
                    "CRASH_JUDGE_ACCOUNTING_SLICE card=\"stand-in card\" first_state={} states={states} slice=0",
                    ordinals.start
                ),
                format!(
                    "CRASH_JUDGE_ACCOUNTING_SLICES card=\"stand-in card\" first_state={} states={states}",
                    ordinals.start
                ),
            ]);
        JudgedOrdinals {
            verdicts: stand_in_verdicts_of(tag, ordinals),
            dispatches: 1,
        }
    }

    /// 结论位照序号出（`common::stand_in_facts_bits`），不打逐次派活的行（真核对器也不打）。
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

    /// 照设定出的毛病答：显存不够的先丢掉全部判器。
    fn after_a_panic(&mut self) -> CardAfterAPanic {
        match self.fault_of_the_panic.take() {
            Some(StandInCardFault::OutOfMemory) => {
                let judges_dropped = self.loaded.len();
                self.loaded.clear();
                CardAfterAPanic::RanOutOfMemory { judges_dropped }
            }
            Some(StandInCardFault::DeviceLost) => CardAfterAPanic::DeviceLost {
                reason: "替身照设定丢了设备".to_string(),
            },
            None => {
                let judges_dropped = self.loaded.len();
                self.loaded.clear();
                CardAfterAPanic::PanickedOtherwise { judges_dropped }
            }
        }
    }

    fn pci_bus_identifier(&self) -> Option<String> {
        Some(STAND_IN_PCI_BUS_IDENTIFIER.to_string())
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
            dispatch_lines: Arc::clone(&self.dispatch_lines),
            faults: Arc::clone(&self.faults),
            fault_of_the_panic: None,
        }
    }

    fn card_that_judged(&self, _card: CardIndex) -> CardThatJudgedAPendingBlock {
        stand_in_card()
    }
}

fn wait_until(what: &str, condition: &mut dyn FnMut() -> bool) {
    let deadline = Instant::now() + STEP_WAIT;
    while Instant::now() < deadline {
        if condition() {
            return;
        }
        std::thread::sleep(STEP_POLL);
    }
    panic!("{} 秒里没等到：{what}", STEP_WAIT.as_secs());
}

/// 测试那一侧结束（含 panic）时叫取活循环停。
struct StopOnDrop(Arc<AtomicBool>);

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

/// 一行里 `键=值` 那几项的键，按出现的先后（行首不算）。
fn field_names_of(line: &str) -> Vec<String> {
    line.split_whitespace()
        .skip(1)
        .filter_map(|word| word.split_once('='))
        .map(|(key, _)| key.to_string())
        .collect()
}
