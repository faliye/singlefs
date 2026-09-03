//! E88 冒名单元与孤儿单元的判别 —— C68 对抗轮反推腿开出的证伪检查（C89 第 ③ 笔）。
//!
//! 反推腿说服判决把 fsid 与诞生代号翻回单元头（D18 已定项 7），并写明证伪条款：
//! **若 47 字节头（无 fsid / 诞生代号）对这些夹具也全判对，两条骑手翻回。**
//! 计数模型把四条臂 × 七类夹具全部跑一遍，逐格钉绝对值。
//!
//! ## 臂
//!
//! | 臂 | 头字段 | 认领规则（认领后跳过载荷内部探针） |
//! |---|---|---|
//! | hdr91_claim   | 含 fsid + 诞生代号 | 开 |
//! | hdr91_noclaim | 含 fsid + 诞生代号 | 关 |
//! | hdr47_claim   | 无 fsid、无诞生代号 | 开 |
//! | hdr47_noclaim | 无 fsid、无诞生代号 | 关 |
//!
//! ## 夹具（数量跑前写死，见常量区）
//!
//! 活 ×37（其中 1 个诞生代号恰 == 根 txg，钉住 > 与 >= 的边界；9 个的载荷内嵌镜像头）、
//! 失账 ×6、撕裂孤儿 ×9、错向写冒名 ×3、嵌套外来 ×5、嵌套自身 ×4、坏校验和 ×2。
//! 嵌套头**故意**放在探针点上（最坏对齐）——防线不许押在「一般不会对齐」上。
//!
//! ## 判据（跑前写死，与 kb/experiments/88-冒名单元与孤儿单元的判别.md 同一份）
//!
//! 1. hdr91_claim 全对：级 2 恰 37、级 1 恰 6、清恰 9、冒名拒收恰 3、嵌套收进恰 0、坏校验和拒 2。
//! 2. 判翻条款：hdr47_claim 错分类（冒名进级 2 + 撕裂未清 + 嵌套收进）为 0 ⇒ 骑手翻回。
//! 3. 认领判别力：hdr91_noclaim 上自身镜像恰收进 4（fsid 挡不住自己人）；hdr91_claim 上恰 0。
//! 4. 撕裂被清数与失账进级 1 数分开报，臂内总和 ≠ 探针总数 ⇒ 扫描器没走完，整轮作废。
//! 5. 阳性对照每臂跑：坏校验和恰拒 2；探针访问总数恰 == 独立算术值。

use e7_index_bench::Emitter;

// ── 几何（出处：D18 已定项 9 修正后口径 —— 步进 16384；数据单元 32768）──
const UNIT: u64 = 32768;
const STEP: u64 = 16384;
const PROBES_PER_UNIT: u64 = UNIT / STEP; // 2：单元起点 + 载荷内部 1 个

// ── 卷身份与根状态 ──
const OWN_FSID: u64 = 0xA11CE;
const FOREIGN_FSID: u64 = 0xF0E1;
const ROOT_TXG: u64 = 100;

// ── 夹具数量（跑前写死）──
const N_LIVE: u64 = 37; // 含 1 个 birth == ROOT_TXG 的边界活单元
const N_LOST: u64 = 6; // 活着但分配记录丢失
const N_TORN: u64 = 9; // 撕裂孤儿：birth > ROOT_TXG、无分配记录
const N_MISDIRECT: u64 = 3; // 错向写冒名：外来 fsid 落在有分配记录的落点上
const N_NEST_FOREIGN: u64 = 5; // 活单元载荷内嵌外来镜像头（最坏对齐）
const N_NEST_SELF: u64 = 4; // 活单元载荷内嵌自身备份镜像头（fsid 相同）
const N_BADCSUM: u64 = 2; // 头部损坏
const N_DROP: u64 = N_LIVE + N_LOST + N_TORN + N_MISDIRECT + N_BADCSUM; // 57 个落点
const N_PROBES: u64 = N_DROP * PROBES_PER_UNIT; // 114 个探针点

/// 一份盘上单元头的最小映像。字段齐全；臂自己决定读不读 fsid / birth。
#[derive(Clone, Copy, PartialEq)]
struct Img {
    fsid: u64,
    birth: u64,
    parse_ok: bool, // magic + 头校验和
}

/// 一个落点：起点必有内容（映像或损坏头），载荷内部探针点可能嵌着镜像头。
#[derive(Clone, Copy)]
struct Drop {
    head: Img,
    allocated: bool,          // 分配记录谓词（D3 已定项 1 逐落点形态；只对起点为真才有意义）
    mid: Option<Img>,         // 嵌套镜像头（最坏对齐：恰在探针点上）
}

fn build_disk() -> Vec<Drop> {
    let mut disk = Vec::new();
    let live = |birth| Drop {
        head: Img { fsid: OWN_FSID, birth, parse_ok: true },
        allocated: true,
        mid: None,
    };
    // 活 ×37：第 0 个 birth 恰 == ROOT_TXG（边界）；前 5 个嵌外来镜像、随后 4 个嵌自身镜像
    for i in 0..N_LIVE {
        let mut d = live(if i == 0 { ROOT_TXG } else { 1 + i });
        if i >= 1 && i <= N_NEST_FOREIGN {
            d.mid = Some(Img { fsid: FOREIGN_FSID, birth: 7, parse_ok: true });
        } else if i > N_NEST_FOREIGN && i <= N_NEST_FOREIGN + N_NEST_SELF {
            d.mid = Some(Img { fsid: OWN_FSID, birth: 7, parse_ok: true });
        }
        disk.push(d);
    }
    // 失账 ×6：分配记录丢失
    for i in 0..N_LOST {
        disk.push(Drop {
            head: Img { fsid: OWN_FSID, birth: 40 + i, parse_ok: true },
            allocated: false,
            mid: None,
        });
    }
    // 撕裂孤儿 ×9：birth > 根 txg
    for i in 0..N_TORN {
        disk.push(Drop {
            head: Img { fsid: OWN_FSID, birth: ROOT_TXG + 1 + i, parse_ok: true },
            allocated: false,
            mid: None,
        });
    }
    // 错向写冒名 ×3：外来单元整个盖在有分配记录的落点上
    for i in 0..N_MISDIRECT {
        disk.push(Drop {
            head: Img { fsid: FOREIGN_FSID, birth: 10 + i, parse_ok: true },
            allocated: true,
            mid: None,
        });
    }
    // 坏校验和 ×2
    for _ in 0..N_BADCSUM {
        disk.push(Drop {
            head: Img { fsid: OWN_FSID, birth: 1, parse_ok: false },
            allocated: true,
            mid: None,
        });
    }
    assert_eq!(disk.len() as u64, N_DROP);
    disk
}

/// 一条臂扫完全盘的分类计数。字段名与 kb 判据一一对应。
#[derive(Default, Debug, PartialEq)]
struct Tally {
    visited: u64,
    skipped_claimed: u64,
    empty: u64,
    rejected_csum: u64,
    rejected_foreign: u64,
    pruned_torn: u64,
    ingest_l2: u64,
    ingest_l1: u64,
    nested_ingested: u64, // ingest_l1/l2 里来自嵌套镜像头的那部分（单列，判据 2/3 用）
}

fn scan(disk: &[Drop], has_id: bool, claim: bool) -> Tally {
    let mut t = Tally::default();
    for d in disk {
        // 起点探针
        t.visited += 1;
        let mut claimed = false;
        if d.head.parse_ok {
            claimed = true; // 认领只看「头解析得出」，与身份判定无关
            if has_id && d.head.fsid != OWN_FSID {
                t.rejected_foreign += 1;
            } else if has_id && d.head.birth > ROOT_TXG {
                t.pruned_torn += 1;
            } else if d.allocated {
                t.ingest_l2 += 1;
            } else {
                t.ingest_l1 += 1;
            }
        } else {
            t.rejected_csum += 1;
        }
        // 载荷内部探针（每单元 1 个）
        t.visited += 1;
        if claim && claimed {
            t.skipped_claimed += 1;
            continue;
        }
        match d.mid {
            None => t.empty += 1,
            Some(m) => {
                if !m.parse_ok {
                    t.rejected_csum += 1;
                } else if has_id && m.fsid != OWN_FSID {
                    t.rejected_foreign += 1;
                } else if has_id && m.birth > ROOT_TXG {
                    t.pruned_torn += 1;
                } else {
                    // 探针地址不是落点 ⇒ 分配记录谓词恒假 ⇒ 至多级 1
                    t.ingest_l1 += 1;
                    t.nested_ingested += 1;
                }
            }
        }
    }
    t
}

/// 独立审计：只从夹具常量出发的纯算术，不碰扫描器的任何计数器
/// （evidence-discipline：审计与被审计不许同一段代码）。
fn audit(has_id: bool, claim: bool) -> Tally {
    let parse_ok_units = N_LIVE + N_LOST + N_TORN + N_MISDIRECT; // 55
    let nested = N_NEST_FOREIGN + N_NEST_SELF; // 9
    let mut a = Tally { visited: N_PROBES, ..Tally::default() };
    a.rejected_csum = N_BADCSUM; // 两种臂、两种认领下都恰拒 2（坏头认领不了，其载荷是 empty）
    if claim {
        a.skipped_claimed = parse_ok_units;
        a.empty = N_BADCSUM; // 只剩坏头单元的载荷探针
    } else {
        a.empty = N_BADCSUM + (N_DROP - N_BADCSUM - nested); // 无嵌套的载荷 + 坏头载荷
    }
    if has_id {
        a.rejected_foreign = N_MISDIRECT;
        a.pruned_torn = N_TORN;
        a.ingest_l2 = N_LIVE;
        a.ingest_l1 = N_LOST;
        if !claim {
            a.rejected_foreign += N_NEST_FOREIGN;
            a.ingest_l1 += N_NEST_SELF;
            a.nested_ingested = N_NEST_SELF;
        }
    } else {
        a.ingest_l2 = N_LIVE + N_MISDIRECT; // 冒名混进级 2
        a.ingest_l1 = N_LOST + N_TORN; // 撕裂与失账混在级 1
        if !claim {
            a.ingest_l1 += nested;
            a.nested_ingested = nested;
        }
    }
    a
}

/// 错分类数（判翻条款的口径）：冒名进级 2 + 撕裂未清 + 嵌套收进。
fn misclass(t: &Tally, has_id: bool) -> u64 {
    let impostor_l2 = if has_id { 0 } else { N_MISDIRECT };
    let torn_unpruned = N_TORN - t.pruned_torn;
    impostor_l2 + torn_unpruned + t.nested_ingested
}

fn main() {
    let disk = build_disk();
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config unit={UNIT} step={STEP} root_txg={ROOT_TXG} drops={N_DROP} probes={N_PROBES} \
             live={N_LIVE} lost={N_LOST} torn={N_TORN} misdirect={N_MISDIRECT} \
             nest_foreign={N_NEST_FOREIGN} nest_self={N_NEST_SELF} badcsum={N_BADCSUM}"
        ))
    );
    for (label, has_id, claim) in [
        ("hdr91_claim", true, true),
        ("hdr91_noclaim", true, false),
        ("hdr47_claim", false, true),
        ("hdr47_noclaim", false, false),
    ] {
        let t = scan(&disk, has_id, claim);
        let a = audit(has_id, claim);
        assert_eq!(t, a, "{label}: 扫描与独立审计对不上");
        println!(
            "{}",
            em.emit_raw(&format!(
                "name={label} visited={} skipped={} empty={} csum={} foreign={} pruned={} \
                 l2={} l1={} nested_in={} misclass={}",
                t.visited,
                t.skipped_claimed,
                t.empty,
                t.rejected_csum,
                t.rejected_foreign,
                t.pruned_torn,
                t.ingest_l2,
                t.ingest_l1,
                t.nested_ingested,
                misclass(&t, has_id),
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 夹具与几何常量钉死——改任何一个，判据的绝对值全体失效。
    #[test]
    fn constants_are_pinned() {
        assert_eq!(UNIT, 32768);
        assert_eq!(STEP, 16384);
        assert_eq!(PROBES_PER_UNIT, 2);
        assert_eq!(ROOT_TXG, 100);
        assert_eq!(N_DROP, 57);
        assert_eq!(N_PROBES, 114);
        assert_eq!(N_LIVE + N_LOST + N_TORN + N_MISDIRECT + N_BADCSUM, N_DROP);
    }

    /// **判据 1：定案形态（hdr91 + 认领）必须全对，逐格钉绝对值。**
    #[test]
    fn settled_form_classifies_everything_exactly() {
        let t = scan(&build_disk(), true, true);
        assert_eq!(t.ingest_l2, 37);
        assert_eq!(t.ingest_l1, 6);
        assert_eq!(t.pruned_torn, 9);
        assert_eq!(t.rejected_foreign, 3);
        assert_eq!(t.rejected_csum, 2);
        assert_eq!(t.nested_ingested, 0);
        assert_eq!(misclass(&t, true), 0);
    }

    /// **判据 2（判翻条款）：47 字节头的错分类不为 0 ⇒ 骑手不翻回。**
    /// 若这条测试因 misclass == 0 而红，按 kb 判据 2 当场重开 D18 已定项 7 的 fsid / birth。
    #[test]
    fn short_header_misclassifies_and_the_count_is_pinned() {
        let tc = scan(&build_disk(), false, true);
        assert_eq!(misclass(&tc, false), 12, "冒名 3 + 撕裂未清 9");
        let tn = scan(&build_disk(), false, false);
        assert_eq!(misclass(&tn, false), 21, "再加嵌套 9");
        assert_eq!(tc.pruned_torn, 0, "没有诞生代号就没有任何单元能被正类判垃圾");
        assert_eq!(tc.ingest_l2, 40, "冒名 3 个混进级 2——双证见证被伪造");
    }

    /// **判据 3：认领规则是自身镜像那一格唯一的防线。**
    #[test]
    fn claim_rule_is_the_only_defense_against_self_images() {
        let no_claim = scan(&build_disk(), true, false);
        assert_eq!(no_claim.nested_ingested, 4, "fsid 相同，挡不住自己人");
        assert_eq!(no_claim.rejected_foreign, 3 + 5, "外来嵌套靠 fsid 还能拒");
        let with_claim = scan(&build_disk(), true, true);
        assert_eq!(with_claim.nested_ingested, 0);
    }

    /// **判据 4：撕裂与失账要分得开（hdr91），且守恒。**
    #[test]
    fn torn_and_lost_are_separable_only_with_birth() {
        let t91 = scan(&build_disk(), true, true);
        assert_eq!((t91.pruned_torn, t91.ingest_l1), (9, 6));
        let t47 = scan(&build_disk(), false, true);
        assert_eq!((t47.pruned_torn, t47.ingest_l1), (0, 15), "撕裂垃圾与失账活单元混在级 1");
    }

    /// **判据 4 后半 + 判据 5：每臂守恒到探针总数，总数由独立算术给出。**
    #[test]
    fn every_arm_conserves_to_the_probe_total() {
        for (has_id, claim) in [(true, true), (true, false), (false, true), (false, false)] {
            let t = scan(&build_disk(), has_id, claim);
            let sum = t.skipped_claimed
                + t.empty
                + t.rejected_csum
                + t.rejected_foreign
                + t.pruned_torn
                + t.ingest_l2
                + t.ingest_l1;
            assert_eq!(sum, 114, "has_id={has_id} claim={claim}");
            assert_eq!(t.visited, 114);
            assert_eq!(t.rejected_csum, 2, "坏校验和在每条臂都恰拒 2");
        }
    }

    /// 扫描与独立审计逐臂相等——审计只用夹具常量算，没碰扫描器计数器。
    #[test]
    fn audit_agrees_with_scan_on_every_arm() {
        for (has_id, claim) in [(true, true), (true, false), (false, true), (false, false)] {
            assert_eq!(scan(&build_disk(), has_id, claim), audit(has_id, claim));
        }
    }

    /// 边界：birth 恰 == 根 txg 的活单元必须收进（> 是严格比较）。
    /// 这颗夹具专为「> 改 >=」那类变异而种。
    #[test]
    fn birth_equal_to_root_txg_is_live_not_torn() {
        let disk = build_disk();
        assert_eq!(disk[0].head.birth, ROOT_TXG);
        let t = scan(&disk, true, true);
        assert_eq!(t.pruned_torn, 9, "边界单元没被误清");
        assert_eq!(t.ingest_l2, 37, "边界单元收进了级 2");
    }

    /// 认领的语义：解析得出就认领，与身份判定无关——
    /// 冒名单元（hdr91 拒收）也认领，其载荷探针被跳过；坏头认领不了，其载荷被探到。
    #[test]
    fn claim_follows_parse_not_identity() {
        let t = scan(&build_disk(), true, true);
        assert_eq!(t.skipped_claimed, 55, "55 个解析得出的头都认领了");
        assert_eq!(t.empty, 2, "只剩 2 个坏头单元的载荷探针");
    }

    /// 嵌套镜像头在无认领臂里至多进级 1：探针地址不是落点，分配记录谓词恒假。
    #[test]
    fn nested_images_never_reach_level_two() {
        let t = scan(&build_disk(), false, false);
        assert_eq!(t.ingest_l2, 40, "级 2 只可能来自起点探针");
        assert_eq!(t.nested_ingested, 9);
        assert_eq!(t.ingest_l1, 15 + 9);
    }
}
