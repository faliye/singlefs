// 正推腿自己的探针：按偏移的部分写切分，对暴力参照逐字节比（不与实现共用切分代码）。
use singlefs_core::address::{DataUnitIndexInFile, FileOffsetInBytes};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::write_request_split::{DataUnitOfAPartialWrite, PartialWriteOfAFile};
use std::num::NonZeroU64;

fn content_of(size: u64) -> Vec<u8> {
    (0..size).map(|i| (i % 251) as u8 + 1).collect()
}

#[test]
fn partial_write_split_matches_a_brute_force_model() {
    let p = data_unit_payload_capacity() as u64;
    let sizes = [0, 1, p - 1, p, p + 1, 2 * p - 1, 2 * p, 2 * p + 1, 3 * p + 5];
    let offsets_rel = |s: u64| -> Vec<u64> {
        let mut v = vec![0, 1, p - 1, p, p + 1, 2 * p - 1, 2 * p, 2 * p + 1, 3 * p, 4 * p - 1, 4 * p + 3];
        v.push(s);
        if s > 0 { v.push(s - 1); }
        v.push(s + 1);
        v.sort(); v.dedup(); v
    };
    let lengths = [1u64, 2, p - 1, p, p + 1, 2 * p, 2 * p + 7];
    let mut cases = 0u64;
    let mut class_counts = std::collections::BTreeMap::new();
    for &size in &sizes {
        let ceil = size.div_ceil(p).max(1);
        for n in 0..=ceil {
            // 没有单元的部分是洞，读出来是 0：旧内容只在前 n 个单元里有字节（上一条腿的探针漏了这一句，n < ⌈size ÷ 净荷⌉ 时参照错了）。
            let old: Vec<u8> = content_of(size)
                .into_iter()
                .enumerate()
                .map(|(i, byte)| if (i as u64) < n * p { byte } else { 0 })
                .collect();
            let old_payload = |u: u64| -> Vec<u8> {
                let start = u * p;
                if start >= size { return vec![]; }
                old[start as usize..(start + p).min(size) as usize].to_vec()
            };
            for &off in &offsets_rel(size) {
                for &len in &lengths {
                    let w = PartialWriteOfAFile::new(n, size, FileOffsetInBytes(off), NonZeroU64::new(len).unwrap()).unwrap();
                    let bytes: Vec<u8> = (0..len).map(|i| 200 + (i % 50) as u8).collect();
                    let size_after = size.max(off + len);
                    assert_eq!(w.size_after_in_bytes(), size_after);
                    let mut want = old.clone();
                    want.resize(size_after as usize, 0);
                    want[off as usize..(off + len) as usize].copy_from_slice(&bytes);
                    let after = w.data_units_after();
                    let first = w.first_rewritten_unit().0;
                    let last_written = (off + len - 1) / p;
                    assert!(after >= n, "写之后单元数不少于写之前");
                    assert_eq!(after, n.max(last_written + 1), "n={n} size={size} off={off} len={len}");
                    assert!(first <= off / p && first <= n, "n={n} size={size} off={off}");
                    assert_eq!(w.rewritten_units(), first..after);
                    for u in 0..first {
                        let before_len = old_payload(u).len() as u64;
                        let after_len = (size_after.saturating_sub(u * p)).min(p);
                        assert!(u < n, "不重写的单元必须是写之前就有的");
                        assert_eq!(before_len, after_len, "不重写的单元声明长度不能变 n={n} size={size} off={off} len={len} u={u}");
                        assert!(w.bytes_written_into(DataUnitIndexInFile(u)).is_none(), "不重写的单元不许被写碰");
                    }
                    let mut rebuilt: Vec<u8> = Vec::new();
                    for u in 0..first { rebuilt.extend(old_payload(u)); }
                    for u in first..after {
                        let unit = DataUnitIndexInFile(u);
                        let class = w.data_unit(unit);
                        *class_counts.entry(class.name()).or_insert(0u64) += 1;
                        let start = u * p;
                        let covers_whole = off <= start && off + len >= start + p;
                        let touched = off < start + p && off + len > start;
                        match class {
                            DataUnitOfAPartialWrite::OldUnitCoveredWhole => assert!(u < n && covers_whole),
                            DataUnitOfAPartialWrite::OldUnitReadBack => assert!(u < n && !covers_whole),
                            DataUnitOfAPartialWrite::NewUnitCoveredWhole => assert!(u >= n && covers_whole),
                            DataUnitOfAPartialWrite::NewUnitReadingTheResidueOfItsSlot => assert!(u >= n && touched && !covers_whole),
                            DataUnitOfAPartialWrite::GapFilledWithZeros => assert!(u >= n && !touched),
                        }
                        let mut payload = if class.reads_the_old_unit_back() { old_payload(u) } else { vec![] };
                        payload.resize(w.declared_length_after_the_write(unit) as usize, 0);
                        assert_eq!(w.declared_length_after_the_write(unit), size_after.saturating_sub(start).min(p));
                        if let Some(b) = w.bytes_written_into(unit) {
                            let a = b.offset_in_the_payload as usize;
                            let f = b.offset_in_the_bytes_written as usize;
                            let l = b.length_in_bytes as usize;
                            payload[a..a + l].copy_from_slice(&bytes[f..f + l]);
                        } else { assert!(!touched); }
                        rebuilt.extend(payload);
                    }
                    rebuilt.resize(size_after as usize, 0);
                    assert!(rebuilt == want, "内容对不上 n={n} size={size} off={off} len={len}");
                    cases += 1;
                }
            }
        }
    }
    println!("name=probe_partial_write_split cases={cases} classes={class_counts:?}");
}
