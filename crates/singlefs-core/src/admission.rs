//! 分配准入的读数与逐设备合取（里程碑「第二个事务」增补 2 收口表第 5 行）：D28（挂载期承诺量） 已定项 1 的式子逐设备算可用，
//! D3（空间分配） 已定项 7 合取表第 1 条「可用 ≥ 需求」按逐设备合取判——每一块盘都够才放行，一块盘不够就拒，
//! 不拿别的盘的富余补（2026-09-23 用户定 C355（checkpoint 保留池池级扣而固定点每块要落两列） 取丁）。
//! 容量之外的前四项取这块盘自己的值；待删占用、已承诺预留、checkpoint 保留池三项是全池的承诺量，按全部副本之和记物理字节，
//! 每块盘各扣「副本之和 ÷ 副本数」（C375（待删占用与已承诺预留按份还是按和记没有条款） 第三轮写回）。
//! 式子不扣「defer 待释放」：「已分配」按读法甲已经含着 defer 里的槽，另扣一次是重复扣（用户 2026-09-25 定，D28（挂载期承诺量） 已定项 1）。
//!
//! 全是只读的纯函数：不动分配器、不发一个写。
//!
//! 接在两处（C363 (b) 判决 `research/prompts/c363b-r1-main-verification.md` 第四节第 2 条，里程碑「第二个事务」增补 2 收口表第 5 行）：
//! - 发布路径：`transaction::prepare_the_version_publish` 在算定这次发布的样子之后、读盘核与动分配器之前，按
//!   [`admission_reading_before_a_publish`] 取读数（实例切换的预留按下一次可写挂载的 rows0，D28（挂载期承诺量） 已定项 3
//!   「发布路径用 rows0 + 1」）、按 [`demand_of_the_roles_on_each_device`] 取这次的需求，逐设备合取；判不判按
//!   [`publish_has_an_ordinary_allocation`]；
//! - 可写挂载：`mount::establish_instance` 在取号之前按 [`admission_reading_of_a_writable_mount`]（这次挂载的 rows0）判
//!   「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13），需求逐盘 0。
//!
//! 条款给的量：checkpoint 保留池的 ckpt_cost 按一次空发布至多写出的固定点计，一律按最坏情况（D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定：
//! 分配记录树按盘分路、每块盘两条叶路径加根，中央映射树逐层按可能改的路径数与可能切出来的节点数，记账树按每发布的节点数，树表一项，实例表链不进；
//! [`checkpoint_cost_of_the_version_to_build_on`]）；挂载期承诺量暖机那一半的 c_max
//! 与保留池「按同一个现算的 c_max 取」（D28（挂载期承诺量） 已定项 4 末条）；需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，
//! 不加这次换下的槽），没有普通分配的发布（空发布、写行、暖机、抬 F）不判（D28（挂载期承诺量） 已定项 1 接线，用户 2026-09-25 定）。
//!
//! 实现员取的读法（条款没写，交主 agent）：需求按盘字节记（C370（需求、可用与 df 没有共同单位） 2026-09-17 收窄：与「已分配」同口径只能读成盘字节），
//! 第一版每个单元落每块盘，每块盘的需求相同；读数取分配器此刻的计数（挂载时是回收与影子账隔离之后、写行之前那一刻；admission 读哪一个
//! 在那两段里条款没写，见 [`AdmissionReading::of_allocator`]）。
//!
//! 准入不够、或准入放行而落点取不到时先推空发布抬 F 再判（D16（发布语义） 已定项 1「准入」那一行，C283（准入失败时不先推发布就报 ENOSPC））
//! 不在这里做：发布那一处归挂着的会话（`mounted_session`），可写挂载那一处归 `mount::establish_instance`（写行之后推）。

use std::collections::BTreeSet;
use std::num::NonZeroU64;

use singlefs_format::{INSTANCE_TABLE_PAGE_RECORDS, SLOT_BYTES};

use crate::address::DeviceIdentity;
use crate::allocation_record_tree::AllocationRecordTreeGeometry;
use crate::allocator::PoolAllocator;
use crate::code_two_tree::{CodeTwoTreeNodeCapacity, CodeTwoTreeShape};
use crate::transaction::{
    CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree, TransactionOutput, TransactionUnit,
};

/// 一块盘上的物理字节：D5（快照 / 空间记账机制） 已定项 7「已分配」的口径——分配记录每个落点每盘一条，逐盘记的是这块盘上真占的字节。
/// 一份副本的大小也用它：一份副本整个落在一块盘上。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytesOnOneDevice(pub u64);

impl BytesOnOneDevice {
    pub const ZERO: BytesOnOneDevice = BytesOnOneDevice(0);

    /// `slots` 个 16 KiB 槽的字节数（落点粒度 16384，D3（空间分配） 已定项 7）。
    ///
    /// # Panics
    /// 乘回字节装不进 u64：槽数来自一块盘的单元区或调用方给的块数，装不下说明调用方给的数不是一块盘上的量。
    #[must_use]
    pub fn of_slots(slots: u64) -> Self {
        Self(
            slots
                .checked_mul(SLOT_BYTES)
                .expect("一块盘上的槽数乘 16384 装得进 u64：再大就不是一块盘上的量"),
        )
    }
}

/// 全池的承诺量：按全部副本之和记的物理字节（C375（待删占用与已承诺预留按份还是按和记没有条款） 第三轮写回：
/// 与「已分配」同一口径；按一份副本记会让 D28（挂载期承诺量） 已定项 1 的式子与 I-3.4（可用空间扣待删占用） 当场为假）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytesSummedOverAllReplicas(pub u64);

impl BytesSummedOverAllReplicas {
    pub const ZERO: BytesSummedOverAllReplicas = BytesSummedOverAllReplicas(0);

    /// 一样东西每份副本各占 `one_copy`、落满 `replicas` 份时，全部副本加起来的字节。
    ///
    /// # Panics
    /// 乘积装不进 u64。
    #[must_use]
    pub fn of_one_copy_on_every_replica(
        one_copy: BytesOnOneDevice,
        replicas: ReplicaCount,
    ) -> Self {
        Self(
            one_copy
                .0
                .checked_mul(replicas.get())
                .expect("一份副本的字节乘副本数装得进 u64"),
        )
    }

    /// 摊到每块盘扣的那一份：副本之和 ÷ 副本数（D28（挂载期承诺量） 已定项 1）。除不尽时向上取整——扣多不扣少，
    /// 可用只会少报、不会多报（式子是按字节的实数除法，条款没写取整；用 `of_one_copy_on_every_replica` 记下的量总除得尽）。
    #[must_use]
    pub fn share_of_one_device(self, replicas: ReplicaCount) -> BytesOnOneDevice {
        BytesOnOneDevice(self.0.div_ceil(replicas.get()))
    }
}

/// 式子里的「副本数」：一个单元落几份，至少一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplicaCount(NonZeroU64);

impl ReplicaCount {
    /// 0 份交回 `None`。
    #[must_use]
    pub fn new(count: u64) -> Option<Self> {
        NonZeroU64::new(count).map(Self)
    }

    /// 第一版的池：每个单元落池里每一块盘——分配器每次分配逐盘各记一条、各盘同槽（`PoolAllocator` 的记录与位图），
    /// 发布路径逐盘各写一份（`CommitStep::WriteUnitToEveryDevice`）⇒ 副本数 = 池里的盘数。
    /// ⚠️ 三块盘以上时两份落哪两块盘没有条款（D28（挂载期承诺量） 已定项 3 射程、C126（切换预留的最坏量没有口径）），
    /// 那时这一句跟着分配器一起改。
    ///
    /// # Panics
    /// 分配器里一块盘都没有：池至少有一块盘（mkfs 的几何检查）。
    #[must_use]
    pub fn of_every_device_in_the_pool(allocator: &PoolAllocator) -> Self {
        Self::new(u64::try_from(allocator.devices.len()).expect("盘数装得进 u64"))
            .expect("分配器里至少一块盘：mkfs 的几何检查不收没有盘的池")
    }

    #[must_use]
    pub fn get(self) -> u64 {
        self.0.get()
    }
}

/// 16 KiB 元数据块的块数：ckpt_cost 与 c_max 的单位（D28（挂载期承诺量） 已定项 4「单位是 16 KiB 元数据块」）。一块占一个槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetadataBlocks(pub u64);

impl MetadataBlocks {
    /// 一份副本占的字节：块数 × 16384。
    #[must_use]
    pub fn bytes_of_one_copy(self) -> BytesOnOneDevice {
        BytesOnOneDevice::of_slots(self.0)
    }
}

/// 式子里的「不可回收」：只在 zoned 上非零（活快照钉住的是 zone 粒度、引用按块计，`invariants.md` 逐字要求另立的量）；
/// 第一版不跑 zoned ⇒ 恒 0。记账树每块盘那一行「不可回收字节」写的就是它（`transaction` 装记账行那一段）。
pub const UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED: BytesOnOneDevice = BytesOnOneDevice::ZERO;

/// 式子里的「待删占用」（记账第 4 项）：第一版不删对象、不销毁快照，没有「待删除但意图未完成」的结构 ⇒ 恒 0。
/// 记账树那一行池级「待删占用」写的就是它。
pub const PENDING_DELETE_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas =
    BytesSummedOverAllReplicas::ZERO;

/// 式子里的「已承诺预留」（记账第 6 项，含墓碑）：第一版没有删除路径，墓碑的预付（D3（空间分配） 已定项 2，量是 C84（墓碑单元的粒度没人定）
/// 那一格）与别的预留都不发生 ⇒ 恒 0。记账树那一行池级「已承诺预留」写的就是它。
pub const COMMITTED_RESERVATION_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas =
    BytesSummedOverAllReplicas::ZERO;

/// 一次挂载允许几次实例切换：N_switch = 3（D23（journal 的角色与格式） 已定项 14，D28（挂载期承诺量） 已定项 3 逐字）。
pub const INSTANCE_SWITCHES_ALLOWED_PER_MOUNT: u64 = 3;

/// 一次切换之后至多推几次暖机空发布：R = 3（D28（挂载期承诺量） 已定项 3「至多 R 次空发布」）。
pub const WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH: u64 = 3;

/// 实例表一片里装行的记录数：一片 370 条，最后一条恒为链指针记录（D18（块里携带什么信息） 已定项 11）⇒ 369
/// （D28（挂载期承诺量） 已定项 3「行宽 88 ⇒ 369 行」）。
pub const INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE: u64 = INSTANCE_TABLE_PAGE_RECORDS - 1;

/// checkpoint 保留池（式子里的「checkpoint 保留池」，D28（挂载期承诺量） 已定项 4）：ckpt_cost 个元数据块，
/// 固定点写出的每一块落每一份副本（D2（RAID 条带策略） 已定项 6）⇒ 按全部副本之和记 ckpt_cost × 16384 × 副本数。
/// `checkpoint_cost` 由调用方给：它从哪读没有条款（C363，见模块文档）。
#[must_use]
pub fn checkpoint_reserve_pool(
    checkpoint_cost: MetadataBlocks,
    replicas: ReplicaCount,
) -> BytesSummedOverAllReplicas {
    BytesSummedOverAllReplicas::of_one_copy_on_every_replica(
        checkpoint_cost.bytes_of_one_copy(),
        replicas,
    )
}

/// 实例切换的预留里这块盘的那一份（式子里「挂载期承诺量」按设备摊的成员，D28（挂载期承诺量） 已定项 3）：
/// (N_switch + 1) × (实例表链重写 + 暖机空发布)
/// = (N_switch + 1) × (一片实例表 32768 × max(1, ⌈(rows0 + N_switch) ÷ 369⌉) + R × c_max × 16384)。
/// 实例表单元与空发布的 c_max 块都是每块盘各落一份，所以每块盘各算一份、数相同。
///
/// `instance_rows_after_this_mounts_row_publish` 是 rows0：挂载时读到的行数加写行那次要写的行数。
/// `worst_empty_publish_cost` 是 c_max，由调用方给（见模块文档）。
///
/// # Panics
/// 乘积装不进 u64：行数与 c_max 都来自一次挂载，大到这一步说明调用方给错了量。
#[must_use]
pub fn instance_switch_reserve_on_one_device(
    instance_rows_after_this_mounts_row_publish: u64,
    worst_empty_publish_cost: MetadataBlocks,
) -> BytesOnOneDevice {
    let instance_table_page =
        BytesOnOneDevice::of_slots(TransactionUnit::InstanceTable.span_slots());
    let pages_of_the_chain = instance_rows_after_this_mounts_row_publish
        .checked_add(INSTANCE_SWITCHES_ALLOWED_PER_MOUNT)
        .expect("行数加 N_switch 装得进 u64")
        .div_ceil(INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE)
        .max(1);
    let chain_rewrite = instance_table_page
        .0
        .checked_mul(pages_of_the_chain)
        .expect("链重写的字节装得进 u64");
    let warm_up = worst_empty_publish_cost
        .bytes_of_one_copy()
        .0
        .checked_mul(WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH)
        .expect("暖机那一半的字节装得进 u64");
    let one_switch = chain_rewrite
        .checked_add(warm_up)
        .expect("一次切换的最坏量装得进 u64");
    BytesOnOneDevice(
        one_switch
            .checked_mul(INSTANCE_SWITCHES_ALLOWED_PER_MOUNT + 1)
            .expect("切换预留装得进 u64"),
    )
}

/// 式子里这块盘自己的五项（D28（挂载期承诺量） 已定项 1「前五项取这块盘自己的值」），全按这块盘上的物理字节。
/// 每个字段的文档先写它是式子里的哪一项（名字照式子逐字），再写从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceAdmissionTerms {
    pub device: DeviceIdentity,
    /// 式子里的「容量」：这块盘单元区的大小（D28（挂载期承诺量） 已定项 4「『容量』是单元区大小」）。
    pub capacity: BytesOnOneDevice,
    /// 式子里的「已分配」（记账第 1 项「已分配字节」）：分配器的占着槽数——仍分配的加上已释放、还在 defer 窗口里的
    /// （I-3.1（已分配统计对得上） 读法甲，2026-09-14 用户定）。defer 里的槽只在这一项里扣一次：式子不再另扣「defer 待释放」
    /// （用户 2026-09-25 定，安全设计轮第二轮判决 `research/prompts/m2-safety-r2-main-verification.md`：那一项与「已分配」重复扣）。
    pub allocated: BytesOnOneDevice,
    /// 式子里的「不可回收」（记账第 3 项）。
    pub unreclaimable: BytesOnOneDevice,
    /// 式子里的「挂载期承诺量」按设备摊的成员：实例切换的预留（D28（挂载期承诺量） 已定项 3，
    /// `instance_switch_reserve_on_one_device`）。它的池级成员 checkpoint 保留池在式子里单列，这里不重复扣。
    pub mount_time_commitment: BytesOnOneDevice,
    /// 式子里的「被抛弃根独占量」（D28（挂载期承诺量） 已定项 1 第九项，C318（影子账隔离的单元没进准入不等式））：
    /// 只被被抛弃根引用的槽，与分配器实际隔离的是同一个集合。
    pub abandoned_root_exclusive: BytesOnOneDevice,
}

/// 式子后三项：全池的承诺量，按全部副本之和记，每块盘各扣「副本之和 ÷ 副本数」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolWideCommitments {
    /// 式子里的「待删占用」（记账第 4 项）。
    pub pending_delete: BytesSummedOverAllReplicas,
    /// 式子里的「已承诺预留」（记账第 6 项，含墓碑）。
    pub committed_reservation: BytesSummedOverAllReplicas,
    /// 式子里的「checkpoint 保留池」（`checkpoint_reserve_pool`）。
    pub checkpoint_reserve_pool: BytesSummedOverAllReplicas,
}

impl PoolWideCommitments {
    /// 第一版的待删占用与已承诺预留（两个 0，记账树那两行写的就是它们）；checkpoint 保留池由调用方给（C363）。
    #[must_use]
    pub fn of_the_first_version(checkpoint_reserve_pool: BytesSummedOverAllReplicas) -> Self {
        Self {
            pending_delete: PENDING_DELETE_OF_THE_FIRST_VERSION,
            committed_reservation: COMMITTED_RESERVATION_OF_THE_FIRST_VERSION,
            checkpoint_reserve_pool,
        }
    }
}

/// 一块盘的可用(d)：容量减去式子里另外八项之后剩下的字节。扣的比容量多时是负的（承诺量压过了容量，这块盘上一个字节都不许再分）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AvailableBytesOnOneDevice(pub i128);

/// 准入读数：式子的八项，逐盘五项加全池三项，外加摊全池三项要用的副本数。
/// 不变量（`new` 判）：至少一块盘、盘身份两两不同——合取对空集恒真，一个没有盘的读数会放行一切。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionReading {
    per_device: Vec<DeviceAdmissionTerms>,
    pool_wide: PoolWideCommitments,
    replicas: ReplicaCount,
}

impl AdmissionReading {
    /// # Panics
    /// 一块盘都没有，或两条的盘身份相同。
    #[must_use]
    pub fn new(
        per_device: Vec<DeviceAdmissionTerms>,
        pool_wide: PoolWideCommitments,
        replicas: ReplicaCount,
    ) -> Self {
        assert!(
            !per_device.is_empty(),
            "准入读数至少一块盘：合取对空集恒真，没有盘的读数会放行一切"
        );
        let distinct_devices: BTreeSet<DeviceIdentity> =
            per_device.iter().map(|terms| terms.device).collect();
        assert_eq!(
            distinct_devices.len(),
            per_device.len(),
            "准入读数里每块盘只一条"
        );
        Self {
            per_device,
            pool_wide,
            replicas,
        }
    }

    /// 从分配器此刻的计数取逐盘各项：容量 = 单元区槽数、已分配 = 占着的槽数（含已释放还在窗口里的）、
    /// 被抛弃根独占量 = 影子账隔离的槽数，不可回收取 [`UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED`]；副本数按
    /// [`ReplicaCount::of_every_device_in_the_pool`]。挂载期承诺量（每块盘同一个数）与全池三项由调用方给。
    ///
    /// D28（挂载期承诺量） 已定项 2 要的是「已发布统计量 − 在飞已批准」。这几个计数就是每次发布写进记账行的那几个数
    /// （装记账行读的是同一组），发布与发布之间与最近一次发布的记账行逐项相等；在飞已批准第一版恒为零——一次发布在同一个调用里
    /// 准入、分配、落盘，调用与调用之间没有在飞的东西（在飞 overlay 没有实现，C87（准入读的合成值无定义））。
    /// ⚠️ 两处例外计数先于记账行变：挂载重建时按回收门槛回收、按影子账隔离之后到写行那次发布之前；抬 F 回收的槽扣住到 F 生效之前
    /// （计数上已回到空闲、分配器却不发它们，式子里没有一项装它们）。那两段里准入读哪一个，条款没写。
    #[must_use]
    pub fn of_allocator(
        allocator: &PoolAllocator,
        mount_time_commitment_on_each_device: BytesOnOneDevice,
        pool_wide: PoolWideCommitments,
    ) -> Self {
        let per_device = allocator
            .devices
            .iter()
            .map(|device_map| DeviceAdmissionTerms {
                device: device_map.device,
                capacity: BytesOnOneDevice::of_slots(device_map.unit_area_slots()),
                allocated: BytesOnOneDevice::of_slots(device_map.allocated_slots()),
                unreclaimable: UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED,
                mount_time_commitment: mount_time_commitment_on_each_device,
                abandoned_root_exclusive: BytesOnOneDevice::of_slots(device_map.isolated_slots()),
            })
            .collect();
        Self::new(
            per_device,
            pool_wide,
            ReplicaCount::of_every_device_in_the_pool(allocator),
        )
    }

    #[must_use]
    pub fn per_device(&self) -> &[DeviceAdmissionTerms] {
        &self.per_device
    }

    #[must_use]
    pub fn pool_wide(&self) -> PoolWideCommitments {
        self.pool_wide
    }

    #[must_use]
    pub fn replicas(&self) -> ReplicaCount {
        self.replicas
    }

    /// 可用(d)，逐盘，按读数里盘的次序：
    /// 容量 − 已分配 − 不可回收 − 挂载期承诺量 − 被抛弃根独占量
    /// − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数（D28（挂载期承诺量） 已定项 1）。
    #[must_use]
    pub fn available_on_each_device(&self) -> Vec<(DeviceIdentity, AvailableBytesOnOneDevice)> {
        let pool_wide_share_of_one_device = [
            self.pool_wide.pending_delete,
            self.pool_wide.committed_reservation,
            self.pool_wide.checkpoint_reserve_pool,
        ]
        .map(|commitment| i128::from(commitment.share_of_one_device(self.replicas).0));
        self.per_device
            .iter()
            .map(|terms| {
                let own_terms = [
                    terms.allocated,
                    terms.unreclaimable,
                    terms.mount_time_commitment,
                    terms.abandoned_root_exclusive,
                ]
                .map(|term| i128::from(term.0));
                let deducted: i128 = own_terms.iter().sum::<i128>()
                    + pool_wide_share_of_one_device.iter().sum::<i128>();
                (
                    terms.device,
                    AvailableBytesOnOneDevice(i128::from(terms.capacity.0) - deducted),
                )
            })
            .collect()
    }
}

/// 一块盘上这次操作要新占的物理字节（式子另一边的「需求」）。怎么摊到每块盘由调用方给（C370（需求、可用与 df 没有共同单位））。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DemandOnDevice {
    pub device: DeviceIdentity,
    pub bytes: BytesOnOneDevice,
}

/// 一块不够的盘：它的可用与需求。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceShortOfDemand {
    pub device: DeviceIdentity,
    pub available: AvailableBytesOnOneDevice,
    pub demand: BytesOnOneDevice,
}

/// 逐设备合取判不过：不够的那几块盘，按读数里盘的次序。调用方要做的决定只有一个——这次不许分配；
/// 先推空发布抬 F 再判一次（D16（发布语义） 已定项 1）是调用方的事，这里不做。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionRefusedOnSomeDevices {
    pub short_devices: Vec<DeviceShortOfDemand>,
}

/// 逐设备合取（D28（挂载期承诺量） 已定项 1「逐设备算，每块盘各自满足」；D3（空间分配） 已定项 7 合取表第 1 条）：
/// 每一块盘都要可用(d) ≥ 需求(d) 才放行；一块盘不够就拒，别的盘的富余补不了它。
///
/// # Errors
/// `AdmissionRefusedOnSomeDevices`：至少一块盘可用 < 需求，交回不够的每一块。
///
/// # Panics
/// 需求点名的盘与读数里的盘不是同一组（少一块、多一块、或同一块点了两次）：每块盘的需求都要写出来，需求为零也写 0。
pub fn admit_on_every_device(
    reading: &AdmissionReading,
    demand_on_each_device: &[DemandOnDevice],
) -> Result<(), AdmissionRefusedOnSomeDevices> {
    let reading_devices: BTreeSet<DeviceIdentity> = reading
        .per_device
        .iter()
        .map(|terms| terms.device)
        .collect();
    let demand_devices: BTreeSet<DeviceIdentity> = demand_on_each_device
        .iter()
        .map(|demand| demand.device)
        .collect();
    assert!(
        demand_devices.len() == demand_on_each_device.len() && demand_devices == reading_devices,
        "需求要逐盘写全、每块盘一条，与读数里的盘是同一组：读数 {reading_devices:?}，需求 {demand_on_each_device:?}"
    );
    let short_devices: Vec<DeviceShortOfDemand> = reading
        .available_on_each_device()
        .into_iter()
        .filter_map(|(device, available)| {
            let demand = demand_on_each_device
                .iter()
                .find(|demand| demand.device == device)
                .expect("上面判过需求与读数是同一组盘")
                .bytes;
            (available.0 < i128::from(demand.0)).then_some(DeviceShortOfDemand {
                device,
                available,
                demand,
            })
        })
        .collect();
    if short_devices.is_empty() {
        Ok(())
    } else {
        Err(AdmissionRefusedOnSomeDevices { short_devices })
    }
}

/// 只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：发布与可写挂载判不判空间准入。产品路径恒 `JudgedByTheFormula`。
/// 准入接进来之后，「准入放行而落点仍取不到、在任何写之前拒绝」那一条（D3（空间分配） 已定项 5；C545（空间准入罩不住分裂与聚簇段层））
/// 在健康的历史里走不到——小盘上式子先拒。关掉准入，那一条拒绝路径（取号之前的预演取不到落点、发布取不到落点、抬 F 的空发布取不到固定点）
/// 照样测得到。装在分配器上（`PoolAllocator::set_space_admission`），可写挂载按调用方给的装（`mount::mount_writable_with_space_admission`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpaceAdmission {
    JudgedByTheFormula,
    SkippedByTheTestOnlySwitch,
}

impl SpaceAdmission {
    /// 这一次走的是哪一臂，运行时报得出（五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn branch_name(self) -> &'static str {
        match self {
            SpaceAdmission::JudgedByTheFormula => "space_admission=judged",
            SpaceAdmission::SkippedByTheTestOnlySwitch => "space_admission=skipped",
        }
    }
}

/// checkpoint 保留池的 ckpt_cost（D28（挂载期承诺量） 已定项 4）：一次空发布至多写出的固定点单元数，按这次发布要接在后面的那一版的结构现算，
/// 一律按最坏情况计（用户 2026-09-27 定：游标跨叶、中央映射树多层都不许少扣；此前按盘分路的那一版只罩「每块盘一片叶、中央映射树 1 层」）：
/// - 分配记录树：每块盘两条从叶到根之下那一层的路径加共用的根，盘数 × 2 × (高 − 1) + 1
///   （[`allocation_record_tree_nodes_on_two_leaf_paths_per_device`]）；
/// - 中央映射树：逐层按这次可能改的路径数与可能切出来的节点数计（[`central_mapping_tree_nodes_an_empty_publish_rewrites_at_most`]）；
/// - 记账树每次发布整批重写（`transaction` 装记账行那一段，条目数每次发布相同、整棵按同一个形状重建）：这一版记账树的节点数；
/// - 树表 1（每次发布重写一个单元，D16（发布语义） 已定项 9）；实例表链不进（它的开销归已定项 3 的切换预留）。
///
/// 「当前」= 这次发布要接在后面的那一版：
/// - 带文件的一版：分配记录树与中央映射树的高从各自根节点的码 2 头里现读（层级 + 1，不用内存里另存一份，已定项 4）；
///   中央映射树每层的节点数从这一版的形状里取。
/// - 树表 0 条的一版（`None`）：没有中央映射树与记账树（0 与 0）；分配记录树只在那一版写过行时有（分配器记着它，
///   `PoolAllocator::allocation_record_tree_of_the_version_without_file`），它按位置寻址、根的层级由池几何定（D8（核心索引结构） 已定项 14），
///   高取几何的高——这一处分配器里只有节点与指针、没有根节点的字节可读；没写过行的（mkfs 的第 0 代）一棵都没有，取 0。
///
/// 中央映射树的节点容量按节点格式算（产品路径）；只供测试的压小容量下量同一个数用
/// [`checkpoint_cost_of_the_version_to_build_on_with_node_capacities`]。
///
/// ⚠️ 射程：分配记录树的「每块盘两条路径」罩的是 bump 游标在一个开放段里走、跨过一片叶的末槽那一次（这次取的落点在新叶、换下的上一版落点在旧叶）。
/// 一次发布里开放段装不下、开新段或回落到最低空槽，或者换下的是很久以前落在别处的节点时，一块盘上改的叶可以多于两片，这一格条款没写怎么计。
///
/// 同一个数也是挂载期承诺量里暖机那一半的 c_max（已定项 4 末条「按同一个现算的 c_max 取」）。
#[must_use]
pub fn checkpoint_cost_of_the_version_to_build_on(
    version_to_build_on: Option<&TransactionOutput>,
    allocator: &PoolAllocator,
) -> MetadataBlocks {
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
        version_to_build_on,
        allocator,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    )
}

/// 同 [`checkpoint_cost_of_the_version_to_build_on`]，中央映射树的节点容量按 `node_capacities` 取：写入口装着只供测试的压小容量时，
/// 它的空发布按压小的容量切分，量它的上界要按同一个容量算。产品路径（发布与可写挂载的准入）按节点格式调上面那一个。
#[must_use]
pub fn checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
    version_to_build_on: Option<&TransactionOutput>,
    allocator: &PoolAllocator,
    node_capacities: CodeTwoTreeNodeCapacities,
) -> MetadataBlocks {
    const TREE_TABLE_UNITS_PER_PUBLISH: u64 = 1;
    let devices_in_the_pool = u64::try_from(allocator.devices.len()).expect("盘数装得进 u64");
    let record_trees_and_accounting_nodes = match version_to_build_on {
        Some(file_version) => {
            let allocation_record_tree_nodes =
                allocation_record_tree_nodes_on_two_leaf_paths_per_device(
                    file_version
                        .position_addressed_tree_heights_read_from_the_root_node_headers()
                        .allocation_record_tree,
                    devices_in_the_pool,
                );
            let accounting_tree_nodes = u64::try_from(file_version.accounting_tree.node_count())
                .expect("记账树的节点数装得进 u64");
            // 一次空发布在中央映射树里改的条目只有这两棵树的节点（别的进映射的单元照抄上一版、key 不变）：
            // 重写的每个节点删一把旧 key（上一版有它时）、插一把这次的 key。
            let changed_nodes_that_are_mapped = allocation_record_tree_nodes
                .checked_add(accounting_tree_nodes)
                .expect("两棵树重写的节点数装得进 u64");
            let central_mapping_tree_nodes =
                central_mapping_tree_nodes_an_empty_publish_rewrites_at_most(
                    &central_mapping_tree_nodes_at_each_level(
                        &file_version.central_mapping_tree.shape,
                        file_version.height_read_from_the_root_node_header(
                            MultiLevelCodeTwoTree::CentralMapping,
                        ),
                    ),
                    u64::try_from(file_version.mapped_units.len())
                        .expect("进映射的单元数装得进 u64"),
                    CentralMappingEntryChangesOfAnEmptyPublish {
                        deleted_at_most: changed_nodes_that_are_mapped,
                        inserted_at_most: changed_nodes_that_are_mapped,
                    },
                    node_capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
                );
            [
                allocation_record_tree_nodes,
                central_mapping_tree_nodes,
                accounting_tree_nodes,
            ]
            .into_iter()
            .try_fold(0u64, u64::checked_add)
            .expect("三棵树的节点数加起来装得进 u64")
        }
        None => match allocator.allocation_record_tree_of_the_version_without_file() {
            Some(_) => allocation_record_tree_nodes_on_two_leaf_paths_per_device(
                AllocationRecordTreeGeometry::of_allocator(allocator).height(),
                devices_in_the_pool,
            ),
            None => 0,
        },
    };
    MetadataBlocks(record_trees_and_accounting_nodes + TREE_TABLE_UNITS_PER_PUBLISH)
}

/// 一次空发布在一块盘上至多改几条分配记录树的叶路径（用户 2026-09-27 定按最坏情况计：游标跨叶那一次每盘多算一条路径）：
/// bump 游标走过一片叶的末槽时，这次取的落点在新叶、换下的上一版落点在旧叶。
const ALLOCATION_RECORD_TREE_LEAF_PATHS_PER_DEVICE_AT_MOST: u64 = 2;

/// 分配记录树一次空发布至多重写的节点数，按盘分路（D8（核心索引结构） 已定项 14：根里按盘分路、根之下每个节点只属于一块盘）：
/// 第一版每个单元落池里每一块盘，一次发布取的落点与换下的落点在每块盘上各有记录要改；每块盘至多改两条从叶到根之下那一层的路径
/// （每条树高 − 1 个节点，[`ALLOCATION_RECORD_TREE_LEAF_PATHS_PER_DEVICE_AT_MOST`]）；根罩整个 key 空间，一次改一个
/// ⇒ 盘数 × 2 × (树高 − 1) + 1。两块 4 GiB 盘高 3 时 9 个（每块盘一片叶的那几次实写 5 个，
/// `second_transaction_supplement_one_write_accounting` 钉的 `ALLOCATION_RECORD_TREE_NODES_REWRITTEN`；跨叶那一次实写 7 个）。
///
/// # Panics
/// 树高是 0：分配记录树的根层级至少 1（`AllocationRecordTreeGeometry::of_devices`），树高至少 2；或乘积装不进 u64。
fn allocation_record_tree_nodes_on_two_leaf_paths_per_device(
    tree_height: u64,
    devices_in_the_pool: u64,
) -> u64 {
    let one_leaf_path_below_the_root = tree_height
        .checked_sub(1)
        .expect("分配记录树的根层级至少 1：树高至少 2");
    devices_in_the_pool
        .checked_mul(ALLOCATION_RECORD_TREE_LEAF_PATHS_PER_DEVICE_AT_MOST)
        .and_then(|leaf_paths| leaf_paths.checked_mul(one_leaf_path_below_the_root))
        .and_then(|nodes_below_the_root| nodes_below_the_root.checked_add(1))
        .expect("盘数 × 2 × 路径长 + 1 装得进 u64：盘数至多 169、树高至多 256")
}

/// 一次空发布在中央映射树里至多删几把 key、插几把 key。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CentralMappingEntryChangesOfAnEmptyPublish {
    deleted_at_most: u64,
    inserted_at_most: u64,
}

/// 中央映射树这一版每层有几个节点，自叶（层级 0）到根，共 `height` 层（根节点头里现读的高）。形状里的节点按 (层级, 层内序号) 升序排，
/// 每层的边界二分找，代价是树高 × log(节点数)，不逐个数。
///
/// # Panics
/// 形状的高与根节点头里读出来的高不等，或有一层一个节点都没有：两样出自同一次发布（或同一次从盘上重建），对不上说明造它的一方写错了。
fn central_mapping_tree_nodes_at_each_level(shape: &CodeTwoTreeShape, height: u64) -> Vec<u64> {
    assert_eq!(
        shape.height(),
        height,
        "中央映射树形状的高与根节点头里的层级 + 1 相等：出自同一次发布或同一次重建"
    );
    let nodes = shape.nodes();
    (0..height)
        .map(|level| {
            let level = u8::try_from(level).expect("码 2 头的层级是 1 字节");
            let first = nodes.partition_point(|node| node.position.level < level);
            let past_the_last = nodes.partition_point(|node| node.position.level <= level);
            let nodes_at_this_level =
                u64::try_from(past_the_last - first).expect("一层的节点数装得进 u64");
            assert!(
                nodes_at_this_level > 0,
                "中央映射树第 {level} 层至少一个节点：高是 {height}"
            );
            nodes_at_this_level
        })
        .collect()
}

/// 一层上至多切几次（D8（核心索引结构） 已定项 11 ②：装不下时从中间切，左半 ⌈n ÷ 2⌉、右半其余）：
/// `additions` 是这一次往这一层加的条目数（叶层是插的 key，内部层是下一层切出来的孩子），`pre_existing_nodes` 是这一层原有的节点数。
/// 原有的节点第一次切至少要加 1 条（它可能正好装满）；切出来的两半都至少空着「⌊(容量 + 1) ÷ 2⌋ − 1」格，
/// 再切一次至少要加 `additions_per_split_of_a_fresh_node` = ⌊(容量 + 1) ÷ 2⌋ 条。先删后插（`code_two_tree::plan_the_tree_after_this_publish`），
/// 删只让节点更空。于是切的次数 ≤ min(原有节点数, 加的条数) + 余下的条数 ÷ 那个数。
fn splits_at_one_level_at_most(
    pre_existing_nodes: u64,
    additions: u64,
    additions_per_split_of_a_fresh_node: u64,
) -> u64 {
    let first_splits_of_pre_existing_nodes = pre_existing_nodes.min(additions);
    first_splits_of_pre_existing_nodes
        + (additions - first_splits_of_pre_existing_nodes) / additions_per_split_of_a_fresh_node
}

/// 中央映射树一次空发布至多重写的节点数（用户 2026-09-27 定按最坏情况计，逐层按可能改的路径数计）：
/// - 每一层原有的节点里，这次重写的都在某一把删掉或插进的 key 的路径上（叶到根整条 COW，D8（核心索引结构） 已定项 11 ①），
///   一把 key 一层一个 ⇒ 至多 min(这一层的节点数, 删的 + 插的)；
/// - 切出来的节点都是新写的：叶层至多 [`splits_at_one_level_at_most`]（加的是插的 key），内部层同一条（加的是下一层切出来的孩子）；
///   这一层装得下下一层全部节点时（叶层：全树的条目加插的装得进一片叶）一次都切不了；
/// - 根切开就长高一层：新根一个，它起步两个孩子、之后下一层每多切一次多一个孩子，装不下再切、再长高。
///
/// # Panics
/// `nodes_at_each_level` 是空的（带文件的一版中央映射树至少一个节点），或加起来装不进 u64。
fn central_mapping_tree_nodes_an_empty_publish_rewrites_at_most(
    nodes_at_each_level: &[u64],
    entries_before_the_publish: u64,
    changes: CentralMappingEntryChangesOfAnEmptyPublish,
    capacity: CodeTwoTreeNodeCapacity,
) -> u64 {
    assert!(
        !nodes_at_each_level.is_empty(),
        "带文件的一版中央映射树至少一个节点"
    );
    let leaf_entries = u64::try_from(capacity.leaf_entries).expect("叶容量装得进 u64");
    let internal_entries =
        u64::try_from(capacity.internal_entries).expect("内部节点容量装得进 u64");
    // ⌊(容量 + 1) ÷ 2⌋ = ⌈容量 ÷ 2⌉：切开之后较空的那一半再切一次至少要加的条数。
    let leaf_additions_per_split_of_a_fresh_node = leaf_entries.div_ceil(2);
    let internal_additions_per_split_of_a_fresh_node = internal_entries.div_ceil(2);
    assert!(
        leaf_additions_per_split_of_a_fresh_node >= 1 && internal_entries >= 2,
        "叶至少装 1 条、内部节点至少装 2 个孩子（`code_two_tree::plan_the_tree_after_this_publish` 同一条断言）"
    );
    let changed_paths = changes
        .deleted_at_most
        .checked_add(changes.inserted_at_most)
        .expect("删的加插的装得进 u64");
    let mut nodes_rewritten: u64 = 0;
    let mut splits_at_the_level_below: u64 = 0;
    // 迭代上界是这一版的高：一层一轮。
    for (level, nodes_at_this_level) in nodes_at_each_level.iter().copied().enumerate() {
        let splits_at_this_level = match level {
            0 => {
                let entries_at_most = entries_before_the_publish
                    .checked_add(changes.inserted_at_most)
                    .expect("条目数加插的装得进 u64");
                if entries_at_most <= leaf_entries {
                    0
                } else {
                    splits_at_one_level_at_most(
                        nodes_at_this_level,
                        changes.inserted_at_most,
                        leaf_additions_per_split_of_a_fresh_node,
                    )
                }
            }
            _ => {
                let nodes_below_at_most =
                    nodes_at_each_level[level - 1] + splits_at_the_level_below;
                if nodes_below_at_most <= internal_entries {
                    0
                } else {
                    splits_at_one_level_at_most(
                        nodes_at_this_level,
                        splits_at_the_level_below,
                        internal_additions_per_split_of_a_fresh_node,
                    )
                }
            }
        };
        nodes_rewritten = nodes_rewritten
            .checked_add(nodes_at_this_level.min(changed_paths) + splits_at_this_level)
            .expect("重写的节点数装得进 u64");
        splits_at_the_level_below = splits_at_this_level;
    }
    // 迭代上界：每一轮新层上切的次数严格少于下一层（新根起步两个孩子，⌊(下一层切的次数 − 1) ÷ 那个数⌋ < 下一层切的次数），到 0 停。
    while splits_at_the_level_below > 0 {
        let children_of_the_new_root = splits_at_the_level_below + 1;
        let splits_of_the_new_level = if children_of_the_new_root <= internal_entries {
            0
        } else {
            (splits_at_the_level_below - 1) / internal_additions_per_split_of_a_fresh_node
        };
        nodes_rewritten = nodes_rewritten
            .checked_add(1 + splits_of_the_new_level)
            .expect("重写的节点数装得进 u64");
        splits_at_the_level_below = splits_of_the_new_level;
    }
    nodes_rewritten
}

/// 一次发布里一个角色的空间归哪一格（依据见各成员）：一次发布判不判空间准入，看它重写的角色里有没有
/// [`SpaceBudgetOfARole::OrdinaryAllocation`]（[`publish_has_an_ordinary_allocation`]）；判的时候需求是这次新写的全部角色，
/// 三格都算（[`demand_of_the_roles_on_each_device`]，D28（挂载期承诺量） 已定项 1 接线）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpaceBudgetOfARole {
    /// 普通分配：用户数据单元，与因为用户这次改了内容才重写的 extent 树、inode 树的节点。保留池对它是纯税
    /// （D23（journal 的角色与格式） 已定项 24「保留池是给 checkpoint 开的一道地板，对普通分配是纯税」）。
    OrdinaryAllocation,
    /// checkpoint 自己的固定点：ckpt_cost 的 Σ 名单里那几棵（分配记录树、中央映射树、记账树）与树表。
    /// 没有普通分配的发布（空发布、暖机、抬 F）只写它们，空间从式子第八项 checkpoint 保留池里出
    /// （D28（挂载期承诺量） 已定项 1「它保证 checkpoint 自己的固定点写得出去」），不判。
    CheckpointReservePool,
    /// 实例表链：写行那次整条重写，从挂载期承诺量里的实例切换预留出（D28（挂载期承诺量） 已定项 3「多的一份给写行那次发布的元数据」、
    /// 已定项 4「实例表链不进 ckpt_cost，它的开销归已定项 3 的切换预留」），不判。
    InstanceSwitchReserve,
}

/// 这个角色的空间归哪一格（[`SpaceBudgetOfARole`]）。
#[must_use]
pub const fn space_budget_of_role(role: TransactionUnit) -> SpaceBudgetOfARole {
    match role {
        TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot => SpaceBudgetOfARole::OrdinaryAllocation,
        TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        | TransactionUnit::AllocationTree
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable => SpaceBudgetOfARole::CheckpointReservePool,
        TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
            SpaceBudgetOfARole::InstanceSwitchReserve
        }
    }
}

/// 这次发布判不判空间准入（D28（挂载期承诺量） 已定项 1 接线「判不判照旧」）：重写的角色里至少一个是普通分配
/// （[`space_budget_of_role`] 是 [`SpaceBudgetOfARole::OrdinaryAllocation`]）才判；没有普通分配的发布（空发布、写行、暖机、抬 F）不判。
#[must_use]
pub fn publish_has_an_ordinary_allocation(rewritten_roles: &[TransactionUnit]) -> bool {
    rewritten_roles
        .iter()
        .any(|role| space_budget_of_role(*role) == SpaceBudgetOfARole::OrdinaryAllocation)
}

/// 一次发布的需求：这次新写的全部角色（普通分配加这次写出的固定点，不加这次换下的槽，D28（挂载期承诺量） 已定项 1 接线，
/// 用户 2026-09-25 定 ND+A1n）要在每块盘上新占的物理字节：
/// 第一版每个单元落池里每一块盘（`ReplicaCount::of_every_device_in_the_pool` 同一条理由），每块盘的需求相同；
/// 按盘字节记（C370（需求、可用与 df 没有共同单位） 2026-09-17 收窄：需求要与逐盘物理字节的「已分配」同口径）。
/// 按 `devices` 的次序每块盘一条，需求为零也写 0（`admit_on_every_device` 要逐盘写全）。
///
/// # Panics
/// 字节数装不进 u64：一次发布的角色数有上界（一版的节点数），装不下说明调用方给的不是一次发布的角色清单。
#[must_use]
pub fn demand_of_the_roles_on_each_device(
    rewritten_roles: &[TransactionUnit],
    devices: &[DeviceIdentity],
) -> Vec<DemandOnDevice> {
    let slots_of_the_demand: u64 = rewritten_roles.iter().map(|role| role.span_slots()).sum();
    let bytes = BytesOnOneDevice::of_slots(slots_of_the_demand);
    devices
        .iter()
        .map(|device| DemandOnDevice {
            device: *device,
            bytes,
        })
        .collect()
}

/// 读数里的实例切换预留按哪一次可写挂载的行数算（D28（挂载期承诺量） 已定项 3「发布路径用 rows0 + 1」，用户 2026-09-26 定）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstanceRowsOfTheSwitchReserve {
    /// 可写挂载自己判：这次挂载的 rows0（挂载时读到的行数加写行那次要写的行数）。
    OfThisMount,
    /// 发布路径：下一次可写挂载的 rows0 = 这次挂载的 rows0 + 1——下一次挂载必写一行，发布放行时就给那一行留出链多一片的份额。
    OfTheNextWritableMount,
}

impl InstanceRowsOfTheSwitchReserve {
    /// 这一格要按的行数：`rows_of_this_mount` 是这次挂载记在分配器上的 rows0。
    ///
    /// # Panics
    /// 行数加一装不进 u64：行数来自一次挂载读到的实例表，大到这一步说明调用方给错了量。
    #[must_use]
    pub fn instance_rows(self, rows_of_this_mount: u64) -> u64 {
        match self {
            InstanceRowsOfTheSwitchReserve::OfThisMount => rows_of_this_mount,
            InstanceRowsOfTheSwitchReserve::OfTheNextWritableMount => rows_of_this_mount
                .checked_add(1)
                .expect("行数加一装得进 u64：实例表的行数是一次挂载读出来的"),
        }
    }
}

/// 准入读数（D28（挂载期承诺量） 已定项 1 的八项）：逐盘各项取分配器此刻的计数（[`AdmissionReading::of_allocator`]）；
/// 挂载期承诺量 = 实例切换的预留（已定项 3），行数按 `rows_of_the_switch_reserve` 从这次挂载记在分配器上的 rows0 取
/// （`PoolAllocator::instance_rows_after_this_mounts_row_publish`，挂载期间常量；mkfs 同一个进程里是 0：mkfs 的实例表一行都没有、
/// 实例 1 不写行），c_max 与 checkpoint 保留池的 ckpt_cost 是同一个现算的数
/// （[`checkpoint_cost_of_the_version_to_build_on_with_node_capacities`]，中央映射树的节点容量取 `node_capacities`：
/// 写入口装的是哪一档，空发布就按哪一档切分）；待删占用与已承诺预留取第一版的两个 0。
#[must_use]
pub fn admission_reading_with_the_switch_reserve_of(
    allocator: &PoolAllocator,
    version_to_build_on: Option<&TransactionOutput>,
    rows_of_the_switch_reserve: InstanceRowsOfTheSwitchReserve,
    node_capacities: CodeTwoTreeNodeCapacities,
) -> AdmissionReading {
    let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
        version_to_build_on,
        allocator,
        node_capacities,
    );
    let instance_rows_of_the_switch_reserve = rows_of_the_switch_reserve
        .instance_rows(allocator.instance_rows_after_this_mounts_row_publish());
    AdmissionReading::of_allocator(
        allocator,
        instance_switch_reserve_on_one_device(instance_rows_of_the_switch_reserve, checkpoint_cost),
        PoolWideCommitments::of_the_first_version(checkpoint_reserve_pool(
            checkpoint_cost,
            ReplicaCount::of_every_device_in_the_pool(allocator),
        )),
    )
}

/// 一次发布之前的准入读数：实例切换的预留按下一次可写挂载的 rows0（[`InstanceRowsOfTheSwitchReserve::OfTheNextWritableMount`]），
/// 中央映射树的节点容量按节点格式算。
#[must_use]
pub fn admission_reading_before_a_publish(
    allocator: &PoolAllocator,
    version_to_build_on: Option<&TransactionOutput>,
) -> AdmissionReading {
    admission_reading_with_the_switch_reserve_of(
        allocator,
        version_to_build_on,
        InstanceRowsOfTheSwitchReserve::OfTheNextWritableMount,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    )
}

/// 可写挂载判「实例切换的预留拿得到」的读数，中央映射树的节点容量按节点格式算：
/// 同 [`admission_reading_of_a_writable_mount_with_node_capacities`] 取 `CodeTwoTreeNodeCapacities::FromTheNodeFormat`。
#[must_use]
pub fn admission_reading_of_a_writable_mount(
    allocator: &PoolAllocator,
    version_to_build_on: Option<&TransactionOutput>,
) -> AdmissionReading {
    admission_reading_of_a_writable_mount_with_node_capacities(
        allocator,
        version_to_build_on,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    )
}

/// 可写挂载判「实例切换的预留拿得到」的读数（取号之前那一判与写行之后推抬 F 再判都是这一份）：
/// 实例切换的预留按这次挂载的 rows0（[`InstanceRowsOfTheSwitchReserve::OfThisMount`]）；ckpt_cost 里中央映射树的节点容量取
/// `node_capacities`——挂载那一处传它写入口装着的那一档（`transaction::PoolWriter::code_two_tree_node_capacities`）。
#[must_use]
pub fn admission_reading_of_a_writable_mount_with_node_capacities(
    allocator: &PoolAllocator,
    version_to_build_on: Option<&TransactionOutput>,
    node_capacities: CodeTwoTreeNodeCapacities,
) -> AdmissionReading {
    admission_reading_with_the_switch_reserve_of(
        allocator,
        version_to_build_on,
        InstanceRowsOfTheSwitchReserve::OfThisMount,
        node_capacities,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::{CheckpointTxg, DataUnitIndexInFile, SlotNumber};
    use crate::allocator::{DeviceFreeMap, UnitFootprint};
    use singlefs_format::UNIT_AREA_START_SLOT;

    fn replicas_of_two_devices() -> ReplicaCount {
        ReplicaCount::new(2).expect("2 份")
    }

    fn slots(count: u64) -> BytesOnOneDevice {
        BytesOnOneDevice::of_slots(count)
    }

    fn available_slots(count: i128) -> AvailableBytesOnOneDevice {
        AvailableBytesOnOneDevice(count * i128::from(SLOT_BYTES))
    }

    /// 一块盘上只有容量、别的四项都是 0 的读数那一条。
    fn empty_device(device: u32, capacity_in_slots: u64) -> DeviceAdmissionTerms {
        DeviceAdmissionTerms {
            device: DeviceIdentity(device),
            capacity: slots(capacity_in_slots),
            allocated: BytesOnOneDevice::ZERO,
            unreclaimable: BytesOnOneDevice::ZERO,
            mount_time_commitment: BytesOnOneDevice::ZERO,
            abandoned_root_exclusive: BytesOnOneDevice::ZERO,
        }
    }

    /// 每块盘各要一个数据单元（2 槽）：一个单元两份落两块盘（C375 / C355 那两格的「请求 1 个单元」）。
    fn one_data_unit_on_each_of(devices: &[u32]) -> Vec<DemandOnDevice> {
        devices
            .iter()
            .map(|device| DemandOnDevice {
                device: DeviceIdentity(*device),
                bytes: slots(TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots()),
            })
            .collect()
    }

    /// 式子的八项各取一个互不相同的数：每一项都在、各扣一次、逐盘各取各的，少扣或多扣任何一项结果都对不上。
    /// 「已分配」里含着 defer 里的槽（读法甲），式子不另扣「defer 待释放」（D28（挂载期承诺量） 已定项 1，用户 2026-09-25 定）。
    #[test]
    fn each_of_the_eight_terms_is_subtracted_once_on_every_device() {
        let replicas = replicas_of_two_devices();
        let reading = AdmissionReading::new(
            vec![
                DeviceAdmissionTerms {
                    device: DeviceIdentity(0),
                    capacity: slots(1000),
                    allocated: slots(100),
                    unreclaimable: slots(1),
                    mount_time_commitment: slots(7),
                    abandoned_root_exclusive: slots(3),
                },
                DeviceAdmissionTerms {
                    device: DeviceIdentity(1),
                    capacity: slots(900),
                    allocated: slots(90),
                    unreclaimable: BytesOnOneDevice::ZERO,
                    mount_time_commitment: slots(7),
                    abandoned_root_exclusive: BytesOnOneDevice::ZERO,
                },
            ],
            PoolWideCommitments {
                pending_delete: BytesSummedOverAllReplicas::of_one_copy_on_every_replica(
                    slots(2),
                    replicas,
                ),
                committed_reservation: BytesSummedOverAllReplicas::of_one_copy_on_every_replica(
                    slots(4),
                    replicas,
                ),
                checkpoint_reserve_pool: checkpoint_reserve_pool(MetadataBlocks(5), replicas),
            },
            replicas,
        );
        assert_eq!(
            reading.available_on_each_device(),
            vec![
                (
                    DeviceIdentity(0),
                    available_slots(1000 - 100 - 1 - 7 - 3 - 2 - 4 - 5)
                ),
                (DeviceIdentity(1), available_slots(900 - 90 - 7 - 2 - 4 - 5)),
            ],
            "逐盘：容量减自己的五项，再各减全池三项按副本之和记、摊到每块盘的那一份"
        );
    }

    /// C375（待删占用与已承诺预留按份还是按和记没有条款） 欠的那一格：两盘各 7 槽、ckpt_cost 4、一个墓碑容器（码 3，每份 2 槽）、
    /// 请求 1 个单元。按副本之和记：每块盘扣墓碑 2 + 保留池 4，剩 1 槽 < 数据单元 2 槽 ⇒ 拒。
    /// 判别力：换成按一份副本记（记下 2 槽、摊到每块盘只扣 1），那一格翻成放行——而每块盘真要落 2 + 4 + 2 = 8 > 7。
    #[test]
    fn c375_two_devices_of_seven_slots_with_checkpoint_cost_four_and_one_tombstone_container_refuse_one_data_unit(
    ) {
        let replicas = replicas_of_two_devices();
        let tombstone_container_of_one_copy = slots(2);
        let reading = AdmissionReading::new(
            vec![empty_device(0, 7), empty_device(1, 7)],
            PoolWideCommitments {
                pending_delete: BytesSummedOverAllReplicas::ZERO,
                committed_reservation: BytesSummedOverAllReplicas::of_one_copy_on_every_replica(
                    tombstone_container_of_one_copy,
                    replicas,
                ),
                checkpoint_reserve_pool: checkpoint_reserve_pool(MetadataBlocks(4), replicas),
            },
            replicas,
        );
        let verdict = admit_on_every_device(&reading, &one_data_unit_on_each_of(&[0, 1]));
        assert_eq!(
            verdict,
            Err(AdmissionRefusedOnSomeDevices {
                short_devices: vec![
                    DeviceShortOfDemand {
                        device: DeviceIdentity(0),
                        available: available_slots(1),
                        demand: slots(2),
                    },
                    DeviceShortOfDemand {
                        device: DeviceIdentity(1),
                        available: available_slots(1),
                        demand: slots(2),
                    },
                ],
            }),
            "按副本之和记：每块盘扣墓碑容器一份 2 槽、保留池 4 槽，剩 1 槽装不下 2 槽的数据单元"
        );
    }

    /// C355（checkpoint 保留池池级扣而固定点每块要落两列） 欠的那一格：盘 0 短 1 块、盘 1 富余。逐设备合取判拒、只点名盘 0。
    /// 判别力两条：把保留池改回池级标量扣一份（每块盘只扣一半），或让盘 1 的富余补盘 0，那一格都翻成放行——
    /// 而固定点每一块要在盘 0 上落一份，盘 0 真装不下。
    #[test]
    fn c355_a_device_one_slot_short_is_refused_even_when_the_other_device_has_plenty() {
        let replicas = replicas_of_two_devices();
        let reading = AdmissionReading::new(
            vec![empty_device(0, 5), empty_device(1, 100)],
            PoolWideCommitments::of_the_first_version(checkpoint_reserve_pool(
                MetadataBlocks(4),
                replicas,
            )),
            replicas,
        );
        let verdict = admit_on_every_device(&reading, &one_data_unit_on_each_of(&[0, 1]));
        assert_eq!(
            verdict,
            Err(AdmissionRefusedOnSomeDevices {
                short_devices: vec![DeviceShortOfDemand {
                    device: DeviceIdentity(0),
                    available: available_slots(1),
                    demand: slots(2),
                }],
            }),
            "盘 0：5 − 保留池 4 = 1 < 2；盘 1 的 96 补不了它"
        );
    }

    /// D28（挂载期承诺量） 已定项 3：(N_switch + 1) × (32768 × 片数 + R × c_max × 16384)，每块盘各一份。
    /// rows0 = 0、c_max = 0 时每块盘 4 × 32768 = 8 块（C126（切换预留的最坏量没有口径） 记的两份之和 16 块的一半）；
    /// rows0 + 3 越过 369 那一刻链长一片。
    #[test]
    fn instance_switch_reserve_counts_four_chain_rewrites_and_twelve_warm_up_publishes_on_each_device(
    ) {
        assert_eq!(
            instance_switch_reserve_on_one_device(0, MetadataBlocks(0)),
            slots(8),
            "rows0 0、c_max 0：4 次链重写，一片 2 槽"
        );
        assert_eq!(
            instance_switch_reserve_on_one_device(1, MetadataBlocks(4)),
            slots(4 * (2 + 3 * 4)),
            "rows0 1、c_max 4：每份 2 槽链重写 + 3 次空发布各 4 块"
        );
        assert_eq!(
            instance_switch_reserve_on_one_device(366, MetadataBlocks(0)),
            slots(4 * 2),
            "366 + 3 = 369 行：一片装得下"
        );
        assert_eq!(
            instance_switch_reserve_on_one_device(367, MetadataBlocks(0)),
            slots(4 * 2 * 2),
            "367 + 3 = 370 行：两片"
        );
    }

    /// ckpt_cost 里分配记录树那一项按盘分路、按最坏情况（D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定）：每块盘两条从叶到根之下那一层的路径
    /// 加共用的根。两块盘高 2 / 3 / 4（1 GiB / 4 GiB 与 64 GiB / 1 TiB）是 5 / 9 / 13，三块盘高 3 是 13；
    /// 每块盘只算一条路径是 3 / 5 / 7 与 7，按树高算是 2 / 3 / 4 与 3。
    #[test]
    fn allocation_record_tree_term_is_two_leaf_paths_on_each_device_plus_the_shared_root() {
        assert_eq!(
            [(2, 2), (3, 2), (4, 2), (3, 3)].map(|(tree_height, devices_in_the_pool)| {
                allocation_record_tree_nodes_on_two_leaf_paths_per_device(
                    tree_height,
                    devices_in_the_pool,
                )
            }),
            [5, 9, 13, 13],
            "盘数 × 2 × (高 − 1) + 1"
        );
    }

    /// 产品容量（叶 294、内部 143，按节点格式算）的中央映射树。
    fn central_mapping_capacity_of_the_node_format() -> CodeTwoTreeNodeCapacity {
        MultiLevelCodeTwoTree::CentralMapping.node_capacity_of_the_node_format()
    }

    /// 两块 4 GiB 盘上一次空发布改的映射条目：分配记录树至多 9 个节点、记账树 1 个，各删一把旧 key、插一把新 key。
    const CHANGES_OF_AN_EMPTY_PUBLISH_ON_TWO_FOUR_GIBIBYTE_DEVICES:
        CentralMappingEntryChangesOfAnEmptyPublish = CentralMappingEntryChangesOfAnEmptyPublish {
        deleted_at_most: 10,
        inserted_at_most: 10,
    };

    /// 中央映射树逐层按可能改的路径数与可能切出来的节点数计（D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定按最坏情况）：
    /// - 一层、条目加插的装得进一片叶：一次都切不了，重写根一个；
    /// - 一层、装不下：根（叶）切一次（10 条插不满第二次，⌊9 ÷ 147⌋ = 0），长出新根：1 + 1 + 1 = 3；
    /// - 两层（叶 2、根 1）：叶层 min(2, 20) + 切 min(2, 10) = 4，根层装得下 4 个孩子不切、1 个 ⇒ 5；
    /// - 三层（叶 100、内部 10、根 1）：叶层 min(100, 20) + 切 10 = 30，内部层 110 个孩子装得下（143）不切、min(10, 20) = 10，根 1 ⇒ 41。
    ///
    /// 判别力：只按树高算是 1 / 1 / 2 / 3；不算切出来的节点是 1 / 1 / 3 / 31；不算长出来的新根是 1 / 2 / 5 / 41。
    #[test]
    fn central_mapping_term_counts_changed_paths_and_splits_on_every_level() {
        let capacity = central_mapping_capacity_of_the_node_format();
        assert_eq!(
            (capacity.leaf_entries, capacity.internal_entries),
            (294, 143),
            "节点格式算出来的中央映射树容量"
        );
        let changes = CHANGES_OF_AN_EMPTY_PUBLISH_ON_TWO_FOUR_GIBIBYTE_DEVICES;
        assert_eq!(
            [
                (vec![1], 20),
                (vec![1], 290),
                (vec![2, 1], 312),
                (vec![100, 10, 1], 20_000),
            ]
            .map(|(nodes_at_each_level, entries)| {
                central_mapping_tree_nodes_an_empty_publish_rewrites_at_most(
                    &nodes_at_each_level,
                    entries,
                    changes,
                    capacity,
                )
            }),
            [1, 3, 5, 41],
            "每层 min(节点数, 删 + 插) + 切出来的，根切开再加新根"
        );
    }

    /// 压小容量下内部层照样切、连着长高：叶 2、内部 2（`CodeTwoTreeNodeCapacities::CappedForTests` 最小的那一档）、插 2 删 2。
    /// - 一层（一片装满 2 条的叶）：叶层 min(1, 4) + 切 ≤ min(1, 2) + ⌊1 ÷ 1⌋ = 2 ⇒ 3；新根起步两个孩子、再多一个就装不下：
    ///   第一层新根 1 + 切 ⌊(2 − 1) ÷ 1⌋ = 1 ⇒ 2，再长一层 1 + 0 ⇒ 1。合计 6。
    /// - 两层（叶 3、根 1，条目 6）：叶层 min(3, 4) + 切 min(3, 2) + 0 = 5；根层下一层至多 3 + 2 = 5 个节点装不下 2 个孩子，
    ///   切 min(1, 2) + ⌊1 ÷ 1⌋ = 2 ⇒ 1 + 2 = 3；新根 1 + 切 1 ⇒ 2，再长一层 1。合计 11。
    ///
    /// 判别力：新根只算一个、不看它还会不会切是 4 与 9；内部层一次都不按会切算，两层那一格是 6；叶层一次都不按会切算是 1 与 4。
    #[test]
    fn central_mapping_term_keeps_growing_new_roots_while_they_overflow_under_capped_capacities() {
        let smallest_capacity = CodeTwoTreeNodeCapacity {
            leaf_entries: 2,
            internal_entries: 2,
        };
        let changes = CentralMappingEntryChangesOfAnEmptyPublish {
            deleted_at_most: 2,
            inserted_at_most: 2,
        };
        assert_eq!(
            [(vec![1], 2), (vec![3, 1], 6)].map(|(nodes_at_each_level, entries)| {
                central_mapping_tree_nodes_an_empty_publish_rewrites_at_most(
                    &nodes_at_each_level,
                    entries,
                    changes,
                    smallest_capacity,
                )
            }),
            [6, 11],
            "叶层、内部层、长出来的每一层新根都按会切算"
        );
    }

    /// 摊到每块盘除不尽时向上取整：扣多不扣少。
    #[test]
    fn pool_wide_commitment_not_divisible_by_the_replica_count_rounds_the_share_up() {
        assert_eq!(
            BytesSummedOverAllReplicas(5).share_of_one_device(replicas_of_two_devices()),
            BytesOnOneDevice(3),
            "5 字节摊两块盘：每块扣 3，不是 2（向下取整会多报可用）"
        );
        assert_eq!(
            BytesSummedOverAllReplicas(4).share_of_one_device(replicas_of_two_devices()),
            BytesOnOneDevice(2),
            "除得尽时正好一半"
        );
    }

    /// 从分配器取读数：每块盘取自己的单元区、占着的槽（含 defer 里的）与隔离的槽；副本数 = 盘数。
    #[test]
    fn reading_of_an_allocator_takes_each_devices_own_capacity_allocated_and_isolated_slots() {
        let unit_area_slots = 256;
        let device_bytes = (UNIT_AREA_START_SLOT + unit_area_slots) * SLOT_BYTES;
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
            DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
        ]);
        let data = allocator
            .allocate_user_data(CheckpointTxg(1))
            .expect("空盘上分得到数据单元");
        allocator
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(1))
            .expect("空盘上分得到一个节点");
        allocator.release(data, CheckpointTxg(2));
        allocator.isolate_abandoned(DeviceIdentity(0), SlotNumber(UNIT_AREA_START_SLOT + 200), 2);
        let mount_time_commitment = slots(11);
        let reading = AdmissionReading::of_allocator(
            &allocator,
            mount_time_commitment,
            PoolWideCommitments::of_the_first_version(BytesSummedOverAllReplicas::ZERO),
        );
        assert_eq!(reading.replicas().get(), 2, "每个单元落两块盘各一份");
        let per_device_expectation = |device: u32, isolated_slots: u64| DeviceAdmissionTerms {
            device: DeviceIdentity(device),
            capacity: slots(unit_area_slots),
            allocated: slots(3),
            unreclaimable: BytesOnOneDevice::ZERO,
            mount_time_commitment,
            abandoned_root_exclusive: slots(isolated_slots),
        };
        assert_eq!(
            reading.per_device(),
            &[per_device_expectation(0, 2), per_device_expectation(1, 0)],
            "已分配含 defer 里的 2 槽（读法甲）；隔离只在盘 0 上"
        );
        assert_eq!(
            reading.available_on_each_device(),
            vec![
                (DeviceIdentity(0), available_slots(256 - 3 - 11 - 2)),
                (DeviceIdentity(1), available_slots(256 - 3 - 11)),
            ],
            "defer 里的 2 槽只在「已分配」里扣一次，不另扣「defer 待释放」"
        );
    }

    /// 需求 = 这次新写的全部角色（普通分配加固定点，D28（挂载期承诺量） 已定项 1 接线）；判不判只看有没有普通分配。
    /// 一次覆盖写的角色：数据单元 2 槽、extent 根 1、inode 叶容器 2、inode 根 1、分配记录树 1、记账树 1、映射树 1、树表 1。
    #[test]
    fn demand_counts_every_rewritten_role_and_only_a_publish_with_an_ordinary_allocation_is_judged()
    {
        let overwrite_roles = [
            TransactionUnit::Data(DataUnitIndexInFile::FIRST),
            TransactionUnit::ExtentRoot,
            TransactionUnit::InodeLeafContainer(
                crate::inode_tree::InodeLeafContainerIndexInTree::LEFTMOST,
            ),
            TransactionUnit::InodeRoot,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ];
        let devices = [DeviceIdentity(0), DeviceIdentity(1)];
        assert_eq!(
            demand_of_the_roles_on_each_device(&overwrite_roles, &devices),
            devices
                .iter()
                .map(|device| DemandOnDevice {
                    device: *device,
                    bytes: slots(2 + 1 + 2 + 1 + 1 + 1 + 1 + 1),
                })
                .collect::<Vec<_>>(),
            "每块盘的需求是这次写出的全部 10 槽，固定点的 4 槽也在内"
        );
        assert!(
            publish_has_an_ordinary_allocation(&overwrite_roles),
            "覆盖写有普通分配：判"
        );
        let empty_publish_roles = [
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ];
        assert!(
            !publish_has_an_ordinary_allocation(&empty_publish_roles),
            "空发布只写固定点：不判"
        );
        assert!(
            !publish_has_an_ordinary_allocation(&[
                TransactionUnit::InstanceTable,
                TransactionUnit::AllocationTree,
                TransactionUnit::TreeTable,
            ]),
            "写行只写实例表链与固定点：不判"
        );
    }

    /// D28（挂载期承诺量） 已定项 3「发布路径用 rows0 + 1」：rows0 = 366 时这次挂载的链是 ⌈(366 + 3) ÷ 369⌉ = 1 片，
    /// 下一次可写挂载要写一行、rows0 = 367，链是 ⌈370 ÷ 369⌉ = 2 片。发布路径的读数按 2 片留份额（每块盘多 4 × 2 = 8 槽），
    /// 可写挂载自己判时照旧按 1 片。
    #[test]
    fn publish_reading_reserves_the_switch_share_of_the_next_writable_mounts_rows_and_the_mount_reading_its_own(
    ) {
        let unit_area_slots = 384;
        let device_bytes = (UNIT_AREA_START_SLOT + unit_area_slots) * SLOT_BYTES;
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
            DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
        ]);
        allocator.record_instance_rows_after_this_mounts_row_publish(366);
        let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on(None, &allocator);
        let publish_reading = admission_reading_before_a_publish(&allocator, None);
        let mount_reading = admission_reading_of_a_writable_mount(&allocator, None);
        let commitment_of = |reading: &AdmissionReading| {
            reading
                .per_device()
                .iter()
                .map(|terms| terms.mount_time_commitment)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            commitment_of(&publish_reading),
            vec![instance_switch_reserve_on_one_device(367, checkpoint_cost); 2],
            "发布路径按下一次挂载的 367 行"
        );
        assert_eq!(
            commitment_of(&mount_reading),
            vec![instance_switch_reserve_on_one_device(366, checkpoint_cost); 2],
            "可写挂载按这次的 366 行"
        );
        assert_eq!(
            instance_switch_reserve_on_one_device(367, checkpoint_cost).0,
            instance_switch_reserve_on_one_device(366, checkpoint_cost).0 + slots(8).0,
            "两片比一片每块盘多 4 次链重写 × 2 槽"
        );
    }
}
