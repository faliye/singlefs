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
  # 冻结副本拷三份进草稿目录：原样（after）、照被判 diff 倒着撤到 W13 之前（before，另撤 harness 里两处引新错误成员的分支）、
  # 原样加打印预演结局与关掉读错那一拒的开关（observed）。原型放进三份的 harness 测试目录，各编一次 release（同一时刻一条 cargo），
  # 再跑：不带故障的读迹、点名计划与之后的重挂、两份都坏的计划扫描（两版）、整池读序窗口扫描（after 一版）、系统配置与 fsid 两段打印。
  local cap="$REPO_ROOT/scripts/hosts/capped.sh"
  local bindgen="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include"
  mkdir -p "$WORK/out"
  for version in after before observed; do
    mkdir -p "$WORK/$version"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/$version/"
    cp "$MODEL/attack_shared_reads_probe.rs" "$WORK/$version/crates/singlefs-harness/tests/attack_shared_reads_probe.rs"
  done
  awk '/^```diff$/{f=1;next} /^```$/{if(f){f=0}} f' "$REPO_ROOT/research/prompts/_m3-b44-code-r3-diff.md" > "$WORK/out/w13-to-w16.diff"
  (cd "$WORK/before" && patch -R -p1 < "$WORK/out/w13-to-w16.diff" && patch -p1 < "$MODEL/patch/before-w13-harness.patch") > "$WORK/out/patch-before.log"
  (cd "$WORK/observed" && patch -p1 < "$MODEL/patch/observed-mount.patch") > "$WORK/out/patch-observed.log"
  local version binary
  for version in after before observed; do
    (cd "$WORK/$version" && BINDGEN_EXTRA_CLANG_ARGS="$bindgen" CARGO_TARGET_DIR="$WORK/target-$version" bash "$cap" default \
      cargo test --offline --release -p singlefs-harness --test attack_shared_reads_probe --no-run) > "$WORK/out/build-$version.log" 2>&1
    binary="$(ls "$WORK/target-$version"/release/deps/attack_shared_reads_probe-* | grep -v '\.d$' | head -n 1)"
    echo "$binary" > "$WORK/out/binary-$version"
  done
  probe() {  # probe <版本> <测试名> <环境变量…>：跑一条，输出原样交回
    local which="$1" test="$2"; shift 2
    (cd "$WORK/$which" && env "$@" bash "$cap" default "$(cat "$WORK/out/binary-$which")" --nocapture --exact "$test") 2>&1
  }
  local image
  for version in after before; do
    for image in unchanged corrupted; do
      probe "$version" fault_free_trace PROBE_IMAGE=$image | grep '^name=' | sed "s/^/$version /" >> "$WORK/out/fault-free-trace.log"
      probe "$version" fault_free_trace PROBE_IMAGE=$image PROBE_PRINT_TRACE=1 | grep '^trace ' | awk '{$2=""; print}' > "$WORK/out/trace-$version-$image.txt"
    done
  done
  for version in after before; do
    for image in corrupted unchanged; do
      probe "$version" follow_up PROBE_IMAGE=$image PROBE_REMOUNTS=5 \
        PROBE_PLANS=w:303:2:garbage,w:303:2:fail,w:303:1:garbage,p:23101440:306:1:garbage,p:23101440:306:1:fail \
        | grep '^follow' | sed "s/^/$version /" >> "$WORK/out/follow-up.log"
    done
  done
  probe before follow_up PROBE_IMAGE=unchanged PROBE_REMOUNTS=5 PROBE_PLANS=p:23101440:962:1:garbage,p:23101440:962:2:garbage \
    | grep '^follow' | sed 's/^/before /' >> "$WORK/out/follow-up.log"
  for image in unchanged corrupted; do
    probe observed follow_up PROBE_IMAGE=$image PROBE_REMOUNTS=1 PROBE_WITHOUT_THE_READ_ERROR_REFUSAL=1 \
      PROBE_PLANS=w:352:2:fail,w:353:2:fail,w:354:2:fail,w:355:2:fail,w:356:2:fail,w:357:2:fail,w:358:2:fail \
      | grep -E '^follow|PROBE_REHEARSAL' >> "$WORK/out/counterfactual.log"
    probe observed follow_up PROBE_IMAGE=$image PROBE_REMOUNTS=0 PROBE_PLANS=w:303:2:garbage,p:23101440:306:1:garbage,p:23101440:306:1:fail \
      | grep -E '^follow|PROBE_REHEARSAL' >> "$WORK/out/rehearsal-endings.log"
  done
  for version in after before; do
    probe "$version" system_configurations_after_the_plans PROBE_IMAGE=unchanged PROBE_REMOUNTS=1 PROBE_PLANS=w:303:1:garbage,w:303:2:garbage \
      | grep '^sysconf' | sed "s/^/$version /" >> "$WORK/out/sysconf.log"
  done
  for version in after before; do
    probe "$version" system_configuration_writes_in_order PROBE_IMAGE=unchanged PROBE_PLANS=w:303:1:garbage,w:303:2:garbage \
      | grep '^sysconf-writes' | sed "s/^/$version /" >> "$WORK/out/sysconf-writes.log"
  done
  probe after unit_header_filesystem_identifiers_of_fixture_pools | grep '^fsid' > "$WORK/out/fsid-fixtures.log"
  local kind
  for version in after before; do
    for image in unchanged corrupted; do
      for kind in fail garbage; do
        probe "$version" scan_both_copies_after_the_anchor PROBE_IMAGE=$image PROBE_KIND=$kind PROBE_AFTER=306 PROBE_FIRST=1 \
          PROBE_OFFSETS_FROM="$WORK/out/trace-after-$image.txt" | grep 'summary' | sed "s/^/$version /" >> "$WORK/out/pair-summary.log"
      done
    done
  done
  local width
  for image in corrupted unchanged; do
    for kind in fail garbage; do
      for width in 1 2 3; do
        probe after scan_windows PROBE_IMAGE=$image PROBE_KIND=$kind PROBE_WIDTHS=$width PROBE_VERBOSE=0 | grep 'summary' >> "$WORK/out/window-summary.log"
      done
    done
  done
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  cat "$WORK/out/fault-free-trace.log"
  grep -E 'first mount|remount' "$WORK/out/follow-up.log" | cut -c1-160
  grep -E 'refusal-skipped|first mount' "$WORK/out/counterfactual.log" | cut -c1-120
  cat "$WORK/out/pair-summary.log" "$WORK/out/window-summary.log"
  echo "比：与 $MODEL/out/ 里同名文件逐行比（fault-free-trace.log、follow-up.log、pair-summary.log、window-summary.log、sysconf.log、sysconf-writes.log、fsid-fixtures.log）"
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
