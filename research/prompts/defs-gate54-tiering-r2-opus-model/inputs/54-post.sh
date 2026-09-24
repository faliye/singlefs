#!/usr/bin/env bash
# gate-stage: 层 0 崩溃点重放（整轮门禁跑快档：两条流不标 ignored 的用例，再核对层 0 全量的全绿标记与这批输入的内容哈希相等；全量由主 agent 暂存之后在 HEAD + 暂存区的 worktree 里跑 --full，全绿标记按输入哈希分格：两条流的全部崩溃状态在 release 下逐个跑恢复，第一个事务与 E142 产物逐字比对，里程碑「第二个事务」固定脚本到 E 与用例的闭式比对；只做过 mkfs 的池可写挂载再发第一个文件版本那条流与第一个事务逐项相同，由 cargo test 里的快用例钉住，不另枚举）
# gate-covers: 崩溃点重放
#
# 分两档（用户 2026-09-19 定，原话「每次主 agent 执行完任务后统一执行」，records/2026-09-19-里程碑二遗留收拢.md「五之二」第 8 问）：
#
#   bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>    主 agent 暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑一次
#     两条流的全量枚举（「全量」「多线程」两段说的就是它），不问复用、不问改动范围，照跑。worktree 的建法与 `gate.sh --staged` 相同，命令在快档的出路句里。
#     开跑与跑完各算一次输入的内容哈希，对不上（跑的过程中输入被改了）判红。全绿标记按输入哈希分格：
#     `$(git rev-parse --git-common-dir)/singlefs-layer0-full-green.<输入哈希>`，不进工作树；放 common-dir，各 worktree 读写的是同一组。
#     开跑一格都不删：同一批输入的结果是确定的，前一趟写下的那一格在这一趟跑的过程中照样算数。这一趟没写成标记就退出（判红、跑的过程中输入变了、
#     被 TERM / INT / HUP 打断）时，退出前删这批输入那一格；被 SIGKILL 杀掉来不及删，前一趟那一格留着。别的格不动；全绿才写这一格。
#     标记里有输入哈希、逐文件的「sha256  路径」、开跑与跑完的 UTC 时刻、工作线程数、两条流的计数行与 CHECKER 行原样。
#   bash .claude/gate.d/54-layer0-replay.sh [项目根]           整轮门禁的默认（gate.sh 只传项目根）
#     快档：两条流的测试二进制在 release 下只跑不标 ignored 的用例，一条都没通过判红；再按这批输入的哈希找那一格：
#     有、里面记的哈希相同、计数行恰好三行、LAYER0 与 LAYER0B 两行都是 exhaustive=true，才判绿，成功句报快档计数、原样带出那一格的全量计数行与时刻。
#     没有那一格判红（列出 common-dir 里最近写的一格与这一次不同的文件；不带哈希的旧名字 singlefs-layer0-full-green 不再认），
#     哈希不同、计数行不对、缺 exhaustive=true 都判红。快档本身红照旧红。
#
# 输入：`.claude/gate.d/stage-inputs.tsv` 里登记给本阶段的路径（唯一登记位，复用判定读的也是它）。文件集是 git 眼里这些路径下
# 磁盘上真有的文件（已跟踪的加没被忽略的未跟踪的）；逐个按内容算 sha256，「sha256  路径」按路径排序，整张再算一次 sha256。
# 按内容算、不按 git 对象算：工作区跑的全量与 `--staged` 临时 worktree 里的同一份内容算出同一个数。
# 主工作区里跑的 --full 读的是工作区（连同别的会话没暂存的改动、工作区那一份 54 号），罩不到这一批暂存内容；所以 --full 在 HEAD + 暂存区的 worktree 里跑。
#
# 全量（里程碑「第一个事务」步 7）：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，每个状态跑三件事：步 6 的恢复与 oracle、
# 池级 checker（23 条不变量）、记录核对器（根在案而记录缺席、恢复自称新态而单元缺席）；三者的计数都由用例钉死。
# 全量 262165 个状态在 debug 下要几分钟，所以平时 `cargo test` 里那条用例标 ignored；--full 在 release 下带 --include-ignored 跑它，
# 把用例打印的 `LAYER0 …` 计数行与逐条不变量的 `CHECKER …` 行报出来。exhaustive=true 才算全量，不是全量判红——层 0 全量是里程碑出口。
# 判别力：用例自己对着产物的十个计数断言（states / violations / root_persisted … 逐字），oracle 的判别力由同文件的靶向阳性对照证明
# （根槽已持久而某个单元两份都没持久 ⇒ 8 个单元逐个都判红）。本阶段没有 fixtures 样本：判红要 cargo 真跑，装不进 fixtures 目录。
#
# 多线程（增补 2 收口表第 41 行；2026-09-18 用户定：测试与崩溃检测优先多线程）：crash.rs 按状态序号区间切片、多线程跑，
# 线程数由 SINGLEFS_LAYER0_THREADS 传进去——没设就取本机核数（nproc）。每跑完一片，用例打一行 `LAYER0_PROGRESS`（片号、序号区间、段号、
# 已跑完的状态数），这里边跑边转到本阶段的输出里，不删；成功行里报实际起了几个工作线程。
# --full 里没显式把 SINGLEFS_LAYER0_THREADS 设成 1、而本机多于 1 核却只起了 1 个工作线程：判红（多半是线程数没传进去）。
# --full 里 LAYER0 行的下一行不是 CHECKER 逐条不变量行：判红（不然成功句里「逐条不变量」打出来是空串，整道照样绿）。
# 这两支判红都只拿合成日志核过：把判定段抽进临时脚本，喂一份缺那一行的日志。
# 标记那一半（相等判绿、改一个输入字节判红、没有标记判红、--full 判红不写标记、分格互不删、同一批输入两趟 --full 撞车不误红、同一批输入先绿后红删掉那一格、只改 Cargo.lock 判红、标记里 exhaustive=false 判红）拿临时仓加一个打合成日志的假 cargo 核过，同样不在 fixtures 里。
set -uo pipefail
# 参数：`--full` 与项目根，顺序不限；gate.sh 只传项目根，于是整轮门禁走快档。
layer0_tier="quick"
root_argument=""
for stage_argument in "$@"; do
  case "$stage_argument" in
    --full) layer0_tier="full" ;;
    -*)
      echo "  ✗ 认不出的参数：$stage_argument"
      echo "     → 怎么办：只认 --full 与项目根，顺序不限：bash .claude/gate.d/54-layer0-replay.sh [--full] [项目根]"
      exit 2 ;;
    *) root_argument="$stage_argument" ;;
  esac
done
ROOT="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

# 这一道读的路径：`.claude/gate.d/stage-inputs.tsv` 里登记给本阶段的那几条（唯一登记位）。快档的改动范围与两档的输入哈希都按它算。
layer0_stage_file_name="$(basename "$0")"
layer0_input_table="$ROOT/.claude/gate.d/stage-inputs.tsv"
layer0_registered_input_paths=()
if [[ -f "$layer0_input_table" ]]; then
  while IFS= read -r registered_row; do
    read -r -a row_paths <<< "$registered_row"
    layer0_registered_input_paths+=("${row_paths[@]}")
  done < <(awk -F'\t' -v stage="$layer0_stage_file_name" '$1 == stage { print $2 }' "$layer0_input_table")
fi
if (( ${#layer0_registered_input_paths[@]} == 0 )); then
  echo "  ✗ $layer0_input_table 里没有 $layer0_stage_file_name 这一行（或读不到这份表）：判不出这一道读哪些路径"
  echo "     → 怎么办：在 .claude/gate.d/stage-inputs.tsv 里给本阶段登记它读的路径（制表符分隔，照别的行写）。"
  exit 1
fi
layer0_input_paths_text="${layer0_registered_input_paths[*]}"

# 快档先问两件事，任一答「可跳过」就退 77（本次未跑），不退 0——`exit 0` 的跳过在汇总里与「判过了」
# 一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。--full 是收尾时点名要跑的，这两问都不问。
# 一问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
# 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
# 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
# 二问这次改动碰没碰登记给本阶段的那几条路径。这是 C8（范围判定）的粗粒度前身：它只摘得掉「零行输入的改动」，摘不出别的，C8 照旧欠着。
if [[ "$layer0_tier" == quick ]]; then
  reuse_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "$ROOT" "$(basename "$0")")"
  reuse_rc=$?
  if [[ "$reuse_rc" != 0 ]]; then
    echo "  ! 本阶段跳过（复用上一次整轮全绿的判定）：$reuse_reason"
    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。"
    exit 77
  fi
  scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" "${layer0_registered_input_paths[@]}")"
  scope_rc=$?
  if [[ "$scope_rc" != 0 ]]; then
    echo "  ! 本阶段跳过（这次改动没碰它判的东西）：$scope_reason"
    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 ${layer0_input_paths_text}，判据见 research/scripts/change-touches-crates.sh。"
    exit 77
  fi
fi
[[ -f Cargo.toml && -d crates/singlefs-harness ]] || { echo "  ! 没有 crates/singlefs-harness，本阶段跳过（步 0 之前没有装置）"; exit 77; }

# 全绿标记放 git 的 common-dir：不进工作树，主工作树与 `gate.sh --staged` 的临时 worktree 读写的是同一份。
if ! git_common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$git_common_directory" ]]; then
  echo "  ✗ $ROOT 不是 git 工作树（取不到 git common-dir）：层 0 全量的全绿标记没处放、也没处读"
  echo "     → 怎么办：在这个项目的 git 仓里跑，或把仓库根作为参数传进来；标记在 \$(git rev-parse --git-common-dir)/singlefs-layer0-full-green.<输入哈希>。"
  exit 1
fi
# 全绿标记按输入哈希分格：一格一个文件，文件名是这个前缀加「.<输入哈希>」。
full_green_marker_prefix="$git_common_directory/singlefs-layer0-full-green"
layer0_scratch_directory="$(mktemp -d)"
trap 'rm -rf -- "${layer0_scratch_directory:?}"' EXIT

machine_cores="$(nproc)"
if [[ -n "${SINGLEFS_LAYER0_THREADS:-}" ]]; then
  threads_origin="显式设的"
else
  SINGLEFS_LAYER0_THREADS="$machine_cores"
  threads_origin="没设，取本机核数"
fi
export SINGLEFS_LAYER0_THREADS

# run_layer0_test_binary <测试二进制> <日志>：cargo 的整段输出进日志；`LAYER0_PROGRESS` 行边跑边转到本阶段的输出（跑全量时这里一直在涨）。
# --full 带 --include-ignored（全量用例在平时 cargo test 里标 ignored）；快档不带，只跑不标 ignored 的那几条。
run_layer0_test_binary() {
  local -a libtest_selection=()
  if [[ "$layer0_tier" == full ]]; then libtest_selection=(--include-ignored); fi
  cargo test --release -p singlefs-harness --test "$1" -- "${libtest_selection[@]}" --nocapture 2>&1 \
    | tee "$2" \
    | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } \
    | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# worker_threads_of_full_run <日志> <计数行>：计数行里的状态数对上的那一行 `LAYER0_PARALLEL_FINISHED`，取实际起的工作线程数。
worker_threads_of_full_run() {
  local states
  states="$(sed -n 's/^[A-Z0-9]* states=\([0-9]*\) .*/\1/p' <<<"$2")"
  grep "^LAYER0_PARALLEL_FINISHED states=$states " "$1" | head -1 | sed -n 's/.* worker_threads=\([0-9]*\) .*/\1/p'
}

# 没显式设成 1、本机多于 1 核却只起了 1 个工作线程（或根本没打收尾行）：判红。返回 0 表示线程数没问题。
worker_threads_are_acceptable() { # <流的名字> <实际起的工作线程数>
  if [[ -z "$2" ]]; then
    echo "  ✗ $1：全量用例跑过了，却没打印状态数对得上的 LAYER0_PARALLEL_FINISHED 行，判不出起了几个工作线程"
    echo "     → 怎么办：全量那条用例要经 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0_in_state_slices 跑（它开跑与跑完各打一行 LAYER0_PARALLEL_*）；"
    echo "                绕开它自己逐个跑状态的写法退回了单线程，改回去。"
    return 1
  fi
  if [[ "$2" == 1 && "$machine_cores" -gt 1 && ! ( "$threads_origin" == "显式设的" && "$SINGLEFS_LAYER0_THREADS" == 1 ) ]]; then
    echo "  ✗ $1：本机 $machine_cores 核、SINGLEFS_LAYER0_THREADS=$SINGLEFS_LAYER0_THREADS（$threads_origin），全量枚举却只起了 1 个工作线程"
    echo "     → 怎么办：看 crash.rs 的 Layer0Parallelism::from_environment 读没读到 SINGLEFS_LAYER0_THREADS、state_slices 切出来的片数够不够分给每个线程；"
    echo "                真要单线程跑（比对单进程读数），显式写 SINGLEFS_LAYER0_THREADS=1 bash .claude/gate.d/54-layer0-replay.sh。"
    return 1
  fi
  return 0
}

# write_layer0_input_manifest <清单文件>：这一道输入的逐文件清单（「sha256  路径」，按路径排序）写进清单文件，
# 整张清单的 sha256 与文件数放进 layer0_input_hash / layer0_input_file_count。路径取 layer0_registered_input_paths。
# 返回非 0：git 列不出文件、一个文件都没有、或读文件出错——三种都判不出哈希。
write_layer0_input_manifest() {
  local manifest_file="$1" listed_file
  local -a existing_files=()
  git -C "$ROOT" ls-files -z --cached --others --exclude-standard -- "${layer0_registered_input_paths[@]}" > "$manifest_file.listing" || return 1
  # 工作区里删了、删除还没暂存的文件不算：跑的是磁盘上这一份，--staged 的临时 worktree 里它还在，两边照样对不上
  while IFS= read -r -d '' listed_file; do
    if [[ -f "$ROOT/$listed_file" ]]; then existing_files+=("$listed_file"); fi
  done < <(LC_ALL=C sort -zu "$manifest_file.listing")
  (( ${#existing_files[@]} > 0 )) || return 1
  ( cd "$ROOT" && sha256sum -- "${existing_files[@]}" ) > "$manifest_file" || return 1
  layer0_input_hash="$(sha256sum < "$manifest_file" | cut -d' ' -f1)"
  layer0_input_file_count="${#existing_files[@]}"
  return 0
}

fail_without_input_manifest() {
  echo "  ✗ 算不出这一道输入的内容哈希：git 在登记的路径（${layer0_input_paths_text}）下一个文件都列不出来，或读文件出错"
  echo "     → 怎么办：在项目根跑 git ls-files -co --exclude-standard -- ${layer0_input_paths_text}，看列不列得出文件；"
  echo "                列得出就逐个 sha256sum 一遍，找读不了的那一个。登记的路径写错了，改 .claude/gate.d/stage-inputs.tsv。"
  exit 1
}

# report_manifest_differences <前一份清单> <后一份清单> <前一份的叫法> <后一份的叫法>：逐个列出两份清单里不同的文件，最多 20 个，另报总数。
report_manifest_differences() {
  local difference_lines difference_count
  difference_lines="$(awk -v earlier_name="$3" -v later_name="$4" '
    FNR == NR { earlier_hash[substr($0, 67)] = substr($0, 1, 64); next }
    {
      later_path = substr($0, 67)
      if (!(later_path in earlier_hash)) print "只在" later_name "里：" later_path
      else if (earlier_hash[later_path] != substr($0, 1, 64)) print "内容不同：" later_path
      delete earlier_hash[later_path]
    }
    END { for (earlier_path in earlier_hash) print "只在" earlier_name "里：" earlier_path }
  ' "$1" "$2" | LC_ALL=C sort)"
  difference_count="$(grep -c . <<< "$difference_lines")"
  echo "       不同的文件共 ${difference_count} 个（最多列 20 个）："
  sed -n '1,20p' <<< "$difference_lines" | sed 's/^/         /'
}

# print_staged_worktree_full_commands：出路里建「HEAD + 暂存区」worktree、在里面用那棵树里的 54 号跑 --full 的命令。
# 建法与共享 gate.sh --staged 相同：worktree add --detach HEAD，再 apply --index 暂存区的 diff（diff 为空就不套）。
print_staged_worktree_full_commands() {
  echo '                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"'
  echo '                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }'
  echo '                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"'
}

# report_newest_other_marker <这一次的清单>：这批输入没有自己那一格时，拿 common-dir 里最近写的一格与这一次比，列出不同的文件；
# 不带哈希的旧名字标记是分格之前写的，不再认，有就点名。
report_newest_other_marker() {
  local newest_marker newest_marker_manifest
  newest_marker="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green.*' ! -name '*.partial.*' -printf '%T@\t%p\n' 2>/dev/null | sort -rn | head -1 | cut -f2-)"
  if [[ -n "$newest_marker" ]]; then
    echo "       common-dir 里最近写的一格是 $(basename "$newest_marker")（跑完于 $(sed -n 's/^finished_utc=//p' "$newest_marker" | head -1)），与这一次的输入比："
    newest_marker_manifest="$layer0_scratch_directory/newest-marker-manifest"
    sed -n 's/^input_file //p' "$newest_marker" > "$newest_marker_manifest"
    report_manifest_differences "$newest_marker_manifest" "$1" "那一格" "这一次"
  else
    echo "       common-dir 里一格全绿标记都没有。"
  fi
  if [[ -f "$full_green_marker_prefix" ]]; then
    echo "       不带哈希的旧名字标记（$full_green_marker_prefix）是按输入哈希分格之前写的，不再认：它罩不到任何一批，可以删掉。"
  fi
}

# run_quick_tier_of_stream <测试二进制> <流的名字>：快档跑一条流。判红打出路、返回 1；判绿把这条流的计数接到 quick_tier_report 后面。
run_quick_tier_of_stream() {
  local quick_log passed_and_ignored passed_count ignored_count
  quick_log="$layer0_scratch_directory/quick-$1.log"
  if ! run_layer0_test_binary "$1" "$quick_log"; then
    tail -40 "$quick_log"
    echo "  ✗ $2的快档用例判红（上面是 cargo test 的尾部）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test $1 -- --nocapture"
    echo "                断言消息里是第一处对不上的计数或违例；改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
    return 1
  fi
  passed_and_ignored="$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$quick_log" | head -1)"
  read -r passed_count ignored_count <<< "$passed_and_ignored"
  if [[ -z "${passed_count:-}" || "$passed_count" == 0 ]]; then
    echo "  ✗ $2的快档跑过了，却读不到 cargo 的 test result 行，或一条用例都没通过：扫到 0 条不是通过"
    echo "     → 怎么办：cargo test --release -p singlefs-harness --test $1 -- --list 看这个测试二进制里还剩几条不标 ignored 的用例；"
    echo "                一条都没有，就是快用例被整批标了 ignored 或删掉了，补回来。"
    return 1
  fi
  quick_tier_report+="${quick_tier_report:+；}$2 ${passed_count} 条通过、${ignored_count} 条 ignored"
  return 0
}

# ── 快档：两条流不标 ignored 的用例，再核对全绿标记 ─────────
if [[ "$layer0_tier" == quick ]]; then
  quick_manifest="$layer0_scratch_directory/quick-manifest"
  write_layer0_input_manifest "$quick_manifest" || fail_without_input_manifest
  full_green_marker_path="$full_green_marker_prefix.$layer0_input_hash"
  quick_tier_report=""
  run_quick_tier_of_stream first_transaction_step_seven_layer0 "第一个事务那条流" || exit 1
  run_quick_tier_of_stream second_transaction_step_zero_layer0 "两次发布那条流" || exit 1
  if [[ ! -f "$full_green_marker_path" ]]; then
    echo "  ✗ 快档绿了（${quick_tier_report}），但这批输入（哈希 ${layer0_input_hash:0:16}…，${layer0_input_file_count} 个文件）没有层 0 全量的全绿标记（$full_green_marker_path）"
    report_newest_other_marker "$quick_manifest"
    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
    print_staged_worktree_full_commands
    exit 1
  fi
  marker_input_hash="$(sed -n 's/^input_hash=//p' "$full_green_marker_path" | head -1)"
  if [[ "$marker_input_hash" != "$layer0_input_hash" ]]; then
    echo "  ✗ 快档绿了（${quick_tier_report}），但这批输入那一格全绿标记里记的输入哈希（${marker_input_hash:-标记里读不到}）与这一次的（${layer0_input_hash}，${layer0_input_file_count} 个文件）不同：那一格被改过或拷错了"
    marker_manifest="$layer0_scratch_directory/marker-manifest"
    sed -n 's/^input_file //p' "$full_green_marker_path" > "$marker_manifest"
    report_manifest_differences "$marker_manifest" "$quick_manifest" "那一格" "这一次"
    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
    print_staged_worktree_full_commands
    exit 1
  fi
  marker_count_lines="$(grep -E '^(LAYER0|CHECKER|LAYER0B) ' "$full_green_marker_path")"
  marker_count_line_total="$(grep -c . <<< "$marker_count_lines")"
  if [[ "$marker_count_line_total" != 3 ]]; then
    echo "  ✗ 这批输入那一格全绿标记的哈希对得上，计数行却不是 LAYER0 / CHECKER / LAYER0B 恰好三行（数到 ${marker_count_line_total} 行）：标记不是 --full 写的，或被改过"
    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
    print_staged_worktree_full_commands
    exit 1
  fi
  # 写这一格的 --full 本该把「不是全量」判红、不写标记；标记里仍有 exhaustive 不是 true 的，说明跑的那一份 54 号的判定被改过
  marker_exhaustive_total="$(grep -cE '^LAYER0B? (.* )?exhaustive=true( |$)' <<< "$marker_count_lines")"
  if [[ "$marker_exhaustive_total" != 2 ]]; then
    echo "  ✗ 这批输入那一格全绿标记里，LAYER0 与 LAYER0B 两行不都是 exhaustive=true（是的只有 ${marker_exhaustive_total} 行）：写它的那一趟 --full 没把「不是全量」判红"
    grep -E '^LAYER0B? ' <<< "$marker_count_lines" | sed 's/^/         /'
    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
    print_staged_worktree_full_commands
    echo "                那一趟判「层 0 不是全量」就是这一批让枚举退化了，去 crash.rs 的 enumerate_layer0 看。"
    exit 1
  fi
  marker_finished_utc="$(sed -n 's/^finished_utc=//p' "$full_green_marker_path" | head -1)"
  echo "  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：${quick_tier_report}"
  echo "  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（${layer0_input_hash:0:16}…，${layer0_input_file_count} 个文件，登记路径 ${layer0_input_paths_text}），两条流都是 exhaustive=true：层 0 全量跑完于 ${marker_finished_utc}，标记里的计数行原样："
  sed 's/^/      /' <<< "$marker_count_lines"
  exit 0
fi

# ── --full：记下开跑时的输入清单，跑完对一遍；开跑一格都不删，这一趟没写成标记就退出时才删这批输入那一格 ─────────
full_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
manifest_at_start="$layer0_scratch_directory/manifest-at-start"
write_layer0_input_manifest "$manifest_at_start" || fail_without_input_manifest
input_hash_at_start="$layer0_input_hash"
full_green_marker_path="$full_green_marker_prefix.$input_hash_at_start"
# 这一趟判红（任何一处 exit）或被打断：同一批输入先绿后红，前一趟那一格不再作数，退出前删掉它；写成了就留着
full_marker_written=0
trap 'if [[ "$full_marker_written" != 1 ]]; then rm -f -- "${full_green_marker_path:?}"; fi; rm -rf -- "${layer0_scratch_directory:?}"' EXIT
echo "  · --full 开跑（${full_started_utc}）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 ${input_hash_at_start:0:16}…（${layer0_input_file_count} 个文件，登记路径 ${layer0_input_paths_text}）"

log="$(mktemp)"
if ! run_layer0_test_binary first_transaction_step_seven_layer0 "$log"; then
  tail -40 "$log"
  echo "  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）"
  echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture"
  echo "                oracle 报的第一条违例在断言消息里。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
  rm -f "$log"
  exit 1
fi
line="$(grep '^LAYER0 ' "$log" | head -1)"
checker_line="$(grep -A1 '^LAYER0 ' "$log" | grep '^CHECKER ' | head -1)"
worker_threads="$(worker_threads_of_full_run "$log" "$line")"
rm -f "$log"
if [[ -z "$line" ]]; then
  echo "  ✗ 用例跑过了，却没打印 LAYER0 计数行"
  echo "     → 怎么办：first_transaction_step_seven_layer0.rs 里全量那条用例要 println! 一行以 LAYER0 开头的计数（字段见该用例的文档注释）。"
  exit 1
fi
if [[ -z "$checker_line" ]]; then
  echo "  ✗ 用例跑过了，LAYER0 计数行的下一行却不是以 CHECKER 开头的逐条不变量行：成功句里「逐条不变量」那一句会是空话"
  echo "     → 怎么办：first_transaction_step_seven_layer0.rs 里全量那条用例打完 LAYER0 行要紧跟着 println! checker_line(&tally) 那一行；"
  echo "                两行之间插进了别的输出，就把 CHECKER 那一行挪回 LAYER0 行的正下方。"
  exit 1
fi
if [[ "$line" != *"exhaustive=true"* ]]; then
  echo "  ✗ 层 0 不是全量：$line"
  echo "     → 怎么办：枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)；少了说明某一段没展开子集，去 crash.rs 的 enumerate_layer0 看。"
  exit 1
fi
worker_threads_are_acceptable "第一个事务那条流" "$worker_threads" || exit 1
echo "  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；${worker_threads} 个工作线程，SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核）：${line#LAYER0 }"
echo "  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：${checker_line#CHECKER }"
# 第二个事务（发布 B）：取号 → 暖机 → A → B 整条流，多版本 oracle（里程碑「第二个事务」步 0 / 步 6 在 B 上的那一半）。
log_b="$(mktemp)"
if ! run_layer0_test_binary second_transaction_step_zero_layer0 "$log_b"; then
  tail -40 "$log_b"
  echo "  ✗ 两次发布那条流的层 0 用例判红（上面是 cargo test 的尾部）"
  echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- --include-ignored --nocapture"
  echo "                多版本 oracle 报的第一条违例在断言消息里：实际走的根是哪一代就得读出那一代的内容。"
  rm -f "$log_b"
  exit 1
fi
line_b="$(grep '^LAYER0B ' "$log_b" | head -1)"
worker_threads_b="$(worker_threads_of_full_run "$log_b" "$line_b")"
rm -f "$log_b"
if [[ -z "$line_b" ]]; then
  echo "  ✗ 两次发布那条流的用例跑过了，却没打印 LAYER0B 计数行"
  echo "     → 怎么办：second_transaction_step_zero_layer0.rs 里全量那条用例要 println! 一行以 LAYER0B 开头的计数。"
  exit 1
fi
if [[ "$line_b" != *"exhaustive=true"* ]]; then
  echo "  ✗ 两次发布那条流的层 0 不是全量：$line_b"
  echo "     → 怎么办：枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)；少了说明某一段没展开子集，去 crash.rs 的 enumerate_layer0_selecting_versions 看。"
  exit 1
fi
worker_threads_are_acceptable "两次发布那条流" "$worker_threads_b" || exit 1
echo "  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；${worker_threads_b} 个工作线程，SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核）：${line_b#LAYER0B }"

# ── --full 判绿：跑完再算一次输入，与开跑时相同才写全绿标记 ─────────
manifest_at_finish="$layer0_scratch_directory/manifest-at-finish"
write_layer0_input_manifest "$manifest_at_finish" || fail_without_input_manifest
if [[ "$layer0_input_hash" != "$input_hash_at_start" ]]; then
  echo "  ✗ 全量跑的过程中这一道的输入变了（开跑 ${input_hash_at_start:0:16}…，跑完 ${layer0_input_hash:0:16}…）：两条流读到的不一定是同一版，不写全绿标记"
  report_manifest_differences "$manifest_at_start" "$manifest_at_finish" "开跑时" "跑完时"
  echo "     → 怎么办：别在有人改这些路径的树里跑。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
  print_staged_worktree_full_commands
  exit 1
fi
full_finished_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
marker_being_written="$full_green_marker_path.partial.$$"
if ! {
  echo "# 层 0 全量全绿标记：.claude/gate.d/54-layer0-replay.sh --full 判绿之后写，按输入哈希分格，整轮门禁的快档按这批输入的哈希读这一格。不进工作树，别手改。"
  echo "input_hash=$layer0_input_hash"
  echo "input_file_count=$layer0_input_file_count"
  echo "input_paths=$layer0_input_paths_text"
  echo "started_utc=$full_started_utc"
  echo "finished_utc=$full_finished_utc"
  echo "judged_root=$ROOT"
  echo "worker_threads=第一个事务那条流 ${worker_threads}、两次发布那条流 ${worker_threads_b}；SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核"
  echo "$line"
  echo "$checker_line"
  echo "$line_b"
  sed 's/^/input_file /' "$manifest_at_finish"
} > "$marker_being_written" || ! mv -f -- "$marker_being_written" "$full_green_marker_path"; then
  rm -f -- "${marker_being_written:?}"
  echo "  ✗ 全量全绿，全绿标记却没写成（$full_green_marker_path）"
  echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full（没有标记，整轮门禁的快档会一直红）。"
  exit 1
fi
full_marker_written=1
marker_slot_total="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green.*' ! -name '*.partial.*' | grep -c .)"
echo "  ✓ 全绿标记写进 $full_green_marker_path（输入哈希 ${layer0_input_hash:0:16}…，${layer0_input_file_count} 个文件；开跑 ${full_started_utc}，跑完 ${full_finished_utc}；common-dir 里现有 ${marker_slot_total} 格）"
