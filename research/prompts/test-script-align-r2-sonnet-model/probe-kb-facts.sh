#!/usr/bin/env bash
# 辩方 U3 / U4 探针：只读主工作区，逐条核欠账行与判决写回里的几个事实。每一段第一行写「核什么」，之后是命令原样输出。
set -uo pipefail
cd "${SINGLEFS_REPOSITORY:-/home/fy5090/code/singlefs}"

echo "## 1 C612 行里 mutate.sh 的行号：行里写的位置 → 这一刻那一句在哪一行（空行或别的内容算对不上）"
awk 'NR==135' research/scripts/mutate.sh | cut -c1-80 | sed 's/^/  第 135 行（行里写 selftest_time_limit=2）：/'
grep -n 'local selftest_time_limit=2' research/scripts/mutate.sh | cut -c1-80 | sed 's/^/  实际：/'
grep -n '^# 每条变异限时 MUTATE_TIMEOUT' research/scripts/mutate.sh | cut -c1-60 | sed 's/^/  实际（行里写第 52–53 行）：/'
grep -n '^run_capped_cargo_test() {' research/scripts/mutate.sh | sed 's/^/  实际（行里写第 709–712 行）：/'
grep -n 'local case_directory=' research/scripts/mutate.sh | cut -c1-70 | sed 's/^/  实际（行里写第 166、216 行）：/'
grep -n '^TEST_ONLY_VARIABLES=' research/scripts/mutate.sh | cut -c1-50 | sed 's/^/  实际（行里写第 119 行）：/'
grep -n '^#   退 252「排不上」' research/scripts/mutate.sh | cut -c1-50 | sed 's/^/  实际（行里写第 56–59 行）：/'
grep -n '^# 只供自证' research/scripts/mutate.sh | cut -c1-50 | sed 's/^/  实际（行里写第 65 行）：/'
awk 'NR==87' research/scripts/run-with-memory-cap.sh | cut -c1-60 | sed 's/^/  run-with-memory-cap.sh 第 87 行（行里写的，这一处对得上）：/'

echo "## 2 带 _BREAK 变量名的脚本数（弄坏开关平常跑也读的同形）：C622 点名的是 .claude/gate.d 各道与 crash-amplification-item.sh"
grep -rlE '\b[A-Z][A-Z0-9_]*_BREAK\b' research/scripts .claude/hooks .claude/scripts .claude/gate.d --include=*.sh --include=*.py 2>/dev/null | sed 's#/[^/]*$##' | sort | uniq -c
for f in research/scripts/run-with-memory-cap.sh research/scripts/layer0-shard-run.sh research/scripts/mutation-shard-run.sh research/scripts/verus-proof-check.py research/scripts/e165-pin-red-examples.py research/scripts/crates-mutation-rows.py research/scripts/detached-run.sh; do
  grep -nE '(RUN_WITH_MEMORY_CAP_BREAK|LAYER0_SHARD_RUN_BREAK|MUTATION_SHARD_RUN_BREAK|VERUS_PROOF_CHECK_BREAK|E165_PIN_RED_EXAMPLES_BREAK|GATE_MUTATION_BREAK|DETACHED_RUN_BREAK)' "$f" | grep -v '^[0-9]*:\s*#' | head -1 | cut -c1-150 | sed "s#^#  $f:#"
done
grep -n 'crash_amplification_item_break_is() ' research/scripts/crash-amplification-lib.sh | cut -c1-150

echo "## 3 feature-gated-tests 只扫 tests/ 顶层；src/lib.rs 里挂特性的模块与里面的 #[test] 条数"
grep -n '^#\[cfg(feature = ' -A1 crates/singlefs-checker-tier/src/lib.rs | grep -v '^--$' | cut -c1-70
total=0
for f in crash_amplification crash_judge_dispatch crash_judge_gpu crash_verify_gpu gpu_unit_checks verdict_store; do
  n="$(grep -c '#\[test\]' "crates/singlefs-checker-tier/src/$f.rs")"; total=$((total + n)); printf '  src/%s.rs #[test]=%s\n' "$f" "$n"
done
echo "  合计 $total"

echo "## 4 全绿标记与全量登记"
printf '  stage-inputs.tsv 里 crash-case: 行数='; grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv
printf '  git common-dir 里 singlefs-crash-case-green.* 个数='; ls "$(git rev-parse --git-common-dir)" | grep -c '^singlefs-crash-case-green'
ls -l --time-style=+%F "$(git rev-parse --git-common-dir)" | grep 'singlefs-crash-case-green' | awk '{print "  标记："$6, $7}' | cut -c1-90
printf '  自 2026-09-27 17:33 起动过 crates/ 或 Cargo 清单的提交数='; git log --since='2026-09-27 17:33' --format='%h' -- crates Cargo.toml Cargo.lock | wc -l

echo "## 5 C610 的数：带日期的 //! 行，同一行有没有 kb 指路（正则是这里写的近似，行里没给命令）"
python3 - <<'PY'
import glob, re
date = re.compile(r'20\d\d-\d\d-\d\d')
kb = re.compile(r'(?:D\d+|E\d+|C\d{2,3})（|已定项|未定项|\.claude/kb/')
who = re.compile(r'用户[^，。；]{0,12}定|用户.{0,6}2026|2026-\d\d-\d\d[^，。；]{0,8}定')
total = no_kb = 0; files = set(); no_kb_files = set(); kb_and_who = 0
for path in glob.glob('crates/*/tests/*.rs'):
    for line in open(path, encoding='utf-8', errors='replace'):
        if line.startswith('//!') and date.search(line):
            total += 1; files.add(path)
            if not kb.search(line):
                no_kb += 1; no_kb_files.add(path)
            elif who.search(line):
                kb_and_who += 1
print(f"  带日期的 //! 行 {total} 行 / {len(files)} 份；同一行没有 kb 指路 {no_kb} 行 / {len(no_kb_files)} 份；有 kb 指路又写「用户…定」的 {kb_and_who} 行")
PY

echo "## 6 登记表头与 C613 对「-V」的说法"
grep -n '登记之前读那份脚本 --selftest 走的代码' .claude/hooks/selftest-registry.tsv | cut -c1-200
grep -n '^research/scripts/admission.py' .claude/hooks/selftest-registry.tsv | cut -c1-200

echo "## 7 ReuseKey 的字段数与 each_attribute_changes_the_reuse_key 点名的变异行数"
grep -n 'pub struct StepAttributes' -A3 crates/singlefs-checker-tier/src/crash_identity.rs | cut -c1-110
printf '  crates/mutations.tsv 里点名 each_attribute_changes_the_reuse_key 的行数='; grep -c each_attribute_changes_the_reuse_key crates/mutations.tsv

echo "## 8 stage-selftest.sh 没带 --item 时仍按名字认（lib_heavy_tests.py 文件头）"
grep -nF '不带 --item / --list-items 的 ① 仍按名字与位置认 stage-selftest.sh' .claude/hooks/lib_heavy_tests.py | cut -c1-120

echo "## 9 T1-2：checks-owed.md 里点名 test-script-discipline.md 小节的写法，节名在副本里逐字存在吗"
python3 - <<'PY'
import re
heads = [line[3:].strip() for line in open('.claude/singlefs-ai-sop/rules/test-script-discipline.md', encoding='utf-8') if line.startswith('## ')]
total = bad = 0
for number, line in enumerate(open('.claude/kb/checks-owed.md', encoding='utf-8'), 1):
    for match in re.finditer(r'test-script-discipline\.md`[^「]{0,3}「([^」]+)」', line):
        total += 1
        if match.group(1) not in heads:
            bad += 1
            print('  对不上', number, match.group(1))
print(f'  共 {total} 处，对不上 {bad} 处')
PY

echo "## 10 T1-5：三处定义对「没登记」「heavy-if-set」的同一句"
for file in .claude/agent-common.md CLAUDE.md .claude/agents/tooling-writer.md; do
  printf '  %s：' "$file"; grep -cF '登记为 heavy 的、登记为 heavy-if-set 而命令里设了那个变量的、没登记的' "$file"
done

echo "## 11 里程碑收尾有没有机器可读的读数；崩溃放量运行目录"
printf '  grep -rn 里程碑收尾 .claude/gate.d .claude/hooks research/scripts 命中行数='; grep -rn '里程碑收尾' .claude/gate.d .claude/hooks research/scripts 2>/dev/null | wc -l
runs="$HOME/.local/share/singlefs/crash-amplification/runs"
printf '  runs/ 下 entry- 开头的目录数（入口新写法）='; ls "$runs" 2>/dev/null | grep -c '^entry-'
printf '  runs/ 下 crash-amplification- 开头的目录数（旧写法）='; ls "$runs" 2>/dev/null | grep -c '^crash-amplification-'

echo "## 12 补丁没合入的东西今天在 crates/ 里的命中数；本机 PCI 上厂商号 0x10de 的设备"
for pattern in NOT_RUN_NO_GPU_ADAPTER decide_gpu_test_start states_reused; do printf '  grep -rn %s crates = ' "$pattern"; grep -rn "$pattern" crates | wc -l; done
for device in /sys/bus/pci/devices/*; do
  vendor="$(cat "$device/vendor" 2>/dev/null)"
  [[ "$vendor" == 0x10de ]] && echo "  $(basename "$device") $vendor $(cat "$device/class")"
done
echo "## 13 mutations.tsv 里点名判法版本的行（键漏判法版本这一类已有变异）"
grep -n '判法版本' crates/mutations.tsv | cut -c1-60

echo "## 14 新记几笔有没有旧行同题（按「要拦什么」「怎么拦」两列扫关键词，列出命中的欠账号）"
for keyword in 弄坏开关 钉表 复用数 快档 加速硬件 平常跑; do
  printf '  %s：' "$keyword"
  awk -F'|' -v keyword="$keyword" 'NR>=16 && $2 ~ /C[0-9]+/ && ($4 ~ keyword || $5 ~ keyword){gsub(/ /,"",$2); printf "%s ", $2}' .claude/kb/checks-owed.md
  echo
done

echo "## 15 T3-3 与 T2-3 同形：crates 里 ignored_points 的命中、欠账表里点名「类名」与 stage-selftest 的行"
grep -rn 'ignored_points' crates | cut -c1-110
printf '  checks-owed.md 里含「类名」的行：'; grep -n '类名' .claude/kb/checks-owed.md | cut -d: -f1 | tr '\n' ' '; echo
printf '  checks-owed.md 里含 stage-selftest 的行：'; grep -n 'stage-selftest' .claude/kb/checks-owed.md | cut -d: -f1 | tr '\n' ' '; echo
