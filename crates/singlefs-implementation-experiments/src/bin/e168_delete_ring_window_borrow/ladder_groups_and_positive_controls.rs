//! 梯子组（同一 (盘大小点, S, K, 臂) 下扫 N）与阳性对照。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 梯子组（同一 (盘大小点, S, K, 臂) 下扫 N）与阳性对照
// ---------------------------------------------------------------------------------------------------------------------

/// 一组梯子的结局，供判决行数。
#[derive(Clone, Debug, Default)]
pub(super) struct GroupTally {
    pub(super) cells_deleted: u64,
    pub(super) cells_refused: u64,
    pub(super) cells_apex_over_reserve: u64,
    pub(super) cells_apex_over_ring_window: u64,
    pub(super) cells_not_fillable: u64,
    pub(super) is_saturated: bool,
    pub(super) are_the_largest_new_slots_growing_with_file_size: bool,
    pub(super) publishes_whose_new_slots_exceed_the_landing_demand: u64,
    pub(super) largest_new_slots: u64,
}

/// 一个盘大小点在这个 S、这个 N 上的几何；P1 先定环（修订第 15 条）、再照读法 C 现找（压满按 `ledger_for_the_fill`），找不到交回
/// None。P1 每找一次打一行 `E name=smallest_usable`（[`smallest_usable_line`]）。
fn geometry_of_the_point(
    point: DeviceSizePoint,
    slots_per_region: u64,
    data_units: u64,
    ledger_for_the_fill: LedgerForTheFill,
) -> Option<Geometry> {
    match point {
        DeviceSizePoint::MakeFilesystemSmallest => {
            Some(make_filesystem_smallest_geometry(slots_per_region))
        }
        DeviceSizePoint::SmallestUsable => {
            let search =
                smallest_usable_geometry(slots_per_region, data_units, ledger_for_the_fill);
            emit_product_line!(
                "{}",
                smallest_usable_line(slots_per_region, data_units, &search)
            );
            search.written_and_fillable
        }
        DeviceSizePoint::Narrow => Some(narrow_geometry(slots_per_region)),
        DeviceSizePoint::Product => Some(product_geometry(slots_per_region)),
    }
}

/// 一组梯子：8.4 的 N 依次跑，写不进那一点是上沿；末端不满足 M3 的饱和规则就按 ×2 续加，直到饱和、撞上沿或续加满
/// [`LADDER_EXTENSIONS_AT_MOST`] 次。每格的 M7 用这一组的 W_max，组行报 M3、M5、W_max。
pub(super) fn run_the_ladder_group(
    point: DeviceSizePoint,
    slots_per_region: u64,
    batch_data_units: u64,
    arm: Arm,
    set_aside_largest_new_slots: u64,
    hooks: &DeletePathHooks,
) -> GroupTally {
    let mut file_sizes: Vec<u64> = ladder_of_file_sizes(batch_data_units, slots_per_region)
        .into_iter()
        .filter(|data_units| *data_units <= LARGEST_FILE_DATA_UNITS)
        .collect();
    let mut deleted: Vec<(CellSetting, DeleteSummary)> = Vec::new();
    let mut tally = GroupTally::default();
    let mut ceiling = "none".to_owned();
    let mut extensions = 0;
    let mut next = 0;
    loop {
        if next == file_sizes.len() {
            let apex_series: Vec<(u64, i128)> = deleted
                .iter()
                .map(|(_, summary)| (summary.publishes, summary.apex))
                .collect();
            if apex_saturation(&apex_series, slots_per_region) == ApexSaturation::Saturated
                || extensions == LADDER_EXTENSIONS_AT_MOST
            {
                break;
            }
            let doubled = file_sizes.last().expect("梯子至少一点") * 2;
            if doubled > LARGEST_FILE_DATA_UNITS {
                ceiling = format!("apparatus_file_size_limit_{LARGEST_FILE_DATA_UNITS}");
                break;
            }
            file_sizes.push(doubled);
            extensions += 1;
        }
        let data_units = file_sizes[next];
        next += 1;
        let ledger_for_the_fill = LedgerForTheFill {
            ledger: ledger_of_the_arm(arm, slots_per_region, set_aside_largest_new_slots),
            set_ledger: hooks.set_ledger,
        };
        let Some(geometry) =
            geometry_of_the_point(point, slots_per_region, data_units, ledger_for_the_fill)
        else {
            ceiling = format!("smallest_usable_not_found_at_{data_units}");
            break;
        };
        let setting = CellSetting {
            geometry,
            batch_data_units,
            arm,
            data_units,
            set_aside_largest_new_slots,
            reclaim: ReclaimDuringTheDelete::AsToday,
        };
        match run_the_cell(&setting, hooks) {
            CellOutcome::FileNotWritten { step } => {
                ceiling = format!("file_not_written_at_{data_units}_step_{step}");
                break;
            }
            CellOutcome::NotFillable => tally.cells_not_fillable += 1,
            CellOutcome::Deleted { summary, .. } => deleted.push((setting, *summary)),
        }
    }
    let largest_new_slots = deleted
        .iter()
        .map(|(_, summary)| summary.largest_new_slots)
        .max()
        .unwrap_or(0);
    let bound = ring_window_bound_slots(slots_per_region, largest_new_slots);
    for (setting, summary) in &deleted {
        let apex_minus_bound = summary.apex - i128::from(bound);
        emit_product_line!(
            "E name=ring_window cell={} apex={} ring_window_bound={bound} apex_minus_bound={apex_minus_bound} m7_within_bound={}",
            setting.label(),
            summary.apex,
            apex_minus_bound <= 0
        );
        tally.cells_apex_over_ring_window += u64::from(apex_minus_bound > 0);
        tally.cells_apex_over_reserve += u64::from(!reserve_covers_the_apex(
            summary.apex,
            summary.reserve_at_the_apex,
        ));
        tally.cells_refused += u64::from(summary.refusal.is_some());
        tally.publishes_whose_new_slots_exceed_the_landing_demand +=
            summary.publishes_whose_new_slots_exceed_the_landing_demand;
    }
    let apex_series: Vec<(u64, i128)> = deleted
        .iter()
        .map(|(_, summary)| (summary.publishes, summary.apex))
        .collect();
    let largest_by_point: Vec<u64> = deleted
        .iter()
        .map(|(_, summary)| summary.largest_new_slots)
        .collect();
    let saturation = apex_saturation(&apex_series, slots_per_region);
    tally.cells_deleted = u64::try_from(deleted.len()).expect("格数");
    tally.is_saturated = saturation == ApexSaturation::Saturated;
    tally.are_the_largest_new_slots_growing_with_file_size =
        largest_new_slots_grow_with_file_size(&largest_by_point);
    tally.largest_new_slots = largest_new_slots;
    emit_product_line!(
        "E name=group point={} S={slots_per_region} K={batch_data_units} arm={} cells_deleted={} cells_not_fillable={} cells_refused={} cells_apex_over_reserve={} cells_apex_over_ring_window={} w_max={largest_new_slots} w_max_intent={} w_max_batches={} ring_window_bound={bound} m3_saturation={saturation:?} m5_w_grows_with_file_size={} w_exceeds_landing_publishes={} ladder_extensions={extensions} ladder_ceiling={ceiling} apex_series={apex_series:?} w_max_series={largest_by_point:?}",
        point.label(),
        arm.label(),
        tally.cells_deleted,
        tally.cells_not_fillable,
        tally.cells_refused,
        tally.cells_apex_over_reserve,
        tally.cells_apex_over_ring_window,
        deleted.iter().map(|(_, summary)| summary.intent_new_slots).max().unwrap_or(0),
        deleted.iter().map(|(_, summary)| summary.largest_batch_new_slots).max().unwrap_or(0),
        tally.are_the_largest_new_slots_growing_with_file_size,
        tally.publishes_whose_new_slots_exceed_the_landing_demand,
    );
    if point == DeviceSizePoint::SmallestUsable {
        emit_product_line!(
            "{}",
            group_line_on_the_one_mebibyte_ring_cells(
                slots_per_region,
                batch_data_units,
                arm,
                &deleted
            )
        );
    }
    tally
}

/// 修订第 15 条记一次输、只许收严那一半：P1 一组里环仍是 1 MiB 的那几格（就是环不放大时在这一组上跑得到的那一段梯子：环随 N 只增，
/// 1 MiB 的格是梯子的前缀），照组行同一套算法另算 M3、M5、W_max 与 M7，打一行 `E name=group_one_mebibyte_ring_cells`。组行的判定
/// 不动；这一行让放大环之后多出来的格不能把环不放大时判得出的「没饱和」「W 随文件大小涨」「越过环窗口界」抹掉。
fn group_line_on_the_one_mebibyte_ring_cells(
    slots_per_region: u64,
    batch_data_units: u64,
    arm: Arm,
    deleted: &[(CellSetting, DeleteSummary)],
) -> String {
    let on_the_one_mebibyte_ring: Vec<&DeleteSummary> = deleted
        .iter()
        .filter(|(setting, _)| setting.geometry.journal_ring_bytes == SMALL_JOURNAL_RING_BYTES)
        .map(|(_, summary)| summary)
        .collect();
    let largest_new_slots = on_the_one_mebibyte_ring
        .iter()
        .map(|summary| summary.largest_new_slots)
        .max()
        .unwrap_or(0);
    let bound = ring_window_bound_slots(slots_per_region, largest_new_slots);
    let cells_apex_over_ring_window = on_the_one_mebibyte_ring
        .iter()
        .filter(|summary| summary.apex - i128::from(bound) > 0)
        .count();
    let apex_series: Vec<(u64, i128)> = on_the_one_mebibyte_ring
        .iter()
        .map(|summary| (summary.publishes, summary.apex))
        .collect();
    let largest_by_point: Vec<u64> = on_the_one_mebibyte_ring
        .iter()
        .map(|summary| summary.largest_new_slots)
        .collect();
    format!(
        "E name=group_one_mebibyte_ring_cells point=P1 S={slots_per_region} K={batch_data_units} arm={} cells_deleted={} cells_refused={} cells_apex_over_reserve={} cells_apex_over_ring_window={cells_apex_over_ring_window} w_max={largest_new_slots} ring_window_bound={bound} m3_saturation={:?} m5_w_grows_with_file_size={} apex_series={apex_series:?} w_max_series={largest_by_point:?}",
        arm.label(),
        on_the_one_mebibyte_ring.len(),
        on_the_one_mebibyte_ring
            .iter()
            .filter(|summary| summary.refusal.is_some())
            .count(),
        on_the_one_mebibyte_ring
            .iter()
            .filter(|summary| !reserve_covers_the_apex(summary.apex, summary.reserve_at_the_apex))
            .count(),
        apex_saturation(&apex_series, slots_per_region),
        largest_new_slots_grow_with_file_size(&largest_by_point),
    )
}

/// 阳性对照里 N 与盘的迭代次数上限。
const POSITIVE_CONTROL_FILE_SIZE_ITERATIONS_AT_MOST: usize = 8;

/// 阳性对照定下来的盘、N 与 free_0（N = free_0 + 2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PositiveControlFileSize {
    pub(super) geometry: Geometry,
    pub(super) data_units: u64,
    free_after_the_fill_slots: u64,
}

/// 阳性对照迭代过的每一次：(N, 那块盘压满之后读到的空闲；取不到盘或压不满是 None)。
type PositiveControlIterations = Vec<(u64, Option<u64>)>;

/// 阳性对照的盘、文件大小 N 与 free_0（登记 5.5：N = free_0 + 2，free_0 = 压满之后那块盘的空闲槽；第十二节修订第 11 条）。N 与盘
/// 互相定——P1 的盘随 N 找，压满之后的空闲随 N 写出的树变——所以从 N = 1 起迭代：按 N 取盘（`geometry_for`）、在那块盘上写进 N 个
/// 单元、压满、读空闲（`free_after_the_fill`），读到的 + 2 = N 就是它，否则 N 取读到的 + 2 再来；至多
/// [`POSITIVE_CONTROL_FILE_SIZE_ITERATIONS_AT_MOST`] 次。交回 (盘, N, free_0)；不收敛、或某一次取不到盘、压不满，交回 Err(迭代过的
/// (N, 读到的空闲))。
fn positive_control_file_size_on_the_filled_disk(
    geometry_for: &dyn Fn(u64) -> Option<Geometry>,
    free_after_the_fill: &dyn Fn(Geometry, u64) -> Option<u64>,
) -> Result<PositiveControlFileSize, PositiveControlIterations> {
    let mut tried = Vec::new();
    let mut data_units = 1;
    for _ in 0..POSITIVE_CONTROL_FILE_SIZE_ITERATIONS_AT_MOST {
        let Some(geometry) = geometry_for(data_units) else {
            tried.push((data_units, None));
            return Err(tried);
        };
        let free_slots_read = free_after_the_fill(geometry, data_units);
        tried.push((data_units, free_slots_read));
        let Some(free_after_the_fill_slots) = free_slots_read else {
            return Err(tried);
        };
        if free_after_the_fill_slots + 2 == data_units {
            return Ok(PositiveControlFileSize {
                geometry,
                data_units,
                free_after_the_fill_slots,
            });
        }
        data_units = free_after_the_fill_slots + 2;
    }
    Err(tried)
}

/// 5.5 阳性对照的两个几何：最小可用盘（P1）× S = 16，与 4 GiB（P4）× S = 4。
pub(super) const POSITIVE_CONTROL_GEOMETRIES: [(DeviceSizePoint, u64); 2] = [
    (
        DeviceSizePoint::SmallestUsable,
        ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
    ),
    (DeviceSizePoint::Product, ROOT_RING_SLOTS_PER_REGION_MINIMUM),
];

/// 几何点与 S 的名字（`P1-S16`），`--geometry` 与 W_max 的键都用它。
pub(super) fn geometry_label_of(point: DeviceSizePoint, slots_per_region: u64) -> String {
    format!("{}-S{slots_per_region}", point.label())
}

/// 阳性对照的一片：一个几何上的一条臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PositiveControlSlice {
    pub(super) point: DeviceSizePoint,
    pub(super) slots_per_region: u64,
    pub(super) arm: Arm,
}

/// 阳性对照各片，按登记 5.5 的次序（几何在外、四条臂在里）；`filter` 只留这一个几何。
fn positive_control_slices(filter: Option<&str>) -> Vec<PositiveControlSlice> {
    POSITIVE_CONTROL_GEOMETRIES
        .into_iter()
        .filter(|(point, slots_per_region)| {
            filter.is_none_or(|wanted| wanted == geometry_label_of(*point, *slots_per_region))
        })
        .flat_map(|(point, slots_per_region)| {
            ALL_ARMS.map(|arm| PositiveControlSlice {
                point,
                slots_per_region,
                arm,
            })
        })
        .collect()
}

fn describe_positive_control(slice: &PositiveControlSlice) -> SliceDescription {
    SliceDescription {
        label: format!(
            "PC-{}-{}",
            geometry_label_of(slice.point, slice.slots_per_region),
            slice.arm.label()
        ),
        is_large: slice.point == DeviceSizePoint::Product,
        start_rank: 0,
    }
}

/// 5.5 阳性对照的一格：K = 1、N = free_0 + 2（free_0 读写进 N 个单元、压满之后那块盘本身，见
/// [`positive_control_file_size_on_the_filled_disk`]）、删除阶段回收关着。交回这一格的两条判定（[`PositiveControlCellVerdict`]）：
/// 出现拒绝（V1），B 臂另要拒绝是经会话推抬 F 那一步交回的（U12）。
fn run_one_positive_control(
    slice: &PositiveControlSlice,
    set_aside_largest_new_slots: u64,
    hooks: &DeletePathHooks,
) -> PositiveControlCellVerdict {
    let PositiveControlSlice {
        point,
        slots_per_region,
        arm,
    } = *slice;
    let group_label = geometry_label_of(point, slots_per_region);
    let ledger_for_the_fill = LedgerForTheFill {
        ledger: ledger_of_the_arm(arm, slots_per_region, set_aside_largest_new_slots),
        set_ledger: hooks.set_ledger,
    };
    let settled = positive_control_file_size_on_the_filled_disk(
        &|data_units| {
            geometry_of_the_point(point, slots_per_region, data_units, ledger_for_the_fill)
        },
        &|geometry, data_units| {
            free_slots_after_the_fill(geometry, data_units, ledger_for_the_fill)
        },
    );
    let PositiveControlFileSize {
        geometry,
        data_units,
        free_after_the_fill_slots,
    } = match settled {
        Ok(settled) => settled,
        Err(tried) => stop_with(
            "v1_positive_control_file_size_does_not_settle",
            &format!("geometry={group_label} arm={} tried={tried:?}", arm.label()),
        ),
    };
    let setting = CellSetting {
        geometry,
        batch_data_units: SMALLEST_BATCH_DATA_UNITS,
        arm,
        data_units,
        set_aside_largest_new_slots,
        reclaim: ReclaimDuringTheDelete::SwitchedOff,
    };
    let (refusal, floor_raise_publishes, floor_raise_stop) = match run_the_cell(&setting, hooks) {
        CellOutcome::Deleted {
            summary,
            floor_raise_publishes,
            floor_raise_stop_of_the_last_publish,
            free_after_the_fill,
        } => {
            if free_after_the_fill.iter().copied().max() != Some(free_after_the_fill_slots) {
                stop_with(
                    "v1_positive_control_free_0_differs_from_the_filled_cell",
                    &format!(
                        "cell={} free_0={free_after_the_fill_slots} cell_free_after_fill={free_after_the_fill:?}",
                        setting.label()
                    ),
                );
            }
            (
                summary.refusal.map(|(_position, refusal, _borrow)| refusal),
                floor_raise_publishes,
                floor_raise_stop_of_the_last_publish,
            )
        }
        CellOutcome::FileNotWritten { .. } | CellOutcome::NotFillable => {
            (None, 0, "none".to_owned())
        }
    };
    let verdict = PositiveControlCellVerdict::of_the_cell(arm.refusal, refusal);
    emit_product_line!(
        "E name=positive_control cell={} free_0={free_after_the_fill_slots} set_aside_w_max={set_aside_largest_new_slots} refused={} refusal={} floor_raise_publishes={floor_raise_publishes} floor_raise_stop={floor_raise_stop} u12_refused_with_no_space_after_raising_the_floor={} holds={}",
        setting.label(),
        verdict.is_refused,
        refusal.map_or("none", DeleteRefusal::label),
        verdict.refused_after_raising_the_floor_label(),
        verdict.holds()
    );
    verdict
}

/// 阳性对照一格的两条判定，各记各的（登记第十一节 V1：出现拒绝；第九节 U12：B 臂被拒之前至少推过一串抬 F）。U12 按读法乙判（
/// 登记第十二节修订第 17 条）：拒绝是经会话推抬 F 那一步交回的（[`DeleteRefusal::NoSpaceAfterRaisingTheFloor`]）即成立，
/// 推的那一串在预演里就被拒、一次都没落盘的也算；抬 F 落没落盘（[`DeletePublishReading::floor_raise_publishes`]）不再是条件。B 臂不经会话
/// 时拒绝种类是 [`DeleteRefusal::Landing`]（或 `Placement`），U12 不成立。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PositiveControlCellVerdict {
    pub(super) is_refused: bool,
    /// B 臂：拒绝种类是 `no_space_after_raising_the_floor`；A 臂没有这一条（None）。
    is_refused_with_no_space_after_raising_the_floor: Option<bool>,
}

impl PositiveControlCellVerdict {
    fn of_the_cell(refusal_handling: RefusalHandling, refusal: Option<DeleteRefusal>) -> Self {
        Self {
            is_refused: refusal.is_some(),
            is_refused_with_no_space_after_raising_the_floor: match refusal_handling {
                RefusalHandling::Direct => None,
                RefusalHandling::ThroughTheSession => {
                    Some(refusal == Some(DeleteRefusal::NoSpaceAfterRaisingTheFloor))
                }
            },
        }
    }

    fn refused_after_raising_the_floor_label(self) -> &'static str {
        match self.is_refused_with_no_space_after_raising_the_floor {
            None => "not_applicable",
            Some(true) => "true",
            Some(false) => "false",
        }
    }

    pub(super) fn holds(self) -> bool {
        self.is_refused && self.is_refused_with_no_space_after_raising_the_floor != Some(false)
    }
}

/// 阳性对照各格的计数，每个数只数一个量（verdict 行逐个报）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PositiveControlTally {
    cells: u64,
    /// 不出拒绝的格（V1）。
    cells_without_refusal: u64,
    /// B 臂出了拒绝、而拒绝种类不是 `no_space_after_raising_the_floor` 的格（U12，读法乙见 [`PositiveControlCellVerdict`]）。
    session_cells_refused_other_than_no_space_after_raising_the_floor: u64,
    /// 两条里有一条不合的格（`holds=false`）。
    cells_not_holding: u64,
}

impl PositiveControlTally {
    fn of_the_cells(verdicts: &[PositiveControlCellVerdict]) -> Self {
        let count = |judged: fn(&PositiveControlCellVerdict) -> bool| -> u64 {
            u64::try_from(verdicts.iter().filter(|verdict| judged(verdict)).count()).expect("格数")
        };
        Self {
            cells: u64::try_from(verdicts.len()).expect("格数"),
            cells_without_refusal: count(|verdict| !verdict.is_refused),
            session_cells_refused_other_than_no_space_after_raising_the_floor: count(|verdict| {
                verdict.is_refused
                    && verdict.is_refused_with_no_space_after_raising_the_floor == Some(false)
            }),
            cells_not_holding: count(|verdict| !verdict.holds()),
        }
    }

    /// verdict 行里阳性对照那几个字段。
    pub(super) fn verdict_fields(self) -> String {
        format!(
            "positive_control_cells={} positive_control_cells_without_refusal={} positive_control_session_cells_refused_other_than_no_space_after_raising_the_floor={} positive_control_cells_not_holding={}",
            self.cells,
            self.cells_without_refusal,
            self.session_cells_refused_other_than_no_space_after_raising_the_floor,
            self.cells_not_holding
        )
    }

    /// 有一格不合就停（打在 verdict 行之后）：不出拒绝按 V1 作废这一段（`v1_positive_control_cells_without_refusal`）；
    /// 都出了拒绝而 B 臂有一格的拒绝不是经会话推抬 F 那一步交回的，U12 那条断言不成立（`u12_session_arm_positive_control_refused_other_than_no_space_after_raising_the_floor`）。
    fn stop_reason(self) -> Option<&'static str> {
        if self.cells_without_refusal > 0 {
            Some("v1_positive_control_cells_without_refusal")
        } else if self.session_cells_refused_other_than_no_space_after_raising_the_floor > 0 {
            Some("u12_session_arm_positive_control_refused_other_than_no_space_after_raising_the_floor")
        } else {
            None
        }
    }
}

/// verdict 行打完之后：阳性对照有一格不合就打停机行、退 [`STOP_EXIT_CODE`]（[`PositiveControlTally::stop_reason`]），不打 `name=done`。
pub(super) fn stop_when_a_positive_control_does_not_hold(tally: PositiveControlTally) {
    if let Some(reason) = tally.stop_reason() {
        stop_with(reason, &tally.verdict_fields());
    }
}

/// 5.5 阳性对照：四条臂 × 两个几何，各格按片并行。W 臂账里那一份取 `largest_new_slots_of_each_geometry` 里那个几何的 W_max（没有记 0）。
pub(super) fn run_the_positive_controls(
    largest_new_slots_of_each_geometry: &BTreeMap<String, u64>,
    filter: Option<&str>,
    hooks: &DeletePathHooks,
    scheduling: SliceScheduling,
) -> PositiveControlTally {
    let slices = positive_control_slices(filter);
    let verdicts =
        run_slices_of_the_apparatus(&slices, &describe_positive_control, scheduling, &|slice| {
            let set_aside = largest_new_slots_of_each_geometry
                .get(&geometry_label_of(slice.point, slice.slots_per_region))
                .copied()
                .unwrap_or(0);
            run_one_positive_control(slice, set_aside, hooks)
        });
    PositiveControlTally::of_the_cells(&verdicts)
}

#[cfg(test)]
mod tests;
