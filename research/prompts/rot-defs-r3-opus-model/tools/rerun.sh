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

# ── 这一腿要跑的步骤（rot-defs-r3 攻方腿）──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引。钩子探针不执行命令本身；主工作区合入锁一律指到 $WORK 里不存在的文件
# （SINGLEFS_WORKSPACE_LOCK_FILE），结果不随别的会话占不占锁变。钩子量三版：今天的（仓根）、开工快照那一版、R2 R3 换上之前那一版
# （后两版由 inputs/rebuild-hooks.py 从今天的倒推，放进 $WORK 里仓的拷贝，target 与 .git 链回仓根、只读）。
# cargo 只在 $WORK 里的一个小包上起一次（经 capped.sh 与内存包装）。输出里的 $REPO_ROOT、$WORK 换成 @REPO@、@WORK@ 再与 $MODEL/out/ 比。
make_tree() {   # make_tree <树名> <钩子目录>：仓的拷贝换上那一版钩子
  local tree="$WORK/$1"
  mkdir -p "$tree"
  rsync -a --exclude target --exclude .git --exclude research "$REPO_ROOT/" "$tree/"
  local name
  for name in lib_heavy_tests.py heavy-test-guard.sh; do
    cp "$2/$name" "$tree/.claude/hooks/$name.new" && mv "$tree/.claude/hooks/$name.new" "$tree/.claude/hooks/$name"
  done
  ln -s "$REPO_ROOT/target" "$tree/target"
  ln -s "$REPO_ROOT/.git" "$tree/.git"
}
probe_cases() {   # probe_cases <仓根>：cases.list 逐条喂那棵树的三道钩子
  local name agent
  while IFS='|' read -r name agent; do
    [[ -n "$name" ]] || continue
    echo "== $name（$agent）"
    (cd "$1" && PROBE_LINES=1 python3 "$MODEL/probe.py" "$1" "$WORK" "$agent" "$1" "$MODEL/cases/$name.txt")
  done < "$MODEL/cases.list"
}
rerun_steps() {
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  (cd "$REPO_ROOT" && sha256sum -c "$FROZEN/defs-sha256.txt" 2>&1 | grep -v ': OK$' || true) > "$WORK/snapshot-drift.out"
  export SINGLEFS_WORKSPACE_LOCK_FILE="$WORK/no-such-lock"
  local normalize="s#$WORK/snapshot-tree#@TREE@#g; s#$WORK#@WORK@#g; s#$REPO_ROOT#@REPO@#g"
  python3 "$MODEL/inputs/rebuild-hooks.py" "$REPO_ROOT" "$MODEL/inputs" "$FROZEN/defs-sha256.txt" "$WORK/hooks" > "$WORK/hooks.out"
  make_tree snapshot-tree "$WORK/hooks/snapshot"
  make_tree pre-swap-tree "$WORK/hooks/pre-swap"
  printf '%s\n' '#!/usr/bin/env bash' 'export GATE_MUTATION_BREAK=quickrunsallrows' \
    "SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=门禁 checker-tier-crates-mutation-replay.sh 的轻型（--quick）; 范围=轻型核心部分; 为什么=改了 singlefs-core 的分配器要核轻型登记的变异行' bash scripts/hosts/run-with-memory-cap.sh 8G bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick $REPO_ROOT" \
    > "$WORK/light-mutation.sh"
  probe_cases "$REPO_ROOT" | sed "$normalize" > "$WORK/probe-today.out"
  probe_cases "$WORK/snapshot-tree" | sed "$normalize" > "$WORK/probe-snapshot.out"
  # 工具员报告第三节那条 41 条喂法，喂开工快照那一版（只喂不换）
  (cd "$WORK/snapshot-tree" && nice -n 19 python3 scripts/gate/hook-swap.py --feed-only --hook .claude/hooks/heavy-test-guard.sh --root "$WORK/snapshot-tree" \
     --feeds "$MODEL/inputs/hook-swap-feeds.json" 2>&1 | tail -n 1 | sed "$normalize") > "$WORK/feeds-snapshot.out"
  # cargo test --release --bin <名> 只编测试版：一个只有一个 bin 的小包，跑完看 target/release 下有没有本体
  mkdir -p "$WORK/tiny/src"
  printf '[package]\nname = "tiny"\nversion = "0.1.0"\nedition = "2021"\n\n[workspace]\n' > "$WORK/tiny/Cargo.toml"
  printf 'fn main() { println!("main"); }\n#[cfg(test)]\nmod tests { #[test] fn one() { assert_eq!(1, 1); } }\n' > "$WORK/tiny/src/main.rs"
  local tiny_exit=0
  (cd "$WORK/tiny" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=tiny 的测试目标 tiny; 范围=1 个测试目标; 为什么=核 cargo test --release --bin 只编测试版、不编出 target/release 下的本体' bash "$REPO_ROOT/scripts/hosts/capped.sh" 4 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 2G cargo test --offline --release -p tiny --bin tiny) > "$WORK/tiny-test.log" 2>&1 || tiny_exit=$?
  echo "name=tiny_cargo_test exit=$tiny_exit release_main=$([[ -f "$WORK/tiny/target/release/tiny" ]] && echo present || echo absent) deps_test_binaries=$(find "$WORK/tiny/target/release/deps" -maxdepth 1 -type f -name 'tiny-*' ! -name '*.d' | wc -l)" > "$WORK/tiny.out"
  # 第二轮判决第一节的引文数（被引文件在那份判决写成之后改过的列进没判；1792341124 是那份判决文件的修改时刻）
  (cd "$REPO_ROOT" && nice -n 19 python3 scripts/agents/cite-check.py research/prompts/rot-defs-r2-sonnet-output.md research/prompts/rot-defs-r2-opus-output.md \
     --root . --background research/prompts/_rot-defs-r2-background.md --unchanged-since 1792341124 2>&1 | tail -n 1) > "$WORK/cite-check.out" || true
  # 第二轮攻方模型复跑：今天的钩子一趟，R2 R3 换上之前那一版一趟
  cp -r "$REPO_ROOT/research/prompts/rot-defs-r2-opus-model" "$WORK/r2-model"
  local frozen_r2="$REPO_ROOT/research/prompts/rot-defs-r2-snapshot"
  bash "$WORK/r2-model/tools/rerun.sh" --repo "$REPO_ROOT" --frozen "$frozen_r2" --work "$WORK/r2-rerun-today" 2>&1 | grep '^name=compare' > "$WORK/r2-rerun-today.out" || true
  bash "$WORK/r2-model/tools/rerun.sh" --repo "$WORK/pre-swap-tree" --frozen "$frozen_r2" --work "$WORK/r2-rerun-pre-swap" 2>&1 | grep '^name=compare' > "$WORK/r2-rerun-pre-swap.out" || true
  diff "$WORK/r2-model/out/capped.out" "$WORK/r2-rerun-pre-swap/capped.out" | sed "$normalize" > "$WORK/r2-capped-diff.out" || true
  # 第二轮判决第一节那一行引文数，照它当时的文件复原一棵根再核：被判的几份从 git 里还找得到的那一版取（HEAD、暂存区、悬空的 blob），
  # 两份钩子用上面倒推的 R2 R3 换上之前那一版；悬空 blob 被 git gc 收走之后这一步报取不到、照实记
  local r2root="$WORK/r2-root" spec source path
  mkdir -p "$r2root/research/prompts"
  rsync -a --exclude target --exclude .git --exclude research "$REPO_ROOT/" "$r2root/"
  cp "$REPO_ROOT"/research/prompts/rot-defs-r2-sonnet-output.md "$REPO_ROOT"/research/prompts/rot-defs-r2-opus-output.md \
     "$REPO_ROOT"/research/prompts/_rot-defs-r2-background.md "$REPO_ROOT"/research/prompts/rot-defs-r1-main-verification.md "$r2root/research/prompts/"
  : > "$WORK/r2-root-sources.out"
  for spec in .claude/agent-common.md=baa6e2e177a148ba27306709333ea35e368e7a8f .claude/agents/experiment-runner.md=4d1e024aa111a3f2736d6a6eb36c76a16c265d7a \
      .claude/agents/implementation-writer.md=cb29ba363331101e74a9101a40ce3b705c948cef .claude/agents/three-way-attack.md=3451d581f74dca2381b357dcd741ceba651fd34e \
      .claude/main-agent.md=8785f2ff5f501933ce68818bd22b43069264a0f1 scripts/crash/crash-point-replay.sh=abc5f8e08aad058cb9ce81505fa53d913ac70ffb \
      .claude/agents/three-way-defense.md=HEAD .claude/agents/three-way-forward.md=HEAD .claude/rules/path-moves.md=HEAD .claude/gate.d/doc-decisions.sh=HEAD; do
    path="${spec%%=*}" source="${spec#*=}"
    if [[ "$source" == HEAD ]]; then
      git -C "$REPO_ROOT" show "HEAD:$path" > "$r2root/$path.new" 2>/dev/null && mv "$r2root/$path.new" "$r2root/$path" && echo "name=r2_root_source file=$path from=HEAD" >> "$WORK/r2-root-sources.out"
    else
      git -C "$REPO_ROOT" cat-file -p "$source" > "$r2root/$path.new" 2>/dev/null && mv "$r2root/$path.new" "$r2root/$path" && echo "name=r2_root_source file=$path from=blob" >> "$WORK/r2-root-sources.out" \
        || echo "name=r2_root_source file=$path from=取不到" >> "$WORK/r2-root-sources.out"
    fi
  done
  cp "$WORK/hooks/pre-swap/lib_heavy_tests.py" "$r2root/.claude/hooks/lib_heavy_tests.py.new" && mv "$r2root/.claude/hooks/lib_heavy_tests.py.new" "$r2root/.claude/hooks/lib_heavy_tests.py"
  cp "$WORK/hooks/pre-swap/heavy-test-guard.sh" "$r2root/.claude/hooks/heavy-test-guard.sh.new" && mv "$r2root/.claude/hooks/heavy-test-guard.sh.new" "$r2root/.claude/hooks/heavy-test-guard.sh"
  ln -s "$REPO_ROOT/.git" "$r2root/.git"
  (cd "$REPO_ROOT" && nice -n 19 python3 scripts/agents/cite-check.py "$r2root/research/prompts/rot-defs-r2-sonnet-output.md" "$r2root/research/prompts/rot-defs-r2-opus-output.md" \
     --root "$r2root" --background "$r2root/research/prompts/_rot-defs-r2-background.md" 2>&1 | tail -n 1) > "$WORK/cite-check-r2-root.out" || true
  rm -rf "$r2root"
  rm -rf "$WORK/snapshot-tree" "$WORK/pre-swap-tree" "$WORK/tiny/target"
  local file
  for file in hooks.out probe-today.out probe-snapshot.out feeds-snapshot.out tiny.out cite-check.out r2-root-sources.out cite-check-r2-root.out r2-rerun-today.out r2-rerun-pre-swap.out r2-capped-diff.out; do
    if [[ -f "$MODEL/out/$file" ]]; then
      if diff -q "$MODEL/out/$file" "$WORK/$file" > /dev/null; then echo "name=compare file=$file same=yes"; else echo "name=compare file=$file same=no"; fi
    else
      echo "name=compare file=$file same=未留底"
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
