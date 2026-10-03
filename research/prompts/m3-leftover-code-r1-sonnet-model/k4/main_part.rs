
// ---------------- 对拍 ----------------
#[derive(Clone, Copy, Debug)]
struct P { start: u64, len: u64, zero: bool }

fn ev(p: &P) -> DeviceEvent {
    DeviceEvent::Write { offset: DeviceOffsetInBytes(p.start), length: p.len,
        content_hash: if p.zero { fnv1a_64_of_zeros(p.len) } else { fnv1a_64_of_zeros(p.len) + 1 } }
}

/// 严格口径：不跨块；前 N 条两两不重叠、全在段内、字节数恰好 = L、块与块之间按序（前面每条碰到的最大块号 <= 后面每条碰到的最小块号）。
fn oracle_strict(s: u64, l: u64, c: u64, seq: &[P]) -> Option<usize> {
    let e = s + l;
    let mut sum = 0u64;
    let mut n = 0usize;
    for p in seq {
        if !p.zero || p.len == 0 { return None; }
        if p.start < s || p.start + p.len > e { return None; }
        sum += p.len; n += 1;
        if sum >= l { break; }
    }
    if sum != l { return None; }
    let pre = &seq[..n];
    for i in 0..n { for j in i + 1..n {
        let (a, b) = (&pre[i], &pre[j]);
        if a.start < b.start + b.len && b.start < a.start + a.len { return None; }
    } }
    let chunk = |x: u64| (x - s) / c;
    for i in 0..n { for j in i + 1..n {
        let max_i = chunk(pre[i].start + pre[i].len - 1);
        let min_j = chunk(pre[j].start);
        if max_i > min_j { return None; }
    } }
    Some(n)
}

fn spans(s: u64, c: u64, seq: &[P], n: usize) -> bool {
    seq[..n.min(seq.len())].iter().any(|p| p.len > 0 && (p.start - s.min(p.start)) / c != (p.start + p.len - 1 - s.min(p.start)) / c && p.start >= s)
}

fn run_impl(s: u64, l: u64, seq: &[P]) -> Option<usize> {
    let observed: Vec<DeviceEvent> = seq.iter().map(ev).collect();
    let declared = vec![(DeviceOffsetInBytes(s), l)];
    tile_of_zeros_starting_at(&observed, 0, &declared).map(|(_, _, n)| n)
}

#[derive(Default, Debug)]
struct Tally { total: u64, agree: u64, impl_only_nospan: u64, impl_only_span: u64, oracle_only: u64, wrong_n: u64, first_unsound: Option<String>, first_incomplete: Option<String> }

fn compare(s: u64, l: u64, c: u64, seq: &[P], t: &mut Tally) {
    t.total += 1;
    let a = run_impl(s, l, seq);
    let b = oracle_strict(s, l, c, seq);
    match (a, b) {
        (None, None) => t.agree += 1,
        (Some(x), Some(y)) if x == y => t.agree += 1,
        (Some(_), Some(_)) => { t.wrong_n += 1; }
        (Some(n), None) => {
            if spans(s, c, seq, n) { t.impl_only_span += 1; } else {
                t.impl_only_nospan += 1;
                if t.first_unsound.is_none() { t.first_unsound = Some(format!("S={s} L={l} C={c} seq={seq:?} impl={a:?}")); }
            }
        }
        (None, Some(_)) => { t.oracle_only += 1; if t.first_incomplete.is_none() { t.first_incomplete = Some(format!("S={s} L={l} C={c} seq={seq:?}")); } }
    }
}

fn enumerate(s: u64, l: u64, c: u64, max_len: usize, t: &mut Tally) {
    let e = s + l;
    let mut pool: Vec<P> = Vec::new();
    for st in s..e { for ln in 1..=(e - st) { pool.push(P { start: st, len: ln, zero: true }); } }
    // 越界与非全 0 的各放几个
    pool.push(P { start: s.saturating_sub(1), len: 1, zero: true });
    pool.push(P { start: e, len: 1, zero: true });
    pool.push(P { start: e - 1, len: 2, zero: true });
    pool.push(P { start: s, len: 1, zero: false });
    let mut idx = vec![0usize; max_len];
    fn rec(depth: usize, max_len: usize, pool: &Vec<P>, seq: &mut Vec<P>, s: u64, l: u64, c: u64, t: &mut Tally) {
        if depth > 0 { compare(s, l, c, seq, t); }
        if depth == max_len { return; }
        for p in pool { seq.push(*p); rec(depth + 1, max_len, pool, seq, s, l, c, t); seq.pop(); }
    }
    let _ = &mut idx;
    let mut seq = Vec::new();
    rec(0, max_len, &pool, &mut seq, s, l, c, t);
}

fn main() {
    let mut grand = Tally::default();
    for c in [3u64, 4] {
        CHUNK.store(c, Ordering::Relaxed);
        for s in [0u64, 1, 2, 5] {
            for l in 1..=8u64 {
                let max_len = if l <= 5 { 5 } else { 4 };
                let mut t = Tally::default();
                enumerate(s, l, c, max_len, &mut t);
                grand.total += t.total; grand.agree += t.agree; grand.impl_only_nospan += t.impl_only_nospan;
                grand.impl_only_span += t.impl_only_span; grand.oracle_only += t.oracle_only; grand.wrong_n += t.wrong_n;
                if grand.first_unsound.is_none() { grand.first_unsound = t.first_unsound.clone(); }
                if grand.first_incomplete.is_none() { grand.first_incomplete = t.first_incomplete.clone(); }
            }
        }
    }
    println!("{grand:#?}");
    // 零长度的清零段不终止吗
    CHUNK.store(4, Ordering::Relaxed);
    let observed = vec![DeviceEvent::Write { offset: DeviceOffsetInBytes(8), length: 0, content_hash: fnv1a_64_of_zeros(0) }];
    let folded = fold_declared_zero_fills(&observed, &[(DeviceOffsetInBytes(8), 0)]);
    println!("length0 declared: folded={folded:?}");
    // 日志里长度 0 的写当作清零段的一部分
    let observed = vec![
        DeviceEvent::Write { offset: DeviceOffsetInBytes(0), length: 0, content_hash: fnv1a_64_of_zeros(0) },
        DeviceEvent::Write { offset: DeviceOffsetInBytes(0), length: 4, content_hash: fnv1a_64_of_zeros(4) },
    ];
    println!("zero-length write at head: {:?}", fold_declared_zero_fills(&observed, &[(DeviceOffsetInBytes(0), 4)]));
    let observed = vec![
        DeviceEvent::Write { offset: DeviceOffsetInBytes(2), length: 0, content_hash: fnv1a_64_of_zeros(0) },
        DeviceEvent::Write { offset: DeviceOffsetInBytes(0), length: 2, content_hash: fnv1a_64_of_zeros(2) },
        DeviceEvent::Write { offset: DeviceOffsetInBytes(2), length: 2, content_hash: fnv1a_64_of_zeros(2) },
    ];
    println!("zero-length ahead write: {:?}", fold_declared_zero_fills(&observed, &[(DeviceOffsetInBytes(0), 4)]));
}
