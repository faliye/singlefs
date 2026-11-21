use super::*;

// ───────────────────────────── 5.2 阳性对照（keys 子命令里对种子下标 0 的随机历史、覆盖写链、冒烟层各条流跑） ─────────────────────────────

/// 一条对照的结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ControlReading {
    pub(super) kind: &'static str,
    pub(super) flow: String,
    pub(super) passed: bool,
    pub(super) detail: String,
}

impl ControlReading {
    pub(super) fn body(&self) -> String {
        format!(
            "name=control kind={} flow={} passed={} {}",
            self.kind, self.flow, self.passed, self.detail
        )
    }
}

pub(super) fn block_keys_of(blocks: &[BlockOfAPlan]) -> BTreeSet<VerdictBlockKey> {
    blocks.iter().map(|block| block.key.clone()).collect()
}

pub(super) fn reuse_keys_of(blocks: &[BlockOfAPlan]) -> BTreeSet<[u8; 32]> {
    blocks
        .iter()
        .map(|block| block.key.node_input_digest.0)
        .collect()
}

/// 候选的块里块键在 `against` 里的有几块、几个状态。
pub(super) fn shared_blocks_and_states(
    candidate: &[BlockOfAPlan],
    against: &BTreeSet<VerdictBlockKey>,
) -> (usize, u64) {
    candidate
        .iter()
        .filter(|block| against.contains(&block.key))
        .fold((0usize, 0u64), |(blocks, states), block| {
            (blocks + 1, states + block.states)
        })
}

/// 写表里有没有一次写碰到 `device` 上的这个扇区。
fn some_write_touches_the_sector(writes: &[RetainedWrite], device: u32, sector: u64) -> bool {
    writes.iter().any(|write| {
        write.device.0 == device
            && write.offset.0 / SECTOR_BYTES <= sector
            && sector < (write.offset.0 + write.contents.length_in_bytes()).div_ceil(SECTOR_BYTES)
    })
}

/// 扇区宽，崩溃点重放取 512 字节（D13 已定项 4）。
const SECTOR_BYTES: u64 = write_dump::SECTOR_BYTES_OF_THE_BASE_IMAGE;

/// 5.2 的对照只在种子下标 0 的随机历史、覆盖写链的每一档、冒烟层的每一条上跑（对照那段循环遍历每一族每一档，不拿第一条跑一下就算）。
pub(super) fn controls_apply_to(dump: &FlowDump) -> bool {
    dump.seed
        .is_none_or(|seed| seed == SEED_BASE_OF_THE_RANDOM_HISTORIES)
}

/// 基镜像里盘 0 上号最大的那个扇区被写成 512 个 0xFF 之后，与原流共用的块数必须恰好 0（计划的块数不变）。
fn base_sector_changed_control(
    dump: &FlowDump,
    base: &MemoryPool,
    full: &PlannedBlocks,
) -> Result<ControlReading, String> {
    let last_sector = base.device_size_in_bytes / SECTOR_BYTES - 1;
    let untouched = !base
        .devices
        .get(&DeviceIdentity(0))
        .is_some_and(|image| image.written_sectors().contains_key(&last_sector))
        && !some_write_touches_the_sector(&dump.writes, 0, last_sector);
    if !untouched {
        return Ok(ControlReading {
            kind: "base_sector_changed",
            flow: dump.label.clone(),
            passed: false,
            detail: format!(
                "盘 0 的扇区 {last_sector} 在基镜像或写表里被碰过，造不出改一扇区的对照"
            ),
        });
    }
    let mut changed = base.clone();
    changed.apply_writes(&[RetainedWrite {
        device: DeviceIdentity(0),
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(last_sector * SECTOR_BYTES),
        contents: WrittenContents::Bytes(vec![0xFF; 512]),
    }]);
    let changed_plan = plan_blocks(&changed, dump, Expansion::Full, default_item())?;
    let (shared_blocks, _shared_states) =
        shared_blocks_and_states(&changed_plan.blocks, &block_keys_of(&full.blocks));
    Ok(ControlReading {
        kind: "base_sector_changed",
        flow: dump.label.clone(),
        passed: shared_blocks == 0 && changed_plan.blocks.len() == full.blocks.len(),
        detail: format!(
            "changed_sector={last_sector} blocks={} blocks_after_the_change={} shared_blocks={shared_blocks} expected_shared_blocks=0",
            full.blocks.len(),
            changed_plan.blocks.len()
        ),
    })
}

/// 同一条流各按全量与甲二排计划：复用键两边同一批，块键共用的块恰好是不管任何段的点（末节点在内）的块。
fn two_expansions_control(
    label: &str,
    full: &PlannedBlocks,
    in_place_subsets: &PlannedBlocks,
) -> ControlReading {
    let (shared_blocks, _shared_states) =
        shared_blocks_and_states(&in_place_subsets.blocks, &block_keys_of(&full.blocks));
    let reuse_keys_equal = reuse_keys_of(&full.blocks) == reuse_keys_of(&in_place_subsets.blocks);
    let expected = full.blocks_of_points_without_segments;
    ControlReading {
        kind: "two_expansions",
        flow: label.to_string(),
        passed: reuse_keys_equal
            && shared_blocks == expected
            && in_place_subsets.blocks_of_points_without_segments == expected,
        detail: format!(
            "reuse_keys_equal={reuse_keys_equal} shared_blocks={shared_blocks} expected_shared_blocks={expected} blocks_full={} blocks_in_place_subsets={}",
            full.blocks.len(),
            in_place_subsets.blocks.len()
        ),
    }
}

/// 换项名与设点文件再排计划，块键逐个相同（D13 已定项 17「名字、路径不进键」）。
fn name_not_in_key_control(
    dump: &FlowDump,
    base: &MemoryPool,
    full: &PlannedBlocks,
) -> Result<ControlReading, String> {
    let renamed = plan_blocks(
        base,
        dump,
        Expansion::Full,
        CrashPointReplayItem {
            name: OTHER_ITEM_NAME_FOR_THE_NAME_CONTROL,
            setting_file: OTHER_SETTING_FILE_FOR_THE_NAME_CONTROL,
        },
    )?;
    Ok(ControlReading {
        kind: "name_not_in_key",
        flow: dump.label.clone(),
        passed: renamed.blocks == full.blocks,
        detail: format!(
            "blocks={} blocks_under_another_name={}",
            full.blocks.len(),
            renamed.blocks.len()
        ),
    })
}

/// 转储写成字节再读回，内容逐项相同（块键是转储内容的函数，读回相同则块键相同）。
fn dump_round_trip_control(dump: &FlowDump) -> ControlReading {
    let read_back = FlowDump::from_bytes(&dump.to_bytes());
    ControlReading {
        kind: "dump_round_trip",
        flow: dump.label.clone(),
        passed: read_back.as_ref().is_ok_and(|again| again == dump),
        detail: match read_back {
            Ok(_) => "read_back_equal_to_the_original".to_string(),
            Err(error) => format!("read_back_refused={error}"),
        },
    }
}

/// 一条流的全部对照。
pub(super) fn controls_of_the_flow(
    dump: &FlowDump,
    base: &MemoryPool,
    reading: &FlowReading,
) -> Result<Vec<ControlReading>, String> {
    let mut controls = vec![dump_round_trip_control(dump)];
    if let Some(full) = &reading.full {
        controls.push(base_sector_changed_control(dump, base, full)?);
        controls.push(name_not_in_key_control(dump, base, full)?);
        if let Some(in_place_subsets) = &reading.in_place_subsets {
            controls.push(two_expansions_control(&dump.label, full, in_place_subsets));
        }
    }
    Ok(controls)
}

#[cfg(test)]
mod tests;
