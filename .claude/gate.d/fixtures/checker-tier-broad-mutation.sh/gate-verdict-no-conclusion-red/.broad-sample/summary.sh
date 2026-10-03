#!/usr/bin/env bash
# 样本：录好的驱动 --summary 输出（这一版 6 条里 3 条没有结论：改到了还没现判），驱动判红退 8
cat <<'EOF'
BROAD_MUTATION_SUMMARY fingerprint=970a802431217260 commit=0123456789ab total=6 caught=2 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 driver_error=0 not_run=3
  抓到 2、没红 0、无效 1、超时 0、内存撞顶 0、只过了第一段 0、驱动错 0、这一版还没跑 3；清单 6 条
BROAD_MUTATION_GATE total=6 concluded=3 carried=2 fresh=1 missed=0 without_conclusion=3 flipped=0 verdict=red
NO_CONCLUSION	4	none	crates/singlefs-core/src/sample.rs:20:5: replace add -> u32 with 0
NO_CONCLUSION	5	none	crates/singlefs-core/src/sample.rs:20:5: replace add -> u32 with 1
NO_CONCLUSION	6	none	crates/singlefs-core/src/sample.rs:24:9: replace == with != in add
EOF
echo "  ✗ 提交门禁判红（C6）：3 条这一版没有结论、0 条没红" >&2
echo "  → 怎么办：没有结论的用 --changed-since 补齐" >&2
exit 8
