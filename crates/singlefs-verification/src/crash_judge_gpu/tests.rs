use super::{
    checker_scan_kernel_source, checker_walk_kernel_source, dispatch_words,
    pool_checker_violated_bits, recovery_kernel_source, verdict_was_written_by_both_kernels,
    GPU_JUDGE_CHECKER_COMMON_SOURCE, POOL_CHECKER_VIOLATED_FIRST_WORD, POOL_CHECKER_VIOLATED_WORDS,
    SCRATCH_WORDS, VERDICT_WORDS,
};

/// 两个内核的源码先经 naga 解析与校验（毫秒级），不等 GPU 编译：内核改坏一个字这里当场红，错误带行号。
fn parse_and_validate(name: &str, source: &str) {
    let module = match wgpu::naga::front::wgsl::parse_str(source) {
        Ok(module) => module,
        Err(error) => panic!("{name} 解析失败：\n{}", error.emit_to_string(source)),
    };
    let mut validator = wgpu::naga::valid::Validator::new(
        wgpu::naga::valid::ValidationFlags::all(),
        wgpu::naga::valid::Capabilities::all(),
    );
    if let Err(error) = validator.validate(&module) {
        panic!("{name} 校验失败：\n{}", error.emit_to_string(source));
    }
}

#[test]
fn recovery_kernel_source_parses_and_validates() {
    parse_and_validate("恢复内核", &recovery_kernel_source());
}

#[test]
fn checker_walk_kernel_source_parses_and_validates() {
    parse_and_validate("池级 checker 走读内核", &checker_walk_kernel_source());
}

#[test]
fn checker_scan_kernel_source_parses_and_validates() {
    parse_and_validate("池级 checker 扫描内核", &checker_scan_kernel_source());
}

#[test]
fn persisted_mask_probe_kernel_source_parses_and_validates() {
    parse_and_validate(
        "持久掩码探针内核",
        &super::persisted_mask_probe_kernel_source(),
    );
}

/// 两个检查内核共用那一半里每一个 `const INVARIANT_I_<类>_<号>: u32 = <下标>u;`，下标处的那一条就是 `I-<类>.<号>`（两个内核照这几个常量置违例位与判到位；
/// 下标写错、清单里插进一条而常量没跟上，这里红）；里程碑四自建 GC 那五条都在里面。
#[test]
fn every_invariant_index_written_into_the_checker_kernels_names_that_invariant_in_the_pool_checker_list(
) {
    let mut constants_checked = Vec::new();
    for line in super::GPU_JUDGE_CHECKER_COMMON_SOURCE.lines() {
        let Some(rest) = line.strip_prefix("const INVARIANT_I_") else {
            continue;
        };
        let (name_part, index_part) = rest
            .split_once(": u32 = ")
            .unwrap_or_else(|| panic!("常量行的形态是 `const INVARIANT_I_x_y: u32 = Nu;`：{line}"));
        let invariant = format!("I-{}", name_part.replacen('_', ".", 1));
        let index: usize = index_part
            .strip_suffix("u;")
            .and_then(|digits| digits.parse().ok())
            .unwrap_or_else(|| panic!("下标写成十进制加 u：{line}"));
        assert_eq!(
            super::IMPLEMENTED_INVARIANTS.get(index).copied(),
            Some(invariant.as_str()),
            "内核常量 INVARIANT_I_{name_part} 的下标 {index} 处不是 {invariant}"
        );
        constants_checked.push(invariant);
    }
    for (invariant, index) in
        super::KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES
            .into_iter()
            .chain(super::KERNEL_INVARIANT_INDEXES_OF_THE_TRUNCATION_JUDGES)
            .chain(super::KERNEL_INVARIANT_INDEX_OF_THE_SCAN_REBUILD_JUDGE)
            .chain(super::KERNEL_INVARIANT_INDEX_OF_THE_UPPER_SHAPE_JUDGE)
            .chain(super::KERNEL_INVARIANT_INDEXES_OF_THE_WRITER_IDENTITY_AND_ALLOCATION_RECORD_SPAN_JUDGES)
            .chain(super::KERNEL_INVARIANT_INDEX_OF_THE_JOURNAL_RECORD_RING_SLOT_JUDGE)
    {
        assert!(
            constants_checked.iter().any(|checked| checked == invariant),
            "内核里有 {invariant}（下标 {index}）那个常量"
        );
    }
}

/// 结论的每个字号互不相同、都在 VERDICT_WORDS 之内，每个字都有用途（放量流水线的 `crate::crash_amplification::gpu_recovery_words` 按具名字号取）。
#[test]
fn every_verdict_word_index_is_distinct_and_within_the_verdict_words() {
    let mut indexes = vec![
        0,
        1,
        super::CONSULTED_PASS_LANDED_INSTANCE_WORD,
        super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD,
        super::CONSULTED_PASS_OUTCOME_WORD,
        super::CONSULTED_PASS_FAILURE_POINT_WORD,
        super::IGNORED_PASS_OUTCOME_WORD,
        super::IGNORED_PASS_FAILURE_POINT_WORD,
        super::POOL_CHECKER_REACHED_WORD,
        super::POOL_CHECKER_UNDECIDABLE_WORD,
        super::IGNORED_PASS_LANDED_INSTANCE_WORD,
        super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD,
        super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD,
        super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD,
        super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD,
        super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD,
        super::SCAN_REBUILD_OUTCOME_WORD,
        super::SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD,
        super::CONSULTED_PASS_LANDED_MARKER_WORD,
        super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD,
        super::IGNORED_PASS_LANDED_MARKER_WORD,
        super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD,
    ];
    indexes.extend(
        POOL_CHECKER_VIOLATED_FIRST_WORD
            ..POOL_CHECKER_VIOLATED_FIRST_WORD + POOL_CHECKER_VIOLATED_WORDS,
    );
    indexes.extend(
        super::POOL_CHECKER_JUDGED_FIRST_WORD
            ..super::POOL_CHECKER_JUDGED_FIRST_WORD + super::POOL_CHECKER_JUDGED_WORDS,
    );
    let count = indexes.len();
    indexes.sort_unstable();
    indexes.dedup();
    assert_eq!(indexes.len(), count, "字号不重");
    assert_eq!(
        indexes,
        (0..VERDICT_WORDS).collect::<Vec<_>>(),
        "每个字都有用途、不越界"
    );
}

/// 打包表的表号常量就是 every_table_name 里的次序（名字大写、加 T_），表数与段表宽也生成进去。
#[test]
fn the_table_numbers_in_the_kernel_follow_every_table_name() {
    let constants = super::judge_table_layout_wgsl_constants();
    let names = crate::crash_judge_tables::JudgeTablesWords::every_table_name();
    for (index, name) in names.iter().enumerate() {
        let line = format!("const T_{}: u32 = {index}u;", name.to_ascii_uppercase());
        assert_eq!(constants.matches(&line).count(), 1, "恰好一行 {line}");
    }
    assert!(constants.contains(&format!("const TABLE_COUNT: u32 = {}u;", names.len())));
    assert!(constants.contains(&format!(
        "const SEGMENT_WORDS: u32 = {}u;",
        crate::crash_judge_tables::SEGMENT_WORDS
    )));
}

/// 扫描重建的结局码一种一个码、互不相同、不撞没走到的那个初值；内核里每个码都有一行常量；CPU 池级 checker 判 I-1.13 的每一种
/// （成立、违例、表里每一句不适用的理由原文）换回表里那一行的码，判定里没有 I-1.13 换成没走到，表外的理由与部分没判报错。
#[test]
fn every_scan_rebuild_outcome_has_its_own_code_generated_into_the_kernels_and_read_back_from_the_pool_checker(
) {
    use singlefs_checker::image::{InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX};
    let mut codes: Vec<u32> = super::SCAN_REBUILD_OUTCOMES
        .iter()
        .map(|outcome| outcome.code)
        .collect();
    codes.push(super::SCAN_REBUILD_OUTCOME_NOT_REACHED);
    let count = codes.len();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), count, "结局码不重，也不撞没走到的初值");
    let constants = super::scan_rebuild_outcome_wgsl_constants();
    for outcome in super::SCAN_REBUILD_OUTCOMES {
        let line = format!("const {}: u32 = {}u;", outcome.constant_name, outcome.code);
        assert_eq!(constants.matches(&line).count(), 1, "恰好一行 {line}");
        let verdict = match outcome.verdict_of_the_pool_checker {
            super::ScanRebuildVerdictOfThePoolChecker::Holds => InvariantVerdict::Holds,
            super::ScanRebuildVerdictOfThePoolChecker::Violated => {
                InvariantVerdict::Violated("扫描重建的三张表与遍历方向的不同".to_string())
            }
            super::ScanRebuildVerdictOfThePoolChecker::NotApplicable(reason) => {
                InvariantVerdict::NotApplicable(reason)
            }
        };
        assert_eq!(
            super::scan_rebuild_outcome_of_the_pool_checker_verdicts(&[(
                SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
                verdict
            )]),
            Ok(outcome.code),
            "{} 换回自己的码",
            outcome.constant_name
        );
    }
    assert_eq!(
        super::scan_rebuild_outcome_of_the_pool_checker_verdicts(&[]),
        Ok(super::SCAN_REBUILD_OUTCOME_NOT_REACHED),
        "判定里没有 I-1.13：池级 checker 没走到它"
    );
    assert!(
        super::scan_rebuild_outcome_of_the_pool_checker_verdicts(&[(
            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
            InvariantVerdict::NotApplicable("表外的一句理由")
        )])
        .is_err(),
        "表外的理由认不得"
    );
    assert!(
        super::scan_rebuild_outcome_of_the_pool_checker_verdicts(&[(
            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
            InvariantVerdict::PartiallyJudged("部分没判".to_string())
        )])
        .is_err(),
        "扫描重建没有部分没判这一种"
    );
    assert!(
        super::kernel_prelude_wgsl_constants().contains(&constants),
        "结局码接在每个内核源码最前面的常量里"
    );
    // 每一种结局都有内核写它：池级 checker 走到扫描重建之前整池交回的四种由走读内核在那几处早退写，别的由扫描重建那一截写
    const WRITTEN_BY_THE_WALK_KERNEL: [&str; 4] = [
        "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_A_SYSTEM_CONFIGURATION_VALUE_THE_READER_DOES_NOT_ACCEPT",
        "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_UNEQUAL_JOURNAL_RING_GEOMETRY_FIELDS",
        "SCAN_REBUILD_OUTCOME_NO_DEVICE_YIELDS_A_VALID_SYSTEM_CONFIGURATION",
        "SCAN_REBUILD_OUTCOME_ROOT_RING_HOLDS_NO_SELF_VERIFIED_ROOT",
    ];
    for outcome in super::SCAN_REBUILD_OUTCOMES {
        let is_a_whole_pool_return = WRITTEN_BY_THE_WALK_KERNEL.contains(&outcome.constant_name);
        let (kernel, kernel_source) = if is_a_whole_pool_return {
            (
                "走读内核",
                super::joined_wgsl_source(&super::GPU_JUDGE_CHECKER_SOURCE_FILES),
            )
        } else {
            (
                "扫描重建那一截",
                super::GPU_JUDGE_CHECKER_SCAN_REBUILD_SOURCE.to_string(),
            )
        };
        assert!(
            kernel_source.contains(&format!("{};", outcome.constant_name))
                || kernel_source.contains(&format!("{},", outcome.constant_name))
                || kernel_source.contains(&format!("{})", outcome.constant_name)),
            "{} 由{kernel}写",
            outcome.constant_name
        );
    }
}

/// 失败点码一码一个成员、互不相同；内核里每个码都有一行常量（名字去掉 GPU_）。
#[test]
fn every_failure_point_code_is_distinct_and_generated_into_the_kernels() {
    let mut codes: Vec<u32> = super::GPU_FAILURE_POINTS
        .iter()
        .map(|(_, code)| *code)
        .collect();
    let count = codes.len();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), count, "码不重");
    let constants = super::failure_point_wgsl_constants();
    for (name, code) in super::GPU_FAILURE_POINTS {
        let line = format!(
            "const {}: u32 = {code}u;",
            name.strip_prefix("GPU_").expect("GPU_ 起头")
        );
        assert_eq!(constants.matches(&line).count(), 1, "恰好一行 {line}");
    }
    assert!(super::kernel_prelude_wgsl_constants().contains(&constants));
}

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::recovery::{
    RecoveryFailure, SystemConfigurationValueOutsideWhatThisReaderAccepts,
};

// 一个成员一条：`gpu_failure_point_code` 把这个成员映到它那一码（成员带的细节不进码）。
#[test]
fn no_valid_system_configuration_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::NoValidSystemConfiguration {
            first_device_with_no_valid_system_configuration_slot: DeviceIdentity(1)
        }),
        super::GPU_FAILURE_POINT_NO_VALID_SYSTEM_CONFIGURATION
    );
}

#[test]
fn incompat_bits_not_recognized_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(
            &RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {
                first_device_carrying_them: DeviceIdentity(0),
                incompat_bitmap: singlefs_core::system_configuration::IncompatBitmap([1; 32])
            }
        ),
        super::GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED
    );
}

#[test]
fn system_configurations_disagree_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::SystemConfigurationsDisagree),
        super::GPU_FAILURE_POINT_SYSTEM_CONFIGURATIONS_DISAGREE
    );
}

#[test]
fn no_device_in_the_pool_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::NoDeviceInThePool),
        super::GPU_FAILURE_POINT_NO_DEVICE_IN_THE_POOL
    );
}

#[test]
fn root_ring_slots_per_region_out_of_range_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::RootRingSlotsPerRegionOutOfRange {
            device: DeviceIdentity(0),
            out_of_range: singlefs_core::root_ring::RootRingSlotsPerRegionOutOfRange {
                declared_slots_per_region: 3,
                minimum: 4,
                maximum: 16
            }
        }),
        super::GPU_FAILURE_POINT_ROOT_RING_SLOTS_PER_REGION_OUT_OF_RANGE
    );
}

#[test]
fn system_configuration_value_refused_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::SystemConfigurationValueRefused {
            device: DeviceIdentity(0),
            value: SystemConfigurationValueOutsideWhatThisReaderAccepts::EncryptionTypeNotOff {
                encryption_type: 1
            }
        }),
        super::GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED
    );
}

#[test]
fn device_ends_before_the_unit_area_start_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
            device: DeviceIdentity(0),
            device_bytes: 4096
        }),
        super::GPU_FAILURE_POINT_DEVICE_ENDS_BEFORE_THE_UNIT_AREA_START
    );
}

#[test]
fn no_valid_root_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::NoValidRoot),
        super::GPU_FAILURE_POINT_NO_VALID_ROOT
    );
}

#[test]
fn unit_unreadable_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::UnitUnreadable {
            slot: SlotNumber(7)
        }),
        super::GPU_FAILURE_POINT_UNIT_UNREADABLE
    );
}

#[test]
fn unit_malformed_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::UnitMalformed {
            what: "树表单元"
        }),
        super::GPU_FAILURE_POINT_UNIT_MALFORMED
    );
}

#[test]
fn invariant_violated_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::InvariantViolated {
            invariant: "I-9.16",
            detail: "树表条目不按树 ID 严格升序"
        }),
        super::GPU_FAILURE_POINT_INVARIANT_VIOLATED
    );
}

#[test]
fn entry_narrower_than_its_field_table_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
            what: "记账条目",
            entry_bytes: 20,
            field_table_bytes: 34
        }),
        super::GPU_FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE
    );
}

#[test]
fn inode_number_watermark_row_missing_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(
            &RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree
        ),
        super::GPU_FAILURE_POINT_INODE_NUMBER_WATERMARK_ROW_MISSING_FROM_THE_ACCOUNTING_TREE
    );
}

#[test]
fn allocation_record_outside_the_pool_geometry_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
            what: "分配记录的设备身份不在池里"
        }),
        super::GPU_FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY
    );
}

#[test]
fn mapping_miss_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::MappingMiss {
            slot: SlotNumber(9)
        }),
        super::GPU_FAILURE_POINT_MAPPING_MISS
    );
}

#[test]
fn mapping_still_unreadable_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::MappingStillUnreadable {
            slot: SlotNumber(9)
        }),
        super::GPU_FAILURE_POINT_MAPPING_STILL_UNREADABLE
    );
}

#[test]
fn root_publish_with_an_undecided_anchor_has_its_failure_point() {
    assert_eq!(
        super::gpu_failure_point_code(&RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
instance: InstanceGeneration(1),
checkpoint_txg: CheckpointTxg(3),
counters: vec![4, 5]
}),
        super::GPU_FAILURE_POINT_ROOT_PUBLISH_CARRIES_MORE_THAN_ONE_LAST_RECORD_FLAG_WHOSE_ANCHOR_IS_UNDECIDED
    );
}

/// GPU 结论字换成对拍字：红位去掉判不了那一位、两遍的根（标记、实例代号、checkpoint_txg 低 32 位与高 32 位，高 32 位取接在末尾的那两个字）
/// 与结局与失败点各取各的字、违例位与判到位各读四个字、I-1.13 的结局码取第 24 字；
/// 判不了两种来路都认。
#[test]
fn the_cross_check_words_of_a_gpu_verdict_take_each_field_from_its_word() {
    let mut words = [0u32; VERDICT_WORDS];
    words[0] = 1 | (1 << super::verdict_bits::UNDECIDABLE);
    words[super::CONSULTED_PASS_LANDED_INSTANCE_WORD] = 7;
    words[super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = 9;
    words[super::CONSULTED_PASS_OUTCOME_WORD] = super::RECOVERY_OUTCOME_CODE_FAILED;
    words[super::CONSULTED_PASS_FAILURE_POINT_WORD] = super::GPU_FAILURE_POINT_MAPPING_MISS;
    words[super::IGNORED_PASS_OUTCOME_WORD] = super::RECOVERY_OUTCOME_CODE_FILE_READ;
    words[super::IGNORED_PASS_LANDED_INSTANCE_WORD] = 6;
    words[super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = 8;
    words[super::CONSULTED_PASS_LANDED_MARKER_WORD] = super::LandedRootWords::LANDED_ON_A_ROOT;
    words[super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = 5;
    words[super::IGNORED_PASS_LANDED_MARKER_WORD] = super::LandedRootWords::LANDED_ON_A_ROOT;
    words[super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = 0xFFFF_FFFF;
    words[POOL_CHECKER_VIOLATED_FIRST_WORD + 3] = 1 << 31;
    words[super::POOL_CHECKER_JUDGED_FIRST_WORD + 2] = 1;
    words[super::SCAN_REBUILD_OUTCOME_WORD] = 25;
    let cross = super::cross_check_words_of_gpu_verdict(&words);
    assert_eq!(
        cross,
        super::CrossCheckWords {
            red_bits: 1,
            pool_checker_violated_bits: 1u128 << 127,
            consulted_pass_landed_root: super::LandedRootWords::of_the_landed_root(
                InstanceGeneration(7),
                CheckpointTxg((5 << 32) | 9)
            ),
            ignored_pass_landed_root: super::LandedRootWords::of_the_landed_root(
                InstanceGeneration(6),
                CheckpointTxg((0xFFFF_FFFF << 32) | 8)
            ),
            consulted_pass_outcome: super::RECOVERY_OUTCOME_CODE_FAILED,
            ignored_pass_outcome: super::RECOVERY_OUTCOME_CODE_FILE_READ,
            consulted_pass_failure_point: super::GPU_FAILURE_POINT_MAPPING_MISS,
            ignored_pass_failure_point: super::GPU_FAILURE_POINT_NONE,
            pool_checker_judged_bits: 1u128 << 64,
            scan_rebuild_outcome: 25,
            gpu_could_not_judge: true,
        }
    );
    let mut checker_gave_up = [0u32; VERDICT_WORDS];
    checker_gave_up[super::POOL_CHECKER_UNDECIDABLE_WORD] = 3;
    assert!(super::cross_check_words_of_gpu_verdict(&checker_gave_up).gpu_could_not_judge);
    assert!(!super::cross_check_words_of_gpu_verdict(&[0u32; VERDICT_WORDS]).gpu_could_not_judge);
}

/// 审计复核 T01（checkpoint_txg「存完整 u64」）：一份手造的结论字，看 journal 那一遍落到实例 7、checkpoint_txg
/// `(5 << 32) | 9` 那条根（高 32 位那一字非 0）、读到文件，不看 journal 那一遍没落到根、失败；GPU 一侧换出的恢复摘要与 CPU 一侧
/// （`crate::crash_amplification::recovery_words_of`）对同一个根、同两种结局换出的那一份逐字段相同、摊出的字逐字相同。
/// 高 32 位那一字改成 0 之后两份不同：GPU 一侧真读了高 32 位那一字。
#[test]
fn the_gpu_recovery_summary_of_a_verdict_whose_checkpoint_txg_high_word_is_not_zero_equals_the_cpu_summary_of_the_same_root(
) {
    let landed_root = (InstanceGeneration(7), CheckpointTxg((5 << 32) | 9));
    let mut words = [0u32; VERDICT_WORDS];
    words[super::CONSULTED_PASS_LANDED_MARKER_WORD] = super::LandedRootWords::LANDED_ON_A_ROOT;
    words[super::CONSULTED_PASS_LANDED_INSTANCE_WORD] = 7;
    words[super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = 9;
    words[super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = 5;
    words[super::CONSULTED_PASS_OUTCOME_WORD] = super::RECOVERY_OUTCOME_CODE_FILE_READ;
    words[super::IGNORED_PASS_LANDED_MARKER_WORD] = super::LandedRootWords::LANDED_ON_NO_ROOT;
    words[super::IGNORED_PASS_LANDED_INSTANCE_WORD] = u32::MAX;
    words[super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = u32::MAX;
    words[super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = u32::MAX;
    words[super::IGNORED_PASS_OUTCOME_WORD] = super::RECOVERY_OUTCOME_CODE_FAILED;
    words[super::IGNORED_PASS_FAILURE_POINT_WORD] = super::GPU_FAILURE_POINT_NO_VALID_ROOT;
    let recovery_report = |outcome, effective_root| singlefs_core::recovery::RecoveryReport {
        outcome,
        effective_root,
        journal: singlefs_core::recovery::JournalScanReport::default(),
        mapping_fallbacks: 0,
        devices_without_a_mountable_system_configuration_slot: Vec::new(),
    };
    let judgement = crate::crash::CrashStateJudgement {
        consulted: recovery_report(
            singlefs_core::recovery::RecoveryOutcome::FileRead {
                root: landed_root,
                content: Vec::new(),
            },
            Some(landed_root),
        ),
        ignored: recovery_report(
            singlefs_core::recovery::RecoveryOutcome::Failed {
                root: None,
                failure: RecoveryFailure::NoValidRoot,
            },
            None,
        ),
        root_persisted: false,
        consulted_oracle: None,
        ignored_oracle: None,
        pool_checker: Vec::new(),
        records: crate::memory_pool::RecordCheck::default(),
    };
    let summary_on_the_cpu = crate::crash_amplification::recovery_words_of(&judgement);
    let summary_on_the_gpu = crate::crash_amplification::gpu_recovery_words(&words);
    assert_eq!(
        summary_on_the_gpu, summary_on_the_cpu,
        "同一个根（实例 7、checkpoint_txg 高 32 位 5、低 32 位 9）、同两种结局：GPU 与 CPU 换出的恢复摘要逐字段相同"
    );
    assert_eq!(
        summary_on_the_gpu.words(),
        summary_on_the_cpu.words(),
        "摊出的字逐字相同"
    );
    assert_eq!(
        summary_on_the_gpu.consulted_pass_landed_root.landed_root(),
        Some(landed_root),
        "GPU 一侧的摘要换得回完整的 checkpoint_txg"
    );
    let mut words_without_the_high_word = words;
    words_without_the_high_word[super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = 0;
    assert_ne!(
        crate::crash_amplification::gpu_recovery_words(&words_without_the_high_word),
        summary_on_the_cpu,
        "高 32 位那一字是 0 的结论落到的是 checkpoint_txg 9 那条根，与 CPU 那一份不同"
    );
}

/// 判不了那一句的原因：池级 checker 内核写的原因字拆成原因码与站点号（`UNDECIDABLE_TOO_MANY_ENTRIES | (13u << 8u)` 是原因码 3、站点 13），
/// 恢复内核判不了时取第 1 字；两个内核都判得了没有这一句（第 1 字照旧带重放停在哪一道，不当原因读）。
#[test]
fn the_undecidable_reasons_of_a_verdict_name_the_reason_code_and_the_site_of_each_kernel() {
    let mut words = [0u32; VERDICT_WORDS];
    words[1] = 0x0005_0002;
    assert_eq!(super::undecidable_reasons_of_the_gpu_verdict(&words), None);
    words[super::POOL_CHECKER_UNDECIDABLE_WORD] = 3 | (13 << super::UNDECIDABLE_SITE_SHIFT);
    assert_eq!(
        super::undecidable_reasons_of_the_gpu_verdict(&words).as_deref(),
        Some("池级 checker 内核原因字 0xd03（原因码 3、站点 13）")
    );
    words[0] = 1 << super::verdict_bits::UNDECIDABLE;
    words[1] = 2;
    assert_eq!(
        super::undecidable_reasons_of_the_gpu_verdict(&words).as_deref(),
        Some("恢复内核原因字 0x2（原因码 2、站点 0）；池级 checker 内核原因字 0xd03（原因码 3、站点 13）")
    );
}

/// 池级 checker 一份判定的违例位与判到位：违例与成立算判到，不适用与部分没判都不算。
#[test]
fn violated_and_judged_bits_count_holds_and_violations_as_judged_only() {
    use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
    let verdicts = vec![
        (IMPLEMENTED_INVARIANTS[0], InvariantVerdict::Holds),
        (
            IMPLEMENTED_INVARIANTS[1],
            InvariantVerdict::Violated("红".to_string()),
        ),
        (
            IMPLEMENTED_INVARIANTS[2],
            InvariantVerdict::NotApplicable("没有对象"),
        ),
        (
            IMPLEMENTED_INVARIANTS[3],
            InvariantVerdict::PartiallyJudged("一部分".to_string()),
        ),
    ];
    assert_eq!(
        super::pool_checker_violated_and_judged_bits(&verdicts),
        (0b10, 0b11)
    );
}

/// 派活给每个状态的草稿区字数与内核里草稿区的末端（`SCRATCH_END`）相同：内核加了表而这里没跟上，每个状态的表就压到下一个状态的草稿区上。
#[test]
fn scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels() {
    let declaration = format!("const SCRATCH_END: u32 = {SCRATCH_WORDS}u;");
    assert_eq!(
        GPU_JUDGE_CHECKER_COMMON_SOURCE
            .matches(&declaration)
            .count(),
        1,
        "池级 checker 共用那一半里恰好一行 `{declaration}`"
    );
}

/// 派活参数四个字各是什么：序号拆成低、高 32 位，状态数，每个状态的草稿区字数。
#[test]
fn dispatch_words_carry_the_first_ordinal_the_state_count_and_the_scratch_size() {
    assert_eq!(
        dispatch_words(0x0000_0002_8000_0001, 2170),
        [
            0x8000_0001,
            2,
            2170,
            u32::try_from(SCRATCH_WORDS).expect("装得进 u32")
        ]
    );
}

/// 违例位的每个字都读进来：第 64 位（第三个字的最低位）与第 127 位（第四个字的最高位）读得出，不被截成 64 位。
#[test]
fn violated_bits_past_the_sixty_fourth_are_read_from_the_third_and_fourth_words() {
    let mut words = [0u32; VERDICT_WORDS];
    words[POOL_CHECKER_VIOLATED_FIRST_WORD] = 1;
    words[POOL_CHECKER_VIOLATED_FIRST_WORD + 2] = 1;
    words[POOL_CHECKER_VIOLATED_FIRST_WORD + POOL_CHECKER_VIOLATED_WORDS - 1] = 1 << 31;
    assert_eq!(
        pool_checker_violated_bits(&words),
        1u128 | (1u128 << 64) | (1u128 << 127),
        "位 0、64、127 各在第一、三、四个违例字里"
    );
    words[super::POOL_CHECKER_REACHED_WORD] = u32::MAX;
    words[super::POOL_CHECKER_UNDECIDABLE_WORD] = u32::MAX;
    assert_eq!(
        pool_checker_violated_bits(&words),
        1u128 | (1u128 << 64) | (1u128 << 127),
        "走到第几步与判不了的原因那两个字不算进违例位"
    );
}

/// 结论里 I-7.4 点名的被抛弃根：点名那一字是 0 交空；是 1 交一条，checkpoint_txg 由低、高 32 位两个字拼回（高位那一字不丢）。
#[test]
fn the_abandoned_root_named_in_a_gpu_verdict_takes_its_instance_and_both_halves_of_its_txg() {
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    let mut words = [0u32; VERDICT_WORDS];
    assert_eq!(
        super::abandoned_roots_named_by_the_pool_checker_of_the_gpu_verdict(&words),
        Vec::new(),
        "点名那一字是 0：没点名"
    );
    words[super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD] = super::ABANDONED_ROOT_NAMED;
    words[super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD] = 3;
    words[super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD] = 7;
    words[super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD] = 2;
    assert_eq!(
        super::abandoned_roots_named_by_the_pool_checker_of_the_gpu_verdict(&words),
        [
            crate::crash_judge_conclusions::AbandonedRootNamedByThePoolChecker {
                instance: InstanceGeneration(3),
                checkpoint_txg: CheckpointTxg((2u64 << 32) | 7),
            }
        ]
    );
}

/// 结论还是初值（全 1）的状态认得出；两个内核各写过自己那几个字的才算写过。
#[test]
fn verdict_still_at_its_initial_value_is_not_taken_as_written() {
    let untouched = [u32::MAX; VERDICT_WORDS];
    assert!(!verdict_was_written_by_both_kernels(&untouched));
    let mut only_recovery = untouched;
    only_recovery[0] = 0;
    assert!(!verdict_was_written_by_both_kernels(&only_recovery));
    let mut both = only_recovery;
    both[super::POOL_CHECKER_REACHED_WORD] = 5;
    assert!(verdict_was_written_by_both_kernels(&both));
}

#[test]
fn facts_verifier_kernel_source_parses_and_validates() {
    parse_and_validate(
        "第 ② 段核对内核",
        &crate::crash_verify_gpu::verifier_shader_source(),
    );
}

/// 解析一份内核源码（先经 [`parse_and_validate`] 核过）。
fn parsed_module(name: &str, source: &str) -> wgpu::naga::Module {
    parse_and_validate(name, source);
    wgpu::naga::front::wgsl::parse_str(source).expect("上面解析过")
}

/// 内核里名叫 `name` 的模块常量的 u32 值（初值是字面量，或顺着别的常量指到字面量）。
fn value_of_the_u32_constant(module: &wgpu::naga::Module, name: &str) -> u32 {
    use wgpu::naga::{Expression, Literal};
    let (_, constant) = module
        .constants
        .iter()
        .find(|(_, constant)| constant.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("内核里没有常量 {name}"));
    let mut initializer = constant.init;
    // 迭代上界：模块常量的个数（每一步换到另一个常量的初值）
    for _step in 0..=module.constants.len() {
        if let Expression::Literal(Literal::U32(value)) = module.global_expressions[initializer] {
            return value;
        }
        let Expression::Constant(other) = module.global_expressions[initializer] else {
            panic!(
                "常量 {name} 的初值既不是 u32 字面量也不是别的常量：{:?}",
                module.global_expressions[initializer]
            );
        };
        initializer = module.constants[other].init;
    }
    panic!(
        "常量 {name} 顺着指了 {} 步还没到字面量",
        module.constants.len() + 1
    );
}

/// 内核里名叫 `name` 的 `var<private>` 是几个元素的定长数组。
fn element_count_of_the_private_array(module: &wgpu::naga::Module, name: &str) -> u32 {
    use wgpu::naga::{ArraySize, TypeInner};
    let (_, variable) = module
        .global_variables
        .iter()
        .find(|(_, variable)| variable.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("内核里没有全局变量 {name}"));
    let TypeInner::Array {
        size: ArraySize::Constant(element_count),
        ..
    } = module.types[variable.ty].inner
    else {
        panic!("{name} 不是定长数组：{:?}", module.types[variable.ty].inner);
    };
    element_count.get()
}

/// 持久掩码的宽与组合掩码的布局在内核里只由 Rust 常量生成（`code-discipline.md`「魔法数字写成命名常量」：一个数一处定义）：
/// 三个判器内核、探针内核与第 ② 段核对内核里 `persisted` 那个数组的元素数就是 [`crate::crash_facts::PERSISTED_MASK_WORDS`]
/// （[`crate::crash_facts::MAXIMUM_ENUMERATED_WRITES`] ÷ 32），判器那几个内核里版本表的宽、组合掩码的字数与组合高位那几字的起点
/// 就是 `crate::crash_judge_tables` 的同名常量；手写的 WGSL 里不另写这几个常量，也不留 64 个字的掩码。
/// 判别力：WGSL 里把 `persisted` 写回 `array<u32, 64>`、或手写一份 `VERSION_WORDS`，这条红（后者 naga 先报重名）。
#[test]
fn the_mask_and_combination_widths_in_every_kernel_are_generated_from_the_rust_constants() {
    use crate::crash_facts::PERSISTED_MASK_WORDS;
    use crate::crash_judge_tables::{
        COMBINATION_WORDS_OF_A_VERSION, VERSION_WORDS,
        VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE,
    };
    let as_u32 = |value: usize| u32::try_from(value).expect("常量装得进 u32");
    let judge_kernels = [
        ("恢复内核", recovery_kernel_source()),
        ("池级 checker 走读内核", checker_walk_kernel_source()),
        ("池级 checker 扫描内核", checker_scan_kernel_source()),
        (
            "持久掩码探针内核",
            super::persisted_mask_probe_kernel_source(),
        ),
    ];
    for (name, source) in &judge_kernels {
        let module = parsed_module(name, source);
        assert_eq!(
            element_count_of_the_private_array(&module, "persisted"),
            as_u32(PERSISTED_MASK_WORDS),
            "{name} 的持久掩码"
        );
        for (constant, value) in [
            ("PERSISTED_MASK_WORDS", PERSISTED_MASK_WORDS),
            ("VERSION_WORDS", VERSION_WORDS),
            (
                "COMBINATION_WORDS_OF_A_VERSION",
                COMBINATION_WORDS_OF_A_VERSION,
            ),
            (
                "VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE",
                VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE,
            ),
        ] {
            assert_eq!(
                value_of_the_u32_constant(&module, constant),
                as_u32(value),
                "{name} 的 {constant}"
            );
        }
    }
    let verifier = parsed_module(
        "第 ② 段核对内核",
        &crate::crash_verify_gpu::verifier_shader_source(),
    );
    for private_array in ["persisted", "torn"] {
        assert_eq!(
            element_count_of_the_private_array(&verifier, private_array),
            as_u32(PERSISTED_MASK_WORDS),
            "第 ② 段核对内核的 {private_array}"
        );
    }
    for (name, handwritten) in super::GPU_JUDGE_COMMON_SOURCE_FILES.into_iter().chain([(
        "第 ② 段核对内核的本体",
        crate::crash_verify_gpu::GPU_VERIFIER_SHADER_SOURCE,
    )]) {
        for generated in [
            "const PERSISTED_MASK_WORDS",
            "const VERSION_WORDS",
            "const COMBINATION_WORDS_OF_A_VERSION",
            "const VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE",
            "array<u32, 64>",
        ] {
            assert!(
                !handwritten.contains(generated),
                "{name} 里手写了 `{generated}`：这个数由 Rust 常量生成"
            );
        }
    }
}

/// 走读内核 I-3.5 每一版引用的落点那张表（`C_APPEARANCE_MAX_VERSION_PLACEMENTS`）取结构上界：版本上限（`C_APPEARANCE_MAX_VERSIONS`，
/// 满了站点 43）乘每版落点上限（`C_MAX_IN_VERSION`，满了站点 4）。那两处先判不了，这张表满了（站点 44 的容量那一半）在每个输入上都走不到。
/// 判别力：这张表改回 8192 个字（整理夹着回退那一组比重交判的 17 张镜像里两张各版合计 19 916、18 664 个，在站点 44 判不了的那个界）这条红。
#[test]
fn the_appearance_version_placements_hold_the_most_versions_times_the_most_placements_of_a_version()
{
    let module = parsed_module("池级 checker 走读内核", &checker_walk_kernel_source());
    let most_versions = value_of_the_u32_constant(&module, "C_APPEARANCE_MAX_VERSIONS");
    let most_placements_of_a_version = value_of_the_u32_constant(&module, "C_MAX_IN_VERSION");
    assert_eq!(
        value_of_the_u32_constant(&module, "C_APPEARANCE_MAX_VERSION_PLACEMENTS"),
        most_versions
            .checked_mul(most_placements_of_a_version)
            .expect("64 版乘每版 2048 个装得进 u32"),
        "每一版引用的落点那张表的条数是版本上限 {most_versions} 乘每版落点上限 {most_placements_of_a_version}"
    );
}

/// 扫描内核 I-1.8 收的扫描项（`C_MAX_SCANNED`，站点 36）与扫描重建那一遍（`SCAN_REBUILD_MAX_UNITS`，站点 110）至少要装下这么多项：
/// 整理夹着回退那一组比重偏移 0–3、40 步交判的 17 张镜像里两张 2148、3030 项（CPU 一侧数的，在 2048 的界上判不了），
/// 同比重 259 张（加种子到偏移 19、拉长到 160 步）至多 3030 项；4096 是 I-1.8 借的次序表（`C_ROW_ORDER`，`C_MAX_REFERENCES` 个字）的宽。
const SCANNED_UNITS_THE_SCAN_KERNEL_HOLDS_AT_LEAST: u32 = 4096;

/// 扫描项那张表至少 [`SCANNED_UNITS_THE_SCAN_KERNEL_HOLDS_AT_LEAST`] 项，扫描重建那一遍与它同宽（收法相同，一边装得下另一边也装得下）。
/// 判别力：`C_MAX_SCANNED` 改回 2048 这条红。
#[test]
fn the_scan_and_the_scan_rebuild_hold_the_scanned_units_of_the_compaction_and_rollback_histories() {
    let module = parsed_module("池级 checker 扫描内核", &checker_scan_kernel_source());
    let scanned_units = value_of_the_u32_constant(&module, "C_MAX_SCANNED");
    assert!(
        scanned_units >= SCANNED_UNITS_THE_SCAN_KERNEL_HOLDS_AT_LEAST,
        "扫描项那张表 {scanned_units} 项，至少要 {SCANNED_UNITS_THE_SCAN_KERNEL_HOLDS_AT_LEAST} 项"
    );
    assert_eq!(
        value_of_the_u32_constant(&module, "SCAN_REBUILD_MAX_UNITS"),
        scanned_units,
        "扫描重建收扫描项那张表与 I-1.8 那一张同宽"
    );
}

/// 恢复内核一版的分配记录（`MAX_ALLOCATION_RECORDS`，满了记条目过多）与扫描内核一条根的分配记录（`C_MAX_ALLOC_RECORDS`，满了记站点 20）
/// 至少要装下这么多条：整理夹着回退那一组比重 40 步交判的 17 张镜像每版至多 4018 条，离 4096 只差 78；同比重拉到 80 步那一组 45 张里 11 张、
/// 160 步那一组 105 张里 27 张多于 4096 条，至多 4728 条（CPU 一侧数的，C894）。
const ALLOCATION_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST: u32 = 8192;

/// 两张分配记录表各至少 [`ALLOCATION_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST`] 条。恢复内核先解析、先判：
/// 判别力：恢复内核那张改回 4096 这条红在恢复内核那一句（恢复内核里没有草稿区末端那条 `const_assert`，照样解析得过；
/// 它挪了 `R_ROWS` 之后的偏移链，扫描内核随后解析也不过）；扫描内核那张改回 4096 时草稿区末端那条 `const_assert` 先不成立，红在解析扫描内核。
#[test]
fn both_allocation_record_tables_hold_the_records_of_a_version_of_the_longer_compaction_and_rollback_histories(
) {
    for (kernel, source, table_constant) in [
        (
            "恢复内核",
            recovery_kernel_source(),
            "MAX_ALLOCATION_RECORDS",
        ),
        (
            "池级 checker 扫描内核",
            checker_scan_kernel_source(),
            "C_MAX_ALLOC_RECORDS",
        ),
    ] {
        let module = parsed_module(kernel, &source);
        let records = value_of_the_u32_constant(&module, table_constant);
        assert!(
            records >= ALLOCATION_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST,
            "{kernel} 的分配记录表 {table_constant} {records} 条，至少要 {ALLOCATION_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST} 条"
        );
    }
}

/// 两个内核收环里 journal 记录的那张表（`crash_judge_common.wgsl` 的 `MAX_RECORDS`，恢复内核、走读内核满了记判不了、原因 2）至少要装下这么多条：
/// 一版分配记录过 8192 条的那条流（新池新建文件之后顺序写 17 次各 200 个单元、再 201 个、再 144 个，被判那次缩回 1 个，被判那一版 8194 条）
/// 盘 0 上 3749 条记录写（CPU 一侧量的，`crash_judge_gpu_tables_at_their_capacity.rs` 里 8194 条那条用例照这条流现数、断言不多于 4096）。
const JOURNAL_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST: u32 = 3749;

/// 三个判器内核的 journal 记录表各至少 [`JOURNAL_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST`] 条（三份都由共用那一半定义，恢复内核先解析、先判）。
/// 判别力：`MAX_RECORDS` 改回 2048 这条红在恢复内核那一句（恢复内核照样解析得过；走读与扫描两个内核里草稿区末端那条 `const_assert` 不成立）。
#[test]
fn every_judge_kernel_holds_the_journal_records_of_the_flow_past_eight_thousand_one_hundred_and_ninety_two_allocation_records(
) {
    for (kernel, source) in [
        ("恢复内核", recovery_kernel_source()),
        ("池级 checker 走读内核", checker_walk_kernel_source()),
        ("池级 checker 扫描内核", checker_scan_kernel_source()),
    ] {
        let module = parsed_module(kernel, &source);
        let records = value_of_the_u32_constant(&module, "MAX_RECORDS");
        assert!(
            records >= JOURNAL_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST,
            "{kernel} 的 journal 记录表 MAX_RECORDS {records} 条，至少要 {JOURNAL_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST} 条"
        );
    }
}

/// 借草稿区一段的一处：说给人看的名字、它起在哪、至多占几个字（都是扫描内核里的常量名；每个字一条）。
struct ScratchSegmentBorrower {
    name: &'static str,
    start_constant: &'static str,
    word_count_constant: &'static str,
}

/// 草稿区里几处先后借的一段：名字、起点与末端（段之后第一个字）的常量名、借它的那几处。
struct BorrowedScratchSegment {
    name: &'static str,
    start_constant: &'static str,
    end_constant: &'static str,
    borrowers: &'static [ScratchSegmentBorrower],
}

/// 扫描内核里轮着借的四段（`crash_judge_checker_common.wgsl` 的 ⑨、⑮ 与 ⑯）与借它们的每一处。
const BORROWED_SCRATCH_SEGMENTS_OF_THE_SCAN_KERNEL: [BorrowedScratchSegment; 4] = [
    BorrowedScratchSegment {
        name: "⑨ 行号次序",
        start_constant: "C_ROW_ORDER",
        end_constant: "C_ROW_ORDER_END",
        borrowers: &[
            ScratchSegmentBorrower {
                name: "走读内核逐盘 journal 那几条的组",
                start_constant: "C_ROW_ORDER",
                word_count_constant: "MAX_RECORDS",
            },
            ScratchSegmentBorrower {
                name: "I-3.5 一版引用的落点",
                start_constant: "C_ROW_ORDER",
                word_count_constant: "C_MAX_IN_VERSION",
            },
            ScratchSegmentBorrower {
                name: "I-5.1 主走读引用集合",
                start_constant: "C_ROW_ORDER",
                word_count_constant: "C_MAX_REFERENCES",
            },
            ScratchSegmentBorrower {
                name: "I-1.8 扫描项",
                start_constant: "C_ROW_ORDER",
                word_count_constant: "C_MAX_SCANNED",
            },
            ScratchSegmentBorrower {
                name: "I-9.20 墓碑两两比的 key",
                start_constant: "C_ROW_ORDER",
                word_count_constant: "DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE",
            },
        ],
    },
    BorrowedScratchSegment {
        name: "⑯ journal 组键",
        start_constant: "C_JOURNAL_GROUP_KEYS",
        end_constant: "C_JOURNAL_GROUP_KEYS_END",
        borrowers: &[ScratchSegmentBorrower {
            name: "I-1.8 扫描项的检查字",
            start_constant: "C_SCANNED_CHECK_WORDS",
            word_count_constant: "C_MAX_SCANNED",
        }],
    },
    BorrowedScratchSegment {
        name: "⑮ 分配记录的行号次序",
        start_constant: "C_ALLOCATION_RECORD_ROW_ORDER",
        end_constant: "C_ALLOCATION_RECORD_BUCKETS",
        borrowers: &[ScratchSegmentBorrower {
            name: "I-5.4 与 I-3.16 排一棵分配记录树的记录",
            start_constant: "C_ALLOCATION_RECORD_ROW_ORDER",
            word_count_constant: "C_MAX_ALLOC_RECORDS",
        }],
    },
    BorrowedScratchSegment {
        name: "⑮ 分配记录的桶表",
        start_constant: "C_ALLOCATION_RECORD_BUCKETS",
        end_constant: "SCRATCH_END",
        borrowers: &[
            ScratchSegmentBorrower {
                name: "I-3.9 已释放记录的桶表",
                start_constant: "C_RELEASED_BUCKETS",
                word_count_constant: "C_RELEASED_BUCKET_COUNT",
            },
            ScratchSegmentBorrower {
                name: "I-3.10 去重表的桶表",
                start_constant: "C_EXAMINED_BUCKETS",
                word_count_constant: "C_EXAMINED_BUCKET_COUNT",
            },
        ],
    },
];

/// 每一处借用者起在它借的那一段里、占的字不越过段的末端；⑮ 的两段接在 I-3.5 每一版引用的落点那张表之后、彼此不重叠。
/// 判别力：I-3.9 已释放记录的桶表改回借 `C_JOURNAL_GROUP_KEYS` 这条红（起点不在 ⑮ 的桶表那一段里；桶数照旧不过那一段的宽，内核照样解析得过）。
#[test]
fn every_borrower_of_the_row_order_and_bucket_segments_fits_inside_the_segment_it_borrows() {
    let module = parsed_module("池级 checker 扫描内核", &checker_scan_kernel_source());
    let value = |name: &str| value_of_the_u32_constant(&module, name);
    let end_of_the_version_placements = value("C_APPEARANCE_VERSION_PLACEMENTS")
        .checked_add(value("C_APPEARANCE_MAX_VERSION_PLACEMENTS"))
        .expect("草稿区的字下标装得进 u32");
    assert!(
        value("C_ALLOCATION_RECORD_ROW_ORDER") >= end_of_the_version_placements,
        "⑮ 的次序表起在 {}，压到 I-3.5 每一版引用的落点那张表（到 {end_of_the_version_placements} 为止）上",
        value("C_ALLOCATION_RECORD_ROW_ORDER")
    );
    for segment in &BORROWED_SCRATCH_SEGMENTS_OF_THE_SCAN_KERNEL {
        let segment_start = value(segment.start_constant);
        let segment_end = value(segment.end_constant);
        assert!(
            segment_start <= segment_end,
            "{}：起点 {segment_start} 在末端 {segment_end} 之后",
            segment.name
        );
        for borrower in segment.borrowers {
            let borrower_start = value(borrower.start_constant);
            let borrower_end = borrower_start
                .checked_add(value(borrower.word_count_constant))
                .expect("草稿区的字下标装得进 u32");
            assert!(
                segment_start <= borrower_start && borrower_end <= segment_end,
                "{} 借 {}：占 [{borrower_start}, {borrower_end})，那一段是 [{segment_start}, {segment_end})",
                borrower.name,
                segment.name
            );
        }
    }
}

/// 原因码 3（条目太多）的站点号里几处共用一个的：同一张表在几处判满（站点号、几处）。
/// 站点 41：一版用户可见树的根那张表满了，与之后给落点打标签时找不到那一版的根（那张表满了才找不到）；
/// 站点 115：扫描内核释放代高于 F 的已释放记录那张表两头往中间长、碰上了（收钉带出来那一截与收 F 之上那一截各判一次）。
const TOO_MANY_ENTRIES_SITES_JUDGED_IN_SEVERAL_PLACES_OF_ONE_TABLE: [(u32, usize); 2] =
    [(41, 2), (115, 2)];

/// 手写的内核源码的每一份文件：文件名与原文（分几份文件的那三族逐份列出）。
fn handwritten_kernel_sources() -> Vec<(&'static str, &'static str)> {
    super::GPU_JUDGE_COMMON_SOURCE_FILES
        .into_iter()
        .chain([
            (
                "crash_judge_recovery.wgsl",
                super::GPU_JUDGE_RECOVERY_SOURCE,
            ),
            (
                "crash_judge_checker_common.wgsl",
                GPU_JUDGE_CHECKER_COMMON_SOURCE,
            ),
        ])
        .chain(super::GPU_JUDGE_CHECKER_SOURCE_FILES)
        .chain(super::GPU_JUDGE_CHECKER_SCAN_SOURCE_FILES)
        .chain([(
            "crash_judge_checker_scan_rebuild.wgsl",
            super::GPU_JUDGE_CHECKER_SCAN_REBUILD_SOURCE,
        )])
        .collect()
}

/// 本包 `src/` 下的每一份 `.wgsl` 恰好是手写的内核源码（[`handwritten_kernel_sources`]）里点名一次的一份：文件名一一对上，
/// 编进去的原文与盘上同名那一份逐字节相同。分几份文件的那三族的文件表（`GPU_JUDGE_COMMON_SOURCE_FILES` 一类）写漏一份、
/// 一份写两次、名字与 `include_str!` 读的不是同一份，都在这里红；写漏的那一份不进任何内核，naga 解析之前看不出来。
/// 读的是本包 `src/` 目录（数据目录单元，目录里任何一份文件变了这一条重判）。
/// 判别力（`crates/mutations.tsv`）：扫描内核文件表里墓碑与谱系那一份的文件名写成回退下限与记账那一份的，这条红。
#[test]
fn every_wgsl_file_of_the_package_is_listed_once_under_its_own_name() {
    let listed = handwritten_kernel_sources();
    let listed_file_names: Vec<&str> = listed
        .iter()
        .map(|(file_name, _source)| *file_name)
        .collect();
    let listed_names: std::collections::BTreeSet<&str> =
        listed_file_names.iter().copied().collect();
    assert_eq!(
        listed_names.len(),
        listed_file_names.len(),
        "手写的内核源码里每个文件名只出现一次：{listed_file_names:?}"
    );
    let mut on_disk: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
        .expect("本包的 src/ 目录读得出")
    {
        let entry = entry.expect("src/ 下的一项读得出");
        let file_name = entry
            .file_name()
            .into_string()
            .expect("src/ 下的文件名是 UTF-8");
        if file_name.ends_with(".wgsl") {
            let text = std::fs::read_to_string(entry.path())
                .unwrap_or_else(|error| panic!("src/{file_name} 读不出：{error}"));
            on_disk.insert(file_name, text);
        }
    }
    assert_eq!(
        on_disk
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<&str>>(),
        listed_names,
        "src/ 下的 .wgsl 与手写的内核源码点名的文件一一对上"
    );
    for (file_name, source) in &listed {
        assert!(
            on_disk.get(*file_name).map(String::as_str) == Some(*source),
            "编进去的 {file_name} 与盘上 src/{file_name} 逐字节相同"
        );
    }
}

/// 分几份文件的那三族内核源码各自接起来（[`super::joined_wgsl_source`]）之后去注释的 SHA-256（[`code_of_the_wgsl_source`] 的写法）：
/// 共用那一半、走读内核、扫描内核。三族拆成几份文件只挪不改，接起来去注释与拆开之前那一份文件逐字相同，钉的就是那一份的摘要。
/// 改了某一族的代码（注释与空行除外），照改后接起来的那一份重钉这一族的值。
const CODE_DIGESTS_OF_THE_JOINED_KERNEL_SOURCE_FAMILIES: [(&str, &str); 3] = [
    (
        "共用那一半",
        "339faf49549508b10b204a55d02b3664aea9c229b43c2dbee004d50bb70745f6",
    ),
    (
        "走读内核",
        "2bc4fdf347ee11346943713e6bf621c133161ad301a364aa4c232e21a654e8db",
    ),
    (
        "扫描内核",
        "abf4c537df56825c1127690585b3b040241a4e8db472298b3a2b12d5cd417517",
    ),
];

/// 一段 WGSL 去注释之后的代码：每行 `//` 之前那一段（[`code_before_the_comment`]）去掉行尾空白，空的不要，每行以换行收尾。
fn code_of_the_wgsl_source(source: &str) -> String {
    source
        .lines()
        .map(|line| code_before_the_comment(line).trim_end())
        .filter(|code| !code.is_empty())
        .flat_map(|code| [code, "\n"])
        .collect()
}

/// 分几份文件的那三族内核源码各自按文件表的次序接起来，去注释之后的摘要与钉住的值
/// （[`CODE_DIGESTS_OF_THE_JOINED_KERNEL_SOURCE_FAMILIES`]）相同：文件表漏了一份、多了一份、次序换了、读错了一份文件，摘要都变。
/// 判别力（`crates/mutations.tsv`）：扫描内核文件表里墓碑与谱系那一份的 `include_str!` 读成回退下限与记账那一份，这条红。
#[test]
fn every_split_kernel_source_family_joins_into_the_code_pinned_for_it() {
    for ((family, source_files), (pinned_family, pinned_digest)) in [
        ("共用那一半", &super::GPU_JUDGE_COMMON_SOURCE_FILES[..]),
        ("走读内核", &super::GPU_JUDGE_CHECKER_SOURCE_FILES[..]),
        ("扫描内核", &super::GPU_JUDGE_CHECKER_SCAN_SOURCE_FILES[..]),
    ]
    .into_iter()
    .zip(CODE_DIGESTS_OF_THE_JOINED_KERNEL_SOURCE_FAMILIES)
    {
        assert_eq!(family, pinned_family, "钉住的值与文件表按族对齐");
        let code = code_of_the_wgsl_source(&super::joined_wgsl_source(source_files));
        assert_eq!(
            singlefs_unit_graph::sha256::sha256_of(code.as_bytes()).hexadecimal(),
            pinned_digest,
            "{family}（{}）接起来去注释之后的摘要",
            source_files
                .iter()
                .map(|(file_name, _source)| *file_name)
                .collect::<Vec<_>>()
                .join("、")
        );
    }
}

/// 一行 WGSL 里 `//` 之前的那一段（WGSL 没有字符串字面量，`//` 起头的都是注释）。
fn code_before_the_comment(line: &str) -> &str {
    line.split("//").next().expect("split 至少交回一段")
}

/// 几份源码里 `const <名字>: u32 = <十进制>u;` 的每个常量：名字与它在各份源码里的值（几份源码可以各定义一个同名常量）。
fn decimal_u32_constants_of(
    sources: &[(&str, &str)],
) -> std::collections::BTreeMap<String, std::collections::BTreeSet<u32>> {
    let mut constants: std::collections::BTreeMap<String, std::collections::BTreeSet<u32>> =
        std::collections::BTreeMap::new();
    for (_file_name, source) in sources {
        for line in source.lines() {
            let Some(definition) = code_before_the_comment(line).trim().strip_prefix("const ")
            else {
                continue;
            };
            let Some((name, after_the_name)) = definition.split_once(':') else {
                continue;
            };
            let Some((_type, initializer)) = after_the_name.split_once('=') else {
                continue;
            };
            let initializer = initializer.trim().trim_end_matches(';').trim();
            let Some(digits) = initializer.strip_suffix('u') else {
                continue;
            };
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                continue;
            }
            constants
                .entry(name.trim().to_string())
                .or_default()
                .insert(
                    digits
                        .parse::<u32>()
                        .expect("u32 常量的十进制初值装得进 u32"),
                );
        }
    }
    constants
}

/// 拼进原因字的那个站点（`<十进制>u`，或一个十进制常量的名字）换成站点号与「在哪写下这个号」：字面的写文件名与行号，
/// 具名常量写它的名字（几处用同一个常量算一处）。
fn site_and_where_it_is_written(
    site_text: &str,
    file_name: &str,
    line_number: usize,
    constants: &std::collections::BTreeMap<String, std::collections::BTreeSet<u32>>,
) -> (u32, String) {
    if let Some(digits) = site_text
        .strip_suffix('u')
        .filter(|digits| !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return (
            digits.parse::<u32>().expect("站点号装得进 u32"),
            format!("{file_name} 第 {line_number} 行"),
        );
    }
    let values = constants.get(site_text).unwrap_or_else(|| {
        panic!(
            "{file_name} 第 {line_number} 行的站点「{site_text}」既不是 `<十进制>u`，也不是哪一份源码里的十进制 u32 常量"
        )
    });
    let mut distinct_values = values.iter().copied();
    let (Some(site), None) = (distinct_values.next(), distinct_values.next()) else {
        panic!(
            "{file_name} 第 {line_number} 行的站点常量 {site_text} 在几份源码里定义成不同的值 {values:?}"
        );
    };
    (site, format!("具名常量 {site_text}"))
}

/// 一次调用 `(` 之后那一段里，到与它配对的 `)` 为止的实参原文。
fn arguments_up_to_the_closing_parenthesis(after_the_opening_parenthesis: &str) -> &str {
    let mut depth_of_nested_parentheses = 0_usize;
    for (byte_index, character) in after_the_opening_parenthesis.char_indices() {
        match character {
            '(' => depth_of_nested_parentheses += 1,
            ')' if depth_of_nested_parentheses == 0 => {
                return &after_the_opening_parenthesis[..byte_index];
            }
            ')' => depth_of_nested_parentheses -= 1,
            _ => {}
        }
    }
    panic!(
        "调用的实参到行尾都没配上 `)`（这一判只认写在一行里的调用）：{after_the_opening_parenthesis}"
    );
}

/// 原因码 3（条目太多）的原因字是 `UNDECIDABLE_TOO_MANY_ENTRIES | (<站点> << 8u)`：看站点号就知道是哪一张表满了
/// （[`super::undecidable_reasons_of_the_gpu_verdict`] 照它打判不了那一句）。读手写的每一份 `.wgsl`（[`handwritten_kernel_sources`]），把每一处写下站点号的地方都收上：
/// 字面的 `<十进制>u`、十进制常量的名字（几处用同一个常量算一处）、拼好整个原因字的具名常量（`const X: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (…)`），
/// 与按参数 `site` 传进去的那几个函数在每个调用点传的实参；不带站点（恢复内核那几处 `= UNDECIDABLE_TOO_MANY_ENTRIES;`）的不收。
/// 每个站点号只在一处写下，登记的几张表（[`TOO_MANY_ENTRIES_SITES_JUDGED_IN_SEVERAL_PLACES_OF_ONE_TABLE`]）除外、处数恰好是登记的数。
/// 认不出写法的那一行直接红，不悄悄漏掉。
/// 判别力：扫描重建读 inode 树的栈满了那一处（`SCAN_REBUILD_SITE_INODE_TREE_PENDING_PAST_THE_STACK`）改回 115，与扫描内核
/// 释放代高于 F 那张表的站点拼出同一个原因字 0x7303，这条红。
#[test]
fn every_too_many_entries_site_is_written_in_one_place_except_the_tables_judged_full_in_several_places(
) {
    const SITE_SHIFTED_INTO_THE_REASON: &str = "UNDECIDABLE_TOO_MANY_ENTRIES | (";
    const END_OF_THE_SITE_SHIFT: &str = " << 8u)";
    const SITE_PASSED_AS_A_PARAMETER: &str = "site";
    let sources = handwritten_kernel_sources();
    let constants = decimal_u32_constants_of(&sources);
    let mut places_of_each_site: std::collections::BTreeMap<
        u32,
        std::collections::BTreeSet<String>,
    > = std::collections::BTreeMap::new();
    let mut functions_taking_the_site_as_a_parameter: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();
    for &(file_name, source) in &sources {
        let code_lines: Vec<&str> = source.lines().map(code_before_the_comment).collect();
        for (line_index, code) in code_lines.iter().enumerate() {
            if !code.contains("UNDECIDABLE_TOO_MANY_ENTRIES") {
                continue;
            }
            let line_number = line_index + 1;
            let trimmed = code.trim();
            let Some((_, after_the_reason)) = code.split_once(SITE_SHIFTED_INTO_THE_REASON) else {
                assert!(
                    trimmed.starts_with("const UNDECIDABLE_TOO_MANY_ENTRIES:")
                        || trimmed.ends_with("= UNDECIDABLE_TOO_MANY_ENTRIES;"),
                    "{file_name} 第 {line_number} 行认不出原因码 3 的写法（定义、不带站点的 `= UNDECIDABLE_TOO_MANY_ENTRIES;`、\
                         `UNDECIDABLE_TOO_MANY_ENTRIES | (<站点> << 8u)` 三种）：{trimmed}"
                );
                continue;
            };
            let (site_text, _) = after_the_reason
                .split_once(END_OF_THE_SITE_SHIFT)
                .unwrap_or_else(|| {
                    panic!("{file_name} 第 {line_number} 行站点之后不是 `{END_OF_THE_SITE_SHIFT}`：{trimmed}")
                });
            let site_text = site_text.trim();
            if site_text == SITE_PASSED_AS_A_PARAMETER {
                let function_line = code_lines[..=line_index]
                    .iter()
                    .rev()
                    .find(|candidate| candidate.starts_with("fn "))
                    .unwrap_or_else(|| {
                        panic!(
                            "{file_name} 第 {line_number} 行按参数传站点，往上找不到它所在的 `fn`"
                        )
                    });
                let function_name = function_line
                    .strip_prefix("fn ")
                    .expect("上面按 `fn ` 起头找的")
                    .split('(')
                    .next()
                    .expect("split 至少交回一段")
                    .trim();
                functions_taking_the_site_as_a_parameter.insert(function_name.to_string());
                continue;
            }
            let (site, mut written_at) =
                site_and_where_it_is_written(site_text, file_name, line_number, &constants);
            if let Some(definition) = trimmed.strip_prefix("const ") {
                let reason_word_constant = definition
                    .split(':')
                    .next()
                    .expect("split 至少交回一段")
                    .trim();
                written_at = format!("具名常量 {reason_word_constant}");
            }
            places_of_each_site
                .entry(site)
                .or_default()
                .insert(written_at);
        }
    }
    for function_name in &functions_taking_the_site_as_a_parameter {
        let call = format!("{function_name}(");
        let definition = format!("fn {call}");
        for &(file_name, source) in &sources {
            for (line_index, line) in source.lines().enumerate() {
                let code = code_before_the_comment(line);
                if code.contains(&definition) {
                    continue;
                }
                let Some((_, after_the_call)) = code.split_once(&call) else {
                    continue;
                };
                let last_argument = arguments_up_to_the_closing_parenthesis(after_the_call)
                    .rsplit(',')
                    .next()
                    .expect("rsplit 至少交回一段")
                    .trim();
                let (site, written_at) = site_and_where_it_is_written(
                    last_argument,
                    file_name,
                    line_index + 1,
                    &constants,
                );
                places_of_each_site
                    .entry(site)
                    .or_default()
                    .insert(written_at);
            }
        }
    }
    let sites_written_in_several_places: std::collections::BTreeMap<u32, Vec<String>> =
        places_of_each_site
            .iter()
            .filter(|(_site, places)| places.len() > 1)
            .map(|(site, places)| (*site, places.iter().cloned().collect()))
            .collect();
    let place_counts_of_those_sites: std::collections::BTreeMap<u32, usize> =
        sites_written_in_several_places
            .iter()
            .map(|(site, places)| (*site, places.len()))
            .collect();
    assert_eq!(
        place_counts_of_those_sites,
        TOO_MANY_ENTRIES_SITES_JUDGED_IN_SEVERAL_PLACES_OF_ONE_TABLE
            .into_iter()
            .collect::<std::collections::BTreeMap<u32, usize>>(),
        "原因码 3 的站点号除登记的几张表之外各只在一处写下（不然看原因字分不出是哪一张表满了）；写在几处的：{sites_written_in_several_places:?}"
    );
}

/// 判器三处省工夫的写法那一条流的状态数上限：几百个以内的小流，单跑几秒。
const MOST_STATES_OF_THE_FLOW_OF_THE_SAVED_WORK: u64 = 1000;

/// 判器三处省工夫的写法那一条流里段长不超过这个数的段逐个真子集展开，更长的段只以整段持久进入后面的状态（状态数压在几百个以内）。
const EXPAND_UP_TO_IN_THE_FLOW_OF_THE_SAVED_WORK: usize = 3;

/// 记账分片切成多少片的那一种切法：每片 [`crate::crash_judge_dispatch::ACCOUNTING_SLICE_POSITIONS`] ÷ 256 个位置，项少的流上多数片一项都不落、
/// 落了项的那几片前后都有空片，下一片从前一片停下的那一项接着做的那一份每一片都要读对。
const ACCOUNTING_SLICES_OF_THE_NARROW_CUT: u32 = 256;

/// 判器三处省工夫的写法（记账分片从前一片停下的那一项接着做、不每片整轮重加一遍权重；走读内核逐条 inode 记录那一圈不回头再读前一条记录的号、
/// 对象出生代只读一次、保留字节按字读；删除态扫描与墓碑、谱系那一遍同一版里同一条根指针读全了的 inode 树与容器树不重读）判出的结论与 CPU 参照判器逐状态相同：
/// 新池新建文件那条流（两块稀疏内存盘上 mkfs、取号、暖机、新建文件，录 mkfs 之后那一段）在同一张借来的卡上一次派活判完，记账分片照判器自己的片宽切、
/// 整轮一片、切成 [`ACCOUNTING_SLICES_OF_THE_NARROW_CUT`] 片三种切法判出的结论字逐字相同，切成那么多片的那一份与 CPU 参照判器逐状态对拍相同。
/// 先核这条流上真有记账那一判（I-3.1）与逐条 inode 记录那几判（I-9.7）判到的状态：三处省掉的工夫在这条流上都有活可做，判错了比得出来。
/// 三种切法在同一张卡上同一个判器依次判（比的就是同一批状态换一种切法），不并行。
/// 判表过库与 CPU 参照取样在借卡之前、都开这条用例自己的一次性库，过库之后核判表写在这一份里：单跑与同一个进程里别的用例设着一次性库时开的是同一份。
/// 判别力（`crates/mutations.tsv`）：不把一次性库设成这个进程开的那一份，判表写到别处，核判表那一句红。
#[test]
fn verdict_words_of_the_new_pool_file_creation_are_the_same_whatever_the_accounting_slices_and_agree_with_the_cpu_judge(
) {
    use crate::crash::{crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion};
    use crate::crash_judge_conclusions::{
        cross_check_words_agree_in_the_cross_check, cross_check_words_of_cpu_judgement,
        CrossCheckWords,
    };
    use crate::crash_judge_dispatch::{
        mebibytes_a_judge_needs, GpuCrashJudge, PackedJudgeTables, ACCOUNTING_SLICE_POSITIONS,
    };
    use crate::crash_judge_tables::{build_judge_tables, JudgeFlow};
    use crate::crash_library::{
        judge_tables_through_the_pairing_library, DisposableLibrary, StageOutcome,
    };
    use crate::crash_state_enumeration::{sample_crash_states_on_the_cpu, CrashPointReplayItem};
    use crate::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    use crate::memory_pool::{
        writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, SparseBlockDevice,
    };
    use crate::segments::FixedGeometry;
    use crate::verdict_store::PruneServerFailureHandling;
    use crate::{RecordingBlockDevice, SharedStream};
    use singlefs_core::address::DeviceIdentity;
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;

    const TEST_NAME: &str = "verdict_words_of_the_new_pool_file_creation_are_the_same_whatever_the_accounting_slices_and_agree_with_the_cpu_judge";
    const DEVICE_BYTES: u64 = 4 << 30;
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let stream = SharedStream::retaining_contents();
    let mut devices = [0, 1].map(|device| {
        (
            DeviceIdentity(device),
            RecordingBlockDevice::with_shared_stream(
                DeviceIdentity(device),
                SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                stream.clone(),
            ),
        )
    });
    let parameters = crate::known_bad_images::parameters_of_the_common_test_pool();
    let run =
        crate::scenario::run_new_pool_file_creation(&parameters, &mut devices, &stream, |_, _| {})
            .expect("新池新建文件那条路做得成");
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], DEVICE_BYTES);
    base.apply(&operations[..run.mkfs_operation_count]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[run.mkfs_operation_count..],
        &FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        },
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == crate::segments::StepKind::RootRecordFua)
        .expect("新建文件那次发布有一条根槽写");
    let versions = vec![PublishedVersion {
        instance: run.output.root.instance,
        checkpoint_txg: run.output.root.checkpoint_txg,
        content: crate::scenario::first_file_content(),
    }];
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO_IN_THE_FLOW_OF_THE_SAVED_WORK {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &base,
        writes: &writes,
        segments: &segments,
        judged_root_index,
        versions: &versions,
        expansion: &expansion,
    })
    .expect("新池新建文件那条流建得出判表");
    let state_count = tables.state_count;
    assert_eq!(
        state_count,
        crash_point_replay_plan_state_count(&base, &writes, &segments, &expansion),
        "判表的状态数与崩溃点重放的计划相同"
    );
    assert!(
        state_count > 0 && state_count <= MOST_STATES_OF_THE_FLOW_OF_THE_SAVED_WORK,
        "这条流 {state_count} 个状态：至少一个，又在 {MOST_STATES_OF_THE_FLOW_OF_THE_SAVED_WORK} 个以内"
    );
    let every_state = usize::try_from(state_count).expect("装得进 usize");
    // 经产品入口开库的两处（判表过库、CPU 参照取样）开的都是这条用例自己的一次性库：没设就开，开到的是同一个进程里
    // 别的用例此刻设着的那一份（那条用例收尾时停掉它的临时服务，这一边接着读写就连不上），没人设着时开到全仓那一份。
    // 设着的那一段只罩这两处、不罩借卡：借卡等别的线程还卡，持着它等，设一次性库的别的用例都跟着等。
    let disposable_library = DisposableLibrary::new("accounting-slice-pairing")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let packed_tables = PackedJudgeTables::of(&tables);
    let (packed, cpu) = {
        let _the_library_of_this_process = disposable_library.make_it_the_library_of_this_process();
        // 判表先入库、再读库，GPU 只拿读回的那一份算
        let packed = match judge_tables_through_the_pairing_library(TEST_NAME, &packed_tables) {
            StageOutcome::Judged(packed) => packed,
            StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => {
                println!("JUDGE_TABLES_RECORDED_ONLY test={TEST_NAME}");
                return;
            }
        };
        let mut cpu: Vec<CrossCheckWords> = Vec::new();
        let Some(judged_on_the_cpu) = sample_crash_states_on_the_cpu(
            CrashPointReplayItem {
                name: TEST_NAME,
                setting_file: "crates/singlefs-verification/src/crash_judge_gpu.rs",
            },
            &base,
            &writes,
            &segments,
            judged_root_index,
            &versions,
            &expansion,
            0..state_count,
            &mut |ordinal, _image, judgement| {
                assert_eq!(
                    ordinal,
                    u64::try_from(cpu.len()).expect("装得进 u64"),
                    "CPU 参照按序号次序交回"
                );
                cpu.push(cross_check_words_of_cpu_judgement(judgement));
            },
        )
        .judged() else {
            return;
        };
        assert_eq!(judged_on_the_cpu, state_count, "CPU 参照判了整条流");
        (packed, cpu)
    };
    assert!(
        disposable_library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .expect("开得了这条用例自己的一次性库")
            .read_packed_judge_tables(&crate::sha256::sha256_digest(&packed_tables.to_bytes()))
            .expect("读得了这条用例自己的一次性库")
            .is_some(),
        "判表过库写进的是这条用例自己的一次性库（不是别的用例设着的那一份、也不是全仓那一份）"
    );
    let Some(lease) = cards
        .lease_card_or_report_not_run(TEST_NAME, mebibytes_a_judge_needs(&packed, every_state))
    else {
        return;
    };
    let judge = GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, every_state);
    assert_eq!(
        judge.states_per_dispatch(),
        every_state,
        "要一次派活判完整条流的判器开出来的派活上限"
    );
    let narrow_cut: Vec<u32> = (1..=ACCOUNTING_SLICES_OF_THE_NARROW_CUT)
        .map(|slice| slice * (ACCOUNTING_SLICE_POSITIONS / ACCOUNTING_SLICES_OF_THE_NARROW_CUT))
        .collect();
    let with_the_slice_widths_of_the_judge = judge.judge_states_in_one_dispatch(0..state_count);
    let in_one_slice = judge.judge_states_in_one_dispatch_slicing_the_accounting_at(
        0..state_count,
        &[ACCOUNTING_SLICE_POSITIONS],
    );
    let in_the_narrow_cut =
        judge.judge_states_in_one_dispatch_slicing_the_accounting_at(0..state_count, &narrow_cut);
    drop(judge);
    drop(lease);
    for (cut, verdicts) in [
        ("照判器自己的片宽", &with_the_slice_widths_of_the_judge),
        ("整轮一片", &in_one_slice),
    ] {
        assert_eq!(verdicts.len(), every_state, "{cut}那一份的结论一个状态一份");
        let first_difference = verdicts
            .iter()
            .zip(&in_the_narrow_cut)
            .position(|(this_cut, narrow)| this_cut != narrow);
        assert_eq!(
            first_difference, None,
            "记账分片{cut}判出的结论字与切成 {ACCOUNTING_SLICES_OF_THE_NARROW_CUT} 片判出的不同，第一个不同的状态序号在这里"
        );
    }
    let judged_on_some_state = |invariant: &str| {
        let bit = 1u128
            << super::IMPLEMENTED_INVARIANTS
                .iter()
                .position(|name| *name == invariant)
                .expect("池级 checker 的清单里有这一条");
        in_the_narrow_cut
            .iter()
            .any(|verdict| super::pool_checker_judged_bits(verdict) & bit != 0)
    };
    for invariant in ["I-3.1", "I-9.7"] {
        assert!(
            judged_on_some_state(invariant),
            "这条流上至少一个状态 GPU 判到了 {invariant}：不然省掉的那份工夫在这条流上没活可做，判错了也比不出来"
        );
    }
    let disagreements: Vec<String> = cpu
        .iter()
        .zip(&in_the_narrow_cut)
        .enumerate()
        .filter_map(|(ordinal, (cpu_words, gpu_verdict))| {
            let gpu_words = super::cross_check_words_of_gpu_verdict(gpu_verdict);
            (!cross_check_words_agree_in_the_cross_check(&gpu_words, cpu_words))
                .then(|| format!("状态 {ordinal}：CPU {cpu_words:?}；GPU {gpu_words:?}"))
        })
        .collect();
    assert!(
        disagreements.is_empty(),
        "{} 个状态 GPU 与 CPU 对拍的字不同（前几个）：\n{}",
        disagreements.len(),
        disagreements
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    println!(
        "CRASH_JUDGE_SAVED_WORK_AGREES test={TEST_NAME} states={state_count} accounting_slices_of_the_narrow_cut={ACCOUNTING_SLICES_OF_THE_NARROW_CUT}"
    );
}
