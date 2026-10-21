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
  mkdir -p "$WORK/tree" "$WORK/out"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  [[ -f "$MODEL/patches/make_filesystem-order.patch" ]] || { echo "name=rerun_skipped reason=model_directory_without_patches"; return 0; }
  # 冻结副本 = 仓里 crates/、Cargo.toml、Cargo.lock、.cargo、.claude/heavy-thread-default 的拷贝；开工时 crates/ 的整树哈希写在报告开头
  echo "name=frozen_crates_hash $(cd "$WORK/tree" && find crates -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum)"
  # 两份补丁来自 m3-close-c656-r1 攻方腿（写序 today / invalidate / sysconf-first / new-then-sysconf），原样拷在本模型目录的 patches/ 下
  patch "$WORK/tree/crates/singlefs-core/src/make_filesystem.rs" "$MODEL/patches/make_filesystem-order.patch"
  patch "$WORK/tree/crates/singlefs-core/src/make_filesystem.rs" "$MODEL/patches/make_filesystem-order-benign.patch"   # 辩方腿加的写序变体 LatestRootLast，在第一份之上
  patch "$WORK/tree/Cargo.toml" "$MODEL/patches/workspace.patch"
  mkdir -p "$WORK/tree/crates/c656-attack-probe/src"
  cp "$MODEL/probe/Cargo.toml" "$WORK/tree/crates/c656-attack-probe/Cargo.toml"
  cp "$MODEL/probe/main.rs" "$WORK/tree/crates/c656-attack-probe/src/main.rs"
  # 只为省编译：RERUN_SEED_TARGET 指到一份别处编好的 target 就拷进来（第三方依赖不重编）；不给就从头编（librocksdb-sys 慢）
  if [[ -n "${RERUN_SEED_TARGET:-}" && -d "${RERUN_SEED_TARGET}" ]]; then cp -a "$RERUN_SEED_TARGET" "$WORK/tree/target"; fi
  local binary="$WORK/tree/target/release/c656-attack-probe" cargo_binary
  if [[ -n "${RERUN_PREBUILT_BINARY:-}" ]]; then
    # 只为省内存与编译时间：指到一份用同样的补丁与探针编好的二进制就不再编（本腿在内存额度紧时用，编译在包装外面单独做）
    binary="$RERUN_PREBUILT_BINARY"
  else
    (cd "$WORK/tree" && nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo build --release -j 4 -p c656-attack-probe)
  fi
  cargo_binary="$(rustup which cargo)"
  run_one() {   # run_one <输出名> <参数…>：在草稿目录里起（草稿目录里没有 multi-host.env，判段走 CPU），库放草稿目录
    local name="$1"; shift
    (cd "$WORK" && CARGO="$cargo_binary" SINGLEFS_CRASH_AMPLIFICATION_LIBRARY="$WORK/lib-$name" \
      nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 "$@") > "$WORK/out/$name.log" 2>&1
  }
  # 阶段：RERUN_STAGES 默认 "b a"。b = 辩方腿的 latest-root-last 写序（fsid 相同也不丢确认过的写的候选）；
  # a = 旧 fsid 与新 fsid 只差后 8 字节（fdiff）与前 8 字节也不同（fpdiff）两种历史上，乙与 new-then-sysconf 做完的池级 checker（看 name=checker 行的 I-7.7）；
  # c = 先写系统配置的写序 × 四份历史（每份 124 499 态，机器忙时每份要二十分钟以上，默认不跑）
  local stages=" ${RERUN_STAGES:-b a} "
  if [[ "$stages" == *" b "* ]]; then
    for history in o512-u5-fsame o512-u6-fsame o512-u23-fsame o65536-u5-fsame o512-u5-fdiff; do
      run_one "run-$history-latest-root-last" env PROBE_WRITABLE=1 "$binary" run "$history" latest-root-last
    done
  fi
  if [[ "$stages" == *" a "* ]]; then
    # 历史名 o<旧 io_min>-u<u>-f<same|diff|pdiff>：pdiff = 旧 fsid 与新 fsid 的前 8 字节也不同（r1 两个原型的 diff 只差第 9 字节起）
    # 乙与「先写单元与根、最后动系统配置」× 前 8 字节不同：看做完那一态池级 checker 红不红（C659 的 I-7.7）
    for history in o512-u5-fdiff o512-u5-fpdiff; do
      run_one "run-$history-invalidate" "$binary" run "$history" invalidate
      run_one "run-$history-new-then-sysconf" "$binary" run "$history" new-then-sysconf
    done
    # 判读：out/run-*.log 里 name=done、name=count、name=checker 行与模型目录 out/ 里同名文件逐行比（name=example 行的序号随线程次序变，不比）
  fi
  if [[ "$stages" == *" c "* ]]; then
    # 先写系统配置的写序 × 历史（默认四份：fsid 相同 / 只差后 8 字节 / 前 8 字节也不同 / 旧池槽距 65536；RERUN_C_HISTORIES 可改）
    for history in ${RERUN_C_HISTORIES:-o512-u5-fdiff o512-u5-fsame o512-u5-fpdiff o65536-u5-fdiff}; do
      run_one "run-$history-sysconf-first" "$binary" run "$history" sysconf-first
    done
  fi
  echo "name=rerun_done out=$WORK/out"
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
