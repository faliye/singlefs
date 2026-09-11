//! E6 的**单元大小 × CPU 特性**档：AES-256-GCM 与 ChaCha20-Poly1305 的单核吞吐。
//!
//! **为什么是新写的，不是复跑**：E6 正文那张承重表（4 KiB `3888.51 / 267.55 / 1192.00`）
//! 是 2026-08-27 在虚机里量的，而**产出它的代码不在仓里，产物也没入库**——
//! `research/results/` 下只有 08-28 那一轮多核的输出。
//! ⇒ 按 `.claude/singlefs-ai-sop/rules/kb-discipline.md`「所有历史数据都只是参考」，
//! 那张表今天不能继续当 D9 已定项 3 的依据，要重测一遍。
//!
//! ⚠️ **这不是复跑，所以不设「必须复现 3888.51」的闸。**
//! 照着已有答案定判据正是 `.claude/singlefs-ai-sop/rules/test-discipline.md`
//! 「实验开跑之前答案不许已经存在」要禁的形态。本轮量到什么记什么，
//! 与 08-27 那组不一致时**如实并列**，不回头改判据。
//!
//! ## 判据（跑前写死，跨档的部分由 `research/scripts/e6-units.sh` 判）
//!
//! 1. **失败条款**：屏蔽 AES-NI 后 AES-GCM 必须有可分辨的下降（≥2×）。
//!    测不出 ⇒ CPU flag 没生效，**整轮作废**（E6 立项时那一条，原样保留）。
//! 2. **阴性对照**：ChaCha20-Poly1305 不使用 AES-NI ⇒ 两档之间必须几乎不受影响
//!    （每一档 |Δ| ≤ 5%）。它证明那个 flag **只作用在该作用的地方**，不是整机变慢。
//! 3. **自带对照**：`tagchk`（认证标签首字节之和）两档之间必须**逐位相同**——
//!    两档跑的必须是同一份计算，差别只许在快慢上。
//! 4. **绝对值断言**：单测钉住「每个单元都被加密到」「计费字节恰等于 `TOTAL_MIB`」
//!    这两条算术，不让「两条臂一起错」躲过互比
//!    （`.claude/singlefs-ai-sop/rules/test-discipline.md`「只让多条臂互相比…」）。
//!
//! **反过来的结果我接不接受**（跑前写下）：若量出「有 AES-NI 时 ChaCha 反而更快」
//! 或「屏蔽后两者持平」，那就是 D9 已定项 3「运行时按 CPU 特性选」的前提塌了，
//! 该重开的是那一条，不是本实验的判据。

// ⚠️ **故意沿用已弃用的 `encrypt_in_place_detached`**：E6 口径写明用的就是它，
// 换 API 就换了被测对象，与 08-27 那组数字再也不可并列。
// 这是**有理由的压制**，不是忽略警告（`rules/command-safety.md`）。
#![allow(deprecated)]

use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::ChaCha20Poly1305;
use e7_index_bench::Emitter;
use std::time::Instant;

/// E6 口径：每档每种算法各加密 256 MiB。
const TOTAL_MIB: usize = 256;
/// 单元档，与 E6 正文那张表逐档对齐。
const UNITS: [usize; 4] = [4096, 16384, 65536, 262144];

#[derive(Clone, Copy, PartialEq)]
enum CipherAlgorithm {
    Aes,
    Chacha,
}

impl CipherAlgorithm {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>` —— 新增算法不补这里就编译不过
            CipherAlgorithm::Aes => "aes256gcm",
            CipherAlgorithm::Chacha => "chacha20poly1305",
        }
    }
}

/// 原地加密 `buffer` 一整遍，按 `unit` 分块。返回 (处理字节数, 标签首字节之和)。
///
/// `tagchk` 不是装饰：它让「两档跑的是不是同一份计算」变成可比对的事实。
/// 只报吞吐的话，一档悄悄少算了一半也看不出来——它只会显得更快。
fn seal(alg: CipherAlgorithm, buffer: &mut [u8], unit: usize) -> (usize, u64) {
    let key = [7u8; 32];
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let mut tagchk = 0u64;
    let byte_count = buffer.len();
    match alg {
        CipherAlgorithm::Aes => {
            let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
            for unit_chunk in buffer.chunks_mut(unit) {
                let tag = cipher.encrypt_in_place_detached(nonce, b"", unit_chunk).unwrap();
                tagchk += tag[0] as u64;
            }
        }
        CipherAlgorithm::Chacha => {
            let cipher = ChaCha20Poly1305::new_from_slice(&key).unwrap();
            for unit_chunk in buffer.chunks_mut(unit) {
                let tag = cipher.encrypt_in_place_detached(nonce, b"", unit_chunk).unwrap();
                tagchk += tag[0] as u64;
            }
        }
    }
    (byte_count, tagchk)
}

/// 跑一档，返回 (最好一轮的纳秒, 字节数, tagchk, 轮间离散万分比)。
///
/// ⚠️ **缓冲必须逐页预热**：`vec![0u8; n]` 给的是惰性映射的零页，
/// 不预热就是在计时区里缺页——E6 多核档实测那样单核只有 2154 MiB/s（差 45%）。
fn run(alg: CipherAlgorithm, unit: usize, rounds: usize) -> (u64, usize, u64, u64) {
    let bytes = TOTAL_MIB * 1024 * 1024;
    let mut buffer = vec![0u8; bytes];
    for page in buffer.chunks_mut(4096) {
        page[0] = 1;
    }
    let (mut best, mut worst, mut best_round_bytes, mut last_tagchk) = (u64::MAX, 0u64, 0usize, 0u64);
    for _ in 0..rounds {
        let round_start = Instant::now();
        let (byte_count, round_tagchk) = seal(alg, &mut buffer, unit);
        let elapsed_nanoseconds = round_start.elapsed().as_nanos() as u64;
        std::hint::black_box(&buffer);
        if elapsed_nanoseconds < best {
            best = elapsed_nanoseconds;
            best_round_bytes = byte_count;
        }
        if elapsed_nanoseconds > worst {
            worst = elapsed_nanoseconds;
        }
        last_tagchk = round_tagchk;
    }
    let spread = if best == 0 {
        0
    } else {
        (worst - best) * 10_000 / best
    };
    (best, best_round_bytes, last_tagchk, spread)
}

/// **已知答案测试**：拿一个外部实现算出的期望值钉住「这台机器今天算的是不是 AES-256-GCM」。
///
/// 期望值由 `python3 -c` 调 `cryptography` 49.0.0（OpenSSL 后端）在宿主上独立算出，
/// **与本二进制不共享任何一行代码**——`.claude/singlefs-ai-sop/rules/evidence-discipline.md`
/// 要的「换一条独立路径复现同一个判断」就是这个形态。
///
/// 为什么非要有它：`tagchk` 只能说「两档算出来的不一样」，**说不出哪一档是对的**。
/// 而「屏蔽 AES-NI」这一档若算错了，它的吞吐数字一个字都不能用。
const KAT_KEY: [u8; 32] = [7u8; 32];
const KAT_AES_TAG: &str = "62d27233cdaa1703b440830408c9d14d";
const KAT_CHACHA_TAG: &str = "5ad7622fdcef10e164b2b6878c3300eb";

fn kat(alg: CipherAlgorithm) -> String {
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let mut buffer = [0u8; 32];
    let tag = match alg {
        CipherAlgorithm::Aes => Aes256Gcm::new_from_slice(&KAT_KEY)
            .unwrap()
            .encrypt_in_place_detached(nonce, b"", &mut buffer)
            .unwrap(),
        CipherAlgorithm::Chacha => ChaCha20Poly1305::new_from_slice(&KAT_KEY)
            .unwrap()
            .encrypt_in_place_detached(nonce, b"", &mut buffer)
            .unwrap(),
    };
    tag.iter().map(|tag_byte| format!("{tag_byte:02x}")).collect()
}

fn kat_expected(alg: CipherAlgorithm) -> &'static str {
    match alg {
        CipherAlgorithm::Aes => KAT_AES_TAG,
        CipherAlgorithm::Chacha => KAT_CHACHA_TAG,
    }
}

fn mibs(bytes: usize, elapsed_nanoseconds: u64) -> f64 {
    if elapsed_nanoseconds == 0 {
        return f64::NAN;
    }
    bytes as f64 / (1024.0 * 1024.0) / (elapsed_nanoseconds as f64 / 1e9)
}

/// 来宾看不看得见 AES-NI。**报出来，不猜**——外层脚本靠它把两档配对，
/// 猜错档位会让「屏蔽生效了没有」这个判定失去意义。
fn cpu_has_aes() -> bool {
    std::fs::read_to_string("/proc/cpuinfo")
        .map(|cpuinfo_text| {
            cpuinfo_text.lines()
                .filter(|line| line.starts_with("flags"))
                .any(|line| line.split_whitespace().any(|flag| flag == "aes"))
        })
        .unwrap_or(false)
}

fn main() {
    // vm-bench.sh 把盘路径当前缀参数塞进来，本实验不碰盘，跳过它们。
    let rounds: usize = std::env::args()
        .skip(1)
        .find(|argument| !argument.starts_with("/dev/"))
        .and_then(|argument_text| argument_text.parse().ok())
        .unwrap_or(5);
    let mut emitter = Emitter::new();
    let mut output_buffer = String::new();
    let mut say = |line: String| {
        output_buffer.push_str(&line);
        output_buffer.push('\n');
    };

    let aes_ni = cpu_has_aes();
    say(emitter.emit_raw(&format!(
        "name=config total_mib={TOTAL_MIB} rounds={rounds} cpu_aes={aes_ni}"
    )));

    // ── 已知答案测试：先判「算的对不对」，再谈「算得快不快」 ──
    // 算错的那一档，它的吞吐数字一个字都不能用。
    let mut kat_ok = true;
    for alg in [CipherAlgorithm::Aes, CipherAlgorithm::Chacha] {
        let got = kat(alg);
        let want = kat_expected(alg);
        let ok = got == want;
        kat_ok &= ok;
        say(emitter.emit_raw(&format!(
            "name=kat cpu_aes={aes_ni} alg={} tag={got} want={want} ok={ok}",
            alg.name()
        )));
    }

    for unit in UNITS {
        for alg in [CipherAlgorithm::Aes, CipherAlgorithm::Chacha] {
            let (elapsed_nanoseconds, best_round_bytes, last_tagchk, spread) = run(alg, unit, rounds);
            let mebibytes_per_second = mibs(best_round_bytes, elapsed_nanoseconds);
            say(emitter.emit_raw(&format!(
                "name=unit cpu_aes={aes_ni} alg={} unit={unit} mibs={mebibytes_per_second:.2} \
                 bytes={best_round_bytes} elapsed_ns={elapsed_nanoseconds} tagchk={last_tagchk} spread_bp={spread}",
                alg.name()
            )));
        }
    }
    say(emitter.finish());
    print!("{output_buffer}");
    if !kat_ok {
        eprintln!("E6U: 已知答案测试不通过 —— 这台机器这一档算的不是 AES-256-GCM / ChaCha20-Poly1305，整轮作废");
        std::process::exit(5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **外部 oracle**：两种算法的标签必须与 `cryptography` 49.0.0 算出的逐字节相同。
    /// 这条是本实验唯一一条不靠自洽性的检查——别的检查都只能说「前后一致」。
    #[test]
    fn known_answer_matches_an_independent_implementation() {
        assert_eq!(kat(CipherAlgorithm::Aes), KAT_AES_TAG, "AES-256-GCM 的标签与外部实现对不上");
        assert_eq!(
            kat(CipherAlgorithm::Chacha),
            KAT_CHACHA_TAG,
            "ChaCha20-Poly1305 的标签与外部实现对不上"
        );
    }

    /// 加密必须真的改变缓冲——否则量到的是一个空循环，而它会显得很快。
    #[test]
    fn seal_actually_transforms_the_buffer() {
        for alg in [CipherAlgorithm::Aes, CipherAlgorithm::Chacha] {
            let mut buffer = vec![0u8; 4096 * 4];
            let before = buffer.clone();
            let (byte_count, _) = seal(alg, &mut buffer, 4096);
            assert_eq!(byte_count, 4096 * 4);
            assert_ne!(buffer, before, "{} 没有改变缓冲，测的是空循环", alg.name());
        }
    }

    /// **绝对值断言**：每个单元都要被加密到，不是只加密第一块。
    /// 互比测不出「两条臂一起只加密了首块」——两边都会一起变快。
    #[test]
    fn every_unit_is_covered() {
        for alg in [CipherAlgorithm::Aes, CipherAlgorithm::Chacha] {
            for unit in [4096usize, 16384] {
                let mut buffer = vec![0u8; unit * 3];
                seal(alg, &mut buffer, unit);
                for (unit_index, ch) in buffer.chunks(unit).enumerate() {
                    assert!(
                        ch.iter().any(|&byte| byte != 0),
                        "{} 的第 {unit_index} 个 {unit} 单元没被加密",
                        alg.name()
                    );
                }
            }
        }
    }

    /// **绝对值断言**：计费字节恰等于 `TOTAL_MIB`，不由被测代码自报。
    #[test]
    fn billed_bytes_equal_the_stated_workload() {
        let bytes = TOTAL_MIB * 1024 * 1024;
        let mut buffer = vec![0u8; 4096 * 8];
        let (byte_count, _) = seal(CipherAlgorithm::Aes, &mut buffer, 4096);
        assert_eq!(byte_count, buffer.len(), "计费字节不等于缓冲长度");
        // 绝对值由独立算术给出（256 × 1024 × 1024），不从被测代码读回来。
        assert_eq!(bytes, 268_435_456);
        // 4 KiB 档的分块数同样钉死：256 MiB ÷ 4 KiB。
        assert_eq!(bytes / 4096, 65536);
    }

    /// 两种算法必须产出不同的密文——否则枚举分派串了，一条臂在冒充另一条。
    #[test]
    fn the_two_algorithms_differ() {
        let (mut aes_ciphertext, mut chacha_ciphertext) = (vec![0u8; 4096], vec![0u8; 4096]);
        seal(CipherAlgorithm::Aes, &mut aes_ciphertext, 4096);
        seal(CipherAlgorithm::Chacha, &mut chacha_ciphertext, 4096);
        assert_ne!(aes_ciphertext, chacha_ciphertext, "两种算法产出相同密文，分派串了");
    }

    /// `tagchk` 必须随单元数变化——它要能分辨「少算了一半」，
    /// 一个恒定的校验值等于没有这道对照。
    #[test]
    fn tagchk_tracks_the_number_of_units() {
        let mut small = vec![0u8; 4096 * 2];
        let mut big = vec![0u8; 4096 * 8];
        let (_, small_buffer_tagchk) = seal(CipherAlgorithm::Aes, &mut small, 4096);
        let (_, big_buffer_tagchk) = seal(CipherAlgorithm::Aes, &mut big, 4096);
        assert_ne!(small_buffer_tagchk, big_buffer_tagchk, "tagchk 与单元数无关 ⇒ 分不出漏算");
    }

    /// 单元大小必须真的改变分块数——否则「单元大小」这个自变量根本没在动。
    #[test]
    fn unit_size_changes_the_chunking() {
        let mut buffer_sealed_with_4096_byte_units = vec![0u8; 65536];
        let mut buffer_sealed_with_65536_byte_units = vec![0u8; 65536];
        let (_, tagchk_with_4096_byte_units) = seal(CipherAlgorithm::Aes, &mut buffer_sealed_with_4096_byte_units, 4096);
        let (_, tagchk_with_65536_byte_units) = seal(CipherAlgorithm::Aes, &mut buffer_sealed_with_65536_byte_units, 65536);
        assert_ne!(tagchk_with_4096_byte_units, tagchk_with_65536_byte_units, "换单元大小之后分块没变，自变量没在动");
    }
}
