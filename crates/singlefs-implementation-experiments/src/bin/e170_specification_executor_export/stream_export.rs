//! 一条流导出：取样的状态过一遍实现的判定，写出中性格式的各个文件。

use super::*;

// ───────────────────────────── 一条流导出 ─────────────────────────────

/// 写出的一个文件：名字、字节数、sha256；带耗时的两个文件另算一份去掉耗时之后的 sha256（登记第十二节修订 V2b）。
pub(super) struct WrittenFile {
    pub(super) name: &'static str,
    pub(super) bytes: usize,
    pub(super) sha256: String,
    pub(super) sha256_without_timing: Option<String>,
}

/// `impl-verdicts.tsv` 每行去掉最后一列（耗时）；`manifest.tsv` 去掉记 `impl-verdicts.tsv` 摘要的那一行。其余文件原样。
pub(super) fn contents_without_timing(file_name: &str, contents: &[u8]) -> Option<Vec<u8>> {
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

pub(super) fn write_export_file(
    directory: &Path,
    name: &'static str,
    contents: &[u8],
) -> WrittenFile {
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

pub(super) struct StreamExport {
    pub(super) name: StreamName,
    pub(super) checks: Vec<StopCheck>,
    pub(super) calls: Vec<RecordedCall>,
    pub(super) operation_count: usize,
    pub(super) write_count: usize,
    pub(super) segment_count: usize,
    pub(super) tearable_write_count: usize,
    pub(super) full_states: u64,
    pub(super) in_place_subsets_states: u64,
    pub(super) torn_scans: Vec<TornScan>,
    pub(super) observations: Vec<StateObservation>,
    pub(super) files: Vec<WrittenFile>,
    pub(super) rebuilt_digest_mismatches: usize,
    pub(super) rebuilt_states: usize,
    pub(super) elapsed_nanoseconds: u128,
}

pub(super) fn export_stream(name: StreamName, output_root: &Path) -> StreamExport {
    let started = Instant::now();
    let stream = prepare_stream(name);
    let mut checks = stream.checks.clone();
    let tables = crash_state_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let full_states = crash_point_replay_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let in_place_subsets_states = crash_point_replay_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &in_place_subsets_with_copy_on_write_none_or_all_expansion,
    );
    let facts = extract_flow_facts(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let label = name.directory_name();
    let (local_full, local_in_place_subsets, local_closed_form) = match name {
        StreamName::NewPoolFileCreation => (
            LOCAL_FIRST_STREAM_FULL_STATES,
            LOCAL_FIRST_STREAM_IN_PLACE_SUBSETS_STATES,
            LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM,
        ),
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => (
            LOCAL_SECOND_STREAM_FULL_STATES,
            LOCAL_SECOND_STREAM_IN_PLACE_SUBSETS_STATES,
            LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM,
        ),
    };
    checks.push(StopCheck::new(
        &format!("{label}_full_states"),
        local_full,
        full_states,
    ));
    checks.push(StopCheck::new(
        &format!("{label}_in_place_subsets_states"),
        local_in_place_subsets,
        in_place_subsets_states,
    ));
    checks.push(StopCheck::new(
        &format!("{label}_two_state_closed_form"),
        local_closed_form,
        singlefs_verification::memory_pool::closed_form_state_count_of_write_index_segments(
            &stream.segments,
        ),
    ));
    checks.push(StopCheck::new(
        &format!("{label}_enumeration_tables_state_count"),
        full_states,
        tables.state_count,
    ));
    checks.push(StopCheck::new(
        &format!("{label}_flow_facts_state_count"),
        full_states,
        facts.state_count,
    ));
    let torn_image_index_by_write: BTreeMap<usize, usize> = tables
        .torn_images
        .iter()
        .map(|row| (row.write_index, row.torn_image_index))
        .collect();
    let segment_has_a_torn_write: Vec<bool> = stream
        .segments
        .iter()
        .map(|segment| {
            segment
                .iter()
                .any(|write_index| tables.is_tearable_by_write[*write_index])
        })
        .collect();
    let recorded_write_count = stream.writes.len();
    let sampled = sample_ordinals(
        &tables.state_ranges_by_segment,
        tables.state_count,
        &segment_has_a_torn_write,
        &|ordinal| {
            landing_vector_of(
                &facts.persisted_writes_of_state(ordinal),
                recorded_write_count,
                &torn_image_index_by_write,
            )
        },
    );
    let observations = observe_sampled_states(
        &stream,
        &tables,
        &facts,
        &torn_image_index_by_write,
        &sampled,
    );
    let directory = output_root.join(label);
    std::fs::create_dir_all(&directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
    let files = write_stream_files(
        &directory,
        &stream,
        &tables,
        &observations,
        full_states,
        in_place_subsets_states,
    );
    // S4：只读刚写出的文件，按 states.tsv 的落地向量重建，摘要与 states.tsv 里（直接从 CrashImage 算的）那一列比。
    let neutral = read_neutral_stream(&directory);
    let mut rebuilt_states = 0usize;
    let mut rebuilt_digest_mismatches = 0usize;
    for line in read_text_lines(&directory, STATES_FILE) {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 6, "states.tsv 一行六列：{line:?}");
        rebuilt_states += 1;
        if sha256_text(&rebuilt_image_digest(&neutral, fields[4])) != fields[5] {
            rebuilt_digest_mismatches += 1;
        }
    }
    StreamExport {
        name,
        checks,
        calls: stream.calls.clone(),
        operation_count: stream.operations.len(),
        write_count: stream.writes.len(),
        segment_count: stream.segments.len(),
        tearable_write_count: torn_image_index_by_write.len(),
        full_states,
        in_place_subsets_states,
        torn_scans: sampled.torn_scans,
        observations,
        files,
        rebuilt_digest_mismatches,
        rebuilt_states,
        elapsed_nanoseconds: started.elapsed().as_nanos(),
    }
}

/// `base-sectors.bin` 的内容（登记 5.3）：基镜像里每个不全为零的扇区，盘身份 u32、扇区号 u64、512 字节。
/// 第一段写导出、第三段核导出与重驱动的流相同（`bad-states` 模式）共用这一份。
pub(super) fn base_sectors_file_bytes(stream: &PreparedStream) -> Vec<u8> {
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
pub(super) fn operations_file_text_and_payload(stream: &PreparedStream) -> (String, Vec<u8>) {
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

/// `calls.tsv` 的文本（登记 5.3），同上两种模式共用。
pub(super) fn calls_file_text(stream: &PreparedStream) -> String {
    stream
        .calls
        .iter()
        .enumerate()
        .map(|(call_index, call)| {
            format!(
                "{call_index}\t{}\t{}\t{}\tok\n",
                call.label, call.first_operation, call.return_point
            )
        })
        .collect()
}

/// 按登记 5.3 写一条流的十个文件，manifest 最后写（它记前九个的 sha256）。
fn write_stream_files(
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
    files
}

#[cfg(test)]
mod tests;
