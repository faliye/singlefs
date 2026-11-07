#!/usr/bin/env bash
# admission: always 整个脚本每次都起：这一段已有没有判定、键对不对得上由 prune-server 里的按段判定现答（scripts/experiment-apparatus/long_run_item_verdicts.py）
# run-condition: command cargo python3 git sha256sum getconf nproc
# task-server-gpu-arguments: host-cpu-per-submission recompute-disagreement
# E185（GC一批搬迁的挂钟拆分）第 3 次重跑一段的跑法（跑前登记 research/prompts/e185-r3-prereg.md 第五节「跑法」）：
#   写输入清单 → 有段要现判才在这一趟自己的编译目录里编装置（crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs；
#   第一、二段带 gpu 特性，第三段不带；编译目录经 scripts/hosts/run-build-directory.sh 建、跑完删）→ 跑那一段 → 产物写进产物目录。
# 按段判定（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：一段一条判定，在 prune-server 的 apparatus-items 族、种类 e185_gc_batch_wall_clock_split
#   （long_run_item_verdicts.py 文件头「判定库」）。键 = 装置入口 main 的单元闭包（release 档、这一段的特性；admission.py 的 compute_unit_key 算）、工具链、闭包真引到的依赖版本、
#   这一段的装置参数（文件路径只写成占位）与它读的输入文件的内容（第一段读的两份）；产物带计时，比法 compare=timing（只比绿红与退出码）。绿 = 装置退 0 且末行是 E7RESULT name=done。
#   键相同、库里判绿的段把库里那一份产物写到这一趟的产物名上（输入清单末尾记一句复用了）、装置不起；判红的照同一把键记红撤绿、下一趟现判；装置退 78（本次未跑）不记。
#   成功那句报「现判 N、复用 M、拒算 R」。一趟只有一段可复用时复用哨兵不足、不抽强制现判（long_run_item_verdicts.py 文件头「复用哨兵」），报一句、不红。
# 准入与运行条件：
#   装置源码的 sha256 要等于登记第十二节写的那一个（下面 registered_apparatus_sha256）；不等就写完输入清单、退 3（停机 J-S7），不编、不跑。
#   输入清单里 `cargo tree` 那一段出现 lookup-work-counters（计数特性被并进来）就退 6，那一段整段作废（登记第十一节作废条款）。
#   第一、二段起卡：装置直接起，不套清场包装（任务服务对要卡的任务在最外层套清场；清场回读没过的由服务报，这一段记「本次未跑」）。
#   第三段不起卡。线程与内存不在脚本里套：服务给的变量（SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS）cargo 与装置直接读。
# 起法（主 agent 在仓根起）：
#   第一、二段在本机，先后起、不同时起：
#     task-server/task submit -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r3-segment-run.sh host-cpu-per-submission
#     跑完再起同一条换成 recompute-disagreement。
#   第三段（附带；主 agent 定了要量才起）：
#     task-server/task submit -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r3-segment-run.sh function-split
#     派哪台、清场与分卡由服务定（不套清场包装，不自己连第二台；records/2026-10-31-task-server巡检.md 问题 94），两台在同一棵共享的树里跑，
#     这一段的产物（名字见「产物」）直接写进 research/results/，不取回；输入清单里这一台本地盘上的目录，主 agent 换成「〔第二台的目录〕」「〔本机的目录〕」再留在 research/results/。
#
# 用法：
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh host-cpu-per-submission   第一段（本机，两张卡）
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh recompute-disagreement    第二段（本机，两张卡）
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh function-split            第三段（附带，第二台，不起卡）
#   段名可以写几个（照写的次序跑，一段一份产物）
#   bash scripts/experiment-apparatus/e185-r3-segment-run.sh --selftest   假 cargo、假 nvidia-smi、假装置在临时树里走一遍（不编、不起卡、不跑真装置）；判定写进临时 prune-server：
#     只在临时服务底下跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem；不在就退 2、一格都不跑），起法：
#     python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e185-r3-segment-run.sh --selftest
# 产物（<日期> 取 E185_RUN_DATE，没设取起跑当天 TZ=Asia/Tokyo date +%F；同一天同一段已有产物时两份一起加 -2、-3……，不盖已有的）：
#   e185-gc-batch-wall-clock-split-<段>-r3-<日期>.out（装置的标准错误写同名加 .err；复用的段不起装置，不写 .err）；
#   输入清单 e185-r3-inputs-<段>-<日期>.txt。
# 环境：E185_PRODUCT_DIRECTORY（产物目录，默认 research/results）、E185_RUN_DATE。线程不设，装置取登记值 .claude/heavy-thread-default（E185_THREADS
#   只在第二段造内容与 CPU 参照、第三段准备队列里用）；第三段装置的内存预算不设 E185_MEMORY_CAP：装置取本进程 cgroup 的 memory.max（任务服务给的上限），读不到才取装置自己的缺省（memory_cap_given）（装置按它定准备队列的并行条数）。
# 退出码：0 跑完；1 有段判红而装置退 0（末行不齐）、复用哨兵红或键在跑的中途变了；2 段名认不出、弄坏开关设在自证之外；3 装置源码的 sha256 与登记不等（停机 J-S7），或装置自己的停机条款；
#   6 计数特性被并进来（整段作废）；75 prune-server 不可用；78 取卡没过或没编 gpu 特性（本次未跑，装置退）；别的照 cargo 与装置自己的退出码（见装置文件头；几段里头一个退非 0 的那一个）。
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
registered_apparatus_sha256=a5b48b734f034114a6e2fe3a81f17ecd171dafba5e5b5ec1082e1b150fa5f082
apparatus_source=crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs
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
  printf '%s\tcrates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs::main\tprofile:release%s\tline:arguments=%s\tline:compare=timing\n' \
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

# 自证：样板树是本脚本的拷贝，.claude、Cargo.lock、输入清单记 sha256 的那几份源码、两份输入、run-build-directory 与 long_run_item_verdicts.py 是指回真仓的符号链接，
# 装置源码是假的一行（登记的 sha256 经 E185_R3_SELFTEST_REGISTERED_SHA256 换成它的，只在自证标记在时认）；
# 假 cargo（build 把假装置放进 CARGO_TARGET_DIR/release，tree 照 E185_SELFTEST_COUNTERS 打计数特性）、假 nvidia-smi 放在 PATH 最前面；不编、不起卡、不跑真装置。
selftest() {
  local work real_root="$PWD" failures=0 cell_count=0 cell exit_code first_exit reuse_output reuse_red link fake_source_sha256
  set +e   # 自证逐格收退出码，不让 -e 在头一个退非 0 的那一趟就收场
  python3 "$real_root/$ITEM_VERDICTS" selftest-service || return 2
  work=$(mktemp -d "${TMPDIR:-/tmp}/e185-r3-selftest.XXXXXX")
  trap 'rm -rf -- "$work"' RETURN
  echo e185-r3-selftest > "$work/mark"
  mkdir -p "$work/bin"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "cargo $*" >> "$E185_SELFTEST_CALLS.cargo"' \
    'if [[ "${1:-}" == tree ]]; then [[ -z "${E185_SELFTEST_COUNTERS:-}" ]] || echo "singlefs-core feature \"lookup-work-counters\""; exit 0; fi' \
    '[[ "${1:-}" == build ]] || exit 0' 'mkdir -p "$CARGO_TARGET_DIR/release"' 'cp -- "$E185_SELFTEST_APPARATUS" "$CARGO_TARGET_DIR/release/e185_gc_batch_wall_clock_split"' > "$work/bin/cargo"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "0, SELFTEST GPU, 1.0, 4, 16, 1 MiB"' > "$work/bin/nvidia-smi"
  # 假装置：记一笔，照参数打一行和收尾行（E185_SELFTEST_APPARATUS_MODE=truncated 不打收尾行），退 0
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*" >> "$E185_SELFTEST_CALLS"' 'printf "E7RESULT name=row arguments=%s\n" "$1"' \
    '[[ "${E185_SELFTEST_APPARATUS_MODE:-ok}" == truncated ]] || printf "E7RESULT name=done emitted=2\n"' > "$work/apparatus"
  chmod +x "$work/bin/cargo" "$work/bin/nvidia-smi" "$work/apparatus"
  local tree="$work/template"
  mkdir -p "$tree/scripts/experiment-apparatus" "$tree/scripts/hosts" "$tree/$(dirname "$attack_ledger")" "$tree/$(dirname "$stage_split_product")"
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
  invoke() {
    local cell="$1"
    shift
    ( cd -- "$cell/tree" && env -u SINGLEFS_REUSE_SENTINEL_SEED PATH="$work/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E185_SELFTEST_APPARATUS="$work/apparatus" \
        E185_SELFTEST_CALLS="$cell/calls" CARGO_TARGET_DIR="$cell/target" E185_PRODUCT_DIRECTORY="$cell/products" E185_RUN_DATE=2099-01-01 \
        E185_R3_SELFTEST_MARK="$work/mark" E185_R3_SELFTEST_REGISTERED_SHA256="${INVOKE_REGISTERED:-$fake_source_sha256}" \
        LONG_RUN_ITEMS_LIBRARY_DIRECTORY="$cell/library" LONG_RUN_ITEMS_SELFTEST_ENTRIES="$work/all-segments.tsv" \
        SINGLEFS_REUSE_SENTINEL_COUNT=1000 bash "$cell/tree/$SCRIPT_KEY" "$@" > "$cell/out" 2>&1 )
  }
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
  # 复用那六格（long_run_item_verdicts.py 文件头「复用格」）：两段一起起（一趟两段）
  reuse_output=$(python3 "$real_root/$ITEM_VERDICTS" reuse-cells --script "$SCRIPT_KEY" --template "$tree" --items host-cpu-per-submission,recompute-disagreement --work "$work/reuse" --green-exit 0 -- \
    env PATH="$work/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E185_SELFTEST_APPARATUS="$work/apparatus" E185_SELFTEST_CALLS="{tree}/../calls" CARGO_TARGET_DIR="{tree}/../target" \
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
product_directory=${E185_PRODUCT_DIRECTORY:-research/results}
run_date=${E185_RUN_DATE:-$(TZ=Asia/Tokyo date +%F)}
mkdir -p "$product_directory"

eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"
source scripts/hosts/run-build-directory.sh
run_directory=$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e185-r3-XXXXXX")
state_directory="$run_directory/items"
finish_the_run() {
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
    cargo tree -p singlefs-verification -e normal,build,features -i singlefs-core --offline ${build_features[@]+"${build_features[@]}"}
    echo "nproc $(nproc)"
    echo "getconf CLK_TCK $(getconf CLK_TCK)"
    echo "上限：外层不套内存包装（额度由任务服务给）；第三段装置内存预算：E185_MEMORY_CAP=${E185_MEMORY_CAP:-没设，装置取本进程 cgroup 的 memory.max（任务服务给的上限）}；编译与装置的线程读服务交的变量（SINGLEFS_THREAD_CAP=${SINGLEFS_THREAD_CAP:-没设}，CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-没设}）"
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
    cargo build --release -p singlefs-verification ${build_features[@]+"${build_features[@]}"} --bin e185_gc_batch_wall_clock_split
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
  status=0
  E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256 "$apparatus" "${arguments[@]}" > "$product" 2> "$product.err" || status=$?
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
(( first_failure == 0 )) || { echo "→ 怎么办：看上面判红（或本次未跑）那几段的错误输出，修好之后同一条命令重起（判绿的段复用）；装置退 3 是停机条款，交主 agent"; exit "$first_failure"; }
(( finish_exit == 0 )) || { echo "✗ 按段判定收尾判红（复用哨兵或键在跑的中途变了，见上面 ✗ 那几行；$counts）"; echo "→ 怎么办：照上面那几行的出路办，再起同一条命令"; exit 1; }
echo "✓ E185 第 3 次重跑 ${#segments[@]} 段（${segments[*]}）跑完；按段判定 $counts"
