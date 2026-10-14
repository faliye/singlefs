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
# m3-c554-c572-r1 攻方（Opus）复跑：冻结副本拷进 $WORK/tree，打 patches/ 里两份补丁（core 里 B2/B4/B5 的臂、工作区加探针包），
# mount.rs 末尾接 src/b3_unit_test.rs，探针包拷进 crates/attack-c554-probe；编、跑，输出写 $WORK/out，最后打 RERUN-CHECK 行与 $MODEL/out 里的数比。
# 线程 RERUN_THREADS（默认 16）；RERUN_LONG=1 时另跑两段各 270378 态的枚举（各约 20 分钟）与 k 扫描那几段。判定库落在 $WORK/crashlib。
rerun_steps() {
  local T="$WORK/tree" OUT="$WORK/out" THREADS="${RERUN_THREADS:-16}"
  local CAPPED="$REPO_ROOT/scripts/hosts/capped.sh"
  mkdir -p "$T" "$OUT"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$T/"
  mkdir -p "$T/.cargo" && cp "$REPO_ROOT/.cargo/config.toml" "$T/.cargo/config.toml"
  ( cd "$T" && patch -p1 -s < "$MODEL/patches/core-arms.diff" && patch -p1 -s < "$MODEL/patches/workspace-member.diff" )
  cat "$MODEL/src/b3_unit_test.rs" >> "$T/crates/singlefs-core/src/mount.rs"
  mkdir -p "$T/crates/attack-c554-probe" && cp -r "$MODEL/src/probe/." "$T/crates/attack-c554-probe/"
  ( cd "$T" && nice -n 19 bash "$CAPPED" "$THREADS" cargo test --offline -p singlefs-core --lib attack_b3 -- --nocapture ) > "$OUT/b3-test.full.log" 2>&1
  grep -E 'OPUS-B3|^test |^test result|^running' "$OUT/b3-test.full.log" > "$OUT/b3-test.log" || true
  ( cd "$T" && nice -n 19 bash "$CAPPED" "$THREADS" cargo build --offline --release -p attack-c554-probe ) > "$OUT/build.log" 2>&1
  local BIN="$T/target/release/attack-c554-probe"
  export CARGO; CARGO="$(command -v cargo)"
  export SINGLEFS_CRASH_AMPLIFICATION_HOME="$WORK/crashlib" SINGLEFS_LAYER0_THREADS="$THREADS"
  run() { rm -rf "$WORK/crashlib"; mkdir -p "$WORK/crashlib"; ( cd "$T" && env "$@" nice -n 19 bash "$CAPPED" "$THREADS" "$BIN" ); }
  for x in 0 1; do for arm in "" verify_quarantined refuse_quarantined verify_all_shared; do
    run MODE=b5 EXTRA_OVERWRITES=$x OPUS_C572_ARM=$arm 2>&1 | grep -E 'OPUS-B5|OPUS-C572|panicked'
  done; done > "$OUT/b5-full.log"
  for arm in "" skip_once; do
    run MODE=b4 OPUS_B4_ARM=$arm K_LIST=2,3,4,5,10 HIDE_LIST=rec,rec+unit 2>&1 | grep -E 'OPUS-B4 k=|panicked'
  done > "$OUT/b4.log"
  run MODE=b2 K_LIST=2,3,4,5 J=3 MOUNT3=1 2>&1 | grep -E '^OPUS-B2 k=' > "$OUT/b2.log"
  for w in 240 384; do for j in 8 14 20; do
    run MODE=b2 UNIT_SLOTS=$w K_LIST=2,3 HIDE_LIST=acct J=$j MOUNT3=0 2>&1 | grep -E 'OPUS-B2 k=|panicked' | sed "s/^/W=$w /"
  done; done > "$OUT/b2-pressure.log"
  run MODE=b1scan HIDE_LIST=rec,rec+unit,unit 2>&1 | grep -E '^OPUS-B1SCAN' > "$OUT/b1scan.log"
  run MODE=b1 OUTER=0 UNIT_SLOTS=240 K_LIST=10 HIDE_LIST=rec 2>&1 | grep -E '^OPUS-' > "$OUT/b1-W240-k10-rec-today.log"
  run OPUS_B4_ARM=skip_once MODE=b1 OUTER=0 UNIT_SLOTS=240 K_LIST=10 HIDE_LIST=rec 2>&1 | grep -E '^OPUS-' > "$OUT/b1-W240-k10-rec-skiponce.log"
  if [[ "${RERUN_LONG:-0}" == 1 ]]; then
    run MODE=b1 OUTER=0 UNIT_SLOTS=240 K_LIST=10 HIDE_LIST=nofault 2>&1 | grep -E '^OPUS-' > "$OUT/b1-W240-k10-nofault.log"
    run MODE=b1 OUTER=0 UNIT_SLOTS=240 K_LIST=10 HIDE_LIST=unit 2>&1 | grep -E '^OPUS-' > "$OUT/b1-W240-k10-unit.log"
    run MODE=b1 OUTER=0 K_LIST=2 HIDE_LIST=unit,rec,rec+unit,acct,new 2>&1 | grep -E '^OPUS-' > "$OUT/b1-k2.log"
    run MODE=b1 OUTER=0 K_LIST=3,4,5,6,8,11 HIDE_LIST=rec 2>&1 | grep -E '^OPUS-' > "$OUT/b1-rec-k.log"
  fi
  check() { # check <名> <文件> <要找的串>
    if grep -qF -- "$3" "$2"; then echo "RERUN-CHECK name=$1 ok=true"; else echo "RERUN-CHECK name=$1 ok=false want=$3"; fi
  }
  check b3_form1 "$OUT/b3-test.log" "OPUS-B3-FORM1 narrow_first=(false, true) wide_first=(false, false)"
  check b3_form2 "$OUT/b3-test.log" "OPUS-B3-FORM2 got=(false, true)"
  check b3_form3 "$OUT/b3-test.log" "OPUS-B3-FORM3 got=(false, true)"
  while IFS= read -r line; do check b5_line "$OUT/b5-full.log" "$line"; done < "$MODEL/out/b5-full.log"
  while IFS= read -r line; do check b4_line "$OUT/b4.log" "$line"; done < "$MODEL/out/b4.log"
  while IFS= read -r line; do check b2_line "$OUT/b2.log" "$line"; done < "$MODEL/out/b2.log"
  while IFS= read -r line; do check b2p_line "$OUT/b2-pressure.log" "$line"; done < "$MODEL/out/b2-pressure.log"
  check b1_today_hits "$OUT/b1-W240-k10-rec-today.log" "lost0_ro_other_failed=12296"
  check b1_today_newbad "$OUT/b1-W240-k10-rec-today.log" "newbad_ro_abandoned_read_ok=9224 newbad_ro_other_read_ok=11292"
  check b1_skip_hits "$OUT/b1-W240-k10-rec-skiponce.log" "lost0_ro_other_failed=20510"
  check b1_skip_newbad "$OUT/b1-W240-k10-rec-skiponce.log" "newbad_ro_other_failed=8219"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
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
