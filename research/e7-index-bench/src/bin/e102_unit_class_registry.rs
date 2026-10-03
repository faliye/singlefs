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
//!   诞生代号 8 + fsid 8 ⇒ 头 91；「自证单元（journal 记录 / 根槽 / 系统配置槽）不用这张表」。
//! - **D18 已定项 8**：「单元类型标签取『墓碑』」。**D18 已定项 10**：墓碑打包共享单元、
//!   按代际装载、回收粒度 = 单元；E83：(32768 − 91) / 56 = 583。E98：(32768 − 91) / 140 = 233。
//! - **D21 已定项 4**：扩展点归属三类（数据单元 / 索引节点 / 自证单元）。
//! - **I-6.2**：两张白名单（数据单元类 / 元数据类）。
//! - **D15 判定表**：新增记录类型 ⇒ compat_ro；改变已有字段含义 ⇒ incompat。
//! - **D23 已定项 1 下属「记录头的类型字段」小节 A 条**：「未知类型 ⇒ 拒绝挂载，由系统配置 incompat 位承载」。
//!
//! ## 提案（`research/prompts/_d18-item11-unit-class-registry.md`）里被模型化的部分
//!
//! 登记表码 0 无效 / 1 数据单元 / 2 索引节点 / 3 打包记录单元 / 16 根记录 / 17 journal 记录 /
//! 18 系统配置槽，其余保留；标签 1 字节住共同前缀偏移 6、是 AAD 首字节；打包记录单元类身份段 51 字节
//! （含 4 字节自包含载荷校验和；2026-09-05 C113 定案再加 10 字节写序；2026-09-12 C288 ① 再加 4 字节出生序号）⇒ 头 107；一个单元只装一种记录类型 / 一个代际 / 一棵树，记录定宽；未登记单元类 ⇒ 拒收 + incompat，
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
//! ## 第二次跑（墓碑记录 56 → 40 字节）的改法，跑前登记 `research/prompts/e102-r2-prereg.md`
//!
//! 上面的判据、失败条款与反向接受条款原样留着并列（原文写 103 / 93，单测钉的是 107 / 105，按字面各算一遍）。
//! 第二次读法乙：墓碑记录宽取 40（D18 已定项 24 第 2 条），码 3 记录区起点取 136（含 29 字节预留位，已定项 16）。
//!
//! - 判据 3：钉 cap(136, 40) = 815、cap(136, 140) = 233，并算 815 / 233 各自成立的记录区起点闭区间，
//!   两端各多 1 字节必须掉 1 格；两区间的交与 136 到两端的余量另报。h = 107 的行与 r = 56 的行留作回归与敏感性取样点。
//! - 判据 4：N = cap(136, 40)，五元组指认不到 N − 1，类身份段 + 解析 0。
//! - 判据 5：解析原点取 136；40 / 140 先后各 10 条错分 5（gcd 20，周期 2），40 / 41 错分 9；不混装恒 0。
//! - 反向接受条款（改写）：若 cap(136, 40) ≠ 815 或 cap(136, 140) ≠ 233 ⇒ P8 的这一改写不成立，
//!   用到那个容量的 E83 / E84 / E98 必须重跑，如实写。815 / 233 是 kb 登记值（`TOMBSTONE_CONTAINER_RECORDS`、`INODE_LEAF_RECORDS`），不是从容量式推出来的。
//! - 新加失败条款 F4：宽度 / 原点旋钮不动输出（`name=positive_control` 任一行 ok=false）⇒ 那一维没进模型，整轮作废。
//!
//! ## 它答不了的
//!
//! 纯算术：不建 AAD 密码学模型（域分隔只按「编码能否相同」判，不判 MAC），不跑扫描器，
//! 不判打包记录单元在正常路径怎么被索引（提案 P7 立成欠账），不答 D8 已定项 6 取哪条路。

use e7_index_bench::Emitter;

const UNIT_BYTES: u64 = 32768;
const COMMON_PREFIX_BYTES: u64 = 42;
/// D18 已定项 7 数据单元头（kb 里 `format-const: DATA_UNIT_HEADER_BYTES`）。
const DATA_UNIT_HEADER_BYTES: u64 = 105;
/// 提案 P4（第三版）：打包记录单元类身份段 = 单元类型标签 1（偏移 6 的密文侧副本）+ 出生树 ID 8 +
/// 打包记录类型 2 + 容器号 8 + 容器出生代 8 + 记录数 2 + 记录宽 2 + 诞生代号 8 + fsid 8 +
/// 载荷校验和 4（CRC32C，D23 已定项 13 口径）+ 写序 10（C113 定案，2026-09-05）+ 出生序号 4（C288 ①，2026-09-12）= 65 ⇒ 头 107。
/// 载荷校验和为什么必须自包含：整单元校验和 / MAC 住父指针（D4），扫描认领时没有父指针，
/// 头校验和按 D18 已定项 7 逐字只是「头完整性唯一防线」——E76 实测「头落了、载荷只落一半」
/// 对只有头校验和的形态 distinguishable=0。记录宽在头里再抄一份：不认识记录类型的读者才判得了
/// 「记录数 × 记录宽 ≤ 声明长度」这条头合法性条件（D23 已定项 1 的 B 条同形）。
const PACKED_BODY_BYTES: u64 = 65;
/// D18 已定项 11 打包记录单元头（kb 里 `format-const: PACKED_UNIT_HEADER_BYTES`）；单测钉它 == 前缀 + 类身份段。
const PACKED_UNIT_HEADER_BYTES: u64 = 107;
/// nonce 12 + MAC 16 + 算法类型 1 的预留位（D18 已定项 16；kb 里 `format-const: NONCE_MAC_ALGORITHM_RESERVED_BYTES`）。
const NONCE_MAC_ALGORITHM_RESERVED_BYTES: u64 = 29;
/// 码 3 记录区起点 = 含预留位的头末端 107 + 29（D18 已定项 16；kb 里 `format-const: PACKED_UNIT_RECORDS_OFFSET`）。
const PACKED_UNIT_RECORDS_OFFSET: u64 = 136;
/// 容量、指认力、解析、下游数共用的记录区原点；读法乙取 `PACKED_UNIT_RECORDS_OFFSET`（登记「读法」一节）。
/// 装置是独立手写模型，不引 `crates/`；这个别名让「原点漏了预留位」只有一处可改。
const PACKED_RECORDS_AREA_ORIGIN: u64 = PACKED_UNIT_RECORDS_OFFSET;
/// 墓碑区间记录：对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8（D18 已定项 24 第 2 条；kb 里 `format-const: TOMBSTONE_RECORD_BYTES`）。
const TOMBSTONE_RECORD_BYTES: u64 = 40;
/// 第一次跑的墓碑记录宽（E83 的量级假设）。只当回归与方向相反的取样点，不是格式常量。
const FIRST_RUN_TOMBSTONE_RECORD_BYTES: u64 = 56;
/// E98 的 inode 记录（kb 里 `format-const: INODE_RECORD_BYTES`）。
const INODE_RECORD_BYTES: u64 = 140;
/// kb 登记的两个下游容量，判 P8 改写用的门槛（`format-const: TOMBSTONE_CONTAINER_RECORDS`、`format-const: INODE_LEAF_RECORDS`）。
const TOMBSTONE_CONTAINER_RECORDS: u64 = 815;
const INODE_LEAF_RECORDS: u64 = 233;
/// 原反向接受条款字面写的头宽 103（判据 3 原文）。
const ORIGINAL_CLAUSE_HEADER_BYTES: u64 = 103;
/// 容量取样点的记录宽：现行 40、第一次 56、inode 140。
const SAMPLED_RECORD_BYTES: [u64; 3] = [TOMBSTONE_RECORD_BYTES, FIRST_RUN_TOMBSTONE_RECORD_BYTES, INODE_RECORD_BYTES];
/// 容量取样点的头宽：原九点 + 新八点（88 / 89 / 128 / 129 是 816 区间两端内外，128 / 129 / 168 / 169 是 815 区间两端内外，134 / 136 是码 1 / 码 3 记录区起点）。
const SAMPLED_HEADER_BYTES: [u64; 17] = [64, 65, 76, 88, 89, DATA_UNIT_HEADER_BYTES, PACKED_UNIT_HEADER_BYTES, 120, 121, 128, 129, 134, PACKED_UNIT_RECORDS_OFFSET, 148, 149, 168, 169];
/// 标签 1 字节。
const TAG_VALUES: u64 = 256;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Invalid,
    Content,
    SelfCertifying,
}

/// 提案 P2 的登记表。
const REGISTRY: [(u8, &str, Kind); 7] = [
    (0, "invalid", Kind::Invalid),
    (1, "data_unit", Kind::Content),
    (2, "index_node", Kind::Content),
    (3, "packed_record_unit", Kind::Content),
    (16, "root_record", Kind::SelfCertifying),
    (17, "journal_record", Kind::SelfCertifying),
    (18, "system_configuration_slot", Kind::SelfCertifying),
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
    ("D18-item7", "system_configuration_slot"),
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
        "system_configuration_slot" => vec![18],
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
    let registered = |code: u8| registry.iter().any(|registry_entry| registry_entry.0 == code && registry_entry.2 != Kind::Invalid);
    let mut unmapped = 0u64;
    let mut multi_code = 0u64;
    let mut unregistered = 0u64;
    for &(_, name) in names {
        let candidate_codes = codes_for(name);
        if candidate_codes.is_empty() {
            unmapped += 1;
            continue;
        }
        if candidate_codes.len() > 1 && !is_umbrella(name) {
            multi_code += 1;
        }
        if candidate_codes.iter().any(|&code| !registered(code)) {
            unregistered += 1;
        }
    }
    (unmapped, multi_code, unregistered)
}

/// 阳性对照用：把 D9 那一行的四个名字当登记表——只登记码 1 / 2 / 16。
const REGISTRY_FROM_DECISION_NINE_ROW: [(u8, &str, Kind); 3] = [
    (1, "data_unit", Kind::Content),
    (2, "index_node", Kind::Content),
    (16, "root_record", Kind::SelfCertifying),
];

/// AAD 规范编码的形状：标签（可无）+ 身体字节数。
#[derive(Clone, Copy, Debug)]
struct AadShape {
    tag: Option<u8>,
    body: u64,
}

/// **判据 2**：两条编码能不能字节相同——标签相同（或都没有）且身体等长。
/// AEAD 隐式认证 assoclen（D9 已定项 6 引 `chacha20poly1305.c:184-185`），所以不等长的永不相同。
fn can_collide(first_shape: AadShape, second_shape: AadShape) -> bool {
    first_shape.tag == second_shape.tag && first_shape.body == second_shape.body
}

fn colliding_pairs(shapes: &[AadShape]) -> u64 {
    let mut collision_count = 0;
    for first_index in 0..shapes.len() {
        for second_index in (first_index + 1)..shapes.len() {
            if can_collide(shapes[first_index], shapes[second_index]) {
                collision_count += 1;
            }
        }
    }
    collision_count
}

/// 已定的两个 AAD 身体：数据单元（树 8 + 对象 8 + 出生代 8 + 锚点 8 = 32）、
/// 打包记录单元（树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8 = 26）。
const AAD_DATA_BODY: u64 = 32;
const AAD_PACKED_BODY: u64 = 26;
/// 阳性对照的合成类：身体与数据单元等宽（四个 u64）。
const AAD_SYNTHETIC_BODY: u64 = 32;

fn aad_shapes(with_tag: bool, include_synthetic: bool) -> Vec<AadShape> {
    let tag_if_enabled = |code: u8| if with_tag { Some(code) } else { None };
    let mut shapes = vec![
        AadShape { tag: tag_if_enabled(1), body: AAD_DATA_BODY },
        AadShape { tag: tag_if_enabled(3), body: AAD_PACKED_BODY },
    ];
    if include_synthetic {
        shapes.push(AadShape { tag: tag_if_enabled(4), body: AAD_SYNTHETIC_BODY });
    }
    shapes
}

/// **判据 3**：打包容量。
fn capacity(header_bytes: u64, record_bytes: u64) -> u64 {
    if record_bytes == 0 || header_bytes >= UNIT_BYTES {
        return 0;
    }
    (UNIT_BYTES - header_bytes) / record_bytes
}

/// 容量恰为 `target_capacity` 时头宽的闭区间 [lo, hi]。
fn header_interval_for(target_capacity: u64, record_bytes: u64) -> (u64, u64) {
    let highest_header_bytes = UNIT_BYTES - target_capacity * record_bytes;
    let lowest_header_bytes = UNIT_BYTES - (target_capacity + 1) * record_bytes + 1;
    (lowest_header_bytes, highest_header_bytes)
}

/// **判据 4**：用五元组指认一个装 n 条的单元，指认不到几条；用类身份段 + 解析枚举则为 0。
fn unnamed_by_five_tuple(record_count: u64) -> u64 {
    record_count.saturating_sub(1)
}
fn unnamed_by_packed_identity(record_count: u64, parsed: u64) -> u64 {
    record_count.saturating_sub(parsed)
}

#[derive(Debug, PartialEq, Eq)]
enum Parse {
    Records(u64),
    Corrupt,
}

/// **判据 5**：定宽解析。只认头里的三个数；`declared_length` 从单元起点量，`records_origin` 是记录区起点。
fn parse_packed_from_origin(count: u64, width: u64, declared_length: u64, records_origin: u64) -> Parse {
    if width == 0 || count.saturating_mul(width).saturating_add(records_origin) > declared_length {
        return Parse::Corrupt;
    }
    Parse::Records(count)
}

/// 第二次的解析：原点取 `PACKED_RECORDS_AREA_ORIGIN`（136）。
fn parse_packed(count: u64, width: u64, declared_length: u64) -> Parse {
    parse_packed_from_origin(count, width, declared_length, PACKED_RECORDS_AREA_ORIGIN)
}

fn greatest_common_divisor(first_value: u64, second_value: u64) -> u64 {
    if second_value == 0 {
        first_value
    } else {
        greatest_common_divisor(second_value, first_value % second_value)
    }
}

/// 混装：先 `first_count` 条宽 `first_width`，再 `second_count` 条宽 `second_width`，按宽 `first_width` 解析。
/// 错分 = 宽 b 的记录里起点不落在 a 的倍数上的条数。独立算术：起点 n_a·a + i·b，
/// 落在 a 的倍数上 ⇔ i·b ≡ 0 (mod a) ⇔ i 是 a/gcd(a,b) 的倍数。
fn misparsed_when_mixed(first_width: u64, first_count: u64, second_width: u64, second_count: u64) -> u64 {
    let _ = first_count; // 前段全对齐，只影响起点，不影响后段的对齐判定
    let period = first_width / greatest_common_divisor(first_width, second_width);
    (0..second_count).filter(|second_record_index| second_record_index % period != 0).count() as u64
}

/// 判据 4 的一行：装满一个容器的条数、五元组指认不到几条、类身份段 + 解析指认不到几条。
fn identity_report(records_origin: u64, record_bytes: u64) -> (u64, u64, u64) {
    let full_container_records = capacity(records_origin, record_bytes);
    let parsed_records = match parse_packed_from_origin(full_container_records, record_bytes, UNIT_BYTES, records_origin) {
        Parse::Records(parsed_count) => parsed_count,
        Parse::Corrupt => 0,
    };
    (
        full_container_records,
        unnamed_by_five_tuple(full_container_records),
        unnamed_by_packed_identity(full_container_records, parsed_records),
    )
}

/// 两个闭区间的交；不相交返回 `None`（读不到 ≠ 读到空区间的两端）。
fn joint_interval(first: (u64, u64), second: (u64, u64)) -> Option<(u64, u64)> {
    let joint_low = first.0.max(second.0);
    let joint_high = first.1.min(second.1);
    (joint_low <= joint_high).then_some((joint_low, joint_high))
}

/// 点到闭区间两端的余量 (到下沿, 到上沿)；点在区间外返回 `None`。
fn margins_inside(interval: (u64, u64), point: u64) -> Option<(u64, u64)> {
    (interval.0 <= point && point <= interval.1).then(|| (point - interval.0, interval.1 - point))
}

/// **Q4**：P8 在 40 下的改写——记录区起点 `records_origin` 下两个下游容量都等于 kb 登记值。
fn p8_rewrite_holds_at(records_origin: u64) -> bool {
    capacity(records_origin, TOMBSTONE_RECORD_BYTES) == TOMBSTONE_CONTAINER_RECORDS
        && capacity(records_origin, INODE_RECORD_BYTES) == INODE_LEAF_RECORDS
}

/// 阳性对照（登记第五节那张表）：拧一个旋钮，输出必须变成表里的数。`expected` 是登记里另算的绝对值。
struct PositiveControl {
    knob: &'static str,
    observed: String,
    expected: &'static str,
}

fn positive_controls() -> Vec<PositiveControl> {
    let (records_at_wide_records, unnamed_by_five_tuple_at_wide_records, _) =
        identity_report(PACKED_RECORDS_AREA_ORIGIN, FIRST_RUN_TOMBSTONE_RECORD_BYTES);
    let wide_interval = header_interval_for(capacity(PACKED_RECORDS_AREA_ORIGIN, FIRST_RUN_TOMBSTONE_RECORD_BYTES), FIRST_RUN_TOMBSTONE_RECORD_BYTES);
    let tombstone_container_records = capacity(PACKED_RECORDS_AREA_ORIGIN, TOMBSTONE_RECORD_BYTES);
    vec![
        PositiveControl {
            knob: "record_bytes_40_to_56_at_hdr_136_capacity",
            observed: capacity(PACKED_RECORDS_AREA_ORIGIN, FIRST_RUN_TOMBSTONE_RECORD_BYTES).to_string(),
            expected: "582",
        },
        PositiveControl {
            knob: "hdr_just_below_interval_136_to_128_capacity_at_rec_40",
            observed: capacity(128, TOMBSTONE_RECORD_BYTES).to_string(),
            expected: "816",
        },
        PositiveControl {
            knob: "hdr_just_above_interval_136_to_169_capacity_at_rec_40",
            observed: capacity(169, TOMBSTONE_RECORD_BYTES).to_string(),
            expected: "814",
        },
        PositiveControl {
            knob: "record_bytes_40_to_56_interval_at_hdr_136",
            observed: format!("{}-{}", wide_interval.0, wide_interval.1),
            expected: "121-176",
        },
        PositiveControl {
            knob: "record_bytes_40_to_56_identity_records_and_unnamed_by_five_tuple",
            observed: format!("{records_at_wide_records}-{unnamed_by_five_tuple_at_wide_records}"),
            expected: "582-581",
        },
        PositiveControl {
            knob: "one_record_more_than_full_container_parse",
            observed: format!("{:?}", parse_packed(tombstone_container_records + 1, TOMBSTONE_RECORD_BYTES, UNIT_BYTES)),
            expected: "Corrupt",
        },
        PositiveControl {
            knob: "mixed_first_width_40_to_41_with_140_misparsed",
            observed: misparsed_when_mixed(41, 10, INODE_RECORD_BYTES, 10).to_string(),
            expected: "9",
        },
    ]
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

/// **判据 6（第一级）**：镜像里 `unknown_class_unit_count` 个未登记单元类。
fn mount_with_unknown_class(unknown_class_unit_count: u64, policy: UnknownClassPolicy) -> Result<MountOutcome, &'static str> {
    if unknown_class_unit_count == 0 {
        return Ok(MountOutcome { mount_ok: true, silent_missed: 0 });
    }
    match policy {
        UnknownClassPolicy::Incompat => Ok(MountOutcome { mount_ok: false, silent_missed: 0 }),
        UnknownClassPolicy::Skip => Ok(MountOutcome { mount_ok: true, silent_missed: unknown_class_unit_count }),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RecordTypeOutcome {
    containers_verified: u64,
    records_applied: u64,
    read_only: bool,
    reclaim_allowed: bool,
}

/// **判据 6（第二级）**：`unknown_record_type_container_count` 个容器的记录类型未登记，但容器头给了记录数 × 记录宽。
fn mount_with_unknown_record_type(unknown_record_type_container_count: u64) -> RecordTypeOutcome {
    RecordTypeOutcome {
        containers_verified: unknown_record_type_container_count,
        records_applied: 0,
        read_only: unknown_record_type_container_count > 0,
        reclaim_allowed: false,
    }
}

/// **判据 7**：值域余量。
fn free_codes(registry: &[(u8, &str, Kind)]) -> u64 {
    TAG_VALUES - registry.len() as u64
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output_lines: Vec<String> = Vec::new();

    output_lines.push(emitter.emit_raw(&format!(
        "name=config unit={UNIT_BYTES} common_prefix={COMMON_PREFIX_BYTES} data_hdr={DATA_UNIT_HEADER_BYTES} packed_body={PACKED_BODY_BYTES} packed_hdr={PACKED_UNIT_HEADER_BYTES} \
         packed_records_offset={PACKED_UNIT_RECORDS_OFFSET} reserved={NONCE_MAC_ALGORITHM_RESERVED_BYTES} \
         tomb_rec={TOMBSTONE_RECORD_BYTES} inode_rec={INODE_RECORD_BYTES} tag_values={TAG_VALUES} registry={} used_names={} \
         model=arithmetic file_ops=0",
        REGISTRY.len(),
        USED_NAMES.len()
    )));

    // 判据 1：覆盖
    let (unmapped_name_count, multi_code_name_count, unregistered_name_count) = coverage(&REGISTRY, &USED_NAMES);
    output_lines.push(emitter.emit_raw(&format!(
        "name=coverage registry=proposal unmapped={unmapped_name_count} multi_code={multi_code_name_count} unregistered={unregistered_name_count}"
    )));
    let (unmapped_name_count, multi_code_name_count, unregistered_name_count) = coverage(&REGISTRY_FROM_DECISION_NINE_ROW, &USED_NAMES);
    output_lines.push(emitter.emit_raw(&format!(
        "name=coverage registry=d9_only unmapped={unmapped_name_count} multi_code={multi_code_name_count} unregistered={unregistered_name_count} \
         note=positive_control_tombstone_missing"
    )));

    // 判据 2：域分隔
    for (with_tag, include_synthetic) in [(false, false), (true, false), (false, true), (true, true)] {
        let shapes = aad_shapes(with_tag, include_synthetic);
        output_lines.push(emitter.emit_raw(&format!(
            "name=aad_domain with_tag={} synthetic_class={} classes={} colliding_pairs={}",
            u8::from(with_tag),
            u8::from(include_synthetic),
            shapes.len(),
            colliding_pairs(&shapes)
        )));
    }
    output_lines.push(emitter.emit_raw(&format!(
        "name=aad_domain with_tag=0 synthetic_class=0 classes=1 colliding_pairs={} note=negative_control",
        colliding_pairs(&aad_shapes(false, false)[..1])
    )));

    // 判据 3：容量与头宽区间
    for &record_bytes in SAMPLED_RECORD_BYTES.iter() {
        for &header_bytes in SAMPLED_HEADER_BYTES.iter() {
            output_lines.push(emitter.emit_raw(&format!(
                "name=capacity rec={record_bytes} hdr={header_bytes} cap={}",
                capacity(header_bytes, record_bytes)
            )));
        }
        // h = 107（类身份段末尾）：第一次的行格式，敏感性取样点与逐字节回归
        let packed_header_capacity = capacity(PACKED_UNIT_HEADER_BYTES, record_bytes);
        let (lowest_header_bytes, highest_header_bytes) = header_interval_for(packed_header_capacity, record_bytes);
        output_lines.push(emitter.emit_raw(&format!(
            "name=capacity_interval rec={record_bytes} cap_at_packed_hdr={packed_header_capacity} hdr_lo={lowest_header_bytes} hdr_hi={highest_header_bytes} \
             cap_at_lo_minus_1={} cap_at_hi_plus_1={}",
            capacity(lowest_header_bytes - 1, record_bytes),
            capacity(highest_header_bytes + 1, record_bytes)
        )));
        // h = 136（记录区起点，读法乙）：主几何
        let records_offset_capacity = capacity(PACKED_RECORDS_AREA_ORIGIN, record_bytes);
        let (lowest_origin_bytes, highest_origin_bytes) = header_interval_for(records_offset_capacity, record_bytes);
        output_lines.push(emitter.emit_raw(&format!(
            "name=capacity_interval_at_records_offset rec={record_bytes} cap_at_records_offset={records_offset_capacity} hdr_lo={lowest_origin_bytes} hdr_hi={highest_origin_bytes} \
             cap_at_lo_minus_1={} cap_at_hi_plus_1={}",
            capacity(lowest_origin_bytes - 1, record_bytes),
            capacity(highest_origin_bytes + 1, record_bytes)
        )));
    }

    // Q4：P8 在 40 下的改写（下游数）与原反向接受条款（按字面）
    let tombstone_container_records = capacity(PACKED_RECORDS_AREA_ORIGIN, TOMBSTONE_RECORD_BYTES);
    let inode_leaf_records = capacity(PACKED_RECORDS_AREA_ORIGIN, INODE_RECORD_BYTES);
    let tombstone_interval = header_interval_for(tombstone_container_records, TOMBSTONE_RECORD_BYTES);
    let inode_interval = header_interval_for(inode_leaf_records, INODE_RECORD_BYTES);
    let joint_hdr_interval = joint_interval(tombstone_interval, inode_interval);
    let joint_margins = joint_hdr_interval.and_then(|interval| margins_inside(interval, PACKED_RECORDS_AREA_ORIGIN));
    let text_of = |value: Option<u64>| value.map_or_else(|| "NA".to_string(), |present| present.to_string());
    output_lines.push(emitter.emit_raw(&format!(
        "name=downstream_at_records_offset records_offset={PACKED_RECORDS_AREA_ORIGIN} tomb_rec={TOMBSTONE_RECORD_BYTES} cap_tomb={tombstone_container_records} \
         kb_tomb={TOMBSTONE_CONTAINER_RECORDS} inode_rec={INODE_RECORD_BYTES} cap_inode={inode_leaf_records} kb_inode={INODE_LEAF_RECORDS} \
         joint_hdr_lo={} joint_hdr_hi={} margin_to_lo={} margin_to_hi={}",
        text_of(joint_hdr_interval.map(|interval| interval.0)),
        text_of(joint_hdr_interval.map(|interval| interval.1)),
        text_of(joint_margins.map(|margins| margins.0)),
        text_of(joint_margins.map(|margins| margins.1))
    )));
    output_lines.push(emitter.emit_raw(&format!(
        "name=reverse_acceptance_original literal_hdr={ORIGINAL_CLAUSE_HEADER_BYTES} cap_rec_56={} cap_rec_140={} note=literal_clause_not_the_downstream_any_more",
        capacity(ORIGINAL_CLAUSE_HEADER_BYTES, FIRST_RUN_TOMBSTONE_RECORD_BYTES),
        capacity(ORIGINAL_CLAUSE_HEADER_BYTES, INODE_RECORD_BYTES)
    )));

    // 判据 4：指认力
    let (packed_record_count, unnamed_five_tuple_count, unnamed_packed_identity_count) =
        identity_report(PACKED_RECORDS_AREA_ORIGIN, TOMBSTONE_RECORD_BYTES);
    output_lines.push(emitter.emit_raw(&format!(
        "name=identity records={packed_record_count} unnamed_by_five_tuple={unnamed_five_tuple_count} unnamed_by_packed_identity={unnamed_packed_identity_count}"
    )));

    // 判据 5：解析与混装
    output_lines.push(emitter.emit_raw(&format!(
        "name=parse count={packed_record_count} width={TOMBSTONE_RECORD_BYTES} declared={UNIT_BYTES} result={:?}",
        parse_packed(packed_record_count, TOMBSTONE_RECORD_BYTES, UNIT_BYTES)
    )));
    output_lines.push(emitter.emit_raw(&format!(
        "name=parse count={} width={TOMBSTONE_RECORD_BYTES} declared={UNIT_BYTES} result={:?} note=one_too_many",
        packed_record_count + 1,
        parse_packed(packed_record_count + 1, TOMBSTONE_RECORD_BYTES, UNIT_BYTES)
    )));
    output_lines.push(emitter.emit_raw(&format!(
        "name=mixing a={TOMBSTONE_RECORD_BYTES} n_a=10 b={INODE_RECORD_BYTES} n_b=10 misparsed={}",
        misparsed_when_mixed(TOMBSTONE_RECORD_BYTES, 10, INODE_RECORD_BYTES, 10)
    )));
    output_lines.push(emitter.emit_raw(&format!(
        "name=mixing a={TOMBSTONE_RECORD_BYTES} n_a=10 b={TOMBSTONE_RECORD_BYTES} n_b=10 misparsed={} note=unmixed",
        misparsed_when_mixed(TOMBSTONE_RECORD_BYTES, 10, TOMBSTONE_RECORD_BYTES, 10)
    )));

    // 判据 6：两级政策
    for &(unknown_class_unit_count, policy) in [(7u64, UnknownClassPolicy::Incompat), (7, UnknownClassPolicy::Skip), (0, UnknownClassPolicy::Incompat)].iter() {
        let mount_outcome = mount_with_unknown_class(unknown_class_unit_count, policy).expect("policy model");
        output_lines.push(emitter.emit_raw(&format!(
            "name=unknown_class k={unknown_class_unit_count} policy={policy:?} mount_ok={} silent_missed={}",
            u8::from(mount_outcome.mount_ok),
            mount_outcome.silent_missed
        )));
    }
    let record_type_outcome = mount_with_unknown_record_type(3);
    output_lines.push(emitter.emit_raw(&format!(
        "name=unknown_record_type m=3 containers_verified={} records_applied={} read_only={} reclaim_allowed={}",
        record_type_outcome.containers_verified,
        record_type_outcome.records_applied,
        u8::from(record_type_outcome.read_only),
        u8::from(record_type_outcome.reclaim_allowed)
    )));

    // 判据 7：余量
    output_lines.push(emitter.emit_raw(&format!(
        "name=headroom tag_values={TAG_VALUES} registered={} free={} flags_high_bits_free=8",
        REGISTRY.len(),
        free_codes(&REGISTRY)
    )));

    // 阳性对照（F4）：每个宽度 / 原点旋钮拧一下，输出必须变成登记第五节表里的数
    let mut knobs_all_moved_output = true;
    for positive_control in positive_controls() {
        let knob_moved_output_as_expected = positive_control.observed == positive_control.expected;
        knobs_all_moved_output &= knob_moved_output_as_expected;
        output_lines.push(emitter.emit_raw(&format!(
            "name=positive_control knob={} observed={} expected={} moved_as_expected={knob_moved_output_as_expected}",
            positive_control.knob, positive_control.observed, positive_control.expected
        )));
    }

    // 判决行：各量各报各的，不合成一个 true / false；字段名都按「过了为 true」写
    let original_clause_not_triggered = capacity(ORIGINAL_CLAUSE_HEADER_BYTES, FIRST_RUN_TOMBSTONE_RECORD_BYTES) == 583
        && capacity(ORIGINAL_CLAUSE_HEADER_BYTES, INODE_RECORD_BYTES) == 233;
    output_lines.push(emitter.emit_raw(&format!(
        "name=verdict quantity=Q4_tomb_capacity cap={tombstone_container_records} kb={TOMBSTONE_CONTAINER_RECORDS} equals_kb={}",
        tombstone_container_records == TOMBSTONE_CONTAINER_RECORDS
    )));
    output_lines.push(emitter.emit_raw(&format!(
        "name=verdict quantity=Q4_inode_capacity cap={inode_leaf_records} kb={INODE_LEAF_RECORDS} equals_kb={}",
        inode_leaf_records == INODE_LEAF_RECORDS
    )));
    output_lines.push(emitter.emit_raw(&format!(
        "name=verdict quantity=Q4_records_offset_inside_joint_interval records_offset={PACKED_RECORDS_AREA_ORIGIN} inside={}",
        joint_margins.is_some()
    )));
    output_lines.push(emitter.emit_raw(&format!(
        "name=verdict quantity=original_clause_literal_103 not_triggered={original_clause_not_triggered}"
    )));
    output_lines.push(emitter.emit_raw(&format!(
        "name=verdict quantity=F4_width_and_origin_knobs all_moved_output_as_expected={knobs_all_moved_output}"
    )));
    output_lines.push(emitter.emit_raw(&format!(
        "name=verdict quantity=P8_check_discriminates_h128_h136_h149 rejects_h128={} accepts_h136={} rejects_h149={}",
        !p8_rewrite_holds_at(128),
        p8_rewrite_holds_at(PACKED_RECORDS_AREA_ORIGIN),
        !p8_rewrite_holds_at(149)
    )));

    for line in &output_lines {
        println!("{line}");
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_constants_match_knowledge_base() {
        assert_eq!(UNIT_BYTES, 32768, "D4 已定项 7");
        assert_eq!(COMMON_PREFIX_BYTES, 42, "D18 已定项 7");
        assert_eq!(DATA_UNIT_HEADER_BYTES, 105, "D18 已定项 7：42 + 33 + 8 + 8 + 写序 10 + 载荷 CRC 4（C113 定案，2026-09-05）");
        assert_eq!(PACKED_UNIT_HEADER_BYTES, 107, "D18 已定项 11：42 + 65");
        assert_eq!(PACKED_UNIT_HEADER_BYTES, COMMON_PREFIX_BYTES + PACKED_BODY_BYTES, "头 = 共同前缀 + 类身份段");
        assert_eq!(AAD_PACKED_BODY, 26, "树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8");
        assert_eq!(TOMBSTONE_RECORD_BYTES, 40, "D18 已定项 24 第 2 条");
        assert_eq!(TOMBSTONE_RECORD_BYTES, 8 + 8 + 8 + 8 + 8, "对象 ID + 对象出生代 + 区间起 + 区间止 + 死亡 txg");
        assert_eq!(FIRST_RUN_TOMBSTONE_RECORD_BYTES, 56, "E83 量级假设，第一次的宽");
        assert_eq!(INODE_RECORD_BYTES, 140, "E98");
        assert_eq!(NONCE_MAC_ALGORITHM_RESERVED_BYTES, 29, "D18 已定项 16：nonce 12 + MAC 16 + 算法类型 1");
        assert_eq!(NONCE_MAC_ALGORITHM_RESERVED_BYTES, 12 + 16 + 1);
        assert_eq!(PACKED_UNIT_RECORDS_OFFSET, 136, "D18 已定项 16：含预留位的码 3 头");
        assert_eq!(
            PACKED_UNIT_RECORDS_OFFSET,
            PACKED_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES,
            "记录区起点 = 类身份段末尾 + 预留位"
        );
        assert_eq!(PACKED_RECORDS_AREA_ORIGIN, 136, "读法乙：容量、指认力、解析、下游数的原点取记录区起点，不取 107");
        assert_eq!(TOMBSTONE_CONTAINER_RECORDS, 815, "D18 已定项 24 第 2 条；layout 的 format-const");
        assert_eq!(INODE_LEAF_RECORDS, 233, "D18 已定项 11 / D8 已定项 16 的 format-const");
        // 加法写法：815 条 × 40 + 136 之后单元里还剩 32 字节
        assert_eq!(
            UNIT_BYTES,
            PACKED_UNIT_RECORDS_OFFSET + TOMBSTONE_CONTAINER_RECORDS * TOMBSTONE_RECORD_BYTES + 32,
            "815 条装完剩 32 字节，再装一条 40 字节装不下"
        );
        assert_eq!(UNIT_BYTES, PACKED_UNIT_RECORDS_OFFSET + INODE_LEAF_RECORDS * INODE_RECORD_BYTES + 12, "233 条装完剩 12 字节");
        assert_eq!(ORIGINAL_CLAUSE_HEADER_BYTES, 103, "原反向接受条款字面");
    }

    /// **判据 1 的绝对值**：提案登记表下映不到 0、一名多码 0、未登记 0。
    /// 阳性对照：只拿 D9 那行当登记表，映不到恰 1（墓碑）。
    #[test]
    fn criterion1_every_used_name_maps_to_exactly_one_registered_code() {
        assert_eq!(coverage(&REGISTRY, &USED_NAMES), (0, 0, 0));
        // 阳性对照：D9 那四个名字当登记表 ⇒ 墓碑（码 3）未登记
        let (unmapped_name_count, multi_code_name_count, unregistered_name_count) = coverage(&REGISTRY_FROM_DECISION_NINE_ROW, &USED_NAMES);
        assert_eq!(unmapped_name_count, 0, "名字都映得到码，缺的是登记位");
        assert_eq!(multi_code_name_count, 0);
        // 映到未登记码的名字：tombstone（3）、metadata_class（罩 3）、journal_record（17）、
        // system_configuration_slot（18）、self_cert_unit（罩 17/18）= 5
        assert_eq!(unregistered_name_count, 5);
        // 单独钉墓碑这一条：它是 D18 已定项 8 在用的那个值
        let (_, _, tombstone_only_unregistered_count) = coverage(&REGISTRY_FROM_DECISION_NINE_ROW, &[("D18-item8", "tombstone")]);
        assert_eq!(tombstone_only_unregistered_count, 1, "墓碑在 D9 那张清单里没有登记位");
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

    /// **判据 3 原条（第一次几何的回归，字面 56 / 140，不读墓碑宽常量）**：107 字节头下 583 / 233 不动（105、103 时同值），区间两端各多 1 字节掉 1 格。
    #[test]
    fn criterion3_first_run_geometry_rec_56_still_gives_583_and_233_under_hdr_107() {
        assert_eq!(capacity(PACKED_UNIT_HEADER_BYTES, 56), 583, "E83 / E84 的 583");
        assert_eq!(capacity(DATA_UNIT_HEADER_BYTES, 56), 583);
        assert_eq!(capacity(ORIGINAL_CLAUSE_HEADER_BYTES, 56), 583, "原反向接受条款字面 cap(103, 56)");
        assert_eq!(capacity(PACKED_UNIT_HEADER_BYTES, 140), 233, "E98 的 233");
        assert_eq!(capacity(DATA_UNIT_HEADER_BYTES, 140), 233);
        assert_eq!(capacity(ORIGINAL_CLAUSE_HEADER_BYTES, 140), 233, "原反向接受条款字面 cap(103, 140)");
        // 583 成立的头宽闭区间 [65, 120]
        assert_eq!(header_interval_for(583, 56), (65, 120));
        assert_eq!(capacity(64, 56), 584);
        assert_eq!(capacity(121, 56), 582);
        // 233 成立的头宽闭区间 [9, 148]
        assert_eq!(header_interval_for(233, 140), (9, 148));
        assert_eq!(capacity(8, 140), 234);
        assert_eq!(capacity(149, 140), 232);
    }

    /// **判据 3 第二次（读法乙）的绝对值**：整张取样表逐格钉死，数来自登记第七节 B 类（`anchors.py` 另算），不由 `capacity` 推出。
    #[test]
    fn criterion3_capacity_table_equals_the_independently_computed_registration_table() {
        let sampled_headers: [u64; 17] = [64, 65, 76, 88, 89, 105, 107, 120, 121, 128, 129, 134, 136, 148, 149, 168, 169];
        assert_eq!(SAMPLED_HEADER_BYTES, sampled_headers, "取样头宽与登记 Q2 一致");
        assert_eq!(SAMPLED_RECORD_BYTES, [40, 56, 140]);
        let expected_by_record_bytes: [[u64; 17]; 3] = [
            [817, 817, 817, 817, 816, 816, 816, 816, 816, 816, 815, 815, 815, 815, 815, 815, 814],
            [584, 583, 583, 583, 583, 583, 583, 583, 582, 582, 582, 582, 582, 582, 582, 582, 582],
            [233, 233, 233, 233, 233, 233, 233, 233, 233, 233, 233, 233, 233, 233, 232, 232, 232],
        ];
        for (record_index, &record_bytes) in SAMPLED_RECORD_BYTES.iter().enumerate() {
            for (header_index, &header_bytes) in SAMPLED_HEADER_BYTES.iter().enumerate() {
                assert_eq!(
                    capacity(header_bytes, record_bytes),
                    expected_by_record_bytes[record_index][header_index],
                    "cap({header_bytes}, {record_bytes})"
                );
            }
        }
    }

    /// **判据 3 第二次**：815 / 233 成立的记录区起点闭区间，两端各多 1 字节掉 1 格；h = 107 的敏感性取样点与 r = 56 的方向相反点。
    #[test]
    fn criterion3_intervals_edges_drop_one_cell_and_136_sits_inside_the_joint_interval() {
        assert_eq!(header_interval_for(815, 40), (129, 168), "815 条成立的记录区起点区间");
        assert_eq!((capacity(128, 40), capacity(169, 40)), (816, 814), "区间两端外各掉 1 格");
        assert_eq!(header_interval_for(233, 140), (9, 148));
        assert_eq!((capacity(8, 140), capacity(149, 140)), (234, 232));
        assert_eq!(header_interval_for(816, 40), (89, 128), "只换宽不换头：h = 107 的 816 区间");
        assert_eq!((capacity(88, 40), capacity(129, 40)), (817, 815));
        assert_eq!(header_interval_for(582, 56), (121, 176), "方向相反的点：r = 56 在 h = 136 的区间");
        assert_eq!((capacity(120, 56), capacity(177, 56)), (583, 581));
        // 两区间的交与 136 到两端的余量
        assert_eq!(joint_interval((129, 168), (9, 148)), Some((129, 148)));
        assert_eq!(joint_interval((89, 128), (9, 148)), Some((89, 128)));
        assert_eq!(joint_interval((129, 168), (200, 300)), None, "不相交时读不到交，不当成空区间的两端");
        assert_eq!(margins_inside((129, 148), PACKED_RECORDS_AREA_ORIGIN), Some((7, 12)));
        assert_eq!(margins_inside((129, 148), 128), None);
        assert_eq!(margins_inside((129, 148), 149), None);
        assert_eq!(129 + 7, 136);
        assert_eq!(136 + 12, 148);
    }

    /// **Q4 与反向接受条款**：读法乙下两个下游容量等于 kb 登记值；判别力自证——挪到区间外检查必须由绿转红。
    #[test]
    fn criterion3_downstream_capacities_at_records_offset_equal_the_kb_values_and_the_check_discriminates() {
        assert_eq!(capacity(PACKED_RECORDS_AREA_ORIGIN, TOMBSTONE_RECORD_BYTES), 815, "TOMBSTONE_CONTAINER_RECORDS");
        assert_eq!(capacity(PACKED_RECORDS_AREA_ORIGIN, INODE_RECORD_BYTES), 233, "INODE_LEAF_RECORDS");
        assert_eq!(capacity(PACKED_RECORDS_AREA_ORIGIN, FIRST_RUN_TOMBSTONE_RECORD_BYTES), 582, "按 56 与 136 的容量，已定项 11");
        assert_eq!(capacity(PACKED_UNIT_HEADER_BYTES, TOMBSTONE_RECORD_BYTES), 816, "只换宽不换头：h = 107 下 40 字节的容量");
        assert!(!p8_rewrite_holds_at(128), "128 在 815 区间外：检查必须红");
        assert!(p8_rewrite_holds_at(136));
        assert!(!p8_rewrite_holds_at(149), "149 在 233 区间外：检查必须红");
        assert!(!p8_rewrite_holds_at(PACKED_UNIT_HEADER_BYTES), "h = 107 下 816 ≠ 815");
    }

    /// **判据 4 的绝对值**：装满一个墓碑容器 815 条，五元组指认不到 814 条；打包类身份段 + 解析恰 0。
    #[test]
    fn criterion4_five_tuple_names_one_object_packed_identity_names_all() {
        assert_eq!(identity_report(PACKED_RECORDS_AREA_ORIGIN, TOMBSTONE_RECORD_BYTES), (815, 814, 0));
        assert_eq!(identity_report(PACKED_RECORDS_AREA_ORIGIN, FIRST_RUN_TOMBSTONE_RECORD_BYTES), (582, 581, 0), "r = 56 的方向相反点");
        assert_eq!(identity_report(PACKED_UNIT_HEADER_BYTES, FIRST_RUN_TOMBSTONE_RECORD_BYTES), (583, 582, 0), "第一次几何的回归：原点 107、宽 56");
        assert_eq!(unnamed_by_five_tuple(1), 0, "装 1 条时五元组刚好够——已定项 8 原话成立的唯一情形");
    }

    /// **判据 5 的绝对值 + 阳性 / 阴性对照**：多一条即损坏；混装错分 5；不混装 0。
    #[test]
    fn criterion5_fixed_width_parsing_rejects_overflow_and_counts_mixing_errors() {
        // 第二次（读法乙）：原点 136，宽 40
        assert_eq!(parse_packed(815, TOMBSTONE_RECORD_BYTES, UNIT_BYTES), Parse::Records(815));
        assert_eq!(parse_packed(816, TOMBSTONE_RECORD_BYTES, UNIT_BYTES), Parse::Corrupt, "816 × 40 + 136 = 32776 > 32768");
        assert_eq!(815 * 40 + PACKED_UNIT_RECORDS_OFFSET + 32, UNIT_BYTES, "815 条恰好差 32 字节装满");
        assert_eq!(parse_packed(1, 0, UNIT_BYTES), Parse::Corrupt, "宽 0 不是记录");
        // 声明长度可以小于单元：声明 1000 时最多 (1000 − 136) / 40 = 21 条
        assert_eq!(parse_packed(21, TOMBSTONE_RECORD_BYTES, 1000), Parse::Records(21));
        assert_eq!(parse_packed(22, TOMBSTONE_RECORD_BYTES, 1000), Parse::Corrupt);
        // 敏感性取样点：原点 107、宽 40 时 816 收 817 坏，声明 1000 时 22 收 23 坏
        assert_eq!(parse_packed_from_origin(816, 40, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES), Parse::Records(816));
        assert_eq!(parse_packed_from_origin(817, 40, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES), Parse::Corrupt, "817 × 40 + 107 = 32787 > 32768");
        assert_eq!(parse_packed_from_origin(22, 40, 1000, PACKED_UNIT_HEADER_BYTES), Parse::Records(22));
        assert_eq!(parse_packed_from_origin(23, 40, 1000, PACKED_UNIT_HEADER_BYTES), Parse::Corrupt);
        // 第一次几何的回归（字面 56，原点 107）：583 收、584 坏，声明 1000 时 15 收 16 坏
        assert_eq!(parse_packed_from_origin(583, 56, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES), Parse::Records(583));
        assert_eq!(parse_packed_from_origin(584, 56, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES), Parse::Corrupt, "584 × 56 + 107 = 32811 > 32768");
        assert_eq!(parse_packed_from_origin(15, 56, 1000, PACKED_UNIT_HEADER_BYTES), Parse::Records(15));
        assert_eq!(parse_packed_from_origin(16, 56, 1000, PACKED_UNIT_HEADER_BYTES), Parse::Corrupt);
        // 阳性对照：40 / 140 混装各 10 条，period = 40 / gcd(40,140) = 40 / 20 = 2 ⇒ 错分 5
        assert_eq!(misparsed_when_mixed(TOMBSTONE_RECORD_BYTES, 10, INODE_RECORD_BYTES, 10), 5);
        // 互素宽度：40 与 41，period 40 ⇒ 10 条里只有 i=0 对齐 ⇒ 错 9；48 与 140 gcd 4，period 12 ⇒ 错 9
        assert_eq!(misparsed_when_mixed(40, 10, 41, 10), 9, "5 不是这条式子在任何宽度下的常数");
        assert_eq!(misparsed_when_mixed(48, 10, 140, 10), 9);
        // 第一次几何的回归（字面 56）：56 / 140 错分 5，56 / 57 错 9
        assert_eq!(misparsed_when_mixed(56, 10, 140, 10), 5);
        assert_eq!(misparsed_when_mixed(56, 10, 57, 10), 9);
        // 阴性对照：不混装恒 0
        assert_eq!(misparsed_when_mixed(TOMBSTONE_RECORD_BYTES, 10, TOMBSTONE_RECORD_BYTES, 10), 0);
        assert_eq!(misparsed_when_mixed(INODE_RECORD_BYTES, 3, INODE_RECORD_BYTES, 300), 0);
    }

    /// **F4 阳性对照表**：每个宽度 / 原点旋钮拧一下，输出就是登记第五节表里的数，七行都过。
    #[test]
    fn positive_controls_move_the_output_to_the_registered_values() {
        let controls = positive_controls();
        assert_eq!(controls.len(), 7);
        for control in &controls {
            assert_eq!(control.observed, control.expected, "旋钮 {}", control.knob);
        }
    }

    /// **判据 6 的绝对值**：incompat 静默漏 0 且拒挂；跳过政策静默漏恰 k；未登记记录类型可验不可用。
    #[test]
    fn criterion6_unknown_class_refuses_mount_and_unknown_record_type_is_read_only() {
        let incompat_outcome = mount_with_unknown_class(7, UnknownClassPolicy::Incompat).unwrap();
        assert_eq!(incompat_outcome, MountOutcome { mount_ok: false, silent_missed: 0 });
        let skip_outcome = mount_with_unknown_class(7, UnknownClassPolicy::Skip).unwrap();
        assert_eq!(skip_outcome, MountOutcome { mount_ok: true, silent_missed: 7 });
        // 阴性对照：没有未登记类时两种政策都正常挂、漏 0
        for policy in [UnknownClassPolicy::Incompat, UnknownClassPolicy::Skip] {
            assert_eq!(
                mount_with_unknown_class(0, policy).unwrap(),
                MountOutcome { mount_ok: true, silent_missed: 0 }
            );
        }
        let record_type_outcome = mount_with_unknown_record_type(3);
        assert_eq!(
            record_type_outcome,
            RecordTypeOutcome { containers_verified: 3, records_applied: 0, read_only: true, reclaim_allowed: false }
        );
        assert!(!mount_with_unknown_record_type(0).read_only);
    }

    /// **判据 7 的绝对值**：256 − 7 = 249。
    #[test]
    fn criterion7_headroom_is_counted_not_estimated() {
        assert_eq!(REGISTRY.len(), 7);
        assert_eq!(free_codes(&REGISTRY), 249);
        assert_eq!(free_codes(&REGISTRY_FROM_DECISION_NINE_ROW), 253);
        // 登记表里码不重复、0 恒无效
        let mut codes: Vec<u8> = REGISTRY.iter().map(|registry_entry| registry_entry.0).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), REGISTRY.len());
        assert_eq!(REGISTRY[0], (0, "invalid", Kind::Invalid));
    }

    /// 不合法几何一律 0 / Corrupt，不许当成测量值。
    #[test]
    fn illegal_geometry_is_not_a_measurement() {
        assert_eq!(capacity(UNIT_BYTES, TOMBSTONE_RECORD_BYTES), 0);
        assert_eq!(capacity(PACKED_UNIT_HEADER_BYTES, 0), 0);
        assert_eq!(parse_packed(1, TOMBSTONE_RECORD_BYTES, 10), Parse::Corrupt);
    }
}
