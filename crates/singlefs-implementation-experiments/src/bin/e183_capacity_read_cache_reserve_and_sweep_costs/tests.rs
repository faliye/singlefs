use super::*;

/// 第一段 `all` 输出的钉死期望值（测试不读 research/ 下的产物，research/ 可以整个删掉）：
/// 输出里每行从 `E7RESULT ` 起截下来、各加一个换行，依次连成的字节流的 SHA-256（小写十六进制）与行数。
/// 来源：第一段 `all` 产物（`research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`，
/// 459 行、152434 字节、每行都以 `E7RESULT ` 起头），对整个文件现算：`sha256sum <产物>` 与 `wc -l <产物>`。
const SEGMENT_ONE_PINNED_SHA256: &str =
    "04050c83f8fa8563df195fab5745c59cc66233e6d0bf6d00e1edfc72d794fd88";
const SEGMENT_ONE_PINNED_LINE_COUNT: usize = 459;

/// SHA-256（FIPS 180-4），小写十六进制。测试里自带一份，免得另加依赖；
/// 正确性由逐字节比对用例里的已知向量（空串、`abc`）先核。
fn sha256_hex_of(bytes: &[u8]) -> String {
    /// 前 64 个素数的立方根小数部分的前 32 位。
    const ROUND_CONSTANTS: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut digest_words: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut padded = bytes.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    let bit_length = u64::try_from(bytes.len()).expect("长度装得进 u64") * 8;
    padded.extend_from_slice(&bit_length.to_be_bytes());
    let (blocks, _) = padded.as_chunks::<64>();
    for block in blocks {
        let mut schedule = [0u32; 64];
        let (block_words, _) = block.as_chunks::<4>();
        for (word, chunk) in schedule.iter_mut().zip(block_words) {
            *word = u32::from_be_bytes(*chunk);
        }
        for index in 16..64 {
            let early = schedule[index - 15];
            let late = schedule[index - 2];
            let early_mix = early.rotate_right(7) ^ early.rotate_right(18) ^ (early >> 3);
            let late_mix = late.rotate_right(17) ^ late.rotate_right(19) ^ (late >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(early_mix)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(late_mix);
        }
        let mut working = digest_words;
        for index in 0..64 {
            let choose_input_word = working[4];
            let majority_input_word = working[0];
            let choose = (choose_input_word & working[5]) ^ (!choose_input_word & working[6]);
            let majority = (majority_input_word & working[1])
                ^ (majority_input_word & working[2])
                ^ (working[1] & working[2]);
            let rotated_mix_of_the_choose_input = choose_input_word.rotate_right(6)
                ^ choose_input_word.rotate_right(11)
                ^ choose_input_word.rotate_right(25);
            let rotated_mix_of_the_majority_input = majority_input_word.rotate_right(2)
                ^ majority_input_word.rotate_right(13)
                ^ majority_input_word.rotate_right(22);
            let sum_entering_both_new_input_words = working[7]
                .wrapping_add(rotated_mix_of_the_choose_input)
                .wrapping_add(choose)
                .wrapping_add(ROUND_CONSTANTS[index])
                .wrapping_add(schedule[index]);
            let new_majority_input_word = sum_entering_both_new_input_words
                .wrapping_add(rotated_mix_of_the_majority_input)
                .wrapping_add(majority);
            working = [
                new_majority_input_word,
                working[0],
                working[1],
                working[2],
                working[3].wrapping_add(sum_entering_both_new_input_words),
                working[4],
                working[5],
                working[6],
            ];
        }
        for (digest_word, worked) in digest_words.iter_mut().zip(working) {
            *digest_word = digest_word.wrapping_add(worked);
        }
    }
    digest_words
        .iter()
        .map(|word| format!("{word:08x}"))
        .collect()
}

/// 设了它，下面那条子进程用例才真跑第一段 `all`。
const SEGMENT_ONE_CHILD_SWITCH: &str = "E183_SEGMENT_ONE_ALL_IN_THIS_PROCESS";
const SEGMENT_ONE_CHILD_TEST: &str =
    "tests::segment_one_all_printed_in_a_child_process_when_the_comparison_spawns_it";

/// 只在逐字节比对的用例起的子进程里跑（设了开关）：那个子进程只跑这一条，计数分配器的量窗里没有别的用例的分配；
/// 直接跑全量时没设开关，什么都不做（第一段的输出由下一条用例在子进程里取）。
#[test]
fn segment_one_all_printed_in_a_child_process_when_the_comparison_spawns_it() {
    if std::env::var_os(SEGMENT_ONE_CHILD_SWITCH).is_none() {
        return;
    }
    assert_eq!(
        measure_the_named_items_returning_the_exit_code(&ITEM_NAMES, true),
        0,
        "第一段 all 退 0"
    );
}

/// 派发要求：加了第二、三段之后，第一段 `all` 的输出与入库产物逐字节相同（比的是源码里钉死的 SHA-256 与行数，不读产物文件）。
#[test]
fn segment_one_all_output_equals_the_registered_product_byte_for_byte() {
    assert_eq!(
        sha256_hex_of(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "SHA-256 的已知向量（空串）"
    );
    assert_eq!(
        sha256_hex_of(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "SHA-256 的已知向量（abc）"
    );
    let output = std::process::Command::new(std::env::current_exe().expect("测试程序的路径"))
        .args([
            "--exact",
            SEGMENT_ONE_CHILD_TEST,
            "--nocapture",
            "--test-threads=1",
        ])
        .env(SEGMENT_ONE_CHILD_SWITCH, "1")
        .output()
        .expect("子进程起得来");
    assert!(
        output.status.success(),
        "子进程退 {:?}：{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let printed = String::from_utf8(output.stdout).expect("输出是 UTF-8");
    let printed_lines: Vec<&str> = printed
        .lines()
        .filter_map(|line| line.find("E7RESULT ").map(|start| &line[start..]))
        .collect();
    assert_eq!(
        printed_lines.len(),
        SEGMENT_ONE_PINNED_LINE_COUNT,
        "现跑的 E7RESULT 行数与钉死的入库产物行数不同"
    );
    let printed_bytes: Vec<u8> = printed_lines
        .iter()
        .flat_map(|line| line.bytes().chain(std::iter::once(b'\n')))
        .collect();
    assert_eq!(
        sha256_hex_of(&printed_bytes),
        SEGMENT_ONE_PINNED_SHA256,
        "逐字节：现跑每行加一个换行连成的字节流的 SHA-256 与钉死的入库产物不同（{} 行、{} 字节）",
        printed_lines.len(),
        printed_bytes.len()
    );
}

#[test]
fn the_program_file_bytes_are_the_size_of_the_file_the_process_was_started_from() {
    let started_from = std::env::current_exe().expect("测试程序的路径");
    let size = std::fs::metadata(&started_from)
        .expect("读得到测试程序")
        .len();
    assert!(size > 0, "{started_from:?}");
    assert_eq!(own_program_file_bytes(), Some(size), "{started_from:?}");
}

#[test]
fn forced_preflight_line_gives_its_summary_with_underscores_and_a_met_line_gives_none() {
    assert_eq!(
        forced_summary_of("forced inputs unchanged since last run"),
        Some("inputs_unchanged_since_last_run".to_string())
    );
    assert_eq!(forced_summary_of("met abc123"), None);
    assert_eq!(forced_summary_of(""), None);
}
