//! E36：多可写头的两种形态，哪一种在写第一行代码之前就带着坑
//!
//! 设计与依据见 `.claude/kb/experiments/36-多可写头两种形态.md`。
//!
//! **它攻的是两条已定项凑在一起**：
//! - D8（核心索引结构）已定项 1：`key = (locality_id, inode, offset)` —— **三段，没有快照维**
//! - D9（加密）未定项 4：nonce **现算**还是每 extent 存 —— 现算的输入就是逻辑身份，也就是 key
//!
//! 两个可写头各写一次同一个 `(locality, inode, offset)`：三段 key 下它们的逻辑身份
//! **逐字节相同** ⇒ 现算出**同一个 nonce** ⇒ 同密钥同 nonce 加两段不同明文
//! ⇒ ChaCha20 的 keystream 复用 ⇒ **两段密文异或 == 两段明文异或**。
//!
//! 这不是「MAC 会失配」那一类可检出的错，是**机密性直接没了**，
//! 而 I-6.1（nonce 不重用）自陈这类事故「不会表现为任何读写错误」。
//!
//! ## 三条臂
//!
//! | 臂 | key | 每头一棵树 | nonce 来源 |
//! |---|---|---|---|
//! | `key3_derived` | 三段（D8 已定） | 否 | 现算（D9 未定项 4 的一个取值） |
//! | `key4_derived` | 四段（加快照维） | 否 | 现算 |
//! | `clone_per_head` | 三段 | **是**（D5 的克隆形态：每头一根一树） | 现算 |
//!
//! `clone_per_head` 不是我发明的：D5（快照 / 空间记账机制）「克隆 / 可写快照要加三条」
//! 一节写的就是它——每个可写头各持一份 `prev_snap_txg` 与一份 deadlist。
//! **它是 D6（快照实现模型）定案没有点名、但仓里现成躺着的另一条路。**
//!
//! ## 判据（两条，必须同时满足）
//!
//! 1. **不许出现 keystream 复用**：`keystream_reuse_pairs == 0`。
//! 2. **两个头各自读得回自己写的那一份**：`own_reads_ok == 2`。
//!
//! ## 阳性对照
//!
//! 直接验「同密钥同 nonce 两段密文异或 == 两段明文异或」这条恒等式：
//! 在出事的那条臂上它**必须成立**（证明复用是真的、不是我数错了），
//! 在另外两条臂上它**必须不成立**（证明度量分得开）。

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use e7_index_bench::Emitter;
use std::collections::BTreeMap;

/// 两个可写头。**由构造给出，不从被测代码读回来。**
const HEADS: usize = 2;
/// 明文长度，异或恒等式按它逐字节比。
const PT_LEN: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// D8 已定的三段 key + D9 未定项 4 取「现算」
    Key3Derived,
    /// 三段 key 加一维快照
    Key4Derived,
    /// D5 里那条克隆形态：每个可写头一棵自己的树
    ClonePerHead,
}

const ARMS: [Arm; 3] = [Arm::Key3Derived, Arm::Key4Derived, Arm::ClonePerHead];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>` —— 新增一条臂不补这里就编译不过
            Arm::Key3Derived => "key3_derived",
            Arm::Key4Derived => "key4_derived",
            Arm::ClonePerHead => "clone_per_head",
        }
    }
    /// 这条臂的逻辑身份里带不带快照维。
    fn key_has_snapshot(self) -> bool {
        matches!(self, Arm::Key4Derived)
    }
    /// 这条臂给不同的可写头分不同的树 ID。
    fn tree_per_head(self) -> bool {
        matches!(self, Arm::ClonePerHead)
    }
}

/// 逻辑身份——**nonce 现算的输入就是它**（D9 未定项 4 取「现算」时的定义）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Ident {
    tree: u64,
    locality: u64,
    inode: u64,
    offset: u64,
    /// 只有带快照维的那条臂会填非零值
    snapshot: u32,
}

impl Ident {
    /// 按 `(可写头, 臂)` 造出这次写的逻辑身份。
    fn of(arm: Arm, head: u32, inode: u64, offset: u64) -> Self {
        Ident {
            tree: if arm.tree_per_head() { 100 + head as u64 } else { 100 },
            locality: 7,
            inode,
            offset,
            snapshot: if arm.key_has_snapshot() { head } else { 0 },
        }
    }
}

/// 现算 nonce：把逻辑身份折进 12 字节。
///
/// ⚠️ **这里故意不掺任何「每次写都不同」的量**——那正是「现算」的定义：
/// 同一个逻辑身份必须现算出同一个 nonce，否则读者算不出来。
/// 重写同一个 key 时要靠一个版本号来避开复用，而**版本号是每个头各自递增的**，
/// 两个头第一次写各自的版本号都是 1 ⇒ 救不了本实验这一格。
fn derive_nonce(id: &Ident, ver: u64) -> [u8; 12] {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |x: u64| {
        for b in x.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    mix(id.tree);
    mix(id.locality);
    mix(id.inode);
    mix(id.offset);
    mix(id.snapshot as u64);
    mix(ver);
    let mut n = [0u8; 12];
    n[..8].copy_from_slice(&h.to_le_bytes());
    n[8..].copy_from_slice(&(h.rotate_left(17) as u32).to_le_bytes());
    n
}

fn cipher() -> ChaCha20Poly1305 {
    ChaCha20Poly1305::new_from_slice(&[9u8; 32]).expect("32 字节密钥")
}

/// 一次写：返回 (nonce, 密文不含 tag, tag)。
fn seal(id: &Ident, ver: u64, pt: &[u8]) -> ([u8; 12], Vec<u8>, [u8; 16]) {
    let nonce = derive_nonce(id, ver);
    let sealed = cipher()
        .encrypt(&Nonce::from(nonce), Payload { msg: pt, aad: b"" })
        .expect("加密不该失败");
    let (ct, tag) = sealed.split_at(sealed.len() - 16);
    let mut t = [0u8; 16];
    t.copy_from_slice(tag);
    (nonce, ct.to_vec(), t)
}

fn open(id: &Ident, ver: u64, ct: &[u8], tag: &[u8; 16]) -> Option<Vec<u8>> {
    let nonce = derive_nonce(id, ver);
    let mut blob = ct.to_vec();
    blob.extend_from_slice(tag);
    cipher()
        .decrypt(&Nonce::from(nonce), Payload { msg: &blob, aad: b"" })
        .ok()
}

fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b).map(|(x, y)| x ^ y).collect()
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Out {
    /// 两个头写同一位置时，用上同一个 (密钥, nonce) 的对数。判据一：必须为 0
    keystream_reuse_pairs: u64,
    /// 恒等式「两段密文异或 == 两段明文异或」成立的对数。
    /// 它是 `keystream_reuse_pairs` 的**独立佐证**：复用是真的，不是我数错了
    xor_identity_holds: u64,
    /// 两个头各自读回自己那一份的次数。判据二：必须为 2
    own_reads_ok: u64,
    /// 删一个可写头时要碰的 key 区间数（连续区间计 1）
    ranges_to_drop_a_head: u64,
    /// 一次点查要比较多少个 key 分量
    key_compare_fields: u64,
}

const INODE: u64 = 42;
const OFFSET: u64 = 0;

fn measure(arm: Arm) -> Out {
    let mut o = Out::default();

    // 两个可写头各写一次同一个 (inode, offset)，内容不同
    let mut store: BTreeMap<Ident, (Vec<u8>, [u8; 16])> = BTreeMap::new();
    let mut nonces: Vec<([u8; 12], Vec<u8>, Vec<u8>)> = Vec::new(); // (nonce, 明文, 密文)
    for head in 0..HEADS as u32 {
        let id = Ident::of(arm, head, INODE, OFFSET);
        // 两个头各自是第一次写这个位置 ⇒ 各自的版本号都是 1
        let pt: Vec<u8> = (0..PT_LEN).map(|i| (i as u8).wrapping_add(head as u8 * 31)).collect();
        let (nonce, ct, tag) = seal(&id, 1, &pt);
        nonces.push((nonce, pt.clone(), ct.clone()));
        store.insert(id, (ct, tag));
    }

    // 判据一：有没有两次写用上同一个 nonce 而明文不同
    for i in 0..nonces.len() {
        for j in (i + 1)..nonces.len() {
            if nonces[i].0 == nonces[j].0 && nonces[i].1 != nonces[j].1 {
                o.keystream_reuse_pairs += 1;
            }
            // ⚠️ **这一条无条件判，不套在「已经判定复用」的分支里。**
            // 套进去的话它只能等于复用数，就不再是独立佐证了
            // （变异 M5 实测：把它改成恒真，一个测试都不红）。
            if xor(&nonces[i].2, &nonces[j].2) == xor(&nonces[i].1, &nonces[j].1) {
                o.xor_identity_holds += 1;
            }
        }
    }

    // 判据二：各头读回自己那一份
    for head in 0..HEADS as u32 {
        let id = Ident::of(arm, head, INODE, OFFSET);
        let want: Vec<u8> = (0..PT_LEN).map(|i| (i as u8).wrapping_add(head as u8 * 31)).collect();
        if let Some((ct, tag)) = store.get(&id) {
            if open(&id, 1, ct, tag).as_deref() == Some(want.as_slice()) {
                o.own_reads_ok += 1;
            }
        }
    }

    // 运营侧的两个量，供两种形态对照
    o.ranges_to_drop_a_head = match arm {
        // 没有 `_ =>`
        // 三段 key 无快照维：两个头共用同一个 key 槽，「删一个头」没有对象可指
        Arm::Key3Derived => 0,
        // 快照维在 key 最低位：同一头的 key 散布在每个 (inode, offset) 分组里
        // ⇒ 区间数正比于被碰到的对象数，这里只有 1 个对象
        Arm::Key4Derived => 1,
        // 每头一棵树：删一个头 = 丢一棵树，一个区间
        Arm::ClonePerHead => 1,
    };
    o.key_compare_fields = if arm.key_has_snapshot() { 4 } else { 3 };
    o
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config heads={HEADS} pt_len={PT_LEN} aead=chacha20poly1305 note=多可写头两种形态"
        ))
    );
    let mut bad_control = 0;
    for arm in ARMS {
        let o = measure(arm);
        // 阳性对照：出事那条臂的异或恒等式必须成立，否则度量本身没在看 keystream
        if o.keystream_reuse_pairs != o.xor_identity_holds {
            bad_control += 1;
        }
        let pass = o.keystream_reuse_pairs == 0 && o.own_reads_ok == HEADS as u64;
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=arm arm={} keystream_reuse_pairs={} xor_identity_holds={} own_reads_ok={} \
                 ranges_to_drop_a_head={} key_compare_fields={} pass={}",
                arm.name(),
                o.keystream_reuse_pairs,
                o.xor_identity_holds,
                o.own_reads_ok,
                o.ranges_to_drop_a_head,
                o.key_compare_fields,
                pass
            ))
        );
    }
    if bad_control > 0 {
        println!("{}", em.finish());
        eprintln!("E36：有 {bad_control} 条臂的复用计数与异或恒等式对不上——度量作废");
        std::process::exit(4);
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **D8 已定的三段 key 配 D9 未定项 4 取「现算」⇒ 恰好一对 keystream 复用。**
    /// 绝对值：两个头 ⇒ 恰好 1 对，不是「大于零」。
    #[test]
    fn the_settled_three_segment_key_with_derived_nonce_reuses_the_keystream() {
        let o = measure(Arm::Key3Derived);
        assert_eq!(o.keystream_reuse_pairs, 1, "两个头写同一位置，该恰好 1 对复用");
        assert_eq!(
            o.xor_identity_holds, 1,
            "复用是真的：两段密文异或该等于两段明文异或"
        );
    }

    /// **阳性对照的另一半：另外两条臂上那条恒等式必须不成立。**
    /// 少了它，「恒等式成立」分不清是复用还是我的异或函数恒等。
    #[test]
    fn the_other_two_arms_do_not_reuse_and_the_identity_fails_there() {
        for arm in [Arm::Key4Derived, Arm::ClonePerHead] {
            let o = measure(arm);
            assert_eq!(o.keystream_reuse_pairs, 0, "{} 不该有复用", arm.name());
            assert_eq!(o.xor_identity_holds, 0, "{} 上那条恒等式不该成立", arm.name());
        }
    }

    /// **两条出路都同时满足两条判据**：加快照维，或每头一棵树。
    #[test]
    fn both_alternatives_satisfy_both_criteria() {
        for arm in [Arm::Key4Derived, Arm::ClonePerHead] {
            let o = measure(arm);
            assert_eq!(o.keystream_reuse_pairs, 0, "{}", arm.name());
            assert_eq!(o.own_reads_ok, HEADS as u64, "{} 两个头都该读回自己的", arm.name());
        }
    }

    /// **出事那条臂上，两个头连「各读回自己的」都做不到**——
    /// 三段 key 下它们共用同一个 key 槽，后写的覆盖先写的。
    /// ⚠️ 这条独立于加密：**就算不加密，这条臂也已经错了。**
    #[test]
    fn under_the_three_segment_key_the_two_heads_collide_on_one_slot() {
        let o = measure(Arm::Key3Derived);
        assert_eq!(
            o.own_reads_ok, 1,
            "两个头共用一个 key 槽 ⇒ 只有后写的那个读得回自己"
        );
        // 构造自证：两个头算出的逻辑身份逐字段相同
        assert_eq!(
            Ident::of(Arm::Key3Derived, 0, INODE, OFFSET),
            Ident::of(Arm::Key3Derived, 1, INODE, OFFSET),
            "三段 key 下两个头的逻辑身份该完全相同"
        );
    }

    /// **判别力：现算 nonce 确实由逻辑身份决定。**
    /// 若它对身份不敏感，全部三条臂都会复用，结论就是假的。
    #[test]
    fn the_derived_nonce_actually_depends_on_the_identity() {
        let a = Ident::of(Arm::Key4Derived, 0, INODE, OFFSET);
        let b = Ident::of(Arm::Key4Derived, 1, INODE, OFFSET);
        let c = Ident::of(Arm::ClonePerHead, 0, INODE, OFFSET);
        let d = Ident::of(Arm::ClonePerHead, 1, INODE, OFFSET);
        assert_ne!(derive_nonce(&a, 1), derive_nonce(&b, 1), "只差快照维，nonce 必须变");
        assert_ne!(derive_nonce(&c, 1), derive_nonce(&d, 1), "只差树 ID，nonce 必须变");
        assert_ne!(derive_nonce(&a, 1), derive_nonce(&a, 2), "只差版本号，nonce 必须变");
    }

    /// **版本号救不了这一格**：两个头各自都是第一次写，版本号都是 1。
    /// ⚠️ 这条挡住一种自然的辩解（「加个版本号就好了」）。
    #[test]
    fn a_per_head_version_counter_does_not_save_the_three_segment_key() {
        let a = Ident::of(Arm::Key3Derived, 0, INODE, OFFSET);
        let b = Ident::of(Arm::Key3Derived, 1, INODE, OFFSET);
        assert_eq!(derive_nonce(&a, 1), derive_nonce(&b, 1), "各自第一次写 ⇒ 同一个 nonce");
    }

    /// **异或恒等式本身要有判别力**：不同 nonce 下它必须不成立。
    /// 少了这条，`xor_identity_holds` 可能恒真而没人发现。
    #[test]
    fn the_xor_identity_is_not_vacuous() {
        let pt1: Vec<u8> = (0..PT_LEN).map(|i| i as u8).collect();
        let pt2: Vec<u8> = (0..PT_LEN).map(|i| (i as u8).wrapping_add(31)).collect();
        let a = Ident::of(Arm::Key4Derived, 0, INODE, OFFSET);
        let b = Ident::of(Arm::Key4Derived, 1, INODE, OFFSET);
        let (_, c1, _) = seal(&a, 1, &pt1);
        let (_, c2, _) = seal(&b, 1, &pt2);
        assert_ne!(xor(&c1, &c2), xor(&pt1, &pt2), "不同 nonce 下恒等式不该成立");
        // 同 nonce 下必须成立
        let (_, d1, _) = seal(&a, 1, &pt1);
        let (_, d2, _) = seal(&a, 1, &pt2);
        assert_eq!(xor(&d1, &d2), xor(&pt1, &pt2), "同 nonce 下恒等式该成立");
    }
}
