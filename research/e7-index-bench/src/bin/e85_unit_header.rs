//! E85：单元头的字段表 —— 把「谁要求哪个头字段」摊成一张可机检的需求矩阵，
//! 三个候选字段集各自够得着哪些判定、各花多少字节。C68 / C69 / C72 收口决策的备料。
//!
//! ## 为什么要有这个实验
//!
//! - C68（不变量白名单与已定条款打架）：I-6.2（明文头字段白名单）的六项里没有一项是
//!   D18（块里携带什么信息）已定项 3 要求数据单元明文携带的五元组，两者的射程没人收口。
//! - C69（已定不变量没有字段可判）：I-1.2（块头代号不超当前）要块的 birth、
//!   I-1.4（块头 fsid 一致）要 fsid，而五元组里都没有——两条已定不变量在第一版落空。
//! - C72（extent 和单元什么关系没人答）的派生问题之一也是头：单元头 55 字节至今是
//!   D21（权威态与派生态的分界）的「提议值，尚未定案」。
//!
//! 收口是决策；E85（单元头的字段表）先把矩阵与字节账算成可机检的东西。
//!
//! ## 三个候选字段集
//!
//! | 集 | 内容 | 出处 |
//! |---|---|---|
//! | set_a_settled | 已定的最小集：五元组 + 声明长度 | D18（块里携带什么信息）已定项 3 + D4（校验和位置）已定项 6 |
//! | set_b_repair | A + magic + 格式版本 + flags + fsid + 块 birth + 头校验和 | C69（已定不变量没有字段可判）的修补方向 + D18 未定项 7 骨架 |
//! | set_c_whitelist | I-6.2（明文头字段白名单）字面六项 | invariants.md 逐字 |
//!
//! ## 需求行（每行一条已定条款/不变量点名的判定，字段名逐字对应）
//!
//! 自身逻辑地址那一行**故意列为「三个集都缺」**：I-1.1（块头自述逻辑地址）要它，
//! 而 D18（块里携带什么信息）已定项 3 的五元组用「锚点偏移 + 对象身份」表达位置、
//! 禁放清单又禁物理落点——**「自身逻辑地址」在数据单元头上指什么，本身就是 C68 要收口的一问**。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. 矩阵逐格钉死（单测）：set_a 判不了 I-1.2（块头代号不超当前）/ I-1.4（块头 fsid 一致）/
//!    头自证——C69（已定不变量没有字段可判）的机检形态；set_c 判不了扫描重建那一整行
//!    （五元组一项都没有）；set_b 除「自身逻辑地址」外全够。
//! 2. 字节账闭式：各集合计、占 16 KiB / 32 KiB 单元的百分比。宽度是标注过的假设，
//!    改宽度数字跟着变、矩阵不变。
//! 3. 白名单冲突必须显式报出：set_b 里不在 I-6.2（明文头字段白名单）六项内的字段逐个点名
//!    ——那就是 C68（不变量白名单与已定条款打架）收口要处置的清单。
//! 4. 不判「选哪个集」——那是收口决策；E85（单元头的字段表）交矩阵与账。
//!
//! ## 它答不了的
//!
//! 宽度全是假设（tree_id/obj_id/obj_birth/anchor_off 取 8 字节、头校验和取 4 字节
//! ——注意 E23（journal 几何）的记录头校验和是 32 字节，单元头要不要同宽是收口的一问）；
//! 「自身逻辑地址在数据单元上指什么」不回答；加密开启后哪些字段挪进密文侧不回答。

use e7_index_bench::Emitter;

/// 字段全集。宽度是**标注过的假设**，不是定案。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Field {
    Magic,        // 4
    FmtVer,       // 2
    Flags,        // 2
    UnitType,     // 1（五元组）
    TreeId,       // 8（五元组）
    ObjId,        // 8（五元组）
    ObjBirth,     // 8（五元组，inode generation）
    AnchorOff,    // 8（五元组）
    DeclaredLen,  // 2（D4 已定项 6）
    Fsid,         // 16
    BlockBirth,   // 8（块的出生代——不是 ObjBirth）
    HdrCsum,      // 4（假设；记录头那边是 32）
    NonceEpoch,   // 4（I-6.2 的「nonce 代号」）
    Mac,          // 16（I-6.2 的 MAC；加密开启才有）
    SelfLaddr,    // 8（I-1.1 要的「自身逻辑地址」——数据单元上指什么未收口）
}

impl Field {
    fn width(self) -> u64 {
        match self {
            Field::Magic => 4,
            Field::FmtVer => 2,
            Field::Flags => 2,
            Field::UnitType => 1,
            Field::TreeId => 8,
            Field::ObjId => 8,
            Field::ObjBirth => 8,
            Field::AnchorOff => 8,
            Field::DeclaredLen => 2,
            Field::Fsid => 16,
            Field::BlockBirth => 8,
            Field::HdrCsum => 4,
            Field::NonceEpoch => 4,
            Field::Mac => 16,
            Field::SelfLaddr => 8,
        }
    }
    fn tag(self) -> &'static str {
        match self {
            Field::Magic => "magic",
            Field::FmtVer => "fmt_ver",
            Field::Flags => "flags",
            Field::UnitType => "unit_type",
            Field::TreeId => "tree_id",
            Field::ObjId => "obj_id",
            Field::ObjBirth => "obj_birth",
            Field::AnchorOff => "anchor_off",
            Field::DeclaredLen => "declared_len",
            Field::Fsid => "fsid",
            Field::BlockBirth => "block_birth",
            Field::HdrCsum => "hdr_csum",
            Field::NonceEpoch => "nonce_epoch",
            Field::Mac => "mac",
            Field::SelfLaddr => "self_laddr",
        }
    }
}

const FIVE_TUPLE: [Field; 5] =
    [Field::UnitType, Field::TreeId, Field::ObjId, Field::ObjBirth, Field::AnchorOff];

fn set_a_settled() -> Vec<Field> {
    let mut v = FIVE_TUPLE.to_vec();
    v.push(Field::DeclaredLen);
    v
}
fn set_b_repair() -> Vec<Field> {
    let mut v = set_a_settled();
    v.extend([Field::Magic, Field::FmtVer, Field::Flags, Field::Fsid, Field::BlockBirth, Field::HdrCsum]);
    v
}
fn set_c_whitelist() -> Vec<Field> {
    vec![Field::Magic, Field::FmtVer, Field::Flags, Field::NonceEpoch, Field::DeclaredLen, Field::Mac]
}

/// 一条需求：哪条条款、要哪些字段。
struct Req {
    name: &'static str,
    needs: Vec<Field>,
}

fn requirements() -> Vec<Req> {
    vec![
        Req { name: "I-1.1_self_laddr", needs: vec![Field::SelfLaddr] },
        Req { name: "I-1.2_block_birth", needs: vec![Field::BlockBirth] },
        Req { name: "I-1.3_tree_id", needs: vec![Field::TreeId] },
        Req { name: "I-1.4_fsid", needs: vec![Field::Fsid] },
        Req { name: "I-2.3_padding_zero", needs: vec![Field::DeclaredLen] },
        Req { name: "D18_3_scan_rebuild", needs: FIVE_TUPLE.to_vec() },
        Req { name: "scrub_watermark", needs: vec![Field::BlockBirth] },
        Req { name: "header_self_integrity", needs: vec![Field::HdrCsum] },
        Req { name: "C74_mount_fsid_check", needs: vec![Field::Fsid] },
        Req { name: "scanner_magic_probe", needs: vec![Field::Magic] },
    ]
}

fn covered(set: &[Field], req: &Req) -> bool {
    req.needs.iter().all(|f| set.contains(f))
}

fn total_bytes(set: &[Field]) -> u64 {
    set.iter().map(|f| f.width()).sum()
}

/// set_b 里不在 I-6.2 白名单六项内的字段——C68 收口要处置的清单。
fn whitelist_conflicts() -> Vec<Field> {
    let wl = set_c_whitelist();
    set_b_repair().into_iter().filter(|f| !wl.contains(f)).collect()
}

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw("name=config model=matrix file_ops=0"));
    let sets: [(&str, Vec<Field>); 3] = [
        ("set_a_settled", set_a_settled()),
        ("set_b_repair", set_b_repair()),
        ("set_c_whitelist", set_c_whitelist()),
    ];
    for (tag, set) in &sets {
        let bytes = total_bytes(set);
        let reqs = requirements();
        let ok: Vec<&str> = reqs.iter().filter(|r| covered(set, r)).map(|r| r.name).collect();
        let missing: Vec<&str> = reqs.iter().filter(|r| !covered(set, r)).map(|r| r.name).collect();
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=set set={tag} bytes={bytes} pct_16k={:.2} pct_32k={:.2} covered={} missing={}",
                100.0 * bytes as f64 / 16384.0,
                100.0 * bytes as f64 / 32768.0,
                ok.join(","),
                if missing.is_empty() { "none".into() } else { missing.join(",") }
            ))
        );
    }
    let conflicts: Vec<&str> = whitelist_conflicts().iter().map(|f| f.tag()).collect();
    println!(
        "{}",
        em.emit_raw(&format!("name=whitelist_conflict fields={}", conflicts.join(",")))
    );
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1（C69 的机检形态）**：set_a 判不了块 birth / fsid / 头自证——
    /// 两条已定不变量与扫描期头完整性在已定字段集下落空。
    #[test]
    fn set_a_leaves_settled_invariants_unjudgeable() {
        let a = set_a_settled();
        let reqs = requirements();
        for name in ["I-1.2_block_birth", "I-1.4_fsid", "header_self_integrity", "scanner_magic_probe"] {
            let r = reqs.iter().find(|r| r.name == name).unwrap();
            assert!(!covered(&a, r), "{name} 不该被 set_a 够到");
        }
        // 够得到的：树 ID、补齐判定、扫描重建
        for name in ["I-1.3_tree_id", "I-2.3_padding_zero", "D18_3_scan_rebuild"] {
            let r = reqs.iter().find(|r| r.name == name).unwrap();
            assert!(covered(&a, r), "{name} 该被 set_a 够到");
        }
    }

    /// **判据 1**：set_c（白名单字面集）判不了扫描重建——五元组一项都没有。
    #[test]
    fn set_c_kills_scan_rebuild() {
        let c = set_c_whitelist();
        let reqs = requirements();
        let r = reqs.iter().find(|r| r.name == "D18_3_scan_rebuild").unwrap();
        assert!(!covered(&c, r));
        let r2 = reqs.iter().find(|r| r.name == "I-1.3_tree_id").unwrap();
        assert!(!covered(&c, r2));
    }

    /// **判据 1**：set_b 除「自身逻辑地址」外全够——那一行三个集都缺，
    /// 因为它在数据单元上指什么本身未收口。
    #[test]
    fn set_b_covers_all_but_self_laddr() {
        let b = set_b_repair();
        for r in requirements() {
            if r.name == "I-1.1_self_laddr" {
                assert!(!covered(&b, &r));
                assert!(!covered(&set_a_settled(), &r));
                assert!(!covered(&set_c_whitelist(), &r));
            } else {
                assert!(covered(&b, &r), "{} 该被 set_b 够到", r.name);
            }
        }
    }

    /// **判据 2 字节账**：set_a = 35（五元组 33 + 声明长度 2）；
    /// set_b = 35 + 4+2+2+16+8+4 = 71（占 16 KiB 的 0.43%、32 KiB 的 0.22%）；
    /// set_c = 4+2+2+4+2+16 = 30。
    #[test]
    fn absolute_byte_accounting() {
        assert_eq!(total_bytes(&set_a_settled()), 35);
        assert_eq!(total_bytes(&set_b_repair()), 71);
        assert_eq!(total_bytes(&set_c_whitelist()), 30);
        assert!((100.0_f64 * 71.0 / 16384.0 - 0.433).abs() < 0.01);
    }

    /// **判据 3**：set_b 与白名单的冲突清单恰为九项（五元组 5 + 声明长度已在白名单的「长度」里？
    /// ——不在：白名单的「长度」按字面就是 DeclaredLen，模型里同一字段 ⇒ 冲突清单不含它）。
    /// 逐个点名：unit_type / tree_id / obj_id / obj_birth / anchor_off / fsid / block_birth / hdr_csum。
    #[test]
    fn whitelist_conflict_list_is_exact() {
        let c: Vec<&str> = whitelist_conflicts().iter().map(|f| f.tag()).collect();
        assert_eq!(
            c,
            vec!["unit_type", "tree_id", "obj_id", "obj_birth", "anchor_off", "fsid", "block_birth", "hdr_csum"]
        );
    }

    /// **部分覆盖不算覆盖**：只有树 ID 的集合够不着扫描重建（五元组要全）。
    /// 三个候选集对这行要么全有要么全无，覆盖语义写错（all→any）在矩阵上不可见，
    /// 只能直接测（2026-09-02 变异测试实测 M1 漏网，补此测）。
    #[test]
    fn partial_coverage_does_not_count() {
        let reqs = requirements();
        let r = reqs.iter().find(|r| r.name == "D18_3_scan_rebuild").unwrap();
        assert!(!covered(&[Field::TreeId], r), "只带树 ID 不许算够到扫描重建");
        assert!(covered(&FIVE_TUPLE, r));
    }

    /// **需求定义自身要钉住**：扫描重建那一行的需求恰为完整五元组——
    /// 需求被悄悄放松（只要树 ID）时矩阵照样全对（2026-09-02 变异测试实测 M5 漏网）。
    #[test]
    fn scan_rebuild_requires_full_five_tuple() {
        let reqs = requirements();
        let r = reqs.iter().find(|r| r.name == "D18_3_scan_rebuild").unwrap();
        assert_eq!(r.needs, FIVE_TUPLE.to_vec());
    }

    /// 五元组宽度假设合计 33（1+8+8+8+8）——D21 那个 55 字节提议值与本模型口径不同
    /// （55 含 magic 等），两个数不许混引，这里钉住本模型自己的口径。
    #[test]
    fn five_tuple_width_assumption() {
        assert_eq!(FIVE_TUPLE.iter().map(|f| f.width()).sum::<u64>(), 33);
    }

    /// 需求行与字段的对应关系抽查：块 birth 与对象 birth 是两个字段，不许混
    /// （C69 的原文逐字：五元组里的「对象出生代」是 inode generation，不是块代）。
    #[test]
    fn block_birth_is_not_obj_birth() {
        assert_ne!(Field::BlockBirth.tag(), Field::ObjBirth.tag());
        let a = set_a_settled();
        assert!(a.contains(&Field::ObjBirth));
        assert!(!a.contains(&Field::BlockBirth));
    }
}
