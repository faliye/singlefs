//! E76：载荷校验和的判别力 —— D23 已定项 13（载荷完整性取候选乙）欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md「把定义句原样贴进实验注释」）
//!
//! - D23 已定项 13（2026-09-01 用户定案）：「**`header_csum` 保持只覆盖记录头，
//!   另加一个覆盖点名项数组的校验和**，算法取 CRC32C（与已定项 11 的反向链同一个，不引入第二种）。」
//!   并逐字记：候选甲（一个校验和覆盖整条记录）被挡掉的理由是
//!   「**会失去『头落尾没落』的可分辨性，而 `header_csum` 只覆盖头正是为那个目的定的**」。
//! - D23 已定项 4：头 **78 字节**；⚠️「两笔已定、待落地的增量不在 78 里：
//!   已定项 7 的事务号 + 提交标记 **9 字节**、已定项 8 的反向链 **4 字节** ⇒ 落地后是 **91 字节**」。
//!   加上已定项 13 这第三笔（载荷校验和 4 字节）⇒ **95 字节**（E75 已量）。
//! - E23 字段表：`header_csum` 宽 **32 字节**，依据逐字是「只覆盖头部——『头落尾没落』可区分的前提」；
//!   点名项宽 **56 字节**。
//! - `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验路径本身也要证明它会红」：
//!   **不注入故障就分不开「两条都对」与「第二条根本没在看」，而它们看起来一模一样。**
//!
//! ## 判据（E76 正文跑前写死，跑完不许改）
//!
//! 四行必须**逐格**符合。任一格不符，候选乙不成立，回 D23 已定项 13 重挑。
//!
//! | 构造 | `header_csum` 应判 | 载荷校验和应判 |
//! |---|---|---|
//! | 整条完整落盘 | 过 | 过 |
//! | **头落了、载荷只落了一半** | **过** | **不过** |
//! | 头被改了一个字节 | 不过 | 过 |
//! | 载荷被改了一个字节 | 过 | 不过 |
//!
//! ## 失败条款
//!
//! - 第二行是承重的那一行——它正是「头落尾没落可分辨」这个已定目的的唯一体现。
//!   若 `header_csum` 在那一行判不过，说明它其实覆盖了载荷，那是候选甲不是候选乙，**整轮作废**。
//! - **不许只跑第一行**：只造完整记录，两个校验和都过，什么也没证明。
//!
//! ## ⚠️ 「载荷只落了一半」必须建成残留字节，不能建成短缓冲
//!
//! 建成短缓冲的话，**长度检查一条就把它挡了，校验和一个字节都没读**——
//! 那时记下的「载荷校验和抓到了」是假的，抓到它的是 `record_length`。
//! journal 是**定长环**（D23 已定项 2），没落到的那半截是**上一圈的残留字节**，不是空洞。
//! ⇒ 主臂按残留字节建模；短缓冲另立一臂并**如实记明它由长度检查拦下**，
//! 好把两条路径各自的功劳分开。
//!
//! ## 它答不了的
//!
//! 没有 journal 实现、没有块层、没有崩溃点重放：**这是一个记录编解码模型**，文件操作 0 处。
//! 它验的是「这两个字段的覆盖范围能不能分辨这四类构造」，
//! **不验**「真实掉电会不会恰好造出第二行那种半截记录」——那要崩溃点重放，仓里还没有。
//! `header_csum` 的**摘要函数** D23 没定（只定了宽度 32 字节与覆盖范围），
//! 所以四行判决对两个不同的摘要函数各跑一遍，证明判决不依赖函数选择。

use e7_index_bench::Emitter;

/// D23 已定项 4 的现行头部字节数。**格式常量**，与 kb 的 format-const 标记绑定。
const JOURNAL_HDR: u64 = 78;
/// 已定项 7 的事务号 + 提交标记。
const INC_TXN_BOUNDARY: u64 = 9;
/// 已定项 8 的反向链，32 位。
const INC_BACK_CHAIN: u64 = 4;
/// 已定项 13 的载荷校验和，CRC32C 32 位。
const INC_PAYLOAD_CSUM: u64 = 4;
/// 三笔增量落地后的头部宽度，E75 已量。
const HDR: usize = (JOURNAL_HDR + INC_TXN_BOUNDARY + INC_BACK_CHAIN + INC_PAYLOAD_CSUM) as usize;
/// E23 字段表的点名项宽度。
const ITEM: usize = 56;

// ── 头部字段落点。E23 字段表的 10 个字段 + 三笔已定增量，合计 95。 ──
const OFF_MAGIC: usize = 0; //   4
const OFF_RECORD_LENGTH: usize = 8; //   4
const OFF_NAMED_COUNT: usize = 12; //   4
const OFF_JSN: usize = 16; //  10（已定项 9：实例代号 32 位 + 计数器 48 位）
const OFF_TXN_ID: usize = 46; //   8（已定项 7）
const OFF_COMMIT_FLAG: usize = 54; //   1（已定项 7）
const OFF_PREV_HASH: usize = 55; //   4（已定项 8）
const OFF_PAYLOAD_CSUM: usize = 59; //   4（已定项 13）
/// `header_csum` 住头部末尾 32 字节 ⇒ 它覆盖的是它自己之前的那 63 字节。
const OFF_HEADER_CSUM: usize = 63; //  32
const HEADER_CSUM_LEN: usize = 32;

const MAGIC: [u8; 4] = *b"SFJR";

/// 完整 32 位 CRC32C（Castagnoli），反射多项式 0x82F63B78。
/// 与 `e61_chain_hash.rs` 的实现同形，**故意各写一份**——共享一份就成了同一段代码。
fn crc32c(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0x82F6_3B78 } else { crc >> 1 };
        }
    }
    !crc
}

/// 第二个摘要函数（FNV-1a 64 取低 32 位），只用来证明**四行判决不依赖摘要函数选择**。
/// 它不是候选，D23 已定项 13 定的算法是 CRC32C。
fn fnv1a32(data: &[u8]) -> u32 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    (h ^ (h >> 32)) as u32
}

/// `header_csum` 覆盖的范围：它自己之前那 63 字节（含 `payload_csum` 那 4 字节）。
/// **编码与校验共用这一个定义**——两边各写一遍的话，改一边就成了
/// 「编码与校验口径不一致」，那是另一个 bug，不是本实验要测的覆盖范围。
fn header_covered(buf: &[u8]) -> &[u8] {
    &buf[..OFF_HEADER_CSUM]
}

/// 载荷校验和覆盖的范围：点名项数组，即头之后到自述长度为止。同样两边共用。
fn payload_covered(buf: &[u8], declared: usize) -> &[u8] {
    &buf[HDR..declared]
}

/// 摘要函数的身份。**判决对两个都要成立**，否则结论是函数的产物不是覆盖范围的产物。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Digest {
    Crc32c,
    Fnv1a,
}

impl Digest {
    fn of(self, data: &[u8]) -> u32 {
        match self {
            // 没有 `_ =>` 通配臂：加第三个函数时这里编译不过（machine-first.md 教条一）
            Digest::Crc32c => crc32c(data),
            Digest::Fnv1a => fnv1a32(data),
        }
    }
    fn tag(self) -> &'static str {
        match self {
            Digest::Crc32c => "crc32c",
            Digest::Fnv1a => "fnv1a",
        }
    }
}

/// 被测的两个候选。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scheme {
    /// 候选乙（D23 已定项 13 取的）：`header_csum` 只覆盖头，另有一个覆盖点名项数组的校验和。
    Yi,
    /// 候选甲（被挡掉的）：一个校验和覆盖整条记录。**它是阳性对照**——
    /// 它必须在第二行上失去可分辨性，否则本实验分不出甲乙，整轮作废。
    Jia,
}

/// 四类构造。判据表的四行，**一行都不许省**。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Construction {
    /// 第一行：整条完整落盘。
    Intact,
    /// 第二行（承重）：头落了、载荷只落了一半，没落的那半是**上一圈的残留字节**。
    HalfPayloadStale,
    /// 第三行：头被改了一个字节。
    HeaderBitFlip,
    /// 第四行：载荷被改了一个字节。
    PayloadBitFlip,
    /// 附加臂：载荷被**截短**（短缓冲）。它由长度检查拦下，功劳不算给校验和。
    PayloadTruncatedShort,
}

impl Construction {
    fn tag(self) -> &'static str {
        match self {
            Construction::Intact => "row1_intact",
            Construction::HalfPayloadStale => "row2_half_payload_stale",
            Construction::HeaderBitFlip => "row3_header_bitflip",
            Construction::PayloadBitFlip => "row4_payload_bitflip",
            Construction::PayloadTruncatedShort => "extra_payload_truncated_short",
        }
    }
}

/// 校验的结果。**三态，不是两态**：长度就不对时校验和根本没被算过，
/// 记成「不过」会把长度检查的功劳算到校验和头上。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verdict {
    Pass,
    Fail,
    /// 长度检查先拦下了，校验和没被计算。
    NotReached,
}

impl Verdict {
    fn tag(self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Fail => "fail",
            Verdict::NotReached => "not_reached",
        }
    }
}

/// 造一条完好的记录：头 95 字节 + `named` 个 56 字节点名项。
/// `stale` 是这块环空间上一圈的残留字节，落盘不全时露出来的就是它。
fn encode(named: u32, jsn: u64, scheme: Scheme, d: Digest) -> Vec<u8> {
    let payload_len = named as usize * ITEM;
    let mut rec = vec![0u8; HDR + payload_len];

    rec[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC);
    rec[OFF_RECORD_LENGTH..OFF_RECORD_LENGTH + 4]
        .copy_from_slice(&((HDR + payload_len) as u32).to_le_bytes());
    rec[OFF_NAMED_COUNT..OFF_NAMED_COUNT + 4].copy_from_slice(&named.to_le_bytes());
    rec[OFF_JSN..OFF_JSN + 8].copy_from_slice(&jsn.to_le_bytes());
    rec[OFF_TXN_ID..OFF_TXN_ID + 8].copy_from_slice(&(jsn ^ 0x5A5A).to_le_bytes());
    rec[OFF_COMMIT_FLAG] = 1;
    rec[OFF_PREV_HASH..OFF_PREV_HASH + 4].copy_from_slice(&(jsn as u32).wrapping_mul(2654435761).to_le_bytes());

    // 点名项数组：每项 56 字节，内容与 jsn 绑定，好让不同记录的载荷真的不同。
    for i in 0..named as usize {
        for k in 0..ITEM {
            rec[HDR + i * ITEM + k] = (jsn as u8).wrapping_add((i * ITEM + k) as u8);
        }
    }

    match scheme {
        Scheme::Yi => {
            // 载荷校验和覆盖点名项数组，写进头里那 4 字节。
            let n = rec.len();
            let pc = d.of(payload_covered(&rec, n));
            rec[OFF_PAYLOAD_CSUM..OFF_PAYLOAD_CSUM + 4].copy_from_slice(&pc.to_le_bytes());
            // header_csum 覆盖它自己之前的 63 字节（**含刚写进去的载荷校验和**）。
            let hc = d.of(header_covered(&rec));
            rec[OFF_HEADER_CSUM..OFF_HEADER_CSUM + 4].copy_from_slice(&hc.to_le_bytes());
        }
        Scheme::Jia => {
            // 候选甲：一个校验和覆盖整条记录（头里那 32 字节先留空再算）。
            let whole = d.of(&[&rec[..OFF_HEADER_CSUM], &rec[OFF_HEADER_CSUM + HEADER_CSUM_LEN..]].concat());
            rec[OFF_HEADER_CSUM..OFF_HEADER_CSUM + 4].copy_from_slice(&whole.to_le_bytes());
        }
    }
    rec
}

/// 把一条记录按某个构造摆到介质上。介质预先填了残留字节。
fn lay_on_medium(rec: &[u8], c: Construction, stale: u8) -> Vec<u8> {
    let mut m = vec![stale; rec.len()];
    match c {
        Construction::Intact => m.copy_from_slice(rec),
        Construction::HalfPayloadStale => {
            // 头全落了，载荷只落了一半，剩下一半是残留字节（介质初值）。
            let half = HDR + (rec.len() - HDR) / 2;
            m[..half].copy_from_slice(&rec[..half]);
        }
        Construction::HeaderBitFlip => {
            m.copy_from_slice(rec);
            m[OFF_JSN] ^= 0x01; // 头里的一个字节
        }
        Construction::PayloadBitFlip => {
            m.copy_from_slice(rec);
            m[HDR] ^= 0x01; // 载荷第一个字节
        }
        Construction::PayloadTruncatedShort => {
            m.copy_from_slice(rec);
            m.truncate(HDR + (rec.len() - HDR) / 2); // 真的短了，长度检查看得见
        }
    }
    m
}

/// 校验：先长度、再头、再载荷。返回 (header_csum 判决, 载荷校验和判决)。
fn verify(m: &[u8], scheme: Scheme, d: Digest) -> (Verdict, Verdict) {
    if m.len() < HDR {
        return (Verdict::NotReached, Verdict::NotReached);
    }
    let declared =
        u32::from_le_bytes(m[OFF_RECORD_LENGTH..OFF_RECORD_LENGTH + 4].try_into().unwrap()) as usize;

    match scheme {
        Scheme::Yi => {
            let stored_hc =
                u32::from_le_bytes(m[OFF_HEADER_CSUM..OFF_HEADER_CSUM + 4].try_into().unwrap());
            let hv = if d.of(header_covered(m)) == stored_hc { Verdict::Pass } else { Verdict::Fail };
            // 载荷校验和只有在长度自洽时才算得动——长度不对是**另一条路径**拦的。
            let pv = if m.len() < declared {
                Verdict::NotReached
            } else {
                let stored_pc =
                    u32::from_le_bytes(m[OFF_PAYLOAD_CSUM..OFF_PAYLOAD_CSUM + 4].try_into().unwrap());
                if d.of(payload_covered(m, declared)) == stored_pc { Verdict::Pass } else { Verdict::Fail }
            };
            (hv, pv)
        }
        Scheme::Jia => {
            // 候选甲只有一个校验和。它既是「头的判决」也是「载荷的判决」——
            // **这正是它失去可分辨性的形态**，两列必然相同。
            if m.len() < declared {
                return (Verdict::NotReached, Verdict::NotReached);
            }
            let stored =
                u32::from_le_bytes(m[OFF_HEADER_CSUM..OFF_HEADER_CSUM + 4].try_into().unwrap());
            let got = d.of(&[&m[..OFF_HEADER_CSUM], &m[OFF_HEADER_CSUM + HEADER_CSUM_LEN..declared]].concat());
            let v = if got == stored { Verdict::Pass } else { Verdict::Fail };
            (v, v)
        }
    }
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config hdr={HDR} item={ITEM} hdr_csum_off={OFF_HEADER_CSUM} \
             payload_csum_off={OFF_PAYLOAD_CSUM} model=encoder file_ops=0"
        ))
    );

    // CRC32C 的公开测试向量：先证明摘要函数本身没写错。
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=digest_vector crc32c_123456789={:#010x} crc32c_empty={:#010x}",
            crc32c(b"123456789"),
            crc32c(b"")
        ))
    );

    // ── 判据表：四行 × 两个候选 × 两个摘要函数 ──────────────────────────
    for d in [Digest::Crc32c, Digest::Fnv1a] {
        for scheme in [Scheme::Yi, Scheme::Jia] {
            for c in [
                Construction::Intact,
                Construction::HalfPayloadStale,
                Construction::HeaderBitFlip,
                Construction::PayloadBitFlip,
                Construction::PayloadTruncatedShort,
            ] {
                // 12 项 = D25 目标负载（E45 口径）。残留字节取 0xEE，与任何真实载荷都不同。
                let rec = encode(12, 0x0102_0304, scheme, d);
                let m = lay_on_medium(&rec, c, 0xEE);
                let (hv, pv) = verify(&m, scheme, d);
                let scheme_tag = match scheme {
                    Scheme::Yi => "yi",
                    Scheme::Jia => "jia",
                };
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=verdict digest={} scheme={scheme_tag} construction={} \
                         header_csum={} payload_csum={} distinguishable={}",
                        d.tag(),
                        c.tag(),
                        hv.tag(),
                        pv.tag(),
                        u8::from(hv != pv)
                    ))
                );
            }
        }
    }

    // ── 承重那一行单独再报一次，含它的机制读数 ────────────────────────────
    for d in [Digest::Crc32c, Digest::Fnv1a] {
        let rec = encode(12, 0x0102_0304, Scheme::Yi, d);
        let m = lay_on_medium(&rec, Construction::HalfPayloadStale, 0xEE);
        let (hv, pv) = verify(&m, Scheme::Yi, d);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=load_bearing_row2 digest={} header_pass={} payload_fail={} \
                 verdict_ok={}",
                d.tag(),
                u8::from(hv == Verdict::Pass),
                u8::from(pv == Verdict::Fail),
                u8::from(hv == Verdict::Pass && pv == Verdict::Fail)
            ))
        );
    }

    // ── 阳性对照：候选甲必须在第二行失去可分辨性 ──────────────────────────
    let rec = encode(12, 0x0102_0304, Scheme::Jia, Digest::Crc32c);
    let m = lay_on_medium(&rec, Construction::HalfPayloadStale, 0xEE);
    let (hv, pv) = verify(&m, Scheme::Jia, Digest::Crc32c);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=positive_control_jia_loses_it header={} payload={} distinguishable={} \
             expect_distinguishable=0",
            hv.tag(),
            pv.tag(),
            u8::from(hv != pv)
        ))
    );

    // ── 扫点名项数：判决不许随载荷长度变 ─────────────────────────────────
    for named in [1u32, 2, 12, 71] {
        let rec = encode(named, 0x0102_0304, Scheme::Yi, Digest::Crc32c);
        let m = lay_on_medium(&rec, Construction::HalfPayloadStale, 0xEE);
        let (hv, pv) = verify(&m, Scheme::Yi, Digest::Crc32c);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=sweep_named named={named} bytes={} header={} payload={}",
                rec.len(),
                hv.tag(),
                pv.tag()
            ))
        );
    }

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言**：CRC32C 的公开测试向量。摘要函数写错的话，下面全部判决都是假的。
    #[test]
    fn absolute_crc32c_matches_published_vector() {
        assert_eq!(crc32c(b"123456789"), 0xE306_9283);
        assert_eq!(crc32c(b""), 0x0000_0000);
    }

    /// **绝对值断言**：头部宽度与字段落点。95 = 78 + 9 + 4 + 4，
    /// 且 `header_csum` 之前恰好 63 字节。
    #[test]
    fn absolute_header_layout() {
        assert_eq!(HDR, 95);
        assert_eq!(OFF_HEADER_CSUM, 63);
        assert_eq!(OFF_HEADER_CSUM + HEADER_CSUM_LEN, HDR);
        assert_eq!(OFF_PAYLOAD_CSUM + 4, OFF_HEADER_CSUM, "载荷校验和紧挨着头校验和");
        assert_eq!(ITEM, 56);
    }

    /// **绝对值断言**：12 项记录恰好 95 + 672 = 767 字节。
    #[test]
    fn absolute_record_bytes() {
        assert_eq!(encode(12, 1, Scheme::Yi, Digest::Crc32c).len(), 767);
        assert_eq!(12 * ITEM, 672);
        assert_eq!(encode(0, 1, Scheme::Yi, Digest::Crc32c).len(), 95);
    }

    /// **判据表第一行**：整条完整落盘 ⇒ 两个都过。
    #[test]
    fn row1_intact_both_pass() {
        for d in [Digest::Crc32c, Digest::Fnv1a] {
            let rec = encode(12, 7, Scheme::Yi, d);
            let m = lay_on_medium(&rec, Construction::Intact, 0xEE);
            assert_eq!(verify(&m, Scheme::Yi, d), (Verdict::Pass, Verdict::Pass), "{d:?}");
        }
    }

    /// **判据表第二行（承重）**：头落了、载荷只落了一半 ⇒ 头**过**、载荷**不过**。
    /// 头判不过就说明 `header_csum` 其实覆盖了载荷 ⇒ 那是候选甲，整轮作废。
    #[test]
    fn row2_half_payload_header_passes_payload_fails() {
        for d in [Digest::Crc32c, Digest::Fnv1a] {
            let rec = encode(12, 7, Scheme::Yi, d);
            let m = lay_on_medium(&rec, Construction::HalfPayloadStale, 0xEE);
            let (hv, pv) = verify(&m, Scheme::Yi, d);
            assert_eq!(hv, Verdict::Pass, "头必须过，否则它覆盖了载荷（{d:?}）");
            assert_eq!(pv, Verdict::Fail, "载荷必须不过（{d:?}）");
        }
    }

    /// **判据表第三行**：头被改一个字节 ⇒ 头不过、载荷过。
    #[test]
    fn row3_header_bitflip_header_fails_payload_passes() {
        for d in [Digest::Crc32c, Digest::Fnv1a] {
            let rec = encode(12, 7, Scheme::Yi, d);
            let m = lay_on_medium(&rec, Construction::HeaderBitFlip, 0xEE);
            assert_eq!(verify(&m, Scheme::Yi, d), (Verdict::Fail, Verdict::Pass), "{d:?}");
        }
    }

    /// **判据表第四行**：载荷被改一个字节 ⇒ 头过、载荷不过。
    /// 这一行证明载荷校验和不是在回声头那一份。
    #[test]
    fn row4_payload_bitflip_header_passes_payload_fails() {
        for d in [Digest::Crc32c, Digest::Fnv1a] {
            let rec = encode(12, 7, Scheme::Yi, d);
            let m = lay_on_medium(&rec, Construction::PayloadBitFlip, 0xEE);
            assert_eq!(verify(&m, Scheme::Yi, d), (Verdict::Pass, Verdict::Fail), "{d:?}");
        }
    }

    /// **阳性对照**：候选甲在第二行上两列必然相同 ⇒ 失去可分辨性。
    /// 抓不到这个差别，说明本实验分不出甲乙，四行判决一文不值。
    #[test]
    fn positive_control_jia_cannot_distinguish_row2() {
        let rec = encode(12, 7, Scheme::Jia, Digest::Crc32c);
        let m = lay_on_medium(&rec, Construction::HalfPayloadStale, 0xEE);
        let (hv, pv) = verify(&m, Scheme::Jia, Digest::Crc32c);
        assert_eq!(hv, pv, "候选甲只有一个校验和，两列不可能不同");
        assert_eq!(hv, Verdict::Fail, "整条的校验和在半截记录上必然不过");
        // 而候选乙在同一个构造上两列不同——这才是甲被挡掉的那件事。
        let rec_yi = encode(12, 7, Scheme::Yi, Digest::Crc32c);
        let m_yi = lay_on_medium(&rec_yi, Construction::HalfPayloadStale, 0xEE);
        let (h2, p2) = verify(&m_yi, Scheme::Yi, Digest::Crc32c);
        assert_ne!(h2, p2);
    }

    /// **功劳要分清**：短缓冲那一臂由长度检查拦下，校验和根本没算。
    /// 记成「载荷校验和抓到了」是把别人的功劳算到它头上。
    #[test]
    fn truncated_short_is_caught_by_length_not_by_csum() {
        let rec = encode(12, 7, Scheme::Yi, Digest::Crc32c);
        let m = lay_on_medium(&rec, Construction::PayloadTruncatedShort, 0xEE);
        let (hv, pv) = verify(&m, Scheme::Yi, Digest::Crc32c);
        assert_eq!(hv, Verdict::Pass, "头完整，头校验和照样过");
        assert_eq!(pv, Verdict::NotReached, "长度先拦下了，校验和没被计算");
    }

    /// 残留字节恰好等于真实载荷时，第二行会退化——**这是已知的盲区，钉住它免得被当成通过**。
    /// 取 0xEE 之外的残留值不改变判决，因为点名项内容与 jsn 绑定。
    #[test]
    fn stale_byte_choice_does_not_change_row2_verdict() {
        for stale in [0x00u8, 0xEE, 0xFF] {
            let rec = encode(12, 7, Scheme::Yi, Digest::Crc32c);
            let m = lay_on_medium(&rec, Construction::HalfPayloadStale, stale);
            assert_eq!(
                verify(&m, Scheme::Yi, Digest::Crc32c),
                (Verdict::Pass, Verdict::Fail),
                "stale={stale:#x}"
            );
        }
    }

    /// 判决不随点名项数变——1 项到 71 项（4 KiB 记录的容量上限，E75 实测）都一样。
    #[test]
    fn verdict_is_independent_of_named_count() {
        for named in [1u32, 2, 12, 71] {
            let rec = encode(named, 7, Scheme::Yi, Digest::Crc32c);
            let m = lay_on_medium(&rec, Construction::HalfPayloadStale, 0xEE);
            assert_eq!(
                verify(&m, Scheme::Yi, Digest::Crc32c),
                (Verdict::Pass, Verdict::Fail),
                "named={named}"
            );
        }
    }

    /// `header_csum` 必须覆盖 `payload_csum` 那 4 字节——否则改掉载荷校验和本身是静默的。
    #[test]
    fn header_csum_covers_the_payload_csum_field() {
        assert!(OFF_PAYLOAD_CSUM < OFF_HEADER_CSUM, "载荷校验和必须落在头校验和覆盖范围内");
        let mut m = encode(12, 7, Scheme::Yi, Digest::Crc32c);
        m[OFF_PAYLOAD_CSUM] ^= 0x01;
        let (hv, _) = verify(&m, Scheme::Yi, Digest::Crc32c);
        assert_eq!(hv, Verdict::Fail, "改载荷校验和字段，头校验和必须判红");
    }

    /// 空载荷（0 项）时载荷校验和仍要算得动，不许 panic 也不许退化成 NotReached。
    #[test]
    fn empty_payload_still_verifies() {
        let rec = encode(0, 7, Scheme::Yi, Digest::Crc32c);
        let m = lay_on_medium(&rec, Construction::Intact, 0xEE);
        assert_eq!(verify(&m, Scheme::Yi, Digest::Crc32c), (Verdict::Pass, Verdict::Pass));
    }

    /// 格式常量必须与 kb 的 format-const 标记一致。
    #[test]
    fn format_constants_match_kb() {
        assert_eq!(JOURNAL_HDR, 78, "D23 已定项 4 的 format-const 标记");
        assert_eq!(INC_TXN_BOUNDARY + INC_BACK_CHAIN + INC_PAYLOAD_CSUM, 17);
    }
}
