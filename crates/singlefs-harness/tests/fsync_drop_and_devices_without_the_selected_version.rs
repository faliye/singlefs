//! 里程碑「覆盖写、释放、回退与复用」增补 2（调度记录 `records/2026-09-24-里程碑二收尾调度.md` 第三节「fsync 失败掉盘与 C542」
//! 「fsync 失败掉盘的核查」两行）：两块盘的池上，一块盘的 fsync（写屏障、FUA、系统配置轮换写）失败之后，池不许退成只剩一块盘可用。
//!
//! 前半是核查时的用例（原样在 `research/prompts/fsync-drop-investigation/fsync_drop_investigation.rs`）搬进来：
//! - 写路径那一半：36 格 fsync 失败历史（六个注入点 × 三种持续档 × 故障前重开与否），每一格判另发一次被冻结拦住、一次性故障
//!   原样重发做成、故障消失之后可写挂载做成、最后那一版每个单元两块盘各一份、池级 checker 不红、冷重开读回最后确认的一版
//!   （「屏障报错丢脏页」那一档的镜像同判）；
//! - 挂载那一半：盘 1 缺席、盘 1 是空盘、盘 1 停在旧状态三格。可写挂载要么拒、要么挂上之后这一版每个单元两块盘各一份；
//!   盘 1 缺席那一格按 w 的下限拒（`MountError::WritableDeviceCountBelowTheStripeWidthLowerBound`）。管理员回退改成挂着时的
//!   一次向前发布之后不再是一次挂载（D23（journal 的角色与格式） 已定项 14），设备准入在它之前那次可写挂载判过，回退那两条删了。
//!   另有读路径一格：盘 1 每次读都报错时可写挂载拒成哪个成员。
//!
//! 后半钉可写挂载准入的逐盘核（D2（RAID 条带策略） 已定项 13「挂载准入」、已定项 6 `w ≥ 2`；D18（块里携带什么信息） 已定项 11
//! 「可写挂载的顺序」里的「可见」与「前提」里的「错过取号的盘整盘作废、只读到重同步完成」）：空盘、停在旧状态的盘拒可写、
//! 报 `MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`、盘上逐字节不变；只读挂载照旧读得回所选那一版。
//!
//! 设备栈：`BarrierFailureLog`（记下每一次屏障报错时录制流有多长）→ `FaultInjectingBlockDevice`（按计划报错）
//! → `RecordingBlockDevice`（只记真落下去的）→ `SparseBlockDevice`（内存盘，没有缓存）。
//! 冷重开的镜像从录制流重建。「fsync 报错丢脏页」那一档（Linux 缓冲写回失败之后页被标干净、内容留在缓存里、盘上没有）
//! 另建一份镜像：每一次屏障在盘 1 上报错，就把盘 1 自上一次成功刷盘（池屏障，或盘 1 上的 FUA 写）以来的普通写从镜像里拿掉。
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/fsync_drop_and_devices_without_the_selected_version.rs"]
mod scenarios_of_fsync_drop_and_devices_without_the_selected_version;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_fsync_drop_and_devices_without_the_selected_version::{
    blank_device, build_rig, content_of,
    full_image_without_the_last_system_configuration_write_on_device_one, image_of,
    journal_position, memory_pool_of, newest_system_configuration_journal_tail_on, overwrite,
    panic_text, pool_at_txg_five_with_device_one_stale_at_txg_three, rig_after_make_filesystem,
    sparse_from, PoolAtTxgFiveWithDeviceOneStaleAtTxgThree, DEVICE_ONE, DEVICE_ZERO, IMAGE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes,
    InodeNumber, InstanceGeneration, JournalSequenceNumber, SlotNumber,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::mount::{
    mount_writable, DeviceCount, DeviceWithoutTheSelectedVersion, MountError, NewerPublishWitness,
    RollbackTarget, SelectedVersionAgainstTheWitness, SelectedVersionLackingOnDevice,
    StillUnreadableAfterOneReread, SystemConfigurationUnreadableAfterTheReadStage,
    SystemConfigurationUnreadableWhen, WitnessedCounterComparison, STRIPE_WIDTH_LOWER_BOUND,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{allocation_records_under_root, PoolReader, RecoveryFailure};
use singlefs_core::transaction::{PoolVersion, TransactionUnit, FIRST_INODE_NUMBER};
use singlefs_format::DATA_UNIT_BYTES;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultOccurrence, FaultPlacement, FaultSchedule,
    InjectedFault,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use std::cell::Cell;
use std::collections::BTreeSet;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_one_shot`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn fsync_drop_one_shot() {
    scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_one_shot(
        &mut PoolCheckerImages::left_to_the_checker_tier(),
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_persists_until_reopen`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn fsync_drop_persists_until_reopen() {
    scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_persists_until_reopen(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_persists_across_reopen`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn fsync_drop_persists_across_reopen() {
    scenarios_of_fsync_drop_and_devices_without_the_selected_version::fsync_drop_persists_across_reopen(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 数自己被读了几次的盘：读写原样转给内存盘。钉「交进来的盘数不够就在读任何一块盘之前拒」。
struct ReadCountingDevice {
    inner: SparseBlockDevice,
    reads: Cell<u64>,
}

impl BlockDevice for ReadCountingDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.reads.set(self.reads.get() + 1);
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 只把盘 0（txg 5 那一版的整池镜像上的那一块）交进来，外面包一层数读的。
fn only_device_zero_counting_reads(
    pool: &PoolAtTxgFiveWithDeviceOneStaleAtTxgThree,
) -> Vec<(DeviceIdentity, ReadCountingDevice)> {
    vec![(
        DEVICE_ZERO,
        ReadCountingDevice {
            inner: sparse_from(&pool.full, DEVICE_ZERO),
            reads: Cell::new(0),
        },
    )]
}

/// 交进来的盘少于 w 的下限、在读任何一块盘之前被拒：成员是「可写设备数低于 w 的下限」，带交进来的 1 块与下限 2；
/// 那块盘一次都没被读、逐字节不变。
fn assert_refused_below_the_stripe_width_lower_bound_before_any_read(
    error: &MountError,
    only_device_zero: &[(DeviceIdentity, ReadCountingDevice)],
    before: &SparseDevice,
    what: &str,
) {
    let MountError::WritableDeviceCountBelowTheStripeWidthLowerBound {
        devices_handed_in,
        stripe_width_lower_bound,
    } = error
    else {
        panic!("{what}：该报 WritableDeviceCountBelowTheStripeWidthLowerBound，实际 {error:?}")
    };
    assert_eq!(
        (*devices_handed_in, *stripe_width_lower_bound),
        (DeviceCount(1), DeviceCount(2)),
        "{what}：交进来 1 块盘，w 的下限是 2（D2（RAID 条带策略） 已定项 6「`w ≥ 2` 是硬下界」）"
    );
    assert_eq!(
        STRIPE_WIDTH_LOWER_BOUND,
        DeviceCount(2),
        "第一版两块盘镜像，w 的下限是 2"
    );
    let [(device, counting)] = only_device_zero else {
        panic!("{what}：只交了盘 0")
    };
    assert_eq!(*device, DEVICE_ZERO);
    assert_eq!(
        counting.reads.get(),
        0,
        "{what}：拒在选系统配置、恢复、重建与读分配记录树之前，盘 0 一次都没被读"
    );
    assert_eq!(
        &counting.inner.image, before,
        "{what}：拒在任何写之前，盘 0 逐字节不变"
    );
}

/// 盘 1 缺席（只把盘 0 交进来）：可写挂载在读任何一块盘之前拒掉，报「可写设备数低于 w 的下限」
/// （D2（RAID 条带策略） 已定项 13 挂载准入的第一条合取、已定项 6 的 `w ≥ 2`；D18（块里携带什么信息） 已定项 11「可写挂载的顺序」先判这一条），
/// 不靠分配记录树按一块盘的几何重建碰巧报 I-1.1。对照：同一个池两块盘都交，可写挂载照旧做成、接在 txg 5 那一版后面。
#[test]
fn mount_writable_with_only_device_zero_is_refused() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let mut only_device_zero = only_device_zero_counting_reads(&pool);
    let before = pool.full.devices[&DEVICE_ZERO].clone();
    let refusal = mount_writable(&pool.parameters, &mut only_device_zero)
        .expect_err("盘 1 缺席时可写挂载必须拒绝，不许单盘可写");
    assert_refused_below_the_stripe_width_lower_bound_before_any_read(
        &refusal,
        &only_device_zero,
        &before,
        "只把盘 0 交给可写挂载",
    );

    let mut both_devices = vec![
        (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
        (DEVICE_ONE, sparse_from(&pool.full, DEVICE_ONE)),
    ];
    let mounted = mount_writable(&pool.parameters, &mut both_devices)
        .unwrap_or_else(|error| panic!("两块盘都交的健康池，可写挂载照旧做成，实际 {error:?}"));
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        CheckpointTxg(5),
        "两块盘都交：接在 txg 5 那一版后面"
    );
}

/// 同一块盘交两次（盘表两项都标成盘 0，都是 txg 5 那一版整池镜像上的盘 0）：可写设备数数的是不同的设备身份，只有 1 块，
/// 可写挂载在读任何一块盘之前拒成「可写设备数低于 w 的下限」（D2（RAID 条带策略） 已定项 13 挂载准入第一条合取；实二八交回第 2 问，
/// 调度记录实四那一项「数不同的设备身份」）。两项都没被读、逐字节不变。按盘表项数数是 2，这一判就放过去了。
#[test]
fn mount_writable_with_device_zero_handed_in_twice_counts_one_device_and_is_refused_before_any_read(
) {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let mut device_zero_twice: Vec<(DeviceIdentity, ReadCountingDevice)> = (0..2)
        .map(|_| {
            (
                DEVICE_ZERO,
                ReadCountingDevice {
                    inner: sparse_from(&pool.full, DEVICE_ZERO),
                    reads: Cell::new(0),
                },
            )
        })
        .collect();
    let before = pool.full.devices[&DEVICE_ZERO].clone();
    let refusal = mount_writable(&pool.parameters, &mut device_zero_twice)
        .expect_err("同一块盘交两次也只是一块盘，可写挂载必须拒绝");
    let MountError::WritableDeviceCountBelowTheStripeWidthLowerBound {
        devices_handed_in,
        stripe_width_lower_bound,
    } = refusal
    else {
        panic!(
            "盘 0 交两次：该报 WritableDeviceCountBelowTheStripeWidthLowerBound，实际 {refusal:?}"
        )
    };
    assert_eq!(
        (devices_handed_in, stripe_width_lower_bound),
        (DeviceCount(1), STRIPE_WIDTH_LOWER_BOUND),
        "盘表两项、不同的设备身份 1 个：数的是 1 块盘"
    );
    for (index, (identity, counting)) in device_zero_twice.iter().enumerate() {
        assert_eq!(*identity, DEVICE_ZERO);
        assert_eq!(
            counting.reads.get(),
            0,
            "盘表第 {index} 项：拒在读任何一块盘之前"
        );
        assert_eq!(
            counting.inner.image, before,
            "盘表第 {index} 项：拒在任何写之前，逐字节不变"
        );
    }
}

/// `crates/mutations.tsv` 里「只交盘 0 时分配记录树那一判 panic」那一行（实二七）原先靠可写挂载只交盘 0 走到分配记录树的读者；
/// 实二八让可写挂载在读盘之前就拒了，那一行点名的用例走不到它（实二八交回第 1 问）。这里从公开入口直接喂一个只有盘 0 的读者：
/// 按一块盘的几何读两块盘写的分配记录树，根里指盘 1 那一格的内部条目 key 在这个几何里不是任何一个孩子那一段的起点
/// ⇒ 交回 I-1.1 的错，不 panic（盘上读来的 key 可以是任何值，`allocation_record_tree::read_allocation_record_tree` 的文档注释）。
#[test]
fn reading_the_allocation_record_tree_with_the_reader_of_only_device_zero_reports_the_child_key_violation_instead_of_panicking(
) {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let mut only_device_zero = MemoryPool::with_devices(&[DEVICE_ZERO], IMAGE_BYTES);
    only_device_zero
        .devices
        .insert(DEVICE_ZERO, pool.full.devices[&DEVICE_ZERO].clone());
    let newest_root = pool.versions[2].root;
    assert_eq!(newest_root.checkpoint_txg, CheckpointTxg(5));
    let both_devices_read = allocation_records_under_root(&pool.full, &newest_root)
        .expect("两块盘都在的读者读得出 txg 5 那一版的分配记录树");
    assert!(
        both_devices_read
            .iter()
            .any(|record| record.device == DEVICE_ONE),
        "对照：两块盘写的树里有盘 1 的记录，根里有指盘 1 那一格的内部条目"
    );
    let refusal = catch_unwind(AssertUnwindSafe(|| {
        allocation_records_under_root(&only_device_zero, &newest_root)
    }))
    .unwrap_or_else(|payload| {
        panic!(
            "只有盘 0 的读者读分配记录树不许 panic：{}",
            panic_text(payload.as_ref())
        )
    });
    assert_eq!(
        refusal,
        Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.1",
            detail: "分配记录树内部条目的 key 不是这个节点里一个孩子那一段的起点",
        }),
        "按一块盘的几何读：指盘 1 那一格的 key 不是这个几何里任何孩子那一段的起点"
    );
}

/// 只交一块盘的只读挂载（D2（RAID 条带策略） 已定项 13「降级期间只读」：可写挂载准入不成立即只读挂载）：读者里只有盘 0
/// （txg 5 那一版整池镜像上的盘 0，盘 1 不在读者里——不是空盘，是读者列不出它）。根环区域 1 归盘 1、读不到就跳过，
/// 盘 0 上两个区域里最新的是 txg 5 那条根；只读挂载做成、沿 (1, 5) 打开，读回第三版内容。一个字节都不写（读者是只读的）。
#[test]
fn read_only_mount_with_only_device_zero_handed_in_opens_the_newest_version_on_it_and_reads_it_back(
) {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let mut only_device_zero = MemoryPool::with_devices(&[DEVICE_ZERO], IMAGE_BYTES);
    only_device_zero
        .devices
        .insert(DEVICE_ZERO, pool.full.devices[&DEVICE_ZERO].clone());
    assert_eq!(
        only_device_zero.device_identities(),
        vec![DEVICE_ZERO],
        "读者只列得出盘 0"
    );
    let mounted = mount_read_only(&only_device_zero).unwrap_or_else(|failure| {
        panic!("只交盘 0：只读挂载要做成（降级期间只读），实际 {failure:?}")
    });
    assert_eq!(
        version_key(
            mounted.effective_root.instance.0,
            mounted.effective_root.checkpoint_txg.0
        ),
        version_key(1, 5),
        "盘 0 上最新的根是 txg 5（根环区域 2 归盘 0），只读挂载沿它打开"
    );
    assert_eq!(
        mounted.journal.prefix_applied, 0,
        "txg 5 之上没有记录，一条都不施加"
    );
    let expected_content = content_of(3);
    let file = mounted
        .mounted
        .open_file(&only_device_zero, InodeNumber(FIRST_INODE_NUMBER))
        .unwrap_or_else(|failure| panic!("只交盘 0：文件要打得开，实际 {failure:?}"));
    let read = file
        .read_at(
            &only_device_zero,
            FileOffsetInBytes(0),
            u64::try_from(expected_content.len()).expect("4100"),
        )
        .unwrap_or_else(|failure| panic!("只交盘 0：文件要读得出，实际 {failure:?}"));
    assert_eq!(read.bytes, expected_content, "只交盘 0：读回第三版内容");
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_fsync_drop_and_devices_without_the_selected_version::mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies() {
    scenarios_of_fsync_drop_and_devices_without_the_selected_version::mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_fsync_drop_and_devices_without_the_selected_version::mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies() {
    scenarios_of_fsync_drop_and_devices_without_the_selected_version::mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 可写挂载被「有盘不带所选那一版」拒掉时交回的三样（`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`）。
struct RefusalByDevicesWithoutTheSelectedVersion {
    selected_version: RollbackTarget,
    selected_version_journal_position: JournalSequenceNumber,
    devices: Vec<DeviceWithoutTheSelectedVersion>,
}

fn refusal_by_devices_without_the_selected_version(
    error: &MountError,
    what: &str,
) -> RefusalByDevicesWithoutTheSelectedVersion {
    let MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
        selected_version,
        selected_version_journal_position,
        devices,
    } = error
    else {
        panic!(
            "{what}：该报 WritableMountRefusedByDevicesWithoutTheSelectedVersion，实际 {error:?}"
        )
    };
    RefusalByDevicesWithoutTheSelectedVersion {
        selected_version: *selected_version,
        selected_version_journal_position: *selected_version_journal_position,
        devices: devices.clone(),
    }
}

/// C554 乙先拒的那一格（`MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`）：
/// 读阶段交出的所选那一版是 `selected_version`、它那次发布末条记录读得出、计数器 `selected_version_last_record_counter`，另一块盘的系统配置
/// 见证过计数器 `witnessed_journal_counter`（更大）；重读一遍读数相同。乙在读阶段之后、取号之前逐盘核之前判，所以这几格走不到逐盘核
/// （调查 `research/prompts/m2-investigate-three-reds-report.md` 第二节）。这几格里有一块盘的系统配置槽读不出（每次读都报错、或空盘），
/// 另一块盘两槽都自证过：C331（择根倒挂压过已确认的写） 改取 R2 之后判得出（用户 2026-10-03 定），读数里的比较是拿见证值比所选那一版末条那一支；
/// 见证值是另一块盘读得出的最大 tail。
fn assert_refused_by_the_witness_of_a_newer_publish(
    error: &MountError,
    what: &str,
    selected_version: RollbackTarget,
    witnessed_journal_counter: u64,
    selected_version_last_record_counter: u64,
) {
    let reading = SelectedVersionAgainstTheWitness {
        selected_version,
        witness: NewerPublishWitness {
            witnessed_journal_counter,
            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                selected_version_last_record_counter,
            },
        },
    };
    assert!(
        matches!(
            error,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if **still_unreadable
                    == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                        first_read: reading,
                        reread: reading,
                    }
        ),
        "{what}：该被 C554 乙拒（见证 {witnessed_journal_counter} > 所选那一版末条 {selected_version_last_record_counter}），实际 {error:?}"
    );
}

fn version_key(instance: u32, txg: u64) -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(txg),
    }
}

/// txg 5 那一版里 txg 3 那一版没有的单元（txg 4、txg 5 写出的；停在 txg 3 之后的盘 1 一个都没收到），按角色与落点排好。
/// 从进程里两次发布交回的 `units` 算，不读盘。
fn units_written_after_txg_three(
    pool: &PoolAtTxgFiveWithDeviceOneStaleAtTxgThree,
) -> Vec<(TransactionUnit, SlotNumber)> {
    let [first, _, third] = &pool.versions;
    let mut written_after: Vec<(TransactionUnit, SlotNumber)> = third
        .units
        .iter()
        .filter(|unit| !first.units.contains(unit))
        .map(|unit| (unit.identity, unit.slot))
        .collect();
    written_after.sort_unstable();
    let roles: BTreeSet<TransactionUnit> = written_after.iter().map(|(role, _)| *role).collect();
    for role in [
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(0)),
        TransactionUnit::InodeRoot,
        TransactionUnit::TreeTable,
    ] {
        assert!(
            roles.contains(&role),
            "覆盖写每次都重写 {role:?}：txg 5 那一版的这一份不是 txg 3 那一版的"
        );
    }
    written_after
}

/// 空盘在盘 1（盘 0 带着 txg 5 那一版）：见证读缺盘 1 两槽，C331（择根倒挂压过已确认的写） 取甲之后判据按真，在逐盘核之前拒，
/// 两块盘逐字节不变；逐盘核点名空盘「没有自证过的系统配置」那一支经可写挂载走不到了（甲 先拒）。
/// 空盘在盘 0（盘 1 带着的最新一版是 txg 4：txg 5 的根在盘 0 上，它那条记录的点名单元盘 0 那一份验不过、施加不了）：
/// 盘 1 的系统配置见证过 txg 5 那次发布（jsn 5 > 所选那一版末条 4），C554 乙在逐盘核之前拒，逐盘核这一格不可达（C554 乙之后）；
/// 同样拒在取号之前、两块盘逐字节不变。逐盘核点名落后盘那一判由
/// `stale_device_one_refuses_the_writable_mount_naming_every_unit_it_misses_before_any_write` 钉着。
#[test]
fn blank_device_refuses_the_writable_mount_by_name_before_any_write() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let [_, second, third] = &pool.versions;
    assert_eq!(
        (second.record.counter, third.record.counter),
        (4, 5),
        "txg 4、txg 5 各一条记录"
    );
    let mut devices_with_device_zero_blank = vec![
        (DEVICE_ZERO, blank_device()),
        (DEVICE_ONE, sparse_from(&pool.full, DEVICE_ONE)),
    ];
    let image_before_the_mount_with_device_zero_blank =
        memory_pool_of(&devices_with_device_zero_blank);
    let refusal_with_device_zero_blank =
        mount_writable(&pool.parameters, &mut devices_with_device_zero_blank)
            .expect_err("盘 0 是空盘，可写挂载要拒");
    assert_refused_by_the_witness_of_a_newer_publish(
        &refusal_with_device_zero_blank,
        "盘 0 是空盘",
        version_key(1, 4),
        5,
        4,
    );
    assert_eq!(
        memory_pool_of(&devices_with_device_zero_blank),
        image_before_the_mount_with_device_zero_blank,
        "盘 0 是空盘：拒在取号之前，两块盘逐字节不变"
    );
    // 盘 1 是空盘：没有槽报读错，R2 判得出；见证值取盘 0 读得出的最大 tail 5，所选那一版 (1, 5) 末条 5，判据 N-配置 为假。
    // 往下逐盘核：盘 1 一份自证过的系统配置都没有（C554 乙-配置续 Q1，不「可见」），以「读阶段判完之后系统配置又读不出」拒，
    // 带读阶段往下走用的那一遍读数（C664（挂载拒绝合法复算把两种拒因报成一个）之后不再并进见证读的拒因）。
    let mut devices_with_device_one_blank = vec![
        (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
        (DEVICE_ONE, blank_device()),
    ];
    let image_before_the_mount_with_device_one_blank =
        memory_pool_of(&devices_with_device_one_blank);
    let refusal_with_device_one_blank =
        mount_writable(&pool.parameters, &mut devices_with_device_one_blank)
            .expect_err("盘 1 是空盘，可写挂载要拒");
    let first_read = SelectedVersionAgainstTheWitness {
        selected_version: version_key(1, 5),
        witness: NewerPublishWitness {
            witnessed_journal_counter: 5,
            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                selected_version_last_record_counter: 5,
            },
        },
    };
    assert!(
        matches!(
            &refusal_with_device_one_blank,
            MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)
                if **unreadable
                    == SystemConfigurationUnreadableAfterTheReadStage {
                        read_stage: first_read,
                        unreadable: SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                            device: DEVICE_ONE,
                        },
                    }
        ),
        "盘 1 是空盘：读阶段判得出、为假，逐盘核点名盘 1 一份自证过的系统配置都没有：{refusal_with_device_one_blank:?}"
    );
    assert_eq!(
        memory_pool_of(&devices_with_device_one_blank),
        image_before_the_mount_with_device_one_blank,
        "盘 1 是空盘：拒在取号之前，两块盘逐字节不变"
    );
}

/// 盘 1 停在 txg 3 之后、盘 0 到了 txg 5：可写挂载在取号之前拒掉，点名盘 1——它最新的系统配置停在 txg 3 那次发布的末条记录
/// (1, 3)，落后于所选那一版 (1, 5)；缺的单元正是 txg 4、txg 5 写出的那几个（从进程里的两次发布算，不读盘）。两块盘逐字节不变。
#[test]
fn stale_device_one_refuses_the_writable_mount_naming_every_unit_it_misses_before_any_write() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let expected_missing = units_written_after_txg_three(&pool);
    let mut devices = vec![
        (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
        (DEVICE_ONE, sparse_from(&pool.stale, DEVICE_ONE)),
    ];
    let before = memory_pool_of(&devices);
    let error =
        mount_writable(&pool.parameters, &mut devices).expect_err("盘 1 停在旧状态，可写挂载要拒");
    let refusal = refusal_by_devices_without_the_selected_version(&error, "盘 1 停在旧状态");
    assert_eq!(refusal.selected_version, version_key(1, 5));
    assert_eq!(
        refusal.selected_version_journal_position,
        journal_position(1, 5)
    );
    let [DeviceWithoutTheSelectedVersion {
        device,
        lacking:
            SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                newest_system_configuration_journal_tail,
                units_missing,
            },
    }] = refusal.devices.as_slice()
    else {
        panic!("只点名盘 1、缺的是 txg 4 之后的单元：{:?}", refusal.devices)
    };
    assert_eq!(*device, DEVICE_ONE);
    assert_eq!(
        *newest_system_configuration_journal_tail,
        journal_position(1, 3),
        "盘 1 最新的系统配置是 txg 3 那次发布轮换写的"
    );
    let mut missing_sorted = units_missing.clone();
    missing_sorted.sort_unstable();
    assert_eq!(
        missing_sorted, expected_missing,
        "缺的就是 txg 4、txg 5 写出、txg 5 那一版还引用着的那几个单元"
    );
    assert_eq!(
        memory_pool_of(&devices),
        before,
        "拒在取号之前，两块盘逐字节不变"
    );
}

/// 只读挂载照旧：空盘在盘 1、盘 1 停在旧状态、空盘在盘 0 三种池上，只读挂载做成、读回所选那一版的文件。
/// 盘 0 是空盘那一格，每个单元的第一条位置条目在盘 0 上验不过，读到的是盘 1 上那一份——带着这一版的那块盘。
#[test]
fn read_only_mount_reads_the_selected_version_from_the_device_that_carries_it() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let mut device_one_blank = pool.full.clone();
    device_one_blank
        .devices
        .insert(DEVICE_ONE, SparseDevice::default());
    let mut device_one_stale = pool.full.clone();
    device_one_stale
        .devices
        .insert(DEVICE_ONE, pool.stale.devices[&DEVICE_ONE].clone());
    let mut device_zero_blank = pool.full.clone();
    device_zero_blank
        .devices
        .insert(DEVICE_ZERO, SparseDevice::default());
    let cases = [
        (
            "盘 1 是空盘",
            device_one_blank,
            version_key(1, 5),
            content_of(3),
        ),
        (
            "盘 1 停在旧状态",
            device_one_stale,
            version_key(1, 5),
            content_of(3),
        ),
        (
            "盘 0 是空盘",
            device_zero_blank,
            version_key(1, 4),
            content_of(2),
        ),
    ];
    for (what, image, expected_version, expected_content) in cases {
        let mounted = mount_read_only(&image)
            .unwrap_or_else(|failure| panic!("{what}：只读挂载要做成，实际 {failure:?}"));
        assert_eq!(
            version_key(
                mounted.effective_root.instance.0,
                mounted.effective_root.checkpoint_txg.0
            ),
            expected_version,
            "{what}：只读挂载沿所选那一版打开"
        );
        let file = mounted
            .mounted
            .open_file(&image, InodeNumber(FIRST_INODE_NUMBER))
            .unwrap_or_else(|failure| panic!("{what}：文件要打得开，实际 {failure:?}"));
        let read = file
            .read_at(
                &image,
                FileOffsetInBytes(0),
                u64::try_from(expected_content.len()).expect("4100"),
            )
            .unwrap_or_else(|failure| panic!("{what}：文件要读得出，实际 {failure:?}"));
        assert_eq!(read.bytes, expected_content, "{what}：读回所选那一版的内容");
    }
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_fsync_drop_and_devices_without_the_selected_version::device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies() {
    scenarios_of_fsync_drop_and_devices_without_the_selected_version::device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 盘 1 的系统配置跟得上所选那一版（(1, 5)），只是 txg 5 那一版的数据单元在盘 1 上那一份坏了一个字节：不拒，可写挂载做成。
/// 钉住「单份坏不算不带」：不是空盘、也没有停在旧状态，条款不让它作废（这一格交主 agent 定，见报告）。
#[test]
fn current_device_one_with_one_corrupted_copy_still_mounts_writable() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let [_, _, third] = &pool.versions;
    let data_location_on_device_one = third.data_pointers[0]
        .locations
        .iter()
        .find(|location| location.device == DEVICE_ONE)
        .expect("数据单元两条位置条目，一条在盘 1");
    let offset = data_location_on_device_one.slot.to_device_offset();
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let mut image = pool.full.clone();
    let mut corrupted = image.devices[&DEVICE_ONE].read(offset, data_unit_bytes);
    corrupted[100] ^= 0xFF;
    image
        .devices
        .get_mut(&DEVICE_ONE)
        .expect("池里有盘 1")
        .write(offset, &corrupted);
    assert_eq!(
        newest_system_configuration_journal_tail_on(&pool.parameters, &image, DEVICE_ONE),
        Some(journal_position(1, 5)),
        "盘 1 的系统配置跟得上所选那一版"
    );
    let mut devices = vec![
        (DEVICE_ZERO, sparse_from(&image, DEVICE_ZERO)),
        (DEVICE_ONE, sparse_from(&image, DEVICE_ONE)),
    ];
    let mounted = mount_writable(&pool.parameters, &mut devices).unwrap_or_else(|error| {
        panic!("跟得上这一版、只坏了一份的盘，可写挂载照样做成，实际 {error:?}")
    });
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        CheckpointTxg(5),
        "接在 txg 5 那一版后面"
    );
}

/// 不接受零长度读的盘（有的块层把长度 0 当成不是物理块整数倍的请求拒掉）：别的读写原样转给内存盘。
struct ZeroLengthReadRefusingDevice {
    inner: SparseBlockDevice,
}

impl BlockDevice for ZeroLengthReadRefusingDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        if buffer.is_empty() {
            return Err(BlockDeviceError::Unaligned {
                offset,
                length: 0,
                physical_block_size: self.inner.probe_physical_block_size().0,
            });
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 盘 1 只是系统配置落后一格，而 txg 5 那一版的数据单元两块盘上都读不出（D19（块指针的结构与宽度预算） 已定项 5 的 N2：
/// 重建照抄它的位置项、不读内容，挂载照常）：那一项没有验得过的一份可比，逐盘核跳过它，可写挂载做成。
/// 两块盘都不接受零长度读：逐盘核要是拿那一项空的字节去读盘比，读就报错、盘 1 被记成缺它。
#[test]
fn device_one_behind_by_the_rotation_with_the_data_unit_unreadable_everywhere_still_mounts_writable(
) {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let [_, _, third] = &pool.versions;
    let mut image = full_image_without_the_last_system_configuration_write_on_device_one(&pool);
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    for location in &third.data_pointers[0].locations {
        image
            .devices
            .get_mut(&location.device)
            .expect("池里有这块盘")
            .write(
                location.slot.to_device_offset(),
                &vec![0u8; data_unit_bytes],
            );
    }
    assert_eq!(
        newest_system_configuration_journal_tail_on(&pool.parameters, &image, DEVICE_ONE),
        Some(journal_position(1, 4)),
        "盘 1 的系统配置落后：逐盘核会去读它的单元"
    );
    let mut devices: Vec<(DeviceIdentity, ZeroLengthReadRefusingDevice)> =
        [DEVICE_ZERO, DEVICE_ONE]
            .into_iter()
            .map(|device| {
                (
                    device,
                    ZeroLengthReadRefusingDevice {
                        inner: sparse_from(&image, device),
                    },
                )
            })
            .collect();
    let mounted = mount_writable(&pool.parameters, &mut devices).unwrap_or_else(|error| {
        panic!("数据单元哪块盘上都读不出，不是哪一块盘缺它：可写挂载照常做成，实际 {error:?}")
    });
    let current = mounted.current.file_version().expect("带文件");
    assert!(
        current
            .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .bytes
            .is_empty(),
        "那个数据单元照抄位置项、不读内容"
    );
}

/// 树表 0 条的一版：mkfs → 第一次可写挂载（取号 1、零单元写行、暖机）→ 盘 1 停在这里 → 第二次可写挂载（取号 2、给实例 1 写行：
/// 重写实例表、写出分配记录树）。盘 1 停在旧状态时第三次可写挂载拒掉，点名盘 1，缺的正是第二次挂载写行那次重写的那几个单元。
#[test]
fn device_one_stale_on_the_version_without_file_refuses_the_writable_mount_naming_what_the_row_publish_rewrote(
) {
    let (mut rig, _genesis) = rig_after_make_filesystem();
    mount_writable(&rig.parameters, &mut rig.devices).expect("第一次可写挂载");
    let stale_point = rig.stream.operation_count();
    let second_mount = mount_writable(&rig.parameters, &mut rig.devices).expect("第二次可写挂载");
    assert!(
        second_mount.current.file_version().is_none(),
        "两次挂载之后仍是树表 0 条的一版"
    );
    let PoolVersion::WithoutFile(row_publish) = &second_mount.output.row_publish else {
        panic!("树表 0 条的一版上写行，走只重写实例表的那条发布路径")
    };
    let mut rewritten_by_the_row_publish = row_publish.rewritten.clone();
    rewritten_by_the_row_publish.sort_unstable();
    assert!(
        rewritten_by_the_row_publish.contains(&TransactionUnit::InstanceTable)
            && rewritten_by_the_row_publish.contains(&TransactionUnit::AllocationTree),
        "给实例 1 写行：重写实例表、写出这一版的分配记录树（C512（树表 0 条的一版上被换下的单元记在哪））：{rewritten_by_the_row_publish:?}"
    );
    let full = image_of(&rig, &BTreeSet::new());
    let mut stale = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    stale.apply(&rig.stream.retained_operations()[..stale_point]);
    let mut devices = vec![
        (DEVICE_ZERO, sparse_from(&full, DEVICE_ZERO)),
        (DEVICE_ONE, sparse_from(&stale, DEVICE_ONE)),
    ];
    let before = memory_pool_of(&devices);
    let error = mount_writable(&rig.parameters, &mut devices)
        .expect_err("盘 1 停在第一次挂载之后，可写挂载要拒");
    let refusal =
        refusal_by_devices_without_the_selected_version(&error, "树表 0 条的一版上盘 1 停在旧状态");
    assert_eq!(
        refusal.selected_version,
        RollbackTarget {
            instance: second_mount.output.instance,
            checkpoint_txg: second_mount.current.root().checkpoint_txg,
        },
        "所选那一版是第二次挂载最后那次暖机的根"
    );
    let [DeviceWithoutTheSelectedVersion {
        device,
        lacking:
            SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                newest_system_configuration_journal_tail,
                units_missing,
            },
    }] = refusal.devices.as_slice()
    else {
        panic!("只点名盘 1：{:?}", refusal.devices)
    };
    assert_eq!(*device, DEVICE_ONE);
    assert_eq!(
        newest_system_configuration_journal_tail.instance_generation,
        InstanceGeneration(1),
        "盘 1 最新的系统配置停在实例 1"
    );
    let mut missing_roles: Vec<TransactionUnit> =
        units_missing.iter().map(|(role, _)| *role).collect();
    missing_roles.sort_unstable();
    assert_eq!(
        missing_roles, rewritten_by_the_row_publish,
        "缺的就是第二次挂载写行那次重写的那几个单元（树表是 mkfs 那一份，盘 1 上有）"
    );
    assert_eq!(
        memory_pool_of(&devices),
        before,
        "拒在取号之前，两块盘逐字节不变"
    );
}

/// 读路径那一格（不是 fsync 失败；调查文件那条 `mount_writable_while_every_read_of_device_one_fails_is_recorded` 只记不判）：
/// 第一个文件（txg 3）→ 覆盖写 txg 4 之后，盘 1 的每一次读都报错、写照常。交进来的是两块盘，w 的下限那一判不拦；
/// 所选那一版是 (1, 3)、不是 (1, 4)：txg 4 的根只落在区域 1（盘 1，读不出），txg 4 那条记录点名的单元在盘 1 上那一份验不了、
/// 那次发布不施加（同「盘 0 是空盘」那一格所选的是 (1, 4) 而不是 (1, 5) 的道理）。盘 0 的系统配置见证过 txg 4 那次发布
/// （jsn 4 > 所选那一版末条 3；盘 1 两槽报读错而盘 0 两槽都自证过，R2 判得出），可写挂载在取号之前被 C554 乙拒——逐盘核（点名盘 1「没有自证过的系统配置」）在乙之后不可达，
/// 那一判由 `blank_device_refuses_the_writable_mount_by_name_before_any_write` 的「盘 1 是空盘」那一格钉着。一个写、一道屏障都没落下。
#[test]
fn mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write()
{
    let (mut rig, mut allocator, first, instance) = build_rig();
    let second =
        overwrite(&mut rig, &mut allocator, &first, &content_of(2), instance).expect("txg 4");
    assert_eq!(
        (instance, second.root.checkpoint_txg, second.record.counter),
        (InstanceGeneration(1), CheckpointTxg(4), 4),
        "实例 1 的 txg 4 那一版，末条记录计数器 4"
    );
    drop(allocator);
    rig.plan.arm(FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::OnlyDevice(DEVICE_ONE),
        placement: FaultPlacement::AnyOffset,
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    });
    let operations_before_the_mount = rig.stream.operation_count();
    let error = mount_writable(&rig.parameters, &mut rig.devices)
        .expect_err("盘 1 每次读都报错，可写挂载要拒");
    rig.plan.disarm();
    assert_refused_by_the_witness_of_a_newer_publish(
        &error,
        "盘 1 每次读都报错",
        version_key(1, 3),
        4,
        3,
    );
    assert_eq!(
        rig.stream.operation_count(),
        operations_before_the_mount,
        "拒在取号之前：录制流里一个写、一道屏障都没多"
    );
}
