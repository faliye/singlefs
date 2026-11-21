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
# 这一腿（merge-verify-disk-governance-r2 辩方）的模型：把第一轮判决的几处依据接到现在的脚本上，跑前写死的预期见每一行的 expected=。
#   一、改法 1（恢复 build、关掉调试信息）：现在脚本自己算出来的 build 包集合（MAIN_TREE_FINGERPRINT_PROGRAM 的 build-packages 一段，从被判脚本里原样取出）
#       在第一轮的单态化玩具树上是不是 poly-core 与 poly-user；拿这个集合起 CARGO_PROFILE_DEV_DEBUG=0 的 cargo build，改前绿、改后红。
#       另有一格「依赖写成 workspace = true」：包集合漏掉依赖方，同一个单态化错漏过（本仓今天没有这种写法，是造得出的形状）。
#   二、判决 2、3 同一形状：只改 bin 源码、只改库源码，被挑的目标与集成测试结局；以及现在脚本对 [[bin]] 的 path 与新加 [[test]] 同名两格的结局。
#   三、build 用 CARGO_PROFILE_DEV_DEBUG=0、测试不设：同一个编译目录里测试那一步要不要把 poly-core 重编一遍（只量、不判）。
# 被判的脚本要是这一轮判的那一份（sha256 写死）；仓根与冻结副本可以是同一个目录。cargo 直接起，编译目录在 TARGET_ROOT 底下每棵树一个。
JUDGED_SCRIPT_SHA256="75c1df5d5d8dc859a68241c9d4fae9adb3b23aa92530e75dfc7b81602fdf8d5c"
MATCHES=0
MISMATCHES=0
TARGET_ROOT=""

# record <名> <zero|nonzero|observe> <退出码> <日志>：打一行结局，observe 只记不判
record() {
  local name="$1" expected="$2" rc="$3" log="$4" verdict=mismatched first_error
  if [[ "$expected" == zero && "$rc" -eq 0 ]] || [[ "$expected" == nonzero && "$rc" -ne 0 ]] || [[ "$expected" == observe ]]; then verdict=matched; fi
  first_error="$(grep -m1 -E '^error|undefined reference|evaluation of|assertion' "$log" || true)"
  if [[ "$verdict" == matched ]]; then MATCHES=$((MATCHES + 1)); else MISMATCHES=$((MISMATCHES + 1)); fi
  echo "name=$name exit=$rc expected=$expected verdict=$verdict first_error=${first_error:0:240}"
}

# run_in <目录> <名> <zero|nonzero|observe> <命令…>：在目录里起命令，输出进 $WORK/logs/<名>.log
run_in() {
  local directory="$1" name="$2" expected="$3" rc
  shift 3
  if (cd "$directory" && CARGO_TARGET_DIR="$TARGET_ROOT/${directory#"$WORK"/}" "$@") > "$WORK/logs/$name.log" 2>&1; then rc=0; else rc=$?; fi
  record "$name" "$expected" "$rc" "$WORK/logs/$name.log"
}

# make_patch <补丁目录> <树根起的相对路径> <旧文件> <新文件>：写一份只改这一个文件的 crates.patch
make_patch() {
  mkdir -p "$1"
  { printf 'diff --git a/%s b/%s\n--- a/%s\n+++ b/%s\n' "$2" "$2" "$2" "$2"
    { git diff --no-index --no-color --no-ext-diff "$3" "$4" || true; } | tail -n +5; } > "$1/crates.patch"
}

# extract_here_document <脚本> <起行全文> <止行全文> <输出文件>：取两行之间的 here 文档正文
extract_here_document() {
  local script="$1" start_text="$2" end_text="$3" output="$4" start end
  start="$(grep -nxF "$start_text" "$script" | head -n 1 | cut -d: -f1)"
  end="$(grep -nxF "$end_text" "$script" | head -n 1 | cut -d: -f1)"
  [[ -n "$start" && -n "$end" && "$end" -gt "$start" ]] || { echo "  ✗ 在 $script 里找不到 $end_text 那一段（start=$start end=$end）" >&2; return 2; }
  sed -n "$((start + 1)),$((end - 1))p" "$script" > "$output"
  echo "name=extracted program=$end_text lines=$((end - start - 1)) sha256=$(sha256sum "$output" | cut -d' ' -f1)"
}

# items_run <名> <树> <补丁目录>：起被判的挑目标程序，打出它的每一行；ITEMS_GOT 是排好序的「项|挑法」用 ; 接，REFUSALS 是 REFUSE 行数
ITEMS_GOT=""
REFUSALS=0
items_run() {
  local name="$1" tree="$2" patch="$3" line
  python3 "$WORK/items_program.py" targets "$tree" "$FROZEN/scripts/gate/rerun-failed-tests.py" "$FROZEN/scripts/gate/lib_rebuilt_packages.py" "" "$patch" \
    > "$WORK/logs/$name.txt" 2>&1 || true
  while IFS= read -r line; do echo "name=${name}_line text=${line//$'\t'/ | }"; done < "$WORK/logs/$name.txt"
  ITEMS_GOT="$(awk -F'\t' '$1 == "ITEM" { print $2 "|" $4 }' "$WORK/logs/$name.txt" | LC_ALL=C sort | paste -sd';' -)"
  REFUSALS="$(grep -c '^REFUSE' "$WORK/logs/$name.txt" || true)"
}

# items_check <名> <树> <补丁目录> <预期的项（排好序、; 接）或 REFUSED（没有项、至少一条 REFUSE）>
items_check() {
  local name="$1" expected="$4" verdict=mismatched
  items_run "$name" "$2" "$3"
  if [[ "$expected" == REFUSED ]]; then
    [[ -z "$ITEMS_GOT" && "$REFUSALS" -ge 1 ]] && verdict=matched
  else
    [[ "$ITEMS_GOT" == "$expected" && "$REFUSALS" -eq 0 ]] && verdict=matched
  fi
  if [[ "$verdict" == matched ]]; then MATCHES=$((MATCHES + 1)); else MISMATCHES=$((MISMATCHES + 1)); fi
  echo "name=$name items=$ITEMS_GOT expected_items=$expected refusals=$REFUSALS verdict=$verdict"
}

# write_workspace <树> [继承依赖：1 写 [workspace.dependencies]]：工作区根的 Cargo.toml（成员是 crates/ 下各包；dev 档只留行号表，与仓根同）
write_workspace() {
  mkdir -p "$1/crates"
  printf '[workspace]\nmembers = ["crates/*"]\nresolver = "2"\n' > "$1/Cargo.toml"
  if [[ "${2:-0}" == 1 ]]; then printf '\n[workspace.dependencies]\npoly-core = { path = "crates/poly-core" }\n' >> "$1/Cargo.toml"; fi
  printf '\n[profile.dev]\ndebug = "line-tables-only"\n' >> "$1/Cargo.toml"
}

# write_poly_tree <树> <上限字节数> <依赖写法：path 或 workspace>：poly-core（泛型里的常量断言，单态化时才求值）与依赖它的 poly-user（一个 bin 存 32 字节的值）
write_poly_tree() {
  local dependency='poly-core = { path = "../poly-core" }' inherit=0
  if [[ "$3" == workspace ]]; then dependency='poly-core = { workspace = true }'; inherit=1; fi
  write_workspace "$1" "$inherit"
  mkdir -p "$1/crates/poly-core/src" "$1/crates/poly-user/src"
  printf '[package]\nname = "poly-core"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n' > "$1/crates/poly-core/Cargo.toml"
  cat > "$1/crates/poly-core/src/lib.rs" <<RUST
//! 模型：泛型函数里的常量断言只在单态化时求值。

/// 存一个值：值的大小超过上限时，在单态化那一步编不过。
pub fn store<T: Copy>(value: T) -> T {
    const { assert!(core::mem::size_of::<T>() <= $2, "值超过上限") };
    value
}

#[cfg(test)]
mod tests {
    #[test]
    fn small_value_round_trips() {
        assert_eq!(super::store(3_u32), 3);
    }
}
RUST
  printf '[package]\nname = "poly-user"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n\n[dependencies]\n%s\n' "$dependency" > "$1/crates/poly-user/Cargo.toml"
  cat > "$1/crates/poly-user/src/main.rs" <<'RUST'
fn main() {
    let wide = poly_core::store([0_u64; 4]);
    println!("{}", wide.len());
}
RUST
}

# build_packages_of <树> <补丁目录>：被判脚本自己算的 build 包集合，一行一个包名，打进 $WORK/logs/<名>.txt 由调用方读
build_packages_of() {
  python3 "$WORK/fingerprint_program.py" "$FROZEN/scripts/gate/lib_rebuilt_packages.py" "$1" build-packages "$2"
}

# build_packages_check <名> <树> <补丁目录> <预期的包集合，排好序、; 接>：打出 BUILD_PACKAGES（; 接）
BUILD_PACKAGES=""
build_packages_check() {
  local name="$1" expected="$4" verdict=mismatched
  build_packages_of "$2" "$3" > "$WORK/logs/$name.txt" 2> "$WORK/logs/$name.err" || true
  BUILD_PACKAGES="$(LC_ALL=C sort "$WORK/logs/$name.txt" | paste -sd';' -)"
  [[ "$BUILD_PACKAGES" == "$expected" ]] && verdict=matched
  if [[ "$verdict" == matched ]]; then MATCHES=$((MATCHES + 1)); else MISMATCHES=$((MISMATCHES + 1)); fi
  echo "name=$name packages=$BUILD_PACKAGES expected_packages=$expected verdict=$verdict"
}

# build_with_set <树> <名> <zero|nonzero|observe> <环境词…>：照 BUILD_PACKAGES（; 接）逐个加 -p、起 cargo build --offline --all-targets（与 check.sh 那一行同参数）
build_with_set() {
  local tree="$1" name="$2" expected="$3" package
  shift 3
  local -a words=(cargo build --offline --all-targets)
  for package in ${BUILD_PACKAGES//;/ }; do words+=(-p "$package"); done
  if (($#)); then run_in "$tree" "$name" "$expected" env "$@" "${words[@]}"; else run_in "$tree" "$name" "$expected" "${words[@]}"; fi
}

# 去掉 build 那一块：改前（上限 64）三步都过，改后（上限 8）依赖方 poly-user 的 main 单态化时求值失败
poly_model() {
  local base="$WORK/poly"
  write_poly_tree "$base/before" 64 path
  write_poly_tree "$base/after" 8 path
  write_poly_tree "$base/before-inherit" 64 workspace
  write_poly_tree "$base/after-inherit" 8 workspace
  make_patch "$base/patch" crates/poly-core/src/lib.rs "$base/before/crates/poly-core/src/lib.rs" "$base/after/crates/poly-core/src/lib.rs"
  make_patch "$base/patch-inherit" crates/poly-core/src/lib.rs "$base/before-inherit/crates/poly-core/src/lib.rs" "$base/after-inherit/crates/poly-core/src/lib.rs"
  build_packages_check poly_build_packages_path "$base/after" "$base/patch" "poly-core;poly-user"
  build_with_set "$base/before" poly_before_build_without_debuginfo zero CARGO_PROFILE_DEV_DEBUG=0
  build_with_set "$base/after" poly_after_build_without_debuginfo nonzero CARGO_PROFILE_DEV_DEBUG=0
  build_with_set "$base/after" poly_after_build_default_debuginfo nonzero
  run_in "$base/after" poly_after_clippy_whole_workspace zero cargo clippy --offline --all-targets -- -D warnings
  run_in "$base/after" poly_after_picked_test zero cargo test --offline --no-fail-fast -p poly-core --lib
  # 依赖写成 workspace = true：被判脚本的 path_dependencies 只认 path，包集合漏掉依赖方（本仓今天没有 workspace = true）
  build_packages_check poly_build_packages_workspace_inherit "$base/after-inherit" "$base/patch-inherit" "poly-core"
  BUILD_PACKAGES="poly-core;poly-user"
  build_with_set "$base/after-inherit" poly_inherit_dependent_build_would_fail nonzero CARGO_PROFILE_DEV_DEBUG=0
}

# alpha_manifest <before|binpath|shadow>：alpha 的 Cargo.toml 正文
alpha_manifest() {
  local bin_path="src/bin/tool.rs"
  [[ "$1" != binpath ]] || bin_path="bins/tool_next.rs"
  printf '[package]\nname = "alpha"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n\n[[bin]]\nname = "tool"\npath = "%s"\n' "$bin_path"
  if [[ "$1" == shadow ]]; then printf '\n[[test]]\nname = "beta"\npath = "checks/other.rs"\n'; fi
}

# write_alpha_tree <树> <before|binpath|shadow>：alpha 包（库、起库的 bin、起 bin 的集成测试 cli、读库的集成测试 beta）；三种只差 Cargo.toml
write_alpha_tree() {
  write_workspace "$1"
  local package="$1/crates/alpha"
  mkdir -p "$package/src/bin" "$package/bins" "$package/tests" "$package/checks"
  alpha_manifest "$2" > "$package/Cargo.toml"
  cat > "$package/src/lib.rs" <<'RUST'
//! 模型：bin 与集成测试共用的库。

/// 工具打出的那一句。
#[must_use]
pub fn greeting() -> &'static str {
    "hello"
}

#[cfg(test)]
mod tests {
    #[test]
    fn greeting_is_hello() {
        assert_eq!(super::greeting(), "hello");
    }
}
RUST
  printf 'fn main() {\n    println!("{}", alpha::greeting());\n}\n' > "$package/src/bin/tool.rs"
  printf 'fn main() {\n    println!("changed");\n}\n' > "$package/bins/tool_next.rs"
  cat > "$package/tests/cli.rs" <<'RUST'
//! 起本包的 bin tool（cargo 交给集成测试的 CARGO_BIN_EXE_tool），核它打的那一句。
#[test]
fn tool_prints_hello() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tool")).output().expect("tool 起得来");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
}
RUST
  printf '#[test]\nfn beta_guards_greeting() {\n    assert_eq!(alpha::greeting(), "hello");\n}\n' > "$package/tests/beta.rs"
  printf '#[test]\nfn other_check() {\n    assert_eq!(alpha::greeting().len(), 5);\n}\n' > "$package/checks/other.rs"
}

# 挑测试目标那一块：同一棵 alpha，四份补丁各改一处
#   srcedit：只改 bin 源码 src/bin/tool.rs（打 changed）；libedit：只改库源码 src/lib.rs（hello 换成 hullo）；
#   binpath：[[bin]] tool 的 path 换成已有的 bins/tool_next.rs；shadow：新加 [[test]] name = "beta" path = "checks/other.rs"。
alpha_model() {
  local base="$WORK/alpha" variant
  for variant in before binpath shadow; do write_alpha_tree "$base/$variant" "$variant"; done
  write_alpha_tree "$base/srcedit" before
  write_alpha_tree "$base/libedit" before
  cp "$base/before/crates/alpha/bins/tool_next.rs" "$base/srcedit/crates/alpha/src/bin/tool.rs"
  printf '//! 模型：bin 与集成测试共用的库。\n\n/// 工具打出的那一句。\n#[must_use]\npub fn greeting() -> &'\''static str {\n    "hullo"\n}\n' > "$base/libedit/crates/alpha/src/lib.rs"
  make_patch "$base/patch-srcedit" crates/alpha/src/bin/tool.rs "$base/before/crates/alpha/src/bin/tool.rs" "$base/srcedit/crates/alpha/src/bin/tool.rs"
  make_patch "$base/patch-libedit" crates/alpha/src/lib.rs "$base/before/crates/alpha/src/lib.rs" "$base/libedit/crates/alpha/src/lib.rs"
  make_patch "$base/patch-binpath" crates/alpha/Cargo.toml "$base/before/crates/alpha/Cargo.toml" "$base/binpath/crates/alpha/Cargo.toml"
  make_patch "$base/patch-shadow" crates/alpha/Cargo.toml "$base/before/crates/alpha/Cargo.toml" "$base/shadow/crates/alpha/Cargo.toml"
  items_check alpha_items_srcedit "$base/srcedit" "$base/patch-srcedit" "alpha:src/bin/tool.rs|--bin tool"
  run_in "$base/srcedit" alpha_srcedit_cli_test_goes_red nonzero cargo test --offline --no-fail-fast -p alpha --test cli
  items_check alpha_items_libedit "$base/libedit" "$base/patch-libedit" "alpha:src/lib.rs|--lib"
  run_in "$base/libedit" alpha_libedit_cli_test_goes_red nonzero cargo test --offline --no-fail-fast -p alpha --test cli
  run_in "$base/libedit" alpha_libedit_beta_test_goes_red nonzero cargo test --offline --no-fail-fast -p alpha --test beta
  items_check alpha_items_binpath "$base/binpath" "$base/patch-binpath" "alpha:bins/tool_next.rs|--bin tool;alpha:tests/beta.rs|--test beta;alpha:tests/cli.rs|--test cli"
  items_check alpha_items_shadow "$base/shadow" "$base/patch-shadow" REFUSED
}

# 一个编译目录里，build 关调试信息、测试不关：测试那一步是不是把 poly-core 重编一遍，再起同一条 build 是不是又重编（只量）
profile_split_model() {
  local tree="$WORK/poly/before" log="$WORK/logs"
  run_in "$tree" split_build_without_debuginfo zero env CARGO_PROFILE_DEV_DEBUG=0 cargo build --offline --all-targets -p poly-core -p poly-user
  run_in "$tree" split_test_after_build observe cargo test --offline --no-fail-fast -p poly-core --lib
  echo "name=split_compiling_poly_core_in_test count=$(grep -c '^ *Compiling poly-core v' "$log/split_test_after_build.log" || true)"
  run_in "$tree" split_build_again observe env CARGO_PROFILE_DEV_DEBUG=0 cargo build --offline --all-targets -p poly-core -p poly-user
  echo "name=split_compiling_poly_core_in_second_build count=$(grep -c '^ *Compiling poly-core v' "$log/split_build_again.log" || true)"
}

# build 那一条的 CARGO_PROFILE_DEV_DEBUG=0 在仓根写了 [profile.dev] debug = "line-tables-only" 的工作区里真起作用没有：同一份源码两棵树，一棵带环境变量、一棵不带，
# 各自 cargo build -v 之后数 rustc 命令行里的 debuginfo= 参数（只量、不判）
debuginfo_flag_model() {
  local base="$WORK/poly" log="$WORK/logs"
  write_poly_tree "$base/flags-env" 64 path
  write_poly_tree "$base/flags-default" 64 path
  run_in "$base/flags-env" flags_build_with_env observe env CARGO_PROFILE_DEV_DEBUG=0 cargo build --offline -v -p poly-core
  run_in "$base/flags-default" flags_build_default observe cargo build --offline -v -p poly-core
  echo "name=flags_debuginfo_with_env values=$(grep -o debuginfo=[0-9a-z-]* "$log/flags_build_with_env.log" | sort -u | paste -sd, - || true)"
  echo "name=flags_debuginfo_default values=$(grep -o debuginfo=[0-9a-z-]* "$log/flags_build_default.log" | sort -u | paste -sd, - || true)"
}

rerun_steps() {
  local judged="$FROZEN/scripts/gate/merge-and-verify.sh" got
  got="$(sha256sum "$judged" | cut -d' ' -f1)"
  if [[ "$got" != "$JUDGED_SCRIPT_SHA256" ]]; then
    echo "  ✗ $judged 的 sha256 是 $got，不是这一轮判的那一份 $JUDGED_SCRIPT_SHA256：模型要读的被判对象变了，没跑" >&2
    echo "  → 怎么办：--frozen 给一棵 scripts/gate/merge-and-verify.sh 还是这一轮判的那一份的树" >&2
    return 2
  fi
  mkdir -p "$WORK/logs"
  TARGET_ROOT="${CARGO_TARGET_DIR:-$WORK/target}"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK target_root=$TARGET_ROOT"
  echo "name=toolchain rustc=$(rustc --version | tr ' ' '_') cargo=$(cargo --version | tr ' ' '_')"
  extract_here_document "$judged" "MERGE_AND_VERIFY_ITEMS_PROGRAM=\"\$(cat <<'PY_ITEMS'" PY_ITEMS "$WORK/items_program.py"
  extract_here_document "$judged" "MAIN_TREE_FINGERPRINT_PROGRAM=\"\$(cat <<'PY_MAIN_TREE_FINGERPRINT'" PY_MAIN_TREE_FINGERPRINT "$WORK/fingerprint_program.py"
  poly_model
  alpha_model
  profile_split_model
  debuginfo_flag_model
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
