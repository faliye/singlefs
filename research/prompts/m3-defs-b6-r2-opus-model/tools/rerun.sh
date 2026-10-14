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

# ── 这一腿要跑的步骤：腿把这一段换成自己的 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 经 "$REPO_ROOT/scripts/hosts/capped.sh" 起，
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
rerun_steps() {
  # 第二份冻结副本（第一轮 after-b6-fixup）与零轮之前那一份从环境变量接：R1_FROZEN（必给）、B3_FROZEN（可不给，不给就不跑 X2 的 b3 一格）。
  # --work 要在工作根（~/.local/share/singlefs/work）底下：X3 仓副本那一支靠它。
  : "${R1_FROZEN:?要给 R1_FROZEN=<第一轮冻结副本>}"
  local out="$WORK/out" v src rc
  mkdir -p "$out"
  for v in r2 r1; do
    src="$FROZEN"; [[ $v == r1 ]] && src="$R1_FROZEN"
    mkdir -p "$WORK/$v"
    rsync -a --exclude target --exclude __pycache__ "$src/.claude" "$src/scripts" "$src/crates" "$src/Cargo.toml" "$src/Cargo.lock" "$WORK/$v/"
    [[ -d "$src/.cargo" ]] && rsync -a "$src/.cargo" "$WORK/$v/"
  done
  cp -a "$MODEL/feeds" "$WORK/feeds"
  local T="$MODEL/tools" F="$WORK/feeds"
  # X1：⑮ 线程上限
  for v in r2 r1; do X1_BASE="$F/x1/base-investigator.txt" bash "$T/x1_feed.sh" "$WORK/$v" investigator "$F/x1/variants.tsv" "$WORK/x1-$v" > "$out/x1-$v-investigator.txt"; done
  # X6：⑭ 多机一行、脚本文件头
  for v in r2 r1; do X1_BASE="$F/x1/base-over2h.txt" bash "$T/x1_feed.sh" "$WORK/$v" investigator "$F/x6/variants.tsv" "$WORK/x6-$v" > "$out/x6-hosts-$v.txt"; done
  for v in r2 r1; do
    for f in h0_canonical h1_note_live h2_wrapped h3_note_eid; do
      rc=0; (cd "$F/x6/scripts" && nice -n 19 python3 "$WORK/$v/.claude/hooks/lib_long_run.py" script "$F/x6/scripts" -- bash "$f.sh") > /dev/null 2> "$WORK/x6-err.txt" || rc=$?
      printf '%s\t%s\trc=%s\t%s\n' "$v" "$f" "$rc" "$(head -1 "$WORK/x6-err.txt" | cut -c1-160)"
    done
  done > "$out/x6-script-headers.txt"
  for v in r2 r1; do
    for f in h1_note_live h3_note_eid; do
      rc=0; (cd "$F/x6/scripts" && nice -n 19 bash "$WORK/$v/scripts/hosts/detached-run.sh" long-run-check -- bash "$f.sh") > "$WORK/x6-dr.txt" 2>&1 || rc=$?
      printf '%s\t%s\tdetached-run long-run-check rc=%s\t%s\n' "$v" "$f" "$rc" "$(head -1 "$WORK/x6-dr.txt" | cut -c1-200)"
    done
  done > "$out/x6-detached-check.txt"
  for v in r2 r1; do echo "== $v"; nice -n 19 python3 "$T/x6_headers.py" "$WORK/$v" "$F/x6/header-scripts.txt" 2>&1 | sed 's#scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/#E171/#'; done > "$out/x6-headers.txt"
  # X2：heavy-thread-defaults（只抽格里的 python 原样跑，基准 none）
  for v in r2 r1; do nice -n 19 python3 "$T/run_htd_cell.py" "$WORK/$v" "$F/x2/sample" > "$out/x2-sample-$v.txt" 2>&1 || true; done
  if [[ -n "${B3_FROZEN:-}" ]]; then nice -n 19 python3 "$T/run_htd_cell.py" "$B3_FROZEN" "$F/x2/sample" > "$out/x2-sample-b3.txt" 2>&1 || true; fi
  for v in r2 r1; do
    mkdir -p "$WORK/lt-$v/.claude"
    rsync -a --exclude fixtures "$WORK/$v/scripts" "$WORK/$v/crates" "$WORK/lt-$v/"
    rsync -a "$WORK/$v/.claude/gate.d" "$WORK/$v/.claude/hooks" "$WORK/lt-$v/.claude/"
    cp "$WORK/$v/.claude/heavy-thread-default" "$WORK/$v/.claude/cargo-command-prefix" "$WORK/lt-$v/.claude/"
    : > "$WORK/lt-$v/.claude/heavy-thread-exempt"
    nice -n 19 python3 "$T/run_htd_cell.py" "$WORK/$v" "$WORK/lt-$v" > "$WORK/x2-noexempt-$v.txt" 2>&1 || true
    grep -m1 '✗' "$WORK/x2-noexempt-$v.txt" | cut -c1-40 > "$out/x2-noexempt-$v.head"
    grep -E '^\s{5}[^ →]+:[0-9]+ ' "$WORK/x2-noexempt-$v.txt" | sed -E 's/^\s+([^:]+):[0-9]+\s+(.*)$/\1\t\2/' | sort > "$out/x2-noexempt-$v.keys"
    cp "$WORK/$v/.claude/heavy-thread-exempt" "$WORK/lt-$v/.claude/heavy-thread-exempt"
    bash "$T/x2_absorb_insert.sh" "$WORK/lt-$v/crates/singlefs-harness/tests/random_histories.rs"
    nice -n 19 python3 "$T/run_htd_cell.py" "$WORK/$v" "$WORK/lt-$v" 2>&1 | grep -E '✓|✗|random_histories|new_device_control' | cut -c1-200 > "$out/x2-absorb-$v.txt" || true
    cp "$F/x2/new_device_control.rs" "$WORK/lt-$v/crates/singlefs-harness/tests/new_device_control.rs"
    nice -n 19 python3 "$T/run_htd_cell.py" "$WORK/$v" "$WORK/lt-$v" 2>&1 | grep -E '✓|✗ [0-9]|random_histories|new_device_control' | cut -c1-200 > "$out/x2-absorb-control-$v.txt" || true
  done
  # X3：重型测试闸的 env 拆法与仓副本那一支
  for v in r2 r1; do nice -n 19 bash "$T/x3_env_feed.sh" "$WORK/$v" > "$out/x3-env-$v.txt" 2>&1; done
  for v in r2 r1; do mkdir -p "$WORK/x3c-$v"; nice -n 19 bash "$T/x3_copy_feed.sh" "$WORK/$v" "$WORK/x3c-$v" > "$out/x3-copy-$v.txt" 2>&1; done
  # X5：private-values 与同一道 doc-text 的三格（样本里的私有值是编的）
  mkdir -p "$WORK/x5-run"; cp -a "$F/x5/pv-mirror-siblings/." "$WORK/x5-run/"; (cd "$WORK/x5-run" && bash setup.sh)
  for v in r2 r1; do
    echo "== $v private-values"; rc=0; nice -n 19 python3 "$T/run_pv_cell.py" "$WORK/$v" "$WORK/x5-run" || rc=$?; echo "rc=$rc"
    for f in clock-times prime-marks session-names; do echo "== $v $f"; rc=0; nice -n 19 python3 "$T/run_scan_form.py" "$WORK/$v" "$WORK/x5-run" "$f" || rc=$?; echo "rc=$rc"; done
  done > "$out/x5-siblings.txt" 2>&1
  nice -n 19 python3 "$T/pv_prefix_probe.py" "$WORK/r2" "$FROZEN/multi-host.env" "$REPO_ROOT" > "$out/x5-prefix-probe.txt" 2>&1 || true
  # X7：豁免表重钉、执行位、行尾
  mkdir -p "$WORK/x7-run"; cp "$F/x7/repin/setup.sh" "$WORK/x7-run/"; (cd "$WORK/x7-run" && bash setup.sh)
  {
    for v in r2 r1; do echo "== $v repin"; rc=0; nice -n 19 python3 "$WORK/$v/.claude/hooks/lib_long_run.py" exempt-check "$WORK/x7-run" --base-table "$WORK/x7-run/base-table.txt" 2>&1 || rc=$?; echo "rc=$rc"; done
    chmod -x "$WORK/x7-run/scripts/live.sh"; echo "== r2 chmod-x"; rc=0; nice -n 19 python3 "$WORK/r2/.claude/hooks/lib_long_run.py" exempt-check "$WORK/x7-run" --base-table "$WORK/x7-run/base-table.txt" 2>&1 || rc=$?; echo "rc=$rc"
    sed -i 's/$/\r/' "$WORK/x7-run/scripts/live.sh"; echo "== r2 crlf"; rc=0; nice -n 19 python3 "$WORK/r2/.claude/hooks/lib_long_run.py" exempt-check "$WORK/x7-run" --base-table "$WORK/x7-run/base-table.txt" 2>&1 || rc=$?; echo "rc=$rc"
  } > "$out/x7-repin.txt"
  mkdir -p "$WORK/x7-r1"; cp "$F/x7/r1-format/setup.sh" "$WORK/x7-r1/"; (cd "$WORK/x7-r1" && bash setup.sh)
  { echo "== r1 改正文、表不动（第一轮写法）"; rc=0; nice -n 19 python3 "$WORK/r1/.claude/hooks/lib_long_run.py" exempt-check "$WORK/x7-r1" --base-table "$WORK/x7-r1/base-table.txt" 2>&1 || rc=$?; echo "rc=$rc"; } > "$out/x7-r1-format.txt"
  # 路径换成占位，逐份与模型目录的 out/ 比
  local file
  for file in "$out"/*; do sed -i "s#$WORK#<WORK>#g; s#$MODEL#<MODEL>#g; s#$REPO_ROOT#<REPO>#g" "$file"; done
  for file in "$out"/*; do
    if cmp -s "$file" "$MODEL/out/$(basename "$file")"; then echo "same $(basename "$file")"; else echo "DIFF $(basename "$file")"; fi
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
