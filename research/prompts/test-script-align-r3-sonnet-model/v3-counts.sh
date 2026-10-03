#!/usr/bin/env bash
# V3 复现：欠账计数、交回里的数、判决里的 find/grep；只读，不编译、不起自证。用法：bash v3-counts.sh（在仓根或任意目录）
cd "$(git rev-parse --show-toplevel)" || exit 2
echo "== C610 带日期的 //! 行 / 文件 / 无 kb 指路行 / 文件 / 日期后跟用户或定 / 其中带 kb 指路 / 既无用户也无定"
grep -E '^//!.*[0-9]{4}-[0-9]{2}-[0-9]{2}' crates/*/tests/*.rs | wc -l
grep -lE '^//!.*[0-9]{4}-[0-9]{2}-[0-9]{2}' crates/*/tests/*.rs | wc -l
grep -E '^//!.*[0-9]{4}-[0-9]{2}-[0-9]{2}' crates/*/tests/*.rs | grep -vE '(D|E|C|I)[0-9]+（|\.claude/kb/|已定项|未定项' | wc -l
grep -E '^//!.*[0-9]{4}-[0-9]{2}-[0-9]{2}' crates/*/tests/*.rs | grep -vE '(D|E|C|I)[0-9]+（|\.claude/kb/|已定项|未定项' | cut -d: -f1 | sort -u | wc -l
grep -E '^//!.*[0-9]{4}-[0-9]{2}-[0-9]{2}.*(用户|定)' crates/*/tests/*.rs | wc -l
grep -E '^//!.*[0-9]{4}-[0-9]{2}-[0-9]{2}.*(用户|定)' crates/*/tests/*.rs | grep -E '(D|E|C|I)[0-9]+（|\.claude/kb/|已定项|未定项' | wc -l
grep -E '^//!.*[0-9]{4}-[0-9]{2}-[0-9]{2}' crates/*/tests/*.rs | grep -vE '用户|定' | wc -l
echo "== C604 tests 顶层 #![cfg( 行数 / 文件数；cfg_attr 命中；NOT_RUN_NO_GPU_ADAPTER 命中；lib.rs 挂特性 6 模块的 #[test] 数"
grep -n '^#!\[cfg(' crates/singlefs-checker-tier/tests/*.rs | wc -l
grep -ln '^#!\[cfg(' crates/singlefs-checker-tier/tests/*.rs | wc -l
grep -rn 'cfg_attr' crates --include=*.rs | cut -c1-120
grep -rn NOT_RUN_NO_GPU_ADAPTER crates | wc -l
t=0; for m in crash_amplification crash_judge_dispatch crash_judge_gpu crash_verify_gpu gpu_unit_checks verdict_store; do n=$(grep -c '#\[test\]' crates/singlefs-checker-tier/src/$m.rs); t=$((t+n)); echo "$m $n"; done; echo "合计 $t"
echo "== C608 mutations.tsv 第五段含 gpu 的行 / 改 .wgsl、crash_verify_gpu、gpu_unit_checks 的行；C605 each_attribute_changes_the_reuse_key 命中"
awk -F'\t' 'NF==6 && $5 ~ /gpu/' crates/mutations.tsv | wc -l
awk -F'\t' 'NF==6 && $2 ~ /\.wgsl|crash_verify_gpu|gpu_unit_checks/' crates/mutations.tsv | wc -l
grep -c each_attribute_changes_the_reuse_key crates/mutations.tsv
echo "== C622 含 _BREAK 的文件：研究脚本（工作区 / 暂存区）、钩子（工作区 / 暂存区）"
grep -l '_BREAK' research/scripts/* 2>/dev/null | wc -l
git grep --cached -l '_BREAK' -- research/scripts | wc -l
grep -l '_BREAK' .claude/hooks/* 2>/dev/null | wc -l
git grep --cached -l '_BREAK' -- .claude/hooks | wc -l
echo "== C620 PCI 厂商号 0x10de 的设备与 class"
for d in /sys/bus/pci/devices/*; do v=$(cat $d/vendor); [ "$v" = 0x10de ] && echo "$d $v $(cat $d/class)"; done
echo "== C623 states_reused 在 crates 里的命中"
grep -rn states_reused crates | wc -l
echo "== C606 DEFAULT_EXPAND_UP_TO"
grep -n 'const DEFAULT_EXPAND_UP_TO: usize' crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
echo "== 登记表：脚本行各判定的份数；样本档各判定的份数；门禁阶段数；--list 格数合计"
awk -F'\t' 'NF>=2 && $1 !~ /^#/ && $1 !~ /gate\.d/ {print $2}' .claude/hooks/selftest-registry.tsv | sed 's/:.*//' | sort | uniq -c
awk -F'\t' 'NF>=2 && $1 ~ /gate\.d/ {print $2}' .claude/hooks/selftest-registry.tsv | sed 's/:.*//' | sort | uniq -c
ls .claude/gate.d/*.sh | wc -l
t=0; for s in .claude/gate.d/*.sh; do n=$(bash $s --list 2>/dev/null | grep -c $'\t'); t=$((t+n)); done; echo "格合计 $t"
echo "== r1 判决 T2-2：54 号四份新样本假工具模式"
find .claude/gate.d/fixtures/checker-tier-layer0-replay.sh/quick-features-*/ -path '*layer0-sample-tools*' -type f -printf '%m %p\n' | sort | awk '{print $1}' | uniq -c
echo "== mutate.sh 文件头 MUTATE_BREAK= 的不同取值数"
grep -oE 'MUTATE_BREAK=[a-z-]+' research/scripts/mutate.sh | sort -u | wc -l
echo "== 术语计数（记录里 H10、改名两处的原数）：全仓除 .git、target、副本；再排 prompts、records、results"
for term in 耗时用例 轻用例 自证 自检; do
  echo "$term 全仓 文件=$(grep -rIl "$term" . --exclude-dir=.git --exclude-dir=target --exclude-dir=singlefs-ai-sop 2>/dev/null | wc -l) 处=$(grep -rIo "$term" . --exclude-dir=.git --exclude-dir=target --exclude-dir=singlefs-ai-sop 2>/dev/null | wc -l)"
  echo "$term 排 prompts/records/results 文件=$(grep -rIl "$term" . --exclude-dir=.git --exclude-dir=target --exclude-dir=singlefs-ai-sop --exclude-dir=prompts --exclude-dir=records --exclude-dir=results 2>/dev/null | wc -l) 处=$(grep -rIo "$term" . --exclude-dir=.git --exclude-dir=target --exclude-dir=singlefs-ai-sop --exclude-dir=prompts --exclude-dir=records --exclude-dir=results 2>/dev/null | wc -l)"
done
