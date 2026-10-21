#!/usr/bin/env bash
# admission: always 每一次调都照此刻片目录里哪几片做完了判还差什么，上一次跑完不替这一次作保
# run-condition: command cargo git sha256sum
# E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数）第二段（问题单 H2，跑前登记 research/prompts/e183-preregistration.md 5.5「第二段」）的跑法：
#   装置是入库装置 crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs（bin 同名）。一片 = 一格一轮 = 装置的一个进程
#   h2 shard <片名> --shard-output <片目录>/<片名> --image-directory <镜像目录> --binary-sha256 <…>（登记 5.2「每格 5 个进程」）；45 片按装置 h2 list-shards 的次序
#   一片一片地跑（挂钟量，同时只跑一片；登记 5.5 第二段在 2 小时内、不分片、单机跑），片齐了由装置 h2 merge 接成产物
#   research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-<日期>.out 并算判定（装置单测 capacity_summary_judges_every_row_of_answer_known_synthetic_readings 钉判定那一半）。
#   每片经 run-with-memory-cap.sh 起，上限 24G（派发给的）；片没做完时把包装的退出码写进 <片目录>/<片名>.wrapper-exit。包装退 250（撞了这一片自己的上限）是定局：
#   合并时记「内存上限之外」（登记 X3）、不调高上限、不重跑。包装退 252（排不上）、254（被总上限挤掉）不是这一片撞了自己的上限，命令没跑完：本脚本在同一次调用里
#   重跑那一片，直到它退 250 或跑成，每片至多重跑 E183_CELL_RETRIES 次（默认 30；252 在「这一条要的量比总上限还大」时是立刻退的，不设上限会空转）；
#   到了次数仍是 252 或 254 的，exit 文件留着、记进没做完的片、退 1，重起同一条命令继续；合并只把 250 读成「内存上限之外」，251–254 读成这一片没跑成（登记第十二节修订）。
#   起跑前把 uptime 与此刻在跑的 cargo / 实验装置进程数写进日志（耗时是挂钟量，本机空闲时起）。
#
# 用法（在仓里任何目录起都行，脚本自己回到仓根）：
#   bash scripts/experiments/e183-capacity-cells.sh                 跑没做完的片（做完的沿用），片齐了合并出产物
#   bash scripts/experiments/e183-capacity-cells.sh --merge-only    不跑，合并出产物（没做完的片按结局记）
#   bash scripts/experiments/e183-capacity-cells.sh --list-items    逐行列片名与做没做完（要装置已编好，不编译）
#   bash scripts/experiments/e183-capacity-cells.sh --selftest      假装置走一遍续跑、内存上限的退出码与合并（不起 cargo）
#   --shard-directory <目录> 可与前三种合用，换片目录。
# 环境：E183_BINARY（装置程序文件；没给就先 cargo build --release 编仓根 target/ 下的那一份）、E183_CELL_DIRECTORY（片目录，默认
#   ~/.local/share/singlefs/work/e183-h2-cells）、E183_IMAGE_DIRECTORY（镜像目录，默认 ~/.local/share/singlefs/work/e183-h2-images，要在本机 ext4 上）、
#   E183_CELL_MEMORY_CAP（每片的内存包装上限，默认 24G）、E183_CELL_RETRIES（包装退 252 或 254 时每片至多重跑几次，默认 30）、E183_THREADS（编译用的线程，没设取登记的每台默认重活线程数）、
#   E183_LOG_DIRECTORY（默认 ~/.local/share/singlefs/work/e183-h2-cells-logs）、E183_PRODUCT_DIRECTORY（默认 research/results）、
#   只给自证换假的：E183_MEMORY_WRAPPER（默认 scripts/hosts/run-with-memory-cap.sh）。
# 退出码：0 产物出了；1 有片没做完（重起同一条命令续跑）或合并没出齐；2 用法错；3 装置报停机（name=stop 那一行交主 agent）；78 装置没编好（--list-items）。
#
# 弄坏开关 E183_CELLS_BREAK=<项>（只在 --selftest 起的那几趟里认）：
#   wrapper-exit-dropped   片没做完时不写 .wrapper-exit          打开后自证「内存上限的退出码写下来」红
#   retry-dropped          包装退 252、254 时不重跑，当场记没做完   打开后自证「排不上的片重跑到做完」「重跑到次数上限仍排不上」红
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../hosts/heavy-threads.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; exit 2; }

BINARY_DEFAULT="target/release/e183_capacity_read_cache_reserve_and_sweep_costs"
PRODUCT_STEM_NAME="e183-capacity-read-cache-reserve-and-sweep-costs-h2"
MEMORY_CAP="${E183_CELL_MEMORY_CAP:-24G}"
CELL_RETRIES="${E183_CELL_RETRIES:-30}"
MERGE_MEMORY_CAP="8G"
IMAGE_DIRECTORY="${E183_IMAGE_DIRECTORY:-$HOME/.local/share/singlefs/work/e183-h2-images}"
LOG_DIRECTORY="${E183_LOG_DIRECTORY:-$HOME/.local/share/singlefs/work/e183-h2-cells-logs}"
PRODUCT_DIRECTORY="${E183_PRODUCT_DIRECTORY:-research/results}"
MEMORY_WRAPPER="${E183_MEMORY_WRAPPER:-scripts/hosts/run-with-memory-cap.sh}"

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
    exit 2
  fi
fi

# 一片做完了：文件在，末行是 E7RESULT name=shard_done shard=<片名> lines=<整份行数>。
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

prepare_binary() {
  local threads
  if [[ -n "${E183_BINARY:-}" ]]; then
    BINARY="$E183_BINARY"
  else
    BINARY="$BINARY_DEFAULT"
    threads="${E183_THREADS:-$(heavy_thread_default)}"
    bash scripts/hosts/capped.sh "$threads" cargo build --release -p singlefs-checker-tier --bin e183_capacity_read_cache_reserve_and_sweep_costs \
      > "$LOG_DIRECTORY/build-$RUN_STAMP.log" 2>&1 || {
      echo "  ✗ 装置编不过（$LOG_DIRECTORY/build-$RUN_STAMP.log）" >&2
      exit 1
    }
  fi
  BINARY_SHA256=$(sha256sum -- "$BINARY" | cut -d' ' -f1)
  [[ "$BINARY_SHA256" =~ ^[0-9a-f]{64}$ ]] || { echo "  ✗ 算不出装置程序文件的 sha256（$BINARY）" >&2; exit 1; }
}

# 包装退 252（内存不够排不上）或 254（被总上限挤掉）：命令没跑完，也不是这一片撞了自己的上限，该重跑。
is_a_wrapper_exit_to_rerun() {
  [[ "$1" == 252 || "$1" == 254 ]]
}

# 一片一片地跑没做完的片：起之前删掉这一片上一次被杀留下的镜像（文件名 e183-<片名>-process-*），跑完没做完就写下包装的退出码；
# 包装退 252、254 的重跑，至多 CELL_RETRIES 次。
run_cells() {
  local line name exit_code attempt log_stem
  RAN_NOW=0; REUSED=0; SETTLED_AT_THE_LIMIT=0; STOPPED=(); FAILED=()
  for line in "${SHARD_LINES[@]}"; do
    name="${line%%$'\t'*}"
    if is_shard_done "$CELL_DIRECTORY/$name"; then REUSED=$((REUSED + 1)); continue; fi
    # 撞过这一片自己的上限（250）的不再起：登记 X3 记「内存上限之外」、不调高；252、254 不是这一片的事，照起。
    if [[ "$(cat -- "$CELL_DIRECTORY/$name.wrapper-exit" 2> /dev/null)" == 250 ]]; then SETTLED_AT_THE_LIMIT=$((SETTLED_AT_THE_LIMIT + 1)); continue; fi
    rm -f -- "$CELL_DIRECTORY/$name.wrapper-exit"
    attempt=0
    while :; do
      attempt=$((attempt + 1))
      log_stem="$LOG_DIRECTORY/$name-$RUN_STAMP-attempt-$attempt"
      rm -f -- "$IMAGE_DIRECTORY/e183-$name-process-"*
      echo "片 $name：起跑（第 $attempt 次）"
      bash "$MEMORY_WRAPPER" "$MEMORY_CAP" nice -n 19 "$BINARY" h2 shard "$name" --shard-output "$CELL_DIRECTORY/$name" \
        --image-directory "$IMAGE_DIRECTORY" --binary-sha256 "$BINARY_SHA256" > "$log_stem.out" 2> "$log_stem.err"
      exit_code=$?
      rm -f -- "$IMAGE_DIRECTORY/e183-$name-process-"*
      if is_a_wrapper_exit_to_rerun "$exit_code" && ((attempt <= CELL_RETRIES)) && ! break_is retry-dropped; then
        echo "  … 片 $name：包装退 $exit_code（不是这一片撞了自己的上限，命令没跑完），重跑（已跑 $attempt 次，至多重跑 $CELL_RETRIES 次）" >&2
        continue
      fi
      break
    done
    if ((exit_code == 0)) && is_shard_done "$CELL_DIRECTORY/$name"; then
      RAN_NOW=$((RAN_NOW + 1))
    elif ((exit_code == 3)); then
      STOPPED+=("$name")
      echo "  ✗ 片 $name：装置报停机：$(grep -m1 '^E7RESULT name=stop ' "$log_stem.out")" >&2
    else
      break_is wrapper-exit-dropped || echo "$exit_code" > "$CELL_DIRECTORY/$name.wrapper-exit"
      if ((exit_code == 250)); then SETTLED_AT_THE_LIMIT=$((SETTLED_AT_THE_LIMIT + 1)); else FAILED+=("$name"); fi
      echo "  ✗ 片 $name：退 $exit_code，没做完（250–254 是内存包装的结局：250 撞了这一片的上限，记「内存上限之外」；252、254 已重跑 $((attempt - 1)) 次仍是这个结局，251、253 不重跑）" >&2
    fi
  done
}

merge_product() {
  local temporary="$LOG_DIRECTORY/merge-$RUN_STAMP.out" destination merge_exit
  bash "$MEMORY_WRAPPER" "$MERGE_MEMORY_CAP" "$BINARY" h2 merge "$CELL_DIRECTORY" --binary-sha256 "$BINARY_SHA256" \
    > "$temporary" 2> "$LOG_DIRECTORY/merge-$RUN_STAMP.err"
  merge_exit=$?
  if ((merge_exit == 3)); then
    echo "  ✗ 合并时装置报停机（$temporary）：$(grep -m1 '^E7RESULT name=stop ' "$temporary")" >&2
    exit 3
  fi
  is_complete_product "$temporary" || { echo "  ✗ 合并没出齐（$temporary，退 $merge_exit）" >&2; exit 1; }
  mkdir -p -- "$PRODUCT_DIRECTORY" || exit 1
  destination=$(next_free_product_path)
  mv -n -- "$temporary" "$destination"
  [[ -e "$temporary" ]] && { echo "  ✗ 产物名 $destination 被占了（合并出的留在 $temporary）" >&2; exit 1; }
  echo "产物：$destination（$(wc -l < "$destination") 行，sha256 $(sha256sum -- "$destination" | cut -d' ' -f1)）"
}

# 自证：假装置（三片，第二片按开关退 250）、假内存包装（直接起命令、照装置的退出码退），每一格在自己的临时目录里从头搭。
run_selftest() {
  local work failures=0 fakes cell exit_code
  work=$(mktemp -d) || return 1
  fakes="$work/fakes"
  mkdir -p -- "$fakes"
  echo e183-cells-selftest > "$work/mark"
  cat > "$fakes/device" <<'FAKE'
#!/usr/bin/env bash
case "$1 $2" in
  "h2 list-shards") printf '%s\t%s\n' h2-a 1 h2-b 1 h2-c 1 ;;
  "h2 shard")
    echo "$3" >> "$E183_SELFTEST_CALLS"
    [[ "$3" == "${E183_SELFTEST_CAPPED_SHARD:-}" ]] && exit 250
    if [[ "$3" == "${E183_SELFTEST_FLAKY_SHARD:-}" ]] && (($(grep -c -x -- "$3" "$E183_SELFTEST_CALLS") <= ${E183_SELFTEST_FLAKY_TIMES:-0})); then
      exit "${E183_SELFTEST_FLAKY_EXIT:-252}"
    fi
    printf 'E7RESULT name=h2_config shard=%s\nE7RESULT name=shard_done shard=%s lines=2\n' "$3" "$3" > "$5.partial-$$" && mv -- "$5.partial-$$" "$5" ;;
  "h2 merge")
    printf 'E7RESULT name=merged wrapper_exits=%s\nE7RESULT name=done emitted=2\n' "$(cat "$3"/*.wrapper-exit 2> /dev/null | tr '\n' ',')" ;;
esac
FAKE
  printf '%s\n' '#!/usr/bin/env bash' 'shift' 'exec "$@"' > "$fakes/wrapper"
  chmod +x "$fakes"/*
  invoke() {
    local cell="$1"
    shift
    E183_BINARY="$fakes/device" E183_MEMORY_WRAPPER="$fakes/wrapper" E183_SELFTEST_CALLS="$cell/calls" E183_CELLS_SELFTEST_MARK="$work/mark" \
      E183_CELL_DIRECTORY="$cell/cells" E183_IMAGE_DIRECTORY="$cell/images" E183_LOG_DIRECTORY="$cell/logs" E183_PRODUCT_DIRECTORY="$cell/product" \
      bash "$SCRIPT_PATH" "$@" > "$cell/out" 2>&1
  }
  expect() {
    local name="$1" condition="$2"
    if eval "$condition"; then echo "  ✓ $name"; else echo "  ✗ $name：条件「$condition」不成立"; failures=$((failures + 1)); fi
  }
  cell="$work/fresh"; mkdir -p -- "$cell"
  invoke "$cell"; exit_code=$?
  expect "全新：三片都现跑、合并出产物" '((exit_code == 0)) && (($(wc -l < "$cell/calls") == 3)) && (($(find "$cell/product" -name "e183-capacity-read-cache-reserve-and-sweep-costs-h2-*.out" | wc -l) == 1))'
  invoke "$cell"; exit_code=$?
  expect "原样重起：三片都沿用、一片都不再起" '((exit_code == 0)) && (($(wc -l < "$cell/calls") == 3)) && grep -q "现跑 0、沿用 3" "$cell/out"'
  cell="$work/capped"; mkdir -p -- "$cell"
  E183_SELFTEST_CAPPED_SHARD=h2-b invoke "$cell"; exit_code=$?
  expect "内存上限的退出码写下来：那一片记 250、别的片做完、照样合并" '((exit_code == 0)) && [[ "$(cat "$cell/cells/h2-b.wrapper-exit" 2> /dev/null)" == 250 ]] && grep -q "wrapper_exits=250," "$cell/product"/*.out'
  cell="$work/rerun-252"; mkdir -p -- "$cell"
  E183_SELFTEST_FLAKY_SHARD=h2-b E183_SELFTEST_FLAKY_EXIT=252 E183_SELFTEST_FLAKY_TIMES=2 invoke "$cell"; exit_code=$?
  expect "排不上（252）的片重跑到做完：h2-b 起三次、不留 .wrapper-exit、照样合并" '((exit_code == 0)) && (($(grep -c -x h2-b "$cell/calls") == 3)) && (($(wc -l < "$cell/calls") == 5)) && [[ ! -e "$cell/cells/h2-b.wrapper-exit" ]] && grep -qx "E7RESULT name=merged wrapper_exits=" "$cell/product"/*.out'
  cell="$work/rerun-254"; mkdir -p -- "$cell"
  E183_SELFTEST_FLAKY_SHARD=h2-c E183_SELFTEST_FLAKY_EXIT=254 E183_SELFTEST_FLAKY_TIMES=1 invoke "$cell"; exit_code=$?
  expect "被总上限挤掉（254）的片同样重跑到做完：h2-c 起两次、照样合并" '((exit_code == 0)) && (($(grep -c -x h2-c "$cell/calls") == 2)) && [[ ! -e "$cell/cells/h2-c.wrapper-exit" ]]'
  cell="$work/rerun-limit"; mkdir -p -- "$cell"
  E183_CELL_RETRIES=2 E183_SELFTEST_FLAKY_SHARD=h2-b E183_SELFTEST_FLAKY_EXIT=252 E183_SELFTEST_FLAKY_TIMES=99 invoke "$cell"; exit_code=$?
  expect "重跑到次数上限仍排不上：h2-b 起三次（一次加至多重跑两次）、记 252、退 1、不合并；别的片照做" '((exit_code == 1)) && (($(grep -c -x h2-b "$cell/calls") == 3)) && [[ "$(cat "$cell/cells/h2-b.wrapper-exit" 2> /dev/null)" == 252 ]] && [[ ! -d "$cell/product" ]] && [[ -f "$cell/cells/h2-c" ]]'
  invoke "$cell"; exit_code=$?
  expect "上限用完之后同一条命令重起：只补 h2-b、合并出产物" '((exit_code == 0)) && grep -q "现跑 1、沿用 2" "$cell/out" && [[ ! -e "$cell/cells/h2-b.wrapper-exit" ]]'
  cell="$work/no-rerun-251"; mkdir -p -- "$cell"
  E183_SELFTEST_FLAKY_SHARD=h2-b E183_SELFTEST_FLAKY_EXIT=251 E183_SELFTEST_FLAKY_TIMES=99 invoke "$cell"; exit_code=$?
  expect "用法错（251）不重跑：h2-b 只起一次、记 251、退 1" '((exit_code == 1)) && (($(grep -c -x h2-b "$cell/calls") == 1)) && [[ "$(cat "$cell/cells/h2-b.wrapper-exit" 2> /dev/null)" == 251 ]]'
  cell="$work/usage"; mkdir -p -- "$cell"
  invoke "$cell" --no-such-switch; exit_code=$?
  expect "认不得的参数：用法错退 2" '((exit_code == 2))'
  rm -rf -- "$work"
  if ((failures > 0)); then echo "e183-capacity-cells.sh 自证：$failures 格红"; return 1; fi
  echo "e183-capacity-cells.sh 自证：9 格全绿"
  return 0
}

mode=run
CELL_DIRECTORY="${E183_CELL_DIRECTORY:-$HOME/.local/share/singlefs/work/e183-h2-cells}"
while (($# > 0)); do
  case "$1" in
    --merge-only) mode=merge; shift ;;
    --list-items) mode=list; shift ;;
    --selftest) mode=selftest; shift ;;
    --shard-directory) (($# >= 2)) || usage_error "--shard-directory 后面要写目录"; CELL_DIRECTORY="$2"; shift 2 ;;
    *) usage_error "认不得的参数「$1」" ;;
  esac
done
if [[ "$mode" == selftest ]]; then
  run_selftest
  exit $?
fi
RUN_STAMP="$(TZ=Asia/Tokyo date +%F)-$$"
mkdir -p -- "$LOG_DIRECTORY" "$CELL_DIRECTORY" "$IMAGE_DIRECTORY" || exit 1
if [[ "$mode" == list ]]; then
  BINARY="${E183_BINARY:-$BINARY_DEFAULT}"
  [[ -x "$BINARY" ]] || { echo "  ✗ 装置还没编好：$BINARY" >&2; exit 78; }
  mapfile -t SHARD_LINES < <(bash "$MEMORY_WRAPPER" 1G "$BINARY" h2 list-shards)
  for line in "${SHARD_LINES[@]}"; do
    name="${line%%$'\t'*}"
    if is_shard_done "$CELL_DIRECTORY/$name"; then echo "$name 做完"; else echo "$name 没做"; fi
  done
  exit 0
fi
prepare_binary
mapfile -t SHARD_LINES < <(bash "$MEMORY_WRAPPER" 1G "$BINARY" h2 list-shards)
((${#SHARD_LINES[@]} > 0)) || { echo "  ✗ 装置 h2 list-shards 什么都没列（$BINARY）" >&2; exit 1; }
{
  echo "起跑时的负载：$(uptime)"
  echo "此刻在跑的 cargo、rustc 与实验装置进程数：$(ps -o args= -u "$(id -u)" | grep -cE '(^|/)(cargo|rustc|e1[0-9]{2}_[a-z_]+)( |$)')"
} | tee "$LOG_DIRECTORY/load-$RUN_STAMP.txt"
echo "e183 第二段：共 ${#SHARD_LINES[@]} 片、每片内存上限 $MEMORY_CAP、片目录 $CELL_DIRECTORY、镜像目录 $IMAGE_DIRECTORY、装置 sha256 $BINARY_SHA256"
if [[ "$mode" == run ]]; then
  run_cells
  ((${#STOPPED[@]} == 0)) || { echo "  ✗ ${#STOPPED[@]} 片装置报停机（${STOPPED[*]}），交主 agent，不重跑" >&2; exit 3; }
  ((${#FAILED[@]} == 0)) || { echo "  ✗ ${#FAILED[@]} 片没做完（${FAILED[*]}）；同一条命令重起只补没做完的" >&2; exit 1; }
  echo "片都有了结局：现跑 $RAN_NOW、沿用 $REUSED、撞了自己的内存上限 $SETTLED_AT_THE_LIMIT（共 ${#SHARD_LINES[@]} 片）"
fi
merge_product
exit 0
