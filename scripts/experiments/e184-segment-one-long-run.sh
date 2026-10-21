#!/usr/bin/env bash
# admission: always 每一次调都照此刻工作目录里哪几件做完了判还差什么，上一次跑完不替这一次作保
# run-condition: command python3 flock
# E184（常规层与大负载层的代价计数）第一段（岔路 G1，跑前登记 research/prompts/e184-preregistration.md 5.6 的第一段与第十二节修订 6 至 10）的长活跑法：
#   装置是入库装置 crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs 与转储 bin
#   e184_regular_layer_cost_counts_write_dump.rs（都由主 agent 经 compile-then-swap.py 编好；二进制路径见「环境」）。
#   本脚本把第一段的产物拆成若干件活，每件活的输出落进工作目录、齐了拼成一份产物
#   research/results/e184-regular-layer-cost-counts-segment-one-<日期>.out。
#   件的次序（--list-segments 逐行列）：读仓里的钉住常量与停机 T6、selfcheck、record-fixed-chains、固定流的 keys、
#   各档随机历史的 dump 与 keys（先小后大）、share、library（只在 share 有一族判「部分」时跑）。
#
# long-run-shards: 一片 = 一条流一件活：装置已把每条流做成一片（转储是一条历史一份文件、块键是一条流一组文件，都写到临时名再改名，完成标记在才算做完）；本脚本在外层按步与档把片排成件（读仓里的钉住常量与停机 T6、selfcheck、record-fixed-chains、固定流的 keys、六档随机历史各一件 dump 与一件 keys、share、library），--share k/n 再把每档 keys 里的流按文件次序对 n 取余分给各台
# long-run-resume: 同一条命令重起就是续跑：每件活做完才写一份完成记录（输出末行是装置的 name=<步>_done 且 emitted= 等于行数、装置二进制的 sha256 对得上），重起先读记录、做完的整件跳过，没做完的那件重起装置，装置自己再把已有完成标记的转储与块键文件整份跳过，只补没做完的流；记的装置 sha256 与这一趟不同的件（连同装置自己写的转储与块键目录）先作废移走，逐件报「作废：<件名>（装置 sha256 不同）」；成功那句报「现跑 N、沿用 M」
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
#   多机的读法：本脚本只管一台。--list-segments 打出 --emit-hosts 读到的台数。第二台由主 agent 照这个台数起：
#     MULTI_HOST_RUN_FETCH_DIRECTORY=<不存在的取回目录> bash scripts/hosts/multi-host-run.sh --peer --label e184-share-1 --fetch e184-share-work-1-of-2/keys -- env E184_WORK_DIRECTORY=e184-share-work-1-of-2 bash scripts/experiments/e184-segment-one-long-run.sh --build-here --share 1/2
#   （工作目录写成树里的相对路径，--fetch 才取得回；--build-here 在第二台那棵树里编装置：编译目录里的二进制烘进了本机的路径，带不过去。）
#   本机同时跑 --share 0/2。两台各自重录同样的转储（种子固定、可重现；实测 50 步档 13 秒、100 步档 7 秒，200、400 步档没量），各只算自己那一半流的块键；
#   取回之后在本机 --import-work <取回目录> 把对方做完的块键文件并进本机（不覆盖已有的），再不带参数跑一次本脚本：块键全部沿用、拼出产物。
#   取回的块键文件有多大随步数档涨（实测 100 步档 20 条流的块键文件合计 15G），传输量大时只跑本机，不起第二台也行：台数是读到的，不是脚本强求的。
#   每台的线程：登记值（.claude/heavy-thread-default）是上限，内存吃紧的档在表里写得更小（E184_THREADS 设了取它与表里的较小者）。
#   装置在一趟里不许换，每件记下打它的是哪一版装置（欠账 C862，共用段落在 long-run-lib/binary-identity.sh）：开跑时把主装置与转储 bin 各拷一份进 $WORK_DIRECTORY/pinned-binary/<sha256>/（只读），
#   每件每次起装置都按这两份拷贝，不再 exec 仓根或编译目录里那一份（别的会话的 cargo 随时会把它换掉）；每件做完的记录（steps/<件>.status）写这份拷贝的 sha256，起装置之前另在
#   dumps/<档>.device-stamp、keys/<档>.device-stamp（其余件在 steps/<件>.device-stamp）写下这一趟的 sha256，因为装置自己会把已经写完的转储与块键文件整份跳过，换了装置之后这些旧文件不能留给新装置。
#   续跑开头先过一遍：记的 sha256 与这一趟不同、或留着东西却没有记录的件，连同它的前提作废的件，逐件报「作废：<件名>（装置 sha256 不同）」，记录、输出和装置自己写的转储与块键目录整份移进
#   $WORK_DIRECTORY/void-<日期>-<进程号>/<件名>/（留着不删）再现跑。--import-work 并进对方的块键文件时核对方 keys/<档>.device-stamp 与本机主装置的 sha256 相同，不同的跳过并报作废；
#   拼产物之前再核各件记的 sha256 彼此相同（不同就拒）。第二台要让对方的块键并得进来，得用与本机同一份装置：把本机私有副本送过去、第二台用 E184_BINARY 指到送到的文件（装置里烘进的编译树路径
#   在第二台上要有，见装置退 preflight.py 时的提示），第二台自己 --build-here 编出来的 sha256 一般不同，并进时会被报作废。老的私有副本留着不删（一个 sha256 一个目录），确认不再续跑那一版之后手删。
#
# 用法（在仓里任何目录起都行，脚本自己回到仓根）：
#   bash scripts/experiments/e184-segment-one-long-run.sh                         本机把没做完的件按次序全做完，拼出产物
#   bash scripts/experiments/e184-segment-one-long-run.sh --only <件名>            只做这一件（它前面的件要已做完），不拼产物
#   bash scripts/experiments/e184-segment-one-long-run.sh --assemble              不跑装置，件全做完时拼出产物
#   bash scripts/experiments/e184-segment-one-long-run.sh --list-segments         逐行列件名、做没做完、内存上限与线程（不起装置、不编译）
#   bash scripts/experiments/e184-segment-one-long-run.sh --print-commands        逐件打出会起的完整命令（可与 --share 合用，不起装置、不建目录）
#   bash scripts/experiments/e184-segment-one-long-run.sh --share <第几份>/<共几份>  只转储各档随机历史、只算 序号对共几份取余等于第几份 的流的块键，不拼产物（第二台上用）
#   bash scripts/experiments/e184-segment-one-long-run.sh --import-work <目录>     把另一台取回的工作目录里做完的块键文件并进本机（不覆盖已有的），不起装置
#   bash scripts/experiments/e184-segment-one-long-run.sh --build-here            装置用这棵树里 cargo build --release 编出的 target/release 下那两份（第二台上用）
#   bash scripts/experiments/e184-segment-one-long-run.sh --selftest              假装置、假内存包装、假机器清单在临时树里走一遍（不起 cargo、不碰真装置）
# 环境：E184_BINARY（主装置二进制，默认 编译目录里的 e184_regular_layer_cost_counts）、E184_DUMP_BINARY（转储 bin，默认 E184_BINARY 加 _write_dump）、
#   E184_WORK_DIRECTORY（工作目录，默认 ~/.local/share/singlefs/work/e184-segment-one）、E184_THREADS（见「多机的读法」）、
#   E184_DUMP_MEMORY_CAP / E184_KEYS_MEMORY_CAP（设了就覆盖表里每档的转储 / 块键内存上限）、E184_LIBRARY（长期库目录，默认 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY，
#   再没设取 ~/.local/share/singlefs/crash-amplification/crash-state-verdicts）、E184_PRODUCT_DIRECTORY（产物目录，默认 research/results）。
#   只给自证换假的：E184_HOSTS_EMITTER（默认 scripts/crash/crash-point-replay-shard-configuration-check.sh）、E184_MEMORY_WRAPPER（默认 scripts/hosts/run-with-memory-cap.sh）。
# 产物名：research/results/e184-regular-layer-cost-counts-segment-one-<日期>.out，已有时同一天再跑加 -r2、-r3；已有的一个不动。
# 退出码：0 做完（--share 是这一份做完）；1 有件没跑成或产物不齐（同一条命令重起续跑）；2 用法错；3 装置或仓里的读数报停机（STOP 行该交主 agent，不重跑）；
#   4 阳性对照没过（装置退 4，那一段作废）；78 装置没编好或台数超过两台。
#
# 弄坏开关 E184_SEGMENT_ONE_BREAK=<项，逗号分隔>（只在 --selftest 起的那几趟里认：自证设 E184_SEGMENT_ONE_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   share-format-unchecked         --share 的写法不核对                                  打开后自证「--share 写坏退 2」三格红
#   missing-value-unchecked        --share、--only、--import-work 后面缺值不拦             打开后自证「后面缺值退 2」三格红
#   unknown-argument-accepted      认不得的参数悄悄跳过                                  打开后自证「认不得的参数退 2」红
#   done-marker-ignored            输出末行不是 name=<步>_done 也算做完                    打开后自证「输出末行对不上行数的件不算做完」红
#   binary-change-ignored          装置二进制变了也沿用旧记录                              打开后自证「换了装置二进制：做完的件全部重做」红
#   stop-exit-ignored              装置退 3（停机）当普通失败（退 1）                      打开后自证「selfcheck 报停机退 3」红
#   control-exit-ignored           装置退 4（阳性对照没过）当普通失败（退 1）               打开后自证「record-fixed-chains 报阳性对照没过退 4」红
#   wrapper-exit-ignored           内存包装自己的结局（252）也当装置的输出收下              打开后自证「内存包装排不上退 1」红
#   product-name-fixed             产物名不取号、一律用本名                              打开后自证「产物本名已有：另存加 -r2」红
#   prerequisite-unchecked         --only 不查前面的件做完没有                            打开后自证「--only 点名的件前面的件没做完退 2」红
#   share-restriction-ignored      --share 不分流、每台算全部流的块键                      打开后自证「--share 0/2 只算自己那一半流」红
#   library-always                 share 没有判「部分」的族也跑 library                    打开后自证「share 没有判部分的族：library 不跑」红
#   pinned-mismatch-ignored        钉住常量与段形现算的两态闭式对不上也往下走               打开后自证「钉住的常量对不上退 3」红
#   crashflow-hit-ignored          停机 T6 的读数有命中也往下走                            打开后自证「crates 里出现 CrashFlow 退 3」红
#   import-overwrites              并进块键文件时覆盖已有的                                打开后自证「--import-work 并进块键文件、已有的不覆盖」红
#   exec-root-binary               装置仍按开跑时给的那个路径起、不按私有副本                打开后自证「装置在一趟里不许换」红（跑着的时候仓根那份被换，后起的件自报的 sha256 就不是开跑时那份）
#   owned-files-kept               换了装置作废一件时，装置自己写的转储与块键目录留在原处   打开后自证「换了装置二进制」红（新装置跳过旧装置写完的块键文件）
#   import-sha-unchecked           --import-work 不核对方块键目录记的装置 sha256              打开后自证「--import-work 对方的装置 sha256 不同：跳过并报作废」红
#   merge-sha-unchecked            拼产物之前不核各件记的装置 sha256 彼此相同              打开后自证「拼产物之前各件记的装置 sha256 不同：拒」红
#   （上面的 binary-change-ignored 同时管续跑开头那一遍作废：打开后记的 sha256 不同的件照旧沿用。）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../hosts/heavy-threads.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "$(dirname "${BASH_SOURCE[0]}")/long-run-lib/binary-identity.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; echo "     → 怎么办：本脚本得留在仓里的 scripts/experiments/ 下（它按自己的位置找仓根）；挪走了就放回去再起" >&2; exit 2; }

# 装置二进制的默认位置：主 agent 经 compile-then-swap.py 编好的那个编译目录（编译目录不进仓；--build-here 时改用这棵树的 target/release）。
DEFAULT_BINARY_DIRECTORY="$HOME/.local/share/singlefs/work/m3-close-w4-e184-runner-b/compile-then-swap/target/release"
MAIN_BINARY_NAME="e184_regular_layer_cost_counts"
DUMP_BINARY_NAME="e184_regular_layer_cost_counts_write_dump"
PRODUCT_STEM_NAME="e184-regular-layer-cost-counts-segment-one"
PRODUCT_DIRECTORY="${E184_PRODUCT_DIRECTORY:-research/results}"
WORK_DIRECTORY="${E184_WORK_DIRECTORY:-$HOME/.local/share/singlefs/work/e184-segment-one}"
HOSTS_EMITTER="${E184_HOSTS_EMITTER:-scripts/crash/crash-point-replay-shard-configuration-check.sh}"
MEMORY_WRAPPER="${E184_MEMORY_WRAPPER:-scripts/hosts/run-with-memory-cap.sh}"
FIXED_MEMORY_CAP="16G"
SEEDS_PER_HISTORY_TIER=20

# 随机历史的各档（登记 5.1 主取样四档、8.2 两个方向相反的取样点），次序就是先小后大。
# 一行：档名|步数|比重|盘宽|转储内存上限|转储线程|块键内存上限|块键线程（线程写 all 取这台的登记值，写数字取它与登记值的较小者）。
# 内存上限与线程是这一位执行员量过 50 步档与 100 步档之后定的：100 步档用 31 线程时块键那一步的峰值正好顶到 16G（内存峰值表 2026-10-19 的记录），
# 200、400 步档没量过（块键上限放不进内存的展开不排计划，一条流最多约 7 GB），所以写得保守；撞顶退 250 时按退出码改表再起，已做完的件不重做。
RANDOM_CONFIGS='
random-broad-4g-s050|50|BROAD|FourGibibytes|16G|all|16G|all
random-broad-4g-s100|100|BROAD|FourGibibytes|16G|all|24G|8
random-broad-240slots-s100|100|BROAD|UnitAreaOf240Slots|16G|all|24G|8
random-compaction-4g-s100|100|BROAD_WITH_COMPACTION_WHILE_MOUNTED|FourGibibytes|16G|all|24G|8
random-broad-4g-s200|200|BROAD|FourGibibytes|24G|16|30G|3
random-broad-4g-s400|400|BROAD|FourGibibytes|24G|8|30G|3
'

usage_error() {
  echo "  ✗ $1" >&2
  echo "     → 怎么办：见本脚本文件头「用法」" >&2
  exit 2
}

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）；别处设了就拒。
break_is() {
  [[ ",${E184_SEGMENT_ONE_BREAK:-}," == *",$1,"* ]]
}
if [[ -n "${E184_SEGMENT_ONE_BREAK:-}" && "${1:-}" != --selftest ]]; then
  if [[ -z "${E184_SEGMENT_ONE_SELFTEST_MARK:-}" || "$(cat -- "${E184_SEGMENT_ONE_SELFTEST_MARK}" 2> /dev/null)" != e184-segment-one-selftest ]]; then
    echo "  ✗ E184_SEGMENT_ONE_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
    echo "     → 怎么办：平常跑时不设它" >&2
    exit 2
  fi
fi

# 这台的线程上限：登记的每台默认重活线程数；E184_THREADS 设了取它与登记值的较小者。
registered_threads() {
  local registered
  registered=$(heavy_thread_default) || exit 2
  if [[ -n "${E184_THREADS:-}" ]]; then
    [[ "$E184_THREADS" =~ ^[1-9][0-9]*$ ]] || usage_error "E184_THREADS 要是正整数，给的是「$E184_THREADS」"
    ((E184_THREADS < registered)) && registered=$E184_THREADS
  fi
  echo "$registered"
}

# 档的线程：表里写 all 取这台的上限，写数字取它与这台上限的较小者。
tier_threads() {
  local wanted="$1" limit="$2"
  if [[ "$wanted" == all ]] || ((wanted > limit)); then echo "$limit"; else echo "$wanted"; fi
}

# 产物做完了：末行是 name=done 且 emitted 等于行数。
is_complete_product() {
  local file="$1" line_count last_line
  [[ -f "$file" ]] || return 1
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" =~ ^E7RESULT\ name=done\ .*emitted=${line_count}$ ]]
}

# 还没被占的产物名：带日期；同一天再跑加 -r2、-r3……
next_free_product_path() {
  local today stem="$PRODUCT_DIRECTORY/$PRODUCT_STEM_NAME" candidate revision
  today=$(TZ=Asia/Tokyo date +%F)
  candidate="$stem-$today.out"
  break_is product-name-fixed && { echo "$stem.out"; return; }
  [[ -e "$candidate" ]] || { echo "$candidate"; return; }
  revision=2
  while [[ -e "$stem-$today-r$revision.out" ]]; do revision=$((revision + 1)); done
  echo "$stem-$today-r$revision.out"
}

# 台数：1 + --emit-hosts 打的 MULTI_HOST_COUNT；双机关着（退 3）只有本机；判不过打 0。
host_count() {
  local emitted exit_code count
  emitted=$(bash "$HOSTS_EMITTER" --emit-hosts 2> /dev/null)
  exit_code=$?
  if ((exit_code == 3)); then echo 1; return 0; fi
  if ((exit_code != 0)); then echo 0; return 1; fi
  count=$(grep -m1 '^MULTI_HOST_COUNT=' <<< "$emitted" | cut -d= -f2)
  [[ "$count" =~ ^[0-9]+$ ]] || { echo 0; return 1; }
  echo $((1 + count))
}

# ───────────────────────────── 件的清单与每件的属性 ─────────────────────────────

declare -A CONFIG_STEPS=() CONFIG_WEIGHTS=() CONFIG_WIDTH=() CONFIG_DUMP_CAP=() CONFIG_DUMP_THREADS=() CONFIG_KEYS_CAP=() CONFIG_KEYS_THREADS=()
CONFIG_LIST=()
SEGMENTS=()

load_segment_table() {
  local name steps weights width dump_cap dump_threads keys_cap keys_threads
  CONFIG_LIST=()
  SEGMENTS=(repo-reads selfcheck record-fixed-chains keys-fixed)
  while IFS='|' read -r name steps weights width dump_cap dump_threads keys_cap keys_threads; do
    [[ -n "$name" ]] || continue
    CONFIG_LIST+=("$name")
    CONFIG_STEPS[$name]=$steps; CONFIG_WEIGHTS[$name]=$weights; CONFIG_WIDTH[$name]=$width
    CONFIG_DUMP_CAP[$name]=$dump_cap; CONFIG_DUMP_THREADS[$name]=$dump_threads
    CONFIG_KEYS_CAP[$name]=$keys_cap; CONFIG_KEYS_THREADS[$name]=$keys_threads
    SEGMENTS+=("dump-$name" "keys-$name")
  done <<< "$RANDOM_CONFIGS"
  SEGMENTS+=(share library)
}

is_segment_name() {
  local name="$1" known
  for known in "${SEGMENTS[@]}"; do [[ "$known" == "$name" ]] && return 0; done
  return 1
}

# 一件活用哪份二进制：main、dump，或 none（脚本自己的读数）。
segment_binary_kind() {
  case "$1" in
    repo-reads) echo none ;;
    dump-*) echo dump ;;
    *) echo main ;;
  esac
}

# 一件活的输出末行该是哪个收尾名。
segment_closing_name() {
  case "$1" in
    repo-reads) echo pinned_smoke_done ;;
    selfcheck) echo selfcheck_done ;;
    record-fixed-chains) echo chains_done ;;
    dump-*) echo dump_done ;;
    keys-*) echo keys_done ;;
    share) echo share_done ;;
    library) echo library_done ;;
  esac
}

# 一件活前面要先做完的件（一行一个）。
segment_prerequisites() {
  local seg="$1" config
  case "$seg" in
    keys-fixed) echo record-fixed-chains ;;
    keys-*) echo "dump-${seg#keys-}" ;;
    share)
      echo repo-reads
      echo keys-fixed
      for config in "${CONFIG_LIST[@]}"; do echo "keys-$config"; done ;;
    library) echo share ;;
  esac
}

# 这一件活的内存上限与线程（线程不超过这台的上限 THREAD_LIMIT）。
segment_cap() {
  local seg="$1" config
  case "$seg" in
    dump-*) config="${seg#dump-}"; echo "${E184_DUMP_MEMORY_CAP:-${CONFIG_DUMP_CAP[$config]}}" ;;
    keys-fixed) echo "$FIXED_MEMORY_CAP" ;;
    keys-*) config="${seg#keys-}"; echo "${E184_KEYS_MEMORY_CAP:-${CONFIG_KEYS_CAP[$config]}}" ;;
    *) echo "$FIXED_MEMORY_CAP" ;;
  esac
}
segment_threads() {
  local seg="$1" config
  case "$seg" in
    dump-*) config="${seg#dump-}"; tier_threads "${CONFIG_DUMP_THREADS[$config]}" "$THREAD_LIMIT" ;;
    keys-fixed) echo "$THREAD_LIMIT" ;;
    keys-*) config="${seg#keys-}"; tier_threads "${CONFIG_KEYS_THREADS[$config]}" "$THREAD_LIMIT" ;;
    *) echo "$THREAD_LIMIT" ;;
  esac
}

# 工作目录里的文件。
segment_output_path() { echo "$WORK_DIRECTORY/steps/$1.out"; }
segment_status_path() { echo "$WORK_DIRECTORY/steps/$1.status"; }
segment_seconds_path() { echo "$WORK_DIRECTORY/steps/$1.seconds"; }
segment_log_path() { echo "$WORK_DIRECTORY/logs/$1-$RUN_STAMP.err"; }

# 状态记录里的一个字段（一行 key=value 空格隔开）。
status_field() {
  local seg="$1" key="$2"
  grep -o -m1 -- "\b$key=[^ ]*" "$(segment_status_path "$seg")" 2> /dev/null | head -n 1 | cut -d= -f2-
}

# 输出末行是该件的收尾行，且 emitted 等于行数（强制跑时多出的 name=preflight 那一行不数）。
output_is_closed() {
  local file="$1" closing="$2" last_line declared total
  [[ -s "$file" ]] || return 1
  break_is done-marker-ignored && return 0
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" =~ ^E7RESULT\ name=${closing}\ .*emitted=([0-9]+)$ ]] || return 1
  declared="${BASH_REMATCH[1]}"
  total=$(wc -l < "$file")
  [[ "$(head -n 1 -- "$file")" == "E7RESULT name=preflight "* ]] && total=$((total - 1))
  ((declared == total))
}

# 这件活的二进制 sha256（none 的件不比）。
expected_binary_sha() {
  case "$(segment_binary_kind "$1")" in
    main) echo "$MAIN_SHA" ;;
    dump) echo "$DUMP_SHA" ;;
    none) echo none ;;
  esac
}

# 做完了：输出与记录都在、输出收尾对得上、记录里的装置 sha256 与现在的二进制相同。
segment_is_done() {
  local seg="$1" recorded
  [[ -f "$(segment_output_path "$seg")" && -f "$(segment_status_path "$seg")" ]] || return 1
  [[ "$(status_field "$seg" exit)" == 0 ]] || return 1
  output_is_closed "$(segment_output_path "$seg")" "$(segment_closing_name "$seg")" || return 1
  if [[ -n "${MAIN_SHA:-}" ]] && ! break_is binary-change-ignored; then
    recorded=$(status_field "$seg" binary_sha256)
    [[ "$recorded" == "$(expected_binary_sha "$seg")" ]] || return 1
  fi
  return 0
}

# 这一件起装置之前写下「这一趟用的装置 sha256」的文件：装置自己写目录的件（转储、块键）放在目录旁（--import-work 取回的 keys 目录里带得走），其余放 steps。
segment_stamp_path() {
  case "$1" in
    record-fixed-chains) echo "$WORK_DIRECTORY/dumps/fixed.device-stamp" ;;
    keys-fixed) echo "$WORK_DIRECTORY/keys/fixed.device-stamp" ;;
    dump-*) echo "$WORK_DIRECTORY/dumps/${1#dump-}.device-stamp" ;;
    keys-*) echo "$WORK_DIRECTORY/keys/${1#keys-}.device-stamp" ;;
    *) echo "$WORK_DIRECTORY/steps/$1.device-stamp" ;;
  esac
}

# 这一件装置自己写的文件与目录（一行一个）：装置会把里面已经写完的整份跳过，换了装置就不能留给新装置。
segment_owned_paths() {
  case "$1" in
    record-fixed-chains) echo "$WORK_DIRECTORY/dumps/fixed" ;;
    keys-fixed) echo "$WORK_DIRECTORY/keys/fixed" ;;
    dump-*) echo "$WORK_DIRECTORY/dumps/${1#dump-}" ;;
    keys-*) echo "$WORK_DIRECTORY/keys/${1#keys-}" ;;
    share) echo "$WORK_DIRECTORY/steps/share.file" ;;
  esac
}

# 这一件在工作目录里留着什么：记录、输出、起装置前写的 sha256 记录，或装置自己写的目录里有东西。
segment_has_trace() {
  local seg="$1" path
  [[ -e "$(segment_status_path "$seg")" || -e "$(segment_output_path "$seg")" || -e "$(segment_stamp_path "$seg")" ]] && return 0
  while IFS= read -r path; do
    [[ -n "$path" ]] || continue
    if [[ -d "$path" ]]; then
      [[ -n "$(ls -A -- "$path" 2> /dev/null)" ]] && return 0
    elif [[ -e "$path" ]]; then
      return 0
    fi
  done < <(segment_owned_paths "$seg")
  return 1
}

# 这一件留下的东西记的是哪一版装置：做完的记录里的，没有就取起装置前写的；都没有打空。
segment_trace_sha() {
  local seg="$1" recorded
  recorded=$(status_field "$seg" binary_sha256)
  if [[ -z "$recorded" && -f "$(segment_stamp_path "$seg")" ]]; then recorded=$(head -n 1 -- "$(segment_stamp_path "$seg")" | tr -d '[:space:]'); fi
  echo "$recorded"
}

# 作废一件：记录、输出、秒数、起装置前写的记录、装置自己写的目录（--share 的半份块键输出也算）整份移进 void 目录，留着不删；报一行。
void_segment() { # <件> <记的 sha256> <这一趟的 sha256> <前提作废了就写前提的件名，否则空>
  local seg="$1" recorded="$2" expected="$3" prerequisite="$4" void="$WORK_DIRECTORY/void-$RUN_STAMP/$1" path
  mkdir -p -- "$void" || exit 1
  for path in "$(segment_output_path "$seg")" "$(segment_status_path "$seg")" "$(segment_seconds_path "$seg")" "$(segment_stamp_path "$seg")" \
    "$WORK_DIRECTORY/logs/$seg".share-*-of-*.out; do
    [[ -e "$path" ]] && { mv -- "$path" "$void/" || exit 1; }
  done
  if ! break_is owned-files-kept; then
    while IFS= read -r path; do
      [[ -n "$path" && -e "$path" ]] && { mv -- "$path" "$void/" || exit 1; }
    done < <(segment_owned_paths "$seg")
  fi
  if [[ -n "$prerequisite" ]]; then
    echo "作废：$seg（前提 $prerequisite 作废了，它的输入变了）：整份移到 $void，不删，现跑"
  else
    binary_identity_void_line "$seg" "$recorded" "$void" "$expected"
  fi
}

# 续跑开头先过一遍（件按次序，前提在前）：留着东西、记的装置 sha256 与这一趟不同（或没有记录）的件作废；前提作废了的件跟着作废。
VOIDED_COUNT=0
void_stale_segments() {
  local seg expected recorded prerequisite voided_prerequisite voided=" "
  VOIDED_COUNT=0
  break_is binary-change-ignored && return 0
  for seg in "${SEGMENTS[@]}"; do
    [[ "$(segment_binary_kind "$seg")" != none ]] || continue
    segment_has_trace "$seg" || continue
    expected=$(expected_binary_sha "$seg")
    recorded=$(segment_trace_sha "$seg")
    voided_prerequisite=""
    while IFS= read -r prerequisite; do
      [[ -n "$prerequisite" && "$voided" == *" $prerequisite "* ]] && voided_prerequisite="$prerequisite"
    done < <(segment_prerequisites "$seg")
    if [[ "$recorded" != "$expected" || -n "$voided_prerequisite" ]]; then
      if [[ "$recorded" != "$expected" ]]; then voided_prerequisite=""; fi
      void_segment "$seg" "$recorded" "$expected" "$voided_prerequisite"
      voided+="$seg "
      VOIDED_COUNT=$((VOIDED_COUNT + 1))
    fi
  done
}

# 这件活累计跑了几次、共多少秒（只数跑完退出了的那几次，被杀掉的没记到）。
segment_seconds_summary() {
  local file; file="$(segment_seconds_path "$1")"
  [[ -f "$file" ]] || { echo "runs=0 seconds_total=0"; return; }
  awk '{ total += $1; runs += 1 } END { printf "runs=%d seconds_total=%d", runs, total }' "$file"
}

# ───────────────────────────── 装置二进制 ─────────────────────────────

# 要用的两份二进制：--build-here 用这棵树 target/release 下的；E184_BINARY 给了用它；都没有取默认编译目录。
resolve_binaries() {
  if ((BUILD_HERE)); then
    MAIN_BINARY="target/release/$MAIN_BINARY_NAME"
    DUMP_BINARY="target/release/$DUMP_BINARY_NAME"
  elif [[ -n "${E184_BINARY:-}" ]]; then
    MAIN_BINARY="$E184_BINARY"
    DUMP_BINARY="${E184_DUMP_BINARY:-${E184_BINARY}_write_dump}"
  else
    MAIN_BINARY="$DEFAULT_BINARY_DIRECTORY/$MAIN_BINARY_NAME"
    DUMP_BINARY="${E184_DUMP_BINARY:-$DEFAULT_BINARY_DIRECTORY/$DUMP_BINARY_NAME}"
  fi
}

build_here() {
  bash scripts/hosts/capped.sh "$THREAD_LIMIT" cargo build --release -p singlefs-checker-tier --bin "$MAIN_BINARY_NAME" --bin "$DUMP_BINARY_NAME" \
    > "$WORK_DIRECTORY/logs/build-$RUN_STAMP.log" 2>&1 || {
    echo "  ✗ 装置编不过（$WORK_DIRECTORY/logs/build-$RUN_STAMP.log）" >&2
    echo "     → 怎么办：照 cargo 打的第一处 error 改装置，再起本脚本" >&2
    exit 1
  }
}

# 两份二进制都得在、能执行；算它们的 sha256。
require_binaries() {
  local binary
  for binary in "$MAIN_BINARY" "$DUMP_BINARY"; do
    if [[ ! -x "$binary" ]]; then
      echo "  ✗ 装置还没编好：$binary" >&2
      echo "     → 怎么办：主 agent 经 compile-then-swap.py 编好并把路径设进 E184_BINARY（转储 bin 同目录加 _write_dump），或本脚本加 --build-here 用这棵树 cargo build 出的" >&2
      exit 78
    fi
  done
  # 开跑时各拷一份私有副本：之后每一件都按这两份拷贝起，MAIN_SHA、DUMP_SHA 是拷贝的 sha256
  MAIN_SOURCE="$MAIN_BINARY"
  DUMP_SOURCE="$DUMP_BINARY"
  binary_identity_pin "$MAIN_SOURCE" "$WORK_DIRECTORY/pinned-binary" || exit 1
  MAIN_BINARY="$PINNED_BINARY"; MAIN_SHA="$PINNED_BINARY_SHA256"
  binary_identity_pin "$DUMP_SOURCE" "$WORK_DIRECTORY/pinned-binary" || exit 1
  DUMP_BINARY="$PINNED_BINARY"; DUMP_SHA="$PINNED_BINARY_SHA256"
}

# ───────────────────────────── 一件活怎么起 ─────────────────────────────

# 这件活的装置参数，放进 COMMAND；share 模式下块键那几件只算自己那份流（链接目录里只有对应的转储文件）。
segment_command() {
  local seg="$1" config candidates=() against=() dump_directory
  COMMAND=()
  case "$seg" in
    selfcheck) COMMAND=(selfcheck) ;;
    record-fixed-chains) COMMAND=(record-fixed-chains --out "$WORK_DIRECTORY/dumps/fixed") ;;
    keys-fixed) COMMAND=(keys --dump "$WORK_DIRECTORY/dumps/fixed" --out "$WORK_DIRECTORY/keys/fixed" --controls) ;;
    dump-*)
      config="${seg#dump-}"
      COMMAND=(dump-random-histories --steps "${CONFIG_STEPS[$config]}" --weights "${CONFIG_WEIGHTS[$config]}" --width "${CONFIG_WIDTH[$config]}"
        --seeds "$SEEDS_PER_HISTORY_TIER" --out "$WORK_DIRECTORY/dumps/$config" --check-determinism) ;;
    keys-*)
      config="${seg#keys-}"
      dump_directory="$WORK_DIRECTORY/dumps/$config"
      if [[ -n "$SHARE" ]]; then dump_directory=$(make_share_link_directory "$config"); fi
      COMMAND=(keys --dump "$dump_directory" --out "$WORK_DIRECTORY/keys/$config" --controls) ;;
    share)
      COMMAND=(share --candidates "$WORK_DIRECTORY/keys/fixed")
      for config in "${CONFIG_LIST[@]}"; do COMMAND+=(--candidates "$WORK_DIRECTORY/keys/$config"); done
      COMMAND+=(--against "$WORK_DIRECTORY/keys/fixed" --out "$WORK_DIRECTORY/steps/share.file" --pinned-smoke "$WORK_DIRECTORY/pinned-smoke.txt") ;;
    library)
      COMMAND=(library --keys "$WORK_DIRECTORY/keys/fixed")
      for config in "${CONFIG_LIST[@]}"; do COMMAND+=(--keys "$WORK_DIRECTORY/keys/$config"); done
      COMMAND+=(--library "$LIBRARY_DIRECTORY") ;;
  esac
}

# --share k/n：这一档转储里按文件次序序号对 n 取余等于 k 的流，用符号链接放进一个目录，块键那一步只读这个目录。
make_share_link_directory() {
  local config="$1" directory="$WORK_DIRECTORY/share-links/$config-${SHARE%/*}-of-${SHARE#*/}" index=0 dump_file
  if [[ "$MODE" == print ]]; then echo "$directory"; return 0; fi
  mkdir -p -- "$directory"
  find "$directory" -maxdepth 1 -type l -delete
  while IFS= read -r dump_file; do
    if break_is share-restriction-ignored || ((index % ${SHARE#*/} == ${SHARE%/*})); then ln -s -- "$(realpath -- "$dump_file")" "$directory/"; fi
    index=$((index + 1))
  done < <(find "$WORK_DIRECTORY/dumps/$config" -maxdepth 1 -name '*.dump' | LC_ALL=C sort)
  echo "$directory"
}

# 装置退 3 时把它报的不合格行转出来；退 4 时转出没过的对照。
show_stop_lines() {
  local file="$1"
  grep -E 'result=fail|match=false|name=flow_failed|name=dump_failed|STOP' -- "$file" 2> /dev/null | head -n 10 | cut -c1-400 >&2
}
show_control_lines() {
  local file="$1"
  grep -E 'passed=false|result=fail' -- "$file" 2> /dev/null | head -n 10 | cut -c1-400 >&2
}

# 写完成记录：输出先写临时名再改名，记录最后写（有记录就是做完了）。
store_segment_result() {
  local seg="$1" out_tmp="$2" seconds="$3" line_count
  line_count=$(wc -l < "$out_tmp")
  mv -- "$out_tmp" "$(segment_output_path "$seg")" || return 1
  printf 'segment=%s exit=0 binary=%s binary_sha256=%s seconds_last=%s lines=%s\n' "$seg" "$(segment_binary_kind "$seg")" "$(expected_binary_sha "$seg")" "$seconds" "$line_count" \
    > "$(segment_status_path "$seg").tmp-$$" && mv -- "$(segment_status_path "$seg").tmp-$$" "$(segment_status_path "$seg")"
}

# 起一件活：返回 0 做完，1 没跑成，3 停机，4 阳性对照没过。装置的标准输出进临时文件，错误输出进 logs。
run_segment() {
  local seg="$1" kind cap threads binary out_tmp log started exit_code seconds
  kind=$(segment_binary_kind "$seg")
  mkdir -p -- "$WORK_DIRECTORY/steps" "$WORK_DIRECTORY/logs" "$WORK_DIRECTORY/dumps" "$WORK_DIRECTORY/keys"
  out_tmp="$WORK_DIRECTORY/steps/$seg.out.running-$$"
  log=$(segment_log_path "$seg")
  started=$(date +%s)
  if [[ "$kind" == none ]]; then
    run_repo_reads > "$out_tmp" 2> "$log"
    exit_code=$?
    cap="-"; threads="-"
  else
    cap=$(segment_cap "$seg"); threads=$(segment_threads "$seg")
    binary="$MAIN_BINARY"; [[ "$kind" == dump ]] && binary="$DUMP_BINARY"
    if break_is exec-root-binary; then
      binary="$MAIN_SOURCE"; [[ "$kind" == dump ]] && binary="$DUMP_SOURCE"
    fi
    binary_identity_assert_unchanged "$MAIN_BINARY" "$MAIN_SHA" && binary_identity_assert_unchanged "$DUMP_BINARY" "$DUMP_SHA" || return 1
    # 起装置之前先写下这一趟用的装置 sha256：装置会把写完的转储与块键文件整份跳过，下一趟换了装置时靠它认出这些文件是旧装置的
    printf '%s\n' "$(expected_binary_sha "$seg")" > "$(segment_stamp_path "$seg").tmp-$$" && mv -f -- "$(segment_stamp_path "$seg").tmp-$$" "$(segment_stamp_path "$seg")" || return 1
    segment_command "$seg"
    echo "件 $seg：起跑（内存上限 $cap、线程 $threads、日志 $log）"
    # 装置自己的线程变量 E184_THREADS 排在 capped.sh 设的 SINGLEFS_THREAD_CAP 前面：用户环境里留着旧值时，这一件的线程数以表为准。
    bash scripts/hosts/capped.sh "$threads" env "E184_THREADS=$threads" bash "$MEMORY_WRAPPER" "$cap" nice -n 19 "$binary" "${COMMAND[@]}" > "$out_tmp" 2> "$log"
    exit_code=$?
  fi
  seconds=$(( $(date +%s) - started ))
  echo "$seconds" >> "$(segment_seconds_path "$seg")"
  case "$exit_code" in
    0) ;;
    3)
      break_is stop-exit-ignored || { mv -- "$out_tmp" "$out_tmp.exit3"; echo "  ✗ 件 $seg：停机条款没过（输出留在 $out_tmp.exit3）" >&2; show_stop_lines "$out_tmp.exit3"; return 3; } ;;
    4)
      break_is control-exit-ignored || { mv -- "$out_tmp" "$out_tmp.exit4"; echo "  ✗ 件 $seg：阳性对照没过，这一段作废（输出留在 $out_tmp.exit4）" >&2; show_control_lines "$out_tmp.exit4"; return 4; } ;;
  esac
  if ((exit_code == 245 || (exit_code >= 250 && exit_code <= 254))) && ! break_is wrapper-exit-ignored; then
    echo "  ✗ 件 $seg：内存包装退 $exit_code（245 显存不够、250 撞了自己的上限、252 内存不够排不上、253 超时、254 被总上限挤掉，见 run-with-memory-cap.sh 文件头「退出码」）：这一次的输出不算结果" >&2
    mv -- "$out_tmp" "$out_tmp.wrapper-exit$exit_code" 2> /dev/null
    return 1
  fi
  if ((exit_code != 0)) && ! break_is wrapper-exit-ignored; then
    echo "  ✗ 件 $seg：装置退 $exit_code（日志 $log）" >&2
    tail -n 5 -- "$log" >&2
    if grep -q 'preflight.py' -- "$log" 2> /dev/null; then
      echo "     → 装置二进制里烘进的编译树路径（CARGO_MANIFEST_DIR）下找不到 preflight.py：compile-then-swap.py 编完删了那棵树。在那个路径下建 crates、.claude 两个指向本仓的符号链接，或用 --build-here 在本棵树里编" >&2
    fi
    mv -- "$out_tmp" "$out_tmp.exit$exit_code" 2> /dev/null
    return 1
  fi
  if ! output_is_closed "$out_tmp" "$(segment_closing_name "$seg")"; then
    echo "  ✗ 件 $seg：输出末行不是 name=$(segment_closing_name "$seg")、或 emitted= 对不上行数（输出留在 $out_tmp.unclosed）" >&2
    mv -- "$out_tmp" "$out_tmp.unclosed"
    return 1
  fi
  if [[ -n "$SHARE" && "$seg" == keys-* && "$seg" != keys-fixed ]]; then
    # --share 只算了一半流：这份输出不是整档的块键清单，不写完成记录（整档的清单由不带 --share 的那一趟沿用块键文件时打出）。
    mv -- "$out_tmp" "$WORK_DIRECTORY/logs/$seg.share-${SHARE%/*}-of-${SHARE#*/}.out" || return 1
    echo "件 $seg（这一份 $SHARE）：块键算完（$seconds 秒；$(tail -n 1 -- "$WORK_DIRECTORY/logs/$seg.share-${SHARE%/*}-of-${SHARE#*/}.out" | cut -c1-200)）"
    return 0
  fi
  store_segment_result "$seg" "$out_tmp" "$seconds" || return 1
  echo "件 $seg：做完（$seconds 秒；$(tail -n 1 -- "$(segment_output_path "$seg")" | cut -c1-200)）"
  return 0
}

# ───────────────────────────── 仓里的读数：钉住的冒烟流与停机 T6 ─────────────────────────────

# 停机 T6（登记第十一节）：crates/singlefs-checker-tier/src/crash_injection.rs 与 crates/singlefs-harness/src 里不能有 CrashFlow；
# 冒烟层测试钉了段形的流：从 SEGMENT_SIZES 现算两态闭式，与同一个测试文件里钉住的两态闭式比，一致才写进 --pinned-smoke 文件
# （登记第十二节、装置文件头：执行员 grep 测试文件取常量、照闭式现算核过再写）。输出是 E7RESULT 行，末行 pinned_smoke_done。
run_repo_reads() {
  local ignore_mismatch=0 ignore_crashflow=0
  break_is pinned-mismatch-ignored && ignore_mismatch=1
  break_is crashflow-hit-ignored && ignore_crashflow=1
  python3 -I - "$PWD" "$WORK_DIRECTORY/pinned-smoke.txt" "$ignore_mismatch" "$ignore_crashflow" <<'PY'
import pathlib, re, sys

root = pathlib.Path(sys.argv[1])
pinned_path = pathlib.Path(sys.argv[2])
ignore_mismatch = sys.argv[3] == "1"
ignore_crashflow = sys.argv[4] == "1"
lines = []


def emit(text):
    lines.append("E7RESULT " + text)


# 停机 T6：读不到文件不等于零命中。
hits, files_read = 0, 0
for relative in ("crates/singlefs-checker-tier/src/crash_injection.rs", "crates/singlefs-harness/src"):
    base = root / relative
    found = [base] if base.is_file() else (sorted(base.rglob("*.rs")) if base.is_dir() else [])
    for source in found:
        files_read += 1
        hits += sum(1 for line in source.read_text(errors="replace").splitlines() if "CrashFlow" in line)
t6_pass = files_read > 0 and hits == 0
emit(f"name=stop_check item=T6_no_crashflow_in_crates result={'pass' if t6_pass else 'fail'} hits={hits} files_read={files_read}")

tests = root / "crates/singlefs-checker-tier/tests"
already_recorded_files = {
    "crash_enumeration_snapshot_creation_stream.rs", "crash_enumeration_fixed_script_stream.rs",
    "crash_enumeration_multi_record_publish_stream.rs", "crash_enumeration_new_pool_file_creation_stream.rs",
}
recorded_cases = {
    "new-pool-file-creation-stream", "overwrite-fixed-script-stream",
    "multi-record-publish-stream", "snapshot-creation-stream",
}
constant_form = re.compile(r"^const (CLOSED_FORM_WITH_TWO_STATES_PER_WRITE|FULL_STATES_WITH_TWO_STATES_PER_WRITE): u64 = ([0-9_]+);", re.M)
asserted_form = re.compile(r"assert_eq!\(closed_form_with_two_states_per_write, ([0-9_]+)\);")
shape_form = re.compile(r"^const SEGMENT_SIZES: \[usize; (\d+)\] = \[(.*?)\];", re.M | re.S)
pinned_lines, pinned_cases, mismatches = [], set(), 0
for source in sorted(tests.glob("crash_enumeration_*_stream.rs")):
    if source.name in already_recorded_files:
        continue
    text = source.read_text()
    shape = shape_form.search(text)
    if not shape:
        continue
    sizes = [int(word) for word in re.findall(r"\d+", shape.group(2))]
    two_state = 1 + sum((1 << size) - 1 for size in sizes)
    constant, asserted = constant_form.search(text), asserted_form.search(text)
    pinned_value, pinned_by = None, "none"
    if constant:
        pinned_value, pinned_by = int(constant.group(2).replace("_", "")), constant.group(1)
    elif asserted:
        pinned_value, pinned_by = int(asserted.group(1).replace("_", "")), "assert_eq_of_closed_form_with_two_states_per_write"
    label = source.name.removeprefix("crash_enumeration_").removesuffix(".rs")
    matches = len(sizes) == int(shape.group(1)) and pinned_value == two_state
    mismatches += 0 if matches else 1
    pinned_cases.add(label.replace("_", "-"))
    emit(f"name=pinned_smoke flow={label} segments={len(sizes)} writes={sum(sizes)} longest_segment={max(sizes)} two_state={two_state} pinned_value={pinned_value if pinned_value is not None else 'none'} pinned_by={pinned_by} match={'true' if matches else 'false'} source={source.name}")
    if matches or ignore_mismatch:
        pinned_lines.append(f"{label} {two_state} {max(sizes)}")

stage_inputs = root / ".claude/gate.d/stage-inputs.tsv"
missing_cases = []
if stage_inputs.is_file():
    for row in stage_inputs.read_text().splitlines():
        if row.startswith("crash-case:"):
            case = row.split("\t")[0][len("crash-case:"):]
            if case not in recorded_cases and case not in pinned_cases:
                missing_cases.append(case)
    for case in missing_cases:
        emit(f"name=pinned_smoke_missing case={case} reason=no_segment_shape_constant_with_a_pinned_closed_form")
else:
    mismatches += 1
    emit("name=stop_check item=pinned_smoke_case_list result=fail detail=stage_inputs_unreadable")

failed = (mismatches > 0 and not ignore_mismatch) or (not t6_pass and not ignore_crashflow)
if not failed:
    temporary = pinned_path.with_name(pinned_path.name + ".tmp")
    temporary.write_text("# 冒烟层测试钉住段形的流：<流名> <两态闭式> <最长段>；两态闭式由 SEGMENT_SIZES 现算并与同文件的钉住常量核过\n" + "\n".join(pinned_lines) + "\n")
    temporary.replace(pinned_path)
emit(f"name=pinned_smoke_done pinned={len(pinned_lines)} missing={len(missing_cases)} mismatches={mismatches} t6_hits={hits} emitted={len(lines) + 1}")
print("\n".join(lines))
sys.exit(3 if failed else 0)
PY
}

# ───────────────────────────── 拼产物、并进对方的块键、列件 ─────────────────────────────

# library 只在 share 有一族（family 范围）判「部分」时跑（登记第六节 J1-库）。
library_is_needed() {
  break_is library-always && return 0
  grep -q -- '^E7RESULT name=share_group scope=family:[^ ]* .*verdict=partial' "$(segment_output_path share)" 2> /dev/null
}

# 件齐了拼出产物：先写临时文件、核末行，再挪到还没被占的产物名；返回 1 表示没拼成。
assemble_product() {
  local seg missing=() temporary destination line_count
  local main_shas dump_shas
  for seg in "${SEGMENTS[@]}"; do
    if [[ "$seg" == library ]] && { ! segment_is_done share || ! library_is_needed; }; then continue; fi
    segment_is_done "$seg" || missing+=("$seg")
  done
  if ((${#missing[@]} > 0)); then
    echo "  ✗ 件还没齐：缺 ${missing[*]}" >&2
    echo "     → 怎么办：不带 --assemble 重起本脚本，做完没做完的件" >&2
    return 1
  fi
  main_shas=$(for seg in "${SEGMENTS[@]}"; do [[ "$(segment_binary_kind "$seg")" == main && -f "$(segment_status_path "$seg")" ]] && status_field "$seg" binary_sha256; done | sort -u)
  dump_shas=$(for seg in "${SEGMENTS[@]}"; do [[ "$(segment_binary_kind "$seg")" == dump && -f "$(segment_status_path "$seg")" ]] && status_field "$seg" binary_sha256; done | sort -u)
  if (($(grep -c . <<< "$main_shas") > 1 || $(grep -c . <<< "$dump_shas") > 1)) && ! break_is merge-sha-unchecked; then
    echo "  ✗ 各件是用不同的装置二进制做的（主装置 sha256 $(tr '\n' ' ' <<< "$main_shas")；转储 bin $(tr '\n' ' ' <<< "$dump_shas")），拼不成一份产物" >&2
    echo "     → 怎么办：换了二进制就换工作目录重做，或删掉旧二进制做的那几件的记录再起" >&2
    return 1
  fi
  temporary="$WORK_DIRECTORY/product-$RUN_STAMP.out"
  {
    echo "E7RESULT name=segment_one_begin date=$(TZ=Asia/Tokyo date +%F) main_binary_sha256=${main_shas:-none} dump_binary_sha256=${dump_shas:-none} segments=${#SEGMENTS[@]}"
    for seg in "${SEGMENTS[@]}"; do
      if [[ "$seg" == library ]] && { ! segment_is_done share || ! library_is_needed; }; then
        echo "E7RESULT name=segment segment=library state=not_run reason=no_family_share_verdict_partial"
        continue
      fi
      echo "E7RESULT name=segment segment=$seg state=done $(segment_seconds_summary "$seg") lines=$(status_field "$seg" lines) binary_sha256=$(status_field "$seg" binary_sha256)"
      cat -- "$(segment_output_path "$seg")"
    done
  } > "$temporary"
  line_count=$(wc -l < "$temporary")
  echo "E7RESULT name=done segments=${#SEGMENTS[@]} emitted=$((line_count + 1))" >> "$temporary"
  if ! is_complete_product "$temporary"; then
    echo "  ✗ 产物不齐（末行不是 name=done 或 emitted= 对不上行数）：$temporary" >&2
    echo "     → 怎么办：产物不入库；看这个临时文件" >&2
    return 1
  fi
  mkdir -p -- "$PRODUCT_DIRECTORY" || return 1
  destination=$(next_free_product_path)
  mv -n -- "$temporary" "$destination"
  if [[ -e "$temporary" ]]; then
    echo "  ✗ 产物名 $destination 被占了，没挪进去（拼出的留在 $temporary）" >&2
    return 1
  fi
  echo "产物：$destination（$(wc -l < "$destination") 行，sha256 $(sha256sum -- "$destination" | cut -d' ' -f1)）"
}

# --import-work：另一台取回的工作目录里，做完的块键与对照文件并进本机（已有同名的不覆盖）。
import_work() {
  local source_directory="$1" file name parent destination imported=0 skipped=0 stamp recorded
  [[ -d "$source_directory" ]] || usage_error "--import-work 点名的目录不在：$source_directory"
  # 对方的块键是哪一版装置算的，要和本机主装置同一版才并进来（对方块键目录旁的 <档>.device-stamp 记着）；本机装置没编好就核不了
  if [[ -z "${MAIN_SHA:-}" ]] && ! break_is import-sha-unchecked; then
    echo "  ✗ 本机主装置不在（$MAIN_BINARY），核不了对方块键文件是哪一版装置算的" >&2
    echo "     → 怎么办：先把本机主装置编好（或 E184_BINARY 指到它），再起 --import-work" >&2
    exit 78
  fi
  while IFS= read -r -d '' file; do
    name="${file##*/}"
    parent="$(basename -- "$(dirname -- "$file")")"
    [[ "$parent" == fixed || " ${CONFIG_LIST[*]} " == *" $parent "* ]] || continue
    destination="$WORK_DIRECTORY/keys/$parent/$name"
    if [[ -e "$destination" ]] && ! break_is import-overwrites; then continue; fi
    stamp="$(dirname -- "$(dirname -- "$file")")/$parent.device-stamp"
    recorded=$(head -n 1 -- "$stamp" 2> /dev/null | tr -d '[:space:]')
    if [[ "$recorded" != "${MAIN_SHA:-}" ]] && ! break_is import-sha-unchecked; then
      echo "作废：keys/$parent/$name（装置 sha256 不同）：对方记的是 ${recorded:-没有记录}，本机主装置的是 $MAIN_SHA；不并进来"
      skipped=$((skipped + 1))
      continue
    fi
    mkdir -p -- "$WORK_DIRECTORY/keys/$parent"
    # 本机这一档还没有 sha256 记录（本机没跑过它）：并进来的块键文件算本机主装置的，记一笔，下一趟续跑开头才不会把它们当成没有记录的旧文件作废
    if [[ -n "${MAIN_SHA:-}" && ! -e "$WORK_DIRECTORY/keys/$parent.device-stamp" ]]; then
      printf '%s\n' "$MAIN_SHA" > "$WORK_DIRECTORY/keys/$parent.device-stamp.tmp-$$" && mv -f -- "$WORK_DIRECTORY/keys/$parent.device-stamp.tmp-$$" "$WORK_DIRECTORY/keys/$parent.device-stamp" || exit 1
    fi
    cp -- "$file" "$destination.importing-$$" && mv -- "$destination.importing-$$" "$destination" && imported=$((imported + 1))
  done < <(find "$source_directory" -maxdepth 6 -type f \( -name '*.keys' -o -name '*.controls' \) -print0)
  echo "并进块键与对照文件 $imported 个（已有的不覆盖），装置 sha256 对不上而跳过 $skipped 个：$WORK_DIRECTORY/keys"
  ((skipped == 0)) || echo "     → 怎么办：对方要用与本机同一份装置算（把本机私有副本送过去，第二台 E184_BINARY 指到送到的文件），再取回重并" >&2
}

# --print-commands：逐件打出会起的完整命令（内存包装、线程、nice、装置二进制与参数），不起装置、不建目录。
print_commands() {
  local seg kind binary cap threads
  for seg in "${SEGMENTS[@]}"; do
    kind=$(segment_binary_kind "$seg")
    if [[ "$kind" == none ]]; then printf '%s\t（脚本自己读仓：python3 -I）\n' "$seg"; continue; fi
    binary="$MAIN_BINARY"; [[ "$kind" == dump ]] && binary="$DUMP_BINARY"
    cap=$(segment_cap "$seg"); threads=$(segment_threads "$seg")
    segment_command "$seg"
    printf '%s\tbash scripts/hosts/capped.sh %s env E184_THREADS=%s bash %s %s nice -n 19 %s %s\n' "$seg" "$threads" "$threads" "$MEMORY_WRAPPER" "$cap" "$binary" "${COMMAND[*]}"
  done
}

# --list-segments：台数、再逐件列。
list_segments() {
  local seg state hosts_text prerequisites kind
  if hosts_text=$(host_count); then :; else hosts_text="读不到（$HOSTS_EMITTER --emit-hosts 判不过）"; fi
  echo "台数 $hosts_text（--emit-hosts 读；本脚本只管一台，第二台照文件头「多机的读法」起）"
  echo "这台线程上限 $THREAD_LIMIT；工作目录 $WORK_DIRECTORY"
  for seg in "${SEGMENTS[@]}"; do
    kind=$(segment_binary_kind "$seg")
    if [[ "$seg" == repo-reads ]]; then state="每次重读"
    elif [[ "$seg" == library ]]; then
      if segment_is_done library; then state="做完"; elif segment_is_done share; then library_is_needed && state="没做完" || state="不需要（share 没有判部分的族）"; else state="等 share"; fi
    elif segment_is_done "$seg"; then state="做完"
    else state="没做完"; fi
    prerequisites=$(segment_prerequisites "$seg" | tr '\n' ',' | sed 's/,$//')
    if [[ "$kind" == none ]]; then
      printf '%s\t%s\t内存上限 -\t线程 -\t前提 %s\n' "$seg" "$state" "${prerequisites:--}"
    else
      printf '%s\t%s\t内存上限 %s\t线程 %s\t前提 %s\n' "$seg" "$state" "$(segment_cap "$seg")" "$(segment_threads "$seg")" "${prerequisites:--}"
    fi
  done
}

# ───────────────────────────── 自证 ─────────────────────────────

# 自证：假装置（主装置与转储 bin 各一份）、假内存包装、假线程包装、假机器清单，每一格在自己的临时树里从头搭，格与格不共用状态。
# 换不掉的一处：产物名里的日期用 TZ=Asia/Tokyo date +%F 取，自证里同样取，跨零点的那一刻会误红，重跑即可。
make_selftest_fakes() {
  local fakes="$1"
  mkdir -p -- "$fakes"
  cat > "$fakes/device" <<'FAKE'
#!/usr/bin/env bash
# 假装置：按子命令造文件、打 E7RESULT 行；E184_SELFTEST_DEVICE 选一种坏法。
kind=main
[[ "$(basename "$0")" == *_write_dump ]] && kind=dump
echo "device $kind $* | threads=${E184_THREADS:-unset}" >> "$E184_SELFTEST_CALLS"
self_sha=$(sha256sum -- "$0" | cut -d' ' -f1)
echo "$kind $self_sha" >> "$E184_SELFTEST_CALLS.self-sha"
mode="${E184_SELFTEST_DEVICE:-ok}"
lines=()
close() { printf '%s\n' "${lines[@]}"; echo "E7RESULT name=$1 $2 emitted=$(( ${#lines[@]} + 1 ))"; }
option() { local flag="$1" index; shift; for ((index = 1; index <= $#; index++)); do [[ "${!index}" == "$flag" ]] && { next=$((index + 1)); echo "${!next}"; return; }; done; }
case "$1" in
  selfcheck)
    case "$mode" in
      stop-selfcheck) echo "E7RESULT name=stop_check item=T1 result=fail detail=selftest"; exit 3 ;;
      fail) echo "boom" >&2; exit 1 ;;
      preflight-missing) echo "python3: can't open file '/nowhere/preflight.py': [Errno 2] No such file or directory" >&2; exit 1 ;;
      unclosed) echo "E7RESULT name=stop_check item=T1 result=pass detail=selftest"; echo "E7RESULT name=selfcheck_done stop_checks=1 failed=0 emitted=99"; exit 0 ;;
    esac
    lines+=("E7RESULT name=stop_check item=T1 result=pass detail=selftest")
    close selfcheck_done "stop_checks=1 failed=0" ;;
  record-fixed-chains)
    out=$(option --out "$@"); mkdir -p "$out"; echo dump > "$out/overwrite_chain-u01.dump"; echo dump > "$out/smoke-a.dump"
    if [[ "$mode" == control-fail ]]; then echo "E7RESULT name=control kind=dump_round_trip flow=x passed=false"; exit 4; fi
    lines+=("E7RESULT name=chain_recorded label=overwrite_chain-u01")
    close chains_done "recorded=2 skipped=0 failed=0 controls_failed=0" ;;
  dump-random-histories)
    out=$(option --out "$@"); mkdir -p "$out"
    for seed in 00 01 02 03; do echo "dump by $self_sha" > "$out/random_history-seed$seed.dump"; lines+=("E7RESULT name=dump label=seed$seed"); done
    close dump_done "histories=4 recorded=4 skipped=0 failed=0 base_mismatches=0" ;;
  keys)
    dump=$(option --dump "$@"); out=$(option --out "$@"); mkdir -p "$out"
    echo "keys-input $(ls "$dump" | tr '\n' ' ')" >> "$E184_SELFTEST_CALLS"
    flows=0
    for file in "$dump"/*.dump; do
      label=$(basename "$file" .dump); flows=$((flows + 1))
      [[ -e "$out/$label.keys" ]] || { echo "keys by $self_sha" > "$out/$label.keys"; echo "keys by $self_sha" > "$out/$label.quick.keys"; }
      lines+=("E7RESULT name=flow label=$label")
    done
    close keys_done "flows=$flows failed=0 controls_failed=0 count_disagreements=0" ;;
  share)
    verdict=independent; [[ "$mode" == partial-share ]] && verdict=partial
    lines+=("E7RESULT name=share_group scope=family:random_history_main flows=4 verdict=$verdict")
    close share_done "candidates=4 against=4 controls_failed=0" ;;
  library)
    lines+=("E7RESULT name=library_share flow=x blocks=1 in_library=0 fraction=0.000000")
    close library_done "flows=1 recorded_blocks=0" ;;
esac
FAKE
  cat > "$fakes/capped.sh" <<'FAKE'
#!/usr/bin/env bash
echo "capped $1" >> "$E184_SELFTEST_CALLS"
# E184_SELFTEST_SWAP_TARGET：起装置之前（脚本已拷过私有副本）把这个路径的装置换成另一份字节不同的，像别的会话的 cargo 把仓根那份换掉。
if [[ -n "${E184_SELFTEST_SWAP_TARGET:-}" && "$*" == *"nice -n 19"* && ! -e "$E184_SELFTEST_SWAP_TARGET.swapped" ]]; then
  { cat -- "$E184_SELFTEST_SWAP_TARGET"; echo "# swapped by a build while the long run is going"; } > "$E184_SELFTEST_SWAP_TARGET.swap" \
    && chmod +x "$E184_SELFTEST_SWAP_TARGET.swap" && mv -f -- "$E184_SELFTEST_SWAP_TARGET.swap" "$E184_SELFTEST_SWAP_TARGET" && : > "$E184_SELFTEST_SWAP_TARGET.swapped"
fi
shift
exec "$@"
FAKE
  printf '%s\n' '#!/usr/bin/env bash' 'echo "cap $1" >> "$E184_SELFTEST_CALLS"' 'shift' '"$@"; code=$?' '[[ -n "${E184_SELFTEST_WRAPPER_EXIT:-}" ]] && exit "$E184_SELFTEST_WRAPPER_EXIT"' 'exit $code' > "$fakes/run-with-memory-cap.sh"
  cat > "$fakes/hosts" <<'FAKE'
#!/usr/bin/env bash
case "${E184_SELFTEST_HOSTS:-2}" in
  1) echo "双机：关"; exit 3 ;;
  *) echo "MULTI_HOST_COUNT=$((E184_SELFTEST_HOSTS - 1))"; exit 0 ;;
esac
FAKE
  cat > "$fakes/cargo" <<'FAKE'
#!/usr/bin/env bash
echo "cargo $*" >> "$E184_SELFTEST_CALLS"
mkdir -p target/release
cp "$E184_SELFTEST_DEVICE_SOURCE" target/release/e184_regular_layer_cost_counts
cp "$E184_SELFTEST_DEVICE_SOURCE" target/release/e184_regular_layer_cost_counts_write_dump
exit 0
FAKE
  chmod +x "$fakes/device" "$fakes/capped.sh" "$fakes/run-with-memory-cap.sh" "$fakes/hosts" "$fakes/cargo"
}

# 一格的树：本脚本的拷贝、假的 scripts/hosts 两件与线程取法、指向真仓 .claude 的符号链接、假装置两份、钉住常量的一个测试夹具、
# 停机 T6 要读的两处空源文件。
make_selftest_tree() {
  local cell="$1" fakes="$2" real_root="$3" tree="$1/tree"
  mkdir -p -- "$tree/scripts/experiments/long-run-lib" "$tree/scripts/hosts" "$tree/scripts/crash" "$tree/research/results" "$cell/bin" "$cell/work" \
    "$tree/crates/singlefs-checker-tier/src" "$tree/crates/singlefs-harness/src" "$tree/crates/singlefs-checker-tier/tests"
  cp -- "$SCRIPT_PATH" "$tree/scripts/experiments/e184-segment-one-long-run.sh"
  cp -- "$(dirname -- "$SCRIPT_PATH")/long-run-lib/binary-identity.sh" "$tree/scripts/experiments/long-run-lib/"
  cp -- "$fakes/capped.sh" "$fakes/run-with-memory-cap.sh" "$real_root/scripts/hosts/heavy-threads.sh" "$tree/scripts/hosts/"
  cp -- "$fakes/hosts" "$tree/scripts/crash/crash-point-replay-shard-configuration-check.sh"
  ln -s -- "$real_root/.claude" "$tree/.claude"
  cp -- "$fakes/device" "$cell/bin/e184_regular_layer_cost_counts"
  cp -- "$fakes/device" "$cell/bin/e184_regular_layer_cost_counts_write_dump"
  echo "// 夹具：没有 T6 要找的词" > "$tree/crates/singlefs-checker-tier/src/crash_injection.rs"
  echo "// 夹具" > "$tree/crates/singlefs-harness/src/lib.rs"
  printf '%s\n' 'const SEGMENT_SIZES: [usize; 2] = [2, 3];' 'const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 11;' \
    > "$tree/crates/singlefs-checker-tier/tests/crash_enumeration_lifecycle_closure_stream.rs"
}

run_selftest() {
  local work fakes real_root today cell exit_code failures=0 cell_count=0 stem product bad_share
  local peer_cell run_zero_exit run_one_exit old_sha original_sha swap_target import_main import_peer
  unset E184_BINARY E184_DUMP_BINARY E184_WORK_DIRECTORY E184_THREADS E184_DUMP_MEMORY_CAP E184_KEYS_MEMORY_CAP E184_LIBRARY E184_PRODUCT_DIRECTORY E184_HOSTS_EMITTER E184_MEMORY_WRAPPER
  real_root="$PWD"
  today=$(TZ=Asia/Tokyo date +%F)
  work=$(mktemp -d) || return 1
  fakes="$work/fakes"
  echo e184-segment-one-selftest > "$work/mark"
  make_selftest_fakes "$fakes"
  stem="research/results/e184-regular-layer-cost-counts-segment-one"
  # 在这一格的树里起脚本拷贝；这一格要设的环境变量写在 invoke 前面（线程默认设成 7，CELL_THREADS 设成空串就不设）。
  invoke() {
    local cell="$1"
    shift
    ( cd -- "$cell/tree" && env PATH="$fakes:$PATH" E184_SELFTEST_CALLS="$cell/calls" E184_SELFTEST_DEVICE_SOURCE="$fakes/device" \
        E184_BINARY="${INVOKE_BINARY-$cell/bin/e184_regular_layer_cost_counts}" E184_WORK_DIRECTORY="$cell/work" E184_THREADS="${CELL_THREADS-7}" \
        E184_SEGMENT_ONE_SELFTEST_MARK="$work/mark" \
        bash "$cell/tree/scripts/experiments/e184-segment-one-long-run.sh" "$@" > "$cell/out" 2>&1 )
  }
  count_calls() {
    local count
    count=$(grep -c -- "$2" "$1/calls" 2> /dev/null) || true
    echo "${count:-0}"
  }
  count_files() {
    find "$1" -type f 2> /dev/null | wc -l
  }
  expect() {
    local name="$1" condition="$2"
    cell_count=$((cell_count + 1))
    if eval "$condition"; then
      echo "  ✓ $name（第 $cell_count 格）"
    else
      echo "  ✗ $name：条件「$condition」不成立"
      [[ -f "$cell/out" ]] && tail -n 6 -- "$cell/out" | sed 's/^/      输出：/'
      echo "     → 怎么办：照上面这一格点名的条件和输出改本脚本，再单独起 bash scripts/experiments/e184-segment-one-long-run.sh --selftest 到它退 0"
      failures=$((failures + 1))
    fi
  }
  new_cell() {
    cell="$work/$1"
    make_selftest_tree "$cell" "$fakes" "$real_root"
  }

  new_cell fresh
  invoke "$cell"; exit_code=$?
  product="$cell/tree/$stem-$today.out"
  expect "单机全新：十七件现跑、library 不跑、产物按带日期的名字落进 research/results" '((exit_code == 0)) && grep -qF "现跑 17、沿用 0" "$cell/out" && [[ -f "$product" ]] && is_complete_product "$product" && (($(grep -c "^E7RESULT name=segment segment=" "$product") == 18)) && grep -qF "segment=library state=not_run" "$product" && (($(count_calls "$cell" "^device main selfcheck") == 1)) && (($(count_calls "$cell" "^device dump dump-random-histories") == 6)) && (($(count_calls "$cell" "^device main keys") == 7)) && (($(count_calls "$cell" "^device main share") == 1)) && (($(count_calls "$cell" "^device main library") == 0))'
  expect "share 没有判部分的族：library 不跑" '(($(count_calls "$cell" "^device main library") == 0)) && grep -qF "name=share_group scope=family:random_history_main" "$product"'
  expect "每档的内存上限与线程：100 步档块键 24G、8 取 7；400 步档块键 30G、3；50 步档块键取这台上限 7" 'grep -qE "^device main keys --dump .*random-broad-4g-s400 .*\| threads=3$" "$cell/calls" && grep -qE "^device main keys --dump .*random-broad-4g-s100 .*\| threads=7$" "$cell/calls" && grep -qx "cap 30G" "$cell/calls" && grep -qx "cap 24G" "$cell/calls" && grep -qE "^device main keys --dump .*random-broad-4g-s050 .*\| threads=7$" "$cell/calls"'
  expect "转储每一档都带 --check-determinism" '(($(count_calls "$cell" "^device dump dump-random-histories .*--check-determinism") == 6))'
  invoke "$cell"; exit_code=$?
  expect "原样重起：做完的件全部沿用（只有读仓里的读数重读）、产物另存 -r2" '((exit_code == 0)) && grep -qF "现跑 1、沿用 16" "$cell/out" && [[ -f "$cell/tree/$stem-$today-r2.out" ]] && (($(count_calls "$cell" "^device main selfcheck") == 1)) && (($(count_calls "$cell" "^device dump") == 6))'
  new_cell name-taken
  echo old-product > "$cell/tree/$stem-$today.out"
  invoke "$cell"; exit_code=$?
  expect "产物本名已有：另存加 -r2" '((exit_code == 0)) && [[ "$(cat -- "$cell/tree/$stem-$today.out")" == old-product ]] && is_complete_product "$cell/tree/$stem-$today-r2.out"'
  new_cell binary-changed
  invoke "$cell"; run_zero_exit=$?
  old_sha=$(sha256sum -- "$cell/bin/e184_regular_layer_cost_counts" | cut -d' ' -f1)
  echo "# changed" >> "$cell/bin/e184_regular_layer_cost_counts"
  invoke "$cell"; exit_code=$?
  expect "换了装置二进制：做完的件全部重做（主装置那几件），转储 bin 没变的沿用" '((run_zero_exit == 0 && exit_code == 0)) && (($(count_calls "$cell" "^device main selfcheck") == 2)) && (($(count_calls "$cell" "^device dump") == 6)) && (($(count_calls "$cell" "^device main keys") == 14))'
  expect "换了装置二进制：旧装置打的主装置那十件逐件报作废、旧的整份留在 void 目录、装置自己写的块键目录不留给新装置、转储 bin 那六件不作废" '(($(grep -c "^作废：.*（装置 sha256 不同）" "$cell/out") == 10)) && grep -q "^作废：selfcheck（装置 sha256 不同）" "$cell/out" && grep -q "^作废：keys-random-broad-4g-s400（装置 sha256 不同）" "$cell/out" && ! grep -q "^作废：dump-" "$cell/out" && (($(find "$cell/work" -path "*/void-*/selfcheck/selfcheck.status" | wc -l) == 1)) && (($(grep -rl "$old_sha" "$cell/work/keys" | wc -l) == 0)) && (($(grep -rl "dump by $old_sha" "$cell/work/dumps" | wc -l) >= 1))'
  new_cell dump-binary-changed
  invoke "$cell"; run_zero_exit=$?
  echo "# changed" >> "$cell/bin/e184_regular_layer_cost_counts_write_dump"
  invoke "$cell"; exit_code=$?
  expect "只换了转储 bin：转储六件作废，吃它们输出的块键六件跟着作废（前提作废了），别的件沿用" '((run_zero_exit == 0 && exit_code == 0)) && (($(grep -c "^作废：dump-.*（装置 sha256 不同）" "$cell/out") == 6)) && (($(grep -c "^作废：keys-random-.*（前提 dump-.* 作废了" "$cell/out") == 6)) && (($(count_calls "$cell" "^device dump") == 12)) && (($(count_calls "$cell" "^device main keys") == 13)) && (($(count_calls "$cell" "^device main selfcheck") == 1))'
  new_cell swap
  swap_target="$cell/bin/e184_regular_layer_cost_counts"
  original_sha=$(sha256sum -- "$swap_target" | cut -d' ' -f1)
  E184_SELFTEST_SWAP_TARGET="$swap_target" invoke "$cell"; exit_code=$?
  expect "装置在一趟里不许换：起装置之前仓根那份被换掉，之后起的每一件自报的 sha256 与完成记录里的、块键文件里的都是开跑时那份" '((exit_code == 0)) && [[ -e "$swap_target.swapped" ]] && [[ "$(sha256sum -- "$swap_target" | cut -d" " -f1)" != "$original_sha" ]] && (($(wc -l < "$cell/calls.self-sha") >= 16)) && [[ "$(awk "{print \$2}" "$cell/calls.self-sha" | sort -u)" == "$original_sha" ]] && [[ "$(grep -h -o "binary_sha256=[0-9a-f]\{64\}" "$cell"/work/steps/*.status | sort -u)" == "binary_sha256=$original_sha" ]] && grep -rqF "keys by $original_sha" "$cell/work/keys"'
  new_cell import-main
  import_main="$cell"
  new_cell import-peer
  import_peer="$cell"
  invoke "$import_peer" --share 1/2; run_zero_exit=$?
  printf '%s\n' "$(printf 'f%.0s' {1..64})" > "$import_peer/work/keys/random-broad-4g-s050.device-stamp"
  invoke "$import_main" --import-work "$import_peer/work"; exit_code=$?
  cell="$import_main"
  expect "--import-work 对方的装置 sha256 不同：这一档的块键文件跳过并报作废，别的档照并" '((run_zero_exit == 0 && exit_code == 0)) && grep -q "^作废：keys/random-broad-4g-s050/random_history-seed01.keys（装置 sha256 不同）" "$cell/out" && [[ ! -e "$cell/work/keys/random-broad-4g-s050/random_history-seed01.keys" ]] && [[ -f "$cell/work/keys/random-broad-4g-s100/random_history-seed01.keys" ]] && [[ -f "$cell/work/keys/random-broad-4g-s100.device-stamp" ]]'
  new_cell assemble-mixed
  invoke "$cell"; run_zero_exit=$?
  sed "s/binary_sha256=[0-9a-f]*/binary_sha256=$(printf 'f%.0s' {1..64})/" "$cell/work/steps/selfcheck.status" > "$cell/work/steps/selfcheck.status.other" \
    && mv -- "$cell/work/steps/selfcheck.status.other" "$cell/work/steps/selfcheck.status"
  INVOKE_BINARY="$cell/bin/not-built" invoke "$cell" --assemble; exit_code=$?
  expect "拼产物之前各件记的装置 sha256 彼此不同：拒、退 1、不另出产物" '((run_zero_exit == 0 && exit_code == 1)) && grep -qF "各件是用不同的装置二进制做的" "$cell/out" && (($(ls "$cell/tree/research/results" | wc -l) == 1))'
  new_cell only
  invoke "$cell" --only dump-random-broad-4g-s050; exit_code=$?
  expect "--only 点名的一件现跑、其余不碰、不拼产物" '((exit_code == 0)) && (($(count_calls "$cell" "^device") == 1)) && grep -q "^device dump dump-random-histories --steps 50 " "$cell/calls" && (($(count_files "$cell/tree/research/results") == 0))'
  invoke "$cell" --only nonsense; exit_code=$?
  expect "--only 点名的件不存在退 2" '((exit_code == 2)) && grep -qF "认不得的件" "$cell/out"'
  new_cell only-prerequisite
  invoke "$cell" --only keys-random-broad-4g-s050; exit_code=$?
  expect "--only 点名的件前面的件没做完退 2、装置一次没起" '((exit_code == 2)) && grep -qF "前面的件没做完" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  new_cell list
  E184_SELFTEST_HOSTS=2 invoke "$cell" --list-segments; exit_code=$?
  expect "--list-segments：读台数、逐件列状态与内存上限线程、不起装置" '((exit_code == 0)) && grep -qF "台数 2" "$cell/out" && (($(grep -c "没做完\|每次重读\|等 share" "$cell/out") >= 16)) && grep -qP "^keys-random-broad-4g-s400\t没做完\t内存上限 30G\t线程 3\t" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  E184_SELFTEST_HOSTS=1 invoke "$cell" --list-segments; exit_code=$?
  expect "--list-segments：双机关着只有本机一台" '((exit_code == 0)) && grep -qF "台数 1" "$cell/out"'
  E184_KEYS_MEMORY_CAP=12G E184_DUMP_MEMORY_CAP=9G invoke "$cell" --list-segments; exit_code=$?
  expect "E184_KEYS_MEMORY_CAP 与 E184_DUMP_MEMORY_CAP 覆盖表里的上限" '((exit_code == 0)) && grep -qP "^keys-random-broad-4g-s400\t.*内存上限 12G" "$cell/out" && grep -qP "^dump-random-broad-4g-s400\t.*内存上限 9G" "$cell/out"'
  CELL_THREADS= invoke "$cell" --list-segments; exit_code=$?
  expect "E184_THREADS 不设：线程上限取登记值（.claude/heavy-thread-default）" '((exit_code == 0)) && grep -qF "这台线程上限 $(heavy_thread_default)" "$cell/out"'
  new_cell print
  invoke "$cell" --print-commands; exit_code=$?
  expect "--print-commands：逐件打出命令、线程与内存上限落在命令里、不起装置、不建目录" '((exit_code == 0)) && grep -qP "^keys-random-broad-4g-s400\tbash scripts/hosts/capped.sh 3 env E184_THREADS=3 bash scripts/hosts/run-with-memory-cap.sh 30G nice -n 19 \S+ keys --dump \S+/dumps/random-broad-4g-s400 --out \S+/keys/random-broad-4g-s400 --controls$" "$cell/out" && grep -qP "^share\t.* share --candidates \S+/keys/fixed --candidates \S+/keys/random-broad-4g-s050 .* --against \S+/keys/fixed --out \S+/steps/share.file --pinned-smoke \S+/pinned-smoke.txt$" "$cell/out" && [[ ! -e "$cell/calls" && ! -d "$cell/work/steps" ]]'
  invoke "$cell" --print-commands --share 1/2; exit_code=$?
  expect "--print-commands --share 1/2：块键那几件读链接目录、链接目录不建" '((exit_code == 0)) && grep -qP "^keys-random-broad-4g-s050\t.* keys --dump \S+/share-links/random-broad-4g-s050-1-of-2 --out " "$cell/out" && [[ ! -d "$cell/work/share-links" ]]'
  new_cell main-host
  local main_cell="$cell"
  new_cell peer-host
  peer_cell="$cell"
  cell="$main_cell"
  invoke "$main_cell" --share 0/2; run_zero_exit=$?
  invoke "$peer_cell" --share 1/2; run_one_exit=$?
  expect "--share 0/2、1/2：只转储与算各自那一半流的块键、不拼产物、块键清单不写完成记录" '((run_zero_exit == 0 && run_one_exit == 0)) && grep -qE "^keys-input random_history-seed00.dump random_history-seed02.dump $" "$main_cell/calls" && grep -qE "^keys-input random_history-seed01.dump random_history-seed03.dump $" "$peer_cell/calls" && (($(count_files "$main_cell/tree/research/results") == 0)) && [[ ! -e "$main_cell/work/steps/keys-random-broad-4g-s050.status" && -e "$main_cell/work/steps/dump-random-broad-4g-s050.status" ]]'
  expect "--share 0/2 只算自己那一半流" '[[ -f "$main_cell/work/keys/random-broad-4g-s050/random_history-seed00.keys" && ! -e "$main_cell/work/keys/random-broad-4g-s050/random_history-seed01.keys" ]]'
  echo kept > "$main_cell/work/keys/random-broad-4g-s050/random_history-seed01.quick.keys"
  invoke "$main_cell" --import-work "$peer_cell/work"; exit_code=$?
  expect "--import-work 并进块键文件、已有的不覆盖、不起装置" '((exit_code == 0)) && [[ -f "$main_cell/work/keys/random-broad-4g-s050/random_history-seed01.keys" && -f "$main_cell/work/keys/random-broad-4g-s050/random_history-seed03.keys" ]] && [[ "$(cat -- "$main_cell/work/keys/random-broad-4g-s050/random_history-seed01.quick.keys")" == kept ]]'
  invoke "$main_cell"; exit_code=$?
  expect "分片、并进、再不带参数跑：块键清单打全、产物拼出" '((exit_code == 0)) && is_complete_product "$main_cell/tree/$stem-$today.out" && grep -qE "^keys-input random_history-seed00.dump random_history-seed01.dump random_history-seed02.dump random_history-seed03.dump $" "$main_cell/calls"'
  for bad_share in abc 1/ 1-2 2/2 0/0; do
    new_cell "bad-share-${bad_share//\//-slash}"
    invoke "$cell" --share "$bad_share"; exit_code=$?
    expect "--share 写坏退 2：$bad_share" '((exit_code == 2)) && grep -qF -e "--share 要写 第几份/共几份" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  done
  new_cell bad-arguments
  invoke "$cell" --share; exit_code=$?
  expect "--share 缺值退 2" '((exit_code == 2)) && grep -qF -e "--share 后面要写" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  invoke "$cell" --only; exit_code=$?
  expect "--only 缺值退 2" '((exit_code == 2)) && grep -qF -e "--only 后面要写" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  invoke "$cell" --import-work; exit_code=$?
  expect "--import-work 缺值退 2" '((exit_code == 2)) && grep -qF -e "--import-work 后面要写" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  invoke "$cell" --bogus; exit_code=$?
  expect "认不得的参数退 2" '((exit_code == 2)) && grep -qF "认不得的参数「--bogus」" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  invoke "$cell" --import-work "$cell/no-such-directory"; exit_code=$?
  expect "--import-work 目录不在退 2" '((exit_code == 2)) && grep -qF "点名的目录不在" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  new_cell selfcheck-stop
  E184_SELFTEST_DEVICE=stop-selfcheck invoke "$cell"; exit_code=$?
  expect "selfcheck 报停机退 3、不合格行转出来、后面的件不起、不拼产物" '((exit_code == 3)) && grep -qF "result=fail" "$cell/out" && grep -qF "停机条款没过" "$cell/out" && (($(count_calls "$cell" "^device") == 1)) && (($(count_files "$cell/tree/research/results") == 0)) && [[ ! -e "$cell/work/steps/selfcheck.status" ]]'
  new_cell control-fail
  E184_SELFTEST_DEVICE=control-fail invoke "$cell"; exit_code=$?
  expect "record-fixed-chains 报阳性对照没过退 4、后面的件不起" '((exit_code == 4)) && grep -qF "阳性对照没过" "$cell/out" && grep -qF "passed=false" "$cell/out" && (($(count_calls "$cell" "^device") == 2)) && (($(count_files "$cell/tree/research/results") == 0))'
  new_cell device-fail
  E184_SELFTEST_DEVICE=fail invoke "$cell"; exit_code=$?
  expect "装置退 1：退 1、说退几、不写完成记录" '((exit_code == 1)) && grep -qF "装置退 1（日志" "$cell/out" && [[ ! -e "$cell/work/steps/selfcheck.status" ]]'
  new_cell preflight-missing
  E184_SELFTEST_DEVICE=preflight-missing invoke "$cell"; exit_code=$?
  expect "装置因编译树里没有 preflight.py 退 1：给出路" '((exit_code == 1)) && grep -qF "找不到 preflight.py" "$cell/out"'
  new_cell wrapper-exit
  E184_SELFTEST_WRAPPER_EXIT=252 invoke "$cell"; exit_code=$?
  expect "内存包装排不上退 1：这一次的输出不算结果、不写完成记录" '((exit_code == 1)) && grep -qF "内存包装退 252" "$cell/out" && [[ ! -e "$cell/work/steps/selfcheck.status" ]]'
  new_cell unclosed
  E184_SELFTEST_DEVICE=unclosed invoke "$cell"; exit_code=$?
  expect "输出末行对不上行数的件不算做完" '((exit_code == 1)) && grep -qF "emitted= 对不上行数" "$cell/out" && [[ ! -e "$cell/work/steps/selfcheck.status" ]]'
  new_cell library-needed
  E184_SELFTEST_DEVICE=partial-share invoke "$cell"; exit_code=$?
  product="$cell/tree/$stem-$today.out"
  expect "share 有一族判部分：library 跑、产物里是 library 的输出" '((exit_code == 0)) && (($(count_calls "$cell" "^device main library") == 1)) && grep -qF "segment=library state=done" "$product" && grep -qF "name=library_share" "$product" && is_complete_product "$product"'
  new_cell pinned-mismatch
  printf '%s\n' 'const SEGMENT_SIZES: [usize; 2] = [2, 3];' 'const CLOSED_FORM_WITH_TWO_STATES_PER_WRITE: u64 = 12;' \
    > "$cell/tree/crates/singlefs-checker-tier/tests/crash_enumeration_lifecycle_closure_stream.rs"
  invoke "$cell"; exit_code=$?
  expect "钉住的常量与段形现算的两态闭式对不上退 3、装置一次没起" '((exit_code == 3)) && grep -qF "match=false" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  new_cell crashflow-hit
  echo "struct CrashFlow;" > "$cell/tree/crates/singlefs-harness/src/lib.rs"
  invoke "$cell"; exit_code=$?
  expect "crates 里出现 CrashFlow 退 3（停机 T6）、装置一次没起" '((exit_code == 3)) && grep -qF "T6_no_crashflow_in_crates result=fail hits=1" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  new_cell crashflow-unreadable
  rm -rf -- "$cell/tree/crates/singlefs-harness/src" "$cell/tree/crates/singlefs-checker-tier/src/crash_injection.rs"
  invoke "$cell"; exit_code=$?
  expect "停机 T6 要读的文件一个都读不到：不当零命中，退 3" '((exit_code == 3)) && grep -qF "files_read=0" "$cell/out"'
  new_cell assemble-missing
  invoke "$cell" --assemble; exit_code=$?
  expect "--assemble：件没齐退 1、点名缺的、装置一次没起" '((exit_code == 1)) && grep -qF "件还没齐" "$cell/out" && [[ ! -e "$cell/calls" ]] && (($(count_files "$cell/tree/research/results") == 0))'
  new_cell assemble
  invoke "$cell"; run_zero_exit=$?
  invoke "$cell" --assemble; exit_code=$?
  expect "--assemble：件齐了不起装置、另存 -r2" '((run_zero_exit == 0 && exit_code == 0)) && is_complete_product "$cell/tree/$stem-$today-r2.out" && (($(count_calls "$cell" "^device main selfcheck") == 1))'
  new_cell build-here
  INVOKE_BINARY= invoke "$cell" --build-here --only selfcheck; exit_code=$?
  expect "--build-here：编一次（两个 bin 一条 cargo）、用这棵树 target/release 下的装置" '((exit_code == 0)) && (($(count_calls "$cell" "^cargo build --release -p singlefs-checker-tier --bin e184_regular_layer_cost_counts --bin e184_regular_layer_cost_counts_write_dump") == 1)) && grep -q "^device main selfcheck" "$cell/calls"'
  new_cell binary-missing
  INVOKE_BINARY="$cell/bin/not-built" invoke "$cell" --only selfcheck; exit_code=$?
  expect "装置二进制没编好退 78、装置一次没起" '((exit_code == 78)) && grep -qF "装置还没编好" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  new_cell lock
  mkdir -p -- "$cell/work"
  ( exec 8> "$cell/work/.run-lock"; flock -n 8 && invoke "$cell" --only selfcheck; echo "$?" > "$cell/lock-exit" )
  exit_code=$(cat -- "$cell/lock-exit" 2> /dev/null || echo missing)
  expect "同一个工作目录已有一趟在跑：退 2、装置一次没起" '[[ "$exit_code" == 2 ]] && grep -qF "已有一趟在跑" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  new_cell registered-threads
  CELL_THREADS= invoke "$cell" --only selfcheck; exit_code=$?
  expect "E184_THREADS 不设：装置的线程取登记值" '((exit_code == 0)) && grep -qF "threads=$(heavy_thread_default)" "$cell/calls"'
  rm -rf -- "${work:?}"
  if ((failures > 0)); then echo "e184-segment-one-long-run.sh 自证：$failures 格红（共 $cell_count 格）"; return 1; fi
  echo "e184-segment-one-long-run.sh 自证：$cell_count 格全绿"
  return 0
}

# ───────────────────────────── 主流程 ─────────────────────────────

BUILD_HERE=0
SHARE=""
ONLY=""
IMPORT_DIRECTORY=""
MODE=run
MAIN_SHA=""
DUMP_SHA=""
MAIN_SOURCE=""
DUMP_SOURCE=""
selftest=0
while (($# > 0)); do
  case "$1" in
    --only) (($# >= 2)) || break_is missing-value-unchecked || usage_error "--only 后面要写件名"; ONLY="$2"; shift 2 ;;
    --share) (($# >= 2)) || break_is missing-value-unchecked || usage_error "--share 后面要写 第几份/共几份"; SHARE="$2"; shift 2 ;;
    --import-work) (($# >= 2)) || break_is missing-value-unchecked || usage_error "--import-work 后面要写目录"; IMPORT_DIRECTORY="$2"; MODE=import; shift 2 ;;
    --assemble) MODE=assemble; shift ;;
    --list-segments) MODE=list; shift ;;
    --print-commands) MODE=print; shift ;;
    --build-here) BUILD_HERE=1; shift ;;
    --selftest) selftest=1; shift ;;
    *) break_is unknown-argument-accepted && { shift; continue; }; usage_error "认不得的参数「$1」" ;;
  esac
done

if ((selftest)); then
  run_selftest
  exit $?
fi

load_segment_table
if [[ -n "$SHARE" ]] && ! break_is share-format-unchecked; then
  [[ "$SHARE" =~ ^[0-9]+/[0-9]+$ ]] || usage_error "--share 要写 第几份/共几份（两个整数），给的是「$SHARE」"
  ((${SHARE#*/} >= 1 && ${SHARE%/*} < ${SHARE#*/})) || usage_error "--share 要写 第几份/共几份（第几份从 0 起、小于共几份、共几份不小于 1），给的是「$SHARE」"
fi
if [[ -n "$ONLY" ]] && ! is_segment_name "$ONLY"; then
  usage_error "认不得的件「$ONLY」（--list-segments 列全部）"
fi

THREAD_LIMIT=$(registered_threads) || exit 2
LIBRARY_DIRECTORY="${E184_LIBRARY:-${SINGLEFS_CRASH_AMPLIFICATION_LIBRARY:-$HOME/.local/share/singlefs/crash-amplification/crash-state-verdicts}}"
RUN_STAMP="$(TZ=Asia/Tokyo date +%F)-$$"

# 列件、拼产物时二进制在就算 sha256（核做完的记录是不是同一份二进制做的）；不在就只看记录。
read_shas_if_present() {
  resolve_binaries
  if [[ -x "$MAIN_BINARY" && -x "$DUMP_BINARY" ]]; then
    MAIN_SHA=$(sha256sum -- "$MAIN_BINARY" | cut -d' ' -f1)
    DUMP_SHA=$(sha256sum -- "$DUMP_BINARY" | cut -d' ' -f1)
  fi
}

case "$MODE" in
  list)
    read_shas_if_present
    list_segments
    exit 0 ;;
  print)
    resolve_binaries
    print_commands
    exit 0 ;;
  import)
    mkdir -p -- "$WORK_DIRECTORY"
    read_shas_if_present
    import_work "$IMPORT_DIRECTORY"
    exit 0 ;;
  assemble)
    read_shas_if_present
    assemble_product
    exit $? ;;
esac

mkdir -p -- "$WORK_DIRECTORY/steps" "$WORK_DIRECTORY/logs" "$WORK_DIRECTORY/dumps" "$WORK_DIRECTORY/keys" || exit 1
exec 9> "$WORK_DIRECTORY/.run-lock"
flock -n 9 || { echo "  ✗ 这个工作目录已有一趟在跑（$WORK_DIRECTORY/.run-lock 被占着）" >&2; echo "     → 怎么办：等它跑完，或换 E184_WORK_DIRECTORY" >&2; exit 2; }
resolve_binaries
if ((BUILD_HERE)); then build_here; fi
require_binaries
echo "e184 第一段长活：这台线程上限 $THREAD_LIMIT、工作目录 $WORK_DIRECTORY、主装置 $MAIN_SOURCE（私有副本 $MAIN_BINARY，sha256 ${MAIN_SHA::12}…）、转储 bin $DUMP_SOURCE（私有副本 $DUMP_BINARY，sha256 ${DUMP_SHA::12}…）${SHARE:+、只跑 --share $SHARE 那一份}"

# 一件活前面的件都得做完（--only 点名时核；顺序跑时前面的件刚做完）。
prerequisites_are_done() {
  local seg="$1" prerequisite unmet=()
  break_is prerequisite-unchecked && return 0
  while IFS= read -r prerequisite; do
    [[ -n "$prerequisite" ]] || continue
    segment_is_done "$prerequisite" || unmet+=("$prerequisite")
  done < <(segment_prerequisites "$seg")
  if ((${#unmet[@]} > 0)); then
    echo "  ✗ 件 $seg 前面的件没做完：${unmet[*]}" >&2
    return 1
  fi
  return 0
}

void_stale_segments
((VOIDED_COUNT == 0)) || echo "作废了 $VOIDED_COUNT 件（装置 sha256 不同、或前提作废了）：旧的整份在 $WORK_DIRECTORY/void-$RUN_STAMP/"
ran=0
reused=0
for seg in "${SEGMENTS[@]}"; do
  [[ -z "$ONLY" || "$seg" == "$ONLY" ]] || continue
  if [[ -n "$SHARE" ]]; then
    case "$seg" in dump-* | keys-random-*) ;; *) continue ;; esac
  fi
  if [[ "$seg" == library && "$ONLY" != library ]]; then
    segment_is_done share && library_is_needed || continue
  fi
  if [[ "$seg" != repo-reads ]] && segment_is_done "$seg"; then
    echo "件 $seg：已做完，沿用"
    reused=$((reused + 1))
    continue
  fi
  if ! prerequisites_are_done "$seg"; then
    echo "     → 怎么办：先做完它前面的件（不带 --only 重起本脚本会按次序做）" >&2
    exit 2
  fi
  run_segment "$seg"
  segment_exit=$?
  case "$segment_exit" in
    0) ran=$((ran + 1)) ;;
    3) echo "  ✗ 该停下交回主 agent，不重跑（日志 $WORK_DIRECTORY/logs）" >&2; exit 3 ;;
    4) echo "  ✗ 阳性对照没过，这一段作废，修装置后重跑" >&2; exit 4 ;;
    *) echo "  ✗ 件 $seg 没做完；同一条命令重起续跑（做完的件沿用，装置再跳过已写完的转储与块键文件）" >&2; exit 1 ;;
  esac
done
echo "件做完：现跑 $ran、沿用 $reused"
if [[ -n "$SHARE" || -n "$ONLY" ]]; then
  exit 0
fi
assemble_product
exit $?
