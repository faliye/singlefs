    /// 探针（正推腿自己的，不入库）：量 `isolate_reclaimed_slots_the_isolation_still_protects` 与 `placements_the_isolation_still_protects` 在 C 那一槽装着 N 个落点时的耗时。
    #[test]
    fn probe_cost_of_the_protected_list_scan() {
        for (protected, reclaimed_count) in [(10_000u64, 1_000u64), (40_000, 1_000), (40_000, 4_000)] {
            let device_bytes = (protected * 4 + 100_000) * singlefs_format::SLOT_BYTES;
            let mut allocator = PoolAllocator::new(vec![DeviceFreeMap::new(DeviceIdentity(0), device_bytes)]);
            let still_protected: Vec<PlacementOnDevice> = (0..protected)
                .map(|n| PlacementOnDevice { device: DeviceIdentity(0), slot: SlotNumber(UNIT_AREA_START_SLOT + 2 * n + 90_000), span: 2 })
                .collect();
            // 被回收的槽落在受护集合之外（最坏：每个都要把整张表扫完才知道不在里面）
            let reclaimed: Vec<AllocationRecord> = (0..reclaimed_count)
                .map(|n| AllocationRecord { device: DeviceIdentity(0), slot: SlotNumber(UNIT_AREA_START_SLOT + 4 * protected + 2 * n + 90_000), span_slots: 2, generation: CheckpointTxg(1), is_released: true })
                .collect();
            let started = std::time::Instant::now();
            allocator.isolate_reclaimed_slots_the_isolation_still_protects(&reclaimed, &still_protected);
            eprintln!("PROBE-COST protected_placements={protected} reclaimed_records={reclaimed_count} elapsed_ms={}", started.elapsed().as_millis());
        }
    }

