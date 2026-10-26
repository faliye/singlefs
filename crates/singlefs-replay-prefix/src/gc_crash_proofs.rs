//! GC 回收门槛：产品执行的比较核，与「门槛蕴涵没被引用」那一条引理。
//!
//! # 证明入口与范围
//! 只有 [`released_record_can_be_reclaimed`] 被产品执行（`singlefs-core` 的 `allocator.rs` 回收释放了的记录时调它）。
//! 搬迁协议（根环按区间、内存 F 与持久 F 两层、回退候选、多逻辑版本、读者、崩溃）在 `gc_ring_interval_proofs`：那里写与回收的门是
//! 「现行、持久 F 之上的每个根、每个读者都没有区间罩住这个槽」（`slot_is_unreferenced`）。
//! 这里的 `lemma_the_reclaim_threshold_leaves_the_released_run_unreferenced` 把产品的门槛 `released && generation <= floor`
//! 接到那一条上：两条前提成立时，门槛放行的记录，它区间里的每个槽都没被引用。两条前提是分配器与 floor 的来历要另证的义务，
//! 不在这里证：
//! - 释放代的含义：版本（txg）不早于释放代的受保护视图都不碰这条记录的区间（`the_release_generation_hides_the_run`）；
//! - floor 的下界：floor 不大于现行、持久 F 之上每个根、每个读者的版本（`the_floor_is_at_most_every_protected_version`）。
//!   core 里 floor 取回退 F、隔离与「每块盘上最新有效根」之下的最小者（`mount.rs` 的抬 F 序列与挂载重建分配器之后的回收），
//!   这一条要从那几处证出来；读者的版本进不进 floor 也在那一侧。
//!
//! 两条前提与门槛三样一起才推得出「没被引用」；去掉任何一条都推不出（`verus-must-red-worlds.tsv` 里这一条引理的几行）。
#[allow(
    unused_imports,
    reason = "只有 Verus 编译时留下的证明代码用这些名字；普通构建擦掉证明代码之后没人用，擦掉的 spec fn 也不在，所以整模块导入"
)]
use crate::gc_ring_interval_proofs::*;
use vstd::prelude::*;

verus! {

// 一个映射不碰这个区间：映射里没有一个区间与它重叠。
pub open spec fn mapping_avoids_run(mapping: Map<u64, SlotRun>, run: SlotRun) -> bool {
    forall|key: u64| mapping.contains_key(key) ==> !runs_overlap(#[trigger] mapping[key], run)
}

// 受保护的每个视图的版本（txg）：现行一个、根环里每个根一个、每个读者一个，下标与 RunRingState 的根与读者对齐。
pub struct ProtectedViewVersions {
    pub head: u64,
    pub roots: Seq<u64>,
    pub readers: Seq<u64>,
}

pub open spec fn versions_line_up_with_the_ring(state: RunRingState, versions: ProtectedViewVersions) -> bool {
    &&& versions.roots.len() == state.roots.len()
    &&& versions.readers.len() == state.readers.len()
}

// 释放代的含义（分配器那一侧的义务）：这条记录在 generation 那一次发布里被释放，版本不早于它的受保护视图都不碰它的区间。
pub open spec fn the_release_generation_hides_the_run(
    state: RunRingState,
    versions: ProtectedViewVersions,
    run: SlotRun,
    generation: u64,
) -> bool {
    &&& generation <= versions.head ==> mapping_avoids_run(state.head.mapping, run)
    &&& forall|index: int| state.persisted_floor <= index < state.roots.len() && generation <= #[trigger] versions.roots[index]
        ==> mapping_avoids_run(state.roots[index].view.mapping, run)
    &&& forall|index: int| 0 <= index < state.readers.len() && generation <= #[trigger] versions.readers[index]
        ==> mapping_avoids_run(state.readers[index].mapping, run)
}

// floor 的下界（floor 的来历要另证的义务）：不大于现行、持久 F 之上每个根、每个读者的版本。
pub open spec fn the_floor_is_at_most_every_protected_version(
    state: RunRingState,
    versions: ProtectedViewVersions,
    floor: u64,
) -> bool {
    &&& floor <= versions.head
    &&& forall|index: int| state.persisted_floor <= index < state.roots.len() ==> floor <= #[trigger] versions.roots[index]
    &&& forall|index: int| 0 <= index < state.readers.len() ==> floor <= #[trigger] versions.readers[index]
}

// 真实分配器调此判定。generation 是释放版本，floor 由根环/回退/隔离计算。
// 唯一产品可执行入口：保留 <= 的相等边界及 u64 全宽，不计算 floor、不改变回收时机。
pub fn released_record_can_be_reclaimed(released: bool, generation: u64, floor: u64) -> (answer: bool)
    ensures
        answer == (released && generation <= floor),
{
    released && generation <= floor
}

// 门槛蕴涵没被引用：门槛放行（释放了、释放代不大于 floor），两条前提成立时，这条记录区间里的每个槽，现行、持久 F 之上的每个根、
// 每个读者都没罩住，搬迁协议的写与回收两道门都放行它。每个受保护视图的版本不小于 floor、floor 不小于释放代，
// 所以版本不早于释放代，照释放代的含义不碰这个区间。
pub proof fn lemma_the_reclaim_threshold_leaves_the_released_run_unreferenced(
    state: RunRingState,
    versions: ProtectedViewVersions,
    released_run: SlotRun,
    released: bool,
    generation: u64,
    floor: u64,
)
    requires
        versions_line_up_with_the_ring(state, versions),
        the_release_generation_hides_the_run(state, versions, released_run, generation),
        the_floor_is_at_most_every_protected_version(state, versions, floor),
        released,
        generation <= floor,
    ensures
        forall|slot: int| run_covers(released_run, slot) ==> #[trigger] slot_is_unreferenced(state, slot),
{
    assert forall|slot: int| run_covers(released_run, slot) implies #[trigger] slot_is_unreferenced(state, slot) by {
        assert(mapping_avoids_run(state.head.mapping, released_run));
        assert forall|key: u64| state.head.mapping.contains_key(key) implies !run_covers(#[trigger] state.head.mapping[key], slot) by {
            assert(!runs_overlap(state.head.mapping[key], released_run));
        }
        assert forall|index: int| state.persisted_floor <= index < state.roots.len() implies mapping_avoids_slot(
            (#[trigger] state.roots[index]).view.mapping,
            slot,
        ) by {
            assert(floor <= versions.roots[index]);
            assert(mapping_avoids_run(state.roots[index].view.mapping, released_run));
            assert forall|key: u64| state.roots[index].view.mapping.contains_key(key) implies !run_covers(
                #[trigger] state.roots[index].view.mapping[key],
                slot,
            ) by {
                assert(!runs_overlap(state.roots[index].view.mapping[key], released_run));
            }
        }
        assert forall|index: int| 0 <= index < state.readers.len() implies mapping_avoids_slot(
            (#[trigger] state.readers[index]).mapping,
            slot,
        ) by {
            assert(floor <= versions.readers[index]);
            assert(mapping_avoids_run(state.readers[index].mapping, released_run));
            assert forall|key: u64| state.readers[index].mapping.contains_key(key) implies !run_covers(
                #[trigger] state.readers[index].mapping[key],
                slot,
            ) by {
                assert(!runs_overlap(state.readers[index].mapping[key], released_run));
            }
        }
    }
}

// 两条前提可以同时成立的见证（防空真）：现行与持久的根在槽 10，一个读者还拿着两槽单元 [30, 32) 那一版（版本 4），
// 这条单元在版本 5 释放；floor 取 4 时门槛不放行，floor 取 5 时读者的版本 4 低于 floor、floor 的下界那一条不成立，
// 读者释放之后 floor 取 5，两条前提都成立、门槛放行。
pub proof fn witness_the_reclaim_threshold_premises_hold_once_the_old_reader_is_released()
    ensures
        exists|state: RunRingState, versions: ProtectedViewVersions|
            #[trigger] the_release_generation_hides_the_run(state, versions, SlotRun { start: 30, span: 2 }, 5)
                && versions_line_up_with_the_ring(state, versions)
                && the_floor_is_at_most_every_protected_version(state, versions, 5) && state.roots.len() == 1,
{
    let view = RunView {
        mapping: Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 }),
        logical: Map::<u64, Seq<Seq<u8>>>::empty().insert(7u64, seq![seq![42u8]]),
    };
    let state = ring_state_of(
        Map::<int, Seq<u8>>::empty().insert(10int, seq![42u8]),
        seq![RingRoot { view, name: RootName::UserVersion { version: 0 } }],
        0,
        0,
        view,
        Seq::empty(),
        2,
        1,
    );
    let versions = ProtectedViewVersions { head: 6, roots: seq![5u64], readers: Seq::empty() };
    let run = SlotRun { start: 30, span: 2 };
    assert(mapping_avoids_run(view.mapping, run)) by {
        assert forall|key: u64| view.mapping.contains_key(key) implies !runs_overlap(#[trigger] view.mapping[key], run) by {
            assert(key == 7u64);
        }
    }
    assert(state.roots[0].view == view);
    assert(versions.roots[0] == 5u64);
    assert(the_release_generation_hides_the_run(state, versions, run, 5));
    assert(the_floor_is_at_most_every_protected_version(state, versions, 5));
}

} // verus!
