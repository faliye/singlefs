//! 导出文件：中性格式的十个文件、调用结局与返回值、流的状态、oracle 判与可写挂载探针的文件。

use super::*;

// ───────────────────────────── 一条流导出（登记 5.3 的十个文件，照抄 E170 的写出） ─────────────────────────────

/// 写出的一个文件：名字、字节数、sha256；带耗时的两个文件另算一份去掉耗时之后的 sha256（登记第十二节修订 V2b）。
pub(super) struct WrittenFile {
    pub(super) name: &'static str,
    pub(super) bytes: usize,
    pub(super) sha256: String,
    pub(super) sha256_without_timing: Option<String>,
}

/// `impl-verdicts.tsv` 每行去掉最后一列（耗时）；`manifest.tsv` 去掉记 `impl-verdicts.tsv` 摘要的那一行。其余文件原样。
fn contents_without_timing(file_name: &str, contents: &[u8]) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(contents).ok()?;
    let kept: Vec<String> = match file_name {
        IMPLEMENTATION_VERDICTS_FILE => text
            .lines()
            .map(|line| {
                line.rsplit_once('\t')
                    .map_or(line, |(without_timing, _)| without_timing)
                    .to_string()
            })
            .collect(),
        MANIFEST_FILE => text
            .lines()
            .filter(|line| !line.starts_with(&format!("sha256:{IMPLEMENTATION_VERDICTS_FILE}\t")))
            .map(str::to_string)
            .collect(),
        _ => return None,
    };
    Some(
        kept.iter()
            .flat_map(|line| format!("{line}\n").into_bytes())
            .collect(),
    )
}

fn write_export_file(directory: &Path, name: &'static str, contents: &[u8]) -> WrittenFile {
    std::fs::write(directory.join(name), contents)
        .unwrap_or_else(|error| panic!("写 {}：{error}", directory.join(name).display()));
    WrittenFile {
        name,
        bytes: contents.len(),
        sha256: sha256_hexadecimal(contents),
        sha256_without_timing: contents_without_timing(name, contents)
            .map(|kept| sha256_hexadecimal(&kept)),
    }
}

/// `base-sectors.bin` 的内容（登记 5.3）：基镜像里每个不全为零的扇区，盘身份 u32、扇区号 u64、512 字节。
/// 第一段写导出、第三段核导出与重驱动的流相同（`bad-states` 模式）共用这一份。
fn base_sectors_file_bytes(stream: &PreparedStream) -> Vec<u8> {
    let mut base_sectors = Vec::new();
    for (device, sparse) in &stream.base.devices {
        for (sector, bytes) in sparse.written_sectors() {
            if bytes.iter().all(|byte| *byte == 0) {
                continue;
            }
            base_sectors.extend_from_slice(&device.0.to_le_bytes());
            base_sectors.extend_from_slice(&sector.to_le_bytes());
            base_sectors.extend_from_slice(bytes);
        }
    }
    base_sectors
}

/// `ops.tsv` 的文本与 `ops-payload.bin` 的字节（登记 5.3），同上两种模式共用。
fn operations_file_text_and_payload(stream: &PreparedStream) -> (String, Vec<u8>) {
    let mut operations_text = String::new();
    let mut payload = Vec::new();
    for (stream_index, retained) in stream.operations.iter().enumerate() {
        let operation = &retained.operation;
        let (payload_start, content_digest) = match operation.kind {
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                let contents = retained
                    .contents
                    .as_ref()
                    .expect("流开了内容保留，普通写与 FUA 写带字节");
                let start = payload.len();
                payload.extend_from_slice(contents);
                (start.to_string(), sha256_hexadecimal(contents))
            }
            RecordedOperationKind::WriteZeroes
            | RecordedOperationKind::Discard
            | RecordedOperationKind::Barrier => ("-".to_string(), "-".to_string()),
        };
        operations_text.push_str(&format!(
            "{stream_index}\t{}\t{}\t{}\t{}\t{payload_start}\t{content_digest}\n",
            operation.device.0,
            operation.kind.name(),
            operation.offset.0,
            operation.length,
        ));
    }
    (operations_text, payload)
}

/// `calls.tsv` 的文本（E170 登记 5.3）：调用序号、名字、发出的第一条 op 的流序下标、返回点、`ok` / `err`。
fn calls_file_text(stream: &PreparedStream) -> String {
    stream
        .calls
        .iter()
        .enumerate()
        .map(|(call_index, call)| {
            format!(
                "{call_index}\t{}\t{}\t{}\t{}\n",
                call.label,
                call.first_operation,
                call.return_point,
                call.outcome.calls_file_word()
            )
        })
        .collect()
}

/// tsv 里的自由文本：制表符与换行换成空格。
pub(super) fn cell_text(text: &str) -> String {
    text.replace(['\t', '\n', '\r'], " ")
}

/// `call-outcomes.tsv`：每次调用一行——序号、名字、第一条 op、返回点、结局（`ok` / `err:<成员>` / `panic` / `in_flight`）、O1-对拍的判定、
/// 对不上时模型与实现各说了什么、panic 的消息，加第 1 次重跑的两列：交回值的 sha256（没有交回值写 `-`）、内存对象的合摘要
/// （各对象 `名字:sha256` 逗号接起来的 sha256）。不含耗时（两次基线逐字节相同，作废条款 Z1）。
fn call_outcomes_file_text(stream: &PreparedStream) -> String {
    stream
        .calls
        .iter()
        .enumerate()
        .map(|(call_index, call)| {
            let (model_answer, implementation_answer) = match &call.model {
                ModelVerdictOfCall::Disagrees {
                    model_answer,
                    implementation_answer,
                    ..
                } => (cell_text(model_answer), cell_text(implementation_answer)),
                ModelVerdictOfCall::NotModelled
                | ModelVerdictOfCall::Agrees
                | ModelVerdictOfCall::SkippedAfterTheModelStopped
                | ModelVerdictOfCall::NotComparedBecauseTheImplementationCrashed => {
                    ("-".to_string(), "-".to_string())
                }
                ModelVerdictOfCall::ModelCannotTakeTheObservation { what } => {
                    ("-".to_string(), cell_text(what))
                }
            };
            let panic_message = match &call.outcome {
                CallOutcome::Panicked(text) => cell_text(text),
                CallOutcome::Succeeded
                | CallOutcome::ReturnedError(_)
                | CallOutcome::InFlightAtTheTruncation => "-".to_string(),
            };
            format!(
                "{call_index}\t{}\t{}\t{}\t{}\t{}\t{model_answer}\t{implementation_answer}\t{panic_message}\t{}\t{}\n",
                call.label,
                call.first_operation,
                call.return_point,
                call.outcome.outcome_word(),
                call.model.word(),
                call.return_value
                    .as_ref()
                    .map_or_else(|| "-".to_string(), |value| value.whole.clone()),
                memory_objects_digest(&call.memory_objects),
            )
        })
        .collect()
}

fn memory_objects_text(memory_objects: &[(&'static str, String)]) -> String {
    if memory_objects.is_empty() {
        return "-".to_string();
    }
    memory_objects
        .iter()
        .map(|(name, digest)| format!("{name}:{digest}"))
        .collect::<Vec<String>>()
        .join(",")
}

pub(super) fn memory_objects_digest(memory_objects: &[(&'static str, String)]) -> String {
    if memory_objects.is_empty() {
        return "-".to_string();
    }
    sha256_hexadecimal(memory_objects_text(memory_objects).as_bytes())
}

/// `call-return-values.tsv`（读法 4，登记 5.3 第 2 处）：每次调用一行——流名、调用序号、标签、交回值的 sha256、交回值顶层字段逐个
/// `名字:sha256`（逗号接起来）、内存对象逐个 `名字:sha256`、胶水交给模型的观测的 sha256。没有交回值（出错、panic、在途）的两格写 `-`，
/// 没交观测的写 `-`。
fn call_return_values_file_text(stream: &PreparedStream) -> String {
    stream
        .calls
        .iter()
        .enumerate()
        .map(|(call_index, call)| {
            let (whole, fields) = match &call.return_value {
                None => ("-".to_string(), "-".to_string()),
                Some(value) => (
                    value.whole.clone(),
                    value
                        .fields
                        .iter()
                        .map(|(name, digest)| format!("{name}:{digest}"))
                        .collect::<Vec<String>>()
                        .join(","),
                ),
            };
            format!(
                "{}\t{call_index}\t{}\t{whole}\t{fields}\t{}\t{}\n",
                stream.name.directory_name(),
                call.label,
                memory_objects_text(&call.memory_objects),
                call.observation_sha256.as_deref().unwrap_or("-")
            )
        })
        .collect()
}

/// 驱动怎么结束的，写成一个词（`stream-status.tsv` 与结果行共用）。
pub(super) fn ending_text(ending: &StreamEnding) -> String {
    match ending {
        StreamEnding::Completed => "completed".to_string(),
        StreamEnding::StoppedByAnError { call_index, member } => {
            format!("stopped_by_an_error:call_{call_index}:{member}")
        }
        StreamEnding::StoppedByAPanic {
            call_index,
            message,
        } => format!(
            "stopped_by_a_panic:call_{call_index}:{}",
            field_text(&cell_text(message))
        ),
        StreamEnding::StoppedWithoutAnError { what } => {
            format!("stopped_without_an_error:{}", field_text(what))
        }
    }
}

/// `stream-status.tsv`：驱动怎么结束的、模型停在哪次调用、被判的根写在不在（`key<TAB>value`）。
fn stream_status_file_text(stream: &PreparedStream) -> String {
    format!(
        "ending\t{}\nmodel_stopped_at_call\t{}\njudged_root_index\t{}\nwrites\t{}\ncalls\t{}\n",
        ending_text(&stream.ending),
        stream
            .model_stopped_at_call
            .map_or_else(|| "none".to_string(), |call| call.to_string()),
        if stream.judged_root_index == usize::MAX {
            "none".to_string()
        } else {
            stream.judged_root_index.to_string()
        },
        stream.writes.len(),
        stream.calls.len()
    )
}

/// `oracle-verdicts.tsv`：每个取样状态一行（次序与 `states.tsv` 相同）——状态号、批号、崩溃状态序号、段号、触达的最后一次调用、
/// O1-模型崩溃判官（`green` / `red` / `na` / `off` / `recovery_panic`）与红的内容、O2（同）与判红的不变量、O2 直接调与
/// `judge_crash_image` 内那一遍是否一致、看 journal 那一遍恢复的摘要（根、结局、读回 sha256、施加记录数）、不看 journal 那一遍的摘要、
/// 崩溃点重放的 `red_items`（附带），加第 1 次重跑的五列：O1-RefFS 的票（`green` / `red` / `abstained` / `recovery_panic` / `off`）、
/// 弃权种类或红的理由、RefFS 的崩溃时刻、允许集合的下标、看 journal 那一遍恢复 panic 的消息（没有写 `-`），加修订 5 第 5.1 节第 5 条的
/// `piece_panics`：回调链 panic 了的态上逐件重做时 panic 的件名，按 [`JudgingPiece`] 的次序逗号连起来（没有写 `-`）。
fn oracle_verdicts_file_text(observations: &[StateObservation]) -> String {
    observations
        .iter()
        .enumerate()
        .map(|(state_number, observation)| {
            let (pool_checker_word, pool_checker_items) = match &observation.pool_checker_red {
                None => ("off", "-".to_string()),
                Some(items) if items.is_empty() => ("green", "-".to_string()),
                Some(items) => ("red", items.join(",")),
            };
            let crash_judge_detail = match &observation.crash_judge {
                CrashJudgeVerdict::Red { what } => cell_text(what),
                CrashJudgeVerdict::Green
                | CrashJudgeVerdict::NotApplicable
                | CrashJudgeVerdict::Off
                | CrashJudgeVerdict::RecoveryPanicked
                | CrashJudgeVerdict::JudgePanicked => "-".to_string(),
            };
            let (reference_filesystem_detail, allowed) = match &observation.reference_filesystem {
                ReferenceFileSystemVoteOfAState::Green { allowed } => ("-".to_string(), allowed.clone()),
                ReferenceFileSystemVoteOfAState::Red { reason, allowed } => {
                    (field_text(&cell_text(reason)), allowed.clone())
                }
                ReferenceFileSystemVoteOfAState::Abstained { kind } => {
                    (field_text(kind), "-".to_string())
                }
                ReferenceFileSystemVoteOfAState::RecoveryPanicked
                | ReferenceFileSystemVoteOfAState::JudgePanicked
                | ReferenceFileSystemVoteOfAState::Off => ("-".to_string(), "-".to_string()),
            };
            format!(
                "{state_number}\t{}\t{}\t{}\t{}\t{}\t{crash_judge_detail}\t{pool_checker_word}\t{pool_checker_items}\t{}\t{}\t{}\t{}\t{}\t{reference_filesystem_detail}\t{}\t{allowed}\t{}\t{}\n",
                observation.batch,
                observation.ordinal,
                observation.segment_index,
                observation
                    .touched_call
                    .map_or_else(|| "-".to_string(), |call| call.to_string()),
                observation.crash_judge.word(),
                observation.pool_checker_matches_the_judgement,
                observation.consulted.text(),
                observation.ignored.text(),
                field_text(&list_text(&observation.verdict.red_items)),
                observation.reference_filesystem.word(),
                observation.reference_filesystem_crash_moment,
                observation
                    .recovery_panic
                    .as_deref()
                    .map_or_else(|| "-".to_string(), |message| field_text(&cell_text(message))),
                piece_panics_text(&observation.piece_panics),
            )
        })
        .collect()
}

/// 一态的 `piece_panics` 列：件名按 [`JudgingPiece`] 的次序逗号连起来，没有写 `-`。
fn piece_panics_text(piece_panics: &[JudgingPiece]) -> String {
    if piece_panics.is_empty() {
        "-".to_string()
    } else {
        piece_panics
            .iter()
            .map(|piece| piece.name())
            .collect::<Vec<&str>>()
            .join(",")
    }
}

/// `writable-mount-probe.tsv`（读法 8，登记 5.3 第 6 样）：每个探过的态一行——流名、批号、崩溃状态序号、状态号、(i) 判不出那一支、
/// (ii) 计数那一支（为真写是哪一句，假写 `false`）、c_见证、所选版本、挂载结局、拒的成员、单子上的理由、第二截判定。
fn writable_mount_probe_file_text(stream: StreamName, observations: &[StateObservation]) -> String {
    observations
        .iter()
        .enumerate()
        .filter_map(|(state_number, observation)| {
            observation.probe.as_ref().map(|probe| {
                format!(
                    "{}\t{}\t{}\t{state_number}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    stream.directory_name(),
                    observation.batch,
                    observation.ordinal,
                    probe
                        .criterion
                        .some_device_has_fewer_than_two_verified_slots,
                    probe
                        .criterion
                        .counter_sentence
                        .map_or_else(|| "false".to_string(), |sentence| format!("{sentence:?}")),
                    probe.criterion.witnessed_counter,
                    probe.selected_version,
                    probe.mount,
                    probe.refusal_member,
                    probe.refusal_on_the_legal_refusal_list,
                    probe.second_crop,
                )
            })
        })
        .collect()
}

const ORACLE_VERDICTS_FILE: &str = "oracle-verdicts.tsv";
const CALL_OUTCOMES_FILE: &str = "call-outcomes.tsv";
const CALL_RETURN_VALUES_FILE: &str = "call-return-values.tsv";
const STREAM_STATUS_FILE: &str = "stream-status.tsv";
const WRITABLE_MOUNT_PROBE_FILE: &str = "writable-mount-probe.tsv";

/// 按 E170 登记 5.3 写一条流的十个文件，manifest 最后写（它记前九个的 sha256）；之后再写本实验自己的几个：
/// `oracle-verdicts.tsv`、`writable-mount-probe.tsv`（开了探针才有）、`call-outcomes.tsv`、`call-return-values.tsv`、`stream-status.tsv`
/// （不进 manifest：manifest 与 E170 导出装置写的同一份逐字节可比，停机 S2）。
pub(super) fn write_stream_files(
    directory: &Path,
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    observations: &[StateObservation],
    full_states: u64,
    in_place_subsets_states: u64,
) -> Vec<WrittenFile> {
    let base_sectors = base_sectors_file_bytes(stream);
    let (operations_text, payload) = operations_file_text_and_payload(stream);
    let calls_text = calls_file_text(stream);
    let mut torn_text = String::new();
    let mut torn_payload = Vec::new();
    let mut torn_rows: Vec<(usize, usize)> = tables
        .torn_images
        .iter()
        .map(|row| (row.write_index, row.torn_image_index))
        .collect();
    torn_rows.sort_unstable();
    for (write_index, torn_image_index) in torn_rows {
        let torn_bytes = tables.enumerated_writes[torn_image_index]
            .bytes()
            .expect("撕裂镜像是带字节的写");
        torn_text.push_str(&format!(
            "{write_index}\t{}\t{}\t{}\n",
            torn_payload.len(),
            torn_bytes.len(),
            sha256_hexadecimal(torn_bytes)
        ));
        torn_payload.extend_from_slice(torn_bytes);
    }
    let mut states_text = String::new();
    let mut verdicts_text = String::new();
    for (state_number, observation) in observations.iter().enumerate() {
        states_text.push_str(&format!(
            "{state_number}\t{}\t{}\t{}\t{}\t{}\n",
            observation.ordinal,
            observation.segment_index,
            observation.batch,
            observation.landing_vector,
            observation.image_digest
        ));
        let verdict = &observation.verdict;
        verdicts_text.push_str(&format!(
            "{state_number}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            verdict.consulted_root,
            verdict.consulted_outcome,
            verdict.ignored_root,
            verdict.ignored_outcome,
            list_text(&verdict.red_items),
            list_text(&verdict.pool_checker_red),
            verdict.root_without_record,
            verdict.claimed_state_missing_unit,
            observation.judge_nanoseconds
        ));
    }
    let in_place_subsets_text =
        format!("in_place_subsets_states\t{in_place_subsets_states}\nfull_states\t{full_states}\n");
    let mut files = vec![
        write_export_file(directory, BASE_SECTORS_FILE, &base_sectors),
        write_export_file(directory, OPERATIONS_FILE, operations_text.as_bytes()),
        write_export_file(directory, OPERATIONS_PAYLOAD_FILE, &payload),
        write_export_file(directory, CALLS_FILE, calls_text.as_bytes()),
        write_export_file(directory, TORN_FILE, torn_text.as_bytes()),
        write_export_file(directory, TORN_PAYLOAD_FILE, &torn_payload),
        write_export_file(directory, STATES_FILE, states_text.as_bytes()),
        write_export_file(
            directory,
            IMPLEMENTATION_VERDICTS_FILE,
            verdicts_text.as_bytes(),
        ),
        write_export_file(
            directory,
            IN_PLACE_SUBSETS_FILE,
            in_place_subsets_text.as_bytes(),
        ),
    ];
    let device_list: Vec<String> = stream
        .base
        .devices
        .keys()
        .map(|device| device.0.to_string())
        .collect();
    let mut manifest_text = format!(
        "stream\t{}\ndevices\t{}\ndevice_bytes\t{}\nsector_bytes\t{LOCAL_SECTOR_BYTES}\nops\t{}\nwrites\t{}\ncalls\t{}\nsampled_states\t{}\n",
        stream.name.directory_name(),
        device_list.join(","),
        stream.base.device_size_in_bytes,
        stream.operations.len(),
        stream.writes.len(),
        stream.calls.len(),
        observations.len(),
    );
    for (listed_name, file) in FILES_LISTED_IN_THE_MANIFEST.iter().zip(&files) {
        assert_eq!(*listed_name, file.name, "manifest 的次序与写出的次序相同");
        manifest_text.push_str(&format!("sha256:{}\t{}\n", file.name, file.sha256));
    }
    files.push(write_export_file(
        directory,
        MANIFEST_FILE,
        manifest_text.as_bytes(),
    ));
    files.push(write_export_file(
        directory,
        ORACLE_VERDICTS_FILE,
        oracle_verdicts_file_text(observations).as_bytes(),
    ));
    if observations
        .iter()
        .any(|observation| observation.probe.is_some())
    {
        files.push(write_export_file(
            directory,
            WRITABLE_MOUNT_PROBE_FILE,
            writable_mount_probe_file_text(stream.name, observations).as_bytes(),
        ));
    }
    files
        .extend(write_the_files_that_exist_even_when_the_stream_is_not_exported(directory, stream));
    files
}

/// 没有根槽写可判、或流水线里别处 panic 的流不导出状态；这三个文件照写（比对器靠它们认出「出错截断」「panic 截断」与交回值）。
pub(super) fn write_the_files_that_exist_even_when_the_stream_is_not_exported(
    directory: &Path,
    stream: &PreparedStream,
) -> Vec<WrittenFile> {
    vec![
        write_export_file(
            directory,
            CALL_OUTCOMES_FILE,
            call_outcomes_file_text(stream).as_bytes(),
        ),
        write_export_file(
            directory,
            CALL_RETURN_VALUES_FILE,
            call_return_values_file_text(stream).as_bytes(),
        ),
        write_export_file(
            directory,
            STREAM_STATUS_FILE,
            stream_status_file_text(stream).as_bytes(),
        ),
    ]
}

#[cfg(test)]
mod tests;
