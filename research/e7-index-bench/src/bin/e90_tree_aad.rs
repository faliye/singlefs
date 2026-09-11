//! E90：树 ID 进 AAD 与跨头共享 —— D6 定案连锁第 4 处那条推论的补跑。
//!
//! 设计与判据见 `.claude/kb/experiments/90-树ID进AAD与跨头共享.md`（跑前写死）。
//!
//! 推论（D6 连锁 4）：「AAD 含树 ID，每头一棵树下两个头的 AAD 天然不同
//! ⇒ E31 那两次静默错配应当被 MAC 抓住。」
//! 反面（连锁 4 没问的）：树 ID 期望值来自查找路径（D9 已定项 6），
//! 而克隆不触碰已发布字节（D6 判据 4）⇒ 克隆点之前诞生、合法共享进新头的块，
//! 密文 AAD 里的树仍是出生树——**合法共享读会不会和错配一起被拒？**
//!
//! 三条臂：
//!   reader_tree     树维 = 读者当前所在的树（连锁 4 推论的字面形态）
//!   pointer_tree    树维 = 指针里存的出生树（「从指针读回来」的退化形态，阴性对照）
//!   ancestry_birth  （出生树, 出生 txg）进 AAD；指针带声明，读者先拿克隆祖先表验收
//!                   （E31 出路 1「出生快照进 AAD」映射到每头一棵树）

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use e7_index_bench::Emitter;
use std::collections::BTreeMap;

/// 由构造直接给出的数，**不从被测代码读回来**。
const SWAPS: u32 = 3;
const FORGED: u32 = 1;
const SHARED_READS: u32 = 6;
const SEALED: u32 = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Birth { tree: u64, txg: u64 }

/// 指针：nonce 代号 + 出生声明 + 密文（含 tag）。
/// nonce 按 D9 已定项 4 存指针，不现算。
#[derive(Clone)]
struct BlockPointer { nonce_identifier: u64, claim: Birth, ciphertext: Vec<u8> }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm { ReaderTree, PointerTree, AncestryBirth }

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::ReaderTree => "reader_tree",
            Arm::PointerTree => "pointer_tree",
            Arm::AncestryBirth => "ancestry_birth",
        }
    }
}

/// 克隆谱系：头 → (origin 头, 克隆点 txg)。头 1 是根。
fn heads() -> BTreeMap<u64, Option<(u64, u64)>> {
    BTreeMap::from([
        (1, None),
        (2, Some((1, 10))),
        (3, Some((2, 20))),
        (4, Some((1, 10))),
    ])
}

/// 读者的祖先验收表：[(树, 该树里可继承的最大出生 txg)]。
/// 逐级 min 夹住：孩子从祖父继承到的，不可能多于父亲继承到的。
fn ancestry_chain(clone_lineage: &BTreeMap<u64, Option<(u64, u64)>>, reader_head: u64) -> Vec<(u64, u64)> {
    let mut inheritable_ancestors = Vec::new();
    let mut current_head = reader_head;
    let mut clamp = u64::MAX;
    while let Some((parent_head, clone_point_txg)) = clone_lineage[&current_head] {
        let inheritable_up_to_txg = clone_point_txg.min(clamp);
        inheritable_ancestors.push((parent_head, inheritable_up_to_txg));
        clamp = inheritable_up_to_txg;
        current_head = parent_head;
    }
    inheritable_ancestors
}

/// 祖先验收：声明的出生 (树, txg) 对读者 reader 是不是合法可继承。
fn ancestry_accepts(clone_lineage: &BTreeMap<u64, Option<(u64, u64)>>, reader: u64, claim: Birth) -> bool {
    if claim.tree == reader { return true; }
    ancestry_chain(clone_lineage, reader).iter()
        .any(|&(ancestor_tree, up_to)| ancestor_tree == claim.tree && claim.txg <= up_to)
}

/// AAD 规范编码：五项照 D9 已定项 6（单元类型标签、树、对象、对象出生代、锚点偏移）。
/// 树维按臂取；`ancestry_birth` 额外把出生 txg 也绑进去——声明必须被 MAC 认证，
/// 否则验收只看指针，伪造声明畅通无阻。
fn aad(arm: Arm, tree_field: u64, object_identifier: u64, anchor_offset: u64, birth_txg: u64) -> Vec<u8> {
    let mut associated_data_bytes = Vec::with_capacity(48);
    associated_data_bytes.extend_from_slice(b"UNIT");
    associated_data_bytes.extend_from_slice(&tree_field.to_le_bytes());
    associated_data_bytes.extend_from_slice(&object_identifier.to_le_bytes());
    associated_data_bytes.extend_from_slice(&0u64.to_le_bytes()); // 对象出生代
    associated_data_bytes.extend_from_slice(&anchor_offset.to_le_bytes());
    if arm == Arm::AncestryBirth {
        associated_data_bytes.extend_from_slice(&birth_txg.to_le_bytes());
    }
    associated_data_bytes
}

fn cipher() -> ChaCha20Poly1305 {
    ChaCha20Poly1305::new_from_slice(&[9u8; 32]).expect("32 字节密钥")
}

fn nonce_of(nonce_identifier: u64) -> Nonce {
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[..8].copy_from_slice(&nonce_identifier.to_le_bytes());
    Nonce::from(nonce_bytes)
}

/// 写侧：出生时封。写者的上下文就是出生树 ⇒ 三条臂的树维在写侧都是出生树。
fn seal(arm: Arm, birth: Birth, object_identifier: u64, anchor_offset: u64, nonce_identifier: u64, plaintext: &[u8]) -> BlockPointer {
    let associated_data = aad(arm, birth.tree, object_identifier, anchor_offset, birth.txg);
    let ciphertext = cipher()
        .encrypt(&nonce_of(nonce_identifier), Payload { msg: plaintext, aad: &associated_data })
        .expect("加密失败");
    BlockPointer { nonce_identifier, claim: birth, ciphertext }
}

/// 读侧：期望值按臂取。返回 None = 被拒（MAC 失配或祖先验收拒收）。
fn open(
    arm: Arm,
    clone_lineage: &BTreeMap<u64, Option<(u64, u64)>>,
    reader_tree: u64,
    object_identifier: u64,
    anchor_offset: u64,
    pointer: &BlockPointer,
) -> Option<Vec<u8>> {
    let tree_field = match arm {
        Arm::ReaderTree => reader_tree,
        Arm::PointerTree | Arm::AncestryBirth => pointer.claim.tree,
    };
    if arm == Arm::AncestryBirth && !ancestry_accepts(clone_lineage, reader_tree, pointer.claim) {
        return None; // 验收拒收：声明不在读者的克隆祖先链可继承范围内
    }
    let associated_data = aad(arm, tree_field, object_identifier, anchor_offset, pointer.claim.txg);
    cipher()
        .decrypt(&nonce_of(pointer.nonce_identifier), Payload { msg: &pointer.ciphertext, aad: &associated_data })
        .ok()
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
struct ArmOutcome {
    own_ok: u32,            // 无 bug 时各头读回自己的数据
    corrupt_detected: u32,  // 阳性对照：翻密文字节被抓
    swap_detected: u32,     // 换挂被拒
    swap_silent_wrong: u32, // 换挂静默读出他人数据
    forged_detected: u32,   // 伪造出生声明被抓
    shared_ok: u32,         // 合法共享读成功
}

fn measure(arm: Arm) -> ArmOutcome {
    let clone_lineage = heads();
    let mut outcome = ArmOutcome::default();

    // extent 表（出生、对象、偏移、明文）。S/S2 是合法共享；A/B/C 是各头分叉后的私有版本。
    let shared_pre_clone_extent  = (Birth { tree: 1, txg: 5 },  7u64, 0u64, b"shared-pre-clone".to_vec());
    let shared_mid_chain_extent = (Birth { tree: 2, txg: 15 }, 8u64, 0u64, b"shared-mid-chain".to_vec());
    let head_2_private_extent  = (Birth { tree: 2, txg: 25 }, 42u64, 0u64, b"private-head-2".to_vec());
    let head_3_private_extent  = (Birth { tree: 3, txg: 25 }, 42u64, 0u64, b"private-head-3".to_vec());
    let head_4_private_extent  = (Birth { tree: 4, txg: 30 }, 42u64, 0u64, b"private-head-4".to_vec());

    let sealed: Vec<(&(Birth, u64, u64, Vec<u8>), BlockPointer)> = [&shared_pre_clone_extent, &shared_mid_chain_extent, &head_2_private_extent, &head_3_private_extent, &head_4_private_extent]
        .into_iter()
        .enumerate()
        .map(|(extent_index, extent)| (extent, seal(arm, extent.0, extent.1, extent.2, 3000 + extent_index as u64, &extent.3)))
        .collect();

    // 1) 无 bug：每个 extent 从自己的出生头读回
    for (extent, pointer) in &sealed {
        if open(arm, &clone_lineage, extent.0.tree, extent.1, extent.2, pointer).as_deref() == Some(extent.3.as_slice()) {
            outcome.own_ok += 1;
        }
    }

    // 2) 阳性对照：逐个翻一个密文字节
    for (extent, pointer) in &sealed {
        let mut corrupted_pointer = pointer.clone();
        corrupted_pointer.ciphertext[0] ^= 0x01;
        if open(arm, &clone_lineage, extent.0.tree, extent.1, extent.2, &corrupted_pointer).is_none() {
            outcome.corrupt_detected += 1;
        }
    }

    // 3) 合法共享读：S 从 1/2/3/4，S2 从 2/3
    let (shared_extent, shared_pointer) = (&sealed[0].0, &sealed[0].1);
    for reader in [1u64, 2, 3, 4] {
        if open(arm, &clone_lineage, reader, shared_extent.1, shared_extent.2, shared_pointer).as_deref() == Some(shared_extent.3.as_slice()) {
            outcome.shared_ok += 1;
        }
    }
    let (mid_chain_extent, mid_chain_pointer) = (&sealed[1].0, &sealed[1].1);
    for reader in [2u64, 3] {
        if open(arm, &clone_lineage, reader, mid_chain_extent.1, mid_chain_extent.2, mid_chain_pointer).as_deref() == Some(mid_chain_extent.3.as_slice()) {
            outcome.shared_ok += 1;
        }
    }

    // 4) 换挂注入恰 3 次：A→头 3、B→头 2、C→头 3（对象与偏移相同，只有树/出生能分辨）
    let (head_2_private_pointer, head_3_private_pointer, head_4_private_pointer) = (&sealed[2].1, &sealed[3].1, &sealed[4].1);
    for (reader, wrong_pointer, other_plaintext) in [
        (3u64, head_2_private_pointer, &head_2_private_extent.3),
        (2u64, head_3_private_pointer, &head_3_private_extent.3),
        (3u64, head_4_private_pointer, &head_4_private_extent.3),
    ] {
        match open(arm, &clone_lineage, reader, 42, 0, wrong_pointer) {
            None => outcome.swap_detected += 1,
            Some(read_plaintext) if read_plaintext == *other_plaintext => outcome.swap_silent_wrong += 1,
            Some(_) => {}
        }
    }

    // 5) 伪造声明注入恰 1 次：A 挂到头 3，且把指针声明改成祖先验收能过的 (2, 15)。
    //    验收挡不住它——必须由「声明进了 AAD」那一层抓住。
    let mut forged = head_2_private_pointer.clone();
    forged.claim = Birth { tree: 2, txg: 15 };
    if open(arm, &clone_lineage, 3, 42, 0, &forged).is_none() {
        outcome.forged_detected += 1;
    }
    outcome
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output_buffer = String::new();
    let mut say = |line: String| { output_buffer.push_str(&line); output_buffer.push('\n'); };
    say(emitter.emit_raw(&format!(
        "name=config heads=4 sealed={SEALED} swaps={SWAPS} forged={FORGED} \
         shared_reads={SHARED_READS} aead=chacha20poly1305 nonce=stored")));

    let mut arms_failing_positive_control = 0;
    for arm in [Arm::ReaderTree, Arm::PointerTree, Arm::AncestryBirth] {
        let outcome = measure(arm);
        if outcome.corrupt_detected != SEALED || outcome.own_ok != SEALED { arms_failing_positive_control += 1; }
        let passes_criterion_one = outcome.swap_detected == SWAPS && outcome.swap_silent_wrong == 0
            && outcome.forged_detected == FORGED;
        let passes_criterion_two = outcome.shared_ok == SHARED_READS;
        say(emitter.emit_raw(&format!(
            "name=arm arm={} own_ok={} corrupt_detected={} swap_detected={} \
             swap_silent_wrong={} forged_detected={} shared_ok={} crit1={passes_criterion_one} crit2={passes_criterion_two} pass={}",
            arm.name(), outcome.own_ok, outcome.corrupt_detected, outcome.swap_detected,
            outcome.swap_silent_wrong, outcome.forged_detected, outcome.shared_ok, passes_criterion_one && passes_criterion_two)));
    }
    if arms_failing_positive_control > 0 {
        say(emitter.finish()); print!("{output_buffer}");
        eprintln!("E90: 有 {arms_failing_positive_control} 条臂没过阳性对照 —— 没在验 MAC，本轮作废");
        std::process::exit(4);
    }
    say(emitter.finish());
    print!("{output_buffer}");
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARMS: [Arm; 3] = [Arm::ReaderTree, Arm::PointerTree, Arm::AncestryBirth];

    /// **构造常量与守恒**：错配 3、伪造 1、共享读 6、封 5；
    /// 每臂换挂三判之和 ≤ 构造数，无 bug 读回恰 5。
    #[test]
    fn construction_constants_and_conservation() {
        assert_eq!((SWAPS, FORGED, SHARED_READS, SEALED), (3, 1, 6, 5));
        for arm in ARMS {
            let outcome = measure(arm);
            assert!(outcome.swap_detected + outcome.swap_silent_wrong <= SWAPS, "{}", arm.name());
            assert_eq!(outcome.own_ok, SEALED, "{} 臂无 bug 时读不回自己的数据", arm.name());
        }
    }

    /// **阳性对照**：翻密文字节，每臂恰抓 5 次。
    #[test]
    fn every_arm_detects_flipped_ciphertext_bytes() {
        for arm in ARMS {
            assert_eq!(measure(arm).corrupt_detected, SEALED,
                "{} 臂没抓全翻字节——它没在验 MAC", arm.name());
        }
    }

    /// **连锁 4 推论的两半，手推钉死**：reader_tree 抓全三次换挂（推论成立的那一半），
    /// 但合法共享读只剩 2（S 只有出生头 1 读得出、S2 只有出生头 2 读得出）——
    /// 推论没问的那一半：树维分不清「合法继承」与「非法换挂」，两者都是「AAD 树 ≠ 读者树」。
    #[test]
    fn reader_tree_catches_swaps_but_rejects_inherited_reads() {
        let outcome = measure(Arm::ReaderTree);
        assert_eq!(outcome.swap_detected, SWAPS);
        assert_eq!(outcome.swap_silent_wrong, 0);
        assert_eq!(outcome.forged_detected, FORGED, "伪造声明不影响它——它压根不看指针声明");
        assert_eq!(outcome.shared_ok, 2, "合法共享读该恰剩出生头那 2 次");
    }

    /// **「从指针读回来」的退化，手推钉死**：pointer_tree 共享读全过，
    /// 但三次换挂全部静默——指针自带声明、AAD 跟着指针走，MAC 必然通过。
    /// 这是 D9 期望值来源判据（不许从指针读回来）的可执行形式。
    #[test]
    fn pointer_tree_shares_fine_but_is_blind_to_every_swap() {
        let outcome = measure(Arm::PointerTree);
        assert_eq!(outcome.shared_ok, SHARED_READS);
        assert_eq!(outcome.swap_detected, 0);
        assert_eq!(outcome.swap_silent_wrong, SWAPS, "三次换挂该全部静默读出他人数据");
        // 伪造只改了声明里的 txg（树仍是 2），而本臂的 AAD 只绑树字段
        // ⇒ 伪造对它完全不可见。跑前手推预测它抓 1 次（以为任何伪造都动 AAD），
        // 实测 0——它连「声明被 MAC 认证」这一层都没有，比预测的更瞎。
        assert_eq!(outcome.forged_detected, 0, "本臂的 AAD 不含 txg，txg 伪造该完全不可见");
    }

    /// **出路臂，手推钉死**：ancestry_birth 两条判据全过——
    /// 换挂 3 次全被祖先验收拒收，伪造声明被 MAC 抓住，合法共享读 6 次全过。
    #[test]
    fn ancestry_birth_satisfies_both_criteria() {
        let outcome = measure(Arm::AncestryBirth);
        assert_eq!(outcome.swap_detected, SWAPS);
        assert_eq!(outcome.swap_silent_wrong, 0);
        assert_eq!(outcome.forged_detected, FORGED);
        assert_eq!(outcome.shared_ok, SHARED_READS);
    }

    /// 祖先链算术钉死：头 3 的链是 [(2,20),(1,10)]，头 4 是 [(1,10)]，头 1 为空。
    #[test]
    fn ancestry_chain_is_pinned() {
        let clone_lineage = heads();
        assert_eq!(ancestry_chain(&clone_lineage, 3), vec![(2, 20), (1, 10)]);
        assert_eq!(ancestry_chain(&clone_lineage, 4), vec![(1, 10)]);
        assert_eq!(ancestry_chain(&clone_lineage, 1), vec![]);
    }

    /// 逐级 min 夹住：孩子从祖父继承到的不许多于父亲继承到的。
    /// 造一个「父亲的上游克隆点比孩子的克隆点晚」的合成谱系钉住 min。
    #[test]
    fn ancestry_chain_clamps_with_minimum() {
        let clone_lineage = BTreeMap::from([
            (1, None),
            (2, Some((1, 30))),
            (3, Some((2, 12))), // 孩子在 12 就分走了 ⇒ 从 1 继承的也被夹到 12
        ]);
        assert_eq!(ancestry_chain(&clone_lineage, 3), vec![(2, 12), (1, 12)]);
    }

    /// 验收边界钉死：克隆点当代的内容算继承（≤），晚一代就不算。
    #[test]
    fn acceptance_boundary_is_inclusive() {
        let clone_lineage = heads();
        assert!(ancestry_accepts(&clone_lineage, 3, Birth { tree: 2, txg: 20 }), "恰在克隆点的该收");
        assert!(!ancestry_accepts(&clone_lineage, 3, Birth { tree: 2, txg: 21 }), "晚一代的不该收");
        assert!(ancestry_accepts(&clone_lineage, 3, Birth { tree: 1, txg: 10 }));
        assert!(!ancestry_accepts(&clone_lineage, 3, Birth { tree: 1, txg: 11 }));
        assert!(!ancestry_accepts(&clone_lineage, 3, Birth { tree: 4, txg: 1 }), "旁支树不在链上，不该收");
        assert!(ancestry_accepts(&clone_lineage, 2, Birth { tree: 2, txg: 999 }), "自己树里的一律收");
    }

    /// 换挂注入必须真的构造在「对象与偏移相同」上——否则五项里的对象/偏移维
    /// 就能抓到，测的不再是树维。
    #[test]
    fn swaps_share_object_and_offset_so_only_the_tree_dimension_can_tell() {
        // A/B/C 三个私有版本都在 (obj=42, off=0)：pointer_tree 臂三次全静默正是证明
        // ——若对象或偏移不同，它的 AAD 会失配，静默数就到不了 3。
        let outcome = measure(Arm::PointerTree);
        assert_eq!(outcome.swap_silent_wrong, SWAPS);
    }
}
