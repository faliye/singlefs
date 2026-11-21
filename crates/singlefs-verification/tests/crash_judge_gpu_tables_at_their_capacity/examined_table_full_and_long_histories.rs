//! I-3.10 的去重表满了之后照判，一版分配记录多于 4096 条的整理夹着回退历史，越过原分配记录墙的随机历史上枚举写表的宽与一个落点的写数。

use super::*;

// ───────── I-3.10 的去重表满了之后照判：老根的分配记录树整棵改代的不合法镜像 ─────────

/// 一直留着的那个文件长到几个数据单元：之后每一版的未释放记录里都有这一批（两块盘各一条），合起来四百来条。
const DATA_UNITS_KEPT_ACROSS_THE_VERSIONS: usize = 200;

/// 文件长好之后再发几次只建一个空 inode 的发布（文件不变）：连长文件那一次，最新那条根之前有这么多条老根，每条的分配记录树都照抄改代。
/// 取十九条：每条改过的老根多出它自己那四百多条未释放记录，十九条之后越过 I-3.10 去重表的 [`EXAMINED_RECORDS_THE_SCAN_KERNEL_HOLDS`] 条
/// （CPU 一侧量的，同一造法：改代之前各版累计 846 条、改代之后 9092 条，每版至多 848 条分配记录；改代之后池级 checker 判红的是
/// I-3.1、I-3.10、I-3.15、I-8.10 四条）。去重表还是 4096 条时取的是十条（改代之后 5006 条）。根环三个区域各 8 槽：长文件那一版（txg 4）
/// 到最新那条根（txg 23）连着 20 条根，环还没转回来盖掉哪一条（照抄老根要在根环里读到它发布时写的那一格）。
const OLD_ROOTS_WHOSE_ALLOCATION_RECORD_TREES_ARE_COPIED: usize = 19;

/// 第 k 条老根（从 0 数）那棵树里每条记录的代号加 (k + 1) × 这个数：各条老根改出来的代号互不相同，也不与原来的相同。
const GENERATION_OFFSET_STEP_OF_THE_COPIED_TREES: u64 = 1000;

/// 抄出来的节点从单元区起点往后这么多个槽起放：这条流分配过的槽都在它前面（几百个单元），那一段没写过（取槽时逐个核全 0）。
const SLOTS_BETWEEN_THE_UNIT_AREA_START_AND_THE_COPIES: u64 = 100_000;

/// 改代之后 CPU 判器判红的那一条：未释放记录的代号与它罩住的单元头里的诞生代号不同。
const ALLOCATION_GENERATION_MATCHES_THE_UNIT_BIRTH: &str = "I-3.10";

/// GPU 对拍的 CPU 参照：一张镜像作一个状态的给定状态表（写表空、一个空的持久集合）经对拍那一类的库读回
/// （`gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library`），CPU 判器判读回的那一份。只录那一趟交 `None`。
// cpu-reference-for-gpu-comparison
fn cpu_conclusion_of_an_image_read_back_from_the_pairing_library(
    table_name: &str,
    image: &MemoryPool,
) -> Option<GivenStateConclusion> {
    let read_back =
        cpu_reference_table_read_back_from_the_pairing_library(&RecordedGivenStatesTable {
            table_name: table_name.to_string(),
            base: image.clone(),
            writes: Vec::new(),
            segments: Vec::new(),
            seams: Vec::new(),
            persisted_sets: vec![Vec::new()],
        })?;
    let mut conclusions =
        read_back.with_the_table(|table| judge_the_given_states_on_the_cpu(table));
    assert_eq!(
        conclusions.states.len(),
        1,
        "{table_name}：一张镜像一个状态"
    );
    Some(conclusions.states.remove(0))
}

/// 表满之后照判（扫描内核 I-3.10 的去重表 `C_MAX_EXAMINED`）在卡上过一遍：新池新建文件之后把文件长到 [`DATA_UNITS_KEPT_ACROSS_THE_VERSIONS`] 个单元，
/// 再发 [`OLD_ROOTS_WHOSE_ALLOCATION_RECORD_TREES_ARE_COPIED`] 次只建一个空 inode 的发布；最新那条根之前那几条老根（长文件那一次起）各把分配记录树
/// 整棵照抄、改代、重封（`common_allocation_record_trees_copied_under_other_generations`）。每一版的分配记录不多于两张表的 8192 条，
/// 各版未释放记录去重累计多于去重表的 8192 条（不合法：合法的流上累计不多于最新那一版的分配记录条数）。这张镜像作一个状态的给定状态表：
/// GPU 判器判得了（没有判不了），对拍字与 CPU 判器逐位相同（两遍 oracle 的位不比：给定状态表那一路没有期望版本表，两边都不判 oracle，
/// GPU 内核照空版本表判出的「读出了文件、这一版却没有」两位不算数），CPU 判器判 I-3.10 红（改过的代与单元头里的诞生代号不同）。
/// 判别力（`crates/mutations.tsv`）：去重表满了退回记判不了（站点 31、返回），GPU 那一位判不了，这条红；GPU 一侧的对拍字不去掉 oracle 的位，
/// 多出两遍 oracle 各一位（位 5、位 13），这条红。
#[test]
fn the_gpu_judge_judges_on_past_a_full_examined_table_like_the_cpu_judge_on_an_image_whose_old_roots_carry_their_allocation_records_under_other_generations(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_judges_on_past_a_full_examined_table_like_the_cpu_judge_on_an_image_whose_old_roots_carry_their_allocation_records_under_other_generations";
    const TABLE_NAME: &str =
        "old-roots-with-their-allocation-record-trees-copied-under-other-generations";
    let mut pool = build_pool(TABLE_NAME);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    let mut counts = vec![(
        pool.output.root.checkpoint_txg,
        allocation_record_counts_of(&pool.output),
    )];
    let mut examined = HashSet::new();
    note_the_unreleased_records(&pool.output, &mut examined);
    publish_a_sequential_write(
        &mut pool,
        DATA_UNITS_KEPT_ACROSS_THE_VERSIONS,
        &mut versions,
        &mut counts,
        &mut examined,
    );
    let oldest_copied_txg = pool.output.root.checkpoint_txg;
    // 迭代上界：OLD_ROOTS_WHOSE_ALLOCATION_RECORD_TREES_ARE_COPIED 次发布
    for _publish in 0..OLD_ROOTS_WHOSE_ALLOCATION_RECORD_TREES_ARE_COPIED {
        publish_new_inodes_and_note_the_version(
            &mut pool,
            1,
            &mut versions,
            &mut counts,
            &mut examined,
        );
    }
    let newest_txg = pool.output.root.checkpoint_txg;
    let mut image = pool.memory_pool();
    let mut free_slots = FreeSlotsForTheCopies::starting_at(
        singlefs_format::UNIT_AREA_START_SLOT + SLOTS_BETWEEN_THE_UNIT_AREA_START_AND_THE_COPIES,
    );
    let mut examined_with_the_copies = examined.clone();
    let mut old_roots_copied = 0;
    // 迭代上界：最新那条根之前、长文件那一次起的每一条老根各一次（OLD_ROOTS_WHOSE_ALLOCATION_RECORD_TREES_ARE_COPIED 条）
    for (position, txg) in (oldest_copied_txg.0..newest_txg.0).enumerate() {
        let copied = copy_the_allocation_record_tree_of_an_old_root_under_another_generation(
            &mut image,
            &parameters(),
            CheckpointTxg(txg),
            GENERATION_OFFSET_STEP_OF_THE_COPIED_TREES
                * (u64::try_from(position).expect("几条老根装得进 u64") + 1),
            &mut free_slots,
        );
        examined_with_the_copies.extend(copied);
        old_roots_copied += 1;
    }
    assert_eq!(
        old_roots_copied, OLD_ROOTS_WHOSE_ALLOCATION_RECORD_TREES_ARE_COPIED,
        "最新那条根之前、长文件那一次起的每一条老根都照抄改代了"
    );
    let most_records_of_a_version = counts
        .iter()
        .map(|(_, counts_of_the_version)| counts_of_the_version.every_record)
        .max()
        .expect("至少一版");
    assert!(
        most_records_of_a_version <= RECORDS_THE_GPU_TABLES_HOLD,
        "每一版的分配记录不多于两张表的 {RECORDS_THE_GPU_TABLES_HOLD} 条（每一版：{counts:?}）"
    );
    assert!(
        examined.len() <= EXAMINED_RECORDS_THE_SCAN_KERNEL_HOLDS
            && examined_with_the_copies.len() > EXAMINED_RECORDS_THE_SCAN_KERNEL_HOLDS,
        "改代之前各版未释放记录去重累计 {} 条装得进去重表的 {EXAMINED_RECORDS_THE_SCAN_KERNEL_HOLDS} 条，改代之后 {} 条越过它",
        examined.len(),
        examined_with_the_copies.len()
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let not_expanded =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let tables = build_judge_tables_for_the_given_persisted_sets(
        &JudgeFlow {
            base: &image,
            writes: &[],
            segments: &[],
            judged_root_index: 0,
            versions: &[],
            expansion: &not_expanded,
        },
        &[],
        vec![Vec::new()],
    )
    .expect("一张镜像一个状态的给定状态表建得出判表");
    let Some(gpu) = judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
    else {
        return;
    };
    let [gpu_words] = gpu.as_slice() else {
        panic!("一个状态一份结论，交回 {} 份", gpu.len());
    };
    let Some(cpu) =
        cpu_conclusion_of_an_image_read_back_from_the_pairing_library(TABLE_NAME, &image)
    else {
        return;
    };
    let cpu_violations: Vec<&'static str> = cpu
        .pool_checker_violations()
        .into_iter()
        .map(|(invariant, _detail)| invariant)
        .collect();
    assert!(
        cpu_violations.contains(&ALLOCATION_GENERATION_MATCHES_THE_UNIT_BIRTH),
        "前提：CPU 判器判 {ALLOCATION_GENERATION_MATCHES_THE_UNIT_BIRTH} 红（改过的代与单元头里的诞生代号不同）：判红的是 {cpu_violations:?}"
    );
    assert!(
        !gpu_could_not_judge(gpu_words),
        "GPU 判得了这个状态（恢复第 1 字 {:#x}、池级 checker 原因字 {:#x}）：去重表满了之后照判",
        gpu_words[1],
        gpu_words[POOL_CHECKER_UNDECIDABLE_WORD]
    );
    // 给定状态表那一路两边都不判 oracle（没有期望版本表）：GPU 一侧的对拍字先去掉两遍 oracle 的位再比，与引擎收卡上结论那一处
    // （`crash_amplification` 里 `GivenStatesOnTheGpu::Judged` 的对拍字）同一个换法；CPU 一侧（`crash_given_states::judge_a_given_state_on_the_cpu`
    // 经 `crash_judge_conclusions::cross_check_words_of_a_given_state`）那十四位恒为 0
    assert_eq!(
        cross_check_words_without_the_oracle(cross_check_words_of_gpu_verdict(gpu_words)),
        cpu.words,
        "GPU 与 CPU 的对拍字逐位相同（两遍 oracle 的位不比；CPU 判红的是 {cpu_violations:?}）"
    );
}

// ───────── 回收过的流上一版分配记录多于 4096 条：整理夹着回退那一组比重拉到 160 步 ─────────

/// 整理夹着回退那一组比重拉长的那一段：种子区间里的偏移与步数。这一组比重 4 段 × 80 步、拉到 160 步回退之后的镜像里，一版分配记录多于 4096 条的
/// 都出在这一段（欠账 C894；CPU 一侧数的引用至多一千二百来条，走读内核那几张 4096 条的表装得下）。160 步的前 80 步与 80 步那一段
/// 逐步相同（同一个种子生成的步序列，前缀一样），80 步那一段的 11 张都在 160 步那 27 张里。
const OFFSET_OF_THE_LONGER_COMPACTION_AND_ROLLBACK_HISTORY: u64 = 2;

const OPERATIONS_OF_THE_LONGER_COMPACTION_AND_ROLLBACK_HISTORY: usize = 160;

/// 按 checker 的读法在镜像上数根环里每一条自证过的根下的分配记录（`singlefs_checker::walk::allocation_record_count_under_root`，
/// 与实现的分配器不共用代码），交回最多的那一条根的条数；一条都数不出交 0。
///
/// # Panics
/// 镜像上一块盘都择不到系统配置。
fn most_allocation_records_under_a_root_of(image: &MemoryPool) -> usize {
    let geometry = chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_device, chosen)| chosen.map(|(_view, geometry)| geometry))
        .expect("回退之后的镜像择得到系统配置");
    valid_roots(image, &geometry)
        .iter()
        .filter_map(|(_region, _slot, view)| allocation_record_count_under_root(image, view))
        .max()
        .unwrap_or(0)
}

/// 回收过的流上一版的分配记录多于抬到 8192 之前的 4096 条、引用不过 4096 条：整理夹着回退那一组比重在偏移
/// [`OFFSET_OF_THE_LONGER_COMPACTION_AND_ROLLBACK_HISTORY`] 那一段种子下跑 [`OPERATIONS_OF_THE_LONGER_COMPACTION_AND_ROLLBACK_HISTORY`] 步，
/// 回退之后的镜像（30 张）里挑出一条根下的分配记录多于 4096 条的（CPU 一侧量的 27 张；前 80 步那 11 张：`Operation(40)` 4246 条，
/// `Operation(44)`、`(46)`、`(47)`、`(52)`、`(53)` 各 4248 条，`(54)` 4250、`(56)` 4276、`(59)` 4294、`(67)` 4360、`(72)` 4400 条；后 80 步那 16 张：
/// `(86)` 4502、`(95)` 4518、`(103)`、`(104)` 各 4580、`(105)`、`(109)` 各 4584、`(110)` 4586、`(118)` 4636、`(119)` 4642、`(126)` 4676、`(135)` 4692、
/// `(141)`、`(144)` 各 4704、`(151)` 4718、`(152)`、`(153)` 各 4728 条；不挑的三张 `(1)` 28、`(17)` 2744、`(35)` 4018 条），每张作一个状态的给定状态表（写表空、
/// 一个空持久集合）交 GPU 判器与 CPU 判器各判一遍：GPU 判得了（没有判不了），对拍字与 CPU 判器逐位相同（两遍 oracle 的位不比：给定状态表那一路
/// 没有期望版本表，两边都不判 oracle）。扫描内核收一条根的分配记录、I-5.4 排相邻两条、I-3.16 按记录口径数全空段都在多于 4096 条的表上走一遍。
/// 判别力（`crates/mutations.tsv`）：扫描内核那张表改回 4096 条时这几张在站点 20 判不了，这条红。I-5.4、I-3.16 的次序表改回借 `C_ROW_ORDER`
/// 那一处改法这条红不了：溢出 `C_ROW_ORDER` 的那一截落进它后面空着的那一段（journal 组键挪到草稿区末尾之前的原处），扫描内核不读那一段；
/// 那两处调用点由 [`the_scan_kernel_orders_the_allocation_records_in_the_allocation_record_row_order_at_both_call_sites`] 在 naga 中间表示上钉。
#[test]
fn the_gpu_judge_judges_rollback_images_of_a_longer_compaction_and_rollback_history_past_four_thousand_and_ninety_six_allocation_records_in_a_version_like_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_judges_rollback_images_of_a_longer_compaction_and_rollback_history_past_four_thousand_and_ninety_six_allocation_records_in_a_version_like_the_cpu_judge";
    let seed = seed_of_the_compaction_and_rollback_sampling(
        OFFSET_OF_THE_LONGER_COMPACTION_AND_ROLLBACK_HISTORY,
    );
    let history = compaction_and_rollback_history(
        seed,
        OPERATIONS_OF_THE_LONGER_COMPACTION_AND_ROLLBACK_HISTORY,
    );
    let HistoryWithItsStepImages { run, step_images } = run_the_history_keeping_the_step_images(
        &history,
        &format!("seed-{seed}"),
        &is_a_rollback_step,
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "种子 {seed}：历史要走完、每一步与模型对上；{:?}",
        run.outcomes
    );
    let rollback_images = step_images.len();
    let images_past_the_old_capacity: Vec<(String, MemoryPool, usize)> = step_images
        .into_iter()
        .map(|(table_name, image)| {
            let most_records = most_allocation_records_under_a_root_of(&image);
            (table_name, image, most_records)
        })
        .filter(|(_table_name, _image, most_records)| {
            *most_records
                > RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE_TO_EIGHT_THOUSAND_ONE_HUNDRED_AND_NINETY_TWO
        })
        .collect();
    let counts: Vec<(&str, usize)> = images_past_the_old_capacity
        .iter()
        .map(|(table_name, _image, most_records)| (table_name.as_str(), *most_records))
        .collect();
    assert!(
        !images_past_the_old_capacity.is_empty(),
        "种子 {seed}：回退之后的 {rollback_images} 张镜像里要有一条根下的分配记录多于 {RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE_TO_EIGHT_THOUSAND_ONE_HUNDRED_AND_NINETY_TWO} 条的"
    );
    assert!(
        counts
            .iter()
            .all(|(_table_name, most_records)| *most_records <= RECORDS_THE_GPU_TABLES_HOLD),
        "挑出来的镜像一条根下的分配记录不多于两张表的 {RECORDS_THE_GPU_TABLES_HOLD} 条：{counts:?}"
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let not_expanded =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let mut problems: Vec<String> = Vec::new();
    for (table_name, image, most_records) in &images_past_the_old_capacity {
        let tables = build_judge_tables_for_the_given_persisted_sets(
            &JudgeFlow {
                base: image,
                writes: &[],
                segments: &[],
                judged_root_index: 0,
                versions: &[],
                expansion: &not_expanded,
            },
            &[],
            vec![Vec::new()],
        )
        .expect("一张镜像一个状态的给定状态表建得出判表");
        let Some(gpu) = judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
        else {
            return;
        };
        let [gpu_words_of_the_image] = gpu.as_slice() else {
            panic!("{table_name}：一个状态一份结论，交回 {} 份", gpu.len());
        };
        let Some(cpu) =
            cpu_conclusion_of_an_image_read_back_from_the_pairing_library(table_name, image)
        else {
            return;
        };
        if gpu_could_not_judge(gpu_words_of_the_image) {
            problems.push(format!(
                "{table_name}（一条根下 {most_records} 条分配记录）：GPU 判不了（恢复第 1 字 {:#x}、池级 checker 原因字 {:#x}）",
                gpu_words_of_the_image[1],
                gpu_words_of_the_image[POOL_CHECKER_UNDECIDABLE_WORD]
            ));
            continue;
        }
        // 给定状态表那一路两边都不判 oracle：GPU 一侧先去掉两遍 oracle 的位（同表满照判那一条的换法），再照对拍的比法比
        let gpu_cross_check_words_without_the_oracle = cross_check_words_without_the_oracle(
            cross_check_words_of_gpu_verdict(gpu_words_of_the_image),
        );
        if !cross_check_words_agree_in_the_cross_check(
            &gpu_cross_check_words_without_the_oracle,
            &cpu.words,
        ) {
            problems.push(format!(
                "{table_name}（一条根下 {most_records} 条分配记录）：GPU 判得了而与 CPU 不一致：GPU {gpu_cross_check_words_without_the_oracle:?}；CPU {:?}",
                cpu.words
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "种子 {seed}：{} 张镜像 GPU 判不了或与 CPU 不一致（一共 {} 张）：\n{}",
        problems.len(),
        images_past_the_old_capacity.len(),
        problems.join("\n")
    );
}

// ───────── 枚举写表的宽与一个落点的写数：随机历史越过原分配记录墙那一族 ─────────

/// 随机历史越过原分配记录墙那一族的一段（种子、步数、比重、怎么跑与 `crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`
/// 全量那一族逐项相同）：全量那一趟它建不出判表（`EnumeratedWritesPastTheMaskWidth`，这一族每段录制写 4436 到 5265 次、
/// 盘 0 系统配置槽 0 号录制写 88 到 111 次）。
const SEED_OF_A_HISTORY_TOWARD_THE_ALLOCATION_RECORD_WALL: u64 = 7_463_871_032_432_355_142;

const OPERATIONS_OF_A_HISTORY_TOWARD_THE_ALLOCATION_RECORD_WALL: usize = 150;

/// 两个上限抬之前装几次：枚举写表 2048 次（持久掩码 64 个字）、一个落点 64 次写（组合掩码两个字）。
const ENUMERATED_WRITES_THE_MASK_HELD_BEFORE_THE_RAISE: usize = 2048;

const WRITES_PER_LOCATION_THE_COMBINATION_HELD_BEFORE_THE_RAISE: usize = 64;

/// GPU 对拍的 CPU 参照：一张给定状态表经对拍那一类的库读回（`gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library`），
/// CPU 判器逐状态判读回的那一份。只录那一趟交 `None`。
// cpu-reference-for-gpu-comparison
fn cpu_conclusions_of_a_table_read_back_from_the_pairing_library(
    table: &RecordedGivenStatesTable,
) -> Option<Vec<GivenStateConclusion>> {
    let read_back = cpu_reference_table_read_back_from_the_pairing_library(table)?;
    Some(
        read_back
            .with_the_table(|read_back_table| judge_the_given_states_on_the_cpu(read_back_table))
            .states,
    )
}

/// 随机历史越过原分配记录墙那一族的一段（[`SEED_OF_A_HISTORY_TOWARD_THE_ALLOCATION_RECORD_WALL`]）每一步的镜像建成一张给定状态表
/// （基线是 mkfs 之后的池、写表是 mkfs 之后的录制流、不接撕裂镜像，每步镜像一个「前缀全落、之后一个都没落」的持久掩码）：
/// 枚举写表多于抬之前的 2048 次、盘 0 系统配置槽 0 号多于抬之前的 64 次写，判表建得出；每个状态 GPU 都判得了，对拍字（两遍 oracle 的位不比，
/// 给定状态表那一路没有期望版本表）与 CPU 判器逐状态相同。这一条在卡上走过 `PERSISTED_MASK_WORDS`（枚举写表上限 ÷ 32）
/// 个字的持久掩码与多于两个字的组合掩码（`crash_judge_common.wgsl` 的 `version_of_location` 逐字比）。
/// 判别力（`crates/mutations.tsv`）：版本表只打包组合的前两个字（第 64 次写往后的位丢了），盘 0 系统配置槽 0 号落了多于 64 次写的状态
/// 内核在版本表里找不到它的组合、记判不了，这条红。
#[test]
fn the_gpu_judge_judges_every_step_image_of_a_history_past_the_old_mask_width_and_writes_per_location_like_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_judges_every_step_image_of_a_history_past_the_old_mask_width_and_writes_per_location_like_the_cpu_judge";
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let history = generate_history_with_weights(
        HistorySeed(SEED_OF_A_HISTORY_TOWARD_THE_ALLOCATION_RECORD_WALL),
        OPERATIONS_OF_A_HISTORY_TOWARD_THE_ALLOCATION_RECORD_WALL,
        &GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
    );
    let stream = SharedStream::retaining_contents();
    let mut operations_recorded_after_each_step: Vec<usize> = Vec::new();
    let run = execute_history_handing_out_every_step_image(
        &history,
        execution,
        &stream,
        &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
        &mut |_| {},
        &mut |step_image| {
            operations_recorded_after_each_step.push(step_image.operations_recorded);
        },
    );
    assert!(
        run.earliest_stream_index_after_each_power_loss.is_empty(),
        "前提：这一段没掉过电，每步镜像都在一张表里、不带接缝"
    );
    let operations = stream.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations,
        &execution.device_width.fixed_geometry(),
    );
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        run.operations_written_by_make_filesystem,
        execution,
    );
    let persisted_sets: Vec<Vec<bool>> = operations_recorded_after_each_step
        .iter()
        .map(|operations_recorded| {
            let persisted_writes = stream_indexes
                .iter()
                .filter(|stream_index| **stream_index < *operations_recorded)
                .count();
            let persisted_over_the_whole_write_table: Vec<bool> = (0..writes.len())
                .map(|write_index| write_index < persisted_writes)
                .collect();
            history_after_the_make_filesystem
                .persisted_set_of_the_table(&persisted_over_the_whole_write_table)
        })
        .collect();
    let table_writes = history_after_the_make_filesystem.writes();
    let not_expanded =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let claimed_versions =
        vec![ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery; persisted_sets.len()];
    let tables =
        build_judge_tables_for_the_given_persisted_sets_of_the_recorded_writes_broken_at_the_seams_with_their_claimed_versions(
            &JudgeFlow {
                base: history_after_the_make_filesystem.base_pool(),
                writes: table_writes,
                segments: history_after_the_make_filesystem.segments(),
                judged_root_index: 0,
                versions: &[],
                expansion: &not_expanded,
            },
            &[],
            persisted_sets.clone(),
            &claimed_versions,
            &[],
        )
        .unwrap_or_else(|refusal| panic!("两个上限抬过之后这张给定状态表建得出判表：{refusal:?}"));
    let writes_on_the_first_system_configuration_slot = tables
        .locations
        .iter()
        .find(|location| location.device == 0 && location.offset == 0)
        .expect("盘 0 的系统配置槽 0 号是判表里的一个落点")
        .writes
        .len();
    assert!(
        usize::try_from(tables.enumerated_write_count).expect("装得进 usize")
            > ENUMERATED_WRITES_THE_MASK_HELD_BEFORE_THE_RAISE
            && writes_on_the_first_system_configuration_slot
                > WRITES_PER_LOCATION_THE_COMBINATION_HELD_BEFORE_THE_RAISE,
        "前提：枚举写表 {} 次写多于抬之前的 {ENUMERATED_WRITES_THE_MASK_HELD_BEFORE_THE_RAISE}，盘 0 系统配置槽 0 号 \
         {writes_on_the_first_system_configuration_slot} 次写多于抬之前的 {WRITES_PER_LOCATION_THE_COMBINATION_HELD_BEFORE_THE_RAISE}",
        tables.enumerated_write_count
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(gpu) = judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
    else {
        return;
    };
    drop(tables);
    let Some(cpu) =
        cpu_conclusions_of_a_table_read_back_from_the_pairing_library(&RecordedGivenStatesTable {
            table_name: format!(
                "toward-the-allocation-record-wall/seed={SEED_OF_A_HISTORY_TOWARD_THE_ALLOCATION_RECORD_WALL}"
            ),
            base: history_after_the_make_filesystem.base_pool().clone(),
            writes: table_writes.to_vec(),
            segments: history_after_the_make_filesystem.segments().to_vec(),
            seams: Vec::new(),
            persisted_sets,
        })
    else {
        return;
    };
    assert_eq!(gpu.len(), cpu.len(), "两边判的状态数相同");
    let mismatches: Vec<String> = gpu
        .iter()
        .zip(&cpu)
        .enumerate()
        .filter_map(|(ordinal, (gpu_words, cpu_conclusion))| {
            let gpu_cross_check_words =
                cross_check_words_without_the_oracle(cross_check_words_of_gpu_verdict(gpu_words));
            (gpu_could_not_judge(gpu_words) || gpu_cross_check_words != cpu_conclusion.words)
                .then(|| {
                    format!(
                        "状态 {ordinal}：GPU 判不了 {}（恢复第 1 字 {:#x}、池级 checker 原因字 {:#x}）；GPU {gpu_cross_check_words:?}；CPU {:?}",
                        gpu_could_not_judge(gpu_words),
                        gpu_words[1],
                        gpu_words[POOL_CHECKER_UNDECIDABLE_WORD],
                        cpu_conclusion.words
                    )
                })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} 个状态 GPU 判不了或与 CPU 不一致（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
