#!/usr/bin/env bash
# test-script-align-r3 正推腿（Sonnet，V2、V3）的复跑。用法：rerun.sh <草稿目录> [--selftests] [--switches]
# 默认只读：grep / awk / git grep / python 读文本、只喂钩子 JSON；不编译、不跑自证。
#   --selftests  另跑登记为 light 的各份 --selftest（selftests.sh，约 2 分钟）
#   --switches   另跑 mutate.sh 自证的 16 个开关（switch-round.sh，约 12 分钟）
# 报告里每一段「rerun.sh 的 X 段」对应下面的「== X ==」；输出写 <草稿目录>/rerun.out。
set -uo pipefail
S="${1:?用法：rerun.sh <草稿目录> [--selftests] [--switches]}"; shift
here="$(cd "$(dirname "$0")" && pwd)"
cd "$(git rev-parse --show-toplevel)" || exit 2
mkdir -p "$S"
exec > >(tee "$S/rerun.out") 2>&1

echo "== V3 欠账计数与交回里的数（v3-counts.sh）"
bash "$here/v3-counts.sh"

echo "== L1 checker 档 tests/*.rs 按 harness 档的一条用例一个场景判"
python3 "$here/one-scenario-on-tier.py"
echo "== L2 用例函数名带来历"
python3 "$here/test-fn-names.py"
echo "== C625 文件头写法的现有份数（启发式）"
python3 "$here/header-survey.py"

echo "== L3 门禁阶段的弄坏开关：含 _BREAK 的阶段文件数、不同变量名数、非注释读取行数、给 _BREAK 赋值再起命令的行"
echo -n "阶段文件数: "; ls .claude/gate.d/*.sh | wc -l
echo -n "含 _BREAK 的阶段文件数: "; grep -l '_BREAK' .claude/gate.d/*.sh | wc -l
echo -n "不同的 *_BREAK* 变量名: "; for f in $(grep -l '_BREAK' .claude/gate.d/*.sh); do grep -oE '[A-Z][A-Z0-9_]*_BREAK[A-Z_]*' "$f"; done | sort -u | wc -l
echo -n "非注释行读 _BREAK 的行数（阶段、共用库、python 库）: "
t=0; for f in .claude/gate.d/*.sh .claude/gate.d/lib/* .claude/gate.d/lib-*.py; do [ -f "$f" ] || continue; n=$(grep -v '^\s*#' "$f" | grep -c '_BREAK'); t=$((t+n)); done; echo "$t"
echo "给 _BREAK 赋值再起命令的非注释行（排除 echo 与文档字符串后只剩 stage-cells.sh:383）："
grep -nE '^[^#]*[A-Z_]+_BREAK=' .claude/gate.d/*.sh .claude/gate.d/lib/* 2>/dev/null | cut -c1-140
echo -n "fixtures 里出现 BREAK 的文件、其中 setup.sh 的份数: "; grep -rl 'BREAK' .claude/gate.d/fixtures | wc -l; grep -rl 'BREAK' .claude/gate.d/fixtures | grep -c 'setup.sh$'
echo -n "source stage-cells.sh 的阶段文件数: "; grep -l 'source .*stage-cells.sh\|^\. .*stage-cells.sh' .claude/gate.d/*.sh | wc -l

echo "== L4 登记为 heavy 的自证"
awk -F'\t' '$2 ~ /^heavy/{print $1" | "$2}' .claude/hooks/selftest-registry.tsv
echo -n "checks-owed 里「假驱动」: "; grep -c '假驱动' .claude/kb/checks-owed.md

echo "== L5 replay.sh 与 broad-mutation.py"
for p in '--list-items' '--item' '--selftest'; do echo -n "replay.sh 里 $p: "; grep -c -- "$p" scripts/experiments/replay.sh; done
awk 'NR==82' scripts/mutation/broad-mutation.py | cut -c1-160

echo "== L6 记录里的尾巴在 checks-owed 里的命中数"
for k in '基线就是红' 'BUILTIN_FEEDS' '版本表' '变异表证过红\|按变异表' 'selftest-pattern-words-judged' '函数名'; do echo -n "$k: "; grep -c "$k" .claude/kb/checks-owed.md; done
echo -n "暂存的 memory-peaks 临时文件: "; git status --short scripts | grep memory-peaks || echo 无
ls -la ~/.local/share/singlefs/memory-peaks.tsv.*.partial 2>/dev/null

echo "== L7 / 1.4 C409 引句在 D13 与 checks-owed 里的命中"
echo -n "D13: "; grep -cF '里程碑收尾、动了崩溃点重放或判器的那一批提交之前 = 全量档全绿' .claude/kb/decisions/13-验证路线.md
echo -n "checks-owed: "; grep -cF '里程碑收尾、动了崩溃点重放或判器的那一批提交之前 = 全量档全绿' .claude/kb/checks-owed.md

echo "== C622 / C625 的上游一侧：stage-selftest.sh 摘的环境、gate.sh 调它"
grep -nF 'env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash "$stage" "$work"' .claude/singlefs-ai-sop/scripts/stage-selftest.sh | cut -c1-120
grep -nF 'stage-selftest.sh" "$ROOT/.claude/gate.d"' .claude/singlefs-ai-sop/scripts/gate.sh

echo "== C623 补丁里 states_reused 的分布（补丁在 /tmp，没合入；不在了就报）"
patch=/tmp/claude-1000/impl-crash-amp-lines/patch
if [[ -f "$patch/crates.patch" ]]; then
  sha256sum "$patch/crates.patch"
  awk '/^\+\+\+ b\//{f=$2} /states_reused/{c[f]++} END{for(k in c) print c[k], k}' "$patch/crates.patch"
  echo -n "mutations-append.tsv 命中行: "; grep -c states_reused "$patch/mutations-append.tsv"
  echo -n "+++ 行指向 crates/ 之外的文件数: "; grep '^+++ b/' "$patch/crates.patch" | grep -vc '^+++ b/crates/'
else echo "补丁不在 $patch"; fi
echo -n "crash-amplification.sh 里非注释行读 crates/ 的行数: "; grep -v '^\s*#' scripts/crash/crash-amplification.sh | grep -c 'crates/'

echo "== 探针：本仓钩子对第三轮正文第二节点名的几条命令（只喂 JSON，implementation-writer 身份）"
probe=research/prompts/test-script-align-r2-opus-model/hook-probe.py
while IFS= read -r c; do python3 "$probe" "$S/det.jsonl" implementation-writer "$PWD" "$c" | cut -c1-170; done <<'CMDS'
bash scripts/mutate.{sh,x} crates/mutations.tsv
bash .claude/gate.d/checker-tier-layer0-replay.{sh,} /home/fy5090/code/singlefs
bash scripts/mutation/mutate.sh crates/mutation?.tsv
ls scripts/{a,b}.sh
cat crates/*.toml
CMDS
echo "== 第二轮攻方 rerun.sh（只喂 JSON）"
bash research/prompts/test-script-align-r2-opus-model/rerun.sh "$S/r2opus" | cut -c1-200

for flag in "$@"; do
  case "$flag" in
    --selftests) echo "== 自证（light，顺序）"; bash "$here/selftests.sh" "$S/selftests"; cat "$S/selftests/s-summary.txt" ;;
    --switches) echo "== mutate.sh 的 16 个开关"; bash "$here/switch-round.sh" "$S/switches"; cat "$S/switches/summary.txt" ;;
    *) echo "不认得的参数 $flag"; exit 2 ;;
  esac
done
