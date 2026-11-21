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

# ── 这一腿要跑的步骤：三轮攻方腿（merge-verify-disk-governance-r3）──
# 被判的是 scripts/gate/merge-and-verify.sh 这一轮的修补：两侧目标集合按根文件比（shadowed_automatic_targets）与算包认 { workspace = true }（path_dependencies）。
# 每一组铺一棵玩具树，补丁只改一份文件；被判的程序原样从被判脚本里取（挑目标那一段 PY_ITEMS、算包那一段 PY_MAIN_TREE_FINGERPRINT），
# cargo 的真值用 cargo metadata（离线、不编）列目标与依赖解析，目录形那一组另 cargo test --list 一次（要编，玩具大小）。
# 两条臂：root（被判那一版，不设开关）与 by_name（弄坏开关 manifest-targets-compared-by-name，即按（段，名字）比的改前写法）；算包另有 inherit_ignored 一臂。
# 每行 name=<格> … verdict=matched 表示读数与跑前写下的预期一致（预期写在每格上面的注释里），mismatched 表示不一致。
JUDGED_SCRIPT_SHA256="5dbf51739d699464addf4a105aa12d9fdb1952c348ea5e4bd69eeaa0baf7210e"
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

# verdict_if <名> <yes|no> <别的字段>：第二个参数是 yes 记 matched，否则 mismatched
verdict_if() {
  if [[ "$2" == yes ]]; then verdict_line "$1" matched "$3"; else verdict_line "$1" mismatched "$3"; fi
}

# make_patch <补丁目录> <树根起的相对路径> <旧文件> <新文件>：写一份只改这一个文件的 crates.patch（git 补丁头，a/ b/ 前缀）
make_patch() {
  mkdir -p "$1"
  { printf 'diff --git a/%s b/%s\n--- a/%s\n+++ b/%s\n' "$2" "$2" "$2" "$2"
    { git diff --no-index --no-color --no-ext-diff "$3" "$4" || true; } | tail -n +5; } > "$1/crates.patch"
}

# extract_program <被判脚本> <here 文档的起头行（整行）> <收尾词> <输出>：原样取出一段 python
extract_program() {
  local script="$1" output="$4" start end
  start="$(grep -nxF -- "$2" "$script" | cut -d: -f1)"
  end="$(grep -nxF -- "$3" "$script" | cut -d: -f1)"
  [[ -n "$start" && -n "$end" && "$end" -gt "$start" ]] || { echo "  ✗ 在 $script 里找不到 $3 那一段 python（start=$start end=$end）" >&2; return 2; }
  sed -n "$((start + 1)),$((end - 1))p" "$script" > "$output"
  echo "name=program_$3 lines=$((end - start - 1)) sha256=$(sha256sum "$output" | cut -d' ' -f1)"
}

# items_run <名> <树> <补丁目录> <弄坏开关或空串>：起被判的挑目标程序，输出进 $WORK/logs/<名>.txt，退出码进 LAST_RC，每一行原样转打
items_run() {
  local line
  if python3 "$WORK/items_program.py" targets "$2" "$FROZEN/scripts/gate/rerun-failed-tests.py" "$FROZEN/scripts/gate/lib_rebuilt_packages.py" "$4" "$3" \
      > "$WORK/logs/$1.txt" 2>&1; then LAST_RC=0; else LAST_RC=$?; fi
  while IFS= read -r line; do echo "name=${1}_line text=${line//$'\t'/ | }"; done < "$WORK/logs/$1.txt"
}

# packages_run <名> <树> <补丁目录> <弄坏开关或空串>：起被判的算包程序（build-packages），包名用空格连成一串放进 PACKAGES
packages_run() {
  if env MERGE_AND_VERIFY_BREAK="$4" python3 - "$FROZEN/scripts/gate/lib_rebuilt_packages.py" "$2" build-packages "$3" < "$WORK/fingerprint_program.py" \
      > "$WORK/logs/$1.txt" 2>&1; then LAST_RC=0; else LAST_RC=$?; fi
  PACKAGES="$(tr '\n' ' ' < "$WORK/logs/$1.txt" | sed 's/ $//')"
}

# count_lines <模式> <文件>：grep -cE 的数（0 命中照打 0）
count_lines() {
  local found
  found="$(grep -cE -- "$1" "$2" || true)"
  echo "${found:-0}"
}

# cargo_targets <名> <树>：cargo metadata 列 alpha 的目标，一行「段 名 根文件」，进 $WORK/logs/<名>.cargo.txt；TARGETS 是用分号连起来的一串
cargo_targets() {
  if (cd "$2" && cargo metadata --offline --no-deps --format-version 1) > "$WORK/logs/$1.cargo.json" 2> "$WORK/logs/$1.cargo.err"; then LAST_RC=0; else LAST_RC=$?; fi
  python3 -c 'import json,sys
m = json.load(open(sys.argv[1]))
for p in m["packages"]:
    for t in p["targets"]:
        print(t["kind"][0], t["name"], t["src_path"].split("/crates/" + p["name"] + "/", 1)[1])' "$WORK/logs/$1.cargo.json" > "$WORK/logs/$1.cargo.txt" 2>> "$WORK/logs/$1.cargo.err" || true
  TARGETS="$(tr '\n' ';' < "$WORK/logs/$1.cargo.txt" | sed 's/;$//; s/ /:/g')"
}

# cargo_dependents <名> <树> <包>：cargo metadata（带依赖、离线）解析图里直接依赖 <包> 的包名，空格连成一串放进 DEPENDENTS
cargo_dependents() {
  if (cd "$2" && cargo metadata --offline --format-version 1) > "$WORK/logs/$1.cargo.json" 2> "$WORK/logs/$1.cargo.err"; then LAST_RC=0; else LAST_RC=$?; fi
  DEPENDENTS="$(python3 -c 'import json,sys
m = json.load(open(sys.argv[1])); ids = {p["id"]: p for p in m["packages"]}
print(" ".join(sorted(ids[n["id"]]["name"] + "@" + ("path" if ids[x]["source"] is None else "registry") for n in m["resolve"]["nodes"] for x in n["dependencies"] if ids[x]["name"] == sys.argv[2])))' "$WORK/logs/$1.cargo.json" "$3" 2>> "$WORK/logs/$1.cargo.err" || true)"
}

# alpha_tree <树> <旧 Cargo.toml 正文（printf %b）> <新 Cargo.toml 正文（printf %b）>：包 alpha，src/lib.rs、tests/a.rs、tests/b.rs；补丁只改 Cargo.toml
alpha_tree() {
  mkdir -p "$1/crates/alpha/src" "$1/crates/alpha/tests" "$1.old"
  printf '[workspace]\nmembers = ["crates/alpha"]\nresolver = "2"\n' > "$1/Cargo.toml"
  printf 'pub fn f() {}\n' > "$1/crates/alpha/src/lib.rs"
  printf '#[test]\nfn a_runs() {}\n' > "$1/crates/alpha/tests/a.rs"
  printf '#[test]\nfn b_runs() {}\n' > "$1/crates/alpha/tests/b.rs"
  printf '%b' "$2" > "$1.old/Cargo.toml"
  printf '%b' "$3" > "$1/crates/alpha/Cargo.toml"
}

# items_summary <名>：REFUSE 行数、ITEM 的「项|挑法」用分号连起来，放进 REFUSALS、ITEMS
items_summary() {
  REFUSALS="$(count_lines '^REFUSE' "$WORK/logs/$1.txt")"
  ITEMS="$(awk -F'\t' '$1 == "ITEM" { printf "%s%s|%s", (n++ ? ";" : ""), $2, $4 }' "$WORK/logs/$1.txt" | tr ' ' ':')"
}

PACKAGE_2021='[package]\nname = "alpha"\nversion = "0.1.0"\nedition = "2021"\n'
PACKAGE_2015='[package]\nname = "alpha"\nversion = "0.1.0"\n'

# dot_path：补丁给自动认的 tests/a.rs 新加一段同名 [[test]]，path 换一种写法（./tests/a.rs、tests/./a.rs），段里另写一个键（harness = true 或 test = true，由写补丁的人定，两样都扫）。
#   预期（被判程序）：两条臂都退 3 一行，点名 test 目标 a 被顶掉（root 臂：新侧根文件串是 ./tests/a.rs，与 tests/a.rs 不是同一个串）。
#   预期（cargo）：test a 的根文件照旧是 tests/a.rs（cargo 把 path 归一），tests/a.rs 照跑，没有被顶掉。
dot_path() {
  local spell spelled body key tree arm shadow
  for spell in dot_prefix dot_inner; do
    if [[ "$spell" == dot_prefix ]]; then spelled="./tests/a.rs"; else spelled="tests/./a.rs"; fi
    for body in harness test_true; do
      if [[ "$body" == harness ]]; then key='harness = true'; else key='test = true'; fi
      tree="$WORK/dot-$spell-$body"
      alpha_tree "$tree" "$PACKAGE_2021" "$PACKAGE_2021\n[[test]]\nname = \"a\"\npath = \"$spelled\"\n$key\n"
      make_patch "$WORK/patch-dot-$spell-$body" crates/alpha/Cargo.toml "$tree.old/Cargo.toml" "$tree/crates/alpha/Cargo.toml"
      for arm in root by_name; do
        items_run "dot_${spell}_${body}_${arm}" "$tree" "$WORK/patch-dot-$spell-$body" "$([[ $arm == by_name ]] && echo manifest-targets-compared-by-name)"
        items_summary "dot_${spell}_${body}_${arm}"
        shadow="$(count_lines '^REFUSE.*自动认的目标被顶掉.*test 目标 a（自动认的，根文件 tests/a.rs）' "$WORK/logs/dot_${spell}_${body}_${arm}.txt")"
        verdict_if "dot_${spell}_${body}_${arm}" "$([[ $LAST_RC -eq 0 && $REFUSALS -eq 1 && $shadow -eq 1 ]] && echo yes || echo no)" \
          "exit=$LAST_RC refusals=$REFUSALS names_test_a_shadowed=$shadow items=${ITEMS:-none} expected=refused_as_shadowed"
      done
      cargo_targets "dot_${spell}_${body}" "$tree"
      verdict_if "dot_${spell}_${body}_cargo" "$([[ $LAST_RC -eq 0 && ";$TARGETS;" == *";test:a:tests/a.rs;"* ]] && echo yes || echo no)" \
        "exit=$LAST_RC targets=$TARGETS expected_has=test:a:tests/a.rs"
    done
  done
}

# edition_2015：包没写 edition（cargo 当 2015），tests/ 下 a、b、c 三份原都是自动认的；补丁只新加一段 [[test]] c（写 path 与不写 path 两样都扫）。
#   对照：同一份补丁打在写了 edition = "2021" 的包上。bin 那一样：src/bin/one.rs、two.rs 原都是自动认的，补丁只新加一段 [[bin]] one。
#   预期（被判程序）：2015 与 2021 两种包上输出一样：不退 3，只挑 c（或 one）。
#   预期（cargo）：2015 的包上 a、b（或 two）不再是目标（声明了一段同种的段，2015 版就不再自动认），2021 的包上照是目标。
edition_2015() {
  local edition header body path_line tree arm
  for edition in 2015 2021; do
    if [[ "$edition" == 2015 ]]; then header="$PACKAGE_2015"; else header="$PACKAGE_2021"; fi
    for body in with_path without_path; do
      if [[ "$body" == with_path ]]; then path_line='path = "tests/c.rs"\n'; else path_line=''; fi
      tree="$WORK/edition-$edition-$body"
      alpha_tree "$tree" "$header" "$header\n[[test]]\nname = \"c\"\n$path_line"
      printf '#[test]\nfn c_runs() {}\n' > "$tree/crates/alpha/tests/c.rs"
      make_patch "$WORK/patch-edition-$edition-$body" crates/alpha/Cargo.toml "$tree.old/Cargo.toml" "$tree/crates/alpha/Cargo.toml"
      for arm in root by_name; do
        items_run "edition_${edition}_${body}_${arm}" "$tree" "$WORK/patch-edition-$edition-$body" "$([[ $arm == by_name ]] && echo manifest-targets-compared-by-name)"
        items_summary "edition_${edition}_${body}_${arm}"
        verdict_if "edition_${edition}_${body}_${arm}" "$([[ $LAST_RC -eq 0 && $REFUSALS -eq 0 && "$ITEMS" == "alpha:tests/c.rs|--test:c" ]] && echo yes || echo no)" \
          "exit=$LAST_RC refusals=$REFUSALS items=${ITEMS:-none} expected=no_refusal_items_alpha:tests/c.rs|--test:c"
      done
      cargo_targets "edition_${edition}_${body}" "$tree"
      if [[ "$edition" == 2015 ]]; then
        verdict_if "edition_${edition}_${body}_cargo" "$([[ $LAST_RC -eq 0 && "$TARGETS" == "lib:alpha:src/lib.rs;test:c:tests/c.rs" ]] && echo yes || echo no)" \
          "exit=$LAST_RC targets=$TARGETS expected=lib:alpha:src/lib.rs;test:c:tests/c.rs"
      else
        verdict_if "edition_${edition}_${body}_cargo" "$([[ $LAST_RC -eq 0 && "$TARGETS" == "lib:alpha:src/lib.rs;test:a:tests/a.rs;test:b:tests/b.rs;test:c:tests/c.rs" ]] && echo yes || echo no)" \
          "exit=$LAST_RC targets=$TARGETS expected=lib:alpha:src/lib.rs;test:a:tests/a.rs;test:b:tests/b.rs;test:c:tests/c.rs"
      fi
    done
  done
  tree="$WORK/edition-2015-bin"
  alpha_tree "$tree" "$PACKAGE_2015" "$PACKAGE_2015\n[[bin]]\nname = \"one\"\npath = \"src/bin/one.rs\"\n"
  mkdir -p "$tree/crates/alpha/src/bin"
  printf 'fn main() {}\n' > "$tree/crates/alpha/src/bin/one.rs"
  printf 'fn main() {}\n#[test]\nfn two_runs() {}\n' > "$tree/crates/alpha/src/bin/two.rs"
  make_patch "$WORK/patch-edition-2015-bin" crates/alpha/Cargo.toml "$tree.old/Cargo.toml" "$tree/crates/alpha/Cargo.toml"
  items_run edition_2015_bin_root "$tree" "$WORK/patch-edition-2015-bin" ""
  items_summary edition_2015_bin_root
  verdict_if edition_2015_bin_root "$([[ $LAST_RC -eq 0 && $REFUSALS -eq 0 && "$ITEMS" == "alpha:src/bin/one.rs|--bin:one" ]] && echo yes || echo no)" \
    "exit=$LAST_RC refusals=$REFUSALS items=${ITEMS:-none} expected=no_refusal_items_alpha:src/bin/one.rs|--bin:one"
  cargo_targets edition_2015_bin "$tree"
  verdict_if edition_2015_bin_cargo "$([[ $LAST_RC -eq 0 && ";$TARGETS;" != *";bin:two:"* && ";$TARGETS;" == *";bin:one:src/bin/one.rs;"* ]] && echo yes || echo no)" \
    "exit=$LAST_RC targets=$TARGETS expected=bin_one_present_bin_two_absent"
}

# dir_form：tests/d/main.rs 原是自动认的目标 d；补丁只新加一段不写 path 的 [[test]] d（段里另写 harness = true 或 test = true，两样都扫）。
#   bin 那一样：src/bin/tool/main.rs 原是自动认的 bin tool，补丁只新加一段不写 path 的 [[bin]] tool（test = true）。
#   预期（被判程序）：退 3 一行「[[test]] 段 d 落不到测试目标」（或 [[bin]] 段 tool），一项都不挑（目标清单那一份只拿 tests/d.rs 当缺省根文件）。
#   预期（cargo）：d 的根文件是 tests/d/main.rs（tool 是 src/bin/tool/main.rs），照是目标；cargo test --test d -- --list 列得出 d_runs。
dir_form() {
  local body key tree arm landed
  for body in harness test_true; do
    if [[ "$body" == harness ]]; then key='harness = true'; else key='test = true'; fi
    tree="$WORK/dirform-$body"
    alpha_tree "$tree" "$PACKAGE_2021" "$PACKAGE_2021\n[[test]]\nname = \"d\"\n$key\n"
    mkdir -p "$tree/crates/alpha/tests/d"
    printf '#[test]\nfn d_runs() {}\n' > "$tree/crates/alpha/tests/d/main.rs"
    make_patch "$WORK/patch-dirform-$body" crates/alpha/Cargo.toml "$tree.old/Cargo.toml" "$tree/crates/alpha/Cargo.toml"
    for arm in root by_name; do
      items_run "dirform_${body}_${arm}" "$tree" "$WORK/patch-dirform-$body" "$([[ $arm == by_name ]] && echo manifest-targets-compared-by-name)"
      items_summary "dirform_${body}_${arm}"
      landed="$(count_lines '^REFUSE.*\[\[test\]\] 段 d 落不到测试目标' "$WORK/logs/dirform_${body}_${arm}.txt")"
      verdict_if "dirform_${body}_${arm}" "$([[ $LAST_RC -eq 0 && $REFUSALS -eq 1 && $landed -eq 1 && -z "$ITEMS" ]] && echo yes || echo no)" \
        "exit=$LAST_RC refusals=$REFUSALS refused_unlanded_d=$landed items=${ITEMS:-none} expected=refused_unlanded"
    done
    cargo_targets "dirform_${body}" "$tree"
    verdict_if "dirform_${body}_cargo" "$([[ $LAST_RC -eq 0 && ";$TARGETS;" == *";test:d:tests/d/main.rs;"* ]] && echo yes || echo no)" \
      "exit=$LAST_RC targets=$TARGETS expected_has=test:d:tests/d/main.rs"
  done
  tree="$WORK/dirform-harness"
  if (cd "$tree" && CARGO_TARGET_DIR="$TARGET_ROOT/dirform" cargo test --offline -p alpha --test d -- --list) > "$WORK/logs/dirform_cargo_list.log" 2>&1; then LAST_RC=0; else LAST_RC=$?; fi
  landed="$(count_lines '^d_runs: test$' "$WORK/logs/dirform_cargo_list.log")"
  verdict_if dirform_cargo_list "$([[ $LAST_RC -eq 0 && $landed -eq 1 ]] && echo yes || echo no)" "exit=$LAST_RC hits=$landed expected=1"

  tree="$WORK/dirform-bin"
  alpha_tree "$tree" "$PACKAGE_2021" "$PACKAGE_2021\n[[bin]]\nname = \"tool\"\ntest = true\n"
  mkdir -p "$tree/crates/alpha/src/bin/tool"
  printf 'fn main() {}\n#[test]\nfn tool_runs() {}\n' > "$tree/crates/alpha/src/bin/tool/main.rs"
  make_patch "$WORK/patch-dirform-bin" crates/alpha/Cargo.toml "$tree.old/Cargo.toml" "$tree/crates/alpha/Cargo.toml"
  items_run dirform_bin_root "$tree" "$WORK/patch-dirform-bin" ""
  items_summary dirform_bin_root
  landed="$(count_lines '^REFUSE.*\[\[bin\]\] 段 tool 落不到测试目标' "$WORK/logs/dirform_bin_root.txt")"
  verdict_if dirform_bin_root "$([[ $LAST_RC -eq 0 && $REFUSALS -eq 1 && $landed -eq 1 ]] && echo yes || echo no)" \
    "exit=$LAST_RC refusals=$REFUSALS refused_unlanded_tool=$landed items=${ITEMS:-none} expected=refused_unlanded"
  cargo_targets dirform_bin "$tree"
  verdict_if dirform_bin_cargo "$([[ $LAST_RC -eq 0 && ";$TARGETS;" == *";bin:tool:src/bin/tool/main.rs;"* ]] && echo yes || echo no)" \
    "exit=$LAST_RC targets=$TARGETS expected_has=bin:tool:src/bin/tool/main.rs"

  # 段已在（两侧都写了不带 path 的 [[test]] d），补丁只改 tests/d/main.rs 的正文：预期被判程序照样退 3（判不出），cargo 那边 d 照是目标
  tree="$WORK/dirform-body"
  alpha_tree "$tree" "$PACKAGE_2021\n[[test]]\nname = \"d\"\n" "$PACKAGE_2021\n[[test]]\nname = \"d\"\n"
  mkdir -p "$tree/crates/alpha/tests/d"
  printf '#[test]\nfn d_runs() {}\n' > "$tree.old/main.rs"
  printf '#[test]\nfn d_runs() {\n    assert_eq!(1 + 1, 2);\n}\n' > "$tree/crates/alpha/tests/d/main.rs"
  make_patch "$WORK/patch-dirform-body" crates/alpha/tests/d/main.rs "$tree.old/main.rs" "$tree/crates/alpha/tests/d/main.rs"
  items_run dirform_body_root "$tree" "$WORK/patch-dirform-body" ""
  items_summary dirform_body_root
  verdict_if dirform_body_root "$([[ $LAST_RC -eq 0 && $REFUSALS -eq 1 && -z "$ITEMS" ]] && echo yes || echo no)" \
    "exit=$LAST_RC refusals=$REFUSALS items=${ITEMS:-none} expected=refused_no_items"
}

# collide（对照）：补丁新加一段 [[test]] b，path 指到自动认的 tests/a.rs。
#   预期（被判程序）：退 3 一行，点名 test 目标 b（根文件 tests/b.rs）被顶掉，不点名 a。预期（cargo）：只剩 test b（根文件 tests/a.rs），tests/b.rs 不再是目标。
# switch（对照）：补丁新加一段 [[bin]] a，path 指到自动认的 tests/a.rs（同一份根文件换到另一种段）。
#   预期（被判程序）：不退 3，只挑 alpha:tests/a.rs|--bin a（目标清单按根文件做键，同一根文件的 test a 被 bin a 占了键）。预期（cargo）：bin a 与 test a 都在。
collide_and_switch() {
  local tree shadow_b shadow_a
  tree="$WORK/collide"
  alpha_tree "$tree" "$PACKAGE_2021" "$PACKAGE_2021\n[[test]]\nname = \"b\"\npath = \"tests/a.rs\"\n"
  make_patch "$WORK/patch-collide" crates/alpha/Cargo.toml "$tree.old/Cargo.toml" "$tree/crates/alpha/Cargo.toml"
  items_run collide_root "$tree" "$WORK/patch-collide" ""
  items_summary collide_root
  shadow_b="$(count_lines '^REFUSE.*自动认的目标被顶掉.*test 目标 b（自动认的，根文件 tests/b.rs）' "$WORK/logs/collide_root.txt")"
  shadow_a="$(count_lines '^REFUSE.*test 目标 a（' "$WORK/logs/collide_root.txt")"
  verdict_if collide_root "$([[ $LAST_RC -eq 0 && $REFUSALS -eq 1 && $shadow_b -eq 1 && $shadow_a -eq 0 ]] && echo yes || echo no)" \
    "exit=$LAST_RC refusals=$REFUSALS names_b=$shadow_b names_a=$shadow_a expected=refused_naming_b_only"
  cargo_targets collide "$tree"
  verdict_if collide_cargo "$([[ $LAST_RC -eq 0 && "$TARGETS" == "lib:alpha:src/lib.rs;test:b:tests/a.rs" ]] && echo yes || echo no)" \
    "exit=$LAST_RC targets=$TARGETS expected=lib:alpha:src/lib.rs;test:b:tests/a.rs"

  tree="$WORK/switch"
  alpha_tree "$tree" "$PACKAGE_2021" "$PACKAGE_2021\n[[bin]]\nname = \"a\"\npath = \"tests/a.rs\"\n"
  make_patch "$WORK/patch-switch" crates/alpha/Cargo.toml "$tree.old/Cargo.toml" "$tree/crates/alpha/Cargo.toml"
  items_run switch_root "$tree" "$WORK/patch-switch" ""
  items_summary switch_root
  verdict_if switch_root "$([[ $LAST_RC -eq 0 && $REFUSALS -eq 0 && "$ITEMS" == "alpha:tests/a.rs|--bin:a" ]] && echo yes || echo no)" \
    "exit=$LAST_RC refusals=$REFUSALS items=${ITEMS:-none} expected=alpha:tests/a.rs|--bin:a"
  cargo_targets switch "$tree"
  verdict_if switch_cargo "$([[ $LAST_RC -eq 0 && ";$TARGETS;" == *";bin:a:tests/a.rs;"* && ";$TARGETS;" == *";test:a:tests/a.rs;"* ]] && echo yes || echo no)" \
    "exit=$LAST_RC targets=$TARGETS expected_has=bin:a:tests/a.rs+test:a:tests/a.rs"
}

# debuginfo_breakdown：第二轮判决 5 的玩具（三包 dbg-leaf ← dbg-core ← dbg-user，根清单 [profile.dev] debug = "line-tables-only"，照第二轮攻方模型的写法重铺），
#   照合入后验证的先后：先 cargo test -p dbg-core --lib，再 cargo build --all-targets -p dbg-core -p dbg-user；off 臂 build 设 CARGO_PROFILE_DEV_DEBUG=0，kept 臂不设。
#   把编译目录拆成可执行、rlib 与 rmeta、incremental、别的四块（字节，find 的文件大小相加；硬链接按 deps/ 里那一份算一次），另读 dbg-user 可执行里 .debug_* 段的字节数。
#   跑前写下的预期（前一条攻方腿推的那一句）：两臂可执行文件的差额大于两臂 rlib 与 rmeta 的差额，即 target_kib 的差主要来自可执行文件、不来自依赖另编的那一份。
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

# bytes_of <find 的条件…>：在 $BREAKDOWN_DIRECTORY 下按条件找普通文件、把大小相加（字节）
bytes_of() {
  find "$BREAKDOWN_DIRECTORY" -type f "$@" -printf '%s\n' | awk '{ n += $1 } END { print n + 0 }'
}

debuginfo_arm() {
  local arm="$1" tree="$WORK/debuginfo-tree" target="$TARGET_ROOT/debuginfo-$1" test_rc build_rc executable debug_bytes
  if (cd "$tree" && CARGO_TARGET_DIR="$target" cargo test --offline -p dbg-core --lib) > "$WORK/logs/debuginfo_${arm}_test.log" 2>&1; then test_rc=0; else test_rc=$?; fi
  if [[ "$arm" == off ]]; then
    if (cd "$tree" && CARGO_TARGET_DIR="$target" env CARGO_PROFILE_DEV_DEBUG=0 cargo build --offline --all-targets -p dbg-core -p dbg-user) > "$WORK/logs/debuginfo_${arm}_build.log" 2>&1; then build_rc=0; else build_rc=$?; fi
  else
    if (cd "$tree" && CARGO_TARGET_DIR="$target" cargo build --offline --all-targets -p dbg-core -p dbg-user) > "$WORK/logs/debuginfo_${arm}_build.log" 2>&1; then build_rc=0; else build_rc=$?; fi
  fi
  BREAKDOWN_DIRECTORY="$target/debug/deps"
  EXECUTABLE_BYTES="$(bytes_of -perm -u+x ! -name '*.so' ! -name '*.d' ! -name '*.rlib' ! -name '*.rmeta')"
  LIBRARY_BYTES="$(bytes_of \( -name '*.rlib' -o -name '*.rmeta' \))"
  BREAKDOWN_DIRECTORY="$target/debug/incremental"
  INCREMENTAL_BYTES="$( [[ -d "$BREAKDOWN_DIRECTORY" ]] && bytes_of || echo 0)"
  TOTAL_KIB="$(du -sk "$target" | cut -f1)"
  executable="$( { find "$target/debug/deps" -maxdepth 1 -type f -perm -u+x -name 'dbg_user-*' ! -name '*.d' || true; } | head -n 1)"
  debug_bytes="$( [[ -n "$executable" ]] && size -A "$executable" | awk '$1 ~ /^\.debug/ { n += $2 } END { print n + 0 }' || echo none)"
  echo "name=debuginfo_${arm}_breakdown test_exit=$test_rc build_exit=$build_rc target_kib=$TOTAL_KIB executable_bytes=$EXECUTABLE_BYTES rlib_rmeta_bytes=$LIBRARY_BYTES incremental_bytes=$INCREMENTAL_BYTES dbg_user_debug_section_bytes=$debug_bytes leaf_rlibs=$( { find "$target/debug/deps" -maxdepth 1 -name 'libdbg_leaf-*.rlib' || true; } | wc -l)"
  if [[ "$arm" == off ]]; then OFF_EXECUTABLE="$EXECUTABLE_BYTES"; OFF_LIBRARY="$LIBRARY_BYTES"; else KEPT_EXECUTABLE="$EXECUTABLE_BYTES"; KEPT_LIBRARY="$LIBRARY_BYTES"; fi
}

debuginfo_breakdown() {
  local executable_gap library_gap
  write_debuginfo_tree "$WORK/debuginfo-tree"
  debuginfo_arm off
  debuginfo_arm kept
  executable_gap=$((KEPT_EXECUTABLE - OFF_EXECUTABLE))
  library_gap=$((OFF_LIBRARY - KEPT_LIBRARY))
  verdict_if debuginfo_gap_source "$([[ $executable_gap -gt $library_gap ]] && echo yes || echo no)" \
    "executable_bytes_saved_by_off=$executable_gap rlib_rmeta_bytes_added_by_off=$library_gap expected=executable_gap_greater"
}

# workspace_forms：工作区 core 与五个依赖它的包，依赖都写成继承：user 写 [dependencies.core] workspace = true，devuser 写在 [dev-dependencies]，
#   builduser 写 [build-dependencies] core.workspace = true，cfguser 写在 [target.'cfg(unix)'.dependencies]，alias 写 core-renamed = { workspace = true }
#   （树根 [workspace.dependencies] 的 core-renamed 带 package = "core" 改名）；bystander 不依赖谁。补丁只改 core 的 src/lib.rs。
#   预期（被判程序 build-packages）：alias builduser cfguser core devuser user（不含 bystander）；弄坏开关 workspace-dependencies-ignored 那一臂只剩 core。
#   预期（cargo）：解析图里直接依赖 core 的是这五个包，都是 path 来源。
workspace_forms() {
  local tree="$WORK/workspace-forms" member
  for member in core user devuser builduser cfguser alias bystander; do
    mkdir -p "$tree/crates/$member/src"; printf 'pub fn f() {}\n' > "$tree/crates/$member/src/lib.rs"
  done
  mkdir -p "$tree.old"
  printf '[workspace]\nmembers = ["crates/*"]\nresolver = "2"\n\n[workspace.dependencies]\ncore = { path = "crates/core" }\ncore-renamed = { path = "crates/core", package = "core" }\n' > "$tree/Cargo.toml"
  printf '[package]\nname = "core"\nversion = "0.1.0"\nedition = "2021"\n' > "$tree/crates/core/Cargo.toml"
  printf '[package]\nname = "bystander"\nversion = "0.1.0"\nedition = "2021"\n' > "$tree/crates/bystander/Cargo.toml"
  printf '[package]\nname = "user"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies.core]\nworkspace = true\n' > "$tree/crates/user/Cargo.toml"
  printf '[package]\nname = "devuser"\nversion = "0.1.0"\nedition = "2021"\n\n[dev-dependencies]\ncore = { workspace = true }\n' > "$tree/crates/devuser/Cargo.toml"
  printf '[package]\nname = "builduser"\nversion = "0.1.0"\nedition = "2021"\n\n[build-dependencies]\ncore.workspace = true\n' > "$tree/crates/builduser/Cargo.toml"
  printf '[package]\nname = "cfguser"\nversion = "0.1.0"\nedition = "2021"\n\n[target.%s.dependencies]\ncore = { workspace = true }\n' "'cfg(unix)'" > "$tree/crates/cfguser/Cargo.toml"
  printf '[package]\nname = "alias"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\ncore-renamed = { workspace = true }\n' > "$tree/crates/alias/Cargo.toml"
  cp "$tree/crates/core/src/lib.rs" "$tree.old/lib.rs"
  printf 'pub fn f() {}\npub fn g() {}\n' > "$tree/crates/core/src/lib.rs"
  make_patch "$WORK/patch-workspace-forms" crates/core/src/lib.rs "$tree.old/lib.rs" "$tree/crates/core/src/lib.rs"
  packages_run workspace_forms_root "$tree" "$WORK/patch-workspace-forms" ""
  verdict_if workspace_forms_root "$([[ $LAST_RC -eq 0 && "$PACKAGES" == "alias builduser cfguser core devuser user" ]] && echo yes || echo no)" \
    "exit=$LAST_RC packages=${PACKAGES// /,} expected=alias,builduser,cfguser,core,devuser,user"
  packages_run workspace_forms_ignored "$tree" "$WORK/patch-workspace-forms" workspace-dependencies-ignored
  verdict_if workspace_forms_ignored "$([[ $LAST_RC -eq 0 && "$PACKAGES" == "core" ]] && echo yes || echo no)" "exit=$LAST_RC packages=${PACKAGES// /,} expected=core"
  cargo_dependents workspace_forms "$tree" core
  verdict_if workspace_forms_cargo "$([[ $LAST_RC -eq 0 && "$DEPENDENTS" == "alias@path builduser@path cfguser@path devuser@path user@path" ]] && echo yes || echo no)" \
    "exit=$LAST_RC dependents=${DEPENDENTS// /,} expected=alias,builduser,cfguser,devuser,user_all_path"
}

# patch_override：工作区 wcore 与 user；user 依赖 wcore 写成 registry 版本（继承那一样：树根 [workspace.dependencies] wcore = "0.1"、user 写 wcore = { workspace = true }；
#   直写那一样：user 写 wcore = "0.1"），树根 [patch.crates-io] wcore = { path = "crates/wcore" } 把它换成工作区里的那一份。补丁只改 wcore 的 src/lib.rs。
#   预期（被判程序 build-packages）：两样都只算出 wcore（[patch] 不认）。预期（cargo）：两样里 user 都解析到 path 来源的 wcore。
patch_override() {
  local form tree
  for form in inherit explicit; do
    tree="$WORK/patch-override-$form"
    mkdir -p "$tree/crates/wcore/src" "$tree/crates/user/src" "$tree.old"
    printf 'pub fn f() {}\n' > "$tree.old/lib.rs"
    printf 'pub fn f() {}\npub fn g() {}\n' > "$tree/crates/wcore/src/lib.rs"
    printf 'pub fn h() {\n    wcore::f()\n}\n' > "$tree/crates/user/src/lib.rs"
    printf '[package]\nname = "wcore"\nversion = "0.1.0"\nedition = "2021"\n' > "$tree/crates/wcore/Cargo.toml"
    if [[ "$form" == inherit ]]; then
      printf '[workspace]\nmembers = ["crates/wcore", "crates/user"]\nresolver = "2"\n\n[workspace.dependencies]\nwcore = "0.1"\n\n[patch.crates-io]\nwcore = { path = "crates/wcore" }\n' > "$tree/Cargo.toml"
      printf '[package]\nname = "user"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nwcore = { workspace = true }\n' > "$tree/crates/user/Cargo.toml"
    else
      printf '[workspace]\nmembers = ["crates/wcore", "crates/user"]\nresolver = "2"\n\n[patch.crates-io]\nwcore = { path = "crates/wcore" }\n' > "$tree/Cargo.toml"
      printf '[package]\nname = "user"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nwcore = "0.1"\n' > "$tree/crates/user/Cargo.toml"
    fi
    make_patch "$WORK/patch-override-$form-patch" crates/wcore/src/lib.rs "$tree.old/lib.rs" "$tree/crates/wcore/src/lib.rs"
    packages_run "patch_override_${form}_root" "$tree" "$WORK/patch-override-$form-patch" ""
    verdict_if "patch_override_${form}_root" "$([[ $LAST_RC -eq 0 && "$PACKAGES" == "wcore" ]] && echo yes || echo no)" \
      "exit=$LAST_RC packages=${PACKAGES// /,} expected=wcore"
    cargo_dependents "patch_override_$form" "$tree" wcore
    verdict_if "patch_override_${form}_cargo" "$([[ $LAST_RC -eq 0 && "$DEPENDENTS" == "user@path" ]] && echo yes || echo no)" \
      "exit=$LAST_RC dependents=${DEPENDENTS// /,} expected=user@path"
  done
}

rerun_steps() {
  local actual
  unset CARGO_PROFILE_DEV_DEBUG MERGE_AND_VERIFY_BREAK
  mkdir -p "$WORK/logs"
  TARGET_ROOT="${CARGO_TARGET_DIR:-$WORK/target}"
  actual="$(sha256sum "$FROZEN/scripts/gate/merge-and-verify.sh" | cut -d' ' -f1)"
  if [[ "$actual" != "$JUDGED_SCRIPT_SHA256" ]]; then
    echo "  ✗ $FROZEN/scripts/gate/merge-and-verify.sh 的 sha256 是 $actual，不是这一轮判的那一份 $JUDGED_SCRIPT_SHA256：被判脚本换过，这一趟不跑" >&2
    return 2
  fi
  echo "name=judged_script sha256=$actual rerun_failed_tests_sha256=$(sha256sum "$FROZEN/scripts/gate/rerun-failed-tests.py" | cut -d' ' -f1) lib_rebuilt_packages_sha256=$(sha256sum "$FROZEN/scripts/gate/lib_rebuilt_packages.py" | cut -d' ' -f1)"
  echo "name=toolchain cargo=$(cargo --version | tr ' ' '_') rustc=$(rustc --version | tr ' ' '_')"
  extract_program "$FROZEN/scripts/gate/merge-and-verify.sh" "MERGE_AND_VERIFY_ITEMS_PROGRAM=\"\$(cat <<'PY_ITEMS'" PY_ITEMS "$WORK/items_program.py"
  extract_program "$FROZEN/scripts/gate/merge-and-verify.sh" "MAIN_TREE_FINGERPRINT_PROGRAM=\"\$(cat <<'PY_MAIN_TREE_FINGERPRINT'" PY_MAIN_TREE_FINGERPRINT "$WORK/fingerprint_program.py"
  echo "name=verification_tests_rs count=$(find "$FROZEN/crates/singlefs-verification/tests" -maxdepth 1 -type f -name '*.rs' | wc -l)"
  dot_path
  edition_2015
  dir_form
  collide_and_switch
  workspace_forms
  patch_override
  debuginfo_breakdown
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
