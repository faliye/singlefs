//! 分配准入的读数与逐设备合取（里程碑「第二个事务」增补 2 收口表第 5 行）：D28（挂载期承诺量） 已定项 1 的九项式子逐设备算可用，
//! D3（空间分配） 已定项 7 合取表第 1 条「可用 ≥ 需求」按逐设备合取判——每一块盘都够才放行，一块盘不够就拒，
//! 不拿别的盘的富余补（2026-09-23 用户定 C355（checkpoint 保留池池级扣而固定点每块要落两列） 取丁）。
//! 前六项取这块盘自己的值；待删占用、已承诺预留、checkpoint 保留池三项是全池的承诺量，按全部副本之和记物理字节，
//! 每块盘各扣「副本之和 ÷ 副本数」（C375（待删占用与已承诺预留按份还是按和记没有条款） 第三轮写回）。
//!
//! 全是只读的纯函数：不动分配器、不发一个写。
//!
//! 接在两处（C363 (b) 判决 `research/prompts/c363b-r1-main-verification.md` 第四节第 2 条，里程碑「第二个事务」增补 2 收口表第 5 行）：
//! - 发布路径：`transaction::prepare_the_version_publish` 在算定这次发布的样子之后、读盘核与动分配器之前，按
//!   [`admission_reading_before_a_publish`] 取读数、按 [`demand_of_the_roles_on_each_device`] 取这次的需求，逐设备合取；
//! - 可写挂载：`mount::establish_instance` 在取号之前按同一份读数判「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13），需求逐盘 0。
//!
//! 条款给的量：checkpoint 保留池的 ckpt_cost 按 D28（挂载期承诺量） 已定项 4 的 Σ 名单（分配记录树、中央映射树按树高，
//! 记账树按每发布的节点数，树表一项，实例表链不进；[`checkpoint_cost_of_the_version_to_build_on`]）；挂载期承诺量暖机那一半的 c_max
//! 与保留池「按同一个现算的 c_max 取」（D28（挂载期承诺量） 已定项 4 末条）。
//!
//! ⚠️ 实现员取的读法（条款没写，交主 agent；[`space_budget_of_role`] 与 [`admission_reading_before_a_publish`] 的文档注释写了依据）：
//! - 需求只算这次发布的普通分配（用户数据单元、extent 树与 inode 树的节点），固定点与实例表链各有自己那一项保留，不重复算需求；
//!   一次发布一个普通分配都没有（空发布、写行）就不判——它们的空间在保留池与切换预留里，判它们会让推空发布抬 F（D3（空间分配）
//!   已定项 17「释放空间这个操作本身不需要申请空间」）与挂载自己那一串被式子挡住；
//! - 需求按盘字节记（C370（需求、可用与 df 没有共同单位） 2026-09-17 收窄：与「已分配」同口径只能读成盘字节），第一版每个单元落每块盘，
//!   每块盘的需求相同；
//! - 读数取分配器此刻的计数（挂载时是回收与影子账隔离之后、写行之前那一刻；admission 读哪一个在那两段里条款没写，见
//!   [`AdmissionReading::of_allocator`]）。
//!
//! 准入不够时先推空发布抬 F 再判（D16（发布语义） 已定项 1，C283（准入失败时不先推发布就报 ENOSPC））没有实现：直接在任何写之前拒。

use std::collections::BTreeSet;
use std::num::NonZeroU64;

use singlefs_format::{INSTANCE_TABLE_PAGE_RECORDS, SLOT_BYTES};

use crate::address::DeviceIdentity;
use crate::allocation_record_tree::AllocationRecordTreeGeometry;
use crate::allocator::PoolAllocator;
use crate::transaction::{MultiLevelCodeTwoTree, TransactionOutput, TransactionUnit};

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

/// 式子前六项里这块盘自己的值（D28（挂载期承诺量） 已定项 1「前六项取这块盘自己的值」），全按这块盘上的物理字节。
/// 每个字段的文档先写它是式子里的哪一项（名字照式子逐字），再写从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceAdmissionTerms {
    pub device: DeviceIdentity,
    /// 式子里的「容量」：这块盘单元区的大小（D28（挂载期承诺量） 已定项 4「『容量』是单元区大小」）。
    pub capacity: BytesOnOneDevice,
    /// 式子里的「已分配」（记账第 1 项「已分配字节」）：分配器的占着槽数——仍分配的加上已释放、还在 defer 窗口里的
    /// （I-3.1（已分配统计对得上） 读法甲，2026-09-14 用户定）。
    /// ⚠️ 它含 defer，而式子另扣一次「defer 待释放」：按读法甲 defer 扣两次。删不删那一项是增补 2 收口表第 ② 行
    /// alloc-basis 岔路 2，还开着；这里照式子逐字扣，不替它定。
    pub allocated: BytesOnOneDevice,
    /// 式子里的「不可回收」（记账第 3 项）。
    pub unreclaimable: BytesOnOneDevice,
    /// 式子里的「defer 待释放」（记账第 5 项）。
    pub deferred: BytesOnOneDevice,
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

/// 准入读数：式子的九项，逐盘六项加全池三项，外加摊全池三项要用的副本数。
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

    /// 从分配器此刻的计数取逐盘各项：容量 = 单元区槽数、已分配 = 占着的槽数、defer 待释放 = 已释放还在窗口里的槽数、
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
                deferred: BytesOnOneDevice::of_slots(device_map.deferred_slots()),
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
    /// 容量 − 已分配 − 不可回收 − defer 待释放 − 挂载期承诺量 − 被抛弃根独占量
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
                    terms.deferred,
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

/// checkpoint 保留池的 ckpt_cost（D28（挂载期承诺量） 已定项 4，Σ 名单照 `research/prompts/c363b-r1-main-verification.md` 第三节 V2-2）：
/// Σ（分配记录树与中央映射树当前的高）+ 记账树每发布的节点数 + 1（树表每次发布重写一个单元，D16（发布语义） 已定项 9）；
/// 实例表链不进（它的开销归已定项 3 的切换预留）。「当前」= 这次发布要接在后面的那一版：
/// - 带文件的一版：两棵树的高从各自根节点的码 2 头里现读（层级 + 1，不用内存里另存一份，已定项 4）；记账树每次发布整批重写
///   （`transaction` 装记账行那一段），每发布的节点数就是这一版记账树的节点数。
/// - 树表 0 条的一版（`None`）：没有中央映射树与记账树（0 与 0）；分配记录树只在那一版写过行时有（分配器记着它，
///   `PoolAllocator::allocation_record_tree_of_the_version_without_file`），它按位置寻址、根的层级由池几何定（D8（核心索引结构） 已定项 14），
///   高取几何的高——这一处分配器里只有节点与指针、没有根节点的字节可读；没写过行的（mkfs 的第 0 代）一棵都没有，取 0。
///
/// 同一个数也是挂载期承诺量里暖机那一半的 c_max（已定项 4 末条「按同一个现算的 c_max 取」）。
#[must_use]
pub fn checkpoint_cost_of_the_version_to_build_on(
    version_to_build_on: Option<&TransactionOutput>,
    allocator: &PoolAllocator,
) -> MetadataBlocks {
    const TREE_TABLE_UNITS_PER_PUBLISH: u64 = 1;
    let record_trees_and_accounting_nodes = match version_to_build_on {
        Some(file_version) => {
            file_version
                .position_addressed_tree_heights_read_from_the_root_node_headers()
                .allocation_record_tree
                + file_version
                    .height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping)
                + u64::try_from(file_version.accounting_tree.node_count())
                    .expect("记账树的节点数装得进 u64")
        }
        None => match allocator.allocation_record_tree_of_the_version_without_file() {
            Some(_) => AllocationRecordTreeGeometry::of_allocator(allocator).height(),
            None => 0,
        },
    };
    MetadataBlocks(record_trees_and_accounting_nodes + TREE_TABLE_UNITS_PER_PUBLISH)
}

/// 一次发布里一个角色的空间归哪一格（实现员取的读法，条款没写；依据见各成员）：
/// 准入不等式另一边的「需求」只算 [`SpaceBudgetOfARole::Demand`] 那一格，另外两格已经作为式子里的一项扣在「可用」那一边。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpaceBudgetOfARole {
    /// 普通分配：用户数据单元，与因为用户这次改了内容才重写的 extent 树、inode 树的节点。保留池对它是纯税
    /// （D23（journal 的角色与格式） 已定项 24「保留池是给 checkpoint 开的一道地板，对普通分配是纯税」）。
    Demand,
    /// checkpoint 自己的固定点：ckpt_cost 的 Σ 名单里那几棵（分配记录树、中央映射树、记账树）与树表。
    /// 它们从式子第八项 checkpoint 保留池里出（D28（挂载期承诺量） 已定项 1「它保证 checkpoint 自己的固定点写得出去」）。
    CheckpointReservePool,
    /// 实例表链：写行那次整条重写，从挂载期承诺量里的实例切换预留出（D28（挂载期承诺量） 已定项 3「多的一份给写行那次发布的元数据」、
    /// 已定项 4「实例表链不进 ckpt_cost，它的开销归已定项 3 的切换预留」）。
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
        | TransactionUnit::InodeRoot => SpaceBudgetOfARole::Demand,
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

/// 一次发布重写的角色里算需求的那几个（[`space_budget_of_role`] 是 `Demand` 的）要在每块盘上新占的物理字节：
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
    let slots_of_the_demand: u64 = rewritten_roles
        .iter()
        .filter(|role| space_budget_of_role(**role) == SpaceBudgetOfARole::Demand)
        .map(|role| role.span_slots())
        .sum();
    let bytes = BytesOnOneDevice::of_slots(slots_of_the_demand);
    devices
        .iter()
        .map(|device| DemandOnDevice {
            device: *device,
            bytes,
        })
        .collect()
}

/// 一次发布之前的准入读数（D28（挂载期承诺量） 已定项 1 的九项）：逐盘各项取分配器此刻的计数（[`AdmissionReading::of_allocator`]）；
/// 挂载期承诺量 = 实例切换的预留（已定项 3），rows0 取这次挂载记在分配器上的那个数
/// （`PoolAllocator::instance_rows_after_this_mounts_row_publish`，挂载期间常量；mkfs 同一个进程里是 0：mkfs 的实例表一行都没有、
/// 实例 1 不写行），c_max 与 checkpoint 保留池的 ckpt_cost 是同一个现算的数（[`checkpoint_cost_of_the_version_to_build_on`]）；
/// 待删占用与已承诺预留取第一版的两个 0。可写挂载在取号之前判的也是这一份（挂载那一刻 rows0 已经记在分配器上）。
#[must_use]
pub fn admission_reading_before_a_publish(
    allocator: &PoolAllocator,
    version_to_build_on: Option<&TransactionOutput>,
) -> AdmissionReading {
    let checkpoint_cost =
        checkpoint_cost_of_the_version_to_build_on(version_to_build_on, allocator);
    AdmissionReading::of_allocator(
        allocator,
        instance_switch_reserve_on_one_device(
            allocator.instance_rows_after_this_mounts_row_publish(),
            checkpoint_cost,
        ),
        PoolWideCommitments::of_the_first_version(checkpoint_reserve_pool(
            checkpoint_cost,
            ReplicaCount::of_every_device_in_the_pool(allocator),
        )),
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

    /// 一块盘上只有容量、别的前五项都是 0 的读数那一条。
    fn empty_device(device: u32, capacity_in_slots: u64) -> DeviceAdmissionTerms {
        DeviceAdmissionTerms {
            device: DeviceIdentity(device),
            capacity: slots(capacity_in_slots),
            allocated: BytesOnOneDevice::ZERO,
            unreclaimable: BytesOnOneDevice::ZERO,
            deferred: BytesOnOneDevice::ZERO,
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

    /// 九项各取一个互不相同的数：每一项都在、各扣一次、逐盘各取各的，少扣或多扣任何一项结果都对不上。
    #[test]
    fn each_of_the_nine_terms_is_subtracted_once_on_every_device() {
        let replicas = replicas_of_two_devices();
        let reading = AdmissionReading::new(
            vec![
                DeviceAdmissionTerms {
                    device: DeviceIdentity(0),
                    capacity: slots(1000),
                    allocated: slots(100),
                    unreclaimable: slots(1),
                    deferred: slots(20),
                    mount_time_commitment: slots(7),
                    abandoned_root_exclusive: slots(3),
                },
                DeviceAdmissionTerms {
                    device: DeviceIdentity(1),
                    capacity: slots(900),
                    allocated: slots(90),
                    unreclaimable: BytesOnOneDevice::ZERO,
                    deferred: slots(10),
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
                    available_slots(1000 - 100 - 1 - 20 - 7 - 3 - 2 - 4 - 5)
                ),
                (
                    DeviceIdentity(1),
                    available_slots(900 - 90 - 10 - 7 - 2 - 4 - 5)
                ),
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

    /// 摊到每块盘除不尽时向上取整：扣多不扣少。
    #[test]
    fn a_pool_wide_commitment_not_divisible_by_the_replica_count_rounds_the_share_up() {
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

    /// 从分配器取读数：每块盘取自己的单元区、占着的槽、defer 里的槽与隔离的槽；副本数 = 盘数。
    #[test]
    fn a_reading_of_an_allocator_takes_each_devices_own_capacity_allocated_deferred_and_isolated_slots(
    ) {
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
            deferred: slots(2),
            mount_time_commitment,
            abandoned_root_exclusive: slots(isolated_slots),
        };
        assert_eq!(
            reading.per_device(),
            &[per_device_expectation(0, 2), per_device_expectation(1, 0)],
            "已分配含 defer 里的 2 槽（读法甲）；隔离只在盘 0 上"
        );
    }
}
