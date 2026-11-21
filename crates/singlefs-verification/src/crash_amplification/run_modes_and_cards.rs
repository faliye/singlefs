//! 这一趟怎么跑：流水线判定行的版本（[`PipelineJudgingCode`]）、四种跑法与各段在 CPU 还是 GPU 上算（[`CrashAmplificationRunMode`]、
//! [`computation_routes`]）、多机配置、没带 `gpu` 特性时的「本次未跑」行、按卡的先后分状态，与这一趟用哪几张卡（[`gpu_cards_in_use`]）。

use super::*;

// ============================== 判定行的版本（流水线单测用） ==============================

/// 流水线单测用的判定行版本（判定块与违例两张表 `01`、`04` 里那 32 字节）：只由这一句定，对全部崩溃点相同。判器的代码（GPU 判器、
/// CPU 判器、池级 checker、记录核对器）不进：改判器不作废已判的判定行，判法改了对不对靠对拍与复用哨兵抓；树与块键不动。
#[derive(Clone, Debug)]
pub struct PipelineJudgingCode {
    digest: [u8; DIGEST_BYTES],
}

impl PipelineJudgingCode {
    /// 流水线判定行的那一把版本。
    #[must_use]
    pub fn of_the_pipeline() -> Self {
        Self {
            digest: sha256_digest(b"singlefs pipeline judging rows\n"),
        }
    }

    #[must_use]
    pub fn digest(&self) -> &[u8; DIGEST_BYTES] {
        &self.digest
    }

    /// 进判定行的版本：就是这份摘要。
    #[must_use]
    pub fn version(&self) -> JudgingUnitsFingerprint {
        JudgingUnitsFingerprint(self.digest)
    }

    /// 换掉摘要（测试里模拟库里另有一份别的版本的判定行）。
    pub fn replace_digest(&mut self, digest: [u8; DIGEST_BYTES]) {
        self.digest = digest;
    }
}

/// 编译期 `file!()` 给的路径归一成仓内路径：去掉 `./`，把 `..` 与它前面那一段一起消掉（`#[path = "../../x/tests/common/mod.rs"]`
/// 引进来的模块，`file!()` 带着 `..`）。消不掉的 `..`（越出仓根）原样留着，交给 [`NodeLabel::new`] 拒绝。
#[must_use]
pub fn repository_relative_file(compile_time_file: &str) -> String {
    let mut components: Vec<&str> = Vec::new();
    for component in compile_time_file.split('/') {
        match component {
            "" | "." => {}
            ".." => match components.last() {
                Some(&last) if last != ".." => {
                    components.pop();
                }
                _ => components.push(component),
            },
            _ => components.push(component),
        }
    }
    components.join("/")
}

// ============================== 跑法与分工 ==============================

/// 跑法：两个开关（配置 `multi-host.env` 的 `ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`，判法在
/// `scripts/crash/crash-point-replay-shard-configuration-check.sh`）组出四种：单 CPU、双机 CPU、单机 CPU + GPU、双机 CPU + 多 GPU。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashAmplificationRunMode {
    SingleHostCpu,
    TwoHostsCpu,
    SingleHostCpuWithGpu,
    TwoHostsCpuWithGpus,
}

impl CrashAmplificationRunMode {
    #[must_use]
    pub fn from_switches(across_machines: bool, gpu: bool) -> Self {
        match (across_machines, gpu) {
            (false, false) => Self::SingleHostCpu,
            (true, false) => Self::TwoHostsCpu,
            (false, true) => Self::SingleHostCpuWithGpu,
            (true, true) => Self::TwoHostsCpuWithGpus,
        }
    }

    /// 从 `multi-host.env` 这类 `键=值` 文件读两个开关；键没写按关（没写就是没有）。文件找不找得到由调用方先判
    /// （[`multi_host_configuration_of_the_file_found`]：找不到就报错，不退回读同名的环境变量）。
    ///
    /// # Errors
    /// 读不了那份文件（没有读权限、不是 UTF-8、找到之后又不在了）：报错、写出路，不按两个开关都关往下跑
    /// （`.claude/rules/verification.md`「所有任务都要多机可用」：没写就是没有，不补缺省；读不了不是没写）。
    pub fn from_configuration_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|error| {
            format!(
                "多机配置 {} 找到了却读不了（{error}）：跑法只由多机配置定，读不了不当两个开关都关、不补缺省。\
                 出路：让这个进程读得了那一份（读权限、存成 UTF-8），或设 {} 指到一份读得了的",
                path.display(),
                crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            )
        })?;
        let switch = |name: &str| {
            text.lines().any(|line| {
                line.trim()
                    .strip_prefix(name)
                    .and_then(|rest| rest.strip_prefix('='))
                    .is_some_and(|value| value.trim().trim_matches('"') == "1")
            })
        };
        Ok(Self::from_switches(
            switch("ENABLE_ACROSS_MACHINES"),
            switch("ENABLE_GPU"),
        ))
    }

    #[must_use]
    pub fn uses_gpu(self) -> bool {
        matches!(self, Self::SingleHostCpuWithGpu | Self::TwoHostsCpuWithGpus)
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::SingleHostCpu => "single_host_cpu",
            Self::TwoHostsCpu => "two_hosts_cpu",
            Self::SingleHostCpuWithGpu => "single_host_cpu_with_gpu",
            Self::TwoHostsCpuWithGpus => "two_hosts_cpu_with_gpus",
        }
    }
}

/// 块怎么分给几方：按权重（一台机器的 CPU 一方，或一张卡一方、权重取空闲显存 MiB）平滑轮转，
/// 第 k 块归哪一方只由权重表与 k 定，几方各算各的、不用商量。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightedShares {
    weights: Vec<u64>,
}

impl WeightedShares {
    /// # Panics
    /// 一方都没有，或权重全是 0。
    #[must_use]
    pub fn new(weights: Vec<u64>) -> Self {
        assert!(
            weights.iter().any(|weight| *weight > 0),
            "至少一方权重大于 0"
        );
        Self { weights }
    }

    /// 平滑加权轮转：前 k 块里每一方分到的块数与权重成比例，任何时刻偏差不超过一块。交回第 `block_sequence` 块的归属方。
    #[must_use]
    pub fn owner_of_block(&self, block_sequence: u64) -> usize {
        let total: u64 = self.weights.iter().sum();
        let position = block_sequence % total;
        let mut current: Vec<i128> = vec![0; self.weights.len()];
        let mut owner = 0usize;
        for _ in 0..=position {
            for (index, weight) in self.weights.iter().enumerate() {
                current[index] += i128::from(*weight);
            }
            owner = (0..current.len())
                .max_by_key(|index| (current[*index], std::cmp::Reverse(*index)))
                .expect("至少一方");
            current[owner] -= i128::from(total);
        }
        owner
    }
}

/// 显卡配置的读法住在 [`crate::gpu_card_configuration`]（只开 `gpu` 的卡池也照它挑卡），这里转出来，调用方与单测照旧经这一份引。
pub use crate::gpu_card_configuration::{
    card_priority_lines_of_the_caller, cards_by_priority_from_configuration_or_environment,
    configuration_value, configured_cards_not_usable, multi_host_configuration_file,
    parse_cards_by_priority, CardByPriority, ConfiguredCards, HostAddress,
};

/// 一类计算在哪算。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputedOn {
    Cpu,
    Gpu,
}

/// 这一趟每一类计算在哪算（[`computation_routes`] 定，`run_from_environment` 照它选路）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComputationRoutes {
    /// 第 ① 段的判器。
    pub crash_judge: ComputedOn,
    /// 第 ② 段读事实表的核对。
    pub facts_verifier: ComputedOn,
}

impl ComputationRoutes {
    /// 有哪一类要卡。
    #[must_use]
    pub fn need_a_card(&self) -> bool {
        [self.crash_judge, self.facts_verifier].contains(&ComputedOn::Gpu)
    }
}

/// 照本机配置定每一类计算在哪算：选路只看本机多机配置的 GPU 开关与仓内豁免表
/// （`.claude/gate.d/gpu-exemptions.tsv`：豁免的那几截在代码里标 `// gpu-exempt: <截名>`，不走这里）——没开 GPU 三类都在 CPU 上；
/// 开了 GPU 三类都走 GPU（「测试能走 GPU 的必须走 GPU」，私有显卡配置里那份登记哪几类走 GPU 的正表已删）。
///
/// # Errors
/// 开了 GPU 而这份二进制没编进 `gpu` 特性（上不了卡，不悄悄退回 CPU）。
pub fn computation_routes(
    gpu_is_switched_on: bool,
    gpu_feature_is_compiled_in: bool,
) -> Result<ComputationRoutes, String> {
    if !gpu_is_switched_on {
        return Ok(ComputationRoutes {
            crash_judge: ComputedOn::Cpu,
            facts_verifier: ComputedOn::Cpu,
        });
    }
    if !gpu_feature_is_compiled_in {
        return Err(
            "配置写了 ENABLE_GPU=1，判器、核对与单元校验和都要走 GPU，这份二进制没有编进 gpu 特性（cargo … --features gpu）"
                .to_string(),
        );
    }
    Ok(ComputationRoutes {
        crash_judge: ComputedOn::Gpu,
        facts_verifier: ComputedOn::Gpu,
    })
}

/// 配置开了 GPU 而这份二进制没编进 gpu 特性时判段报本次未跑的原因（「配 GPU 必用 GPU」：不带特性编的那一趟
/// 不因为配置开了 GPU 就拒跑，也不退回 CPU 判；录入段照录，带特性那一趟从库里读回在卡上判）；配置没开 GPU、或特性编进来了是 `None`。
#[must_use]
pub fn judge_not_run_without_the_gpu_feature(
    gpu_is_switched_on: bool,
    gpu_feature_is_compiled_in: bool,
) -> Option<String> {
    (gpu_is_switched_on && !gpu_feature_is_compiled_in).then(|| {
        "配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性：判段这一趟不跑（录入段照录、写进库），带 gpu 特性那一趟从库里读回在卡上判（cargo … --features gpu）"
            .to_string()
    })
}

/// 这个进程读到的多机配置（那一份文件）与它定的跑法。
#[derive(Clone, Debug)]
pub struct MultiHostConfigurationOfThisProcess {
    pub configuration_file: PathBuf,
    pub mode: CrashAmplificationRunMode,
}

/// 这个进程读的多机配置：照 [`multi_host_configuration_file`] 的找法从这个进程的当前目录起找（设了 `SINGLEFS_MULTI_HOST_CONFIG` 照它，
/// 相对路径按当前目录；没设按当前目录的 git common-dir 找主工作树根下那一份），找不到就报错（[`multi_host_configuration_of_the_file_found`]）。
/// 仓根不是键的输入、也不是库的输入（`records/2026-10-13-复用键与全仓改名.md`）：
/// 多机配置是这台机器的，按进程从哪起找、不按这个 crate 编在哪找。cargo 把测试进程的当前目录设成包目录，入口脚本在仓根或仓副本里起，
/// 都落在同一个 git 工作树里，找到的是同一份。
///
/// # Errors
/// 取不到当前目录；`SINGLEFS_MULTI_HOST_CONFIG` 设了而那份文件不在；没设而主工作树根下也没有 `multi-host.env`。
pub fn multi_host_configuration_of_this_process(
) -> Result<MultiHostConfigurationOfThisProcess, String> {
    let current_directory = std::env::current_dir()
        .map_err(|error| format!("取不到这个进程的当前目录（多机配置从这里起找）：{error}"))?;
    multi_host_configuration_of_the_file_found(
        multi_host_configuration_file(&current_directory)?,
        &current_directory,
    )
}

/// [`multi_host_configuration_of_this_process`] 去掉找文件的那一层：找到的那一份（`found`，没找到是 `None`）定跑法。
/// 环境相关的一切只由多机配置定（`.claude/rules/verification.md`「所有任务都要多机可用，不许绑定某一台」：没写就是没有，不补缺省），
/// 找不到就报错、写出路，不读同名的环境变量 `ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`，也不按两个开关都关往下跑。
///
/// # Errors
/// `found` 是 `None`（`looked_from` 是从哪起找的，只进说明）；找到的那一份读不了（[`CrashAmplificationRunMode::from_configuration_file`]）。
pub fn multi_host_configuration_of_the_file_found(
    found: Option<PathBuf>,
    looked_from: &Path,
) -> Result<MultiHostConfigurationOfThisProcess, String> {
    match found {
        Some(configuration_file) => Ok(MultiHostConfigurationOfThisProcess {
            mode: CrashAmplificationRunMode::from_configuration_file(&configuration_file)?,
            configuration_file,
        }),
        None => Err(format!(
            "找不到多机配置（从 {} 起找：{} 没设，这个 git 工作树的主工作树根下也没有 {}）：跑法只由多机配置定，不读同名的环境变量、\
             不补缺省。出路：照仓根 multi-host.env.example 在主工作树根写一份 {}，或设 {} 指到那一份",
            looked_from.display(),
            crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_FILE_NAME,
            crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_FILE_NAME,
            crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        )),
    }
}

/// 这份二进制在本机多机配置下判段跑不跑：配置开了 GPU 而没编进 gpu 特性时交回原因（[`judge_not_run_without_the_gpu_feature`]），
/// 跑得了是 `None`。在进程里拿给定状态表的结论接着往下走的装置与用例开头调它：交回原因就打本次未跑那一行、提前返回，
/// 不在占位结论上 panic 或断言（判归带 gpu 特性那一趟）。
///
/// # Errors
/// 多机配置读不出（[`multi_host_configuration_of_this_process`]）。
pub fn judge_not_run_in_this_binary() -> Result<Option<String>, String> {
    let configuration = multi_host_configuration_of_this_process()?;
    Ok(judge_not_run_without_the_gpu_feature(
        configuration.mode.uses_gpu(),
        cfg!(feature = "gpu"),
    ))
}

/// 在进程里拿给定状态表的结论往下判的用例（装置的内联单测、验证包集成测试）开头调：这份二进制在本机配置下判段不跑
/// （[`judge_not_run_in_this_binary`]）就打门禁认的本次未跑那一行、交回 `true`，用例随即返回；判得了交回 `false`。
/// 多机配置读不出直接 panic（配置写坏了不该当成没开 GPU）。
#[must_use]
pub fn judging_test_not_run_without_the_gpu_feature(test_name: &str) -> bool {
    match judge_not_run_in_this_binary() {
        Ok(Some(reason)) => {
            println!(
                "{}",
                crate::gpu_memory_ledger::gpu_not_run_line(test_name, &reason)
            );
            true
        }
        Ok(None) => false,
        Err(refusal) => panic!("读不出多机配置，判不了这份二进制的判段跑不跑：{refusal}"),
    }
}

/// 判段报本次未跑那一趟打的两行（[`judge_not_run_without_the_gpu_feature`]）：崩溃放量入口认的 `CRASH_AMPLIFICATION_NOT_RUN step=judge`，
/// 与门禁认的 GPU 用例那一族记号（`singlefs_verification::gpu_memory_ledger::gpu_not_run_line`）。入口与门禁记本次未跑、不算通过、不判红。
#[must_use]
pub fn not_run_lines_without_the_gpu_feature(flow_name: &str, reason: &str) -> Vec<String> {
    vec![
        format!(
            "{CRASH_AMPLIFICATION_NOT_RUN_LINE_PREFIX} step=judge test=crash_amplification::run_from_environment flow={flow_name} reason={reason}"
        ),
        crate::gpu_memory_ledger::gpu_not_run_line(
            "crash_amplification::run_from_environment",
            reason,
        ),
    ]
}

/// 给定状态表的引擎判段报本次未跑那一行（[`judge_not_run_without_the_gpu_feature`]）：门禁认的 GPU 用例那一族记号
/// （`singlefs_verification::gpu_memory_ledger::gpu_not_run_line`，`NOT_RUN_<原因> test=<用例> reason=<一句>`），用例是引擎这一项的名字。
/// 不打 `GIVEN_STATES_JUDGE route=cpu`：判段不跑，也不退回 CPU 判。
#[must_use]
pub fn given_states_not_run_line_without_the_gpu_feature(item_name: &str, reason: &str) -> String {
    crate::gpu_memory_ledger::gpu_not_run_line(&format!("given_states_engine::{item_name}"), reason)
}

/// 崩溃放量本次未跑那一行的头（入口 `scripts/crash/crash-amplification-item.sh` 与双机驱动按它认）。
pub const CRASH_AMPLIFICATION_NOT_RUN_LINE_PREFIX: &str = "CRASH_AMPLIFICATION_NOT_RUN";

/// GPU 判器宽度停在拆不开的下限上那一行的头（`crash_judge_dispatch::StuckAtTheIndivisibleFloor::line` 照它起头）：可 grep。
/// 放在这里不放在派活那一份：判段在没编进 gpu 特性时也要认它（[`gpu_judging_refusal_naming_the_flow`]）。
pub const STUCK_AT_THE_INDIVISIBLE_FLOOR_LINE_PREFIX: &str =
    "CRASH_JUDGE_STUCK_AT_THE_INDIVISIBLE_FLOOR";

/// 第 ① 段在卡上判报的错：宽度停在拆不开的下限上的那一种（理由以 [`STUCK_AT_THE_INDIVISIBLE_FLOOR_LINE_PREFIX`] 起头，判器不知道流名）
/// 在行首之后补上 `flow=<流名>`、当场打到标准错误（可 grep）再交回；别的原样交回。
#[must_use]
pub fn gpu_judging_refusal_naming_the_flow(flow_name: &str, reason: String) -> String {
    match reason.strip_prefix(STUCK_AT_THE_INDIVISIBLE_FLOOR_LINE_PREFIX) {
        Some(fields) => {
            let line =
                format!("{STUCK_AT_THE_INDIVISIBLE_FLOOR_LINE_PREFIX} flow={flow_name}{fields}");
            eprintln!("{line}");
            line
        }
        None => reason,
    }
}

/// 崩溃放量判段观察者交回的盘上的池级 checker 走哪一路（照第 ① 段的选路：配了 GPU 在卡上判，没配由 CPU 判）。
/// GPU 那一路建的时候不取卡（[`pool_checker_of_the_images_on_the_gpu`]）：挂了观察者而一张盘都不交回的分项整条路不取卡、不读显卡配置。
/// `card_sources_of_the_entry` 是判段入口照起步那一刻的环境记下的显卡来路，CPU 那一路用不上。
///
/// # Errors
/// 选路走 GPU 而这份二进制没编进 gpu 特性。
pub(crate) fn pool_checker_of_the_images_after_the_observers(
    route: ComputedOn,
    card_sources_of_the_entry: CardSourcesOfTheEngineStart,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    match route {
        ComputedOn::Cpu => Ok(PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu),
        ComputedOn::Gpu => pool_checker_of_the_images_on_the_gpu(card_sources_of_the_entry),
    }
}

/// GPU 那一路：建的时候不取卡（[`CardsTakenAtTheFirstImageToJudge`]）；头一张要判的盘那一刻照判段入口记下的多机配置与服务交的卡取
/// （`card_sources_of_the_entry`，[`gpu_cards_of_this_run_from_the_multi_host_configuration`]），取不到（调用方没交卡）的判表
/// 交 GPU 批处理服务判（[`WayToTheGpuVerificationServer::TheRealService`]），不退回 CPU 判。
#[cfg(feature = "gpu")]
#[allow(
    clippy::unnecessary_wraps,
    reason = "没编进 gpu 特性的那一份报错，两份同一个签名"
)]
fn pool_checker_of_the_images_on_the_gpu(
    card_sources_of_the_entry: CardSourcesOfTheEngineStart,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    Ok(
        PoolCheckerOfTheImagesAfterTheObservers::TakingTheGpuCardsAtTheFirstImageToJudge(
            CardsTakenAtTheFirstImageToJudge::new(
                card_sources_of_the_entry,
                WayToTheGpuVerificationServer::TheRealService,
            ),
        ),
    )
}

/// 没编进 `gpu` 特性：选路走不到 GPU（[`computation_routes`] 先报错，判段报本次未跑的那一趟不判）。
///
/// # Errors
/// 恒报错。
#[cfg(not(feature = "gpu"))]
fn pool_checker_of_the_images_on_the_gpu(
    _card_sources_of_the_entry: CardSourcesOfTheEngineStart,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性：观察者交回的盘上的池级 checker 上不了卡（选路不该走到这里）".to_string())
}

/// 按优先级把 `pending_states` 个状态装到卡上：逐张装到这张卡一次派活带得了的状态数为止，装不下的才给下一张。
/// 交回每张卡装了多少，0 就是这一趟不用这张卡；每张卡都装满还有剩的，剩下的排队等前面的判完（不算在任何一张卡上）。
#[must_use]
pub fn place_states_on_cards_by_priority(pending_states: u64, capacities: &[u64]) -> Vec<u64> {
    let mut remaining = pending_states;
    capacities
        .iter()
        .map(|capacity| {
            let placed = remaining.min(*capacity);
            remaining -= placed;
            placed
        })
        .collect()
}

/// 这一趟用的卡：编进 `gpu` 特性时是起好上下文的卡，按优先级；没编进来时没有这个东西。
#[cfg(feature = "gpu")]
pub(crate) type CardsInUse = Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>;
#[cfg(not(feature = "gpu"))]
pub(crate) type CardsInUse = ();

/// 这一趟的卡（[`gpu_cards_in_use`]）：这一趟自己起的那几张（卡池的是空的：池里的卡借到哪张才在哪张上起设备，
/// 用的时候照池取，文件头第 5 条），与卡从哪来。
#[cfg_attr(
    not(feature = "gpu"),
    allow(dead_code, reason = "没编进 gpu 特性时不取卡，这个结局只在编进来时造")
)]
pub(crate) struct GpuCardsOfThisRun {
    pub(crate) cards: CardsInUse,
    pub(crate) taken_from: GpuCardsSource,
}

/// 这个进程的卡池（[`crate::gpu_unit_checks::gpu_test_start::card_pool_of_this_process_started_now`]：没起过就当场起，起过的不再起；
/// 别的线程正在起时等它起完）里的卡就是这一趟的卡（池里的卡要与配置对得上，[`the_card_pool_matches_the_configured_cards`]），
/// 此刻不登记、不起设备：借到哪张卡才在哪张上起。带 `gpu` 特性的每个进程都这样取（库单测、集成测试二进制、崩溃放量分项驱动起的测试二进制、
/// `src/bin/` 的装置）：同一个进程里经引擎起步的判与直接借卡的 GPU 用例走同一套显存账，不另起不进账的设备
/// （[`crate::gpu_unit_checks::devices_marked_outside_the_card_pool_in_this_process`] 不因崩溃放量取卡而涨）。
///
/// # Errors
/// 有卡而起不了卡池（显卡配置写错、没给这台机器点卡、点名的卡少一张找不到）；卡池判成没有卡（本机没有 NVIDIA 卡，或调用方没交卡串）；
/// 池里的卡与配置对不上。
#[cfg(feature = "gpu")]
pub(crate) fn gpu_cards_in_use(
    configured: Option<&ConfiguredCards>,
) -> Result<GpuCardsOfThisRun, String> {
    let Some(pool) =
        crate::gpu_unit_checks::gpu_test_start::card_pool_of_this_process_started_now()?
    else {
        return Err(
            "配置写了 ENABLE_GPU=1，这个进程的卡池却判成没有卡（本机没有 NVIDIA 独显，或调用方没交卡串：验证包不借卡）"
                .to_string(),
        );
    };
    the_card_pool_matches_the_configured_cards(
        &pool.cards_in_priority_order_as_configured(),
        configured,
    )?;
    Ok(GpuCardsOfThisRun {
        cards: Vec::new(),
        taken_from: GpuCardsSource::TheCardPoolOfThisProcess,
    })
}

/// 照判段入口记下的多机配置与服务交的卡（`card_sources_of_the_entry`，[`CardSourcesOfTheEngineStart::cards_by_priority`]，不读取卡那一刻的环境）
/// 取这一趟的卡（[`gpu_cards_in_use`]）。
///
/// # Errors
/// 没经服务给卡而多机配置开了 GPU 或写了显卡先后；卡串写法不对；卡池起不来、判成本机没有 NVIDIA 卡、与配置对不上。
#[cfg(feature = "gpu")]
pub(crate) fn gpu_cards_of_this_run_from_the_multi_host_configuration(
    card_sources_of_the_entry: &CardSourcesOfTheEngineStart,
) -> Result<GpuCardsOfThisRun, String> {
    gpu_cards_in_use(card_sources_of_the_entry.cards_by_priority()?.as_ref())
}

/// 池里的卡（`pool_cards`：按池的优先级，每张的序号与配置给它的额度）对不对得上这一趟读到的显卡配置：配置点了名的（`configured` 里
/// 这一台地址那几项），池里的卡要与它逐张同序号、同额度，对不上就报错（池与这一趟读到的显卡配置不同，不悄悄换卡）；没配就照池里的卡用。
///
/// # Errors
/// 两份显卡配置对不上。
#[cfg(feature = "gpu")]
fn the_card_pool_matches_the_configured_cards(
    pool_cards: &[(u32, Option<u64>)],
    configured: Option<&ConfiguredCards>,
) -> Result<(), String> {
    if let Some(configured) = configured {
        let mine: Vec<(u32, Option<u64>)> = configured
            .cards_of_this_host()
            .into_iter()
            .map(|(index, memory_quota_mebibytes)| (index, Some(memory_quota_mebibytes)))
            .collect();
        if mine != pool_cards {
            return Err(format!(
                "这个进程的卡池里是 {pool_cards:?}（序号，额度 MiB），这一趟的显卡配置给这一台（HOSTS 里的 {}）点的是 {mine:?}：两份显卡配置对不上，\
                 不另起一套设备、也不悄悄换卡",
                configured.this_host().as_str()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
