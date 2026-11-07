#!/usr/bin/env bash
# admission: always 整个脚本每次都起：哪一片已有判定、键对不对得上由 prune-server 里的按片判定现答（scripts/experiment-apparatus/long_run_item_verdicts.py），上一次跑完不替这一次作保
# run-condition: command cargo git python3
# E181（落得下那一判与挂载下界）第一段（固定点与落得下段，跑前登记 research/prompts/e181-preregistration.md 5.5 节第一段）的长跑跑法：
#   装置是入库装置 crates/singlefs-verification/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs；装置按「段」（命令行参数）一段一段跑，
#   本脚本把它们切成八个互不依赖的片，每片一个进程、一份输出，全部判完再按固定次序拼成产物
#   research/results/e181-landing-check-fixed-point-and-mount-lower-bound-landing.out（多段产物，每段有自己的 name=done 收尾行），
#   第一片（常量回比、锚点、自证、小几何的阳性对照）另存一份给复跑登记用的产物 …-checks.out（逐字节可比，不含计时与线程数）。
#
# long-run-shards: 一片 = 装置的一段输出：checks、f11、f9、measure、controls、narrow、unequal、small 八片；每片一个进程，--only 点名哪几片，两台机器可以各判一部分片
# long-run-resume: 每片一条判定，在 prune-server 的 apparatus-items 族、种类 e181_landing_check_fixed_point_and_mount_lower_bound（读写经 scripts/experiment-apparatus/long_run_item_verdicts.py，
#   它文件头「判定库」）：键 = 装置入口 main 的单元闭包（release 档；admission.py 的 compute_unit_key 算）、工具链、闭包真引到的依赖版本、这一片点名的段；
#   checks 那一片产物逐字节可比（compare=exact），别的片带计时与线程数（compare=timing，只比绿红与退出码）。键相同、库里判绿的片取库里的片产物、不起装置；
#   判红的照同一把键记红撤绿、下一趟现判。同一条命令重起就是续跑，成功那句报「现判 N、复用 M、拒算 R」。本地不留片文件、不读旧片文件（旧产物不导入）。
# long-run-hosts: 两台各判一部分片：主 agent 用 --only 点名交统一任务服务（派哪台由服务定），判完的片都写进 prune-server 那一份，不拷、不取回；
#   之后在哪一台起一次不带参数的本脚本：判完的片复用、拼出产物。台数与每台线程不在这里读（并发数照下面的内存预算算）。
#
# 片与段、并发数（并发数显式给：一份试写的拷贝约占 5 GiB，一个走种子的线程约占 2 GiB 以上，装置里的默认是每台默认重活线程数，会撞顶；
#   脚本里不套内存包装，整条命令交 task-server，内存额度由服务按记录给；下面的内存预算只用来算并发数，并发数不进键）：
#   checks   constants anchors selfcheck f10-controls-small
#   f11      f11                                              （计时片：开跑前机器要闲，跑的时候不和别的片并行）
#   f9       f9                                               预算 E181_CAP_F9（默认 16G），同时试写的拷贝数 (预算 − 4) ÷ 5 取整
#   measure  f10-measure（G默认上量 F10 的 10 步，登记 5.6 节）      （填满 G默认 的阶梯第一档是 10 万个单元，内容加两块盘的层约 10 GiB 以上，推的）
#   controls f10-controls（G窄 的阳性对照）
#   narrow   f10-narrow                                       预算 E181_CAP_F10（默认 24G），走种子的线程数 (预算 − 8) ÷ 4 取整，E181_WALKERS 给了照它
#   unequal  f10-unequal                                      同上
#   small    f10-small                                        同上
# measure 片和别的片一样一律跑完、不设外推停机（挂钟由统一任务服务管）；外推读数在 measure 片的输出里（name=f10_measure_extrapolation）。
# 装置在一趟里不许换（共用段落在 long-run-lib/binary-identity.sh）：开跑时把装置二进制拷一份进 $E181_LOG_DIRECTORY/pinned-binary/<sha256>/（只读），
#   八片都按这份拷贝起，不再 exec 开跑时给的那个路径（别的会话的 cargo 随时会把它换掉）；跑着的时候源码被改了，long_run_item_verdicts.py 的 finish 重算键、
#   撤掉这一趟写进库的那几片、判红。老的私有副本留着不删（一个 sha256 一个目录），确认不再用那一版之后手删。
#
# 用法（在仓里任何目录起都行，脚本自己回到仓根）：
#   bash scripts/experiment-apparatus/e181-landing-check-long-run.sh                      判全部片（判完的复用）并拼产物
#   bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --only <片名>[,<片名>…]   只判点名的片，不拼产物（分给第二台用）
#   bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --list-segments      列八片和它们的段，不编译、不跑、不连 prune-server
#   bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --selftest           假装置在临时树里走一遍（不起 cargo、不碰真装置）；判定写进临时 prune-server：
#     只在临时服务底下跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem；不在就退 2、一格都不跑），起法：
#     python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --selftest
# 环境：E181_PRODUCT_DIRECTORY（产物目录，默认 research/results；自证用）、E181_BINARY（装置二进制；没给就在要起装置时先 cargo build --release 编，装置取 cargo 自己报的这一趟编出来的位置
#   ——编译目录由 task-server 给、经 CARGO_TARGET_DIR 交下来，仓根 target/ 下那一份可能是早先编的旧二进制，不取它；取法在 long-run-lib/binary-identity.sh 的 binary_identity_build）、
#   E181_LOG_DIRECTORY（装置私有副本与这一趟的状态目录，默认工作根下的 e181-long-run，工作根取 .claude/hooks/lib_work_root.py、两台同一份）、
#   E181_CAP_F9、E181_CAP_F10、E181_PROBE_COUNT（f9 同时试写的拷贝数）、E181_WALKERS（走种子的线程数）。
# 产物名：…-landing.out 已有时另存 …-landing-<日期>.out，同一天再跑加 -r2、-r3，已有的一个不动；…-checks.out 同样。
# 退出码：0 做完；1 有片判红、产物不齐、复用哨兵红或键在跑的中途变了；2 用法错；3 装置报停机（日志里 name=stop 那一行）；
#   75 prune-server 不可用；78 装置没编好（--list-segments 不需要）。
#
# 复用三问（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：自证经 long_run_item_verdicts.py reuse-cells 跑那六格（reuse-three-questions、
#   key-independent-of-directory、reuse-sentinel 与哨兵的三种红），假驱动是自证的假装置，checks、f11 两片的装置入口换成假树里两个 Python 函数（它文件头「复用格」）。
# 弄坏开关 E181_LONG_RUN_BREAK=<项，逗号分隔>（只在 --selftest 起的那几趟里认：自证设 E181_LONG_RUN_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   E181_LONG_RUN_BREAK=exec-root-binary  每片仍按开跑时给的那个路径起、不按私有副本
#       「装置在一趟里不许换：跑着的时候仓根那份被换掉，之后起的八片自报的 sha256 都是开跑时那份」
#   E181_LONG_RUN_BREAK=done-marker-ignored  片输出末行不齐也当做完
#       「末行不齐的片判红：退 1、头一片记红、别的没跑到、不拼产物」
#   E181_LONG_RUN_BREAK=binary-from-root-target  没给 E181_BINARY 时编完取仓根 target/release 下那一份，不取 cargo 报的这一趟编出来的位置
#       「没给 E181_BINARY：装置取 cargo 报的这一趟编出来的那一份、不取仓根 target/release 下早先编的那一份」
#   E181_LONG_RUN_BREAK=reuse-ignored  库里判绿的片也照起装置现判（不认 plan 答的 reuse）
#       「原样重起：八片全复用、装置一次没起、产物另存带日期的」「--only 点名的片判完不拼产物；再不带参数起，那两片复用、其余现判、拼出产物」
#       「reuse-three-questions」「key-independent-of-directory」「reuse-sentinel」「forced-still-reused-red」
#   E181_LONG_RUN_BREAK=finish-ignored  不认 long_run_item_verdicts.py finish 的退出码（复用哨兵红、键中途变了照退 0）
#       「sentinel-rejudged-red」「forced-verdict-differs-red」「forced-still-reused-red」
# 复用那几格另认 long_run_item_verdicts.py 的弄坏开关 LONG_RUN_ITEMS_BREAK（它文件头；打开后本脚本的自证红在它点名的同名格）。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "$(dirname "${BASH_SOURCE[0]}")/long-run-lib/binary-identity.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; echo "     → 怎么办：本脚本得留在仓里的 scripts/experiment-apparatus/ 下（它按自己的位置找仓根）；挪走了就放回去再起" >&2; exit 2; }

SCRIPT_KEY="scripts/experiment-apparatus/e181-landing-check-long-run.sh"
ITEM_VERDICTS="scripts/experiment-apparatus/long_run_item_verdicts.py"   # 按片判定的读写（文件头 long-run-resume）
PRODUCT_DIRECTORY="${E181_PRODUCT_DIRECTORY:-research/results}"
LANDING_STEM="$PRODUCT_DIRECTORY/e181-landing-check-fixed-point-and-mount-lower-bound-landing"
CHECKS_STEM="$PRODUCT_DIRECTORY/e181-landing-check-fixed-point-and-mount-lower-bound-checks"
BINARY_NAME="e181_landing_check_fixed_point_and_mount_lower_bound"
APPARATUS_ENTRY="crates/singlefs-verification/src/bin/$BINARY_NAME.rs::main"
BINARY="${E181_BINARY:-}"   # 没给：build_the_binary 照 cargo 报的这一趟编出来的位置填
LOG_DIRECTORY="${E181_LOG_DIRECTORY:-}"
if [[ -z "$LOG_DIRECTORY" ]]; then   # 没设：工作根（.claude/hooks/lib_work_root.py，两台同一份）下的 e181-long-run；自证的临时树各格都给了目录，不走这里
  if ! LOG_DIRECTORY="$(python3 .claude/hooks/lib_work_root.py)" || [[ "$LOG_DIRECTORY" != /* ]]; then
    echo "  ✗ 取不到工作根（.claude/hooks/lib_work_root.py 跑不起来），缺省的日志目录没处放" >&2
    echo "     → 怎么办：修好 .claude/hooks/lib_work_root.py，或设 E181_LOG_DIRECTORY 指到日志目录" >&2
    exit 2
  fi
  LOG_DIRECTORY="$LOG_DIRECTORY/e181-long-run"
fi
CAP_F9="${E181_CAP_F9:-16G}"
CAP_F10="${E181_CAP_F10:-24G}"
SEGMENT_ORDER=(checks f11 f9 measure controls narrow unequal small)

usage_error() {
  echo "  ✗ $1" >&2
  echo "     → 怎么办：见本脚本文件头「用法」" >&2
  exit 2
}

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）；别处设了就拒。
break_is() {
  [[ ",${E181_LONG_RUN_BREAK:-}," == *",$1,"* ]]
}
if [[ -n "${E181_LONG_RUN_BREAK:-}" && "${1:-}" != --selftest ]]; then
  if [[ -z "${E181_LONG_RUN_SELFTEST_MARK:-}" || "$(cat -- "${E181_LONG_RUN_SELFTEST_MARK}" 2> /dev/null)" != e181-long-run-selftest ]]; then
    echo "  ✗ E181_LONG_RUN_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
    echo "     → 怎么办：平常跑时不设它" >&2
    exit 2
  fi
fi

# 片名 → 装置的段（空格隔开）。
sections_of_segment() {
  case "$1" in
    checks) echo "constants anchors selfcheck f10-controls-small" ;;
    f11) echo "f11" ;;
    f9) echo "f9" ;;
    measure) echo "f10-measure" ;;
    controls) echo "f10-controls" ;;
    narrow) echo "f10-narrow" ;;
    unequal) echo "f10-unequal" ;;
    small) echo "f10-small" ;;
    *) return 1 ;;
  esac
}

cap_of_segment() { # 并发数的内存预算：只用来算 f9 同时试写的拷贝数与走种子的线程数，不是包装上限（内存额度由 task-server 给）；别的片不按它算
  case "$1" in
    checks | f11 | measure | controls) echo "" ;;
    f9) echo "$CAP_F9" ;;
    narrow | unequal | small) echo "$CAP_F10" ;;
    *) return 1 ;;
  esac
}

gibibytes_of() { # <上限，例 24G>
  [[ "$1" =~ ^([0-9]+)G$ ]] || usage_error "内存上限要写成 <整数>G，给的是「$1」"
  echo "${BASH_REMATCH[1]}"
}

# 同时试写的拷贝数：一份拷贝约占 5 GiB，留 4 GiB 给底与内容。
probe_count_of_cap() { # <上限>
  local gibibytes
  gibibytes=$(gibibytes_of "$1")
  local count=$(((gibibytes - 4) / 5))
  ((count >= 1)) || count=1
  echo "$count"
}

# 走种子的线程数：一个线程约占 4 GiB（小几何量到约 2.2 GiB，768 MiB 环的几何按两倍留余量），留 8 GiB 给填满数据那一侧的底。
walker_count_of_cap() { # <上限>
  local gibibytes
  gibibytes=$(gibibytes_of "$1")
  local count=$(((gibibytes - 8) / 4))
  ((count >= 1)) || count=1
  echo "$count"
}

# 一份片输出做完了：每一段的收尾行 name=done emitted=N 的 N 等于它那一段的 E7RESULT 行数，且末行就是收尾行。
is_complete_output() { # <文件>
  [[ -s "$1" ]] || return 1
  break_is done-marker-ignored && return 0
  awk '
    !/^E7RESULT / { malformed = 1 }
    /^E7RESULT / { count++ }
    /^E7RESULT name=done emitted=/ { split($0, parts, "emitted="); if (parts[2] + 0 != count) malformed = 1; count = 0; finished = 1; next }
    { finished = 0 }
    END { exit (malformed || count > 0 || !finished) ? 1 : 0 }
  ' "$1"
}

# 还没被占的产物名：先试没有日期的，已有就加日期，同一天再加 -r2、-r3……
next_free_product_path() { # <名的前段，不带 .out>
  local stem="$1" today candidate revision
  today=$(TZ=Asia/Tokyo date +%F)
  candidate="$stem.out"
  [[ -e "$candidate" ]] || { echo "$candidate"; return; }
  candidate="$stem-$today.out"
  [[ -e "$candidate" ]] || { echo "$candidate"; return; }
  revision=2
  while [[ -e "$stem-$today-r$revision.out" ]]; do revision=$((revision + 1)); done
  echo "$stem-$today-r$revision.out"
}

# 一片的判定请求（long_run_item_verdicts.py 文件头「请求文件」）：装置入口、release 档、这一片点名的段；checks 逐字节可比，别的片带计时（timing）
segment_request() { # <片名>
  local compare=timing sections
  [[ "$1" == checks ]] && compare=exact
  sections=$(sections_of_segment "$1")
  printf '%s\t%s\tprofile:release\tline:segment=%s\tline:sections=%s\tline:compare=%s\n' "$1" "$APPARATUS_ENTRY" "$1" "${sections// /,}" "$compare"
}

decision_of() { # <片名>：plan 答的 judge / reuse / refused
  awk -F'\t' -v item="$1" '$1 == item {print $2}' "$PLAN_DIRECTORY/decisions.tsv"
}

# 判定库那一步没成：75（prune-server 不可用）照交，用法错交 2，别的按 1 交（3 是「装置报停机」，不混用）
library_failure_exit() { # <哪一步> <退出码>：打原因，交该退的码
  echo "  ✗ 按片判定的 $1 没成（long_run_item_verdicts.py 退 $2）" >&2
  echo "     → 怎么办：退 75 是 prune-server 不可用：python3 prune-server/control check 看服务，修好之后重交同一条命令（报给主 agent）；别的照上面它打的出路改" >&2
  if (($2 == 75 || $2 == 2)); then return "$2"; fi
  return 1
}

# 编译与跑装置都直接起、不再套线程包装：整条命令交 task-server，线程照服务交的 SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS。
# 装置取 cargo 报的这一趟编出来的位置（binary_identity_build；编译目录由服务给，不是仓根 target/）。弄坏开关 binary-from-root-target 下取仓根 target/release 下那一份
build_the_binary() {
  local build_status
  binary_identity_build "$BINARY_NAME" --release -p singlefs-verification --bin "$BINARY_NAME"; build_status=$?
  if ((build_status == 1)); then
    echo "  ✗ 装置编不过" >&2
    echo "     → 怎么办：照 cargo 打的第一处 error 改装置，再起本脚本（判完的片在库里，复用）" >&2
    exit 1
  fi
  ((build_status == 0)) || exit 78
  BINARY="$BUILT_BINARY"
  break_is binary-from-root-target && BINARY="target/release/$BINARY_NAME"
  return 0
}

# 判一片：输出先写 seg-<片名>.partial，退 0 且齐了才改名成 seg-<片名>.out、记绿；不然记红。退出码照装置的；记库没成交 75 或 1。
run_segment() { # <片名>
  local segment="$1" sections cap environment=() code record_code
  sections=$(sections_of_segment "$segment") || usage_error "没有这一片：$segment"
  cap=$(cap_of_segment "$segment")
  case "$segment" in
    f9) environment+=("E181_PROBE_THREADS=${E181_PROBE_COUNT:-$(probe_count_of_cap "$cap")}") ;;
    narrow | unequal | small) environment+=("E181_THREADS=${E181_WALKERS:-$(walker_count_of_cap "$cap")}") ;;
    *) ;;
  esac
  binary_identity_assert_unchanged || return 1
  # shellcheck disable=SC2086  # 段名按空格拆成装置的几个参数
  env ${environment[@]+"${environment[@]}"} \
    "$LAUNCH_BINARY" $sections \
    > "$SEGMENT_DIRECTORY/seg-$segment.partial" 2> "$SEGMENT_DIRECTORY/seg-$segment.err"
  code=$?
  if ((code == 0)) && is_complete_output "$SEGMENT_DIRECTORY/seg-$segment.partial"; then
    mv -f -- "$SEGMENT_DIRECTORY/seg-$segment.partial" "$SEGMENT_DIRECTORY/seg-$segment.out"
    python3 "$ITEM_VERDICTS" record "$PLAN_DIRECTORY" "$segment" 0 "$SEGMENT_DIRECTORY/seg-$segment.out" || { library_failure_exit "record（片 $segment）" "$?"; return 75; }
    return 0
  fi
  ((code != 0)) || code=1
  python3 "$ITEM_VERDICTS" record "$PLAN_DIRECTORY" "$segment" "$code" "$SEGMENT_DIRECTORY/seg-$segment.partial" --red \
    || { library_failure_exit "record（片 $segment）" "$?"; return 75; }
  return "$code"
}

report_segment_failure() { # <片名> <退出码>
  if (($2 == 75)); then
    echo "  ✗ 片 $1 判完了、判定记不进 prune-server（见上一行）" >&2
    echo "     → 怎么办：python3 prune-server/control check 看服务，修好之后重交同一条命令" >&2
    return 75
  fi
  echo "  ✗ 片 $1 退 $2，判红（错误输出末尾在下面）" >&2
  echo "     → 怎么办：先看下面日志末尾几行；装置报停机的另有提示，出路在后面" >&2
  tail -n 5 -- "$SEGMENT_DIRECTORY/seg-$1.err" >&2
  if grep -q '^E7RESULT name=stop ' -- "$SEGMENT_DIRECTORY/seg-$1.partial" 2>/dev/null; then
    echo "  ✗ 装置报停机，该停下交回主 agent：" >&2
    grep -m3 '^E7RESULT name=stop ' -- "$SEGMENT_DIRECTORY/seg-$1.partial" >&2
    echo "     → 怎么办：把上面的 name=stop 行交主 agent，不重跑" >&2
    return 3
  fi
  echo "     → 怎么办：看日志末尾；内存撞顶由 task-server 放大重排，整条命令的结局看 task-server/task status；要少占内存就调小 E181_CAP_F9 / E181_CAP_F10 / E181_WALKERS 再起同一条命令" >&2
  return 1
}

assemble_the_products() {
  local segment landing checks_product assembled="$SEGMENT_DIRECTORY/landing.assembled"
  landing=$(next_free_product_path "$LANDING_STEM")
  checks_product=$(next_free_product_path "$CHECKS_STEM")
  rm -f -- "$assembled"
  for segment in "${SEGMENT_ORDER[@]}"; do cat -- "$SEGMENT_DIRECTORY/seg-$segment.out" >> "$assembled" || return 1; done
  is_complete_output "$assembled" || { echo "  ✗ 拼出来的产物不齐：$assembled" >&2; echo "     → 怎么办：看这份文件末尾缺哪一段的收尾行，交主 agent 查那一片的判定（库里那一条判绿而产物不齐，是装置或收尾判法的缺陷）" >&2; return 1; }
  mkdir -p -- "$PRODUCT_DIRECTORY" || return 1
  mv -n -- "$assembled" "$landing" || return 1
  [[ ! -e "$checks_product" ]] || { echo "  ✗ 复跑登记用的产物名被占了：$checks_product" >&2; echo "     → 怎么办：落地产物已经落在 $landing；照文件头「产物名」另起一个没被占的 …-checks 名字" >&2; return 1; }
  cp -- "$SEGMENT_DIRECTORY/seg-checks.out" "$checks_product" || return 1
  echo "产物：$landing（$(wc -l < "$landing") 行）；复跑登记用：$checks_product（$(wc -l < "$checks_product") 行）；按片判定 $counts"
}

# 自证：假装置在每一格自己的临时树里走一遍（本脚本与共用库拷进树里，树里 .claude 与 long_run_item_verdicts.py 是指回真仓的符号链接）；不起 cargo、不碰真装置。
# 判定写进临时 prune-server（客户端 selftest_service_problem 判在不在临时服务底下），每格一份命名空间；八片的装置入口都换成假树里的 measure_alpha（不然算不出键、全拒算）。
# 树里没有 scripts/hosts/：常规路径要是又去套那一层线程包装，起不来，各格照红。
# 换不掉的一处：产物名里的日期用 TZ=Asia/Tokyo date +%F 取，自证里同样取，跨零点的那一刻会误红，重跑即可。
run_selftest() {
  local work failures=0 cell_count=0 fakes real_root today cell exit_code first_exit old_sha original_sha swap_target stem_landing stem_checks segment reuse_output reuse_red
  unset E181_PRODUCT_DIRECTORY E181_BINARY E181_LOG_DIRECTORY E181_CAP_F9 E181_CAP_F10 E181_PROBE_COUNT E181_WALKERS
  real_root="$PWD"
  python3 "$real_root/$ITEM_VERDICTS" selftest-service || return 2
  today=$(TZ=Asia/Tokyo date +%F)
  work=$(mktemp -d) || return 1
  fakes="$work/fakes"
  mkdir -p -- "$fakes"
  echo e181-long-run-selftest > "$work/mark"
  stem_landing="research/results/e181-landing-check-fixed-point-and-mount-lower-bound-landing"
  stem_checks="research/results/e181-landing-check-fixed-point-and-mount-lower-bound-checks"
  cat > "$fakes/device" <<'FAKE'
#!/usr/bin/env bash
# 假装置：每个段名一行 E7RESULT name=row，再一行该段的收尾行 name=done（emitted 含收尾行自己）；f10-measure 多一行外推。
# E181_SELFTEST_SWAP_TARGET：头一回起来、还没干活之前（脚本已拷过私有副本）把这个路径的装置换成另一份字节不同的，像别的会话的 cargo 把仓根那份换掉，
#   再按自己被起的那个路径 exec 一次：按私有副本起的照旧是开跑时那份，按仓根路径起的（弄坏开关 exec-root-binary）就成了换后那份。
if [[ -n "${E181_SELFTEST_SWAP_TARGET:-}" && ! -e "$E181_SELFTEST_SWAP_TARGET.swapped" ]]; then
  { cat -- "$E181_SELFTEST_SWAP_TARGET"; echo "# swapped by a build while the long run is going"; } > "$E181_SELFTEST_SWAP_TARGET.swap" \
    && chmod +x "$E181_SELFTEST_SWAP_TARGET.swap" && mv -f -- "$E181_SELFTEST_SWAP_TARGET.swap" "$E181_SELFTEST_SWAP_TARGET" && : > "$E181_SELFTEST_SWAP_TARGET.swapped" \
    && exec "$0" "$@"
fi
self_sha=$(sha256sum -- "$0" | cut -d' ' -f1)
echo "$* | threads=${E181_THREADS:-unset} probe=${E181_PROBE_THREADS:-unset}" >> "$E181_SELFTEST_CALLS"
echo "$* $self_sha" >> "$E181_SELFTEST_CALLS.self-sha"
case "${E181_SELFTEST_DEVICE:-ok}" in
  stop) echo "E7RESULT name=stop reason=selftest_stop"; exit 3 ;;
  fail) echo "boom" >&2; exit 1 ;;
esac
for section in "$@"; do
  rows=1
  printf 'E7RESULT name=row section=%s device=%s\n' "$section" "$self_sha"
  if [[ "$section" == f10-measure ]]; then
    rows=2
    printf 'E7RESULT name=f10_measure_extrapolation extrapolated_hours_single_thread=%s note=selftest\n' 1.00
  fi
  emitted=$((rows + 1))
  [[ "${E181_SELFTEST_DEVICE:-ok}" == truncated ]] && emitted=99
  printf 'E7RESULT name=done emitted=%s\n' "$emitted"
done
FAKE
  chmod +x "$fakes/device"
  # 样板树：本脚本与共用库的拷贝、树里的假装置（fake-bin/device）、指向真仓 .claude 与 long_run_item_verdicts.py 的符号链接、假装置入口、空的 git 仓。每格拷一份。
  local tree="$work/template"
  mkdir -p -- "$tree/scripts/experiment-apparatus/long-run-lib" "$tree/research/results" "$tree/fake-bin"
  cp -- "$SCRIPT_PATH" "$tree/scripts/experiment-apparatus/e181-landing-check-long-run.sh"
  cp -- "$(dirname -- "$SCRIPT_PATH")/long-run-lib/binary-identity.sh" "$tree/scripts/experiment-apparatus/long-run-lib/"
  cp -- "$fakes/device" "$tree/fake-bin/device"
  ln -s -- "$real_root/$ITEM_VERDICTS" "$tree/$ITEM_VERDICTS"
  ln -s -- "$real_root/.claude" "$tree/.claude"
  python3 "$real_root/$ITEM_VERDICTS" plant-fake-entries "$tree" || { rm -rf -- "${work:?}"; return 1; }
  git -C "$tree" init -q > /dev/null 2>&1
  for segment in "${SEGMENT_ORDER[@]}"; do printf '%s\tscripts/experiment-apparatus/long_run_reuse_fake_entries.py::measure_alpha\n' "$segment"; done > "$work/all-segments.tsv"
  make_tree() {
    mkdir -p -- "$1/logs"
    cp -R -- "$work/template" "$1/tree"
  }
  # 在这一格的树里起脚本拷贝；这一格要设的环境变量写在 invoke 前面（INVOKE_BINARY 换装置路径；INVOKE_BUILDS 设了就不给 E181_BINARY，PATH 最前面放这一格的假 cargo）。
  invoke() {
    local cell="$1" binary_setting
    shift
    binary_setting=("E181_BINARY=${INVOKE_BINARY:-$cell/tree/fake-bin/device}")
    [[ -n "${INVOKE_BUILDS:-}" ]] && binary_setting=("PATH=$cell/fake-cargo:$PATH")
    ( cd -- "$cell/tree" && env -u SINGLEFS_REUSE_SENTINEL_SEED "${binary_setting[@]}" GIT_CEILING_DIRECTORIES="$work" E181_SELFTEST_CALLS="$cell/calls" \
        E181_LOG_DIRECTORY="$cell/logs" E181_LONG_RUN_SELFTEST_MARK="$work/mark" LONG_RUN_ITEMS_LIBRARY_DIRECTORY="$cell/library" \
        LONG_RUN_ITEMS_SELFTEST_ENTRIES="$work/all-segments.tsv" SINGLEFS_REUSE_SENTINEL_COUNT=1000 \
        bash "$cell/tree/scripts/experiment-apparatus/e181-landing-check-long-run.sh" "$@" > "$cell/out" 2>&1 )
  }
  calls_of() { # <格> [<后缀>]：假装置被起了几次
    if [[ -f "$1/calls${2:-}" ]]; then wc -l < "$1/calls${2:-}"; else echo 0; fi
  }
  products_of() { # <格>：产物目录里有几份
    find "$1/tree/research/results" -type f | wc -l
  }
  expect() {
    local name="$1" condition="$2"
    cell_count=$((cell_count + 1))
    if eval "$condition"; then
      echo "  ✓ $name（第 $cell_count 格）"
    else
      echo "  ✗ $name：条件「$condition」不成立"
      [[ -f "$cell/out" ]] && tail -n 6 -- "$cell/out" | sed 's/^/      输出：/'
      echo "     → 怎么办：照上面这一格点名的条件和输出改本脚本，再在临时 prune-server 底下单独起本脚本的 --selftest（文件头「用法」那一条）到它退 0"
      failures=$((failures + 1))
    fi
  }
  cell="$work/fresh"; make_tree "$cell"
  original_sha=$(sha256sum -- "$cell/tree/fake-bin/device" | cut -d' ' -f1)
  invoke "$cell"; exit_code=$?
  expect "全新：八片现判、拼出落地产物与复跑登记用的产物、成功句报现判 8、复用 0、拒算 0" '((exit_code == 0)) && grep -qF "现判 8、复用 0、拒算 0" "$cell/out" && (($(wc -l < "$cell/tree/$stem_landing.out") == 23)) && (($(wc -l < "$cell/tree/$stem_checks.out") == 8)) && (($(calls_of "$cell") == 8)) && grep -qF "装置 sha256 $original_sha（私有副本" "$cell/out"'
  invoke "$cell"; exit_code=$?
  expect "原样重起：八片全复用、装置一次没起、产物另存带日期的" '((exit_code == 0)) && grep -qF "现判 0、复用 8、拒算 0" "$cell/out" && (($(calls_of "$cell") == 8)) && [[ -f "$cell/tree/$stem_landing-$today.out" ]]'
  cell="$work/swap"; make_tree "$cell"
  swap_target="$cell/tree/fake-bin/device"
  original_sha=$(sha256sum -- "$swap_target" | cut -d' ' -f1)
  E181_SELFTEST_SWAP_TARGET="$swap_target" invoke "$cell"; exit_code=$?
  expect "装置在一趟里不许换：跑着的时候仓根那份被换掉，之后起的八片自报的 sha256 都是开跑时那份" '((exit_code == 0)) && [[ -e "$swap_target.swapped" ]] && [[ "$(sha256sum -- "$swap_target" | cut -d" " -f1)" != "$original_sha" ]] && (($(calls_of "$cell" .self-sha) == 8)) && [[ "$(awk "{print \$NF}" "$cell/calls.self-sha" | sort -u)" == "$original_sha" ]]'
  cell="$work/only"; make_tree "$cell"
  invoke "$cell" --only checks,f11; first_exit=$?
  cp -- "$cell/out" "$cell/out.first"
  invoke "$cell"; exit_code=$?
  expect "--only 点名的片判完不拼产物；再不带参数起，那两片复用、其余现判、拼出产物" '((first_exit == 0 && exit_code == 0)) && grep -qF "点名的片判完" "$cell/out.first" && grep -qF "现判 2、复用 0、拒算 0" "$cell/out.first" && grep -qF "现判 6、复用 2、拒算 0" "$cell/out" && (($(calls_of "$cell") == 8)) && (($(products_of "$cell") == 2))'
  cell="$work/device-stop"; make_tree "$cell"
  E181_SELFTEST_DEVICE=stop invoke "$cell"; exit_code=$?
  expect "装置报停机：退 3、name=stop 行转出来、不拼产物" '((exit_code == 3)) && grep -qF "name=stop reason=selftest_stop" "$cell/out" && (($(products_of "$cell") == 0))'
  cell="$work/truncated"; make_tree "$cell"
  E181_SELFTEST_DEVICE=truncated invoke "$cell"; exit_code=$?
  expect "末行不齐的片判红：退 1、头一片记红、别的没跑到、不拼产物" '((exit_code == 1)) && grep -qF "· checks：红，现判" "$cell/out" && (($(grep -c "：没跑到" "$cell/out") == 7)) && (($(products_of "$cell") == 0))'
  cell="$work/binary-missing"; make_tree "$cell"
  INVOKE_BINARY="$cell/tree/fake-bin/not-built" invoke "$cell"; exit_code=$?
  expect "装置二进制不在：退 78、装置一次没起" '((exit_code == 78)) && grep -qF "装置二进制不在或不可执行" "$cell/out" && (($(calls_of "$cell") == 0))'
  # 没给 E181_BINARY：编一次，装置取 cargo 报的这一趟编出来的位置（这一格的编译目录 build/release 下，演 task-server 给的那一份），
  #   仓根 target/release 下另放一份早先编的旧装置（字节不同，像并包前编的那份）：八片自报的 sha256 都是编出来的那一份
  cell="$work/built-here"; make_tree "$cell"
  mkdir -p -- "$cell/build/release" "$cell/tree/target/release" "$cell/fake-cargo"
  cp -- "$fakes/device" "$cell/build/release/e181_landing_check_fixed_point_and_mount_lower_bound"
  { cat -- "$fakes/device"; echo "# an old build left in the root target directory"; } > "$cell/tree/target/release/e181_landing_check_fixed_point_and_mount_lower_bound"
  chmod +x -- "$cell/tree/target/release/e181_landing_check_fixed_point_and_mount_lower_bound"
  original_sha=$(sha256sum -- "$cell/build/release/e181_landing_check_fixed_point_and_mount_lower_bound" | cut -d' ' -f1)
  old_sha=$(sha256sum -- "$cell/tree/target/release/e181_landing_check_fixed_point_and_mount_lower_bound" | cut -d' ' -f1)
  cat > "$cell/fake-cargo/cargo" <<'FAKE'
#!/usr/bin/env bash
# 假 cargo（built-here 那一格）：记参数；build 时照 cargo 的机读输出报一行 compiler-artifact，executable 指 E181_SELFTEST_BUILT（这一格编译目录里那一份）
echo "$*" >> "$E181_SELFTEST_CALLS.cargo"
[[ "${1:-}" == build ]] || exit 0
printf '{"reason":"compiler-artifact","target":{"name":"%s"},"executable":"%s","fresh":false}\n' "${E181_SELFTEST_BUILT##*/}" "$E181_SELFTEST_BUILT"
printf '{"reason":"build-finished","success":true}\n'
FAKE
  chmod +x -- "$cell/fake-cargo/cargo"
  E181_SELFTEST_BUILT="$cell/build/release/e181_landing_check_fixed_point_and_mount_lower_bound" INVOKE_BUILDS=1 invoke "$cell"; exit_code=$?
  expect "没给 E181_BINARY：装置取 cargo 报的这一趟编出来的那一份、不取仓根 target/release 下早先编的那一份" '((exit_code == 0)) && grep -qF "build --release -p singlefs-verification --bin e181_landing_check_fixed_point_and_mount_lower_bound --message-format=json-render-diagnostics" "$cell/calls.cargo" && (($(calls_of "$cell" .self-sha) == 8)) && [[ "$(awk "{print \$NF}" "$cell/calls.self-sha" | sort -u)" == "$original_sha" ]] && grep -qF "源 $cell/build/release/e181_landing_check_fixed_point_and_mount_lower_bound" "$cell/out" && [[ "$old_sha" != "$original_sha" ]]'
  cell="$work/list"; make_tree "$cell"
  invoke "$cell" --list-segments; exit_code=$?
  expect "--list-segments：列八片、不起装置" '((exit_code == 0)) && (($(grep -c "（上限 " "$cell/out") == 8)) && (($(calls_of "$cell") == 0))'
  cell="$work/bad-argument"; make_tree "$cell"
  invoke "$cell" --bogus; exit_code=$?
  expect "认不得的参数退 2" '((exit_code == 2)) && grep -qF "认不得的参数「--bogus」" "$cell/out" && (($(calls_of "$cell") == 0))'
  # 复用那六格（long_run_item_verdicts.py 文件头「复用格」）：checks、f11 两片，--only 跑完退 0
  reuse_output=$(python3 "$real_root/$ITEM_VERDICTS" reuse-cells --script "$SCRIPT_KEY" --template "$work/template" --items checks,f11 --work "$work/reuse" --green-exit 0 -- \
    env E181_BINARY="{tree}/fake-bin/device" GIT_CEILING_DIRECTORIES="$work" E181_SELFTEST_CALLS="{tree}/../calls" E181_LOG_DIRECTORY="{tree}/../logs" \
    E181_LONG_RUN_SELFTEST_MARK="$work/mark" bash "{tree}/scripts/experiment-apparatus/e181-landing-check-long-run.sh" --only checks,f11 2>&1)
  echo "$reuse_output"
  reuse_red=$(sed -n 's/^long_run_item_verdicts 复用格.*：6 格，红 \([0-9]*\) 格$/\1/p' <<< "$reuse_output")
  cell_count=$((cell_count + 6))
  if [[ -z "$reuse_red" ]]; then failures=$((failures + 6)); echo "  ✗ 复用那六格没跑完（没有「long_run_item_verdicts 复用格」那一行），按六格红算"; echo "     → 怎么办：看上面 reuse-cells 打的输出里第一处 ✗ 或 Python 报错（复用格起不来多半是 prune-server 不可用或 long_run_item_verdicts.py 抛了异常），修好之后在临时 prune-server 底下单独起本脚本的 --selftest（文件头「用法」那一条）到它退 0"; else failures=$((failures + reuse_red)); fi
  rm -rf -- "${work:?}"
  if ((failures > 0)); then echo "e181-landing-check-long-run.sh 自证：$failures 格红（共 $cell_count 格）"; return 1; fi
  echo "e181-landing-check-long-run.sh 自证：$cell_count 格全绿"
  return 0
}

list_segments=0
selftest=0
only_text=""
while (($# > 0)); do
  case "$1" in
    --selftest) selftest=1; shift ;;
    --list-segments) list_segments=1; shift ;;
    --only) (($# >= 2)) || usage_error "--only 后面要写片名"; only_text="$2"; shift 2 ;;
    *) usage_error "认不得的参数「$1」" ;;
  esac
done

if ((selftest)); then
  run_selftest
  exit $?
fi

if ((list_segments)); then
  for segment in "${SEGMENT_ORDER[@]}"; do echo "$segment: $(sections_of_segment "$segment")（上限 $(cap_of_segment "$segment")）"; done
  exit 0
fi

wanted=("${SEGMENT_ORDER[@]}")
if [[ -n "$only_text" ]]; then
  IFS=',' read -r -a wanted <<< "$only_text"
  for segment in "${wanted[@]}"; do sections_of_segment "$segment" > /dev/null || usage_error "--only 点名的片不存在：$segment"; done
fi

mkdir -p -- "$LOG_DIRECTORY" || exit 1
# 这一趟的状态目录（long_run_item_verdicts.py 的计划、复用片取回的产物、这一趟判的片输出）：跑完、失败、被打断都删
STATE_DIRECTORY=$(mktemp -d "$LOG_DIRECTORY/run-XXXXXX") || exit 1
trap 'rm -rf -- "$STATE_DIRECTORY"' EXIT
PLAN_DIRECTORY="$STATE_DIRECTORY/plan"
SEGMENT_DIRECTORY="$STATE_DIRECTORY/segments"
mkdir -p -- "$SEGMENT_DIRECTORY" || exit 1
for segment in "${wanted[@]}"; do segment_request "$segment"; done > "$STATE_DIRECTORY/requests.tsv"
python3 "$ITEM_VERDICTS" plan "$PLAN_DIRECTORY" --script "$SCRIPT_KEY" --kind "$BINARY_NAME" --root "$PWD" --requests "$STATE_DIRECTORY/requests.tsv" \
  || { library_failure_exit plan "$?"; exit $?; }

pending=()
for segment in "${wanted[@]}"; do
  if [[ "$(decision_of "$segment")" == reuse ]] && ! break_is reuse-ignored; then
    cp -- "$PLAN_DIRECTORY/outputs/$segment.out" "$SEGMENT_DIRECTORY/seg-$segment.out" || exit 1
    echo "片 $segment：复用（键相同、库里判绿）"
  else
    pending+=("$segment")
  fi
done

contains_segment() { local needle="$1" item; shift; for item in "$@"; do [[ "$item" == "$needle" ]] && return 0; done; return 1; }
run_and_check() { # <片名>
  local segment="$1" code
  run_segment "$segment"; code=$?
  ((code == 0)) || { report_segment_failure "$segment" "$code"; return $?; }
}

chain_status=0
f9_status=0
if ((${#pending[@]} > 0)); then
  [[ -n "${E181_BINARY:-}" ]] || build_the_binary
  [[ -x "$BINARY" ]] || { echo "  ✗ 装置二进制不在或不可执行：$BINARY" >&2; echo "     → 怎么办：没给 E181_BINARY 就看上面 cargo build --release 的输出；给了就核对它指向的文件存在且可执行" >&2; exit 78; }
  # 开跑时拷一份私有副本：八片都按这份拷贝起
  binary_identity_pin "$BINARY" "$LOG_DIRECTORY/pinned-binary" || exit 1
  LAUNCH_BINARY="$PINNED_BINARY"
  break_is exec-root-binary && LAUNCH_BINARY="$BINARY"
  load_one_minute=$(cut -d' ' -f1 /proc/loadavg)
  echo "开跑前负载 loadavg1=$load_one_minute（f11 的挂钟读数在机器忙时偏大；装置收尾行会再记一次）；装置 sha256 $PINNED_BINARY_SHA256（私有副本 $PINNED_BINARY，源 $BINARY）"

  # 计时片先跑，机器上没有别的片在跑
  for segment in checks f11; do
    if contains_segment "$segment" "${pending[@]}"; then run_and_check "$segment" || { chain_status=$?; break; }; fi
  done

  # f9 与 F10 那一串并行：每片把退出码写进自己的文件，最后单独一个 wait 等齐
  f9_code_file="$STATE_DIRECTORY/f9.code"
  rm -f -- "$f9_code_file"
  f9_started=0
  if ((chain_status == 0)) && contains_segment f9 "${pending[@]}"; then
    { run_segment f9; echo "$?" > "$f9_code_file"; } &
    f9_started=1
  fi
  if ((chain_status == 0)); then
    for segment in measure controls narrow unequal small; do
      if contains_segment "$segment" "${pending[@]}"; then
        run_and_check "$segment" || { chain_status=$?; break; }
      fi
    done
  fi
  wait  # shell-lint:exit-collected f9 片的退出码写进 $f9_code_file，下面读它；文件不在就判红
  if ((f9_started)); then
    [[ -f "$f9_code_file" ]] || { echo "  ✗ f9 片的退出码文件不在：$f9_code_file" >&2; echo "     → 怎么办：看上面其他片的输出，再起同一条命令（判完的片复用）" >&2; f9_status=1; }
    if ((f9_status == 0)); then
      f9_status=$(cat -- "$f9_code_file")
      ((f9_status == 0)) || { report_segment_failure f9 "$f9_status"; f9_status=$?; }
    fi
  fi
fi
python3 "$ITEM_VERDICTS" finish "$PLAN_DIRECTORY" > "$STATE_DIRECTORY/finish.out"; finish_exit=$?
cat -- "$STATE_DIRECTORY/finish.out"
((finish_exit <= 1)) || { library_failure_exit finish "$finish_exit"; exit $?; }
break_is finish-ignored && finish_exit=0
counts=$(sed -n 's/^LONG_RUN_ITEMS .* \(现判 [0-9]*、复用 [0-9]*、拒算 [0-9]*\)$/\1/p' "$STATE_DIRECTORY/finish.out")
echo "按片判定：$counts"
((chain_status == 0)) || exit "$chain_status"
((f9_status == 0)) || exit "$f9_status"
((finish_exit == 0)) || { echo "  ✗ 按片判定收尾判红（复用哨兵或键在跑的中途变了，见上面 ✗ 那几行）" >&2; echo "     → 怎么办：照上面那几行的出路办，再起同一条命令" >&2; exit 1; }
if [[ -n "$only_text" ]]; then
  echo "点名的片判完了（不拼产物）：$only_text；判定在 prune-server，不带参数再起一次就复用它们、拼产物"
  exit 0
fi
assemble_the_products
