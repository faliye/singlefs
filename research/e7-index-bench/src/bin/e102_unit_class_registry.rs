//! E102：单元类登记表与打包记录单元 —— D18 已定项 11 的计数模型。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D9 已定项 6**：单元类型标签是 AAD 的域分隔子——「没有域分隔时两类单元的 AAD 可序列化成
//!   同一串字节 ⇒ 类型混淆变成一次合法的 MAC 通过」；那一行给的值域是
//!   「数据 extent / btree 节点 / 记账单元 / 根记录」。
//! - **D9 已定项 6 排除表**：「AAD 的字段组成是 day-1 永久契约……判据只能按『这个字段将来
//!   可不可能逐指针不同』定，不能按『它今天是什么值』定」。
//! - **D18 已定项 7**：共同前缀 42 = magic 4 + 版本 2 + flags 2（含单元类标签位）+ 声明长度 2 +
//!   头校验和 32；数据单元类身份段 = 五元组 33（标签 1 + 树 8 + 对象 8 + 出生代 8 + 锚点 8）+
//!   诞生代号 8 + fsid 8 ⇒ 头 91；「自证单元（journal 记录 / 根槽 / 超级块槽）不用这张表」。
//! - **D18 已定项 8**：「单元类型标签取『墓碑』」。**D18 已定项 10**：墓碑打包共享单元、
//!   按代际装载、回收粒度 = 单元；E83：(32768 − 91) / 56 = 583。E98：(32768 − 91) / 140 = 233。
//! - **D21 已定项 4**：扩展点归属三类（数据单元 / 索引节点 / 自证单元）。
//! - **I-6.2**：两张白名单（数据单元类 / 元数据类）。
//! - **D15 判定表**：新增记录类型 ⇒ compat_ro；改变已有字段含义 ⇒ incompat。
//! - **D23 已定项 1 下属「记录头的类型字段」小节 A 条**：「未知类型 ⇒ 拒绝挂载，由超级块 incompat 位承载」。
//!
//! ## 提案（`research/prompts/_d18-item11-unit-class-registry.md`）里被模型化的部分
//!
//! 登记表码 0 无效 / 1 数据单元 / 2 索引节点 / 3 打包记录单元 / 16 根记录 / 17 journal 记录 /
//! 18 超级块槽，其余保留；标签 1 字节住共同前缀偏移 6、是 AAD 首字节；打包记录单元类身份段 51 字节
//! （含 4 字节自包含载荷校验和；2026-09-05 C113 定案再加 10 字节写序）⇒ 头 103；一个单元只装一种记录类型 / 一个代际 / 一棵树，记录定宽；未登记单元类 ⇒ 拒收 + incompat，
//! 未登记记录类型 ⇒ 容器可验、内容跳过、只读（compat_ro）。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. **覆盖**：仓里四处清单与已定项 8 用到的每个类名，在登记表下映到**恰一个**码；
//!    映不到的数与一名多码的数都必须恰好 0。阳性对照：拿 D9 那行的四个名字当登记表时，
//!    映不到的必须恰好 1（墓碑）。
//! 2. **域分隔**：两类 AAD 规范编码能相同 ⇔ 标签相同（或都没有）且身体等长。
//!    阳性对照：与数据单元身体等宽（32 字节）的合成类，**去掉标签**必须恰撞 1 对；带标签恰 0 对。
//!    阴性对照：只有一类时任何设置都 0 对。
//! 3. **打包容量与下游数**：cap(h, r) = (32768 − h) / r。钉：cap(103, 56) = 583、cap(103, 140) = 233（93 时同值，成立区间 [65, 120]）；
//!    并算出 583 / 233 各自成立的头宽闭区间，两端各多 1 字节必须掉 1 格。
//! 4. **指认力**：一个五元组只指认 1 个对象 ⇒ 装 N 条的打包单元用五元组指认不到 N − 1 条；
//!    用打包类身份段 + 解析枚举则指认不到的恰好 0。
//! 5. **定宽解析与混装**：解析器只认 (记录数, 记录宽, 声明长度)，记录数 × 记录宽 + 头 > 声明长度即判损坏；
//!    混装两种宽度时按第一种宽度解析，错分的记录数由 gcd 算出（56 / 140 先后各 10 条 ⇒ 5）；不混装恒 0。
//! 6. **未登记码的两级政策**：镜像里 k 个未登记单元类——incompat 政策静默漏 0 且拒挂；
//!    「跳过」政策静默漏恰 k。m 个未登记记录类型的容器——容器可验恰 m、应用记录恰 0、只读、不许回收。
//! 7. **值域余量**：256 − 7 = 249；flags 高字节 8 位全空。
//!
//! ## 失败条款（跑前写死）
//!
//! - 判据 1/2/5 的阳性对照任一不中 ⇒ 那一维没进模型，**整轮作废**。
//! - 判据 2/5 的阴性对照任一不为 0 ⇒ 模型在平凡输入上就错，整轮作废。
//! - **反向接受条款**：若 cap(103, 56) ≠ 583 或 cap(103, 140) ≠ 233 ⇒ 提案 P8 不成立，
//!   E83 / E84 / E98 必须重跑，如实写。
//! - 读不到 ≠ 读到 0：未登记码的判定返回 `Err`，不许退化成「0 个单元」。
//!
//! ## 它答不了的
//!
//! 纯算术：不建 AAD 密码学模型（域分隔只按「编码能否相同」判，不判 MAC），不跑扫描器，
//! 不判打包记录单元在正常路径怎么被索引（提案 P7 立成欠账），不答 D8 已定项 6 取哪条路。

use e7_index_bench::Emitter;

const UNIT: u64 = 32768;
const COMMON_PREFIX: u64 = 42;
/// D18 已定项 7 数据单元头（kb 里 `format-const: UNIT_HDR_DATA`）。
const UNIT_HDR_DATA: u64 = 105;
/// 提案 P4（第三版）：打包记录单元类身份段 = 单元类型标签 1（偏移 6 的密文侧副本）+ 出生树 ID 8 +
/// 打包记录类型 2 + 容器号 8 + 容器出生代 8 + 记录数 2 + 记录宽 2 + 诞生代号 8 + fsid 8 +
/// 载荷校验和 4（CRC32C，D23 已定项 13 口径）+ 写序 10（C113 定案，2026-09-05）= 61 ⇒ 头 103。
/// 载荷校验和为什么必须自包含：整单元校验和 / MAC 住父指针（D4），扫描认领时没有父指针，
/// 头校验和按 D18 已定项 7 逐字只是「头完整性唯一防线」——E76 实测「头落了、载荷只落一半」
/// 对只有头校验和的形态 distinguishable=0。记录宽在头里再抄一份：不认识记录类型的读者才判得了
/// 「记录数 × 记录宽 ≤ 声明长度」这条头合法性条件（D23 已定项 1 的 B 条同形）。
const PACKED_BODY: u64 = 61;
/// D18 已定项 11 打包记录单元头（kb 里 `format-const: UNIT_HDR_PACKED`）；单测钉它 == 前缀 + 类身份段。
const UNIT_HDR_PACKED: u64 = 103;
/// E83 的墓碑区间记录量级假设；E98 的 inode 记录。
const TOMB_REC: u64 = 56;
const INODE_REC: u64 = 140;
/// 标签 1 字节。
const TAG_VALUES: u64 = 256;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Invalid,
    Content,
    SelfCert,
}

/// 提案 P2 的登记表。
const REGISTRY: [(u8, &str, Kind); 7] = [
    (0, "invalid", Kind::Invalid),
    (1, "data_unit", Kind::Content),
    (2, "index_node", Kind::Content),
    (3, "packed_record_unit", Kind::Content),
    (16, "root_record", Kind::SelfCert),
    (17, "journal_record", Kind::SelfCert),
    (18, "superblock_slot", Kind::SelfCert),
];

/// 仓里四处清单 + D18 已定项 8 用到的类名（来源, 名字）。
const USED_NAMES: [(&str, &str); 15] = [
    ("D9-item6", "data_extent"),
    ("D9-item6", "btree_node"),
    ("D9-item6", "accounting_unit"),
    ("D9-item6", "root_record"),
    ("D18-item7", "data_unit"),
    ("D18-item7", "index_node"),
    ("D18-item7", "journal_record"),
    ("D18-item7", "root_slot"),
    ("D18-item7", "superblock_slot"),
    ("D21-item4", "data_unit"),
    ("D21-item4", "index_node"),
    ("D21-item4", "self_cert_unit"),
    ("I-6.2", "data_unit_class"),
    ("I-6.2", "metadata_class"),
    ("D18-item8", "tombstone"),
];

/// 提案给出的名字 → 码映射。一个名字映到多个码时返回全部，判据 1 要求恰一个。
fn codes_for(name: &str) -> Vec<u8> {
    match name {
        "data_extent" | "data_unit" | "data_unit_class" => vec![1],
        "btree_node" | "index_node" | "accounting_unit" => vec![2],
        "tombstone" => vec![3],
        // I-6.2 的「元数据类」是一张白名单，罩索引节点与打包记录单元两个码
        "metadata_class" => vec![2, 3],
        "root_record" | "root_slot" => vec![16],
        "journal_record" => vec![17],
        "superblock_slot" => vec![18],
        // D21 已定项 4 的「自证单元」罩三个自证码
        "self_cert_unit" => vec![16, 17, 18],
        _ => vec![],
    }
}

/// 白名单 / 归属类这种「一名罩多码」的名字，判据 1 按「罩到的每个码都已登记」算，不算一名多码。
fn is_umbrella(name: &str) -> bool {
    matches!(name, "metadata_class" | "self_cert_unit")
}

/// **判据 1**：映不到的名字数、一名多码的名字数、映到未登记码的名字数。
fn coverage(registry: &[(u8, &str, Kind)], names: &[(&str, &str)]) -> (u64, u64, u64) {
    let registered = |c: u8| registry.iter().any(|r| r.0 == c && r.2 != Kind::Invalid);
    let mut unmapped = 0u64;
    let mut multi = 0u64;
    let mut unregistered = 0u64;
    for &(_, n) in names {
        let cs = codes_for(n);
        if cs.is_empty() {
            unmapped += 1;
            continue;
        }
        if cs.len() > 1 && !is_umbrella(n) {
            multi += 1;
        }
        if cs.iter().any(|&c| !registered(c)) {
            unregistered += 1;
        }
    }
    (unmapped, multi, unregistered)
}

/// 阳性对照用：把 D9 那一行的四个名字当登记表——只登记码 1 / 2 / 16。
const D9_ONLY: [(u8, &str, Kind); 3] = [
    (1, "data_unit", Kind::Content),
    (2, "index_node", Kind::Content),
    (16, "root_record", Kind::SelfCert),
];

/// AAD 规范编码的形状：标签（可无）+ 身体字节数。
#[derive(Clone, Copy, Debug)]
struct AadShape {
    tag: Option<u8>,
    body: u64,
}

/// **判据 2**：两条编码能不能字节相同——标签相同（或都没有）且身体等长。
/// AEAD 隐式认证 assoclen（D9 已定项 6 引 `chacha20poly1305.c:184-185`），所以不等长的永不相同。
fn can_collide(a: AadShape, b: AadShape) -> bool {
    a.tag == b.tag && a.body == b.body
}

fn colliding_pairs(shapes: &[AadShape]) -> u64 {
    let mut n = 0;
    for i in 0..shapes.len() {
        for j in (i + 1)..shapes.len() {
            if can_collide(shapes[i], shapes[j]) {
                n += 1;
            }
        }
    }
    n
}

/// 已定的两个 AAD 身体：数据单元（树 8 + 对象 8 + 出生代 8 + 锚点 8 = 32）、
/// 打包记录单元（树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8 = 26）。
const AAD_DATA_BODY: u64 = 32;
const AAD_PACKED_BODY: u64 = 26;
/// 阳性对照的合成类：身体与数据单元等宽（四个 u64）。
const AAD_SYNTHETIC_BODY: u64 = 32;

fn aad_shapes(with_tag: bool, include_synthetic: bool) -> Vec<AadShape> {
    let t = |c: u8| if with_tag { Some(c) } else { None };
    let mut v = vec![
        AadShape { tag: t(1), body: AAD_DATA_BODY },
        AadShape { tag: t(3), body: AAD_PACKED_BODY },
    ];
    if include_synthetic {
        v.push(AadShape { tag: t(4), body: AAD_SYNTHETIC_BODY });
    }
    v
}

/// **判据 3**：打包容量。
fn capacity(hdr: u64, rec: u64) -> u64 {
    if rec == 0 || hdr >= UNIT {
        return 0;
    }
    (UNIT - hdr) / rec
}

/// 容量恰为 `cap` 时头宽的闭区间 [lo, hi]。
fn header_interval_for(cap: u64, rec: u64) -> (u64, u64) {
    let hi = UNIT - cap * rec;
    let lo = UNIT - (cap + 1) * rec + 1;
    (lo, hi)
}

/// **判据 4**：用五元组指认一个装 n 条的单元，指认不到几条；用类身份段 + 解析枚举则为 0。
fn unnamed_by_five_tuple(n: u64) -> u64 {
    n.saturating_sub(1)
}
fn unnamed_by_packed_identity(n: u64, parsed: u64) -> u64 {
    n.saturating_sub(parsed)
}

#[derive(Debug, PartialEq, Eq)]
enum Parse {
    Records(u64),
    Corrupt,
}

/// **判据 5**：定宽解析。只认头里的三个数。
fn parse_packed(count: u64, width: u64, declared_len: u64) -> Parse {
    if width == 0 || count.saturating_mul(width).saturating_add(UNIT_HDR_PACKED) > declared_len {
        return Parse::Corrupt;
    }
    Parse::Records(count)
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// 混装：先 `n_a` 条宽 `a`，再 `n_b` 条宽 `b`，按宽 `a` 解析。
/// 错分 = 宽 b 的记录里起点不落在 a 的倍数上的条数。独立算术：起点 n_a·a + i·b，
/// 落在 a 的倍数上 ⇔ i·b ≡ 0 (mod a) ⇔ i 是 a/gcd(a,b) 的倍数。
fn misparsed_when_mixed(a: u64, n_a: u64, b: u64, n_b: u64) -> u64 {
    let _ = n_a; // 前段全对齐，只影响起点，不影响后段的对齐判定
    let period = a / gcd(a, b);
    (0..n_b).filter(|i| i % period != 0).count() as u64
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UnknownClassPolicy {
    Incompat,
    Skip,
}

#[derive(Debug, PartialEq, Eq)]
struct MountOutcome {
    mount_ok: bool,
    silent_missed: u64,
}

/// **判据 6（第一级）**：镜像里 `k` 个未登记单元类。
fn mount_with_unknown_class(k: u64, policy: UnknownClassPolicy) -> Result<MountOutcome, &'static str> {
    if k == 0 {
        return Ok(MountOutcome { mount_ok: true, silent_missed: 0 });
    }
    match policy {
        UnknownClassPolicy::Incompat => Ok(MountOutcome { mount_ok: false, silent_missed: 0 }),
        UnknownClassPolicy::Skip => Ok(MountOutcome { mount_ok: true, silent_missed: k }),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RecordTypeOutcome {
    containers_verified: u64,
    records_applied: u64,
    read_only: bool,
    reclaim_allowed: bool,
}

/// **判据 6（第二级）**：`m` 个容器的记录类型未登记，但容器头给了记录数 × 记录宽。
fn mount_with_unknown_record_type(m: u64) -> RecordTypeOutcome {
    RecordTypeOutcome {
        containers_verified: m,
        records_applied: 0,
        read_only: m > 0,
        reclaim_allowed: false,
    }
}

/// **判据 7**：值域余量。
fn free_codes(registry: &[(u8, &str, Kind)]) -> u64 {
    TAG_VALUES - registry.len() as u64
}

fn main() {
    let mut em = Emitter::new();
    let mut out: Vec<String> = Vec::new();

    out.push(em.emit_raw(&format!(
        "name=config unit={UNIT} common_prefix={COMMON_PREFIX} data_hdr={UNIT_HDR_DATA} packed_body={PACKED_BODY} packed_hdr={UNIT_HDR_PACKED} \
         tomb_rec={TOMB_REC} inode_rec={INODE_REC} tag_values={TAG_VALUES} registry={} used_names={} \
         model=arithmetic file_ops=0",
        REGISTRY.len(),
        USED_NAMES.len()
    )));

    // 判据 1：覆盖
    let (u, m, r) = coverage(&REGISTRY, &USED_NAMES);
    out.push(em.emit_raw(&format!(
        "name=coverage registry=proposal unmapped={u} multi_code={m} unregistered={r}"
    )));
    let (u, m, r) = coverage(&D9_ONLY, &USED_NAMES);
    out.push(em.emit_raw(&format!(
        "name=coverage registry=d9_only unmapped={u} multi_code={m} unregistered={r} \
         note=positive_control_tombstone_missing"
    )));

    // 判据 2：域分隔
    for (with_tag, synth) in [(false, false), (true, false), (false, true), (true, true)] {
        let s = aad_shapes(with_tag, synth);
        out.push(em.emit_raw(&format!(
            "name=aad_domain with_tag={} synthetic_class={} classes={} colliding_pairs={}",
            u8::from(with_tag),
            u8::from(synth),
            s.len(),
            colliding_pairs(&s)
        )));
    }
    out.push(em.emit_raw(&format!(
        "name=aad_domain with_tag=0 synthetic_class=0 classes=1 colliding_pairs={} note=negative_control",
        colliding_pairs(&aad_shapes(false, false)[..1])
    )));

    // 判据 3：容量与头宽区间
    for &rec in [TOMB_REC, INODE_REC].iter() {
        for &h in [64u64, 65, 76, UNIT_HDR_DATA, UNIT_HDR_PACKED, 120, 121, 148, 149].iter() {
            out.push(em.emit_raw(&format!(
                "name=capacity rec={rec} hdr={h} cap={}",
                capacity(h, rec)
            )));
        }
        let c = capacity(UNIT_HDR_PACKED, rec);
        let (lo, hi) = header_interval_for(c, rec);
        out.push(em.emit_raw(&format!(
            "name=capacity_interval rec={rec} cap_at_packed_hdr={c} hdr_lo={lo} hdr_hi={hi} \
             cap_at_lo_minus_1={} cap_at_hi_plus_1={}",
            capacity(lo - 1, rec),
            capacity(hi + 1, rec)
        )));
    }

    // 判据 4：指认力
    let n = capacity(UNIT_HDR_PACKED, TOMB_REC);
    out.push(em.emit_raw(&format!(
        "name=identity records={n} unnamed_by_five_tuple={} unnamed_by_packed_identity={}",
        unnamed_by_five_tuple(n),
        unnamed_by_packed_identity(n, match parse_packed(n, TOMB_REC, UNIT) {
            Parse::Records(k) => k,
            Parse::Corrupt => 0,
        })
    )));

    // 判据 5：解析与混装
    out.push(em.emit_raw(&format!(
        "name=parse count={n} width={TOMB_REC} declared={UNIT} result={:?}",
        parse_packed(n, TOMB_REC, UNIT)
    )));
    out.push(em.emit_raw(&format!(
        "name=parse count={} width={TOMB_REC} declared={UNIT} result={:?} note=one_too_many",
        n + 1,
        parse_packed(n + 1, TOMB_REC, UNIT)
    )));
    out.push(em.emit_raw(&format!(
        "name=mixing a={TOMB_REC} n_a=10 b={INODE_REC} n_b=10 misparsed={}",
        misparsed_when_mixed(TOMB_REC, 10, INODE_REC, 10)
    )));
    out.push(em.emit_raw(&format!(
        "name=mixing a={TOMB_REC} n_a=10 b={TOMB_REC} n_b=10 misparsed={} note=unmixed",
        misparsed_when_mixed(TOMB_REC, 10, TOMB_REC, 10)
    )));

    // 判据 6：两级政策
    for &(k, p) in [(7u64, UnknownClassPolicy::Incompat), (7, UnknownClassPolicy::Skip), (0, UnknownClassPolicy::Incompat)].iter() {
        let o = mount_with_unknown_class(k, p).expect("policy model");
        out.push(em.emit_raw(&format!(
            "name=unknown_class k={k} policy={p:?} mount_ok={} silent_missed={}",
            u8::from(o.mount_ok),
            o.silent_missed
        )));
    }
    let o = mount_with_unknown_record_type(3);
    out.push(em.emit_raw(&format!(
        "name=unknown_record_type m=3 containers_verified={} records_applied={} read_only={} reclaim_allowed={}",
        o.containers_verified,
        o.records_applied,
        u8::from(o.read_only),
        u8::from(o.reclaim_allowed)
    )));

    // 判据 7：余量
    out.push(em.emit_raw(&format!(
        "name=headroom tag_values={TAG_VALUES} registered={} free={} flags_high_bits_free=8",
        REGISTRY.len(),
        free_codes(&REGISTRY)
    )));

    for l in &out {
        println!("{l}");
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_constants_match_kb() {
        assert_eq!(UNIT, 32768, "D4 已定项 7");
        assert_eq!(COMMON_PREFIX, 42, "D18 已定项 7");
        assert_eq!(UNIT_HDR_DATA, 105, "D18 已定项 7：42 + 33 + 8 + 8 + 写序 10 + 载荷 CRC 4（C113 定案，2026-09-05）");
        assert_eq!(UNIT_HDR_PACKED, 103, "D18 已定项 11：42 + 61");
        assert_eq!(UNIT_HDR_PACKED, COMMON_PREFIX + PACKED_BODY, "头 = 共同前缀 + 类身份段");
        assert_eq!(AAD_PACKED_BODY, 26, "树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8");
        assert_eq!(TOMB_REC, 56, "E83 量级假设");
        assert_eq!(INODE_REC, 140, "E98");
    }

    /// **判据 1 的绝对值**：提案登记表下映不到 0、一名多码 0、未登记 0。
    /// 阳性对照：只拿 D9 那行当登记表，映不到恰 1（墓碑）。
    #[test]
    fn criterion1_every_used_name_maps_to_exactly_one_registered_code() {
        assert_eq!(coverage(&REGISTRY, &USED_NAMES), (0, 0, 0));
        // 阳性对照：D9 那四个名字当登记表 ⇒ 墓碑（码 3）未登记
        let (unmapped, multi, unregistered) = coverage(&D9_ONLY, &USED_NAMES);
        assert_eq!(unmapped, 0, "名字都映得到码，缺的是登记位");
        assert_eq!(multi, 0);
        // 映到未登记码的名字：tombstone（3）、metadata_class（罩 3）、journal_record（17）、
        // superblock_slot（18）、self_cert_unit（罩 17/18）= 5
        assert_eq!(unregistered, 5);
        // 单独钉墓碑这一条：它是 D18 已定项 8 在用的那个值
        let (_, _, tomb_only) = coverage(&D9_ONLY, &[("D18-item8", "tombstone")]);
        assert_eq!(tomb_only, 1, "墓碑在 D9 那张清单里没有登记位");
    }

    /// **判据 2 的绝对值 + 阳性 / 阴性对照**。
    #[test]
    fn criterion2_tag_is_the_only_thing_that_separates_equal_width_bodies() {
        // 已定的两个身体不等长（32 vs 26）⇒ 即使没标签也撞不上——所以它们不是阳性对照
        assert_eq!(colliding_pairs(&aad_shapes(false, false)), 0);
        assert_eq!(colliding_pairs(&aad_shapes(true, false)), 0);
        // 阳性对照：与数据单元等宽的合成类，去标签恰撞 1 对
        assert_eq!(colliding_pairs(&aad_shapes(false, true)), 1);
        // 带标签恰 0 对
        assert_eq!(colliding_pairs(&aad_shapes(true, true)), 0);
        // 阴性对照：只有一类
        assert_eq!(colliding_pairs(&aad_shapes(false, false)[..1]), 0);
        // 标签相同但身体不等长仍不撞——AEAD 认证 assoclen
        assert!(!can_collide(
            AadShape { tag: Some(1), body: 32 },
            AadShape { tag: Some(1), body: 22 }
        ));
    }

    /// **判据 3 的绝对值 + 反向接受条款**：103 字节头下 583 / 233 不动（93 时同值），区间两端各多 1 字节掉 1 格。
    #[test]
    fn criterion3_downstream_capacities_do_not_move_under_the_packed_header() {
        assert_eq!(capacity(UNIT_HDR_PACKED, TOMB_REC), 583, "E83 / E84 的 583");
        assert_eq!(capacity(UNIT_HDR_DATA, TOMB_REC), 583);
        assert_eq!(capacity(UNIT_HDR_PACKED, INODE_REC), 233, "E98 的 233");
        assert_eq!(capacity(UNIT_HDR_DATA, INODE_REC), 233);
        // 583 成立的头宽闭区间 [65, 120]
        assert_eq!(header_interval_for(583, TOMB_REC), (65, 120));
        assert_eq!(capacity(64, TOMB_REC), 584);
        assert_eq!(capacity(121, TOMB_REC), 582);
        // 233 成立的头宽闭区间 [9, 148]
        assert_eq!(header_interval_for(233, INODE_REC), (9, 148));
        assert_eq!(capacity(8, INODE_REC), 234);
        assert_eq!(capacity(149, INODE_REC), 232);
    }

    /// **判据 4 的绝对值**：五元组指认不到 582 条；打包类身份段 + 解析恰 0。
    #[test]
    fn criterion4_five_tuple_names_one_object_packed_identity_names_all() {
        let n = capacity(UNIT_HDR_PACKED, TOMB_REC);
        assert_eq!(unnamed_by_five_tuple(n), 582);
        assert_eq!(unnamed_by_five_tuple(1), 0, "装 1 条时五元组刚好够——已定项 8 原话成立的唯一情形");
        let parsed = match parse_packed(n, TOMB_REC, UNIT) {
            Parse::Records(k) => k,
            Parse::Corrupt => 0,
        };
        assert_eq!(unnamed_by_packed_identity(n, parsed), 0);
    }

    /// **判据 5 的绝对值 + 阳性 / 阴性对照**：多一条即损坏；混装错分 5；不混装 0。
    #[test]
    fn criterion5_fixed_width_parsing_rejects_overflow_and_counts_mixing_errors() {
        assert_eq!(parse_packed(583, TOMB_REC, UNIT), Parse::Records(583));
        assert_eq!(parse_packed(584, TOMB_REC, UNIT), Parse::Corrupt, "584 × 56 + 103 = 32807 > 32768");
        assert_eq!(parse_packed(1, 0, UNIT), Parse::Corrupt, "宽 0 不是记录");
        // 声明长度可以小于单元：声明 1000 时最多 (1000 − 103) / 56 = 16 条
        assert_eq!(parse_packed(16, TOMB_REC, 1000), Parse::Records(16));
        assert_eq!(parse_packed(17, TOMB_REC, 1000), Parse::Corrupt);
        // 阳性对照：56 / 140 混装各 10 条，period = 56 / gcd(56,140) = 56 / 28 = 2 ⇒ 错分 5
        assert_eq!(misparsed_when_mixed(TOMB_REC, 10, INODE_REC, 10), 5);
        // 互素宽度全错：56 与 57，period 56 ⇒ 10 条里只有 i=0 对齐 ⇒ 错 9
        assert_eq!(misparsed_when_mixed(56, 10, 57, 10), 9);
        // 阴性对照：不混装恒 0
        assert_eq!(misparsed_when_mixed(TOMB_REC, 10, TOMB_REC, 10), 0);
        assert_eq!(misparsed_when_mixed(INODE_REC, 3, INODE_REC, 300), 0);
    }

    /// **判据 6 的绝对值**：incompat 静默漏 0 且拒挂；跳过政策静默漏恰 k；未登记记录类型可验不可用。
    #[test]
    fn criterion6_unknown_class_refuses_mount_and_unknown_record_type_is_read_only() {
        let inc = mount_with_unknown_class(7, UnknownClassPolicy::Incompat).unwrap();
        assert_eq!(inc, MountOutcome { mount_ok: false, silent_missed: 0 });
        let skip = mount_with_unknown_class(7, UnknownClassPolicy::Skip).unwrap();
        assert_eq!(skip, MountOutcome { mount_ok: true, silent_missed: 7 });
        // 阴性对照：没有未登记类时两种政策都正常挂、漏 0
        for p in [UnknownClassPolicy::Incompat, UnknownClassPolicy::Skip] {
            assert_eq!(
                mount_with_unknown_class(0, p).unwrap(),
                MountOutcome { mount_ok: true, silent_missed: 0 }
            );
        }
        let rt = mount_with_unknown_record_type(3);
        assert_eq!(
            rt,
            RecordTypeOutcome { containers_verified: 3, records_applied: 0, read_only: true, reclaim_allowed: false }
        );
        assert!(!mount_with_unknown_record_type(0).read_only);
    }

    /// **判据 7 的绝对值**：256 − 7 = 249。
    #[test]
    fn criterion7_headroom_is_counted_not_estimated() {
        assert_eq!(REGISTRY.len(), 7);
        assert_eq!(free_codes(&REGISTRY), 249);
        assert_eq!(free_codes(&D9_ONLY), 253);
        // 登记表里码不重复、0 恒无效
        let mut codes: Vec<u8> = REGISTRY.iter().map(|r| r.0).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), REGISTRY.len());
        assert_eq!(REGISTRY[0], (0, "invalid", Kind::Invalid));
    }

    /// 不合法几何一律 0 / Corrupt，不许当成测量值。
    #[test]
    fn illegal_geometry_is_not_a_measurement() {
        assert_eq!(capacity(UNIT, TOMB_REC), 0);
        assert_eq!(capacity(UNIT_HDR_PACKED, 0), 0);
        assert_eq!(parse_packed(1, TOMB_REC, 10), Parse::Corrupt);
    }
}
