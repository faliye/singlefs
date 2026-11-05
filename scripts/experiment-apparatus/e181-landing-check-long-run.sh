#!/usr/bin/env bash
# admission: always 每一次调都照此刻各段输出齐不齐判还差什么，上一次跑完不替这一次作保
# run-condition: command cargo git
# E181（落得下那一判与挂载下界）第一段（固定点与落得下段，跑前登记 research/prompts/e181-preregistration.md 5.5 节第一段）的长跑跑法：
#   装置是入库装置 crates/singlefs-verification/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs；装置按「段」（命令行参数）一段一段跑，
#   本脚本把它们切成八个互不依赖的片，每片一个进程、一份输出，全部做完再按固定次序拼成产物
#   research/results/e181-landing-check-fixed-point-and-mount-lower-bound-landing.out（多段产物，每段有自己的 name=done 收尾行），
#   第一片（常量回比、锚点、自证、小几何的阳性对照）另存一份给复跑登记用的产物 …-checks.out（逐字节可比，不含计时与线程数）。
#
# long-run-shards: 一片 = 装置的一段输出：checks、f11、f9、measure、controls、narrow、unequal、small 八片；每片一个进程，--only 点名哪几片，两台机器可以各跑一部分片
# long-run-resume: 同一条命令重起就是续跑：每片做完把输出落成日志目录里的 seg-<片名>.out（片旁 seg-<片名>.out.binary-sha256 记着打它的装置 sha256），重起先看它末行是不是 name=done emitted=行数、片旁记的装置 sha256 是不是这一趟的，两样都齐的直接取、只补没做完的（记的不同或没有记录的片整份移到日志目录的 void-<日期>-<进程号>/ 再现跑，逐片报「作废：<片名>（装置 sha256 不同）」），成功那句报「现判 N、复用 M」
# long-run-unshardable: 多机这一段缺：本脚本不自己读多机配置去分发，第二台上的片由主 agent 用 --only 点名起、再把 seg-*.out 取回本机后用 --assemble 并成产物
#
# 片与段、并发数（并发数显式给：一份试写的拷贝约占 5 GiB，一个走种子的线程约占 2 GiB 以上，装置里的默认是每台默认重活线程数，会撞顶；
#   脚本里不套内存包装，整条命令交 task-server，内存额度由服务按记录给；下面的内存预算只用来算并发数）：
#   checks   constants anchors selfcheck f10-controls-small
#   f11      f11                                              （计时片：开跑前机器要闲，跑的时候不和别的片并行）
#   f9       f9                                               预算 E181_CAP_F9（默认 16G），同时试写的拷贝数 (预算 − 4) ÷ 5 取整
#   measure  f10-measure（G默认上量 F10 的 10 步，登记 5.6 节）      （填满 G默认 的阶梯第一档是 10 万个单元，内容加两块盘的层约 10 GiB 以上，推的）
#   controls f10-controls（G窄 的阳性对照）
#   narrow   f10-narrow                                       预算 E181_CAP_F10（默认 24G），走种子的线程数 (预算 − 8) ÷ 4 取整，E181_WALKERS 给了照它
#   unequal  f10-unequal                                      同上
#   small    f10-small                                        同上
# measure 片和别的片一样一律跑完、不设外推停机（挂钟由统一任务服务管）；外推读数在 measure 片的输出里（name=f10_measure_extrapolation）。
# 装置在一趟里不许换，每片记下打它的是哪一版装置（欠账 C862，共用段落在 long-run-lib/binary-identity.sh）：开跑时把装置二进制拷一份进 $E181_LOG_DIRECTORY/pinned-binary/<sha256>/（只读），
#   八片都按这份拷贝起，不再 exec 仓根 target/release 下那一份（别的会话的 cargo 随时会把它换掉）；每片做完先在片旁写 seg-<片名>.out.binary-sha256（这份拷贝的 sha256）再换名成 seg-<片名>.out；
#   续跑只沿用片旁记的 sha256 与这一趟相同的片；拼产物之前再核八片记的 sha256 都是这一趟的（--assemble 不起装置时核八片彼此相同），不同就拒、不出产物。
#   第二台上用 --only 跑的那几片要用与本机同一份装置（把本机私有副本送过去、第二台 E181_BINARY 指到送到的文件），用自己编的另一份，取回后本机拼产物时会因 sha256 不同被拒。
#   老的私有副本留着不删（一个 sha256 一个目录），确认不再续跑那一版之后手删。
#
# 用法（在仓里任何目录起都行，脚本自己回到仓根）：
#   bash scripts/experiment-apparatus/e181-landing-check-long-run.sh                      本机跑全部片并拼产物（做完的片直接取）
#   bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --only <片名>[,<片名>…]   只做点名的片，不拼产物（分给第二台用）
#   bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --assemble           不跑，只拼产物（八片都齐才拼）
#   bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --list-segments      列八片和它们的段，不编译、不跑
#   bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --selftest           假装置在临时树里走一遍续跑、换装置作废、跑着换二进制、拼产物前核 sha256（不起 cargo、不碰真装置）
# 环境：E181_PRODUCT_DIRECTORY（产物目录，默认 research/results；自证用）、E181_BINARY（装置二进制；没给就先 cargo build --release 编仓根 target/ 下的那一份）、E181_LOG_DIRECTORY（片输出与进度日志目录，默认
#   工作根下的 e181-long-run，工作根取 .claude/hooks/lib_work_root.py、两台同一份）、E181_CAP_F9、E181_CAP_F10、E181_PROBE_COUNT（f9 同时试写的拷贝数）、E181_WALKERS（走种子的线程数）。
# 产物名：…-landing.out 已有时另存 …-landing-<日期>.out，同一天再跑加 -r2、-r3，已有的一个不动；…-checks.out 同样。
# 退出码：0 做完；1 装置跑失败或产物不齐；2 用法错；3 装置报停机（日志里 name=stop 那一行）；
#   78 装置没编好（--list-segments 不需要）。
#
# 弄坏开关 E181_LONG_RUN_BREAK=<项，逗号分隔>（只在 --selftest 起的那几趟里认：自证设 E181_LONG_RUN_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   E181_LONG_RUN_BREAK=exec-root-binary  每片仍按开跑时给的那个路径起、不按私有副本
#       「装置在一趟里不许换：跑着的时候仓根那份被换掉，之后起的八片自报的 sha256 与片旁记的都是开跑时那份」
#   E181_LONG_RUN_BREAK=stale-binary-reused  沿用只看片末行、不看片旁记的装置 sha256
#       「旧装置打的八片全作废：逐片报作废、八片全现跑、旧片连记录整份留在作废目录、片旁记的都是新装置的」「片旁记的 sha256 不同的、没有记录的片都作废重跑，另外六片沿用」
#   E181_LONG_RUN_BREAK=merge-sha-unchecked  拼产物之前不核各片记的装置 sha256
#       「拼产物之前各片记的装置 sha256 不同：拒、点名那一片、退 1、不另出产物」「--assemble 八片记的 sha256 彼此相同：不起装置、核过、另存带日期的产物」
#   E181_LONG_RUN_BREAK=done-marker-ignored  片输出末行不齐也当做完
#       「末行不齐的片输出不算做完：退 1、没有 seg-*.out、不拼产物」
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "$(dirname "${BASH_SOURCE[0]}")/long-run-lib/binary-identity.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; echo "     → 怎么办：本脚本得留在仓里的 scripts/experiment-apparatus/ 下（它按自己的位置找仓根）；挪走了就放回去再起" >&2; exit 2; }

PRODUCT_DIRECTORY="${E181_PRODUCT_DIRECTORY:-research/results}"
LANDING_STEM="$PRODUCT_DIRECTORY/e181-landing-check-fixed-point-and-mount-lower-bound-landing"
CHECKS_STEM="$PRODUCT_DIRECTORY/e181-landing-check-fixed-point-and-mount-lower-bound-checks"
BINARY="${E181_BINARY:-target/release/e181_landing_check_fixed_point_and_mount_lower_bound}"
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

# 编译与跑装置都直接起、不再套线程包装：整条命令交 task-server，线程照服务交的 SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS
build_the_binary() {
  cargo build --release -p singlefs-verification --bin e181_landing_check_fixed_point_and_mount_lower_bound || {
    echo "  ✗ 装置编不过" >&2
    echo "     → 怎么办：照 cargo 打的第一处 error 改装置，再起本脚本" >&2
    exit 1
  }
}

# 做一片：输出落 seg-<片名>.out（先写 .partial，退 0 且齐了才改名），进度写 seg-<片名>.err。退出码照装置的（被内存包装拦下的也照它的）。
run_segment() { # <片名>
  local segment="$1" sections cap environment=() code
  sections=$(sections_of_segment "$segment") || usage_error "没有这一片：$segment"
  cap=$(cap_of_segment "$segment")
  case "$segment" in
    f9) environment+=("E181_PROBE_THREADS=${E181_PROBE_COUNT:-$(probe_count_of_cap "$cap")}") ;;
    narrow | unequal | small) environment+=("E181_THREADS=${E181_WALKERS:-$(walker_count_of_cap "$cap")}") ;;
    *) ;;
  esac
  rm -f -- "$LOG_DIRECTORY/seg-$segment.partial"
  binary_identity_assert_unchanged || return 1
  # shellcheck disable=SC2086  # 段名按空格拆成装置的几个参数
  env ${environment[@]+"${environment[@]}"} \
    "$LAUNCH_BINARY" $sections \
    > "$LOG_DIRECTORY/seg-$segment.partial" 2> "$LOG_DIRECTORY/seg-$segment.err"
  code=$?
  if ((code == 0)) && is_complete_output "$LOG_DIRECTORY/seg-$segment.partial"; then
    # 先在片旁记下打它的装置 sha256（这份私有副本的），再换名成 .out：有 .out 就一定有记录
    binary_identity_stamp_write "$LOG_DIRECTORY/seg-$segment.out" || return 1
    mv -f -- "$LOG_DIRECTORY/seg-$segment.partial" "$LOG_DIRECTORY/seg-$segment.out"
    return 0
  fi
  ((code != 0)) || code=1
  return "$code"
}

report_segment_failure() { # <片名> <退出码>
  echo "  ✗ 片 $1 退 $2（日志 $LOG_DIRECTORY/seg-$1.err，输出 $LOG_DIRECTORY/seg-$1.partial）" >&2
  echo "     → 怎么办：先看下面日志末尾几行；装置报停机的另有提示，出路在后面" >&2
  tail -n 5 -- "$LOG_DIRECTORY/seg-$1.err" >&2
  if grep -q '^E7RESULT name=stop ' -- "$LOG_DIRECTORY/seg-$1.partial" 2>/dev/null; then
    echo "  ✗ 装置报停机，该停下交回主 agent：" >&2
    grep -m3 '^E7RESULT name=stop ' -- "$LOG_DIRECTORY/seg-$1.partial" >&2
    echo "     → 怎么办：把上面的 name=stop 行交主 agent，不重跑" >&2
    return 3
  fi
  echo "     → 怎么办：看日志末尾；内存撞顶由 task-server 放大重排，整条命令的结局看 task-server/task status；要少占内存就调小 E181_CAP_F9 / E181_CAP_F10 / E181_WALKERS 再起同一条命令" >&2
  return 1
}

assemble_the_products() {
  local segment missing=0 landing checks_product checked_pairs=()
  for segment in "${SEGMENT_ORDER[@]}"; do
    [[ -f "$LOG_DIRECTORY/seg-$segment.out" ]] || { echo "  ✗ 片 $segment 还没做完：$LOG_DIRECTORY/seg-$segment.out 不在" >&2; missing=$((missing + 1)); }
  done
  ((missing == 0)) || { echo "     → 怎么办：不带 --assemble 再起一次本脚本（或在第二台上 --only 跑完点名的片、把 seg-*.out 取回本机）" >&2; return 1; }
  # 拼产物之前再核一遍：八片片旁记的装置 sha256 都得是这一趟的（--assemble 不起装置时没有这一趟的，核八片彼此相同），不同或没有记录就拒、不出产物
  if ! break_is merge-sha-unchecked; then
    for segment in "${SEGMENT_ORDER[@]}"; do
      checked_pairs+=("$segment" "$(binary_identity_stamp_read "$LOG_DIRECTORY/seg-$segment.out")")
    done
    binary_identity_check_same "${PINNED_BINARY_SHA256:-}" "${checked_pairs[@]}" || return 1
  fi
  landing=$(next_free_product_path "$LANDING_STEM")
  checks_product=$(next_free_product_path "$CHECKS_STEM")
  local assembled="$LOG_DIRECTORY/landing.assembled"
  rm -f -- "$assembled"
  for segment in "${SEGMENT_ORDER[@]}"; do cat -- "$LOG_DIRECTORY/seg-$segment.out" >> "$assembled"; done
  is_complete_output "$assembled" || { echo "  ✗ 拼出来的产物不齐：$assembled" >&2; echo "     → 怎么办：看这份文件末尾缺哪一段的收尾行，把对应片的 $LOG_DIRECTORY/seg-<片名>.out 删掉，不带 --assemble 再起一次本脚本重做那一片" >&2; return 1; }
  mv -n -- "$assembled" "$landing" || return 1
  [[ ! -e "$checks_product" ]] || { echo "  ✗ 复跑登记用的产物名被占了：$checks_product" >&2; echo "     → 怎么办：落地产物已经落在 $landing；把 $LOG_DIRECTORY/seg-checks.out 拷成一个没被占的 …-checks 名字（命名规则见文件头「产物名」）" >&2; return 1; }
  cp -- "$LOG_DIRECTORY/seg-checks.out" "$checks_product" || return 1
  echo "产物：$landing（$(wc -l < "$landing") 行）；复跑登记用：$checks_product（$(wc -l < "$checks_product") 行）；进度日志目录：$LOG_DIRECTORY"
}

# 自证：假装置在每一格自己的临时树里走一遍（本脚本与共用库拷进树里，树里 .claude 指向真仓的）；不起 cargo、不碰真装置。
# 树里没有 scripts/hosts/：常规路径要是又去套那一层线程包装，起不来，各格照红。
# 换不掉的一处：产物名里的日期用 TZ=Asia/Tokyo date +%F 取，自证里同样取，跨零点的那一刻会误红，重跑即可。
run_selftest() {
  local work failures=0 cell_count=0 fakes real_root today cell exit_code first_exit old_sha original_sha swap_target stem_landing stem_checks
  unset E181_PRODUCT_DIRECTORY E181_BINARY E181_LOG_DIRECTORY E181_CAP_F9 E181_CAP_F10 E181_PROBE_COUNT E181_WALKERS
  real_root="$PWD"
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
  # 一格的树：本脚本与共用库的拷贝、假装置、指向真仓 .claude 的符号链接、空的 git 仓。
  make_tree() {
    local tree="$1/tree"
    mkdir -p -- "$tree/scripts/experiment-apparatus/long-run-lib" "$tree/research/results" "$1/bin" "$1/logs"
    cp -- "$SCRIPT_PATH" "$tree/scripts/experiment-apparatus/e181-landing-check-long-run.sh"
    cp -- "$(dirname -- "$SCRIPT_PATH")/long-run-lib/binary-identity.sh" "$tree/scripts/experiment-apparatus/long-run-lib/"
    cp -- "$fakes/device" "$1/bin/device"
    ln -s -- "$real_root/.claude" "$tree/.claude"
    git -C "$tree" init -q > /dev/null 2>&1
  }
  # 在这一格的树里起脚本拷贝；这一格要设的环境变量写在 invoke 前面（INVOKE_BINARY 换装置路径）。
  invoke() {
    local cell="$1"
    shift
    ( cd -- "$cell/tree" && env GIT_CEILING_DIRECTORIES="$work" E181_SELFTEST_CALLS="$cell/calls" E181_BINARY="${INVOKE_BINARY:-$cell/bin/device}" \
        E181_LOG_DIRECTORY="$cell/logs" E181_LONG_RUN_SELFTEST_MARK="$work/mark" \
        bash "$cell/tree/scripts/experiment-apparatus/e181-landing-check-long-run.sh" "$@" > "$cell/out" 2>&1 )
  }
  expect() {
    local name="$1" condition="$2"
    cell_count=$((cell_count + 1))
    if eval "$condition"; then
      echo "  ✓ $name（第 $cell_count 格）"
    else
      echo "  ✗ $name：条件「$condition」不成立"
      [[ -f "$cell/out" ]] && tail -n 6 -- "$cell/out" | sed 's/^/      输出：/'
      echo "     → 怎么办：照上面这一格点名的条件和输出改本脚本，再单独起 bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --selftest 到它退 0"
      failures=$((failures + 1))
    fi
  }
  cell="$work/fresh"; make_tree "$cell"
  original_sha=$(sha256sum -- "$cell/bin/device" | cut -d' ' -f1)
  invoke "$cell"; exit_code=$?
  expect "全新：八片现判、拼出落地产物与复跑登记用的产物、每片片旁记着装置 sha256" '((exit_code == 0)) && grep -qF "现判 8、复用 0" "$cell/out" && (($(wc -l < "$cell/tree/$stem_landing.out") == 23)) && (($(wc -l < "$cell/tree/$stem_checks.out") == 8)) && (($(wc -l < "$cell/calls") == 8)) && (($(ls "$cell"/logs/seg-*.out.binary-sha256 | wc -l) == 8)) && [[ "$(cat "$cell"/logs/seg-*.out.binary-sha256 | sort -u)" == "$original_sha" ]] && grep -qF "装置 sha256 $original_sha（私有副本" "$cell/out"'
  invoke "$cell"; exit_code=$?
  expect "原样重起：八片全沿用、装置一次没起、产物另存带日期的" '((exit_code == 0)) && grep -qF "现判 0、复用 8" "$cell/out" && (($(wc -l < "$cell/calls") == 8)) && [[ -f "$cell/tree/$stem_landing-$today.out" ]]'
  cell="$work/old-binary"; make_tree "$cell"
  invoke "$cell"; first_exit=$?
  old_sha=$(sha256sum -- "$cell/bin/device" | cut -d' ' -f1)
  echo "# a newer build of the device" >> "$cell/bin/device"
  invoke "$cell"; exit_code=$?
  expect "旧装置打的八片全作废：逐片报作废、八片全现跑、旧片连记录整份留在作废目录、片旁记的都是新装置的" '((first_exit == 0 && exit_code == 0)) && grep -qF "现判 8、复用 0" "$cell/out" && (($(grep -c "^作废：.*（装置 sha256 不同）" "$cell/out") == 8)) && grep -q "^作废：checks（装置 sha256 不同）" "$cell/out" && (($(find "$cell"/logs/void-* -name "seg-*.out" | wc -l) == 8)) && (($(find "$cell"/logs/void-* -name "*.binary-sha256" | wc -l) == 8)) && ! grep -q "$old_sha" "$cell"/logs/seg-*.out.binary-sha256 && (($(wc -l < "$cell/calls") == 16))'
  cell="$work/stale-two"; make_tree "$cell"
  invoke "$cell"; first_exit=$?
  printf '%s\n' "$(printf 'f%.0s' {1..64})" > "$cell/logs/seg-narrow.out.binary-sha256"
  rm -f -- "$cell/logs/seg-small.out.binary-sha256"
  invoke "$cell"; exit_code=$?
  expect "片旁记的 sha256 不同的、没有记录的片都作废重跑，另外六片沿用" '((first_exit == 0 && exit_code == 0)) && grep -qF "现判 2、复用 6" "$cell/out" && grep -q "^作废：narrow（装置 sha256 不同）：记的是 f" "$cell/out" && grep -q "^作废：small（装置 sha256 不同）：记的是 没有记录" "$cell/out" && (($(wc -l < "$cell/calls") == 10))'
  cell="$work/swap"; make_tree "$cell"
  swap_target="$cell/bin/device"
  original_sha=$(sha256sum -- "$swap_target" | cut -d' ' -f1)
  E181_SELFTEST_SWAP_TARGET="$swap_target" invoke "$cell"; exit_code=$?
  expect "装置在一趟里不许换：跑着的时候仓根那份被换掉，之后起的八片自报的 sha256 与片旁记的都是开跑时那份" '((exit_code == 0)) && [[ -e "$swap_target.swapped" ]] && [[ "$(sha256sum -- "$swap_target" | cut -d" " -f1)" != "$original_sha" ]] && (($(wc -l < "$cell/calls.self-sha") == 8)) && [[ "$(awk "{print \$NF}" "$cell/calls.self-sha" | sort -u)" == "$original_sha" ]] && [[ "$(cat "$cell"/logs/seg-*.out.binary-sha256 | sort -u)" == "$original_sha" ]]'
  cell="$work/assemble-mixed"; make_tree "$cell"
  invoke "$cell"; first_exit=$?
  printf '%s\n' "$(printf 'f%.0s' {1..64})" > "$cell/logs/seg-f9.out.binary-sha256"
  invoke "$cell" --assemble; exit_code=$?
  expect "拼产物之前各片记的装置 sha256 不同：拒、点名那一片、退 1、不另出产物" '((first_exit == 0 && exit_code == 1)) && grep -qF "片 f9 的装置 sha256 对不上" "$cell/out" && grep -qF "合并产物之前核全部片的装置 sha256" "$cell/out" && (($(ls "$cell/tree/research/results" | wc -l) == 2))'
  cell="$work/assemble-same"; make_tree "$cell"
  invoke "$cell"; first_exit=$?
  invoke "$cell" --assemble; exit_code=$?
  expect "--assemble 八片记的 sha256 彼此相同：不起装置、核过、另存带日期的产物" '((first_exit == 0 && exit_code == 0)) && grep -qF "核过：8 片的装置 sha256 相同" "$cell/out" && (($(wc -l < "$cell/calls") == 8)) && [[ -f "$cell/tree/$stem_landing-$today.out" ]]'
  cell="$work/only"; make_tree "$cell"
  invoke "$cell" --only checks,f11; first_exit=$?
  cp -- "$cell/out" "$cell/out.first"
  invoke "$cell" --assemble; exit_code=$?
  expect "--only 点名的片做完不拼产物；其余片没做完时 --assemble 退 1" '((first_exit == 0 && exit_code == 1)) && (($(wc -l < "$cell/calls") == 2)) && grep -qF "点名的片做完了" "$cell/out.first" && grep -qF "还没做完" "$cell/out" && (($(ls "$cell/tree/research/results" | wc -l) == 0))'
  cell="$work/device-stop"; make_tree "$cell"
  E181_SELFTEST_DEVICE=stop invoke "$cell"; exit_code=$?
  expect "装置报停机：退 3、name=stop 行转出来、不拼产物" '((exit_code == 3)) && grep -qF "name=stop reason=selftest_stop" "$cell/out" && (($(ls "$cell/tree/research/results" | wc -l) == 0))'
  cell="$work/truncated"; make_tree "$cell"
  E181_SELFTEST_DEVICE=truncated invoke "$cell"; exit_code=$?
  expect "末行不齐的片输出不算做完：退 1、没有 seg-*.out、不拼产物" '((exit_code == 1)) && (($(ls "$cell"/logs/seg-*.out 2> /dev/null | wc -l) == 0)) && (($(ls "$cell/tree/research/results" | wc -l) == 0))'
  cell="$work/binary-missing"; make_tree "$cell"
  INVOKE_BINARY="$cell/bin/not-built" invoke "$cell"; exit_code=$?
  expect "装置二进制不在：退 78、装置一次没起" '((exit_code == 78)) && grep -qF "装置二进制不在或不可执行" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/list"; make_tree "$cell"
  invoke "$cell" --list-segments; exit_code=$?
  expect "--list-segments：列八片、不起装置" '((exit_code == 0)) && (($(grep -c "（上限 " "$cell/out") == 8)) && [[ ! -e "$cell/calls" ]]'
  cell="$work/bad-argument"; make_tree "$cell"
  invoke "$cell" --bogus; exit_code=$?
  expect "认不得的参数退 2" '((exit_code == 2)) && grep -qF "认不得的参数「--bogus」" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  rm -rf -- "${work:?}"
  if ((failures > 0)); then echo "e181-landing-check-long-run.sh 自证：$failures 格红（共 $cell_count 格）"; return 1; fi
  echo "e181-landing-check-long-run.sh 自证：$cell_count 格全绿"
  return 0
}

assemble_only=0
list_segments=0
selftest=0
only_text=""
while (($# > 0)); do
  case "$1" in
    --selftest) selftest=1; shift ;;
    --assemble) assemble_only=1; shift ;;
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
if ((assemble_only)); then assemble_the_products; exit $?; fi

[[ -n "${E181_BINARY:-}" ]] || build_the_binary
[[ -x "$BINARY" ]] || { echo "  ✗ 装置二进制不在或不可执行：$BINARY" >&2; echo "     → 怎么办：没给 E181_BINARY 就看上面 cargo build --release 的输出；给了就核对它指向的文件存在且可执行" >&2; exit 78; }
# 开跑时拷一份私有副本：八片都按这份拷贝起，binary_stamp 是拷贝的 sha256
binary_identity_pin "$BINARY" "$LOG_DIRECTORY/pinned-binary" || exit 1
binary_stamp="$PINNED_BINARY_SHA256"
LAUNCH_BINARY="$PINNED_BINARY"
break_is exec-root-binary && LAUNCH_BINARY="$BINARY"

load_one_minute=$(cut -d' ' -f1 /proc/loadavg)
echo "开跑前负载 loadavg1=$load_one_minute（f11 的挂钟读数在机器忙时偏大；装置收尾行会再记一次）；装置 sha256 $binary_stamp（私有副本 $PINNED_BINARY，源 $BINARY）"

reused=0
judged=0
pending=()
void_directory="$LOG_DIRECTORY/void-$(TZ=Asia/Tokyo date +%F)-$$"
for segment in "${wanted[@]}"; do
  if is_complete_output "$LOG_DIRECTORY/seg-$segment.out"; then
    # 做完了还要是这一趟的装置打的；记的不同或没有记录的整份移到作废目录，逐片报作废，现跑
    if break_is stale-binary-reused || binary_identity_is_current "$LOG_DIRECTORY/seg-$segment.out"; then
      reused=$((reused + 1))
    else
      binary_identity_void_shard "$segment" "$LOG_DIRECTORY/seg-$segment.out" "$void_directory" || exit 1
      pending+=("$segment")
    fi
  else
    pending+=("$segment")
  fi
done

contains_segment() { local needle="$1" item; shift; for item in "$@"; do [[ "$item" == "$needle" ]] && return 0; done; return 1; }
run_and_check() { # <片名>
  local segment="$1" code
  run_segment "$segment"; code=$?
  ((code == 0)) || { report_segment_failure "$segment" "$code"; return $?; }
  judged=$((judged + 1))
}

# 计时片先跑，机器上没有别的片在跑
for segment in checks f11; do
  if contains_segment "$segment" "${pending[@]+"${pending[@]}"}"; then run_and_check "$segment" || exit $?; fi
done

# f9 与 F10 那一串并行：每片把退出码写进自己的文件，最后单独一个 wait 等齐
f9_code_file="$LOG_DIRECTORY/f9.code"
rm -f -- "$f9_code_file"
f9_started=0
if contains_segment f9 "${pending[@]+"${pending[@]}"}"; then
  { run_segment f9; echo "$?" > "$f9_code_file"; } &
  f9_started=1
fi
chain_status=0
for segment in measure controls narrow unequal small; do
  contains_segment "$segment" "${wanted[@]}" || continue
  if contains_segment "$segment" "${pending[@]+"${pending[@]}"}"; then
    run_and_check "$segment" || { chain_status=$?; break; }
  fi
done
wait  # shell-lint:exit-collected f9 片的退出码写进 $f9_code_file，下面读它；文件不在就判红
f9_status=0
if ((f9_started)); then
  [[ -f "$f9_code_file" ]] || { echo "  ✗ f9 片的退出码文件不在：$f9_code_file" >&2; echo "     → 怎么办：看 $LOG_DIRECTORY/seg-f9.err 和上面其他片的输出，再起同一条命令（做完的片直接取）" >&2; f9_status=1; }
  if ((f9_status == 0)); then
    f9_status=$(cat -- "$f9_code_file")
    if ((f9_status == 0)); then judged=$((judged + 1)); else report_segment_failure f9 "$f9_status"; f9_status=$?; fi
  fi
fi
echo "现判 $judged、复用 $reused"
((chain_status == 0)) || exit "$chain_status"
((f9_status == 0)) || exit "$f9_status"
if [[ -n "$only_text" ]]; then
  echo "点名的片做完了（不拼产物）：$only_text；片输出在 $LOG_DIRECTORY"
  exit 0
fi
assemble_the_products
