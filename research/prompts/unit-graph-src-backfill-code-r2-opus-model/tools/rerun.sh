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

# 夹具喂 Rust 与 python 两份、夹具的运行期探针、链接探针、增量探针、观察者那一组的成员 A/B。读数各打一行 name=…，原样输出在 $WORK/out。
rerun_steps() {
  local out="$WORK/out" leftovers=0
  mkdir -p "$out" "$WORK/fixtures" "$WORK/probes"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  echo "name=inherited_environment CARGO_INCREMENTAL=${CARGO_INCREMENTAL-unset} CARGO_BUILD_INCREMENTAL=${CARGO_BUILD_INCREMENTAL-unset} RUSTFLAGS=${RUSTFLAGS-unset} RUSTC_WRAPPER=${RUSTC_WRAPPER-unset}"

  # 样本进程各在自己的进程组里起（setsid），跑完按组号在 /proc 里核组里没留下进程；留下的停整组、记一行
  group_left_behind() {   # group_left_behind <组号> <哪一条>
    local group="$1" label="$2" stat_file text left=0
    local -a fields
    for stat_file in /proc/[0-9]*/stat; do
      text="$(cat "$stat_file" 2>/dev/null)" || continue   # discarded-output: 读之前进程已退出，读不到就是不在
      text="${text##*) }"
      read -r -a fields <<< "$text"
      if [[ "${fields[2]:-}" == "$group" ]]; then left=$((left + 1)); fi
    done
    if ((left)); then
      echo "name=process_group_left_behind sample=$label group=$group processes=$left"
      kill -TERM -- -"$group" 2>/dev/null || true   # discarded-output: 组里的进程可能刚退出
      leftovers=$((leftovers + 1))
    fi
  }

  # 冻结副本就是开工快照：十七份的 sha256 对不上就不往下跑
  if ! (cd "$FROZEN" && sha256sum -c "$MODEL/tools/frozen-sha256.txt") > "$out/frozen-check.txt" 2>&1; then
    echo "name=frozen_check files_ok=$(grep -c ': OK$' "$out/frozen-check.txt") verdict=mismatch"
    return 1
  fi
  echo "name=frozen_check files_ok=$(grep -c ': OK$' "$out/frozen-check.txt") verdict=ok"

  setsid --wait cargo build --manifest-path "$FROZEN/Cargo.toml" -p singlefs-unit-graph --bin unit-graph --target-dir "$WORK/target" > "$out/build-unit-graph.txt" 2>&1 &
  local sample=$! rc=0
  wait "$sample" || rc=$?
  group_left_behind "$sample" build-unit-graph
  echo "name=build_unit_graph exit=$rc"
  ((rc == 0)) || return "$rc"

  rsync -a "$MODEL/fixtures/" "$WORK/fixtures/"
  mkdir -p "$WORK/fixtures/root[with]brackets"
  rsync -a "$MODEL/fixtures/members-under-a-root-with-glob-characters/" "$WORK/fixtures/root[with]brackets/"
  mkdir -p "$WORK/fixtures/members-with-an-absolute-path" "$WORK/outside-the-root/beta/src"
  printf '[workspace]\nmembers = ["%s"]\n' "$WORK/outside-the-root/beta" > "$WORK/fixtures/members-with-an-absolute-path/Cargo.toml"
  printf '[package]\nname = "beta"\n' > "$WORK/outside-the-root/beta/Cargo.toml"
  printf 'pub fn beta_value() -> u32 {\n    1\n}\n' > "$WORK/outside-the-root/beta/src/lib.rs"

  rust_closure() {   # rust_closure <输出名> <夹具目录名> <入口> <目标>
    setsid --wait "$WORK/target/debug/unit-graph" closure "$WORK/fixtures/$2" "$3" --target "$4" --toolchain-line 'host: x86_64-unknown-linux-gnu' > "$out/$1.rust.txt" 2> "$out/$1.rust.err" &
    local sample=$! rc=0
    wait "$sample" || rc=$?
    group_left_behind "$sample" "$1.rust"
    echo "$rc" > "$out/$1.rust.rc"
  }
  python_closure() {   # python_closure <输出名> <夹具目录名> <入口> <目标，lib 写 ->
    local sample rc=0
    if [[ "$4" == - ]]; then
      setsid --wait python3 "$FROZEN/scripts/gate/lib_function_units.py" closure "$WORK/fixtures/$2" "$3" > "$out/$1.python.txt" 2> "$out/$1.python.err" &
      sample=$!
    else
      setsid --wait python3 "$FROZEN/scripts/gate/lib_function_units.py" closure "$WORK/fixtures/$2" "$3" --target "$4" > "$out/$1.python.txt" 2> "$out/$1.python.err" &
      sample=$!
    fi
    wait "$sample" || rc=$?
    group_left_behind "$sample" "$1.python"
    echo "$rc" > "$out/$1.python.rc"
  }
  closure_summary() {   # closure_summary <输出名> <rust|python> <要找的 文件:起-止>
    local file="$out/$1.$2.txt" refused found forms
    refused="$(grep -c $'^refused\t' "$file")" || refused=0
    found="$(grep -cF -- "$3" "$file")" || found=0
    forms="$(grep $'^refused\t' "$file" | cut -f2 | sort -u | paste -sd, -)" || forms=""
    echo "name=closure case=$1 implementation=$2 exit=$(cat "$out/$1.$2.rc") refused_lines=$refused refused_forms=${forms:--} looked_for=$3 found_lines=$found"
  }
  dump_pair() {   # dump_pair <夹具目录名>
    local sample rc_rust=0 rc_python=0 identical=no
    setsid --wait "$WORK/target/debug/unit-graph" dump "$WORK/fixtures/$1" > "$out/dump-$1.rust.txt" 2> "$out/dump-$1.rust.err" &
    sample=$!
    wait "$sample" || rc_rust=$?
    group_left_behind "$sample" "dump-$1.rust"
    setsid --wait python3 "$FROZEN/scripts/gate/lib_function_units.py" dump "$WORK/fixtures/$1" --language rust > "$out/dump-$1.python.txt" 2> "$out/dump-$1.python.err" &
    sample=$!
    wait "$sample" || rc_python=$?
    group_left_behind "$sample" "dump-$1.python"
    if diff "$out/dump-$1.rust.txt" "$out/dump-$1.python.txt" > "$out/dump-$1.diff"; then identical=yes; fi
    echo "name=dump fixture=$1 rust_exit=$rc_rust python_exit=$rc_python rust_lines=$(wc -l < "$out/dump-$1.rust.txt") python_lines=$(wc -l < "$out/dump-$1.python.txt") identical=$identical"
  }

  # 第 3 件：同一条 use 里的改名与相对路径、宏展开出的改名（对照：分两条 use 写，第一轮修过的那一形）
  rust_closure same-use-two-uses use-relative-path-alias-in-the-same-use fsameuse::through_two_uses::entry_point lib
  python_closure same-use-two-uses use-relative-path-alias-in-the-same-use fsameuse::through_two_uses::entry_point -
  rust_closure same-use-one-use use-relative-path-alias-in-the-same-use fsameuse::through_one_use::entry_point lib
  python_closure same-use-one-use use-relative-path-alias-in-the-same-use fsameuse::through_one_use::entry_point -
  rust_closure same-use-macro use-relative-path-alias-in-the-same-use fsameuse::through_a_macro::entry_point lib
  python_closure same-use-macro use-relative-path-alias-in-the-same-use fsameuse::through_a_macro::entry_point -
  local case implementation
  for case in same-use-two-uses same-use-one-use same-use-macro; do
    for implementation in rust python; do
      closure_summary "$case" "$implementation" crates/fsameuse/src/lib.rs:3-5
    done
  done

  # 第 2 件：工作区里与标准库宏同名的 macro_rules!（对照：没有局部绑定）
  rust_closure std-name-format shadow-through-a-macro-named-like-a-standard-one fstdname::entry_through_a_macro_named_format lib
  python_closure std-name-format shadow-through-a-macro-named-like-a-standard-one fstdname::entry_through_a_macro_named_format -
  rust_closure std-name-vec shadow-through-a-macro-named-like-a-standard-one fstdname::entry_through_a_macro_named_vec lib
  python_closure std-name-vec shadow-through-a-macro-named-like-a-standard-one fstdname::entry_through_a_macro_named_vec -
  rust_closure std-name-control shadow-through-a-macro-named-like-a-standard-one fstdname::entry_without_a_binding lib
  python_closure std-name-control shadow-through-a-macro-named-like-a-standard-one fstdname::entry_without_a_binding -
  for case in std-name-format std-name-vec std-name-control; do
    for implementation in rust python; do
      closure_summary "$case" "$implementation" crates/fstdname/src/lib.rs:14-16
    done
  done

  # 第 5 件：最终产物里别的目标、别的包、构建依赖带进来的库（对照：哪儿都没有 link）
  rust_closure extern-test-target extern-link-from-another-target-of-the-final-binary fexternfinal::fdatasync_through_the_library test:linked
  python_closure extern-test-target extern-link-from-another-target-of-the-final-binary crates/fexternfinal/tests/linked.rs::fdatasync_through_the_library fexternfinal:test:linked
  rust_closure extern-dependency extern-link-from-another-target-of-the-final-binary fexterndep::entry_point lib
  python_closure extern-dependency extern-link-from-another-target-of-the-final-binary fexterndep::entry_point -
  rust_closure extern-control extern-link-from-another-target-of-the-final-binary fexternclean::entry_point lib
  python_closure extern-control extern-link-from-another-target-of-the-final-binary fexternclean::entry_point -
  rust_closure extern-build-helper extern-link-through-a-build-dependency fbuildlink::entry_point lib
  python_closure extern-build-helper extern-link-through-a-build-dependency fbuildlink::entry_point -
  rust_closure extern-cc-crate extern-link-through-a-build-dependency fcclink::entry_point lib
  python_closure extern-cc-crate extern-link-through-a-build-dependency fcclink::entry_point -
  for case in extern-test-target extern-dependency extern-control extern-build-helper extern-cc-crate; do
    for implementation in rust python; do
      closure_summary "$case" "$implementation" extern-function-interface
    done
  done

  # 第 6 件：工作区成员两份的读法（dump 逐行比、闭包各找一份点名的单元）
  local fixture
  for fixture in members-dot-with-root-package members-dot-unnamed-root workspace-inline-empty-table members-in-a-multi-line-string \
      nested-member-inside-another-member members-sharing-a-directory-name 'root[with]brackets' members-with-an-absolute-path; do
    dump_pair "$fixture"
  done
  echo "name=nested_member_file_lines rust=$(grep -c 'crates/outer/inner/src/lib.rs' "$out/dump-nested-member-inside-another-member.rust.txt") python=$(grep -c 'crates/outer/inner/src/lib.rs' "$out/dump-nested-member-inside-another-member.python.txt")"
  rust_closure dot-named members-dot-with-root-package rootpkg::root_value lib
  python_closure dot-named members-dot-with-root-package rootpkg::root_value -
  rust_closure dot-unnamed members-dot-unnamed-root root::root_value lib
  python_closure dot-unnamed members-dot-unnamed-root root::root_value -
  rust_closure inline-empty workspace-inline-empty-table alpha::alpha_value lib
  python_closure inline-empty workspace-inline-empty-table alpha::alpha_value -
  rust_closure multi-line-string members-in-a-multi-line-string alpha::alpha_value lib
  python_closure multi-line-string members-in-a-multi-line-string alpha::alpha_value -
  rust_closure brackets 'root[with]brackets' alpha::alpha_value lib
  python_closure brackets 'root[with]brackets' alpha::alpha_value -
  rust_closure absolute-member members-with-an-absolute-path beta::beta_value lib
  python_closure absolute-member members-with-an-absolute-path beta::beta_value -
  rust_closure nested nested-member-inside-another-member outer::outer_value lib
  python_closure nested nested-member-inside-another-member outer::outer_value -
  for implementation in rust python; do closure_summary nested "$implementation" crates/outer/inner/src/lib.rs:1-3; done
  for case in dot-named dot-unnamed; do
    for implementation in rust python; do closure_summary "$case" "$implementation" src/lib.rs:1-3; done
  done
  for case in inline-empty multi-line-string brackets; do
    for implementation in rust python; do closure_summary "$case" "$implementation" crates/alpha/src/lib.rs:1-3; done
  done
  for implementation in rust python; do closure_summary absolute-member "$implementation" beta/src/lib.rs:1-3; done
  rust_closure same-name-tool members-sharing-a-directory-name commontool::smoke_runs test:smoke
  python_closure same-name-tool members-sharing-a-directory-name tools/common/tests/smoke.rs::smoke_runs common:test:smoke
  rust_closure same-name-library members-sharing-a-directory-name commonlib::smoke_runs test:smoke
  python_closure same-name-library members-sharing-a-directory-name crates/common/tests/smoke.rs::smoke_runs common:test:smoke
  for implementation in rust python; do
    closure_summary same-name-tool "$implementation" tools/common/src/lib.rs:1-3
    closure_summary same-name-library "$implementation" crates/common/src/lib.rs:1-3
  done

  # 运行期探针：夹具原样编成程序跑一遍，看运行时真调到的是哪一份（第 2、3 件的期望）
  local probe sample rc
  for probe in use-relative-path-alias-in-the-same-use shadow-through-a-macro-named-like-a-standard-one; do
    rc=0
    setsid --wait rustc --edition 2021 -A warnings -o "$WORK/probes/$probe" "$MODEL/tools/probes/$probe.rs" > "$out/runtime-$probe.compile.txt" 2>&1 &
    sample=$!
    wait "$sample" || rc=$?
    group_left_behind "$sample" "runtime-$probe.compile"
    echo "name=runtime_compile probe=$probe exit=$rc"
    ((rc == 0)) || continue
    setsid --wait "$WORK/probes/$probe" > "$out/runtime-$probe.txt" 2>&1 &
    sample=$!
    wait "$sample" || rc=$?
    group_left_behind "$sample" "runtime-$probe"
    sed -e "s/^runtime /name=runtime probe=$probe exit=$rc /" "$out/runtime-$probe.txt"
  done

  # 链接探针：libfakesync.so 导出 fdatasync（交 4242），夹具各包的测试二进制起来打 fdatasync-result；4242 是调到了它、-1 是 C 库那一份
  mkdir -p "$WORK/native" "$WORK/link"
  gcc -shared -fPIC -o "$WORK/native/libfakesync.so" "$MODEL/tools/probes/fakesync.c"
  rsync -a "$MODEL/fixtures/extern-link-from-another-target-of-the-final-binary/" "$WORK/link/extern-link-from-another-target-of-the-final-binary/"
  rsync -a "$MODEL/fixtures/extern-link-through-a-build-dependency/" "$WORK/link/extern-link-through-a-build-dependency/"
  rm -f "$WORK/link/extern-link-from-another-target-of-the-final-binary/Cargo.lock" "$WORK/link/extern-link-through-a-build-dependency/Cargo.lock"
  link_probe() {   # link_probe <包名> <夹具目录名>
    local sample rc=0 result
    setsid --wait env RUSTFLAGS="-L native=$WORK/native" LD_LIBRARY_PATH="$WORK/native" cargo test --offline --manifest-path "$WORK/link/$2/crates/$1/Cargo.toml" --target-dir "$WORK/link/target" -- --nocapture > "$out/link-$1.txt" 2>&1 &
    sample=$!
    wait "$sample" || rc=$?
    group_left_behind "$sample" "link-$1"
    result="$(grep -o 'fdatasync-result [-0-9]*' "$out/link-$1.txt" | head -n 1)" || result=""
    echo "name=link_probe crate=$1 exit=$rc ${result:-fdatasync-result missing}"
  }
  link_probe fexternfinal extern-link-from-another-target-of-the-final-binary
  link_probe fexterndep extern-link-from-another-target-of-the-final-binary
  link_probe fexternclean extern-link-from-another-target-of-the-final-binary
  link_probe fbuildlink extern-link-through-a-build-dependency
  link_probe fcclink extern-link-through-a-build-dependency

  # 增量探针：同一个小包在五种设置下编，读 cargo -v 给工作区包与依赖包的 rustc 参数，再算 unit-graph 的键（--toolchain-of-the-build dev）
  local setting
  for setting in default cargo-build-incremental-false config-build-incremental-false cargo-incremental-zero explicit-codegen-units-256; do
    mkdir -p "$WORK/incremental/$setting/src"
    cp "$MODEL/tools/probes/incremental-probe-crate/Cargo.toml.template" "$WORK/incremental/$setting/Cargo.toml"
    cp "$MODEL/tools/probes/incremental-probe-crate/src/lib.rs" "$WORK/incremental/$setting/src/lib.rs"
  done
  printf '\n[profile.dev]\ncodegen-units = 256\n' >> "$WORK/incremental/explicit-codegen-units-256/Cargo.toml"
  mkdir -p "$WORK/incremental/config-build-incremental-false/.cargo"
  printf '[build]\nincremental = false\n' > "$WORK/incremental/config-build-incremental-false/.cargo/config.toml"
  incremental_build() {   # incremental_build <设置>：照设置名定环境，编、算键
    local sample rc=0
    case "$1" in
      cargo-build-incremental-false)
        setsid --wait env -u CARGO_INCREMENTAL -u CARGO_PROFILE_DEV_INCREMENTAL -u CARGO_PROFILE_DEV_CODEGEN_UNITS -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS CARGO_BUILD_INCREMENTAL=false cargo build -v --offline --manifest-path "$WORK/incremental/$1/Cargo.toml" --target-dir "$WORK/incremental/$1/target" > "$out/incremental-$1.build.txt" 2>&1 &
        sample=$! ;;
      cargo-incremental-zero)
        setsid --wait env -u CARGO_BUILD_INCREMENTAL -u CARGO_PROFILE_DEV_INCREMENTAL -u CARGO_PROFILE_DEV_CODEGEN_UNITS -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS CARGO_INCREMENTAL=0 cargo build -v --offline --manifest-path "$WORK/incremental/$1/Cargo.toml" --target-dir "$WORK/incremental/$1/target" > "$out/incremental-$1.build.txt" 2>&1 &
        sample=$! ;;
      *)
        setsid --wait env -u CARGO_INCREMENTAL -u CARGO_BUILD_INCREMENTAL -u CARGO_PROFILE_DEV_INCREMENTAL -u CARGO_PROFILE_DEV_CODEGEN_UNITS -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS cargo build -v --offline --manifest-path "$WORK/incremental/$1/Cargo.toml" --target-dir "$WORK/incremental/$1/target" > "$out/incremental-$1.build.txt" 2>&1 &
        sample=$! ;;
    esac
    wait "$sample" || rc=$?
    group_left_behind "$sample" "incremental-$1.build"
    echo "$rc" > "$out/incremental-$1.build.rc"
    rc=0
    case "$1" in
      cargo-build-incremental-false)
        setsid --wait env -u CARGO_INCREMENTAL -u CARGO_PROFILE_DEV_INCREMENTAL -u CARGO_PROFILE_DEV_CODEGEN_UNITS -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS CARGO_BUILD_INCREMENTAL=false "$WORK/target/debug/unit-graph" key "$WORK/incremental/$1" incprobe::probe_value --toolchain-of-the-build dev > "$out/incremental-$1.key.txt" 2>&1 &
        sample=$! ;;
      cargo-incremental-zero)
        setsid --wait env -u CARGO_BUILD_INCREMENTAL -u CARGO_PROFILE_DEV_INCREMENTAL -u CARGO_PROFILE_DEV_CODEGEN_UNITS -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS CARGO_INCREMENTAL=0 "$WORK/target/debug/unit-graph" key "$WORK/incremental/$1" incprobe::probe_value --toolchain-of-the-build dev > "$out/incremental-$1.key.txt" 2>&1 &
        sample=$! ;;
      *)
        setsid --wait env -u CARGO_INCREMENTAL -u CARGO_BUILD_INCREMENTAL -u CARGO_PROFILE_DEV_INCREMENTAL -u CARGO_PROFILE_DEV_CODEGEN_UNITS -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS "$WORK/target/debug/unit-graph" key "$WORK/incremental/$1" incprobe::probe_value --toolchain-of-the-build dev > "$out/incremental-$1.key.txt" 2>&1 &
        sample=$! ;;
    esac
    wait "$sample" || rc=$?
    group_left_behind "$sample" "incremental-$1.key"
    echo "$rc" > "$out/incremental-$1.key.rc"
  }
  rustc_flag() {   # rustc_flag <设置> <crate 名> <incremental|codegen-units>：cargo -v 给这个 crate 的 rustc 那一行里这个开关的值
    local line
    line="$(grep -E "Running \`.*--crate-name $2 " "$out/incremental-$1.build.txt" | head -n 1)" || line=""
    if [[ -z "$line" ]]; then echo missing; return; fi
    case "$3" in
      incremental) if grep -q -- '-C incremental=' <<< "$line"; then echo yes; else echo no; fi ;;
      codegen-units) grep -oE -- '-C codegen-units=[0-9]+' <<< "$line" | head -n 1 | sed 's/.*=//' | grep . || echo unset ;;
    esac
  }
  for setting in default cargo-build-incremental-false config-build-incremental-false cargo-incremental-zero explicit-codegen-units-256; do
    incremental_build "$setting"
    echo "name=incremental setting=$setting build_exit=$(cat "$out/incremental-$setting.build.rc") member_incremental=$(rustc_flag "$setting" incprobe incremental) member_codegen_units=$(rustc_flag "$setting" incprobe codegen-units) dependency_incremental=$(rustc_flag "$setting" cfg_if incremental) dependency_codegen_units=$(rustc_flag "$setting" cfg_if codegen-units) key_exit=$(cat "$out/incremental-$setting.key.rc") $(grep -m 1 -E '^key' "$out/incremental-$setting.key.txt" | tr '\t' '=' || echo key=missing)"
  done

  # 第 3 条（验证包判器组键）：同一份冻结树，根清单 members 有没有 gpu-verification-server 两臂，收 observer_conclusion_composition 那四个入口的闭包
  local arm
  for arm in with-the-server without-the-server; do
    mkdir -p "$WORK/members-ab/$arm"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/members-ab/$arm/"
  done
  grep -vxF '    "gpu-verification-server",' "$FROZEN/Cargo.toml" > "$WORK/members-ab/without-the-server/Cargo.toml"
  echo "name=members_ab_manifest_lines_removed count=$(( $(wc -l < "$FROZEN/Cargo.toml") - $(wc -l < "$WORK/members-ab/without-the-server/Cargo.toml") ))"
  for arm in with-the-server without-the-server; do
    rc=0
    setsid --wait "$WORK/target/debug/unit-graph" closure "$WORK/members-ab/$arm" \
      singlefs_verification::crash_amplification::judge_the_images_after_the_observers_on_the_cpu \
      singlefs_verification::crash_amplification::judge_the_images_after_the_observers_on_the_gpu \
      singlefs_verification::crash_amplification::plan_the_judge_tables_of_the_images_after_the_observers \
      singlefs_verification::crash_amplification::pool_after_the_observer \
      --target lib --cfg 'feature="gpu"' --judgment-kind product-behavior --toolchain-line 'host: x86_64-unknown-linux-gnu' \
      > "$out/members-ab-$arm.txt" 2> "$out/members-ab-$arm.err" &
    sample=$!
    wait "$sample" || rc=$?
    group_left_behind "$sample" "members-ab-$arm"
    echo "name=members_ab arm=$arm exit=$rc lines=$(wc -l < "$out/members-ab-$arm.txt") refused_lines=$(grep -c $'^refused\t' "$out/members-ab-$arm.txt" || true) key_line=$(grep -m 1 -E '^key' "$out/members-ab-$arm.txt" | cut -f2 || echo none)"
  done
  sort "$out/members-ab-with-the-server.txt" > "$out/members-ab-with-the-server.sorted"
  sort "$out/members-ab-without-the-server.txt" > "$out/members-ab-without-the-server.sorted"
  comm -23 "$out/members-ab-with-the-server.sorted" "$out/members-ab-without-the-server.sorted" > "$out/members-ab-only-with-the-server.txt"
  comm -13 "$out/members-ab-with-the-server.sorted" "$out/members-ab-without-the-server.sorted" > "$out/members-ab-only-without-the-server.txt"
  echo "name=members_ab_difference only_with_the_server=$(wc -l < "$out/members-ab-only-with-the-server.txt") only_without_the_server=$(wc -l < "$out/members-ab-only-without-the-server.txt")"
  sed -e 's/^/name=members_ab_only_with_the_server line=/' "$out/members-ab-only-with-the-server.txt" | head -n 60
  sed -e 's/^/name=members_ab_only_without_the_server line=/' "$out/members-ab-only-without-the-server.txt" | head -n 60
  local unit_substring
  for unit_substring in prune-server/ gpu-verification-server/; do
    rc=0
    setsid --wait "$WORK/target/debug/unit-graph" why "$WORK/members-ab/with-the-server" \
      singlefs_verification::crash_amplification::judge_the_images_after_the_observers_on_the_cpu \
      singlefs_verification::crash_amplification::judge_the_images_after_the_observers_on_the_gpu \
      singlefs_verification::crash_amplification::plan_the_judge_tables_of_the_images_after_the_observers \
      singlefs_verification::crash_amplification::pool_after_the_observer \
      --unit "$unit_substring" --target lib --cfg 'feature="gpu"' --judgment-kind product-behavior --toolchain-line 'host: x86_64-unknown-linux-gnu' \
      > "$out/members-ab-why-${unit_substring%/}.txt" 2>&1 &
    sample=$!
    wait "$sample" || rc=$?
    group_left_behind "$sample" "members-ab-why-${unit_substring%/}"
    echo "name=members_ab_why unit=$unit_substring exit=$rc lines=$(wc -l < "$out/members-ab-why-${unit_substring%/}.txt")"
  done

  # 第 1 条：第一轮那八棵夹具原样喂今天的两份（fixtures/round-one 与第一轮草稿目录里的逐字节相同）
  rust_closure r1-macro-path round-one/shadow-through-macro fshadow::entry_through_the_module_path lib
  python_closure r1-macro-path round-one/shadow-through-macro fshadow::entry_through_the_module_path -
  rust_closure r1-macro-nested round-one/shadow-through-macro fshadow::entry_inside_a_nested_function lib
  python_closure r1-macro-nested round-one/shadow-through-macro fshadow::entry_inside_a_nested_function -
  rust_closure r1-macro-control round-one/shadow-through-macro fshadow::entry_without_a_binding lib
  python_closure r1-macro-control round-one/shadow-through-macro fshadow::entry_without_a_binding -
  for case in r1-macro-path r1-macro-nested r1-macro-control; do
    for implementation in rust python; do closure_summary "$case" "$implementation" crates/fshadow/src/lib.rs:14-16; done
  done
  rust_closure r1-cfg-swap round-one/use-cfg-swapped-re-export fswap::entry_point lib
  python_closure r1-cfg-swap round-one/use-cfg-swapped-re-export fswap::entry_point -
  rust_closure r1-module-alias round-one/use-relative-path-through-module-alias falias::entry_point lib
  python_closure r1-module-alias round-one/use-relative-path-through-module-alias falias::entry_point -
  rust_closure r1-alias-leaf round-one/use-alias-leaf-divergence fdiverge::entry_point lib
  python_closure r1-alias-leaf round-one/use-alias-leaf-divergence fdiverge::entry_point -
  for implementation in rust python; do
    closure_summary r1-cfg-swap "$implementation" crates/fswap/src/lib.rs:12-14
    closure_summary r1-module-alias "$implementation" crates/falias/src/lib.rs:3-5
    closure_summary r1-alias-leaf "$implementation" crates/fdiverge/src/lib.rs:12-14
  done
  rust_closure r1-relaunch round-one/relaunch-shadowed-binding frelaunch::main bin:frelaunch
  python_closure r1-relaunch round-one/relaunch-shadowed-binding crates/frelaunch/src/main.rs::main frelaunch:bin:frelaunch
  rust_closure r1-fallback round-one/restart-with-argument-fallback ffallback::main bin:ffallback
  python_closure r1-fallback round-one/restart-with-argument-fallback crates/ffallback/src/main.rs::main ffallback:bin:ffallback
  for case in r1-relaunch r1-fallback; do
    for implementation in rust python; do closure_summary "$case" "$implementation" current-exe; done
  done
  rust_closure r1-extern-attribute round-one/extern-link-through-cfg-attr fexternattr::entry_point lib
  python_closure r1-extern-attribute round-one/extern-link-through-cfg-attr fexternattr::entry_point -
  rust_closure r1-extern-other-block round-one/extern-link-on-another-block fexternother::entry_point lib
  python_closure r1-extern-other-block round-one/extern-link-on-another-block fexternother::entry_point -
  for case in r1-extern-attribute r1-extern-other-block; do
    for implementation in rust python; do closure_summary "$case" "$implementation" extern-function-interface; done
  done

  echo "name=rerun_done process_groups_left_behind=$leftovers"
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
