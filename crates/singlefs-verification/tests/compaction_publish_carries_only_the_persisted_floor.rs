//! 整理的发布只带持久 F、不带内存里抬过的在 core 上的样子。
//! 协议核那一侧是 `singlefs_replay_prefix::gc_ring_interval_proofs` 的
//! `lemma_a_compaction_publish_over_a_floor_raised_only_in_memory_loses_one_candidate_across_a_crash` 与 G7（连崩溃，
//! `theorem_compaction_keeps_what_the_user_sees`）：同一串用户历史插 k 次整理的发布、之后崩溃，回退候选（按用户的版本点名）是不插整理那一份的后缀、
//! 至多少 k 个，现行内容相同。
//!
//! core 里整理的发布（`transaction::publish_relocation_batch`）带上一版的根带着的 F（`publish_compaction_step_after_its_check` 那一处
//! `rollback_floor: previous.root.rollback_floor`）。core 里没有「只在内存里抬过的 F」：抬 F 那一串先把新 F 写进每块盘的系统配置、
//! 过屏障，再推带新 F 的空发布（`mount::raise_rollback_floor`），崩溃之后的 F 生效值连系统配置一起取（`recovery::effective_rollback_floor`）。
//! 所以这里造的是协议核那个例子在 core 上写得出的一形：根环转满一圈的用户覆盖写，中间插一次整理的发布，崩溃，按盘上读出的根环与 F 生效值
//! 数用户版本的回退候选；整理的发布要是把 F 抬过上一版的根带着的，候选就少不止 k 个。
//! 两份池都是内存盘（同一个进程里 mkfs、取号、暖机、第一个文件），「崩溃」就是丢掉进程里的分配器与现行那一版，只按盘上的字节读。
//! 期望值都从被测之前现算（每一版发布交回的 txg），读回一律从盘上。

mod common;

use common::{parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, InstanceGeneration,
};
use singlefs_core::allocator::{Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::PlannedRelocation;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::recovery::{effective_rollback_floor, readable_roots, walk_to_file};
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file,
    publish_overwrite, publish_relocation_batch, warm_up, FirstFile, MappingLookup, PoolWriter,
    TransactionOutput, TransactionUnit,
};
use singlefs_verification::memory_pool::SparseBlockDevice;

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

/// 第一个文件之后用户覆盖写的次数：根环（3 个区域、每区域 8 槽，共 24 个根）转过一圈，不插整理那一份的根环里 24 个根全是带文件的用户版本
/// （第 1 版到第 24 版；第一个文件是第 0 版，已被挤出根环）。
const USER_OVERWRITES: usize = 24;

/// 整理的发布插在用户第几次覆盖写之前：插在中间，之后的用户发布接着带它那一版的 F。
const USER_OVERWRITES_BEFORE_THE_COMPACTION_PUBLISH: usize = 12;

/// 这一份历史插不插那一次整理的发布。
#[derive(Clone, Copy, PartialEq, Eq)]
enum CompactionPublishInTheHistory {
    Inserted,
    NotInserted,
}

/// 一份历史崩溃之后从盘上读出来的东西。
struct HistoryReadAfterTheCrash {
    /// 回退候选里的用户版本（第几版），从旧到新：根环里有那一版的根、它的 txg 不低于 F 生效值。
    candidate_user_versions: Vec<usize>,
    effective_floor: CheckpointTxg,
    /// 崩溃之后重现的现行内容：根环里最新那条根下面走到的文件。
    newest_content: Vec<u8>,
    /// 整理那一次发布的根带的 F 与上一版的根带的 F；没插整理的发布时是 `None`。
    floors_of_the_compaction_publish_and_its_previous_root: Option<(CheckpointTxg, CheckpointTxg)>,
}

/// 同一个进程里还开着的一份内存池：两块盘、分配器、现行那一版。
struct PoolInThisProcess {
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    output: TransactionOutput,
}

/// 新池：mkfs、取号（实例 1）、暖机、第一个文件（第 0 版）。
fn pool_with_the_first_file() -> PoolInThisProcess {
    let pool_parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    let genesis = make_filesystem(&pool_parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&pool_parameters, &devices, &genesis);
    let first_content = content_of_user_version(0);
    let output = {
        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, INSTANCE, "新池取到的是实例 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写了两条根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    PoolInThisProcess {
        devices,
        allocator,
        output,
    }
}

/// 用户第 `user_version` 版的内容：每一版不同，装得进一个数据单元。
fn content_of_user_version(user_version: usize) -> Vec<u8> {
    format!("user version {user_version} of the first file").into_bytes()
}

/// `version` 里第一个文件的数据单元的映射 key 与它的源落点（按映射位置）：整理那一批搬它的计划项。
fn planned_relocation_of_the_data_unit(version: &TransactionOutput) -> PlannedRelocation {
    let identity = TransactionUnit::Data(DataUnitIndexInFile::FIRST);
    let (_, mapping_key) = version
        .mapped_units
        .iter()
        .find(|(role, _)| *role == identity)
        .expect("第一个文件的数据单元进映射");
    let MappingLookup::Found(locations) =
        central_mapping_locations_in_the_version(version, mapping_key)
    else {
        panic!("第一个文件的数据单元在这一版的映射里查得到");
    };
    PlannedRelocation {
        mapping_key: mapping_key.clone(),
        source: Placement {
            slot: locations[0].slot,
            span: identity.span_slots(),
        },
    }
}

/// 整理的一次发布（搬第一个文件的数据单元），接在 `pool.output` 上；交回它的根带的 F 与上一版的根带的 F。
fn publish_the_compaction_batch(pool: &mut PoolInThisProcess) -> (CheckpointTxg, CheckpointTxg) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let planned = planned_relocation_of_the_data_unit(&previous);
    let mut writer = PoolWriter::new(&publish_parameters, pool.devices.as_mut_slice());
    let relocated = publish_relocation_batch(
        &mut writer,
        &mut pool.allocator,
        &previous,
        std::slice::from_ref(&planned),
        INSTANCE,
    )
    .expect("整理的一批");
    let floors = (relocated.root.rollback_floor, previous.root.rollback_floor);
    pool.output = relocated;
    floors
}

/// 用户第 `user_version` 版的覆盖写，接在 `pool.output` 上；交回它的 txg。
fn publish_user_version(pool: &mut PoolInThisProcess, user_version: usize) -> CheckpointTxg {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let content = content_of_user_version(user_version);
    let mut writer = PoolWriter::new(&publish_parameters, pool.devices.as_mut_slice());
    let overwritten = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS
                + u64::try_from(user_version).expect("版本数装得进 u64"),
        },
        INSTANCE,
    )
    .expect("覆盖写");
    let txg = overwritten.root.checkpoint_txg;
    pool.output = overwritten;
    txg
}

/// 建池、第一个文件（第 0 版），再覆盖写 `USER_OVERWRITES` 次（第 1 版起），`compaction_publish` 是 `Inserted` 时在第
/// `USER_OVERWRITES_BEFORE_THE_COMPACTION_PUBLISH` 次之前插一次整理的发布；然后崩溃（不卸载、不再发布，丢掉进程里的分配器与现行那一版），
/// 从盘上读根环与 F 生效值。
fn run_the_history_and_read_it_after_a_crash(
    compaction_publish: CompactionPublishInTheHistory,
) -> HistoryReadAfterTheCrash {
    let mut pool = pool_with_the_first_file();
    let mut user_version_txgs = vec![pool.output.root.checkpoint_txg];
    let mut floors_of_the_compaction_publish_and_its_previous_root = None;
    for user_version in 1..=USER_OVERWRITES {
        if user_version == USER_OVERWRITES_BEFORE_THE_COMPACTION_PUBLISH + 1
            && compaction_publish == CompactionPublishInTheHistory::Inserted
        {
            floors_of_the_compaction_publish_and_its_previous_root =
                Some(publish_the_compaction_batch(&mut pool));
        }
        user_version_txgs.push(publish_user_version(&mut pool, user_version));
    }
    let devices = pool.devices;
    let pool_parameters = parameters();
    let roots_in_the_ring = readable_roots(
        &devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    let effective_floor = effective_rollback_floor(
        &devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    let candidate_user_versions = user_version_txgs
        .iter()
        .enumerate()
        .filter(|(_, user_version_txg)| {
            **user_version_txg >= effective_floor
                && roots_in_the_ring.iter().any(|root| {
                    root.instance == INSTANCE && root.checkpoint_txg == **user_version_txg
                })
        })
        .map(|(user_version, _)| user_version)
        .collect();
    let newest_root = roots_in_the_ring
        .iter()
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .expect("根环里读得出根");
    let newest_content = walk_to_file(&devices, newest_root, &mut 0)
        .expect("最新那条根下面走得到文件")
        .expect("最新那一版带文件");
    HistoryReadAfterTheCrash {
        candidate_user_versions,
        effective_floor,
        newest_content,
        floors_of_the_compaction_publish_and_its_previous_root,
    }
}

/// 同一串用户覆盖写，一份不插整理、一份在中间插一次整理的发布（k = 1），都在最后崩溃。不插整理那一份根环里是第 1 版到第 24 版；
/// 插了的那一份整理的根占根环一槽，挤掉最旧的第 1 版，候选是第 2 版到第 24 版：是不插整理那一份的后缀、恰少 1 个；F 生效值与现行内容都相同，
/// 整理的根带的 F 就是上一版的根带的。
/// 判别力：整理的发布把 F 抬到上一版的 txg（`crates/mutations.tsv` 里那一行），之后的用户发布接着带它，崩溃之后的 F 生效值就是它，
/// 候选只剩第 12 版起的 13 个，少 11 个。
#[test]
fn a_compaction_publish_carries_the_floor_of_the_previous_root_so_after_a_crash_the_candidates_lose_only_the_version_it_pushed_out_of_the_ring(
) {
    let without_compaction =
        run_the_history_and_read_it_after_a_crash(CompactionPublishInTheHistory::NotInserted);
    let with_compaction =
        run_the_history_and_read_it_after_a_crash(CompactionPublishInTheHistory::Inserted);
    assert_eq!(
        without_compaction.candidate_user_versions,
        (1..=USER_OVERWRITES).collect::<Vec<usize>>(),
        "不插整理那一份：根环转过一圈，24 个根全是用户版本，F 没抬过，第 1 版到第 24 版都是候选"
    );
    assert!(
        without_compaction
            .candidate_user_versions
            .ends_with(&with_compaction.candidate_user_versions),
        "插了整理的那一份崩溃之后的候选（{:?}）是不插整理那一份（{:?}）的后缀",
        with_compaction.candidate_user_versions,
        without_compaction.candidate_user_versions
    );
    assert_eq!(
        without_compaction
            .candidate_user_versions
            .len()
            .checked_sub(with_compaction.candidate_user_versions.len()),
        Some(1),
        "只少 k = 1 个：整理的根占了根环一槽，挤掉最旧的那一版；插了整理的那一份的候选是 {:?}",
        with_compaction.candidate_user_versions
    );
    assert_eq!(
        with_compaction.effective_floor, without_compaction.effective_floor,
        "崩溃之后的 F 生效值相同：整理的发布不抬 F"
    );
    assert_eq!(
        with_compaction.newest_content, without_compaction.newest_content,
        "崩溃之后重现的现行内容相同"
    );
    let (floor_of_the_compaction_publish, floor_of_its_previous_root) = with_compaction
        .floors_of_the_compaction_publish_and_its_previous_root
        .expect("这一份插了整理的发布");
    assert_eq!(
        floor_of_the_compaction_publish, floor_of_its_previous_root,
        "整理的发布带上一版的根带着的 F"
    );
}
