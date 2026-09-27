from edit_lib import edit

S4 = "crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs"
edit(S4, """    is_a_user_visible_unit, mount_writable, mount_writable_with_test_only_switches,
    roll_back_by_a_forward_publish, Mounted, RollbackCandidateExclusion, RollbackError,
    RollbackTarget, RolledBack, ShadowLedger,
};""", """    is_a_user_visible_unit, mount_writable, mount_writable_with_test_only_switches,
    roll_back_by_a_forward_publish, Mounted, RollbackCandidateExclusion, RollbackError,
    RollbackTarget, RolledBack, ShadowLedger, StillUnreadableAfterOneReread,
};""")
edit(S4, """    user_visible_tree_root_pointers, walk_to_file, JournalPolicy, RecoveryFailure, RecoveryOutcome,
};""", """    user_visible_tree_root_pointers, walk_to_file, BadRootRingSlotReading, JournalPolicy,
    RecoveryFailure, RecoveryOutcome,
};""")
edit(S4, """/// 验收「水位」：C 那一版新建了三个 inode（号 2、3、4，水位 5），回退那一刻 C 的根槽读不出——环里读得出的根带的 inode 号水位都是 2；
/// D 的 inode 号水位取 max(C 内存里的 5, 环里的 2) = 5，D 之后再新建的 inode 从 5 起、不重发 2–4。树 ID 水位同一个取法。
/// 只取环里读得出的根时 D 的水位是 2——`crates/mutations.tsv` 里那一行证这条断言红。
#[test]
fn the_rollback_takes_the_watermarks_from_the_current_version_in_memory_when_its_root_is_unreadable(
) {""", """/// C 那一版新建了三个 inode（号 2、3、4，水位 5），回退那一刻 C 的根槽被改成全 0（自证不过）。C 的根是这个进程写过、FUA 返回过的，
/// 它住的槽在这个进程的根环表里：D16（发布语义） 已定项 1「根槽这一次读坏」那一行全句——读不出或自证不过就重读一次，仍坏就拒——
/// 管理员回退判候选读根环时拒成 `CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot { 自证不过, 自证不过 })`，
/// 在任何写之前：盘上逐字节不变、现行版本与分配器不动。根槽改回原样之后回退照做：D 的 inode 号水位 5，D 之后再新建的 inode 从 5 起、
/// 不重发 2–4；树 ID 水位同一个取法。
/// 这一条原来钉的是「C 的根槽读不出、回退照做、水位取内存里的」（合入后验证一只管「读不出」时自证不过当没有根）；照全句之后那一形在
/// 判候选时就拒（代码三方第二轮之后的规格第 3 条：已知根被改坏之后回退，改钉拒）。「水位取 max(内存里的, 环里读得出的)」那一判挪到
/// `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的
/// `the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root`
/// （判候选两遍读得出、算水位那一遍瞬时读不出），`crates/mutations.tsv` 里实三「水位」那一行跟着换靶。
#[test]
fn rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write(
) {""")
edit(S4, """    let third_root_slot = root_slot_position_of(8);
    let saved_third_root_slot = {
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let saved = read_root_slot(open_devices, third_root_slot);
        write_root_slot(open_devices, third_root_slot, &vec![0u8; root_slot_bytes()]);
        saved
    };
    roll_back(&mut pool, first_root()).expect("回退到 A：C 的根读不出不碍事，候选与账都读得出");
    assert_eq!(
        pool.output.inode_number_watermark(),
        5,
        "D 的 inode 号水位 = max(C 内存里的 5, 环里读得出的根的 2)"
    );""", """    let third_root_slot = root_slot_position_of(8);
    let saved_third_root_slot = {
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let saved = read_root_slot(open_devices, third_root_slot);
        write_root_slot(open_devices, third_root_slot, &vec![0u8; root_slot_bytes()]);
        saved
    };
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let current_before = pool.output.clone();
    let records_before = pool.allocator.records().to_vec();
    let refused = roll_back(&mut pool, first_root());
    assert!(
        matches!(
            &refused,
            Err(RollbackError::CandidateJudgementStillUnreadableAfterOneReread(still_bad))
                if **still_bad
                    == StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                        ring_slot: target_for_publish(
                            CheckpointTxg(8),
                            parameters().geometry.root_ring_slots_per_region,
                        ),
                        first_reading: BadRootRingSlotReading::NotSelfVerified,
                        reread: BadRootRingSlotReading::NotSelfVerified,
                    }
        ),
        "C 的根槽是这个进程写过的，自证不过重读仍坏：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在任何写之前"
    );
    assert_eq!(pool.output, current_before, "现行版本不动");
    assert_eq!(
        pool.allocator.records(),
        records_before.as_slice(),
        "分配器不动"
    );
    {
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        write_root_slot(open_devices, third_root_slot, &saved_third_root_slot);
    }
    roll_back(&mut pool, first_root()).expect("C 的根槽改回原样之后回退到 A");
    assert_eq!(
        pool.output.inode_number_watermark(),
        5,
        "D 的 inode 号水位 = max(C 内存里的 5, 环里读得出的根的)"
    );""")
edit(S4, """        BTreeSet::from([1, 5, 6]),
        "D 之后新建的号从 5 起：C 发过的 2–4 不重发"
    );
    {
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        write_root_slot(open_devices, third_root_slot, &saved_third_root_slot);
    }
    let verdicts""", """        BTreeSet::from([1, 5, 6]),
        "D 之后新建的号从 5 起：C 发过的 2–4 不重发"
    );
    let verdicts""")

S5 = "crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs"
edit(S5, """    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, MountError, RaisedFloor,
    RollbackCandidateExclusion, RollbackError, RollbackTarget, RolledBack, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, recover, scan_journal, JournalPolicy,
    RecoveryOutcome,
};""", """    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, MountError, RaisedFloor,
    RollbackError, RollbackTarget, RolledBack, ShadowLedger, StillUnreadableAfterOneReread,
};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, recover, scan_journal, BadRootRingSlotReading,
    JournalPolicy, RecoveryOutcome,
};""")
edit(S5, """/// 回退候选集的 F 用 F_生效（D16（发布语义） 已定项 1「生效」，SysPre：max(根上带的, 系统配置里读得出的)）：抬到 11 之后把盘 1 的载体
/// （txg 16）改坏，F_生效 仍是 11（盘 0 的 txg 15 与两块盘系统配置里都带 11），txg 9 的根 D 在 F 之下、不是候选，挂着的时候回退在任何写之前
/// 拒成 `BelowEffectiveFloor`、盘上逐字节不变、分配器与现行版本不动。改之前按「各盘所带 F 最大值的最小值」只读根，F_生效 回到 0、D 退得到。
#[test]
fn floor_carried_by_only_one_device_root_and_the_system_configuration_keeps_the_roots_below_it_out_of_the_rollback_candidates(
) {""", """/// 抬到 11 之后把盘 1 的载体（txg 16 的根）改坏一个字节（自证不过），挂着的时候回退到 txg 9 的根 D：那条载体是这个进程写过、FUA 返回过的，
/// 它住的槽在这个进程的根环表里——D16（发布语义） 已定项 1「根槽这一次读坏」那一行全句：读不出或自证不过就重读一次，仍坏就拒。
/// 回退判候选读根环时拒成 `CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot { 自证不过, 自证不过 })`，
/// 在任何写之前：盘上逐字节不变、分配器与现行版本不动。
/// 这一条原来钉的是「载体坏了当没有根，F_生效 照 SysPre（max(根上带的, 系统配置里读得出的)）仍是 11、D 拒成 `BelowEffectiveFloor`」；
/// 照全句之后已知根被改坏那一形在判候选时就拒（代码三方第二轮之后的规格第 3 条：已知根被改坏之后回退，改钉拒）。
/// 改坏挪到挂载之前（那时它不是已知根）保不住原来那一判：之后可写挂载的写行与暖机两块盘都写带 F 11 的根，「F 只在系统配置里」
/// 那一形就没了（推的，没造）。
#[test]
fn rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write(
) {""")
edit(S5, """    let second_carrier = raised.publishes[1].root.checkpoint_txg;
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let target = target_for_publish(
            second_carrier,
            parameters().geometry.root_ring_slots_per_region,
        );""", """    let second_carrier = raised.publishes[1].root.checkpoint_txg;
    let target = target_for_publish(
        second_carrier,
        parameters().geometry.root_ring_slots_per_region,
    );
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");""")
edit(S5, """    assert!(
        matches!(
            &refused,
            Err(RollbackError::TargetNotACandidate {
                exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,
                ..
            })
        ),
        "F_生效 仍是 11，txg 9 的根不在候选集里：{:?}",
        refused.as_ref().err()
    );""", """    assert!(
        matches!(
            &refused,
            Err(RollbackError::CandidateJudgementStillUnreadableAfterOneReread(still_bad))
                if **still_bad
                    == StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                        ring_slot: target,
                        first_reading: BadRootRingSlotReading::NotSelfVerified,
                        reread: BadRootRingSlotReading::NotSelfVerified,
                    }
        ),
        "载体是这个进程写过的，自证不过重读仍坏：{:?}",
        refused.as_ref().err()
    );""")
print("11_step_four_five done")
