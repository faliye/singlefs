#!/usr/bin/env bash
# admission: always 探针与演示判的是 hook、包装、门禁阶段今天的样子，随时可能改
# run-condition: command python3
# defs-m2-closeout-r3 云端攻方腿的全部复跑：hook 只喂 JSON、只看退出码与 stderr，被判的命令一条都不执行；
# 阶段演示在临时目录的镜像根里跑（上限写错时包装在起 scope 之前退 2，不带单位的上限外壳一起就被停，cargo 一行都不跑）；
# 40 号在只含文件名的副本里跑；弹窗闸与判决行扫描只读。编译一行都没有。
# 用法（仓根下）：bash research/prompts/defs-m2-closeout-r3-opus-model/rerun.sh [临时目录，默认 ${TMPDIR:-/tmp}]
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
scratch="${1:-${TMPDIR:-/tmp}}"
export TMPDIR="$scratch"
status=0
copy="$(mktemp -d "$scratch/probe-copy.XXXXXX")"
( cd "$root" && rsync -a --exclude target research/Cargo.toml research/Cargo.lock research/e7-index-bench "$copy/" )
cd "$root" || exit 2
echo "== probe cases-k1.json"
PROBE_COPY_ROOT="$copy" nice -n 19 python3 "$here/probe.py" "$here/cases-k1.json" | sed "s#$copy#@COPY@#g" || status=1
rm -rf -- "${copy:?}"
echo "== probe cases-k3-detector.json"
nice -n 19 python3 "$here/probe.py" "$here/cases-k3-detector.json" --full-stderr || status=1
echo "== probe cases-k3-ask.json"
nice -n 19 python3 "$here/probe.py" "$here/cases-k3-ask.json" || status=1
for demo in stage-cap-syntax-demo.sh ampersand-demo.sh g2-results-cited-demo.sh; do
  echo "== $demo"
  nice -n 19 bash "$here/$demo" "$scratch" 2>&1 || status=1
done
echo "== cap-syntax-fix.py（改法，只在副本上）"
fix="$(mktemp -d "$scratch/fixdemo.XXXXXX")"
mkdir -p "$fix/.claude/gate.d" "$fix/research" "$fix/crates/singlefs-harness"
ln -s "$root/research/scripts" "$fix/research/scripts"
printf '[workspace]\nmembers = []\n' > "$fix/research/Cargo.toml"
printf '[workspace]\nmembers = []\n' > "$fix/Cargo.toml"
python3 "$here/cap-syntax-fix.py" "$root/.claude/gate.d/15-research-build.sh" "$fix/.claude/gate.d/15-research-build.sh" || status=1
python3 "$here/cap-syntax-fix.py" "$root/.claude/gate.d/74-model-differential.sh" "$fix/.claude/gate.d/74-model-differential.sh" || status=1
for cap in 12GiB 16; do
  GATE_RESEARCH_BUILD_MEMORY_MAX="$cap" bash "$fix/.claude/gate.d/15-research-build.sh" "$fix"; echo "15 cap=$cap exit=$?"
  GATE_MODEL_DIFFERENTIAL_MEMORY_MAX="$cap" SINGLEFS_GATE_FULL=1 bash "$fix/.claude/gate.d/74-model-differential.sh" "$fix"; echo "74 cap=$cap exit=$?"
done
rm -rf -- "${fix:?}"
echo "== claim-scan.py"
nice -n 19 python3 "$here/claim-scan.py" || status=1
echo "== verdict-fields.py"
nice -n 19 python3 "$here/verdict-fields.py" || status=1
exit "$status"
