#!/usr/bin/env bash
# 门禁 code-tooling 的格函数库：research-script-selftests、research-script-selftest-coverage、temporary-service-runners、temporary-service-wrapper-form。
# 由 .claude/gate.d/code-tooling.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法库变量、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# 各格共用的全局变量由入口设好，别份库里的函数与变量在调用时才取。

# ── 格 research-script-selftests：runner 表里每一条研究脚本与共用库的自证都通过（分批并行跑）
#
# 判据：SELFTEST_RUNNERS 里每一条自证都退 0；表全不全归 research-script-selftest-coverage 格。lib/stage-cells.sh 不是阶段，它的自证只在这张表里跑。
#   自证退 77 是「本次未跑」（它自己造不出前提，例如 crates-mutation-rows.py 并发领片号那一格两次调用没重叠）：那一条不判绿也不判红，不记进判定库（下一趟照跑），
#   成功句与汇总里逐条列名、往 $GATE_NOT_RUN_FILE 各报一行；这一趟真起的每一条都退 77、又没有复用的，整格退 77（一条都没判过）。
#   样本 runner-not-run-green（两条，一条退 0、一条退 77）必须判绿并列出本次未跑的那一条；runner-all-not-run（两条都退 77）整格退 77。
#   弄坏开关 CODE_TOOLING_BREAK=selftest-not-run-as-pass：77 当 0，照绿记，这两份样本都判错。
#   每一条经 scripts/gate/selftest-process-guard.py 起（登记表取被判那棵树的 .claude/hooks/selftest-registry.tsv）：登记过的，
#   跑的时候它整棵进程树里起了真 cargo / rustc（版本查询之外）、真清场（gpu-quiesce-run.sh 读真多机配置）或私有清场脚本，守卫退 3，这一条照没过记；
#   经 PATH 起的 cargo、rustc 被守卫的替身当场拒、真的那一份不起。判据、射程与本次未查写在守卫文件头。别的（表外的、
#   不是单独一个 --selftest 的）守卫原样 exec、本次未盯，成功句逐条列名。每条的判定写 <序号>.guard，按表次序回读、数份数，缺一份整格红。
#
# 分批并行：SELFTEST_RUNNERS_STARTED_ALONE 里最慢的那几条先各自单独起、跑到完；其余按表次序每批 N 条并行，一批收齐再起下一批，
#   N = 每台默认重活线程数（scripts/hosts/heavy-threads.sh 的 heavy_thread_default：SINGLEFS_THREAD_CAP 设了照它，否则 .claude/heavy-thread-default 的登记值；
#   取不了这一格判红）减去单独起的条数，至少 1。
#   SELFTEST_RUNNERS_MEMORY_BOUND 里的大自证（真仓现算整张单元图的这类，一条十几 GiB；这一道整格登记在 .claude/memory-bounded-parallelism.tsv 里准许按内存定并发）各占一批：
#   排到它时先收齐手上这一批，它单独起、跑完再往下，大自证不互相叠、也不与别的自证同批；同时写在 SELFTEST_RUNNERS_STARTED_ALONE 里的照这一条办。登记的不在 runner 表里判红。
#   新量到一条十几 GiB 的自证（task-server/task status 的内存峰值），连同出处加进这张表。样本根的 .selftest-memory-bound（一行一条，只在样本根有 .selftest-runners 时认）顶替这张表。
#   样本 runner-memory-bound-batches-green：四条自证，表里 大、小 相间，.selftest-environment 设每批至多 8 条：每条开跑留一个记号、睡 1 秒，大的那条看见别的记号、小的那条看见大的记号就退 1；判绿。
#   样本 runner-memory-bound-unlisted-red：登记的一条不在样本的 runner 表里，整格判红、一条都不起。
#
# 按条复用：表里每一条自证一条判定（scripts/gate/admission.py verdict-plan / verdict-record，种类 code-tooling.sh:research-script-selftests），
#   键是那份脚本的单元闭包（它 import / source 到的模块与函数、读的数据文件、解释器版本，算法在 admission.py 文件头「单元键」；bash 起的另带
#   program-argument:<脚本之后的每个词>，脚本体只收这组参数真走到的那一支，lib_function_units.py 文件头「程序入口带参数」；python 起的整份收）、
#   这一条命令原文（line:command=）、selftest-registry.tsv 里那份脚本登记的种类（line:selftest-registration=）与判法的闭包（judge:，在这一道所在的仓里算）；不带路径与行号。
#   判法取 selftest-process-guard.py 里判盯不盯的那几个函数（SELFTEST_JUDGE_ENTRIES：认哪条命令要盯、真工具链的位置、替身、进程树与 execve 记录怎么判；
#   名单只在守卫里那一份，selftest_judge_entries_load 按 ast 读它，读不出这一格判红；样本 runner-judge-entries-unreadable-red 三条自证都退 0、样本根的
#   .selftest-judge-guard.py 顶替守卫而没有那一句，必须判红、点名读不出，三条照样跑完），
#   不取整份程序：它的 main / guard 起的是调用方给的那条命令（subprocess 的命令折不出），整份闭包必拒算、每一条都现判；被起的那条命令已由入口与 line:command= 进键。键相同、判过绿的不起，表全部复用就一条都不起；拒算的（闭包收不全）照跑、不记。
#   现判而判绿、守卫给了判定的照键记进 prune-server 那一份判定库（unit-verdicts 族，种类 singlefs-unit-verdicts/code-tooling.sh:research-script-selftests，
#   库名固定、库根不进键，scripts/gate/admission.py 文件头「判定库」；盯了而 execve 跟踪没跑的不记；样本根在临时 prune-server 底下起，另占一份命名空间）。
#   记红、撤绿、下趟现判：现判而判红的（自证退非 0、不是 77）照同一把键记红、撤掉这把键原有的绿（selftest_record_verdicts 经 admission.py verdict-record --red-names，
#   打一行「· 判红记库：…」），下一趟这一条现判、原因以「上次判红」起头；退 77 的不是判定，不记；拒算的没有键，不记。
#   一条判完就记：每条自证跑完，起它的那个后台作业写完退出码就当场照键记这一条（selftest_record_one，判法与回读同一份 selftest_outcome；
#   admission.py verdict-record 每把键一份文件、同目录临时文件写完 fsync 再改名换上），这一格中途被停，已经跑完的那几条都在库里，下一趟它们复用、只补没判的；
#   当场记的那一趟不判复用哨兵，收尾照旧把这一趟的判绿、判红名单整份再交一次 verdict-record（同一把键写同样的内容），哨兵与「· 判绿记库」那一行在收尾那一趟。
#   收尾那一趟退 0 时它打的「!」行（跑的过程中键变了、不记的）原样转进日志，至多 SELFTEST_RECORD_SKIP_LINES_SHOWN 行、多出的报「另有 N 条」（selftest_record_skips_report；
#   当场记的那几趟不转：收尾那一趟照同一份计划再比一次，变了的还在它里面）；样本 runner-record-skips-forwarded-green，弄坏开关 CODE_TOOLING_BREAK=record-skip-lines-dropped。
#   样本 runner-recorded-before-the-run-ends（两条，样本根的 .selftest-environment 设 SINGLEFS_THREAD_CAP=1：每批一条、按表次序）：
#   a 退 0；b 在样本自己的判定库（git common-dir 底下）里找 a 那一条的判绿记录，找到退 0、找不到退 1：整格收尾之前 a 已经在库里，判绿、两条都记。
#   成功句报「表里 N 条：现判 J、复用 M、拒算 R、本次未跑 0」，现判与拒算逐条列为什么（键对不上时是变了的单元），没过的那一条另报为什么现判。
#   样本根另放了 .unit-verdict-sample/prior/（上一版的树）的，先拿上一版算键、照判绿记进样本自己的判定库：runner-unit-key 三条自证、上一版只有 c 那一份不同，
#   必须只现判 c、另两条复用。
#   按条复用失灵（verdict-plan 退非 0、退 0 而没打 PLAN、它或 verdict-record 的复用哨兵报「复用失灵」「判定不稳」「强制现判没起作用」）：判法只在
#   scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」），这一格把那一趟的输出交给它；它判失灵，这一格照旧全部现判跑完、结论判红。
#   样本 runner-plan-exit-red（.unit-verdict-sample/plan-failure 写 exit-1：verdict-plan 退 1）与 runner-plan-sentinel-red（写 sentinel-reuse-failed：哨兵报复用失灵）
#   各三条自证都退 0，必须判红、点名失灵在哪，三条照样跑完。
#   样本 runner-red-over-green（记红、撤绿、下趟现判）：表两条（a 自证退 0、b 自证在样本根有开关文件 .red-switch 时退 1），.unit-verdict-sample/prior/ 是这一版逐字拷的一份
#   （上一版两条照判绿记进库，同一把键），.unit-verdict-sample/red-over-green 在：这一格的计划另设 ADMISSION_BREAK=verdict-lookup-skipped（不读库）照样两条都现判，b 判红、照键记红盖掉绿，
#   样本档接着不设它再算一次计划：复用 a、现判 b、原因以「上次判红」起头，整格判红。
#   每条的输出与退出码各写自己的文件，跑完按表次序回读、数份数（.claude/singlefs-ai-sop/rules/command-safety.md「并行不许把失败吃掉」）。
#   几份自证抢同一份状态（同一个临时路径、同一个端口）的，只在并行下红：那一条挪进 SELFTEST_RUNNERS_STARTED_ALONE 单独起，或让它的自证各用各的临时目录。
#   计划或记库退 75（prune-server 不可用）：plan-failure 判不可用、退 75（admission.py 文件头「按条复用失灵」），这一格不现判、退 75；整道由 lib/stage-cells.sh 汇总成 75。
#   样本 runner-plan-unavailable（.unit-verdict-sample/plan-failure 写 exit-75）三条自证都退 0，这一格退 75、一条都不起，整道 75、汇总写 prune-server 不可用。
#   逐项点名（这一道「逐项点名」一节）：项是 <脚本路径>，表里只留起那几份脚本的自证，算键、复用、记库照整格的走法。样本 runner-item-picked（三条里点一份，只跑那一条）、
#   runner-item-unknown（项写错，退 2）。
#
# 只在临时 prune-server 上跑的自证（.claude/hooks/selftest-temporary-service.tsv 登记成 required 的：自证开跑先调 selftest_service_problem，scripts/prune-server/prune_server_client.py 文件头「命令行」，裸跑退 2、不写真服务；
#   认哪些是 required 读这张表，不看源码里有没有那几个字，读法在 TEMPORARY_SERVICE_TABLE 的注释；登记成 own-server 的两份自己起真服务，不要求包装）：
#   表里写成包装写法「$SELFTEST_TEMPORARY_SERVICE_PREFIX <原命令>」，不写二进制；裸写的整格判红、一条都不起，点名该写成的那一句（样本 runner-bare-temporary-service-red，样本根放自己的要临时服务的表）。
#   runner 表与要临时服务的表两边对不对得上归 temporary-service-runners 格（文件头开关表里的样本）。
#   键与登记表里的登记都按 -- 后面那条原命令认（line:command= 仍是表里那一行整句），另加 file:scripts/prune-server/prune_server_client_selftest.py。
#   这一趟要起的里面有包装写法的，开跑前取一次 prune-server 二进制（code_tooling_prune_server_binary：PRUNE_SERVER_BINARY 设了用它，没设照 prune-server-selftest 格的编法
#   编进这一格的临时目录、跑完随它删；取不到整格判红），每条照「包装 --server-binary <那一份> -- 守卫 -- <原命令>」起：守卫盯的仍是原命令那一棵。
#   弄坏开关 CODE_TOOLING_BREAK=bare-temporary-service-selftest-accepted：不核写法，runner-bare-temporary-service-red 判错。
#
# 样本：每份样本都放 setup.sh 做 git init（判定库的名字取被判那棵树的 git common-dir，stage-selftest 拷出的临时目录不是 git 工作树时 admission.py verdict-plan 退 2）。
#   被判的根是样本时（judged_root_is_sample），样本根的 .selftest-runners（一行一条命令，空行与 # 开头的行不算）换掉 runner 表，
#   单独起的那一路不走；真仓与 --staged 下放了它也不认。runner-red 四条里第二条必红，必须判红、点名它并列出输出，另三条照样跑完、
#   报 4 条里 1 条没过；runner-green 三条都退 0，判绿并报跑了 3 条。
#   runner-guard-red 与 runner-guard-green 各放一份自己的 .claude/hooks/selftest-registry.tsv，把一份样本自证登记成 selftest：
#   red 那一份经 PATH 起 cargo build（清单不在），必须被守卫的替身拒、这一条退 3 判红，报 2 条里 1 条没过；green 那一份只用样本自己的假 cargo，
#   判绿并报盯了 1 条、本次未盯 1 条（另一条没登记）。
SELFTEST_RUNNERS=(
  "python3 scripts/agents/checklist-specs.py --selftest" "bash scripts/hosts/two-logs.sh --selftest" "python3 scripts/hosts/two_logs.py --selftest" "bash scripts/gate/baseline-diff.sh --selftest" "python3 scripts/kb/batch-target.py --selftest"
  "python3 scripts/kb/quote-kb.py --selftest" "python3 scripts/kb/kb-sections.py --selftest"
  "python3 scripts/crash/check-segment-registry.py --selftest" "python3 scripts/kb/replace-once.py --selftest"
  "python3 scripts/kb/replace-batch.py --selftest" "python3 scripts/experiment-apparatus/e152-tables.py --selftest"
  "python3 scripts/experiment-apparatus/e203_admission_scan_cost_counters.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e203-admission-scan-cost-run.sh --selftest"
  "python3 scripts/experiment-apparatus/e206_fixed_point_cost_counters.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e206-fixed-point-cost-run.sh --selftest"
  "python3 scripts/experiment-apparatus/e207_strict_scan_steps_counters.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e207-strict-scan-steps-run.sh --selftest"
  "python3 scripts/experiment-apparatus/e209_spike_cause_counters.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e209-spike-cause-run.sh --selftest"
  "python3 scripts/agents/agent-watch.py --selftest" "python3 scripts/kb/quote-rust-items.py --selftest"
  "python3 scripts/kb/sweep-term.py --selftest" "python3 scripts/kb/archive-past-rounds.py --selftest"
  "python3 scripts/kb/insert-row.py --selftest" "bash scripts/hosts/cache-keepalive.sh --selftest"
  "python3 scripts/kb/ledger-append-section.py --selftest" "python3 scripts/kb/append-record.py --selftest" "python3 scripts/agents/lib_session_directory.py --selftest" "python3 scripts/agents/lib_session_registry.py --selftest"
  "bash scripts/agents/watch.sh --selftest" "python3 scripts/kb/stale-candidates.py --selftest"
  "python3 scripts/kb/stale-candidates.py --benchmark" "python3 scripts/hosts/test-environment-check.py --selftest"
  "bash scripts/kb/verify-citations.sh --selftest"
  "python3 scripts/agents/agent-handover.py --selftest" "python3 scripts/kb/decision-slim-check.py --selftest"
  "python3 scripts/kb/pdf-text.py --selftest" "python3 scripts/kb/rules-sweep-audit.py --selftest"
  "python3 scripts/agents/dispatch-prompt.py --selftest" "python3 scripts/agents/three-way-materials.py --selftest" "python3 scripts/agents/verdict-skeleton.py --selftest" "python3 scripts/agents/who-touched.py --selftest"
  "python3 scripts/agents/active-sessions.py --selftest"
  "python3 scripts/gate/lib_rebuilt_packages.py --selftest"
  "python3 scripts/gate/lib_rust_test_module_files.py --selftest"
  "python3 scripts/mutation/lib_release_only_tests.py --selftest"
  "python3 scripts/gate/lib_function_units.py --selftest"
  "python3 scripts/gate/checker-direction-overlap.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/crate-cross-references-check.py --selftest"
  "bash scripts/agents/three-way-rerun-template.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest"
  "python3 scripts/agents/agent-cost.py --selftest" "bash scripts/devices/snapshot-check.sh --selftest" "bash scripts/devices/e152-run.sh --selftest"
  "python3 scripts/agents/subagent_transcript.py --selftest"
  "bash scripts/gate/changed-paths.sh --selftest"
  "bash scripts/gate/gate-staged.sh --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/admission.py --selftest"
  "bash scripts/hosts/capped.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/mutation/mutation-shard-run.sh --selftest"
  "bash scripts/hosts/multi-host-run.sh --selftest" "bash scripts/hosts/run-with-memory-cap.sh --selftest" "bash scripts/hosts/run-build-directory.sh --selftest" "python3 scripts/agents/copy-originals.py --selftest"
  "python3 task-server/task_server_lib.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/replay.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/mutation/mutate.sh --selftest" "python3 scripts/agents/cite-check.py --selftest"
  "python3 scripts/agents/snapshot-tree.py --selftest"
  "python3 scripts/kb/kb-spec-check.py --selftest" "python3 scripts/crash/crash-case-check.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/crash/crash-flow-coverage.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/mutation/prove-red.sh --selftest" "python3 scripts/gate/apply-writer-patch.py --selftest"
  "python3 scripts/gate/lib_libtest_result_lines.py --selftest" "python3 scripts/gate/rerun-failed-tests.py --selftest"
  "python3 scripts/gate/test-targets-parallel.py --selftest"
  "python3 scripts/gate/closeout-status.py --selftest" "python3 scripts/hosts/compile-then-swap.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/mutation/crates-mutation-rows.py --selftest"
  "bash .claude/gate.d/lib/stage-cells.sh --selftest" "python3 scripts/gate/gate-structure-check.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-amplification.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-amplification-item.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-amplification-two-hosts.sh --selftest"
  "bash scripts/crash/gpu-quiesce-run.sh --selftest"
  "python3 scripts/hosts/gpu-card-health.py --selftest"
  "python3 scripts/hosts/queue-health.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-point-replay.sh --selftest"
  "python3 scripts/crash/gpu-exemptions-check.py --selftest"
  "python3 scripts/crash/cpu-judging-sites-check.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/crash/lib_verdict_library.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/mutation/broad-mutation.py --selftest" "python3 scripts/mutation/mutation-prune.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/verus-proof-check.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e165-pin-red-examples.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --selftest"
  "python3 scripts/experiment-apparatus/e165-compare-kill-bounds.py --selftest"
  "python3 scripts/experiment-apparatus/e173_r2_compare.py --selftest"
  "python3 scripts/gate/hook-swap.py --selftest" "python3 scripts/gate/make-swap-spec.py --selftest" "python3 scripts/hosts/host-dispatch.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/selftest-process-guard.py --selftest" "bash scripts/devices/vm-bench.sh --selftest"
  "python3 scripts/hosts/source-stamp.py --selftest" "python3 scripts/hosts/memory_peaks_lib.py --selftest"
  "python3 scripts/hosts/minimal-copy.py --selftest" "python3 scripts/hosts/lib_repository_copy.py --selftest" "bash scripts/hosts/work-directory-ownership.sh --selftest"
  "bash scripts/devices/qemu-blklogwrites-check.sh --selftest" "bash scripts/devices/build-patched-qemu.sh --selftest"
  "python3 scripts/gate/stage-mine.py --selftest"
  "python3 scripts/gate/patch-of-the-current-tree.py --selftest"
  "python3 scripts/kb/relabel-item.py --selftest" "bash scripts/kb/claim-experiment.sh --selftest" "python3 scripts/kb/lib_experiment_number_directories.py --selftest"
  "python3 scripts/hosts/target-prune.py --selftest" "python3 scripts/gate/crate-sources-reachable.py --selftest" "python3 scripts/gate/refused-test-registry.py --selftest"
  "python3 scripts/gate/comment-history-check.py --selftest" "python3 scripts/gate/directory-layout-check.py --selftest" "python3 scripts/gate/crate-roles-check.py --selftest" "python3 scripts/gate/rust-name-collisions.py --selftest"
  "python3 scripts/gate/large-file-git-hook.py --selftest" "python3 scripts/gate/git-hooks-install.py --selftest"
  "python3 scripts/gate/data-name-lint.py --selftest" "python3 scripts/gate/wallclock-assert-lint.py --selftest"
  "bash scripts/agents/remind.sh --selftest"
  "python3 .claude/hooks/lib_workspace_cargo_lock.py --selftest"
  "python3 .claude/hooks/lib_prompt_scratch_line.py --selftest" "python3 .claude/hooks/lib_work_root.py --selftest"
  "bash scripts/hosts/heavy-threads.sh --selftest" "python3 scripts/hosts/heavy_threads.py --selftest"
  "python3 scripts/hosts/multi_host_addresses.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e185-r2-segment-run.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-amplification-lib.sh --selftest" "python3 scripts/kb/doc-consolidate.py --selftest" "python3 scripts/kb/history-consolidate.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/lib_reuse_checks.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/lkmm-selftest.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/mutation/tests/test_audit_regressions.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/device-code-key.py --selftest" "python3 scripts/gate/shared-index-stale-check.py --selftest"
  "bash scripts/experiment-apparatus/e180-reading-shards.sh --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e182-single-device-long-run.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --selftest"
  "bash scripts/experiment-apparatus/e56-sweep.sh --selftest" "bash scripts/experiment-apparatus/e56-vm-bench-run.sh --selftest"
  "bash scripts/experiment-apparatus/e07-vm-bench-run.sh --selftest" "bash scripts/experiment-apparatus/e53-vm-bench-run.sh --selftest" "bash scripts/experiment-apparatus/e159-vm-bench-run.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e182-single-device-units-kept-long-run.sh --selftest"
  "bash scripts/experiment-apparatus/e183-capacity-cells.sh --selftest" "bash scripts/experiment-apparatus/e183-sweep-history-shards.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e184-segment-one-long-run.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e185-r3-segment-run.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e185-r4-segment-run.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_two.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_three.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_two_q5.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_q5_record_path_counts.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_replay_result_lines.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_one_b_anchors_and_freeze.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/long_run_item_verdicts.py --selftest"
  "python3 scripts/hosts/rocksdb-prebuilt.py --selftest"
  "python3 scripts/experiment-apparatus/e21-card-lease-selftest.py --selftest"
  "python3 scripts/kb/milestone-close-owed-check.py --selftest" "python3 scripts/kb/move-directory-in-use-check.py --selftest"
  "python3 scripts/hosts/clean-session-scratch.py --selftest" "bash scripts/gate/staged-tree-worktree.sh --selftest"
  "bash scripts/crash/crash-point-replay-shard-configuration-check.sh --selftest" "python3 scripts/kb/path-successor.py --selftest"
  "bash scripts/crash/crash-library-totals.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/prune-server/prune-server-load-simulation.py --selftest"
  "python3 .claude/gate.d/lib-lag-baseline.py --selftest"
  "python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py --selftest"
  "python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e171_three_oracle_collusion/shared_steps.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e171-row-shard-selftest.sh --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/contrast_revision_6_against_revision_8.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/contrast_segment_two_revision_10.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_tree_lines_revision_8.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/cross_host_o3_check.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/revision_10_placeholders.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/revision_8_placeholders.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_revision_11.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_revision_8.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two_revision_10.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two_revision_11.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_11.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py --selftest"
)
# 登记的大自证（这一格文件头「分批并行」）：各占一批，整条写法与 SELFTEST_RUNNERS 里的逐字相同
SELFTEST_RUNNERS_MEMORY_BOUND=("python3 scripts/gate/lib_function_units.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/verus-proof-check.py --selftest")
# 包装写法的前缀（这一格文件头「只在临时 prune-server 上跑的自证」）与认它要包的那一句（被起的脚本源码里出现它就要包）
SELFTEST_TEMPORARY_SERVICE_PREFIX="python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --"
SELFTEST_TEMPORARY_SERVICE_MARK="selftest_service_problem"
# selftest_inner_into <表里的一条>：包装写法取 -- 后面那条原命令，别的原样，放进 selftest_inner（不起子 shell：表一百多条，逐条调要起一百多个）
selftest_inner_into() {
  if [[ "$1" == "$SELFTEST_TEMPORARY_SERVICE_PREFIX "* ]]; then selftest_inner="${1#"$SELFTEST_TEMPORARY_SERVICE_PREFIX "}"; else selftest_inner="$1"; fi
}
# 要临时服务的自证登记表（TEMPORARY_SERVICE_TABLE，写法在它的文件头，读法是 .claude/hooks/lib_project_code_runs.py 的 read_temporary_service_table，这里不另写一份解析）：
#   这一道里凡是判「这份自证要不要临时服务」的地方（research-script-selftests 格 runner 表里该不该套包装、reuse-selftests 与 break-switch-replay 两格要不要先取二进制、
#   temporary-service-runners 格两边对齐）一律读这张表登记成 required 的，不看源码里有没有 $SELFTEST_TEMPORARY_SERVICE_MARK：
#   own-server 的两份源码里也有这几个字，它们的 --selftest 自己起真服务、裸跑不退 2，不许被要求套包装（runner 表里包着它们只为把二进制交下去，包不包都不判）。
#   被判的根是样本时表可以不放（当作空表）；真仓与 --staged 的树里表不在、读不了或有写坏的行，用到它的格判红。
#   弄坏开关 CODE_TOOLING_BREAK=temporary-service-table-ignored：看源码里有没有那几个字：own-server 的被要求包装，表里是 required 而源码没写那几个字的认不出。
TEMPORARY_SERVICE_TABLE=".claude/hooks/selftest-temporary-service.tsv"
declare -A TEMPORARY_SERVICE_JUDGEMENT=()
# temporary_service_table_load：把被判的根里的表读进 TEMPORARY_SERVICE_JUDGEMENT（路径 → required 或 own-server）；读不成打出为什么与出路、返回 1
temporary_service_table_load() {
  local output key judgement
  TEMPORARY_SERVICE_JUDGEMENT=()
  if [[ ! -f "$TEMPORARY_SERVICE_TABLE" ]]; then
    judged_root_is_sample && return 0
    echo "  ✗ 没有要临时服务的自证登记表 $TEMPORARY_SERVICE_TABLE，哪份自证要套临时 prune-server 判不了"
    echo "     → 怎么办：表是新立的，写法在它的文件头；确认它进了这一棵树（--staged 判的是暂存区，要先 git add 这个文件）"
    return 1
  fi
  # 同一趟里表不变：第一个读成的格把读出的行落进本趟共用的临时目录，别的格直接读那份（每读一次要起一个 python 载入 lib_project_code_runs.py）；读不成的不落，各格各自报
  local cached="${CODE_TOOLING_RUN_SCRATCH:+$CODE_TOOLING_RUN_SCRATCH/temporary-service-table.tsv}"
  if [[ -n "$cached" && -f "$cached" ]]; then
    output="$(<"$cached")"
  elif ! output="$(python3 - "$STAGE_REPOSITORY/.claude/hooks/lib_project_code_runs.py" "$TEMPORARY_SERVICE_TABLE" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("lib_project_code_runs", sys.argv[1])
library = importlib.util.module_from_spec(spec)
spec.loader.exec_module(library)
rows, problems = library.read_temporary_service_table(sys.argv[2])
if problems:
    print("\n".join(problems))
    sys.exit(1)
for key, judgement in sorted(rows.items()):
    print(f"{key}\t{judgement}")
PY
  )"; then
    echo "  ✗ 要临时服务的自证登记表 $TEMPORARY_SERVICE_TABLE 读不了或有写坏的行："
    printf '%s\n' "$output" | sed 's/^/      /'   # gate-lint:detail
    echo "     → 怎么办：照上面逐行改表（一行 <路径> required 或 own-server 加 # 理由，写法在表的文件头）；判法在 .claude/hooks/lib_project_code_runs.py 的 temporary_service_table_at"
    return 1
  fi
  if [[ -n "$cached" && ! -f "$cached" ]]; then
    printf '%s\n' "$output" >"$cached.$BASHPID" && mv -f -- "$cached.$BASHPID" "$cached" || rm -f -- "$cached.$BASHPID"
  fi
  while IFS=$'\t' read -r key judgement; do
    [[ -n "$key" ]] && TEMPORARY_SERVICE_JUDGEMENT[$key]="$judgement"
  done <<<"$output"
  return 0
}
# temporary_service_is_required <路径>：表里这份脚本登记成 required（它的 --selftest 开跑先调客户端核对，没有临时服务就退 2）就返回 0
temporary_service_is_required() {
  if [[ "${CODE_TOOLING_BREAK:-}" == temporary-service-table-ignored ]]; then
    [[ -f "$1" ]] && grep -qF -- "$SELFTEST_TEMPORARY_SERVICE_MARK" "$1"
    return
  fi
  [[ "${TEMPORARY_SERVICE_JUDGEMENT[$1]:-}" == required ]]
}
# temporary_service_bare_runners：${runners[@]} 里裸写（没套包装写法）而表里是 required 的，一行一条打出
temporary_service_bare_runners() {
  local runner
  local -a bare_words=()
  for runner in "${runners[@]}"; do
    [[ "$runner" == "$SELFTEST_TEMPORARY_SERVICE_PREFIX "* ]] && continue
    read -r -a bare_words <<<"$runner"
    if temporary_service_is_required "${bare_words[1]:-}"; then printf '%s\n' "$runner"; fi
  done
  return 0
}
# code_tooling_prune_server_binary <编译目录>：打一份 prune-server 二进制的路径（research-script-selftests 一趟取一次；reuse-selftests 有要临时服务的一行要现跑时，
#   那一格的程序经 export -f 起它取一次）：
#   PRUNE_SERVER_BINARY 设了（非空，绝对路径）用它（与 scripts/prune-server/prune_server_client_selftest.py 的 resolve_binary 认同一个变量）；
#   没设照 prune-server-selftest 格的编法（经 capped.sh、--offline、被判那棵树的 prune-server/）编进 <编译目录>/target（调用的那一格自己的临时目录，跑完随它删）。
#   取不到退 1，为什么打在标准错误
#   整道门禁这一趟只编一次：开跑前建了本趟共用的临时目录（CODE_TOOLING_RUN_SCRATCH）时，没设 PRUNE_SERVER_BINARY 的各格取它都落到那个目录里的同一份编译
#   （flock 排队：并发的格等第一个编完；编不过的结局也记下，后到的格直接得到同一句原因、不再编第二次）；没有共用目录、没有 flock 的才编进调用方给的 <编译目录>。
#   样本根里放了 sample-prune-server-build 的（只在样本根认），不认 PRUNE_SERVER_BINARY、不起 cargo：`<那份文件> <目标目录>`（直接执行，带执行位），由它在 <目标目录>/release/ 下造出 prune-server。
#   弄坏开关 CODE_TOOLING_BREAK=prune-server-build-not-shared：不共用，各格各编进自己的 <编译目录>，prune-server-shared-build-green 里第二次起假编译被拒、两格判红。
code_tooling_prune_server_binary() {
  local sample_build=0 shared="${CODE_TOOLING_RUN_SCRATCH:-}" cache
  [[ -f "${ROOT:-}/sample-prune-server-build" ]] && declare -F judged_root_is_sample >/dev/null && judged_root_is_sample && sample_build=1
  if [[ -n "${PRUNE_SERVER_BINARY:-}" ]] && ((! sample_build)); then
    [[ "$PRUNE_SERVER_BINARY" == /* && -x "$PRUNE_SERVER_BINARY" ]] || { echo "PRUNE_SERVER_BINARY=$PRUNE_SERVER_BINARY 不是可执行文件的绝对路径" >&2; return 1; }
    printf '%s\n' "$PRUNE_SERVER_BINARY"
    return 0
  fi
  cache="$shared/prune-server-build"
  if [[ -z "$shared" || "${CODE_TOOLING_BREAK:-}" == prune-server-build-not-shared ]] || ! command -v flock >/dev/null || ! mkdir -p -- "$cache"; then
    code_tooling_prune_server_build_into "$1"
    return
  fi
  (
    flock 9 || exit 1
    [[ -e "$cache/outcome" ]] && exit 0
    if code_tooling_prune_server_build_into "$cache" >"$cache/binary" 2>"$cache/error"; then printf '0\n' >"$cache/outcome"; else printf '1\n' >"$cache/outcome"; fi
  ) 9>"$cache/lock"
  if [[ "$(cat -- "$cache/outcome" 2>/dev/null)" == 0 ]]; then
    cat -- "$cache/binary"
    return 0
  fi
  cat -- "$cache/error" >&2 2>/dev/null
  return 1
}
# code_tooling_prune_server_build_into <编译目录>：把被判那棵树的 prune-server 编进 <编译目录>/target（样本根的 sample-prune-server-build 顶替 cargo），打二进制的路径；编不成退 1、原因打在标准错误，日志是 <编译目录>/prune-server-build.log
code_tooling_prune_server_build_into() {
  if [[ -f "${ROOT:-}/sample-prune-server-build" ]] && declare -F judged_root_is_sample >/dev/null && judged_root_is_sample; then
    if ! "$ROOT/sample-prune-server-build" "$1/target" >"$1/prune-server-build.log" 2>&1; then
      echo "样本的假编译没成：$(tail -n 3 "$1/prune-server-build.log" | tr '\n' ' ' | clip_text 300)" >&2
      return 1
    fi
  elif ! CARGO_TARGET_DIR="$1/target" \
      cargo build --release --offline --manifest-path "$ROOT/prune-server/Cargo.toml" --bin prune-server >"$1/prune-server-build.log" 2>&1; then
    echo "cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server 没成：$(tail -n 3 "$1/prune-server-build.log" | tr '\n' ' ' | clip_text 300)" >&2
    return 1
  fi
  [[ -x "$1/target/release/prune-server" ]] || { echo "编完不见 $1/target/release/prune-server" >&2; return 1; }
  printf '%s\n' "$1/target/release/prune-server"
}
SELFTEST_RUNNERS_STARTED_ALONE=("python3 scripts/agents/agent-watch.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest")

# selftest_plan_runners <草稿目录>：table_runners 每一条一条判定请求（这一格文件头「按条复用」），经 scripts/gate/admission.py verdict-plan 算键、查库；
# 结论放进 selftest_decision[<命令>]（reuse、judge、refused）与 selftest_reason[<命令>]，计划文件是 <草稿目录>/plan；算不出返回 1、原因放进 selftest_plan_problem。
# 样本根另放了 .unit-verdict-sample/prior/（上一版的树）的，先拿上一版算键、照判绿记进样本自己的判定库。
# 弄坏开关 CODE_TOOLING_BREAK=selftest-key-whole-runner-table：每一条的键另加表里每一条自证的脚本（file:），一份脚本改了整张表都作废；
#   =selftest-judge-whole-guard：判法取整份 selftest-process-guard.py，每一条都拒算
# selftest_judge_entries_load：判法那几个函数只有一份，在 selftest-process-guard.py 的 SELFTEST_JUDGE_ENTRIES（守卫单独起时算键用的同一份）；
#   按 ast 读那一句字面元组，不载入那份脚本。放进全局 SELFTEST_JUDGE_ENTRIES；读不出打 ✗ 与出路、记 selftest_plan_red=1（这一格判红），返回 1
#   只供样本（被判的根是样本才认）：样本根的 .selftest-judge-guard.py 顶替这一道所在仓里的守卫被读（样本根换不掉那一份）；样本 runner-judge-entries-unreadable-red
#     放一份没有那一句的，这一格判红、点名读不出。
#   弄坏开关 CODE_TOOLING_BREAK=selftest-judge-entries-unread-ignored：读不出不判红，名单空着往下算（每一条的键里没有判法那几项），
#     runner-judge-entries-unreadable-red 判错（照判绿、退 0）
selftest_judge_entries_load() {
  local guard="$STAGE_REPOSITORY/scripts/gate/selftest-process-guard.py" text
  judged_root_is_sample && [[ -f .selftest-judge-guard.py ]] && guard="$ROOT/.selftest-judge-guard.py"
  SELFTEST_JUDGE_ENTRIES=()
  if text="$(python3 - "$guard" 2>&1 <<'PY_SELFTEST_JUDGE_ENTRIES'
import ast
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    tree = ast.parse(handle.read())
for node in tree.body:
    if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "SELFTEST_JUDGE_ENTRIES" for target in node.targets):
        entries = ast.literal_eval(node.value)
        if not entries or not all(isinstance(entry, str) and "::" in entry for entry in entries):
            sys.exit("SELFTEST_JUDGE_ENTRIES 不是一串「<文件>::<限定名>」")
        print("\n".join(entries))
        sys.exit(0)
sys.exit("找不到顶层一句 SELFTEST_JUDGE_ENTRIES = (…)")
PY_SELFTEST_JUDGE_ENTRIES
)"; then
    mapfile -t SELFTEST_JUDGE_ENTRIES <<<"$text"
    return 0
  fi
  # 弄坏开关 selftest-judge-entries-unread-ignored：读不出不判红、名单空着往下算（这一格函数上方的注释）
  [[ "${CODE_TOOLING_BREAK:-}" == selftest-judge-entries-unread-ignored ]] && return 0
  echo "  ✗ 判法那几个函数取不出：$guard 的 SELFTEST_JUDGE_ENTRIES 读不出（${text:0:300}）"
  echo "     → 怎么办：那一句要是顶层的字面元组、每一项写成「scripts/gate/selftest-process-guard.py::<限定名>」（守卫单独起时算键用的就是它，这一格不另抄一份）；改好再跑这一格"
  selftest_plan_red=1
  return 1
}
selftest_plan_runners() {
  local scratch="$1" runner tier name decision key reason entry plan_exit word
  local -a words=() table_files=() judge_tokens=() plan_environment=() wrapper_token=() argument_tokens=()
  # 样本根带 .unit-verdict-sample/red-over-green：库里已有上一版照判绿记进来的同一把键，这一格的计划另设 ADMISSION_BREAK=verdict-lookup-skipped（不读库、一律现判）
  #   照样现判（这一格文件头「按条复用」记红、撤绿）；门禁样本根里 admission.py 才认这个开关
  judged_root_is_sample && [[ -f .unit-verdict-sample/red-over-green ]] && plan_environment=(env ADMISSION_BREAK=verdict-lookup-skipped)
  local admission="$STAGE_REPOSITORY/scripts/gate/admission.py" kind="$STAGE_NAME:research-script-selftests"
  selftest_plan_problem=""; selftest_plan_failure=()
  selftest_judge_entries_load || { selftest_plan_problem="判法那几个函数取不出（上面那一句）"; return 1; }
  for entry in "${SELFTEST_JUDGE_ENTRIES[@]}"; do judge_tokens+=("judge:$entry"); done
  [[ "${CODE_TOOLING_BREAK:-}" == selftest-judge-whole-guard ]] && judge_tokens=("judge:scripts/gate/selftest-process-guard.py")
  : >"$scratch/requests"
  # 登记表（登记在哪一档）读一次放进 selftest_tier_of：每条自证查一次，不逐条起 awk 读整张表；一个脚本登记了几行取头一行
  local registry_line registry_key registry_rest
  local -A selftest_tier_of=()
  if [[ -f .claude/hooks/selftest-registry.tsv ]]; then
    while IFS= read -r registry_line || [[ -n "$registry_line" ]]; do
      registry_key="${registry_line%%$'\t'*}"
      [[ -n "$registry_key" && -z "${selftest_tier_of[$registry_key]+set}" ]] || continue
      registry_rest=""; [[ "$registry_line" == *$'\t'* ]] && registry_rest="${registry_line#*$'\t'}"
      selftest_tier_of[$registry_key]="${registry_rest%%$'\t'*}"
    done <.claude/hooks/selftest-registry.tsv
  fi
  if [[ "${CODE_TOOLING_BREAK:-}" == selftest-key-whole-runner-table ]]; then
    for runner in "${table_runners[@]}"; do selftest_inner_into "$runner"; read -r -a words <<<"$selftest_inner"; [[ -f "${words[1]:-}" ]] && table_files+=("file:${words[1]}"); done
  fi
  for runner in "${table_runners[@]}"; do
    selftest_inner_into "$runner"; read -r -a words <<<"$selftest_inner"
    wrapper_token=()
    [[ "$runner" == "$SELFTEST_TEMPORARY_SERVICE_PREFIX "* ]] && wrapper_token=("file:scripts/prune-server/prune_server_client_selftest.py")
    # bash 起的自证判的是「这份脚本带这组参数跑」：脚本之后的词照次序进 program-argument:，闭包只收它们真走到的那一支（这一格文件头「按条复用」）
    argument_tokens=()
    if [[ "${words[0]:-}" == bash ]]; then
      for word in "${words[@]:2}"; do argument_tokens+=("program-argument:$word"); done
    fi
    tier=""; [[ -n "${words[1]:-}" ]] && tier="${selftest_tier_of[${words[1]}]:-}"
    { printf '%s\t%s' "$runner" "${words[1]:-${words[0]:-}}"
      printf '\t%s' "${judge_tokens[@]}" "judge-root:$STAGE_REPOSITORY" "line:command=$runner" "line:selftest-registration=${tier:-没登记}" \
        ${argument_tokens[@]+"${argument_tokens[@]}"} ${wrapper_token[@]+"${wrapper_token[@]}"} ${table_files[@]+"${table_files[@]}"}
      printf '\n'; } >>"$scratch/requests"
  done
  if judged_root_is_sample && [[ -d .unit-verdict-sample/prior ]]; then
    if python3 "$admission" verdict-plan "$ROOT/.unit-verdict-sample/prior" "$kind" "$scratch/requests" "$scratch/seed.plan" >"$scratch/seed.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
    if ((plan_exit != 0)); then
      selftest_plan_problem="样本：上一版的树上算不出键（$(tail -n 2 "$scratch/seed.out" | tr '\n' ' ' | cut -c1-300)）"
      selftest_plan_failure=(verdict-plan "$plan_exit" "$scratch/seed.out"); return 1
    fi
    cut -f1 "$scratch/requests" >"$scratch/seed.names"
    if python3 "$admission" verdict-record "$ROOT/.unit-verdict-sample/prior" "$kind" "$scratch/seed.plan" "$scratch/seed.names" --how "样本：上一版" >"$scratch/seed.record" 2>&1; then plan_exit=0; else plan_exit=$?; fi
    if ((plan_exit != 0)); then
      selftest_plan_problem="样本：上一版那几条没记进库（$(tail -n 2 "$scratch/seed.record" | tr '\n' ' ')）"
      selftest_plan_failure=(verdict-record "$plan_exit" "$scratch/seed.record"); return 1
    fi
    echo "  · 样本：上一版（.unit-verdict-sample/prior）的 $(grep -c . "$scratch/seed.names") 条自证照判绿记进这一版样本的判定库"
  fi
  if ${plan_environment[@]+"${plan_environment[@]}"} python3 "$admission" verdict-plan "$ROOT" "$kind" "$scratch/requests" "$scratch/plan" >"$scratch/plan.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
  if ((plan_exit != 0)); then
    selftest_plan_problem="admission.py verdict-plan 没成（$(tail -n 2 "$scratch/plan.out" | tr '\n' ' ' | cut -c1-300)）"
    selftest_plan_failure=(verdict-plan "$plan_exit" "$scratch/plan.out"); return 1
  fi
  grep -q '^PLAN ' "$scratch/plan.out" || { selftest_plan_problem="admission.py verdict-plan 没打 PLAN 那一行"; selftest_plan_failure=(verdict-plan 0 "$scratch/plan.out"); return 1; }
  while IFS=$'\t' read -r name decision key reason; do
    [[ -n "$name" && "$name" != PLAN* ]] || continue
    selftest_decision[$name]="$decision"
    selftest_reason[$name]="$reason"
  done <"$scratch/plan.out"
  return 0
}

# selftest_plan_failure_verdict <子命令> <退出码> <输出文件>：这一趟按条复用失灵没有，判法只在 scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」）；
# 失灵它打 ✗ 与出路，这里记 selftest_plan_red=1，这一格照旧跑完、结论判红
# plan-failure 退 75 是 prune-server 不可用（不是复用失灵）：记 selftest_service_unavailable=1，这一格退 75（这一格文件头「按条复用」）
selftest_plan_failure_verdict() {
  local verdict_exit
  if python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" plan-failure "$STAGE_NAME 的 research-script-selftests 格" "$@"; then return 0; else verdict_exit=$?; fi
  if ((verdict_exit == 75)); then selftest_service_unavailable=1; else selftest_plan_red=1; fi
}

# 记库时交给 admission.py verdict-record 的跑法（当场记的一条与收尾那一趟同一句）
SELFTEST_RECORD_HOW="经 scripts/gate/selftest-process-guard.py 起"

# selftest_outcome <序号> <命令>：读 <逐条目录>/<序号>.exit 与 .guard，定这一条的结局（这一格文件头「判据」「按条复用」）；回读与当场记库同一份判法。
# 放进 selftest_rc（照弄坏开关 runner-exit-swallowed、selftest-not-run-as-pass 改过的退出码）、selftest_guard_verdict（守卫判定的头一行）与 selftest_result：
#   not-run（退 77）、red（退非 0）、green-record（现判而判绿、守卫给了判定：记库；盯了而 execve 跟踪没跑的不在这里）、green（判绿而不记）
selftest_outcome() {
  selftest_rc="$(<"$scratch/$1.exit")"
  [[ "${CODE_TOOLING_BREAK:-}" == runner-exit-swallowed ]] && selftest_rc=0
  [[ "$selftest_rc" == 77 && "${CODE_TOOLING_BREAK:-}" == selftest-not-run-as-pass ]] && selftest_rc=0
  selftest_guard_verdict=""
  [[ -f "$scratch/$1.guard" ]] && selftest_guard_verdict="$(head -n 1 "$scratch/$1.guard")"
  if [[ "$selftest_rc" == 77 ]]; then
    selftest_result=not-run
  elif [[ "$selftest_rc" != 0 ]]; then
    selftest_result=red
  elif [[ "${selftest_decision[$2]:-}" == judge && ( "$selftest_guard_verdict" == not-watched$'\t'* || ( "$selftest_guard_verdict" == watched$'\t'* && "$selftest_guard_verdict" != *$'\ttrace=no'* ) ) ]]; then
    selftest_result=green-record
  else
    selftest_result=green
  fi
}

# selftest_record_one <序号> <命令>：一条自证跑完当场照键记（这一格文件头「按条复用」一条判完就记），在起它的那个后台作业里、退出码写完之后调：
# 判绿该记的记绿，判红的记红、撤同一把键原有的绿（弄坏开关 red-not-recorded 下不记红），退 77、复用、拒算的不记。经 admission.py verdict-record 写
# （每把键一份文件，同目录临时文件写完 fsync 再改名换上），计划文件用不带复用哨兵状态的那个名字（<算键目录>/plan-per-entry 指向 plan：哨兵在收尾那一趟判一次）。
# 输出写 <序号>.record、退出码写 <序号>.record-exit。中途被停，已经跑完的那几条都在库里，下一趟它们复用、只补没判的。
# 弄坏开关 CODE_TOOLING_BREAK=selftest-recorded-at-end：不当场记，只在收尾一次记（中途被停，判完的全丢）
selftest_record_one() {
  local names="$scratch/$1.record-names" record_exit
  local -a lists=()
  ((selftest_plan_ok)) || return 0
  [[ "${CODE_TOOLING_BREAK:-}" == selftest-recorded-at-end ]] && return 0
  selftest_outcome "$1" "$2"
  case "$selftest_result" in
    green-record) printf '%s\n' "$2" >"$names"; lists=("$names") ;;
    red)
      [[ "${CODE_TOOLING_BREAK:-}" == red-not-recorded ]] && return 0
      printf '%s\n' "$2" >"$names"; : >"$names.green"; lists=("$names.green" --red-names "$names") ;;
    *) return 0 ;;
  esac
  if python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" verdict-record "$ROOT" "$STAGE_NAME:research-script-selftests" "$selftest_plan_scratch/plan-per-entry" "${lists[@]}" \
       --how "$SELFTEST_RECORD_HOW" >"$scratch/$1.record" 2>&1; then record_exit=0; else record_exit=$?; fi
  printf '%s\n' "$record_exit" >"$scratch/$1.record-exit"
}

# 收尾那一趟 verdict-record 退 0 时转进日志的「!」行至多几行（同 UNIT_VERDICT_ROUNDS_LIBRARY 的 RECORD_SKIP_LINES_SHOWN，那一份在 python 的 here-doc 里，两处同改）
SELFTEST_RECORD_SKIP_LINES_SHOWN=20

# selftest_record_skips_report <记库的输出文件>：收尾那一趟 verdict-record 退 0 时它逐条打的「!」行（跑的过程中键变了、不记的，写不进服务的）原样转进日志：
# 草稿目录收尾就删，不转就只剩 RECORD 行里的 changed= 一个数。至多 SELFTEST_RECORD_SKIP_LINES_SHOWN 行（样本根 .unit-verdict-sample/record-skip-lines-shown
# 写着正整数的照它），多出的报一句「另有 N 条」；同 UNIT_VERDICT_ROUNDS_LIBRARY 的 ItemVerdicts.report_record_skips。
# 弄坏开关 CODE_TOOLING_BREAK=record-skip-lines-dropped：一行都不转
selftest_record_skips_report() {
  local shown="$SELFTEST_RECORD_SKIP_LINES_SHOWN" total value
  [[ "${CODE_TOOLING_BREAK:-}" == record-skip-lines-dropped ]] && return 0
  if judged_root_is_sample && [[ -f .unit-verdict-sample/record-skip-lines-shown ]]; then
    value="$(tr -d '[:space:]' < .unit-verdict-sample/record-skip-lines-shown)"
    [[ "$value" =~ ^[1-9][0-9]*$ ]] && shown="$value"
  fi
  total="$(awk '/^[[:space:]]*!/ { count++ } END { print count + 0 }' "$1")"
  awk -v shown="$shown" '/^[[:space:]]*!/ && printed < shown { print; printed++ }' "$1"
  if ((total > shown)); then
    echo "  ! 记库没记的另有 $((total - shown)) 条没转进日志（verdict-record 这一趟共打 $total 条「!」行）"
  fi
  return 0
}

# selftest_record_verdicts <草稿目录> <判绿的命令文件> <判红的命令文件>：现判而判绿的那几条照键记进库，现判而判红的（自证退非 0、不是 77）照同一把键记红、
# 撤掉这把键原有的绿（这一格文件头「按条复用」记红、撤绿、下趟现判），经 admission.py verdict-record 的 --red-names；拒算的没有键，不记。
# 只记绿的那一趟记不下只报、不判红，复用哨兵报红（强制现判那一条判定不稳）的经 plan-failure 判红；交了红的记不下带 --red 交 plan-failure：记红没成它判失灵、
# 撤掉这几把键原有的绿（撤不掉说整道判红，admission.py 文件头「按条复用失灵」）。弄坏开关 CODE_TOOLING_BREAK=red-not-recorded 下不交 --red-names（只记绿）；
# =red-record-failure-kept-green 下记红没成只打一行「!」、不交 plan-failure（同一把键原有的绿留着，下一趟照复用）
selftest_record_verdicts() {
  local record_exit
  local -a red_option=()
  [[ -s "$2" || -s "$3" ]] || return 0
  [[ -s "$3" && "${CODE_TOOLING_BREAK:-}" != red-not-recorded ]] && red_option=(--red-names "$3")
  if python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" verdict-record "$ROOT" "$STAGE_NAME:research-script-selftests" "$1/plan" "$2" ${red_option[@]+"${red_option[@]}"} \
       --how "$SELFTEST_RECORD_HOW" >"$1/record.out" 2>&1; then record_exit=0; else record_exit=$?; fi
  if ((record_exit == 0)); then
    [[ -s "$2" ]] && echo "  · 判绿记库：$(tail -n 1 "$1/record.out")"
    ((${#red_option[@]})) && echo "  · 判红记库：$(grep -c . "$3") 条记红、撤掉同一把键原有的绿，下一趟这几条现判（$(tail -n 1 "$1/record.out")）"
    selftest_record_skips_report "$1/record.out"
  elif ((${#red_option[@]})) && [[ "${CODE_TOOLING_BREAK:-}" == red-record-failure-kept-green ]]; then
    # 弄坏开关：记红没成只打一行「!」、不交 plan-failure，同一把键原有的绿留着、下一趟照复用
    echo "  ! 判定记库：admission.py verdict-record 没成（$(tail -n 2 "$1/record.out" | tr '\n' ' ')），下一趟这几条照跑"
  elif ((${#red_option[@]})); then
    # 记红没成（admission.py 文件头「按条复用失灵」）：带 --red 交 plan-failure，它判失灵、撤掉这几把键原有的绿（撤不掉说整道判红）
    echo "  · 判定记库：admission.py verdict-record 退 $record_exit（$(tail -n 2 "$1/record.out" | tr '\n' ' ' | clip_text 300)），记红没成，交 plan-failure 判、撤同一把键原有的绿"
    selftest_plan_failure_verdict verdict-record "$record_exit" --red "$ROOT" "$STAGE_NAME:research-script-selftests" "$1/plan" "$3" "$1/record.out"
  else
    echo "  ! 判定记库：admission.py verdict-record 没成（$(tail -n 2 "$1/record.out" | tr '\n' ' ')），下一趟这几条照跑"
    selftest_plan_failure_verdict verdict-record "$record_exit" "$1/record.out"
  fi
}

# selftest_reread_after_red <草稿目录>：样本根带 .unit-verdict-sample/red-over-green 时（这一格文件头「按条复用」），判红记库之后不设 ADMISSION_BREAK 再算一次表的计划，
# 打复用几条、现判几条，现判的逐条写原因（记红盖掉了绿：红的那一条答「上次判红」；没盖掉：它答复用）
selftest_reread_after_red() {
  [[ -f .unit-verdict-sample/red-over-green ]] || return 0
  if python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" verdict-plan "$ROOT" "$STAGE_NAME:research-script-selftests" "$1/requests" "$1/reread.plan" >"$1/reread.out" 2>&1; then
    echo "  · 样本档：判红记库之后再算一次计划：复用 $(awk -F'\t' '$2 == "reuse" { count++ } END { print count + 0 }' "$1/reread.out") 条、现判 $(awk -F'\t' '$2 == "judge" { count++ } END { print count + 0 }' "$1/reread.out") 条"
    awk -F'\t' '$2 == "judge" { printf "    现判「%s」：%s\n", $1, $4 }' "$1/reread.out"
  else
    echo "  ✗ 样本档：判红记库之后再算计划没成（$(tail -n 2 "$1/reread.out" | tr '\n' ' ')）"
    echo "     → 怎么办：看括号里 admission.py verdict-plan 报的错，修好样本或判定库，再单跑这一份样本"
  fi
}

# selftest_runner_table_load：取这一趟的 runner 表（research-script-selftests 与 temporary-service-runners 两格共用）：
#   设全局 runners（表）、table_source（表取自哪，报错时写）、started_alone（单独起的那几条）、memory_bound_runners（登记的大自证）；
#   被判的根是样本且有 .selftest-runners 时，换成样本根里的（单独起的那一路不走，大自证取样本根的 .selftest-memory-bound，没有就是没有）
selftest_runner_table_load() {
  runners=("${SELFTEST_RUNNERS[@]}")
  table_source="这一道的 SELFTEST_RUNNERS"
  started_alone=("${SELFTEST_RUNNERS_STARTED_ALONE[@]}")
  memory_bound_runners=("${SELFTEST_RUNNERS_MEMORY_BOUND[@]}")
  if judged_root_is_sample && [[ -f .selftest-runners ]]; then
    runners=()
    while IFS= read -r line || [[ -n "$line" ]]; do
      [[ -z "${line//[[:space:]]/}" || "$line" =~ ^[[:space:]]*# ]] && continue
      runners+=("$line")
    done < .selftest-runners
    table_source="样本根的 .selftest-runners（$ROOT 是样本，不是这一道所在的仓）"
    started_alone=()
    memory_bound_runners=()
    if [[ -f .selftest-memory-bound ]]; then
      while IFS= read -r line || [[ -n "$line" ]]; do
        [[ -z "${line//[[:space:]]/}" || "$line" =~ ^[[:space:]]*# ]] && continue
        memory_bound_runners+=("$line")
      done < .selftest-memory-bound
    fi
  fi
}

# start_selftest <表里的序号> <命令>：后台起一条，输出写 <序号>.out、退出码写 <序号>.exit、守卫的判定写 <序号>.guard，进程号记进 started_pids；
# 退出码写完，同一个后台作业里当场照键记这一条（selftest_record_one）。只由 research-script-selftests 格调，读那一格设的 scratch、selftest_process_guard、
# selftest_server_binary，往它的 started_pids 里记。写在格外的顶层：套在格函数里时，格函数的单元把这段正文也当自己的命令认，
# 命令位置上的 $inner 落在格函数名下拒算（scripts/gate/lib_function_units.py 的 UNKEYED_PROGRAM_WORDS 登的是 start_selftest 这一个函数）
start_selftest() {
  # runner 表每条是「解释器 脚本 参数」，按空白切词起，与原来逐条顺序跑时同一个起法；经守卫起：登记过的盯着它的进程树，别的守卫原样 exec
  local wrap_prefix=() inner
  # 包装写法：包装在守卫外面起、二进制交这一趟取的那一份，守卫盯 -- 后面那条原命令（这一格文件头「只在临时 prune-server 上跑的自证」）
  selftest_inner_into "$2"; inner="$selftest_inner"
  [[ "$inner" != "$2" ]] && wrap_prefix=(python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary "$selftest_server_binary" --)
  if [[ "${CODE_TOOLING_BREAK:-}" == guard-skipped ]]; then
    { printf 'watched\tfindings=0\tscan=no\texamined=0\texit=-\n' > "$scratch/$1.guard"; ${wrap_prefix[@]+"${wrap_prefix[@]}"} $inner > "$scratch/$1.out" 2>&1; printf '%s\n' "$?" > "$scratch/$1.exit"
      selftest_record_one "$1" "$2"; } &
  else
    { ${wrap_prefix[@]+"${wrap_prefix[@]}"} python3 "$selftest_process_guard" --registry-root "$ROOT" --verdict-file "$scratch/$1.guard" -- $inner > "$scratch/$1.out" 2>&1
      printf '%s\n' "$?" > "$scratch/$1.exit"
      selftest_record_one "$1" "$2"; } &
  fi
  started_pids+=("$!")
}

cell_research_script_selftests() {
selftest_runner_table_load
# 逐项点名（这一道「逐项点名」一节）：--item 点到这一格的，表里只留起那几份脚本的自证（按包装 -- 后面那条原命令的第二个词认），单独起的那一路只留还在表里的；
# 键、判法、复用与记库照整格的走法，点名单跑一条与它在整格里跑的判定相同
if [[ -n "${CODE_TOOLING_PICKED[research-script-selftests]:-}" ]]; then
  declare -A picked_selftest_scripts=()
  while IFS= read -r line; do [[ -n "$line" ]] && picked_selftest_scripts[$line]=1; done <<<"${CODE_TOOLING_PICKED[research-script-selftests]}"
  picked_selftest_runners=(); picked_alone=(); picked_memory_bound=()
  for runner in "${runners[@]}"; do
    selftest_inner_into "$runner"; read -r -a words <<<"$selftest_inner"
    if [[ -n "${words[1]:-}" && -n "${picked_selftest_scripts[${words[1]}]+set}" ]]; then picked_selftest_runners+=("$runner"); fi
  done
  for runner in ${started_alone[@]+"${started_alone[@]}"}; do
    for line in ${picked_selftest_runners[@]+"${picked_selftest_runners[@]}"}; do [[ "$line" == "$runner" ]] && picked_alone+=("$runner"); done
  done
  for runner in ${memory_bound_runners[@]+"${memory_bound_runners[@]}"}; do
    for line in ${picked_selftest_runners[@]+"${picked_selftest_runners[@]}"}; do [[ "$line" == "$runner" ]] && picked_memory_bound+=("$runner"); done
  done
  echo "  · 只跑点名的 ${#picked_selftest_scripts[@]} 份脚本的自证：表里 ${#runners[@]} 条留 ${#picked_selftest_runners[@]} 条（表取自 $table_source）"
  runners=(${picked_selftest_runners[@]+"${picked_selftest_runners[@]}"}); started_alone=(${picked_alone[@]+"${picked_alone[@]}"}); memory_bound_runners=(${picked_memory_bound[@]+"${picked_memory_bound[@]}"})
  table_source="$table_source里 --item 点名的那几条"
fi
# 样本根的 .selftest-environment（一行一条，空行与 # 开头的行不算）：`名字=值` 导出、`!名字` 摘掉，在这一格开跑前套上，当作这一格被起的那一刻外面的环境；
# 只在被判的根是样本时认，真仓与 --staged 下放了它也不认。样本借它造外面的环境（设上或摘掉一个变量）
if judged_root_is_sample && [[ -f .selftest-environment ]]; then
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -z "${line//[[:space:]]/}" || "$line" =~ ^[[:space:]]*# ]] && continue
    case "$line" in
      '!'*) unset "${line#!}" ;;
      *=*) export "${line%%=*}=${line#*=}" ;;
    esac
  done < .selftest-environment
fi
# 样本里的假自证要读样本那一份库（在临时 prune-server 里样本那一份命名空间）时，从 CODE_TOOLING_SAMPLE_STAGE_REPOSITORY 取这一道所在仓里的 admission.py 与客户端；只给样本设
judged_root_is_sample && export CODE_TOOLING_SAMPLE_STAGE_REPOSITORY="$STAGE_REPOSITORY"
if ((${#runners[@]} == 0)); then
  echo "  ✗ runner 表一条都没有（取自 $table_source）"
  echo "     → 怎么办：跑到 0 条自证而报绿，与判过了一模一样；表写在这一道的 SELFTEST_RUNNERS，样本写在样本根的 .selftest-runners。"
  exit 1
fi
# 只在临时 prune-server 上跑的自证要写成包装写法（这一格文件头「只在临时 prune-server 上跑的自证」）：要临时服务的表里登记成 required 而裸写的，整格判红、一条都不起
temporary_service_table_load || exit 1
mapfile -t bare_temporary_service < <(temporary_service_bare_runners)
if ((${#bare_temporary_service[@]})) && [[ "${CODE_TOOLING_BREAK:-}" != bare-temporary-service-selftest-accepted ]]; then
  echo "  ✗ 这几条自证只在临时 prune-server 上跑（$TEMPORARY_SERVICE_TABLE 登记成 required：开跑先调 $SELFTEST_TEMPORARY_SERVICE_MARK，裸跑退 2），runner 表里却是裸写的（取自 $table_source）："
  for runner in "${bare_temporary_service[@]}"; do echo "      「$runner」该写成「$SELFTEST_TEMPORARY_SERVICE_PREFIX $runner」"; done   # gate-lint:detail
  echo "     → 怎么办：照上面逐条改成包装写法（不写 --server-binary：二进制由这一格一趟取一份交给每一条）；这一格一条都没起"
  exit 1
fi
declare -A in_table=() alone_set=()
for runner in "${runners[@]}"; do in_table[$runner]=1; done
for runner in ${started_alone[@]+"${started_alone[@]}"}; do
  if [[ -z "${in_table[$runner]+set}" ]]; then
    echo "  ✗ SELFTEST_RUNNERS_STARTED_ALONE 里的「$runner」不在 runner 表里"
    echo "     → 怎么办：表里改了名或删了那一条，就同步改 SELFTEST_RUNNERS_STARTED_ALONE；对不上的那一条会被悄悄少跑。"
    exit 1
  fi
  alone_set[$runner]=1
done
# 登记的大自证（这一格文件头「分批并行」）：不在 runner 表里判红；弄坏开关 selftest-memory-bound-ignored 下照别的一样进批
declare -A selftest_memory_bound_set=()
for runner in ${memory_bound_runners[@]+"${memory_bound_runners[@]}"}; do
  if [[ -z "${in_table[$runner]+set}" && "${CODE_TOOLING_BREAK:-}" != selftest-memory-bound-unlisted-accepted ]]; then
    echo "  ✗ 登记的大自证「$runner」不在 runner 表里（取自 $table_source）"
    echo "     → 怎么办：runner 表里改了名或删了那一条，就同步改 SELFTEST_RUNNERS_MEMORY_BOUND；对不上的那一条会悄悄不分批、与别的自证叠在一起。"
    exit 1
  fi
  [[ "${CODE_TOOLING_BREAK:-}" == selftest-memory-bound-ignored ]] || selftest_memory_bound_set[$runner]=1
done

# 按条复用（这一格文件头「按条复用」）：每一条自证一条判定，键相同的不起；表全部复用就一条都不起
selftest_plan_scratch="$(mktemp -d)" || { echo "  ✗ 建不了算键的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格；一条自证都没跑。"; exit 1; }
trap 'rm -rf -- "${selftest_plan_scratch:?}"' EXIT
: >"$selftest_plan_scratch/green.names"
: >"$selftest_plan_scratch/red.names"
table_runners=("${runners[@]}")
declare -A selftest_decision=() selftest_reason=()
selftest_plan_ok=0; selftest_judged=0; selftest_reused=0; selftest_refused=0; selftest_plan_red=0; selftest_service_unavailable=0; selftest_plan_failure=()
if selftest_plan_runners "$selftest_plan_scratch"; then
  selftest_plan_ok=1
  # 一条判完就记（selftest_record_one）用的计划文件名：指向同一份计划，旁边没有复用哨兵状态（哨兵只在收尾那一趟判）
  ln -s plan "$selftest_plan_scratch/plan-per-entry"
  runners=()
  for runner in "${table_runners[@]}"; do
    case "${selftest_decision[$runner]:-judge}" in
      reuse) selftest_reused=$((selftest_reused + 1)) ;;
      refused) selftest_refused=$((selftest_refused + 1)); runners+=("$runner") ;;
      *) selftest_judged=$((selftest_judged + 1)); runners+=("$runner") ;;
    esac
  done
  echo "  · 按条复用：表里 ${#table_runners[@]} 条自证，现判 ${selftest_judged}、复用 ${selftest_reused}、拒算 ${selftest_refused}"
  # 现判与拒算的逐条列（至多 20 条，连同为什么：键对不上时是变了的单元）；复用的只在样本或不过 20 条时逐条列
  shown=0
  for runner in "${table_runners[@]}"; do
    case "${selftest_decision[$runner]:-judge}" in
      reuse) continue ;;
      refused) selftest_word="拒算" ;;
      *) selftest_word="现判" ;;
    esac
    shown=$((shown + 1))
    ((shown <= 20)) && echo "    ${selftest_word}「$runner」：${selftest_reason[$runner]:-}"
  done
  ((shown > 20)) && echo "    …现判与拒算另有 $((shown - 20)) 条没列"
  if ((selftest_reused <= 20)) || judged_root_is_sample; then
    for runner in "${table_runners[@]}"; do [[ "${selftest_decision[$runner]:-}" == reuse ]] && echo "    复用「$runner」：${selftest_reason[$runner]:-}"; done
  fi
else
  echo "  ! 按条复用算不出（$selftest_plan_problem）：这一趟表里每一条都现判、一条都不记库"
  ((${#selftest_plan_failure[@]})) && selftest_plan_failure_verdict "${selftest_plan_failure[@]}"
  if ((selftest_service_unavailable)); then
    rm -rf -- "${selftest_plan_scratch:?}"
    echo "  ✗ research 脚本的自证这一格没判：prune-server 不可用（上面那一句），表里 ${#table_runners[@]} 条一条都没起，这一格退 75"
    echo "     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check research-script-selftests"
    exit 75
  fi
fi
if ((${#runners[@]} == 0)); then
  rm -rf -- "${selftest_plan_scratch:?}"
  echo "  ✓ research 脚本的自证都通过（表取自 $table_source；表里 ${#table_runners[@]} 条都有键相同的判定，全部复用，这一格一条都没起：现判 0、复用 ${selftest_reused}、拒算 0、本次未跑 0）"
  exit 0
fi

if ! thread_cap="$(heavy_thread_default)"; then
  echo "  ✗ 取不了每台默认重活线程数（原因在上一行），自证分批的并行度定不了，一条自证都没跑"
  echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
  exit 1
fi
batch_size=$((thread_cap - ${#alone_set[@]}))
((batch_size >= 1)) || batch_size=1
selftest_memory_bound_count=0
for runner in "${runners[@]}"; do
  [[ -n "${selftest_memory_bound_set[$runner]+set}" ]] && selftest_memory_bound_count=$((selftest_memory_bound_count + 1))
done
((selftest_memory_bound_count)) && echo "  · 这一趟要起的里面，登记的大自证 $selftest_memory_bound_count 条各占一批，不与别的自证同批（SELFTEST_RUNNERS_MEMORY_BOUND）"

# 这一趟要起的里面有包装写法的：开跑前取一次 prune-server 二进制，交给每一条（这一格文件头「只在临时 prune-server 上跑的自证」）
selftest_server_binary=""; selftest_wrapped_count=0
for runner in "${runners[@]}"; do [[ "$runner" == "$SELFTEST_TEMPORARY_SERVICE_PREFIX "* ]] && selftest_wrapped_count=$((selftest_wrapped_count + 1)); done
if ((selftest_wrapped_count)); then
  if ! selftest_server_binary="$(code_tooling_prune_server_binary "$selftest_plan_scratch" 2>"$selftest_plan_scratch/server-binary.err")"; then
    echo "  ✗ 取不到 prune-server 二进制（$(tail -n 3 "$selftest_plan_scratch/server-binary.err" | tr '\n' ' ' | clip_text 400)），要包在临时服务底下起的 $selftest_wrapped_count 条一条都没起"
    echo "     → 怎么办：修好 prune-server/ 的编译（cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server）再跑这一格；二进制不另指，这一格自己编一份"
    exit 1
  fi
  echo "  · 临时 prune-server 的二进制：$selftest_server_binary（这一趟取一次，交给包装写法的 $selftest_wrapped_count 条）"
fi
scratch="$(mktemp -d)" || { echo "  ✗ 建不了放逐条输出与退出码的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格；一条自证都没跑。"; exit 1; }
started_pids=()
selftest_process_guard="$STAGE_REPOSITORY/scripts/gate/selftest-process-guard.py"
for index in "${!runners[@]}"; do
  [[ -n "${alone_set[${runners[$index]}]+set}" && -z "${selftest_memory_bound_set[${runners[$index]}]+set}" ]] && start_selftest "$index" "${runners[$index]}"
done
alone_pids=(${started_pids[@]+"${started_pids[@]}"})
batch=(); batches=0
run_selftest_batch() {
  local index pid
  started_pids=()
  for index in "${batch[@]}"; do start_selftest "$index" "${runners[$index]}"; done
  # 退出码在各自的 .exit 文件里，全部跑完按表次序逐条回读；这里只等这一批收齐
  for pid in "${started_pids[@]}"; do wait "$pid"; done
  batches=$((batches + 1))
  batch=()
}
for index in "${!runners[@]}"; do
  if [[ -n "${selftest_memory_bound_set[${runners[$index]}]+set}" ]]; then
    ((${#batch[@]})) && run_selftest_batch
    batch=("$index")
    run_selftest_batch
    continue
  fi
  [[ -n "${alone_set[${runners[$index]}]+set}" ]] && continue
  batch+=("$index")
  ((${#batch[@]} == batch_size)) && run_selftest_batch
done
((${#batch[@]})) && run_selftest_batch
for pid in ${alone_pids[@]+"${alone_pids[@]}"}; do wait "$pid"; done

collected=0; failed_count=0; uncollected=(); watched_count=0; unwatched=(); unjudged=(); untraced=(); not_run_runners=()
for index in "${!runners[@]}"; do
  runner="${runners[$index]}"
  if [[ ! -f "$scratch/$index.exit" ]]; then uncollected+=("$runner"); continue; fi
  collected=$((collected + 1))
  selftest_outcome "$index" "$runner"
  rc="$selftest_rc"; guard_verdict="$selftest_guard_verdict"
  if [[ -f "$scratch/$index.record-exit" && "$(<"$scratch/$index.record-exit")" != 0 ]]; then
    echo "  ! 「$runner」跑完当场记库没成（$(tail -n 2 "$scratch/$index.record" | tr '\n' ' ')），收尾那一趟再记一次"
  fi
  case "$guard_verdict" in
    watched$'\t'*)
      watched_count=$((watched_count + 1))
      # 判定文件末尾的 trace=no：守卫没能经 strace 跟踪 execve（strace 不在或起不了），这一条只有采样与替身那一半，不当绿报
      [[ "$guard_verdict" == *$'\ttrace=no'* ]] && untraced+=("$runner") ;;
    not-watched$'\t'*) unwatched+=("$runner") ;;
    *) unjudged+=("$runner") ;;
  esac
  if [[ "$selftest_result" == not-run ]]; then
    # 本次未跑：不判绿也不判红、不记库，下面逐条列名、报进汇总（这一格文件头「判据」）
    not_run_runners+=("$runner")
  elif [[ "$selftest_result" == red ]]; then
    failed_count=$((failed_count + 1))
    printf '%s\n' "$runner" >>"$selftest_plan_scratch/red.names"   # 判红：跑完当场已照键记红、撤绿（selftest_record_one），收尾 selftest_record_verdicts 连同复用哨兵再交一次
    echo "  ✗ $runner 没过（退出码 $rc）："
    sed 's/^/    /' "$scratch/$index.out"   # gate-lint:detail
    ((selftest_plan_ok)) && echo "    这一条这一趟为什么现判：${selftest_reason[$runner]:-（计划里没有它）}"
    echo "  → 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。"
  elif [[ "$selftest_result" == green-record ]]; then
    # 现判而判绿、守卫给了判定（盯了而 execve 跟踪没跑的不记：那一条只验了一半）：跑完当场已记（selftest_record_one），收尾连同复用哨兵再交一次
    printf '%s\n' "$runner" >>"$selftest_plan_scratch/green.names"
  fi
done
rm -rf "${scratch:?}"
if ((selftest_plan_ok)); then
  selftest_record_verdicts "$selftest_plan_scratch" "$selftest_plan_scratch/green.names" "$selftest_plan_scratch/red.names"
  ((failed_count)) && selftest_reread_after_red "$selftest_plan_scratch"
fi
if ((collected != ${#runners[@]})); then
  echo "  ✗ runner 表派出去 ${#runners[@]} 条，只收回 $collected 份退出码；没收回的：${uncollected[*]}"
  echo "     → 怎么办：临时目录在跑的过程中被删了或写不进去；看 \$TMPDIR 再重跑这一格，收不全的一轮不算判过。"
  exit 1
fi
if ((${#unjudged[@]})); then
  echo "  ✗ scripts/gate/selftest-process-guard.py 没给 ${#unjudged[@]} 条写判定（盯没盯说不清）：$(printf '「%s」' "${unjudged[@]}")"
  echo "     → 怎么办：单跑 python3 scripts/gate/selftest-process-guard.py --verdict-file <文件> -- <那条命令> 看它报什么；判定不全的一轮不算判过。"
  exit 1
fi
not_run_text=""
if ((${#not_run_runners[@]})); then
  not_run_text="：$(printf '「%s」' "${not_run_runners[@]}")"
  echo "  · 本次未跑 ${#not_run_runners[@]} 条（自证退 77：自己造不出前提，不算通过、不记库，下一趟照跑）$not_run_text"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    for runner in "${not_run_runners[@]}"; do printf '%s\n' "$STAGE_NAME 的 research-script-selftests 格：$runner 退 77，本次未跑" >> "$GATE_NOT_RUN_FILE"; done
  fi
fi
if ((failed_count)); then
  echo "  ✗ runner 表 ${#runners[@]} 条里 $failed_count 条没过（上面按表次序逐条列着）；本次未跑 ${#not_run_runners[@]} 条$not_run_text"
  echo "     → 怎么办：逐条按那份自证给的下一步修；只在并行下红、单跑绿的，是几份自证抢同一份状态，照这一格文件头那一段办。"
  exit 1
fi
if ((selftest_service_unavailable)); then
  echo "  ✗ research 脚本的自证这一格：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75"
  echo "     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check research-script-selftests"
  exit 75
fi
if ((selftest_plan_red)); then
  echo "  ✗ research 脚本的自证这一格判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；表里 ${#table_runners[@]} 条照旧全部现判跑完、都过了"
  echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check research-script-selftests；不能复用就判红，不退回全部现判当绿"
  exit 1
fi
unwatched_text=""
((${#unwatched[@]})) && unwatched_text="：$(printf '「%s」' "${unwatched[@]}")"
untraced_text=""
((${#untraced[@]})) && untraced_text="；其中 ${#untraced[@]} 条的 execve 跟踪本次未跑（strace 不在或起不了，只有采样与替身那一半）：$(printf '「%s」' "${untraced[@]}")"
selftest_counts="表里 ${#table_runners[@]} 条：现判 ${selftest_judged}、复用 ${selftest_reused}、拒算 ${selftest_refused}、本次未跑 ${#not_run_runners[@]}"
((selftest_plan_ok)) || selftest_counts="表里 ${#table_runners[@]} 条，按条复用算不出、全部现判，本次未跑 ${#not_run_runners[@]}"
if ((${#not_run_runners[@]} == ${#runners[@]} && selftest_reused == 0)); then
  echo "  ! research 脚本的自证这一趟一条都没判过：起的 ${#runners[@]} 条都退 77（本次未跑$not_run_text），也没有复用的；这一格记本次未跑，不记通过"
  echo "     → 怎么办：照那几条自证打的原因造出前提（机器闲一点再跑这类），再跑 bash .claude/gate.d/code-tooling.sh --check research-script-selftests"
  exit 77
fi
selftest_verdict_text="research 脚本的自证都通过"
((${#not_run_runners[@]})) && selftest_verdict_text="research 脚本的自证判了的都通过、本次未跑 ${#not_run_runners[@]} 条$not_run_text"
echo "  ✓ ${selftest_verdict_text}（这一格跑了 ${#runners[@]} 条，表取自 $table_source；${selftest_counts}；单独起 ${#alone_pids[@]} 条，其余 $((${#runners[@]} - ${#alone_pids[@]})) 条按每批至多 $batch_size 条并行、共 $batches 批；登记过的 $watched_count 条经 selftest-process-guard.py 盯着进程树、经 strace 记下每一次 execve，没起真 cargo / rustc / qemu / herd7 / verus（版本查询除外）与真清场，本次未盯 ${#unwatched[@]} 条$unwatched_text$untraced_text）"
}

# ── 格 temporary-service-runners：runner 表里套了临时服务包装的自证，与要临时服务的表里登记成 required 的两边对齐
#
# 判据：.claude/hooks/selftest-temporary-service.tsv（读法在 TEMPORARY_SERVICE_TABLE 的注释，一份表、认 required 与 own-server 两种）与 runner 表（SELFTEST_RUNNERS；样本根的 .selftest-runners 换掉它，
#   和 research-script-selftests 格同一个取法 selftest_runner_table_load）逐条对一遍，对不上的逐条点名，整格判红：
#   ① 表里 required 而 runner 表里裸写（该写成包装写法；research-script-selftests 格也在起任何一条之前拒，同一个函数 temporary_service_bare_runners）；
#   ② 表里 required 而 runner 表里一条都没有（没人跑它的自证）；
#   ③ runner 表里写成包装写法而表里没有这一行（包装只给表里登记的套）。
#   own-server 的两份不许被要求包装：runner 表里包着它们只为把二进制交下去，包不包都不判，源码里有没有 $SELFTEST_TEMPORARY_SERVICE_MARK 也不看。
#   表不在（样本根可以不放，当作空表）、读不了或有写坏的行：红，出路写在 temporary_service_table_load 里。
# 样本：temporary-service-runners-red（样本根的表四行：一份 required 而 runner 表裸写、一份 required 而 runner 表里没有、一份 required 且写成包装写法、一份 own-server 包着；
#   runner 表另有一条包装写法而表里没有这一行：①②③ 各一处，必须判红并逐条点名，另两份不点名）；
#   temporary-service-runners-green（required 的包着、own-server 的一份包着一份裸写，裸写那份源码里有那几个字，runner 表里另有一条不在表里的裸写：必须判绿，报 required、own-server 与包装写法各几份）。
# 弄坏开关 CODE_TOOLING_BREAK=temporary-service-alignment-unchecked：不判两边对不对得上（只有 research-script-selftests 格看裸写），temporary-service-runners-red 判绿（期望退出 1，实测 0）；
#   =temporary-service-table-ignored（文件头开关表）：认 required 退回看源码，temporary-service-runners-red 里「该写成」那条 want 找不到，temporary-service-runners-green 判红（own-server 裸写那份被点名）。
cell_temporary_service_runners() {
  selftest_runner_table_load
  temporary_service_table_load || exit 1
  local runner script
  local -a words=() bare=() unregistered=() absent=()
  local -A runner_scripts=()
  local required_count=0 own_server_count=0 wrapped_count=0 problem_count
  mapfile -t bare < <(temporary_service_bare_runners)
  for runner in "${runners[@]}"; do
    selftest_inner_into "$runner"; read -r -a words <<<"$selftest_inner"
    script="${words[1]:-空}"
    runner_scripts[$script]=1
    [[ "$runner" == "$SELFTEST_TEMPORARY_SERVICE_PREFIX "* ]] || continue
    wrapped_count=$((wrapped_count + 1))
    [[ -n "${TEMPORARY_SERVICE_JUDGEMENT[$script]:-}" ]] || unregistered+=("$runner")
  done
  while IFS= read -r script; do
    [[ -n "$script" ]] || continue
    if [[ "${TEMPORARY_SERVICE_JUDGEMENT[$script]}" == required ]]; then
      required_count=$((required_count + 1))
      [[ -n "${runner_scripts[$script]+set}" ]] || absent+=("$script")
    else
      own_server_count=$((own_server_count + 1))
    fi
  done < <(printf '%s\n' "${!TEMPORARY_SERVICE_JUDGEMENT[@]}" | sort)
  problem_count=$((${#bare[@]} + ${#absent[@]} + ${#unregistered[@]}))
  if ((problem_count)) && [[ "${CODE_TOOLING_BREAK:-}" != temporary-service-alignment-unchecked ]]; then
    echo "  ✗ runner 表与要临时服务的表 $TEMPORARY_SERVICE_TABLE 有 $problem_count 处对不上（runner 表 ${#runners[@]} 条取自 $table_source，其中套包装的 $wrapped_count 条；表里 required $required_count 份、own-server $own_server_count 份）："
    for runner in ${bare[@]+"${bare[@]}"}; do echo "      「$runner」裸写，表里是 required：该写成「$SELFTEST_TEMPORARY_SERVICE_PREFIX $runner」"; done   # gate-lint:detail
    for script in ${absent[@]+"${absent[@]}"}; do echo "      $script：表里是 required，runner 表里一条都没有（没人跑它的自证）：补上「$SELFTEST_TEMPORARY_SERVICE_PREFIX <解释器> $script --selftest」"; done   # gate-lint:detail
    for runner in ${unregistered[@]+"${unregistered[@]}"}; do echo "      「$runner」写成了包装写法，表里没有这一行：包装只给表里登记的套"; done   # gate-lint:detail
    echo "     → 怎么办：照上面逐条改 runner 表（这一道的 SELFTEST_RUNNERS）：required 的写成包装写法、缺的补上；包装写法而表里没有的，要么这份自证确实只在临时 prune-server 上跑（先在 .claude/hooks/selftest-registry.tsv 登记，再往 $TEMPORARY_SERVICE_TABLE 加一行 required 或 own-server），要么去掉包装；own-server 的不要求包装"
    exit 1
  fi
  echo "  ✓ 要临时服务的表与 runner 表两边对齐：表里 required ${required_count} 份都在 runner 表里写成包装写法，own-server ${own_server_count} 份不要求包装；runner 表 ${#runners[@]} 条，套包装的 ${wrapped_count} 条都在表里"
}

# ── 格 temporary-service-wrapper-form：要临时服务的自证的整条写法，各处逐字相同、包装的解析收得下、不带尖括号占位的 --server-binary
#
# 判据：整条写法散在几处（读表的代码都跳过 # 行，没有别的检查绑住它），这一格逐处取出来与包装自己认的用法比；只读文件，不起 prune-server、不起 cargo。
#   比的对象是包装 scripts/prune-server/prune_server_client_selftest.py：文件头用法行「<解释器> <包装> --with-temporary-service [--server-binary <…>] -- <命令> [<参数>…]」去掉方括号里的可选项再接上 --，
#   就是整条该有的前缀（不带 --server-binary：任务服务把整条认成会编的、给编译目录，自己编了的二进制才另加 --server-binary，写在整条之外）；
#   with_temporary_service 里给 unknown 赋值的那一句（认不出的参数退 2）原样取出来，对每一处的选项求值，这里不另抄一份解析。
#   取包装写法的法子：找「<路径以 prune_server_client_selftest.py 结尾的词> --with-temporary-service」，往后取到第一个单独的 -- 为止，前面粘着的解释器词连在一起当前缀；
#   后面没有单独 -- 的是行文里提到它，不算整条；选项恰好是用法行里的 [--server-binary <…>] 的是用法写法，也不算整条。
#   取的七处（红行点名是哪一处、文件与行、原文）：
#     登记表文件头：.claude/hooks/selftest-temporary-service.tsv 的注释行；
#     拒绝说明：.claude/hooks/lib_project_code_runs.py 的 temporary_service_command 对一个 .py 键与一个 .sh 键各打的整条（载入库调它，不读源码）；
#     骨架：scripts/agents/dispatch-prompt.py 里给 TEMPORARY_SERVICE_WRAPPER 赋的字符串；
#     客户端文件头：scripts/prune-server/prune_server_client.py 的文件头；
#     自证登记表：.claude/hooks/selftest-registry.tsv 的注释行，加上要临时服务的表里登记成 required 的那几行的理由（own-server 行的理由说它自己怎么起真服务，不判）；
#     runner 表：这一道的 SELFTEST_RUNNERS（样本根的 .selftest-runners 换掉它）里套了包装的每一条；
#     门禁的包装前缀常量：SELFTEST_TEMPORARY_SERVICE_PREFIX。
#   判红四种：选项里带尖括号（`--server-binary <编好的 prune-server>` 这类占位，照抄进 shell 尖括号被当成输入重定向、命令起不来）；包装的解析认不出这些选项；
#   前缀与整条该有的逐字不同（含写了 --server-binary）；某一处一条整条写法都没有。包装文件读不出用法行或认选项的那一句时，整格红、点名包装。
#   表不在、读不了或有写坏的行：红，出路写在 temporary_service_table_load 里。
# 样本：temporary-service-wrapper-form-green（七处都对：自证登记表里 own-server 行的理由带 --server-binary <…> 占位，不判；客户端文件头另有一行用法写法，不算整条）；
#   -binary-placeholder-red（自证登记表里 required 行的理由写成带 --server-binary <编好的 prune-server> 占位，只点名那一处）；
#   -prefix-differs-red（骨架的包装前缀少了 scripts/，只点名骨架）；
#   -unparsable-red（runner 表一条写了 --memory 4G，包装认不出，只点名 runner 表）；
#   -place-missing-red（拒绝说明打的是裸命令、没有包装，两个键各点名一次）。
# 弄坏开关 CODE_TOOLING_BREAK=temporary-service-form-unchecked：有问题也不判红（没有检查绑住这几处），四份红样本判绿（期望退出 1，实测 0）。
cell_temporary_service_wrapper_form() {
  selftest_runner_table_load
  temporary_service_table_load || exit 1
  local output status script required_text runner_text kind text canonical="" tally=""
  local -a problem_lines=()
  required_text="$(for script in "${!TEMPORARY_SERVICE_JUDGEMENT[@]}"; do [[ "${TEMPORARY_SERVICE_JUDGEMENT[$script]}" == required ]] && printf '%s\n' "$script"; done | sort)"
  runner_text="$(printf '%s\n' ${runners[@]+"${runners[@]}"})"
  output="$(python3 - "$SELFTEST_TEMPORARY_SERVICE_PREFIX" "$required_text" "$runner_text" "${table_source%%（*}" <<'PY_WRAPPER_FORM'
import ast, importlib.util, re, sys

sys.dont_write_bytecode = True   # 载入被判的钩子库不在样本根与真仓里留 __pycache__
constant_prefix, required_text, runner_text, runner_label = sys.argv[1:5]
WRAPPER_PATH = "scripts/prune-server/prune_server_client_selftest.py"
CLIENT_PATH = "scripts/prune-server/prune_server_client.py"
DISPATCH_PATH = "scripts/agents/dispatch-prompt.py"
LIBRARY_PATH = ".claude/hooks/lib_project_code_runs.py"
TABLE_PATH = ".claude/hooks/selftest-temporary-service.tsv"
REGISTRY_PATH = ".claude/hooks/selftest-registry.tsv"
USAGE_LINE = re.compile(r"^[ \t]*(?P<head>\S+[ \t]+\S*prune_server_client_selftest\.py[ \t]+--with-temporary-service)(?:[ \t]+\[[^\]\n]*\])?[ \t]+--[ \t]", re.M)
ANCHOR = re.compile(r"(?:(?P<launcher>\S+)[ \t]+)?(?P<path>\S*prune_server_client_selftest\.py)[ \t]+--with-temporary-service(?P<rest>[^\n]*)")
LEADING = "\"'`(["
CLOSERS = "\"'`」』”）)"
problems = []
tallies = {}


def fail(place, location, found, reason):
    problems.append((place, location, found, reason))


def finish():
    print(f"CANONICAL\t{canonical}")
    print("TALLY\t" + "、".join(f"{place} {wholes} 条" + (f"（另有用法行 {usage} 行）" if usage else "") for place, (wholes, usage, _mentions) in tallies.items()))
    for place, location, found, reason in problems:
        print(f"PROBLEM\t{place}（{location}）：原文「{found}」，{reason}")
    sys.exit(1 if problems else 0)


def read_text(path, place):
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read()
    except (OSError, UnicodeDecodeError) as error:
        fail(place, path, "（读不到）", f"{type(error).__name__}：{error}")
        return None


def parse_tree(text, path, place):
    if text is None:
        return None
    try:
        return ast.parse(text)
    except SyntaxError as error:
        fail(place, path, "（语法错）", f"{error}")
        return None


# 包装自己的用法行与认选项的那一句：整条该是什么、认得下哪些选项都从这里取，这一格不另抄一份
canonical = "（读不出）"
unknown_of = None
wrapper_source = read_text(WRAPPER_PATH, "包装")
wrapper_tree = parse_tree(wrapper_source, WRAPPER_PATH, "包装")
if wrapper_tree is not None:
    usage = USAGE_LINE.search(ast.get_docstring(wrapper_tree, clean=False) or "")
    if usage is None:
        fail("包装", WRAPPER_PATH, "（文件头里找不到用法行）", "文件头要有一行「<解释器> <包装> --with-temporary-service -- <命令> [<参数>…]」（可选项写在方括号里），这一格从它取整条的前缀")
    else:
        canonical = usage.group("head") + " --"
    for function in ast.walk(wrapper_tree):
        if isinstance(function, ast.FunctionDef) and function.name == "with_temporary_service":
            for statement in ast.walk(function):
                if isinstance(statement, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "unknown" for target in statement.targets):
                    unknown_code = compile(ast.Expression(statement.value), WRAPPER_PATH, "eval")
                    unknown_of = lambda options, code=unknown_code: eval(code, {"options": options})
    if unknown_of is None:
        fail("包装", WRAPPER_PATH, "（找不到）", "with_temporary_service 里找不到给 unknown 赋值的那一句，认不出的选项怎么判这一格读不到")
if wrapper_tree is None or canonical == "（读不出）":
    finish()


def judge_options(options, text):
    if any("<" in token or ">" in token for token in options):
        return "带尖括号占位的 --server-binary：照抄进 shell 时尖括号被当成输入重定向、命令起不来；整条不带 --server-binary，编好的二进制写在整条之外"
    unknown = unknown_of(options) if options and unknown_of is not None else []
    if unknown:
        return f"包装的解析认不出 {' '.join(unknown)}（认不出的参数退 2）"
    if text != canonical:
        return "整条不带 --server-binary，编好的二进制写在整条之外" if options else "前缀与包装用法行写的不同"
    return None


def ascii_tail(word):
    """词前面粘着的全角括号与引号、行文里的汉字（「的包装写法（python3」）不算词：取最后一个非 ASCII 字符之后的那一截，再去掉 ASCII 的引号与括号。"""
    return re.sub(r"^.*[^\x00-\x7f]", "", word).lstrip(LEADING)


def scan(place, lines):
    """lines 是 (位置, 一行文字)：逐行找包装写法，取到第一个单独的 -- 为止的那一段当前缀；后面没有单独 -- 的是行文里提到它，不算整条；
    选项整个写在方括号里（[--server-binary <…>]，用法行里表示可选项的写法）的是用法写法，也不算整条。"""
    wholes = usage = prose = 0
    for location, line in lines:
        for found in ANCHOR.finditer(line):
            tokens = found.group("rest").split()
            separator = next((index for index, token in enumerate(tokens) if token.rstrip(CLOSERS) == "--"), None)
            if separator is None:
                prose += 1
                continue
            options = tokens[:separator]
            if options and options[0].startswith("[") and options[-1].endswith("]"):
                usage += 1
                continue
            wholes += 1
            text = " ".join(word for word in (ascii_tail(found.group("launcher") or ""), ascii_tail(found.group("path")), "--with-temporary-service", *options, "--") if word)
            reason = judge_options(options, text)
            if reason:
                fail(place, location, text, reason)
    total = tallies.setdefault(place, [0, 0, 0])
    total[0] += wholes
    total[1] += usage
    total[2] += prose
    return wholes


def scan_each(place, lines, empty_location):
    if scan(place, lines) == 0:
        fail(place, empty_location, "（没有）", "这一处没有写出整条包装写法")


def numbered(text, path, first=1):
    return [(f"{path}:{number}", line) for number, line in enumerate(text.split("\n"), first)]


table_text = read_text(TABLE_PATH, "登记表文件头")
if table_text is not None:
    scan_each("登记表文件头", [(location, line) for location, line in numbered(table_text, TABLE_PATH) if line.lstrip().startswith("#")], TABLE_PATH)

library_source = read_text(LIBRARY_PATH, "拒绝说明")
library_tree = parse_tree(library_source, LIBRARY_PATH, "拒绝说明")
if library_tree is not None:
    definitions = [node.lineno for node in ast.walk(library_tree) if isinstance(node, ast.FunctionDef) and node.name == "temporary_service_command"]
    try:
        spec = importlib.util.spec_from_file_location("lib_project_code_runs_for_wrapper_form", LIBRARY_PATH)
        library = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(library)
        for key in ("scripts/gate/sample-script.py", "scripts/gate/sample-script.sh"):
            location = f"{LIBRARY_PATH}:{definitions[0] if definitions else 0} temporary_service_command(\"{key}\")"
            scan_each("拒绝说明", [(location, library.temporary_service_command(key))], location)
    except Exception as error:   # 库载不进来、函数不在、调用抛异常都算这一处读不到
        fail("拒绝说明", LIBRARY_PATH, "（取不到）", f"{type(error).__name__}：{error}")

dispatch_tree = parse_tree(read_text(DISPATCH_PATH, "骨架"), DISPATCH_PATH, "骨架")
if dispatch_tree is not None:
    wrappers = [(node.lineno, node.value.value) for node in dispatch_tree.body
                if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "TEMPORARY_SERVICE_WRAPPER" for target in node.targets)
                and isinstance(node.value, ast.Constant) and isinstance(node.value.value, str)]
    if wrappers:
        scan_each("骨架", [(f"{DISPATCH_PATH}:{wrappers[0][0]} TEMPORARY_SERVICE_WRAPPER", wrappers[0][1])], DISPATCH_PATH)
    else:
        fail("骨架", DISPATCH_PATH, "（找不到）", "找不到给 TEMPORARY_SERVICE_WRAPPER 赋字符串的那一句，骨架拼的整条取自它")

client_tree = parse_tree(read_text(CLIENT_PATH, "客户端文件头"), CLIENT_PATH, "客户端文件头")
if client_tree is not None:
    header = ast.get_docstring(client_tree, clean=False)
    if header is None:
        fail("客户端文件头", CLIENT_PATH, "（没有文件头）", "客户端要有文件头，「命令行」一段给整条写法")
    else:
        scan_each("客户端文件头", numbered(header, CLIENT_PATH, client_tree.body[0].lineno), CLIENT_PATH)

# 自证登记表：文件头的注释行，加上要临时服务的表里登记成 required 的那几行的理由；own-server 行的理由说的是它自己怎么起真服务，不判
registry_text = read_text(REGISTRY_PATH, "自证登记表")
if registry_text is not None:
    required = set(required_text.split("\n"))
    scan_each("自证登记表", [(location, line) for location, line in numbered(registry_text, REGISTRY_PATH)
                            if line.lstrip().startswith("#") or line.split("\t", 1)[0] in required], REGISTRY_PATH)

scan_each("runner 表", [(f"{runner_label} 第 {number} 条", runner) for number, runner in enumerate(runner_text.split("\n"), 1) if runner], runner_label)
scan_each("门禁的包装前缀常量", [("code-tooling.sh 的 SELFTEST_TEMPORARY_SERVICE_PREFIX", constant_prefix)], "code-tooling.sh 的 SELFTEST_TEMPORARY_SERVICE_PREFIX")
finish()
PY_WRAPPER_FORM
  )"
  status=$?
  while IFS=$'\t' read -r kind text; do
    case "$kind" in
      CANONICAL) canonical="$text" ;;
      TALLY) tally="$text" ;;
      PROBLEM) problem_lines+=("$text") ;;
    esac
  done <<<"$output"
  if ((status > 1)) || [[ -z "$canonical" ]]; then
    echo "  ✗ 要临时服务的自证的整条写法判不了（python 退 $status，没有打出整条该有的前缀）：${output:-（没有输出）}"   # gate-lint:detail
    echo "     → 怎么办：看上面 python 的报错；判法与取的几处写在 code-tooling.sh 里这一格函数上方的注释"
    exit 1
  fi
  if ((${#problem_lines[@]})) && [[ "${CODE_TOOLING_BREAK:-}" != temporary-service-form-unchecked ]]; then
    echo "  ✗ 要临时服务的自证的整条写法有 ${#problem_lines[@]} 处对不上包装的用法（用法行与认选项的那一句取自 scripts/prune-server/prune_server_client_selftest.py 文件头；整条该是「$canonical <解释器> <脚本> --selftest」）："
    for text in "${problem_lines[@]}"; do echo "      $text"; done   # gate-lint:detail
    echo "     → 怎么办：把点名的那一处改回整条该有的写法「$canonical <解释器> <脚本> --selftest」（整条不带 --server-binary、不写尖括号占位；自己编了、两台都读得到的二进制另加 --server-binary，写在整条之外）；用法以包装文件头那一行为准，包装改了用法就各处一起改；新增了写整条的一处，加进这一格的取法（这一格函数上方的注释列了全部几处）"
    exit 1
  fi
  echo "  ✓ 要临时服务的自证的整条写法各处逐字相同（前缀「$canonical」，不带 --server-binary），包装的解析收得下：$tally"
}

# ── 格 research-script-selftest-coverage：scripts/ 里代码实现了 --selftest 的脚本，都有门禁阶段在跑或登记了为什么不跑
#
# 判据：scripts/ 里实现了 --selftest 的脚本（递归到装置目录那一层），要么有阶段在调它（脚本名后直接跟 --selftest，或先赋给变量再 "$变量" --selftest；
#   脚本名取它在这一批里唯一的最短路径尾，左边不许紧挨字母、数字、点与连字符：crash-point-replay.sh 不算调了 replay.sh），
#   要么登记进 NOT_RUN_HERE 写明为什么；NOT_RUN_HERE 里那份脚本已经有阶段在跑、或不再实现 --selftest 的，是过期豁免，也红。
# 「实现了」只认 .sh / .py 经 code_of 去掉注释与出路文字之后的代码，不列具体写法（==、case、match 都认）；行尾注释同样去掉（shell 不在引号里、前面是空白的 #
#   之后那一段，python 的注释记号；例：共用库函数头后面「# 调用方的 --selftest 起的：…」不算实现了）；「有阶段在调」只认 code_of 之后的代码。
# 阶段取门禁目录顶层全部 *.sh，与 gate.sh 起的是同一批。
# 扫哪些：scripts/ 下全部文件（隐藏的目录与文件不进，叫 target 的编译产物目录不进：里面没有 .sh 与 .py，几千份二进制不整读）；先看后缀、是 .sh / .py 才打开。
#   每道阶段的代码只扫一遍，找出全部「<词> --selftest」与「变量="…py/sh"」，再按路径尾对每份脚本查，不为每道阶段、每份脚本各跑一次全文正则。
#   弄坏开关 CODE_TOOLING_BREAK=coverage-scans-build-output：进 target 目录，coverage-claims-green（setup.sh 在 scripts/gate/target/release/ 下造一份写着 --selftest 的 .sh）判错。
#
# 样本：red 放一份只在某个阶段的 echo 出路里被提到的 --selftest 脚本，必须报它没人跑；green 放一份真被阶段调用的与 NOT_RUN_HERE 每一行的那份脚本，判绿并报对数。
#   NOT_RUN_HERE 是这一道自己的表，绿样本里缺了哪一行的脚本，那一行就判过期：往表里加一行，同时往 green、coverage-claims-green 里各放一份那个路径的脚本、改两份 expect 的份数。
#   coverage-claims-green 放一份只在注释、行尾注释与出路里提 --selftest 的 .py、一份只在行尾注释里提它的共用库 .sh、一份提到它的 .tsv（都不算实现了）
#   与一份由不带编号的阶段 tooling-runs.sh 在跑的 .sh，
#   判绿并报「声称有 --selftest 的 N 份中 1 份有门禁阶段在跑」，其余 N − 1 份就是 NOT_RUN_HERE 的每一行（N 随表的行数走，不写死）；coverage-claims-red 放一份用 match 语句实现 --selftest、没人跑的 .py，必须报它没人跑。
#   coverage-nested-red 放一份装置目录那一层的 scripts/experiment-apparatus/sample_rig/drive.py（实现了 --selftest、没人跑），必须报它没人跑；
#   coverage-suffix-red 放 scripts/experiment-apparatus/replay.sh 与 scripts/crash/crash-point-replay.sh 两份，阶段只跑后一份的自证，必须报前一份没人跑。
NOT_RUN_HERE=(
  "scripts/crash/lib_crash_point_replay_items.py	它自己没有自证：用法里提到的 --selftest 指 scripts/crash/crash-case-check.py 的自证（那一条已在这张表里）"
  "scripts/kb/lib_notation_check.py	它自己没有自证：只被 import，调它的三个脚本（replace-once.py、insert-row.py、replace-batch.py）各自的 --selftest 走一遍拒绝与放行"
  "scripts/mutation/lib_mutation_units.py	它自己没有自证：只被 import、没有命令行入口，判法由 crates-mutation-rows.py 与 mutate.sh 两边的 --selftest 覆盖"
  "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/fake_row_driver.py	它自己没有自证：rows_orchestrator.py --selftest 起它当假的逐行驱动"
  "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/revision_8_selftest_harness.py	它自己没有自证：只被修订 8 那几份 import，没有命令行入口，起的是它们各自的 --selftest"
  "scripts/gate/large-file-check.py	它自己没有自证：文件头那句「这份脚本没有 --selftest」被算成实现了；判法由这一道 committed-file-size 格的五份样本与五条弄坏开关证红"
  "scripts/kb/lib_atomic_replace.py	它自己没有自证：只被 import，代码里的 --selftest 是认祖先进程命令行的那个串；自证在 replace-once.py、insert-row.py、replace-batch.py 各自的 --selftest"
  "scripts/prune-server/prune_server_client.py	它自己没有自证：命令行认到 --selftest 只为退 2、指到 prune_server_client_selftest.py（那一份的 --selftest 在 runner 表里）"
  "scripts/experiment-apparatus/e203_admission_scan_cost_library.py	它自己没有自证：只由入口 e203-admission-scan-cost-run.sh 起，文中的 --selftest 是入口自证的起法与弄坏开关的说明；判法由入口 --selftest（经临时 prune-server，在 runner 表里）覆盖"
  "scripts/experiment-apparatus/e206_fixed_point_cost_library.py	它自己没有自证：只由入口 e206-fixed-point-cost-run.sh 起，文中的 --selftest 是入口自证的起法与弄坏开关的说明；判法由入口 --selftest（经临时 prune-server，在 runner 表里）覆盖"
  "scripts/experiment-apparatus/e207_strict_scan_steps_library.py	它自己没有自证：只由入口 e207-strict-scan-steps-run.sh 起，文中的 --selftest 是入口自证的起法与弄坏开关的说明；判法由入口 --selftest（经临时 prune-server，在 runner 表里）覆盖"
  "scripts/experiment-apparatus/e209_spike_cause_library.py	它自己没有自证：只由入口 e209-spike-cause-run.sh 起，文中的 --selftest 是入口自证的起法与弄坏开关的说明；判法由入口 --selftest（经临时 prune-server，在 runner 表里）覆盖"
  "scripts/experiment-apparatus/e21_card_lease.py	它自己没有自证：只被 e21-compute.py、e21-prove-gpu.py、e21-transfer.py 与自证 import，直接执行只打用法；代码里认到 --selftest 只是弄坏开关的守卫的拒绝说明里提到 --selftest（那一个是自证自己起的）；判法由 e21-card-lease-selftest.py 的 --selftest（在 runner 表里）覆盖"
)
cell_research_script_selftest_coverage() {
coverage="$(python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" "$TREE_LISTING_LIBRARY" "${NOT_RUN_HERE[@]}" <<'PY_COVERAGE'
import glob, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
code_of = code_text.code_of
broken = sys.argv[3]
tree = types.ModuleType("lib_tree_listing")
exec(compile(sys.argv[4], "tree-listing-library", "exec"), tree.__dict__)
exempt = dict(row.split("\t", 1) for row in sys.argv[5:])
EVERY_STRING = re.compile(r"(?s).*")   # python 只去注释记号，字符串字面量一概留着（scan_strings）：实现 --selftest 的那个串就在字面量里
def without_trailing_comments(path, text):
    """去掉行尾注释（shell 不在引号里、前面是空白的 #，python 的注释记号）：行尾注释里提到的 --selftest 不算实现了。"""
    if broken == "selftest-coverage-trailing-comments-kept":
        return text
    if path.endswith(".sh"):
        return "\n".join(source_lines.shell_code_lines(text, code_text.OUTPUT_TEXT))
    return "\n".join(source_lines.python_code_lines(text, code_text.OUTPUT_TEXT, EVERY_STRING, scan_strings=True))
def implements_selftest(path):
    # 后缀先判、再打开：编译产物与别的二进制（scripts 下的 target 目录里几千份、几个 GiB）不整读；claims-any-file 下不看后缀
    if broken != "claims-any-file" and not path.endswith((".sh", ".py")):
        return False
    text = open(path, encoding="utf-8", errors="replace").read()
    if broken == "claims-any-file":
        return "--selftest" in text
    if "--selftest" not in text:
        return False
    return "--selftest" in code_of(without_trailing_comments(path, text))
if broken == "selftest-coverage-top-level-only":
    scanned = glob.glob("scripts/*/*")
elif broken == "coverage-scans-build-output":
    scanned = tree.files_under("scripts", skipped=())
else:
    scanned = tree.files_under("scripts")
claimed = sorted(path for path in scanned if os.path.isfile(path) and implements_selftest(path))
# 路径尾 → 有几份 claimed 以它收尾（按整段路径切）：只指一份的最短尾就是这份脚本在这一批里的名字
tail_count = {}
for other in claimed:
    parts = other.split("/")
    for size in range(1, len(parts) + 1):
        tail = "/".join(parts[-size:])
        tail_count[tail] = tail_count.get(tail, 0) + 1
def unique_tail(script):
    parts = script.split("/")
    for size in range(1, len(parts) + 1):
        tail = "/".join(parts[-size:])
        if tail_count[tail] == 1:
            return tail
    return script
tail_of = {script: (os.path.basename(script) if broken == "selftest-coverage-name-substring" else unique_tail(script)) for script in claimed}
left_boundary = "" if broken == "selftest-coverage-name-substring" else r"(?<![\w.-])"
run = set()
stage_pattern = "[0-9][0-9]-*.sh" if broken == "numbered-stages-only" else "*.sh"
SELFTEST_AFTER_SPACE = re.compile(r"\s+--selftest")
BOUNDARY_CHARACTER = re.compile(r"[\w.-]")
def tail_at_boundary(text, end, tail):
    """text[:end] 以 tail 收尾、且 tail 左边不紧挨字母、数字、点与连字符（left_boundary 那一条；name-substring 下不看左边）。"""
    start = end - len(tail)
    if start < 0 or text[start:end] != tail:
        return False
    return not left_boundary or start == 0 or BOUNDARY_CHARACTER.match(text[start - 1]) is None
for stage in sorted(glob.glob(os.path.join(".claude/gate.d", stage_pattern)) + glob.glob(".claude/gate.d/lib/code-tooling-*.sh")):
    code = code_of(open(stage, encoding="utf-8").read())
    variables = dict(re.findall(r'^\s*(\w+)="([^"\n]*\.(?:py|sh))"', code, re.M))
    # 阶段代码只扫一遍：每处「空白 --selftest」前面（可隔一个引号）收尾的词，是不是哪份脚本的尾；变量只看有没有被 "$变量" --selftest 起过
    selftest_ends = []
    for found in SELFTEST_AFTER_SPACE.finditer(code):
        selftest_ends.append(found.start())
        if found.start() and code[found.start() - 1] in "\"'":
            selftest_ends.append(found.start() - 1)
    invoked_values = [value for variable, value in variables.items() if re.search(r'"\$' + variable + r'"\s+--selftest', code)]
    for script in claimed:
        tail = tail_of[script]
        if any(tail_at_boundary(code, end, tail) for end in selftest_ends) or any(tail_at_boundary(value, len(value), tail) for value in invoked_values):
            run.add(script)
print(f"CLAIMED\t{len(claimed)}\tRUN\t{len(run)}")
for script in claimed:
    if script not in run and script not in exempt:
        print(f"MISSING\t{script}")
for path, why in exempt.items():
    if path not in claimed or path in run:
        print(f"STALE\t{path}\t{'已经有阶段在跑它' if path in run else '它的代码里已经没有 --selftest，或文件不在了'}")
    else:
        print(f"EXEMPT\t{path}\t{why}")
PY_COVERAGE
)" || { echo "  ✗ 自证覆盖的对账脚本没跑成"; echo "  → 怎么办：看上面 python 的报错修这一段；对账没跑等于这一格没验。"; exit 1; }
if grep -q '^MISSING' <<<"$coverage"; then
  echo "  ✗ 这些 research 脚本声称有 --selftest，却没有任何门禁阶段在跑它："
  grep '^MISSING' <<<"$coverage" | cut -f2 | sed 's/^/      /'   # gate-lint:detail
  echo "  → 怎么办：把它加进 research-script-selftests 格的 SELFTEST_RUNNERS；确实不该每轮跑的，登记进 NOT_RUN_HERE 并写明为什么——没人跑的自证只在写它的那天跑过一次。"
  exit 1
fi
if grep -q '^STALE' <<<"$coverage"; then
  echo "  ✗ NOT_RUN_HERE 里有过期的豁免："
  grep '^STALE' <<<"$coverage" | cut -f2,3 | sed 's/\t/：/; s/^/      /'   # gate-lint:detail
  echo "  → 怎么办：从 NOT_RUN_HERE 里删掉这一行；留着它会让人以为那份脚本还被绕开着。"
  exit 1
fi
read -r _ claimed_count _ run_count < <(grep '^CLAIMED' <<<"$coverage")
echo "  ✓ scripts/ 里声称有 --selftest 的 $claimed_count 份中 $run_count 份有门禁阶段在跑（只认 .sh / .py 代码里的 --selftest）"
echo "    没跑的 $(grep -c '^EXEMPT' <<<"$coverage") 份（登记在这一格的 NOT_RUN_HERE）："
grep '^EXEMPT' <<<"$coverage" | cut -f2,3 | sed 's/\t/：/; s/^/      /'
}
