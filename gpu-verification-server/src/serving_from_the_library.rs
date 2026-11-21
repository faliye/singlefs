//! 服务进程的取活循环：起来就开服务持着的那一组卡、起一段卡上的会话（一条线程跑 [`serve_the_cards`]，块进内存里的待判池，批次调度照旧），
//! 一直持着到停；从判定库那一份待判池取活、交服务主循环判、判完写回。卡不经统一任务服务领、不还（卡归这个服务管）。
//!
//! 一圈：收主循环判完的块逐块写回（[`LibraryPendingPool::write_back`]；被撤的块放回库里等下一轮）；在判的块到点续认领；扫库挑能取的块
//! （[`LibraryPendingPool::candidates`]），认领、登记流、入池。有块在判时每圈隔 [`POLL_WHILE_JUDGING`]，没有时隔 `library_scan_interval`。
//! 核对待判行那一族同一圈里另走一路：在核的块到点续认领；扫核对待判行挑能取的块（`LibraryPendingPool::facts_verifier_candidates`），
//! 认领、按组读回材料，一组照材料与组里最大的那一块算核对器要借的显存（`crate::card_memory_ledger::mebibytes_of_a_facts_verifier_under`）、
//! 交服务主循环在一张卡上核；核完的组逐块写回（`LibraryPendingPool::write_back_the_facts_verifier_block`），被拒的组逐块放回库里等下一轮。
//! 判表待判行那一族同一圈里再走一路：在判的判表到点续认领；扫判表待判行挑能取的（`LibraryPendingPool::packed_tables_candidates`），
//! 认领、配上流（照行读回判表、照旋钮栏取源码），登记流、把 `0..态数` 当一整块入池，走同一条判器路（`register_stream` / `submit_block`），
//! 显存照判器记账；判完的写回（`LibraryPendingPool::write_back_the_packed_tables`），被拒的放回库里等下一轮。
//! `should_stop` 为真时收尾：会话判完已入池的块与判表、核完已交的组、逐块写回、放掉卡。
//!
//! 登记的临时库（`crate::registered_libraries` 文件头；[`serve_from_the_libraries`] 给了登记的设置才有）：一圈里先续在判的块的认领（各份库各续各的），
//! 之后（不在借出期间）列真库里的登记、丢掉没了的、打开新的，各份库扫一遍挑能取的块，认领之后三族各合起来按待判行的入池序号升序入池
//! （先来先判，真库一直有活时登记的库的活照样排得到）；入了池的块记着它所在的库，判完写回那一份，那一份库已经丢掉的不写回
//! （[`TEMPORARY_LIBRARY_BLOCK_DISCARDED_LINE_PREFIX`]）。借卡只经真库那一份待判池；借出期间哪一份库都不扫、不认领。
//! 登记的库读写失败只丢那一份，真库读写失败照旧交回 [`ServingFailure::Library`]。
//!
//! 开卡没成（起来那一回）整个循环交回 [`ServingFailure::CardsNotOpened`]；读写判定库失败（连不上、忙到上限、服务停了）交回
//! [`ServingFailure::Library`]，调用方照判定库客户端的退出码退出。一块被主循环拒（卡坏了）只打一行、照转。
//! 判不了：主循环拒的理由是「在腾空的卡上也显存不够」或「判法自己 panic 了」（`crate::service_loop` 文件头「接住 Out of Memory」「判法自己 panic」）
//! 的块、判表与核对块，这一台判不了：当场以自己的持有者放掉它的认领（`LibraryPendingPool::release_the_claim_left_to_another_service` 与另两族的，
//! 另一台的服务随即取得到；弄坏开关 block-not-judgeable-claim-kept 下不放、留到租期过了），打一行判不了（[`BLOCK_NOT_JUDGEABLE_LINE_PREFIX`]，
//! 弄坏开关 block-not-judgeable-line-left-out 下不打），记下名字，这个进程此后不再认领它（待判行留在库里；弄坏开关 block-refused-for-memory-claimed-again
//! 下照认领）。不退回 CPU 判。`control check` 读到这个进程打的判不了那一行就退 1，任务服务照 `control cards-failed` 转来的这种行给交任务的会话发告警。
//!
//! 日志：验证包逐次派活的行不进服务日志（`crate::service_log`），服务按块、按卡打汇总行，都经待判池的
//! `print_a_service_line`：每判完一块一行（[`BLOCK_JUDGED_LINE_PREFIX`]：流键头、块号、态数、哪几张卡、派活几次、卡上用时，
//! 账由工作线程量，`crate::card_worker` 文件头「判的账」）；每判完一张判表一行（`PACKED_TABLES_JUDGED_LINE_PREFIX`：判表摘要头、态数，
//! 其余字段与判完一块那一行同名）；每核完一块核对块一行（`FACTS_BLOCK_VERIFIED_LINE_PREFIX`，字段与判完一块那一行同名：
//! 流键头、块号、态数、哪张卡、派活几次（核对器一块一次派活）、卡上用时）；每隔 [`CARD_SUMMARY_PERIOD`] 每张卡一行（[`CARD_PERIOD_LINE_PREFIX`]：
//! 这一段判了几块几态（一张判表在主循环里是一整块，记一块）、核了几块几态、卡上用时（判与核合起来）占这一段的千分数，一块在几张卡上判的各卡各记一块），
//! 再一行这一段滤掉了几行逐次派活的行；停的时候把没满的那一段也打出来。
//! 主循环交出的几件事（`crate::service_loop::ServiceNotice`）每一圈先收、各打一行（[`service_notice_line`]）：每装上一个判器一行
//! （[`JUDGE_LOADED_LINE_PREFIX`]）；接住 Out of Memory 一行（[`OUT_OF_MEMORY_CAUGHT_LINE_PREFIX`]，不是卡坏）；一张卡坏了一行
//! （[`CARD_FAILED_LINE_PREFIX`]：卡、`nvidia-smi` 的序号、卡名、PCI 总线号、设备丢没丢、原因；`control check` 读到就退 1，任务服务照它发告警）。
//! 判不了的块每块一行（[`block_not_judgeable_line`]：族、流键头、块号、卡、为什么、认领放没放、原因），见上面「判不了」。
//! 会话收尾时主循环交回的坏了的卡与接住几回 Out of Memory 记进循环的账（停的时候 `GPU_VERIFICATION_SERVER_STOPPED` 那一行打它）。
//! 认领、别的持有者持着、写回成了、放认领那几行由待判池打（`crate::library_pending_pool` 文件头「日志」）。
//!
//! 借卡（`crate::card_lease` 文件头；待判池给了这一台的借卡键才做，[`LibraryPendingPool::lend_the_cards_of_this_host`]）：
//! 卡开好、头一圈之前以待判池的持有者认领卡键：认领到就从「持着」起步，别人持着（借的一方还没还，或它没了而租期没到）就从「借出」起步
//! （卡照开着，打一行 [`crate::card_lease::CARDS_LENT_LINE_PREFIX`]，`blocks_finished_first=0`）。之后每一圈在认领候选之前：
//! 1. 探请求键：以待判池的持有者、[`crate::card_lease::PROBE_LEASE_MILLISECONDS`] 毫秒的租期认领一次请求键；认领到是没人要借
//!    （那 1 毫秒过了自己失效），别人持着是有人要借。至多每个扫库间隔探一次：有块在判时一圈只隔 [`POLL_WHILE_JUDGING`]，
//!    每圈都探就是每秒几十帧带同步的认领；隔着的那几圈照上一次探到的办，开始不认领晚至多一个扫库间隔，借的一方本来每两秒才试一次。
//! 2. 有人要借：这一圈起三族候选都不扫、不认领；手上入了池的块与判表照常判完、写回；都写回了（在判、在核的都空了）放掉卡键（放认领帧），
//!    打一行 [`crate::card_lease::CARDS_LENT_LINE_PREFIX`]，`blocks_finished_first` 是头一次探到有人要借起到放卡键之间写回成了的块数
//!    （三族合起来，一张判表算一块）。借出期间每一圈照样探请求键、不认领候选；每分钟卡账那一行照打（块数是 0），字段不改。
//! 3. 没人要借而卡键不在自己手里（只在探了的那一圈）：认领卡键；认领到打一行 [`crate::card_lease::CARDS_TAKEN_BACK_LINE_PREFIX`]
//!    （借出了多少秒），这一圈起照常认领候选；别人持着就再等一圈。
//! 4. 借出期间设备不放：会话与开好的卡照旧持着，只是不派活；借的一方开不开得起同一张卡、显存够不够是借卡头一次跑要量的，
//!    量到不行再加放设备、收回时重开（循环里放卡再开的那条路此刻只在弄坏开关下走到）。
//!
//! 持着卡键时每过三分之一租期续一次；续不上（停得太久、租期过了被借的一方认领走）当借出了：打借出那一行，这一圈起不认领候选，
//! 手上已在判的照判完写回。停的时候卡键随待判池放认领一起放（`LibraryPendingPool::release_the_claims_of_this_process`）。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use singlefs_verification::crash_amplification::CardThatJudgedAPendingBlock;
use singlefs_verification::hexadecimal::hexadecimal_text;

use crate::batch_scheduling::BatchSchedulingConfiguration;
use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::card_lease::{
    cards_lent_line, cards_taken_back_line, holder_text, CardLeaseKeyAnswer, CardLeaseKeys,
    PROBE_LEASE_MILLISECONDS,
};
use crate::card_memory_ledger::{mebibytes_of_a_facts_verifier_under, CardIndex};
use crate::cards_of_the_service::CardsOfTheService;
use crate::facts_verifier_family::{FactsVerifierBlockName, FACTS_BLOCK_VERIFIED_LINE_PREFIX};
use crate::library_pending_pool::{
    ClaimsReleased, FactsVerifierJobOfTheLibrary, LibraryFailure, LibraryPendingPool,
    PendingBlockName, TakenBlock, TakenPackedTables, VerificationPackageSide,
};
use crate::packed_tables_family::{PackedTablesName, PACKED_TABLES_JUDGED_LINE_PREFIX};
use crate::pending_pool::{InMemoryPendingPool, JudgedBlock};
use crate::registered_libraries::{
    CandidatesOfEveryLibrary, LibrariesOfTheService, LibrariesScanned, LibraryOfTheService,
    RegisteredLibrariesSettings, WriteBackOutcome, TEMPORARY_LIBRARY_BLOCK_DISCARDED_LINE_PREFIX,
};
use crate::service_configuration::ServiceConfiguration;
use crate::service_log::DISPATCH_LINES_DROPPED;
use crate::service_loop::{
    serve_the_cards, service_channel_telling_the_notices, CardWithItsJudges, FactsVerifierJob,
    ServiceClient, ServiceNotice, ServiceRefusal, ServiceRunSummary, VerifiedFactsJob,
};
use crate::stream_identity::StreamKey;

/// 有块在判时一圈隔多久（收判完的块不等满一个扫库间隔）。
pub const POLL_WHILE_JUDGING: Duration = Duration::from_millis(20);
/// 开了卡的那一行的开头。
pub const CARDS_OPENED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CARDS_OPENED";
/// 一块被主循环拒、续认领没续上的那一行的开头。
pub const SERVING_PROBLEM_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_SERVING_PROBLEM";
/// 每判完一块那一行的开头（文件头「日志」）。
pub const BLOCK_JUDGED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_BLOCK_JUDGED";
/// 每隔一段每张卡那一行的开头（文件头「日志」）。
pub const CARD_PERIOD_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CARD_PERIOD";
/// 一张卡坏了那一行的开头（文件头「日志」；`control` 的 `CARD_FAILED_LINE_PREFIX` 与它逐字相同，任务服务读 `control cards-failed` 打的）。
pub const CARD_FAILED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CARD_FAILED";
/// 判不了那一行的开头（文件头「判不了」；`control` 的 `BLOCK_NOT_JUDGEABLE_LINE_PREFIX` 与它逐字相同，任务服务读 `control cards-failed` 转来的）。
pub const BLOCK_NOT_JUDGEABLE_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE";
/// 接住 Out of Memory 那一行的开头（文件头「日志」）。
pub const OUT_OF_MEMORY_CAUGHT_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT";
/// 装上一个判器那一行的开头（文件头「日志」）。
pub const JUDGE_LOADED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_JUDGE_LOADED";
/// 每隔一段滤掉了几行逐次派活的行那一行的开头。
pub const DISPATCH_LINES_DROPPED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_DISPATCH_LINES_DROPPED";
/// 按卡汇总隔多久打一回：一条大流判几分钟到几十分钟，一分钟一段看得出各卡忙闲，每分钟只多卡数加一行。
pub const CARD_SUMMARY_PERIOD: Duration = Duration::from_secs(60);
/// 块那一行里流键只打头几个字节（十六进制）：认得出是哪条流就够。
const STREAM_KEY_HEAD_BYTES: usize = 8;
/// 核对器核一块派几次活（验证包 `GpuFactsVerifier::verify_states` 一块一次派活）。
const FACTS_VERIFIER_DISPATCHES_PER_BLOCK: u64 = 1;
const NANOSECONDS_PER_MILLISECOND: u64 = 1_000_000;
/// 忙的占比打成千分数。
const PERMILLE: u128 = 1_000;

/// 一张卡这一段的账。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CardPeriodWork {
    blocks: u64,
    states: u64,
    facts_blocks: u64,
    facts_states: u64,
    card_nanoseconds: u64,
}

/// 判完一块的那一行（文件头「日志」）。
#[must_use]
pub fn block_judged_line(name: &PendingBlockName, judged: &JudgedBlock) -> String {
    let cards: Vec<String> = judged
        .work_on_each_card
        .keys()
        .map(|card| card.0.to_string())
        .collect();
    let dispatches: u64 = judged
        .work_on_each_card
        .values()
        .map(|work| work.dispatches)
        .sum();
    let card_nanoseconds: u64 = judged
        .work_on_each_card
        .values()
        .map(|work| work.card_nanoseconds)
        .sum();
    format!(
        "{BLOCK_JUDGED_LINE_PREFIX} stream={} block={} states={} cards={} dispatches={dispatches} card_milliseconds={}",
        hexadecimal_text(&name.stream.0[..STREAM_KEY_HEAD_BYTES]),
        name.block_number,
        judged.ordinals.end - judged.ordinals.start,
        cards.join(","),
        card_nanoseconds / NANOSECONDS_PER_MILLISECOND
    )
}

/// 核完一块核对块的那一行（文件头「日志」）：字段与 [`block_judged_line`] 同名。
#[must_use]
pub fn facts_block_verified_line(
    name: &FactsVerifierBlockName,
    states: u64,
    card: CardIndex,
    card_nanoseconds: u64,
) -> String {
    format!(
        "{FACTS_BLOCK_VERIFIED_LINE_PREFIX} stream={} block={} states={states} cards={} dispatches={FACTS_VERIFIER_DISPATCHES_PER_BLOCK} card_milliseconds={}",
        hexadecimal_text(&name.stream.0[..STREAM_KEY_HEAD_BYTES]),
        name.block_number,
        card.0,
        card_nanoseconds / NANOSECONDS_PER_MILLISECOND
    )
}

/// 判完一张判表的那一行（文件头「日志」）：判表摘要头，其余字段与 [`block_judged_line`] 同名。
#[must_use]
pub fn packed_tables_judged_line(name: &PackedTablesName, judged: &JudgedBlock) -> String {
    let cards: Vec<String> = judged
        .work_on_each_card
        .keys()
        .map(|card| card.0.to_string())
        .collect();
    let dispatches: u64 = judged
        .work_on_each_card
        .values()
        .map(|work| work.dispatches)
        .sum();
    let card_nanoseconds: u64 = judged
        .work_on_each_card
        .values()
        .map(|work| work.card_nanoseconds)
        .sum();
    format!(
        "{PACKED_TABLES_JUDGED_LINE_PREFIX} tables={} states={} cards={} dispatches={dispatches} card_milliseconds={}",
        hexadecimal_text(&name.packed_judge_tables_digest[..STREAM_KEY_HEAD_BYTES]),
        judged.ordinals.end - judged.ordinals.start,
        cards.join(","),
        card_nanoseconds / NANOSECONDS_PER_MILLISECOND
    )
}

/// 判不了那一行里一块是哪一族的哪一块（文件头「判不了」）：族名、流键头（判表写判表摘要头）、块号（判表写 `tables`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockNotJudgeableHere {
    pub family: &'static str,
    pub stream_head: String,
    pub block: String,
    pub card: CardIndex,
    /// `out-of-memory-on-an-emptied-card` 或 `judge-panicked`。
    pub why: &'static str,
    /// 认领：`released`（放掉了）、`not-held`（放的时候已不在自己手里）、`kept`（没放，留到租期过了）。
    pub claim: &'static str,
    pub reason: String,
}

/// 判不了的那一行（文件头「判不了」）。
#[must_use]
pub fn block_not_judgeable_line(here: &BlockNotJudgeableHere) -> String {
    format!(
        "{BLOCK_NOT_JUDGEABLE_LINE_PREFIX} family={} stream={} block={} card={} why={} claim={} reason=\"{}\"",
        here.family,
        here.stream_head,
        here.block,
        here.card.0,
        here.why,
        here.claim,
        reason_on_one_line(&here.reason)
    )
}

/// 一行里的原因写成一行、不带双引号（panic 的那一句可能带换行）。
fn reason_on_one_line(reason: &str) -> String {
    reason.trim().replace(['\n', '\r'], " ").replace('"', "'")
}

/// 主循环交出的一件事打成的那一行（文件头「日志」）。`described` 是那张卡是什么卡（最近开的那一组卡里的，卡坏那一行写它的序号与卡名；
/// 不知道的写 `unknown`）；`name_of_a_block` 给撤回、报拒的块起名字（流与序号）。
#[must_use]
pub fn service_notice_line(
    notice: &ServiceNotice,
    described: Option<&CardThatJudgedAPendingBlock>,
    name_of_a_block: &dyn Fn(StreamKey, &Range<u64>) -> String,
) -> String {
    match notice {
        ServiceNotice::JudgeLoaded {
            card,
            stream,
            table_mebibytes,
            scratch_mebibytes,
            states_per_dispatch,
            judges_on_the_card,
            free_mebibytes,
        } => format!(
            "{JUDGE_LOADED_LINE_PREFIX} card={} stream={} table_mebibytes={table_mebibytes} scratch_mebibytes={scratch_mebibytes} \
             states_per_dispatch={states_per_dispatch} judges_on_the_card={judges_on_the_card} free_mebibytes={free_mebibytes}",
            card.0,
            hexadecimal_text(&stream.tables_digest[..STREAM_KEY_HEAD_BYTES]),
        ),
        ServiceNotice::OutOfMemoryCaught {
            card,
            judges_dropped,
            blocks_put_back,
            blocks_refused,
            facts_jobs_put_back,
            facts_jobs_refused,
            reason,
        } => {
            let names = |blocks: &[(StreamKey, Range<u64>)]| {
                let named: Vec<String> = blocks
                    .iter()
                    .map(|(stream, ordinals)| name_of_a_block(*stream, ordinals))
                    .collect();
                if named.is_empty() {
                    "-".to_string()
                } else {
                    named.join(",")
                }
            };
            format!(
                "{OUT_OF_MEMORY_CAUGHT_LINE_PREFIX} card={} judges_dropped={judges_dropped} blocks_put_back={} blocks_refused={} \
                 facts_jobs_put_back={facts_jobs_put_back} facts_jobs_refused={facts_jobs_refused} put_back={} refused={} reason=\"{}\"",
                card.0,
                blocks_put_back.len(),
                blocks_refused.len(),
                names(blocks_put_back),
                names(blocks_refused),
                reason_on_one_line(reason)
            )
        }
        ServiceNotice::CardFailed {
            card,
            device_lost,
            pci_bus_identifier,
            reason,
        } => format!(
            "{CARD_FAILED_LINE_PREFIX} card={} nvidia_smi_index={} name=\"{}\" pci={} device_lost={} reason=\"{}\"",
            card.0,
            described.map_or_else(|| "unknown".to_string(), |known| known.card_index.to_string()),
            described.map_or("unknown", |known| known.card_name.as_str()).replace('"', "'"),
            pci_bus_identifier.as_deref().unwrap_or("unknown"),
            if *device_lost { "yes" } else { "no" },
            reason_on_one_line(reason)
        ),
    }
}

/// 续卡键按三分之一租期：租期过一次续不上，还有两次机会（与续待判块的认领同一个节奏）。
const CARD_KEY_RENEWALS_PER_LEASE: u32 = 3;

/// 循环退出时交回的账。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServingSummary {
    /// 开卡开成了几回（起来那一回；弄坏开关之外只有这一回）。
    pub card_openings: u64,
    /// 停之前放掉过几回卡（弄坏开关之外是 0：卡一直持着）。
    pub cards_let_go_before_stopping: u64,
    pub blocks_written_back: u64,
    pub blocks_refused_at_the_write: u64,
    pub blocks_withdrawn: u64,
    /// 判完而可复用、待判行带守卫删掉了的块数。
    pub reusable_rows_deleted: u64,
    pub facts_blocks_written_back: u64,
    pub facts_blocks_refused_at_the_write: u64,
    pub facts_blocks_withdrawn: u64,
    /// 已核过、核对待判行带守卫删掉了的块数。
    pub verified_facts_rows_deleted: u64,
    pub packed_tables_written_back: u64,
    pub packed_tables_refused_at_the_write: u64,
    pub packed_tables_withdrawn: u64,
    /// 结论字已在表 `21`、判表待判行带守卫删掉了的张数。
    pub packed_tables_rows_deleted: u64,
    /// 卡借出去过几回（起来时就从借出起步的算一回）。
    pub cards_lent: u64,
    /// 卡收回来过几回。
    pub cards_taken_back: u64,
    /// 会话里坏了、不再排的卡（会话收尾时主循环交回的；弄坏开关 cards-failed-left-out-of-the-summary 下不记）。
    pub cards_failed: Vec<CardIndex>,
    /// 会话里接住了几回 Out of Memory。
    pub out_of_memory_caught: u64,
    /// 在腾空的卡上也显存不够、被主循环拒的块（三族合起来，一张判表算一块、一组核对块按块数）。
    pub blocks_refused_for_running_out_of_memory: u64,
    /// 判法自己 panic、被主循环拒的块（同上的算法）。
    pub blocks_refused_for_a_judge_panic: u64,
    /// 打开过几份登记的临时库（`crate::registered_libraries`）。
    pub registered_libraries_opened: u64,
    /// 丢掉过几份登记的临时库（登记没了、读写失败、停的时候）。
    pub registered_libraries_dropped: u64,
    /// 判完而它所在的登记的库已经丢掉、不写回的块数（三族合起来）。
    pub blocks_discarded_with_their_library: u64,
}

/// 循环没跑下去的原因。
#[derive(Debug)]
pub enum ServingFailure {
    /// 起来那一回开卡没成：服务没有卡可判，不起。
    CardsNotOpened(String),
    /// 读写判定库失败。
    Library(LibraryFailure),
}

impl From<LibraryFailure> for ServingFailure {
    fn from(failure: LibraryFailure) -> Self {
        Self::Library(failure)
    }
}

/// 卡键此刻在谁手里（文件头「借卡」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardKeyState {
    /// 服务持着：上一次认领或续上在 `renewed_at`。
    HeldByTheService { renewed_at: Instant },
    /// 借出去了（放掉了、起来时别人持着、续不上）：从 `since` 起。
    Lent { since: Instant },
}

/// 探请求键探到的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LeaseRequest {
    Requested,
    NotRequested,
}

/// 这一圈认不认领候选。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidatesThisRound {
    Claimed,
    LeftAlone,
}

/// 借卡那一半在取活循环里的状态。
struct CardLending {
    keys: CardLeaseKeys,
    state: CardKeyState,
    /// 上一次探请求键的时刻与探到的。
    last_probe: Option<(Instant, LeaseRequest)>,
    /// 头一次探到有人要借时已写回成了的块数（三族合起来）；没人要借、已借出时是 `None`。
    written_back_when_requested: Option<u64>,
}

/// 入了池、在等主循环交回的一块。
struct BlockInFlight {
    library: LibraryOfTheService,
    name: PendingBlockName,
    stream: StreamKey,
    ordinals: Range<u64>,
    answer: Receiver<Result<JudgedBlock, ServiceRefusal>>,
}

/// 入了池、在等主循环交回的一张判表（`0..态数` 一整块）。
struct PackedTablesInFlight {
    library: LibraryOfTheService,
    name: PackedTablesName,
    stream: StreamKey,
    ordinals: Range<u64>,
    answer: Receiver<Result<JudgedBlock, ServiceRefusal>>,
}

/// 交了主循环、在等它交回的一组核对块：每块的名字与态序号区间（与交的次序相同）。
struct FactsVerifierJobInFlight {
    library: LibraryOfTheService,
    blocks: Vec<(FactsVerifierBlockName, Range<u64>)>,
    answer: Receiver<Result<VerifiedFactsJob, ServiceRefusal>>,
}

/// 持着卡的一段会话：主循环的客户端、跑主循环的线程（退出时交回卡）、登记过的流。
struct CardSession<Cards: CardsOfTheService> {
    client: ServiceClient<Cards::Materials>,
    thread: JoinHandle<(Cards, ServiceRunSummary)>,
    registered: BTreeSet<StreamKey>,
    /// 主循环交出的几件事（文件头「日志」）。
    notices: Receiver<ServiceNotice>,
}

fn start_the_session<Cards: CardsOfTheService>(
    cards: Cards,
    configuration: BatchSchedulingConfiguration,
) -> CardSession<Cards> {
    let (client, inbox, notices) = service_channel_telling_the_notices::<Cards::Materials>();
    let thread = std::thread::spawn(move || {
        let summary = {
            let cards_ref = &cards;
            let cards_with_judges = cards_ref
                .cards_of_the_service()
                .into_iter()
                .map(|card| {
                    let index = card.card;
                    CardWithItsJudges {
                        card,
                        make_the_judges: move || cards_ref.judges_on(index),
                    }
                })
                .collect();
            let pool = InMemoryPendingPool::empty(configuration.break_switches.clone());
            serve_the_cards(&configuration, cards_with_judges, pool, inbox, &mut |_| {})
        };
        (cards, summary)
    });
    CardSession {
        client,
        thread,
        registered: BTreeSet::new(),
        notices,
    }
}

/// 取活循环的状态。
struct Serving<'pool, 'opener, Side, Cards: CardsOfTheService> {
    configuration: &'pool ServiceConfiguration,
    break_switches: &'pool BreakSwitches,
    pool: LibrariesOfTheService<'pool, Side>,
    open_the_cards: &'opener mut dyn FnMut() -> Result<Cards, String>,
    session: Option<CardSession<Cards>>,
    /// 最近开的那一组卡每张是什么卡（会话收掉之后写回它交回的块还要用）。
    cards_that_judge: BTreeMap<CardIndex, CardThatJudgedAPendingBlock>,
    in_flight: Vec<BlockInFlight>,
    facts_in_flight: Vec<FactsVerifierJobInFlight>,
    packed_tables_in_flight: Vec<PackedTablesInFlight>,
    summary: ServingSummary,
    /// 按卡汇总的这一段从什么时候起、各卡这一段的账。
    period_started: Instant,
    period_work: BTreeMap<CardIndex, CardPeriodWork>,
    /// 借卡那一半（待判池给了借卡键才有）。
    lending: Option<CardLending>,
    /// 这一台判不了的块、判表与核对块：这个进程不再认领（文件头「判不了」）。
    /// 块名只在一份库里认得：几份库里的同名块各记各的，所以连库一起记。
    refused_for_running_out_of_memory: BTreeSet<(LibraryOfTheService, PendingBlockName)>,
    packed_tables_refused_for_running_out_of_memory:
        BTreeSet<(LibraryOfTheService, PackedTablesName)>,
    facts_blocks_refused_for_running_out_of_memory:
        BTreeSet<(LibraryOfTheService, FactsVerifierBlockName)>,
}

/// 跑取活循环（只有真库那一份）：先开卡（开不成交回 [`ServingFailure::CardsNotOpened`]），待判池给了借卡键的再认领卡键（文件头「借卡」），
/// `should_stop` 为真时收尾返回。
///
/// # Errors
/// 起来那一回开卡没成；读写判定库失败。
pub fn serve_from_the_library<Side, Cards>(
    configuration: &ServiceConfiguration,
    break_switches: &BreakSwitches,
    pool: &mut LibraryPendingPool<Side>,
    open_the_cards: &mut dyn FnMut() -> Result<Cards, String>,
    should_stop: &dyn Fn() -> bool,
) -> Result<ServingSummary, ServingFailure>
where
    Side: VerificationPackageSide,
    Cards: CardsOfTheService<Materials = Side::Materials>,
{
    serve_from_the_libraries(
        configuration,
        break_switches,
        pool,
        None,
        open_the_cards,
        should_stop,
    )
}

/// 跑取活循环：同 [`serve_from_the_library`]，`registered_libraries` 给了的，每一圈另列真库里的登记、各开一份登记的临时库取活
/// （`crate::registered_libraries` 文件头）；各份库按入池序号先来先判，登记的库读写失败只丢那一份。
///
/// # Errors
/// 起来那一回开卡没成；读写真库失败。
pub fn serve_from_the_libraries<'libraries, Side, Cards>(
    configuration: &'libraries ServiceConfiguration,
    break_switches: &'libraries BreakSwitches,
    pool: &'libraries mut LibraryPendingPool<Side>,
    registered_libraries: Option<RegisteredLibrariesSettings<'libraries, Side>>,
    open_the_cards: &mut dyn FnMut() -> Result<Cards, String>,
    should_stop: &dyn Fn() -> bool,
) -> Result<ServingSummary, ServingFailure>
where
    Side: VerificationPackageSide,
    Cards: CardsOfTheService<Materials = Side::Materials>,
{
    let mut serving = Serving {
        configuration,
        break_switches,
        pool: LibrariesOfTheService::new(
            pool,
            registered_libraries,
            break_switches,
            configuration.library_scan_interval,
        ),
        open_the_cards,
        session: None,
        cards_that_judge: BTreeMap::new(),
        in_flight: Vec::new(),
        facts_in_flight: Vec::new(),
        packed_tables_in_flight: Vec::new(),
        summary: ServingSummary::default(),
        period_started: Instant::now(),
        period_work: BTreeMap::new(),
        lending: None,
        refused_for_running_out_of_memory: BTreeSet::new(),
        packed_tables_refused_for_running_out_of_memory: BTreeSet::new(),
        facts_blocks_refused_for_running_out_of_memory: BTreeSet::new(),
    };
    if !break_switches.is_on(BreakSwitch::CardsOpenedOnlyWithWork) {
        serving
            .open_the_session()
            .map_err(ServingFailure::CardsNotOpened)?;
    }
    let outcome = serving
        .start_lending_the_cards()
        .and_then(|()| serving.serve_until(should_stop));
    serving.end_the_session();
    serving.print_the_card_period();
    serving.pool.release_the_registered_libraries();
    serving.summary.reusable_rows_deleted = serving.pool.reusable_rows_deleted();
    serving.summary.verified_facts_rows_deleted = serving.pool.verified_facts_rows_deleted();
    serving.summary.packed_tables_rows_deleted = serving.pool.packed_tables_rows_deleted();
    (
        serving.summary.registered_libraries_opened,
        serving.summary.registered_libraries_dropped,
    ) = serving.pool.registered_libraries_opened_and_dropped();
    outcome.map(|()| serving.summary)
}

impl<Side, Cards> Serving<'_, '_, Side, Cards>
where
    Side: VerificationPackageSide,
    Cards: CardsOfTheService<Materials = Side::Materials>,
{
    fn scheduling_configuration(&self) -> BatchSchedulingConfiguration {
        BatchSchedulingConfiguration {
            oldest_pending_wait_limit: self.configuration.oldest_pending_wait_limit,
            break_switches: self.break_switches.clone(),
        }
    }

    fn serve_until(&mut self, should_stop: &dyn Fn() -> bool) -> Result<(), ServingFailure> {
        // 迭代上界：不设，服务进程一直取活，直到 should_stop（测试）或进程被停
        while !should_stop() {
            self.print_the_service_notices();
            self.pool.drop_lapsed_registrations()?;
            self.take_the_judged_blocks();
            if self.period_started.elapsed() >= CARD_SUMMARY_PERIOD {
                self.print_the_card_period();
            }
            self.pool.renew_the_claims()?;
            let candidates = match self.lend_or_take_back_the_cards()? {
                CandidatesThisRound::Claimed => {
                    self.pool.follow_the_registrations()?;
                    let candidates = self.pool.scan_the_candidates(
                        LibrariesScanned::TheRealLibraryAndTheRegisteredOnes,
                    )?;
                    self.candidates_left_after_running_out_of_memory(candidates)
                }
                CandidatesThisRound::LeftAlone
                    if self
                        .break_switches
                        .is_on(BreakSwitch::TemporaryLibrariesScannedWhileLent) =>
                {
                    self.pool.follow_the_registrations()?;
                    let candidates = self
                        .pool
                        .scan_the_candidates(LibrariesScanned::TheRegisteredOnesOnly)?;
                    self.candidates_left_after_running_out_of_memory(candidates)
                }
                CandidatesThisRound::LeftAlone => CandidatesOfEveryLibrary::none(),
            };
            if !candidates.is_empty() && self.session.is_none() {
                // 只在弄坏开关下走到：起来没开卡，或池空时放掉了卡
                self.open_the_session()
                    .map_err(ServingFailure::CardsNotOpened)?;
            }
            let mut claimed_any = false;
            if !candidates.is_empty() {
                let claimed = self.pool.claim_the_candidates(candidates)?;
                claimed_any = !claimed.is_empty();
                for (library, taken) in claimed.blocks {
                    self.submit(library, taken);
                }
                for (library, job) in claimed.facts_verifier_jobs {
                    self.submit_the_facts_verifier_job(library, job);
                }
                for (library, packed_tables) in claimed.packed_tables {
                    self.submit_the_packed_tables(library, packed_tables);
                }
            }
            let is_idle = !claimed_any && !self.has_blocks_in_hand();
            if is_idle
                && self.session.is_some()
                && self
                    .break_switches
                    .is_on(BreakSwitch::CardsLetGoWhenThePoolIdles)
            {
                self.end_the_session();
                self.summary.cards_let_go_before_stopping += 1;
            }
            std::thread::sleep(
                if self.in_flight.is_empty()
                    && self.facts_in_flight.is_empty()
                    && self.packed_tables_in_flight.is_empty()
                {
                    self.configuration.library_scan_interval
                } else {
                    POLL_WHILE_JUDGING
                },
            );
        }
        Ok(())
    }

    /// 待判池给了借卡键的，头一圈之前认领卡键定起步的那一态（文件头「借卡」）。
    /// 弄坏开关 card-key-held-by-another-at-start-ignored 下别人持着也当自己持着。
    fn start_lending_the_cards(&mut self) -> Result<(), ServingFailure> {
        let Some(keys) = self.pool.card_lease_keys().cloned() else {
            return Ok(());
        };
        let answer = self.pool.claim_a_card_lease_key(
            keys.card_key(),
            self.configuration.claim_lease_milliseconds(),
        )?;
        let state = match answer {
            CardLeaseKeyAnswer::HeldByOther { .. }
                if !self
                    .break_switches
                    .is_on(BreakSwitch::CardKeyHeldByAnotherAtStartIgnored) =>
            {
                self.summary.cards_lent += 1;
                self.pool.print_a_service_line(cards_lent_line(&keys, 0));
                CardKeyState::Lent {
                    since: Instant::now(),
                }
            }
            CardLeaseKeyAnswer::Claimed | CardLeaseKeyAnswer::HeldByOther { .. } => {
                CardKeyState::HeldByTheService {
                    renewed_at: Instant::now(),
                }
            }
        };
        self.lending = Some(CardLending {
            keys,
            state,
            last_probe: None,
            written_back_when_requested: None,
        });
        Ok(())
    }

    /// 探请求键（至多每个扫库间隔一次，文件头「借卡」第 1 步）：交回这一圈照什么办、这一圈探了没有。
    /// 弄坏开关 card-lease-request-ignored 下不探、一律当没人要借。
    fn probe_the_lease_request(
        &mut self,
        lending: &mut CardLending,
    ) -> Result<(LeaseRequest, bool), ServingFailure> {
        if let Some((probed_at, request)) = lending.last_probe {
            if probed_at.elapsed() < self.configuration.library_scan_interval {
                return Ok((request, false));
            }
        }
        let request = if self
            .break_switches
            .is_on(BreakSwitch::CardLeaseRequestIgnored)
        {
            LeaseRequest::NotRequested
        } else {
            match self
                .pool
                .claim_a_card_lease_key(lending.keys.request_key(), PROBE_LEASE_MILLISECONDS)?
            {
                CardLeaseKeyAnswer::Claimed => LeaseRequest::NotRequested,
                CardLeaseKeyAnswer::HeldByOther { .. } => LeaseRequest::Requested,
            }
        };
        lending.last_probe = Some((Instant::now(), request));
        Ok((request, true))
    }

    /// 持着卡键、过了三分之一租期的续一次；续不上当借出了（文件头「借卡」）。
    fn renew_the_card_key_when_due(
        &mut self,
        lending: &mut CardLending,
    ) -> Result<(), ServingFailure> {
        let CardKeyState::HeldByTheService { renewed_at } = lending.state else {
            return Ok(());
        };
        let lease_milliseconds = self.configuration.claim_lease_milliseconds();
        let renew_after =
            Duration::from_millis(u64::from(lease_milliseconds / CARD_KEY_RENEWALS_PER_LEASE));
        if renewed_at.elapsed() < renew_after {
            return Ok(());
        }
        match self
            .pool
            .claim_a_card_lease_key(lending.keys.card_key(), lease_milliseconds)?
        {
            CardLeaseKeyAnswer::Claimed => {
                lending.state = CardKeyState::HeldByTheService {
                    renewed_at: Instant::now(),
                };
            }
            CardLeaseKeyAnswer::HeldByOther { holder, .. } => {
                eprintln!(
                    "{SERVING_PROBLEM_LINE_PREFIX} 卡键续不上（{} 持着）：当借出了，这一圈起不认领候选，手上在判的照判完写回",
                    holder_text(&holder)
                );
                self.lend(lending, 0);
            }
        }
        Ok(())
    }

    /// 记一回借出、打借出那一行；弄坏开关 devices-let-go-while-lent 下连设备一起放。
    fn lend(&mut self, lending: &mut CardLending, blocks_finished_first: u64) {
        lending.state = CardKeyState::Lent {
            since: Instant::now(),
        };
        lending.written_back_when_requested = None;
        self.summary.cards_lent += 1;
        self.pool
            .print_a_service_line(cards_lent_line(&lending.keys, blocks_finished_first));
        if self
            .break_switches
            .is_on(BreakSwitch::DevicesLetGoWhileLent)
            && self.session.is_some()
        {
            self.end_the_session();
            self.summary.cards_let_go_before_stopping += 1;
        }
    }

    /// 手上还有没写回的块（在判、在核，或认领着；三族）。
    fn has_blocks_in_hand(&self) -> bool {
        !self.in_flight.is_empty()
            || !self.facts_in_flight.is_empty()
            || !self.packed_tables_in_flight.is_empty()
            || self.pool.claimed_count() != 0
            || self.pool.facts_claimed_count() != 0
            || self.pool.packed_tables_claimed_count() != 0
    }

    /// 借卡那一半的一圈（文件头「借卡」第 1 到 3 步）：交回这一圈认不认领候选。没给借卡键的照认领。
    fn lend_or_take_back_the_cards(&mut self) -> Result<CandidatesThisRound, ServingFailure> {
        let Some(mut lending) = self.lending.take() else {
            return Ok(CandidatesThisRound::Claimed);
        };
        let outcome = self.step_the_card_lending(&mut lending);
        self.lending = Some(lending);
        outcome
    }

    fn step_the_card_lending(
        &mut self,
        lending: &mut CardLending,
    ) -> Result<CandidatesThisRound, ServingFailure> {
        let (request, probed_this_round) = self.probe_the_lease_request(lending)?;
        match (lending.state, request) {
            (CardKeyState::HeldByTheService { .. }, LeaseRequest::NotRequested) => {
                lending.written_back_when_requested = None;
                self.renew_the_card_key_when_due(lending)?;
            }
            (CardKeyState::HeldByTheService { .. }, LeaseRequest::Requested) => {
                let written_back_now = self.summary.blocks_written_back
                    + self.summary.facts_blocks_written_back
                    + self.summary.packed_tables_written_back;
                let written_back_when_requested = *lending
                    .written_back_when_requested
                    .get_or_insert(written_back_now);
                if self.has_blocks_in_hand()
                    && !self
                        .break_switches
                        .is_on(BreakSwitch::CardsLentBeforeTheBlocksInHandAreWrittenBack)
                {
                    self.renew_the_card_key_when_due(lending)?;
                } else {
                    self.pool.release_the_card_key()?;
                    self.lend(lending, written_back_now - written_back_when_requested);
                }
            }
            (CardKeyState::Lent { .. }, LeaseRequest::Requested) => {}
            (CardKeyState::Lent { since }, LeaseRequest::NotRequested) => {
                if probed_this_round && !self.break_switches.is_on(BreakSwitch::CardKeyNotTakenBack)
                {
                    let answer = self.pool.claim_a_card_lease_key(
                        lending.keys.card_key(),
                        self.configuration.claim_lease_milliseconds(),
                    )?;
                    if answer == CardLeaseKeyAnswer::Claimed {
                        lending.state = CardKeyState::HeldByTheService {
                            renewed_at: Instant::now(),
                        };
                        self.summary.cards_taken_back += 1;
                        self.pool.print_a_service_line(cards_taken_back_line(
                            &lending.keys,
                            since.elapsed().as_secs(),
                        ));
                    }
                }
            }
        }
        Ok(match lending.state {
            CardKeyState::HeldByTheService { .. } if request == LeaseRequest::NotRequested => {
                CandidatesThisRound::Claimed
            }
            CardKeyState::HeldByTheService { .. } | CardKeyState::Lent { .. } => {
                CandidatesThisRound::LeftAlone
            }
        })
    }

    fn open_the_session(&mut self) -> Result<(), String> {
        let cards = (self.open_the_cards)()?;
        self.cards_that_judge = cards
            .cards_of_the_service()
            .into_iter()
            .map(|card| (card.card, cards.card_that_judged(card.card)))
            .collect();
        eprintln!(
            "{CARDS_OPENED_LINE_PREFIX} cards={}",
            self.cards_that_judge.len()
        );
        self.summary.card_openings += 1;
        self.session = Some(start_the_session(cards, self.scheduling_configuration()));
        Ok(())
    }

    fn submit(&mut self, library: LibraryOfTheService, taken: TakenBlock<Side::Materials>) {
        let session = self.session.as_mut().expect("认领只在持着卡时做");
        let TakenBlock { pending, stream } = taken;
        if !session.registered.contains(&stream.stream) {
            if let Err(refusal) =
                session
                    .client
                    .register_stream(stream.stream, stream.facts, stream.materials)
            {
                eprintln!(
                    "{SERVING_PROBLEM_LINE_PREFIX} {} 的流登记不上：{refusal:?}（留在库里）",
                    pending.name
                );
                self.withdraw_or_release_after_refusal(library, pending.name, &refusal);
                self.summary.blocks_withdrawn += 1;
                return;
            }
            session.registered.insert(stream.stream);
        }
        let ordinals = pending.row.state_ordinals.clone();
        let answer = session.client.submit_block(stream.stream, ordinals.clone());
        self.in_flight.push(BlockInFlight {
            library,
            name: pending.name,
            stream: stream.stream,
            ordinals,
            answer,
        });
    }

    /// 一张判表交主循环（文件头）：流没登记过的先登记，`0..态数` 一整块入池；登记不上的放回库里。
    fn submit_the_packed_tables(
        &mut self,
        library: LibraryOfTheService,
        taken: TakenPackedTables<Side::Materials>,
    ) {
        let session = self.session.as_mut().expect("认领只在持着卡时做");
        let TakenPackedTables { pending, stream } = taken;
        if !session.registered.contains(&stream.stream) {
            if let Err(refusal) =
                session
                    .client
                    .register_stream(stream.stream, stream.facts, stream.materials)
            {
                eprintln!(
                    "{SERVING_PROBLEM_LINE_PREFIX} 判表 {} 的流登记不上：{refusal:?}（留在库里）",
                    pending.name
                );
                self.withdraw_or_release_packed_tables_after_refusal(
                    library,
                    pending.name,
                    &refusal,
                );
                self.summary.packed_tables_withdrawn += 1;
                return;
            }
            session.registered.insert(stream.stream);
        }
        let ordinals = 0..pending.row.state_count;
        let answer = session.client.submit_block(stream.stream, ordinals.clone());
        self.packed_tables_in_flight.push(PackedTablesInFlight {
            library,
            name: pending.name,
            stream: stream.stream,
            ordinals,
            answer,
        });
    }

    /// 一组核对块交主循环（文件头）：核对器要借的显存照这一组的材料与组里最大的那一块算。
    fn submit_the_facts_verifier_job(
        &mut self,
        library: LibraryOfTheService,
        job: FactsVerifierJobOfTheLibrary,
    ) {
        let session = self.session.as_ref().expect("认领只在持着卡时做");
        let FactsVerifierJobOfTheLibrary { blocks, material } = job;
        let states_of_the_largest_block = blocks
            .iter()
            .map(|pending| pending.row.state_ordinals.end - pending.row.state_ordinals.start)
            .max()
            .unwrap_or(0);
        let verifier_mebibytes = mebibytes_of_a_facts_verifier_under(
            material.facts(),
            states_of_the_largest_block,
            self.break_switches,
        );
        let answer = session.client.submit_facts_verifier_job(FactsVerifierJob {
            material,
            blocks: blocks
                .iter()
                .map(|pending| pending.row.state_ordinals.clone())
                .collect(),
            verifier_mebibytes,
        });
        self.facts_in_flight.push(FactsVerifierJobInFlight {
            library,
            blocks: blocks
                .into_iter()
                .map(|pending| (pending.name, pending.row.state_ordinals))
                .collect(),
            answer,
        });
    }

    /// 收主循环交回的一组核对块：核完的逐块打一行、记进这一段各卡的账、写回；被拒的逐块放回库里。
    ///
    /// # Panics
    /// 主循环交回的块数、结论位份数、挂钟份数对不上交的块数。
    fn take_one_facts_verifier_answer(
        &mut self,
        library: LibraryOfTheService,
        blocks: Vec<(FactsVerifierBlockName, Range<u64>)>,
        answer: Result<VerifiedFactsJob, ServiceRefusal>,
    ) {
        match answer {
            Ok(VerifiedFactsJob {
                card,
                bits_of_each_block,
                card_nanoseconds_of_each_block,
            }) => {
                assert!(
                    bits_of_each_block.len() == blocks.len()
                        && card_nanoseconds_of_each_block.len() == blocks.len(),
                    "主循环交回 {} 份结论位、{} 份挂钟，交的是 {} 块",
                    bits_of_each_block.len(),
                    card_nanoseconds_of_each_block.len(),
                    blocks.len()
                );
                for (((name, ordinals), bits), card_nanoseconds) in blocks
                    .into_iter()
                    .zip(bits_of_each_block)
                    .zip(card_nanoseconds_of_each_block)
                {
                    let states = ordinals.end - ordinals.start;
                    self.pool.print_a_service_line(facts_block_verified_line(
                        &name,
                        states,
                        card,
                        card_nanoseconds,
                    ));
                    let period = self.period_work.entry(card).or_default();
                    period.facts_blocks += 1;
                    period.facts_states += states;
                    period.card_nanoseconds += card_nanoseconds;
                    match self
                        .pool
                        .write_back_the_facts_verifier_block(library, &name, bits)
                    {
                        WriteBackOutcome::Written => self.summary.facts_blocks_written_back += 1,
                        WriteBackOutcome::RefusedAtTheWrite => {
                            self.summary.facts_blocks_refused_at_the_write += 1;
                        }
                        WriteBackOutcome::LibraryGone => {
                            self.discard_a_block_of_a_dropped_library(library, &name);
                        }
                    }
                }
            }
            Err(refusal) => {
                let not_judgeable = not_judgeable_here(&refusal);
                for (name, _) in blocks {
                    eprintln!("{SERVING_PROBLEM_LINE_PREFIX} 核对块 {name} 被主循环拒了：{refusal:?}（留在库里，{}）", refusal_after_word(not_judgeable.is_some()));
                    self.summary.facts_blocks_withdrawn += 1;
                    let Some((card, why, reason)) = not_judgeable.clone() else {
                        self.withdraw_or_release_facts_verifier_after_refusal(
                            library, name, &refusal,
                        );
                        continue;
                    };
                    let claim = if self
                        .break_switches
                        .is_on(BreakSwitch::BlockNotJudgeableClaimKept)
                    {
                        self.pool.withdraw_the_facts_verifier_block(library, &name);
                        CLAIM_KEPT
                    } else {
                        claim_left_to_another_service(
                            self.pool
                                .release_the_facts_verifier_claim_left_to_another_service(
                                    library, &name,
                                ),
                            &format!("核对块 {name}"),
                        )
                    };
                    self.tell_the_block_is_not_judgeable_here(&BlockNotJudgeableHere {
                        family: FACTS_VERIFIER_FAMILY_NAME,
                        stream_head: hexadecimal_text(&name.stream.0[..STREAM_KEY_HEAD_BYTES]),
                        block: name.block_number.to_string(),
                        card,
                        why,
                        claim,
                        reason,
                    });
                    if !self
                        .break_switches
                        .is_on(BreakSwitch::BlockRefusedForMemoryClaimedAgain)
                    {
                        self.facts_blocks_refused_for_running_out_of_memory
                            .insert((library, name));
                    }
                }
            }
        }
    }

    /// 收主循环交回的块：判完的写回，被拒的放回库里；核对组与判表同样。
    fn take_the_judged_blocks(&mut self) {
        let mut still_in_flight = Vec::with_capacity(self.in_flight.len());
        for block in std::mem::take(&mut self.in_flight) {
            match block.answer.try_recv() {
                Ok(answer) => {
                    self.take_one_answer(block.library, block.name, &block.ordinals, answer);
                }
                Err(TryRecvError::Empty) => still_in_flight.push(block),
                Err(TryRecvError::Disconnected) => self.take_one_answer(
                    block.library,
                    block.name,
                    &block.ordinals,
                    Err(ServiceRefusal::ServiceStopping),
                ),
            }
        }
        self.in_flight = still_in_flight;
        let mut facts_still_in_flight = Vec::with_capacity(self.facts_in_flight.len());
        for job in std::mem::take(&mut self.facts_in_flight) {
            match job.answer.try_recv() {
                Ok(answer) => self.take_one_facts_verifier_answer(job.library, job.blocks, answer),
                Err(TryRecvError::Empty) => facts_still_in_flight.push(job),
                Err(TryRecvError::Disconnected) => self.take_one_facts_verifier_answer(
                    job.library,
                    job.blocks,
                    Err(ServiceRefusal::ServiceStopping),
                ),
            }
        }
        self.facts_in_flight = facts_still_in_flight;
        let mut packed_tables_still_in_flight =
            Vec::with_capacity(self.packed_tables_in_flight.len());
        for packed_tables in std::mem::take(&mut self.packed_tables_in_flight) {
            match packed_tables.answer.try_recv() {
                Ok(answer) => self.take_one_packed_tables_answer(
                    packed_tables.library,
                    packed_tables.name,
                    &packed_tables.ordinals,
                    answer,
                ),
                Err(TryRecvError::Empty) => packed_tables_still_in_flight.push(packed_tables),
                Err(TryRecvError::Disconnected) => self.take_one_packed_tables_answer(
                    packed_tables.library,
                    packed_tables.name,
                    &packed_tables.ordinals,
                    Err(ServiceRefusal::ServiceStopping),
                ),
            }
        }
        self.packed_tables_in_flight = packed_tables_still_in_flight;
    }

    /// 一块的几截落在几张卡上时，结论块属性里写号最小的那一张（同一台的卡同型同驱动时没有差别）。
    fn card_that_judged(&self, judged: &JudgedBlock) -> CardThatJudgedAPendingBlock {
        let card = judged
            .work_on_each_card
            .keys()
            .next()
            .expect("判完的一块至少有一截在一张卡上判过");
        self.cards_that_judge
            .get(card)
            .cloned()
            .unwrap_or_else(|| panic!("{card:?} 不是最近开的那一组卡里的"))
    }

    /// 判完的一块所在的登记的库已经丢掉：结论不写回，打一行，记一笔。
    fn discard_a_block_of_a_dropped_library(
        &mut self,
        library: LibraryOfTheService,
        name: &dyn std::fmt::Display,
    ) {
        self.summary.blocks_discarded_with_their_library += 1;
        self.pool.print_a_service_line(format!(
            "{TEMPORARY_LIBRARY_BLOCK_DISCARDED_LINE_PREFIX} library={library} {name}"
        ));
    }

    fn take_one_answer(
        &mut self,
        library: LibraryOfTheService,
        name: PendingBlockName,
        ordinals: &Range<u64>,
        answer: Result<JudgedBlock, ServiceRefusal>,
    ) {
        match answer {
            Ok(judged) => {
                assert_eq!(
                    &judged.ordinals, ordinals,
                    "主循环交回的 {name} 不是入池的那一段"
                );
                let card = self.card_that_judged(&judged);
                self.pool
                    .print_a_service_line(block_judged_line(&name, &judged));
                for (judging_card, work) in &judged.work_on_each_card {
                    let period = self.period_work.entry(*judging_card).or_default();
                    period.blocks += 1;
                    period.states += work.states;
                    period.card_nanoseconds += work.card_nanoseconds;
                }
                match self.pool.write_back(library, &name, judged.verdicts, &card) {
                    WriteBackOutcome::Written => self.summary.blocks_written_back += 1,
                    WriteBackOutcome::RefusedAtTheWrite => {
                        self.summary.blocks_refused_at_the_write += 1;
                    }
                    WriteBackOutcome::LibraryGone => {
                        self.discard_a_block_of_a_dropped_library(library, &name);
                    }
                }
            }
            Err(refusal) => {
                let not_judgeable = not_judgeable_here(&refusal);
                eprintln!("{SERVING_PROBLEM_LINE_PREFIX} {name} 被主循环拒了：{refusal:?}（留在库里，{}）", refusal_after_word(not_judgeable.is_some()));
                self.summary.blocks_withdrawn += 1;
                let Some((card, why, reason)) = not_judgeable else {
                    self.withdraw_or_release_after_refusal(library, name, &refusal);
                    return;
                };
                let claim = if self
                    .break_switches
                    .is_on(BreakSwitch::BlockNotJudgeableClaimKept)
                {
                    self.pool.withdraw(library, &name);
                    CLAIM_KEPT
                } else {
                    claim_left_to_another_service(
                        self.pool
                            .release_the_claim_left_to_another_service(library, &name),
                        &name.to_string(),
                    )
                };
                self.tell_the_block_is_not_judgeable_here(&BlockNotJudgeableHere {
                    family: IMAGE_JUDGEMENT_FAMILY_NAME,
                    stream_head: hexadecimal_text(&name.stream.0[..STREAM_KEY_HEAD_BYTES]),
                    block: name.block_number.to_string(),
                    card,
                    why,
                    claim,
                    reason,
                });
                if !self
                    .break_switches
                    .is_on(BreakSwitch::BlockRefusedForMemoryClaimedAgain)
                {
                    self.refused_for_running_out_of_memory
                        .insert((library, name));
                }
            }
        }
    }

    /// 收主循环交回的一张判表：判完的打一行、记进这一段各卡的账（一张判表记一块）、写回；被拒的放回库里。
    fn take_one_packed_tables_answer(
        &mut self,
        library: LibraryOfTheService,
        name: PackedTablesName,
        ordinals: &Range<u64>,
        answer: Result<JudgedBlock, ServiceRefusal>,
    ) {
        match answer {
            Ok(judged) => {
                assert_eq!(
                    &judged.ordinals, ordinals,
                    "主循环交回的判表 {name} 不是入池的那一段"
                );
                let card = self.card_that_judged(&judged);
                self.pool
                    .print_a_service_line(packed_tables_judged_line(&name, &judged));
                for (judging_card, work) in &judged.work_on_each_card {
                    let period = self.period_work.entry(*judging_card).or_default();
                    period.blocks += 1;
                    period.states += work.states;
                    period.card_nanoseconds += work.card_nanoseconds;
                }
                match self
                    .pool
                    .write_back_the_packed_tables(library, &name, judged.verdicts, &card)
                {
                    WriteBackOutcome::Written => self.summary.packed_tables_written_back += 1,
                    WriteBackOutcome::RefusedAtTheWrite => {
                        self.summary.packed_tables_refused_at_the_write += 1;
                    }
                    WriteBackOutcome::LibraryGone => {
                        self.discard_a_block_of_a_dropped_library(library, &name);
                    }
                }
            }
            Err(refusal) => {
                let not_judgeable = not_judgeable_here(&refusal);
                eprintln!("{SERVING_PROBLEM_LINE_PREFIX} 判表 {name} 被主循环拒了：{refusal:?}（留在库里，{}）", refusal_after_word(not_judgeable.is_some()));
                self.summary.packed_tables_withdrawn += 1;
                let Some((card, why, reason)) = not_judgeable else {
                    self.withdraw_or_release_packed_tables_after_refusal(library, name, &refusal);
                    return;
                };
                let claim = if self
                    .break_switches
                    .is_on(BreakSwitch::BlockNotJudgeableClaimKept)
                {
                    self.pool.withdraw_the_packed_tables(library, &name);
                    CLAIM_KEPT
                } else {
                    claim_left_to_another_service(
                        self.pool
                            .release_the_packed_tables_claim_left_to_another_service(
                                library, &name,
                            ),
                        &format!("判表 {name}"),
                    )
                };
                self.tell_the_block_is_not_judgeable_here(&BlockNotJudgeableHere {
                    family: PACKED_TABLES_FAMILY_NAME,
                    stream_head: hexadecimal_text(
                        &name.packed_judge_tables_digest[..STREAM_KEY_HEAD_BYTES],
                    ),
                    block: PACKED_TABLES_BLOCK_WORD.to_string(),
                    card,
                    why,
                    claim,
                    reason,
                });
                if !self
                    .break_switches
                    .is_on(BreakSwitch::BlockRefusedForMemoryClaimedAgain)
                {
                    self.packed_tables_refused_for_running_out_of_memory
                        .insert((library, name));
                }
            }
        }
    }

    /// 收主循环交出的几件事、各打一行（文件头「日志」）。
    fn print_the_service_notices(&mut self) {
        let Some(session) = self.session.take() else {
            return;
        };
        self.print_the_notices_of(&session.notices);
        self.session = Some(session);
    }

    fn print_the_notices_of(&mut self, notices: &Receiver<ServiceNotice>) {
        while let Ok(notice) = notices.try_recv() {
            let card = match &notice {
                ServiceNotice::JudgeLoaded { card, .. }
                | ServiceNotice::OutOfMemoryCaught { card, .. }
                | ServiceNotice::CardFailed { card, .. } => {
                    self.cards_that_judge.get(card).cloned()
                }
            };
            let line = service_notice_line(&notice, card.as_ref(), &|stream, ordinals| {
                self.name_of_a_block_in_flight(stream, ordinals)
            });
            self.pool.print_a_service_line(line);
        }
    }

    /// 撤回、报拒的那一块在行里的名字（一个词）：在 in_flight 里的写 `<流键头>/<块号>`，判表写 `tables-<判表摘要头>`；交回过了、找不到的写
    /// `<判表摘要头>:<序号区间>`。
    fn name_of_a_block_in_flight(&self, stream: StreamKey, ordinals: &Range<u64>) -> String {
        if let Some(block) = self
            .in_flight
            .iter()
            .find(|block| block.stream == stream && &block.ordinals == ordinals)
        {
            return format!(
                "{}/{}",
                hexadecimal_text(&block.name.stream.0[..STREAM_KEY_HEAD_BYTES]),
                block.name.block_number
            );
        }
        if let Some(packed_tables) = self.packed_tables_in_flight.iter().find(|packed_tables| {
            packed_tables.stream == stream && &packed_tables.ordinals == ordinals
        }) {
            return format!(
                "tables-{}",
                hexadecimal_text(
                    &packed_tables.name.packed_judge_tables_digest[..STREAM_KEY_HEAD_BYTES]
                )
            );
        }
        format!(
            "{}:{}..{}",
            hexadecimal_text(&stream.tables_digest[..STREAM_KEY_HEAD_BYTES]),
            ordinals.start,
            ordinals.end
        )
    }

    /// 一块判不了（文件头「判不了」）：记进循环的账、打那一行（弄坏开关 block-not-judgeable-line-left-out 下不打）。
    fn tell_the_block_is_not_judgeable_here(&mut self, here: &BlockNotJudgeableHere) {
        if here.why == WHY_OUT_OF_MEMORY_ON_AN_EMPTIED_CARD {
            self.summary.blocks_refused_for_running_out_of_memory += 1;
        } else {
            self.summary.blocks_refused_for_a_judge_panic += 1;
        }
        if !self
            .break_switches
            .is_on(BreakSwitch::BlockNotJudgeableLineLeftOut)
        {
            self.pool
                .print_a_service_line(block_not_judgeable_line(here));
        }
    }

    /// 主循环拒收一块（非 `BLOCK_NOT_JUDGEABLE` 告警行那两种）：当场放掉认领（避免锁满租期）；
    /// 若是这一台没一张没坏的卡装得下或区间不对，记进这个进程不再认领的集合，让另一台去取。
    fn withdraw_or_release_after_refusal(
        &mut self,
        library: LibraryOfTheService,
        name: PendingBlockName,
        refusal: &ServiceRefusal,
    ) {
        if self
            .break_switches
            .is_on(BreakSwitch::BlockNotJudgeableClaimKept)
        {
            self.pool.withdraw(library, &name);
        } else {
            let _ = self
                .pool
                .release_the_claim_left_to_another_service(library, &name);
        }
        if is_unjudgeable_on_this_host(refusal)
            && !self
                .break_switches
                .is_on(BreakSwitch::BlockRefusedForMemoryClaimedAgain)
        {
            self.refused_for_running_out_of_memory
                .insert((library, name));
        }
    }

    fn withdraw_or_release_facts_verifier_after_refusal(
        &mut self,
        library: LibraryOfTheService,
        name: FactsVerifierBlockName,
        refusal: &ServiceRefusal,
    ) {
        if self
            .break_switches
            .is_on(BreakSwitch::BlockNotJudgeableClaimKept)
        {
            self.pool.withdraw_the_facts_verifier_block(library, &name);
        } else {
            let _ = self
                .pool
                .release_the_facts_verifier_claim_left_to_another_service(library, &name);
        }
        if is_unjudgeable_on_this_host(refusal)
            && !self
                .break_switches
                .is_on(BreakSwitch::BlockRefusedForMemoryClaimedAgain)
        {
            self.facts_blocks_refused_for_running_out_of_memory
                .insert((library, name));
        }
    }

    fn withdraw_or_release_packed_tables_after_refusal(
        &mut self,
        library: LibraryOfTheService,
        name: PackedTablesName,
        refusal: &ServiceRefusal,
    ) {
        if self
            .break_switches
            .is_on(BreakSwitch::BlockNotJudgeableClaimKept)
        {
            self.pool.withdraw_the_packed_tables(library, &name);
        } else {
            let _ = self
                .pool
                .release_the_packed_tables_claim_left_to_another_service(library, &name);
        }
        if is_unjudgeable_on_this_host(refusal)
            && !self
                .break_switches
                .is_on(BreakSwitch::BlockRefusedForMemoryClaimedAgain)
        {
            self.packed_tables_refused_for_running_out_of_memory
                .insert((library, name));
        }
    }

    /// 去掉这个进程不再认领的候选（文件头：在腾空的卡上也显存不够而被拒过的）。
    fn candidates_left_after_running_out_of_memory(
        &self,
        mut candidates: CandidatesOfEveryLibrary,
    ) -> CandidatesOfEveryLibrary {
        candidates.retain_blocks(|library, pending| {
            !self
                .refused_for_running_out_of_memory
                .contains(&(library, pending.name))
        });
        candidates.retain_facts_verifier_blocks(|library, pending| {
            !self
                .facts_blocks_refused_for_running_out_of_memory
                .contains(&(library, pending.name))
        });
        candidates.retain_packed_tables(|library, pending| {
            !self
                .packed_tables_refused_for_running_out_of_memory
                .contains(&(library, pending.name))
        });
        candidates
    }

    /// 打这一段的按卡汇总（文件头「日志」）：最近开的那一组卡与这一段判过、核过块的卡各一行，再一行这一段滤掉了几行逐次派活的行；之后另起一段。
    fn print_the_card_period(&mut self) {
        let elapsed = self.period_started.elapsed();
        let elapsed_nanoseconds = elapsed.as_nanos().max(1);
        let cards: BTreeSet<CardIndex> = self
            .cards_that_judge
            .keys()
            .chain(self.period_work.keys())
            .copied()
            .collect();
        for card in cards {
            let work = self.period_work.get(&card).copied().unwrap_or_default();
            let busy_permille = u128::from(work.card_nanoseconds) * PERMILLE / elapsed_nanoseconds;
            self.pool.print_a_service_line(format!(
                "{CARD_PERIOD_LINE_PREFIX} card={} seconds={} blocks={} states={} facts_blocks={} facts_states={} busy_permille={busy_permille}",
                card.0,
                elapsed.as_secs(),
                work.blocks,
                work.states,
                work.facts_blocks,
                work.facts_states
            ));
        }
        let dropped = DISPATCH_LINES_DROPPED.swap(0, Ordering::Relaxed);
        self.pool.print_a_service_line(format!(
            "{DISPATCH_LINES_DROPPED_LINE_PREFIX} seconds={} lines={dropped}",
            elapsed.as_secs()
        ));
        self.period_work.clear();
        self.period_started = Instant::now();
    }

    /// 叫会话判完就停，等它退出，写回它交回的块，放掉卡。
    fn end_the_session(&mut self) {
        let Some(session) = self.session.take() else {
            return;
        };
        session.client.drain_and_stop();
        let joined = session.thread.join();
        let cards = match joined {
            Ok((cards, run_summary)) => {
                if !self
                    .break_switches
                    .is_on(BreakSwitch::CardsFailedLeftOutOfTheSummary)
                {
                    self.summary.cards_failed.extend(run_summary.cards_failed);
                }
                self.summary.out_of_memory_caught += run_summary.out_of_memory_caught;
                cards
            }
            Err(panic) => std::panic::resume_unwind(panic),
        };
        // 会话已退出：它交出的几件事都在那一头了，先打出来（那几块的名字还在 in_flight 里）
        self.print_the_notices_of(&session.notices);
        // 会话已退出：每一块的回话都已交到（判完或被拒），没交到的是那一头没了，照拒收；写完再放掉卡
        for block in std::mem::take(&mut self.in_flight) {
            let answer = block
                .answer
                .recv()
                .unwrap_or(Err(ServiceRefusal::ServiceStopping));
            self.take_one_answer(block.library, block.name, &block.ordinals, answer);
        }
        for job in std::mem::take(&mut self.facts_in_flight) {
            let answer = job
                .answer
                .recv()
                .unwrap_or(Err(ServiceRefusal::ServiceStopping));
            self.take_one_facts_verifier_answer(job.library, job.blocks, answer);
        }
        for packed_tables in std::mem::take(&mut self.packed_tables_in_flight) {
            let answer = packed_tables
                .answer
                .recv()
                .unwrap_or(Err(ServiceRefusal::ServiceStopping));
            self.take_one_packed_tables_answer(
                packed_tables.library,
                packed_tables.name,
                &packed_tables.ordinals,
                answer,
            );
        }
        drop(cards);
    }
}

/// 被主循环拒的那一行末尾：在腾空的卡上也显存不够的这个进程不再取，别的下一轮再取。
fn refusal_after_word(not_judgeable_here: bool) -> &'static str {
    if not_judgeable_here {
        "这一台判不了：放掉认领留给另一台，这个进程不再取"
    } else {
        "下一轮再取"
    }
}

/// 判不了那一行的 `why=`：在腾空的卡上也显存不够。
const WHY_OUT_OF_MEMORY_ON_AN_EMPTIED_CARD: &str = "out-of-memory-on-an-emptied-card";
/// 判不了那一行的 `why=`：判法自己 panic 了。
const WHY_JUDGE_PANICKED: &str = "judge-panicked";
/// 判不了那一行的 `claim=`：放掉了。
const CLAIM_RELEASED: &str = "released";
/// 判不了那一行的 `claim=`：放的时候已不在自己手里。
const CLAIM_NOT_HELD: &str = "not-held";
/// 判不了那一行的 `claim=`：没放，留到租期过了。
const CLAIM_KEPT: &str = "kept";
/// 判不了那一行的 `family=`：三族的族名。
const IMAGE_JUDGEMENT_FAMILY_NAME: &str = "image-judgement";
const FACTS_VERIFIER_FAMILY_NAME: &str = "facts-verifier";
const PACKED_TABLES_FAMILY_NAME: &str = "packed-tables";
/// 判表那一族判不了那一行的 `block=`（一张判表在主循环里是一整块，没有块号）。
const PACKED_TABLES_BLOCK_WORD: &str = "tables";

fn is_unjudgeable_on_this_host(refusal: &ServiceRefusal) -> bool {
    matches!(
        refusal,
        ServiceRefusal::StreamFitsNoCard { .. }
            | ServiceRefusal::FactsVerifierFitsNoCard { .. }
            | ServiceRefusal::NoWorkingCard
            | ServiceRefusal::OrdinalsOutsideTheStream { .. }
            | ServiceRefusal::StreamRegisteredWithOtherFacts { .. }
    )
}

/// 主循环拒的理由是这一台判不了（文件头「判不了」）：交（卡、`why=` 那个词、原因）；别的拒交 `None`。
fn not_judgeable_here(refusal: &ServiceRefusal) -> Option<(CardIndex, &'static str, String)> {
    match refusal {
        ServiceRefusal::OutOfMemoryOnAnEmptiedCard { card, reason } => {
            Some((*card, WHY_OUT_OF_MEMORY_ON_AN_EMPTIED_CARD, reason.clone()))
        }
        ServiceRefusal::JudgePanicked { card, reason } => {
            Some((*card, WHY_JUDGE_PANICKED, reason.clone()))
        }
        ServiceRefusal::StreamNotRegistered { .. }
        | ServiceRefusal::StreamRegisteredWithOtherFacts { .. }
        | ServiceRefusal::StreamFitsNoCard { .. }
        | ServiceRefusal::OrdinalsOutsideTheStream { .. }
        | ServiceRefusal::CardFailed { .. }
        | ServiceRefusal::NoWorkingCard
        | ServiceRefusal::ServiceStopping
        | ServiceRefusal::FactsVerifierFitsNoCard { .. } => None,
    }
}

/// 放认领的结局写成判不了那一行的 `claim=`；放的时候读写判定库失败的打一行，认领留到租期过了。
/// `None` 是那一份登记的库已经丢掉（认领已不在自己手里）。
fn claim_left_to_another_service(
    released: Option<Result<ClaimsReleased, LibraryFailure>>,
    what: &str,
) -> &'static str {
    match released {
        Some(Ok(released)) if released.released > 0 => CLAIM_RELEASED,
        Some(Ok(_)) | None => CLAIM_NOT_HELD,
        Some(Err(failure)) => {
            eprintln!("{SERVING_PROBLEM_LINE_PREFIX} {what} 这一台判不了，放它的认领时读写判定库失败：{failure:?}（认领留到租期过了）");
            CLAIM_KEPT
        }
    }
}
