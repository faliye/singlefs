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
# 这一腿（merge-verify-disk-governance-r1 攻方）的两个模型，跑前写死的预期见每一行的 expected=：
#   去掉 build 那一块：玩具工作区 poly-core（库）与 poly-user（依赖它的 bin），补丁只改 poly-core 的 src/lib.rs；
#     泛型里的常量断言（单态化时才求值）与按符号链接的导出函数，clippy、合入后验证挑出的测试目标都编得过，只有 cargo build 编不过。
#   挑测试目标那一块：玩具包 alpha，补丁只改它的 Cargo.toml 的 [[bin]] / [[test]] 段；被判的挑目标程序从冻结的 merge-and-verify.sh 里原样取出来跑，
#     再用真 cargo 看被挑漏的那几个目标的结局。
# 被判的脚本要是这一轮判的那一份（sha256 写死）；cargo 直接起，编译目录在 TARGET_ROOT（服务给的 CARGO_TARGET_DIR，没给就是 $WORK/target）底下每棵树一个。
JUDGED_SCRIPT_SHA256="c6badd7bdfb5505053ed6cb89a8f4f2db577539f46753a95aff0bca9f8b00d2a"
MATCHES=0
MISMATCHES=0
TARGET_ROOT=""

# record <名> <zero|nonzero> <退出码> <日志>：打一行结局，记对不对得上预期
record() {
  local name="$1" expected="$2" rc="$3" log="$4" verdict=mismatched first_error
  if [[ "$expected" == zero && "$rc" -eq 0 ]] || [[ "$expected" == nonzero && "$rc" -ne 0 ]]; then verdict=matched; fi
  first_error="$(grep -m1 -E '^error|undefined reference|evaluation of|assertion' "$log" || true)"
  if [[ "$verdict" == matched ]]; then MATCHES=$((MATCHES + 1)); else MISMATCHES=$((MISMATCHES + 1)); fi
  echo "name=$name exit=$rc expected=$expected verdict=$verdict first_error=${first_error:0:240}"
}

# run_in <目录> <名> <zero|nonzero> <命令…>：在目录里起命令，输出进 $WORK/logs/<名>.log
run_in() {
  local directory="$1" name="$2" expected="$3" rc
  shift 3
  if (cd "$directory" && CARGO_TARGET_DIR="$TARGET_ROOT/${directory#"$WORK"/}" "$@") > "$WORK/logs/$name.log" 2>&1; then rc=0; else rc=$?; fi
  record "$name" "$expected" "$rc" "$WORK/logs/$name.log"
}

# make_patch <补丁目录> <树根起的相对路径> <旧文件> <新文件>：写一份只改这一个文件的 crates.patch（git 补丁头，a/ b/ 前缀）
make_patch() {
  mkdir -p "$1"
  { printf 'diff --git a/%s b/%s\n--- a/%s\n+++ b/%s\n' "$2" "$2" "$2" "$2"
    { git diff --no-index --no-color --no-ext-diff "$3" "$4" || true; } | tail -n +5; } > "$1/crates.patch"
}

# items_check <名> <树> <补丁目录> <预期的 ITEM（「项|挑目标的参数」，几项用 ; 接，按打出的次序）>：起被判的挑目标程序（targets 那一样），
#   打出它的每一行，判 ITEM 恰好是预期的那几项、没有 REFUSE
items_check() {
  local name="$1" tree="$2" patch="$3" expected="$4" rc got refusals verdict=mismatched line
  if python3 "$WORK/items_program.py" targets "$tree" "$FROZEN/scripts/gate/rerun-failed-tests.py" "$FROZEN/scripts/gate/lib_rebuilt_packages.py" "" "$patch" \
      > "$WORK/logs/$name.txt" 2>&1; then rc=0; else rc=$?; fi
  while IFS= read -r line; do echo "name=${name}_line text=${line//$'\t'/ | }"; done < "$WORK/logs/$name.txt"
  got="$(awk -F'\t' '$1 == "ITEM" { printf "%s%s|%s", (n++ ? ";" : ""), $2, $4 }' "$WORK/logs/$name.txt")"
  refusals="$(grep -c '^REFUSE' "$WORK/logs/$name.txt" || true)"
  if [[ "$rc" -eq 0 && "$got" == "$expected" && "$refusals" -eq 0 ]]; then verdict=matched; fi
  if [[ "$verdict" == matched ]]; then MATCHES=$((MATCHES + 1)); else MISMATCHES=$((MISMATCHES + 1)); fi
  echo "name=$name exit=$rc items=$got expected_items=$expected refusals=$refusals verdict=$verdict"
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

# write_workspace <树>：工作区根的 Cargo.toml（成员是 crates/ 下各包）
write_workspace() {
  mkdir -p "$1/crates"
  printf '[workspace]\nmembers = ["crates/*"]\nresolver = "2"\n' > "$1/Cargo.toml"
}

# poly_core_library <上限字节数> <导不导出符号：1 或 0>：poly-core 的 src/lib.rs 正文
poly_core_library() {
  local attribute=""
  [[ "$2" == 1 ]] && attribute='#[no_mangle]'
  cat <<RUST
//! 模型：泛型函数里的常量断言只在单态化时求值；按 C 的写法导出、给别的包按符号链接的函数。

/// 存一个值：值的大小超过上限时，在单态化那一步编不过。
pub fn store<T: Copy>(value: T) -> T {
    const { assert!(core::mem::size_of::<T>() <= $1, "值超过上限") };
    value
}

/// 别的包在 extern 块里按这个名字声明、链接时找它。
$attribute
pub extern "C" fn singlefs_model_answer() -> i32 {
    42
}

#[cfg(test)]
mod tests {
    #[test]
    fn small_value_round_trips() {
        assert_eq!(super::store(3_u32), 3);
    }
}
RUST
}

# write_poly_tree <树> <上限字节数> <导不导出符号：1 或 0>：poly-core 与依赖它的 poly-user（一个 bin 存 32 字节的值，一个 bin 按符号调 poly-core）
write_poly_tree() {
  write_workspace "$1"
  mkdir -p "$1/crates/poly-core/src" "$1/crates/poly-user/src/bin"
  printf '[package]\nname = "poly-core"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n' > "$1/crates/poly-core/Cargo.toml"
  poly_core_library "$2" "$3" > "$1/crates/poly-core/src/lib.rs"
  printf '[package]\nname = "poly-user"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n\n[dependencies]\npoly-core = { path = "../poly-core" }\n' \
    > "$1/crates/poly-user/Cargo.toml"
  cat > "$1/crates/poly-user/src/main.rs" <<'RUST'
fn main() {
    let wide = poly_core::store([0_u64; 4]);
    println!("{}", wide.len());
}
RUST
  cat > "$1/crates/poly-user/src/bin/links.rs" <<'RUST'
extern "C" {
    fn singlefs_model_answer() -> i32;
}

fn main() {
    let small = poly_core::store(1_u8);
    // SAFETY: 符号由 poly-core 按 C 的写法导出，不带参数、不读写别的内存
    let answer = unsafe { singlefs_model_answer() };
    println!("{small} {answer}");
}
RUST
}

# 去掉 build 那一块：改前的树三步都过；两份补丁各打一种只在 cargo build 才暴露的错，clippy 与挑出的测试目标照过、build 红
postmono_model() {
  local base="$WORK/poly" variant
  write_poly_tree "$base/before" 64 1
  write_poly_tree "$base/after-postmono" 8 1
  write_poly_tree "$base/after-link" 64 0
  make_patch "$base/patch-postmono" crates/poly-core/src/lib.rs "$base/before/crates/poly-core/src/lib.rs" "$base/after-postmono/crates/poly-core/src/lib.rs"
  make_patch "$base/patch-link" crates/poly-core/src/lib.rs "$base/before/crates/poly-core/src/lib.rs" "$base/after-link/crates/poly-core/src/lib.rs"
  items_check poly_items_postmono "$base/after-postmono" "$base/patch-postmono" "poly-core:src/lib.rs|--lib"
  items_check poly_items_link "$base/after-link" "$base/patch-link" "poly-core:src/lib.rs|--lib"
  for variant in before after-postmono after-link; do
    run_in "$base/$variant" "poly_${variant}_clippy" zero cargo clippy --offline --all-targets -- -D warnings
    run_in "$base/$variant" "poly_${variant}_picked_test" zero cargo test --offline --no-fail-fast -p poly-core --lib
    # 改法臂（只在这个模型上量、被攻过零轮）：build 照旧跑、调试信息整个关掉（CARGO_PROFILE_DEV_DEBUG=0），看它照不照样红
    if [[ "$variant" == before ]]; then
      run_in "$base/$variant" "poly_${variant}_build" zero cargo build --offline --all-targets -p poly-core -p poly-user
      run_in "$base/$variant" "poly_${variant}_build_without_debuginfo" zero env CARGO_PROFILE_DEV_DEBUG=0 cargo build --offline --all-targets -p poly-core -p poly-user
    else
      run_in "$base/$variant" "poly_${variant}_build" nonzero cargo build --offline --all-targets -p poly-core -p poly-user
      run_in "$base/$variant" "poly_${variant}_build_without_debuginfo" nonzero env CARGO_PROFILE_DEV_DEBUG=0 cargo build --offline --all-targets -p poly-core -p poly-user
    fi
  done
}

# alpha_manifest <before|binpath|shadow>：alpha 的 Cargo.toml 正文
alpha_manifest() {
  local bin_path="src/bin/tool.rs"
  [[ "$1" != binpath ]] || bin_path="bins/tool_next.rs"
  printf '[package]\nname = "alpha"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n\n[[bin]]\nname = "tool"\npath = "%s"\n' "$bin_path"
  if [[ "$1" == shadow ]]; then printf '\n[[test]]\nname = "beta"\npath = "checks/other.rs"\n'; fi
}

# write_alpha_tree <树> <before|binpath|shadow>：alpha 包；三种只差 Cargo.toml，别的文件三种都一样（补丁只改 Cargo.toml）
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

# list_check <树> <名> <要不要在列表里：present|absent> <测试名> <cargo test 的参数…>：cargo test … -- --list 里有没有这条测试
list_check() {
  local tree="$1" name="$2" wanted="$3" test_name="$4" rc hits verdict=mismatched
  shift 4
  if (cd "$tree" && CARGO_TARGET_DIR="$TARGET_ROOT/${tree#"$WORK"/}" cargo test --offline "$@" -- --list) > "$WORK/logs/$name.log" 2>&1; then rc=0; else rc=$?; fi
  hits="$(grep -c "^${test_name}: test" "$WORK/logs/$name.log" || true)"
  if [[ "$rc" -eq 0 ]] && { [[ "$wanted" == present && "$hits" -ge 1 ]] || [[ "$wanted" == absent && "$hits" -eq 0 ]]; }; then verdict=matched; fi
  if [[ "$verdict" == matched ]]; then MATCHES=$((MATCHES + 1)); else MISMATCHES=$((MISMATCHES + 1)); fi
  echo "name=$name exit=$rc test=$test_name hits=$hits expected=$wanted verdict=$verdict"
}

# 挑测试目标那一块：补丁只改 alpha 的 Cargo.toml
#   binpath：[[bin]] tool 的 path 换成另一份已有的文件。被判的程序只挑 --bin tool；起 tool 的集成测试 tests/cli.rs 没挑上，真 cargo 下它由绿转红。
#   shadow：新加 [[test]] name = "beta" path = "checks/other.rs"。cargo 按名字不再自动认 tests/beta.rs，那份里的测试从此不跑；
#     被判的程序只挑 checks/other.rs（--test beta），不打一句 tests/beta.rs 没了。
manifest_model() {
  local base="$WORK/alpha" variant
  for variant in before binpath shadow; do write_alpha_tree "$base/$variant" "$variant"; done
  make_patch "$base/patch-binpath" crates/alpha/Cargo.toml "$base/before/crates/alpha/Cargo.toml" "$base/binpath/crates/alpha/Cargo.toml"
  make_patch "$base/patch-shadow" crates/alpha/Cargo.toml "$base/before/crates/alpha/Cargo.toml" "$base/shadow/crates/alpha/Cargo.toml"
  items_check alpha_items_binpath "$base/binpath" "$base/patch-binpath" "alpha:bins/tool_next.rs|--bin tool"
  items_check alpha_items_shadow "$base/shadow" "$base/patch-shadow" "alpha:checks/other.rs|--test beta"
  echo "name=alpha_items_shadow_mentions_tests_beta count=$(grep -c 'tests/beta.rs' "$WORK/logs/alpha_items_shadow.txt" || true)"
  run_in "$base/binpath" alpha_binpath_picked_test zero cargo test --offline --no-fail-fast -p alpha --bin tool
  run_in "$base/before" alpha_before_cli_test zero cargo test --offline --no-fail-fast -p alpha --test cli
  run_in "$base/binpath" alpha_binpath_cli_test nonzero cargo test --offline --no-fail-fast -p alpha --test cli
  list_check "$base/before" alpha_before_list present beta_guards_greeting -p alpha
  list_check "$base/shadow" alpha_shadow_list absent beta_guards_greeting -p alpha
  list_check "$base/shadow" alpha_shadow_picked_list present other_check -p alpha --test beta
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
  TARGET_ROOT="${CARGO_TARGET_DIR:-$WORK/target}"   # 每棵玩具树各用它底下一个编译目录：几棵树里同名的 bin 不共用 target/debug/ 下那一份
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK target_root=$TARGET_ROOT"
  echo "name=judged_inputs merge_and_verify=$got rerun_failed_tests=$(sha256sum "$FROZEN/scripts/gate/rerun-failed-tests.py" | cut -d' ' -f1) lib_rebuilt_packages=$(sha256sum "$FROZEN/scripts/gate/lib_rebuilt_packages.py" | cut -d' ' -f1)"
  echo "name=toolchain rustc=$(rustc --version | tr ' ' '_') cargo=$(cargo --version | tr ' ' '_') clippy=$(cargo clippy --version 2>&1 | tr ' ' '_')"
  extract_items_program "$judged" "$WORK/items_program.py"
  postmono_model
  manifest_model
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
