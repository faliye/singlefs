//! 给定状态表那一路（[`GivenStatesEngine`]）：起步取哪几张卡、按段切块、录与判、存红与重判，
//! 连同 GPU 那一路同一刻建着的表有几张（`given_states_tables_in_flight_on_the_gpu_route`）。

use super::*;

/// 给定状态表走 GPU 判那一路同时建表、等卡的张数：只管主机内存，不进判定。
#[cfg(feature = "gpu")]
mod given_states_tables_in_flight_on_the_gpu_route {
    /// 同时在「建判表、打包、等卡、上卡判」这一段里的给定状态表不超过给的张数：一张表在这一段里占着它的判表与打包那一份
    /// （大的一张几百 MiB 到一个多 GiB 的主机内存），工作线程有几条就同时占几份；卡一次只判得了卡数那么多张，多建出来的只是占着内存等卡。
    pub(super) struct TablesBuiltAtOnce {
        inside: std::sync::Mutex<usize>,
        one_left: std::sync::Condvar,
    }

    /// 这一段里的一张表；丢掉时让出位置、叫醒等着的线程。
    pub(super) struct OneOfTheTablesBuiltAtOnce<'gate> {
        gate: &'gate TablesBuiltAtOnce,
    }

    impl TablesBuiltAtOnce {
        pub(super) const fn new() -> Self {
            Self {
                inside: std::sync::Mutex::new(0),
                one_left: std::sync::Condvar::new(),
            }
        }

        /// 这一段里已有 `at_most` 张就等到有一张出去再进。
        pub(super) fn enter(&self, at_most: usize) -> OneOfTheTablesBuiltAtOnce<'_> {
            assert!(at_most >= 1, "至少放一张表进来，不然谁都进不去");
            let mut inside = self
                .inside
                .lock()
                .expect("这把锁只护一个计数，持锁的线程不会在持锁时 panic");
            // 每有一张出去就叫醒全部等着的；循环到这一条线程抢到位置为止，出去的张数有限、等的轮数有限
            while *inside >= at_most {
                inside = self
                    .one_left
                    .wait(inside)
                    .expect("这把锁只护一个计数，持锁的线程不会在持锁时 panic");
            }
            *inside += 1;
            OneOfTheTablesBuiltAtOnce { gate: self }
        }

        /// 单测：此刻这一段里有几张表。
        #[cfg(test)]
        pub(super) fn tables_inside_now(&self) -> usize {
            *self
                .inside
                .lock()
                .expect("这把锁只护一个计数，持锁的线程不会在持锁时 panic")
        }
    }

    impl Drop for OneOfTheTablesBuiltAtOnce<'_> {
        fn drop(&mut self) {
            let mut inside = self
                .gate
                .inside
                .lock()
                .expect("这把锁只护一个计数，持锁的线程不会在持锁时 panic");
            *inside -= 1;
            self.gate.one_left.notify_all();
        }
    }

    /// 给定状态表上卡判那一段共用的那一道。
    pub(super) static GIVEN_STATES_TABLES_BUILT_AT_ONCE: TablesBuiltAtOnce =
        TablesBuiltAtOnce::new();
}

// ============================== 给定状态表：随机历史两项的入口（四件） ==============================

/// 随机历史的一项（「随机历史两项（崩溃注入默认批、读故障态不持有见证）加「给定状态表」一种展开，也是这个入口与这个判定库里的项」）：
/// 项名只进行里报的名字。块键只由表的内容定（基线、写表、接缝、持久集合、旋钮栏，按段切成几块），设点文件、项名、表名、种子与规模都不进；
/// 结论的版本由结论种类与判器引到的被检测产品代码定（`crash_identity::ConclusionVersion::of_the_given_states_judgement`），判器的代码不进键。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GivenStatesItem {
    pub item_name: String,
}

/// 往 GPU 那一侧塞的已知故障：只给证明 GPU 判与 CPU 判两趟分开跑、比结论会红的那条用例用，产品路径恒是 [`FaultPutOnTheGpuSide::NoFault`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPutOnTheGpuSide {
    NoFault,
    /// 每张表第一个状态的 GPU 结论里第一条不变量（`IMPLEMENTED_INVARIANTS[0]`）的违例位翻过来，判到位一起置上。
    FirstInvariantViolatedBitFlippedOnTheFirstStateOfEachTable,
}

/// 给定状态表这一趟的账（随 [`GivenStatesEngine::finish`] 交回，那一刻打 [`GIVEN_STATES_LINE_PREFIX`] 那一行）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GivenStatesTally {
    pub tables_judged_here: u64,
    pub tables_reused: u64,
    pub states_judged_by_the_gpu_here: u64,
    pub states_judged_by_the_cpu_here: u64,
    pub states_reused: u64,
    /// GPU 判不了的表（建表交了拒因：哪张表满了）或状态（结论里判不了那一位），一条一句：照 记一条不一致、判红。
    pub states_the_gpu_could_not_judge: Vec<String>,
    /// 两台分片时归别的机器判、这一趟库里又没有的表（「配了双机就两台分片」）：这一趟交回的是占位的结论，不作数，
    /// 要等那一台判完、写进服务那一份之后再跑一趟（[`GivenStatesRun::left_tables_to_the_other_hosts`]）。
    pub tables_left_to_the_other_hosts: u64,
    /// 这一趟只跑录入段（[`RUN_STAGE_ENVIRONMENT_VARIABLE`]`=record`）：写进库、没判的表数（交的是占位结论，[`GivenStatesRun::recorded_only`]）。
    pub tables_recorded_only: u64,
    /// 录入段写进库的表数（只录与两段先后跑都算；只判的那一趟是 0）。
    pub tables_recorded_here: u64,
    /// 配置开了 GPU 而这份二进制没编进 gpu 特性：判段不跑、交占位结论的表数（[`GivenStatesRun::not_run_without_the_gpu_feature`]）。
    pub tables_not_judged_without_the_gpu_feature: u64,
    /// 录入段录入键命中（库里已有逐字节相同的表）、不再写的表数（L11）。
    pub tables_whose_recording_was_reused: u64,
    /// 判红的表数（这一趟现判的与从库里读回的都算：表里有一个状态带红项就算这张表红）；门禁核的块判定行里的 `red`。
    pub tables_with_red_states: u64,
    /// 库里这一把判法单元指纹下结论齐全、却有一块记着红的表，这一趟不复用、现判的张数（「记红、撤绿、下趟现判」；
    /// 判完照新结论写回，也数进 `tables_judged_here`）。
    pub tables_stored_red_judged_again: u64,
}

/// 给定状态表在库里的结论记着红时，这一趟照不照它（[`GivenStatesEngine::stored_conclusions`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoredRedTableOfTheRun {
    /// 这台机器判的表：记着红的不复用，这一趟现判（「记红、撤绿、下趟现判」）。
    JudgedAgainHere,
    /// 归别的机器判的表：那一台现判过、写进服务那一份的结论照用（记着红的由那一台现判，这台不另判）。
    TakenAsTheOtherHostJudgedIt,
}

/// [`GivenStatesEngine::finish`] 交回的这一趟。
#[derive(Clone, Debug, PartialEq)]
pub struct GivenStatesRun {
    pub mode: CrashAmplificationRunMode,
    /// 这一趟跑哪一段（[`RUN_STAGE_ENVIRONMENT_VARIABLE`]）。
    pub stage: RunStage,
    pub share: HostShare,
    pub judged_by: JudgedBy,
    /// 这一趟的卡从哪来：判器走 CPU 是 `Some(`[`GpuCardsSource::NoCardNeeded`]`)`；判器走 GPU 的，第一次要在卡上判一张表时才取卡
    /// （`GivenStatesEngine::cards_taken_at_the_first_table_judged_on_the_gpu`），取到了是那一次的来路，
    /// 没取过卡（一张表都没走到卡上：只录、全部复用、归别的机器判）或取卡没成（没经服务给卡、起不了卡池）是 `None`。
    pub gpu_cards_taken_from: Option<GpuCardsSource>,
    /// 库的名字（连的是 prune-server 哪一处、哪一个键空间）：只进行里报的名字。
    pub library_name: String,
    /// 这一趟见过叫停（[`stop_is_requested_by_the_environment`]，照流水线那一侧的叫停层：见过一次就记住）：崩溃注入不再领新的一片，
    /// 判过的表在库里；调用方照「本次未跑」交回。
    pub stopped_early: bool,
    /// 配置开了 GPU 而这份二进制没编进 gpu 特性时判段不跑的原因（[`judge_not_run_without_the_gpu_feature`]）：表照录、一张不判，开跑时打了
    /// 本次未跑那一行（[`given_states_not_run_line_without_the_gpu_feature`]），调用方照 [`GivenStatesRun::recorded_only`] 交回。
    pub not_run_without_the_gpu_feature: Option<String>,
    pub tally: GivenStatesTally,
}

impl GivenStatesRun {
    /// 这一趟有表归别的机器判、库里又没有（两台分片、那一台还没判完写进服务）：交回的结论里有占位的，调用方不照它判，照「本次未跑」交回。
    #[must_use]
    pub fn left_tables_to_the_other_hosts(&self) -> bool {
        self.tally.tables_left_to_the_other_hosts > 0
    }

    /// 这一趟只跑了录入段（[`RUN_STAGE_ENVIRONMENT_VARIABLE`]`=record`，或配置开了 GPU 而这份二进制没编进 gpu 特性、判段不跑）：
    /// 表都写进了库、一张没判，交回的结论是占位的，调用方不照它判，照「本次未跑」交回，等判段那一趟（`judge`，或不设、两段先后跑；
    /// 配了 GPU 的是带特性那一趟）从库里读回判。
    #[must_use]
    pub fn recorded_only(&self) -> bool {
        self.stage == RunStage::RecordOnly || self.not_run_without_the_gpu_feature.is_some()
    }

    /// 这一趟判红的那几样（GPU 判不了）一条一句；空就是不红（叫停、分片留给别的机器另看 [`GivenStatesRun::stopped_early`]、
    /// [`GivenStatesRun::left_tables_to_the_other_hosts`]）。
    #[must_use]
    pub fn red_items(&self) -> Vec<String> {
        self.tally
            .states_the_gpu_could_not_judge
            .iter()
            .map(|state| format!("GPU 判不了：{state}"))
            .collect()
    }
}

/// 给定状态表那一路每一趟收尾打的一行的头（项名、判器、对照、表数与态数、比不上与判不了的条数）。
pub const GIVEN_STATES_LINE_PREFIX: &str = "CRASH_POINT_REPLAY_GIVEN_STATES";

/// 配了 GPU 而给定状态表的判器借不到显存时打的那一行的头（运行条件不满足，入口照它退 78；不打「本次未跑」那一行、不换成 CPU 判器的结论往下走）。
pub const GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX: &str =
    "CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET";

/// 给定状态表交崩溃点重放的引擎判：照本机多机配置的 GPU 开关选路（没配走 CPU 判器、配了走 GPU 判器，
/// 卡用不上、借不到显存都报错、不退回 CPU）；两台分片照流水线那一侧的做法（`SINGLEFS_CRASH_AMPLIFICATION_SHARE` 领哪一份、
/// `…_HOST_WEIGHTS` 权重，一张表一块、按块键的输入指纹归一台；两台各判各领的表、直接写进 prune-server 那一份，别的机器判过的表照库复用）；叫停照流水线那一侧的叫停层
/// （[`stop_is_requested_by_the_environment`]，见过一次就记住，崩溃注入照它不再领新的一片）；配了 GPU 只由 GPU 判器判、没配只由 CPU 判器判，
/// 每张表先写进判定库、再从库里读回来判（一段里 CPU 不既跑数据又对拍，段与段之间不在内存里交数据）；结论按（块键，结论种类，版本）存进判定库，一张表按段切成几块（H8「判定按段存」、L6：每个状态归它第一次有写没持久的那一段，全持久的一块），
/// 同一把版本下判过的表照库里的复用。GPU 判不了的表（建不出表：掩码装不下、落点版本太多……）照 H6 记一条不一致、判红，这几个状态的结论取 CPU 判器的
/// （崩溃注入照它往下走，这一趟已经红了）。几条崩溃注入的工作线程共用一个：库与账各一把锁。
pub struct GivenStatesEngine {
    item: GivenStatesItem,
    /// 这一趟跑哪一段（「给定状态引擎认选段」）：`record` 只把表写进库、按项名与表名登清单，`judge` 只凭项名与表名从库里读回判，
    /// 不设两段先后跑、中间照样经库。
    stage: RunStage,
    mode: CrashAmplificationRunMode,
    pub(super) judged_by: JudgedBy,
    /// 配置开了 GPU 而这份二进制没编进 gpu 特性：判段不跑（表照录、交占位结论），见 [`GivenStatesRun::not_run_without_the_gpu_feature`]。
    not_run_without_the_gpu_feature: Option<String>,
    /// 给定状态判定的版本（[`ConclusionVersion::of_the_given_states_judgement`]：由结论种类与判器引到的被检测产品代码定，判器的代码不进），GPU 判的与 CPU 判的都记在它下面。
    judgement_version: ConclusionVersion,
    library: LibraryOpenInThisProcess,
    pub(super) library_name: String,
    /// 判器走哪一路（起步时照配置定）：走 CPU 的这一趟不要卡。
    crash_judge_route: ComputedOn,
    /// 起步那一刻环境里的显卡来路（只记、不领卡）：第一次要在卡上判一张表时照它取卡。
    #[cfg_attr(
        not(feature = "gpu"),
        allow(
            dead_code,
            reason = "没编进 gpu 特性时选路走不到 GPU，取卡只在编进来时做"
        )
    )]
    card_sources_of_the_start: CardSourcesOfTheEngineStart,
    /// 第一次要在卡上判一张表时取的卡（`Self::cards_taken_at_the_first_table_judged_on_the_gpu`）：起步时不领卡、不起卡池，
    /// 只录的那一趟、一张表都没走到卡上的那一趟一直是空的；取卡没成的那一句记下，之后每张要上卡的表交同一句。
    cards_taken_at_the_first_table_judged_on_the_gpu:
        std::sync::OnceLock<Result<CardsTakenByTheGivenStatesEngine, String>>,
    /// 取卡没成（调用方没交卡）的那几张表经哪一条路交 GPU 批处理服务判（产品是起步时取的控制命令，单测换成替身）。
    #[cfg(feature = "gpu")]
    way_to_the_gpu_verification_server: WayToTheGpuVerificationServer,
    /// 上卡判那一段同时建判表、等卡的表的那一道（产品是全进程共用的
    /// `given_states_tables_in_flight_on_the_gpu_route::GIVEN_STATES_TABLES_BUILT_AT_ONCE`，单测换成自己的一道、数里面有几张）。
    #[cfg(feature = "gpu")]
    tables_built_at_once:
        &'static given_states_tables_in_flight_on_the_gpu_route::TablesBuiltAtOnce,
    share: HostShare,
    host_shares: WeightedShares,
    stop_request: StopRequestRememberedInThisRun<'static>,
    /// 崩溃注入问叫停时这一趟答过「叫停了」（之后它不再领新的一片）：收尾照它报 `stopped_early`，不在收尾那一刻再读一次叫停文件。
    answered_a_stop_request: std::sync::atomic::AtomicBool,
    #[cfg_attr(
        not(feature = "gpu"),
        allow(
            dead_code,
            reason = "没编进 gpu 特性时 GPU 那一侧不判，塞故障只在编进来时用"
        )
    )]
    fault_put_on_the_gpu_side: FaultPutOnTheGpuSide,
    tally: std::sync::Mutex<GivenStatesTally>,
}

/// 判段起步那一刻环境里的显卡来路与多机配置：服务交的卡串（`GPU_CARDS_BY_PRIORITY`）、清没清场，只记、不取卡。给定状态引擎起步、
/// 崩溃放量一趟与崩溃枚举用例的判段入口各记一份。第一次要在卡上判一张表、一张盘时照它取卡（`Self::cards_by_priority`）：照的是起步那一刻的环境
/// （用例给这一趟设的临时配置；只读环境起引擎的用例起完就放改环境的锁，[`one_test_changes_the_environment_at_a_time`]），不是判表那一刻的。
/// 借卡通道不记：判段取卡不问借卡通道、不借卡（借卡只给判段之外要卡的进程）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时选路走不到 GPU，取卡只在编进来时做"
    )
)]
pub(crate) struct CardSourcesOfTheEngineStart {
    pub(super) configuration_file: PathBuf,
    pub(super) cards_handed_out_by_the_service: Option<String>,
    quiesce: crate::gpu_card_configuration::GpuQuiesce,
}

impl CardSourcesOfTheEngineStart {
    /// 此刻环境里的那几样，连同这一趟的多机配置（读环境，不取卡）。
    pub(crate) fn read_from_the_environment_now(configuration_file: PathBuf) -> Self {
        Self {
            configuration_file,
            cards_handed_out_by_the_service: std::env::var(
                crate::gpu_card_configuration::CARDS_BY_PRIORITY_KEY,
            )
            .ok(),
            quiesce: crate::gpu_card_configuration::GpuQuiesce::of_this_process(),
        }
    }

    /// 显卡的先后与额度（同 [`cards_by_priority_from_configuration_or_environment`]，环境那一层读的是起步时记下的这一份）：
    /// 服务交了卡串照它；没交不借卡，照 [`crate::gpu_card_configuration::cards_by_priority_from`] 判没经服务给卡。
    ///
    /// # Errors
    /// 同 [`crate::gpu_card_configuration::cards_by_priority_from`]。
    #[cfg(feature = "gpu")]
    pub(super) fn cards_by_priority(&self) -> Result<Option<ConfiguredCards>, String> {
        crate::gpu_card_configuration::cards_by_priority_from(
            self.cards_handed_out_by_the_service.clone(),
            Some(self.configuration_file.as_path()),
            self.quiesce,
            &crate::gpu_card_configuration::multi_host_addresses_of_the_caller,
        )
    }

    /// 这一趟在几张卡上判，取不小于真张数的那个数，只数、不领卡、不起卡池（给定状态表上卡判那一段限流用：同时建判表、等卡的表不超过它加一张，
    /// `given_states_tables_in_flight_on_the_gpu_route::GIVEN_STATES_TABLES_BUILT_AT_ONCE`；只管主机内存，偏松不改判定）。
    /// `cards_in_the_card_pool_started_in_this_process` 是这个进程已经起好的卡池里几张卡，没起过是 `None`
    /// （`crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process`，只看、不起池）。取卡一律照这个进程的卡池（[`gpu_cards_in_use`]），
    /// 池里的卡要与这一趟的显卡配置里这一台那几项逐张对得上（[`the_card_pool_matches_the_configured_cards`]），对不上取卡就报错、这张表到不了卡上。
    /// - 卡池已经起好：池里几张就是几张。
    /// - 卡池没起、服务起跑交了卡串（`GPU_CARDS_BY_PRIORITY`）：卡串里的项数（两台的都算），不小于这一台那几项、也就不小于池里的张数。
    /// - 卡池没起、没交卡串（没设、空串或只有空白，[`crate::gpu_card_configuration::card_string_handed_over_by_the_service`]）：0
    ///   （判段不借卡）。判器走 GPU 的这一趟配置开着 GPU，没经服务给卡，取卡照 [`crate::gpu_card_configuration::cards_by_priority_from`] 报错。
    #[cfg(feature = "gpu")]
    fn card_count_without_taking_a_card(
        &self,
        cards_in_the_card_pool_started_in_this_process: Option<usize>,
    ) -> usize {
        if let Some(cards_in_the_pool) = cards_in_the_card_pool_started_in_this_process {
            return cards_in_the_pool;
        }
        // 空白卡串数出来是 0 项，与没交同一个数（card_string_handed_over_by_the_service 当它没交）
        self.cards_handed_out_by_the_service
            .as_deref()
            .map_or(0, |cards_handed_out| {
                cards_handed_out.split_whitespace().count()
            })
    }
}

/// 给定状态引擎第一次要在卡上判一张表时取到的卡（`GivenStatesEngine::cards_taken_at_the_first_table_judged_on_the_gpu`）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(dead_code, reason = "没编进 gpu 特性时不在卡上判，取卡只在编进来时做")
)]
struct CardsTakenByTheGivenStatesEngine {
    /// 这一趟自己起的卡（卡来自卡池时是空的：借到哪张才在哪张上起）。
    cards: CardsInUse,
    taken_from: GpuCardsSource,
    /// 这一趟自己起的卡（[`GpuCardsSource::DevicesStartedForThisRun`]）上同时活着的判器按显存排队（几条工作线程同时交表，
    /// 每个判器照借的那么多开草稿区；不排队时各自照整张卡的额度开、叠起来撞顶）；卡来自卡池时是空的、用不上。
    memory_queue_of_the_cards_started_for_this_run: CardMemoryQueue,
}

/// GPU 判器判一张给定状态表的结局。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时不在卡上判，这几种结局只在编进来时造"
    )
)]
enum GivenStatesOnTheGpu {
    /// 判完：逐状态的对拍字（oracle 的位已去掉）、池级 checker 判 I-7.4 时违例细节里点名的被抛弃根、判不了的那几个状态判不了的原因
    /// （原因码与站点号，`crate::crash_judge_gpu::undecidable_reasons_of_the_gpu_verdict`；判得了的是 None）与在哪张卡上判的。
    Judged {
        words: Vec<CrossCheckWords>,
        abandoned_roots_named_of_each_state: Vec<Vec<AbandonedRootNamedByThePoolChecker>>,
        undecidable_reason_of_each_state: Vec<Option<String>>,
        judged_on: JudgingDevice,
    },
    /// 建不出这张表（[`crate::crash_judge_tables::JudgeTablesRefusal`] 的写法）：H6 的「哪张表满了」。
    TablesRefused { refusal: String },
}

impl GivenStatesEngine {
    /// 照环境起一趟（读法同 [`run_from_environment`]：多机配置、显卡先后；库是这个进程的那一份）。
    ///
    /// # Errors
    /// 同 [`GivenStatesEngine::start_from_environment_putting_a_fault_on_the_gpu_side`]。
    pub fn start_from_environment(
        item: GivenStatesItem,
        toolchain: &CrashPointReplayToolchainIdentity,
    ) -> Result<Self, String> {
        Self::start_from_environment_putting_a_fault_on_the_gpu_side(
            item,
            toolchain,
            FaultPutOnTheGpuSide::NoFault,
        )
    }

    /// 同 [`GivenStatesEngine::start_from_environment`]，GPU 那一侧塞 `fault`（证明 GPU 与 CPU 两趟分开跑、比结论会红的那条用例用）。
    ///
    /// 起步不领卡、不起卡池：判器走 GPU 的，第一次要在卡上判一张表时才取卡（`Self::cards_taken_at_the_first_table_judged_on_the_gpu`），
    /// 调用方没交卡（起步时环境里没有服务交的卡串，取卡报没经服务给卡）的表交 GPU 批处理服务判
    /// （[`judge_packed_tables_through_the_gpu_verification_server`]），交了卡串而取卡没成的在那张表上报错，都不退回 CPU；
    /// 判段取卡不问借卡通道、不借卡。
    ///
    /// # Errors
    /// 多机配置读不了或写坏了；配置开了 GPU 而这份二进制没有 `gpu` 特性；库开不了；分片与权重写法错
    /// （[`host_share_and_weights_from_the_environment`]）；连不上 prune-server（全仓那一份照退出码退出进程）。
    ///
    /// `_toolchain` 这一路不读：结论的版本由结论种类与判器引到的被检测产品代码定、不取判法单元键，签名照旧留给几十处调用方。
    pub fn start_from_environment_putting_a_fault_on_the_gpu_side(
        item: GivenStatesItem,
        _toolchain: &CrashPointReplayToolchainIdentity,
        fault: FaultPutOnTheGpuSide,
    ) -> Result<Self, String> {
        let stage = RunStage::from_the_environment()?;
        let MultiHostConfigurationOfThisProcess {
            configuration_file,
            mode,
        } = multi_host_configuration_of_this_process()?;
        // 配了 GPU 而这份二进制没编进 gpu 特性（不带特性编的那一趟）：判段这一趟不跑、报本次未跑（不判红、不退回 CPU 判），录入段照录、
        // 表写进库，带特性那一趟从库里读回、在卡上判（「配 GPU 必用 GPU、做不到报错不降级」，与 run_from_environment 同一种写法）
        let not_run_without_the_gpu_feature =
            judge_not_run_without_the_gpu_feature(mode.uses_gpu(), cfg!(feature = "gpu"));
        if let Some(reason) = &not_run_without_the_gpu_feature {
            println!(
                "{}",
                given_states_not_run_line_without_the_gpu_feature(&item.item_name, reason)
            );
        }
        let routes = computation_routes(
            mode.uses_gpu() && not_run_without_the_gpu_feature.is_none(),
            cfg!(feature = "gpu"),
        )?;
        // 显卡的来路起步时只记、不取（同 run_from_environment：录的时候不领卡、不起卡池）：第一次要在卡上判一张表时才照它取
        // （cards_taken_at_the_first_table_judged_on_the_gpu），没经服务给卡、没清场而配置开着 GPU 的那一句也到那时才报
        let card_sources_of_the_start =
            CardSourcesOfTheEngineStart::read_from_the_environment_now(configuration_file);
        let (share, host_weights) = host_share_and_weights_from_the_environment()?;
        // 配了 GPU 只由 GPU 判器判，没配只由 CPU 判器判：同一批状态这一趟只判一次，不在同一段里对拍；判段不跑的那一趟记配置的那一路（GPU）
        let judged_by = match (routes.crash_judge, &not_run_without_the_gpu_feature) {
            (ComputedOn::Cpu, None) => JudgedBy::CpuJudge,
            (ComputedOn::Gpu, None) | (ComputedOn::Cpu | ComputedOn::Gpu, Some(_)) => {
                JudgedBy::GpuJudge
            }
        };
        // 库：prune-server 里那一份（几类测试同一份，不收选库的变量）
        let library = LibraryOpenInThisProcess::open_or_share()?;
        let library_name = library.lock().library_name().to_string();
        let judgement_version = ConclusionVersion::of_the_given_states_judgement();
        Ok(Self {
            item,
            stage,
            mode,
            judged_by,
            not_run_without_the_gpu_feature,
            judgement_version,
            library,
            library_name,
            crash_judge_route: routes.crash_judge,
            card_sources_of_the_start,
            cards_taken_at_the_first_table_judged_on_the_gpu: std::sync::OnceLock::new(),
            #[cfg(feature = "gpu")]
            way_to_the_gpu_verification_server: WayToTheGpuVerificationServer::TheRealService,
            #[cfg(feature = "gpu")]
            tables_built_at_once:
                &given_states_tables_in_flight_on_the_gpu_route::GIVEN_STATES_TABLES_BUILT_AT_ONCE,
            share,
            host_shares: WeightedShares::new(host_weights),
            stop_request: stop_request_remembering_the_stop_file_of_the_engine_start(),
            answered_a_stop_request: std::sync::atomic::AtomicBool::new(false),
            fault_put_on_the_gpu_side: fault,
            tally: std::sync::Mutex::new(GivenStatesTally::default()),
        })
    }

    /// 第一次要在卡上判一张表时取卡（照起步时记下的显卡来路，`CardSourcesOfTheEngineStart::cards_by_priority`；取卡经
    /// [`gpu_cards_in_use`]，这个进程的卡池没起过就在这时起），之后每张要上卡的表交同一份；取卡没成的那一句同样记下、之后每次交它。
    /// 几条工作线程同时走到这里时只有一条取，别的等它取完。
    ///
    /// # Errors
    /// 取卡没成：没经服务给卡而配置开着 GPU、起不了卡池或卡池里的卡与配置对不上、本机没有用得了的卡。
    #[cfg(feature = "gpu")]
    fn cards_taken_at_the_first_table_judged_on_the_gpu(
        &self,
    ) -> Result<&CardsTakenByTheGivenStatesEngine, String> {
        self.cards_taken_at_the_first_table_judged_on_the_gpu
            .get_or_init(|| {
                let cards_by_priority = self.card_sources_of_the_start.cards_by_priority()?;
                let GpuCardsOfThisRun { cards, taken_from } =
                    gpu_cards_in_use(cards_by_priority.as_ref())?;
                let memory_queue_of_the_cards_started_for_this_run =
                    memory_queue_of_the_cards_started_for_this_run(Some(&cards));
                Ok(CardsTakenByTheGivenStatesEngine {
                    cards,
                    taken_from,
                    memory_queue_of_the_cards_started_for_this_run,
                })
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    /// 单测：取卡没成的表交替身服务判（[`WayToTheGpuVerificationServer::StandIn`]），不登记临时库。
    #[cfg(all(feature = "gpu", test))]
    fn going_to_the_gpu_verification_server_through_the_stand_in(
        mut self,
        stand_in: std::sync::Arc<GpuVerificationServerStandIn>,
    ) -> Self {
        self.way_to_the_gpu_verification_server = WayToTheGpuVerificationServer::StandIn(stand_in);
        self
    }

    /// 单测：上卡判那一段同时建判表、等卡的表照 `tables_built_at_once` 那一道限（单测自己的一道，数得出里面有几张，不与别的用例共用）。
    #[cfg(all(feature = "gpu", test))]
    fn limiting_its_tables_built_at_once_with(
        mut self,
        tables_built_at_once: &'static given_states_tables_in_flight_on_the_gpu_route::TablesBuiltAtOnce,
    ) -> Self {
        self.tables_built_at_once = tables_built_at_once;
        self
    }

    /// 收尾：关库，打 [`GIVEN_STATES_LINE_PREFIX`] 那一行，交回这一趟。
    #[must_use]
    pub fn finish(self) -> GivenStatesRun {
        let tally = self
            .tally
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let stopped_early = self
            .answered_a_stop_request
            .load(std::sync::atomic::Ordering::SeqCst);
        drop(self.library);
        println!(
            "{GIVEN_STATES_LINE_PREFIX} item={} stage={} mode={} judged_by={} tables_recorded_here={} tables_recorded_only={} tables_judged_here={} tables_reused={} states_judged_by_the_gpu_here={} states_judged_by_the_cpu_here={} states_reused={} states_the_gpu_could_not_judge={} share={}/{} tables_left_to_the_other_hosts={} stopped_early={} tables_whose_recording_was_reused={}",
            self.item.item_name,
            self.stage.name(),
            self.mode.name(),
            self.judged_by.conclusion_kind().version_text(),
            tally.tables_recorded_here,
            tally.tables_recorded_only,
            tally.tables_judged_here,
            tally.tables_reused,
            tally.states_judged_by_the_gpu_here,
            tally.states_judged_by_the_cpu_here,
            tally.states_reused,
            tally.states_the_gpu_could_not_judge.len(),
            self.share.index,
            self.share.count,
            tally.tables_left_to_the_other_hosts,
            stopped_early,
            tally.tables_whose_recording_was_reused
        );
        // 门禁 crash-point-replay 对每条 crash-case 核的块判定行（接口在 scripts/gate/admission.py 文件头，崩溃枚举那一路由
        // crash_state_enumeration 打）：给定状态这一路的「块」是表，blocks = reused + fresh + refused；判器的代码不进键、没有拒算，refused 恒为 0
        println!(
            "{} blocks={} reused={} fresh={} red={} refused=0",
            crate::crash_state_enumeration::CRASH_CASE_BLOCKS_LINE_PREFIX,
            tally.tables_reused + tally.tables_judged_here,
            tally.tables_reused,
            tally.tables_judged_here,
            tally.tables_with_red_states
        );
        GivenStatesRun {
            mode: self.mode,
            stage: self.stage,
            share: self.share,
            judged_by: self.judged_by,
            gpu_cards_taken_from: match self.crash_judge_route {
                ComputedOn::Cpu => Some(GpuCardsSource::NoCardNeeded),
                ComputedOn::Gpu => self
                    .cards_taken_at_the_first_table_judged_on_the_gpu
                    .get()
                    .and_then(|taken| taken.as_ref().ok())
                    .map(|taken| taken.taken_from),
            },
            library_name: self.library_name,
            stopped_early,
            not_run_without_the_gpu_feature: self.not_run_without_the_gpu_feature,
            tally,
        }
    }

    /// 这张表归不归这台机器判：这张表除持久集合之外那几样的摘要（[`frame_digest_of_a_given_states_table`]）头 8 个字节当块号，
    /// 照几台机器的权重平滑轮转（[`WeightedShares::owner_of_block`]，与流水线那一侧同一种分法；几台各算各的、不用商量）。
    fn owns_the_table(&self, frame_digest: &[u8; DIGEST_BYTES]) -> bool {
        let block_number = u64::from_le_bytes(
            frame_digest[..8]
                .try_into()
                .expect("表的摘要是 SHA-256，头 8 个字节总在"),
        );
        self.host_shares.owner_of_block(block_number) == self.share.index
    }

    /// 这一种判器判出的结论记在哪一把版本下：两种判器都是给定状态判定那一把（在哪判的只进块属性）。
    fn version_of(&self, _judged_by_either_judge: JudgedBy) -> ConclusionVersion {
        self.judgement_version
    }

    fn note(&self, change: impl FnOnce(&mut GivenStatesTally)) {
        change(
            &mut self
                .tally
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
    }

    fn store(&self) -> crate::crash_library::LibraryLockHeld<'_> {
        self.library.lock()
    }

    /// 一张给定状态表按段切成的块（L6：给定状态表不再一张一块）：每个状态归它第一次有写没持久的那一段（全持久的归一块「全部持久」），
    /// 几个状态同一段的进同一块，块按段号排、块里按状态在表里的次序。块键只由内容定：段写表与起步镜像那一半是这张表除持久集合之外那几样
    /// （基线、写表连同段号、接缝、旋钮栏）的摘要连这一段的段号，计划哈希那一半是这一块每个状态的持久集合与它那一栏恢复自称的那一版；
    /// 项名、表名、设点文件、种子与规模都不进。
    fn blocks_of_the_table(
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Vec<BlockOfAGivenStatesTable> {
        let table = table_with_its_columns.table;
        let frame_digest = frame_digest_of_a_given_states_table(table_with_its_columns);
        let mut states_of_each_group: BTreeMap<SegmentGroupOfAGivenState, Vec<usize>> =
            BTreeMap::new();
        for (position, persisted) in table.persisted_sets.iter().enumerate() {
            states_of_each_group
                .entry(segment_group_of_a_given_state(table.segments, persisted))
                .or_default()
                .push(position);
        }
        states_of_each_group
            .into_iter()
            .map(|(group, positions)| {
                let mut plan_text = format!("given-states-block-v1\ngroup={}\n", group.name());
                for position in &positions {
                    plan_text.push_str("persisted=");
                    plan_text.extend(table.persisted_sets[*position].iter().map(|is_persisted| {
                        if *is_persisted {
                            '1'
                        } else {
                            '0'
                        }
                    }));
                    plan_text.push('\n');
                    plan_text.push_str(&claimed_version_line_of_the_block_key(
                        table_with_its_columns
                            .claimed_versions
                            .of_the_state(*position),
                    ));
                    plan_text.push('\n');
                }
                let mut node_input = b"given-states-segment-v1\n".to_vec();
                node_input.extend_from_slice(&frame_digest);
                node_input.extend_from_slice(format!("group={}\n", group.name()).as_bytes());
                BlockOfAGivenStatesTable {
                    key: VerdictBlockKey {
                        node_input_digest: NodeInputDigest(sha256_digest(&node_input)),
                        enumeration_plan_hash: EnumerationPlanHash(sha256_digest(
                            plan_text.as_bytes(),
                        )),
                        block_start: BlockStartStateIndex(0),
                    },
                    positions,
                }
            })
            .collect()
    }

    /// 库里这张表每一块在 `judged_by` 那一把判法单元指纹下的结论，按状态在表里的次序拼回逐状态的对拍字（判到位不存，读回是 0）与计数向量
    /// （各块的计数向量相加）；有一块没存过是 `None`。这台机器判的表有一块记着红（`stored_red` 是
    /// [`StoredRedTableOfTheRun::JudgedAgainHere`]）同样是 `None`：这一趟现判，记进账的 `tables_stored_red_judged_again`。
    ///
    /// # Errors
    /// 库读失败；存着的块不是这一路写得出的（红项认不得、没带恢复字、状态数与块不等）。
    fn stored_conclusions(
        &self,
        blocks: &[BlockOfAGivenStatesTable],
        judged_by: JudgedBy,
        state_count: usize,
        stored_red: StoredRedTableOfTheRun,
    ) -> Result<Option<GivenStatesConclusionsOfATable>, String> {
        let version = self.version_of(judged_by);
        let mut states_of_each_position: Vec<Option<GivenStateConclusion>> =
            vec![None; state_count];
        let mut count_vectors = Vec::with_capacity(blocks.len());
        let mut table_is_red = false;
        for block in blocks {
            let Some(stored) = self
                .store()
                .read_conclusion_block(&block.key, &judged_by.conclusion_kind(), &version)
                .map_err(|error| format!("读不了给定状态表的结论：{error:?}"))?
            else {
                return Ok(None);
            };
            crate::crash_library::assert_this_thread_holds_no_library_lock("拼给定状态表的结论");
            if !stored.violations.is_empty() {
                match stored_red {
                    StoredRedTableOfTheRun::JudgedAgainHere => {
                        self.note(|tally| tally.tables_stored_red_judged_again += 1);
                        return Ok(None);
                    }
                    StoredRedTableOfTheRun::TakenAsTheOtherHostJudgedIt => {}
                }
            }
            let numbers = stored.block.verdict_vector_number_of_each_state();
            let recovery_words = stored
                .recovery_words
                .as_ref()
                .ok_or_else(|| "库里给定状态表的结论没带恢复字".to_string())?;
            let abandoned_roots_named_of_each_state = stored
                .abandoned_roots_named_by_the_pool_checker
                .as_ref()
                .ok_or_else(|| "库里给定状态表的结论没带 I-7.4 点名的被抛弃根".to_string())?;
            let block_states = block.positions.len();
            if numbers.len() != block_states
                || recovery_words.len() != block_states
                || abandoned_roots_named_of_each_state.len() != block_states
            {
                return Err(format!(
                    "库里给定状态表的一块结论有 {} 个状态、{} 份恢复字、{} 份点名的被抛弃根，这一块 {block_states} 个状态",
                    numbers.len(),
                    recovery_words.len(),
                    abandoned_roots_named_of_each_state.len()
                ));
            }
            let verdict_vectors_of_the_block = {
                let store = self.store();
                numbers
                    .iter()
                    .map(|number| {
                        store
                            .verdict_vector_of_number(*number)
                            .cloned()
                            .ok_or_else(|| format!("库里没有判定向量 {}", number.as_u32()))
                    })
                    .collect::<Result<Vec<EncodedVerdictVector>, String>>()?
            };
            for (((vector, words), abandoned_roots_named), position) in verdict_vectors_of_the_block
                .iter()
                .zip(recovery_words)
                .zip(abandoned_roots_named_of_each_state)
                .zip(&block.positions)
            {
                states_of_each_position[*position] = Some(GivenStateConclusion {
                    words: cross_check_words_of_a_stored_given_state(
                        &red_items_of_the_encoded_verdict(vector),
                        *words,
                    )?,
                    pool_checker_violation_details: Vec::new(),
                    abandoned_roots_named_by_the_pool_checker: abandoned_roots_named.clone(),
                });
            }
            table_is_red |= !stored.violations.is_empty();
            count_vectors.push(stored.count_vector);
        }
        if table_is_red {
            self.note(|tally| tally.tables_with_red_states += 1);
        }
        let states = states_of_each_position
            .into_iter()
            .map(|state| state.expect("块按段把表里每个状态分进了恰好一块"))
            .collect();
        Ok(Some(GivenStatesConclusionsOfATable {
            states,
            count_vector: sum_of_the_count_vectors(&count_vectors),
            source: GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost,
        }))
    }

    /// 一张表在 `judged_by` 那一把判法单元指纹下的结论按段存成几块（每块的红项、违例状态、计数向量、恢复字与在哪判的一批写下去）。
    ///
    /// # Errors
    /// 判定向量超过 2^32 种、这个（块键，种类，版本）下已经存着不同的结论、库写失败。
    fn store_the_conclusions(
        &self,
        blocks: &[BlockOfAGivenStatesTable],
        judged_by: JudgedBy,
        conclusions: &GivenStatesConclusionsOfATable,
        judged_on: JudgingDevice,
    ) -> Result<(), String> {
        let version = self.version_of(judged_by);
        let mut table_is_red = false;
        for block in blocks {
            crate::crash_library::assert_this_thread_holds_no_library_lock(
                "拼给定状态表一块要存的结论",
            );
            let mut encoded_verdicts = Vec::with_capacity(block.positions.len());
            let mut violating_states = Vec::new();
            for (offset, position) in block.positions.iter().enumerate() {
                let items = red_items_of_a_given_state(&conclusions.states[*position].words);
                if !items.is_empty() {
                    violating_states.push(StateOffsetInBlock(
                        u64::try_from(offset).expect("块内序号装得进 u64"),
                    ));
                }
                encoded_verdicts.push(encoded_verdict_of_the_red_items(&items));
            }
            table_is_red |= !violating_states.is_empty();
            let block_states: Vec<&GivenStateConclusion> = block
                .positions
                .iter()
                .map(|position| &conclusions.states[*position])
                .collect();
            let words_of_the_block: Vec<CrossCheckWords> =
                block_states.iter().map(|state| state.words).collect();
            let abandoned_roots_named_of_each_state: Vec<Vec<AbandonedRootNamedByThePoolChecker>> =
                block_states
                    .iter()
                    .map(|state| state.abandoned_roots_named_by_the_pool_checker.clone())
                    .collect();
            let count_vector = count_vector_of_the_given_states(&words_of_the_block);
            let recovery_words: Vec<RecoveryWordsOfState> = words_of_the_block
                .iter()
                .map(recovery_words_of_the_cross_check_words)
                .collect();
            let mut store = self.store();
            let mut numbers = Vec::with_capacity(encoded_verdicts.len());
            for encoded_verdict in &encoded_verdicts {
                numbers.push(
                    store
                        .register_verdict_vector(encoded_verdict)
                        .map_err(|error| format!("登记不了判定向量：{error:?}"))?,
                );
            }
            let verdicts = VerdictBlock::new(numbers)
                .map_err(|_empty| "给定状态表的一块一个状态都没有，不存".to_string())?;
            store
                .store_conclusion_block_naming_the_abandoned_roots(
                    &block.key,
                    &judged_by.conclusion_kind(),
                    &version,
                    &ConclusionBlockRecord {
                        block: verdicts,
                        violating_states,
                        count_vector,
                        judged_on: judged_on.clone(),
                        recovery_words: Some(recovery_words),
                    },
                    &abandoned_roots_named_of_each_state,
                )
                .map_err(|error| format!("存不了给定状态表的结论：{error:?}"))?;
        }
        if table_is_red {
            self.note(|tally| tally.tables_with_red_states += 1);
        }
        Ok(())
    }

    /// GPU 判器判一张给定状态表：照 mkfs 之后的基线与这张表的写表、切段、接缝建表（每段不展开，状态是给定的持久掩码，按枚举写表补齐撕裂镜像
    /// 与重放那几位：随机历史不摆撕裂态，全是没持久），在一张卡上判（这一趟自己起的卡按显存排进第一张放得下的，
    /// [`judge_on_a_card_started_for_this_run_queued_by_memory`]；起过卡池的照账借第一张此刻放得下的，此刻都放不下而这个进程别的线程借着卡或正在借就排队等，
    /// `crate::gpu_unit_checks::gpu_test_start::GpuCardPool::lease_the_first_card_with_room_waiting_for_the_other_threads`）。
    /// 取卡没成（`Self::cards_taken_at_the_first_table_judged_on_the_gpu`：调用方没交卡）的，读回的判表交 GPU 批处理服务判
    /// （[`WayToTheGpuVerificationServer::judge_the_packed_tables`]，内核源码照这张表那一栏旋钮取），读回的结论字照本进程上卡那一支同一套解。
    ///
    /// # Errors
    /// 自己起的卡：同 [`judge_on_a_card_started_for_this_run_queued_by_memory`]。起过卡池的进程借不到判器的显存（此刻借不到又没有可等的、借到的卡上起设备时显存不够：运行条件不满足，打
    /// [`GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX`] 那一行，不报本次未跑、不换成 CPU 判器的结论往下走）、要的比每张卡空着时能借的都大、
    /// 卡上的显存读不出；这一趟没有起好的卡；调用方交了卡串而第一次上卡时取卡没成（`Self::cards_taken_at_the_first_table_judged_on_the_gpu`）；
    /// 调用方没交卡、交服务判那一条报的错（[`judge_packed_tables_through_the_gpu_verification_server`]）。
    #[cfg(feature = "gpu")]
    fn judge_the_table_on_the_gpu(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesOnTheGpu, String> {
        use crate::crash_judge_conclusions::cross_check_words_without_the_oracle;
        use crate::crash_judge_dispatch::{
            mebibytes_a_judge_needs, states_per_scratch_lane, GpuCrashJudge, PackedJudgeTables,
        };
        use crate::crash_judge_gpu::{
            abandoned_roots_named_by_the_pool_checker_of_the_gpu_verdict,
            cross_check_words_of_gpu_verdict, undecidable_reasons_of_the_gpu_verdict,
            JudgeKernelSources,
        };
        let table = table_with_its_columns.table;
        // 同时建判表、等卡的表不超过卡数加一张（一张在建、其余在卡上）：工作线程多于卡时，多出来的线程在这里等，不先把判表建出来占着主机内存等卡。
        // 卡数照起步时记下的显卡来路与已经起好的卡池数，不为数张数取卡、起卡池（服务交了卡串取项数、没交取 0、卡池起好了取池里的张数，
        // CardSourcesOfTheEngineStart::card_count_without_taking_a_card）
        let cards_this_run_judges_on = self
            .card_sources_of_the_start
            .card_count_without_taking_a_card(
                crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process()
                    .map(crate::gpu_unit_checks::gpu_test_start::GpuCardPool::card_count),
            );
        let one_of_the_tables_built_at_once = self
            .tables_built_at_once
            .enter(cards_this_run_judges_on.saturating_add(1));
        // 扫描内核照这张表那一栏旋钮值编（没填是产品容量那一份，与判法摘要罩的那三份逐字相同）
        let kernel_sources = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(
            table_with_its_columns.container_capacity_knobs,
        );
        use crate::crash_judge_tables::{
            build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions,
            JudgeFlow,
        };
        let not_expanded = |_segment_index: usize, _segment: &[usize]| {
            CrashPointReplaySegmentExpansion::NotExpanded
        };
        let enumerated_write_count = crate::crash::crash_state_enumeration_tables(
            table.base,
            table.writes,
            table.segments,
            &not_expanded,
        )
        .enumerated_writes
        .len();
        let masks: Vec<Vec<bool>> = table
            .persisted_sets
            .iter()
            .map(|persisted| {
                let mut mask = persisted.clone();
                mask.resize(enumerated_write_count.max(persisted.len()), false);
                mask
            })
            .collect();
        // 每个状态那一栏恢复自称的那一版跟着进判表（恢复内核的记录核对器第二条判据照它取那一版）
        let claimed_versions = table_with_its_columns
            .claimed_versions
            .of_every_state(table.persisted_sets.len());
        let tables = match build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions(
            &JudgeFlow {
                base: table.base,
                writes: table.writes,
                segments: table.segments,
                judged_root_index: 0,
                versions: &[],
                expansion: &not_expanded,
            },
            &[],
            masks,
            &claimed_versions,
            table.seams,
        ) {
            Ok(tables) => tables,
            Err(refusal) => {
                return Ok(GivenStatesOnTheGpu::TablesRefused {
                    refusal: format!("{refusal:?}"),
                })
            }
        };
        // 判表先入库、再读库，GPU 只拿读回的那一份算；打包在锁外
        crate::crash_library::assert_this_thread_holds_no_library_lock("打包给定状态表的判表");
        let tables_packed = PackedJudgeTables::of(&tables);
        let packed =
            crate::crash_library::record_the_judge_tables_and_read_them_back_outside_the_lock(
                &self.library,
                &tables_packed,
            )?;
        drop(tables_packed);
        drop(tables);
        let judged_on_the_card = |verdicts: Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]>,
                                  judged_on: JudgingDevice|
         -> GivenStatesOnTheGpu {
            GivenStatesOnTheGpu::Judged {
                words: verdicts
                    .iter()
                    .map(|verdict| {
                        cross_check_words_without_the_oracle(cross_check_words_of_gpu_verdict(
                            verdict,
                        ))
                    })
                    .collect(),
                abandoned_roots_named_of_each_state: verdicts
                    .iter()
                    .map(abandoned_roots_named_by_the_pool_checker_of_the_gpu_verdict)
                    .collect(),
                undecidable_reason_of_each_state: verdicts
                    .iter()
                    .map(undecidable_reasons_of_the_gpu_verdict)
                    .collect(),
                judged_on,
            }
        };
        let no_card = || {
            "配置开了 GPU，给定状态表的判器（crash_judge_gpu）走 GPU，这一趟却没有起好的卡：不退回 CPU 判器".to_string()
        };
        // 这一趟第一次真要在卡上判：到这里才取卡（起步、录表、判表前读库与建判表都不持卡）。取卡交错的照起步时记下的卡串分
        // （way_after_the_cards_are_not_taken）：调用方没交卡的照读回的判表交 GPU 批处理服务判、读回结论字，往下照本进程上卡那一支同一套解，
        // 等服务的时候不占「同时建判表、等卡的表」那一份额（判表已在库里）；交了卡串而取不成的报取卡那一句
        let taken = match self.cards_taken_at_the_first_table_judged_on_the_gpu() {
            Ok(taken) => taken,
            Err(reason) => match way_after_the_cards_are_not_taken(
                self.card_sources_of_the_start
                    .cards_handed_out_by_the_service
                    .as_deref(),
                reason,
            ) {
                WayAfterTheCardsAreNotTaken::ThroughTheGpuVerificationServer => {
                    drop(one_of_the_tables_built_at_once);
                    let verdict_words = self
                        .way_to_the_gpu_verification_server
                        .judge_the_packed_tables(
                            &LibraryTakenAtEachReadOrWrite::SharedInThisProcess(&self.library),
                            &packed,
                            table_with_its_columns.container_capacity_knobs,
                            crate::verdict_store::ForcedRejudging::NotForced,
                        )?;
                    return Ok(judged_on_the_card(
                        verdict_words.verdict_words,
                        verdict_words.judged_on,
                    ));
                }
                WayAfterTheCardsAreNotTaken::CardStringHandedOverButNotTaken { reason } => {
                    return Err(reason);
                }
            },
        };
        match taken.taken_from {
            GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
                let cards = &taken.cards;
                if cards.is_empty() {
                    return Err(no_card());
                }
                let (verdicts, judged_on) = judge_on_a_card_started_for_this_run_queued_by_memory(
                    cards,
                    &taken.memory_queue_of_the_cards_started_for_this_run,
                    &packed,
                    &kernel_sources,
                    &table.table_name,
                )?;
                Ok(judged_on_the_card(verdicts, judged_on))
            }
            GpuCardsSource::TheCardPoolOfThisProcess => {
                let pool = the_card_pool_this_run_takes_its_cards_from();
                let states_per_dispatch =
                    states_per_scratch_lane(pool.smallest_maximum_storage_buffer_binding_size());
                // 调用方几条工作线程各判一张表、各借各的：这个进程别的线程借满了卡就排队等它们还，不当成显存不够
                match pool.lease_the_first_card_with_room_waiting_for_the_other_threads(
                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                    mebibytes_a_judge_needs(&packed, states_per_dispatch),
                ) {
                    Ok(lease) => {
                        let judge = GpuCrashJudge::on_card_compiling_these_kernel_sources(
                            lease.card(),
                            &packed,
                            states_per_dispatch,
                            &kernel_sources,
                        );
                        let verdicts = judge.judge_states(0..packed.state_count());
                        let judged_on = JudgingDevice::Gpu {
                            card_type: lease.card().name.clone(),
                            driver_version: lease.card().driver_version.clone(),
                        };
                        drop(judge);
                        drop(lease);
                        Ok(judged_on_the_card(verdicts, judged_on))
                    }
                    Err(refusal) => {
                        let reason = given_states_lease_refusal_as_a_run_condition_not_met(
                            &table.table_name,
                            &refusal,
                        );
                        println!("{reason}");
                        Err(reason)
                    }
                }
            }
        }
    }

    /// 没编进 `gpu` 特性走不到这里（选路先报错）。
    ///
    /// # Errors
    /// 恒报错：这份二进制没有 `gpu` 特性。
    #[cfg(not(feature = "gpu"))]
    fn judge_the_table_on_the_gpu(
        &self,
        _table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesOnTheGpu, String> {
        Err(
            "配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features gpu）"
                .to_string(),
        )
    }

    /// GPU 判完一张表之后：塞故障（只给 GPU 与 CPU 两趟分开跑、比结论会红的那条用例）、记判不了的状态、存一块，交回 GPU 那一份结论
    /// （有判不了的状态时整张表交 CPU 判器判，这一趟已经红了）。
    fn settle_the_gpu_judgement(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
        blocks: &[BlockOfAGivenStatesTable],
        mut gpu_words: Vec<CrossCheckWords>,
        abandoned_roots_named_of_each_state: Vec<Vec<AbandonedRootNamedByThePoolChecker>>,
        undecidable_reason_of_each_state: &[Option<String>],
        judged_on: JudgingDevice,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        let table = table_with_its_columns.table;
        assert_eq!(
            abandoned_roots_named_of_each_state.len(),
            gpu_words.len(),
            "GPU 判器逐状态交回结论字，点名的被抛弃根与对拍字同一个状态数"
        );
        assert_eq!(
            undecidable_reason_of_each_state.len(),
            gpu_words.len(),
            "GPU 判器逐状态交回结论字，判不了的原因与对拍字同一个状态数"
        );
        match self.fault_put_on_the_gpu_side {
            FaultPutOnTheGpuSide::NoFault => {}
            FaultPutOnTheGpuSide::FirstInvariantViolatedBitFlippedOnTheFirstStateOfEachTable => {
                if let Some(first) = gpu_words.first_mut() {
                    first.pool_checker_violated_bits ^= 1;
                    first.pool_checker_judged_bits |= 1;
                }
            }
        }
        let could_not_judge: Vec<String> = gpu_words
            .iter()
            .enumerate()
            .filter(|(_, words)| words.gpu_could_not_judge)
            .map(|(ordinal, _)| {
                format!(
                    "{} 第 {ordinal} 个状态（结论里判不了那一位：{}）",
                    table.table_name,
                    undecidable_reason_of_each_state[ordinal].as_deref().expect(
                        "判不了那一位由恢复内核判不了那一位与池级 checker 判不了那一字置，原因就是这两样里不为 0 的那几个"
                    )
                )
            })
            .collect();
        let gpu_states = gpu_words.len();
        self.note(|tally| {
            tally.tables_judged_here += 1;
            tally.states_judged_by_the_gpu_here += u64::try_from(gpu_states).expect("装得进 u64");
            tally
                .states_the_gpu_could_not_judge
                .extend(could_not_judge.iter().cloned());
        });
        let gpu = GivenStatesConclusionsOfATable {
            count_vector: count_vector_of_the_given_states(&gpu_words),
            states: gpu_words
                .into_iter()
                .zip(abandoned_roots_named_of_each_state)
                .map(|(words, abandoned_roots_named)| GivenStateConclusion {
                    words,
                    pool_checker_violation_details: Vec::new(),
                    abandoned_roots_named_by_the_pool_checker: abandoned_roots_named,
                })
                .collect(),
            source: GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost,
        };
        if !could_not_judge.is_empty() {
            // GPU 判不了的状态不进库（照 H6 判红）；往下走的结论取 CPU 判器的
            return Ok(self.judged_on_the_cpu_here(table_with_its_columns));
        }
        self.store_the_conclusions(blocks, JudgedBy::GpuJudge, &gpu, judged_on)?;
        Ok(gpu)
    }

    fn judged_on_the_cpu_here(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> GivenStatesConclusionsOfATable {
        let cpu = judge_the_given_states_on_the_cpu(table_with_its_columns);
        let states = cpu.states.len();
        self.note(|tally| {
            tally.states_judged_by_the_cpu_here += u64::try_from(states).expect("装得进 u64");
        });
        cpu
    }
}

/// 一张给定状态表除持久集合之外那几样（基线、写表、分段、接缝、旋钮栏）的摘要，与录进库的字节同一种写法、表名写成空串、持久集合那一节空着：
/// 进块键段写表与起步镜像那一半（表名不进：内容相同、名字不同的两张表是同几块），两台分片按它分表。
fn frame_digest_of_a_given_states_table(
    table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
) -> [u8; DIGEST_BYTES] {
    let table = table_with_its_columns.table;
    let frame = RecordedGivenStatesTableWithItsColumns {
        table: RecordedGivenStatesTable {
            table_name: String::new(),
            base: table.base.clone(),
            writes: table.writes.to_vec(),
            segments: table.segments.to_vec(),
            seams: table.seams.to_vec(),
            persisted_sets: Vec::new(),
        },
        container_capacity_knobs: table_with_its_columns.container_capacity_knobs,
        claimed_versions: Vec::new(),
    };
    sha256_digest(&frame.to_bytes())
}

/// 给定状态表的一块（[`GivenStatesEngine::blocks_of_the_table`]）：块键与这一块的状态在表里的位置（按表里的次序）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct BlockOfAGivenStatesTable {
    key: VerdictBlockKey,
    positions: Vec<usize>,
}

/// 一个给定状态归哪一段：第一次有写没持久的那一段；每次写都持久的归「全部持久」那一块。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SegmentGroupOfAGivenState {
    FirstUnpersistedWriteInSegment(usize),
    EveryWritePersisted,
}

impl SegmentGroupOfAGivenState {
    fn name(self) -> String {
        match self {
            Self::FirstUnpersistedWriteInSegment(segment_index) => {
                format!("first_unpersisted_write_in_segment_{segment_index}")
            }
            Self::EveryWritePersisted => "every_write_persisted".to_string(),
        }
    }
}

/// 一个给定状态（持久集合按写表下标）归哪一段（[`SegmentGroupOfAGivenState`]）。
fn segment_group_of_a_given_state(
    segments: &[Vec<usize>],
    persisted: &[bool],
) -> SegmentGroupOfAGivenState {
    segments
        .iter()
        .position(|segment| {
            segment
                .iter()
                .any(|write_index| !persisted.get(*write_index).copied().unwrap_or(false))
        })
        .map_or(
            SegmentGroupOfAGivenState::EveryWritePersisted,
            SegmentGroupOfAGivenState::FirstUnpersistedWriteInSegment,
        )
}

/// 几块的计数向量相加（逐条不变量的判到、违例、不适用，各观察者计数与按发布分的态数各按名字相加）：给定状态表按段存成几块，读回时拼回整张表的。
fn sum_of_the_count_vectors(count_vectors: &[BlockCountVector]) -> BlockCountVector {
    let mut sum = BlockCountVector::default();
    for count_vector in count_vectors {
        for (invariant, counts) in &count_vector.per_invariant {
            let entry =
                sum.per_invariant
                    .entry(invariant.clone())
                    .or_insert(InvariantStateCounts {
                        judged: 0,
                        violated: 0,
                        not_applicable: 0,
                    });
            entry.judged += counts.judged;
            entry.violated += counts.violated;
            entry.not_applicable += counts.not_applicable;
        }
        for (observer, count) in &count_vector.per_observer {
            *sum.per_observer.entry(observer.clone()).or_insert(0) += count;
        }
        for (publish, count) in &count_vector.states_by_publish {
            *sum.states_by_publish.entry(publish.clone()).or_insert(0) += count;
        }
    }
    sum
}

/// 没判的一张表交回的占位结论（每个状态都记成不红、两遍都没落到根）：两台分片、归别的机器判而这一趟库里又没有
/// （[`GivenStatesRun::left_tables_to_the_other_hosts`]），或这一趟只跑录入段（[`GivenStatesRun::recorded_only`]）。崩溃注入照它往下走
/// 只为把这一批的表都判完或录进库，交回的报告不作数。
fn placeholder_conclusions_of_a_table(
    state_count: usize,
    source: GivenStatesConclusionsSource,
) -> GivenStatesConclusionsOfATable {
    let placeholder_words = CrossCheckWords {
        red_bits: 0,
        pool_checker_violated_bits: 0,
        consulted_pass_landed_root: LandedRootWords::NOT_LANDED,
        ignored_pass_landed_root: LandedRootWords::NOT_LANDED,
        consulted_pass_outcome: 0,
        ignored_pass_outcome: 0,
        consulted_pass_failure_point: 0,
        ignored_pass_failure_point: 0,
        pool_checker_judged_bits: 0,
        scan_rebuild_outcome: SCAN_REBUILD_OUTCOME_NOT_REACHED,
        gpu_could_not_judge: false,
    };
    let every_word = vec![placeholder_words; state_count];
    GivenStatesConclusionsOfATable {
        count_vector: count_vector_of_the_given_states(&every_word),
        states: every_word
            .into_iter()
            .map(|words| GivenStateConclusion {
                words,
                pool_checker_violation_details: Vec::new(),
                abandoned_roots_named_by_the_pool_checker: Vec::new(),
            })
            .collect(),
        source,
    }
}

impl GivenStatesJudge for GivenStatesEngine {
    fn stop_is_requested(&self) -> bool {
        let requested = self.stop_request.is_requested();
        if requested {
            self.answered_a_stop_request
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        requested
    }

    fn judge_the_given_states(
        &self,
        table: &GivenStatesOfATable<'_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        self.judge_a_table_with_its_columns(table.into())
    }

    fn judge_the_given_states_with_its_columns(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        self.judge_a_table_with_its_columns(table_with_its_columns)
    }
}

impl GivenStatesEngine {
    /// 判一张带两栏的给定状态表（[`GivenStatesJudge::judge_the_given_states`] 是两栏都不填的那一种）：两栏跟着表录进库
    /// （[`RecordedGivenStatesTableWithItsColumns`]）、进块键，判段按读回的两栏判（CPU 判器调池级 checker 带旋钮值的入口、记录核对器第二条判据
    /// 照那一栏恢复自称的那一版；GPU 判器把旋钮值编进扫描内核、那一栏进判表）。
    ///
    /// # Errors
    /// 同 [`GivenStatesJudge::judge_the_given_states`]。
    fn judge_a_table_with_its_columns(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        let table = table_with_its_columns.table;
        let state_count = table.persisted_sets.len();
        if state_count == 0 {
            return Ok(GivenStatesConclusionsOfATable {
                states: Vec::new(),
                count_vector: count_vector_of_the_given_states(&[]),
                source: GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost,
            });
        }
        if u64::try_from(state_count).expect("装得进 u64") > STATES_PER_BLOCK {
            return Err(format!(
                "给定状态表 {} 有 {state_count} 个状态，一张表最多 {STATES_PER_BLOCK} 个（按段切成的几块合起来）：第一版不收更大的表",
                table.table_name
            ));
        }
        // 两段（选段照 RunStage「给定状态引擎认选段」）：录入段把这张表按它的内容摘要写进库
        // （库里已有逐字节相同的那一份就不再写，L11）；判段照同一张表现算摘要、从库里读回那一份判，块键与结论都从读回的那一份来
        let recorded_bytes =
            RecordedGivenStatesTableWithItsColumns::of(table_with_its_columns).to_bytes();
        let digest = sha256_digest(&recorded_bytes);
        if self.stage.records() {
            self.record_the_table(&table.table_name, &digest, &recorded_bytes)?;
        }
        drop(recorded_bytes);
        // 配了 GPU 而这份二进制没编进 gpu 特性：表照录（上面），判段不跑、交占位结论，开跑时打过本次未跑那一行；不退回 CPU 判
        if self.not_run_without_the_gpu_feature.is_some() {
            self.note(|tally| tally.tables_not_judged_without_the_gpu_feature += 1);
            return Ok(placeholder_conclusions_of_a_table(
                state_count,
                GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly,
            ));
        }
        if !self.stage.judges() {
            self.note(|tally| tally.tables_recorded_only += 1);
            return Ok(placeholder_conclusions_of_a_table(
                state_count,
                GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly,
            ));
        }
        let read_back_bytes = self
            .store()
            .read_given_states_table(&digest)
            .map_err(|error| format!("给定状态表 {} 从库里读不回：{error:?}", table.table_name))?
            .ok_or_else(|| {
                format!(
                    "给定状态表 {} 这一份内容（{}）不在库里：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）",
                    table.table_name,
                    hexadecimal_text(&digest[..8])
                )
            })?;
        if sha256_digest(&read_back_bytes) != digest {
            return Err(format!(
                "给定状态表 {} 从库里读回的摘要与键对不上",
                table.table_name
            ));
        }
        let read_back = RecordedGivenStatesTableWithItsColumns::from_bytes(&read_back_bytes)
            .ok_or_else(|| format!("给定状态表 {} 从库里读回的字节解不出", table.table_name))?;
        read_back.with_the_table(|table_read_back_from_the_library| {
            self.judge_a_table_read_back(table_read_back_from_the_library)
        })
    }

    /// 录入段：这张表按它的内容摘要写进库；库里已有逐字节相同的那一份（同一个摘要）就不再写（录入键命中，L11）。
    ///
    /// # Errors
    /// 库读写失败。
    fn record_the_table(
        &self,
        table_name: &str,
        digest: &[u8; DIGEST_BYTES],
        recorded_bytes: &[u8],
    ) -> Result<(), String> {
        let stored = self
            .store()
            .read_given_states_table(digest)
            .map_err(|error| format!("给定状态表 {table_name} 读不了库：{error:?}"))?;
        crate::crash_library::assert_this_thread_holds_no_library_lock("比对给定状态表的字节");
        if stored.is_some_and(|stored| stored == recorded_bytes) {
            self.note(|tally| tally.tables_whose_recording_was_reused += 1);
            return Ok(());
        }
        self.store()
            .store_given_states_table(digest, recorded_bytes)
            .map_err(|error| format!("给定状态表 {table_name} 写不进库：{error:?}"))?;
        self.note(|tally| tally.tables_recorded_here += 1);
        Ok(())
    }

    /// 判一张从库里读回的表：块键、复用、两台分片的占位、判都照读回的这一份。
    fn judge_a_table_read_back(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        let table = table_with_its_columns.table;
        let state_count = table.persisted_sets.len();
        let blocks = Self::blocks_of_the_table(table_with_its_columns);
        let owns_the_table = self.owns_the_table(&frame_digest_of_a_given_states_table(
            table_with_its_columns,
        ));
        let stored_red = if owns_the_table {
            StoredRedTableOfTheRun::JudgedAgainHere
        } else {
            StoredRedTableOfTheRun::TakenAsTheOtherHostJudgedIt
        };
        if let Some(stored) =
            self.stored_conclusions(&blocks, self.judged_by, state_count, stored_red)?
        {
            self.note(|tally| {
                tally.tables_reused += 1;
                tally.states_reused += u64::try_from(state_count).expect("装得进 u64");
            });
            return Ok(stored);
        }
        if !owns_the_table {
            self.note(|tally| tally.tables_left_to_the_other_hosts += 1);
            return Ok(placeholder_conclusions_of_a_table(
                state_count,
                GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost,
            ));
        }
        self.judge_a_table_read_back_from_the_library(table_with_its_columns, &blocks, state_count)
    }

    /// 判一张从库里读回的给定状态表：配了 GPU 只由 GPU 判器判（GPU 判不了的整张交 CPU 判器），没配只由 CPU 判器判；结论按段存进库。
    fn judge_a_table_read_back_from_the_library(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
        blocks: &[BlockOfAGivenStatesTable],
        state_count: usize,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        let table = table_with_its_columns.table;
        match self.judged_by {
            JudgedBy::CpuJudge => {
                let cpu = self.judged_on_the_cpu_here(table_with_its_columns);
                self.note(|tally| tally.tables_judged_here += 1);
                self.store_the_conclusions(blocks, JudgedBy::CpuJudge, &cpu, JudgingDevice::Cpu)?;
                Ok(cpu)
            }
            JudgedBy::GpuJudge => match self.judge_the_table_on_the_gpu(table_with_its_columns)? {
                GivenStatesOnTheGpu::Judged {
                    words,
                    abandoned_roots_named_of_each_state,
                    undecidable_reason_of_each_state,
                    judged_on,
                } => self.settle_the_gpu_judgement(
                    table_with_its_columns,
                    blocks,
                    words,
                    abandoned_roots_named_of_each_state,
                    &undecidable_reason_of_each_state,
                    judged_on,
                ),
                GivenStatesOnTheGpu::TablesRefused { refusal } => {
                    self.note(|tally| {
                        tally.states_the_gpu_could_not_judge.push(format!(
                            "{} 建不出 GPU 判表（{state_count} 个状态都判不了）：{refusal}",
                            table.table_name
                        ));
                    });
                    Ok(self.judged_on_the_cpu_here(table_with_its_columns))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests;
