#!/usr/bin/env bash
# admission: always 探针与演示判的是 hook、包装、门禁阶段今天的样子，随时可能改
# run-condition: command python3
# defs-m2-closeout-r2 云端攻方腿的全部复跑：hook 只喂 JSON、只看退出码，被判的命令一条都不执行；
# 包装的两个演示各在自己开的临时 slice（总上限 600M）里跑 true 与一次 400 MiB 的分配，跑完停掉并 revert 那个 slice；
# 40、86 号在仓副本里跑；F5 扫描只读 research/results/。
# 用法（仓根下）：bash research/prompts/defs-m2-closeout-r2-opus-model/rerun.sh [临时目录，默认 ${TMPDIR:-/tmp}]
# 检出记录、仓副本、演示的草稿都建在那个临时目录下；仓副本跑完删掉，检出记录文件路径打在末尾，看完自己删。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
scratch="${1:-${TMPDIR:-/tmp}}"
export TMPDIR="$scratch"
status=0
copy="$(mktemp -d "$scratch/probe-copy.XXXXXX")"
( cd "$root" && rsync -a --exclude target Cargo.toml Cargo.lock crates "$copy/" )
tests="$copy/crates/singlefs-harness/tests"
cp "$tests/second_transaction_step_three_acquisition_barrier_layer0.rs" "$tests/proto_acquisition_barrier.rs"
cp "$tests/second_transaction_step_zero_layer0.rs" "$tests/proto_step_zero.rs"
cd "$root" || exit 2
for cases in cases-f13.json cases-hooks.json cases-dispatch.json; do
  echo "== probe $cases"
  PROBE_COPY_ROOT="$copy" nice -n 19 python3 "$here/probe.py" "$here/$cases" | sed "s#$copy#@COPY@#g" || status=1
done
rm -rf -- "${copy:?}"
for demo in f13-fix-g1.sh nest-demo.sh oom-demo.sh stage-demo.sh rc-stale-demo.sh; do
  echo "== $demo"
  nice -n 19 bash "$here/$demo" 2>&1 | sed -E 's#/home/[^ ]*/rc-stale-demo.sh: line [0-9]+: [0-9]+ Killed#<bash: 作业被 KILL>#' || status=1
done
echo "== verdict-scan.py"
nice -n 19 python3 "$here/verdict-scan.py" || status=1
exit "$status"
