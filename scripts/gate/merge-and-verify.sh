#!/usr/bin/env bash
# admission: always 每一批补丁都要打一次、验一次，上一次的结论不替这一次作保
# run-condition: command git cargo
# 一批实现员交回之后的「合入后验证」：打全部补丁、编一次、跑补丁改到的包（与依赖它们的包）的 harness 档测试二进制、fmt、clippy 与 build，红的列出来。替掉派一个实现员做这件活。
#
#   merge-and-verify.sh [--memory <上限>] [--threads <N>] [--skip-apply] [--copy] [--all-packages] <补丁目录> …
#     --threads 缺省取登记的重活默认并行度（.claude/heavy-thread-default，scripts/hosts/heavy-threads.sh），不看忙闲。
#     --all-packages 测试步照旧跑工作区成员减掉 checker 档包的全部包，不按补丁挑。
#   merge-and-verify.sh --selftest                     # 用假的 apply-writer-patch 与假 cargo 走一遍；MERGE_AND_VERIFY_BREAK=ignore-red 时必须判红
#
# --copy：不往这份脚本所在的仓里打，自己建一份副本在副本里打、编、验，收尾删掉它自己建的副本与编译目录（成功、失败、被 INT / TERM 停都删）：
#   副本 <日志目录>/repo（scripts/hosts/source-stamp.py sync 拷：不拷 .git 与任意一层的 target、target-*，拷过去的 mtime 是此刻），
#   补丁经 apply-writer-patch.py --root <副本> 打；编译目录调用方设了 CARGO_TARGET_DIR 就用它、不删（不是这里建的），没设是 <日志目录>/target、收尾删。
#   日志目录留着（日志小，汇总行指着它）。删完打一行「删了副本 … 与编译目录 …（共 …）」；删不成退 1、写明哪一份没删成。
#   要留现场：MERGE_AND_VERIFY_KEEP_COPY=1，收尾不删，打出副本与编译目录的路径和删法，看完自己删（command-safety.md「测试镜像一律放临时目录」：
#   要留着看现场由显式的开关打开、留下时打印路径）。不带 --copy 时照旧打进这份脚本所在的仓（合入主工作区就是这一种），不建副本、不删任何东西。
#
# 为什么：2026-09-20 起 8 天里「合入后验证」派了 3 个实现员、平均 14.3M 折算，是实现员里最贵的一种；它做的前三步（打补丁、编、逐个跑测试）
# 是 apply-writer-patch.py 加几条命令，只有「红了改钉值」要判断——那一步红了才派实现员。
# 做法：每个补丁目录经 scripts/gate/apply-writer-patch.py 打（退 2 就整批停）；编之前经 scripts/hosts/source-stamp.py refresh 刷一次源码戳：
# 编译目录（CARGO_TARGET_DIR，不设是树根的 target）是复用的，cargo 按 mtime 判本地包新不新鲜，被 rsync -a 铺进来的新内容带着旧 mtime 会被当成已编
# （records/2026-09-30-里程碑三开工第一轮.md「GPU 与 CPU 不一致调查员交回」那一行）；refresh 把内容或 mtime 与上一次不一样的源文件 touch 到此刻，
# 编译目录里有东西而没有戳（头一次）就全 touch、重编一遍工作区的本地包。刷不成就不编、退 1。手工往副本里拷树用 source-stamp.py sync <源树> <副本>。
# 然后经 run-with-memory-cap.sh 与 capped.sh 跑 test、fmt、clippy、build 四条 cargo，与上游 .claude/singlefs-ai-sop/scripts/check.sh 同一套判据：
#   test   `cargo test --offline --no-fail-fast -p <包>…`：包是工作区成员（树根 Cargo.toml 的 [workspace] members，逐个读成员 Cargo.toml 的包名）
#          减掉 checker 档包 singlefs-checker-tier（那归提交时的主 agent）；check.sh 跑的是 `cargo test --all`，差的只有这一个包。
#          成员里没有 singlefs-checker-tier（改了名、挪了）、成员写成通配、读不出包名，都判红、不编：排不掉它就宁可不跑。
#          退 0 而 test result 行里一条测试都没数到，也判红（check.sh「一条测试都没跑」那一条）。
#          测试只跑改到的包：上面那几个包里只留这一批补丁改到的包与工作区里依赖它们的包（成员 Cargo.toml 的 [dependencies]、[dev-dependencies]、
#          [build-dependencies] 与 [target.*] 下的这三张表里写了它，一层层往上传）。改到的包按给的补丁目录里 crates.patch 改过的路径算（路径往上找到的
#          第一份带 [package] 的 Cargo.toml，scripts/gate/lib_rebuilt_packages.py 的 package_of，与重编核对同一份；包里改过的不论是不是 .rs 都算），
#          --skip-apply 时照样读给的补丁目录。没挑上的包逐个报「本次未跑：补丁没改到」，成功句里逐个列名；一个包都没挑上（补丁只改了 checker 档包、
#          或没有 crates.patch）时测试这一步不跑、报「测试本次未跑」，不记红也不算测过。照旧全跑的：带 --all-packages；没给补丁目录；
#          补丁改到工作区根的 Cargo.toml、Cargo.lock 或 .cargo/；补丁改到不落在任何包里的路径（测试运行时可能读它，判不出归哪个包）。
#          harness 档按测试函数单元键入库复用（只跑键对不上的测试）归 .claude/kb/decisions/13-验证路线.md 已定项 17 的下一步，这里还是按包挑。
#   fmt    `cargo fmt --all -- --check`；clippy `cargo clippy --offline --all-targets --all-features -- -D warnings` 加编码纪律那张 lint 表；build `cargo build --offline --all-targets`。
#   lint 表只在 check.sh 一处定义：每次开跑从它的 `CODE_DISCIPLINE_LINTS=(` … `)` 那一段现读（一条一对 `-D clippy::<名>`），不在这里另抄一份；
#   同时核 check.sh 里 fmt、clippy、build、test 四条 cargo 的写法（CHECK_SH_COMMANDS）还是这里照着对齐的那一种。读不出表、表的形状不对、
#   哪一条写法变了，都判红、一个补丁都不打：上游改了判据，先回来对齐这里。
# 日志落 $MERGE_AND_VERIFY_LOG_DIRECTORY（默认工作根下的 merge-and-verify/<日期>-<进程号>，工作根只在 .claude/hooks/lib_work_root.py 里定义）。
# cargo test 那一趟内存包装退 245（显存不够，run-with-memory-cap.sh 文件头「退出码」）：这一趟的测试结果（连同 FAILED 行）不算数，记一项红、
#   报「显存不够」、不列 FAILED 也不给只重跑它们的命令（不是代码红），出路是看谁在用卡、等它跑完重跑这一条。退 250–254 照旧报包装自己的结局。
# 改过的包都重编了没有（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 1 行；判法在 scripts/gate/lib_rebuilt_packages.py，这里不另写）：
#   改过的包从给的补丁目录里的 crates.patch 算（--skip-apply 时照样读给的补丁目录，只是不打）。编之前（刷源码戳之前）把补丁改过、落在包里的文件刷到此刻，
#   编完核 test 与 build 两趟的编译输出：每个改过的包都要有「Compiling <包名> v…」行，缺一个记一项红，出路是刷源码修改时刻或换干净编译目录整趟重跑。
#   只核包里改过的 .rs；包里改过的 .rs 一个都不在编译目录的 dep-info 里的包不核（这几趟编译不读它们，例如 checker 档包 gpu、verdict-store 下才编的模块）。
#   没给补丁目录、或补丁里没有要核的包：报「重编核对本次未判」，不记红也不算核过。
# 主工作区合入锁（同一张表第 20 行；判法与锁文件在 scripts/gate/lib_workspace_lock.py 的文件头，这里不另写）：不带 --copy 时，打补丁与编之前先拿锁，
#   持有者是这个进程（$$），被别的合入占着就排队、每过一段时间报一行谁占着，持有者已死的判失效、报出来再接管；退出时（成功、红、被 INT / TERM 停）放掉。
#   锁按工作树分开：在 git worktree 加出来的树里跑时拿那棵树自己那一把，不排在主工作树的合入后面。
#   拿着锁时 export SINGLEFS_WORKSPACE_LOCK_HOLDER=$$，它起的 apply-writer-patch.py 认出是在持有者之下、不另拿。拿不成退 1、一个补丁都没打、没编。
#   带 --copy 时在自己的副本里打与编，不拿这一把。
# MERGE_AND_VERIFY_TREE（只给自证）：刷源码戳与编译在这棵树里（默认这份脚本所在的仓根）；打补丁照旧是 apply-writer-patch.py 自己的仓根。
# MERGE_AND_VERIFY_CHECK_SH（只给自证）：从哪一份 check.sh 读 lint 表与四条写法（默认这份脚本所在仓的 .claude/singlefs-ai-sop/scripts/check.sh）。
# 退出码：0 全绿；1 有红（测试、fmt、clippy、build、apply 退 1、显存不够、源码戳刷不成、check.sh 读不出、工作区成员排不掉 checker 档包、
#   改过的包没重编、主工作区合入锁没拿成）；
#   2 补丁核不过（apply 退 2）、用法错。
# 弄坏开关 MERGE_AND_VERIFY_BREAK=<项>（只给证红用；打开时自证退 1）。逐项写开关名、弄坏哪一处、打开后自证哪几项红
# （自证的三例没有标签，项名直接写那条「✗ 自检：」失败行的句首，括号里写那一行接着报的）：
#   MERGE_AND_VERIFY_BREAK=ignore-red  verify() 抓到 FAILED 行、cargo test 退非 0 都不记红，一律报测试全绿
#       「一条测试红时应当退 1、列出那条测试与只重跑它的命令」（实际退出码 0，报「✓ 测试全绿（0 个测试二进制）」）；
#       连带「--copy 测试红时也应当删掉副本与编译目录」（那一格的测试红被吞掉，退 0 不是要的 1）
#   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（lib_work_root.py 的开关：工作根退回旧根 /tmp/claude-<uid>）  没设日志目录时日志落进旧根
#       「没设日志目录时日志应当落在工作根下」（跑完删掉旧根下多出的那个 merge-and-verify/<日期>-<进程号>）
#   MERGE_AND_VERIFY_BREAK=no-source-stamp  编之前不刷源码戳
#       「往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验，应当重编改过的 walk.rs、测试全绿」（实际退出码 1，假 cargo 报 Fresh、那一条 FAILED）；
#       连带「--copy 全绿时应当在副本里打补丁、收尾删掉副本与编译目录」（那一格还核源码戳记在 <日志目录>/target 下，不刷就没有那一行）
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
#       「工作区成员里没有 singlefs-checker-tier 时应当退 1、不编」也红（写死的表不读成员，照编）；
#       按补丁挑包的那几格与别的带补丁目录的格也红（写死的四个包都不在样本工作区里，按补丁挑出来一个不剩，测试本次未跑）
#   MERGE_AND_VERIFY_BREAK=no-checker-tier-guard  工作区成员里找不到 singlefs-checker-tier 也照挑全部成员
#       「工作区成员里没有 singlefs-checker-tier 时应当退 1、不编」（实际退 0，改了名的 checker 档包被跑进来）
#   MERGE_AND_VERIFY_BREAK=zero-tests-green  cargo test 退 0 而一条测试都没数到时照报全绿
#       「cargo test 退 0 却一条测试都没跑时应当退 1」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=keep-copy  --copy 收尾不删副本与编译目录、也不打路径（改前的样子：副本留在工作根下没人删）
#       「--copy 全绿时应当在副本里打补丁、收尾删掉副本与编译目录」与「--copy 测试红时也应当删掉副本与编译目录」（实际两份都还在）；
#       连带「MERGE_AND_VERIFY_KEEP_COPY=1 时应当留着副本与编译目录、打出路径」（开关在留现场那一句之前就退出，路径不打）
#   MERGE_AND_VERIFY_BREAK=cleanup-only-on-success  --copy 只在退 0 时删副本与编译目录，红了留着
#       「--copy 测试红时也应当删掉副本与编译目录」（实际退 1、两份都还在）
#   MERGE_AND_VERIFY_BREAK=skip-rebuild-check  编完不核改过的包有没有 Compiling 行
#       「编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行：应当退 1、点名 demo」（实际退 0）
#       连带：「头一次编、补丁改过的 demo 有 Compiling 行：应当退 0、报都重编」「编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0」（不核就不报「都重编了」，判这一句的两格跟着红）
#   MERGE_AND_VERIFY_BREAK=no-touch-patched  编之前不把补丁改过的文件刷到此刻（刷源码戳照刷）
#       「编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0」（实际源码戳没变、没有 Compiling demo，退 1）
#   MERGE_AND_VERIFY_BREAK=ignore-workspace-lock  不带 --copy 时不拿主工作区合入锁
#       「两个合入同时起：后起的应当排队、报出前者的进程号、前者编完放锁后才编」（实际两趟的 cargo test 叠在一起）与
#       「锁文件里的持有者已死：应当报出来、接管、照常验完、放锁后记录清空」（实际没报接管）
#   MERGE_AND_VERIFY_BREAK=all-packages-always  测试步不按补丁挑包，照旧跑全部（改前的样子）
#       「补丁改到 alpha 与 beta 两个包：cargo test 只该挑这两个、报 demo 与 delta 本次未跑」（实际收到四个包）；
#       连带「补丁改到 demo：…」「补丁只改到 checker 档包：…」（照旧全跑）与「补丁改到工作区根的 Cargo.lock：…」（挑法那一句写的是开关，不是 Cargo.lock）
#   MERGE_AND_VERIFY_BREAK=lock-changes-narrowed  补丁改到工作区根的 Cargo.toml、Cargo.lock 或 .cargo/ 时不照旧全跑（当成没改到包）
#       「补丁改到工作区根的 Cargo.lock：cargo test 应当照旧挑全部四个包」（实际测试本次未跑）
#   MERGE_AND_VERIFY_BREAK=dependents-dropped  只挑补丁改到的包，不带上工作区里依赖它们的包
#       「补丁改到 demo：cargo test 应当挑 demo 与依赖它的 delta」（实际只收到 -p demo）
#   本库 scripts/gate/lib_rebuilt_packages.py 的 REBUILT_PACKAGES_BREAK=any-compiling-line 也让「编译目录里的产物比源码新……」那一格红（另一个包有 Compiling 行就当 demo 也重编了）；
#   scripts/gate/lib_workspace_lock.py 的 WORKSPACE_LOCK_BREAK=trust-dead-holder 让「锁文件里的持有者已死……」那一格红（排队等死掉的持有者，到 SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS 退 1）；
#   它的 WORKSPACE_LOCK_BREAK=no-wait 让「两个合入同时起……」与「锁文件里的持有者已死……」两格红（活着的持有者也照拿、不排队；死的不报接管）；
#   它的 WORKSPACE_LOCK_BREAK=lock-in-common-dir 让「主工作树的锁被占着时在 git worktree 加出来的树里验……」那一格红（锁文件照改前放 git common-dir，排到点退 1）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
source "$here/../hosts/heavy-threads.sh"
APPLY="${MERGE_AND_VERIFY_APPLY:-$here/apply-writer-patch.py}"
WRAPPER="${MERGE_AND_VERIFY_WRAPPER:-$here/../hosts/run-with-memory-cap.sh}"
CAPPED="${MERGE_AND_VERIFY_CAPPED:-$here/../hosts/capped.sh}"
STAMP="$here/../hosts/source-stamp.py"
REBUILT="$here/lib_rebuilt_packages.py"          # 改过的包重编了没有（文件头）
WORKSPACE_LOCK="$here/lib_workspace_lock.py"     # 主工作区合入锁（文件头）
CHECK_PATCH_DIRS=()                              # 算改过的包用的补丁目录（绝对路径；--skip-apply 时也留着）
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

# 不带 --copy 时的收尾（EXIT trap）：放掉主工作区合入锁（文件头「主工作区合入锁」）；放不成只报一行，退出码照旧
release_workspace_lock() {
  local rc=$?
  trap - EXIT
  python3 "$WORKSPACE_LOCK" release --root "$tree" --holder-pid "$$" || echo "  ! 放主工作区合入锁没成（上面一行是原因）：看 $WORKSPACE_LOCK status --root $tree"
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

# select_test_packages <树>：TEST_PACKAGES 里只留这一批补丁改到的包与工作区里依赖它们的包（文件头「测试只跑改到的包」）；照旧全跑的原样留着。
# 没挑上的包名进 UNTESTED_PACKAGES，怎么挑的一句进 TEST_SELECTION_NOTE；读不了补丁或成员清单返回 1，原因进 TEST_SELECTION_PROBLEM。
select_test_packages() {
  local listing kind name candidates=() position
  UNTESTED_PACKAGES=(); TEST_SELECTION_NOTE=""; TEST_SELECTION_PROBLEM=""
  if ((ALL_PACKAGES)); then TEST_SELECTION_NOTE="带了 --all-packages，照旧全跑"; return 0; fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "all-packages-always" ]]; then TEST_SELECTION_NOTE="弄坏开关 MERGE_AND_VERIFY_BREAK=all-packages-always，照旧全跑"; return 0; fi
  if ((${#CHECK_PATCH_DIRS[@]} == 0)); then TEST_SELECTION_NOTE="没给补丁目录，算不出改到的包，照旧全跑"; return 0; fi
  for ((position = 1; position < ${#TEST_PACKAGES[@]}; position += 2)); do candidates+=("${TEST_PACKAGES[position]}"); done
  if ! listing="$(python3 - "$1" "$REBUILT" "${MERGE_AND_VERIFY_BREAK:-}" "${candidates[*]}" "${CHECK_PATCH_DIRS[@]}" <<'PY_SELECT' 2>&1
import importlib.util, os, sys, tomllib
tree, library, broken, candidate_text, *patch_directories = sys.argv[1:]
candidates = candidate_text.split()
specification = importlib.util.spec_from_file_location("lib_rebuilt_packages_for_merge_and_verify", library)
rebuilt = importlib.util.module_from_spec(specification)
specification.loader.exec_module(rebuilt)
try:
    paths = rebuilt.collect_paths(patch_directories, [])
except OSError as error:
    print(f"读不了补丁：{error}")
    sys.exit(1)
WORKSPACE_WIDE = ("Cargo.toml", "Cargo.lock")
changed, outside = [], []
for path in paths:
    if path in WORKSPACE_WIDE or path == ".cargo" or path.startswith(".cargo/"):
        if broken == "lock-changes-narrowed":
            continue
        print(f"ALL\t补丁改到工作区根的 {path}")
        sys.exit(0)
    name = rebuilt.package_of(tree, path)
    if name is None:
        outside.append(path)
    elif name not in changed:
        changed.append(name)
if outside:
    print(f"ALL\t补丁改到不落在任何包里的 {'、'.join(outside[:3])}{' 等 ' + str(len(outside)) + ' 处' if len(outside) > 3 else ''}（测试运行时可能读它，判不出归哪个包）")
    sys.exit(0)
def load(path):
    with open(path, "rb") as handle:
        return tomllib.load(handle)
try:
    members = load(os.path.join(tree, "Cargo.toml")).get("workspace", {}).get("members") or []
    manifests = {}
    for member in members:
        manifest = load(os.path.join(tree, member, "Cargo.toml"))
        manifests[manifest["package"]["name"]] = manifest
except (OSError, KeyError, TypeError, tomllib.TOMLDecodeError) as error:
    print(f"读不了工作区成员的 Cargo.toml：{error!r}")
    sys.exit(1)
def dependency_names(manifest):
    """一个成员在三张依赖表（连同 [target.*] 下的）里写的包名（改了名的照 package = 认）。"""
    tables = [manifest.get(key) for key in ("dependencies", "dev-dependencies", "build-dependencies")]
    for target in (manifest.get("target") or {}).values():
        if isinstance(target, dict):
            tables += [target.get(key) for key in ("dependencies", "dev-dependencies", "build-dependencies")]
    for table in tables:
        for key, value in (table or {}).items():
            yield value.get("package", key) if isinstance(value, dict) else key
depends_on = {name: set(dependency_names(manifest)) & set(manifests) for name, manifest in manifests.items()}
affected = set(changed)
dependents = []
while broken != "dependents-dropped":
    grown = sorted(name for name, needed in depends_on.items() if name not in affected and needed & affected)
    if not grown:
        break
    affected |= set(grown)
    dependents += grown
for name in changed:
    print(f"CHANGED\t{name}")
for name in dependents:
    print(f"DEPENDENT\t{name}")
for name in candidates:
    print(f"{'RUN' if name in affected else 'SKIP'}\t{name}")
PY_SELECT
  )"; then
    TEST_SELECTION_PROBLEM="$listing"; return 1
  fi
  if grep -q $'^ALL\t' <<< "$listing"; then
    TEST_SELECTION_NOTE="$(sed -n 's/^ALL\t//p' <<< "$listing" | head -n 1)，照旧全跑"; return 0
  fi
  local changed_text="" dependent_text=""
  TEST_PACKAGES=()
  while IFS=$'\t' read -r kind name; do
    case "$kind" in
      RUN) TEST_PACKAGES+=(-p "$name") ;;
      SKIP) UNTESTED_PACKAGES+=("$name") ;;
      CHANGED) changed_text+="$name、" ;;
      DEPENDENT) dependent_text+="$name、" ;;
    esac
  done <<< "$listing"
  TEST_SELECTION_NOTE="补丁改到的包：${changed_text:-一个都没有}"
  TEST_SELECTION_NOTE="${TEST_SELECTION_NOTE%、}"
  [[ -z "$dependent_text" ]] || TEST_SELECTION_NOTE+="；另带上依赖它们的：${dependent_text%、}"
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
  local dir rc applied=0 warned=0 failed_tests reds=0 test_count untested_text
  local -a clippy_lints=() patch_arguments=() compile_logs=()
  mkdir -p "$logs"
  rm -f -- "$logs/test.log" "$logs/build.log"   # 重编核对只认这一趟的编译输出（日志目录可能是调用方给的旧目录）
  for dir in ${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"}; do patch_arguments+=(--patch-dir "$dir"); done
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
  # 补丁改过、落在包里的文件刷到此刻（文件头「改过的包都重编了没有」）：再验同一批补丁时源码戳没变，不刷就不会重编
  if ((${#patch_arguments[@]} > 0)); then
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "no-touch-patched" ]]; then
      echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=no-touch-patched：编之前不把补丁改过的文件刷到此刻"
    elif python3 "$REBUILT" touch --tree "$tree" "${patch_arguments[@]}" > "$logs/touch-patched.log" 2>&1; then
      echo "$(tail -1 "$logs/touch-patched.log")"
    else
      die "编之前没把补丁改过的文件刷到此刻，没编（日志 $logs/touch-patched.log）" "照那份日志修好再跑（补丁目录在不在、树可不可写）" 1
    fi
  fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "no-source-stamp" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=no-source-stamp：编之前不刷源码戳"
  elif python3 "$STAMP" refresh "$tree" > "$logs/source-stamp.log" 2>&1; then
    echo "  源码戳：$(grep '^SOURCE_STAMP ' "$logs/source-stamp.log" | tail -1)"
  else
    die "编之前没刷成源码戳，没编（日志 $logs/source-stamp.log）" "照那份日志修好再跑；刷不成就别复用这个编译目录（设 CARGO_TARGET_DIR 换一个新的）" 1
  fi
  test_packages "$tree" || die "挑不出要测的包：$TEST_PACKAGES_PROBLEM；没编" \
    "工作区成员要逐个写出目录、各有 [package] name，checker 档包 $CHECKER_TIER_PACKAGE 要在成员里（这里靠它的名字把它排掉）；改了名就同步改这份脚本的 CHECKER_TIER_PACKAGE" 1
  select_test_packages "$tree" || die "算不出这一批补丁改到哪几个包：$TEST_SELECTION_PROBLEM；没编" \
    "看补丁目录在不在、crates.patch 与工作区成员的 Cargo.toml 读不读得了；要照旧全跑就加 --all-packages" 1
  untested_text=""
  if ((${#UNTESTED_PACKAGES[@]})); then untested_text="$(printf '%s、' "${UNTESTED_PACKAGES[@]}")"; untested_text="；本次未跑：补丁没改到 ${#UNTESTED_PACKAGES[@]} 个包：${untested_text%、}"; fi
  rc=0; test_count=0; failed_tests=""
  if ((${#TEST_PACKAGES[@]})); then
    echo "  编译并跑 ${TEST_PACKAGES[*]} 的全部测试二进制（工作区成员减掉 $CHECKER_TIER_PACKAGE，${TEST_SELECTION_NOTE}${untested_text}；内存上限 $MEMORY、线程上限 $THREADS）…"
    run_capped "$logs/test.log" cargo test --offline --no-fail-fast "${TEST_PACKAGES[@]}"; rc=$?
    test_count="$(awk 'match($0, /^test result: [A-Za-z]+\. [0-9]+ passed; [0-9]+ failed/) { split($0, f, " "); passed += f[4]; failed += f[6] } END { print passed + failed + 0 }' "$logs/test.log")"
    failed_tests="$(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' "$logs/test.log")"
  fi
  if ((${#TEST_PACKAGES[@]} == 0)); then
    echo "  - 测试本次未跑：这一批补丁一个要测的包都没改到（${TEST_SELECTION_NOTE}；checker 档包 $CHECKER_TIER_PACKAGE 归提交时）${untested_text}"
  elif [[ $rc -eq $GPU_MEMORY_HIT_EXIT && "${MERGE_AND_VERIFY_BREAK:-}" != "gpu-memory-as-test-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 那一趟显存不够（内存包装退 $GPU_MEMORY_HIT_EXIT：起止之间内核报了 NV_ERR_NO_MEMORY），这一趟的测试结果不算数（日志 $logs/test.log）"
    echo "  → 怎么办：nvidia-smi 看谁在用卡，等它跑完再重跑这一条 merge-and-verify.sh；不是代码红，别照日志里的 FAILED 改钉值"
  elif [[ -n "$failed_tests" && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ 红的测试 $(printf '%s\n' "$failed_tests" | wc -l) 条："
    printf '%s\n' "$failed_tests" | sed 's/^/      /'   # gate-lint:detail
    echo "  → 怎么办：只重跑红的这几条：python3 scripts/gate/rerun-failed-tests.py $logs/test.log -p <包>；照实现员报告里的算法改钉值，改完再只跑这几条"
  elif [[ $rc -ne 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 $rc 而一条 FAILED 都没抓到（编不过、或包装退 250–254）：$(tail -3 "$logs/test.log" | tr '\n' ' ')"
    echo "  → 怎么办：看 $logs/test.log；退 250–254 是内存包装自己的结局，按 run-with-memory-cap.sh 文件头处理"
  elif [[ "$test_count" == 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "zero-tests-green" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 0，却一条测试都没跑（test result 行里 passed 与 failed 加起来是 0；日志 $logs/test.log）"
    echo "  → 怎么办：看那份日志里编了哪些包、有没有 test result 行；一条都没跑的「全绿」与跑过了长得一样（check.sh 同一条）"
  else
    echo "  ✓ 测试全绿（$(grep -c '^test result: ok' "$logs/test.log") 个测试二进制，共 $test_count 条测试；跑了 $((${#TEST_PACKAGES[@]} / 2)) 个包${untested_text}）"
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
  # 改过的包这一趟都重编了没有（文件头）：test 与 build 两趟的编译输出合起来核
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-rebuild-check" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-rebuild-check：不核改过的包重编了没有"
  elif ((${#patch_arguments[@]} == 0)); then
    echo "  - 重编核对本次未判：没给补丁目录，算不出改过的包"
  elif [[ ! -f "$logs/test.log" && ! -f "$logs/build.log" ]]; then
    echo "  - 重编核对本次未判：这一趟测试与 build 都没编（测试本次未跑、build 没跑），没有编译输出可核"
  else
    compile_logs=()
    [[ ! -f "$logs/test.log" ]] || compile_logs+=(--log "$logs/test.log")   # 测试本次未跑时没有这一份
    [[ ! -f "$logs/build.log" ]] || compile_logs+=(--log "$logs/build.log")
    python3 "$REBUILT" check --tree "$tree" --target-dir "${CARGO_TARGET_DIR:-$tree/target}" "${compile_logs[@]}" "${patch_arguments[@]}" > "$logs/rebuilt.log" 2>&1; rc=$?
    sed 's/^/  /' "$logs/rebuilt.log"
    case "$rc" in
      0|77) ;;
      1) reds=$((reds+1)) ;;
      *) reds=$((reds+1)); echo "  ✗ 重编核对判不了（lib_rebuilt_packages.py 退 $rc，日志 $logs/rebuilt.log）"
         echo "  → 怎么办：照上面那一行修（编译日志在不在、补丁目录读不读得了）再整趟重跑；判不了不算绿" ;;
    esac
  fi
  echo "  汇总：打上 ${applied} 个补丁目录（其中 code-source-discipline 红 $warned 个），验证红 $reds 项，日志在 $logs"
  [[ $reds -eq 0 && $warned -eq 0 ]]
}

selftest() {
  local scratch fake_cargo_dir failures=0 checked=0 out rc expected_lints expected_lint_count check_case
  unset MERGE_AND_VERIFY_KEEP_COPY   # 留现场那一格自己设；调用方设了不能带进别的格
  scratch="$(mktemp -d)"; fake_cargo_dir="$scratch/bin"; mkdir -p "$fake_cargo_dir" "$scratch/patch-a" "$scratch/patch-b"
  # patch-a 改的是 demo 里的一份数据文件（不是 .rs：重编核对不核它，测试照挑 demo）
  printf -- '--- a/crates/demo/src/table.txt\n+++ b/crates/demo/src/table.txt\n@@ -1 +1 @@\n-1\n+2\n' > "$scratch/patch-a/crates.patch"
  # 主工作区合入锁：每一格的锁文件都是临时目录里这一份（不碰仓里 git 目录那一把；按工作树分开那一格另在临时仓里用默认位置）；排队时每 0.3 秒报一次、每 0.05 秒看一次；调用方的持有者变量不带进来
  # 排队最多等 30 秒：锁永远拿不到（弄坏开关、或哪一格没放锁）时到点退 1、那一格红，不挂住
  export SINGLEFS_WORKSPACE_LOCK_FILE="$scratch/main-workspace.lock" SINGLEFS_WORKSPACE_LOCK_REPORT_SECONDS=0.3 SINGLEFS_WORKSPACE_LOCK_POLL_SECONDS=0.05 SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS=30
  unset SINGLEFS_WORKSPACE_LOCK_HOLDER
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
  # 测试只跑改到的包（文件头）：样本工作区五个成员 demo、alpha、beta、delta（依赖 demo）与 checker 档包；补丁改的都是包里的数据文件（重编核对不核）
  local narrow_tree="$scratch/narrow-tree" narrow_member
  mkdir -p "$narrow_tree/crates/singlefs-checker-tier"
  printf '[workspace]\nmembers = ["crates/demo", "crates/alpha", "crates/beta", "crates/delta", "crates/singlefs-checker-tier"]\n' > "$narrow_tree/Cargo.toml"
  for narrow_member in demo alpha beta delta; do
    mkdir -p "$narrow_tree/crates/$narrow_member/src" "$narrow_tree/crates/$narrow_member/tests"; printf 'pub fn f() {}\n' > "$narrow_tree/crates/$narrow_member/src/lib.rs"
    printf '[package]\nname = "%s"\n' "$narrow_member" > "$narrow_tree/crates/$narrow_member/Cargo.toml"
  done
  printf '\n[dev-dependencies]\ndemo = { path = "../demo" }\n' >> "$narrow_tree/crates/delta/Cargo.toml"
  printf '[package]\nname = "singlefs-checker-tier"\n' > "$narrow_tree/crates/singlefs-checker-tier/Cargo.toml"
  narrow_patch() {   # $1 补丁目录，其余是改到的路径
    mkdir -p "$1"; local narrow_path; : > "$1/crates.patch"
    for narrow_path in "${@:2}"; do printf -- '--- a/%s\n+++ b/%s\n@@ -1 +1 @@\n-1\n+2\n' "$narrow_path" "$narrow_path" >> "$1/crates.patch"; done
  }
  narrow_patch "$scratch/narrow-two" crates/alpha/src/table.txt crates/beta/tests/data.txt
  narrow_patch "$scratch/narrow-demo" crates/demo/src/table.txt
  narrow_patch "$scratch/narrow-lock" Cargo.lock
  narrow_case() {   # $1 标签，其余是参数
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-narrow-$1.txt" PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
      MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-narrow-$1" MERGE_AND_VERIFY_TREE="$narrow_tree" env -u CARGO_TARGET_DIR bash "$0" "${@:2}" > "$out" 2>&1; rc=$?
  }
  out="$scratch/out-narrow-two.txt"; narrow_case two --skip-apply "$scratch/narrow-two"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast -p alpha -p beta" "$scratch/calls-narrow-two.txt" \
     || ! grep -qF '本次未跑：补丁没改到 2 个包：demo、delta' "$out" || ! grep -q '✓ 测试全绿.*跑了 2 个包；本次未跑：补丁没改到 2 个包：demo、delta' "$out"; then
    echo "  ✗ 自检：补丁改到 alpha 与 beta 两个包：cargo test 只该挑这两个、报 demo 与 delta 本次未跑，实际退出码 $rc，假 cargo 收到 $(grep '^test' "$scratch/calls-narrow-two.txt" 2>/dev/null)：$(grep -E '✓ 测试|本次未跑|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-narrow-demo.txt"; narrow_case demo --skip-apply "$scratch/narrow-demo"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast -p demo -p delta" "$scratch/calls-narrow-demo.txt" || ! grep -qF '另带上依赖它们的：delta' "$out"; then
    echo "  ✗ 自检：补丁改到 demo：cargo test 应当挑 demo 与依赖它的 delta，实际退出码 $rc，假 cargo 收到 $(grep '^test' "$scratch/calls-narrow-demo.txt" 2>/dev/null)：$(grep -E '编译并跑|✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-narrow-lock.txt"; narrow_case lock --skip-apply "$scratch/narrow-lock"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast -p demo -p alpha -p beta -p delta" "$scratch/calls-narrow-lock.txt" || ! grep -qF '补丁改到工作区根的 Cargo.lock，照旧全跑' "$out"; then
    echo "  ✗ 自检：补丁改到工作区根的 Cargo.lock：cargo test 应当照旧挑全部四个包，实际退出码 $rc，假 cargo 收到 $(grep '^test' "$scratch/calls-narrow-lock.txt" 2>/dev/null)：$(grep -E '编译并跑|本次未跑|✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-narrow-all.txt"; narrow_case all --skip-apply --all-packages "$scratch/narrow-two"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast -p demo -p alpha -p beta -p delta" "$scratch/calls-narrow-all.txt" || ! grep -qF '带了 --all-packages，照旧全跑' "$out"; then
    echo "  ✗ 自检：带 --all-packages：cargo test 应当照旧挑全部四个包，实际退出码 $rc，假 cargo 收到 $(grep '^test' "$scratch/calls-narrow-all.txt" 2>/dev/null)"; failures=1; fi   # gate-lint:detail
  narrow_patch "$scratch/narrow-checker" crates/singlefs-checker-tier/src/table.txt
  mkdir -p "$narrow_tree/crates/singlefs-checker-tier/src"
  out="$scratch/out-narrow-checker.txt"; narrow_case checker --skip-apply "$scratch/narrow-checker"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || grep -q '^test' "$scratch/calls-narrow-checker.txt" 2>/dev/null || ! grep -qF '测试本次未跑：这一批补丁一个要测的包都没改到' "$out"; then
    echo "  ✗ 自检：补丁只改到 checker 档包：测试这一步应当不跑、报测试本次未跑、退 0，实际退出码 $rc，假 cargo 收到 $(grep '^test' "$scratch/calls-narrow-checker.txt" 2>/dev/null)：$(grep -E '本次未跑|✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
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
  # 改过的包重编了没有（文件头）：假 cargo 按包、按 mtime 判新不新鲜，重编时照真 cargo 打「Compiling <包名> v…」（lib_rebuilt_packages.py --write-selftest-cargo）；
  # 补丁目录 rebuilt-patch 的 crates.patch 改的是 crates/demo/src/walk.rs（树里已经是改过的样子，--skip-apply 只验不打）
  local rebuilt_tree="$scratch/rebuilt-tree" rebuilt_bin="$scratch/rebuilt-bin" rebuilt_patch="$scratch/rebuilt-patch"
  mkdir -p "$rebuilt_tree/crates/demo/src" "$rebuilt_bin" "$rebuilt_patch"
  write_workspace "$rebuilt_tree" singlefs-checker-tier
  printf 'pub mod walk;\n' > "$rebuilt_tree/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 22 }\n' > "$rebuilt_tree/crates/demo/src/walk.rs"
  printf 'diff --git a/crates/demo/src/walk.rs b/crates/demo/src/walk.rs\n--- a/crates/demo/src/walk.rs\n+++ b/crates/demo/src/walk.rs\n@@ -1 +1 @@\n-pub fn walk() -> u32 { 1 }\n+pub fn walk() -> u32 { 22 }\n' \
    > "$rebuilt_patch/crates.patch"
  python3 "$REBUILT" --write-selftest-cargo "$rebuilt_bin/cargo"
  rebuilt_case() { PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$rebuilt_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply "$rebuilt_patch" > "$out" 2>&1; rc=$?; }
  # 绿样本：头一次编（编译目录是空的），demo 有 Compiling 行
  out="$scratch/out-rebuilt-first.txt"; rebuilt_case rebuilt-first; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '改过的包这一趟都重编了（核了 1 个包：demo）' "$out"; then
    echo "  ✗ 自检：头一次编、补丁改过的 demo 有 Compiling 行：应当退 0、报都重编了，实际退出码 $rc：$(grep -E '✗|✓|重编' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 编过之后再验同一批补丁：源码戳没变（refresh 一个都不 touch），要靠编之前把补丁改过的文件刷到此刻，demo 才重编
  out="$scratch/out-rebuilt-rerun.txt"; rebuilt_case rebuilt-rerun; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '改过的包这一趟都重编了（核了 1 个包：demo）' "$out"; then
    echo "  ✗ 自检：编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0，实际退出码 $rc：$(grep -E '✗|✓|重编|刷到此刻' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 红样本：编译目录里 demo 的产物比源码新（另一份副本共用这个编译目录、后编的那种），源码的修改时刻比它旧；
  # checker 档那个包的文件刷到此刻、照常重编（有一行别的 Compiling），demo 没有 Compiling 行，测试照样全绿
  touch -d '+1 day' "$rebuilt_tree/target/debug/.fake/demo.d"
  touch "$rebuilt_tree/crates/singlefs-checker-tier/Cargo.toml"
  out="$scratch/out-rebuilt-stale.txt"; rebuilt_case rebuilt-stale; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -q '改过的包这一趟没重编：demo' "$out" || ! grep -q 'Compiling singlefs-checker-tier' "$scratch/logs-rebuilt-stale/test.log" 2>/dev/null; then
    echo "  ✗ 自检：编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行：应当退 1、点名 demo，实际退出码 $rc：$(grep -E '✗|✓|重编' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 主工作区合入锁（文件头）：两个合入同时起。A 的 cargo test 睡 1.5 秒；锁记录里出现 A 的进程号之后起 B；
  # B 要报「在排队」与 A 的进程号，B 的 cargo test 要在 A 的结束之后才开始
  local lock_tree="$scratch/lock-tree" timeline="$scratch/lock-timeline.txt" a_pid b_pid a_rc b_rc attempt holder_pid=""
  mkdir -p "$lock_tree/crates/demo/src"; write_workspace "$lock_tree" singlefs-checker-tier; printf 'pub fn f() {}\n' > "$lock_tree/crates/demo/src/lib.rs"
  PATH="$rebuilt_bin:$PATH" FAKE_CARGO_TIMELINE="$timeline" FAKE_CARGO_TAG=A FAKE_CARGO_SLEEP=1.5 MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-lock-a" MERGE_AND_VERIFY_TREE="$lock_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$scratch/out-lock-a.txt" 2>&1 &
  a_pid=$!
  for attempt in $(seq 1 200); do
    holder_pid="$(sed -n 's/^pid=//p' "$SINGLEFS_WORKSPACE_LOCK_FILE" 2>/dev/null)"
    [[ -z "$holder_pid" ]] || break
    kill -0 "$a_pid" 2>/dev/null || break   # A 已经跑完（不认锁时它根本不记），不再等
    sleep 0.05
  done
  PATH="$rebuilt_bin:$PATH" FAKE_CARGO_TIMELINE="$timeline" FAKE_CARGO_TAG=B MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-lock-b" MERGE_AND_VERIFY_TREE="$lock_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$scratch/out-lock-b.txt" 2>&1 &
  b_pid=$!
  if wait "$a_pid"; then a_rc=0; else a_rc=$?; fi
  if wait "$b_pid"; then b_rc=0; else b_rc=$?; fi
  local a_end b_start
  a_end="$(awk '$1 == "end" && $2 == "A" { print $3 }' "$timeline" 2>/dev/null)"
  b_start="$(awk '$1 == "start" && $2 == "B" { print $3 }' "$timeline" 2>/dev/null)"
  checked=$((checked+1))
  if [[ $a_rc -ne 0 || $b_rc -ne 0 || -z "$a_end" || -z "$b_start" || "$holder_pid" != "$a_pid" ]] || ((b_start <= a_end)) \
     || ! grep -q "在排队.*进程 $a_pid" "$scratch/out-lock-b.txt"; then
    lock_order_detail="A 的 cargo test 结束于 ${a_end:-没有}、B 的开始于 ${b_start:-没有}，B 报的：$(grep -E '排队|拿到|✗' "$scratch/out-lock-b.txt" | head -3 | tr '\n' ' ')"
    echo "  ✗ 自检：两个合入同时起：后起的应当排队、报出前者的进程号、前者编完放锁后才编，实际 A 退 $a_rc（进程 $a_pid，锁记录里 ${holder_pid:-没有}）、B 退 $b_rc， $lock_order_detail"; failures=1; fi   # gate-lint:detail
  # 锁文件里的持有者已死（记一个已经退出的进程）：报出来、接管、照常验完，放锁后记录清空
  local dead_pid
  bash -c 'exit 0' & dead_pid=$!
  wait "$dead_pid"
  printf 'pid=%s\nstart=1\nsession=死掉的会话\ncommand=merge-and-verify.sh 假的\nsince=%s\n' "$dead_pid" "$(date +%s)" > "$SINGLEFS_WORKSPACE_LOCK_FILE"
  out="$scratch/out-lock-dead.txt"; checked=$((checked+1))
  FAKE_CARGO_MODE=green SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS=5 run_case lock-dead --skip-apply
  if [[ $rc -ne 0 ]] || ! grep -q '已死.*死掉的会话.*接管' "$out" || grep -q '^pid=' "$SINGLEFS_WORKSPACE_LOCK_FILE"; then
    echo "  ✗ 自检：锁文件里的持有者已死：应当报出来、接管、照常验完、放锁后记录清空，实际退出码 $rc：$(grep -E '锁|✗' "$out" | head -3 | tr '\n' ' ')；锁文件 $(tr '\n' ' ' < "$SINGLEFS_WORKSPACE_LOCK_FILE")"; failures=1; fi   # gate-lint:detail
  # 每棵工作树各一把（lib_workspace_lock.py 文件头「锁文件」）：临时仓的主工作树的锁被一个活着的假合入占着（它拿到锁之后 exec sleep，进程号不变），
  # 在它 git worktree 加出来的树里验不排队、照常验完；这一格不设 SINGLEFS_WORKSPACE_LOCK_FILE，锁文件在各自的 git 目录里。放弃时限 3 秒
  # （弄坏开关 WORKSPACE_LOCK_BREAK=lock-in-common-dir 下排到点退 1、这一格红，不挂住）；用完只停自己起的那一个进程号
  local wt_main="$scratch/wt-main" wt_linked="$scratch/wt-linked" wt_holder wt_attempt
  mkdir -p "$wt_main/crates/demo/src"; write_workspace "$wt_main" singlefs-checker-tier; printf 'pub fn f() {}\n' > "$wt_main/crates/demo/src/lib.rs"
  { git init -q "$wt_main" && git -C "$wt_main" add -A && git -C "$wt_main" -c user.name=s -c user.email=s@s commit -qm base && git -C "$wt_main" worktree add -q "$wt_linked"; } > /dev/null 2>&1
  env -u SINGLEFS_WORKSPACE_LOCK_FILE bash -c 'python3 "$0" acquire --root "$1" --holder-pid $$ --command "假合入 主工作树" > /dev/null || exit 1; : > "$2"; exec sleep 30' \
    "$WORKSPACE_LOCK" "$wt_main" "$scratch/wt-held" &
  wt_holder=$!
  for wt_attempt in $(seq 1 200); do [[ -e "$scratch/wt-held" ]] && break; kill -0 "$wt_holder" 2>/dev/null || break; sleep 0.05; done
  out="$scratch/out-lock-per-tree.txt"; checked=$((checked+1))
  PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-lock-per-tree" \
    MERGE_AND_VERIFY_TREE="$wt_linked" SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS=3 env -u CARGO_TARGET_DIR -u SINGLEFS_WORKSPACE_LOCK_FILE bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  kill "$wt_holder" 2>/dev/null
  wait "$wt_holder" 2>/dev/null
  if [[ ! -e "$scratch/wt-held" || $rc -ne 0 ]] || grep -q '在排队' "$out"; then
    echo "  ✗ 自检：主工作树的锁被占着时在 git worktree 加出来的树里验：应当不排队、照常验完退 0，实际$([[ -e "$scratch/wt-held" ]] || echo '主工作树的假合入没拿到锁、')退出码 $rc：$(grep -E '锁|排队|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -rf "${scratch:?}"
  if [[ $failures -ne 0 ]]; then
    echo "    → 怎么办：看 verify() 抓 FAILED 行与退出码、编之前刷源码戳的那几处；MERGE_AND_VERIFY_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 种）：测试红时退 1 并列出红的与重跑命令、补丁核不过整批停退 2、全绿退 0、没设日志目录时日志落在工作根下、"
  echo "    往编过的副本里 rsync -a 铺旧 mtime 的新内容之后照样重编、包装退 245 报显存不够不报测试红、"
  echo "    clippy 带上 check.sh 的 $expected_lint_count 条编码纪律 lint、跑 build、cargo test 挑工作区成员减掉 checker 档包、"
  echo "    check.sh 读不出 lint 表或 clippy 那一行变了退 1 不编、成员里没有 checker 档包退 1 不编、一条测试都没跑退 1、"
  echo "    --copy 在自己建的副本里打补丁与编、全绿与测试红收尾都删掉副本与编译目录、MERGE_AND_VERIFY_KEEP_COPY=1 留着并打出路径、"
  echo "    补丁改过的包头一次编与再验同一批补丁都有 Compiling 行、编译目录里的产物比源码新时点名没重编的包退 1、"
  echo "    两个合入同时起后起的排队并报出前者、前者放锁后才编、锁文件里的持有者已死时报出来并接管、主工作树的锁被占着时 git worktree 加出来的树里照常验、"
  echo "    测试只挑补丁改到的包（改两个包只跑两个、没改到的逐个报本次未跑）并带上依赖它们的包、改到 Cargo.lock 与带 --all-packages 照旧全跑、只改到 checker 档包时测试本次未跑"
  return 0
}

if [[ "${1:-}" == "--selftest" ]]; then selftest; exit $?; fi
MEMORY="${REPLAY_MEMORY_CAP:-8G}"; THREADS="$(heavy_thread_default)"; skip_apply=0; copy_mode=0; ALL_PACKAGES=0; dirs=(); invocation="merge-and-verify.sh $*"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --memory) MEMORY="${2:?}"; shift 2 ;;
    --threads) THREADS="${2:?}"; shift 2 ;;
    --skip-apply) skip_apply=1; shift ;;
    --copy) copy_mode=1; shift ;;
    --all-packages) ALL_PACKAGES=1; shift ;;
    --*) die "认不出参数 $1" "只认 --memory <上限>、--threads <N>、--skip-apply、--copy、--all-packages" ;;
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
# 主工作区合入锁（文件头）：不带 --copy 时拿，EXIT trap 放
if ((copy_mode == 0)); then
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "ignore-workspace-lock" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=ignore-workspace-lock：不拿主工作区合入锁"
  else
    python3 "$WORKSPACE_LOCK" acquire --root "$tree" --holder-pid "$$" --command "$invocation" \
      || die "主工作区合入锁没拿成（上面一行是原因），一个补丁都没打、没编" "照上面那一行的出路办；要在别处验就加 --copy 在自己的副本里打与编" 1
    trap release_workspace_lock EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    export SINGLEFS_WORKSPACE_LOCK_HOLDER="$$"
  fi
fi
# 算改过的包用的补丁目录（文件头「改过的包都重编了没有」）：进树之前转成绝对路径，--skip-apply 时也留着
for dir in ${dirs[@]+"${dirs[@]}"}; do
  if [[ -d "$dir" ]]; then CHECK_PATCH_DIRS+=("$(cd "$dir" && pwd)"); else CHECK_PATCH_DIRS+=("$dir"); fi
done
cd "$tree" || die "进不去要编的树 $tree" "在仓里跑；MERGE_AND_VERIFY_TREE 只给自证用，平时不设"
if [[ $skip_apply -eq 1 ]]; then dirs=(); fi
verify "$logs" "${dirs[@]}"
