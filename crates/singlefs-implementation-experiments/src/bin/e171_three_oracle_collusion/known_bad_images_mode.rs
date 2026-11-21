//! 模式 known-bad-images：已知坏镜像与 E170（O3规约执行器第一版） 坏状态的录入与判。

use super::*;

// ───────────────────────────── 坏镜像：镜像物化、补丁与摘要（登记 5.5） ─────────────────────────────

/// 一处字节补丁：在底态镜像上把 (盘, 偏移) 起的这几个字节换掉（与 E170 的 `bad-patches.bin` 同一种补丁）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BytePatch {
    pub(super) device: u32,
    pub(super) offset: u64,
    pub(super) bytes: Vec<u8>,
}

/// 把一个崩溃镜像物化成 `MemoryPool`：基镜像拷一份，按写表次序施加持久了的写。
fn materialized_memory_pool(image: &CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    let persisted_writes: Vec<RetainedWrite> = image
        .writes
        .iter()
        .zip(&image.persisted)
        .filter(|(_, is_persisted)| **is_persisted)
        .map(|(write, _)| write.clone())
        .collect();
    pool.apply_writes(&persisted_writes);
    pool
}

/// 阳性对照 O2：把已知坏镜像族的第一条改法施加在这一态的物化镜像上，再交 `check_pool_image`（只为计时，名单走两段）。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
pub(super) fn pool_checker_on_the_first_known_bad_image(
    image: &CrashImage<'_>,
) -> Vec<(&'static str, InvariantVerdict)> {
    check_pool_image(&first_known_bad_image_of(image))
}

/// 这一态的物化镜像上施加已知坏镜像族的第一条改法之后的那张盘（阳性对照 O2 判的那一张）。
fn first_known_bad_image_of(image: &CrashImage<'_>) -> MemoryPool {
    let clean = materialized_memory_pool(image);
    let entries = known_bad_images(&clean);
    let (_, apply_the_first_entry) = entries.first().expect("已知坏镜像表至少一条");
    let mut bad = clean.clone();
    apply_the_first_entry(&mut bad);
    bad
}

/// 录入段录下取样的一态上 O2 要判的盘（O2 的名单按判到位与违例位能两段，计时与按件 panic 留 CPU）：
/// 这一态的物化镜像（阳性对照那一态是改坏的那一张，[`first_known_bad_image_of`]）作一张给定状态表交 `tables` 录下。
/// 表名带流、序号与镜像摘要（同名的表内容就相同，判过的复用）。
pub(super) fn pool_checker_table_of_a_sampled_state(
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    stream_name: StreamName,
    ordinal: u64,
    image_digest: &str,
    image: &CrashImage<'_>,
    pool_checker_control: bool,
) -> RecordedTableReference {
    if pool_checker_control {
        tables.record_a_table(
            &format!(
                "sampled-state/{}/ordinal={ordinal}/image={image_digest}/first-known-bad-entry",
                stream_name.directory_name()
            ),
            &first_known_bad_image_of(image),
        )
    } else {
        tables.record_a_table(
            &format!(
                "sampled-state/{}/ordinal={ordinal}/image={image_digest}",
                stream_name.directory_name()
            ),
            &materialized_memory_pool(image),
        )
    }
}

/// 一个 `MemoryPool` 的镜像摘要（E170 登记 5.3 的算法）：每块盘（身份升序）、每个不全为零的扇区（扇区号升序）依次喂进盘身份 u32、扇区号 u64、512 字节。
pub(super) fn memory_pool_digest(pool: &MemoryPool) -> [u8; DIGEST_BYTES] {
    let mut message = Vec::new();
    for (device, sparse) in &pool.devices {
        for (sector, bytes) in sparse.written_sectors() {
            append_sector_to_digest_message(&mut message, device.0, *sector, bytes);
        }
    }
    sha256_digest(&message)
}

/// 从中性格式按落地向量重建一态的 `MemoryPool`（按流序叠，规则与 [`rebuilt_image_digest`] 相同；两处各算各的摘要，S4 互比）。
pub(super) fn rebuilt_memory_pool(stream: &NeutralStream, landing_vector: &str) -> MemoryPool {
    let mut pool =
        MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], LOCAL_DEVICE_BYTES);
    for ((device, sector), bytes) in &stream.base_sectors {
        pool.devices
            .get_mut(&DeviceIdentity(*device))
            .expect("基镜像的盘在池里")
            .write(DeviceOffsetInBytes(sector * LOCAL_SECTOR_BYTES), bytes);
    }
    let mut landings = landing_vector.chars();
    let mut write_index = 0usize;
    for operation in &stream.operations {
        if operation.kind == RecordedOperationKind::Barrier {
            continue;
        }
        let landing = landings.next().expect("落地向量与写数一样长");
        let length = usize::try_from(operation.length).expect("写长装得进 usize");
        let device = pool
            .devices
            .get_mut(&DeviceIdentity(operation.device))
            .expect("写的盘在池里");
        match (landing, operation.kind) {
            ('0', _) => {}
            ('2', RecordedOperationKind::WriteZeroes) => {
                device.zero_fill(DeviceOffsetInBytes(operation.offset), operation.length);
            }
            ('2', _) => {
                let start = operation.payload_start.expect("带字节的写有载荷起点");
                device.write(
                    DeviceOffsetInBytes(operation.offset),
                    &stream.payload[start..start + length],
                );
            }
            ('1', _) => {
                let (start, torn_length) = stream.torn_bytes_by_write[&write_index];
                assert_eq!(torn_length, length, "撕裂那一段罩整次写");
                device.write(
                    DeviceOffsetInBytes(operation.offset),
                    &stream.torn_payload[start..start + torn_length],
                );
            }
            (unknown, _) => panic!("落地向量里认不出的字符 {unknown:?}"),
        }
        write_index += 1;
    }
    assert!(landings.next().is_none(), "落地向量比写数长");
    pool
}

/// 把一串补丁施加到 `MemoryPool` 上：补丁的起点与长度不一定按扇区对齐（E170 的 P 行只翻一个字节），
/// 所以先读出罩住它的整扇区、把补丁字节叠上去、再整扇区写回。
pub(super) fn apply_patches(pool: &mut MemoryPool, patches: &[BytePatch]) {
    let sector_bytes = LOCAL_SECTOR_BYTES;
    for patch in patches {
        let device = pool
            .devices
            .get_mut(&DeviceIdentity(patch.device))
            .expect("补丁的盘在池里");
        let patch_length = u64::try_from(patch.bytes.len()).expect("补丁长装得进 u64");
        let first_sector_start = patch.offset / sector_bytes * sector_bytes;
        let end = (patch.offset + patch_length).div_ceil(sector_bytes) * sector_bytes;
        let mut covered = device.read(
            DeviceOffsetInBytes(first_sector_start),
            usize::try_from(end - first_sector_start).expect("罩住补丁的扇区长装得进 usize"),
        );
        let start_in_covered =
            usize::try_from(patch.offset - first_sector_start).expect("补丁起点在罩住的扇区里");
        covered[start_in_covered..start_in_covered + patch.bytes.len()]
            .copy_from_slice(&patch.bytes);
        device.write(DeviceOffsetInBytes(first_sector_start), &covered);
    }
}

/// 两份镜像逐 512 字节扇区的差（缺的扇区按全零），相邻的差异扇区并成一段补丁；补丁带「改坏之后」的字节。
pub(super) fn sector_difference_as_patches(clean: &MemoryPool, bad: &MemoryPool) -> Vec<BytePatch> {
    let sector_bytes = usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
    let zero_sector = vec![0u8; sector_bytes];
    let mut patches: Vec<BytePatch> = Vec::new();
    for (device, bad_sparse) in &bad.devices {
        let clean_sparse = clean.devices.get(device).expect("两份镜像的盘相同");
        let sectors: BTreeSet<u64> = bad_sparse
            .written_sectors()
            .keys()
            .chain(clean_sparse.written_sectors().keys())
            .copied()
            .collect();
        let mut open: Option<(u64, BytePatch)> = None;
        for sector in sectors {
            let bad_bytes = bad_sparse
                .written_sectors()
                .get(&sector)
                .unwrap_or(&zero_sector);
            let clean_bytes = clean_sparse
                .written_sectors()
                .get(&sector)
                .unwrap_or(&zero_sector);
            if bad_bytes == clean_bytes {
                continue;
            }
            match &mut open {
                Some((last_sector, patch)) if *last_sector + 1 == sector => {
                    patch.bytes.extend_from_slice(bad_bytes);
                    *last_sector = sector;
                }
                Some(_) | None => {
                    if let Some((_, finished)) = open.take() {
                        patches.push(finished);
                    }
                    open = Some((
                        sector,
                        BytePatch {
                            device: device.0,
                            offset: sector * LOCAL_SECTOR_BYTES,
                            bytes: bad_bytes.clone(),
                        },
                    ));
                }
            }
        }
        if let Some((_, finished)) = open.take() {
            patches.push(finished);
        }
    }
    patches
}

// ───────────────────────────── `known-bad-images` 模式：已知坏镜像与 E170 的 P 行、盲测族上的 O2（登记 5.5） ─────────────────────────────

/// 补丁文本：`设备:偏移:在补丁文件里的起点:长度`，逗号隔开（E170 的 `bad-states.tsv` 同一种写法）；没有补丁写 `-`。
pub(super) fn patch_list_text(patches: &[BytePatch], patches_file: &mut Vec<u8>) -> String {
    if patches.is_empty() {
        return "-".to_string();
    }
    patches
        .iter()
        .map(|patch| {
            let start = patches_file.len();
            patches_file.extend_from_slice(&patch.bytes);
            format!(
                "{}:{}:{start}:{}",
                patch.device,
                patch.offset,
                patch.bytes.len()
            )
        })
        .collect::<Vec<String>>()
        .join(",")
}

/// 读 E170 的补丁文本回成补丁。
fn patches_of_a_patch_list_text(list: &str, patches_file: &[u8]) -> Vec<BytePatch> {
    if list == "-" {
        return Vec::new();
    }
    list.split(',')
        .map(|item| {
            let fields: Vec<&str> = item.split(':').collect();
            assert_eq!(fields.len(), 4, "补丁四段：{item:?}");
            let start: usize = parse_number(fields[2], "补丁起点");
            let length: usize = parse_number(fields[3], "补丁长度");
            BytePatch {
                device: parse_number(fields[0], "补丁的盘"),
                offset: parse_number(fields[1], "补丁偏移"),
                bytes: patches_file[start..start + length].to_vec(),
            }
        })
        .collect()
}

/// 录入段录下的已知坏镜像表一条：判段写 `known-bad-o2.tsv` 那一行与结果行要的（目标不变量、补丁几处、补丁叠回之后的摘要与直接改坏的相同不相同、
/// 两张要判的盘）。补丁与摘要进 `known-bad-states.tsv`、`known-bad-patches.bin`，录入段就拼好。
pub(super) struct RecordedKnownBadEntry {
    pub(super) target: String,
    pub(super) patch_count: usize,
    pub(super) digest_equals_the_direct_one: bool,
    pub(super) direct: RecordedTableReference,
    pub(super) patched: RecordedTableReference,
}

/// 录入段录完已知坏镜像表一条交回主线程的：交给判段的那一份，与按条目的次序拼 `known-bad-states.tsv`、`known-bad-patches.bin` 要的补丁与摘要。
struct KnownBadEntryOfTheRecordStage {
    pub(super) hand_off: RecordedKnownBadEntry,
    pub(super) patches: Vec<BytePatch>,
    patched_digest: String,
}

/// 录入段录已知坏镜像表的一条：施加在干净镜像的拷贝上，直接改坏的那张录下，再取补丁（逐扇区的差）叠回干净镜像、叠完的那张录下。
/// 表的改法（`Box<dyn Fn>`）不跨线程，这条线程照干净镜像现取一份表；干净镜像定了，表的条数、次序与改法就定了。
fn record_a_known_bad_entry(
    clean: &MemoryPool,
    entry_index: usize,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> KnownBadEntryOfTheRecordStage {
    let entries = known_bad_images(clean);
    let (target, apply) = &entries[entry_index];
    let mut bad = clean.clone();
    apply(&mut bad);
    let direct = tables.record_a_table(&format!("known-bad/entry-{entry_index}/direct"), &bad);
    let patches = sector_difference_as_patches(clean, &bad);
    let mut patched = clean.clone();
    apply_patches(&mut patched, &patches);
    let patched_table =
        tables.record_a_table(&format!("known-bad/entry-{entry_index}/patched"), &patched);
    let patched_digest = sha256_text(&memory_pool_digest(&patched));
    let digest_equals_the_direct_one = patched_digest == sha256_text(&memory_pool_digest(&bad));
    KnownBadEntryOfTheRecordStage {
        hand_off: RecordedKnownBadEntry {
            target: (*target).to_string(),
            patch_count: patches.len(),
            digest_equals_the_direct_one,
            direct,
            patched: patched_table,
        },
        patches,
        patched_digest,
    }
}

/// 录入段录下的 E170 坏状态一行：判段写 `e170-bad-rows-o2.tsv` 那一行与结果行要的（流、行号、底态摘要、两个摘要与 E170 写的相同不相同、
/// E170 池级 checker 那一列、底态与叠完补丁的两张要判的盘）。
pub(super) struct RecordedE170BadRow {
    pub(super) label: String,
    pub(super) row_identifier: String,
    pub(super) base_digest: String,
    pub(super) digests_equal: bool,
    pub(super) e170_pool_checker_column: String,
    pub(super) base: RecordedTableReference,
    pub(super) patched: RecordedTableReference,
}

/// 录入段录 E170 坏状态的一行：从中性格式重建底态、录下底态，叠补丁、录下叠完的那张，两个摘要与 E170 写的两个摘要比。
fn record_an_e170_bad_row(
    label: &str,
    neutral: &NeutralStream,
    patches_file: &[u8],
    state_row: &str,
    verdict_row: &str,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> RecordedE170BadRow {
    let state_fields: Vec<&str> = state_row.split('\t').collect();
    assert_eq!(
        state_fields.len(),
        5,
        "bad-states.tsv 一行五列：{state_row:?}"
    );
    let verdict_fields: Vec<&str> = verdict_row.split('\t').collect();
    assert_eq!(
        verdict_fields.len(),
        10,
        "bad-impl-verdicts.tsv 一行十列：{verdict_row:?}"
    );
    let row_identifier = state_fields[0];
    let mut pool = rebuilt_memory_pool(neutral, state_fields[1]);
    let base_digest = sha256_text(&memory_pool_digest(&pool));
    // 不打补丁的底态上的 O2（登记 5.6 坏镜像行的阴性：F、Mk、Mk⁺、Mk⁻ 各一张，必须不红）。
    let base = tables.record_a_table(
        &format!("e170-bad-row/{label}/{row_identifier}/base"),
        &pool,
    );
    apply_patches(
        &mut pool,
        &patches_of_a_patch_list_text(state_fields[4], patches_file),
    );
    let patched_digest = sha256_text(&memory_pool_digest(&pool));
    let patched = tables.record_a_table(
        &format!("e170-bad-row/{label}/{row_identifier}/patched"),
        &pool,
    );
    let digests_equal = base_digest == state_fields[2] && patched_digest == state_fields[3];
    RecordedE170BadRow {
        label: label.to_string(),
        row_identifier: row_identifier.to_string(),
        base_digest,
        digests_equal,
        e170_pool_checker_column: verdict_fields[6].to_string(),
        base,
        patched,
    }
}

/// E170 坏状态一行判完的结局：`e170-bad-rows-o2.tsv` 的那一行与结果行，按行的次序在主线程上收。
struct JudgedE170BadRow {
    row_failed: bool,
    pool_checker_file_row: String,
    result_line: String,
}

/// 判段：E170 坏状态一行的底态与叠完补丁的那张判完之后，判红的不变量与 E170 池级 checker 那一列比（S3），拼产物那一行与结果行。
fn judged_e170_bad_row(
    row: &RecordedE170BadRow,
    base_red: &[String],
    red_by_the_pool_checker: &[String],
) -> JudgedE170BadRow {
    let RecordedE170BadRow {
        label,
        row_identifier,
        base_digest,
        digests_equal,
        e170_pool_checker_column,
        ..
    } = row;
    let pool_checker_text_of_the_row = list_text(red_by_the_pool_checker);
    let pool_checker_equals_e170 = pool_checker_text_of_the_row == *e170_pool_checker_column;
    JudgedE170BadRow {
        row_failed: !(*digests_equal && pool_checker_equals_e170),
        pool_checker_file_row: format!(
            "{label}\t{row_identifier}\t{}\t{pool_checker_text_of_the_row}\t{e170_pool_checker_column}\t{pool_checker_equals_e170}\t{digests_equal}\t{}\t{base_digest}\n",
            if red_by_the_pool_checker.is_empty() { "green" } else { "red" },
            list_text(base_red)
        ),
        result_line: format!(
            "name=e170_bad_row stream={label} row={row_identifier} o2={} o2_red={} e170_pool_checker={} o2_equals_e170={pool_checker_equals_e170} digests_equal={digests_equal} base_o2_red={}",
            if red_by_the_pool_checker.is_empty() { "green" } else { "red" },
            field_text(&pool_checker_text_of_the_row),
            field_text(e170_pool_checker_column),
            field_text(&list_text(base_red))
        ),
    }
}

/// `known-bad-images` 模式录入段构造要判的盘的三步，按两段之前打结果行的次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum KnownBadImagesStep {
    /// 从导出目录重建干净镜像、两条 S4 回比、数表的条数、录下干净镜像那张盘。
    CleanImage,
    /// 已知坏镜像表逐条构造、录下。
    KnownBadEntries,
    /// E170 坏状态逐流、逐行构造、录下（只在带 `--e170-bad-rows` 时）。
    E170BadRows,
}

/// 录入段构造要判的盘时 panic 了：在哪一步、panic 的消息。判段照两段之前的次序打到那一步之前，再以同一句 panic。
pub(super) struct PanicOfTheRecordStage {
    pub(super) step: KnownBadImagesStep,
    pub(super) message: String,
}

/// 录下了的干净镜像：已知坏镜像表的条数与干净镜像那张盘。
pub(super) struct RecordedCleanImage {
    pub(super) entry_count: usize,
    pub(super) table: RecordedTableReference,
}

/// 录入段交给判段的 `known-bad-images` 那一趟：已知坏镜像表在流一全持久态上（干净镜像的两条 S4 回比、表的条数与干净镜像那张盘、每一条录下的，
/// 与按条目的次序拼好的 `known-bad-states.tsv`、`known-bad-patches.bin`），与带 `--e170-bad-rows` 时 E170 坏状态两条流按行的次序录下的每一行（不带是空的）。
/// 构造时 panic 了的，各样录到 panic 之前为止（`clean` 没录下是 `None`），`panic_of_the_record_stage` 记着在哪一步、那一句。
pub(super) struct RecordedKnownBadImages {
    pub(super) clean_image_checks: Vec<StopCheck>,
    pub(super) clean: Option<RecordedCleanImage>,
    pub(super) entries: Vec<RecordedKnownBadEntry>,
    pub(super) states_text: String,
    pub(super) patches_file: Vec<u8>,
    pub(super) e170_bad_rows: Vec<RecordedE170BadRow>,
    pub(super) panic_of_the_record_stage: Option<PanicOfTheRecordStage>,
}

/// 录入段在 `step` 这一步 panic 过（`panic_of_the_record_stage`）：判段以同一句 panic（与两段之前构造到那里 panic 时一样，之后的结果行与产物都没有）。
fn panic_again_if_the_record_stage_panicked_at(
    panic_of_the_record_stage: Option<&PanicOfTheRecordStage>,
    step: KnownBadImagesStep,
) {
    if let Some(panic) =
        panic_of_the_record_stage.filter(|recorded_panic| recorded_panic.step == step)
    {
        std::panic::panic_any(panic.message.clone());
    }
}

/// `known-bad-images` 模式的录入段：从导出目录重建流一全持久态（干净镜像），回比两个摘要（S4），已知坏镜像表每一条（一条怎么录见
/// [`record_a_known_bad_entry`]）与带 `--e170-bad-rows` 时 E170 坏状态每一行（[`record_an_e170_bad_row`]）要判的盘录进 `tables`，不判。
/// 条目与行分给 [`sampling_threads_of_this_run`] 条线程录（每条各录各的表，表名、内容与一条线程时相同，判定库的键不变）；补丁文件的起点与
/// 录下的次序都按条目、按行的次序在主线程上收，与线程数无关。构造时 panic 了（次序上第一个 panic 的条目或行）：接住，录到它之前为止，
/// 记下在哪一步、那一句（[`PanicOfTheRecordStage`]），交给判段照两段之前的次序打到那里再 panic。
pub(super) fn record_the_known_bad_images(
    command_line: &KnownBadImagesCommandLine,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> RecordedKnownBadImages {
    let mut recorded = RecordedKnownBadImages {
        clean_image_checks: Vec::new(),
        clean: None,
        entries: Vec::new(),
        states_text: String::new(),
        patches_file: Vec::new(),
        e170_bad_rows: Vec::new(),
        panic_of_the_record_stage: None,
    };
    let mut step = KnownBadImagesStep::CleanImage;
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        record_the_known_bad_images_step_by_step(command_line, tables, &mut recorded, &mut step);
    }));
    if let Err(payload) = caught {
        recorded.panic_of_the_record_stage = Some(PanicOfTheRecordStage {
            step,
            message: panic_text(&*payload),
        });
    }
    recorded
}

/// [`record_the_known_bad_images`] 的三步，录下的随录随放进 `recorded`，走到哪一步记在 `step`（panic 时那两样留到 panic 之前）。
fn record_the_known_bad_images_step_by_step(
    command_line: &KnownBadImagesCommandLine,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    recorded: &mut RecordedKnownBadImages,
    step: &mut KnownBadImagesStep,
) {
    let threads = sampling_threads_of_this_run(command_line.switches);
    let stream_directory = command_line
        .dump_directory
        .join(StreamName::NewPoolFileCreation.directory_name());
    let neutral = read_neutral_stream(&stream_directory);
    let write_count = neutral
        .operations
        .iter()
        .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
        .count();
    let all_persisted = "2".repeat(write_count);
    let clean = rebuilt_memory_pool(&neutral, &all_persisted);
    let clean_digest = sha256_text(&memory_pool_digest(&clean));
    let reference_digest = sha256_text(&memory_pool_digest(
        &experiment_pool::build_pool("e171-known-bad").memory_pool(),
    ));
    let states_digest_of_the_full_state = read_text_lines(&stream_directory, STATES_FILE)
        .iter()
        .map(|line| {
            line.split('\t')
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
        .find(|fields| fields[4] == all_persisted)
        .map(|fields| fields[5].clone());
    recorded.clean_image_checks = vec![
        StopCheck::new(
            "known_bad_clean_image_digest_equals_build_pool",
            &reference_digest,
            &clean_digest,
        ),
        StopCheck::new(
            "known_bad_clean_image_digest_equals_the_full_state_of_the_dump",
            Some(&clean_digest),
            states_digest_of_the_full_state.as_ref(),
        ),
    ];
    let entry_count = known_bad_images(&clean).len();
    recorded.clean = Some(RecordedCleanImage {
        entry_count,
        table: tables.record_a_table("known-bad/clean", &clean),
    });
    *step = KnownBadImagesStep::KnownBadEntries;
    eprintln!("E171_KNOWN_BAD_IMAGES entries={entry_count} threads={threads}");
    for (entry_index, recorded_entry) in
        outcomes_up_to_the_earliest_panic(entry_count, threads, &|entry_index| {
            record_a_known_bad_entry(&clean, entry_index, tables)
        })
        .into_iter()
        .enumerate()
    {
        let KnownBadEntryOfTheRecordStage {
            hand_off,
            patches,
            patched_digest,
        } = match recorded_entry {
            Ok(entry_of_the_record_stage) => entry_of_the_record_stage,
            Err(payload) => std::panic::resume_unwind(payload),
        };
        recorded.states_text.push_str(&format!(
            "{entry_index}\t{all_persisted}\t{clean_digest}\t{patched_digest}\t{}\n",
            patch_list_text(&patches, &mut recorded.patches_file)
        ));
        recorded.entries.push(hand_off);
    }
    *step = KnownBadImagesStep::E170BadRows;
    if command_line.with_e170_bad_rows {
        record_the_e170_bad_rows_of_both_streams(
            &command_line.dump_directory,
            threads,
            tables,
            &mut recorded.e170_bad_rows,
        );
    }
}

/// E170 的 `bad-states` 模式造的坏状态（P 行、N 行、盲测族）：每一行读 `bad-states.tsv` 的落地向量与补丁、`bad-impl-verdicts.tsv` 里池级 checker
/// 那一列，录下要判的两张盘（一行怎么录见 [`record_an_e170_bad_row`]），按流、按行的次序放进 `rows`，与线程数无关。一条流的行分给 `threads` 条线程录；
/// 次序上第一个 panic 的行原样抛出，它之前的行都放进去了。只读 `bad-states.tsv`、`bad-patches.bin`、`bad-impl-verdicts.tsv`，不读期望文件。
fn record_the_e170_bad_rows_of_both_streams(
    dump_directory: &Path,
    threads: NonZeroUsize,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    rows: &mut Vec<RecordedE170BadRow>,
) {
    for name in STREAM_NAMES {
        let label = name.directory_name();
        let stream_directory = dump_directory.join(label);
        let neutral = read_neutral_stream(&stream_directory);
        let patches_file = read_file(&stream_directory, "bad-patches.bin");
        let state_rows = read_text_lines(&stream_directory, "bad-states.tsv");
        let verdict_rows = read_text_lines(&stream_directory, "bad-impl-verdicts.tsv");
        assert_eq!(
            state_rows.len(),
            verdict_rows.len(),
            "bad-states.tsv 与 bad-impl-verdicts.tsv 同行数"
        );
        eprintln!(
            "E171_E170_BAD_ROWS stream={label} rows={} threads={threads}",
            state_rows.len()
        );
        for recorded in
            outcomes_up_to_the_earliest_panic(state_rows.len(), threads, &|row_position| {
                record_an_e170_bad_row(
                    label,
                    &neutral,
                    &patches_file,
                    &state_rows[row_position],
                    &verdict_rows[row_position],
                    tables,
                )
            })
        {
            match recorded {
                Ok(row) => rows.push(row),
                Err(payload) => std::panic::resume_unwind(payload),
            }
        }
    }
}

/// `known-bad-images` 模式的判段：录下的盘按录下的次序（干净镜像、每一条的直接改坏与叠完补丁、每一行 E170 坏状态的底态与叠完补丁）
/// 分给 [`sampling_threads_of_this_run`] 条线程判，结果行与产物照一趟写完时的次序拼：两边判红的不变量集合要相等（S4）、目标不变量要在里面（S4）；
/// E170 坏状态每一行判红的不变量与 E170 池级 checker 那一列比（S3）。判器在次序上第一个 panic 的那张盘照原样抛出，抛出之前的结果行与产物照写
/// （与一条线程判到那里 panic 时相同）。录入段构造时 panic 过的，打到那一步之前，再以同一句 panic（[`panic_again_if_the_record_stage_panicked_at`]）：
/// 录下的盘都在 panic 之前，判器在其中一张上 panic 的先抛。结果行写进 `result_lines_output`，产物写进命令行给的输出目录。交回退出码。
///
/// # Errors
/// 交接与录下的对不上（[`PoolCheckerTablesOfTheTwoStages::judge_a_recorded_table`] 的 `Err`）：一个文件都不写、一行结果行都不打就交回那一句。
pub(super) fn judge_the_known_bad_images(
    command_line: &KnownBadImagesCommandLine,
    recorded: RecordedKnownBadImages,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    result_lines_output: &mut dyn std::io::Write,
) -> Result<i32, String> {
    let RecordedKnownBadImages {
        clean_image_checks,
        clean,
        entries,
        states_text,
        patches_file,
        e170_bad_rows,
        panic_of_the_record_stage,
    } = recorded;
    let recorded_tables: Vec<&RecordedTableReference> = clean
        .iter()
        .map(|recorded_clean_image| &recorded_clean_image.table)
        .chain(
            entries
                .iter()
                .flat_map(|entry| [&entry.direct, &entry.patched]),
        )
        .chain(
            e170_bad_rows
                .iter()
                .flat_map(|row| [&row.base, &row.patched]),
        )
        .collect();
    let mut judged_in_the_recorded_order = Vec::with_capacity(recorded_tables.len());
    for judged in outcomes_up_to_the_earliest_panic(
        recorded_tables.len(),
        sampling_threads_of_this_run(command_line.switches),
        &|position| tables.judge_a_recorded_table(recorded_tables[position]),
    ) {
        judged_in_the_recorded_order.push(match judged {
            Ok(red_or_refusal) => Ok(red_or_refusal?),
            Err(payload) => Err(payload),
        });
    }
    let mut judged_in_the_recorded_order = judged_in_the_recorded_order.into_iter();
    let mut lines = ResultLines {
        emitted: 0,
        output: result_lines_output,
    };
    lines.emit_result_line(&format!(
        "name=config segment=s1 mode=known-bad-images e170_bad_rows={}",
        command_line.with_e170_bad_rows
    ));
    let mut known_bad_failed = 0usize;
    for check in &clean_image_checks {
        known_bad_failed += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s4 check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let Some(RecordedCleanImage { entry_count, .. }) = clean else {
        let panic = panic_of_the_record_stage
            .expect("录入段没录下干净镜像，只可能是构造干净镜像那一步 panic 了");
        std::panic::panic_any(panic.message);
    };
    lines.emit_result_line(&format!("name=known_bad_entry_count count={entry_count}"));
    let mut red_names_of_the_next_table = || match judged_in_the_recorded_order
        .next()
        .expect("录下的每张盘都判过一次：判的次序就是录下的次序，panic 的那张之后不再往下走")
    {
        Ok(red) => red,
        Err(payload) => std::panic::resume_unwind(payload),
    };
    let clean_red = red_names_of_the_next_table();
    lines.emit_result_line(&format!(
        "name=known_bad_clean_image invariants_red={}",
        field_text(&list_text(&clean_red))
    ));
    let mut pool_checker_text = String::new();
    for (entry_index, entry) in entries.iter().enumerate() {
        let direct_red = red_names_of_the_next_table();
        let patched_red = red_names_of_the_next_table();
        let target = &entry.target;
        let digest_equals_the_direct_one = entry.digest_equals_the_direct_one;
        let sets_equal = direct_red == patched_red;
        let target_is_red = patched_red.iter().any(|name| name == target);
        known_bad_failed +=
            usize::from(!(sets_equal && digest_equals_the_direct_one && target_is_red));
        pool_checker_text.push_str(&format!(
            "{entry_index}\t{target}\t{}\t{}\t{sets_equal}\t{target_is_red}\n",
            list_text(&patched_red),
            list_text(&direct_red)
        ));
        lines.emit_result_line(&format!(
            "name=known_bad_entry index={entry_index} target={target} o2={} patched_red={} direct_red={} sets_equal={sets_equal} digest_equals_the_direct_one={digest_equals_the_direct_one} target_is_red={target_is_red} patches={}",
            if patched_red.is_empty() { "green" } else { "red" },
            field_text(&list_text(&patched_red)),
            field_text(&list_text(&direct_red)),
            entry.patch_count
        ));
    }
    panic_again_if_the_record_stage_panicked_at(
        panic_of_the_record_stage.as_ref(),
        KnownBadImagesStep::KnownBadEntries,
    );
    std::fs::create_dir_all(&command_line.output_directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", command_line.output_directory.display()));
    for (file_name, contents) in [
        ("known-bad-states.tsv", states_text.into_bytes()),
        ("known-bad-patches.bin", patches_file),
        ("known-bad-o2.tsv", pool_checker_text.into_bytes()),
    ] {
        let written = write_export_file(&command_line.output_directory, file_name, &contents);
        lines.emit_result_line(&format!(
            "name=file file={file_name} bytes={} sha256={}",
            written.bytes, written.sha256
        ));
    }
    let e170_rows_failed = if command_line.with_e170_bad_rows {
        let mut failed = 0usize;
        let mut e170_pool_checker_text = String::new();
        for row in &e170_bad_rows {
            let base_red = red_names_of_the_next_table();
            let red_by_the_pool_checker = red_names_of_the_next_table();
            let judged = judged_e170_bad_row(row, &base_red, &red_by_the_pool_checker);
            failed += usize::from(judged.row_failed);
            e170_pool_checker_text.push_str(&judged.pool_checker_file_row);
            lines.emit_result_line(&judged.result_line);
        }
        panic_again_if_the_record_stage_panicked_at(
            panic_of_the_record_stage.as_ref(),
            KnownBadImagesStep::E170BadRows,
        );
        let written = write_export_file(
            &command_line.output_directory,
            "e170-bad-rows-o2.tsv",
            e170_pool_checker_text.as_bytes(),
        );
        lines.emit_result_line(&format!(
            "name=file file=e170-bad-rows-o2.tsv bytes={} sha256={}",
            written.bytes, written.sha256
        ));
        failed
    } else {
        0
    };
    lines.emit_result_line(&format!(
        "name=verdict known_bad_entries_failing_s4={known_bad_failed} e170_bad_rows_failing_s3_or_s4={e170_rows_failed}"
    ));
    lines.finish();
    if known_bad_failed > 0 || e170_rows_failed > 0 {
        Ok(3)
    } else {
        Ok(0)
    }
}

#[cfg(test)]
mod tests;
