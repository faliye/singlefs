#!/usr/bin/env bash
# task-server-compiles: yes
# task-server-shards: *   经统一任务服务交的一律分片：服务拆成计划、各片、合并三步（下面「分片」；task-server/task_server_lib.py 文件头「分片」）；--selftest、--list-items、--help、-h 服务不分
# E207（B8严格判据下的步数）量的那一路的入口：给仓副本套上读数，起历史用例，产物按 <点>__<臂>（探针项 <点>__<臂>__<探针>）分文件写进 research/results/，每项的判定照单元键进 prune-server。
# 跑前登记 research/prompts/e207-preregistration.md 第五节「判定入库与按点单跑」、第十四节、第十二节执行员修订 4（分片）。
#
# 用法：
#   e207-strict-scan-steps-run.sh --list-items
#   e207-strict-scan-steps-run.sh --work <共享根下的工作目录> [--segment first|second|third|fourth|hundred_units_per_file_repeat|reconciliation_split | --item <点>__<臂>[__<探针>]…] [--source <仓根>] [--shard-count <片数>]
#   e207-strict-scan-steps-run.sh --work <共享根下的工作目录> --break-history units_of_not_refreshed --item <项>
#       登记第九节 M13 的证红：带弄坏开关 E207_RUN_BREAK=units_of_not_refreshed 起这一项（只此一项、不入判定库、原始输出只写进工作目录），
#       工作目录里 break-history-<开关>-<项>.out 是产物（只留 E207RESULT 行、去掉 libtest 接在第一行前的「test <用例名> ... 」，读数器与对拍直接读它），
#       break-history-<开关>-<项>.log 是 cargo test 的原始输出。
#       打 name=break_history … caught=yes|no（同一次挂载里 W2 声明 user_units=K 的行多于不同文件数就是抓到）；抓到退 0，没抓到退 1。
#       登记修订 3 的两条阳性对照另认两个开关：slots_placed_undercounted（只配 --item base__head：对账行打的 slots_placed 减 1，
#       原始输出里 slots_placed 少于 image_changed_unit_slots 的行有 ≥ 1 行就是抓到，打 caught=yes）与 image_witness_flipped（只配 --item base__plain：
#       plain 构建里把 remounted W2 第 5 次那一行的镜像翻一位再取哈希，打 caught=judged_by_the_model_against_the_good_base_head，由模型的 head_against_plain 对拍判）。
#       不带范围参数是全部登记的项（不指定范围）；--item 给几次取并集，项名写错退 2 并列可点的项。
#   e207-strict-scan-steps-run.sh --work <共享根下的工作目录> --device-test
#       紧凑盘的等价单测（四条，见 device_test）：起 plain 副本、只跑 compact_device::tests；四条全过退 0，否则退 1。
#   e207-strict-scan-steps-run.sh --selftest        核入口自己的列项、拒绝、完成标记、续跑、按键复用与复用哨兵，用假驱动，不编译、不跑实现；
#                                                  只在临时 prune-server 上跑（开跑先经判定库那一份调客户端 selftest_service_problem，不是临时服务退 2、一格不跑）
# 判定库（.claude/rules/verification.md「判定库只有一份」；键、库与哨兵的写法在同目录 e207_strict_scan_steps_library.py 文件头）：
#   每个 <点>__<臂> 一条判定，键是副本里用例 item_<点>__<臂> 的单元闭包键加工具链行、闭包真引到的依赖行、臂与跑法；副本路径不进键。族 apparatus-items，种类 e207-strict-scan-steps-run。
#   research/results/ 里的产物不当判定读，也不导入：库里没有的项照键现判一次。
# 一趟只编译一次：一份副本一棵 cargo 目录（服务给的 CARGO_TARGET_DIR），同一条臂同一个探针的项在同一个进程里按登记线程并行；臂与探针由环境变量 E207_ARM、E207_PROBE
#   在进程开头读一次（项名 __ 之后那一段：<臂> 或 <臂>__<探针>，没有探针的是 standard），所以一组（臂、探针）一个 cargo test。
# 同一条命令重起就是续跑：副本照计数脚本的开拷、完成标记；库里有键相同、做完了的项不重跑；上一次尝试的原始输出（副本里 .e207-raw 下，旁边 .keys 记着起 cargo 时的键）里
#   已经跑完、键又相同的现判项直接写成产物、照键入库；产物先写同目录隐藏 .partial 再换名；做完而没有 item_done 的产物不删不覆盖，重起时另写新文件名。
# 进度：历史那一段每过 5 分钟一行 `e207 progress item=<项> publishes_done=<n> of=<总数> peak_rounds=<r> peak_W=<w>`，转一份到标准输出；判结果只读原始输出。不作计时用。
#
# 分片（一个取样点一条任务跑不进服务的四小时硬上限，所以一项拆成片）：
#   经服务交的每一种跑法（文件头第三行 task-server-shards: *）拆成计划、各片、合并三步，每步都照原命令起这一份一次，环境里 TASK_SERVER_SHARD_PHASE 说是哪一步；
#   片数、哪台、内存上限由服务定，命令里不写。直接跑（环境里没有 TASK_SERVER_SHARD_PHASE）照旧整项整趟在这一台跑，不分片（自证与小项核对用；--shard-count 只在服务的计划一步里认）。
#   一项按历史里用户改动的序号切片（历史用例 e207_strict_scan_steps_histories.rs 的 ShardRange；序号 = History::note 被叫的次数，产物 name=history 行的 user_changes）：
#   第 i 片（共 K 片，i 从 0 起）管序号 (from, to]，from = i×N÷K、to = (i+1)×N÷K，N = 2 + 2×文件数 + 2×unlink 数（e207_strict_scan_steps_window_choices.rs 的 notes_of_the_history；
#   单测钉在 21 项里已有产物的 user_changes 上）。
#   前面的发布怎么到位：每一片都从头把历史放一遍，序号不超过 from 的读数照常取走（读数模块的状态——固定点次数、节点年龄表、直方图的桶——与不分片时逐步相同）、不打；
#   区间里的读数照打，与不分片时同一行；不是最后一片的，序号到 to 打完片尾标记就收尾，后面的历史、量窗与重挂载归后面的片；最后一片放完历史、做量窗与重挂载。
#   为什么读数与不分片时逐行相同：每一片跑的是同一段确定的程序（历史各步、种子、固定点都不随片变），只是打印按序号过滤；被过滤的行本来就由同一个状态机算出来。
#   这个选择的代价写在登记执行员修订 4：放前缀不比量快多少，所以第 i 片的挂钟约等于不分片时放到序号 to 的那一段，最后一片仍要放完整段历史；分片换来的是前面几片短、各片被打断后只补没判的片，
#   不是最后一片也短。
#   计划  编不过时（退非 0）把编译器报的每条 error 的正文（文件:行与那几行源码，取 JSON 里 compiler-message 的 rendered）打进日志，条数多的打前 10 条、余下报条数
#         （e207_strict_scan_steps_shards.py errors），再退 1；cargo 的 JSON 留在 <工作目录>/<counts|plain>-cargo-artifacts.json。
#   计划  编一次测试程序（cargo test --no-run --message-format=json）拷进 <计划目录>/binary-<counts|plain>，各片用同一份；写 <计划目录>/shards.tsv（每行：副本种类、项、片号、片数、决定）与 plan.json
#         （每片只写 label，不钉台）。计划按项键查库：整项键在库里的项决定复用，不进片；--break-history 的用法计划写 0 片，合并那一步照它跑。每项几片见 shard_count_of；总片数超过服务给的 TASK_SERVER_SHARD_MAXIMUM 时片数大的项一片一片往下减。
#   片数与内存：同一个点的几片各涨到自己那一段的峰值、同在一台上会一起涨：紧凑盘上量到的单片内存到这一台能给的内存四分之三以上的项，片数取 1（item_needs_a_whole_machine，读按项的内存表 memory_readings）。
#   各片  照 TASK_SERVER_SHARD=<序>/<总> 取 shards.tsv 的一行：库里有这一片的块（片键 = 项键 + 片号 + 片数，见 e207_strict_scan_steps_library.py 文件头「分片的片」）就用，
#         没有就用计划拷好的程序文件起这一片（E207_SHARD=<片号>/<片数>，--exact 点名一条用例），片文件（E207RESULT 行与末行 E207SHARD 片尾标记）写进服务给的产物目录并按片键入库；
#         片的进度行照转到标准输出（带 shard=、stage=replay|measure）。测试程序经 e207_strict_scan_steps_process_sampler.py 起：每 300 秒与退出时各一行
#         `e207 process item=<项> stage=sample|end …`（这一片的用户态与内核态 CPU 秒、缺页、自愿与被抢的切换、所在 cgroup 等 CPU、等内存、等读写的累计微秒），
#         转到标准输出、留在原始输出，不进片文件；平均在用核数低时拿它分「等 CPU」「等换页与回收」「等读写」（登记第十二节执行员修订 10）。
#   合并  读 TASK_SERVER_SHARD_OUTPUTS 里每片的片文件，e207_strict_scan_steps_shards.py merge 核账：片数、片号、区间首尾相接不重不漏、固定点次数首尾相接、
#         各片发布数之和 = 总数（= name=history 行的 user_changes）、最后一片 item_done；缺一片或对不上判红、不出产物。核过按片的次序首尾相接，加产物头，
#         写成与不分片时同名同形的一份（末行 item_done），照项键入库、跑复用哨兵的账（finish）。TASK_SERVER_SHARD_MISSING 不空即红。
#
# run-condition: 无（不要 GPU、不要第二台）。
# 退出码：0 跑到的全绿；1 有项没跑出 item_done、复用哨兵红、或计划里要现判的项这一趟没记；2 用法错、项名写错或工作目录里有不是本脚本拷的东西；
#   3 prune-server 客户端别的错，或锚点不是恰好命中一次（登记停机条款第 1 条）；75 prune-server 不可用（照客户端：不现判、不当「库里没有」）；
#   77 一项都没跑到；78 准入条件不满足（仓根或前置脚本不在）。
# 弄坏开关（只给 --selftest 用，平常跑时不认）。逐项写：开关、弄坏哪一处、打开后哪几格该红（失败行句首 name=selftest_case case=<格名> result=FAILED）：
#   E207_RUN_BREAK=done-marker-ignored  完成标记不看（凡判「做完没有」的地方都当做完了）
#       done_marker
#   E207_RUN_BREAK=done-items-rerun  计划复用的项照样重跑（不看 plan 给的 decision）
#       one_finished_item_is_not_run_again
#   E207_RUN_BREAK=harvest-off  不从上一次尝试的原始输出里取跑完的项
#       an_item_finished_inside_a_killed_run_is_taken_and_not_run_again
#   E207_RUN_BREAK=harvest-ignores-the-key  从原始输出取项时不核旁边 .keys 记的键
#       an_item_finished_inside_a_killed_run_under_another_key_is_run_again
#   E207_RUN_BREAK=cut-lines-kept  取项时不丢最后一行不带换行的残行
#       one_line_cut_by_a_kill_does_not_finish_an_item
#   E207_RUN_BREAK=product-written-in-place  产物直接写在产物名下（被杀时留下半份）
#       one_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name
#   E207_RUN_BREAK=filter-by-substring  点名不带 --exact，item_files_1__head 会连带跑 item_files_100__head 等
#       the_item_filter_names_one_test_only
#   E207_RUN_BREAK=copy-resume-off  副本不认完成标记与开拷标记，非空一律拒（在计数脚本里读，入口只是把环境带过去）
#       killed_after_the_copy_and_run_again_completes
#   E207_RUN_BREAK=progress-not-forwarded  进度行不转到标准输出
#       progress_lines_reach_the_standard_output
#   E207_RUN_BREAK=one-process-for-every-arm  几条臂的项并进一个 cargo test（E207_ARM 只有头一条臂的值）
#       each_arm_runs_in_its_own_process
#   下面几项在判定库那一份里读（e207_strict_scan_steps_library.py 文件头「弄坏开关」，入口只是把环境带过去）：
#   E207_RUN_BREAK=library-lookup-ignored  决定复用那一路不读库
#       one_finished_item_is_not_run_again、reuse-three-questions
#   E207_RUN_BREAK=record-skipped  跑完的项不往库里写
#       one_finished_item_is_not_run_again、reuse-three-questions
#   E207_RUN_BREAK=key-with-copy-path  请求另加副本的绝对路径
#       the_item_key_does_not_depend_on_the_copy_path
#   E207_RUN_BREAK=key-by-whole-histories-file  整份历史用例进键
#       reuse-three-questions
#   E207_RUN_BREAK=sentinel-skipped  不抽哨兵与强制现判、不判
#       reuse_sentinel_draws_and_the_forced_item_is_run_again、one_rejudged_sentinel_turns_the_run_red
#   E207_RUN_BREAK=sentinel-rejudge-unobserved  跑完把哨兵一律记成复用
#       one_rejudged_sentinel_turns_the_run_red
#   E207_RUN_BREAK=forced-read-as-reused  抽中强制现判的项照库复用
#       reuse_sentinel_draws_and_the_forced_item_is_run_again
#   E207_RUN_BREAK=forced-verdict-unchecked  不比强制现判的判定与库里那一份
#       one_forced_rejudge_with_another_verdict_turns_the_run_red
#   E207_RUN_BREAK=refused-items-reused  拒算的项照复用跳过、不跑
#       an_item_without_a_key_runs_and_is_never_recorded
#   E207_RUN_BREAK=unreachable-as-empty  读库时 prune-server 连不上照「库里没有」现判
#       an_unreachable_library_exits_75_and_runs_nothing
#   E207_RUN_BREAK=unrun-items-unchecked  跑完不核计划里要现判的项这一趟记过没有
#       an_item_planned_for_judging_but_not_run_turns_the_run_red
#   下面几项管分片（合并核账在 e207_strict_scan_steps_shards.py，片的入库在判定库那一份，入口只是把环境带过去；假程序在 write_fake_histories）：
#   E207_RUN_BREAK=merge-drops-a-shard  合并时丢掉第 1 片的正文（片数 ≥ 3），核账照丢之前的片文件核
#       sharded_product_equals_the_whole_product
#   E207_RUN_BREAK=merge-gap-unchecked  合并不核区间首尾相接、固定点次数与总数
#       shards_that_do_not_meet_turn_the_merge_red、the_totals_must_add_up
#   E207_RUN_BREAK=shards-guard-passes-always  分片核账脚本的守卫永远放行（自证把守卫那一趟的标记文件写成 e207-selftest-guard-broken，脚本里 guard_broken_by_the_selftest 认它）
#       the_shards_script_refuses_the_break_switch_outside_the_selftest
#   （分片核账脚本自己在参数解析之前拒平常跑时设的 E207_RUN_BREAK，自证两格：the_shards_script_refuses_the_break_switch_outside_the_selftest、the_shards_script_takes_the_break_switch_inside_the_selftest）
#   E207_RUN_BREAK=fake-driver-interval-off-by-one  假程序从第 1 片起把每片的区间起点错一位（片区间错一位）
#       sharded_product_equals_the_whole_product
#   上面的 library-lookup-ignored 与 record-skipped 同样管片块（重交时库里的片块不用、不写），另在
#       finished_shards_are_not_run_again 红
#   E207_RUN_BREAK=plan-ignores-the-shard-limit  计划不按服务给的 TASK_SERVER_SHARD_MAXIMUM 减片数
#       the_plan_writes_one_shard_per_interval_and_cuts_to_the_limit
#   E207_RUN_BREAK=plan-writes-shards-for-break-history  --break-history 的用法计划也写片
#       break_history_plans_zero_shards
#   E207_RUN_BREAK=compile-errors-swallowed  计划那一步编不过时不把编译器报的 error 正文打进日志（回到只有 cargo 的 Compiling 行与末行 could not compile）
#       a_failed_compile_prints_the_compilers_errors_up_to_the_limit
#   E207_RUN_BREAK=text-file-busy-retry-off  片执行测试程序撞 Text file busy（退 126）时不重试（计划那一步在别的台经共享目录刚写完测试程序，本机的片紧接着执行会撞；正常是 5 次重试、每次隔 3 秒）
#       a_shard_that_meets_text_file_busy_is_tried_again_and_a_stuck_one_is_named
#   下面一项管片数（memory_readings 的按项表、item_needs_a_whole_machine）：
#   E207_RUN_BREAK=whole-machine-items-stay-sharded  紧凑盘上量到接近一整台的项也照表分片
#       a_point_that_needs_a_whole_machine_gets_one_shard
#   E207_RUN_BREAK=break-history-prefix-kept  --break-history 的产物不去掉 libtest 接在第一行前的「test <用例名> ... 」（name=point 行行首对不上 E207RESULT）
#       a_break_history_product_keeps_only_the_result_lines_and_starts_at_the_point
#   E207_RUN_BREAK=process-sample-off  片直接起测试程序，不套进程读数那一层（日志里没有 e207 process 行）
#       a_shard_prints_process_samples_and_keeps_the_exit_code
set -u

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
counters="$here/e207_strict_scan_steps_counters.py"
library="$here/e207_strict_scan_steps_library.py"
histories="$here/e207_strict_scan_steps_histories.rs"
window_choices="$here/e207_strict_scan_steps_window_choices.rs"   # 历史用例 #[path] 引的量窗取法，拷进副本的 tests/e207_window_choices/mod.rs
compact_device="$here/e207_strict_scan_steps_compact_device.rs"   # 历史用例 #[path] 引的紧凑盘（单元区里内容相同的扇区只存一份），拷进副本的 tests/e207_compact_device/mod.rs
shards_py="$here/e207_strict_scan_steps_shards.py"   # 服务分片：找测试程序、合并核账、片数上限
process_sampler="$here/e207_strict_scan_steps_process_sampler.py"   # 片起测试程序时套的一层：每个间隔往日志打一行进程读数（CPU、缺页、切换、cgroup 压力），不进产物
shard_count_override=""
text_file_busy_retries=5   # 片执行测试程序撞 Text file busy（退 126）时最多重试几次
text_file_busy_pause=3     # 每次重试前等几秒
registered_threads_file="$here/../../.claude/heavy-thread-default"
entry_script_key="scripts/experiment-apparatus/e207-strict-scan-steps-run.sh"   # 复用三问规范输出认的脚本名

# 项 段（登记第五节「取样点」「实现量与分段」；与历史用例的 item_ 名单同一份，自证 the_table_matches_the_histories 核）
items_table() {
  local point probe item
  for point in base files_1000 files_10000 files_1000_units_100 files_100_units_100 files_10_units_1000 files_100_units_1000 files_10_units_10000 \
    files_then_big_1000 files_then_big_10000 files_10_units_1000_unit_area_2_pow_17 files_100_units_1000_unit_area_2_pow_20; do
    echo "${point}__head first"
  done
  echo "base__plain first"
  echo "files_1000__plain first"
  for probe in cold_first create_first young_first; do echo "files_1000_units_100__head__$probe second"; done
  echo "files_30000__head third"
  for point in files_10_units_1000_unit_area_2_pow_21 files_100_units_1000_unit_area_2_pow_21 files_1000_units_1000; do
    echo "${point}__head fourth"
  done
  # 复量段（登记设计员修订 2）：四个新点加 files_100_units_100 再跑一次（钉）；files_100_units_100__head 同属第一段，同一项名两段各一行
  for point in files_56_units_100 files_100_units_100 files_178_units_100 files_316_units_100 files_562_units_100; do
    echo "${point}__head hundred_units_per_file_repeat"
  done
  # 对账分辨段（登记设计员修订 3）：第一段同名的四项再跑，打逐操作的镜像哈希；同一项名两段各一行
  for item in base__head base__plain files_1000__head files_1000__plain; do
    echo "$item reconciliation_split"
  done
}

usage() { sed -n '2,/^set -u/p' "$0" | sed '$d' >&2; exit 2; }

broken() { [ "${E207_RUN_BREAK:-}" = "$1" ]; }

# 一份产物是否做完：有 item_done 行。弄坏开关打开时不看。
product_is_done() {
  broken done-marker-ignored && return 0
  grep -q '^E207RESULT name=item_done ' "$1" 2>/dev/null
}

histories_key() {
  sha256sum "$histories" | cut -c1-16
}

window_choices_key() {
  sha256sum "$window_choices" | cut -c1-16
}

# 历史用例与它 #[path] 引的量窗取法拷进副本的 tests 目录（内容相同的不动；先写同目录 .partial 再换名）。
place_the_histories() {
  local tests_directory=$1
  mkdir -p "$tests_directory/e207_window_choices" "$tests_directory/e207_compact_device" || return 1
  if ! cmp -s "$histories" "$tests_directory/e207_strict_scan_steps_histories.rs"; then
    cp "$histories" "$tests_directory/.e207_histories.partial" && mv -f "$tests_directory/.e207_histories.partial" "$tests_directory/e207_strict_scan_steps_histories.rs" || return 1
  fi
  if ! cmp -s "$window_choices" "$tests_directory/e207_window_choices/mod.rs"; then
    cp "$window_choices" "$tests_directory/e207_window_choices/.mod.rs.partial" && mv -f "$tests_directory/e207_window_choices/.mod.rs.partial" "$tests_directory/e207_window_choices/mod.rs" || return 1
  fi
  if ! cmp -s "$compact_device" "$tests_directory/e207_compact_device/mod.rs"; then
    cp "$compact_device" "$tests_directory/e207_compact_device/.mod.rs.partial" && mv -f "$tests_directory/e207_compact_device/.mod.rs.partial" "$tests_directory/e207_compact_device/mod.rs" || return 1
  fi
}

# 新文件名：e207-strict-scan-steps-<点>-<臂>-<日期>.out，同一天再来加 -r2、-r3；已有的一个不动。
new_product_name() {
  local stem day name attempt=1
  stem="${1//__/-}"
  day=$(TZ=Asia/Tokyo date +%F)
  name="$repo/research/results/e207-strict-scan-steps-$stem-$day.out"
  while [ -e "$name" ]; do
    attempt=$((attempt + 1))
    name="$repo/research/results/e207-strict-scan-steps-$stem-$day-r$attempt.out"
  done
  printf '%s\n' "$name"
}

complete_lines() {
  if broken cut-lines-kept; then cat "$1"; else head -n "$(wc -l < "$1")" "$1"; fi
}

raw_has_finished_item() {
  complete_lines "$1" | grep -qxF "E207RESULT name=item_done item=$2"
}

raw_keyed_for_the_item() {
  local raw=$1 item=$2 key=$3
  broken harvest-ignores-the-key && return 0
  [ "$key" != - ] && grep -qxF "$(printf '%s\t%s' "$item" "$key")" "$raw.keys" 2>/dev/null
}

write_raw_keys() {
  local raw=$1 item
  shift
  for item in "$@"; do printf '%s\t%s\n' "$item" "${item_key[$item]}"; done > "$raw.keys.partial" && mv -f "$raw.keys.partial" "$raw.keys"
}

# 同一天内容相同的 diff 引用已有那份，没有才写新名。交回产物目录里的路径。
publish_diff() {
  local text=$1 existing day name attempt=1 temporary
  for existing in "$repo"/research/results/e207-strict-scan-steps-counters-diff-*.diff; do
    if [ -f "$existing" ] && cmp -s "$text" "$existing"; then printf '%s\n' "$existing"; return 0; fi
  done
  day=$(TZ=Asia/Tokyo date +%F)
  name="$repo/research/results/e207-strict-scan-steps-counters-diff-$day.diff"
  while [ -e "$name" ]; do attempt=$((attempt + 1)); name="$repo/research/results/e207-strict-scan-steps-counters-diff-$day-r$attempt.diff"; done
  temporary="$(dirname "$name")/.$(basename "$name").partial"
  cp "$text" "$temporary" && mv -f "$temporary" "$name" || return 1
  printf '%s\n' "$name"
}

# 产物头：构建、线程、历史用例的内容键、副本的 diff、这一项的键、主工作区三份被测文件的 sha256（停机条款第 3 条）与锚点行号。字段名不写 item=，判定只取带 item= 的行。
header_for() {
  local item=$1 file
  echo "E207RESULT name=header for_item=$item build=$kind profile=release opt_level=3 test_threads=$threads histories=$(histories_key) window_choices=$(window_choices_key) diff_file=$(basename "$diff_file") diff_lines=$(wc -l < "$diff_file") item_key=${item_key[$item]:--}"
  for file in transaction.rs allocation_record_tree.rs allocator.rs; do
    echo "E207RESULT name=source_sha256 file=crates/singlefs-core/src/$file sha256=$(sha256sum "$source_tree/crates/singlefs-core/src/$file" 2>/dev/null | cut -d' ' -f1)"
  done
  python3 -I "$counters" --check-anchors "$source_tree" | sed 's/^/E207RESULT /'
}

# 一项跑完（或没跑出来）交判定库那一份记下。没记成的报出来、这一项下一趟照键现判；退 75、3 留给末尾照它退。
record_item() {
  local item=$1 code
  shift
  python3 -I "$library" record --plan "$plan_file" --item "$item" "$@"
  code=$?
  [ "$code" -eq 0 ] && return 0
  echo "  ✗ 判定库那一份没记下 $item（退 $code）：产物已写，这一项下一趟照键现判"
  echo "     → 怎么办：退 75 是 prune-server 不可用（python3 prune-server/control check，修好之后整条命令重跑，库里有的项照键复用）；别的退出码照它上面那几行 ✗ 的出路改"
  case "$code" in 3|75) library_trouble=$code ;; esac
  return 1
}

# 把一份原始输出里这一项的行写成产物：先写同目录隐藏 .partial，写完换名；回读有 item_done 才交判定库记下。
write_product() {
  local item=$1 raw=$2 seconds=$3 product temporary view
  product=$(new_product_name "$item")
  temporary="$(dirname "$product")/.$(basename "$product").partial"
  broken product-written-in-place && temporary=$product
  view="$raw_directory/view.tmp"
  complete_lines "$raw" > "$view"
  { header_for "$item"; grep -E "^E207RESULT .* item=$item( |$)" "$view"; } > "$temporary"
  [ "$temporary" = "$product" ] || mv -f "$temporary" "$product" || return 1
  if ! product_is_done "$product"; then
    echo "name=item_failed item=$item"
    echo "name=item_line item=$item exit=1 judged=1 red=1 reused=0 seconds=$seconds"
    record_item "$item" --failed
    return 1
  fi
  echo "name=item_line item=$item exit=0 judged=1 red=0 reused=0 seconds=$seconds"
  record_item "$item" --product "$product"
}

# 上一次尝试（被杀）留下的原始输出里已经完整跑完、键与此刻相同的现判项：直接写成产物、入库，从 todo 里拿掉。
harvest_finished_items() {
  local item raw kept=() taken
  broken harvest-off && return 0
  for item in "${todo[@]}"; do
    taken=0
    if [ "${item_decision[$item]}" = judge ]; then
      for raw in "$raw_directory"/*.out; do
        [ -f "$raw" ] || continue
        raw_keyed_for_the_item "$raw" "$item" "${item_key[$item]}" || continue
        if raw_has_finished_item "$raw" "$item"; then
          write_product "$item" "$raw" 0 || failed=1
          echo "name=item_taken_from_an_earlier_attempt item=$item raw=$(basename "$raw")"
          taken=1
          break
        fi
      done
    fi
    [ "$taken" -eq 1 ] || kept+=("$item")
  done
  todo=("${kept[@]+"${kept[@]}"}")
}

next_raw_name() {
  local label=$1 attempt=1
  while [ -e "$raw_directory/$label-$attempt.out" ]; do attempt=$((attempt + 1)); done
  printf '%s\n' "$raw_directory/$label-$attempt.out"
}

# 一条臂一个 cargo test：同一个进程里 E207_ARM 只有一个值，项按登记线程并行；进度行另转一份到标准输出。
run_one_arm() {
  local arm=$1 raw filters=() exact=(--exact) item started
  shift
  broken filter-by-substring && exact=()
  for item in "$@"; do filters+=("item_$item"); done
  raw=$(next_raw_name "$arm")
  write_raw_keys "$raw" "$@" || return 1
  started=$SECONDS
  export E207_ARM="${arm%%__*}" E207_PROBE=standard
  case "$arm" in *__*) E207_PROBE="${arm#*__}" ;; esac
  if broken progress-not-forwarded; then
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E207 历史用例; 范围=1 个测试目标; 为什么=量固定点逐轮读数，只跑这一趟判定库里没有的项' \
      cargo test --release -p singlefs-verification --test e207_strict_scan_steps_histories -- "${filters[@]}" "${exact[@]}" --nocapture --test-threads="$threads" > "$raw" 2>&1
  else
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E207 历史用例; 范围=1 个测试目标; 为什么=量固定点逐轮读数，只跑这一趟判定库里没有的项' \
      cargo test --release -p singlefs-verification --test e207_strict_scan_steps_histories -- "${filters[@]}" "${exact[@]}" --nocapture --test-threads="$threads" 2>&1 | tee "$raw" | grep --line-buffered '^e207 \(progress\|memory\)'
  fi
  for item in "$@"; do write_product "$item" "$raw" "$((SECONDS - started))" || failed=1; done
}

# 一份副本（counts 或 plain）的主体：从 kind、work、source_tree 与这份副本的项 chosen_of_kind 起。退出码见文件头。
run_flow_of_kind() {
  kind=$1
  shift
  local item todo=() copy diff_text tests_directory plan_output plan_code finish_code ran_before item_arguments=() tag item_field decision_field key_field arm
  local -a arms=()
  local -A items_of_arm=()
  item_key=() item_decision=()
  threads=$(sed -n 's/^\([0-9][0-9]*\).*/\1/p' "$registered_threads_file" | head -n 1)
  run_flow_shell=$BASHPID
  copy="$work/$kind"
  diff_text="$work/$kind-counters.diff"
  python3 -I "$counters" --build "$kind" --source "$source_tree" --into "$copy" --diff-out "$diff_text"
  plan_code=$?
  [ "$plan_code" -eq 0 ] || return "$plan_code"
  diff_file=$(publish_diff "$diff_text") || return 1
  tests_directory="$copy/crates/singlefs-verification/tests"
  place_the_histories "$tests_directory" || return 1
  plan_file="$work/$kind-plan.json"
  plan_output="$work/$kind-plan.out"
  for item in "$@"; do item_arguments+=(--item "$item"); done
  python3 -I "$library" plan --copy "$copy" --library-root "$repo" --plan "$plan_file" "${item_arguments[@]}" > "$plan_output"
  plan_code=$?
  cat "$plan_output"
  [ "$plan_code" -eq 0 ] || { echo "name=run_done build=$kind ran=0 failed=1 plan_exit=$plan_code"; return "$plan_code"; }
  while read -r tag item_field decision_field key_field; do
    [ "$tag" = E207PLAN ] || continue
    item_decision[${item_field#item=}]=${decision_field#decision=}
    item_key[${item_field#item=}]=${key_field#key=}
  done < "$plan_output"
  for item in "$@"; do
    if [ -z "${item_decision[$item]:-}" ]; then
      echo "  ✗ 判定库那一份的计划里没有项 $item（$plan_output），不知道它该复用还是现判"
      echo "     → 怎么办：看 $plan_output 里 E207PLAN 那几行少了哪一项、e207_strict_scan_steps_library.py plan 为什么没打它；修好整条命令重跑"
      return 1
    fi
    if [ "${item_decision[$item]}" = reuse ] && ! broken done-items-rerun; then
      echo "name=item_skipped item=$item reason=library key=${item_key[$item]}"
      echo "name=item_line item=$item exit=0 judged=0 red=0 reused=1 seconds=0"
    else
      todo+=("$item")
    fi
  done
  ran_before=${#todo[@]}
  if [ "${#todo[@]}" -gt 0 ]; then
    for item in "${todo[@]}"; do rm -f "$repo"/research/results/.e207-strict-scan-steps-"${item//__/-}"-*.partial; done
    cd "$copy" || return 1
    raw_directory="$copy/.e207-raw"
    mkdir -p "$raw_directory" || return 1
    harvest_finished_items
    for item in ${todo[@]+"${todo[@]}"}; do
      arm=${item#*__}
      broken one-process-for-every-arm && arm=${todo[0]#*__}
      [ -n "${items_of_arm[$arm]:-}" ] || arms+=("$arm")
      items_of_arm[$arm]="${items_of_arm[$arm]:-} $item"
    done
    for arm in ${arms[@]+"${arms[@]}"}; do
      # shellcheck disable=SC2086  # 项名里没有空白，这里按空白切成几项
      run_one_arm "$arm" ${items_of_arm[$arm]}
    done
    cd "$repo" || return 1
  fi
  python3 -I "$library" finish --plan "$plan_file"
  finish_code=$?
  [ "$finish_code" -eq 0 ] || failed=1
  echo "name=run_done build=$kind items=$# ran=$ran_before failed=$failed"
  [ "$library_trouble" -eq 0 ] || return "$library_trouble"
  return "$failed"
}

# 入口的主体：从 chosen 起，plain 那几项走另一份副本（只拷不插），别的走 counts。
run_flow() {
  local item counts_items=() plain_items=() rc=0 one
  failed=0 library_trouble=0
  declare -gA item_key=() item_decision=()
  for item in "${chosen[@]}"; do
    if [ "${item##*__}" = plain ]; then plain_items+=("$item"); else counts_items+=("$item"); fi
  done
  if [ "${#counts_items[@]}" -gt 0 ]; then
    run_flow_of_kind counts "${counts_items[@]}"
    one=$?
    [ "$one" -eq 0 ] || rc=$one
  fi
  if [ "${#plain_items[@]}" -gt 0 ]; then
    run_flow_of_kind plain "${plain_items[@]}"
    one=$?
    [ "$rc" -ne 0 ] || rc=$one
  fi
  return "$rc"
}

# ---- 服务分片（文件头「分片」）：计划、各片、合并三步 ----

read_threads() {
  sed -n 's/^\([0-9][0-9]*\).*/\1/p' "$registered_threads_file" | head -n 1
}

# 按项的内存表：一行一个读数，四列以制表符隔开：项（计划那一步写 plan）、片（<第几片>/<共几片>，计划那一步写 -）、盘（sparse 稀疏盘、compact 紧凑盘、any 与盘无关）、
# 量到的峰值（MiB，向上取整）。只有量到的才进表；定片数只读 compact 行（memory_largest_compact_bytes）。
# plan：编带计数的测试程序的峰值 10.71 GiB（任务 15483 第 4 次）。files_1000_units_1000__head：第 1/8 片末 26.82 GiB、第 2/8 片末 40.36 GiB，都是稀疏盘量的
# （任务 15487 第 6 次、15488 第 7 次）。files_100_units_1000_unit_area_2_pow_21__head：紧凑盘、整项一片 3.64 GiB（任务 15662 的片 15670）。紧凑盘的读数量到之后以 compact 行加进来。
memory_readings() {
  printf '%s\t%s\t%s\t%s\n' \
    plan - any 10968 \
    files_1000_units_1000__head 1/8 sparse 27464 \
    files_1000_units_1000__head 2/8 sparse 41329 \
    files_100_units_1000_unit_area_2_pow_21__head 1/1 compact 3728
}

# 一个项在紧凑盘上量到的最大一片的字节数；没有 compact 行交非 0。E207_MEMORY_READINGS_FILE 设了就读那份文件代替上面的表（只给自证用）。
memory_largest_compact_bytes() {
  { if [ -n "${E207_MEMORY_READINGS_FILE:-}" ]; then cat "$E207_MEMORY_READINGS_FILE"; else memory_readings; fi; } \
    | awk -F'\t' -v key="$1" '$1 == key && $3 == "compact" && $4 > largest { largest = $4 } END { if (largest > 0) { printf "%.0f\n", largest * 1048576; exit 0 } exit 1 }'
}

# 这一台能给的内存字节数（多机配置里这一台那一项，scripts/hosts/task_memory.py 的 host_slice_cap_bytes）；E207_HOST_MEMORY_BYTES 设了就用它（只给自证用）；取不到交非 0。
host_memory_bytes() {
  if [ -n "${E207_HOST_MEMORY_BYTES:-}" ]; then echo "$E207_HOST_MEMORY_BYTES"; return 0; fi
  python3 -I -c '
import os, sys
sys.path.insert(0, sys.argv[1])
import task_memory
value, _ = task_memory.host_slice_cap_bytes(os.environ)
if value is None:
    sys.exit(1)
print(value)' "$here/../hosts"
}

# 同一个点的几片各从头放一遍历史、各涨到自己那一段的峰值，同在一台上会一起涨：紧凑盘上量到的单片内存已到这一台能给的内存的四分之三以上的项，片数取 1
# （只剩一片涨，别的项的片照常排）。要的量取不到（没有 compact 行、这一台的量取不到）时不动，照 shard_count_of 的表。
item_needs_a_whole_machine() {
  local largest host
  broken whole-machine-items-stay-sharded && return 1
  largest=$(memory_largest_compact_bytes "$1") || return 1
  host=$(host_memory_bytes) || return 1
  [[ "$host" =~ ^[1-9][0-9]*$ ]] || return 1
  [ $((largest * 4)) -ge $((host * 3)) ]
}

# 一项分几片：每一片都要把前面的历史放一遍（文件头「分片」），所以只切放得久的项；--shard-count 给了就全按它（自证与小项核对用）。
# 依据（推的，不是量的）：files_1000_units_100 的整段历史超过四小时（任务 13894 累计 14421 秒停在 993 次发布）；第一段里别的大点各约一到两小时；plain 与小点几分钟。
shard_count_of() {
  local point=${1%%__*}
  if [ -n "$shard_count_override" ]; then echo "$shard_count_override"; return 0; fi
  if item_needs_a_whole_machine "$1"; then echo 1; return 0; fi
  case "$point" in
    files_1000_units_1000) echo 8 ;;
    files_30000) echo 6 ;;
    files_1000_units_100) echo 4 ;;
    files_562_units_100 | files_316_units_100) echo 2 ;;   # 复量段：线性缩放 files_1000_units_100 的整段历史（超过四小时）约 2.3 小时与 1.3 小时，推的，切两片留余量
    files_10000 | files_10_units_10000 | files_100_units_1000 | files_100_units_1000_unit_area_2_pow_20) echo 2 ;;
    *) echo 1 ;;
  esac
}

# 往计划目录写 plan.json：每片只写 label、不钉台（先写临时文件再改名）。参数是标签文件（每行一片）；0 片传 -。
write_shard_plan() {
  python3 -I - "$plan_directory" "$1" <<'PLAN_PYTHON' || { echo "  ✗ 计划目录 $plan_directory 写不进 plan.json（上面是 python 的报错）" >&2; echo "     → 怎么办：看那个目录可不可写" >&2; return 1; }
import json
import os
import sys

directory, labels_file = sys.argv[1], sys.argv[2]
labels = []
if labels_file != "-":
    with open(labels_file, encoding="utf-8") as handle:
        labels = [line.rstrip("\n") for line in handle if line.strip()]
temporary = os.path.join(directory, "plan.json.partial")
with open(temporary, "w", encoding="utf-8") as handle:
    json.dump({"shards": [{"label": label} for label in labels], "note": "E207 按历史里用户改动的序号分片：一片 = 一项的一段序号"}, handle, ensure_ascii=False)
    handle.flush()
    os.fsync(handle.fileno())
os.replace(temporary, os.path.join(directory, "plan.json"))
PLAN_PYTHON
}

# 把一份 E207PLAN 输出里的决定与键读进 item_decision、item_key。
load_the_decisions() {
  local tag item_field decision_field key_field
  while read -r tag item_field decision_field key_field; do
    [ "$tag" = E207PLAN ] || continue
    item_decision[${item_field#item=}]=${decision_field#decision=}
    item_key[${item_field#item=}]=${key_field#key=}
  done < "$1"
}

# 测试程序编一次（kind 与 copy 已设）：拷进 <计划目录>/binary-<kind>，各片用同一份。
compile_the_test_program() {
  local json="$work/$kind-cargo-artifacts.json" executable
  (
    cd "$copy" || exit 1
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E207 历史用例编译; 范围=1 个测试目标; 为什么=分片计划编一次测试程序，各片用同一份' \
      cargo test --release -p singlefs-verification --test e207_strict_scan_steps_histories --no-run --message-format=json > "$json"
  ) || {
    echo "  ✗ 历史用例编不过（$kind 副本 $copy；cargo 的 JSON 在 $json）" >&2
    broken compile-errors-swallowed || python3 -I "$shards_py" errors --limit 10 < "$json" >&2
    echo "     → 怎么办：看上面 cargo 的报错，改好之后整条命令重交（计划那一步重编）" >&2
    return 1
  }
  executable=$(python3 -I "$shards_py" executable --target e207_strict_scan_steps_histories < "$json") || return 1
  cp -- "$executable" "$plan_directory/.binary-$kind.partial" && chmod +x "$plan_directory/.binary-$kind.partial" \
    && mv -f "$plan_directory/.binary-$kind.partial" "$plan_directory/binary-$kind" || return 1
  echo "name=shard_binary build=$kind sha256=$(sha256sum "$plan_directory/binary-$kind" | cut -d' ' -f1)"
}

# 计划里一种副本（counts 或 plain）的那一半：建副本、放历史用例、查库定每项复用还是现判；有要现判的就编一次测试程序。要现判的项记进 planned（「种类 项 决定」）。
plan_of_kind() {
  local item plan_output plan_code tests_directory item_arguments=() planned_here=0
  kind=$1
  shift
  item_key=() item_decision=()
  threads=$(read_threads)
  copy="$work/$kind"
  python3 -I "$counters" --build "$kind" --source "$source_tree" --into "$copy" --diff-out "$work/$kind-counters.diff"
  plan_code=$?
  [ "$plan_code" -eq 0 ] || return "$plan_code"
  diff_file=$(publish_diff "$work/$kind-counters.diff") || return 1
  tests_directory="$copy/crates/singlefs-verification/tests"
  place_the_histories "$tests_directory" || return 1
  plan_file="$work/$kind-plan.json"
  plan_output="$work/$kind-plan.out"
  for item in "$@"; do item_arguments+=(--item "$item"); done
  python3 -I "$library" plan --copy "$copy" --library-root "$repo" --plan "$plan_file" "${item_arguments[@]}" > "$plan_output"
  plan_code=$?
  cat "$plan_output"
  [ "$plan_code" -eq 0 ] || return "$plan_code"
  load_the_decisions "$plan_output"
  for item in "$@"; do
    if [ -z "${item_decision[$item]:-}" ]; then
      echo "  ✗ 判定库那一份的计划里没有项 $item（$plan_output）"
      echo "     → 怎么办：看 $plan_output 里 E207PLAN 那几行少了哪一项；修好整条命令重跑"
      return 1
    fi
    if [ "${item_decision[$item]}" != reuse ] || broken done-items-rerun; then
      planned+=("$kind $item ${item_decision[$item]}")
      planned_here=$((planned_here + 1))
    fi
  done
  [ "$planned_here" -eq 0 ] || compile_the_test_program
}

# 计划（TASK_SERVER_SHARD_PHASE=plan）：每个要现判的项按 shard_count_of 切成几片，一片一行写进 shards.tsv，plan.json 每片一个 label。
plan_phase() {
  local item entry counts_items=() plain_items=() pairs=() counts_output pair maximum total_shards=0
  local plan_kind plan_item plan_decision slice slice_count labels="$plan_directory/.labels.partial" table="$plan_directory/.shards.tsv.partial"
  local -A wanted=()
  failed=0 library_trouble=0
  planned=()
  declare -gA item_key=() item_decision=()
  : > "$labels" && : > "$table" || return 1
  if [ "$device_test_only" -eq 1 ]; then
    write_shard_plan - || return 1
    echo "计划：--device-test 不分片（0 片），合并那一步照它跑"
    return 0
  fi
  if [ -n "$break_switch" ]; then
    if broken plan-writes-shards-for-break-history; then echo "break_history 的片" > "$labels"; write_shard_plan "$labels"; return $?; fi
    write_shard_plan - || return 1
    echo "计划：--break-history 的用法不分片（0 片），合并那一步照它跑"
    return 0
  fi
  # 脚本永远不报自己要多少内存
  for item in "${chosen[@]}"; do
    if [ "${item##*__}" = plain ]; then plain_items+=("$item"); else counts_items+=("$item"); fi
  done
  if [ "${#counts_items[@]}" -gt 0 ]; then plan_of_kind counts "${counts_items[@]}" || return $?; fi
  if [ "${#plain_items[@]}" -gt 0 ]; then plan_of_kind plain "${plain_items[@]}" || return $?; fi
  for entry in ${planned[@]+"${planned[@]}"}; do
    read -r plan_kind plan_item plan_decision <<< "$entry"
    pairs+=("$plan_item=$(shard_count_of "$plan_item")")
  done
  maximum=${TASK_SERVER_SHARD_MAXIMUM:-64}
  broken plan-ignores-the-shard-limit && maximum=100000
  if [ "${#pairs[@]}" -gt 0 ]; then
    counts_output=$(python3 -I "$shards_py" counts --total "$maximum" "${pairs[@]}") || return 1
    while read -r pair; do wanted[${pair%=*}]=${pair#*=}; done <<< "$counts_output"
  fi
  for entry in ${planned[@]+"${planned[@]}"}; do
    read -r plan_kind plan_item plan_decision <<< "$entry"
    slice_count=${wanted[$plan_item]}
    for ((slice = 0; slice < slice_count; slice++)); do
      printf '%s\t%s\t%s\t%s\t%s\n' "$plan_kind" "$plan_item" "$slice" "$slice_count" "$plan_decision" >> "$table"
      printf '%s 第 %s/%s 片（历史序号区间）\n' "$plan_item" "$((slice + 1))" "$slice_count" >> "$labels"
      total_shards=$((total_shards + 1))
    done
  done
  mv -f "$table" "$plan_directory/shards.tsv" || return 1
  write_shard_plan "$labels" || return 1
  echo "计划：$total_shards 片（要现判的 ${#planned[@]} 项；每项几片：${pairs[*]:-无}→按上限 $maximum 收后 $(for item in "${!wanted[@]}"; do printf '%s=%s ' "$item" "${wanted[$item]}"; done)），不钉台、由服务挑机器"
  echo "name=run_done build=plan items=${#chosen[@]} ran=${#planned[@]} failed=$failed"
}

# 一片（TASK_SERVER_SHARD_PHASE=shard）：TASK_SERVER_SHARD=<序>/<总> 取 shards.tsv 的一行。库里有这一片的块就用；没有就用计划拷好的测试程序起这一片，
# 片文件（E207RESULT 行与末行的片尾标记）写进服务给的产物目录并按片键入库。
shard_phase() {
  local spec=${TASK_SERVER_SHARD:-} index count line shard_kind item slice total decision arm probe binary out_directory raw raw_attempt=1 started code record_code busy_attempts
  if ! [[ "$spec" =~ ^([0-9]+)/([1-9][0-9]*)$ ]]; then
    echo "  ✗ 服务的 shard 那一步没给片号（TASK_SERVER_SHARD=${spec:-没设}）" >&2
    echo "     → 怎么办：这一步只由 task-server 起；直接跑就去掉 TASK_SERVER_SHARD_PHASE" >&2
    return 2
  fi
  index=${BASH_REMATCH[1]} count=${BASH_REMATCH[2]}
  if ! [ -f "$plan_directory/shards.tsv" ] || [ "$(wc -l < "$plan_directory/shards.tsv")" -ne "$count" ]; then
    echo "  ✗ 计划目录的 shards.tsv 不在，或行数不是服务给的片数 $count（$plan_directory）" >&2
    echo "     → 怎么办：计划那一步没写全：看计划那一步的日志，重交这一件" >&2
    return 1
  fi
  line=$(sed -n "$((index + 1))p" "$plan_directory/shards.tsv")
  IFS=$'\t' read -r shard_kind item slice total decision <<< "$line"
  if [ -z "$item" ]; then echo "  ✗ shards.tsv 第 $((index + 1)) 行读不出项名" >&2; echo "     → 怎么办：重交这一件" >&2; return 1; fi
  out_directory=${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}
  if [ -z "$out_directory" ]; then echo "  ✗ 服务的 shard 那一步没给产物目录（TASK_SERVER_SHARD_OUTPUT_DIRECTORY）" >&2; echo "     → 怎么办：这一步只由 task-server 起" >&2; return 2; fi
  mkdir -p -- "$out_directory" || return 1
  plan_file="$work/$shard_kind-plan.json"
  python3 -I "$library" shard-read --plan "$plan_file" --item "$item" --shard "$slice/$total" --out "$out_directory/.shard.out.partial" > "$out_directory/.shard-read.txt" 2>&1
  code=$?
  case "$code" in
    0)
      mv -f "$out_directory/.shard.out.partial" "$out_directory/shard.out" || return 1
      echo "name=shard_reused item=$item shard=$slice/$total"
      return 0 ;;
    1) ;;
    *) cat "$out_directory/.shard-read.txt"; return "$code" ;;
  esac
  # 脚本永远不报自己要多少内存
  binary="$plan_directory/binary-$shard_kind"
  if ! [ -x "$binary" ]; then echo "  ✗ 计划目录里没有测试程序 $binary" >&2; echo "     → 怎么办：计划那一步没写全：看计划那一步的日志，重交这一件" >&2; return 1; fi
  mkdir -p "$work/shard-raw" || return 1
  while [ -e "$work/shard-raw/$item-$slice-of-$total-$raw_attempt.out" ]; do raw_attempt=$((raw_attempt + 1)); done
  raw="$work/shard-raw/$item-$slice-of-$total-$raw_attempt.out"
  arm=${item#*__}
  probe=standard
  case "$arm" in *__*) probe=${arm#*__} ;; esac
  started=$SECONDS
  busy_attempts=0
  while :; do
    (
      cd "$work/$shard_kind/crates/singlefs-verification" || exit 1
      export E207_ARM="${arm%%__*}" E207_PROBE="$probe" E207_SHARD="$slice/$total"
      if broken process-sample-off; then
        "$binary" "item_$item" --exact --nocapture --test-threads=2 2>&1 | tee "$raw" | grep --line-buffered '^e207 \(progress\|memory\)'
        exit "${PIPESTATUS[0]}"
      fi
      # 套一层进程读数（文件头「各片」）：它把测试程序的退出码原样交回，起不来时照 shell 打「<程序>: Text file busy」退 126，下面的重试照认
      python3 -I "$process_sampler" --label "$item" -- "$binary" "item_$item" --exact --nocapture --test-threads=2 2>&1 | tee "$raw" | grep --line-buffered '^e207 \(progress\|memory\|process\)'
      exit "${PIPESTATUS[0]}"
    )
    code=$?
    # 测试程序是计划那一步在第二台经共享目录刚写完的，本机的片紧接着执行会撞 Text file busy（bash 退 126、原始输出里有这句）：有限次短暂重试，只认这一个错；别的 126 与别的退出码不重试
    if [ "$code" -eq 126 ] && grep -q 'Text file busy' "$raw" && [ "$busy_attempts" -lt "$text_file_busy_retries" ] && ! broken text-file-busy-retry-off; then
      busy_attempts=$((busy_attempts + 1))
      echo "name=shard_binary_busy item=$item shard=$slice/$total attempt=$busy_attempts of=$text_file_busy_retries"
      sleep "${E207_TEXT_FILE_BUSY_PAUSE:-$text_file_busy_pause}"
      continue
    fi
    break
  done
  complete_lines "$raw" | sed -E 's/^test [^ ]+ \.\.\. //' | grep -E '^(E207RESULT|E207SHARD) ' > "$out_directory/.shard.out.partial"
  if [ "$code" -ne 0 ]; then
    echo "name=shard_failed item=$item shard=$slice/$total exit=$code raw=$raw"
    if [ "$code" -eq 126 ] && grep -q 'Text file busy' "$raw"; then
      echo "  ✗ 撞的是 Text file busy（计划那一步在别的台写完的测试程序，本机刚执行就被占着）：已重试 $busy_attempts 次（上限 $text_file_busy_retries 次、每次隔 ${E207_TEXT_FILE_BUSY_PAUSE:-$text_file_busy_pause} 秒），仍然占着"
    fi
    echo "  ✗ $item 第 $slice/$total 片的测试程序退了 $code（原始输出 $raw）"
    echo "     原始输出最后 20 行："
    tail -n 20 "$raw" | sed 's/^/     | /'
    echo "     → 怎么办：看原始输出末尾的 panic；同一条命令重交，库里有的片照键复用、只补这一片"
    return 1
  fi
  python3 -I "$library" shard-record --plan "$plan_file" --item "$item" --shard "$slice/$total" --product "$out_directory/.shard.out.partial"
  record_code=$?
  case "$record_code" in
    0) ;;
    3 | 75) return "$record_code" ;;
    *)
      echo "name=shard_failed item=$item shard=$slice/$total exit=0 raw=$raw"
      echo "  ✗ $item 第 $slice/$total 片的输出末行不是片尾标记（原始输出 $raw）"
      echo "     → 怎么办：看原始输出，测试程序没打片尾标记就退 0 多半是 E207_SHARD 没传进去；同一条命令重交"
      return 1 ;;
  esac
  mv -f "$out_directory/.shard.out.partial" "$out_directory/shard.out" || return 1
  echo "name=shard_done item=$item shard=$slice/$total seconds=$((SECONDS - started))"
}

# 合并里一种副本的那一半：复用的项照旧报；要现判的项核各片、拼正文、加产物头、照项键入库。
merge_of_kind() {
  local item index shard_kind shard_item slice total decision body product temporary merge_output ran=0
  local -a files=()
  kind=$1
  shift
  item_key=() item_decision=()
  threads=$(read_threads)
  diff_file=$(publish_diff "$work/$kind-counters.diff") || return 1
  plan_file="$work/$kind-plan.json"
  load_the_decisions "$work/$kind-plan.out"
  for item in "$@"; do
    if [ -z "${item_decision[$item]:-}" ]; then
      echo "  ✗ 计划那一步留下的 $work/$kind-plan.out 里没有项 $item"
      echo "     → 怎么办：计划与合并要用同一个 --work 和同一组项；整条命令重交"
      return 1
    fi
    if [ "${item_decision[$item]}" = reuse ] && ! broken done-items-rerun; then
      echo "name=item_skipped item=$item reason=library key=${item_key[$item]}"
      echo "name=item_line item=$item exit=0 judged=0 red=0 reused=1 seconds=0"
      continue
    fi
    ran=$((ran + 1))
    files=()
    total=0
    for index in "${!shard_lines[@]}"; do
      IFS=$'\t' read -r shard_kind shard_item slice total decision <<< "${shard_lines[$index]}"
      [ "$shard_kind" = "$kind" ] && [ "$shard_item" = "$item" ] || continue
      files+=("${shard_directory[$index]}/shard.out")
    done
    body="$work/merged-$item.body"
    rm -f -- "$body"
    if [ "${#files[@]}" -eq 0 ] || ! merge_output=$(python3 -I "$shards_py" merge --item "$item" --count "${#files[@]}" --out "$body" "${files[@]}"); then
      echo "${merge_output:-}"
      echo "name=item_failed item=$item"
      echo "name=item_line item=$item exit=1 judged=1 red=1 reused=0 seconds=0 shards=${#files[@]}"
      echo "  ✗ $item 的各片合并不出产物（片数 ${#files[@]}）：不写产物，这一项记没做完"
      echo "     → 怎么办：看上面每一处 ✗；缺片或区间对不上就同一条命令重交（库里有的片照键复用），对不上多半是历史用例与 notes_of_the_history 不一致：两边都查"
      record_item "$item" --failed
      failed=1
      continue
    fi
    echo "$merge_output"
    product=$(new_product_name "$item")
    temporary="$(dirname "$product")/.$(basename "$product").partial"
    { header_for "$item"; cat -- "$body"; } > "$temporary" && mv -f "$temporary" "$product" || return 1
    if ! product_is_done "$product"; then
      echo "name=item_failed item=$item"
      echo "name=item_line item=$item exit=1 judged=1 red=1 reused=0 seconds=0 shards=${#files[@]}"
      record_item "$item" --failed
      failed=1
      continue
    fi
    echo "name=item_line item=$item exit=0 judged=1 red=0 reused=0 seconds=0 shards=${#files[@]}"
    record_item "$item" --product "$product" || failed=1
  done
  python3 -I "$library" finish --plan "$plan_file"
  [ "$?" -eq 0 ] || failed=1
  echo "name=run_done build=$kind items=$# ran=$ran failed=$failed"
}

# 合并（TASK_SERVER_SHARD_PHASE=merge）：核每一片都在，再按种类合并。--break-history 的用法照它自己的流程跑。
merge_phase() {
  local outputs=${TASK_SERVER_SHARD_OUTPUTS:-} missing=${TASK_SERVER_SHARD_MISSING-} item index host directory rc=0 one counts_items=() plain_items=()
  failed=0 library_trouble=0
  declare -gA item_key=() item_decision=() shard_directory=()
  declare -ga shard_lines=()
  if [ "$device_test_only" -eq 1 ]; then device_test; return $?; fi
  if [ -n "$break_switch" ]; then break_history "$break_switch" "${requested[0]}"; return $?; fi
  if [ -n "$missing" ]; then
    echo "  ✗ 缺片：$missing（服务说这几片没有产物）：不合并、不出产物"
    echo "     → 怎么办：同一条命令重交，库里有的片照键复用、只补缺的"
    return 1
  fi
  if [ -f "$plan_directory/shards.tsv" ]; then
    while IFS= read -r line; do shard_lines+=("$line"); done < "$plan_directory/shards.tsv"
  fi
  if [ -n "$outputs" ] && [ -f "$outputs" ]; then
    while IFS=$'\t' read -r index host directory; do shard_directory[$index]=$directory; done < "$outputs"
  fi
  for index in "${!shard_lines[@]}"; do
    if [ -z "${shard_directory[$index]:-}" ] || ! [ -f "${shard_directory[$index]}/shard.out" ]; then
      echo "  ✗ 第 $index 片没有产物（TASK_SERVER_SHARD_OUTPUTS 里没有它，或产物目录里没有 shard.out）：不合并、不出产物"
      echo "     → 怎么办：同一条命令重交，库里有的片照键复用、只补缺的"
      return 1
    fi
  done
  for item in "${chosen[@]}"; do
    if [ "${item##*__}" = plain ]; then plain_items+=("$item"); else counts_items+=("$item"); fi
  done
  if [ "${#counts_items[@]}" -gt 0 ]; then
    merge_of_kind counts "${counts_items[@]}"
    one=$?
    [ "$one" -eq 0 ] || rc=$one
  fi
  if [ "${#plain_items[@]}" -gt 0 ]; then
    merge_of_kind plain "${plain_items[@]}"
    one=$?
    [ "$rc" -ne 0 ] || rc=$one
  fi
  [ "$rc" -eq 0 ] || return "$rc"
  [ "$library_trouble" -eq 0 ] || return "$library_trouble"
  return "$failed"
}

# 入口的主体：直接跑整项整趟；经服务时按 TASK_SERVER_SHARD_PHASE 换成计划、一片、合并。
dispatch() {
  local phase=${TASK_SERVER_SHARD_PHASE:-}
  plan_directory=${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}
  case "$phase" in
    "")
      if [ "$device_test_only" -eq 1 ]; then device_test
      elif [ -n "$break_switch" ]; then break_history "$break_switch" "${requested[0]}"
      else run_flow; fi ;;
    plan | shard | merge)
      if [ -z "$plan_directory" ] || ! [ -d "$plan_directory" ]; then
        echo "  ✗ 服务的 $phase 那一步没给在的计划目录（TASK_SERVER_SHARD_PLAN_DIRECTORY=${plan_directory:-没设}）" >&2
        echo "     → 怎么办：这一步只由 task-server 起；直接跑就去掉 TASK_SERVER_SHARD_PHASE" >&2
        return 2
      fi
      "${phase}_phase" ;;
    *)
      echo "  ✗ TASK_SERVER_SHARD_PHASE=$phase 认不出（认 plan、shard、merge）" >&2
      echo "     → 怎么办：这个变量只由 task-server 设；直接跑就去掉它" >&2
      return 2 ;;
  esac
}

# ---- 自证：假驱动（cargo 换成读模式文件的函数，计数脚本跑真的、拷一棵小的假仓根；算键与判定库跑真的，判定库是临时 prune-server），不编译、不跑实现 ----

# 模拟这一趟被系统杀掉：点名两个进程，不在循环里发；假 cargo 跑在管道里，先发 run_flow 那个、再发自己。
simulate_this_run_being_killed() {
  kill -KILL "${run_flow_shell:?}"
  kill -KILL "$BASHPID"
}

# 假 cargo：把参数记进 calls.log、把 E207_ARM 记进 arms.log；按 $world_directory/mode 决定怎么跑。点名的 item_<项> 各打 row 与 item_done 两行。
#   ok 全跑完；rows-differ 全跑完、row 那一行的读数换一个；kill-immediately 一开始就被杀；kill-after-first 第一项跑完、第二项打一行就被杀；
#   kill-cut-line 打出一行被截断的 item_done 就被杀。
fake_cargo() {
  local mode filter name index=0 filters=() argument seen_separator=0 reading=1
  for argument in "$@"; do
    if [ "$argument" = --no-run ]; then fake_cargo_no_run; return $?; fi
  done
  echo "$*" >> "$calls_log"
  echo "${E207_ARM:-unset}" >> "$world_directory/arms.log"
  mode=$(cat "$world_directory/mode")
  [ "$mode" = rows-differ ] && reading=2
  for argument in "$@"; do
    if [ "$seen_separator" -eq 1 ]; then case "$argument" in --*) ;; *) filters+=("$argument") ;; esac; fi
    [ "$argument" = "--" ] && seen_separator=1
  done
  printf 'e207 progress item=fake publishes_done=1 of=2 peak_rounds=1 peak_W=1\n'
  [ "$mode" = kill-immediately ] && simulate_this_run_being_killed
  if [ "$mode" = use-binary ]; then
    for filter in "${filters[@]}"; do E207_FAKE_WORLD="$world_directory" "$world_directory/fake-histories" "$filter" --exact --nocapture; done
    return 0
  fi
  for filter in "${filters[@]}"; do
    name=${filter#item_}
    index=$((index + 1))
    if [ "$mode" = kill-cut-line ]; then
      printf 'E207RESULT name=item_done item=%s' "${name%??}"
      simulate_this_run_being_killed
    fi
    printf 'E207RESULT name=row item=%s reading=%s\n' "$name" "$reading"
    if [ "$mode" = kill-after-first ] && [ "$index" -eq 2 ]; then simulate_this_run_being_killed; fi
    printf 'E207RESULT name=item_done item=%s\n' "$name"
  done
  return 0
}

# 假 cargo test --no-run --message-format=json：编译次数记进 compile.log，交一行 compiler-artifact 指到假测试程序。
fake_cargo_no_run() {
  local number errors
  echo "no-run" >> "$world_directory/compile.log"
  if [ "$(cat "$world_directory/mode")" = compile-error ]; then
    errors=$(cat "$world_directory/fake-errors")
    for ((number = 1; number <= errors; number++)); do
      printf '{"reason":"compiler-message","package_id":"fake","message":{"level":"error","message":"fake compile error number %s","rendered":"error: fake compile error number %s\\n   --> crates/singlefs-core/src/allocator.rs:%s:9\\n    |\\n%s |         SlotPairCount(\\n    |         ^^^^\\n"}}\n' \
        "$number" "$number" "$((873 + number))" "$((873 + number))"
    done
    printf '{"reason":"compiler-message","package_id":"fake","message":{"level":"error","message":"aborting due to %s previous errors","rendered":"error: aborting due to %s previous errors\\n"}}\n' "$errors" "$errors"
    printf '{"reason":"build-finished","success":false}\n'
    return 101
  fi
  printf '{"reason":"compiler-artifact","package_id":"fake","target":{"kind":["test"],"name":"e207_strict_scan_steps_histories"},"executable":"%s"}\n' "$world_directory/fake-histories"
  printf '{"reason":"build-finished","success":true}\n'
}

# 假测试程序（分片的各片起的是它）：照历史用例的分片语义打——E207_SHARD=<片>/<片数> 时第 0 片打 point，每片打序号 (from, to] 的读数，非最后一片到 to 打片尾标记就退出，
# 最后一片接着打历史汇总、量窗行、item_done 和片尾标记；不设 E207_SHARD 是不分片，全打、没有片尾标记。from = 片号×总数÷片数。E207_FAKE_DRIVER 是格自己设的假程序毛病：
#   gap 第 1 片的区间起点错一位；short-total 最后一片的汇总行少算一个用户改动。弄坏开关 fake-driver-interval-off-by-one：从第 1 片起每片的起点错一位。
write_fake_histories() {
  cat > "$1" <<'FAKE_HISTORIES'
#!/usr/bin/env bash
set -u
name=${1#item_}
world=${E207_FAKE_WORLD:?}
total=$(cat "$world/fake-notes")
if [ -s "$world/busy-left" ] && [ "$(cat "$world/busy-left")" -gt 0 ]; then
  # 假的「刚写完的测试程序被占着」：bash 执行不了时打 <路径>: Text file busy 并退 126（说的话可由 busy-says 换成别的）
  echo $(($(cat "$world/busy-left") - 1)) > "$world/busy-left"
  echo "$0: $(cat "$world/busy-says" 2>/dev/null || echo 'Text file busy')" >&2
  echo "attempt" >> "$world/busy-calls.log"
  exit 126
fi
# 格要一段跑得久的片（进程读数那一层要打得出间隔行）时写 fake-sleep：先睡这么多秒再打
if [ -s "$world/fake-sleep" ]; then sleep "$(cat "$world/fake-sleep")"; fi
echo "$name ${E207_SHARD:-whole}" >> "$world/binary-calls.log"
shard=${E207_SHARD:-0/1}
index=${shard%/*}
count=${shard#*/}
from=$((index * total / count))
to=$(((index + 1) * total / count))
if [ "$index" -ge 1 ] && { [ "${E207_RUN_BREAK:-}" = fake-driver-interval-off-by-one ] || { [ "${E207_FAKE_DRIVER:-}" = gap ] && [ "$index" -eq 1 ]; }; }; then from=$((from + 1)); fi
reported=$total
[ "${E207_FAKE_DRIVER:-}" = short-total ] && reported=$((total - 1))
[ "$index" -eq 0 ] && echo "E207RESULT name=point item=$name point=fake notes=$total"
for ((note = 1; note <= total; note++)); do
  if [ "$note" -gt "$from" ] && [ "$note" -le "$to" ]; then
    echo "E207RESULT name=history_settle item=$name settle_call=$note rounds=3 reading=$((note * 7 % 5))"
  fi
  if [ "$note" -eq "$to" ] && [ "$((index + 1))" -lt "$count" ]; then
    echo "E207SHARD name=shard_done item=$name shard=$index/$count notes_from=$from notes_to=$to settle_calls_from=$from settle_calls_to=$to notes_total=$total"
    exit 0
  fi
done
echo "E207RESULT name=history item=$name user_changes=$reported settle_calls=$total peak_rounds=3 peak_round_work=9"
echo "E207RESULT name=row item=$name phase=window state=same_mount operation=W1 reading=9"
echo "E207RESULT name=item_done item=$name"
[ -n "${E207_SHARD:-}" ] && echo "E207SHARD name=shard_done item=$name shard=$index/$count notes_from=$from notes_to=$to settle_calls_from=$from settle_calls_to=$total notes_total=$total"
exit 0
FAKE_HISTORIES
  chmod +x "$1"
}

# 假历史用例：两个小项共用一段历史（复用三问改的就是它），大项另走一段，再加一条别的臂；入口照真的那样拷进副本、在副本上算键。
write_selftest_histories() {
  cat > "$1" <<'EOF'
//! 入口自证用的假历史用例：两个小项共用一段历史，大项另走一段。
use singlefs_core::scan_step;

fn shared_history_of_the_small_items() -> u64 {
    scan_step() + 1
}

fn history_of_the_large_item() -> u64 {
    scan_step() + 2
}

#[test]
fn item_unlinks_1__head() {
    assert_eq!(shared_history_of_the_small_items(), 2);
}

#[test]
fn item_unlinks_100__head() {
    assert_eq!(shared_history_of_the_small_items() + 1, 3);
}

#[test]
fn item_unlinks_1000__head() {
    assert_eq!(history_of_the_large_item(), 3);
}

#[test]
fn item_unlinks_1__control() {
    assert_eq!(shared_history_of_the_small_items(), 2);
}
EOF
}

# 假仓根（计数脚本写的那一棵）上补成一个能算键的工作区：两个包，实现包只有一个 scan_step；计数脚本插的那几份文件不在任何模块树里，不进任何项的闭包。
write_selftest_workspace() {
  local root=$1
  mkdir -p "$root/crates/singlefs-verification/src" "$root/crates/singlefs-core/src" || return 1
  cat > "$root/Cargo.toml" <<'EOF'
[workspace]
members = ["crates/singlefs-verification", "crates/singlefs-core"]
exclude = ["prune-server"]
resolver = "2"

[profile.release]
overflow-checks = true
EOF
  cat > "$root/crates/singlefs-verification/Cargo.toml" <<'EOF'
[package]
name = "singlefs-verification"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters", "test-only-publish-thresholds"] }
EOF
  printf 'pub fn harness_marker() -> u32 {\n    1\n}\n' > "$root/crates/singlefs-verification/src/lib.rs" || return 1
  cat > "$root/crates/singlefs-core/Cargo.toml" <<'EOF'
[package]
name = "singlefs-core"
version = "0.1.0"
edition = "2021"

[features]
lookup-work-counters = []
test-only-publish-thresholds = []
EOF
  printf 'pub fn scan_step() -> u64 {\n    1\n}\n' > "$root/crates/singlefs-core/src/lib.rs"
}

# 只答版本的 rustc 与 cargo（算键要 rustc -vV 的 host: 与 cargo -V）；起测试的 cargo 是 shell 函数 cargo（fake_cargo），不经 PATH。
write_version_only_toolchain() {
  local tools=$1
  mkdir -p "$tools" || return 1
  cat > "$tools/rustc" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  -V) echo "rustc 0.0.0-e207-selftest" ;;
  -vV) printf 'rustc 0.0.0-e207-selftest\nbinary: rustc\nhost: x86_64-unknown-linux-gnu\nrelease: 0.0.0-e207-selftest\n' ;;
esac
exit 0
EOF
  cat > "$tools/cargo" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  -V|--version) echo "cargo 0.0.0-e207-selftest" ;;
esac
exit 0
EOF
  chmod +x "$tools/rustc" "$tools/cargo"
}

replace_in_file() {
  local file=$1 old=$2 new=$3 text
  text=$(cat "$file") || return 1
  case "$text" in *"$old"*) ;; *) echo "自证样本 $file 里没有「$old」"; return 1 ;; esac
  printf '%s\n' "${text/"$old"/"$new"}" > "$file"
}

# 一间自证用的小世界：假仓根、假历史用例、假源仓根（补成能算键的工作区）、只答版本的工具链、工作目录、调用记录、模式文件。哨兵条数默认取得很大：键相同的全当哨兵、不抽强制现判。
world() {
  world_directory=$1
  mkdir -p "$world_directory/repo/research/results" "$world_directory/src" || return 1
  repo="$world_directory/repo"
  histories="$world_directory/histories.rs"
  write_selftest_histories "$histories" || return 1
  source_tree="$world_directory/src"
  python3 -I "$counters" --write-fake-source "$source_tree" || return 1
  write_selftest_workspace "$source_tree" || return 1
  write_version_only_toolchain "$world_directory/tools" || return 1
  export PATH="$world_directory/tools:$PATH" CARGO_HOME="$world_directory/tools/cargo-home" SINGLEFS_REUSE_SENTINEL_COUNT=1000
  unset SINGLEFS_REUSE_SENTINEL_SEED E207_MEMORY_READINGS_FILE E207_HOST_MEMORY_BYTES   # 自证这一趟自己设的读数文件与台内存不能带进各格
  work="$world_directory/work"
  write_fake_histories "$world_directory/fake-histories" || return 1
  printf '12\n' > "$world_directory/fake-notes"
  : > "$world_directory/binary-calls.log"
  : > "$world_directory/compile.log"
  calls_log="$world_directory/calls.log"
  : > "$calls_log"
  : > "$world_directory/arms.log"
  set_mode ok
}

set_mode() { printf '%s\n' "$1" > "$world_directory/mode"; }

# 跑一遍入口主体（子 shell：被杀只杀这一遍）。输出进 $world_directory/last.out，退出码进 last_exit_code。
go() {
  chosen=("$@")
  { ( run_flow ) > "$world_directory/last.out" 2>&1; last_exit_code=$?; } 2>/dev/null
}

calls() { wc -l < "$calls_log" | tr -d ' '; }
products() { find "$repo/research/results" -maxdepth 1 -name "e207-strict-scan-steps-${1//__/-}-*.out" | wc -l | tr -d ' '; }
latest_product() { find "$repo/research/results" -maxdepth 1 -name "e207-strict-scan-steps-${1//__/-}-*.out" -printf '%T@ %p\n' | sort -n | tail -n 1 | cut -d' ' -f2-; }

stored_in_library() {
  local arguments=() item
  for item in "$@"; do arguments+=(--item "$item"); done
  python3 -I "$library" stored --copy "$work/counts" --library-root "$repo" "${arguments[@]}" > "$world_directory/stored.out" 2>&1 || { cat "$world_directory/stored.out"; return 1; }
}

plan_directly() {
  local arguments=() item
  for item in "$@"; do arguments+=(--item "$item"); done
  python3 -I "$library" plan --copy "$work/counts" --library-root "$repo" --plan "$world_directory/direct-plan.json" "${arguments[@]}" > "$world_directory/direct-plan.out" 2>&1 \
    || { echo "plan 该退 0"; cat "$world_directory/direct-plan.out"; return 1; }
}
planned_sentinels() { sed -n 's/^E207SENTINEL .* sentinels=\([^ ]*\) forced=.*$/\1/p' "$1"; }
planned_forced() { sed -n 's/^E207SENTINEL .* forced=\([^ ]*\)$/\1/p' "$1"; }
finish_directly() {
  python3 -I "$library" finish --plan "$world_directory/direct-plan.json" > "$world_directory/finish.out" 2>&1
  finish_exit_code=$?
}

# 假服务：照 task-server/task_server_lib.py 文件头「分片」逐步起这一份的计划、各片、合并（每步一个子 shell，环境变量照协议）。
# 一次提交 = svc_new_round 开一个新轮（新计划目录、空的产物登记），再 svc_phase plan、svc_run_shard <序>…、svc_merge [缺的片号…]。
svc_new_round() {
  svc_round=$((${svc_round:-0} + 1))
  svc_plan="$world_directory/svc/round-$svc_round/plan"
  svc_outputs="$world_directory/svc/round-$svc_round/outputs.tsv"
  rm -rf "$world_directory/svc/round-$svc_round"
  mkdir -p "$svc_plan" || return 1
  : > "$svc_outputs"
}

# <阶段> [变量=值…]：输出进 $world_directory/svc-<阶段>.out，退出码进 phase_exit_code。
svc_phase() {
  local phase=$1
  shift
  { (
      export TASK_SERVER_SHARD_PHASE="$phase" TASK_SERVER_SHARD_PLAN_DIRECTORY="$svc_plan"
      export E207_FAKE_WORLD="$world_directory"   # 假测试程序认的小世界；真服务不设，入口也不读
      if [ "$#" -gt 0 ]; then export "$@"; fi
      dispatch
    ) > "$world_directory/svc-$phase.out" 2>&1
    phase_exit_code=$?
  } 2>/dev/null
}

# <序>：起 shards.tsv 的第 <序> 片；做完（退 0）的登记进这一轮的产物清单。
svc_run_shard() {
  local index=$1 count directory
  directory="$world_directory/svc/round-$svc_round/output-$index"
  count=$(wc -l < "$svc_plan/shards.tsv" | tr -d ' ')
  svc_phase shard TASK_SERVER_SHARD="$index/$count" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$directory"
  [ "$phase_exit_code" -ne 0 ] || printf '%s\t127.0.0.1\t%s\n' "$index" "$directory" >> "$svc_outputs"
}

svc_merge() {
  svc_phase merge TASK_SERVER_SHARD_OUTPUTS="$svc_outputs" TASK_SERVER_SHARD_MISSING="$*"
}

binary_calls() { wc -l < "$world_directory/binary-calls.log" | tr -d ' '; }

# 计划 → 全部片 → 合并走一遍；哪一步没退 0 就把那一步的输出打出来、退 1。
svc_full_run() {
  local index count
  svc_new_round || return 1
  svc_phase plan
  [ "$phase_exit_code" -eq 0 ] || { echo "计划该退 0，得 $phase_exit_code"; cat "$world_directory/svc-plan.out"; return 1; }
  count=$(wc -l < "$svc_plan/shards.tsv" | tr -d ' ')
  for ((index = 0; index < count; index++)); do
    svc_run_shard "$index"
    [ "$phase_exit_code" -eq 0 ] || { echo "第 $index 片该退 0，得 $phase_exit_code"; cat "$world_directory/svc-shard.out"; return 1; }
  done
  svc_merge
}

# 登记 12 条内的小世界里，分片与不分片两条路出同一份产物（文件头「分片」）：不分片走原路径（假 cargo 起假测试程序），分片走假服务三步，两份产物逐字节相同。
case_sharded_product_equals_the_whole_product() {
  local whole sharded
  set_mode use-binary
  go unlinks_1__head
  [ "$last_exit_code" -eq 0 ] || { echo "不分片的一遍该跑完（0），得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  whole=$(latest_product unlinks_1__head)
  [ -n "$whole" ] && cp "$whole" "$world_directory/whole.out" || { echo "不分片的一遍该有产物"; return 1; }
  # 另一份仓根与工作目录：判定库按仓根分，不然头一遍的判定让这一遍直接复用
  repo="$world_directory/repo-sharded"
  work="$world_directory/work-sharded"
  mkdir -p "$repo/research/results" || return 1
  shard_count_override=3
  chosen=(unlinks_1__head)
  svc_full_run || return 1
  [ "$phase_exit_code" -eq 0 ] || { echo "合并该退 0，得 $phase_exit_code"; cat "$world_directory/svc-merge.out"; return 1; }
  sharded=$(latest_product unlinks_1__head)
  [ -n "$sharded" ] || { echo "合并之后该有产物"; cat "$world_directory/svc-merge.out"; return 1; }
  cmp -s "$world_directory/whole.out" "$sharded" || { echo "分片合并出的产物与不分片的不同："; diff "$world_directory/whole.out" "$sharded" | head -n 12; return 1; }
  [ "$(binary_calls)" -eq 4 ] || { echo "该起 1 次不分片 + 3 片，得 $(binary_calls) 次"; return 1; }
  grep -q 'name=shard_merge item=unlinks_1__head shards=3 notes=12 settle_calls=12 ' "$world_directory/svc-merge.out" || { echo "合并该报 3 片、12 个序号"; cat "$world_directory/svc-merge.out"; return 1; }
  stored_in_library unlinks_1__head || { echo "合并出的整项该照项键进了判定库"; return 1; }
}

# 被打断后重交只补没判的片：首次提交只做完 0、1 片（第 2 片没做），合并红、不出产物；再次重交，0、1 片从库里取、只起第 2 片。
case_finished_shards_are_not_run_again() {
  local index
  shard_count_override=3
  chosen=(unlinks_1__head)
  svc_new_round || return 1
  svc_phase plan
  [ "$phase_exit_code" -eq 0 ] || { echo "计划该退 0"; cat "$world_directory/svc-plan.out"; return 1; }
  for index in 0 1; do svc_run_shard "$index"; done
  [ "$(binary_calls)" -eq 2 ] || { echo "头一轮该起 2 次测试程序，得 $(binary_calls)"; return 1; }
  svc_merge 2
  [ "$phase_exit_code" -eq 1 ] && [ "$(products unlinks_1__head)" -eq 0 ] || { echo "缺第 2 片，合并该退 1、不出产物（得 $phase_exit_code，产物 $(products unlinks_1__head) 份）"; cat "$world_directory/svc-merge.out"; return 1; }
  svc_full_run || return 1
  [ "$phase_exit_code" -eq 0 ] || { echo "第二轮合并该退 0，得 $phase_exit_code"; cat "$world_directory/svc-merge.out"; return 1; }
  [ "$(binary_calls)" -eq 3 ] || { echo "第二轮只该补第 2 片，测试程序共该起 3 次，得 $(binary_calls)"; cat "$world_directory/binary-calls.log"; return 1; }
  [ "$(products unlinks_1__head)" -eq 1 ] || { echo "第二轮该出恰好一份产物"; return 1; }
}

# 缺一片判红、不出产物：服务报缺（TASK_SERVER_SHARD_MISSING）与产物清单里少一行各试一次。
case_a_missing_shard_turns_the_merge_red() {
  shard_count_override=3
  chosen=(unlinks_1__head)
  svc_full_run || return 1
  [ "$(products unlinks_1__head)" -eq 1 ] || { echo "全齐的一轮该出产物"; return 1; }
  rm -f "$(latest_product unlinks_1__head)"
  sed -i '/^1\t/d' "$svc_outputs"
  svc_merge 1
  [ "$phase_exit_code" -eq 1 ] && [ "$(products unlinks_1__head)" -eq 0 ] || { echo "服务报缺第 1 片，合并该退 1、不出产物（得 $phase_exit_code）"; cat "$world_directory/svc-merge.out"; return 1; }
  svc_merge
  [ "$phase_exit_code" -eq 1 ] && [ "$(products unlinks_1__head)" -eq 0 ] || { echo "产物清单里没有第 1 片，合并该退 1、不出产物（得 $phase_exit_code）"; cat "$world_directory/svc-merge.out"; return 1; }
  grep -q '第 1 片没有产物' "$world_directory/svc-merge.out" || { echo "该点名缺的是第 1 片"; cat "$world_directory/svc-merge.out"; return 1; }
}

# 区间错一位（假测试程序让第 1 片的起点比第 0 片的终点晚一个）：合并红、不出产物、点名没有首尾相接。
case_shards_that_do_not_meet_turn_the_merge_red() {
  shard_count_override=3
  chosen=(unlinks_1__head)
  export E207_FAKE_DRIVER=gap
  svc_full_run
  [ "$phase_exit_code" -eq 1 ] && [ "$(products unlinks_1__head)" -eq 0 ] || { echo "区间没有首尾相接，合并该退 1、不出产物（得 $phase_exit_code）"; cat "$world_directory/svc-merge.out"; return 1; }
  grep -q '区间没有首尾相接（漏了 1 个）' "$world_directory/svc-merge.out" || { echo "该点名漏了 1 个"; cat "$world_directory/svc-merge.out"; return 1; }
}

# 总数对不上（最后一片的汇总行少算一个用户改动）：合并红、不出产物。
case_the_totals_must_add_up() {
  shard_count_override=3
  chosen=(unlinks_1__head)
  export E207_FAKE_DRIVER=short-total
  svc_full_run
  [ "$phase_exit_code" -eq 1 ] && [ "$(products unlinks_1__head)" -eq 0 ] || { echo "汇总行的 user_changes 少 1，合并该退 1、不出产物（得 $phase_exit_code）"; cat "$world_directory/svc-merge.out"; return 1; }
  grep -q 'name=history 行的 user_changes=11，不等于总数 12' "$world_directory/svc-merge.out" || { echo "该点名 user_changes=11 不等于 12"; cat "$world_directory/svc-merge.out"; return 1; }
}

# 计划：每项按片数写片、顺序是项的次序再片号；总片数超过服务给的上限时片数大的项一片一片往下减。
case_the_plan_writes_one_shard_per_interval_and_cuts_to_the_limit() {
  local planned
  shard_count_override=2
  chosen=(unlinks_1__head unlinks_100__head)
  svc_new_round || return 1
  svc_phase plan
  [ "$phase_exit_code" -eq 0 ] || { echo "计划该退 0"; cat "$world_directory/svc-plan.out"; return 1; }
  planned=$(cut -f2,3,4 "$svc_plan/shards.tsv" | tr '\t\n' ' /' )
  [ "$planned" = "unlinks_1__head 0 2/unlinks_1__head 1 2/unlinks_100__head 0 2/unlinks_100__head 1 2/" ] || { echo "该是两项各两片、按项再按片号：得 $planned"; return 1; }
  [ "$(python3 -I -c 'import json,sys; print(len(json.load(open(sys.argv[1]))["shards"]))' "$svc_plan/plan.json")" -eq 4 ] || { echo "plan.json 该有 4 片"; return 1; }
  [ "$(binary_calls)" -eq 0 ] || { echo "计划不起测试程序"; return 1; }
  [ "$(wc -l < "$world_directory/compile.log" | tr -d ' ')" -eq 1 ] || { echo "计划该只编一次测试程序（要现判的都在 counts 副本里）"; return 1; }
  svc_new_round || return 1
  svc_phase plan TASK_SERVER_SHARD_MAXIMUM=3
  [ "$phase_exit_code" -eq 0 ] || { echo "上限 3 的计划该退 0"; cat "$world_directory/svc-plan.out"; return 1; }
  planned=$(cut -f2,3,4 "$svc_plan/shards.tsv" | tr '\t\n' ' /' )
  [ "$planned" = "unlinks_1__head 0 1/unlinks_100__head 0 2/unlinks_100__head 1 2/" ] || { echo "上限 3：第一项减到 1 片、第二项仍 2 片，得 $planned"; return 1; }
}

# 计划那一步编不过：编译器的每条 error 的正文进日志（前 10 条），余下报条数；退非 0。
case_a_failed_compile_prints_the_compilers_errors_up_to_the_limit() {
  local out="$world_directory/svc-plan.out"
  set_mode compile-error
  printf '12\n' > "$world_directory/fake-errors"
  chosen=(unlinks_1__head)
  svc_new_round || return 1
  svc_phase plan
  [ "$phase_exit_code" -ne 0 ] || { echo "编不过的计划该退非 0"; return 1; }
  grep -q 'error: fake compile error number 1$' "$out" || { echo "第 1 条 error 的正文该在日志里"; cat "$out"; return 1; }
  grep -q 'allocator.rs:874:9' "$out" && grep -q '874 |         SlotPairCount(' "$out" || { echo "error 的文件:行与源码那一行该在日志里"; cat "$out"; return 1; }
  grep -q 'error: fake compile error number 10$' "$out" || { echo "第 10 条该打"; cat "$out"; return 1; }
  ! grep -q 'fake compile error number 11' "$out" || { echo "第 11 条起不打"; cat "$out"; return 1; }
  grep -q '余下 2 条没打' "$out" || { echo "该报余下 2 条"; cat "$out"; return 1; }
  ! grep -q 'aborting due to' "$out" || { echo "汇总行 aborting 不算一条 error"; cat "$out"; return 1; }
}

# 片执行测试程序撞 Text file busy（退 126、原始输出有这句）：头几次占着、之后正常的，重试后做完、只起一次真程序；一直占着的，用尽上限（5 次重试，共 6 次尝试）后退 1、日志写明撞的是这个错；
# 同样退 126 而说的是别的话的，一次不重试。
case_a_shard_that_meets_text_file_busy_is_tried_again_and_a_stuck_one_is_named() {
  local output="$world_directory/svc-shard.out"
  export E207_TEXT_FILE_BUSY_PAUSE=0
  shard_count_override=2
  chosen=(unlinks_1__head)
  svc_new_round || return 1
  svc_phase plan
  [ "$phase_exit_code" -eq 0 ] || { echo "计划该退 0"; cat "$world_directory/svc-plan.out"; return 1; }
  printf '3\n' > "$world_directory/busy-left"
  svc_run_shard 0
  [ "$phase_exit_code" -eq 0 ] || { echo "头 3 次占着、之后正常的片该退 0，得 $phase_exit_code"; cat "$output"; return 1; }
  [ "$(grep -c 'name=shard_binary_busy ' "$output")" -eq 3 ] || { echo "该报 3 次重试"; cat "$output"; return 1; }
  [ "$(binary_calls)" -eq 1 ] || { echo "真程序该只起 1 次，得 $(binary_calls)"; return 1; }
  grep -q 'name=shard_done item=unlinks_1__head shard=0/2 ' "$output" || { echo "该做完这一片"; cat "$output"; return 1; }
  printf '99\n' > "$world_directory/busy-left"
  : > "$world_directory/busy-calls.log"
  svc_run_shard 1
  [ "$phase_exit_code" -eq 1 ] || { echo "一直占着的片该退 1，得 $phase_exit_code"; cat "$output"; return 1; }
  [ "$(wc -l < "$world_directory/busy-calls.log" | tr -d ' ')" -eq 6 ] || { echo "该尝试 6 次（1 次 + 5 次重试），得 $(wc -l < "$world_directory/busy-calls.log")"; cat "$output"; return 1; }
  grep -q '撞的是 Text file busy' "$output" || { echo "日志该写明撞的是 Text file busy"; cat "$output"; return 1; }
  printf '99\n' > "$world_directory/busy-left"
  printf 'Permission denied\n' > "$world_directory/busy-says"
  : > "$world_directory/busy-calls.log"
  svc_run_shard 1
  [ "$phase_exit_code" -eq 1 ] && [ "$(wc -l < "$world_directory/busy-calls.log" | tr -d ' ')" -eq 1 ] || { echo "别的 126 一次都不该重试（得退出码 $phase_exit_code、尝试 $(wc -l < "$world_directory/busy-calls.log") 次）"; cat "$output"; return 1; }
  ! grep -q '撞的是 Text file busy' "$output" || { echo "别的 126 不该写成 Text file busy"; cat "$output"; return 1; }
}

# 片起测试程序时套进程读数那一层（文件头「各片」）：跑得比间隔久的片，日志（转到标准输出的那一份）有间隔行、恰好一行退出行，读数是数；
# 原始输出里也有，片文件里没有；退出码照测试程序：一直撞 Text file busy（退 126）的片照旧重试 5 次、退 1、写明撞的是什么。
case_a_shard_prints_process_samples_and_keeps_the_exit_code() {
  local output="$world_directory/svc-shard.out" raw found_raw=""
  export E207_PROCESS_SAMPLE_SECONDS=0.2 E207_TEXT_FILE_BUSY_PAUSE=0
  printf '1\n' > "$world_directory/fake-sleep"
  shard_count_override=2
  chosen=(unlinks_1__head)
  svc_new_round || return 1
  svc_phase plan
  [ "$phase_exit_code" -eq 0 ] || { echo "计划该退 0"; cat "$world_directory/svc-plan.out"; return 1; }
  svc_run_shard 0
  [ "$phase_exit_code" -eq 0 ] || { echo "睡 1 秒的片该退 0，得 $phase_exit_code"; cat "$output"; return 1; }
  grep -Eq '^e207 process item=unlinks_1__head stage=sample wall_seconds=[0-9]+ user_seconds=[0-9.]+ system_seconds=[0-9.]+ minor_faults=[0-9]+ ' "$output" \
    || { echo "睡 1 秒、间隔 0.2 秒的片，日志里该有间隔行"; cat "$output"; return 1; }
  [ "$(grep -c '^e207 process item=unlinks_1__head stage=end ' "$output")" -eq 1 ] || { echo "该恰好一行退出行"; cat "$output"; return 1; }
  grep -Eq '^e207 process item=unlinks_1__head stage=end .* voluntary_switches=[0-9]+ involuntary_switches=[0-9]+ ' "$output" \
    || { echo "退出行的切换次数该是数"; cat "$output"; return 1; }
  for raw in "$work"/shard-raw/unlinks_1__head-0-of-2-*.out; do [ -f "$raw" ] && found_raw=$raw; done
  [ -n "$found_raw" ] && grep -q '^e207 process item=unlinks_1__head stage=end ' "$found_raw" || { echo "原始输出里该留着进程读数（$found_raw）"; return 1; }
  ! grep -q 'e207 process' "$world_directory/svc/round-$svc_round/output-0/shard.out" || { echo "进程读数不该进片文件"; return 1; }
  rm -f "$world_directory/fake-sleep"
  printf '99\n' > "$world_directory/busy-left"
  : > "$world_directory/busy-calls.log"
  svc_run_shard 1
  [ "$phase_exit_code" -eq 1 ] || { echo "一直占着的片该退 1，得 $phase_exit_code"; cat "$output"; return 1; }
  [ "$(wc -l < "$world_directory/busy-calls.log" | tr -d ' ')" -eq 6 ] || { echo "套了那一层之后仍该尝试 6 次，得 $(wc -l < "$world_directory/busy-calls.log")"; cat "$output"; return 1; }
  grep -q '撞的是 Text file busy' "$output" || { echo "日志该写明撞的是 Text file busy"; cat "$output"; return 1; }
}

# 真编不过留下的编译记录（任务 15183 的计划子任务，三条 error 指到 allocator.rs:874–876）喂给 errors 子命令：三条的正文与文件:行都打出来，不报余下。
case_the_real_compile_record_prints_its_three_errors() {
  local data="$here/e207-selftest-real-compile-errors.json" out="$world_directory/real-errors.out"
  python3 -I "$shards_py" errors --limit 10 < "$data" > "$out" || { echo "errors 子命令该退 0"; cat "$out"; return 1; }
  grep -q '编译器报了 3 条 error，下面打前 3 条' "$out" || { echo "该报 3 条"; cat "$out"; return 1; }
  grep -q 'allocator.rs:874:9' "$out" && grep -q 'allocator.rs:875:103' "$out" && grep -q 'allocator.rs:876:124' "$out" || { echo "三条 error 的文件:行该都在"; cat "$out"; return 1; }
  grep -q '874 |         SlotPairCount(' "$out" || { echo "874 行的源码该在"; cat "$out"; return 1; }
  grep -q 'error\[E0061\]: this struct takes 1 argument but 3 arguments were supplied' "$out" || { echo "E0061 的正文该在"; cat "$out"; return 1; }
  ! grep -q '余下' "$out" || { echo "只有 3 条，不该报余下"; cat "$out"; return 1; }
}

# --break-history 的用法不分片：计划写 0 片，不建副本、不编、不写 shards.tsv。
case_break_history_plans_zero_shards() {
  break_switch=units_of_not_refreshed
  requested=(base__head)
  chosen=(base__head)
  svc_new_round || return 1
  svc_phase plan
  [ "$phase_exit_code" -eq 0 ] || { echo "计划该退 0"; cat "$world_directory/svc-plan.out"; return 1; }
  [ "$(python3 -I -c 'import json,sys; print(len(json.load(open(sys.argv[1]))["shards"]))' "$svc_plan/plan.json")" -eq 0 ] || { echo "plan.json 该是 0 片"; cat "$svc_plan/plan.json"; return 1; }
  [ ! -e "$svc_plan/shards.tsv" ] && [ ! -e "$work/counts" ] && [ "$(wc -l < "$world_directory/compile.log" | tr -d ' ')" -eq 0 ] || { echo "不分片的用法不该建副本、编测试程序、写 shards.tsv"; return 1; }
}

# --break-history 的产物只留 E207RESULT 行、第一行是 name=point：原始日志开头是编译输出，libtest 单线程时把「test <用例名> ... 」接在 point 行前面；
# 产物里不许有编译输出、测试框架的行与那个前缀（对拍的读数器认行首的 E207RESULT，对不上就报「没有 name=point 行」）。
case_a_break_history_product_keeps_only_the_result_lines_and_starts_at_the_point() {
  local log="$world_directory/break.log" product="$world_directory/break.product"
  {
    printf '   Compiling singlefs-verification v0.1.0 (/somewhere)\n'
    printf 'warning: for loop over an `Option`\n'
    printf '    Finished `release` profile [optimized] target(s) in 1m 02s\n'
    printf 'running 1 test\n'
    printf 'test item_base__head ... E207RESULT name=point item=base__head point=base notes=3\n'
    printf 'E207RESULT name=history_settle item=base__head settle_call=1 rounds=3 reading=2\n'
    printf 'e207 progress item=base__head publishes_done=1 of=2 peak_rounds=1 peak_W=1\n'
    printf 'E207RESULT name=item_done item=base__head\n'
    printf 'ok\n'
    printf 'test result: ok. 1 passed; 0 failed\n'
  } > "$log"
  break_history_product "$log" "$product" || { echo "产物写不出来"; return 1; }
  [ "$(head -n 1 "$product")" = 'E207RESULT name=point item=base__head point=base notes=3' ] || { echo "产物第一行该是去掉前缀的 name=point 行"; cat "$product"; return 1; }
  [ "$(wc -l < "$product" | tr -d ' ')" -eq 3 ] && [ "$(grep -vc '^E207RESULT ' "$product")" -eq 0 ] || { echo "产物该只剩 3 条 E207RESULT 行"; cat "$product"; return 1; }
  grep -qx 'E207RESULT name=item_done item=base__head' "$product" || { echo "item_done 行该在"; cat "$product"; return 1; }
}

# 紧凑盘上量到的单片内存到这一台能给的内存的四分之三以上的项，片数取 1；没到、只有稀疏盘的读数、这一台的量取不到都照 shard_count_of 的表（files_10000 两片、files_1000_units_100 四片）。
case_a_point_that_needs_a_whole_machine_gets_one_shard() {
  local readings="$world_directory/memory-readings.tsv" count
  export E207_MEMORY_READINGS_FILE="$readings" E207_HOST_MEMORY_BYTES=53687091200
  : > "$readings"
  count=$(shard_count_of files_10000__head)
  [ "$count" = 2 ] || { echo "没有读数，files_10000__head 该是表里的 2 片，得 $count"; return 1; }
  printf 'files_10000__head\t1/2\tcompact\t38400\nfiles_1000_units_100__head\t2/4\tsparse\t60000\n' > "$readings"
  count=$(shard_count_of files_10000__head)
  [ "$count" = 1 ] || { echo "紧凑盘 38400 MiB 正好是 50 GiB 的四分之三，该取 1 片，得 $count"; return 1; }
  count=$(shard_count_of files_1000_units_100__head)
  [ "$count" = 4 ] || { echo "只有稀疏盘的读数不定片数，files_1000_units_100__head 该是表里的 4 片，得 $count"; return 1; }
  printf 'files_10000__head\t1/2\tcompact\t38399\n' > "$readings"
  count=$(shard_count_of files_10000__head)
  [ "$count" = 2 ] || { echo "紧凑盘 38399 MiB 没到四分之三，该是 2 片，得 $count"; return 1; }
  printf 'files_10000__head\t1/2\tcompact\t38400\n' > "$readings"
  E207_HOST_MEMORY_BYTES=不是整数
  count=$(shard_count_of files_10000__head)
  [ "$count" = 2 ] || { echo "这一台的量取不到，该照表 2 片，得 $count"; return 1; }
}

case_killed_after_the_copy_and_run_again_completes() {
  set_mode kill-immediately
  go unlinks_1__head unlinks_100__head
  [ "$last_exit_code" -eq 137 ] || { echo "第一遍该被杀（137），得 $last_exit_code"; return 1; }
  [ -f "$work/counts/.e203-copy-done" ] || { echo "被杀时副本该已拷完、有完成标记"; return 1; }
  set_mode ok
  go unlinks_1__head unlinks_100__head
  [ "$last_exit_code" -eq 0 ] || { echo "第二遍该跑完（0），得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  grep -q 'name=run_tally judged=2 forced=0 reused=0 refused=0 not_done=0 not_run=0' "$world_directory/last.out" || { echo "计账行不对"; cat "$world_directory/last.out"; return 1; }
  stored_in_library unlinks_1__head unlinks_100__head || { echo "两项该照键进了判定库"; return 1; }
}

case_a_directory_that_is_not_our_copy_is_refused_and_left_alone() {
  mkdir -p "$work/counts" && printf 'mine\n' > "$work/counts/mine.txt" || return 1
  go unlinks_1__head
  [ "$last_exit_code" -eq 2 ] || { echo "该拒绝（2），得 $last_exit_code"; return 1; }
  [ -f "$work/counts/mine.txt" ] || { echo "不是我们拷的东西不许清"; return 1; }
  [ "$(calls)" -eq 0 ] || { echo "拒绝时不许起 cargo"; return 1; }
}

case_one_finished_item_is_not_run_again() {
  go unlinks_1__head unlinks_100__head
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] && [ "$(products unlinks_1__head)" -eq 1 ] || { echo "第一遍该跑一次 cargo、各一份产物"; cat "$world_directory/last.out"; return 1; }
  go unlinks_1__head unlinks_100__head
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "第二遍不该再起 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  grep -q 'name=run_tally judged=0 forced=0 reused=2 refused=0' "$world_directory/last.out" || { echo "第二遍该现判 0、复用 2"; cat "$world_directory/last.out"; return 1; }
  [ "$(products unlinks_1__head)" -eq 1 ] || { echo "复用的项不另写产物"; return 1; }
}

case_an_item_finished_inside_a_killed_run_is_taken_and_not_run_again() {
  set_mode kill-after-first
  go unlinks_1__head unlinks_100__head
  [ "$last_exit_code" -eq 137 ] || { echo "第一遍该被杀，得 $last_exit_code"; return 1; }
  set_mode ok
  go unlinks_1__head unlinks_100__head
  [ "$last_exit_code" -eq 0 ] || { echo "第二遍该跑完，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  [ "$(calls)" -eq 2 ] || { echo "该共起两次 cargo"; return 1; }
  ! sed -n 2p "$calls_log" | grep -q 'item_unlinks_1__head ' || { echo "第二次不该再点名跑完的那一项"; sed -n 2p "$calls_log"; return 1; }
  sed -n 2p "$calls_log" | grep -q 'item_unlinks_100__head' || { echo "第二次该点名没跑完的那一项"; return 1; }
  stored_in_library unlinks_1__head unlinks_100__head || { echo "两项该照键进了判定库（取来的那一项也入库）"; return 1; }
}

case_an_item_finished_inside_a_killed_run_under_another_key_is_run_again() {
  set_mode kill-after-first
  go unlinks_1__head unlinks_100__head
  [ "$last_exit_code" -eq 137 ] || { echo "第一遍该被杀，得 $last_exit_code"; return 1; }
  replace_in_file "$histories" 'scan_step() + 1' 'scan_step() + 3' || return 1
  set_mode ok
  go unlinks_1__head unlinks_100__head
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "第二遍该跑完、共起两次 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  sed -n 2p "$calls_log" | grep -q 'item_unlinks_1__head ' && sed -n 2p "$calls_log" | grep -q 'item_unlinks_100__head' \
    || { echo "键变了，第二次该把两项都点名"; sed -n 2p "$calls_log"; return 1; }
}

case_one_line_cut_by_a_kill_does_not_finish_an_item() {
  set_mode kill-cut-line
  go unlinks_1__head
  [ "$last_exit_code" -eq 137 ] || { echo "第一遍该被杀，得 $last_exit_code"; return 1; }
  set_mode ok
  # 残行的项名少了末两个字符；项名短一截时它正好是别的项名，这里手写一份完整项名的残行
  printf 'E207RESULT name=item_done item=unlinks_1__head' > "$work/counts/.e207-raw/head-9.out"
  printf 'unlinks_1__head\t%s\n' "$(python3 -I "$library" keys --copy "$work/counts" --item unlinks_1__head | sed -n 's/.* key=\([^ ]*\).*/\1/p')" > "$work/counts/.e207-raw/head-9.out.keys"
  go unlinks_1__head
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "残行不算跑完，第二遍该再起 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
}

case_the_item_filter_names_one_test_only() {
  go unlinks_1__head
  sed -n 1p "$calls_log" | grep -q -- ' --exact ' || { echo "点名该带 --exact"; sed -n 1p "$calls_log"; return 1; }
}

case_one_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name() {
  { ( header_for() { kill -KILL "$BASHPID"; }; chosen=(unlinks_1__head); run_flow ) > "$world_directory/last.out" 2>&1; last_exit_code=$?; } 2>/dev/null
  [ "$last_exit_code" -eq 137 ] || { echo "第一遍该在写产物时被杀，得 $last_exit_code"; return 1; }
  [ "$(products unlinks_1__head)" -eq 0 ] || { echo "被杀时产物名下不该有文件"; return 1; }
  go unlinks_1__head
  [ "$last_exit_code" -eq 0 ] || { echo "第二遍该跑完，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  [ "$(calls)" -eq 1 ] || { echo "原始输出已跑完，第二遍不该再起 cargo（calls=$(calls)）"; return 1; }
  [ "$(products unlinks_1__head)" -eq 1 ] || { echo "该恰好一份产物"; return 1; }
  [ -z "$(find "$repo/research/results" -maxdepth 1 -name '.*.partial')" ] || { echo "残留的隐藏 .partial 该已清掉"; return 1; }
}

case_progress_lines_reach_the_standard_output() {
  go unlinks_1__head
  grep -q '^e207 progress item=fake publishes_done=1 of=2 peak_rounds=1 peak_W=1$' "$world_directory/last.out" || { echo "进度行该转到标准输出"; cat "$world_directory/last.out"; return 1; }
  product=$(latest_product unlinks_1__head)
  ! grep -q '^e207 progress' "$product" || { echo "进度行不进产物"; return 1; }
}

case_the_product_header_records_the_tested_files() {
  go unlinks_1__head
  product=$(latest_product unlinks_1__head)
  [ "$(grep -c '^E207RESULT name=source_sha256 ' "$product")" -eq 3 ] || { echo "产物头该记三份被测文件的 sha256"; cat "$product"; return 1; }
  grep -q '^E207RESULT name=header for_item=unlinks_1__head ' "$product" || { echo "产物头该有 header 行"; cat "$product"; return 1; }
}

case_each_arm_runs_in_its_own_process() {
  go unlinks_1__head unlinks_1__control
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "两条臂该起两次 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  [ "$(sed -n 1p "$world_directory/arms.log")" = head ] && [ "$(sed -n 2p "$world_directory/arms.log")" = control ] || { echo "每次的 E207_ARM 该是那条臂"; cat "$world_directory/arms.log"; return 1; }
  sed -n 1p "$calls_log" | grep -q 'item_unlinks_1__head ' && ! sed -n 1p "$calls_log" | grep -q 'item_unlinks_1__control' || { echo "一次 cargo 只点名一条臂的项"; cat "$calls_log"; return 1; }
}

case_the_plain_item_goes_to_a_copy_without_the_counters() {
  go unlinks_1__plain
  [ -f "$work/plain/.e203-copy-done" ] || { echo "plain 该有自己的副本"; cat "$world_directory/last.out"; return 1; }
  ! grep -q e207_strict_scan_steps "$work/plain/crates/singlefs-core/src/transaction.rs" || { echo "plain 的副本里不该有读数"; return 1; }
  grep -q 'pub fn build_name' "$work/plain/crates/singlefs-verification/tests/e207_readings/mod.rs" || { echo "plain 的副本要有读数的空实现"; return 1; }
}

# 复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：头一趟库是空的、同一份输入、闭包外改动、闭包内改动。
case_reuse_three_questions() {
  local runs="$world_directory/three-questions"
  mkdir -p "$runs" || return 1
  go unlinks_1__head unlinks_100__head unlinks_1000__head
  cp "$world_directory/last.out" "$runs/first.out" || return 1
  go unlinks_1__head unlinks_100__head unlinks_1000__head
  cp "$world_directory/last.out" "$runs/second.out" || return 1
  printf '\n// 历史用例里加的一行注释\nfn spare_history_nobody_runs() -> u64 {\n    9\n}\n' >> "$histories" || return 1
  printf '// 源码变了的一行\n' >> "$source_tree/crates/singlefs-core/src/transaction.rs" || return 1
  go unlinks_1__head unlinks_100__head unlinks_1000__head
  cp "$world_directory/last.out" "$runs/unrelated.out" || return 1
  replace_in_file "$histories" 'scan_step() + 1' 'scan_step() + 3' || return 1
  go unlinks_1__head unlinks_100__head unlinks_1000__head
  cp "$world_directory/last.out" "$runs/unit-change.out" || return 1
  python3 -I "$here/../gate/lib_reuse_checks.py" three-questions --script "$entry_script_key" --first "$runs/first.out" --second "$runs/second.out" \
    --unrelated "$runs/unrelated.out" --unit-change "$runs/unit-change.out" --affected unlinks_1__head,unlinks_100__head --from-output
}

# 键不随副本的路径变：同一份副本拷到两个别的目录，键逐字相同；工作目录换到别处，库里判过的照键复用。
case_the_item_key_does_not_depend_on_the_copy_path() {
  local one="$world_directory/one/place" another="$world_directory/another/place/deeper"
  go unlinks_1__head unlinks_1000__head
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "头一遍该跑完、起一次 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  mkdir -p "$one" "$another" && cp -r "$work/counts/." "$one/" && cp -r "$work/counts/." "$another/" || return 1
  python3 -I "$library" keys --copy "$one" --item unlinks_1__head --item unlinks_1000__head > "$world_directory/keys-one.out" 2>&1 || { cat "$world_directory/keys-one.out"; return 1; }
  python3 -I "$library" keys --copy "$another" --item unlinks_1__head --item unlinks_1000__head > "$world_directory/keys-another.out" 2>&1 || { cat "$world_directory/keys-another.out"; return 1; }
  [ "$(grep -c '^E207KEY ' "$world_directory/keys-one.out")" -eq 2 ] && ! grep -q 'key=refused' "$world_directory/keys-one.out" || { echo "两项都该算得出键"; cat "$world_directory/keys-one.out"; return 1; }
  cmp -s "$world_directory/keys-one.out" "$world_directory/keys-another.out" || { echo "同一份源码放两个目录，键该相同"; cat "$world_directory/keys-one.out" "$world_directory/keys-another.out"; return 1; }
  work="$world_directory/elsewhere/work"
  go unlinks_1__head unlinks_1000__head
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "换了副本的路径，该照键复用、不再起 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  grep -q 'name=run_tally judged=0 forced=0 reused=2 refused=0' "$world_directory/last.out" || { echo "该现判 0、复用 2"; cat "$world_directory/last.out"; return 1; }
}

case_reuse_sentinel_draws_and_the_forced_item_is_run_again() {
  local forced
  go unlinks_1__head unlinks_100__head unlinks_1000__head
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "头一遍该跑完（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  go unlinks_1__head unlinks_100__head unlinks_1000__head
  [ "$last_exit_code" -eq 0 ] || { echo "哨兵复用、强制现判与库里相同，该退 0，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  forced=$(planned_forced "$world_directory/last.out")
  [ -n "$forced" ] && [ "$forced" != - ] || { echo "三项键相同、哨兵 1 条，该抽出一条强制现判"; cat "$world_directory/last.out"; return 1; }
  [ "$(calls)" -eq 2 ] && [ "$(sed -n 2p "$calls_log" | grep -o 'item_[a-z0-9_]*' | wc -l)" -eq 1 ] && sed -n 2p "$calls_log" | grep -q "item_$forced " \
    || { echo "第二遍该只起一次 cargo、只点名强制现判的 $forced"; cat "$calls_log"; return 1; }
  grep -q 'name=run_tally judged=1 forced=1 reused=2 refused=0 not_done=0 not_run=0' "$world_directory/last.out" || { echo "计账行不对"; cat "$world_directory/last.out"; return 1; }
}

case_one_rejudged_sentinel_turns_the_run_red() {
  local sentinel forced
  go unlinks_1__head unlinks_100__head unlinks_1000__head
  [ "$last_exit_code" -eq 0 ] || { echo "头一遍该跑完，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  plan_directly unlinks_1__head unlinks_100__head unlinks_1000__head || return 1
  sentinel=$(planned_sentinels "$world_directory/direct-plan.out")
  forced=$(planned_forced "$world_directory/direct-plan.out")
  [ -n "$sentinel" ] && [ "$sentinel" != - ] && [ -n "$forced" ] && [ "$forced" != - ] || { echo "该抽出一条哨兵、一条强制现判"; cat "$world_directory/direct-plan.out"; return 1; }
  python3 -I "$library" record --plan "$world_directory/direct-plan.json" --item "$sentinel" --product "$(latest_product "$sentinel")" || return 1
  python3 -I "$library" record --plan "$world_directory/direct-plan.json" --item "$forced" --product "$(latest_product "$forced")" || return 1
  finish_directly
  [ "$finish_exit_code" -eq 1 ] && grep -q '复用失灵' "$world_directory/finish.out" || { echo "哨兵被现判，该退 1、报复用失灵（得 $finish_exit_code）"; cat "$world_directory/finish.out"; return 1; }
}

case_one_forced_rejudge_with_another_verdict_turns_the_run_red() {
  go unlinks_1__head unlinks_100__head unlinks_1000__head
  [ "$last_exit_code" -eq 0 ] || { echo "头一遍该跑完，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  set_mode rows-differ
  go unlinks_1__head unlinks_100__head unlinks_1000__head
  [ "$last_exit_code" -eq 1 ] && grep -q '判定不稳' "$world_directory/last.out" || { echo "强制现判的读数与库里不同，该退 1、报判定不稳（得 $last_exit_code）"; cat "$world_directory/last.out"; return 1; }
}

case_an_item_planned_for_judging_but_not_run_turns_the_run_red() {
  go unlinks_1000__head
  [ "$last_exit_code" -eq 0 ] || { echo "头一遍该跑完，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  plan_directly unlinks_1__head unlinks_100__head || return 1
  finish_directly
  [ "$finish_exit_code" -eq 1 ] && grep -q '没跑到' "$world_directory/finish.out" || { echo "要现判的两项一条都没记，该退 1、报没跑到（得 $finish_exit_code）"; cat "$world_directory/finish.out"; return 1; }
}

case_an_item_without_a_key_runs_and_is_never_recorded() {
  # 假历史用例里没有 item_files_1__head：入口找不到，拒算
  go files_1__head
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "拒算的项照跑、退 0（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  grep -q 'E207PLAN item=files_1__head decision=refused key=-' "$world_directory/last.out" && grep -q 'name=run_tally judged=0 forced=0 reused=0 refused=1' "$world_directory/last.out" \
    || { echo "该报拒算 1"; cat "$world_directory/last.out"; return 1; }
  go files_1__head
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "拒算的项不入库，下一趟照跑（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
}

case_an_unreachable_library_exits_75_and_runs_nothing() {
  local port
  port=$(python3 -c 'import socket; listener = socket.socket(); listener.bind(("127.0.0.1", 0)); print(listener.getsockname()[1]); listener.close()') || return 1
  export SINGLEFS_PRUNE_SERVER_SOCKET="$world_directory/no-server.sock" SINGLEFS_PRUNE_SERVER_TCP_ADDRESS="127.0.0.1:$port" SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=2
  go unlinks_1__head
  [ "$last_exit_code" -eq 75 ] || { echo "判定库连不上该退 75，得 $last_exit_code"; return 1; }
  [ "$(calls)" -eq 0 ] || { echo "连不上时不许起 cargo（不当库里没有现判）"; return 1; }
}

selftest() {
  local failures=0 out name display reuse_line="" line real_histories=$histories real_entry=$0
  selftest_root=$(mktemp -d "${TMPDIR:-/tmp}/e207-run-selftest.XXXXXX") || exit 1
  if [ "${SINGLEFS_KEEP_TEST_TEMPORARY_FILES:-}" = 1 ]; then
    echo "name=selftest_kept path=$selftest_root"
  else
    trap 'rm -rf "$selftest_root"' EXIT
  fi
  # 判定库那一份只在有这份标记文件时认弄坏开关
  printf 'e207-selftest\n' > "$selftest_root/mark"
  export E207_SELFTEST_MARK="$selftest_root/mark"
  # 自证只在临时 prune-server 上跑：经判定库那一份调客户端 selftest_service_problem，不是临时服务退 2、一格不跑
  python3 -I "$library" selftest-service-problem || return 2
  cargo() { fake_cargo "$@"; }
  out=$(items_table | wc -l)
  [ "$out" -eq 30 ] || { echo "name=selftest_case case=item_count result=FAILED got=$out"; failures=$((failures + 1)); }
  # 复量段五项、对账分辨段四项；第一段仍是十四项；两段里与第一段同名的项各占一行，所以行数 30 对项名 25 个（21 + 4 个新点）
  [ "$(items_table | awk '$2=="hundred_units_per_file_repeat"' | wc -l)" -eq 5 ] || { echo "name=selftest_case case=hundred_units_per_file_repeat_segment_items result=FAILED"; failures=$((failures + 1)); }
  [ "$(items_table | awk '$2=="reconciliation_split"' | wc -l)" -eq 4 ] || { echo "name=selftest_case case=reconciliation_split_segment_items result=FAILED"; failures=$((failures + 1)); }
  [ "$(items_table | cut -d' ' -f1 | sort -u | wc -l)" -eq 25 ] || { echo "name=selftest_case case=distinct_item_names result=FAILED"; failures=$((failures + 1)); }
  [ "$(items_table | awk '$2=="first"' | wc -l)" -eq 14 ] || { echo "name=selftest_case case=first_segment_items result=FAILED"; failures=$((failures + 1)); }
  if [ "$(items_table | cut -d' ' -f1 | sort -u)" = "$(sed -n 's/^fn item_\(.*\)() {$/\1/p' "$real_histories" | sort)" ]; then
    echo "name=selftest_case case=the_table_matches_the_histories result=ok"
  else
    echo "name=selftest_case case=the_table_matches_the_histories result=FAILED"
    failures=$((failures + 1))
  fi
  local product_file="$selftest_root/product"
  printf 'E207RESULT name=row reading=1\n' > "$product_file"
  if product_is_done "$product_file"; then echo "name=selftest_case case=done_marker result=FAILED"; failures=$((failures + 1)); else echo "name=selftest_case case=one_product_without_the_marker_is_not_done result=ok"; fi
  printf 'E207RESULT name=item_done item=base__head\n' >> "$product_file"
  if product_is_done "$product_file"; then echo "name=selftest_case case=one_product_with_the_marker_is_done result=ok"; else echo "name=selftest_case case=done_marker result=FAILED"; failures=$((failures + 1)); fi
  if E207_RUN_BREAK='' bash "$real_entry" --item no_such_item__head --work "$selftest_root/unused" > /dev/null 2>&1; then
    echo "name=selftest_case case=an_unknown_item_name_exits_2 result=FAILED"; failures=$((failures + 1))
  else
    echo "name=selftest_case case=an_unknown_item_name_exits_2 result=ok"
  fi
  if [ "$(E207_RUN_BREAK='' bash "$real_entry" --list-items | wc -l)" -eq 30 ]; then
    echo "name=selftest_case case=list_items_lists_every_item result=ok"
  else
    echo "name=selftest_case case=list_items_lists_every_item result=FAILED"; failures=$((failures + 1))
  fi
  # 分片核账脚本平常跑时设了 E207_RUN_BREAK 要在参数解析之前被拒（退 2，点名只在自证里认）；有自证的标记文件时放行（照常往下解析、因为命令不认而退 2 但不点名）
  local guard_out guard_code
  local guard_mark=""   # 空：守卫那一趟不带标记文件；开关 shards-guard-passes-always 打开时带一份写着 e207-selftest-guard-broken 的
  if broken shards-guard-passes-always; then
    guard_mark="$selftest_root/guard-broken-mark"
    printf 'e207-selftest-guard-broken\n' > "$guard_mark"
  fi
  guard_out=$(E207_SELFTEST_MARK="$guard_mark" E207_RUN_BREAK=merge-gap-unchecked python3 -I "$shards_py" --force --break-guard-probe-unknown-option 2>&1 < /dev/null); guard_code=$?
  if [ "$guard_code" -eq 2 ] && grep -q 'E207_RUN_BREAK 只在自证里认' <<< "$guard_out"; then
    echo "name=selftest_case case=the_shards_script_refuses_the_break_switch_outside_the_selftest result=ok"
  else
    echo "name=selftest_case case=the_shards_script_refuses_the_break_switch_outside_the_selftest result=FAILED"; failures=$((failures + 1))
  fi
  guard_out=$(E207_RUN_BREAK=merge-gap-unchecked python3 -I "$shards_py" --force --break-guard-probe-unknown-option 2>&1 < /dev/null); guard_code=$?
  if [ "$guard_code" -eq 2 ] && ! grep -q 'E207_RUN_BREAK 只在自证里认' <<< "$guard_out"; then
    echo "name=selftest_case case=the_shards_script_takes_the_break_switch_inside_the_selftest result=ok"
  else
    echo "name=selftest_case case=the_shards_script_takes_the_break_switch_inside_the_selftest result=FAILED"; failures=$((failures + 1))
  fi
  # 各格各造各的小世界（自己的假仓根、工作目录、调用记录，判定库命名空间按各自的仓根分），互不依赖：按每台默认重活线程数并行起，结果按下面的固定次序打。
  local threads running=0 cases=(
    case_killed_after_the_copy_and_run_again_completes
    case_a_directory_that_is_not_our_copy_is_refused_and_left_alone
    case_one_finished_item_is_not_run_again
    case_an_item_finished_inside_a_killed_run_is_taken_and_not_run_again
    case_an_item_finished_inside_a_killed_run_under_another_key_is_run_again
    case_one_line_cut_by_a_kill_does_not_finish_an_item
    case_the_item_filter_names_one_test_only
    case_one_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name
    case_progress_lines_reach_the_standard_output
    case_the_product_header_records_the_tested_files
    case_each_arm_runs_in_its_own_process
    case_the_plain_item_goes_to_a_copy_without_the_counters
    case_reuse_three_questions
    case_the_item_key_does_not_depend_on_the_copy_path
    case_reuse_sentinel_draws_and_the_forced_item_is_run_again
    case_one_rejudged_sentinel_turns_the_run_red
    case_one_forced_rejudge_with_another_verdict_turns_the_run_red
    case_an_item_planned_for_judging_but_not_run_turns_the_run_red
    case_an_item_without_a_key_runs_and_is_never_recorded
    case_an_unreachable_library_exits_75_and_runs_nothing
    case_sharded_product_equals_the_whole_product
    case_finished_shards_are_not_run_again
    case_a_missing_shard_turns_the_merge_red
    case_shards_that_do_not_meet_turn_the_merge_red
    case_the_totals_must_add_up
    case_the_plan_writes_one_shard_per_interval_and_cuts_to_the_limit
    case_break_history_plans_zero_shards
    case_a_break_history_product_keeps_only_the_result_lines_and_starts_at_the_point
    case_a_point_that_needs_a_whole_machine_gets_one_shard
    case_a_failed_compile_prints_the_compilers_errors_up_to_the_limit
    case_the_real_compile_record_prints_its_three_errors
    case_a_shard_that_meets_text_file_busy_is_tried_again_and_a_stuck_one_is_named
    case_a_shard_prints_process_samples_and_keeps_the_exit_code
  )
  # shellcheck source=../hosts/heavy-threads.sh
  source "$here/../hosts/heavy-threads.sh"
  threads=$(heavy_thread_default) || return 2
  for name in "${cases[@]}"; do
    mkdir -p "$selftest_root/$name"
    (
      if ( world "$selftest_root/$name" && "case_${name#case_}" ) > "$selftest_root/$name/out.txt" 2>&1; then echo ok > "$selftest_root/$name/status"; else echo FAILED > "$selftest_root/$name/status"; fi
    ) &
    running=$((running + 1))
    if [ "$running" -ge "$threads" ]; then wait -n; running=$((running - 1)); fi
  done
  wait
  for name in "${cases[@]}"; do
    display=${name#case_}
    [ "$display" = reuse_three_questions ] && display=reuse-three-questions
    if [ "$(cat "$selftest_root/$name/status" 2>/dev/null)" = ok ]; then
      echo "name=selftest_case case=$display result=ok"
    else
      echo "name=selftest_case case=$display result=FAILED"
      sed 's/^/  | /' "$selftest_root/$name/out.txt" | tail -n 15
      failures=$((failures + 1))
    fi
    line=$(grep '^REUSE_SELFTEST ' "$selftest_root/$name/out.txt" | tail -n 1)
    [ -z "$line" ] || reuse_line=$line
  done
  [ -z "$reuse_line" ] || echo "$reuse_line"
  echo "name=selftest_done failures=$failures 并行 $threads 路（${#cases[@]} 格）"
  [ "$failures" -eq 0 ]
}

# 弄坏开关只在 --selftest 里认；别处设了拒（判定库那一份由自证设的标记文件放行，见 selftest）。
if [ -n "${E207_RUN_BREAK:-}" ] && [ "${1:-}" != --selftest ]; then
  echo "  ✗ E207_RUN_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "     → 怎么办：去掉 E207_RUN_BREAK 再跑；要证红就跑 E207_RUN_BREAK=<项> bash $0 --selftest（经临时 prune-server 的包装）" >&2
  exit 2
fi

# --break-history 的产物：编译输出与测试框架的行都不要，只留 E207RESULT 行。libtest 单线程带 --nocapture 时把「test <用例名> ... 」接在第一行（name=point）前面，
# 行首对不上 E207RESULT，先去掉那个前缀；弄坏开关 break-history-prefix-kept 不去。原始输出（日志）另存在 $log。
break_history_product() {
  local log=$1 product=$2 temporary
  temporary="$(dirname "$product")/.$(basename "$product").partial"
  if broken break-history-prefix-kept; then
    complete_lines "$log" | grep -E '^E207RESULT ' > "$temporary"
  else
    complete_lines "$log" | sed -E 's/^test [^ ]+ \.\.\. //' | grep -E '^E207RESULT ' > "$temporary"
  fi
  mv -f "$temporary" "$product"
}

# 登记第九节 M13：带弄坏开关起一项（mutate.sh 跑不到副本里的历史用例），读声明行判抓没抓到；不入判定库、原始输出只写进工作目录。
break_history() {
  local switch=$1 item=$2 copy tests_directory raw log declared_at_a_thousand distinct kind=counts arm=head undercounted
  case "$switch" in
    units_of_not_refreshed)
      [ "${item##*__}" = head ] || { echo "  ✗ --break-history 的项要是今天实现那条臂（<点>__head）：$item" >&2; return 2; } ;;
    slots_placed_undercounted)
      [ "$item" = base__head ] || { echo "  ✗ slots_placed_undercounted 只起 base__head 一项（登记修订 3 阳性对照 ①）：$item" >&2; return 2; } ;;
    image_witness_flipped)
      [ "$item" = base__plain ] || { echo "  ✗ image_witness_flipped 只起 base__plain 一项（登记修订 3 阳性对照 ②）：$item" >&2; return 2; }
      kind=plain arm=plain ;;
    *) echo "  ✗ --break-history 只认 units_of_not_refreshed、slots_placed_undercounted、image_witness_flipped，收到 $switch" >&2; return 2 ;;
  esac
  copy="$work/$kind"
  python3 -I "$counters" --build "$kind" --source "$source_tree" --into "$copy" --diff-out "$work/$kind-counters.diff" || return $?
  tests_directory="$copy/crates/singlefs-verification/tests"
  place_the_histories "$tests_directory" || return 1
  raw="$work/break-history-$switch-$item.out"
  log="$work/break-history-$switch-$item.log"
  (
    cd "$copy" || exit 1
    export E207_ARM=$arm E207_PROBE=standard E207_RUN_BREAK="$switch"
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E207 历史用例带弄坏开关; 范围=1 个测试目标; 为什么=登记第九节 M13 与修订 3 的阳性对照证红：弄坏一处时读数要变' \
      cargo test --release -p singlefs-verification --test e207_strict_scan_steps_histories -- "item_$item" --exact --nocapture --test-threads=1 > "$log" 2>&1
  )
  break_history_product "$log" "$raw"
  grep -q "^E207RESULT name=item_done item=$item\$" "$raw" || { echo "name=break_history switch=$switch item=$item caught=not_run raw=$raw"; return 1; }
  if [ "$switch" = slots_placed_undercounted ]; then
    # 对账行里计数记的落点槽数比盘上变了的单元槽数少的行数（模型的 placement_gap 的 negative）：base 的 W1 第 1 次原本 32 对 32，减 1 之后是 31 对 32
    undercounted=$(grep "^E207RESULT name=reconciliation item=$item " "$raw" | sed 's/.* image_changed_unit_slots=\([0-9]*\) slots_placed=\([0-9]*\).*/\1 \2/' | awk '$2 < $1' | wc -l)
    if [ "$undercounted" -ge 1 ]; then
      echo "name=break_history switch=$switch item=$item operations_with_fewer_slots_placed_than_changed=$undercounted caught=yes raw=$raw"
      return 0
    fi
    echo "name=break_history switch=$switch item=$item operations_with_fewer_slots_placed_than_changed=$undercounted caught=no raw=$raw"
    return 1
  fi
  if [ "$switch" = image_witness_flipped ]; then
    # 判在模型对拍那一步：模型读正常的 base__head 与这份坏产物，head_against_plain 的 first_differing 要是 remounted/W2/5（原始输出里的 E207RESULT 行就是产物）
    echo "name=break_history switch=$switch item=$item caught=judged_by_the_model_against_the_good_base_head raw=$raw"
    return 0
  fi
  declared_at_a_thousand=$(grep -c "^E207RESULT name=declared item=$item phase=window state=same_mount operation=W2 .* user_units=1000 " "$raw")
  distinct=$(grep "^E207RESULT name=declared item=$item phase=window state=same_mount operation=W2 .* old_units=1000 " "$raw" | wc -l)
  if [ "$declared_at_a_thousand" -gt 5 ]; then
    echo "name=break_history switch=$switch item=$item same_mount_cold_overwrite_declared_at_a_thousand=$declared_at_a_thousand distinct_files=5 caught=yes raw=$raw"
    return 0
  fi
  echo "name=break_history switch=$switch item=$item same_mount_cold_overwrite_declared_at_a_thousand=$declared_at_a_thousand first_touch_lines=$distinct distinct_files=5 caught=no raw=$raw"
  return 1
}

# 紧凑盘（e207_strict_scan_steps_compact_device.rs）的等价单测：它的单测要 singlefs-core 与验证包，只能在副本里跑；起 plain 副本（不插读数）、
# 只跑 compact_device::tests 四条（与整块稀疏盘逐字节读回相同、跨单元区起点、清零与 discard、内容只存一份、没有单元区起点时就是稀疏盘）。
device_test() {
  local copy tests_directory raw passed
  copy="$work/plain"
  python3 -I "$counters" --build plain --source "$source_tree" --into "$copy" --diff-out "$work/plain-counters.diff" || return $?
  tests_directory="$copy/crates/singlefs-verification/tests"
  place_the_histories "$tests_directory" || return 1
  raw="$work/device-test.out"
  (
    cd "$copy" || exit 1
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E207 紧凑盘的等价单测; 范围=1 个测试目标; 为什么=大点内存办法：紧凑盘与整块稀疏盘读回逐字节相同' \
      cargo test --release -p singlefs-verification --test e207_strict_scan_steps_histories -- compact_device::tests > "$raw" 2>&1
  )
  passed=$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed.*/\1/p' "$raw" | tail -n 1)
  if [ "${passed:-0}" -eq 4 ]; then
    echo "name=device_test passed=4 failed=0 raw=$raw"
    return 0
  fi
  echo "name=device_test passed=${passed:-0} expected=4 raw=$raw"
  return 1
}

work="" source_tree="$repo" segment="" requested=() chosen=() break_switch=""
list_only=0 device_test_only=0
needs_value() {
  [ "$1" -ge 2 ] && return 0
  echo "  ✗ $2 后面缺值；可点名的项：" >&2
  items_table >&2
  echo "     → 怎么办：在 $2 后面写上面列的一项，或用 --list-items 看全部" >&2
  exit 2
}
while [ $# -gt 0 ]; do
  case "$1" in
    --list-items) list_only=1; shift ;;
    --device-test) device_test_only=1; shift ;;
    --selftest) selftest; exit $? ;;
    --work) needs_value $# "$1"; work=$2; shift 2 ;;
    --source) needs_value $# "$1"; source_tree=$2; shift 2 ;;
    --segment) needs_value $# "$1"; segment=$2; shift 2 ;;
    --item) needs_value $# "$1"; requested+=("$2"); shift 2 ;;
    --break-history) needs_value $# "$1"; break_switch=$2; shift 2 ;;
    --shard-count)
      needs_value $# "$1"
      case "$2" in
        [1-9] | [1-9][0-9]) shard_count_override=$2 ;;
        *) echo "  ✗ --shard-count 要 1 到 99 的整数，收到 $2" >&2; echo "     → 怎么办：写片数，例 --shard-count 3" >&2; exit 2 ;;
      esac
      shift 2 ;;
    *) usage ;;
  esac
done

if [ "$list_only" -eq 1 ]; then items_table; exit 0; fi
for item in "${requested[@]+"${requested[@]}"}"; do
  items_table | awk -v wanted="$item" '$1==wanted {found=1} END {exit !found}' || { echo "项名写错：$item；可点名的项：" >&2; items_table >&2; exit 2; }
done
[ -n "$work" ] || usage
[ -f "$counters" ] && [ -f "$library" ] && [ -f "$histories" ] && [ -f "$window_choices" ] && [ -f "$compact_device" ] || { echo "前置文件不在：$counters、$library、$histories、$window_choices 或 $compact_device" >&2; exit 78; }

if [ -n "$segment" ]; then
  while read -r item listed_segment; do [ "$listed_segment" = "$segment" ] && chosen+=("$item"); done < <(items_table)
  [ "${#chosen[@]}" -gt 0 ] || usage
fi
for item in "${requested[@]+"${requested[@]}"}"; do
  case " ${chosen[*]+"${chosen[*]}"} " in *" $item "*) ;; *) chosen+=("$item") ;; esac
done
if [ -z "$segment" ] && [ "${#requested[@]}" -eq 0 ]; then
  while read -r item _; do chosen+=("$item"); done < <(items_table)
fi

mkdir -p "$work" && work=$(cd "$work" && pwd) || { echo "工作目录建不出：$work" >&2; exit 2; }
[ -d "$source_tree" ] || { echo "仓根不在：$source_tree" >&2; exit 78; }
source_tree=$(cd "$source_tree" && pwd)
if [ -n "$break_switch" ]; then
  [ "${#requested[@]}" -eq 1 ] && [ -z "$segment" ] || { echo "  ✗ --break-history 只配一个 --item，不配 --segment" >&2; exit 2; }
fi
dispatch
exit $?
