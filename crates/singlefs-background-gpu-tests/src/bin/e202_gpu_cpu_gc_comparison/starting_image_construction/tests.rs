use super::*;

/// 改前（各存各的）与改后（同内容块共用、零块不存）两种存法照同一串改动造同一个池：发成的版数、抬 F、几何读数、碎片段、全空段数、
/// 映射条数逐项相同，两块盘上每一个块读回的字节逐个相同；共用的那一份占的块字节少于各存各的。
#[test]
fn the_two_chunk_storages_build_the_same_pool_from_the_same_changes() {
    let build = |storage: ChunkStorage| {
        let mut progress_dropped = |_line: &str| {};
        let mut clock = ConstructionClock::new(
            format!("对拍-{}", storage.name()),
            CONSTRUCTION_PROGRESS_INTERVAL,
            &mut progress_dropped,
        );
        smallest_pool_on_the_clock(storage, &mut clock)
    };
    let copied = build(ChunkStorage::EachDeviceCopiesItsChunks);
    let shared = build(ChunkStorage::SharedWhenTheBytesAreEqual);
    let numbers = |image: &StartingImage| {
        let construction = &image.construction;
        (
            construction.publishes,
            construction.floor_raises,
            construction.release_rounds,
            construction.spacer_units,
            construction.alignment_spacer_units,
            construction.live_data_units_found,
            construction.empty_segments,
            construction.mapped_units,
        )
    };
    assert_eq!(numbers(&copied), numbers(&shared));
    assert_eq!(copied.construction.geometry, shared.construction.geometry);
    assert_eq!(
        copied.fragment_segment_starts,
        shared.fragment_segment_starts
    );
    assert_eq!(copied.lowest_live_data_inode, shared.lowest_live_data_inode);
    for ((copied_identity, copied_device), (shared_identity, shared_device)) in
        copied.devices.iter().zip(&shared.devices)
    {
        assert_eq!(copied_identity, shared_identity);
        let chunk_indexes: BTreeSet<u64> = copied_device
            .chunks
            .keys()
            .chain(shared_device.chunks.keys())
            .copied()
            .collect();
        for chunk_index in chunk_indexes {
            assert!(
                read_a_chunk(copied_device, chunk_index)
                    == read_a_chunk(shared_device, chunk_index),
                "盘 {copied_identity:?} 块 {chunk_index} 两种存法读回的字节不同"
            );
        }
    }
    assert!(
        chunk_bytes_of_the_pool(&shared.devices).stored
            < chunk_bytes_of_the_pool(&copied.devices).stored
    );
}

/// 最小池（P = 64，a 是这池上量到的补齐单元数）照登记第十二节修订第 14 条造：每次数据写类调用写满一个文件，16 个单元的 3P ÷ 32 = 6 次、
/// a > 0 时补齐文件 a 个单元的 1 次；造数据挂载四个阶段发成的版数逐个等于登记的算式（建 inode 1、顺序写 2、补齐 a > 0 时 1、交错 4）；
/// 两种造池结果行都带的读数里常驻内存的峰读到了、各次数据写的单元数照记的打。
#[test]
fn every_data_write_of_the_smallest_pool_writes_a_whole_file_in_one_publish() {
    let image = {
        let mut progress_dropped = |_line: &str| {};
        let mut clock = ConstructionClock::new(
            "整文件".to_string(),
            CONSTRUCTION_PROGRESS_INTERVAL,
            &mut progress_dropped,
        );
        smallest_pool_on_the_clock(ChunkStorage::SharedWhenTheBytesAreEqual, &mut clock)
    };
    let construction = &image.construction;
    let alignment_spacer_units = construction.alignment_spacer_units;
    eprintln!("E202 最小池补齐单元数 a = {alignment_spacer_units}");
    assert!(alignment_spacer_units < data_units_of_a_cluster_segment());
    let mut expected_data_writes = BTreeMap::from([(LIVE_DATA_UNITS_PER_FILE, 6)]);
    if alignment_spacer_units > 0 {
        *expected_data_writes
            .entry(alignment_spacer_units)
            .or_insert(0) += 1;
    }
    assert_eq!(
        construction.data_writes_by_units, expected_data_writes,
        "{construction:?}"
    );
    let registered = RegisteredPublishesOfTheDataMount {
        creating_the_inodes: 1,
        sequential_half: 2,
        alignment: u64::from(alignment_spacer_units > 0),
        interleaved_half: 4,
    };
    assert_eq!(
        construction.registered_publishes_of_the_data_mount,
        registered
    );
    for timing in &construction.phases {
        if let Some(registered_publishes) = registered.publishes_of_the_phase(timing.phase) {
            assert_eq!(
                timing.publishes,
                registered_publishes,
                "{}",
                timing.phase.name()
            );
        }
    }
    assert_eq!(
        construction.live_data_units_found,
        LIVE_DATA_UNITS_OF_THE_SMALLEST_POOL
    );
    let peak = construction
        .peak_resident_bytes_of_the_process
        .expect("造完那一刻读得到本进程常驻内存的峰");
    assert!(peak > 0);
    let fields = whole_file_construction_fields(construction);
    let data_writes: Vec<String> = expected_data_writes
        .iter()
        .map(|(units, writes)| format!("{units}:{writes}"))
        .collect();
    assert!(
        fields.starts_with(&format!(
            "peak_resident_bytes_of_the_process={peak} data_writes_by_units={} registered_publishes_of_the_data_mount={} ",
            data_writes.join(","),
            registered.total()
        )),
        "{fields}"
    );
}
