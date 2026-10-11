#!/usr/bin/env bash
# admission: always 每一批补丁都要打一次、验一次，上一次的结论不替这一次作保
# run-condition: command git cargo
# 一批实现员交回之后的「合入后验证」：打全部补丁、编一次、跑 harness 档全部测试二进制、fmt、clippy 与 build，红的列出来。替掉派一个实现员做这件活。
#
#   merge-and-verify.sh [--memory <上限>] [--threads <N>] [--skip-apply] [--copy] <补丁目录> …
#   merge-and-verify.sh --selftest                     # 用假的 apply-writer-patch 与假 cargo 走一遍；MERGE_AND_VERIFY_BREAK=ignore-red 时必须判红
#
# --copy：不往这份脚本所在的仓里打，自己建一份副本在副本里打、编、验，收尾删掉它自己建的副本与编译目录（成功、失败、被 INT / TERM 停都删）：
#   副本 <日志目录>/repo（scripts/source-stamp.py sync 拷：不拷 .git 与任意一层的 target、target-*，拷过去的 mtime 是此刻），
#   补丁经 apply-writer-patch.py --root <副本> 打；编译目录调用方设了 CARGO_TARGET_DIR 就用它、不删（不是这里建的），没设是 <日志目录>/target、收尾删。
#   日志目录留着（日志小，汇总行指着它）。删完打一行「删了副本 … 与编译目录 …（共 …）」；删不成退 1、写明哪一份没删成。
#   要留现场：MERGE_AND_VERIFY_KEEP_COPY=1，收尾不删，打出副本与编译目录的路径和删法，看完自己删（command-safety.md「测试镜像一律放临时目录」：
#   要留着看现场由显式的开关打开、留下时打印路径）。不带 --copy 时照旧打进这份脚本所在的仓（合入主工作区就是这一种），不建副本、不删任何东西。
#
# 为什么：2026-09-20 起 8 天里「合入后验证」派了 3 个实现员、平均 14.3M 折算，是实现员里最贵的一种；它做的前三步（打补丁、编、逐个跑测试）
# 是 apply-writer-patch.py 加几条命令，只有「红了改钉值」要判断——那一步红了才派实现员。
# 做法：每个补丁目录经 scripts/apply-writer-patch.py 打（退 2 就整批停）；编之前经 scripts/source-stamp.py refresh 刷一次源码戳：
# 编译目录（CARGO_TARGET_DIR，不设是树根的 target）是复用的，cargo 按 mtime 判本地包新不新鲜，被 rsync -a 铺进来的新内容带着旧 mtime 会被当成已编
# （records/2026-09-30-里程碑三开工第一轮.md「GPU 与 CPU 不一致调查员交回」那一行）；refresh 把内容或 mtime 与上一次不一样的源文件 touch 到此刻，
# 编译目录里有东西而没有戳（头一次）就全 touch、重编一遍工作区的本地包。刷不成就不编、退 1。手工往副本里拷树用 source-stamp.py sync <源树> <副本>。
# 然后经 run-with-memory-cap.sh 与 capped.sh 跑 test、fmt、clippy、build 四条 cargo，与上游 .claude/singlefs-ai-sop/scripts/check.sh 同一套判据：
#   test   `cargo test --offline --no-fail-fast -p <包>…`：包是工作区成员（树根 Cargo.toml 的 [workspace] members，逐个读成员 Cargo.toml 的包名）
#          减掉 checker 档包 singlefs-checker-tier（那归提交时的主 agent）；check.sh 跑的是 `cargo test --all`，差的只有这一个包。
#          成员里没有 singlefs-checker-tier（改了名、挪了）、成员写成通配、读不出包名，都判红、不编：排不掉它就宁可不跑。
#          退 0 而 test result 行里一条测试都没数到，也判红（check.sh「一条测试都没跑」那一条）。
#   fmt    `cargo fmt --all -- --check`；clippy `cargo clippy --offline --all-targets --all-features -- -D warnings` 加编码纪律那张 lint 表；build `cargo build --offline --all-targets`。
#   lint 表只在 check.sh 一处定义：每次开跑从它的 `CODE_DISCIPLINE_LINTS=(` … `)` 那一段现读（一条一对 `-D clippy::<名>`），不在这里另抄一份；
#   同时核 check.sh 里 fmt、clippy、build、test 四条 cargo 的写法（CHECK_SH_COMMANDS）还是这里照着对齐的那一种。读不出表、表的形状不对、
#   哪一条写法变了，都判红、一个补丁都不打：上游改了判据，先回来对齐这里。
# 日志落 $MERGE_AND_VERIFY_LOG_DIRECTORY（默认工作根下的 merge-and-verify/<日期>-<进程号>，工作根只在 .claude/hooks/lib_work_root.py 里定义）。
# cargo test 那一趟内存包装退 245（显存不够，run-with-memory-cap.sh 文件头「退出码」）：这一趟的测试结果（连同 FAILED 行）不算数，记一项红、
#   报「显存不够」、不列 FAILED 也不给只重跑它们的命令（不是代码红），出路是看谁在用卡、等它跑完重跑这一条。退 250–254 照旧报包装自己的结局。
# MERGE_AND_VERIFY_TREE（只给自证）：刷源码戳与编译在这棵树里（默认这份脚本所在的仓根）；打补丁照旧是 apply-writer-patch.py 自己的仓根。
# MERGE_AND_VERIFY_CHECK_SH（只给自证）：从哪一份 check.sh 读 lint 表与四条写法（默认这份脚本所在仓的 .claude/singlefs-ai-sop/scripts/check.sh）。
# 退出码：0 全绿；1 有红（测试、fmt、clippy、build、apply 退 1、显存不够、源码戳刷不成、check.sh 读不出、工作区成员排不掉 checker 档包）；
#   2 补丁核不过（apply 退 2）、用法错。
# 弄坏开关 MERGE_AND_VERIFY_BREAK=<项>（只给证红用；打开时自证退 1）。逐项写开关名、弄坏哪一处、打开后自证哪几项红
# （自证的三例没有标签，项名直接写那条「✗ 自检：」失败行的句首，括号里写那一行接着报的）：
#   MERGE_AND_VERIFY_BREAK=ignore-red  verify() 抓到 FAILED 行、cargo test 退非 0 都不记红，一律报测试全绿
#       「一条测试红时应当退 1、列出那条测试与只重跑它的命令」（实际退出码 0，报「✓ 测试全绿（0 个测试二进制）」）
#   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（lib_work_root.py 的开关：工作根退回旧根 /tmp/claude-<uid>）  没设日志目录时日志落进旧根
#       「没设日志目录时日志应当落在工作根下」（跑完删掉旧根下多出的那个 merge-and-verify/<日期>-<进程号>）
#   MERGE_AND_VERIFY_BREAK=no-source-stamp  编之前不刷源码戳
#       「往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验，应当重编改过的 walk.rs、测试全绿」（实际退出码 1，假 cargo 报 Fresh、那一条 FAILED）
#   MERGE_AND_VERIFY_BREAK=gpu-memory-as-test-red  cargo test 那一趟包装退 245 也照 FAILED 行报测试红
#       「cargo test 那一趟包装退 245 时应当报显存不够、退 1，不列 FAILED、不给重跑命令」（实际报「红的测试 1 条」与重跑命令）
#   MERGE_AND_VERIFY_BREAK=drop-discipline-lint  从 check.sh 读到的 lint 表丢掉最后一条再交给 clippy
#       「clippy 应当带上 check.sh 的 lint 表里每一条」（实际少一条 -D clippy::shadow_unrelated 这类的）
#   MERGE_AND_VERIFY_BREAK=lint-table-optional  check.sh 读不出 lint 表、或四条写法对不上时只提一句、照常往下验
#       「check.sh 里读不出 lint 表时应当退 1、不编」与「check.sh 的 clippy 那一行变了时应当退 1、不编」（实际退 0、照常编）
#   MERGE_AND_VERIFY_BREAK=skip-build  不跑 cargo build --offline --all-targets
#       「应当跑 cargo build --offline --all-targets」（实际假 cargo 没收到 build）
#   MERGE_AND_VERIFY_BREAK=fixed-harness-packages  cargo test 退回写死的四个包 singlefs-format、singlefs-core、singlefs-harness、singlefs-checker
#       「cargo test 应当挑工作区成员减掉 checker 档包」（实际收到的是写死的四个包，样本树的成员 demo 不在里面）；
#       「工作区成员里没有 singlefs-checker-tier 时应当退 1、不编」也红（写死的表不读成员，照编）
#   MERGE_AND_VERIFY_BREAK=no-checker-tier-guard  工作区成员里找不到 singlefs-checker-tier 也照挑全部成员
#       「工作区成员里没有 singlefs-checker-tier 时应当退 1、不编」（实际退 0，改了名的 checker 档包被跑进来）
#   MERGE_AND_VERIFY_BREAK=zero-tests-green  cargo test 退 0 而一条测试都没数到时照报全绿
#       「cargo test 退 0 却一条测试都没跑时应当退 1」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=keep-copy  --copy 收尾不删副本与编译目录、也不打路径（改前的样子：副本留在工作根下没人删）
#       「--copy 全绿时应当在副本里打补丁、收尾删掉副本与编译目录」与「--copy 测试红时也应当删掉副本与编译目录」（实际两份都还在）
#   MERGE_AND_VERIFY_BREAK=cleanup-only-on-success  --copy 只在退 0 时删副本与编译目录，红了留着
#       「--copy 测试红时也应当删掉副本与编译目录」（实际退 1、两份都还在）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
APPLY="${MERGE_AND_VERIFY_APPLY:-$here/apply-writer-patch.py}"
WRAPPER="${MERGE_AND_VERIFY_WRAPPER:-$here/../hosts/run-with-memory-cap.sh}"
CAPPED="${MERGE_AND_VERIFY_CAPPED:-$here/../hosts/capped.sh}"
STAMP="$here/../hosts/source-stamp.py"
GPU_MEMORY_HIT_EXIT=245   # run-with-memory-cap.sh：显存不够（命令退了非 0、起止之间内核报了 NV_ERR_NO_MEMORY），结果不算数
CHECK_SH="${MERGE_AND_VERIFY_CHECK_SH:-$root/.claude/singlefs-ai-sop/scripts/check.sh}"
CHECKER_TIER_PACKAGE=singlefs-checker-tier   # checker 档包：只归提交时的主 agent（.claude/rules/verification.md），这里的 cargo test 不挑它
# check.sh 里四条 cargo 的写法：每条在它里面恰好出现一次，这里的 test、fmt、clippy、build 照它们对齐（test 只少一个 checker 档包）。
CHECK_SH_COMMANDS=(
  'run_cargo fmt --all -- --check '
  'run_cargo clippy --all-targets --all-features -- -D warnings "${CODE_DISCIPLINE_LINTS[@]}"'
  'run_cargo build --all-targets '
  'run_cargo test --all 2>&1'
)

die() { echo "  ✗ $1"; echo "  → 怎么办：$2"; exit "${3:-2}"; }

# --copy 收尾（EXIT trap）：删掉这里建的副本与编译目录；MERGE_AND_VERIFY_KEEP_COPY=1 时留着、打路径。删不成退 1。
COPY_TREE=""; COPY_TARGET=""
cleanup_copy() {
  local rc=$? kept_size
  trap - EXIT
  [[ -n "$COPY_TREE" ]] || exit "$rc"
  cd / || true
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "keep-copy" ]] || { [[ "${MERGE_AND_VERIFY_BREAK:-}" == "cleanup-only-on-success" ]] && ((rc != 0)); }; then
    exit "$rc"
  fi
  if [[ "${MERGE_AND_VERIFY_KEEP_COPY:-}" == 1 ]]; then
    echo "  ! 留着副本 $COPY_TREE${COPY_TARGET:+ 与编译目录 $COPY_TARGET}（MERGE_AND_VERIFY_KEEP_COPY=1）"
    echo "  → 看完自己删：rm -rf -- '$COPY_TREE'${COPY_TARGET:+ '$COPY_TARGET'}"
    exit "$rc"
  fi
  kept_size="$(du -sch -- "$COPY_TREE" ${COPY_TARGET:+"$COPY_TARGET"} 2>/dev/null | tail -n 1 | cut -f1)"
  rm -rf -- "${COPY_TREE:?}"
  [[ -z "$COPY_TARGET" ]] || rm -rf -- "${COPY_TARGET:?}"
  if [[ -e "$COPY_TREE" || ( -n "$COPY_TARGET" && -e "$COPY_TARGET" ) ]]; then
    echo "  ✗ 副本 $COPY_TREE 或编译目录 ${COPY_TARGET:-（没建）} 没删成"
    echo "  → 怎么办：看谁还开着里面的文件（ls -l /proc/*/cwd 2>/dev/null | grep '$COPY_TREE'），停了再 rm -rf -- 那一份"
    exit 1
  fi
  echo "  删了副本 $COPY_TREE${COPY_TARGET:+ 与编译目录 $COPY_TARGET}（共 ${kept_size:-?}；日志留在 $(dirname "$COPY_TREE")）"
  exit "$rc"
}

# read_check_sh <check.sh>：现读编码纪律那张 lint 表进 DISCIPLINE_LINTS，核四条 cargo 写法；读不出、对不上返回 1，原因进 CHECK_SH_PROBLEM。
read_check_sh() {
  local file="$1" line in_table=0 i command_text hits
  local -a words=()
  DISCIPLINE_LINTS=(); CHECK_SH_PROBLEM=""
  [[ -f "$file" ]] || { CHECK_SH_PROBLEM="$file 不在"; return 1; }
  while IFS= read -r line || [[ -n "$line" ]]; do
    if ((in_table == 1)); then
      if [[ "$line" =~ ^\)[[:space:]]*$ ]]; then in_table=2; break; fi
      line="${line%%#*}"
      read -r -a words <<< "$line"
      DISCIPLINE_LINTS+=(${words[@]+"${words[@]}"})
    elif [[ "$line" =~ ^CODE_DISCIPLINE_LINTS=\([[:space:]]*$ ]]; then
      in_table=1
    fi
  done < "$file"
  if ((in_table != 2)); then CHECK_SH_PROBLEM="$file 里找不到 CODE_DISCIPLINE_LINTS=( … ) 那一段（起止各占一行）"; return 1; fi
  if ((${#DISCIPLINE_LINTS[@]} == 0 || ${#DISCIPLINE_LINTS[@]} % 2 != 0)); then
    CHECK_SH_PROBLEM="$file 的 CODE_DISCIPLINE_LINTS 读出 ${#DISCIPLINE_LINTS[@]} 个词，不是一条一对的「-D clippy::<名>」"; return 1
  fi
  for ((i = 0; i < ${#DISCIPLINE_LINTS[@]}; i += 2)); do
    if [[ ! "${DISCIPLINE_LINTS[i]}" =~ ^-[DWAF]$ || ! "${DISCIPLINE_LINTS[i+1]}" =~ ^clippy::[a-z0-9_]+$ ]]; then
      CHECK_SH_PROBLEM="$file 的 CODE_DISCIPLINE_LINTS 第 $((i / 2 + 1)) 条读成「${DISCIPLINE_LINTS[i]} ${DISCIPLINE_LINTS[i+1]}」，不是「-D clippy::<名>」的形状"; return 1
    fi
  done
  for command_text in "${CHECK_SH_COMMANDS[@]}"; do
    hits="$(grep -cF -- "$command_text" "$file")"
    if [[ "$hits" != 1 ]]; then CHECK_SH_PROBLEM="$file 里「$command_text」出现 ${hits:-0} 次，不是 1 次：上游改了这一条 cargo 的写法"; return 1; fi
  done
  return 0
}

# test_packages <树>：工作区成员减掉 checker 档包，逐个写成「-p <包名>」进 TEST_PACKAGES；排不掉 checker 档包、读不出返回 1，原因进 TEST_PACKAGES_PROBLEM。
test_packages() {
  local listing
  TEST_PACKAGES=(); TEST_PACKAGES_PROBLEM=""
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "fixed-harness-packages" ]]; then
    TEST_PACKAGES=(-p singlefs-format -p singlefs-core -p singlefs-harness -p singlefs-checker); return 0
  fi
  if ! listing="$(python3 - "$1" "$CHECKER_TIER_PACKAGE" "${MERGE_AND_VERIFY_BREAK:-}" <<'PY_MEMBERS' 2>&1
import os, sys, tomllib
tree, excluded, broken = sys.argv[1], sys.argv[2], sys.argv[3]
def fail(text):
    print(text); sys.exit(1)
try:
    with open(os.path.join(tree, "Cargo.toml"), "rb") as handle:
        members = tomllib.load(handle).get("workspace", {}).get("members")
except (OSError, tomllib.TOMLDecodeError) as error:
    fail(f"读不了 {tree}/Cargo.toml：{error}")
if not isinstance(members, list) or not members:
    fail(f"{tree}/Cargo.toml 没有 [workspace] members")
names = []
for member in members:
    if not isinstance(member, str) or any(mark in member for mark in "*?["):
        fail(f"成员「{member}」写成了通配或不是字符串，这里只认逐个写出的目录")
    try:
        with open(os.path.join(tree, member, "Cargo.toml"), "rb") as handle:
            name = tomllib.load(handle).get("package", {}).get("name")
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"读不了成员 {member} 的 Cargo.toml：{error}")
    if not isinstance(name, str) or not name:
        fail(f"成员 {member} 的 Cargo.toml 没有 [package] name")
    names.append(name)
if excluded not in names and broken != "no-checker-tier-guard":
    fail(f"工作区成员（{'、'.join(names)}）里没有 {excluded}：checker 档包改了名或挪了，排不掉它")
kept = [name for name in names if name != excluded]
if not kept:
    fail(f"工作区成员减掉 {excluded} 之后一个包都不剩")
print("\n".join(kept))
PY_MEMBERS
  )"; then
    TEST_PACKAGES_PROBLEM="$listing"; return 1
  fi
  local name
  while IFS= read -r name; do [[ -n "$name" ]] && TEST_PACKAGES+=(-p "$name"); done <<< "$listing"
  return 0
}

run_capped() {   # $1 日志文件，其余是命令；经内存包装与线程上限（WRAPPER 为 none 时直接跑，自检用）
  local log="$1"; shift
  if [[ "$WRAPPER" == "none" ]]; then "$@" > "$log" 2>&1
  else bash "$WRAPPER" "$MEMORY" bash "$CAPPED" "$THREADS" "$@" > "$log" 2>&1; fi
}

verify() {
  local logs="$1"; shift
  local -a dirs=("$@")
  local dir rc applied=0 warned=0 failed_tests reds=0 test_count
  local -a clippy_lints=()
  mkdir -p "$logs"
  if ! read_check_sh "$CHECK_SH"; then
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "lint-table-optional" ]]; then
      echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=lint-table-optional：$CHECK_SH_PROBLEM，照常往下验"
    else
      die "check.sh 读不出、或与这里对不上：$CHECK_SH_PROBLEM；一个补丁都没打、没编" \
        "上游 check.sh 的 lint 表或那四条 cargo 写法变了：先照它改这份脚本的 CHECK_SH_COMMANDS 与 verify()，让合入后验证与提交时的判据仍是同一套" 1
    fi
  fi
  clippy_lints=(${DISCIPLINE_LINTS[@]+"${DISCIPLINE_LINTS[@]}"})
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "drop-discipline-lint" && ${#clippy_lints[@]} -ge 2 ]]; then
    clippy_lints=("${clippy_lints[@]:0:${#clippy_lints[@]}-2}")
  fi
  for dir in "${dirs[@]}"; do
    [[ -d "$dir" ]] || die "补丁目录不在：$dir" "给实现员交回的补丁目录（里面有 crates.patch 或 mutations-append.tsv）"
    python3 "$APPLY" "$dir" ${COPY_TREE:+--root "$COPY_TREE"} > "$logs/apply-$(basename "$dir").log" 2>&1; rc=$?
    case "$rc" in
      0) applied=$((applied+1)); echo "  打上：$dir" ;;
      1) applied=$((applied+1)); warned=$((warned+1)); echo "  打上、但 code-source-discipline 红：$dir（日志 $logs/apply-$(basename "$dir").log）" ;;
      *) die "补丁核不过、一个字没改：$dir（apply-writer-patch.py 退 $rc，日志 $logs/apply-$(basename "$dir").log）" "按那份日志给的下一步让实现员改补丁再交" 2 ;;
    esac
  done
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "no-source-stamp" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=no-source-stamp：编之前不刷源码戳"
  elif python3 "$STAMP" refresh "$tree" > "$logs/source-stamp.log" 2>&1; then
    echo "  源码戳：$(grep '^SOURCE_STAMP ' "$logs/source-stamp.log" | tail -1)"
  else
    die "编之前没刷成源码戳，没编（日志 $logs/source-stamp.log）" "照那份日志修好再跑；刷不成就别复用这个编译目录（设 CARGO_TARGET_DIR 换一个新的）" 1
  fi
  test_packages "$tree" || die "挑不出要测的包：$TEST_PACKAGES_PROBLEM；没编" \
    "工作区成员要逐个写出目录、各有 [package] name，checker 档包 $CHECKER_TIER_PACKAGE 要在成员里（这里靠它的名字把它排掉）；改了名就同步改这份脚本的 CHECKER_TIER_PACKAGE" 1
  echo "  编译并跑 ${TEST_PACKAGES[*]} 的全部测试二进制（工作区成员减掉 $CHECKER_TIER_PACKAGE；内存上限 $MEMORY、线程上限 $THREADS）…"
  run_capped "$logs/test.log" cargo test --offline --no-fail-fast "${TEST_PACKAGES[@]}"; rc=$?
  test_count="$(awk 'match($0, /^test result: [A-Za-z]+\. [0-9]+ passed; [0-9]+ failed/) { split($0, f, " "); passed += f[4]; failed += f[6] } END { print passed + failed + 0 }' "$logs/test.log")"
  failed_tests="$(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' "$logs/test.log")"
  if [[ $rc -eq $GPU_MEMORY_HIT_EXIT && "${MERGE_AND_VERIFY_BREAK:-}" != "gpu-memory-as-test-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 那一趟显存不够（内存包装退 $GPU_MEMORY_HIT_EXIT：起止之间内核报了 NV_ERR_NO_MEMORY），这一趟的测试结果不算数（日志 $logs/test.log）"
    echo "  → 怎么办：nvidia-smi 看谁在用卡，等它跑完再重跑这一条 merge-and-verify.sh；不是代码红，别照日志里的 FAILED 改钉值"
  elif [[ -n "$failed_tests" && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ 红的测试 $(printf '%s\n' "$failed_tests" | wc -l) 条："
    printf '%s\n' "$failed_tests" | sed 's/^/      /'   # gate-lint:detail
    echo "  → 怎么办：只重跑红的这几条：python3 scripts/rerun-failed-tests.py $logs/test.log -p <包>；照实现员报告里的算法改钉值，改完再只跑这几条"
  elif [[ $rc -ne 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 $rc 而一条 FAILED 都没抓到（编不过、或包装退 250–254）：$(tail -3 "$logs/test.log" | tr '\n' ' ')"
    echo "  → 怎么办：看 $logs/test.log；退 250–254 是内存包装自己的结局，按 run-with-memory-cap.sh 文件头处理"
  elif [[ "$test_count" == 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "zero-tests-green" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 0，却一条测试都没跑（test result 行里 passed 与 failed 加起来是 0；日志 $logs/test.log）"
    echo "  → 怎么办：看那份日志里编了哪些包、有没有 test result 行；一条都没跑的「全绿」与跑过了长得一样（check.sh 同一条）"
  else
    echo "  ✓ 测试全绿（$(grep -c '^test result: ok' "$logs/test.log") 个测试二进制，共 $test_count 条测试）"
  fi
  run_capped "$logs/fmt.log" cargo fmt --all -- --check; rc=$?
  if [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ cargo fmt --check 红（日志 $logs/fmt.log）"; echo "  → 怎么办：cargo fmt --all，再看 diff 只动了格式"; else echo "  ✓ fmt 过"; fi
  run_capped "$logs/clippy.log" cargo clippy --offline --all-targets --all-features -- -D warnings ${clippy_lints[@]+"${clippy_lints[@]}"}; rc=$?
  if [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ clippy 红（日志 $logs/clippy.log）"; echo "  → 怎么办：按日志逐条改，不加 allow 绕过"; else echo "  ✓ clippy 过（带 check.sh 的编码纪律 lint $((${#clippy_lints[@]} / 2)) 条）"; fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-build" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-build：不跑 cargo build"
  else
    run_capped "$logs/build.log" cargo build --offline --all-targets; rc=$?
    if [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ cargo build --all-targets 红（日志 $logs/build.log）"; echo "  → 怎么办：从日志里第一条 rustc 报错改起，后面的多半是它的连锁反应"; else echo "  ✓ build 过"; fi
  fi
  echo "  汇总：打上 ${applied} 个补丁目录（其中 code-source-discipline 红 $warned 个），验证红 $reds 项，日志在 $logs"
  [[ $reds -eq 0 && $warned -eq 0 ]]
}

selftest() {
  local scratch fake_cargo_dir failures=0 checked=0 out rc expected_lints expected_lint_count check_case
  unset MERGE_AND_VERIFY_KEEP_COPY   # 留现场那一格自己设；调用方设了不能带进别的格
  scratch="$(mktemp -d)"; fake_cargo_dir="$scratch/bin"; mkdir -p "$fake_cargo_dir" "$scratch/patch-a" "$scratch/patch-b"
  cat > "$scratch/fake-apply.py" <<'FAKE'
import sys
if "--root" in sys.argv:
    print("root=" + sys.argv[sys.argv.index("--root") + 1])
sys.exit(2 if sys.argv[1].endswith("patch-b") else 0)
FAKE
  cat > "$fake_cargo_dir/cargo" <<'FAKE'
#!/usr/bin/env bash
[[ -z "${FAKE_CARGO_CALLS:-}" ]] || printf '%s\n' "$*" >> "$FAKE_CARGO_CALLS"
case "$1" in
  test) [[ "${FAKE_CARGO_MODE:-}" == "empty" ]] && exit 0
        printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts_in_%s ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed\n' "${FAKE_CARGO_MODE:-x}"; [[ "${FAKE_CARGO_MODE:-}" == "green" ]] && exit 0 || exit 101 ;;
  fmt|clippy) exit 0 ;;
esac
FAKE
  [[ "${FAKE_CARGO_MODE:-}" == "" ]] || true
  chmod +x "$fake_cargo_dir/cargo"
  # 刷源码戳与编译都在临时树里（MERGE_AND_VERIFY_TREE），不碰本仓的源文件与编译目录
  mkdir -p "$scratch/tree/crates/demo/src"; printf 'pub fn f() {}\n' > "$scratch/tree/crates/demo/src/lib.rs"
  # 样本工作区两个成员：demo 与 checker 档包；cargo test 只该挑 demo
  write_workspace() {   # $1 树，$2 checker 档那个成员的包名
    mkdir -p "$1/crates/demo" "$1/crates/singlefs-checker-tier"
    printf '[workspace]\nmembers = ["crates/demo", "crates/singlefs-checker-tier"]\n' > "$1/Cargo.toml"
    printf '[package]\nname = "demo"\n' > "$1/crates/demo/Cargo.toml"
    printf '[package]\nname = "%s"\n' "$2" > "$1/crates/singlefs-checker-tier/Cargo.toml"
  }
  write_workspace "$scratch/tree" singlefs-checker-tier
  run_case() { PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" "${@:2}" > "$out" 2>&1; rc=$?; }
  out="$scratch/out1.txt"; run_case one "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -q 'subtracts_in_x' "$out" || ! grep -q 'rerun-failed-tests.py' "$out"; then
    echo "  ✗ 自检：一条测试红时应当退 1、列出那条测试与只重跑它的命令，实际退出码 $rc：$(tail -4 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  out="$scratch/out2.txt"; run_case two "$scratch/patch-a" "$scratch/patch-b"; checked=$((checked+1))
  if [[ $rc -ne 2 ]] || ! grep -q '补丁核不过' "$out"; then
    echo "  ✗ 自检：apply 退 2 时应当整批停、退 2，实际 $rc：$(tail -2 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # --copy 测试红（假 cargo 这时还报一条 FAILED）：退 1，自己建的副本与编译目录照样删掉
  out="$scratch/out-copy-red.txt"; run_case copy-red --copy "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || [[ -e "$scratch/logs-copy-red/repo" || -e "$scratch/logs-copy-red/target" ]] || ! grep -q '删了副本' "$out"; then
    echo "  ✗ 自检：--copy 测试红时也应当删掉副本与编译目录，实际退出码 $rc，日志目录里还有 $(ls "$scratch/logs-copy-red" 2>/dev/null | tr '\n' ' ')：$(grep -E '✗|删了|建副本' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 假 cargo 全绿的一趟：把 FAILED 行换掉
  sed -i.bak 's/ \.\.\. FAILED/ ... ok/; s/FAILED\. 1 passed; 1 failed/ok. 2 passed; 0 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$scratch/out3.txt"; FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-three.txt" run_case three "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '测试全绿' "$out"; then
    echo "  ✗ 自检：全绿时应当退 0 并说全绿，实际 $rc：$(tail -3 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # --copy 全绿：补丁打进副本（apply 收到 --root <日志目录>/repo）、源码戳记在 <日志目录>/target，收尾两份都删掉、打一行删了什么
  out="$scratch/out-copy-green.txt"; FAKE_CARGO_MODE=green run_case copy-green --copy "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || [[ -e "$scratch/logs-copy-green/repo" || -e "$scratch/logs-copy-green/target" ]] || ! grep -q '删了副本' "$out" \
     || ! grep -qxF "root=$scratch/logs-copy-green/repo" "$scratch/logs-copy-green/apply-patch-a.log" 2>/dev/null \
     || ! grep -qF "stamp=$scratch/logs-copy-green/target/" "$out"; then
    echo "  ✗ 自检：--copy 全绿时应当在副本里打补丁、收尾删掉副本与编译目录，实际退出码 $rc，日志目录里还有 $(ls "$scratch/logs-copy-green" 2>/dev/null | tr '\n' ' ')，apply 收到 $(cat "$scratch/logs-copy-green/apply-patch-a.log" 2>/dev/null)：$(grep -E '✗|删了|源码戳' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # MERGE_AND_VERIFY_KEEP_COPY=1：留着副本与编译目录、打出路径
  out="$scratch/out-copy-kept.txt"; FAKE_CARGO_MODE=green MERGE_AND_VERIFY_KEEP_COPY=1 run_case copy-kept --copy "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || [[ ! -d "$scratch/logs-copy-kept/repo" ]] || ! grep -qF "留着副本 $scratch/logs-copy-kept/repo 与编译目录 $scratch/logs-copy-kept/target" "$out"; then
    echo "  ✗ 自检：MERGE_AND_VERIFY_KEEP_COPY=1 时应当留着副本与编译目录、打出路径，实际退出码 $rc：$(grep -E '✗|留着|删了' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 与 check.sh 同一套判据：lint 表在这里另取一遍（sed 取那一段、grep 数条数），与脚本里 read_check_sh 不是同一段代码
  expected_lints="$(sed -n '/^CODE_DISCIPLINE_LINTS=(/,/^)/p' "$CHECK_SH" | grep -oE -- '-[DWAF] clippy::[a-z0-9_]+' | tr '\n' ' ')"
  expected_lints="${expected_lints% }"
  expected_lint_count="$(sed -n '/^CODE_DISCIPLINE_LINTS=(/,/^)/p' "$CHECK_SH" | grep -cE -- '-[DWAF] clippy::')"
  checked=$((checked+1))
  if [[ "$expected_lint_count" -lt 1 ]] || ! grep -qxF -- "clippy --offline --all-targets --all-features -- -D warnings $expected_lints" "$scratch/calls-three.txt"; then
    echo "  ✗ 自检：clippy 应当带上 check.sh 的 lint 表里每一条（$CHECK_SH 里数到 $expected_lint_count 条），假 cargo 收到的是：$(grep '^clippy' "$scratch/calls-three.txt")"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  if ! grep -qxF -- "build --offline --all-targets" "$scratch/calls-three.txt"; then
    echo "  ✗ 自检：应当跑 cargo build --offline --all-targets（check.sh 的构建那一条），假 cargo 收到的是：$(tr '\n' '|' < "$scratch/calls-three.txt")"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  if ! grep -qxF -- "test --offline --no-fail-fast -p demo" "$scratch/calls-three.txt"; then
    echo "  ✗ 自检：cargo test 应当挑工作区成员减掉 checker 档包（样本里只剩 -p demo），假 cargo 收到的是：$(grep '^test' "$scratch/calls-three.txt")"; failures=1; fi   # gate-lint:detail
  # check.sh 读不出 lint 表、或 clippy 那一行变了：退 1，一个补丁都不打、不编
  sed '/^CODE_DISCIPLINE_LINTS=(/,/^)/d' "$CHECK_SH" > "$scratch/check-no-table.sh"
  sed 's/-- -D warnings "\${CODE_DISCIPLINE_LINTS\[@\]}"/-- -D warnings -W clippy::pedantic "${CODE_DISCIPLINE_LINTS[@]}"/' "$CHECK_SH" > "$scratch/check-clippy-changed.sh"
  for check_case in no-table clippy-changed; do
    out="$scratch/out-check-$check_case.txt"; checked=$((checked+1))
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-check-$check_case.txt" MERGE_AND_VERIFY_CHECK_SH="$scratch/check-$check_case.sh" run_case "check-$check_case" --skip-apply
    if [[ $rc -ne 1 ]] || ! grep -q 'check.sh 读不出、或与这里对不上' "$out" || [[ -s "$scratch/calls-check-$check_case.txt" ]]; then
      echo "  ✗ 自检：check.sh $([[ $check_case == no-table ]] && echo '里读不出 lint 表' || echo '的 clippy 那一行变了')时应当退 1、不编，实际退出码 $rc，假 cargo 收到 $(cat "$scratch/calls-check-$check_case.txt" 2>/dev/null | wc -l) 条：$(grep -E '✗|!' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  done
  # 工作区成员里没有 singlefs-checker-tier（样本里那个成员改了名）：排不掉它，退 1、不编
  write_workspace "$scratch/tree-renamed" singlefs-checker-tier-renamed
  mkdir -p "$scratch/tree-renamed/crates/demo/src"; printf 'pub fn f() {}\n' > "$scratch/tree-renamed/crates/demo/src/lib.rs"
  out="$scratch/out-renamed.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-renamed.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-renamed" MERGE_AND_VERIFY_TREE="$scratch/tree-renamed" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 ]] || ! grep -q '排不掉它' "$out" || grep -q '^test' "$scratch/calls-renamed.txt" 2>/dev/null; then
    echo "  ✗ 自检：工作区成员里没有 singlefs-checker-tier 时应当退 1、不编，实际退出码 $rc：$(grep '^test' "$scratch/calls-renamed.txt" 2>/dev/null | tr '\n' ' ')$(grep -E '✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  # cargo test 退 0 而一条测试都没跑（假 cargo 什么都不打）：退 1
  out="$scratch/out-empty.txt"; checked=$((checked+1)); FAKE_CARGO_MODE=empty run_case empty --skip-apply
  if [[ $rc -ne 1 ]] || ! grep -q '一条测试都没跑' "$out"; then
    echo "  ✗ 自检：cargo test 退 0 却一条测试都没跑时应当退 1，实际退出码 $rc：$(grep -E '✓ 测试|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 没设日志目录：日志落在工作根下（SINGLEFS_WORK_ROOT 指进临时目录）
  out="$scratch/out4.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none SINGLEFS_WORK_ROOT="$scratch/work-root" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u MERGE_AND_VERIFY_LOG_DIRECTORY -u CARGO_TARGET_DIR bash "$0" "$scratch/patch-a" > "$out" 2>&1; rc=$?
  if [[ $rc -ne 0 ]] || ! compgen -G "$scratch/work-root/merge-and-verify/*/test.log" > /dev/null; then
    echo "  ✗ 自检：没设日志目录时日志应当落在工作根下（$scratch/work-root/merge-and-verify/<日期>-<进程号>/），实际退出码 $rc：$(grep '日志在' "$out" | tail -1)"; failures=1; fi   # gate-lint:detail
  # 换了源码却没重编：临时副本先验一趟（编过），再从源树 rsync -a 铺进新的 walk.rs（mtime 比编译目录里的 dep-info 旧，取 2020 年初的 epoch 秒）；
  # 假 cargo 照 mtime 判新不新鲜（source-stamp.py --write-selftest-cargo 写的那一份），没刷源码戳就认 walk.rs 已编、那一条 FAILED
  local stale_tree="$scratch/stale-tree" stale_source="$scratch/stale-source" stale_bin="$scratch/stale-bin"
  mkdir -p "$stale_tree/crates/demo/src" "$stale_source/crates/demo/src" "$stale_bin"
  write_workspace "$stale_tree" singlefs-checker-tier
  python3 "$STAMP" --write-selftest-cargo "$stale_bin/cargo"
  printf 'pub mod walk;\n' > "$stale_tree/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 1 }\n' > "$stale_tree/crates/demo/src/walk.rs"
  touch -d @1577836800 "$stale_tree/crates/demo/src/lib.rs" "$stale_tree/crates/demo/src/walk.rs"
  cp -p "$stale_tree/crates/demo/src/lib.rs" "$stale_source/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 22 }\n' > "$stale_source/crates/demo/src/walk.rs"
  touch -d @1577836900 "$stale_source/crates/demo/src/walk.rs"
  stale_case() { PATH="$stale_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$stale_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?; }
  out="$scratch/out5-first.txt"; stale_case five-first
  rsync -a "$stale_source/" "$stale_tree/"
  out="$scratch/out5.txt"; stale_case five; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q 'Compiling crates/demo/src/walk.rs' "$scratch/logs-five/test.log" 2>/dev/null || ! grep -q '测试全绿' "$out"; then
    echo "  ✗ 自检：往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验，应当重编改过的 walk.rs、测试全绿，实际退出码 $rc：$(grep -E 'Fresh|Compiling|FAILED' "$scratch/logs-five/test.log" 2>/dev/null | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # cargo test 那一趟内存包装退 245（显存不够）：假包装打一条 FAILED 再退 245
  cat > "$scratch/fake-wrapper-245.sh" <<'FAKE'
#!/usr/bin/env bash
# 假内存包装：cargo test 那一趟打一条 FAILED、退 245（显存不够）；fmt、clippy 照退 0
for word in "$@"; do
  if [[ "$word" == test ]]; then
    printf 'test tests::on_gpu ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed\nrun-with-memory-cap: 显存不够（假包装），退 245\n'
    exit 245
  fi
done
exit 0
FAKE
  out="$scratch/out6.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER="$scratch/fake-wrapper-245.sh" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-six" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 ]] || ! grep -q '显存不够' "$out" || grep -q 'rerun-failed-tests.py' "$out" || grep -q '红的测试' "$out"; then
    echo "  ✗ 自检：cargo test 那一趟包装退 245 时应当报显存不够、退 1，不列 FAILED、不给重跑命令，实际退出码 $rc：$(grep -E '✗|→' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -rf "${scratch:?}"
  if [[ $failures -ne 0 ]]; then
    echo "    → 怎么办：看 verify() 抓 FAILED 行与退出码、编之前刷源码戳的那几处；MERGE_AND_VERIFY_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 种）：测试红时退 1 并列出红的与重跑命令、补丁核不过整批停退 2、全绿退 0、没设日志目录时日志落在工作根下、"
  echo "    往编过的副本里 rsync -a 铺旧 mtime 的新内容之后照样重编、包装退 245 报显存不够不报测试红、"
  echo "    clippy 带上 check.sh 的 $expected_lint_count 条编码纪律 lint、跑 build、cargo test 挑工作区成员减掉 checker 档包、"
  echo "    check.sh 读不出 lint 表或 clippy 那一行变了退 1 不编、成员里没有 checker 档包退 1 不编、一条测试都没跑退 1、"
  echo "    --copy 在自己建的副本里打补丁与编、全绿与测试红收尾都删掉副本与编译目录、MERGE_AND_VERIFY_KEEP_COPY=1 留着并打出路径"
  return 0
}

if [[ "${1:-}" == "--selftest" ]]; then selftest; exit $?; fi
MEMORY="${REPLAY_MEMORY_CAP:-8G}"; THREADS="$(nproc)"; skip_apply=0; copy_mode=0; dirs=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --memory) MEMORY="${2:?}"; shift 2 ;;
    --threads) THREADS="${2:?}"; shift 2 ;;
    --skip-apply) skip_apply=1; shift ;;
    --copy) copy_mode=1; shift ;;
    --*) die "认不出参数 $1" "只认 --memory <上限>、--threads <N>、--skip-apply、--copy" ;;
    *) dirs+=("$1"); shift ;;
  esac
done
[[ ${#dirs[@]} -gt 0 || $skip_apply -eq 1 ]] || die "没有给补丁目录" "merge-and-verify.sh [--memory 16G] [--threads N] <补丁目录> …；只验不打用 --skip-apply"
if [[ -n "${MERGE_AND_VERIFY_LOG_DIRECTORY:-}" ]]; then logs="$MERGE_AND_VERIFY_LOG_DIRECTORY"
elif work_root="$(python3 "$root/.claude/hooks/lib_work_root.py")"; then logs="$work_root/merge-and-verify/$(TZ=Asia/Tokyo date +%F)-$$"
else die "取不到工作根（.claude/hooks/lib_work_root.py 跑不起来）" "恢复那份库，或设 MERGE_AND_VERIFY_LOG_DIRECTORY 指到日志目录"; fi
tree="${MERGE_AND_VERIFY_TREE:-$root}"
# --copy（文件头）：副本与编译目录是这里建的才登记进 COPY_TREE / COPY_TARGET，收尾的 EXIT trap 只删登记了的
if ((copy_mode)); then
  [[ ! -e "$logs/repo" ]] || die "副本位置 $logs/repo 已经有东西，不往里铺、也不删它" "换一个日志目录（MERGE_AND_VERIFY_LOG_DIRECTORY 不设就是新的 <日期>-<进程号>），或先看清那里是什么"
  [[ -n "${CARGO_TARGET_DIR:-}" || ! -e "$logs/target" ]] || die "编译目录位置 $logs/target 已经有东西，不往里编、也不删它" "换一个日志目录，或设 CARGO_TARGET_DIR 指到要复用的编译目录（那一份收尾不删）"
  mkdir -p "$logs" || die "建不了日志目录 $logs" "看工作根在不在、可不可写"
  COPY_TREE="$logs/repo"
  if [[ -z "${CARGO_TARGET_DIR:-}" ]]; then COPY_TARGET="$logs/target"; CARGO_TARGET_DIR="$COPY_TARGET"; export CARGO_TARGET_DIR; fi
  trap cleanup_copy EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  echo "  建副本：$tree → $COPY_TREE（编译目录 $CARGO_TARGET_DIR$([[ -n "$COPY_TARGET" ]] && echo '，收尾删' || echo '，调用方设的，收尾不删')）"
  python3 "$STAMP" sync "$tree" "$COPY_TREE" --target-dir "$CARGO_TARGET_DIR" > "$logs/copy-sync.log" 2>&1 \
    || die "建副本没成（日志 $logs/copy-sync.log）" "照那份日志修（rsync 在不在、盘满没满）再跑；建了一半的副本收尾照删" 1
  tree="$COPY_TREE"
fi
cd "$tree" || die "进不去要编的树 $tree" "在仓里跑；MERGE_AND_VERIFY_TREE 只给自证用，平时不设"
if [[ $skip_apply -eq 1 ]]; then dirs=(); fi
verify "$logs" "${dirs[@]}"
