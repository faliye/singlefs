//! GC 搬迁协议的根环版，映射值是槽区间（起点加跨度），F 分内存与持久两层，根按版本号点名。
//!
//! 出处： 三行与第三节「抽象状态」「G7」，
//! D13（验证路线）已定项 18。协议是区间版根环，
//! 另补三样：
//! - 「内存 F / 持久 F」两层：回退候选是内存 F 之上的根；回收与覆写要避开持久 F 之上的每个根。抬 F 先只改内存 F，
//!   下一次用户那一侧的发布把它带进持久的根，持久 F 才跟上；崩溃让内存 F 回到持久 F。core 里对应的是回收出来的槽先扣住、带新 F 的根落满每块盘
//!   之后才放开（`singlefs-core` 的 `allocator.rs` 里 `hold_until_floor_takes_effect` 与 `release_reclaim_holds`）。
//!   整理的发布只带持久 F、不带内存里抬过的：它落成根时只在根数超过容量时把持久 F 挤上一格，内存里抬过的 F
//!   照旧只由用户那一侧的发布落盘。core 里整理的发布带上一版的根带着的 F（`singlefs-core` 的 `transaction.rs` 里
//!   `publish_compaction_step_after_its_check` 那一处 `rollback_floor: previous.root.rollback_floor`）。
//! - 根按版本号点名：用户发布的根带它的版本号，整理的发布占一个根，但没有用户点得到的版本号；回退与抬 F 都按版本号点名。
//! - 多个逻辑版本：现行、每个根、每个读者各带自己的逻辑内容。
//!
//! 本模块证三样：
//! - 任意有限历史（写、回收、搬迁、用户发布、两种发布、按版本号回退、只在内存里抬 F、读者取得与释放、崩溃）之后，
//!   现行、持久 F 之上的每个根、每个读者都读到自己的逻辑内容（`theorem_every_finite_run_ring_history_keeps_every_protected_view`）。
//! - 三个反例被拒：两槽单元第二槽、整理发布之后回收源槽，与只看内存 F 就放开回收那一种。
//! - G7：整理不改现行内容；整理那份的回退候选（按版本号）是不整理那份的后缀、至多少 k 个，抬 F 的上限恰多 k
//!   （`theorem_compaction_keeps_what_the_user_sees`，k 是这一段整理占的发布数）。历史里可以有崩溃：崩溃之后回退候选照样是不整理那份的后缀、
//!   至多少 k 个，现行内容照样相同；整理的发布只在现行内容与最后一个根相同时发（整理不把用户还没落成根的改动先落盘）：
//!   这一条是根环协议里整理那一步的门，投影引理把它带到 G7 那一层（G7 要的「整理那份也接受每一步」不靠协议外的假设）。
//!
//! 槽号与跨度用 int（协议层的数学整数）；接 core 的 u64 时要另证起点加跨度不溢出。这是协议模型，不是 core 的内存布局：
//! 没有设备编号、根环槽、journal 编码与屏障。回收门槛 `generation <= floor` 蕴涵这里的「没被引用」那一条在 `gc_crash_proofs`。
use vstd::prelude::*;

mod compaction_keeps_what_the_user_sees;
// 反例引理只装证明、没有调用方：普通构建里 `verus!` 把这个模块擦空，再导出它就成了没用到的 import。
mod counterexamples_refused;

pub use compaction_keeps_what_the_user_sees::*;

verus! {

// ===== 槽区间 =====
pub struct SlotRun {
    pub start: int,
    pub span: int,
}

pub open spec fn run_covers(run: SlotRun, slot: int) -> bool {
    run.start <= slot < run.start + run.span
}

pub open spec fn runs_overlap(this_run: SlotRun, other_run: SlotRun) -> bool {
    this_run.start < other_run.start + other_run.span && other_run.start < this_run.start + this_run.span
}

// 盘上一个区间读出来的单元：区间里每个槽上的字节，按槽序排成一串。
pub open spec fn run_bytes_on_disk(disk: Map<int, Seq<u8>>, run: SlotRun) -> Seq<Seq<u8>> {
    Seq::new(run.span as nat, |offset: int| disk[run.start + offset])
}

pub open spec fn run_is_on_disk(disk: Map<int, Seq<u8>>, run: SlotRun) -> bool {
    &&& run.span >= 1
    &&& forall|offset: int| 0 <= offset < run.span ==> disk.contains_key(#[trigger] (run.start + offset))
}

// ===== 状态 =====
// 一个视图：key 到槽区间的映射，加上这一版的逻辑内容（每个 key 读到的单元）。
pub struct RunView {
    pub mapping: Map<u64, SlotRun>,
    pub logical: Map<u64, Seq<Seq<u8>>>,
}

// 根怎么被用户点名：用户发布的根带版本号；整理的发布占根环的一个根，用户点不到它。
pub enum RootName {
    UserVersion { version: nat },
    CompactionPublish,
}

pub struct RingRoot {
    pub view: RunView,
    pub name: RootName,
}

pub struct RunRingState {
    // 已持久的抽象槽，每个槽一段字节。
    pub disk: Map<int, Seq<u8>>,
    // 根环里的持久根，旧到新；最后一个是崩溃后重现的那一个。
    pub roots: Seq<RingRoot>,
    // 内存 F：回退候选是 roots[in_memory_floor..]。
    pub in_memory_floor: nat,
    // 持久 F：最后一个持久的根带着的 F；回收与覆写要避开 roots[persisted_floor..] 的每个根，崩溃让内存 F 回到它。
    pub persisted_floor: nat,
    // 内存里的现行版本，可以领先最后一个持久根（发布还没落成根）。
    pub head: RunView,
    // 进程内的留存读者，各带自己的逻辑版本；崩溃丢掉它们。
    pub readers: Seq<RunView>,
    // 根环容得下的根数（持久 F 之上），恒大于 0。
    pub window: nat,
    // 下一次用户发布成根时拿的版本号。
    pub next_user_version: nat,
}

pub enum RunRingStep {
    Write { slot: int, bytes: Seq<u8> },
    Reclaim { slot: int },
    Relocate { key: u64, destination: SlotRun },
    UserPublish { new_view: RunView },
    UserCheckpoint,
    CompactionCheckpoint,
    Rollback { version: nat },
    RaiseFloorInMemory { version: nat },
    AcquireReader,
    ReleaseReader { index: int },
    Crash,
}

pub open spec fn ring_state_of(
    disk: Map<int, Seq<u8>>,
    roots: Seq<RingRoot>,
    in_memory_floor: nat,
    persisted_floor: nat,
    head: RunView,
    readers: Seq<RunView>,
    window: nat,
    next_user_version: nat,
) -> RunRingState {
    RunRingState { disk, roots, in_memory_floor, persisted_floor, head, readers, window, next_user_version }
}

// ===== 不变量 =====
// 读抽象：映射的 key 集合恰好等于逻辑内容的 key 集合，每个区间都在盘上、读出的单元等于逻辑内容。
pub open spec fn view_reads_its_logical_content(view: RunView, disk: Map<int, Seq<u8>>) -> bool {
    &&& view.mapping.dom() == view.logical.dom()
    &&& forall|key: u64| view.mapping.contains_key(key) ==> run_is_on_disk(disk, #[trigger] view.mapping[key])
        && view.logical[key] == run_bytes_on_disk(disk, view.mapping[key])
}

pub open spec fn run_ring_is_shaped(state: RunRingState) -> bool {
    &&& state.roots.len() >= 1
    &&& state.persisted_floor <= state.in_memory_floor
    &&& state.in_memory_floor < state.roots.len()
    &&& state.window >= 1
    &&& state.roots.len() - state.persisted_floor <= state.window
}

// 现行、持久 F 之上的每个根（回退候选是其中内存 F 之上的那几个）、每个读者，各自读到自己的逻辑内容。
pub open spec fn every_protected_view_reads_right(state: RunRingState) -> bool {
    &&& run_ring_is_shaped(state)
    &&& view_reads_its_logical_content(state.head, state.disk)
    &&& forall|index: int| state.persisted_floor <= index < state.roots.len()
        ==> view_reads_its_logical_content((#[trigger] state.roots[index]).view, state.disk)
    &&& forall|index: int| 0 <= index < state.readers.len()
        ==> view_reads_its_logical_content(#[trigger] state.readers[index], state.disk)
}

// 按区间避让：映射里没有一个区间罩住这个槽。
pub open spec fn mapping_avoids_slot(mapping: Map<u64, SlotRun>, slot: int) -> bool {
    forall|key: u64| mapping.contains_key(key) ==> !run_covers(#[trigger] mapping[key], slot)
}

// 回收与覆写的保护条件：现行、持久 F 之上的每个根、每个读者，都没有区间罩住这个槽。
pub open spec fn slot_is_unreferenced(state: RunRingState, slot: int) -> bool {
    &&& mapping_avoids_slot(state.head.mapping, slot)
    &&& forall|index: int| state.persisted_floor <= index < state.roots.len()
        ==> mapping_avoids_slot((#[trigger] state.roots[index]).view.mapping, slot)
    &&& forall|index: int| 0 <= index < state.readers.len()
        ==> mapping_avoids_slot((#[trigger] state.readers[index]).mapping, slot)
}

// ===== 用户看得见的那一层 =====
// 根被用户看见的只有它的逻辑内容与名字；回退与抬 F 按名字在这一层上找根，G7 也在这一层上说。
pub struct ObservedRoot {
    pub logical: Map<u64, Seq<Seq<u8>>>,
    pub name: RootName,
}

pub open spec fn observed_root_of(root: RingRoot) -> ObservedRoot {
    ObservedRoot { logical: root.view.logical, name: root.name }
}

pub open spec fn observed_roots_of(roots: Seq<RingRoot>) -> Seq<ObservedRoot> {
    roots.map_values(|root: RingRoot| observed_root_of(root))
}

// 用户点得到的版本号：回退候选（内存 F 之上）里有一个根带着它。
pub open spec fn version_is_a_candidate(roots: Seq<ObservedRoot>, floor: nat, version: nat) -> bool {
    exists|index: int| floor <= index < roots.len() && (#[trigger] roots[index]).name == (RootName::UserVersion { version })
}

pub open spec fn index_of_the_candidate_version(roots: Seq<ObservedRoot>, floor: nat, version: nat) -> int {
    choose|index: int| floor <= index < roots.len() && (#[trigger] roots[index]).name == (RootName::UserVersion { version })
}

// 用户看得见的那一层逐个对得上根环里的根。
pub proof fn lemma_observed_roots_keep_each_root(roots: Seq<RingRoot>)
    ensures
        observed_roots_of(roots).len() == roots.len(),
        forall|index: int| 0 <= index < roots.len() ==> #[trigger] observed_roots_of(roots)[index] == observed_root_of(roots[index]),
{
}

pub open spec fn user_version_is_a_candidate(roots: Seq<RingRoot>, floor: nat, version: nat) -> bool {
    version_is_a_candidate(observed_roots_of(roots), floor, version)
}

pub open spec fn index_of_user_version(roots: Seq<RingRoot>, floor: nat, version: nat) -> int {
    index_of_the_candidate_version(observed_roots_of(roots), floor, version)
}

// ===== 步骤 =====

// 用户那一侧的发布落成根之后的 F：根数超过根环容量，就把最旧的候选挤出去（内存 F 加一）。新根带着这个 F 持久，持久 F 跟上。
pub open spec fn floor_after_a_checkpoint(state: RunRingState) -> nat {
    if state.roots.len() + 1 - state.in_memory_floor > state.window {
        (state.in_memory_floor + 1) as nat
    } else {
        state.in_memory_floor
    }
}

// 整理的发布落成根之后的持久 F：整理的发布只带持久 F、不带内存里抬过的，根数超过根环容量就挤掉持久 F 之上
// 最旧的根（持久 F 加一）。只在内存里抬过的 F 不跟着落盘，崩溃之后照旧回到这里。
pub open spec fn persisted_floor_after_a_compaction_publish(state: RunRingState) -> nat {
    if state.roots.len() + 1 - state.persisted_floor > state.window {
        (state.persisted_floor + 1) as nat
    } else {
        state.persisted_floor
    }
}

// 整理的发布之后的内存 F：内存里抬过的照旧留在内存里；被挤上来的持久 F 越过它时跟上。
pub open spec fn in_memory_floor_after_a_compaction_publish(state: RunRingState) -> nat {
    if state.in_memory_floor >= persisted_floor_after_a_compaction_publish(state) {
        state.in_memory_floor
    } else {
        persisted_floor_after_a_compaction_publish(state)
    }
}

pub open spec fn relocation_is_enabled(state: RunRingState, key: u64, destination: SlotRun) -> bool {
    &&& state.head.mapping.contains_key(key)
    &&& run_is_on_disk(state.disk, state.head.mapping[key])
    &&& destination.span == state.head.mapping[key].span
    &&& run_is_on_disk(state.disk, destination)
    &&& run_bytes_on_disk(state.disk, destination) == run_bytes_on_disk(state.disk, state.head.mapping[key])
}

// 协议允许的步骤。整理的写（目的地）与回收都走 Write、Reclaim 这两道门；搬迁只换现行映射，要求目的区间已持久、跨度相同、
// 字节与源相同（完整字节相等强于校验和相等）。
// 整理的发布只在现行的逻辑内容与最后一个根相同时发（映射可以不同：搬迁只换映射）：现行领先最后一个根（用户的改动还没落成根）时它不发，
// 不把那些改动先落盘。这是 G7 那一条前提（compaction_publish_is_accepted）在根环这一层的门，投影引理把它带到用户看得见的那一层。
// core 里整理的发布接在上一版后面（`singlefs-core` 的 `transaction.rs` 里 `publish_compaction_step_after_its_check` 照 `previous` 的树只搬单元），
// 落成的根与上一个根逻辑内容相同；攒批的会话在整理起跑之前先把待发排空（`mounted_session.rs` 里
// `MountedSessionWithPendingChanges::run_compaction_in_the_background`），排空不成就不跑。
pub open spec fn run_ring_step_is_enabled(state: RunRingState, step: RunRingStep) -> bool {
    match step {
        RunRingStep::Write { slot, bytes: _ } => slot_is_unreferenced(state, slot),
        RunRingStep::Reclaim { slot } => slot_is_unreferenced(state, slot),
        RunRingStep::Relocate { key, destination } => relocation_is_enabled(state, key, destination),
        RunRingStep::UserPublish { new_view } => view_reads_its_logical_content(new_view, state.disk),
        RunRingStep::UserCheckpoint => true,
        RunRingStep::CompactionCheckpoint => state.roots.len() >= 1
            && state.head.logical == state.roots[state.roots.len() - 1].view.logical,
        RunRingStep::Rollback { version } => user_version_is_a_candidate(state.roots, state.in_memory_floor, version),
        RunRingStep::RaiseFloorInMemory { version } => user_version_is_a_candidate(
            state.roots,
            state.in_memory_floor,
            version,
        ),
        RunRingStep::AcquireReader => true,
        RunRingStep::ReleaseReader { index } => 0 <= index < state.readers.len(),
        RunRingStep::Crash => true,
    }
}

// 纯状态转换；与 run_ring_step_is_enabled 成对用，单独调不拒绝不合法的步骤。
pub open spec fn run_ring_next(state: RunRingState, step: RunRingStep) -> RunRingState {
    match step {
        RunRingStep::Write { slot, bytes } => ring_state_of(
            state.disk.insert(slot, bytes),
            state.roots,
            state.in_memory_floor,
            state.persisted_floor,
            state.head,
            state.readers,
            state.window,
            state.next_user_version,
        ),
        RunRingStep::Reclaim { slot } => ring_state_of(
            state.disk.remove(slot),
            state.roots,
            state.in_memory_floor,
            state.persisted_floor,
            state.head,
            state.readers,
            state.window,
            state.next_user_version,
        ),
        RunRingStep::Relocate { key, destination } => ring_state_of(
            state.disk,
            state.roots,
            state.in_memory_floor,
            state.persisted_floor,
            RunView { mapping: state.head.mapping.insert(key, destination), logical: state.head.logical },
            state.readers,
            state.window,
            state.next_user_version,
        ),
        RunRingStep::UserPublish { new_view } => ring_state_of(
            state.disk,
            state.roots,
            state.in_memory_floor,
            state.persisted_floor,
            new_view,
            state.readers,
            state.window,
            state.next_user_version,
        ),
        RunRingStep::UserCheckpoint => ring_state_of(
            state.disk,
            state.roots.push(RingRoot { view: state.head, name: RootName::UserVersion { version: state.next_user_version } }),
            floor_after_a_checkpoint(state),
            floor_after_a_checkpoint(state),
            state.head,
            state.readers,
            state.window,
            state.next_user_version + 1,
        ),
        RunRingStep::CompactionCheckpoint => ring_state_of(
            state.disk,
            state.roots.push(RingRoot { view: state.head, name: RootName::CompactionPublish }),
            in_memory_floor_after_a_compaction_publish(state),
            persisted_floor_after_a_compaction_publish(state),
            state.head,
            state.readers,
            state.window,
            state.next_user_version,
        ),
        RunRingStep::Rollback { version } => ring_state_of(
            state.disk,
            state.roots,
            state.in_memory_floor,
            state.persisted_floor,
            state.roots[index_of_user_version(state.roots, state.in_memory_floor, version)].view,
            state.readers,
            state.window,
            state.next_user_version,
        ),
        RunRingStep::RaiseFloorInMemory { version } => ring_state_of(
            state.disk,
            state.roots,
            index_of_user_version(state.roots, state.in_memory_floor, version) as nat,
            state.persisted_floor,
            state.head,
            state.readers,
            state.window,
            state.next_user_version,
        ),
        RunRingStep::AcquireReader => ring_state_of(
            state.disk,
            state.roots,
            state.in_memory_floor,
            state.persisted_floor,
            state.head,
            state.readers.push(state.head),
            state.window,
            state.next_user_version,
        ),
        RunRingStep::ReleaseReader { index } => ring_state_of(
            state.disk,
            state.roots,
            state.in_memory_floor,
            state.persisted_floor,
            state.head,
            state.readers.remove(index),
            state.window,
            state.next_user_version,
        ),
        RunRingStep::Crash => ring_state_of(
            state.disk,
            state.roots,
            state.persisted_floor,
            state.persisted_floor,
            state.roots[state.roots.len() - 1].view,
            Seq::empty(),
            state.window,
            state.next_user_version,
        ),
    }
}

// ===== 一步保持 =====
// 避开这个槽的视图，在这个槽被写或被回收之后仍读对。
pub proof fn lemma_writing_an_avoided_slot_keeps_the_view(
    view: RunView,
    disk: Map<int, Seq<u8>>,
    slot: int,
    bytes: Seq<u8>,
)
    requires
        view_reads_its_logical_content(view, disk),
        mapping_avoids_slot(view.mapping, slot),
    ensures
        view_reads_its_logical_content(view, disk.insert(slot, bytes)),
        view_reads_its_logical_content(view, disk.remove(slot)),
{
    let written = disk.insert(slot, bytes);
    let removed = disk.remove(slot);
    assert forall|key: u64| view.mapping.contains_key(key) implies run_is_on_disk(written, #[trigger] view.mapping[key])
        && view.logical[key] == run_bytes_on_disk(written, view.mapping[key]) by {
        let run = view.mapping[key];
        assert(!run_covers(run, slot));
        assert(run_is_on_disk(disk, run));
        assert forall|offset: int| 0 <= offset < run.span implies written.contains_key(#[trigger] (run.start + offset)) by {
            assert(run.start + offset != slot);
        }
        assert(run_bytes_on_disk(written, run) =~= run_bytes_on_disk(disk, run)) by {
            assert forall|offset: int| 0 <= offset < run.span implies #[trigger] written[run.start + offset]
                == disk[run.start + offset] by {
                assert(run.start + offset != slot);
            }
        }
    }
    assert forall|key: u64| view.mapping.contains_key(key) implies run_is_on_disk(removed, #[trigger] view.mapping[key])
        && view.logical[key] == run_bytes_on_disk(removed, view.mapping[key]) by {
        let run = view.mapping[key];
        assert(!run_covers(run, slot));
        assert(run_is_on_disk(disk, run));
        assert forall|offset: int| 0 <= offset < run.span implies removed.contains_key(#[trigger] (run.start + offset)) by {
            assert(run.start + offset != slot);
        }
        assert(run_bytes_on_disk(removed, run) =~= run_bytes_on_disk(disk, run)) by {
            assert forall|offset: int| 0 <= offset < run.span implies #[trigger] removed[run.start + offset]
                == disk[run.start + offset] by {
                assert(run.start + offset != slot);
            }
        }
    }
}

// 按十一种步骤分类：写与回收用上面那条帧引理；搬迁分被搬的 key 与其余 key；两种发布分新根与旧根；回退取一个候选；
// 只在内存里抬 F 不动保护范围；读者取得与释放分前后下标；崩溃取最后一个根、内存 F 回到持久 F。
pub proof fn lemma_every_run_ring_step_keeps_every_protected_view(state: RunRingState, step: RunRingStep)
    requires
        every_protected_view_reads_right(state),
        run_ring_step_is_enabled(state, step),
    ensures
        every_protected_view_reads_right(run_ring_next(state, step)),
{
    let after = run_ring_next(state, step);
    match step {
        RunRingStep::Write { slot, bytes } => {
            lemma_writing_an_avoided_slot_keeps_the_view(state.head, state.disk, slot, bytes);
            assert forall|index: int| state.persisted_floor <= index < state.roots.len() implies view_reads_its_logical_content(
                (#[trigger] state.roots[index]).view,
                state.disk.insert(slot, bytes),
            ) by {
                lemma_writing_an_avoided_slot_keeps_the_view(state.roots[index].view, state.disk, slot, bytes);
            }
            assert forall|index: int| 0 <= index < state.readers.len() implies view_reads_its_logical_content(
                #[trigger] state.readers[index],
                state.disk.insert(slot, bytes),
            ) by {
                lemma_writing_an_avoided_slot_keeps_the_view(state.readers[index], state.disk, slot, bytes);
            }
        },
        RunRingStep::Reclaim { slot } => {
            lemma_writing_an_avoided_slot_keeps_the_view(state.head, state.disk, slot, Seq::empty());
            assert forall|index: int| state.persisted_floor <= index < state.roots.len() implies view_reads_its_logical_content(
                (#[trigger] state.roots[index]).view,
                state.disk.remove(slot),
            ) by {
                lemma_writing_an_avoided_slot_keeps_the_view(state.roots[index].view, state.disk, slot, Seq::empty());
            }
            assert forall|index: int| 0 <= index < state.readers.len() implies view_reads_its_logical_content(
                #[trigger] state.readers[index],
                state.disk.remove(slot),
            ) by {
                lemma_writing_an_avoided_slot_keeps_the_view(state.readers[index], state.disk, slot, Seq::empty());
            }
        },
        RunRingStep::Relocate { key, destination } => {
            let moved = state.head.mapping.insert(key, destination);
            assert(moved.dom() =~= state.head.mapping.dom());
            assert forall|moved_key: u64| moved.contains_key(moved_key) implies run_is_on_disk(state.disk, #[trigger] moved[moved_key])
                && state.head.logical[moved_key] == run_bytes_on_disk(state.disk, moved[moved_key]) by {
                if moved_key != key {
                    assert(moved[moved_key] == state.head.mapping[moved_key]);
                } else {
                    assert(moved[moved_key] == destination);
                }
            }
        },
        RunRingStep::UserPublish { new_view } => {},
        RunRingStep::UserCheckpoint => {
            assert forall|index: int| after.persisted_floor <= index < after.roots.len() implies view_reads_its_logical_content(
                (#[trigger] after.roots[index]).view,
                after.disk,
            ) by {
                if index < state.roots.len() {
                    assert(after.roots[index] == state.roots[index]);
                } else {
                    assert(after.roots[index].view == state.head);
                }
            }
        },
        RunRingStep::CompactionCheckpoint => {
            assert forall|index: int| after.persisted_floor <= index < after.roots.len() implies view_reads_its_logical_content(
                (#[trigger] after.roots[index]).view,
                after.disk,
            ) by {
                if index < state.roots.len() {
                    assert(after.roots[index] == state.roots[index]);
                } else {
                    assert(after.roots[index].view == state.head);
                }
            }
        },
        RunRingStep::Rollback { version } => {
            let target = index_of_user_version(state.roots, state.in_memory_floor, version);
            assert(state.in_memory_floor <= target < state.roots.len());
            assert(view_reads_its_logical_content(state.roots[target].view, state.disk));
        },
        RunRingStep::RaiseFloorInMemory { version } => {
            let target = index_of_user_version(state.roots, state.in_memory_floor, version);
            assert(state.in_memory_floor <= target < state.roots.len());
        },
        RunRingStep::AcquireReader => {
            assert forall|index: int| 0 <= index < after.readers.len() implies view_reads_its_logical_content(
                #[trigger] after.readers[index],
                after.disk,
            ) by {
                if index < state.readers.len() {
                    assert(after.readers[index] == state.readers[index]);
                } else {
                    assert(after.readers[index] == state.head);
                }
            }
        },
        RunRingStep::ReleaseReader { index } => {
            assert forall|reader_index: int| 0 <= reader_index < after.readers.len() implies view_reads_its_logical_content(
                #[trigger] after.readers[reader_index],
                after.disk,
            ) by {
                if reader_index < index {
                    assert(after.readers[reader_index] == state.readers[reader_index]);
                } else {
                    assert(after.readers[reader_index] == state.readers[reader_index + 1]);
                }
            }
        },
        RunRingStep::Crash => {
            assert(view_reads_its_logical_content(state.roots[state.roots.len() - 1].view, state.disk));
        },
    }
}

// 任意有限历史：count 是最后一个要证的状态下标（含端点），states 比 steps 多一个；取 count = steps.len() 得整段历史的结论。
// 没有固定的长度上界，崩溃、回退、抬 F、整理的发布与读者都在归纳域里。
pub proof fn theorem_every_finite_run_ring_history_keeps_every_protected_view(
    states: Seq<RunRingState>,
    steps: Seq<RunRingStep>,
    count: nat,
)
    requires
        states.len() == steps.len() + 1,
        count < states.len(),
        every_protected_view_reads_right(states[0]),
        forall|index: int| 0 <= index < steps.len() ==> run_ring_step_is_enabled(#[trigger] states[index], steps[index])
            && states[index + 1] == run_ring_next(states[index], steps[index]),
    ensures
        forall|index: int| 0 <= index <= count ==> every_protected_view_reads_right(#[trigger] states[index]),
    decreases count,
{
    if count > 0 {
        theorem_every_finite_run_ring_history_keeps_every_protected_view(states, steps, (count - 1) as nat);
        lemma_every_run_ring_step_keeps_every_protected_view(states[count - 1], steps[count - 1]);
        assert(states[count as int] == run_ring_next(states[count - 1], steps[count - 1]));
    }
}

// 崩溃恢复的是最后一个持久的根：现行换成它、读者清空、盘不动；再崩溃一次结果相同。内存里领先于它的现行（还没落成根的发布、搬迁换过的映射）不当恢复根。
pub proof fn theorem_a_crash_recovers_the_newest_root_and_is_idempotent(state: RunRingState)
    ensures
        run_ring_next(state, RunRingStep::Crash).head == state.roots[state.roots.len() - 1].view,
        run_ring_next(state, RunRingStep::Crash).disk == state.disk,
        run_ring_next(state, RunRingStep::Crash).readers.len() == 0,
        run_ring_next(run_ring_next(state, RunRingStep::Crash), RunRingStep::Crash) == run_ring_next(state, RunRingStep::Crash),
{
}

// ===== 两层 F =====
// 只在内存里抬 F 不放开任何一个槽：保护范围看的是持久 F，抬 F 之后到下一次用户那一侧的发布落成根之前，旧候选的槽照样扣着。
// 崩溃让内存 F 回到持久 F；用户那一侧的发布落成根时持久 F 跟上内存 F；整理的发布落成根时持久 F 至多挤上一格、不跟内存 F。
pub proof fn lemma_the_in_memory_floor_and_the_persisted_floor(state: RunRingState, version: nat, slot: int)
    requires
        run_ring_is_shaped(state),
        user_version_is_a_candidate(state.roots, state.in_memory_floor, version),
    ensures
        slot_is_unreferenced(run_ring_next(state, RunRingStep::RaiseFloorInMemory { version }), slot)
            == slot_is_unreferenced(state, slot),
        run_ring_next(state, RunRingStep::RaiseFloorInMemory { version }).persisted_floor == state.persisted_floor,
        run_ring_next(state, RunRingStep::RaiseFloorInMemory { version }).in_memory_floor >= state.in_memory_floor,
        run_ring_next(state, RunRingStep::Crash).in_memory_floor == state.persisted_floor,
        run_ring_next(state, RunRingStep::Crash).roots == state.roots,
        run_ring_next(state, RunRingStep::UserCheckpoint).persisted_floor
            == run_ring_next(state, RunRingStep::UserCheckpoint).in_memory_floor,
        run_ring_next(state, RunRingStep::CompactionCheckpoint).persisted_floor <= state.persisted_floor + 1,
        run_ring_next(state, RunRingStep::CompactionCheckpoint).in_memory_floor >= state.in_memory_floor,
        run_ring_next(state, RunRingStep::CompactionCheckpoint).in_memory_floor
            >= run_ring_next(state, RunRingStep::CompactionCheckpoint).persisted_floor,
{
    let raised = run_ring_next(state, RunRingStep::RaiseFloorInMemory { version });
    assert(raised.roots == state.roots && raised.head == state.head && raised.readers == state.readers);
    let target = index_of_user_version(state.roots, state.in_memory_floor, version);
    assert(state.in_memory_floor <= target < state.roots.len());
}

} // verus!
