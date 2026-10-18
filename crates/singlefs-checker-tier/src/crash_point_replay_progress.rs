//! 崩溃枚举用例跑出来的两样文字：发现日志（出错那一份：每趟枚举往 `SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE` 指的文件末尾追加一节，
//! 红例按签名去重、留样本、收尾一行汇总；标准输出上的 `CRASH_POINT_REPLAY_FINDING` / `CRASH_POINT_REPLAY_FINDINGS` 两种行同一套字段）与跑这一趟的
//! 工具链身份（`rustc -Vv`、`cargo -V`、target triple，进判法单元指纹与结论版本，`crash_identity`）。
//!
//! 崩溃枚举用例判过的状态都在判定库里（`crash_library` 那一类的长期库，两段协议：流先入库、判段从库里读回判），进度文件与
//! 双机账本没有了：分片、导入、续跑都走库（`crash_state_enumeration`）。
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::crash::{
    CrashPointReplayFindingSample, CrashPointReplayFindingSignature, CrashPointReplayFindings,
    CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT,
};

/// 发现日志写到哪个文件（崩溃点重放放量的「出错那一份」，用户 2026-09-27 定「全量和错误双份日志」）：每趟枚举往末尾追加一节
/// （[`CrashPointReplayFindingsLogSection`]）。没设就不写（标准输出上的 `CRASH_POINT_REPLAY_FINDING` / `CRASH_POINT_REPLAY_FINDINGS` 照打）。
pub const CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE";

/// 编这个测试二进制、跑这一趟的工具链：`rustc -Vv` 前三行、`cargo -V`、target triple（`rustc -Vv` 的 `host:` 那一行：
/// 这几条流都不交叉编译，编出来的就是 host）。进判法单元指纹与结论版本（`crash_identity`）：两台的工具链不同，判定就落在不同的版本下，不互相复用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointReplayToolchainIdentity {
    pub rustc_version_lines: String,
    pub cargo_version: String,
    pub target_triple: String,
}

impl CrashPointReplayToolchainIdentity {
    /// 问跑这个测试的 cargo（环境变量 `CARGO`，cargo test 起测试二进制时设）与它旁边的 rustc（旁边没有就取 `PATH` 上的）。
    ///
    /// # Panics
    /// 环境里没有 `CARGO`（不是经 cargo test 起的）、两个命令起不来或退出码不是 0、输出不是 UTF-8、`rustc -Vv` 不到三行或没有 `host:` 行：
    /// 记不下工具链就不判，不悄悄记一个空的。
    #[must_use]
    pub fn of_the_cargo_running_this_test() -> Self {
        let cargo = PathBuf::from(std::env::var_os("CARGO").unwrap_or_else(|| {
            panic!(
                "要经 cargo test 起（环境里要有 CARGO）：判法单元指纹要记编这个测试二进制的工具链"
            )
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

/// 发现日志格式的版本：写进每一节的 begin 行，格式改了就换号。
const FINDINGS_LOG_FORMAT: u32 = 1;
/// 没有流名的枚举，begin 行的 `stream=` 写这个。
const FINDINGS_LOG_UNNAMED_STREAM: &str = "unnamed";
/// 定稿里每个样本的原文字段名，依次是第 1、2、3 个样本（长度跟着 [`CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT`] 走：改了样本数这里编译不过）。
const FINDINGS_LOG_SAMPLE_VIOLATION_KEYS: [&str; CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT] = [
    "sample_violation_1",
    "sample_violation_2",
    "sample_violation_3",
];

/// 崩溃点重放枚举写不写发现日志、写到哪。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CrashPointReplayFindingsLog {
    /// 不写（标准输出上的 `CRASH_POINT_REPLAY_FINDING` / `CRASH_POINT_REPLAY_FINDINGS` 照打）。
    NotWritten,
    /// 每趟枚举往这个文件末尾追加一节（[`CrashPointReplayFindingsLogSection`]）。
    AppendedTo(PathBuf),
}

impl CrashPointReplayFindingsLog {
    /// 从 [`CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE`] 取。
    ///
    /// # Panics
    /// 设了却是空串：配错了就停，不悄悄不写。
    #[must_use]
    pub fn from_environment() -> Self {
        Self::from_environment_value(std::env::var_os(
            CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE,
        ))
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
                    "{CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE} 设了却是空串"
                );
                Self::AppendedTo(PathBuf::from(path))
            }
        }
    }
}

/// 一节发现日志的 begin 行说的事：哪条流、整条流几个状态、是不是分片跑的一片。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointReplayFindingsLogBegin {
    /// 流名（崩溃枚举用例的项名）；没有流名的枚举是 `None`。
    pub stream_name: Option<String>,
    pub whole_stream_states: u64,
}

/// 同一个进程里同一时刻只开一节发现日志：跑完那一下要把这一节截到起点重写，别的节插在中间会被一起截掉。
static FINDINGS_LOG_SECTION_OPEN_IN_THIS_PROCESS: Mutex<()> = Mutex::new(());

/// 开着的一节发现日志：跑的过程中逐行追加、每行落盘（中途读得到）；跑完换成定稿（[`Self::replace_with_final_lines`]）。
/// 没换成定稿就结束的（被杀、panic）留着已经追加的那些行，没有汇总行。
pub struct CrashPointReplayFindingsLogSection {
    path: PathBuf,
    appender: File,
    /// 这一节从文件里第几个字节起：开这一节时文件的长度。
    section_start_in_bytes: u64,
    _only_section_open_in_this_process: MutexGuard<'static, ()>,
}

impl CrashPointReplayFindingsLogSection {
    /// 在 `path` 末尾开一节：先等进程里别的节写完，没有文件就建（连同目录），记下这一节的起点，追加 begin 行并落盘。
    ///
    /// # Panics
    /// 建目录、打开、写、落盘失败。
    #[must_use]
    pub fn begin(path: &Path, begin: &CrashPointReplayFindingsLogBegin) -> Self {
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

/// 发现日志与 `CRASH_POINT_REPLAY_FINDING` 行里的违例原文：`\` 写成 `\\`、制表符写成 `\t`、换行写成 `\n`、回车写成 `\r`，其余原样——
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

/// 签名的四样（pass、violated、段、发布），发现日志与 `CRASH_POINT_REPLAY_FINDING` 行同一个写法。
fn finding_signature_fields(
    signature: &CrashPointReplayFindingSignature,
) -> [(&'static str, String); 4] {
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
    signature: &CrashPointReplayFindingSignature,
    first_sample: &CrashPointReplayFindingSample,
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

/// 一节的 begin 行：`crash_point_replay_findings_begin`，字段 `format=1`、`stream=<流名或 unnamed>`、`states=<整条流>`。
#[must_use]
pub fn findings_log_begin_line(begin: &CrashPointReplayFindingsLogBegin) -> String {
    let fields = vec![
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
    tab_separated_findings_line("crash_point_replay_findings_begin", &fields)
}

/// 标准输出：一个签名第一次出现，`CRASH_POINT_REPLAY_FINDING event=new …`（`first_violation=` 一直到行尾）。
#[must_use]
pub fn findings_standard_output_new_signature_line(
    number: usize,
    signature: &CrashPointReplayFindingSignature,
    first_sample: &CrashPointReplayFindingSample,
) -> String {
    let mut fields = vec![("event", "new".to_string())];
    fields.extend(new_signature_fields(number, signature, first_sample));
    space_separated_findings_line("CRASH_POINT_REPLAY_FINDING", &fields)
}

/// 标准输出：一趟枚举跑完，`CRASH_POINT_REPLAY_FINDINGS signatures=… red_states=… states=…`（分片跑一片时调用方另接上分片的字段）。
#[must_use]
pub fn findings_standard_output_summary_line(
    findings: &CrashPointReplayFindings,
    states_of_this_run: u64,
) -> String {
    space_separated_findings_line(
        "CRASH_POINT_REPLAY_FINDINGS",
        &[
            ("signatures", findings.by_signature.len().to_string()),
            ("red_states", findings.red_states.to_string()),
            ("states", states_of_this_run.to_string()),
        ],
    )
}

/// 一节的定稿：begin 行，每个签名一行 `crash_point_replay_finding`（号按 [`CrashPointReplayFindings::signatures_in_first_state_order`]），
/// 末尾一行 `crash_point_replay_findings_summary`。只看发现表与 begin：与线程数、切法、续跑与否无关。
#[must_use]
pub fn findings_log_final_lines(
    begin: &CrashPointReplayFindingsLogBegin,
    findings: &CrashPointReplayFindings,
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
        lines.push(tab_separated_findings_line(
            "crash_point_replay_finding",
            &fields,
        ));
    }
    let states_by_finding: Vec<String> = ordered
        .iter()
        .enumerate()
        .map(|(position, (_signature, finding))| format!("{}:{}", position + 1, finding.states))
        .collect();
    lines.push(tab_separated_findings_line(
        "crash_point_replay_findings_summary",
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
    use std::collections::BTreeMap;

    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_harness::memory_pool::RecordCheck;

    use super::*;
    use crate::crash::{
        CrashPointReplayFinding, CrashPointReplayOracleViolationKind,
        CrashPointReplayPublishOfState, CrashPointReplayRedPass, CrashPointReplaySegmentOfState,
    };

    /// 四遍各一个签名（段、发布各种写法都有），样本原文带制表符、换行、回车、反斜杠、空格、`=`、`,`、`:`。
    fn findings_with_every_kind_of_signature() -> CrashPointReplayFindings {
        let sample = |state_ordinal: u64, violation: &str| CrashPointReplayFindingSample {
            state_ordinal,
            violation: violation.to_string(),
        };
        CrashPointReplayFindings {
            red_states: 2,
            by_signature: BTreeMap::from([
                (
                    CrashPointReplayFindingSignature {
                        red_pass: CrashPointReplayRedPass::JournalConsultedOracle(
                            CrashPointReplayOracleViolationKind::WrongContent,
                        ),
                        segment: CrashPointReplaySegmentOfState::Segment(3),
                        publish: CrashPointReplayPublishOfState::UpToTheRootOf {
                            root_write_index: 7,
                            instance: InstanceGeneration(2),
                            checkpoint_txg: CheckpointTxg(9),
                        },
                    },
                    CrashPointReplayFinding {
                        states: 2,
                        earliest_samples: vec![
                            sample(2, "读回的内容不对\t（制表符）\n第二行\r\\ 反斜杠 = , :"),
                            sample(3, "第二个样本"),
                        ],
                    },
                ),
                (
                    CrashPointReplayFindingSignature {
                        red_pass: CrashPointReplayRedPass::JournalIgnoredOracle(
                            CrashPointReplayOracleViolationKind::NoRootChosen,
                        ),
                        segment: CrashPointReplaySegmentOfState::Segment(0),
                        publish: CrashPointReplayPublishOfState::AfterTheLastRoot,
                    },
                    CrashPointReplayFinding {
                        states: 1,
                        earliest_samples: vec![sample(2, "没择到根")],
                    },
                ),
                (
                    CrashPointReplayFindingSignature {
                        red_pass: CrashPointReplayRedPass::PoolChecker(vec!["I-3.1", "I-7.9"]),
                        segment: CrashPointReplaySegmentOfState::AllPersisted,
                        publish: CrashPointReplayPublishOfState::EveryWritePersisted,
                    },
                    CrashPointReplayFinding {
                        states: 1,
                        earliest_samples: vec![sample(3, "I-3.1：细节；I-7.9：细节")],
                    },
                ),
                (
                    CrashPointReplayFindingSignature {
                        red_pass: CrashPointReplayRedPass::RecordChecker(RecordCheck {
                            root_without_record: true,
                            claimed_state_missing_unit: true,
                        }),
                        segment: CrashPointReplaySegmentOfState::Segment(1),
                        publish: CrashPointReplayPublishOfState::AfterTheLastRoot,
                    },
                    CrashPointReplayFinding {
                        states: 1,
                        earliest_samples: vec![sample(2, "两条判据都成立")],
                    },
                ),
            ]),
        }
    }

    /// 一节的定稿：begin 行带流名与状态数；每个签名一行、按最先那个状态的序号排号，样本原文转义之后一行装得下；末行汇总按号列每个签名的状态数。
    #[test]
    fn the_final_lines_list_every_signature_in_first_state_order_with_escaped_samples_and_a_summary(
    ) {
        let findings = findings_with_every_kind_of_signature();
        let begin = CrashPointReplayFindingsLogBegin {
            stream_name: Some("stream_a".to_string()),
            whole_stream_states: 40,
        };
        let lines = findings_log_final_lines(&begin, &findings, 40);
        assert_eq!(lines.len(), 1 + 4 + 1, "begin、四个签名、汇总");
        assert!(
            lines[0].starts_with("crash_point_replay_findings_begin\t")
                && lines[0].contains("\tstream=stream_a\t")
                && lines[0].ends_with("\tstates=40"),
            "{}",
            lines[0]
        );
        let ordered = findings.signatures_in_first_state_order();
        for (position, line) in lines[1..5].iter().enumerate() {
            let (signature, finding) = &ordered[position];
            assert!(
                line.starts_with(&format!(
                    "crash_point_replay_finding\tfinding={}\tpass={}\t",
                    position + 1,
                    signature.red_pass.pass_name()
                )),
                "{line}"
            );
            assert!(
                line.contains(&format!("\tstates={}\t", finding.states)),
                "{line}"
            );
            assert!(
                !line.contains(['\n', '\r']),
                "样本原文转义之后一行装得下：{line}"
            );
        }
        assert!(
            lines[5].starts_with("crash_point_replay_findings_summary\tsignatures=4\tred_states=2\tstates=40\tstates_by_finding=1:"),
            "{}",
            lines[5]
        );
    }

    /// 发现日志的环境变量：没设不写，设了写到那个文件，设了空串停下。
    #[test]
    fn the_environment_decides_whether_the_findings_log_is_written() {
        assert_eq!(
            CrashPointReplayFindingsLog::from_environment_value(None),
            CrashPointReplayFindingsLog::NotWritten
        );
        assert_eq!(
            CrashPointReplayFindingsLog::from_environment_value(Some(OsString::from(
                "/tmp/findings.tsv"
            ))),
            CrashPointReplayFindingsLog::AppendedTo(PathBuf::from("/tmp/findings.tsv"))
        );
        let empty = std::panic::catch_unwind(|| {
            CrashPointReplayFindingsLog::from_environment_value(Some(OsString::new()))
        })
        .expect_err("设了空串要停下");
        let message = empty.downcast_ref::<String>().cloned().unwrap_or_default();
        assert!(
            message.contains(CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE),
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
