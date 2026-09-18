//! 分配器（里程碑步 2 / 步 3）：内存里一张单元区的空闲图，两条落点政策，分配记录与记账增量在提交时增量维护。
//!
//! 用户数据落点（D3（空间分配） 已定项 8 / 已定项 10）：已选设备内起点槽号最小、起点 32768 对齐（偶数槽）、跨度内两槽都空、
//! 不在任何开放的聚簇段里、不在 `R` 保护期内（第一版 R 为空）。
//! 提交内生块（D3（空间分配） 已定项 5 / 已定项 10）：从开放聚簇段 bump，开放段 = 单元区内最低的 64 槽对齐全空段，bump 只在内存；
//! 码 3 容器按数据单元那一档取落点（起点 32768 对齐），码 2 节点取最低空槽。
//! 开放段装不下、又没有全空段时回落（D3（空间分配） 已定项 8 ②）：该设备内槽号最小的空槽，与 bump 游标绕开同一套位（已分配、影子账隔离、抬 F 扣住）。
//! 落点按设备取（D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取」）：每块盘按自己的空闲图各答一个，各盘一致才分配；
//! 有的盘答不出、或各盘答得不同，在动任何状态之前拒绝（`PlacementRefusal`）——`Placement` 两盘同槽，各盘不同槽第一版不支持。
//! 记账（D5（快照 / 空间记账机制） 已定项 7 / 已定项 8）：已分配 = 落点之和、容量 = 单元区、runs = 空闲槽的连续段数（逐设备）、
//! 全空聚簇段数逐设备——全部在分配那一刻增量维护，运行时不扫盘（`.claude/rules/fs-design.md` 第一格）。

use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES, UNIT_AREA_START_SLOT};

use std::collections::BTreeSet;

use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use crate::bytes::ByteWriter;

/// 跨度段的最高位借作已释放标志（D3（空间分配） 已定项 7 / 已定项 11）：0 = 仍分配、1 = 已释放，不占表达跨度值的位。
pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;

/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备 4, 槽号 6) + value (跨度 2 含已释放标志, 分配代或释放代 8)。
/// 释放时条目不删、不点删：改写成已释放 + 释放代，留到该落点被重新分配时覆盖（覆盖是步 5 回收接上时的事：
/// 回收过的落点被再分配时那条记录改写、不追加，槽号在每盘仍唯一；新落点罩住的别的已回收记录随这次分配删掉，同一块盘上的记录
/// 罩住的槽互不相交（I-5.4（分配记录罩住的槽互不相交））；回收要 F_生效 抬到释放代之上，见 `reclaim_released_up_to`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllocationRecord {
    pub device: DeviceIdentity,
    pub slot: SlotNumber,
    /// 纯跨度，不含标志位。
    pub span_slots: u16,
    /// 仍分配时是分配代；已释放时是释放代。
    pub generation: CheckpointTxg,
    pub is_released: bool,
}

impl AllocationRecord {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        assert_eq!(
            self.span_slots & ALLOCATION_RECORD_RELEASED_FLAG,
            0,
            "跨度值不许占到标志位"
        );
        let span_field = if self.is_released {
            self.span_slots | ALLOCATION_RECORD_RELEASED_FLAG
        } else {
            self.span_slots
        };
        let mut writer = ByteWriter::new(20);
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u16(span_field);
        writer.put_u64(self.generation.0);
        writer.assert_position(20, "分配记录");
        writer.into_bytes()
    }
    /// key = (设备 4, 槽号 6)。
    #[must_use]
    pub fn key_bytes(&self) -> Vec<u8> {
        self.to_bytes()[..10].to_vec()
    }
    /// 按字段比：设备身份，再槽号（D8（核心索引结构） 已定项 11）。
    #[must_use]
    pub fn sort_key(&self) -> (u32, u64) {
        (self.device.0, self.slot.0)
    }
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Self {
        let mut reader = crate::bytes::ByteReader::at(bytes, 0);
        let device = DeviceIdentity(reader.get_u32());
        let slot = SlotNumber(reader.get_six_byte_unsigned());
        let span_field = reader.get_u16();
        let generation = CheckpointTxg(reader.get_u64());
        Self {
            device,
            slot,
            span_slots: span_field & !ALLOCATION_RECORD_RELEASED_FLAG,
            generation,
            is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0,
        }
    }
}

/// 一个单元占几个槽：数据单元与打包容器 2，索引节点 1。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitFootprint {
    TwoSlotsAligned,
    OneSlot,
}

impl UnitFootprint {
    const fn slots(self) -> u64 {
        match self {
            UnitFootprint::TwoSlotsAligned => 2,
            UnitFootprint::OneSlot => 1,
        }
    }

    /// 起点对齐的步长（槽）：两槽的单元起点 32768 对齐（D18（块里携带什么信息） 已定项 9 按单元大小分档），一槽的单元每个槽都能起。
    const fn alignment_in_slots(self) -> u64 {
        match self {
            UnitFootprint::TwoSlotsAligned => 2,
            UnitFootprint::OneSlot => 1,
        }
    }
}

/// 开放段没有或装不下时，一块盘按自己的空闲图给提交内生块的去处。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitGeneratedDeviceAnswer {
    /// 这块盘上最低的全空段的起点：在这里开新段（D3（空间分配） 已定项 10 ①）。
    OpenEmptySegment(SlotNumber),
    /// 这块盘上没有全空段：回落到这块盘上槽号最小的空槽（D3（空间分配） 已定项 8 ②）。
    FallBackToLowestFreeSlot(SlotNumber),
}

/// 分配器拒绝一次分配的原因。拒绝发生在动任何状态之前：记录、位图、计数、开放段与 bump 游标都照旧。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlacementRefusal {
    /// 每块盘上都没有合政策的落点。
    NoFreeSlotOnAnyDevice,
    /// 有的盘上没有合政策的落点、别的盘上还有（盘不等大时小盘先满）：第一版的设备集合就是池里全部的盘，
    /// 小盘写满之后「当时可写的设备集合」怎么选是 D2（RAID 条带策略） 已定项 2 ⚠️ 那一半，没有条款；第一版不支持。
    SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec<DeviceIdentity> },
    /// 各盘按自己的空闲图取出的用户数据落点不同：D3（空间分配） 已定项 8 要各盘各取、D2（RAID 条带策略） 已定项 10 每个副本一条位置条目，
    /// 而 `Placement` 两盘同槽、发布路径的位置条目只带一个槽号；第一版不支持。
    UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
        slot_per_device: Vec<(DeviceIdentity, SlotNumber)>,
    },
    /// 开放段装不下之后各盘给的去处不同（一块开段一块回落，或开的段不同）：各盘上的聚簇段要不要对齐是 D3（空间分配） 已定项 8 待办 ①，
    /// 没有条款；`Placement` 两盘同槽；第一版不支持。
    CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
        answer_per_device: Vec<(DeviceIdentity, CommitGeneratedDeviceAnswer)>,
    },
}

/// 各盘各答一个之后合成的结论。
enum DeviceAgreement<Answer> {
    Agreed(Answer),
    NoAnswerOnAnyDevice,
    SomeDevicesWithoutAnswer(Vec<DeviceIdentity>),
    AnswersDiffer(Vec<(DeviceIdentity, Answer)>),
}

/// 每块盘的答案（`None` = 这块盘答不出）合成一个结论：全答不出、部分答不出、答得不同、一致。
fn agreement_across_devices<Answer: Copy + PartialEq>(
    answer_per_device: &[(DeviceIdentity, Option<Answer>)],
) -> DeviceAgreement<Answer> {
    let devices_without_answer: Vec<DeviceIdentity> = answer_per_device
        .iter()
        .filter(|(_, answer)| answer.is_none())
        .map(|(device, _)| *device)
        .collect();
    if devices_without_answer.len() == answer_per_device.len() {
        return DeviceAgreement::NoAnswerOnAnyDevice;
    }
    if !devices_without_answer.is_empty() {
        return DeviceAgreement::SomeDevicesWithoutAnswer(devices_without_answer);
    }
    let answers: Vec<(DeviceIdentity, Answer)> = answer_per_device
        .iter()
        .filter_map(|(device, answer)| answer.map(|present| (*device, present)))
        .collect();
    let first_answer = answers
        .first()
        .expect("上面两个判断之后每块盘都有答案、且至少一块盘")
        .1;
    if answers.iter().all(|(_, answer)| *answer == first_answer) {
        DeviceAgreement::Agreed(first_answer)
    } else {
        DeviceAgreement::AnswersDiffer(answers)
    }
}

/// 一块设备的空闲图与它的记账增量。
#[derive(Clone, Debug)]
pub struct DeviceFreeMap {
    pub device: DeviceIdentity,
    unit_area_slots: u64,
    /// 每个 16 KiB 槽一位：true = 已分配。
    allocated: Vec<bool>,
    /// 每个 64 槽聚簇段里已分配的槽数（增量维护全空段数用）。
    used_per_segment: Vec<u64>,
    /// 空闲槽的连续段数（增量维护）。
    free_runs: u64,
    /// 占着的槽数：仍分配的加上已释放、还在 defer 窗口里的——它们仍被根环里的有效根引用、仍占着空间
    /// （I-3.1（已分配统计对得上） 的读法 2026-09-14 用户定甲：按根环里全部有效根的引用取并集）。
    allocated_slots: u64,
    /// 空闲槽数，独立维护：分配时减、回收放回时加，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️：
    /// 那样 I-5.2（空闲统计对得上） 是恒真式；三方代码第一轮攻方腿打中）。已释放而还在 defer 窗口里的不算空闲。
    free_slots: u64,
    /// 影子账（D23（journal 的角色与格式） 已定项 14 回退段；D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」）：
    /// 只被被抛弃根引用的槽，当前账里既不是已分配也不是 defer，分配器却不许发出去，直到被抛弃的根离开根环。只住内存、不进记账行。
    isolated: Vec<bool>,
    isolated_per_segment: Vec<u64>,
    isolated_slots: u64,
    /// 其中已释放、还在 defer 窗口里的槽数（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
    deferred_slots: u64,
    /// 抬 F 回收、但 F 还没在每块盘上生效的槽：记账已经算它空闲，分配器却不许发出去，直到带新 F 的根落满每块盘
    /// （D16（发布语义） 已定项 1「每块幸存盘上都有带新 F 的持久根才生效」；alloc-basis 第二轮云端攻方腿打中：抬 F 自己的空发布开新段时
    /// 正好开到刚回收空的那一段，崩在两条带新 F 的根之间时 F 之下的根仍是候选、它们的单元已被盖）。只住内存。
    held_until_floor_takes_effect: Vec<bool>,
    held_per_segment: Vec<u64>,
}

impl DeviceFreeMap {
    /// 单元区从 50176 起到设备末尾；mkfs 占的槽由调用方标上。
    #[must_use]
    pub fn new(device: DeviceIdentity, device_bytes: u64) -> Self {
        let unit_area_slots = device_bytes / SLOT_BYTES - UNIT_AREA_START_SLOT;
        let segments = unit_area_slots.div_ceil(CLUSTER_SEGMENT_SLOTS);
        Self {
            device,
            unit_area_slots,
            allocated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
            used_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            free_runs: u64::from(unit_area_slots > 0),
            allocated_slots: 0,
            free_slots: unit_area_slots,
            deferred_slots: 0,
            isolated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
            isolated_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            isolated_slots: 0,
            held_until_floor_takes_effect: vec![
                false;
                usize::try_from(unit_area_slots)
                    .expect("单元区槽数")
            ],
            held_per_segment: vec![0; usize::try_from(segments).expect("段数")],
        }
    }

    fn index(slot: SlotNumber) -> usize {
        usize::try_from(slot.0 - UNIT_AREA_START_SLOT).expect("槽号在单元区内")
    }

    #[must_use]
    pub fn is_free(&self, slot: SlotNumber) -> bool {
        slot.0 >= UNIT_AREA_START_SLOT
            && slot.0 < UNIT_AREA_START_SLOT + self.unit_area_slots
            && !self.allocated[Self::index(slot)]
            && !self.isolated[Self::index(slot)]
            && !self.held_until_floor_takes_effect[Self::index(slot)]
    }

    #[must_use]
    pub fn unit_area_slots(&self) -> u64 {
        self.unit_area_slots
    }
    #[must_use]
    pub fn allocated_slots(&self) -> u64 {
        self.allocated_slots
    }
    /// 独立维护的空闲槽数（I-5.2（空闲统计对得上） 拿它与「已分配」相加对容量），已释放而还在 defer 窗口里的不算空闲。
    #[must_use]
    pub fn free_slots(&self) -> u64 {
        self.free_slots
    }
    #[must_use]
    pub fn deferred_slots(&self) -> u64 {
        self.deferred_slots
    }

    /// 把一个仍分配的落点放进 defer 队列：槽仍占着（分配器不许再发它），只是记账上从「仍分配」挪到「待释放」。
    pub fn mark_released(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            self.allocated[start..end].iter().all(|taken| *taken),
            "释放的跨度里有没分配的槽"
        );
        self.deferred_slots += span;
    }
    /// 隔离一个只被被抛弃根引用的落点：不进已分配、不进 defer、不动空闲计数，只让分配器绕开它（用户数据落点与开放段都不落在它上面）。
    /// 隔离位独立于分配位：一个槽在当前账里已分配、后来被释放、再被回收，隔离位照样让它发不出去——抬 F 之后候选集缩小，
    /// 一个此前靠候选根豁免的槽会在回收之前被补隔离（`mount::raise_rollback_floor`）。同一个槽隔离两次不重复计数。
    pub fn isolate(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
        for index in start..end {
            if !self.isolated[index] {
                self.isolated[index] = true;
                let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
                self.isolated_per_segment[segment] += 1;
                self.isolated_slots += 1;
            }
        }
    }

    /// 影子账隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」的这块盘那一项）。
    #[must_use]
    pub fn isolated_slots(&self) -> u64 {
        self.isolated_slots
    }

    /// 抬 F 回收的落点先扣住：空闲计数照加，位图照清，但 `is_free` 与开段都绕开它，直到 `release_holds`。
    pub fn hold_until_floor_takes_effect(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
        for index in start..end {
            if !self.held_until_floor_takes_effect[index] {
                self.held_until_floor_takes_effect[index] = true;
                let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
                self.held_per_segment[segment] += 1;
            }
        }
    }

    /// 提交内生块的 bump 游标要绕开的槽：已分配、影子账隔离、抬 F 扣住三种位任一为真（代码三方第三轮云端攻方腿打中：开段那一刻
    /// 的三个条件担保不了开段之后才置的隔离位与扣住位）。
    #[must_use]
    pub fn is_blocked_for_commit_generated(&self, slot: SlotNumber) -> bool {
        let index = Self::index(slot);
        self.allocated[index] || self.isolated[index] || self.held_until_floor_takes_effect[index]
    }

    /// F 在每块盘上生效之后把扣住的槽放开。
    pub fn release_holds(&mut self) {
        self.held_until_floor_takes_effect.fill(false);
        self.held_per_segment.fill(0);
    }

    /// 回收：一个已释放、释放代 ≤ max(F_生效, 环里最旧有效根) 的落点回到空闲（D16（发布语义） 已定项 1 的可再分配谓词）：
    /// 位图清掉、占着的槽数与 defer 队列各减、空闲加——记账的空闲字节到这一刻才动（里程碑「第二个事务」步 5）。
    pub fn mark_reclaimed(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            self.allocated[start..end].iter().all(|taken| *taken),
            "回收的跨度里有没分配的槽"
        );
        let left_free = start > 0 && !self.allocated[start - 1];
        let right_free = end < self.allocated.len() && !self.allocated[end];
        // runs 的增量：两边都空是把两段并成一段（−1），两边都占是新开一段（+1），一边空是接上去（不变）。
        self.free_runs = self.free_runs + 1 - u64::from(left_free) - u64::from(right_free);
        for index in start..end {
            self.allocated[index] = false;
            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
            self.used_per_segment[segment] -= 1;
        }
        self.allocated_slots -= span;
        self.deferred_slots -= span;
        self.free_slots += span;
    }

    #[must_use]
    pub fn free_runs(&self) -> u64 {
        self.free_runs
    }
    #[must_use]
    pub fn empty_segments(&self) -> u64 {
        u64::try_from(
            self.used_per_segment
                .iter()
                .filter(|used| **used == 0)
                .count(),
        )
        .expect("段数")
    }

    /// 段内是不是整段空闲。
    #[must_use]
    pub fn segment_is_empty(&self, segment_index: usize) -> bool {
        self.used_per_segment[segment_index] == 0
    }

    /// 把 `[slot, slot + span)` 标成已分配，并增量更新 runs 与段计数。要求整个跨度此前都空闲。
    pub fn mark_allocated(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
        assert!(
            self.allocated[start..end].iter().all(|taken| !*taken),
            "跨度里有已分配的槽"
        );
        // runs 的增量：被切的那一段空闲 run 左右各剩不剩东西。
        let left_free = start > 0 && !self.allocated[start - 1];
        let right_free = end < self.allocated.len() && !self.allocated[end];
        let pieces_left = u64::from(left_free) + u64::from(right_free);
        self.free_runs = self.free_runs + pieces_left - 1;
        for index in start..end {
            self.allocated[index] = true;
            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
            self.used_per_segment[segment] += 1;
        }
        self.allocated_slots += span;
        self.free_slots -= span;
    }

    /// 用户数据落点：起点槽号最小、偶数槽（32768 对齐）、两槽都空、不在任何一个聚簇段里（D3（空间分配） 已定项 10 ②
    /// 逐字「候选落点不在任何开放的聚簇段里」，量词是「任何」不是「当前那一个」）。
    #[must_use]
    pub fn lowest_user_data_slot(
        &self,
        cluster_segments: &BTreeSet<SlotNumber>,
    ) -> Option<SlotNumber> {
        let mut candidate = UNIT_AREA_START_SLOT;
        let end = UNIT_AREA_START_SLOT + self.unit_area_slots;
        while candidate + 1 < end {
            let inside_cluster_segment = cluster_segments.iter().any(|segment| {
                candidate >= segment.0 && candidate < segment.0 + CLUSTER_SEGMENT_SLOTS
            });
            if !inside_cluster_segment
                && self.is_free(SlotNumber(candidate))
                && self.is_free(SlotNumber(candidate + 1))
            {
                return Some(SlotNumber(candidate));
            }
            candidate += 2;
        }
        None
    }

    /// 单元区内最低的 64 槽对齐全空段的起点。
    #[must_use]
    pub fn lowest_empty_segment(&self) -> Option<SlotNumber> {
        let per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        let full_segments = self.allocated.len() / per_segment;
        (0..full_segments)
            .find(|segment| {
                self.used_per_segment[*segment] == 0
                    && self.isolated_per_segment[*segment] == 0
                    && self.held_per_segment[*segment] == 0
            })
            .map(|segment| {
                SlotNumber(
                    UNIT_AREA_START_SLOT
                        + u64::try_from(segment).expect("段号") * CLUSTER_SEGMENT_SLOTS,
                )
            })
    }

    /// 提交内生块的回落落点（D3（空间分配） 已定项 8 ②「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」）：
    /// 这块盘单元区里起点槽号最小、整个跨度都不被挡的落点——挡的是 bump 游标绕开的同一套位（已分配、影子账隔离、抬 F 扣住），
    /// 回落不是绕开影子账或扣住的第二条路；两槽的码 3 容器照数据单元那一档起点 32768 对齐（D3（空间分配） 已定项 10 ⑤）。第一版 `R` 为空。
    #[must_use]
    pub fn lowest_commit_generated_fallback_slot(
        &self,
        footprint: UnitFootprint,
    ) -> Option<SlotNumber> {
        let unit_area_end = UNIT_AREA_START_SLOT + self.unit_area_slots;
        let mut candidate = UNIT_AREA_START_SLOT;
        while candidate + footprint.slots() <= unit_area_end {
            let is_whole_span_unblocked = (candidate..candidate + footprint.slots())
                .all(|slot| !self.is_blocked_for_commit_generated(SlotNumber(slot)));
            if is_whole_span_unblocked {
                return Some(SlotNumber(candidate));
            }
            candidate += footprint.alignment_in_slots();
        }
        None
    }

    /// 开放段没有或装不下时这块盘给提交内生块的去处：最低的全空段；没有全空段就回落；连回落的空槽都没有就 `None`。
    #[must_use]
    pub fn commit_generated_answer_without_open_segment(
        &self,
        footprint: UnitFootprint,
    ) -> Option<CommitGeneratedDeviceAnswer> {
        match self.lowest_empty_segment() {
            Some(segment_start) => {
                Some(CommitGeneratedDeviceAnswer::OpenEmptySegment(segment_start))
            }
            None => self
                .lowest_commit_generated_fallback_slot(footprint)
                .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),
        }
    }
}

/// 回收的槽什么时候能再发出去：重建分配器时 F 已经生效，立刻；抬 F 时要等带新 F 的根落满每块盘。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReclaimedReuse {
    Immediately,
    HeldUntilFloorTakesEffect,
}

/// 一次发布里分配的一个落点，两盘同槽（D2（RAID 条带策略） 已定项 10：一个单元整个落在一列上、两盘各一份）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub slot: SlotNumber,
    pub span: u64,
}

/// 一块盘上、起点就是这次落点的那个槽的记录，让位之后是什么样子。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordAtThePlacementSlot {
    /// 那里原来是一条已回收的记录，已经改写成这次的分配：不再追加。
    RewrittenFromReclaimed,
    /// 那里没有记录：由调用方追加一条。
    Absent,
}

/// 一块盘上，与新落点 [槽, 槽 + 跨度) 相交的已有记录给这次分配让位（I-5.4（分配记录罩住的槽互不相交））：
/// 起点就是这个槽的已回收记录改写成这次的分配（字节表五「已释放记录被重用之后再改写一次」），起点不是这个槽的已回收记录
/// 随这次分配一起删掉——留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽，重建在 `mark_allocated` 的断言上 panic、
/// 池从此挂不上可写（代码三方第二轮 Z1-d：两槽的数据单元复用一条跨 1 的已回收记录，下一槽另一条跨 1 的已回收记录留着）。
/// 删掉的记录罩住的槽已经回到空闲（`mark_reclaimed` 清了位图、减了计数），删它不动位图与计数；起点不在这次落点里的那一截槽
/// 删完之后就没有记录，与从没分配过的槽同样（D3（空间分配） 已定项 7：条目留到该落点被重新分配时覆盖，而它罩住的槽正被这次分配重新占用）。
///
/// # Panics
/// 相交的记录里有不是已回收的（仍分配的，或已释放还在 defer 窗口里的）：分配器只把位图上空闲的槽发出来，而那样的记录罩住的槽
/// 在位图上仍占着——走到这里说明位图与记录分叉了。
fn make_room_for_record_on_device(
    records: &mut Vec<AllocationRecord>,
    reclaimed: &mut BTreeSet<(DeviceIdentity, SlotNumber)>,
    device: DeviceIdentity,
    placement: Placement,
    generation: CheckpointTxg,
) -> RecordAtThePlacementSlot {
    let placement_end = placement.slot.0 + placement.span;
    let intersecting_record_slots: Vec<SlotNumber> = records
        .iter()
        .filter(|record| {
            record.device == device
                && record.slot.0 < placement_end
                && placement.slot.0 < record.slot.0 + u64::from(record.span_slots)
        })
        .map(|record| record.slot)
        .collect();
    let placement_slot_was_reclaimed = reclaimed.contains(&(device, placement.slot));
    let mut record_at_the_placement_slot = RecordAtThePlacementSlot::Absent;
    for record_slot in intersecting_record_slots {
        let was_reclaimed = reclaimed.remove(&(device, record_slot));
        assert!(
            was_reclaimed,
            "盘 {device:?} 上与新落点（槽 {:?} 跨 {}）相交的记录（槽 {record_slot:?}）不是已回收的：分配器只发位图上空闲的槽，位图与记录分叉了",
            placement.slot,
            placement.span
        );
        if record_slot == placement.slot {
            let existing = records
                .iter_mut()
                .find(|record| record.device == device && record.slot == record_slot)
                .expect("上面刚从记录里找出这个槽");
            assert!(existing.is_released, "回收过的落点的记录该是已释放");
            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");
            existing.generation = generation;
            existing.is_released = false;
            record_at_the_placement_slot = RecordAtThePlacementSlot::RewrittenFromReclaimed;
        } else {
            records.retain(|record| !(record.device == device && record.slot == record_slot));
        }
    }
    assert!(
        !placement_slot_was_reclaimed
            || record_at_the_placement_slot == RecordAtThePlacementSlot::RewrittenFromReclaimed,
        "回收过的落点有它的已释放记录"
    );
    record_at_the_placement_slot
}

/// 池级分配器：每块盘按同一条规则各自取，各盘取得一致才给号（盘可以不等大，D2（RAID 条带策略） 已定项 2）。
#[derive(Clone, Debug)]
pub struct PoolAllocator {
    pub devices: Vec<DeviceFreeMap>,
    /// 开放聚簇段的起点与 bump 游标（只在内存）。`None` = 这会儿没有段可 bump（还没开过，或上一次落点走了回落）。
    open_segment: Option<SlotNumber>,
    bump_cursor: u64,
    /// 这次挂载开过的聚簇段的起点，开过就一直算数（只在内存，重开后按 D3（空间分配） 已定项 10 ①「挂载后新开一段」重来）。
    /// 用户数据的候选集排除它们全部（已定项 10 ②「不在任何开放的聚簇段里」），不只排除当前那一个——回落把 `open_segment` 置空之后
    /// 那一段仍装着这次提交的内生块，D3（空间分配） 已定项 8 第 2 条「聚簇段只给提交内生块」照样压着它（增补 2 第 20c 行，
    /// 代码三方第一轮打中）。段里的块被回收空了也仍算聚簇段：它还能被 `lowest_empty_segment` 重新开来装提交内生块。
    cluster_segments: BTreeSet<SlotNumber>,
    /// C146（无空段时的回落政策全仓无定义） ② 的运行时计数：实际落点与政策函数不一致的次数，第一个事务恒 0。
    /// 落点按设备取之后，发出去的落点就是每块盘各自的政策答案（答得不同在动状态之前拒绝），这个数按构造恒 0；
    /// 此前它比的是「盘 0 的答案」与「各盘答案的最小值」。字段留着是因为发布结果与真设备二进制照报它；删还是改成别的口径，没有定。
    pub policy_mismatches: u64,
    records: Vec<AllocationRecord>,
    /// 已回收、还没被复用的落点（盘上那条记录仍写着已释放；复用时那条记录被改写）。只住内存。
    reclaimed: BTreeSet<(DeviceIdentity, SlotNumber)>,
    /// mkfs 写出的第 0 版树表单元的落点：第一个文件版本重写树表时把它释放（COW 换下的单元进 defer 队列，D3（空间分配） 已定项 7）；
    /// 重开之后上一版从盘上重建、释放经映射与根记录走，这里留空。
    format_time_tree_table: Option<Placement>,
}

impl PoolAllocator {
    #[must_use]
    pub fn new(devices: Vec<DeviceFreeMap>) -> Self {
        Self {
            devices,
            open_segment: None,
            bump_cursor: 0,
            cluster_segments: BTreeSet::new(),
            policy_mismatches: 0,
            records: Vec::new(),
            reclaimed: BTreeSet::new(),
            format_time_tree_table: None,
        }
    }

    /// 重开时从盘上那棵分配记录树重建分配器（D23（journal 的角色与格式） 已定项 14：defer 队列、分配器游标、记账的现行值从所选根那棵账重新载入）：
    /// 每条记录把自己那块盘上的槽标成占着，已释放的再进 defer 队列；空闲计数随 `mark_allocated` 减、与写路径同一条事件路径。
    /// 开放段与 bump 游标只在内存、盘上没有（分配记录树不记它们），重开后按「没有开放段」起步：下一个提交内生块从最低的全空段开——
    /// 上一段里没用完的槽先留着（C146（无空段时的回落政策全仓无定义） 之外的另一格，里程碑步 3 的决策点）。
    #[must_use]
    pub fn rebuild_from_records(
        devices: Vec<DeviceFreeMap>,
        records: Vec<AllocationRecord>,
    ) -> Self {
        let mut allocator = Self::new(devices);
        for record in &records {
            let device_map = allocator
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == record.device)
                .expect("分配记录的盘在池里：走读逐盘核过");
            let span = u64::from(record.span_slots);
            device_map.mark_allocated(record.slot, span);
            if record.is_released {
                device_map.mark_released(record.slot, span);
            }
        }
        allocator.records = records;
        // mkfs 那片第 0 版树表单元还没被换下（分配代 0、跨度 1、未释放；m1 实例表跨度 2，字节表一）时，重开之后第一个文件版本
        // 照样要把它释放——不然 mkfs 与第一个文件版本不在同一个进程里那一格永远漏一槽（步 4 / 步 5 代码三方第一轮本地辩方腿两份样本都指出）。
        allocator.format_time_tree_table = allocator
            .records
            .iter()
            .find(|record| {
                record.generation == CheckpointTxg(0)
                    && record.span_slots == 1
                    && !record.is_released
            })
            .map(|record| Placement {
                slot: record.slot,
                span: u64::from(record.span_slots),
            });
        allocator
    }

    /// mkfs 写在单元区里的两个单元：每盘一条分配代 0 的分配记录（字节表五：20 条里 m1 / m2 各两条，分配代 0）。
    /// 单元区之外的固定结构（超级块、根环、journal 环）不写分配记录、分配器不下探（D3（空间分配） 已定项 10 ④）。
    pub fn mark_format_time_units(&mut self, instance_table: Placement, tree_table: Placement) {
        self.record(instance_table, CheckpointTxg(0));
        self.record(tree_table, CheckpointTxg(0));
        self.format_time_tree_table = Some(tree_table);
    }

    /// mkfs 写出的第 0 版树表单元（还没被第一个文件版本换下时 `Some`）。
    #[must_use]
    pub fn format_time_tree_table(&self) -> Option<Placement> {
        self.format_time_tree_table
    }

    #[must_use]
    pub fn records(&self) -> &[AllocationRecord] {
        &self.records
    }

    /// 某块盘上某个槽的分配记录；没分配过就 `None`。释放判定路径拿它核「映射查出来的落点真的在册、跨度对得上」。
    #[must_use]
    pub fn record_for(
        &self,
        device: DeviceIdentity,
        slot: SlotNumber,
    ) -> Option<&AllocationRecord> {
        self.records
            .iter()
            .find(|record| record.device == device && record.slot == slot)
    }

    /// 记下一次分配：每块盘上一条落点 (设备, 槽, 跨度) 的记录，再在位图上标占。同一块盘上与 [槽, 槽 + 跨度) 相交的已有记录
    /// 先让位（`make_room_for_record_on_device`）：起点就是这个槽的已回收记录改写成这次的分配，别的已回收记录删掉。
    fn record(&mut self, placement: Placement, generation: CheckpointTxg) {
        for device in &self.devices {
            match make_room_for_record_on_device(
                &mut self.records,
                &mut self.reclaimed,
                device.device,
                placement,
                generation,
            ) {
                RecordAtThePlacementSlot::RewrittenFromReclaimed => {}
                RecordAtThePlacementSlot::Absent => {
                    self.records.push(AllocationRecord {
                        device: device.device,
                        slot: placement.slot,
                        span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),
                        generation,
                        is_released: false,
                    });
                }
            }
        }
        for device in &mut self.devices {
            device.mark_allocated(placement.slot, placement.span);
        }
    }

    /// 释放一个落点（D3（空间分配） 已定项 7：「释放」= 放进 defer 队列那一刻）：每盘那条分配记录改写成已释放 + 释放代，
    /// 条目不删；槽仍占着，要等释放代 ≤ max(F_生效, 环里最旧有效根) 才可再分配（D16（发布语义） 已定项 1）。
    pub fn release(&mut self, placement: Placement, release_generation: CheckpointTxg) {
        for device in &mut self.devices {
            let record = self
                .records
                .iter_mut()
                .find(|record| record.device == device.device && record.slot == placement.slot)
                .expect("释放的落点要有分配记录");
            assert!(!record.is_released, "同一个落点释放了两次");
            assert_eq!(
                u64::from(record.span_slots),
                placement.span,
                "释放的跨度与分配记录不符"
            );
            record.is_released = true;
            record.generation = release_generation;
            device.mark_released(placement.slot, placement.span);
        }
    }

    /// 用户数据：见 `try_allocate_user_data`；拒绝的原因在这里丢掉，发布路径把 `None` 报成装不下。
    pub fn allocate_user_data(&mut self, generation: CheckpointTxg) -> Option<Placement> {
        self.try_allocate_user_data(generation).ok()
    }

    /// 用户数据：每块盘按政策函数各自取（D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取该设备内」），各盘一致才分配。
    ///
    /// # Errors
    /// 拒绝时分配器一样都没动：全部盘都没有落点、有的盘没有（盘不等大时小盘先满）、各盘的落点不同。
    pub fn try_allocate_user_data(
        &mut self,
        generation: CheckpointTxg,
    ) -> Result<Placement, PlacementRefusal> {
        let cluster_segments = &self.cluster_segments;
        let slot_per_device: Vec<(DeviceIdentity, Option<SlotNumber>)> = self
            .devices
            .iter()
            .map(|device| {
                (
                    device.device,
                    device.lowest_user_data_slot(cluster_segments),
                )
            })
            .collect();
        let slot = match agreement_across_devices(&slot_per_device) {
            DeviceAgreement::Agreed(slot) => slot,
            DeviceAgreement::NoAnswerOnAnyDevice => {
                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
            }
            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {
                return Err(
                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },
                );
            }
            DeviceAgreement::AnswersDiffer(slot_per_device) => {
                return Err(
                    PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
                        slot_per_device,
                    },
                );
            }
        };
        let placement = Placement { slot, span: 2 };
        self.record(placement, generation);
        Ok(placement)
    }

    /// 提交内生块：见 `try_allocate_commit_generated`；拒绝的原因在这里丢掉，发布路径把 `None` 报成装不下。
    pub fn allocate_commit_generated(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Option<Placement> {
        self.try_allocate_commit_generated(footprint, generation)
            .ok()
    }

    /// 提交内生块：从开放段 bump；容器按 32768 对齐档，节点取游标处最低空槽。开放段没有或装不下时每块盘按自己的空闲图答一个去处——
    /// 最低的全空段（开新段），没有全空段就回落到槽号最小的空槽（D3（空间分配） 已定项 8 ②）——各盘答得一致才分配。
    ///
    /// # Errors
    /// 拒绝时分配器一样都没动（装不下的开放段也照旧开着）：全部盘都没有去处、有的盘没有、各盘的去处不同。
    pub fn try_allocate_commit_generated(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Result<Placement, PlacementRefusal> {
        if let Some(open) = self.open_segment {
            if let Some(slot) = self.bump_slot_in_open_segment(open, footprint) {
                return Ok(self.record_bumped(slot, footprint, generation));
            }
        }
        let answer_per_device: Vec<(DeviceIdentity, Option<CommitGeneratedDeviceAnswer>)> = self
            .devices
            .iter()
            .map(|device| {
                (
                    device.device,
                    device.commit_generated_answer_without_open_segment(footprint),
                )
            })
            .collect();
        let answer = match agreement_across_devices(&answer_per_device) {
            DeviceAgreement::Agreed(answer) => answer,
            DeviceAgreement::NoAnswerOnAnyDevice => {
                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
            }
            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {
                return Err(
                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },
                );
            }
            DeviceAgreement::AnswersDiffer(answer_per_device) => {
                return Err(
                    PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
                        answer_per_device,
                    },
                );
            }
        };
        match answer {
            CommitGeneratedDeviceAnswer::OpenEmptySegment(segment_start) => {
                self.open_segment = Some(segment_start);
                self.bump_cursor = segment_start.0;
                self.cluster_segments.insert(segment_start);
                let slot = self
                    .bump_slot_in_open_segment(segment_start, footprint)
                    .expect("每块盘都答了同一个全空段（已分配、隔离、扣住都是 0，整段在每块盘的单元区里）：64 槽装得下任何一个单元");
                Ok(self.record_bumped(slot, footprint, generation))
            }
            CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(slot) => {
                // 回落之后没有段可 bump 了，但刚才那一段仍是聚簇段、仍装着这次提交的内生块：它留在 `cluster_segments` 里，
                // 用户数据照旧不许落进去（D3（空间分配） 已定项 8 第 2 条；增补 2 第 20c 行）。
                self.open_segment = None;
                let placement = Placement {
                    slot,
                    span: footprint.slots(),
                };
                self.record(placement, generation);
                Ok(placement)
            }
        }
    }

    /// 开放段里从游标起第一个整跨度在每块盘上都不被挡的对齐起点；段里装不下就 `None`。只读，不挪游标。
    fn bump_slot_in_open_segment(
        &self,
        open: SlotNumber,
        footprint: UnitFootprint,
    ) -> Option<SlotNumber> {
        let segment_end = open.0 + CLUSTER_SEGMENT_SLOTS;
        let mut start = self.bump_cursor;
        if footprint == UnitFootprint::TwoSlotsAligned && !start.is_multiple_of(2) {
            start += 1;
        }
        // 游标前方的槽在开段之后可能被隔离或扣住（抬 F 重算影子账、扣住回收的槽都发生在活分配器上）：跨度里有一个被挡就整个往后挪，
        // 挪的步长按对齐档。
        while start + footprint.slots() <= segment_end
            && (start..start + footprint.slots()).any(|slot| {
                self.devices
                    .iter()
                    .any(|device| device.is_blocked_for_commit_generated(SlotNumber(slot)))
            })
        {
            start += footprint.alignment_in_slots();
        }
        if start + footprint.slots() > segment_end {
            return None;
        }
        Some(SlotNumber(start))
    }

    /// 记下开放段里 bump 出的一个落点，游标挪到它后面。
    fn record_bumped(
        &mut self,
        slot: SlotNumber,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Placement {
        let placement = Placement {
            slot,
            span: footprint.slots(),
        };
        self.bump_cursor = slot.0 + footprint.slots();
        self.record(placement, generation);
        placement
    }

    /// 回收：释放代 ≤ `floor` 的已释放落点回到空闲，等着被再分配（D16（发布语义） 已定项 1：可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)；
    /// 第一版根环 24 槽、种子 txg 0，环里最旧有效根恒 0，`floor` 就是 F_生效）。回收过的不重复回收；返回这次回收的落点（两盘同槽，按盘 0 报）。
    /// `reuse` 说回收的槽什么时候能发：重建时 F 已经生效，立刻；抬 F 时要等带新 F 的根落满每块盘（`release_reclaim_holds`）。
    pub fn reclaim_released_up_to(
        &mut self,
        floor: CheckpointTxg,
        reuse: ReclaimedReuse,
    ) -> Vec<Placement> {
        let mut reclaimed_now = Vec::new();
        let candidates: Vec<AllocationRecord> = self
            .records
            .iter()
            .filter(|record| record.is_released && record.generation <= floor)
            .copied()
            .collect();
        for record in candidates {
            let key = (record.device, record.slot);
            if !self.reclaimed.insert(key) {
                continue;
            }
            let device_map = self
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == record.device)
                .expect("分配记录的盘在池里");
            device_map.mark_reclaimed(record.slot, u64::from(record.span_slots));
            match reuse {
                ReclaimedReuse::Immediately => {}
                ReclaimedReuse::HeldUntilFloorTakesEffect => {
                    device_map
                        .hold_until_floor_takes_effect(record.slot, u64::from(record.span_slots));
                }
            }
            if record.device == self.devices[0].device {
                reclaimed_now.push(Placement {
                    slot: record.slot,
                    span: u64::from(record.span_slots),
                });
            }
        }
        reclaimed_now
    }

    /// 抬 F 生效（带新 F 的根落满每块盘）之后，把扣住的回收槽放开。
    pub fn release_reclaim_holds(&mut self) {
        for device_map in &mut self.devices {
            device_map.release_holds();
        }
    }

    /// 回退的影子账：把一个只被被抛弃根引用的落点在它那块盘上隔离。
    pub fn isolate_abandoned(&mut self, device: DeviceIdentity, slot: SlotNumber, span: u64) {
        let device_map = self
            .devices
            .iter_mut()
            .find(|device_map| device_map.device == device)
            .expect("被抛弃根的分配记录的盘在池里：走读逐盘核过");
        device_map.isolate(slot, span);
    }

    #[must_use]
    pub fn open_segment(&self) -> Option<SlotNumber> {
        self.open_segment
    }

    /// 这次挂载开过的聚簇段的起点（含已经回落掉、这会儿不 bump 的那些）：用户数据一个都不许落进去。
    #[must_use]
    pub fn cluster_segments(&self) -> &BTreeSet<SlotNumber> {
        &self.cluster_segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 开段之后才隔离的槽（抬 F 重算影子账落在活分配器上）bump 游标要绕开：先开段发一个单槽单元，再把游标前方的 50180 隔离，
    /// 之后连发四个单槽单元一个都不落在 50180 上。
    #[test]
    fn commit_generated_bump_skips_a_slot_isolated_after_the_segment_was_opened() {
        let mut pool = pool_after_mkfs();
        let first = pool
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
            .expect("开段");
        let open = pool.open_segment().expect("开放段");
        let isolated = SlotNumber(open.0 + 4);
        assert!(first.slot < isolated, "隔离的槽在游标前方");
        pool.isolate_abandoned(DeviceIdentity(0), isolated, 1);
        pool.isolate_abandoned(DeviceIdentity(1), isolated, 1);
        let handed_out: Vec<SlotNumber> = (0..4)
            .map(|_| {
                pool.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
                    .expect("开放段还有空")
                    .slot
            })
            .collect();
        assert!(
            !handed_out.contains(&isolated),
            "被隔离的 {isolated:?} 发了出去：{handed_out:?}"
        );
        assert!(
            handed_out.contains(&SlotNumber(isolated.0 + 1)),
            "游标越过隔离的槽接着发：{handed_out:?}"
        );
    }

    fn pool_after_mkfs() -> PoolAllocator {
        let device_bytes = 4u64 << 30;
        let mut pool = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
            DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
        ]);
        pool.mark_format_time_units(
            Placement {
                slot: SlotNumber(50176),
                span: 2,
            },
            Placement {
                slot: SlotNumber(50178),
                span: 1,
            },
        );
        pool
    }

    #[test]
    fn unit_area_of_a_4_gib_image_has_211968_slots_in_3312_segments() {
        let map = DeviceFreeMap::new(DeviceIdentity(0), 4 << 30);
        assert_eq!(map.unit_area_slots(), 211_968);
        assert_eq!(map.empty_segments(), 3312);
        assert_eq!(map.free_runs(), 1);
    }

    #[test]
    fn first_transaction_placements_follow_the_byte_table() {
        let mut pool = pool_after_mkfs();
        assert_eq!(
            pool.devices[0].free_runs(),
            1,
            "mkfs 占了单元区开头，空闲仍是一段"
        );
        assert_eq!(pool.devices[0].empty_segments(), 3311);
        let data = pool.allocate_user_data(CheckpointTxg(3)).expect("有空槽");
        assert_eq!(
            data,
            Placement {
                slot: SlotNumber(50180),
                span: 2
            },
            "t1@50180：最低的偶数空槽对"
        );
        assert_eq!(
            pool.records().len(),
            6,
            "mkfs 两个单元 × 2 盘（分配代 0）+ t1 × 2 盘"
        );
        assert_eq!(pool.policy_mismatches, 0);
        let extent_root = pool
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
            .expect("开放段");
        assert_eq!(
            extent_root.slot,
            SlotNumber(50240),
            "开放段 = 最低的 64 槽对齐全空段"
        );
        let inode_leaf = pool
            .allocate_commit_generated(UnitFootprint::TwoSlotsAligned, CheckpointTxg(3))
            .expect("开放段");
        assert_eq!(
            inode_leaf,
            Placement {
                slot: SlotNumber(50242),
                span: 2
            },
            "容器按 32768 对齐，槽 50241 空着"
        );
        for expected in [50244u64, 50245, 50246, 50247, 50248] {
            let node = pool
                .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
                .expect("开放段");
            assert_eq!(node.slot, SlotNumber(expected));
        }
        assert_eq!(
            pool.records().len(),
            20,
            "m1 m2 t1..t8 十个单元 × 2 盘（字节表五）"
        );
    }

    #[test]
    fn accounting_deltas_are_maintained_incrementally() {
        let mut pool = pool_after_mkfs();
        pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
        assert_eq!(pool.devices[0].allocated_slots(), 5, "mkfs 3 + t1 2");
        assert_eq!(pool.devices[0].free_slots(), 211_968 - 5);
        assert_eq!(pool.devices[0].free_runs(), 2, "[50179] 与 [50182, 末]");
        assert_eq!(pool.devices[0].empty_segments(), 3311);
        for footprint in [
            UnitFootprint::OneSlot,
            UnitFootprint::TwoSlotsAligned,
            UnitFootprint::OneSlot,
            UnitFootprint::OneSlot,
            UnitFootprint::OneSlot,
            UnitFootprint::OneSlot,
            UnitFootprint::OneSlot,
        ] {
            pool.allocate_commit_generated(footprint, CheckpointTxg(3))
                .expect("开放段");
        }
        assert_eq!(pool.devices[0].allocated_slots(), 13, "字节表五：13 个槽");
        assert_eq!(pool.devices[0].free_slots() * SLOT_BYTES, 3_472_670_720);
        assert_eq!(
            pool.devices[0].free_runs(),
            4,
            "[50179]、[50182, 50239]、[50241]、[50249, 末]"
        );
        assert_eq!(pool.devices[0].empty_segments(), 3310);
        assert_eq!(pool.devices[1].free_runs(), 4, "两盘同构");
    }

    #[test]
    fn releasing_a_placement_keeps_the_slots_occupied_and_moves_them_into_the_defer_queue() {
        let mut pool = pool_after_mkfs();
        let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
        pool.release(data, CheckpointTxg(4));
        assert_eq!(pool.devices[0].allocated_slots(), 5, "占着的槽数不变");
        assert_eq!(pool.devices[0].deferred_slots(), 2, "两槽进了 defer 队列");
        assert_eq!(pool.devices[0].free_slots(), 211_968 - 5, "空闲不变");
        let released: Vec<&AllocationRecord> = pool
            .records()
            .iter()
            .filter(|record| record.slot == data.slot)
            .collect();
        assert_eq!(released.len(), 2, "每盘一条");
        for record in released {
            assert!(record.is_released);
            assert_eq!(record.generation, CheckpointTxg(4), "释放代");
            assert_eq!(record.span_slots, 2, "跨度值不含标志位");
            let parsed = AllocationRecord::parse(&record.to_bytes());
            assert_eq!(parsed, *record, "标志位进跨度段最高位再读回来");
            assert_eq!(
                u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]),
                2 | ALLOCATION_RECORD_RELEASED_FLAG
            );
        }
        assert_eq!(
            pool.allocate_user_data(CheckpointTxg(4))
                .expect("有空槽")
                .slot,
            SlotNumber(50182),
            "已释放的 50180 不许再发，下一个偶数空槽对是 50182"
        );
    }

    /// 重建时认出还没被换下的第 0 版树表单元（分配代 0、跨度 1、未释放）；换下之后（已释放）就不认。
    #[test]
    fn rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released() {
        let device_bytes = 4_u64 * 1024 * 1024 * 1024;
        let records = |released: bool| {
            vec![
                AllocationRecord {
                    device: DeviceIdentity(0),
                    slot: SlotNumber(50176),
                    span_slots: 2,
                    generation: CheckpointTxg(0),
                    is_released: false,
                },
                AllocationRecord {
                    device: DeviceIdentity(0),
                    slot: SlotNumber(50178),
                    span_slots: 1,
                    generation: if released {
                        CheckpointTxg(3)
                    } else {
                        CheckpointTxg(0)
                    },
                    is_released: released,
                },
            ]
        };
        let fresh = PoolAllocator::rebuild_from_records(
            vec![DeviceFreeMap::new(DeviceIdentity(0), device_bytes)],
            records(false),
        );
        assert_eq!(
            fresh.format_time_tree_table(),
            Some(Placement {
                slot: SlotNumber(50178),
                span: 1
            })
        );
        let replaced = PoolAllocator::rebuild_from_records(
            vec![DeviceFreeMap::new(DeviceIdentity(0), device_bytes)],
            records(true),
        );
        assert_eq!(replaced.format_time_tree_table(), None);
    }

    /// 两块盘上各一份同样的记录。
    fn records_on_both_devices(placements: &[(u64, u16, u64, bool)]) -> Vec<AllocationRecord> {
        [DeviceIdentity(0), DeviceIdentity(1)]
            .iter()
            .flat_map(|device| {
                placements
                    .iter()
                    .map(
                        |(slot, span_slots, generation, is_released)| AllocationRecord {
                            device: *device,
                            slot: SlotNumber(*slot),
                            span_slots: *span_slots,
                            generation: CheckpointTxg(*generation),
                            is_released: *is_released,
                        },
                    )
            })
            .collect()
    }

    fn rebuilt_pool(records: Vec<AllocationRecord>) -> PoolAllocator {
        let device_bytes = 4u64 << 30;
        PoolAllocator::rebuild_from_records(
            vec![
                DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
                DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
            ],
            records,
        )
    }

    /// 某块盘上的记录按槽号排好：(槽, 跨度, 代, 已释放)。
    fn records_on_device(
        pool: &PoolAllocator,
        device: DeviceIdentity,
    ) -> Vec<(u64, u16, u64, bool)> {
        let mut on_device: Vec<(u64, u16, u64, bool)> = pool
            .records()
            .iter()
            .filter(|record| record.device == device)
            .map(|record| {
                (
                    record.slot.0,
                    record.span_slots,
                    record.generation.0,
                    record.is_released,
                )
            })
            .collect();
        on_device.sort_unstable();
        on_device
    }

    /// 代码三方第二轮 Z1-d 的形状：用户数据（两槽）落在一对都已回收的单槽记录上——50178 那条改写成这次的分配、跨度 2，
    /// 50179 那条罩住的槽被新记录罩住，随这次分配删掉；之后按这份记录重建分配器（下一次可写挂载做的事）不 panic、占着的槽数对得上。
    #[test]
    fn reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_rewrites_the_first_and_deletes_the_second(
    ) {
        let mut pool = rebuilt_pool(records_on_both_devices(&[
            (50176, 2, 0, false),
            (50178, 1, 3, true),
            (50179, 1, 4, true),
        ]));
        assert_eq!(
            pool.reclaim_released_up_to(CheckpointTxg(4), ReclaimedReuse::Immediately),
            vec![
                Placement {
                    slot: SlotNumber(50178),
                    span: 1
                },
                Placement {
                    slot: SlotNumber(50179),
                    span: 1
                },
            ]
        );
        let data = pool
            .try_allocate_user_data(CheckpointTxg(5))
            .expect("有空槽");
        assert_eq!(
            data,
            Placement {
                slot: SlotNumber(50178),
                span: 2
            },
            "最低的偶数空槽对是两条已回收记录罩住的 50178–50179"
        );
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            assert_eq!(
                records_on_device(&pool, device),
                vec![(50176, 2, 0, false), (50178, 2, 5, false)],
                "盘 {device:?}：50178 那条改写成跨 2、代 5，50179 那条删掉"
            );
            assert!(
                !pool.reclaimed.contains(&(device, SlotNumber(50179))),
                "删掉的记录也不再算已回收"
            );
        }
        let rebuilt = rebuilt_pool(pool.records().to_vec());
        assert_eq!(
            rebuilt.devices[0].allocated_slots(),
            4,
            "50176–50179 四个槽"
        );
        assert_eq!(rebuilt.devices[0].deferred_slots(), 0);
    }

    /// 反过来的形状：一个单槽落点落在一条已回收的两槽记录的第二个槽上——那条记录起点不是这个槽，罩住的槽被新落点罩住，删掉；
    /// 它罩住的另一个槽（50180）之后没有记录，与从没分配过的槽一样空着。
    #[test]
    fn one_slot_placement_inside_reclaimed_two_slot_record_deletes_that_record() {
        let mut pool = rebuilt_pool(records_on_both_devices(&[
            (50176, 2, 0, false),
            (50178, 1, 0, false),
            (50180, 2, 3, true),
        ]));
        pool.reclaim_released_up_to(CheckpointTxg(3), ReclaimedReuse::Immediately);
        pool.record(
            Placement {
                slot: SlotNumber(50181),
                span: 1,
            },
            CheckpointTxg(4),
        );
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            assert_eq!(
                records_on_device(&pool, device),
                vec![
                    (50176, 2, 0, false),
                    (50178, 1, 0, false),
                    (50181, 1, 4, false)
                ],
                "盘 {device:?}：50180 那条删掉、50181 追加一条"
            );
        }
        assert!(pool.devices[0].is_free(SlotNumber(50180)), "50180 仍空着");
        let rebuilt = rebuilt_pool(pool.records().to_vec());
        assert_eq!(
            rebuilt.devices[0].allocated_slots(),
            4,
            "50176–50178 与 50181"
        );
    }

    /// 新落点罩住一条没回收的记录（已释放、还在 defer 窗口里）：位图与记录分叉了，在记录这一层就断言，不等到位图的断言。
    /// 不用 `should_panic`：门禁 59 号按「test 名 ... FAILED」认红，`should_panic` 那一行多一段「- should panic」认不出。
    #[test]
    fn placement_over_record_that_was_not_reclaimed_is_asserted_on_the_records() {
        let mut pool = rebuilt_pool(records_on_both_devices(&[
            (50176, 2, 0, false),
            (50178, 1, 0, false),
            (50180, 2, 3, true),
        ]));
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            pool.record(
                Placement {
                    slot: SlotNumber(50181),
                    span: 1,
                },
                CheckpointTxg(4),
            );
        }));
        let payload = outcome.expect_err("新落点罩住一条没回收的记录，要断言");
        let message = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                payload
                    .downcast_ref::<&str>()
                    .map(|text| (*text).to_string())
            })
            .unwrap_or_default();
        assert!(
            message.contains("不是已回收的"),
            "断言在记录这一层（不是位图那一条）：{message}"
        );
    }

    /// 每块盘上把 `[from, to)` 里还空着的槽都占掉（只动空闲图，不写分配记录：造形态用）。
    fn fill_free_slots(pool: &mut PoolAllocator, from: u64, to: u64) {
        for device_map in &mut pool.devices {
            for slot in from..to {
                if device_map.is_free(SlotNumber(slot)) {
                    device_map.mark_allocated(SlotNumber(slot), 1);
                }
            }
        }
    }

    /// 增补 2 第 20c 行（代码三方第一轮打中）：回落把开放段置空之后，那一段仍是聚簇段、仍不给用户数据
    /// （D3（空间分配） 已定项 8 第 2 条「聚簇段只给提交内生块」；已定项 10 ② 的量词是「任何开放的聚簇段」）。
    /// 形态：开段 [50240, 50304) 发第一个事务那七个提交内生块 ⇒ 游标停在 50249；段里游标前方的槽占满、单元区里一个全空段都不剩
    /// ⇒ 下一个提交内生块回落、`open_segment` 置空；再把段里那个两槽容器释放、回收（游标不回头，bump 够不着它）——
    /// 段里于是有一对空的偶数槽 50242–50243，而且它是全池最低的一对，段里 50244–50248 还躺着活着的内生块。
    /// 这时要一个用户数据落点：它必须落在段外面。
    #[test]
    fn user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed() {
        let mut pool = pool_after_mkfs();
        let extent_root = pool
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
            .expect("开段");
        assert_eq!(extent_root.slot, SlotNumber(50240));
        let inode_leaf = pool
            .allocate_commit_generated(UnitFootprint::TwoSlotsAligned, CheckpointTxg(3))
            .expect("开放段");
        assert_eq!(
            inode_leaf,
            Placement {
                slot: SlotNumber(50242),
                span: 2
            }
        );
        for expected in [50244u64, 50245, 50246, 50247, 50248] {
            assert_eq!(
                pool.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
                    .expect("开放段")
                    .slot,
                SlotNumber(expected)
            );
        }
        assert_eq!(
            pool.cluster_segments(),
            &BTreeSet::from([SlotNumber(50240)])
        );
        // 段里游标前方占满、别的段各占掉段首一槽 ⇒ 没有全空段；段外面（第三段起）还有大片空槽。
        fill_free_slots(&mut pool, 50249, 50304);
        for device_map in &mut pool.devices {
            let full_segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
            for segment_index in 0..full_segments {
                if device_map.segment_is_empty(usize::try_from(segment_index).expect("段号")) {
                    device_map.mark_allocated(
                        SlotNumber(UNIT_AREA_START_SLOT + segment_index * CLUSTER_SEGMENT_SLOTS),
                        1,
                    );
                }
            }
        }
        let fell_back = pool
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
            .expect("没有全空段、段外面还有空槽：回落");
        assert_eq!(
            pool.open_segment(),
            None,
            "回落之后没有段可 bump（回落落点 {:?}）",
            fell_back.slot
        );
        assert_eq!(
            pool.cluster_segments(),
            &BTreeSet::from([SlotNumber(50240)]),
            "回落不把那一段从聚簇段里去掉：它还装着这次提交的内生块"
        );
        // 段里那个两槽容器被换下、释放、回收：50242–50243 空出来，游标（50249）不回头，bump 够不着它们。
        pool.release(inode_leaf, CheckpointTxg(4));
        assert_eq!(
            pool.reclaim_released_up_to(CheckpointTxg(4), ReclaimedReuse::Immediately),
            vec![inode_leaf]
        );
        // 段外面最低的偶数空槽对：50176–50239 与 [50240, 50304) 之外，第三段段首 50304 占着 ⇒ 50306–50307。
        fill_free_slots(&mut pool, UNIT_AREA_START_SLOT, 50240);
        for device_map in &pool.devices {
            assert!(
                device_map.is_free(SlotNumber(50242)) && device_map.is_free(SlotNumber(50243)),
                "盘 {:?}：段里那一对空着，而且是全池最低的一对",
                device_map.device
            );
        }
        let user_data = pool
            .try_allocate_user_data(CheckpointTxg(5))
            .expect("段外面有空槽对");
        assert_eq!(
            user_data,
            Placement {
                slot: SlotNumber(50306),
                span: 2
            },
            "用户数据要落在聚簇段外面：段里空着的 50242–50243 与它旁边活着的内生块 50244–50248 同一段"
        );
    }
}
