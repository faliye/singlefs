#!/usr/bin/env bash
# 把攻方腿 m4-gpugc-code-r3 的探针接进一棵仓副本（不进仓）：
#   ① probes/background_gpu_attack_probes_round_three.rs 接在 crates/singlefs-background-gpu/src/background_gpu.rs 测试模块的末尾
#      （去掉文件最后一行、即测试模块的收尾花括号，接上探针；探针自带收尾花括号）；
#   ② 副本里 crates/singlefs-background-gpu/Cargo.toml 的 wgpu 一行特性加上 noop（wgpu 30.0.1 的 noop 后端：不起 GPU、不碰 Vulkan），
#      只供 attack_probe_r3_wgpu_noop_out_of_memory_inside_the_scope_of_a_chunk 用；Cargo.lock 里 wgpu 的依赖已列着 noop 要的 smallvec 与 wgpu-hal。
# 用法：apply-probes.sh <模型目录> <仓副本根>。只改仓副本，两处各核一遍改上了，改不上退 1。
set -euo pipefail
model="$1"
tree="$2"
source_file="$tree/crates/singlefs-background-gpu/src/background_gpu.rs"
manifest="$tree/crates/singlefs-background-gpu/Cargo.toml"
probes="$model/probes/background_gpu_attack_probes_round_three.rs"
[[ -f "$probes" ]] || { echo "✗ 探针 $probes 不在" >&2; exit 1; }
[[ "$(tail -n 1 "$source_file")" == "}" ]] || { echo "✗ $source_file 最后一行不是收尾花括号" >&2; exit 1; }
if grep -qF 'fn attack_probe_r3_' "$source_file"; then
  echo "✗ $source_file 里已经有 attack_probe_r3_ 探针：给一棵新拷的副本" >&2
  exit 1
fi
head -n -1 "$source_file" > "$source_file.with-probes"
cat "$probes" >> "$source_file.with-probes"
mv "$source_file.with-probes" "$source_file"
wgpu_line='wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"] }'
[[ "$(grep -cxF "$wgpu_line" "$manifest")" == 1 ]] || { echo "✗ $manifest 里 wgpu 那一行不是 $wgpu_line" >&2; exit 1; }
sed 's/^wgpu = { version = "=30.0.1", default-features = false, features = \["vulkan", "wgsl", "std"\] }$/wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std", "noop"] }/' \
  "$manifest" > "$manifest.with-noop"
mv "$manifest.with-noop" "$manifest"
[[ "$(grep -c '"noop"' "$manifest")" == 1 ]] || { echo "✗ $manifest 的 wgpu 一行没加上 noop" >&2; exit 1; }
echo "name=apply_probes probes_appended_to=$source_file noop_feature_added_to=$manifest"
