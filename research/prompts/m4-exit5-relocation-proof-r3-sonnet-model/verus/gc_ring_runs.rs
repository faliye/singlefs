//! 第三轮辩方腿：把第二轮正推的根环协议 `gc_ring_proofs` 的映射值从「一个槽」提成「槽区间」（起点加跨度），
//! 其余照原文件（步骤、窗口、F、读者、Crash 都不动）。目的：回答 Z1 的两个变体——
//!   变体一：搬迁的目的写覆写回退目标还在读的槽；
//!   变体二：回退目标里的两槽单元，第二槽照样可写（原协议按起点槽号避让）。
//! 槽号与跨度用 int，和第二轮正推的 `relocation_interval_proofs` 同一选择（接 core 的 u64 另证）。
use vstd::prelude::*;

verus! {

pub struct SlotRun {
    pub start: int,
    pub span: int,
}

pub open spec fn run_covers(run: SlotRun, slot: int) -> bool {
    run.start <= slot < run.start + run.span
}

// 一个单元的内容：区间里每个槽上的字节，按槽序排成一串。
pub open spec fn run_bytes(disk: Map<int, Seq<u8>>, run: SlotRun) -> Seq<Seq<u8>> {
    Seq::new(run.span as nat, |offset: int| disk[run.start + offset])
}

pub open spec fn run_is_on_disk(disk: Map<int, Seq<u8>>, run: SlotRun) -> bool {
    &&& run.span >= 1
    &&& forall|offset: int| 0 <= offset < run.span ==> disk.contains_key(#[trigger] (run.start + offset))
}

pub struct RunView {
    pub mapping: Map<u64, SlotRun>,
    pub logical: Map<u64, Seq<Seq<u8>>>,
}

pub struct RunRingState {
    pub disk: Map<int, Seq<u8>>,
    pub roots: Seq<RunView>,
    pub floor: nat,
    pub head: RunView,
    pub readers: Seq<RunView>,
    pub window: nat,
}

pub enum RunStep {
    Write { slot: int, bytes: Seq<u8> },
    Reclaim { slot: int },
    Relocate { key: u64, destination: SlotRun },
    UserPublish { new_view: RunView },
    Checkpoint,
    Rollback { target: int },
    RaiseFloor { new_floor: nat },
    AcquireReader,
    ReleaseReader { index: int },
    Crash,
}

pub open spec fn run_mapping_matches(
    mapping: Map<u64, SlotRun>, disk: Map<int, Seq<u8>>, logical: Map<u64, Seq<Seq<u8>>>,
) -> bool {
    &&& mapping.dom() == logical.dom()
    &&& forall|key: u64| mapping.contains_key(key) ==>
        run_is_on_disk(disk, #[trigger] mapping[key]) && logical[key] == run_bytes(disk, mapping[key])
}

pub open spec fn run_view_reads_right(view: RunView, disk: Map<int, Seq<u8>>) -> bool {
    run_mapping_matches(view.mapping, disk, view.logical)
}

pub open spec fn run_ring_state_is_shaped(s: RunRingState) -> bool {
    &&& s.roots.len() >= 1
    &&& s.floor < s.roots.len()
    &&& s.window >= 1
    &&& s.roots.len() - s.floor <= s.window
}

pub open spec fn run_ring_all_views_read_right(s: RunRingState) -> bool {
    &&& run_ring_state_is_shaped(s)
    &&& run_view_reads_right(s.head, s.disk)
    &&& forall|i: int| s.floor <= i < s.roots.len() ==> run_view_reads_right(#[trigger] s.roots[i], s.disk)
    &&& forall|i: int| 0 <= i < s.readers.len() ==> run_view_reads_right(#[trigger] s.readers[i], s.disk)
}

// 按区间避让：映射里没有任何一个区间罩住这个槽。
pub open spec fn run_map_avoids(mapping: Map<u64, SlotRun>, slot: int) -> bool {
    forall|key: u64| mapping.contains_key(key) ==> !run_covers(#[trigger] mapping[key], slot)
}

pub open spec fn run_slot_is_unreferenced(s: RunRingState, slot: int) -> bool {
    &&& run_map_avoids(s.head.mapping, slot)
    &&& forall|i: int| s.floor <= i < s.roots.len() ==> run_map_avoids(#[trigger] s.roots[i].mapping, slot)
    &&& forall|i: int| 0 <= i < s.readers.len() ==> run_map_avoids(#[trigger] s.readers[i].mapping, slot)
}

pub open spec fn run_step_enabled(s: RunRingState, step: RunStep) -> bool {
    match step {
        RunStep::Write { slot, .. } | RunStep::Reclaim { slot } => run_slot_is_unreferenced(s, slot),
        RunStep::Relocate { key, destination } => {
            &&& s.head.mapping.contains_key(key)
            &&& run_is_on_disk(s.disk, s.head.mapping[key])
            &&& destination.span == s.head.mapping[key].span
            &&& run_is_on_disk(s.disk, destination)
            &&& run_bytes(s.disk, destination) == run_bytes(s.disk, s.head.mapping[key])
        },
        RunStep::UserPublish { new_view } => run_view_reads_right(new_view, s.disk),
        RunStep::Rollback { target } => s.floor <= target < s.roots.len(),
        RunStep::RaiseFloor { new_floor } => s.floor <= new_floor < s.roots.len(),
        RunStep::ReleaseReader { index } => 0 <= index < s.readers.len(),
        _ => true,
    }
}

pub open spec fn run_floor_after_a_checkpoint(s: RunRingState) -> nat {
    if s.roots.len() + 1 - s.floor > s.window { (s.floor + 1) as nat } else { s.floor }
}

pub open spec fn run_ring_next(s: RunRingState, step: RunStep) -> RunRingState {
    match step {
        RunStep::Write { slot, bytes } => RunRingState { disk: s.disk.insert(slot, bytes), ..s },
        RunStep::Reclaim { slot } => RunRingState { disk: s.disk.remove(slot), ..s },
        RunStep::Relocate { key, destination } => RunRingState {
            head: RunView { mapping: s.head.mapping.insert(key, destination), logical: s.head.logical },
            ..s
        },
        RunStep::UserPublish { new_view } => RunRingState { head: new_view, ..s },
        RunStep::Checkpoint => RunRingState {
            roots: s.roots.push(s.head),
            floor: run_floor_after_a_checkpoint(s),
            ..s
        },
        RunStep::Rollback { target } => RunRingState { head: s.roots[target], ..s },
        RunStep::RaiseFloor { new_floor } => RunRingState { floor: new_floor, ..s },
        RunStep::AcquireReader => RunRingState { readers: s.readers.push(s.head), ..s },
        RunStep::ReleaseReader { index } => RunRingState { readers: s.readers.remove(index), ..s },
        RunStep::Crash => RunRingState { head: s.roots[s.roots.len() - 1], readers: Seq::empty(), ..s },
    }
}

// 避开这个槽的视图，在盘上这个槽被写或被回收之后仍读对。
proof fn lemma_run_write_keeps_a_protected_view(
    view: RunView, disk: Map<int, Seq<u8>>, slot: int, bytes: Seq<u8>,
)
    requires
        run_view_reads_right(view, disk),
        run_map_avoids(view.mapping, slot),
    ensures
        run_view_reads_right(view, disk.insert(slot, bytes)),
        run_view_reads_right(view, disk.remove(slot)),
{
    let written = disk.insert(slot, bytes);
    let removed = disk.remove(slot);
    assert forall|key: u64| view.mapping.contains_key(key) implies
        run_is_on_disk(written, #[trigger] view.mapping[key]) && view.logical[key] == run_bytes(written, view.mapping[key]) by {
        let run = view.mapping[key];
        assert(!run_covers(run, slot));
        assert(run_is_on_disk(disk, run));
        assert forall|offset: int| 0 <= offset < run.span implies written.contains_key(#[trigger] (run.start + offset)) by {
            assert(run.start + offset != slot);
        }
        assert(run_bytes(written, run) =~= run_bytes(disk, run)) by {
            assert forall|offset: int| 0 <= offset < run.span implies #[trigger] written[run.start + offset] == disk[run.start + offset] by {
                assert(run.start + offset != slot);
            }
        }
    }
    assert forall|key: u64| view.mapping.contains_key(key) implies
        run_is_on_disk(removed, #[trigger] view.mapping[key]) && view.logical[key] == run_bytes(removed, view.mapping[key]) by {
        let run = view.mapping[key];
        assert(!run_covers(run, slot));
        assert(run_is_on_disk(disk, run));
        assert forall|offset: int| 0 <= offset < run.span implies removed.contains_key(#[trigger] (run.start + offset)) by {
            assert(run.start + offset != slot);
        }
        assert(run_bytes(removed, run) =~= run_bytes(disk, run)) by {
            assert forall|offset: int| 0 <= offset < run.span implies #[trigger] removed[run.start + offset] == disk[run.start + offset] by {
                assert(run.start + offset != slot);
            }
        }
    }
}

pub proof fn lemma_run_ring_step_preserves_every_view(s: RunRingState, step: RunStep)
    requires
        run_ring_all_views_read_right(s),
        run_step_enabled(s, step),
    ensures
        run_ring_all_views_read_right(run_ring_next(s, step)),
{
    let t = run_ring_next(s, step);
    match step {
        RunStep::Write { slot, bytes } => {
            lemma_run_write_keeps_a_protected_view(s.head, s.disk, slot, bytes);
            assert forall|i: int| s.floor <= i < s.roots.len() implies run_view_reads_right(
                #[trigger] s.roots[i], s.disk.insert(slot, bytes)) by {
                lemma_run_write_keeps_a_protected_view(s.roots[i], s.disk, slot, bytes);
            }
            assert forall|i: int| 0 <= i < s.readers.len() implies run_view_reads_right(
                #[trigger] s.readers[i], s.disk.insert(slot, bytes)) by {
                lemma_run_write_keeps_a_protected_view(s.readers[i], s.disk, slot, bytes);
            }
        },
        RunStep::Reclaim { slot } => {
            lemma_run_write_keeps_a_protected_view(s.head, s.disk, slot, Seq::empty());
            assert forall|i: int| s.floor <= i < s.roots.len() implies run_view_reads_right(
                #[trigger] s.roots[i], s.disk.remove(slot)) by {
                lemma_run_write_keeps_a_protected_view(s.roots[i], s.disk, slot, Seq::empty());
            }
            assert forall|i: int| 0 <= i < s.readers.len() implies run_view_reads_right(
                #[trigger] s.readers[i], s.disk.remove(slot)) by {
                lemma_run_write_keeps_a_protected_view(s.readers[i], s.disk, slot, Seq::empty());
            }
        },
        RunStep::Relocate { key, destination } => {
            let moved = s.head.mapping.insert(key, destination);
            assert(moved.dom() =~= s.head.mapping.dom());
            assert forall|k: u64| moved.contains_key(k) implies
                run_is_on_disk(s.disk, #[trigger] moved[k]) && s.head.logical[k] == run_bytes(s.disk, moved[k]) by {
                if k != key { assert(moved[k] == s.head.mapping[k]); }
                else { assert(moved[k] == destination); }
            }
        },
        RunStep::UserPublish { new_view } => {},
        RunStep::Checkpoint => {
            let floor_after = run_floor_after_a_checkpoint(s);
            assert forall|i: int| floor_after <= i < t.roots.len() implies run_view_reads_right(
                #[trigger] t.roots[i], t.disk) by {
                if i < s.roots.len() { assert(t.roots[i] == s.roots[i]); }
                else { assert(t.roots[i] == s.head); }
            }
        },
        RunStep::Rollback { target } => {
            assert(run_view_reads_right(s.roots[target], s.disk));
        },
        RunStep::RaiseFloor { new_floor } => {},
        RunStep::AcquireReader => {
            assert forall|i: int| 0 <= i < t.readers.len() implies run_view_reads_right(
                #[trigger] t.readers[i], t.disk) by {
                if i < s.readers.len() { assert(t.readers[i] == s.readers[i]); }
                else { assert(t.readers[i] == s.head); }
            }
        },
        RunStep::ReleaseReader { index } => {
            assert forall|i: int| 0 <= i < t.readers.len() implies run_view_reads_right(
                #[trigger] t.readers[i], t.disk) by {
                if i < index { assert(t.readers[i] == s.readers[i]); }
                else { assert(t.readers[i] == s.readers[i + 1]); }
            }
        },
        RunStep::Crash => {
            assert(run_view_reads_right(s.roots[s.roots.len() - 1], s.disk));
        },
    }
}

pub proof fn theorem_run_ring_every_finite_history_keeps_every_view(
    states: Seq<RunRingState>, steps: Seq<RunStep>, count: nat,
)
    requires
        states.len() == steps.len() + 1,
        count < states.len(),
        run_ring_all_views_read_right(states[0]),
        forall|i: int| 0 <= i < steps.len() ==>
            run_step_enabled(#[trigger] states[i], steps[i]) && states[i + 1] == run_ring_next(states[i], steps[i]),
    ensures
        forall|i: int| 0 <= i <= count ==> run_ring_all_views_read_right(#[trigger] states[i]),
    decreases count,
{
    if count > 0 {
        theorem_run_ring_every_finite_history_keeps_every_view(states, steps, (count - 1) as nat);
        lemma_run_ring_step_preserves_every_view(states[count - 1], steps[count - 1]);
        assert(states[count as int] == run_ring_next(states[count - 1], steps[count - 1]));
    }
}

// ===== Z1 变体一：目的写落在回退目标还在读的槽（单槽）=====
// 现行那一版只有 key 7 在槽 10；回退目标（用户删掉 key 4 之前）里 key 4 在槽 20。往槽 20 写不合法。
// ===== Z1 变体二：回退目标里 key 4 是两槽单元（槽 20、21），往槽 21 写也不合法 =====
pub proof fn variants_one_and_two_are_refused_by_the_run_protocol()
    ensures
        ({
            let current = RunView {
                mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }),
                logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]),
            };
            let target = RunView {
                mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 })
                    .insert(4u64, SlotRun { start: 20, span: 2 }),
                logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]])
                    .insert(4u64, seq![seq![99u8], seq![98u8]]),
            };
            let s = RunRingState {
                disk: Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![99u8]).insert(21int, seq![98u8]),
                roots: seq![target, current], floor: 0, head: current, readers: Seq::empty(), window: 2,
            };
            &&& run_ring_all_views_read_right(s)
            &&& !run_step_enabled(s, RunStep::Write { slot: 20, bytes: seq![42u8] })
            &&& !run_step_enabled(s, RunStep::Write { slot: 21, bytes: seq![42u8] })
            &&& !run_step_enabled(s, RunStep::Reclaim { slot: 21 })
            &&& run_step_enabled(s, RunStep::Write { slot: 30, bytes: seq![42u8] })
        }),
{
    let current = RunView {
        mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }),
        logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]),
    };
    let target = RunView {
        mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 })
            .insert(4u64, SlotRun { start: 20, span: 2 }),
        logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]])
            .insert(4u64, seq![seq![99u8], seq![98u8]]),
    };
    let s = RunRingState {
        disk: Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]).insert(20int, seq![99u8]).insert(21int, seq![98u8]),
        roots: seq![target, current], floor: 0, head: current, readers: Seq::empty(), window: 2,
    };
    // 见证：这个状态满足不变量（防空真）。
    assert(target.mapping.dom() =~= target.logical.dom());
    assert(current.mapping.dom() =~= current.logical.dom());
    assert(run_is_on_disk(s.disk, target.mapping[7u64]));
    assert(run_is_on_disk(s.disk, target.mapping[4u64])) by {
        assert forall|offset: int| 0 <= offset < 2 implies s.disk.contains_key(#[trigger] (20int + offset)) by {
            assert(offset == 0 || offset == 1);
        }
    }
    assert(run_is_on_disk(s.disk, current.mapping[7u64]));
    assert(run_bytes(s.disk, target.mapping[7u64]) =~= seq![seq![42u8]]);
    assert(run_bytes(s.disk, target.mapping[4u64]) =~= seq![seq![99u8], seq![98u8]]);
    assert(run_bytes(s.disk, current.mapping[7u64]) =~= seq![seq![42u8]]);
    assert(run_view_reads_right(target, s.disk));
    assert(run_view_reads_right(current, s.disk));
    assert(run_ring_all_views_read_right(s));
    // 槽 20、21 都被候选 0（target）的区间罩着。
    assert(s.roots[0] == target);
    assert(run_covers(target.mapping[4u64], 20int));
    assert(run_covers(target.mapping[4u64], 21int));
    assert(!run_map_avoids(s.roots[0].mapping, 20int));
    assert(!run_map_avoids(s.roots[0].mapping, 21int));
    // 槽 30 谁也不罩：写得进去。
    assert(run_slot_is_unreferenced(s, 30int)) by {
        assert forall|key: u64| current.mapping.contains_key(key) implies !run_covers(#[trigger] current.mapping[key], 30int) by {
            assert(key == 7u64);
        }
        assert forall|i: int| s.floor <= i < s.roots.len() implies run_map_avoids(#[trigger] s.roots[i].mapping, 30int) by {
            if i == 0 {
                assert forall|key: u64| target.mapping.contains_key(key) implies !run_covers(#[trigger] target.mapping[key], 30int) by {
                    assert(key == 7u64 || key == 4u64);
                }
            } else {
                assert(i == 1);
                assert forall|key: u64| current.mapping.contains_key(key) implies !run_covers(#[trigger] current.mapping[key], 30int) by {
                    assert(key == 7u64);
                }
            }
        }
    }
}

} // verus!
