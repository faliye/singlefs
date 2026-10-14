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
  # 正推腿 m3-defs-b6-r3：Y6 的计数现算、Y4 的钩子喂法；输出写进 $WORK/out，最后与 $MODEL/out/ 里同名文件逐行比（路径换成 <WORK>、<NEW>、<ROOT> 再比）。
  # 冻结副本：--frozen 是第三轮（after-b8-fixup）；第一轮、第二轮、零轮之前那一版用 R1_FROZEN、R2_FROZEN、B3_FROZEN 给，
  # 没给时取 $FROZEN 的同级 m3-defs-b6-r1 / m3-defs-b6-r2 / m3-defs-b3-r2。集成副本工作根与补丁目录取 WORK_ROOT（默认 $FROZEN 的上上级，即 ~/.local/share/singlefs/work）。
  # 不读 multi-host.env 的任何值：pv_counts.py 只打个数。--work 要在 WORK_ROOT 底下（四条旧腿的复跑里 V5 的仓副本样本靠它）。
  local out="$WORK/out" tools="$MODEL/tools" prompts="$REPO_ROOT/research/prompts"
  local work_root="${WORK_ROOT:-$(dirname "$(dirname "$FROZEN")")}"
  local r1="${R1_FROZEN:-$work_root/m3-defs-b6-r1/repo}" r2="${R2_FROZEN:-$work_root/m3-defs-b6-r2/repo}" b3="${B3_FROZEN:-$work_root/m3-defs-b3-r2/repo}"
  local integration="${INTEGRATION:-$work_root/m3-tooling-b2-integration}"
  local parts=" ${RERUN_PARTS:-counts snapshot cite htd pv y4 models-r1 models-r2-opus models-r2-sonnet} "
  local d
  for d in "$r1" "$r2" "$b3" "$integration"; do [[ -d "$d" ]] || refuse "$d 不在（用 R1_FROZEN / R2_FROZEN / B3_FROZEN / INTEGRATION 指过去）"; done
  [[ "$WORK" == "$work_root"/* ]] || refuse "--work 要在 $work_root 底下"
  export PYTHONDONTWRITEBYTECODE=1
  mkdir -p "$out"
  if [[ $parts == *" counts "* ]]; then   # 汇总文件、补丁、提交、排除表行数
    nice -n 19 bash "$tools/summary_counts.sh" "$integration" > "$out/summary-counts.txt" 2>&1 || true
    nice -n 19 bash "$tools/patch_counts.sh" "$work_root" "$integration/repo" > "$out/patch-counts.txt" 2>&1 || true
  fi
  if [[ $parts == *" snapshot "* ]]; then
    nice -n 19 bash "$tools/snapshot_check.sh" "$REPO_ROOT" "$work_root" > "$out/snapshot-check.txt" 2>&1 || true
  fi
  if [[ $parts == *" cite "* ]]; then   # 两轮判决写的 cite-check 行：第一轮 --root 第一轮冻结副本，第二轮 --root 第二轮冻结副本与仓根各一遍
    ( cd "$REPO_ROOT"
      nice -n 19 python3 -B scripts/agents/cite-check.py "$prompts/m3-defs-b6-r1-sonnet-output.md" "$prompts/m3-defs-b6-r1-opus-output.md" --root "$(dirname "$r1")/repo" --background "$prompts/_m3-defs-b6-r1-background.md" 2>&1 | tail -n 1
      nice -n 19 python3 -B scripts/agents/cite-check.py "$prompts/m3-defs-b6-r2-sonnet-output.md" "$prompts/m3-defs-b6-r2-opus-output.md" --root "$r2" 2>&1 | tail -n 1
      nice -n 19 python3 -B scripts/agents/cite-check.py "$prompts/m3-defs-b6-r2-sonnet-output.md" "$prompts/m3-defs-b6-r2-opus-output.md" --root "$REPO_ROOT" 2>&1 | tail -n 1 ) > "$out/cite-check.txt" 2>&1 || true
  fi
  if [[ $parts == *" htd "* ]]; then   # heavy-thread-defaults 格本体（抽格里的 python，不起 --check）在四份副本的真树上：扫的份数、可疑写法、排除表、自己读的线程变量
    { for pair in "b3-r2:$b3" "r1:$r1" "r2:$r2" "r3:$FROZEN"; do
        name="${pair%%:*}"; tree="${pair#*:}"
        echo "name=htd_cell tree=$name $(nice -n 19 python3 -I "$tools/run_htd_cell.py" "$tree" "$tree" 2>&1 | head -n 1 | sed -E 's/.*扫了 ([0-9]+) 份（([^）]*)），可疑写法 ([0-9]+) 处.*（表 ([0-9]+) 行.*线程变量 ([0-9]+) 个.*/scanned=\1 by_dir=\2 suspicious=\3 table_rows=\4 own_variables=\5/')"
        echo "name=scan_count tree=$name $(nice -n 19 python3 -I "$tools/scan_count.py" "$tree")"
      done; } > "$out/htd-cell.txt" 2>&1 || true
    # 攻方样本在第三轮真树上（V3 九种写法、X2 四种从未看过的漏）：哪些文件被点名
    mkdir -p "$WORK/samples"
    cp -a "$prompts/m3-defs-b6-r1-opus-model/feeds/v3/sample" "$WORK/samples/v3"
    cp -a "$prompts/m3-defs-b6-r2-opus-model/feeds/x2/sample" "$WORK/samples/x2"
    { for s in v3 x2; do
        echo "name=htd_sample sample=$s named=$(nice -n 19 python3 -I "$tools/run_htd_cell.py" "$FROZEN" "$WORK/samples/$s" 2>&1 | grep -E '^ +[a-z].*:[0-9]+ ' | awk '{print $1}' | sed 's/:[0-9]*$//' | sort -u | tr '\n' ' ')"
      done; } > "$out/htd-samples.txt" 2>&1 || true
  fi
  if [[ $parts == *" pv "* ]]; then   # 私有配置的词数：只打个数
    nice -n 19 python3 -I "$tools/pv_counts.py" "$FROZEN" > "$out/pv-counts.txt" 2>&1 || true
  fi
  if [[ $parts == *" y4 "* ]]; then   # V2：派发闸 --check-prompt 喂线程上限与已判行几种写法（在第三轮冻结副本的拷贝上）
    mkdir -p "$WORK/r3copy"
    rsync -a --exclude __pycache__ "$FROZEN/.claude" "$FROZEN/scripts" "$FROZEN/Cargo.toml" "$WORK/r3copy/"
    nice -n 19 bash "$tools/y4_feed.sh" "$WORK/r3copy" "$WORK/y4-work" > "$out/y4-feed.txt" 2>&1 || true
    ( cd "$FROZEN" && {
        echo "name=y4_grep file=.claude/agents count_define_thread_limit=$(grep -lF '线程上限' .claude/agents/*.md | wc -l)"
        echo "name=y4_grep continuation-guard_thread_mentions=$(grep -c '线程' .claude/hooks/continuation-guard.sh)"
        echo "name=y4_grep test_threads crash-amplification-lib=$(grep -c 'test-threads' scripts/crash/crash-amplification-lib.sh) two-hosts=$(grep -c 'test-threads' scripts/crash/crash-amplification-two-hosts.sh) crash-amplification=$(grep -c 'test-threads' scripts/crash/crash-amplification.sh) admission=$(grep -c 'test-threads' scripts/gate/admission.py)"
        echo "name=y4_grep crates_mutation_rows=$(awk -F'\t' 'NF==6' crates/mutations.tsv | wc -l) gpu_in_command=$(awk -F'\t' '$5 ~ /gpu/' crates/mutations.tsv | wc -l) gpu_in_feature_list=$(awk -F'\t' '$5 ~ /--features [^ ]*gpu/' crates/mutations.tsv | wc -l) gpu_feature_rows_without_filter=$(awk -F'\t' '$5 ~ /--features [^ ]*gpu/ && $5 !~ /-- [^ ]/' crates/mutations.tsv | wc -l) rows_with_test_threads=$(grep -c 'test-threads' crates/mutations.tsv)"
      } ) > "$out/y4-grep.txt" 2>&1 || true
  fi
  if [[ $parts == *" models-r1 "* || $parts == *" models-r2-opus "* || $parts == *" models-r2-sonnet "* ]]; then   # 四条旧腿各自的 rerun.sh 在拷贝上复跑，只留 same / DIFF 行；分三段是因为单段前台超不过 240 秒
    local name
    mkdir -p "$WORK/models"
    for name in r1-sonnet r1-opus r2-sonnet r2-opus; do cp -a "$prompts/m3-defs-b6-$name-model" "$WORK/models/$name"; done
    mkdir -p "$WORK/rr"
    run_leg() {   # <腿名> <日志名> <冻结副本> [环境变量赋值…]；日志留在 $out，数 same / DIFF 打一行
      local leg="$1" tag="$2" frozen="$3"; shift 3
      env -u RERUN_PARTS "$@" nice -n 19 bash "$WORK/models/$leg/tools/rerun.sh" --repo "$REPO_ROOT" --frozen "$frozen" --work "$WORK/rr/$tag" --model "$WORK/models/$leg" > "$out/model-rerun-$tag.log" 2>&1 || true
      echo "name=model_rerun leg=$tag same=$(grep -c '^same \|result=same' "$out/model-rerun-$tag.log") DIFF=$(grep -c '^DIFF \|result=differs' "$out/model-rerun-$tag.log")"
    }
    {
      if [[ $parts == *" models-r1 "* ]]; then
        run_leg r1-sonnet r1-sonnet "$r1"
        run_leg r1-opus r1-opus "$r1" PRE_ZERO_FROZEN="$b3"
      fi
      if [[ $parts == *" models-r2-opus "* ]]; then
        run_leg r2-opus r2-opus-without-B3 "$r2" R1_FROZEN="$r1"
        run_leg r2-opus r2-opus-with-B3 "$r2" R1_FROZEN="$r1" B3_FROZEN="$b3"
      fi
      if [[ $parts == *" models-r2-sonnet "* ]]; then
        run_leg r2-sonnet r2-sonnet "$r2"
      fi
    } > "$out/model-reruns-$(tr " " - <<<"${RERUN_PARTS:-all}").txt" 2>&1 || true
    # 旧腿的复跑副本很大（每份一份冻结副本的拷贝）：只留 out/，其余删掉
    local leg
    for leg in r1-sonnet r1-opus r2-opus-without-B3 r2-opus-with-B3 r2-sonnet; do
      [[ -d "$WORK/rr/$leg" ]] && find "${WORK:?}/rr/$leg" -mindepth 1 -maxdepth 1 ! -name out -exec rm -rf {} +
    done
    rm -rf "${WORK:?}/models"
  fi
  if [[ $parts == *" htd "* ]]; then rm -rf "${WORK:?}/samples"; fi
  if [[ $parts == *" y4 "* ]]; then rm -rf "${WORK:?}/r3copy"; fi
  # 路径换成占位再与模型目录里存的比
  local file
  for file in "$out"/*.txt; do
    [[ -f "$file" ]] || continue
    python3 -I - "$file" "$WORK" "$FROZEN" "$work_root" <<'PYEOF2' || true
import sys
path, work, new, root = sys.argv[1:5]
text = open(path, encoding="utf-8", errors="replace").read()
for real, token in ((work, "<WORK>"), (new, "<NEW>"), (root, "<ROOT>")):
    text = text.replace(real, token)
open(path + ".normalized", "w", encoding="utf-8").write(text)
PYEOF2
    base="$(basename "$file")"
    if [[ -f "$MODEL/out/$base" ]]; then
      if diff -q "$MODEL/out/$base" "$file.normalized" >/dev/null; then echo "name=rerun_compare file=$base result=same"; else echo "name=rerun_compare file=$base result=differs"; fi
    else
      echo "name=rerun_compare file=$base result=no_saved_copy"
    fi
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
