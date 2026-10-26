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
  mkdir -p "$WORK/out"
  # 一、小模型（纯 python，不起 cargo）：扫描重建几种认领规则在部分写之后认不认得回活单元
  nice -n 19 python3 -I "$MODEL/scan_rule_model.py" 10 > "$WORK/out/scan-rule-model-depth10.txt" 2>&1
  echo "name=scan_rule_model rc=$? out=$WORK/out/scan-rule-model-depth10.txt"
  nice -n 19 python3 -I "$MODEL/ext_and_hole_write_check.py" 10 > "$WORK/out/ext-and-hole-write-check.txt" 2>&1
  echo "name=ext_and_hole_write_check rc=$? out=$WORK/out/ext-and-hole-write-check.txt"
  # 二、两个 Rust 探针（harness 档：singlefs-harness 的一个新测试目标、singlefs-checker 的 lib 测试一条），在冻结副本的拷贝上跑，不改冻结副本
  mkdir -p "$WORK/tree"
  rsync -a --exclude target --exclude .git --exclude research --exclude scripts "$FROZEN/" "$WORK/tree/"
  cp "$MODEL/partial_write_cost_probe.rs" "$WORK/tree/crates/singlefs-harness/tests/partial_write_cost_probe.rs"
  python3 -I - "$WORK/tree/crates/singlefs-checker/src/rebuild.rs" "$MODEL/rebuild_mirror_probe.rs.txt" <<'PY'
import sys
path, snippet = sys.argv[1], open(sys.argv[2]).read()
source = open(path).read()
anchor = "    /// 题面那一形（判决 m4-trunc-r2 第三节 R3）：两个单元同在 txg 3 写下"
assert source.count(anchor) == 1
open(path, "w").write(source.replace(anchor, snippet + anchor))
PY
  cd "$WORK/tree"
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 partial_write_cost_probe; 范围=1 个测试目标; 为什么=量重写一个数据单元与整份重写 n 个单元各发多少角色' \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 8G \
    cargo test --offline -p singlefs-harness --test partial_write_cost_probe -- --nocapture print_the_cost_of_rewriting_one_data_unit > "$WORK/out/cost-probe-run.txt" 2>&1
  echo "name=cost_probe rc=$? out=$WORK/out/cost-probe-run.txt"
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 partial_write_cost_probe 的第二条; 范围=1 个测试目标; 为什么=量截断意图与 unlink 意图各发多少角色，给区间墓碑每条记录的单价一个上界' \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 8G \
    cargo test --offline -p singlefs-harness --test partial_write_cost_probe -- --nocapture truncation_intent_against > "$WORK/out/tombstone-proxy-run.txt" 2>&1
  echo "name=tombstone_proxy rc=$? out=$WORK/out/tombstone-proxy-run.txt"
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker 的 lib 测试 probe_a_partial_write_in_the_middle_unit; 范围=1 个测试; 为什么=部分写之后只走扫描重建的镜像' \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 8G \
    cargo test --offline -p singlefs-checker --lib -- probe_a_partial_write_in_the_middle_unit --nocapture > "$WORK/out/rebuild-mirror-run.txt" 2>&1
  echo "name=rebuild_mirror rc=$? out=$WORK/out/rebuild-mirror-run.txt"
  # 与 $MODEL/out/ 里的逐行比（只比 name= 开头的行）
  for file in scan-rule-model-depth10.txt ext-and-hole-write-check.txt cost-probe-run.txt tombstone-proxy-run.txt rebuild-mirror-run.txt; do
    if diff <(grep -E '^(name|variant|selfcheck|  example)' "$MODEL/out/$file") <(grep -E '^(name|variant|selfcheck|  example)' "$WORK/out/$file") > /dev/null; then
      echo "name=compare file=$file same=yes"
    else
      echo "name=compare file=$file same=NO"
    fi
  done
  rm -rf "$WORK/tree"
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
