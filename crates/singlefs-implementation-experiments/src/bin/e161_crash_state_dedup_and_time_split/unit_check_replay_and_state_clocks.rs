//! 单元级检查重放（G3 的 Q3f、Q3g）、分区计时与键的算法。

use super::*;

// ───────────────────────────── 单元级检查重放（G3 的 Q3f、Q3g） ─────────────────────────────

/// U(字节)：指针里的整单元校验和那一道，再按字节 6 的类标签走 checker 的公开检查（第六节 G3）；结果过 `black_box`。
pub(super) fn replay_unit_check(bytes: &[u8]) {
    black_box(singlefs_checker::crc32_castagnoli_table(bytes));
    match bytes.get(6).copied() {
        Some(2) => {
            let view = singlefs_checker::index_node_view(bytes);
            if let Ok(view) = &view {
                let schema = match view.key_width {
                    LOCAL_KEY_WIDTH_EXTENT => Some(singlefs_checker::KEY_SCHEMA_EXTENT),
                    LOCAL_KEY_WIDTH_INODE => Some(singlefs_checker::KEY_SCHEMA_INODE),
                    LOCAL_KEY_WIDTH_ALLOCATION => Some(singlefs_checker::KEY_SCHEMA_ALLOCATION),
                    LOCAL_KEY_WIDTH_ACCOUNTING => Some(singlefs_checker::KEY_SCHEMA_ACCOUNTING),
                    LOCAL_KEY_WIDTH_MAPPING => Some(singlefs_checker::KEY_SCHEMA_MAPPING),
                    _other_width => None,
                };
                if let Some(schema) = schema {
                    if view.level == 0 {
                        black_box(singlefs_checker::check_index_node_keys(view, schema).is_ok());
                    } else {
                        black_box(
                            singlefs_checker::check_internal_node_separators(view, schema).is_ok(),
                        );
                    }
                }
            }
            black_box(view.is_ok());
        }
        Some(3) => {
            black_box(singlefs_checker::packed_unit_view(bytes).is_ok());
        }
        Some(_) | None => {
            black_box(singlefs_checker::check_unit(bytes).is_ok());
        }
    }
}

/// G3 的六个计时区（五段 + 单元级重放），PC3 在其中一个区里加忙等。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TimingZone {
    RecoveryConsult,
    RecoveryIgnore,
    Oracle,
    PoolChecker,
    RecordChecker,
    UnitCheckReplay,
}

pub(super) const TIMING_ZONES: [TimingZone; 6] = [
    TimingZone::RecoveryConsult,
    TimingZone::RecoveryIgnore,
    TimingZone::Oracle,
    TimingZone::PoolChecker,
    TimingZone::RecordChecker,
    TimingZone::UnitCheckReplay,
];

impl TimingZone {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RecoveryConsult => "recovery_consult",
            Self::RecoveryIgnore => "recovery_ignore",
            Self::Oracle => "oracle",
            Self::PoolChecker => "pool_checker",
            Self::RecordChecker => "record_checker",
            Self::UnitCheckReplay => "unit_check_replay",
        }
    }
}

/// 计时读哪一种钟。挂钟量真实耗时，产物与 PC3 用它；计数钟不读机器时间，只在忙等那一处往前拨 `TIMING_CONTROL_BUSY_WAIT`，
/// 量出来的只是忙等记到了哪一区，与机器负载无关（单测拿它判「计时记到哪一区」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ZoneClock {
    Wall,
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "计数钟只给单测判计时记到哪一区，产物那几格读挂钟")
    )]
    Counted,
}

/// 一趟的计时控制：忙等加在哪一区（不加是 `None`）与读哪一种钟。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TimingControl {
    pub(super) busy_zone: Option<TimingZone>,
    pub(super) clock: ZoneClock,
}

impl TimingControl {
    /// 产物那几格：读挂钟、不加忙等。
    pub(super) const WALL_CLOCK_WITHOUT_BUSY_WAIT: Self = Self {
        busy_zone: None,
        clock: ZoneClock::Wall,
    };
}

/// 一次读钟：挂钟那一刻与计数钟此刻的纳秒数都记下，量一段时按钟的种类取其中一样。
#[derive(Clone, Copy, Debug)]
pub(super) struct ClockReading {
    wall: Instant,
    counted_nanoseconds: u128,
}

/// 一个函数里量各区用的钟：每个量时的函数各起一份，量的都是同一个函数里的一段；计数钟的读数是这一份钟上忙等拨过的纳秒。
pub(super) struct StateClock {
    pub(super) clock: ZoneClock,
    counted_nanoseconds: u128,
}

impl StateClock {
    pub(super) fn new(clock: ZoneClock) -> Self {
        Self {
            clock,
            counted_nanoseconds: 0,
        }
    }

    pub(super) fn now(&self) -> ClockReading {
        ClockReading {
            wall: Instant::now(),
            counted_nanoseconds: self.counted_nanoseconds,
        }
    }

    pub(super) fn nanoseconds_since(&self, started: ClockReading) -> u128 {
        match self.clock {
            ZoneClock::Wall => started.wall.elapsed().as_nanos(),
            ZoneClock::Counted => self.counted_nanoseconds - started.counted_nanoseconds,
        }
    }

    /// 忙等区是 `zone` 时加一段忙等：挂钟真等 `TIMING_CONTROL_BUSY_WAIT`；计数钟往前拨这么多纳秒，不等。
    pub(super) fn busy_wait_if(&mut self, zone: TimingZone, busy_zone: Option<TimingZone>) {
        if busy_zone != Some(zone) {
            return;
        }
        match self.clock {
            ZoneClock::Wall => {
                let started = Instant::now();
                while started.elapsed() < TIMING_CONTROL_BUSY_WAIT {
                    black_box(started);
                }
            }
            ZoneClock::Counted => {
                self.counted_nanoseconds += TIMING_CONTROL_BUSY_WAIT.as_nanos();
            }
        }
    }
}

/// 一个状态的计时（纳秒）。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct StateTimings {
    pub(super) state_nanoseconds: u128,
    pub(super) recovery_consult_nanoseconds: u128,
    pub(super) recovery_ignore_nanoseconds: u128,
    pub(super) oracle_nanoseconds: u128,
    pub(super) pool_checker_nanoseconds: u128,
    pub(super) record_checker_nanoseconds: u128,
    pub(super) unit_positions_replay_nanoseconds: u128,
    pub(super) unit_events_replay_nanoseconds: u128,
    pub(super) walk_checker_nanoseconds: u128,
}

impl StateTimings {
    pub(super) fn zone_nanoseconds(&self, zone: TimingZone) -> u128 {
        match zone {
            TimingZone::RecoveryConsult => self.recovery_consult_nanoseconds,
            TimingZone::RecoveryIgnore => self.recovery_ignore_nanoseconds,
            TimingZone::Oracle => self.oracle_nanoseconds,
            TimingZone::PoolChecker => self.pool_checker_nanoseconds,
            TimingZone::RecordChecker => self.record_checker_nanoseconds,
            TimingZone::UnitCheckReplay => self.unit_positions_replay_nanoseconds,
        }
    }

    pub(super) fn five_segments_nanoseconds(&self) -> u128 {
        self.recovery_consult_nanoseconds
            + self.recovery_ignore_nanoseconds
            + self.oracle_nanoseconds
            + self.pool_checker_nanoseconds
            + self.record_checker_nanoseconds
    }
}

/// 读集按次序算键；集合（位置 + 内容）排序去重之后算键。
pub(super) fn sequence_key(calls: &[LoggedCall]) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_KEY_SEQUENCE);
    builder.feed_word(u64::try_from(calls.len()).expect("个数"));
    for call in calls {
        builder.feed_digest(call.digest);
    }
    builder.finish()
}

/// K4-walk、K4-units 的集合：（位置摘要，位置 + 内容的摘要，位置原样），按位置排序去重。
pub(super) type SetElements = BTreeSet<(Digest128, Digest128, PositionDescription)>;

pub(super) fn set_key(root: Digest128, elements: &SetElements) -> Digest128 {
    set_key_of(
        root,
        elements.len(),
        elements
            .iter()
            .map(|(_position, element, _description)| *element),
    )
}

pub(super) fn set_key_of(
    root: Digest128,
    element_count: usize,
    elements: impl Iterator<Item = Digest128>,
) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_KEY_SET);
    builder.feed_digest(root);
    builder.feed_word(u64::try_from(element_count).expect("个数"));
    for element in elements {
        builder.feed_digest(element);
    }
    builder.finish()
}

pub(super) fn pair_digest(left: Digest128, right: Digest128) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_PAIR);
    builder.feed_digest(left);
    builder.feed_digest(right);
    builder.finish()
}

/// 单元区里的读的集合元素：（位置摘要，位置 + 内容的摘要）。读不到的也进集合（答案是「读不到」）。
pub(super) fn unit_area_elements(reads: &[LoggedRead]) -> SetElements {
    reads
        .iter()
        .filter(|read| read.offset >= UNIT_AREA_START_BYTES)
        .map(|read| {
            let arguments = [
                u64::from(read.device),
                read.offset,
                u64::try_from(read.length).expect("读长"),
            ];
            let position = position_digest(CallKind::ImageRead, &arguments);
            (
                position,
                call_digest(position, read.content.as_ref().map(|(digest, _)| *digest)),
                PositionDescription::of(CallKind::ImageRead, &arguments),
            )
        })
        .collect()
}

/// G4 的「所选根」：checker 自己的择法（`chosen_system_configurations` 取第一块盘的几何、`valid_roots` 按 (txg, 实例) 取最大），
/// 记（实例、txg、根记录字节）的摘要；择不到记一个固定摘要。
pub(super) fn chosen_root_digest(image: &CrashImage<'_>) -> Digest128 {
    let configurations = chosen_system_configurations(image);
    let Some((_view, geometry)) = configurations
        .into_iter()
        .find_map(|(_device, chosen)| chosen)
    else {
        return digest_of_words(&[0]);
    };
    let roots = valid_roots(image, &geometry);
    let Some((_region, _slot, root)) = roots
        .into_iter()
        .max_by_key(|(_, _, root)| (root.checkpoint_txg, root.instance))
    else {
        return digest_of_words(&[1]);
    };
    let mut builder = DigestBuilder::new(DIGEST_TAG_CONTENT);
    builder.feed_word(u64::from(root.instance));
    builder.feed_word(root.checkpoint_txg);
    builder.feed_bytes(&root.record_bytes);
    builder.finish()
}

#[cfg(test)]
mod tests;
