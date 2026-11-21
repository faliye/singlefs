use super::*;

fn key_of(reuse: u8, plan: u8, start: u64) -> VerdictBlockKey {
    VerdictBlockKey {
        node_input_digest: NodeInputDigest([reuse; 32]),
        enumeration_plan_hash: EnumerationPlanHash([plan; 32]),
        block_start: BlockStartStateIndex(start),
    }
}

pub(super) fn block_of(
    reuse: u8,
    plan: u8,
    start: u64,
    states: u64,
    point_index: usize,
) -> BlockOfAPlan {
    BlockOfAPlan {
        key: key_of(reuse, plan, start),
        states,
        point_index,
    }
}

pub(super) fn row_of(
    label: &str,
    family: &str,
    tier: &str,
    writes: usize,
    two_state: Option<u64>,
    longest_segment: usize,
) -> FlowRow {
    FlowRow {
        family: family.to_string(),
        tier: tier.to_string(),
        label: label.to_string(),
        seed: None,
        ending: "Completed".to_string(),
        writes,
        segments: 3,
        longest_segment,
        two_state,
        full: two_state,
        in_place_subsets: two_state,
        blocks_full: Some(3),
        blocks_in_place_subsets: Some(3),
        points: Some(2),
        keys_seconds: 1.5,
        two_state_agrees_with_crates: two_state.map(|_| true),
    }
}

pub(super) fn keys_file_of(row: FlowRow, blocks: Vec<BlockOfAPlan>) -> KeysFile {
    KeysFile {
        expansion: Expansion::Full,
        row,
        blocks,
    }
}

pub(super) fn line_with<'lines>(output: &'lines Output, prefix: &str) -> &'lines str {
    output
        .lines
        .iter()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("没有以 {prefix:?} 起头的行：{:?}", output.lines))
}

pub(super) fn tiny_write(device: u32, kind: StepKind, offset: u64, fill: u8) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(device),
        kind,
        is_force_unit_access: kind == StepKind::RootRecordFua,
        offset: DeviceOffsetInBytes(offset),
        contents: WrittenContents::Bytes(vec![fill; 512]),
    }
}

/// 三次写分两段（写 0、1 一段，写 2 一段），第二次写是第一条 journal 记录写。
pub(super) fn tiny_dump(label: &str, fill_of_the_record_write: u8) -> FlowDump {
    FlowDump {
        family: "random_history".to_string(),
        label: label.to_string(),
        tier: "candidate".to_string(),
        seed: Some(SEED_BASE_OF_THE_RANDOM_HISTORIES),
        ending: "Completed".to_string(),
        device_bytes: 1 << 20,
        device_identities: vec![0, 1],
        operations_of_the_make_filesystem: 1,
        base_writes: vec![tiny_write(0, StepKind::UnitWrite, 0, 0x11)],
        writes: vec![
            tiny_write(0, StepKind::UnitWrite, 4096, 1),
            tiny_write(1, StepKind::JournalRecord, 4096, fill_of_the_record_write),
            tiny_write(0, StepKind::RootRecordFua, 8192, 3),
        ],
        segments: vec![vec![0, 1], vec![2]],
        judged_root_index: Some(2),
        versions_with_a_file: Vec::new(),
        versions_without_a_file: Vec::new(),
        compaction_operations: None,
    }
}

/// 这条用例自己的临时目录，交回认领着它的守卫：用例做完、断言失败、panic 都删掉它。
pub(super) fn scratch_directory(
    name: &str,
) -> singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop {
    let directory = singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
        std::env::temp_dir().join(format!("e184-main-test-{}-{name}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&*directory);
    std::fs::create_dir_all(&*directory).expect("建临时目录");
    directory
}

pub(super) fn planned_blocks_of(
    blocks: Vec<BlockOfAPlan>,
    blocks_of_points_without_segments: usize,
) -> PlannedBlocks {
    let state_count = blocks.iter().map(|block| block.states).sum();
    PlannedBlocks {
        state_count,
        state_count_by_the_counting_function: state_count,
        point_count: blocks
            .iter()
            .map(|block| block.point_index)
            .max()
            .map_or(0, |last| last + 1),
        blocks,
        blocks_of_points_without_segments,
    }
}
