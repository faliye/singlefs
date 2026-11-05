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
#       连带红的格：「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
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
  local W="$WORK" side f fam t
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  export REPO_ROOT   # 不 export SINGLEFS_WORK_ROOT：自证与第二轮腿乙的案例要用真的工作根（feedhooks.py 喂钩子时自己给每次调用设）
  unset SINGLEFS_WORK_ROOT
  mkdir -p "$W/tools" "$W/template" "$W/oracle" "$W/cases" "$W/res" "$W/out" "$W/tmp" "$W/trees"
  cp -r "$MODEL/tools/." "$W/tools/"; cp -r "$MODEL/template/." "$W/template/"; cp -r "$MODEL/oracle/." "$W/oracle/"
  for f in "$MODEL"/cases/*.json; do sed "s#__W__#$W#g" "$f" > "$W/cases/$(basename "$f")"; done
  cp -r "$W/template" "$W/scratch"; mkdir -p "$W/scratch-r1/sub"; cp "$W"/template/s.sh "$W"/template/light.sh "$W"/template/s01.sh "$W"/template/s02.sh "$W/scratch-r1/"
  # 主工作区替身（编译目录一道要在 git 工作树里）
  mkdir -p "$W/mainws/crates/singlefs-core/src" "$W/mainws/sub" "$W/mainws/records" "$W/mainws/scripts"
  (cd "$W/mainws" && git init -q . && echo 'fn main(){}' > crates/singlefs-core/src/lib.rs && printf '[workspace]\nmembers=["crates/*"]\n' > Cargo.toml)
  # 四棵树：now = 冻结副本五份钩子；r2 = 打回 fix2.diff；r1 = 再打回 fix.diff；pre = 再打回 r1.diff（第一轮判决之前，guard 里 3 个块打不回）
  for side in now r2 r1 pre; do
    mkdir -p "$W/trees/$side/.claude/singlefs-ai-sop" "$W/trees/$side/scripts" "$W/trees/$side/crates/singlefs-core/src" "$W/trees/$side/.claude/gate.d"
    cp -r "$REPO_ROOT/.claude/hooks" "$REPO_ROOT/.claude/scripts" "$REPO_ROOT/.claude/agents" "$W/trees/$side/.claude/"
    cp -r "$REPO_ROOT/.claude/singlefs-ai-sop/scripts" "$W/trees/$side/.claude/singlefs-ai-sop/"
    cp "$REPO_ROOT/.claude/heavy-thread-default" "$W/trees/$side/.claude/"
    for f in hosts gate mutation crash agents devices kb; do cp -r "$REPO_ROOT/scripts/$f" "$W/trees/$side/scripts/"; done
    cp "$REPO_ROOT"/.claude/gate.d/*.sh "$REPO_ROOT"/.claude/gate.d/*.tsv "$W/trees/$side/.claude/gate.d/" 2>/dev/null || true
    for f in bash-command-detector.sh heavy-test-guard.sh lib_shell_words.py lib_heavy_tests.py lib_workspace_cargo_lock.py; do
      rm -f "$W/trees/$side/.claude/hooks/$f"; cp "$FROZEN/.claude/hooks/$f" "$W/trees/$side/.claude/hooks/$f"
    done
    echo 'fn main(){}' > "$W/trees/$side/crates/singlefs-core/src/lib.rs"
  done
  (cd "$W/trees/r2" && patch -R -p1 -s < "$FROZEN/fix2.diff")
  for f in .claude/hooks/bash-command-detector.sh .claude/hooks/heavy-test-guard.sh .claude/hooks/lib_shell_words.py .claude/hooks/lib_heavy_tests.py .claude/hooks/lib_workspace_cargo_lock.py; do
    rm -f "$W/trees/r1/$f" "$W/trees/pre/$f"; cp "$W/trees/r2/$f" "$W/trees/r1/$f"
  done
  (cd "$W/trees/r1" && patch -R -p1 -s < "$FROZEN/fix.diff")
  for f in .claude/hooks/bash-command-detector.sh .claude/hooks/heavy-test-guard.sh .claude/hooks/lib_shell_words.py .claude/hooks/lib_heavy_tests.py .claude/hooks/lib_workspace_cargo_lock.py; do cp "$W/trees/r1/$f" "$W/trees/pre/$f"; done
  (cd "$W/trees/pre" && python3 -I - "$FROZEN/r1.diff" <<'PY'
import re, subprocess, sys
for part in re.split(r"(?m)^(?=diff --git )", open(sys.argv[1], encoding="utf-8").read()):
    if part.strip():
        name = re.search(r"\+\+\+ b/own/(\S+)", part).group(1)
        subprocess.run(["patch", "-R", "-s", "-r", "/dev/null", ".claude/hooks/" + name], input=part, text=True, check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
PY
  )
  for side in now r2 r1 pre; do (cd "$W/trees/$side/.claude/hooks" && sha256sum lib_shell_words.py bash-command-detector.sh heavy-test-guard.sh lib_heavy_tests.py lib_workspace_cargo_lock.py | cut -c1-12,65- | sed "s#^#name=fingerprint_$side #"); done
  cd "$W"
  if [[ -n "${RERUN_REUSE_RES:-}" ]]; then cp -r "$RERUN_REUSE_RES/res/." "$W/res/"; echo "name=reuse_res from=$RERUN_REUSE_RES files=$(ls "$W/res" | wc -l)"; fi
  if [[ -z "${RERUN_REUSE_RES:-}" ]]; then
  # 1) 第一轮、第二轮两条腿登记的样本族（腿的 mkcases.py 造的清单，已拷进 cases/）：真 bash 裁判一遍，r1 / r2 / now 三棵树各喂一遍
  for fam in key cpforms brace heredoc redirect ln env shell builtin function fncall empties hwrite ansic fuzz2 head1; do nice -n 19 bash tools/runfam.sh "$fam" > "$W/out/run-$fam.log" 2>&1; echo "name=family_$fam $(tail -n 1 "$W/out/run-$fam.log" | cut -c1-100)"; done
  nice -n 19 bash tools/runfam.sh r1-575 pre r1 r2 now > "$W/out/run-r1-575.log" 2>&1
  # 2) 这一轮 X1 自己造的样本集
  for fam in x1-W1 x1-W2 x1-W3 x1-W4 x1-W5 x1-V2-2 x1-V2-3 x1-V2-6 x1-V3-7; do nice -n 19 bash tools/runfam.sh "$fam" > "$W/out/run-$fam.log" 2>&1; done
  nice -n 19 python3 -I tools/myoracle.py cases/cases-x8.json res/x8.oracle.json 20 > /dev/null
  for t in r1 r2 now; do
    nice -n 19 python3 -I tools/feedhooks.py cases/cases-x8.json "$t" "res/x8.$t.json" 20 > /dev/null
    nice -n 19 python3 -I tools/feedhooks.py cases/cases-x9.json "$t" "res/x9.$t.json" 12 > /dev/null
    nice -n 19 python3 -I tools/feedhooks.py "cases/cases-x10-$t.json" "$t" "res/x10.$t.json" 12 > /dev/null
    nice -n 19 python3 -I tools/feedhooks.py cases/cases-kf.json "$t" "res/kf.$t.json" 8 > /dev/null
  done
  cp res/r1-575.oracle.json res/r1-575-bare.oracle.json
  for t in pre r1 r2 now; do nice -n 19 python3 -I tools/feedhooks.py cases/cases-r1-575-bare.json "$t" "res/r1-575-bare.$t.json" 20 > /dev/null; done
  fi
  # 3) 第二轮正推腿乙的 1943 案例
  nice -n 19 bash "$REPO_ROOT/research/prompts/shell-words-r2-sonnet-s2-model/tools/build-skeleton.sh" "$REPO_ROOT" "$FROZEN" "$W/s2work" > "$W/out/skeleton.log" 2>&1
  nice -n 19 python3 -I tools/s2feed.py res/s2.now.json 14 > "$W/out/s2feed.log" 2>&1
  # 4) 花括号序列、杂拌词与真 bash 逐词比；内建表等小事
  : > "$W/out/x3brace.txt"
  for t in r2 now; do for seed in 3 4; do nice -n 19 python3 -I tools/bracecmp.py "$t" "$seed" $((seed == 3 ? 1500 : 3000)) | head -n 3 >> "$W/out/x3brace.txt"; done; done
  { for t in r2 now; do echo "== $t"; nice -n 19 python3 -I tools/smallchk.py "$t"; done; } > "$W/out/x3small.txt"
  # 5) 14 份自证（主工作区仓根里跑，各放进自己的进程组，220 秒杀整组）
  SELFTEST_TIMEOUT=600 nice -n 19 python3 -I tools/selftests.py > "$W/out/x3selftests.raw" 2>&1 || true
  sed 's/ 末行：.*//' "$W/out/x3selftests.raw" > "$W/out/x3selftests.txt"
  # 6) 汇总
  python3 -I tools/x3fam.py > out/x3fam.txt; python3 -I tools/x3chg.py > out/x3chg.txt; python3 -I tools/x3pre.py > out/x3pre.txt; python3 -I tools/x3kf.py > out/x3kf.txt
  python3 -I tools/x3s2.py "$REPO_ROOT/research/prompts/shell-words-r2-sonnet-s2-model/out" > out/x3s2.txt; python3 -I tools/x1sum.py > out/x1.txt; python3 -I tools/x8910.py > out/x8910.txt
  for f in x3fam x3chg x3pre x3kf x3s2 x3brace x3small x3selftests x1 x8910; do
    python3 -I -c "import sys; p=sys.argv[1]; s=open(p).read().replace(sys.argv[2], '<work>'); open(p+'.n','w').write(s)" "$W/out/$f.txt" "$W" && mv "$W/out/$f.txt.n" "$W/out/$f.txt"
    echo "name=out_$f lines=$(wc -l < "$W/out/$f.txt") sha256=$(sha256sum "$W/out/$f.txt" | cut -c1-16)"
    if diff -q "$MODEL/out/$f.txt" "$W/out/$f.txt" > /dev/null 2>&1; then echo "name=${f}_vs_model identical"; else echo "name=${f}_vs_model DIFFERENT"; { diff "$MODEL/out/$f.txt" "$W/out/$f.txt" 2>&1 || true; } | head -n 12; fi
  done
  # 7) 自己起的进程一个不剩
  echo "name=leftover_processes $(ps -u "$(id -u)" -o args | grep -F "$W/" | grep -vc "grep -F" || true)"
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
