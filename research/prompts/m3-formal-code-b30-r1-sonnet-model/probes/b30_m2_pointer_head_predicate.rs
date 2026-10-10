//! b30 正推腿 M2：core 的指针头部判（`NodePointer` / `DataPointer` 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from`）
//! 与 GPU 恢复内核两个 WGSL 谓词的逐字翻译比：`pointer_encryption_and_compression_fields_are_the_first_version`（crash_judge_common.wgsl 第 507–513 行）
//! 加 `pointer_extent_offset_is_zero`（偏移 32 的 2 字节）。翻译是腿照 WGSL 原文写的（推的：WGSL 本身这里没跑），
//! 比的是字段偏移与宽度：头部 34 字节里每个位置单字节改成每一个非 0 值、再加随机组合，两边接受与否逐个相同。
use singlefs_core::bytes::ByteReader;
use singlefs_core::pointer::{DataPointer, NodePointer};

// WGSL: all_zero(base, o, 28u) && rd8(o+28)==0 && rd8(o+29)==0 && rd16(o+30)==0
fn wgsl_encryption_and_compression_fields_are_the_first_version(p: &[u8]) -> bool {
    p[0..28].iter().all(|b| *b == 0) && p[28] == 0 && p[29] == 0 && u16::from_le_bytes([p[30], p[31]]) == 0
}
// WGSL: rd16(base, o + 32u) == 0u
fn wgsl_extent_offset_is_zero(p: &[u8]) -> bool {
    u16::from_le_bytes([p[32], p[33]]) == 0
}
fn wgsl_accepts(p: &[u8]) -> bool {
    wgsl_encryption_and_compression_fields_are_the_first_version(p) && wgsl_extent_offset_is_zero(p)
}

#[test]
fn b30_m2_the_core_pointer_head_judgement_and_the_wgsl_predicates_accept_the_same_heads() {
    let mut disagreements = Vec::new();
    let mut checked = 0u64;
    let mut check = |pointer: &[u8]| {
        let node = NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(&mut ByteReader::at(pointer, 0)).is_ok();
        let data = DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(&mut ByteReader::at(pointer, 0)).is_ok();
        let wgsl = wgsl_accepts(pointer);
        checked += 1;
        if node != wgsl || data != wgsl {
            disagreements.push((pointer[..34].to_vec(), node, data, wgsl));
        }
    };
    let base = vec![0u8; 88];
    check(&base);
    for position in 0..34usize {
        for value in 1..=255u8 {
            let mut pointer = base.clone();
            pointer[position] = value;
            check(&pointer);
        }
    }
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    for _ in 0..20_000 {
        let mut pointer = base.clone();
        for byte in pointer.iter_mut().take(34) {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            // 大多数位置保持 0，少数位置随机，覆盖「几段同时非 0」
            if (state >> 59) == 0 {
                *byte = u8::try_from(state >> 40 & 0xFF).unwrap();
            }
        }
        check(&pointer);
    }
    println!("B30HEADS checked={checked} disagreements={}", disagreements.len());
    assert!(disagreements.is_empty(), "{:?}", &disagreements[..disagreements.len().min(5)]);
}
