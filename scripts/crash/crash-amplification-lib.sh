#!/usr/bin/env bash
# 崩溃放量的两个驱动共用的那几段：读配置的开关与显卡配置、组 cargo 的参数与环境、在本机跑一趟用例、从日志里取派活计划的数。
# 被 source 的函数库，只定义函数、不改 shell 选项；单独执行只认 --selftest（文件头「自证」一段）；scripts/crash/crash-amplification-item.sh（一个分项的一趟）与
# scripts/crash/crash-amplification-two-hosts.sh（两台各领一份）source 它。
# gate-similar: peer-host-lib.sh 它管第二台那几段（读多机配置、在第二台上跑命令、清场，不复原）；这里管本机那一趟与派活计划的数，不连第二台
#
# 调用方先设好这几个变量再调 crash_amplification_prepare：
#   repository_root  仓根
#   test_target      crates/singlefs-checker-tier/tests/ 下的测试目标；test_filter 用例函数名（可空）
#   expand_up_to、threads  可空；memory_cap 本机那一趟的内存上限
#   library_is_temporary   1 表示库只活这一趟（自证用）
#   quiesced               1 表示清过场（卡上别的服务停了）：显卡的先后与额度取清场之后那一份
# crash_amplification_prepare 之后可用：
#   configuration_file（多机配置的路径，文件头「配置」一段怎么找）、gpu_switch（0 / 1）、across_machines_switch（0 / 1）、features、
#   gpu_cards_by_priority（这一趟用的那一份，没写是空串）、gpu_cards_by_priority_when_quiesced（清场之后那一份，没写是空串）、
#   common_environment、cargo_build_arguments、cargo_test_arguments（数组）
#   crash_amplification_prepare 交回非 0（都是 2）只有一种，是配置的事，调用方退 78：说是清过场、开了 GPU 或写了不清场那一份显卡先后，多机配置里却没写清场之后那一份。
# 交给本机那一趟的环境由 crash_amplification_environment_for_this_host 组（数组 this_host_environment，env 的参数）；
#   驱动自证里换掉本机那一趟时也经它组，只把起 cargo 换成假的。
#
# 运行条件与失败分类（崩溃放量入口与分项驱动共用）：
#   crash_amplification_run_condition_holds [<仓根>]   两个驱动文件头 run-condition: check 调它（preflight 用 bash -c 在仓根起）；只判配置：
#       开了 GPU 就要在多机配置里写了显卡、点名的卡与双机开关对得上。不连第二台、不碰卡：那两样跑起来才知道，用不上时驱动在运行中退 78。
#   crash_amplification_cards_match_the_hosts <双机 0/1> <显卡配置的值>   双机开着两台都要有点名的卡，双机没开不许点名第二台的卡
#   crash_amplification_failure_kind <日志…>           一趟没跑成，日志里说的是哪一类：device（显卡或显存用不上，含给定状态表的判器借不到显存那一行，运行条件，调用方退 78）、
#       build（这份二进制跑不了这一趟：结论的版本与树对不上、配置开了 GPU 而没编进 gpu 特性，运行条件，调用方退 78）、
#       algorithm（算法上判不了：有状态 GPU 判不了、内核展开超预算、建不出表，调用方退 248），都不是打空串；几类都有时先 device、再 build
#   crash_amplification_exit_code_of_a_failed_run <那一趟的退出码> <日志…>   打调用方该退的码：device 与 build 78、algorithm 248，别的原样
#     （内存包装退的 245 显存不够照原样，出路由 crash_amplification_explain_exit_code 的 245 那一支打）
#   弄坏开关读 CRASH_AMPLIFICATION_ITEM_BREAK（开关表与该红的格在 scripts/crash/crash-amplification-item.sh 文件头）：
#   accept-missing-peer-cards、accept-peer-cards-without-across、accept-missing-card-list、device-unusable-as-run-failure、algorithm-refusal-as-run-failure、
#   version-differs-as-run-failure、gpu-feature-missing-as-run-failure、library-in-the-old-directory。
#   given-states-run-condition-as-run-failure（给定状态表借不到显存那一行不认、不退 78）只由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红
#   （红在哪一格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#   其中 device-unusable-as-run-failure、algorithm-refusal-as-run-failure 打开时 scripts/crash/crash-amplification-two-hosts.sh --selftest 也红
#   （红在哪几格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#   走 GPU 的计算类那几处的开关同样读 CRASH_AMPLIFICATION_ITEM_BREAK，只由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红
#   （开关表与该红的格在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）：
#   inherited-classes-kept、classes-sent-again、configuration-from-tree-root。
#   配置那几处的开关 cards-read-from-cards-file 由 scripts/crash/crash-amplification-item.sh --selftest 证红（表与该红的格在那份文件头）。
#
# 弄坏开关轮（崩溃放量入口与双机驱动的 --selftest 末尾共用；.claude/rules/test-script-discipline.md「测试脚本自己也要被测试证明有效」：
# 复跑的一方逐个打开开关、看它红在点名的那一项）：
#   crash_amplification_read_switch_table <脚本文件> <变量名…>
#       读那份文件头的开关表（第一行代码之前的注释块），一个开关打一行「<变量>=<开关名> <TAB> <点名的格，空格隔开>」，照表里的次序。表的写法（B 写法，
#       scripts/mutation/mutate.sh 文件头「弄坏开关」一段是样板）：一行「#   <变量>=<开关名>  <弄坏哪一处>」（变量是给的变量名之一），下一行起以「#       」
#       开头写点名的格（、隔开；括号里是说明与失败行的句首，不读，括号外的每一段都算一格）。「第 N 格」读成 N（崩溃放量入口的格是编号的）；
#       写错的格名照样读进来，开关轮比格时红。崩溃放量入口与双机驱动的 --selftest 都经它读表。
#   crash_amplification_switch_wheel <放输出的目录> <取格的正则> <只比退出码 0/1> -- <自证命令…>
#       调用方先从自己文件头的开关表现读出开关（crash_amplification_read_switch_table，不在代码里另抄名单），填两个同序数组：switch_wheel_assignments 每个开关一项「<变量>=<开关名>」；
#       switch_wheel_expected 那个开关点名的格，空白隔开，写成「-」表示表里没列它、打开后应当退 0、一格不红。
#       每个开关一个子进程：先 env -u 清掉数组里出现过的每个变量、只设这一个，再设 CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER（指到这一轮写的标记文件；
#       调用方用 crash_amplification_switch_wheel_inner 认它，认出来就不再起开关轮），跑自证命令，标准输出与标准错误并进 <目录>/switch-wheel-<序号>.out，
#       一批一批起、逐个 wait 收退出码。并行度：CRASH_AMPLIFICATION_SELFTEST_JOBS（正整数），没给取 SINGLEFS_THREAD_CAP（scripts/hosts/capped.sh 设的），再没给取登记的重活默认并行度（.claude/heavy-thread-default）。
#       对上的逐个打 ·（标准输出），对不上的逐个打 ✗（标准错误，出路由调用方的汇总句给）；交回之后 switch_wheel_failures 是对不上的处数
#       （含派出收回对不上）、switch_wheel_collected 是收回的个数。
#   crash_amplification_switch_wheel_verdict <点名的格> <退出码> <输出文件> <取格的正则> <只比退出码 0/1>
#       一个开关对不对得上：打实测红的格（取格的正则是 sed -E 的写法、一组括号括住格名；去重、sort -V 排好、空格隔开）。点名了格的要退 1、
#       红的格与点名的逐格相等；「-」的要退 0、一格不红；点名的格是空串（表里那一行没写格）一律对不上。
#       第五个参数是 1 时不比格、只比退出码：调用方的弄坏开关用（双机驱动的 wheel-ignores-cells），由它自证的 switch-wheel-judges 格拿造出来的输出证红。
#
# 线程数的缺省（用户 2026-10-12 定：所有的默认都跑 31 线程，双机各台各 31，机器忙时固定、不看忙闲）：
#   crash_amplification_default_threads 打登记的重活默认并行度（.claude/heavy-thread-default，经 scripts/hosts/heavy-threads.sh；SINGLEFS_THREAD_CAP 设了取它）。
#   本机跑的那一步（没给 --threads）、本机只编那一步（crash_amplification_run_on_this_host）与双机驱动送第二台的那两趟（排计划、分份）都取它：
#   crash_amplification_threads_now <local|peer> [<显式给的线程数>]：调用方显式给了就用它（要是正整数；本机只编那一步读环境变量
#   CRASH_AMPLIFICATION_BUILD_THREADS，第二台那两趟读 CRASH_AMPLIFICATION_PEER_THREADS），没给取上面那个数。显式给的不是正整数打 ✗ 与出路、交回 1：
#   本机只编那一步交回 78、不起 cargo；双机驱动退 78、第二台那一趟不起。
#   弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=threads-default-cores：缺省取本机核数减一（改前本机跑的那一步的样子），由 scripts/crash/crash-amplification-item.sh --selftest 的第 27 格证红。
#   弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=build-threads-fixed：本机只编那一步照改前写死 6 路，由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红
#   （红在哪几格写在那份文件头「共用库 crash-amplification-lib.sh 的开关」一段）。
#
# 被停（crash_amplification_run_on_this_host；欠账 C797、C801）：本机那一趟的编、跑两步各放后台起、进程号记下、wait 它；收到 TERM 只把 TERM 转给记下的那一个进程号
#   （内存包装；不按名字找，不遍历它的子孙：包装收到 TERM 自己 systemctl --user stop 它开的 scope，scope 里的 cargo 随之停，见 scripts/hosts/run-with-memory-cap.sh 文件头「起了 scope 之后包装被 TERM / INT 停」）、不再起下一步、
#   等记下的进程号退出（包装退出时 scope 已停，本机库的锁随 cargo 退出放掉；等的上限 CRASH_AMPLIFICATION_TERM_WAIT_SECONDS，默认 100 秒，没退完打 ✗ 与出路）、
#   放回原来的 TERM 处理，再对自己发 TERM：调用方没设 TERM 处理（崩溃放量分项驱动的主线、放后台的子 shell）就此按被停退 143；设了（双机驱动的主线）它的处理跑过之后这一趟交回 143。
#   没被停时退出码照旧转出：编的那一步退非 0 就不起跑的那一步，交回的是那一步的退出码。日志仍是一份：编的输出在前、跑的输出在后。
#
# 自证（单独执行 bash scripts/crash/crash-amplification-lib.sh --selftest；被 source 时不跑）：调真的 crash_amplification_run_on_this_host，
#   仓根换成临时目录下的假树，内存包装像真的那样收到 TERM 停自己起的子命令（假 scope）再退 143，cargo 是 PATH 前面的假脚本（占着一个文件当「库的锁」、收到 TERM 慢一点放掉），不起重活；
#   失败行是「✗ 自证 <格名>：…」。格：term-reaches-cargo（放后台的子 shell 里被停）、term-reaches-cargo-in-foreground（调用方的 shell 里没设 TERM 处理）、
#   term-returns-143-under-caller-trap（调用方设了 TERM 处理）、term-during-build-skips-test、exit-code-passes-through、build-failure-skips-test、caller-term-trap-kept。
#   末尾一轮（开关轮）从下面的表现读开关与点名的格，逐个打开再跑一遍 --selftest，核红的格与点名的逐格相等（判法是同一份库的 crash_amplification_switch_wheel）。
# 弄坏开关 CRASH_AMPLIFICATION_LIB_BREAK=<项>（几个用逗号连；只给 --selftest 证红用）。写法同上面「弄坏开关轮」一段：一行「#   <变量>=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格。
#   CRASH_AMPLIFICATION_LIB_BREAK=term-not-forwarded  收到 TERM 只记一笔、不转给记下的那一趟（cargo 收不到，库的锁一直占到它自己跑完）
#       term-reaches-cargo、term-reaches-cargo-in-foreground、term-returns-143-under-caller-trap、term-during-build-skips-test
#   CRASH_AMPLIFICATION_LIB_BREAK=term-not-waited  转完 TERM 不等它退出、不等库的锁放掉，当场按被停退
#       term-reaches-cargo、term-reaches-cargo-in-foreground、term-returns-143-under-caller-trap
#   CRASH_AMPLIFICATION_LIB_BREAK=term-trap-not-restored  那一趟返回之后不放回调用方原来的 TERM 处理
#       term-returns-143-under-caller-trap、caller-term-trap-kept
#
# 配置（用户 2026-10-10 定：多机配置合成一份，不零散配置；判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J5、J11）：
#   只读一份多机配置 ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}（git 忽略、这台机器一份；模板是仓根的 multi-host.env.example）。
#   主工作树的根按 git common-dir 取（crash_amplification_multi_host_configuration_file，找法与 scripts/crash/crash-point-replay-shard-configuration-check.sh、
#   scripts/crash/gpu-quiesce-run.sh 相同）：在 HEAD + 暂存区的临时 worktree 里跑也读主工作树那一份；树不是 git 工作树时退回 <树根>/multi-host.env。
#   开关 ENABLE_GPU、ENABLE_ACROSS_MACHINES 的判法不改（没写或写 0 算关，键的定义只在 crash-point-replay-shard-configuration-check.sh）。
#   显卡的两个键也写在这一份里：GPU_CARDS_BY_PRIORITY（不清场时显卡的先后与每张卡的显存额度，读法在
#   crates/singlefs-checker-tier/src/gpu_card_configuration.rs 的 parse_cards_by_priority）、GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（清场之后的先后与额度，同一种写法；
#   全量开跑就清场，用的是这一份）。挑中的那一份显卡先后经环境变量 GPU_CARDS_BY_PRIORITY 交给用例（本机与第二台同一份，各取点名给自己的卡）。
#   私有显卡配置（SINGLEFS_GPU_CARDS_CONFIG 指的那一份、~/.config/singlefs/gpu-cards.env）退役：这里不读它，本机那一趟起 cargo 前 env -u 清掉这个变量，
#   用例也就读不到它；第二台那几趟本来就只看命令里送过去的环境。
# 走 GPU 的计算类：不再有正表，也不再经环境变量交。开了 GPU（ENABLE_GPU=1）引擎就三类全走 GPU，它不读 GPU_COMPUTATION_CLASSES
#   （用例经 CrashAmplificationEnvironment::cleared_then_set 设一个引擎一趟不读的变量会 panic），本机与第二台都不送它；
#   本机那一趟起 cargo 前照样用 env -u 清掉调用方环境里漏进来的同名变量。弄坏开关 classes-sent-again：开了 GPU 照改前往本机那一趟送三类。
#   不走 GPU 的那几截计算登记在仓内豁免表 .claude/gate.d/gpu-exemptions.tsv（每行用户点头），门禁 code-source-discipline 的 gpu-exemptions 格核它与代码标记逐项对上。
#   本机那一趟另交 SINGLEFS_MULTI_HOST_CONFIG=<这一份的绝对路径>：用例不设它时按自己的树根找 multi-host.env，在 worktree 里会找不到主工作树那一份（第二台不交，它的树副本不带配置）。
# 判定库（分项驱动与双机驱动共用；崩溃放量入口与崩溃点重放入口经它们用）：没给 --library 时用 <家目录>/<CRASH_LIBRARY_DIRECTORY_NAME>，
#   家目录是 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}，目录名是 crash-state-verdicts（一份库，几类测试同一份，
#   目录名不带类名、不带存法名；与 crates/singlefs-checker-tier/src/crash_library.rs 的 CRASH_LIBRARY_DIRECTORY_NAME 逐字相同）；
#   crash_amplification_default_library <家目录> 打这个路径。块键带名字的旧库（<家目录>/library、library-by-conclusion-kind 这几份）引擎开库就拒、不迁移；
#   旧目录这里不读、不删（由主 agent 删）。弄坏开关 library-in-the-old-directory：照改前打 <家目录>/library，由 scripts/crash/crash-amplification-item.sh --selftest 证红（表与该红的格在那份文件头）。

source "$(dirname "${BASH_SOURCE[0]}")/../hosts/heavy-threads.sh"
CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include"  # librocksdb-sys 的 bindgen 找 stdbool.h；哪个版本的目录在就用哪个，不在的 -I 无害
CRASH_LIBRARY_DIRECTORY_NAME="crash-state-verdicts"  # 那一份判定库在家目录下的目录名（文件头「判定库」）：一份库、目录名固定一处；块键带名字的旧库引擎开库就拒
# 给定状态表的判器借不到显存时引擎打的那一行的行首（crates/singlefs-checker-tier/src/crash_amplification.rs 的 GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX）：
# 运行条件不满足，调用方退 78；crash_amplification_failure_kind 的 device 那一组与崩溃点重放入口 scripts/crash/crash-point-replay.sh 的快档都照这一份认
CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX="CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET"

crash_amplification_default_library() { # <家目录>：打没给 --library 时的判定库路径（文件头「判定库」）
  if crash_amplification_item_break_is library-in-the-old-directory; then printf '%s/library\n' "$1"; return 0; fi
  printf '%s/%s\n' "$1" "$CRASH_LIBRARY_DIRECTORY_NAME"
}

crash_amplification_configuration_value() { # <配置文件> <键>：打那个键的值（去掉两头成对的一层引号）；文件不在、键没写打空串
  [[ -f "$1" ]] || return 0
  python3 - "$1" "$2" <<'PYTHON'
import sys
path, key = sys.argv[1], sys.argv[2]
for line in open(path, encoding="utf-8"):
    line = line.strip()
    if not line.startswith(key + "="):
        continue
    value = line[len(key) + 1:].strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
        value = value[1:-1]
    print(value)
    break
PYTHON
}

# 多机配置在哪（文件头「配置」）：打路径。SINGLEFS_MULTI_HOST_CONFIG 设了照它；没设取 <树根> 的 git common-dir 的上一层（主工作树的根）下的 multi-host.env，
# 取不到 common-dir（不是 git 工作树）退回 <树根>/multi-host.env。文件在不在不判（不在就是两个开关都关、一张卡都没写）。
# 弄坏开关 configuration-from-tree-root：照改前按 <树根>/multi-host.env 找（在 worktree 里读不到主工作树那一份）。
crash_amplification_multi_host_configuration_file() { # <树根>
  local root="$1" common_directory=""
  if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then printf '%s\n' "$SINGLEFS_MULTI_HOST_CONFIG"; return 0; fi
  if ! crash_amplification_item_break_is configuration-from-tree-root \
     && common_directory="$(git -C "$root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    printf '%s/multi-host.env\n' "$(dirname "$common_directory")"
    return 0
  fi
  printf '%s/multi-host.env\n' "$root"
}

# 显卡的两个键从哪一份读（文件头「配置」：多机配置那一份）；弄坏开关 cards-read-from-cards-file 下照改前读退役的私有显卡配置
crash_amplification_cards_source_file() { # <多机配置的路径>
  if crash_amplification_item_break_is cards-read-from-cards-file; then
    printf '%s\n' "${SINGLEFS_GPU_CARDS_CONFIG:-${XDG_CONFIG_HOME:-$HOME/.config}/singlefs/gpu-cards.env}"
    return 0
  fi
  printf '%s\n' "$1"
}

crash_amplification_prepare() {
  configuration_file="$(crash_amplification_multi_host_configuration_file "$repository_root")"
  gpu_switch=0; across_machines_switch=0; features="verdict-store"
  [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)" == 1 ]] && gpu_switch=1
  [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_ACROSS_MACHINES)" == 1 ]] && across_machines_switch=1
  if [[ "${library_is_temporary:-0}" == 1 ]]; then gpu_switch=0; fi  # 自证不上卡
  (( gpu_switch )) && features="verdict-store,gpu"
  local cards_file
  cards_file="$(crash_amplification_cards_source_file "$configuration_file")"
  gpu_cards_by_priority="$(crash_amplification_configuration_value "$cards_file" GPU_CARDS_BY_PRIORITY)"
  gpu_cards_by_priority_when_quiesced="$(crash_amplification_configuration_value "$cards_file" GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED)"
  # 清过场：用清场之后那一份；开了 GPU、或写了不清场那一份显卡先后（要用卡）却没写清场之后那一份，交回 2。没开 GPU 也没写卡的（只清场跑 CPU）照跑
  if [[ "${quiesced:-0}" == 1 ]] && { (( gpu_switch )) || [[ -n "$gpu_cards_by_priority" ]]; }; then
    if [[ -z "$gpu_cards_by_priority_when_quiesced" ]]; then
      echo "  ✗ 说是清过场，多机配置里却没写 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（多机配置：$configuration_file）" >&2
      echo "  → 怎么办：修好设备，或改配置：照仓根 multi-host.env.example 在多机配置里写清场之后的显卡先后与额度" >&2
      return 2
    fi
    gpu_cards_by_priority="$gpu_cards_by_priority_when_quiesced"
  fi
  common_environment=(SINGLEFS_HEAVY_TESTS=user-request "BINDGEN_EXTRA_CLANG_ARGS=$CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS")
  [[ -n "${expand_up_to:-}" ]] && common_environment+=("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=$expand_up_to")
  local cores="${threads:-}"
  [[ -n "$cores" ]] || cores="$(crash_amplification_default_threads)"
  common_environment+=("SINGLEFS_CRASH_AMPLIFICATION_THREADS=$cores")
  cargo_build_arguments=(test --release -p singlefs-checker-tier --features "$features" --test "$test_target" --no-run)
  cargo_test_arguments=(test --release -p singlefs-checker-tier --features "$features" --test "$test_target" -- --nocapture)
  # 点名的用例可能标着 #[ignore]（崩溃点重放的全量用例都是）：不带 --include-ignored 它一条都不跑、退 0，录入段就空着
  # （弄坏开关 include-ignored-dropped 照改前只带 --exact，由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红）
  if [[ -n "${test_filter:-}" ]]; then
    if crash_amplification_item_break_is include-ignored-dropped; then
      cargo_test_arguments+=(--exact "$test_filter")
    else
      cargo_test_arguments+=(--include-ignored --exact "$test_filter")
    fi
  fi
  return 0
}

# 交给本机那一趟的环境（env 的参数）放进数组 this_host_environment：先 -u 清掉调用方环境里漏进来的 GPU_COMPUTATION_CLASSES（引擎不读它，文件头「配置」）
# 与退役的 SINGLEFS_GPU_CARDS_CONFIG，再是 common_environment、附加环境、份与库、多机配置的路径（文件在才交，文件头「配置」），挑中的那份显卡先后与额度（空的不交）。
crash_amplification_environment_for_this_host() { # <share> <库目录> [<附加环境 键=值>…]
  local share="$1" library="$2"
  shift 2
  this_host_environment=(-u SINGLEFS_GPU_CARDS_CONFIG)
  crash_amplification_item_break_is inherited-classes-kept || this_host_environment+=(-u GPU_COMPUTATION_CLASSES)
  this_host_environment+=("${common_environment[@]}" "$@" "SINGLEFS_CRASH_AMPLIFICATION_SHARE=$share" "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=$library")
  [[ -n "${configuration_file:-}" && -f "${configuration_file:-}" ]] && this_host_environment+=("SINGLEFS_MULTI_HOST_CONFIG=$configuration_file")
  [[ -n "${gpu_cards_by_priority:-}" ]] && this_host_environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_by_priority")
  if (( ${gpu_switch:-0} )) && crash_amplification_item_break_is classes-sent-again; then
    this_host_environment+=("GPU_COMPUTATION_CLASSES=gpu_unit_checks crash_verify_gpu crash_judge_gpu")   # 弄坏开关（文件头「配置」）：改前送的那三类
  fi
  return 0
}

# 没给线程数时一台机器用几个核（文件头「线程数的缺省」）：登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它）；
# 弄坏开关 threads-default-cores 下是改前本机跑的那一步的样子（核数减一）
crash_amplification_default_threads() {
  if crash_amplification_item_break_is threads-default-cores; then
    echo $(( $(nproc) > 1 ? $(nproc) - 1 : 1 ))
    return 0
  fi
  heavy_thread_default
}

# 那一台起 capped.sh 的线程数（文件头「线程数的缺省」）：打一个正整数；显式给的不是正整数打 ✗ 与出路、交回 1
crash_amplification_threads_now() { # <local|peer> [<显式给的线程数>]
  local host="$1" given="${2:-}" name=本机
  [[ "$host" == peer ]] && name=第二台
  if [[ -z "$given" ]]; then crash_amplification_default_threads; return 0; fi
  if [[ "$given" =~ ^[1-9][0-9]*$ ]]; then echo "$given"; return 0; fi
  echo "  ✗ 显式给的${name}线程数「$given」不是正整数" >&2
  echo "  → 怎么办：写成正整数，或不给、取登记的重活默认并行度（crash-amplification-lib.sh 文件头「线程数的缺省」）" >&2
  return 1
}

# 本机的一趟：先只编（线程数同「线程数的缺省」，不看忙闲：librocksdb-sys 的 C++ 按并行度吃内存，由内存包装的上限兜着），再跑；
# 跑的那一步用 threads 个核，没给就是登记的重活默认并行度（用户 2026-10-12 定：所有的默认都跑 31 线程，机器忙时固定 31、不看忙闲）。
# 只编那一步的线程数算不出：打 ✗ 与出路（标准错误，同一句写进日志），交回 78、不起 cargo。
# 要这一趟中途停下：附加环境里给 SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=<文件>，建出那个文件它就不再领新的块、手里那一块判完写进库、
# 打一行汇总退 0；判过的块在库里，下一趟接着判。
#
# 被 TERM 停（文件头「被停」）：编与跑两步各放后台起（起法见 crash_amplification_run_step）、进程号记下、wait 它；收到 TERM 时 trap 当场就跑——
# 只把 TERM 转给记下的那一个进程号（内存包装：它收到 TERM 自己停它开的 scope，cargo 随之停）、不再起下一步、等记下的那个进程号退出（库的锁随 cargo 退出放掉）、
# 放回原来的 TERM 处理，
# 再对自己发 TERM：调用方没设 TERM 处理的（崩溃放量分项驱动的主线、双机驱动放后台的本机那一趟）就此按被停退 143，设了的（双机驱动主线）它的处理跑过之后交回 143。
# 弄坏开关 CRASH_AMPLIFICATION_LIB_BREAK（文件头「弄坏开关」）：term-not-forwarded、term-not-waited、term-trap-not-restored。
crash_amplification_lib_break_is() { [[ ",${CRASH_AMPLIFICATION_LIB_BREAK:-}," == *",$1,"* ]]; }

crash_amplification_process_is_running() { # <进程号>：在跑（没有、或已是僵尸都算没在跑）交回 0
  local process_status=""
  { read -r process_status <"/proc/$1/stat"; } 2>/dev/null || return 1
  process_status="${process_status##*) }"   # 进程名可以带空格与括号：取最后一个「) 」之后，第一段是状态
  [[ "${process_status%% *}" != Z ]]
}

# 记下的进程号（crash_amplification_own_run_pid）是这个 shell 的子进程才发（收过之后进程号可能被别的进程重用），只给它这一个发 TERM（内存包装）；
# 它记进 crash_amplification_term_forwarded，等它退出时用
crash_amplification_forward_term_to_own_run() {
  local root="$crash_amplification_own_run_pid" process_status=""
  [[ -n "$root" ]] || return 0
  { read -r process_status <"/proc/$root/stat"; } 2>/dev/null || return 0
  process_status="${process_status##*) }"
  process_status="${process_status#* }"
  [[ "${process_status%% *}" == "$BASHPID" ]] || return 0
  kill -TERM "$root" 2>/dev/null
  crash_amplification_term_forwarded+=("$root")
  return 0
}

crash_amplification_on_term() { # TERM 的 trap：记一笔、打断正在等的 wait、转给记下的那一趟
  crash_amplification_term_received=1
  crash_amplification_term_interrupted=1
  crash_amplification_lib_break_is term-not-forwarded || crash_amplification_forward_term_to_own_run
}

# 起一步：放后台（子 shell 里先进仓根再 exec 那条命令，cd 不成就退 1、不起命令），进程号记在 crash_amplification_own_run_pid，wait 它；
# 标准输出与标准错误追加进 <日志>（调用方先清空）。交回那一步的退出码；wait 被 TERM 的 trap 打断就接着等（命令没退之前不放手）。
crash_amplification_run_step() { # <日志> <命令与参数…>
  local log="$1" step_status=0 reaped=""
  shift
  ( cd "$repository_root" && exec "$@" ) >>"$log" 2>&1 &
  crash_amplification_own_run_pid=$!
  # 记下进程号之前 TERM 就到了（trap 跑时没有可转的）：补发
  if (( crash_amplification_term_received )); then crash_amplification_forward_term_to_own_run; fi
  while :; do
    crash_amplification_term_interrupted=0
    if wait "$crash_amplification_own_run_pid"; then step_status=0; else step_status=$?; fi
    # 接着等时它已经收过（wait 退 127）就留上一次的码
    if [[ -n "$reaped" && "$step_status" == 127 ]]; then step_status="$reaped"; break; fi
    reaped="$step_status"
    (( crash_amplification_term_interrupted )) || break
    ! crash_amplification_lib_break_is term-not-waited || break
  done
  crash_amplification_own_run_pid=""
  return "$step_status"
}

crash_amplification_wait_until_the_forwarded_are_gone() { # 转过 TERM 的那个进程号（内存包装）退出（它退出时 scope 已停，库的锁随 cargo 退出放掉）交回 0；等满 CRASH_AMPLIFICATION_TERM_WAIT_SECONDS（默认 100，在 detached-run.sh 的 180 秒停机时限之内）还有在跑的，打 ✗ 与出路、交回 1
  local limit="${CRASH_AMPLIFICATION_TERM_WAIT_SECONDS:-100}" ticks=0 pid alive=()
  [[ "$limit" =~ ^[1-9][0-9]*$ ]] || limit=100
  while :; do
    alive=()
    for pid in ${crash_amplification_term_forwarded[@]+"${crash_amplification_term_forwarded[@]}"}; do
      ! crash_amplification_process_is_running "$pid" || alive+=("$pid")
    done
    (( ${#alive[@]} == 0 )) && return 0
    if (( ticks >= limit * 10 )); then
      echo "  ✗ 收到 TERM 转给本机那一趟的内存包装后等了 $limit 秒，进程号 ${alive[*]} 还在跑，本机库的锁可能还没放" >&2
      echo "  → 怎么办：这个包装是不是没停它的 scope（ps -o pid,ppid,args -p ${alive[*]}；systemctl --user list-units 'singlefs-memory-cap-*' 看它开的 scope 还在不在）；确认是这一趟自己起的之后 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号> 停它，再重跑同一条命令接着判" >&2
      return 1
    fi
    sleep 0.1
    ticks=$((ticks + 1))
  done
}

crash_amplification_restore_term_trap() { # <起头记下的 trap -p TERM 输出，没有是空串>
  crash_amplification_lib_break_is term-trap-not-restored && return 0
  if [[ -n "$1" ]]; then eval "$1"; else trap - TERM; fi
}

crash_amplification_run_on_this_host() { # <share> <库目录> <日志> [<附加环境 键=值>…]
  local share="$1" library="$2" log="$3"
  shift 3
  local cores="${threads:-}"
  [[ -n "$cores" ]] || cores="$(crash_amplification_default_threads)"
  local build_threads=""
  if crash_amplification_item_break_is build-threads-fixed; then
    build_threads=6
  elif ! build_threads="$(crash_amplification_threads_now local "${CRASH_AMPLIFICATION_BUILD_THREADS:-}" 2>"$log")"; then
    cat -- "$log" >&2
    return 78
  fi
  crash_amplification_environment_for_this_host "$share" "$library" "$@"
  local local_environment=("${this_host_environment[@]}")
  local step_status=0 previous_term_trap=""
  crash_amplification_term_received=0; crash_amplification_term_interrupted=0; crash_amplification_own_run_pid=""; crash_amplification_term_forwarded=()
  # 原来的 TERM 处理：放后台的子 shell（BASHPID 不是 $$）里 trap -p 显示的是父 shell 的，不是它自己的，那里原来就是默认
  if [[ "$BASHPID" == "$$" ]]; then previous_term_trap="$(trap -p TERM)"; fi
  : >"$log"
  trap 'crash_amplification_on_term' TERM
  crash_amplification_run_step "$log" env "${local_environment[@]}" bash scripts/hosts/run-with-memory-cap.sh "$memory_cap" bash scripts/hosts/capped.sh "$build_threads" cargo "${cargo_build_arguments[@]}"; step_status=$?
  if (( step_status == 0 && ! crash_amplification_term_received )); then
    crash_amplification_run_step "$log" env "${local_environment[@]}" bash scripts/hosts/run-with-memory-cap.sh "$memory_cap" bash scripts/hosts/capped.sh "$cores" cargo "${cargo_test_arguments[@]}"; step_status=$?
  fi
  if (( crash_amplification_term_received )); then
    echo "  · 收到 TERM：已转给本机那一趟（记下的进程号，内存包装），等它退出、库的锁放掉后按被停退 143" >&2
    crash_amplification_lib_break_is term-not-waited || crash_amplification_wait_until_the_forwarded_are_gone || true
    crash_amplification_restore_term_trap "$previous_term_trap"
    kill -TERM "$BASHPID"   # 调用方没设 TERM 处理就在这里按被停退；设了的，它的处理跑过之后走下面的交回
    return 143
  fi
  crash_amplification_restore_term_trap "$previous_term_trap"
  return "$step_status"
}

# 录入段（用户 2026-10-11 定：录入段在 CPU 上跑数据、写进库；判段只从库里读，段与段之间不在内存里交数据）：本机起一趟
# SINGLEFS_CRASH_AMPLIFICATION_STAGE=record，日志写 <日志>；跑成而且打了录入行交回 0 并把那几行转打出来。录入行两种都算：崩溃放量的分项打
# CRASH_AMPLIFICATION_RECORDED（crates/singlefs-checker-tier/src/crash_amplification.rs），崩溃枚举用例经判定库两段跑的（崩溃点重放双机那几条）打
# CRASH_LIBRARY_RECORDED（crates/singlefs-checker-tier/src/crash_library.rs 的 record_the_item）；弄坏开关 library-marker-not-accepted 照改前只认前一种，
# 由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红。
# 之后的每一趟照 SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge 起（调用方 export）。退了非 0 交回照日志分类之后该退的码；退 0 而一行都没打交回 1。
# 份：不给是 0/1（整份）。引擎录入段带 SHARE=k/n 时，清单、流程、事实表与判表（配了 GPU）照旧整条录（录入键不含份，crates/singlefs-checker-tier/src/crash_state_enumeration.rs
#   的 recording_inputs_digest），只有观察者的块按份筛（record_the_observers_of_the_flow 的 is_mine，与判段同一个 blocks_of_this_host：份加
#   SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS、按块序号平滑轮转）。所以两台观察者录入各录各的那一份时，本机录入段给 0/2，附加环境里带上判段也要带的权重（双机驱动
#   crash-amplification-two-hosts.sh 的分着录），录入段与判段的分法才是同一个；判段读到录入段没录的观察者块会报「先跑录入段」。
crash_amplification_record_on_this_host() { # <库目录> <日志> [<份> [<附加环境 键=值>…]]
  local library="$1" log="$2" share="${3:-0/1}" record_exit exit_code recorded_line_pattern
  shift $(( $# < 3 ? $# : 3 ))
  crash_amplification_run_on_this_host "$share" "$library" "$log" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "$@"; record_exit=$?
  if [[ "$record_exit" != 0 ]]; then
    exit_code="$(crash_amplification_exit_code_of_a_failed_run "$record_exit" "$log")"
    echo "  ✗ 录入段（SINGLEFS_CRASH_AMPLIFICATION_STAGE=record）退 $record_exit；日志 $log" >&2
    echo "  → 怎么办：看日志里 panicked 与 run-with-memory-cap 那几行，修好再重跑同一条命令；录入段没跑成，判段一趟都不起" >&2
    return "$exit_code"
  fi
  recorded_line_pattern='^(CRASH_AMPLIFICATION_RECORDED|CRASH_LIBRARY_RECORDED) '
  if crash_amplification_item_break_is library-marker-not-accepted; then
    recorded_line_pattern='^CRASH_AMPLIFICATION_RECORDED '
  fi
  if ! grep -hE "$recorded_line_pattern" "$log"; then
    echo "  ✗ 录入段退 0，日志 $log 里却一行 CRASH_AMPLIFICATION_RECORDED 或 CRASH_LIBRARY_RECORDED 都没有（引擎没跑到录入段，或用例没经 run_from_environment 起）" >&2
    echo "  → 怎么办：看那份日志跑的是不是这一项的用例、用例有没有在只跑录入段的那一趟提早返回" >&2
    return 1
  fi
  return 0
}

# 一台机器判这一趟的总速度（态/秒，取整）：各条流要判的状态加起来 ÷ 各条流估的用时加起来。一条用例里几条流各打一行，
# 各行的 states_per_second_on_this_host 是各自那条流的速度，直接相加没有意义（五条流各每秒 50 万，合起来仍是每秒 50 万上下）。
# 一行都没有打空串；估的用时合计是 0（没有一条流量出速度）打 0。
crash_amplification_placement_rate() { # <日志>
  python3 - "$1" <<'PYTHON'
import re, sys
pending, seconds, lines = 0, 0, 0
for line in open(sys.argv[1], encoding="utf-8", errors="replace"):
    if not line.startswith("CRASH_AMPLIFICATION_PLACEMENT "):
        continue
    lines += 1
    fields = dict(re.findall(r"(\w+)=([0-9]+)(?= |$)", line.strip()))
    pending += int(fields.get("pending_states", 0))
    seconds += int(fields.get("estimated_seconds_on_this_host", 0))
if lines:
    print(pending // seconds if seconds else 0)
PYTHON
}

# 日志里 CRASH_AMPLIFICATION_PLACEMENT 行某个字段的合计（一条用例里几条流各打一行）；一行都没有打空串。
crash_amplification_placement_total() { # <日志> <字段名>
  python3 - "$1" "$2" <<'PYTHON'
import re, sys
log, field = sys.argv[1], sys.argv[2]
total, lines = 0, 0
for line in open(log, encoding="utf-8", errors="replace"):
    if not line.startswith("CRASH_AMPLIFICATION_PLACEMENT "):
        continue
    lines += 1
    found = re.search(r"(?:^| )" + re.escape(field) + r"=([0-9]+)(?: |$)", line.strip())
    if found:
        total += int(found.group(1))
if lines:
    print(total)
PYTHON
}

crash_amplification_item_break_is() { [[ ",${CRASH_AMPLIFICATION_ITEM_BREAK:-}," == *",$1,"* ]]; }

# 显卡配置与双机开关对不对得上：双机开着两台都要有点名的卡，双机没开不许点名第二台的卡。对得上交回 0，对不上打原因与出路、交回 1
crash_amplification_cards_match_the_hosts() { # <双机开关 0/1> <显卡配置的值>
  local across="$1" cards="$2" names_local=0 names_peer=0 entry
  for entry in $cards; do
    [[ "$entry" == local:* ]] && names_local=1
    [[ "$entry" == peer:* ]] && names_peer=1
  done
  if [[ "$across" == 1 ]] && (( ! names_local || ! names_peer )) && ! crash_amplification_item_break_is accept-missing-peer-cards; then
    echo "  ✗ 配置开了双机与 GPU，显卡先后「$cards」却没有两台各点名至少一张卡" >&2
    echo "  → 怎么办：修好设备，或改配置：照仓根 multi-host.env.example 在多机配置里给两台各点名卡（local:<序号>:<额度> 与 peer:<序号>:<额度>）；不想用第二台就把多机配置的 ENABLE_ACROSS_MACHINES 改成 0" >&2
    return 1
  fi
  if [[ "$across" != 1 ]] && (( names_peer )) && ! crash_amplification_item_break_is accept-peer-cards-without-across; then
    echo "  ✗ 多机配置没开双机，显卡先后「$cards」却点名了第二台的卡：点名的卡用不上" >&2
    echo "  → 怎么办：修好设备，或改配置：要用第二台就把多机配置的 ENABLE_ACROSS_MACHINES 改成 1；不用就从多机配置的显卡先后里删掉 peer: 那几项" >&2
    return 1
  fi
  return 0
}

# 运行条件：配置里开了的硬件在配置上对不对得上。开关与显卡都读多机配置（文件头「配置」；CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机> 只供测试，给了照它），
# 显卡读两份先后（不清场的与清场之后的，这里不知道调用方清没清场，写了的每一份都要对得上；CRASH_AMPLIFICATION_ITEM_CARDS 只供测试，给了只判它）。
crash_amplification_run_condition_holds() { # [<仓根>]：对得上交回 0；对不上打原因与出路、交回 1
  local root="${1:-$PWD}" configuration gpu=0 across=0 cards_file key list lists=()
  configuration="$(crash_amplification_multi_host_configuration_file "$root")"
  [[ "$(crash_amplification_configuration_value "$configuration" ENABLE_GPU)" == 1 ]] && gpu=1
  [[ "$(crash_amplification_configuration_value "$configuration" ENABLE_ACROSS_MACHINES)" == 1 ]] && across=1
  if [[ -n "${CRASH_AMPLIFICATION_ITEM_SWITCHES:-}" ]]; then IFS=, read -r gpu across <<<"$CRASH_AMPLIFICATION_ITEM_SWITCHES"; fi
  [[ "$gpu" == 1 ]] || return 0
  cards_file="$(crash_amplification_cards_source_file "$configuration")"
  if [[ -n "${CRASH_AMPLIFICATION_ITEM_CARDS:-}" ]]; then
    lists=("$CRASH_AMPLIFICATION_ITEM_CARDS")
  else
    for key in GPU_CARDS_BY_PRIORITY GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED; do
      list="$(crash_amplification_configuration_value "$cards_file" "$key")"
      [[ -n "$list" ]] && lists+=("$list")
    done
  fi
  if (( ${#lists[@]} == 0 )) && ! crash_amplification_item_break_is accept-missing-card-list; then
    echo "  ✗ 配置开了 GPU（ENABLE_GPU=1），多机配置里 GPU_CARDS_BY_PRIORITY 与 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 一份都没写（多机配置：$cards_file）" >&2
    echo "  → 怎么办：修好设备，或改配置：照仓根 multi-host.env.example 在多机配置里写每台机器用哪几张卡、各多少显存；不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
    return 1
  fi
  for list in ${lists[@]+"${lists[@]}"}; do
    crash_amplification_cards_match_the_hosts "$across" "$list" || return 1
  done
  return 0
}

# 一趟没跑成，日志里说的是哪一类（判据是用例与判器报错的原话，出处见各串旁边）：
#   device：显卡或显存用不上——GPU_CARD_NOT_USABLE 行（crates/singlefs-checker-tier/src/gpu_unit_checks.rs）、
#     「这台机器却没有一张用得了的卡」「没有一张卡装得下这条流的表」「GPU_CARDS_BY_PRIORITY 里没有」（crates/singlefs-checker-tier/src/crash_amplification.rs）、
#     行首（可带空白）是 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 的那一行（给定状态表的判器起过卡池而借不到显存，引擎打这一行后报错；
#     CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX）
#   build：这份二进制跑不了这一趟——行首是 CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD 的那一行（结论的版本编译时与跑时对不上、
#     整趟不写库；crates/singlefs-checker-tier/src/crash_amplification.rs 的 CONCLUSION_VERSION_NOT_WRITTEN_LINE_PREFIX）、
#     「这份二进制没有编进 gpu 特性」（多机配置开了 GPU 而编的不带 gpu 特性，crash_amplification.rs）
#   algorithm：算法上判不了——「表越过了内核的容量」（有状态 GPU 判不了）、「全部调用展开之后 <数> 个表达式，超过预算」（内核展开超预算，crash_judge_dispatch.rs）、
#     「GPU 判器建不出这条流的表」（crash_amplification.rs）
# 不在的日志跳过；几类都有时先 device、再 build（这两类时那一趟判的不算数）；都不是打空串。
# 弄坏开关 version-differs-as-run-failure、gpu-feature-missing-as-run-failure：build 那两句各自不认（表与该红的格在 crash-amplification-item.sh 文件头）。
# 弄坏开关 given-states-run-condition-as-run-failure：给定状态表借不到显存那一行不认（红在 crash-amplification-two-hosts.sh --selftest 的 local-share-given-states-run-condition）。
crash_amplification_failure_kind() { # <日志…>
  local device_pattern='GPU_CARD_NOT_USABLE|这台机器却没有一张用得了的卡|没有一张卡装得下这条流的表|GPU_CARDS_BY_PRIORITY 里没有'
  local given_states_run_condition_pattern="^[[:space:]]*${CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX}( |\$)"
  local version_differs_pattern='^CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD( |$)'
  local gpu_feature_missing_pattern='这份二进制没有编进 gpu 特性'
  local algorithm_pattern='表越过了内核的容量|全部调用展开之后 [0-9]+ 个表达式，超过预算|GPU 判器建不出这条流的表'
  local logs=() log
  for log in "$@"; do [[ -f "$log" ]] && logs+=("$log"); done
  (( ${#logs[@]} > 0 )) || return 0
  if ! crash_amplification_item_break_is device-unusable-as-run-failure && grep -qE -- "$device_pattern" "${logs[@]}"; then echo device; return 0; fi
  if ! crash_amplification_item_break_is given-states-run-condition-as-run-failure && grep -qE -- "$given_states_run_condition_pattern" "${logs[@]}"; then echo device; return 0; fi
  if ! crash_amplification_item_break_is version-differs-as-run-failure && grep -qE -- "$version_differs_pattern" "${logs[@]}"; then echo build; return 0; fi
  if ! crash_amplification_item_break_is gpu-feature-missing-as-run-failure && grep -qF -- "$gpu_feature_missing_pattern" "${logs[@]}"; then echo build; return 0; fi
  if ! crash_amplification_item_break_is algorithm-refusal-as-run-failure && grep -qE -- "$algorithm_pattern" "${logs[@]}"; then echo algorithm; return 0; fi
  return 0
}

crash_amplification_exit_code_of_a_failed_run() { # <那一趟的退出码> <日志…>：打调用方该退的码
  local run_exit="$1" kind
  shift
  kind="$(crash_amplification_failure_kind "$@")"
  case "$kind" in
    device|build) echo 78 ;;
    algorithm) echo 248 ;;
    *) echo "$run_exit" ;;
  esac
}

crash_amplification_explain_exit_code() { # <调用方要退的码>：78、248 与 245（内存包装认出的显存不够）打对应的出路（标准错误），别的不打
  case "$1" in
    245) echo "  → 怎么办：显存不够（那一趟退了非 0，内存包装认出它起止之间内核报了 NV_ERR_NO_MEMORY，退 245），这一趟的结果不算数：nvidia-smi 看谁在用卡（同一段时间别的进程撞的也算进来），等它跑完、或少给这一趟显存，再重跑同一条命令接着判（判过的块已经在库里）" >&2 ;;
    78) echo "  → 怎么办：修好设备，或改配置：日志里 GPU_CARD_NOT_USABLE 行点名的卡用不了（没有这个序号、空闲显存不够、上下文起不来）、显存装不下这条流的表、给定状态表的判器借不到显存（CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 那一行，等卡上占着显存的跑完再跑），或第二台连不上；查那张卡或那台机器，或改显卡配置与多机配置；日志里有 CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD 那一行的，是二进制与盘上的树对不上、整趟没写库，重编再跑（或在提交那一趟的 worktree 里跑）；有「这份二进制没有编进 gpu 特性」的，带 --features verdict-store,gpu 重编，或把多机配置的 ENABLE_GPU 改成 0（退 78：运行条件不满足）" >&2 ;;
    248) echo "  → 怎么办：算法上判不了（有状态 GPU 判不了、内核展开超预算、建不出表），不是设备的事、也不是红：照日志里 panicked 那几行改判器的表或内核的容量再跑（退 248）" >&2 ;;
  esac
}

# 弄坏开关轮（文件头「弄坏开关轮」一段）
CRASH_AMPLIFICATION_SWITCH_WHEEL_MARK_CONTENT="crash_amplification_switch_wheel 起的子进程"

crash_amplification_read_switch_table() { # <脚本文件> <变量名…>：读文件头的开关表（文件头「弄坏开关轮」一段的写法），一个开关打一行「<变量>=<开关名> <TAB> <点名的格>」
  python3 - "$@" <<'PYTHON'
import re, sys
variables = "|".join(re.escape(name) for name in sys.argv[2:])
start = re.compile(r"^#   (" + variables + r")=([a-z0-9-]+)(\s|$)")
rows, current = [], None
with open(sys.argv[1], encoding="utf-8") as handle:
    for line in handle:
        line = line.rstrip("\n")
        if not line.startswith("#"):
            break
        found = start.match(line)
        if found:
            current = [found.group(1) + "=" + found.group(2), ""]
            rows.append(current)
        elif current is not None and line.startswith("#       "):
            current[1] += " " + line[1:]
        else:
            current = None
for assignment, text in rows:
    previous = None
    while previous != text:
        previous = text
        text = re.sub(r"（[^（）]*）", " ", text)
        text = re.sub(r"\([^()]*\)", " ", text)
    text = re.sub(r"第\s*([0-9]+)\s*格", r" \1 ", text)
    print(assignment + "\t" + " ".join(part for part in re.split(r"[\s、，,；;]+", text) if part))
PYTHON
}

crash_amplification_switch_wheel_inner() { # 这一趟是不是开关轮起的子进程：标记文件在、内容对得上交回 0
  [[ -n "${CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER:-}" && -f "${CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER:-}" ]] \
    && [[ "$(cat -- "$CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER")" == "$CRASH_AMPLIFICATION_SWITCH_WHEEL_MARK_CONTENT" ]]
}

crash_amplification_switch_wheel_verdict() { # <点名的格> <退出码> <输出文件> <取格的正则> <只比退出码 0/1>：打实测红的格，对得上交回 0
  local expected="$1" exit_code="$2" output="$3" pattern="$4" exit_only="$5" observed="" wanted=""
  if [[ -f "$output" ]]; then
    observed="$(sed -nE "s/.*${pattern}.*/\\1/p" -- "$output" | LC_ALL=C sort -V | uniq | tr '\n' ' ')"
  fi
  printf '%s' "${observed% }"
  if [[ "$expected" == - ]]; then
    [[ "$exit_code" == 0 ]] && { [[ "$exit_only" == 1 ]] || [[ -z "$observed" ]]; }
    return
  fi
  wanted="$(tr -s ' \t' '\n\n' <<<"$expected" | sed '/^$/d' | LC_ALL=C sort -V | uniq | tr '\n' ' ')"
  [[ -n "$wanted" && "$exit_code" == 1 ]] || return 1
  [[ "$exit_only" == 1 || "$observed" == "$wanted" ]]
}

crash_amplification_switch_wheel() { # <放输出的目录> <取格的正则> <只比退出码 0/1> -- <自证命令…>（两个数组与交回的变量见文件头）
  local directory="$1" pattern="$2" exit_only="$3" jobs batch index count assignment variable observed shown
  local variables=" " unset_arguments=() pids=() exits=()
  shift 3
  [[ "${1:-}" == -- ]] && shift
  switch_wheel_failures=0; switch_wheel_collected=0
  count="${#switch_wheel_assignments[@]}"
  jobs="${CRASH_AMPLIFICATION_SELFTEST_JOBS:-$(heavy_thread_default)}"
  [[ "$jobs" =~ ^[1-9][0-9]*$ ]] || jobs=1
  for assignment in ${switch_wheel_assignments[@]+"${switch_wheel_assignments[@]}"}; do
    variable="${assignment%%=*}"
    [[ "$variables" == *" $variable "* ]] && continue
    variables+="$variable "; unset_arguments+=(-u "$variable")
  done
  printf '%s\n' "$CRASH_AMPLIFICATION_SWITCH_WHEEL_MARK_CONTENT" >"$directory/switch-wheel-mark"
  for ((batch = 0; batch < count; batch += jobs)); do
    for ((index = batch; index < batch + jobs && index < count; index++)); do
      env "${unset_arguments[@]}" "${switch_wheel_assignments[index]}" CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER="$directory/switch-wheel-mark" "$@" \
        >"$directory/switch-wheel-$index.out" 2>&1 &
      pids[index]=$!
    done
    for ((index = batch; index < batch + jobs && index < count; index++)); do
      wait "${pids[index]}"; exits[index]=$?
    done
  done
  for ((index = 0; index < count; index++)); do
    [[ -n "${exits[index]:-}" ]] && switch_wheel_collected=$((switch_wheel_collected + 1))
    if observed="$(crash_amplification_switch_wheel_verdict "${switch_wheel_expected[index]:-}" "${exits[index]:-}" "$directory/switch-wheel-$index.out" "$pattern" "$exit_only")"; then
      shown="${observed// /、}"
      echo "  · 开关轮：${switch_wheel_assignments[index]} 红在「${shown:-（一格不红，表里没列它）}」，与文件头相同"
    else
      shown="${switch_wheel_expected[index]:-}"
      [[ "$shown" == - ]] && shown="（表里没列它：应当退 0、一格不红）"
      echo "  ✗ 开关轮：${switch_wheel_assignments[index]} 打开后 --selftest 退 ${exits[index]:-（没收到）}、红在「${observed:-（没有）}」，文件头点名的是「${shown:-（没写）}」" >&2 # gate-lint:detail
      switch_wheel_failures=$((switch_wheel_failures + 1))
    fi
  done
  if (( switch_wheel_collected != count )); then
    echo "  ✗ 开关轮：派出 $count 个开关、收回 $switch_wheel_collected 个" >&2 # gate-lint:detail
    switch_wheel_failures=$((switch_wheel_failures + 1))
  fi
  return 0
}

# 自证（文件头「自证」）：单独执行带 --selftest 才跑，被 source 时不跑。调真的 crash_amplification_run_on_this_host，
# 仓根换成格目录下的假树：内存包装像真的那样把命令放后台等、收到 TERM 先停自己起的子命令（假 scope）再退 143，capped.sh 去掉线程数照起，cargo 是 PATH 前面的一个假脚本
# （像真 cargo 那样占着「库的锁」一个文件、收到 TERM 慢一点放掉、再退）。失败行都是「✗ 自证 <格名>：<细节>」，开关轮靠它认红的格。
crash_amplification_lib_selftest() {
  local lib_script work failures=0 cells=0 cell_exit=0 cell_directory=""
  lib_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-amplification-lib-selftest.XXXXXX")" || {
    echo "  ✗ --selftest：建不了临时目录（TMPDIR=${TMPDIR:-/tmp}）" >&2
    echo "  → 怎么办：看 TMPDIR 可不可写" >&2
    return 2
  }
  trap "rm -rf -- '$work'" EXIT   # 现在展开：函数返回之后局部变量 work 已经没了
  cat >"$work/driver.sh" <<'DRIVER'
#!/usr/bin/env bash
# 自证里调真的 crash_amplification_run_on_this_host 的外壳：$1 仓根（假树）$2 共用库 $3 格目录 $4 形态（foreground 在本 shell 里调、background 像双机驱动那样放后台的子 shell 里调）
repository_root="$1"; source "$2" || exit 3
cell="$3"; shape="$4"
threads=7; memory_cap=1G; common_environment=(); gpu_cards_by_priority=""
cargo_build_arguments=(test --no-run); cargo_test_arguments=(test)
export CRASH_AMPLIFICATION_BUILD_THREADS=3 PATH="$cell/bin:$PATH"
lock_state() { if [[ -e "$STUB_LOCK" ]]; then echo present; else echo absent; fi; }
if [[ "${STUB_CALLER_TRAP:-0}" == 1 ]]; then trap 'echo caller-term-trap >>"$STUB_CALLS"' TERM; fi
if [[ "$shape" == background ]]; then
  crash_amplification_run_on_this_host 0/1 "$cell/library" "$cell/run.log" &
  run_pid=$!
  echo "$run_pid" >"$cell/run-pid.partial"; mv -- "$cell/run-pid.partial" "$cell/run-pid"
  if wait "$run_pid"; then run_exit=0; else run_exit=$?; fi
  echo "run-returned rc=$run_exit lock=$(lock_state)" >>"$STUB_CALLS"
  exit 0
fi
if crash_amplification_run_on_this_host 0/1 "$cell/library" "$cell/run.log"; then run_exit=0; else run_exit=$?; fi
echo "run-returned rc=$run_exit lock=$(lock_state)" >>"$STUB_CALLS"
trap -p TERM >"$cell/trap-after"
exit "$run_exit"
DRIVER
  cat >"$work/cargo" <<'CARGO'
#!/usr/bin/env bash
step=test; [[ " $* " == *" --no-run "* ]] && step=build
echo "cargo-$step started" >>"${STUB_CALLS:?}"
echo "out-$step"; echo "err-$step" >&2
: >"${STUB_LOCK:?}"
release() { sleep 0.3; rm -f -- "$STUB_LOCK"; echo "cargo-$step lock-released" >>"$STUB_CALLS"; }
trap 'echo "cargo-$step got-TERM" >>"$STUB_CALLS"; release; exit 143' TERM
: >"${STUB_DIR:?}/started.$step"
tick=0
if [[ "$step" == "${STUB_HOLD_STEP:-}" ]]; then while (( tick < 50 )); do sleep 0.1; tick=$((tick + 1)); done; fi
echo "cargo-$step done" >>"$STUB_CALLS"
release
exit_variable="STUB_${step^^}_EXIT"
exit "${!exit_variable:-0}"
CARGO
  # 假的内存包装照真包装的样子（scripts/hosts/run-with-memory-cap.sh）：命令放后台起当「scope」，用 wait 等，所以 TERM 的处理当场就跑——
  # 先停自己起的这个子命令（真包装是 systemctl --user stop 自己开的 scope），等它退，再退 143；命令自己退了就原样转出它的退出码
  cat >"$work/memory-cap" <<'MEMORYCAP'
#!/usr/bin/env bash
shift
"$@" &
scope_child=$!
trap 'kill -TERM "$scope_child" 2>/dev/null; wait "$scope_child"; exit 143' TERM
wait "$scope_child"
exit $?
MEMORYCAP

  run_cell() { # <格名> <形态 foreground|background> <给 TERM 的时机 none|build|test> [<环境 键=值>…]：交回值放 cell_exit，调用记录放 <格目录>/calls
    local name="$1" shape="$2" term_at="$3" waited=0 driver_pid hold_step="" target=""
    shift 3
    [[ "$term_at" == none ]] || hold_step="$term_at"
    cell_directory="$work/$name"
    mkdir -p "$cell_directory/root/scripts/hosts" "$cell_directory/bin"
    : >"$cell_directory/calls"
    cells=$((cells + 1))
    cp -- "$work/memory-cap" "$cell_directory/root/scripts/hosts/run-with-memory-cap.sh"
    printf '%s\n' '#!/usr/bin/env bash' 'shift; exec "$@"' >"$cell_directory/root/scripts/hosts/capped.sh"
    cp -- "$work/cargo" "$cell_directory/bin/cargo"; chmod +x "$cell_directory/bin/cargo"
    env -u CRASH_AMPLIFICATION_TERM_WAIT_SECONDS STUB_CALLS="$cell_directory/calls" STUB_LOCK="$cell_directory/library.lock" STUB_DIR="$cell_directory" STUB_HOLD_STEP="$hold_step" \
      STUB_CALLER_TRAP=0 STUB_BUILD_EXIT=0 STUB_TEST_EXIT=0 "$@" bash "$work/driver.sh" "$cell_directory/root" "$lib_script" "$cell_directory" "$shape" >"$cell_directory/output" 2>&1 &
    driver_pid=$!
    if [[ "$term_at" != none ]]; then
      while (( waited < 100 )) && { [[ ! -e "$cell_directory/started.$term_at" ]] || { [[ "$shape" == background && ! -e "$cell_directory/run-pid" ]]; }; }; do
        sleep 0.1; waited=$((waited + 1))
      done
      target="$driver_pid"
      [[ "$shape" == background ]] && target="$(cat -- "$cell_directory/run-pid" 2>/dev/null)"
      [[ "$target" =~ ^[0-9]+$ ]] && kill -TERM "$target" 2>/dev/null
    fi
    if wait "$driver_pid"; then cell_exit=0; else cell_exit=$?; fi
    echo "driver-exited rc=$cell_exit lock=$([[ -e "$cell_directory/library.lock" ]] && echo present || echo absent)" >>"$cell_directory/calls"
    # 被停的格：驱动退了之后假 cargo 的 TERM 处理（记 got-TERM、放锁）可能还在跑；等它放掉（至多 8 秒）再判，判的是它收没收到 TERM，不是赶不赶得上驱动退出
    if [[ "$term_at" != none ]]; then
      for ((waited = 0; waited < 80; waited++)); do
        [[ -e "$cell_directory/library.lock" ]] || break
        sleep 0.1
      done
    fi
  }
  cell_failed() { # <格名> <判错的一句> <出路>
    echo "  ✗ 自证 $1：$2（退 $cell_exit；调用记录「$(tr '\n' ';' <"$work/$1/calls")」）" >&2
    echo "     → 怎么办：$3" >&2
    tail -n 8 "$work/$1/output" | sed 's/^/       | /' >&2
    failures=$((failures + 1))
  }
  calls_have() { grep -qE -- "$2" "$work/$1/calls"; }   # <格名> <正则>
  line_of() { grep -nE -m1 -- "$2" "$work/$1/calls" | cut -d: -f1; }   # <格名> <正则>：第一次出现的行号，没有是空串
  before() { local first second; first="$(line_of "$1" "$2")"; second="$(line_of "$1" "$3")"; [[ -n "$first" && -n "$second" ]] && (( first < second )); }   # <格名> <正则> <正则>：前者先出现

  # 被 TERM 停（C797）：放后台的子 shell 里（双机驱动起本机那一趟的样子），分份途中给它 TERM
  run_cell term-reaches-cargo background test
  if [[ "$cell_exit" != 0 ]] || ! calls_have term-reaches-cargo '^cargo-test got-TERM$' || calls_have term-reaches-cargo '^cargo-test done$' \
     || ! before term-reaches-cargo '^cargo-test lock-released$' '^run-returned rc=143 lock=absent$' || ! calls_have term-reaches-cargo '^driver-exited rc=0 lock=absent$'; then
    cell_failed term-reaches-cargo "跑 cargo test 那一步时给本机那一趟（放后台的子 shell）TERM：假 cargo 应当收到 TERM、没跑到底，库的锁在那一趟返回之前放掉，那一趟退 143" \
      "看 crash_amplification_run_on_this_host 与 crash_amplification_on_term：TERM 要转给记下的那一个进程号（内存包装，它自己停它开的 scope）、等它退出、放回原来的 TERM 处理，再对自己发 TERM"
  fi
  # 同样的事发生在调用方自己的 shell 里（崩溃放量分项驱动的主线）：没设 TERM 处理，库的锁放掉之后自己按被停退 143
  run_cell term-reaches-cargo-in-foreground foreground test
  if [[ "$cell_exit" != 143 ]] || ! calls_have term-reaches-cargo-in-foreground '^cargo-test got-TERM$' || calls_have term-reaches-cargo-in-foreground '^cargo-test done$' \
     || ! before term-reaches-cargo-in-foreground '^cargo-test lock-released$' '^driver-exited rc=143 lock=absent$'; then
    cell_failed term-reaches-cargo-in-foreground "跑 cargo test 那一步时给调用方的 shell（没设 TERM 处理）TERM：假 cargo 应当收到 TERM、库的锁在调用方退出之前放掉，调用方退 143" \
      "看 crash_amplification_run_on_this_host 收尾：放回原来的 TERM 处理（没有就是默认）之后对自己发 TERM"
  fi
  # 调用方设了 TERM 处理（双机驱动的主线）：它的处理在库的锁放掉之后跑一次，那一趟交回 143
  run_cell term-returns-143-under-caller-trap foreground test STUB_CALLER_TRAP=1
  if [[ "$cell_exit" != 143 ]] || ! calls_have term-returns-143-under-caller-trap '^cargo-test got-TERM$' || [[ "$(grep -c '^caller-term-trap$' "$work/term-returns-143-under-caller-trap/calls")" != 1 ]] \
     || ! before term-returns-143-under-caller-trap '^cargo-test lock-released$' '^caller-term-trap$' || ! calls_have term-returns-143-under-caller-trap '^run-returned rc=143 lock=absent$'; then
    cell_failed term-returns-143-under-caller-trap "调用方设了 TERM 处理：假 cargo 收到 TERM，库的锁放掉之后调用方的处理跑恰好一次，那一趟交回 143" \
      "看 crash_amplification_run_on_this_host 收尾：先放回调用方原来的 TERM 处理、再对自己发 TERM，它的处理才轮得到"
  fi
  # 编的那一步被停：不起跑的那一步
  run_cell term-during-build-skips-test foreground build
  if [[ "$cell_exit" != 143 ]] || ! calls_have term-during-build-skips-test '^cargo-build got-TERM$' || calls_have term-during-build-skips-test '^cargo-test started$'; then
    cell_failed term-during-build-skips-test "只编那一步时给调用方 TERM：假 cargo 收到 TERM，跑的那一步不起，调用方退 143" \
      "看 crash_amplification_run_on_this_host：收到 TERM 之后不再起下一步（crash_amplification_term_received）"
  fi
  # 没被停：两步按序各起一次、日志里两步的输出按序都在，退出码原样转出；编的那一步红了不起跑的那一步
  run_cell exit-code-passes-through foreground none STUB_TEST_EXIT=7
  if [[ "$cell_exit" != 7 ]] || ! before exit-code-passes-through '^cargo-build done$' '^cargo-test started$' || ! calls_have exit-code-passes-through '^cargo-test done$' \
     || [[ "$(grep -cE '^(out|err)-(build|test)$' "$work/exit-code-passes-through/run.log")" != 4 ]] \
     || [[ "$(grep -nE '^out-test$' "$work/exit-code-passes-through/run.log" | cut -d: -f1)" -le "$(grep -nE '^out-build$' "$work/exit-code-passes-through/run.log" | cut -d: -f1)" ]]; then
    cell_failed exit-code-passes-through "没被停：编、跑两步按序各起一次，日志里两步的标准输出与标准错误按序都在，跑的那一步退 7 就原样转出 7" \
      "看 crash_amplification_run_step 与 crash_amplification_run_on_this_host：日志追加（调用方先清空）、wait 的退出码原样交回"
  fi
  run_cell build-failure-skips-test foreground none STUB_BUILD_EXIT=5
  if [[ "$cell_exit" != 5 ]] || calls_have build-failure-skips-test '^cargo-test started$'; then
    cell_failed build-failure-skips-test "编的那一步退 5：跑的那一步不起，原样转出 5" \
      "看 crash_amplification_run_on_this_host：编的那一步退出码非 0 就不起下一步（原来的 && 链）"
  fi
  # 没被停：调用方设的 TERM 处理原样放回
  run_cell caller-term-trap-kept foreground none STUB_CALLER_TRAP=1
  if [[ "$cell_exit" != 0 ]] || ! grep -qF 'caller-term-trap' "$work/caller-term-trap-kept/trap-after" || grep -qF 'crash_amplification_on_term' "$work/caller-term-trap-kept/trap-after"; then
    cell_failed caller-term-trap-kept "没被停：那一趟返回之后调用方原来的 TERM 处理还在、本库的处理已拆掉（实有「$(cat -- "$work/caller-term-trap-kept/trap-after" 2>/dev/null)」）" \
      "看 crash_amplification_restore_term_trap 与 crash_amplification_run_on_this_host 起头记下的 trap -p TERM"
  fi

  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 bash $lib_script --selftest" >&2
    return 1
  fi
  if crash_amplification_switch_wheel_inner; then echo "  ✓ --selftest（开关轮里起的一份，不再起开关轮）：$cells 格都对"; return 0; fi

  # 开关轮（文件头「弄坏开关」）：从文件头现读开关与点名的格、与代码里认的开关比齐，逐个打开再跑一遍 --selftest，红的格要与点名的逐格相等
  local table assignment named in_table="" in_code duplicated round_failures=0 listed_count
  switch_wheel_assignments=(); switch_wheel_expected=()
  table="$(crash_amplification_read_switch_table "$lib_script" CRASH_AMPLIFICATION_LIB_BREAK)"
  while IFS=$'\t' read -r assignment named; do
    [[ -n "$assignment" ]] || continue
    switch_wheel_assignments+=("$assignment"); switch_wheel_expected+=("$named")
    in_table+="${assignment#*=}"$'\n'
  done <<<"$table"
  listed_count="${#switch_wheel_assignments[@]}"
  if (( listed_count == 0 )); then
    echo "  ✗ 开关轮：文件头「弄坏开关」那张表一个开关都没读到" >&2
    echo "     → 怎么办：表里一行「#   CRASH_AMPLIFICATION_LIB_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（文件头「弄坏开关轮」一段的写法）" >&2
    return 1
  fi
  duplicated="$(printf '%s\n' "${switch_wheel_assignments[@]}" | LC_ALL=C sort | uniq -d | tr '\n' ' ')"
  if [[ -n "$duplicated" ]]; then
    echo "  ✗ 开关轮：文件头表里同一个开关列了不止一次：$duplicated" >&2
    echo "     → 怎么办：一个开关只写一行，点名的格都写在它下面" >&2
    round_failures=$((round_failures + 1))
  fi
  in_code="$(grep -oE 'crash_amplification_lib_break_is [a-z0-9-]+' "$lib_script" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table="$(printf '%s' "$in_table" | LC_ALL=C sort -u)"
  if [[ "$in_code" != "$in_table" ]]; then
    echo "  ✗ 开关轮：代码里 crash_amplification_lib_break_is 认的开关与文件头表里列的不一样（只在代码里：$(LC_ALL=C comm -23 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table") | tr '\n' ' ')；只在表里：$(LC_ALL=C comm -13 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table") | tr '\n' ' ')）" >&2
    echo "     → 怎么办：代码里每一处认的开关都在表里写一行（连同点名的格），表里每一行都在代码里有对应的一处" >&2
    round_failures=$((round_failures + 1))
  fi
  crash_amplification_switch_wheel "$work" '✗ 自证 ([a-z0-9-]+)：' 0 -- bash "$lib_script" --selftest
  round_failures=$((round_failures + switch_wheel_failures))
  if (( round_failures > 0 )); then
    echo "  ✗ --selftest：$cells 格都对，开关轮有 $round_failures 处对不上" >&2
    echo "     → 怎么办：红的格与表里点名的不同，要么改那一处判法让开关红在点名的格，要么照实改文件头那一行；单看一个开关：CRASH_AMPLIFICATION_LIB_BREAK=<开关> bash $lib_script --selftest" >&2
    return 1
  fi
  echo "  ✓ --selftest：本机那一趟被 TERM 停时（放后台的子 shell 里、调用方的 shell 里没设与设了 TERM 处理）TERM 到得了假 cargo、库的锁在那一趟返回之前放掉、退 143，编的那一步被停不起跑的那一步，没被停时两步按序起、日志按序、退出码原样转出、调用方的 TERM 处理原样放回，$cells 格都对（cargo、内存包装、capped.sh 全是假的，不起重活）；"
  echo "    开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的格"
  return 0
}

# 单独执行：只认 --selftest；被 source 时（调用方的 $0 不是这个文件）什么都不做
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  set -uo pipefail
  if [[ "${1:-}" == --selftest && $# -eq 1 ]]; then crash_amplification_lib_selftest; exit $?; fi
  echo "  ✗ 用法：$0 --selftest（这是被 source 的函数库，单独执行只认 --selftest）" >&2
  echo "  → 怎么办：要用库里的函数就在脚本里 source 它；要自证本机那一趟起 cargo 的那一段就跑 bash $0 --selftest" >&2
  exit 2
fi
