#!/usr/bin/env bash
# 把攻方探针（probes/background_gpu_attack_probes.rs）接进一棵仓副本里 crates/singlefs-background-gpu/src/background_gpu.rs 的测试模块末尾：
# 去掉文件最后一行（测试模块的收尾花括号），接上探针（探针自带收尾花括号）。用法：apply-probes.sh <模型目录> <仓副本根>
set -euo pipefail
model="$1"
tree="$2"
file="$tree/crates/singlefs-background-gpu/src/background_gpu.rs"
[[ "$(tail -n 1 "$file")" == "}" ]] || { echo "✗ $file 最后一行不是收尾花括号" >&2; exit 1; }
head -n -1 "$file" > "$file.with-probes"
cat "$model/probes/background_gpu_attack_probes.rs" >> "$file.with-probes"
mv "$file.with-probes" "$file"
echo "applied probes to background_gpu.rs"
