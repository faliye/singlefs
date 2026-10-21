//! 计划跳过这次发布本来就重写的节点、补上 R 里真要搬的单元（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，
//! 用户 2026-10-16 定「规划时补足」）：照抄意图的一批计划里每个单元都真搬，一批至少搬一个；R 里只剩这次发布本来就重写的节点时照 F2
//! 同一次发布删意图。改前计划按槽升序取前 K 个可搬单元，取到的全是这次发布本来就重写的节点时这一批一个都没搬、R 还不空、意图照抄，
//! 理想模型判「一次整理发布要么搬一批、要么删掉意图」。
//!
//! 五段历史是两条腿造出的五个取样点（辩方 `research/prompts/m4-segment-code-r2-sonnet-model/out/seeds.txt` 的种子偏移 940015、940105、
//! 960004；攻方 `research/prompts/m4-segment-code-r2-opus-model/out/trace297.txt` 与 `out/seed6318-small-device.txt` 的种子下标 297、6318），
//! 在改前的核心上按「模型判这一次整理发布对意图做的事不合、实现搬了 0 个单元、活意图的 R 按记录不空」缩过（逐段删步、仍打中才删），
//! 每一步照抄缩出来的那一份。执行器每一版按模型比：照抄意图的一批没搬单元，模型就判不合，历史走不完。
//! 模型一侧「一批至少搬一个 R 里真要搬的单元」由模型那一批另改；这里只断言核心一侧：历史走完（没有「没搬、意图照抄」的一批），
//! 每一趟整理报的批数不多于搬的单元数。

use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, CompactionChoice, CompactionRunEnding, ContentChoice,
    ContentLength, GeneratedHistory, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker, RollbackTargetChoice,
    SnapshotChoice, StepOutcome,
};
use singlefs_harness::SharedStream;

fn compaction(
    batch_units: u64,
    region_selector: u64,
    ending: CompactionRunEnding,
) -> HistoryOperation {
    HistoryOperation::CompactWhileMounted(CompactionChoice {
        batch_units,
        region_selector,
        ending,
    })
}

fn overwrite_spanning_data_units(
    data_units: u64,
    selector: u64,
    fill_seed: u64,
) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
            data_units,
            selector,
        },
        fill_seed,
    })
}

fn overwrite_inside_one_data_unit(selector: u64, fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector },
        fill_seed,
    })
}

fn roll_back_to_the_ring_root(index_from_newest: u64) -> HistoryOperation {
    HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest })
}

fn destroy_snapshot(index_among_the_snapshots: u64) -> HistoryOperation {
    HistoryOperation::DestroySnapshot(SnapshotChoice {
        index_among_the_snapshots,
    })
}

/// 两条腿跑这几段历史时的执行器：不跑每一步的池级 checker、判空间准入，盘宽照取样点那一档。
fn execution_on(device_width: HistoryDeviceWidth) -> HistoryExecution {
    HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    }
}

/// 从第一个文件起跑 `operations`：每一版与模型对得上、历史走完，每一趟整理报的批数不多于搬的单元数（一批至少搬一个）。
fn every_compaction_batch_relocates_a_unit(
    sample: &str,
    operations: Vec<HistoryOperation>,
    device_width: HistoryDeviceWidth,
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let run = execute_history_with(
        &history,
        execution_on(device_width),
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "取样点 {sample}：每一版与模型对得上——照抄意图的一批计划里有这次发布本来就重写的节点、一个没搬而 R 还不空时，模型判「一次整理发布要么搬一批、要么删掉意图」；各步结局 {:?}",
        run.outcomes
    );
    for outcome in &run.outcomes {
        if let StepOutcome::Applied(AppliedEffect::Compacted {
            batches_published,
            units_relocated,
            ..
        }) = outcome
        {
            assert!(
                batches_published <= units_relocated,
                "取样点 {sample}：一批至少搬一个单元，报了 {batches_published} 批、只搬了 {units_relocated} 个"
            );
        }
    }
}

/// 辩方种子偏移 940015（比重 `COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`，两块 4 GiB 的盘）缩出的 23 步：最后那一趟整理一批 2 个。
#[test]
fn the_defense_seed_940015_with_rollback_runs_every_batch_relocating_a_unit() {
    every_compaction_batch_relocates_a_unit(
        "defense-940015",
        vec![
            overwrite_inside_one_data_unit(8_979_874_509_176_992_068, 10_312_936_057_541_030_951),
            compaction(
                3,
                8_730_098_076_572_958_705,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 3 },
            ),
            compaction(
                3,
                3_156_702_167_382_834_759,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 3 },
            ),
            overwrite_spanning_data_units(
                147,
                2_874_267_565_918_133_058,
                13_465_300_084_566_805_270,
            ),
            HistoryOperation::CloseAndMountWritable,
            overwrite_spanning_data_units(
                2,
                14_376_923_743_907_527_920,
                10_310_547_561_581_956_661,
            ),
            HistoryOperation::CloseAndMountWritable,
            overwrite_spanning_data_units(146, 852_016_302_994_383_801, 2_822_401_457_709_994_635),
            overwrite_spanning_data_units(150, 11_307_181_166_667_380_573, 982_563_485_564_695_347),
            HistoryOperation::ColdStartRecover,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::Unlink,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CreateInode,
            roll_back_to_the_ring_root(4_578_263_121_225_100_889),
            HistoryOperation::CreateInode,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CreateInode,
            compaction(
                1,
                10_352_106_060_286_177_707,
                CompactionRunEnding::RunsUntilItStops,
            ),
            destroy_snapshot(15_790_580_355_894_682_684),
            destroy_snapshot(6_196_520_852_254_731_681),
            compaction(
                2,
                13_042_593_846_479_419_307,
                CompactionRunEnding::RunsUntilItStops,
            ),
        ],
        HistoryDeviceWidth::FourGibibytes,
    );
}

/// 辩方种子偏移 940105（比重 `COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`，两块 4 GiB 的盘）缩出的 15 步：最后那一趟整理一批 1 个。
#[test]
fn the_defense_seed_940105_with_rollback_runs_every_batch_relocating_a_unit() {
    every_compaction_batch_relocates_a_unit(
        "defense-940105",
        vec![
            overwrite_inside_one_data_unit(11_709_615_003_056_211_060, 16_403_038_083_313_145_199),
            HistoryOperation::CloseAndMountWritable,
            compaction(
                1,
                2_241_302_317_298_536_861,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 3 },
            ),
            overwrite_inside_one_data_unit(12_755_404_112_133_445_388, 12_124_826_240_021_228_177),
            overwrite_spanning_data_units(
                150,
                6_795_418_216_703_896_325,
                18_378_029_841_729_159_842,
            ),
            roll_back_to_the_ring_root(1),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::Unlink,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CreateInode,
            HistoryOperation::CreateInode,
            compaction(
                2,
                710_203_653_678_073_136,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 3 },
            ),
            compaction(
                1,
                10_268_428_566_788_353_321,
                CompactionRunEnding::RunsUntilItStops,
            ),
        ],
        HistoryDeviceWidth::FourGibibytes,
    );
}

/// 辩方种子偏移 960004（比重 `COMPACTION_WHILE_MOUNTED`，单元区 256 槽的盘）缩出的 12 步：最后那一趟整理一批 1 个。
#[test]
fn the_defense_seed_960004_on_a_unit_area_of_256_slots_runs_every_batch_relocating_a_unit() {
    every_compaction_batch_relocates_a_unit(
        "defense-960004",
        vec![
            HistoryOperation::CloseAndMountWritable,
            compaction(
                2,
                5_886_892_624_684_841_274,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
            HistoryOperation::CreateSnapshot,
            compaction(
                2,
                12_715_887_468_824_387_439,
                CompactionRunEnding::RunsUntilItStops,
            ),
            HistoryOperation::CloseAndMountWritable,
            overwrite_spanning_data_units(150, 2_687_902_185_441_531_965, 719_385_292_118_242_150),
            HistoryOperation::CreateSnapshot,
            destroy_snapshot(14_596_391_302_675_724_985),
            compaction(
                2,
                6_768_340_407_047_764_127,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
            compaction(
                3,
                8_577_925_439_445_034_100,
                CompactionRunEnding::RunsUntilItStops,
            ),
            overwrite_spanning_data_units(290, 223_620_745_509_267_937, 2_469_904_695_844_261_333),
            compaction(
                1,
                4_922_804_022_857_305_027,
                CompactionRunEnding::RunsUntilItStops,
            ),
        ],
        HistoryDeviceWidth::UnitAreaOf256Slots,
    );
}

/// 攻方种子下标 297（比重 `COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`，两块 4 GiB 的盘）缩出的 20 步（`ATTACK_KEEP` 那一份）：最后那一趟整理一批 1 个，改前计划的是槽 51965 上一个这次发布本来就重写的码 2 节点。
#[test]
fn the_attack_seed_297_with_rollback_runs_every_batch_relocating_a_unit() {
    every_compaction_batch_relocates_a_unit(
        "attack-297",
        vec![
            overwrite_spanning_data_units(
                149,
                14_805_491_705_457_366_444,
                17_954_462_892_967_153_791,
            ),
            compaction(
                1,
                13_885_814_567_853_251_451,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
            HistoryOperation::CloseAndMountWritable,
            compaction(
                3,
                2_062_025_774_686_577_469,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
            overwrite_spanning_data_units(
                149,
                17_457_907_918_451_066_060,
                8_190_456_517_236_343_239,
            ),
            compaction(
                3,
                16_601_565_660_308_693_961,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
            overwrite_spanning_data_units(
                145,
                1_351_345_780_662_220_768,
                7_249_505_663_551_106_811,
            ),
            overwrite_spanning_data_units(
                11,
                9_352_197_017_032_374_714,
                11_039_974_942_020_947_325,
            ),
            compaction(
                3,
                3_549_472_089_299_149_804,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 },
            ),
            overwrite_inside_one_data_unit(4_646_736_669_117_851_417, 7_854_517_636_618_916_420),
            overwrite_spanning_data_units(
                150,
                8_408_847_985_767_659_421,
                3_161_318_307_980_811_632,
            ),
            compaction(
                3,
                2_777_600_857_152_106_407,
                CompactionRunEnding::RunsUntilItStops,
            ),
            overwrite_spanning_data_units(
                147,
                15_857_935_507_500_078_114,
                15_753_153_114_236_951_071,
            ),
            roll_back_to_the_ring_root(8_371_341_961_774_013_443),
            compaction(
                1,
                6_219_102_914_233_570_556,
                CompactionRunEnding::RunsUntilItStops,
            ),
            overwrite_spanning_data_units(
                146,
                15_631_396_774_324_747_441,
                1_953_502_442_396_461_192,
            ),
            overwrite_spanning_data_units(
                147,
                7_022_280_228_427_239_610,
                3_709_565_245_189_348_056,
            ),
            compaction(
                2,
                16_012_380_545_371_126_641,
                CompactionRunEnding::RunsUntilItStops,
            ),
            HistoryOperation::CloseAndMountWritable,
            compaction(
                1,
                9_180_866_823_888_211_979,
                CompactionRunEnding::RunsUntilItStops,
            ),
        ],
        HistoryDeviceWidth::FourGibibytes,
    );
}

/// 攻方种子下标 6318（攻方第二轮比重 `R2_MIX`，单元区 240 槽的盘）缩出的 21 步：最后那一趟整理一批 2 个，改前计划的两个码 2 节点都是这次发布本来就重写的。
#[test]
fn the_attack_seed_6318_on_a_unit_area_of_240_slots_runs_every_batch_relocating_a_unit() {
    every_compaction_batch_relocates_a_unit(
        "attack-6318",
        vec![
            overwrite_spanning_data_units(7, 9_706_620_714_983_381_225, 13_475_346_977_979_937_379),
            overwrite_inside_one_data_unit(13_936_815_927_496_291_419, 7_067_381_312_692_110_561),
            HistoryOperation::CloseAndMountWritable,
            overwrite_inside_one_data_unit(15_481_590_606_002_455_836, 959_582_832_489_908_953),
            roll_back_to_the_ring_root(3),
            roll_back_to_the_ring_root(3),
            overwrite_spanning_data_units(
                148,
                3_172_577_091_865_207_195,
                14_718_877_161_866_370_201,
            ),
            HistoryOperation::CloseAndMountWritable,
            compaction(
                2,
                9_913_521_736_953_594_569,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
            roll_back_to_the_ring_root(7_868_911_180_495_501_231),
            compaction(
                3,
                14_733_301_700_930_671_043,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
            compaction(
                1,
                13_382_315_964_437_831_618,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::Unlink,
            HistoryOperation::CreateSnapshot,
            destroy_snapshot(14_961_788_493_459_036_253),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CreateInode,
            compaction(
                2,
                2_553_121_787_666_098_568,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
        ],
        HistoryDeviceWidth::UnitAreaOf240Slots,
    );
}
