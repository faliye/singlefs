#!/usr/bin/env bash
# admission: always 每一次调都照此刻片目录里哪几片做完了判还差什么，上一次跑完不替这一次作保
# run-condition: command cargo git sha256sum python3
# task-server-shards: *
# E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数）第三段（问题单 H5，跑前登记 research/prompts/e183-preregistration.md 5.5「第三段」与其下的分片、续跑、多机三行）的分片跑法：
#   装置是入库装置 crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs（bin 同名）。一片 = 装置的一次
#   h5 shard <片名> --shard-output <片目录>/<片名> --binary-sha256 <装置程序文件的 sha256>；片齐了由装置 h5 merge <片目录> --binary-sha256 <…> 按片的固定次序
#   接成产物 research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-<日期>.out 并算判定（装置单测
#   sweep_summary_judges_the_page_reading_the_write_amplification_and_the_candidate_clause_on_answer_known_synthetic_shards 钉判定那一半）。
#
# long-run-shards: 一片 = 装置一次 h5 shard <片名>：S 三档 × θ 四档 × 卸载方式两种 = 24 格各一片，加「交给后台」对照一片、段首那一格的确定性重跑一片，共 26 片（装置 h5 list-shards 逐行列片名与这一片占的线程数）；片与片互不依赖、各从自己的 mkfs 起，装置 h5 merge 按 list-shards 的次序接成产物
# long-run-resume: 同一条命令重起就是续跑：装置把一片先写进 <片>.partial-<进程号>、刷盘再换名成 <片>，末行是 E7RESULT name=shard_done shard=<片名> lines=<整份行数>；重起时末行齐、且 h5_config 行的 binary_sha256 与这一趟的程序文件相同的片沿用、只补没做完的（sha256 不同的片移到 <片目录>.void-<前 12 位>/ 再现跑，逐片报「作废：<片名>（装置 sha256 不同）」），成功那句报「现跑 N、沿用 M」
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
#   多机的读法：台数 = 1 + --emit-hosts 打的 MULTI_HOST_COUNT（双机关着、它退 3 时只有本机）；服务分片的计划写台数那么多片（服务片），第 k 片领装置片里片号 mod 台数等于 k 的那几片。
#   本脚本不自己连第二台（不起 multi-host-run.sh --peer）：经 task-server 分片（下面「经 task-server 分片」），各服务片由服务挑机器；
#   片目录在工作根下（NFS 共享根，两台同一个绝对路径），哪一台做完的片都直接落进它，合并照样沿用，不送、不取回。
#   经 task-server 分片（task-server/task_server_lib.py 文件头「分片」；文件头那一行 task-server-shards 声明了，提交 task-server/task submit -- bash scripts/experiment-apparatus/e183-reading-shards.sh）：
#     plan  在本机：编好装置、拷私有副本，把私有副本的绝对路径写进 <计划目录>/binary，plan.json 写台数那么多片（每片只写 label，不钉台：records/2026-10-31-task-server巡检.md 问题 94）；
#           带 --merge-only、--host-share 的用法计划写 0 片，合并那一步照用法跑
#     shard 照 TASK_SERVER_SHARD=<k>/<n> 跑 --host-share k/n，装置用 <计划目录>/binary 那一份（不自己编），建好服务给的产物目录（空的）
#     merge 在本机：片齐了合并出产物（与 --merge-only 同一段）
#   不经服务分片（环境里没有 TASK_SERVER_SHARD_PHASE）：全部片在这一台跑，片齐了合并。
#   各服务片不自己编装置：两台各编一份 sha256 就不同，合并时装置按登记停机。各片用计划那一步拷的私有副本（私有副本在日志目录下，
#   日志目录在工作根下、两台同一份），照同一个绝对路径用同一个文件；程序文件里编进的准入脚本路径（CARGO_MANIFEST_DIR）在两台上也是同一棵树，准入照样现判。
#   一片沿用要两样都对：末行完成标记在，h5_config 行的 binary_sha256 与这一趟的程序文件相同；标记在而 sha256 不同的片（上一趟别的程序文件做的）
#   不沿用，整份移到 <片目录>.void-<sha256 前 12 位>/ 再现跑（登记「那几片作废」，作废的留着不删）。
#   每台的线程是 scripts/hosts/heavy-threads.sh 的 heavy_thread_default（E183_THREADS 设了用它）：这台上同时在跑的片数的上限（装置一片单线程，list-shards 第二列都是 1）；
#   每片不在脚本里套内存包装：整条命令交 task-server（task-server/task submit -- bash scripts/experiment-apparatus/e183-sweep-history-shards.sh …），内存额度按记录由服务给，
#   撞上限由服务放大重排（登记 5.5「每台起的进程数 = 线程数与内存上限能装下的片数两者取小」的内存那一半不再由本脚本管）。
#   一片起一次：退非 0 就记没跑成、退 1，不在脚本里重跑；重起同一条命令只补没做完的片（撞上限、挤掉的重排由 task-server 管，整条命令一起重排）。
#   两台的装置程序文件 sha256 写进每一片（h5_config 行）；合并时与本机的不同 ⇒ 装置报停机、不出产物（登记「两台二进制 sha256 进产物，不同 ⇒ 那几片作废」）。
#   装置在一趟里不许换（欠账 C862，共用段落在 long-run-lib/binary-identity.sh）：开跑时把程序文件拷一份进 $LOG_DIRECTORY/pinned-binary/<sha256>/（只读），
#   写进每一片与合并时核的 sha256 是这份拷贝的，每片每次重跑、列片与合并也都按这份拷贝起，不再 exec 仓根 target/release 下那一份（别的会话的 cargo 随时会把它换掉）；
#   脚本合并之前先自己再核一遍全部片的 h5_config 行（不同就拒、退 1、装置 merge 不起），装置合并时照旧再核一遍。老的拷贝留着不删（一个 sha256 一个目录），确认不再续跑那一版之后手删。
#
# 用法（在仓里任何目录起都行，脚本自己回到仓根）：
#   bash scripts/experiment-apparatus/e183-sweep-history-shards.sh                         经服务交时服务按「经 task-server 分片」拆；直接跑时全部片在这一台跑，片齐了合并出产物
#   bash scripts/experiment-apparatus/e183-sweep-history-shards.sh --host-share <k>/<n>    只跑第 k 份（片号 mod n 等于 k 的片），写进片目录，不合并（服务片用的就是这一种）
#   bash scripts/experiment-apparatus/e183-sweep-history-shards.sh --merge-only            不跑，片齐了合并出产物
#   bash scripts/experiment-apparatus/e183-sweep-history-shards.sh --list-items            逐行列片名与做没做完（要装置已编好，不编译）
#   bash scripts/experiment-apparatus/e183-sweep-history-shards.sh --selftest              假装置、假机器清单、假的 task-server（照分片协议逐步起这一份）走一遍续跑、分台与停机（不起 cargo）
#   --shard-directory <目录> 可与前四种合用，换片目录（相对仓根或绝对路径）。
#   --binary <程序文件> 可与前四种合用，装置用这一份、不编（命令行优先于 E183_BINARY）。接着上一趟续跑时用它钉住那一趟的程序文件：
#     仓根 target/release 下那一份会被别的会话的 cargo 换成另一个编译单元的产物（同一份源码两个 sha256），不钉就可能与已做完的片对不上。
# 环境：E183_BINARY（装置程序文件；没给就先 cargo build --release 编仓根 target/ 下的那一份）、E183_SHARD_DIRECTORY（片目录，默认
#   工作根下的 e183-h5-shards，工作根取 .claude/hooks/lib_work_root.py、两台同一份）、#   E183_THREADS（见「多机的读法」）、
#   E183_LOG_DIRECTORY（每片的标准输出与错误输出，默认工作根下的 e183-h5-shards-logs）、E183_PRODUCT_DIRECTORY（产物目录，默认 research/results）。
#   只给自证换假的：E183_HOSTS_EMITTER（默认 scripts/crash/crash-point-replay-shard-configuration-check.sh）。
# 产物名：…-h5-<日期>.out，同一天再跑加 -r2、-r3；已有的一个不动。合并出的与最新那一份逐字节相同时不另存，只报已在。
# 退出码：0 做完（--host-share 与服务片是这一份的片做完，计划那一步是计划写好）；1 有片没跑成或合并失败（同一条命令重起续跑）；2 用法错（含 TASK_SERVER_SHARD_PHASE 认不出、
#   服务的那一步缺变量）；3 装置报停机（name=stop 那一行交主 agent，不重跑）；78 装置没编好（--list-items）。
#
# 弄坏开关 E183_SHARDS_BREAK=<项>（只在 --selftest 起的那几趟里认：自证设 E183_SHARDS_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   E183_SHARDS_BREAK=done-marker-ignored  片文件在就算做完、不看末行
#       「删一片的完成标记重起只补那一片」
#   E183_SHARDS_BREAK=host-share-ignored  --host-share 不分份、每台跑全部片
#       「两台各跑自己那一份（本机片号 0、2、4，第二台 1、3）」「三台：计划写三片、各领片号 mod 3 的那几片、合并出产物」
#   E183_SHARDS_BREAK=exit-code-ignored  片退非 0 而片文件已写完时仍算做完（只看片文件、不看退出码）
#       「片退非 0 而片文件已写完：仍记没跑成（退 1、不合并），不只看片文件」
#   E183_SHARDS_BREAK=shard-binary-not-from-plan  服务片不读计划目录里私有副本的路径，用那一台自己的那一份（假的第二台给的是它自己编的另一份）
#       「两台各跑自己那一份（本机片号 0、2、4，第二台 1、3）」「第二台用本机那一份私有副本（同一个绝对路径）：五片的 sha256 只有一种、合并出产物」
#       「三台：计划写三片、各领片号 mod 3 的那几片、合并出产物」
#   E183_SHARDS_BREAK=stale-sha-reused  沿用只看完成标记、不看片里的 sha256
#       「别的程序文件做的片不沿用：移到 .void 目录留着、逐片报作废、只现跑那一片、沿用另外四片」「旧装置打的五片全作废：逐片报作废、五片全现跑、旧片整份留在 .void 目录、产物的片都是新装置的」
#   E183_SHARDS_BREAK=exec-root-binary  每片仍按开跑时给的那个路径起、不按私有副本
#       「装置在一趟里不许换：跑着的时候仓根那份被换掉，之后起的五片自报的 sha256 仍是开跑时那份」
#   E183_SHARDS_BREAK=merge-sha-unchecked  合并之前脚本不核全部片的 sha256
#       「合并之前核全部片的装置 sha256：有一片对不上，脚本先拒（退 1、点名那一片、装置 merge 不起、不另出产物）」
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../hosts/heavy-threads.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "$(dirname "${BASH_SOURCE[0]}")/long-run-lib/binary-identity.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; echo "     → 怎么办：在仓里起这份脚本（它自己回到仓根；进不去多半是脚本被挪出了 scripts/experiment-apparatus/）" >&2; exit 2; }

BINARY_DEFAULT="target/release/e183_capacity_read_cache_reserve_and_sweep_costs"
PRODUCT_STEM_NAME="e183-capacity-read-cache-reserve-and-sweep-costs-h5"
# 缺省的片目录与日志目录放工作根（.claude/hooks/lib_work_root.py，两台同一份）下：只在没给那个目录时取（自证的各格都给了），取不到就停，不退回 $HOME 下
work_root_or_stop() {   # <没给的那个变量名>：成了把工作根放进 WORK_ROOT
  [[ -n "${WORK_ROOT:-}" ]] && return 0
  if ! WORK_ROOT="$(python3 .claude/hooks/lib_work_root.py)" || [[ "$WORK_ROOT" != /* ]]; then
    echo "  ✗ 取不到工作根（.claude/hooks/lib_work_root.py 跑不起来），缺省的目录没处放" >&2
    echo "     → 怎么办：修好 .claude/hooks/lib_work_root.py，或设 $1 指到要用的目录" >&2
    exit 2
  fi
}
LOG_DIRECTORY="${E183_LOG_DIRECTORY:-}"
[[ -n "$LOG_DIRECTORY" ]] || { work_root_or_stop E183_LOG_DIRECTORY; LOG_DIRECTORY="$WORK_ROOT/e183-h5-shards-logs"; }
PRODUCT_DIRECTORY="${E183_PRODUCT_DIRECTORY:-research/results}"
HOSTS_EMITTER="${E183_HOSTS_EMITTER:-scripts/crash/crash-point-replay-shard-configuration-check.sh}"

usage_error() {
  echo "  ✗ $1" >&2
  echo "     → 怎么办：见本脚本文件头「用法」" >&2
  exit 2
}

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）；别处设了就拒。
break_is() {
  [[ ",${E183_SHARDS_BREAK:-}," == *",$1,"* ]]
}
if [[ -n "${E183_SHARDS_BREAK:-}" && "${1:-}" != --selftest ]]; then
  if [[ -z "${E183_SHARDS_SELFTEST_MARK:-}" || "$(cat -- "${E183_SHARDS_SELFTEST_MARK}" 2> /dev/null)" != e183-shards-selftest ]]; then
    echo "  ✗ E183_SHARDS_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
    echo "     → 怎么办：去掉 E183_SHARDS_BREAK 再跑；要证红就跑 E183_SHARDS_BREAK=<项> bash scripts/experiment-apparatus/e183-sweep-history-shards.sh --selftest" >&2
    exit 2
  fi
fi

# 一片做完了：文件在，末行是 E7RESULT name=shard_done shard=<片名> lines=<整份行数>。
is_shard_done() {
  local file="$1" name line_count last_line
  [[ -f "$file" ]] || return 1
  break_is done-marker-ignored && return 0
  name=$(basename -- "$file")
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" == "E7RESULT name=shard_done shard=$name lines=$line_count" ]]
}

# 片里 h5_config 行写的装置程序文件 sha256；文件不在或没有这一行打空。
shard_binary_sha256() {
  grep -m1 '^E7RESULT name=h5_config ' -- "$1" 2> /dev/null | grep -o -m1 'binary_sha256=[0-9a-f]*' | cut -d= -f2
}

# 一片能沿用：做完了，且是这一趟的程序文件做的（沿用的键含程序文件的 sha256）。
is_shard_reusable() {
  is_shard_done "$1" || return 1
  break_is stale-sha-reused && return 0
  [[ "$(shard_binary_sha256 "$1")" == "$BINARY_SHA256" ]]
}

# 做完了而 sha256 与这一趟的不同的片：整份移到 <片目录>.void-<sha256 前 12 位>/，留着不删。
set_aside_stale_shard() {
  local name="$1" stale_sha void destination
  stale_sha=$(shard_binary_sha256 "$SHARD_DIRECTORY/$name")
  void="${SHARD_DIRECTORY%/}.void-${stale_sha:0:12}"
  [[ -n "$stale_sha" ]] || void="${SHARD_DIRECTORY%/}.void-no-sha256"
  mkdir -p -- "$void" || exit 1
  destination="$void/$name"
  [[ -e "$destination" ]] && destination="$void/$name.$RUN_STAMP"
  mv -n -- "$SHARD_DIRECTORY/$name" "$destination" || exit 1
  binary_identity_void_line "$name" "$stale_sha" "$destination"
}

# 产物齐了：末行是 E7RESULT name=done emitted=<行数>，且没有停机行。
is_complete_product() {
  local file="$1" line_count last_line
  [[ -f "$file" ]] || return 1
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" == "E7RESULT name=done emitted=$line_count" ]] && ! grep -q '^E7RESULT name=stop ' -- "$file"
}

# 还没被占的产物名：<词干>-<日期>.out，已有就加 -r2、-r3……
next_free_product_path() {
  local today revision stem="$PRODUCT_DIRECTORY/$PRODUCT_STEM_NAME"
  today=$(TZ=Asia/Tokyo date +%F)
  [[ -e "$stem-$today.out" ]] || { echo "$stem-$today.out"; return; }
  revision=2
  while [[ -e "$stem-$today-r$revision.out" ]]; do revision=$((revision + 1)); done
  echo "$stem-$today-r$revision.out"
}

# 这台的线程：E183_THREADS 设了用它，没设取登记的每台默认重活线程数。
thread_count() {
  if [[ -n "${E183_THREADS:-}" ]]; then
    [[ "$E183_THREADS" =~ ^[1-9][0-9]*$ ]] || usage_error "E183_THREADS 要是正整数，给的是「$E183_THREADS」"
    echo "$E183_THREADS"
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
    echo "     → 怎么办：照它打的原因修 multi-host.env（单跑 bash $HOSTS_EMITTER 看原因）" >&2
    return 1
  fi
  count=$(grep -m1 '^MULTI_HOST_COUNT=' <<< "$emitted" | cut -d= -f2)
  [[ "$count" =~ ^[0-9]+$ ]] || { echo "  ✗ --emit-hosts 没打 MULTI_HOST_COUNT" >&2; echo "     → 怎么办：单跑 bash $HOSTS_EMITTER --emit-hosts 看它打了什么；双机配置在仓根 multi-host.env（模板 multi-host.env.example）" >&2; return 1; }
  echo $((1 + count))
}

# 装置：E183_BINARY 给了用它（不编），没给就先编仓根 target/ 下那一份。BINARY_SHA256 是程序文件的 sha256，写进每一片、合并时比。
prepare_binary() {
  if [[ -n "$BINARY_OPTION" ]]; then
    BINARY="$BINARY_OPTION"
  elif [[ -n "${E183_BINARY:-}" ]]; then
    BINARY="$E183_BINARY"
  else
    BINARY="$BINARY_DEFAULT"
    bash scripts/hosts/capped.sh "$THREADS" cargo build --release -p singlefs-verification --bin e183_capacity_read_cache_reserve_and_sweep_costs \
      > "$LOG_DIRECTORY/build-$RUN_STAMP.log" 2>&1 || {
      echo "  ✗ 装置编不过（$LOG_DIRECTORY/build-$RUN_STAMP.log）" >&2
      echo "     → 怎么办：照 cargo 打的第一处 error 改装置，再起本脚本" >&2
      exit 1
    }
  fi
  [[ -x "$BINARY" ]] || { echo "  ✗ 装置程序文件不在或不可执行（$BINARY）" >&2; echo "     → 怎么办：装置没编好就先 cargo build --release，或设 E183_BINARY 指到编好的一份" >&2; exit 1; }
  # 开跑时拷一份私有副本：之后列片、起每一片、合并都按这份拷贝，BINARY_SHA256 是拷贝的 sha256
  SOURCE_BINARY="$BINARY"
  binary_identity_pin "$SOURCE_BINARY" "$LOG_DIRECTORY/pinned-binary" || exit 1
  BINARY="$PINNED_BINARY"
  BINARY_SHA256="$PINNED_BINARY_SHA256"
  LAUNCH_BINARY="$BINARY"
  break_is exec-root-binary && LAUNCH_BINARY="$SOURCE_BINARY"
}

# 片的清单：一行「片名 TAB 占的线程数」，次序是装置 h5 list-shards 的次序（也是合并的次序）。
load_shard_list() {
  mapfile -t SHARD_LINES < <("$BINARY" h5 list-shards)
  ((${#SHARD_LINES[@]} > 0)) || { echo "  ✗ 装置 h5 list-shards 什么都没列（$BINARY）" >&2; echo "     → 怎么办：单跑 $BINARY h5 list-shards 看装置为什么没列片" >&2; exit 1; }
}

# 跑点名的那几片（SHARD_LINES 的下标，按给的次序起）：这台上在跑的片占的线程合计不超过 THREADS，每片的退出码写进这一趟的 jobs 目录。
# 交回时设 RAN_NOW（这一趟跑完、片齐了的个数）、STOPPED（装置报停机的片）、FAILED（别的没跑成的片）。
run_shards() {
  local jobs="$LOG_DIRECTORY/jobs-$RUN_STAMP" index name threads busy running exit_code job_exit
  local -A threads_of=()
  local launched=()
  mkdir -p -- "$jobs"
  for index in "$@"; do
    IFS=$'\t' read -r name threads <<< "${SHARD_LINES[$index]}"
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
    { nice -n 19 "$LAUNCH_BINARY" h5 shard "$name" --shard-output "$SHARD_DIRECTORY/$name" \
        --binary-sha256 "$BINARY_SHA256" > "$LOG_DIRECTORY/$name-$RUN_STAMP.out" 2> "$LOG_DIRECTORY/$name-$RUN_STAMP.err"
      job_exit=$?
      echo "$job_exit" > "$jobs/$name.exit"; } &
    launched+=("$name")
  done
  wait  # shell-lint:exit-collected 每片的退出码写进 $jobs/<片名>.exit，下面按派活的次序逐个读，并核收回的个数等于派出的个数
  RAN_NOW=0; STOPPED=(); FAILED=()
  for name in ${launched[@]+"${launched[@]}"}; do
    exit_code=$(cat -- "$jobs/$name.exit" 2> /dev/null || echo missing)
    if { [[ "$exit_code" == 0 ]] || break_is exit-code-ignored; } && is_shard_done "$SHARD_DIRECTORY/$name"; then
      RAN_NOW=$((RAN_NOW + 1))
      echo "片 $name：做完（$(grep -m1 '^E7RESULT name=shard_written ' "$LOG_DIRECTORY/$name-$RUN_STAMP.out" || echo '没有 shard_written 行')）"
    elif [[ "$exit_code" == 3 ]]; then
      STOPPED+=("$name")
      echo "  ✗ 片 $name：装置报停机：$(grep -m1 '^E7RESULT name=stop ' "$LOG_DIRECTORY/$name-$RUN_STAMP.out")" >&2
      echo "     → 怎么办：该停下，把这一行 name=stop 交回主 agent，不重跑" >&2
    else
      FAILED+=("$name")
      echo "  ✗ 片 $name：退 $exit_code，没做完（错误输出 $LOG_DIRECTORY/$name-$RUN_STAMP.err；内存不够的撞顶、重排与放大由 task-server 管，整条命令的结局看 task-server/task status）" >&2
      echo "     → 怎么办：看那份错误输出；同一条命令重起只补没做完的片" >&2
    fi
  done
  local finished_count
  finished_count=$(find "$jobs" -maxdepth 1 -name '*.exit' | wc -l)
  if ((finished_count != ${#launched[@]})); then
    echo "  ✗ 起了 ${#launched[@]} 片，收回 $finished_count 个退出码，对不上：这一趟作废" >&2
    echo "     → 怎么办：看 $jobs 下少了哪几片的 .exit，同一条命令重起（片目录里做完的片照样沿用）" >&2
    exit 1
  fi
}

# 服务分片的计划（文件头「经 task-server 分片」plan）：<片数>：往计划目录写 plan.json，每片只写 label、不钉台（先写临时文件再改名）
write_plan() {
  python3 - "$plan_directory" "$1" <<'PLAN_PYTHON' || { echo "  ✗ 计划目录 $plan_directory 写不进 plan.json（上面是 python 的报错）" >&2; echo "     → 怎么办：看那个目录可不可写" >&2; return 1; }
import json, os, sys
directory, count = sys.argv[1], int(sys.argv[2])
shards = [{"label": f"--host-share {index}/{count}（片号 mod {count} 等于 {index} 的那几片）"} for index in range(count)]
temporary = os.path.join(directory, "plan.json.partial")
with open(temporary, "w", encoding="utf-8") as handle:
    json.dump({"shards": shards, "note": "e183 第三段分片（e183-sweep-history-shards.sh 文件头「经 task-server 分片」）"}, handle, ensure_ascii=False)
os.replace(temporary, os.path.join(directory, "plan.json"))
PLAN_PYTHON
}

# 片齐了合并出产物（与最新那一份逐字节相同就不另存）。装置报停机（退 3：缺片、程序文件对不上、段首重跑与原片不同）照退 3。
merge_product() {
  local temporary="$LOG_DIRECTORY/merge-$RUN_STAMP.out" newest destination merge_exit checked_pairs=() checked_line checked_name
  # 合并之前脚本先自己再核一遍：每一片 h5_config 行记的装置 sha256 都得是这一趟的（装置合并时照旧再核）
  if ! break_is merge-sha-unchecked; then
    for checked_line in "${SHARD_LINES[@]}"; do
      checked_name="${checked_line%%$'\t'*}"
      checked_pairs+=("$checked_name" "$(shard_binary_sha256 "$SHARD_DIRECTORY/$checked_name")")
    done
    binary_identity_check_same "$BINARY_SHA256" "${checked_pairs[@]}" || exit 1
  fi
  "$BINARY" h5 merge "$SHARD_DIRECTORY" --binary-sha256 "$BINARY_SHA256" \
    > "$temporary" 2> "$LOG_DIRECTORY/merge-$RUN_STAMP.err"
  merge_exit=$?
  if ((merge_exit == 3)); then
    echo "  ✗ 合并时装置报停机（$temporary）：$(grep -m1 '^E7RESULT name=stop ' "$temporary")" >&2
    echo "     → 怎么办：停机行点名的片与原因交主 agent（登记第十一节），产物不入库、不重跑" >&2
    exit 3
  fi
  if ! is_complete_product "$temporary"; then
    echo "  ✗ 合并没出齐（$temporary，退 $merge_exit）：$(tail -n 1 -- "$temporary")" >&2
    echo "     → 怎么办：看 $LOG_DIRECTORY/merge-$RUN_STAMP.err；产物不入库" >&2
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
  [[ -e "$temporary" ]] && { echo "  ✗ 产物名 $destination 被占了，没挪进去（合并出的留在 $temporary）" >&2; echo "     → 怎么办：看谁占了 $destination（别的会话同时在合并？），确认后手动把 $temporary 挪成新名字" >&2; exit 1; }
  echo "产物：$destination（$(wc -l < "$destination") 行，sha256 $(sha256sum -- "$destination" | cut -d' ' -f1)）"
}

# 自证：假装置（五片）、假机器清单、假的 task-server（run_phases：照分片协议起计划、各服务片、合并，片目录是同一个；
# 第 0 片当本机、别的当第二台，第二台那几片的环境里 E183_BINARY 是它自己编的另一份），每一格在自己的临时目录里从头搭，格与格不共用状态。
run_selftest() {
  local work failures=0 fakes
  work=$(mktemp -d) || return 1
  fakes="$work/fakes"
  mkdir -p -- "$fakes"
  echo e183-shards-selftest > "$work/mark"
  cat > "$fakes/device" <<'FAKE'
#!/usr/bin/env bash
# E183_SELFTEST_SWAP_TARGET：列片的时候（开跑之后、起第一片之前）把这个路径的装置换成另一份字节不同的，像别的会话的 cargo 把仓根那份换掉。
if [[ -n "${E183_SELFTEST_SWAP_TARGET:-}" && "$1 $2" == "h5 list-shards" && ! -e "$E183_SELFTEST_SWAP_TARGET.swapped" ]]; then
  { cat -- "$E183_SELFTEST_SWAP_TARGET"; echo "# swapped by a build while the long run is going"; } > "$E183_SELFTEST_SWAP_TARGET.swap" \
    && chmod +x "$E183_SELFTEST_SWAP_TARGET.swap" && mv -f -- "$E183_SELFTEST_SWAP_TARGET.swap" "$E183_SELFTEST_SWAP_TARGET" && : > "$E183_SELFTEST_SWAP_TARGET.swapped"
fi
case "$1 $2" in
  "h5 list-shards") printf '%s\t%s\n' h5-a 1 h5-b 1 h5-c 1 h5-d 1 h5-e 1 ;;
  "h5 shard")
    echo "${E183_SELFTEST_HOST_TAG:-local} $3" >> "$E183_SELFTEST_CALLS"
    echo "$3 $(sha256sum -- "$0" | cut -d' ' -f1)" >> "$E183_SELFTEST_CALLS.self-sha"
    if [[ "$3" == "${E183_SELFTEST_FLAKY_SHARD:-}" ]] && (($(grep -c -x -- "${E183_SELFTEST_HOST_TAG:-local} $3" "$E183_SELFTEST_CALLS") <= ${E183_SELFTEST_FLAKY_TIMES:-0})); then
      exit "${E183_SELFTEST_FLAKY_EXIT:-252}"
    fi
    if [[ "$3" == "${E183_SELFTEST_STOP_SHARD:-}" ]]; then echo "E7RESULT name=stop shard=$3 reason=selftest_stop"; exit 3; fi
    printf 'E7RESULT name=h5_config shard=%s binary_sha256=%s\nE7RESULT name=shard_done shard=%s lines=2\n' "$3" "$7" "$3" > "$5.partial-$$" && mv -- "$5.partial-$$" "$5"
    echo "E7RESULT name=shard_written shard=$3 lines=2"
    [[ "$3" == "${E183_SELFTEST_FAIL_AFTER_WRITE_SHARD:-}" ]] && exit 1
    exit 0 ;;
  "h5 merge")
    for shard in h5-a h5-b h5-c h5-d h5-e; do
      [[ -f "$3/$shard" ]] || { echo "E7RESULT name=stop reason=shard_missing_or_unfinished shard=$shard"; echo "E7RESULT name=done emitted=2"; exit 3; }
      shard_sha=$(grep -m1 '^E7RESULT name=h5_config ' "$3/$shard" | grep -o 'binary_sha256=[0-9a-f]*' | cut -d= -f2)
      [[ "$shard_sha" == "$5" ]] || { echo "E7RESULT name=stop reason=binary_sha256_differs shard=$shard shard_binary_sha256=$shard_sha"; exit 3; }
    done
    printf 'E7RESULT name=merged shards=5\nE7RESULT name=done emitted=2\n' ;;
esac
FAKE
  cat > "$fakes/hosts" <<'FAKE'
#!/usr/bin/env bash
case "$E183_SELFTEST_HOSTS" in
  1) echo "双机：关"; exit 3 ;;
  *) echo "MULTI_HOST_COUNT=$((E183_SELFTEST_HOSTS - 1))"; exit 0 ;;
esac
FAKE
  # 第二台自己编的那一份：同一份源码、字节不同（编进去的树路径不同）
  { cat "$fakes/device"; echo '# built on the peer in another tree'; } > "$fakes/device-built-on-peer"
  chmod +x "$fakes"/*
  # 一格的环境：片目录、日志、产物都在这一格自己的目录里。
  invoke() {
    local cell="$1" hosts="$2"
    shift 2
    [[ -n "${INVOKE_APPEND:-}" ]] || : > "$cell/out"
    env -u TASK_SERVER_SHARD_PHASE -u TASK_SERVER_SHARD -u TASK_SERVER_SHARD_PLAN_DIRECTORY -u TASK_SERVER_SHARD_OUTPUT_DIRECTORY -u TASK_SERVER_SHARD_OUTPUTS -u TASK_SERVER_SHARD_HOST \
      E183_BINARY="${INVOKE_BINARY:-$fakes/device}" E183_HOSTS_EMITTER="$fakes/hosts" \
      E183_SELFTEST_HOSTS="$hosts" E183_SELFTEST_CALLS="$cell/calls" E183_THREADS=3 E183_SHARDS_SELFTEST_MARK="$work/mark" \
      E183_SHARD_DIRECTORY="$cell/shards" E183_LOG_DIRECTORY="$cell/logs" E183_PRODUCT_DIRECTORY="$cell/product" ${INVOKE_ENVIRONMENT[@]+"${INVOKE_ENVIRONMENT[@]}"} \
      bash "$SCRIPT_PATH" "$@" >> "$cell/out" 2>&1
  }
  # invoke_phase <格目录> <台数> <环境 键=值…> -- [参数…]：照服务的一步起（环境放进 INVOKE_ENVIRONMENT，输出接在 $cell/out 后面）；退那一步的码
  invoke_phase() {
    local cell="$1" hosts="$2" code=0
    shift 2
    INVOKE_ENVIRONMENT=()
    while (($# > 0)) && [[ "$1" != -- ]]; do INVOKE_ENVIRONMENT+=("$1"); shift; done
    shift
    INVOKE_APPEND=1 invoke "$cell" "$hosts" "$@" || code=$?
    INVOKE_ENVIRONMENT=()
    return "$code"
  }
  # 假的 task-server（文件头「经 task-server 分片」）：<格目录> <台数> [参数…]：计划 → 读 plan.json 照片数起各服务片（第 0 片本机、别的当第二台：环境里 E183_BINARY 换成
  #   第二台自己编的那一份、调用记录标 peer）→ 各片都退 0 才起合并。phase_exits 是「plan=<码> shard-0=<码> … merge=<码>」；plan_keys 是各片的键（排好、逗号连）
  run_phases() {
    local cell="$1" hosts="$2" count index tag step_exit failed=0
    shift 2
    phase_exits="" plan_keys=""
    mkdir -p -- "$cell/plan"
    : > "$cell/out"
    step_exit=0; invoke_phase "$cell" "$hosts" TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" -- "$@" || step_exit=$?
    phase_exits="plan=$step_exit"
    ((step_exit == 0)) || return 0
    read -r count plan_keys <<< "$(python3 -c 'import json, sys
plan = json.load(open(sys.argv[1], encoding="utf-8"))
print(len(plan["shards"]), "|".join(",".join(sorted(shard)) for shard in plan["shards"]) or "-")' "$cell/plan/plan.json" 2>/dev/null)"
    [[ "$count" =~ ^[0-9]+$ ]] || { phase_exits+=" plan-unreadable"; return 0; }
    : > "$cell/outputs.tsv"
    for ((index = 0; index < count; index++)); do
      tag=peer
      ((index == 0)) && tag=local
      step_exit=0
      if [[ "$tag" == peer ]]; then
        INVOKE_BINARY="$fakes/device-built-on-peer" invoke_phase "$cell" "$hosts" TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD="$index/$count" TASK_SERVER_SHARD_HOST=192.0.2.2 \
          E183_SELFTEST_HOST_TAG=peer TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$cell/output-$index" -- "$@" || step_exit=$?
      else
        invoke_phase "$cell" "$hosts" TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD="$index/$count" TASK_SERVER_SHARD_HOST=192.0.2.1 \
          TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$cell/output-$index" -- "$@" || step_exit=$?
      fi
      phase_exits+=" shard-$index=$step_exit"
      [[ -d "$cell/output-$index" ]] || phase_exits+="(没建产物目录)"
      printf '%s\t%s\t%s\n' "$index" "$tag" "$cell/output-$index" >> "$cell/outputs.tsv"
      ((step_exit == 0)) || failed=1
    done
    ((failed == 0)) || return 0
    step_exit=0; invoke_phase "$cell" "$hosts" TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" TASK_SERVER_SHARD_OUTPUTS="$cell/outputs.tsv" -- "$@" || step_exit=$?
    phase_exits+=" merge=$step_exit"
  }
  expect() {
    local name="$1" condition="$2"
    if eval "$condition"; then echo "  ✓ $name"; else echo "  ✗ $name：条件「$condition」不成立"; echo "     → 怎么办：按这一格的名字找对应的续跑、分台或合并逻辑改，或看 E183_SHARDS_BREAK 是不是设着"; failures=$((failures + 1)); fi
  }
  local cell exit_code first_exit old_sha original_sha phase_exits="" plan_keys=""
  local -a INVOKE_ENVIRONMENT=()
  cell="$work/single"; mkdir -p -- "$cell"
  invoke "$cell" 1; exit_code=$?
  expect "单机全新：五片都现跑、合并出产物" '((exit_code == 0)) && grep -q "现跑 5、沿用 0" "$cell/out" && (($(wc -l < "$cell/calls") == 5)) && (($(find "$cell/product" -name "e183-capacity-read-cache-reserve-and-sweep-costs-h5-*.out" | wc -l) == 1))'
  : > "$cell/out"; invoke "$cell" 1; exit_code=$?
  expect "原样重起：五片都沿用、产物不另存" '((exit_code == 0)) && grep -q "现跑 0、沿用 5" "$cell/out" && (($(wc -l < "$cell/calls") == 5)) && grep -q "不另存" "$cell/out"'
  head -n 1 -- "$cell/shards/h5-b" > "$cell/shards/h5-b.cut" && mv -- "$cell/shards/h5-b.cut" "$cell/shards/h5-b"
  : > "$cell/out"; invoke "$cell" 1; exit_code=$?
  expect "删一片的完成标记重起只补那一片" '((exit_code == 0)) && grep -q "现跑 1、沿用 4" "$cell/out" && [[ "$(tail -n 1 -- "$cell/calls")" == "local h5-b" ]] && (($(wc -l < "$cell/calls") == 6))'
  cell="$work/two"; mkdir -p -- "$cell"
  run_phases "$cell" 2
  expect "两台各跑自己那一份（本机片号 0、2、4，第二台 1、3）" '[[ "$phase_exits" == "plan=0 shard-0=0 shard-1=0 merge=0" ]] && [[ "$(grep "^local " "$cell/calls" | sort | tr "\n" " ")" == "local h5-a local h5-c local h5-e " ]] && [[ "$(grep "^peer " "$cell/calls" | sort | tr "\n" " ")" == "peer h5-b peer h5-d " ]]'
  expect "第二台用本机那一份私有副本（同一个绝对路径）：五片的 sha256 只有一种、合并出产物" '[[ "$phase_exits" == "plan=0 shard-0=0 shard-1=0 merge=0" ]] && (($(cat "$cell"/shards/h5-* | grep -o "binary_sha256=[0-9a-f]*" | sort -u | wc -l) == 1)) && (($(find "$cell/product" -name "e183-capacity-read-cache-reserve-and-sweep-costs-h5-*.out" | wc -l) == 1))'
  expect "计划不钉台：两片都只写 label、私有副本的路径写进计划目录" '[[ "$plan_keys" == "label|label" ]] && [[ -x "$(head -n 1 "$cell/plan/binary" 2>/dev/null)" ]]'
  cell="$work/three"; mkdir -p -- "$cell"
  run_phases "$cell" 3
  expect "三台：计划写三片、各领片号 mod 3 的那几片、合并出产物" '[[ "$phase_exits" == "plan=0 shard-0=0 shard-1=0 shard-2=0 merge=0" ]] && (($(wc -l < "$cell/calls") == 5)) && [[ "$(grep -c "^local " "$cell/calls")" == 2 ]] && (($(find "$cell/product" -name "e183-capacity-read-cache-reserve-and-sweep-costs-h5-*.out" | wc -l) == 1))'
  cell="$work/direct-two"; mkdir -p -- "$cell"
  invoke "$cell" 2; exit_code=$?
  expect "不经服务分片直接跑：多机配置给两台也全部片在这一台跑，不连第二台" '((exit_code == 0)) && [[ "$(grep -c "^local " "$cell/calls")" == 5 ]] && ! grep -q "^peer " "$cell/calls"'
  cell="$work/merge-only-phases"; mkdir -p -- "$cell"
  invoke "$cell" 1; first_exit=$?
  run_phases "$cell" 2 --merge-only
  expect "带 --merge-only 经服务交：计划写 0 片、合并那一步照用法合并" '((first_exit == 0)) && [[ "$phase_exits" == "plan=0 merge=0" ]] && grep -q "这个用法不分片（0 片）" "$cell/out" && grep -q "不另存" "$cell/out"'
  cell="$work/stop"; mkdir -p -- "$cell"
  E183_SELFTEST_STOP_SHARD=h5-d invoke "$cell" 1; exit_code=$?
  expect "装置报停机：退 3、那一片没有片文件、别的片做完" '((exit_code == 3)) && [[ ! -e "$cell/shards/h5-d" ]] && (($(find "$cell/shards" -name "h5-*" | wc -l) == 4))'
  cell="$work/shard-exit"; mkdir -p -- "$cell"
  E183_SELFTEST_FLAKY_SHARD=h5-b E183_SELFTEST_FLAKY_EXIT=252 E183_SELFTEST_FLAKY_TIMES=1 invoke "$cell" 1; exit_code=$?
  expect "片退非 0 记没跑成、不在脚本里重跑：h5-b 只起一次、退 1、不合并；别的片照做" '((exit_code == 1)) && (($(grep -c -x "local h5-b" "$cell/calls") == 1)) && [[ ! -e "$cell/shards/h5-b" ]] && [[ -f "$cell/shards/h5-c" ]] && [[ ! -d "$cell/product" ]]'
  invoke "$cell" 1; exit_code=$?
  expect "同一条命令重起：只补 h5-b、合并出产物" '((exit_code == 0)) && grep -q "现跑 1、沿用 4" "$cell/out"'
  cell="$work/exit-after-write"; mkdir -p -- "$cell"
  E183_SELFTEST_FAIL_AFTER_WRITE_SHARD=h5-b invoke "$cell" 1; exit_code=$?
  expect "片退非 0 而片文件已写完：仍记没跑成（退 1、不合并），不只看片文件" '((exit_code == 1)) && (($(grep -c -x "local h5-b" "$cell/calls") == 1)) && grep -q "片 h5-b：退 1，没做完" "$cell/out" && [[ ! -d "$cell/product" ]]'
  cell="$work/usage"; mkdir -p -- "$cell"
  invoke "$cell" 1 --host-share 2/2; exit_code=$?
  expect "第几份不小于共几份：用法错退 2" '((exit_code == 2))'
  cell="$work/merge-stop"; mkdir -p -- "$cell"
  invoke "$cell" 1; exit_code=$?
  sed "s/binary_sha256=[0-9a-f]*/binary_sha256=$(printf 'f%.0s' {1..64})/" "$cell/shards/h5-c" > "$cell/shards/h5-c.other" && mv -- "$cell/shards/h5-c.other" "$cell/shards/h5-c"
  invoke "$cell" 1 --merge-only; exit_code=$?
  expect "合并之前核全部片的装置 sha256：有一片对不上，脚本先拒（退 1、点名那一片、装置 merge 不起、不另出产物）" '((exit_code == 1)) && grep -q "片 h5-c 的装置 sha256 对不上" "$cell/out" && grep -q "合并产物之前核全部片的装置 sha256" "$cell/out" && ! grep -q "binary_sha256_differs" "$cell/out" && (($(find "$cell/product" -name "e183-capacity-read-cache-reserve-and-sweep-costs-h5-*.out" | wc -l) == 1))'
  cell="$work/stale-sha"; mkdir -p -- "$cell"
  invoke "$cell" 1; exit_code=$?
  sed "s/binary_sha256=[0-9a-f]*/binary_sha256=$(printf 'f%.0s' {1..64})/" "$cell/shards/h5-b" > "$cell/shards/h5-b.other" && mv -- "$cell/shards/h5-b.other" "$cell/shards/h5-b"
  invoke "$cell" 1; exit_code=$?
  expect "别的程序文件做的片不沿用：移到 .void 目录留着、逐片报作废、只现跑那一片、沿用另外四片" '((exit_code == 0)) && grep -q "现跑 1、沿用 4" "$cell/out" && grep -q "^作废：h5-b（装置 sha256 不同）" "$cell/out" && [[ -f "$cell/shards.void-ffffffffffff/h5-b" ]] && [[ "$(tail -n 1 -- "$cell/calls")" == "local h5-b" ]] && (($(wc -l < "$cell/calls") == 6))'
  cell="$work/old-binary"; mkdir -p -- "$cell/bin"; cp -- "$fakes/device" "$cell/bin/device"
  INVOKE_BINARY="$cell/bin/device" invoke "$cell" 1; first_exit=$?
  old_sha=$(sha256sum -- "$cell/bin/device" | cut -d' ' -f1)
  echo "# a newer build of the device" >> "$cell/bin/device"
  INVOKE_BINARY="$cell/bin/device" invoke "$cell" 1; exit_code=$?
  expect "旧装置打的五片全作废：逐片报作废、五片全现跑、旧片整份留在 .void 目录、产物的片都是新装置的" '((first_exit == 0 && exit_code == 0)) && grep -q "现跑 5、沿用 0" "$cell/out" && (($(grep -c "^作废：h5-.（装置 sha256 不同）" "$cell/out") == 5)) && (($(find "$cell/shards.void-${old_sha:0:12}" -name "h5-*" | wc -l) == 5)) && (($(cat "$cell"/shards/h5-* | grep -o "binary_sha256=[0-9a-f]*" | sort -u | wc -l) == 1)) && ! grep -q "$old_sha" "$cell"/shards/h5-*'
  cell="$work/swap"; mkdir -p -- "$cell/bin"; cp -- "$fakes/device" "$cell/bin/device"
  original_sha=$(sha256sum -- "$cell/bin/device" | cut -d' ' -f1)
  E183_SELFTEST_SWAP_TARGET="$cell/bin/device" INVOKE_BINARY="$cell/bin/device" invoke "$cell" 1; exit_code=$?
  expect "装置在一趟里不许换：跑着的时候仓根那份被换掉，之后起的五片自报的 sha256 仍是开跑时那份" '((exit_code == 0)) && [[ -e "$cell/bin/device.swapped" ]] && [[ "$(sha256sum -- "$cell/bin/device" | cut -d" " -f1)" != "$original_sha" ]] && (($(wc -l < "$cell/calls.self-sha") == 5)) && [[ "$(awk "{print \$2}" "$cell/calls.self-sha" | sort -u)" == "$original_sha" ]] && grep -q "binary_sha256=$original_sha" "$cell/shards/h5-d"'
  rm -rf -- "$work"
  if ((failures > 0)); then echo "e183-sweep-history-shards.sh 自证：$failures 格红"; return 1; fi
  echo "e183-sweep-history-shards.sh 自证：18 格全绿"
  return 0
}

mode=run
share=""
SHARD_DIRECTORY="${E183_SHARD_DIRECTORY:-}"
[[ -n "$SHARD_DIRECTORY" ]] || { work_root_or_stop E183_SHARD_DIRECTORY; SHARD_DIRECTORY="$WORK_ROOT/e183-h5-shards"; }
BINARY_OPTION=""
while (($# > 0)); do
  case "$1" in
    --binary) (($# >= 2)) || usage_error "--binary 后面要写装置程序文件"; BINARY_OPTION="$2"; shift 2 ;;
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

if [[ "$mode" == selftest ]]; then
  run_selftest
  exit $?
fi

RUN_STAMP="$(TZ=Asia/Tokyo date +%F)-$$"
THREADS=$(thread_count) || { echo "  ✗ 取不到这台的线程数（原因见上一行）" >&2; echo "     → 怎么办：设 E183_THREADS，或修好 .claude/heavy-thread-default" >&2; exit 2; }
mkdir -p -- "$LOG_DIRECTORY" "$SHARD_DIRECTORY" || exit 1

if [[ "$mode" == list ]]; then
  BINARY="${BINARY_OPTION:-${E183_BINARY:-$BINARY_DEFAULT}}"
  [[ -x "$BINARY" ]] || { echo "  ✗ 装置还没编好：$BINARY" >&2; echo "     → 怎么办：先不带 --list-items 跑一次本脚本，或 cargo build --release -p singlefs-verification --bin e183_capacity_read_cache_reserve_and_sweep_costs" >&2; exit 78; }
  load_shard_list
  BINARY_SHA256=$(binary_identity_sha256_of "$BINARY") || { echo "  ✗ 算不出装置程序文件的 sha256（$BINARY）" >&2; echo "     → 怎么办：核这个文件在、当前用户读得了；装置没编好就先 cargo build --release，或设 E183_BINARY 指到编好的一份" >&2; exit 1; }
  for line in "${SHARD_LINES[@]}"; do
    name="${line%%$'\t'*}"
    if is_shard_reusable "$SHARD_DIRECTORY/$name"; then echo "$name 做完"
    elif is_shard_done "$SHARD_DIRECTORY/$name"; then echo "$name 做完（装置 sha256 与现在这份不同，续跑时作废）"
    else echo "$name 没做"; fi
  done
  exit 0
fi

# 经 task-server 分片（文件头「经 task-server 分片」）：服务起的那一步照 TASK_SERVER_SHARD_PHASE 换用法
task_server_phase="${TASK_SERVER_SHARD_PHASE:-}"
plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}"
case "$task_server_phase" in
  "") ;;
  plan|shard|merge)
    [[ -n "$plan_directory" && -d "$plan_directory" ]] || { echo "  ✗ 服务的 $task_server_phase 那一步没给在的计划目录（TASK_SERVER_SHARD_PLAN_DIRECTORY=${plan_directory:-没设}）" >&2; echo "     → 怎么办：这一步只由 task-server 起；直接跑就去掉 TASK_SERVER_SHARD_PHASE" >&2; exit 2; } ;;
  *) echo "  ✗ TASK_SERVER_SHARD_PHASE=$task_server_phase 认不出（认 plan、shard、merge）" >&2; echo "     → 怎么办：这个变量只由 task-server 设；直接跑就去掉它" >&2; exit 2 ;;
esac
if [[ "$task_server_phase" == plan ]]; then
  if [[ "$mode" != run ]]; then
    # 不带参数之外的用法（--merge-only、--host-share）不分：计划写 0 片，合并那一步照用法跑
    write_plan 0 || exit 1
    echo "计划：这个用法不分片（0 片），合并那一步照它跑"
    exit 0
  fi
  prepare_binary
  load_shard_list
  hosts=$(host_count) || exit 1
  printf '%s\n' "$BINARY" > "$plan_directory/binary" || { echo "  ✗ 计划目录 $plan_directory 写不进私有副本的路径" >&2; echo "     → 怎么办：看那个目录可不可写" >&2; exit 1; }
  write_plan "$hosts" || exit 1
  echo "计划：$hosts 片（各片领片号 mod $hosts 的那几片，不钉台、由服务挑机器），私有副本 $BINARY（sha256 $BINARY_SHA256），共 ${#SHARD_LINES[@]} 片装置片"
  exit 0
fi
if [[ "$task_server_phase" == shard ]]; then
  [[ "${TASK_SERVER_SHARD:-}" =~ ^[0-9]+/[1-9][0-9]*$ ]] || { echo "  ✗ 服务的 shard 那一步没给片号（TASK_SERVER_SHARD=${TASK_SERVER_SHARD:-没设}）" >&2; echo "     → 怎么办：这一步只由 task-server 起" >&2; exit 2; }
  [[ -z "${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}" ]] || mkdir -p -- "$TASK_SERVER_SHARD_OUTPUT_DIRECTORY" || exit 1
  mode=share share="$TASK_SERVER_SHARD"
  # 各片用计划那一步拷好的私有副本（两台同一个绝对路径），不自己编；弄坏开关 shard-binary-not-from-plan 下不读计划里的路径
  if ! break_is shard-binary-not-from-plan; then
    BINARY_OPTION="$(head -n 1 -- "$plan_directory/binary" 2>/dev/null)"
    [[ -n "$BINARY_OPTION" ]] || { echo "  ✗ 计划目录里没有私有副本的路径（$plan_directory/binary）" >&2; echo "     → 怎么办：计划那一步没写全：看计划那一步的日志，重交这一件" >&2; exit 1; }
  fi
fi
if [[ "$task_server_phase" == merge ]]; then
  if [[ "$mode" != run ]]; then
    task_server_phase=""   # 计划写了 0 片的用法：合并那一步照用法跑
  else
    mode=merge
    BINARY_OPTION="$(head -n 1 -- "$plan_directory/binary" 2>/dev/null)"
    [[ -n "$BINARY_OPTION" ]] || { echo "  ✗ 计划目录里没有私有副本的路径（$plan_directory/binary）" >&2; echo "     → 怎么办：计划那一步没写全：看计划那一步的日志，重交这一件" >&2; exit 1; }
  fi
fi

prepare_binary
load_shard_list
echo "e183 第三段分片跑法：这台线程 $THREADS、片目录 $SHARD_DIRECTORY、共 ${#SHARD_LINES[@]} 片、装置 sha256 $BINARY_SHA256"

if [[ "$mode" == merge ]]; then
  for line in "${SHARD_LINES[@]}"; do
    name="${line%%$'\t'*}"
    is_shard_done "$SHARD_DIRECTORY/$name" || { echo "  ✗ 片还没齐：缺 $name" >&2; echo "     → 怎么办：先不带 --merge-only 跑一趟把缺的片做完，再合并" >&2; exit 1; }
  done
  merge_product
  echo "片齐了：合并出产物（共 ${#SHARD_LINES[@]} 片）"
  exit 0
fi

# 这台要跑的片：分份时片号 mod 共几份等于第几份；直接跑（不经服务分片）时全部片都在这一台跑。做完的沿用。
this_share=0; share_count=1
if [[ "$mode" == share ]]; then this_share=${share%/*}; share_count=${share#*/}; fi
break_is host-share-ignored && share_count=1 && this_share=0
selected=(); reused=0
for index in "${!SHARD_LINES[@]}"; do
  name="${SHARD_LINES[$index]%%$'\t'*}"
  ((index % share_count == this_share)) || continue
  if is_shard_reusable "$SHARD_DIRECTORY/$name"; then
    reused=$((reused + 1))
    continue
  fi
  # 做完了而是别的程序文件做的：移开、现跑
  is_shard_done "$SHARD_DIRECTORY/$name" && set_aside_stale_shard "$name"
  selected+=("$index")
done
run_shards ${selected[@]+"${selected[@]}"}
if ((${#STOPPED[@]} > 0)); then
  echo "  ✗ ${#STOPPED[@]} 片装置报停机（${STOPPED[*]}）" >&2
  echo "     → 怎么办：该停下，把各片 .out 里 name=stop 那一行交回主 agent，不重跑" >&2
  exit 3
fi
((${#FAILED[@]} == 0)) || { echo "  ✗ ${#FAILED[@]} 片没做完（${FAILED[*]}）" >&2; echo "     → 怎么办：看各片的错误输出；同一条命令重起只补没做完的" >&2; exit 1; }
if [[ "$mode" == share ]]; then
  echo "这一份（$share）的片做完：现跑 $RAN_NOW、沿用 $reused；片目录 $SHARD_DIRECTORY"
  exit 0
fi
merge_product
echo "片齐了：现跑 $RAN_NOW、沿用 $reused（共 ${#SHARD_LINES[@]} 片）"
exit 0
