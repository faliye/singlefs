
    // ---- 攻方腿（代码欠账批甲簇代码轮，Opus）加的原型用例：只在冻结副本的拷贝里，不进仓 ----

    /// 同 [`allocator_after_the_shadow_ledger`]，另把 `deferred` 那几段的分配位置上、不进现行账的已分配记录
    /// （现行账里已释放、还在 defer 窗口里的槽就是这样：分配位置着，`current_records` 里那条是已释放）。
    fn opus_allocator_with_deferred_bits(
        pool: &FormattedPool,
        inputs: &ShadowLedgerInputs<'_>,
        deferred: &[(DeviceIdentity, SlotNumber, u64)],
    ) -> PoolAllocator {
        let mut allocator = PoolAllocator::new(
            pool.devices
                .iter()
                .map(|(identity, _)| {
                    DeviceFreeMap::with_unit_area_start(*identity, DEVICE_BYTES, pool.unit_area_start)
                })
                .collect(),
        );
        for record in inputs.current_records.iter().filter(|record| !record.is_released) {
            allocator
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == record.device)
                .expect("盘在池里")
                .mark_allocated(record.slot, u64::from(record.span_slots));
        }
        for (device, slot, span) in deferred {
            allocator
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == *device)
                .expect("盘在池里")
                .mark_allocated(*slot, *span);
        }
        let is_abandoned = |root: &RootRecord| inputs.abandoned_instances.contains(&root.instance);
        isolate_slots_referenced_only_by_abandoned_roots(
            &pool.devices,
            pool.unit_area_start,
            &mut allocator,
            inputs.roots,
            &is_abandoned,
            CheckpointTxg(0),
            inputs.current_records,
            inputs.placements_named_by_unapplied_journal_records,
            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
                inputs.recorded_placements_of_abandoned_roots,
            ),
            &mut BeforeTheOneReread::RereadImmediately,
            inputs.placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
        )
        .expect("不拒");
        allocator
    }

    /// K1 次序：起点槽 s 同时被候选根（mkfs 那一版的实例表 (s, 跨 2)）、一条根环表里记着的被抛弃根 (s, 跨 1) 与水位之上没施加的记录 (s, 跨 1) 引用，
    /// 现行账里没有它。被抛弃根那一圈豁免它（候选根引用）；C 那一圈只豁免现行账，要隔离它。先看豁免再判重：s 隔离；
    /// 先判重再看豁免：被抛弃根那一圈把 s 记进判重集合，C 那一圈判重跳过，s 发得出去。
    #[test]
    fn opus_k1_exempt_before_dedup_keeps_the_c_ring_isolating_a_slot_the_abandoned_ring_exempted() {
        let pool = formatted_pool();
        let start = pool.unit_area_start.slot();
        let root = abandoned_root(&pool, 1, 8);
        let recorded = BTreeMap::from([(
            (root.instance, root.checkpoint_txg),
            vec![placement_on_device_zero(start, 1)],
        )]);
        let allocator = allocator_after_the_shadow_ledger(
            &pool,
            &ShadowLedgerInputs {
                roots: &[pool.make_filesystem_root, root],
                abandoned_instances: &[root.instance],
                current_records: &[],
                recorded_placements_of_abandoned_roots: &recorded,
                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
                placements_named_by_unapplied_journal_records: &[placement_on_device_zero(start, 1)],
            },
        );
        let got = isolated_and_free_on_device_zero(&allocator, start);
        eprintln!("OPUS-K1-ORDER via=abandoned_ring slot_s isolated_and_free={got:?}");
        assert_eq!(got, (true, false), "C 那一圈要隔离 s");
    }

    /// 同上，豁免 s 的换成根槽这一遍读坏的被抛弃根那一圈（记着 (s, 跨 1)）。
    #[test]
    fn opus_k1_exempt_before_dedup_keeps_the_c_ring_isolating_a_slot_the_ring_slot_read_bad_ring_exempted() {
        let pool = formatted_pool();
        let start = pool.unit_area_start.slot();
        let allocator = allocator_after_the_shadow_ledger(
            &pool,
            &ShadowLedgerInputs {
                roots: &[pool.make_filesystem_root],
                abandoned_instances: &[],
                current_records: &[],
                recorded_placements_of_abandoned_roots: &BTreeMap::new(),
                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[
                    placement_on_device_zero(start, 1),
                ],
                placements_named_by_unapplied_journal_records: &[placement_on_device_zero(start, 1)],
            },
        );
        let got = isolated_and_free_on_device_zero(&allocator, start);
        eprintln!("OPUS-K1-ORDER via=ring_slot_read_bad slot_s isolated_and_free={got:?}");
        assert_eq!(got, (true, false), "C 那一圈要隔离 s");
    }

    /// K1 判重跨强弱两种调法：根环表里记着的被抛弃根 A1 走「只隔离分配位没置着的槽」，根环表里没记着、从盘上读账的被抛弃根 A2
    /// （mkfs 那一版：实例表 (s, 跨 2)、树表 (s + 2, 跨 1)，两块盘）走整段隔离。盘 0 上 s、s + 1 分配位置着（现行账里已释放、在 defer 窗口里）。
    /// 只打印两种读出次序下盘 0、盘 1 上 s 的隔离位，不判：看判重是不是让结局随读出次序变。
    #[test]
    fn opus_k1_dedup_across_the_weak_and_the_strong_isolation_by_read_order() {
        let pool = formatted_pool();
        let start = pool.unit_area_start.slot();
        let recorded_root = abandoned_root(&pool, 1, 8);
        let read_root = abandoned_root(&pool, 2, 12);
        let recorded = BTreeMap::from([(
            (recorded_root.instance, recorded_root.checkpoint_txg),
            vec![placement_on_device_zero(start, 2)],
        )]);
        for (label, roots) in [
            ("recorded_root_earlier", [recorded_root, read_root]),
            ("disk_read_root_earlier", [read_root, recorded_root]),
        ] {
            let allocator = opus_allocator_with_deferred_bits(
                &pool,
                &ShadowLedgerInputs {
                    roots: &roots,
                    abandoned_instances: &[recorded_root.instance, read_root.instance],
                    current_records: &[],
                    recorded_placements_of_abandoned_roots: &recorded,
                    placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
                    placements_named_by_unapplied_journal_records: &[],
                },
                &[(DeviceIdentity(0), start, 2)],
            );
            eprintln!(
                "OPUS-K1-STRENGTH order={label} dev0_s_isolated={} dev0_s1_isolated={} dev1_s_isolated={}",
                allocator.devices[0].is_isolated(start),
                allocator.devices[0].is_isolated(SlotNumber(start.0 + 1)),
                allocator.devices[1].is_isolated(start),
            );
        }
    }
