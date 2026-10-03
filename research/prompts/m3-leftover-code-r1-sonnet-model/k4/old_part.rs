
// 旧规则（diff 里被删的行，按序、首尾相接）：单个声明段
fn old_tile(observed: &[DeviceEvent], index: usize, declared: &[(DeviceOffsetInBytes, u64)]) -> Option<(DeviceOffsetInBytes, u64, usize)> {
    let DeviceEvent::Write { offset: start, .. } = &observed[index] else { return None; };
    let (declared_offset, declared_length) = declared.iter().find(|(declared_offset, _)| declared_offset == start)?;
    let end = declared_offset.0 + declared_length;
    let mut cursor = declared_offset.0;
    let mut consumed = 0usize;
    while cursor < end {
        let Some(DeviceEvent::Write { offset, length, content_hash }) = observed.get(index + consumed) else { return None; };
        if offset.0 != cursor || *content_hash != fnv1a_64_of_zeros(*length) { return None; }
        cursor += *length;
        consumed += 1;
    }
    (cursor == end).then_some((*declared_offset, *declared_length, consumed))
}

pub fn old_subset_mode() {
    let mut total = 0u64; let mut old_some = 0u64; let mut not_subset = 0u64; let mut differs = 0u64;
    let mut first: Option<String> = None;
    for c in [3u64, 4] {
        CHUNK.store(c, Ordering::Relaxed);
        for s in [0u64, 1, 2, 5] {
            for l in 1..=8u64 {
                let e = s + l;
                let mut pool: Vec<P> = Vec::new();
                for st in s..e { for ln in 1..=(e - st) { pool.push(P { start: st, len: ln, zero: true }); } }
                pool.push(P { start: e - 1, len: 2, zero: true });
                pool.push(P { start: s, len: 1, zero: false });
                let max_len = if l <= 5 { 5 } else { 4 };
                fn rec(depth: usize, max_len: usize, pool: &Vec<P>, seq: &mut Vec<P>, s: u64, l: u64, tot: &mut (u64,u64,u64,u64), first: &mut Option<String>) {
                    if depth > 0 {
                        let observed: Vec<DeviceEvent> = seq.iter().map(ev).collect();
                        let declared = vec![(DeviceOffsetInBytes(s), l)];
                        let o = old_tile(&observed, 0, &declared).map(|(_,_,n)| n);
                        let n = tile_of_zeros_starting_at(&observed, 0, &declared).map(|(_,_,n)| n);
                        tot.0 += 1;
                        if o.is_some() { tot.1 += 1; if o != n { tot.2 += 1; if first.is_none() { *first = Some(format!("{seq:?} old={o:?} new={n:?}")); } } }
                        else if n.is_some() { tot.3 += 1; }
                    }
                    if depth == max_len { return; }
                    for p in pool { seq.push(*p); rec(depth + 1, max_len, pool, seq, s, l, tot, first); seq.pop(); }
                }
                let mut tot = (0u64, 0u64, 0u64, 0u64);
                let mut seq = Vec::new();
                rec(0, max_len, &pool, &mut seq, s, l, &mut tot, &mut first);
                total += tot.0; old_some += tot.1; not_subset += tot.2; differs += tot.3;
            }
        }
    }
    println!("old-vs-new: sequences={total} old_folds={old_some} old_fold_but_new_differs={not_subset} new_folds_where_old_did_not={differs} first_counterexample={first:?}");
}
