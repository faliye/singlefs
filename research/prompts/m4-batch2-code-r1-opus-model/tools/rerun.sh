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
# 这一腿的步骤（m4-batch2-code-r1 攻方）：冻结副本拷进 $WORK/tree、把 $MODEL/proto 的两份原型放进 harness 的 tests、逐步跑，
# 每步的规整输出写进 $WORK/out/<步>.txt，与 $MODEL/out/ 同名文件逐行比。环境变量 RERUN_ONLY 给步名（空格分隔）只跑那几步，默认全跑；
# RERUN_TARGET_DIR 给编译目录（默认 $WORK/target）。cargo 经 $REPO_ROOT/scripts/hosts/capped.sh 8 起（整条 rerun 由调用方套内存包装）。
rerun_steps() {
  local tree="$WORK/tree" out="$WORK/out" logs="$WORK/logs" step
  export CARGO_TARGET_DIR="${RERUN_TARGET_DIR:-$WORK/target}"
  mkdir -p "$tree" "$out" "$logs"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$tree/"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  # 模板自证（--selftest）给的假冻结副本里没有 Cargo.toml：只核四个路径与拷贝，不起 cargo
  [[ -f "$tree/Cargo.toml" ]] || return 0
  cp "$MODEL/proto/attack_i113_reasons.rs" "$MODEL/proto/attack_snapshot_and_region_scan.rs" "$tree/crates/singlefs-harness/tests/"
  run_cargo() {   # run_cargo <日志名> <子命令> <cargo 参数…>：在副本里跑一条 cargo，退出码写进日志末行
    local name="$1" subcommand="$2"
    shift 2
    local rc=0
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 8 cargo "$subcommand" --offline --manifest-path "$tree/Cargo.toml" "$@" > "$logs/$name.log" 2>&1 || rc=$?
    echo "cargo_rc=$rc" >> "$logs/$name.log"
  }
  normalize() {   # normalize <日志名> <出口>：只留 ATTACK_ 行、测试结论行与断言两边，去掉耗时，排序（并行测试的次序不定）
    grep -E '^(ATTACK_[A-Z0-9_]+ |test [a-z_:0-9]+ \.\.\. |  left: | right: |cargo_rc=)' "$logs/$1.log" \
      | sed -E 's/ elapsed_s=[0-9.]+//' | LC_ALL=C sort > "$out/$2.txt" || true
  }
  wanted() { [[ -z "${RERUN_ONLY:-}" ]] || [[ " $RERUN_ONLY " == *" $1 "* ]]; }
  local copy="$tree"
  if wanted s1-snapshot-and-region-scan; then
    run_cargo s1 test -p singlefs-harness --test attack_snapshot_and_region_scan -- --ignored --nocapture --test-threads 1
    normalize s1 s1-snapshot-and-region-scan
  fi
  if wanted s2-i113-random-histories; then
    run_cargo s2-build test -p singlefs-harness --test attack_i113_reasons --no-run
    local binary batch
    binary=$(grep -oE '/[^ ()]*/deps/attack_i113_reasons-[0-9a-f]+' "$logs/s2-build.log" | tail -n 1)
    for batch in "b1 1000 8 broad" "b2 2000 16 delete" "b3 3000 16 reuse" "b4 4000 16 orphans"; do
      set -- $batch
      ATTACK_FIRST_SEED="$2" ATTACK_SEEDS="$3" ATTACK_OPS=30 ATTACK_WEIGHTS="$4" ATTACK_THREADS=8 \
        nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 8 "$binary" --ignored --nocapture > "$logs/s2-$1.log" 2>&1 || true
      normalize "s2-$1" "s2-i113-$1"
    done
  fi
  if wanted s3-whitelist-bypass; then
    python3 "$MODEL/tools/mutate_whitelist.py" "$copy/crates/singlefs-checker/src/rebuild.rs" apply
    run_cargo s3 test -p singlefs-checker --lib -- rebuild::tests::the_use_list_of_the_scan_rebuild_is_exactly_the_format_parsing_whitelist rebuild::tests::the_scan_rebuild_names_no_crate_item_outside_the_whitelist_by_a_path
    python3 "$MODEL/tools/mutate_whitelist.py" "$copy/crates/singlefs-checker/src/rebuild.rs" restore
    normalize s3 s3-whitelist-bypass
  fi
  if wanted s4-pairing-equality; then
    python3 "$MODEL/tools/mutate_text.py" "$copy/crates/singlefs-core/src/recovery.rs" apply 'inode_root_entry.tree.0 == extent_root_entry.head_identifier' 'inode_root_entry.tree.0 <= extent_root_entry.head_identifier'
    run_cargo s4 test -p singlefs-harness --test tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds
    run_cargo s4-probe test -p singlefs-harness --test attack_snapshot_and_region_scan -- --ignored --nocapture two_extent_roots
    python3 "$MODEL/tools/mutate_text.py" "$copy/crates/singlefs-core/src/recovery.rs" restore
    normalize s4 s4-pairing-equality-six-tests
    normalize s4-probe s4-pairing-equality-probe
  fi
  if wanted s5-region-scan-unpinned; then
    local label original replacement
    while IFS=$'\t' read -r label original replacement; do
      python3 "$MODEL/tools/mutate_text.py" "$copy/crates/singlefs-core/src/region_scan.rs" apply "$original" "$replacement"
      run_cargo "s5-$label-core" test -p singlefs-core --lib -- region_scan::tests
      run_cargo "s5-$label-harness" test -p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses
      python3 "$MODEL/tools/mutate_text.py" "$copy/crates/singlefs-core/src/region_scan.rs" restore
      normalize "s5-$label-core" "s5-$label-core"
      normalize "s5-$label-harness" "s5-$label-harness"
    done < "$MODEL/tools/region-scan-mutations.tsv"
  fi
  if wanted s6-discard-rows; then
    local row
    for row in 4037 4040; do
      python3 "$MODEL/tools/apply_row.py" "$copy" "$row" apply
      if [[ $row == 4037 ]]; then
        run_cargo "s6-$row" test -p singlefs-harness --test block_devices_record_and_apply_discards -- a_backend_keeping_the_default_answers_unsupported
      else
        run_cargo "s6-$row" test -p singlefs-harness --test block_devices_record_and_apply_discards -- segments_count_a_discard_as_one_plain_write
      fi
      python3 "$MODEL/tools/apply_row.py" "$copy" "$row" restore
      normalize "s6-$row" "s6-row-$row"
    done
  fi
  local file same=0 differ=0 missing=0
  for file in "$out"/*.txt; do
    step=$(basename "$file")
    if [[ ! -f "$MODEL/out/$step" ]]; then
      missing=$((missing + 1)); echo "name=rerun_out file=$step model_out=missing"
    elif cmp -s "$file" "$MODEL/out/$step"; then
      same=$((same + 1)); echo "name=rerun_out file=$step same=1"
    else
      differ=$((differ + 1)); echo "name=rerun_out file=$step same=0"
    fi
  done
  echo "name=rerun_summary same=$same differ=$differ missing_in_model_out=$missing work=$WORK"
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
