#!/usr/bin/env python3
"""m3-batch10z-no-code-r1 攻方：给一份 crates 副本打探针。用法：patch_probe.py <副本根> today|pre14。
today：冻结副本原样（第十四块合入之后）；pre14：冻结副本把第十四块 core 两份文件的改动倒推回去之后。
每一处替换都核命中次数，对不上就退出 3。"""
import sys
root, which = sys.argv[1], sys.argv[2]
m = root + '/crates/singlefs-core/src/mount.rs'
a = root + '/crates/singlefs-core/src/allocator.rs'

def rep(path, old, new, count=1):
    s = open(path).read()
    n = s.count(old)
    if n != count:
        print("命中次数不对", path, n, repr(old[:90])); sys.exit(3)
    open(path, 'w').write(s.replace(old, new))

# 分配器：几个只读探针
ALLOC = '''
    /// 攻方探针（只在草稿副本里）。
    pub fn opus_no_is_allocated(&self, device: DeviceIdentity, slot: u64) -> bool {
        let map = self.devices.iter().find(|m| m.device == device).expect("盘");
        map.allocated[map.index(SlotNumber(slot))]
    }
    pub fn opus_no_is_isolated(&self, device: DeviceIdentity, slot: u64) -> bool {
        let map = self.devices.iter().find(|m| m.device == device).expect("盘");
        map.isolated[map.index(SlotNumber(slot))]
    }
    pub fn opus_no_watch(&self, slot: u64) -> String {
        let (abandoned, c) = match self.root_ring.as_ref() {
            Some(o) => (o.placements_referenced_by_abandoned_roots(), o.placements_named_by_unapplied_journal_records()),
            None => (Vec::new(), Vec::new()),
        };
        let mut out = String::new();
        for map in &self.devices {
            let s = SlotNumber(slot);
            let i = map.index(s);
            let recs: Vec<String> = self.records.iter().filter(|r| r.device == map.device && r.slot.0 <= slot && slot < r.slot.0 + u64::from(r.span_slots)).map(|r| format!("rec@{}+{}:rel={}:gen={}", r.slot.0, r.span_slots, r.is_released, r.generation.0)).collect();
            out.push_str(&format!("dev{}:alloc={}:iso={}:held={}:in_ab={}:in_c={}:{} ", map.device.0, map.allocated[i], map.isolated[i], map.held_until_floor_takes_effect[i], abandoned.iter().any(|p| p.covers(map.device, s)), c.iter().any(|p| p.covers(map.device, s)), recs.join(",")));
        }
        if let Some(o) = self.root_ring.as_ref() {
            for (ring_slot, list) in &o.placements_named_by_unapplied_journal_records {
                for p in list.iter().filter(|p| p.slot.0 <= slot && slot < p.slot.0 + p.span) {
                    out.push_str(&format!("C[{ring_slot:?}]:{}+{}@dev{} ", p.slot.0, p.span, p.device.0));
                }
            }
        }
        out.push_str(&format!("c_total={} ab_total={}", c.len(), abandoned.len()));
        out
    }
    pub fn opus_no_dbl_slots(&self) -> String {
        let mut out = String::new();
        for map in &self.devices {
            let start = map.unit_area_start.slot().0;
            let list: Vec<String> = (0..map.allocated.len()).filter(|i| map.allocated[*i] && map.isolated[*i]).map(|i| (start + i as u64).to_string()).collect();
            out.push_str(&format!("dev{}:[{}] ", map.device.0, list.join(",")));
        }
        out
    }
    /// 隔离还护着的槽（环里被抛弃根引用的、还没解开的 C）里此刻空闲的、只扣住的；分配位与隔离位同时置着的，按归属拆
    /// （落在被抛弃根引用里的、只落在 C 里的、两样都不罩的）。
    pub fn opus_no_probe(&self) -> [u64; 6] {
        let (abandoned, c) = match self.root_ring.as_ref() {
            Some(o) => (o.placements_referenced_by_abandoned_roots(), o.placements_named_by_unapplied_journal_records()),
            None => (Vec::new(), Vec::new()),
        };
        let mut out = [0u64; 6];
        for map in &self.devices {
            let start = map.unit_area_start.slot().0;
            for index in 0..map.allocated.len() {
                let slot = SlotNumber(start + index as u64);
                let in_ab = abandoned.iter().any(|p| p.covers(map.device, slot));
                let in_c = c.iter().any(|p| p.covers(map.device, slot));
                let protected = in_ab || in_c;
                let blocked_other = map.allocated[index] || map.isolated[index];
                if protected && !blocked_other && !map.held_until_floor_takes_effect[index] { out[0] += 1; }
                if protected && !blocked_other && map.held_until_floor_takes_effect[index] { out[1] += 1; }
                if map.allocated[index] && map.isolated[index] {
                    out[2] += 1;
                    if in_ab { out[3] += 1; } else if in_c { out[4] += 1; } else { out[5] += 1; }
                }
            }
        }
        out
    }

'''
anchor = '    /// 这会儿装着的根环；`None` 见 `root_ring` 字段的注释。\n'
rep(a, anchor, ALLOC + anchor)

# 挂载与抬 F 的影子账那一圈：按起点槽豁免而跨度更宽、同一个起点后来的跨度更宽，计数
rep(m, "fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(\n",
"pub static OPUS_NO_C_EXEMPT_PARTIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);\n"
"pub static OPUS_NO_C_EXEMPT_PARTIAL_UNPROTECTED_SLOTS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);\n"
"pub static OPUS_NO_DUP_KEY_WIDER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);\n"
"pub static OPUS_NO_AB_EXEMPT_PARTIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);\n"
"fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(\n")
rep(m, """    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut unreadable = 0;""", """    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut opus_spans: BTreeMap<(u32, u64), u64> = BTreeMap::new();
    let mut unreadable = 0;""")
rep(m, """            let key = (device.0, slot.0);
            if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
                continue;
            }
            allocator.isolate_abandoned(device, slot, span_slots);""", """            let key = (device.0, slot.0);
            if referenced_by_candidates.contains(&key) {
                let live = |s: u64| current_records.iter().any(|r| !r.is_released && r.device == device && r.slot.0 <= s && s < r.slot.0 + u64::from(r.span_slots));
                if !(slot.0..slot.0 + span_slots).all(live) && !(slot.0..slot.0 + span_slots).all(|s| allocator.opus_no_is_allocated(device, s)) {
                    OPUS_NO_AB_EXEMPT_PARTIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
            if !referenced_by_candidates.contains(&key) && isolated.contains(&key) && opus_spans.get(&key).is_some_and(|w| *w < span_slots) {
                OPUS_NO_DUP_KEY_WIDER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if std::env::var("S3_DEBUG_DUP").is_ok() { eprintln!("DUPDBG abandoned-loop root=({:?},{:?}) dev={} slot={} earlier_span={:?} span={}", root.instance, root.checkpoint_txg, device.0, slot.0, opus_spans.get(&key), span_slots); }
            }
            if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
                continue;
            }
            opus_spans.insert(key, span_slots);
            allocator.isolate_abandoned(device, slot, span_slots);""")
rep(m, """        if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
            continue;
        }
        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);
    }
    // C 那一份""", """        if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
            continue;
        }
        opus_spans.insert(key, placement.span);
        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);
    }
    // C 那一份""")
C_HEAD = """    for placement in placements_named_by_unapplied_journal_records {
        let key = (placement.device.0, placement.slot.0);
        if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {
            continue;
        }
"""
C_PROBE = """    for placement in placements_named_by_unapplied_journal_records {
        let key = (placement.device.0, placement.slot.0);
        if referenced_by_the_current_account.contains(&key) {
            let live = |s: u64| current_records.iter().any(|r| !r.is_released && r.device == placement.device && r.slot.0 <= s && s < r.slot.0 + u64::from(r.span_slots));
            let uncovered: Vec<u64> = (placement.slot.0..placement.slot.0 + placement.span).filter(|s| !live(*s)).collect();
            if !uncovered.is_empty() {
                OPUS_NO_C_EXEMPT_PARTIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let unprotected = uncovered.iter().filter(|s| !allocator.opus_no_is_allocated(placement.device, **s) && !allocator.opus_no_is_isolated(placement.device, **s)).count();
                OPUS_NO_C_EXEMPT_PARTIAL_UNPROTECTED_SLOTS.fetch_add(unprotected as u64, std::sync::atomic::Ordering::Relaxed);
            }
        }
        if !referenced_by_the_current_account.contains(&key) && isolated.contains(&key) && opus_spans.get(&key).is_some_and(|w| *w < placement.span) {
            OPUS_NO_DUP_KEY_WIDER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if std::env::var("S3_DEBUG_DUP").is_ok() { eprintln!("DUPDBG c-loop dev={} slot={} earlier_span={:?} span={}", placement.device.0, placement.slot.0, opus_spans.get(&key), placement.span); }
        }
        if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {
            continue;
        }
        opus_spans.insert(key, placement.span);
"""
rep(m, C_HEAD, C_PROBE)

if which == 'today':
    # 四条臂（只看 OPUS_NO_ARM）：抬 F 那一串回收之后不补隔离（卸载那一支 / 准入那一支 / 两支），C 那一圈按起点槽一槽判整段
    rep(m, """    match shadow_ledger {
        ShadowLedger::On => allocator
            .reclaim_released_up_to_isolating_what_the_isolation_still_protects(floor, reuse),
        ShadowLedger::Off => allocator.reclaim_released_up_to(floor, reuse),
    }""", """    let arm = std::env::var("OPUS_NO_ARM").unwrap_or_default();
    match shadow_ledger {
        ShadowLedger::On if arm == "unmount_plain" && reuse == ReclaimedReuse::Immediately => allocator.reclaim_released_up_to(floor, reuse),
        ShadowLedger::On if arm == "held_plain" && reuse == ReclaimedReuse::HeldUntilFloorTakesEffect => allocator.reclaim_released_up_to(floor, reuse),
        ShadowLedger::On if arm == "raise_plain" => allocator.reclaim_released_up_to(floor, reuse),
        ShadowLedger::On => allocator
            .reclaim_released_up_to_isolating_what_the_isolation_still_protects(floor, reuse),
        ShadowLedger::Off => allocator.reclaim_released_up_to(floor, reuse),
    }""")
    rep(a, """        for index in start..end {
            if !self.allocated[index] {
                self.isolate_slot_at_index(index);
            }
        }""", """        let arm = std::env::var("OPUS_NO_ARM").unwrap_or_default();
        let first_only = arm == "first_slot_only";
        let isolate_all = arm.split('+').any(|a| a == "isoall");
        let first_allocated = self.allocated[start];
        for index in start..end {
            if isolate_all || if first_only { !first_allocated } else { !self.allocated[index] } {
                self.isolate_slot_at_index(index);
            }
        }""")
    rep(a, """    pub fn slots_counted_both_as_allocated_and_as_isolated(&self) -> u64 {
        u64::try_from(""", """    pub fn slots_counted_both_as_allocated_and_as_isolated(&self) -> u64 {
        if std::env::var("OPUS_NO_ARM").unwrap_or_default().split('+').any(|a| a == "acczero") {
            return 0;
        }
        u64::try_from(""")
print("打上了：", which)
