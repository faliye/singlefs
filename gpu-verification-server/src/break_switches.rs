//! 弄坏开关 `GPU_VERIFICATION_SERVER_BREAK=<项>[,<项>…]`：只给测试证红用（`.claude/rules/test-script-discipline.md`「测试脚本自己也要被测试证明有效」）。
//! 批次调度、待判池（内存那一份与判定库那一份）与取活的主循环都不读环境：开关经 [`BreakSwitches`] 交进它们，服务进程交的是
//! [`BreakSwitches::none`]；测试经 [`BreakSwitches::of_the_environment`] 读这个变量。二进制的 `lease` 子命令只在临时判定库服务底下读它
//! （`crate::card_lease` 文件头「弄坏开关」，借卡那几条测试起的就是这个二进制）。每一项弄坏哪一处、打开后哪个测试目标里哪一条测试红，
//! 表在 [`BreakSwitch::red_test_target`] 与 [`BreakSwitch::red_test`]，README「弄坏开关」一节逐项相同。

use std::collections::BTreeSet;

/// 环境变量名。
pub const BREAK_VARIABLE: &str = "GPU_VERIFICATION_SERVER_BREAK";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BreakSwitch {
    /// 一张卡的显存账收判器时不核额度，越过额度的也收下。
    MemoryLedgerUnchecked,
    /// 给新装的判器分草稿区时不扣这张卡上留着的判器占的显存。
    ScratchSplitIgnoresKeptJudges,
    /// 切批时取过的态不从池里那一截扣掉，下一批又从同一个态起切。
    CarvedStatesNotConsumed,
    /// 内存池把一块派出去一部分时，没派到的那一截也当派出去了。
    PoolDropsTheRestOfAPartlyDispatchedBlock,
    /// 不看水位线：池里有态就发。
    WatermarkIgnored,
    /// 不看限时：池里的态不够填满每张卡时，最老的待判等多久都不发。
    WaitLimitIgnored,
    /// 新装的判器一次派活的态数不按这条流的态数压。
    BatchNotCappedByTheStateCount,
    /// 一张卡上新装的几条流不均分剩下的显存，按入池先后先到先拿。
    ScratchSplitByEntryOrder,
    /// 一张卡上一条流这一波能派的批连着派完，才轮到下一条流。
    BatchesOfAStreamBackToBack,
    /// 流不按入池先后排，按键排。
    PoolOrderIgnored,
    /// 等过限时、哪张空着的卡都装不下的流不腾卡给它。
    StarvingStreamsNotRescued,
    /// 不认驻留：已经在卡上的判器不先派回那张卡，当新流重装。
    ResidenceIgnored,
    /// 要给新流腾显存时把这张卡上的判器全换出，有待判态的也换。
    EveryResidentJudgeEvicted,
    /// 调度不把输入排成规范次序，照调用方给的次序走。
    InputOrderFollowed,
    /// 判定库那一份待判池认领时别的持有者持着也当认领到了。
    ClaimHeldByOthersIgnored,
    /// 强制现判的块也先问库里判完没有，判完的不取。
    ForcedBlockReadsTheLibrary,
    /// 强制现判的块写结论那一帧不写凭据行。
    RejudgedRowNotWritten,
    /// 结论那一帧没写成时结论不留，只打红行。
    RefusedConclusionsDropped,
    /// 起来不开卡，库里有了能取的块才开。
    CardsOpenedOnlyWithWork,
    /// 池空一圈（没有在判的块、这一圈也没认领到）就放掉卡，再来活再开。
    CardsLetGoWhenThePoolIdles,
    /// 判完而可复用的块不删它的待判行（只不取）。
    ReusableBlockPendingRowKept,
    /// 写结论那一帧因待判行读完之后变了被拒时不重读、当场报红。
    PendingRowChangeNotReread,
    /// 写结论那一帧因结论块那一行编帧之后变了被拒时不重编、当场报红。
    GuardRowChangeNotReread,
    /// 认领到之后不重读待判行：扫库与认领之间别人写回了的块照取、刚认领到的不放。
    PendingRowNotRereadAfterTheClaim,
    /// 起来时不放上一个服务进程（已不在）留下的认领。
    PreviousProcessClaimsKept,
    /// 上一个服务进程还活着也当它没了，放掉它的认领。
    PreviousProcessTakenForGone,
    /// 停的时候不放自己还持着的认领。
    ClaimsKeptAtTheStop,
    /// 服务日志不滤逐次派活的那两种行，照转。
    DispatchLinesForwarded,
    /// 取活循环不扫核对待判行那一张表。
    FactsVerifierRowsNotScanned,
    /// 核对行那一帧的认领键写成镜像判定那一族的形（`gpu-claim/<流键>/<块号>`），认领到的那一把核对的认领键没随那一帧放掉。
    FactsVerifierFrameUnderTheImageJudgementClaimKey,
    /// 不在库上现算核没核过：核对行已在的块照取、照核。
    FactsVerifierVerifiedBlockVerifiedAgain,
    /// 核对行那一帧的核对行守卫不取编帧之前读到的那一份，取写之前此刻库里的那一份（守卫形同不设）。
    FactsVerifierRowGuardTakenAtTheWrite,
    /// 核对行那一帧删核对待判行的期待不取服务取这一行时读到的那一份，取写之前此刻库里的那一份（守卫形同不设）。
    FactsVerifierPendingRowGuardTakenAtTheWrite,
    /// 核对行那一帧守卫不过时不重编、不重读，当场报红。
    FactsVerifierGuardRefusalNotRetried,
    /// 起来时放上一个进程的认领、停的时候放自己的认领都只放镜像判定那一族的，核对待判行那一族的不放。
    FactsVerifierClaimsLeftAtTheRelease,
    /// 认领到核对待判行之后不重读：扫库与认领之间别人写回了的块照取、刚认领到的不放。
    FactsVerifierRowNotRereadAfterTheClaim,
    /// 服务主循环派核对器上卡时不照显存账腾地方：驻留的判器不换出，放不放得下都派。
    FactsVerifierMemoryUnchecked,
    /// 取活循环不扫判表待判行那一张表。
    PackedTablesRowsNotScanned,
    /// 不在库上现算结论字在不在：表 `21` 已有这张判表的结论字、不强制现判的照取、照判。
    PackedTablesVerdictWordsJudgedAgain,
    /// 结论字那一帧的表 `21` 守卫不取编帧之前读到的那一份，取写之前此刻库里的那一份（守卫形同不设）。
    PackedTablesVerdictWordsGuardTakenAtTheWrite,
    /// 结论字那一帧删判表待判行的期待不取服务取这一行时读到的那一份，取写之前此刻库里的那一份（守卫形同不设）。
    PackedTablesPendingRowGuardTakenAtTheWrite,
    /// 结论字那一帧守卫不过时不重编、不重读，当场报红。
    PackedTablesGuardRefusalNotRetried,
    /// 起来时放上一个进程的认领、停的时候放自己的认领都不放判表待判行那一族的。
    PackedTablesClaimsLeftAtTheRelease,
    /// 认领到判表待判行之后不重读：扫库与认领之间别人写回了的那张表照取、刚认领到的不放。
    PackedTablesRowNotRereadAfterTheClaim,
    /// 取活循环不探请求键：有人要借也照认领候选、不放卡键。
    CardLeaseRequestIgnored,
    /// 探到有人要借就放卡键，手上入了池的块还没判完写回也放。
    CardsLentBeforeTheBlocksInHandAreWrittenBack,
    /// 卡借出去时把设备也放掉（收回时有活再开）。
    DevicesLetGoWhileLent,
    /// 借出之后请求键没人持了也不收回卡键，一直不认领候选。
    CardKeyNotTakenBack,
    /// 起来时卡键被别人持着也当自己持着，照认领候选。
    CardKeyHeldByAnotherAtStartIgnored,
    /// `lease take` 认领到请求键就打卡串，不等卡键。
    CardLeasePrintedBeforeTheCardKey,
    /// `lease return` 不认领停止键、不等续租进程，只放两把键：续租进程照续。
    CardLeaseReturnLeavesTheRenewerRunning,
    /// 续租进程不看借的进程在不在，一直续。
    CardLeaseRenewerIgnoresTheBorrower,
    /// `lease take` 认领请求键、卡键时别人持着也当认领到。
    CardLeaseKeysHeldByAnotherTakenAsClaimed,
    /// 借的一方的变量没给时取子命令的父进程当借的进程。
    CardLeaseBorrowerFromTheParentProcess,
    /// `lease take` 在多机配置没点名这一台的卡时照借（卡串是空的）。
    CardLeaseTakenWithoutCardsOfThisHost,
    /// `lease return` 的卡串是空的（或只有空白）当用法错、退 2，不还。
    CardLeaseReturnRefusesAnEmptyCardString,
    /// `lease take` 不照借卡白名单认借的一方：谁都借给。
    CardLeaseBorrowerWhitelistUnchecked,
    /// `lease take` 不看等号的次序：每一回都直接去认领请求键（谁先撞上谁拿）。
    CardLeaseWaitingOrderIgnored,
    /// 新装判器的那一刻只给表留一份（不留 wgpu 上卡时另开的那一份暂存）。
    TableStagingNotReserved,
    /// 工作线程接住设备没丢的 Out of Memory 也报卡坏、线程退出。
    OutOfMemoryTakenForACardFailure,
    /// 主循环接住 Out of Memory 之后不清这张卡驻留的判器（卡上已丢了，账还记着）。
    ResidentsKeptAfterRunningOutOfMemory,
    /// 接住 Out of Memory 之后这一波在判的块整块报拒，不撤回重排。
    BlocksOfTheOutOfMemoryWaveRefused,
    /// 接住 Out of Memory 之后不交「接住」那一件（取活循环不打那一行）。
    OutOfMemoryCaughtNotTold,
    /// 撤回的块照常交调度，不等一张腾空的卡单独判。
    RetractedBlockSharesTheNextWave,
    /// 单独在腾空的卡上又显存不够的块照样撤回重排，不报拒。
    OutOfMemoryOnAnEmptiedCardRetried,
    /// 核的时候显存不够、设备没丢，那一组核对块报拒，不放回去腾空一张卡再核。
    FactsJobOfTheOutOfMemoryRoundRefused,
    /// 一张卡坏了不交「卡坏」那一件（取活循环不打卡坏那一行）。
    CardFailureNotTold,
    /// 装上一个判器不交「装上」那一件（取活循环不打装判器那一行）。
    JudgeLoadedNotTold,
    /// 会话收尾时丢掉主循环交回的坏了的卡，取活循环的账（STOPPED 那一行）里没有它们。
    CardsFailedLeftOutOfTheSummary,
    /// 在腾空的卡上也显存不够而被拒的块，这个进程下一轮照样认领。
    BlockRefusedForMemoryClaimedAgain,
    /// 登记没了（租期过了、判段放了）也不丢那一份登记的库，照扫、在判的块照写回。
    RegistrationLapseIgnored,
    /// 登记的库读写失败照真库那样交回失败，让服务退出。
    TemporaryLibraryFailureEndsTheService,
    /// 指到真服务的登记、键空间前缀头一个字节不对的登记照开。
    RealServiceRegistrationOpened,
    /// 各份库认领到的块不合起来按入池序号排，真库的先、登记的库的后。
    RealLibraryClaimedFirst,
    /// 卡借出期间登记的库照扫、照认领。
    TemporaryLibrariesScannedWhileLent,
    /// 打开登记的库时不放上一个服务进程留在那份库里的认领。
    PreviousClaimsKeptInTemporaryLibraries,
    /// 登记的库的认领与写结论的客户端连真服务：结论写进真库。
    ConclusionWrittenIntoTheRealLibrary,
    /// 判完的块照块名找回它的库（排在前面认领着这个名字的那一份），不照入池时记的库。
    JudgedBlockMatchedToItsLibraryByName,
    /// 工作线程接住设备没丢的 Out of Memory 之后不在这张卡上做那一次提交、不等它做完，就报接住、收下一波。
    CardNotSettledAfterRunningOutOfMemory,
    /// 工作线程把因显存不够丢了、已重开设备的也报卡坏、线程退出。
    DeviceLostForWantOfMemoryTakenForACardFailure,
    /// 工作线程把判法自己的 panic（设备没丢、没记 Out of Memory）报卡坏、线程退出。
    JudgePanicTakenForACardFailure,
    /// 显存不够撤回的块一律只等撤回它的那张卡腾空单独判，别的没坏、装得下的卡不拿。
    PutBackBlocksWaitForTheirOwnCard,
    /// 一张卡坏了之后，只有它装得下的流的块不报拒：池里的挂着，再入池的照收。
    BlocksOnlyTheFailedCardCouldHoldLeftPending,
    /// 核对器借的显存里六张事实表只算一份（不留上卡时 wgpu 另开的那一份暂存）。
    FactsTablesStagingNotReserved,
    /// 在腾空的卡上也显存不够、或判法 panic 而被拒的块，取活循环不当场放掉它的认领（留到租期过了）。
    BlockNotJudgeableClaimKept,
    /// 在腾空的卡上也显存不够、或判法 panic 而被拒的块，取活循环不打「判不了」那一行。
    BlockNotJudgeableLineLeftOut,
    /// 显存不够或判法 panic 时撤整块：一截没交回结论的块整块从池里撤掉、照整块的序号重新入池，回话改挂到新的号上，
    /// 旧号上别的卡交回的结论不记（别的卡判过的态再判一遍）。
    PutBackTakesTheWholeBlock,
}

/// 批次调度那几项红在这个测试目标里。
const BATCH_SCHEDULING_TARGET: &str = "batch_scheduling_requirements";
/// 判定库那一份待判池与取活循环持卡那几项红在这个测试目标里。
const LIBRARY_POOL_TARGET: &str = "library_pending_pool_on_a_temporary_library";
/// 服务主循环派核对器上卡、接住 Out of Memory、卡坏与装判器那几件红在这个测试目标里。
const SERVICE_LOOP_TARGET: &str = "service_loop_with_judges_off_the_card";
/// 借卡那几项（取活循环那一半与 `lease` 子命令）红在这个测试目标里。
const CARD_LEASE_TARGET: &str = "card_lease_on_a_temporary_library";
/// 登记的临时判定库那几项红在这个测试目标里。
const REGISTERED_LIBRARIES_TARGET: &str =
    "library_pending_pool_serves_registered_temporary_libraries";

impl BreakSwitch {
    pub const EVERY: [BreakSwitch; 87] = [
        BreakSwitch::MemoryLedgerUnchecked,
        BreakSwitch::ScratchSplitIgnoresKeptJudges,
        BreakSwitch::CarvedStatesNotConsumed,
        BreakSwitch::PoolDropsTheRestOfAPartlyDispatchedBlock,
        BreakSwitch::WatermarkIgnored,
        BreakSwitch::WaitLimitIgnored,
        BreakSwitch::BatchNotCappedByTheStateCount,
        BreakSwitch::ScratchSplitByEntryOrder,
        BreakSwitch::BatchesOfAStreamBackToBack,
        BreakSwitch::PoolOrderIgnored,
        BreakSwitch::StarvingStreamsNotRescued,
        BreakSwitch::ResidenceIgnored,
        BreakSwitch::EveryResidentJudgeEvicted,
        BreakSwitch::InputOrderFollowed,
        BreakSwitch::ClaimHeldByOthersIgnored,
        BreakSwitch::ForcedBlockReadsTheLibrary,
        BreakSwitch::RejudgedRowNotWritten,
        BreakSwitch::RefusedConclusionsDropped,
        BreakSwitch::CardsOpenedOnlyWithWork,
        BreakSwitch::CardsLetGoWhenThePoolIdles,
        BreakSwitch::ReusableBlockPendingRowKept,
        BreakSwitch::PendingRowChangeNotReread,
        BreakSwitch::GuardRowChangeNotReread,
        BreakSwitch::PendingRowNotRereadAfterTheClaim,
        BreakSwitch::PreviousProcessClaimsKept,
        BreakSwitch::PreviousProcessTakenForGone,
        BreakSwitch::ClaimsKeptAtTheStop,
        BreakSwitch::DispatchLinesForwarded,
        BreakSwitch::FactsVerifierRowsNotScanned,
        BreakSwitch::FactsVerifierFrameUnderTheImageJudgementClaimKey,
        BreakSwitch::FactsVerifierVerifiedBlockVerifiedAgain,
        BreakSwitch::FactsVerifierRowGuardTakenAtTheWrite,
        BreakSwitch::FactsVerifierPendingRowGuardTakenAtTheWrite,
        BreakSwitch::FactsVerifierGuardRefusalNotRetried,
        BreakSwitch::FactsVerifierClaimsLeftAtTheRelease,
        BreakSwitch::FactsVerifierRowNotRereadAfterTheClaim,
        BreakSwitch::FactsVerifierMemoryUnchecked,
        BreakSwitch::PackedTablesRowsNotScanned,
        BreakSwitch::PackedTablesVerdictWordsJudgedAgain,
        BreakSwitch::PackedTablesVerdictWordsGuardTakenAtTheWrite,
        BreakSwitch::PackedTablesPendingRowGuardTakenAtTheWrite,
        BreakSwitch::PackedTablesGuardRefusalNotRetried,
        BreakSwitch::PackedTablesClaimsLeftAtTheRelease,
        BreakSwitch::PackedTablesRowNotRereadAfterTheClaim,
        BreakSwitch::CardLeaseRequestIgnored,
        BreakSwitch::CardsLentBeforeTheBlocksInHandAreWrittenBack,
        BreakSwitch::DevicesLetGoWhileLent,
        BreakSwitch::CardKeyNotTakenBack,
        BreakSwitch::CardKeyHeldByAnotherAtStartIgnored,
        BreakSwitch::CardLeasePrintedBeforeTheCardKey,
        BreakSwitch::CardLeaseReturnLeavesTheRenewerRunning,
        BreakSwitch::CardLeaseRenewerIgnoresTheBorrower,
        BreakSwitch::CardLeaseKeysHeldByAnotherTakenAsClaimed,
        BreakSwitch::CardLeaseBorrowerFromTheParentProcess,
        BreakSwitch::CardLeaseTakenWithoutCardsOfThisHost,
        BreakSwitch::CardLeaseReturnRefusesAnEmptyCardString,
        BreakSwitch::CardLeaseBorrowerWhitelistUnchecked,
        BreakSwitch::CardLeaseWaitingOrderIgnored,
        BreakSwitch::TableStagingNotReserved,
        BreakSwitch::OutOfMemoryTakenForACardFailure,
        BreakSwitch::ResidentsKeptAfterRunningOutOfMemory,
        BreakSwitch::BlocksOfTheOutOfMemoryWaveRefused,
        BreakSwitch::OutOfMemoryCaughtNotTold,
        BreakSwitch::RetractedBlockSharesTheNextWave,
        BreakSwitch::OutOfMemoryOnAnEmptiedCardRetried,
        BreakSwitch::FactsJobOfTheOutOfMemoryRoundRefused,
        BreakSwitch::CardFailureNotTold,
        BreakSwitch::JudgeLoadedNotTold,
        BreakSwitch::CardsFailedLeftOutOfTheSummary,
        BreakSwitch::BlockRefusedForMemoryClaimedAgain,
        BreakSwitch::RegistrationLapseIgnored,
        BreakSwitch::TemporaryLibraryFailureEndsTheService,
        BreakSwitch::RealServiceRegistrationOpened,
        BreakSwitch::RealLibraryClaimedFirst,
        BreakSwitch::TemporaryLibrariesScannedWhileLent,
        BreakSwitch::PreviousClaimsKeptInTemporaryLibraries,
        BreakSwitch::ConclusionWrittenIntoTheRealLibrary,
        BreakSwitch::JudgedBlockMatchedToItsLibraryByName,
        BreakSwitch::CardNotSettledAfterRunningOutOfMemory,
        BreakSwitch::DeviceLostForWantOfMemoryTakenForACardFailure,
        BreakSwitch::JudgePanicTakenForACardFailure,
        BreakSwitch::PutBackBlocksWaitForTheirOwnCard,
        BreakSwitch::BlocksOnlyTheFailedCardCouldHoldLeftPending,
        BreakSwitch::FactsTablesStagingNotReserved,
        BreakSwitch::BlockNotJudgeableClaimKept,
        BreakSwitch::BlockNotJudgeableLineLeftOut,
        BreakSwitch::PutBackTakesTheWholeBlock,
    ];

    /// 变量里写的项名。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            BreakSwitch::MemoryLedgerUnchecked => "memory-ledger-unchecked",
            BreakSwitch::ScratchSplitIgnoresKeptJudges => "scratch-split-ignores-kept-judges",
            BreakSwitch::CarvedStatesNotConsumed => "carved-states-not-consumed",
            BreakSwitch::PoolDropsTheRestOfAPartlyDispatchedBlock => {
                "pool-drops-the-rest-of-a-partly-dispatched-block"
            }
            BreakSwitch::WatermarkIgnored => "watermark-ignored",
            BreakSwitch::WaitLimitIgnored => "wait-limit-ignored",
            BreakSwitch::BatchNotCappedByTheStateCount => "batch-not-capped-by-the-state-count",
            BreakSwitch::ScratchSplitByEntryOrder => "scratch-split-by-entry-order",
            BreakSwitch::BatchesOfAStreamBackToBack => "batches-of-a-stream-back-to-back",
            BreakSwitch::PoolOrderIgnored => "pool-order-ignored",
            BreakSwitch::StarvingStreamsNotRescued => "starving-streams-not-rescued",
            BreakSwitch::ResidenceIgnored => "residence-ignored",
            BreakSwitch::EveryResidentJudgeEvicted => "every-resident-judge-evicted",
            BreakSwitch::InputOrderFollowed => "input-order-followed",
            BreakSwitch::ClaimHeldByOthersIgnored => "claim-held-by-others-ignored",
            BreakSwitch::ForcedBlockReadsTheLibrary => "forced-block-reads-the-library",
            BreakSwitch::RejudgedRowNotWritten => "rejudged-row-not-written",
            BreakSwitch::RefusedConclusionsDropped => "refused-conclusions-dropped",
            BreakSwitch::CardsOpenedOnlyWithWork => "cards-opened-only-with-work",
            BreakSwitch::CardsLetGoWhenThePoolIdles => "cards-let-go-when-the-pool-idles",
            BreakSwitch::ReusableBlockPendingRowKept => "reusable-block-pending-row-kept",
            BreakSwitch::PendingRowChangeNotReread => "pending-row-change-not-reread",
            BreakSwitch::GuardRowChangeNotReread => "guard-row-change-not-reread",
            BreakSwitch::PendingRowNotRereadAfterTheClaim => {
                "pending-row-not-reread-after-the-claim"
            }
            BreakSwitch::PreviousProcessClaimsKept => "previous-process-claims-kept",
            BreakSwitch::PreviousProcessTakenForGone => "previous-process-taken-for-gone",
            BreakSwitch::ClaimsKeptAtTheStop => "claims-kept-at-the-stop",
            BreakSwitch::DispatchLinesForwarded => "dispatch-lines-forwarded",
            BreakSwitch::FactsVerifierRowsNotScanned => "facts-verifier-rows-not-scanned",
            BreakSwitch::FactsVerifierFrameUnderTheImageJudgementClaimKey => {
                "facts-verifier-frame-under-the-image-judgement-claim-key"
            }
            BreakSwitch::FactsVerifierVerifiedBlockVerifiedAgain => {
                "facts-verifier-verified-block-verified-again"
            }
            BreakSwitch::FactsVerifierRowGuardTakenAtTheWrite => {
                "facts-verifier-row-guard-taken-at-the-write"
            }
            BreakSwitch::FactsVerifierPendingRowGuardTakenAtTheWrite => {
                "facts-verifier-pending-row-guard-taken-at-the-write"
            }
            BreakSwitch::FactsVerifierGuardRefusalNotRetried => {
                "facts-verifier-guard-refusal-not-retried"
            }
            BreakSwitch::FactsVerifierClaimsLeftAtTheRelease => {
                "facts-verifier-claims-left-at-the-release"
            }
            BreakSwitch::FactsVerifierRowNotRereadAfterTheClaim => {
                "facts-verifier-row-not-reread-after-the-claim"
            }
            BreakSwitch::FactsVerifierMemoryUnchecked => "facts-verifier-memory-unchecked",
            BreakSwitch::PackedTablesRowsNotScanned => "packed-tables-rows-not-scanned",
            BreakSwitch::PackedTablesVerdictWordsJudgedAgain => {
                "packed-tables-verdict-words-judged-again"
            }
            BreakSwitch::PackedTablesVerdictWordsGuardTakenAtTheWrite => {
                "packed-tables-verdict-words-guard-taken-at-the-write"
            }
            BreakSwitch::PackedTablesPendingRowGuardTakenAtTheWrite => {
                "packed-tables-pending-row-guard-taken-at-the-write"
            }
            BreakSwitch::PackedTablesGuardRefusalNotRetried => {
                "packed-tables-guard-refusal-not-retried"
            }
            BreakSwitch::PackedTablesClaimsLeftAtTheRelease => {
                "packed-tables-claims-left-at-the-release"
            }
            BreakSwitch::PackedTablesRowNotRereadAfterTheClaim => {
                "packed-tables-row-not-reread-after-the-claim"
            }
            BreakSwitch::CardLeaseRequestIgnored => "card-lease-request-ignored",
            BreakSwitch::CardsLentBeforeTheBlocksInHandAreWrittenBack => {
                "cards-lent-before-the-blocks-in-hand-are-written-back"
            }
            BreakSwitch::DevicesLetGoWhileLent => "devices-let-go-while-lent",
            BreakSwitch::CardKeyNotTakenBack => "card-key-not-taken-back",
            BreakSwitch::CardKeyHeldByAnotherAtStartIgnored => {
                "card-key-held-by-another-at-start-ignored"
            }
            BreakSwitch::CardLeasePrintedBeforeTheCardKey => {
                "card-lease-printed-before-the-card-key"
            }
            BreakSwitch::CardLeaseReturnLeavesTheRenewerRunning => {
                "card-lease-return-leaves-the-renewer-running"
            }
            BreakSwitch::CardLeaseRenewerIgnoresTheBorrower => {
                "card-lease-renewer-ignores-the-borrower"
            }
            BreakSwitch::CardLeaseKeysHeldByAnotherTakenAsClaimed => {
                "card-lease-keys-held-by-another-taken-as-claimed"
            }
            BreakSwitch::CardLeaseBorrowerFromTheParentProcess => {
                "card-lease-borrower-from-the-parent-process"
            }
            BreakSwitch::CardLeaseTakenWithoutCardsOfThisHost => {
                "card-lease-taken-without-cards-of-this-host"
            }
            BreakSwitch::CardLeaseReturnRefusesAnEmptyCardString => {
                "card-lease-return-refuses-an-empty-card-string"
            }
            BreakSwitch::CardLeaseBorrowerWhitelistUnchecked => {
                "card-lease-borrower-whitelist-unchecked"
            }
            BreakSwitch::CardLeaseWaitingOrderIgnored => "card-lease-waiting-order-ignored",
            BreakSwitch::TableStagingNotReserved => "table-staging-not-reserved",
            BreakSwitch::OutOfMemoryTakenForACardFailure => {
                "out-of-memory-taken-for-a-card-failure"
            }
            BreakSwitch::ResidentsKeptAfterRunningOutOfMemory => {
                "residents-kept-after-running-out-of-memory"
            }
            BreakSwitch::BlocksOfTheOutOfMemoryWaveRefused => {
                "blocks-of-the-out-of-memory-wave-refused"
            }
            BreakSwitch::OutOfMemoryCaughtNotTold => "out-of-memory-caught-not-told",
            BreakSwitch::RetractedBlockSharesTheNextWave => "retracted-block-shares-the-next-wave",
            BreakSwitch::OutOfMemoryOnAnEmptiedCardRetried => {
                "out-of-memory-on-an-emptied-card-retried"
            }
            BreakSwitch::FactsJobOfTheOutOfMemoryRoundRefused => {
                "facts-job-of-the-out-of-memory-round-refused"
            }
            BreakSwitch::CardFailureNotTold => "card-failure-not-told",
            BreakSwitch::JudgeLoadedNotTold => "judge-loaded-not-told",
            BreakSwitch::CardsFailedLeftOutOfTheSummary => "cards-failed-left-out-of-the-summary",
            BreakSwitch::BlockRefusedForMemoryClaimedAgain => {
                "block-refused-for-memory-claimed-again"
            }
            BreakSwitch::RegistrationLapseIgnored => "registration-lapse-ignored",
            BreakSwitch::TemporaryLibraryFailureEndsTheService => {
                "temporary-library-failure-ends-the-service"
            }
            BreakSwitch::RealServiceRegistrationOpened => "real-service-registration-opened",
            BreakSwitch::RealLibraryClaimedFirst => "real-library-claimed-first",
            BreakSwitch::TemporaryLibrariesScannedWhileLent => {
                "temporary-libraries-scanned-while-lent"
            }
            BreakSwitch::PreviousClaimsKeptInTemporaryLibraries => {
                "previous-claims-kept-in-temporary-libraries"
            }
            BreakSwitch::ConclusionWrittenIntoTheRealLibrary => {
                "conclusion-written-into-the-real-library"
            }
            BreakSwitch::JudgedBlockMatchedToItsLibraryByName => {
                "judged-block-matched-to-its-library-by-name"
            }
            BreakSwitch::CardNotSettledAfterRunningOutOfMemory => {
                "card-not-settled-after-running-out-of-memory"
            }
            BreakSwitch::DeviceLostForWantOfMemoryTakenForACardFailure => {
                "device-lost-for-want-of-memory-taken-for-a-card-failure"
            }
            BreakSwitch::JudgePanicTakenForACardFailure => "judge-panic-taken-for-a-card-failure",
            BreakSwitch::PutBackBlocksWaitForTheirOwnCard => {
                "put-back-blocks-wait-for-their-own-card"
            }
            BreakSwitch::BlocksOnlyTheFailedCardCouldHoldLeftPending => {
                "blocks-only-the-failed-card-could-hold-left-pending"
            }
            BreakSwitch::FactsTablesStagingNotReserved => "facts-tables-staging-not-reserved",
            BreakSwitch::BlockNotJudgeableClaimKept => "block-not-judgeable-claim-kept",
            BreakSwitch::BlockNotJudgeableLineLeftOut => "block-not-judgeable-line-left-out",
            BreakSwitch::PutBackTakesTheWholeBlock => "put-back-takes-the-whole-block",
        }
    }

    /// 打开这一项之后必须红的那一条测试在哪个测试目标里（`tests/<目标>.rs`）。
    #[must_use]
    pub fn red_test_target(self) -> &'static str {
        match self {
            BreakSwitch::ClaimHeldByOthersIgnored
            | BreakSwitch::ForcedBlockReadsTheLibrary
            | BreakSwitch::RejudgedRowNotWritten
            | BreakSwitch::RefusedConclusionsDropped
            | BreakSwitch::CardsOpenedOnlyWithWork
            | BreakSwitch::CardsLetGoWhenThePoolIdles
            | BreakSwitch::ReusableBlockPendingRowKept
            | BreakSwitch::PendingRowChangeNotReread
            | BreakSwitch::GuardRowChangeNotReread
            | BreakSwitch::PendingRowNotRereadAfterTheClaim
            | BreakSwitch::PreviousProcessClaimsKept
            | BreakSwitch::PreviousProcessTakenForGone
            | BreakSwitch::ClaimsKeptAtTheStop
            | BreakSwitch::DispatchLinesForwarded
            | BreakSwitch::FactsVerifierRowsNotScanned
            | BreakSwitch::FactsVerifierFrameUnderTheImageJudgementClaimKey
            | BreakSwitch::FactsVerifierVerifiedBlockVerifiedAgain
            | BreakSwitch::FactsVerifierRowGuardTakenAtTheWrite
            | BreakSwitch::FactsVerifierPendingRowGuardTakenAtTheWrite
            | BreakSwitch::FactsVerifierGuardRefusalNotRetried
            | BreakSwitch::FactsVerifierClaimsLeftAtTheRelease
            | BreakSwitch::FactsVerifierRowNotRereadAfterTheClaim
            | BreakSwitch::PackedTablesRowsNotScanned
            | BreakSwitch::PackedTablesVerdictWordsJudgedAgain
            | BreakSwitch::PackedTablesVerdictWordsGuardTakenAtTheWrite
            | BreakSwitch::PackedTablesPendingRowGuardTakenAtTheWrite
            | BreakSwitch::PackedTablesGuardRefusalNotRetried
            | BreakSwitch::PackedTablesClaimsLeftAtTheRelease
            | BreakSwitch::PackedTablesRowNotRereadAfterTheClaim
            | BreakSwitch::CardsFailedLeftOutOfTheSummary
            | BreakSwitch::BlockRefusedForMemoryClaimedAgain
            | BreakSwitch::BlockNotJudgeableClaimKept
            | BreakSwitch::BlockNotJudgeableLineLeftOut => LIBRARY_POOL_TARGET,
            BreakSwitch::MemoryLedgerUnchecked
            | BreakSwitch::ScratchSplitIgnoresKeptJudges
            | BreakSwitch::CarvedStatesNotConsumed
            | BreakSwitch::PoolDropsTheRestOfAPartlyDispatchedBlock
            | BreakSwitch::WatermarkIgnored
            | BreakSwitch::WaitLimitIgnored
            | BreakSwitch::BatchNotCappedByTheStateCount
            | BreakSwitch::ScratchSplitByEntryOrder
            | BreakSwitch::BatchesOfAStreamBackToBack
            | BreakSwitch::PoolOrderIgnored
            | BreakSwitch::StarvingStreamsNotRescued
            | BreakSwitch::ResidenceIgnored
            | BreakSwitch::EveryResidentJudgeEvicted
            | BreakSwitch::InputOrderFollowed
            | BreakSwitch::TableStagingNotReserved
            | BreakSwitch::FactsTablesStagingNotReserved => BATCH_SCHEDULING_TARGET,
            BreakSwitch::FactsVerifierMemoryUnchecked
            | BreakSwitch::OutOfMemoryTakenForACardFailure
            | BreakSwitch::ResidentsKeptAfterRunningOutOfMemory
            | BreakSwitch::BlocksOfTheOutOfMemoryWaveRefused
            | BreakSwitch::OutOfMemoryCaughtNotTold
            | BreakSwitch::PutBackTakesTheWholeBlock
            | BreakSwitch::RetractedBlockSharesTheNextWave
            | BreakSwitch::OutOfMemoryOnAnEmptiedCardRetried
            | BreakSwitch::FactsJobOfTheOutOfMemoryRoundRefused
            | BreakSwitch::CardFailureNotTold
            | BreakSwitch::JudgeLoadedNotTold
            | BreakSwitch::CardNotSettledAfterRunningOutOfMemory
            | BreakSwitch::DeviceLostForWantOfMemoryTakenForACardFailure
            | BreakSwitch::JudgePanicTakenForACardFailure
            | BreakSwitch::PutBackBlocksWaitForTheirOwnCard
            | BreakSwitch::BlocksOnlyTheFailedCardCouldHoldLeftPending => SERVICE_LOOP_TARGET,
            BreakSwitch::CardLeaseRequestIgnored
            | BreakSwitch::CardsLentBeforeTheBlocksInHandAreWrittenBack
            | BreakSwitch::DevicesLetGoWhileLent
            | BreakSwitch::CardKeyNotTakenBack
            | BreakSwitch::CardKeyHeldByAnotherAtStartIgnored
            | BreakSwitch::CardLeasePrintedBeforeTheCardKey
            | BreakSwitch::CardLeaseReturnLeavesTheRenewerRunning
            | BreakSwitch::CardLeaseRenewerIgnoresTheBorrower
            | BreakSwitch::CardLeaseKeysHeldByAnotherTakenAsClaimed
            | BreakSwitch::CardLeaseBorrowerFromTheParentProcess
            | BreakSwitch::CardLeaseTakenWithoutCardsOfThisHost
            | BreakSwitch::CardLeaseReturnRefusesAnEmptyCardString
            | BreakSwitch::CardLeaseBorrowerWhitelistUnchecked
            | BreakSwitch::CardLeaseWaitingOrderIgnored => CARD_LEASE_TARGET,
            BreakSwitch::RegistrationLapseIgnored
            | BreakSwitch::TemporaryLibraryFailureEndsTheService
            | BreakSwitch::RealServiceRegistrationOpened
            | BreakSwitch::RealLibraryClaimedFirst
            | BreakSwitch::TemporaryLibrariesScannedWhileLent
            | BreakSwitch::PreviousClaimsKeptInTemporaryLibraries
            | BreakSwitch::ConclusionWrittenIntoTheRealLibrary
            | BreakSwitch::JudgedBlockMatchedToItsLibraryByName => REGISTERED_LIBRARIES_TARGET,
        }
    }

    /// 打开这一项之后必须红的那一条测试（[`Self::red_test_target`] 那个目标里的函数名）。
    #[must_use]
    pub fn red_test(self) -> &'static str {
        match self {
            BreakSwitch::MemoryLedgerUnchecked | BreakSwitch::ScratchSplitIgnoresKeptJudges => {
                "judges_resident_on_a_card_never_exceed_the_quota_less_the_driver_share"
            }
            BreakSwitch::CarvedStatesNotConsumed
            | BreakSwitch::PoolDropsTheRestOfAPartlyDispatchedBlock => {
                "every_pending_state_is_dispatched_exactly_once_and_the_rest_stays_in_the_pool"
            }
            BreakSwitch::WatermarkIgnored | BreakSwitch::WaitLimitIgnored => {
                "a_wave_goes_out_when_every_card_fills_once_or_the_oldest_waited_the_limit_and_never_from_an_empty_pool"
            }
            BreakSwitch::BatchNotCappedByTheStateCount | BreakSwitch::ScratchSplitByEntryOrder => {
                "streams_sharing_a_card_split_the_remaining_memory_and_a_short_stream_takes_its_own_state_count"
            }
            BreakSwitch::BatchesOfAStreamBackToBack => {
                "batches_of_streams_sharing_a_card_alternate_from_wave_to_wave"
            }
            BreakSwitch::PoolOrderIgnored | BreakSwitch::StarvingStreamsNotRescued => {
                "streams_are_taken_in_entry_order_and_a_short_stream_is_not_left_behind_a_long_one"
            }
            BreakSwitch::ResidenceIgnored | BreakSwitch::EveryResidentJudgeEvicted => {
                "a_resident_stream_goes_back_to_its_card_and_only_judges_without_pending_states_are_evicted"
            }
            BreakSwitch::InputOrderFollowed => {
                "the_same_inputs_give_the_same_plan_whatever_order_they_come_in"
            }
            BreakSwitch::ClaimHeldByOthersIgnored => {
                "a_block_claimed_by_another_holder_is_left_to_it"
            }
            BreakSwitch::ForcedBlockReadsTheLibrary | BreakSwitch::RejudgedRowNotWritten => {
                "a_forced_block_is_judged_without_reading_the_library_and_leaves_a_rejudged_row"
            }
            BreakSwitch::RefusedConclusionsDropped => {
                "conclusions_refused_at_the_write_are_kept_and_reported_red"
            }
            BreakSwitch::CardsOpenedOnlyWithWork | BreakSwitch::CardsLetGoWhenThePoolIdles => {
                "the_cards_are_opened_at_start_and_held_while_the_pool_idles"
            }
            BreakSwitch::ReusableBlockPendingRowKept => {
                "a_reusable_block_is_not_taken_and_its_pending_row_is_deleted_under_the_guard"
            }
            BreakSwitch::PendingRowChangeNotReread => {
                "a_pending_row_raised_to_forced_while_judged_is_reread_and_written_with_its_credential"
            }
            BreakSwitch::GuardRowChangeNotReread => {
                "a_conclusion_changed_under_the_guard_is_compared_again_before_writing"
            }
            BreakSwitch::PendingRowNotRereadAfterTheClaim => {
                "a_block_written_back_between_the_scan_and_the_claim_is_not_taken_again"
            }
            BreakSwitch::PreviousProcessClaimsKept => {
                "blocks_claimed_by_a_service_process_that_is_gone_are_released_at_the_next_start"
            }
            BreakSwitch::PreviousProcessTakenForGone => {
                "blocks_claimed_by_a_service_process_still_running_are_not_released"
            }
            BreakSwitch::ClaimsKeptAtTheStop => {
                "claims_still_held_when_the_service_stops_are_released"
            }
            BreakSwitch::DispatchLinesForwarded => {
                "a_judged_block_leaves_one_summary_line_and_no_dispatch_lines_in_the_service_log"
            }
            BreakSwitch::FactsVerifierRowsNotScanned
            | BreakSwitch::FactsVerifierFrameUnderTheImageJudgementClaimKey => {
                "a_facts_verifier_pending_row_is_claimed_verified_and_written_back_by_the_serving_loop"
            }
            BreakSwitch::FactsVerifierVerifiedBlockVerifiedAgain => {
                "an_already_verified_facts_block_only_has_its_pending_row_deleted"
            }
            BreakSwitch::FactsVerifierRowGuardTakenAtTheWrite => {
                "a_facts_verifier_frame_whose_verifier_row_changed_after_it_was_read_does_not_land"
            }
            BreakSwitch::FactsVerifierPendingRowGuardTakenAtTheWrite => {
                "a_facts_verifier_frame_whose_pending_row_changed_after_it_was_read_does_not_land"
            }
            BreakSwitch::FactsVerifierGuardRefusalNotRetried => {
                "a_facts_verifier_frame_refused_once_by_each_guard_is_composed_again_and_lands"
            }
            BreakSwitch::FactsVerifierClaimsLeftAtTheRelease => {
                "facts_verifier_claims_are_released_at_the_next_start_and_at_the_stop"
            }
            BreakSwitch::FactsVerifierRowNotRereadAfterTheClaim => {
                "a_facts_verifier_block_written_back_between_the_scan_and_the_claim_is_not_taken_again"
            }
            BreakSwitch::FactsVerifierMemoryUnchecked => {
                "a_facts_verifier_job_borrows_card_memory_and_evicts_resident_judges_only_as_needed"
            }
            BreakSwitch::PackedTablesRowsNotScanned => {
                "a_packed_tables_pending_row_is_claimed_judged_and_written_back_by_the_serving_loop"
            }
            BreakSwitch::PackedTablesVerdictWordsJudgedAgain => {
                "packed_tables_whose_verdict_words_are_in_the_library_only_have_their_pending_row_deleted"
            }
            BreakSwitch::PackedTablesVerdictWordsGuardTakenAtTheWrite => {
                "a_packed_tables_frame_whose_verdict_words_row_changed_after_it_was_read_does_not_land"
            }
            BreakSwitch::PackedTablesPendingRowGuardTakenAtTheWrite => {
                "a_packed_tables_frame_whose_pending_row_changed_after_it_was_read_does_not_land"
            }
            BreakSwitch::PackedTablesGuardRefusalNotRetried => {
                "a_packed_tables_frame_refused_once_by_each_guard_is_composed_again_and_lands"
            }
            BreakSwitch::PackedTablesClaimsLeftAtTheRelease => {
                "packed_tables_claims_are_released_at_the_next_start_and_at_the_stop"
            }
            BreakSwitch::PackedTablesRowNotRereadAfterTheClaim => {
                "a_packed_tables_row_written_back_between_the_scan_and_the_claim_is_not_taken_again"
            }
            BreakSwitch::CardLeaseRequestIgnored
            | BreakSwitch::CardsLentBeforeTheBlocksInHandAreWrittenBack
            | BreakSwitch::DevicesLetGoWhileLent
            | BreakSwitch::CardKeyNotTakenBack => {
                "a_lease_request_stops_new_claims_and_the_cards_are_lent_once_the_blocks_in_hand_are_written_back"
            }
            BreakSwitch::CardKeyHeldByAnotherAtStartIgnored => {
                "a_service_starting_while_another_holds_the_card_key_starts_lent_with_its_cards_open"
            }
            BreakSwitch::CardLeasePrintedBeforeTheCardKey
            | BreakSwitch::CardLeaseReturnLeavesTheRenewerRunning => {
                "lease_take_prints_the_cards_only_once_it_holds_the_card_key_and_lease_return_lets_both_keys_go"
            }
            BreakSwitch::CardLeaseRenewerIgnoresTheBorrower => {
                "both_keys_are_let_go_within_five_seconds_once_the_borrower_is_gone"
            }
            BreakSwitch::CardLeaseKeysHeldByAnotherTakenAsClaimed => {
                "two_borrowers_on_one_host_get_the_cards_one_after_the_other"
            }
            BreakSwitch::CardLeaseBorrowerFromTheParentProcess => {
                "lease_subcommands_refuse_a_missing_or_malformed_borrower_variable_with_the_usage_exit"
            }
            BreakSwitch::CardLeaseTakenWithoutCardsOfThisHost => {
                "lease_take_refuses_with_78_when_the_configuration_names_no_card_of_this_host"
            }
            BreakSwitch::CardLeaseReturnRefusesAnEmptyCardString => {
                "lease_return_takes_an_empty_card_string_and_refuses_a_missing_one_with_the_usage_exit"
            }
            BreakSwitch::CardLeaseBorrowerWhitelistUnchecked => {
                "lease_take_refuses_a_borrower_whose_executable_name_is_not_on_the_whitelist_and_claims_no_key"
            }
            BreakSwitch::CardLeaseWaitingOrderIgnored => {
                "the_borrower_that_started_waiting_first_takes_the_cards_first_and_the_later_one_waits_second_in_the_queue"
            }
            BreakSwitch::TableStagingNotReserved => {
                "a_newly_loaded_judge_leaves_room_for_a_second_copy_of_its_table_while_it_loads"
            }
            BreakSwitch::OutOfMemoryTakenForACardFailure
            | BreakSwitch::ResidentsKeptAfterRunningOutOfMemory
            | BreakSwitch::BlocksOfTheOutOfMemoryWaveRefused
            | BreakSwitch::OutOfMemoryCaughtNotTold
            | BreakSwitch::PutBackTakesTheWholeBlock => {
                "a_card_that_ran_out_of_memory_takes_the_next_wave_and_every_block_comes_back_judged_once"
            }
            BreakSwitch::RetractedBlockSharesTheNextWave
            | BreakSwitch::OutOfMemoryOnAnEmptiedCardRetried => {
                "a_block_running_out_of_memory_again_alone_on_an_emptied_card_is_refused_and_the_card_takes_the_next_block"
            }
            BreakSwitch::FactsJobOfTheOutOfMemoryRoundRefused => {
                "a_facts_verifier_job_that_ran_out_of_memory_is_verified_again_on_an_emptied_card"
            }
            BreakSwitch::CardFailureNotTold => {
                "a_card_whose_device_is_lost_while_judging_still_fails_and_tells_one_card_failure"
            }
            BreakSwitch::JudgeLoadedNotTold => {
                "every_judge_loaded_tells_one_line_with_its_table_scratch_and_what_the_card_has_left"
            }
            BreakSwitch::CardsFailedLeftOutOfTheSummary => {
                "a_card_failure_leaves_one_card_failed_line_and_the_summary_names_the_card"
            }
            BreakSwitch::BlockRefusedForMemoryClaimedAgain
            | BreakSwitch::BlockNotJudgeableClaimKept
            | BreakSwitch::BlockNotJudgeableLineLeftOut => {
                "a_block_running_out_of_memory_on_an_emptied_card_is_named_left_to_another_service_and_not_claimed_again_here"
            }
            BreakSwitch::CardNotSettledAfterRunningOutOfMemory => {
                "a_card_that_ran_out_of_memory_waits_for_one_submission_to_finish_before_loading_again"
            }
            BreakSwitch::DeviceLostForWantOfMemoryTakenForACardFailure => {
                "a_device_lost_for_want_of_memory_is_reopened_and_the_card_takes_the_next_block"
            }
            BreakSwitch::JudgePanicTakenForACardFailure => {
                "a_judge_panicking_on_loading_refuses_its_blocks_and_every_card_keeps_judging_other_streams"
            }
            BreakSwitch::PutBackBlocksWaitForTheirOwnCard => {
                "blocks_put_back_after_running_out_of_memory_are_taken_by_another_card_that_fits"
            }
            BreakSwitch::BlocksOnlyTheFailedCardCouldHoldLeftPending => {
                "a_block_only_the_failed_big_card_could_hold_is_refused_not_left_hanging"
            }
            BreakSwitch::FactsTablesStagingNotReserved => {
                "a_facts_verifier_borrows_two_copies_of_its_tables_and_its_passes"
            }
            BreakSwitch::RegistrationLapseIgnored => {
                "a_registration_whose_lease_lapsed_drops_its_library_and_discards_its_blocks"
            }
            BreakSwitch::TemporaryLibraryFailureEndsTheService => {
                "an_unreachable_temporary_library_is_named_once_and_the_service_keeps_serving_the_real_library"
            }
            BreakSwitch::RealServiceRegistrationOpened => {
                "a_registration_pointing_at_the_real_service_or_with_a_foreign_prefix_is_not_opened"
            }
            BreakSwitch::RealLibraryClaimedFirst => {
                "candidates_of_every_library_are_claimed_in_pool_entry_order"
            }
            BreakSwitch::TemporaryLibrariesScannedWhileLent => {
                "no_library_is_scanned_while_the_cards_are_lent"
            }
            BreakSwitch::PreviousClaimsKeptInTemporaryLibraries => {
                "a_restarted_service_releases_the_claims_its_previous_process_left_in_a_temporary_library"
            }
            BreakSwitch::ConclusionWrittenIntoTheRealLibrary => {
                "a_registered_temporary_library_is_judged_and_written_back_into_that_library_only"
            }
            BreakSwitch::JudgedBlockMatchedToItsLibraryByName => {
                "blocks_with_the_same_name_in_two_libraries_are_each_written_back_into_their_own_library"
            }
        }
    }
}

/// 打开了的几项。没有不变量，空的就是平常跑。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreakSwitches(BTreeSet<BreakSwitch>);

impl BreakSwitches {
    /// 一项都不开：服务主循环交的就是它。
    #[must_use]
    pub fn none() -> Self {
        Self(BTreeSet::new())
    }

    /// 从变量的值读（`None` 是没设）：逗号隔开的项名，前后空白不算，空项不算。
    ///
    /// # Errors
    /// 有一项不是 [`BreakSwitch::name`] 里的名字：交回那一项与认得的全部项名。
    pub fn from_the_variable_value(value: Option<&str>) -> Result<Self, String> {
        let mut switches = BTreeSet::new();
        for written in value.unwrap_or("").split(',').map(str::trim) {
            if written.is_empty() {
                continue;
            }
            let switch = BreakSwitch::EVERY
                .into_iter()
                .find(|switch| switch.name() == written)
                .ok_or_else(|| {
                    let known: Vec<&str> = BreakSwitch::EVERY
                        .iter()
                        .map(|switch| switch.name())
                        .collect();
                    format!(
                        "{BREAK_VARIABLE} 里的「{written}」不是认得的项；认得的：{}",
                        known.join("、")
                    )
                })?;
            switches.insert(switch);
        }
        Ok(Self(switches))
    }

    /// 测试照这个进程的环境读。
    ///
    /// # Panics
    /// 变量里有认不得的项（写错的项不许悄悄当没开）。
    #[must_use]
    pub fn of_the_environment() -> Self {
        let value = std::env::var(BREAK_VARIABLE).ok();
        Self::from_the_variable_value(value.as_deref())
            .unwrap_or_else(|refusal| panic!("{refusal}"))
    }

    #[must_use]
    pub fn is_on(&self, switch: BreakSwitch) -> bool {
        self.0.contains(&switch)
    }
}
