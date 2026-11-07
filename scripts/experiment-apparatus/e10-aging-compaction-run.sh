#!/usr/bin/env bash
# admission: always 整个脚本每次都起：哪一片已有判定、键对不对得上由 prune-server 里的按片判定现答（scripts/experiment-apparatus/long_run_item_verdicts.py），上一次跑完不替这一次作保
# run-condition: command cargo python3 nice
# E10（老化：结构抗老化 vs 事后整理） 第 1 次跑的第一段（跑前登记 research/prompts/e10-r1-prereg.md 第五节「每段跑法」）的分片跑法：
#   装置 scripts/experiment-apparatus/e10_aging_compaction.rs（bin e10-aging-compaction）按（取样点, S）切片跑，片齐了用装置的 --merge 拼成
#   与不带参数跑逐字节相同的产物 research/results/e10-aging-compaction-<日期>.out（日期取 E10_RUN_DATE，默认 2026-10-13）。history-kept: 默认日期是产物文件名的一部分
#
# long-run-shards: 一片 =（取样点, S），七个取样点 × S 三档共二十一片；片名 <取样点>-s<S>；片的次序就是全跑的正式跑次序，装置的 --merge 逐条核次序与条数
# long-run-resume: 每片一条判定，在 prune-server 的 apparatus-items 族、种类 e10-aging-compaction（读写经 scripts/experiment-apparatus/long_run_item_verdicts.py，它文件头「判定库」）：
#   键 = 装置入口 e10_aging_compaction.rs::main 的单元闭包（root:scripts/experiment-apparatus、release 档；admission.py 的 compute_unit_key 算）、工具链、闭包真引到的依赖版本、
#   这一片的参数（取样点、S）；路径、二进制在哪、哪一台都不进键。键相同、库里判绿的片直接取库里的片产物、不起装置；判红的照同一把键记红撤绿、下一趟现判。
#   同一条命令重起就是续跑：判完的片都在库里，只补没判的。本地不留片目录、不读旧片文件（旧产物不导入）。
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
#   多机的读法：每台线程是 scripts/hosts/heavy-threads.sh 的 heavy_thread_default（E10_THREADS 设了就用它）；一半的片由主 agent 整条交统一任务服务：
#   task-server/task submit -- bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --shards <片名,…>，派哪台由服务定（调用方不钉台）；
#   两台各自编、各自跑，判完的片都写进 prune-server 那一份（键由源码算，不看是哪一份二进制打的），不拷、不取回；两半都判完，在哪一台再起一次不带参数的本脚本：
#   二十一片全复用、合并出产物。装置在一趟里不许换（共用段落在 long-run-lib/binary-identity.sh）：开跑时把装置程序文件拷一份进 $E10_LOG_DIRECTORY/pinned-binary/<sha256>/（只读），
#   每片、合并都按这份拷贝起，不再 exec 编译目录里那一份（别的会话的 cargo 随时会把它换掉）；跑着的时候源码被改了，long_run_item_verdicts.py 的 finish 重算键、撤掉这一趟写进库的那几片、判红。
#
# 用法（在仓里任何目录起都行，脚本自己回到仓根）：
#   bash scripts/experiment-apparatus/e10-aging-compaction-run.sh                 判全部片（键相同、库里判绿的复用）；二十一片齐了合并出产物（产物已在就只核、不重写）
#   bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --shards a,b    只判点名的片，不合并（退 3）
#   bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --list-items    逐行列片名，不编、不跑、不连 prune-server
#   bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --selftest      假装置、假 nice、假 cargo 在临时树里走一遍（不起 cargo、不碰真装置）；判定写进临时 prune-server：
#     只在临时服务底下跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem；不在就退 2、一格都不跑），起法：
#     python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --selftest
# 环境：E10_THREADS（装置的线程数，没设取登记的每台默认重活线程数）、E10_RUN_DATE（产物日期）、
#   E10_LOG_DIRECTORY（装置私有副本与这一趟的状态目录放这里，默认工作根下的 e10-run，工作根取 .claude/hooks/lib_work_root.py、两台同一份）、E10_BINARY（装置程序文件；设了就不 cargo build，用它）。
# 成功句报「现判 N、复用 M、拒算 R」（拒算 = 键算不出的片：照跑、不复用、不入库）。
# 退出码：0 二十一片齐、产物在；1 有片判红、合并失败、复用哨兵红或键在跑的中途变了；2 用法错；3 片还没齐（--shards 跑完的正常结局）；75 prune-server 不可用。
#
# 复用三问（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：自证经 long_run_item_verdicts.py reuse-cells 跑那六格（reuse-three-questions、
#   key-independent-of-directory、reuse-sentinel 与哨兵的三种红），假驱动是自证的假装置，main-s4、main-s8 两片的装置入口换成假树里两个 Python 函数（它文件头「复用格」）。
# 弄坏开关 E10_LONG_RUN_BREAK=<项，逗号分隔>（只在 --selftest 起的那几趟里认：自证设 E10_LONG_RUN_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   E10_LONG_RUN_BREAK=exec-root-binary  每片仍按编译目录那个路径起、不按私有副本
#       「装置在一趟里不许换：跑着的时候编译目录里那份被换掉，之后起的二十一片自报的 sha256 都是开跑时那份」
#   E10_LONG_RUN_BREAK=done-marker-ignored  片输出末行不齐也当做完
#       「末行不齐的片判红：退 1、二十一片都记红、不合并；换好装置再起，二十一片都现判」
#   E10_LONG_RUN_BREAK=reuse-ignored  库里判绿的片也照起装置现判（不认 plan 答的 reuse）
#       「原样重起：二十一片全复用、装置一次没起、不编、产物已在不重写」「两台各判一半：第三趟不带参数，二十一片全复用、合并出产物」
#       「reuse-three-questions」「key-independent-of-directory」「reuse-sentinel」「forced-still-reused-red」
#   E10_LONG_RUN_BREAK=finish-ignored  不认 long_run_item_verdicts.py finish 的退出码（复用哨兵红、键中途变了照退 0 / 3）
#       「sentinel-rejudged-red」「forced-verdict-differs-red」「forced-still-reused-red」
# 复用那几格另认 long_run_item_verdicts.py 的弄坏开关 LONG_RUN_ITEMS_BREAK（它文件头；打开后本脚本的自证红在它点名的同名格）。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../hosts/heavy-threads.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "$(dirname "${BASH_SOURCE[0]}")/long-run-lib/binary-identity.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; echo "     → 怎么办：从仓根照文件头的用法起：bash scripts/experiment-apparatus/e10-aging-compaction-run.sh" >&2; exit 2; }

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）；别处设了就拒。
break_is() {
  [[ ",${E10_LONG_RUN_BREAK:-}," == *",$1,"* ]]
}
if [[ -n "${E10_LONG_RUN_BREAK:-}" && "${1:-}" != --selftest ]]; then
  if [[ -z "${E10_LONG_RUN_SELFTEST_MARK:-}" || "$(cat -- "${E10_LONG_RUN_SELFTEST_MARK}" 2> /dev/null)" != e10-long-run-selftest ]]; then
    echo "  ✗ E10_LONG_RUN_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
    echo "     → 怎么办：平常跑时不设它" >&2
    exit 2
  fi
fi

SCRIPT_KEY="scripts/experiment-apparatus/e10-aging-compaction-run.sh"
ITEM_VERDICTS="scripts/experiment-apparatus/long_run_item_verdicts.py"   # 按片判定的读写（文件头 long-run-resume）
LIBRARY_KIND="e10-aging-compaction"                                       # 判定库的种类：装置的 bin 名
RUN_DATE="${E10_RUN_DATE:-2026-10-13}"
PRODUCT="research/results/e10-aging-compaction-$RUN_DATE.out"
BINARY="${E10_BINARY:-}"   # 没给：要起装置时编一次，取 cargo 报的这一趟编出来的位置（long-run-lib/binary-identity.sh 的 binary_identity_build；编译目录由 task-server 给）
LOG_DIRECTORY="${E10_LOG_DIRECTORY:-}"
if [[ -z "$LOG_DIRECTORY" ]]; then   # 没设：工作根（.claude/hooks/lib_work_root.py，两台同一份）下的 e10-run；自证的临时树各格都给了目录，不走这里
  if ! LOG_DIRECTORY="$(python3 .claude/hooks/lib_work_root.py)" || [[ "$LOG_DIRECTORY" != /* ]]; then
    echo "  ✗ 取不到工作根（.claude/hooks/lib_work_root.py 跑不起来），缺省的日志目录没处放" >&2
    echo "     → 怎么办：修好 .claude/hooks/lib_work_root.py，或设 E10_LOG_DIRECTORY 指到日志目录" >&2
    exit 2
  fi
  LOG_DIRECTORY="$LOG_DIRECTORY/e10-run"
fi
POINTS=(main disk_1gib no_remount delete_units snapshots_10 leaf_203 base_6sc)
ROOT_RING_SLOTS_PER_REGION_CHOICES=(4 8 16)

all_shard_names() {
  local point root_ring_slots_per_region
  for point in "${POINTS[@]}"; do
    for root_ring_slots_per_region in "${ROOT_RING_SLOTS_PER_REGION_CHOICES[@]}"; do
      echo "$point-s$root_ring_slots_per_region"
    done
  done
}

# 一份片（或产物）做完了：末行是 name=done emitted=N 且 N 等于行数。
is_complete_output() {
  local file="$1" line_count last_line
  [[ -f "$file" ]] || return 1
  break_is done-marker-ignored && return 0
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" == "E7RESULT name=done emitted=$line_count" ]]
}

# 装置的线程数：E10_THREADS 设了用它，没设取登记的每台默认重活线程数（heavy_thread_default，拒的时候它在 stderr 写原因与出路）。
thread_count() {
  if [[ -n "${E10_THREADS:-}" ]]; then
    echo "$E10_THREADS"
    return
  fi
  heavy_thread_default
}

# 一片的判定请求（long_run_item_verdicts.py 文件头「请求文件」）：装置入口、release 档、这一片的参数；片产物逐字节可比（compare=exact，计时只打到标准错误）
shard_request() { # <片名>
  printf '%s\te10_aging_compaction.rs::main\troot:scripts/experiment-apparatus\tprofile:release\tline:point=%s\tline:root-ring-slots-per-region=%s\tline:compare=exact\n' \
    "$1" "${1%-s*}" "${1##*-s}"
}

decision_of() { # <计划目录> <片名>：plan 答的 judge / reuse / refused
  awk -F'\t' -v item="$2" '$1 == item {print $2}' "$1/decisions.tsv"
}

# 判定库那一步没成：75（prune-server 不可用）照交，别的按 1 交（3 是「片还没齐」，不混用）
library_step_failed() { # <哪一步> <退出码>
  echo "  ✗ 按片判定的 $1 没成（long_run_item_verdicts.py 退 $2）" >&2
  echo "     → 怎么办：退 75 是 prune-server 不可用：python3 prune-server/control check 看服务，修好之后重交同一条命令（报给主 agent）；别的照上面它打的出路改" >&2
  if (($2 == 75)); then exit 75; fi
  if (($2 == 2)); then exit 2; fi
  exit 1
}

record_shard() { # <片名> <退出码> <片产物> [--red]
  python3 "$ITEM_VERDICTS" record "$PLAN_DIRECTORY" "$@" || library_step_failed "record（片 $1）" "$?"
}

# 装置打一片：退 0 且末行齐了才换名成片产物、记绿；不然记红（产物照留着交给 record，便于看）
judge_shard() { # <片名>
  local name="$1" destination="$SHARD_DIRECTORY/$1.out" started finished exit_code
  binary_identity_assert_unchanged || exit 1
  started=$(date +%s)
  E10_THREADS="$THREADS" nice -n 19 "$LAUNCH_BINARY" \
    --point "${name%-s*}" --root-ring-slots-per-region "${name##*-s}" > "$destination.partial" 2> "$destination.err"
  exit_code=$?
  finished=$(date +%s)
  if [[ "$exit_code" == 0 ]] && is_complete_output "$destination.partial"; then
    mv -- "$destination.partial" "$destination"
    echo "片 $name：退 0，$((finished - started)) 秒，$(wc -l < "$destination") 行，现判"
    record_shard "$name" 0 "$destination"
    return 0
  fi
  echo "片 $name：退 $exit_code，$((finished - started)) 秒，判红（末行不齐或装置退非 0；错误输出末尾：$(tail -n 3 -- "$destination.err" | tr '\n' ' ')）" >&2
  ((exit_code != 0)) || exit_code=1
  record_shard "$name" "$exit_code" "$destination.partial" --red
  return 1
}

# 自证：假装置、假 nice、假 cargo 在每一格自己的临时树里走一遍（本脚本与共用库拷进树里，树里 .claude 与 long_run_item_verdicts.py 是指回真仓的符号链接）；不起 cargo、不碰真装置。
# 判定写进临时 prune-server（客户端 selftest_service_problem 判在不在临时服务底下），每格一份命名空间；二十一片的装置入口都换成假树里的 measure_alpha（不然算不出键、全拒算）。
run_selftest() {
  local work failures=0 cell_count=0 fakes real_root cell exit_code first_exit original_sha swap_target product shard_count=21 reuse_output reuse_red name
  unset E10_BINARY E10_LOG_DIRECTORY
  real_root="$PWD"
  python3 "$real_root/$ITEM_VERDICTS" selftest-service || return 2
  work=$(mktemp -d) || return 1
  fakes="$work/fakes"
  mkdir -p -- "$fakes/bin"
  echo e10-long-run-selftest > "$work/mark"
  product="research/results/e10-aging-compaction-2099-01-01.out"
  cat > "$fakes/device" <<'FAKE'
#!/usr/bin/env bash
# 假装置：--point <点> --root-ring-slots-per-region <S> 打一行 row 和收尾行；--merge <片…> 打 merged 和收尾行。
self_sha=$(sha256sum -- "$0" | cut -d' ' -f1)
echo "$*" >> "$E10_SELFTEST_CALLS"
echo "$* $self_sha" >> "$E10_SELFTEST_CALLS.self-sha"
if [[ "$1" == --merge ]]; then
  shift
  printf 'E7RESULT name=merged shards=%s\nE7RESULT name=done emitted=2\n' "$#"
  exit 0
fi
case "${E10_SELFTEST_DEVICE:-ok}" in
  fail) echo "boom" >&2; exit 1 ;;
  truncated) printf 'E7RESULT name=row\nE7RESULT name=done emitted=99\n'; exit 0 ;;
esac
printf 'E7RESULT name=row point=%s s=%s device=%s\nE7RESULT name=done emitted=2\n' "$2" "$4" "$self_sha"
FAKE
  cat > "$fakes/bin/nice" <<'FAKE'
#!/usr/bin/env bash
# 自证的假 nice（每片装置都经 nice -n 19 起）：吞掉 -n 19，起后面的命令。
# E10_SELFTEST_SWAP_TARGET：起装置之前（脚本已拷过私有副本）把这个路径的装置换成另一份字节不同的，像别的会话的 cargo 把编译目录里那份换掉。
if [[ -n "${E10_SELFTEST_SWAP_TARGET:-}" && ! -e "$E10_SELFTEST_SWAP_TARGET.swapped" ]]; then
  { cat -- "$E10_SELFTEST_SWAP_TARGET"; echo "# swapped by a build while the long run is going"; } > "$E10_SELFTEST_SWAP_TARGET.swap" \
    && chmod +x "$E10_SELFTEST_SWAP_TARGET.swap" && mv -f -- "$E10_SELFTEST_SWAP_TARGET.swap" "$E10_SELFTEST_SWAP_TARGET" && : > "$E10_SELFTEST_SWAP_TARGET.swapped"
fi
shift 2
exec "$@"
FAKE
  cat > "$fakes/bin/cargo" <<'FAKE'
#!/usr/bin/env bash
# 假 cargo：记参数；build 时照 cargo 的机读输出报一行 compiler-artifact，executable 指这一格树里默认位置上的假装置（脚本照它取装置，不自己拼路径）
echo "cargo $*" >> "$E10_SELFTEST_CALLS.cargo"
[[ "${1:-}" == build ]] && printf '{"reason":"compiler-artifact","target":{"name":"e10-aging-compaction"},"executable":"%s","fresh":false}\n' "$PWD/scripts/experiment-apparatus/target/release/e10-aging-compaction"
exit 0
FAKE
  chmod +x "$fakes/device" "$fakes/bin/nice" "$fakes/bin/cargo"
  # 样板树：本脚本与共用库的拷贝、真的线程取法、默认位置上的假装置、指向真仓 .claude 与 long_run_item_verdicts.py 的符号链接、假装置入口、空的 git 仓。每格拷一份。
  local tree="$work/template"
  mkdir -p -- "$tree/scripts/experiment-apparatus/long-run-lib" "$tree/scripts/experiment-apparatus/target/release" "$tree/scripts/hosts" "$tree/research/results"
  cp -- "$SCRIPT_PATH" "$tree/scripts/experiment-apparatus/e10-aging-compaction-run.sh"
  cp -- "$(dirname -- "$SCRIPT_PATH")/long-run-lib/binary-identity.sh" "$tree/scripts/experiment-apparatus/long-run-lib/"
  cp -- "$real_root/scripts/hosts/heavy-threads.sh" "$tree/scripts/hosts/"
  cp -- "$fakes/device" "$tree/scripts/experiment-apparatus/target/release/e10-aging-compaction"
  ln -s -- "$real_root/$ITEM_VERDICTS" "$tree/$ITEM_VERDICTS"
  ln -s -- "$real_root/.claude" "$tree/.claude"
  python3 "$real_root/$ITEM_VERDICTS" plant-fake-entries "$tree" || { rm -rf -- "${work:?}"; return 1; }
  git -C "$tree" init -q > /dev/null 2>&1
  while read -r name; do printf '%s\tscripts/experiment-apparatus/long_run_reuse_fake_entries.py::measure_alpha\n' "$name"; done < <(all_shard_names) > "$work/all-shards.tsv"
  make_tree() {
    mkdir -p -- "$1/logs" "$1/given"
    cp -R -- "$work/template" "$1/tree"
    cp -- "$fakes/device" "$1/given/e10-aging-compaction"
  }
  invoke() {
    local cell="$1"
    shift
    ( cd -- "$cell/tree" && env -u SINGLEFS_REUSE_SENTINEL_SEED PATH="$fakes/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E10_SELFTEST_CALLS="$cell/calls" E10_THREADS=3 E10_RUN_DATE=2099-01-01 \
        E10_LOG_DIRECTORY="$cell/logs" E10_LONG_RUN_SELFTEST_MARK="$work/mark" LONG_RUN_ITEMS_LIBRARY_DIRECTORY="$cell/library" \
        LONG_RUN_ITEMS_SELFTEST_ENTRIES="$work/all-shards.tsv" SINGLEFS_REUSE_SENTINEL_COUNT=1000 \
        bash "$cell/tree/scripts/experiment-apparatus/e10-aging-compaction-run.sh" "$@" > "$cell/out" 2>&1 )
  }
  calls_of() { # <格> [<后缀>]：假装置（或假 cargo）被起了几次
    if [[ -f "$1/calls${2:-}" ]]; then wc -l < "$1/calls${2:-}"; else echo 0; fi
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
  invoke "$cell"; exit_code=$?
  expect "全新：二十一片现判、合并出产物、cargo 编一次、成功句报现判 21、复用 0、拒算 0" '((exit_code == 0)) && (($(calls_of "$cell") == shard_count + 1)) && [[ -f "$cell/tree/$product" ]] && (($(calls_of "$cell" .cargo) == 1)) && grep -qF "现判 21、复用 0、拒算 0" "$cell/out"'
  invoke "$cell"; exit_code=$?
  expect "原样重起：二十一片全复用、装置一次没起、不编、产物已在不重写" '((exit_code == 0)) && (($(grep -c "：复用（键相同、库里判绿）" "$cell/out") == shard_count)) && (($(calls_of "$cell") == shard_count + 1)) && (($(calls_of "$cell" .cargo) == 1)) && grep -qF "产物已在" "$cell/out" && grep -qF "现判 0、复用 21、拒算 0" "$cell/out"'
  cell="$work/halves"; make_tree "$cell"
  invoke "$cell" --shards "$(all_shard_names | head -n 10 | paste -sd,)"; first_exit=$?
  invoke "$cell" --shards "$(all_shard_names | tail -n 11 | paste -sd,)"; exit_code=$?
  invoke "$cell"
  expect "两台各判一半：第三趟不带参数，二十一片全复用、合并出产物" '((first_exit == 3 && exit_code == 3)) && grep -qF "现判 0、复用 21、拒算 0" "$cell/out" && [[ -f "$cell/tree/$product" ]] && (($(calls_of "$cell") == shard_count + 1))'
  cell="$work/swap"; make_tree "$cell"
  swap_target="$cell/tree/scripts/experiment-apparatus/target/release/e10-aging-compaction"
  original_sha=$(sha256sum -- "$swap_target" | cut -d' ' -f1)
  E10_SELFTEST_SWAP_TARGET="$swap_target" invoke "$cell"; exit_code=$?
  expect "装置在一趟里不许换：跑着的时候编译目录里那份被换掉，之后起的二十一片自报的 sha256 都是开跑时那份" '((exit_code == 0)) && [[ -e "$swap_target.swapped" ]] && [[ "$(sha256sum -- "$swap_target" | cut -d" " -f1)" != "$original_sha" ]] && [[ "$(grep -v -- "--merge" "$cell/calls.self-sha" | awk "{print \$NF}" | sort -u)" == "$original_sha" ]] && (($(grep -vc -- "--merge" "$cell/calls.self-sha") == shard_count))'
  cell="$work/given"; make_tree "$cell"
  E10_BINARY="$cell/given/e10-aging-compaction" invoke "$cell" --shards main-s4,main-s8; exit_code=$?
  expect "设了 E10_BINARY：不 cargo build、点名的两片现判、退 3" '((exit_code == 3)) && (($(calls_of "$cell" .cargo) == 0)) && (($(calls_of "$cell") == 2)) && grep -qF "现判 2、复用 0、拒算 0" "$cell/out"'
  cell="$work/list"; make_tree "$cell"
  invoke "$cell" --list-items; exit_code=$?
  expect "--list-items：逐行列二十一片、不编、不起装置" '((exit_code == 0)) && (($(wc -l < "$cell/out") == shard_count)) && (($(calls_of "$cell") == 0)) && (($(calls_of "$cell" .cargo) == 0))'
  cell="$work/truncated"; make_tree "$cell"
  E10_SELFTEST_DEVICE=truncated invoke "$cell"; first_exit=$?
  cp -- "$cell/out" "$cell/out.truncated"
  invoke "$cell"; exit_code=$?
  expect "末行不齐的片判红：退 1、二十一片都记红、不合并；换好装置再起，二十一片都现判" '((first_exit == 1)) && (($(grep -c "：红，现判" "$cell/out.truncated") == shard_count)) && grep -qF "21 片判红" "$cell/out.truncated" && ((exit_code == 0)) && grep -qF "现判 21、复用 0、拒算 0" "$cell/out" && [[ -f "$cell/tree/$product" ]]'
  cell="$work/bad-argument"; make_tree "$cell"
  invoke "$cell" --bogus; exit_code=$?
  expect "认不得的参数退 2" '((exit_code == 2)) && grep -qF "认不出的参数：--bogus" "$cell/out" && (($(calls_of "$cell") == 0))'
  invoke "$cell" --shards nonsense; exit_code=$?
  expect "不认得的片名退 2" '((exit_code == 2)) && grep -qF "不认得的片名" "$cell/out" && (($(calls_of "$cell") == 0))'
  # 复用那六格（long_run_item_verdicts.py 文件头「复用格」）：main-s4、main-s8 两片，--shards 跑完的正常结局是退 3
  reuse_output=$(python3 "$real_root/$ITEM_VERDICTS" reuse-cells --script "$SCRIPT_KEY" --template "$work/template" --items main-s4,main-s8 --work "$work/reuse" --green-exit 3 -- \
    env PATH="$fakes/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E10_SELFTEST_CALLS="{tree}/../calls" E10_THREADS=3 E10_RUN_DATE=2099-01-01 \
    E10_LOG_DIRECTORY="{tree}/../logs" E10_LONG_RUN_SELFTEST_MARK="$work/mark" bash "{tree}/scripts/experiment-apparatus/e10-aging-compaction-run.sh" --shards main-s4,main-s8 2>&1)
  echo "$reuse_output"
  reuse_red=$(sed -n 's/^long_run_item_verdicts 复用格.*：6 格，红 \([0-9]*\) 格$/\1/p' <<< "$reuse_output")
  cell_count=$((cell_count + 6))
  if [[ -z "$reuse_red" ]]; then failures=$((failures + 6)); echo "  ✗ 复用那六格没跑完（没有「long_run_item_verdicts 复用格」那一行），按六格红算"; echo "     → 怎么办：看上面 reuse-cells 打的输出里第一处 ✗ 或 Python 报错（复用格起不来多半是 prune-server 不可用或 long_run_item_verdicts.py 抛了异常），修好之后在临时 prune-server 底下单独起本脚本的 --selftest（文件头「用法」那一条）到它退 0"; else failures=$((failures + reuse_red)); fi
  rm -rf -- "${work:?}"
  if ((failures > 0)); then echo "e10-aging-compaction-run.sh 自证：$failures 格红（共 $cell_count 格）"; return 1; fi
  echo "e10-aging-compaction-run.sh 自证：$cell_count 格全绿"
  return 0
}

selected=()
case "${1:-}" in
  "") mapfile -t selected < <(all_shard_names) ;;
  --list-items)
    all_shard_names
    exit 0 ;;
  --selftest)
    run_selftest
    exit $? ;;
  --shards)
    [[ -n "${2:-}" ]] || { echo "  ✗ --shards 后面缺片名（逗号分隔）" >&2; echo "     → 怎么办：--list-items 列出可点名的片，几片用逗号隔开写在 --shards 后面" >&2; exit 2; }
    IFS=',' read -r -a selected <<< "$2"
    known_shard_names="$(all_shard_names)"
    for name in "${selected[@]}"; do
      grep -qxF -- "$name" <<<"$known_shard_names" || { echo "  ✗ 不认得的片名「$name」" >&2; echo "     → 怎么办：bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --list-items 列出可点名的片，照它写" >&2; exit 2; }
    done ;;
  *) echo "  ✗ 认不出的参数：$1" >&2; echo "     → 怎么办：照用法写：e10-aging-compaction-run.sh [--shards <片名,…> | --list-items | --selftest]" >&2; exit 2 ;;
esac

mkdir -p -- "$LOG_DIRECTORY" || exit 1
# 这一趟的状态目录（long_run_item_verdicts.py 的计划、复用片取回的产物、这一趟判的片产物）：跑完、失败、被打断都删
STATE_DIRECTORY=$(mktemp -d "$LOG_DIRECTORY/run-XXXXXX") || exit 1
trap 'rm -rf -- "$STATE_DIRECTORY"' EXIT
PLAN_DIRECTORY="$STATE_DIRECTORY/plan"
SHARD_DIRECTORY="$STATE_DIRECTORY/shards"
mkdir -p -- "$SHARD_DIRECTORY" || exit 1
for name in "${selected[@]}"; do shard_request "$name"; done > "$STATE_DIRECTORY/requests.tsv"
python3 "$ITEM_VERDICTS" plan "$PLAN_DIRECTORY" --script "$SCRIPT_KEY" --kind "$LIBRARY_KIND" --root "$PWD" --requests "$STATE_DIRECTORY/requests.tsv" \
  || library_step_failed plan "$?"

# 要不要装置：有片要现判（或拒算照跑），或者这一趟要合并而产物还没有
need_binary=0
for name in "${selected[@]}"; do [[ "$(decision_of "$PLAN_DIRECTORY" "$name")" == reuse ]] || need_binary=1; done
break_is reuse-ignored && need_binary=1
[[ -z "${1:-}" ]] && ! is_complete_output "$PRODUCT" && need_binary=1
if ((need_binary)); then
  if [[ -z "${E10_BINARY:-}" ]]; then
    binary_identity_build e10-aging-compaction --release --manifest-path scripts/experiment-apparatus/Cargo.toml --bin e10-aging-compaction 2> "$STATE_DIRECTORY/build.log" \
      || { echo "  ✗ 装置编不过，或编过了 cargo 没报出它编在哪（构建日志末尾：$(tail -n 5 -- "$STATE_DIRECTORY/build.log" | tr '\n' ' ')）" >&2; echo "     → 怎么办：照构建日志里第一处 error（或末尾那两行的出路）修好，再起同一条命令（判完的片在库里，复用）" >&2; exit 1; }
    BINARY="$BUILT_BINARY"
  fi
  # 开跑时拷一份私有副本：每片、合并都按这份拷贝起
  SOURCE_BINARY="$BINARY"
  binary_identity_pin "$SOURCE_BINARY" "$LOG_DIRECTORY/pinned-binary" || exit 1
  BINARY="$PINNED_BINARY"
  LAUNCH_BINARY="$BINARY"
  break_is exec-root-binary && LAUNCH_BINARY="$SOURCE_BINARY"
  THREADS=$(thread_count) || { echo "  ✗ 取不到装置的线程数（原因见上一行）" >&2; echo "     → 怎么办：设 E10_THREADS，或照上一行的出路修好 .claude/heavy-thread-default" >&2; exit 2; }
  echo "e10 分片跑法：线程 $THREADS、装置 sha256 $PINNED_BINARY_SHA256（私有副本 $PINNED_BINARY，源 $SOURCE_BINARY）"
fi
failed=0
for name in "${selected[@]}"; do
  if [[ "$(decision_of "$PLAN_DIRECTORY" "$name")" == reuse ]] && ! break_is reuse-ignored; then
    cp -- "$PLAN_DIRECTORY/outputs/$name.out" "$SHARD_DIRECTORY/$name.out" || exit 1
    echo "片 $name：复用（键相同、库里判绿）"
    continue
  fi
  judge_shard "$name" || failed=$((failed + 1))
done
python3 "$ITEM_VERDICTS" finish "$PLAN_DIRECTORY" > "$STATE_DIRECTORY/finish.out"; finish_exit=$?
cat -- "$STATE_DIRECTORY/finish.out"
((finish_exit <= 1)) || library_step_failed finish "$finish_exit"
break_is finish-ignored && finish_exit=0
counts=$(sed -n 's/^LONG_RUN_ITEMS .* \(现判 [0-9]*、复用 [0-9]*、拒算 [0-9]*\)$/\1/p' "$STATE_DIRECTORY/finish.out")
[[ "$failed" == 0 ]] || { echo "  ✗ $failed 片判红（$counts）" >&2; echo "     → 怎么办：看上面那几片各自的错误输出，修好之后同一条命令重起：判绿的片复用，判红的现判" >&2; exit 1; }
((finish_exit == 0)) || { echo "  ✗ 按片判定收尾判红（复用哨兵或键在跑的中途变了，见上面 ✗ 那几行；$counts）" >&2; echo "     → 怎么办：照上面那几行的出路办，再起同一条命令" >&2; exit 1; }
[[ -z "${1:-}" ]] || { echo "点名的片判完（$counts），不合并"; exit 3; }

shard_files=()
while read -r name; do shard_files+=("$SHARD_DIRECTORY/$name.out"); done < <(all_shard_names)
if is_complete_output "$PRODUCT"; then
  echo "产物已在：$PRODUCT（不重写）；按片判定 $counts"
  exit 0
fi
[[ ! -e "$PRODUCT" ]] || { echo "  ✗ $PRODUCT 在而收尾行不对，不覆盖" >&2; echo "     → 怎么办：交主 agent 看那份产物；要重跑就设 E10_RUN_DATE=<跑的那天> 写一份新产物，不盖旧的" >&2; exit 1; }
"$BINARY" --merge "${shard_files[@]}" > "$STATE_DIRECTORY/merged" 2> "$STATE_DIRECTORY/merged.err" \
  && is_complete_output "$STATE_DIRECTORY/merged" \
  || { echo "  ✗ 合并失败（错误输出末尾：$(tail -n 5 -- "$STATE_DIRECTORY/merged.err" | tr '\n' ' ')）" >&2; echo "     → 怎么办：修好装置的合并那一步再起同一条命令（判完的片在库里，复用）" >&2; exit 1; }
mv -- "$STATE_DIRECTORY/merged" "$PRODUCT"
echo "产物：$PRODUCT（$(wc -l < "$PRODUCT") 行，sha256 $(sha256sum "$PRODUCT" | cut -d' ' -f1)）；按片判定 $counts"
