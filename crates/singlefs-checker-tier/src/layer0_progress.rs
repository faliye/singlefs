//! 层 0 全量的断点续跑：进度文件怎么起名、怎么写、怎么读回（层 0 规模三轮判决的 R1–R5 与 U1–U3，
//! `research/prompts/m2-layer0-scale-r1-main-verification.md` 第三节、`-r2-` 第三节、`-r3-` 第三节；用户 2026-09-26 定「跑前实现」）。
//!
//! - 一个进度文件对应「输入指纹 + 流名 + 枚举计划哈希」（R1、U1）：输入指纹由调用方给（门禁 54 号那一份清单），流名由用例给，
//!   计划哈希由枚举器按基线、写表、段、每段怎么展开、被判的根、版本表、切法、有没有观察者现算（`crash` 模块）；三样都进文件名，
//!   文件头再写一遍，换了任何一样都是另一个文件，互不作废。
//! - 文件头写片方案（状态数、片数、片长，R2）；读回时逐片核区间，对不上整份作废。
//! - 每行带自己的 CRC-32C 校验和，换行结尾才算整行；末尾没换行的半行丢掉、那一片重跑（R3）；整行里校验和不对、缺任何一个字段、
//!   多出字段、字段解不开，整份作废（U2）。同一片出现两次，计数逐项相同就去重，不同整份作废。
//! - 片行记这一片的全部计数，连同观察者在这一片上累计的数（`observer_counts`、`observed_states`，R4、U3）。
//!
//! 写片行的时机、判红删文件、合并前核片数、报续跑了几片都在 `crash::enumerate_layer0_in_state_slices`。
//!
//! 双机分片（里程碑三第六项，`.claude/kb/milestone/03-third-txn.md` 第六节）：`SINGLEFS_LAYER0_SHARD=<i>/<n>` 只跑切片序号
//! `slice_index % n == i` 的那些片，跑完把这些片的片行写成这一片的账本（与进度文件同一种片行，另加文件头、片的线程行与收尾行）；
//! `SINGLEFS_LAYER0_SHARD=merge/<n>` 不枚举，读 n 份账本、核齐、按切片序号交回。分片跑的进度文件名、文件头与计划哈希都带
//! `shard-<i>-of-<n>`，与单机的进度文件不混。

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::num::NonZeroU32;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use singlefs_checker::image::IMPLEMENTED_INVARIANTS;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::checksum::crc32_castagnoli;

use crate::crash::{
    Layer0Finding, Layer0FindingSample, Layer0FindingSignature, Layer0Findings,
    Layer0ObserverCounts, Layer0OracleViolationKind, Layer0PublishOfState, Layer0RedPass,
    Layer0SegmentOfState, Layer0Tally, LAYER0_FINDING_SAMPLES_KEPT,
    LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE, LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE,
    LAYER0_RED_PASS_POOL_CHECKER, LAYER0_RED_PASS_RECORD_CHECKER,
    RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT, RECORD_CHECKER_ROOT_WITHOUT_RECORD,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::RecordCheck;

/// 进度文件放在哪个目录。没设就不留进度文件（平时的 `cargo test`）。
pub const LAYER0_PROGRESS_DIRECTORY_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_LAYER0_PROGRESS_DIRECTORY";
/// 这一趟的输入指纹（门禁 54 号算的那一份，与它的全绿标记同一份清单）。设了进度目录就必须设。
pub const LAYER0_INPUT_FINGERPRINT_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_LAYER0_INPUT_FINGERPRINT";
/// 设成 `1`：不管有没有进度文件都从头跑（先删掉这一格的进度文件）。没设或设成 `0` 就续跑。
pub const LAYER0_START_OVER_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_LAYER0_START_OVER";
/// 双机分片：`<i>/<n>`（0 ≤ i < n）只跑分给第 i 片的切片、写这一片的账本；`merge/<n>` 只读 n 份账本并起来。没设就不分片。
/// 设了就要同时设进度目录与输入指纹（账本写在进度目录里，文件头记输入指纹）。
pub const LAYER0_SHARD_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_LAYER0_SHARD";
/// 发现日志写到哪个文件（层 0 放量的「出错那一份」，用户 2026-09-27 定「全量和错误双份日志」）：每趟枚举往末尾追加一节
/// （[`Layer0FindingsLogSection`]）。没设就不写（标准输出上的 `LAYER0_FINDING` / `LAYER0_FINDINGS` 照打）。
pub const LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_LAYER0_FINDINGS_FILE";

/// 进度文件格式的版本：写进文件头，格式改了就换号，旧文件整份作废。2：片行多了发现表（`findings=`）。
const PROGRESS_FILE_FORMAT: u32 = 2;
/// 文件名里一段（输入指纹、流名）最长几个字节。
const NAME_PART_MAXIMUM_BYTES: usize = 128;

/// 进度文件名里的一段（输入指纹、流名）：只许 ASCII 字母、数字、`_`、`-`，1 到 128 字节——它直接进文件名，也进文件头里按空格分的字段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0ProgressFileNamePart(String);

/// 给的文本不能当文件名里的一段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0ProgressFileNamePartRejected {
    pub text: String,
}

impl Layer0ProgressFileNamePart {
    /// # Errors
    /// 空、超过 128 字节、或有 ASCII 字母数字与 `_`、`-` 之外的字符。
    pub fn new(text: &str) -> Result<Self, Layer0ProgressFileNamePartRejected> {
        let every_character_allowed = text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
        if text.is_empty() || text.len() > NAME_PART_MAXIMUM_BYTES || !every_character_allowed {
            return Err(Layer0ProgressFileNamePartRejected {
                text: text.to_string(),
            });
        }
        Ok(Self(text.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 有进度文件时，开跑那一刻怎么对待它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer0ResumeStart {
    /// 读回来、核过的片不再跑。
    ResumeFromTheProgressFile,
    /// 强制从头跑：先删掉这一格的进度文件（R5 的开关）。
    StartOverDiscardingTheProgressFile,
}

/// 整条流跑完（每一片都并进来了）之后，进度文件怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer0ProgressFileAfterCompletion {
    /// 删掉：进度文件只在「没跑完」时存在，跑完之后判红判绿都由那一趟自己的输出说，下一趟从头跑。
    Deleted,
    /// 留着：只供测试强制进入（`.claude/rules/fs-design.md` 五条硬要求第 2 条），用例拿整份文件截断、改坏，造「跑到一半被杀」的样子。
    KeptForTheTestThatInspectsIt,
}

/// 一格进度文件：放在哪、属于哪份输入与哪条流、开跑时怎么对待、跑完怎么办。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0ProgressFileSettings {
    pub directory: PathBuf,
    pub input_fingerprint: Layer0ProgressFileNamePart,
    pub stream_name: Layer0ProgressFileNamePart,
    pub start: Layer0ResumeStart,
    pub after_completion: Layer0ProgressFileAfterCompletion,
}

/// 双机分片里的一片：第几片、共几片（0 ≤ 第几片 < 共几片）。第 i 片拿切片序号 `slice_index % n == i` 的那些切片（交错分，
/// 越往后越贵的状态不会集中到一台）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layer0ShardOfShards {
    shard_index: u32,
    shard_count: NonZeroU32,
}

/// 第几片不小于共几片。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layer0ShardIndexOutOfRange {
    pub shard_index: u32,
    pub shard_count: NonZeroU32,
}

impl Layer0ShardOfShards {
    /// # Errors
    /// `shard_index` 不小于 `shard_count`。
    pub fn new(
        shard_index: u32,
        shard_count: NonZeroU32,
    ) -> Result<Self, Layer0ShardIndexOutOfRange> {
        if shard_index >= shard_count.get() {
            return Err(Layer0ShardIndexOutOfRange {
                shard_index,
                shard_count,
            });
        }
        Ok(Self {
            shard_index,
            shard_count,
        })
    }

    #[must_use]
    pub fn shard_index(self) -> u32 {
        self.shard_index
    }

    #[must_use]
    pub fn shard_count(self) -> NonZeroU32 {
        self.shard_count
    }

    /// 这个切片归不归这一片：`slice_index % n == i`。
    #[must_use]
    pub fn owns_slice(self, slice_index: usize) -> bool {
        shard_owning_slice(slice_index, self.shard_count) == self.shard_index
    }

    /// 日志与文件头里的写法：`<i>/<n>`。
    #[must_use]
    pub fn text(self) -> String {
        format!("{}/{}", self.shard_index, self.shard_count)
    }

    /// 文件名里的写法：`shard-<i>-of-<n>`。
    #[must_use]
    pub fn file_name_part(self) -> String {
        format!("shard-{}-of-{}", self.shard_index, self.shard_count)
    }
}

/// 切片序号归 n 片里的第几片：`slice_index % n`。
#[must_use]
pub fn shard_owning_slice(slice_index: usize, shard_count: NonZeroU32) -> u32 {
    let shard_count_as_usize = usize::try_from(shard_count.get()).expect("片数装得进 usize");
    u32::try_from(slice_index % shard_count_as_usize).expect("余数小于片数，装得进 u32")
}

/// 编这个测试二进制、跑这一片的工具链：`rustc -Vv` 前三行、`cargo -V`、target triple（`rustc -Vv` 的 `host:` 那一行：
/// 这几条流都不交叉编译，编出来的就是 host）。进账本文件头，merge 时 n 份与 merge 那一趟自己的逐字段相同才并。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0ToolchainIdentity {
    pub rustc_version_lines: String,
    pub cargo_version: String,
    pub target_triple: String,
}

impl Layer0ToolchainIdentity {
    /// 问跑这个测试的 cargo（环境变量 `CARGO`，cargo test 起测试二进制时设）与它旁边的 rustc（旁边没有就取 `PATH` 上的）。
    ///
    /// # Panics
    /// 环境里没有 `CARGO`（不是经 cargo test 起的）、两个命令起不来或退出码不是 0、输出不是 UTF-8、`rustc -Vv` 不到三行或没有 `host:` 行：
    /// 账本头记不下工具链就不分片，不悄悄记一个空的。
    #[must_use]
    pub fn of_the_cargo_running_this_test() -> Self {
        let cargo = PathBuf::from(std::env::var_os("CARGO").unwrap_or_else(|| {
            panic!("分片跑与 merge 要经 cargo test 起（环境里要有 CARGO）：账本头要记编这个测试二进制的工具链")
        }));
        let rustc_next_to_cargo = cargo.with_file_name("rustc");
        let rustc = if rustc_next_to_cargo.is_file() {
            rustc_next_to_cargo
        } else {
            PathBuf::from("rustc")
        };
        let rustc_verbose_version = standard_output_of(&rustc, "-Vv");
        let cargo_version = standard_output_of(&cargo, "-V");
        let rustc_lines: Vec<&str> = rustc_verbose_version.lines().collect();
        assert!(
            rustc_lines.len() >= 3,
            "{} -Vv 不到三行：{rustc_verbose_version:?}",
            rustc.display()
        );
        let target_triple = rustc_lines
            .iter()
            .find_map(|line| line.strip_prefix("host: "))
            .unwrap_or_else(|| {
                panic!(
                    "{} -Vv 里没有 host: 那一行：{rustc_verbose_version:?}",
                    rustc.display()
                )
            });
        Self {
            rustc_version_lines: rustc_lines[..3].join("\n"),
            cargo_version: cargo_version.trim_end().to_string(),
            target_triple: target_triple.to_string(),
        }
    }
}

/// 起一个命令、带一个参数，交回它的标准输出。
///
/// # Panics
/// 起不来、退出码不是 0、输出不是 UTF-8。
fn standard_output_of(program: &Path, argument: &str) -> String {
    let output = std::process::Command::new(program)
        .arg(argument)
        .output()
        .unwrap_or_else(|error| panic!("起 {} {argument}：{error}", program.display()));
    assert!(
        output.status.success(),
        "{} {argument} 退出码 {:?}：{}",
        program.display(),
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap_or_else(|error| {
        panic!("{} {argument} 的输出不是 UTF-8：{error}", program.display())
    })
}

/// 分片跑的一片：进度文件照单机的续跑走（文件名、文件头、计划哈希另带这一片），跑完写这一片的账本（在进度目录里）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0ShardRun {
    pub progress: Layer0ProgressFileSettings,
    pub shard: Layer0ShardOfShards,
    pub toolchain: Layer0ToolchainIdentity,
}

/// merge：读 `directory` 里这条流的 n 份账本，文件头要与这一趟逐字段相同（输入指纹、流名、计划、切片方案、有没有观察者、片数、工具链）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0ShardMerge {
    pub directory: PathBuf,
    pub input_fingerprint: Layer0ProgressFileNamePart,
    pub stream_name: Layer0ProgressFileNamePart,
    pub shard_count: NonZeroU32,
    pub toolchain: Layer0ToolchainIdentity,
}

/// 层 0 枚举留不留进度文件、分不分片。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layer0Resume {
    /// 不留（平时的 `cargo test`、用例之间对拍）。
    NoProgressFile,
    /// 留，按这一格续跑（单机跑全部切片）。
    KeepProgressFile(Layer0ProgressFileSettings),
    /// 只跑分给这一片的切片（照样留进度文件续跑），跑完写这一片的账本。
    RunOneShardKeepingProgressFile(Layer0ShardRun),
    /// 不枚举：读 n 份账本、核齐、按切片序号并起来。
    MergeShardLedgers(Layer0ShardMerge),
}

/// [`LAYER0_SHARD_ENVIRONMENT_VARIABLE`] 解出来的开关。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer0ShardSwitch {
    RunOneShard(Layer0ShardOfShards),
    MergeShardLedgers { shard_count: NonZeroU32 },
}

/// 解 `<i>/<n>` 或 `merge/<n>`：两段都是十进制数，n ≥ 1，i < n。
///
/// # Errors
/// 形态不对、数解不开、n 是 0、i 不小于 n（给人看的一句）。
pub fn shard_switch_from_text(text: &str) -> Result<Layer0ShardSwitch, String> {
    let (first, count_text) = text
        .split_once('/')
        .ok_or_else(|| format!("要写成 <i>/<n> 或 merge/<n>：{text:?}"))?;
    let shard_count = u32::try_from(decimal_field(count_text)?)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or_else(|| format!("片数要是 1 到 {} 的整数：{text:?}", u32::MAX))?;
    if first == "merge" {
        return Ok(Layer0ShardSwitch::MergeShardLedgers { shard_count });
    }
    let shard_index = u32::try_from(decimal_field(first)?)
        .map_err(|error| format!("第几片装不进 u32 {text:?}：{error}"))?;
    Layer0ShardOfShards::new(shard_index, shard_count)
        .map(Layer0ShardSwitch::RunOneShard)
        .map_err(|out_of_range| format!("第几片要小于片数：{out_of_range:?}"))
}

impl Layer0Resume {
    /// 从环境变量取：[`LAYER0_PROGRESS_DIRECTORY_ENVIRONMENT_VARIABLE`] 没设就不留；设了就要同时设
    /// [`LAYER0_INPUT_FINGERPRINT_ENVIRONMENT_VARIABLE`]；[`LAYER0_START_OVER_ENVIRONMENT_VARIABLE`] 为 `1` 时强制从头跑；
    /// [`LAYER0_SHARD_ENVIRONMENT_VARIABLE`] 设了就分片跑一片或 merge（工具链问跑这个测试的 cargo）。
    /// 跑完删进度文件。`stream_name` 是这条流的名字（进文件名）。
    ///
    /// # Panics
    /// 设了进度目录却没设输入指纹；设了分片开关却没设进度目录；merge 时设了从头跑；四个变量里有设了而值不合法的
    /// （空目录、指纹或流名不能进文件名、从头跑开关不是 `0` / `1`、分片开关解不开、不是 UTF-8）：配错了就停，不悄悄退回不续跑、不分片。
    #[must_use]
    pub fn from_environment(stream_name: &str) -> Self {
        Self::from_environment_values(
            stream_name,
            std::env::var(LAYER0_PROGRESS_DIRECTORY_ENVIRONMENT_VARIABLE),
            std::env::var(LAYER0_INPUT_FINGERPRINT_ENVIRONMENT_VARIABLE),
            std::env::var(LAYER0_START_OVER_ENVIRONMENT_VARIABLE),
            std::env::var(LAYER0_SHARD_ENVIRONMENT_VARIABLE),
            Layer0ToolchainIdentity::of_the_cargo_running_this_test,
        )
    }

    /// [`Self::from_environment`] 的判定本身：四个环境变量读到什么、工具链是什么由调用方给（用例不改进程的环境变量、不起子进程）。
    /// `toolchain` 只在分片开关设了时调。
    ///
    /// # Panics
    /// 同 [`Self::from_environment`]。
    #[must_use]
    pub fn from_environment_values(
        stream_name: &str,
        directory: Result<String, std::env::VarError>,
        input_fingerprint: Result<String, std::env::VarError>,
        start_over: Result<String, std::env::VarError>,
        shard_switch: Result<String, std::env::VarError>,
        toolchain: impl FnOnce() -> Layer0ToolchainIdentity,
    ) -> Self {
        let shard_switch = match shard_switch {
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(raw)) => {
                panic!("{LAYER0_SHARD_ENVIRONMENT_VARIABLE} 读到的不是 UTF-8：{raw:?}")
            }
            Ok(text) => Some(shard_switch_from_text(&text).unwrap_or_else(|problem| {
                panic!("{LAYER0_SHARD_ENVIRONMENT_VARIABLE} 解不开：{problem}")
            })),
        };
        if let (Err(std::env::VarError::NotPresent), Some(switch)) = (&directory, shard_switch) {
            panic!(
                "设了 {LAYER0_SHARD_ENVIRONMENT_VARIABLE}（{switch:?}）就要设 {LAYER0_PROGRESS_DIRECTORY_ENVIRONMENT_VARIABLE}：账本写在进度目录里"
            );
        }
        let directory = match directory {
            Err(std::env::VarError::NotPresent) => return Self::NoProgressFile,
            Err(std::env::VarError::NotUnicode(raw)) => {
                panic!("{LAYER0_PROGRESS_DIRECTORY_ENVIRONMENT_VARIABLE} 读到的不是 UTF-8：{raw:?}")
            }
            Ok(directory) if directory.is_empty() => {
                panic!("{LAYER0_PROGRESS_DIRECTORY_ENVIRONMENT_VARIABLE} 设了却是空串")
            }
            Ok(directory) => PathBuf::from(directory),
        };
        let input_fingerprint = match input_fingerprint {
            Ok(text) => Layer0ProgressFileNamePart::new(&text).unwrap_or_else(|rejected| {
                panic!(
                    "{LAYER0_INPUT_FINGERPRINT_ENVIRONMENT_VARIABLE} 要是 1 到 128 个 ASCII 字母、数字、_、-：{rejected:?}"
                )
            }),
            Err(error) => panic!(
                "设了 {LAYER0_PROGRESS_DIRECTORY_ENVIRONMENT_VARIABLE} 就要设 {LAYER0_INPUT_FINGERPRINT_ENVIRONMENT_VARIABLE}（进度文件按输入指纹分格）：{error}"
            ),
        };
        let start = match start_over.as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("0") => {
                Layer0ResumeStart::ResumeFromTheProgressFile
            }
            Ok("1") => Layer0ResumeStart::StartOverDiscardingTheProgressFile,
            Ok(other) => {
                panic!("{LAYER0_START_OVER_ENVIRONMENT_VARIABLE} 只许 0 或 1，读到 {other:?}")
            }
            Err(std::env::VarError::NotUnicode(raw)) => {
                panic!("{LAYER0_START_OVER_ENVIRONMENT_VARIABLE} 读到的不是 UTF-8：{raw:?}")
            }
        };
        let stream_name = Layer0ProgressFileNamePart::new(stream_name).unwrap_or_else(|rejected| {
            panic!("流名要是 1 到 128 个 ASCII 字母、数字、_、-：{rejected:?}")
        });
        let progress = Layer0ProgressFileSettings {
            directory,
            input_fingerprint,
            stream_name,
            start,
            after_completion: Layer0ProgressFileAfterCompletion::Deleted,
        };
        match shard_switch {
            None => Self::KeepProgressFile(progress),
            Some(Layer0ShardSwitch::RunOneShard(shard)) => {
                Self::RunOneShardKeepingProgressFile(Layer0ShardRun {
                    progress,
                    shard,
                    toolchain: toolchain(),
                })
            }
            Some(Layer0ShardSwitch::MergeShardLedgers { shard_count }) => match progress.start {
                Layer0ResumeStart::StartOverDiscardingTheProgressFile => panic!(
                    "{LAYER0_SHARD_ENVIRONMENT_VARIABLE}=merge/{shard_count} 只读账本、没有进度文件可丢：别跟 {LAYER0_START_OVER_ENVIRONMENT_VARIABLE}=1 一起设"
                ),
                Layer0ResumeStart::ResumeFromTheProgressFile => {
                    Self::MergeShardLedgers(Layer0ShardMerge {
                        directory: progress.directory,
                        input_fingerprint: progress.input_fingerprint,
                        stream_name: progress.stream_name,
                        shard_count,
                        toolchain: toolchain(),
                    })
                }
            },
        }
    }
}

/// 这一趟枚举的计划：计划哈希（`crash` 模块现算，64 个小写十六进制字符）与片方案。进度文件头写它，读回时逐片核它。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0ProgressPlan {
    pub plan_hash: String,
    pub state_count: u64,
    /// 首尾相接的片，从 0 起、到状态数止。
    pub slices: Vec<Range<u64>>,
    /// 有没有观察者：有的话每片的 `observed_states` 必须等于这片的状态数，没有的话必须是 0（U3）。
    pub has_observer: bool,
    /// 分片跑时是第几片（进度文件只收归这一片的切片）；单机跑全部切片时是 `None`。
    pub shard: Option<Layer0ShardOfShards>,
}

impl Layer0ProgressPlan {
    fn states_per_slice(&self) -> u64 {
        self.slices
            .first()
            .map_or(0, |slice| slice.end - slice.start)
    }
}

/// 进度文件放在哪：`<目录>/layer0-progress-<流名>-<输入指纹>-<计划哈希>.txt`；分片跑时 `.txt` 前另带 `-shard-<i>-of-<n>`。
#[must_use]
pub fn progress_file_path(
    settings: &Layer0ProgressFileSettings,
    plan: &Layer0ProgressPlan,
) -> PathBuf {
    let shard_suffix = plan
        .shard
        .map_or_else(String::new, |shard| format!("-{}", shard.file_name_part()));
    settings.directory.join(format!(
        "layer0-progress-{}-{}-{}{shard_suffix}.txt",
        settings.stream_name.as_str(),
        settings.input_fingerprint.as_str(),
        plan.plan_hash
    ))
}

/// 一行的正文加上它自己的校验和：`<正文> checksum=<正文的 CRC-32C，8 个小写十六进制字符>`（不带换行）。
fn line_with_checksum(body: &str) -> String {
    format!("{body} checksum={:08x}", crc32_castagnoli(body.as_bytes()))
}

/// 文件头的正文；分片跑时末尾另带 ` shard=<i>/<n>`。
fn header_body(settings: &Layer0ProgressFileSettings, plan: &Layer0ProgressPlan) -> String {
    let shard_suffix = plan
        .shard
        .map_or_else(String::new, |shard| format!(" shard={}", shard.text()));
    format!(
        "layer0_progress_file format={PROGRESS_FILE_FORMAT} input_fingerprint={} stream={} plan={} states={} slices={} states_per_slice={} has_observer={}{shard_suffix}",
        settings.input_fingerprint.as_str(),
        settings.stream_name.as_str(),
        plan.plan_hash,
        plan.state_count,
        plan.slices.len(),
        plan.states_per_slice(),
        plan.has_observer
    )
}

/// 片行的字段，按行里的次序。读回时逐个核：少一个、多一个、次序不对，整份作废。
const SLICE_LINE_KEYS: [&str; 25] = [
    "slice",
    "first",
    "end",
    "states",
    "states_by_publish",
    "violations",
    "root_persisted_states",
    "no_file_states",
    "file_read_states",
    "failed_states",
    "journal_differing_states",
    "verification_ran_states",
    "verification_failed_states",
    "first_violation",
    "ignored_violations",
    "first_ignored_violation",
    "record_root_without_record",
    "record_claimed_state_missing_unit",
    "checker_evaluated_states",
    "checker_violated_states",
    "checker_first_violation",
    "checker_not_applicable_states",
    "observed_states",
    "observer_counts",
    "findings",
];

fn optional_text_field(text: Option<&str>) -> String {
    text.map_or_else(
        || "none".to_string(),
        |text| format!("text:{}", hexadecimal_text(text.as_bytes())),
    )
}

fn publish_field(publish: Layer0PublishOfState) -> String {
    match publish {
        Layer0PublishOfState::UpToTheRootOf {
            root_write_index,
            instance,
            checkpoint_txg,
        } => format!(
            "root_{root_write_index}_{}_{}",
            instance.0, checkpoint_txg.0
        ),
        Layer0PublishOfState::AfterTheLastRoot => "after_the_last_root".to_string(),
        Layer0PublishOfState::EveryWritePersisted => "every_write_persisted".to_string(),
    }
}

/// 一张表写成 `name:value,name:value`；空表写 `empty`。
fn map_field<'entry>(entries: impl Iterator<Item = (String, String)> + 'entry) -> String {
    let joined: Vec<String> = entries
        .map(|(name, value)| format!("{name}:{value}"))
        .collect();
    if joined.is_empty() {
        "empty".to_string()
    } else {
        joined.join(",")
    }
}

/// 片行：这一片的片号、序号区间与这一片的全部计数（连同观察者在这一片上的累计），带校验和，不带换行。
/// 按字段拆开写全：`Layer0Tally` 新加一个字段而这里没写，编译不过。
#[must_use]
pub fn slice_line(slice_index: usize, slice: &Range<u64>, tally: &Layer0Tally) -> String {
    let Layer0Tally {
        states,
        states_by_publish,
        violations,
        root_persisted_states,
        no_file_states,
        file_read_states,
        failed_states,
        journal_differing_states,
        verification_ran_states,
        verification_failed_states,
        first_violation,
        ignored_violations,
        first_ignored_violation,
        record_root_without_record,
        record_claimed_state_missing_unit,
        checker_evaluated_states,
        checker_violated_states,
        checker_first_violation,
        checker_not_applicable_states,
        observed_states,
        observer_counts,
        findings,
    } = tally;
    let invariant_counts = |counts: &BTreeMap<&'static str, u64>| {
        map_field(
            counts
                .iter()
                .map(|(invariant, count)| ((*invariant).to_string(), count.to_string())),
        )
    };
    let values: [String; 25] = [
        slice_index.to_string(),
        slice.start.to_string(),
        slice.end.to_string(),
        states.to_string(),
        map_field(
            states_by_publish
                .iter()
                .map(|(publish, count)| (publish_field(*publish), count.to_string())),
        ),
        violations.to_string(),
        root_persisted_states.to_string(),
        no_file_states.to_string(),
        file_read_states.to_string(),
        failed_states.to_string(),
        journal_differing_states.to_string(),
        verification_ran_states.to_string(),
        verification_failed_states.to_string(),
        optional_text_field(first_violation.as_deref()),
        ignored_violations.to_string(),
        optional_text_field(first_ignored_violation.as_deref()),
        record_root_without_record.to_string(),
        record_claimed_state_missing_unit.to_string(),
        invariant_counts(checker_evaluated_states),
        invariant_counts(checker_violated_states),
        map_field(checker_first_violation.iter().map(|(invariant, detail)| {
            (
                (*invariant).to_string(),
                hexadecimal_text(detail.as_bytes()),
            )
        })),
        invariant_counts(checker_not_applicable_states),
        observed_states.to_string(),
        map_field(
            observer_counts
                .iter()
                .map(|(name, count)| (name.to_string(), count.to_string())),
        ),
        findings_field(findings),
    ];
    let body: Vec<String> = SLICE_LINE_KEYS
        .iter()
        .zip(values)
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    line_with_checksum(&body.join(" "))
}

/// 读回一行：校验和对得上才交回正文。
fn body_of_a_checked_line(line: &str) -> Result<&str, String> {
    let (body, checksum_text) = line
        .rsplit_once(" checksum=")
        .ok_or_else(|| format!("这一行没有校验和：{line:?}"))?;
    let expected = format!("{:08x}", crc32_castagnoli(body.as_bytes()));
    if checksum_text != expected {
        return Err(format!(
            "这一行的校验和对不上（写的 {checksum_text:?}，算的 {expected}）：{line:?}"
        ));
    }
    Ok(body)
}

fn decimal_field(text: &str) -> Result<u64, String> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("不是十进制数：{text:?}"));
    }
    text.parse::<u64>()
        .map_err(|error| format!("十进制数解不开 {text:?}：{error}"))
}

fn decoded_hexadecimal_text(text: &str) -> Result<String, String> {
    if !text.len().is_multiple_of(2) {
        return Err(format!("十六进制文本长度不是偶数：{text:?}"));
    }
    let bytes: Result<Vec<u8>, String> = (0..text.len())
        .step_by(2)
        .map(|start| {
            let pair = &text[start..start + 2];
            if pair
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                u8::from_str_radix(pair, 16).map_err(|error| format!("{pair:?}：{error}"))
            } else {
                Err(format!("不是小写十六进制：{pair:?}"))
            }
        })
        .collect();
    String::from_utf8(bytes?).map_err(|error| format!("十六进制解出来不是 UTF-8：{error}"))
}

fn optional_text_from_field(text: &str) -> Result<Option<String>, String> {
    if text == "none" {
        return Ok(None);
    }
    let hexadecimal = text
        .strip_prefix("text:")
        .ok_or_else(|| format!("可选文本要是 none 或 text:<十六进制>：{text:?}"))?;
    decoded_hexadecimal_text(hexadecimal).map(Some)
}

/// `name:value,…` 或 `empty` 拆成一对对；名字重复算解不开。
fn map_entries(text: &str) -> Result<Vec<(&str, &str)>, String> {
    if text == "empty" {
        return Ok(Vec::new());
    }
    let mut entries: Vec<(&str, &str)> = Vec::new();
    for entry in text.split(',') {
        let (name, value) = entry
            .split_once(':')
            .ok_or_else(|| format!("表里一项要是 name:value：{entry:?}"))?;
        if entries
            .iter()
            .any(|(earlier_name, _)| *earlier_name == name)
        {
            return Err(format!("表里 {name:?} 出现了两次"));
        }
        entries.push((name, value));
    }
    Ok(entries)
}

fn implemented_invariant_named(name: &str) -> Result<&'static str, String> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .find(|invariant| **invariant == name)
        .copied()
        .ok_or_else(|| format!("{name:?} 不在 checker 的不变量清单里"))
}

fn invariant_counts_from_field(text: &str) -> Result<BTreeMap<&'static str, u64>, String> {
    map_entries(text)?
        .into_iter()
        .map(|(name, value)| Ok((implemented_invariant_named(name)?, decimal_field(value)?)))
        .collect()
}

fn publish_from_field(text: &str) -> Result<Layer0PublishOfState, String> {
    match text {
        "after_the_last_root" => Ok(Layer0PublishOfState::AfterTheLastRoot),
        "every_write_persisted" => Ok(Layer0PublishOfState::EveryWritePersisted),
        root => {
            let parts: Vec<&str> = root
                .strip_prefix("root_")
                .ok_or_else(|| format!("按发布分的格解不开：{root:?}"))?
                .split('_')
                .collect();
            let [root_write_index, instance, checkpoint_txg] = parts.as_slice() else {
                return Err(format!(
                    "按发布分的格要是 root_<写表下标>_<实例>_<txg>：{root:?}"
                ));
            };
            Ok(Layer0PublishOfState::UpToTheRootOf {
                root_write_index: usize::try_from(decimal_field(root_write_index)?)
                    .map_err(|error| format!("写表下标装不进 usize：{error}"))?,
                instance: InstanceGeneration(
                    u32::try_from(decimal_field(instance)?)
                        .map_err(|error| format!("实例代号装不进 u32：{error}"))?,
                ),
                checkpoint_txg: CheckpointTxg(decimal_field(checkpoint_txg)?),
            })
        }
    }
}

/// 片行里的发现表（[`Layer0Findings`]）：空表写 `none`；否则写 `hex:<十六进制>`，解开是几行（最后一行不带换行）：
/// 第一行 `red_states=<至少一遍判红的状态数>`，之后每个签名一行，列与列之间是制表符：pass、violated、段、发布（片行写法）、
/// 状态数，再跟每个样本的序号与原文（原文照 [`escaped_finding_text`] 转义）。
fn findings_field(findings: &Layer0Findings) -> String {
    if *findings == Layer0Findings::default() {
        return "none".to_string();
    }
    let mut lines = vec![format!("red_states={}", findings.red_states)];
    for (signature, finding) in &findings.by_signature {
        let mut columns = vec![
            signature.red_pass.pass_name().to_string(),
            signature.red_pass.violated_names().join(","),
            signature.segment.name(),
            publish_field(signature.publish),
            finding.states.to_string(),
        ];
        for sample in &finding.earliest_samples {
            columns.push(sample.state_ordinal.to_string());
            columns.push(escaped_finding_text(&sample.violation));
        }
        lines.push(columns.join("\t"));
    }
    format!("hex:{}", hexadecimal_text(lines.join("\n").as_bytes()))
}

/// 判红的那一遍与违了哪几条，从名字解回来（[`Layer0RedPass::pass_name`] 与 [`Layer0RedPass::violated_names`] 反过来）。
fn red_pass_from_names(pass: &str, violated: &str) -> Result<Layer0RedPass, String> {
    let names: Vec<&str> = violated.split(',').collect();
    let oracle_kind = || match names.as_slice() {
        [name] => Layer0OracleViolationKind::from_name(name)
            .ok_or_else(|| format!("oracle 违例的类解不开：{name:?}")),
        _not_exactly_one => Err(format!("oracle 那一遍恰好违一类：{violated:?}")),
    };
    match pass {
        LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE => {
            Ok(Layer0RedPass::JournalConsultedOracle(oracle_kind()?))
        }
        LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE => {
            Ok(Layer0RedPass::JournalIgnoredOracle(oracle_kind()?))
        }
        LAYER0_RED_PASS_POOL_CHECKER => names
            .iter()
            .map(|name| implemented_invariant_named(name))
            .collect::<Result<Vec<&'static str>, String>>()
            .map(Layer0RedPass::PoolChecker),
        LAYER0_RED_PASS_RECORD_CHECKER => {
            let red_pass = Layer0RedPass::RecordChecker(RecordCheck {
                root_without_record: names.contains(&RECORD_CHECKER_ROOT_WITHOUT_RECORD),
                claimed_state_missing_unit: names
                    .contains(&RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT),
            });
            if red_pass.violated_names() == names {
                Ok(red_pass)
            } else {
                Err(format!("记录核对器违的判据解不开：{violated:?}"))
            }
        }
        unknown_pass => Err(format!("判红的那一遍解不开：{unknown_pass:?}")),
    }
}

fn segment_from_name(name: &str) -> Result<Layer0SegmentOfState, String> {
    if name == Layer0SegmentOfState::AllPersisted.name() {
        return Ok(Layer0SegmentOfState::AllPersisted);
    }
    usize::try_from(decimal_field(name)?)
        .map(Layer0SegmentOfState::Segment)
        .map_err(|error| format!("段号装不进 usize：{error}"))
}

/// [`findings_field`] 反过来，另核发现表的形状：每个签名至少一个状态、不多于判红的状态数；样本个数 = min(状态数, 3)、序号从小到大；
/// 判红的状态数是 0 当且仅当一个签名都没有；同一签名不出现两次。
fn findings_from_field(text: &str) -> Result<Layer0Findings, String> {
    if text == "none" {
        return Ok(Layer0Findings::default());
    }
    let hexadecimal = text
        .strip_prefix("hex:")
        .ok_or_else(|| format!("发现表要是 none 或 hex:<十六进制>：{text:?}"))?;
    let decoded = decoded_hexadecimal_text(hexadecimal)?;
    let mut lines = decoded.split('\n');
    let red_states = decimal_field(
        lines
            .next()
            .and_then(|line| line.strip_prefix("red_states="))
            .ok_or_else(|| format!("发现表第一行要是 red_states=<数>：{decoded:?}"))?,
    )?;
    let mut by_signature: BTreeMap<Layer0FindingSignature, Layer0Finding> = BTreeMap::new();
    for line in lines {
        let columns: Vec<&str> = line.split('\t').collect();
        let [pass, violated, segment, publish, states, sample_columns @ ..] = columns.as_slice()
        else {
            return Err(format!("发现表的一行少于五列：{line:?}"));
        };
        let (sample_pairs, unpaired_columns) = sample_columns.as_chunks::<2>();
        if !unpaired_columns.is_empty() {
            return Err(format!("发现表的样本要成对（序号、原文）：{line:?}"));
        }
        let earliest_samples = sample_pairs
            .iter()
            .map(|[state_ordinal, violation]| {
                Ok(Layer0FindingSample {
                    state_ordinal: decimal_field(state_ordinal)?,
                    violation: unescaped_finding_text(violation)?,
                })
            })
            .collect::<Result<Vec<Layer0FindingSample>, String>>()?;
        let finding = Layer0Finding {
            states: decimal_field(states)?,
            earliest_samples,
        };
        let expected_samples = usize::try_from(finding.states)
            .map_or(LAYER0_FINDING_SAMPLES_KEPT, |states_of_the_signature| {
                states_of_the_signature.min(LAYER0_FINDING_SAMPLES_KEPT)
            });
        let samples_ascend = finding
            .earliest_samples
            .windows(2)
            .all(|pair| pair[0].state_ordinal < pair[1].state_ordinal);
        if finding.states == 0
            || finding.states > red_states
            || finding.earliest_samples.len() != expected_samples
            || !samples_ascend
        {
            return Err(format!(
                "发现表的一个签名形状不对（状态数 {}、判红的状态数 {red_states}、样本 {} 个、序号从小到大 {samples_ascend}）：{line:?}",
                finding.states,
                finding.earliest_samples.len()
            ));
        }
        let signature = Layer0FindingSignature {
            red_pass: red_pass_from_names(pass, violated)?,
            segment: segment_from_name(segment)?,
            publish: publish_from_field(publish)?,
        };
        if by_signature.insert(signature, finding).is_some() {
            return Err(format!("发现表里同一签名出现了两次：{line:?}"));
        }
    }
    if (red_states == 0) != by_signature.is_empty() {
        return Err(format!(
            "判红的状态数 {red_states} 与签名数 {} 对不上（有一个就都有）",
            by_signature.len()
        ));
    }
    Ok(Layer0Findings {
        red_states,
        by_signature,
    })
}

/// 读回来的一片：片号、区间与计数。
struct RestoredSlice {
    slice_index: usize,
    slice: Range<u64>,
    tally: Layer0Tally,
}

/// 解一行片行（校验和已核过的正文）：字段一个不多一个不少、次序照 [`SLICE_LINE_KEYS`]。
fn restored_slice_from_body(body: &str) -> Result<RestoredSlice, String> {
    let fields: Vec<(&str, &str)> = body
        .split(' ')
        .map(|field| {
            field
                .split_once('=')
                .ok_or_else(|| format!("字段要是 key=value：{field:?}"))
        })
        .collect::<Result<_, _>>()?;
    let keys: Vec<&str> = fields.iter().map(|(key, _)| *key).collect();
    if keys != SLICE_LINE_KEYS {
        return Err(format!(
            "片行的字段对不上（缺字段、多字段或次序不对）：{keys:?}"
        ));
    }
    let value = |position: usize| fields[position].1;
    let count = |position: usize| decimal_field(value(position));
    let states_by_publish = map_entries(value(4))?
        .into_iter()
        .map(|(publish, states)| Ok((publish_from_field(publish)?, decimal_field(states)?)))
        .collect::<Result<BTreeMap<Layer0PublishOfState, u64>, String>>()?;
    let checker_first_violation = map_entries(value(20))?
        .into_iter()
        .map(|(name, detail)| {
            Ok((
                implemented_invariant_named(name)?,
                decoded_hexadecimal_text(detail)?,
            ))
        })
        .collect::<Result<BTreeMap<&'static str, String>, String>>()?;
    let mut observer_counts = Layer0ObserverCounts::default();
    for (name, amount) in map_entries(value(23))? {
        if !Layer0ObserverCounts::is_counter_name(name) {
            return Err(format!("观察者计数的名字只许小写字母、数字、_：{name:?}"));
        }
        observer_counts.add(name, decimal_field(amount)?);
    }
    let tally = Layer0Tally {
        states: count(3)?,
        states_by_publish,
        violations: count(5)?,
        root_persisted_states: count(6)?,
        no_file_states: count(7)?,
        file_read_states: count(8)?,
        failed_states: count(9)?,
        journal_differing_states: count(10)?,
        verification_ran_states: count(11)?,
        verification_failed_states: count(12)?,
        first_violation: optional_text_from_field(value(13))?,
        ignored_violations: count(14)?,
        first_ignored_violation: optional_text_from_field(value(15))?,
        record_root_without_record: count(16)?,
        record_claimed_state_missing_unit: count(17)?,
        checker_evaluated_states: invariant_counts_from_field(value(18))?,
        checker_violated_states: invariant_counts_from_field(value(19))?,
        checker_first_violation,
        checker_not_applicable_states: invariant_counts_from_field(value(21))?,
        observed_states: count(22)?,
        observer_counts,
        findings: findings_from_field(value(24))?,
    };
    Ok(RestoredSlice {
        slice_index: usize::try_from(count(0)?)
            .map_err(|error| format!("片号装不进 usize：{error}"))?,
        slice: count(1)?..count(2)?,
        tally,
    })
}

/// 读回一份进度文件：交回核过的片（按片号）与丢掉的半行数；整份作废时交回原因。
///
/// 判法：换行结尾的才是整行，末尾没换行的半行丢掉（那一片重跑）；第一行是文件头，逐字等于这一趟该写的头；
/// 其余每行是片行，校验和、字段、片号、区间、状态数、观察者看过的状态数都对得上；同一片两次、计数逐项相同就去重。
///
/// # Errors
/// 整份作废的原因（给人看的一句）。
pub fn restored_slices_of(
    content: &str,
    settings: &Layer0ProgressFileSettings,
    plan: &Layer0ProgressPlan,
) -> Result<(BTreeMap<usize, Layer0Tally>, usize), String> {
    let mut lines: Vec<&str> = content.split('\n').collect();
    // split 之后最后一段是最后一个换行之后的东西：空串说明文件以换行结尾，否则是被截断的半行。
    let half_line_dropped = match lines.pop() {
        Some("") | None => 0,
        Some(_half_line) => 1,
    };
    let Some((header, slice_lines)) = lines.split_first() else {
        return Err("没有一整行（连文件头都没写完）".to_string());
    };
    let expected_header = header_body(settings, plan);
    if body_of_a_checked_line(header)? != expected_header {
        return Err(format!(
            "文件头对不上这一趟：写的 {header:?}，该是 {expected_header:?}"
        ));
    }
    Ok((restored_slice_lines(slice_lines, plan)?, half_line_dropped))
}

/// 一串片行（进度文件与账本同一种）逐行核：校验和、字段、片号、区间、状态数、观察者看过的状态数都对得上这一趟的计划；
/// 计划带着片时每一行都要归那一片；同一片两次、计数逐项相同就去重，不同整份作废。交回按片号的计数。
fn restored_slice_lines(
    slice_lines: &[&str],
    plan: &Layer0ProgressPlan,
) -> Result<BTreeMap<usize, Layer0Tally>, String> {
    let mut restored: BTreeMap<usize, Layer0Tally> = BTreeMap::new();
    for line in slice_lines {
        let RestoredSlice {
            slice_index,
            slice,
            tally,
        } = restored_slice_from_body(body_of_a_checked_line(line)?)?;
        let Some(expected_slice) = plan.slices.get(slice_index) else {
            return Err(format!("片号 {slice_index} 超出片数 {}", plan.slices.len()));
        };
        if slice != *expected_slice {
            return Err(format!(
                "第 {slice_index} 片的区间 {slice:?} 对不上这一趟的切法 {expected_slice:?}"
            ));
        }
        if let Some(shard) = plan.shard {
            if !shard.owns_slice(slice_index) {
                return Err(format!(
                    "第 {slice_index} 个切片不归第 {} 片（归第 {} 片）",
                    shard.text(),
                    shard_owning_slice(slice_index, shard.shard_count())
                ));
            }
        }
        let slice_states = slice.end - slice.start;
        if tally.states != slice_states {
            return Err(format!(
                "第 {slice_index} 片记了 {} 个状态，区间里是 {slice_states} 个",
                tally.states
            ));
        }
        let expected_observed_states = if plan.has_observer { slice_states } else { 0 };
        if tally.observed_states != expected_observed_states {
            return Err(format!(
                "第 {slice_index} 片观察者看过 {} 个状态，该是 {expected_observed_states} 个",
                tally.observed_states
            ));
        }
        match restored.get(&slice_index) {
            Some(earlier) if *earlier != tally => {
                return Err(format!("第 {slice_index} 片出现两次、计数不同"));
            }
            Some(_same) => {}
            None => {
                restored.insert(slice_index, tally);
            }
        }
    }
    Ok(restored)
}

/// 开着的进度文件：只往末尾追加片行，每行追加完落盘。
pub struct Layer0ProgressFile {
    path: PathBuf,
    appender: File,
    after_completion: Layer0ProgressFileAfterCompletion,
}

/// 开跑时打开进度文件的结局。
pub struct Layer0ProgressFileOpened {
    pub file: Layer0ProgressFile,
    /// 核过、这一趟不再跑的片（按片号）。
    pub restored_slices: BTreeMap<usize, Layer0Tally>,
    /// 读回时丢掉的末尾半行数（0 或 1）。
    pub half_lines_dropped: usize,
    /// 原有的进度文件为什么整份不用了（强制从头跑、或读回时作废）；没有原文件或整份用上时是 `None`。
    pub discarded_because: Option<String>,
}

impl Layer0ProgressFile {
    /// 开跑：按 `settings.start` 读回或删掉原有的进度文件，再把文件头与核过的片行重写成一份新文件（先写临时文件、落盘、再换名，
    /// 末尾的半行因此不会接在下一行前面），留着追加的句柄。
    ///
    /// # Panics
    /// 进度目录建不了、文件读写或换名失败：续跑的前提是进度能落盘，落不了就停，不悄悄不留进度。
    #[must_use]
    pub fn open(
        settings: &Layer0ProgressFileSettings,
        plan: &Layer0ProgressPlan,
    ) -> Layer0ProgressFileOpened {
        std::fs::create_dir_all(&settings.directory)
            .unwrap_or_else(|error| panic!("建进度目录 {}：{error}", settings.directory.display()));
        let path = progress_file_path(settings, plan);
        let existing = match std::fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => panic!("读进度文件 {}：{error}", path.display()),
        };
        let (restored_slices, half_lines_dropped, discarded_because) =
            match (settings.start, existing) {
                (_, None) => (BTreeMap::new(), 0, None),
                (Layer0ResumeStart::StartOverDiscardingTheProgressFile, Some(_bytes)) => {
                    (BTreeMap::new(), 0, Some("强制从头跑".to_string()))
                }
                (Layer0ResumeStart::ResumeFromTheProgressFile, Some(bytes)) => {
                    match String::from_utf8(bytes)
                        .map_err(|error| format!("不是 UTF-8：{error}"))
                        .and_then(|content| restored_slices_of(&content, settings, plan))
                    {
                        Ok((restored_slices, half_lines_dropped)) => {
                            (restored_slices, half_lines_dropped, None)
                        }
                        Err(reason) => (BTreeMap::new(), 0, Some(reason)),
                    }
                }
            };
        let mut rewritten = line_with_checksum(&header_body(settings, plan));
        rewritten.push('\n');
        for (slice_index, tally) in &restored_slices {
            rewritten.push_str(&slice_line(*slice_index, &plan.slices[*slice_index], tally));
            rewritten.push('\n');
        }
        let rewriting_path = path.with_extension("rewriting");
        {
            let mut rewriting = File::create(&rewriting_path)
                .unwrap_or_else(|error| panic!("建 {}：{error}", rewriting_path.display()));
            rewriting
                .write_all(rewritten.as_bytes())
                .and_then(|()| rewriting.sync_all())
                .unwrap_or_else(|error| panic!("写 {}：{error}", rewriting_path.display()));
        }
        std::fs::rename(&rewriting_path, &path).unwrap_or_else(|error| {
            panic!(
                "{} 换名成 {}：{error}",
                rewriting_path.display(),
                path.display()
            )
        });
        File::open(&settings.directory)
            .and_then(|directory| directory.sync_all())
            .unwrap_or_else(|error| {
                panic!("落盘进度目录 {}：{error}", settings.directory.display())
            });
        let appender = OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap_or_else(|error| panic!("打开 {} 追加：{error}", path.display()));
        Layer0ProgressFileOpened {
            file: Self {
                path,
                appender,
                after_completion: settings.after_completion,
            },
            restored_slices,
            half_lines_dropped,
            discarded_because,
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 追加一片（这一片的观察者已经看完、没 panic）：整行连换行一次写下去、落盘之后才交回。
    ///
    /// # Panics
    /// 写或落盘失败。
    pub fn append_finished_slice(
        &mut self,
        slice_index: usize,
        slice: &Range<u64>,
        tally: &Layer0Tally,
    ) {
        let mut line = slice_line(slice_index, slice, tally);
        line.push('\n');
        self.appender
            .write_all(line.as_bytes())
            .and_then(|()| self.appender.sync_data())
            .unwrap_or_else(|error| panic!("追加进度文件 {}：{error}", self.path.display()));
    }

    /// 整条流跑完（每一片都并进来了）：按设置删掉或留着。交回删没删。
    ///
    /// # Panics
    /// 要删而删不掉。
    pub fn finish(self) -> Layer0ProgressFileAfterCompletion {
        match self.after_completion {
            Layer0ProgressFileAfterCompletion::Deleted => {
                std::fs::remove_file(&self.path).unwrap_or_else(|error| {
                    panic!("跑完删进度文件 {}：{error}", self.path.display())
                });
            }
            Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt => {}
        }
        self.after_completion
    }
}

/// 判红的那一趟（枚举途中有工作线程或观察者 panic）删掉进度文件：下一趟从头跑，不拿红的那一趟跑过的片接着拼（R4、R5）。
/// 在 panic 展开时由 drop 删；没在 panic 就什么都不做。
pub struct DeleteTheProgressFileWhenPanicking {
    pub path: PathBuf,
}

impl Drop for DeleteTheProgressFileWhenPanicking {
    fn drop(&mut self) {
        if std::thread::panicking() {
            match std::fs::remove_file(&self.path) {
                Ok(()) => println!(
                    "LAYER0_RESUME progress_file_deleted_after_a_red_run={}",
                    self.path.display()
                ),
                Err(error) => println!(
                    "LAYER0_RESUME progress_file_not_deleted_after_a_red_run={} error={error}",
                    self.path.display()
                ),
            }
        }
    }
}

/// 账本格式的版本：写进文件头，格式改了就换号，merge 读到别的号就不并。2：片行多了发现表（`findings=`）。
const SHARD_LEDGER_FORMAT: u32 = 2;

/// 这一片的账本放在哪：`<目录>/layer0-shard-<流名>-<i>-of-<n>.tally`。名字里不带输入指纹与计划哈希：两台的指纹或切法对不上时，
/// merge 照样读得到这一份、报出是文件头哪一处不同，而不是报缺账本。
#[must_use]
pub fn shard_ledger_path(
    directory: &Path,
    stream_name: &Layer0ProgressFileNamePart,
    shard: Layer0ShardOfShards,
) -> PathBuf {
    directory.join(format!(
        "layer0-shard-{}-{}.tally",
        stream_name.as_str(),
        shard.file_name_part()
    ))
}

/// 一片跑的时候起了几个工作线程、配的几个、从哪来、那台机器几核、从进度文件读回几片、这一趟跑了几片、跑了多久：进账本，
/// merge 时按片报进 `LAYER0_PARALLEL_START` / `LAYER0_PARALLEL_FINISHED`（门禁 54 号按片判工作线程）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0ShardWorkerThreads {
    pub spawned_worker_threads: usize,
    pub configured_worker_threads: usize,
    /// `crash::Layer0WorkerThreadsSource` 的名字。
    pub worker_threads_source: String,
    /// 跑这一片的机器报的 `available_parallelism`；报不出来是 0。
    pub available_parallelism: usize,
    pub resumed_slices: usize,
    pub freshly_run_slices: usize,
    pub elapsed_milliseconds: u64,
}

/// 账本第二行（这一片自己的那一行）的字段，按行里的次序。
const SHARD_LINE_KEYS: [&str; 9] = [
    "shard",
    "owned_slices",
    "worker_threads",
    "configured_worker_threads",
    "worker_threads_source",
    "available_parallelism",
    "resumed_slices",
    "freshly_run_slices",
    "elapsed_milliseconds",
];

/// 账本文件头的正文：同一趟分片的 n 份逐字相同（这一片是第几片不在这一行，在第二行）。`whole_plan` 是整条流的计划（不带片）。
fn shard_ledger_header_body(
    input_fingerprint: &Layer0ProgressFileNamePart,
    stream_name: &Layer0ProgressFileNamePart,
    whole_plan: &Layer0ProgressPlan,
    shard_count: NonZeroU32,
    toolchain: &Layer0ToolchainIdentity,
) -> String {
    format!(
        "layer0_shard_ledger format={SHARD_LEDGER_FORMAT} input_fingerprint={} stream={} plan={} states={} slices={} states_per_slice={} has_observer={} shards={shard_count} rustc={} cargo={} target={}",
        input_fingerprint.as_str(),
        stream_name.as_str(),
        whole_plan.plan_hash,
        whole_plan.state_count,
        whole_plan.slices.len(),
        whole_plan.states_per_slice(),
        whole_plan.has_observer,
        hexadecimal_text(toolchain.rustc_version_lines.as_bytes()),
        hexadecimal_text(toolchain.cargo_version.as_bytes()),
        hexadecimal_text(toolchain.target_triple.as_bytes())
    )
}

/// 账本第二行的正文。
fn shard_line_body(
    shard: Layer0ShardOfShards,
    owned_slices: usize,
    threads: &Layer0ShardWorkerThreads,
) -> String {
    let values = [
        shard.shard_index().to_string(),
        owned_slices.to_string(),
        threads.spawned_worker_threads.to_string(),
        threads.configured_worker_threads.to_string(),
        threads.worker_threads_source.clone(),
        threads.available_parallelism.to_string(),
        threads.resumed_slices.to_string(),
        threads.freshly_run_slices.to_string(),
        threads.elapsed_milliseconds.to_string(),
    ];
    let fields: Vec<String> = SHARD_LINE_KEYS
        .iter()
        .zip(values)
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    format!("layer0_shard {}", fields.join(" "))
}

/// 账本末行的正文：片行有几行。末行在、数对得上，才算整份写完。
fn shard_ledger_end_body(slice_lines: usize) -> String {
    format!("layer0_shard_ledger_end slice_lines={slice_lines}")
}

/// n 片里归第 i 片的切片有几个。
fn slices_owned_by(shard: Layer0ShardOfShards, slice_count: usize) -> usize {
    (0..slice_count)
        .filter(|slice_index| shard.owns_slice(*slice_index))
        .count()
}

/// 这一片的全部切片跑完之后写它的账本：文件头、这一片的线程行、按切片序号的片行（与进度文件同一种）、末行；
/// 先写临时文件、落盘、再换名，换名之后落盘目录——账本要么不在，要么是整份。交回账本的路径。
///
/// # Panics
/// `whole_plan` 带着片（要的是整条流的计划）；给的片不是恰好归这一片的全部切片（不变量被破坏）；写、落盘、换名失败。
#[must_use]
pub fn write_shard_ledger(
    run: &Layer0ShardRun,
    whole_plan: &Layer0ProgressPlan,
    threads: &Layer0ShardWorkerThreads,
    slice_tallies: &BTreeMap<usize, Layer0Tally>,
) -> PathBuf {
    assert!(
        whole_plan.shard.is_none(),
        "账本头记的是整条流的计划，不带片：{:?}",
        whole_plan.shard
    );
    let owned_slices: Vec<usize> = (0..whole_plan.slices.len())
        .filter(|slice_index| run.shard.owns_slice(*slice_index))
        .collect();
    assert_eq!(
        slice_tallies.keys().copied().collect::<Vec<usize>>(),
        owned_slices,
        "账本要恰好记归第 {} 片的每一个切片",
        run.shard.text()
    );
    let mut text = line_with_checksum(&shard_ledger_header_body(
        &run.progress.input_fingerprint,
        &run.progress.stream_name,
        whole_plan,
        run.shard.shard_count(),
        &run.toolchain,
    ));
    text.push('\n');
    text.push_str(&line_with_checksum(&shard_line_body(
        run.shard,
        owned_slices.len(),
        threads,
    )));
    text.push('\n');
    for (slice_index, tally) in slice_tallies {
        text.push_str(&slice_line(
            *slice_index,
            &whole_plan.slices[*slice_index],
            tally,
        ));
        text.push('\n');
    }
    text.push_str(&line_with_checksum(&shard_ledger_end_body(
        slice_tallies.len(),
    )));
    text.push('\n');
    let path = shard_ledger_path(
        &run.progress.directory,
        &run.progress.stream_name,
        run.shard,
    );
    let writing_path = path.with_extension("writing");
    {
        let mut writing = File::create(&writing_path)
            .unwrap_or_else(|error| panic!("建 {}：{error}", writing_path.display()));
        writing
            .write_all(text.as_bytes())
            .and_then(|()| writing.sync_all())
            .unwrap_or_else(|error| panic!("写 {}：{error}", writing_path.display()));
    }
    std::fs::rename(&writing_path, &path).unwrap_or_else(|error| {
        panic!(
            "{} 换名成 {}：{error}",
            writing_path.display(),
            path.display()
        )
    });
    File::open(&run.progress.directory)
        .and_then(|directory| directory.sync_all())
        .unwrap_or_else(|error| {
            panic!("落盘进度目录 {}：{error}", run.progress.directory.display())
        });
    path
}

/// merge 读回、核齐的 n 份账本。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0MergedShardLedgers {
    /// 按第几片排。
    pub ledger_paths: Vec<PathBuf>,
    /// 按第几片排。
    pub worker_threads_by_shard: Vec<Layer0ShardWorkerThreads>,
    /// 每个切片恰好一个，按切片序号。
    pub slice_tallies: BTreeMap<usize, Layer0Tally>,
}

/// 两行正文按 `key=value` 逐字段比，交回不同的那几处（给人看；`rustc`、`cargo`、`target` 三样解回文本）。
fn differing_fields(written_body: &str, expected_body: &str) -> Vec<String> {
    let fields_of = |body: &str| -> Vec<(String, String)> {
        body.split(' ')
            .map(|field| match field.split_once('=') {
                Some((key, value)) => (key.to_string(), value.to_string()),
                None => (field.to_string(), String::new()),
            })
            .collect()
    };
    let readable = |key: &str, value: &str| match key {
        "rustc" | "cargo" | "target" => {
            decoded_hexadecimal_text(value).unwrap_or_else(|_undecodable| value.to_string())
        }
        _other_key => value.to_string(),
    };
    let written = fields_of(written_body);
    let expected = fields_of(expected_body);
    let mut differences = Vec::new();
    for (key, expected_value) in &expected {
        match written.iter().find(|(written_key, _)| written_key == key) {
            None => differences.push(format!("缺字段 {key}")),
            Some((_, written_value)) if written_value != expected_value => {
                differences.push(format!(
                    "{key} 账本写的 {:?}、这一趟是 {:?}",
                    readable(key, written_value),
                    readable(key, expected_value)
                ))
            }
            Some(_same) => {}
        }
    }
    for (key, _) in &written {
        if !expected.iter().any(|(expected_key, _)| expected_key == key) {
            differences.push(format!("多出字段 {key}"));
        }
    }
    differences
}

/// 解账本第二行（校验和已核过的正文）。
fn shard_line_from_body(body: &str) -> Result<(u32, usize, Layer0ShardWorkerThreads), String> {
    let fields_text = body
        .strip_prefix("layer0_shard ")
        .ok_or_else(|| format!("第二行要以 layer0_shard 起头：{body:?}"))?;
    let fields: Vec<(&str, &str)> = fields_text
        .split(' ')
        .map(|field| {
            field
                .split_once('=')
                .ok_or_else(|| format!("字段要是 key=value：{field:?}"))
        })
        .collect::<Result<_, _>>()?;
    let keys: Vec<&str> = fields.iter().map(|(key, _)| *key).collect();
    if keys != SHARD_LINE_KEYS {
        return Err(format!(
            "第二行的字段对不上（缺字段、多字段或次序不对）：{keys:?}"
        ));
    }
    let count = |position: usize| -> Result<usize, String> {
        usize::try_from(decimal_field(fields[position].1)?)
            .map_err(|error| format!("{} 装不进 usize：{error}", fields[position].0))
    };
    let worker_threads_source = fields[4].1;
    if !Layer0ObserverCounts::is_counter_name(worker_threads_source) {
        return Err(format!(
            "worker_threads_source 只许小写字母、数字、_：{worker_threads_source:?}"
        ));
    }
    let shard_index = u32::try_from(decimal_field(fields[0].1)?)
        .map_err(|error| format!("第几片装不进 u32：{error}"))?;
    Ok((
        shard_index,
        count(1)?,
        Layer0ShardWorkerThreads {
            spawned_worker_threads: count(2)?,
            configured_worker_threads: count(3)?,
            worker_threads_source: worker_threads_source.to_string(),
            available_parallelism: count(5)?,
            resumed_slices: count(6)?,
            freshly_run_slices: count(7)?,
            elapsed_milliseconds: decimal_field(fields[8].1)?,
        },
    ))
}

/// 核一份账本：整份写完（换行结尾、末行记的片行数对得上）；文件头与这一趟逐字段相同；第二行记的是文件名上的那一片；
/// 片行各自对得上这一趟的计划、都归这一片、同一切片不出现两次、归这一片的切片一个不缺。
fn slices_of_one_shard_ledger(
    shard: Layer0ShardOfShards,
    path: &Path,
    content: &str,
    expected_header_body: &str,
    whole_plan: &Layer0ProgressPlan,
) -> Result<(Layer0ShardWorkerThreads, BTreeMap<usize, Layer0Tally>), String> {
    let at = || format!("第 {} 片的账本（{}）", shard.text(), path.display());
    let Some(whole_lines) = content.strip_suffix('\n') else {
        return Err(format!("{}不以换行结尾：没写完", at()));
    };
    let lines: Vec<&str> = whole_lines.split('\n').collect();
    let [header, shard_line, slice_lines @ .., end_line] = lines.as_slice() else {
        return Err(format!("{}不到三行（文件头、这一片的线程行、末行）", at()));
    };
    let header_body =
        body_of_a_checked_line(header).map_err(|problem| format!("{}文件头：{problem}", at()))?;
    if header_body != expected_header_body {
        return Err(format!(
            "{}文件头与这一趟 merge 不同：{}",
            at(),
            differing_fields(header_body, expected_header_body).join("；")
        ));
    }
    let (written_shard_index, owned_slices, threads) = body_of_a_checked_line(shard_line)
        .and_then(shard_line_from_body)
        .map_err(|problem| format!("{}第二行：{problem}", at()))?;
    if written_shard_index != shard.shard_index() {
        return Err(format!(
            "{}记的是第 {written_shard_index} 片：两份账本是同一片、第 {} 片的账本没有",
            at(),
            shard.shard_index()
        ));
    }
    let expected_owned_slices = slices_owned_by(shard, whole_plan.slices.len());
    if owned_slices != expected_owned_slices {
        return Err(format!(
            "{}记了 {owned_slices} 个切片归它，这一趟的切法下是 {expected_owned_slices} 个",
            at()
        ));
    }
    let end_body =
        body_of_a_checked_line(end_line).map_err(|problem| format!("{}末行：{problem}", at()))?;
    if end_body != shard_ledger_end_body(slice_lines.len()) {
        return Err(format!(
            "{}末行 {end_body:?} 对不上片行数 {}：没写完或多了行",
            at(),
            slice_lines.len()
        ));
    }
    let plan_of_this_shard = Layer0ProgressPlan {
        plan_hash: whole_plan.plan_hash.clone(),
        state_count: whole_plan.state_count,
        slices: whole_plan.slices.clone(),
        has_observer: whole_plan.has_observer,
        shard: Some(shard),
    };
    let slice_tallies = restored_slice_lines(slice_lines, &plan_of_this_shard)
        .map_err(|problem| format!("{}片行：{problem}", at()))?;
    if slice_tallies.len() != slice_lines.len() {
        return Err(format!(
            "{}有切片出现两次（{} 行片行、{} 个切片）",
            at(),
            slice_lines.len(),
            slice_tallies.len()
        ));
    }
    let missing_slices: Vec<usize> = (0..whole_plan.slices.len())
        .filter(|slice_index| {
            shard.owns_slice(*slice_index) && !slice_tallies.contains_key(slice_index)
        })
        .collect();
    if !missing_slices.is_empty() {
        return Err(format!("{}缺切片 {missing_slices:?}", at()));
    }
    Ok((threads, slice_tallies))
}

/// merge 读 n 份账本、逐份核齐：n 份都在，每份照 [`slices_of_one_shard_ledger`] 核过；交回按片排的线程行与按切片序号的计数
/// （切片 0..S 每个恰好一次：每份恰好是归它那一片的切片，n 片把 0..S 分完）。`whole_plan` 是这一趟 merge 现算的整条流的计划（不带片）。
///
/// # Errors
/// 缺了哪几片的账本（逐片列路径）；哪一片的账本哪一处不对（给人看的一句）。
pub fn read_shard_ledgers_for_merge(
    merge: &Layer0ShardMerge,
    whole_plan: &Layer0ProgressPlan,
) -> Result<Layer0MergedShardLedgers, String> {
    let expected_header_body = shard_ledger_header_body(
        &merge.input_fingerprint,
        &merge.stream_name,
        whole_plan,
        merge.shard_count,
        &merge.toolchain,
    );
    let shards: Vec<Layer0ShardOfShards> = (0..merge.shard_count.get())
        .map(|shard_index| {
            Layer0ShardOfShards::new(shard_index, merge.shard_count)
                .expect("0..n 里的每一个都小于 n")
        })
        .collect();
    let mut contents: Vec<(Layer0ShardOfShards, PathBuf, String)> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    for shard in shards {
        let path = shard_ledger_path(&merge.directory, &merge.stream_name, shard);
        match std::fs::read(&path) {
            Ok(bytes) => {
                let content = String::from_utf8(bytes).map_err(|error| {
                    format!(
                        "第 {} 片的账本（{}）不是 UTF-8：{error}",
                        shard.text(),
                        path.display()
                    )
                })?;
                contents.push((shard, path, content));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(format!("第 {} 片（{}）", shard.text(), path.display()));
            }
            Err(error) => {
                return Err(format!(
                    "读第 {} 片的账本（{}）：{error}",
                    shard.text(),
                    path.display()
                ))
            }
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "缺 {} 份账本（共 {} 片）：{}",
            missing.len(),
            merge.shard_count,
            missing.join("、")
        ));
    }
    let mut merged = Layer0MergedShardLedgers {
        ledger_paths: Vec::new(),
        worker_threads_by_shard: Vec::new(),
        slice_tallies: BTreeMap::new(),
    };
    for (shard, path, content) in contents {
        let (threads, slice_tallies) =
            slices_of_one_shard_ledger(shard, &path, &content, &expected_header_body, whole_plan)?;
        merged.slice_tallies.extend(slice_tallies);
        merged.worker_threads_by_shard.push(threads);
        merged.ledger_paths.push(path);
    }
    assert_eq!(
        merged.slice_tallies.keys().copied().collect::<Vec<usize>>(),
        (0..whole_plan.slices.len()).collect::<Vec<usize>>(),
        "n 份账本各自恰好是归它那一片的切片，并起来是 0..S 每个恰好一次"
    );
    Ok(merged)
}

/// 发现日志格式的版本：写进每一节的 begin 行，格式改了就换号。
const FINDINGS_LOG_FORMAT: u32 = 1;
/// 不留进度文件的枚举没有流名，begin 行的 `stream=` 写这个。
const FINDINGS_LOG_UNNAMED_STREAM: &str = "unnamed";
/// 定稿里每个样本的原文字段名，依次是第 1、2、3 个样本（长度跟着 [`LAYER0_FINDING_SAMPLES_KEPT`] 走：改了样本数这里编译不过）。
const FINDINGS_LOG_SAMPLE_VIOLATION_KEYS: [&str; LAYER0_FINDING_SAMPLES_KEPT] = [
    "sample_violation_1",
    "sample_violation_2",
    "sample_violation_3",
];

/// 层 0 枚举写不写发现日志、写到哪。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layer0FindingsLog {
    /// 不写（标准输出上的 `LAYER0_FINDING` / `LAYER0_FINDINGS` 照打）。
    NotWritten,
    /// 每趟枚举往这个文件末尾追加一节（[`Layer0FindingsLogSection`]）。
    AppendedTo(PathBuf),
}

impl Layer0FindingsLog {
    /// 从 [`LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE`] 取。
    ///
    /// # Panics
    /// 设了却是空串：配错了就停，不悄悄不写。
    #[must_use]
    pub fn from_environment() -> Self {
        Self::from_environment_value(std::env::var_os(LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE))
    }

    /// [`Self::from_environment`] 的判定本身：环境变量读到什么由调用方给（用例不改进程的环境变量）。
    ///
    /// # Panics
    /// 同 [`Self::from_environment`]。
    #[must_use]
    pub fn from_environment_value(value: Option<OsString>) -> Self {
        match value {
            None => Self::NotWritten,
            Some(path) => {
                assert!(
                    !path.is_empty(),
                    "{LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE} 设了却是空串"
                );
                Self::AppendedTo(PathBuf::from(path))
            }
        }
    }
}

/// 一节发现日志的 begin 行说的事：哪条流、整条流几个状态、是不是分片跑的一片。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer0FindingsLogBegin {
    /// 续跑、分片、merge 用的流名；不留进度文件的枚举没有流名。
    pub stream_name: Option<String>,
    pub whole_stream_states: u64,
    /// 分片跑一片时：第几片、这一片负责的状态数。整条流（单机跑完，或 merge）时没有：merge 的定稿与单机跑的逐字节相同。
    pub shard: Option<(Layer0ShardOfShards, u64)>,
}

/// 同一个进程里同一时刻只开一节发现日志：跑完那一下要把这一节截到起点重写，别的节插在中间会被一起截掉。
static FINDINGS_LOG_SECTION_OPEN_IN_THIS_PROCESS: Mutex<()> = Mutex::new(());

/// 开着的一节发现日志：跑的过程中逐行追加、每行落盘（中途读得到）；跑完换成定稿（[`Self::replace_with_final_lines`]）。
/// 没换成定稿就结束的（被杀、panic）留着已经追加的那些行，没有汇总行。
pub struct Layer0FindingsLogSection {
    path: PathBuf,
    appender: File,
    /// 这一节从文件里第几个字节起：开这一节时文件的长度。
    section_start_in_bytes: u64,
    _only_section_open_in_this_process: MutexGuard<'static, ()>,
}

impl Layer0FindingsLogSection {
    /// 在 `path` 末尾开一节：先等进程里别的节写完，没有文件就建（连同目录），记下这一节的起点，追加 begin 行并落盘。
    ///
    /// # Panics
    /// 建目录、打开、写、落盘失败。
    #[must_use]
    pub fn begin(path: &Path, begin: &Layer0FindingsLogBegin) -> Self {
        // 锁中毒说明上一节所在的枚举 panic 了：那一节照原样留着（没有汇总行），这一节照开。
        let only_section_open_in_this_process = FINDINGS_LOG_SECTION_OPEN_IN_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(directory) = path
            .parent()
            .filter(|directory| !directory.as_os_str().is_empty())
        {
            std::fs::create_dir_all(directory).unwrap_or_else(|error| {
                panic!("建发现日志的目录 {}：{error}", directory.display())
            });
        }
        let appender = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap_or_else(|error| panic!("打开发现日志 {}：{error}", path.display()));
        let section_start_in_bytes = appender
            .metadata()
            .unwrap_or_else(|error| panic!("读发现日志 {} 的长度：{error}", path.display()))
            .len();
        let mut section = Self {
            path: path.to_path_buf(),
            appender,
            section_start_in_bytes,
            _only_section_open_in_this_process: only_section_open_in_this_process,
        };
        section.append_line(&findings_log_begin_line(begin));
        section
    }

    /// 追加一行（补上换行）并落盘。
    ///
    /// # Panics
    /// 写或落盘失败。
    pub fn append_line(&mut self, line: &str) {
        self.appender
            .write_all(format!("{line}\n").as_bytes())
            .unwrap_or_else(|error| panic!("写发现日志 {}：{error}", self.path.display()));
        self.appender
            .sync_data()
            .unwrap_or_else(|error| panic!("落盘发现日志 {}：{error}", self.path.display()));
    }

    /// 跑完：把这一节截到起点，写定稿（每行补上换行），落盘。前面别的节不动。
    ///
    /// # Panics
    /// 截短、写或落盘失败。
    pub fn replace_with_final_lines(mut self, final_lines: &[String]) {
        self.appender
            .set_len(self.section_start_in_bytes)
            .unwrap_or_else(|error| panic!("截短发现日志 {}：{error}", self.path.display()));
        let final_section: String = final_lines.iter().map(|line| format!("{line}\n")).collect();
        self.appender
            .write_all(final_section.as_bytes())
            .unwrap_or_else(|error| panic!("写发现日志 {}：{error}", self.path.display()));
        self.appender
            .sync_all()
            .unwrap_or_else(|error| panic!("落盘发现日志 {}：{error}", self.path.display()));
    }
}

/// 发现日志与 `LAYER0_FINDING` 行里的违例原文：`\` 写成 `\\`、制表符写成 `\t`、换行写成 `\n`、回车写成 `\r`，其余原样——
/// 一条一行、按制表符切得开。
#[must_use]
pub fn escaped_finding_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\t' => escaped.push_str("\\t"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            unescaped_character => escaped.push(unescaped_character),
        }
    }
    escaped
}

/// [`escaped_finding_text`] 反过来。
///
/// # Errors
/// `\` 后面跟的不是 `\`、`t`、`n`、`r` 之一，或 `\` 在末尾。
pub fn unescaped_finding_text(text: &str) -> Result<String, String> {
    let mut unescaped = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            unescaped.push(character);
            continue;
        }
        match characters.next() {
            Some('\\') => unescaped.push('\\'),
            Some('t') => unescaped.push('\t'),
            Some('n') => unescaped.push('\n'),
            Some('r') => unescaped.push('\r'),
            Some(unknown_escape) => {
                return Err(format!("转义解不开：\\{unknown_escape}（{text:?}）"));
            }
            None => return Err(format!("转义解不开：\\ 在末尾（{text:?}）")),
        }
    }
    Ok(unescaped)
}

/// 签名的四样（pass、violated、段、发布），发现日志与 `LAYER0_FINDING` 行同一个写法。
fn finding_signature_fields(signature: &Layer0FindingSignature) -> [(&'static str, String); 4] {
    [
        ("pass", signature.red_pass.pass_name().to_string()),
        ("violated", signature.red_pass.violated_names().join(",")),
        ("segment", signature.segment.name()),
        (
            "publish",
            signature.publish.name_with_the_root_write_index(),
        ),
    ]
}

/// 发现日志的一行：种类，再逐个 `\t<key>=<value>`。
fn tab_separated_findings_line(kind: &str, fields: &[(&str, String)]) -> String {
    fields.iter().fold(kind.to_string(), |line, (key, value)| {
        format!("{line}\t{key}={value}")
    })
}

/// 标准输出的一行：前缀，再逐个 ` <key>=<value>`。
fn space_separated_findings_line(prefix: &str, fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .fold(prefix.to_string(), |line, (key, value)| {
            format!("{line} {key}={value}")
        })
}

fn new_signature_fields(
    number: usize,
    signature: &Layer0FindingSignature,
    first_sample: &Layer0FindingSample,
) -> Vec<(&'static str, String)> {
    let mut fields = vec![("finding", number.to_string())];
    fields.extend(finding_signature_fields(signature));
    fields.push(("first_state", first_sample.state_ordinal.to_string()));
    fields.push((
        "first_violation",
        escaped_finding_text(&first_sample.violation),
    ));
    fields
}

fn threshold_fields(
    number: usize,
    signature: &Layer0FindingSignature,
    states_at_least: u64,
) -> Vec<(&'static str, String)> {
    let mut fields = vec![("finding", number.to_string())];
    fields.extend(finding_signature_fields(signature));
    fields.push(("states_at_least", states_at_least.to_string()));
    fields
}

/// 一节的 begin 行：`layer0_findings_begin`，字段 `format=1`、`stream=<流名或 unnamed>`、`states=<整条流>`，
/// 分片跑一片另带 `shard=`、`shard_states=`。
#[must_use]
pub fn findings_log_begin_line(begin: &Layer0FindingsLogBegin) -> String {
    let mut fields = vec![
        ("format", FINDINGS_LOG_FORMAT.to_string()),
        (
            "stream",
            begin
                .stream_name
                .clone()
                .unwrap_or_else(|| FINDINGS_LOG_UNNAMED_STREAM.to_string()),
        ),
        ("states", begin.whole_stream_states.to_string()),
    ];
    if let Some((shard_of_this_run, states_of_this_shard)) = begin.shard {
        fields.push(("shard", shard_of_this_run.text()));
        fields.push(("shard_states", states_of_this_shard.to_string()));
    }
    tab_separated_findings_line("layer0_findings_begin", &fields)
}

/// 跑的过程中：一个签名第一次出现（发现日志那一行）。
#[must_use]
pub fn findings_log_new_signature_line(
    number: usize,
    signature: &Layer0FindingSignature,
    first_sample: &Layer0FindingSample,
) -> String {
    tab_separated_findings_line(
        "layer0_finding_new",
        &new_signature_fields(number, signature, first_sample),
    )
}

/// 跑的过程中：一个签名的状态数跨过一级台阶（发现日志那一行）。
#[must_use]
pub fn findings_log_threshold_line(
    number: usize,
    signature: &Layer0FindingSignature,
    states_at_least: u64,
) -> String {
    tab_separated_findings_line(
        "layer0_finding_threshold",
        &threshold_fields(number, signature, states_at_least),
    )
}

/// 标准输出：一个签名第一次出现，`LAYER0_FINDING event=new …`（`first_violation=` 一直到行尾）。
#[must_use]
pub fn findings_standard_output_new_signature_line(
    number: usize,
    signature: &Layer0FindingSignature,
    first_sample: &Layer0FindingSample,
) -> String {
    let mut fields = vec![("event", "new".to_string())];
    fields.extend(new_signature_fields(number, signature, first_sample));
    space_separated_findings_line("LAYER0_FINDING", &fields)
}

/// 标准输出：一个签名的状态数跨过一级台阶，`LAYER0_FINDING event=threshold …`。
#[must_use]
pub fn findings_standard_output_threshold_line(
    number: usize,
    signature: &Layer0FindingSignature,
    states_at_least: u64,
) -> String {
    let mut fields = vec![("event", "threshold".to_string())];
    fields.extend(threshold_fields(number, signature, states_at_least));
    space_separated_findings_line("LAYER0_FINDING", &fields)
}

/// 标准输出：一趟枚举跑完，`LAYER0_FINDINGS signatures=… red_states=… states=…`（分片跑一片时调用方另接上分片的字段）。
#[must_use]
pub fn findings_standard_output_summary_line(
    findings: &Layer0Findings,
    states_of_this_run: u64,
) -> String {
    space_separated_findings_line(
        "LAYER0_FINDINGS",
        &[
            ("signatures", findings.by_signature.len().to_string()),
            ("red_states", findings.red_states.to_string()),
            ("states", states_of_this_run.to_string()),
        ],
    )
}

/// 一节的定稿：begin 行，每个签名一行 `layer0_finding`（号按 [`Layer0Findings::signatures_in_first_state_order`]），
/// 末尾一行 `layer0_findings_summary`。只看发现表与 begin：与线程数、切法、续跑与否无关。
#[must_use]
pub fn findings_log_final_lines(
    begin: &Layer0FindingsLogBegin,
    findings: &Layer0Findings,
    states_of_this_section: u64,
) -> Vec<String> {
    let ordered = findings.signatures_in_first_state_order();
    let mut lines = vec![findings_log_begin_line(begin)];
    for (position, (signature, finding)) in ordered.iter().enumerate() {
        let mut fields = vec![("finding", (position + 1).to_string())];
        fields.extend(finding_signature_fields(signature));
        fields.push(("states", finding.states.to_string()));
        fields.push((
            "sample_states",
            finding
                .earliest_samples
                .iter()
                .map(|sample| sample.state_ordinal.to_string())
                .collect::<Vec<String>>()
                .join(","),
        ));
        for (key, sample) in FINDINGS_LOG_SAMPLE_VIOLATION_KEYS
            .iter()
            .zip(&finding.earliest_samples)
        {
            fields.push((key, escaped_finding_text(&sample.violation)));
        }
        lines.push(tab_separated_findings_line("layer0_finding", &fields));
    }
    let states_by_finding: Vec<String> = ordered
        .iter()
        .enumerate()
        .map(|(position, (_signature, finding))| format!("{}:{}", position + 1, finding.states))
        .collect();
    lines.push(tab_separated_findings_line(
        "layer0_findings_summary",
        &[
            ("signatures", ordered.len().to_string()),
            ("red_states", findings.red_states.to_string()),
            ("states", states_of_this_section.to_string()),
            (
                "states_by_finding",
                if states_by_finding.is_empty() {
                    "none".to_string()
                } else {
                    states_by_finding.join(",")
                },
            ),
        ],
    ));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> Layer0ProgressFileSettings {
        Layer0ProgressFileSettings {
            directory: PathBuf::from("/nonexistent"),
            input_fingerprint: Layer0ProgressFileNamePart::new("fingerprint0").expect("合法"),
            stream_name: Layer0ProgressFileNamePart::new("stream_a").expect("合法"),
            start: Layer0ResumeStart::ResumeFromTheProgressFile,
            after_completion: Layer0ProgressFileAfterCompletion::Deleted,
        }
    }

    fn plan(has_observer: bool) -> Layer0ProgressPlan {
        Layer0ProgressPlan {
            plan_hash: "ab".repeat(32),
            state_count: 5,
            slices: vec![0..2, 2..4, 4..5],
            has_observer,
            shard: None,
        }
    }

    /// 每个字段都不是默认值的一片计数：往返之后逐项相同，才说明每个字段都写进去了、都读回来了。
    fn tally_with_every_field_set(states: u64, observed_states: u64) -> Layer0Tally {
        let mut observer_counts = Layer0ObserverCounts::default();
        observer_counts.add("states_through_the_landed_root", 1);
        Layer0Tally {
            states,
            states_by_publish: BTreeMap::from([
                (
                    Layer0PublishOfState::UpToTheRootOf {
                        root_write_index: 7,
                        instance: InstanceGeneration(2),
                        checkpoint_txg: CheckpointTxg(9),
                    },
                    1,
                ),
                (Layer0PublishOfState::AfterTheLastRoot, states - 1),
            ]),
            violations: 1,
            root_persisted_states: 2,
            no_file_states: 3,
            file_read_states: 4,
            failed_states: 5,
            journal_differing_states: 6,
            verification_ran_states: 7,
            verification_failed_states: 8,
            first_violation: Some("读回的内容不对（走的是第 9 代根） = , :".to_string()),
            ignored_violations: 9,
            first_ignored_violation: Some("另一处".to_string()),
            record_root_without_record: 10,
            record_claimed_state_missing_unit: 11,
            checker_evaluated_states: BTreeMap::from([("I-3.1", 12)]),
            checker_violated_states: BTreeMap::from([("I-3.1", 13)]),
            checker_first_violation: BTreeMap::from([("I-3.1", "细节 带空格".to_string())]),
            checker_not_applicable_states: BTreeMap::from([("I-7.9", 14)]),
            observed_states,
            observer_counts,
            findings: findings_with_every_kind_of_signature(),
        }
    }

    /// 四遍各一个签名（段、发布各种写法都有），样本原文带制表符、换行、回车、反斜杠、空格、`=`、`,`、`:`：往返之后逐项相同。
    fn findings_with_every_kind_of_signature() -> Layer0Findings {
        let sample = |state_ordinal: u64, violation: &str| Layer0FindingSample {
            state_ordinal,
            violation: violation.to_string(),
        };
        Layer0Findings {
            red_states: 2,
            by_signature: BTreeMap::from([
                (
                    Layer0FindingSignature {
                        red_pass: Layer0RedPass::JournalConsultedOracle(
                            Layer0OracleViolationKind::WrongContent,
                        ),
                        segment: Layer0SegmentOfState::Segment(3),
                        publish: Layer0PublishOfState::UpToTheRootOf {
                            root_write_index: 7,
                            instance: InstanceGeneration(2),
                            checkpoint_txg: CheckpointTxg(9),
                        },
                    },
                    Layer0Finding {
                        states: 2,
                        earliest_samples: vec![
                            sample(2, "读回的内容不对\t（制表符）\n第二行\r\\ 反斜杠 = , :"),
                            sample(3, "第二个样本"),
                        ],
                    },
                ),
                (
                    Layer0FindingSignature {
                        red_pass: Layer0RedPass::JournalIgnoredOracle(
                            Layer0OracleViolationKind::NoRootChosen,
                        ),
                        segment: Layer0SegmentOfState::Segment(0),
                        publish: Layer0PublishOfState::AfterTheLastRoot,
                    },
                    Layer0Finding {
                        states: 1,
                        earliest_samples: vec![sample(2, "没择到根")],
                    },
                ),
                (
                    Layer0FindingSignature {
                        red_pass: Layer0RedPass::PoolChecker(vec!["I-3.1", "I-7.9"]),
                        segment: Layer0SegmentOfState::AllPersisted,
                        publish: Layer0PublishOfState::EveryWritePersisted,
                    },
                    Layer0Finding {
                        states: 1,
                        earliest_samples: vec![sample(3, "I-3.1：细节；I-7.9：细节")],
                    },
                ),
                (
                    Layer0FindingSignature {
                        red_pass: Layer0RedPass::RecordChecker(RecordCheck {
                            root_without_record: true,
                            claimed_state_missing_unit: true,
                        }),
                        segment: Layer0SegmentOfState::Segment(1),
                        publish: Layer0PublishOfState::AfterTheLastRoot,
                    },
                    Layer0Finding {
                        states: 1,
                        earliest_samples: vec![sample(2, "两条判据都成立")],
                    },
                ),
            ]),
        }
    }

    fn file_with(lines: &[String], ends_with_newline: bool) -> String {
        let mut content = lines.join("\n");
        if ends_with_newline {
            content.push('\n');
        }
        content
    }

    fn header(has_observer: bool) -> String {
        line_with_checksum(&header_body(&settings(), &plan(has_observer)))
    }

    /// 片行写出去再读回来，每个字段逐项相同。
    #[test]
    fn a_slice_line_reads_back_to_the_same_tally_field_by_field() {
        let tally = tally_with_every_field_set(2, 2);
        let content = file_with(&[header(true), slice_line(1, &(2..4), &tally)], true);
        let (restored, half_lines_dropped) =
            restored_slices_of(&content, &settings(), &plan(true)).expect("整份读得回来");
        assert_eq!(half_lines_dropped, 0);
        assert_eq!(restored, BTreeMap::from([(1, tally)]));
    }

    /// 末尾没换行的半行丢掉，前面的整行照收（R3）。
    #[test]
    fn a_half_written_last_line_is_dropped_and_the_whole_lines_before_it_are_kept() {
        let first = tally_with_every_field_set(2, 2);
        let second_line = slice_line(1, &(2..4), &tally_with_every_field_set(2, 2));
        let half = &second_line[..second_line.len() / 2];
        let content = format!(
            "{}\n{}\n{half}",
            header(true),
            slice_line(0, &(0..2), &first)
        );
        let (restored, half_lines_dropped) =
            restored_slices_of(&content, &settings(), &plan(true)).expect("半行丢掉、其余照收");
        assert_eq!(half_lines_dropped, 1);
        assert_eq!(restored, BTreeMap::from([(0, first)]));
    }

    /// 整行里少一个字段（连校验和一起重算过，校验和对得上）：整份作废，不按 0 收（U2）。
    #[test]
    fn a_whole_line_missing_one_field_voids_the_whole_file() {
        let line = slice_line(0, &(0..2), &tally_with_every_field_set(2, 2));
        let (body, _checksum) = line.rsplit_once(" checksum=").expect("有校验和");
        let without_violations: Vec<&str> = body
            .split(' ')
            .filter(|field| !field.starts_with("violations="))
            .collect();
        let content = file_with(
            &[
                header(true),
                line_with_checksum(&without_violations.join(" ")),
            ],
            true,
        );
        let voided =
            restored_slices_of(&content, &settings(), &plan(true)).expect_err("少一个字段整份作废");
        assert!(voided.contains("字段对不上"), "{voided}");
    }

    /// 整行的校验和对不上（中间一个字节坏了）：整份作废。
    #[test]
    fn a_whole_line_with_a_wrong_checksum_voids_the_whole_file() {
        let line = slice_line(0, &(0..2), &tally_with_every_field_set(2, 2));
        let corrupted = line.replacen("violations=1", "violations=7", 1);
        let content = file_with(&[header(true), corrupted], true);
        let voided = restored_slices_of(&content, &settings(), &plan(true))
            .expect_err("校验和对不上整份作废");
        assert!(voided.contains("校验和对不上"), "{voided}");
    }

    /// 片方案对不上（文件头的片数、片的区间）、观察者看过的状态数对不上：整份作废（R2、U3）。
    #[test]
    fn a_slice_plan_or_observed_state_count_that_does_not_match_voids_the_whole_file() {
        let other_plan = Layer0ProgressPlan {
            slices: vec![0..3, 3..5],
            ..plan(true)
        };
        let content = file_with(
            &[line_with_checksum(&header_body(&settings(), &other_plan))],
            true,
        );
        assert!(restored_slices_of(&content, &settings(), &plan(true))
            .expect_err("文件头的片方案对不上")
            .contains("文件头对不上"));
        let wrong_range = file_with(
            &[
                header(true),
                slice_line(1, &(2..3), &tally_with_every_field_set(1, 1)),
            ],
            true,
        );
        assert!(restored_slices_of(&wrong_range, &settings(), &plan(true))
            .expect_err("片区间对不上")
            .contains("对不上这一趟的切法"));
        let observer_short = file_with(
            &[
                header(true),
                slice_line(1, &(2..4), &tally_with_every_field_set(2, 1)),
            ],
            true,
        );
        assert!(
            restored_slices_of(&observer_short, &settings(), &plan(true))
                .expect_err("观察者少看了一个状态")
                .contains("观察者看过")
        );
    }

    /// 同一片写了两次：计数相同去重，不同整份作废。
    #[test]
    fn the_same_slice_twice_is_deduplicated_only_when_the_counts_are_identical() {
        let tally = tally_with_every_field_set(2, 2);
        let twice = file_with(
            &[
                header(true),
                slice_line(0, &(0..2), &tally),
                slice_line(0, &(0..2), &tally),
            ],
            true,
        );
        let (restored, _) =
            restored_slices_of(&twice, &settings(), &plan(true)).expect("相同的去重");
        assert_eq!(restored.len(), 1);
        let mut different = tally.clone();
        different.violations += 1;
        let conflicting = file_with(
            &[
                header(true),
                slice_line(0, &(0..2), &tally),
                slice_line(0, &(0..2), &different),
            ],
            true,
        );
        assert!(restored_slices_of(&conflicting, &settings(), &plan(true))
            .expect_err("同一片两次计数不同")
            .contains("出现两次"));
    }

    /// 环境变量：没设进度目录就不留；设了目录没设指纹停下；从头跑开关只认 0、1。
    #[test]
    fn the_environment_decides_whether_and_how_the_progress_file_is_kept() {
        use std::env::VarError::NotPresent;
        assert_eq!(
            Layer0Resume::from_environment_values(
                "stream_a",
                Err(NotPresent),
                Err(NotPresent),
                Err(NotPresent),
                Err(NotPresent),
                toolchain_never_asked
            ),
            Layer0Resume::NoProgressFile
        );
        let resume = Layer0Resume::from_environment_values(
            "stream_a",
            Ok("/tmp/progress".to_string()),
            Ok("fingerprint0".to_string()),
            Ok("1".to_string()),
            Err(NotPresent),
            toolchain_never_asked,
        );
        assert_eq!(
            resume,
            Layer0Resume::KeepProgressFile(Layer0ProgressFileSettings {
                directory: PathBuf::from("/tmp/progress"),
                input_fingerprint: Layer0ProgressFileNamePart::new("fingerprint0").expect("合法"),
                stream_name: Layer0ProgressFileNamePart::new("stream_a").expect("合法"),
                start: Layer0ResumeStart::StartOverDiscardingTheProgressFile,
                after_completion: Layer0ProgressFileAfterCompletion::Deleted,
            })
        );
        for (fingerprint, start_over) in [
            (Err(NotPresent), Err(NotPresent)),
            (Ok("has space".to_string()), Err(NotPresent)),
            (Ok("fingerprint0".to_string()), Ok("yes".to_string())),
        ] {
            let outcome = std::panic::catch_unwind(|| {
                Layer0Resume::from_environment_values(
                    "stream_a",
                    Ok("/tmp/progress".to_string()),
                    fingerprint.clone(),
                    start_over.clone(),
                    Err(NotPresent),
                    toolchain_never_asked,
                )
            });
            assert!(
                outcome.is_err(),
                "配错了要停：{fingerprint:?} {start_over:?}"
            );
        }
    }

    /// 没设分片开关时不问工具链（平时的 `cargo test` 不起子进程）。
    fn toolchain_never_asked() -> Layer0ToolchainIdentity {
        panic!("没设分片开关就不该问工具链")
    }

    fn toolchain_of_the_test() -> Layer0ToolchainIdentity {
        Layer0ToolchainIdentity {
            rustc_version_lines: "rustc 1.0.0 (test)\nbinary: rustc\ncommit-hash: test".to_string(),
            cargo_version: "cargo 1.0.0 (test)".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        }
    }

    fn shard_count(count: u32) -> NonZeroU32 {
        NonZeroU32::new(count).expect("片数不是 0")
    }

    /// 分片开关：`<i>/<n>` 跑一片、`merge/<n>` 只并账本，两样都带工具链；i 不小于 n、n 是 0、形态不对、没设进度目录、
    /// merge 带从头跑，都停下。切片按 `slice_index % n` 交错归片。
    #[test]
    fn the_shard_switch_decides_one_shard_or_merge_and_misconfiguration_stops() {
        use std::env::VarError::NotPresent;
        let progress = Layer0ProgressFileSettings {
            directory: PathBuf::from("/tmp/progress"),
            input_fingerprint: Layer0ProgressFileNamePart::new("fingerprint0").expect("合法"),
            stream_name: Layer0ProgressFileNamePart::new("stream_a").expect("合法"),
            start: Layer0ResumeStart::ResumeFromTheProgressFile,
            after_completion: Layer0ProgressFileAfterCompletion::Deleted,
        };
        let with_switch = |switch: &str| {
            Layer0Resume::from_environment_values(
                "stream_a",
                Ok("/tmp/progress".to_string()),
                Ok("fingerprint0".to_string()),
                Err(NotPresent),
                Ok(switch.to_string()),
                toolchain_of_the_test,
            )
        };
        assert_eq!(
            with_switch("1/3"),
            Layer0Resume::RunOneShardKeepingProgressFile(Layer0ShardRun {
                progress: progress.clone(),
                shard: Layer0ShardOfShards::new(1, shard_count(3)).expect("1 < 3"),
                toolchain: toolchain_of_the_test(),
            })
        );
        assert_eq!(
            with_switch("merge/2"),
            Layer0Resume::MergeShardLedgers(Layer0ShardMerge {
                directory: progress.directory.clone(),
                input_fingerprint: progress.input_fingerprint.clone(),
                stream_name: progress.stream_name.clone(),
                shard_count: shard_count(2),
                toolchain: toolchain_of_the_test(),
            })
        );
        for misconfigured in ["2/2", "0/0", "merge/0", "merge", "x/2", "1/ 2", "-1/2"] {
            let outcome = std::panic::catch_unwind(|| with_switch(misconfigured));
            assert!(outcome.is_err(), "分片开关 {misconfigured:?} 配错了要停");
        }
        let without_progress_directory = std::panic::catch_unwind(|| {
            Layer0Resume::from_environment_values(
                "stream_a",
                Err(NotPresent),
                Err(NotPresent),
                Err(NotPresent),
                Ok("0/2".to_string()),
                toolchain_of_the_test,
            )
        });
        assert!(
            without_progress_directory.is_err(),
            "分片开关设了、进度目录没设，要停"
        );
        let merge_starting_over = std::panic::catch_unwind(|| {
            Layer0Resume::from_environment_values(
                "stream_a",
                Ok("/tmp/progress".to_string()),
                Ok("fingerprint0".to_string()),
                Ok("1".to_string()),
                Ok("merge/2".to_string()),
                toolchain_of_the_test,
            )
        });
        assert!(merge_starting_over.is_err(), "merge 带从头跑要停");
        let second_of_three = Layer0ShardOfShards::new(1, shard_count(3)).expect("1 < 3");
        assert_eq!(
            (0..8)
                .filter(|slice_index| second_of_three.owns_slice(*slice_index))
                .collect::<Vec<usize>>(),
            vec![1, 4, 7],
            "第 1/3 片拿序号模 3 余 1 的切片"
        );
    }

    /// 分片跑的进度文件：文件名与文件头都带这一片，读回时不归这一片的切片整份作废。
    #[test]
    fn a_shard_progress_file_is_named_after_its_shard_and_holds_only_its_slices() {
        let shard = Layer0ShardOfShards::new(0, shard_count(2)).expect("0 < 2");
        let shard_plan = Layer0ProgressPlan {
            shard: Some(shard),
            ..plan(true)
        };
        let path = progress_file_path(&settings(), &shard_plan);
        assert_ne!(path, progress_file_path(&settings(), &plan(true)));
        assert!(
            path.to_string_lossy().ends_with("-shard-0-of-2.txt"),
            "{}",
            path.display()
        );
        let shard_header = line_with_checksum(&header_body(&settings(), &shard_plan));
        assert!(shard_header.contains(" shard=0/2 "), "{shard_header}");
        let owned = file_with(
            &[
                shard_header.clone(),
                slice_line(2, &(4..5), &tally_with_every_field_set(1, 1)),
            ],
            true,
        );
        let (restored, _) =
            restored_slices_of(&owned, &settings(), &shard_plan).expect("第 2 个切片归第 0/2 片");
        assert_eq!(restored.keys().copied().collect::<Vec<usize>>(), vec![2]);
        let not_owned = file_with(
            &[
                shard_header,
                slice_line(1, &(2..4), &tally_with_every_field_set(2, 2)),
            ],
            true,
        );
        assert!(restored_slices_of(&not_owned, &settings(), &shard_plan)
            .expect_err("第 1 个切片不归第 0/2 片")
            .contains("不归"));
        assert!(
            restored_slices_of(&owned, &settings(), &plan(true))
                .expect_err("单机那一格不认分片的文件头")
                .contains("文件头对不上"),
            "单机与分片的进度文件不混"
        );
    }

    /// 片行里的发现表形状不对（样本多于状态数、样本序号倒着、签名的状态数多于判红的状态数、有签名而判红的状态数是 0）：
    /// 校验和对得上也整份作废，不带着坏的发现表并片。
    #[test]
    fn a_findings_table_whose_shape_does_not_hold_voids_the_whole_file() {
        let with_findings = |findings: Layer0Findings| {
            let mut tally = tally_with_every_field_set(2, 2);
            tally.findings = findings;
            file_with(&[header(true), slice_line(1, &(2..4), &tally)], true)
        };
        let well_formed = findings_with_every_kind_of_signature();
        let first_signature = well_formed
            .by_signature
            .keys()
            .next()
            .expect("有签名")
            .clone();
        let mut more_samples_than_states = well_formed.clone();
        more_samples_than_states
            .by_signature
            .get_mut(&first_signature)
            .expect("有这个签名")
            .states = 1;
        let mut samples_in_reverse = well_formed.clone();
        samples_in_reverse
            .by_signature
            .get_mut(&first_signature)
            .expect("有这个签名")
            .earliest_samples
            .reverse();
        let mut more_states_than_red_states = well_formed.clone();
        more_states_than_red_states.red_states = 1;
        let mut signatures_without_red_states = well_formed;
        signatures_without_red_states.red_states = 0;
        for (broken, expected_reason) in [
            (more_samples_than_states, "形状不对"),
            (samples_in_reverse, "形状不对"),
            (more_states_than_red_states, "形状不对"),
            (signatures_without_red_states, "形状不对"),
        ] {
            let voided = restored_slices_of(&with_findings(broken), &settings(), &plan(true))
                .expect_err("发现表形状不对整份作废");
            assert!(voided.contains(expected_reason), "{voided}");
        }
    }

    /// 发现日志的环境变量：没设不写，设了写到那个文件，设了空串停下。
    #[test]
    fn the_environment_decides_whether_the_findings_log_is_written() {
        assert_eq!(
            Layer0FindingsLog::from_environment_value(None),
            Layer0FindingsLog::NotWritten
        );
        assert_eq!(
            Layer0FindingsLog::from_environment_value(Some(OsString::from("/tmp/findings.tsv"))),
            Layer0FindingsLog::AppendedTo(PathBuf::from("/tmp/findings.tsv"))
        );
        let empty = std::panic::catch_unwind(|| {
            Layer0FindingsLog::from_environment_value(Some(OsString::new()))
        })
        .expect_err("设了空串要停下");
        let message = empty.downcast_ref::<String>().cloned().unwrap_or_default();
        assert!(
            message.contains(LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE),
            "停下时说清是哪个环境变量：{message}"
        );
    }

    /// 违例原文转义之后一行装得下、按制表符切得开，反过来解得回原样；`\` 后面跟别的、或 `\` 在末尾解不开。
    #[test]
    fn escaped_finding_text_fits_one_tab_separated_field_and_reads_back() {
        let original = "第一行\t制表符\n第二行\r回车 \\ 反斜杠 \\t 字面的反斜杠 t";
        let escaped = escaped_finding_text(original);
        assert!(
            !escaped.contains(['\t', '\n', '\r']),
            "转义之后没有制表符、换行、回车：{escaped:?}"
        );
        assert_eq!(
            unescaped_finding_text(&escaped).as_deref(),
            Ok(original),
            "解得回原样"
        );
        assert!(unescaped_finding_text("末尾\\").is_err());
        assert!(unescaped_finding_text("\\x").is_err());
    }
}
