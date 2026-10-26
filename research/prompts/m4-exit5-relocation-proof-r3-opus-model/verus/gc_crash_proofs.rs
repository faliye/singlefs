//! 同一逻辑版本的 GC 搬迁协议：持久状态、留存读者与任意有限崩溃历史。
//!
//! # 证明入口与范围
//! `lemma_gc_step_preserves_current_durable_and_old_readers` 证明一步保持；
//! `theorem_gc_every_finite_crash_history_preserves_reads` 对合法历史的前缀长度归纳。
//! 两者都以 `gc_all_views_match` 和 `gc_step_enabled` 为前提；本模块不证明真实设备
//! 或调用方自动满足这些前提。只有 [`released_record_can_be_reclaimed`] 被产品执行。
//!
//! 所有读者共享同一个 `logical`，连 key 的定义域也相同。因此“旧”指搬迁前的物理映射，
//! 不包含用户修改前的历史逻辑版本；AcquireReader 不是持久快照创建，Crash 会丢掉它。
//! 完整定义、逐条推导、反例及尚未连接的产品语义见同目录 `README.md`。
//!
//! 这是以槽为地址的协议模型；设备/校验和/解析器不是本模块的已证实现。
//! Write 可以把任意字节（包括撕裂数据）写入未被任何根或读者引用的槽；
//! Relocate 只有在目的字节已持久且与源相等时才发布映射。
//! Checkpoint 保存当前映射；Crash 丢弃未 checkpoint 的映射并恢复持久根。
//! 每一步均保持当前、持久根和旧读者的逻辑字节，归纳覆盖任意有限步数。
//! journal 完整提交的重放应映射为 Relocate/Checkpoint，解码和映射仍需调用方证明。
use vstd::prelude::*;

verus! {

// 协议状态，而非 core 的内存布局：disk 只表示已持久的抽象数据单元。
// current 是当前发布视图，durable 是模型中唯一的恢复根；readers 是进程内仍持有的映射。
// 没有设备编号、span、根环、写缓存和 journal 编码，不能将其字段直接等同于盘上结构。
pub struct GcCrashState {
    pub disk: Map<u64, Seq<u8>>,
    pub current: Map<u64, u64>,
    pub durable: Map<u64, u64>,
    pub readers: Seq<Map<u64, u64>>,
}

// 七种抽象步骤。Write 对未引用地址可写任意字节，并不表示一次设备写原子持久化。
// Checkpoint 原子切换恢复根是模型定义，真实屏障/根槽/journal 如何实现它尚未证明。
pub enum GcCrashStep {
    Write { slot: u64, bytes: Seq<u8> },
    Relocate { key: u64, destination: u64 },
    Checkpoint,
    AcquireReader,
    ReleaseReader { index: int },
    Reclaim { slot: u64 },
    Crash,
}

// 读抽象：映射 key 集合必须恰好等于 logical 的 key 集合，且每个映射目标存在、字节相同。
// 显式 contains_key 避免在 Map 的未定义项上读任意值而误认为满足规格。
pub open spec fn gc_view_matches(
    mapping: Map<u64, u64>, disk: Map<u64, Seq<u8>>, logical: Map<u64, Seq<u8>>,
) -> bool {
    &&& mapping.dom() == logical.dom()
    &&& forall|key: u64| mapping.contains_key(key) ==>
        disk.contains_key(#[trigger] mapping[key]) && disk[mapping[key]] == logical[key]
}

// 一个逻辑版本的所有物理视图均可读。旧读者可以继续指向旧地址，但必须读到相同字节。
// 此处没有量化不同读者各自的历史逻辑版本。
pub open spec fn gc_all_views_match(s: GcCrashState, logical: Map<u64, Seq<u8>>) -> bool {
    &&& gc_view_matches(s.current, s.disk, logical)
    &&& gc_view_matches(s.durable, s.disk, logical)
    &&& forall|i: int| 0 <= i < s.readers.len() ==>
        gc_view_matches(#[trigger] s.readers[i], s.disk, logical)
}

// 对所有有效映射项检查物理地址；不要求 mapping 单射，同字节的别名不会被忽略。
pub open spec fn gc_map_avoids(mapping: Map<u64, u64>, slot: u64) -> bool {
    forall|key: u64| mapping.contains_key(key) ==> #[trigger] mapping[key] != slot
}

// 可覆盖/删除的必要保护条件：当前根、持久根、每个留存读者三类缺一不可。
// 真实分配器必须另证其回收条件蕴涵这一条件；本模型没有从 floor 推出它。
pub open spec fn gc_slot_is_unreferenced(s: GcCrashState, slot: u64) -> bool {
    &&& gc_map_avoids(s.current, slot)
    &&& gc_map_avoids(s.durable, slot)
    &&& forall|i: int| 0 <= i < s.readers.len() ==> gc_map_avoids(#[trigger] s.readers[i], slot)
}

// 协议允许条件，不是对真实系统是否执行了检查的证明。
// Relocate 只要求已持久的源/目的字节相等；不要求目的地址新鲜或推进 GC，故也允许原地/no-op 搬迁。
// 完整字节相等强于校验和相等；“拒绝撕裂”定理没有证明 CRC 无碰撞。
pub open spec fn gc_step_enabled(s: GcCrashState, step: GcCrashStep) -> bool {
    match step {
        GcCrashStep::Write { slot, .. } | GcCrashStep::Reclaim { slot } =>
            gc_slot_is_unreferenced(s, slot),
        GcCrashStep::Relocate { key, destination } => {
            &&& s.current.contains_key(key)
            &&& s.disk.contains_key(s.current[key])
            &&& s.disk.contains_key(destination)
            &&& s.disk[destination] == s.disk[s.current[key]]
        },
        GcCrashStep::ReleaseReader { index } => 0 <= index < s.readers.len(),
        _ => true,
    }
}

// 纯状态转换。应与 gc_step_enabled 成对使用；单独调用并不拒绝不合法步骤。
// Crash 的恢复结果是此处的定义，后面的幂等定理不是对 recovery.rs 解析和择根算法的独立证明。
pub open spec fn gc_next(s: GcCrashState, step: GcCrashStep) -> GcCrashState {
    match step {
        GcCrashStep::Write { slot, bytes } => GcCrashState { disk: s.disk.insert(slot, bytes), ..s },
        GcCrashStep::Reclaim { slot } => GcCrashState { disk: s.disk.remove(slot), ..s },
        GcCrashStep::Relocate { key, destination } =>
            GcCrashState { current: s.current.insert(key, destination), ..s },
        GcCrashStep::Checkpoint => GcCrashState { durable: s.current, ..s },
        GcCrashStep::AcquireReader => GcCrashState { readers: s.readers.push(s.current), ..s },
        GcCrashStep::ReleaseReader { index } =>
            GcCrashState { readers: s.readers.remove(index), ..s },
        GcCrashStep::Crash => GcCrashState { current: s.durable, readers: Seq::empty(), ..s },
    }
}

// 局部帧性质：只改不被 mapping 引用的地址，每个 key 的目标仍存在且保持旧字节。
// 空 proof 体表示 SMT 能直接由 Map 的 insert/remove 规格推出，不表示跳过验证。
pub proof fn lemma_gc_write_keeps_a_protected_view(
    mapping: Map<u64, u64>, disk: Map<u64, Seq<u8>>, logical: Map<u64, Seq<u8>>,
    slot: u64, bytes: Seq<u8>,
)
    requires
        gc_view_matches(mapping, disk, logical),
        gc_map_avoids(mapping, slot),
    ensures
        gc_view_matches(mapping, disk.insert(slot, bytes), logical),
        gc_view_matches(mapping, disk.remove(slot), logical),
{
}

// 按七种步骤分类：Write/Reclaim 用帧引理；Relocate 分被搬 key 与其余 key；
// Acquire/Release 分追加/删除前后的读者下标；Checkpoint/Crash 复用已证安全的视图。
pub proof fn lemma_gc_step_preserves_current_durable_and_old_readers(
    s: GcCrashState, step: GcCrashStep, logical: Map<u64, Seq<u8>>,
)
    requires
        gc_all_views_match(s, logical),
        gc_step_enabled(s, step),
    ensures
        gc_all_views_match(gc_next(s, step), logical),
{
    match step {
        GcCrashStep::Write { slot, bytes } => {
            lemma_gc_write_keeps_a_protected_view(s.current, s.disk, logical, slot, bytes);
            lemma_gc_write_keeps_a_protected_view(s.durable, s.disk, logical, slot, bytes);
            assert forall|i: int| 0 <= i < s.readers.len() implies
                gc_view_matches(#[trigger] s.readers[i], s.disk.insert(slot, bytes), logical) by {
                lemma_gc_write_keeps_a_protected_view(s.readers[i], s.disk, logical, slot, bytes);
            }
        },
        GcCrashStep::Reclaim { slot } => {
            lemma_gc_write_keeps_a_protected_view(s.current, s.disk, logical, slot, Seq::empty());
            lemma_gc_write_keeps_a_protected_view(s.durable, s.disk, logical, slot, Seq::empty());
            assert forall|i: int| 0 <= i < s.readers.len() implies
                gc_view_matches(#[trigger] s.readers[i], s.disk.remove(slot), logical) by {
                lemma_gc_write_keeps_a_protected_view(s.readers[i], s.disk, logical, slot, Seq::empty());
            }
        },
        GcCrashStep::ReleaseReader { index } => {
            assert forall|i: int| 0 <= i < s.readers.remove(index).len() implies
                gc_view_matches(#[trigger] s.readers.remove(index)[i], s.disk, logical) by {
                if i < index { assert(s.readers.remove(index)[i] == s.readers[i]); }
                else { assert(s.readers.remove(index)[i] == s.readers[i + 1]); }
            }
        },
        GcCrashStep::Relocate { key, destination } => {
            assert(s.disk[s.current[key]] == logical[key]);
            assert forall|k: u64| s.current.insert(key, destination).contains_key(k) implies
                s.disk.contains_key(#[trigger] s.current.insert(key, destination)[k])
                    && s.disk[s.current.insert(key, destination)[k]] == logical[k] by {
                if k != key { assert(s.current.insert(key, destination)[k] == s.current[k]); }
            }
            assert(s.current.insert(key, destination).dom() =~= s.current.dom());
            assert(gc_view_matches(s.current.insert(key, destination), s.disk, logical));
        },
        GcCrashStep::AcquireReader => {
            assert forall|i: int| 0 <= i < s.readers.push(s.current).len() implies
                gc_view_matches(#[trigger] s.readers.push(s.current)[i], s.disk, logical) by {
                if i < s.readers.len() { assert(s.readers.push(s.current)[i] == s.readers[i]); }
                else { assert(s.readers.push(s.current)[i] == s.current); }
            }
        },
        GcCrashStep::Checkpoint => {},
        GcCrashStep::Crash => {},
    }
}

// 与单个运行长度无关；重复 crash/recovery、读者取得/释放和任意槽写都在归纳域中。
// count 是最后一个要证明的状态下标（包含端点），不是 steps 的下标。
// states 比 steps 多一个；count=0 用初始不变量，count>0 用前缀归纳假设及最后一步保持。
// 取 count=steps.len() 得到整段历史结论；没有固定长度上界，也不证明无限运行的进展。
pub proof fn theorem_gc_every_finite_crash_history_preserves_reads(
    states: Seq<GcCrashState>, steps: Seq<GcCrashStep>, logical: Map<u64, Seq<u8>>, count: nat,
)
    requires
        states.len() == steps.len() + 1,
        count < states.len(),
        gc_all_views_match(states[0], logical),
        forall|i: int| 0 <= i < steps.len() ==>
            gc_step_enabled(#[trigger] states[i], steps[i]) && states[i + 1] == gc_next(states[i], steps[i]),
    ensures
        forall|i: int| 0 <= i <= count ==> gc_all_views_match(#[trigger] states[i], logical),
    decreases count,
{
    if count > 0 {
        theorem_gc_every_finite_crash_history_preserves_reads(states, steps, logical, (count - 1) as nat);
        lemma_gc_step_preserves_current_durable_and_old_readers(states[count - 1], steps[count - 1], logical);
        assert(states[count as int] == gc_next(states[count - 1], steps[count - 1]));
    }
}

// 检查抽象转换的恢复结果与幂等性。无 requires，任意抽象状态都适用；
// 但合法盘面能否解出这个 durable 映射，属于尚未完成的实现精化。
pub proof fn theorem_gc_crash_recovers_checkpoint_and_is_idempotent(s: GcCrashState)
    ensures
        gc_next(s, GcCrashStep::Crash).current == s.durable,
        gc_next(s, GcCrashStep::Crash).disk == s.disk,
        gc_next(gc_next(s, GcCrashStep::Crash), GcCrashStep::Crash)
            == gc_next(s, GcCrashStep::Crash),
        gc_next(gc_next(s, GcCrashStep::Checkpoint), GcCrashStep::Crash).current == s.current,
{
}

// 给出任意一个有效读者及其 key 即足以反驳“无人引用”，不依赖字节正确性不变量。
// 这是保护条件的逻辑后果，不意味着产品存在同形状的 reader 注册表。
pub proof fn lemma_gc_any_old_reader_prevents_source_reclamation(s: GcCrashState, reader: int, key: u64)
    requires
        0 <= reader < s.readers.len(),
        s.readers[reader].contains_key(key),
    ensures
        !gc_step_enabled(s, GcCrashStep::Reclaim { slot: s.readers[reader][key] }),
        !gc_step_enabled(s, GcCrashStep::Write { slot: s.readers[reader][key], bytes: Seq::empty() }),
{
    assert(!gc_map_avoids(s.readers[reader], s.readers[reader][key]));
}

// 已知源和目的均存在且完整字节不同，就不满足搬迁前提；没有建模检测过程或设备错误概率。
pub proof fn lemma_gc_torn_destination_cannot_be_published(s: GcCrashState, key: u64, destination: u64)
    requires
        s.current.contains_key(key),
        s.disk.contains_key(s.current[key]),
        s.disk.contains_key(destination),
        s.disk[destination] != s.disk[s.current[key]],
    ensures
        !gc_step_enabled(s, GcCrashStep::Relocate { key, destination }),
{
}

// 真实 allocator 调此判定。generation 是释放版本，floor 由根环/回退/隔离计算。
// 唯一产品可执行入口：保留 <= 的相等边界及 u64 全宽，不计算 floor、不改变回收时机。
pub fn released_record_can_be_reclaimed(released: bool, generation: u64, floor: u64) -> (answer: bool)
    ensures
        answer == (released && generation <= floor),
{
    released && generation <= floor
}

// 条件算术引理：floor <= reader_txg < generation 推出 generation > floor。
// reader_txg < generation 及 floor <= reader_txg 都是调用方义务，不是由 release 标志自动得到。
// 与地址级 gc_slot_is_unreferenced 之间没有已证明的转换桥梁。
pub proof fn lemma_gc_old_reader_blocks_reuse(released: bool, generation: u64, floor: u64, reader_txg: u64)
    requires
        floor <= reader_txg,
        reader_txg < generation,
    ensures
        !(released && generation <= floor),
{
}

// 无前提的构造见证：key=7，源=10，目的=20，字节=[42]，一个持有源地址的读者。
// 函数体逐步验证复制/换映射/保存根/释放/回收和崩溃分支，防止这些前提仅在空映射上成立。
// ensures 只暴露初始非空状态的存在；函数体断言被验证，但调用者不能据此取得一份完整历史。
pub proof fn witness_gc_nonempty_copy_publish_crash_and_reader_release()
    ensures
        exists|s: GcCrashState, logical: Map<u64, Seq<u8>>|
            gc_all_views_match(s, logical) && logical.contains_key(7)
                && s.readers.len() == 1 && !gc_slot_is_unreferenced(s, 10),
{
    let bytes = seq![42u8];
    let logical = Map::empty().insert(7u64, bytes);
    let mapping = Map::empty().insert(7u64, 10u64);
    let s = GcCrashState { disk: Map::empty().insert(10u64, bytes), current: mapping,
        durable: mapping, readers: seq![mapping] };
    assert(gc_all_views_match(s, logical));
    let copied = gc_next(s, GcCrashStep::Write { slot: 20, bytes });
    lemma_gc_step_preserves_current_durable_and_old_readers(s, GcCrashStep::Write { slot: 20, bytes }, logical);
    let moved = gc_next(copied, GcCrashStep::Relocate { key: 7, destination: 20 });
    lemma_gc_step_preserves_current_durable_and_old_readers(copied, GcCrashStep::Relocate { key: 7, destination: 20 }, logical);
    assert(!gc_step_enabled(moved, GcCrashStep::Reclaim { slot: 10 }));
    let checkpointed = gc_next(moved, GcCrashStep::Checkpoint);
    lemma_gc_step_preserves_current_durable_and_old_readers(moved, GcCrashStep::Checkpoint, logical);
    assert(checkpointed.readers[0][7] == 10);
    assert(checkpointed.readers[0].contains_key(7));
    assert(!gc_map_avoids(checkpointed.readers[0], 10));
    assert(!gc_step_enabled(checkpointed, GcCrashStep::Reclaim { slot: 10 }));
    let released = gc_next(checkpointed, GcCrashStep::ReleaseReader { index: 0 });
    lemma_gc_step_preserves_current_durable_and_old_readers(checkpointed, GcCrashStep::ReleaseReader { index: 0 }, logical);
    assert(gc_step_enabled(released, GcCrashStep::Reclaim { slot: 10 }));
    lemma_gc_step_preserves_current_durable_and_old_readers(released, GcCrashStep::Reclaim { slot: 10 }, logical);
    lemma_gc_step_preserves_current_durable_and_old_readers(checkpointed, GcCrashStep::Crash, logical);
    assert(gc_all_views_match(s, logical) && logical.contains_key(7)
        && s.readers.len() == 1 && !gc_slot_is_unreferenced(s, 10));
}
}
