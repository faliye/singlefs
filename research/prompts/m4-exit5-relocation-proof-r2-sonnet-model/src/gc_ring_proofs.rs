//! GC 搬迁协议的第二版：持久的回退目标视图、多个逻辑版本、根环窗口与 F。
//!
//! 第一轮攻方对 `gc_crash_proofs` 打中的两处：协议里只有「当前根、持久根、留存读者」三类视图、读者都共享同一个 `logical`，
//! 所以 (一) 搬迁、checkpoint 之后立刻回收并覆写源槽，每步都合法而回退要复活的旧版映射读坏；
//! (二) 把回退目标当读者塞进去，只要它是另一个逻辑版本（用户在它之后覆盖写过），`gc_all_views_match` 对任何 `logical` 都假。
//! 这一版：
//!   - 每个视图带自己的逻辑内容（`RootView { mapping, logical }`）；
//!   - 根环里的持久根是视图的一类，Crash 不清它；回退候选是 `roots[floor..]`（F 之上的根）；
//!   - 回收与覆写要避开现行、每个回退候选、每个读者三类视图引用的槽；
//!   - Checkpoint 把现行版本追加成新根，候选数超过窗口就把最旧的候选挤出去（floor 加一）——这是整理的发布占根环槽的那一件事；
//!   - 管理员回退把现行版本换成某个候选根的视图；抬 F 只许往上抬。
//! 恒等的主语因此是：现行版本的逻辑内容、每个仍是候选的根的逻辑内容；整理的 checkpoint 对候选窗口的唯一效应是窗口移一格。
//! 协议层模型，不是 core 的内存布局：槽是原子地址，没有设备编号、span、journal 编码、屏障。
use vstd::prelude::*;

verus! {

pub struct RootView {
    pub mapping: Map<u64, u64>,
    pub logical: Map<u64, Seq<u8>>,
}

pub struct GcRingState {
    pub disk: Map<u64, Seq<u8>>,
    // 根环里的持久根，旧到新；最后一个是持久的现行根（崩溃后重现的那一个）。
    pub roots: Seq<RootView>,
    // 回退候选是 roots[floor..]；floor 只增不减（F 只许往上抬）。
    pub floor: nat,
    // 内存里的现行版本，可以领先最后一个持久根（发布还没 checkpoint）。
    pub head: RootView,
    // 进程内的留存读者，各带自己的逻辑版本；Crash 丢掉它们。
    pub readers: Seq<RootView>,
    // 根环能容的回退候选数，恒大于 0。
    pub window: nat,
}

pub enum RingStep {
    Write { slot: u64, bytes: Seq<u8> },
    Reclaim { slot: u64 },
    Relocate { key: u64, destination: u64 },
    UserPublish { new_view: RootView },
    Checkpoint,
    Rollback { target: int },
    RaiseFloor { new_floor: nat },
    AcquireReader,
    ReleaseReader { index: int },
    Crash,
}

pub open spec fn ring_mapping_matches(
    mapping: Map<u64, u64>, disk: Map<u64, Seq<u8>>, logical: Map<u64, Seq<u8>>,
) -> bool {
    &&& mapping.dom() == logical.dom()
    &&& forall|key: u64| mapping.contains_key(key) ==>
        disk.contains_key(#[trigger] mapping[key]) && disk[mapping[key]] == logical[key]
}

pub open spec fn view_reads_right(view: RootView, disk: Map<u64, Seq<u8>>) -> bool {
    ring_mapping_matches(view.mapping, disk, view.logical)
}

pub open spec fn ring_state_is_shaped(s: GcRingState) -> bool {
    &&& s.roots.len() >= 1
    &&& s.floor < s.roots.len()
    &&& s.window >= 1
    &&& s.roots.len() - s.floor <= s.window
}

// 不变量：现行、每个回退候选、每个读者，各自读到自己的逻辑内容。
pub open spec fn ring_all_views_read_right(s: GcRingState) -> bool {
    &&& ring_state_is_shaped(s)
    &&& view_reads_right(s.head, s.disk)
    &&& forall|i: int| s.floor <= i < s.roots.len() ==> view_reads_right(#[trigger] s.roots[i], s.disk)
    &&& forall|i: int| 0 <= i < s.readers.len() ==> view_reads_right(#[trigger] s.readers[i], s.disk)
}

pub open spec fn ring_map_avoids(mapping: Map<u64, u64>, slot: u64) -> bool {
    forall|key: u64| mapping.contains_key(key) ==> #[trigger] mapping[key] != slot
}

// 回收与覆写的保护条件：现行、每个回退候选（F 之上的根）、每个读者引用的槽都不许动。
pub open spec fn ring_slot_is_unreferenced(s: GcRingState, slot: u64) -> bool {
    &&& ring_map_avoids(s.head.mapping, slot)
    &&& forall|i: int| s.floor <= i < s.roots.len() ==> ring_map_avoids(#[trigger] s.roots[i].mapping, slot)
    &&& forall|i: int| 0 <= i < s.readers.len() ==> ring_map_avoids(#[trigger] s.readers[i].mapping, slot)
}

pub open spec fn ring_step_enabled(s: GcRingState, step: RingStep) -> bool {
    match step {
        RingStep::Write { slot, .. } | RingStep::Reclaim { slot } => ring_slot_is_unreferenced(s, slot),
        RingStep::Relocate { key, destination } => {
            &&& s.head.mapping.contains_key(key)
            &&& s.disk.contains_key(s.head.mapping[key])
            &&& s.disk.contains_key(destination)
            &&& s.disk[destination] == s.disk[s.head.mapping[key]]
        },
        RingStep::UserPublish { new_view } => view_reads_right(new_view, s.disk),
        RingStep::Rollback { target } => s.floor <= target < s.roots.len(),
        RingStep::RaiseFloor { new_floor } => s.floor <= new_floor < s.roots.len(),
        RingStep::ReleaseReader { index } => 0 <= index < s.readers.len(),
        _ => true,
    }
}

// Checkpoint 之后候选数超过窗口，就把最旧的候选挤出去。
pub open spec fn floor_after_a_checkpoint(s: GcRingState) -> nat {
    if s.roots.len() + 1 - s.floor > s.window { (s.floor + 1) as nat } else { s.floor }
}

pub open spec fn ring_next(s: GcRingState, step: RingStep) -> GcRingState {
    match step {
        RingStep::Write { slot, bytes } => GcRingState { disk: s.disk.insert(slot, bytes), ..s },
        RingStep::Reclaim { slot } => GcRingState { disk: s.disk.remove(slot), ..s },
        RingStep::Relocate { key, destination } => GcRingState {
            head: RootView { mapping: s.head.mapping.insert(key, destination), logical: s.head.logical },
            ..s
        },
        RingStep::UserPublish { new_view } => GcRingState { head: new_view, ..s },
        RingStep::Checkpoint => GcRingState {
            roots: s.roots.push(s.head),
            floor: floor_after_a_checkpoint(s),
            ..s
        },
        RingStep::Rollback { target } => GcRingState { head: s.roots[target], ..s },
        RingStep::RaiseFloor { new_floor } => GcRingState { floor: new_floor, ..s },
        RingStep::AcquireReader => GcRingState { readers: s.readers.push(s.head), ..s },
        RingStep::ReleaseReader { index } => GcRingState { readers: s.readers.remove(index), ..s },
        RingStep::Crash => GcRingState { head: s.roots[s.roots.len() - 1], readers: Seq::empty(), ..s },
    }
}

proof fn lemma_ring_write_keeps_a_protected_view(
    view: RootView, disk: Map<u64, Seq<u8>>, slot: u64, bytes: Seq<u8>,
)
    requires
        view_reads_right(view, disk),
        ring_map_avoids(view.mapping, slot),
    ensures
        view_reads_right(view, disk.insert(slot, bytes)),
        view_reads_right(view, disk.remove(slot)),
{
}

pub proof fn lemma_ring_step_preserves_every_view(s: GcRingState, step: RingStep)
    requires
        ring_all_views_read_right(s),
        ring_step_enabled(s, step),
    ensures
        ring_all_views_read_right(ring_next(s, step)),
{
    let t = ring_next(s, step);
    match step {
        RingStep::Write { slot, bytes } => {
            lemma_ring_write_keeps_a_protected_view(s.head, s.disk, slot, bytes);
            assert forall|i: int| s.floor <= i < s.roots.len() implies view_reads_right(
                #[trigger] s.roots[i], s.disk.insert(slot, bytes)) by {
                lemma_ring_write_keeps_a_protected_view(s.roots[i], s.disk, slot, bytes);
            }
            assert forall|i: int| 0 <= i < s.readers.len() implies view_reads_right(
                #[trigger] s.readers[i], s.disk.insert(slot, bytes)) by {
                lemma_ring_write_keeps_a_protected_view(s.readers[i], s.disk, slot, bytes);
            }
        },
        RingStep::Reclaim { slot } => {
            lemma_ring_write_keeps_a_protected_view(s.head, s.disk, slot, Seq::empty());
            assert forall|i: int| s.floor <= i < s.roots.len() implies view_reads_right(
                #[trigger] s.roots[i], s.disk.remove(slot)) by {
                lemma_ring_write_keeps_a_protected_view(s.roots[i], s.disk, slot, Seq::empty());
            }
            assert forall|i: int| 0 <= i < s.readers.len() implies view_reads_right(
                #[trigger] s.readers[i], s.disk.remove(slot)) by {
                lemma_ring_write_keeps_a_protected_view(s.readers[i], s.disk, slot, Seq::empty());
            }
        },
        RingStep::Relocate { key, destination } => {
            let moved = s.head.mapping.insert(key, destination);
            assert(moved.dom() =~= s.head.mapping.dom());
            assert forall|k: u64| moved.contains_key(k) implies
                s.disk.contains_key(#[trigger] moved[k]) && s.disk[moved[k]] == s.head.logical[k] by {
                if k != key { assert(moved[k] == s.head.mapping[k]); }
            }
        },
        RingStep::UserPublish { new_view } => {},
        RingStep::Checkpoint => {
            let floor_after = floor_after_a_checkpoint(s);
            assert forall|i: int| floor_after <= i < t.roots.len() implies view_reads_right(
                #[trigger] t.roots[i], t.disk) by {
                if i < s.roots.len() { assert(t.roots[i] == s.roots[i]); }
                else { assert(t.roots[i] == s.head); }
            }
        },
        RingStep::Rollback { target } => {
            assert(view_reads_right(s.roots[target], s.disk));
        },
        RingStep::RaiseFloor { new_floor } => {},
        RingStep::AcquireReader => {
            assert forall|i: int| 0 <= i < t.readers.len() implies view_reads_right(
                #[trigger] t.readers[i], t.disk) by {
                if i < s.readers.len() { assert(t.readers[i] == s.readers[i]); }
                else { assert(t.readers[i] == s.head); }
            }
        },
        RingStep::ReleaseReader { index } => {
            assert forall|i: int| 0 <= i < t.readers.len() implies view_reads_right(
                #[trigger] t.readers[i], t.disk) by {
                if i < index { assert(t.readers[i] == s.readers[i]); }
                else { assert(t.readers[i] == s.readers[i + 1]); }
            }
        },
        RingStep::Crash => {
            assert(view_reads_right(s.roots[s.roots.len() - 1], s.disk));
        },
    }
}

// 任意有限历史（含崩溃、回退、抬 F、挤窗口、不同逻辑版本的读者与根）之后，每个候选与每个读者仍读到自己的逻辑内容。
pub proof fn theorem_ring_every_finite_history_keeps_every_view(
    states: Seq<GcRingState>, steps: Seq<RingStep>, count: nat,
)
    requires
        states.len() == steps.len() + 1,
        count < states.len(),
        ring_all_views_read_right(states[0]),
        forall|i: int| 0 <= i < steps.len() ==>
            ring_step_enabled(#[trigger] states[i], steps[i]) && states[i + 1] == ring_next(states[i], steps[i]),
    ensures
        forall|i: int| 0 <= i <= count ==> ring_all_views_read_right(#[trigger] states[i]),
    decreases count,
{
    if count > 0 {
        theorem_ring_every_finite_history_keeps_every_view(states, steps, (count - 1) as nat);
        lemma_ring_step_preserves_every_view(states[count - 1], steps[count - 1]);
        assert(states[count as int] == ring_next(states[count - 1], steps[count - 1]));
    }
}

// ===== 第一轮攻方反例的排除 =====

// 回退候选引用的槽，回收与覆写都不合法：搬迁、checkpoint 之后，旧版（搬迁之前的根）只要还在窗口里，它的源槽就回收不了。
pub proof fn lemma_the_source_of_a_rollback_candidate_cannot_be_reclaimed(
    s: GcRingState, candidate: int, key: u64,
)
    requires
        s.floor <= candidate < s.roots.len(),
        s.roots[candidate].mapping.contains_key(key),
    ensures
        !ring_step_enabled(s, RingStep::Reclaim { slot: s.roots[candidate].mapping[key] }),
        !ring_step_enabled(s, RingStep::Write { slot: s.roots[candidate].mapping[key], bytes: Seq::empty() }),
{
    assert(!ring_map_avoids(s.roots[candidate].mapping, s.roots[candidate].mapping[key]));
}

// Crash 不动根环与 floor，候选在崩溃之后仍然保护着自己的槽（第一轮「读者被 Crash 清空后源槽就能回收」的变体）。
pub proof fn lemma_a_crash_keeps_the_rollback_candidates_protecting_their_slots(s: GcRingState)
    ensures
        ring_next(s, RingStep::Crash).roots == s.roots,
        ring_next(s, RingStep::Crash).floor == s.floor,
{
}

// 具体重放：第一轮攻方的五步（写目的地、换映射、checkpoint、回收源槽、覆写源槽），窗口容得下两个候选时，第四步不合法。
pub proof fn lemma_the_first_round_sequence_stops_at_the_reclaim_when_the_old_root_is_still_a_candidate()
    ensures
        ({
            let bytes = seq![42u8];
            let logical = Map::<u64, Seq<u8>>::empty().insert(7u64, bytes);
            let before = RootView { mapping: Map::<u64, u64>::empty().insert(7u64, 10u64), logical };
            let s0 = GcRingState {
                disk: Map::<u64, Seq<u8>>::empty().insert(10u64, bytes),
                roots: seq![before], floor: 0, head: before, readers: Seq::empty(), window: 2,
            };
            let s1 = ring_next(s0, RingStep::Write { slot: 20, bytes });
            let s2 = ring_next(s1, RingStep::Relocate { key: 7, destination: 20 });
            let s3 = ring_next(s2, RingStep::Checkpoint);
            &&& ring_step_enabled(s0, RingStep::Write { slot: 20, bytes })
            &&& ring_step_enabled(s1, RingStep::Relocate { key: 7, destination: 20 })
            &&& ring_step_enabled(s2, RingStep::Checkpoint)
            &&& s3.floor == 0
            &&& !ring_step_enabled(s3, RingStep::Reclaim { slot: 10 })
        }),
{
    let bytes = seq![42u8];
    let logical = Map::<u64, Seq<u8>>::empty().insert(7u64, bytes);
    let before = RootView { mapping: Map::<u64, u64>::empty().insert(7u64, 10u64), logical };
    let s0 = GcRingState {
        disk: Map::<u64, Seq<u8>>::empty().insert(10u64, bytes),
        roots: seq![before], floor: 0, head: before, readers: Seq::empty(), window: 2,
    };
    assert(view_reads_right(before, s0.disk));
    assert(ring_all_views_read_right(s0));
    assert(ring_map_avoids(before.mapping, 20u64)) by {
        assert forall|k: u64| before.mapping.contains_key(k) implies #[trigger] before.mapping[k] != 20u64 by {
            assert(k == 7u64);
        }
    }
    assert(ring_slot_is_unreferenced(s0, 20));
    let s1 = ring_next(s0, RingStep::Write { slot: 20, bytes });
    assert(s1.disk[20u64] == bytes);
    assert(s1.disk[s1.head.mapping[7u64]] == bytes);
    let s2 = ring_next(s1, RingStep::Relocate { key: 7, destination: 20 });
    let s3 = ring_next(s2, RingStep::Checkpoint);
    assert(s2.roots.len() + 1 - s2.floor == 2);
    assert(floor_after_a_checkpoint(s2) == 0);
    assert(s3.roots[0] == before);
    assert(s3.roots[0].mapping.contains_key(7u64));
    assert(s3.roots[0].mapping[7u64] == 10u64);
    lemma_the_source_of_a_rollback_candidate_cannot_be_reclaimed(s3, 0, 7u64);
}

// 窗口只容一个候选时，同样的五步里第四步合法：旧根被挤出窗口，回退到不了它，所以回收它的源槽不读坏任何候选。
// 这正是整理的发布占根环槽的那件事：用户看得见的变化是窗口移了一格，不是读到坏字节。
pub proof fn lemma_with_a_window_of_one_the_old_root_leaves_and_its_source_becomes_reclaimable()
    ensures
        ({
            let bytes = seq![42u8];
            let logical = Map::<u64, Seq<u8>>::empty().insert(7u64, bytes);
            let before = RootView { mapping: Map::<u64, u64>::empty().insert(7u64, 10u64), logical };
            let after = RootView { mapping: Map::<u64, u64>::empty().insert(7u64, 20u64), logical };
            let s2 = GcRingState {
                disk: Map::<u64, Seq<u8>>::empty().insert(10u64, bytes).insert(20u64, bytes),
                roots: seq![before], floor: 0, head: after, readers: Seq::empty(), window: 1,
            };
            let s3 = ring_next(s2, RingStep::Checkpoint);
            &&& ring_all_views_read_right(s2)
            &&& s3.floor == 1
            &&& ring_step_enabled(s3, RingStep::Reclaim { slot: 10 })
            &&& !ring_step_enabled(s3, RingStep::Rollback { target: 0 })
        }),
{
    let bytes = seq![42u8];
    let logical = Map::<u64, Seq<u8>>::empty().insert(7u64, bytes);
    let before = RootView { mapping: Map::<u64, u64>::empty().insert(7u64, 10u64), logical };
    let after = RootView { mapping: Map::<u64, u64>::empty().insert(7u64, 20u64), logical };
    let s2 = GcRingState {
        disk: Map::<u64, Seq<u8>>::empty().insert(10u64, bytes).insert(20u64, bytes),
        roots: seq![before], floor: 0, head: after, readers: Seq::empty(), window: 1,
    };
    assert(view_reads_right(before, s2.disk));
    assert(view_reads_right(after, s2.disk));
    assert(ring_all_views_read_right(s2));
    let s3 = ring_next(s2, RingStep::Checkpoint);
    assert(floor_after_a_checkpoint(s2) == 1);
    assert(s3.floor == 1);
    assert(s3.roots.len() == 2);
    assert(s3.roots[1] == after);
    assert(ring_map_avoids(s3.head.mapping, 10u64)) by {
        assert forall|k: u64| s3.head.mapping.contains_key(k) implies #[trigger] s3.head.mapping[k] != 10u64 by {
            assert(k == 7u64);
        }
    }
    assert(ring_map_avoids(s3.roots[1].mapping, 10u64)) by {
        assert forall|k: u64| s3.roots[1].mapping.contains_key(k) implies #[trigger] s3.roots[1].mapping[k] != 10u64 by {
            assert(k == 7u64);
        }
    }
    assert(ring_slot_is_unreferenced(s3, 10));
}

// 多个逻辑版本的见证：两个根、key 集合不同（用户在旧根之后覆盖写出新 key），不变量照样成立——第一轮「前提对任何 logical 都假」不再发生。
pub proof fn witness_two_roots_with_different_logical_versions_satisfy_the_invariant()
    ensures
        exists|s: GcRingState|
            ring_all_views_read_right(s) && s.roots.len() == 2 && s.floor == 0
                && s.roots[0].logical.dom() != s.roots[1].logical.dom(),
{
    let b1 = seq![1u8];
    let b2 = seq![2u8];
    let old_view = RootView {
        mapping: Map::<u64, u64>::empty().insert(7u64, 10u64),
        logical: Map::<u64, Seq<u8>>::empty().insert(7u64, b1),
    };
    let new_view = RootView {
        mapping: Map::<u64, u64>::empty().insert(7u64, 10u64).insert(8u64, 11u64),
        logical: Map::<u64, Seq<u8>>::empty().insert(7u64, b1).insert(8u64, b2),
    };
    let s = GcRingState {
        disk: Map::<u64, Seq<u8>>::empty().insert(10u64, b1).insert(11u64, b2),
        roots: seq![old_view, new_view], floor: 0, head: new_view, readers: seq![old_view], window: 2,
    };
    assert(view_reads_right(old_view, s.disk)) by {
        assert(old_view.mapping.dom() =~= old_view.logical.dom());
    }
    assert(view_reads_right(new_view, s.disk)) by {
        assert(new_view.mapping.dom() =~= new_view.logical.dom());
        assert forall|k: u64| new_view.mapping.contains_key(k) implies
            s.disk.contains_key(#[trigger] new_view.mapping[k]) && s.disk[new_view.mapping[k]] == new_view.logical[k] by {
            if k == 8u64 { assert(new_view.mapping[k] == 11u64); } else { assert(k == 7u64); assert(new_view.mapping[k] == 10u64); }
        }
    }
    assert(ring_all_views_read_right(s));
    assert(!(old_view.logical.dom() =~= new_view.logical.dom())) by {
        assert(new_view.logical.dom().contains(8u64));
        assert(!old_view.logical.dom().contains(8u64));
    }
    assert(s.roots[0].logical.dom() != s.roots[1].logical.dom());
}

// ===== G7 的改写：整理对「用户看得见的东西」的效应 =====
// 用户看得见的 = 现行版本的逻辑内容、回退候选窗口里各根的逻辑内容（按新旧次序）、能抬到的 F 上限（最后一个根的下标）。

pub open spec fn window_logicals(s: GcRingState) -> Seq<Map<u64, Seq<u8>>> {
    s.roots.subrange(s.floor as int, s.roots.len() as int).map_values(|view: RootView| view.logical)
}

pub open spec fn raise_floor_ceiling(s: GcRingState) -> int {
    s.roots.len() - 1
}

// 整理里不发布的三步（写目的地、换映射、回收）：现行版本的逻辑内容、根环、floor 都不动，窗口原样。
pub proof fn lemma_the_non_publishing_gc_steps_change_nothing_the_user_sees(s: GcRingState, step: RingStep)
    requires
        step is Write || step is Reclaim || step is Relocate,
    ensures
        ring_next(s, step).head.logical == s.head.logical,
        ring_next(s, step).roots == s.roots,
        ring_next(s, step).floor == s.floor,
        window_logicals(ring_next(s, step)) == window_logicals(s),
{
}

// 每个步骤都不改已经在环里的根：整理的任何一步都不会把旧根的逻辑内容改掉。
pub proof fn lemma_no_step_rewrites_a_root_already_in_the_ring(s: GcRingState, step: RingStep, i: int)
    requires
        0 <= i < s.roots.len(),
    ensures
        ring_next(s, step).roots.len() >= s.roots.len(),
        ring_next(s, step).roots[i] == s.roots[i],
{
    if step is Checkpoint {
        assert(ring_next(s, step).roots[i] == s.roots[i]);
    }
}

// 整理的发布（现行版本的逻辑内容与最后一个根相同，即没有用户的改动在飞）：现行逻辑内容不变；窗口只有一件事变——
// 新根追加在末尾，候选超过窗口时最旧的候选被挤出。没挤时原窗口是新窗口的前缀（什么都没少）；挤了时少的恰是最旧的一个。
pub proof fn lemma_a_gc_checkpoint_only_shifts_the_window(s: GcRingState)
    requires
        ring_state_is_shaped(s),
        s.head.logical == s.roots[s.roots.len() - 1].logical,
    ensures
        ring_next(s, RingStep::Checkpoint).head.logical == s.head.logical,
        s.roots.len() + 1 - s.floor <= s.window ==> window_logicals(ring_next(s, RingStep::Checkpoint))
            == window_logicals(s).push(s.head.logical),
        s.roots.len() + 1 - s.floor > s.window ==> window_logicals(ring_next(s, RingStep::Checkpoint))
            == window_logicals(s).push(s.head.logical).drop_first(),
        raise_floor_ceiling(ring_next(s, RingStep::Checkpoint)) == raise_floor_ceiling(s) + 1,
{
    let t = ring_next(s, RingStep::Checkpoint);
    if s.roots.len() + 1 - s.floor <= s.window {
        assert(window_logicals(t) =~= window_logicals(s).push(s.head.logical));
    } else {
        assert(window_logicals(t) =~= window_logicals(s).push(s.head.logical).drop_first());
    }
}

// 整理的发布挤掉的候选至多一个，且只有窗口满时才挤：窗口没满，回退能到的逻辑版本一个不少。
pub proof fn lemma_a_gc_checkpoint_with_room_in_the_window_loses_no_rollback_target(s: GcRingState, i: int)
    requires
        ring_state_is_shaped(s),
        s.head.logical == s.roots[s.roots.len() - 1].logical,
        s.roots.len() + 1 - s.floor <= s.window,
        0 <= i < window_logicals(s).len(),
    ensures
        window_logicals(ring_next(s, RingStep::Checkpoint))[i] == window_logicals(s)[i],
{
    lemma_a_gc_checkpoint_only_shifts_the_window(s);
}

// 反过来：窗口满的时候，整理的一次发布使窗口少掉最旧的那个候选（用户看得见的变化：回退少一个落点）。
// 这条是第一轮攻方「整理改了回退能退回的内容集合」的精确表述：不是读到坏字节，是窗口移一格。
pub proof fn lemma_a_gc_checkpoint_with_a_full_window_drops_exactly_the_oldest_rollback_target(s: GcRingState, i: int)
    requires
        ring_state_is_shaped(s),
        s.head.logical == s.roots[s.roots.len() - 1].logical,
        s.roots.len() + 1 - s.floor > s.window,
        0 <= i < window_logicals(s).len() - 1,
    ensures
        window_logicals(ring_next(s, RingStep::Checkpoint))[i] == window_logicals(s)[i + 1],
{
    lemma_a_gc_checkpoint_only_shifts_the_window(s);
}

} // verus!
