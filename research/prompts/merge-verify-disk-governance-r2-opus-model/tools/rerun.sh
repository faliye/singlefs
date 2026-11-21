#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条交 task-server 跑，工作目录随意；不套内存包装、不写线程：上限、线程与机器都由服务定）：
#   task-server/task submit -- bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 task-server/task（判是不是本仓）
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
#   RERUN_TEMPLATE_BREAK=root-check-by-wrapper  判是不是本仓看 scripts/hosts/run-with-memory-cap.sh 在不在（不看 task-server/task）
#       「✗ 自检：--repo 指到只有内存包装、没有 task-server/task 的旧仓应当退 2」那一格红
#       连带红的格（自证里的假仓只有 task-server/task、没有内存包装，被当成不是本仓）：「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」、「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条交 task-server/task submit -- bash <这份脚本> … 跑" >&2
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
  local repository_marker="task-server/task"
  [[ "${RERUN_TEMPLATE_BREAK:-}" != root-check-by-wrapper ]] || repository_marker="scripts/hosts/run-with-memory-cap.sh"
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/$repository_marker" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 $repository_marker）"
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
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 直接起（线程由 task-server 给整条套的 capped.sh <N> 定，这里不再套），
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
# 这一腿（merge-verify-disk-governance-r2 攻方）的四组格，跑前写死的预期写在每一行的 expected= 或格名旁的注释里：
#   verification_bin_section：照本仓验证包的清单与文件名铺一棵只有名字、文件都是空的树，补丁只往它的 Cargo.toml 加一段要特性的 [[bin]]；
#     被判的挑目标程序挑上的 --test 目标个数等于 tests/ 下 .rs 的个数；弄坏开关 manifest-bin-tests-unpicked 那一臂一个都不挑；验证包里写 CARGO_BIN_EXE_ 的文件 0 份
#   renamed_targets：补丁加一段名字不同、根文件就是自动认的那份的 [[bin]] 与 [[test]]（本仓 singlefs-unit-graph 的 unit-graph 段就是这个写法）：
#     被判程序判「自动认的目标被顶掉」，真 cargo 照新名字列得出那两份文件里的测试
#   bin_body：补丁只改 [[bin]] 根文件的正文，起它的集成测试由绿变红，被判程序只挑 --bin
#   debuginfo_profiles：测试那一趟之后关掉调试信息 build，依赖包 cargo 另编一份；不关的那一臂不另编
# 被判的脚本要是这一轮判的那一份（sha256 写死）；cargo 直接起，编译目录在 TARGET_ROOT（服务给的 CARGO_TARGET_DIR，没给就是 $WORK/target）底下每组一个。
JUDGED_SCRIPT_SHA256="75c1df5d5d8dc859a68241c9d4fae9adb3b23aa92530e75dfc7b81602fdf8d5c"
MATCHES=0
MISMATCHES=0
TARGET_ROOT=""
LAST_RC=0

# verdict_line <名> <matched|mismatched> <别的字段>：打一行结局、记数
verdict_line() {
  local name="$1" verdict="$2" fields="$3"
  if [[ "$verdict" == matched ]]; then MATCHES=$((MATCHES + 1)); else MISMATCHES=$((MISMATCHES + 1)); fi
  echo "name=$name $fields verdict=$verdict"
}

# make_patch <补丁目录> <树根起的相对路径> <旧文件> <新文件>：写一份只改这一个文件的 crates.patch（git 补丁头，a/ b/ 前缀）
make_patch() {
  mkdir -p "$1"
  { printf 'diff --git a/%s b/%s\n--- a/%s\n+++ b/%s\n' "$2" "$2" "$2" "$2"
    { git diff --no-index --no-color --no-ext-diff "$3" "$4" || true; } | tail -n +5; } > "$1/crates.patch"
}

# items_run <名> <树> <补丁目录> <弄坏开关或空串>：起被判的挑目标程序（targets 那一样），输出进 $WORK/logs/<名>.txt，退出码进 LAST_RC，每一行原样转打
items_run() {
  local line
  if python3 "$WORK/items_program.py" targets "$2" "$FROZEN/scripts/gate/rerun-failed-tests.py" "$FROZEN/scripts/gate/lib_rebuilt_packages.py" "$4" "$3" \
      > "$WORK/logs/$1.txt" 2>&1; then LAST_RC=0; else LAST_RC=$?; fi
  while IFS= read -r line; do echo "name=${1}_line text=${line//$'\t'/ | }"; done < "$WORK/logs/$1.txt"
}

# count_lines <模式> <文件>：grep -cE 的数（0 命中照打 0，文件读不了退出）
count_lines() {
  local found
  found="$(grep -cE -- "$1" "$2" || true)"
  echo "${found:-0}"
}

# 从被判的脚本里原样取出挑目标那一段 python（MERGE_AND_VERIFY_ITEMS_PROGRAM 的 here 文档正文）
extract_items_program() {
  local script="$1" output="$2" start end
  start="$(grep -nxF "MERGE_AND_VERIFY_ITEMS_PROGRAM=\"\$(cat <<'PY_ITEMS'" "$script" | cut -d: -f1)"
  end="$(grep -nxF 'PY_ITEMS' "$script" | cut -d: -f1)"
  [[ -n "$start" && -n "$end" && "$end" -gt "$start" ]] || { echo "  ✗ 在 $script 里找不到挑目标那一段 python（start=$start end=$end）" >&2; return 2; }
  sed -n "$((start + 1)),$((end - 1))p" "$script" > "$output"
  echo "name=items_program lines=$((end - start - 1)) sha256=$(sha256sum "$output" | cut -d' ' -f1)"
}

# verification_bin_section：本仓验证包的形状。树里只有工作区根、验证包此刻的 Cargo.toml 与 src/bin、tests 下各份根文件的名字（空文件），
#   补丁只往清单末尾加一段要特性的 [[bin]]（根文件是树里已有的空文件，补丁不碰它）。
verification_bin_section() {
  local real="$FROZEN/crates/singlefs-verification" tree="$WORK/verification-tree" package_directory file tests_count picked notes requires exe_files refusals
  package_directory="$tree/crates/singlefs-verification"
  mkdir -p "$package_directory/src/bin" "$package_directory/tests" "$WORK/old"
  printf '[workspace]\nmembers = ["crates/singlefs-verification"]\nresolver = "2"\n' > "$tree/Cargo.toml"
  : > "$package_directory/src/lib.rs"
  for file in "$real"/src/bin/*.rs; do : > "$package_directory/src/bin/${file##*/}"; done
  for file in "$real"/tests/*.rs; do : > "$package_directory/tests/${file##*/}"; done
  : > "$package_directory/src/bin/e999_model_apparatus.rs"
  cp "$real/Cargo.toml" "$WORK/old/verification-Cargo.toml"
  { cat "$real/Cargo.toml"; printf '\n[[bin]]\nname = "e999_model_apparatus"\npath = "src/bin/e999_model_apparatus.rs"\nrequired-features = ["apparatus-binaries"]\n'; } \
    > "$package_directory/Cargo.toml"
  make_patch "$WORK/patch-verification-bin" "crates/singlefs-verification/Cargo.toml" "$WORK/old/verification-Cargo.toml" "$package_directory/Cargo.toml"
  tests_count="$(find "$package_directory/tests" -maxdepth 1 -type f -name '*.rs' | wc -l)"
  echo "name=verification_shape tests_rs=$tests_count bins_rs=$(find "$package_directory/src/bin" -maxdepth 1 -type f -name '*.rs' | wc -l) patch_lines=$(wc -l < "$WORK/patch-verification-bin/crates.patch")"

  items_run verification_bin_intact "$tree" "$WORK/patch-verification-bin" ""
  picked="$(awk -F'\t' '$1 == "ITEM" && $3 == "singlefs-verification" && $4 ~ /^--test / { n++ } END { print n + 0 }' "$WORK/logs/verification_bin_intact.txt")"
  refusals="$(count_lines '^REFUSE' "$WORK/logs/verification_bin_intact.txt")"
  notes="$(count_lines "^NOTE.*集成测试目标 $tests_count 个" "$WORK/logs/verification_bin_intact.txt")"
  requires="$(count_lines $'^REQUIRES\tsinglefs-verification\te999_model_apparatus\t' "$WORK/logs/verification_bin_intact.txt")"
  # 预期：退 0、不退 3；挑上的 --test 目标个数等于 tests/ 下 .rs 的个数且不少于 400；说明那一句写着这个数；改了的那个 bin 自己记要特性（本次未跑）
  if [[ "$LAST_RC" -eq 0 && "$refusals" -eq 0 && "$picked" -eq "$tests_count" && "$tests_count" -ge 400 && "$notes" -eq 1 && "$requires" -eq 1 ]]; then
    verdict_line verification_bin_intact matched "exit=$LAST_RC picked_test_targets=$picked tests_rs=$tests_count refusals=$refusals note_with_count=$notes changed_bin_requires=$requires expected=picked_equals_tests_rs"
  else
    verdict_line verification_bin_intact mismatched "exit=$LAST_RC picked_test_targets=$picked tests_rs=$tests_count refusals=$refusals note_with_count=$notes changed_bin_requires=$requires expected=picked_equals_tests_rs"
  fi

  items_run verification_bin_unpicked "$tree" "$WORK/patch-verification-bin" "manifest-bin-tests-unpicked"
  picked="$(awk -F'\t' '$1 == "ITEM" && $3 == "singlefs-verification" && $4 ~ /^--test / { n++ } END { print n + 0 }' "$WORK/logs/verification_bin_unpicked.txt")"
  # 预期：弄坏开关 manifest-bin-tests-unpicked（不挑同包集成测试）那一臂一个 --test 目标都不挑：上面那几百个全是 [[bin]] 那一段挑的
  if [[ "$LAST_RC" -eq 0 && "$picked" -eq 0 ]]; then
    verdict_line verification_bin_unpicked matched "exit=$LAST_RC picked_test_targets=$picked expected=0"
  else
    verdict_line verification_bin_unpicked mismatched "exit=$LAST_RC picked_test_targets=$picked expected=0"
  fi

  exe_files="$(grep -rlF --include='*.rs' 'CARGO_BIN_EXE_' "$real" | wc -l || true)"
  # 预期：验证包里没有一份 .rs 写 CARGO_BIN_EXE_（挑上的集成测试一个都不起包里的 bin）
  if [[ "$exe_files" -eq 0 ]]; then
    verdict_line verification_bin_executable_users matched "files=$exe_files expected=0"
  else
    verdict_line verification_bin_executable_users mismatched "files=$exe_files expected=0"
  fi
}

# renamed_targets：包 ug 里 src/bin/ug_tool.rs 与 tests/foo.rs 原是自动认的目标（名字 ug_tool、foo）；补丁只往 Cargo.toml 加两段，
#   名字换成 ug-tool、foo-renamed，path 就是这两份文件（本仓 crates/singlefs-unit-graph/Cargo.toml 的 [[bin]] unit-graph 段就是这种写法）。两份文件照样是 cargo test 的目标。
renamed_targets() {
  local tree="$WORK/renamed-tree" target="$TARGET_ROOT/renamed" refusals shadow_bin shadow_test bin_listed test_listed
  mkdir -p "$tree/crates/ug/src/bin" "$tree/crates/ug/tests" "$WORK/old"
  printf '[workspace]\nmembers = ["crates/ug"]\nresolver = "2"\n' > "$tree/Cargo.toml"
  printf '[package]\nname = "ug"\nversion = "0.1.0"\nedition = "2021"\n' > "$WORK/old/ug-Cargo.toml"
  { cat "$WORK/old/ug-Cargo.toml"; printf '\n[[bin]]\nname = "ug-tool"\npath = "src/bin/ug_tool.rs"\n\n[[test]]\nname = "foo-renamed"\npath = "tests/foo.rs"\n'; } > "$tree/crates/ug/Cargo.toml"
  printf 'pub fn answer() -> u32 {\n    42\n}\n' > "$tree/crates/ug/src/lib.rs"
  printf 'fn main() {\n    println!("{}", ug::answer());\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn bin_unit_test_still_runs() {\n        assert_eq!(ug::answer(), 42);\n    }\n}\n' \
    > "$tree/crates/ug/src/bin/ug_tool.rs"
  printf '#[test]\nfn foo_still_runs() {\n    assert_eq!(ug::answer(), 42);\n}\n' > "$tree/crates/ug/tests/foo.rs"
  make_patch "$WORK/patch-renamed" "crates/ug/Cargo.toml" "$WORK/old/ug-Cargo.toml" "$tree/crates/ug/Cargo.toml"

  items_run renamed_items "$tree" "$WORK/patch-renamed" ""
  refusals="$(count_lines '^REFUSE' "$WORK/logs/renamed_items.txt")"
  shadow_bin="$(count_lines '^REFUSE.*自动认的目标被顶掉.*bin 目标 ug_tool' "$WORK/logs/renamed_items.txt")"
  shadow_test="$(count_lines '^REFUSE.*自动认的目标被顶掉.*test 目标 foo（' "$WORK/logs/renamed_items.txt")"
  # 预期：被判程序把这一份补丁判成「自动认的目标被顶掉」（REFUSE 一行，点名 bin 目标 ug_tool 与 test 目标 foo），合入后验证照它退 3
  if [[ "$LAST_RC" -eq 0 && "$refusals" -eq 1 && "$shadow_bin" -eq 1 && "$shadow_test" -eq 1 ]]; then
    verdict_line renamed_items matched "exit=$LAST_RC refusals=$refusals names_bin_ug_tool=$shadow_bin names_test_foo=$shadow_test expected=refused_as_shadowed"
  else
    verdict_line renamed_items mismatched "exit=$LAST_RC refusals=$refusals names_bin_ug_tool=$shadow_bin names_test_foo=$shadow_test expected=refused_as_shadowed"
  fi

  # 真 cargo：两份文件照新名字仍是目标，里面的测试照列（被判程序那一句「cargo test 从此不跑它的根文件」不成立）
  if (cd "$tree" && CARGO_TARGET_DIR="$target" cargo test --offline -p ug --bin ug-tool -- --list) > "$WORK/logs/renamed_bin_list.log" 2>&1; then LAST_RC=0; else LAST_RC=$?; fi
  bin_listed="$(count_lines '^tests::bin_unit_test_still_runs: test$' "$WORK/logs/renamed_bin_list.log")"
  if [[ "$LAST_RC" -eq 0 && "$bin_listed" -eq 1 ]]; then
    verdict_line renamed_bin_list matched "exit=$LAST_RC hits=$bin_listed expected=1"
  else
    verdict_line renamed_bin_list mismatched "exit=$LAST_RC hits=$bin_listed expected=1"
  fi
  if (cd "$tree" && CARGO_TARGET_DIR="$target" cargo test --offline -p ug --test foo-renamed -- --list) > "$WORK/logs/renamed_test_list.log" 2>&1; then LAST_RC=0; else LAST_RC=$?; fi
  test_listed="$(count_lines '^foo_still_runs: test$' "$WORK/logs/renamed_test_list.log")"
  if [[ "$LAST_RC" -eq 0 && "$test_listed" -eq 1 ]]; then
    verdict_line renamed_test_list matched "exit=$LAST_RC hits=$test_listed expected=1"
  else
    verdict_line renamed_test_list mismatched "exit=$LAST_RC hits=$test_listed expected=1"
  fi
}

# bin_body：包 alpha 的 [[bin]] tool（src/bin/tool.rs 打 hello），tests/cli.rs 经 CARGO_BIN_EXE_tool 起它、要 hello；补丁只改 tool.rs 的正文（打 changed）
bin_body() {
  local tree="$WORK/bin-body-tree" target="$TARGET_ROOT/bin-body" got refusals
  mkdir -p "$tree/crates/alpha/src/bin" "$tree/crates/alpha/tests" "$WORK/old"
  printf '[workspace]\nmembers = ["crates/alpha"]\nresolver = "2"\n' > "$tree/Cargo.toml"
  printf '[package]\nname = "alpha"\nversion = "0.1.0"\nedition = "2021"\n\n[[bin]]\nname = "tool"\npath = "src/bin/tool.rs"\n' > "$tree/crates/alpha/Cargo.toml"
  printf 'pub fn library_answer() -> u32 {\n    7\n}\n' > "$tree/crates/alpha/src/lib.rs"
  printf 'fn main() {\n    println!("hello");\n}\n' > "$tree/crates/alpha/src/bin/tool.rs"
  printf '#[test]\nfn tool_prints_hello() {\n    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tool")).output().expect("tool 起得来");\n    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");\n}\n' \
    > "$tree/crates/alpha/tests/cli.rs"
  if (cd "$tree" && CARGO_TARGET_DIR="$target" cargo test --offline -p alpha --test cli) > "$WORK/logs/bin_body_before_cli.log" 2>&1; then LAST_RC=0; else LAST_RC=$?; fi
  # 预期：补丁之前 cli 绿
  if [[ "$LAST_RC" -eq 0 ]]; then verdict_line bin_body_before_cli matched "exit=$LAST_RC expected=zero"; else verdict_line bin_body_before_cli mismatched "exit=$LAST_RC expected=zero"; fi

  cp "$tree/crates/alpha/src/bin/tool.rs" "$WORK/old/tool.rs"
  printf 'fn main() {\n    println!("changed");\n}\n' > "$tree/crates/alpha/src/bin/tool.rs"
  make_patch "$WORK/patch-bin-body" "crates/alpha/src/bin/tool.rs" "$WORK/old/tool.rs" "$tree/crates/alpha/src/bin/tool.rs"
  items_run bin_body_items "$tree" "$WORK/patch-bin-body" ""
  got="$(awk -F'\t' '$1 == "ITEM" { printf "%s%s|%s", (n++ ? ";" : ""), $2, $4 }' "$WORK/logs/bin_body_items.txt")"
  refusals="$(count_lines '^REFUSE' "$WORK/logs/bin_body_items.txt")"
  # 预期：被判程序只挑 alpha:src/bin/tool.rs|--bin tool，不挑 tests/cli.rs，不退 3
  if [[ "$LAST_RC" -eq 0 && "$got" == "alpha:src/bin/tool.rs|--bin tool" && "$refusals" -eq 0 ]]; then
    verdict_line bin_body_items matched "exit=$LAST_RC items=$got refusals=$refusals expected_items=alpha:src/bin/tool.rs|--bin tool"
  else
    verdict_line bin_body_items mismatched "exit=$LAST_RC items=$got refusals=$refusals expected_items=alpha:src/bin/tool.rs|--bin tool"
  fi
  if (cd "$tree" && CARGO_TARGET_DIR="$target" cargo test --offline -p alpha --bin tool) > "$WORK/logs/bin_body_after_picked.log" 2>&1; then LAST_RC=0; else LAST_RC=$?; fi
  # 预期：挑上的那一项绿（合入后验证照它退 0）
  if [[ "$LAST_RC" -eq 0 ]]; then verdict_line bin_body_after_picked matched "exit=$LAST_RC expected=zero"; else verdict_line bin_body_after_picked mismatched "exit=$LAST_RC expected=zero"; fi
  if (cd "$tree" && CARGO_TARGET_DIR="$target" cargo test --offline -p alpha --test cli) > "$WORK/logs/bin_body_after_cli.log" 2>&1; then LAST_RC=0; else LAST_RC=$?; fi
  # 预期：没挑上的 cli 红（提交时 cargo test --all 才报）
  if [[ "$LAST_RC" -ne 0 ]]; then
    verdict_line bin_body_after_cli matched "exit=$LAST_RC expected=nonzero first_error=$(grep -m1 -E 'assertion|panicked' "$WORK/logs/bin_body_after_cli.log" || true)"
  else
    verdict_line bin_body_after_cli mismatched "exit=$LAST_RC expected=nonzero"
  fi
}

# debuginfo_profiles：三包工作区 dbg-leaf ← dbg-core ← dbg-user，根清单 [profile.dev] debug = "line-tables-only"（同本仓根 Cargo.toml）。
#   照合入后验证的先后：先 cargo test -p dbg-core --lib（测试那一步），再 cargo build --all-targets -p dbg-core -p dbg-user（build 那一步）。
#   关掉那一臂 build 设 CARGO_PROFILE_DEV_DEBUG=0（被判脚本的写法），不关那一臂不设；各用一个新编译目录。
#   读：build 那一趟有没有 Compiling dbg-leaf（测试那一趟已编过它）、编译目录里 libdbg_leaf-*.rlib 几份、编译目录多大（玩具的数，不外推到本仓）。
write_debuginfo_tree() {
  local tree="$1" name
  for name in leaf core user; do mkdir -p "$tree/crates/dbg-$name/src"; done
  printf '[workspace]\nmembers = ["crates/dbg-leaf", "crates/dbg-core", "crates/dbg-user"]\nresolver = "2"\n\n[profile.dev]\ndebug = "line-tables-only"\n' > "$tree/Cargo.toml"
  printf '[package]\nname = "dbg-leaf"\nversion = "0.1.0"\nedition = "2021"\n' > "$tree/crates/dbg-leaf/Cargo.toml"
  printf 'pub fn base(value: u64) -> u64 {\n    value.wrapping_mul(31)\n}\n' > "$tree/crates/dbg-leaf/src/lib.rs"
  printf '[package]\nname = "dbg-core"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\ndbg-leaf = { path = "../dbg-leaf" }\n' > "$tree/crates/dbg-core/Cargo.toml"
  printf 'pub fn twice(value: u64) -> u64 {\n    dbg_leaf::base(value) * 2\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn twice_of_three() {\n        assert_eq!(super::twice(3), 186);\n    }\n}\n' \
    > "$tree/crates/dbg-core/src/lib.rs"
  printf '[package]\nname = "dbg-user"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\ndbg-core = { path = "../dbg-core" }\n' > "$tree/crates/dbg-user/Cargo.toml"
  printf 'fn main() {\n    println!("{}", dbg_core::twice(3));\n}\n' > "$tree/crates/dbg-user/src/main.rs"
}

# debuginfo_arm <臂名> <off|kept> <预期的 Compiling dbg-leaf 行数> <预期的 libdbg_leaf rlib 份数>
debuginfo_arm() {
  local arm="$1" mode="$2" expected_compiling="$3" expected_rlibs="$4" tree="$WORK/debuginfo-tree" target="$TARGET_ROOT/debuginfo-$1" test_rc build_rc compiling rlibs size_kib
  if (cd "$tree" && CARGO_TARGET_DIR="$target" cargo test --offline -p dbg-core --lib) > "$WORK/logs/debuginfo_${arm}_test.log" 2>&1; then test_rc=0; else test_rc=$?; fi
  if [[ "$mode" == off ]]; then
    if (cd "$tree" && CARGO_TARGET_DIR="$target" env CARGO_PROFILE_DEV_DEBUG=0 cargo build --offline --all-targets -p dbg-core -p dbg-user) > "$WORK/logs/debuginfo_${arm}_build.log" 2>&1; then build_rc=0; else build_rc=$?; fi
  else
    if (cd "$tree" && CARGO_TARGET_DIR="$target" cargo build --offline --all-targets -p dbg-core -p dbg-user) > "$WORK/logs/debuginfo_${arm}_build.log" 2>&1; then build_rc=0; else build_rc=$?; fi
  fi
  compiling="$(count_lines '^[[:space:]]*Compiling dbg-leaf v' "$WORK/logs/debuginfo_${arm}_build.log")"
  rlibs="$( { find "$target/debug/deps" -maxdepth 1 -name 'libdbg_leaf-*.rlib' || true; } | wc -l)"
  size_kib="$(du -sk "$target" | cut -f1)"
  if [[ "$test_rc" -eq 0 && "$build_rc" -eq 0 && "$compiling" -eq "$expected_compiling" && "$rlibs" -eq "$expected_rlibs" ]]; then
    verdict_line "debuginfo_$arm" matched "test_exit=$test_rc build_exit=$build_rc build_compiling_leaf=$compiling leaf_rlibs=$rlibs target_kib=$size_kib expected_compiling_leaf=$expected_compiling expected_leaf_rlibs=$expected_rlibs"
  else
    verdict_line "debuginfo_$arm" mismatched "test_exit=$test_rc build_exit=$build_rc build_compiling_leaf=$compiling leaf_rlibs=$rlibs target_kib=$size_kib expected_compiling_leaf=$expected_compiling expected_leaf_rlibs=$expected_rlibs"
  fi
}

rerun_steps() {
  local actual
  unset CARGO_PROFILE_DEV_DEBUG
  mkdir -p "$WORK/logs"
  TARGET_ROOT="${CARGO_TARGET_DIR:-$WORK/target}"
  actual="$(sha256sum "$FROZEN/scripts/gate/merge-and-verify.sh" | cut -d' ' -f1)"
  if [[ "$actual" != "$JUDGED_SCRIPT_SHA256" ]]; then
    echo "  ✗ $FROZEN/scripts/gate/merge-and-verify.sh 的 sha256 是 $actual，不是这一轮判的那一份 $JUDGED_SCRIPT_SHA256：被判脚本换过，这一趟不跑" >&2
    return 2
  fi
  echo "name=judged_script sha256=$actual rerun_failed_tests_sha256=$(sha256sum "$FROZEN/scripts/gate/rerun-failed-tests.py" | cut -d' ' -f1) lib_rebuilt_packages_sha256=$(sha256sum "$FROZEN/scripts/gate/lib_rebuilt_packages.py" | cut -d' ' -f1)"
  echo "name=toolchain cargo=$(cargo --version | tr ' ' '_') rustc=$(rustc --version | tr ' ' '_')"
  extract_items_program "$FROZEN/scripts/gate/merge-and-verify.sh" "$WORK/items_program.py"
  verification_bin_section
  renamed_targets
  bin_body
  write_debuginfo_tree "$WORK/debuginfo-tree"
  debuginfo_arm off off 1 2
  debuginfo_arm kept kept 0 1
  echo "name=summary matched=$MATCHES mismatched=$MISMATCHES"
}

# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  export RERUN_TEMPLATE_SELFTEST_MARK="rerun-template-selftest"   # 自证里拷出去起的子进程（环境原样继承）靠它认：弄坏开关只在自证里认
  mkdir -p "$scratch/fake-repo/.git" "$scratch/fake-repo/task-server" "$scratch/old-repo/.git" "$scratch/old-repo/scripts/hosts" "$scratch/frozen/crates/target" \
    "$scratch/elsewhere/r1-opus-model/tools" "$scratch/cwd"
  : > "$scratch/fake-repo/task-server/task"
  : > "$scratch/old-repo/scripts/hosts/run-with-memory-cap.sh"
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
  if bash "$copy" --repo "$scratch/old-repo" --frozen "$scratch/frozen" --work "$scratch/work6" > "$scratch/run6.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run6.out"; then
    failures+=("--repo 指到只有内存包装、没有 task-server/task 的旧仓应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run6.out")")
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
  echo "  ✓ 自检通过（查了 $checked 格）：拷进与仓无关的临时目录、从别的目录起，传参跑通；不传参、缺 --frozen、--repo 不是仓、--repo 是没有 task-server/task 的旧仓、--work 非空都退 2 并说明"
  return 0
}

# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（RERUN_TEMPLATE_SELFTEST_MARK 对得上）而变量非空，参数解析之前就退 2（不管值是什么）。
if [[ -n "${RERUN_TEMPLATE_BREAK:-}" && ! ( "$#" -eq 1 && "${1:-}" == --selftest ) && "${RERUN_TEMPLATE_SELFTEST_MARK:-}" != rerun-template-selftest ]]; then
  echo "  ✗ RERUN_TEMPLATE_BREAK=${RERUN_TEMPLATE_BREAK} 是弄坏开关，只在自证（bash <这份脚本> --selftest）里认，平常跑设了就拒，这一趟什么都没做" >&2
  echo "     → 怎么办：去掉 RERUN_TEMPLATE_BREAK 再跑；要证红就跑 RERUN_TEMPLATE_BREAK=<项> bash scripts/agents/three-way-rerun-template.sh --selftest" >&2
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
