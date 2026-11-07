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


// ===== G7：整理对用户看得见的东西的效应 =====
// D13 已定项 18 性质那一句：整理不改现行内容；
// 回退候选集是不整理那份的后缀、至多少 k 个，抬 F 的上限恰多 k（k 为这一段整理占的发布数），按版本号点名。
// 这一层只留用户看得见的：根的逻辑内容与名字、内存 F、持久 F、现行内容、根环容量与下一个版本号。整理那几步里不发布的（写目的地、换映射、回收）
// 与读者不改这一层，整理的发布在这一层上是「推一个不带版本号的根」；见 lemma_run_ring_steps_project_onto_the_observed_operations。
// 历史里可以有崩溃（两份同时崩溃，G7 连崩溃一起成立）：崩溃让内存 F 回到持久 F、现行回到最后一个根，
// 所以持久 F 之上的根也要一层一层对得上，两份最后一个根的内容也要相同。整理的发布只在现行内容与最后一个根相同时发
// （compaction_publish_is_accepted；穷举 g7_named_roots.py 同样只在这时插整理的发布）：不然它把用户还没落成根的改动先落盘，
// 崩溃之后整理那份的现行比不整理那份新。根环那一层的整理的发布带着同一道门（run_ring_step_is_enabled），
// lemma_run_ring_steps_project_onto_the_observed_operations 说两道门相同；门拦下的那一段历史在
// lemma_a_compaction_publish_is_refused_while_the_head_is_ahead_of_the_last_root。
// 只说两份都接受了每一步的历史：同一步两份都拒的是两边的空操作，删掉它不改结论；不整理那份接受、整理那份拒的
// 就是候选移出窗口本身，之后两份不再是同一串用户历史。整理那份接受、不整理那份拒的从不出现
// （lemma_compaction_never_admits_an_operation_the_plain_history_refuses）。
pub struct RingObservation {
    pub roots: Seq<ObservedRoot>,
    pub in_memory_floor: nat,
    pub persisted_floor: nat,
    pub head_logical: Map<u64, Seq<Seq<u8>>>,
    pub window: nat,
    pub next_user_version: nat,
}

pub enum UserOperation {
    Publish { logical: Map<u64, Seq<Seq<u8>>> },
    Checkpoint,
    Rollback { version: nat },
    RaiseFloor { version: nat },
    Crash,
}

pub open spec fn observation_of(
    roots: Seq<ObservedRoot>,
    in_memory_floor: nat,
    persisted_floor: nat,
    head_logical: Map<u64, Seq<Seq<u8>>>,
    window: nat,
    next_user_version: nat,
) -> RingObservation {
    RingObservation { roots, in_memory_floor, persisted_floor, head_logical, window, next_user_version }
}

pub open spec fn observe_the_run_ring(state: RunRingState) -> RingObservation {
    observation_of(
        observed_roots_of(state.roots),
        state.in_memory_floor,
        state.persisted_floor,
        state.head.logical,
        state.window,
        state.next_user_version,
    )
}

pub open spec fn observed_floor_after_a_checkpoint(observation: RingObservation) -> nat {
    if observation.roots.len() + 1 - observation.in_memory_floor > observation.window {
        (observation.in_memory_floor + 1) as nat
    } else {
        observation.in_memory_floor
    }
}

// 整理的发布落成根之后的持久 F 与内存 F：persisted_floor_after_a_compaction_publish、in_memory_floor_after_a_compaction_publish 在这一层上的写法。
pub open spec fn observed_persisted_floor_after_a_compaction_publish(observation: RingObservation) -> nat {
    if observation.roots.len() + 1 - observation.persisted_floor > observation.window {
        (observation.persisted_floor + 1) as nat
    } else {
        observation.persisted_floor
    }
}

pub open spec fn observed_in_memory_floor_after_a_compaction_publish(observation: RingObservation) -> nat {
    if observation.in_memory_floor >= observed_persisted_floor_after_a_compaction_publish(observation) {
        observation.in_memory_floor
    } else {
        observed_persisted_floor_after_a_compaction_publish(observation)
    }
}

pub open spec fn user_operation_is_accepted(observation: RingObservation, operation: UserOperation) -> bool {
    match operation {
        UserOperation::Publish { logical: _ } => true,
        UserOperation::Checkpoint => true,
        UserOperation::Rollback { version } => version_is_a_candidate(
            observation.roots,
            observation.in_memory_floor,
            version,
        ),
        UserOperation::RaiseFloor { version } => version_is_a_candidate(
            observation.roots,
            observation.in_memory_floor,
            version,
        ),
        UserOperation::Crash => true,
    }
}

pub open spec fn observation_after_user_operation(observation: RingObservation, operation: UserOperation) -> RingObservation {
    match operation {
        UserOperation::Publish { logical } => observation_of(
            observation.roots,
            observation.in_memory_floor,
            observation.persisted_floor,
            logical,
            observation.window,
            observation.next_user_version,
        ),
        UserOperation::Checkpoint => observation_of(
            observation.roots.push(
                ObservedRoot {
                    logical: observation.head_logical,
                    name: RootName::UserVersion { version: observation.next_user_version },
                },
            ),
            observed_floor_after_a_checkpoint(observation),
            observed_floor_after_a_checkpoint(observation),
            observation.head_logical,
            observation.window,
            observation.next_user_version + 1,
        ),
        UserOperation::Rollback { version } => observation_of(
            observation.roots,
            observation.in_memory_floor,
            observation.persisted_floor,
            observation.roots[index_of_the_candidate_version(
                observation.roots,
                observation.in_memory_floor,
                version,
            )].logical,
            observation.window,
            observation.next_user_version,
        ),
        UserOperation::RaiseFloor { version } => observation_of(
            observation.roots,
            index_of_the_candidate_version(observation.roots, observation.in_memory_floor, version) as nat,
            observation.persisted_floor,
            observation.head_logical,
            observation.window,
            observation.next_user_version,
        ),
        // 崩溃：内存 F 回到持久 F，现行回到最后一个根。
        UserOperation::Crash => observation_of(
            observation.roots,
            observation.persisted_floor,
            observation.persisted_floor,
            observation.roots[observation.roots.len() - 1].logical,
            observation.window,
            observation.next_user_version,
        ),
    }
}

pub open spec fn observation_after_a_compaction_publish(observation: RingObservation) -> RingObservation {
    observation_of(
        observation.roots.push(ObservedRoot { logical: observation.head_logical, name: RootName::CompactionPublish }),
        observed_in_memory_floor_after_a_compaction_publish(observation),
        observed_persisted_floor_after_a_compaction_publish(observation),
        observation.head_logical,
        observation.window,
        observation.next_user_version,
    )
}

// 整理的发布只在现行内容与最后一个根相同时发：整理不把用户还没落成根的改动先落盘。
pub open spec fn compaction_publish_is_accepted(observation: RingObservation) -> bool {
    &&& observation.roots.len() >= 1
    &&& observation.head_logical == observation.roots[observation.roots.len() - 1].logical
}

// 根环的每一步在用户看得见的那一层上是什么：不发布的步骤（写、回收、搬迁、读者取得与释放）不改这一层；用户发布、两种发布、
// 按版本号回退、只在内存里抬 F 与崩溃各是这一层的一步，根环允许的这一层也接受。整理的发布、回退与抬 F 三样，根环允许的恰是这一层接受的：
// G7 要「整理那份也接受每一步」，其中整理的发布那一条（compaction_publish_is_accepted）由此从根环的门推出，不是另加的假设。
pub proof fn lemma_run_ring_steps_project_onto_the_observed_operations(state: RunRingState, step: RunRingStep)
    ensures
        match step {
            RunRingStep::Write { slot: _, bytes: _ } => observe_the_run_ring(run_ring_next(state, step)) == observe_the_run_ring(state),
            RunRingStep::Reclaim { slot: _ } => observe_the_run_ring(run_ring_next(state, step)) == observe_the_run_ring(state),
            RunRingStep::Relocate { key: _, destination: _ } => observe_the_run_ring(run_ring_next(state, step))
                == observe_the_run_ring(state),
            RunRingStep::UserPublish { new_view } => observe_the_run_ring(run_ring_next(state, step))
                == observation_after_user_operation(
                observe_the_run_ring(state),
                UserOperation::Publish { logical: new_view.logical },
            ),
            RunRingStep::UserCheckpoint => observe_the_run_ring(run_ring_next(state, step)) == observation_after_user_operation(
                observe_the_run_ring(state),
                UserOperation::Checkpoint,
            ),
            RunRingStep::CompactionCheckpoint => {
                &&& run_ring_step_is_enabled(state, step) == compaction_publish_is_accepted(observe_the_run_ring(state))
                &&& observe_the_run_ring(run_ring_next(state, step)) == observation_after_a_compaction_publish(
                    observe_the_run_ring(state),
                )
            },
            RunRingStep::Rollback { version } => {
                &&& run_ring_step_is_enabled(state, step) == user_operation_is_accepted(
                    observe_the_run_ring(state),
                    UserOperation::Rollback { version },
                )
                &&& (run_ring_step_is_enabled(state, step) ==> observe_the_run_ring(run_ring_next(state, step))
                    == observation_after_user_operation(observe_the_run_ring(state), UserOperation::Rollback { version }))
            },
            RunRingStep::RaiseFloorInMemory { version } => {
                &&& run_ring_step_is_enabled(state, step) == user_operation_is_accepted(
                    observe_the_run_ring(state),
                    UserOperation::RaiseFloor { version },
                )
                &&& observe_the_run_ring(run_ring_next(state, step)) == observation_after_user_operation(
                    observe_the_run_ring(state),
                    UserOperation::RaiseFloor { version },
                )
            },
            RunRingStep::AcquireReader => observe_the_run_ring(run_ring_next(state, step)) == observe_the_run_ring(state),
            RunRingStep::ReleaseReader { index: _ } => observe_the_run_ring(run_ring_next(state, step))
                == observe_the_run_ring(state),
            RunRingStep::Crash => state.roots.len() >= 1 ==> observe_the_run_ring(run_ring_next(state, step))
                == observation_after_user_operation(observe_the_run_ring(state), UserOperation::Crash),
        },
{
    let after = run_ring_next(state, step);
    lemma_observed_roots_keep_each_root(state.roots);
    match step {
        RunRingStep::UserCheckpoint => {
            assert(observed_roots_of(after.roots) =~= observed_roots_of(state.roots).push(
                ObservedRoot { logical: state.head.logical, name: RootName::UserVersion { version: state.next_user_version } },
            ));
        },
        RunRingStep::CompactionCheckpoint => {
            assert(observed_roots_of(after.roots) =~= observed_roots_of(state.roots).push(
                ObservedRoot { logical: state.head.logical, name: RootName::CompactionPublish },
            ));
            if state.roots.len() >= 1 {
                assert(observed_roots_of(state.roots)[state.roots.len() - 1] == observed_root_of(
                    state.roots[state.roots.len() - 1],
                ));
            }
        },
        RunRingStep::Rollback { version } => {
            if run_ring_step_is_enabled(state, step) {
                let target = index_of_user_version(state.roots, state.in_memory_floor, version);
                assert(state.in_memory_floor <= target < state.roots.len());
                assert(observed_roots_of(state.roots)[target] == observed_root_of(state.roots[target]));
            }
        },
        RunRingStep::Crash => {
            if state.roots.len() >= 1 {
                assert(observed_roots_of(state.roots)[state.roots.len() - 1] == observed_root_of(
                    state.roots[state.roots.len() - 1],
                ));
            }
        },
        RunRingStep::Write { slot: _, bytes: _ } => {},
        RunRingStep::Reclaim { slot: _ } => {},
        RunRingStep::Relocate { key: _, destination: _ } => {},
        RunRingStep::UserPublish { new_view: _ } => {},
        RunRingStep::RaiseFloorInMemory { version: _ } => {},
        RunRingStep::AcquireReader => {},
        RunRingStep::ReleaseReader { index: _ } => {},
    }
}

pub open spec fn observation_is_shaped(observation: RingObservation) -> bool {
    &&& observation.roots.len() >= 1
    &&& observation.persisted_floor <= observation.in_memory_floor
    &&& observation.in_memory_floor < observation.roots.len()
    &&& observation.window >= 1
    &&& observation.roots.len() - observation.in_memory_floor <= observation.window
    &&& observation.roots.len() - observation.persisted_floor <= observation.window
}

pub open spec fn version_of_root(root: ObservedRoot) -> nat {
    match root.name {
        RootName::UserVersion { version } => version,
        RootName::CompactionPublish => 0,
    }
}

// 不整理那份的每个根都是用户发布的，版本号按根环次序严格升、都小于下一个版本号：按版本号点名恰点到一个根。
pub open spec fn every_root_is_a_user_version_in_order(observation: RingObservation) -> bool {
    &&& forall|index: int| 0 <= index < observation.roots.len() ==> (#[trigger] observation.roots[index]).name is UserVersion
    &&& forall|index: int| 0 <= index < observation.roots.len() ==> version_of_root(#[trigger] observation.roots[index])
        < observation.next_user_version
    &&& forall|earlier: int, later: int|
        #![trigger observation.roots[earlier], observation.roots[later]]
        0 <= earlier < later < observation.roots.len() ==> version_of_root(observation.roots[earlier]) < version_of_root(
            observation.roots[later],
        )
}

// 抬 F 能抬到的最高下标（最后一个根）。
pub open spec fn raise_floor_ceiling(observation: RingObservation) -> int {
    observation.roots.len() - 1
}

// 一层 F 上的对应（内存 F 那一层是回退候选，持久 F 那一层是崩溃之后的回退候选）：oldest 是不整理那份里第一个在整理那份这一层 F 之上的根。
pub open spec fn floors_correspond(
    without_compaction_floor: nat,
    with_compaction_floor: nat,
    without_compaction_root_count: nat,
    with_compaction_root_count: nat,
    position: Seq<int>,
    oldest: int,
    compaction_publishes: nat,
) -> bool {
    // 整理那份这一层 F 之上带版本号的根，恰是不整理那份这一层 F 之上从 oldest 起的那一截（后缀）
    &&& without_compaction_floor <= oldest <= without_compaction_root_count
    &&& forall|index: int|
        0 <= index < position.len() ==> (with_compaction_floor <= #[trigger] position[index] <==> oldest <= index)
    // 至多少 k 个
    &&& oldest - without_compaction_floor <= compaction_publishes
    // 归纳用：不整理那份这一层 F 之上的根数不多于整理那份（整理那份多出来的都是整理的发布）
    &&& without_compaction_root_count - without_compaction_floor <= with_compaction_root_count - with_compaction_floor
}

// 不整理那份与整理那份之间的对应：position 把不整理那份的每个根送到整理那份里同内容、同版本号的根，按原次序；
// oldest_still_shown 是不整理那份里第一个在整理那份回退候选（内存 F 之上）里的根，oldest_still_protected 是第一个在整理那份持久 F 之上的根。
// 各条的意思写在行尾那一句里。
pub open spec fn compaction_relation(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
) -> bool {
    &&& observation_is_shaped(without_compaction)
    &&& observation_is_shaped(with_compaction)
    &&& every_root_is_a_user_version_in_order(without_compaction)
    // 整理不改现行内容
    &&& with_compaction.head_logical == without_compaction.head_logical
    // 崩溃之后重现的内容相同：两份最后一个根的内容相同
    &&& with_compaction.roots[with_compaction.roots.len() - 1].logical == without_compaction.roots[without_compaction.roots.len()
        - 1].logical
    &&& with_compaction.window == without_compaction.window
    &&& with_compaction.next_user_version == without_compaction.next_user_version
    // 抬 F 的上限恰多 k
    &&& raise_floor_ceiling(with_compaction) == raise_floor_ceiling(without_compaction) + compaction_publishes
    // 不整理那份的根按原次序、同内容同版本号出现在整理那份里；整理那份里带版本号的根都是这样来的
    &&& position.len() == without_compaction.roots.len()
    &&& forall|index: int|
        0 <= index < position.len() ==> 0 <= #[trigger] position[index] < with_compaction.roots.len()
            && with_compaction.roots[position[index]] == without_compaction.roots[index]
    &&& forall|earlier: int, later: int|
        #![trigger position[earlier], position[later]]
        0 <= earlier < later < position.len() ==> position[earlier] < position[later]
    &&& forall|with_index: int|
        0 <= with_index < with_compaction.roots.len() && (#[trigger] with_compaction.roots[with_index]).name is UserVersion
            ==> exists|index: int| 0 <= index < position.len() && #[trigger] position[index] == with_index
    // 回退候选（内存 F 之上）是不整理那份的后缀、至多少 k 个
    &&& floors_correspond(
        without_compaction.in_memory_floor,
        with_compaction.in_memory_floor,
        without_compaction.roots.len(),
        with_compaction.roots.len(),
        position,
        oldest_still_shown,
        compaction_publishes,
    )
    // 持久 F 之上（崩溃之后的回退候选）同样是不整理那份的后缀、至多少 k 个
    &&& floors_correspond(
        without_compaction.persisted_floor,
        with_compaction.persisted_floor,
        without_compaction.roots.len(),
        with_compaction.roots.len(),
        position,
        oldest_still_protected,
        compaction_publishes,
    )
}

pub open spec fn compaction_keeps_what_the_user_sees(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
) -> bool {
    exists|position: Seq<int>, oldest_still_shown: int, oldest_still_protected: int|
        #[trigger] compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        )
}

// 严格升的下标序列，两项之间的差不小于它们在序列里隔开的距离。
pub proof fn lemma_strictly_increasing_positions_spread_out(position: Seq<int>, earlier: int, later: int)
    requires
        0 <= earlier <= later < position.len(),
        forall|this_index: int, that_index: int|
            #![trigger position[this_index], position[that_index]]
            0 <= this_index < that_index < position.len() ==> position[this_index] < position[that_index],
    ensures
        position[later] - position[earlier] >= later - earlier,
    decreases later - earlier,
{
    if earlier < later {
        lemma_strictly_increasing_positions_spread_out(position, earlier, later - 1);
        assert(position[later - 1] < position[later]);
    }
}

// 从一层 F 上的对应推出的三件计数：oldest 不超过整理那份这一层的 F；整理那份这一层之上至少装得下不整理那份从 oldest 起的根；
// 两个数相等时整理那份这一层之上最旧的根就是 oldest 那个根。
pub proof fn lemma_corresponding_floors_count(
    without_compaction_floor: nat,
    with_compaction_floor: nat,
    without_compaction_root_count: nat,
    with_compaction_root_count: nat,
    position: Seq<int>,
    oldest: int,
    compaction_publishes: nat,
)
    requires
        floors_correspond(
            without_compaction_floor,
            with_compaction_floor,
            without_compaction_root_count,
            with_compaction_root_count,
            position,
            oldest,
            compaction_publishes,
        ),
        position.len() == without_compaction_root_count,
        forall|index: int| 0 <= index < position.len() ==> 0 <= #[trigger] position[index] < with_compaction_root_count,
        forall|earlier: int, later: int|
            #![trigger position[earlier], position[later]]
            0 <= earlier < later < position.len() ==> position[earlier] < position[later],
    ensures
        oldest <= with_compaction_floor,
        with_compaction_root_count - with_compaction_floor >= without_compaction_root_count - oldest,
        oldest < without_compaction_root_count && with_compaction_root_count - with_compaction_floor
            == without_compaction_root_count - oldest ==> position[oldest] == with_compaction_floor,
        oldest < without_compaction_root_count ==> position[oldest] >= with_compaction_floor,
{
    let length = without_compaction_root_count as int;
    if oldest > 0 {
        lemma_strictly_increasing_positions_spread_out(position, 0, oldest - 1);
        assert(position[0] >= 0);
        assert(!(oldest <= oldest - 1));
        assert(position[oldest - 1] < with_compaction_floor);
    }
    if oldest < length {
        lemma_strictly_increasing_positions_spread_out(position, oldest, length - 1);
        assert(position[length - 1] < with_compaction_root_count);
        assert(position[oldest] >= with_compaction_floor);
    }
}

// 整理那份这一层的 F 挤上一格时，oldest 跟着走：最旧的那个被挤到 F 之下、而它正是 oldest 那个根时加一，否则不变。
pub open spec fn oldest_after_the_floor_moves(position: Seq<int>, oldest: int, floor_before: nat, floor_after: nat) -> int {
    if floor_after != floor_before && oldest < position.len() && position[oldest] == floor_before {
        oldest + 1
    } else {
        oldest
    }
}

// 整理那份多一个根（整理的发布，k 加一）、这一层 F 至多挤上一格：这一层的对应照旧，oldest 照 oldest_after_the_floor_moves 走。
pub proof fn lemma_a_floor_moving_up_by_at_most_one_keeps_the_correspondence(
    without_compaction_floor: nat,
    with_compaction_floor: nat,
    with_compaction_floor_after: nat,
    without_compaction_root_count: nat,
    with_compaction_root_count: nat,
    position: Seq<int>,
    oldest: int,
    compaction_publishes: nat,
)
    requires
        floors_correspond(
            without_compaction_floor,
            with_compaction_floor,
            without_compaction_root_count,
            with_compaction_root_count,
            position,
            oldest,
            compaction_publishes,
        ),
        with_compaction_floor <= with_compaction_floor_after <= with_compaction_floor + 1,
        position.len() == without_compaction_root_count,
        forall|index: int| 0 <= index < position.len() ==> 0 <= #[trigger] position[index] < with_compaction_root_count,
        forall|earlier: int, later: int|
            #![trigger position[earlier], position[later]]
            0 <= earlier < later < position.len() ==> position[earlier] < position[later],
    ensures
        floors_correspond(
            without_compaction_floor,
            with_compaction_floor_after,
            without_compaction_root_count,
            with_compaction_root_count + 1,
            position,
            oldest_after_the_floor_moves(position, oldest, with_compaction_floor, with_compaction_floor_after),
            compaction_publishes + 1,
        ),
{
    lemma_corresponding_floors_count(
        without_compaction_floor,
        with_compaction_floor,
        without_compaction_root_count,
        with_compaction_root_count,
        position,
        oldest,
        compaction_publishes,
    );
    let oldest_after = oldest_after_the_floor_moves(position, oldest, with_compaction_floor, with_compaction_floor_after);
    assert forall|index: int| 0 <= index < position.len() implies (with_compaction_floor_after <= #[trigger] position[index]
        <==> oldest_after <= index) by {
        if with_compaction_floor_after != with_compaction_floor && oldest < position.len() && position[oldest]
            == with_compaction_floor {
            if index > oldest {
                assert(position[oldest] < position[index]);
            } else if index < oldest {
                assert(position[index] < position[oldest]);
            }
        } else if with_compaction_floor_after != with_compaction_floor {
            if index > oldest {
                assert(position[oldest] < position[index]);
            }
        }
    }
}

// 不整理那份按版本号点到的根，在整理那份里就是它的对应根；整理那份接受了的点名，不整理那份也接受。
pub proof fn lemma_a_version_named_in_the_compacted_history_is_the_corresponding_root(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
    version: nat,
)
    requires
        compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        ),
        version_is_a_candidate(with_compaction.roots, with_compaction.in_memory_floor, version),
    ensures
        version_is_a_candidate(without_compaction.roots, without_compaction.in_memory_floor, version),
        index_of_the_candidate_version(with_compaction.roots, with_compaction.in_memory_floor, version) == position[index_of_the_candidate_version(
            without_compaction.roots,
            without_compaction.in_memory_floor,
            version,
        )],
        oldest_still_shown <= index_of_the_candidate_version(without_compaction.roots, without_compaction.in_memory_floor, version),
{
    let with_index = index_of_the_candidate_version(with_compaction.roots, with_compaction.in_memory_floor, version);
    assert(with_compaction.roots[with_index].name is UserVersion);
    let index = choose|index: int| 0 <= index < position.len() && #[trigger] position[index] == with_index;
    assert(without_compaction.roots[index] == with_compaction.roots[with_index]);
    assert(oldest_still_shown <= index);
    assert(without_compaction.roots[index].name == (RootName::UserVersion { version }));
    assert(version_is_a_candidate(without_compaction.roots, without_compaction.in_memory_floor, version));
    let without_index = index_of_the_candidate_version(without_compaction.roots, without_compaction.in_memory_floor, version);
    assert(without_compaction.roots[without_index].name == (RootName::UserVersion { version }));
    if without_index < index {
        assert(version_of_root(without_compaction.roots[without_index]) < version_of_root(without_compaction.roots[index]));
    } else if index < without_index {
        assert(version_of_root(without_compaction.roots[index]) < version_of_root(without_compaction.roots[without_index]));
    }
    assert(without_index == index);
}

// 整理那份接受了的点名，不整理那份也接受（有界穷举里的 b_accepts_a_rejects = 0）。
pub proof fn lemma_compaction_never_admits_an_operation_the_plain_history_refuses(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    operation: UserOperation,
)
    requires
        compaction_keeps_what_the_user_sees(without_compaction, with_compaction, compaction_publishes),
        user_operation_is_accepted(with_compaction, operation),
    ensures
        user_operation_is_accepted(without_compaction, operation),
{
    let (position, oldest_still_shown, oldest_still_protected) = choose|
        position: Seq<int>,
        oldest_still_shown: int,
        oldest_still_protected: int,
    |
        #[trigger] compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        );
    match operation {
        UserOperation::Rollback { version } => {
            lemma_a_version_named_in_the_compacted_history_is_the_corresponding_root(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected, version,
            );
        },
        UserOperation::RaiseFloor { version } => {
            lemma_a_version_named_in_the_compacted_history_is_the_corresponding_root(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected, version,
            );
        },
        UserOperation::Publish { logical: _ } => {},
        UserOperation::Checkpoint => {},
        UserOperation::Crash => {},
    }
}

// 整理的发布（只在整理那份里，现行内容与最后一个根相同）：k 加一，对应照旧；两层 F 各自至多挤上一格，
// oldest_still_shown 与 oldest_still_protected 各照 oldest_after_the_floor_moves 走。整理的发布要是带内存 F，持久 F 一次能跳过几个根，这里就断。
pub proof fn lemma_a_compaction_publish_keeps_the_relation(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
)
    requires
        compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        ),
        compaction_publish_is_accepted(with_compaction),
    ensures
        compaction_relation(
            without_compaction,
            observation_after_a_compaction_publish(with_compaction),
            compaction_publishes + 1,
            position,
            oldest_after_the_floor_moves(
                position,
                oldest_still_shown,
                with_compaction.in_memory_floor,
                observed_in_memory_floor_after_a_compaction_publish(with_compaction),
            ),
            oldest_after_the_floor_moves(
                position,
                oldest_still_protected,
                with_compaction.persisted_floor,
                observed_persisted_floor_after_a_compaction_publish(with_compaction),
            ),
        ),
{
    let after = observation_after_a_compaction_publish(with_compaction);
    lemma_a_floor_moving_up_by_at_most_one_keeps_the_correspondence(
        without_compaction.in_memory_floor,
        with_compaction.in_memory_floor,
        after.in_memory_floor,
        without_compaction.roots.len(),
        with_compaction.roots.len(),
        position,
        oldest_still_shown,
        compaction_publishes,
    );
    lemma_a_floor_moving_up_by_at_most_one_keeps_the_correspondence(
        without_compaction.persisted_floor,
        with_compaction.persisted_floor,
        after.persisted_floor,
        without_compaction.roots.len(),
        with_compaction.roots.len(),
        position,
        oldest_still_protected,
        compaction_publishes,
    );
    assert forall|index: int| 0 <= index < position.len() implies 0 <= #[trigger] position[index] < after.roots.len()
        && after.roots[position[index]] == without_compaction.roots[index] by {
        assert(after.roots[position[index]] == with_compaction.roots[position[index]]);
    }
    assert forall|with_index: int|
        0 <= with_index < after.roots.len() && (#[trigger] after.roots[with_index]).name is UserVersion implies exists|index: int|
        0 <= index < position.len() && #[trigger] position[index] == with_index by {
        if with_index < with_compaction.roots.len() {
            assert(after.roots[with_index] == with_compaction.roots[with_index]);
        } else {
            assert(after.roots[with_index].name == RootName::CompactionPublish);
        }
    }
    assert(after.roots[after.roots.len() - 1].logical == with_compaction.head_logical);
}

// 两份同时崩溃：内存 F 回到持久 F、现行回到最后一个根。崩溃之后的回退候选就是持久 F 那一层的对应：整理那份照样是不整理那份的后缀、至多少 k 个，
// 现行内容照样相同。
pub proof fn lemma_a_crash_of_both_keeps_the_relation(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
)
    requires
        compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        ),
    ensures
        compaction_relation(
            observation_after_user_operation(without_compaction, UserOperation::Crash),
            observation_after_user_operation(with_compaction, UserOperation::Crash),
            compaction_publishes,
            position,
            oldest_still_protected,
            oldest_still_protected,
        ),
{
    let without_after = observation_after_user_operation(without_compaction, UserOperation::Crash);
    let with_after = observation_after_user_operation(with_compaction, UserOperation::Crash);
    assert(without_after.roots == without_compaction.roots);
    assert(with_after.roots == with_compaction.roots);
}

// 用户的一步、两份都接受：对应关系照旧（发布落成根时 position 多一项，两层 F 一起照 oldest_after_the_floor_moves 走；
// 抬 F 时 oldest_still_shown 就是点到的那个根；崩溃时两层都取持久 F 那一层的）。
pub proof fn lemma_an_operation_accepted_by_both_keeps_the_relation(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
    operation: UserOperation,
)
    requires
        compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        ),
        user_operation_is_accepted(without_compaction, operation),
        user_operation_is_accepted(with_compaction, operation),
    ensures
        compaction_keeps_what_the_user_sees(
            observation_after_user_operation(without_compaction, operation),
            observation_after_user_operation(with_compaction, operation),
            compaction_publishes,
        ),
{
    let without_after = observation_after_user_operation(without_compaction, operation);
    let with_after = observation_after_user_operation(with_compaction, operation);
    match operation {
        UserOperation::Publish { logical: _ } => {
            assert(compaction_relation(
                without_after,
                with_after,
                compaction_publishes,
                position,
                oldest_still_shown,
                oldest_still_protected,
            ));
        },
        UserOperation::Rollback { version } => {
            lemma_a_version_named_in_the_compacted_history_is_the_corresponding_root(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected, version,
            );
            let without_index = index_of_the_candidate_version(
                without_compaction.roots,
                without_compaction.in_memory_floor,
                version,
            );
            assert(with_compaction.roots[position[without_index]] == without_compaction.roots[without_index]);
            assert(compaction_relation(
                without_after,
                with_after,
                compaction_publishes,
                position,
                oldest_still_shown,
                oldest_still_protected,
            ));
        },
        UserOperation::RaiseFloor { version } => {
            lemma_a_version_named_in_the_compacted_history_is_the_corresponding_root(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected, version,
            );
            let without_index = index_of_the_candidate_version(
                without_compaction.roots,
                without_compaction.in_memory_floor,
                version,
            );
            let length = without_compaction.roots.len() as int;
            lemma_strictly_increasing_positions_spread_out(position, without_index, length - 1);
            assert(position[length - 1] < with_compaction.roots.len());
            assert forall|index: int| 0 <= index < position.len() implies (with_after.in_memory_floor <= #[trigger] position[index]
                <==> without_index <= index) by {
                if index > without_index {
                    assert(position[without_index] < position[index]);
                } else if index < without_index {
                    assert(position[index] < position[without_index]);
                }
            }
            assert(compaction_relation(
                without_after,
                with_after,
                compaction_publishes,
                position,
                without_index,
                oldest_still_protected,
            ));
        },
        UserOperation::Checkpoint => {
            lemma_a_user_checkpoint_keeps_the_relation(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected,
            );
            let oldest_after = oldest_after_the_floor_moves(
                position,
                oldest_still_shown,
                with_compaction.in_memory_floor,
                observed_floor_after_a_checkpoint(with_compaction),
            );
            assert(compaction_relation(
                without_after,
                with_after,
                compaction_publishes,
                position.push(with_compaction.roots.len() as int),
                oldest_after,
                oldest_after,
            ));
        },
        UserOperation::Crash => {
            lemma_a_crash_of_both_keeps_the_relation(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected,
            );
            assert(compaction_relation(
                without_after,
                with_after,
                compaction_publishes,
                position,
                oldest_still_protected,
                oldest_still_protected,
            ));
        },
    }
}

// 用户的发布落成根（两份都做）：position 多一项指到整理那份的新根，内存 F 那一层的 oldest_still_shown 照 oldest_after_the_floor_moves 走，
// 持久 F 跟上内存 F，持久那一层取同一个。不整理那份挤掉最旧的候选时它的候选是满的，整理那份的候选不少于它、也满、也挤；
// oldest_still_shown 等于不整理那份的 F 时，整理那份最旧的候选就是 oldest_still_shown 那个根（lemma_corresponding_floors_count 的第三条）。
pub proof fn lemma_a_user_checkpoint_keeps_the_relation(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
)
    requires
        compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        ),
    ensures
        compaction_relation(
            observation_after_user_operation(without_compaction, UserOperation::Checkpoint),
            observation_after_user_operation(with_compaction, UserOperation::Checkpoint),
            compaction_publishes,
            position.push(with_compaction.roots.len() as int),
            oldest_after_the_floor_moves(
                position,
                oldest_still_shown,
                with_compaction.in_memory_floor,
                observed_floor_after_a_checkpoint(with_compaction),
            ),
            oldest_after_the_floor_moves(
                position,
                oldest_still_shown,
                with_compaction.in_memory_floor,
                observed_floor_after_a_checkpoint(with_compaction),
            ),
        ),
{
    lemma_corresponding_floors_count(
        without_compaction.in_memory_floor,
        with_compaction.in_memory_floor,
        without_compaction.roots.len(),
        with_compaction.roots.len(),
        position,
        oldest_still_shown,
        compaction_publishes,
    );
    let without_after = observation_after_user_operation(without_compaction, UserOperation::Checkpoint);
    let with_after = observation_after_user_operation(with_compaction, UserOperation::Checkpoint);
    let new_root = ObservedRoot {
        logical: without_compaction.head_logical,
        name: RootName::UserVersion { version: without_compaction.next_user_version },
    };
    let position_after = position.push(with_compaction.roots.len() as int);
    let oldest_still_shown_after = oldest_after_the_floor_moves(
        position,
        oldest_still_shown,
        with_compaction.in_memory_floor,
        observed_floor_after_a_checkpoint(with_compaction),
    );
    let without_length = without_compaction.roots.len() as int;
    let with_length = with_compaction.roots.len() as int;
    let without_floor = without_compaction.in_memory_floor as int;
    let with_floor = with_compaction.in_memory_floor as int;
    assert(with_after.roots[with_length] == new_root);
    assert(without_after.roots[without_length] == new_root);
    assert forall|index: int| 0 <= index < without_after.roots.len() implies (#[trigger] without_after.roots[index]).name is UserVersion
        && version_of_root(without_after.roots[index]) < without_after.next_user_version by {
        if index < without_length {
            assert(without_after.roots[index] == without_compaction.roots[index]);
        }
    }
    assert forall|earlier: int, later: int|
        #![trigger without_after.roots[earlier], without_after.roots[later]]
        0 <= earlier < later < without_after.roots.len() implies version_of_root(without_after.roots[earlier])
        < version_of_root(without_after.roots[later]) by {
        assert(without_after.roots[earlier] == without_compaction.roots[earlier]);
        if later < without_length {
            assert(without_after.roots[later] == without_compaction.roots[later]);
        }
    }
    assert forall|index: int| 0 <= index < position_after.len() implies 0 <= #[trigger] position_after[index]
        < with_after.roots.len() && with_after.roots[position_after[index]] == without_after.roots[index] by {
        if index < without_length {
            assert(position_after[index] == position[index]);
            assert(with_after.roots[position[index]] == with_compaction.roots[position[index]]);
            assert(without_after.roots[index] == without_compaction.roots[index]);
        }
    }
    assert forall|earlier: int, later: int|
        #![trigger position_after[earlier], position_after[later]]
        0 <= earlier < later < position_after.len() implies position_after[earlier] < position_after[later] by {
        assert(position_after[earlier] == position[earlier]);
        if later < without_length {
            assert(position_after[later] == position[later]);
        }
    }
    assert forall|with_index: int|
        0 <= with_index < with_after.roots.len() && (#[trigger] with_after.roots[with_index]).name is UserVersion implies exists|index: int|
        0 <= index < position_after.len() && #[trigger] position_after[index] == with_index by {
        if with_index < with_length {
            assert(with_after.roots[with_index] == with_compaction.roots[with_index]);
            let index = choose|index: int| 0 <= index < position.len() && #[trigger] position[index] == with_index;
            assert(position_after[index] == position[index]);
        } else {
            assert(position_after[without_length] == with_index);
        }
    }
    assert forall|index: int| 0 <= index < position_after.len() implies (with_after.in_memory_floor
        <= #[trigger] position_after[index] <==> oldest_still_shown_after <= index) by {
        if index < without_length {
            assert(position_after[index] == position[index]);
            if with_after.in_memory_floor != with_floor && oldest_still_shown < position.len() && position[oldest_still_shown] == with_floor {
                if index > oldest_still_shown {
                    assert(position[oldest_still_shown] < position[index]);
                } else if index < oldest_still_shown {
                    assert(position[index] < position[oldest_still_shown]);
                }
            } else if with_after.in_memory_floor != with_floor {
                if index > oldest_still_shown {
                    assert(position[oldest_still_shown] < position[index]);
                }
            }
        }
    }
    if without_after.in_memory_floor != without_floor {
        // 不整理那份挤掉了最旧的候选：它的候选是满的，整理那份也是满的、也挤；oldest_still_shown 等于不整理那份的 F 时
        // 整理那份最旧的候选就是 oldest_still_shown 那个根。
        assert(with_after.in_memory_floor != with_floor);
        if oldest_still_shown == without_floor {
            assert(position[oldest_still_shown] == with_floor);
        }
    }
    assert(floors_correspond(
        without_after.in_memory_floor,
        with_after.in_memory_floor,
        without_after.roots.len(),
        with_after.roots.len(),
        position_after,
        oldest_still_shown_after,
        compaction_publishes,
    ));
    assert(compaction_relation(
        without_after,
        with_after,
        compaction_publishes,
        position_after,
        oldest_still_shown_after,
        oldest_still_shown_after,
    ));
}

// G7 的历史：一串事件，用户的一步（含崩溃）两份都做，整理的发布只在整理那份里做。
pub enum ObservedEvent {
    User { operation: UserOperation },
    CompactionPublish,
}

pub open spec fn observation_without_compaction(start: RingObservation, events: Seq<ObservedEvent>, count: int) -> RingObservation
    decreases count,
{
    if count <= 0 {
        start
    } else {
        let before = observation_without_compaction(start, events, count - 1);
        match events[count - 1] {
            ObservedEvent::User { operation } => observation_after_user_operation(before, operation),
            ObservedEvent::CompactionPublish => before,
        }
    }
}

pub open spec fn observation_with_compaction(start: RingObservation, events: Seq<ObservedEvent>, count: int) -> RingObservation
    decreases count,
{
    if count <= 0 {
        start
    } else {
        let before = observation_with_compaction(start, events, count - 1);
        match events[count - 1] {
            ObservedEvent::User { operation } => observation_after_user_operation(before, operation),
            ObservedEvent::CompactionPublish => observation_after_a_compaction_publish(before),
        }
    }
}

// k：前 count 个事件里整理占的发布数。
pub open spec fn compaction_publishes_among(events: Seq<ObservedEvent>, count: int) -> nat
    decreases count,
{
    if count <= 0 {
        0
    } else {
        match events[count - 1] {
            ObservedEvent::User { operation: _ } => compaction_publishes_among(events, count - 1),
            ObservedEvent::CompactionPublish => compaction_publishes_among(events, count - 1) + 1,
        }
    }
}

pub open spec fn event_is_accepted_by_both(start: RingObservation, events: Seq<ObservedEvent>, index: int) -> bool {
    match events[index] {
        ObservedEvent::User { operation } => user_operation_is_accepted(
            observation_without_compaction(start, events, index),
            operation,
        ) && user_operation_is_accepted(observation_with_compaction(start, events, index), operation),
        ObservedEvent::CompactionPublish => compaction_publish_is_accepted(observation_with_compaction(start, events, index)),
    }
}

// G7：从同一个起点出发（每个根都是用户发布的、版本号按次序），同一串用户历史（可以有崩溃）两份都接受了每一步，整理那份另插 k 次整理的发布；
// 前 count 个事件之后，现行内容相同，整理那份的回退候选（按版本号）是不整理那份候选的后缀、至多少 k 个，抬 F 的上限恰多 k；
// 持久 F 之上的根同样是后缀、至多少 k 个，所以下一次崩溃之后的回退候选也是。
pub proof fn theorem_compaction_keeps_what_the_user_sees(start: RingObservation, events: Seq<ObservedEvent>, count: nat)
    requires
        observation_is_shaped(start),
        every_root_is_a_user_version_in_order(start),
        count <= events.len(),
        forall|index: int| 0 <= index < events.len() ==> #[trigger] event_is_accepted_by_both(start, events, index),
    ensures
        compaction_keeps_what_the_user_sees(
            observation_without_compaction(start, events, count as int),
            observation_with_compaction(start, events, count as int),
            compaction_publishes_among(events, count as int),
        ),
    decreases count,
{
    if count == 0 {
        let identity = Seq::new(start.roots.len(), |index: int| index);
        assert forall|with_index: int|
            0 <= with_index < start.roots.len() && (#[trigger] start.roots[with_index]).name is UserVersion implies exists|index: int|
            0 <= index < identity.len() && #[trigger] identity[index] == with_index by {
            assert(identity[with_index] == with_index);
        }
        assert(compaction_relation(
            start,
            start,
            0,
            identity,
            start.in_memory_floor as int,
            start.persisted_floor as int,
        ));
    } else {
        let previous = (count - 1) as nat;
        theorem_compaction_keeps_what_the_user_sees(start, events, previous);
        let without_before = observation_without_compaction(start, events, previous as int);
        let with_before = observation_with_compaction(start, events, previous as int);
        let publishes_before = compaction_publishes_among(events, previous as int);
        let (position, oldest_still_shown, oldest_still_protected) = choose|
            position: Seq<int>,
            oldest_still_shown: int,
            oldest_still_protected: int,
        |
            #[trigger] compaction_relation(
                without_before,
                with_before,
                publishes_before,
                position,
                oldest_still_shown,
                oldest_still_protected,
            );
        assert(event_is_accepted_by_both(start, events, previous as int));
        match events[previous as int] {
            ObservedEvent::User { operation } => {
                lemma_an_operation_accepted_by_both_keeps_the_relation(
                    without_before, with_before, publishes_before, position, oldest_still_shown, oldest_still_protected,
                    operation,
                );
            },
            ObservedEvent::CompactionPublish => {
                lemma_a_compaction_publish_keeps_the_relation(
                    without_before, with_before, publishes_before, position, oldest_still_shown, oldest_still_protected,
                );
                assert(compaction_relation(
                    without_before,
                    observation_after_a_compaction_publish(with_before),
                    publishes_before + 1,
                    position,
                    oldest_after_the_floor_moves(
                        position,
                        oldest_still_shown,
                        with_before.in_memory_floor,
                        observed_in_memory_floor_after_a_compaction_publish(with_before),
                    ),
                    oldest_after_the_floor_moves(
                        position,
                        oldest_still_protected,
                        with_before.persisted_floor,
                        observed_persisted_floor_after_a_compaction_publish(with_before),
                    ),
                ));
            },
        }
    }
}

} // verus!
