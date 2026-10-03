
fn xorshift(state: &mut u64) -> u64 { let mut x = *state; x ^= x << 13; x ^= x >> 7; x ^= x << 17; *state = x; x }

pub fn random_mode() {
    let mut state = 0x9E3779B97F4A7C15u64;
    let mut t = Tally::default();
    for _ in 0..40_000_000u64 {
        let c = 2 + xorshift(&mut state) % 5;
        CHUNK.store(c, Ordering::Relaxed);
        let s = xorshift(&mut state) % 7;
        let l = 1 + xorshift(&mut state) % 14;
        let e = s + l;
        // 构造：先切成合法块序的片，再随机打乱块内次序/随机删一片/随机插一片/随机复制一片
        let mut pieces: Vec<P> = Vec::new();
        let mut pos = s;
        while pos < e {
            let chunk_end = (s + ((pos - s) / c + 1) * c).min(e);
            let len = 1 + xorshift(&mut state) % (chunk_end - pos);
            pieces.push(P { start: pos, len, zero: true });
            pos += len;
        }
        // 打乱：随机交换两片
        let swaps = xorshift(&mut state) % 4;
        for _ in 0..swaps { let n = pieces.len() as u64; let i = (xorshift(&mut state) % n) as usize; let j = (xorshift(&mut state) % n) as usize; pieces.swap(i, j); }
        match xorshift(&mut state) % 6 {
            0 => { if pieces.len() > 1 { let i = (xorshift(&mut state) % pieces.len() as u64) as usize; pieces.remove(i); } }
            1 => { let i = (xorshift(&mut state) % pieces.len() as u64) as usize; let p = pieces[i]; pieces.insert(i, p); }
            2 => { let i = (xorshift(&mut state) % pieces.len() as u64) as usize; pieces[i].len += 1; }
            3 => { let i = (xorshift(&mut state) % pieces.len() as u64) as usize; if pieces[i].len > 1 { pieces[i].len -= 1; } }
            4 => { let i = (xorshift(&mut state) % pieces.len() as u64) as usize; pieces[i].zero = false; }
            _ => {}
        }
        compare(s, l, c, &pieces, &mut t);
    }
    println!("random: {t:#?}");
}
