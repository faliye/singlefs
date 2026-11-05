#!/usr/bin/env bash
# 样本：经 lib_work_root.py 取；注释里写的 ~/.local/share/singlefs/work 不判
last_runs="$(python3 .claude/hooks/lib_work_root.py shared last-runs)"
echo "  ✗ 取不到 ~/.local/share/singlefs/work 下的东西"
refuse "工作根缺省在 ~/.local/share/singlefs/work 下" "设 SAMPLE_ROOT 指过去"
qemu_home="${SAMPLE_QEMU_HOME:-$HOME/.local/share/singlefs/qemu}"
run_selftest() {
  sample="$HOME/.local/share/singlefs/work/selftest-sample"
  echo "$sample"
}
ls "$last_runs" "$qemu_home"
