// ───────── 第 33 批代码轮攻方（opus）原型：由记录施加出来的版本与「下一次挂载先施加的那一版」不进 I-5.4 ─────────
// 只在攻方副本里追加到 crates/singlefs-harness/tests/checker_known_bad_images.rs 末尾，复用那份文件的私有搭建与改法。
// 搭建：`image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record`（副本里给它的结构多带一个字段，
// 见 tools/patch_builder.py）。打印行以 `B33X` 开头，判定交 tools/rerun.sh 汇总，不在这里断言结论。

fn b33x_line(label: &str, image: &MemoryPool) {
    let verdicts = check_pool_image(image);
    let violated = violated_invariants(&verdicts);
    let i54 = verdicts
        .iter()
        .find(|(name, _)| *name == "I-5.4")
        .expect("清单里有 I-5.4")
        .1
        .clone();
    println!("B33X label={label} violated={violated:?} i54={i54:?}");
    for (name, verdict) in &verdicts {
        if let InvariantVerdict::Violated(detail) = verdict {
            let short: String = detail.chars().take(400).collect();
            println!("B33X-DETAIL label={label} {name}: {short}");
        }
    }
}

/// 一片分配记录叶（码 2、层级 0、key 宽 10）里第一对「同盘、前一条正好接到后一条」的记录，前一条跨度加一：两条罩住同一个槽
/// （I-5.4 那一行的 Z1-d 形态）。交回改了几条（0 或 1）。
fn b33x_overlap_the_first_adjacent_pair(bytes: &mut Vec<u8>) -> usize {
    if bytes[INDEX_NODE_LEVEL_OFFSET] != 0 || bytes[51] != 10 {
        return 0;
    }
    let count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let record_at = |bytes: &[u8], index: usize| {
        let start = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
        let device = u32::from_le_bytes(bytes[start..start + 4].try_into().expect("4"));
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&bytes[start + 4..start + 10]);
        let span_field = u16::from_le_bytes([bytes[start + 10], bytes[start + 11]]);
        (start, device, u64::from_le_bytes(slot_bytes), span_field)
    };
    for index in 0..count.saturating_sub(1) {
        let (start, device, slot, span_field) = record_at(bytes, index);
        let (_, next_device, next_slot, _) = record_at(bytes, index + 1);
        let span = u64::from(span_field & !ALLOCATION_RECORD_RELEASED_FLAG);
        if device == next_device && slot + span == next_slot {
            let widened = (span_field & ALLOCATION_RECORD_RELEASED_FLAG)
                | u16::try_from(span + 1).expect("跨度小");
            bytes[start + 10..start + 12].copy_from_slice(&widened.to_le_bytes());
            return 1;
        }
    }
    0
}

/// 这一版这次发布重写的分配记录树里是叶的那几个单元的槽（两盘同槽）。
fn b33x_rewritten_allocation_leaf_slots(
    image: &MemoryPool,
    version: &singlefs_core::transaction::TransactionOutput,
) -> Vec<u64> {
    version
        .units
        .iter()
        .filter(|unit| {
            version.rewritten.contains(&unit.identity)
                && matches!(
                    unit.identity,
                    TransactionUnit::AllocationTree | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
                )
        })
        .map(|unit| unit.slot.0)
        .filter(|slot| read(image, 0, slot * SLOT, 16384)[INDEX_NODE_LEVEL_OFFSET] == 0)
        .collect()
}

/// 改 `version` 自己重写的第一片能改的分配记录叶（叶 → 内部节点 → 根 → 树表，沿引用链补校验和），再把这一版那条 journal 记录的
/// 新根段树表指针与点名项里跟着变的校验和改成新的、重封、后面的记录重新接链。交回改的是哪一片叶。
fn b33x_overlap_in_the_version_and_reseal_its_record(
    image: &mut MemoryPool,
    version: &singlefs_core::transaction::TransactionOutput,
) -> u64 {
    let units = units_found_on_device_zero(image);
    let before: Vec<(u64, u32)> = version
        .units
        .iter()
        .map(|unit| {
            let length = if unit.bytes.len() > 16384 { 32768 } else { 16384 };
            (unit.slot.0, crc32_castagnoli(&read(image, 0, unit.slot.0 * SLOT, length)))
        })
        .collect();
    let leaf = b33x_rewritten_allocation_leaf_slots(image, version)
        .into_iter()
        .find(|leaf| {
            let mut probe = read(image, 0, leaf * SLOT, 16384);
            b33x_overlap_the_first_adjacent_pair(&mut probe) == 1
        })
        .expect("这一版重写的叶里有一对同盘相邻的记录");
    mutate_unit_of(
        image,
        &units,
        leaf,
        |bytes| {
            assert_eq!(b33x_overlap_the_first_adjacent_pair(bytes), 1);
        },
        true,
    );
    let changed: Vec<(u64, u32, u32)> = version
        .units
        .iter()
        .zip(&before)
        .map(|(unit, (slot, old))| {
            let length = if unit.bytes.len() > 16384 { 32768 } else { 16384 };
            (*slot, *old, crc32_castagnoli(&read(image, 0, slot * SLOT, length)))
        })
        .filter(|(_, old, new)| old != new)
        .collect();
    println!("B33X resealed_units_of_the_version={changed:?}");
    b33x_rewrite_record_in_place(
        image,
        version.record.counter,
        |record| {
            let fix = |location: &mut singlefs_core::pointer::LocationEntry| {
                for (slot, old, new) in &changed {
                    if location.slot.0 == *slot && location.unit_checksum == *old {
                        location.unit_checksum = *new;
                    }
                }
            };
            record.new_tree_table.locations.iter_mut().for_each(&fix);
            record.new_allocation_record_tree_root.locations.iter_mut().for_each(&fix);
            record.new_mapping_root.locations.iter_mut().for_each(&fix);
            record.new_instance_table.locations.iter_mut().for_each(&fix);
            for named in &mut record.named {
                named.locations.iter_mut().for_each(&fix);
            }
        },
    );
    leaf
}

/// 镜像上可写挂载一次（内存盘，不改传进来的镜像）：做成、报错、panic 三种结局各打一行。
fn b33x_mount(label: &str, image: &MemoryPool) {
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(image, &[], &[], &stream);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        mount_writable(&parameters(), &mut devices).map(|mounted| {
            (
                mounted.output.chosen_root.checkpoint_txg.0,
                mounted.output.effective_root.checkpoint_txg.0,
                mounted.output.journal.prefix_applied,
            )
        })
    }));
    match outcome {
        Ok(Ok((chosen, effective, applied))) => println!(
            "B33X mount label={label} outcome=ok chosen_txg={chosen} effective_txg={effective} records_applied={applied}"
        ),
        Ok(Err(error)) => println!("B33X mount label={label} outcome=err error={error:?}"),
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_owned()))
                .unwrap_or_default();
            println!("B33X mount label={label} outcome=panic message={message:?}");
        }
    }
}

/// 一版里分配记录树的叶（不论这次重写没重写）：对照组用。
fn b33x_allocation_leaf_slots_of(
    image: &MemoryPool,
    version: &singlefs_core::transaction::TransactionOutput,
) -> Vec<u64> {
    version
        .units
        .iter()
        .filter(|unit| {
            matches!(
                unit.identity,
                TransactionUnit::AllocationTree | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            )
        })
        .map(|unit| unit.slot.0)
        .filter(|slot| read(image, 0, slot * SLOT, 16384)[INDEX_NODE_LEVEL_OFFSET] == 0)
        .collect()
}

#[test]
fn b33x_versions_built_from_journal_records_against_the_disjointness_invariant() {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "b33x-records-applied",
    );
    let version = &built.version_applied_only_by_its_record;
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    // 挂载之后：(1, 5) 只由记录施加出来，checker 把它并进回退候选集（I-3.1 / I-5.1 / I-7.4 / I-4.8 / I-5.5 / I-9.5 都走它）。
    b33x_line("a0-after-mount-clean", &built.image);
    let mut a1 = built.image.clone();
    for device in DEVICES {
        write(
            &mut a1,
            device,
            built.tree_table_slot_of_the_version_applied_only_by_its_record * SLOT,
            &vec![0u8; node_bytes],
        );
    }
    b33x_line("a1-after-mount-record-version-tree-table-zeroed", &a1);
    let allocation_root = version.unit(TransactionUnit::AllocationTree).slot.0;
    println!(
        "B33X record_version=({}, {}) counter={} allocation_root_slot={allocation_root} allocation_root_rewritten={} rewritten_allocation_leaves={:?}",
        version.record.instance.0,
        version.record.checkpoint_txg.0,
        version.record.counter,
        version.rewritten.contains(&TransactionUnit::AllocationTree),
        b33x_rewritten_allocation_leaf_slots(&built.image, version)
    );
    let mut a2 = built.image.clone();
    for device in DEVICES {
        write(&mut a2, device, allocation_root * SLOT, &vec![0u8; node_bytes]);
    }
    b33x_line("a2-after-mount-record-version-allocation-root-zeroed", &a2);
    let mut a3 = built.image.clone();
    let leaf = b33x_overlap_in_the_version_and_reseal_its_record(&mut a3, version);
    println!("B33X a3_leaf={leaf}");
    b33x_line("a3-after-mount-record-version-overlapping-records", &a3);
    // 对照：同一种改法改在挂载之后最新那一版（候选根）的一片分配记录叶上，I-5.4 该红。
    let mut c3 = built.image.clone();
    let units = units_found_on_device_zero(&c3);
    let newest_leaf = b33x_allocation_leaf_slots_of(&c3, &built.newest)
        .into_iter()
        .find(|leaf| {
            let mut probe = read(&c3, 0, leaf * SLOT, 16384);
            b33x_overlap_the_first_adjacent_pair(&mut probe) == 1
        })
        .expect("最新那一版的叶里有一对同盘相邻的记录");
    mutate_unit_of(
        &mut c3,
        &units,
        newest_leaf,
        |bytes| {
            assert_eq!(b33x_overlap_the_first_adjacent_pair(bytes), 1);
        },
        true,
    );
    println!("B33X c3_leaf={newest_leaf}");
    b33x_line("c3-after-mount-newest-version-overlapping-records", &c3);
    // 挂载之前：(1, 5) 的记录已持久、根槽没落，下一次可写挂载先施加它、从它的分配记录树重建分配器。
    b33x_line("b0-before-mount-clean", &built.image_before_the_mount);
    b33x_mount("b0-before-mount-clean", &built.image_before_the_mount);
    let mut b2 = built.image_before_the_mount.clone();
    for device in DEVICES {
        write(&mut b2, device, allocation_root * SLOT, &vec![0u8; node_bytes]);
    }
    b33x_line("b2-before-mount-next-version-allocation-root-zeroed", &b2);
    b33x_mount("b2-before-mount-next-version-allocation-root-zeroed", &b2);
    let mut b3 = built.image_before_the_mount.clone();
    let leaf = b33x_overlap_in_the_version_and_reseal_its_record(&mut b3, version);
    println!("B33X b3_leaf={leaf}");
    b33x_line("b3-before-mount-next-version-overlapping-records", &b3);
    b33x_mount("b3-before-mount-next-version-overlapping-records", &b3);
}

/// 默认环里 `counter` 那条记录两盘各一份照 `change` 改、`JournalRecord::to_bytes` 重封，不动后面的记录：
/// `common::rewrite_journal_record_and_rechain_the_records_after_it` 把后面别的实例的记录也接到它上面（I-8.6 只链本实例），这里用不上。
fn b33x_rewrite_record_in_place(
    image: &mut MemoryPool,
    counter: u64,
    change: impl Fn(&mut singlefs_core::journal::JournalRecord),
) {
    let filesystem_identifier =
        singlefs_core::unit::unit_filesystem_identifier(&parameters().filesystem_identifier);
    let offset =
        singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
    for device in DEVICES {
        let bytes = read(image, device, offset, 4096);
        let mut record = singlefs_core::journal::JournalRecord::parse(&bytes, filesystem_identifier)
            .expect("改之前这条记录自证过");
        change(&mut record);
        write(image, device, offset, &record.to_bytes());
    }
}

/// F2 那三段在根记录里的四条指针上：core 的根记录读者只判 extent 偏移（`root_record.rs` 四处 `read_judging_the_extent_offset_from`），
/// 池级 checker 从第 32 批起在走读跟随的每条指针上判算法类型。发布 B（txg 4）那条根的树表指针 / 实例表指针 / 映射根指针 /
/// （树表 0 条那一版才非零的）分配记录树根指针，算法类型（指针内偏移 28）写 1、重封根槽、抄进记录新根段；打印 checker 判定与可写挂载的结局。
#[test]
fn b33x_root_record_pointer_naming_an_encryption_algorithm_core_versus_checker() {
    let clean = image_after_the_overwrite("b33x-root-record-pointer-algorithm");
    b33x_line("f0-clean", &clean);
    b33x_mount("f0-clean", &clean);
    for (label, pointer_offset_in_the_root_record) in [
        ("f1-tree-table-pointer", 36usize),
        ("f2-instance-table-pointer", 170),
        ("f3-mapping-root-pointer", 256),
    ] {
        let mut image = clean.clone();
        rewrite_the_root_slot_of(&mut image, CheckpointTxg(4), |bytes| {
            assert_eq!(bytes[pointer_offset_in_the_root_record + 28], 0);
            bytes[pointer_offset_in_the_root_record + 28] = 1;
        });
        b33x_line(label, &image);
        b33x_mount(label, &image);
    }
}

/// 正文第二节末那一处：树表指针全零的根（I-9.28 / I-9.5 把它算成没有树表）、根记录那一项非零而那棵树读不出。
/// 建在 33-c 那份镜像上（写行那两版 txg 3、4 树表 0 条，根记录直接持有分配记录树；第一个文件版本 txg 5）：
/// y1 只把 txg 4 那条根的树表指针整条写零、重封根槽；y2 再把根记录那一项指着的分配记录树根两份清零。打印 checker 判定与可写挂载结局。
#[test]
fn b33x_root_with_an_all_zero_tree_table_pointer_and_an_unreadable_root_record_allocation_tree() {
    let (clean, root_record_allocation_tree_root_slot) =
        image_after_the_first_file_on_the_version_whose_rows_were_written("b33x-all-zero-tree-table-pointer");
    b33x_line("y0-clean", &clean);
    let mut y1 = clean.clone();
    rewrite_the_root_slot_of(&mut y1, CheckpointTxg(4), |bytes| {
        bytes[36..122].fill(0);
    });
    b33x_line("y1-txg4-tree-table-pointer-all-zero", &y1);
    b33x_mount("y1-txg4-tree-table-pointer-all-zero", &y1);
    let mut y2 = y1.clone();
    zero_both_copies_of_the_unit(
        &mut y2,
        root_record_allocation_tree_root_slot,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    b33x_line("y2-y1-plus-root-record-allocation-tree-zeroed", &y2);
    b33x_mount("y2-y1-plus-root-record-allocation-tree-zeroed", &y2);
    let mut y3 = clean.clone();
    zero_both_copies_of_the_unit(
        &mut y3,
        root_record_allocation_tree_root_slot,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    b33x_line("y3-only-root-record-allocation-tree-zeroed", &y3);
    // y4：最新那条根（第一个文件版本 txg 5）的树表指针整条写零、重封根槽。
    let mut y4 = clean.clone();
    rewrite_the_root_slot_of(&mut y4, CheckpointTxg(5), |bytes| {
        bytes[36..122].fill(0);
    });
    b33x_line("y4-newest-txg5-tree-table-pointer-all-zero", &y4);
    b33x_mount("y4-newest-txg5-tree-table-pointer-all-zero", &y4);
}
