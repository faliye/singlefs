// 攻方腿（m4-close-code-r1）的差分探针：同一份 suite.rs 编进两个模块，一边连改前的 core（HEAD 那一份，包名改成
// singlefs-core-old），一边连改后的 core（冻结副本），同一份输入喂两边，逐例比 Debug 串。
// 用法：attack-codec-differential [每格例数，默认 3000]；末行 `name=codec_differential cases=… mismatches=…`。
mod old_side {
    use singlefs_core_old as c;
    include!("suite.rs");
    include!("trees.rs");
}
mod new_side {
    use singlefs_core as c;
    include!("suite.rs");
    include!("trees.rs");
}
use old_side::Rng;

fn unit_header_end(case: &str, bytes: &[u8]) -> usize {
    match case { "unit_packed" => 107, "unit_data" => 105, _ => 86 + 2 * usize::from(*bytes.get(51).unwrap_or(&0)) }
}

/// 解码格的输入：一部分是改前那一边的写者写出的合法单元或记录再改几个字节、重封校验和，一部分是随机字节。
fn input_for(case: &str, r: &mut Rng, widths: &[usize]) -> (Vec<u8>, u64) {
    let extra = r.next();
    let checksummed = matches!(case, "unit_index" | "unit_packed" | "unit_data" | "unit_declared_length" | "unit_payload" | "journal" | "root_slot" | "back_chain"
        | "logged_entry" | "tree_table" | "tree_table_mark" | "root_pointer_bytes");
    if checksummed && r.below(4) != 0 {
        let writer_case = match case { "unit_index" => "w_index", "unit_packed" => "w_packed", "journal" | "back_chain" => "w_journal", "root_slot" => "w_root",
            "unit_data" | "unit_payload" => "w_data", "logged_entry" => "w_logged_entry", "tree_table" | "tree_table_mark" | "root_pointer_bytes" => "w_tree_table",
            _ => ["w_index", "w_packed", "w_data"][r.below(3) as usize] };
        if let Some(mut bytes) = old_side::valid_bytes(writer_case, r.next()) {
            let flips = r.below(4);
            let header_end = match writer_case { "w_journal" => 4096, "w_root" | "w_logged_entry" | "w_tree_table" => bytes.len(), _ => unit_header_end(case, &bytes).min(bytes.len()) };
            for _ in 0..flips {
                let at = if r.below(4) == 0 { r.below(bytes.len() as u64) } else { r.below(header_end.max(1) as u64) } as usize;
                bytes[at] = if r.below(2) == 0 { r.next() as u8 } else { bytes[at] ^ (1 << r.below(8)) };
            }
            if flips > 0 && r.below(4) != 0 && !matches!(writer_case, "w_logged_entry" | "w_tree_table") { old_side::reseal(writer_case, &mut bytes, header_end); }
            if r.below(10) == 0 { let cut = r.below(bytes.len() as u64 + 1) as usize; bytes.truncate(cut); }
            return (bytes, extra);
        }
    }
    let len = if !widths.is_empty() && r.below(5) < 3 {
        let w = widths[r.below(widths.len() as u64) as usize];
        if r.below(6) == 0 { (w + r.below(3) as usize).saturating_sub(1) } else { w }
    } else { r.below(if checksummed { 33000 } else { 300 }) as usize };
    (r.bytes(len), extra)
}

/// 部分写切分的性质格（只对改后的一份）：按字节的理想读法逐单元算出该重写哪几片、每片怎么装、这次的字节落在哪，与
/// `PartialWriteOfAFile` 逐项比。P 是数据单元净荷 32634。
fn partial_write_oracle(per_case: u64, report: &mut dyn FnMut(&str, String, &str, &str)) -> u64 {
    use singlefs_core::write_request_split::{DataUnitOfAPartialWrite as K, PartialWriteOfAFile};
    let p: u64 = 32634;
    let mut r = Rng(0x3141_5926);
    let mut cases = 0u64;
    for _ in 0..per_case * 4 {
        let size_before = match r.below(5) { 0 => 0, 1 => r.below(4) * p, 2 => r.below(4) * p + r.below(3), 3 => r.below(5 * p), _ => (r.below(4) * p).saturating_sub(r.below(2) * r.below(3)) };
        let full_units = size_before.div_ceil(p);
        // 洞只在尾部：单元数不多于 ceil(size / P)
        let units_before = if r.below(4) == 0 { r.below(full_units + 1) } else { full_units };
        let offset = match r.below(5) { 0 => r.below(6 * p), 1 => r.below(6) * p, 2 => (r.below(6) * p).saturating_sub(1), 3 => size_before + r.below(3), _ => size_before.saturating_sub(r.below(3)) };
        let len = match r.below(4) { 0 => 1 + r.below(3), 1 => p - r.below(2), 2 => p * (1 + r.below(3)) + r.below(3) - 1, _ => 1 + r.below(3 * p) };
        let Some(len_nz) = std::num::NonZeroU64::new(len) else { continue };
        let Some(w) = PartialWriteOfAFile::new(units_before, size_before, singlefs_core::address::FileOffsetInBytes(offset), len_nz) else { continue };
        cases += 1;
        let end = offset + len;
        let size_after = size_before.max(end);
        let first_written = offset / p;
        let last_written = (end - 1) / p;
        let units_after = units_before.max(last_written + 1);
        let tail_partial = first_written >= units_before && units_before > 0 && size_before.saturating_sub((units_before - 1) * p).min(p) < p;
        let first_rewritten = if first_written < units_before { first_written } else if tail_partial { units_before - 1 } else { units_before };
        let mut want = format!("size_after={size_after} units_after={units_after} first_written={first_written} last_written={last_written} rewritten={first_rewritten}..{units_after}");
        let mut got = format!("size_after={} units_after={} first_written={} last_written={} rewritten={:?}", w.size_after_in_bytes(), w.data_units_after(), w.first_written_unit().0, w.last_written_unit().0, w.rewritten_units());
        for unit in first_rewritten.min(w.first_rewritten_unit().0)..units_after.max(w.data_units_after()) {
            let (us, ue) = (unit * p, unit * p + p);
            let old = unit < units_before;
            let lo = offset.max(us);
            let hi = end.min(ue);
            let overlap = (lo < hi).then(|| (lo - us, lo - offset, hi - lo));
            let covered = offset <= us && ue <= end;
            let kind = if old { if covered { "OldUnitCoveredWhole" } else { "OldUnitReadBack" } } else if covered { "NewUnitCoveredWhole" } else if overlap.is_some() { "NewUnitReadingTheResidueOfItsSlot" } else { "GapFilledWithZeros" };
            let declared = size_after.saturating_sub(us).min(p);
            want.push_str(&format!(" | u{unit}:{kind} decl={declared} bytes={overlap:?}"));
            let got_kind = std::panic::catch_unwind(|| match w.data_unit(singlefs_core::address::DataUnitIndexInFile(unit)) {
                K::OldUnitReadBack => "OldUnitReadBack", K::OldUnitCoveredWhole => "OldUnitCoveredWhole", K::NewUnitReadingTheResidueOfItsSlot => "NewUnitReadingTheResidueOfItsSlot",
                K::NewUnitCoveredWhole => "NewUnitCoveredWhole", K::GapFilledWithZeros => "GapFilledWithZeros" }).unwrap_or("PANIC");
            let got_bytes = w.bytes_written_into(singlefs_core::address::DataUnitIndexInFile(unit)).map(|b| (b.offset_in_the_payload, b.offset_in_the_bytes_written, b.length_in_bytes));
            got.push_str(&format!(" | u{unit}:{got_kind} decl={} bytes={got_bytes:?}", w.declared_length_after_the_write(singlefs_core::address::DataUnitIndexInFile(unit))));
        }
        if want != got { report("partial_write_oracle", format!("units_before={units_before} size_before={size_before} offset={offset} len={len}"), &want, &got); }
    }
    cases
}

/// 两边都崩时只比「崩了」，不比 panic 的那句话（改写换了断言的措辞、slice 越界换成下标越界，不算行为不同）；那句话另记一笔数。
fn without_panic_messages(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut inside = false;
    for ch in text.chars() {
        match ch { '⟦' => inside = true, '⟧' => inside = false, _ if !inside => out.push(ch), _ => {} }
    }
    out
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let per_case: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3000);
    let mut total = 0u64;
    let mut mismatches = 0u64;
    let mut shown: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut message_only = std::collections::BTreeMap::<String, u64>::new();
    let mut report = |name: &str, detail: String, o: &str, n: &str, mismatches: &mut u64| {
        if without_panic_messages(o) == without_panic_messages(n) { *message_only.entry(name.to_string()).or_insert(0) += 1; return; }
        *mismatches += 1;
        let c = shown.entry(name.to_string()).or_insert(0);
        *c += 1;
        if *c <= 4 {
            println!("MISMATCH case={name} {detail}\n  old={}\n  new={}", o.chars().take(700).collect::<String>(), n.chars().take(700).collect::<String>());
        }
    };
    for &case in old_side::PARSE_CASES {
        let mut r = Rng(0x9e37_79b9_7f4a_7c15 ^ old_side::fnv(case.as_bytes()));
        // 记录宽从改前那一边的读者认得的宽里找：稀疏随机字节在 0..300 的每个宽上各试 48 次，收下过的就是合法宽
        let mut widths = Vec::new();
        for len in 0..300usize {
            for _ in 0..48 { let b = r.bytes(len); let o = old_side::parse_case(case, &b, 0); if o.starts_with("Some") || o.starts_with("Ok") || o.starts_with("(Some") || o.starts_with("(Ok") { widths.push(len); break; } }
        }
        let mut accepted = (0u64, 0u64);
        for i in 0..per_case {
            let (input, extra) = input_for(case, &mut r, &widths);
            let o = old_side::parse_case(case, &input, extra);
            let n = new_side::parse_case(case, &input, extra);
            total += 1;
            if o.contains("Some(") || o.contains("Ok(") { accepted.0 += 1; }
            if o.contains("PANIC") { accepted.1 += 1; }
            if o != n {
                let hex: String = input.iter().take(160).map(|b| format!("{b:02x}")).collect();
                report(case, format!("i={i} len={} extra={extra:#x} input[..160]={hex}", input.len()), &o, &n, &mut mismatches);
            }
        }
        println!("case={case} widths={:?} examples={per_case} old_side_some_or_ok={} old_side_panics={}", &widths[..widths.len().min(8)], accepted.0, accepted.1);
    }
    for &case in old_side::WRITE_CASES {
        let mut panics = 0u64;
        // 往返：改后那一边写者没崩、写出来的它自己的读者却不收（或收回来的与写进去的不同）
        let mut new_round_trip_breaks = 0u64;
        let mut new_round_trip_breaks_at_ordinal_zero = 0u64;
        for i in 0..per_case {
            let seed = old_side::fnv(case.as_bytes()) ^ (i.wrapping_mul(0x9e37_79b9_7f4a_7c15));
            let o = old_side::write_case(case, seed);
            let n = new_side::write_case(case, seed);
            total += 1;
            if o.contains("PANIC") { panics += 1; }
            let breaks = n.split(" | ").any(|part| !part.contains("PANIC") && (part.ends_with(", false, true)") || part.ends_with(", false, false)") || part.contains("\", Err(") || part.contains("\", None)")));
            if breaks {
                new_round_trip_breaks += 1;
                if n.ends_with("| ordinal=0") { new_round_trip_breaks_at_ordinal_zero += 1; }
                if new_round_trip_breaks <= 3 { println!("ROUND_TRIP case={case} i={i} seed={seed:#x} new={}", n.chars().take(600).collect::<String>()); }
            }
            if o != n { report(case, format!("i={i} seed={seed:#x}"), &o, &n, &mut mismatches); }
        }
        println!("case={case} examples={per_case} old_side_panics={panics} new_side_round_trip_breaks={new_round_trip_breaks} of_which_at_journal_ordinal_zero={new_round_trip_breaks_at_ordinal_zero}");
    }
    for i in 0..per_case {
        let seed = 0x5151 ^ i.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        let o = old_side::deleted_inodes_case(seed);
        let n = new_side::deleted_inodes_case(seed);
        total += 1;
        if o != n { report("deleted_inodes", format!("i={i} seed={seed:#x}"), &o, &n, &mut mismatches); }
    }
    for (name, which) in [("tree_shape", 0u8), ("tree_append", 1), ("inode_tree", 2), ("tree_capacities", 3)] {
        let mut panics = 0u64;
        let mut reached = 0u64;
        // inode 树一例里有几百次写与删，例数取六分之一
        let examples = if which == 2 { per_case / 6 } else { per_case };
        for i in 0..examples {
            let seed = old_side::fnv(name.as_bytes()) ^ i.wrapping_mul(0x9e37_79b9_7f4a_7c15);
            let (o, n) = match which {
                0 => (old_side::shape_case(seed), new_side::shape_case(seed)),
                1 => (old_side::append_case(seed), new_side::append_case(seed)),
                2 => (old_side::inode_tree_case(seed), new_side::inode_tree_case(seed)),
                _ => (old_side::capacity_case(seed), new_side::capacity_case(seed)),
            };
            total += 1;
            if o.contains("PANIC") { panics += 1; }
            if o.contains("RetiredInodeLeafContainer {") || o.contains("grew=true") { reached += 1; }
            if o != n { report(name, format!("i={i} seed={seed:#x}"), &o, &n, &mut mismatches); }
        }
        println!("case={name} examples={examples} old_side_examples_with_a_panic={panics} old_side_examples_with_a_merge_or_a_new_level={reached}");
    }
    // 根记录的钉：改前收 Option<CheckpointTxg>，改后收 Option<u64>（签名变了，不进共用的一份）
    let mut r = Rng(0x77);
    for i in 0..(per_case + 300) {
        let v = if i < 300 { i } else { r.edgy() };
        let o = old_side::guard(|| singlefs_core_old::root_record::PinnedRingSlot::of_the_lowest_pin(Some(singlefs_core_old::address::CheckpointTxg(v))), |x| format!("{:?}", x));
        let n = new_side::guard(|| singlefs_core::root_record::PinnedRingSlot::of_the_lowest_pin(Some(v)), |x| format!("{:?}", x));
        total += 1;
        if o != n { report("pinned_ring_slot", format!("v={v}"), &o, &n, &mut mismatches); }
    }
    let mut oracle_mismatches = 0u64;
    let oracle_cases = partial_write_oracle(per_case, &mut |name: &str, detail: String, o: &str, n: &str| report(name, detail, o, n, &mut oracle_mismatches));
    println!("case=partial_write_oracle examples={oracle_cases} mismatches={oracle_mismatches}");
    total += oracle_cases;
    mismatches += oracle_mismatches;
    println!("per_case_mismatches={:?}", shown);
    println!("panic_message_only_differences={:?}", message_only);
    println!("name=codec_differential cases={total} mismatches={mismatches}");
}
