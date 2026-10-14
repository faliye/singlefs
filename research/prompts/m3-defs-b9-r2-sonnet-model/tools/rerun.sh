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

# ── 这一腿要跑的步骤：m3-defs-b9-r2 辩方腿（Sonnet） ──
# $FROZEN＝这一轮冻结副本（after-b10-fixup）；第一轮冻结副本（after-b9-fixup）取环境变量 B9_FROZEN，默认 $FROZEN 同级的 m3-defs-b9-r1/repo。
# 先各造一棵只含 .claude、scripts、records 与 research/prompts 里叫停登记/轮名材料的小树（不拷 multi-host.env 与 crates），再按 RERUN_PARTS（默认全跑）跑：
#   r1sonnet  第一轮正推腿模型里的六个喂法，b9、b10 各一遍            r1opus  第一轮攻方腿模型里的喂法（围栏、叫停表三种改动、五格 doc-text、豁免表、shell 样本），b9、b10 各一遍
#   s1..s7    本腿的新喂法（每格对应正文「三」的专问）
# 输出写进 $WORK/out，路径与随时间变的东西换成占位后与 $MODEL/out/ 同名文件逐字比，末尾逐份打 same / DIFF / 新出。不起 cargo、不跑 --selftest、不跑门禁阶段。
make_small_tree() {   # 源冻结副本 目标目录
  local source="$1" target="$2"
  mkdir -p "$target/research/prompts" "$target/research/results"
  rsync -a --exclude target --exclude .git "$source/.claude" "$source/scripts" "$source/records" "$target/"
  rsync -a --include='abandoned-rounds.tsv' --include='_*-body.md' --include='*-main-verification.md' --exclude='*' "$source/research/prompts/" "$target/research/prompts/"
  local name
  while read -r name; do mkdir -p "$target/research/prompts/$name"; done < <(cd "$source/research/prompts" && find . -maxdepth 1 -name '*-snapshot' -type d -printf '%f\n')
}

rerun_steps() {
  local b9="${B9_FROZEN:-$(dirname "$FROZEN")/../m3-defs-b9-r1/repo}"
  [[ -d "$b9" ]] || refuse "第一轮冻结副本 $b9 不在：用环境变量 B9_FROZEN 指过去"
  b9="$(cd "$b9" && pwd)"
  local parts="${RERUN_PARTS:-r1sonnet r1opus s1 s2 s3 s4 s5 s6 s7}" tools="$MODEL/tools" O="$WORK/raw" out="$WORK/out" S="$WORK/scratch"
  local r1s="$REPO_ROOT/research/prompts/m3-defs-b9-r1-sonnet-model/tools" r1o="$REPO_ROOT/research/prompts/m3-defs-b9-r1-opus-model"
  local t9="$WORK/t9" t10="$WORK/t10" label tree
  export PYTHONDONTWRITEBYTECODE=1
  mkdir -p "$O" "$out" "$S"
  make_small_tree "$b9" "$t9"; make_small_tree "$FROZEN" "$t10"
  part() { case " $parts " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }
  local N="nice -n 19"
  if part r1sonnet; then
    for label in b9 b10; do
      tree="$t9"; [[ "$label" == b10 ]] && tree="$t10"
      $N python3 -B "$r1s/z1_thread_prompt_cases.py" "$tree" > "$O/r1s-thread-prompt-$label.tsv"
      $N python3 -B "$r1s/z6_long_run_prompt_cases.py" "$tree" > "$O/r1s-long-run-prompt-$label.tsv"
      $N python3 -B "$r1s/z3_heavy_guard_feed.py" "$tree" "$S/r1s-z3-$label" > "$O/r1s-heavy-guard-$label.txt"
      $N python3 -B "$r1s/z4_abandoned_cases.py" "$tree" "$S/r1s-z4-$label" > "$O/r1s-abandoned-$label.tsv"
      $N python3 -B "$r1s/z6_script_header_cases.py" "$tree" "$S/r1s-z6h-$label" > "$O/r1s-script-header-$label.tsv"
      $N python3 -B "$r1s/z7_exempt_cases.py" "$tree" "$S/r1s-z7-$label" > "$O/r1s-exempt-$label.tsv"
      $N python3 -B "$tree/.claude/hooks/lib_abandoned_rounds.py" "$tree" > "$O/r1s-real-abandoned-table-$label.txt" || true
      $N python3 -B "$tree/.claude/hooks/lib_long_run.py" exempt-check "$tree" > "$O/r1s-real-exempt-table-$label.txt" || true
    done
  fi
  if part r1opus; then
    $N python3 -B "$r1o/tools/z1_fence_feed.py" --root b10="$t10" --root b9="$t9" --work "$S/o-z1" > "$O/r1o-fence-feed.txt" 2>&1
    $N python3 -B "$r1o/tools/z1_stdin_feed.py" --root b10="$t10" --root b9="$t9" --prompts "$S/o-z1" --work "$S/o-z1-stdin" \
      T1-tilde-lone-fence-16 T2-list-fence-container-end-16 O2-nested-list-fence-example > "$O/r1o-stdin-feed.txt" 2>&1
    { bash "$r1o/tools/z4_source_drift.sh" b10="$t10" "$t10" "$S/o-z4" excerpt; bash "$r1o/tools/z4_source_drift.sh" b9="$t9" "$t10" "$S/o-z4" excerpt
      bash "$r1o/tools/z4_source_drift.sh" b10="$t10" "$t10" "$S/o-z4" line; bash "$r1o/tools/z4_source_drift.sh" b9="$t9" "$t10" "$S/o-z4" line; } > "$O/r1o-z4-drift.txt" 2>&1
    { bash "$r1o/tools/z4_real_dispatch.sh" b10="$FROZEN" "$S/o-z4-real"; bash "$r1o/tools/z4_real_dispatch.sh" b9="$b9" "$S/o-z4-real"; } > "$O/r1o-z4-real.txt" 2>&1
    bash "$r1o/tools/z4_misc.sh" "$S/o-z4-misc" b10="$t10" b9="$t9" > "$O/r1o-z4-misc.txt" 2>&1
    mkdir -p "$S/o-z5-sample"; ( cd "$S/o-z5-sample" && bash "$r1o/feeds/z5/setup.sh" > /dev/null 2>&1 )
    for label in b9 b10; do
      tree="$t9"; [[ "$label" == b10 ]] && tree="$t10"
      $N python3 -B "$tools/s6_cells.py" "$r1o/tools/z5_cells.py" "$tree" "$S/o-z5-sample" clock-times date-once term-renames private-values > "$O/r1o-z5-cells-$label.txt" 2>&1
      $N python3 -B "$r1o/tools/z2_htd_cell.py" "$tree" "$r1o/feeds/z2/sample" > "$O/r1o-z2-shell-$label.txt" 2>&1 || true
    done
    { bash "$r1o/tools/z7_exempt_repin.sh" b10="$FROZEN" "$S/o-z7"; bash "$r1o/tools/z7_exempt_repin.sh" b9="$b9" "$S/o-z7"; } > "$O/r1o-z7.txt" 2>&1
  fi
  if part s1; then
    $N python3 -B "$tools/s1_construct_matrix.py" "$t9" "$t10" "$S/s1m" > "$O/s1-construct-matrix.txt" 2>&1
    $N python3 -B "$tools/s1_differential_fuzz.py" "$t9" "$t10" 1 20000 > "$O/s1-differential-fuzz.txt" 2>&1
    { for seed in 1 2 3 4 5 6 7 8 9; do $N python3 -B "$tools/s1_fuzz_without_code_blocks.py" "$t10" "$seed" 3000 2>/dev/null; done; } > "$O/s1-fuzz-b10.txt"
    $N bash "$tools/s1_library_missing.sh" "$t10" "$MODEL/feeds/shim" "$S/s1lib" > "$O/s1-library-missing.txt" 2>&1
    $N python3 -B "$tools/s1_fallback_trial.py" "$t10" "$MODEL/feeds/shim" "$S/s1fb" "$tools/s1_library_missing.sh" > "$O/s1-fallback-trial.txt" 2>&1
  fi
  if part s2; then $N python3 -B "$tools/s2_exit_matrix.py" "$t9" "$t10" "$S/s2" > "$O/s2-exit-matrix.txt" 2>&1; fi
  if part s3; then
    cp -a "$MODEL/feeds/s3/sample" "$S/s3-sample"
    for label in b9 b10; do
      tree="$t9"; [[ "$label" == b10 ]] && tree="$t10"
      $N python3 -B "$r1o/tools/z2_htd_cell.py" "$tree" "$S/s3-sample" > "$O/s3-cell-$label.txt" 2>&1 || true
    done
    $N bash "$tools/s3_fixture_both.sh" "$t9" "$t10" "$S/s3fx" "$r1o/tools/z2_htd_cell.py" > "$O/s3-fixtures-both.txt" 2>&1
  fi
  if part s4; then
    for label in b9 b10; do
      tree="$t9"; [[ "$label" == b10 ]] && tree="$t10"
      $N python3 -B "$tools/s4_heavy_guard_feed.py" "$tree" "$S/s4-$label" > "$O/s4-heavy-guard-$label.txt" 2>&1
    done
  fi
  if part s5; then
    $N python3 -B "$tools/s5_drift_matrix.py" b9="$t9" b10="$t10" --work "$S/s5" > "$O/s5-drift-matrix.txt" 2>&1
    LIB_ABANDONED_ROUNDS_BREAK=line-number-read $N python3 -B "$tools/s5_drift_matrix.py" b10-line-number-read="$t10" --work "$S/s5lnr" > "$O/s5-line-number-read.txt" 2>&1   # 库自带的弄坏开关：退回按行号读（第一轮 r3 那一版的判法）
    $N python3 -B "$tools/s5_real_records.py" "$t9" "$t10" "$S/s5r" > "$O/s5-real-records.txt" 2>&1
    $N python3 -B "$tools/s5_similarity_trial.py" "$t10" "$S/s5sim" "$tools/s5_drift_matrix.py" > "$O/s5-similarity-trial.txt" 2>&1
    { for fixture in abandoned-rounds-source-green abandoned-rounds-source-red; do
        for label in b9 b10; do
          tree="$t9"; [[ "$label" == b10 ]] && tree="$t10"
          printf '## %s / %s 的库\n' "$fixture" "$label"
          ( cd "$t10/.claude/gate.d/fixtures/doc-process-records.sh/$fixture" && python3 -B "$tree/.claude/hooks/lib_abandoned_rounds.py" . 2>&1 | awk -F'\t' '{print $1 "\t" substr($2, 1, 40)}' ) || true
        done
      done; } > "$O/s5-fixtures-lib.txt"
  fi
  if part s6; then
    mkdir -p "$S/s6-plain"; ( cd "$S/s6-plain" && bash "$MODEL/feeds/s6/setup.sh" > /dev/null 2>&1 )
    cp -a "$S/s6-plain" "$S/s6-anchor"
    printf '%s\n' "records/2026-10-13-样本调度.md:「不存在的锚 sample-second-host-77 之类」  # 行内锚对不上，sweep-term 会把锚原样打出" >> "$S/s6-anchor/.claude/term-rename-exempt"
    for label in b9 b10; do
      tree="$t9"; [[ "$label" == b10 ]] && tree="$t10"
      $N python3 -B "$tools/s6_cells.py" "$r1o/tools/z5_cells.py" "$tree" "$S/s6-plain" clock-times prime-marks session-names date-once term-renames private-values > "$O/s6-cells-$label.txt" 2>&1
      $N python3 -B "$tools/s6_cells.py" "$r1o/tools/z5_cells.py" "$tree" "$S/s6-anchor" term-renames > "$O/s6-anchor-$label.txt" 2>&1
    done
  fi
  if part s7; then $N python3 -B "$tools/s7_prose_feed.py" "$t9" "$t10" "$S/s7" > "$O/s7-prose-feed.txt" 2>&1; fi
  local file name
  for file in "$O"/*; do
    name="$(basename "$file")"
    sed -e "s#$WORK#<WORK>#g" -e "s#$MODEL#<MODEL>#g" -e "s#$FROZEN#<FROZEN-b10>#g" -e "s#$b9#<FROZEN-b9>#g" -e "s#$REPO_ROOT#<REPO>#g" "$file" \
      | NORMALIZE_NAME="$name" python3 -B -c 'import os, re, sys
text = sys.stdin.read()
if os.environ["NORMALIZE_NAME"] in ("r1o-z7.txt", "r1s-script-header-b9.tsv", "r1s-script-header-b10.tsv"):   # 这三份的喂法脚本先按字数截行、之后才换占位：草稿目录路径长短不同，截剩的尾巴不同，只留最后一栏的前 8 个字（✓ 或 ✗ 与头几个字）
    text = "".join("\t".join(line.split("\t")[:-1] + [line.split("\t")[-1][:8]]) + "\n" for line in text.split("\n") if line)
text = re.sub("[" + "".join(map(chr, (8242, 8243, 8244, 697, 698))) + "]", "<撇号>", text)
text = re.sub(r"(\d{4}-\d{2}-\d{2})T\d{2}:\d{2}:\d{2}", r"\1T<时分秒>", text)
text = text.replace("singlefs" + "-ab12cd", "singlefs-<十六进制>")
text = re.sub(r"/tmp/claude-[0-9]+/[^ \t\n）)]*", "<TMP>", text)
sys.stdout.write(text)' > "$out/$name"
    if [[ ! -f "$MODEL/out/$name" ]]; then echo "新出 $name"
    elif cmp -s "$out/$name" "$MODEL/out/$name"; then echo "same $name"
    else echo "DIFF $name"; fi
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
