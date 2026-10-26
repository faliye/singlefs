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
  # 正推腿 m3-defs-b6-r1：对着冻结副本喂钩子 / 判法函数，输出写进 $WORK/out，最后与 $MODEL/out/ 里同名文件逐行比（路径换成 <WORK>、<NEW>、<OLD> 再比）。
  # 旧冻结副本（零轮之前那一版）用环境变量 OLD_FROZEN 给；没给时取 $FROZEN 的同级 m3-defs-b3-r2/repo。不读 multi-host.env 的任何值。
  local old out="$WORK/out" tools="$MODEL/tools" trees="$WORK/gate-trees" fr="$FROZEN"
  old="${OLD_FROZEN:-$(dirname "$(dirname "$FROZEN")")/m3-defs-b3-r2/repo}"
  [[ -d "$old/.claude/hooks" ]] || refuse "旧冻结副本 $old 不在（用 OLD_FROZEN 指到零轮之前那一版）"
  mkdir -p "$out" "$WORK/lr"
  # V1/V2/V8：派发闸 --check-prompt 与 stdin 两个入口，两份副本各喂一遍
  nice -n 19 python3 -I "$tools/feed.py" --frozen "$fr" --old "$old" --work "$WORK/feed-work" --raw "$out/feed-raw.txt" > "$out/feed.out" 2>&1 || true
  nice -n 19 python3 -I "$tools/feed_stdin.py" --frozen "$fr" --old "$old" --work "$WORK/stdin-work" > "$out/stdin.out" 2>&1 || true
  # V1：四处类型表逐个比
  nice -n 19 python3 -I "$tools/tables.py" "$fr" > "$out/tables.out" 2>&1 || true
  # V6：登记行判法（库的两种读法）加派发闸 ⑥ 端到端
  nice -n 19 python3 -I "$tools/v6.py" --frozen "$fr" --work "$WORK/v6-work" > "$out/v6.out" 2>&1 || true
  { echo "--- 冻结副本自己的登记表 ⑥ 读法"; python3 -I "$fr/.claude/hooks/lib_abandoned_rounds.py" "$fr"
    echo "--- 冻结副本自己的登记表 门禁读法（--legacy-ok）"; python3 -I "$fr/.claude/hooks/lib_abandoned_rounds.py" "$fr" --legacy-ok; } > "$out/lib-table.out" 2>&1 || true
  # V8：被起脚本文件头的读法（detached-run.sh long-run-check 与 host-dispatch.py 的 header_hosts），豁免表里的与不在表里的同一份内容
  local exempt="scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_one_revision_11_multi_host.sh"
  cp "$fr/$exempt" "$WORK/lr/copy_of_exempt.sh"
  { echo "--- long-run-check 豁免表里的脚本"; (cd "$fr" && bash scripts/hosts/detached-run.sh long-run-check -- bash "$exempt" x; echo "退出=$?")
    echo "--- long-run-check 同一份内容拷到表外"; (cd "$fr" && bash scripts/hosts/detached-run.sh long-run-check -- bash "$WORK/lr/copy_of_exempt.sh" x; echo "退出=$?")
    echo "--- host-dispatch.py header_hosts"; python3 -I "$tools/hd_header.py" "$fr" "$fr/$exempt" "$WORK/lr/copy_of_exempt.sh"
    echo "--- exempt-check 冻结副本自己的表"; python3 -I "$fr/.claude/hooks/lib_long_run.py" exempt-check "$fr"; echo "退出=$?"
    echo "--- 豁免表六份脚本现行 sha256 与 m3-leaf-death-r3-snapshot/tree-sha256.txt 里的钉比"
    local pinned file want have
    pinned="$fr/research/prompts/m3-leaf-death-r3-snapshot/tree-sha256.txt"
    for file in $(grep -h 'multi_host.sh' "$pinned" | awk '{print $2}'); do
      want="$(grep -h "  $file$" "$pinned" | awk '{print $1}')"; have="$(sha256sum "$fr/$file" | awk '{print $1}')"
      if [[ "$want" == "$have" ]]; then echo "same $(basename "$file")"; else echo "DIFF $(basename "$file")"; fi
    done; } > "$out/lr.out" 2>&1 || true
  # V6/V8：造小树，库与派发闸并排看；门禁格本身（--check）归主 agent，命令见报告「要主 agent 跑的命令」
  bash "$tools/make_gate_trees.sh" "$trees" > "$out/gate-trees.out" 2>&1 || true
  { for t in legacy-sourceless other-name-unrecognised symlink-source; do
      echo "--- $t 库默认读法（⑥）"; python3 -I "$fr/.claude/hooks/lib_abandoned_rounds.py" "$trees/$t"
      echo "--- $t 库 --legacy-ok 读法（门禁格）"; python3 -I "$fr/.claude/hooks/lib_abandoned_rounds.py" "$trees/$t" --legacy-ok; done
    echo "--- exempt-check exempt-table 树"; python3 -I "$fr/.claude/hooks/lib_long_run.py" exempt-check "$trees/exempt-table"; echo "退出=$?"; } > "$out/gate-trees-lib.out" 2>&1 || true
  # V6：⑥ 读不到判法库时不收任何叫停的轮
  bash "$tools/nolib.sh" "$fr" "$trees/other-name-unrecognised" "$WORK/nolib-work" > "$out/nolib.out" 2>&1 || true
  # 零散的现查（登记读不到时 ⑮ 放行、SendMessage 钩子不看线程上限、⑥ 不收旧日期行的说明、旧行的轮有没有快照）
  OLD_FROZEN="$old" bash "$tools/adhoc.sh" "$fr" "$trees" "$WORK/adhoc-work" > "$out/adhoc.out" 2>&1 || true
  # V4：capped.sh 的拒绝说明与 default、N 两种取法对 SINGLEFS_CRASH_JUDGE_CPU_THREADS 的态度
  { (cd "$fr" && bash scripts/hosts/capped.sh abc true; echo "退出=$?")
    (cd "$fr" && bash scripts/hosts/capped.sh default true; echo "default 退出=$?")
    (cd "$fr" && bash scripts/hosts/capped.sh 7 bash -c 'echo CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS RUST_TEST_THREADS=$RUST_TEST_THREADS SINGLEFS_CRASH_JUDGE_CPU_THREADS=${SINGLEFS_CRASH_JUDGE_CPU_THREADS-unset}')
    (cd "$fr" && bash scripts/hosts/capped.sh 7 bash scripts/hosts/capped.sh default bash -c 'echo CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS SINGLEFS_THREAD_CAP=$SINGLEFS_THREAD_CAP')
    echo "--- --test-threads 零命中（共用库、双机驱动、崩溃放量）"; (cd "$fr" && grep -c 'test-threads' scripts/crash/crash-amplification-lib.sh scripts/crash/crash-amplification-two-hosts.sh scripts/crash/crash-amplification.sh || true)
    echo "--- admission.py 里的 test-threads（只应有自证临时目录前缀一处）"; (cd "$fr" && grep -n 'test-threads' scripts/gate/admission.py | python3 -I -c 'import sys; [print(line.rstrip("\n")[:100]) for line in sys.stdin]' || true)
    echo "--- 三份定义里的「编译」"; (cd "$fr" && grep -n '编译' .claude/agents/clerk.md .claude/agents/three-way-forward.md .claude/agents/three-way-defense.md | python3 -I -c 'import sys; [print(line.rstrip("\n")[:80]) for line in sys.stdin]' || true); } > "$out/capped.out" 2>&1 || true
  # 路径换成占位再与模型目录里存的比
  local file
  for file in feed.out feed-raw.txt stdin.out tables.out v6.out lib-table.out lr.out gate-trees.out gate-trees-lib.out nolib.out adhoc.out capped.out; do
    [[ -f "$out/$file" ]] || continue
    python3 -I - "$out/$file" "$WORK" "$fr" "$old" <<'PYEOF' || true
import sys
path, work, new, old = sys.argv[1:5]
text = open(path, encoding="utf-8", errors="replace").read()
for real, token in ((work, "<WORK>"), (new, "<NEW>"), (old, "<OLD>")):
    text = text.replace(real, token)
open(path + ".normalized", "w", encoding="utf-8").write(text)
PYEOF
    [[ -f "$out/$file.normalized" ]] || continue
    if [[ -f "$MODEL/out/$file" ]]; then
      if diff -q "$MODEL/out/$file" "$out/$file.normalized" >/dev/null; then echo "name=rerun_compare file=$file result=same"; else echo "name=rerun_compare file=$file result=differs"; fi
    else
      echo "name=rerun_compare file=$file result=no_saved_copy"
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
