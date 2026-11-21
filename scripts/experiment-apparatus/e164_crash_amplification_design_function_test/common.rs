//! E164 装置的公共部分：SHA-256（手写，不加依赖）、十六进制、E7RESULT 行、子进程白名单（跑前登记 5.1）。

use std::path::Path;
use std::process::{Command, Output};

const SHA256_INITIAL_HASH: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hash = SHA256_INITIAL_HASH;
    let bit_length = u64::try_from(bytes.len()).expect("长度装得进 u64") * 8;
    let mut message = bytes.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_length.to_be_bytes());
    for chunk in message.as_chunks::<64>().0 {
        let mut schedule = [0u32; 64];
        for (word_index, word) in schedule.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes(chunk[word_index * 4..word_index * 4 + 4].try_into().expect("切了 4 字节"));
        }
        for word_index in 16..64 {
            let sigma_zero = schedule[word_index - 15].rotate_right(7)
                ^ schedule[word_index - 15].rotate_right(18)
                ^ (schedule[word_index - 15] >> 3);
            let sigma_one = schedule[word_index - 2].rotate_right(17)
                ^ schedule[word_index - 2].rotate_right(19)
                ^ (schedule[word_index - 2] >> 10);
            schedule[word_index] = schedule[word_index - 16]
                .wrapping_add(sigma_zero)
                .wrapping_add(schedule[word_index - 7])
                .wrapping_add(sigma_one);
        }
        let mut registers = hash;
        for round in 0..64 {
            let big_sigma_one = registers[4].rotate_right(6) ^ registers[4].rotate_right(11) ^ registers[4].rotate_right(25);
            let choose = (registers[4] & registers[5]) ^ ((!registers[4]) & registers[6]);
            let temporary_one = registers[7]
                .wrapping_add(big_sigma_one)
                .wrapping_add(choose)
                .wrapping_add(SHA256_ROUND_CONSTANTS[round])
                .wrapping_add(schedule[round]);
            let big_sigma_zero = registers[0].rotate_right(2) ^ registers[0].rotate_right(13) ^ registers[0].rotate_right(22);
            let majority = (registers[0] & registers[1]) ^ (registers[0] & registers[2]) ^ (registers[1] & registers[2]);
            let temporary_two = big_sigma_zero.wrapping_add(majority);
            registers = [
                temporary_one.wrapping_add(temporary_two),
                registers[0],
                registers[1],
                registers[2],
                registers[3].wrapping_add(temporary_one),
                registers[4],
                registers[5],
                registers[6],
            ];
        }
        for (slot, value) in hash.iter_mut().zip(registers) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut digest = [0u8; 32];
    for (word_index, word) in hash.iter().enumerate() {
        digest[word_index * 4..word_index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

pub fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn sha256_hexadecimal(bytes: &[u8]) -> String {
    hexadecimal(&sha256(bytes))
}

/// 一行 `E7RESULT … key=value …` 拆成 key → value（值里不带空白）。
pub fn parse_result_fields(line: &str) -> std::collections::BTreeMap<String, String> {
    line.split_whitespace()
        .filter_map(|token| token.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// 跑前登记 5.1 的子进程白名单：起之前逐条核参数，碰到别的就整轮作废（退 3）。
pub fn assert_subprocess_is_allowed(program: &str, arguments: &[String]) {
    let joined = arguments.join(" ");
    let forbidden_words = ["singlefs-verification", "layer0", "gate.d", "mutate.sh"];
    if program == "cargo" && forbidden_words.iter().any(|word| joined.contains(word)) {
        refuse_subprocess(program, &joined, "参数里有 验证包、layer0、门禁阶段或 mutate.sh");
    }
    let allowed = match program {
        "cargo" => match arguments.first().map(String::as_str) {
            Some("build" | "test") => {
                let package_position = arguments.iter().position(|argument| argument == "-p");
                let package = package_position.and_then(|position| arguments.get(position + 1)).map(String::as_str);
                matches!(package, Some("singlefs-verification" | "singlefs-format"))
            }
            Some("metadata") => arguments.iter().any(|argument| argument == "--offline"),
            _ => false,
        },
        "rustc" => {
            arguments == ["-vV"] || arguments.iter().any(|argument| argument == "--emit=metadata")
                && arguments.windows(2).any(|pair| pair[0] == "--crate-type" && pair[1] == "lib")
        }
        "git" => {
            let subcommand = arguments.iter().find(|argument| !argument.starts_with('-') && !argument.contains('='));
            matches!(subcommand.map(String::as_str), Some("log" | "rev-list" | "ls-files" | "show" | "grep" | "diff" | "rev-parse" | "ls-tree"))
        }
        "python3" => arguments.first().is_some_and(|script| script.ends_with("scripts/gate/admission.py"))
            && arguments.get(1).is_some_and(|subcommand| subcommand == "crash-case-manifest"),
        _ => false,
    };
    if !allowed {
        refuse_subprocess(program, &joined, "不在跑前登记 5.1 的子进程白名单里");
    }
}

fn refuse_subprocess(program: &str, joined: &str, reason: &str) -> ! {
    eprintln!("  ✗ 装置要起白名单之外的子进程（{reason}）：{program} {joined}");
    eprintln!("  → 怎么办：这一轮作废（跑前登记第十一节 V-tier）；改装置，不放宽白名单");
    std::process::exit(3)
}

/// 核过白名单之后起子进程，在 `directory` 里跑，环境按 `environment`（值为 None 的删掉）。
pub fn run_allowed(
    program: &str,
    arguments: &[String],
    directory: &Path,
    environment: &[(String, Option<String>)],
) -> Output {
    assert_subprocess_is_allowed(program, arguments);
    let mut command = Command::new(program);
    command.args(arguments).current_dir(directory);
    for (key, value) in environment {
        match value {
            Some(text) => {
                command.env(key, text);
            }
            None => {
                command.env_remove(key);
            }
        }
    }
    command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 {program}：{error}");
        eprintln!("  → 怎么办：确认 {program} 在 PATH 上");
        std::process::exit(3)
    })
}

pub fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}
