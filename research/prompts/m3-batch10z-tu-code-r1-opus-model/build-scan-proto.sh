#!/usr/bin/env bash
# K4：把冻结副本 checker 档两份源文件测试模块里的扫源码函数原样抄进一个独立的小 crate（不是 checker 档包，不跑它的测试），编成两个二进制：
# atk-stop-scan（crash_amplification.rs 收叫停那一扫）、atk-gpu-scan（gpu_unit_checks.rs 卡池那一扫）。build-scan-proto.sh <冻结副本根> <输出目录>
set -euo pipefail
FROZEN=${1:?}; P=${2:?}
ROOT=/home/fy5090/code/singlefs
C=$FROZEN/crates/singlefs-checker-tier/src/crash_amplification.rs
G=$FROZEN/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
mkdir -p "$P"/src/bin
printf '[package]\nname = "atk-stop-scan"\nversion = "0.1.0"\nedition = "2021"\n\n[workspace]\n' > "$P"/Cargo.toml
{
  echo '#![allow(dead_code, clippy::all)]'
  echo '// 抄自冻结副本 crates/singlefs-checker-tier/src/crash_amplification.rs 测试模块的扫源码函数（行 3581-3583、4666-4679、4834-5067、5123-5162、5174-5355），原样；main 是攻方加的。'
  sed -n 3581,3583p "$C"; sed -n 4666,4679p "$C"; sed -n 4834,5067p "$C"; sed -n 5123,5162p "$C"; sed -n 5174,5355p "$C"
  cat <<'RS'

fn main() {
    // 用法：atk-stop-scan <源文件>…：每份打印扫出来的条数与每一条的头 160 字。
    for path in std::env::args().skip(1) {
        let source = std::fs::read_to_string(&path).expect("读源文件");
        let found = ways_a_layer_handed_the_stop_request_could_drop_it(&source);
        println!("ATK scan {path} found={}", found.len());
        for finding in &found {
            println!("ATK scan   {}", finding.chars().take(160).collect::<String>());
        }
    }
}
RS
} > "$P"/src/main.rs
{
  echo '#![allow(dead_code, clippy::all)]'
  echo '// 抄自冻结副本 crates/singlefs-checker-tier/src/gpu_unit_checks.rs 测试模块的扫源码函数（行 1920、2467-2802），原样；main 是攻方加的。'
  sed -n 1920p "$G"; sed -n 2467,2802p "$G"
  cat <<'RS'

fn main() {
    // 用法：atk-gpu-scan <源文件>…：每份取测试模块之前那一段，打印扫出来的条数与每一条的头 160 字；扫的时候 panic 的打印 panic。
    for path in std::env::args().skip(1) {
        let source = std::fs::read_to_string(&path).expect("读源文件");
        let code = source.split(TESTS_MODULE_OPENING).next().expect("split 至少交一段").to_string();
        match std::panic::catch_unwind(|| ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&code)) {
            Ok(found) => {
                println!("ATK gpuscan {path} found={}", found.len());
                for finding in &found {
                    println!("ATK gpuscan   {}", finding.chars().take(160).collect::<String>());
                }
            }
            Err(_) => println!("ATK gpuscan {path} panicked"),
        }
    }
}
RS
} > "$P"/src/bin/atk-gpu-scan.rs
cd "$P"
bash "$ROOT"/research/scripts/run-with-memory-cap.sh 4G nice -n 19 bash "$ROOT"/research/scripts/capped.sh 4 cargo build --offline --release
