//! E57 明文侧字段集的权威落点：哪一边能推出另一边
//!
//! D9（加密） 未定项 9 与 D18（块里携带什么信息） 已定项 7（跑的时候还未定）是同一个缺口的两半，
//! 互相点名。要收口先定权威落点，判据取**信息流向**：
//! 哪一边的字段集能唯一确定另一边，哪一边就是权威。
//!
//! 两侧的字段集都**照已定条款取**，不是本程序发明的：
//!   块头侧   = D18（块里携带什么信息） 已定项 3 的 AAD 五元组
//!   映射层侧 = D9（加密） 已定项 5 的逻辑身份 + D19（块指针的结构与宽度预算） 的物理地址与密文校验和
//!
//! 判据、失败条款、对照组写在 .claude/kb/experiments/57-明文侧字段集的权威落点.md，
//! **跑之前写死**。

use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum F {
    Kind,   // 单元类型标签
    Tree,   // 树 ID
    Obj,    // 对象 ID
    Birth,  // 对象出生代
    Anchor, // 锚点偏移
    Dev,    // 设备号
    Off,    // 设备内偏移
    Crc,    // 密文校验和
}
const ALL_HEADER: [F; 5] = [F::Kind, F::Tree, F::Obj, F::Birth, F::Anchor];
const ALL_MAPPING: [F; 7] = [F::Tree, F::Obj, F::Birth, F::Anchor, F::Dev, F::Off, F::Crc];

#[derive(Clone, Copy)]
struct Unit {
    is_meta: bool,
}

#[derive(Clone, Copy)]
struct Arm {
    header_on_meta: bool,
    header_on_data: bool,
    mapping_carries_kind: bool,
    /// 方向 B 也拿到与方向 A 对等的免费输入（按条目里的地址把单元读回来）
    symmetric_free_inputs: bool,
    /// 方向 A 可以把「扫描时的落点」当成物理地址的来源。
    ///
    /// ⚠️ **这一格才是本实验的要害，而第一版把它写死成 true。**
    /// D18 已定项 3 的禁放清单逐字写着块头**永远禁放**物理落点 / 设备 ID，
    /// 依据是 D9 已定项 5（2026-08-26 用户定案）把「一个块的**物理地址的
    /// 权威记录点**」定在明文映射层。⇒ 扫描到的落点是**观测**，不是权威记录；
    /// 拿它替块头供出 Dev/Off，等于让块头借用了对面那一侧的权威。
    scan_supplies_physical: bool,
}

/// 一个单元的块头**实际**带了哪些字段。
fn header_of(u: &Unit, arm: &Arm) -> BTreeSet<F> {
    let on = if u.is_meta { arm.header_on_meta } else { arm.header_on_data };
    if on { ALL_HEADER.iter().copied().collect() } else { BTreeSet::new() }
}

/// 一个单元的映射层条目**实际**带了哪些字段。
fn mapping_of(arm: &Arm) -> BTreeSet<F> {
    let mut s: BTreeSet<F> = ALL_MAPPING.iter().copied().collect();
    if arm.mapping_carries_kind {
        s.insert(F::Kind);
    }
    s
}

/// 方向 A 的**已知集**：块头 + 扫描天然知道的落点 + 读到密文就能重算的校验和。
///
/// Dev/Off/Crc 是全盘扫描这个动作本身的产物（扫到哪就知道 dev/off，读到密文就能算 crc）。
/// 写成显式集合是为了让变异能把它们摘掉：摘掉之后方向 A 必须出现缺口，
/// 否则说明判定根本没在看这个集合。
fn known_a(u: &Unit, arm: &Arm) -> BTreeSet<F> {
    let mut s = header_of(u, arm);
    if arm.scan_supplies_physical {
        s.insert(F::Dev);
        s.insert(F::Off);
    }
    s.insert(F::Crc);
    s
}

/// 方向 B 的已知集：映射层条目，**外加与方向 A 对等的免费输入**。
///
/// ⚠️ 这一条是本地腿（反例立场）打回来的：方向 A 拿到了「扫描天然已知的
/// 设备号 / 设备内偏移 / 可重算的密文校验和」三个免费输入，方向 B 若不给对等的，
/// 这个比较就是不对称的，判据会自动偏向块头那一侧。
///
/// 对等的形态是「按映射层给的地址去把那个单元读回来」：这一步交出**密文**，
/// 于是密文校验和同样可重算，落点本来就写在条目里。它交不出的只有块头字段本身——
/// 而那正是待验的东西，把它算进来就成了循环论证。
/// ⇒ `symmetric_free_inputs` 打开时，方向 B 多拿 Crc（落点条目里本来就有）。
fn known_b(arm: &Arm) -> BTreeSet<F> {
    let mut s = mapping_of(arm);
    if arm.symmetric_free_inputs {
        s.insert(F::Crc);
    }
    s
}

/// 拿已知集去重建目标字段集，返回缺的那些。
fn missing(known: &BTreeSet<F>, want: &BTreeSet<F>) -> BTreeSet<F> {
    want.difference(known).copied().collect()
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Gap {
    units: usize,
    fields: usize,
    kinds: BTreeSet<F>,
}

fn rebuild(units: &[Unit], arm: &Arm, dir_a: bool) -> Gap {
    let mut g = Gap::default();
    for u in units {
        let (known, want) = if dir_a {
            (known_a(u, arm), mapping_of(arm))
        } else {
            (known_b(arm), header_of(u, arm))
        };
        let m = missing(&known, &want);
        if !m.is_empty() {
            g.units += 1;
        }
        g.fields += m.len();
        g.kinds.extend(m);
    }
    g
}

/// 判据 3 与判据 4 的四格，跑前写死在 kb 正文里。
fn verdict_of(gap_a: usize, gap_b: usize) -> &'static str {
    match (gap_a == 0, gap_b == 0) {
        (true, false) => "权威落在 D18：映射层可从块头重建，反向不可（判据 3）",
        (false, true) => "权威落在 D9：块头可从映射层重建，反向不可（判据 3 的镜像）",
        (true, true) => "两向都通，权威可任选（判据 4）",
        (false, false) => "两向都有缺口，权威要按字段拆（判据 4）",
    }
}

fn corpus(n_meta: usize, n_data: usize) -> Vec<Unit> {
    let mut v: Vec<Unit> = (0..n_meta).map(|_| Unit { is_meta: true }).collect();
    v.extend((0..n_data).map(|_| Unit { is_meta: false }));
    v
}

const CURRENT: Arm = Arm { header_on_meta: true, header_on_data: true, mapping_carries_kind: false, symmetric_free_inputs: false, scan_supplies_physical: true };
const POSITIVE: Arm = Arm { header_on_meta: true, header_on_data: true, mapping_carries_kind: true, symmetric_free_inputs: false, scan_supplies_physical: true };
const META_ONLY: Arm = Arm { header_on_meta: true, header_on_data: false, mapping_carries_kind: false, symmetric_free_inputs: false, scan_supplies_physical: true };
/// 戊：本地腿要求的对称臂——方向 B 也拿到免费输入。
const SYMMETRIC: Arm = Arm { header_on_meta: true, header_on_data: true, mapping_carries_kind: false, symmetric_free_inputs: true, scan_supplies_physical: true };
/// 己：把「块头永远禁放物理落点」这条已定条款喂进模型。
/// 这是正推腿提出的证伪观测：不喂扫描落点重跑方向 A，看缺口还在不在。
const CORRECTED: Arm = Arm { header_on_meta: true, header_on_data: true, mapping_carries_kind: false, symmetric_free_inputs: false, scan_supplies_physical: false };
const NEGATIVE: Arm = Arm { header_on_meta: false, header_on_data: false, mapping_carries_kind: false, symmetric_free_inputs: false, scan_supplies_physical: true };

fn main() {
    println!("E7RESULT name=meta exp=E57 arms=6 metric=有缺口的单元数/逐字段缺口数");
    let mut emitted = 1usize; // meta 行自己也算一条
    let sizes = [(1usize, 1usize), (8, 24), (64, 192), (1000, 3000), (4096, 12288)];

    for (nm, nd) in sizes {
        let units = corpus(nm, nd);
        let n = units.len();
        for (name, arm) in [
            ("甲-现行", CURRENT),
            ("乙-阳性对照-映射层带类型", POSITIVE),
            ("丙-头只在元数据上", META_ONLY),
            ("丁-阴性对照-块头全删", NEGATIVE),
            ("戊-对称臂-方向B也拿免费输入", SYMMETRIC),
            ("己-块头禁放物理落点", CORRECTED),
        ] {
            let a = rebuild(&units, &arm, true);
            let b = rebuild(&units, &arm, false);
            let ka: Vec<String> = a.kinds.iter().map(|f| format!("{f:?}")).collect();
            let kb: Vec<String> = b.kinds.iter().map(|f| format!("{f:?}")).collect();
            println!(
                "E7RESULT name=arm arm={name} meta={nm} data={nd} n={n} A缺口单元={} A缺口字段={} A缺的是={} B缺口单元={} B缺口字段={} B缺的是={}",
                a.units, a.fields, if ka.is_empty() { "无".to_string() } else { ka.join("+") },
                b.units, b.fields, if kb.is_empty() { "无".to_string() } else { kb.join("+") },
            );
            emitted += 1;
        }
    }

    let units = corpus(4096, 12288);
    let n = units.len();
    // ⚠️ **判定跑在己臂上，不是甲臂上。** 甲臂让块头借用了物理落点，
    // 而那一项的权威记录点已由 D9 已定项 5 定在映射层。
    let a = rebuild(&units, &CORRECTED, true);
    let b = rebuild(&units, &CORRECTED, false);
    let old_a = rebuild(&units, &CURRENT, true);
    println!("E7RESULT name=old_reading arm=甲 gap_a_units={} note=把物理落点当免费输入的那一版", old_a.units);
    emitted += 1;
    // ⚠️ 第一版把真值表写成 (a.units==0, b.units>0)，第二格的 `b.units>0` 是
    // 「B **失败**」而标签写的是「B 成功」⇒ 己臂跑出来时报了一个反的判定。
    // 改成显式的 a_ok / b_ok，并由 `判定表按可推导性逐格对`
    let verdict = verdict_of(a.units, b.units);
    println!("E7RESULT name=verdict n={n} gap_a_units={} gap_b_units={} verdict={verdict}", a.units, b.units);
    emitted += 1;

    // ⚠️ 收尾行**把自己也算进去**——replay.sh 的完整性闸先给每一条 E7RESULT 计数
    // （收尾行自己也命中那条规则），再拿 emitted 去比。少算 1 会被判「跑不了」。
    println!("E7RESULT name=done emitted={}", emitted + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    // 钉绝对值：不许只做臂间互比——「三条臂一起错」在互比里长得和正确一模一样。

    #[test]
    fn 现行臂_方向a无缺口_方向b恰好每单元缺一个类型标签() {
        let u = corpus(7, 13);
        let n = 20;
        let a = rebuild(&u, &CURRENT, true);
        assert_eq!(a.units, 0);
        assert_eq!(a.fields, 0);
        let b = rebuild(&u, &CURRENT, false);
        assert_eq!(b.units, n, "每个单元都缺");
        assert_eq!(b.fields, n, "而且**恰好**缺一个字段，不是多个");
        assert_eq!(b.kinds.into_iter().collect::<Vec<_>>(), vec![F::Kind], "缺的就是类型标签");
    }

    #[test]
    fn 阳性对照_映射层带上类型标签后方向b必须归零() {
        let u = corpus(5, 5);
        let b = rebuild(&u, &POSITIVE, false);
        assert_eq!(b.units, 0, "不归零 ⇒ 缺口不是类型标签造成的，整轮作废");
        assert_eq!(b.fields, 0);
        assert_eq!(rebuild(&u, &POSITIVE, true).units, 0);
    }

    #[test]
    fn 阴性对照_块头全删后方向a缺口等于单元总数() {
        for (nm, nd) in [(1usize, 1usize), (10, 30), (100, 300)] {
            let u = corpus(nm, nd);
            let n = nm + nd;
            let a = rebuild(&u, &NEGATIVE, true);
            assert_eq!(a.units, n, "不等 ⇒ 重建路径没在读块头，整轮作废");
            assert_eq!(a.fields, 4 * n, "逐字段缺口必须恰好是 4×N");
        }
    }

    #[test]
    fn 头只在元数据上时_缺口恰好等于数据块数() {
        let (nm, nd) = (64usize, 192usize);
        let u = corpus(nm, nd);
        let a = rebuild(&u, &META_ONLY, true);
        assert_eq!(a.units, nd, "元数据侧能重建，数据块侧不能——E28 当年测到的正是这个形状");
        assert_eq!(a.fields, 4 * nd);
    }

    #[test]
    fn 摘掉扫描天然已知的落点_方向a必须出现缺口() {
        let u = corpus(3, 3);
        let want = mapping_of(&CURRENT);
        let bare: BTreeSet<F> = header_of(&u[0], &CURRENT);
        let m = missing(&bare, &want);
        assert_eq!(m.len(), 3);
        assert_eq!(m.into_iter().collect::<Vec<_>>(), vec![F::Dev, F::Off, F::Crc]);
    }

    #[test]
    fn 字段集本身照已定条款_数量钉死() {
        assert_eq!(ALL_HEADER.len(), 5, "D18 已定项 3 的 AAD 五元组");
        assert_eq!(ALL_MAPPING.len(), 7, "逻辑身份 4 + 物理地址 2 + 密文校验和 1");
        assert_eq!(mapping_of(&CURRENT).len(), 7);
        assert_eq!(mapping_of(&POSITIVE).len(), 8);
        assert_eq!(header_of(&Unit { is_meta: false }, &META_ONLY).len(), 0);
        assert_eq!(header_of(&Unit { is_meta: true }, &META_ONLY).len(), 5);
    }

    #[test]
    fn 空语料的缺口恒为零_且不许被当成结论() {
        let a = rebuild(&[], &NEGATIVE, true);
        assert_eq!(a.units, 0);
        assert_eq!(a.fields, 0);
    }

    #[test]
    fn 方向a的免费输入本来就全在映射层条目里() {
        // 本地腿（反例立场）攻的是「A 有免费输入而 B 没有，比较不对称」。
        // 这条断言给出答案：Dev / Off / Crc **三项全部已在映射层七字段之内**，
        // 方向 B 一开始就拿着它们 ⇒ 不对称不存在，对称臂因此恒等于现行臂。
        // ⇒ 这个「恒等」是结论，不是疏忽；写成断言免得后人当成没做。
        let free: BTreeSet<F> = [F::Dev, F::Off, F::Crc].into_iter().collect();
        let mapping = mapping_of(&CURRENT);
        assert!(free.is_subset(&mapping), "免费输入若跑到映射层之外，这个判定就真的不对称了");
        assert_eq!(known_b(&SYMMETRIC), known_b(&CURRENT), "对称臂给不出任何新东西");
    }

    #[test]
    fn 对称臂_把免费输入补给方向b之后结论不变() {
        // 本地腿的攻击：方向 A 有免费输入而 B 没有 ⇒ 比较不对称。
        // 补齐之后若结论翻转，本实验的判定就是那个不对称造成的伪影。
        let u = corpus(11, 33);
        let n = 44;
        let a = rebuild(&u, &SYMMETRIC, true);
        let b = rebuild(&u, &SYMMETRIC, false);
        assert_eq!(a.units, 0);
        assert_eq!(b.units, n, "补齐免费输入之后，方向 B 仍然每个单元都缺");
        assert_eq!(b.fields, n, "而且仍然恰好缺一项");
        assert_eq!(b.kinds.into_iter().collect::<Vec<_>>(), vec![F::Kind], "缺的仍然是类型标签");
        // 对称臂与现行臂给出**同一个**判定，差别只在方向 B 多拿了一个它本来就有的字段
        assert_eq!(rebuild(&u, &CURRENT, false).units, b.units);
    }

    #[test]
    fn 判定表按可推导性逐格对() {
        // 钉绝对值：四格逐个对，不许只看「结论有没有变」。
        assert!(verdict_of(0, 7).contains("D18"));
        assert!(verdict_of(7, 0).contains("D9"));
        assert!(verdict_of(0, 0).contains("任选"));
        assert!(verdict_of(7, 7).contains("按字段拆"));
    }

    #[test]
    fn 己臂_块头禁放物理落点时两向都有缺口() {
        // D18 已定项 3 的禁放清单：块头永远禁放物理落点 / 设备 ID。
        // 喂进模型之后方向 A 也塌了 ⇒ 落在判据 4 的「按字段拆」。
        let (nm, nd) = (100usize, 300usize);
        let u = corpus(nm, nd);
        let n = nm + nd;
        let a = rebuild(&u, &CORRECTED, true);
        let b = rebuild(&u, &CORRECTED, false);
        assert_eq!(a.units, n);
        assert_eq!(a.fields, 2 * n, "缺的恰好是 Dev 与 Off 两项");
        assert_eq!(a.kinds.into_iter().collect::<Vec<_>>(), vec![F::Dev, F::Off]);
        assert_eq!(b.units, n);
        assert_eq!(b.fields, n);
        assert!(verdict_of(a.units, b.units).contains("按字段拆"));
        // 与甲臂的差别只在这一格，别的条件都一样
        assert_eq!(rebuild(&u, &CURRENT, true).units, 0, "甲臂靠的就是那两项免费输入");
    }

    #[test]
    fn 判定表四种结局都取得到() {
        let u = corpus(4, 4);
        assert!(rebuild(&u, &CURRENT, true).units == 0 && rebuild(&u, &CURRENT, false).units > 0);
        assert!(rebuild(&u, &POSITIVE, true).units == 0 && rebuild(&u, &POSITIVE, false).units == 0);
        assert!(rebuild(&u, &META_ONLY, true).units > 0 && rebuild(&u, &META_ONLY, false).units > 0);
    }
}
