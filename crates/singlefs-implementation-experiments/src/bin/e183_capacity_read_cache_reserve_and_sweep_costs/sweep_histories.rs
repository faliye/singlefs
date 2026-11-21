use super::*;

// ============================================================================================
// E36：H5 的清扫历史（登记 X16–X21、5.2、5.3）。4 GiB × 2、L = 768 MiB、内存稀疏盘；一格一片。
// ============================================================================================

/// 阈值候选（登记 X17）。
const SWEEP_THRESHOLD_CANDIDATE_ROWS: [u64; 4] = [64, 365, 369, 738];
/// 「交给后台」对照（登记 5.2）：S = 8、正常卸载、400 次挂载。
const SWEEP_CONTROL_SLOTS_PER_REGION: u64 = 8;
const SWEEP_CONTROL_MOUNTS: u64 = 400;
/// 读法甲（登记 X19）：链重写那一项取一片 ⟺ rows0 ≤ 366。
const ROWS_KEPT_ON_ONE_CHAIN_PAGE: u64 = 366;
/// 判别力自证与 R5c 的门槛：W_θ ÷ 四个 θ 里最小的 W ≤ 1.10（登记第六节 R5c），按千分比。
pub(super) const WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE: u128 = 1100;
/// 清扫历史结束于第二次清扫发布（登记 X16）。
const SWEEPS_TO_FINISH_A_HISTORY: usize = 2;
/// 根环区域落在哪块盘上：[0, 1, 0]（登记 X1，同 `make_filesystem_parameters` 的 `region_devices`）。
const ROOT_RING_REGION_DEVICES: [u64; 3] = [0, 1, 0];
/// 锚点模型里 mkfs 那个进程占的 txg：mkfs 0、暖机 1 与 2、首个文件 3，都是实例 1（`anchors_e183.py` 的 `owner`）。
const MAKE_FILESYSTEM_PROCESS_LAST_TXG: u64 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UnmountKind {
    Normal,
    Crash,
}

impl UnmountKind {
    pub(super) fn name(self) -> &'static str {
        match self {
            UnmountKind::Normal => "normal_unmount",
            UnmountKind::Crash => "crash",
        }
    }
}

/// 清扫在挂载末尾怎么走：开关关着、阈值 θ（四条臂）；开关开着、交给后台（对照臂）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SweepArm {
    AtTheEndOfTheMountAboveTheThreshold { threshold_rows: u64 },
    HandedToTheBackground,
}

impl SweepArm {
    pub(super) fn configuration(self) -> InstanceTableSweepConfiguration {
        match self {
            SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows } => {
                InstanceTableSweepConfiguration {
                    switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
                    row_count_threshold: InstanceTableRowCountThreshold(threshold_rows),
                    sweep_publish_placements:
                        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
                }
            }
            SweepArm::HandedToTheBackground => InstanceTableSweepConfiguration {
                switch: InstanceTableSweepSwitch::HandedToTheBackgroundSweepWhileMounted,
                row_count_threshold:
                    InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE,
                sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
            },
        }
    }

    pub(super) fn threshold_rows(self) -> Option<u64> {
        match self {
            SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows } => {
                Some(threshold_rows)
            }
            SweepArm::HandedToTheBackground => None,
        }
    }
}

/// K33 的装置侧判法：开关关着时，发了清扫发布 ⟺ 那一次 rows0 > θ。
fn sweep_is_due(rows0: u64, threshold_rows: u64) -> bool {
    rows0 > threshold_rows
}

/// 一格：S、清扫臂、卸载方式、挂载次数上限（四条臂 2θ + 3S + 20，对照 400）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SweepHistoryCell {
    pub(super) slots_per_region: u64,
    pub(super) arm: SweepArm,
    pub(super) unmount_kind: UnmountKind,
    mounts_at_most: u64,
}

impl SweepHistoryCell {
    pub(super) fn name(self) -> String {
        match self.arm {
            SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows } => format!(
                "h5-slots-per-region-{}-threshold-{threshold_rows}-{}",
                self.slots_per_region,
                self.unmount_kind.name()
            ),
            SweepArm::HandedToTheBackground => format!(
                "h5-control-slots-per-region-{}-handed-to-the-background-{}",
                self.slots_per_region,
                self.unmount_kind.name()
            ),
        }
    }
}

/// 24 格（次序同锚点脚本：S 在外、θ、卸载方式）加对照 1 格。
pub(super) fn sweep_history_cells() -> Vec<SweepHistoryCell> {
    let mut cells = Vec::new();
    for slots_per_region in SLOTS_PER_REGION_SAMPLING_POINTS {
        for threshold_rows in SWEEP_THRESHOLD_CANDIDATE_ROWS {
            for unmount_kind in [UnmountKind::Normal, UnmountKind::Crash] {
                cells.push(SweepHistoryCell {
                    slots_per_region,
                    arm: SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows },
                    unmount_kind,
                    mounts_at_most: 2 * threshold_rows + ROOT_RING_REGIONS * slots_per_region + 20,
                });
            }
        }
    }
    cells.push(SweepHistoryCell {
        slots_per_region: SWEEP_CONTROL_SLOTS_PER_REGION,
        arm: SweepArm::HandedToTheBackground,
        unmount_kind: UnmountKind::Normal,
        mounts_at_most: SWEEP_CONTROL_MOUNTS,
    });
    cells
}

/// 确定性核（登记第五节「段首那一格多跑 1 轮」）：第一格再跑一片，片名前缀 `h5-repeat-of-`。
pub(super) const SWEEP_REPEAT_SHARD_PREFIX: &str = "h5-repeat-of-";

/// 全部片：25 格各一片，加段首那一格的重跑一片。
pub(super) fn sweep_shards() -> Vec<(String, SweepHistoryCell)> {
    let cells = sweep_history_cells();
    let mut shards: Vec<(String, SweepHistoryCell)> =
        cells.iter().map(|cell| (cell.name(), *cell)).collect();
    shards.push((
        format!("{SWEEP_REPEAT_SHARD_PREFIX}{}", cells[0].name()),
        cells[0],
    ));
    shards
}

/// 锚点模型（`anchors_e183.py` 的 `h5`，登记第十三节命令一，本地移植；与登记那 24 行逐项比，`REGISTERED_SWEEP_HISTORY_ANCHORS`）里的一次挂载。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AnchorModelMount {
    pub(super) rows0: u64,
    /// 这次挂载发了清扫：删了几行、删完剩几行。
    sweep: Option<(u64, u64)>,
    /// 这次挂载交回时（清扫发布之后、卸载之前）现行那一版的 txg。
    checkpoint_txg_when_the_mount_returns: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AnchorModelHistory {
    pub(super) mounts: Vec<AnchorModelMount>,
    final_txg: u64,
}

impl AnchorModelHistory {
    fn sweeps(&self) -> Vec<(u64, u64, u64)> {
        self.mounts
            .iter()
            .zip(1u64..)
            .filter_map(|(mount, number)| {
                mount
                    .sweep
                    .map(|(deleted, residual)| (number, deleted, residual))
            })
            .collect()
    }

    fn peak_rows0(&self) -> u64 {
        self.mounts
            .iter()
            .map(|mount| mount.rows0)
            .max()
            .unwrap_or(0)
    }
}

/// 移植 `anchors_e183.py` 的 `h5`：每次可写挂载取新实例号、写上一个实例那一行、暖机到两块盘上各有一条本实例的根（txg u 的根落盘
/// `ROOT_RING_REGION_DEVICES[u % 3]`）；行数 > θ 就清扫，删去根环（最近 3S 个 txg）里一条根都没有的实例的行，清扫发布占一个 txg；
/// 正常卸载推空发布：现行 txg 除以 3 余 1 时 3 次、否则 2 次。`threshold_rows` 为 `None` 是「交给后台」（从不清扫）。
fn sweep_history_by_the_anchor_model(
    slots_per_region: u64,
    threshold_rows: Option<u64>,
    unmount_kind: UnmountKind,
    mounts_at_most: u64,
) -> AnchorModelHistory {
    let ring_roots = ROOT_RING_REGIONS * slots_per_region;
    let mut owner: BTreeMap<u64, u64> = (0..=MAKE_FILESYSTEM_PROCESS_LAST_TXG)
        .map(|txg| (txg, 1))
        .collect();
    let mut txg = MAKE_FILESYSTEM_PROCESS_LAST_TXG;
    let mut instance = 1u64;
    let push_empty_publishes_of_an_unmount =
        |current_txg: &mut u64, owners: &mut BTreeMap<u64, u64>, unmounting_instance: u64| {
            let publishes = if *current_txg % 3 == 1 { 3 } else { 2 };
            for _ in 0..publishes {
                *current_txg += 1;
                owners.insert(*current_txg, unmounting_instance);
            }
        };
    push_empty_publishes_of_an_unmount(&mut txg, &mut owner, instance);
    let mut rows: BTreeSet<u64> = BTreeSet::new();
    let mut mounts = Vec::new();
    let mut sweeps_done = 0;
    for _ in 0..mounts_at_most {
        instance += 1;
        rows.insert(instance - 1);
        txg += 1;
        owner.insert(txg, instance);
        let mut covered: BTreeSet<u64> =
            BTreeSet::from([ROOT_RING_REGION_DEVICES[usize::try_from(txg % 3).expect("小于 3")]]);
        while covered.len() < 2 {
            txg += 1;
            owner.insert(txg, instance);
            covered.insert(ROOT_RING_REGION_DEVICES[usize::try_from(txg % 3).expect("小于 3")]);
        }
        let rows0 = u64::try_from(rows.len()).expect("行数装得进 u64");
        let mut sweep = None;
        if threshold_rows.is_some_and(|threshold| sweep_is_due(rows0, threshold)) {
            let first_in_the_ring = (txg + 1).saturating_sub(ring_roots);
            let in_the_ring: BTreeSet<u64> = (first_in_the_ring..=txg)
                .map(|ring_txg| owner[&ring_txg])
                .collect();
            let deletable: BTreeSet<u64> = rows.difference(&in_the_ring).copied().collect();
            rows.retain(|row| !deletable.contains(row));
            txg += 1;
            owner.insert(txg, instance);
            sweep = Some((
                u64::try_from(deletable.len()).expect("行数"),
                u64::try_from(rows.len()).expect("行数"),
            ));
            sweeps_done += 1;
        }
        mounts.push(AnchorModelMount {
            rows0,
            sweep,
            checkpoint_txg_when_the_mount_returns: txg,
        });
        if sweeps_done == SWEEPS_TO_FINISH_A_HISTORY {
            break;
        }
        match unmount_kind {
            UnmountKind::Normal => {
                push_empty_publishes_of_an_unmount(&mut txg, &mut owner, instance)
            }
            UnmountKind::Crash => {}
        }
    }
    AnchorModelHistory {
        mounts,
        final_txg: txg,
    }
}

/// 登记第十三节命令一 H5 那 24 行（原样的数）：S、θ、卸载方式、挂了几次、两次清扫各删几行、各剩几行、在第几次挂载、rows0 峰值、片数峰值、末了的 txg。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RegisteredSweepHistoryAnchor {
    pub(super) slots_per_region: u64,
    pub(super) threshold_rows: u64,
    pub(super) unmount_kind: UnmountKind,
    pub(super) mounts: u64,
    rows_deleted: [u64; 2],
    rows_left: [u64; 2],
    mounts_at_sweep: [u64; 2],
    peak_rows0: u64,
    peak_pages: u64,
    final_txg: u64,
}

impl RegisteredSweepHistoryAnchor {
    pub(super) fn is_reproduced_by(&self, history: &AnchorModelHistory) -> bool {
        let sweeps = history.sweeps();
        sweeps.len() == 2
            && u64::try_from(history.mounts.len()).ok() == Some(self.mounts)
            && sweeps
                .iter()
                .map(|(_, deleted, _)| *deleted)
                .collect::<Vec<_>>()
                == self.rows_deleted
            && sweeps.iter().map(|(_, _, left)| *left).collect::<Vec<_>>() == self.rows_left
            && sweeps
                .iter()
                .map(|(mount, _, _)| *mount)
                .collect::<Vec<_>>()
                == self.mounts_at_sweep
            && history.peak_rows0() == self.peak_rows0
            && instance_table_chain_pages(history.peak_rows0()) == self.peak_pages
            && history.final_txg == self.final_txg
    }
}

/// 登记第十三节命令一的原样输出（`anchors_e183.py` H5 那 24 行）；`sweep_history_by_the_anchor_model` 必须逐行重现（停机 T3 的同类核）。
const REGISTERED_SWEEP_HISTORY_ANCHORS: [RegisteredSweepHistoryAnchor; 24] = [
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Normal,
        mounts: 128,
        rows_deleted: [63, 63],
        rows_left: [2, 2],
        mounts_at_sweep: [65, 128],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 770,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Crash,
        mounts: 127,
        rows_deleted: [62, 62],
        rows_left: [3, 3],
        mounts_at_sweep: [65, 127],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 386,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Normal,
        mounts: 730,
        rows_deleted: [364, 364],
        rows_left: [2, 2],
        mounts_at_sweep: [366, 730],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 4382,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Crash,
        mounts: 729,
        rows_deleted: [363, 363],
        rows_left: [3, 3],
        mounts_at_sweep: [366, 729],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 2192,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Normal,
        mounts: 738,
        rows_deleted: [368, 368],
        rows_left: [2, 2],
        mounts_at_sweep: [370, 738],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 4430,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Crash,
        mounts: 737,
        rows_deleted: [367, 367],
        rows_left: [3, 3],
        mounts_at_sweep: [370, 737],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 2216,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Normal,
        mounts: 1476,
        rows_deleted: [737, 737],
        rows_left: [2, 2],
        mounts_at_sweep: [739, 1476],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 8858,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Crash,
        mounts: 1475,
        rows_deleted: [736, 736],
        rows_left: [3, 3],
        mounts_at_sweep: [739, 1475],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 4430,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Normal,
        mounts: 126,
        rows_deleted: [61, 61],
        rows_left: [4, 4],
        mounts_at_sweep: [65, 126],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 758,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Crash,
        mounts: 123,
        rows_deleted: [58, 58],
        rows_left: [7, 7],
        mounts_at_sweep: [65, 123],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 374,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Normal,
        mounts: 728,
        rows_deleted: [362, 362],
        rows_left: [4, 4],
        mounts_at_sweep: [366, 728],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 4370,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Crash,
        mounts: 725,
        rows_deleted: [359, 359],
        rows_left: [7, 7],
        mounts_at_sweep: [366, 725],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 2180,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Normal,
        mounts: 736,
        rows_deleted: [366, 366],
        rows_left: [4, 4],
        mounts_at_sweep: [370, 736],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 4418,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Crash,
        mounts: 733,
        rows_deleted: [363, 363],
        rows_left: [7, 7],
        mounts_at_sweep: [370, 733],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 2204,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Normal,
        mounts: 1474,
        rows_deleted: [735, 735],
        rows_left: [4, 4],
        mounts_at_sweep: [739, 1474],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 8846,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Crash,
        mounts: 1471,
        rows_deleted: [732, 732],
        rows_left: [7, 7],
        mounts_at_sweep: [739, 1471],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 4418,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Normal,
        mounts: 122,
        rows_deleted: [57, 57],
        rows_left: [8, 8],
        mounts_at_sweep: [65, 122],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 734,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Crash,
        mounts: 115,
        rows_deleted: [50, 50],
        rows_left: [15, 15],
        mounts_at_sweep: [65, 115],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 350,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Normal,
        mounts: 724,
        rows_deleted: [358, 358],
        rows_left: [8, 8],
        mounts_at_sweep: [366, 724],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 4346,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Crash,
        mounts: 717,
        rows_deleted: [351, 351],
        rows_left: [15, 15],
        mounts_at_sweep: [366, 717],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 2156,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Normal,
        mounts: 732,
        rows_deleted: [362, 362],
        rows_left: [8, 8],
        mounts_at_sweep: [370, 732],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 4394,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Crash,
        mounts: 725,
        rows_deleted: [355, 355],
        rows_left: [15, 15],
        mounts_at_sweep: [370, 725],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 2180,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Normal,
        mounts: 1470,
        rows_deleted: [731, 731],
        rows_left: [8, 8],
        mounts_at_sweep: [739, 1470],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 8822,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Crash,
        mounts: 1463,
        rows_deleted: [724, 724],
        rows_left: [15, 15],
        mounts_at_sweep: [739, 1463],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 4394,
    },
];

/// 两块盘计数里全部区间的写字节与读字节之和（X21 的「设备写字节」「读字节」）。
fn written_and_read_bytes(tallies: &[DeviceTally]) -> (u64, u64) {
    tallies.iter().fold((0, 0), |(written, read), tally| {
        let regions = [
            tally.system_configuration,
            tally.root_ring,
            tally.journal_ring,
            tally.unit_area,
        ];
        (
            written
                + regions
                    .iter()
                    .map(|region| region.writes.bytes)
                    .sum::<u64>(),
            read + regions.iter().map(|region| region.reads.bytes).sum::<u64>(),
        )
    })
}

/// 同一份盘面的一份拷贝（X21：两次挂载各从同一份镜像副本起）：盘面照抄，计数从 0 起。
fn copy_of_the_pool(pool: &CountedPool, device_bytes: u64) -> CountedPool {
    pool.iter()
        .map(|(identity, device)| {
            let mut copied = SparseBlockDevice::new(
                device_bytes,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            );
            copied.image = device.inner.image.clone();
            (*identity, CountingBlockDevice::new(copied, device.regions))
        })
        .collect()
}

/// 根环里读得出的每条根：(区域, 槽, txg, 实例)，按根环槽的次序。
fn ring_roots_of(
    pool: &CountedPool,
    parameters: &MakeFilesystemParameters,
) -> Vec<(u64, u64, u64, u32)> {
    readable_roots_with_ring_slots(
        pool,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .into_iter()
    .map(|(ring_slot, root)| {
        (
            ring_slot.region,
            ring_slot.slot,
            root.checkpoint_txg.0,
            root.instance.0,
        )
    })
    .collect()
}

/// 每次读根环，把读到的 txg → 实例记下来（K34 要「清扫判的那一刻根环里有哪几个实例」：那一刻的环 = 清扫发布之前最近 3S 个 txg 的根）。
fn note_ring_owners(owner_by_txg: &mut BTreeMap<u64, u32>, roots: &[(u64, u64, u64, u32)]) {
    for (_, _, txg, instance) in roots {
        owner_by_txg.insert(*txg, *instance);
    }
}

/// 清扫判的那一刻根环里有根的实例：清扫发布的 txg 是 `sweep_txg`，环里是它之前的 3S 个 txg；有一个 txg 没读到过交回 `None`（K34 判不了）。
fn instances_in_the_ring_when_the_sweep_judges(
    owner_by_txg: &BTreeMap<u64, u32>,
    sweep_txg: u64,
    ring_roots: u64,
) -> Option<BTreeSet<u32>> {
    (sweep_txg.saturating_sub(ring_roots)..sweep_txg)
        .map(|txg| owner_by_txg.get(&txg).copied())
        .collect()
}

/// K34 的装置侧判法：该删的行 = 清扫之前的行里，实例在那一刻根环里一条根都没有的那些。
fn rows_the_sweep_should_delete(
    rows_before: &[u32],
    instances_in_the_ring: &BTreeSet<u32>,
) -> BTreeSet<u32> {
    rows_before
        .iter()
        .filter(|instance| !instances_in_the_ring.contains(instance))
        .copied()
        .collect()
}

/// 一版的实例表里有哪几个实例的行；读不出交回 `None`。
fn instance_rows_of(
    pool: &CountedPool,
    root: &singlefs_core::root_record::RootRecord,
) -> Option<Vec<u32>> {
    instance_table_chain_of_root(pool, root).ok().map(|chain| {
        chain
            .records
            .rows
            .iter()
            .map(|row| row.instance.0)
            .collect()
    })
}

/// 挂载末尾那一判清扫的结局，按装置要判的几样拆开。
#[derive(Clone, Debug, PartialEq, Eq)]
struct SweepAtTheMount {
    pub(super) text: String,
    /// 清扫发布发成了（删了行）：K33 的「发了清扫发布」。
    rows_deleted: Option<Vec<u32>>,
    heads_erased: usize,
    handed_to_the_background: bool,
}

fn sweep_at_the_mount(outcome: &InstanceTableSweepAtTheEndOfTheMount) -> SweepAtTheMount {
    let not_published = |text: String, heads_erased: usize| SweepAtTheMount {
        text,
        rows_deleted: None,
        heads_erased,
        handed_to_the_background: false,
    };
    match outcome {
        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. } => {
            not_published("not_swept_at_or_below_the_threshold".to_string(), 0)
        }
        InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted => {
            SweepAtTheMount {
                text: "handed_to_the_background".to_string(),
                rows_deleted: None,
                heads_erased: 0,
                handed_to_the_background: true,
            }
        }
        InstanceTableSweepAtTheEndOfTheMount::Swept(swept) => match swept {
            InstanceTableSweepOutcome::RowsDeleted {
                rows_deleted,
                heads_erased,
            } => SweepAtTheMount {
                text: "rows_deleted".to_string(),
                rows_deleted: Some(rows_deleted.iter().map(|row| row.instance.0).collect()),
                heads_erased: heads_erased.len(),
                handed_to_the_background: false,
            },
            InstanceTableSweepOutcome::NoRowDeleted(reason) => not_published(
                format!("no_row_deleted:{}", variant_name_of(&format!("{reason:?}"))),
                0,
            ),
            InstanceTableSweepOutcome::SweepPublishRefusedForSpace { heads_erased, .. } => {
                not_published(
                    "sweep_publish_refused_for_space".to_string(),
                    heads_erased.len(),
                )
            }
            InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
                heads_erased,
                ..
            } => not_published(
                "stopped_by_a_block_device_error_while_erasing".to_string(),
                heads_erased.len(),
            ),
            InstanceTableSweepOutcome::SweepPublishFailed { heads_erased, .. } => {
                not_published("sweep_publish_failed".to_string(), heads_erased.len())
            }
        },
    }
}

/// 一格历史的公共前缀：mkfs → 可写挂载 → 首个文件 →（记下 W_θ 的起点计数）→ 正常卸载（登记 X16）。交回 W_θ 起点那一刻的计数。
fn sweep_history_prefix(
    parameters: &MakeFilesystemParameters,
    pool: &mut CountedPool,
    configuration: InstanceTableSweepConfiguration,
) -> Result<Vec<DeviceTally>, String> {
    guarded(|| {
        make_filesystem(parameters, pool)
            .map(|_| ())
            .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("make_filesystem"))?;
    let mounted = guarded(|| {
        mount_writable_with_instance_table_sweep_configuration(
            parameters,
            pool,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            configuration,
        )
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("first_mount"))?;
    let Mounted {
        output,
        mut allocator,
        current,
    } = mounted;
    let first_file = first_file_content();
    let first = guarded(|| {
        let mut writer = PoolWriter::new(parameters, pool.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &first_file,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            output.instance,
            current.record_bytes(),
        )
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("first_file"))?;
    let mut version_with_the_first_file = PoolVersion::of_a_version_of_trees(first);
    let tallies_after_the_first_file = tallies_of(pool);
    guarded(|| {
        unmount(
            parameters,
            pool,
            &mut allocator,
            &mut version_with_the_first_file,
            output.shadow_ledger,
        )
        .map(|_| ())
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("prefix_unmount"))?;
    Ok(tallies_after_the_first_file)
}

/// 一次挂载的读数（R5a 那一行、K33、K34、A36、与锚点模型逐次比）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HistoryMount {
    pub(super) number: u64,
    pub(super) instance: u32,
    rows_written: u64,
    pub(super) rows0: u64,
    rows0_by_the_allocator: u64,
    rows_before_the_sweep: Vec<u32>,
    sweep: SweepAtTheMount,
    rows_left: Option<u64>,
    pub(super) checkpoint_txg: u64,
    mount_nanoseconds: u64,
    written_and_read_bytes: (u64, u64),
}

/// 这次挂载对得上的几样：`None` 是这一格不适用（没有清扫、对照臂）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HistoryMountChecks {
    sweep_matches_the_threshold: bool,
    deleted_rows_match_the_ring: Option<bool>,
    no_head_erased: Option<bool>,
    matches_the_model: bool,
}

impl HistoryMountChecks {
    pub(super) fn all_hold(&self) -> bool {
        self.sweep_matches_the_threshold
            && self.deleted_rows_match_the_ring != Some(false)
            && self.no_head_erased != Some(false)
            && self.matches_the_model
    }

    fn failed_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if !self.sweep_matches_the_threshold {
            names.push("sweep_matches_the_threshold");
        }
        if self.deleted_rows_match_the_ring == Some(false) {
            names.push("deleted_rows_match_the_ring");
        }
        if self.no_head_erased == Some(false) {
            names.push("no_head_erased");
        }
        if !self.matches_the_model {
            names.push("a34_a35_trajectory");
        }
        names
    }
}

fn checks_of_a_history_mount(
    mount: &HistoryMount,
    arm: SweepArm,
    model: Option<&AnchorModelMount>,
    instances_in_the_ring: Option<&BTreeSet<u32>>,
) -> HistoryMountChecks {
    let sweep_matches_the_threshold = match arm.threshold_rows() {
        Some(threshold) => {
            mount.sweep.rows_deleted.is_some() == sweep_is_due(mount.rows0, threshold)
        }
        None => mount.sweep.handed_to_the_background,
    };
    let deleted_rows_match_the_ring = mount.sweep.rows_deleted.as_ref().map(|deleted| {
        instances_in_the_ring.is_some_and(|in_the_ring| {
            rows_the_sweep_should_delete(&mount.rows_before_the_sweep, in_the_ring)
                == deleted.iter().copied().collect::<BTreeSet<u32>>()
        })
    });
    let no_head_erased = mount
        .sweep
        .rows_deleted
        .as_ref()
        .map(|_| mount.sweep.heads_erased == 0);
    let actual_sweep = mount.sweep.rows_deleted.as_ref().map(|deleted| {
        (
            u64::try_from(deleted.len()).expect("行数"),
            mount.rows_left.unwrap_or(u64::MAX),
        )
    });
    let matches_the_model = model
        .is_some_and(|expected| expected.rows0 == mount.rows0 && expected.sweep == actual_sweep);
    HistoryMountChecks {
        sweep_matches_the_threshold,
        deleted_rows_match_the_ring,
        no_head_erased,
        matches_the_model,
    }
}

/// 一次可写挂载（清扫配置照这一格的臂）连同它的读数；挂载没成交回 `Err`。
fn mount_once_in_the_history(
    parameters: &MakeFilesystemParameters,
    pool: &mut CountedPool,
    configuration: InstanceTableSweepConfiguration,
    number: u64,
) -> Result<(Mounted, HistoryMount), String> {
    let tallies_before = tallies_of(pool);
    let started = Instant::now();
    let mounted = guarded(|| {
        mount_writable_with_instance_table_sweep_configuration(
            parameters,
            pool,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            configuration,
        )
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })?;
    let mount_nanoseconds = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let written_and_read = written_and_read_bytes(&tally_difference(pool, &tallies_before));
    let rows_before_the_sweep = instance_rows_of(pool, mounted.output.row_publish.root())
        .ok_or("row_publish_table_unreadable")?;
    let sweep = sweep_at_the_mount(&mounted.output.instance_table_sweep);
    let rows_left = sweep
        .rows_deleted
        .as_ref()
        .and_then(|_| instance_rows_of(pool, mounted.current.root()))
        .map(|rows| u64::try_from(rows.len()).expect("行数"));
    let mount = HistoryMount {
        number,
        instance: mounted.output.instance.0,
        rows_written: u64::try_from(mounted.output.rows_written.len()).expect("行数"),
        rows0: u64::try_from(rows_before_the_sweep.len()).expect("行数"),
        rows0_by_the_allocator: mounted
            .allocator
            .instance_rows_after_this_mounts_row_publish(),
        rows_before_the_sweep,
        sweep,
        rows_left,
        checkpoint_txg: mounted.current.root().checkpoint_txg.0,
        mount_nanoseconds,
        written_and_read_bytes: written_and_read,
    };
    Ok((mounted, mount))
}

fn optional_truth_text(value: Option<bool>) -> String {
    value.map_or_else(|| "not_applicable".to_string(), |truth| truth.to_string())
}

fn emit_history_mount(
    emitter: &mut Emitter,
    shard: &str,
    mount: &HistoryMount,
    model: Option<&AnchorModelMount>,
    checks: HistoryMountChecks,
) {
    emitter.emit(&format!(
        "name=h5_mount shard={shard} mount={} instance={} rows_at_mount={} rows0={} rows0_by_the_allocator={} model_rows0={} chain_pages={} sweep={} rows_deleted={} rows_left={} heads_erased={} checkpoint_txg={} model_checkpoint_txg={} sweep_matches_the_threshold={} deleted_rows_match_the_ring={} no_head_erased={} trajectory_matches_the_model={}",
        mount.number,
        mount.instance,
        mount.rows0.saturating_sub(mount.rows_written),
        mount.rows0,
        mount.rows0_by_the_allocator,
        optional_number_text(model.map(|expected| expected.rows0)),
        instance_table_chain_pages(mount.rows0),
        mount.sweep.text,
        mount.sweep.rows_deleted.as_ref().map_or_else(|| "none".to_string(), |deleted| deleted.len().to_string()),
        optional_number_text(mount.rows_left),
        mount.sweep.heads_erased,
        mount.checkpoint_txg,
        optional_number_text(model.map(|expected| expected.checkpoint_txg_when_the_mount_returns)),
        checks.sweep_matches_the_threshold,
        optional_truth_text(checks.deleted_rows_match_the_ring),
        optional_truth_text(checks.no_head_erased),
        checks.matches_the_model,
    ));
}

/// T5 停机：把那一刻根环每槽的 (txg, 实例)、实例表的行、清扫结局原样写出（登记第十一节 T5「处置」）。
fn emit_history_stop(
    emitter: &mut Emitter,
    shard: &str,
    mount: &HistoryMount,
    reasons: &[&str],
    ring: &[(u64, u64, u64, u32)],
) -> ShardEnd {
    for (region, slot, txg, instance) in ring {
        emitter.emit(&format!(
            "name=h5_ring_slot shard={shard} mount={} region={region} slot={slot} checkpoint_txg={txg} instance={instance}",
            mount.number
        ));
    }
    emitter.emit(&format!(
        "name=h5_rows_before_the_sweep shard={shard} mount={} instances={}",
        mount.number,
        joined_or_none(&mount.rows_before_the_sweep)
    ));
    ShardEnd::Stopped {
        reason: format!("t5_at_mount_{}:{}", mount.number, reasons.join(",")),
    }
}

/// X21 ①：同一份挂载前镜像上，只差清扫开关的两次挂载（阈值 θ 那一次 − 「交给后台」那一次）的写字节、读字节之差。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SweepDifferential {
    write_bytes_difference: i128,
    read_bytes_difference: i128,
}

/// X21 ① 的差：阈值那一次的（写字节，读字节）减「交给后台」那一次的。
fn sweep_differential(
    with_the_threshold: (u64, u64),
    handed_to_the_background: (u64, u64),
) -> SweepDifferential {
    SweepDifferential {
        write_bytes_difference: i128::from(with_the_threshold.0)
            - i128::from(handed_to_the_background.0),
        read_bytes_difference: i128::from(with_the_threshold.1)
            - i128::from(handed_to_the_background.1),
    }
}

/// 「交给后台」那一次挂载在拷贝上做；做不成交回 `None`（那一次的差分不出，V35 按格判）。
fn background_mount_on_a_copy(
    parameters: &MakeFilesystemParameters,
    pool: &CountedPool,
    device_bytes: u64,
) -> Option<(u64, u64)> {
    let mut copy = copy_of_the_pool(pool, device_bytes);
    let configuration = SweepArm::HandedToTheBackground.configuration();
    let mounted = mount_once_in_the_history(parameters, &mut copy, configuration, 0).ok()?;
    Some(mounted.1.written_and_read_bytes)
}

/// PC-差分（登记 5.3）：第一次没越过阈值的那一次挂载两样差都是 0；第一次越过的那一次写字节之差 > 0。
/// 读字节那一半按登记字面判、不放宽：阈值那一次挂载末尾要读实例表链数行数（`sweep_the_instance_table_at_the_end_of_the_mount`），「交给后台」那一次不读，
/// 推的读字节之差不为 0，那一格就是 V35 打中、记一次输（R5c、R5d 不出，登记第十二节修订）；汇总行另把写、读两样差分开报，
/// 读字节之差另有新臂 `chain_read_differential_holds` 各自判。
fn differential_control_below_holds(differential: Option<SweepDifferential>) -> bool {
    differential.is_some_and(|difference| {
        difference.write_bytes_difference == 0 && difference.read_bytes_difference == 0
    })
}

fn differential_control_above_holds(differential: Option<SweepDifferential>) -> bool {
    differential.is_some_and(|difference| difference.write_bytes_difference > 0)
}

/// 沿实例表链读一遍要读几片：`max(1, ⌈行数 ÷ 369⌉)`（一片 369 行）。与 `instance_table_chain_pages` 不是一个量：那个是切换预留的式子（行数加 N_switch），
/// 这里是阈值那一次挂载末尾数行数时真读的片数，行数就是链上此刻的行数。
fn instance_table_chain_pages_read(rows: u64) -> u64 {
    rows.div_ceil(INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE).max(1)
}

/// 阈值那一次挂载在末尾读实例表链的字节数，另算、不取装置的读数：每片读一次 `INSTANCE_TABLE_PAGE_BYTES`
/// （`recovery::read_unit_via_locations` 读第一条位置条目、校验和对就收，不读第二份），片数照 `instance_table_chain_pages_read`。
fn instance_table_chain_read_bytes(rows: u64) -> u64 {
    instance_table_chain_pages_read(rows) * INSTANCE_TABLE_PAGE_BYTES
}

/// PC-差分的新臂（登记第十二节修订加的，原臂 `differential_control_below_holds` 照字面另判、两臂不合成一个真假）：没越过阈值的那一次，
/// 读字节之差等于阈值那一次读实例表链的字节数；差没做成、期望没算出都不算过。
fn chain_read_differential_holds(
    differential: Option<SweepDifferential>,
    expected_chain_read_bytes: Option<u64>,
) -> bool {
    match (differential, expected_chain_read_bytes) {
        (Some(difference), Some(expected)) => {
            difference.read_bytes_difference == i128::from(expected)
        }
        (None, _) | (_, None) => false,
    }
}

/// 新臂的结果行：每格一行，在没越过阈值的那一次挂载上；期望字节取锚点模型这一次的 rows0，不取装置读到的。
fn chain_read_differential_line(
    shard: &str,
    mount: u64,
    differential: Option<SweepDifferential>,
    model_rows0: Option<u64>,
) -> String {
    let expected = model_rows0.map(instance_table_chain_read_bytes);
    format!(
        "name=h5_chain_read_differential shard={shard} mount={mount} read_bytes_difference={} model_rows0={} expected_chain_pages={} expected_chain_read_bytes={} read_bytes_difference_equals_the_chain_read={}",
        differential.map_or_else(|| "none".to_string(), |difference| difference.read_bytes_difference.to_string()),
        optional_number_text(model_rows0),
        optional_number_text(model_rows0.map(instance_table_chain_pages_read)),
        optional_number_text(expected),
        chain_read_differential_holds(differential, expected),
    )
}

/// R5a、R5b 甲、R5c、PC-不扫 要的整段历史的汇总。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct HistoryTally {
    pub(super) mounts: u64,
    sweeps_done: usize,
    peak_rows0: u64,
    mounts_with_rows0_above_the_threshold: u64,
    mounts_with_rows0_above_366: u64,
    pub(super) peak_chain_pages: u64,
    mounts_with_two_or_more_chain_pages: u64,
    final_rows: u64,
    last_rows0: u64,
    /// PC-不扫：第一次挂载 rows0 = 1，之后每次挂载比上一次多 1。
    rows0_plus_one_each_mount: bool,
    /// 每次清扫的写、读字节之差；那一次的差分没做成是 `None`（读不到 ≠ 读到 0）。
    sweep_write_bytes: Vec<Option<i128>>,
    sweep_read_bytes: Vec<Option<i128>>,
    sweep_mount_nanoseconds: Vec<u64>,
    differential_below_the_threshold: Option<SweepDifferential>,
    differential_at_the_first_sweep: Option<SweepDifferential>,
}

impl HistoryTally {
    pub(super) fn note(&mut self, mount: &HistoryMount, arm: SweepArm) {
        if self.mounts == 0 {
            self.rows0_plus_one_each_mount = mount.rows0 == 1;
        } else if mount.rows0 != self.last_rows0 + 1 {
            self.rows0_plus_one_each_mount = false;
        }
        self.mounts += 1;
        self.peak_rows0 = self.peak_rows0.max(mount.rows0);
        self.mounts_with_rows0_above_the_threshold += u64::from(
            arm.threshold_rows()
                .is_some_and(|threshold| mount.rows0 > threshold),
        );
        self.mounts_with_rows0_above_366 += u64::from(mount.rows0 > ROWS_KEPT_ON_ONE_CHAIN_PAGE);
        let pages = instance_table_chain_pages(mount.rows0);
        self.peak_chain_pages = self.peak_chain_pages.max(pages);
        self.mounts_with_two_or_more_chain_pages += u64::from(pages >= 2);
        self.final_rows = mount.rows_left.unwrap_or(mount.rows0);
        self.last_rows0 = mount.rows0;
        self.sweeps_done += usize::from(mount.sweep.rows_deleted.is_some());
    }
}

/// 一片 H5：锚点模型与登记 24 行比（不符 ⇒ 停机 T3）、公共前缀、逐次挂载（每次读根环、比模型、判 K33/K34/A36；第一次挂载与模型预计清扫的那几次
/// 另在拷贝上做「交给后台」那一次挂载取差分）、正常卸载或崩溃、汇总。任一次挂载对不上 ⇒ 停机 T5（片文件不写，那一刻的根环与行原样打出）。
pub(super) fn run_sweep_history_shard(
    shard: &str,
    cell: SweepHistoryCell,
    binary_sha256: &str,
    emitter: &mut Emitter,
) -> ShardEnd {
    run_sweep_history_on_a_pool(
        shard,
        cell,
        PoolGeometry {
            device_bytes: DEFAULT_DEVICE_BYTES,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        },
        binary_sha256,
        emitter,
    )
}

/// 盘的几何：产物取 4 GiB、768 MiB 环（登记 X16）；单测在小盘上走同一条路。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PoolGeometry {
    pub(super) device_bytes: u64,
    pub(super) journal_ring_bytes: u64,
}

fn run_sweep_history_on_a_pool(
    shard: &str,
    cell: SweepHistoryCell,
    geometry: PoolGeometry,
    binary_sha256: &str,
    emitter: &mut Emitter,
) -> ShardEnd {
    emitter.emit(&format!(
        "name=h5_config shard={shard} slots_per_region={} threshold_rows={} unmount_kind={} mounts_at_most={} device_bytes={} journal_ring_bytes={} binary_sha256={binary_sha256}",
        cell.slots_per_region,
        optional_number_text(cell.arm.threshold_rows()),
        cell.unmount_kind.name(),
        cell.mounts_at_most,
        geometry.device_bytes,
        geometry.journal_ring_bytes,
    ));
    let model = sweep_history_by_the_anchor_model(
        cell.slots_per_region,
        cell.arm.threshold_rows(),
        cell.unmount_kind,
        cell.mounts_at_most,
    );
    if let Some(threshold) = cell.arm.threshold_rows() {
        let registered = REGISTERED_SWEEP_HISTORY_ANCHORS.iter().find(|anchor| {
            anchor.slots_per_region == cell.slots_per_region
                && anchor.threshold_rows == threshold
                && anchor.unmount_kind == cell.unmount_kind
        });
        let reproduced = registered.map(|anchor| anchor.is_reproduced_by(&model));
        emitter.emit(&format!(
            "name=h5_anchor_model shard={shard} model_mounts={} model_sweeps_mount_deleted_left={} model_final_txg={} matches_the_registered_script_output={}",
            model.mounts.len(),
            joined_or_none(
                &model
                    .sweeps()
                    .iter()
                    .map(|(mount, deleted, left)| format!("{mount}:{deleted}:{left}"))
                    .collect::<Vec<_>>()
            ),
            model.final_txg,
            match reproduced {
                Some(true) => "true",
                Some(false) => "false",
                None => "not_registered",
            },
        ));
        if reproduced == Some(false) {
            return ShardEnd::Stopped {
                reason: "t3_anchor_model_differs_from_the_registered_rows".to_string(),
            };
        }
    }
    let parameters = make_filesystem_parameters(geometry.journal_ring_bytes, cell.slots_per_region);
    let mut pool = counted_pool(
        geometry.device_bytes,
        geometry.journal_ring_bytes,
        cell.slots_per_region,
    );
    let configuration = cell.arm.configuration();
    let tallies_after_the_first_file =
        match sweep_history_prefix(&parameters, &mut pool, configuration) {
            Ok(tallies) => tallies,
            Err(failed_step) => {
                return ShardEnd::Stopped {
                    reason: format!("history_prefix_failed:{failed_step}"),
                }
            }
        };
    let mut owner_by_txg: BTreeMap<u64, u32> = BTreeMap::new();
    note_ring_owners(&mut owner_by_txg, &ring_roots_of(&pool, &parameters));
    let model_sweep_mounts: BTreeSet<u64> =
        model.sweeps().iter().map(|(mount, _, _)| *mount).collect();
    let mut tally = HistoryTally::default();
    for number in 1..=cell.mounts_at_most {
        let take_a_differential = cell.arm.threshold_rows().is_some()
            && (number == 1 || model_sweep_mounts.contains(&number));
        let background = if take_a_differential {
            background_mount_on_a_copy(&parameters, &pool, geometry.device_bytes)
        } else {
            None
        };
        let (mut mounted, mount) =
            match mount_once_in_the_history(&parameters, &mut pool, configuration, number) {
                Ok(pair) => pair,
                Err(reason) => {
                    return ShardEnd::Stopped {
                        reason: format!("mount_{number}_failed:{reason}"),
                    }
                }
            };
        let ring = ring_roots_of(&pool, &parameters);
        note_ring_owners(&mut owner_by_txg, &ring);
        let in_the_ring = instances_in_the_ring_when_the_sweep_judges(
            &owner_by_txg,
            mount.checkpoint_txg,
            ROOT_RING_REGIONS * cell.slots_per_region,
        );
        let model_mount = model
            .mounts
            .get(usize::try_from(number - 1).expect("挂载序号"));
        let checks = checks_of_a_history_mount(&mount, cell.arm, model_mount, in_the_ring.as_ref());
        emit_history_mount(emitter, shard, &mount, model_mount, checks);
        if take_a_differential {
            let differential = background.map(|background_bytes| {
                sweep_differential(mount.written_and_read_bytes, background_bytes)
            });
            emitter.emit(&format!(
                "name=h5_differential shard={shard} mount={number} swept={} write_bytes_difference={} read_bytes_difference={} mount_nanoseconds={}",
                mount.sweep.rows_deleted.is_some(),
                differential.map_or_else(|| "none".to_string(), |difference| difference.write_bytes_difference.to_string()),
                differential.map_or_else(|| "none".to_string(), |difference| difference.read_bytes_difference.to_string()),
                mount.mount_nanoseconds,
            ));
            if number == 1 {
                tally.differential_below_the_threshold = differential;
                emitter.emit(&chain_read_differential_line(
                    shard,
                    number,
                    differential,
                    model_mount.map(|expected| expected.rows0),
                ));
            }
            if mount.sweep.rows_deleted.is_some() {
                if tally.sweeps_done == 0 {
                    tally.differential_at_the_first_sweep = differential;
                }
                tally
                    .sweep_write_bytes
                    .push(differential.map(|difference| difference.write_bytes_difference));
                tally
                    .sweep_read_bytes
                    .push(differential.map(|difference| difference.read_bytes_difference));
                tally.sweep_mount_nanoseconds.push(mount.mount_nanoseconds);
            }
        }
        if !checks.all_hold() {
            return emit_history_stop(emitter, shard, &mount, &checks.failed_names(), &ring);
        }
        tally.note(&mount, cell.arm);
        if cell.arm.threshold_rows().is_some() && tally.sweeps_done == SWEEPS_TO_FINISH_A_HISTORY {
            break;
        }
        match cell.unmount_kind {
            UnmountKind::Normal => {
                let unmounted = guarded(|| {
                    unmount(
                        &parameters,
                        &mut pool,
                        &mut mounted.allocator,
                        &mut mounted.current,
                        mounted.output.shadow_ledger,
                    )
                    .map(|_| ())
                    .map_err(|error| variant_name_of(&format!("{error:?}")))
                });
                if let Err(reason) = unmounted {
                    return ShardEnd::Stopped {
                        reason: format!("unmount_after_mount_{number}_failed:{reason}"),
                    };
                }
                note_ring_owners(&mut owner_by_txg, &ring_roots_of(&pool, &parameters));
            }
            UnmountKind::Crash => drop(mounted),
        }
    }
    let total_written =
        written_and_read_bytes(&tally_difference(&pool, &tallies_after_the_first_file)).0;
    emit_history_summary(emitter, shard, cell, &tally, total_written);
    ShardEnd::Finished
}

/// 一列可能缺的数：缺的写 `none`，整列为空写 `none`。
fn joined_optional(values: &[Option<i128>]) -> String {
    joined_or_none(
        &values
            .iter()
            .map(|value| value.map_or_else(|| "none".to_string(), |number| number.to_string()))
            .collect::<Vec<_>>(),
    )
}

/// PC-不扫（登记 5.3）跑前写的是「400 次挂载之后 rows0 到 401」，与登记自己的锚点读法（第 m 次挂载 rows0 = m）对不上；
/// 登记第十二节修订把判据改成锚点模型给的 400（依据：单测 `anchor_model_without_a_threshold_never_sweeps_and_rows0_counts_the_mounts` 的读数，时点在产物之前）。
/// 跑前写的 401 只在产物里作为 `original_registered_final_rows0` 照抄一份，不再判。
const SWEEP_CONTROL_REVISED_FINAL_ROWS0: u64 = 400;
const SWEEP_CONTROL_ORIGINALLY_REGISTERED_FINAL_ROWS0: u64 = 401;

/// 「交给后台」对照的 rows0 期末值是不是修订后的登记值。
fn control_final_rows0_equals_the_revised_registered_value(last_rows0: u64) -> bool {
    last_rows0 == SWEEP_CONTROL_REVISED_FINAL_ROWS0
}

/// 一格历史的汇总行（R5a 的峰值、> θ 与 > 366 的次数、期末值；R5b 甲 的片数峰值与 ≥ 2 片的次数；X21 的写字节与挂载次数；PC-差分；对照格的 PC-不扫）。
fn emit_history_summary(
    emitter: &mut Emitter,
    shard: &str,
    cell: SweepHistoryCell,
    tally: &HistoryTally,
    total_write_bytes: u64,
) {
    let mut fields = vec![
        format!("name=h5_cell_summary shard={shard}"),
        format!("slots_per_region={}", cell.slots_per_region),
        format!(
            "threshold_rows={}",
            optional_number_text(cell.arm.threshold_rows())
        ),
        format!("unmount_kind={}", cell.unmount_kind.name()),
        format!("writable_mounts={}", tally.mounts),
        format!("sweeps_done={}", tally.sweeps_done),
        format!("peak_rows0={}", tally.peak_rows0),
        format!(
            "mounts_with_rows0_above_the_threshold={}",
            tally.mounts_with_rows0_above_the_threshold
        ),
        format!(
            "mounts_with_rows0_above_366={}",
            tally.mounts_with_rows0_above_366
        ),
        format!("peak_chain_pages={}", tally.peak_chain_pages),
        format!(
            "mounts_with_two_or_more_chain_pages={}",
            tally.mounts_with_two_or_more_chain_pages
        ),
        format!("final_rows={}", tally.final_rows),
        format!("total_write_bytes={total_write_bytes}"),
        format!(
            "sweep_write_bytes={}",
            joined_optional(&tally.sweep_write_bytes)
        ),
        format!(
            "sweep_read_bytes={}",
            joined_optional(&tally.sweep_read_bytes)
        ),
        format!(
            "sweep_mount_nanoseconds={}",
            joined_or_none(&tally.sweep_mount_nanoseconds)
        ),
    ];
    match cell.arm {
        SweepArm::AtTheEndOfTheMountAboveTheThreshold { .. } => {
            fields.push(format!(
                "positive_control_differential_below_the_threshold={} positive_control_differential_at_the_first_sweep={} differential_below_the_threshold_write_bytes={} differential_below_the_threshold_read_bytes={}",
                differential_control_below_holds(tally.differential_below_the_threshold),
                differential_control_above_holds(tally.differential_at_the_first_sweep),
                joined_optional(&[tally
                    .differential_below_the_threshold
                    .map(|difference| difference.write_bytes_difference)]),
                joined_optional(&[tally
                    .differential_below_the_threshold
                    .map(|difference| difference.read_bytes_difference)]),
            ));
        }
        SweepArm::HandedToTheBackground => {
            fields.push(format!(
                "positive_control_no_sweep_publish={} positive_control_rows0_plus_one_each_mount={} final_rows0={} registered_final_rows0={SWEEP_CONTROL_REVISED_FINAL_ROWS0} original_registered_final_rows0={SWEEP_CONTROL_ORIGINALLY_REGISTERED_FINAL_ROWS0} final_rows0_equals_the_registered_value={}",
                tally.sweeps_done == 0,
                tally.rows0_plus_one_each_mount,
                tally.last_rows0,
                control_final_rows0_equals_the_revised_registered_value(tally.last_rows0),
            ));
        }
    }
    emitter.emit(&fields.join(" "));
}

#[cfg(test)]
mod tests;
