#!/usr/bin/env bash
# 复跑闭包模型。用法：bash rerun.sh [<仓根>]（默认取这份文件往上三层）。
# 整条经内存包装跑：bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-crash-enumeration-merge-r3-opus-model/rerun.sh
# 先核本目录 SHA256SUMS，再从仓里的 crash_identity.rs 按钉好的行区间抽出闭包算法（那份文件变了退 3），编一个不带依赖的单文件程序、跑。
# 末行 MODEL_SELFCHECK … mismatched=0 [] 才算复现。不链 checker 档、不跑任何 checker 档测试。
set -euo pipefail
model="$(cd "$(dirname "$0")" && pwd)"
repo="${1:-$(cd "$model/../../.." && pwd)}"
(cd "$model" && sha256sum -c --quiet SHA256SUMS)
work="$(mktemp -d "${TMPDIR:-/tmp}/closure-model.XXXXXX")"
trap 'rm -rf "$work"' EXIT
bash "$model/extract.sh" "$repo" "$work/build"
(cd "$work/build" && CARGO_TARGET_DIR="$work/target" cargo build --release --offline --quiet)
"$work/target/release/closure-model" "$repo" "$work/out"
