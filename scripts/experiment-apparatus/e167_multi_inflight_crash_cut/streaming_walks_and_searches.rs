//! 两盘几何的流式遍历：位集与步、交错的遍历、签名的求值、搜索器与按搜索求真值。

use e7_index_bench::{low_half_of_u128, u8_from_u32};
use super::*;

#[derive(Default)]
pub(super) struct MixingHasher {
    state: u64,
}

impl MixingHasher {
    fn mix(&mut self, word: u64) {
        self.state = (self.state.rotate_left(5) ^ word).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
}

impl Hasher for MixingHasher {
    fn finish(&self) -> u64 {
        let mut value = self.state;
        value ^= value >> 33;
        value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
        value ^= value >> 33;
        value
    }

    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0u8; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.mix(u64::from_le_bytes(word));
        }
    }

    fn write_u128(&mut self, value: u128) {
        self.mix(low_half_of_u128(value));
        self.mix((value >> 64) as u64);
    }
}

pub(super) type FastMap<Key, Value> = HashMap<Key, Value, BuildHasherDefault<MixingHasher>>;
type FastSet<Key> = std::collections::HashSet<Key, BuildHasherDefault<MixingHasher>>;

pub(super) struct Bitset {
    words: Vec<u64>,
}

impl Bitset {
    pub(super) fn new(bits: u64) -> Bitset {
        Bitset { words: vec![0; usize::try_from(bits.div_ceil(64)).expect("位集大小装得进 usize")] }
    }

    pub(super) fn insert_index(&mut self, index: u64) {
        self.words[(index >> 6) as usize] |= 1u64 << (index & 63);
    }

    pub(super) fn has(&self, index: u64) -> bool {
        self.words[(index >> 6) as usize] & (1u64 << (index & 63)) != 0
    }

    fn or_with(&mut self, other: &Bitset) {
        for (word, other_word) in self.words.iter_mut().zip(&other.words) {
            *word |= *other_word;
        }
    }

    fn indices(&self) -> Vec<u64> {
        let mut found = Vec::new();
        for (word_position, word) in self.words.iter().enumerate() {
            let mut remaining = *word;
            while remaining != 0 {
                found.push(word_position as u64 * 64 + u64::from(remaining.trailing_zeros()));
                remaining &= remaining - 1;
            }
        }
        found
    }
}

pub(super) const MAXIMUM_STREAMING_WRITES: usize = 20;

/// 写表的定长摘要：每次写的取值个数（两态 2、原地覆写 3）、混合进制的权、各谓词用的位掩码（每次写两位，0 / 1 / 2）。
pub(super) struct WriteTableSummary {
    pub(super) write_count: usize,
    pub(super) disk_count: usize,
    radix: Vec<u8>,
    weight: Vec<u64>,
    pub(super) total_vectors: u64,
    writes_on_disk: Vec<u32>,
    older_owner_any: u64,
    older_owner_ones: u64,
    newer_owner_any: u64,
    newer_owner_ones: u64,
    older_record_any: u64,
    older_record_ones: u64,
    newer_record_any: u64,
    newer_record_ones: u64,
}

impl WriteTableSummary {
    pub(super) fn new(table: &[WriteSpecification]) -> WriteTableSummary {
        let write_count = table.len();
        assert!(write_count <= MAXIMUM_STREAMING_WRITES, "流式路径的写表最多 20 次写");
        let radix: Vec<u8> = table.iter().map(|specification| if specification.is_in_place { 3 } else { 2 }).collect();
        let mut weight = Vec::new();
        let mut running = 1u64;
        for &write_radix in &radix {
            weight.push(running);
            running *= u64::from(write_radix);
        }
        let disk_count = table.iter().map(|specification| specification.disk).max().map_or(1, |highest| highest + 1);
        assert!(disk_count <= 2, "搜索状态里的未放行盘集合只留两位");
        let mut writes_on_disk = vec![0u32; disk_count];
        for (write_index, specification) in table.iter().enumerate() {
            writes_on_disk[specification.disk] |= 1u32 << write_index;
        }
        let masks_of = |select: &dyn Fn(&WriteSpecification) -> bool| -> (u64, u64) {
            let mut any = 0u64;
            let mut ones = 0u64;
            for (write_index, specification) in table.iter().enumerate() {
                if select(specification) {
                    any |= 0b11u64 << (2 * write_index);
                    ones |= 0b01u64 << (2 * write_index);
                }
            }
            (any, ones)
        };
        let (older_owner_any, older_owner_ones) = masks_of(&|specification| specification.owner == Owner::OlderTransaction);
        let (newer_owner_any, newer_owner_ones) = masks_of(&|specification| specification.owner == Owner::NewerTransaction);
        let (older_record_any, older_record_ones) = masks_of(&|specification| specification.owner == Owner::OlderTransaction && specification.kind == StepKind::JournalRecord);
        let (newer_record_any, newer_record_ones) = masks_of(&|specification| specification.owner == Owner::NewerTransaction && specification.kind == StepKind::JournalRecord);
        WriteTableSummary {
            write_count,
            disk_count,
            radix,
            weight,
            total_vectors: running,
            writes_on_disk,
            older_owner_any,
            older_owner_ones,
            newer_owner_any,
            newer_owner_ones,
            older_record_any,
            older_record_ones,
            newer_record_any,
            newer_record_ones,
        }
    }

    /// 混合进制序号 → 每次写两位的编码。
    pub(super) fn code_of_index(&self, mut index: u64) -> u64 {
        let mut code = 0u64;
        for write in 0..self.write_count {
            let write_radix = u64::from(self.radix[write]);
            code |= (index % write_radix) << (2 * write);
            index /= write_radix;
        }
        code
    }

    /// 每次写两位的编码 → 混合进制序号（`code_of_index` 的反向）。
    pub(super) fn index_of_code(&self, code: u64) -> u64 {
        (0..self.write_count).map(|write| ((code >> (2 * write)) & 0b11) * self.weight[write]).sum()
    }

    pub(super) fn is_cross(&self, code: u64) -> bool {
        code & self.older_owner_any != 0 && code & self.newer_owner_any != 0
    }

    pub(super) fn is_invalid_by_record(&self, code: u64) -> bool {
        code & self.newer_record_any == self.newer_record_ones && code & self.older_record_any != self.older_record_ones
    }

    pub(super) fn is_invalid_by_all_writes(&self, code: u64) -> bool {
        code & self.newer_owner_any == self.newer_owner_ones && code & self.older_owner_any != 0 && code & self.older_owner_any != self.older_owner_ones
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Step {
    Write { index: u8, disk: u8, fua: bool },
    Barrier { disk: u8 },
}

pub(super) fn steps_of(table: &[WriteSpecification], chain: &[Event]) -> Vec<Step> {
    chain
        .iter()
        .map(|event| match *event {
            Event::Write(write_index) => Step::Write {
                index: u8::try_from(write_index).expect("写下标装得进 u8"),
                disk: u8::try_from(table[write_index].disk).expect("盘号装得进 u8"),
                fua: table[write_index].kind == StepKind::RootRecordFua,
            },
            Event::Barrier(disk) => Step::Barrier { disk: u8::try_from(disk).expect("盘号装得进 u8") },
        })
        .collect()
}

/// 两条链与第二条链的等待表（走过 A 的第几步之后 B 的这一步才可发出）。
pub(super) struct WalkContext {
    pub(super) older_steps: Vec<Step>,
    pub(super) newer_steps: Vec<Step>,
    newer_waits: Vec<Option<usize>>,
    writes_on_disk: Vec<u32>,
}

impl WalkContext {
    pub(super) fn new(world: &World, table_summary: &WriteTableSummary) -> WalkContext {
        let older_steps = steps_of(&world.table, &world.search_chains[0]);
        let newer_steps = steps_of(&world.table, &world.search_chains[1]);
        assert!(older_steps.len() <= 31 && newer_steps.len() <= 31, "搜索状态里每条链的位置只留五位");
        let newer_waits = if newer_steps.is_empty() { Vec::new() } else { world.search_waits_of_second_chain.clone() };
        WalkContext { older_steps, newer_steps, newer_waits, writes_on_disk: table_summary.writes_on_disk.clone() }
    }

    fn newer_step_may_follow(&self, older_position: usize, newer_position: usize) -> bool {
        self.newer_waits[newer_position].is_none_or(|needed| older_position > needed)
    }
}

/// 逐条数 Σ(P)：在（A 走到哪、B 走到哪）的格子上数路径，B 的等待表限制能走哪条边。
pub(super) fn count_interleavings(context: &WalkContext) -> u128 {
    let (older_length, newer_length) = (context.older_steps.len(), context.newer_steps.len());
    let mut ways = vec![vec![0u128; newer_length + 1]; older_length + 1];
    ways[0][0] = 1;
    for older_position in 0..=older_length {
        for newer_position in 0..=newer_length {
            let here = ways[older_position][newer_position];
            if here == 0 {
                continue;
            }
            if older_position < older_length {
                ways[older_position + 1][newer_position] += here;
            }
            if newer_position < newer_length && context.newer_step_may_follow(older_position, newer_position) {
                ways[older_position][newer_position + 1] += here;
            }
        }
    }
    ways[older_length][newer_length]
}

/// 深搜走到一半的状态。`key` 每次写占五位：它发出时已关了几段（段号）；`path` 每步一位（0 = A、1 = B），高位在前。
#[derive(Clone, Copy)]
pub(super) struct Walk {
    pub(super) older_position: u8,
    pub(super) newer_position: u8,
    unreleased: u8,
    writes_in_current: u8,
    closed: u8,
    issued: u32,
    armed: u32,
    forced: u32,
    pub(super) key: u128,
    pub(super) path: u64,
}

impl Walk {
    pub(super) fn start() -> Walk {
        Walk { older_position: 0, newer_position: 0, unreleased: 0, writes_in_current: 0, closed: 0, issued: 0, armed: 0, forced: 0, key: 0, path: 0 }
    }
}

/// 走一步：段怎么关与 `operation_segments` 同一条规矩（屏障或 FUA、当前段有写、每块有写的盘都放行）；
/// 直接前驱怎么记与 `must_persist_before_masks` 同一条规矩（被自己盘上屏障钉住的写、FUA 写，都先于之后发出的每一次写）。
fn apply_step(context: &WalkContext, walk: &mut Walk, step: Step, masks: &mut [u32; MAXIMUM_STREAMING_WRITES]) {
    let mut may_close = false;
    match step {
        Step::Write { index, disk, fua } => {
            let bit = 1u32 << index;
            masks[usize::from(index)] = walk.armed | walk.forced;
            walk.key |= u128::from(walk.closed) << (5 * u32::from(index));
            walk.writes_in_current += 1;
            walk.issued |= bit;
            if fua {
                walk.unreleased &= !(1u8 << disk);
                walk.forced |= bit;
                may_close = true;
            } else {
                walk.unreleased |= 1u8 << disk;
            }
        }
        Step::Barrier { disk } => {
            walk.unreleased &= !(1u8 << disk);
            walk.armed |= walk.issued & context.writes_on_disk[usize::from(disk)];
            may_close = true;
        }
    }
    if may_close && walk.writes_in_current > 0 && walk.unreleased == 0 {
        walk.closed += 1;
        walk.writes_in_current = 0;
    }
}

pub(super) trait LeafSink {
    fn leaf(&mut self, walk: &Walk, masks: &[u32; MAXIMUM_STREAMING_WRITES]);
}

pub(super) fn walk_interleavings<Sink: LeafSink>(context: &WalkContext, walk: Walk, masks: &mut [u32; MAXIMUM_STREAMING_WRITES], sink: &mut Sink) {
    let (older_position, newer_position) = (usize::from(walk.older_position), usize::from(walk.newer_position));
    if older_position == context.older_steps.len() && newer_position == context.newer_steps.len() {
        sink.leaf(&walk, masks);
        return;
    }
    if older_position < context.older_steps.len() {
        let mut next = walk;
        apply_step(context, &mut next, context.older_steps[older_position], masks);
        next.older_position += 1;
        next.path <<= 1;
        walk_interleavings(context, next, masks, sink);
    }
    if newer_position < context.newer_steps.len() && context.newer_step_may_follow(older_position, newer_position) {
        let mut next = walk;
        apply_step(context, &mut next, context.newer_steps[newer_position], masks);
        next.newer_position += 1;
        next.path = (next.path << 1) | 1;
        walk_interleavings(context, next, masks, sink);
    }
}

#[derive(Clone, Copy)]
pub(super) struct CoverSearchTask {
    pub(super) walk: Walk,
    pub(super) masks: [u32; MAXIMUM_STREAMING_WRITES],
}

/// 把深搜的前 `steps_deep` 步展开成任务，线程按任务领活；任务按字典序排，结果与线程数无关。
pub(super) fn split_tasks(context: &WalkContext, steps_deep: usize) -> Vec<CoverSearchTask> {
    let mut frontier = vec![CoverSearchTask { walk: Walk::start(), masks: [0; MAXIMUM_STREAMING_WRITES] }];
    for _ in 0..steps_deep {
        let mut next_frontier = Vec::new();
        for task in &frontier {
            let (older_position, newer_position) = (usize::from(task.walk.older_position), usize::from(task.walk.newer_position));
            if older_position == context.older_steps.len() && newer_position == context.newer_steps.len() {
                next_frontier.push(*task);
                continue;
            }
            if older_position < context.older_steps.len() {
                let mut child = *task;
                apply_step(context, &mut child.walk, context.older_steps[older_position], &mut child.masks);
                child.walk.older_position += 1;
                child.walk.path <<= 1;
                next_frontier.push(child);
            }
            if newer_position < context.newer_steps.len() && context.newer_step_may_follow(older_position, newer_position) {
                let mut child = *task;
                apply_step(context, &mut child.walk, context.newer_steps[newer_position], &mut child.masks);
                child.walk.newer_position += 1;
                child.walk.path = (child.walk.path << 1) | 1;
                next_frontier.push(child);
            }
        }
        frontier = next_frontier;
    }
    frontier
}

#[derive(Default)]
struct PassOneSink {
    pub(super) leaves: u64,
    pub(super) signatures: FastMap<u128, (u64, u64)>,
    pub(super) masks: FastSet<[u32; MAXIMUM_STREAMING_WRITES]>,
}

impl LeafSink for PassOneSink {
    fn leaf(&mut self, walk: &Walk, masks: &[u32; MAXIMUM_STREAMING_WRITES]) {
        self.leaves += 1;
        let entry = self.signatures.entry(walk.key).or_insert((0, u64::MAX));
        entry.0 += 1;
        if walk.path < entry.1 {
            entry.1 = walk.path;
        }
        self.masks.insert(*masks);
    }
}

pub(super) struct PassOne {
    pub(super) leaves: u64,
    /// （段签名，这个签名的流条数，这个签名里字典序最小的那条流的路径）
    pub(super) signatures: Vec<(u128, u64, u64)>,
    pub(super) masks: Vec<[u32; MAXIMUM_STREAMING_WRITES]>,
}

pub(super) fn pass_one(context: &WalkContext, threads: usize) -> PassOne {
    let tasks = split_tasks(context, 12usize.min((context.older_steps.len() + context.newer_steps.len()) / 2));
    let next_task = AtomicUsize::new(0);
    let mut sinks: Vec<PassOneSink> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut sink = PassOneSink::default();
                    loop {
                        let position = next_task.fetch_add(1, AtomicOrdering::Relaxed);
                        if position >= tasks.len() {
                            break;
                        }
                        let mut masks = tasks[position].masks;
                        walk_interleavings(context, tasks[position].walk, &mut masks, &mut sink);
                    }
                    sink
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程不 panic")).collect()
    });
    let mut merged = sinks.pop().expect("至少一个线程");
    for other in sinks {
        merged.leaves += other.leaves;
        for (key, (count, representative)) in other.signatures {
            let entry = merged.signatures.entry(key).or_insert((0, u64::MAX));
            entry.0 += count;
            entry.1 = entry.1.min(representative);
        }
        for masks in other.masks {
            merged.masks.insert(masks);
        }
    }
    let mut signatures: Vec<(u128, u64, u64)> = merged.signatures.into_iter().map(|(key, (count, representative))| (key, count, representative)).collect();
    signatures.sort_unstable();
    let mut masks: Vec<[u32; MAXIMUM_STREAMING_WRITES]> = merged.masks.into_iter().collect();
    masks.sort_unstable();
    PassOne { leaves: merged.leaves, signatures, masks }
}

/// 段签名 → 各段的写集合（按段号排）。
pub(super) fn segments_of_signature(key: u128, write_count: usize) -> Vec<u32> {
    let mut by_index = [0u32; 32];
    let mut highest = 0usize;
    for write in 0..write_count {
        let segment = ((key >> (5 * write)) & 31) as usize;
        by_index[segment] |= 1u32 << write;
        highest = highest.max(segment);
    }
    by_index[..=highest].to_vec()
}

/// 一串段的枚举域：每个状态连同它的混合进制序号一起交给 `emit`。「前面各段全持久 + 当前段任意组合」，
/// 当前段的全零组合不发（它就是上一段全持久那个状态），全局的全零状态发一次；个数 = 1 + Σ(2^两态写数 · 3^原地覆写数 − 1)。
#[inline]
pub(super) fn for_each_crash_state(table_summary: &WriteTableSummary, segments: &[u32], mut emit: impl FnMut(u64, u64)) {
    emit(0, 0);
    let mut prefix_code = 0u64;
    let mut prefix_index = 0u64;
    let mut writes = [0usize; MAXIMUM_STREAMING_WRITES];
    for &segment in segments {
        let mut count = 0usize;
        let mut remaining = segment;
        while remaining != 0 {
            writes[count] = remaining.trailing_zeros() as usize;
            remaining &= remaining - 1;
            count += 1;
        }
        let mut digits = [0u8; MAXIMUM_STREAMING_WRITES];
        let mut code = prefix_code;
        let mut index = prefix_index;
        'combinations: loop {
            let mut position = 0usize;
            loop {
                if position == count {
                    break 'combinations;
                }
                let write = writes[position];
                if digits[position] + 1 < table_summary.radix[write] {
                    digits[position] += 1;
                    code += 1u64 << (2 * write);
                    index += table_summary.weight[write];
                    break;
                }
                code -= u64::from(digits[position]) << (2 * write);
                index -= u64::from(digits[position]) * table_summary.weight[write];
                digits[position] = 0;
                position += 1;
            }
            emit(code, index);
        }
        for &write in &writes[..count] {
            prefix_code |= 1u64 << (2 * write);
            prefix_index += table_summary.weight[write];
        }
    }
}

pub(super) fn expansion_of_segment(table_summary: &WriteTableSummary, segment: u32) -> u128 {
    (0..table_summary.write_count).filter(|write| segment & (1u32 << write) != 0).map(|write| u128::from(table_summary.radix[write])).product()
}

pub(super) fn closed_form_of_segments(table_summary: &WriteTableSummary, segments: &[u32]) -> u128 {
    1 + segments.iter().map(|segment| expansion_of_segment(table_summary, *segment) - 1).sum::<u128>()
}

/// 一个段签名上的读数。K1 在这个签名对应的每一条流上的读数都相同；`count` 是这个签名有几条流。
pub(super) struct SignatureRow {
    pub(super) key: u128,
    pub(super) count: u64,
    pub(super) representative_path: u64,
    pub(super) states: u32,
    pub(super) cross_states: u32,
    pub(super) invalid_record_states: u32,
    pub(super) invalid_all_writes_states: u32,
    pub(super) closed_form_matches: bool,
    pub(super) largest_segment_writes: u8,
    pub(super) largest_segment_expansion: u32,
}

fn evaluate_signatures(table_summary: &WriteTableSummary, signatures: &[(u128, u64, u64)], threads: usize) -> (Bitset, Vec<SignatureRow>) {
    const CHUNK: usize = 64;
    let next_start = AtomicUsize::new(0);
    let parts: Vec<(Bitset, Vec<SignatureRow>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut union = Bitset::new(table_summary.total_vectors);
                    let mut rows = Vec::new();
                    loop {
                        let start = next_start.fetch_add(CHUNK, AtomicOrdering::Relaxed);
                        if start >= signatures.len() {
                            break;
                        }
                        for &(key, count, representative_path) in &signatures[start..(start + CHUNK).min(signatures.len())] {
                            let segments = segments_of_signature(key, table_summary.write_count);
                            let (mut states, mut cross, mut invalid_record, mut invalid_all) = (0u32, 0u32, 0u32, 0u32);
                            for_each_crash_state(table_summary, &segments, |code, index| {
                                union.insert_index(index);
                                states += 1;
                                if table_summary.is_cross(code) {
                                    cross += 1;
                                }
                                if table_summary.is_invalid_by_record(code) {
                                    invalid_record += 1;
                                }
                                if table_summary.is_invalid_by_all_writes(code) {
                                    invalid_all += 1;
                                }
                            });
                            rows.push(SignatureRow {
                                key,
                                count,
                                representative_path,
                                states,
                                cross_states: cross,
                                invalid_record_states: invalid_record,
                                invalid_all_writes_states: invalid_all,
                                closed_form_matches: u128::from(states) == closed_form_of_segments(table_summary, &segments),
                                largest_segment_writes: u8_from_u32(segments.iter().map(|segment| segment.count_ones()).max().unwrap_or(0)),
                                largest_segment_expansion: u32::try_from(segments.iter().map(|segment| expansion_of_segment(table_summary, *segment)).max().unwrap_or(1))
                                    .expect("两盘几何一段的展开数装得进 u32"),
                            });
                        }
                    }
                    (union, rows)
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程不 panic")).collect()
    });
    let mut union = Bitset::new(table_summary.total_vectors);
    let mut rows = Vec::new();
    for (part_union, mut part_rows) in parts {
        union.or_with(&part_union);
        rows.append(&mut part_rows);
    }
    rows.sort_unstable_by_key(|row| row.key);
    (union, rows)
}

fn truth_list_of(table_summary: &WriteTableSummary, bits: &Bitset) -> StateList {
    let mut states: StateList = bits.indices().into_iter().map(|index| StateCode::from(table_summary.code_of_index(index))).collect();
    states.sort_unstable();
    states
}

// ── T_exact 的流式算法 ────────────────────────────────────────────────────────────────────

/// 与 `must_persist_before_masks` 末尾同一个传递闭包循环。
pub(super) fn close_masks(direct: &[u32; MAXIMUM_STREAMING_WRITES], write_count: usize) -> [u32; MAXIMUM_STREAMING_WRITES] {
    let mut masks = *direct;
    loop {
        let mut changed = false;
        for write in 0..write_count {
            let mut extended = masks[write];
            for earlier in 0..write_count {
                if masks[write] & (1u32 << earlier) != 0 {
                    extended |= masks[earlier];
                }
            }
            if extended != masks[write] {
                masks[write] = extended;
                changed = true;
            }
        }
        if !changed {
            return masks;
        }
    }
}

struct ExactWalk<'summary> {
    pub(super) table_summary: &'summary WriteTableSummary,
    pub(super) masks: [u32; MAXIMUM_STREAMING_WRITES],
    order: Vec<usize>,
    successors: Vec<u32>,
    pub(super) in_place: Vec<bool>,
}

impl ExactWalk<'_> {
    pub(super) fn extend(&self, position: usize, chosen: u32, index: u64, emit: &mut dyn FnMut(u64)) {
        if position == self.order.len() {
            emit(index);
            let eligible: Vec<usize> = (0..self.table_summary.write_count).filter(|&write| self.in_place[write] && chosen & (1u32 << write) != 0 && self.successors[write] & chosen == 0).collect();
            for subset in 1u32..(1u32 << eligible.len()) {
                let torn_index = eligible.iter().enumerate().filter(|(eligible_position, _)| subset & (1u32 << eligible_position) != 0).fold(index, |sum, (_, &write)| sum + self.table_summary.weight[write]);
                emit(torn_index);
            }
            return;
        }
        let write = self.order[position];
        self.extend(position + 1, chosen, index, emit);
        if self.masks[write] & !chosen == 0 {
            self.extend(position + 1, chosen | (1u32 << write), index + self.table_summary.weight[write], emit);
        }
    }
}

/// 一份偏序的全部可达状态（序号）：偏序的下闭集，撕裂态只在原地覆写是下闭集的极大元时取（与 `reachable_states_of_partial_order` 同义，只是按下闭集递归、不扫 2^n）。
pub(super) fn for_each_exact_state(table_summary: &WriteTableSummary, direct: &[u32; MAXIMUM_STREAMING_WRITES], emit: &mut dyn FnMut(u64)) {
    let masks = close_masks(direct, table_summary.write_count);
    let mut order: Vec<usize> = (0..table_summary.write_count).collect();
    order.sort_by_key(|&write| (masks[write].count_ones(), write));
    let successors: Vec<u32> = (0..table_summary.write_count).map(|write| (0..table_summary.write_count).filter(|&later| masks[later] & (1u32 << write) != 0).fold(0u32, |sum, later| sum | (1u32 << later))).collect();
    let in_place: Vec<bool> = table_summary.radix.iter().map(|write_radix| *write_radix == 3).collect();
    let walk = ExactWalk { table_summary, masks, order, successors, in_place };
    walk.extend(0, 0, 0, emit);
}

fn evaluate_exact_union(table_summary: &WriteTableSummary, direct_masks: &[[u32; MAXIMUM_STREAMING_WRITES]], threads: usize) -> Bitset {
    let next_position = AtomicUsize::new(0);
    let parts: Vec<Bitset> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut union = Bitset::new(table_summary.total_vectors);
                    loop {
                        let position = next_position.fetch_add(1, AtomicOrdering::Relaxed);
                        if position >= direct_masks.len() {
                            break;
                        }
                        for_each_exact_state(table_summary, &direct_masks[position], &mut |index| union.insert_index(index));
                    }
                    union
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程不 panic")).collect()
    });
    let mut union = Bitset::new(table_summary.total_vectors);
    for part in &parts {
        union.or_with(part);
    }
    union
}

// ── 算法 (b) 的定长数组版 ─────────────────────────────────────────────────────────────────

/// 与 `some_stream_realizes_target` 同一个搜索（状态 = 两条链各走到哪、未放行的盘、当前段的相、上一段的相），
/// 只是状态压成 16 位、已访问用世代戳的定长数组，不用哈希表。相：0 之前（必须全持久）、1 当前（任意）、2 之后（必须全没持久）。
pub(super) struct Searcher<'context> {
    context: &'context WalkContext,
    stamps: Vec<u32>,
    epoch: u32,
    stack: Vec<u16>,
    visited_states: u64,
}

impl<'context> Searcher<'context> {
    pub(super) fn new(context: &'context WalkContext) -> Searcher<'context> {
        Searcher { context, stamps: vec![0; 1 << 16], epoch: 0, stack: Vec::new(), visited_states: 0 }
    }

    /// 低六位：未放行盘（两位）、当前段的相加一（两位，0 = 没有开着的段）、上一段的相（两位）；高十位：两条链的位置各五位。
    fn successors(&self, packed: u16, step: Step, target: u64, out: &mut [u16; 3]) -> usize {
        let position_part = packed & !0x3F;
        let unreleased = (packed >> 4) & 3;
        let open = (packed >> 2) & 3;
        let last = packed & 3;
        let compose = |unreleased_part: u16, open_part: u16, last_part: u16| position_part | (unreleased_part << 4) | (open_part << 2) | last_part;
        match step {
            Step::Barrier { disk } => {
                let unreleased_after = unreleased & !(1u16 << disk);
                let (open_after, last_after) = if open != 0 && unreleased_after == 0 { (0, open - 1) } else { (open, last) };
                out[0] = compose(unreleased_after, open_after, last_after);
                1
            }
            Step::Write { index, disk, fua } => {
                let value = ((target >> (2 * u32::from(index))) & 3) as u8;
                let mut produced = 0usize;
                for role in 0u16..3 {
                    if open != 0 {
                        if role != open - 1 {
                            continue;
                        }
                    } else if !(role >= last && !(role == 1 && last == 1)) {
                        continue;
                    }
                    let allowed = match role {
                        0 => value == PERSISTED,
                        1 => true,
                        _ => value == NOT_PERSISTED,
                    };
                    if !allowed {
                        continue;
                    }
                    let unreleased_after = if fua { unreleased & !(1u16 << disk) } else { unreleased | (1u16 << disk) };
                    let (open_after, last_after) = if fua && unreleased_after == 0 { (0, role) } else { (role + 1, last) };
                    out[produced] = compose(unreleased_after, open_after, last_after);
                    produced += 1;
                }
                produced
            }
        }
    }

    fn realizable(&mut self, target: u64) -> bool {
        self.epoch += 1;
        let epoch = self.epoch;
        self.stack.clear();
        self.stamps[0] = epoch;
        self.stack.push(0);
        let (older_length, newer_length) = (self.context.older_steps.len(), self.context.newer_steps.len());
        let mut out = [0u16; 3];
        while let Some(packed) = self.stack.pop() {
            self.visited_states += 1;
            let older_position = usize::from(packed >> 11);
            let newer_position = usize::from((packed >> 6) & 31);
            if older_position == older_length && newer_position == newer_length {
                return true;
            }
            if older_position < older_length {
                let produced = self.successors(packed, self.context.older_steps[older_position], target, &mut out);
                for &state in &out[..produced] {
                    let next = state + (1 << 11);
                    if self.stamps[usize::from(next)] != epoch {
                        self.stamps[usize::from(next)] = epoch;
                        self.stack.push(next);
                    }
                }
            }
            if newer_position < newer_length && self.context.newer_step_may_follow(older_position, newer_position) {
                let produced = self.successors(packed, self.context.newer_steps[newer_position], target, &mut out);
                for &state in &out[..produced] {
                    let next = state + (1 << 6);
                    if self.stamps[usize::from(next)] != epoch {
                        self.stamps[usize::from(next)] = epoch;
                        self.stack.push(next);
                    }
                }
            }
        }
        false
    }
}

fn search_truth(context: &WalkContext, table_summary: &WriteTableSummary, threads: usize) -> Bitset {
    const CHUNK: u64 = 4096;
    let next_chunk = AtomicUsize::new(0);
    let parts: Vec<Bitset> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut found = Bitset::new(table_summary.total_vectors);
                    let mut searcher = Searcher::new(context);
                    loop {
                        let start = next_chunk.fetch_add(1, AtomicOrdering::Relaxed) as u64 * CHUNK;
                        if start >= table_summary.total_vectors {
                            break;
                        }
                        for index in start..(start + CHUNK).min(table_summary.total_vectors) {
                            if searcher.realizable(table_summary.code_of_index(index)) {
                                found.insert_index(index);
                            }
                        }
                    }
                    found
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程不 panic")).collect()
    });
    let mut found = Bitset::new(table_summary.total_vectors);
    for part in &parts {
        found.or_with(part);
    }
    found
}

/// 登记 S3 要报的「逐状态深搜的规模」：每隔 `stride` 个状态向量跑一次搜索，报跑了几个、共访问几个搜索状态、单个最多访问几个、几个搜得到。
pub(super) fn search_sample(context: &WalkContext, table_summary: &WriteTableSummary, stride: u64) -> (u64, u64, u64, u64) {
    let mut searcher = Searcher::new(context);
    let (mut vectors, mut visited_maximum, mut realized) = (0u64, 0u64, 0u64);
    let mut index = 0u64;
    while index < table_summary.total_vectors {
        let before = searcher.visited_states;
        if searcher.realizable(table_summary.code_of_index(index)) {
            realized += 1;
        }
        visited_maximum = visited_maximum.max(searcher.visited_states - before);
        vectors += 1;
        index += stride;
    }
    (vectors, searcher.visited_states, visited_maximum, realized)
}

pub(super) fn truth_by_search_fast(world: &World) -> StateList {
    let table_summary = WriteTableSummary::new(&world.table);
    let context = WalkContext::new(world, &table_summary);
    truth_list_of(&table_summary, &search_truth(&context, &table_summary, worker_thread_count()))
}

// ── 两条链的世界，不存流 ──────────────────────────────────────────────────────────────────

pub(super) fn build_two_chain_world_without_streams(candidate: Candidate, geometry: &Geometry) -> World {
    let mut table = Vec::new();
    let older_writes = push_transaction_writes(&mut table, geometry, Owner::OlderTransaction, geometry.root_disk_of_the_older_transaction);
    let newer_writes = push_transaction_writes(&mut table, geometry, Owner::NewerTransaction, geometry.root_disk_of_the_newer_transaction);
    let older_chain = single_transaction_chain(&older_writes, geometry.disk_count);
    let newer_chain = single_transaction_chain(&newer_writes, geometry.disk_count);
    let mut serial_stream = older_chain.clone();
    serial_stream.extend(newer_chain.iter().copied());
    let waits = match candidate {
        Candidate::SerialPipeline => vec![Some(older_chain.len() - 1); newer_chain.len()],
        Candidate::ManyPublishesInFlightFree => vec![None; newer_chain.len()],
        Candidate::ManyPublishesInFlightOrdered => ordering_waits(&table, &older_chain, &newer_chain),
        Candidate::OnePublishManyTransactions => unreachable!("一次发布装多个事务不走两条链的交错"),
    };
    World {
        candidate,
        geometry: geometry.clone(),
        table,
        streams: Vec::new(),
        search_chains: [older_chain.clone(), newer_chain.clone()],
        search_waits_of_second_chain: waits,
        arm_chains: [older_chain, newer_chain],
        arm_tail: Vec::new(),
        serial_stream: Some(serial_stream),
    }
}

/// 路径（每步一位，高位在前，0 = A、1 = B）→ 发出流的文字。
pub(super) fn render_path(world: &World, path: u64) -> String {
    render_stream(&world.table, &events_of_path(world, path))
}

pub(super) fn events_of_path(world: &World, path: u64) -> Vec<Event> {
    let [older_chain, newer_chain] = &world.search_chains;
    let total_steps = older_chain.len() + newer_chain.len();
    let (mut older_position, mut newer_position) = (0usize, 0usize);
    let mut events = Vec::new();
    for step in 0..total_steps {
        if (path >> (total_steps - 1 - step)) & 1 == 0 {
            events.push(older_chain[older_position]);
            older_position += 1;
        } else {
            events.push(newer_chain[newer_position]);
            newer_position += 1;
        }
    }
    events
}

pub(super) struct StreamingEvaluation {
    pub(super) label: &'static str,
    pub(super) sampling_point: &'static str,
    pub(super) world: World,
    pub(super) table_summary: WriteTableSummary,
    pub(super) leaves: u64,
    pub(super) expected_leaves: u128,
    pub(super) order_masks: usize,
    pub(super) rows: Vec<SignatureRow>,
    pub(super) truth: StateList,
    pub(super) truth_by_search: StateList,
    pub(super) truth_cross: usize,
    pub(super) truth_invalid_record: usize,
    pub(super) truth_invalid_all_writes: usize,
    pub(super) arm_by_transaction: StateList,
    pub(super) exact_reachable: StateList,
}

pub(super) fn evaluate_streaming_cell(specification: &CellSpecification, threads: usize) -> StreamingEvaluation {
    let world = build_two_chain_world_without_streams(specification.candidate, &specification.geometry);
    let table_summary = WriteTableSummary::new(&world.table);
    let context = WalkContext::new(&world, &table_summary);
    let expected_leaves = count_interleavings(&context);
    let pass = pass_one(&context, threads);
    let (union, rows) = evaluate_signatures(&table_summary, &pass.signatures, threads);
    let truth = truth_list_of(&table_summary, &union);
    let truth_by_search = truth_list_of(&table_summary, &search_truth(&context, &table_summary, threads));
    let exact_reachable = truth_list_of(&table_summary, &evaluate_exact_union(&table_summary, &pass.masks, threads));
    let arm_by_transaction = arm_by_transaction_states(&world);
    let code_of = |state: &StateCode| u64::try_from(*state).expect("两盘几何的状态编码装得进 u64");
    let truth_cross = truth.iter().filter(|state| table_summary.is_cross(code_of(state))).count();
    let truth_invalid_record = truth.iter().filter(|state| table_summary.is_invalid_by_record(code_of(state))).count();
    let truth_invalid_all_writes = truth.iter().filter(|state| table_summary.is_invalid_by_all_writes(code_of(state))).count();
    StreamingEvaluation {
        label: specification.label,
        sampling_point: specification.sampling_point,
        world,
        table_summary,
        leaves: pass.leaves,
        expected_leaves,
        order_masks: pass.masks.len(),
        rows,
        truth,
        truth_by_search,
        truth_cross,
        truth_invalid_record,
        truth_invalid_all_writes,
        arm_by_transaction,
        exact_reachable,
    }
}

// ── 第二段的单元格输出 ────────────────────────────────────────────────────────────────────

/// 第二段每格要留到末尾判决用的读数（`CellJudgement` 之外再加 L3a、L3b、L10 与 K1 的最大一段）。
pub(super) struct SecondJudgement {
    pub(super) base: CellJudgement,
    pub(super) l3a: usize,
    pub(super) l3b: usize,
    pub(super) exact_not_in_truth: usize,
    pub(super) truth_not_in_exact: usize,
    pub(super) best_stream_recorded_covered: usize,
    /// K2 在「按记录」那一档上覆盖到的不合法前缀数。
    pub(super) arm_recorded_covered: usize,
    pub(super) root_slot_cut_largest_segment_writes: usize,
    pub(super) root_slot_cut_largest_segment_expansion: u128,
}

/// L3b：|(T ∩ T_exact ∩ C) \ 臂|。
pub(super) fn true_cross_misses(table: &[WriteSpecification], truth: &[StateCode], exact: &[StateCode], arm: &[StateCode]) -> usize {
    let reachable_truth: Vec<StateCode> = truth.iter().copied().filter(|state| contains(exact, *state)).collect();
    count_cross_transaction_misses(table, &reachable_truth, arm)
}

pub(super) fn second_judgement_of_in_memory(evaluation: &CellEvaluation, base: CellJudgement) -> SecondJudgement {
    let table = &evaluation.world.table;
    SecondJudgement {
        l3a: base.arm_by_transaction_cross_misses,
        l3b: true_cross_misses(table, &evaluation.truth, &evaluation.exact_reachable, &evaluation.arm_by_transaction),
        exact_not_in_truth: difference_count(&evaluation.exact_reachable, &evaluation.truth),
        truth_not_in_exact: difference_count(&evaluation.truth, &evaluation.exact_reachable),
        best_stream_recorded_covered: evaluation.streams.iter().map(|metrics| metrics.recorded_prefixes_covered).max().unwrap_or(0),
        arm_recorded_covered: count_invalid_prefixes_covered(table, &evaluation.truth, &evaluation.arm_by_transaction, InvalidPrefixKind::ByRecord),
        root_slot_cut_largest_segment_writes: evaluation.streams.iter().map(|metrics| metrics.largest_segment_writes).max().unwrap_or(0),
        root_slot_cut_largest_segment_expansion: evaluation.streams.iter().map(|metrics| metrics.largest_segment_expansion).max().unwrap_or(1),
        base,
    }
}
