//! # E117 预留位算进去之后的头宽与几何
//!
//! **问的是**：D18 已定项 7 的字段表算出数据单元头 105 字节
//! （共同前缀 42 = magic 4 + 格式版本 2 + flags 2 + 声明长度 2 + 头校验和 32；
//! 类身份段 63 = 五元组 33 + 诞生代号 8 + fsid 8 + 写序 10 + 载荷 CRC 4），
//! **而同一节那句「v1 就预留 nonce 代号与 MAC 的字段位」里的两个位在表外**。
//! 把它们算进去之后，三类单元的头各多宽？下游几何各变多少？
//!
//! **它答的是** C193（加密开启时的单元头有多大，全仓没算过），
//! 也是 D18 已定项 14 定案前用户要的那个数（2026-09-07：「取甲，但先把头宽算清楚再定」）。
//!
//! ## 逐字贴进来的条款
//!
//! - D18 已定项 7：「**v1 就预留 nonce 代号与 MAC 的字段位**」，理由「不预留则开加密成了布局变更」
//!   ⇒ 位在第一版就占着盘上的字节。
//! - D9 已定项 2：MAC **满 128 位**不截断 ⇒ 预留的 MAC 位 = 16 字节。
//! - D9 已定项 4：「指针头部为每个逻辑 extent 存一份 **96 位** nonce」⇒ 完整 nonce = 12 字节。
//! - D18 已定项 14 臂甲（等用户定案）：nonce 代号就是完整 nonce ⇒ 代号位 = 12 字节。
//! - D18 已定项 7 补注：E73 的三档下界 58 / 67 / 76 各加 10 成 **68 / 77 / 86**。
//!
//! ## 跑前写死的判据（跑完一个字都没改）
//!
//! 1. **头宽闭式**：每类头 = 表内字段 + 预留位（nonce 代号 + MAC 16）。逐类钉绝对值。
//! 2. **码 2 扇出** = `(16384 − 头) / 条目宽`。条目宽取两个：E73 自己的 54
//!    （key 22 + 子指针 32）与按 D19 已定项 4 更新后的 81（key 22 + 指针 59）。逐格钉绝对值。
//! 3. **码 3 每容器记录数** = `(32768 − 头) / 140`。逐格钉绝对值。
//! 4. **码 1 净荷** = `32768 − 头`。逐格钉绝对值。
//! 5. **判别力对照**：`resv12`（代号 12）与 `resv4`（代号 4）必须**至少在一格上不同**。
//!    全格相同 ⇒ 判「取样点不敏感」（`.claude/rules/mutation-sampling.md` 第三类），
//!    要去找一个跨整数边界的取样点，**不许记成「预留位不影响几何」**。
//! 6. **阳性对照，三条臂各跑一遍**：头强制为 0 时扇出 = 单元 / 条目宽、记录数 = 32768 / 140。
//!    对照不成立 ⇒ 整轮作废。
//! 7. **阴性对照**：条目宽 > 可用字节 ⇒ 扇出 0（不是「扇出 0 条」，是这组参数不合法）。
//!
//! **失败条款**：判据 5 不成立时不许改判据，按第三类处置补取样点。
//!
//! ## 它答不了的
//!
//! 纯算术，没有 I/O、没有崩溃点重放。不答「E73 的子指针宽度 32 该不该改成 59」——
//! 那是 C89 的账，这里只把两个宽度都摆出来。也不重算 E97 / E98 / E103 的内部模型，
//! 只报它们承重的那几个几何数在新头宽下是多少。

/// D8 已定项 2：节点 16 KiB。
const NODE_BYTES: u64 = 16384;
/// D4 已定项 5：数据单元恒 32768 含头。
const DATA_UNIT_BYTES: u64 = 32768;
/// D18 已定项 7 共同前缀：magic 4 + 格式版本 2 + flags 2 + 声明长度 2 + 头校验和 32。
const COMMON_PREFIX: u64 = 42;
/// D18 已定项 7 数据单元类身份段：五元组 33 + 诞生代号 8 + fsid 8 + 写序 10 + 载荷 CRC 4。
const DATA_IDENTITY: u64 = 63;
/// D18 已定项 11：打包记录单元头（表内部分）。
const PACKED_HEADER_BARE_BYTES: u64 = 103;
/// D18 已定项 7 补注：码 2 三档基础头（58 / 67 / 76 各加写序 10）。
const NODE_HEADER_BYTES_BY_TIER: [u64; 3] = [68, 77, 86];
/// D9 已定项 2：MAC 满 128 位不截断。
const MAC_BYTES: u64 = 16;
/// E98 的 format-const：inode 记录定长。
const INODE_RECORD_BYTES: u64 = 140;
/// E73 自己的条目宽：key 22 + 子指针 32。
const E73_ENTRY: u64 = 54;
/// 按 D19 已定项 4 更新后的条目宽：key 22 + 指针 59。
const UPDATED_POINTER_ENTRY_BYTES: u64 = 22 + 59;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 头不含预留位——今天四个下游实验用的口径。
    Bare,
    /// 含预留位，nonce 代号 12（D18 已定项 14 臂甲）。
    ReservedTwelveByteNonceCode,
    /// 含预留位，nonce 代号 4——**判别力对照臂，不是候选**。
    ReservedFourByteNonceCode,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Bare => "bare",
            Arm::ReservedTwelveByteNonceCode => "resv12",
            Arm::ReservedFourByteNonceCode => "resv4",
        }
    }
    /// 预留位一共多少字节。**没有通配臂**：加一种臂编译器会报。
    fn reserved(self) -> u64 {
        match self {
            Arm::Bare => 0,
            Arm::ReservedTwelveByteNonceCode => 12 + MAC_BYTES,
            Arm::ReservedFourByteNonceCode => 4 + MAC_BYTES,
        }
    }
}

/// 扇出 = `(单元 − 头) / 条目宽`。头装不下时返回 0——**不是「扇出 0 条」，是参数不合法**。
fn fanout(unit: u64, header: u64, entry: u64) -> u64 {
    if unit <= header || entry == 0 {
        return 0;
    }
    (unit - header) / entry
}

fn data_unit_header_bytes(arm: Arm) -> u64 {
    COMMON_PREFIX + DATA_IDENTITY + arm.reserved()
}
fn packed_header_bytes(arm: Arm) -> u64 {
    PACKED_HEADER_BARE_BYTES + arm.reserved()
}
fn node_header_bytes(arm: Arm, tier: usize) -> u64 {
    NODE_HEADER_BYTES_BY_TIER[tier] + arm.reserved()
}

fn main() {
    // 结果行必须带 E7RESULT 前缀并以 `name=done emitted=N` 收尾——
    // 抓取方按这个数比对条数，对不上整轮作废（command-safety.md「结果抓取要有完整性闸」）。
    let mut emitter = e7_index_bench::Emitter::new();
    let arms = [Arm::Bare, Arm::ReservedTwelveByteNonceCode, Arm::ReservedFourByteNonceCode];
    let mut line = |result_line: String| {
        println!("{}", emitter.emit_raw(&result_line));
    };

    for arm in arms {
        line(format!(
            "name=hdr arm={} reserved={} data_hdr={} packed_hdr={} payload={}",
            arm.name(),
            arm.reserved(),
            data_unit_header_bytes(arm),
            packed_header_bytes(arm),
            DATA_UNIT_BYTES - data_unit_header_bytes(arm)
        ));
        for (tier, _) in NODE_HEADER_BYTES_BY_TIER.iter().enumerate() {
            line(format!(
                "name=node arm={} tier={} node_hdr={} fanout54={} fanout81={}",
                arm.name(),
                tier,
                node_header_bytes(arm, tier),
                fanout(NODE_BYTES, node_header_bytes(arm, tier), E73_ENTRY),
                fanout(NODE_BYTES, node_header_bytes(arm, tier), UPDATED_POINTER_ENTRY_BYTES)
            ));
        }
        line(format!(
            "name=packed arm={} records={}",
            arm.name(),
            fanout(DATA_UNIT_BYTES, packed_header_bytes(arm), INODE_RECORD_BYTES)
        ));
        // 阳性对照，逐臂跑：头强制为 0
        line(format!(
            "name=poscontrol arm={} fanout54={} fanout81={} records={}",
            arm.name(),
            fanout(NODE_BYTES, 0, E73_ENTRY),
            fanout(NODE_BYTES, 0, UPDATED_POINTER_ENTRY_BYTES),
            fanout(DATA_UNIT_BYTES, 0, INODE_RECORD_BYTES)
        ));
    }
    // 阴性对照：条目宽大于可用字节
    line(format!(
        "name=negcontrol fanout={}",
        fanout(NODE_BYTES, 100, NODE_BYTES)
    ));
    // 判别力：resv12 与 resv4 有没有一格不同
    let mut differing_cells = 0u64;
    for (tier, _) in NODE_HEADER_BYTES_BY_TIER.iter().enumerate() {
        if fanout(NODE_BYTES, node_header_bytes(Arm::ReservedTwelveByteNonceCode, tier), E73_ENTRY)
            != fanout(NODE_BYTES, node_header_bytes(Arm::ReservedFourByteNonceCode, tier), E73_ENTRY)
        {
            differing_cells += 1;
        }
        if fanout(NODE_BYTES, node_header_bytes(Arm::ReservedTwelveByteNonceCode, tier), UPDATED_POINTER_ENTRY_BYTES)
            != fanout(NODE_BYTES, node_header_bytes(Arm::ReservedFourByteNonceCode, tier), UPDATED_POINTER_ENTRY_BYTES)
        {
            differing_cells += 1;
        }
    }
    if fanout(DATA_UNIT_BYTES, packed_header_bytes(Arm::ReservedTwelveByteNonceCode), INODE_RECORD_BYTES)
        != fanout(DATA_UNIT_BYTES, packed_header_bytes(Arm::ReservedFourByteNonceCode), INODE_RECORD_BYTES)
    {
        differing_cells += 1;
    }
    line(format!("name=discriminate cells_differing={differing_cells}"));
    drop(line);
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判据 1：头宽逐类钉绝对值。**写成加法不写减法**——变异把常量改大时
    /// 减法会编译期溢出、被记成无效变异（test-discipline 那条坑）。
    #[test]
    fn head_widths_absolute() {
        assert_eq!(COMMON_PREFIX + DATA_IDENTITY, 105);
        assert_eq!(data_unit_header_bytes(Arm::Bare), 105);
        assert_eq!(data_unit_header_bytes(Arm::ReservedTwelveByteNonceCode), 105 + 28);
        assert_eq!(data_unit_header_bytes(Arm::ReservedFourByteNonceCode), 105 + 20);
        assert_eq!(packed_header_bytes(Arm::Bare), 103);
        assert_eq!(packed_header_bytes(Arm::ReservedTwelveByteNonceCode), 103 + 28);
    }

    /// 判据 1：预留位的构成钉死。
    #[test]
    fn reserved_composition() {
        assert_eq!(Arm::Bare.reserved(), 0);
        assert_eq!(Arm::ReservedTwelveByteNonceCode.reserved(), 12 + 16);
        assert_eq!(Arm::ReservedFourByteNonceCode.reserved(), 4 + 16);
        assert_eq!(MAC_BYTES, 16);
    }

    /// 判据 2：码 2 扇出逐格钉绝对值（E73 口径，条目 54）。
    ///
    /// ⚠️ **跑前登记的值判否，留档不改**：判据写的是「Bare 三档全是 302」，
    /// 实测 **302 / 301 / 301**——第二档 `(16384 − 77) / 54 = 301.05` 就已经掉一格。
    /// 登记那个值是主 agent 手算时只算了第一档就外推的，第二次改判时又只改了第三档、
    /// 还是没算第二档。**两次都错在同一个毛病：拿一个取样点当三个。**
    /// E73 自己的说法是「三档结论一致（掉 0 或 1 格）」，与实测不冲突；
    /// 冲突的是本实验跑前那句更强的登记。
    #[test]
    fn node_fanout_e73_entry() {
        let node_fanout_for = |arm: Arm, tier: usize| fanout(NODE_BYTES, node_header_bytes(arm, tier), E73_ENTRY);
        const PREREG_BARE: [u64; 3] = [302, 302, 302];
        const MEASURED_BARE: [u64; 3] = [302, 301, 301];
        assert_ne!(PREREG_BARE, MEASURED_BARE, "跑前登记若与实测相等，上面那段注释就该删掉");
        for (tier, expected) in MEASURED_BARE.iter().enumerate() {
            assert_eq!(node_fanout_for(Arm::Bare, tier), *expected, "bare tier {tier}");
        }
        for (tier, expected) in [301u64, 301, 301].iter().enumerate() {
            assert_eq!(node_fanout_for(Arm::ReservedTwelveByteNonceCode, tier), *expected, "resv12 tier {tier}");
        }
    }

    /// 判据 2：按 D19 已定项 4 的指针宽度重算（条目 81）。
    /// **这一格才是有判别力的那一格**——见 t09。
    #[test]
    fn node_fanout_at_updated_pointer_entry() {
        let node_fanout_for = |arm: Arm, tier: usize| fanout(NODE_BYTES, node_header_bytes(arm, tier), UPDATED_POINTER_ENTRY_BYTES);
        assert_eq!(UPDATED_POINTER_ENTRY_BYTES, 81);
        for (tier, expected) in [201u64, 201, 201].iter().enumerate() {
            assert_eq!(node_fanout_for(Arm::Bare, tier), *expected, "bare tier {tier}");
        }
        for (tier, expected) in [201u64, 200, 200].iter().enumerate() {
            assert_eq!(node_fanout_for(Arm::ReservedTwelveByteNonceCode, tier), *expected, "resv12 tier {tier}");
        }
        for (tier, expected) in [201u64, 201, 200].iter().enumerate() {
            assert_eq!(node_fanout_for(Arm::ReservedFourByteNonceCode, tier), *expected, "resv4 tier {tier}");
        }
    }

    /// 判据 3：码 3 每容器记录数逐格钉绝对值。
    #[test]
    fn packed_records_absolute() {
        let records_per_container_for = |arm: Arm| fanout(DATA_UNIT_BYTES, packed_header_bytes(arm), INODE_RECORD_BYTES);
        assert_eq!(records_per_container_for(Arm::Bare), 233);
        assert_eq!(records_per_container_for(Arm::ReservedTwelveByteNonceCode), 233);
        assert_eq!(records_per_container_for(Arm::ReservedFourByteNonceCode), 233);
    }

    /// 判据 4：码 1 净荷逐格钉绝对值。
    #[test]
    fn payload_absolute() {
        assert_eq!(DATA_UNIT_BYTES, data_unit_header_bytes(Arm::Bare) + 32663);
        assert_eq!(DATA_UNIT_BYTES, data_unit_header_bytes(Arm::ReservedTwelveByteNonceCode) + 32635);
    }

    /// 判据 6：阳性对照，**三条臂各跑一遍**——头为 0 时扇出必须等于闭式值。
    #[test]
    fn positive_control_on_every_arm() {
        for arm in [Arm::Bare, Arm::ReservedTwelveByteNonceCode, Arm::ReservedFourByteNonceCode] {
            assert_eq!(fanout(NODE_BYTES, 0, E73_ENTRY), 303, "{}", arm.name());
            assert_eq!(fanout(NODE_BYTES, 0, UPDATED_POINTER_ENTRY_BYTES), 202, "{}", arm.name());
            assert_eq!(fanout(DATA_UNIT_BYTES, 0, INODE_RECORD_BYTES), 234, "{}", arm.name());
        }
    }

    /// 判据 7：阴性对照——条目宽大于可用字节时返回 0，且头装不下时也返回 0。
    #[test]
    fn negative_control() {
        assert_eq!(fanout(NODE_BYTES, 100, NODE_BYTES), 0);
        assert_eq!(fanout(NODE_BYTES, NODE_BYTES, E73_ENTRY), 0);
        assert_eq!(fanout(NODE_BYTES, node_header_bytes(Arm::Bare, 0), 0), 0);
    }

    /// 判据 5：判别力——resv12 与 resv4 必须至少在一格上不同。
    /// **在 E73 的条目宽 54 上它们逐格相同**（向下取整吃掉那 8 字节差），
    /// 而在按 D19 已定项 4 更新后的条目宽 81 上**第二档就分开**：resv12 给 200、resv4 给 201。
    /// ⇒ 判据 5 由这一格满足，不必造人造取样点。
    #[test]
    fn discriminating_sample_point() {
        assert_eq!(node_header_bytes(Arm::ReservedTwelveByteNonceCode, 1), 105);
        assert_eq!(node_header_bytes(Arm::ReservedFourByteNonceCode, 1), 97);
        assert_eq!(fanout(NODE_BYTES, node_header_bytes(Arm::ReservedTwelveByteNonceCode, 1), UPDATED_POINTER_ENTRY_BYTES), 200);
        assert_eq!(fanout(NODE_BYTES, node_header_bytes(Arm::ReservedFourByteNonceCode, 1), UPDATED_POINTER_ENTRY_BYTES), 201);
    }

    /// 判据 5 的反面：在 **E73 的条目宽 54** 上两条臂逐格相同。
    /// 这不是「预留位不影响几何」，是**取样点不敏感**
    /// （`.claude/rules/mutation-sampling.md` 第三类）——判别力靠 t09 那一格。
    #[test]
    fn sampling_insensitive_at_e73_entry() {
        for (tier, _) in NODE_HEADER_BYTES_BY_TIER.iter().enumerate() {
            assert_eq!(
                fanout(NODE_BYTES, node_header_bytes(Arm::ReservedTwelveByteNonceCode, tier), E73_ENTRY),
                fanout(NODE_BYTES, node_header_bytes(Arm::ReservedFourByteNonceCode, tier), E73_ENTRY),
                "tier {tier}"
            );
        }
        assert_eq!(
            fanout(DATA_UNIT_BYTES, packed_header_bytes(Arm::ReservedTwelveByteNonceCode), INODE_RECORD_BYTES),
            fanout(DATA_UNIT_BYTES, packed_header_bytes(Arm::ReservedFourByteNonceCode), INODE_RECORD_BYTES)
        );
    }

    /// 格式常量必须与 kb 的 format-const 标记一致。
    #[test]
    fn format_constants() {
        assert_eq!(NODE_BYTES, 16384);
        assert_eq!(DATA_UNIT_BYTES, 32768);
        assert_eq!(INODE_RECORD_BYTES, 140);
        assert_eq!(PACKED_HEADER_BARE_BYTES, 103);
    }

    /// 三档基础头就是 D18 已定项 7 补注那三个数。
    #[test]
    fn node_tiers() {
        assert_eq!(NODE_HEADER_BYTES_BY_TIER, [68, 77, 86]);
        assert_eq!(node_header_bytes(Arm::ReservedTwelveByteNonceCode, 0), 96);
        assert_eq!(node_header_bytes(Arm::ReservedTwelveByteNonceCode, 2), 114);
    }
}
