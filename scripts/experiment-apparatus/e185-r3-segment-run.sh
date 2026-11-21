#!/usr/bin/env bash
# admission: always 整个脚本每次都起：这一段已有没有判定、键对不对得上由 prune-server 里的按段判定现答（scripts/experiment-apparatus/long_run_item_verdicts.py）
# run-condition: command cargo python3 git sha256sum getconf nproc
# task-server-card-lease: on-demand
# E185（GC一批搬迁的挂钟拆分）第 3 次重跑一段的跑法（跑前登记 research/prompts/e185-r3-prereg.md 第五节「跑法」）：
#   写输入清单 → 有段要现判才在这一趟自己的编译目录里编装置（crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split.rs；
#   第一、二段带 gpu 特性，第三段不带；编译目录经 scripts/hosts/run-build-directory.sh 建、跑完删）→ 跑那一段 → 产物写进产物目录。
# 按段判定（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：一段一条判定，在 prune-server 的 apparatus-items 族、种类 e185_gc_batch_wall_clock_split
#   （long_run_item_verdicts.py 文件头「判定库」）。键 = 装置入口 main 的单元闭包（release 档、这一段的特性；admission.py 的 compute_unit_key 算）、工具链、闭包真引到的依赖版本、
#   这一段的装置参数（文件路径只写成占位）与它读的输入文件的内容（第一段读的两份）；产物带计时，比法 compare=timing（只比绿红与退出码）。绿 = 装置退 0 且末行是 E7RESULT name=done。
#   键相同、库里判绿的段把库里那一份产物写到这一趟的产物名上（输入清单末尾记一句复用了）、装置不起；判红的照同一把键记红撤绿、下一趟现判；装置退 78（本次未跑）不记。
#   成功那句报「现判 N、复用 M、拒算 R」。一趟只有一段可复用时复用哨兵不足、不抽强制现判（long_run_item_verdicts.py 文件头「复用哨兵」），报一句、不红。
# 准入与运行条件：
#   装置源码的 sha256 要等于登记第十二节写的那一个（下面 registered_apparatus_sha256）；不等就写完输入清单、退 3（停机 J-S7），不编、不跑。
#   输入清单里 `cargo tree` 那一段出现 lookup-work-counters（计数特性被并进来）就退 6，那一段整段作废（登记第十一节作废条款）。
#   按需领卡（文件头第 4 行；客户端 task-server/task 的 lease take|return，task_server.py 文件头 ②「按需领卡」）：服务起跑不分卡、不清场；编译、算键、读判定库、复用的段与第三段都不持卡。
#     第一、二段（segment_uses_card：编装置的特性里有 gpu）现跑时，脚本在起装置之前、编译之后经 task-server/task lease take 领一张卡（服务交卡之前对那张清场回读；卡都被占着就等，
#     客户端往标准错误打在等什么），领到的卡串交给装置（GPU_CARDS_BY_PRIORITY，装置不再自己领），装置退出（退 0、判红、退 78）就 lease return 还掉；被 TERM / INT 打断时退出的收尾也还。
#     环境里原有的 GPU_CARDS_BY_PRIORITY 开跑前清掉：卡只从领卡来，不要卡的段看不到卡。
#     现跑的段里有要卡的、环境里却没有领卡通道（TASK_SERVER_CARD_LEASE_DIRECTORY 与 TASK_SERVER_CARD_LEASE_TOKEN，task-server/task_server_lib.py 的 CARD_LEASE_DIRECTORY_VARIABLE、
#     CARD_LEASE_TOKEN_VARIABLE；不是服务照按需领卡起的）：看过判定库、定下哪些段现跑之后、编译之前退 78，一个装置都没起，写原因与出路。
#     领卡没成（lease take 退非 0：服务拒了、通道没了）：那一段不起装置、不进判定库（没跑就没有判定，不撤绿），整趟退 1（几段里头一个退非 0 的那一个）。
#     还卡没成（lease return 退非 0，客户端 120 秒等不到回执退 1）：那一段照已判的算，打一行「!」点名退出码，不改这一趟的退出码（尝试结束时服务照样收回没还的卡）。
#     本脚本不套清场包装。线程与内存不在脚本里套：服务给的变量（SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS）cargo 与装置直接读。
# 起法（主 agent 在仓根起）：
#   第一、二段在本机，先后起、不同时起：
#     task-server/task submit -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r3-segment-run.sh host-cpu-per-submission
#     跑完再起同一条换成 recompute-disagreement。
#   第三段（附带；主 agent 定了要量才起）：
#     task-server/task submit -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r3-segment-run.sh function-split
#     派哪台由服务定、清场与分卡在要卡的段领卡时由服务做（不套清场包装，不自己连第二台；records/2026-10-31-task-server巡检.md 问题 94），两台在同一棵共享的树里跑，
#     这一段的产物（名字见「产物」）直接写进 research/results/，不取回；输入清单里这一台本地盘上的目录，主 agent 换成「〔第二台的目录〕」「〔本机的目录〕」再留在 research/results/。
#
# 用法：
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh host-cpu-per-submission   第一段（本机，两张卡）
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh recompute-disagreement    第二段（本机，两张卡）
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh function-split            第三段（附带，第二台，不起卡）
#   段名可以写几个（照写的次序跑，一段一份产物）
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh --selftest   假 cargo、假 nvidia-smi、假领卡客户端、假装置在临时树里走一遍（不编、不起卡、不跑真装置）；判定写进临时 prune-server：
#     只在临时服务底下跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem；不在就退 2、一格都不跑），起法：
#     python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e185-r3-segment-run.sh --selftest
# 产物（<日期> 取 E185_RUN_DATE，没设取起跑当天 TZ=Asia/Tokyo date +%F；同一天同一段已有产物时两份一起加 -2、-3……，不盖已有的）：
#   e185-gc-batch-wall-clock-split-<段>-r3-<日期>.out（装置的标准错误写同名加 .err；复用的段不起装置，不写 .err）；
#   输入清单 e185-r3-inputs-<段>-<日期>.txt。
# 环境：E185_PRODUCT_DIRECTORY（产物目录，默认 research/results）、E185_RUN_DATE。线程不设，装置取登记值 .claude/heavy-thread-default（E185_THREADS
#   只在第二段造内容与 CPU 参照、第三段准备队列里用）；装置的内存预算 E185_MEMORY_CAP 没设时，本脚本（e185_r3_settle_memory_cap）把它设成这一台能给的内存
#   （多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项，scripts/hosts/task_memory.py 的 host_slice_cap_bytes），装置照它定准备队列的并行条数，不取本进程 cgroup 的 memory.max（任务服务给的额度，不是这一台能给多少）；
#   调用方设了 E185_MEMORY_CAP 照它；这一台那一项取不到就不设，装置取它自己的缺省（memory_cap_given：本进程 cgroup 的 memory.max，读不到取缺省）。
# 退出码：0 跑完；1 有段判红而装置退 0（末行不齐）、复用哨兵红或键在跑的中途变了；2 段名认不出、弄坏开关设在自证之外；3 装置源码的 sha256 与登记不等（停机 J-S7），或装置自己的停机条款；
#   6 计数特性被并进来（整段作废）；75 prune-server 不可用；78 现跑的段里有要卡的而没有领卡通道（一个装置都没起），或取卡没过、没编 gpu 特性（本次未跑，装置退）；
#   领卡没成的那一段不起、整趟退 1；别的照 cargo 与装置自己的退出码（见装置文件头；几段里头一个退非 0 的那一个）。
# 弄坏开关 E185_R3_BREAK=<项，逗号分隔>（只在 --selftest 起的那几趟里认：自证设 E185_R3_SELFTEST_MARK 指到它建的标记文件，别处设了退 2）：
#   E185_R3_BREAK=sha-unchecked  不核装置源码的 sha256
#       「装置源码的 sha256 与登记不等：退 3、装置一次没起」
#   E185_R3_BREAK=counters-unchecked  不看输入清单里的 lookup-work-counters
#       「cargo tree 里有 lookup-work-counters：退 6、装置一次没起」
#   E185_R3_BREAK=done-line-unchecked  装置退 0 就记绿，不看末行
#       「末行不是 name=done 的段判红：退 1、记红；装置修好再起，那一段现判」
#   E185_R3_BREAK=reuse-ignored  库里判绿的段也照起装置现判（不认 plan 答的 reuse）
#       「一段现判、产物与输入清单写进产物目录；原样重起复用、装置不起、产物另存一份带 -2 的」「reuse-three-questions」「key-independent-of-directory」「reuse-sentinel」「forced-still-reused-red」
#   E185_R3_BREAK=finish-ignored  不认 long_run_item_verdicts.py finish 的退出码（复用哨兵红、键中途变了照退 0）
#       「sentinel-rejudged-red」「forced-verdict-differs-red」「forced-still-reused-red」
#   E185_R3_BREAK=card-leased-for-every-segment  不要卡的段也领卡、也还
#       「不要卡的段不领卡」
#   E185_R3_BREAK=ambient-cards-kept  环境里原有的 GPU_CARDS_BY_PRIORITY 开跑前不清掉
#       「环境里原有的卡串开跑前清掉」
#   E185_R3_BREAK=card-return-skipped  装置退出后不调还卡（收尾也不调）
#       「要卡的段领一次还一次」「要卡的段装置判红也还卡」「要卡的段被打断也还卡」「还卡没成只打一行、退出码不变」
#   E185_R3_BREAK=card-lease-refusal-ignored  领卡没成照起装置（不带卡）
#       「领卡被拒的段不起、不进判定库、整趟退 1；再起时那一段现判」
#   E185_R3_BREAK=card-return-failure-unreported  还卡没成不打那一行「!」
#       「还卡没成只打一行、退出码不变」
#   E185_R3_BREAK=card-lease-channel-unchecked  不核领卡通道
#       「没有领卡通道而要现跑要卡的段：退 78、编译与装置都没起」
#   E185_R3_BREAK=memory-by-task-cap  不把 E185_MEMORY_CAP 设成这一台能给的内存（装置取本进程 cgroup 的 memory.max）
#       「装置内存预算取这一台能给的内存、不取服务给的上限」
# 复用那几格另认 long_run_item_verdicts.py 的弄坏开关 LONG_RUN_ITEMS_BREAK（它文件头；打开后本脚本的自证红在它点名的同名格）。
# 复用三问的假驱动：自证的假装置，host-cpu-per-submission、recompute-disagreement 两段的装置入口换成假树里两个 Python 函数（long_run_item_verdicts.py 文件头「复用格」）。
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || { echo "进不去仓根"; echo "→ 怎么办：从仓根照文件头的用法起"; exit 2; }

SCRIPT_KEY="scripts/experiment-apparatus/e185-r3-segment-run.sh"
ITEM_VERDICTS="scripts/experiment-apparatus/long_run_item_verdicts.py"   # 按段判定的读写（文件头「按段判定」）
LIBRARY_KIND="e185_gc_batch_wall_clock_split"                            # 判定库的种类：装置的 bin 名
registered_apparatus_sha256=4368c0e7a7870a09810fa5667173f9cf14625f4d64c4e0287ee10ec7ef80e6ea
apparatus_source=crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split.rs
attack_ledger=research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out
stage_split_product=research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
# 输入清单 sha256 的那几份（自证照这张表搭树）
INPUT_SOURCES=(crates/singlefs-verification/src/gpu_unit_checks.rs crates/singlefs-core/src/mounted_session.rs
  crates/singlefs-core/src/region_scan.rs crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/checksum.rs)

# 自证起的那几趟（标记文件在、内容对得上）：弄坏开关只在这里认；样板树里的装置源码是假的，登记的 sha256 换成 E185_R3_SELFTEST_REGISTERED_SHA256 给的那一个
e185_r3_selftest_marked() {
  [[ -n "${E185_R3_SELFTEST_MARK:-}" ]] && [[ "$(cat -- "$E185_R3_SELFTEST_MARK" 2> /dev/null)" == e185-r3-selftest ]]
}
e185_r3_break_is() {
  [[ ",${E185_R3_BREAK:-}," == *",$1,"* ]] && e185_r3_selftest_marked
}
if [[ -n "${E185_R3_SELFTEST_REGISTERED_SHA256:-}" ]] && e185_r3_selftest_marked; then
  registered_apparatus_sha256=$E185_R3_SELFTEST_REGISTERED_SHA256
fi

# 装置的内存预算（文件头「环境」）：<仓根>。E185_MEMORY_CAP 没设就设成这一台能给的内存（<仓根>/scripts/hosts/task_memory.py 的 host_slice_cap_bytes），
# 装置不再取本进程 cgroup 的 memory.max；这一台那一项取不到（多机配置不在、没写、<仓根> 里没有 task_memory.py）不设。说明写进 memory_cap_note（输入清单里打）。
# 弄坏开关 memory-by-task-cap：不设
e185_r3_settle_memory_cap() {
  local reading host_cap host_text
  if [[ -n "${E185_MEMORY_CAP:-}" ]]; then
    memory_cap_note="E185_MEMORY_CAP=$E185_MEMORY_CAP（调用方给的）"
    return 0
  fi
  reading=$(python3 -c 'import os, sys
sys.dont_write_bytecode = True
try:
    sys.path.insert(0, os.path.join(sys.argv[1], "scripts", "hosts"))
    import task_memory
    cap, text = task_memory.host_slice_cap_bytes(os.environ)
except Exception as error:
    cap, text = None, "%s：%s" % (type(error).__name__, error)
print("%s\t%s" % (cap if cap is not None else "-", " ".join(text.split())))' "$1") || reading=$'-\tpython3 起不来'
  host_cap=${reading%%$'\t'*}
  host_text=${reading#*$'\t'}
  if [[ "$host_cap" == - ]]; then
    memory_cap_note="E185_MEMORY_CAP 没设（这一台能给的内存取不到：$host_text），装置取本进程 cgroup 的 memory.max（任务服务给的额度），读不到取装置自己的缺省"
  elif e185_r3_break_is memory-by-task-cap; then
    memory_cap_note="弄坏开关 memory-by-task-cap：E185_MEMORY_CAP 不设，装置取本进程 cgroup 的 memory.max"
  else
    export E185_MEMORY_CAP=$host_cap
    memory_cap_note="E185_MEMORY_CAP=$host_cap（这一台能给的内存：$host_text），装置不取本进程 cgroup 的 memory.max"
  fi
}

refuse_an_unknown_segment_name() {
  echo "✗ 段名认不出：「$1」"
  echo "→ 怎么办：段名写 host-cpu-per-submission、recompute-disagreement、function-split 之一（文件头的用法），可以写几个"
  exit 2
}

segment_features() { # <段>：编装置的特性（进键）
  case "$1" in
    host-cpu-per-submission | recompute-disagreement) echo gpu ;;
    function-split) echo "" ;;
  esac
}

segment_uses_card() { # <段>：现跑要上卡的段 = 编装置的特性里有 gpu 的（文件头「按需领卡」）
  [[ " $(segment_features "$1") " == *" gpu "* ]]
}

# 一段的装置参数：文件路径写成占位（进键的那一份；起装置时换成真路径），文件内容经 @input 按角色进键
segment_arguments() { # <段>
  case "$1" in
    host-cpu-per-submission) echo "host-cpu-per-submission --cards 0,1 --stage-split-product <stage-split-product> --attack-ledger <attack-ledger>" ;;
    recompute-disagreement) echo "recompute-disagreement --cards 0,1" ;;
    function-split) echo "function-split --point 0.25:1024 --point 0.25:8192 --point 0.75:1024 --point 0.75:8192" ;;
  esac
}

segment_request() { # <段>（long_run_item_verdicts.py 文件头「请求文件」）
  local feature=""
  [[ -n "$(segment_features "$1")" ]] && feature=$'\t'"feature:singlefs-verification/$(segment_features "$1")"
  printf '%s\tcrates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split.rs::main\tprofile:release%s\tline:arguments=%s\tline:compare=timing\n' \
    "$1" "$feature" "$(segment_arguments "$1")"
  if [[ "$1" == host-cpu-per-submission ]]; then
    printf '@input\t%s\tstage-split-product\t%s\n@input\t%s\tattack-ledger\t%s\n' "$1" "$stage_split_product" "$1" "$attack_ledger"
  fi
}

# 判定库那一步没成：75（prune-server 不可用）与 2（用法错）照交，别的按 1 交
library_failure_exit() { # <哪一步> <退出码>
  echo "✗ 按段判定的 $1 没成（long_run_item_verdicts.py 退 $2）"
  echo "→ 怎么办：退 75 是 prune-server 不可用：python3 prune-server/control check 看服务，修好之后重交同一条命令（报给主 agent）；别的照上面它打的出路改"
  if (($2 == 75 || $2 == 2)); then exit "$2"; fi
  exit 1
}

# 领卡通道（文件头「按需领卡」）：服务照文件头的按需领卡声明起的这一次尝试才有。没有就返回 1（打 ✗ 与 → 出路）：开跑前一次说清，不让不要卡的段先跑完、到要卡的段才红。
# 弄坏开关 card-lease-channel-unchecked：不核。
e185_r3_card_lease_channel_present() { # <现跑要卡的段，空格隔开>
  e185_r3_break_is card-lease-channel-unchecked && return 0
  if [[ -d "${TASK_SERVER_CARD_LEASE_DIRECTORY:-}" && -n "${TASK_SERVER_CARD_LEASE_TOKEN:-}" ]]; then
    return 0
  fi
  echo "✗ 现跑的段里有要卡的（$1），环境里却没有领卡通道（TASK_SERVER_CARD_LEASE_DIRECTORY 不是在的目录，或没有 TASK_SERVER_CARD_LEASE_TOKEN），一个装置都没起"
  echo "→ 怎么办：整条经 task-server/task submit -- bash $SCRIPT_KEY … 交（文件头的按需领卡声明让服务给这一次尝试建领卡通道，要卡的段本脚本自己领、自己还）；不另套清场包装"
  return 1
}

# 此刻持着的卡串（空 = 没持卡）。客户端是这棵树里的 task-server/task（脚本已 cd 进仓根）。
held_card=""
# 领到了把卡串放进 held_card、返回 0；没领到（客户端退非 0，或退 0 却没打卡串）打 ✗ 与 → 出路、返回 1。客户端在等卡时往标准错误打的行照原样留在这一趟的标准错误里。
e185_r3_card_lease_take() { # <段>
  local lease_output lease_exit
  held_card=""
  if lease_output="$(task-server/task lease take)"; then lease_exit=0; else lease_exit=$?; fi
  lease_output="${lease_output//[$'\n']/}"
  if ((lease_exit == 0)) && [[ -n "${lease_output//[[:space:]]/}" ]]; then
    held_card="$lease_output"
    echo "  · 领卡：$1 段领到 $held_card，装置退出就还"
    return 0
  fi
  ((lease_exit != 0)) || lease_exit=1
  echo "  ✗ $1 段领卡没成（task-server/task lease take 退 $lease_exit，原因在上面客户端那一句），装置没起、不进判定库"
  echo "  → 怎么办：照客户端那一句的出路办（不在按需领卡的尝试里就经 task-server/task submit 整条重交；这一次尝试已持着卡说明前一段没还）；别的段判过的照键复用"
  return 1
}
# 还掉持着的卡。客户端退非 0（120 秒没回执退 1）只打一行「!」点名退出码，不改这一趟的退出码：尝试结束时服务照样收回。装置退出后调一次，收尾（EXIT 的 trap）再调一次：打断时也还。
# 弄坏开关 card-return-skipped：不调客户端；card-return-failure-unreported：还卡没成不打那一行。
e185_r3_card_lease_return() {
  local card="$held_card" return_exit
  [[ -n "$card" ]] || return 0
  held_card=""
  ! e185_r3_break_is card-return-skipped || return 0
  if task-server/task lease return; then return_exit=0; else return_exit=$?; fi
  if ((return_exit == 0)); then
    echo "  · 还卡：$card 还了"
  elif ! e185_r3_break_is card-return-failure-unreported; then
    echo "  ! 还卡没成：task-server/task lease return 退 $return_exit（原因在上面客户端那一句），卡 $card 到这一次尝试结束由服务收回；这一段照已判的算"
  fi
  return 0
}

# 自证：样板树是本脚本的拷贝，.claude、Cargo.lock、输入清单记 sha256 的那几份源码、两份输入、run-build-directory 与 long_run_item_verdicts.py 是指回真仓的符号链接，
# 装置源码是假的一行（登记的 sha256 经 E185_R3_SELFTEST_REGISTERED_SHA256 换成它的，只在自证标记在时认）；
# 假 cargo（build 把假装置放进 CARGO_TARGET_DIR/release，tree 照 E185_SELFTEST_COUNTERS 打计数特性）、假 nvidia-smi 放在 PATH 最前面；不编、不起卡、不跑真装置。
selftest() {
  local work real_root="$PWD" failures=0 cell_count=0 cell exit_code first_exit reuse_output reuse_red link fake_source_sha256 interrupted_pid refused_events refused_calls memory_given memory_absent memory_kept
  set +e   # 自证逐格收退出码，不让 -e 在头一个退非 0 的那一趟就收场
  python3 "$real_root/$ITEM_VERDICTS" selftest-service || return 2
  work=$(mktemp -d "${TMPDIR:-/tmp}/e185-r3-selftest.XXXXXX")
  trap 'rm -rf -- "$work"' RETURN
  echo e185-r3-selftest > "$work/mark"
  mkdir -p "$work/bin"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "cargo $*" >> "$E185_SELFTEST_CALLS.cargo"' \
    'if [[ "${1:-}" == tree ]]; then [[ -z "${E185_SELFTEST_COUNTERS:-}" ]] || echo "singlefs-core feature \"lookup-work-counters\""; exit 0; fi' \
    '[[ "${1:-}" == build ]] || exit 0' '[[ -z "${E185_SELFTEST_EVENTS:-}" ]] || echo build >> "$E185_SELFTEST_EVENTS"' \
    'mkdir -p "$CARGO_TARGET_DIR/release"' 'cp -- "$E185_SELFTEST_APPARATUS" "$CARGO_TARGET_DIR/release/e185_gc_batch_wall_clock_split"' > "$work/bin/cargo"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "0, SELFTEST GPU, 1.0, 4, 16, 1 MiB"' > "$work/bin/nvidia-smi"
  # 假装置：记一笔，照参数打一行和收尾行（E185_SELFTEST_APPARATUS_MODE=truncated 不打收尾行；=hang 记下「ready」后等 release 文件出现，至多 20 秒），退 0；
  #   E185_SELFTEST_EVENTS 设了就往它记一行「apparatus cards=<它看到的 GPU_CARDS_BY_PRIORITY>」
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*" >> "$E185_SELFTEST_CALLS"' \
    '[[ -z "${E185_SELFTEST_EVENTS:-}" ]] || echo "apparatus cards=${GPU_CARDS_BY_PRIORITY:-none}" >> "$E185_SELFTEST_EVENTS"' \
    'if [[ "${E185_SELFTEST_APPARATUS_MODE:-ok}" == hang ]]; then' '  : > "$E185_SELFTEST_EVENTS.ready"' \
    '  for _ in $(seq 200); do [[ -e "$E185_SELFTEST_EVENTS.release" ]] && break; sleep 0.1; done' 'fi' \
    'printf "E7RESULT name=row arguments=%s\n" "$1"' \
    '[[ "${E185_SELFTEST_APPARATUS_MODE:-ok}" == truncated ]] || printf "E7RESULT name=done emitted=2\n"' > "$work/apparatus"
  # 假领卡客户端（样板树里的 task-server/task）：每一回往 E185_SELFTEST_EVENTS 记「lease take」「lease return」；
  #   E185_SELFTEST_CARD_LEASE=refused 时领卡退 3（服务拒），return-unanswered 时还卡退 1（没回执），没设就领到 selftest-card、还了
  printf '%s\n' '#!/usr/bin/env bash' '[[ -z "${E185_SELFTEST_EVENTS:-}" ]] || echo "lease ${2:-none}" >> "$E185_SELFTEST_EVENTS"' \
    'case "${1:-} ${2:-}" in' \
    '  "lease take") if [[ "${E185_SELFTEST_CARD_LEASE:-}" == refused ]]; then echo "  ✗ 服务没给这一回领卡：自证演的拒" >&2; exit 3; fi; echo selftest-card ;;' \
    '  "lease return") if [[ "${E185_SELFTEST_CARD_LEASE:-}" == return-unanswered ]]; then echo "  ✗ 还卡的请求 120 秒没有回执（自证演的）" >&2; exit 1; fi ;;' \
    '  *) echo "假领卡客户端只认 lease take、lease return，收到：$*" >&2; exit 2 ;;' 'esac' > "$work/task"
  chmod +x "$work/bin/cargo" "$work/bin/nvidia-smi" "$work/apparatus" "$work/task"
  mkdir -p "$work/channel"
  local tree="$work/template"
  mkdir -p "$tree/scripts/experiment-apparatus" "$tree/scripts/hosts" "$tree/task-server" "$tree/$(dirname "$attack_ledger")" "$tree/$(dirname "$stage_split_product")"
  cp -- "$work/task" "$tree/task-server/task"
  cp -- "$SCRIPT_PATH" "$tree/$SCRIPT_KEY"
  for link in "$ITEM_VERDICTS" scripts/hosts/run-build-directory.sh scripts/hosts/run_build_directory.py .claude Cargo.lock "$attack_ledger" "$stage_split_product" "${INPUT_SOURCES[@]}"; do
    mkdir -p -- "$tree/$(dirname "$link")"
    ln -s -- "$real_root/$link" "$tree/$link"
  done
  # 样板树里的装置源码是假的一行；登记的 sha256 照它算、经 E185_R3_SELFTEST_REGISTERED_SHA256 交下去（「源码不等」那一格另给一个对不上的）
  mkdir -p -- "$tree/$(dirname "$apparatus_source")"
  echo "// 自证样板树里的假装置源码" > "$tree/$apparatus_source"
  fake_source_sha256=$(sha256sum "$tree/$apparatus_source" | cut -d' ' -f1)
  python3 "$real_root/$ITEM_VERDICTS" plant-fake-entries "$tree" || return 1
  git -C "$tree" init -q > /dev/null 2>&1
  printf '%s\tscripts/experiment-apparatus/long_run_reuse_fake_entries.py::measure_alpha\n' host-cpu-per-submission recompute-disagreement function-split > "$work/all-segments.tsv"
  make_tree() { mkdir -p -- "$1"; cp -R -- "$tree" "$1/tree"; }
  # 在调用它的子 shell 里 cd 进这一格的树、起本脚本（环境里给领卡通道，INVOKE_NO_CHANNEL 设了就不给；INVOKE_EXEC=exec 时这个子 shell 本身变成脚本，后台起时 $! 就是脚本的进程号）
  invoke_command() { # <格> <段…>
    local cell="$1" channel=() invoke_environment=()
    shift
    [[ -n "${INVOKE_NO_CHANNEL:-}" ]] || channel=(TASK_SERVER_CARD_LEASE_DIRECTORY="$work/channel" TASK_SERVER_CARD_LEASE_TOKEN=selftest-token)
    invoke_environment=(-u SINGLEFS_REUSE_SENTINEL_SEED -u TASK_SERVER_CARD_LEASE_DIRECTORY -u TASK_SERVER_CARD_LEASE_TOKEN
        PATH="$work/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E185_SELFTEST_APPARATUS="$work/apparatus" ${channel[@]+"${channel[@]}"}
        E185_SELFTEST_CALLS="$cell/calls" E185_SELFTEST_EVENTS="$cell/events" CARGO_TARGET_DIR="$cell/target" E185_PRODUCT_DIRECTORY="$cell/products" E185_RUN_DATE=2099-01-01
        E185_R3_SELFTEST_MARK="$work/mark" E185_R3_SELFTEST_REGISTERED_SHA256="${INVOKE_REGISTERED:-$fake_source_sha256}"
        LONG_RUN_ITEMS_LIBRARY_DIRECTORY="$cell/library" LONG_RUN_ITEMS_SELFTEST_ENTRIES="$work/all-segments.tsv" SINGLEFS_REUSE_SENTINEL_COUNT=1000)
    cd -- "$cell/tree" || return 1
    if [[ -n "${INVOKE_EXEC:-}" ]]; then
      exec env "${invoke_environment[@]}" bash "$cell/tree/$SCRIPT_KEY" "$@" > "$cell/out" 2>&1
    fi
    env "${invoke_environment[@]}" bash "$cell/tree/$SCRIPT_KEY" "$@" > "$cell/out" 2>&1
  }
  invoke() { ( invoke_command "$@" ); }
  # 这一格的事件记录（假 cargo 的 build、假领卡客户端的 lease、假装置的 apparatus）压成一行，竖线隔开；没有记录是空串
  events_of() { if [[ -f "$1/events" ]]; then paste -sd'|' < "$1/events"; fi; }
  calls_of() { if [[ -f "$1/calls${2:-}" ]]; then wc -l < "$1/calls${2:-}"; else echo 0; fi; }
  expect() {
    local name="$1" condition="$2"
    cell_count=$((cell_count + 1))
    if eval "$condition"; then
      echo "  ✓ e185-r3 自证：$name（第 $cell_count 格）"
    else
      echo "  ✗ e185-r3 自证：$name：条件「$condition」不成立"
      [[ -f "$cell/out" ]] && tail -n 6 -- "$cell/out" | sed 's/^/      输出：/'
      echo "     → 怎么办：照这一格点名的条件和输出改本脚本，再在临时 prune-server 底下单独起本脚本的 --selftest（文件头「用法」那一条）到它退 0"
      failures=$((failures + 1))
    fi
  }
  cell="$work/fresh"; make_tree "$cell"
  invoke "$cell" function-split; first_exit=$?
  invoke "$cell" function-split; exit_code=$?
  expect "一段现判、产物与输入清单写进产物目录；原样重起复用、装置不起、产物另存一份带 -2 的" '((first_exit == 0 && exit_code == 0)) && (($(calls_of "$cell") == 1)) && grep -qF "现判 0、复用 1、拒算 0" "$cell/out" && [[ -f "$cell/products/e185-gc-batch-wall-clock-split-function-split-r3-2099-01-01-2.out" ]] && grep -qF "复用了 prune-server 里的判定" "$cell/products/e185-r3-inputs-function-split-2099-01-01-2.txt"'
  cell="$work/unknown"; make_tree "$cell"
  invoke "$cell" no-such-segment; exit_code=$?
  expect "段名认不出：退 2、装置一次没起" '((exit_code == 2)) && grep -qF "段名认不出" "$cell/out" && (($(calls_of "$cell") == 0))'
  cell="$work/source-differs"; make_tree "$cell"
  INVOKE_REGISTERED=$(printf '0%.0s' {1..64}) invoke "$cell" function-split; exit_code=$?
  expect "装置源码的 sha256 与登记不等：退 3、装置一次没起" '((exit_code == 3)) && grep -qF "停机 J-S7" "$cell/out" && (($(calls_of "$cell") == 0))'
  cell="$work/counters"; make_tree "$cell"
  E185_SELFTEST_COUNTERS=1 invoke "$cell" function-split; exit_code=$?
  expect "cargo tree 里有 lookup-work-counters：退 6、装置一次没起" '((exit_code == 6)) && grep -qF "lookup-work-counters" "$cell/out" && (($(calls_of "$cell") == 0))'
  cell="$work/truncated"; make_tree "$cell"
  E185_SELFTEST_APPARATUS_MODE=truncated invoke "$cell" function-split; first_exit=$?
  cp -- "$cell/out" "$cell/out.first"
  invoke "$cell" function-split; exit_code=$?
  expect "末行不是 name=done 的段判红：退 1、记红；装置修好再起，那一段现判" '((first_exit == 1)) && grep -qF "· function-split：红，现判" "$cell/out.first" && ((exit_code == 0)) && grep -qF "现判 1、复用 0、拒算 0" "$cell/out"'
  # 按需领卡那八格（文件头「按需领卡」）：事件记录（events）是编译、领卡、起装置（带它看到的卡串）、还卡的先后
  cell="$work/no-card-segment"; make_tree "$cell"
  INVOKE_NO_CHANNEL=1 invoke "$cell" function-split; exit_code=$?
  expect "不要卡的段不领卡（没有领卡通道也照跑）：编译之后直接起装置、装置看不到卡串" '((exit_code == 0)) && [[ "$(events_of "$cell")" == "build|apparatus cards=none" ]]'
  cell="$work/ambient-cards"; make_tree "$cell"
  GPU_CARDS_BY_PRIORITY=ambient-card invoke "$cell" function-split; exit_code=$?
  expect "环境里原有的卡串开跑前清掉：不要卡的段的装置看不到它" '((exit_code == 0)) && [[ "$(events_of "$cell")" == *"apparatus cards="* && "$(events_of "$cell")" != *ambient-card* ]]'
  cell="$work/card-segment"; make_tree "$cell"
  invoke "$cell" host-cpu-per-submission; exit_code=$?
  expect "要卡的段领一次还一次：编译之后领卡、领到的卡串交给装置、装置退出就还" '((exit_code == 0)) && [[ "$(events_of "$cell")" == "build|lease take|apparatus cards=selftest-card|lease return" ]] && grep -qF "领卡：host-cpu-per-submission 段领到 selftest-card" "$cell/out" && grep -qF "还卡：selftest-card 还了" "$cell/out"'
  cell="$work/card-segment-red"; make_tree "$cell"
  E185_SELFTEST_APPARATUS_MODE=truncated invoke "$cell" host-cpu-per-submission; exit_code=$?
  expect "要卡的段装置判红也还卡" '((exit_code == 1)) && [[ "$(events_of "$cell")" == "build|lease take|apparatus cards=selftest-card|lease return" ]]'
  cell="$work/card-segment-interrupted"; make_tree "$cell"
  E185_SELFTEST_APPARATUS_MODE=hang INVOKE_EXEC=exec invoke_command "$cell" host-cpu-per-submission &
  interrupted_pid=$!
  for _ in $(seq 200); do [[ -e "$cell/events.ready" ]] && break; sleep 0.1; done
  kill -TERM "$interrupted_pid"   # 自己起的那一个：exec 之后后台子 shell 就是脚本
  : > "$cell/events.release"
  wait "$interrupted_pid"; exit_code=$?
  expect "要卡的段被打断也还卡：装置在跑时脚本收到 TERM，装置退出后退 143、收尾还卡" '((exit_code == 143)) && [[ "$(events_of "$cell")" == "build|lease take|apparatus cards=selftest-card|lease return" ]]'
  cell="$work/lease-refused"; make_tree "$cell"
  E185_SELFTEST_CARD_LEASE=refused invoke "$cell" host-cpu-per-submission; first_exit=$?
  refused_events=$(events_of "$cell"); refused_calls=$(calls_of "$cell")
  invoke "$cell" host-cpu-per-submission; exit_code=$?
  expect "领卡被拒的段不起、不进判定库、整趟退 1；再起时那一段现判" '((first_exit == 1)) && [[ "$refused_events" == "build|lease take" ]] && ((refused_calls == 0)) && ((exit_code == 0)) && grep -qF "现判 1、复用 0、拒算 0" "$cell/out"'
  cell="$work/return-unanswered"; make_tree "$cell"
  E185_SELFTEST_CARD_LEASE=return-unanswered invoke "$cell" host-cpu-per-submission; exit_code=$?
  expect "还卡没成只打一行、退出码不变" '((exit_code == 0)) && grep -qF "! 还卡没成：task-server/task lease return 退 1" "$cell/out" && [[ "$(events_of "$cell")" == "build|lease take|apparatus cards=selftest-card|lease return" ]]'
  cell="$work/no-channel"; make_tree "$cell"
  INVOKE_NO_CHANNEL=1 invoke "$cell" function-split host-cpu-per-submission; exit_code=$?
  expect "没有领卡通道而要现跑要卡的段：退 78、编译与装置都没起、点名通道变量" '((exit_code == 78)) && grep -qF "TASK_SERVER_CARD_LEASE_DIRECTORY" "$cell/out" && [[ -z "$(events_of "$cell")" ]] && (($(calls_of "$cell") == 0))'
  # 装置内存预算那一格（文件头「环境」）：多机配置（HOSTS 写 127.0.0.1 与一个文档用的假地址）里这一台那一项 50G、假地址那一项 1G，服务交 4000M：装置收到 50G 的字节数；
  # 多机配置不在：不设；调用方已设 E185_MEMORY_CAP：照留
  cell="$work/memory-budget"; mkdir -p "$cell"
  printf '%s\n' "HOSTS=127.0.0.1 192.0.2.70" \
    "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.70=/selftest/other-cargo" \
    "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.70=/selftest/other-builds" \
    "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=50G 192.0.2.70=1G" \
    "QUIESCE_STOP_COMMAND=" "QUIESCE_STOPPED_CHECK_COMMAND=" "QUIESCE_START_COMMAND=" "QUIESCE_STARTED_CHECK_COMMAND=" \
    "ENABLE_ACROSS_MACHINES=0" "ENABLE_GPU=0" "NFS_SHARED_ROOTS=$work" > "$cell/multi-host.env"
  memory_given=$(unset E185_MEMORY_CAP; export SINGLEFS_MULTI_HOST_CONFIG="$cell/multi-host.env" TASK_SERVER_MEMORY_CAP_BYTES=4194304000 E185_R3_SELFTEST_MARK="$work/mark"
    e185_r3_settle_memory_cap "$real_root"; echo "${E185_MEMORY_CAP:-unset}")
  memory_absent=$(unset E185_MEMORY_CAP; export SINGLEFS_MULTI_HOST_CONFIG="$cell/multi-host.env.absent" TASK_SERVER_MEMORY_CAP_BYTES=4194304000
    e185_r3_settle_memory_cap "$real_root"; echo "${E185_MEMORY_CAP:-unset}")
  memory_kept=$(export E185_MEMORY_CAP=3G SINGLEFS_MULTI_HOST_CONFIG="$cell/multi-host.env"
    e185_r3_settle_memory_cap "$real_root"; echo "${E185_MEMORY_CAP:-unset}")
  printf '这一台 50G、服务交 4000M，装置收到 %s（要 53687091200）；多机配置不在，装置收到 %s（要 unset）；调用方给了 3G，装置收到 %s（要 3G）\n' "$memory_given" "$memory_absent" "$memory_kept" > "$cell/out"
  expect "装置内存预算取这一台能给的内存、不取服务给的上限" '[[ "$memory_given" == 53687091200 && "$memory_absent" == unset && "$memory_kept" == 3G ]]'
  # 复用那六格（long_run_item_verdicts.py 文件头「复用格」）：两段一起起（一趟两段）
  reuse_output=$(python3 "$real_root/$ITEM_VERDICTS" reuse-cells --script "$SCRIPT_KEY" --template "$tree" --items host-cpu-per-submission,recompute-disagreement --work "$work/reuse" --green-exit 0 -- \
    env PATH="$work/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E185_SELFTEST_APPARATUS="$work/apparatus" E185_SELFTEST_CALLS="{tree}/../calls" CARGO_TARGET_DIR="{tree}/../target" \
    TASK_SERVER_CARD_LEASE_DIRECTORY="$work/channel" TASK_SERVER_CARD_LEASE_TOKEN=selftest-token \
    E185_PRODUCT_DIRECTORY="{tree}/../products" E185_RUN_DATE=2099-01-01 E185_R3_SELFTEST_MARK="$work/mark" E185_R3_SELFTEST_REGISTERED_SHA256="$fake_source_sha256" \
    bash "{tree}/$SCRIPT_KEY" host-cpu-per-submission recompute-disagreement 2>&1) || true
  echo "$reuse_output"
  reuse_red=$(sed -n 's/^long_run_item_verdicts 复用格.*：6 格，红 \([0-9]*\) 格$/\1/p' <<< "$reuse_output")
  cell_count=$((cell_count + 6))
  if [[ -z "$reuse_red" ]]; then failures=$((failures + 6)); echo "  ✗ e185-r3 自证：复用那六格没跑完（没有「long_run_item_verdicts 复用格」那一行），按六格红算"; echo "     → 怎么办：看上面 reuse-cells 打的输出里第一处 ✗ 或 Python 报错（复用格起不来多半是 prune-server 不可用或 long_run_item_verdicts.py 抛了异常），修好之后在临时 prune-server 底下单独起本脚本的 --selftest（文件头「用法」那一条）到它退 0"; else failures=$((failures + reuse_red)); fi
  echo "e185-r3-segment-run.sh 自证：$cell_count 格里 $failures 格红"
  (( failures == 0 ))
}

if [[ -n "${E185_R3_BREAK:-}" && "${1:-}" != --selftest ]] && [[ -z "${E185_R3_SELFTEST_MARK:-}" || "$(cat -- "${E185_R3_SELFTEST_MARK}" 2> /dev/null)" != e185-r3-selftest ]]; then
  echo "✗ E185_R3_BREAK 只在 --selftest 里认（弄坏开关只给证红用），这一趟什么都没做"
  echo "→ 怎么办：去掉 E185_R3_BREAK 再跑；要证红就在临时 prune-server 底下跑本脚本的 --selftest（起法见文件头「用法」）"
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then
  selftest
  exit $?
fi

(($# > 0)) || refuse_an_unknown_segment_name ""
segments=("$@")
for segment in "${segments[@]}"; do
  case "$segment" in
    host-cpu-per-submission | recompute-disagreement | function-split) ;;
    *) refuse_an_unknown_segment_name "$segment" ;;
  esac
done
# 卡只从领卡来（文件头「按需领卡」）：环境里原有的 GPU_CARDS_BY_PRIORITY 清掉，不要卡的段看不到卡。弄坏开关 ambient-cards-kept：不清
e185_r3_break_is ambient-cards-kept || unset GPU_CARDS_BY_PRIORITY
product_directory=${E185_PRODUCT_DIRECTORY:-research/results}
run_date=${E185_RUN_DATE:-$(TZ=Asia/Tokyo date +%F)}
mkdir -p "$product_directory"

eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"
e185_r3_settle_memory_cap "$PWD"
source scripts/hosts/run-build-directory.sh
run_directory=$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e185-r3-XXXXXX")
state_directory="$run_directory/items"
finish_the_run() {
  e185_r3_card_lease_return || true   # 打断（TERM / INT）时装置先退出，持着的卡在这里还
  rm -rf -- "$state_directory"
  run_build_directory_end || true
  rmdir "$run_directory" 2>/dev/null || true
}
trap finish_the_run EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
run_build_directory_begin "$run_directory"

apparatus_sha256=$(sha256sum "$apparatus_source" | cut -d' ' -f1)
declare -A product_of inputs_of
for segment in "${segments[@]}"; do
  # 同一天同一段再跑：两份一起加 -2、-3……，取两份都还不在的头一个后缀
  suffix=""
  next_rerun=2
  while [[ -e "$product_directory/e185-gc-batch-wall-clock-split-$segment-r3-$run_date$suffix.out" \
    || -e "$product_directory/e185-r3-inputs-$segment-$run_date$suffix.txt" ]]; do
    suffix="-$next_rerun"
    next_rerun=$((next_rerun + 1))
  done
  product_of[$segment]="$product_directory/e185-gc-batch-wall-clock-split-$segment-r3-$run_date$suffix.out"
  inputs_of[$segment]="$product_directory/e185-r3-inputs-$segment-$run_date$suffix.txt"
  build_features=()
  [[ -z "$(segment_features "$segment")" ]] || build_features=(--features "$(segment_features "$segment")")
  {
    echo "$apparatus_sha256  $apparatus_source"
    sha256sum "${INPUT_SOURCES[@]}" Cargo.lock
    echo "Cargo.lock 里 wgpu 的版本：$(grep -A1 '^name = "wgpu"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')"
    echo "HEAD $(git rev-parse HEAD 2>/dev/null || echo none)"
    case "$segment" in
      host-cpu-per-submission | recompute-disagreement)
        nvidia-smi --query-gpu=index,name,driver_version,pcie.link.gen.max,pcie.link.width.max,memory.total --format=csv ;;
      function-split) echo "第三段不起卡，不查 nvidia-smi" ;;
    esac
    if [[ "$segment" == host-cpu-per-submission ]]; then
      sha256sum "$attack_ledger" "$stage_split_product"
    fi
    cargo tree -p singlefs-implementation-experiments -e normal,build,features -i singlefs-core --offline ${build_features[@]+"${build_features[@]}"}
    echo "nproc $(nproc)"
    echo "getconf CLK_TCK $(getconf CLK_TCK)"
    echo "上限：外层不套内存包装（额度由任务服务给）；装置内存预算：$memory_cap_note；编译与装置的线程读服务交的变量（SINGLEFS_THREAD_CAP=${SINGLEFS_THREAD_CAP:-没设}，CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-没设}）"
    echo "E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256"
  } > "${inputs_of[$segment]}" 2>&1
done

if [[ "$apparatus_sha256" != "$registered_apparatus_sha256" ]] && ! e185_r3_break_is sha-unchecked; then
  echo "✗ 装置源码的 sha256 是 $apparatus_sha256，登记第十二节写的是 $registered_apparatus_sha256（停机 J-S7）"
  echo "→ 怎么办：核装置是不是换过；换过就交主 agent 改登记第十二节与这一份的 registered_apparatus_sha256，不照跑"
  exit 3
fi
for segment in "${segments[@]}"; do
  if grep -q 'lookup-work-counters' "${inputs_of[$segment]}" && ! e185_r3_break_is counters-unchecked; then
    echo "✗ 输入清单的 cargo tree 里有 lookup-work-counters：计数特性被并进来，这一段整段作废（登记第十一节）"
    echo "→ 怎么办：查是谁把 singlefs-core 的 lookup-work-counters 特性并进了 验证包的这一次编译，去掉再起"
    exit 6
  fi
done

mkdir -p "$state_directory"
for segment in "${segments[@]}"; do segment_request "$segment"; done > "$state_directory/requests.tsv"
plan_exit=0
python3 "$ITEM_VERDICTS" plan "$state_directory/plan" --script "$SCRIPT_KEY" --kind "$LIBRARY_KIND" --root "$PWD" --requests "$state_directory/requests.tsv" || plan_exit=$?
(( plan_exit == 0 )) || library_failure_exit plan "$plan_exit"
decision_of() { awk -F'\t' -v item="$1" '$1 == item {print $2}' "$state_directory/plan/decisions.tsv"; }
# 现跑的段里有要卡的，而环境里没有领卡通道：编译之前退 78（复用的段不跑装置，不要通道）
live_card_segments=()
for segment in "${segments[@]}"; do
  if segment_uses_card "$segment" && { [[ "$(decision_of "$segment")" != reuse ]] || e185_r3_break_is reuse-ignored; }; then live_card_segments+=("$segment"); fi
done
if ((${#live_card_segments[@]} > 0)) && ! e185_r3_card_lease_channel_present "${live_card_segments[*]}"; then exit 78; fi

first_failure=0
declare -A built_with
for segment in "${segments[@]}"; do
  product="${product_of[$segment]}"
  if [[ "$(decision_of "$segment")" == reuse ]] && ! e185_r3_break_is reuse-ignored; then
    cp -- "$state_directory/plan/outputs/$segment.out" "$product"
    echo "这一趟复用了 prune-server 里的判定（键相同、库里判绿）：产物照库里那一份写出，装置这一趟没起" >> "${inputs_of[$segment]}"
    echo "E185 第 3 次重跑 $segment 段：复用（键相同、库里判绿）：产物 $product（输入清单 ${inputs_of[$segment]}）"
    continue
  fi
  features=$(segment_features "$segment")
  if [[ -z "${built_with[x$features]:-}" ]]; then
    build_features=()
    [[ -z "$features" ]] || build_features=(--features "$features")
    cargo build --release -p singlefs-implementation-experiments ${build_features[@]+"${build_features[@]}"} --bin e185_gc_batch_wall_clock_split
    built_with[x$features]=1
  fi
  apparatus="$CARGO_TARGET_DIR/release/e185_gc_batch_wall_clock_split"
  read -r -a arguments <<< "$(segment_arguments "$segment")"
  for index in "${!arguments[@]}"; do
    case "${arguments[$index]}" in
      "<stage-split-product>") arguments[index]=$stage_split_product ;;
      "<attack-ledger>") arguments[index]=$attack_ledger ;;
    esac
  done
  # 要卡的段：编译之后、起装置之前领一张卡，领到的卡串交给装置；领卡没成这一段不起、不进判定库（弄坏开关 card-lease-refusal-ignored：照起）。
  # 弄坏开关 card-leased-for-every-segment：不要卡的段也领
  if segment_uses_card "$segment" || e185_r3_break_is card-leased-for-every-segment; then
    if e185_r3_card_lease_take "$segment"; then
      export GPU_CARDS_BY_PRIORITY="$held_card"
    elif ! e185_r3_break_is card-lease-refusal-ignored; then
      echo "  $segment 段本次未跑（领卡没成），装置没起，不记判定"
      (( first_failure != 0 )) || first_failure=1
      continue
    fi
  fi
  status=0
  E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256 "$apparatus" "${arguments[@]}" > "$product" 2> "$product.err" || status=$?
  unset GPU_CARDS_BY_PRIORITY
  e185_r3_card_lease_return   # 装置退出（退 0、判红、退 78）就还
  echo "E185 第 3 次重跑 $segment 段退出码 $status：产物 $product（输入清单 ${inputs_of[$segment]}）"
  if (( status == 78 )); then
    echo "  $segment 段本次未跑（装置退 78：取卡没过或没编 gpu 特性），不记判定"
    (( first_failure != 0 )) || first_failure=78
    continue
  fi
  red=()
  if (( status != 0 )) || { [[ "$(tail -n 1 -- "$product")" != "E7RESULT name=done "* ]] && ! e185_r3_break_is done-line-unchecked; }; then
    red=(--red)
    echo "✗ $segment 段判红：装置退 $status$( (( status == 0 )) && echo "、末行不是 E7RESULT name=done")（错误输出 $product.err）"
    (( first_failure != 0 )) || first_failure=$(( status == 0 ? 1 : status ))
  fi
  record_exit=0
  python3 "$ITEM_VERDICTS" record "$state_directory/plan" "$segment" "$status" "$product" ${red[@]+"${red[@]}"} || record_exit=$?
  (( record_exit == 0 )) || library_failure_exit "record（$segment 段）" "$record_exit"
done
finish_exit=0
python3 "$ITEM_VERDICTS" finish "$state_directory/plan" > "$state_directory/finish.out" || finish_exit=$?
cat -- "$state_directory/finish.out"
(( finish_exit <= 1 )) || library_failure_exit finish "$finish_exit"
! e185_r3_break_is finish-ignored || finish_exit=0
counts=$(sed -n 's/^LONG_RUN_ITEMS .* \(现判 [0-9]*、复用 [0-9]*、拒算 [0-9]*\)$/\1/p' "$state_directory/finish.out")
(( first_failure == 0 )) || { echo "→ 怎么办：看上面判红（或本次未跑、领卡没成）那几段的错误输出，修好之后同一条命令重起（判绿的段复用）；装置退 3 是停机条款，交主 agent"; exit "$first_failure"; }
(( finish_exit == 0 )) || { echo "✗ 按段判定收尾判红（复用哨兵或键在跑的中途变了，见上面 ✗ 那几行；$counts）"; echo "→ 怎么办：照上面那几行的出路办，再起同一条命令"; exit 1; }
echo "✓ E185 第 3 次重跑 ${#segments[@]} 段（${segments[*]}）跑完；按段判定 $counts"
