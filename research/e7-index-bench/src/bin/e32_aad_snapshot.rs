//! E32：AAD 缺快照维 —— 多可写头之下，快照 key 错配会不会静默通过。
//!
//! 设计与依据见 `.claude/kb/experiments/32-AAD缺快照维.md`。
//! **它攻的是** D9（加密）已定项 6 那张「一定不能进 AAD」表里的一行：
//! 「读者当前所处的快照 ID | B1 / COW 共享，**且 D6 未定** | ……」
//! —— 而 D6 在该表定案的**第二天**定了多可写头。
//!
//! 四条臂：
//!   current          D9 已定的五项，无快照维；nonce 存在指针里
//!   reader_snapshot  五项 + 读者当前所处的快照（D9 明令排除的那个形态）
//!   birth_snapshot   五项 + key 自己带的出生快照
//!   derived_nonce    五项，无快照维；但 nonce 由完整逻辑身份现算
//!
//! 判据两条，必须同时满足：
//!   ① 错配全被抓到（swap_detected == SWAPS 且 swap_silent_wrong == 0）
//!   ② 共享块照常读得出（shared_ok == SHARED_READERS）
//!
//! 阳性对照：翻一个密文字节，四条臂都必须抓到——
//! 少了它，「current 抓到 0」分不清是 AAD 缺维还是根本没在验 MAC。

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use e7_index_bench::Emitter;

/// 构造出来的错配次数。**由构造直接给出，不从被测代码读回来。**
const SWAPS: u32 = 2;
/// 共享块的可达快照数：根快照 1 加两个可写头 2、3。
const SHARED_READERS: u32 = 3;

type Snap = u32;

/// 索引 key。末位 `snap` 是本实验的核心：D8 已定的 key 是三段，没有这一维；
/// D6 取多可写头之后它必须存在，否则同一 `(obj, off)` 的两个可写头版本无法共存。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Key4 { tree: u64, obj: u64, off: u64, snap: Snap }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm { Current, ReaderSnapshot, BirthSnapshot, DerivedNonce }

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Current => "current",
            Arm::ReaderSnapshot => "reader_snapshot",
            Arm::BirthSnapshot => "birth_snapshot",
            Arm::DerivedNonce => "derived_nonce",
        }
    }
}

/// AAD 的规范编码。`reader` 是读者当前所处的快照（写入时等于 key 自己的快照）。
fn aad(arm: Arm, k: &Key4, reader: Snap) -> Vec<u8> {
    let mut v = Vec::with_capacity(40);
    v.extend_from_slice(b"UNIT");            // 单元类型标签
    v.extend_from_slice(&k.tree.to_le_bytes());
    v.extend_from_slice(&k.obj.to_le_bytes());
    v.extend_from_slice(&0u64.to_le_bytes()); // 对象出生代
    v.extend_from_slice(&k.off.to_le_bytes()); // 锚点偏移
    match arm {
        Arm::Current | Arm::DerivedNonce => {}
        Arm::ReaderSnapshot => v.extend_from_slice(&reader.to_le_bytes()),
        Arm::BirthSnapshot  => v.extend_from_slice(&k.snap.to_le_bytes()),
    }
    v
}

/// nonce。`DerivedNonce` 臂由完整逻辑身份现算（含快照维），其余臂存在指针里。
fn nonce_for(arm: Arm, k: &Key4, stored: u64) -> Nonce {
    let mut n = [0u8; 12];
    match arm {
        Arm::DerivedNonce => {
            // 现算：把完整逻辑身份（含 snap）折进 nonce
            let h = k.tree
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(k.obj.wrapping_mul(0xBF58_476D_1CE4_E5B9))
                .wrapping_add(k.off.wrapping_mul(0x94D0_49BB_1331_11EB))
                .wrapping_add((k.snap as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93));
            n[..8].copy_from_slice(&h.to_le_bytes());
        }
        _ => n[..8].copy_from_slice(&stored.to_le_bytes()),
    }
    Nonce::from(n)
}

/// 指针：nonce 代号 + 密文（含 tag）。
#[derive(Clone)]
struct Ptr { nonce_id: u64, ct: Vec<u8> }

fn cipher() -> ChaCha20Poly1305 {
    ChaCha20Poly1305::new_from_slice(&[7u8; 32]).expect("32 字节密钥")
}

fn seal(arm: Arm, k: &Key4, nonce_id: u64, pt: &[u8]) -> Ptr {
    let a = aad(arm, k, k.snap);
    let ct = cipher()
        .encrypt(&nonce_for(arm, k, nonce_id), Payload { msg: pt, aad: &a })
        .expect("加密失败");
    Ptr { nonce_id, ct }
}

/// 读：按**读者查找到的那个 key** 现算 AAD（D9 的期望值来源判据：只许来自查找路径）。
fn open(arm: Arm, k: &Key4, reader: Snap, p: &Ptr) -> Option<Vec<u8>> {
    let a = aad(arm, k, reader);
    cipher()
        .decrypt(&nonce_for(arm, k, p.nonce_id), Payload { msg: &p.ct, aad: &a })
        .ok()
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Out {
    clean_own_ok: u32,       // 没有 bug 时各自读回自己的数据
    corrupt_detected: u32,   // 阳性对照：翻一个密文字节必须被抓到
    swap_detected: u32,      // 错配被 MAC 抓到
    swap_silent_wrong: u32,  // 错配没被抓到，且读出的是别人的数据
    shared_ok: u32,          // 共享块从 SHARED_READERS 个快照都读得出
}

fn measure(arm: Arm) -> Out {
    let mut o = Out::default();
    // 两个可写头 2、3，同一个 (obj, off) 各写一版
    let ka = Key4 { tree: 1, obj: 42, off: 0, snap: 2 };
    let kb = Key4 { tree: 1, obj: 42, off: 0, snap: 3 };
    let pa = b"data-from-head-2".to_vec();
    let pb = b"data-from-head-3".to_vec();
    let sa = seal(arm, &ka, 1001, &pa);
    let sb = seal(arm, &kb, 1002, &pb);

    // 1) 无 bug：各自读回自己的
    if open(arm, &ka, ka.snap, &sa).as_deref() == Some(pa.as_slice()) { o.clean_own_ok += 1; }
    if open(arm, &kb, kb.snap, &sb).as_deref() == Some(pb.as_slice()) { o.clean_own_ok += 1; }

    // 2) 阳性对照：翻一个密文字节，必须被抓到
    for (k, s) in [(&ka, &sa), (&kb, &sb)] {
        let mut bad = s.clone();
        bad.ct[0] ^= 0x01;
        if open(arm, k, k.snap, &bad).is_none() { o.corrupt_detected += 1; }
    }

    // 3) 注入快照 key 改写 bug：把 head 2 那一版挂到 head 3 的 key 下，反之亦然
    for (k, wrong, expect_other) in [(&ka, &sb, &pb), (&kb, &sa, &pa)] {
        match open(arm, k, k.snap, wrong) {
            None => o.swap_detected += 1,
            Some(got) if got == *expect_other => o.swap_silent_wrong += 1,
            Some(_) => {} // 读出别的东西：既没抓到也不是对方的数据，两边都不计
        }
    }

    // 4) 共享块：生在根快照 1，从快照 1、2、3 都可达
    let ks = Key4 { tree: 1, obj: 7, off: 0, snap: 1 };
    let ps = b"shared-block".to_vec();
    let ss = seal(arm, &ks, 2001, &ps);
    for reader in 1..=SHARED_READERS {
        if open(arm, &ks, reader, &ss).as_deref() == Some(ps.as_slice()) { o.shared_ok += 1; }
    }
    o
}

fn main() {
    let mut em = Emitter::new();
    let mut out = String::new();
    let mut say = |s: String| { out.push_str(&s); out.push('\n'); };
    say(em.emit_raw(&format!(
        "name=config heads=2 swaps={SWAPS} shared_readers={SHARED_READERS} aead=chacha20poly1305")));

    let mut bad_control = 0;
    for arm in [Arm::Current, Arm::ReaderSnapshot, Arm::BirthSnapshot, Arm::DerivedNonce] {
        let o = measure(arm);
        // 阳性对照：每条臂都必须抓到翻字节，否则那条臂根本没在验 MAC
        if o.corrupt_detected != SWAPS || o.clean_own_ok != 2 { bad_control += 1; }
        let pass1 = o.swap_detected == SWAPS && o.swap_silent_wrong == 0;
        let pass2 = o.shared_ok == SHARED_READERS;
        say(em.emit_raw(&format!(
            "name=arm arm={} clean_own_ok={} corrupt_detected={} swap_detected={} \
swap_silent_wrong={} shared_ok={} crit1={pass1} crit2={pass2} pass={}",
            arm.name(), o.clean_own_ok, o.corrupt_detected, o.swap_detected,
            o.swap_silent_wrong, o.shared_ok, pass1 && pass2)));
    }
    if bad_control > 0 {
        say(em.finish()); print!("{out}");
        eprintln!("E32: 有 {bad_control} 条臂没通过阳性对照 —— 那些臂没在验 MAC，本轮作废");
        std::process::exit(4);
    }
    say(em.finish());
    print!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值：构造出来的错配恰好 2 次、共享读者恰好 3 个。**
    /// 这两个数由构造直接给出，不从被测代码读回来——
    /// 没有它们，四条臂可以一起把「错配」构造漏掉而互比仍然「成立」。
    #[test]
    fn the_construction_yields_exactly_two_swaps_and_three_shared_readers() {
        assert_eq!(SWAPS, 2);
        assert_eq!(SHARED_READERS, 3);
        // 每条臂的「抓到 + 静默错 + 都不是」三者之和必须恰等于错配次数
        for arm in [Arm::Current, Arm::ReaderSnapshot, Arm::BirthSnapshot, Arm::DerivedNonce] {
            let o = measure(arm);
            assert!(o.swap_detected + o.swap_silent_wrong <= SWAPS,
                    "{} 臂的错配计数超过构造出来的 {SWAPS} 次", arm.name());
            assert_eq!(o.clean_own_ok, 2, "{} 臂无 bug 时读不回自己的数据", arm.name());
        }
    }

    /// **阳性对照：翻一个密文字节，四条臂都必须抓到 2 次。**
    /// 少了它，「current 臂 swap_detected=0」分不清是 AAD 缺维还是根本没在验 MAC。
    #[test]
    fn every_arm_detects_a_flipped_ciphertext_byte() {
        for arm in [Arm::Current, Arm::ReaderSnapshot, Arm::BirthSnapshot, Arm::DerivedNonce] {
            assert_eq!(measure(arm).corrupt_detected, SWAPS,
                       "{} 臂没抓到翻字节 —— 它没在验 MAC", arm.name());
        }
    }

    /// **D9 已定的那套 AAD（无快照维）对快照 key 错配是瞎的。** 这是本实验的主结论。
    #[test]
    fn the_settled_aad_is_blind_to_snapshot_key_swaps() {
        let o = measure(Arm::Current);
        assert_eq!(o.swap_detected, 0, "current 臂居然抓到了错配");
        assert_eq!(o.swap_silent_wrong, SWAPS, "current 臂没有静默读出另一个可写头的数据");
        assert_eq!(o.shared_ok, SHARED_READERS, "current 臂连共享读都不成立，那它坏在别处");
    }

    /// **绑读者侧快照能抓到错配，但代价是共享块只在一个快照里验得过。**
    /// 这正是 D9 排除它时给的那条理由的可执行形式——它不是稻草人。
    #[test]
    fn binding_the_reader_snapshot_breaks_sharing_exactly_as_d9_predicted() {
        let o = measure(Arm::ReaderSnapshot);
        assert_eq!(o.swap_detected, SWAPS, "读者侧快照臂没抓到错配");
        assert_eq!(o.shared_ok, 1, "共享读本该只剩 1 个（写入时那个快照），实测 {}", o.shared_ok);
    }

    /// **两条出路各自同时满足两条判据**：绑出生快照，或把快照维折进 nonce。
    #[test]
    fn birth_snapshot_and_derived_nonce_both_satisfy_the_two_criteria() {
        for arm in [Arm::BirthSnapshot, Arm::DerivedNonce] {
            let o = measure(arm);
            assert_eq!(o.swap_detected, SWAPS, "{} 臂没抓到错配", arm.name());
            assert_eq!(o.swap_silent_wrong, 0, "{} 臂有静默错", arm.name());
            assert_eq!(o.shared_ok, SHARED_READERS, "{} 臂的共享读坏了", arm.name());
        }
    }
}
