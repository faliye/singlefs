#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条经内存包装跑，工作目录随意）：
#   bash scripts/hosts/run-with-memory-cap.sh <上限> bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 scripts/hosts/run-with-memory-cap.sh（判是不是本仓）
#     --frozen  冻结副本（腿开工时拷出的那棵树）：要是一个目录
#     --work    新的草稿目录：不在、或是空目录；跑出来的全写在这里
#     --model   模型目录（原型、变异表、out/）：默认是本脚本的上一层——模型目录整份拷走时它跟着走；只拷这一份脚本时用它指过去
# 退出码：0 跑完；2 用法错（缺参数、仓根不像本仓、冻结副本不在、草稿目录已存在且非空、模型目录不在），stderr 列用法；
#         别的是 rerun_steps 里出错那一步的退出码。--selftest：0 全过，1 有格红。
#
# 准入与运行条件不写、不调 preflight：这份模板拷进模型目录、草稿目录跑，不从自身路径就找不到 .claude/scripts/preflight.sh；
# 环境要求（仓根、冻结副本、草稿目录）由 parse_arguments 自己判，不满足退 2。
#
# 弄坏开关（只给 --selftest 证红用）：
#   RERUN_TEMPLATE_BREAK=root-from-self-path  仓根改回从本脚本路径往上推四层（旧 rerun.sh 的写法），--repo 不认
#       「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」那一格红
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条经 scripts/hosts/run-with-memory-cap.sh 跑" >&2
  cat >&2 <<'USAGE'
用法：rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
      rerun.sh --selftest
USAGE
  exit 2
}

parse_arguments() {
  REPO_ROOT="" FROZEN="" WORK="" MODEL=""
  while (($#)); do
    case "$1" in
      --repo|--frozen|--work|--model)
        (($# >= 2)) || refuse "$1 后面要跟一个路径"
        case "$1" in
          --repo) REPO_ROOT="$2" ;;
          --frozen) FROZEN="$2" ;;
          --work) WORK="$2" ;;
          --model) MODEL="$2" ;;
        esac
        shift 2 ;;
      *) refuse "认不出参数：$1" ;;
    esac
  done
  [[ -n "$REPO_ROOT" ]] || refuse "缺 --repo <仓根>"
  [[ -n "$FROZEN" ]] || refuse "缺 --frozen <冻结副本>"
  [[ -n "$WORK" ]] || refuse "缺 --work <新的草稿目录>"
  if [[ "${RERUN_TEMPLATE_BREAK:-}" == root-from-self-path ]]; then
    REPO_ROOT="$(dirname "${BASH_SOURCE[0]}")/../../../.."
  fi
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 scripts/hosts/run-with-memory-cap.sh）"
  REPO_ROOT="$(cd "$REPO_ROOT" && pwd)"
  [[ -d "$FROZEN" ]] || refuse "--frozen $FROZEN 不是目录"
  FROZEN="$(cd "$FROZEN" && pwd)"
  MODEL="${MODEL:-$(dirname "${BASH_SOURCE[0]}")/..}"
  [[ -d "$MODEL" ]] || refuse "模型目录 $MODEL 不在（只拷了这一份脚本的，用 --model 指过去）"
  MODEL="$(cd "$MODEL" && pwd)"
  if [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
    refuse "--work $WORK 已存在且非空：给一个新的草稿目录，不在旧输出上接着写"
  fi
  mkdir -p "$WORK"
  WORK="$(cd "$WORK" && pwd)"
}

# ── 这一腿要跑的步骤（m3-defs-b6-r3 攻方腿）──
# --frozen 是第三轮冻结副本。第二轮、第一轮、零轮之前那一版用环境变量 R2_FROZEN、R1_FROZEN、B3_FROZEN 给，默认取 --frozen 同级的
# m3-defs-b6-r2/repo、m3-defs-b6-r1/repo、m3-defs-b3-r2/repo。四份都先拷进 $WORK/copies/（不拷 target、.git、research/results 与 multi-host.env），
# 钩子与判法只在拷贝上跑；试改（patches/ 三份）打在第三轮拷贝的再一份拷贝上（$WORK/copies/fix）。
# RERUN_PARTS 选哪几步（默认「y1 y2 y3」；「def」另把第二轮辩方模型的矩阵拿第三轮当 after 重跑一遍、只留汇总，单趟约 2 分钟，与前三步分两趟起）。
# 不读任何真 multi-host.env：y3 的样本根里放的是编的假配置。
rerun_steps() {
  local parent; parent="$(dirname "$(dirname "$FROZEN")")"
  local r2="${R2_FROZEN:-$parent/m3-defs-b6-r2/repo}" r1="${R1_FROZEN:-$parent/m3-defs-b6-r1/repo}" b3="${B3_FROZEN:-$parent/m3-defs-b3-r2/repo}"
  local parts=" ${RERUN_PARTS:-y1 y2 y3} " out="$WORK/out" tools="$MODEL/tools" copies="$WORK/copies" v c f
  export PYTHONDONTWRITEBYTECODE=1
  mkdir -p "$out" "$copies"
  local pairs=("r3=$FROZEN" "r2=$r2" "r1=$r1" "b3=$b3")
  for v in "${pairs[@]}"; do
    [[ -d "${v#*=}/.claude/hooks" ]] || refuse "冻结副本 ${v#*=} 不在（用 R2_FROZEN / R1_FROZEN / B3_FROZEN 指过去）"
    nice -n 19 rsync -a --exclude target --exclude .git --exclude research/results --exclude multi-host.env "${v#*=}/" "$copies/${v%%=*}/"
  done
  nice -n 19 rsync -a "$copies/r3/" "$copies/fix/"
  for f in "$MODEL"/patches/*.diff; do patch -s -p1 -d "$copies/fix" < "$f"; done
  local scrub=(sed -e "s#$WORK#<WORK>#g" -e "s#$MODEL#<MODEL>#g")
  if [[ $parts == *" y1 "* ]]; then
    nice -n 19 python3 -B "$tools/y1_fence_feed.py" --root "r3=$copies/r3" --root "r2=$copies/r2" --root "r1=$copies/r1" --root "b3=$copies/b3" --root "fix=$copies/fix" \
      --work "$WORK/y1-prompts" 2>&1 | "${scrub[@]}" | cut -c1-400 > "$out/y1-fence.txt" || true
    nice -n 19 python3 -B "$tools/y1_stdin_feed.py" --root "r3=$copies/r3" --root "r2=$copies/r2" --prompts "$WORK/y1-prompts" --work "$WORK/y1-stdin" \
      F0-control-31 F3-fence-shift-16 F2-fence-shift-bracket-load H2-fence-shift-live 2>&1 | "${scrub[@]}" | cut -c1-200 > "$out/y1-stdin.txt" || true
    for v in r3 r2 r1; do nice -n 19 bash "$tools/y1_drift.sh" "$copies/$v" "$WORK/y1-drift-$v" 2>&1 | cut -c1-200 > "$out/y1-drift-$v.txt" || true; done
    mkdir -p "$WORK/y1-drift-b3/records" "$WORK/y1-drift-b3/research/prompts/dr-r1-snapshot"
    cp -a "$WORK/y1-drift-r3/research/prompts/." "$WORK/y1-drift-b3/research/prompts/"
    printf '# 样本调度\n\n- 2026-10-13 用户叫停 dr-r1：「这一轮先放一放，别再派腿了」\n- dr-r1 之后的活并进别处。\n' > "$WORK/y1-drift-b3/records/log.md"
    nice -n 19 python3 -B "$tools/y1_drift_b3.py" "$copies/b3" "$WORK/y1-drift-b3" 2>&1 | cut -c1-200 > "$out/y1-drift-b3.txt" || true
  fi
  if [[ $parts == *" y2 "* ]]; then
    for v in r3 r2 r1 b3 fix; do nice -n 19 bash "$tools/y2_env_feed.sh" "$copies/$v" 2>&1 | "${scrub[@]}" > "$out/y2-env-$v.txt" || true; done
    for v in r3 fix; do nice -n 19 bash "$tools/y2_allow_feed.sh" "$copies/$v" > "$out/y2-allow-$v.txt" 2>&1 || true; done
    for v in r3 r2 b3; do nice -n 19 python3 -B "$tools/run_htd_cell.py" "$copies/$v" "$MODEL/feeds/y2htd/sample" 2>&1 | "${scrub[@]}" | cut -c1-260 > "$out/y2-htd-$v.txt" || true; done
  fi
  if [[ $parts == *" y3 "* ]]; then
    for c in symlink-dangling symlink-mount unreadable; do
      mkdir -p "$WORK/y3/$c"; (cd "$WORK/y3/$c" && MOUNT_ROOT="$WORK/y3/mnt-$c" bash "$MODEL/feeds/y3/setup.sh" "$c")
      { for v in r3 r2 r1 b3 fix; do echo "=== $c $v"; (cd "$WORK/y3" && nice -n 19 python3 -B "$tools/run_pv_cell3.py" "$copies/$v" "$WORK/y3/$c"; echo "rc=$?") 2>&1 | tail -n 6; done; } \
        | "${scrub[@]}" | cut -c1-220 > "$out/y3-$c.txt" || true
    done
    { for c in symlink-dangling symlink-mount; do (cd "$WORK/y3/$c" && git add -A && git diff --cached --name-only | while read -r f; do
          printf '%s 模式=%s 进仓的内容=%s\n' "$f" "$(git ls-files -s -- "$f" | cut -d' ' -f1)" "$(git cat-file -p ":$f")"; done); done; } 2>&1 | "${scrub[@]}" > "$out/y3-git-blob.txt" || true
    chmod -R u+rwX "$WORK/y3/unreadable"
  fi
  if [[ $parts == *" def "* ]]; then
    local defense="$REPO_ROOT/research/prompts/m3-defs-b6-r2-sonnet-model"
    BEFORE_FROZEN="$copies/r2" PRE0_FROZEN="$copies/b3" RERUN_PARTS="fm feed v5 v6 htd" nice -n 19 bash "$defense/tools/rerun.sh" --repo "$REPO_ROOT" --frozen "$copies/r3" --work "$WORK/def" > "$WORK/def.log" 2>&1 || true
    { for f in "$WORK"/def/out/fm-*.txt; do awk -F'\t' -v f="$(basename "$f")" '{a=$2;b=$3; sub("after=","",a); sub("before=","",b); n++; if(a!=b) d[a">"b]++} END{printf "%s 行=%d", f, n; for(k in d) printf " r3=%s,r2=%s:%d", substr(k,1,1), substr(k,3,1), d[k]; printf "\n"}' "$f"; done
      awk '{for (i = 1; i <= NF; i++) if ($i ~ /^\[(after|before):/) { split($i, cells, "]"); for (j in cells) if (cells[j] ~ /^\[(after|before):/) count[cells[j] "]"]++ }} END {for (k in count) print "v5-env-matrix " k " " count[k]}' "$WORK/def/out/v5-env-matrix.txt" | sort
      for n in wrap_helper alias_env copied_env use_alias vars_iter project_wrapper harness_wrapper or_default and_default lib.rs; do echo "v3 $n r3=$(grep -c "$n" "$WORK/def/out/v3-after.txt") r2=$(grep -c "$n" "$WORK/def/out/v3-before.txt")"; done
      diff <(grep -v '^\s' "$WORK/def/out/v5-chdir-after.txt" | cut -f1,2) <(grep -v '^\s' "$WORK/def/out/v5-chdir-before.txt" | cut -f1,2) && echo "v5-chdir r3 与 r2 逐行同"
      sed -n '1,6p' "$WORK/def/out/x4-real-table.txt"; } > "$out/def-summary.txt" 2>&1 || true
  fi
  echo "name=rerun_outputs work=$WORK"
  for f in "$out"/*; do
    if [[ -f "$MODEL/out/$(basename "$f")" ]] && cmp -s "$f" "$MODEL/out/$(basename "$f")"; then echo "name=rerun_compare file=$(basename "$f") result=same"
    else echo "name=rerun_compare file=$(basename "$f") result=differs"; fi
  done
}

# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  mkdir -p "$scratch/fake-repo/.git" "$scratch/fake-repo/scripts/hosts" "$scratch/frozen/crates/target" \
    "$scratch/elsewhere/r1-opus-model/tools" "$scratch/cwd"
  : > "$scratch/fake-repo/scripts/hosts/run-with-memory-cap.sh"
  echo 冻结副本里的文件 > "$scratch/frozen/crates/marker.txt"
  echo 编译产物 > "$scratch/frozen/crates/target/junk"
  copy="$scratch/elsewhere/r1-opus-model/tools/rerun.sh"
  cp "${BASH_SOURCE[0]}" "$copy"

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1") \
      > "$scratch/run1.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 0 ]] || ! grep -qF "name=rerun_roots repo=$scratch/fake-repo frozen=$scratch/frozen model=$scratch/elsewhere/r1-opus-model " "$scratch/run1.out" \
      || [[ ! -f "$scratch/work1/tree/crates/marker.txt" || -e "$scratch/work1/tree/crates/target" ]]; then
    failures+=("拷进与仓无关的临时目录、传参应当跑通（退 0、仓根取 --repo、冻结副本拷进草稿目录、target 不拷），实际退 $rc：$(tail -n 4 "$scratch/run1.out")")
  fi

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy") > "$scratch/run2.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "用法：rerun.sh --repo" "$scratch/run2.out"; then
    failures+=("不传参应当退 2 并列出用法，实际退 $rc：$(tail -n 4 "$scratch/run2.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --work "$scratch/work3" > "$scratch/run3.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "缺 --frozen" "$scratch/run3.out"; then
    failures+=("缺 --frozen 应当退 2 并点名缺它，实际退 $rc：$(tail -n 4 "$scratch/run3.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/cwd" --frozen "$scratch/frozen" --work "$scratch/work4" > "$scratch/run4.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run4.out"; then
    failures+=("--repo 指到不是仓的目录应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run4.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1" > "$scratch/run5.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "已存在且非空" "$scratch/run5.out"; then
    failures+=("--work 指到已有输出的目录应当退 2、不在旧输出上接着写，实际退 $rc：$(tail -n 4 "$scratch/run5.out")")
  fi

  if ((${#failures[@]})); then
    local failure
    for failure in "${failures[@]}"; do
      echo "  ✗ 自检：$failure"   # gate-lint:detail
    done
    echo "    → 怎么办：看 parse_arguments() 与 rerun_steps()；RERUN_TEMPLATE_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 格）：拷进与仓无关的临时目录、从别的目录起，传参跑通；不传参、缺 --frozen、--repo 不是仓、--work 非空都退 2 并说明"
  return 0
}

if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
