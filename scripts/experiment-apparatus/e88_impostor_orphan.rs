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
const LIVE_UNIT_COUNT: u64 = 37; // 含 1 个 birth == ROOT_TXG 的边界活单元
const LOST_RECORD_UNIT_COUNT: u64 = 6; // 活着但分配记录丢失
const TORN_ORPHAN_COUNT: u64 = 9; // 撕裂孤儿：birth > ROOT_TXG、无分配记录
const MISDIRECTED_IMPOSTOR_COUNT: u64 = 3; // 错向写冒名：外来 fsid 落在有分配记录的落点上
const NESTED_FOREIGN_IMAGE_COUNT: u64 = 5; // 活单元载荷内嵌外来镜像头（最坏对齐）
const NESTED_SELF_IMAGE_COUNT: u64 = 4; // 活单元载荷内嵌自身备份镜像头（fsid 相同）
const BAD_CHECKSUM_UNIT_COUNT: u64 = 2; // 头部损坏
const DROP_POINT_COUNT: u64 = LIVE_UNIT_COUNT + LOST_RECORD_UNIT_COUNT + TORN_ORPHAN_COUNT + MISDIRECTED_IMPOSTOR_COUNT + BAD_CHECKSUM_UNIT_COUNT; // 57 个落点
const PROBE_POINT_COUNT: u64 = DROP_POINT_COUNT * PROBES_PER_UNIT; // 114 个探针点

/// 一份盘上单元头的最小映像。字段齐全；臂自己决定读不读 fsid / birth。
#[derive(Clone, Copy, PartialEq)]
struct HeaderImage {
    fsid: u64,
    birth: u64,
    is_header_parseable: bool, // magic + 头校验和
}

/// 一个落点：起点必有内容（映像或损坏头），载荷内部探针点可能嵌着镜像头。
#[derive(Clone, Copy)]
struct DropPoint {
    head: HeaderImage,
    allocated: bool,          // 分配记录谓词（D3 已定项 1 逐落点形态；只对起点为真才有意义）
    nested_image_at_payload_probe: Option<HeaderImage>,         // 嵌套镜像头（最坏对齐：恰在探针点上）
}

fn build_disk() -> Vec<DropPoint> {
    let mut disk = Vec::new();
    let make_live_drop_point = |birth| DropPoint {
        head: HeaderImage { fsid: OWN_FSID, birth, is_header_parseable: true },
        allocated: true,
        nested_image_at_payload_probe: None,
    };
    // 活 ×37：第 0 个 birth 恰 == ROOT_TXG（边界）；前 5 个嵌外来镜像、随后 4 个嵌自身镜像
    for live_index in 0..LIVE_UNIT_COUNT {
        let mut live_drop_point = make_live_drop_point(if live_index == 0 { ROOT_TXG } else { 1 + live_index });
        if live_index >= 1 && live_index <= NESTED_FOREIGN_IMAGE_COUNT {
            live_drop_point.nested_image_at_payload_probe = Some(HeaderImage { fsid: FOREIGN_FSID, birth: 7, is_header_parseable: true });
        } else if live_index > NESTED_FOREIGN_IMAGE_COUNT && live_index <= NESTED_FOREIGN_IMAGE_COUNT + NESTED_SELF_IMAGE_COUNT {
            live_drop_point.nested_image_at_payload_probe = Some(HeaderImage { fsid: OWN_FSID, birth: 7, is_header_parseable: true });
        }
        disk.push(live_drop_point);
    }
    // 失账 ×6：分配记录丢失
    for lost_index in 0..LOST_RECORD_UNIT_COUNT {
        disk.push(DropPoint {
            head: HeaderImage { fsid: OWN_FSID, birth: 40 + lost_index, is_header_parseable: true },
            allocated: false,
            nested_image_at_payload_probe: None,
        });
    }
    // 撕裂孤儿 ×9：birth > 根 txg
    for torn_index in 0..TORN_ORPHAN_COUNT {
        disk.push(DropPoint {
            head: HeaderImage { fsid: OWN_FSID, birth: ROOT_TXG + 1 + torn_index, is_header_parseable: true },
            allocated: false,
            nested_image_at_payload_probe: None,
        });
    }
    // 错向写冒名 ×3：外来单元整个盖在有分配记录的落点上
    for misdirect_index in 0..MISDIRECTED_IMPOSTOR_COUNT {
        disk.push(DropPoint {
            head: HeaderImage { fsid: FOREIGN_FSID, birth: 10 + misdirect_index, is_header_parseable: true },
            allocated: true,
            nested_image_at_payload_probe: None,
        });
    }
    // 坏校验和 ×2
    for _ in 0..BAD_CHECKSUM_UNIT_COUNT {
        disk.push(DropPoint {
            head: HeaderImage { fsid: OWN_FSID, birth: 1, is_header_parseable: false },
            allocated: true,
            nested_image_at_payload_probe: None,
        });
    }
    assert_eq!(disk.len() as u64, DROP_POINT_COUNT);
    disk
}

/// 一条臂扫完全盘的分类计数。字段名与 kb 判据一一对应。
#[derive(Default, Debug, PartialEq)]
struct Tally {
    visited: u64,
    skipped_claimed: u64,
    empty: u64,
    rejected_bad_checksum: u64,
    rejected_foreign: u64,
    pruned_torn: u64,
    ingested_level_two: u64,
    ingested_level_one: u64,
    nested_ingested: u64, // ingest_l1/l2 里来自嵌套镜像头的那部分（单列，判据 2/3 用）
}

fn scan(disk: &[DropPoint], has_identity_fields: bool, claim_rule_enabled: bool) -> Tally {
    let mut tally = Tally::default();
    for drop_point in disk {
        // 起点探针
        tally.visited += 1;
        let mut claimed = false;
        if drop_point.head.is_header_parseable {
            claimed = true; // 认领只看「头解析得出」，与身份判定无关
            if has_identity_fields && drop_point.head.fsid != OWN_FSID {
                tally.rejected_foreign += 1;
            } else if has_identity_fields && drop_point.head.birth > ROOT_TXG {
                tally.pruned_torn += 1;
            } else if drop_point.allocated {
                tally.ingested_level_two += 1;
            } else {
                tally.ingested_level_one += 1;
            }
        } else {
            tally.rejected_bad_checksum += 1;
        }
        // 载荷内部探针（每单元 1 个）
        tally.visited += 1;
        if claim_rule_enabled && claimed {
            tally.skipped_claimed += 1;
            continue;
        }
        match drop_point.nested_image_at_payload_probe {
            None => tally.empty += 1,
            Some(nested_image) => {
                if !nested_image.is_header_parseable {
                    tally.rejected_bad_checksum += 1;
                } else if has_identity_fields && nested_image.fsid != OWN_FSID {
                    tally.rejected_foreign += 1;
                } else if has_identity_fields && nested_image.birth > ROOT_TXG {
                    tally.pruned_torn += 1;
                } else {
                    // 探针地址不是落点 ⇒ 分配记录谓词恒假 ⇒ 至多级 1
                    tally.ingested_level_one += 1;
                    tally.nested_ingested += 1;
                }
            }
        }
    }
    tally
}

/// 独立审计：只从夹具常量出发的纯算术，不碰扫描器的任何计数器
/// （evidence-discipline：审计与被审计不许同一段代码）。
fn audit(has_identity_fields: bool, claim_rule_enabled: bool) -> Tally {
    let parseable_unit_count = LIVE_UNIT_COUNT + LOST_RECORD_UNIT_COUNT + TORN_ORPHAN_COUNT + MISDIRECTED_IMPOSTOR_COUNT; // 55
    let nested_image_count = NESTED_FOREIGN_IMAGE_COUNT + NESTED_SELF_IMAGE_COUNT; // 9
    let mut expected_tally = Tally { visited: PROBE_POINT_COUNT, ..Tally::default() };
    expected_tally.rejected_bad_checksum = BAD_CHECKSUM_UNIT_COUNT; // 两种臂、两种认领下都恰拒 2（坏头认领不了，其载荷是 empty）
    if claim_rule_enabled {
        expected_tally.skipped_claimed = parseable_unit_count;
        expected_tally.empty = BAD_CHECKSUM_UNIT_COUNT; // 只剩坏头单元的载荷探针
    } else {
        expected_tally.empty = BAD_CHECKSUM_UNIT_COUNT + (DROP_POINT_COUNT - BAD_CHECKSUM_UNIT_COUNT - nested_image_count); // 无嵌套的载荷 + 坏头载荷
    }
    if has_identity_fields {
        expected_tally.rejected_foreign = MISDIRECTED_IMPOSTOR_COUNT;
        expected_tally.pruned_torn = TORN_ORPHAN_COUNT;
        expected_tally.ingested_level_two = LIVE_UNIT_COUNT;
        expected_tally.ingested_level_one = LOST_RECORD_UNIT_COUNT;
        if !claim_rule_enabled {
            expected_tally.rejected_foreign += NESTED_FOREIGN_IMAGE_COUNT;
            expected_tally.ingested_level_one += NESTED_SELF_IMAGE_COUNT;
            expected_tally.nested_ingested = NESTED_SELF_IMAGE_COUNT;
        }
    } else {
        expected_tally.ingested_level_two = LIVE_UNIT_COUNT + MISDIRECTED_IMPOSTOR_COUNT; // 冒名混进级 2
        expected_tally.ingested_level_one = LOST_RECORD_UNIT_COUNT + TORN_ORPHAN_COUNT; // 撕裂与失账混在级 1
        if !claim_rule_enabled {
            expected_tally.ingested_level_one += nested_image_count;
            expected_tally.nested_ingested = nested_image_count;
        }
    }
    expected_tally
}

/// 错分类数（判翻条款的口径）：冒名进级 2 + 撕裂未清 + 嵌套收进。
fn misclassification_count(tally: &Tally, has_identity_fields: bool) -> u64 {
    let impostors_in_level_two = if has_identity_fields { 0 } else { MISDIRECTED_IMPOSTOR_COUNT };
    let torn_unpruned = TORN_ORPHAN_COUNT - tally.pruned_torn;
    impostors_in_level_two + torn_unpruned + tally.nested_ingested
}

fn main() {
    let disk = build_disk();
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config unit={UNIT} step={STEP} root_txg={ROOT_TXG} drops={DROP_POINT_COUNT} probes={PROBE_POINT_COUNT} \
             live={LIVE_UNIT_COUNT} lost={LOST_RECORD_UNIT_COUNT} torn={TORN_ORPHAN_COUNT} misdirect={MISDIRECTED_IMPOSTOR_COUNT} \
             nest_foreign={NESTED_FOREIGN_IMAGE_COUNT} nest_self={NESTED_SELF_IMAGE_COUNT} badcsum={BAD_CHECKSUM_UNIT_COUNT}"
        ))
    );
    for (label, has_identity_fields, claim_rule_enabled) in [
        ("hdr91_claim", true, true),
        ("hdr91_noclaim", true, false),
        ("hdr47_claim", false, true),
        ("hdr47_noclaim", false, false),
    ] {
        let scan_tally = scan(&disk, has_identity_fields, claim_rule_enabled);
        let audit_tally = audit(has_identity_fields, claim_rule_enabled);
        assert_eq!(scan_tally, audit_tally, "{label}: 扫描与独立审计对不上");
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name={label} visited={} skipped={} empty={} csum={} foreign={} pruned={} \
                 l2={} l1={} nested_in={} misclass={}",
                scan_tally.visited,
                scan_tally.skipped_claimed,
                scan_tally.empty,
                scan_tally.rejected_bad_checksum,
                scan_tally.rejected_foreign,
                scan_tally.pruned_torn,
                scan_tally.ingested_level_two,
                scan_tally.ingested_level_one,
                scan_tally.nested_ingested,
                misclassification_count(&scan_tally, has_identity_fields),
            ))
        );
    }
    println!("{}", emitter.finish());
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
        assert_eq!(DROP_POINT_COUNT, 57);
        assert_eq!(PROBE_POINT_COUNT, 114);
        assert_eq!(LIVE_UNIT_COUNT + LOST_RECORD_UNIT_COUNT + TORN_ORPHAN_COUNT + MISDIRECTED_IMPOSTOR_COUNT + BAD_CHECKSUM_UNIT_COUNT, DROP_POINT_COUNT);
    }

    /// **判据 1：定案形态（hdr91 + 认领）必须全对，逐格钉绝对值。**
    #[test]
    fn settled_form_classifies_everything_exactly() {
        let tally = scan(&build_disk(), true, true);
        assert_eq!(tally.ingested_level_two, 37);
        assert_eq!(tally.ingested_level_one, 6);
        assert_eq!(tally.pruned_torn, 9);
        assert_eq!(tally.rejected_foreign, 3);
        assert_eq!(tally.rejected_bad_checksum, 2);
        assert_eq!(tally.nested_ingested, 0);
        assert_eq!(misclassification_count(&tally, true), 0);
    }

    /// **判据 2（判翻条款）：47 字节头的错分类不为 0 ⇒ 骑手不翻回。**
    /// 若这条测试因 misclass == 0 而红，按 kb 判据 2 当场重开 D18 已定项 7 的 fsid / birth。
    #[test]
    fn short_header_misclassifies_and_the_count_is_pinned() {
        let short_header_claim_tally = scan(&build_disk(), false, true);
        assert_eq!(misclassification_count(&short_header_claim_tally, false), 12, "冒名 3 + 撕裂未清 9");
        let short_header_no_claim_tally = scan(&build_disk(), false, false);
        assert_eq!(misclassification_count(&short_header_no_claim_tally, false), 21, "再加嵌套 9");
        assert_eq!(short_header_claim_tally.pruned_torn, 0, "没有诞生代号就没有任何单元能被正类判垃圾");
        assert_eq!(short_header_claim_tally.ingested_level_two, 40, "冒名 3 个混进级 2——双证见证被伪造");
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
        let full_header_tally = scan(&build_disk(), true, true);
        assert_eq!((full_header_tally.pruned_torn, full_header_tally.ingested_level_one), (9, 6));
        let short_header_tally = scan(&build_disk(), false, true);
        assert_eq!((short_header_tally.pruned_torn, short_header_tally.ingested_level_one), (0, 15), "撕裂垃圾与失账活单元混在级 1");
    }

    /// **判据 4 后半 + 判据 5：每臂守恒到探针总数，总数由独立算术给出。**
    #[test]
    fn every_arm_conserves_to_the_probe_total() {
        for (has_identity_fields, claim_rule_enabled) in [(true, true), (true, false), (false, true), (false, false)] {
            let tally = scan(&build_disk(), has_identity_fields, claim_rule_enabled);
            let sum = tally.skipped_claimed
                + tally.empty
                + tally.rejected_bad_checksum
                + tally.rejected_foreign
                + tally.pruned_torn
                + tally.ingested_level_two
                + tally.ingested_level_one;
            assert_eq!(sum, 114, "has_id={has_identity_fields} claim={claim_rule_enabled}");
            assert_eq!(tally.visited, 114);
            assert_eq!(tally.rejected_bad_checksum, 2, "坏校验和在每条臂都恰拒 2");
        }
    }

    /// 扫描与独立审计逐臂相等——审计只用夹具常量算，没碰扫描器计数器。
    #[test]
    fn audit_agrees_with_scan_on_every_arm() {
        for (has_identity_fields, claim_rule_enabled) in [(true, true), (true, false), (false, true), (false, false)] {
            assert_eq!(scan(&build_disk(), has_identity_fields, claim_rule_enabled), audit(has_identity_fields, claim_rule_enabled));
        }
    }

    /// 边界：birth 恰 == 根 txg 的活单元必须收进（> 是严格比较）。
    /// 这颗夹具专为「> 改 >=」那类变异而种。
    #[test]
    fn birth_equal_to_root_txg_is_live_not_torn() {
        let disk = build_disk();
        assert_eq!(disk[0].head.birth, ROOT_TXG);
        let tally = scan(&disk, true, true);
        assert_eq!(tally.pruned_torn, 9, "边界单元没被误清");
        assert_eq!(tally.ingested_level_two, 37, "边界单元收进了级 2");
    }

    /// 认领的语义：解析得出就认领，与身份判定无关——
    /// 冒名单元（hdr91 拒收）也认领，其载荷探针被跳过；坏头认领不了，其载荷被探到。
    #[test]
    fn claim_follows_parse_not_identity() {
        let tally = scan(&build_disk(), true, true);
        assert_eq!(tally.skipped_claimed, 55, "55 个解析得出的头都认领了");
        assert_eq!(tally.empty, 2, "只剩 2 个坏头单元的载荷探针");
    }

    /// 嵌套镜像头在无认领臂里至多进级 1：探针地址不是落点，分配记录谓词恒假。
    #[test]
    fn nested_images_never_reach_level_two() {
        let tally = scan(&build_disk(), false, false);
        assert_eq!(tally.ingested_level_two, 40, "级 2 只可能来自起点探针");
        assert_eq!(tally.nested_ingested, 9);
        assert_eq!(tally.ingested_level_one, 15 + 9);
    }
}
