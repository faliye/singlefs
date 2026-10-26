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

# ── 这一腿要跑的步骤（m4-exit5-relocation-proof-r2 攻方）──
# 一、直接起 Verus（--rlimit 10，登记的数；带不带 --no-cheating 逐份写明）：作弊标记三份、Y1/Y2 原型 crate、Y5 攻击模块与三份必红变体。
# 二、Y1/Y2 原型 crate 照产品构建 cargo build（不带 verus_keep_ghost），直接执行编出来的二进制，逐个核三个函数的 ensures。
# 三、冻结副本的拷贝上用 scripts/gate/lib_function_units.py 收四条闭包、数 Y3/Y4 那几样；判器闭包键在四种改法下变不变。
# 末了与 $MODEL/out/summary.txt 逐行比，不同退 1。
rerun_steps() {
  local V="${VERUS_INSTALL:-$HOME/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux}"
  local summary="$WORK/summary.txt"
  : > "$summary"
  mkdir -p "$WORK/out" "$WORK/mut"
  verus_case() {   # verus_case <名> <目录> <入口> [参数…]：一行判定写进 summary
    local name=$1 dir=$2 entry=$3 out rc=0; shift 3
    out=$(cd "$dir" && "$V/verus" --crate-type=lib --rlimit 10 "$@" "$entry" 2>&1) || rc=$?
    printf '%s\n' "$out" > "$WORK/out/verus-$name.txt"
    printf 'name=verus case=%s rc=%s %s\n' "$name" "$rc" \
      "$(printf '%s\n' "$out" | grep -E 'verification results|^error' | grep -v '^error: aborting' | head -2 | tr '\n' ' ')" | tee -a "$summary"
  }
  replace_once() {  # replace_once <文件> <原文> <替换文>
    python3 - "$1" "$2" "$3" <<'PY'
import sys
p, old, new = sys.argv[1:4]; t = open(p).read()
assert t.count(old) == 1, old
open(p, "w").write(t.replace(old, new))
PY
  }
  # 一、作弊标记
  mkdir -p "$WORK/probe"; cp "$MODEL"/verus/axiom_plain.rs "$MODEL"/verus/axiom_broadcast.rs "$MODEL"/verus/assume_spec.rs "$WORK/probe/"
  verus_case axiom-plain-no-cheating "$WORK/probe" axiom_plain.rs --no-cheating
  verus_case axiom-plain-without-no-cheating "$WORK/probe" axiom_plain.rs
  verus_case axiom-broadcast-no-cheating "$WORK/probe" axiom_broadcast.rs --no-cheating
  verus_case assume-specification-no-cheating "$WORK/probe" assume_spec.rs --no-cheating
  verus_case assume-specification-without-no-cheating "$WORK/probe" assume_spec.rs
  # Y1/Y2 原型 crate
  cp -r "$MODEL/y1crate" "$WORK/y1crate"; cp "$FROZEN/Cargo.lock" "$WORK/y1crate/"
  verus_case divergence-only-no-cheating "$WORK/y1crate/src" divergence_root.rs --no-cheating
  verus_case both-modules-without-no-cheating "$WORK/y1crate/src" lib.rs
  cp -r "$WORK/y1crate" "$WORK/mut/y1-red-items"
  replace_once "$WORK/mut/y1-red-items/src/divergence.rs" $'{\n    start + span\n}\n\n#[cfg(not(verus_keep_ghost))]' $'{\n    start + 1\n}\n\n#[cfg(not(verus_keep_ghost))]'
  verus_case divergence-must-red-verified-body "$WORK/mut/y1-red-items/src" divergence_root.rs --no-cheating
  cp -r "$WORK/y1crate" "$WORK/mut/y1-red-macro"
  replace_once "$WORK/mut/y1-red-macro/src/divergence.rs" "if cfg!(verus_keep_ghost) { start + span } else { start + 1 }" "if cfg!(verus_keep_ghost) { start + 1 } else { start + 1 }"
  verus_case divergence-must-red-cfg-macro "$WORK/mut/y1-red-macro/src" divergence_root.rs --no-cheating
  cp -r "$WORK/y1crate" "$WORK/mut/y1-red-first-index"
  replace_once "$WORK/mut/y1-red-first-index/src/std_spec_probe.rs" $'        Ok(i) => Some(i),\n' $'        Ok(i) => if i > 0 { Some(i - 1) } else { Some(i) },\n'
  verus_case std-spec-must-red-first-index "$WORK/mut/y1-red-first-index/src" lib.rs
  # Y5
  mkdir -p "$WORK/y5/src"
  cp "$FROZEN/crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs" "$FROZEN/crates/singlefs-replay-prefix/src/gc_crash_proofs.rs" \
    "$MODEL/verus/attack_patched_p4_and_rollback_views.rs" "$WORK/y5/src/"
  printf '%s\n' "pub mod compaction_plan_proofs;" "pub mod gc_crash_proofs;" > "$WORK/y5/src/base.rs"
  printf '%s\n' "pub mod compaction_plan_proofs;" "pub mod gc_crash_proofs;" "pub mod attack_patched_p4_and_rollback_views;" > "$WORK/y5/src/lib.rs"
  verus_case y5-baseline-two-frozen-proofs "$WORK/y5/src" base.rs --no-cheating
  verus_case y5-attack "$WORK/y5/src" lib.rs --no-cheating
  local variant old new
  for variant in rollback-read-unchanged protocol-allows-the-write two-slot-write-refused; do
    case $variant in
      rollback-read-unchanged) old="        != bytes_read_through_the_mapping(rollback_target_mapping(), storage_before(), 4u64)) by {"; new="${old/!=/==}" ;;
      protocol-allows-the-write) old="    assert(!patched_slot_is_unreferenced(s, seq![rollback_target_mapping()], 20u64)) by {"; new="${old/assert(!/assert(}" ;;
      two-slot-write-refused) old="    assert(patched_slot_is_unreferenced(s, views, 21u64)) by {"; new="${old/assert(/assert(!}" ;;
    esac
    cp -r "$WORK/y5" "$WORK/mut/y5-$variant"
    replace_once "$WORK/mut/y5-$variant/src/attack_patched_p4_and_rollback_views.rs" "$old" "$new"
    verus_case "y5-must-red-$variant" "$WORK/mut/y5-$variant/src" lib.rs --no-cheating
  done
  # 二、产品构建
  local rc=0
  "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo build --offline --manifest-path "$WORK/y1crate/Cargo.toml" --target-dir "$WORK/y1-target" > "$WORK/out/y1-build.txt" 2>&1 || rc=$?
  echo "name=product-build rc=$rc" | tee -a "$summary"
  "$WORK/y1-target/debug/divergence_probe" | tee -a "$summary"
  rm -rf "$WORK/y1-target"
  # 三、闭包与键
  mkdir -p "$WORK/tree"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  local entry name
  for entry in singlefs_core::transaction::publish_relocation_batch singlefs_core::mount::roll_back_by_a_forward_publish \
      singlefs_checker_tier::crash::judge_crash_image singlefs_core::recovery::recover; do
    name=${entry##*::}; [[ $name == publish_relocation_batch ]] && name=publish
    python3 "$REPO_ROOT/scripts/gate/lib_function_units.py" closure "$WORK/tree" "$entry" > "$WORK/closure-$name.txt" 2> "$WORK/closure-$name.err"
  done
  python3 "$MODEL/tools/closure-analysis.py" "$WORK" | tee -a "$summary"
  python3 "$MODEL/tools/cfg-in-closure.py" "$WORK" "$WORK/tree" | tee -a "$summary"
  bash "$MODEL/tools/key-variants.sh" "$WORK" "$REPO_ROOT" "$FROZEN" | tee -a "$summary"
  if cargo tree -p singlefs-core -e normal --offline --locked --manifest-path "$WORK/tree/Cargo.toml" 2>&1 | grep -q '└── vstd v0.0.0-2026-09-20-0158'; then
    echo "name=cargo-tree core_already_depends_on_vstd_through_singlefs_replay_prefix=yes" | tee -a "$summary"
  else
    echo "name=cargo-tree core_already_depends_on_vstd_through_singlefs_replay_prefix=no" | tee -a "$summary"
  fi
  python3 - "$REPO_ROOT" <<'PY' | tee -a "$summary"
import re, sys
line = [l for l in open(sys.argv[1] + "/scripts/gate/verus-proof-check.py") if l.startswith("CHEATING_MARKER = re.compile(")][0]
marker = re.compile(eval(line.split("re.compile(", 1)[1].rsplit(")", 1)[0]))
for text in ("pub axiom fn a_lemma_that_proves_false()", "pub broadcast axiom fn axiom_every_slot_is_free(slot: u64)", "#[cfg(verus_keep_ghost)]", "if cfg!(verus_keep_ghost) { start + span } else { start + 1 }", "pub assume_specification<T: Ord>[ <[T]>::binary_search ]"):
    print(f"name=cheating-regex text={text.split('(')[0].strip()[:40].replace(' ', '_')} matches={int(bool(marker.search(text)))}")
PY
  rm -rf "$WORK/tree" "$WORK/mut" "$WORK/y5" "$WORK/y1crate"
  if diff "$MODEL/out/summary.txt" "$summary" > "$WORK/summary.diff"; then
    echo "name=rerun-compare out/summary.txt same=yes"
  else
    echo "name=rerun-compare out/summary.txt same=no"; cat "$WORK/summary.diff"; return 1
  fi
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
