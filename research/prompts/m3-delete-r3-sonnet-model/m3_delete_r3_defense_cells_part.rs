
// =====================================================================================================================
// m3-delete-r3 辩方腿追加的用例（接在 m3-delete-r2 攻方原型文件后面，用它的私有辅助函数；副本专用，被攻过零轮）。
// 全部只打印 `R name=...` 行。原型没有意图记录、`deleted_inodes`、墓碑、谱系记录：它们的槽数在报告里按角色表另加，不在这里。
// 账房：`LEDGER_FOR_TESTS` 给 (标签 4, 标签 6)，两个都是按全部副本之和记的字节；这里的环窗口账 = (0, label6(…))。
// =====================================================================================================================

/// 每个活对象一份墓碑的预付，按记录摊（每盘字节）：⌈(2 槽容器 × (1 + 1/135 根摊)) × 16384 ÷ 815⌉ = 41。
const TOMB_BYTES_PER_LIVE_OBJECT_PER_DISK: u64 = 41;
/// 全池一份「开放墓碑容器 + 根」的整容器余量（每盘槽）：容器 2 + 根 1。
const TOMB_SLACK_SLOTS_PER_DISK: u64 = 3;
/// 分配记录树一片叶罩的槽数（`singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS`）与内部节点扇出。
const ALLOCATION_LEAF_SLOTS: u64 = 812;
const ALLOCATION_FANOUT: u64 = 169;

/// 分配记录树每一层（0 = 叶层）两块盘合起来罩到的位置数（K1 口径）：根层级 R = 最小的 r，使得第 r − 1 层的位置数合计 ≤ 内部节点扇出；
/// 返回层 0 … R − 1 的合计（根本身另算 1 个）。
fn allocation_level_counts(pool: &Pool) -> Vec<u64> {
    let per_device = |level: u32| -> u64 {
        let span = ALLOCATION_LEAF_SLOTS * ALLOCATION_FANOUT.pow(level);
        pool.allocator
            .devices
            .iter()
            .map(|map| {
                let start = map.unit_area_start().slot().0;
                let last = start + map.unit_area_slots() - 1;
                last / span - start / span + 1
            })
            .sum()
    };
    let mut counts = vec![per_device(0)];
    while *counts.last().expect("有") > ALLOCATION_FANOUT {
        let level = u32::try_from(counts.len()).expect("层数");
        counts.push(per_device(level));
    }
    counts
}

/// 一次删除类发布每盘新占槽数的上界（我提的，零轮）：普通分配 16（含意图那次的 `deleted_inodes` 根 1 + 墓碑容器 2 + 墓碑树根 1）
/// + 四个固定点根 4 + 映射树根以下 4 + 分配记录树根以下每一层 min(这一层的位置数, 2 × (K + 26))。
fn w_upper_bound(batch: u64, level_counts: &[u64]) -> u64 {
    16 + 4 + 4 + level_counts.iter().map(|count| (*count).min(2 * (batch + 26))).sum::<u64>()
}

fn window_slots(ring_roots: u64, batch: u64, level_counts: &[u64]) -> u64 {
    (ring_roots + 1) * w_upper_bound(batch, level_counts)
}

/// 标签 6（按全部副本之和记的字节）= 每盘 (窗口 + 整容器余量) 槽 + 每个活对象 41 字节，乘盘数（第一版每个单元落每块盘）。
fn label_six(live_objects: u64, window: u64) -> u64 {
    DEVICES * ((window + TOMB_SLACK_SLOTS_PER_DISK) * SLOT_BYTES + live_objects * TOMB_BYTES_PER_LIVE_OBJECT_PER_DISK)
}

fn available_of(allocator: &PoolAllocator, output: &TransactionOutput) -> Vec<i128> {
    admission_reading_before_a_publish(allocator, Some(output))
        .available_on_each_device()
        .into_iter()
        .map(|(_, available)| available.0)
        .collect()
}

/// 意图那次照辩方腿第二轮的做法：`keep = u64::MAX` = extent 与数据单元照抄、只把 inode 1 记录写成 nlink = 0；之后各批按 `batch` 从尾部截。
fn delete_with_the_intent_carried(
    pool: &mut Pool,
    batch: u64,
    before_each: &mut dyn FnMut(&str, u64, &Pool),
) -> Result<Vec<DeletePublish>, (String, PublishError)> {
    let size = pool.current.inode_record.size;
    let mut published = run_delete_publishes(pool, vec![("intent".to_owned(), u64::MAX, size)], before_each)?;
    published.extend(continue_the_delete(pool, batch, before_each)?);
    Ok(published)
}

fn counts_of(allocator: &PoolAllocator) -> String {
    per_device_counts(allocator)
}

/// G1：3 单元文件、K = 1 删除，每次发布之后的标签 4、6 与每盘可用。mkfs 同一个进程里做（分配器不回收），4 GiB × 2 盘、2 副本。
/// 先打印写入那两步（第一个文件、顺序写成 3 个单元）在环窗口账与零账下的每盘可用；再在「写完之后可用恰为 0」的池上删一遍：
/// ① 式子那一判开着（朴素）；② 式子那一判关、落点那一判开（环窗口的做法）；③ 同一个池、不关式子那一判但账里窗口已经是 label_six。
#[test]
fn g1_cells_three_units_k_one_under_the_ring_window() {
    let geometry = DEFAULT_GEOMETRY;
    let one = pool_with_a_file(1, geometry);
    let nodes = allocation_level_counts(&one);
    let window = window_slots(3 * geometry.ring_slots_per_region, PRODUCT_BATCH, &nodes);
    println!(
        "R name=geometry image_bytes={} ring_roots={} allocation_level_counts={nodes:?} w_upper_bound={} window_slots_per_disk={window} tomb_slack_slots_per_disk={TOMB_SLACK_SLOTS_PER_DISK} label6_one_live_object_all_replicas={}",
        geometry.image_bytes,
        3 * geometry.ring_slots_per_region,
        w_upper_bound(PRODUCT_BATCH, &nodes),
        label_six(1, window)
    );
    for (step, units) in [("after_first_file", 1u64), ("after_sequential_write", 3)] {
        let pool = pool_with_a_file(units, geometry);
        set_ledger(Some((0, label_six(1, window))));
        let with_window = available(&pool);
        set_ledger(Some((0, 0)));
        let zero_ledger = available(&pool);
        let reserve = reserve_slots(&pool);
        set_ledger(None);
        println!(
            "R name=cell step={step} units={units} live_objects=1 label4=0 label6_all_replicas={} available_with_window={with_window:?} available_zero_ledger={zero_ledger:?} reserve_slots={reserve:?} {}",
            label_six(1, window),
            counts_of(&pool.allocator)
        );
    }
    // 名义上的池（可用很大）：删除类发布不判式子那一判，标签 4 恒 0、标签 6 = 窗口 + 余量（活对象 0 个：意图那次起墓碑预付已转成真分配）。
    let mut pool = pool_with_a_file(3, geometry);
    set_ledger(Some((0, label_six(1, window))));
    println!("R name=nominal_before_the_delete available={:?} {}", available(&pool), counts_of(&pool.allocator));
    set_r3_batch_switches(true, false);
    let result = delete_with_the_intent_carried(&mut pool, 1, &mut |_, _, _| set_ledger(Some((0, label_six(0, window)))));
    set_r3_batch_switches(false, false);
    match &result {
        Ok(published) => {
            for publish in published {
                println!(
                    "R name=nominal_delete step={} keep={} txg={} new_slots={} ordinary={} fixed_point={} label4_row={} label6_row={} available={:?} {}",
                    publish.name,
                    publish.keep,
                    publish.shape.txg,
                    publish.shape.new_slots,
                    publish.shape.ordinary_slots,
                    publish.shape.fixed_point_slots,
                    accounting_row(&publish.output, singlefs_core::records::STATISTIC_PENDING_DELETE_BYTES),
                    accounting_row(&publish.output, singlefs_core::records::STATISTIC_COMMITTED_RESERVATION_BYTES),
                    available_of(&publish.allocator_after, &publish.output),
                    counts_of(&publish.allocator_after)
                );
            }
        }
        Err((name, error)) => println!("R name=nominal_delete result=Err({name}:{})", short_error(error)),
    }
    set_ledger(None);

    // 写完之后可用恰为 0 的池：窗口在账里、池填到可用 = 0。
    let mut full = pool_with_a_file(3, geometry);
    set_ledger(Some((0, label_six(1, window))));
    fill_until_available_is_at_most(&mut full, 0);
    let reserve0 = reserve_slots(&full)[0];
    println!(
        "R name=full_pool_before_the_delete available={:?} reserve_slots_dev0={reserve0} free_minus_reserve_slots_dev0={} {}",
        available(&full),
        i128::from(full.allocator.devices[0].free_slots()) - reserve0,
        counts_of(&full.allocator)
    );
    for (variant, skip_formula) in [("formula_judged_naive", false), ("formula_skipped_landing_judged", true)] {
        let mut pool = full.clone();
        set_ledger(Some((0, label_six(1, window))));
        set_r3_batch_switches(skip_formula, false);
        let mut headroom: Vec<i128> = Vec::new();
        let result = delete_with_the_intent_carried(&mut pool, 1, &mut |_, _, pool| {
            set_ledger(Some((0, label_six(0, window))));
            headroom.push(i128::from(pool.allocator.devices[0].free_slots()) - reserve0);
        });
        set_r3_batch_switches(false, false);
        match &result {
            Ok(published) => {
                let rows: Vec<String> = published
                    .iter()
                    .map(|publish| {
                        format!(
                            "{}:new={}:available={:?}:free_minus_reserve={}",
                            publish.name,
                            publish.shape.new_slots,
                            available_of(&publish.allocator_after, &publish.output),
                            i128::from(publish.allocator_after.devices[0].free_slots()) - reserve0
                        )
                    })
                    .collect();
                println!("R name=full_pool_delete variant={variant} result=Ok rows={rows:?}");
            }
            Err((name, error)) => println!(
                "R name=full_pool_delete variant={variant} result=Err({name}:{}) headroom_before_each={headroom:?}",
                short_error(error)
            ),
        }
        set_ledger(None);
    }
}

fn allocation_nodes_rewritten(output: &TransactionOutput) -> u64 {
    output
        .rewritten
        .iter()
        .filter(|role| {
            matches!(
                role,
                TransactionUnit::AllocationTreeNodeBelowTheRoot(_) | TransactionUnit::AllocationTree
            )
        })
        .count() as u64
}

/// mkfs → 取号 → 暖机 → 第一个文件（1 单元）→ 把每块盘的空闲槽标成已分配、只留 `holes` 个相隔 816 槽（超过一片分配记录叶 812 槽）的两槽空洞
/// 与尾部 8192 槽的全空段 → 顺序写成 `units` 个数据单元（落进空洞，一个单元一片叶）。盘上不写标记，只动分配器（同辩方腿第二轮的造法）。
fn pool_with_a_scattered_file(units: u64, holes: u64, geometry: Geometry) -> Pool {
    let parameters = parameters(geometry);
    let mut devices: Devices = (0..DEVICES)
        .map(|number| {
            let identity = DeviceIdentity(u32::try_from(number).expect("盘号"));
            (identity, SparseBlockDevice::new(geometry.image_bytes, PhysicalBlockSizeInBytes(512)))
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(
        (0..DEVICES)
            .map(|number| DeviceFreeMap::new(DeviceIdentity(u32::try_from(number).expect("盘号")), geometry.image_bytes))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let first_content = content_needing(1, 1);
    let (instance, current) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机根"),
            FirstFile { content: &first_content, write_time_seconds: WRITE_TIME },
            instance,
            &warm.last_record_bytes,
        )
        .expect("第一个文件");
        for map in &mut allocator.devices {
            let start = map.unit_area_start().slot().0;
            let end = start + map.unit_area_slots();
            let tail_start = end - if geometry.image_bytes > (4u64 << 30) { 80_000 } else { 8192 };
            let hole_starts: std::collections::BTreeSet<u64> = (0..holes).map(|k| start + 4096 + k * 816).collect();
            let mut slot = start;
            while slot < tail_start {
                let in_hole = hole_starts.contains(&slot) || hole_starts.contains(&(slot - 1));
                if map.is_free(SlotNumber(slot)) && !in_hole {
                    map.mark_allocated(SlotNumber(slot), 1);
                }
                slot += 1;
            }
        }
        let content = content_needing(units, 2);
        let written = publish_sequential_write(
            &mut pool,
            &mut allocator,
            &first,
            FirstFile { content: &content, write_time_seconds: WRITE_TIME + 60 },
            instance,
        )
        .expect("顺序写");
        (instance, written)
    };
    Pool { geometry, devices, allocator, current, instance }
}

/// G1 / G8：每批新占槽数的最大值随批大小 K、散落程度怎么走，与我提的上界 `w_upper_bound` 并排（4 GiB × 2 盘，mkfs 同一个进程里删）。
/// 一个单元一片叶是 K 个单元落在 K 片不同的分配记录叶上的最坏造法；各批的单元从尾部取，落在最右那几片叶。
#[test]
fn g1_w_by_batch_size_and_scatter_against_the_upper_bound() {
    for (image_bytes, holes, batch) in [
        (4u64 << 30, 16u64, 16u64),
        (4 << 30, 64, 64),
        (4 << 30, 144, 144),
        (4 << 30, 240, 144),
        (4 << 30, 144, 16),
        (4 << 30, 144, 1),
        (16 << 30, 144, 144),
        (16 << 30, 240, 144),
    ] {
        let geometry = Geometry { image_bytes, ring_slots_per_region: DEFAULT_GEOMETRY.ring_slots_per_region };
        let mut pool = pool_with_a_scattered_file(holes, holes, geometry);
        let nodes = allocation_level_counts(&pool);
        let before = pool.current.clone();
        set_r3_batch_switches(true, false);
        let published = delete_with_the_intent_carried(&mut pool, batch, &mut |_, _, _| {}).expect("删得完");
        set_r3_batch_switches(false, false);
        let news: Vec<u64> = published.iter().map(|publish| publish.shape.new_slots).collect();
        let allocs: Vec<u64> = published.iter().map(|publish| allocation_nodes_rewritten(&publish.output)).collect();
        let fixed: Vec<u64> = published.iter().map(|publish| publish.shape.fixed_point_slots).collect();
        let ordinary: Vec<u64> = published.iter().map(|publish| publish.shape.ordinary_slots).collect();
        println!(
            "R name=w_scatter image_gib={} units={holes} holes={holes} batch={batch} level_counts={nodes:?} w_upper_bound={} max_batch_new_slots={} new_slots={news:?} ordinary={ordinary:?} fixed_point={fixed:?} alloc_nodes_rewritten={allocs:?} checkpoint_cost_before={}",
            image_bytes >> 30,
            w_upper_bound(batch, &nodes),
            news[1..].iter().max().copied().unwrap_or(0),
            checkpoint_cost_of_the_version_to_build_on(Some(&before), &pool.allocator).0
        );
    }
}

/// G1：挂载之后（分配器按根环回收）、可用 = 0 的池上删一个 K = 1 的文件，两种账：零账（窗口不在账里）与环窗口账。
/// 顶点 = 保留量 − 空闲槽的最大值（opus 第二轮同口径）；「窗口没动到保留量」= 删除全程 空闲槽 ≥ 保留量。
#[test]
fn g1_ring_window_against_the_borrowed_apex() {
    for (ring, units) in [(8u64, 100u64), (8, 200), (16, 200)] {
        let geometry = Geometry { image_bytes: DEFAULT_GEOMETRY.image_bytes, ring_slots_per_region: ring };
        for with_window in [false, true] {
            let mut pool = remounted(&pool_with_a_file(units, geometry));
            let nodes = allocation_level_counts(&pool);
            let window = window_slots(3 * ring, PRODUCT_BATCH, &nodes);
            let ledger_bytes = if with_window { label_six(1, window) } else { 0 };
            set_ledger(Some((0, ledger_bytes)));
            fill_until_available_is_at_most(&mut pool, 0);
            let reserve0 = reserve_slots(&pool)[0];
            let mut free_series: Vec<i128> = vec![i128::from(pool.allocator.devices[0].free_slots())];
            set_r3_batch_switches(true, false);
            let result = delete_with_the_intent_carried(&mut pool, 1, &mut |_, _, pool| {
                set_ledger(Some((0, if with_window { label_six(0, window) } else { 0 })));
                free_series.push(i128::from(pool.allocator.devices[0].free_slots()));
            });
            free_series.push(i128::from(pool.allocator.devices[0].free_slots()));
            set_r3_batch_switches(false, false);
            set_ledger(None);
            let min_free = free_series.iter().min().copied().unwrap_or(0);
            let (min_index, _) = free_series.iter().enumerate().min_by_key(|(_, value)| **value).unwrap_or((0, &0));
            println!(
                "R name=ring_apex ring_roots={} units={units} with_window={with_window} window_slots={window} reserve_slots_dev0={reserve0} free_before={} min_free={min_free} at_publish={min_index} apex_deficit_slots={} min_free_minus_reserve={} result={}",
                3 * ring,
                free_series[0],
                free_series[0] - min_free,
                min_free - reserve0,
                match &result { Ok(published) => format!("Ok(publishes={})", published.len()), Err((name, error)) => format!("Err({name}:{})", short_error(error)) }
            );
        }
    }
}

/// G1：R3 的那个回退后果在环窗口账下还在不在。历史同 opus 第二轮：顺序写 300 单元（A）→ 覆盖写成 1 单元（C）→ 池填到可用 = 0 → 回退到 A。
/// 环窗口账里两版的活对象都是 1 个，标签 6 不变；打印回退前后每盘可用与标签 6 那一行。
#[test]
fn g1_rollback_to_a_larger_version_under_the_ring_window() {
    let geometry = DEFAULT_GEOMETRY;
    let mut pool = remounted(&pool_with_a_file(1, geometry));
    let nodes = allocation_level_counts(&pool);
    let window = window_slots(3 * geometry.ring_slots_per_region, PRODUCT_BATCH, &nodes);
    set_ledger(Some((0, label_six(1, window))));
    let big = 300u64;
    let content = content_needing(big, 3);
    let version_a = publish_on(&mut pool, |writer, allocator, previous, instance| {
        publish_sequential_write(writer, allocator, previous, FirstFile { content: &content, write_time_seconds: WRITE_TIME + 10 }, instance)
    })
    .expect("A：300 个单元");
    let small = content_needing(1, 4);
    publish_on(&mut pool, |writer, allocator, previous, instance| {
        publish_overwrite(writer, allocator, previous, FirstFile { content: &small, write_time_seconds: WRITE_TIME + 20 }, instance)
    })
    .expect("C：1 个单元");
    fill_until_available_is_at_most(&mut pool, 0);
    let available_at_c = available(&pool);
    let reserve_at_c = reserve_slots(&pool);
    let parameters = parameters(geometry);
    let rolled = roll_back_by_a_forward_publish(
        &parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.current,
        RollbackTarget { instance: version_a.root.instance, checkpoint_txg: CheckpointTxg(version_a.root.checkpoint_txg.0) },
    );
    let available_after = available(&pool);
    println!(
        "R name=rollback_under_the_window window_slots={window} label6_row_before={} rollback={} units_after={} label6_row_after={} available_at_c={available_at_c:?} available_after={available_after:?} reserve_slots_at_c={reserve_at_c:?} {}",
        label_six(1, window),
        match &rolled { Ok(_) => "Ok".to_owned(), Err(error) => format!("Err({error:?})") },
        pool.current.data_pointers.len(),
        accounting_row(&pool.current, singlefs_core::records::STATISTIC_COMMITTED_RESERVATION_BYTES),
        per_device_counts(&pool.allocator)
    );
    set_ledger(None);
}

/// G2：①b（同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同）的样本数。辩方腿第二轮只量了一个样本（3 单元、意图之后）；
/// 这里换六种镜像：文件大小、批大小、散落程度、停在意图之后还是第一批之后。每份镜像各自重挂载两次、各自续做到底，比两份盘、
/// 最后一版输出、实例号与 txg。
#[test]
fn g2_same_image_remounted_twice_continues_byte_equal() {
    let geometry = DEFAULT_GEOMETRY;
    let cases: [(&str, u64, u64, Option<u64>, usize); 6] = [
        ("n3_k1_after_intent", 3, 1, None, 1),
        ("n3_k1_after_batch_1", 3, 1, None, 2),
        ("n300_k144_after_intent", 300, 144, None, 1),
        ("n300_k144_after_batch_1", 300, 144, None, 2),
        ("n144_scattered_k144_after_intent", 144, 144, Some(144), 1),
        ("n240_scattered_k144_after_batch_1", 240, 144, Some(240), 2),
    ];
    for (label, units, batch, scattered, stop_after) in cases {
        let mut pool = match scattered {
            Some(holes) => pool_with_a_scattered_file(units, holes, geometry),
            None => pool_with_a_file(units, geometry),
        };
        let size = pool.current.inode_record.size;
        let mut steps = vec![("intent".to_owned(), u64::MAX, size)];
        // 意图之后的各批计划只读那一版的单元数（与 `plan_of_the_continuation` 同一个算法，意图那次不改单元数）
        let mut after_the_intent = plan_of_the_continuation(&pool.current, batch);
        steps.append(&mut after_the_intent);
        steps.truncate(stop_after);
        run_delete_publishes(&mut pool, steps, &mut |_, _, _| {}).expect("停在中间之前的那几次发布做成");
        let image = pool.image();
        let mut results = Vec::new();
        for _ in 0..2 {
            let mut devices = devices_of(&image, geometry);
            let mounted = mount_writable(&parameters(geometry), &mut devices).expect("可写挂载");
            let PoolVersion::WithFile(current) = mounted.current else { panic!("带文件的一版") };
            let mut remounted_pool = Pool {
                geometry,
                devices,
                allocator: mounted.allocator,
                current: current.clone(),
                instance: mounted.output.instance,
            };
            let published = continue_the_delete(&mut remounted_pool, batch, &mut |_, _, _| {}).expect("续做到底");
            results.push((
                remounted_pool.image(),
                remounted_pool.current.clone(),
                remounted_pool.instance,
                published.len(),
            ));
        }
        println!(
            "R name=redo_from_the_same_image case={label} images_byte_equal={} outputs_equal={} instance_first={} instance_second={} txg_first={} txg_second={} continuation_publishes={}",
            results[0].0 == results[1].0,
            results[0].1 == results[1].1,
            results[0].2 .0,
            results[1].2 .0,
            results[0].1.root.checkpoint_txg.0,
            results[1].1.root.checkpoint_txg.0,
            results[0].3
        );
        assert!(results[0].0 == results[1].0, "{label}：同一份镜像各自重挂载续做，盘上逐字节相同");
    }
}
