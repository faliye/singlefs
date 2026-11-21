//! 扫描内核读一条根自己实例表全部行的那张表：恰好装满与多一行交 CPU。

use super::*;

// ───────── 扫描内核读一条根自己实例表全部行的那张表：恰好装满与多一行 ─────────

/// 扫描内核读一条根自己实例表全部行的那张表（`crash_judge_checker_common.wgsl` 的 `C_R_MAX_ROWS`）此刻装几行：与恢复内核读同一种表的
/// `MAX_ROWS`（`crash_judge_common.wgsl`）同宽。
const INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD: usize = 24576;

/// 两张表抬到 24576 之前装几行：恰好 512 行那条流照旧判得了，513 行那条流多于它。
const INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE: usize = 512;

/// 第一次抬之前扫描内核那张表装几行：512 行那条流多于它。
const INSTANCE_TABLE_ROWS_THE_SCAN_KERNEL_HELD_BEFORE_THE_FIRST_RAISE: usize = 256;

/// 走读内核最新根实例表的行那一张（`C_MAX_INSTANCE_ROWS`）抬到 32768 之前装几行：24576 行那条流多于它。
const INSTANCE_ROWS_THE_WALK_KERNEL_HELD_BEFORE_THE_RAISE: usize = 1024;

const _: () = assert!(
    INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE
        > INSTANCE_TABLE_ROWS_THE_SCAN_KERNEL_HELD_BEFORE_THE_FIRST_RAISE
        && INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD
            > INSTANCE_ROWS_THE_WALK_KERNEL_HELD_BEFORE_THE_RAISE,
    "每条「判得了」的流的行数要多于那张表抬之前的行数"
);

/// 扫描内核那张表满了时池级 checker 判不了的原因字（`crash_judge_checker_scan.wgsl` 的 `r_instance_rows`，站点 32；条目过多那一类）。
const SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL: u32 =
    RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES | (32 << 8);

/// 两块盘（同 `crash_enumeration_position_addressed_trees.rs` 取号那条流）。
const DEVICES_OF_THE_ROW_PUBLISH_FLOW: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 一条录制写的稀疏盘：取号与可写挂载直接在它上面跑，录制流记下每一次写。
type RecordingSparseDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 一条流：起点镜像、被录那次可写挂载的写表与段、被判的那条根槽写，与那次挂载写出的每一条根（写行那次与之后的暖机）的
/// (实例, txg) 和它自己那张实例表沿链读出的行数。这一版没有文件，版本表为空。
struct RowPublishFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    roots_of_the_recorded_mount: Vec<(InstanceGeneration, CheckpointTxg, usize)>,
}

/// mkfs → 可写挂载一次（实例 1，树表 0 条，上一个实例是 0、不写行）→「取号之后崩溃」`instances_acquired_and_crashed` 次 →
/// 被录的那一次可写挂载：写行那次给实例 1 与中间每个取过号的实例各写一行，共 `instances_acquired_and_crashed + 1` 行
/// （一片实例表装 369 行，五百来行是两片），之后零单元暖机，暖机的根照抄这张表。造法同 `crash_enumeration_position_addressed_trees.rs`
/// 的 `VersionWithoutFileRowPublishWithItsOwnTree`（那条取号 369 次、写 370 行，扫描内核那张表 256 行时在站点 32 判不了）。
/// 写表只录那一次挂载（取号、写行、暖机）；起点镜像是它之前的整段录制流施加出来的。
fn record_the_row_publish_flow(instances_acquired_and_crashed: usize) -> RowPublishFlow {
    let stream = SharedStream::retaining_contents();
    let mut devices: RecordingSparseDevices = DEVICES_OF_THE_ROW_PUBLISH_FLOW
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let first_mount = mount_writable(&parameters(), &mut devices).expect("mkfs 之后第一次可写挂载");
    assert!(
        first_mount.output.rows_written.is_empty(),
        "上一个实例是 0：mkfs 之后第一次可写挂载不写行，被录那次挂载写的行就是那张表的全部行：{:?}",
        first_mount.output.rows_written
    );
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        // 迭代上界：`instances_acquired_and_crashed` 次取号。
        for _ in 0..instances_acquired_and_crashed {
            acquire_instance(&mut writer).expect("取号之后崩溃");
        }
    }
    let operations_before_the_mount = stream.operations().len();
    let mut base = MemoryPool::with_devices(&DEVICES_OF_THE_ROW_PUBLISH_FLOW, IMAGE_BYTES);
    base.apply(&stream.retained_operations()[..operations_before_the_mount]);
    let mounted = mount_writable(&parameters(), &mut devices).expect("树表 0 条的一版上写行");
    let operations = stream.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[operations_before_the_mount..], &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写行与暖机各有根槽写");
    let roots_of_the_recorded_mount = std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .map(|version| {
            let root = version.root();
            let rows = instance_table_chain_of_root(&devices, root)
                .expect("被录那次挂载刚写的实例表沿链读得出")
                .records
                .rows
                .len();
            (root.instance, root.checkpoint_txg, rows)
        })
        .collect();
    RowPublishFlow {
        base,
        writes,
        segments,
        judged_root_index,
        roots_of_the_recorded_mount,
    }
}

/// 被录那次挂载写出的每一条根自己那张实例表都恰好 `rows` 行。
fn assert_every_root_of_the_recorded_mount_has_an_instance_table_of(
    flow_name: &str,
    flow: &RowPublishFlow,
    rows: usize,
) {
    assert!(
        !flow.roots_of_the_recorded_mount.is_empty()
            && flow
                .roots_of_the_recorded_mount
                .iter()
                .all(|(_, _, rows_of_the_root)| *rows_of_the_root == rows),
        "流 {flow_name}：被录那次挂载写出的每一条根自己那张实例表都是 {rows} 行（每一条根的实例、txg、行数：{:?}）",
        flow.roots_of_the_recorded_mount
    );
}

/// 一个状态 CPU 那一侧判出的（实例表行那两条流）：对拍字、红项、看 journal 那一遍恢复落没落到被录那次挂载写的根、被判那次的根槽写落没落。
struct CpuJudgementOfARowPublishState {
    cross_check_words: CrossCheckWords,
    red_items: Vec<String>,
    consulted_recovery_lands_on_a_root_of_the_recorded_mount: bool,
    judged_root_persisted: bool,
}

/// 每个状态两边各判一遍（实例表行那两条流）：GPU 结论字与 CPU 那一侧，按状态序号。卡借不到（等满了、或没有独显）交 `None`。
fn judge_every_row_publish_state_on_both_sides(
    cards: GpuCardPoolStartedAtTheFirstLease,
    test_name: &str,
    flow: &RowPublishFlow,
) -> Option<(
    Vec<[u32; VERDICT_WORDS]>,
    Vec<CpuJudgementOfARowPublishState>,
)> {
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let no_versions: Vec<PublishedVersion> = Vec::new();
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &no_versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    let state_count =
        crash_point_replay_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count, "表与枚举的状态数相同");
    let gpu = judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)?;
    let roots_of_the_recorded_mount: HashSet<(InstanceGeneration, CheckpointTxg)> = flow
        .roots_of_the_recorded_mount
        .iter()
        .map(|(instance, txg, _)| (*instance, *txg))
        .collect();
    let mut cpu: Vec<CpuJudgementOfARowPublishState> = Vec::new();
    let item_name = format!("tables_at_their_capacity::{test_name}::row_publish_cpu_side");
    let judged_on_the_cpu = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &no_versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| {
            cpu.push(CpuJudgementOfARowPublishState {
                cross_check_words: cross_check_words_of_cpu_judgement(judgement),
                red_items: judgement.red_items(),
                consulted_recovery_lands_on_a_root_of_the_recorded_mount: judgement
                    .consulted
                    .effective_root
                    .is_some_and(|root| roots_of_the_recorded_mount.contains(&root)),
                judged_root_persisted: judgement.root_persisted,
            });
        },
    )
    .judged()?;
    assert_eq!(judged_on_the_cpu, state_count, "CPU 判了每个状态");
    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
    Some((gpu, cpu))
}

/// 实例表行那两条流本身合法：CPU 判器每个状态都不红。
fn assert_the_cpu_judge_finds_every_row_publish_state_green(
    flow_name: &str,
    cpu: &[CpuJudgementOfARowPublishState],
) {
    let red: Vec<String> = cpu
        .iter()
        .enumerate()
        .filter(|(_, judgement)| !judgement.red_items.is_empty())
        .take(8)
        .map(|(ordinal, judgement)| format!("状态 {ordinal}：{:?}", judgement.red_items))
        .collect();
    assert!(
        red.is_empty(),
        "流 {flow_name} 合法，CPU 判器每个状态都不该红（前几个）：\n{}",
        red.join("\n")
    );
}

/// 取号之后崩溃 `rows - 1` 次再挂载：被录那次挂载写出的每条根自己那张实例表恰好 `rows` 行（取卡之前在 CPU 上核）。
/// 每个状态 GPU 两个内核都判得了，对拍字与 CPU 逐状态相同；要有被判那条根落了的状态（扫描内核读那张 `rows` 行的表）。
fn assert_the_gpu_judge_judges_roots_whose_own_instance_table_has_these_many_rows_like_the_cpu_judge(
    test_name: &str,
    flow_name: &str,
    rows: usize,
) {
    let flow = record_the_row_publish_flow(rows - 1);
    assert_every_root_of_the_recorded_mount_has_an_instance_table_of(flow_name, &flow, rows);
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_row_publish_state_on_both_sides(cards, test_name, &flow)
    else {
        return;
    };
    assert_the_cpu_judge_finds_every_row_publish_state_green(flow_name, &cpu);
    assert!(
        cpu.iter().any(|judgement| judgement.judged_root_persisted),
        "流 {flow_name}：要有被判那次的根槽写落了的状态（扫描内核读那条根 {rows} 行的表）"
    );
    let cpu_cross_check_words: Vec<CrossCheckWords> = cpu
        .iter()
        .map(|judgement| judgement.cross_check_words)
        .collect();
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(
        flow_name,
        &gpu,
        &cpu_cross_check_words,
    );
}

/// 取号之后崩溃 511 次再挂载：被录那次挂载写出的每条根自己那张实例表恰好 512 行（多于第一次抬之前扫描内核那张表的 256 行）。
/// 每个状态 GPU 两个内核都判得了，对拍字与 CPU 逐状态相同。扫描内核那张表改回 256 行时，被判那条根落了的状态在站点 32 记判不了，这条红。
#[test]
fn the_gpu_judge_judges_roots_whose_own_instance_table_has_exactly_five_hundred_and_twelve_rows_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_roots_whose_own_instance_table_has_these_many_rows_like_the_cpu_judge(
        "the_gpu_judge_judges_roots_whose_own_instance_table_has_exactly_five_hundred_and_twelve_rows_like_the_cpu_judge",
        "instance-table-rows-at-the-old-capacity-of-the-gpu-tables",
        INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE,
    );
}

/// 取号之后崩溃 512 次再挂载：每条根自己那张实例表 513 行（多于两张表抬到 24576 之前的 512 行）。每个状态 GPU 两个内核都判得了，
/// 对拍字与 CPU 逐状态相同。扫描内核那张表改回 512 行时被判那条根落了的状态在站点 32 记判不了，恢复内核那张改回 512 行时
/// 恢复落到那次挂载的根的状态记条目过多：两样都让这条红。
#[test]
fn the_gpu_judge_judges_roots_whose_own_instance_table_has_five_hundred_and_thirteen_rows_past_the_old_capacity_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_roots_whose_own_instance_table_has_these_many_rows_like_the_cpu_judge(
        "the_gpu_judge_judges_roots_whose_own_instance_table_has_five_hundred_and_thirteen_rows_past_the_old_capacity_like_the_cpu_judge",
        "instance-table-rows-just-past-the-old-capacity-of-the-gpu-tables",
        INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE + 1,
    );
}

/// 取号之后崩溃 24575 次再挂载：每条根自己那张实例表恰好 24576 行（67 片）。每个状态 GPU 两个内核都判得了，对拍字与 CPU 逐状态相同：
/// 扫描内核、恢复内核那两张表恰好装满；走读内核最新根实例表的行那一张（`C_MAX_INSTANCE_ROWS`）多于抬之前的 1024 行，改回 1024 时
/// 走读那条根的状态在站点 8 记判不了，这条红（这一张的散列索引在这条流上建起来：几万行时按桶表找）。
#[test]
fn the_gpu_judge_judges_roots_whose_own_instance_table_has_exactly_twenty_four_thousand_five_hundred_and_seventy_six_rows_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_roots_whose_own_instance_table_has_these_many_rows_like_the_cpu_judge(
        "the_gpu_judge_judges_roots_whose_own_instance_table_has_exactly_twenty_four_thousand_five_hundred_and_seventy_six_rows_like_the_cpu_judge",
        "instance-table-rows-at-the-capacity-of-the-gpu-tables",
        INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD,
    );
}

/// 取号之后崩溃 24576 次再挂载：被录那次挂载写出的每条根自己那张实例表 24577 行（多新上限一行）。被判那条根落了的状态，
/// 扫描内核读到第 24577 行时在站点 32 记判不了（[`SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL`]；走读内核那一张 32768 行装得下，扫描内核跑得到）；
/// 看 journal 那一遍恢复落到那次挂载写的根的状态，恢复内核读那一版的实例表读到第 24577 行同样记判不了（条目过多 [`RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES`]，两个内核的表同宽）。
/// 两个内核判不了的只许是这两个原因；两个内核都判得了的状态对拍字与 CPU 相同；CPU 判器每个状态照旧判、都不红。
#[test]
fn the_gpu_judge_leaves_roots_whose_own_instance_table_has_twenty_four_thousand_five_hundred_and_seventy_seven_rows_to_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_leaves_roots_whose_own_instance_table_has_twenty_four_thousand_five_hundred_and_seventy_seven_rows_to_the_cpu_judge";
    const FLOW_NAME: &str = "instance-table-rows-just-past-the-capacity-of-the-gpu-tables";
    let flow = record_the_row_publish_flow(INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD);
    assert_every_root_of_the_recorded_mount_has_an_instance_table_of(
        FLOW_NAME,
        &flow,
        INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD + 1,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_row_publish_state_on_both_sides(cards, TEST_NAME, &flow)
    else {
        return;
    };
    assert_the_cpu_judge_finds_every_row_publish_state_green(FLOW_NAME, &cpu);
    let mut states_left_to_the_cpu_at_site_thirty_two = 0usize;
    let mut problems: Vec<String> = Vec::new();
    for (ordinal, (cpu_judgement, gpu_words)) in cpu.iter().zip(&gpu).enumerate() {
        let recovery_could_not_judge = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0;
        let checker_reason = gpu_words[POOL_CHECKER_UNDECIDABLE_WORD];
        if recovery_could_not_judge && gpu_words[1] != RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES {
            problems.push(format!(
                "状态 {ordinal}：GPU 恢复内核判不了的原因 {:#x} 不是条目过多（{RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES}）",
                gpu_words[1]
            ));
        }
        if cpu_judgement.consulted_recovery_lands_on_a_root_of_the_recorded_mount
            && !recovery_could_not_judge
        {
            problems.push(format!(
                "状态 {ordinal}：看 journal 那一遍恢复落到 24577 行那张表的根，GPU 恢复内核该记判不了（条目过多），实际判得了、第 1 字 {:#x}",
                gpu_words[1]
            ));
        }
        if checker_reason != 0 && checker_reason != SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL {
            problems.push(format!(
                "状态 {ordinal}：GPU 池级 checker 判不了的原因字 {checker_reason:#x} 不是站点 32（{SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL:#x}）"
            ));
        }
        if cpu_judgement.judged_root_persisted
            && checker_reason != SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL
        {
            problems.push(format!(
                "状态 {ordinal}：24577 行那张表的被判那条根槽写落了，扫描内核该在站点 32 记判不了（{SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL:#x}），实际原因字 {checker_reason:#x}"
            ));
        }
        if checker_reason == SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL {
            states_left_to_the_cpu_at_site_thirty_two += 1;
        }
        if gpu_could_not_judge(gpu_words) {
            continue;
        }
        let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
        if !cross_check_words_agree_in_the_cross_check(
            &gpu_cross_check_words,
            &cpu_judgement.cross_check_words,
        ) {
            problems.push(format!(
                "状态 {ordinal}：GPU 判得了而与 CPU 不一致：GPU {gpu_cross_check_words:?}；CPU {:?}",
                cpu_judgement.cross_check_words
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "流 {FLOW_NAME}：{} 处不对（前几处）：\n{}",
        problems.len(),
        problems
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        states_left_to_the_cpu_at_site_thirty_two > 0,
        "流 {FLOW_NAME}：要有扫描内核在站点 32 记判不了、交 CPU 判的状态（被判那条根槽写落了的状态）"
    );
}
