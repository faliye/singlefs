#!/usr/bin/env bash
# K1 全部实测（路径 2、路径 3）加 K4 的 classify 逐项、K5 的判法单元数：在 <工作目录> 下建树、跑、把结果写进 <工作目录>/out/。只算 python，不起 cargo、不编译。
# 用法：k1_run_all.sh <仓根> <工作目录> <部分…>
#   部分：trees  建 base 与各变体（k1_trees.sh）
#         k5     判法单元指纹（带不带弄坏开关 judging-files-included）、各文件的单元数
#         k4     lib_heavy_tests.py 的 classify 逐项问实验复跑入口
#         resolve  整张变异表问每个变体的 resolve_test、mutated_units
#         direct-record direct-check-a direct-check-b direct-compare
#                  50 行取样：executed_units、executed_set_is_current、baseline_reuse_problem
#         wgsl  旁注：覆盖记录里 wgsl 单元此刻找不找得回（要先有 direct-record）
#         plan-seed plan-a plan-b plan-c plan-d plan-summary
#                  两份造好的库 × 各变体 × 两棵树的真 plan 子命令（每段 3 次 plan，约 3 分钟）
# 线程数：capped.sh 31（派发提示「线程上限：31」）；GATE_MUTATION_KEY_WORKERS 取 16。
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1   # 不在模型目录的 tools/ 下留 __pycache__
repo="$1"; work="$2"; shift 2
tools="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export SINGLEFS_RUN_SCOPE='种类=items; 跑什么=K1 探针（crates-mutation-rows.py 的 judging-units、plan 与只读 python 探针）; 范围=整张变异表或取样; 为什么=三方腿复跑 K1/K4/K5 的实测'
capped="$repo/scripts/hosts/capped.sh"
out="$work/out"; mkdir -p "$out" "$work/resolve" "$work/direct" "$work/plans" "$work/seed"
export GATE_MUTATION_KEY_WORKERS=16

has() { local part; for part in "$@"; do [[ " $PARTS " == *" $part "* ]] && return 0; done; return 1; }
PARTS="$*"

if has trees; then
  bash "$tools/k1_trees.sh" "$repo" "$work/w1" > "$out/trees.txt"
fi

if has k5; then
  {
    for v in base v-E1 v-E2 v-E3 v-E4 v-E5 v-cell v-lvl; do
      for flag in "" judging-files-included; do
        printf '%s flag=[%s] ' "$v" "$flag"
        (cd "$work/w1/$v" && GATE_MUTATION_BREAK="$flag" nice -n 19 python3 scripts/mutation/crates-mutation-rows.py judging-units . 2>&1 | head -1)
      done
    done
  } > "$out/k1-judging-digests.txt"
  (cd "$work/w1/base" && GATE_MUTATION_BREAK=judging-files-included nice -n 19 python3 scripts/mutation/crates-mutation-rows.py judging-units . --units) \
    | awk -F'\t' 'NR>1{split($3,a,":"); print a[1]}' | sort | uniq -c | sort -rn > "$out/k5-units-by-file-included.txt"
  (cd "$work/w1/base" && nice -n 19 python3 scripts/mutation/crates-mutation-rows.py judging-units . --units) \
    | awk -F'\t' 'NR>1{split($3,a,":"); print a[1]}' | sort | uniq -c | sort -rn > "$out/k5-units-by-file.txt"
fi

if has k4; then
  python3 -I "$tools/k4_classify.py" --repo "$repo" > "$out/k4-classify.txt"
  bash "$tools/k4_patch_reverse.sh" "$repo" "$work/k4" > "$out/k4-patch-reverse.txt" 2>&1 || true   # 补丁倒着打在现文拷贝上；crates-mutation-rows.py 被别的会话改过时那一行的 sha256 会变，不进 rerun 的比对
  rm -rf "$work/k4"
fi

if has resolve; then
  for v in base v-E1 v-E2 v-E3 v-E4 v-E5; do
    timeout 200 bash "$capped" 31 nice -n 19 python3 -I "$tools/k1_resolve_all.py" --script-repo "$work/w1/$v" --root "$work/w1/base" --out "$work/resolve/$v.jsonl" > /dev/null
  done
  python3 -I "$tools/k1_resolve_summary.py" "$work/resolve" base v-E1 v-E2 v-E3 v-E4 v-E5 > "$out/k1-resolve-summary.txt"
fi

# direct 与 plan 分段，每段在一次前台命令（不到 4 分钟）里跑得完；整套 = direct-record direct-check-a direct-check-b direct-compare plan-seed plan-a plan-b plan-c plan-summary
direct_check() {   # direct_check <标签> <判法所在的变体> <crates 树>
  GATE_MUTATION_RECORDS_HOME="$work/lib-direct" timeout 1500 bash "$capped" 31 nice -n 19 python3 -I "$tools/k1_direct.py" --script-repo "$work/w1/$2" --root "$work/w1/$3" \
    --store "$work/direct/store.json" --out "$work/direct/$1.jsonl" > /dev/null
}
if has direct-record; then
  GATE_MUTATION_RECORDS_HOME="$work/lib-direct" timeout 1500 bash "$capped" 31 nice -n 19 python3 -I "$tools/k1_direct.py" --script-repo "$work/w1/base" --root "$work/w1/base" \
    --store "$work/direct/store.json" --out "$work/direct/base-base.jsonl" --record > /dev/null
fi
if has direct-check-a; then
  direct_check E1-base v-E1 base; direct_check E2-base v-E2 base; direct_check E3-base v-E3 base; direct_check E4-base v-E4 base
fi
if has direct-check-b; then
  direct_check E5-base v-E5 base; direct_check base-t1 base t1; direct_check E5-t1 v-E5 t1; direct_check E4-t1 v-E4 t1
fi
if has wgsl; then
  timeout 1500 bash "$capped" 31 nice -n 19 python3 -I "$tools/k1_wgsl_probe.py" --script-repo "$work/w1/base" --root "$work/w1/base" --store "$work/direct/store.json" > "$out/k1-wgsl-probe.txt"
fi
if has direct-compare; then
  python3 -I "$tools/k1_compare.py" "$work/direct" base-base E1-base E2-base E3-base E4-base E5-base base-t1 E5-t1 E4-t1 > "$out/k1-direct-compare.txt"
fi

plan() {   # plan <标签> <判法所在的变体> <crates 树> <库> <弄坏开关>
  (cd "$work/w1/$2" && GATE_MUTATION_BREAK="$5" GATE_MUTATION_RECORDS_HOME="$work/$4" timeout 1500 bash "$capped" 31 nice -n 19 \
    python3 scripts/mutation/crates-mutation-rows.py plan "$work/w1/$3" > "$work/plans/$1.txt")
}
if has plan-seed; then
  GATE_MUTATION_RECORDS_HOME="$work/lib-seed-base" timeout 1500 bash "$capped" 31 nice -n 19 python3 -I "$tools/k1_seed.py" --script-repo "$work/w1/base" --root "$work/w1/base" > "$work/seed/seed-base.txt"
  GATE_MUTATION_BREAK=judging-files-included GATE_MUTATION_RECORDS_HOME="$work/lib-seed-incl" timeout 1500 bash "$capped" 31 nice -n 19 \
    python3 -I "$tools/k1_seed.py" --script-repo "$work/w1/base" --root "$work/w1/base" > "$work/seed/seed-incl.txt"
fi
# 每一行 plan：<标签> <判法所在的变体> <crates 树> <库> <弄坏开关，- 是不设> <组>；部分写 plan-<标签> 跑那一份，写组名（plan-a…plan-d）跑那一组
PLAN_TABLE='seedbase-base-base base base lib-seed-base - plan-a
seedbase-E1-base v-E1 base lib-seed-base - plan-a
seedbase-E3-base v-E3 base lib-seed-base - plan-a
seedbase-E2-base v-E2 base lib-seed-base - plan-b
seedbase-E4-base v-E4 base lib-seed-base - plan-b
seedbase-base-t1 base t1 lib-seed-base - plan-b
seedbase-E5-base v-E5 base lib-seed-base - plan-c
seedbase-E5-t1 v-E5 t1 lib-seed-base - plan-c
seedbase-E4-t1 v-E4 t1 lib-seed-base - plan-c
seedincl-E5-t1 v-E5 t1 lib-seed-incl judging-files-included plan-d
seedincl-E3-base v-E3 base lib-seed-incl judging-files-included plan-d
seedincl-base-t1 base t1 lib-seed-incl judging-files-included plan-d'
while read -r label variant tree library flag group; do
  if has "plan-$label" "$group"; then
    plan "$label" "$variant" "$tree" "$library" "$([[ "$flag" == "-" ]] || echo "$flag")"
  fi
done <<< "$PLAN_TABLE"
if has plan-summary; then
  python3 -I "$tools/k1_plan_summary.py" "$work/plans" seedbase-base-base seedbase-E1-base seedbase-E2-base seedbase-E3-base seedbase-E4-base seedbase-E5-base \
    seedbase-base-t1 seedbase-E5-t1 seedbase-E4-t1 seedincl-E5-t1 seedincl-E3-base seedincl-base-t1 > "$out/k1-plan-summary.txt"
fi
echo "name=k1_run_all parts=$PARTS work=$work"
