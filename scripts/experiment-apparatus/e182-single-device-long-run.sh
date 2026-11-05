#!/usr/bin/env bash
# admission: always 每一次调都照此刻片结果库里哪几片做完了判还差什么，上一次跑完不替这一次作保
# run-condition: command cargo git
# task-server-shards: *
# E182（被抛弃根读不出的三条岔路）第一段（单盘择根段，跑前登记 research/prompts/e182-preregistration.md 5.4 与 5.5）的长活跑法：
#   装置是入库装置 crates/singlefs-verification/src/bin/e182_unreadable_abandoned_root_forks.rs（bin e182_unreadable_abandoned_root_forks），
#   按「一条历史 × 它录制流里的一个段」切片判，片结果按复用键存进片结果库（下面「片结果库」），片齐了由装置自己合并计数、打产物
#   research/results/e182-unreadable-abandoned-root-forks-single-device.out。
#
# long-run-shards: 一片 = 一条历史 × 它录制流里的一个段，另加整条流末尾那个全持久的状态为一片；历史 18 条（G默认四个起点与 G窄两个起点，各三族），片数由段数定，片之间互不依赖，装置 list-items 列历史名、片号按历史次序与段号排下来
# long-run-resume: 同一条命令重起就是续跑：每片判完原子写一个以复用键命名的文件进这一趟的中转目录，本脚本每 MERGE_INTERVAL_SECONDS 秒与装置退出时把它并回 prune-server 里的片结果库，重起先从服务拉库，键相同的片直接取、只补没判的，成功那句报「现判 N、复用 M」；复用键 = 判法入口按 singlefs-unit-graph 算的函数单元闭包指纹（去注释）加工具链行、历史名、段号、展开方式
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
#   多机的读法：本脚本不自己连第二台；经 task-server 分片（task-server/task_server_lib.py 文件头「分片」；文件头那一行 task-server-shards 声明了，
#   提交 task-server/task submit -- bash scripts/experiment-apparatus/e182-single-device-long-run.sh，派哪台由服务定，两台在同一棵共享的树里跑）：
#     plan  在本机：plan.json 写台数那么多片（台数 = 1 + --emit-hosts 打的 MULTI_HOST_COUNT，双机关着它退 3、只有本机；每片只写 label，不钉台）
#     shard 照 TASK_SERVER_SHARD=<k>/<n> 只判片号对 n 取余等于 k 的片，判完并回片结果库，不打产物；建好服务给的产物目录（空的）
#     merge 在本机：片全取自库、合并出产物（与不分片的整趟同一段）；TASK_SERVER_SHARD_MISSING 不空（有片被人手取消）就退 77、点名缺的片，不当全了
#   不经服务分片（环境里没有 TASK_SERVER_SHARD_PHASE）：全部片在这一台判，片齐了出产物。两边判完的片各自并回同一份 prune-server 片结果库；
#   两边跑完再不带参数跑一次本脚本：片全部取自库、合并出产物。
#   装置在一趟里不许换（欠账 C862，共用段落在 long-run-lib/binary-identity.sh）：开跑时把装置二进制拷一份进 $LOG_DIRECTORY/pinned-binary/<sha256>/（只读），装置按这份拷贝起，
#   不再 exec 仓根 target/release 下那一份（别的会话的 cargo 随时会把它换掉）；开跑那句报这份拷贝的 sha256。这一段的片是片结果库里按复用键命名的条目，复用键含判法入口的函数单元闭包指纹
#   与工具链行，哪一版代码判的由键管，所以本脚本不给库里的片另记二进制 sha256（记了会把「代码没变、二进制重编过」的片也作废，违背按键复用）；--list-items 只列名，照旧直接起仓根那份。
#
# 片结果库（是判定库：片按复用键存、键相同的下一趟直接取；.claude/rules/verification.md「判定库只有一份」）：在 prune-server 里，族 unit-verdicts、种类 LIBRARY_KIND，
#   一片一条 <片键的 sha256>.shard（键空间在 scripts/prune-server/prune_server_client.py 文件头「键空间」）；库名 <这棵树的 git common-dir>/singlefs-unit-verdicts/<种类>
#   只当名字（键里只用种类名，哪棵仓不进键，命名空间只在临时服务底下给），本地不建、不读、不写。不在 git 工作树里退 2。
#   装置读写的是目录，所以本脚本每一趟开一个中转目录（$LOG_DIRECTORY 下 <种类>-<日期>-<进程号>.transit-shards）：开跑前经 scripts/gate/admission.py library-files fetch
#   把库拉进去，装置照 E182_VERDICT_LIBRARY 指到它，跑着时每 MERGE_INTERVAL_SECONDS 秒、装置退出时（退非 0 也并）经 library-files store 把判完的片并回服务，
#   收尾删掉中转目录。服务连不上（fetch 或最后那一次 store 退 75）：整趟退 75，不退回本地、不当库里没有；跑着时那几次并回退 75 只报一行，跑完再并。
#   被整组杀掉（撞内存上限）时最后一次并回之后判完的片没记下，下一趟照判；留下的中转目录不再读。
#   装置没有缺省的本地库：不设 E182_VERDICT_LIBRARY 就退 2（判定库只有 prune-server 那一份）；本脚本每一趟都把它设成中转目录（装置读写的就是它）。
#
# 用法（在仓里任何目录起都行，脚本自己回到仓根）：
#   bash scripts/experiment-apparatus/e182-single-device-long-run.sh                          本机跑全部片并出产物（片结果库里已有的片直接取）
#   bash scripts/experiment-apparatus/e182-single-device-long-run.sh --list-items             列可点名的历史名（要装置已编好，不编译）
#   bash scripts/experiment-apparatus/e182-single-device-long-run.sh --selftest               假装置、假 cargo 在临时树里走一遍整库续跑、分片三步、并回服务、产物名取号与完成标记（不起 cargo、编译）
# 环境：E182_LOG_DIRECTORY（进度日志与中转目录放哪，默认工作根下的 e182-long-run，工作根取 .claude/hooks/lib_work_root.py、两台同一份）。
# 产物名：research/results/e182-unreadable-abandoned-root-forks-single-device.out 已有时另存 …-single-device-<日期>.out，同一天再跑加 -r2、-r3，已有的一个不动。
# 退出码：0 做完（分片时是这一份的片判完）；1 装置跑失败或产物不齐；2 用法错、不在 git 工作树里；3 装置报停机（该停下交回主 agent 的，日志里 STOP: 那一行）；
#   75 prune-server 不可用（片结果库拉不下来或判完的片并不回去）；78 装置没编好（--list-items）。
#
# 自证的做法：把本脚本拷进临时树（scripts/experiment-apparatus/ 下；树里没有 scripts/hosts/，常规路径要是又套上线程包装就起不来、各格照红），树里放
#   假的 target/release/ 装置（按 E182_SHARD_SHARE 与 E182_VERDICT_LIBRARY 往中转目录写六个片文件、打现判/复用那一行、按 E182_SELFTEST_DEVICE 装停机/失败/产物不齐/判完几片再失败/判完几片再拿住；
#   E182_SELFTEST_SWAP_TARGET 设了时头一回起来先把仓根那份换掉、再按自己被起的路径 exec 一次）、
#   PATH 最前面的假 cargo（E182_SELFTEST_CARGO=fail 退 101）、指向真仓 .claude 与 scripts/gate 的符号链接（准入判法、library-files）；每一格各搭一棵树、各用各的日志目录，格与格不共用状态。
#   片结果库写进临时 prune-server（库名取每一格的树，键带临时目录的命名空间）；库里有几片经 admission.py library-files fetch 拉进这一格的临时目录数。
#   只在临时 prune-server 上跑：开跑先调客户端 selftest_service_problem，不是临时服务就退 2、一格都不跑，不写真服务。起法：
#     python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <prune-server 二进制> -- bash scripts/experiment-apparatus/e182-single-device-long-run.sh --selftest
#   换不掉的一处：产物名里的日期用 TZ=Asia/Tokyo date +%F 取，自证里同样取，跨零点的那一刻会误红，重跑即可。
# 弄坏开关 E182_SINGLE_DEVICE_BREAK=<项>（只在 --selftest 起的那几趟里认：自证设 E182_SINGLE_DEVICE_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   E182_SINGLE_DEVICE_BREAK=unknown-argument-accepted  认不得的参数悄悄跳过
#       「认不得的参数退 2」
#   E182_SINGLE_DEVICE_BREAK=list-items-binary-unchecked  --list-items 不核对装置在不在
#       「--list-items：装置没编好退 78」
#   E182_SINGLE_DEVICE_BREAK=git-common-directory-unchecked  不在 git 工作树里也往下走（库名落在当前目录底下）
#       「不在 git 工作树里退 2、装置一次没起」
#   E182_SINGLE_DEVICE_BREAK=build-failure-ignored  装置编不过也往下跑
#       「装置编不过退 1、装置一次没起」
#   E182_SINGLE_DEVICE_BREAK=device-failure-ignored  装置退非 0 也不当失败
#       「装置失败退 1 并说退几」「装置判完三片再失败：退 1，判完的三片已并回服务」
#   E182_SINGLE_DEVICE_BREAK=stop-exit-ignored  装置退 3（停机）当普通失败（退 1）
#       「装置报停机退 3、STOP 行转出来、不打产物」
#   E182_SINGLE_DEVICE_BREAK=complete-marker-ignored  产物末行对不上行数也算齐
#       「产物末行对不上行数退 1、不入库」
#   E182_SINGLE_DEVICE_BREAK=share-not-passed  shard 那一步不把片号传给装置
#       「shard 0/2 只判自己那一份、不打产物」
#   E182_SINGLE_DEVICE_BREAK=plan-shard-count-ignored  plan 那一步不管台数、一律写 1 片
#       「plan 两台：写 2 片」
#   E182_SINGLE_DEVICE_BREAK=missing-shards-ignored  merge 那一步有片缺（TASK_SERVER_SHARD_MISSING 不空）也照整趟判
#       「merge 缺片：退 77、点名缺的片、装置一次没起」
#   E182_SINGLE_DEVICE_BREAK=product-name-fixed  产物名不取号、一律用本名
#       「原样重起：六片全取自库」「产物本名已有：另存带日期的」「本名与带日期的都已有：加 -r2」
#   E182_SINGLE_DEVICE_BREAK=exec-root-binary  装置仍按仓根那个路径起、不按私有副本
#       「装置在一趟里不许换：起装置之前仓根那份被换掉，装置自报的 sha256 与片里记的、开跑那句报的都是开跑时那份」
#   E182_SINGLE_DEVICE_BREAK=transit-not-merged  中转目录里判完的片不并回服务（跑着时与装置退出时都不并）
#       「单机全新：构建一次、六片现判、产物按本名落进 research/results，六片并回服务、本地不建库」「原样重起：六片全取自库」
#       「shard 两份各自并回服务、merge 那一步片全取自库、合并出产物」「装置在一趟里不许换：起装置之前仓根那份被换掉，装置自报的 sha256 与片里记的、开跑那句报的都是开跑时那份」
#       「装置判完三片再失败：退 1，判完的三片已并回服务」「跑着时定时并回：判完三片之后整趟被 SIGKILL，三片已在服务里」
#   E182_SINGLE_DEVICE_BREAK=failed-run-not-merged  装置退非 0 时不并回（改错的样子：失败那一趟判完的片丢了）
#       「装置判完三片再失败：退 1，判完的三片已并回服务」
#   E182_SINGLE_DEVICE_BREAK=periodic-merge-off  跑着时不定时并回，只在装置退出时并一次
#       「跑着时定时并回：判完三片之后整趟被 SIGKILL，三片已在服务里」
#   E182_SINGLE_DEVICE_BREAK=unreachable-treated-as-empty  片结果库拉取时连不上服务照「库里没有」往下跑（装置照起）
#       「服务连不上退 75、点名 prune-server 不可用、装置一次没起」
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "$(dirname "${BASH_SOURCE[0]}")/long-run-lib/binary-identity.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; echo "     → 怎么办：本脚本得留在仓里的 scripts/experiment-apparatus/ 下（它按自己的位置找仓根）；挪走了就放回去再起" >&2; exit 2; }

BINARY="target/release/e182_unreadable_abandoned_root_forks"
DEVICE_MODE="single-device"
PRODUCT_STEM="research/results/e182-unreadable-abandoned-root-forks-single-device"
LIBRARY_KIND="e182-single-device"           # 片结果库在 prune-server 里的种类（文件头「片结果库」）
ADMISSION_MODULE="scripts/gate/admission.py"  # library-files：片结果库在服务与中转目录之间搬
MERGE_INTERVAL_SECONDS=300                    # 跑着时每隔几秒把判完的片并回服务一次（被整组杀掉时丢的至多是这一段）
SELFTEST_MARK_TEXT="e182-single-device-selftest"
LOG_DIRECTORY="${E182_LOG_DIRECTORY:-}"
if [[ -z "$LOG_DIRECTORY" ]]; then   # 没设：工作根（.claude/hooks/lib_work_root.py，两台同一份）下的 e182-long-run；自证的临时树各格都给了目录，不走这里
  if ! LOG_DIRECTORY="$(python3 .claude/hooks/lib_work_root.py)" || [[ "$LOG_DIRECTORY" != /* ]]; then
    echo "  ✗ 取不到工作根（.claude/hooks/lib_work_root.py 跑不起来），缺省的日志目录没处放" >&2
    echo "     → 怎么办：修好 .claude/hooks/lib_work_root.py，或设 E182_LOG_DIRECTORY 指到日志目录" >&2
    exit 2
  fi
  LOG_DIRECTORY="$LOG_DIRECTORY/e182-long-run"
fi

usage_error() {
  echo "  ✗ $1" >&2
  echo "     → 怎么办：见本脚本文件头「用法」" >&2
  exit 2
}

# 弄坏开关与自证专用的变量只在自证起的那几趟里认（标记文件在、内容对得上）；弄坏开关别处设了就拒。
break_is() {
  [[ ",${E182_SINGLE_DEVICE_BREAK:-}," == *",$1,"* ]]
}
selftest_mark_valid() {
  [[ -n "${E182_SINGLE_DEVICE_SELFTEST_MARK:-}" && "$(cat -- "${E182_SINGLE_DEVICE_SELFTEST_MARK}" 2> /dev/null)" == "$SELFTEST_MARK_TEXT" ]]
}
if [[ -n "${E182_SINGLE_DEVICE_BREAK:-}" && "${1:-}" != --selftest ]] && ! selftest_mark_valid; then
  echo "  ✗ E182_SINGLE_DEVICE_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "     → 怎么办：平常跑时不设它" >&2
  exit 2
fi

# 台数：1 + --emit-hosts 打的 MULTI_HOST_COUNT（双机关着它退 3，只有本机）。E182_HOSTS_EMITTER 只给自证换假的。
HOSTS_EMITTER="scripts/crash/crash-point-replay-shard-configuration-check.sh"
if selftest_mark_valid && [[ -n "${E182_HOSTS_EMITTER:-}" ]]; then HOSTS_EMITTER="$E182_HOSTS_EMITTER"; fi
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
  [[ "$count" =~ ^[0-9]+$ ]] || { echo "  ✗ --emit-hosts 没打 MULTI_HOST_COUNT" >&2; echo "     → 怎么办：单跑 bash $HOSTS_EMITTER --emit-hosts 看它打了什么" >&2; return 1; }
  echo $((1 + count))
}

# 服务的 plan 那一步：往计划目录写 plan.json（每片只写 label，不钉台）。
write_plan() { # <片数>
  python3 - "$plan_directory" "$1" << 'PLAN_PYTHON' || { echo "  ✗ 计划目录 $plan_directory 写不进 plan.json（上面是 python 的报错）" >&2; echo "     → 怎么办：看那个目录可不可写" >&2; return 1; }
import json, os, sys
directory, count = sys.argv[1], int(sys.argv[2])
shards = [{"label": f"shard {index}/{count}（片号对 {count} 取余等于 {index} 的片）"} for index in range(count)]
temporary = os.path.join(directory, "plan.json.partial")
with open(temporary, "w", encoding="utf-8") as handle:
    json.dump({"shards": shards, "note": "e182 分片（本脚本文件头「多机的读法」）"}, handle, ensure_ascii=False)
os.replace(temporary, os.path.join(directory, "plan.json"))
PLAN_PYTHON
}

# 产物做完了：末行是 name=done emitted=N 且 N 等于行数。
is_complete_output() {
  local file="$1" line_count last_line
  [[ -f "$file" ]] || return 1
  break_is complete-marker-ignored && return 0
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" == "E7RESULT name=done emitted=$line_count" ]]
}

# 还没被占的产物名：先试没有日期的，已有就加日期，同一天再加 -r2、-r3……
next_free_product_path() {
  local today candidate revision
  today=$(TZ=Asia/Tokyo date +%F)
  candidate="$PRODUCT_STEM.out"
  break_is product-name-fixed && { echo "$candidate"; return; }
  [[ -e "$candidate" ]] || { echo "$candidate"; return; }
  candidate="$PRODUCT_STEM-$today.out"
  [[ -e "$candidate" ]] || { echo "$candidate"; return; }
  revision=2
  while [[ -e "$PRODUCT_STEM-$today-r$revision.out" ]]; do revision=$((revision + 1)); done
  echo "$PRODUCT_STEM-$today-r$revision.out"
}

# 编译与跑装置都直接起、不再套线程包装：整条命令交 task-server，线程照服务交的 SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS
build_the_binary() {
  cargo build --release -p singlefs-verification --bin e182_unreadable_abandoned_root_forks || break_is build-failure-ignored || {
    echo "  ✗ 装置编不过" >&2
    echo "     → 怎么办：照 cargo 打的第一处 error 改装置，再起本脚本" >&2
    exit 1
  }
}


# 自证：假装置、假 cargo 在每一格自己的临时树里走一遍（做法与换不掉的一处见文件头）。
run_selftest() {
  local work failures=0 cell_count=0 fakes real_root today cell exit_code swap_target original_sha service_problem unreachable_address
  local script_name="${SCRIPT_PATH##*/}" script_pid device_pid waited
  unset E182_LOG_DIRECTORY
  real_root="$PWD"
  # 片结果库写进 prune-server：只在临时服务上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service），
  # 不是就一格都不跑、退 2，不写真服务（判据只在客户端的 selftest_service_problem 一处）
  if ! service_problem="$(python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem())' "$real_root/scripts/prune-server" 2>&1)" \
     || [[ -n "$service_problem" ]]; then
    echo "  ✗ $script_name 自证没跑：${service_problem:-载入 scripts/prune-server/prune_server_client.py 没成}"
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）"
    return 2
  fi
  today=$(TZ=Asia/Tokyo date +%F)
  work=$(mktemp -d) || return 1
  fakes="$work/fakes"
  mkdir -p -- "$fakes/bin"
  echo "$SELFTEST_MARK_TEXT" > "$work/mark"
  cat > "$fakes/device" <<'FAKE'
#!/usr/bin/env bash
# E182_SELFTEST_SWAP_TARGET：头一回起来、还没干活之前（脚本已拷过私有副本）把这个路径的装置换成另一份字节不同的，像别的会话的 cargo 把仓根那份换掉，
#   再按自己被起的那个路径 exec 一次：按私有副本起的照旧是开跑时那份，按仓根路径起的（弄坏开关 exec-root-binary）就成了换后那份。
if [[ -n "${E182_SELFTEST_SWAP_TARGET:-}" && ! -e "$E182_SELFTEST_SWAP_TARGET.swapped" ]]; then
  { cat -- "$E182_SELFTEST_SWAP_TARGET"; echo "# swapped by a build while the long run is going"; } > "$E182_SELFTEST_SWAP_TARGET.swap" \
    && chmod +x "$E182_SELFTEST_SWAP_TARGET.swap" && mv -f -- "$E182_SELFTEST_SWAP_TARGET.swap" "$E182_SELFTEST_SWAP_TARGET" && : > "$E182_SELFTEST_SWAP_TARGET.swapped" \
    && exec "$0" "$@"
fi
echo "device $* | library=${E182_VERDICT_LIBRARY:-unset} share=${E182_SHARD_SHARE:-unset}" >> "$E182_SELFTEST_CALLS"
echo "$(sha256sum -- "$0" | cut -d' ' -f1)" >> "$E182_SELFTEST_CALLS.self-sha"
if [[ "$1" == list-items ]]; then printf 'history_a\nhistory_b\n'; exit 0; fi
case "${E182_SELFTEST_DEVICE:-ok}" in
  stop) echo "STOP: selftest stop reason" >&2; exit 3 ;;
  fail) echo "boom" >&2; exit 1 ;;
esac
library="${E182_VERDICT_LIBRARY:?本脚本每一趟都把它设成中转目录}"
mkdir -p -- "$library"
share_index=0; share_total=1
if [[ -n "${E182_SHARD_SHARE:-}" ]]; then share_index=${E182_SHARD_SHARE%/*}; share_total=${E182_SHARD_SHARE#*/}; fi
shards=(0 1 2 3 4 5)
# 判完三片再失败 / 再拿住（被整组杀掉那一格）：只判 0、1、2 三片
case "${E182_SELFTEST_DEVICE:-ok}" in fail-after-shards|hold-after-shards) shards=(0 1 2) ;; esac
fresh=0; reused=0
for index in "${shards[@]}"; do
  ((index % share_total == share_index)) || continue
  shard_file="$library/shard-$index.shard"
  if [[ -f "$shard_file" ]]; then reused=$((reused + 1)); else echo "judged shard $index by device $(sha256sum -- "$0" | cut -d' ' -f1)" > "$shard_file"; fresh=$((fresh + 1)); fi
done
echo "现判 $fresh、复用 $reused" >&2
case "${E182_SELFTEST_DEVICE:-ok}" in
  fail-after-shards) echo "boom after three shards" >&2; exit 1 ;;
  hold-after-shards) echo "$$" > "$E182_SELFTEST_CALLS.device-pid"; : > "$E182_SELFTEST_CALLS.shards-written"; sleep 6; exit 0 ;;
esac
if [[ -n "${E182_SHARD_SHARE:-}" ]]; then exit 0; fi
for index in 0 1 2 3 4 5; do printf 'E7RESULT name=row shard=%s\n' "$index"; done
emitted=7
if [[ "${E182_SELFTEST_DEVICE:-ok}" == truncated ]]; then emitted=99; fi
printf 'E7RESULT name=done emitted=%s\n' "$emitted"
FAKE
  cat > "$fakes/bin/cargo" <<'FAKE'
#!/usr/bin/env bash
echo "cargo $*" >> "$E182_SELFTEST_CALLS"
if [[ "${E182_SELFTEST_CARGO:-ok}" == fail ]]; then echo "error: selftest compile error" >&2; exit 101; fi
exit 0
FAKE
  printf '#!/usr/bin/env bash\nprintf "MULTI_HOST_COUNT=1\\n"\n' > "$fakes/hosts-two"
  printf '#!/usr/bin/env bash\nexit 3\n' > "$fakes/hosts-one"
  chmod +x "$fakes/device" "$fakes/bin/cargo" "$fakes/hosts-two" "$fakes/hosts-one"
  # 一格的树：本脚本的拷贝、假装置、指向真仓 .claude 与 scripts/gate 的符号链接、空的 git 仓。
  make_tree() {
    local tree="$1/tree"
    mkdir -p -- "$tree/scripts/experiment-apparatus/long-run-lib" "$tree/target/release" "$tree/research/results" "$1/logs"
    cp -- "$SCRIPT_PATH" "$tree/scripts/experiment-apparatus/$script_name"
    cp -- "$(dirname -- "$SCRIPT_PATH")/long-run-lib/binary-identity.sh" "$tree/scripts/experiment-apparatus/long-run-lib/"
    cp -- "$fakes/device" "$tree/target/release/e182_unreadable_abandoned_root_forks"
    ln -s -- "$real_root/.claude" "$tree/.claude"
    ln -s -- "$real_root/scripts/gate" "$tree/scripts/gate"
    git -C "$tree" init -q > /dev/null 2>&1
  }
  # 在这一格的树里起脚本拷贝；这一格要设的环境变量写在 invoke 前面。
  invoke() {
    local cell="$1"
    shift
    ( cd -- "$cell/tree" && env PATH="$fakes/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E182_SELFTEST_CALLS="$cell/calls" \
        E182_LOG_DIRECTORY="$cell/logs" SINGLEFS_WORK_ROOT="$cell/work-root" E182_SINGLE_DEVICE_SELFTEST_MARK="$work/mark" \
        bash "$cell/tree/scripts/experiment-apparatus/$script_name" "$@" > "$cell/out" 2>&1 )
  }
  count_calls() {
    local count
    count=$(grep -c -- "$2" "$1/calls" 2> /dev/null) || true
    echo "${count:-0}"
  }
  count_files() {
    find "$1" -type f 2> /dev/null | wc -l
  }
  # 服务里这一格的片结果库有几片：经 admission.py library-files 拉进 <格>/fetched 数；拉不成打「拉不成」
  fetched_count() {
    rm -rf -- "${1:?}/fetched"
    if python3 "$real_root/$ADMISSION_MODULE" library-files fetch "$1/tree/.git/singlefs-unit-verdicts/$LIBRARY_KIND" "$1/fetched" > /dev/null 2>&1; then
      find "$1/fetched" -name '*.shard' 2> /dev/null | wc -l
    else
      echo 拉不成
    fi
  }
  expect() {
    local name="$1" condition="$2"
    cell_count=$((cell_count + 1))
    if eval "$condition"; then
      echo "  ✓ $name（第 $cell_count 格）"
    else
      echo "  ✗ $name：条件「$condition」不成立"
      [[ -f "$cell/out" ]] && tail -n 6 -- "$cell/out" | sed 's/^/      输出：/'
      echo "     → 怎么办：照上面这一格点名的条件和输出改本脚本，再单独起 python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/$script_name --selftest 到它退 0"
      failures=$((failures + 1))
    fi
  }
  local stem="$PRODUCT_STEM"
  local first=0 second=0
  cell="$work/fresh"; make_tree "$cell"
  invoke "$cell"; exit_code=$?
  expect "单机全新：构建一次、六片现判、产物按本名落进 research/results，六片并回服务、本地不建库" '((exit_code == 0)) && grep -qF "产物：$stem.out（7 行）；现判 6、复用 0" "$cell/out" && (($(wc -l < "$cell/tree/$stem.out") == 7)) && (($(count_calls "$cell" "^cargo build --release -p singlefs-verification --bin e182_unreadable_abandoned_root_forks") == 1)) && grep -qF "device $DEVICE_MODE | library=$cell/logs/$LIBRARY_KIND-" "$cell/calls" && grep -qF ".transit-shards share=unset" "$cell/calls" && [[ "$(fetched_count "$cell")" == 6 && ! -e "$cell/tree/.git/singlefs-unit-verdicts" ]] && ! compgen -G "$cell/logs/*.transit-shards" > /dev/null'
  cell="$work/restart"; make_tree "$cell"
  invoke "$cell"; first=$?
  invoke "$cell"; exit_code=$?
  expect "原样重起：六片全取自库" '((first == 0 && exit_code == 0)) && grep -qF "现判 0、复用 6" "$cell/out" && [[ -f "$cell/tree/$stem-$today.out" ]]'
  cell="$work/name-taken"; make_tree "$cell"
  echo old-product > "$cell/tree/$stem.out"
  invoke "$cell"; exit_code=$?
  expect "产物本名已有：另存带日期的" '((exit_code == 0)) && (($(wc -l < "$cell/tree/$stem-$today.out") == 7)) && [[ "$(cat -- "$cell/tree/$stem.out")" == old-product ]]'
  cell="$work/name-and-date-taken"; make_tree "$cell"
  echo old-product > "$cell/tree/$stem.out"; echo old-dated-product > "$cell/tree/$stem-$today.out"
  invoke "$cell"; exit_code=$?
  expect "本名与带日期的都已有：加 -r2" '((exit_code == 0)) && (($(wc -l < "$cell/tree/$stem-$today-r2.out") == 7)) && [[ "$(cat -- "$cell/tree/$stem.out")" == old-product ]] && [[ "$(cat -- "$cell/tree/$stem-$today.out")" == old-dated-product ]]'
  cell="$work/plan-two-hosts"; make_tree "$cell"; mkdir -p -- "$cell/plan"
  TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" E182_HOSTS_EMITTER="$fakes/hosts-two" invoke "$cell"; exit_code=$?
  expect "plan 两台：写 2 片" '((exit_code == 0)) && (($(grep -o "\"label\"" "$cell/plan/plan.json" | wc -l) == 2)) && [[ ! -e "$cell/calls" ]]'
  cell="$work/plan-one-host"; make_tree "$cell"; mkdir -p -- "$cell/plan"
  TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" E182_HOSTS_EMITTER="$fakes/hosts-one" invoke "$cell"; exit_code=$?
  expect "plan 只有本机：写 1 片" '((exit_code == 0)) && (($(grep -o "\"label\"" "$cell/plan/plan.json" | wc -l) == 1)) && [[ ! -e "$cell/calls" ]]'
  cell="$work/share"; make_tree "$cell"; mkdir -p -- "$cell/plan"
  TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD=0/2 TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$cell/output-0" invoke "$cell"; exit_code=$?
  expect "shard 0/2 只判自己那一份、不打产物" '((exit_code == 0)) && grep -qF "这一份（0/2）的片判完：现判 3、复用 0" "$cell/out" && grep -qF "share=0/2" "$cell/calls" && (($(count_files "$cell/tree/research/results") == 0)) && [[ -d "$cell/output-0" ]]'
  cell="$work/shares-then-merge"; make_tree "$cell"; mkdir -p -- "$cell/plan"
  TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD=0/2 TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$cell/output-0" invoke "$cell"; first=$?
  TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD=1/2 TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$cell/output-1" invoke "$cell"; second=$?
  TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" TASK_SERVER_SHARD_MISSING="" invoke "$cell"; exit_code=$?
  expect "shard 两份各自并回服务、merge 那一步片全取自库、合并出产物" '((first == 0 && second == 0 && exit_code == 0)) && grep -qF "现判 0、复用 6" "$cell/out" && (($(wc -l < "$cell/tree/$stem.out") == 7)) && (($(count_calls "$cell" "^device $DEVICE_MODE") == 3))'
  cell="$work/merge-missing-shard"; make_tree "$cell"; mkdir -p -- "$cell/plan"
  TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$cell/plan" TASK_SERVER_SHARD_MISSING="1" invoke "$cell"; exit_code=$?
  expect "merge 缺片：退 77、点名缺的片、装置一次没起" '((exit_code == 77)) && grep -qF "TASK_SERVER_SHARD_MISSING=1" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/unknown-phase"; make_tree "$cell"
  TASK_SERVER_SHARD_PHASE=never-used-phase invoke "$cell"; exit_code=$?
  expect "服务的步骤认不出退 2、装置一次没起" '((exit_code == 2)) && grep -qF "TASK_SERVER_SHARD_PHASE=never-used-phase 认不出" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/unknown-argument"; make_tree "$cell"
  invoke "$cell" --bogus; exit_code=$?
  expect "认不得的参数退 2" '((exit_code == 2)) && grep -qF "认不得的参数「--bogus」" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/without-git"; make_tree "$cell"
  rm -rf -- "${cell:?}/tree/.git"
  invoke "$cell"; exit_code=$?
  expect "不在 git 工作树里退 2、装置一次没起" '((exit_code == 2)) && grep -qF "不在 git 工作树里" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/build-failure"; make_tree "$cell"
  E182_SELFTEST_CARGO=fail invoke "$cell"; exit_code=$?
  expect "装置编不过退 1、装置一次没起" '((exit_code == 1)) && grep -qF "装置编不过" "$cell/out" && (($(count_calls "$cell" "^device") == 0))'
  cell="$work/device-stop"; make_tree "$cell"
  E182_SELFTEST_DEVICE=stop invoke "$cell"; exit_code=$?
  expect "装置报停机退 3、STOP 行转出来、不打产物" '((exit_code == 3)) && grep -qF "STOP: selftest stop reason" "$cell/out" && (($(count_files "$cell/tree/research/results") == 0))'
  cell="$work/device-failure"; make_tree "$cell"
  E182_SELFTEST_DEVICE=fail invoke "$cell"; exit_code=$?
  expect "装置失败退 1 并说退几" '((exit_code == 1)) && grep -qF "装置退 1（日志" "$cell/out" && (($(count_files "$cell/tree/research/results") == 0))'
  cell="$work/failure-after-shards"; make_tree "$cell"
  E182_SELFTEST_DEVICE=fail-after-shards invoke "$cell"; exit_code=$?
  expect "装置判完三片再失败：退 1，判完的三片已并回服务" '((exit_code == 1)) && grep -qF "装置退 1（日志" "$cell/out" && [[ "$(fetched_count "$cell")" == 3 ]]'
  cell="$work/product-truncated"; make_tree "$cell"
  E182_SELFTEST_DEVICE=truncated invoke "$cell"; exit_code=$?
  expect "产物末行对不上行数退 1、不入库" '((exit_code == 1)) && grep -qF "产物不齐" "$cell/out" && (($(count_files "$cell/tree/research/results") == 0))'
  cell="$work/swap"; make_tree "$cell"
  swap_target="$cell/tree/target/release/e182_unreadable_abandoned_root_forks"
  original_sha=$(sha256sum -- "$swap_target" | cut -d' ' -f1)
  E182_SELFTEST_SWAP_TARGET="$swap_target" invoke "$cell"; exit_code=$?
  expect "装置在一趟里不许换：起装置之前仓根那份被换掉，装置自报的 sha256 与片里记的、开跑那句报的都是开跑时那份" '((exit_code == 0)) && [[ -e "$swap_target.swapped" ]] && [[ "$(sha256sum -- "$swap_target" | cut -d" " -f1)" != "$original_sha" ]] && [[ "$(sort -u "$cell/calls.self-sha")" == "$original_sha" ]] && grep -qF "装置 sha256 $original_sha（私有副本" "$cell/out" && [[ "$(fetched_count "$cell")" == 6 ]] && (($(grep -l "by device $original_sha" "$cell"/fetched/*.shard | wc -l) == 6))'
  # 跑着时定时并回（文件头「片结果库」）：并回间隔设 1 秒，装置判完三片之后拿住；等过几次并回再对整趟（脚本这一层，本格自己起的进程）发 SIGKILL，
  # 像撞上限被整组杀掉：三片已在服务里。装置自己拿住 6 秒后退出，这里只等它退、不对它发信号
  cell="$work/periodic-merge"; make_tree "$cell"
  ( cd -- "$cell/tree" && exec env PATH="$fakes/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E182_SELFTEST_CALLS="$cell/calls" \
      E182_LOG_DIRECTORY="$cell/logs" SINGLEFS_WORK_ROOT="$cell/work-root" E182_SINGLE_DEVICE_SELFTEST_MARK="$work/mark" \
      E182_SELFTEST_DEVICE=hold-after-shards E182_SELFTEST_MERGE_SECONDS=1 \
      bash "$cell/tree/scripts/experiment-apparatus/$script_name" > "$cell/out" 2>&1 ) &
  script_pid=$!
  waited=0
  while [[ ! -e "$cell/calls.shards-written" ]] && ((waited < 300)) && kill -0 "$script_pid" 2> /dev/null; do sleep 0.1; waited=$((waited + 1)); done
  sleep 3
  kill -KILL "$script_pid" 2> /dev/null
  wait "$script_pid" 2> /dev/null
  device_pid=$(cat -- "$cell/calls.device-pid" 2> /dev/null)
  waited=0
  while [[ -n "$device_pid" ]] && kill -0 "$device_pid" 2> /dev/null && ((waited < 100)); do sleep 0.1; waited=$((waited + 1)); done
  expect "跑着时定时并回：判完三片之后整趟被 SIGKILL，三片已在服务里" '[[ -e "$cell/calls.shards-written" && "$(fetched_count "$cell")" == 3 ]]'
  cell="$work/unreachable"; make_tree "$cell"
  unreachable_address="$(python3 -B -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print("127.0.0.1:%d" % s.getsockname()[1]); s.close()')"
  SINGLEFS_PRUNE_SERVER_SOCKET="$cell/no-such.sock" SINGLEFS_PRUNE_SERVER_TCP_ADDRESS="$unreachable_address" SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=2 invoke "$cell"; exit_code=$?
  expect "服务连不上退 75、点名 prune-server 不可用、装置一次没起" '((exit_code == 75)) && grep -qF "prune-server 不可用" "$cell/out" && (($(count_calls "$cell" "^device") == 0))'
  cell="$work/list-without-binary"; make_tree "$cell"
  rm -f -- "$cell/tree/target/release/e182_unreadable_abandoned_root_forks"
  invoke "$cell" --list-items; exit_code=$?
  expect "--list-items：装置没编好退 78" '((exit_code == 78)) && grep -qF "装置还没编好" "$cell/out"'
  cell="$work/list"; make_tree "$cell"
  invoke "$cell" --list-items; exit_code=$?
  expect "--list-items：编好了直接列名、不编译" '((exit_code == 0)) && grep -qx "history_a" "$cell/out" && (($(count_calls "$cell" "^cargo") == 0))'
  rm -rf -- "${work:?}"
  if ((failures > 0)); then echo "$script_name 自证：$failures 格红（共 $cell_count 格）"; return 1; fi
  echo "$script_name 自证：$cell_count 格全绿"
  return 0
}

share=""
list_items=0
selftest=0
while (($# > 0)); do
  case "$1" in
    --list-items) list_items=1; shift ;;
    --selftest) selftest=1; shift ;;
    *) break_is unknown-argument-accepted && { shift; continue; }; usage_error "认不得的参数「$1」" ;;
  esac
done

if ((selftest)); then
  run_selftest
  exit $?
fi

# 经 task-server 分片（文件头「多机的读法」）：服务起的那一步照 TASK_SERVER_SHARD_PHASE 换用法
task_server_phase="${TASK_SERVER_SHARD_PHASE:-}"
plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}"
case "$task_server_phase" in
  "") ;;
  plan|shard|merge)
    [[ -n "$plan_directory" && -d "$plan_directory" ]] || { echo "  ✗ 服务的 $task_server_phase 那一步没给在的计划目录（TASK_SERVER_SHARD_PLAN_DIRECTORY=${plan_directory:-没设}）" >&2; echo "     → 怎么办：这一步只由 task-server 起" >&2; exit 2; } ;;
  *) echo "  ✗ TASK_SERVER_SHARD_PHASE=$task_server_phase 认不出（认 plan、shard、merge）" >&2; echo "     → 怎么办：这个变量只由 task-server 设；直接跑就去掉它" >&2; exit 2 ;;
esac
if [[ "$task_server_phase" == plan ]]; then
  hosts=$(host_count) || exit 1
  break_is plan-shard-count-ignored && hosts=1
  write_plan "$hosts" || exit 1
  echo "计划：$hosts 片（各片判片号对 $hosts 取余等于片号的片，不钉台、由服务挑机器）"
  exit 0
fi
if [[ "$task_server_phase" == shard ]]; then
  [[ "${TASK_SERVER_SHARD:-}" =~ ^[0-9]+/[1-9][0-9]*$ ]] || { echo "  ✗ 服务的 shard 那一步没给片号（TASK_SERVER_SHARD=${TASK_SERVER_SHARD:-没设}）" >&2; echo "     → 怎么办：这一步只由 task-server 起" >&2; exit 2; }
  [[ -z "${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}" ]] || mkdir -p -- "$TASK_SERVER_SHARD_OUTPUT_DIRECTORY" || exit 1
  share="$TASK_SERVER_SHARD"
fi
if [[ "$task_server_phase" == merge && -n "${TASK_SERVER_SHARD_MISSING:-}" ]] && ! break_is missing-shards-ignored; then
  echo "  ✗ 合并那一步缺片（TASK_SERVER_SHARD_MISSING=$TASK_SERVER_SHARD_MISSING）：这几片被人手取消、库里没有它们判的结果，不当全了判" >&2
  echo "     → 怎么办：重交这一件，让缺的片重判" >&2
  exit 77
fi

if ((list_items)); then
  [[ -x "$BINARY" ]] || break_is list-items-binary-unchecked || { echo "  ✗ 装置还没编好：$BINARY" >&2; echo "     → 怎么办：先不带 --list-items 跑一次本脚本，或 cargo build --release -p singlefs-verification --bin e182_unreadable_abandoned_root_forks" >&2; exit 78; }
  "$BINARY" list-items
  exit $?
fi

# 片结果库的库名（文件头「片结果库」）：这棵树的 git common-dir 底下 singlefs-unit-verdicts/<种类>，只当名字，本地不建
common=$(git rev-parse --path-format=absolute --git-common-dir 2> /dev/null) || common=""
if [[ -z "$common" ]]; then
  if break_is git-common-directory-unchecked; then
    common="$PWD/.git"
  else
    echo "  ✗ 取不到判定库的名字：$PWD 不在 git 工作树里，片结果库没有名字" >&2
    echo "     → 怎么办：在 git 工作树里起本脚本；rsync 拷的仓副本先在副本根 git init（键里不带哪棵仓，副本与主仓读写同一份库）" >&2
    exit 2
  fi
fi
library_name="$common/singlefs-unit-verdicts/$LIBRARY_KIND"

build_the_binary
mkdir -p -- "$LOG_DIRECTORY" || exit 1
run_stamp="$(TZ=Asia/Tokyo date +%F)-$$"
progress_log="$LOG_DIRECTORY/$LIBRARY_KIND-$run_stamp.progress.log"
product_scratch="$LOG_DIRECTORY/$LIBRARY_KIND-$run_stamp.product.out"
transit="$LOG_DIRECTORY/$LIBRARY_KIND-$run_stamp.transit-shards"
merge_log="$LOG_DIRECTORY/$LIBRARY_KIND-$run_stamp.merge.log"
# 中转目录这一趟用完就删（文件头「片结果库」）：正常结束、出错、被停都删；并回服务之前就被整组杀掉的，留下的这一份照「没并回」算，下一趟那几片照判
trap 'rm -rf -- "$transit"' EXIT
rm -rf -- "$transit"
mkdir -p -- "$transit" || exit 1

# merge_transit：中转目录里判完的片并回服务（逐字节相同的跳过）；交 library-files store 的退出码，输出进 $merge_log
merge_transit() {
  break_is transit-not-merged && return 0
  python3 "$ADMISSION_MODULE" library-files store "$library_name" "$transit" > "$merge_log" 2>&1
}
# report_unavailable <哪一步>：服务不可用，点名哪一步退了 75，整趟退 75（不退回本地、不当库里没有）
report_unavailable() {
  echo "  ✗ 片结果库：prune-server 不可用（$1 退 75，日志 $merge_log）" >&2
  tail -n 3 -- "$merge_log" >&2
  echo "     → 怎么办：python3 prune-server/control check 看服务，修好、重起之后重交这一条命令（报给主 agent）；这一趟不退回本地文件" >&2
  exit 75
}

python3 "$ADMISSION_MODULE" library-files fetch "$library_name" "$transit" > "$merge_log" 2>&1
fetch_exit=$?
if ((fetch_exit == 75)) && ! break_is unreachable-treated-as-empty; then report_unavailable "library-files fetch"; fi
if ((fetch_exit != 0 && fetch_exit != 75)); then
  echo "  ✗ 片结果库拉不下来（library-files fetch 退 $fetch_exit，日志 $merge_log）" >&2
  tail -n 3 -- "$merge_log" >&2
  echo "     → 怎么办：照日志里的出路修，再起本脚本" >&2
  exit 1
fi

arguments=("$DEVICE_MODE")
environment=("E182_VERDICT_LIBRARY=$transit")
if [[ -n "$share" ]] && ! break_is share-not-passed; then environment+=("E182_SHARD_SHARE=$share"); fi

# 开跑时拷一份私有副本，装置按这份拷贝起（上面 cargo build 与这一步之间被换掉的，拷的就是换后那一份，sha256 是拷贝的）
binary_identity_pin "$BINARY" "$LOG_DIRECTORY/pinned-binary" || exit 1
launch_binary="$PINNED_BINARY"
break_is exec-root-binary && launch_binary="$BINARY"
echo "装置 sha256 $PINNED_BINARY_SHA256（私有副本 $PINNED_BINARY，源 $BINARY）"
binary_identity_assert_unchanged || exit 1
# 装置放后台跑，每 merge_interval 秒把判完的片并回服务一次（文件头「片结果库」：被整组杀掉时丢的至多是最后这一段）；被停时把 TERM 转给装置
env ${environment[@]+"${environment[@]}"} \
  "$launch_binary" "${arguments[@]}" > "$product_scratch" 2> "$progress_log" &
device_pid=$!
trap 'kill -TERM "$device_pid" 2> /dev/null' TERM INT
merge_interval=$MERGE_INTERVAL_SECONDS
[[ -n "${E182_SELFTEST_MERGE_SECONDS:-}" ]] && selftest_mark_valid && merge_interval=$E182_SELFTEST_MERGE_SECONDS
since_merge=0
while kill -0 "$device_pid" 2> /dev/null; do
  sleep 1
  since_merge=$((since_merge + 1))
  if ((since_merge >= merge_interval)) && ! break_is periodic-merge-off; then
    since_merge=0
    merge_transit
    periodic_exit=$?
    ((periodic_exit == 75)) && echo "  ! 片结果库：这一次并回时 prune-server 不可用（日志 $merge_log），装置照跑，跑完再并一次" >&2
  fi
done
wait "$device_pid"
exit_code=$?
trap - TERM INT
if ((exit_code == 0)) || ! break_is failed-run-not-merged; then
  merge_transit
  merge_exit=$?
  ((merge_exit == 75)) && report_unavailable "library-files store（装置退 $exit_code 之后并回）"
  if ((merge_exit != 0)); then
    echo "  ✗ 判完的片并不回服务（library-files store 退 $merge_exit，日志 $merge_log）：这一趟判的片没记下" >&2
    tail -n 3 -- "$merge_log" >&2
    echo "     → 怎么办：照日志里的出路修，再起本脚本（没记下的片照判）" >&2
    exit 1
  fi
fi
reuse_line=$(grep -m1 '^现判 ' -- "$progress_log")
if ((exit_code == 3)) && ! break_is stop-exit-ignored; then
  echo "  ✗ 装置报停机，该停下交回主 agent" >&2
  grep -m3 '^STOP: ' -- "$progress_log" >&2
  echo "     → 怎么办：把日志 $progress_log 里的 STOP 行交主 agent，不重跑" >&2
  exit 3
fi
if ((exit_code != 0)) && ! break_is device-failure-ignored; then
  echo "  ✗ 装置退 $exit_code（日志 $progress_log；判完的片已并回服务）" >&2
  tail -n 5 -- "$progress_log" >&2
  echo "     → 怎么办：看日志末尾；内存撞顶由 task-server 放大重排，整条命令的结局看 task-server/task status" >&2
  exit 1
fi
if [[ -n "$share" ]]; then
  echo "这一份（$share）的片判完：${reuse_line:-（日志里没有现判/复用那一行）}；片已并回 prune-server（库名 $library_name）"
  exit 0
fi
if ! is_complete_output "$product_scratch"; then
  echo "  ✗ 产物不齐（末行不是 name=done emitted=行数）：$product_scratch" >&2
  echo "     → 怎么办：看日志 $progress_log；产物不入库" >&2
  exit 1
fi
destination=$(next_free_product_path)
mv -n -- "$product_scratch" "$destination" || exit 1
echo "产物：$destination（$(wc -l < "$destination") 行）；${reuse_line:-（日志里没有现判/复用那一行）}；进度日志：$progress_log"
