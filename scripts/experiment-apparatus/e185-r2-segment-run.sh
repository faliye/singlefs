#!/usr/bin/env bash
# admission: always 整个脚本每次都起：这一段的每一项已有没有判定、键对不对得上由 prune-server 里的按项判定现答（scripts/experiment-apparatus/long_run_item_verdicts.py）
# run-condition: command cargo python3 git sha256sum findmnt lsblk getconf
# E185（GC一批搬迁的挂钟拆分）第 2 次重跑一段的跑法（跑前登记 research/prompts/e185-r2-prereg.md 第五节「跑法」）：
#   写输入清单 → 有项要现判才在这一趟的编译目录（CARGO_TARGET_DIR：派到第二台的，multi-host-run.sh 在第二台本地盘上这一趟新建、跑完删；本机那一支，脚本里的 cargo 不在命令位置上，服务不设，是树里的 target）里编一次装置（crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs，
#   不带 gpu 特性）→ 跑这一段的各项（键相同、库里判绿的取库里的产物，不起装置）→ 产物写进产物目录。三趟先后跑，每趟一条命令，主 agent 在本机仓根起：
#     task-server/task submit -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r2-segment-run.sh <段> --disk-model <盘的型号>
#   这个入口整条交统一任务服务起，派哪台由服务定（records/2026-10-31-task-server巡检.md 问题 94：不登只派一台的表）；两台在同一棵共享的树里跑
#   （仓经 NFS 两台挂在同一绝对路径），产物（名字见「产物」）直接写进这棵树的 research/results/，两台都读得到，不拷回、不取；
#   输入清单里这一台本地盘上的目录（镜像目录），主 agent 换成「〔第二台的目录〕」「〔本机的目录〕」再留在 research/results/。
# 按项判定（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：一项是装置的一次起法，geometry-check 段两项（geometry-check、geometry-check-slot-order），
#   wall-clock、stage-split 段各一项；判定在 prune-server 的 apparatus-items 族、种类 e185_gc_batch_wall_clock_split（long_run_item_verdicts.py 文件头「判定库」）。
#   键 = 装置入口 main 的单元闭包（release 档；admission.py 的 compute_unit_key 算）、工具链、闭包真引到的依赖版本、这一项的装置参数（镜像目录只写成占位、不进键）、
#   wall-clock 另加点名的盘的型号；产物带计时，比法 compare=timing（只比绿红与退出码）。绿 = 装置退 0 且末行是 E7RESULT name=done。
#   键相同、库里判绿的项把库里那一份产物写到这一趟的产物名上、装置不起；判红的照同一把键记红撤绿、下一趟现判。成功那句报「现判 N、复用 M、拒算 R」。
#   一趟只有一项可复用时复用哨兵不足、不抽强制现判（long_run_item_verdicts.py 文件头「复用哨兵」），报一句、不红。
# 点盘（真盘臂量的是哪块盘）：--disk-model <型号> 必写，照 lsblk -s 的 MODEL 一栏逐字比（镜像目录所在设备往上各层里有一层是这个型号才算在这块盘上）。
#   点到的盘这一台没有（镜像目录不在那块盘上）就退 78、一步都不编不跑，出路写「在有那块盘的那一台跑，或把 E185_IMAGE_DIRECTORY 指到那块盘上的目录」，
#   不悄悄换一块盘量；量哪块盘照跑前登记第五节（登记写的是第二台那块盘的型号，主 agent 照登记填）。
#
# 用法：
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh geometry-check --disk-model <型号>    单点核：改法甲与旧造法各一条 geometry-check（P = 1024 两个占比）
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh wall-clock --disk-model <型号>        第一段
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh stage-split --disk-model <型号>       第二段
#   段名可以写几个（照写的次序跑），--disk-model 写在最后
#   bash scripts/experiment-apparatus/e185-r2-segment-run.sh --selftest    自证：点盘那两格（假 findmnt、假 lsblk 放在 PATH 最前面）与复用那六格（假 cargo、假装置），不编、不跑真装置；
#     判定写进临时 prune-server：只在临时服务底下跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem；不在就退 2、一格都不跑），起法：
#     python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e185-r2-segment-run.sh --selftest
# 弄坏开关（只在 --selftest 起的那几趟里认：自证设 E185_R2_SELFTEST_MARK 指到它建的标记文件，别处设了退 2）。逐项写开关名、弄坏哪一处，下一行起写打开后红的格（自证打「✗ e185-r2 自证：<格名>（…）」）：
#   E185_R2_BREAK=disk-unchecked  点盘不比型号、一律算在那块盘上
#       「e185-r2 自证：点到的盘这一台没有：退 78、点名型号与出路」
#   E185_R2_BREAK=reuse-ignored  库里判绿的项也照起装置现判（不认 plan 答的 reuse）
#       「reuse-three-questions」「key-independent-of-directory」「reuse-sentinel」「forced-still-reused-red」
#   E185_R2_BREAK=finish-ignored  不认 long_run_item_verdicts.py finish 的退出码（复用哨兵红、键中途变了照退 0）
#       「sentinel-rejudged-red」「forced-verdict-differs-red」「forced-still-reused-red」
#   复用那几格另认 long_run_item_verdicts.py 的弄坏开关 LONG_RUN_ITEMS_BREAK（它文件头；打开后本脚本的自证红在它点名的同名格）。
#   复用三问的假驱动：自证的假装置，geometry-check 段那两项的装置入口换成假树里两个 Python 函数（long_run_item_verdicts.py 文件头「复用格」）。
# 产物（<日期> 取 E185_RUN_DATE，没设取起跑当天 TZ=Asia/Tokyo date +%F）：
#   输入清单 e185-r2-inputs-<段>-<日期>.txt；单点核两份 e185-gc-batch-wall-clock-split-geometry-check-r2-<日期>.out、
#   e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-<日期>.out；第一、二段各一份 e185-gc-batch-wall-clock-split-<段>-r2-<日期>.out。
#   装置的标准错误各写同名加 .err（复用的项不起装置，不写 .err）。
# 环境：E185_PRODUCT_DIRECTORY（产物目录，默认 research/results）、E185_IMAGE_DIRECTORY（真盘臂的镜像目录，默认 <MULTI_HOST_RUN_PEER_SCRATCH>/e185-images：
#   派到第二台时 multi-host-run.sh 给的第二台本地这一趟的目录，不在 NFS 上、不在开机清空的目录里，跑完连目录删；没有这个变量取 ${CARGO_TARGET_DIR:-target}/e185-images）、E185_RUN_DATE。
#   镜像目录要落在 --disk-model 点名的那块盘上（上面「点盘」）。
# 上限：装置的内存预算不设 E185_MEMORY_CAP：装置取本进程 cgroup 的 memory.max（任务服务给的上限），读不到才取装置自己的缺省（memory_cap_given）；外层不套内存包装（额度由任务服务给）；线程不设，装置取登记值 .claude/heavy-thread-default。
# 退出码：0 跑完；1 有项判红而装置退 0（末行不齐）、复用哨兵红或键在跑的中途变了；2 段名认不出、没写 --disk-model、弄坏开关设在自证之外；75 prune-server 不可用；
#   78 点到的盘这一台没有（上面「点盘」）；别的照 cargo 与装置自己的退出码（装置退 3 是停机条款，见登记第十一节；几项里头一个退非 0 的那一个）。
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || { echo "进不去仓根"; echo "→ 怎么办：从仓根照文件头的用法起"; exit 2; }

SCRIPT_KEY="scripts/experiment-apparatus/e185-r2-segment-run.sh"
ITEM_VERDICTS="scripts/experiment-apparatus/long_run_item_verdicts.py"   # 按项判定的读写（文件头「按项判定」）
LIBRARY_KIND="e185_gc_batch_wall_clock_split"                            # 判定库的种类：装置的 bin 名
APPARATUS_ENTRY="crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs::main"

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）
e185_r2_break_is() {
  [[ ",${E185_R2_BREAK:-}," == *",$1,"* && -n "${E185_R2_SELFTEST_MARK:-}" ]] && [[ "$(cat -- "$E185_R2_SELFTEST_MARK" 2> /dev/null)" == e185-r2-selftest ]]
}

# 点盘（文件头「点盘」）：<镜像目录> <型号>；在那块盘上退 0，不在打原因与出路退 78
disk_check() {
  local directory=$1 wanted=$2 device models
  if e185_r2_break_is disk-unchecked; then return 0; fi
  device=$(findmnt -no SOURCE --target "$directory" 2>/dev/null) || device=""
  models=$( [[ -n "$device" ]] && lsblk -s -no MODEL "$device" 2>/dev/null | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | grep -v '^$' | tr '\n' '|' ) || models=""
  if [[ "|$models" == *"|$wanted|"* ]]; then return 0; fi
  echo "✗ 点到的盘这一台没有：要型号「$wanted」的盘，镜像目录 $directory 所在设备往上各层的型号是「${models%|}」（没读到算空）：这一趟一步都没编、没跑（退 78，运行条件不满足）"
  echo "→ 怎么办：在有那块盘的那一台跑（同一条命令重交，派哪台由服务定；这一台没有就会再退 78），或把 E185_IMAGE_DIRECTORY 指到那块盘上的目录；不换一块盘量"
  return 78
}

items_of_segment() { # <段>：这一段的项（装置的一次起法一项）
  case "$1" in
    geometry-check) echo geometry-check geometry-check-slot-order ;;
    wall-clock) echo wall-clock ;;
    stage-split) echo stage-split ;;
  esac
}

# 一项的装置参数，镜像目录写成占位 <镜像目录>（进键的那一份；起装置时换成这一趟的目录）
item_arguments() { # <项>
  case "$1" in
    geometry-check) echo "geometry-check --point 0.25:1024 --point 0.75:1024" ;;
    geometry-check-slot-order) echo "geometry-check --point 0.25:1024 --point 0.75:1024 --overwrite-order slot-order" ;;
    wall-clock) echo "wall-clock --image-directory <镜像目录>" ;;
    stage-split) echo "stage-split" ;;
  esac
}

item_request() { # <项> <盘的型号>（long_run_item_verdicts.py 文件头「请求文件」）
  local disk=""
  [[ "$1" == wall-clock ]] && disk=$'\t'"line:disk-model=$2"
  printf '%s\t%s\tprofile:release\tline:arguments=%s\tline:compare=timing%s\n' "$1" "$APPARATUS_ENTRY" "$(item_arguments "$1")" "$disk"
}

# 判定库那一步没成：75（prune-server 不可用）与 2（用法错）照交，别的按 1 交
library_failure_exit() { # <哪一步> <退出码>
  echo "✗ 按项判定的 $1 没成（long_run_item_verdicts.py 退 $2）"
  echo "→ 怎么办：退 75 是 prune-server 不可用：python3 prune-server/control check 看服务，修好之后重交同一条命令（报给主 agent）；别的照上面它打的出路改"
  if (($2 == 75 || $2 == 2)); then exit "$2"; fi
  exit 1
}

selftest() {
  local work exit_code failures=0 reuse_output reuse_red real_root="$PWD"
  python3 "$real_root/$ITEM_VERDICTS" selftest-service || return 2
  work=$(mktemp -d "${TMPDIR:-/tmp}/e185-r2-selftest.XXXXXX")
  trap 'rm -rf -- "$work"' RETURN
  echo e185-r2-selftest > "$work/mark"
  export E185_R2_SELFTEST_MARK="$work/mark"
  mkdir -p "$work/bin" "$work/images"
  printf '%s\n' '#!/usr/bin/env bash' 'echo /dev/selftest-partition' > "$work/bin/findmnt"
  printf '%s\n' '#!/usr/bin/env bash' 'printf "%s\n" "" "  SELFTEST DISK 2TB  "' > "$work/bin/lsblk"
  # 假 cargo：build 时把假装置放到 ${CARGO_TARGET_DIR:-target}/release 下（像真编出来的那一份），别的子命令（cargo tree）什么都不打
  printf '%s\n' '#!/usr/bin/env bash' '[[ "${1:-}" == build ]] || exit 0' 'mkdir -p "${CARGO_TARGET_DIR:-target}/release"' \
    'cp -- "$E185_SELFTEST_APPARATUS" "${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split"' > "$work/bin/cargo"
  # 假装置：照参数打一行和收尾行，退 0
  printf '%s\n' '#!/usr/bin/env bash' 'printf "E7RESULT name=row arguments=%s\nE7RESULT name=done emitted=2\n" "$*"' > "$work/apparatus"
  chmod +x "$work/bin/findmnt" "$work/bin/lsblk" "$work/bin/cargo" "$work/apparatus"
  exit_code=0; PATH="$work/bin:$PATH" disk_check "$work/images" "SELFTEST DISK 2TB" > "$work/match.out" || exit_code=$?
  if (( exit_code == 0 )); then echo "  ✓ e185-r2 自证：点到的盘在这一台：退 0"; else echo "  ✗ e185-r2 自证：点到的盘在这一台：退 0（退 $exit_code：$(cat "$work/match.out")）"; failures=$((failures + 1)); fi  # gate-lint:detail
  exit_code=0; PATH="$work/bin:$PATH" disk_check "$work/images" "OTHER DISK 4TB" > "$work/mismatch.out" || exit_code=$?
  if (( exit_code == 78 )) && grep -q '要型号「OTHER DISK 4TB」' "$work/mismatch.out" && grep -q '在有那块盘的那一台跑' "$work/mismatch.out"; then
    echo "  ✓ e185-r2 自证：点到的盘这一台没有：退 78、点名型号与出路"
  else
    echo "  ✗ e185-r2 自证：点到的盘这一台没有：退 78、点名型号与出路（退 $exit_code：$(cat "$work/mismatch.out")）"; failures=$((failures + 1))  # gate-lint:detail
  fi
  # 复用那六格（long_run_item_verdicts.py 文件头「复用格」）：样板树是本脚本的拷贝，crates、Cargo.lock、.claude、long_run_item_verdicts.py 是指回真仓的符号链接（输入清单 sha256 那几份照读）
  mkdir -p "$work/template/scripts/experiment-apparatus"
  cp -- "$SCRIPT_PATH" "$work/template/$SCRIPT_KEY"
  ln -s -- "$real_root/$ITEM_VERDICTS" "$work/template/$ITEM_VERDICTS"
  for link in .claude crates Cargo.lock; do ln -s -- "$real_root/$link" "$work/template/$link"; done
  git -C "$work/template" init -q > /dev/null 2>&1
  reuse_output=$(python3 "$real_root/$ITEM_VERDICTS" reuse-cells --script "$SCRIPT_KEY" --template "$work/template" --items geometry-check,geometry-check-slot-order --work "$work/reuse" --green-exit 0 -- \
    env PATH="$work/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E185_SELFTEST_APPARATUS="$work/apparatus" CARGO_TARGET_DIR="{tree}/../target" \
    E185_PRODUCT_DIRECTORY="{tree}/../products" E185_IMAGE_DIRECTORY="{tree}/../images" E185_RUN_DATE=2099-01-01 E185_R2_SELFTEST_MARK="$work/mark" \
    bash "{tree}/$SCRIPT_KEY" geometry-check --disk-model "SELFTEST DISK 2TB" 2>&1) || true
  echo "$reuse_output"
  reuse_red=$(sed -n 's/^long_run_item_verdicts 复用格.*：6 格，红 \([0-9]*\) 格$/\1/p' <<< "$reuse_output")
  if [[ -z "$reuse_red" ]]; then failures=$((failures + 6)); echo "  ✗ e185-r2 自证：复用那六格没跑完（没有「long_run_item_verdicts 复用格」那一行），按六格红算"; else failures=$((failures + reuse_red)); fi
  echo "e185-r2-segment-run.sh 自证：8 格里 $failures 格红"
  (( failures == 0 )) || echo "  → 怎么办：照上面 ✗ 那一格括号里的输出看 disk_check 的判法与假 lsblk 的样子，复用那几格看 long_run_item_verdicts.py 打的条件"
  (( failures == 0 ))
}

if [[ -n "${E185_R2_BREAK:-}" && "${1:-}" != --selftest ]] && [[ -z "${E185_R2_SELFTEST_MARK:-}" || "$(cat -- "${E185_R2_SELFTEST_MARK}" 2> /dev/null)" != e185-r2-selftest ]]; then
  echo "✗ E185_R2_BREAK 只在 --selftest 里认（弄坏开关只给证红用），这一趟什么都没做"
  echo "→ 怎么办：去掉 E185_R2_BREAK 再跑；要证红就在临时 prune-server 底下跑 E185_R2_BREAK=disk-unchecked bash scripts/experiment-apparatus/e185-r2-segment-run.sh --selftest（起法见文件头「用法」）"
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then
  selftest
  exit $?
fi

registered_apparatus_sha256=a5b48b734f034114a6e2fe3a81f17ecd171dafba5e5b5ec1082e1b150fa5f082
segments=()
while (($# > 0)) && [[ "$1" != --disk-model ]]; do
  case "$1" in
    geometry-check | wall-clock | stage-split) segments+=("$1") ;;
    *)
      echo "✗ 段名认不出：「$1」"
      echo "→ 怎么办：段名写 geometry-check、wall-clock、stage-split 之一（文件头的用法），可以写几个"
      exit 2
      ;;
  esac
  shift
done
if ((${#segments[@]} == 0)); then
  echo "✗ 没写段名"
  echo "→ 怎么办：第一个参数写 geometry-check、wall-clock、stage-split 之一（文件头的用法）"
  exit 2
fi
if [[ "${1:-}" != --disk-model || -z "${2:-}" || $# -ne 2 ]]; then
  echo "✗ 没写要量的那块盘：段名后面要写 --disk-model <型号>（文件头「点盘」）"
  echo "→ 怎么办：照跑前登记第五节写那块盘的型号（lsblk -o MODEL 打的那一栏），例 bash scripts/experiment-apparatus/e185-r2-segment-run.sh ${segments[0]} --disk-model <型号>"
  exit 2
fi
disk_model=$2
product_directory=${E185_PRODUCT_DIRECTORY:-research/results}
image_directory=${E185_IMAGE_DIRECTORY:-${MULTI_HOST_RUN_PEER_SCRATCH:-${CARGO_TARGET_DIR:-target}}/e185-images}
run_date=${E185_RUN_DATE:-$(TZ=Asia/Tokyo date +%F)}
mkdir -p "$product_directory" "$image_directory"
disk_exit=0; disk_check "$image_directory" "$disk_model" || disk_exit=$?
(( disk_exit == 0 )) || exit "$disk_exit"
eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"

for segment in "${segments[@]}"; do
  {
    sha256sum crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs \
      crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/mounted_session.rs crates/singlefs-core/src/region_scan.rs \
      crates/singlefs-core/src/compaction_plan.rs crates/singlefs-core/src/unit.rs crates/singlefs-core/src/checksum.rs \
      crates/singlefs-core/src/block_device.rs crates/singlefs-core/src/allocator.rs crates/singlefs-format/src/lib.rs Cargo.lock
    echo "HEAD $(git rev-parse HEAD 2>/dev/null || echo none)（两台都在主工作区那棵共享的树里跑，同一棵树）"
    echo "点名的盘：型号 $disk_model"
    cargo tree -p singlefs-verification -e normal,build,features -i singlefs-core --offline
    image_device=$(findmnt -no SOURCE --target "$image_directory")
    echo "镜像目录所在盘（lsblk -s，NAME MODEL LOG-SEC）："
    lsblk -s -no NAME,MODEL,LOG-SEC "$image_device"
    echo "镜像目录所在文件系统：$(findmnt -no FSTYPE --target "$image_directory")"
    echo "nproc $(nproc)"
    echo "getconf CLK_TCK $(getconf CLK_TCK)"
    echo "上限：外层不套内存包装（额度由任务服务给）；装置内存预算：E185_MEMORY_CAP=${E185_MEMORY_CAP:-没设，装置取本进程 cgroup 的 memory.max（任务服务给的上限）}"
    echo "E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256"
  } > "$product_directory/e185-r2-inputs-$segment-$run_date.txt" 2>&1
done

# 这一趟的状态目录（long_run_item_verdicts.py 的计划与复用项取回的产物）：跑完、失败、被打断都删
state_directory=$(mktemp -d "${TMPDIR:-/tmp}/e185-r2-items-XXXXXX")
trap 'rm -rf -- "$state_directory"' EXIT
items=()
for segment in "${segments[@]}"; do read -r -a segment_items <<< "$(items_of_segment "$segment")"; items+=("${segment_items[@]}"); done
for item in "${items[@]}"; do item_request "$item" "$disk_model"; done > "$state_directory/requests.tsv"
plan_exit=0
python3 "$ITEM_VERDICTS" plan "$state_directory/plan" --script "$SCRIPT_KEY" --kind "$LIBRARY_KIND" --root "$PWD" --requests "$state_directory/requests.tsv" || plan_exit=$?
(( plan_exit == 0 )) || library_failure_exit plan "$plan_exit"
decision_of() { awk -F'\t' -v item="$1" '$1 == item {print $2}' "$state_directory/plan/decisions.tsv"; }

product_stem=$product_directory/e185-gc-batch-wall-clock-split
need_build=0
for item in "${items[@]}"; do [[ "$(decision_of "$item")" == reuse ]] && ! e185_r2_break_is reuse-ignored || need_build=1; done
if (( need_build )); then
  cargo build --release -p singlefs-verification --bin e185_gc_batch_wall_clock_split
fi
apparatus=${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split

first_failure=0
for item in "${items[@]}"; do
  product="$product_stem-$item-r2-$run_date.out"
  if [[ "$(decision_of "$item")" == reuse ]] && ! e185_r2_break_is reuse-ignored; then
    cp -- "$state_directory/plan/outputs/$item.out" "$product"
    echo "项 $item：复用（键相同、库里判绿）：产物 $product 照库里那一份写出，装置没起"
    continue
  fi
  read -r -a arguments <<< "$(item_arguments "$item")"
  for index in "${!arguments[@]}"; do [[ "${arguments[$index]}" == "<镜像目录>" ]] && arguments[index]=$image_directory; done
  code=0
  E185_REGISTERED_APPARATUS_SHA256=$registered_apparatus_sha256 "$apparatus" "${arguments[@]}" > "$product" 2> "$product.err" || code=$?
  red=()
  if (( code != 0 )) || [[ "$(tail -n 1 -- "$product")" != "E7RESULT name=done "* ]]; then
    red=(--red)
    echo "✗ 项 $item 判红：装置退 $code$( (( code == 0 )) && echo "、末行不是 E7RESULT name=done")（错误输出 $product.err）"
    (( first_failure != 0 )) || first_failure=$(( code == 0 ? 1 : code ))
  else
    echo "项 $item：现判，装置退 0：产物 $product"
  fi
  record_exit=0
  python3 "$ITEM_VERDICTS" record "$state_directory/plan" "$item" "$code" "$product" ${red[@]+"${red[@]}"} || record_exit=$?
  (( record_exit == 0 )) || library_failure_exit "record（项 $item）" "$record_exit"
done
finish_exit=0
python3 "$ITEM_VERDICTS" finish "$state_directory/plan" > "$state_directory/finish.out" || finish_exit=$?
cat -- "$state_directory/finish.out"
(( finish_exit <= 1 )) || library_failure_exit finish "$finish_exit"
! e185_r2_break_is finish-ignored || finish_exit=0
counts=$(sed -n 's/^LONG_RUN_ITEMS .* \(现判 [0-9]*、复用 [0-9]*、拒算 [0-9]*\)$/\1/p' "$state_directory/finish.out")
(( first_failure == 0 )) || { echo "→ 怎么办：看上面判红那几项的错误输出，修好之后同一条命令重起（判绿的项复用）；装置退 3 是停机条款，交主 agent"; exit "$first_failure"; }
(( finish_exit == 0 )) || { echo "✗ 按项判定收尾判红（复用哨兵或键在跑的中途变了，见上面 ✗ 那几行；$counts）"; echo "→ 怎么办：照上面那几行的出路办，再起同一条命令"; exit 1; }
echo "✓ E185 第 2 次重跑 ${#segments[@]} 段（${segments[*]}）跑完：产物在 $product_directory（日期 $run_date）；按项判定 $counts"
