//! 设计轮 m4-admission-delta-r1 攻方 opus-s2 那一次抽样的模型（攻方副本用，不进仓的 crates/）：主 agent 照 `tools/rerun.sh` 把它拷进冻结副本的
//! `crates/singlefs-harness/tests/` 跑两遍——今天的代码一遍（三条都该绿），套上 `tools/apply_a3_emulation.py` 之后一遍
//! （候选 A3「准入判挪到差量上界、全量规划挪到准入之后」的模拟：式子那一判挪到 `settle_the_allocation_record_tree` 之前，
//! 需求取普通分配加 ckpt_cost）。三条：
//! - 今天：式子已经不够、而一次顺序写切出的 journal 记录多于在飞上限时，会话交回记录超上限、一串抬 F 都不推、盘上逐字节不变；
//!   套上 A3 模拟之后这一条该红（会话先见到空间不够、推抬 F）。
//! - 今天：1 MiB 环、4 GiB 盘上顺序写 85 个数据单元照发；按 ckpt_cost 给固定点那一截取上界去切记录，条数多于上限。
//! - 今天：每一次建 inode 的发布给记账树留一行台阶行，行数随这类发布的次数一行一行涨（ckpt_cost 的记账树那一项逐行走它们）。
//! 盘都是内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`）。
#![allow(dead_code, reason = "搭池那一份各用例只用到其中一部分")]

#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;

use common_admission_pool::{plain_device, PoolUnderTest};
use singlefs_core::admission::{
    admission_reading_before_a_publish, checkpoint_cost_of_the_version_to_build_on,
};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::{UserChangePublished, UserChangeRefused};
use singlefs_core::transaction::{
    journal_record_limit_of_one_publish, JournalRecordNamedEntryCapacity, PublishError,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::scenario::e142_parameters;

/// 1 MiB 的 journal 环：在飞上限 256 ÷ 3 = 85 条（`core_review_unit_area_start_and_publish_limits.rs` 用同一个环）。
const JOURNAL_RING_OF_ONE_MEBIBYTE: u64 = 1 << 20;
/// 多于在飞上限的那次顺序写切成的数据单元数：85 + 1。
const DATA_UNITS_OF_THE_WRITE_PAST_THE_LIMIT: u64 = 86;
/// 正好等于在飞上限的那次顺序写切成的数据单元数。
const DATA_UNITS_OF_THE_WRITE_AT_THE_LIMIT: u64 = 85;
/// 一个数据单元两槽。
const SLOTS_OF_A_DATA_UNIT: u64 = 2;
/// 扫单元区槽数时从多少往下扫、到多少停、一步多少槽。
const UNIT_AREA_SLOTS_SCANNED_FROM: u64 = 4000;
const UNIT_AREA_SLOTS_SCANNED_DOWN_TO: u64 = 1100;
const UNIT_AREA_SLOTS_SCAN_STEP: u64 = 32;
/// 数台阶行那一条建 inode 的次数。
const INODE_CREATING_PUBLISHES: u64 = 40;

fn parameters_on_the_one_mebibyte_ring() -> MakeFilesystemParameters {
    let mut parameters = e142_parameters(512, 512);
    parameters.geometry.journal_ring_bytes = JOURNAL_RING_OF_ONE_MEBIBYTE;
    parameters
}

fn unit_area_start_slot_on_the_one_mebibyte_ring() -> u64 {
    UnitAreaStart::following_the_journal_ring(JOURNAL_RING_OF_ONE_MEBIBYTE)
        .expect("1 MiB 的环末尾的下一个槽 1088 = 17 × 64，落在聚簇段边界上")
        .slot()
        .0
}

/// 从文件偏移 0 顺序写这么多字节切成 `data_units` 个一单元事务（每个数据单元装 `data_unit_payload_capacity()` 字节净荷）。
fn content_bytes_of_a_sequential_write_into(data_units: u64) -> usize {
    let payload_bytes =
        u64::try_from(data_unit_payload_capacity()).expect("一个数据单元的净荷装得进 u64");
    usize::try_from((data_units - 1) * payload_bytes + 1).expect("几兆字节装得进 usize")
}

/// 会话现行那一版上，式子那一判（发布之前的读数，`admission_reading_before_a_publish`）每块盘的可用里最小的那个，以槽计（向下取整，可为负）。
fn smallest_formula_available_slots<Device: common_admission_pool::DeviceOverASparseImage>(
    pool: &PoolUnderTest<Device>,
) -> i128 {
    let reading = admission_reading_before_a_publish(
        &pool.session().allocator,
        Some(pool.current_file_version()),
    );
    reading
        .available_on_each_device()
        .into_iter()
        .map(|(_device, available)| available.0.div_euclid(i128::from(SLOT_BYTES)))
        .min()
        .expect("两块盘")
}

/// 一次用户改动的结局写成一行：哪一种、推了几串抬 F、被拒的话是哪一种拒绝。
fn outcome_line(outcome: &Result<UserChangePublished, UserChangeRefused>) -> String {
    match outcome {
        Ok(published) => format!("published floor_raises={}", published.floor_raises.len()),
        Err(UserChangeRefused::Publish {
            cause,
            floor_raises,
            ..
        }) => format!(
            "refused_publish cause={} floor_raises={}",
            publish_error_name(cause),
            floor_raises.len()
        ),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => format!(
            "refused_no_space_after_raising_the_floor last_refusal={} floor_raises={}",
            publish_error_name(&no_space.last_refusal),
            no_space.floor_raises.len()
        ),
        Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed)) => format!(
            "refused_floor_raise_failed floor_raises={}",
            failed.floor_raises.len()
        ),
        Err(other) => format!("refused_other {other:?}"),
    }
}

fn publish_error_name(error: &PublishError) -> String {
    match error {
        PublishError::JournalRecordsOfThePublishExceedTheLimit {
            records_of_the_publish,
            record_limit,
        } => format!("journal_records_exceed_the_limit({records_of_the_publish}>{record_limit})"),
        PublishError::SpaceAdmissionRefused(_) => "space_admission_refused".to_string(),
        PublishError::PlacementRefused { .. } => "placement_refused".to_string(),
        other => {
            let debug = format!("{other:?}");
            debug.split(['(', ' ', '{']).next().unwrap_or("unknown").to_string()
        }
    }
}

/// 今天：单元区从 4000 槽往下按 32 槽一步扫，取头一个「第一个文件之后式子那一判每块盘的可用少于 86 个数据单元的 172 槽」的几何——
/// 这次顺序写的确切需求至少是这 172 槽（另加 extent、inode 节点与固定点），式子在任何差量上界下都判不够；而它切出 86 条记录、多于上限 85。
/// 今天的次序是落点那一判 → 全量规划（固定点）→ 记录条数 → 合取 7 → 式子（`transaction::judge_the_publish_before_any_read_or_write`），
/// 会话交回记录超上限，一串抬 F 都不推，盘上逐字节不变、F 不动。A3 把式子挪到全量规划之前、记录条数还要全量规划才数得出，
/// 这一条在 A3 模拟上红：会话先见到空间不够、推抬 F。
#[test]
fn a_write_past_the_journal_limit_on_a_pool_short_by_the_formula_is_refused_for_its_records_without_raising_the_floor(
) {
    let limit = journal_record_limit_of_one_publish(JOURNAL_RING_OF_ONE_MEBIBYTE);
    assert_eq!(limit, DATA_UNITS_OF_THE_WRITE_AT_THE_LIMIT, "256 ÷ 3");
    let unit_area_start = unit_area_start_slot_on_the_one_mebibyte_ring();
    let demand_lower_bound_slots =
        i128::from(DATA_UNITS_OF_THE_WRITE_PAST_THE_LIMIT * SLOTS_OF_A_DATA_UNIT);
    let mut unit_area_slots = UNIT_AREA_SLOTS_SCANNED_FROM;
    let mut geometries_built: u64 = 0;
    // 迭代上界：(4000 − 1100) ÷ 32 + 1 = 91 个几何；每轮单元区少 32 槽。
    let mut pool = loop {
        assert!(
            unit_area_slots >= UNIT_AREA_SLOTS_SCANNED_DOWN_TO,
            "扫到单元区 {UNIT_AREA_SLOTS_SCANNED_DOWN_TO} 槽都没找到式子不够 172 槽的几何（建了 {geometries_built} 个）"
        );
        let pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
            parameters_on_the_one_mebibyte_ring(),
            (unit_area_start + unit_area_slots) * SLOT_BYTES,
            plain_device,
        );
        geometries_built += 1;
        let available = smallest_formula_available_slots(&pool);
        println!(
            "name=reorder_scan unit_area_slots={unit_area_slots} smallest_formula_available_slots={available}"
        );
        if available < demand_lower_bound_slots {
            break pool;
        }
        unit_area_slots -= UNIT_AREA_SLOTS_SCAN_STEP;
    };
    let available = smallest_formula_available_slots(&pool);
    assert!(
        available >= 0,
        "取到的几何上式子那一判还没到负数：{available} 槽"
    );
    let images_before = pool.image().devices;
    let floor_before = pool.rollback_floor();
    let outcome = pool.sequential_write(content_bytes_of_a_sequential_write_into(
        DATA_UNITS_OF_THE_WRITE_PAST_THE_LIMIT,
    ));
    let line = outcome_line(&outcome);
    let images_unchanged = pool.image().devices == images_before;
    let floor_after = pool.rollback_floor();
    println!(
        "name=reorder_probe unit_area_slots={unit_area_slots} geometries_built={geometries_built} \
         smallest_formula_available_slots={available} demand_lower_bound_slots={demand_lower_bound_slots} \
         outcome={line} images_unchanged={images_unchanged} floor_before={floor_before:?} floor_after={floor_after:?}"
    );
    assert!(
        matches!(
            &outcome,
            Err(UserChangeRefused::Publish {
                cause: PublishError::JournalRecordsOfThePublishExceedTheLimit {
                    records_of_the_publish: 86,
                    record_limit: 85,
                },
                floor_raises,
                ..
            }) if floor_raises.is_empty()
        ),
        "今天：记录超上限排在式子之前，会话原样交回、不推抬 F：{line}"
    );
    assert!(images_unchanged, "两块盘逐字节不变");
    assert_eq!(floor_after, floor_before, "F 不动");
}

/// 今天：1 MiB 环、4 GiB 盘上第一个文件之后顺序写 85 个数据单元照发（与 `core_review_unit_area_start_and_publish_limits.rs` 的
/// `a_sequential_write_cut_into_as_many_journal_records_as_the_in_flight_limit_is_published` 同形，这里经会话发）。
/// 记录条数若照 A3 的口径改成在全量规划之前由上界数（固定点那一截取 ckpt_cost，代码里罩固定点的那个上界），末条要点名的项
/// 至少是最后那个数据单元加 ckpt_cost 个固定点角色，按一条 64 项切，条数 = 84 + ⌈(1 + ckpt_cost) ÷ 64⌉，多于上限 85：这次写会被拒成
/// 记录超上限（不是空间不够，会话不推抬 F，调用方被叫去把改动切小）。今天照发、上界口径拒，两条都在这里判。
#[test]
fn a_write_at_the_journal_limit_is_published_today_and_a_fixed_point_bound_from_the_checkpoint_cost_would_cut_it_past_the_limit(
) {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_on_the_one_mebibyte_ring(),
        4 << 30,
        plain_device,
    );
    let checkpoint_cost =
        checkpoint_cost_of_the_version_to_build_on(Some(pool.current_file_version()), &pool.session().allocator).0;
    let named_entries_per_record = u64::try_from(
        JournalRecordNamedEntryCapacity::FromTheRecordFormat.named_entries_per_record(),
    )
    .expect("64");
    let records_under_the_bound = (DATA_UNITS_OF_THE_WRITE_AT_THE_LIMIT - 1)
        + (1 + checkpoint_cost).div_ceil(named_entries_per_record);
    let limit = journal_record_limit_of_one_publish(JOURNAL_RING_OF_ONE_MEBIBYTE);
    let outcome = pool.sequential_write(content_bytes_of_a_sequential_write_into(
        DATA_UNITS_OF_THE_WRITE_AT_THE_LIMIT,
    ));
    let line = outcome_line(&outcome);
    println!(
        "name=bounded_journal_count checkpoint_cost={checkpoint_cost} named_entries_per_record={named_entries_per_record} \
         records_under_the_bound={records_under_the_bound} record_limit={limit} today_outcome={line}"
    );
    assert!(outcome.is_ok(), "今天 85 条记录不多于上限 85、照发：{line}");
    assert!(
        records_under_the_bound > limit,
        "按 ckpt_cost 取固定点上界切记录：{records_under_the_bound} 条，上限 {limit}"
    );
}

/// 今天：第一个文件之后接连 40 次建 inode 的发布，每一次记账树的行数多 1（这次的 inode 号水位台阶行，随根 COW、一直留着），
/// ckpt_cost 的记账树那一项（`admission::accounting_tree_nodes_after_the_publish_built_on`）每次发布前逐行过滤、把台阶行全收进一个集合再交规划器——
/// 这一步的代价随这类发布的次数涨，四条候选的定义（这一次的规划、这一次的分配量、上一版的表示）都没点到它。
#[test]
fn every_inode_creating_publish_leaves_one_more_accounting_row_that_the_checkpoint_cost_walks() {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_on_the_one_mebibyte_ring(),
        4 << 30,
        plain_device,
    );
    let rows_at_the_start = pool.current_file_version().accounting_entries.len();
    let steps_at_the_start = pool.current_file_version().inode_number_watermark_steps().len();
    let mut rows_before = rows_at_the_start;
    // 迭代上界：40 次建 inode。
    for publish in 1..=INODE_CREATING_PUBLISHES {
        pool.create_one_inode().expect("4 GiB 盘上建一个 inode 照发");
        let rows_after = pool.current_file_version().accounting_entries.len();
        assert_eq!(
            rows_after,
            rows_before + 1,
            "建 inode 的发布做到 {publish} 次之后记账行多 1：{rows_before} → {rows_after}"
        );
        rows_before = rows_after;
    }
    let version = pool.current_file_version();
    println!(
        "name=accounting_rows publishes={INODE_CREATING_PUBLISHES} rows_at_the_start={rows_at_the_start} rows_at_the_end={} \
         watermark_steps_at_the_start={steps_at_the_start} watermark_steps_at_the_end={} accounting_tree_nodes={}",
        version.accounting_entries.len(),
        version.inode_number_watermark_steps().len(),
        version.accounting_tree.node_count()
    );
}

/// 量的是：一次写入类发布真写出的固定点槽数（这次重写的角色里归 checkpoint 保留池那几类，`admission::space_budget_of_role`）
/// 与它接着的那一版的 ckpt_cost（`checkpoint_cost_of_the_version_to_build_on_with_node_capacities`，一次空发布至多写出的固定点）相比多多少。
/// 今天的落点那一判拿 ckpt_cost 顶固定点（`admission::units_of_a_publish_to_land`），之后还有走完固定点的式子那一判按真数判；
/// A3 若沿用这个数当固定点的上界，又把按真数的那一判挪走，多出来的那几槽就是「上界放行、真实不够」的窗口。
/// 两档节点容量：产品容量与只供测试压小的中央映射树（叶 4、内部 8，`CodeTwoTreeNodeCapacities::CappedForTests`）。
/// 窄盘（1 MiB 环、单元区 2000 槽：分配记录树每盘 3 片叶，K1 = 7）上第一个文件之后顺序写 N 个数据单元。这一条只打数，不判多少。
#[test]
fn the_fixed_point_a_write_publishes_against_the_checkpoint_cost_of_the_version_it_builds_on() {
    use singlefs_core::address::DeviceIdentity;
    use singlefs_core::admission::{
        checkpoint_cost_of_the_version_to_build_on_with_node_capacities, space_budget_of_role,
        SpaceBudgetOfARole,
    };
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::code_two_tree::CodeTwoTreeNodeCapacity;
    use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
    use singlefs_core::transaction::{
        acquire_instance, publish_first_file, publish_sequential_write, warm_up,
        CodeTwoTreeNodeCapacities, FirstFile, PoolWriter,
    };
    use singlefs_harness::memory_pool::SparseBlockDevice;

    const UNIT_AREA_SLOTS: u64 = 2000;
    const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
    let capped_mapping = CodeTwoTreeNodeCapacities::CappedForTests {
        accounting: CodeTwoTreeNodeCapacity {
            leaf_entries: 477,
            internal_entries: 150,
        },
        central_mapping: CodeTwoTreeNodeCapacity {
            leaf_entries: 4,
            internal_entries: 8,
        },
    };
    let device_bytes = (unit_area_start_slot_on_the_one_mebibyte_ring() + UNIT_AREA_SLOTS) * SLOT_BYTES;
    // 迭代上界：两档容量 × 五个 N。
    for (capacities_name, capacities) in [
        ("from_the_node_format", CodeTwoTreeNodeCapacities::FromTheNodeFormat),
        ("capped_mapping_4_8", capped_mapping),
    ] {
        for data_units in [1_u64, 10, 20, 40, DATA_UNITS_OF_THE_WRITE_AT_THE_LIMIT] {
            let parameters = parameters_on_the_one_mebibyte_ring();
            let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0_u32, 1]
                .into_iter()
                .map(|number| {
                    (
                        DeviceIdentity(number),
                        SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                    )
                })
                .collect();
            let genesis = make_filesystem(&parameters, &mut devices).expect("窄盘上 mkfs");
            let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            writer.set_code_two_tree_node_capacities(capacities);
            let instance = acquire_instance(&mut writer).expect("取号 1");
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            let first_content = vec![7_u8; 3000];
            let first = publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机写了两条根"),
                FirstFile {
                    content: &first_content,
                    write_time_seconds: WRITE_TIME_SECONDS,
                },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("第一个文件");
            let checkpoint_cost =
                checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
                    Some(&first),
                    &allocator,
                    capacities,
                )
                .0;
            let content = vec![9_u8; content_bytes_of_a_sequential_write_into(data_units)];
            let outcome = publish_sequential_write(
                &mut writer,
                &mut allocator,
                &first,
                FirstFile {
                    content: &content,
                    write_time_seconds: WRITE_TIME_SECONDS + 1,
                },
                instance,
            );
            match outcome {
                Ok(published) => {
                    let fixed_point_slots: u64 = published
                        .rewritten
                        .iter()
                        .filter(|role| {
                            space_budget_of_role(**role) == SpaceBudgetOfARole::CheckpointReservePool
                        })
                        .map(|role| role.span_slots())
                        .sum();
                    let excess = i128::from(fixed_point_slots) - i128::from(checkpoint_cost);
                    println!(
                        "name=fixed_point_vs_checkpoint_cost capacities={capacities_name} data_units={data_units} \
                         checkpoint_cost_of_the_previous_version={checkpoint_cost} fixed_point_slots_written={fixed_point_slots} \
                         excess_slots={excess} mapping_tree_nodes_after={}",
                        published.central_mapping_tree.node_count()
                    );
                }
                Err(refused) => println!(
                    "name=fixed_point_vs_checkpoint_cost capacities={capacities_name} data_units={data_units} \
                     checkpoint_cost_of_the_previous_version={checkpoint_cost} outcome=refused({})",
                    publish_error_name(&refused)
                ),
            }
        }
    }
}
