#!/usr/bin/env bash
# 门禁 crates-mutation-replay（crates 变异表复跑）的分片驱动，只由 task-server 按分片协议起（用户：「现在就把分片收进服务」，records/2026-10-30-编译绕过内存总上限.md）：
# 本机库里没有键相同判定的行按用时表分成两堆，服务把两堆当两片各起一趟 crates-mutation-replay、只判自己那一堆（GATE_MUTATION_ROW_SELECTION）；
# 两片判出的记录都经 prune-server 写进同一份判定库（.claude/rules/verification.md「判定库只有一份」），不拷库目录、不导入；
# 合并那一步只核两片判的行按本机此刻的行键在服务里读得到（scripts/mutation/crates-mutation-rows.py row-plan）。
# 判定不在这里：之后照常经 task-server 跑门禁 crates-mutation-replay 整张判（行键对得上的复用、对不上的现判），判定与不分片逐字相同（复用的报数不同）。
# 门禁自己不调这一份、不自己分两台，这一份也不自己连第二台、不给哪一片套内存包装（机器、上限、清场由服务定，收尾批 GT02、M03）。
#
#   task-server/task submit -- bash scripts/mutation/mutation-shard-run.sh . [--only-rows <选行文件>]
#                                      在仓根提交（工作目录与树根都在 NFS 共享根下，第二台在同一个绝对路径里跑同一棵树）才派得到第二台；服务拆成计划、各片、合并几个子任务（task-server/task_server_lib.py 的 SHARDED_ENTRIES），
#                                      这一份照环境里的 TASK_SERVER_SHARD_PHASE 只跑其中一步（各变量的写法在 task-server/task_server_lib.py 文件头「分片」）。
#                                      --only-rows 的选行文件一行一个变异表行号：只在选中的行里分（③ 同一套规则，两处不同：要现判的有 1 行就分、不等 2 行；两堆用时相等时放另一堆）。
#                                      没有 TASK_SERVER_SHARD_PHASE（不是服务按分片协议起的，task-server 起的一件普通任务里也算）退 2、什么都不分，出路写上面那一条提交写法；
#                                      在选行模式下被调（环境里设着 GATE_MUTATION_ROW_SELECTION）退 2；弄坏开关 MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外退 2
#   mutation-shard-run.sh --selftest   自证：假 cargo、假 llvm 工具，在临时目录里的小仓上照 task-server 设的变量逐步起三步，逐格见文件末尾 mutation_shard_selftest 的说明；格数由成功行现算。
#                                      退 0 都对；退 1 有格判错
#
# 三步（TASK_SERVER_SHARD_PHASE）：
#   plan   ① 配置照 scripts/crash/crash-point-replay-shard-configuration-check.sh（只调，与崩溃点重放分片同一份）：判法退 3（多机配置没写 ENABLE_ACROSS_MACHINES=1）
#          写 0 片；判不过退 1。③ 分行（mutation_shard_split_plan）：crates-mutation-rows.py row-plan 列出每一行的行键、这一趟会不会复用、上一次现跑的用时与锚点失效没有；
#          要现判的行（拒算的不算：另一片判出来也没有键可导，留给最后那一趟在本机判；第五列是 anchor-broken 的锚点失效行不算：这一趟不跑，最后那一趟在本机记锚点失效）
#          按用时从长到短，每条放进合计用时少的那一堆（相等放本机那一堆；带 --only-rows 时放另一堆），没记过用时的按表里记过的中位数算（一条都没记过都按 1 秒）。
#          要现判的少于 2 行（带 --only-rows 时少于 1 行）写 0 片。带 --only-rows 时 row-plan 带 --rows <选行文件>（mutation_shard_row_plan）只对选中的行算键、别的行只给用时（键写 -），
#          没记过用时的中位数仍取全表。本机那一堆与另一堆各写一份选行文件 <计划目录>/shard-<片号>.selection，plan.json 每堆一片：
#          两堆都不钉（records/2026-10-31-task-server巡检.md 问题 94：分片计划不钉台，片里写了 pin 键（空串也算）服务拒，每片只写 label），各由服务照记录挑机器；两片判的记录都经 prune-server 写进同一份库，
#          拒算与锚点失效的行本来就不进两堆、留给最后那一趟，没有要钉在本机才写得到的东西。不写 memory_floor（一片的上限由服务按记录给）。
#          「本机那一堆」「另一堆」只是分行时两堆的名字（相等放哪一堆照上面的规矩），不是哪一台跑。
#   shard  照 TASK_SERVER_SHARD=<片号>/<片数> 跑那一堆：GATE_MUTATION_ROW_SELECTION=<那一堆> bash <crates-mutation-replay> <树根>，退 1、3 都算这一片成了（退 0），别的照原码退；
#          两片判的记录都写进 prune-server 里同一份库（库根照被判那棵树取，两台同名）；起了门禁的那一片把它判的选行文件拷进 <产物目录>/judged-rows，合并照它核；
#          每一片不看派到哪一台、在跑它的这一台现查 llvm 工具（记覆盖要的 llvm-profdata、llvm-cov，--version 的 LLVM 版本要以这一台 rustc -vV 的「LLVM version」起头；
#          目录取 GATE_MUTATION_LLVM_TOOLS，没设取 rustc --print sysroot 下 lib/rustlib/<host>/bin；缺了退 78、一条变异都不起，出路写 rustup component add llvm-tools），
#          再在树根跑 python3 scripts/hosts/rocksdb-prebuilt.py entry --root .（RocksDB 预编库的取值，判红退 78、一条变异都不起）；两台同一条规则；
#          变异复跑带崩溃判定库的测试各对它自己起的临时 prune-server 跑（scripts/mutation/crates-mutation-rows.py 文件头「变异复跑的崩溃判定库」），这里不设库的变量。
#   merge  照 TASK_SERVER_SHARD_OUTPUTS 逐片读 judged-rows，经 crates-mutation-rows.py row-plan --rows 按本机此刻的源码算行键、查服务（第三列 1 是读得到）：
#          有一片判的行一行都读不到（它写进了别的库）退 1、点名那一片；row-plan 退非 0 退 1；读不到的另几行逐行列出，留给最后那一趟照跑；
#          成功句报两片判的行里读得到几行、读不到几行。不拷库目录、不 import：两台写的是服务里同一份库。
# 真跑那一步（shard）跑一部分 crates/mutations.tsv，起 cargo，与 crates-mutation-replay 同一类，经统一任务服务起；--selftest 的登记档在 .claude/hooks/selftest-registry.tsv，改了就跑（同一份判定）。
#
# 弄坏开关 MUTATION_SHARD_RUN_BREAK=<项>（只给证红用，逗号分隔可给几个；打开时自证退 1）。只在 --selftest 起的进程里生效：自证在它的临时目录里放一份标记文件，
#   路径经 MUTATION_SHARD_RUN_SELFTEST_MARK 带给它起的驱动，文件在、内容是 MUTATION_SHARD_SELFTEST_MARK_CONTENT 那一行才认开关；别处设了它，驱动拒、退 2，什么都没分。
#   逐项写开关名、弄坏哪一处、打开后自证哪几格红
# （项名是自证 expect 的格名，一行一项；失败行是「✗ <格名>：<细节>」，括号里抄细节的句首，随跑变的路径、进程号、指纹写成 …）：
#   MUTATION_SHARD_RUN_BREAK=only-rows-ignored  驱动不看 --only-rows，照整张表分两堆：点名的一行连同别的行都分出去
#       ㉒ 计划那一步带 --only-rows 点名一行：row-plan 只对那一行算键（算键 1 行，不是整张表的 6 行），计划只分那一行（「✗ ㉒ 计划那一步带 --only-rows 点名一行：」；row-plan 那一行写的是算了 6 行的键）
#       ⑱ ③ 分行带选行文件：只在选中的行里分，要现判的有 1 行也分、两堆用时相等放第二台、本机那一堆空着；有 2 行时第二台拿用时长的那一行（「✗ ⑱ ③ 分行带选行文件：」；两堆里有选行文件之外的行）
#   MUTATION_SHARD_RUN_BREAK=row-plan-whole-table  驱动调 row-plan 不带 --rows，照整张表算键：点名一行也付整张表算键的代价
#       ㉒ 计划那一步带 --only-rows 点名一行：row-plan 只对那一行算键（算键 1 行，不是整张表的 6 行），计划只分那一行（「✗ ㉒ 计划那一步带 --only-rows 点名一行：」；row-plan 那一行写的是算了 6 行的键）
#   MUTATION_SHARD_RUN_BREAK=row-plan-selection-ignored  自证里直接起 row-plan 点名两行那一趟带 GATE_MUTATION_BREAK=rowplanselectionignored（crates-mutation-rows.py 的 row-plan 不看 --rows）
#       ㉒ row-plan 点名两行：只对两行算键（算键 2 行），别的 4 行只给用时、键写 -；不点名算整张 6 行（「✗ ㉒ row-plan 点名两行：」；点名两行那一趟写的是算了 6 行的键）
#   MUTATION_SHARD_RUN_BREAK=anchor-broken-dispatched  ③ 分行不看 row-plan 的第五列：锚点失效、而行键算得出的那一行照样分出去
#       ⑭ ③ 分行：锚点失效的行（第五列 anchor-broken，有键没键都算）不进两堆、单报几行（「✗ ⑭ ③ 分行：锚点失效的行（第五列 anchor-broken，有键没键都算）不进两堆、单报几行：本机「1 」第二台「2 4 」」）
#   MUTATION_SHARD_RUN_BREAK=pile-pinned-local  计划给本机那一堆写 pin=local：钉在本机
#       ㉔ 经 task-server 分片：计划写两片（两堆都不钉、片里没有 pin 键，都不带内存下限），两堆合起来是全部 6 行、不相交（「✗ ㉔ 经 task-server 分片：计划写两片」；plan.json：pin=local:False unpinned:False）
#   MUTATION_SHARD_RUN_BREAK=break-honored-anywhere  弄坏开关在 --selftest 之外也认
#       ⑨ MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外（不带自证的标记）：驱动拒、退 2，说设在 --selftest 之外（「退 2；输出：」；输出里没有「设在 --selftest 之外」）
#   MUTATION_SHARD_RUN_BREAK=llvm-tools-missing-skipped  一片在这一台查出缺 llvm 工具时这一堆不判、退 0（不退 78）
#       ㉖ 每一片在跑它的这一台现查 llvm 工具：缺了退 78、一条变异都不起，出路写装哪个包；两片同一条规则（「✗ ㉖ 每一片在跑它的这一台现查 llvm 工具：」；两片退「0 0 」）
#   MUTATION_SHARD_RUN_BREAK=merge-rows-unchecked  合并不查服务，judged-rows 里的行一律当读得到
#       ㉕ 合并：另一片判的行按此刻的行键在服务里一行都读不到（写进了别的库）⇒ 退 1、点名那一片（「✗ ㉕ 合并：」；合并退 0）
#   MUTATION_SHARD_RUN_BREAK=selftest-tmpdir-outside  自证不把 TMPDIR 换成自己临时目录下的 tmp/，场景照外面那一层（自证在调用方的 TMPDIR 下另建的）建临时目录
#       ⑫ 场景都跑在自证自己的 tmp/ 下……跑完外面的临时目录……里没多出 singlefs-mutation-replay-*，自己的 tmp/ 下也一个不剩（「✗ ⑫ 场景都跑在自证自己的 tmp/ 下：」；
#          「工作目录不在自证自己的 tmp/ 下的」：本机那一侧的假 cargo 在外面那一层的 singlefs-mutation-replay-* 里起）
# 只供测试的开关：SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1（照崩溃点重放分片：计划那一步读配置时第二台是本机上的目录）。
#
# admission: always 每次调都分这一刻这棵树上没有作数记录的行；复用判定在判定库记录里，不在这里
# run-condition: command python3 rsync ssh git cargo rustc
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
mutation_shard_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=peer-host-lib.sh
source "$mutation_shard_script_directory/../hosts/peer-host-lib.sh"

mutation_shard_break_is() { [[ ",${MUTATION_SHARD_RUN_BREAK:-}," == *",$1,"* ]]; }

# 自证起的进程的标记（文件头「弄坏开关」那一段）
MUTATION_SHARD_SELFTEST_MARK_CONTENT="mutation-shard-run.sh --selftest 起的进程"
mutation_shard_inside_selftest() {
  [[ -n "${MUTATION_SHARD_RUN_SELFTEST_MARK:-}" && -f "$MUTATION_SHARD_RUN_SELFTEST_MARK" ]] \
    && [[ "$(head -n 1 -- "$MUTATION_SHARD_RUN_SELFTEST_MARK")" == "$MUTATION_SHARD_SELFTEST_MARK_CONTENT" ]]
}

mutation_shard_llvm_check_command() { # 打在跑这一片的这一台查 llvm 工具的那一段：llvm-profdata、llvm-cov 在、llvm-cov 的 LLVM 版本以这一台 rustc 的起头（文件头「三步」shard 那一步）；查得过退 0 打一句版本，查不过退 3 打为什么
  printf 'directory=%q\n' "${GATE_MUTATION_LLVM_TOOLS:-}"
  cat <<'LLVM_CHECK'
if [[ -z "$directory" ]]; then directory="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"; fi
for tool in llvm-profdata llvm-cov; do [[ -x "$directory/$tool" ]] || { echo "$tool 不在或不能执行"; exit 3; }; done
wanted="$(rustc -vV | sed -n 's/^LLVM version: //p')"
[[ -n "$wanted" ]] || { echo "这一台 rustc -vV 读不出 LLVM 版本"; exit 3; }
found="$("$directory/llvm-cov" --version | sed -n 's/.*LLVM version \([^ ]*\).*/\1/p' | head -n 1)"
case "$found" in
  "$wanted"|"$wanted"-*) echo "llvm-cov 的 LLVM $found，rustc 的 $wanted" ;;
  *) echo "llvm-cov 的 LLVM 版本「${found:-读不出}」与这一台 rustc 的「${wanted:-读不出}」对不上"; exit 3 ;;
esac
LLVM_CHECK
}

mutation_shard_row_plan() { # <crates-mutation-rows.py> <树根> <输出文件> [<选行文件>]：③ 列每一行的行键、这一趟复用不复用、上一次的用时，写进输出文件；
  # 带选行文件时只对选中的行算键（row-plan --rows）：点名一行不付整张表算键的代价，别的行只给用时（分行取没记过用时的中位数仍取全表）
  local selection_arguments=()
  if [[ -n "${4:-}" ]]; then
    if mutation_shard_break_is row-plan-whole-table || mutation_shard_break_is only-rows-ignored; then
      echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=row-plan-whole-table（或 only-rows-ignored）：row-plan 不带 --rows，整张表算键（改前的样子）" >&2
    else
      selection_arguments=(--rows "$4")
    fi
  fi
  python3 "$1" row-plan "$2" ${selection_arguments[@]+"${selection_arguments[@]}"} > "$3"
}

mutation_shard_split_plan() { # <row-plan 的输出> <本机选行文件> <第二台选行文件> <键文件> [<只分这几行的选行文件>]：③ 分两堆、写选行文件与键文件，stdout 打一行汇总
  # 汇总：<表里的行数> <要现判的> <记过用时的> <没记过的按几秒> <本机行数> <本机秒数> <第二台行数> <第二台秒数> <拒算的> <锚点失效的>
  # 要现判 = 不复用、有键、第五列不是 anchor-broken（锚点失效的行这一趟不跑，crates-mutation-rows.py 文件头「锚点失效」）
  local anchor_dispatched="" only_rows_file="${5:-}"
  mutation_shard_break_is anchor-broken-dispatched && anchor_dispatched=1
  mutation_shard_break_is only-rows-ignored && only_rows_file=""   # 弄坏开关 only-rows-ignored：不看选行文件，照整张表分
  MUTATION_SHARD_ANCHOR_DISPATCHED="$anchor_dispatched" MUTATION_SHARD_ONLY_ROWS_FILE="$only_rows_file" python3 - "${@:1:4}" <<'PY_SPLIT'
import os, statistics, sys
plan_path, local_path, peer_path, keys_path = sys.argv[1:]
only_rows_path = os.environ.get("MUTATION_SHARD_ONLY_ROWS_FILE", "")   # 只分这几行（--only-rows）：空是整张表
anchor_dispatched = os.environ.get("MUTATION_SHARD_ANCHOR_DISPATCHED") == "1"   # 弄坏开关 anchor-broken-dispatched：不看第五列
with open(plan_path, encoding="utf-8") as handle:
    rows = [line.rstrip("\n").split("\t") for line in handle if line.strip()]
with open(keys_path, "w", encoding="utf-8") as handle:
    handle.write("".join(row[1] + "\n" for row in rows))
def anchor_broken(row):
    return not anchor_dispatched and len(row) > 4 and row[4] == "anchor-broken"
known = [float(row[3]) for row in rows if row[3] != "-"]
default = statistics.median(known) if known else 1.0
if only_rows_path:
    with open(only_rows_path, encoding="utf-8") as handle:
        only_rows = {int(text) for text in (line.strip() for line in handle) if text.isdigit()}
    rows = [row for row in rows if int(row[0]) in only_rows]
pending = sorted(((float(row[3]) if row[3] != "-" else default, int(row[0])) for row in rows if row[2] == "0" and row[1] != "-" and not anchor_broken(row)),
                 key=lambda item: (-item[0], item[1]))
heaps = [[0.0, []], [0.0, []]]
for seconds, line_number in pending:
    if only_rows_path:   # 点名的行：两堆用时相等放第二台（本机排不上才会点名走到这里）
        lighter = heaps[1] if heaps[1][0] <= heaps[0][0] else heaps[0]
    else:
        lighter = heaps[0] if heaps[0][0] <= heaps[1][0] else heaps[1]
    lighter[0] += seconds
    lighter[1].append(line_number)
for path, (_total, lines) in zip((local_path, peer_path), heaps):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{line_number}\n" for line_number in sorted(lines)))
broken = sum(1 for row in rows if anchor_broken(row))
refused = sum(1 for row in rows if row[1] == "-" and not anchor_broken(row))
print(len(rows), len(pending), len(known), f"{default:.1f}", len(heaps[0][1]), f"{heaps[0][0]:.0f}", len(heaps[1][1]), f"{heaps[1][0]:.0f}", refused, broken)
PY_SPLIT
}

mutation_shard_fail() { # <原因> <出路>
  echo "  ✗ 双机分片：$1"
  echo "     → 怎么办：$2；没判的行由 crates-mutation-replay 最后那一趟在本机补跑"
  exit 1
}

# ── 经 task-server 分片（文件头「经 task-server 分片」）───────────────────────────────
mutation_shard_write_plan() { # <计划目录> <一句说明> [<堆名 local|other> <行数>]…：写 plan.json（先写临时文件再改名）；各片都不钉（文件头「三步」plan），弄坏开关 pile-pinned-local 下本机那一堆钉
  MUTATION_SHARD_PILE_PINNED="$(mutation_shard_break_is pile-pinned-local && echo 1)" python3 - "$@" <<'PY_PLAN'
import json, os, sys
directory, note, *rest = sys.argv[1:]
pinned = os.environ.get("MUTATION_SHARD_PILE_PINNED") == "1"
shards = []
for index in range(0, len(rest), 2):
    pile, count = rest[index:index + 2]
    shard = {"label": f"{'本机那一堆' if pile == 'local' else '另一堆'} {count} 行（服务挑台）"}
    if pinned and pile == "local":
        shard["pin"] = "local"
    shards.append(shard)
temporary = os.path.join(directory, "plan.json.partial")
with open(temporary, "w", encoding="utf-8") as handle:
    json.dump({"shards": shards, "note": note}, handle, ensure_ascii=False)
os.replace(temporary, os.path.join(directory, "plan.json"))
PY_PLAN
}

mutation_shard_task_server_plan() { # 文件头「经 task-server 分片」的 plan
  local plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}" configuration_exit=0 split_summary pile count index=0 entries=()
  [[ -n "$plan_directory" && -d "$plan_directory" ]] \
    || mutation_shard_fail "计划那一步没给在的计划目录（TASK_SERVER_SHARD_PLAN_DIRECTORY=${plan_directory:-没设}）" "这一步只由 task-server 起：task-server/task submit -- bash scripts/mutation/mutation-shard-run.sh ."
  peer_host_load_configuration "$mutation_shard_script_directory/../crash/crash-point-replay-shard-configuration-check.sh" "$tree_root" || configuration_exit=$?
  if (( configuration_exit == 3 )); then
    echo "  · 双机：关（多机配置没写 ENABLE_ACROSS_MACHINES=1）：计划写 0 片；没判的行交给 crates-mutation-replay 在本机判"
    mutation_shard_write_plan "$plan_directory" "双机：关，不分" || mutation_shard_fail "写不了 $plan_directory/plan.json" "看计划目录可不可写"
    return 0
  fi
  (( configuration_exit == 0 )) || return 1
  mutation_shard_row_plan "$rows_module" "$tree_root" "$plan_directory/row-plan.tsv" "$only_rows_file" \
    || mutation_shard_fail "本机列不出每一行的记录与用时（crates-mutation-rows.py row-plan 退非 0）" "单跑 python3 scripts/mutation/crates-mutation-rows.py row-plan <树根> [--rows <选行文件>] 看它报什么"
  split_summary="$(mutation_shard_split_plan "$plan_directory/row-plan.tsv" "$plan_directory/pile-local.selection" "$plan_directory/pile-other.selection" "$plan_directory/keys" "$only_rows_file")" \
    || mutation_shard_fail "分行那一段 python 没跑成" "看上面 python 的报错"
  local table_rows pending_rows timed_rows default_seconds local_rows local_seconds peer_rows peer_seconds refused_rows anchor_broken_rows minimum_pending=2
  read -r table_rows pending_rows timed_rows default_seconds local_rows local_seconds peer_rows peer_seconds refused_rows anchor_broken_rows <<< "$split_summary"
  [[ -n "$only_rows_file" ]] && minimum_pending=1
  if (( pending_rows < minimum_pending )); then
    echo "  · ③ $table_rows 行里要现判的只有 $pending_rows 行（拒算的 $refused_rows 行、锚点失效的 $anchor_broken_rows 行不分）：计划写 0 片，交给 crates-mutation-replay 在本机判"
    mutation_shard_write_plan "$plan_directory" "要现判的不够分" || mutation_shard_fail "写不了 $plan_directory/plan.json" "看计划目录可不可写"
    return 0
  fi
  for pile in local other; do
    count="$(grep -c '[0-9]' "$plan_directory/pile-$pile.selection")"
    (( count > 0 )) || continue
    cp -- "$plan_directory/pile-$pile.selection" "$plan_directory/shard-$index.selection" || mutation_shard_fail "拷不了选行文件" "看计划目录可不可写"
    entries+=("$pile" "$count")
    index=$((index + 1))
  done
  mutation_shard_write_plan "$plan_directory" "本机那一堆 $local_rows 行约 $local_seconds 秒、另一堆 $peer_rows 行约 $peer_seconds 秒" "${entries[@]}" \
    || mutation_shard_fail "写不了 $plan_directory/plan.json" "看计划目录可不可写"
  echo "  · ③ $table_rows 行里要现判的 $pending_rows 行按用时两堆均分：本机那一堆 $local_rows 行约 $local_seconds 秒、另一堆 $peer_rows 行约 $peer_seconds 秒，计划写 $index 片（拒算的 $refused_rows 行、锚点失效的 $anchor_broken_rows 行留给最后那一趟）"
}

mutation_shard_task_server_shard() { # 文件头「经 task-server 分片」的 shard
  local plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}" output="${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}" index="${TASK_SERVER_SHARD%%/*}"
  local selection stage_exit=0 llvm_report
  local -a stage_environment=(env -u PEER_HOST_LIB_BREAK)
  [[ -n "$plan_directory" && -d "$plan_directory" && -n "$output" && "$index" =~ ^[0-9]+$ ]] \
    || mutation_shard_fail "这一片缺 task-server 给的变量（计划目录 ${plan_directory:-没设}、产物目录 ${output:-没设}、片号 ${TASK_SERVER_SHARD:-没设}）" "这一步只由 task-server 起"
  mkdir -p -- "$output" || mutation_shard_fail "建不了产物目录 $output" "看它可不可写"
  output="$(cd "$output" && pwd)"
  plan_directory="$(cd "$plan_directory" && pwd)"
  selection="$plan_directory/shard-$index.selection"
  [[ -s "$selection" ]] || mutation_shard_fail "计划目录里没有第 $index 片的选行文件" "计划那一步没写全：看计划那一步的日志"
  # 不看派到哪一台：每一片在跑它的这一台现查 llvm 工具与 RocksDB 预编库的取值（文件头「三步」shard），缺了退 78。弄坏开关 llvm-tools-missing-skipped 下缺 llvm 工具不判、退 0
  if ! llvm_report="$(bash -c "$(mutation_shard_llvm_check_command)" 2>&1)"; then
    if mutation_shard_break_is llvm-tools-missing-skipped; then
      echo "  · 这一台缺 llvm-tools（${llvm_report##*$'\n'}）：这一堆不判"
      return 0
    fi
    echo "  ✗ 这一台缺记覆盖要的 llvm 工具（${llvm_report##*$'\n'}），这一片一条变异都没起（退 78）"
    echo "  → 怎么办：在这一台装 llvm-tools：rustup component add llvm-tools（装进 rustc --print sysroot 下 lib/rustlib/<host>/bin）；工具放在别处的设 GATE_MUTATION_LLVM_TOOLS=<目录>；装好再重投"
    return 78
  fi
  if ! ( cd "$tree_root" && python3 scripts/hosts/rocksdb-prebuilt.py entry --root . ); then
    echo "  ✗ 这一片起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条变异都没起（退 78）"
    echo "  → 怎么办：在这一台上照上面的出路修（没装就跑 python3 scripts/hosts/rocksdb-prebuilt.py install；装了而 ssh 起的命令拿不到取值，看 ~/.bashrc 最前面那一段），再重投"
    return 78
  fi
  echo "  · 第 $TASK_SERVER_SHARD 片（这一台，判的记录写进 prune-server 里同一份判定库）：$(grep -c '[0-9]' "$selection") 行"
  if ( cd "$tree_root" && exec "${stage_environment[@]}" GATE_MUTATION_ROW_SELECTION="$selection" bash "$stage_script" "$tree_root" ); then stage_exit=0; else stage_exit=$?; fi
  # 起了门禁（退 0、1、3）的这一片把它判的行交给合并去核（文件头「三步」merge）；别的退出码这一片没成，不交
  case "$stage_exit" in
    0|1|3) cp -- "$selection" "$output/judged-rows" || mutation_shard_fail "拷不了选行文件到 $output/judged-rows" "看产物目录可不可写" ;;
  esac
  case "$stage_exit" in
    1|3) echo "  · 这一片的 crates-mutation-replay 退 $stage_exit（3 是选中的都抓到、1 是有红或没判完；判定以最后那一趟为准）"; return 0 ;;
    *) echo "  ✗ 这一片的 crates-mutation-replay 退 $stage_exit"; echo "  → 怎么办：看这一片的日志（task-server/task log <号>）"; return "$stage_exit" ;;
  esac
}

mutation_shard_rows_in_service() { # <row-plan 的输出> <judged-rows> <读不到的行写进哪个文件>：stdout 打「<读得到的行数> <读不到的行数>」（文件头「三步」merge）
  MUTATION_SHARD_MERGE_UNCHECKED="$(mutation_shard_break_is merge-rows-unchecked && echo 1)" python3 - "$@" <<'PY_ROWS'
import os, sys
plan_path, judged_path, unreadable_path = sys.argv[1:]
with open(judged_path, encoding="utf-8") as handle:
    judged = sorted({int(text) for text in (line.strip() for line in handle) if text.isdigit()})
with open(plan_path, encoding="utf-8") as handle:
    readable_rows = {int(fields[0]) for fields in (line.rstrip("\n").split("\t") for line in handle if line.strip()) if len(fields) > 2 and fields[2] == "1"}
if os.environ.get("MUTATION_SHARD_MERGE_UNCHECKED") == "1":   # 弄坏开关 merge-rows-unchecked：不查服务，一律当读得到
    readable_rows = set(judged)
unreadable = [line_number for line_number in judged if line_number not in readable_rows]
with open(unreadable_path, "w", encoding="utf-8") as handle:
    handle.write("".join(f"{line_number}\n" for line_number in unreadable))
print(len(judged) - len(unreadable), len(unreadable))
PY_ROWS
}

mutation_shard_task_server_merge() { # 文件头「三步」的 merge：只核两片判的行按本机此刻的行键在 prune-server 里读得到；不拷库目录、不 import
  local outputs="${TASK_SERVER_SHARD_OUTPUTS:-}" index host directory scratch counts readable unreadable merged=0 total_readable=0 total_unreadable=0
  local -a problems=() unreadable_notes=()
  [[ -n "$outputs" && -f "$outputs" ]] || mutation_shard_fail "合并那一步没给在的各片产物表（TASK_SERVER_SHARD_OUTPUTS=${outputs:-没设}）" "这一步只由 task-server 起"
  scratch="$(mktemp -d)" || mutation_shard_fail "建不了放 row-plan 输出的临时目录" "看 \$TMPDIR 满没满、有没有写权限"
  while IFS=$'\t' read -r index host directory; do
    [[ -n "$directory" && -s "$directory/judged-rows" ]] || continue
    if ! python3 "$rows_module" row-plan "$tree_root" --rows "$directory/judged-rows" > "$scratch/row-plan-$index.tsv" 2> "$scratch/row-plan-$index.err"; then
      problems+=("第 $index 片（$host）：crates-mutation-rows.py row-plan 退非 0（$(tail -n 1 "$scratch/row-plan-$index.err")）")
      continue
    fi
    counts="$(mutation_shard_rows_in_service "$scratch/row-plan-$index.tsv" "$directory/judged-rows" "$scratch/unreadable-$index")" \
      || { problems+=("第 $index 片（$host）：核读得到读不到的那一段 python 没跑成"); continue; }
    read -r readable unreadable <<< "$counts"
    merged=$((merged + 1))
    total_readable=$((total_readable + readable)); total_unreadable=$((total_unreadable + unreadable))
    if (( readable == 0 && unreadable > 0 )); then
      problems+=("第 $index 片（$host）判的 $unreadable 行按此刻的行键在 prune-server 里一行都读不到（它写进了别的库：看那一片连的是不是同一份服务、被判的是不是同一棵树）")
    elif (( unreadable > 0 )); then
      unreadable_notes+=("第 $index 片（$host）读不到的 $unreadable 行：$(tr '\n' ' ' < "$scratch/unreadable-$index")")
    fi
  done < "$outputs"
  rm -rf -- "${scratch:?}"
  (( ${#problems[@]} == 0 )) || mutation_shard_fail "$(IFS='；'; echo "${problems[*]}")" "照括号里点名的那一片查它写进了哪份库；row-plan 退非 0 的单跑 python3 scripts/mutation/crates-mutation-rows.py row-plan <树根> --rows <那一片的 judged-rows> 看它报什么"
  (( ${#unreadable_notes[@]} == 0 )) || echo "  · 合并：$(IFS='；'; echo "${unreadable_notes[*]}")（没判成或不算数的结局，最后那一趟照跑）"
  echo "  ✓ 双机分片（task-server 拆的）合并：$merged 片判的 $((total_readable + total_unreadable)) 行里 $total_readable 行按此刻的行键在 prune-server 里读得到、$total_unreadable 行读不到；crates-mutation-replay 最后那一趟整张判"
}

mutation_shard_main() {
  if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
    echo "  ✗ 分片驱动在选行模式下被调了（GATE_MUTATION_ROW_SELECTION 设着）：各片起的 crates-mutation-replay 不许再调驱动"
    echo "     → 怎么办：驱动只由 task-server 按分片协议起；选行模式下 crates-mutation-replay 自己不调，这里被调说明调用方写错了"
    exit 2
  fi
  tree_root="${1:-}"
  only_rows_file=""
  if (( $# >= 1 )); then shift; fi
  while (( $# )); do
    case "$1" in
      --only-rows)
        if (( $# < 2 )) || [[ ! -s "$2" ]]; then
          echo "  ✗ --only-rows 后面要给一份非空的选行文件（一行一个变异表行号）：读到「${2:-}」"
          echo "     → 怎么办：写成 task-server/task submit -- bash scripts/mutation/mutation-shard-run.sh . --only-rows <选行文件>"
          exit 2
        fi
        only_rows_file="$2"; shift 2 ;;
      *)
        echo "  ✗ 认不出的参数 $1"
        echo "     → 怎么办：只认一个树根，后面可跟 --only-rows <选行文件>；或单独的 --selftest"
        exit 2 ;;
    esac
  done
  if [[ -z "$tree_root" || ! -d "$tree_root" ]]; then
    echo "  ✗ 用法：mutation-shard-run.sh <树根> [--only-rows <选行文件>]，或 --selftest"
    echo "     → 怎么办：在仓根经 task-server 提交：task-server/task submit -- bash scripts/mutation/mutation-shard-run.sh . [--only-rows <选行文件>]"
    exit 2
  fi
  tree_root="$(cd "$tree_root" && pwd)"
  stage_repository="$(cd "$mutation_shard_script_directory/../.." && pwd)"
  stage_script="$stage_repository/.claude/gate.d/crates-mutation-replay.sh"
  rows_module="$stage_repository/scripts/mutation/crates-mutation-rows.py"
  # 只跑服务拆的那一步（文件头「三步」）；不是服务按分片协议起的一概不分
  case "${TASK_SERVER_SHARD_PHASE:-}" in
    plan) mutation_shard_task_server_plan; exit $? ;;
    shard) mutation_shard_task_server_shard; exit $? ;;
    merge) mutation_shard_task_server_merge; exit $? ;;
    "")
      echo "  ✗ mutation-shard-run.sh 只由 task-server 按分片协议起（环境里要有 TASK_SERVER_SHARD_PHASE）：它不自己连第二台、不给哪一片套内存包装，这一次什么都没分（退 2）"
      echo "     → 怎么办：在仓根经 task-server 提交：task-server/task submit -- bash scripts/mutation/mutation-shard-run.sh . [--only-rows <选行文件>]（树根写 . 才派得到第二台）；服务拆成计划、各片、合并几步，之后照常经 task-server 跑门禁 crates-mutation-replay 整张判"
      exit 2 ;;
    *)
      echo "  ✗ TASK_SERVER_SHARD_PHASE=$TASK_SERVER_SHARD_PHASE 认不出（只认 plan、shard、merge）"
      echo "     → 怎么办：这个变量只由 task-server 设；去掉它再跑"
      exit 2 ;;
  esac
}

# ── 自证 ──────────────────────────────────────────────────────────────────────
# 在临时目录里搭一棵小仓（本仓的 .claude/scripts、crates-mutation-replay 与登记表、scripts 整个目录、.claude/hooks/lib_work_root.py（判法单元的闭包够到它）、规范副本的符号链接，加一个 crates/tiny 与六行变异表），
# 配置写在临时目录里（SINGLEFS_MULTI_HOST_CONFIG），「第二台」是本机上的另一个目录（SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1）。
# 假 cargo（往自己目录下的 invocations.log 记起了哪几条）照 lib.rs 里「// TEST <名> NEEDS <原文>」判 ok / FAILED。逐格：
#   单机那一趟：配置没开双机：crates-mutation-replay 判绿、不调驱动（它的 stdout 是 ㉔ 比对的基准）
#   ⑦ 选行模式下调驱动：退 2 拒
#   ⑨ 弄坏开关设在自证之外（起时去掉自证的标记）：驱动拒、退 2
#   ㉔ 经 task-server 分片的三步：开着双机的场景里照 task-server 设的变量逐步起计划、两片、合并，计划写两片、都不带内存下限，
#      两片判的记录都写进场景那一份库（服务里同一份，不拷库目录）、各片的产物目录里有 judged-rows，合并按此刻的行键在服务里读得到判的每一行，
#      之后门禁整张判全部复用、判定与单机逐字相同；不是服务按分片协议起的（没有 TASK_SERVER_SHARD_PHASE）退 2、提示经 task-server 提交
#   ㉕ 合并只认服务里读得到的：两片起之前就合并，另一片的 judged-rows 点名的行还没判进服务（同「写进了别的库」）⇒ 合并退 1、点名那一片（弄坏开关 merge-rows-unchecked）
#   ㉒ 计划那一步带 --only-rows 点名一行：row-plan 只算那一行的键（弄坏开关 row-plan-whole-table、only-rows-ignored），计划只分那一行；
#      直接起 row-plan 点名两行算两行、别的行键写 -、不点名算整张 6 行（弄坏开关 row-plan-selection-ignored）
#   ⑭ ③ 分行（mutation_shard_split_plan）喂一份写好的 row-plan 输出：锚点失效的两行（一行没键、一行有键）与拒算、复用的不进两堆，汇总单报锚点失效 2 行
#   ⑱ ③ 分行喂 --only-rows 的选行文件：只在选中的行里分；选中的要现判的有 1 行也分，两堆用时相等放第二台、本机那一堆空着；有 2 行时第二台拿用时长的那一行
#   ⑫ 场景都跑在 TMPDIR=<自证的临时目录>/tmp 下（command-safety.md「测试镜像一律放临时目录」）：本机那一侧的假 cargo（往自己目录下的 working-directories.log 记工作目录）
#      在 singlefs-mutation-replay-* 里起的每一次都在自证自己的 tmp/ 下、至少一次；跑完外面的临时目录里没多出 singlefs-mutation-replay-*，
#      「外面的临时目录」是自证在调用方的 TMPDIR 下另建的一层（自证的临时目录就在它里面）：同一个 TMPDIR 下并行的几份自证互不数到对方建的目录，
#      自证自己的 tmp/ 下也一个不剩（删不删由 crates-mutation-rows.py 的 run_pool 管）
# 弄坏开关各让对应的格判错（见文件头「弄坏开关」）。
# 场景里的门禁经 prune-server 写判定库：只在临时服务上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service），不是就退 2、一格都不跑，不写真服务。
mutation_shard_selftest() {
  local repository_root work template checked=0 failures=0 service_problem
  if ! service_problem="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem())' "$mutation_shard_script_directory/../prune-server" 2>&1)" || [[ -n "$service_problem" ]]; then
    echo "  ✗ mutation-shard-run.sh 自证没跑：${service_problem:-读不了 scripts/prune-server/prune_server_client.py}"
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）"
    return 2
  fi
  repository_root="$(cd "$mutation_shard_script_directory/../.." && pwd)"
  # 自证自己的外层临时根：调用方的 TMPDIR 下另建一层，自证的工作目录与 ⑫ 比的「外面的临时目录」都在它里面。
  #   同一个 TMPDIR 下并行起的几份自证（弄坏开关 selftest-tmpdir-outside 下那一份把场景建在外面）互不数到对方建的 singlefs-mutation-replay-*
  local own_temporary
  own_temporary="$(mktemp -d)"
  export TMPDIR="$own_temporary"
  work="$(mktemp -d)"
  mutation_shard_selftest_work="$own_temporary"
  # 自证起的驱动认弄坏开关的标记：场景里起的都带着它
  printf '%s\n' "$MUTATION_SHARD_SELFTEST_MARK_CONTENT" > "$work/selftest-mark"
  export MUTATION_SHARD_RUN_SELFTEST_MARK="$work/selftest-mark"
  # 配置判法与 peer-host-lib.sh 只在调用方自证的标记在时认只供测试的 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1（peer_host_inside_selftest）
  printf '%s\n' "$PEER_HOST_SELFTEST_MARK_CONTENT" > "$work/peer-host-selftest-mark"
  export PEER_HOST_SELFTEST_MARK="$work/peer-host-selftest-mark"
  # 复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：场景里 crates-mutation-replay 每一趟都抽；这里比的是复用与现跑的条数，
  #   键相同的全当哨兵、不抽强制现判（哨兵数比行数多），哨兵与强制现判本身在 crates-mutation-rows.py 的自证 reuse-sentinel 那一格判
  export SINGLEFS_REUSE_SENTINEL_COUNT=1000
  unset SINGLEFS_REUSE_SENTINEL_SEED
  trap 'echo "  · 自证的临时目录（$(du -sh "$mutation_shard_selftest_work" 2>/dev/null | cut -f1)）删掉"; rm -rf -- "${mutation_shard_selftest_work:?}"' EXIT
  # 场景起的 crates-mutation-replay 与它的工作进程都照 TMPDIR 建临时目录：换成自证自己目录下的 tmp/，跑完随自证的临时目录一起删；
  # 开跑前先记下外面那个临时目录里已有的 singlefs-mutation-replay-*，⑫ 比的是自证跑完多出来的
  local outside_temporary="${TMPDIR:-/tmp}" outside_before
  outside_before="$(compgen -G "$outside_temporary/singlefs-mutation-replay-*" | sort || true)"
  mkdir -p "$work/tmp"
  if ! mutation_shard_break_is selftest-tmpdir-outside; then
    export TMPDIR="$work/tmp"
  fi
  expect() { # expect <格名> <判据的退出码：0 为过> <不过时的细节>
    checked=$((checked + 1))
    if [[ "$2" == 0 ]]; then
      echo "  ✓ $1"
    else
      echo "  ✗ $1：$3"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  }

  # 小仓的模板
  template="$work/template"
  mkdir -p "$template/.claude/gate.d" "$template/crates/tiny/src" "$template/research"
  rsync -a --exclude __pycache__ "$repository_root/.claude/scripts/" "$template/.claude/scripts/"
  cp -- "$repository_root/.claude/gate.d/crates-mutation-replay.sh" "$repository_root/.claude/gate.d/stage-inputs.tsv" "$template/.claude/gate.d/"
  cp -- "$repository_root/.claude/heavy-thread-default" "$template/.claude/"   # 算闭包开几个子进程照每台默认重活线程数的登记（scripts/hosts/heavy_threads.py）
  rsync -a --exclude __pycache__ "$repository_root/.claude/gate.d/lib/" "$template/.claude/gate.d/lib/"   # crates-mutation-replay source 的格共用库
  rsync -a --exclude __pycache__ --exclude /experiment-apparatus/target/ "$repository_root/scripts/" "$template/scripts/"   # 装置的编译目录几十 G，自证用不到
  ln -s "$repository_root/.claude/singlefs-ai-sop" "$template/.claude/singlefs-ai-sop"
  # 判法单元的闭包（crates-mutation-rows.py 文件头「判法单元」）够得到 .claude/hooks/lib_work_root.py（memory_peaks_lib.py 的自证按路径载入它）：场景小仓里没有它，闭包记拒算 spec-path-missing，
  #   算不出判法单元指纹，计划那一步与门禁整张判都退 1。只拷这一份，不拷整个 .claude/hooks/
  mkdir -p "$template/.claude/hooks"
  cp -- "$repository_root/.claude/hooks/lib_work_root.py" "$template/.claude/hooks/"
  printf 'target/\n__pycache__/\n/multi-host.env\n/.claude/singlefs-ai-sop\n' > "$template/.gitignore"
  printf '[workspace]\nmembers = ["crates/tiny"]\n' > "$template/Cargo.toml"
  printf '[package]\nname = "tiny"\nversion = "0.1.0"\n' > "$template/crates/tiny/Cargo.toml"
  cat > "$template/crates/tiny/src/lib.rs" <<'SAMPLE_SOURCE'
// 双机分片驱动自证的样本 crate（假 cargo 按下面几行判；下面的 #[test] 函数只给行键认点名的测试是哪一个单元）
// TEST adds NEEDS left + right
// TEST doubles NEEDS value * 2
// TEST negates NEEDS 0 - value
// TEST halves NEEDS value / 2
// TEST triples NEEDS value * 3
// TEST squares NEEDS value * value
pub fn add(left: u32, right: u32) -> u32 { left + right }
pub fn double(value: u32) -> u32 { value * 2 }
pub fn negate(value: i32) -> i32 { 0 - value }
pub fn halve(value: u32) -> u32 { value / 2 }
pub fn triple(value: u32) -> u32 { value * 3 }
pub fn square(value: u32) -> u32 { value * value }

#[cfg(test)]
mod tests {
    #[test]
    fn adds() { assert_eq!(super::add(2, 3), 5); }
    #[test]
    fn doubles() { assert_eq!(super::double(3), 6); }
    #[test]
    fn negates() { assert_eq!(super::negate(3), -3); }
    #[test]
    fn halves() { assert_eq!(super::halve(6), 3); }
    #[test]
    fn triples() { assert_eq!(super::triple(2), 6); }
    #[test]
    fn squares() { assert_eq!(super::square(3), 9); }
}
SAMPLE_SOURCE
  {
    printf '# 每条变异的内存上限：256M\n# 每条变异的超时秒数：120\n# 变异名\t文件\t原文\t替换文\tcargo test 参数\t必须红的测试名\n'
    printf 'add-becomes-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- adds\tadds\n'
    printf 'double-becomes-quintuple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 5 }\t-p tiny --lib -- doubles\tdoubles\n'
    printf 'negate-becomes-identity\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ value }\t-p tiny --lib -- negates\tnegates\n'
    printf 'halve-becomes-third\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t-p tiny --lib -- halves\thalves\n'
    printf 'triple-becomes-add\tcrates/tiny/src/lib.rs\t{ value * 3 }\t{ value + 3 }\t-p tiny --lib -- triples\ttriples\n'
    printf 'square-becomes-double\tcrates/tiny/src/lib.rs\t{ value * value }\t{ value + value }\t-p tiny --lib -- squares\tsquares\n'
  } > "$template/crates/mutations.tsv"
  mkdir -p "$work/fake"
  cat > "$work/fake/cargo" <<'FAKE_CARGO'
#!/usr/bin/env python3
# rocksdb-prebuilt-entry: selftest-fake-cargo（起 cargo 之前那几道 rocksdb-prebuilt.py entry 认这一行、不判）
# 双机分片驱动自证的假 cargo：-V 打版本；test 照 crates/tiny/src/lib.rs 里「// TEST <名> NEEDS <原文>」判，原文在就 ok、不在就 FAILED；
# 每起一次往自己目录下的 invocations.log 记「<过滤词>\t<lib.rs 的 sha256 前 12 位>」。
# 记覆盖那两次（照 crates-mutation-rows.py 自证的假 cargo）：开头的 --config <值> 跳过；test --no-run 在 $CARGO_TARGET_DIR/debug/deps/ 下写假的可执行文件
# （JSON：副本的根与 crates/tiny/src 下的 .rs）与它的 dep-info、打一行 compiler-artifact；设了 LLVM_PROFILE_FILE 的把跑到的用例执行的函数
# （定义行里有那段原文的）写进那份 .profraw；假 llvm 工具是 crates-mutation-rows.py 的 FAKE_LLVM_TOOL
import hashlib, json, os, re, sys
here = os.path.dirname(os.path.abspath(__file__))
arguments = sys.argv[1:]
while arguments[:1] == ["--config"]:
    arguments = arguments[2:]
if arguments[:1] in (["-V"], ["--version"]):
    print("cargo 0.0.0 (mutation-shard-run selftest)")
    sys.exit(0)
if arguments[:1] != ["test"]:
    sys.exit(2)
# 自证 ⑫：每起一次 test 往自己目录下的 working-directories.log 记它的工作目录（经 crates-mutation-rows.py 起的在它 $TMPDIR 下的 singlefs-mutation-replay-* 里）
with open(os.path.join(here, "working-directories.log"), "a", encoding="utf-8") as seen:
    seen.write(os.getcwd() + "\n")
source = open("crates/tiny/src/lib.rs", encoding="utf-8").read()
if "--no-run" in arguments:
    deps = os.path.join(os.environ["CARGO_TARGET_DIR"], "debug", "deps")
    os.makedirs(deps, exist_ok=True)
    executable = os.path.join(deps, "tiny-0000000000000000")
    with open(executable, "w", encoding="utf-8") as handle:
        json.dump({"root": os.getcwd(), "sources": ["crates/tiny/src/lib.rs"]}, handle)
    with open(executable + ".d", "w", encoding="utf-8") as handle:
        handle.write(f"{executable}: crates/tiny/src/lib.rs\n\n")
    print(json.dumps({"reason": "compiler-artifact", "manifest_path": os.path.join(os.getcwd(), "crates", "tiny", "Cargo.toml"),
                      "target": {"kind": ["lib"], "name": "tiny"}, "profile": {"test": True}, "filenames": [executable], "executable": executable}))
    sys.exit(0)
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
filters = [word for word in after if not word.startswith("-")]
with open(os.path.join(here, "invocations.log"), "a", encoding="utf-8") as log:
    log.write(" ".join(filters) + "\t" + hashlib.sha256(source.encode()).hexdigest()[:12] + "\n")
failed, executed = False, set()
definitions = re.findall(r"(?m)^pub fn (\w+).*$", source)
lines = source.splitlines()
for line in lines:
    words = line.split(" ", 4)
    if len(words) < 5 or words[:2] != ["//", "TEST"]:
        continue
    name, what = words[2], words[4]
    if filters and not any(word in "tests::" + name for word in filters):
        continue
    executed |= {match.group(1) for definition in lines for match in [re.match(r"^pub fn (\w+)", definition)] if match and what in definition}
    ok = what in source.replace(line, "")
    failed = failed or not ok
    print(f"test tests::{name} ... {'ok' if ok else 'FAILED'}")
if os.environ.get("LLVM_PROFILE_FILE"):
    with open(os.environ["LLVM_PROFILE_FILE"].replace("%p", str(os.getpid())).replace("%m", "0"), "w", encoding="utf-8") as handle:
        json.dump({"executed": sorted(executed)}, handle)
sys.exit(101 if failed else 0)
FAKE_CARGO
  chmod +x "$work/fake/cargo"
  # 假 llvm 工具：取 crates-mutation-rows.py 里那一份（FAKE_LLVM_TOOL），两个名字各一份
  mkdir -p "$work/llvm"
  python3 - "$repository_root/scripts/mutation/crates-mutation-rows.py" "$work/llvm" <<'FAKE_LLVM' || { echo "  ✗ 写不出假 llvm 工具"; echo "     → 怎么办：单跑 python3 scripts/mutation/crates-mutation-rows.py --selftest 看共用模块读不读得进来"; return 1; }
import importlib.util, os, sys
specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_shard_selftest", sys.argv[1])
module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(module)
for tool in module.LLVM_TOOLS:
    path = os.path.join(sys.argv[2], tool)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(module.FAKE_LLVM_TOOL)
    os.chmod(path, 0o755)
FAKE_LLVM
  local own_repository_sample_line
  own_repository_sample_line="$(sed -n 's/^STAGE_CELLS_OWN_REPOSITORY_SAMPLE_LINE="\(.*\)"$/\1/p' "$template/.claude/gate.d/lib/stage-cells.sh")"
  if [[ -z "$own_repository_sample_line" ]]; then
    echo "  ✗ 自证读不到 .claude/gate.d/lib/stage-cells.sh 的 STAGE_CELLS_OWN_REPOSITORY_SAMPLE_LINE（场景小仓的 .gate-break-sample 第一行要写它）"
    echo "     → 怎么办：看那份共用库里这一行还在不在、是不是还写成 STAGE_CELLS_OWN_REPOSITORY_SAMPLE_LINE=\"…\" 一整行；改了名字就把这里的 sed 一起改"
    return 1
  fi

  # 每一格一个场景目录：repo（被判的树，git init）、local-bin 与 peer-bin（两台 PATH 上的假 cargo）、local-home 与 peer-home（配置里两台在 BUILD_ROOT_BY_HOST 里的那一项）、配置
  prepare_scene() { # <场景名> [off]：off 时配置里写 ENABLE_ACROSS_MACHINES=0（判法退 3，双机：关）
    local scene="$work/$1" across_machines_switch=1
    [[ "${2:-}" == off ]] && across_machines_switch=0
    mkdir -p "$scene/local-bin" "$scene/peer-bin" "$scene/local-home" "$scene/peer-home" "$scene/shared-root"
    rsync -a "$template/" "$scene/repo/"
    git -C "$scene/repo" init -q
    # 门禁共用库认这棵小仓是自证搭的（.claude/gate.d/lib/stage-cells.sh「弄坏开关只在样本根上认」）：场景里那一道就在这棵树里，
    #   .gate-break-sample 第一行写那一句接这棵树的 pwd -P，弄坏开关才进得了经 crates-mutation-replay 跑的那几格
    printf '%s%s\n' "$own_repository_sample_line" "$(cd "$scene/repo" && pwd -P)" > "$scene/repo/.gate-break-sample"
    cp -- "$work/fake/cargo" "$scene/local-bin/cargo"
    cp -- "$work/fake/cargo" "$scene/peer-bin/cargo"
    {
      echo "HOSTS=127.0.0.1 192.0.2.64"   # 主机数组（判法文件头「主机数组」）：127.0.0.1 在每一台的网卡上是自己，192.0.2.64（文档用的地址段）是第二台
      # 按台的三个键（判法文件头「按台的三个键」）两台各一项；门禁的并发数读这一台那一项的 slice 上限，写大些不压并发
      echo "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=$scene/local-bin 192.0.2.64=$scene/peer-bin"
      echo "BUILD_ROOT_BY_HOST=127.0.0.1=$scene/local-home 192.0.2.64=$scene/peer-home"
      echo "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=50G 192.0.2.64=50G"
      echo "ENABLE_ACROSS_MACHINES=$across_machines_switch"
      echo "ENABLE_GPU=0"   # 两个开关都要明确写（配置判法不兜底）
      # 配置判法把它当必写的键（每段要是本机已在的目录）；crates-mutation-rows.py 的用时表与占盘记录放它点名的、不含被判的树也不在树里的头一段
      #   （run_records_directory）：写场景自己的 shared-root（上面 mkdir 过），不写含着 repo 的场景目录，不碰真共享根
      echo "NFS_SHARED_ROOTS=$scene/shared-root"
      echo "QUIESCE_STOP_COMMAND=touch $scene/quiesce-marker-selftest"
      echo "QUIESCE_STOPPED_CHECK_COMMAND=test -e $scene/quiesce-marker-selftest"
      echo "QUIESCE_START_COMMAND=rm -f $scene/quiesce-marker-selftest"
      echo "QUIESCE_STARTED_CHECK_COMMAND=test ! -e $scene/quiesce-marker-selftest"
    } > "$scene/multi-host.env"
  }
  scene_command_for() { # <场景名>：scene_command 设成在这个场景的环境里起命令的 env 前缀（PATH 先找假 cargo；crates-mutation-replay 读的变量从调用方的环境里清掉，弄坏开关留着）
    local scene="$work/$1"
    scene_command=(env -u GATE_MUTATION_ROW_SELECTION -u SINGLEFS_REUSE_HOURS -u GATE_MUTATION_WORKERS
      -u RUN_WITH_MEMORY_CAP_RESERVE
      -u GPU_QUIESCE_ORIGIN_READ_BACK -u GPU_QUIESCE_HOLDER
      PATH="$scene/local-bin:$PATH" SINGLEFS_MULTI_HOST_CONFIG="$scene/multi-host.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1
      GATE_MUTATION_TARGET_DIR="$scene/local-target" RUN_WITH_MEMORY_CAP_PEAKS="$scene/local-peaks.tsv"
      GATE_MUTATION_LLVM_TOOLS="$work/llvm")
  }
  scene_environment() { # <场景名> <命令…>：在这个场景的环境里跑命令
    scene_command_for "$1"
    shift
    "${scene_command[@]}" "$@"
  }
  run_stage_in() { # <场景名> [变量=值…]：在这个场景的 repo 上跑 crates-mutation-replay，stdout、stderr、退出码各写场景目录一个文件
    local scene="$work/$1" exit_code
    shift
    if scene_environment "${scene##*/}" env "$@" bash "$scene/repo/.claude/gate.d/crates-mutation-replay.sh" "$scene/repo" > "$scene/stdout" 2> "$scene/stderr"; then
      exit_code=0
    else
      exit_code=$?
    fi
    echo "$exit_code" > "$scene/exit"
  }
  verdict_view() { # <stdout 文件>：去掉随记录变的几处（「复用：」「覆盖：」两行、抓到那一行末尾的栏、成功句末尾「；这一趟现判 …」），剩下的是判定；
    #   场景小仓的路径（stdout 文件所在的场景目录下的 repo，照写的与 pwd -P 两种）换成 <场景的小仓>：设了弄坏开关时门禁共用库 .claude/gate.d/lib/stage-cells.sh
    #   往 stdout 打一行「<被判的根> 是自证搭的小仓…认环境里的弄坏开关：…」，各场景的根不同，不换掉的话每条开关下比判定的几格都因这一行不同而红
    local scene_repository scene_repository_physical
    scene_repository="$(dirname "$1")/repo"
    scene_repository_physical="$(cd "$scene_repository" 2>/dev/null && pwd -P)"
    sed -E 's/（现判：[^）]*）$|（复用）$|（拒算）$//; s/；这一趟现判 .*$//' "$1" | grep -v -e '^  复用：' -e '^  覆盖：' \
      | VERDICT_VIEW_ROOT="$scene_repository" VERDICT_VIEW_ROOT_PHYSICAL="$scene_repository_physical" awk '
          BEGIN { roots[1] = ENVIRON["VERDICT_VIEW_ROOT"]; roots[2] = ENVIRON["VERDICT_VIEW_ROOT_PHYSICAL"] }
          {
            for (each = 1; each <= 2; each++) {
              if (roots[each] == "") continue
              while ((at = index($0, roots[each])) > 0) $0 = substr($0, 1, at - 1) "<场景的小仓>" substr($0, at + length(roots[each]))
            }
            print
          }'
  }

  # 单机那一趟：配置没开双机（ENABLE_ACROSS_MACHINES=0），作为 ㉔ 逐字比对的基准；门禁不调分片驱动
  prepare_scene single off
  run_stage_in single
  expect "单机那一趟（配置没开双机）判绿、门禁不调分片驱动" \
    "$([[ "$(cat "$work/single/exit")" == 0 ]] && ! grep -q '双机分片：' "$work/single/stderr"; echo $?)" \
    "退 $(cat "$work/single/exit")；stderr 尾部：$(tail -3 "$work/single/stderr" | tr '\n' ' ')；stdout 尾部：$(tail -3 "$work/single/stdout" | cut -c1-300 | tr '\n' ' ')"

  # ⑦ 选行模式下调驱动
  local selection_exit
  if scene_environment single env GATE_MUTATION_ROW_SELECTION="$work/single/stdout" bash "$work/single/repo/scripts/mutation/mutation-shard-run.sh" "$work/single/repo" > "$work/single/driver.out" 2>&1; then
    selection_exit=0
  else
    selection_exit=$?
  fi
  expect "⑦ 选行模式下调驱动：退 2 拒" \
    "$([[ "$selection_exit" == 2 ]] && grep -q '选行模式下被调了' "$work/single/driver.out"; echo $?)" "退 $selection_exit"

  # ⑨ 弄坏开关设在自证之外（起时去掉自证的标记，弄坏开关照调用方的再加一项）：驱动拒、退 2
  local outside_driver_exit
  if scene_environment single env -u MUTATION_SHARD_RUN_SELFTEST_MARK MUTATION_SHARD_RUN_BREAK="${MUTATION_SHARD_RUN_BREAK:+$MUTATION_SHARD_RUN_BREAK,}anchor-broken-dispatched" \
       bash "$work/single/repo/scripts/mutation/mutation-shard-run.sh" "$work/single/repo" > "$work/single/outside-driver.out" 2>&1; then
    outside_driver_exit=0
  else
    outside_driver_exit=$?
  fi
  expect "⑨ MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外（不带自证的标记）：驱动拒、退 2，说设在 --selftest 之外" \
    "$([[ "$outside_driver_exit" == 2 ]] && grep -q '设在 --selftest 之外' "$work/single/outside-driver.out"; echo $?)" \
    "退 $outside_driver_exit；输出：$(tail -3 "$work/single/outside-driver.out" | tr '\n' ' ')"

  # ㉔ 经 task-server 分片的三步（文件头「三步」）：在一个开着双机的场景里照 task-server 设的变量逐步起——计划写两片（本机那一堆钉在本机、另一堆不钉，
  #   两堆不相交、合起来是全部 6 行，都不带 memory_floor）；两片都判进场景那一份库（服务里同一份），各写 judged-rows；合并按此刻的行键核两片判的行在服务里读得到；
  #   之后门禁整张判全部复用、判定与单机那一趟逐字相同。另核不是服务按分片协议起的（没有 TASK_SERVER_SHARD_PHASE）：退 2、提示经 task-server 提交
  prepare_scene steps
  local steps="$work/steps" steps_plan_exit steps_merge_exit steps_direct_exit step_index steps_shard_exits="" steps_piles steps_plan_view
  mkdir -p "$steps/plan"
  if scene_environment steps env TASK_SERVER_TASK=selftest-plan TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" \
       bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/plan.out" 2>&1; then
    steps_plan_exit=0
  else
    steps_plan_exit=$?
  fi
  # ㉒ 计划那一步带 --only-rows 点名一行（在两片与合并之前，那一行还要现判）：row-plan 只对那一行算键（row-plan 自己打的「算了 N 行的键」），计划只分那一行
  local steps_only_rows="$steps/only-one.selection" steps_only_plan_exit steps_only_piles
  mkdir -p "$steps/plan-only"
  printf '7\n' > "$steps_only_rows"   # 变异表前三行是表头注释：第 7 行 halve-becomes-third
  if scene_environment steps env TASK_SERVER_TASK=selftest-plan-only TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan-only" \
       bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" --only-rows "$steps_only_rows" > "$steps/plan-only.out" 2>&1; then
    steps_only_plan_exit=0
  else
    steps_only_plan_exit=$?
  fi
  steps_only_piles="$(cat "$steps/plan-only"/shard-*.selection 2>/dev/null | tr '\n' ' ')"
  expect "㉒ 计划那一步带 --only-rows 点名一行：row-plan 只对那一行算键（算键 1 行，不是整张表的 6 行），计划只分那一行" \
    "$([[ "$steps_only_plan_exit" == 0 && "$steps_only_piles" == "7 " ]] && grep -q 'row-plan：算了 1 行的键（表里成形的 6 行' "$steps/plan-only.out"; echo $?)" \
    "计划那一步退 $steps_only_plan_exit；两堆里的行「$steps_only_piles」；row-plan 那一行：$(grep 'row-plan：' "$steps/plan-only.out" | tail -1 | cut -c1-120)；计划那一步输出尾部：$(tail -3 "$steps/plan-only.out" | cut -c1-300 | tr '\n' ' ')"
  steps_plan_view="$(python3 -c 'import json, sys; plan = json.load(open(sys.argv[1], encoding="utf-8")); print(" ".join(("pin=" + shard["pin"] if "pin" in shard else "unpinned") + ":" + str("memory_floor" in shard) for shard in plan["shards"]))' "$steps/plan/plan.json" 2>&1)"
  steps_piles="$(cat "$steps/plan/shard-0.selection" "$steps/plan/shard-1.selection" 2>/dev/null | grep -c '[0-9]') $(cat "$steps/plan/shard-0.selection" "$steps/plan/shard-1.selection" 2>/dev/null | sort -u | grep -c '[0-9]')"
  # ㉕ 合并只认服务里读得到的（文件头「三步」merge）：两片起之前就合并，另一片的 judged-rows 点名的行还没判进服务（与它写进了别的库同形）⇒ 合并退 1、点名那一片
  local steps_lost_exit
  mkdir -p "$steps/output-lost"
  cp -- "$steps/plan/shard-1.selection" "$steps/output-lost/judged-rows" 2>/dev/null
  printf '1\tpeer\t%s\n' "$steps/output-lost" > "$steps/outputs-lost.tsv"
  if scene_environment steps env TASK_SERVER_TASK=selftest-merge-lost TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" TASK_SERVER_SHARD_OUTPUTS="$steps/outputs-lost.tsv" \
       bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/merge-lost.out" 2>&1; then
    steps_lost_exit=0
  else
    steps_lost_exit=$?
  fi
  expect "㉕ 合并：另一片判的行按此刻的行键在服务里一行都读不到（写进了别的库）⇒ 退 1、点名那一片" \
    "$([[ "$steps_lost_exit" == 1 && -s "$steps/output-lost/judged-rows" ]] && grep -q '第 1 片（peer）判的 [1-9][0-9]* 行按此刻的行键在 prune-server 里一行都读不到' "$steps/merge-lost.out"; echo $?)" \
    "合并退 $steps_lost_exit；输出尾部：$(tail -3 "$steps/merge-lost.out" | tr '\n' ' ')"
  # ㉖ 每一片在跑它的这一台现查 llvm 工具（文件头「三步」shard）：llvm 工具目录指到一个空目录，两片都退 78、一条变异都不起（没写 judged-rows），出路写装哪个包
  local missing_tools_exits="" missing_tools_problems=""
  mkdir -p "$steps/no-llvm-tools"
  for step_index in 0 1; do
    if scene_environment steps env TASK_SERVER_TASK="selftest-shard-no-llvm-$step_index" TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD="$step_index/2" GATE_MUTATION_LLVM_TOOLS="$steps/no-llvm-tools" \
         TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$steps/output-no-llvm-$step_index" \
         bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/shard-no-llvm-$step_index.out" 2>&1; then
      missing_tools_exits+="0 "
    else
      missing_tools_exits+="$? "
    fi
    [[ ! -e "$steps/output-no-llvm-$step_index/judged-rows" ]] && grep -q 'rustup component add llvm-tools' "$steps/shard-no-llvm-$step_index.out" \
      || missing_tools_problems+="第 $step_index 片写了 judged-rows 或出路里没写 rustup component add llvm-tools（$(tail -2 "$steps/shard-no-llvm-$step_index.out" | tr '\n' ' ')）；"
  done
  expect "㉖ 每一片在跑它的这一台现查 llvm 工具：缺了退 78、一条变异都不起，出路写装哪个包；两片同一条规则" \
    "$([[ "$missing_tools_exits" == "78 78 " && -z "$missing_tools_problems" ]]; echo $?)" "两片退「$missing_tools_exits」；$missing_tools_problems"
  for step_index in 0 1; do
    if scene_environment steps env TASK_SERVER_TASK="selftest-shard-$step_index" TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD="$step_index/2" \
         TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$steps/output-$step_index" \
         bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/shard-$step_index.out" 2>&1; then
      steps_shard_exits+="0 "
    else
      steps_shard_exits+="$? "
    fi
  done
  printf '0\tlocal\t%s\n1\tpeer\t%s\n' "$steps/output-0" "$steps/output-1" > "$steps/outputs.tsv"
  if scene_environment steps env TASK_SERVER_TASK=selftest-merge TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" TASK_SERVER_SHARD_OUTPUTS="$steps/outputs.tsv" \
       bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/merge.out" 2>&1; then
    steps_merge_exit=0
  else
    steps_merge_exit=$?
  fi
  run_stage_in steps
  expect "㉔ 经 task-server 分片：计划写两片（两堆都不钉、片里没有 pin 键，都不带内存下限），两堆合起来是全部 6 行、不相交" \
    "$([[ "$steps_plan_exit" == 0 && "$steps_plan_view" == "unpinned:False unpinned:False" && "$steps_piles" == "6 6" ]]; echo $?)" \
    "计划那一步退 $steps_plan_exit；plan.json：$steps_plan_view；两堆合起来的行数与去重之后的行数：$steps_piles；输出尾部：$(tail -2 "$steps/plan.out" | tr '\n' ' ')"
  expect "㉔ 经 task-server 分片：两片各退 0、各写 judged-rows（不拷库目录），合并按此刻的行键在服务里读得到两片判的每一行" \
    "$([[ "$steps_shard_exits" == "0 0 " && -s "$steps/output-0/judged-rows" && -s "$steps/output-1/judged-rows" && ! -e "$steps/output-1/library-path" && ! -d "$steps/output-1/records-home" && "$steps_merge_exit" == 0 ]] \
       && grep -q '合并：2 片判的 [1-9][0-9]* 行里 [1-9][0-9]* 行按此刻的行键在 prune-server 里读得到、0 行读不到' "$steps/merge.out"; echo $?)" \
    "两片退「$steps_shard_exits」；合并退 $steps_merge_exit；合并输出尾部：$(tail -3 "$steps/merge.out" | tr '\n' ' ')；第二片尾部：$(tail -2 "$steps/shard-1.out" | tr '\n' ' ')"
  expect "㉔ 经 task-server 分片：之后门禁整张判全部复用（复用抓到 6 条、现跑变异至多 0 条），判定与单机那一趟逐字相同" \
    "$([[ "$(cat "$steps/exit")" == 0 ]] && cmp -s <(verdict_view "$work/single/stdout") <(verdict_view "$steps/stdout")  \
       && grep -q '判定库记录：复用抓到 6 条.*现跑变异至多 0 条' "$steps/stderr"; echo $?)" \
    "退 $(cat "$steps/exit")；判定库记录行：$(grep '判定库记录：' "$steps/stderr" | tail -1)"
  if scene_environment steps env -u MUTATION_SHARD_RUN_SELFTEST_MARK -u TASK_SERVER_TASK -u MUTATION_SHARD_RUN_BREAK bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/direct.out" 2>&1; then
    steps_direct_exit=0
  else
    steps_direct_exit=$?
  fi
  expect "㉔ 不是服务按分片协议起的（没有 TASK_SERVER_SHARD_PHASE）：退 2、提示经 task-server 提交，第二台这一趟的目录没建" \
    "$([[ "$steps_direct_exit" == 2 && ! -d "$steps/peer-home/mutation-runs" ]] && grep -q 'task-server/task submit -- bash scripts/mutation/mutation-shard-run.sh' "$steps/direct.out"; echo $?)" \
    "退 $steps_direct_exit；输出：$(tail -2 "$steps/direct.out" | tr '\n' ' ')"

  # ㉒ 直接起 row-plan：点名两行只算两行、别的行键写 -，不点名算整张 6 行
  local row_plan_selection="$work/split-only/row-plan-two-rows" row_plan_selected_stderr row_plan_whole_stderr row_plan_selected_lines row_plan_keyed_lines
  mkdir -p "$work/split-only"
  printf '4\n7\n' > "$row_plan_selection"   # 变异表前三行是表头注释：第 4 行 add-becomes-sub、第 7 行 halve-becomes-third
  local row_plan_break=()
  mutation_shard_break_is row-plan-selection-ignored && row_plan_break=(GATE_MUTATION_BREAK=rowplanselectionignored)
  row_plan_selected_stderr="$(scene_environment steps env ${row_plan_break[@]+"${row_plan_break[@]}"} bash -c 'python3 "$1" row-plan "$2" --rows "$3" 2>&1 >"$4" | tail -n 1' _ \
    "$steps/repo/scripts/mutation/crates-mutation-rows.py" "$steps/repo" "$row_plan_selection" "$work/split-only/row-plan-two.tsv")"
  row_plan_whole_stderr="$(scene_environment steps bash -c 'python3 "$1" row-plan "$2" 2>&1 >"$3" | tail -n 1' _ \
    "$steps/repo/scripts/mutation/crates-mutation-rows.py" "$steps/repo" "$work/split-only/row-plan-whole.tsv")"
  row_plan_selected_lines="$(wc -l < "$work/split-only/row-plan-two.tsv")"
  row_plan_keyed_lines="$(awk -F'\t' '$2 ~ /^[0-9a-f]{64}$/ { count++ } END { print count + 0 }' "$work/split-only/row-plan-two.tsv")"
  expect "㉒ row-plan 点名两行：只对两行算键（算键 2 行），别的 4 行只给用时、键写 -；不点名算整张 6 行" \
    "$([[ "$row_plan_selected_stderr" == *'算了 2 行的键（表里成形的 6 行'* && "$row_plan_selected_lines" == 6 && "$row_plan_keyed_lines" == 2 && "$row_plan_whole_stderr" == *'算了 6 行的键'* ]]; echo $?)" \
    "点名两行那一趟：「${row_plan_selected_stderr:0:100}」（输出 $row_plan_selected_lines 行、有键的 $row_plan_keyed_lines 行：$(head -2 "$work/split-only/row-plan-two.tsv" | cut -c1-400 | tr '\n' ' ')）；整张那一趟：「${row_plan_whole_stderr:0:100}」"

  # ⑭ ③ 分行：喂一份 row-plan 的输出——两行要现判、第 3 行锚点失效没键、第 4 行锚点失效而键算得出（数据文件那一类）、第 5 行拒算、第 6 行复用：
  #    两堆合起来只有第 1、2 行，汇总报拒算 1 行、锚点失效 2 行（弄坏开关 anchor-broken-dispatched 下第 4 行进了一堆）
  mkdir -p "$work/split"
  printf '1\t%s\t0\t5.000\t-\n2\t%s\t0\t4.000\t-\n3\t-\t0\t-\tanchor-broken\n4\t%s\t0\t3.000\tanchor-broken\n5\t-\t0\t-\t-\n6\t%s\t1\t2.000\t-\n' \
    "$(printf 'row-one' | sha256sum | cut -c1-64)" "$(printf 'row-two' | sha256sum | cut -c1-64)" "$(printf 'row-four' | sha256sum | cut -c1-64)" \
    "$(printf 'row-six' | sha256sum | cut -c1-64)" > "$work/split/plan.tsv"
  local split_line split_local split_peer
  split_line="$(mutation_shard_split_plan "$work/split/plan.tsv" "$work/split/local.selection" "$work/split/peer.selection" "$work/split/keys")"
  split_local="$(tr '\n' ' ' < "$work/split/local.selection" 2>/dev/null)"
  split_peer="$(tr '\n' ' ' < "$work/split/peer.selection" 2>/dev/null)"
  expect "⑭ ③ 分行：锚点失效的行（第五列 anchor-broken，有键没键都算）不进两堆、单报几行" \
    "$([[ "$(sort -n "$work/split/local.selection" "$work/split/peer.selection" | tr '\n' ' ')" == "1 2 " && "$split_line" == "6 2 4 3.5 1 5 1 4 1 2" ]]; echo $?)" \
    "本机「$split_local」第二台「$split_peer」；汇总「$split_line」"

  # ⑱ 分行带选行文件：喂一份写好的 row-plan 输出（同 ⑭），选行文件点名第 2、6 行（第 6 行复用）与第 1、2 行（两行都要现判）
  mkdir -p "$work/split-only"
  printf '1\t%s\t0\t5.000\t-\n2\t%s\t0\t4.000\t-\n3\t-\t0\t-\tanchor-broken\n4\t%s\t0\t3.000\tanchor-broken\n5\t-\t0\t-\t-\n6\t%s\t1\t2.000\t-\n' \
    "$(printf 'row-one' | sha256sum | cut -c1-64)" "$(printf 'row-two' | sha256sum | cut -c1-64)" "$(printf 'row-four' | sha256sum | cut -c1-64)" \
    "$(printf 'row-six' | sha256sum | cut -c1-64)" > "$work/split-only/plan.tsv"
  printf '2\n6\n' > "$work/split-only/only-one"
  printf '1\n2\n' > "$work/split-only/only-two"
  local only_line_one only_line_two
  only_line_one="$(mutation_shard_split_plan "$work/split-only/plan.tsv" "$work/split-only/local-one" "$work/split-only/peer-one" "$work/split-only/keys-one" "$work/split-only/only-one")"
  only_line_two="$(mutation_shard_split_plan "$work/split-only/plan.tsv" "$work/split-only/local-two" "$work/split-only/peer-two" "$work/split-only/keys-two" "$work/split-only/only-two")"
  expect "⑱ ③ 分行带选行文件：只在选中的行里分，要现判的有 1 行也分、两堆用时相等放第二台、本机那一堆空着；有 2 行时第二台拿用时长的那一行" \
    "$([[ "$only_line_one" == "2 1 4 3.5 0 0 1 4 0 0" && -z "$(tr -d '\n' < "$work/split-only/local-one")" && "$(tr '\n' ' ' < "$work/split-only/peer-one")" == "2 " \
       && "$only_line_two" == "2 2 4 3.5 1 4 1 5 0 0" && "$(tr '\n' ' ' < "$work/split-only/local-two")" == "2 " && "$(tr '\n' ' ' < "$work/split-only/peer-two")" == "1 " ]]; echo $?)" \
    "选中第 2、6 行：汇总「$only_line_one」本机「$(tr '\n' ' ' < "$work/split-only/local-one")」第二台「$(tr '\n' ' ' < "$work/split-only/peer-one")」；选中第 1、2 行：汇总「$only_line_two」本机「$(tr '\n' ' ' < "$work/split-only/local-two")」第二台「$(tr '\n' ' ' < "$work/split-only/peer-two")」"

  # ⑫ 场景跑在自证自己的 tmp/ 下、跑完不留：本机那一侧的假 cargo 在 singlefs-mutation-replay-* 里起的每一次，工作目录都在自证自己的 tmp/ 下（至少一次）；
  #   外面的临时目录里没多出 singlefs-mutation-replay-*，自证自己的 tmp/ 下也一个不剩。前一样看的是跑的时候在哪：TMPDIR 没换过来时运行目录建在外面那一层，
  #   跑完照样删掉、后两样看不出来（弄坏开关 selftest-tmpdir-outside 单开就红在这一格）；第二台那一侧的假 cargo 照第二台的 TMPDIR，不比
  local outside_after outside_new inside_left launch_logs=() launch_counts launched_total launched_outside launched_example
  outside_after="$(compgen -G "$outside_temporary/singlefs-mutation-replay-*" | sort || true)"
  outside_new="$(comm -13 <(printf '%s\n' "$outside_before" | sed '/^$/d') <(printf '%s\n' "$outside_after" | sed '/^$/d'))"
  inside_left="$(compgen -G "$work/tmp/singlefs-mutation-replay-*" || true)"
  mapfile -t launch_logs < <(compgen -G "$work/*/local-bin/working-directories.log" || true)
  launch_counts="$(MUTATION_SHARD_OWN_RUNS_PREFIX="$(cd "$work/tmp" && pwd -P)/singlefs-mutation-replay-" awk '
      index($0, "/singlefs-mutation-replay-") { total++; if (index($0, ENVIRON["MUTATION_SHARD_OWN_RUNS_PREFIX"]) != 1) { outside++; if (example == "") example = $0 } }
      END { printf "%d\t%d\t%s\n", total, outside, example }' /dev/null ${launch_logs[@]+"${launch_logs[@]}"})"
  IFS=$'\t' read -r launched_total launched_outside launched_example <<< "$launch_counts"
  expect "⑫ 场景都跑在自证自己的 tmp/ 下：跑完外面的临时目录（$outside_temporary）里没多出 singlefs-mutation-replay-*，自己的 tmp/ 下也一个不剩" \
    "$([[ "${launched_total:-0}" -gt 0 && "${launched_outside:-0}" == 0 && -z "$outside_new" && -z "$inside_left" ]]; echo $?)" \
    "本机那一侧的假 cargo 在 singlefs-mutation-replay-* 里起了 ${launched_total:-0} 次，工作目录不在自证自己的 tmp/ 下的 ${launched_outside:-0} 次（${launched_example:-没有}）；外面多出来的「$(tr '\n' ' ' <<< "$outside_new")」；自证自己的 tmp/ 里剩下的「$(tr '\n' ' ' <<< "$inside_left")」"
  # 弄坏开关下外面多出来的，是这一趟自证自己建的：照名字逐个删，不留给别人
  local outside_entry
  while IFS= read -r outside_entry; do
    [[ -n "$outside_entry" && "$outside_entry" == "$outside_temporary"/singlefs-mutation-replay-* ]] && rm -rf -- "${outside_entry:?}"
  done <<< "$outside_new"

  if (( failures > 0 )); then
    echo "  ✗ mutation-shard-run.sh 自证有 $failures 格判错（共 $checked 格）"
    echo "     → 怎么办：设了 MUTATION_SHARD_RUN_BREAK 的，这就是要的结果（那个开关把对应的一步弄坏了）；没设的，照上面那几格去看驱动的计划、各片、合并三步"
    return 1
  fi
  echo "  ✓ mutation-shard-run.sh 自证 $checked 格都对（经 task-server 分片的计划、各片、合并三步：计划两堆不相交且不带内存下限、两片判进服务里同一份库、合并按此刻的行键核两片判的行读得到（读不到的那一片退 1）、之后门禁整张判全部复用且判定与单机逐字相同；不是服务按分片协议起的退 2、选行模式下被调退 2、弄坏开关设在自证之外退 2；--only-rows 只对选中的行算键、只在选中的行里分；锚点失效的行不分；场景跑完不留 singlefs-mutation-replay-*）"
}

if [[ "${1:-}" == --selftest ]]; then
  mutation_shard_selftest
  exit $?
fi
if [[ -n "${MUTATION_SHARD_RUN_BREAK:-}" ]] && ! mutation_shard_break_is break-honored-anywhere && ! mutation_shard_inside_selftest; then
  echo "  ✗ MUTATION_SHARD_RUN_BREAK=$MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外：弄坏开关只在自证起的进程里生效，这一次什么都没分（退 2）"
  echo "     → 怎么办：去掉它再跑（env -u MUTATION_SHARD_RUN_BREAK …）；要证红就跑 MUTATION_SHARD_RUN_BREAK=<项> bash scripts/mutation/mutation-shard-run.sh --selftest"
  exit 2
fi
mutation_shard_main "$@"
