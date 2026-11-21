//! 反例在区间版根环上被拒：两槽单元与回退目标还在读的槽、整理发布之后回收源槽、只看内存 F 就放开回收，
//! 窗口为一时整理过的根离开、源槽才可回收，整理的发布越过只在内存里抬过的 F 崩溃后少一个候选，
//! 现行领先最后一个根时整理的发布被拒；连同两个逻辑版本加一个旧读者满足不变量的见证。
use super::*;

verus! {

// ===== 三个反例在这一版上被拒 =====
// 两槽单元（变体二）与回退目标还在读的槽（变体一）：现行只有 key 7 在槽 10；回退目标（用户删掉 key 4 之前那一版，
// 版本号 0）里 key 4 是槽 20、21 两槽的单元。往槽 20、21 写，回收槽 21，都不合法；没人罩的槽 30 写得进去（防空真）。
pub proof fn lemma_both_slots_of_a_two_slot_unit_of_a_rollback_target_are_refused()
    ensures
        ({
            let current = RunView {
                mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }),
                logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]),
            };
            let target = RunView {
                mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }).insert(
                    4u64,
                    SlotRun { start: 20, span: 2 },
                ),
                logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]).insert(
                    4u64,
                    seq![seq![99u8], seq![98u8]],
                ),
            };
            let state = ring_state_of(
                Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![99u8]).insert(21int, seq![98u8]),
                seq![
                    RingRoot { view: target, name: RootName::UserVersion { version: 0 } },
                    RingRoot { view: current, name: RootName::UserVersion { version: 1 } },
                ],
                0,
                0,
                current,
                Seq::empty(),
                2,
                2,
            );
            &&& every_protected_view_reads_right(state)
            &&& user_version_is_a_candidate(state.roots, state.in_memory_floor, 0)
            &&& !run_ring_step_is_enabled(state, RunRingStep::Write { slot: 20, bytes: seq![42u8] })
            &&& !run_ring_step_is_enabled(state, RunRingStep::Write { slot: 21, bytes: seq![42u8] })
            &&& !run_ring_step_is_enabled(state, RunRingStep::Reclaim { slot: 21 })
            &&& run_ring_step_is_enabled(state, RunRingStep::Write { slot: 30, bytes: seq![42u8] })
        }),
{
    let current = RunView {
        mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }),
        logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]),
    };
    let target = RunView {
        mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }).insert(
            4u64,
            SlotRun { start: 20, span: 2 },
        ),
        logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]).insert(
            4u64,
            seq![seq![99u8], seq![98u8]],
        ),
    };
    let state = ring_state_of(
        Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![99u8]).insert(21int, seq![98u8]),
        seq![
            RingRoot { view: target, name: RootName::UserVersion { version: 0 } },
            RingRoot { view: current, name: RootName::UserVersion { version: 1 } },
        ],
        0,
        0,
        current,
        Seq::empty(),
        2,
        2,
    );
    assert(target.mapping.dom() =~= target.logical.dom());
    assert(current.mapping.dom() =~= current.logical.dom());
    assert(run_is_on_disk(state.disk, target.mapping[4u64])) by {
        assert forall|offset: int| 0 <= offset < 2 implies state.disk.contains_key(#[trigger] (20int + offset)) by {
            assert(offset == 0 || offset == 1);
        }
    }
    assert(run_is_on_disk(state.disk, target.mapping[7u64]));
    assert(run_bytes_on_disk(state.disk, target.mapping[7u64]) =~= seq![seq![42u8]]);
    assert(run_bytes_on_disk(state.disk, target.mapping[4u64]) =~= seq![seq![99u8], seq![98u8]]);
    assert(view_reads_its_logical_content(target, state.disk));
    assert(view_reads_its_logical_content(current, state.disk));
    assert(state.roots[0].view == target);
    assert(state.roots[1].view == current);
    assert(every_protected_view_reads_right(state));
    lemma_observed_roots_keep_each_root(state.roots);
    assert(observed_roots_of(state.roots)[0].name == (RootName::UserVersion { version: 0 }));
    assert(run_covers(target.mapping[4u64], 20int));
    assert(run_covers(target.mapping[4u64], 21int));
    assert(!mapping_avoids_slot(state.roots[0].view.mapping, 20int));
    assert(!mapping_avoids_slot(state.roots[0].view.mapping, 21int));
    assert(slot_is_unreferenced(state, 30int)) by {
        assert forall|key: u64| current.mapping.contains_key(key) implies !run_covers(#[trigger] current.mapping[key], 30int) by {
            assert(key == 7u64);
        }
        assert forall|index: int| state.persisted_floor <= index < state.roots.len() implies mapping_avoids_slot(
            (#[trigger] state.roots[index]).view.mapping,
            30int,
        ) by {
            if index == 0 {
                assert forall|key: u64| target.mapping.contains_key(key) implies !run_covers(
                    #[trigger] target.mapping[key],
                    30int,
                ) by {
                    assert(key == 7u64 || key == 4u64);
                }
            } else {
                assert(index == 1);
                assert forall|key: u64| current.mapping.contains_key(key) implies !run_covers(
                    #[trigger] current.mapping[key],
                    30int,
                ) by {
                    assert(key == 7u64);
                }
            }
        }
    }
}

// 五步（写目的地、换映射、整理的发布、回收源槽、覆写源槽）：根环容得下两个根时，整理发布之后旧根还在持久 F 之上，
// 第四步回收源槽不合法。
pub proof fn lemma_reclaiming_the_source_after_a_compaction_publish_is_refused_while_the_old_root_is_a_candidate()
    ensures
        ({
            let before = RunView {
                mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }),
                logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]),
            };
            let start = ring_state_of(
                Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]),
                seq![RingRoot { view: before, name: RootName::UserVersion { version: 0 } }],
                0,
                0,
                before,
                Seq::empty(),
                2,
                1,
            );
            let written = run_ring_next(start, RunRingStep::Write { slot: 20, bytes: seq![42u8] });
            let moved = run_ring_next(written, RunRingStep::Relocate { key: 7, destination: SlotRun { start: 20, span: 1 } });
            let published = run_ring_next(moved, RunRingStep::CompactionCheckpoint);
            &&& every_protected_view_reads_right(start)
            &&& run_ring_step_is_enabled(start, RunRingStep::Write { slot: 20, bytes: seq![42u8] })
            &&& run_ring_step_is_enabled(written, RunRingStep::Relocate { key: 7, destination: SlotRun { start: 20, span: 1 } })
            &&& published.persisted_floor == 0
            &&& user_version_is_a_candidate(published.roots, published.in_memory_floor, 0)
            &&& !run_ring_step_is_enabled(published, RunRingStep::Reclaim { slot: 10 })
            &&& !run_ring_step_is_enabled(published, RunRingStep::Write { slot: 10, bytes: seq![0u8] })
        }),
{
    let before = RunView {
        mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }),
        logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]),
    };
    let start = ring_state_of(
        Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]),
        seq![RingRoot { view: before, name: RootName::UserVersion { version: 0 } }],
        0,
        0,
        before,
        Seq::empty(),
        2,
        1,
    );
    assert(before.mapping.dom() =~= before.logical.dom());
    assert(run_bytes_on_disk(start.disk, before.mapping[7u64]) =~= seq![seq![42u8]]);
    assert(view_reads_its_logical_content(before, start.disk));
    assert(start.roots[0].view == before);
    assert(every_protected_view_reads_right(start));
    assert(mapping_avoids_slot(before.mapping, 20int)) by {
        assert forall|key: u64| before.mapping.contains_key(key) implies !run_covers(#[trigger] before.mapping[key], 20int) by {
            assert(key == 7u64);
        }
    }
    assert(slot_is_unreferenced(start, 20int));
    let written = run_ring_next(start, RunRingStep::Write { slot: 20, bytes: seq![42u8] });
    let destination = SlotRun { start: 20, span: 1 };
    assert(run_bytes_on_disk(written.disk, destination) =~= run_bytes_on_disk(written.disk, written.head.mapping[7u64]));
    assert(relocation_is_enabled(written, 7, destination));
    let moved = run_ring_next(written, RunRingStep::Relocate { key: 7, destination });
    let published = run_ring_next(moved, RunRingStep::CompactionCheckpoint);
    assert(persisted_floor_after_a_compaction_publish(moved) == 0);
    assert(in_memory_floor_after_a_compaction_publish(moved) == 0);
    assert(published.roots[0] == start.roots[0]);
    lemma_observed_roots_keep_each_root(published.roots);
    assert(observed_roots_of(published.roots)[0].name == (RootName::UserVersion { version: 0 }));
    assert(published.roots[0].view.mapping[7u64] == (SlotRun { start: 10, span: 1 }));
    assert(run_covers(published.roots[0].view.mapping[7u64], 10int));
    assert(!mapping_avoids_slot(published.roots[0].view.mapping, 10int));
}

// 根环只容一个根时，同样的五步里第四步合法：旧根被挤出持久 F 之下，回退点不到它，回收它的源槽不读坏任何受保护的视图。
// 用户看得见的变化是候选移了一格，不是读到坏字节。
pub proof fn lemma_with_a_window_of_one_the_compacted_root_leaves_and_its_source_becomes_reclaimable()
    ensures
        ({
            let logical = Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]);
            let before = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }), logical };
            let after = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 20, span: 1 }), logical };
            let moved = ring_state_of(
                Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![42u8]),
                seq![RingRoot { view: before, name: RootName::UserVersion { version: 0 } }],
                0,
                0,
                after,
                Seq::empty(),
                1,
                1,
            );
            let published = run_ring_next(moved, RunRingStep::CompactionCheckpoint);
            &&& every_protected_view_reads_right(moved)
            &&& published.persisted_floor == 1
            &&& run_ring_step_is_enabled(published, RunRingStep::Reclaim { slot: 10 })
            &&& !run_ring_step_is_enabled(published, RunRingStep::Rollback { version: 0 })
        }),
{
    let logical = Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]);
    let before = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }), logical };
    let after = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 20, span: 1 }), logical };
    let moved = ring_state_of(
        Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![42u8]),
        seq![RingRoot { view: before, name: RootName::UserVersion { version: 0 } }],
        0,
        0,
        after,
        Seq::empty(),
        1,
        1,
    );
    assert(before.mapping.dom() =~= before.logical.dom());
    assert(after.mapping.dom() =~= after.logical.dom());
    assert(run_bytes_on_disk(moved.disk, before.mapping[7u64]) =~= seq![seq![42u8]]);
    assert(run_bytes_on_disk(moved.disk, after.mapping[7u64]) =~= seq![seq![42u8]]);
    assert(view_reads_its_logical_content(before, moved.disk));
    assert(view_reads_its_logical_content(after, moved.disk));
    assert(moved.roots[0].view == before);
    assert(every_protected_view_reads_right(moved));
    let published = run_ring_next(moved, RunRingStep::CompactionCheckpoint);
    assert(persisted_floor_after_a_compaction_publish(moved) == 1);
    assert(in_memory_floor_after_a_compaction_publish(moved) == 1);
    assert(published.roots.len() == 2);
    assert(published.roots[1].view == after);
    assert(published.roots[1].name == RootName::CompactionPublish);
    assert(mapping_avoids_slot(after.mapping, 10int)) by {
        assert forall|key: u64| after.mapping.contains_key(key) implies !run_covers(#[trigger] after.mapping[key], 10int) by {
            assert(key == 7u64);
        }
    }
    assert(slot_is_unreferenced(published, 10int)) by {
        assert forall|index: int| published.persisted_floor <= index < published.roots.len() implies mapping_avoids_slot(
            (#[trigger] published.roots[index]).view.mapping,
            10int,
        ) by {
            assert(index == 1);
        }
    }
    lemma_observed_roots_keep_each_root(published.roots);
    assert(!user_version_is_a_candidate(published.roots, published.in_memory_floor, 0)) by {
        assert forall|index: int| published.in_memory_floor <= index < published.roots.len() implies (#[trigger] observed_roots_of(
            published.roots,
        )[index]).name != (RootName::UserVersion { version: 0 }) by {
            assert(index == 1);
        }
    }
}

// 只看内存 F 就放开回收的那一种：两个根（版本 0 的 key 7 在槽 10，版本 1 搬到了槽 20），F 只在内存里抬到版本 1。
// 这时回退已经点不到版本 0，但持久 F 还在它下面：回收槽 10 不合法。崩溃之后内存 F 回到持久 F，版本 0 又是回退候选，读得出它的内容；
// 下一次发布把 F 带进持久的根，槽 10 才放开。
pub proof fn lemma_a_floor_raised_only_in_memory_keeps_the_old_root_protected_until_a_publish_carries_it()
    ensures
        ({
            let logical = Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]);
            let old_view = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }), logical };
            let new_view = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 20, span: 1 }), logical };
            let state = ring_state_of(
                Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![42u8]),
                seq![
                    RingRoot { view: old_view, name: RootName::UserVersion { version: 0 } },
                    RingRoot { view: new_view, name: RootName::UserVersion { version: 1 } },
                ],
                0,
                0,
                new_view,
                Seq::empty(),
                3,
                2,
            );
            let raised = run_ring_next(state, RunRingStep::RaiseFloorInMemory { version: 1 });
            let crashed = run_ring_next(raised, RunRingStep::Crash);
            let carried = run_ring_next(raised, RunRingStep::UserCheckpoint);
            &&& every_protected_view_reads_right(state)
            &&& run_ring_step_is_enabled(state, RunRingStep::RaiseFloorInMemory { version: 1 })
            &&& raised.in_memory_floor == 1
            &&& raised.persisted_floor == 0
            &&& !run_ring_step_is_enabled(raised, RunRingStep::Rollback { version: 0 })
            &&& !run_ring_step_is_enabled(raised, RunRingStep::Reclaim { slot: 10 })
            &&& run_ring_step_is_enabled(crashed, RunRingStep::Rollback { version: 0 })
            &&& every_protected_view_reads_right(crashed)
            &&& carried.persisted_floor == 1
            &&& run_ring_step_is_enabled(carried, RunRingStep::Reclaim { slot: 10 })
        }),
{
    let logical = Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]);
    let old_view = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }), logical };
    let new_view = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 20, span: 1 }), logical };
    let state = ring_state_of(
        Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![42u8]),
        seq![
            RingRoot { view: old_view, name: RootName::UserVersion { version: 0 } },
            RingRoot { view: new_view, name: RootName::UserVersion { version: 1 } },
        ],
        0,
        0,
        new_view,
        Seq::empty(),
        3,
        2,
    );
    assert(old_view.mapping.dom() =~= old_view.logical.dom());
    assert(new_view.mapping.dom() =~= new_view.logical.dom());
    assert(run_bytes_on_disk(state.disk, old_view.mapping[7u64]) =~= seq![seq![42u8]]);
    assert(run_bytes_on_disk(state.disk, new_view.mapping[7u64]) =~= seq![seq![42u8]]);
    assert(view_reads_its_logical_content(old_view, state.disk));
    assert(view_reads_its_logical_content(new_view, state.disk));
    assert(state.roots[0].view == old_view);
    assert(state.roots[1].view == new_view);
    assert(every_protected_view_reads_right(state));
    lemma_observed_roots_keep_each_root(state.roots);
    assert(observed_roots_of(state.roots)[1].name == (RootName::UserVersion { version: 1 }));
    assert(user_version_is_a_candidate(state.roots, state.in_memory_floor, 1));
    let target = index_of_user_version(state.roots, state.in_memory_floor, 1);
    assert(target == 1) by {
        if target == 0 {
            assert(observed_roots_of(state.roots)[0].name == (RootName::UserVersion { version: 0 }));
        }
    }
    let raised = run_ring_next(state, RunRingStep::RaiseFloorInMemory { version: 1 });
    lemma_every_run_ring_step_keeps_every_protected_view(state, RunRingStep::RaiseFloorInMemory { version: 1 });
    assert(raised.in_memory_floor == 1);
    assert(!user_version_is_a_candidate(raised.roots, raised.in_memory_floor, 0)) by {
        assert forall|index: int| raised.in_memory_floor <= index < raised.roots.len() implies (#[trigger] observed_roots_of(
            raised.roots,
        )[index]).name != (RootName::UserVersion { version: 0 }) by {
            assert(index == 1);
        }
    }
    assert(raised.roots[0].view.mapping[7u64] == (SlotRun { start: 10, span: 1 }));
    assert(run_covers(raised.roots[0].view.mapping[7u64], 10int));
    assert(!mapping_avoids_slot(raised.roots[0].view.mapping, 10int));
    let crashed = run_ring_next(raised, RunRingStep::Crash);
    lemma_every_run_ring_step_keeps_every_protected_view(raised, RunRingStep::Crash);
    assert(crashed.in_memory_floor == 0);
    assert(observed_roots_of(crashed.roots)[0].name == (RootName::UserVersion { version: 0 }));
    let carried = run_ring_next(raised, RunRingStep::UserCheckpoint);
    assert(floor_after_a_checkpoint(raised) == 1);
    assert(carried.roots.len() == 3);
    assert(carried.roots[1].view == new_view);
    assert(carried.roots[2].view == new_view);
    assert(mapping_avoids_slot(new_view.mapping, 10int)) by {
        assert forall|key: u64| new_view.mapping.contains_key(key) implies !run_covers(#[trigger] new_view.mapping[key], 10int) by {
            assert(key == 7u64);
        }
    }
    assert(slot_is_unreferenced(carried, 10int)) by {
        assert forall|index: int| carried.persisted_floor <= index < carried.roots.len() implies mapping_avoids_slot(
            (#[trigger] carried.roots[index]).view.mapping,
            10int,
        ) by {
            assert(index == 1 || index == 2);
        }
    }
}

// 协议核实现员推的那个例子：根环容 3 个根，版本 0、1、2 三个根，
// F 在版本 0；用户把 F 只在内存里抬到版本 2；整理那份插一次整理的发布，之后两份都崩溃。整理的发布只带持久 F，根数超过容量挤掉版本 0
// （持久 F 到版本 1）：崩溃之后整理那份的候选是版本 1、2，不整理那份是版本 0、1、2，只少 1 个（k = 1）。
// 整理的发布要是带内存 F，持久 F 跟到版本 2，崩溃之后整理那份只剩版本 2，少 2 个。
pub proof fn lemma_a_compaction_publish_over_a_floor_raised_only_in_memory_loses_one_candidate_across_a_crash()
    ensures
        ({
            let view = RunView { mapping: Map::<u64, SlotRun>::empty(), logical: Map::<u64, Seq<Seq<u8>>>::empty() };
            let state = ring_state_of(
                Map::<int, Seq<u8>>::empty(),
                seq![
                    RingRoot { view, name: RootName::UserVersion { version: 0 } },
                    RingRoot { view, name: RootName::UserVersion { version: 1 } },
                    RingRoot { view, name: RootName::UserVersion { version: 2 } },
                ],
                0,
                0,
                view,
                Seq::empty(),
                3,
                3,
            );
            let raised = run_ring_next(state, RunRingStep::RaiseFloorInMemory { version: 2 });
            let crashed_without_compaction = run_ring_next(raised, RunRingStep::Crash);
            let compacted = run_ring_next(raised, RunRingStep::CompactionCheckpoint);
            let crashed_with_compaction = run_ring_next(compacted, RunRingStep::Crash);
            &&& every_protected_view_reads_right(state)
            &&& run_ring_step_is_enabled(state, RunRingStep::RaiseFloorInMemory { version: 2 })
            &&& raised.in_memory_floor == 2
            &&& raised.persisted_floor == 0
            &&& compacted.persisted_floor == 1
            &&& user_version_is_a_candidate(crashed_without_compaction.roots, crashed_without_compaction.in_memory_floor, 0)
            &&& user_version_is_a_candidate(crashed_without_compaction.roots, crashed_without_compaction.in_memory_floor, 1)
            &&& user_version_is_a_candidate(crashed_without_compaction.roots, crashed_without_compaction.in_memory_floor, 2)
            &&& !user_version_is_a_candidate(crashed_with_compaction.roots, crashed_with_compaction.in_memory_floor, 0)
            &&& user_version_is_a_candidate(crashed_with_compaction.roots, crashed_with_compaction.in_memory_floor, 1)
            &&& user_version_is_a_candidate(crashed_with_compaction.roots, crashed_with_compaction.in_memory_floor, 2)
        }),
{
    let view = RunView { mapping: Map::<u64, SlotRun>::empty(), logical: Map::<u64, Seq<Seq<u8>>>::empty() };
    let state = ring_state_of(
        Map::<int, Seq<u8>>::empty(),
        seq![
            RingRoot { view, name: RootName::UserVersion { version: 0 } },
            RingRoot { view, name: RootName::UserVersion { version: 1 } },
            RingRoot { view, name: RootName::UserVersion { version: 2 } },
        ],
        0,
        0,
        view,
        Seq::empty(),
        3,
        3,
    );
    assert(view.mapping.dom() =~= view.logical.dom());
    assert(view_reads_its_logical_content(view, state.disk));
    assert(every_protected_view_reads_right(state)) by {
        assert forall|index: int| state.persisted_floor <= index < state.roots.len() implies view_reads_its_logical_content(
            (#[trigger] state.roots[index]).view,
            state.disk,
        ) by {
            assert(state.roots[index].view == view);
        }
    }
    lemma_observed_roots_keep_each_root(state.roots);
    assert(observed_roots_of(state.roots)[0].name == (RootName::UserVersion { version: 0 }));
    assert(observed_roots_of(state.roots)[1].name == (RootName::UserVersion { version: 1 }));
    assert(observed_roots_of(state.roots)[2].name == (RootName::UserVersion { version: 2 }));
    assert(user_version_is_a_candidate(state.roots, state.in_memory_floor, 2));
    let target = index_of_user_version(state.roots, state.in_memory_floor, 2);
    assert(target == 2) by {
        if target == 0 {
            assert(observed_roots_of(state.roots)[0].name == (RootName::UserVersion { version: 0 }));
        } else if target == 1 {
            assert(observed_roots_of(state.roots)[1].name == (RootName::UserVersion { version: 1 }));
        }
    }
    let raised = run_ring_next(state, RunRingStep::RaiseFloorInMemory { version: 2 });
    assert(raised.roots == state.roots);
    assert(raised.in_memory_floor == 2);
    let crashed_without_compaction = run_ring_next(raised, RunRingStep::Crash);
    assert(crashed_without_compaction.roots == state.roots);
    assert(crashed_without_compaction.in_memory_floor == 0);
    assert(user_version_is_a_candidate(crashed_without_compaction.roots, crashed_without_compaction.in_memory_floor, 0));
    assert(user_version_is_a_candidate(crashed_without_compaction.roots, crashed_without_compaction.in_memory_floor, 1));
    let compacted = run_ring_next(raised, RunRingStep::CompactionCheckpoint);
    assert(persisted_floor_after_a_compaction_publish(raised) == 1);
    assert(in_memory_floor_after_a_compaction_publish(raised) == 2);
    assert(compacted.persisted_floor == 1);
    let crashed_with_compaction = run_ring_next(compacted, RunRingStep::Crash);
    assert(crashed_with_compaction.in_memory_floor == 1);
    assert(crashed_with_compaction.roots.len() == 4);
    lemma_observed_roots_keep_each_root(crashed_with_compaction.roots);
    assert(crashed_with_compaction.roots[1] == state.roots[1]);
    assert(crashed_with_compaction.roots[2] == state.roots[2]);
    assert(crashed_with_compaction.roots[3].name == RootName::CompactionPublish);
    assert(observed_roots_of(crashed_with_compaction.roots)[1].name == (RootName::UserVersion { version: 1 }));
    assert(observed_roots_of(crashed_with_compaction.roots)[2].name == (RootName::UserVersion { version: 2 }));
    assert(!user_version_is_a_candidate(crashed_with_compaction.roots, crashed_with_compaction.in_memory_floor, 0)) by {
        assert forall|index: int| crashed_with_compaction.in_memory_floor <= index < crashed_with_compaction.roots.len() implies (
        #[trigger] observed_roots_of(crashed_with_compaction.roots)[index]).name != (RootName::UserVersion { version: 0 }) by {
            assert(index == 1 || index == 2 || index == 3);
        }
    }
}

// 整理的发布那一道门拦下的历史（`research/prompts/m4-close-code-r1-opus-s2-output.md` T8 G7 那一段手推的反例：用户改现行、整理发布、崩溃）：
// 起点一个用户根（版本 0，key 7 在槽 10、字节 42；槽 20 也写着 42），现行就是它。把 key 7 搬到槽 20 之后现行的映射与根不同、逻辑内容相同，
// 整理的发布照发；用户删掉 key 7（UserPublish，还没落成根）之后整理的发布不发——发了就把这次删除先落成根，崩溃之后整理那份重现的是删过的内容、
// 不整理那份是版本 0；这次删除落成根（UserCheckpoint）之后又照发。
pub proof fn lemma_a_compaction_publish_is_refused_while_the_head_is_ahead_of_the_last_root()
    ensures
        ({
            let logical = Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]);
            let before = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }), logical };
            let deleted = RunView { mapping: Map::<u64, SlotRun>::empty(), logical: Map::<u64, Seq<Seq<u8>>>::empty() };
            let start = ring_state_of(
                Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![42u8]),
                seq![RingRoot { view: before, name: RootName::UserVersion { version: 0 } }],
                0,
                0,
                before,
                Seq::empty(),
                2,
                1,
            );
            let moved = run_ring_next(start, RunRingStep::Relocate { key: 7, destination: SlotRun { start: 20, span: 1 } });
            let changed = run_ring_next(start, RunRingStep::UserPublish { new_view: deleted });
            let carried = run_ring_next(changed, RunRingStep::UserCheckpoint);
            &&& every_protected_view_reads_right(start)
            &&& run_ring_step_is_enabled(start, RunRingStep::Relocate { key: 7, destination: SlotRun { start: 20, span: 1 } })
            &&& moved.head.mapping != moved.roots[0].view.mapping
            &&& run_ring_step_is_enabled(moved, RunRingStep::CompactionCheckpoint)
            &&& run_ring_step_is_enabled(start, RunRingStep::UserPublish { new_view: deleted })
            &&& !run_ring_step_is_enabled(changed, RunRingStep::CompactionCheckpoint)
            &&& run_ring_step_is_enabled(carried, RunRingStep::CompactionCheckpoint)
        }),
{
    let logical = Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]);
    let before = RunView { mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }), logical };
    let deleted = RunView { mapping: Map::<u64, SlotRun>::empty(), logical: Map::<u64, Seq<Seq<u8>>>::empty() };
    let start = ring_state_of(
        Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![42u8]),
        seq![RingRoot { view: before, name: RootName::UserVersion { version: 0 } }],
        0,
        0,
        before,
        Seq::empty(),
        2,
        1,
    );
    assert(before.mapping.dom() =~= before.logical.dom());
    assert(run_bytes_on_disk(start.disk, before.mapping[7u64]) =~= seq![seq![42u8]]);
    assert(view_reads_its_logical_content(before, start.disk));
    assert(start.roots[0].view == before);
    assert(every_protected_view_reads_right(start));
    let destination = SlotRun { start: 20, span: 1 };
    assert(run_bytes_on_disk(start.disk, destination) =~= run_bytes_on_disk(start.disk, start.head.mapping[7u64]));
    assert(relocation_is_enabled(start, 7, destination));
    let moved = run_ring_next(start, RunRingStep::Relocate { key: 7, destination });
    assert(moved.roots[0].view == before);
    assert(moved.head.mapping[7u64] == destination);
    assert(moved.roots[0].view.mapping[7u64] == (SlotRun { start: 10, span: 1 }));
    assert(moved.head.mapping != moved.roots[0].view.mapping);
    assert(moved.head.logical == moved.roots[0].view.logical);
    assert(deleted.mapping.dom() =~= deleted.logical.dom());
    assert(view_reads_its_logical_content(deleted, start.disk));
    let changed = run_ring_next(start, RunRingStep::UserPublish { new_view: deleted });
    assert(changed.roots[0].view == before);
    assert(changed.roots[0].view.logical.contains_key(7u64));
    assert(!changed.head.logical.contains_key(7u64));
    assert(changed.head.logical != changed.roots[0].view.logical);
    let carried = run_ring_next(changed, RunRingStep::UserCheckpoint);
    assert(carried.roots.len() == 2);
    assert(carried.roots[1].view == deleted);
    assert(carried.head == deleted);
}

// 多个逻辑版本的见证：两个根的 key 集合不同（用户在旧根之后写出了新 key），一个读者拿着旧版本，不变量照样成立；
// 防止「所有视图共享同一份逻辑内容」那一形回来，也防止历史定理只在空映射上成立。
pub proof fn witness_two_logical_versions_and_an_old_reader_satisfy_the_invariant()
    ensures
        exists|state: RunRingState|
            #[trigger] every_protected_view_reads_right(state) && state.roots.len() == 2 && state.readers.len() == 1
                && state.roots[0].view.logical.dom() != state.roots[1].view.logical.dom(),
{
    let old_view = RunView {
        mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }),
        logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![1u8]]),
    };
    let new_view = RunView {
        mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }).insert(
            8u64,
            SlotRun { start: 11, span: 2 },
        ),
        logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![1u8]]).insert(8u64, seq![seq![2u8], seq![3u8]]),
    };
    let state = ring_state_of(
        Map::<int, Seq<u8>>::empty().insert(10int, seq![1u8]).insert(11int, seq![2u8]).insert(12int, seq![3u8]),
        seq![
            RingRoot { view: old_view, name: RootName::UserVersion { version: 0 } },
            RingRoot { view: new_view, name: RootName::UserVersion { version: 1 } },
        ],
        0,
        0,
        new_view,
        seq![old_view],
        2,
        2,
    );
    assert(old_view.mapping.dom() =~= old_view.logical.dom());
    assert(new_view.mapping.dom() =~= new_view.logical.dom());
    assert(run_bytes_on_disk(state.disk, old_view.mapping[7u64]) =~= seq![seq![1u8]]);
    assert(run_is_on_disk(state.disk, new_view.mapping[8u64])) by {
        assert forall|offset: int| 0 <= offset < 2 implies state.disk.contains_key(#[trigger] (11int + offset)) by {
            assert(offset == 0 || offset == 1);
        }
    }
    assert(run_bytes_on_disk(state.disk, new_view.mapping[8u64]) =~= seq![seq![2u8], seq![3u8]]);
    assert(view_reads_its_logical_content(old_view, state.disk));
    assert(view_reads_its_logical_content(new_view, state.disk)) by {
        assert forall|key: u64| new_view.mapping.contains_key(key) implies run_is_on_disk(state.disk, #[trigger] new_view.mapping[key])
            && new_view.logical[key] == run_bytes_on_disk(state.disk, new_view.mapping[key]) by {
            if key == 8u64 {
            } else {
                assert(key == 7u64);
                assert(run_bytes_on_disk(state.disk, new_view.mapping[key]) =~= seq![seq![1u8]]);
            }
        }
    }
    assert(state.roots[0].view == old_view);
    assert(state.roots[1].view == new_view);
    assert(state.readers[0] == old_view);
    assert(every_protected_view_reads_right(state));
    assert(!(old_view.logical.dom() =~= new_view.logical.dom())) by {
        assert(new_view.logical.dom().contains(8u64));
        assert(!old_view.logical.dom().contains(8u64));
    }
}

} // verus!
