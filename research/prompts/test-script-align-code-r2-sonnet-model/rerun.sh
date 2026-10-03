#!/usr/bin/env bash
# 辩方腿 test-script-align-code-r2 的复跑：全是只读命令（grep、awk、两份 python 静态核），不编译、不跑任何测试。
# 用法：bash research/prompts/test-script-align-code-r2-sonnet-model/rerun.sh [输出目录]
# 读：冻结副本 /tmp/claude-1000/test-script-align-code-r2/tree（第二轮）与 /tmp/claude-1000/test-script-align-code-r1/tree（只取变异表对照）。
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="${1:-$HERE/outputs}"
TREE="${K3_TREE:-/tmp/claude-1000/test-script-align-code-r2/tree}"
REPO="${REPO_ROOT:-$(cd "$HERE/../../.." && pwd)}"
mkdir -p "$OUT"
cd "$TREE/crates" || exit 2
{
  echo "## J3 静态核（变异表 r1→r2 变动与新增的行）"
  nice -n 19 python3 "$HERE/k3_static_check.py" | grep -E '^BAD|^rows_checked'
  echo "## J3 筛选词核（筛选词在目标里匹配到几条 fn；门禁只要求点名的完整名在输出里恰好一个，所以匹配多条不等于红）"
  nice -n 19 python3 "$HERE/k3_filter_check.py" | grep -E '^BAD|^filter_rows_checked'
} > "$OUT/j3.out" 2>&1
{
  echo "## J5：读环境变量里数值的写法（单行 env::var 命中）"
  grep -rnE 'env::var(_os)?\(' . --include=*.rs | wc -l
  echo "## J5：悄悄退回默认的两处同形（线程数）"
  grep -n 'SINGLEFS_CRASH_AMPLIFICATION_THREADS' -A5 singlefs-checker-tier/src/crash_amplification.rs | sed -n 1,8p
  grep -n 'SINGLEFS_THREAD_CAP' -B1 -A4 singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs
  echo "## J5：写错就 panic 的同形（线程数）"
  grep -n 'LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE 要是正整数' singlefs-checker-tier/src/crash.rs
  grep -n 'FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE}={text} 不是十进制正整数' singlefs-harness/src/fault_injection.rs
  grep -n '要是正整数，读到' singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs
  grep -n 'fn number_from_environment' -A4 singlefs-harness/tests/random_histories.rs
  echo "## J5：set_var / remove_var 全部落点"
  grep -rnE 'set_var|remove_var' . --include=*.rs
  echo "## J5：by=wall_clock 全部落点（测试只有 crash_amplification.rs 的两处断言串）"
  grep -rn 'by=wall_clock' . --include=*.rs | cut -c1-120
  echo "## J5：a_line_was_printed_at 的调用与定义"
  grep -n 'a_line_was_printed_at' singlefs-checker-tier/src/crash.rs
  echo "## J5：const ALL 个数与 pub struct X(pub ..) 个数"
  grep -rnE 'const ALL\b' . --include=*.rs | wc -l
  grep -rnE 'pub struct [A-Za-z]+\(pub ' . --include=*.rs | wc -l
  echo "## J4：死代码（无调用方）"
  grep -rn 'number_from_process_environment_panicking_when_unreadable' . --include=*.rs | cut -c1-140
  echo "## J4：枚举加 ALL 的宏"
  grep -rn 'macro_rules' . --include=*.rs | cut -c1-140
} > "$OUT/j4_j5.out" 2>&1
{
  echo "## J5：E165 钉表五对列逐行核（账第 3、4、5、12、13 段对钉表 ②③④⑦⑧；钉表 21 段 = 自己 8 段 + 账 13 段）"
  awk -F'\t' '!/^#/ && NF>0 {n++; if (NF!=21) bad++; else if ($11!=$2||$12!=$3||$13!=$4||$20!=$7||$21!=$8) diff++} END{print "rows",n,"nf!=21",bad+0,"pairdiff",diff+0}' "$REPO/.claude/gate.d/e165-pinned-red-examples.tsv"
} > "$OUT/j5_e165.out" 2>&1
echo "写在 $OUT：j3.out j4_j5.out j5_e165.out"
