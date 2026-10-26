#!/usr/bin/env bash
# 起 cargo 的 shell 入口每一趟自己的编译目录（用户 2026-10-23 定案「自己用完自己删」「所有脚本自行编译，完成后自行删除 不再预留生成物」，不留给以后的趟复用）。
# 两种用法：被 source（入口脚本调下面两个函数）；单独执行时包一层起一条命令（文件末尾「包一层起」一段）。
# 被 source 时只定义函数与几个全局变量、不改 shell 选项。删之前核开着的文件那一判在 scripts/hosts/run_build_directory.py（python 入口直接 import 它），两种用法都经它删。
#
#   run_build_directory_begin <这一趟的运行目录>
#       先照 run_directory_owner_record 给运行目录写归属记录（目录已经在的这时写，还没建的建出来之后写）。
#       调用方设了 CARGO_TARGET_DIR：不动，交回 0（那一份归调用方删）。没设：建 <运行目录>/target、export 成 CARGO_TARGET_DIR，
#       run_build_directory_owned 记下它；<运行目录>/target 已经在（不是这一趟建的）或建不成：打 ✗ 与出路（标准错误），交回 1，什么都没设。
#   run_build_directory_end [keep|unchecked]
#       删 run_build_directory_owned 那一份（没建的交回 0）：删之前核本机没有进程开着里面的文件，有就不删、打 ✗ 与逐个进程、交回 1；删不成交回 1。
#       删成了不打字（调用方的输出不多一行）。keep：不删（调用方的弄坏开关用），打一行 !；unchecked：不核开着的文件就删（同上）。
#       删成了（或本来就没建）之后，运行目录里只剩这一趟写的那份 .owner 的，连它删掉（调用方随后 rmdir 运行目录删得掉）；还有别的东西的留着它。
#   run_directory_owner_record <运行目录> [<用途一句>]
#       运行目录的归属记录 <运行目录>/.owner：三行 session、created、purpose，写法同 scripts/hosts/work-directory-ownership.sh 的归属文件；
#       完工清理闸（.claude/hooks/completion-cleanup-guard.sh 文件头「判哪几处」运行目录那一段）起它的进程不在了之后照它认会话号。
#       会话号依次取 WORK_DIRECTORY_OWNER_SESSION、CLAUDE_CODE_SESSION_ID、CLAUDE_SESSION_ID（同 work-directory-ownership.sh 的 resolve_session），
#       都没有（会话外跑的）或带空白不写；目录不在、已有 .owner 的不动；用途没给写「<入口脚本名> 这一趟的运行目录」。写不成不报错（归属记录只是给闸认人用）。
# 调用方在 EXIT trap 里调 run_build_directory_end，另设 TERM / INT 的 trap（exit 143 / 130），被停时才走得到 EXIT。

run_build_directory_owned=""
run_build_directory_owner_written=""   # run_directory_owner_record 这一趟写下的那份 .owner（run_build_directory_end 只删它）
run_build_directory_library="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/run_build_directory.py"

# 弄坏开关只在 --selftest 起的那几趟里认（自证设 RUN_BUILD_DIRECTORY_SELFTEST_MARK 指到它建的标记文件）：被 source 进入口脚本时外面漏进来的开关不生效
run_build_directory_selftest_break_is() {
  [[ ",${RUN_BUILD_DIRECTORY_BREAK:-}," == *",$1,"* && -n "${RUN_BUILD_DIRECTORY_SELFTEST_MARK:-}" ]] \
    && [[ "$(cat -- "$RUN_BUILD_DIRECTORY_SELFTEST_MARK" 2>/dev/null)" == run-build-directory-selftest ]]
}

run_directory_owner_record() { # <运行目录> [<用途一句>]
  local session="${WORK_DIRECTORY_OWNER_SESSION:-${CLAUDE_CODE_SESSION_ID:-${CLAUDE_SESSION_ID:-}}}"
  run_build_directory_selftest_break_is owner-unrecorded && return 0
  [[ -n "$session" && "$session" != *[[:space:]]* && -d "$1" && ! -e "$1/.owner" && ! -L "$1/.owner" ]] || return 0
  if ( set -o noclobber; printf 'session: %s\ncreated: %s\npurpose: %s\n' "$session" "$(TZ=Asia/Tokyo date +%F)" \
         "${2:-$(basename -- "$0") 这一趟的运行目录}" > "$1/.owner" ) 2>/dev/null; then
    run_build_directory_owner_written="$1/.owner"
  fi
  return 0
}

# run_build_directory_end 删完编译目录之后：运行目录里只剩这一趟写的 .owner，就删掉它
run_build_directory_owner_drop_if_alone() {
  local record="$run_build_directory_owner_written"
  run_build_directory_owner_written=""
  [[ -n "$record" && -f "$record" ]] || return 0
  run_build_directory_selftest_break_is owner-left-alone && return 0
  [[ "$(ls -A -- "${record%/.owner}" 2>/dev/null)" == .owner ]] && rm -f -- "$record"
  return 0
}

run_build_directory_begin() {
  run_directory_owner_record "$1"
  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    return 0
  fi
  if [[ -e "$1/target" ]]; then
    echo "  ✗ 这一趟的编译目录位置 $1/target 已经有东西，不往里编、也不删它" >&2
    echo "  → 怎么办：换一趟（这一趟的运行目录带时刻与进程号，重跑就是新的），或先看清那里是什么；要用现成的编译目录就设 CARGO_TARGET_DIR（那一份收尾不删）" >&2
    return 1
  fi
  if ! mkdir -p -- "$1" || ! mkdir -- "$1/target"; then
    echo "  ✗ 建不了这一趟的编译目录 $1/target" >&2
    echo "  → 怎么办：看 $1 的上一层可不可写、盘满没满，修好再跑" >&2
    return 1
  fi
  run_directory_owner_record "$1"
  run_build_directory_owned="$1/target"
  export CARGO_TARGET_DIR="$run_build_directory_owned"
  return 0
}

run_build_directory_end() {
  local output
  if [[ -z "$run_build_directory_owned" ]]; then
    run_build_directory_owner_drop_if_alone
    return 0
  fi
  if [[ "${1:-}" == keep ]]; then
    echo "  ! 这一趟的编译目录 $run_build_directory_owned 不删（弄坏开关）" >&2
    return 0
  fi
  if output="$(python3 "$run_build_directory_library" remove "$run_build_directory_owned" $([[ "${1:-}" == unchecked ]] && echo --skip-open-check) 2>&1)"; then
    run_build_directory_owned=""
    run_build_directory_owner_drop_if_alone
    return 0
  fi
  printf '%s\n' "$output" >&2
  return 1
}

# ── 包一层起（单独执行这一份） ──
#   bash scripts/hosts/run-build-directory.sh [--] <命令> [参数…]
#       主工作区里直接敲的 cargo 编进 <仓根>/target，由 .claude/hooks/heavy-test-guard.sh 在执行前拒（判法在 .claude/hooks/lib_workspace_cargo_lock.py 文件头「编译目录」一段），
#       出路就是这一条：调用方没设 CARGO_TARGET_DIR 时，在 ${TMPDIR:-/tmp} 下 mktemp -d 这一趟的运行目录 singlefs-run-build-directory-<进程号>-XXXXXX，
#       照 run_build_directory_begin 把里面的 target 设成 CARGO_TARGET_DIR 交给命令；命令正常退、出错、包装被 TERM / INT 停之后都照 run_build_directory_end 删
#       （删之前核本机没有进程开着里面的文件），再删运行目录。调用方设了 CARGO_TARGET_DIR 的照它的、不建不删（同 run_build_directory_begin）。
#       经 run-with-memory-cap.sh 起的（它给命令一个这一条自己的 TMPDIR）运行目录落在那一份里，这一层被 KILL 时由外层删。
#       范围声明（SINGLEFS_RUN_SCOPE）与 capped.sh、run-with-memory-cap.sh、nice、timeout、env 这些包装写在这一层外面；钩子经 lib_heavy_tests.py 的 command_under_launcher
#       看穿这一层（command_under_run_build_directory），里面那条照原判法判。
#       退出码：命令自己的；编译目录删不成（有进程开着里面的文件、删完回读还在）或运行目录删完编译目录之后不是空的：打 ✗ 与出路，命令退 0 的改退 1；
#       没给命令、命令前写了认不出的选项退 2；运行目录建不了退 1、命令没跑。
#       包装被 TERM / INT 停：先给命令（这一层自己 `&` 起、`$!` 记下的那一个进程）转同一个信号、等它退，再删，退 143 / 130。
#   bash scripts/hosts/run-build-directory.sh --selftest
#       自证：命令正常退、出错、包装被 TERM 三格「跑完不剩编译目录」（这三格设着会话号，运行目录里写过 .owner，跑完连它一起不剩），
#       调用方给了 CARGO_TARGET_DIR 照它的且不删一格，没给命令退 2 一格，归属记录三格（begin 建出来的运行目录里有 .owner、三行、会话号是给的那个；
#       没有会话号不写；已有 .owner 不动）；只起 bash、sleep 与 python3（删法），不起 cargo。
# 弄坏开关 RUN_BUILD_DIRECTORY_BREAK=<项>（只在 --selftest 起的那几趟里认：自证设 RUN_BUILD_DIRECTORY_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   RUN_BUILD_DIRECTORY_BREAK=kept  跑完不删这一趟的编译目录与运行目录（run_build_directory_end keep）
#       「命令正常退出之后，这一趟的编译目录应当删掉」「命令出错退出之后，这一趟的编译目录应当删掉」「包装被 TERM 停之后，这一趟的编译目录应当删掉」
#   RUN_BUILD_DIRECTORY_BREAK=foreground  命令在前台跑、不放后台用 wait 等（bash 等前台命令退出之前不跑 trap，TERM 转不到命令）
#       「包装被 TERM 停之后，这一趟的编译目录应当删掉」（实际包装等命令自己睡完才退）
#   RUN_BUILD_DIRECTORY_BREAK=owner-unrecorded  run_directory_owner_record 不写归属记录（改前的样子）
#       「run_build_directory_begin 建出来的运行目录里应当有归属记录 .owner」
#   RUN_BUILD_DIRECTORY_BREAK=owner-left-alone  run_build_directory_end 删完编译目录之后不删只剩它自己的 .owner（运行目录 rmdir 不掉）
#       「命令正常退出之后，这一趟的编译目录应当删掉」「命令出错退出之后，这一趟的编译目录应当删掉」「包装被 TERM 停之后，这一趟的编译目录应当删掉」
#       「run_build_directory_begin 建出来的运行目录里应当有归属记录 .owner」（end 之后还剩 .owner）

run_build_directory_break_is() {
  [[ ",${RUN_BUILD_DIRECTORY_BREAK:-}," == *",$1,"* ]]
}

run_build_directory_child=""           # 包一层起的那条命令：`&` 起、`$!` 记下，wait 等到它退出就清空
run_build_directory_run_directory=""   # 这一趟的运行目录（调用方给了 CARGO_TARGET_DIR 时是空的）

run_build_directory_stop() { # run_build_directory_stop <信号名> <退出码>
  if [[ -n "$run_build_directory_child" ]]; then
    kill -"$1" "$run_build_directory_child" 2>/dev/null
    wait "$run_build_directory_child" 2>/dev/null
    run_build_directory_child=""
  fi
  exit "$2"
}

run_build_directory_finish() {
  local status=$?
  trap - EXIT TERM INT
  if ! run_build_directory_end $(run_build_directory_break_is kept && echo keep); then
    ((status == 0)) && status=1
  fi
  if [[ -n "$run_build_directory_run_directory" && -d "$run_build_directory_run_directory" ]] && ! run_build_directory_break_is kept \
      && ! rmdir -- "$run_build_directory_run_directory" 2>/dev/null; then
    echo "  ✗ 这一趟的运行目录 $run_build_directory_run_directory 删编译目录之后不是空的（或删不成），没删" >&2
    echo "  → 怎么办：ls -la $run_build_directory_run_directory 看是谁写进去的；还开着编译目录里文件的进程照上面那几行停掉再手删" >&2
    ((status == 0)) && status=1
  fi
  exit "$status"
}

run_build_directory_wrapper() {
  local status
  local -a interrupt_option=(--default-signal=INT)
  # 脚本里 & 起的命令一律忽略 INT：进来时 INT 没被忽略的，给命令设回默认；进来时就被忽略的（调用方有意如此）照旧忽略
  [[ "$(trap -p INT)" != *"''"* ]] || interrupt_option=()
  if [[ "${1:-}" == -- ]]; then
    shift
  elif [[ "${1:-}" == -* ]]; then
    set --
  fi
  if [[ $# -eq 0 ]]; then
    echo "  ✗ 用法不对：bash scripts/hosts/run-build-directory.sh [--] <命令> [参数…]（或 --selftest）" >&2
    echo "  → 怎么办：把要编译的那条命令（例 cargo build --offline -p singlefs-harness）写在后面；范围声明与内存包装写在这一层外面（文件头「包一层起」一段）" >&2
    exit 2
  fi
  if [[ -z "${CARGO_TARGET_DIR:-}" ]] && ! run_build_directory_run_directory="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-run-build-directory-$$-XXXXXX")"; then
    run_build_directory_run_directory=""
    echo "  ✗ 建不了这一趟的运行目录（mktemp -d 在 ${TMPDIR:-/tmp} 下失败），命令没跑" >&2
    echo "  → 怎么办：看 ${TMPDIR:-/tmp} 在不在、可不可写、盘满没满；或自己设 CARGO_TARGET_DIR 到仓外一个用完自己删的目录" >&2
    exit 1
  fi
  trap run_build_directory_finish EXIT
  trap 'run_build_directory_stop TERM 143' TERM
  trap 'run_build_directory_stop INT 130' INT
  if [[ -n "$run_build_directory_run_directory" ]] && ! run_build_directory_begin "$run_build_directory_run_directory"; then
    exit 1
  fi
  if run_build_directory_break_is foreground; then
    "$@"
    exit $?
  fi
  env ${interrupt_option[@]+"${interrupt_option[@]}"} "$@" <&0 &
  run_build_directory_child=$!
  wait "$run_build_directory_child"
  status=$?
  run_build_directory_child=""
  exit "$status"
}

run_build_directory_selftest() {
  local self scratch failures=0 checked=0 status name output leftover root recorded wrapper_pid waited stat_line code
  self="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
  if ! scratch="$(mktemp -d "${TMPDIR:-/tmp}/run-build-directory-selftest-XXXXXX")"; then
    echo "  ✗ 自检：建不了临时目录（mktemp -d 在 ${TMPDIR:-/tmp} 下失败）"
    echo "  → 怎么办：看 ${TMPDIR:-/tmp} 可不可写、盘满没满，修好再跑"
    return 1
  fi
  printf 'run-build-directory-selftest' > "$scratch/mark"
  selftest_fail() { echo "  ✗ 自检：$1"; failures=$((failures + 1)); }   # gate-lint:detail
  # 取一份文件的头 400 字节给失败说明用：按字节截会截出半个汉字（输出就不是合法 UTF-8 了），所以截完丢掉末尾不完整的那几个字节
  selftest_head_text() { python3 -c 'import sys; sys.stdout.write(open(sys.argv[1], "rb").read(400).decode("utf-8", "ignore"))' "$1" 2>/dev/null; }
  # 假命令：把拿到的 CARGO_TARGET_DIR 写进 scratch 里的一份文件、往里写一份「编出来的」文件，再照各自的结局退。每一例一个自己的 TMPDIR（运行目录建在它底下）
  code='printf "%s\n" "${CARGO_TARGET_DIR-}" > "$1"; if [[ -n "${CARGO_TARGET_DIR-}" ]]; then mkdir -p "$CARGO_TARGET_DIR/debug" && : > "$CARGO_TARGET_DIR/debug/built-by-command"; fi; '
  for name in normal error; do
    root="$scratch/root-$name"
    recorded="$scratch/$name.path"
    mkdir -p "$root"
    checked=$((checked + 1))
    if env -u CARGO_TARGET_DIR TMPDIR="$root" RUN_BUILD_DIRECTORY_SELFTEST_MARK="$scratch/mark" WORK_DIRECTORY_OWNER_SESSION=run-build-directory-selftest \
        bash "$self" bash -c "$code$([[ $name == normal ]] && echo 'exit 0' || echo 'exit 3')" fake-build "$recorded" >"$scratch/$name.out" 2>"$scratch/$name.err"; then status=0; else status=$?; fi
    output="$(cat "$recorded" 2>/dev/null)"
    leftover="$(ls -A "$root" 2>/dev/null)"
    if [[ $status -ne $([[ $name == normal ]] && echo 0 || echo 3) || "$output" != "$root"/singlefs-run-build-directory-*/target || -e "$output" || -n "$leftover" ]]; then
      selftest_fail "命令$([[ $name == normal ]] && echo '正常' || echo '出错')退出之后，这一趟的编译目录应当删掉（退 $([[ $name == normal ]] && echo 0 || echo 3)、命令拿到的 CARGO_TARGET_DIR 在 $root 底下、跑完那一份与运行目录都不在），\
实际退 $status、命令拿到的是「$output」、$root 底下剩「$leftover」：$(selftest_head_text "$scratch/$name.err")"
    fi
  done
  # 包装被 TERM 停：命令（exec 成 sleep 8）随之收到 TERM；包装 5 秒内退 143，编译目录与运行目录都不在
  root="$scratch/root-term"
  recorded="$scratch/term.path"
  mkdir -p "$root"
  checked=$((checked + 1))
  env -u CARGO_TARGET_DIR TMPDIR="$root" RUN_BUILD_DIRECTORY_SELFTEST_MARK="$scratch/mark" WORK_DIRECTORY_OWNER_SESSION=run-build-directory-selftest \
    bash "$self" bash -c "$code"': > "$2"; exec sleep 8' fake-build "$recorded" "$scratch/term.started" >"$scratch/term.out" 2>"$scratch/term.err" &
  wrapper_pid=$!
  for _ in $(seq 100); do
    [[ -e "$scratch/term.started" ]] && break
    sleep 0.1
  done
  kill -TERM "$wrapper_pid" 2>/dev/null
  waited=0
  # 包装退了之后在 wait 之前是僵尸，kill -0 照样成功：看 /proc 里的状态
  while ((waited < 50)) && IFS= read -r stat_line 2>/dev/null < "/proc/$wrapper_pid/stat" && [[ "${stat_line##*) }" != Z* ]]; do
    sleep 0.1
    waited=$((waited + 1))
  done
  wait "$wrapper_pid"
  status=$?
  output="$(cat "$recorded" 2>/dev/null)"
  leftover="$(ls -A "$root" 2>/dev/null)"
  if [[ $status -ne 143 || $waited -ge 50 || "$output" != "$root"/singlefs-run-build-directory-*/target || -e "$output" || -n "$leftover" ]]; then
    selftest_fail "包装被 TERM 停之后，这一趟的编译目录应当删掉（5 秒内退 143、命令拿到的 CARGO_TARGET_DIR 在 $root 底下、跑完那一份与运行目录都不在），\
实际退 $status、等了 $((waited / 10)) 秒多、命令拿到的是「$output」、$root 底下剩「$leftover」：$(selftest_head_text "$scratch/term.err")"
  fi
  # 调用方给了 CARGO_TARGET_DIR：命令拿到的就是它，不建运行目录、跑完不删
  root="$scratch/root-caller"
  recorded="$scratch/caller.path"
  mkdir -p "$root" "$scratch/caller-target"
  : > "$scratch/caller-target/placed-by-caller"
  checked=$((checked + 1))
  if env CARGO_TARGET_DIR="$scratch/caller-target" TMPDIR="$root" RUN_BUILD_DIRECTORY_SELFTEST_MARK="$scratch/mark" \
      bash "$self" bash -c "${code}exit 0" fake-build "$recorded" >"$scratch/caller.out" 2>"$scratch/caller.err"; then status=0; else status=$?; fi
  output="$(cat "$recorded" 2>/dev/null)"
  leftover="$(ls -A "$root" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != "$scratch/caller-target" || ! -e "$scratch/caller-target/placed-by-caller" || ! -e "$scratch/caller-target/debug/built-by-command" || -n "$leftover" ]]; then
    selftest_fail "调用方给了 CARGO_TARGET_DIR 时命令应当拿到它、跑完那一份与里面的文件都还在、不另建运行目录，实际退 $status、命令拿到的是「$output」、$root 底下剩「$leftover」"
  fi
  # 没给命令：用法错退 2
  checked=$((checked + 1))
  if env RUN_BUILD_DIRECTORY_SELFTEST_MARK="$scratch/mark" bash "$self" 2>"$scratch/usage.err"; then status=0; else status=$?; fi
  [[ $status -eq 2 ]] || selftest_fail "没给命令应当按用法错退 2，实际退 $status：$(cat "$scratch/usage.err")"
  # 归属记录（文件头 run_directory_owner_record）：另起一个 bash 只 source 这一份，begin 之后打出 .owner、再 end；三格各给不同的会话号环境
  local owner_probe owner_lines leftover_after
  owner_probe='source "$1" || exit 3; unset CARGO_TARGET_DIR; run_build_directory_begin "$2" || exit 4; cat -- "$2/.owner" 2>/dev/null; run_build_directory_end || exit 5; exit 0'
  checked=$((checked + 1))
  if owner_lines="$(env -u CLAUDE_CODE_SESSION_ID -u CLAUDE_SESSION_ID WORK_DIRECTORY_OWNER_SESSION=run-build-directory-selftest-owner \
       RUN_BUILD_DIRECTORY_SELFTEST_MARK="$scratch/mark" bash -c "$owner_probe" owner-probe "$self" "$scratch/owner-given/run" 2>"$scratch/owner-given.err")"; then status=0; else status=$?; fi
  leftover_after="$(ls -A "$scratch/owner-given/run" 2>/dev/null)"
  if [[ $status -ne 0 || "$(grep -c . <<<"$owner_lines")" != 3 || "$(sed -n 1p <<<"$owner_lines")" != "session: run-build-directory-selftest-owner" \
        || ! "$(sed -n 2p <<<"$owner_lines")" =~ ^created:\ [0-9]{4}-[0-9]{2}-[0-9]{2}$ || "$(sed -n 3p <<<"$owner_lines")" != "purpose: owner-probe 这一趟的运行目录" \
        || -n "$leftover_after" ]]; then
    selftest_fail "run_build_directory_begin 建出来的运行目录里应当有归属记录 .owner（三行 session、created、purpose，session 是给的会话号 run-build-directory-selftest-owner），end 之后运行目录里不剩它；\
实际退 $status、.owner「$(tr '\n' '|' <<<"$owner_lines")」、end 之后剩「$leftover_after」：$(tr '\n' ' ' <"$scratch/owner-given.err")"
  fi
  checked=$((checked + 1))
  if owner_lines="$(env -u WORK_DIRECTORY_OWNER_SESSION -u CLAUDE_CODE_SESSION_ID -u CLAUDE_SESSION_ID \
       RUN_BUILD_DIRECTORY_SELFTEST_MARK="$scratch/mark" bash -c "$owner_probe" owner-probe "$self" "$scratch/owner-none/run" 2>"$scratch/owner-none.err")"; then status=0; else status=$?; fi
  if [[ $status -ne 0 || -n "$owner_lines" ]]; then
    selftest_fail "没有会话号（三个变量都没设）时不应当写归属记录，实际退 $status、.owner「$(tr '\n' '|' <<<"$owner_lines")」"
  fi
  mkdir -p "$scratch/owner-kept/run"
  printf 'session: someone-else\ncreated: 2026-01-01\npurpose: 别人先写的\n' >"$scratch/owner-kept/run/.owner"
  checked=$((checked + 1))
  if owner_lines="$(env -u CLAUDE_CODE_SESSION_ID -u CLAUDE_SESSION_ID WORK_DIRECTORY_OWNER_SESSION=run-build-directory-selftest-owner \
       RUN_BUILD_DIRECTORY_SELFTEST_MARK="$scratch/mark" bash -c "$owner_probe" owner-probe "$self" "$scratch/owner-kept/run" 2>"$scratch/owner-kept.err")"; then status=0; else status=$?; fi
  if [[ $status -ne 0 || "$owner_lines" != $'session: someone-else\ncreated: 2026-01-01\npurpose: 别人先写的' \
        || "$(cat "$scratch/owner-kept/run/.owner" 2>/dev/null)" != $'session: someone-else\ncreated: 2026-01-01\npurpose: 别人先写的' ]]; then
    selftest_fail "运行目录里已有的 .owner 不应当被改写、end 之后也不应当被删，实际退 $status、begin 之后「$(tr '\n' '|' <<<"$owner_lines")」、end 之后「$(tr '\n' '|' <"$scratch/owner-kept/run/.owner" 2>&1)」"
  fi
  rm -rf -- "${scratch:?}"
  if ((failures)); then
    echo "  ✗ run-build-directory.sh 自检 $failures 处不对（查了 $checked 项）"
    echo "  → 怎么办：照上面逐条改 run_build_directory_wrapper / run_build_directory_finish / run_build_directory_stop 与删法 scripts/hosts/run_build_directory.py；RUN_BUILD_DIRECTORY_BREAK 设着的话本来就该红"
    return 1
  fi
  echo "  ✓ run-build-directory.sh 自检通过（查了 $checked 项：命令正常退、出错、包装被 TERM 之后这一趟的编译目录与运行目录都不在，调用方给了 CARGO_TARGET_DIR 照它的且不删，没给命令退 2，begin 建的运行目录里有会话号对得上的 .owner、没有会话号不写、已有的不动）"
  return 0
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  set -uo pipefail
  if [[ -n "${RUN_BUILD_DIRECTORY_BREAK:-}" && "${1:-}" != --selftest ]] \
      && [[ -z "${RUN_BUILD_DIRECTORY_SELFTEST_MARK:-}" || "$(cat -- "${RUN_BUILD_DIRECTORY_SELFTEST_MARK}" 2>/dev/null)" != run-build-directory-selftest ]]; then
    echo "  ✗ RUN_BUILD_DIRECTORY_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
    echo "  → 怎么办：去掉 RUN_BUILD_DIRECTORY_BREAK 再跑；要证红就跑 RUN_BUILD_DIRECTORY_BREAK=<项> bash scripts/hosts/run-build-directory.sh --selftest" >&2
    exit 2
  fi
  if [[ "${1:-}" == --selftest ]]; then
    run_build_directory_selftest
    exit $?
  fi
  run_build_directory_wrapper "$@"
fi
