#!/usr/bin/env bash
# admission: always 每一次调照服务给的那一步跑；实验一级的准入在计划那一步问 admission.py experiment … E183/h2（键与入口登记在 .claude/gate.d/stage-inputs.tsv），上一次跑完不替这一次作保
# run-condition: command bash python3 sha256sum stat cargo
# task-server-shards: run
# E183（容量斜率读缓存小盘准入与清扫阈值的代价数）第二段（问题单 H2）的跑法，照第 2 次重跑登记 research/prompts/e183-r2-prereg.md 5.3（X22–X27）：
#   装置是入库装置 crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs（bin 同名），一格一轮是装置的一个进程 h2 shard <片名> …；
#   统一任务服务照上面那一行分片声明把 run 拆成计划、各片（一片一轮，5 片）、合并三步（task-server/task_server_lib.py 文件头「分片」）。
#   任务里不套任何包装：内存上限、哪台机器、线程都归服务（编译的线程读服务交的 CARGO_BUILD_JOBS）；「内存上限之外」由装置读进程高水位、与登记写死的阈值比（X22）。
#
# 用法（仓里任何目录起都行，脚本自己回到仓根）：
#   task-server/task submit -- bash scripts/experiment-apparatus/e183-capacity-cells.sh run
#       只经服务起：环境里没有 TASK_SERVER_SHARD_PHASE 就退 2、一步不做。
#       计划（plan）：问准入 python3 scripts/gate/admission.py experiment <仓根> E183/h2（退 0 往下，77 整步退 77，3 退 78，别的退 2）；运行目录 =
#         <运行根>/<准入 sha256 的前 16 个十六进制字符>/，准入的标准输出存成 admission-header.txt（已有就核逐字节相同，不同退 2）；运行目录里没有程序文件就编一次
#         （cargo build --release；编译目录：第二台取 MULTI_HOST_RUN_PEER_SCRATCH/target，本机取服务给的 CARGO_TARGET_DIR，都没有取 <运行目录>/target、编完删），
#         拷成运行目录里的 e183_capacity_read_cache_reserve_and_sweep_costs、记 binary.sha256；已有就照用（重交时同一份二进制、同一个 sha256，合并只认它）。
#         计划目录写 plan.json（label 是 round-1 … round-5）与 run-directory（运行目录的绝对路径）；运行根下 latest-run-directory 记最近一次的运行目录。
#       各片（shard，TASK_SERVER_SHARD=<片号>/5，轮 = 片号 + 1）：台与镜像目录照 X24——有 MULTI_HOST_RUN_PEER_SCRATCH 是第二台、镜像放它下面的 e183-images；
#         没有是本机、镜像放服务给这一次尝试的私有临时目录 TMPDIR 下的 e183-images（本地盘，跑完连目录删）。镜像目录在 nfs 上、没有 TMPDIR、
#         程序文件在这一台起 h2 list-shards 起不来，都退 78（T8）。第几次尝试读 TASK_SERVER_ATTEMPT。一轮一台：这一轮已做完的格记的台与这一次不同，
#         这一轮做完的格全部挪成 <片名>.other-host-<序号>（不删），整轮重跑。照 h2 list-shards 的次序（一轮之内大的先）逐格：做完的沿用；
#         有 <片名>.service-outcome 的跳过（服务已给这一格定了结局）；其余起
#         h2 shard … --host <本机|第二台> --service-attempt <片号>/5:<第几次> --image-filesystem <类型>，前后各记一次 /proc/loadavg 与时刻。
#       合并（merge）：h2 merge <运行目录> 的输出齐了（末行 name=done、行数对得上、没有 name=stop），把 admission-header.txt 接在最前面，
#         落成 research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-<日期>.out（已占就 -r<序号>，不覆盖）。
#   bash scripts/experiment-apparatus/e183-capacity-cells.sh --merge-only [--run-directory <目录>]   只合并（没给目录取 latest-run-directory）
#   bash scripts/experiment-apparatus/e183-capacity-cells.sh --list-items [--run-directory <目录>]   逐行列各轮各格做没做完（要运行目录里已有程序文件，不编译）
#   bash scripts/experiment-apparatus/e183-capacity-cells.sh --selftest                              假装置、假编译、假准入走一遍 11 格（不起 cargo）
# 环境（只给自证换假的，平常不设）：E183_RUN_ROOT（运行根，默认工作根下的 e183-h2-run，工作根取 .claude/hooks/lib_work_root.py、两台同一份）、
#   E183_PRODUCT_DIRECTORY（默认 research/results）、E183_ADMISSION_SCRIPT（默认 scripts/gate/admission.py）。
# 退出码：0 这一步做完（合并时产物出了）；1 片没做完、编不过、合并没出齐（同一条命令重交只补没做完的）；2 用法错、不经服务起 run、准入登记有错、准入头对不上；
#   3 装置报停机（name=stop 那一行交主 agent）；77 准入说同一把键已有产物（整件不跑）；78 准入前提没齐、镜像目录在 nfs 上、没有本地临时目录、程序文件起不来、
#   --list-items 与 --merge-only 时程序文件不在。
#
# 弄坏开关（只在 --selftest 起的那几趟里认；每条下面缩进的是打开后该红的自证格）：
#   E183_CELLS_BREAK=admission-header-dropped  计划不把准入那几行存进运行目录、合并不接
#       「计划：准入退 0 时写 5 片的 plan.json、运行目录按键起名、准入那几行落进运行目录」「合并：产物最前面是准入那几行、不覆盖已有的产物名」
#   E183_CELLS_BREAK=admission-exit-swallowed  准入退非 0 也往下走
#       「计划：准入退 77 时整步退 77、不编译；退 3 时退 78」
#   E183_CELLS_BREAK=binary-rebuilt  运行目录里已有程序文件也重编
#       「计划：运行目录里已有程序文件就不再编」
#   E183_CELLS_BREAK=order-ascending  一轮之内倒过来、小的先起
#       「片：一轮之内大的先起；做完的沿用」
#   E183_CELLS_BREAK=service-outcome-ignored  不看服务结局文件
#       「片：有 .service-outcome 的格跳过」
#   E183_CELLS_BREAK=other-host-kept  别的台做完的格留着照用
#       「片：这一轮有别的台做完的格，挪成 .other-host-<序号>、整轮重跑」
#   E183_CELLS_BREAK=nfs-image-accepted  镜像目录在 nfs 上照跑
#       「片：镜像目录是 nfs 退 78；本机没有 TMPDIR 退 78；程序文件起不来退 78」
#   E183_CELLS_BREAK=attempt-argument-dropped  交给装置的参数里不带 --service-attempt
#       「片：交给装置的参数里有 --host、--service-attempt、--image-filesystem 且值对，每格前后各记一次负载」
#   E183_CELLS_BREAK=phase-unchecked  不经服务起 run 不拦（照计划那一步往下走）
#       「不经服务起 run 退 2」
#   E183_CELLS_BREAK=wrapper-reintroduced  静态核的那一份正文里多一行包装
#       「脚本正文里命令位置上没有包装（静态核）」
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; echo "     → 怎么办：在仓里起这份脚本（它自己回到仓根；进不去多半是脚本被挪出了 scripts/experiment-apparatus/）" >&2; exit 2; }
REPOSITORY_ROOT="$PWD"

BINARY_NAME=e183_capacity_read_cache_reserve_and_sweep_costs
ADMISSION_KEY=E183/h2
ROUNDS=5
PRODUCT_STEM_NAME=e183-capacity-read-cache-reserve-and-sweep-costs-h2
PRODUCT_DIRECTORY="${E183_PRODUCT_DIRECTORY:-research/results}"
ADMISSION_SCRIPT="${E183_ADMISSION_SCRIPT:-scripts/gate/admission.py}"
# 静态核找的三样（内存包装、线程包装、降优先级的前缀）：拼开写，这一份正文自己不命中。
WRAPPER_PATTERN="run-with-memory""-cap|capped""\\.sh|ni""ce -n"

usage_error() {
  echo "  ✗ $1" >&2
  echo "     → 怎么办：见本脚本文件头「用法」" >&2
  exit 2
}

break_is() {
  [[ ",${E183_CELLS_BREAK:-}," == *",$1,"* ]]
}
if [[ -n "${E183_CELLS_BREAK:-}" && "${1:-}" != --selftest ]]; then
  if [[ -z "${E183_CELLS_SELFTEST_MARK:-}" || "$(cat -- "${E183_CELLS_SELFTEST_MARK}" 2> /dev/null)" != e183-cells-selftest ]]; then
    echo "  ✗ E183_CELLS_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
    echo "     → 怎么办：去掉 E183_CELLS_BREAK 再跑；要证红就跑 E183_CELLS_BREAK=<项> bash scripts/experiment-apparatus/e183-capacity-cells.sh --selftest" >&2
    exit 2
  fi
fi

# 运行根：E183_RUN_ROOT，没给取工作根下的 e183-h2-run（工作根两台同一个绝对路径）。
run_root() {
  local root
  if [[ -n "${E183_RUN_ROOT:-}" ]]; then
    echo "$E183_RUN_ROOT"
    return 0
  fi
  if ! root="$(python3 .claude/hooks/lib_work_root.py)" || [[ "$root" != /* ]]; then
    echo "  ✗ 取不到工作根（.claude/hooks/lib_work_root.py 跑不起来），运行目录没处放" >&2
    echo "     → 怎么办：修好 .claude/hooks/lib_work_root.py 再重交" >&2
    return 1
  fi
  echo "$root/e183-h2-run"
}

# 一格做完了：文件在，末行是 E7RESULT name=shard_done shard=<片名> lines=<整份行数>。
is_shard_done() {
  local file="$1" name line_count last_line
  [[ -f "$file" ]] || return 1
  name=$(basename -- "$file")
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" == "E7RESULT name=shard_done shard=$name lines=$line_count" ]]
}

is_complete_product() {
  local file="$1" line_count last_line
  [[ -f "$file" ]] || return 1
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" == "E7RESULT name=done emitted=$line_count" ]] && ! grep -q '^E7RESULT name=stop ' -- "$file"
}

next_free_product_path() {
  local today revision stem="$PRODUCT_DIRECTORY/$PRODUCT_STEM_NAME"
  today=$(TZ=Asia/Tokyo date +%F)
  [[ -e "$stem-$today.out" ]] || { echo "$stem-$today.out"; return; }
  revision=2
  while [[ -e "$stem-$today-r$revision.out" ]]; do revision=$((revision + 1)); done
  echo "$stem-$today-r$revision.out"
}

# 运行目录里的程序文件：已有且与 binary.sha256 相符就照用；没有就编一次（编译目录见文件头「计划」）。
prepare_binary_in() {
  local run_directory="$1" binary="$1/$BINARY_NAME" target own_target=0 build_exit sha256
  if [[ -x "$binary" && -f "$run_directory/binary.sha256" ]] && ! break_is binary-rebuilt; then
    sha256=$(sha256sum -- "$binary" | cut -d' ' -f1)
    if [[ "$sha256" == "$(cat -- "$run_directory/binary.sha256")" ]]; then
      echo "程序文件：运行目录里已有（sha256 $sha256），照用、不编"
      return 0
    fi
    echo "  ✗ 运行目录里的程序文件与 binary.sha256 记的不符（$binary）" >&2
    echo "     → 怎么办：查谁改了它；要重编就删掉 $binary 与 binary.sha256 再重交（换了二进制，已做完的格合并时记 binary_other_than_the_merge）" >&2
    return 1
  fi
  if [[ -n "${MULTI_HOST_RUN_PEER_SCRATCH:-}" ]]; then
    target="$MULTI_HOST_RUN_PEER_SCRATCH/target"
  elif [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    target="$CARGO_TARGET_DIR"
  else
    target="$run_directory/target"
    own_target=1
  fi
  mkdir -p -- "$run_directory/logs" || return 1
  CARGO_TARGET_DIR="$target" cargo build --release -p singlefs-verification --bin e183_capacity_read_cache_reserve_and_sweep_costs \
    > "$run_directory/logs/build-$$.log" 2>&1
  build_exit=$?
  if ((build_exit == 0)); then
    { cp -- "$target/release/$BINARY_NAME" "$binary.partial-$$" && mv -f -- "$binary.partial-$$" "$binary"; } || build_exit=1
  fi
  ((own_target == 0)) || rm -rf -- "${target:?}"
  if ((build_exit != 0)); then
    echo "  ✗ 装置编不过或拷不进运行目录（$run_directory/logs/build-$$.log）" >&2
    echo "     → 怎么办：看那份编译日志改到编得过，再同一条命令重交" >&2
    return 1
  fi
  sha256sum -- "$binary" | cut -d' ' -f1 > "$run_directory/binary.sha256" || return 1
  echo "程序文件：编了一次（sha256 $(cat -- "$run_directory/binary.sha256")）"
}

# 计划（文件头「用法」plan）。
plan_step() {
  local plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}" root header admission_exit key run_directory
  [[ -n "$plan_directory" && -d "$plan_directory" ]] || usage_error "计划那一步要 TASK_SERVER_SHARD_PLAN_DIRECTORY 指到服务建好的计划目录"
  root=$(run_root) || exit 2
  mkdir -p -- "$root" || exit 1
  header=$(mktemp "$root/admission-header.XXXXXX") || exit 1
  python3 "$ADMISSION_SCRIPT" experiment "$REPOSITORY_ROOT" "$ADMISSION_KEY" > "$header"
  admission_exit=$?
  if ((admission_exit != 0)) && ! break_is admission-exit-swallowed; then
    rm -f -- "$header"
    case "$admission_exit" in
      77) echo "准入：键 $ADMISSION_KEY 已有一份产物头记着同一把键（admission.py 退 77），这一件不跑"; exit 77 ;;
      3) echo "  ✗ 准入前提没齐（admission.py 退 3）" >&2
         echo "     → 怎么办：看上面 admission.py 的说明补齐前提再重交" >&2
         exit 78 ;;
      *) echo "  ✗ 准入退 $admission_exit（登记或调用有错）" >&2
         echo "     → 怎么办：核 .claude/gate.d/stage-inputs.tsv 里键 $ADMISSION_KEY 那一行，单跑 python3 scripts/gate/admission.py experiment <仓根> $ADMISSION_KEY 看说明" >&2
         exit 2 ;;
    esac
  fi
  key=$(grep -m1 -oE 'sha256=[0-9a-f]{16}' -- "$header" | cut -d= -f2)
  if [[ ! "$key" =~ ^[0-9a-f]{16}$ ]]; then
    if break_is admission-exit-swallowed; then
      key=not-admitted
    else
      rm -f -- "$header"
      echo "  ✗ 准入退 0 却没给出带 sha256= 的 E7INPUT 行" >&2
      echo "     → 怎么办：单跑 python3 scripts/gate/admission.py experiment <仓根> $ADMISSION_KEY 看它打了什么，修准入模块" >&2
      exit 2
    fi
  fi
  run_directory="$root/$key"
  mkdir -p -- "$run_directory/logs" || exit 1
  if break_is admission-header-dropped; then
    rm -f -- "$header"
  elif [[ -f "$run_directory/admission-header.txt" ]]; then
    if ! cmp -s -- "$header" "$run_directory/admission-header.txt"; then
      rm -f -- "$header"
      echo "  ✗ 这一次准入的输出与运行目录里存的 admission-header.txt 不同（$run_directory）" >&2
      echo "     → 怎么办：同一把键的准入头应逐字节相同；查 admission.py 的输出为什么变了，交主 agent 定，不删运行目录" >&2
      exit 2
    fi
    rm -f -- "$header"
  else
    mv -- "$header" "$run_directory/admission-header.txt" || exit 1
  fi
  prepare_binary_in "$run_directory" || exit 1
  printf '%s\n' "$run_directory" > "$plan_directory/run-directory" || exit 1
  python3 -c 'import json, sys
json.dump({"shards": [{"label": "round-%d" % round_number} for round_number in range(1, int(sys.argv[2]) + 1)],
           "note": "E183 第二段：一片一轮"}, open(sys.argv[1], "w"), ensure_ascii=False)' "$plan_directory/plan.json" "$ROUNDS" || exit 1
  printf '%s\n' "$run_directory" > "$root/latest-run-directory.partial-$$" && mv -f -- "$root/latest-run-directory.partial-$$" "$root/latest-run-directory"
  echo "计划：运行目录 $run_directory，程序文件 sha256 $(cat -- "$run_directory/binary.sha256")，$ROUNDS 片（一片一轮）"
}

# 计划目录里记的运行目录（各片与合并都从这里读）。
run_directory_of_the_plan() {
  local plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}" run_directory
  [[ -n "$plan_directory" && -f "$plan_directory/run-directory" ]] || usage_error "这一步要 TASK_SERVER_SHARD_PLAN_DIRECTORY 指到计划写过 run-directory 的计划目录"
  run_directory=$(cat -- "$plan_directory/run-directory")
  [[ -d "$run_directory" ]] || usage_error "计划记的运行目录不在：$run_directory"
  echo "$run_directory"
}

# 运行目录里做完的这一格挪成下一个没用过的 <片名>.other-host-<序号>（一轮一台，X23）。
move_aside_as_other_host() {
  local shard_file="$1" sequence=1
  while [[ -e "$shard_file.other-host-$sequence" ]]; do sequence=$((sequence + 1)); done
  mv -n -- "$shard_file" "$shard_file.other-host-$sequence"
}

# 一片（文件头「用法」shard）：跑这一轮的 9 格。
shard_step() {
  local shard_count round attempt filesystem name line recorded other_host=0 moved=0 ran=0 reused=0 skipped=0 exit_code log log_stem
  local -a round_shards arguments
  RUN_DIRECTORY=$(run_directory_of_the_plan) || exit 2
  [[ -n "${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}" ]] || usage_error "片那一步要 TASK_SERVER_SHARD_OUTPUT_DIRECTORY"
  [[ "${TASK_SERVER_SHARD:-}" =~ ^([0-9]+)/([0-9]+)$ ]] || usage_error "片那一步要 TASK_SERVER_SHARD=<片号>/<片数>"
  round=$((BASH_REMATCH[1] + 1))
  shard_count="${BASH_REMATCH[2]}"
  ((shard_count == ROUNDS && round >= 1 && round <= ROUNDS)) || usage_error "片数应是 $ROUNDS、片号在 0 到 $((ROUNDS - 1))：TASK_SERVER_SHARD=${TASK_SERVER_SHARD}"
  attempt="${TASK_SERVER_ATTEMPT:-}"
  [[ "$attempt" =~ ^[0-9]+$ ]] || usage_error "取不到这是第几次尝试（服务设的 TASK_SERVER_ATTEMPT）"
  # 程序文件名写成字面尾巴（与 BINARY_NAME 同一个名字）：函数单元键把命令位置上「$目录/变量」折不出的程序拒算，实验键就算不出。
  BINARY="$RUN_DIRECTORY/e183_capacity_read_cache_reserve_and_sweep_costs"
  SHA256=$(cat -- "$RUN_DIRECTORY/binary.sha256" 2> /dev/null) || usage_error "运行目录里没有 binary.sha256：计划那一步没做完"
  if [[ -n "${MULTI_HOST_RUN_PEER_SCRATCH:-}" ]]; then
    HOST=第二台
    IMAGE_DIRECTORY="$MULTI_HOST_RUN_PEER_SCRATCH/e183-images"
  elif [[ -n "${TMPDIR:-}" ]]; then
    HOST=本机
    IMAGE_DIRECTORY="$TMPDIR/e183-images"
  else
    echo "  ✗ 本机这一次尝试没有私有临时目录（TMPDIR 没设），镜像没有本地盘可放；一格都没起" >&2
    echo "     → 怎么办：只经 task-server 交（服务给每一次尝试一个本地的私有临时目录）；照登记 T8 停下，task status 与 task log 交主 agent" >&2
    exit 78
  fi
  mkdir -p -- "$IMAGE_DIRECTORY" "$RUN_DIRECTORY/logs" || exit 1
  trap 'rm -rf -- "${IMAGE_DIRECTORY:?}"' EXIT
  filesystem=$(stat -f -c %T -- "$IMAGE_DIRECTORY") || { echo "  ✗ 读不出镜像目录的文件系统类型（$IMAGE_DIRECTORY）" >&2; echo "     → 怎么办：照登记 T8 停下，交主 agent" >&2; exit 78; }
  if [[ "$filesystem" == nfs* ]] && ! break_is nfs-image-accepted; then
    echo "  ✗ 镜像目录在 $filesystem 上（$IMAGE_DIRECTORY）：镜像要放跑的那一台的本地盘（X24）；一格都没起" >&2
    echo "     → 怎么办：照登记 T8 停下，task status 与 task log 交主 agent 问管任务服务的会话" >&2
    exit 78
  fi
  log="$RUN_DIRECTORY/logs/round-$round-attempt-$attempt.log"
  if ! "$BINARY" h2 list-shards > "$RUN_DIRECTORY/logs/list-shards-round-$round-attempt-$attempt.txt" 2>> "$log"; then
    echo "  ✗ 程序文件在这一台起不来（$BINARY h2 list-shards 退非 0，$log）；一格都没起" >&2
    echo "     → 怎么办：照登记 T8 停下，交主 agent（一份二进制两台都要起得来）" >&2
    exit 78
  fi
  mapfile -t round_shards < <(awk -F'\t' -v round="$round" '$2 == round { print $1 }' "$RUN_DIRECTORY/logs/list-shards-round-$round-attempt-$attempt.txt")
  ((${#round_shards[@]} > 0)) || { echo "  ✗ h2 list-shards 里第 $round 轮一格都没有" >&2; echo "     → 怎么办：单跑 $BINARY h2 list-shards 看装置列了什么" >&2; exit 1; }
  if break_is order-ascending; then
    mapfile -t round_shards < <(printf '%s\n' "${round_shards[@]}" | tac)
  fi
  echo "第 $round 轮（第 $attempt 次尝试，台：$HOST，镜像目录 $IMAGE_DIRECTORY 在 $filesystem 上）：${#round_shards[@]} 格" | tee -a "$log"
  if ! break_is other-host-kept; then
    for name in "${round_shards[@]}"; do
      is_shard_done "$RUN_DIRECTORY/$name" || continue
      recorded=$(grep -m1 '^E7RESULT name=h2_config ' -- "$RUN_DIRECTORY/$name" | grep -oE ' host=[^ ]+' | cut -d= -f2)
      [[ "$recorded" == "$HOST" ]] || other_host=1
    done
    if ((other_host == 1)); then
      for name in "${round_shards[@]}"; do
        is_shard_done "$RUN_DIRECTORY/$name" || continue
        move_aside_as_other_host "$RUN_DIRECTORY/$name" || exit 1
        moved=$((moved + 1))
      done
      echo "一轮一台：这一轮有别的台做完的格，做完的 $moved 格挪成 <片名>.other-host-<序号>，整轮在$HOST重跑" | tee -a "$log"
    fi
  fi
  for name in "${round_shards[@]}"; do
    if is_shard_done "$RUN_DIRECTORY/$name"; then reused=$((reused + 1)); continue; fi
    if ! break_is service-outcome-ignored; then
      if [[ -f "$RUN_DIRECTORY/$name.service-outcome" ]]; then
        echo "格 $name：有服务结局文件（X26），跳过" | tee -a "$log"
        skipped=$((skipped + 1))
        continue
      fi
    fi
    log_stem="$RUN_DIRECTORY/logs/$name-attempt-$attempt"
    rm -f -- "$IMAGE_DIRECTORY/e183-$name-process-"*
    echo "格 $name：起跑 时刻 $(date +%s) 负载 $(cat /proc/loadavg)" | tee -a "$log"
    arguments=(h2 shard "$name" --shard-output "$RUN_DIRECTORY/$name" --image-directory "$IMAGE_DIRECTORY" --binary-sha256 "$SHA256" --host "$HOST")
    break_is attempt-argument-dropped || arguments+=(--service-attempt "$TASK_SERVER_SHARD:$attempt")
    arguments+=(--image-filesystem "$filesystem")
    "$BINARY" "${arguments[@]}" > "$log_stem.out" 2> "$log_stem.err"
    exit_code=$?
    rm -f -- "$IMAGE_DIRECTORY/e183-$name-process-"*
    echo "格 $name：收尾 时刻 $(date +%s) 退 $exit_code 负载 $(cat /proc/loadavg)" | tee -a "$log"
    if ((exit_code == 3)); then
      echo "  ✗ 格 $name：装置报停机：$(grep -m1 '^E7RESULT name=stop ' -- "$log_stem.out")" >&2
      echo "     → 怎么办：该停下，把这一行 name=stop 交主 agent，不重交" >&2
      exit 3
    fi
    if ((exit_code != 0)) || ! is_shard_done "$RUN_DIRECTORY/$name"; then
      echo "  ✗ 格 $name：退 $exit_code，片文件没写成（$log_stem.err）" >&2
      echo "     → 怎么办：看这一格的错误输出；同一条命令重交只补没做完的" >&2
      exit $((exit_code == 0 ? 1 : exit_code))
    fi
    ran=$((ran + 1))
  done
  mkdir -p -- "$TASK_SERVER_SHARD_OUTPUT_DIRECTORY" || exit 1
  echo "第 $round 轮做完：现跑 $ran、沿用 $reused、跳过 $skipped、挪走 $moved（共 ${#round_shards[@]} 格）" | tee -a "$log"
}

# 合并（文件头「用法」merge 与 --merge-only）：$1 是运行目录。
merge_into_product() {
  # 程序文件名写成字面尾巴，理由同 shard_step 里给 BINARY 赋值那一处。
  local run_directory="$1" binary="$1/e183_capacity_read_cache_reserve_and_sweep_costs" sha256 temporary composed merge_exit destination header="$1/admission-header.txt"
  [[ -x "$binary" && -f "$run_directory/binary.sha256" ]] || { echo "  ✗ 运行目录里没有程序文件或 binary.sha256（$run_directory）" >&2; echo "     → 怎么办：先经服务交 run 让计划那一步编好" >&2; exit 78; }
  sha256=$(cat -- "$run_directory/binary.sha256")
  mkdir -p -- "$run_directory/logs" || exit 1
  temporary="$run_directory/logs/merge-$$.out"
  "$binary" h2 merge "$run_directory" --binary-sha256 "$sha256" > "$temporary" 2> "$run_directory/logs/merge-$$.err"
  merge_exit=$?
  if ((merge_exit == 3)); then
    echo "  ✗ 合并时装置报停机（$temporary）：$(grep -m1 '^E7RESULT name=stop ' -- "$temporary")" >&2
    echo "     → 怎么办：该停下，把这一行 name=stop 交主 agent，不重交" >&2
    exit 3
  fi
  is_complete_product "$temporary" || { echo "  ✗ 合并没出齐（$temporary，退 $merge_exit）" >&2; echo "     → 怎么办：看 $run_directory/logs/merge-$$.err；同一条命令重交补没做完的格再合并" >&2; exit 1; }
  composed="$run_directory/logs/product-$$.out"
  if break_is admission-header-dropped; then
    cp -- "$temporary" "$composed" || exit 1
  else
    [[ -s "$header" ]] || { echo "  ✗ 运行目录里没有准入那几行（$header）" >&2; echo "     → 怎么办：经服务重交 run，计划那一步会问准入并存下它" >&2; exit 2; }
    cat -- "$header" "$temporary" > "$composed" || exit 1
  fi
  mkdir -p -- "$PRODUCT_DIRECTORY" || exit 1
  destination=$(next_free_product_path)
  mv -n -- "$composed" "$destination"
  if [[ -e "$composed" ]]; then
    echo "  ✗ 产物名 $destination 被占了（合并出的留在 $composed）" >&2
    echo "     → 怎么办：看谁占了 $destination（别的会话同时在合并？），确认后手动把 $composed 挪成新名字" >&2
    exit 1
  fi
  echo "产物：$destination（$(wc -l < "$destination") 行，sha256 $(sha256sum -- "$destination" | cut -d' ' -f1)）"
}

# --merge-only 与 --list-items 用的运行目录：--run-directory 给的，没给取运行根下 latest-run-directory 记的。
named_or_latest_run_directory() {
  local root
  if [[ -n "$RUN_DIRECTORY_ARGUMENT" ]]; then
    echo "$RUN_DIRECTORY_ARGUMENT"
    return 0
  fi
  root=$(run_root) || exit 2
  [[ -f "$root/latest-run-directory" ]] || usage_error "运行根 $root 下没有 latest-run-directory：还没经服务跑过计划，或用 --run-directory 点名"
  cat -- "$root/latest-run-directory"
}

list_items() {
  local run_directory="$1" name round state
  [[ -x "$run_directory/$BINARY_NAME" ]] || { echo "  ✗ 运行目录里还没有程序文件：$run_directory" >&2; echo "     → 怎么办：先经服务交 run 让计划那一步编好" >&2; exit 78; }
  while IFS=$'\t' read -r name round; do
    if is_shard_done "$run_directory/$name"; then state=做完
    elif [[ -f "$run_directory/$name.service-outcome" ]]; then state=服务结局文件
    else state=没做; fi
    echo "第 $round 轮 $name $state"
  done < <("$run_directory/e183_capacity_read_cache_reserve_and_sweep_costs" h2 list-shards)
}

# 自证：假装置（每轮三格：大、中、小）、PATH 最前面放假编译与假 stat、假准入；照服务的分片协议逐步起本脚本的 run，每格在自己的目录里从头搭。
run_selftest() {
  local work failures=0 checked=0 fakes cell run exit_code first second nfs_exit broken_exit no_temporary_exit admission_77_exit admission_3_exit today scanned
  work=$(mktemp -d) || return 1
  fakes="$work/fakes"
  mkdir -p -- "$fakes"
  echo e183-cells-selftest > "$work/mark"
  cat > "$fakes/device" <<'FAKE'
#!/usr/bin/env bash
[[ -n "${E183_SELFTEST_LIST_FAILS:-}" ]] && exit 1
subcommand="$1 $2"
case "$subcommand" in
  "h2 list-shards")
    for round in 1 2 3 4 5; do printf 'h2-big-round-%s\t%s\nh2-mid-round-%s\t%s\nh2-small-round-%s\t%s\n' "$round" "$round" "$round" "$round" "$round" "$round"; done ;;
  "h2 shard")
    echo "$*" >> "$E183_SELFTEST_CALLS"
    shard="$3"; output="$5"; images="$7"; host=""
    shift 3
    while (($# > 0)); do if [[ "$1" == --host ]]; then host="$2"; shift 2; else shift; fi; done
    [[ -d "$images" ]] || exit 9
    printf 'E7RESULT name=h2_config shard=%s host=%s\nE7RESULT name=shard_done shard=%s lines=2\n' "$shard" "$host" "$shard" > "$output.partial-$$" && mv -- "$output.partial-$$" "$output" ;;
  "h2 merge")
    printf 'E7RESULT name=merged shards=%s\nE7RESULT name=done emitted=2\n' "$(find "$3" -maxdepth 1 -name 'h2-*-round-[0-9]' | wc -l)" ;;
  *) exit 2 ;;
esac
FAKE
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*" >> "$E183_SELFTEST_BUILDS"' \
    'mkdir -p -- "$CARGO_TARGET_DIR/release" && cp -- "$E183_SELFTEST_FAKE_DEVICE" "$CARGO_TARGET_DIR/release/e183_capacity_read_cache_reserve_and_sweep_costs"' > "$fakes/cargo"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "${E183_SELFTEST_FILESYSTEM_TYPE:-ext2/ext3}"' > "$fakes/stat"
  printf '%s\n' 'import os, sys' 'exit_code = int(os.environ.get("E183_SELFTEST_ADMISSION_EXIT", "0"))' \
    'print("E7INPUT name=input_fingerprint key=%s sha256=%s units=2" % (sys.argv[3], "0123456789abcdef" * 4)) if exit_code == 0 else print("假准入退 %d" % exit_code, file=sys.stderr)' \
    'sys.exit(exit_code)' > "$fakes/admission.py"
  chmod +x "$fakes/device" "$fakes/cargo" "$fakes/stat"
  # 照服务的分片协议起 run 的一步：<格目录> <步> [<变量>=<值>…]；服务给的变量先从外面清掉（自证经服务起时环境里就有），再只设这一格点名的。
  phase() {
    local cell="$1" step="$2"
    shift 2
    mkdir -p -- "$cell/plan" "$cell/tmp"
    env -u MULTI_HOST_RUN_PEER_SCRATCH -u CARGO_TARGET_DIR -u TASK_SERVER_SHARD -u TASK_SERVER_ATTEMPT -u TASK_SERVER_SHARD_OUTPUT_DIRECTORY \
      PATH="$fakes:$PATH" E183_CELLS_SELFTEST_MARK="$work/mark" E183_RUN_ROOT="$cell/run-root" E183_PRODUCT_DIRECTORY="$cell/product" \
      E183_ADMISSION_SCRIPT="$fakes/admission.py" E183_SELFTEST_FAKE_DEVICE="$fakes/device" E183_SELFTEST_CALLS="$cell/calls" \
      E183_SELFTEST_BUILDS="$cell/builds" TASK_SERVER_SHARD_PHASE="$step" TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" TMPDIR="$cell/tmp" \
      "$@" bash "$SCRIPT_PATH" run >> "$cell/out" 2>&1
  }
  # 片那一步：<格目录> <片号> <第几次> [<变量>=<值>…]
  shard_phase() {
    local cell="$1" shard_number="$2" attempt="$3"
    shift 3
    phase "$cell" shard TASK_SERVER_SHARD="$shard_number/5" TASK_SERVER_ATTEMPT="$attempt" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$cell/output-$shard_number" "$@"
  }
  called_shards() { cut -d' ' -f3 "$1/calls" 2> /dev/null | tr '\n' ' '; }
  expect() {
    local name="$1" condition="$2"
    checked=$((checked + 1))
    if eval "$condition"; then echo "  ✓ $name"; else echo "  ✗ $name：条件「$condition」不成立"; echo "     → 怎么办：按这一格的名字找对应的计划、片、合并或静态核的逻辑改，或看 E183_CELLS_BREAK 是不是设着"; failures=$((failures + 1)); fi
  }
  run=0123456789abcdef
  today=$(TZ=Asia/Tokyo date +%F)
  cell="$work/plan"; phase "$cell" plan; exit_code=$?
  expect "计划：准入退 0 时写 5 片的 plan.json、运行目录按键起名、准入那几行落进运行目录" '((exit_code == 0)) && python3 -c "import json, sys; plan = json.load(open(sys.argv[1])); sys.exit(0 if [shard[\"label\"] for shard in plan[\"shards\"]] == [\"round-%d\" % number for number in range(1, 6)] else 1)" "$cell/plan/plan.json" && [[ "$(cat "$cell/plan/run-directory")" == "$cell/run-root/$run" ]] && grep -q "^E7INPUT name=input_fingerprint key=E183/h2 " "$cell/run-root/$run/admission-header.txt" && [[ -x "$cell/run-root/$run/e183_capacity_read_cache_reserve_and_sweep_costs" && ! -e "$cell/run-root/$run/target" ]]'
  cell="$work/admission-77"; phase "$cell" plan E183_SELFTEST_ADMISSION_EXIT=77; admission_77_exit=$?
  phase "$work/admission-3" plan E183_SELFTEST_ADMISSION_EXIT=3; admission_3_exit=$?
  expect "计划：准入退 77 时整步退 77、不编译；退 3 时退 78" '((admission_77_exit == 77 && admission_3_exit == 78)) && [[ ! -e "$cell/builds" && ! -e "$work/admission-3/builds" ]]'
  cell="$work/plan-twice"; phase "$cell" plan; first=$?; phase "$cell" plan; second=$?
  expect "计划：运行目录里已有程序文件就不再编" '((first == 0 && second == 0)) && (($(wc -l < "$cell/builds") == 1))'
  cell="$work/order"; phase "$cell" plan; shard_phase "$cell" 0 1; first=$?; shard_phase "$cell" 0 2; second=$?
  expect "片：一轮之内大的先起；做完的沿用" '((first == 0 && second == 0)) && [[ "$(called_shards "$cell")" == "h2-big-round-1 h2-mid-round-1 h2-small-round-1 " && -d "$cell/output-0" ]]'
  cell="$work/skips"; phase "$cell" plan
  echo "type=memory-cap-at-maximum reached=52613349376 peak=none task=1 shard=1/5 attempt=4" > "$cell/run-root/$run/h2-big-round-2.service-outcome"
  shard_phase "$cell" 1 1; exit_code=$?
  expect "片：有 .service-outcome 的格跳过" '((exit_code == 0)) && [[ "$(called_shards "$cell")" == "h2-mid-round-2 h2-small-round-2 " ]] && grep -q "有服务结局文件" "$cell/out"'
  cell="$work/other-host"; phase "$cell" plan; mkdir -p -- "$cell/peer"
  shard_phase "$cell" 2 1 MULTI_HOST_RUN_PEER_SCRATCH="$cell/peer"; first=$?; shard_phase "$cell" 2 2; second=$?
  expect "片：这一轮有别的台做完的格，挪成 .other-host-<序号>、整轮重跑" '((first == 0 && second == 0)) && (($(wc -l < "$cell/calls") == 6)) && grep -q " host=第二台" "$cell/run-root/$run/h2-big-round-3.other-host-1" && grep -q " host=本机" "$cell/run-root/$run/h2-big-round-3"'
  cell="$work/nfs"; phase "$cell" plan; shard_phase "$cell" 0 1 E183_SELFTEST_FILESYSTEM_TYPE=nfs; nfs_exit=$?
  shard_phase "$cell" 0 2 E183_SELFTEST_LIST_FAILS=1; broken_exit=$?
  shard_phase "$cell" 0 3 TMPDIR=; no_temporary_exit=$?
  expect "片：镜像目录是 nfs 退 78；本机没有 TMPDIR 退 78；程序文件起不来退 78" '((nfs_exit == 78 && broken_exit == 78 && no_temporary_exit == 78)) && [[ ! -e "$cell/calls" ]]'
  cell="$work/arguments"; phase "$cell" plan; shard_phase "$cell" 0 4; exit_code=$?
  expect "片：交给装置的参数里有 --host、--service-attempt、--image-filesystem 且值对，每格前后各记一次负载" '((exit_code == 0)) && (($(grep -c "：起跑 时刻 [0-9]* 负载 " "$cell/out") == 3 && $(grep -c "：收尾 时刻 [0-9]* 退 0 负载 " "$cell/out") == 3)) && grep -qF -- "--image-directory $cell/tmp/e183-images --binary-sha256 " "$cell/calls" && (($(grep -cF -- "--host 本机 --service-attempt 0/5:4 --image-filesystem ext2/ext3" "$cell/calls") == 3)) && [[ ! -e "$cell/tmp/e183-images" ]]'
  cell="$work/merge"; phase "$cell" plan; shard_phase "$cell" 0 1
  mkdir -p -- "$cell/product"; echo 占位 > "$cell/product/e183-capacity-read-cache-reserve-and-sweep-costs-h2-$today.out"
  phase "$cell" merge; exit_code=$?
  expect "合并：产物最前面是准入那几行、不覆盖已有的产物名" '((exit_code == 0)) && [[ "$(cat "$cell/product/e183-capacity-read-cache-reserve-and-sweep-costs-h2-$today.out")" == 占位 ]] && head -n 1 "$cell/product/e183-capacity-read-cache-reserve-and-sweep-costs-h2-$today-r2.out" | grep -q "^E7INPUT name=input_fingerprint key=E183/h2 " && grep -qx "E7RESULT name=merged shards=3" "$cell/product/e183-capacity-read-cache-reserve-and-sweep-costs-h2-$today-r2.out"'
  cell="$work/unsharded"; mkdir -p -- "$cell"
  env -u TASK_SERVER_SHARD_PHASE PATH="$fakes:$PATH" E183_CELLS_SELFTEST_MARK="$work/mark" E183_RUN_ROOT="$cell/run-root" E183_ADMISSION_SCRIPT="$fakes/admission.py" \
    bash "$SCRIPT_PATH" run > "$cell/out" 2>&1; exit_code=$?
  expect "不经服务起 run 退 2" '((exit_code == 2)) && grep -q "只经统一任务服务起" "$cell/out" && [[ ! -e "$cell/run-root" ]]'
  scanned="$SCRIPT_PATH"
  if break_is wrapper-reintroduced; then
    scanned="$work/scanned.sh"
    cp -- "$SCRIPT_PATH" "$scanned" && printf '%s%s\n' 'ni' 'ce -n 19 true' >> "$scanned"
  fi
  expect "脚本正文里命令位置上没有包装（静态核）" '(($(grep -cE "$WRAPPER_PATTERN" "$scanned") == 0))'
  rm -rf -- "${work:?}"
  if ((failures > 0)); then echo "e183-capacity-cells.sh 自证：$checked 格里 $failures 格红"; return 1; fi
  echo "e183-capacity-cells.sh 自证：$checked 格全绿"
  return 0
}

mode=""
RUN_DIRECTORY_ARGUMENT=""
while (($# > 0)); do
  case "$1" in
    run) mode=run; shift ;;
    --merge-only) mode=merge; shift ;;
    --list-items) mode=list; shift ;;
    --selftest) mode=selftest; shift ;;
    --run-directory) (($# >= 2)) || usage_error "--run-directory 后面要写目录"; RUN_DIRECTORY_ARGUMENT="$2"; shift 2 ;;
    *) usage_error "认不得的参数「$1」" ;;
  esac
done
case "$mode" in
  selftest)
    run_selftest
    exit $?
    ;;
  list)
    chosen_run_directory=$(named_or_latest_run_directory) || exit 2
    list_items "$chosen_run_directory"
    exit 0
    ;;
  merge)
    chosen_run_directory=$(named_or_latest_run_directory) || exit 2
    merge_into_product "$chosen_run_directory"
    exit 0
    ;;
  run)
    case "${TASK_SERVER_SHARD_PHASE:-}" in
      plan) plan_step ;;
      shard) shard_step ;;
      merge)
        chosen_run_directory=$(run_directory_of_the_plan) || exit 2
        merge_into_product "$chosen_run_directory"
        ;;
      "")
        if break_is phase-unchecked; then
          plan_step
        else
          echo "  ✗ run 只经统一任务服务起（环境里没有 TASK_SERVER_SHARD_PHASE）；一步都没做" >&2
          echo "     → 怎么办：task-server/task submit -- bash scripts/experiment-apparatus/e183-capacity-cells.sh run（服务照文件头的分片声明拆成计划、各片、合并）" >&2
          exit 2
        fi
        ;;
      *) usage_error "认不得的 TASK_SERVER_SHARD_PHASE「${TASK_SERVER_SHARD_PHASE}」（只认 plan、shard、merge）" ;;
    esac
    exit 0
    ;;
  "") usage_error "要写 run、--merge-only、--list-items 或 --selftest" ;;
esac
