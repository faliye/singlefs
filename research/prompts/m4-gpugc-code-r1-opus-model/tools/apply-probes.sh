#!/usr/bin/env bash
# 把攻方探针（probes/ 下三段测试）接进一棵仓副本的后台 GPU 包三份源文件的测试模块末尾：去掉文件最后一行（测试模块的收尾花括号），
# 接上探针（探针自带收尾花括号）。用法：apply-probes.sh <模型目录> <仓副本根>
set -euo pipefail
model="$1"
tree="$2"
src="$tree/crates/singlefs-background-gpu/src"
for name in waiting_for_the_gpu adapter_inventory region_record_kernels; do
  file="$src/$name.rs"
  [[ "$(tail -n 1 "$file")" == "}" ]] || { echo "✗ $file 最后一行不是收尾花括号" >&2; exit 1; }
  head -n -1 "$file" > "$file.with-probe"
  cat "$model/probes/${name}_probe.rs" >> "$file.with-probe"
  mv "$file.with-probe" "$file"
  echo "applied probe to $name.rs"
done
