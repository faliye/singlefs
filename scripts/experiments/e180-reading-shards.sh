#!/usr/bin/env bash
# admission: always 每一次调都照此刻片目录里哪几片做完了判还差什么，上一次跑完不替这一次作保
# run-condition: command cargo git python3
# E180（准入读数与真可写量）第一段（读数段，跑前登记 research/prompts/e180-preregistration.md 5.5 节与第十二节修订 4）的分片跑法：
#   装置是入库装置 crates/singlefs-checker-tier/src/bin/e180_admission_reading_versus_writable_bytes.rs（bin 同名）。一片 = 装置的一次
#   reading --geometry <几何名> --family <族名> --shard-output <片目录>/<片名>；片齐了由装置 merge <片目录> 合成产物
#   research/results/e180-admission-reading-versus-writable-bytes-reading.out。合并与一次 reading 跑全部共用装置里拼产物的那一个函数
#   （装置单测 merging_the_shards_of_every_cell_gives_the_product_of_running_every_cell_at_once_byte_for_byte）；族汇总行的 seconds 是各片自己的耗时。
#
# long-run-shards: 一片 = 装置一次 reading --geometry <几何名> --family <族名> --shard-output <片文件>：5 个几何 × 5 个带探测的族，加 unequal_devices_geometry 之外 4 个几何的 hl，共 29 片（装置 list-shards 逐行列片名、几何、族与这一片占的线程数）；片之间互不依赖，装置 merge 按片的固定次序（几何 × 族）合成产物
# long-run-resume: 同一条命令重起就是续跑：装置把一片先写进 <片>.partial-<进程号>、刷盘再换名成 <片>（.shard 结尾），末行是 shard_done<TAB><整份行数>；重起时末行齐、且片旁 <片>.binary-sha256 记的装置 sha256 与这一趟的相同的片沿用、只补没做完的（记的不同或没有记录的片整份移到 <日志目录>/void-<日期>-<进程号>/ 再现跑，逐片报「作废：<片名>（装置 sha256 不同）」），成功那句报「现跑 N、沿用 M」
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
#   多机的读法：台数 = 1 + --emit-hosts 打的 MULTI_HOST_COUNT（双机关着、它退 3 时只有本机）；片按 list-shards 的次序轮流分给各台（第 k 片给第 k mod 台数 台，本机是第 0 台）。
#   第二台那一份由本脚本经 scripts/hosts/multi-host-run.sh --peer 起（在第二台的树里跑 本脚本 --host-share 1/2，--send 把本机片目录带过去好让做完的片沿用，
#   --fetch 取回它的片目录），取回后把做完的片并进本机片目录；台数超过 2 时 multi-host-run.sh 只到得了 PEER 那一台，退 78。
#   每台的线程是 scripts/hosts/heavy-threads.sh 的 heavy_thread_default（E180_THREADS 设了用它）；同一个数当装置的 E180_THREADS 传给每一片（探测线程数由它定，
#   装置合并时核每片的这个数相同），也是这台上同时在跑的片占的线程合计的上限（每片占几个照 list-shards 的第四列）。
#   装置在一趟里不许换，每片记下打它的是哪一版装置（欠账 C862，共用段落在 long-run-lib/binary-identity.sh）：开跑时把装置程序文件拷一份进 $LOG_DIRECTORY/pinned-binary/<sha256>/（只读），
#   列片、起每一片、合并都按这份拷贝，不再 exec 仓根 target/release 下那一份（别的会话的 cargo 随时会把它换掉）；每片做完在片旁写 <片>.binary-sha256（这份拷贝的 sha256），
#   续跑只沿用记的 sha256 相同的片，合并之前再核一遍全部片的记录相同（不同就拒、退 1、装置 merge 不起）。老的拷贝留着不删（一个 sha256 一个目录），确认不再续跑那一版之后手删。
#   第二台不自己编装置：两台各编一份，程序文件里编进了编译机的绝对路径，树在哪不同 sha256 就不同，第二台的片就会被本机当成别的版本作废。所以本机那一份拷贝经 multi-host-run.sh 的第二个 --send 送进第二台的树，
#   第二台的命令带 --binary <送到的路径> --shipped-binary-sha256 <本机算的 sha256>：第二台先核送到的文件 sha256 与本机的相同，不同就退 1、一片不跑；第二台跑装置前在 PATH 最前面放一个 python3 垫片
#   （写在 $LOG_DIRECTORY 下），装置起 python3 判准入时把编译机的 preflight.py 与装置源文件路径换成第二台这棵树里的同名文件（装置不起 python3 判准入时垫片不被用到）。
#
# 用法（在仓里任何目录起都行，脚本自己回到仓根）：
#   bash scripts/experiments/e180-reading-shards.sh                         本机跑本机那一份、起第二台那一份，片齐了合并出产物
#   bash scripts/experiments/e180-reading-shards.sh --local-only            全部片在本机跑（不读多机配置），片齐了合并出产物
#   bash scripts/experiments/e180-reading-shards.sh --host-share <k>/<n>    只跑第 k 份（片号 mod n 等于 k 的片），写进片目录，不合并（第二台上用）
#   bash scripts/experiments/e180-reading-shards.sh --merge-only            不跑，片齐了合并出产物
#   bash scripts/experiments/e180-reading-shards.sh --list-items            逐行列片名与做没做完（要装置已编好，不编译）
#   bash scripts/experiments/e180-reading-shards.sh --selftest              假装置、假多机驱动走一遍续跑、分台与停机（不起 cargo）
#   --shard-directory <目录> 可与前四种合用，换片目录（相对仓根或绝对路径）。
#   --binary <程序文件> 可与前四种合用，装置用这一份、不编（命令行优先于 E180_BINARY）。
#   --shipped-binary-sha256 <64 位十六进制> 与 --binary 合用，说明这份程序文件是别台编好送来的：核 sha256 相同才往下（不同退 1），并放 python3 垫片。本机起的命令里不带；第二台那一份由本脚本自己带。
# 环境：E180_BINARY（装置二进制；没给就先 cargo build --release 编仓根 target/ 下的那一份）、E180_SHARD_DIRECTORY（片目录，默认
#   ~/.local/share/singlefs/work/e180-shards）、E180_SHARD_MEMORY_CAP（每片的内存包装上限，默认 16G）、E180_THREADS（见「多机的读法」）、
#   E180_LOG_DIRECTORY（每片的标准输出与错误输出，默认 ~/.local/share/singlefs/work/e180-shards-logs）、E180_PRODUCT_DIRECTORY（产物目录，默认 research/results）。
#   只给自证换假的：E180_HOSTS_EMITTER（默认 scripts/crash/crash-point-replay-shard-configuration-check.sh）、E180_MULTI_HOST_RUN（默认 scripts/hosts/multi-host-run.sh）、
#   E180_MEMORY_WRAPPER（默认 scripts/hosts/run-with-memory-cap.sh）。
# 产物名：…-reading.out 已有时另存 …-reading-<日期>.out，同一天再跑加 -r2、-r3；已有的一个不动。合并出的与最新那一份逐字节相同时不另存，只报已在。
# 退出码：0 做完（--host-share 是这一份的片做完）；1 有片没跑成或合并失败（同一条命令重起续跑）；2 用法错；3 装置报停机（name=stop 那一行交主 agent，不重跑）；
#   78 装置没编好（--list-items）或台数超过 multi-host-run.sh 到得了的两台。
#
# 弄坏开关 E180_SHARDS_BREAK=<项>（只在 --selftest 起的那几趟里认：自证设 E180_SHARDS_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   done-marker-ignored   .shard 文件在就算做完、不看末行          打开后自证「删一片的完成标记重起只补那一片」红
#   host-share-ignored    --host-share 不分份、每台跑全部片        打开后自证「两台各跑自己那一份」红
#   exec-root-binary      每片仍按开跑时给的那个路径起、不按私有副本  打开后自证「装置在一趟里不许换」红（跑着的时候仓根那份被换，后起的片自报的 sha256 就不是开跑时那份）
#   stale-binary-reused   沿用只看完成标记、不看片旁记的装置 sha256    打开后自证「旧装置打的片作废」红
#   merge-sha-unchecked   合并之前不核全部片的装置 sha256            打开后自证「合并之前核全部片的装置 sha256」红（旧装置的片被合进产物）
#   peer-binary-not-shipped  第二台那一份不送程序文件、不带 --binary，第二台用自己编的另一份   打开后自证「两台各跑自己那一份」「第二台用本机送去的同一份程序文件」红
#   no-peer-sha-check     第二台不核送到的程序文件 sha256        打开后自证「第二台拿到的程序文件与本机的 sha256 不同」红
#   no-python-shim        第二台不放 python3 垫片                 打开后自证「第二台用本机送去的同一份程序文件」红（装置的准入路径是编译机的，第二台上找不到）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../hosts/heavy-threads.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "$(dirname "${BASH_SOURCE[0]}")/long-run-lib/binary-identity.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; exit 2; }

BINARY_DEFAULT="target/release/e180_admission_reading_versus_writable_bytes"
PRODUCT_STEM_NAME="e180-admission-reading-versus-writable-bytes-reading"
MEMORY_CAP="${E180_SHARD_MEMORY_CAP:-16G}"
MERGE_MEMORY_CAP="8G"
LOG_DIRECTORY="${E180_LOG_DIRECTORY:-$HOME/.local/share/singlefs/work/e180-shards-logs}"
PRODUCT_DIRECTORY="${E180_PRODUCT_DIRECTORY:-research/results}"
HOSTS_EMITTER="${E180_HOSTS_EMITTER:-scripts/crash/crash-point-replay-shard-configuration-check.sh}"
MULTI_HOST_RUN="${E180_MULTI_HOST_RUN:-scripts/hosts/multi-host-run.sh}"
MEMORY_WRAPPER="${E180_MEMORY_WRAPPER:-scripts/hosts/run-with-memory-cap.sh}"
PEER_SHARD_DIRECTORY="e180-shards-share-1-of-2"
PEER_BINARY_DIRECTORY="e180-binary-for-peer"
DEVICE_SOURCE_PATH="crates/singlefs-checker-tier/src/bin/e180_admission_reading_versus_writable_bytes.rs"

usage_error() {
  echo "  ✗ $1" >&2
  echo "     → 怎么办：见本脚本文件头「用法」" >&2
  exit 2
}

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）；别处设了就拒。
break_is() {
  [[ ",${E180_SHARDS_BREAK:-}," == *",$1,"* ]]
}
if [[ -n "${E180_SHARDS_BREAK:-}" && "${1:-}" != --selftest ]]; then
  if [[ -z "${E180_SHARDS_SELFTEST_MARK:-}" || "$(cat -- "${E180_SHARDS_SELFTEST_MARK}" 2> /dev/null)" != e180-shards-selftest ]]; then
    echo "  ✗ E180_SHARDS_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
    exit 2
  fi
fi

# 一片做完了：文件在，末行是 shard_done<TAB><整份行数>。
is_shard_done() {
  local file="$1" line_count last_line
  [[ -f "$file" ]] || return 1
  break_is done-marker-ignored && return 0
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" == "shard_done"$'\t'"$line_count" ]]
}

# 一片能沿用：做完了，且片旁记的装置 sha256 是这一趟的（没有记录算不同）。
is_shard_reusable() {
  is_shard_done "$1" || return 1
  break_is stale-binary-reused && return 0
  binary_identity_is_current "$1"
}

# 做完了而片旁记的装置 sha256 不是这一趟的（或没有记录）：整份移到作废目录，留着不删，逐片报「作废：…」。
set_aside_stale_shard() {
  binary_identity_void_shard "$1" "$SHARD_DIRECTORY/$1" "$VOID_DIRECTORY" || exit 1
}

# 产物齐了：末行是 E name=done emitted=<行数>，且没有停机行。
is_complete_product() {
  local file="$1" line_count last_line
  [[ -f "$file" ]] || return 1
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" == "E name=done emitted=$line_count" ]] && ! grep -q '^E name=stop ' -- "$file"
}

# 还没被占的产物名：先试没有日期的，已有就加日期，同一天再加 -r2、-r3……
next_free_product_path() {
  local today candidate revision stem="$PRODUCT_DIRECTORY/$PRODUCT_STEM_NAME"
  today=$(TZ=Asia/Tokyo date +%F)
  candidate="$stem.out"
  [[ -e "$candidate" ]] || { echo "$candidate"; return; }
  candidate="$stem-$today.out"
  [[ -e "$candidate" ]] || { echo "$candidate"; return; }
  revision=2
  while [[ -e "$stem-$today-r$revision.out" ]]; do revision=$((revision + 1)); done
  echo "$stem-$today-r$revision.out"
}

# 这台的线程：E180_THREADS 设了用它，没设取登记的每台默认重活线程数。
thread_count() {
  if [[ -n "${E180_THREADS:-}" ]]; then
    [[ "$E180_THREADS" =~ ^[1-9][0-9]*$ ]] || usage_error "E180_THREADS 要是正整数，给的是「$E180_THREADS」"
    echo "$E180_THREADS"
    return
  fi
  heavy_thread_default
}

# 台数：1 + --emit-hosts 打的 MULTI_HOST_COUNT；双机关着（退 3）只有本机。判不过就停。
host_count() {
  local emitted exit_code count
  emitted=$(bash "$HOSTS_EMITTER" --emit-hosts 2> /dev/null)
  exit_code=$?
  if ((exit_code == 3)); then echo 1; return 0; fi
  if ((exit_code != 0)); then
    echo "  ✗ 多机配置判不过（$HOSTS_EMITTER --emit-hosts 退 $exit_code）" >&2
    echo "     → 怎么办：照它打的原因修 multi-host.env，或用 --local-only 只在本机跑" >&2
    return 1
  fi
  count=$(grep -m1 '^MULTI_HOST_COUNT=' <<< "$emitted" | cut -d= -f2)
  [[ "$count" =~ ^[0-9]+$ ]] || { echo "  ✗ --emit-hosts 没打 MULTI_HOST_COUNT" >&2; return 1; }
  echo $((1 + count))
}

# 装置：E180_BINARY 给了用它（不编），没给就先编仓根 target/ 下那一份。
prepare_binary() {
  if [[ -n "$BINARY_OPTION" ]]; then
    BINARY="$BINARY_OPTION"
  elif [[ -n "${E180_BINARY:-}" ]]; then
    BINARY="$E180_BINARY"
  else
    BINARY="$BINARY_DEFAULT"
    bash scripts/hosts/capped.sh "$THREADS" cargo build --release -p singlefs-checker-tier --bin e180_admission_reading_versus_writable_bytes \
      > "$LOG_DIRECTORY/build-$RUN_STAMP.log" 2>&1 || {
      echo "  ✗ 装置编不过（$LOG_DIRECTORY/build-$RUN_STAMP.log）" >&2
      echo "     → 怎么办：照 cargo 打的第一处 error 改装置，再起本脚本" >&2
      exit 1
    }
  fi
  [[ -x "$BINARY" ]] || { echo "  ✗ 装置程序文件不在或不可执行（$BINARY）" >&2; echo "     → 怎么办：没给 E180_BINARY 就看上面 cargo build --release 的输出；给了就核对它指向的文件存在且可执行" >&2; exit 1; }
  # 开跑时拷一份私有副本：之后列片、起每一片、合并都按这份拷贝，BINARY_SHA256 是拷贝的 sha256
  SOURCE_BINARY="$BINARY"
  binary_identity_pin "$SOURCE_BINARY" "$LOG_DIRECTORY/pinned-binary" || exit 1
  BINARY="$PINNED_BINARY"
  BINARY_SHA256="$PINNED_BINARY_SHA256"
  LAUNCH_BINARY="$BINARY"
  break_is exec-root-binary && LAUNCH_BINARY="$SOURCE_BINARY"
  if [[ -n "$SHIPPED_SHA256" ]]; then
    if [[ "$BINARY_SHA256" != "$SHIPPED_SHA256" ]] && ! break_is no-peer-sha-check; then
      echo "  ✗ 送到的装置程序文件 sha256 是 $BINARY_SHA256，本机的是 $SHIPPED_SHA256，不是同一份：这一份一片都不跑" >&2
      echo "     → 怎么办：看 multi-host-run.sh 的 --send 有没有送全（盘满、传输中断）；重起本机那条命令" >&2
      exit 1
    fi
    install_python_path_shim
  fi
}

# 别台编好送来的装置：它的准入脚本路径是编译机的绝对路径，这台上没有；PATH 最前面放 python3 垫片（做法在 long-run-lib/binary-identity.sh）。
install_python_path_shim() {
  break_is no-python-shim && return 0
  binary_identity_install_python_shim "$DEVICE_SOURCE_PATH" "$LOG_DIRECTORY/python3-path-shim-$RUN_STAMP" || exit 1
}

# 片的清单：一行「片名 TAB 几何 TAB 族 TAB 占的线程数」，次序是装置 list-shards 的次序（也是合并的次序）。
load_shard_list() {
  mapfile -t SHARD_LINES < <(E180_THREADS="$THREADS" bash "$MEMORY_WRAPPER" 1G "$BINARY" list-shards)
  ((${#SHARD_LINES[@]} > 0)) || { echo "  ✗ 装置 list-shards 什么都没列（$BINARY）" >&2; exit 1; }
}

# 跑点名的那几片（SHARD_LINES 的下标，按给的次序起）：这台上在跑的片占的线程合计不超过 THREADS，每片的退出码写进这一趟的 jobs 目录。
# 交回时设 RAN_NOW（这一趟跑完、片齐了的个数）、STOPPED（装置报停机的片）、FAILED（别的没跑成的片）。
run_shards() {
  local jobs="$LOG_DIRECTORY/jobs-$RUN_STAMP" index name geometry family threads busy running exit_code
  local -A threads_of=()
  local launched=()
  mkdir -p -- "$jobs"
  for index in "$@"; do
    IFS=$'\t' read -r name geometry family threads <<< "${SHARD_LINES[$index]}"
    ((threads > THREADS)) && threads=$THREADS
    while :; do
      busy=0
      for running in "${!threads_of[@]}"; do
        if [[ -e "$jobs/$running.exit" ]]; then unset 'threads_of[$running]'; else busy=$((busy + threads_of[$running])); fi
      done
      ((busy + threads <= THREADS)) && break
      wait -n
    done
    threads_of[$name]=$threads
    binary_identity_assert_unchanged || exit 1
    echo "片 $name：起跑（占 $threads 线程，这台已占 $busy）"
    { E180_THREADS="$THREADS" bash "$MEMORY_WRAPPER" "$MEMORY_CAP" nice -n 19 "$LAUNCH_BINARY" reading --geometry "$geometry" --family "$family" \
        --shard-output "$SHARD_DIRECTORY/$name" > "$LOG_DIRECTORY/$name-$RUN_STAMP.out" 2> "$LOG_DIRECTORY/$name-$RUN_STAMP.err"
      echo "$?" > "$jobs/$name.exit"; } &
    launched+=("$name")
  done
  wait
  RAN_NOW=0; STOPPED=(); FAILED=()
  for name in ${launched[@]+"${launched[@]}"}; do
    exit_code=$(cat -- "$jobs/$name.exit" 2> /dev/null || echo missing)
    if [[ "$exit_code" == 0 ]] && is_shard_done "$SHARD_DIRECTORY/$name"; then
      # 片旁记下打它的装置 sha256（这份私有副本的）；记不下这片就不算做完，下一趟照没记录作废重跑
      if ! binary_identity_stamp_write "$SHARD_DIRECTORY/$name"; then
        FAILED+=("$name")
        echo "  ✗ 片 $name：做完了但片旁的装置记录写不下去" >&2
        echo "     → 怎么办：核对片目录可写、盘没满，同一条命令重起；这一片会按没有记录作废重跑" >&2
        continue
      fi
      RAN_NOW=$((RAN_NOW + 1))
      echo "片 $name：做完（$(grep -m1 '^E name=shard_written ' "$LOG_DIRECTORY/$name-$RUN_STAMP.out" || echo '没有 shard_written 行')）"
    elif [[ "$exit_code" == 4 ]]; then
      STOPPED+=("$name")
      echo "  ✗ 片 $name：装置报停机：$(grep -m1 '^E name=stop ' "$LOG_DIRECTORY/$name-$RUN_STAMP.out")" >&2
    else
      FAILED+=("$name")
      echo "  ✗ 片 $name：退 $exit_code，没做完（错误输出 $LOG_DIRECTORY/$name-$RUN_STAMP.err；245 与 250–254 是内存包装的结局，见 run-with-memory-cap.sh 文件头「退出码」）" >&2
    fi
  done
  local finished_count
  finished_count=$(find "$jobs" -maxdepth 1 -name '*.exit' | wc -l)
  if ((finished_count != ${#launched[@]})); then
    echo "  ✗ 起了 ${#launched[@]} 片，收回 $finished_count 个退出码，对不上：这一趟作废" >&2
    exit 1
  fi
}

# 第二台那一份：经 multi-host-run.sh --peer 在第二台的树里跑 --host-share 1/2；本机片目录送过去（做完的片在那边沿用），它的片目录取回。
start_peer_share() {
  PEER_FETCH_DIRECTORY="$LOG_DIRECTORY/peer-fetch-$RUN_STAMP"
  PEER_EXIT_FILE="$LOG_DIRECTORY/peer-$RUN_STAMP.exit"
  local binary_name binary_arguments=() binary_send=()
  if ! break_is peer-binary-not-shipped; then
    # 本机这一份程序文件（私有副本）送过去（暂存目录里放一个硬链接，跨文件系统时拷一份），第二台核 sha256 后用它，不自己编
    binary_name=$(basename -- "$BINARY")
    PEER_BINARY_STAGE="$LOG_DIRECTORY/peer-binary-$RUN_STAMP"
    mkdir -p -- "$PEER_BINARY_STAGE" || exit 1
    trap 'rm -rf -- "$PEER_BINARY_STAGE"' EXIT
    ln -- "$BINARY" "$PEER_BINARY_STAGE/$binary_name" 2> /dev/null || cp -p -- "$BINARY" "$PEER_BINARY_STAGE/$binary_name" || exit 1
    binary_send=(--send "$PEER_BINARY_STAGE" "$PEER_BINARY_DIRECTORY")
    binary_arguments=(--binary "$PEER_BINARY_DIRECTORY/$binary_name" --shipped-binary-sha256 "$BINARY_SHA256")
  fi
  echo "第二台：起 --host-share 1/2（日志 $LOG_DIRECTORY/peer-$RUN_STAMP.log）"
  { MULTI_HOST_RUN_FETCH_DIRECTORY="$PEER_FETCH_DIRECTORY" bash "$MULTI_HOST_RUN" --peer --label "e180-reading-$RUN_STAMP" \
      --send "$SHARD_DIRECTORY" "$PEER_SHARD_DIRECTORY" ${binary_send[@]+"${binary_send[@]}"} --fetch "$PEER_SHARD_DIRECTORY" \
      -- bash scripts/experiments/e180-reading-shards.sh --host-share 1/2 --shard-directory "$PEER_SHARD_DIRECTORY" \
      ${binary_arguments[@]+"${binary_arguments[@]}"} \
      > "$LOG_DIRECTORY/peer-$RUN_STAMP.log" 2>&1
    echo "$?" > "$PEER_EXIT_FILE"; } &
}

# 取回的片里做完的并进本机片目录（本机已做完的不覆盖）；交回时设 IMPORTED。
import_peer_shards() {
  local fetched="$PEER_FETCH_DIRECTORY/$PEER_SHARD_DIRECTORY" file name
  IMPORTED=0
  [[ -d "$fetched" ]] || { echo "  ✗ 第二台的片目录没取回来（$fetched）" >&2; return 1; }
  for file in "$fetched"/*.shard; do
    [[ -e "$file" ]] || continue
    name=$(basename -- "$file")
    is_shard_done "$file" || continue
    # 第二台取回的片要记着是这一趟的装置打的（第二台用本机送去的那一份），不是的不并进来
    if ! is_shard_reusable "$file"; then
      echo "  ✗ 第二台取回的片 $name 的装置 sha256（$(binary_identity_stamp_read "$file")）不是这一趟本机的（$BINARY_SHA256），不并进来" >&2
      continue
    fi
    is_shard_reusable "$SHARD_DIRECTORY/$name" && continue
    cp -- "$file" "$SHARD_DIRECTORY/$name.importing-$$" && mv -- "$SHARD_DIRECTORY/$name.importing-$$" "$SHARD_DIRECTORY/$name" \
      && binary_identity_stamp_write "$SHARD_DIRECTORY/$name" "$(binary_identity_stamp_read "$file")" \
      && IMPORTED=$((IMPORTED + 1))
  done
}

# 片齐了合并出产物（与最新那一份逐字节相同就不另存）。
merge_product() {
  local temporary="$LOG_DIRECTORY/merge-$RUN_STAMP.out" newest destination checked_pairs=() checked_line checked_name
  # 合并之前再核一遍：每一片片旁记的装置 sha256 都得是这一趟的，不同或没有记录就拒、不出产物
  if ! break_is merge-sha-unchecked; then
    for checked_line in "${SHARD_LINES[@]}"; do
      checked_name="${checked_line%%$'\t'*}"
      checked_pairs+=("$checked_name" "$(binary_identity_stamp_read "$SHARD_DIRECTORY/$checked_name")")
    done
    binary_identity_check_same "$BINARY_SHA256" "${checked_pairs[@]}" || exit 1
  fi
  bash "$MEMORY_WRAPPER" "$MERGE_MEMORY_CAP" "$BINARY" merge "$SHARD_DIRECTORY" > "$temporary" 2> "$LOG_DIRECTORY/merge-$RUN_STAMP.err"
  if ! is_complete_product "$temporary"; then
    echo "  ✗ 合并没出齐（$temporary）：$(grep -m1 '^E name=stop ' "$temporary" || tail -n 1 -- "$temporary")" >&2
    echo "     → 怎么办：片对不上（合并拒）就看那一行点名的片；别的看 $LOG_DIRECTORY/merge-$RUN_STAMP.err；产物不入库" >&2
    exit 1
  fi
  mkdir -p -- "$PRODUCT_DIRECTORY" || exit 1
  newest=$(ls -t -- "$PRODUCT_DIRECTORY/$PRODUCT_STEM_NAME"*.out 2> /dev/null | head -n 1)
  if [[ -n "$newest" ]] && cmp -s -- "$newest" "$temporary"; then
    echo "产物已在、与这一次合并出的逐字节相同：$newest（不另存）"
    rm -f -- "$temporary"
    return 0
  fi
  destination=$(next_free_product_path)
  mv -n -- "$temporary" "$destination"
  [[ -e "$temporary" ]] && { echo "  ✗ 产物名 $destination 被占了，没挪进去（合并出的留在 $temporary）" >&2; exit 1; }
  echo "产物：$destination（$(wc -l < "$destination") 行，sha256 $(sha256sum -- "$destination" | cut -d' ' -f1)）"
}

# 自证：假装置（五片、两个假几何）、假内存包装（直接起命令）、假机器清单、假 multi-host-run（在临时树里跑 --host-share 1/2），
# 每一格在自己的临时目录里从头搭，格与格不共用状态。
run_selftest() {
  local work failures=0 fakes
  work=$(mktemp -d) || return 1
  fakes="$work/fakes"
  mkdir -p -- "$fakes"
  echo e180-shards-selftest > "$work/mark"
  cat > "$fakes/device" <<'FAKE'
#!/usr/bin/env bash
# 若装置起 python3 判准入，路径编进了编译机的树；E180_SELFTEST_BAKED_TREE 设了就照这个样子判，判不过照真装置的样子退 python3 的码。
if [[ -n "${E180_SELFTEST_BAKED_TREE:-}" ]]; then
  python3 "$E180_SELFTEST_BAKED_TREE/crates/singlefs-checker-tier/../../.claude/singlefs-ai-sop/scripts/preflight.py" check \
    "$E180_SELFTEST_BAKED_TREE/crates/singlefs-checker-tier/src/bin/e180_admission_reading_versus_writable_bytes.rs" -- "$@" > /dev/null 2>&1 || exit 2
  echo "${E180_SELFTEST_HOST_TAG:-local} $*" >> "$E180_SELFTEST_CALLS.preflight-passed"
fi
# E180_SELFTEST_SWAP_TARGET：列片的时候（开跑之后、起第一片之前）把这个路径的装置换成另一份字节不同的，像别的会话的 cargo 把仓根那份换掉。
if [[ -n "${E180_SELFTEST_SWAP_TARGET:-}" && "$1" == list-shards && ! -e "$E180_SELFTEST_SWAP_TARGET.swapped" ]]; then
  { cat -- "$E180_SELFTEST_SWAP_TARGET"; echo "# swapped by a build while the long run is going"; } > "$E180_SELFTEST_SWAP_TARGET.swap" \
    && chmod +x "$E180_SELFTEST_SWAP_TARGET.swap" && mv -f -- "$E180_SELFTEST_SWAP_TARGET.swap" "$E180_SELFTEST_SWAP_TARGET" && : > "$E180_SELFTEST_SWAP_TARGET.swapped"
fi
case "$1" in
  list-shards) printf '%s\t%s\t%s\t%s\n' g_a-hf.shard g_a hf 2 g_a-ho.shard g_a ho 2 g_a-hl.shard g_a hl 1 g_b-hf.shard g_b hf 2 g_b-ho.shard g_b ho 2 ;;
  reading)
    echo "${E180_SELFTEST_HOST_TAG:-local} $3-$5" >> "$E180_SELFTEST_CALLS"
    echo "$3-$5 $(sha256sum -- "$0" | cut -d' ' -f1)" >> "$E180_SELFTEST_CALLS.self-sha"
    if [[ "$3-$5" == "${E180_SELFTEST_STOP_SHARD:-}" ]]; then echo "E name=stop reason=selftest_stop"; exit 4; fi
    printf 'geometry\t%s\nfamily\t%s\nshard_done\t3\n' "$3" "$5" > "$7.partial-$$" && mv -- "$7.partial-$$" "$7"
    echo "E name=shard_written shard=$3-$5.shard" ;;
  merge)
    for shard in g_a-hf g_a-ho g_a-hl g_b-hf g_b-ho; do
      [[ -f "$2/$shard.shard" ]] || { echo "E name=stop reason=shard_merge_refused missing_shard=$shard.shard"; exit 4; }
    done
    printf 'E name=merged shards=5\nE name=done emitted=2\n' ;;
esac
FAKE
  printf '%s\n' '#!/usr/bin/env bash' 'shift' 'exec "$@"' > "$fakes/wrapper"
  cat > "$fakes/hosts" <<'FAKE'
#!/usr/bin/env bash
case "$E180_SELFTEST_HOSTS" in
  1) echo "双机：关"; exit 3 ;;
  *) echo "MULTI_HOST_COUNT=$((E180_SELFTEST_HOSTS - 1))"; exit 0 ;;
esac
FAKE
  cat > "$fakes/multi-host-run" <<'FAKE'
#!/usr/bin/env bash
# 假 multi-host-run.sh --peer：每个 --send 的目录拷进临时树（保留执行位），--shard-directory 与 --binary 的值换成临时树里的绝对路径，在仓根跑那条命令，再把 --fetch 的目录拷到取回目录。
# 命令里没有 --binary（第二台自己编）时，第二台用的是它自己编的另一份装置（字节不同，sha256 就不同）；有 --binary 时第二台上的装置带着编译机的准入路径（那棵树在这里不存在）。
# E180_SELFTEST_PEER_CORRUPTS_BINARY=1：送到的程序文件末尾多一个字节（传输坏了）。
tree=$(mktemp -d); fetch=""; send_sources=(); send_relatives=()
while [[ "$1" != -- ]]; do
  case "$1" in
    --peer) shift ;;
    --label) shift 2 ;;
    --fetch) fetch="$2"; shift 2 ;;
    --send) send_sources+=("$2"); send_relatives+=("$3"); shift 3 ;;
  esac
done
shift
for send_index in "${!send_sources[@]}"; do
  mkdir -p -- "$tree/${send_relatives[$send_index]}" && cp -p -- "${send_sources[$send_index]}"/* "$tree/${send_relatives[$send_index]}/" 2> /dev/null
  if [[ "${send_relatives[$send_index]}" == *binary* && "${E180_SELFTEST_PEER_CORRUPTS_BINARY:-0}" == 1 ]]; then
    for sent_file in "$tree/${send_relatives[$send_index]}"/*; do chmod u+w "$sent_file"; printf x >> "$sent_file"; done
  fi
done
words=(); shipped=0
while (($# > 0)); do
  if [[ "$1" == --shard-directory || "$1" == --binary ]]; then
    [[ "$1" == --binary ]] && shipped=1
    words+=("$1" "$tree/$2"); shift 2
  else words+=("$1"); shift; fi
done
if ((shipped)); then export E180_SELFTEST_BAKED_TREE="$tree/builder-tree-that-is-not-here"; else export E180_BINARY="$E180_SELFTEST_PEER_BUILT_DEVICE"; fi
E180_SELFTEST_HOST_TAG=peer "${words[@]}"
exit_code=$?
mkdir -p -- "$MULTI_HOST_RUN_FETCH_DIRECTORY" && cp -r -- "$tree/$fetch" "$MULTI_HOST_RUN_FETCH_DIRECTORY/$fetch"
exit "$exit_code"
FAKE
  # 第二台自己编的那一份：同一份源码、字节不同（编进去的树路径不同）
  { cat "$fakes/device"; echo '# built on the peer in another tree'; } > "$fakes/device-built-on-peer"
  chmod +x "$fakes"/*
  # 一格的环境：片目录、日志、产物都在这一格自己的目录里。
  invoke() {
    local cell="$1" hosts="$2"
    shift 2
    E180_BINARY="${INVOKE_BINARY:-$fakes/device}" E180_MEMORY_WRAPPER="$fakes/wrapper" E180_HOSTS_EMITTER="$fakes/hosts" E180_MULTI_HOST_RUN="$fakes/multi-host-run" \
      E180_SELFTEST_PEER_BUILT_DEVICE="$fakes/device-built-on-peer" E180_SELFTEST_HOSTS="$hosts" E180_SELFTEST_CALLS="$cell/calls" E180_THREADS=3 E180_SHARDS_SELFTEST_MARK="$work/mark" \
      E180_SHARD_DIRECTORY="$cell/shards" E180_LOG_DIRECTORY="$cell/logs" E180_PRODUCT_DIRECTORY="$cell/product" \
      bash "$SCRIPT_PATH" "$@" > "$cell/out" 2>&1
  }
  expect() {
    local name="$1" condition="$2"
    cell_count=$((cell_count + 1))
    if eval "$condition"; then echo "  ✓ $name"; else echo "  ✗ $name：条件「$condition」不成立"; failures=$((failures + 1)); fi
  }
  local cell exit_code first_exit old_sha original_sha cell_count=0
  cell="$work/single"; mkdir -p -- "$cell"
  invoke "$cell" 1; exit_code=$?
  expect "单机全新：五片都现跑、合并出产物" '((exit_code == 0)) && grep -q "现跑 5（本机 5、第二台 0）、沿用 0" "$cell/out" && (($(wc -l < "$cell/calls") == 5)) && [[ -f "$cell/product/e180-admission-reading-versus-writable-bytes-reading.out" ]]'
  invoke "$cell" 1; exit_code=$?
  expect "原样重起：五片都沿用、产物不另存" '((exit_code == 0)) && grep -q "现跑 0（本机 0、第二台 0）、沿用 5" "$cell/out" && (($(wc -l < "$cell/calls") == 5)) && grep -q "不另存" "$cell/out"'
  head -n 2 -- "$cell/shards/g_a-ho.shard" > "$cell/shards/g_a-ho.shard.cut" && mv -- "$cell/shards/g_a-ho.shard.cut" "$cell/shards/g_a-ho.shard"
  invoke "$cell" 1; exit_code=$?
  expect "删一片的完成标记重起只补那一片" '((exit_code == 0)) && grep -q "现跑 1（本机 1、第二台 0）、沿用 4" "$cell/out" && [[ "$(tail -n 1 -- "$cell/calls")" == "local g_a-ho" ]] && (($(wc -l < "$cell/calls") == 6))'
  cell="$work/two"; mkdir -p -- "$cell"
  invoke "$cell" 2; exit_code=$?
  expect "两台各跑自己那一份（本机片号 0、2、4，第二台 1、3）" '((exit_code == 0)) && [[ "$(grep "^local " "$cell/calls" | sort | tr "\n" " ")" == "local g_a-hf local g_a-hl local g_b-ho " ]] && [[ "$(grep "^peer " "$cell/calls" | sort | tr "\n" " ")" == "peer g_a-ho peer g_b-hf " ]] && grep -q "现跑 5（本机 3、第二台 2）、沿用 0" "$cell/out"'
  cell="$work/three"; mkdir -p -- "$cell"
  invoke "$cell" 3; exit_code=$?
  expect "三台：退 78、一片都不跑" '((exit_code == 78)) && [[ ! -e "$cell/calls" ]]'
  cell="$work/stop"; mkdir -p -- "$cell"
  E180_SELFTEST_STOP_SHARD=g_b-hf invoke "$cell" 1; exit_code=$?
  expect "装置报停机：退 3、那一片没有片文件、别的片做完" '((exit_code == 3)) && [[ ! -e "$cell/shards/g_b-hf.shard" ]] && (($(find "$cell/shards" -name "*.shard" | wc -l) == 4))'
  cell="$work/usage"; mkdir -p -- "$cell"
  invoke "$cell" 1 --host-share 2/2; exit_code=$?
  expect "第几份不小于共几份：用法错退 2" '((exit_code == 2))'
  invoke "$cell" 1 --shipped-binary-sha256 "$(printf 'a%.0s' {1..64})"; exit_code=$?
  expect "--shipped-binary-sha256 不带 --binary：用法错退 2" '((exit_code == 2)) && grep -q "要与 --binary 合用" "$cell/out"'
  cell="$work/two-shipped"; mkdir -p -- "$cell"
  invoke "$cell" 2; exit_code=$?
  expect "第二台用本机送去的同一份程序文件：装置的准入路径换到第二台的树里判过三次（列片、两片）、五片旁记的 sha256 只有一种、合并出产物" '((exit_code == 0)) && [[ "$(grep -c "^peer " "$cell/calls.preflight-passed")" == 3 ]] && (($(cat "$cell"/shards/*.binary-sha256 | sort -u | wc -l) == 1)) && (($(ls "$cell"/shards/*.binary-sha256 | wc -l) == 5)) && [[ -f "$cell/product/e180-admission-reading-versus-writable-bytes-reading.out" ]]'
  cell="$work/peer-corrupted"; mkdir -p -- "$cell"
  E180_SELFTEST_PEER_CORRUPTS_BINARY=1 invoke "$cell" 2; exit_code=$?
  expect "第二台拿到的程序文件与本机的 sha256 不同：第二台一片不跑、本机不合并、退 1" '((exit_code == 1)) && [[ ! -e "$cell/calls.preflight-passed" ]] && ! grep -q "^peer " "$cell/calls" && [[ ! -d "$cell/product" ]] && grep -q "不是同一份" "$cell"/logs/peer-*.log'
  cell="$work/old-binary"; mkdir -p -- "$cell/bin"; cp -- "$fakes/device" "$cell/bin/device"
  INVOKE_BINARY="$cell/bin/device" invoke "$cell" 1; first_exit=$?
  old_sha=$(sha256sum -- "$cell/bin/device" | cut -d' ' -f1)
  echo "# a newer build of the device" >> "$cell/bin/device"
  INVOKE_BINARY="$cell/bin/device" invoke "$cell" 1; exit_code=$?
  expect "旧装置打的五片全作废：逐片报作废、五片全现跑、旧片连记录整份留在作废目录、片旁记的都是新装置的" '((first_exit == 0 && exit_code == 0)) && grep -q "现跑 5（本机 5、第二台 0）、沿用 0" "$cell/out" && (($(grep -c "^作废：g_.-h.\.shard（装置 sha256 不同）" "$cell/out") == 5)) && (($(find "$cell"/logs/void-* -name "*.shard" | wc -l) == 5)) && (($(find "$cell"/logs/void-* -name "*.binary-sha256" | wc -l) == 5)) && ! grep -q "$old_sha" "$cell"/shards/*.binary-sha256 && (($(cat "$cell"/shards/*.binary-sha256 | sort -u | wc -l) == 1))'
  cell="$work/stale-one"; mkdir -p -- "$cell"
  invoke "$cell" 1; exit_code=$?
  printf '%s\n' "$(printf 'f%.0s' {1..64})" > "$cell/shards/g_b-hf.shard.binary-sha256"
  rm -f -- "$cell/shards/g_a-hl.shard.binary-sha256"
  invoke "$cell" 1; exit_code=$?
  expect "片旁记的 sha256 不同的、没有记录的片都作废重跑，另外三片沿用" '((exit_code == 0)) && grep -q "现跑 2（本机 2、第二台 0）、沿用 3" "$cell/out" && grep -q "^作废：g_b-hf.shard（装置 sha256 不同）：记的是 f" "$cell/out" && grep -q "^作废：g_a-hl.shard（装置 sha256 不同）：记的是 没有记录" "$cell/out" && (($(wc -l < "$cell/calls") == 7))'
  cell="$work/merge-check"; mkdir -p -- "$cell"
  invoke "$cell" 1; exit_code=$?
  printf '%s\n' "$(printf 'f%.0s' {1..64})" > "$cell/shards/g_a-ho.shard.binary-sha256"
  invoke "$cell" 1 --merge-only; exit_code=$?
  expect "合并之前核全部片的装置 sha256：有一片对不上就拒（退 1、点名那一片、不另出产物）" '((exit_code == 1)) && grep -q "片 g_a-ho.shard 的装置 sha256 对不上" "$cell/out" && grep -q "合并产物之前核全部片的装置 sha256" "$cell/out" && (($(find "$cell/product" -name "e180-admission-reading-versus-writable-bytes-reading*.out" | wc -l) == 1))'
  cell="$work/list"; mkdir -p -- "$cell/bin"; cp -- "$fakes/device" "$cell/bin/device"
  INVOKE_BINARY="$cell/bin/device" invoke "$cell" 1; first_exit=$?
  echo "# a newer build of the device" >> "$cell/bin/device"
  INVOKE_BINARY="$cell/bin/device" invoke "$cell" 1 --list-items; exit_code=$?
  expect "--list-items：装置换了之后，旧装置打的片标出续跑时作废" '((first_exit == 0 && exit_code == 0)) && (($(grep -c "做完（装置 sha256 与现在这份不同，续跑时作废）" "$cell/out") == 5))'
  cell="$work/swap"; mkdir -p -- "$cell/bin"; cp -- "$fakes/device" "$cell/bin/device"
  original_sha=$(sha256sum -- "$cell/bin/device" | cut -d' ' -f1)
  E180_SELFTEST_SWAP_TARGET="$cell/bin/device" INVOKE_BINARY="$cell/bin/device" invoke "$cell" 1; exit_code=$?
  expect "装置在一趟里不许换：跑着的时候仓根那份被换掉，之后起的五片自报的 sha256 与片旁记的都是开跑时那份" '((exit_code == 0)) && [[ -e "$cell/bin/device.swapped" ]] && [[ "$(sha256sum -- "$cell/bin/device" | cut -d" " -f1)" != "$original_sha" ]] && (($(wc -l < "$cell/calls.self-sha") == 5)) && [[ "$(awk "{print \$2}" "$cell/calls.self-sha" | sort -u)" == "$original_sha" ]] && [[ "$(cat "$cell"/shards/*.binary-sha256 | sort -u)" == "$original_sha" ]]'
  rm -rf -- "$work"
  if ((failures > 0)); then echo "e180-reading-shards.sh 自证：$failures 格红（共 $cell_count 格）"; return 1; fi
  echo "e180-reading-shards.sh 自证：$cell_count 格全绿"
  return 0
}

mode=run
local_only=0
share=""
SHARD_DIRECTORY="${E180_SHARD_DIRECTORY:-$HOME/.local/share/singlefs/work/e180-shards}"
BINARY_OPTION=""
SHIPPED_SHA256=""
while (($# > 0)); do
  case "$1" in
    --binary) (($# >= 2)) || usage_error "--binary 后面要写装置程序文件"; BINARY_OPTION="$2"; shift 2 ;;
    --shipped-binary-sha256) (($# >= 2)) || usage_error "--shipped-binary-sha256 后面要写 64 位十六进制"; SHIPPED_SHA256="$2"; shift 2 ;;
    --local-only) local_only=1; shift ;;
    --host-share) (($# >= 2)) || usage_error "--host-share 后面要写 第几份/共几份"; share="$2"; mode=share; shift 2 ;;
    --merge-only) mode=merge; shift ;;
    --list-items) mode=list; shift ;;
    --selftest) mode=selftest; shift ;;
    --shard-directory) (($# >= 2)) || usage_error "--shard-directory 后面要写目录"; SHARD_DIRECTORY="$2"; shift 2 ;;
    *) usage_error "认不得的参数「$1」" ;;
  esac
done
if [[ "$mode" == share && ! "$share" =~ ^[0-9]+/[1-9][0-9]*$ ]]; then usage_error "--host-share 要写 第几份/共几份（两个整数），给的是「$share」"; fi
if [[ "$mode" == share ]] && ((${share%/*} >= ${share#*/})); then usage_error "--host-share 的第几份要小于共几份，给的是「$share」"; fi

if [[ -n "$SHIPPED_SHA256" ]]; then
  [[ "$SHIPPED_SHA256" =~ ^[0-9a-f]{64}$ ]] || usage_error "--shipped-binary-sha256 要写 64 位小写十六进制，给的是「$SHIPPED_SHA256」"
  [[ -n "$BINARY_OPTION" ]] || usage_error "--shipped-binary-sha256 要与 --binary 合用（核的是送到的那一份程序文件）"
fi

if [[ "$mode" == selftest ]]; then
  run_selftest
  exit $?
fi

RUN_STAMP="$(TZ=Asia/Tokyo date +%F)-$$"
VOID_DIRECTORY="$LOG_DIRECTORY/void-$RUN_STAMP"
THREADS=$(thread_count) || { echo "  ✗ 取不到这台的线程数（原因与出路见上一行）；设 E180_THREADS，或修好 .claude/heavy-thread-default" >&2; exit 2; }
mkdir -p -- "$LOG_DIRECTORY" "$SHARD_DIRECTORY" || exit 1

if [[ "$mode" == list ]]; then
  BINARY="${BINARY_OPTION:-${E180_BINARY:-$BINARY_DEFAULT}}"
  [[ -x "$BINARY" ]] || { echo "  ✗ 装置还没编好：$BINARY" >&2; echo "     → 怎么办：先不带 --list-items 跑一次本脚本，或 cargo build --release -p singlefs-checker-tier --bin e180_admission_reading_versus_writable_bytes" >&2; exit 78; }
  load_shard_list
  PINNED_BINARY_SHA256=$(binary_identity_sha256_of "$BINARY") || { echo "  ✗ 算不出装置程序文件的 sha256（$BINARY）" >&2; exit 1; }
  for line in "${SHARD_LINES[@]}"; do
    name="${line%%$'\t'*}"
    if is_shard_reusable "$SHARD_DIRECTORY/$name"; then echo "$name 做完"
    elif is_shard_done "$SHARD_DIRECTORY/$name"; then echo "$name 做完（装置 sha256 与现在这份不同，续跑时作废）"
    else echo "$name 没做"; fi
  done
  exit 0
fi

hosts=1
if [[ "$mode" == run && "$local_only" == 0 ]]; then
  hosts=$(host_count) || exit 1
  if ((hosts > 2)); then
    echo "  ✗ 多机配置列了 $hosts 台，multi-host-run.sh --peer 只到得了 PEER 那一台" >&2
    echo "     → 怎么办：改多机配置，或用 --local-only 只在本机跑、再把别的台跑出的片拷进片目录" >&2
    exit 78
  fi
fi
prepare_binary
load_shard_list
echo "e180 分片跑法：$hosts 台，这台线程 $THREADS、每片内存上限 $MEMORY_CAP、片目录 $SHARD_DIRECTORY、共 ${#SHARD_LINES[@]} 片"

if [[ "$mode" == merge ]]; then
  for line in "${SHARD_LINES[@]}"; do
    name="${line%%$'\t'*}"
    is_shard_done "$SHARD_DIRECTORY/$name" || { echo "  ✗ 片还没齐：缺 $name" >&2; exit 1; }
  done
  merge_product
  exit 0
fi

# 这台要跑的片：分份时片号 mod 共几份等于第几份；默认跑法时片号 mod 台数等于 0（本机是第 0 台）。做完的沿用。
this_share=0; share_count=$hosts
if [[ "$mode" == share ]]; then this_share=${share%/*}; share_count=${share#*/}; fi
break_is host-share-ignored && share_count=1 && this_share=0
selected=(); reused=0
for index in "${!SHARD_LINES[@]}"; do
  name="${SHARD_LINES[$index]%%$'\t'*}"
  in_this_share=0
  ((index % share_count == this_share)) && in_this_share=1
  if is_shard_reusable "$SHARD_DIRECTORY/$name"; then
    # 默认跑法数全部做完的片（第二台那一份里做完的也是沿用）；分份时只数这一份里的。
    [[ "$mode" == run || "$in_this_share" == 1 ]] && reused=$((reused + 1))
    continue
  fi
  # 做完了而片旁记的装置 sha256 不是这一趟的：移开、这一份里的现跑（默认跑法里谁的份都移开，别台送来的片目录里不留它）。
  if is_shard_done "$SHARD_DIRECTORY/$name" && [[ "$mode" == run || "$in_this_share" == 1 ]]; then
    set_aside_stale_shard "$name"
  fi
  ((in_this_share)) && selected+=("$index")
done
peer_started=0
if [[ "$mode" == run ]] && ((hosts == 2)); then
  start_peer_share
  peer_started=1
fi
run_shards ${selected[@]+"${selected[@]}"}
if ((${#STOPPED[@]} > 0)); then
  echo "  ✗ ${#STOPPED[@]} 片装置报停机（${STOPPED[*]}），该停下交回主 agent，不重跑" >&2
  exit 3
fi
if [[ "$mode" == share ]]; then
  ((${#FAILED[@]} == 0)) || { echo "  ✗ ${#FAILED[@]} 片没做完（${FAILED[*]}）；同一条命令重起只补没做完的" >&2; exit 1; }
  echo "这一份（$share）的片做完：现跑 $RAN_NOW、沿用 $reused；片目录 $SHARD_DIRECTORY"
  exit 0
fi
IMPORTED=0
if ((peer_started)); then
  peer_exit=$(cat -- "$PEER_EXIT_FILE" 2> /dev/null || echo missing)
  [[ "$peer_exit" == 0 ]] || echo "  ✗ 第二台那一份退 $peer_exit（日志 $LOG_DIRECTORY/peer-$RUN_STAMP.log）；取回了的片照样并进来" >&2
  import_peer_shards
  [[ "$peer_exit" == 3 ]] && { echo "  ✗ 第二台那一份里装置报停机，该停下交回主 agent，不重跑" >&2; exit 3; }
fi
((${#FAILED[@]} == 0)) || { echo "  ✗ ${#FAILED[@]} 片没做完（${FAILED[*]}）；同一条命令重起只补没做完的" >&2; exit 1; }
missing=()
for line in "${SHARD_LINES[@]}"; do
  name="${line%%$'\t'*}"
  is_shard_done "$SHARD_DIRECTORY/$name" || missing+=("$name")
done
((${#missing[@]} == 0)) || { echo "  ✗ 片还没齐：缺 ${missing[*]}；同一条命令重起只补没做完的" >&2; exit 1; }
merge_product
echo "片齐了：现跑 $((RAN_NOW + IMPORTED))（本机 $RAN_NOW、第二台 $IMPORTED）、沿用 $reused（共 ${#SHARD_LINES[@]} 片）"
exit 0
