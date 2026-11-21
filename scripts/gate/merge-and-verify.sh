#!/usr/bin/env bash
# admission: always 每一批补丁都要打一次、验一次，上一次的结论不替这一次作保
# run-condition: command git cargo rsync find xargs diff
# selftest-serial: 自证的各格共用一份假 cargo（格与格之间用 sed -i 换它的输出）与几棵样本树（narrow-tree、merge-tree 一格接一格地往里写、删补丁改后的文件，后一格读前一格留下的树与输出文件），次序本身是自证的一部分，拆成并行要先给每格各建一份假 cargo 与样本树
# 代码分块：本文件留函数之外的流程（参数解析、快照与编译目录的登记、各步的次序、收尾）、inside_selftest / settle_break_switches / clip_text / die，函数写在同目录几份被 source 的库里（只定义函数、不改 shell 选项），
#   入口在 here 算出来之后、各步开跑之前 source 它们；库里的函数读写入口的全局变量，同一个 shell 里一份：
#   merge-and-verify-snapshot-lib.sh    快照与槽位（take_snapshot、finish、各槽的挑选与回收）、进度行的后台进程
#   merge-and-verify-steps-lib.sh       补丁的核对、cargo 命令与 check.sh 的读法、测试目标的挑选、每一步的起法（run_step）、验证包那一步
#   merge-and-verify-items-lib.sh       逐项点名与按条复用（项的收集、test-plan 与记库）、verify（测试那一步）、--list-items、合进主工作区的命令
#   merge-and-verify-selftest-lib.sh    自证 selftest（入口在 --selftest 时调它）
# 弄坏开关表仍写在本文件下面「弄坏开关」那一段；读开关的函数在哪一份（按开关名搜得到；自证里设开关的写法不在这里列）：
#   merge-and-verify.sh：verify-in-main-tree、workspace-dependencies-ignored、patch-paths-ignored、keep-owner-unrecorded、merge-in-dropped-on-untargeted、merge-patch-single-directory、merge-patch-stat-cache、merge-patch-preimage-dropped、merge-in-after-skip-apply、main-tree-fingerprint-touched-files-only、fingerprint-dependencies-skipped、fingerprint-of-unchecked、target-under-work-root、copy-not-git-initialized、whole-package-tests、fallback-runs-whole-workspace、document-readers-unscanned、document-comments-counted、document-refused、path-attribute-ignored、subdirectory-whole-directory-added、subdirectory-fallback-dropped、subdirectory-fallback-note-dropped、registration-tables-refused、formal-tables-refused、formal-tables-by-suffix、refusal-packages-unnamed、manifest-sections-refused、manifest-other-sections-unchecked、manifest-default-path-ignored、manifest-shadowed-targets-unchecked、manifest-targets-compared-by-name、manifest-paths-unnormalized、directory-form-default-root-ignored、edition-2015-automatic-targets-kept、unknown-option-accepted、expand-up-to-unchecked、crash-domain-whole-table-item、crash-domain-subject-without-root、earlier-attempt-slot-not-preferred、slot-not-reused、slot-kept-taken、slot-limit-ignored、slot-lock-skipped、slot-surplus-taken、slot-surplus-left、slot-surplus-kept-removed、anchor-own-table-not-restored、anchor-created-files-kept、anchor-check-log-truncated、anchor-skip-replacement-not-reversed
#   merge-and-verify-snapshot-lib.sh：snapshot-keeps-times、keep-copy、cleanup-only-on-success、keep-owner-unrecorded、keep-run-target、cleanup-after-other-steps、finish-in-forked-shell、run-target-open-files-unchecked、earlier-attempt-not-recorded、earlier-attempt-snapshot-kept、earlier-attempt-task-ignored、earlier-attempt-not-smaller-removed、earlier-attempt-open-files-unchecked、slot-table-unchecked、slot-copy-keeps-old-times、slot-deleted-files-kept、slot-target-unbounded、slot-target-on-shared-root、slot-build-outputs-kept、slot-release-keeps-build-outputs、slot-fallback-unsummarized、peer-target-called-caller、slot-incremental-on、slot-incremental-cache-kept、slot-incremental-caller-overridden、slot-work-root-unchecked、slot-default-work-root-refused、slot-default-work-root-follows-data-root
#   merge-and-verify-steps-lib.sh：logs-single、cargo-wrapper-fixed、cargo-wrapper-unrecognized、clippy-combinations-unchecked、no-verification-package-guard、skip-apply-unchecked、reverse-check-each-alone、reverse-check-copy-preserves、way-out-without-current-tree-patch、verification-rerun-on-main-tree、rerun-confirmation-dropped、verification-rerun-items-dropped、verification-step-skipped、whole-verification-cells、crash-cases-whole-cell、feature-gated-unrouted、feature-gated-directory-target-unrouted、product-gpu-unrouted、verification-step-sample-exit-anywhere、rerun-command-wrapped、rerun-whole-cells、verification-step-red-ignored、verification-step-failed-lines-ignored、expand-up-to-not-passed、scaled-run-unlabelled、rerun-command-unscaled、judge-expand-variable-not-passed、card-handed-to-the-gate、start-card-stripped、crash-domain-unbounded、crash-domain-under-limit-narrowed、crash-domain-narrow-ignored、crash-domain-narrow-note-dropped、crash-domain-known-depth-replaced、crash-domain-unknown-passed、crash-domain-fallback-note-dropped、crash-domain-fallback-depth-wrong、crash-domain-given-value-overridden、crash-domain-program-failure-ignored、list-items-refusal-drops-items、progress-lines-dropped、unresolved-lines-dropped、anchor-check-skipped、anchor-check-baseline-ignored、anchor-check-release-row-number-kept、anchor-mutation-only-skipped、anchor-skip-apply-unchecked、anchor-skip-table-not-reversed、anchor-way-out-delete-dropped、anchor-way-out-keep-copy-dropped、anchor-skip-no-directories-silent、anchor-skip-none-records-red、anchor-skip-none-reason-dropped、anchor-skip-irreversible-passes
#   merge-and-verify-items-lib.sh：tests-one-binary-at-a-time、ignore-red、drop-discipline-lint、lint-table-optional、cargo-arguments-fixed、single-clippy-pass、clippy-combination-red-uncounted、clippy-registration-ignored、skip-build、build-whole-workspace、build-debuginfo-kept、mutation-tables-claimed-judged、zero-tests-green、skip-rebuild-check、rebuild-evidence-test-only、merge-rerun-assumes-applied、merge-in-commands-dropped、merge-in-without-kept-snapshot、mutation-names-keep-deleted、apply-refusal-way-out-unchanged、refused-patch-way-out-unchanged、segment-tree-command-narrow、main-tree-check-dropped、fingerprint-mutation-rows-dropped、rerun-keeps-no-snapshot、rerun-batch-unnamed、rocksdb-entry-unchecked、test-arguments-added、refusal-packages-unnamed、verification-release-dropped、verification-default-features-kept、slot-changed-sources-unchecked、items-ignored、item-unknown-accepted、reuse-skips-dropped、red-not-recorded、plan-failure-ignored、plan-unavailable-run-anyway、item-done-unreported、item-plan-reusable-zero、list-items-unpatched、unresolved-option-ignored、unresolved-without-option、unresolved-without-item、unresolved-summary-dropped、unresolved-option-dropped-from-rerun、merge-in-dry-run-dropped、merge-in-anchored-rows-before-dropped、merge-in-anchored-rows-after-dropped、merge-in-quick-checks-before-dry-run、merge-in-dry-run-way-out-dropped、merge-in-dry-run-outcomes-collapsed、merge-in-redo-way-out-narrow
#   merge-and-verify-selftest-lib.sh：dry-run-baseline-on-live-root、dry-run-outside-files-not-copied
#   lib_merge_verify_formal_tables.py：formal-items-dropped、formal-items-whole-table
# 一批实现员交回之后的「合入后验证」：开跑那一刻把要验的树拷成一份快照，在快照上打全部补丁、编一次、只跑补丁改到的测试目标与变异行点名的测试目标（下面「挑测试目标」）、
# fmt、clippy 与 build，再把跑到的验证包目标里登记的崩溃枚举用例与挂特性的测试文件逐项交门禁 crash-point-replay（下面「验证包那一步」），红的列出来。
# 合入后验证做 fmt、clippy、build、测试、验证包五步。build 那一步要回答的是补丁有没有把依赖改到的包的那些包编坏：单态化出错、链接出错只有真编译才报，
#   clippy 只做类型检查、挑出的测试只编挑出的目标，两样都拦不住（下面 build 一段）。
# 不跑整个包，也不起门禁的整格：判不出改到哪个测试目标的那几种测试那一步不跑、整条退 3（下面「挑测试目标」）。替掉派一个实现员做这件活。
#
#   merge-and-verify.sh [--skip-apply] [--crash-expand-up-to <n>] [--item <包>:<目标根文件>]… [--unresolved-to-commit-gate] [--fingerprint-of <日志目录>] <补丁目录> …
#     --fingerprint-of <日志目录>（不带 --skip-apply 的那一趟；只重跑红的那几项的出路自己带上）：开跑先照那一趟记的指纹核主工作区自那一趟开跑以来没变，变了退 2、一个补丁都不打（下面「开跑那一刻的指纹」）。
#     --unresolved-to-commit-gate（只在又点了 --item 的真跑那一趟生效；没点 --item 的不生效、判不出测试目标的照旧退 3，--list-items 也不看它）：判不出改到哪个测试目标的几处
#       不挡测试那一步与验证包那一步，逐条打成「留给提交时整轮门禁」，点名的项照跑（下面「判不出的留给提交时整轮门禁」）。
#   merge-and-verify.sh --list-items [--with-keys] [同一趟的别的参数…]   逐行列这一趟测试那一步可点的项，不编、不跑（下面「逐项点名与按条复用」）
#     起法：task-server/task submit -- bash scripts/gate/merge-and-verify.sh …（一整条一个任务；内存上限、线程、机器由服务定，命令里不写；
#       这一份与验证包那一步交的门禁都不领卡，卡只归 GPU 批处理验证服务，下面「显卡」）。
#     --item <项>（给几次取并集）在挑出的测试目标里只跑点名的那几个（下面「逐项点名与按条复用」）；fmt、clippy、build、重编核对照旧，验证包那一步只交点名的验证包目标里的。
#     经 task-server/task submit 交的要带 --item（不带的提交时被服务当没点名拒，task-server/task_server_lib.py 的 unnamed_run_problem）：项是 --list-items 列的第一列
#     （不带 --skip-apply 时在打上这一批补丁的临时快照上列，补丁新建的测试目标也列得出，下面「逐项点名与按条复用」）；点法照 .claude/skills/merge-verify/SKILL.md「写与验分两段」验证段第 1 步「点哪些项」。
#     测试那一步的并行度取 heavy_thread_default（scripts/hosts/heavy-threads.sh；经服务起时是服务套的 capped.sh 设的 SINGLEFS_THREAD_CAP）。
#     --crash-expand-up-to <n> 把崩溃放量、崩溃枚举用例与验证包里直接调判器的对拍的展开深度缩到不超过 n（变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 与
#     SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO 两个同设成 n，只往小缩，大于引擎常量的不放宽），
#     只管验证包目标的 cargo test 与验证包那一步（下面「验证包那一步」）；环境里已设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 的当作给了同一个值；不是非负整数退 2、一样都不做；
#     不给时验证包那一步先估崩溃域（下面「崩溃域上限」），不超上限就是整个域。
#   merge-and-verify.sh --selftest                     # 用假的 apply-writer-patch 与假 cargo 走一遍；MERGE_AND_VERIFY_BREAK=ignore-red 时必须判红
#     验证包那一步换上假的那一道（照真的那一道的写法答 --list-items、打按条复用与崩溃状态块那几行）；逐项与复用那几格经 admission.py 查库、记库（判定库在 prune-server 里）：只在临时服务底下跑，
#     裸跑开跑前调客户端 selftest_service_problem、退 2、一格都不跑。起法：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service
#     --server-binary <prune-server 二进制> -- bash scripts/gate/merge-and-verify.sh --selftest
#
# 快照（不拿主工作区的锁）：主工作区一个字节不写。快照放在哪、编译目录用哪一份分两种：占到槽的那一趟放在槽里、接着这一槽上一趟的编译目录编（下面「槽位」）；
#   没占到槽的那一趟一趟一份，照这一段与下面「这一趟的编译目录」。两种都在快照上打补丁、编、验，下面说「快照」两种都指。
#   一趟一份那一种：开跑那一刻把要验的树（主工作区；补丁已打在上面的带 --skip-apply）拷进 <日志目录>/repo（日志目录在工作根下，第二台同一路径读得到）：
#   rsync --recursive --links，不拷任意一层的 .git、target、target-* 与 __pycache__，带工作区里的全部源码与未提交改动；不保留属性（权限位照建文件时的缺省、
#   不带属主属组：有的临时目录不认保留属性），不保留修改时刻（拷过去的 mtime 是拷的那一刻：调用方给的编译目录编过也照新内容重编，
#   .claude/rules/test-script-discipline.md「编译与算法都要快」）。拷完在快照根 git init（不加文件、不提交）：读写判定库的脚本只认
#   <被判那棵树的 git common-dir>/<库名> 这个名字，库在 prune-server 里、键里不带哪棵仓（scripts/crash/lib_verdict_library.py 的 library_directory），
#   不是 git 工作树就退 2。拷不成、git init 不成退 1、一个补丁都不打。快照放进了要验的树里面（日志目录指进了树里）就退 2、什么都不拷。
#   之后打补丁（apply-writer-patch.py --root <快照>）、编、跑测试、验证包（起的是快照里那一份门禁、交给它的根是快照）、算复用键全对着快照：
#   主工作区随后怎么改都不影响这一趟的结论与写进判定库的键。编译目录照下面「这一趟的编译目录」（槽位那一种照「槽位」）。
#   一趟一份那一种收尾删掉快照（成功、失败、被 INT / TERM 停都删）；日志目录留着（日志小，汇总行指着它）。删完打一行「删了快照 …（共 …）」；删不成退 1、写明没删成。
#   收尾只在起这一趟的那个 shell 里做（finish 认 BASHPID 等于 $$）：run_step 放后台的进度进程是 fork 出来的子 shell，fork 之后、bash 把继承来的 trap 换回缺省之前那一刻
#   收到 stop_progress_ticker 的 TERM，子 shell 照主 shell 的 TERM trap 退出、跑继承来的 EXIT trap；在那里收尾就是这一趟还在往下跑时删掉它的快照与编译目录
#   （步骤的命令几毫秒就退时撞得上，自证的假命令就是这样）。子 shell 跑到 finish 时照原退出码退、什么都不删（槽位那一种也不放槽）。
#   要留现场：MERGE_AND_VERIFY_KEEP_COPY=1，一趟一份那一种收尾不删，打出快照与编译目录的路径和删法，看完自己删（command-safety.md「测试镜像一律放临时目录」：
#   要留着看现场由显式的开关打开、留下时打印路径）；槽位那一种收尾把槽记成留着、打出放槽那一条（下面「槽位」）。不带 --skip-apply 的那一趟（.claude/skills/merge-verify/SKILL.md「写与验分两段」验证段第 1 步）一律带它：
#   验证段第 2 步在这棵快照上跑（它是开跑那一刻的整个要验的树加打上的全部补丁，带门禁表与仓里的脚本、已 git init），修补的实现员副本从它建；
#   没带它的那一趟验绿了不打合进主工作区的命令（下面「先验后合」）。经服务交写成 task-server/task submit -- env MERGE_AND_VERIFY_KEEP_COPY=1 bash scripts/gate/merge-and-verify.sh …。
#   归属记录：一趟一份那一种留着快照的那一趟（带 MERGE_AND_VERIFY_KEEP_COPY=1，快照拷好、git init 之后）在日志目录写 <日志目录>/.owner，三行 session、created、purpose（槽位那一种写在槽里，下面「槽位」），
#   写法就是崩溃放量运行目录的那一份（scripts/hosts/run-build-directory.sh 的 run_directory_owner_record；会话号依次取 WORK_DIRECTORY_OWNER_SESSION、CLAUDE_CODE_SESSION_ID、
#   CLAUDE_SESSION_ID，都没有的不写）；日志目录没设时是 <工作根>/merge-and-verify/<日期>-<进程号>/.owner，快照在同一层的 repo/。不留快照的那一趟不写。
# 上一次尝试留下的快照：统一任务服务每次尝试交环境变量 TASK_SERVER_TASK（任务号）与 TASK_SERVER_ATTEMPT（尝试号，从 1 起）；撞内存上限是 cgroup 杀，EXIT trap 走不到，
#   快照留在那一趟的日志目录里，下一次尝试另起一个日志目录。开跑时（建好日志目录之后、拷快照之前）：
#   · 任务号有、尝试号是正整数的，在日志目录写 <日志目录>/task-attempt，一行「<任务号> <尝试号>」（record_task_attempt）；手跑没有任务号，不写；
#   · 任务号有、尝试号大于 1 的，在这一趟日志目录的同一层（默认是 <工作根>/merge-and-verify/）找 task-attempt 里任务号相同、尝试号更小的日志目录，删它的 repo/ 快照
#     （remove_earlier_attempt_snapshots）；带 MERGE_AND_VERIFY_KEEP_COPY=1 留下的也删（同一任务的重试会另留一棵新的）；别的任务号、尝试号不小于这一次的、没有 task-attempt 的日志目录不碰；
#   · 删之前核本机没有进程开着里面的文件（scripts/hosts/run_build_directory.py holders：退 3 是有，别的非 0 是核不成；核的只是本机的进程）：有或核不成就不删，打一行写明；
#     删不成也只打一行 !，不改这一趟的退出码；
#   删了打一行「删了上一次尝试留下的快照 <路径>（共 <大小>；…）」，日志目录与里面的日志、.owner 留着。编译目录建在 TMPDIR 下、不在日志目录里（下面「这一趟的编译目录」），不归这一步删。
#   上面删的只是一趟一份那一种留在日志目录里的 repo/。槽位那一种被 cgroup 杀的那一次尝试，占槽的锁随进程一起放，槽与里面的快照、这一台的编译目录都留着、不删：
#   同一任务的下一次尝试挑槽时先挑它记下的那一槽（尝试号更小、最大的那一次尝试的日志目录里的 slot，earlier_attempt_slot），空着就接着那一槽的编译目录编（下面「槽位」）。
# 这一趟的编译目录（一趟一份那一种：自己用完自己删，不留给以后的趟复用）：调用方没设 CARGO_TARGET_DIR、又没占到槽时
#   在这一台自己的 ${TMPDIR:-/tmp} 下新建 merge-and-verify-target.<随机>、export 成 CARGO_TARGET_DIR，四条 cargo 都用它；不建在日志目录下：
#   日志目录在工作根下，工作根两台共享（第二台经 NFS 写回本机的盘），编译目录有上百 G（records/2026-10-31-task-server巡检.md 问题 72 之二）。日志照旧在工作根下。
#   正常结束、红了退出、被 INT / TERM 停都在 EXIT trap 里删掉，而且是 trap 的第一步、在删快照之前（那几步写满盘、出错都挡不住它；删不成退 1，照接着删快照）
#   （scripts/hosts/run_build_directory.py remove：删之前核本机没有进程开着里面的文件，有就是这一趟的子进程没收干净，不删、逐个报进程、退 1）。
#   建不成退 2。调用方设了 CARGO_TARGET_DIR 就不占槽、照它编、收尾不删（不是这里建的，归调用方删；为什么不占槽见下面「槽位」）。
# 槽位（快照放在固定路径的槽里、一槽一个编译目录接着用：.claude/rules/test-script-discipline.md「编译与算法都要快」只管不同的树不共用编译目录，
#   固定路径、每趟按内容拷进来的一槽算一棵树）：
#   只在本机占槽：第二台那一层（scripts/hosts/multi-host-run.sh 文件头 ⑤）一律给 CARGO_TARGET_DIR，派到第二台的那一趟照下面「没占到槽的」不占槽、一趟一份、从头编。
#   槽：<工作根>/merge-and-verify/slot.<随机>/（与默认的日志目录同一层；工作根两台共享，留着的快照两台都读得到）。里面 repo/ 是快照，lock 是占槽的锁，used 记最近一次占它的时刻，
#     kept 在、repo/ 也在的是留着的槽（下面「留着的槽」）。工作根照 .claude/hooks/lib_work_root.py 取；槽那一层在要验的树里面就退 2、什么都不拷。
#   槽位登记 scripts/gate/merge-and-verify-slots.conf（槽数上限与槽的编译目录占盘上限只从它读）：slots=<至多几槽>、slot-target-limit-gib=<一槽的编译目录至多占多少 GiB>，
#     两项都要写、都是非负整数，别的行只许 # 起头的注释与空行；读不出、缺一项、值不是非负整数、有认不出的行都退 1、一个补丁都不打（参数解析之后、建日志目录之前读，read_slot_table）。
#     MERGE_AND_VERIFY_SLOT_TABLE 换掉这份登记，只给自证（settle_break_switches 拒平常设了的）；它与 MERGE_AND_VERIFY_SLOT_BUILD_ROOT、SINGLEFS_WORK_ROOT 三样要一起给，
#     只给了一部分的不读工作根与多机配置、不占槽也不清（choose_slot：缺一样，清槽与清编译目录就会拿自证的那一层对着真的那一层删）。
#     没给 MERGE_AND_VERIFY_SLOT_BUILD_ROOT 时（编译根是这一台真的那一个），工作根还要是这一台缺省的那一个：多机配置里 SHARED_DATA_ROOT 下的 work，与编译根读同一份配置
#     （SLOT_CONFIGURED_ROOTS_PROGRAM 的 default-work-root，照 .claude/hooks/lib_work_root.py 的 configured_data_root 核），不看 SINGLEFS_DATA_ROOT；两边都按真路径比。
#     只给了另一个 SINGLEFS_WORK_ROOT（两样自证的都不给也一样）、连同 SINGLEFS_DATA_ROOT 一起换了工作根、换了 SINGLEFS_DATA_ROOT 而不设 SINGLEFS_WORK_ROOT 的都不占也不清，
#     打一行「这一趟不占槽：工作根 <它> 不是这一台缺省的那一个（<缺省的>），不占也不清…」，照一趟一份跑（choose_slot：换了工作根，清编译目录就拿那个工作根的槽那一层
#     对着真的编译根比，缺省工作根下别的趟占着的、留着的各槽的编译目录都会当成槽已不在删掉）；只换了 SINGLEFS_DATA_ROOT、工作根仍是缺省的那一个的照占。
#     取不到缺省的工作根（配置读不了、没写 SHARED_DATA_ROOT、写的不是绝对路径或不在 NFS_SHARED_ROOTS 下）同样不占也不清，那一行写出配置那一侧说的原因。
#     统一任务服务交的 SINGLEFS_WORK_ROOT 是服务那一侧 work_root() 的真路径（task-server/singlefs-task-server.service.in 不设 SINGLEFS_WORK_ROOT 与 SINGLEFS_DATA_ROOT，
#     就是配置里 SHARED_DATA_ROOT 下的 work），照占。
#   槽的编译目录：在本机本地，多机配置 BUILD_ROOT_BY_HOST 里这一台那一项（scripts/hosts/multi_host_addresses.py 认这一台、按台取值，配置的位置照
#     scripts/hosts/lib_repository_copy.py 的 configuration_path；SLOT_CONFIGURED_ROOTS_PROGRAM 的 build-root）下的 merge-and-verify-slots/<槽名>/target，不放共享根；
#     接着本机上一次在这一槽编的。MERGE_AND_VERIFY_SLOT_BUILD_ROOT 换掉这一台的编译根，只给自证（settle_break_switches 拒平常设了的）。
#   占哪一槽（take_slot 起 SLOT_HOLDER_PROGRAM：一个 python 进程挑槽、占着锁直到这一趟收尾）：几趟同时挑时排他（槽那一层的 slots.lock，挑完、清完多出来的槽就放），
#     只在按 used 从新到旧排的前 slots 槽里挑，挑法依次是：
#     同一任务更早一次尝试记下的那一槽（在前 slots 槽里才算；上面「上一次尝试留下的快照」）、空着的槽里最近占过的那一槽（used 最新的）、槽数没到登记的上限就新开一槽。
#     空着 = 锁拿得到而且不是留着的槽。锁是 POSIX 记录锁（fcntl.lockf）：本机加的锁与第二台经 NFS 加的锁落在服务端同一种锁上、互斥；flock 在服务端那一台本地加的与
#     NFS 客户端加的不互斥，不用它。占到之后删掉槽里上一趟留下的 kept、.owner、.deliverables（这一槽此刻没留着），写 used，在日志目录写 slot（这一槽的路径），
#     打「占了槽 <槽>（<怎么挑的>）」。锁跟着那个 python 进程：收尾 kill 它、等它退（release_slot）；这一趟被 cgroup 杀时它一起没；它另外每半秒核一次起它的 shell 还在不在，不在就退。
#   多出来的槽：按 used 排在前 slots 槽之后的那几槽（调低了 slots、slots 是 0 时已有的槽）不挑；挑槽的进程拿着 slots.lock 逐个清：锁拿得到、不是留着的槽整个删掉，
#     打「删了多出来的槽 <槽>」；别的趟占着的、留着的跳过，删不成的也不重试，各打一行 !，下一趟再清。slots 是 0 的那一趟不占槽，槽那一层里有槽就照样起挑槽的进程，
#     只清这一种、不新开不占（trim）。清完照「占盘」删这一台编译根下槽已不在的那几份编译目录。挑槽的进程答的这几行逐行照打，每一行至多等 SLOT_HOLDER_SECONDS 秒。
#   没占到槽的打一行「这一趟不占槽：<为什么>」，照一趟一份那一种跑，不等：第二台那一层给了编译目录（CARGO_TARGET_DIR 落在它给的 MULTI_HOST_RUN_PEER_SCRATCH 下面）、
#     调用方给了 CARGO_TARGET_DIR、登记的 slots 是 0、取不到工作根或这一台的编译根、槽都占着或留着又到了上限、挑槽出错。
#   调用方给了 CARGO_TARGET_DIR 的不占槽（第二台那一层给的同样）：给的那一份不是这一槽的，按内容拷、留着旧修改时刻的快照对着别处编过的产物，cargo 会把内容变了的文件判成已编。
#     统一任务服务按「--cwd 那一棵树 × 台」给编译目录（task-server/task_server_lib.py 文件头「编译目录」：组里那几个空着的轮流给）：合入后验证交的 --cwd 是主工作区、编的是槽，
#     接服务给的那一份就是几槽连同主工作区共用编译目录、互相盖掉增量。所以这份脚本文件头不写会编的声明「# task-server-compiles: yes」，服务在本机不给它编译目录；
#     第二台那一层（multi-host-run.sh）不看这份声明，派到第二台的一律给。编译目录只用这一槽自己那一份。
#   拷进槽（refresh_slot_snapshot）：先删掉槽里快照下任意一层的 target/、target-*/、__pycache__/（.git 照留；rsync 排除了它们，下面的 --delete 删不到：
#     验证段第 2 步在留着的槽里不带 CARGO_TARGET_DIR 起的 cargo 照默认编进 <快照>/target，留在共享根上），逐个打要删的，删完回读一个不剩，剩了退 1；
#     再 rsync --recursive --links --checksum --no-times，与 scripts/hosts/lib_repository_copy.py 的 copy_entries 同一种按内容比的拷法：
#     内容没变的不动、留着上一趟的修改时刻，内容变了与新加的整份重写、修改时刻是此刻，cargo 照修改时刻判，改过的必重编；另带 --delete（源树里没有的在槽里删掉，
#     槽里不留上一趟打的补丁与别的残文件；排除的 .git 不删，槽根的 .git 是这里 git init 的）与 --executability（内容没变而可执行位变了的照改）。
#     rsync 退 24 同一趟一份那一种；拷不成退 1、一个补丁都不打（槽里拷了一半的，下一趟照内容补齐）。拷完照「快照」那一段 git init（已是 git 工作树的照常 init，不加文件、不提交）。
#     --itemize-changes 逐个列出写了、删了的条目（<日志目录>/slot-changed-sources.itemized），其中的 .rs 写进 <日志目录>/slot-changed-sources。
#   重编核对（下面「改过的包都重编了没有」）在槽位那一种另读 slot-changed-sources：内容变了、新加、删掉的 .rs 都核；打补丁的那一趟另照补丁目录核；
#     带 --skip-apply 的那一趟不读补丁目录（补丁早在主工作区上，槽里没变的那几份这一槽上一趟已编过，这一趟本来就不重编）。
#   收尾（finish 的槽位那一支，同样只在起这一趟的那个 shell 里做）：不删快照与编译目录；照「占盘」量这一槽的编译目录；带 MERGE_AND_VERIFY_KEEP_COPY=1 的写 kept
#     （两行 log、created）与归属记录 <槽>/.owner（run_directory_owner_record，三行 session、created、purpose），槽就留着，打放槽那一条；没带的打「放了槽 <槽>」；最后放锁。
#   留着的槽：别的趟不占它，直到放槽——rm -rf -- <槽>/kept <槽>/.owner <槽>/.deliverables <槽>/repo/target（slot_release_command），快照与这一台的编译目录留给下一趟接着编；
#     repo/ 被删了的槽不算留着（下一趟占它就整份拷、从头编）。留着没有期限，只由收尾的槽位汇总逐个报、不收。
#     收尾那一行打出这一条，验绿之后合进主工作区那一段的末尾、槽位汇总也写它，三处是同一条（下面「先验后合」）。
#   槽位汇总（report_slot_state，收尾第一步，同样只在起这一趟的那个 shell 里打）：没占到槽的打「槽位汇总：这一趟不占槽（<为什么>）…」；取到工作根的，
#     槽那一层里有留着的槽就逐个打槽名、留了多久（从 kept 里的 created 那一天算，读不出取 kept 的修改时刻）、.owner 里的会话与放槽那一条。只报，不收、不改退出码。
#   占盘：一槽的编译目录在占到槽之后、编之前与收尾各量一次（du），超过 slot-target-limit-gib 就整个删掉（scripts/hosts/run_build_directory.py remove：本机有进程开着里面的文件就不删、
#     打一行），这一槽下一次从头编；挑过槽之后（占到、没占到、slots 是 0 都一样）另删这一台编译根下 merge-and-verify-slots/ 里槽已不在的那几份（槽目录被删了的）。
#     本机至多 slots × slot-target-limit-gib（只在上面那几刻量，一趟编的过程中可以超过）。
#   增量编译（slot_incremental_off，占到槽、编译目录建好之后、编之前）：槽的编译目录一趟接一趟地用，增量缓存（<档>/incremental）跟着一趟趟累积、占一槽编译目录的一大块
#     （量的数在 records/2026-11-18-合入后验证压盘.md），所以槽位那一种关掉增量：export CARGO_INCREMENTAL=0，这一趟起的每条 cargo（fmt、clippy、build、test 与验证包那一步）都继承它；
#     再删这一槽编译目录下已有的 <档>/incremental 与 <目标三元组>/<档>/incremental，逐个打「删了这一槽编译目录里的增量编译缓存 <目录>（<大小>）」
#     （run_build_directory.py remove：本机有进程开着里面的文件就不删、打一行 !，照往下编）。一趟一份那一种（没占到槽，编译目录收尾整份删）不设。
#     调用方设了 CARGO_INCREMENTAL（非空）的照它、不盖，这一槽的 incremental 也不删，打一行「增量编译：调用方设了 CARGO_INCREMENTAL=<值>…」（与调用方给了 CARGO_TARGET_DIR 同理：调用方给的照它）。
#   槽位那一种的弄坏开关在开关表「槽位」那一族。
#
# 为什么：「合入后验证」做的前三步（打补丁、编、逐个跑测试）
# 是 apply-writer-patch.py 加几条命令，只有「红了改钉值」要判断——那一步红了才派实现员。
# 做法：每个补丁目录经 scripts/gate/apply-writer-patch.py --root <快照> 打（退 2 就整批停）。调用方给的编译目录（CARGO_TARGET_DIR）可能是编过的，
# cargo 按 mtime 判本地包新不新鲜，带着旧 mtime 的新内容会被当成已编：
# 一趟一份那一种的快照不保留修改时刻（文件头「快照」），里面每个文件都比编译目录里的产物新，不另刷源码戳；
# 槽位那一种按内容拷（文件头「槽位」），内容变了的修改时刻是此刻、比这一槽编译目录里的产物新，没变的留着上一趟的、照指纹不重编。
# 然后在本机直接跑 test、fmt、clippy、build 四步 cargo（clippy 按特性组合各一条；不套内存包装与 capped.sh，这一整条的上限与线程由起它的服务定），与上游 .claude/singlefs-ai-sop/scripts/check.sh 同一套判据（test 与 build 的范围另照下面两段收窄）：
#   test   只跑挑出的测试目标（下面「挑测试目标」），一个目标一项，一个包一条 `cargo test --offline --no-fail-fast -p <包> <点到的 --lib / --test 名 / --bin 名…>`，几个包的几条依次跑
#          （日志 test-<序号>.log，接成一份 test.log 与 test.problems.log）；不带上依赖它们的包，也不跑包里没点到的目标。check.sh 跑的是 `cargo test --all`，这里只跑其中点到的目标。
#          验证包 singlefs-verification 的目标带 --release 跑（与门禁 crash-point-replay 的 crash-case-replay 格同一档、同一种类的判定，下面「逐项点名与按条复用」），
#          登记成崩溃枚举用例与放量分项的用例函数经 --skip 排掉（崩溃枚举用例交验证包那一步）；验证包的计划算不出时这一包不起（登记的用例排不掉），按条复用失灵照判红。
#          工作区成员（树根 Cargo.toml 的 [workspace] members，逐个读成员 Cargo.toml 的包名）里没有 singlefs-verification（改了名、挪了）、成员写成通配、读不出包名，都判红、不编：
#          认不出验证包就排不掉登记的崩溃枚举用例。
#          退 0 而 test result 行里一条测试都没数到、按条复用也一条都没复用，判红（check.sh「一条测试都没跑」那一条）。挑出的目标一个都没有（补丁只改了要特性的目标、只删了测试目标、
#          只往变异表里删行）时测试这一步不跑、报「测试本次未跑」，不记红也不算测过。
#          每一条交给 scripts/gate/test-targets-parallel.py --jobs <线程上限>（用例少的二进制跑的时候核就空着：第二台上只用到约 5 核、登记 31，
#          records/2026-10-18-里程碑三收尾与近期欠账.md 第 780 行）：先 --no-run 一次编好，再按测试目标各起一条 cargo test -p <包> <挑法>、同时至多线程上限那么多条，
#          每条的 RUST_TEST_THREADS 分摊，各目标的输出按派活的次序回读，派出与收回的条数对不上判红。跑的用例与 cargo test -p … <挑法> 同一批，
#          FAILED 行、test result 行与 Compiling 行照旧落进 test.log，下面的判法不变。
#          MERGE_AND_VERIFY_TEST_TARGETS=serial（只给自证）退回一条 cargo test --offline --no-fail-fast -p …；自证的别的格都这么设，只有「按测试目标并行起」那一格不设。
# 挑测试目标（测试那一步与验证包那一步都照它；判法在 MERGE_AND_VERIFY_ITEMS_PROGRAM 的 targets 一段）：读给的每个补丁目录里 crates.patch 改到的路径（新旧两侧，
#   scripts/gate/lib_rebuilt_packages.py 的 collect_paths）与 mutations-append.tsv、mutations-replacements.tsv 的行；--skip-apply 时照样读给的补丁目录，不打。
#   路径往上找到的第一份带 [package] 的 Cargo.toml 是它的包（lib_rebuilt_packages.py 的 package_of），包里的相对路径照这几条落到测试目标
#   （目标清单是 scripts/gate/rerun-failed-tests.py 的 test_targets_of_package：cargo test 不带特性会跑的那几个；推目标的办法——段里写的 path 先归一、
#   没写 path 的缺省根文件含目录形、edition 2015 的自动认的目标——只在它那里写一份（它文件头「项的写法」），这里经 importlib 调，下面两侧目标集合也用它的 section_targets）：
#     正是一个测试目标的根文件（tests/<目标>.rs、tests/<目标>/main.rs、src/bin/<名>.rs、src/main.rs、[[test]] / [[bin]] 写的 path）→ 那个目标；
#     tests/<子目录>/ 下的（不是目标根文件）→ 只落到下面那条「mod / #[path] 走得到它的目标」：从每个目标的根文件起照 mod 与 #[path] 跟到这一份文件为止，
#       经子目录 mod.rs 走的只认 mod.rs 里写了 `mod <名>;` 的（子目录里有这份文件、mod.rs 没 mod 它的不算）；
#       一个目标都走不到它（含不是 .rs 的数据文件）才退回整目录：tests/<子目录>/main.rs 那个目标，没有它就是 tests/ 顶层里写了 mod <子目录>（或 #[path = "<子目录>/…"]）的那几个目标，
#       退回时打一行「· …退回整目录：…共 N 个（…）」点名落到了哪几个；整目录也落不到的照下面判不出办；
#     src/bin/<名>/ 下的 → src/bin/<名>/main.rs 那个目标；
#     src/ 下别的（不论是不是 .rs）→ 库目标（[lib] 的 path，缺省 src/lib.rs）；包里没有库目标的落到 src/main.rs 那个目标；
#     写了 required-features 的 [[test]] / [[bin]]（cargo test 不带特性不编）记「要特性」：产品 GPU 包的交验证包那一步（真卡对拍那一格），别的包报本次未跑；
#     以上之外（tests/<子目录>/ 下的只照这一条，见上）再加上 mod / #[path] 走得到它的目标：从每个测试目标（含要特性的）的根文件起沿 `mod <名>;` 与 `#[path = "…"] mod <名>;` 往下跟（#[path] 的值相对写这一行的文件所在目录，
#       引进来的文件里再有 mod 或 #[path] 照样往下跟；tests/ 下与 src/ 下一样认），补丁改到哪一份被引的文件，就落到引它的每个目标
#       （两个 bin 经 #[path] 共用一个模块，只改这个模块时两个 bin 都挑）；走得到它的是要特性的目标，记「要特性」同上。写在内联模块 mod x { … } 里的 mod 声明与 cfg_attr 里写的 path 不认。
#     测试目标根文件那一类的路径在树里已经没有（补丁删了这个目标）：报一句、不跑。
#     包的 Cargo.toml（包目录下的，不是工作区根的）：改动只落在 [[test]] / [[bin]] 段里的，新加、改了、删掉的每个段落到它点名的目标：段里写了 path 的照写（先归一：
#       相对包目录解析、去掉开头与中间的 ./、合并多余的分隔符，path = "./tests/case_a.rs" 就是 tests/case_a.rs，与 cargo 认的一样），没写的照 cargo 的缺省路径
#       （[[test]] 是 tests/<名>.rs、tests/<名>/main.rs；[[bin]] 是 src/main.rs（名字与包名相同）、src/bin/<名>.rs、src/bin/<名>/main.rs，树里在哪个认哪个）；改了的段新旧两侧的根文件都落；
#       写了 required-features 的段记「要特性」同上；段删掉而根文件不再是测试目标（或段写了 test = false）的报一句、不跑；落不到又不是这两种的整份判不出。
#       新旧两侧比：旧侧是在 ${TMPDIR:-/tmp} 下的临时 git 仓里放一份树里此刻的 Cargo.toml、照给的次序倒着逐份 git apply --reverse（--include 只认点到这份文件的）倒出来的，两侧读成表之后比：
#       [[test]] / [[bin]] 之外的键（[package]、[dependencies]、[features]、[lib]、[[example]] 这类）有任何差别，两侧段的 name 缺了、不是字符串或重名，旧侧倒不掉或读不了，
#       没有一个 [[test]] / [[bin]] 段有差别（只动了注释、空行或写法），都整份照下面判不出办，原因写是哪一种。
#       自动认的目标被顶掉：新旧两侧各算一遍包的 [[test]] / [[bin]] 目标集合（目标名到根文件；两侧都照树里此刻的文件推自动认的目标，只差 Cargo.toml；
#       写了的段照写（归一过），没写 path 的照上面的缺省路径；推自动认的目标时，tests/<名>.rs、tests/<名>/main.rs、src/main.rs、src/bin/<名>.rs、
#       src/bin/<名>/main.rs 里名字与根文件都没被段点到的是自动认的，与 cargo 认的一样；推不推照 cargo：[package] 的 autotests / autobins 写了 true、false 的照写，
#       没写的看 edition——包没写 edition（cargo 按 2015 算）或写了 2015、而这一侧写了同一种段的，不再推自动认的同一种目标（新加一段 [[test]] 就把别的自动认的测试目标都顶掉，
#       原因里另写是 edition 2015 那一条；edition 写成 { workspace = true } 的不往上读工作区根，同样照 2015 算，只会多判不出）；required-features 与 test = false 不影响这个集合），
#       两侧集合按根文件比：旧侧某个自动认的目标的根文件在新侧不再是同一种段（[[test]] 或 [[bin]]）里任何目标的根文件（新加一段 [[test]] 与它同名而 path 指到别处，
#       自动认的那份文件就没人认了），cargo test 从此不跑那份文件、一句不报：整份照下面判不出办，原因点名被顶掉的目标与它的根文件。
#       新加的段 path 指到自动认的那份文件而名字不同（例 [[bin]] name = "unit-graph" path = "src/bin/unit_graph.rs"）：根文件还在新侧的目标里，不算顶掉，照上面落到新段。
#       [[bin]] 段有差别、或补丁只改 bin 的根文件，都只挑那个 bin（--bin <名>），不另挑同包的集成测试（只按改动测，.claude/main-agent.md「写与验分两段」验证段第 1 步）。
#     不变量文档（包里 invariants/ 下的 .md）：先查工作区各包的 .rs 读没读它。去掉注释之后字符串字面量里写着 invariants/、整段是 invariants 这个路径段、或写着这份文档文件名的算读它
#       （scripts/gate/admission.py 的 rust_code_without_comments；注释里提到的不算；只认写在字面量里的路径，遍历整个包目录、按通配读的认不出）。
#       没有读它的：报一句「不影响测试目标」、不跑、不退 3；有读它的：落到读它的那份文件所在的测试目标（照上面几条落）；读它的文件落不到测试目标（build.rs 这类）：照下面判不出办。
#     登记表（REGISTRATION_TABLES：.claude/gate.d/stage-inputs.tsv、.claude/gate.d/crash-point-replay-items.tsv、.claude/gate.d/refused-tests.tsv 与 scripts/crash/crash-amplification.sh；
#       实现员把新崩溃枚举用例与放量分项的登记行写进 crates.patch 带过来，.claude/agents/implementation-writer.md「产出」；拒算测试的登记行由主 agent 判过、
#       做成一份只含 refused-tests.tsv 的补丁目录并进这一批，.claude/main-agent.md「合入后验证之后判新出现的拒算」）：不落测试目标、不退 3，报一句；
#       快照里照打上之后的登记排掉登记的用例、交验证包那一步。
#     必红世界表（.claude/gate.d/verus-pins.tsv 的 must-red-worlds 行点名的表，一个 crate 一张）：门禁 formal-verification 的输入，不归 cargo 的测试目标；
#       登记照被判的那棵树打上补丁之后的 verus-pins.tsv 现读（lib_merge_verify_formal_tables.py 的 registered_tables），不写死表的路径。补丁改到它们：不落测试目标、不退 3，报一句
#       「补丁改到必红世界表 <路径>…」；改到的每张表里这一批补丁新加或改过的行（打补丁之前开跑时存下的那一份表与打上之后的逐行比：变异名新出现的，或同名而别的列变了的；删掉的行不点；
#       打补丁之前的那一份在 <日志目录>/merge-baseline 里，经环境变量 MERGE_AND_VERIFY_BASELINE_ROOT 交给 lib_merge_verify_formal_tables.py 的 changed_items；变异名是第一段，项名 must-red:<变异名>，
#       同 scripts/gate/verus-proof-check.py 的 must_red_item）另记下来，验证段第 2 步在快照上多打一条门禁 formal-verification 的整条命令点这几项（--item 给几次，带范围声明，写法照 .claude/agents/implementation-writer.md 第 1d 步形式化那一条）；
#       一张表都没改到、或改到的表没有新加或改过的行（补丁只删了行）的不打。判测试目标退 3 的那一趟照样打（它与测试目标无关）。别的不在登记里的 .tsv 落在包里的，照旧判不出、退 3。
#   变异行第五段（cargo test 的参数）里 -p <包> 与 --lib、--test <名>、--bin <名>、--example <名>、--bench <名>、--doc 点名的目标 → 那个目标
#     （`--` 之后的筛选词只报一句：跑的是整个目标，键相同的测试函数照复用）。
#   判不出的这几种：测试那一步与验证包那一步都不跑、整条退 3（fmt、clippy、重编核对照跑，build 照下面 build 一段挑包，有红照退 1），逐条列原因，
#     出路写「整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户」：
#     没给补丁目录；改到工作区根的 Cargo.toml、Cargo.lock 或 .cargo/；改到不落在任何包里的路径（登记表除外）；落在包里而照上面几条落不到测试目标的
#     （包的 Cargo.toml 改到 [[test]] / [[bin]] 段之外的或顶掉了自动认的目标的、build.rs、tests/ 顶层的数据文件、没登记的 .tsv 这类；verus-pins.tsv 登记的必红世界表除外）；变异行只点了包、点了一类目标（--tests、--bins、--examples、--benches、--all-targets、--workspace、--all）、
#     没点包，或点的目标在包里找不到。
#     用户同意整包跑的（.claude/skills/merge-verify/SKILL.md「写与验分两段」验证段第 1 步「退 3」那一支），包取上面逐条点名的那几个（「<包> 包里的…」「读它的 <包> 的…」「只点了包 <包>」）；
#     没点名包的几种（没给补丁目录、工作区根的文件、不落在任何包里的路径、变异行没点包或点了一类目标）脚本点不出包：工作区根的文件、不落在任何包里的路径那两条的原因里写明没有包可点、整包跑的范围由用户来选，出路里另有一句把这几种都点到，弹窗时一并问。
#   判不出的留给提交时整轮门禁（--unresolved-to-commit-gate，用户同意只跑落得到的项、整包留给提交时的整轮门禁才带）：带了它、又点了 --item 的真跑那一趟，上面判不出的每一处不再挡测试那一步
#     与验证包那一步：逐条打成一行「留给提交时整轮门禁：<原因>」（原因点名文件）；挑出的项照留，--item 只能点落得到的项（点了判不出的退 2，同别的写错的项），点名的项照常跑测试那一步与验证包那一步，
#     fmt、clippy、build、重编核对照旧。没有红时汇总写「有 N 处留给提交时整轮门禁」并在汇总行之后把那几处再逐条打一遍，验证段第 2 步与合进主工作区的命令照常打（下面「先验后合」），退 0；
#     没点 --item 的、没带这个选项的照旧不跑测试那一步、退 3，带了而没点 --item 的在出路里补一句不生效。只重跑红的命令（merge_rerun_command）点了项就带上这个选项。
#     弄坏开关 unresolved-option-ignored、unresolved-without-option、unresolved-without-item、unresolved-lines-dropped、unresolved-summary-dropped、unresolved-option-dropped-from-rerun 各弄坏一处（文件头开关表）。
#   读不了补丁、成员清单或目标清单退 1。挑出的项、各自为什么挑上（补丁改到哪个文件、哪一条变异行点名）逐项打一行。
#   fmt    此刻是 `cargo fmt --all -- --check`。
#   build  只编补丁改到的包与工作区里照 path 依赖传递依赖它们的包：参数照 check.sh 那一行现读（此刻是 `cargo build --offline --all-targets`，下面「cargo 命令现读」），
#          拼命令时在读来的参数之后逐个加 `-p <包>`（按包名排序）。包集合取「开跑那一刻的指纹」起点的同一套算法（MAIN_TREE_FINGERPRINT_PROGRAM 的 build-packages 一段，
#          与指纹共用 touched_packages、dependents_of）：在打上补丁的快照上，从给的补丁目录里 crates.patch 改到的路径（新旧两侧；--skip-apply 时照样读给的补丁目录）
#          往上找到的包起，取改到的包连同传递依赖它们的包那一半，不含它们只被依赖的那一侧，只留工作区成员。
#          依赖写成 <名> = { workspace = true } 的，照树根 Cargo.toml [workspace.dependencies] 里同名那一项的 path（相对树根）认（path_dependencies，指纹那一路同一份）；
#          不是 path 的依赖（registry、git，写成版本串的也算；workspace = true 而树根那一项不是 path 的也算），包名（项里 package 改的名字，没写是键名）在树根 Cargo.toml
#          某个 [patch.<源>] 里写了 path 的，照那一项的 path（相对树根）认（patch_paths，指纹那一路同一份）。
#          为什么：这一步要回答的是这一批补丁有没有把谁编坏，编坏的只会是改到的包与依赖它们的包；改到被依赖的包时，依赖它的包里的单态化错与链接错
#          只有真编译才报（clippy 只做类型检查、测试那一步只编挑出的目标）。调试构建把工作区全部目标链接一遍要写两百多 GiB
#          （records/2026-11-13-全量提交门禁前收尾调度.md 第 1503 行量的那一趟：595 个可执行、合计 233.4 GiB），所以只编这几个包。
#          这一条 cargo 的环境里设 CARGO_PROFILE_DEV_DEBUG=0（只设在 build 这一条上，test、fmt、clippy 不设）：编出的可执行后面哪一步都不用，调试信息只占盘，
#          占每个测试二进制的 73%–94%（records/2026-11-18-合入后验证压盘.md 课题那一行量的：抽 5 个调试可执行读节表 `.debug_*`）；关掉它照样报单态化错与链接错。
#          一个包都没有（没给补丁目录、补丁只改不落在包里的文件、只改变异表）时 build 这一步不跑、报「build 本次未跑」，不记红也不算编过。
#          成功句与红句写出这一趟编了哪几个包；算不出包集合（build-packages 那一段退非 0）记一项红、build 不跑。
#   clippy 照 check.sh 按特性组合逐个跑：每个组合一条 `cargo clippy --offline --all-targets <特性参数> -- -D warnings` 加编码纪律那张 lint 表
#          （组合 `default` 不带特性参数，别的照组合里的词带上），日志 <日志目录>/clippy-<组合>.log（组合去掉开头的 -、别的非字母数字换成 -）；
#          任一组合红整步红、红句点名是哪几个组合，过的那句列出跑了的组合。
#   lint 表与特性组合都只在 check.sh 一处定义：每次开跑从它的 `CODE_DISCIPLINE_LINTS=(` … `)` 那一段现读 lint 表（一条一对 `-D clippy::<名>`），
#   从行首那一行 `CLIPPY_FEATURE_COMBINATIONS=("…" "…")` 现读缺省的特性组合（每个组合是 `default` 或以 - 起头的特性参数），不在这里另抄一份；
#   缺省组合读出来不到两个判红（check.sh 退回了单趟 clippy，是上游改了判据）。
#   同时核 check.sh 里 clippy 按组合循环的三行结构（CHECK_SH_STRUCTURE_LINES）还是这里照着对齐的那一种。
#   cargo 命令现读：fmt、clippy、build、test 四条 cargo 命令的写法不在这里抄表，每次开跑从 check.sh 现读（read_cargo_command）：命令位置上写 cargo <子命令>
#   或 run_cargo <子命令>（前面可带 if）的行，每个子命令恰好一条；子命令之后到第一个 |、;、\ 之前的词（2>&1 不算）就是它的参数，这里的四步照这些词拼
#   （cargo 子命令后接 --offline；test 步另按包、按测试目标挑，所以那一行只认 --all 或 --workspace 加不带值的 --标志，带 --lib、--doc 这类收窄目标的标志判红；
#   clippy 那一行里 ${clippy_feature_arguments[@]+…} 换成这个组合的特性参数、"${CODE_DISCIPLINE_LINTS[@]}" 换成 lint 表，各恰好一次；别的带 $、引号、括号的词一律不认）。
#   所以上游只换包装（run_cargo 与 cargo 来回换）、加删标志（--locked 这类）时这里不用改；找不到那一条、写了不止一条、有不认的词才判红，
#   红句点名是哪一条命令、第几行、哪个词，出路写往 read_cargo_command 或 read_check_sh 里补哪一种识别，不是笼统的「写法变了」。
#   不退回「读不出就用写死的」：那样合入后验证跑的就不一定是 check.sh 跑的了（用户要的是同一套判据）。
#   check.sh 认项目根的特性组合登记（上游 lib.sh 的 CLIPPY_FEATURE_COMBINATIONS_FILE，登记了就只跑登记的）：这里不读登记，要验的树里有这份登记就判红、不编，
#   出路是先把登记的读法接进这里。读不出表或组合、形状不对、结构行或哪条 cargo 命令读不出，都判红、一个补丁都不打：上游改了判据，先回来对齐这里。
# 进度行（统一任务服务按「日志满 10 分钟没涨」判卡住，.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」要求跑得久的单项每 5 分钟至少打一行进度）：
#   run_step 起的每一步（测试每一条、fmt、clippy 每个组合、build、验证包那一步起的门禁每一次）跑着的时候，另起一个只往标准输出打进度的后台进程，每 PROGRESS_INTERVAL（300）秒打一行
#   「  · 进度：<日志名> 已跑 N 秒，日志 M 行」（日志名是这一步的日志去掉 .log，例 test-1、clippy-default、verification-crash-cases，与前面「测试第 N 条」「验证包那一步（…）：起 …」那几行对得上；
#   M 是全量日志此刻的行数）。步骤的命令一退，先取它的退出码，再按那个后台进程自己的 pid 停它、等它收尾（它的退出码不进这一步的退出码）；整条退出（含被 INT / TERM 停）时 finish 也停它；
#   它每个间隔核一次主进程还在不在，主进程没了自己退。MERGE_AND_VERIFY_PROGRESS_INTERVAL（秒，正整数）只给自证缩小间隔，平常设了拒（同弄坏开关，settle_break_switches）。
# 日志落 $MERGE_AND_VERIFY_LOG_DIRECTORY（默认工作根下的 merge-and-verify/<日期>-<进程号>，工作根只在 .claude/hooks/lib_work_root.py 里定义）。
# 改过的包都重编了没有（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 1 行；判法在 scripts/gate/lib_rebuilt_packages.py，这里不另写）：
#   改过的包从给的补丁目录里的 crates.patch 算（--skip-apply 时照样读给的补丁目录，只是不打；打没打先核，见「--skip-apply 先核补丁打过没有」）。
#   槽位那一种（文件头「槽位」）另加拷进槽时内容变了、新加、删掉的 .rs（<日志目录>/slot-changed-sources，逐个 --path 交给 lib_rebuilt_packages.py），
#   带 --skip-apply 的那一趟只核这几份、不读补丁目录；这几份一份都没有、又不读补丁目录的，报「重编核对本次未判」。
#   编完核 test、build 与 clippy 三趟的编译输出：每个改过的包都要有「Compiling <包名> v…」行（clippy 打的是「Checking <包名> v…」，核对时当同一种行，
#   clippy_checking_as_compiling 换写之后交给 lib_rebuilt_packages.py；测试那一步不编的包——只挂特性的测试目标、只交验证包那一步的产品 GPU 包——靠 build 与 clippy 那两趟的行证明这一趟重编了），
#   缺一个记一项红（编译目录里的产物比快照的拷入时刻还新，例如别的趟往同一个编译目录里后编过），出路是换干净编译目录整趟重跑。
#   只核包里改过的 .rs；包里改过的 .rs 一个都不在编译目录的 dep-info 里的包不核（这几趟编译不读它们，例如 验证包 gpu 特性下才编的模块）。
#   没给补丁目录、或补丁里没有要核的包：报「重编核对本次未判」，不记红也不算核过。
# --skip-apply 先核补丁打过没有：建好快照、进快照之后，编之前，在 ${TMPDIR:-/tmp} 下建一份临时副本（git init 的空仓，只拷要验的树里给的那几份 crates.patch 点到的文件），
#   照给的补丁目录的次序倒着逐份 `git apply --reverse`（后打的先倒）：全部倒得掉才算都已打在树上，同一批两份补丁改到同一行也判得对（reverse_check_in_order）。
#   有一份倒不掉、或给的补丁目录不在、或临时副本建不成，就退 1、点名那一份（倒不掉的连同它前面没往下核的份数）、一个都不编，
#   出路是没打过的那几份去掉 --skip-apply 让这里打、次序给错了的照打的次序重给；补丁是三方合并打进去的、撤回了一半、或主工作区上另有别人的改动而倒不掉的，这两条走不通，
#   出路里另写整条命令：先交 scripts/gate/patch-of-the-current-tree.py 按此刻的主工作区现算一份补丁目录（照给的次序，--out 在这一趟的日志目录下），出了目录再拿它 --skip-apply 验
#   （原命令里别的参数照旧带上；current_tree_patch_way_out）。临时副本用完就删，快照一个字节不动。
#   没有 crates.patch 的目录（只有 mutations-append.tsv 的）不核，报「本次未核」；不带 --skip-apply 时不核（这里自己打）。
# 先验后合（.claude/main-agent.md「写与验分两段」验证段）：不带 --skip-apply 的那一趟，补丁只打进快照、主工作区一个字节不写（文件头「快照」），
#   补丁这时只在写段的段树（有依赖的几批才有）与这一趟的快照上。这一趟的出路照这一趟的形态打，不假定补丁已在主工作区上：
#   合进主工作区的那一份补丁目录：打补丁之前把补丁动到的文件（crates.patch 新旧两侧的路径，有变异表文件的另加 crates/mutations.tsv）在快照里的原样存进
#     <日志目录>/merge-baseline、动到的路径写进 <日志目录>/merge-baseline.paths（MERGE_BASELINE_PROGRAM；存不成退 1、一个补丁都不打）；全部打上之后照「原样 → 快照此刻」
#     算一份补丁目录 <日志目录>/merge-patch（MERGE_PATCH_PROGRAM）：crates.patch（临时 git 仓里 git diff --binary --full-index --no-renames，补丁新建的写成新文件、删掉的写成删除，
#     登记表的改动一起在里面）、baseline/<路径>（原样，apply-writer-patch.py 三方合并取基线用）、变异表按名字比出的 mutations-append.tsv、mutations-replacements.tsv、
#     mutations-delete.txt（替换一行是改后的行接 merge-baseline 里同名那一行、删除一行是 merge-baseline 里那一行：都带前像，apply-writer-patch.py 打之前逐行比对主工作区此刻那一行，
#     格式在它文件头「补丁目录里的文件」），与各份补丁目录的 result.md 照给的次序接成的一份 result.md（每份前面一行标题写补丁目录）。做不成记一项红。
#     日志目录里上一趟留下的 merge-baseline、merge-baseline.paths 与没合过的 merge-patch 照删；merge-patch 里有 apply-outcome.tsv 的（合过）退 2、一个补丁都不打。
#     几份补丁合成一份，合进主工作区就是一次 apply-writer-patch.py：它自己整份写完或一个字不动（它文件头「换上与被杀」），没有合到一半。
#   变异锚点核（merge-patch 算好之后；anchor_check_new_breaks，判法摆在 MERGE_ANCHOR_CHECK_PROGRAM）：判这一批补丁合起来让哪些变异行新红了。别批变异行的原文原来在文件里恰好命中 1 次、
#     补丁把它改成 0 次或多次，打补丁那一步（apply-writer-patch.py 不加 --dry-run 的真打不跑变异锚点静态核）与合进主工作区那一步都看不见，只有合完另交的 mutation-tables 格才报，
#     那一格对整棵树判、存量红行多，认不出这一批新弄断的。这里不另写判法：调 apply-writer-patch.py 的 static_check_report（它文件头「变异锚点静态核」），基线是开跑那一刻的快照
#     （快照此刻的拷贝范围，补丁动到的文件换回 <日志目录>/merge-baseline 里的原样、补丁新建的删掉，变异表一起换回），补丁之后的树是这份基线加 merge-patch 的 crates.patch
#     （只收落在拷贝范围里的文件段）与快照此刻的变异表；补丁之后有、基线上没有的红行判红（这一趟记一项红、不打合进主工作区的命令），逐行点名「<种类> <行号> <变异名> 原文在 <文件> 里命中 N 次，要恰好 1 次」，
#     出路是以这一趟留下的快照为基线重出那几行替换，锚着的代码被这一批整段删掉、那一行已没有意义的改写进 mutations-delete.txt（每行是快照里那一行的原样），
#     没留快照（没设 MERGE_AND_VERIFY_KEEP_COPY=1）的先带它整条重交；补丁之前就红的行不算，只报一行计数；RELEASE 行同 ANCHOR 一样去掉行号再比（补丁删掉前面一行，补丁前就红的 RELEASE 行号少 1，不算新红）。
#     核没做成（拷贝范围算不成、static-check 判不了）同样记红、点名原因。
#     新红全数写进日志 <日志目录>/anchor-check.log（static_check_report 的 shown 给 None，一处不省），标准输出只列日志的前 ANCHOR_CHECK_SHOWN_LINES 行并写明一共几处、全部在日志里。
#     两趟 static-check 先后跑：真仓上一趟（补丁之前、补丁之后各一次 static-check）约 400–740 秒、内存峰值约 15 GiB（量的；挂钟受同机别的任务影响，各趟不同），日志 <日志目录>/anchor-check.log。
#     带 --skip-apply 的那一趟补丁已在要验的树上，没有 merge-patch：照样核，补丁之前的树用 check_patches_applied 同一份倒打（reverse_check_in_order 在临时副本里照给的次序倒着逐份 git apply --reverse，
#     补丁点到的文件倒回原样；变异表照各份补丁目录的 mutations-append.tsv、mutations-replacements.tsv、mutations-delete.txt 倒着摆回补丁之前，倒不回的记红、点名哪一行），
#     补丁之后就是快照此刻；原样与 merge-patch 照不带 --skip-apply 的同一份程序算（<日志目录>/anchor-skip-baseline*、anchor-skip-patch），耗时与内存同上。
#     给了补丁目录而里面什么都没有的、不带 --skip-apply 时 merge-patch 里什么都没有的，都不做。带 --skip-apply 而没给补丁目录的那一趟也不核（没有补丁，摆不出补丁之前的树），
#     打一行「变异锚点核本次未核」点名为什么，不记红。
#     MERGE_AND_VERIFY_STATIC_CHECK 换掉 static-check 的命令（<脚本> static-check <树>），只给自证（settle_break_switches 拒平常设了的）；自证里只在带 ANCHOR_CHECK_SAMPLE_MARK 标记的样本树上做。
#   合入前的 --dry-run（merge_in_commands 的 ②）：上面的变异锚点核与开跑那一刻的指纹都对着开跑那一刻的快照与主工作区，开跑之后别的会话改了这一批追加的变异行锚着的文件、
#     往表里追加了点名同一目标的变异行、改了变异行要看的登记表（static-check 读被判那棵树的 .claude/gate.d/stage-inputs.tsv），合进去之后才断。所以合之前多一条
#     task-server/task submit -- python3 scripts/gate/apply-writer-patch.py --dry-run <日志目录>/merge-patch（不带 --root：对着这份脚本所在的主工作区）：在合入那一刻的主工作区上，
#     先把拷贝范围加补丁改到的文件从主工作区同一刻拷成补丁之前的树，再整棵拷成补丁之后的树、套上 merge-patch，两棵树各跑一次 static-check（apply-writer-patch.py 文件头「变异锚点静态核」；
#     两趟要几百秒，别的会话这时改了主工作区也不影响基线；merge-patch 带拷贝范围之外的文件段，拒算表、角色表，照样套得上），补丁之后有、补丁之前没有的红行才拦（退 2、一个字没改、逐行点名），
#     补丁之前就红的不算；同一份文件里不受影响的追加不拦。罩住的有：这一批自己追加的行锚在补丁没改到的文件上而那份文件开跑之后被别的会话改了、别的会话开跑之后追加锚在这一批改到的文件上的行、
#     登记表开跑之后被改（这一批追加的行点名的测试目标被登记成崩溃枚举用例，或这一批登记了测试目标而别的会话追加了点名它的行）。代价是多一趟约 400–740 秒、内存峰值约 15 GiB（量的，同上），
#     出路里写明。合进主工作区那一段按 ①到⑤ 的次序打：① 记下此刻变异表里锚在这一批补丁改到的文件上的行（只读、秒级，写进 <日志目录>/anchored-rows-before.txt）、② 这一条 --dry-run、
#     ③ 照 ① 同一条命令重算再与 ① 比（② 的 static-check 只看它读表那一刻的表，它跑着的几百秒里别的会话往表里追加、改了锚在这一批改到的文件上的行，② 看不见，③ 补上）、
#     ④ 核主工作区自开跑以来没变与核这一批要换掉、删掉的变异行没被改（两条快核排在 ② 之后、紧挨着合的那一条）、⑤ 合。③ 退非 0 不整批重交，从 ① 重做（表变了，再核一遍就够）；④ 退非 0 整批重交。
#     ② 各结局分开写出路（合入这一段打的出路以这一段为准，--dry-run 自己拒绝说明里「改好再跑一次」那一句只在合入这一段之外适用）：退 2 点名「变异表有 N 处静态核红」的整批重交；退 2 写「没做成」「判不了」的，
#     照它点名的原因办（工具的毛病当场报给管它的会话），办完从 ① 重做；被停、撞内存上限这类不是 2 的，照原样再交同一条 --dry-run；⑤ 退 2 等完（开工快照、同一个仓有别的在换）与三方合并冲突、
#     核与换之间被改了的，从 ① 重做到 ⑤。弄坏开关 merge-in-dry-run-dropped：不打 ②。
#   验绿（verify 退 0）或判不出测试目标（退 3）：汇总行之后打验证段第 2 步在快照上起的几条与合进主工作区的整条命令（merge_in_commands），这里自己不打、不合：
#     没设 MERGE_AND_VERIFY_KEEP_COPY=1 的只打一句没留快照、带它整条重交（第 2 步要在这一趟的快照上跑）；
#     退 3 的另打一句：测试那一步与验证包那一步没跑，合之前照 .claude/skills/merge-verify/SKILL.md「写与验分两段」验证段第 1 步「退 3」那一支办；
#     第 2 步那几条：快照路径（--cwd 与根都写它）、crash-case-check.py 判 placement,modules,provenance,must-red、refused-test-registry.py list、改到必红世界表的另一条 formal-verification 点那几张表里新加或改过的行（上面「登记表」必红世界表那一条；
#     lib_merge_verify_formal_tables.py 的 second-step 打；这一段的三条都由它打），这一批新写、改写的变异行名
#     （取合进主工作区的那一份补丁目录里的 mutations-append.tsv 与 mutations-replacements.tsv：照补丁次序合完之后新加、改写的行，后面的补丁 mutations-delete.txt 删掉的不在里面；
#     crates-mutation-replay 带 --item 点它们）；
#     合进主工作区：按 ①到⑤ 的次序逐条做（上面「合入前的 --dry-run」），前一条退 0 才做下一条：① 在仓根记下变异表里锚在这一批补丁改到的文件上的行；② 合入前的 --dry-run（经服务交）；
#     ③ 照 ① 同一条命令重算再与 ① 比；④ 在仓根核主工作区自这一趟开跑以来没变（下面「开跑那一刻的指纹」），有这一批要换掉、删掉的变异行的再一条核那几行；
#     ⑤ 再一条 task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <日志目录>/merge-patch（不带 --root：打进这份脚本所在的主工作区）。
#     ③ 退非 0 从 ① 重做；④ 退非 0 不合，照原命令去掉 --fingerprint-of、--item 照 --list-items 对这一批补丁目录重列，整批重交、第 2 步在新留下的快照上重验。
#     ⑤ 退 2（一个字没改）照拒绝说明里的原因分开办：开着的三方轮开工快照、同一个仓有别的在换只等，等完从 ① 重做到 ⑤；
#     三方合并冲突、核与换之间被改了的，也从 ① 重做到 ⑤（④ 退非 0 照上一句整批重交）；走不到的理由那一节行号对不上的，在 merge-patch/result.md 末尾照拒绝说明点名的 文件:行 补一节，再从 ① 重做到 ⑤。
#     不是 2 的结局（被停、撞内存上限）先交 apply-writer-patch.py --pending-status，退 3 的照它的出路 --rollback-pending（或 --finish-pending）收，退 0 的从 ① 重做到 ⑤。
#     合完交 apply-writer-patch.py --read-outcome <日志目录>/merge-patch/apply-outcome.tsv，退 0 才算合全；再交 mutation-tables 那一条（下面「门禁 code-source-discipline 的 mutation-tables 格」），
#     删这一趟留下的快照（槽位那一种是放这一趟留着的槽：rm -rf -- <槽>/kept <槽>/.owner <槽>/.deliverables <槽>/repo/target，文件头「槽位」）。
#     带 --skip-apply 的那一趟（补丁已在主工作区上）与退 1、2、75 的都不打这几条。
#   开跑那一刻的指纹（不带 --skip-apply 的那一趟；拷快照之前记，记不成退 1、一个补丁都不打，日志 <日志目录>/main-tree-at-start.log；MAIN_TREE_FINGERPRINT_PROGRAM）：
#     起点：补丁改到的路径（crates.patch 新旧两侧）往上找到的第一份带 [package] 的 Cargo.toml 所在的包目录，连同工作区里它们照 path 依赖传递依赖的包与传递依赖它们的包
#     （各包 Cargo.toml 的 dependencies、dev-dependencies、build-dependencies 与 target.<平台> 下这三张表里写了 path 的，写成 { workspace = true } 而树根
#     [workspace.dependencies] 里同名那一项写了 path 的，与包名在树根 [patch.<源>] 里写了 path 的；工作区成员照树根 Cargo.toml 的 [workspace] members，通配照展开），
#     有包目录时另加树根的 Cargo.toml、Cargo.lock 与 .cargo/（在的才算）；不落在包里的只算那一份文件（此刻在的普通文件）。登记表（REGISTRATION_TABLES）与 crates/mutations.tsv
#     不进起点：各会话都在往里追加，三方合并在 apply-writer-patch.py 里核（冲突退 2）。这一批要换掉、删掉的变异行（mutations-replacements.tsv 的第一段、mutations-delete.txt 的名字）
#     在要验的树 crates/mutations.tsv 里那几行的原样另记进 <日志目录>/main-tree-at-start.mutation-rows（按字节排序）。
#     指纹不罩变异表与登记表里开跑之后的追加，变异锚点核看的是开跑那一刻的快照，这两样由合进主工作区那一段的合入前 --dry-run 对着合入那一刻的主工作区核（上面「合入前的 --dry-run」）。
#     包目录里不算 .git 与名叫 target、target-*、__pycache__ 的目录（与快照不拷的同一组），符号链接不算。
#     每份普通文件一行 sha256sum 的写法（绝对路径，按字节排序）写进 <日志目录>/main-tree-at-start.sha256，起点（包目录与单份文件）一行一个写进 <日志目录>/main-tree-at-start.roots。
#     合进主工作区那一段 ④ 的头一条照起点现算同一张单子与它比（find <起点…> … | LC_ALL=C sort -z | xargs -0 -r sha256sum | diff -u <指纹> -，整条的退出码是 diff 的），
#     变异行那一条照记下的行在此刻的表里逐行找（LC_ALL=C grep -xF -f <记下的行> <表> | LC_ALL=C sort -u | diff -u <记下的行> -）：都退 0 是补丁改到的包与它们依赖、被依赖的包
#     自开跑以来一份没改、没加、没删，要换掉、删掉的变异行没被改。先记指纹再拷快照：记完之后、拷完之前被改的照「变了」算。起点一个都没有的不打头一条，记下的变异行是空的不打后一条。
#     --fingerprint-of <日志目录>：开跑先照那一趟记的起点与变异行现算一遍（记进 <这一趟的日志目录>/main-tree-recheck.*），与那一趟的两份逐字节比，对不上退 2、一个补丁都不打，
#     出路写整批重交（去掉 --fingerprint-of，--item 照 --list-items 对这一批补丁目录重列）；对上了照常记这一趟自己的指纹。只重跑红的那几项（带 --item）的那一条出路自己带上它，带的是这一趟的日志目录：
#     每一趟开跑时核过上一趟，最后合进主工作区那一趟的指纹在头一趟的起点上等于头一趟开跑那一刻。
#   测试那一步红：只重跑红的那几项的合入后验证（merge_rerun_command）不带 --skip-apply、在 submit -- 之后带 env MERGE_AND_VERIFY_KEEP_COPY=1，带 --item <红的项>… 与同一批补丁目录
#     （照给的次序全给），末尾带 --fingerprint-of <这一趟的日志目录>；红的测试函数归不到项的，打整批再验的那一条（--item 点这一趟跑过的全部项、不带 --fingerprint-of，范围写项数）。
#     rerun-failed-tests.py 那一条不打：它打的 cargo test 对着主工作区起，那里没有这批补丁。
#   验证包那一步红：只重跑的是合入后验证（verification_merge_rerun_command），不是主工作区那一份门禁：--item 点这一趟跑到的验证包目标那几项
#     （登记的崩溃枚举用例与挂特性的文件都照它们挑，块与测试函数照键复用）；红的那一次是产品 GPU 包的真卡对拍（点了 --item 时那一格不起）或三格整格的，
#     不点项、submit 带 --whole-package-confirmed（下面「只重跑的命令点名」）、范围写补丁份数；展开深度缩过的带 --crash-expand-up-to <同一个 n>。
#   改完再验：修补以这一趟留下的快照为基线写（实现员副本从它建），修补的补丁目录接在那一条的补丁目录之后再交；修补还改到别的测试目标的，
#     去掉 --fingerprint-of，--item 照 --list-items 对全部补丁目录（含修补的）重列、范围写项数。
#   只重跑的命令点名（经 task-server/task submit 交的合入后验证不带 --item、门禁点起测试的格不带 --item，提交时被服务当没点名拒：task_server_lib.py 的 unnamed_run_problem）：
#     这里打出的每条只重跑的命令照原样交得过：合入后验证带 --item（只重跑红的点红的项；整批再验点这一趟跑过的全部项），门禁带 --item；
#     要罩的东西点了 --item 就不起的（产品 GPU 包的真卡对拍那一格）不点项，submit 与 -- 之间带 --whole-package-confirmed <整包确认文件>（WHOLE_PACKAGE_CONFIRMATION_PLACEHOLDER）：
#     交之前照 .claude/skills/merge-verify/SKILL.md「写与验分两段」验证段第 1 步「退 3」那一支问用户，照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍写成文件，换掉占位。
#     --skip-apply 核不过的出路里拿按此刻树现算的补丁目录再验的那一条，带原命令点的 --item；原命令没点项的同样带 --whole-package-confirmed。
#     弄坏开关 rerun-batch-unnamed、rerun-confirmation-dropped、verification-rerun-items-dropped 各弄坏一处（文件头开关表）。
#   打补丁核不过（apply-writer-patch.py 退 2、整批停）的出路另写段树那一句：补丁照段树的次序给、后一份以段树上打过前一份的样子为基线（「段树此刻 → 改后」）时，
#     照次序打进快照，几份改同一行是叠着的、照打得上；会停的是主工作区在段树建好之后又改了补丁改的同一段（快照与段树的基线不同，三方合并冲突），
#     或同一个基线上各自改同一行的两份（写段打进段树时就该停在那里）。出路：以主工作区此刻建一棵新段树（SEGMENT_TREE_COMMAND，与 .claude/main-agent.md
#     「写与验分两段」写段第 2 条同一条：--with-test-inputs --with-gate-tables 加三条 --with），冲突那一份之前的照次序打进去，
#     冲突那一份退给实现员以新段树为基线重做，它之后的照次序再打（打不上的同样重做），新的补丁目录整批再交这一条。
#     核不过的那一份是只含拒算表（.claude/gate.d/refused-tests.tsv：crates.patch 里每个 diff --git 头点的都是它、没有变异表文件）的补丁目录时，出路不指实现员与段树：
#     那一份是主 agent 自己从主工作区做的，出路写主 agent 从主工作区此刻重做这一份（.claude/skills/merge-verify/SKILL.md「合入后验证之后判新出现的拒算」第 1 条），替掉这一批里原来那一份，整批再交这一条。
# 门禁 code-source-discipline 的 mutation-tables 格：打补丁脚本（apply-writer-patch.py）不在它自己的进程里起这一格，打上了就退 0、末尾打出要另交的命令
#   （它文件头「mutation-tables 那一格」）；这里也不起它。补丁都打上之后打一行「· code-source-discipline 的 mutation-tables 格本次未跑」，接着写补丁打进
#   主工作区之后另交的整条命令（--cwd 与根都是要验的树，带范围声明），汇总行照写「mutation-tables 格本次未跑」；本次未跑不改退出码，也不算判过。
#   打补丁脚本退 1（它不该退 1：只有它的弄坏开关 mutation-tables-in-process 下才在进程里判这一格）照判红、退 1，点名那份日志。
# MERGE_AND_VERIFY_TREE（只给自证）：要验的树（默认这份脚本所在的仓根），快照从它拷。
# MERGE_AND_VERIFY_CHECK_SH（只给自证）：从哪一份 check.sh 读 lint 表、特性组合、clippy 循环的结构行与四条 cargo 命令（默认这份脚本所在仓的 .claude/singlefs-ai-sop/scripts/check.sh）。
# 上限、线程、机器（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03、M06：脚本里层的包装与上限一律删，一整条一个任务，上限、线程、机器由服务定）：
#   这里没有对应的参数，认不出的参数一律退 2、一样都不做（服务提交时拒这类参数，这里不另拒）。
#   撞内存上限、显存不够、排不上都是服务那一层的结局（服务放大重排、记错误类型），这里不另认包装的退出码。
# 退出码：0 全绿；1 有红（测试、fmt、clippy、build、apply 退 1、快照拷不成或 git init 不成、check.sh 读不出、工作区成员里认不出 验证包、读不了补丁或目标清单、
#   改过的包没重编、起 cargo 之前 RocksDB 预编库的取值判红（下面「RocksDB 预编库的取值」）、验证包那一步红（下面「验证包那一步」）、槽位登记读不出或写坏、
#   带 MERGE_AND_VERIFY_KEEP_COPY=1 的那一趟收尾没把槽记成留着（文件头「槽位」））；
#   2 补丁核不过（apply 退 2）、用法错（含认不出的参数、--item 的项写错、--fingerprint-of 与 --skip-apply 同给）、快照放进了要验的树里面（槽那一层在要验的树里面同样）、
#     --fingerprint-of 那一趟开跑以来主工作区在它的起点上变了、日志目录里的 merge-patch 已经合过（有 apply-outcome.tsv）。验证包那一步本次未跑、测试本次未跑都不改退出码，汇总行里点名。
#   3 判不出改到哪个测试目标（下面「挑测试目标」那几种）：测试那一步与验证包那一步没跑，别的步照跑、没有红；有红退 1。
#     出路：整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户。用户同意只跑落得到的项、整包留给提交时整轮门禁的，带 --unresolved-to-commit-gate 与 --item 整条重交（不退 3，下面「判不出的留给提交时整轮门禁」）。
#   75 按条复用那一层连不上 prune-server（admission.py plan-failure 判不可用），别的没有红；有红退 1，判不出测试目标退 3。
# RocksDB 预编库的取值：clippy --all-features 编工作区全部成员，连同 验证包（编到 librocksdb-sys；build 改到验证包或依赖它的包时同样编到）。
#   补丁打完之后、四条 cargo 之前跑 python3 scripts/hosts/rocksdb-prebuilt.py entry --root <快照>（这台装了、此刻进程环境的取值对得上），
#   判红退 1、一条 cargo 都不起。
#   entry 的跳过办法：树里没有依赖 rocksdb 的清单（自证的样本工作区就是这样），或 PATH 上的 cargo 是带「# rocksdb-prebuilt-entry: selftest-fake-cargo」那一行的假 cargo。
# 验证包那一步（records/2026-09-16-subagent拆分提案.md 第九十节 ㉓；按块、按条都照单元键复用，不起门禁的整格）：test、fmt、clippy、build 与重编核对之后，
#   这一趟跑到的验证包目标里要另走门禁的几样，逐样交快照里那一份门禁 <快照>/.claude/gate.d/crash-point-replay.sh（交给它的根是快照），一样一次、一次一份日志：
#     登记的崩溃枚举用例：先起一次那一道的 --list-items（不起 cargo；日志 verification-list-items.log）读它现列的「crash-case:<用例名><TAB><test=>」，
#       test= 的测试目标是这一趟跑到的验证包目标的 → --check crash-case-replay --crash-cases-only --item crash-case:<用例名>…
#       （只走按块那一路：块照块键复用、缺的当场判；不起包里按条复用那一趟，那几个目标的别的测试函数在测试那一步跑过）；日志 verification-crash-cases.log；
#     挂特性的测试文件：同一次 --list-items 读的「feature-gated:<文件名>」是这一趟跑到的 tests/<文件名> 目标的、跑到了验证包库目标时的 feature-gated:lib →
#       --check feature-gated-tests --item feature-gated:<文件名>…（只编、只跑点名的目标，特性照整格那一组，按条复用）；日志 verification-feature-gated.log；
#     产品 GPU 包的真卡对拍：挑测试目标记了「要特性」、包名是那一道里 PRODUCT_GPU_PACKAGE_NAME="<包名>" 那一行现读的那个包（此刻 singlefs-background-gpu-tests；读不出这一行的
#       照起并点名读不出）→ --check product-gpu-real-card-tests（那一格不收点名，跑的是那个包真卡对拍的几个目标，键相同的照复用）；日志 verification-product-gpu.log；
#       点了 --item 时不起、报本次未跑。别的包要特性的目标报本次未跑（归提交时）。
#   三样都没有就不起那一道，报「验证包那一步本次未跑」与原因（不记红、不算过）。放量分项（scripts/crash/crash-amplification.sh 的 ITEMS）经 --skip 排掉之后这里不起，归提交时。
#   每一次：退 0 是过；退 3 只在交给它的根是它的样本根（有 .crash-point-replay-sample-tools 目录，自证用）时算过，别处退 3 判红；退 77 报本次未跑；别的退出码判红。
#     过了还要读到那一次的行：崩溃枚举用例那一次「✓ 崩溃状态块：现判 …、复用 …、拒算 …」，另两样「按条复用：…现判 N、复用 M、拒算 R」；读不到不算过。现判、复用、拒算加总进汇总行。
#     那一道的 --list-items 退非 0：验证包那一步判红，点名那份日志。
#   红了逐条列出日志里 FAILED 的测试函数、它报的「红的 …」（各自变了的单元）与 ITEM_DONE 里 exit=1 的项，出路写红的测试函数并进写段的下一批，
#     与只重跑红的那几项的整条命令（task-server/task submit -- … 交，范围声明写在 task-server/task 前面：种类 items，范围数是 --check 的格数加 --item 的项数；
#     ITEM_DONE 里认不出红在哪一项的，重跑那一次点的全部项）；带 --skip-apply 的那一趟，那条命令起的是主工作区那一份门禁、交给它的根是主工作区
#     （快照收尾就删，修完在主工作区上重跑）；不带的那一趟补丁不在主工作区上，那条命令是合入后验证（文件头「先验后合」）。
#   在这一整条里面直接起（run_step：两份日志），环境里的 MERGE_AND_VERIFY_BREAK 不交给它。起不起这一步不看环境变量：交给 task-server 的命令什么前缀都不写，
#     什么时候跑由规矩管（`.claude/main-agent.md`「写与验分两段」验证段第 1 步）。
#   --crash-expand-up-to <n>（环境里已设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 的当作给了同一个值）把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO（放量与崩溃枚举用例的引擎读）
#     与 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO（验证包里直接调判器的对拍读）两个变量设成同一个 n，交给门禁那几次与验证包目标的 cargo test（连同算键的请求），
#     缩崩溃域（只往小缩）：缩过域的那一趟汇总里写「缩过域、不算门禁通过」，出路里只重跑的命令也带同一个展开深度：不带 --skip-apply 的那一趟（只重跑的是合入后验证）写 --crash-expand-up-to <同一个 n>，
#     带 --skip-apply 的那一趟（只重跑的是主工作区那一份门禁）在 submit -- 之后写 env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=<同一个 n> SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=<同一个 n>。
#     n 的来历三种（参数、环境里已设的、下面「崩溃域上限」估算回落的）一样两个都交。n 不是非负整数退 2、一样都不做。
#   崩溃域上限（.claude/rules/verification.md「一个门禁：两种跑法」：指定范围的崩溃状态数压在 20 万量级）：有要交的崩溃枚举用例、又没给 --crash-expand-up-to（环境里也没设）时，
#     起那几次之前调 scripts/crash/lib_crash_point_replay_items.py 的 scoped-domain guard（run_scoped_domain；估法与上限 SCOPED_CRASH_STATES_LIMIT 都只在那份库，文件头「指定范围的崩溃域上限」）。
#     点名的项是这一趟 VERIFICATION_CASE_ITEMS 落到的合表项，每项只数这一趟点名的用例：放量分项与没点名的用例这一步不起，不算（verification_crash_domain 的 python 段照合表分组）。
#     起读库程序、运行记录的位置、GPU 开关与入口 scripts/crash/crash-point-replay.sh 的 scoped_domain_guard 同一套环境（放量家目录的 runs，共享根里两台同一份）；日志 verification-crash-domain.log。
#     判法三支：不超上限（含全部复用）照旧、整个域；
#     超上限或有项估不出、而各项在同一展开深度下有记录、合计不超上限（suggested_depth 算得出 n）：CRASH_EXPAND_UP_TO 取这个 n 接着跑，打一行说明，门禁那几次都带它（同给了 --crash-expand-up-to <n> 的门禁那一段），
#       汇总写「缩过域、不算门禁通过」，出路里只重跑的命令带同一个展开深度（写法同上一段）；测试那一步这时已跑完（登记的崩溃枚举用例与放量分项经 --skip 排掉，不在那一步里），不受它影响；
#     有项估不出或超上限、又算不出 n：CRASH_EXPAND_UP_TO 取 CRASH_DOMAIN_FALLBACK_DEPTH（5）接着跑，打一行说明这个值的来历，别的同上一支。这一步编完了不白费：缩域接着跑，不退出、不重编。
#     给了值的不调这份库、照给的跑：值不小于引擎展开上限常量的也照给的（那一趟等于没缩，汇总照写缩过域），要全量就这样给，或留给提交时不指定范围的那一趟。估算的 python 段起不来或退非 0：判红、验证包那一步不起，不当估不出。
#     不指定范围那一趟（提交时的门禁、不带 --item 的入口）不经这里，判法与域不变。MERGE_AND_VERIFY_CRASH_DOMAIN_LIBRARY 换掉那份库（默认是快照里那一份），只给自证。
#   MERGE_AND_VERIFY_VERIFICATION_STAGE 换掉那一道（默认是快照里那一份；出路那条命令里也写它）、MERGE_AND_VERIFY_VERIFICATION_ROOT 换掉交给它的根
#   （默认是快照；出路那条命令里也写它）；都只给自证。
# 显卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：卡只归 GPU 批处理验证服务（gpu-verification-server/），它开卡、持卡、清场、判。
#   这一份不领卡、不挑卡、不清场，文件头不写要卡的声明：打补丁、编、测试、fmt、clippy、build、重编核对、估崩溃域都不上卡。
#   验证包那一步交门禁的每一次（verification_call）照原样带着环境里的 GPU_CARDS_BY_PRIORITY（有就交、没有不设），不另设、不 -u；
#   上卡的那几段归门禁照它文件头「显卡」办（崩溃枚举用例的判段往判定库写待判行、等 GPU 服务判；带 gpu 特性那一趟与真卡对拍那一趟把环境里有的卡原样交给 cargo）。
#   GPU 服务不在跑、要卡的测试读不到结论，都是门禁那一格判红：这里照「验证包那一步」的判法记红、退 1，出路里只重跑的命令照样经 task-server/task submit 交。
# 逐项点名与按条复用（测试那一步；.claude/rules/test-script-discipline.md「入口脚本：不带参数就是不指定范围」「判定可复用：按输入存，按前缀剪」）：
#   项：挑出的测试目标，一个目标一项，写成 <包>:<目标根文件>（目标根文件从包目录起，就是 cargo 段头 `Running …` 里的源文件，
#     例 singlefs-verification:tests/model_differential.rs、singlefs-core:src/lib.rs），文档测试 <包>:doc（只有变异行点了 --doc 才有）。项的写法、挑目标的参数、日志里一段归哪一项，
#     都照 scripts/gate/rerun-failed-tests.py 文件头「项的写法」（test_targets_of_package、section_item；这里经 importlib 调，不另写一份）。
#   --list-items：逐行打「<项><TAB>挑目标的参数」，不编、不跑、不建日志目录与编译目录；项照这一趟的挑法（给的补丁目录）。
#     不带 --skip-apply、给了补丁目录的：在 ${TMPDIR:-/tmp} 下新建一份临时快照（拷法同「快照」那一段的 take_snapshot）、git init、照给的次序逐份 apply-writer-patch.py --root <它>，
#     在打上全部补丁的这一份上挑（补丁新建的测试目标、变异行点名的新目标与真跑那一趟一样认得出），列完删（退出时都删）；带 --skip-apply 的补丁已在要验的树上，照要验的树此刻的样子挑。
#     退 0；有判不出测试目标的退 3（同真跑，逐条打原因；登记表与 verus-pins.tsv 登记的必红世界表不算判不出，它们的 formal-verification 项不在这里列，只在真跑那一趟验证段第 2 步打），落得到的那几项照列（带 --with-keys 的照打 ITEM_PLAN），补丁打不进临时快照退 2（同真跑打补丁核不过），读不了、拷不成退 1。
#     另带 --with-keys 时每项再打一行 ITEM_PLAN item=<项> to_judge=<要现判的条数> reusable=<能复用的条数>（键在列项的那一棵树上算，经 admission.py test-plan、连 prune-server；
#     文档测试一项一件、算要现判；一个 #[test] 都没有的包两样都是 0；算不成的那一包不打它的 ITEM_PLAN，另打一行「算不出」）；同时给了 --item 的只给点名的项打。
#   --item <项>：只跑点名的项；写错（不在这一趟挑出的项里）退 2、列出可点的项（真跑在打了补丁的快照上认）。不点就是挑出的全部项。
#     每个包一条（test-targets-parallel.py 或自证的 serial 一条 cargo test，`-p <包>` 只带这个包点到的 --lib / --test / --bin / --example / --bench），点了 <包>:doc 的另起一条
#     cargo test --offline --no-fail-fast -p <包>… --doc。点名单跑一项与它在整趟里跑的判定相同：同一套键、同一份库、同一个判法。
#   按条复用：每条测试函数一条判定（unit-verdicts 族，prune-server 那一份库；库名照 admission.py 文件头「判定库」，库根不进键）。种类两样：
#     验证包之外的包是 merge-and-verify.sh:test-functions（dev 档：不交 --profile，cfg 带 debug_assertions）；验证包是 crash-point-replay.sh:test-functions，
#     请求照 admission.py test-functions-request crash-point-replay.sh 打的整份交（--profile release，与门禁 crash-point-replay 的 crash-case-replay 格同一种类：
#     这里判绿记下的那一格照键复用，那一格记下的这里照键复用），另加 --exclude-registered（登记的崩溃枚举用例与放量分项记 registered、不判、经 --skip 排掉）。
#     键由 admission.py test-plan 算（它文件头「单元键」）：测试函数的单元闭包、闭包真引到的 cfg 原子与这一次编译的真值、
#     工具链、解析出的闭包真引到的依赖版本、判法 lib_libtest_result_lines.py::recordable_results 的闭包（judge:）、改判定域的环境变量（admission.py test-functions-request 照这一刻的环境打的
#     variable:）。路径、名字、行号、提交与树不进键。每个要测、跑到的项所在的包一份计划（$logs/test-plan.<包>.out 与 .plan）；
#     包目录下一个 .rs 里都没有测试属性（#[test]、#[<路径>::test]）的不起 test-plan，计划是空的（没有测试函数可复用）。
#     键相同、库里不红的经 cargo test -- --exact --skip <名> 跳过（名单 admission.py plan-skip-names，各包只算点到的目标）；
#     键对不上与拒算的照跑。跑完照 rerun-failed-tests.py 的 section_item 把 test.log 逐段归到项与包（认不出、拿不准的段不记库，打一行），每个包那几段交 lib_libtest_result_lines.py recordable
#     逐条认结局、admission.py plan-green-names 取判绿可记的名字；FAILED 的测试函数按（目标根文件，libtest 全名）对到计划里要现判的那一行交 --red-names，与判绿的同一次 verdict-record 记进库：
#     红的照同一把键记红、撤掉这把键原有的绿，下一趟现判（记红撤绿）。文档测试与拒算的不复用、不入库（每趟照跑）。
#     逐条结局写进 $logs/test-functions.tsv：「<名字><TAB><项><TAB><现判|复用|拒算><TAB><绿|红|没跑到>」（现判 = 这一趟日志里有它的结果行）。
#     复用哨兵照 admission.py（test-plan 抽、verdict-record 判；被判的树下有 .unit-verdict-sample/ 的不抽）。
#     按条复用失灵（test-plan 退非 0 或没打 PLAN、verdict-record 报哨兵红）：照 admission.py plan-failure 判，失灵的那一包一条都不跳、全部现判跑完（验证包不起，见 test 一段），测试这一步判红；
#     它判 prune-server 不可用（退 75）的，测试这一步不起 cargo、报不可用，别的步照跑，整条退 75（另有红退 1）。
#     成功句报「按条复用：现判 N、复用 M、拒算 R」（跑到的项里计划的条数）；cargo test 退 0 而跑到的与复用的加起来是 0 才判「一条测试都没跑」。
#   红了只重跑红的：带 --skip-apply 的那一趟，出路打 python3 scripts/gate/rerun-failed-tests.py <test.log> --item <红的项>…（红的测试函数归不到项的另打 -p <包> 那一种）；
#     修完也可以只重跑那几项的合入后验证：照出路打的那一条（范围声明种类 items、范围是项数，task-server/task submit -- bash scripts/gate/merge-and-verify.sh --skip-apply --item <红的项>… <同一批补丁目录>；
#     判绿记过库的照复用，只现判上次红的与改后键变了的）。不带 --skip-apply 的那一趟照文件头「先验后合」。
#   机读行（写法同 scripts/crash/crash-point-replay.sh 文件头「输出」，task-server/task_server_lib.py 的 ITEM_PLAN_LINE、ITEM_DONE_LINE 认）：
#     ITEM_PLAN 见上面 --list-items；ITEM_DONE item=<项> seconds=<秒> judged=<现判的件数> reused=<复用的件数> exit=<0|1|77>，真跑测试那一步收尾每项一行（stdout）：
#     秒数是这一项那几段 libtest 汇总行「finished in <秒>s」之和（只算测试跑的时间，编译几项共用一份、不分摊），四舍五入成整数；件数照这一包的计划（文档测试一件现判）；
#     exit 1 是这一项有 FAILED 的测试函数（cargo test 退非 0 而一条 FAILED 都归不到项时，跑到的每一项都记 1），77 是日志里没有这一项那一段（没跑到），别的 0。
# 弄坏开关 MERGE_AND_VERIFY_BREAK=<项>（只给证红用；打开时自证退 1）。逐项写开关名、弄坏哪一处、打开后自证哪几项红
# （自证的三例没有标签，项名直接写那条「✗ 自检：」失败行的句首，括号里写那一行接着报的）：
#   MERGE_AND_VERIFY_BREAK=logs-single  每一步的日志只写一份全量的（不经 two-logs.sh，没有 .problems.log）
#       「一条测试红时每一步的日志应当出两份（test.log 全量；test.problems.log 带行号收那条 FAILED）」
#   MERGE_AND_VERIFY_BREAK=tests-one-binary-at-a-time  测试那一步一条 cargo test 一个测试二进制接一个地跑，不经 test-targets-parallel.py
#       「测试那一步应当经 test-targets-parallel.py 按测试目标并行起」（实际假 cargo 只收到一条 test --offline --no-fail-fast -p demo --lib --test alpha）；
#       连带「测试那一步的 cargo test 在 -- 之后应当什么都不带」（那一格还核并行起那一趟的成功句里有共 4 条测试，一个测试二进制接一个地跑成了 1 个测试二进制、共 2 条测试）
#   MERGE_AND_VERIFY_BREAK=ignore-red  verify() 抓到 FAILED 行、cargo test 退非 0 都不记红，一律报测试全绿
#       「一条测试红时应当退 1、列出那条测试与只重跑它的命令」（实际退出码 0，报『✓ 测试全绿（0 个测试二进制）』）；
#       连带「快照：测试红时也应当删掉快照与编译目录」（那一格的测试红被吞掉，退 0 不是要的 1）
#       连带红的格：「✗ 自检：一条测试红时每一步的日志」、
#       「✗ 自检：测试那一步…按测试目标并行起」（红的那一趟要退 1）、
#       「逐项与复用：记红撤绿」（FAKE_UNITS_RED 那一趟退 0）、
#       「先验后合：测试那一步红了，只重跑红的那几项应当是不带 --skip-apply 的合入后验证…」「先验后合：测试那一步红了，只重跑的那一条应当带 env MERGE_AND_VERIFY_KEEP_COPY=1…」（红被吞掉，退 0，出路只剩没留快照那一句）、
#       「判不出的留给提交时整轮门禁：带 --unresolved-to-commit-gate、点了项的那一趟测试红了应当退 1…」（那一趟带 --unresolved-to-commit-gate、点了 --item，测试红被吞掉，实际退 0，没有只重跑红的那一条）
#   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（lib_work_root.py 的开关：工作根退回旧根 /tmp/claude-<uid>）  没设日志目录时日志落进旧根
#       「没设日志目录时日志应当落在工作根下」「自检：编译目录不在工作根下」（后一格也不给日志目录、工作根指进 scratch，日志目录落进旧根，那一格核日志目录照旧在工作根下那一半判不过；
#       跑完删掉旧根下这两格多出的两个 merge-and-verify/<日期>-<进程号>）
#   MERGE_AND_VERIFY_BREAK=snapshot-keeps-times  一趟一份那一种的快照带源树的修改时刻拷（take_snapshot 的 rsync 带 --times）
#       「快照：MERGE_AND_VERIFY_KEEP_COPY=1 时应当留着快照与编译目录、打出路径，快照里的修改时刻是拷的那一刻」（实际留下的快照里那份文件是源树的 2020 年的 mtime）；
#       连带「往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验，应当重编改过的 walk.rs、测试全绿」（实际退出码 1，假 cargo 报 Fresh、那一条 FAILED）、
#       「编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0」（实际快照里 walk.rs 比上一趟的 dep-info 旧、没有 Compiling demo，退 1）
#   MERGE_AND_VERIFY_BREAK=verify-in-main-tree  不拷快照，打补丁、编、验、验证包都在要验的树（主工作区）上
#       「快照：跑着的时候主工作区里那份源码被改，结论与复用键应当照开跑时那一份算」（实际测试编的是改后的 source_edited_mid_run、验证包那一步起的是改后的那一份门禁）；
#       连带「快照：测试红时也应当删掉快照与编译目录」「快照：全绿时应当在快照里打补丁、收尾删掉快照与编译目录」（没有删了快照那一行，apply 收到的 --root 是要验的树）、
#       「快照：MERGE_AND_VERIFY_KEEP_COPY=1 时……」「快照：快照根应当 git init」「快照：留着快照的那一趟应当在日志目录写归属记录 .owner」（没有快照）、
#       「往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验……」「编过之后 --skip-apply 再验同一批补丁……」（树里是旧 mtime，假 cargo 报 Fresh）、
#       「先验后合：验绿了应当打第 2 步在快照上起的几条与合进主工作区的整条命令」（第 2 步那几条的 --cwd 是要验的树，不是 <日志目录>/repo）、
#       「必红世界表：补丁改到 verus-pins.tsv 登记的必红世界表」（那一条 formal-verification 的 --cwd 是要验的树，不是 <日志目录>/repo）、
#       「先验后合：合进主工作区那一段应当先打核主工作区自开跑以来没变的那一条，主工作区没动时它退 0」（补丁打在要验的树上，核那一条退非 0）、
#       「先验后合：带 --fingerprint-of 的重跑，头一趟开跑之后主工作区变了应当退 2、一个补丁都不打」（头一趟把补丁打在要验的树上，没动的那一趟重跑也核到变了、退 2）、
#       「先验后合：这一批要换掉的变异行开跑之后被改了，核变异行的那一条应当退非 0」（补丁把那一行换在要验的树上，只追加了别的行的那一趟核变异行那一条也退非 0）、
#       「合入前的 --dry-run：别的会话开跑之后追加的、锚在这一批改到的文件上的变异行应当拦住」
#       「合入前的 --dry-run：这一批只追加一行变异行、锚着的文件开跑之后被别的会话改了应当拦住」
#       「合入前的 --dry-run：这一批自己追加的行锚在补丁没改到的文件上、开跑之后别的会话改了那份文件（只改变异表）应当拦住」
#       「合入前的 --dry-run：这一批自己追加的行锚在补丁没改到的文件上、开跑之后别的会话改了那份文件（连代码一起改）应当拦住」
#       「合入前的 --dry-run：这一批追加的行点名的测试目标开跑之后被别的会话登记成崩溃枚举用例应当拦住」
#       「合入前的 --dry-run：这一批登记崩溃枚举用例、开跑之后别的会话追加了点名它的变异行应当拦住」
#       「合入前的 --dry-run：两棵树同一刻拷出（dry-transient）：别的会话追加的、锚在这一批改到的文件上的行被这一批弄断了」
#       「合入前的 --dry-run 之后的核（window-unchanged）：主工作区没变时 ① 记下、③ 比对、④ 核主工作区没变都应当退 0」
#       （补丁已经打在样本树上：这几格再在样本树上跑 --dry-run，补丁套不上、这一批追加的行已在表里、记下的指纹与此刻的主工作区对不上，应当退 2 的退 0 或点不到行，④ 核主工作区没变退非 0）、
#       「上一次尝试：尝试 2 应当删掉…」「上一次尝试：别的任务号…」「上一次尝试：有进程开着…」（没有快照，这三格要查的 <日志目录>/repo 都不存在：没有删了上一次尝试留下的快照那一行，别的任务号与尝试号不小的那几棵、被开着的那一棵都查不到）、
#       「槽位：占槽、放槽，槽的编译目录在这一台的编译根下」「槽位：留着的槽不给别的趟、没空槽时这一趟照一趟一份、放了槽再接着用」「槽位：同一槽先后两趟，下一趟只重编改过的包」
#       「槽位：并发两趟各占一槽」「槽位：源树里改了、删了的文件在槽里照改照删，改过的包必重编」
#       「槽位：槽的编译目录里的产物比改过的源码新、改过的 demo 没有 Compiling 行：应当退 1、点名 demo」「槽位：槽的编译目录超上限就清，槽没了的编译目录跟着删」
#       「上一次尝试：撞上限重来的尝试 2 接着用尝试 1 记下的那一槽」（不占槽，日志目录里没有 slot、没有占槽与这一槽的编译目录那几行，留着的槽也记不成）、
#       「槽位：slots 是 0 时清多出来的槽与它的编译目录，留着的槽不动」「槽位：拷进槽之前删掉槽里上一趟留下的编译产物」
#       「槽位：三槽都留着时槽位汇总里有不占槽那一行与三个留着的槽」「槽位：第二台那一层给了编译目录时不占槽，那一行说是第二台那一层给的」
#       「槽位：占到槽的那一趟每条 cargo 收到 CARGO_INCREMENTAL=0、编之前删掉这一槽编译目录里的 incremental，不占槽的那一趟不设」
#       「槽位：调用方设了 CARGO_INCREMENTAL 的照它、不盖，这一槽编译目录里的 incremental 不删」
#       （不挑槽：多出来的槽不清、槽里的编译产物不删，没有不占槽那一行与槽位汇总；不占槽也就不设 CARGO_INCREMENTAL、不删 incremental）
#   MERGE_AND_VERIFY_BREAK=drop-discipline-lint  从 check.sh 读到的 lint 表丢掉最后一条再交给 clippy
#       「clippy 应当带上 check.sh 的 lint 表里每一条」（实际少一条 -D clippy::shadow_unrelated 这类的）
#   MERGE_AND_VERIFY_BREAK=lint-table-optional  check.sh 读不出 lint 表、特性组合、四条 cargo 命令，或结构行对不上时只提一句、照常往下验
#       「check.sh 里读不出 lint 表时应当退 1、不编」与「check.sh 的 clippy 那一行没带 lint 表时应当退 1、不编」（实际退 0、照常编）；
#       「check.sh 的 fmt 那一条换成认不出的包装时应当退 1、不编」与「check.sh 的 test 那一行带了收窄目标的标志 --lib 时应当退 1、不编」（同上）；
#       「check.sh 的缺省特性组合只剩一个时应当退 1、不编」与「check.sh 的 clippy 循环改成只取一个组合时应当退 1、不编」（同上）
#   MERGE_AND_VERIFY_BREAK=cargo-wrapper-fixed  读 check.sh 里 cargo 命令的识别只认 run_cargo <子命令> 那一种包装，不带包装的 cargo 一律当找不到（上游去掉 run_cargo 那一天整个挡住）
#       「cargo 命令现读：真 check.sh 同一趟应当退 0」（真 check.sh 此刻是不带包装的 cargo，读不出 fmt 那一条，退 1、不编）；
#       连带红的格：「✗ 自检：」（凡是经真 check.sh 往下验的格都停在读 check.sh 那一步，没有一格验得到后面）
#   MERGE_AND_VERIFY_BREAK=cargo-wrapper-unrecognized  读 cargo 命令的识别只认不带包装的 cargo，run_cargo <子命令> 当找不到（上游哪天改回包装函数就挡住）
#       「check.sh 的 cargo 命令包了 run_cargo 时应当照读、收到的命令与真 check.sh 相同」（样本里四条都是 run_cargo <子命令>，读不出，退 1、不编）
#   MERGE_AND_VERIFY_BREAK=cargo-arguments-fixed  四条 cargo 命令的参数用这里写死的，不用从 check.sh 现读
#       「check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们」（实际收到的是不带 --verbose、-W clippy::pedantic、--locked 的）
#   MERGE_AND_VERIFY_BREAK=single-clippy-pass  clippy 那一步只跑 --all-features 一趟，从 check.sh 读到的特性组合不用
#       「clippy 应当照 check.sh 的特性组合逐个跑」（实际假 cargo 只收到 --all-features 那一条）；
#       「clippy 一个特性组合红时应当整步红、退 1、点名红的那个组合」（不带特性那一趟没跑，实际退 0）；
#       连带「check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们」（每个组合的 clippy 都带 -W，实际只收到一条）
#   MERGE_AND_VERIFY_BREAK=clippy-combinations-unchecked  check.sh 的缺省特性组合读出不到两个也照收
#       「check.sh 的缺省特性组合只剩一个时应当退 1、不编」（实际退 0、只跑那一个组合）
#   MERGE_AND_VERIFY_BREAK=clippy-combination-red-uncounted  一个特性组合红时照打那一行，不记进整步的红
#       「clippy 一个特性组合红时应当整步红、退 1、点名红的那个组合」（实际退 0，没有整步红那一句）
#   MERGE_AND_VERIFY_BREAK=clippy-registration-ignored  要验的树里有 check.sh 认的特性组合登记也照读缺省组合往下验
#       「要验的树里有特性组合登记时应当退 1、不编」（实际退 0、照常编）
#   MERGE_AND_VERIFY_BREAK=skip-build  不跑 cargo build（改到了包也不起、也不报本次未跑）
#       「应当跑 cargo build --offline --all-targets -p demo」（实际假 cargo 没收到 build）；
#       「build 那一条的环境里应当收到 CARGO_PROFILE_DEV_DEBUG=0…」（假 cargo 没记下 build 那一行）；
#       连带「check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们」（没有 build --locked -p demo 那一条）、
#       「判不出测试目标：」那四格（改到 alpha 的那一格要收到 -p alpha -p beta，没有包的三格要报 build 本次未跑）、
#       「build 那一步：补丁只改验证包时…」（两趟都没收到 build）、「build 那一步：补丁只改变异表时…」（没有 build 本次未跑那一句）
#   MERGE_AND_VERIFY_BREAK=build-whole-workspace  build 不带 -p、照编整个工作区（一个包都没有时也编）
#       「应当跑 cargo build --offline --all-targets -p demo」（实际收到的是不带 -p 的 build --offline --all-targets）；
#       连带「check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们」（收到的是 build --offline --all-targets --locked，没有 -p demo）、
#       「判不出测试目标：」那四格（改到 alpha 的那一格收到不带 -p 的 build，没有包的三格收到了 build）、
#       「build 那一步：补丁只改验证包时…」（两趟收到的都是不带 -p 的 build）、「build 那一步：补丁只改变异表时…」（收到了 build、报 build 过）
#   MERGE_AND_VERIFY_BREAK=build-debuginfo-kept  build 那一条的环境里不设 CARGO_PROFILE_DEV_DEBUG=0（照仓根 [profile.dev] 带调试信息编）
#       「build 那一条的环境里应当收到 CARGO_PROFILE_DEV_DEBUG=0…」（假 cargo 记下 build 那一行收到的是 unset）
#   MERGE_AND_VERIFY_BREAK=workspace-dependencies-ignored  算包时不认写成 { workspace = true } 的依赖（只认自己写了 path 的），build 的包集合与开跑那一刻的指纹都漏掉这样依赖改到的包的包
#       「build 那一步：依赖方写 { workspace = true }…」（实际算出的包只有 core，没有依赖它的 user）、
#       「开跑那一刻的指纹：依赖方写 { workspace = true }…」（实际起点里没有 user 依赖的 core）
#   MERGE_AND_VERIFY_BREAK=patch-paths-ignored  算包时不读树根 Cargo.toml 的 [patch.<源>]，经它换成工作区里的包的依赖不认，build 的包集合与开跑那一刻的指纹都漏掉依赖它的包
#       「build 那一步：registry 依赖经树根 [patch.crates-io] 换成工作区里的包…」（实际算出的包只有 wcore，没有依赖它的 user）
#   MERGE_AND_VERIFY_BREAK=mutation-tables-claimed-judged  汇总写「其中 code-source-discipline 红 0 个」，不报 mutation-tables 格本次未跑、不打另交的命令
#       「打补丁之后应当报 code-source-discipline 的 mutation-tables 格本次未跑、打出补丁打进主工作区之后另交的整条命令」（实际汇总写红 0 个，没有那一行）
#   MERGE_AND_VERIFY_BREAK=no-verification-package-guard  工作区成员里找不到 singlefs-verification 也照挑全部成员
#       「工作区成员里没有 singlefs-verification 时应当退 1、不编」（实际退 0，改了名的 验证包被跑进来）
#   MERGE_AND_VERIFY_BREAK=zero-tests-green  cargo test 退 0 而一条测试都没数到时照报全绿
#       「cargo test 退 0 却一条测试都没跑时应当退 1」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=keep-copy  收尾不删快照与编译目录、也不打路径（快照留在工作根下没人删）
#       「快照：全绿时应当在快照里打补丁、收尾删掉快照与编译目录」与「快照：测试红时也应当删掉快照与编译目录」（实际两份都还在）；
#       连带「快照：MERGE_AND_VERIFY_KEEP_COPY=1 时……」（开关在留现场那一句之前就退出，路径不打）、
#       「跑完运行目录里不剩它建的编译目录……」（全绿、测试红、被 TERM 停三趟的快照与编译目录都还在）、
#       「这一趟的编译目录里还有进程开着文件……」（不删也不核，退 0）、「收尾时前面一步写满盘（用开关演）编译目录照删……」（编译目录还在）；
#       「逐项与复用：reuse-three-questions」「逐项与复用：整趟每项一行 ITEM_DONE」「逐项与复用：记红撤绿」（同一个日志目录的下一趟碰上留下的快照，退 2）
#   MERGE_AND_VERIFY_BREAK=cleanup-only-on-success  只在退 0 时删快照与编译目录，红了留着
#       「快照：测试红时也应当删掉快照与编译目录」（实际退 1、两份都还在）；
#       连带「跑完运行目录里不剩它建的编译目录……」（测试红与被 TERM 停那两趟的快照与编译目录都还在）、
#       「逐项与复用：记红撤绿」（红的那一趟留下快照，下一趟退 2）；
#       连带「收尾时前面一步写满盘（用开关演）编译目录照删」（那一格的收尾退 1，编译目录没删）
#   MERGE_AND_VERIFY_BREAK=keep-owner-unrecorded  留着快照的那一趟（MERGE_AND_VERIFY_KEEP_COPY=1）不写归属记录 .owner（一趟一份那一种写在日志目录、槽位那一种写在槽里，两处都不写）
#       「快照：留着快照的那一趟应当在日志目录写归属记录 .owner」（实际留着快照的那一趟日志目录里没有 .owner）；
#       连带「槽位：留着的槽不给别的趟、没空槽时这一趟照一趟一份、放了槽再接着用」（留着的那一槽里没有 .owner）
#   MERGE_AND_VERIFY_BREAK=skip-rebuild-check  编完不核改过的包有没有 Compiling 行
#       「编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行：应当退 1、点名 demo」（实际退 0）
#       连带：「头一次编、补丁改过的 demo 有 Compiling 行：应当退 0、报都重编」「编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0」（不核就不报「都重编了」，判这一句的两格跟着红）；
#       连带「重编核对读 clippy 的 Checking 行：补丁只改产品 GPU 包的真卡对拍 real_card…」（不核就没有「都重编了」那一句）；
#       连带「槽位：同一槽先后两趟，下一趟只重编改过的包」「槽位：源树里改了、删了的文件在槽里照改照删，改过的包必重编」（没有「都重编了」那一句）、
#       「槽位：槽的编译目录里的产物比改过的源码新、改过的 demo 没有 Compiling 行：应当退 1、点名 demo」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=rebuild-evidence-test-only  重编核对只读测试与 build 的编译输出，不读 clippy 的「Checking」行
#       「重编核对读 clippy 的 Checking 行：补丁只改产品 GPU 包的真卡对拍 real_card…」（测试那一步不跑它、那一趟 build 不打它的 Compiling 行，重编核对判红，没有「都重编了」那一句）；
#       连带「验证包那一步：补丁改到产品 GPU 包的真卡对拍 real_card 时…」（同一趟，重编核对判红退 1，那一格要退 0）
#   MERGE_AND_VERIFY_BREAK=skip-apply-unchecked  --skip-apply 时不核补丁打过没有
#       「--skip-apply 给的补丁没打在树上：应当退 1、点名那份补丁、不编」（实际退 0、照常编，在没打补丁的树上验出绿）；
#       连带「补丁改到 alpha 的库与 beta 的测试目标 case：…」「--skip-apply 给的两份补丁改同一行、照打的次序给：…」与「临时目录不认保留属性时 --skip-apply 照样核得过补丁已打上、退 0」（这三格另核『核过 N 份补丁都已打在树上』这一行，不核就不打）；
#       连带「--skip-apply 核不过的出路里应当写按此刻树现算补丁的整条命令与拿它 --skip-apply 的整条命令」（不核就没有出路）
#   MERGE_AND_VERIFY_BREAK=reverse-check-each-alone  --skip-apply 在要验的树上逐份单独 git apply --check --reverse，不在临时副本里照次序倒着逐份倒
#       「--skip-apply 给的两份补丁改同一行、照打的次序给：应当判都已打上、退 0」（实际先打的那一份单独反着核不过，退 1）；
#       连带「--skip-apply 核不过的出路里应当写按此刻树现算补丁的整条命令与拿它 --skip-apply 的整条命令」（这一支核不过的 die 只写没打过的那几份去掉 --skip-apply，不写那两条整命令）
#   MERGE_AND_VERIFY_BREAK=reverse-check-copy-preserves  --skip-apply 核补丁打没打的临时副本带 -p 拷（保留权限与属主）
#       「临时目录不认保留属性时 --skip-apply 照样核得过补丁已打上、退 0」（实际 cp 报 preserving permissions … Operation not supported，退 1）
#   MERGE_AND_VERIFY_BREAK=way-out-without-current-tree-patch  --skip-apply 倒不掉补丁时，出路里不写按此刻树现算补丁的那两条整命令
#       「--skip-apply 核不过的出路里应当写按此刻树现算补丁的整条命令与拿它 --skip-apply 的整条命令」
#   先验后合（文件头「先验后合」）的开关，点名的格都以「先验后合：」起头（另有一格「一条测试红时…」与「apply 退 2 时…」）：
#   MERGE_AND_VERIFY_BREAK=merge-rerun-assumes-applied  不带 --skip-apply 的那一趟测试那一步红了，出路照带 --skip-apply 的那一种打（rerun-failed-tests.py 与带 --skip-apply 的合入后验证）
#       「先验后合：测试那一步红了，只重跑红的那几项应当是不带 --skip-apply 的合入后验证」（实际带 --skip-apply、打了 rerun-failed-tests.py）；
#       连带「一条测试红时应当退 1、列出那条测试与只重跑它的命令」（红的归不到项，实际只打 rerun-failed-tests.py -p <包>，没有整批再验的那一条）、
#       「先验后合：测试那一步红了，只重跑的那一条应当带 env MERGE_AND_VERIFY_KEEP_COPY=1 与 --fingerprint-of <这一趟的日志目录>」（带 --skip-apply 的那一种两样都不带）
#   MERGE_AND_VERIFY_BREAK=verification-rerun-on-main-tree  不带 --skip-apply 的那一趟验证包那一步红了，出路照带 --skip-apply 的那一种打（主工作区那一份门禁、根是主工作区）
#       「先验后合：验证包那一步红了，只重跑红的应当是不带 --skip-apply 的合入后验证」（实际打的是起门禁的那一条）
#   MERGE_AND_VERIFY_BREAK=merge-in-commands-dropped  不带 --skip-apply、留着快照的那一趟验绿了不打第 2 步那几条与合进主工作区的命令
#       「先验后合：验绿了应当打第 2 步在快照上起的几条与合进主工作区的整条命令」（实际没有那一段）；
#       连带「先验后合：合进主工作区那一段应当先打核主工作区自开跑以来没变的那一条，主工作区没动时它退 0」
#       「先验后合：同包里别的文件开跑之后改了、加了，核主工作区没变的那一条应当退非 0」「先验后合：依赖关系上的包开跑之后改了，核主工作区没变的那一条应当退非 0」
#       「先验后合：这一批要换掉的变异行开跑之后被改了，核变异行的那一条应当退非 0」（那一段整个没打，没有那几条可跑）、
#       「合入前的 --dry-run：别的会话开跑之后追加的、锚在这一批改到的文件上的变异行应当拦住」
#       「合入前的 --dry-run：这一批只追加一行变异行、锚着的文件开跑之后被别的会话改了应当拦住」
#       「合入前的 --dry-run：这一批自己追加的行锚在补丁没改到的文件上、开跑之后别的会话改了那份文件（只改变异表）应当拦住」
#       「合入前的 --dry-run：这一批自己追加的行锚在补丁没改到的文件上、开跑之后别的会话改了那份文件（连代码一起改）应当拦住」
#       「合入前的 --dry-run：这一批追加的行点名的测试目标开跑之后被别的会话登记成崩溃枚举用例应当拦住」
#       「合入前的 --dry-run：这一批登记崩溃枚举用例、开跑之后别的会话追加了点名它的变异行应当拦住」
#       「合入前的 --dry-run：同一份文件里不受影响的追加应当放行」（同上，那一段没打，没有那一条可核）、
#       「合入前的 --dry-run 之后的核（window-fingerprint）：没走到」「合入前的 --dry-run 之后的核（window-table-append）：没走到」「合入前的 --dry-run 之后的核（window-unchanged）：没走到」
#       「合入前的 --dry-run：合并补丁带拷贝范围外的文件段（拒算表一行）时应当照常判」（同上，没有 ② 与 ⑤ 的命令，那一段没打）、
#       「先验后合：判不出测试目标（退 3）的那一趟也应当打合进主工作区的命令、写明退 3 那一支」、
#       「先验后合：打出的变异行名应当是照补丁次序合完之后还在的行」（没有那一行）、
#       「必红世界表：补丁改到 verus-pins.tsv 登记的必红世界表」（汇总行之后没有点那两行的 formal-verification 命令）、
#       「判不出的留给提交时整轮门禁：带 --unresolved-to-commit-gate、点了 --item、有判不出测试目标的改动，应当只跑点名的项…」（那一趟验绿了也不打合进主工作区的命令，实际汇总行之后没有那一段）
#   MERGE_AND_VERIFY_BREAK=merge-in-without-kept-snapshot  没设 MERGE_AND_VERIFY_KEEP_COPY=1 的那一趟验绿了也打合进主工作区的命令（第 2 步没有树可跑）
#       「先验后合：没留快照的那一趟验绿了不该打合进主工作区的命令」
#   MERGE_AND_VERIFY_BREAK=merge-in-dropped-on-untargeted  判不出测试目标（退 3）的那一趟不打第 2 步那几条与合进主工作区的命令
#       「先验后合：判不出测试目标（退 3）的那一趟也应当打合进主工作区的命令、写明退 3 那一支」
#   MERGE_AND_VERIFY_BREAK=merge-patch-single-directory  合进主工作区的那一份补丁目录只算头一份补丁目录动到的文件
#       「先验后合：合进主工作区的那一份补丁目录应当是几份补丁合起来的样子」（实际 crates.patch 里没有第二份补丁才动到的 beta 的 merge_extra.txt）
#   MERGE_AND_VERIFY_BREAK=merge-patch-stat-cache  算合进主工作区的那一份补丁目录时，补丁之后那一份写在基线那一份的原 inode 上（修改时刻照源、git 不比改变时刻），git 的 stat 缓存漏掉大小与修改时刻都相同的改动（只在带 .merge-patch-stat-sample 标记的样本树上生效，别的格的补丁不受影响）
#       「先验后合：合进主工作区的那一份补丁目录里大小与修改时刻都相同的改动不能被 git 的 stat 缓存漏掉」（实际算出改了 0 份文件，补丁目录里没有 crates.patch）
#   MERGE_AND_VERIFY_BREAK=merge-patch-preimage-dropped  算合进主工作区的那一份补丁目录时，mutations-replacements.tsv 只写改后的行、mutations-delete.txt 只写名字（不带前像）
#       「先验后合：合进主工作区的那一份补丁目录里的变异行替换与删除应当带前像」（实际替换那一行只有六段、删除那一行只有名字）
#   MERGE_AND_VERIFY_BREAK=mutation-names-keep-deleted  第 2 步那一段打出的这一批新写、改写的变异行名逐份补丁目录收，不减后面的补丁 mutations-delete.txt 删掉的
#       「先验后合：打出的变异行名应当是照补丁次序合完之后还在的行」（实际列了 2 行，含后一份补丁删掉的 merge-gone-row）
#   MERGE_AND_VERIFY_BREAK=anchor-check-skipped  变异锚点核一进来就放行（不做核、不点名新红的变异行）
#       「变异锚点核：补丁弄断别批变异行的锚点应当判红」（实际退出码 0，没有点名 merge-data-row 的 ANCHOR 行，打出了合进主工作区的命令）、
#       「变异锚点核：补丁之前就失效的变异行不算这一批弄断的」（实际没有「补丁前就红的 1 处」那一行，核根本没做）、
#       「变异锚点核：自己追加的行锚点不中（只改变异表）」「变异锚点核：自己追加的行锚点不中（连代码一起改）」（实际退 0，没有点名 merge-own-broken-row）、
#       「变异锚点核：补丁只新建一份文件、不碰变异行，应当过」（实际没有「变异锚点核过」那一句）、
#       「变异锚点核：补丁之前就红的 RELEASE 行只因这一批删了它前面一行、行号前移」（实际没有「补丁前就红的 1 处」那一行）、
#       「变异锚点核：带 --skip-apply 的那一趟补丁弄断别批变异行的锚点应当判红」「变异锚点核：带 --skip-apply 的那一趟补丁自己追加的行锚点不中应当判红」
#       「变异锚点核：带 --skip-apply 的那一趟补丁把变异行换成锚点不中的一行应当判红」（实际退 0，没有点名的 ANCHOR 行）、
#       「变异锚点核：新红多过标准输出列的行数时」（实际退 0，日志里没有那 45 处）、
#       「变异锚点核：没留快照的那一趟出路应当写带 MERGE_AND_VERIFY_KEEP_COPY=1 整条重交」（实际退 0，没有出路）
#   MERGE_AND_VERIFY_BREAK=anchor-check-baseline-ignored  变异锚点核不拿补丁之前的树当基线（static_check_report 经 APPLY_WRITER_PATCH_BREAK=static-check-baseline-ignored 照没有基线算），补丁之前就红的行也算这一批带来的
#       「变异锚点核：补丁之前就失效的变异行不算这一批弄断的」（实际退 1，点名了补丁之前就失效的 merge-old-broken-row）、
#       「变异锚点核：补丁之前就红的 RELEASE 行只因这一批删了它前面一行、行号前移」（实际退 1，点名了补丁之前就红的 merge-release-red-row）
#   MERGE_AND_VERIFY_BREAK=anchor-check-release-row-number-kept  变异锚点核的 static_check_report 比基线时 RELEASE 行连变异表行号一起比（经 APPLY_WRITER_PATCH_BREAK=static-check-release-row-number-kept）
#       「变异锚点核：补丁之前就红的 RELEASE 行只因这一批删了它前面一行、行号前移」（实际退 1，补丁前就红的 merge-release-red-row 行号少 1、被当成这一批带来的）
#   MERGE_AND_VERIFY_BREAK=anchor-own-table-not-restored  变异锚点核摆补丁之前的树时不把变异表换回原样（补丁之前的表就是补丁之后的表，这一批自己追加的锚点不中的行补丁前后都红）
#       「变异锚点核：自己追加的行锚点不中（只改变异表）」「变异锚点核：自己追加的行锚点不中（连代码一起改）」（实际退 0，没有点名 merge-own-broken-row）、
#       「变异锚点核：带 --skip-apply 的那一趟补丁自己追加的行锚点不中应当判红」（实际退 0，没有点名 merge-own-broken-row）、
#       「变异锚点核：带 --skip-apply 的那一趟补丁把变异行换成锚点不中的一行应当判红」（实际退 0，补丁之前的表没换回前像，换成的那一行补丁前后都红）
#   MERGE_AND_VERIFY_BREAK=anchor-created-files-kept  变异锚点核摆补丁之前的树时补丁新建的文件不删
#       「变异锚点核：补丁只新建一份文件、不碰变异行，应当过」（实际退 1、没有变异锚点核过那一句：补丁之前的树里已有那份文件，补丁套不上，核没做成）
#   MERGE_AND_VERIFY_BREAK=anchor-mutation-only-skipped  没有 crates.patch、只改变异表的补丁不做变异锚点核
#       「变异锚点核：自己追加的行锚点不中（只改变异表）」「变异锚点核：带 --skip-apply 的那一趟补丁自己追加的行锚点不中应当判红」（实际退 0，没有点名 merge-own-broken-row）、
#       「变异锚点核：补丁之前就红的 RELEASE 行只因这一批删了它前面一行、行号前移」（那一份补丁也只有变异表文件，核没做，没有「补丁前就红的 1 处」那一行）、
#       「变异锚点核：带 --skip-apply 的那一趟补丁把变异行换成锚点不中的一行应当判红」（那一趟的补丁目录也只有变异表文件，核没做，实际退 0）
#   MERGE_AND_VERIFY_BREAK=anchor-skip-apply-unchecked  带 --skip-apply 的那一趟不做变异锚点核
#       「变异锚点核：带 --skip-apply 的那一趟补丁弄断别批变异行的锚点应当判红」「变异锚点核：带 --skip-apply 的那一趟补丁自己追加的行锚点不中应当判红」（实际退 0，没有点名的 ANCHOR 行）、
#       「变异锚点核：带 --skip-apply 的那一趟补丁把变异行换成锚点不中的一行应当判红」「变异锚点核：带 --skip-apply 的那一趟变异表倒不回补丁之前应当记红」（实际退 0，核没做）
#   MERGE_AND_VERIFY_BREAK=anchor-skip-table-not-reversed  带 --skip-apply 的那一趟变异表不倒回补丁之前（补丁之前的表就是补丁之后的表）
#       「变异锚点核：带 --skip-apply 的那一趟补丁自己追加的行锚点不中应当判红」（实际退 0，没有点名 merge-own-broken-row）、
#       「变异锚点核：带 --skip-apply 的那一趟补丁把变异行换成锚点不中的一行应当判红」「变异锚点核：带 --skip-apply 的那一趟变异表倒不回补丁之前应当记红」（实际退 0：没有倒回、也就没有倒不回）
#   MERGE_AND_VERIFY_BREAK=anchor-skip-replacement-not-reversed  带 --skip-apply 的那一趟倒变异表时，mutations-replacements.tsv 的改后行不换回前像
#       「变异锚点核：带 --skip-apply 的那一趟补丁把变异行换成锚点不中的一行应当判红」（实际退 0，补丁之前的表里还是换成的那一行）
#   MERGE_AND_VERIFY_BREAK=anchor-skip-irreversible-passes  带 --skip-apply 的那一趟变异表倒不回补丁之前（追加的行之后被改过）时照放行
#       「变异锚点核：带 --skip-apply 的那一趟变异表倒不回补丁之前应当记红」（实际退 0，没有写变异锚点核没做成）
#   MERGE_AND_VERIFY_BREAK=anchor-skip-no-directories-silent  带 --skip-apply 没给补丁目录的那一趟不打「变异锚点核本次未核」那一行
#       「变异锚点核：带 --skip-apply 没给补丁目录应当写本次未核」（实际输出里没有变异锚点核本次未核）；
#       连带「变异锚点核：带 --skip-apply 没给补丁目录的本次未核应当不记红、写明为什么」（那一行没打，没有摆不出补丁之前的树）
#   MERGE_AND_VERIFY_BREAK=anchor-skip-none-records-red  带 --skip-apply 没给补丁目录的那一趟打了「本次未核」之后也记一项红
#       「变异锚点核：带 --skip-apply 没给补丁目录的本次未核应当不记红、写明为什么」（实际放了标记那一趟退 1、不放标记的那一趟退 3）
#   MERGE_AND_VERIFY_BREAK=anchor-skip-none-reason-dropped  带 --skip-apply 没给补丁目录的那一趟「本次未核」那一行只写「变异锚点核本次未核」、不写为什么
#       「变异锚点核：带 --skip-apply 没给补丁目录的本次未核应当不记红、写明为什么」（实际那一行里没有摆不出补丁之前的树）
#   MERGE_AND_VERIFY_BREAK=anchor-check-log-truncated  变异锚点核红了只打前 20 处新红（日志里也只有这 20 处）
#       「变异锚点核：新红多过标准输出列的行数时」（实际日志里只有 20 处，不是 45 处）
#   MERGE_AND_VERIFY_BREAK=anchor-way-out-delete-dropped  变异锚点核红了的出路不写锚着的代码被整段删掉的改写进 mutations-delete.txt
#       「变异锚点核：补丁弄断别批变异行的锚点应当判红」（实际出路里没有 mutations-delete.txt）
#   MERGE_AND_VERIFY_BREAK=anchor-way-out-keep-copy-dropped  没留快照的那一趟变异锚点核红了，出路不写带 MERGE_AND_VERIFY_KEEP_COPY=1 整条重交
#       「变异锚点核：没留快照的那一趟出路应当写带 MERGE_AND_VERIFY_KEEP_COPY=1 整条重交」（实际出路里没有那一句）
#   MERGE_AND_VERIFY_BREAK=merge-in-dry-run-dropped  合进主工作区那一段不打合入前的 apply-writer-patch.py --dry-run 那一条
#       「合入前的 --dry-run：别的会话开跑之后追加的、锚在这一批改到的文件上的变异行应当拦住」
#       「合入前的 --dry-run：这一批只追加一行变异行、锚着的文件开跑之后被别的会话改了应当拦住」
#       「合入前的 --dry-run：这一批自己追加的行锚在补丁没改到的文件上、开跑之后别的会话改了那份文件（只改变异表）应当拦住」
#       「合入前的 --dry-run：这一批自己追加的行锚在补丁没改到的文件上、开跑之后别的会话改了那份文件（连代码一起改）应当拦住」
#       「合入前的 --dry-run：这一批追加的行点名的测试目标开跑之后被别的会话登记成崩溃枚举用例应当拦住」
#       「合入前的 --dry-run：这一批登记崩溃枚举用例、开跑之后别的会话追加了点名它的变异行应当拦住」
#       「合入前的 --dry-run：同一份文件里不受影响的追加应当放行」（实际合进主工作区那一段里没有那一条）
#       「合入前的 --dry-run 之后的核（window-fingerprint）：没走到」「合入前的 --dry-run 之后的核（window-table-append）：没走到」「合入前的 --dry-run 之后的核（window-unchanged）：没走到」
#       「合入前的 --dry-run：合并补丁带拷贝范围外的文件段（拒算表一行）时应当照常判」（同上，没有 ② 那一条）
#   MERGE_AND_VERIFY_BREAK=merge-in-anchored-rows-before-dropped  合进主工作区那一段不打 ① 记下变异表里锚在这一批改到的文件上的行的那一条
#       「合入前的 --dry-run 之后的核（window-unchanged）：主工作区没变时 ① 记下、③ 比对、④ 核主工作区没变都应当退 0」（实际 ① 退 missing：③ 没有 ① 记下的那一份可比）
#   MERGE_AND_VERIFY_BREAK=merge-in-anchored-rows-after-dropped  合进主工作区那一段不打 ③ 照 ① 重算再比的那一条
#       「合入前的 --dry-run 之后的核（window-table-append）：--dry-run 跑完之后别的会话往变异表追加了锚在这一批改到的文件上的行」（实际 ③ 退 missing）、
#       「合入前的 --dry-run 之后的核（window-unchanged）：主工作区没变时 ① 记下、③ 比对、④ 核主工作区没变都应当退 0」（实际 ③ 退 missing）
#   MERGE_AND_VERIFY_BREAK=merge-in-quick-checks-before-dry-run  ④ 的两条快核排在 ① 之前、不在 ② 之后（② 跑着的几百秒里主工作区变了，合之前没有一条核看它）
#       「合入前的 --dry-run 之后的核（window-fingerprint）：--dry-run 跑着的时候别的会话改了这一批改到的包里的文件」（实际 ② 之后到 ⑤ 之前没有核主工作区没变的那一条）、
#       「合入前的 --dry-run 之后的核（window-unchanged）：主工作区没变时 ① 记下、③ 比对、④ 核主工作区没变都应当退 0」（实际 ④ 核主工作区没变退 missing）
#   MERGE_AND_VERIFY_BREAK=merge-in-dry-run-way-out-dropped  ② 之后不写代价与各结局的出路
#       「合入前的 --dry-run 之后的核（dry-run-way-out-outcomes）：合入前的 --dry-run 那一条的出路应当分开写」（实际那一段是空的）、
#       「合入前的 --dry-run 之后的核（dry-run-way-out-cost）：合入前的 --dry-run 那一条之后应当写代价」（实际那一段是空的）
#   MERGE_AND_VERIFY_BREAK=merge-in-dry-run-outcomes-collapsed  ② 的出路只写点名「变异表有 N 处静态核红」的整批重交那一种，不分「没做成」「判不了」、不是 2 的结局
#       「合入前的 --dry-run 之后的核（dry-run-way-out-outcomes）：合入前的 --dry-run 那一条的出路应当分开写」（实际缺没做成、判不了、被停、撞内存上限、再交同一条 --dry-run、从 ① 重做 这几句）
#   MERGE_AND_VERIFY_BREAK=merge-in-redo-way-out-narrow  ⑤ 退 2 等完与三方合并冲突、核与换之间被改了的出路只写重做核的那几条、再交同一条，不从 ① 重做
#       「合入前的 --dry-run 之后的核（redo-loop）：合的那一条退 2 等完」（实际两句里都没有从 ① 重做到 ⑤）
#   MERGE_AND_VERIFY_BREAK=dry-run-baseline-on-live-root  合入前的 --dry-run 的没打补丁那一趟 static-check 对活的主工作区跑、不对同一刻拷出的补丁之前那一棵跑（经 APPLY_WRITER_PATCH_BREAK=static-check-baseline-on-live-root）
#       「合入前的 --dry-run：两棵树同一刻拷出（dry-transient）：别的会话追加的、锚在这一批改到的文件上的行被这一批弄断了」（实际退 0：这一行在活的主工作区上同样命中 0 次，被当成补丁之前就红的放行）、
#       「合入前的 --dry-run：补丁之前就红的行不算这一批弄断的（dry-skew）：dry-skew-prebroken-row」（实际退 2：活的主工作区上它被修好了，补丁之后那一棵里还红，被当成这一批新弄断的）
#   MERGE_AND_VERIFY_BREAK=dry-run-outside-files-not-copied  合入前的 --dry-run 的两棵临时树不放补丁改到的拷贝范围之外的文件（经 APPLY_WRITER_PATCH_BREAK=static-check-outside-files-not-copied）
#       「合入前的 --dry-run：合并补丁带拷贝范围外的文件段（拒算表一行）时应当照常判」（实际退 2、报静态核没做成：补丁在临时树上套不上）
#   MERGE_AND_VERIFY_BREAK=merge-in-after-skip-apply  带 --skip-apply 的那一趟验绿了也打合进主工作区的命令
#       「先验后合：带 --skip-apply 的那一趟验绿了不该打合进主工作区的命令」
#   MERGE_AND_VERIFY_BREAK=apply-refusal-way-out-unchanged  打补丁核不过（apply 退 2）的出路不写段树那一句
#       「apply 退 2 时应当整批停、退 2，出路写段树那一句」
#   MERGE_AND_VERIFY_BREAK=refused-patch-way-out-unchanged  打补丁核不过的是只含拒算表的补丁目录时，出路也照段树那一句写（指实现员与新段树）
#       「打补丁核不过的是只含拒算表的补丁目录时应当整批停、退 2」（实际出路写以新段树为基线退给实现员重做，没有主 agent 从主工作区此刻重做那一句）
#   MERGE_AND_VERIFY_BREAK=segment-tree-command-narrow  打补丁核不过（apply 退 2）的出路里建新段树的那一条只带 --with-test-inputs
#       「apply 退 2 时应当整批停、退 2，出路写段树那一句」（实际建新段树的那一条不带 --with-gate-tables 与三条 --with）
#   MERGE_AND_VERIFY_BREAK=main-tree-check-dropped  验绿了打合进主工作区那一段时不打核主工作区没变的那一条
#       「先验后合：合进主工作区那一段应当先打核主工作区自开跑以来没变的那一条，主工作区没动时它退 0」（实际没有那一行）；
#       连带「先验后合：同包里别的文件开跑之后改了、加了，核主工作区没变的那一条应当退非 0」「先验后合：依赖关系上的包开跑之后改了，核主工作区没变的那一条应当退非 0」（没有那一条可跑）、
#       「合入前的 --dry-run 之后的核（window-fingerprint）：--dry-run 跑着的时候别的会话改了这一批改到的包里的文件」「合入前的 --dry-run 之后的核（window-unchanged）：主工作区没变时 ① 记下、③ 比对、④ 核主工作区没变都应当退 0」（没有 ④ 核主工作区没变的那一条可跑）
#   MERGE_AND_VERIFY_BREAK=main-tree-fingerprint-touched-files-only  开跑那一刻的指纹只记补丁改到的那几份文件，不记它们所在的包与依赖关系上的包
#       「先验后合：同包里别的文件开跑之后改了、加了，核主工作区没变的那一条应当退非 0」「先验后合：依赖关系上的包开跑之后改了，核主工作区没变的那一条应当退非 0」（实际都退 0）；
#       连带「合入前的 --dry-run 之后的核（window-fingerprint）：--dry-run 跑着的时候别的会话改了这一批改到的包里的文件」（改的是同包里的 tests/case_a.rs，指纹只记补丁改到的 merge_table.txt，实际 ④ 退 0）、
#       连带「先验后合：带 --fingerprint-of 的重跑，头一趟开跑之后主工作区变了应当退 2、一个补丁都不打」（头一趟只记了补丁改到的文件，同包里改的那一份核不到，实际退 0）、
#       「开跑那一刻的指纹：依赖方写 { workspace = true }…」（起点只有补丁改到的 user/src/lib.rs 那一份文件，没有 core 与 user 两个包目录）
#   MERGE_AND_VERIFY_BREAK=fingerprint-dependencies-skipped  开跑那一刻的指纹只记补丁改到的包，不记工作区里依赖它们与被它们依赖的包、不记树根的 Cargo.toml、Cargo.lock、.cargo/
#       「先验后合：依赖关系上的包开跑之后改了，核主工作区没变的那一条应当退非 0」（实际两趟都退 0）、
#       「开跑那一刻的指纹：依赖方写 { workspace = true }…」（起点只有 user，没有它依赖的 core）
#   MERGE_AND_VERIFY_BREAK=fingerprint-mutation-rows-dropped  合进主工作区那一段不打核这一批要换掉、删掉的变异行的那一条
#       「先验后合：这一批要换掉的变异行开跑之后被改了，核变异行的那一条应当退非 0」（实际没有那一条）
#   MERGE_AND_VERIFY_BREAK=fingerprint-of-unchecked  给了 --fingerprint-of 也不核那一趟的指纹
#       「先验后合：带 --fingerprint-of 的重跑，头一趟开跑之后主工作区变了应当退 2、一个补丁都不打」（实际退 0、照常打补丁）
#   MERGE_AND_VERIFY_BREAK=rerun-keeps-no-snapshot  不带 --skip-apply 的那一趟只重跑红的那一条不带 env MERGE_AND_VERIFY_KEEP_COPY=1 与 --fingerprint-of
#       「先验后合：测试那一步红了，只重跑的那一条应当带 env MERGE_AND_VERIFY_KEEP_COPY=1 与 --fingerprint-of <这一趟的日志目录>」；
#       连带「先验后合：验证包那一步红了，只重跑红的应当是不带 --skip-apply 的合入后验证…」（真卡对拍红的那一条 submit 的 -- 之后没有 env MERGE_AND_VERIFY_KEEP_COPY=1）
#   只重跑的命令点名（文件头「只重跑的命令点名」）的开关：
#   MERGE_AND_VERIFY_BREAK=rerun-batch-unnamed  红的测试函数归不到项时，整批再验的那一条不点项（不带 --item、范围写补丁份数、不带 --whole-package-confirmed）
#       「一条测试红时应当退 1、列出那条测试与只重跑它的命令」（整批再验那一条应当点这一趟跑过的 demo:src/lib.rs）；
#       「重跑命令点名：打出的每条只重跑的命令照 unnamed_run_problem 判都点了名」（out1.txt 那一条没点名）
#   MERGE_AND_VERIFY_BREAK=rerun-confirmation-dropped  不点项的那几条（真卡对拍红的只重跑、--skip-apply 核不过而原命令没点项的那一条）submit 不带 --whole-package-confirmed
#       「先验后合：验证包那一步红了，只重跑红的应当是不带 --skip-apply 的合入后验证…」（真卡对拍红的那一条应当带 --whole-package-confirmed）；
#       「重跑命令点名：打出的每条只重跑的命令照 unnamed_run_problem 判都点了名」（真卡对拍红的两条与 --skip-apply 核不过的那一条没点名）
#   MERGE_AND_VERIFY_BREAK=verification-rerun-items-dropped  验证包那一步红了而 ITEM_DONE 里 exit=1 的项都不在这一次点的项里时，只重跑的门禁一项都不点（应当点这一次的全部项）
#       「重跑命令点名：打出的每条只重跑的命令照 unnamed_run_problem 判都点了名」（红的项认不出的那一条 --check crash-case-replay 没带 --item）
#   MERGE_AND_VERIFY_BREAK=keep-run-target  收尾不删这一趟自己建的编译目录（编进树根的 target、跨趟留着）
#       「跑完运行目录里不剩它建的编译目录」（正常、出错、被 TERM 三种结局，实际编译目录还在）；
#       连带「快照：全绿时应当在快照里打补丁、收尾删掉快照与编译目录」与「快照：测试红时也应当删掉快照与编译目录」（编译目录还在）、
#       「这一趟的编译目录里还有进程开着文件：应当不删、报出那个进程、退 1」（不删也不核，退 0）、
#       「收尾时前面一步写满盘（用开关演）编译目录照删」（编译目录还在）
#   MERGE_AND_VERIFY_BREAK=target-under-work-root  一趟一份那一种的编译目录建在 <日志目录>/target（日志目录在工作根下，派到第二台时上百 G 经 NFS 写回本机的盘）
#       「编译目录不在工作根下」（实际在 <工作根>/merge-and-verify/<日期>-<进程号>/target）
#   MERGE_AND_VERIFY_BREAK=cleanup-after-other-steps  EXIT trap 先删快照，最后才删编译目录（前面一步写满盘停住，编译目录就留下了）
#       「收尾时前面一步写满盘（用开关演）编译目录照删」（实际编译目录还在）
#   MERGE_AND_VERIFY_BREAK=finish-in-forked-shell  收尾（EXIT trap 的 finish）不认是不是起这一趟的那个 shell：fork 出来的子 shell 跑了继承来的 EXIT trap 也删快照与编译目录
#       「快照：fork 出来的子 shell 跑了继承来的 EXIT trap 时不该删这一趟的快照与编译目录」（MERGE_AND_VERIFY_SELFTEST_FORKED_FINISH 演：测试那一步的进度进程就删了快照与编译目录，删了快照不止收尾那一行、收尾报编译目录没建）
#   MERGE_AND_VERIFY_BREAK=run-target-open-files-unchecked  删这一趟的编译目录之前不核有没有进程开着里面的文件（run_build_directory.py remove --skip-open-check）
#       「这一趟的编译目录里还有进程开着文件：应当不删、报出那个进程、退 1」（实际删了、退 0）
#   MERGE_AND_VERIFY_BREAK=rocksdb-entry-unchecked  本机起四条 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry
#       「起 cargo 之前 RocksDB 预编库的取值缺了……应当退 1、一条 cargo 都不起」（实际假 cargo 收到了命令）
#   MERGE_AND_VERIFY_BREAK=copy-not-git-initialized  一趟一份那一种拷完快照之后不在快照根 git init（快照不是 git 工作树，读写判定库的脚本在快照上退 2）
#       「快照：快照根应当 git init」（实际留下的快照取不到 git common-dir）；
#       连带「逐项与复用：reuse-three-questions」「逐项与复用：整趟每项一行 ITEM_DONE」「逐项与复用：点名单跑 alpha:tests/beta.rs」「逐项与复用：记红撤绿」
#       「逐项与复用：--list-items --with-keys 每项一行 ITEM_PLAN」
#       「缩崩溃域（来历：argument）…」「缩崩溃域（来历：environment）…」（admission.py test-plan 在快照上退 2，按条复用失灵、测试那一步判红，一条都没记库；验证包的 cargo test 没被起，收到的变量是空的）
#   MERGE_AND_VERIFY_BREAK=test-arguments-added  测试那一步的 cargo test 在 -- 之后多带一个参数（--show-output）
#       「测试那一步的 cargo test 在 -- 之后应当什么都不带」（实际假 cargo 收到的测试命令以 -- --show-output 收尾）；
#       连带逐字比假 cargo 收到的测试命令的各格：「cargo test 应当只起补丁改到的 demo 的库目标」「测试那一步应当经 test-targets-parallel.py 按测试目标并行起」
#       「补丁改到 alpha 的库与 beta 的测试目标 case：…」「补丁改到 alpha 的 src/lib.rs、变异行点名 alpha 的 case_a：…」「check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们」
#       「不变量文档：beta 的测试目标 reads_invariants 读它…」
#       「挑测试目标：两个 bin 经 #[path] 共用一个模块…」「挑测试目标：子目录里的文件只被两个目标经 #[path] 引…」「挑测试目标：子目录里的文件经 mod.rs 被引…」「挑测试目标：子目录里的文件谁都走不到…」
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段写了 path 的 [[test]]…」「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段没写 path 的 [[bin]]…」「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段 path 指到自动认的 src/bin/tool.rs、名字不同的 [[bin]]…」
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里改了一段 [[bin]]…」「挑测试目标：补丁只改 alpha 的 bin 根文件 src/bin/tool.rs…」
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段 path 写成 ./tests/case_a.rs 的 [[test]]…」「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加两段没写 path 的目录形目标…」
#       「挑测试目标：段已写在 Cargo.toml 里、补丁只改目录形根文件…」
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」（多了 -- 之后的参数就对不上）
#   挑测试目标（文件头「挑测试目标」）的开关：
#   MERGE_AND_VERIFY_BREAK=whole-package-tests  改到的包与工作区里依赖它们的包，每个目标都挑（照整包跑）
#       「补丁改到 alpha 的库与 beta 的测试目标 case：…」「补丁改到 alpha 的 src/lib.rs、变异行点名 alpha 的 case_a：…」（实际 alpha 与依赖它的 beta 的目标都起了）；
#       「不变量文档：没有代码读它（注释里提到不算）…」「不变量文档：beta 的测试目标 reads_invariants 读它…」（文档所在的包 alpha 与依赖它的 beta 整包挑）；
#       连带「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」（另起了依赖 demo 的 delta、验证包的库，交了 feature-gated:lib）、
#       「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」（验证包那一条另带 --test setting）、
#       「验证包那一步：跑在第二台上时…」「验证包那一步：--crash-expand-up-to 6 …」「验证包那一步：不给 --crash-expand-up-to 时…」「验证包那一步：环境里已设展开深度变量时…」
#       （那一道多起了一次 feature-gated:lib：起的次数、记下的变量行数对不上）、
#       「验证包那一步：补丁改到产品 GPU 包的真卡对拍 real_card 时…」（产品 GPU 包整包挑，测试那一步起了它的库目标）、
#       「重编核对读 clippy 的 Checking 行：补丁只改产品 GPU 包的真卡对拍 real_card…」（同一起因：测试那一步编了这个包，test.log 里有它的 Compiling 行，这一格要的是测试那一步不跑它）、
#       「挑测试目标：两个 bin 经 #[path] 共用一个模块…」（alpha 整包挑，除了 first、second 两个 bin 还起了别的目标）、
#       「挑测试目标：子目录里的文件只被两个目标经 #[path] 引…」「挑测试目标：子目录里的文件经 mod.rs 被引…」「挑测试目标：子目录里的文件谁都走不到…」（alpha 整包挑，另起了库、case_a 与没引它的目标）、
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段写了 path 的 [[test]]…」「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段没写 path 的 [[bin]]…」「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段 path 指到自动认的 src/bin/tool.rs、名字不同的 [[bin]]…」
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段写了 required-features 的 [[test]]…」「挑测试目标：补丁只在 alpha 的 Cargo.toml 里删掉一段 [[test]]…」
#       （alpha 与依赖它的 beta 整包挑：前三格起的目标不止点名的那几个，后两格本该一条测试命令都没有）、
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里改了一段 [[bin]]…」「挑测试目标：补丁只改 alpha 的 bin 根文件 src/bin/tool.rs…」
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段 path 写成 ./tests/case_a.rs 的 [[test]]…」「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加两段没写 path 的目录形目标…」
#       「挑测试目标：段已写在 Cargo.toml 里、补丁只改目录形根文件…」（alpha 与依赖它的 beta 整包挑，起的目标不止点名的 bin 与目录形那两个）、
#       「先验后合：验证包那一步红了，只重跑红的应当是不带 --skip-apply 的合入后验证…」（整包挑之后验证包的目标不止 setting.rs，只重跑的整命令点的 --item 不止那一项）、
#       「逐项与复用：--list-items 有判不出测试目标的照列落得到的项…」（变异行点到的 alpha 整包挑，列出的项除了 alpha:tests/beta.rs 还有 alpha:src/lib.rs，不是恰好一行）
#   MERGE_AND_VERIFY_BREAK=fallback-runs-whole-workspace  判不出测试目标的不拒，改成每个包的每个目标都挑（照旧全跑）
#       「判不出测试目标：」那九格（没给补丁目录、改 Cargo.lock、改不落在任何包里的路径、改到落不到目标的文件、不变量文档只被 build.rs 读、alpha 的 Cargo.toml 新加 [[test]] 段同时改 [dependencies]、
#       alpha 的 Cargo.toml 只加了一行注释、alpha 的 Cargo.toml 新加与自动认的 case_a 同名的 [[test]]、没写 edition 的 alpha 新加一段 [[test]]：实际退 0、假 cargo 收到各包的测试命令）、
#       「逐项与复用：--list-items 有判不出测试目标的照列落得到的项…」（列项那一趟也不报只点了包 alpha 的那一行变异、实际退 0，列出的项多了 alpha:src/lib.rs）、
#       「必红世界表：补丁改到包里没登记的 .tsv」（没登记的 other-table.tsv 本该判不出、退 3，实际不报、退 0，汇总行之后没有 formal-verification）；
#       连带「先验后合：判不出测试目标（退 3）的那一趟也应当打合进主工作区的命令、写明退 3 那一支」（实际退 0）、
#       「判不出的留给提交时整轮门禁：带 --unresolved-to-commit-gate、点了 --item、有判不出测试目标的改动，应当只跑点名的项…」
#       （判不出的不报，那一趟汇总行里没有留给提交时整轮门禁的那一处）、
#       「判不出的留给提交时整轮门禁：不带 --unresolved-to-commit-gate 时判不出测试目标的照旧退 3…」「判不出的留给提交时整轮门禁：带了 --unresolved-to-commit-gate 而没点 --item 时不生效…」
#       （判不出的不拒、照整包挑：不带那个选项的那一趟与带了选项而没点 --item 的那一趟，本该退 3、测试那一步不跑，实际都退 0、测试那一步照跑）
#   MERGE_AND_VERIFY_BREAK=document-readers-unscanned  不变量文档不查有没有代码读，一律当没有读它的放过
#       「不变量文档：beta 的测试目标 reads_invariants 读它…」「判不出测试目标：不变量文档只被 alpha 的 build.rs 读…」
#   MERGE_AND_VERIFY_BREAK=document-comments-counted  查不变量文档读没读时不去注释（注释里提到也算读）
#       「不变量文档：没有代码读它（注释里提到不算）…」「不变量文档：beta 的测试目标 reads_invariants 读它…」（只提它的 mentions_invariants 也被挑上）
#   MERGE_AND_VERIFY_BREAK=document-refused  不变量文档不查读没读，照落不到测试目标拒、退 3
#       「不变量文档：没有代码读它（注释里提到不算）…」「不变量文档：beta 的测试目标 reads_invariants 读它…」「判不出测试目标：不变量文档只被 alpha 的 build.rs 读…」
#   MERGE_AND_VERIFY_BREAK=path-attribute-ignored  找被引的模块文件时 #[path = "…"] 当没写，`mod <名>;` 只找 <名>.rs 与 <名>/mod.rs
#       「挑测试目标：两个 bin 经 #[path] 共用一个模块…」（补丁只改那个模块里 mod 出来的文件，实际判不出改到哪个测试目标、退 3，假 cargo 一条测试命令都没收到）；
#       连带「挑测试目标：子目录里的文件只被两个目标经 #[path] 引…」（谁都走不到它，退回整目录，另起了 mod 这个子目录的两个目标）
#   MERGE_AND_VERIFY_BREAK=subdirectory-whole-directory-added  tests/<子目录>/ 下的文件不先看 mod / #[path] 走得到它的目标，照整目录那一条落、再加上走得到它的
#       「挑测试目标：子目录里的文件只被两个目标经 #[path] 引…」「挑测试目标：子目录里的文件经 mod.rs 被引…」（实际起了 tests/ 顶层里 mod 或 #[path] 到这个子目录的全部四个目标）；
#       连带「挑测试目标：子目录里的文件谁都走不到…」（照整目录落、不打退回整目录那一句）
#   MERGE_AND_VERIFY_BREAK=subdirectory-fallback-dropped  tests/<子目录>/ 下的文件谁都走不到时不退回整目录那一条，照判不出拒
#       「挑测试目标：子目录里的文件谁都走不到…」（实际判不出、退 3，假 cargo 一条测试命令都没收到）；
#       连带「挑测试目标：段已写在 Cargo.toml 里、补丁只改目录形根文件…」（补丁里与 tests/d/main.rs 同目录的 part.rs 谁也没 mod 它，不退回整目录就落不到目标，实际判不出、退 3）
#   MERGE_AND_VERIFY_BREAK=subdirectory-fallback-note-dropped  退回整目录那一条时不打说明
#       「挑测试目标：子目录里的文件谁都走不到…」（挑的四个目标对，输出里没有「退回整目录」那一句）
#   MERGE_AND_VERIFY_BREAK=registration-tables-refused  补丁改到登记表（REGISTRATION_TABLES）照不落在任何包里的路径拒、退 3
#       「挑测试目标：补丁改到登记表时不落测试目标、不退 3」（实际退 3）；
#       连带「合入前的 --dry-run：这一批登记崩溃枚举用例、开跑之后别的会话追加了点名它的变异行应当拦住」「合入前的 --dry-run：合并补丁带拷贝范围外的文件段（拒算表一行）时应当照常判」（补丁改到登记表，这一趟退 3、不是 0）
#   MERGE_AND_VERIFY_BREAK=formal-tables-refused  补丁改到 verus-pins.tsv 登记的必红世界表当没登记：照包里一份判不出的文件拒、退 3
#       「必红世界表：补丁改到 verus-pins.tsv 登记的必红世界表」「必红世界表：补丁只删一行的那张表不点、不交 formal-verification」（实际退 3，原因写判不出改到哪个测试目标，没有那一条 formal-verification）
#   MERGE_AND_VERIFY_BREAK=formal-tables-by-suffix  包里任何 .tsv 都当必红世界表（不读 verus-pins.tsv 的登记）：不落测试目标、不退 3
#       「必红世界表：补丁改到包里没登记的 .tsv」（实际退 0、报一句必红世界表，没登记的 other-table.tsv 没有被判不出）
#   MERGE_AND_VERIFY_BREAK=formal-items-dropped  改到必红世界表的那一趟验证段第 2 步不打那一条 formal-verification
#       「必红世界表：补丁改到 verus-pins.tsv 登记的必红世界表」（实际退 0、报了一句，汇总行之后没有点那两行的 formal-verification 命令）
#   MERGE_AND_VERIFY_BREAK=formal-items-whole-table  改到必红世界表就点那张表的全部行，不只点这一批新加或改过的行
#       「必红世界表：补丁改到 verus-pins.tsv 登记的必红世界表」（点的是 alpha 那张表的四行，多了没改的 alpha-row-kept 与 alpha-row-removable）、
#       「必红世界表：补丁只删一行的那张表不点、不交 formal-verification」（删了一行的那张表还剩的两行照点，汇总行之后多了一条 formal-verification 命令）
#   MERGE_AND_VERIFY_BREAK=refusal-packages-unnamed  判不出测试目标的出路不写点哪几个包：工作区根的文件、不落在任何包里的路径那两条原因后面不写没有包可点、整包跑的范围由用户来选那半句，出路里不打「用户同意整包跑的，点上面逐条点名的包」那一句
#       「判不出测试目标的出路应当写明点哪几个包」（实际两条原因里没有「没有包可点」，出路里没有那一句）
#   MERGE_AND_VERIFY_BREAK=manifest-sections-refused  包的 Cargo.toml 的改动一律照落不到测试目标拒、退 3（不认只改 [[test]] / [[bin]] 段的）
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段写了 path 的 [[test]]…」「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段没写 path 的 [[bin]]…」
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段写了 required-features 的 [[test]]…」「挑测试目标：补丁只在 alpha 的 Cargo.toml 里删掉一段 [[test]]…」
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段 path 指到自动认的 src/bin/tool.rs、名字不同的 [[bin]]…」
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里改了一段 [[bin]]…」「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段 path 写成 ./tests/case_a.rs 的 [[test]]…」
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加两段没写 path 的目录形目标…」（实际退 3、假 cargo 一条测试命令都没收到）；
#       连带「判不出测试目标：补丁在 alpha 的 Cargo.toml 里新加 [[test]] 段、同时改了 [dependencies]…」「判不出测试目标：补丁只在 alpha 的 Cargo.toml 里加了一行注释…」
#       「判不出测试目标：补丁在 alpha 的 Cargo.toml 里新加一段与自动认的 case_a 同名的 [[test]]…」「判不出测试目标：没写 edition 的 alpha（cargo 按 2015 算）…」
#       （照旧退 3，但原因句退回笼统写法，没有 Cargo.toml 只改 [[test]] / [[bin]] 段才认得出那一段，查原因句的这四格红）
#   MERGE_AND_VERIFY_BREAK=manifest-other-sections-unchecked  包的 Cargo.toml 的改动不核 [[test]] / [[bin]] 之外的键有没有改、也不核有没有段有差别，照落得到放过
#       「判不出测试目标：补丁在 alpha 的 Cargo.toml 里新加 [[test]] 段、同时改了 [dependencies]…」（实际退 0、起了 --test extra）、
#       「判不出测试目标：补丁只在 alpha 的 Cargo.toml 里加了一行注释…」（实际退 0、什么都没挑）
#   MERGE_AND_VERIFY_BREAK=manifest-default-path-ignored  [[test]] / [[bin]] 段没写 path 的当没有根文件，不照 cargo 的缺省路径找
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段没写 path 的 [[bin]]…」（实际落不到 src/bin/tool.rs、退 3）、
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里改了一段 [[bin]]…」（tool 那一段没写 path，同样落不到、退 3）、
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加两段没写 path 的目录形目标…」（d 与 probe 都当没有根文件，落不到、退 3）
#   MERGE_AND_VERIFY_BREAK=manifest-shadowed-targets-unchecked  包的 Cargo.toml 的改动不比新旧两侧的目标集合，自动认的目标被同名段顶掉也照落到新段
#       「判不出测试目标：补丁在 alpha 的 Cargo.toml 里新加一段与自动认的 case_a 同名的 [[test]]…」（实际退 0、起了 --test case_a，跑的是 checks/extra.rs）、
#       「判不出测试目标：没写 edition 的 alpha（cargo 按 2015 算）…」（实际退 0、起了 --test extra，没写 edition 顶掉的 case_a 一句不报）
#   MERGE_AND_VERIFY_BREAK=manifest-targets-compared-by-name  新旧两侧的目标集合按（段，名字）比、不按根文件比：旧侧自动认的目标在新侧没有同名的就算顶掉
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段 path 指到自动认的 src/bin/tool.rs、名字不同的 [[bin]]…」（实际判不出、退 3，原因点名自动认的 tool 被顶掉，假 cargo 一条测试命令都没收到）
#   MERGE_AND_VERIFY_BREAK=manifest-paths-unnormalized  [[test]] / [[bin]] 段写的 path 不归一，原样当根文件（两侧目标集合与目标清单都按写的字符串比）
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段 path 写成 ./tests/case_a.rs 的 [[test]]…」（实际判成自动认的 case_a 被顶掉、退 3，假 cargo 一条测试命令都没收到）
#   MERGE_AND_VERIFY_BREAK=directory-form-default-root-ignored  段没写 path 时不找目录形的 <目录>/<名>/main.rs（交给 rerun-failed-tests.py 的 default_root_paths，目标清单与两侧目标集合同一份）
#       「挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加两段没写 path 的目录形目标…」（实际判成自动认的 d、probe 被顶掉、退 3）、
#       「挑测试目标：段已写在 Cargo.toml 里、补丁只改目录形根文件…」（两份根文件都不在目标清单里，判不出、退 3）
#   MERGE_AND_VERIFY_BREAK=edition-2015-automatic-targets-kept  推自动认的目标不看 edition：没写 edition 的包（cargo 按 2015 算）写了同一种段也照推
#       「判不出测试目标：没写 edition 的 alpha（cargo 按 2015 算）…」（实际退 0、起了 --test extra，case_a 当还在）
#   验证包那一步（文件头「验证包那一步」）的开关，点名的格都以「验证包那一步：」起头：
#   MERGE_AND_VERIFY_BREAK=verification-step-skipped  不跑 验证包那一步
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」
#       「验证包那一步：按块那一路判红时…」「验证包那一步：补丁只改到 alpha、beta 时…」「验证包那一步：补丁改到产品 GPU 包的真卡对拍 real_card 时…」
#       「验证包那一步：那一道退 3 而根不是它的样本根…」「验证包那一步：那一道 --list-items 退 1 时…」「验证包那一步：跑在第二台上时…」
#       「验证包那一步：--crash-expand-up-to 6 …」「验证包那一步：不给 --crash-expand-up-to 时…」「验证包那一步：环境里已设展开深度变量时…」「验证包那一步：缩过域的那一趟红了…」
#       「缩崩溃域（来历：argument）…」「缩崩溃域（来历：environment）…」
#       （那一道一次都没被起、报不出那一步的行）；连带「快照：跑着的时候主工作区里那份源码被改……」（日志里没有按块那一路的那一行）；
#       「崩溃域：不超上限…」「崩溃域：超上限且算得出 n…」「崩溃域：缩域接着跑的那一趟汇总…」「崩溃域：估不出应当缩到小值…」「崩溃域：超上限而算不出 n…」「崩溃域：估不出而有缩域记录时…」
#       「崩溃域：点名的用例不在合表里…」「崩溃域：给了值…」「崩溃域：只数这一趟点名的用例…」「崩溃域：估算库起不来…」（没估也没起，退 0、那一道收到的变量是空的）、
#       「崩溃域：验证包的 setting 是目录式测试目标，读库程序应当点名它的 main.rs 与 steps.rs…」（读库程序没起，假 cargo 一条 run 都没有）、
#       「卡：环境里没有卡时交给那一道的每一次…」「卡：环境里交的卡照传…」（那一道一次都没被起，记不到它收到的变量）、
#       「进度行：子进程跑超过一个间隔时…」（verification-crash-cases 那一项没起，进度行不够）、
#       「先验后合：验证包那一步红了，只重跑红的应当是不带 --skip-apply 的合入后验证…」（那一道没被起，红不起来，两趟都退 0）
#   MERGE_AND_VERIFY_BREAK=whole-verification-cells  有要交的就起那一道的三格整格（--check crash-case-replay,feature-gated-tests,product-gpu-real-card-tests，不带 --item）
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」（实际那一道收到的是三格整格）、
#       「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」「验证包那一步：补丁改到产品 GPU 包的真卡对拍 real_card 时…」（同上）、
#       「验证包那一步：按块那一路判红时…」（出路里只重跑的命令不带 --item crash-case:demo-case）、「验证包那一步：跑在第二台上时…」（报的是三格整格那一次，不是按块那一路）；
#       连带「快照：跑着的时候主工作区里那份源码被改……」（那一格样本里的门禁只打崩溃状态块那一行，三格整格那一次要读的按条复用那一行读不到，判红）；
#       「进度行：子进程跑超过一个间隔时…」（三格整格那一次不按用例起，verification-crash-cases 那一项的进度行不够）、
#       「先验后合：验证包那一步红了，只重跑红的应当是不带 --skip-apply 的合入后验证…」（三格整格那一次的参数里带真卡对拍那一格，只重跑的整命令照真卡对拍红的写，不点 --item singlefs-verification:tests/setting.rs）
#   MERGE_AND_VERIFY_BREAK=crash-cases-whole-cell  崩溃枚举用例那一次不带 --crash-cases-only（门禁照跑整个包的按条复用那一趟）
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」「验证包那一步：按块那一路判红时…」（实际收到与打出的都是 --check crash-case-replay --item …）；
#       连带「重跑命令点名：打出的每条只重跑的命令照 unnamed_run_problem 判都点了名」（打出的 --check crash-case-replay --item … 不带 --crash-cases-only，照起整个验证包算没点名）
#   MERGE_AND_VERIFY_BREAK=feature-gated-unrouted  挂特性的测试文件一样都不交
#       「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」（实际那一道只被起了 --list-items 那一次）
#   MERGE_AND_VERIFY_BREAK=feature-gated-directory-target-unrouted  目录式测试目标的项（feature-gated:<名>/main.rs）按 --test <名>/main 找，对不上 --test <名>，不交
#       「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」（split_pair 那一项没交，--item 只有 lib 与 gpu_pair.rs 两项）
#   MERGE_AND_VERIFY_BREAK=product-gpu-unrouted  产品 GPU 包要特性的目标也当别的包，报本次未跑、不交真卡对拍那一格
#       「验证包那一步：补丁改到产品 GPU 包的真卡对拍 real_card 时…」（实际那一道一次都没被起）；
#       连带「卡：环境里交的卡照传…」（只改到 real_card，那一道一次都没被起）、
#       「先验后合：验证包那一步红了，只重跑红的应当是不带 --skip-apply 的合入后验证…」（只改到 real_card 的那一趟，真卡对拍没被交、那一道没被起，退 0 不是要的 1）
#   MERGE_AND_VERIFY_BREAK=verification-release-dropped  验证包那一条 cargo test 不带 --release
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」（实际收到的验证包测试命令没有 --release）
#   MERGE_AND_VERIFY_BREAK=verification-default-features-kept  验证包那一条 cargo test 不带 --no-default-features（没点装置二进制也把它们全编一遍）
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」（实际收到的验证包测试命令没有 --no-default-features）
#   MERGE_AND_VERIFY_BREAK=verification-step-sample-exit-anywhere  那一道退 3 不看根是不是样本根，一律算过
#       「验证包那一步：那一道退 3 而根不是它的样本根…」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=unknown-option-accepted  认不出的选项也照收、不拒（当没给）
#       「给了认不出的选项 --never-used-option 应当退 2、点名它」（实际照往下验、退 0）；连带「给了 --all-packages（不认的参数）应当退 2、点名它、一样都不做」
#   MERGE_AND_VERIFY_BREAK=rerun-command-wrapped  只重跑验证包那一步红的那几项的出路套 capped.sh 与 run-with-memory-cap.sh <上限>，不经 task-server/task submit
#       「验证包那一步：按块那一路判红时…」（出路里没有 task-server/task submit --、带了 run-with-memory-cap.sh）；
#       连带「验证包那一步：缩过域的那一趟红了…」「缩崩溃域（来历：argument）…」「缩崩溃域（来历：environment）…」（出路里没有 task-server/task submit -- env …）；
#       连带「重跑命令点名：打出的每条只重跑的命令照 unnamed_run_problem 判都点了名」（那几条不经 task-server/task submit，拆不开）
#   MERGE_AND_VERIFY_BREAK=rerun-whole-cells  只重跑的出路打三格整格（不带 --item）
#       「验证包那一步：按块那一路判红时…」（出路里的范围声明写的是三格整格、3 项，没有点名的 crash-case:demo-case，不是 2 项（1 格加点名的 1 项））
#   MERGE_AND_VERIFY_BREAK=verification-step-red-ignored  那一道红了也报过
#       「验证包那一步：按块那一路判红时…」与「验证包那一步：那一道退 3 而根不是它的样本根…」（实际都退 0）；
#       连带「验证包那一步：缩过域的那一趟红了…」「缩崩溃域（来历：argument）…」「缩崩溃域（来历：environment）…」（实际退 0、没有只重跑的出路）、
#       「先验后合：验证包那一步红了，只重跑红的应当是不带 --skip-apply 的合入后验证…」（两趟都退 0、没有只重跑的整命令）
#   MERGE_AND_VERIFY_BREAK=verification-step-failed-lines-ignored  数红的测试函数只认「test <名> ... FAILED」，不认那一道逐个红的测试目标抄下的「FAILED <名>」
#       「验证包那一步：日志里两种写法的红应当一起数、按名去重……」（实际只数出 1 条）
#   MERGE_AND_VERIFY_BREAK=expand-up-to-not-passed  给了 --crash-expand-up-to 不把变量交给那一道（汇总照写缩过域）
#       「验证包那一步：--crash-expand-up-to 6 应当把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 与 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO 都设成 6 交给那一道、汇总写缩过域、不算门禁通过」（实际那一道收到的是 unset）；
#       连带「验证包那一步：环境里已设展开深度变量时…」（环境里的 4 照样到了那一道、对拍读的那个没设：那一道收到 amplification=4 judge=unset）、
#       「缩崩溃域（来历：argument）…」（那一道收到 unset）「缩崩溃域（来历：environment）…」（那一道收到 amplification=6 judge=unset）、
#       「崩溃域：超上限且算得出 n…」「崩溃域：估不出而有缩域记录时…」（算出的 6 没交给那一道）、「崩溃域：给了值…」（给的 6 与环境里的 5 都没交给那一道）、
#       「崩溃域：估不出应当缩到小值…」「崩溃域：超上限而算不出 n…」「崩溃域：点名的用例不在合表里…」（小值没交给那一道）
#   MERGE_AND_VERIFY_BREAK=scaled-run-unlabelled  缩过域的那一趟汇总里不写「缩过域、不算门禁通过」
#       「验证包那一步：--crash-expand-up-to 6 应当把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 与 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO 都设成 6 交给那一道、汇总写缩过域、不算门禁通过」（实际汇总里没有那一句）；
#       连带「验证包那一步：环境里已设展开深度变量时当作给了同一个值」「验证包那一步：缩过域的那一趟红了…」「崩溃域：缩域接着跑的那一趟汇总…」、
#       「崩溃域：估不出应当缩到小值…」「崩溃域：超上限而算不出 n…」「崩溃域：估不出而有缩域记录时…」「崩溃域：点名的用例不在合表里…」（汇总里没有那一句）
#   MERGE_AND_VERIFY_BREAK=rerun-command-unscaled  缩过域的那一趟红了，只重跑的命令不带变量
#       「验证包那一步：缩过域的那一趟红了…」；连带「缩崩溃域（来历：argument）…」「缩崩溃域（来历：environment）…」（只重跑的命令里没有两个变量）
#   MERGE_AND_VERIFY_BREAK=judge-expand-variable-not-passed  缩崩溃域只交 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO、不交 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO（四处：那一道、验证包的 cargo test、算键的请求、只重跑的命令）
#       「验证包那一步：--crash-expand-up-to 6 应当把 … 都设成 6 交给那一道…」「验证包那一步：环境里已设展开深度变量时…」「验证包那一步：缩过域的那一趟红了，只重跑的命令应当带同一个展开深度的两个变量…」
#       「缩崩溃域（来历：argument）…」「缩崩溃域（来历：environment）…」（那一道收到 amplification=6 judge=unset、验证包的 cargo test 同、请求里缺 judge 那一项、重跑命令里缺 judge 那一个）；
#       连带「崩溃域：超上限且算得出 n…」「崩溃域：估不出而有缩域记录时…」「崩溃域：给了值…」「崩溃域：估不出应当缩到小值…」「崩溃域：超上限而算不出 n…」「崩溃域：点名的用例不在合表里…」
#       （算出的、给的、回落的 n 只交了引擎读的那一个：那一道收到的是 amplification=<n> judge=unset，不是一个值）
#   MERGE_AND_VERIFY_BREAK=expand-up-to-unchecked  --crash-expand-up-to 的值与环境里的变量不核是不是非负整数
#       「--crash-expand-up-to 不是非负整数时应当退 2、一样都不做」（参数写 abc 与环境里的变量写 abc 两格，实际照往下验）
#   显卡（文件头「显卡」）的开关，点名的格都以「卡：」起头：
#   MERGE_AND_VERIFY_BREAK=card-handed-to-the-gate  环境里没有卡时，验证包那一步交门禁的那几次也设一张 GPU_CARDS_BY_PRIORITY（这一层替门禁挑卡、把卡串交下去）
#       「卡：环境里没有卡时交给那一道的每一次…」（那几次收到了卡串）
#   MERGE_AND_VERIFY_BREAK=start-card-stripped  验证包那一步交门禁的那几次 env -u 掉环境里的 GPU_CARDS_BY_PRIORITY
#       「卡：环境里交的卡照传…」（那一次收到的是 none）
#   崩溃域上限（文件头「崩溃域上限」）的开关，点名的格都以「崩溃域：」起头：
#   MERGE_AND_VERIFY_BREAK=crash-domain-unbounded  不估崩溃域，没给 --crash-expand-up-to 的照整个域起
#       「崩溃域：不超上限…」（没有放行那一句）、「崩溃域：超上限且算得出 n…」「崩溃域：缩域接着跑的那一趟汇总…」（照整个域起、那一道收到 unset）、
#       「崩溃域：估不出应当缩到小值…」「崩溃域：超上限而算不出 n…」「崩溃域：估不出而有缩域记录时…」「崩溃域：点名的用例不在合表里…」（照整个域起、那一道收到 unset）、「崩溃域：只数这一趟点名的用例…」（没有放行那一句）、
#       「崩溃域：估算库起不来…」（没起它，退 0）、
#       「崩溃域：验证包的 setting 是目录式测试目标，读库程序应当点名它的 main.rs 与 steps.rs…」（没估，读库程序没起，假 cargo 一条 run 都没有）
#   MERGE_AND_VERIFY_BREAK=crash-domain-under-limit-narrowed  不超上限也缩到展开深度 1
#       「崩溃域：不超上限…」（那一道收到 1、汇总写缩过域）；连带「验证包那一步：不给 --crash-expand-up-to 时…」（同上）；
#       连带「验证包那一步：按块那一路判红时…」「先验后合：验证包那一步红了，只重跑红的应当是不带 --skip-apply 的合入后验证…」（缩到 1 之后只重跑的命令多带缩域：env 变量，或 --crash-expand-up-to 1，与那两格要的整命令对不上）
#   MERGE_AND_VERIFY_BREAK=crash-domain-narrow-ignored  算得出 n 也不缩，照整个域接着跑
#       「崩溃域：超上限且算得出 n…」「崩溃域：估不出而有缩域记录时…」（那一道收到 unset）、「崩溃域：缩域接着跑的那一趟汇总…」（汇总没写缩过域）
#   MERGE_AND_VERIFY_BREAK=crash-domain-narrow-note-dropped  算得出 n 缩域接着跑，不打那一行说明
#       「崩溃域：超上限且算得出 n…」「崩溃域：估不出而有缩域记录时…」（标准输出里没有『缩到展开深度 6 接着跑』）
#   MERGE_AND_VERIFY_BREAK=crash-domain-known-depth-replaced  算得出 n 也取小值 CRASH_DOMAIN_FALLBACK_DEPTH
#       「崩溃域：超上限且算得出 n…」「崩溃域：估不出而有缩域记录时…」（那一道收到 5）、「崩溃域：缩域接着跑的那一趟汇总…」（汇总写的是 5）
#   MERGE_AND_VERIFY_BREAK=crash-domain-unknown-passed  估不出、超上限而算不出 n 都当不超上限，照整个域起
#       「崩溃域：估不出应当缩到小值…」「崩溃域：超上限而算不出 n…」「崩溃域：点名的用例不在合表里…」（那一道收到 unset）
#   MERGE_AND_VERIFY_BREAK=crash-domain-fallback-note-dropped  缩到小值接着跑，不打那一行说明
#       「崩溃域：估不出应当缩到小值…」「崩溃域：超上限而算不出 n…」「崩溃域：点名的用例不在合表里…」（标准输出里没有这个值的来历）
#   MERGE_AND_VERIFY_BREAK=crash-domain-fallback-depth-wrong  小值多取一层
#       「崩溃域：估不出应当缩到小值…」「崩溃域：超上限而算不出 n…」「崩溃域：点名的用例不在合表里…」（那一道收到 6）
#   MERGE_AND_VERIFY_BREAK=crash-domain-given-value-overridden  给了值也估，超上限时用算出的 n 盖掉给的值
#       「崩溃域：给了值…」（实际那一道收到记录里算出的 4）
#   MERGE_AND_VERIFY_BREAK=crash-domain-whole-table-item  估的时候照合表整项（放量分项与没点名的用例都算）
#       「崩溃域：只数这一趟点名的用例…」（放量分项没有记录，估不出，实际没有放行那一句）
#   MERGE_AND_VERIFY_BREAK=crash-domain-subject-without-root  估算那一段给 Subject 不带仓根，设点文件只认 tests/<目标>.rs，目录式目标的 main.rs 与子模块文件落空
#       「崩溃域：验证包的 setting 是目录式测试目标，读库程序应当点名它的 main.rs 与 steps.rs…」（读库程序点名的是 tests/setting.rs）
#   MERGE_AND_VERIFY_BREAK=crash-domain-program-failure-ignored  估算的 python 段退非 0 也当放行
#       「崩溃域：估算库起不来…」（实际退 0、那一道起了）
#   上一次尝试留下的快照（文件头「上一次尝试留下的快照」）的开关，点名的格都以「上一次尝试：」起头：
#   MERGE_AND_VERIFY_BREAK=earlier-attempt-not-recorded  开跑时不在日志目录记任务号与尝试号（后一次尝试找不到前一次的日志目录）
#       「上一次尝试：尝试 2 应当删掉…」（尝试 1 的日志目录里没有 task-attempt，快照还在）、「上一次尝试：有进程开着…」（那两棵都没被找到，没有写明不删的那一行、没人开着的那棵也还在）、
#       「上一次尝试：进程放手之后…」（还留着快照）
#   MERGE_AND_VERIFY_BREAK=earlier-attempt-snapshot-kept  找到了上一次尝试的日志目录也不删里面的快照
#       「上一次尝试：尝试 2 应当删掉…」「上一次尝试：有进程开着…」「上一次尝试：进程放手之后…」（实际快照都还在，没有写删了的那一行）
#   MERGE_AND_VERIFY_BREAK=earlier-attempt-task-ignored  找上一次尝试时不比任务号，尝试号更小的日志目录一律删
#       「上一次尝试：别的任务号…」（别的任务号的快照被删了）
#   MERGE_AND_VERIFY_BREAK=earlier-attempt-not-smaller-removed  尝试号相同的别的日志目录也删（比较写成不大于）
#       「上一次尝试：别的任务号…」（同一任务号、尝试号相同的那一棵被删了，输出里有删了上一次尝试留下的快照那一行）、「上一次尝试：有进程开着…」（要被开着的那一棵已经没了）
#   MERGE_AND_VERIFY_BREAK=earlier-attempt-open-files-unchecked  删上一次尝试的快照之前不核有没有进程开着里面的文件
#       「上一次尝试：有进程开着…」（实际被开着的那一棵也删了，没有写明不删的那一行）
#   MERGE_AND_VERIFY_BREAK=earlier-attempt-slot-not-preferred  挑槽时不先挑同一任务更早一次尝试记下的那一槽（只按最近占过的挑）
#       「上一次尝试：撞上限重来的尝试 2 接着用尝试 1 记下的那一槽」（实际占的是最近占过的 slot.retry-new）
#   槽位（文件头「槽位」）的开关，点名的格都以「槽位：」起头（另有一格「上一次尝试：撞上限重来…」）：
#   MERGE_AND_VERIFY_BREAK=slot-table-unchecked  槽位登记里值不是非负整数的那一项当 0 收，不退 1
#       「槽位：槽位登记写坏（slots 不是非负整数）时应当退 1」（实际 slots 当 0、不占槽照一趟一份跑完，退 0）
#   MERGE_AND_VERIFY_BREAK=slot-not-reused  挑槽时不接着用空着的槽，一律新开（到了上限就不占槽）
#       「槽位：同一槽先后两趟，下一趟只重编改过的包」（下一趟新开另一槽，两个包都重编）、
#       「槽位：留着的槽不给别的趟、没空槽时这一趟照一趟一份、放了槽再接着用」（登记一槽，放了槽之后那一趟到了上限、不占槽）、
#       「槽位：源树里改了、删了的文件在槽里照改照删，改过的包必重编」（下一趟占的不是同一槽）、
#       「槽位：槽的编译目录里的产物比改过的源码新、改过的 demo 没有 Compiling 行：应当退 1、点名 demo」（下一趟新开一槽、从头编，退 0）、
#       「上一次尝试：撞上限重来的尝试 2 接着用尝试 1 记下的那一槽」（登记两槽都在，到了上限、不占槽）、
#       「槽位：调低上限之后多出来的槽不占」（三个挑槽进程一个都没占到）、
#       「槽位：拷进槽之前删掉槽里上一趟留下的编译产物」（新开另一槽，留着编译产物的 slot.leftover 没被占、也没被拷）、
#       「槽位：占到槽的那一趟每条 cargo 收到 CARGO_INCREMENTAL=0、编之前删掉这一槽编译目录里的 incremental，不占槽的那一趟不设」
#       「槽位：调用方设了 CARGO_INCREMENTAL 的照它、不盖，这一槽编译目录里的 incremental 不删」（两格都新开另一槽，事先放好 incremental 的 slot.incremental 没被占）
#   MERGE_AND_VERIFY_BREAK=slot-kept-taken  挑槽时留着的槽（kept 在、repo/ 也在）也当空着的占
#       「槽位：留着的槽不给别的趟、没空槽时这一趟照一趟一份、放了槽再接着用」（没空槽的那一趟占了留着的那一槽、删了它的 kept）、
#       「槽位：三槽都留着时槽位汇总里有不占槽那一行与三个留着的槽」（占了其中一槽、删了它的 kept，槽位汇总不写这一趟不占槽）
#   MERGE_AND_VERIFY_BREAK=slot-limit-ignored  挑槽时不看登记的上限，没空槽就新开
#       「槽位：留着的槽不给别的趟、没空槽时这一趟照一趟一份、放了槽再接着用」（登记一槽，没空槽的那一趟新开了另一槽）、
#       「槽位：调低上限之后多出来的槽不占」（后两个挑槽进程各新开一槽）、
#       「槽位：三槽都留着时槽位汇总里有不占槽那一行与三个留着的槽」（新开了第四槽，槽位汇总不写这一趟不占槽）
#   MERGE_AND_VERIFY_BREAK=slot-lock-skipped  挑槽时不加锁（别的趟占着的槽也当空着）
#       「槽位：并发两趟各占一槽」（后起的那一趟占了先起的那一趟正占着的那一槽）、
#       「槽位：调低上限之后多出来的槽不占」（三个挑槽进程都占了 slot.surplus-top）
#   MERGE_AND_VERIFY_BREAK=slot-copy-keeps-old-times  拷进槽之后把槽里每份文件的修改时刻拨回拷之前（内容变了的也留着旧修改时刻）
#       「槽位：同一槽先后两趟，下一趟只重编改过的包」（改过的 demo 不重编，重编核对点名 demo、退 1）、
#       「槽位：源树里改了、删了的文件在槽里照改照删，改过的包必重编」（改过的验证包不重编，重编核对点名它、退 1）
#   MERGE_AND_VERIFY_BREAK=slot-deleted-files-kept  拷进槽不带 --delete（源树里删掉的在槽里留着）
#       「槽位：源树里改了、删了的文件在槽里照改照删，改过的包必重编」（槽里还留着源树里删掉的 notes.txt）
#   MERGE_AND_VERIFY_BREAK=slot-changed-sources-unchecked  重编核对不读拷进槽时写了或删了的 .rs（槽位那一种带 --skip-apply 的就一份都不核）
#       「槽位：同一槽先后两趟，下一趟只重编改过的包」「槽位：源树里改了、删了的文件在槽里照改照删，改过的包必重编」（报重编核对本次未判，没有都重编了那一句）、
#       「槽位：槽的编译目录里的产物比改过的源码新、改过的 demo 没有 Compiling 行：应当退 1、点名 demo」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=slot-target-unbounded  槽的编译目录不量也不清：超上限不删，槽已不在的那几份也不删
#       「槽位：槽的编译目录超上限就清，槽没了的编译目录跟着删」（这一槽的编译目录与 slot.gone 那一份都还在，没有清了那一行）、
#       「槽位：slots 是 0 时清多出来的槽与它的编译目录，留着的槽不动」（slot.idle 删了，它在这一台的编译目录还在）
#   MERGE_AND_VERIFY_BREAK=slot-target-on-shared-root  槽的编译目录建在槽里（<槽>/target，工作根下，两台共享、派到第二台时经 NFS 写回本机的盘）
#       「槽位：占槽、放槽，槽的编译目录在这一台的编译根下」（实际编译目录在 <槽>/target）、
#       「槽位：占到槽的那一趟每条 cargo 收到 CARGO_INCREMENTAL=0、编之前删掉这一槽编译目录里的 incremental，不占槽的那一趟不设」（删的是 <槽>/target 下的，这一台编译根下那一份 debug/incremental 还在）
#   MERGE_AND_VERIFY_BREAK=slot-surplus-taken  挑槽时不截按 used 排的前 slots 槽：已有的槽都可挑，也就没有多出来的槽要清
#       「槽位：调低上限之后多出来的槽不占」（三个挑槽进程各占一槽）、
#       「槽位：slots 是 0 时清多出来的槽与它的编译目录，留着的槽不动」（slot.idle 与它的编译目录都还在）
#   MERGE_AND_VERIFY_BREAK=slot-surplus-left  多出来的槽不清（照旧不挑）
#       「槽位：slots 是 0 时清多出来的槽与它的编译目录，留着的槽不动」（slot.idle 与它的编译目录都还在，没有跳过留着的那一行）
#   MERGE_AND_VERIFY_BREAK=slot-surplus-kept-removed  清多出来的槽时留着的槽（kept 在、repo/ 也在）也删
#       「槽位：slots 是 0 时清多出来的槽与它的编译目录，留着的槽不动」（留着的 slot.kept-back 删了，它的编译目录跟着删，没有跳过它那一行）
#   MERGE_AND_VERIFY_BREAK=slot-build-outputs-kept  拷进槽之前不删槽里快照下的 target/、target-*/、__pycache__/
#       「槽位：拷进槽之前删掉槽里上一趟留下的编译产物」（repo/target/release/tool 与 repo/scripts/__pycache__/x.pyc 都还在）
#   MERGE_AND_VERIFY_BREAK=slot-release-keeps-build-outputs  放槽那一条不带 <槽>/repo/target（收尾那一行、合进主工作区那一段的末一句与槽位汇总都是这一条）
#       「槽位：留着的槽不给别的趟、没空槽时这一趟照一趟一份、放了槽再接着用」（收尾打的放槽那一条没有 <槽>/repo/target）
#   MERGE_AND_VERIFY_BREAK=slot-fallback-unsummarized  收尾不打槽位汇总（这一趟不占槽与留着的槽都不报）
#       「槽位：三槽都留着时槽位汇总里有不占槽那一行与三个留着的槽」（没有槽位汇总那几行）
#   MERGE_AND_VERIFY_BREAK=peer-target-called-caller  第二台那一层给的编译目录（CARGO_TARGET_DIR 落在 MULTI_HOST_RUN_PEER_SCRATCH 下面）也说成调用方给的
#       「槽位：第二台那一层给了编译目录时不占槽，那一行说是第二台那一层给的」（那一行写的是调用方给了编译目录）
#   MERGE_AND_VERIFY_BREAK=slot-incremental-on  占到槽的那一趟不设 CARGO_INCREMENTAL（增量缓存照删）
#       「槽位：占到槽的那一趟每条 cargo 收到 CARGO_INCREMENTAL=0、编之前删掉这一槽编译目录里的 incremental，不占槽的那一趟不设」（占槽那一趟 cargo 收到的是 unset）
#   MERGE_AND_VERIFY_BREAK=slot-incremental-cache-kept  占到槽的那一趟不删这一槽编译目录里已有的 incremental（CARGO_INCREMENTAL=0 照设）
#       「槽位：占到槽的那一趟每条 cargo 收到 CARGO_INCREMENTAL=0、编之前删掉这一槽编译目录里的 incremental，不占槽的那一趟不设」（debug/incremental 还在，没有删了增量编译缓存那一行）
#   MERGE_AND_VERIFY_BREAK=slot-incremental-caller-overridden  调用方设了 CARGO_INCREMENTAL 也盖成 0、照删这一槽的 incremental
#       「槽位：调用方设了 CARGO_INCREMENTAL 的照它、不盖，这一槽编译目录里的 incremental 不删」（cargo 收到的是 0，debug/incremental/x 删了，没有调用方设了的那一行）
#   MERGE_AND_VERIFY_BREAK=slot-work-root-unchecked  没给 MERGE_AND_VERIFY_SLOT_BUILD_ROOT 时不判工作根是不是这一台缺省的那一个（只给另一个 SINGLEFS_WORK_ROOT 也照占、照清）
#       「槽位：登记三槽、只给另一个工作根而不给自证的登记与编译根时不占也不清，原工作根下各槽的编译目录都在」（在另一个工作根下新开一槽，原工作根下 slot.idle 与 slot.kept 的编译目录都删了）、
#       「槽位：登记 0 槽、只给另一个工作根而不给自证的登记与编译根时不清，原工作根下各槽的编译目录都在」（原工作根下 slot.idle 与 slot.kept 的编译目录都删了，没有不是缺省的那一个那一句）、
#       「槽位：连同 SINGLEFS_DATA_ROOT 一起换了工作根时不占也不清、原工作根下各槽的编译目录都在，只换 SINGLEFS_DATA_ROOT 时照常占槽」
#       （连同数据根一起换的两种都在另一个数据根下新开一槽，原工作根下 slot.idle 与 slot.kept 的编译目录都删了）
#   MERGE_AND_VERIFY_BREAK=slot-default-work-root-refused  没给 MERGE_AND_VERIFY_SLOT_BUILD_ROOT 时工作根就是这一台缺省的那一个也判成不是，不占也不清
#       「槽位：不给自证的登记与编译根、工作根就是这一台缺省的那一个时照常占槽」（没占槽，为什么那一句是工作根不是这一台缺省的那一个）、
#       「槽位：连同 SINGLEFS_DATA_ROOT 一起换了工作根时不占也不清、原工作根下各槽的编译目录都在，只换 SINGLEFS_DATA_ROOT 时照常占槽」（只换数据根的那一种没占槽）
#   MERGE_AND_VERIFY_BREAK=slot-default-work-root-follows-data-root  缺省的工作根照 .claude/hooks/lib_work_root.py shared work 取（跟着 SINGLEFS_DATA_ROOT 走，与只读多机配置的编译根不同源）
#       「槽位：连同 SINGLEFS_DATA_ROOT 一起换了工作根时不占也不清、原工作根下各槽的编译目录都在，只换 SINGLEFS_DATA_ROOT 时照常占槽」
#       （连同数据根一起换的两种都在另一个数据根下新开一槽、原工作根下 slot.idle 与 slot.kept 的编译目录都删了；只换数据根的那一种没占槽，为什么那一句是工作根不是这一台缺省的那一个）
#   逐项点名与按条复用（文件头「逐项点名与按条复用」）的开关，点名的格都以「逐项与复用：」起头：
#   MERGE_AND_VERIFY_BREAK=items-ignored  --item 点了也照整趟跑（全部项）
#       「逐项与复用：点名单跑 alpha:tests/beta.rs」（实际假 cargo 收到的测试命令不带 --test beta、alpha 的库也跑了）；
#       连带「判不出的留给提交时整轮门禁：带 --unresolved-to-commit-gate、点了 --item、有判不出测试目标的改动，应当只跑点名的项…」「判不出的留给提交时整轮门禁：带 --unresolved-to-commit-gate、点了项的那一趟测试红了应当退 1…」
#       （点名的项被忽略，判不出测试目标的照旧整趟拒：实际退 3、假 cargo 一条测试都没收到，汇总行是测试那一步没跑的那一句，没有只重跑红的命令）
#   MERGE_AND_VERIFY_BREAK=item-unknown-accepted  --item 写错的项悄悄丢掉、不退 2
#       「逐项与复用：--item 写错的项应当退 2」（实际一项都没点，照整趟跑、退 0）
#   MERGE_AND_VERIFY_BREAK=reuse-skips-dropped  键相同的测试函数不经 --skip 跳过（照跑，计划照算）
#       「逐项与复用：reuse-three-questions」（第二趟、闭包外改动那一趟复用的那几条都现判了，same-inputs 与 unrelated-change 两问 red）
#   MERGE_AND_VERIFY_BREAK=red-not-recorded  判红的测试函数不交 verdict-record --red-names（判绿的照记）
#       「逐项与复用：记红撤绿」（上一趟判红的 uses_helper 下一趟照键复用了，没现判）
#   MERGE_AND_VERIFY_BREAK=plan-failure-ignored  按条复用失灵（plan-failure 退 1）不判红，照旧全部现判跑完当绿
#       「逐项与复用：按条复用失灵（test-plan 退 1）应当判红」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=plan-unavailable-run-anyway  test-plan 判 prune-server 不可用（plan-failure 退 75）时照全部现判起 cargo、不退 75
#       「逐项与复用：prune-server 不可用（test-plan 退 75）应当退 75、测试那一步不起 cargo」（实际退 0、假 cargo 收到测试命令）
#   MERGE_AND_VERIFY_BREAK=item-done-unreported  测试那一步收尾不打 ITEM_DONE
#       「逐项与复用：整趟每项一行 ITEM_DONE」与「逐项与复用：点名单跑 alpha:tests/beta.rs」（找不到 ITEM_DONE 行）
#   MERGE_AND_VERIFY_BREAK=item-plan-reusable-zero  --list-items --with-keys 的 ITEM_PLAN 里 reusable 一律 0（键相同的测试函数不数）
#       「逐项与复用：--list-items --with-keys 每项一行 ITEM_PLAN」（记过库之后 alpha:src/lib.rs 的 reusable 该是 2，实际 0）
#   MERGE_AND_VERIFY_BREAK=list-items-unpatched  --list-items 不拷临时快照、不打补丁，照要验的树此刻的样子挑（补丁新建的测试目标列不出，变异行点名它的整条退 3）
#       「逐项与复用：--list-items 应当在打上这一批补丁的临时快照上列」（实际退 3、没有 alpha:tests/gamma.rs 那一行）
#   MERGE_AND_VERIFY_BREAK=list-items-refusal-drops-items  --list-items 有判不出测试目标的就一项都不列（落得到的也不列，只打判不出的、退 3）
#       「逐项与复用：--list-items 有判不出测试目标的照列落得到的项」（实际没有 alpha:tests/beta.rs 那一行）（推的，没复跑）
#   判不出的留给提交时整轮门禁（文件头「判不出的留给提交时整轮门禁」）的开关，点名的格都以「判不出的留给提交时整轮门禁：」起头：
#   MERGE_AND_VERIFY_BREAK=unresolved-option-ignored  带了 --unresolved-to-commit-gate 也当没带：判不出测试目标的照旧不跑测试那一步、退 3
#       「判不出的留给提交时整轮门禁：带 --unresolved-to-commit-gate、点了 --item、有判不出测试目标的改动」（实际退 3、假 cargo 一条测试都没收到）、
#       「判不出的留给提交时整轮门禁：带 --unresolved-to-commit-gate、点了项的那一趟测试红了」（实际退 3、测试没跑，没有只重跑的命令）
#   MERGE_AND_VERIFY_BREAK=unresolved-without-option  没带 --unresolved-to-commit-gate 也照带了算：点了 --item 而判不出测试目标的照点名的项跑、退 0
#       「判不出的留给提交时整轮门禁：不带 --unresolved-to-commit-gate 时判不出测试目标的照旧退 3」（实际退 0、假 cargo 收到 alpha 的库那一条测试）
#   MERGE_AND_VERIFY_BREAK=unresolved-without-item  带了 --unresolved-to-commit-gate 而没点 --item 也生效：判不出的留给提交时整轮门禁、落得到的项全跑
#       「判不出的留给提交时整轮门禁：带了 --unresolved-to-commit-gate 而没点 --item 时不生效」（实际退 0、假 cargo 收到测试命令）
#   MERGE_AND_VERIFY_BREAK=unresolved-lines-dropped  判不出的几处不逐条打「留给提交时整轮门禁」（点名的项照跑、汇总照写）
#       「判不出的留给提交时整轮门禁：带 --unresolved-to-commit-gate、点了 --item、有判不出测试目标的改动」（实际「留给提交时整轮门禁：」那几行一条都没有）
#   MERGE_AND_VERIFY_BREAK=unresolved-summary-dropped  汇总行不写「有 N 处留给提交时整轮门禁」
#       「判不出的留给提交时整轮门禁：带 --unresolved-to-commit-gate、点了 --item、有判不出测试目标的改动」（实际汇总行里没有那一句）
#   MERGE_AND_VERIFY_BREAK=unresolved-option-dropped-from-rerun  只重跑红的那几项的命令不带 --unresolved-to-commit-gate
#       「判不出的留给提交时整轮门禁：带 --unresolved-to-commit-gate、点了项的那一趟测试红了」（实际只重跑的命令里没有这个选项）
#   进度行（文件头「进度行」）的开关：
#   MERGE_AND_VERIFY_BREAK=progress-lines-dropped  run_step 不起打进度的后台进程
#       「进度行：子进程跑超过一个间隔时标准输出应当有进度行」（间隔缩到 1 秒、假 cargo 测试与假的那一道各跑 3 秒，实际 test-1、test-2、verification-crash-cases 都一行进度也没有）
#   本库 scripts/gate/lib_rebuilt_packages.py 的 REBUILT_PACKAGES_BREAK=any-compiling-line 也让「编译目录里的产物比源码新……」那一格红（另一个包有 Compiling 行就当 demo 也重编了）；
#   它的 REBUILT_PACKAGES_BREAK=absolute-dep-info-only 让「头一次编、补丁改过的 demo 有 Compiling 行……」「编过之后 --skip-apply 再验同一批补丁……」
#     「编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行……」三格红（假 cargo 写的 dep-info 是相对路径，只拿绝对路径比就一个都对不上，
#     三格都报「重编核对本次未判」、不报都重编了也不点名 demo；拆分提案第一百零八节 ③）；同理让「槽位：同一槽先后两趟…」「槽位：源树里改了、删了的文件…」
#     「槽位：槽的编译目录里的产物比改过的源码新…」三格红（前两格报本次未判，后一格退 0）；
set -uo pipefail
# 弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道：命令行第一个词是 --selftest，或祖先里有一个的命令行里带 --selftest（自证起的子进程）才放行；
# 否则下面这些变量有设了的，点名它、说它只在自证里认，退 2
#   MERGE_AND_VERIFY_PROGRESS_INTERVAL 也只在自证里认：设了就把进度行的间隔缩到这么多秒（文件头「进度行」）
#   MERGE_AND_VERIFY_SELFTEST_FULL_DISK 也只在自证里认：设了，EXIT trap 删完编译目录之后、删快照之前演一步写满盘、整条收尾停在那里（「收尾时前面一步写满盘」那一格用）
#   MERGE_AND_VERIFY_SELFTEST_FORKED_FINISH 也只在自证里认：设了，run_step 放后台的进度进程起头就调一次 finish，演 fork 窗口里子 shell 跑了继承来的 EXIT trap 的那一种
#     （文件头「快照」末尾那一段；「快照：fork 出来的子 shell 跑了继承来的 EXIT trap……」那一格用）
#   MERGE_AND_VERIFY_SLOT_TABLE、MERGE_AND_VERIFY_SLOT_BUILD_ROOT 也只在自证里认：换掉槽位登记、这一台的编译根（文件头「槽位」；平常槽数与上限只从登记读、编译根只从多机配置读）
#   MERGE_AND_VERIFY_STATIC_CHECK 也只在自证里认：换掉变异锚点核里 static-check 的命令（文件头「先验后合」变异锚点核；平常只跑 scripts/mutation/crates-mutation-rows.py static-check）
BREAK_VARIABLES=(MERGE_AND_VERIFY_BREAK MERGE_AND_VERIFY_SELFTEST_FULL_DISK MERGE_AND_VERIFY_PROGRESS_INTERVAL MERGE_AND_VERIFY_SELFTEST_FORKED_FINISH
  MERGE_AND_VERIFY_SLOT_TABLE MERGE_AND_VERIFY_SLOT_BUILD_ROOT MERGE_AND_VERIFY_STATIC_CHECK)
inside_selftest() { # 往上数这个进程的祖先，有一个的命令行里带 --selftest 就退 0
  local pid="$PPID" stat words word fields
  while ((pid > 1)); do
    mapfile -d '' -t words 2> /dev/null < "/proc/$pid/cmdline" || return 1
    for word in "${words[@]}"; do [[ "$word" == --selftest ]] && return 0; done
    IFS= read -r stat 2> /dev/null < "/proc/$pid/stat" || return 1
    read -ra fields <<< "${stat##*) }"
    pid="${fields[1]:-0}"
  done
  return 1
}
settle_break_switches() { # settle_break_switches <命令行的词…>
  local name setting=()
  for name in "${BREAK_VARIABLES[@]}"; do [[ -n "${!name:-}" ]] && setting+=("$name"); done
  ((${#setting[@]} == 0)) && return 0
  [[ "${1:-}" == --selftest ]] && return 0
  inside_selftest && return 0
  echo "  ✗ ${setting[*]} 是只给自证用的变量（弄坏开关、缩短进度间隔这类），只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做" >&2
  echo "     → 怎么办：unset ${setting[*]} 再来；要看开关红在哪一项，跑 ${setting[0]}=<项> bash scripts/gate/merge-and-verify.sh --selftest" >&2
  exit 2
}
settle_break_switches "$@"
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
source "$here/../hosts/heavy-threads.sh"
source "$here/../hosts/run-build-directory.sh"   # 只用它的 run_directory_owner_record：留着快照的那一趟在日志目录写归属记录（文件头「快照」）
# clip_text <字节数>：标准输入每行取前 <字节数> 个字节，再丢掉被切在中间的半个字符，输出总是合法的 UTF-8（GNU 的 cut -c 与 cut -b 都按字节切，三个字节的汉字会被切断；给人看的失败行与日志摘录用它截）
# 函数库（文件头「代码分块」）：只定义函数；在 here 算出来之后、各步开跑之前 source
source "$here/merge-and-verify-snapshot-lib.sh"
source "$here/merge-and-verify-steps-lib.sh"
source "$here/merge-and-verify-items-lib.sh"
source "$here/merge-and-verify-selftest-lib.sh"
clip_text() { cut -b "1-$1" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null; }
APPLY="${MERGE_AND_VERIFY_APPLY:-$here/apply-writer-patch.py}"
TWO_LOGS="${MERGE_AND_VERIFY_TWO_LOGS:-$here/../hosts/two-logs.sh}"   # 每一步的日志出两份（run_step）
TEST_TARGETS_PARALLEL="$here/test-targets-parallel.py"   # 测试那一步按测试目标并行起（文件头 test 一段）
STAMP="$here/../hosts/source-stamp.py"          # 自证里写按 mtime 判新不新鲜的假 cargo（--write-selftest-cargo）
REBUILT="$here/lib_rebuilt_packages.py"          # 改过的包重编了没有（文件头）
APPLY_WRITER_PATCH_LIBRARY="$here/apply-writer-patch.py"   # 变异锚点核载入它取 static_check_report（文件头「先验后合」变异锚点核）；$APPLY 自证里换成假的，这一份不换
CHECK_PATCH_DIRS=()                              # 算改过的包用的补丁目录（绝对路径；--skip-apply 时也留着）
RUN_BUILD_DIRECTORY="$here/../hosts/run_build_directory.py"   # 删这一趟自己建的编译目录（文件头「这一趟的编译目录」）、槽的编译目录超上限与槽已不在时删（文件头「槽位」）
# 槽位（文件头「槽位」）：登记、槽那一层与这一台编译根下的那一层、日志目录里记的两样；MERGE_AND_VERIFY_SLOT_TABLE 只给自证（settle_break_switches 拒平常设了的）
SLOT_TABLE="${MERGE_AND_VERIFY_SLOT_TABLE:-$here/merge-and-verify-slots.conf}"
SLOT_LIMIT=""; SLOT_TARGET_LIMIT_GIB=""           # read_slot_table 填：两台合计至多几槽、一槽的编译目录在一台上至多占多少 GiB
SLOT_ROOT_NAME="merge-and-verify"                  # 槽放在 <工作根>/merge-and-verify/ 下一层，与默认的日志目录同一层
SLOT_BUILD_DIRECTORY_NAME="merge-and-verify-slots" # 这一台编译根下放各槽编译目录的那一层
SLOT_RECORD_NAME="slot"                            # 日志目录里：这一趟占的槽（同一任务的下一次尝试照它先挑这一槽）
SLOT_CHANGED_SOURCES_NAME="slot-changed-sources"   # 日志目录里：拷进槽时写了或删了的 .rs（重编核对的输入），同名加 .itemized 是 rsync 逐个列的那一份
SLOT_HOLDER_SECONDS=120                            # 等挑槽的进程答一行至多这么多秒（挑槽那一段排他，别的趟挑完就放）
SLOT_DIRECTORY=""; SLOT_TARGET=""; SLOT_FALLBACK_REASON=""   # choose_slot 填：占着的槽、它在这一台的编译目录；没占到槽的为什么
SLOT_ROOT_DIRECTORY=""                             # choose_slot 填：取到工作根时槽那一层（收尾的槽位汇总从这里列留着的槽）
SLOT_HOLDER_PID=""; SLOT_HOLDER_FD=""              # take_slot 填：占着锁的那个 python 进程、读它答话的那一端
SNAPSHOT_RELEASE_TEXT=""                           # 槽位那一种合进主工作区之后怎么放槽（merge_in_commands 的末一句）；一趟一份那一种空着，照「删这一趟留下的快照」
VERIFICATION_STAGE_PATH=".claude/gate.d/crash-point-replay.sh"   # 验证包那一步起的门禁在树里的位置（文件头「验证包那一步」）
# 打补丁核不过时建新段树的那一条（文件头「先验后合」；与 .claude/main-agent.md「写与验分两段」写段第 2 条同一条）
SEGMENT_TREE_COMMAND='python3 scripts/hosts/minimal-copy.py <仓根> <新段树> --with-test-inputs --with-gate-tables --with scripts=<为什么> --with .claude/scripts=<为什么> --with .claude/singlefs-ai-sop=<为什么>'
MAIN_TREE_FINGERPRINT_NAME="main-tree-at-start.sha256"   # 开跑那一刻的指纹（文件头「先验后合」），放日志目录
MAIN_TREE_ROOTS_NAME="main-tree-at-start.roots"          # 指纹的起点：包目录与单份文件，一行一个
MAIN_TREE_MUTATION_ROWS_NAME="main-tree-at-start.mutation-rows"   # 这一批要换掉、删掉的变异行开跑那一刻的原样，按字节排序（文件头「开跑那一刻的指纹」）
ANCHOR_CHECK_SHOWN_LINES=40                       # 变异锚点核红了标准输出列日志的前这么多行，全部在 <日志目录>/anchor-check.log（文件头「先验后合」变异锚点核）
MERGE_BASELINE_NAME="merge-baseline"; MERGE_BASELINE_PATHS_NAME="merge-baseline.paths"   # 打补丁之前补丁动到的文件在快照里的原样与动到的路径（文件头「先验后合」），放日志目录
MERGE_PATCH_NAME="merge-patch"                    # 合进主工作区的那一份补丁目录（文件头「先验后合」），放日志目录
ANCHOR_CHECK_SAMPLE_MARK=".anchor-check-sample"   # 自证里变异锚点核只在带这个标记的样本树上做（文件头「先验后合」变异锚点核），别的格的补丁不受影响
FINGERPRINT_OF=""                                 # --fingerprint-of <日志目录>（文件头「开跑那一刻的指纹」）
# 只重跑的命令里不点项的那几条（文件头「只重跑的命令点名」）：submit 与 -- 之间带 --whole-package-confirmed 与这个占位，命令后面接这一句出路
WHOLE_PACKAGE_CONFIRMATION_PLACEHOLDER="<整包确认文件>"
WHOLE_PACKAGE_CONFIRMATION_NOTE="（这一条不点项：产品 GPU 包的真卡对拍那一格点了 --item 就不起，或原命令就没点项；交之前照 .claude/skills/merge-verify/SKILL.md「写与验分两段」验证段第 1 步「退 3」那一支问用户，照 .claude/agent-common.md「日常开发的测试不跑整个包」确认三遍写成文件，换掉 $WHOLE_PACKAGE_CONFIRMATION_PLACEHOLDER）"
LIST_ITEMS_SCRATCH=""                             # --list-items 建的临时目录（临时快照与计划都在它下面），退出时删（文件头「逐项点名与按条复用」）
# 登记表（文件头「挑测试目标」「开跑那一刻的指纹」）：崩溃枚举用例、放量分项与拒算测试的登记行跟着补丁目录的 crates.patch 带过来；挑测试目标与记指纹的两段 python 从这个变量读
REFUSED_TESTS_TABLE=.claude/gate.d/refused-tests.tsv   # 拒算表：主 agent 判过的拒算行做成一份只含它的补丁目录并进这一批（.claude/main-agent.md「合入后验证之后判新出现的拒算」）
REGISTRATION_TABLES=(.claude/gate.d/stage-inputs.tsv .claude/gate.d/crash-point-replay-items.tsv "$REFUSED_TESTS_TABLE" scripts/crash/crash-amplification.sh)
MERGE_AND_VERIFY_REGISTRATION_TABLE_PATHS="$(printf '%s\n' "${REGISTRATION_TABLES[@]}")"; export MERGE_AND_VERIFY_REGISTRATION_TABLE_PATHS
VERIFICATION_STAGE=""; VERIFICATION_RERUN_STAGE=""   # 建好快照之后填：起的是快照里那一份，出路那条命令写主工作区那一份（MERGE_AND_VERIFY_VERIFICATION_STAGE 换掉两样，只给自证）
SOURCE_TREE=""                                    # 要验的树（快照从它拷；出路那条命令交给门禁的根），参数解析之后填
# 验证包那一步点的三格与只走按块那一路的选项（文件头「验证包那一步」）；WHOLE_VERIFICATION_CELLS 只给弄坏开关 whole-verification-cells 与 rerun-whole-cells 用
CRASH_CASE_CELL="crash-case-replay"; FEATURE_GATED_CELL="feature-gated-tests"; PRODUCT_GPU_CELL="product-gpu-real-card-tests"
CRASH_CASES_ONLY_OPTION="--crash-cases-only"
WHOLE_VERIFICATION_CELLS="$CRASH_CASE_CELL,$FEATURE_GATED_CELL,$PRODUCT_GPU_CELL"
VERIFICATION_STATE=""; VERIFICATION_SCALED=""     # verification_step 填：汇总行里的一句；缩过域的那一趟置 1
AMPLIFICATION_LIBRARY="$here/../crash/crash-amplification-lib.sh"   # 崩溃域上限取放量家目录、多机配置、GPU 开关与 bindgen 参数（文件头「崩溃域上限」）
CRASH_EXPAND_UP_TO=""                             # 缩崩溃域的展开深度上限（--crash-expand-up-to，或环境里已设的变量）；空就是整个域
CRASH_EXPAND_UP_TO_VARIABLE=SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO   # 放量与崩溃枚举用例的引擎读的变量名（值只往小缩）；调用方环境里认它，汇总里点名它
CRASH_JUDGE_EXPAND_UP_TO_VARIABLE=SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO     # 验证包里直接调判器的对拍读的变量名；缩域时与上面那个设成同一个值
CRASH_EXPAND_ASSIGNMENTS=()                       # crash_expand_assignments 填：缩域要交出去的「变量=值」，两个都在；没给值就是空
CRASH_DOMAIN_FALLBACK_DEPTH=5                     # 崩溃域估不出、或超上限又算不出 n 时缩到的展开深度（文件头「崩溃域上限」）；合入后验证缩域跑时固定取这个深度，等同给 --crash-expand-up-to 5
# 挑测试目标（文件头「挑测试目标」），select_test_targets 填
ALL_ITEM_LINES=()            # 挑出的项，一项一行「项<TAB>包<TAB>挑目标的参数」
ITEM_REASONS=()              # 一项一行「项<TAB>为什么挑上」
REQUIRED_FEATURE_TARGETS=()  # 要特性的目标，一行「包<TAB>目标名<TAB>特性<TAB>为什么」
TARGET_REFUSALS=()           # 判不出测试目标的原因；有一条就不跑测试那一步与验证包那一步、整条退 3
LEFT_TO_COMMIT=()            # 带了 UNRESOLVED_OPTION 又点了 --item 的真跑那一趟：判不出测试目标的原因改放这里（不挡测试那一步与验证包那一步，留给提交时的整轮门禁；文件头「判不出的留给提交时整轮门禁」）
LEAVE_UNRESOLVED_TO_COMMIT=0; UNRESOLVED_OPTION=--unresolved-to-commit-gate   # 命令行上带没带这个选项、它的写法
TARGET_NOTES=(); FORMAL_ITEMS=()   # 报一句的：变异行的筛选词、补丁删掉的目标、登记表；必红世界表交门禁 formal-verification 点的项，一项一行「表路径<TAB>项」（merge_in_commands 打验证段第 2 步那一条）
TARGET_PROBLEM=""            # 读不了补丁、成员清单或目标清单的原因
# 逐项点名与按条复用（文件头「逐项点名与按条复用」）
RERUN_FAILED_TESTS="$here/rerun-failed-tests.py"   # 项的清单与「日志里一段归哪一项」（它文件头「项的写法」），经 importlib 调
ADMISSION="$here/admission.py"                     # test-functions-request、test-plan、plan-skip-names、plan-green-names、verdict-record、plan-failure
LIBTEST_RESULT_LINES="$here/lib_libtest_result_lines.py"   # recordable：逐条认结局
LIST_ITEMS=0; LIST_WITH_KEYS=0; NAMED_ITEMS=()      # --list-items、--with-keys、--item 点名的项
RUN_ITEMS=()                                        # collect_test_items 填：这一趟测试那一步跑的项，一项一行「项<TAB>包<TAB>挑目标的参数」
TESTED_PACKAGES=()                                  # select_test_targets 填：挑出的项所在、包目录下有测试属性的包（只给它们起 test-plan）
TEST_FUNCTIONS_KIND=""; TEST_PLAN_ARGUMENTS=()      # plan_test_functions 填：验证包之外的判定种类与交给 test-plan 的参数（admission.py test-functions-request merge-and-verify.sh）
VERIFICATION_FUNCTIONS_KIND=""; VERIFICATION_PLAN_ARGUMENTS=()   # plan_test_functions 填：验证包的（test-functions-request crash-point-replay.sh，另加 --exclude-registered）
TESTS_REFUSED=0                                     # 判不出测试目标，测试那一步与验证包那一步没跑（文件头「退出码」3）
PLANNED_PACKAGES=(); FAILED_PLAN_PACKAGES=()        # 计划算成了的包、按条复用失灵的包
PLAN_UNAVAILABLE=0; PLAN_RED=0                      # prune-server 不可用、按条复用失灵（测试这一步判红）
PLAN_JUDGE=0; PLAN_REUSE=0; PLAN_REFUSED=0          # 跑到的项里计划的条数（成功句）
CHECK_SH="${MERGE_AND_VERIFY_CHECK_SH:-$root/.claude/singlefs-ai-sop/scripts/check.sh}"
VERIFICATION_PACKAGE=singlefs-verification   # 验证包（.claude/rules/verification.md）：它的目标带 --release 跑、按门禁 crash-point-replay 的种类复用，登记的崩溃枚举用例归 verification_step
# check.sh 里 clippy 按特性组合循环的三行结构（不是 cargo 命令本身）：每行在它里面恰好出现一次，这里的 clippy 一步照它们对齐
# （组合 default 不带特性参数；登记文件那一行是 check.sh 认登记的地方，见文件头 clippy 一段）。
# fmt、clippy、build、test 四条 cargo 命令的写法不在这里抄第二份：开跑时从 check.sh 现读（read_cargo_command，文件头 clippy 一段之后的「cargo 命令现读」）。
CHECK_SH_STRUCTURE_LINES=(
  'CLIPPY_COMBINATIONS_FILE="$ROOT/$CLIPPY_FEATURE_COMBINATIONS_FILE"'
  'for clippy_combination in "${CLIPPY_FEATURE_COMBINATIONS[@]}"; do'
  '[[ "$clippy_combination" == default ]] || read -r -a clippy_feature_arguments <<< "$clippy_combination"'
)
# check.sh 的 clippy 那一行里认得的两个占位词（别的带 $、引号、括号的词一律不认）：特性参数、编码纪律 lint 表
CLIPPY_FEATURES_PLACEHOLDER='${clippy_feature_arguments[@]+"${clippy_feature_arguments[@]}"}'
CLIPPY_LINTS_PLACEHOLDER='"${CODE_DISCIPLINE_LINTS[@]}"'
# check.sh 的 test 那一行里不许带的收窄目标的标志：这里的测试步自己按包、按测试目标挑，check.sh 那一行收窄了就不是同一批用例
TEST_NARROWING_FLAGS=(--lib --bins --bin --tests --test --examples --example --benches --bench --doc --package --exclude)
# read_cargo_command 填 CARGO_COMMAND_WORDS（check.sh 那一行里子命令之后的词）；read_check_sh 分到：
CARGO_FMT_WORDS=(); CARGO_CLIPPY_WORDS=(); CARGO_BUILD_WORDS=(); CARGO_TEST_FLAGS=(); CARGO_COMMAND_WORDS=()   # CARGO_TEST_FLAGS：test 那一行除 --all / --workspace 之外带的标志
# check.sh 认的特性组合登记放在项目根下哪里：上游 lib.sh 的 CLIPPY_FEATURE_COMBINATIONS_FILE（check.sh source 的那一份），开跑时现读，不在这里另抄
CHECK_SH_LIBRARY="$root/.claude/singlefs-ai-sop/scripts/lib.sh"
CLIPPY_COMBINATIONS=(); CLIPPY_REGISTRATION=""   # read_check_sh 填：check.sh 的缺省特性组合与登记文件的相对路径

die() { echo "  ✗ $1"; echo "  → 怎么办：$2"; exit "${3:-2}"; }

# 进度行（文件头「进度行」）：间隔秒数；MERGE_AND_VERIFY_PROGRESS_INTERVAL 只给自证（settle_break_switches 已拒平常设了的）
PROGRESS_INTERVAL=300; PROGRESS_TICKER_PID=""
if [[ -n "${MERGE_AND_VERIFY_PROGRESS_INTERVAL:-}" ]]; then
  [[ "$MERGE_AND_VERIFY_PROGRESS_INTERVAL" =~ ^[1-9][0-9]*$ ]] || die "MERGE_AND_VERIFY_PROGRESS_INTERVAL 的值「$MERGE_AND_VERIFY_PROGRESS_INTERVAL」不是正整数（秒），一样都没做" "只给自证用；平常不设"
  PROGRESS_INTERVAL="$MERGE_AND_VERIFY_PROGRESS_INTERVAL"
fi
# 收尾（EXIT trap，正常结束、红了退出、被 INT / TERM 停都走这里）：第一步删掉这一趟自己建的编译目录（文件头「这一趟的编译目录」），在别的任何一条命令之前，
# 删不成记退 1、照接着往下收；然后删掉这里建的快照（文件头「快照」）。
# MERGE_AND_VERIFY_KEEP_COPY=1 时快照与编译目录都留着、打路径。删不成退 1。
SNAPSHOT_TREE=""; RUN_TARGET=""
# 上一次尝试留下的快照（文件头「上一次尝试留下的快照」）：日志目录里记任务号与尝试号的文件名；两个变量是统一任务服务每次尝试交的（task-server/task_server.py）
ATTEMPT_RECORD_NAME="task-attempt"
# 槽位从多机配置取的两个根（文件头「槽位」），读的是同一份配置（scripts/hosts/lib_repository_copy.py 的 configuration_path）：
#   python3 - <仓根> build-root：这一台的编译根，BUILD_ROOT_BY_HOST 里这一台那一项（槽的编译目录）；取不到或不是绝对路径退 1、打原因
#   python3 - <仓根> default-work-root：这一台缺省的工作根，SHARED_DATA_ROOT 下的 work（照 .claude/hooks/lib_work_root.py 的 configured_data_root 核：绝对路径、落在 NFS_SHARED_ROOTS 某一段下）；
#     不看 SINGLEFS_DATA_ROOT（跟着它走的话，与只从配置读的编译根就不同源）；取不到退 1、打原因
SLOT_CONFIGURED_ROOTS_PROGRAM='
import importlib.util, os, sys
root, wanted = sys.argv[1], sys.argv[2]
def load(name, relative):
    specification = importlib.util.spec_from_file_location(name, os.path.join(root, relative))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module
copying = load("lib_repository_copy_for_slots", "scripts/hosts/lib_repository_copy.py")
path = copying.configuration_path(root)
if wanted == "default-work-root":
    work_roots = load("lib_work_root_for_slots", ".claude/hooks/lib_work_root.py")
    try:
        data_root = work_roots.configured_data_root({**os.environ, work_roots.MULTI_HOST_CONFIGURATION_VARIABLE: path})
    except ValueError as error:
        print(f"照多机配置 {path} 取数据根 SHARED_DATA_ROOT 没取成（这里不看 SINGLEFS_DATA_ROOT）：{error}")
        sys.exit(1)
    print(os.path.join(data_root, work_roots.WORK_ENTRY))
    sys.exit(0)
if wanted != "build-root":
    print(f"认不出要取哪一个根「{wanted}」（只认 build-root、default-work-root）")
    sys.exit(1)
addresses = load("multi_host_addresses_for_slots", "scripts/hosts/multi_host_addresses.py")
try:
    this = addresses.resolve(path).this
    value = addresses.configured_by_host(path, "BUILD_ROOT_BY_HOST")[this]
except (ValueError, KeyError) as error:
    print(f"多机配置 {path} 里取不到这一台的编译根 BUILD_ROOT_BY_HOST（{error}）")
    sys.exit(1)
if not os.path.isabs(value):
    print(f"多机配置 {path} 里这一台的编译根 BUILD_ROOT_BY_HOST 不是绝对路径")
    sys.exit(1)
print(value)
'
# 挑槽、占锁的那个进程（文件头「槽位」的占哪一槽、多出来的槽）。python3 -c <它> <槽那一层> <登记的槽数> <优先的槽> <起它的 shell 的进程号> <弄坏开关> <take|trim>：
#   只在按 used 从新到旧排的前 <登记的槽数> 槽里挑；排在后面的是多出来的槽，拿着 slots.lock 逐个清（锁拿得到、不是留着的整个删掉，占着的、留着的跳过），
#   每清或跳过一槽答一行「SLOT note <要打的一行>」。take：答一行「SLOT held <槽><TAB><怎么挑的>」之后占着锁，起它的 shell 不在了就退（锁随之放）；
#   没占到答「SLOT none <为什么>」、退 0。trim：只清多出来的槽、不挑不占，答「SLOT none 只清多出来的槽…」、退 0。
#   弄坏开关：slot-lock-skipped 不加锁；slot-kept-taken 留着的槽也占；slot-not-reused 不接着用空着的槽；slot-limit-ignored 不看上限；earlier-attempt-slot-not-preferred 不先挑优先的槽；
#   slot-surplus-taken 不截前 <登记的槽数> 槽（已有的槽都可挑，也就没有多出来的）；slot-surplus-left 多出来的槽不清；slot-surplus-kept-removed 清多出来的槽时留着的也删。
#   清多出来的槽另加锁、另认留着的，不看 slot-lock-skipped 与 slot-kept-taken
SLOT_HOLDER_PROGRAM='
import errno, fcntl, os, shutil, sys, tempfile, time
root, limit, preferred, parent, broken, mode = sys.argv[1], int(sys.argv[2]), sys.argv[3], int(sys.argv[4]), sys.argv[5], sys.argv[6]
def say(text):
    print(text, flush=True)
def is_kept(slot):
    return os.path.isfile(os.path.join(slot, "kept")) and os.path.isdir(os.path.join(slot, "repo"))
def lock_slot(slot):
    handle = open(os.path.join(slot, "lock"), "a")
    if broken != "slot-lock-skipped":
        try:
            fcntl.lockf(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError as error:
            handle.close()
            if error.errno in (errno.EACCES, errno.EAGAIN):
                return None
            raise
    if is_kept(slot) and broken != "slot-kept-taken":
        handle.close()
        return None
    return handle
def used_at(slot):
    try:
        return os.stat(os.path.join(slot, "used")).st_mtime_ns
    except OSError:
        return 0
def trim_surplus(slot, count):
    handle = open(os.path.join(slot, "lock"), "a")
    try:
        fcntl.lockf(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError as error:
        handle.close()
        if error.errno in (errno.EACCES, errno.EAGAIN):
            return f"! 多出来的槽 {slot} 有别的趟占着，这一次不删"
        raise
    handle.close()
    if is_kept(slot) and broken != "slot-surplus-kept-removed":
        return f"! 多出来的槽 {slot} 留着（kept 在、repo/ 也在），不删；放了槽之后下一趟再清"
    try:
        shutil.rmtree(slot)
    except OSError as error:
        return f"! 多出来的槽 {slot} 没删成（{type(error).__name__}: {error}），下一趟再清"
    return f"删了多出来的槽 {slot}（槽位登记 slots={limit}，这一刻 {count} 槽；这一台编译根下它的编译目录在下面跟着删）"
try:
    os.makedirs(root, exist_ok=True)
    registry = open(os.path.join(root, "slots.lock"), "a")
    fcntl.lockf(registry, fcntl.LOCK_EX)
    slots = [os.path.realpath(entry.path) for entry in os.scandir(root) if entry.name.startswith("slot.") and entry.is_dir(follow_symlinks=False)]
    order = sorted(slots, key=used_at, reverse=True)
    eligible, surplus = (order, []) if broken == "slot-surplus-taken" else (order[:limit], order[limit:])
    preferred = os.path.realpath(preferred) if preferred else ""
    if preferred in eligible and broken != "earlier-attempt-slot-not-preferred":
        eligible = [preferred] + [slot for slot in eligible if slot != preferred]
    held, handle, how = None, None, ""
    if mode == "take":
        for slot in ([] if broken == "slot-not-reused" else eligible):
            handle = lock_slot(slot)
            if handle is not None:
                held = slot
                how = "同一任务上一次尝试占过的那一槽" if slot == preferred else "空着的槽里最近占过的那一槽"
                break
        if held is None and (len(slots) < limit or broken == "slot-limit-ignored"):
            held = os.path.realpath(tempfile.mkdtemp(prefix="slot.", dir=root))
            handle = lock_slot(held)
            how = f"新开一槽：这一刻 {len(slots) + 1} 槽，登记的上限 {limit} 槽"
        if held is not None and handle is not None:
            for name in ("kept", ".owner", ".deliverables"):
                try:
                    os.remove(os.path.join(held, name))
                except FileNotFoundError:
                    pass
            with open(os.path.join(held, "used"), "w") as stamp:
                stamp.write(f"{time.time_ns()}\n")
    for slot in ([] if broken == "slot-surplus-left" else surplus):
        say(f"SLOT note {trim_surplus(slot, len(slots))}")
    registry.close()
except OSError as error:
    say(f"SLOT none 挑槽出错（{type(error).__name__}: {error}）")
    sys.exit(0)
if mode != "take":
    say(f"SLOT none 只清多出来的槽、不占（槽位登记 slots={limit}）")
    sys.exit(0)
if held is None or handle is None:
    kept = sum(1 for slot in eligible if is_kept(slot))
    say(f"SLOT none 登记的 {limit} 槽都有了、没有空着的（{kept} 槽是带 MERGE_AND_VERIFY_KEEP_COPY=1 留下快照、还没放的，别的有别的趟在跑）")
    sys.exit(0)
say(f"SLOT held {held}\t{how}")
while True:
    try:
        os.kill(parent, 0)
    except OSError:
        break
    time.sleep(0.5)
'
# ── 崩溃域上限（文件头「崩溃域上限」）用的 python 段 ────────────
# python3 -c <它> <估算库> <根> <运行记录目录> <gpu 0|1> <弄坏开关> <判定文件> <crash-case: 键…>：照合表把点名的用例分到项（每项只算点名的用例），调估算库的 run_scoped_domain guard。
#   判定文件一行一样：① pass | narrow <n> | fallback（估不出或算不出 n，缩多少由 verification_crash_domain 定）；② 一句话；其余是每项的估数。库打的放行句、拒绝说明与读库的原因进日志（标准输出、标准错误）。
#   run_scoped_domain 不交各项的估数，这里包一层 estimate_subject 记下来：分得清估不出与超上限，不解析拒绝说明的字。
#   点名的用例有不在合表里的：估不出。弄坏开关 crash-domain-whole-table-item：照合表整项
MERGE_AND_VERIFY_CRASH_DOMAIN_PROGRAM="$(cat <<'PY_CRASH_DOMAIN'
import importlib.util
import io
import sys

library_path, root, runs, gpu, broken, decision_path = sys.argv[1:7]
named = sys.argv[7:]
specification = importlib.util.spec_from_file_location("lib_crash_point_replay_items", library_path)
library = importlib.util.module_from_spec(specification)
specification.loader.exec_module(library)

limit = library.SCOPED_CRASH_STATES_LIMIT
items, table_problems = library.read_items(root)
try:
    case_rows = library.crash_case_rows(root)
except OSError as error:
    case_rows = {}
    table_problems.append(f"读不了 stage-inputs.tsv：{error}")
subjects, placed = [], set()
for item in items:
    keys = [key for key in item.crash_cases if key in named]
    if not keys:
        continue
    placed.update(keys)
    if broken == "crash-domain-whole-table-item":
        subjects.append(library.merged_subject(root, item.name))
    else:
        subjects.append(library.Subject(item.name, keys, [], [case_rows[key][0] for key in keys if key in case_rows],
                                        root=None if broken == "crash-domain-subject-without-root" else root))
unplaced = [key for key in named if key not in placed]

estimates = []
estimate_subject = library.estimate_subject

def recording_estimate(*arguments):
    estimate = estimate_subject(*arguments)
    estimates.append(estimate)
    return estimate

library.estimate_subject = recording_estimate

def decide():
    """→ (判定, 一句话)；库退 0 与拒绝码之外的码照那个码退出，判定文件不写。"""
    if unplaced:
        return "fallback", f"点名的 {len(unplaced)} 条崩溃枚举用例不在合表里，估不出要现判多少态（{'、'.join(unplaced)}；{'；'.join(table_problems) or '合表读得了'}）"
    stream = io.StringIO()
    code = library.run_scoped_domain("guard", root, runs, None, gpu == "1", "", subjects, stream, sys.stderr)
    guard_lines = stream.getvalue().splitlines()
    print("\n".join(guard_lines))
    if code == 0:
        return "pass", (guard_lines[0].strip(" ·") if guard_lines else f"崩溃域上限 {limit} 态：放行")
    if code != library.SCOPED_DOMAIN_REFUSED_EXIT:
        sys.exit(code)
    constant = library.engine_expand_constant(root)
    best = library.suggested_depth([library.prior_parts(subject, runs, constant) for subject in subjects], constant, limit)
    unknown = [estimate for estimate in estimates if estimate.states is None]
    total = sum(estimate.states or 0 for estimate in estimates)
    estimated = f"要现判多少崩溃状态估不出（合表 {len(unknown)} 项算不出，上限 {limit} 态）" if unknown else f"要现判约 {total} 态，超过上限 {limit} 态"
    if best:
        return f"narrow {best[0]}", f"点名的 {len(named)} 条崩溃枚举用例{estimated}；n 取 {best[0]} 时这几条上一趟真跑共 {best[1]} 态，在上限以内，缩到展开深度 {best[0]} 接着跑（缩过域、不算门禁通过）"
    return "fallback", f"点名的 {len(named)} 条崩溃枚举用例{estimated}" + ("" if unknown else "，又算不出能压到上限以内的 n")

decision, head = decide()
details = []
for estimate in estimates:
    shown = "算不出" if estimate.states is None else f"约 {estimate.states} 态"
    details.append(f"     · {estimate.label}：{shown}（{estimate.text}）")
with open(decision_path, "w", encoding="utf-8") as handle:
    handle.write("\n".join([decision, head] + details) + "\n")
PY_CRASH_DOMAIN
)"

# ── 挑测试目标、逐项点名与按条复用（文件头「挑测试目标」「逐项点名与按条复用」）────────────
# 用的 python 段（python3 - <一样> … <<<"$MERGE_AND_VERIFY_ITEMS_PROGRAM"），照第一个参数做一样：
#   targets <树> <rerun-failed-tests.py> <lib_rebuilt_packages.py> <弄坏开关> <补丁目录>…   挑测试目标（文件头「挑测试目标」）：判不出的每处一行「REFUSE<TAB>原因」，
#        挑出的每项一行「ITEM<TAB>项<TAB>包<TAB>挑目标的参数<TAB>为什么挑上」（照包在工作区里的次序、包里目标的次序），挑出的项所在而包目录下有测试属性的「TESTED<TAB>包」，
#        要特性的目标「REQUIRES<TAB>包<TAB>目标名<TAB>特性,特性<TAB>为什么」，报一句的「NOTE<TAB>一句」，改到的必红世界表里新加或改过的每个项一行「FORMAL<TAB>表路径<TAB>项」（lib_merge_verify_formal_tables.py 的 Registered 打）；读不了补丁、成员清单或目标清单退 1、打一句原因。
#        弄坏开关 whole-package-tests：改到的包与工作区里依赖它们的包，每个目标都挑（文档测试除外）；fallback-runs-whole-workspace：判不出的不报、改成每个包的每个目标都挑（文档测试除外）；
#        path-attribute-ignored：找被引的模块文件时 #[path] 当没写；subdirectory-whole-directory-added、subdirectory-fallback-dropped、subdirectory-fallback-note-dropped：
#        tests/<子目录>/ 下的文件照整目录那一条加挑、走不到时不退回、退回时不打说明（roots_of_path）；
#        manifest-sections-refused、manifest-other-sections-unchecked、manifest-default-path-ignored：包的 Cargo.toml 的改动一律判不出、
#        不核 [[test]] / [[bin]] 之外的键有没有改与有没有段有差别、段里没写 path 的当没有根文件（land_manifest_changes、manifest_target_changes）；
#        manifest-shadowed-targets-unchecked、manifest-targets-compared-by-name：不比新旧两侧的目标集合（自动认的目标被顶掉也照落）、两侧集合按（段，名字）比而不按根文件比
#        （land_manifest_changes、shadowed_automatic_targets）；manifest-paths-unnormalized、directory-form-default-root-ignored、edition-2015-automatic-targets-kept：
#        段写的 path 不归一、没写 path 的不找目录形根文件、推自动认的目标不看 edition（target_options 交给 rerun-failed-tests.py 推目标的那几个函数）
#   split <树> <rerun-failed-tests.py> <日志> <输出目录> <项>…   日志逐段归到项（section_item）：每段一行
#        「SECTION<TAB>项（归不到写 -）<TAB>段头的源文件<TAB>finished in 秒数之和<TAB>FAILED 条数<TAB>结果行条数」，每条 FAILED 一行「FAILED<TAB>项<TAB>libtest 全名」，
#        段头之前的结果行打「LOOSE<TAB>条数」；归到项的段按包接进 <输出目录>/<包>.log
#   outcomes <rerun-failed-tests.py> <test-plan 的输出> <这个包那几段的日志，没有写 -> <包> <红的名字落进的文件> <项>…   这个包跑到的项里每条测试函数一行
#        「<名字><TAB><项><TAB><现判|复用|拒算><TAB><绿|红|没跑到>」（拒算的写拒算；日志里有结果行的写现判；别的照计划写复用或现判）；计划里要现判而 FAILED 的名字写进第五个参数
MERGE_AND_VERIFY_ITEMS_PROGRAM="$(cat <<'PY_ITEMS'
import importlib.util, os, re, shlex, shutil, subprocess, sys, tempfile, tomllib
TEST_ATTRIBUTE = re.compile(r"#\[\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*test\s*[\](]")
FINISHED = re.compile(r"finished in ([0-9]+(?:\.[0-9]+)?)s")
def load(path, name="rerun_failed_tests_for_merge_and_verify"):
    specification = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module
def has_test_attribute(directory):
    for base, directories, files in os.walk(directory):
        directories[:] = [name for name in directories if name not in ("target", ".git") and not name.startswith("target-")]
        for name in files:
            if not name.endswith(".rs"):
                continue
            try:
                with open(os.path.join(base, name), encoding="utf-8", errors="replace") as handle:
                    if TEST_ATTRIBUTE.search(handle.read()):
                        return True
            except OSError:
                return True
    return False
def sections_of(rerun, path):
    sections, loose, current = [], 0, None
    with open(path, encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
            header, doctest = rerun.SECTION_HEADER.match(line), rerun.DOCTEST_HEADER.match(line)
            running = re.match(r"^\s*Running\s+(.+)", line)
            if header or doctest or running:
                source = header.group(1) if header else (f"doc-tests {doctest.group(1)}" if doctest else running.group(1))
                current = {"source": source, "artifact": header.group(2).strip() if header else "", "lines": [line]}
                sections.append(current)
            elif current is None:
                loose += rerun.libtest_result_lines.result_line(line) is not None
            else:
                current["lines"].append(line)
    return sections, loose
WORKSPACE_WIDE = ("Cargo.toml", "Cargo.lock")
SELECTOR_VALUE_OPTIONS = ("--test", "--bin", "--example", "--bench")
SELECTOR_FLAGS = ("--lib", "--doc")
WHOLE_CLASS_OPTIONS = ("--tests", "--bins", "--examples", "--benches", "--all-targets", "--workspace", "--all", "--exclude")
CARGO_VALUE_OPTIONS = ("--features", "-F", "--target", "--target-dir", "--profile", "-j", "--jobs", "--manifest-path", "--color", "--message-format", "--config", "-Z")
def workspace_wide(path):
    return path in WORKSPACE_WIDE or path == ".cargo" or path.startswith(".cargo/")
def target_root_form(relative):
    """测试目标根文件那一类的路径：tests/<目标>.rs、tests/<目标>/main.rs、src/bin/<名>.rs、src/bin/<名>/main.rs。"""
    parts = relative.split("/")
    return (len(parts) == 2 and parts[0] == "tests" and parts[1].endswith(".rs")) or (len(parts) == 3 and parts[0] == "tests" and parts[2] == "main.rs") \
        or (len(parts) == 3 and parts[:2] == ["src", "bin"] and parts[2].endswith(".rs")) or (len(parts) == 4 and parts[:2] == ["src", "bin"] and parts[3] == "main.rs")
def target_options(broken):
    """交给 rerun-failed-tests.py 推目标那几个函数（section_targets、entry_root_paths、test_targets_of_package）的关键字参数：
    弄坏开关 manifest-paths-unnormalized、directory-form-default-root-ignored、edition-2015-automatic-targets-kept 在那里认。"""
    return {"normalized": broken != "manifest-paths-unnormalized", "directory_form": broken != "directory-form-default-root-ignored",
            "edition_ignored": broken == "edition-2015-automatic-targets-kept"}
def manifest_old_text(tree, path, patch_directories):
    """给的补丁改之前的 path（包的 Cargo.toml，树根起的相对路径）→ (文本, 原因)：在临时 git 仓里放一份树里此刻的它，照给的次序倒着逐份 git apply --reverse
    （--include 只认点到这份文件的，后打的先倒）。补丁新建的文件倒完不在，文本交空串；倒不掉、临时仓建不成交 (None, 原因)。"""
    copy = tempfile.mkdtemp(prefix="merge-and-verify-manifest.")
    try:
        started = subprocess.run(["git", "init", "-q", copy], capture_output=True, text=True)
        if started.returncode != 0:
            return None, f"临时目录 {copy} 里 git init 不成：{started.stderr.strip()}"
        target = os.path.join(copy, path)
        os.makedirs(os.path.dirname(target), exist_ok=True)
        shutil.copyfile(os.path.join(tree, path), target)
        for directory in reversed(patch_directories):
            patch = os.path.join(directory, "crates.patch")
            if not os.path.isfile(patch):
                continue
            applied = subprocess.run(["git", "-C", copy, "apply", "--reverse", f"--include={path}", patch], capture_output=True, text=True)
            if applied.returncode != 0:
                return None, f"{directory} 的 crates.patch 倒不掉：{applied.stderr.strip()}"
        if not os.path.isfile(target):
            return "", ""
        with open(target, encoding="utf-8", errors="replace") as handle:
            return handle.read(), ""
    except OSError as error:
        return None, f"读不了或拷不成：{error}"
    finally:
        shutil.rmtree(copy, ignore_errors=True)
def manifest_target_changes(old, new, broken):
    """包的 Cargo.toml 新旧两侧读成的表 → ([(段, 名字, 旧侧的段或 None, 新侧的段或 None)…], 不是只落在 [[test]] / [[bin]] 段里的原因或 "")。
    [[test]] / [[bin]] 之外的键有任何差别、两侧段名缺了或重名、没有一个 [[test]] / [[bin]] 段有差别，都交原因。
    弄坏开关 manifest-other-sections-unchecked：不核这几样，别的键有差别、没有段有差别也照交（交空表）。"""
    sections = ("test", "bin")
    if broken != "manifest-other-sections-unchecked":
        changed = sorted(key for key in set(old) | set(new) if key not in sections and old.get(key) != new.get(key))
        if changed:
            return [], f"{'、'.join('[' + key + ']' for key in changed)} 段有改动"
    changes = []
    for section in sections:
        before, after = old.get(section, []), new.get(section, [])
        if not all(isinstance(entries, list) and all(isinstance(entry, dict) for entry in entries) for entries in (before, after)):
            return [], f"[{section}] 不是 [[{section}]] 那样的段数组"
        names = [[entry.get("name") for entry in entries] for entries in (before, after)]
        if any(not isinstance(name, str) for listed in names for name in listed) or any(len(set(listed)) != len(listed) for listed in names):
            return [], f"[[{section}]] 段的 name 缺了、不是字符串或重名，新旧两侧的段对不上"
        old_by_name, new_by_name = dict(zip(names[0], before)), dict(zip(names[1], after))
        for name in dict.fromkeys(names[1] + names[0]):
            if old_by_name.get(name) != new_by_name.get(name):
                changes.append((section, name, old_by_name.get(name), new_by_name.get(name)))
    if not changes and broken != "manifest-other-sections-unchecked":
        return [], "没有一个 [[test]] / [[bin]] 段有差别（Cargo.toml 只动了注释、空行或写法）"
    return changes, ""
def manifest_targets(rerun, directory, package, manifest, options):
    """包的 [[test]] / [[bin]] 目标集合（文件头「挑测试目标」自动认的目标被顶掉那一条）→ {(段, 目标名): (根文件, 是不是自动认的)}：
    照 rerun-failed-tests.py 的 section_targets（写了的段、path 归一、缺省根文件、自动认的与 edition 2015 那一条都在那一份里；options 是 target_options）。
    required-features 与 test = false 不影响这个集合。"""
    return {(section, name): (root, automatic) for section in ("bin", "test")
            for name, (root, automatic, _entry) in rerun.section_targets(directory, manifest, package, section, options["normalized"], options["directory_form"],
                                                                         options["edition_ignored"]).items()}
def shadowed_automatic_targets(old_targets, new_targets, broken):
    """旧侧自动认的目标的根文件在新侧不再是同一种段里任何目标的根文件（manifest_targets 的两侧，按根文件比）→ 每个一句，点名段、目标名与根文件。
    新加的段 path 指到自动认的那份文件而名字不同的，根文件还有目标认，不算顶掉。
    弄坏开关 manifest-targets-compared-by-name：按（段，名字）比，新侧没有同名的、或同名的根文件换了就算顶掉。"""
    new_roots = {(section, root) for (section, _name), (root, _automatic) in new_targets.items()}
    shadowed = []
    for (section, name), (root, automatic) in old_targets.items():
        after = new_targets.get((section, name))
        if not automatic or (after is not None and after[0] == root):
            continue
        if broken != "manifest-targets-compared-by-name" and (section, root) in new_roots:
            continue
        change = f"改后同名的 [[{section}]] 段指到 {after[0]}" if after is not None else f"改后它的根文件不再是任何 [[{section}]] 目标的根文件"
        shadowed.append(f"{section} 目标 {name}（自动认的，根文件 {root}）{change}")
    return shadowed
def module_pattern(name):
    return re.compile(rf"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+{re.escape(name)}\s*;|#\[\s*path\s*=\s*\"{re.escape(name)}/", re.MULTILINE)
MODULE_DECLARATION = re.compile(r"((?:#\[(?:[^\]\"]|\"(?:[^\"\\]|\\.)*\")*\]\s*)*)(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")
PATH_ATTRIBUTE = re.compile(r"#\[\s*path\s*=\s*\"([^\"]*)\"\s*\]")
SOURCE_COMMENTS = re.compile(r"//[^\n]*|/\*.*?\*/", re.DOTALL)
def files_reached_from(directory, root, path_attribute_ignored):
    """从测试目标根文件起沿 `mod <名>;` 与 `#[path = "…"] mod <名>;` 往下跟（照 Rust 找模块文件的办法），交跟得到的文件（包内相对路径，含根文件自己）。
    #[path] 的值相对写这一行的文件所在目录；根文件、mod.rs 与经 #[path] 引进来的文件，它们的子模块在它们自己所在的目录里找，别的 <名>.rs 的子模块在同目录的 <名>/ 里找。
    写在内联模块 `mod x { … }` 里的 mod 声明与 cfg_attr 里写的 path 不认。弄坏开关 path-attribute-ignored：#[path] 当没写，`mod <名>;` 只找 <名>.rs 与 <名>/mod.rs。"""
    reached, pending = {root}, [(root, True)]
    while pending:
        relative, owns_directory = pending.pop()
        try:
            with open(os.path.join(directory, relative), encoding="utf-8", errors="replace") as handle:
                text = SOURCE_COMMENTS.sub("", handle.read())
        except OSError:
            continue
        here = os.path.dirname(relative)
        children = here if owns_directory else os.path.join(here, os.path.splitext(os.path.basename(relative))[0])
        for attributes, name in MODULE_DECLARATION.findall(text):
            written = None if path_attribute_ignored else PATH_ATTRIBUTE.search(attributes)
            if written:
                candidates = [(os.path.normpath(os.path.join(here, written.group(1))), True)]
            else:
                candidates = [(os.path.normpath(os.path.join(children, name + ".rs")), False), (os.path.normpath(os.path.join(children, name, "mod.rs")), True)]
            for candidate, owns in candidates:
                if os.path.isfile(os.path.join(directory, candidate)):
                    if candidate not in reached:
                        reached.add(candidate)
                        pending.append((candidate, owns))
                    break
    return reached
def mutation_target(words):
    """变异行第五段 → (包, [挑法…], `--` 之后的筛选词, 点了一类目标时那个选项)。"""
    cut = words.index("--") if "--" in words else len(words)
    head, filters = words[:cut], words[cut + 1:]
    package, selectors, whole_class, position = None, [], None, 0
    while position < len(head):
        word = head[position]
        option, _, value = word.partition("=")
        if word in ("-p", "--package"):
            package = head[position + 1] if position + 1 < len(head) else None
            position += 2
            continue
        if option == "--package" and value:
            package = value
        elif word.startswith("-p") and len(word) > 2:
            package = word[2:]
        elif word in SELECTOR_FLAGS:
            selectors.append([word])
        elif word in SELECTOR_VALUE_OPTIONS:
            selectors.append([word, head[position + 1] if position + 1 < len(head) else ""])
            position += 2
            continue
        elif option in SELECTOR_VALUE_OPTIONS and value:
            selectors.append([option, value])
        elif option in WHOLE_CLASS_OPTIONS:
            whole_class = word
        elif word in CARGO_VALUE_OPTIONS:
            position += 2
            continue
        position += 1
    return package, selectors, filters, whole_class
def mutation_rows(directory):
    for file_name in ("mutations-append.tsv", "mutations-replacements.tsv"):
        path = os.path.join(directory, file_name)
        if not os.path.isfile(path):
            continue
        with open(path, encoding="utf-8") as handle:
            for raw in handle:
                line = raw.rstrip("\n")
                if line.strip() and not line.startswith("#"):
                    yield file_name, line.split("\t")
def string_literal_names_document(literal, file_name):
    """字符串字面量（不带引号）里写的是不是这份不变量文档：含 invariants/、整段是 invariants 这个路径段，或含文档文件名。"""
    return "invariants/" in literal or literal == "invariants" or literal.endswith("/invariants") or file_name in literal
STRING_LITERAL = re.compile(r'"((?:[^"\\\n]|\\.)*)"')
def select_targets(tree, rerun, rebuilt, broken, patch_directories):
    packages = rerun.workspace_packages(tree)
    order = list(packages)
    options = target_options(broken)
    targets_of = {name: rerun.test_targets_of_package(directory, manifest, name, options["normalized"], options["directory_form"], options["edition_ignored"])
                  for name, (directory, manifest) in packages.items()}
    selected, refusals, required, notes, touched = {}, [], {}, [], []
    def select(package, root, why):
        entry = selected.setdefault((package, root), [])
        if why not in entry:
            entry.append(why)
    def required_entries(package):
        """写了 required-features 的 [[test]] / [[bin]] 逐个交 (目标名, 特性, 根文件, 根文件是 main.rs 时它所在的目录，别的是 None)。"""
        manifest = packages[package][1]
        for section in ("test", "bin"):
            for entry in manifest.get(section, []):
                entry_name, features = entry.get("name"), entry.get("required-features")
                if not isinstance(entry_name, str) or not features:
                    continue
                path = rerun.entry_root_paths(packages[package][0], package, section, entry, options["normalized"], options["directory_form"])[0]
                yield entry_name, features, path, os.path.dirname(path) if path.endswith("/main.rs") else None
    def required_entry(package, relative=None, name=None):
        """写了 required-features 的 [[test]] / [[bin]]：按路径（relative）或按名字（name）找 → (目标名, 特性) 或 None。"""
        for entry_name, features, path, folder_root in required_entries(package):
            if (name is not None and entry_name == name) or (relative is not None and (relative == path or (folder_root and relative.startswith(folder_root + "/")))):
                return entry_name, features
        return None
    reach_cache = {}
    def reached_by(package, root):
        """测试目标根文件 root 经 mod / #[path] 链走得到的文件（files_reached_from），每个根算一次。"""
        if (package, root) not in reach_cache:
            reach_cache[(package, root)] = files_reached_from(packages[package][0], root, broken == "path-attribute-ignored")
        return reach_cache[(package, root)]
    def roots_reaching(package, relative):
        """不要特性的测试目标里，哪些的 mod / #[path] 链走得到包里的 relative（只有 .rs 能是模块）。"""
        if not relative.endswith(".rs"):
            return []
        return [root for root in targets_of[package] if root != rerun.DOC_TARGET and relative in reached_by(package, root)]
    def required_reaching(package, relative):
        """要特性的测试目标里，哪些的 mod / #[path] 链走得到包里的 relative → [(目标名, 特性)…]。"""
        if not relative.endswith(".rs"):
            return []
        return [(entry_name, features) for entry_name, features, path, _folder in required_entries(package)
                if os.path.isfile(os.path.join(packages[package][0], path)) and relative in reached_by(package, path)]
    def in_tests_subdirectory(package, relative):
        """tests/<子目录>/ 下、又不是测试目标根文件的路径。"""
        parts = relative.split("/")
        return parts[0] == "tests" and len(parts) >= 3 and relative not in targets_of[package]
    def roots_of_path(package, relative, why):
        """relative 落到哪些不要特性的测试目标。tests/<子目录>/ 下的先按 mod / #[path] 链走得到它的落（要特性的目标走得到也算走得到，那几个由 land_on_targets 记）；
        一个都走不到才退回整目录那一条（subdirectory_roots），退回时记一句 NOTE。别的照位置落的（roots_by_location）加上链走得到它的。
        弄坏开关 subdirectory-whole-directory-added：子目录里的也照位置落、再加链走得到的；subdirectory-fallback-dropped：走不到不退回；
        subdirectory-fallback-note-dropped：退回不记那一句。"""
        reaching = roots_reaching(package, relative)
        if in_tests_subdirectory(package, relative) and broken != "subdirectory-whole-directory-added":
            if reaching or required_reaching(package, relative) or broken == "subdirectory-fallback-dropped":
                return reaching
            roots = subdirectory_roots(package, relative)
            if roots and broken != "subdirectory-fallback-note-dropped":
                folder = relative.split("/")[1]
                notes.append(f"{why}：{package} 的测试目标没有一个经 mod / #[path] 走得到 {relative}，退回整目录：落到 tests/{folder}/main.rs，"
                             f"或 tests/ 顶层里写了 mod {folder}（或 #[path = \"{folder}/…\"]）的目标，共 {len(roots)} 个（{'、'.join(roots)}）")
            return roots
        roots = roots_by_location(package, relative)
        return roots + [root for root in reaching if root not in roots]
    def subdirectory_roots(package, relative):
        """tests/<子目录>/ 下的整目录那一条：tests/<子目录>/main.rs 那个目标；没有它，就是 tests/ 顶层里写了 mod <子目录>（或 #[path = "<子目录>/…"]）的那几个目标。"""
        directory, targets, folder = packages[package][0], targets_of[package], relative.split("/")[1]
        main = f"tests/{folder}/main.rs"
        if main in targets:
            return [main]
        pattern, hits = module_pattern(folder), []
        for root in targets:
            if root.startswith("tests/") and root.count("/") == 1:
                try:
                    with open(os.path.join(directory, root), encoding="utf-8", errors="replace") as handle:
                        if pattern.search(handle.read()):
                            hits.append(root)
                except OSError:
                    continue
        return hits
    def roots_by_location(package, relative):
        directory, manifest = packages[package]
        targets = targets_of[package]
        if relative in targets and relative != rerun.DOC_TARGET:
            return [relative]
        parts = relative.split("/")
        if parts[0] == "tests" and len(parts) >= 3:
            return subdirectory_roots(package, relative)
        if parts[:2] == ["src", "bin"]:
            main = f"src/bin/{parts[2]}/main.rs" if len(parts) >= 4 else None
            return [main] if main in targets else []
        if parts[0] == "src" and len(parts) >= 2:
            library = (manifest.get("lib") or {}).get("path", "src/lib.rs")
            for root in (library, "src/main.rs"):
                if root in targets:
                    return [root]
        return []
    def land_on_targets(package, relative, why):
        """包里的 relative 落到测试目标（要特性的记进 required）：落到了交 True。"""
        feature = required_entry(package, relative=relative)
        if feature:
            required.setdefault((package, feature[0]), [",".join(feature[1]), []])[1].append(why)
            return True
        roots = roots_of_path(package, relative, why)
        for root in roots:
            select(package, root, why)
        reached_features = required_reaching(package, relative)
        for entry_name, features in reached_features:
            required.setdefault((package, entry_name), [",".join(features), []])[1].append(why)
        return bool(roots or reached_features)
    def readers_of_document(path):
        """工作区各包的 .rs 里读这份不变量文档的（文件头「挑测试目标」不变量文档那一条）→ [(包, 包里的相对路径)…]。"""
        file_name = os.path.basename(path)
        admission = load(os.path.join(os.path.dirname(rerun.__file__), "admission.py"), "admission_for_merge_and_verify")
        readers = []
        for package in order:
            for base, directories, files in os.walk(packages[package][0]):
                directories[:] = [name for name in directories if name not in ("target", ".git") and not name.startswith("target-")]
                for name in files:
                    if not name.endswith(".rs"):
                        continue
                    full = os.path.join(base, name)
                    with open(full, encoding="utf-8", errors="replace") as handle:
                        text = handle.read()
                    if "invariants" not in text and file_name not in text:
                        continue
                    code = text if broken == "document-comments-counted" else admission.rust_code_without_comments(text)
                    if broken == "document-comments-counted":
                        named = "invariants/" in code or file_name in code
                    else:
                        named = any(string_literal_names_document(found.group(1), file_name) for found in STRING_LITERAL.finditer(code))
                    if named:
                        readers.append((package, os.path.relpath(full, packages[package][0])))
        return readers
    def land_manifest_changes(package, directory, path, why):
        """包的 Cargo.toml（path，树根起的相对路径）只改了 [[test]] / [[bin]] 段（文件头「挑测试目标」包的 Cargo.toml 那一条）：新加、改了、删掉的每个段落到它点名的目标
        （新旧两侧的根文件都落，要特性的记进 required，落不到又不是删掉或 test = false 的段判不出）；旧侧自动认的目标被顶掉的整份判不出。
        [[bin]] 段有差别的只落到那个 bin，不另挑同包的集成测试。交空串，或整份判不出的原因。
        弄坏开关 manifest-default-path-ignored：段里没写 path 的当没有根文件；manifest-shadowed-targets-unchecked：不比两侧的目标集合；
        manifest-targets-compared-by-name 在 shadowed_automatic_targets 里认，manifest-other-sections-unchecked 在 manifest_target_changes 里认。"""
        old_text, problem = manifest_old_text(tree, path, patch_directories)
        if old_text is None:
            return f"算不出补丁改之前的 {path}：{problem}"
        try:
            old = tomllib.loads(old_text)
        except tomllib.TOMLDecodeError as error:
            return f"补丁改之前的 {path} 读不了：{error}"
        new = packages[package][1]
        changes, problem = manifest_target_changes(old, new, broken)
        if problem:
            return problem
        if broken != "manifest-shadowed-targets-unchecked":
            shadowed = shadowed_automatic_targets(manifest_targets(rerun, directory, package, old, options), manifest_targets(rerun, directory, package, new, options), broken)
            if shadowed:
                # [package] 两侧相同（不同的上面已判不出），旧侧推、新侧不推自动认的目标只会是 edition 2015 那一条（rerun-failed-tests.py 的 automatic_targets_on）
                stopped = [f"[[{section}]]" for section in ("test", "bin") if rerun.automatic_targets_on(old, section, options["edition_ignored"])
                           and not rerun.automatic_targets_on(new, section, options["edition_ignored"])]
                edition_note = f"（包的 edition 照 cargo 算是 2015，写了 {'、'.join(stopped)} 段就不再自动认同一种目标）" if stopped else ""
                return f"自动认的目标被顶掉，cargo test 从此不跑它的根文件、一句不报：{'；'.join(shadowed)}{edition_note}"
        for section, name, before, after in changes:
            label = f"[[{section}]] 段 {name}"
            reason = f"{why}（{label}：{'新加' if before is None else '删掉' if after is None else '改了'}）"
            roots = []
            for entry in (after, before):
                if entry is None or (broken == "manifest-default-path-ignored" and not isinstance(entry.get("path"), str)):
                    continue
                roots += [found for found in rerun.entry_root_paths(directory, package, section, entry, options["normalized"], options["directory_form"]) if found not in roots]
            landed = False
            for root in roots:
                landed = land_on_targets(package, root, reason) or landed
            if landed:
                continue
            if after is None or after.get("test") is False:
                notes.append(f"{reason}：它的根文件 {'、'.join(roots)} 现在不是 cargo test 会跑的目标（段删掉了或写了 test = false），不跑")
            else:
                return f"{label} 落不到测试目标（根文件 {'、'.join(roots) or '没写 path'} 不是 cargo test 会跑的目标）"
        return ""
    # 这两种退 3（工作区根的文件、不落在任何包里的路径）没有包可点：整包跑的范围由用户来选（别的几种点名了包）。弄坏开关 refusal-packages-unnamed：不写这半句
    no_package_way_out = "" if broken == "refusal-packages-unnamed" else "；没有包可点，整包跑的范围交用户定"
    if not patch_directories:
        refusals.append("没给补丁目录：算不出这一批改到哪几个测试目标")
    registered = load(os.path.join(os.path.dirname(rebuilt.__file__), "lib_merge_verify_formal_tables.py"), "lib_merge_verify_formal_tables").Registered(tree, broken)
    for path in rebuilt.collect_paths(patch_directories, []):
        if note := registered.note_for(path):   # 登记表与 verus-pins.tsv 登记的必红世界表不落测试目标；弄坏开关 registration-tables-refused、formal-tables-refused、formal-tables-by-suffix 在它里面认
            notes.append(note)
            continue
        if workspace_wide(path):
            refusals.append(f"补丁改到工作区根的 {path}（改的是整个工作区的编法，判不出改到哪个测试目标{no_package_way_out}）")
            continue
        package = rebuilt.package_of(tree, path)
        if package not in packages:
            refusals.append(f"补丁改到不落在任何包里的 {path}（测试运行时可能读它，判不出归哪个测试目标{no_package_way_out}）")
            continue
        touched.append(package)
        directory = packages[package][0]
        relative = os.path.relpath(os.path.join(tree, path), directory)
        if land_on_targets(package, relative, f"补丁改到 {path}"):
            continue
        if target_root_form(relative) and not os.path.lexists(os.path.join(directory, relative)):
            notes.append(f"补丁删掉了 {path}（{package} 的这个测试目标已经没有，不跑）")
            continue
        if relative.startswith("invariants/") and relative.endswith(".md") and broken != "document-refused":
            readers = [] if broken == "document-readers-unscanned" else readers_of_document(path)
            if not readers:
                notes.append(f"补丁改到 {path}（{package} 包的不变量文档，工作区的 .rs 去掉注释之后没有一处读它，不影响测试目标，不跑）")
                continue
            for reader_package, reader_relative in readers:
                touched.append(reader_package)
                if not land_on_targets(reader_package, reader_relative, f"补丁改到 {path}，{reader_package} 的 {reader_relative} 读它"):
                    refusals.append(f"补丁改到 {path}，读它的 {reader_package} 的 {reader_relative} 落不到测试目标，判不出改到哪个测试目标")
            continue
        manifest_problem = ""
        if relative == "Cargo.toml" and broken != "manifest-sections-refused":   # 弄坏开关 manifest-sections-refused：包的 Cargo.toml 的改动一律照落不到测试目标拒
            manifest_problem = land_manifest_changes(package, directory, path, f"补丁改到 {path}")
            if not manifest_problem:
                continue
            manifest_problem = f"；包的 Cargo.toml 只改 [[test]] / [[bin]] 段才认得出：{manifest_problem}"
        refusals.append(f"补丁改到 {package} 包里的 {relative}，判不出改到哪个测试目标（不是测试目标的根文件、不在 src/ 下，tests/ 下也没有测试目标 mod 它{manifest_problem}）")
    for directory in patch_directories:
        for file_name, columns in mutation_rows(directory):
            name, label = columns[0], f"变异行「{columns[0]}」（{os.path.basename(directory)}/{file_name}）"
            if len(columns) < 6:
                refusals.append(f"{label}不是六段，读不出它点名的测试目标")
                continue
            try:
                words = shlex.split(columns[4])
            except ValueError as error:
                refusals.append(f"{label}的 cargo test 参数切不开（{error}）")
                continue
            package, selectors, filters, whole_class = mutation_target(words)
            if whole_class:
                refusals.append(f"{label}的 cargo test 参数点了一类目标 {whole_class}，判不出是哪一个测试目标")
                continue
            if package not in packages:
                refusals.append(f"{label}的 cargo test 参数{'没点包' if package is None else f'点的包 {package} 不在工作区里'}")
                continue
            if not selectors:
                refusals.append(f"{label}的 cargo test 参数只点了包 {package}、没点测试目标（等于跑整个包）")
                continue
            touched.append(package)
            for selector in selectors:
                if selector == ["--doc"]:
                    root = rerun.DOC_TARGET if rerun.DOC_TARGET in targets_of[package] else None
                else:
                    root = next((key for key, found in targets_of[package].items() if key != rerun.DOC_TARGET and list(found.selector) == selector), None)
                feature = required_entry(package, name=selector[1]) if root is None and len(selector) == 2 else None
                if root:
                    select(package, root, f"变异行「{name}」点名")
                    if filters:
                        notes.append(f"变异行「{name}」在 {rerun.item_name(package, root)} 里的筛选词 {' '.join(filters)}：跑的是整个目标，键相同的测试函数照复用")
                elif feature:
                    required.setdefault((package, feature[0]), [",".join(feature[1]), []])[1].append(f"变异行「{name}」点名")
                else:
                    refusals.append(f"{label}点名的 {' '.join(selector)} 在包 {package} 里找不到")
    if broken == "whole-package-tests":
        depends_on = {}
        for name, (directory, manifest) in packages.items():
            tables = [manifest.get(key) for key in ("dependencies", "dev-dependencies", "build-dependencies")]
            for target in (manifest.get("target") or {}).values():
                if isinstance(target, dict):
                    tables += [target.get(key) for key in ("dependencies", "dev-dependencies", "build-dependencies")]
            depends_on[name] = {value.get("package", key) if isinstance(value, dict) else key for table in tables for key, value in (table or {}).items()}
        widened = set(touched)
        while True:
            grown = {name for name, needed in depends_on.items() if name not in widened and needed & widened}
            if not grown:
                break
            widened |= grown
        for package in widened:
            for root in targets_of[package]:
                if root != rerun.DOC_TARGET:
                    select(package, root, "弄坏开关 whole-package-tests：改到的包与依赖它们的包整包挑")
    if refusals and broken == "fallback-runs-whole-workspace":
        notes.append(f"弄坏开关 fallback-runs-whole-workspace：判不出的 {len(refusals)} 处不报，每个包的每个目标都挑")
        refusals = []
        for package in order:
            for root in targets_of[package]:
                if root != rerun.DOC_TARGET:
                    select(package, root, "弄坏开关 fallback-runs-whole-workspace")
    for reason in refusals:
        print(f"REFUSE\t{reason}")
    # 判不出的之外落得到的照打 ITEM 等行：--list-items 照列（select_test_targets 只在列项时留着它们，真跑那一趟照旧清掉、测试那一步不跑）
    position = {package: index for index, package in enumerate(order)}
    ranks = {(package, root): rank for package in order for rank, root in enumerate(targets_of[package])}
    for package, root in sorted(selected, key=lambda key: (position[key[0]], ranks[key])):
        found = targets_of[package][root]
        print(f"ITEM\t{rerun.item_name(package, root)}\t{package}\t{' '.join(found.selector)}\t{'；'.join(selected[(package, root)])}")
    for package in sorted({package for package, _root in selected}, key=position.get):
        if has_test_attribute(packages[package][0]):
            print(f"TESTED\t{package}")
    for (package, name), (features, reasons) in required.items():
        print(f"REQUIRES\t{package}\t{name}\t{features}\t{'；'.join(dict.fromkeys(reasons))}")
    registered.report(notes)
mode = sys.argv[1]
if mode == "targets":
    tree, rerun, rebuilt, broken = sys.argv[2], load(sys.argv[3]), load(sys.argv[4], "lib_rebuilt_packages_for_merge_and_verify"), sys.argv[5]
    try:
        select_targets(tree, rerun, rebuilt, broken, sys.argv[6:])
    except (OSError, rerun.PackageNotFound) as error:
        print(f"读不了补丁、成员清单或目标清单：{error}")
        sys.exit(1)
elif mode == "split":
    tree, rerun, log, output = sys.argv[2], load(sys.argv[3]), sys.argv[4], sys.argv[5]
    packages = rerun.workspace_packages(tree)
    targets = {name: rerun.test_targets_of_package(directory, manifest, name) for name, (directory, manifest) in packages.items()}
    sections, loose = sections_of(rerun, log)
    os.makedirs(output, exist_ok=True)
    for section in sections:
        item = rerun.section_item(section["source"], section["artifact"], targets)
        seconds = sum(float(found.group(1)) for line in section["lines"] for found in [FINISHED.search(line)]
                      if found and rerun.libtest_result_lines.summary_line(line) is not None)
        failed = [name for name in (rerun.failed_name_of(line) for line in section["lines"]) if name is not None]
        results = sum(rerun.libtest_result_lines.result_line(line) is not None for line in section["lines"])
        shown = item if item else "-"
        print(f"SECTION\t{shown}\t{section['source']}\t{seconds}\t{len(failed)}\t{results}")
        for name in failed:
            print(f"FAILED\t{shown}\t{name}")
        if item:
            with open(os.path.join(output, rerun.split_item(item)[0] + ".log"), "a", encoding="utf-8") as handle:
                handle.write("\n".join(section["lines"]) + "\n")
    if loose:
        print(f"LOOSE\t{loose}")
elif mode == "outcomes":
    rerun, plan_output, package_log, package, red_file = load(sys.argv[2]), sys.argv[3], sys.argv[4], sys.argv[5], sys.argv[6]
    items, results, root = set(sys.argv[7:]), {}, None
    if package_log != "-" and os.path.isfile(package_log):
        with open(package_log, encoding="utf-8", errors="replace") as handle:
            for raw in handle:
                line = raw.rstrip("\n")
                header = rerun.SECTION_HEADER.match(line)
                if header or rerun.DOCTEST_HEADER.match(line):
                    root = header.group(1) if header else None
                    continue
                parsed = rerun.libtest_result_lines.result_line(line)
                if root and parsed is not None and parsed.outcome in ("ok", "FAILED"):
                    red = parsed.outcome == "FAILED" or results.get((root, parsed.full_name)) == "红"
                    results[(root, parsed.full_name)] = "红" if red else "绿"
    reds = []
    with open(plan_output, encoding="utf-8", errors="surrogateescape") as handle:
        for raw in handle:
            columns = raw.rstrip("\n").split("\t")
            if len(columns) < 7 or columns[1] not in ("judge", "reuse", "refused") or rerun.item_name(package, columns[4]) not in items:
                continue
            outcome = results.get((columns[4], columns[5]))
            state = "拒算" if columns[1] == "refused" else "现判" if outcome or columns[1] == "judge" else "复用"
            print(f"{columns[0]}\t{rerun.item_name(package, columns[4])}\t{state}\t{outcome or '没跑到'}")
            if outcome == "红" and columns[1] == "judge":
                reds.append(columns[0])
    with open(red_file, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{name}\n" for name in reds))
PY_ITEMS
)"

# 打补丁之前存原样的 python 段（文件头「先验后合」，verify 起）：python3 - <lib_rebuilt_packages.py> <快照> <原样目录> <路径单子> <补丁目录…>，
#   补丁动到的路径（crates.patch 新旧两侧，有变异表文件的另加 crates/mutations.tsv）写进路径单子，此刻在快照里的普通文件照相对路径拷进原样目录（原样目录要不在）；
#   点到树外的路径（绝对路径、.. 起头）退 1
MERGE_BASELINE_PROGRAM="$(cat <<'PY_MERGE_BASELINE'
import importlib.util
import os
import shutil
import sys

specification = importlib.util.spec_from_file_location("lib_rebuilt_packages", sys.argv[1])
library = importlib.util.module_from_spec(specification)
specification.loader.exec_module(library)
tree, baseline, paths_file = sys.argv[2], sys.argv[3], sys.argv[4]
directories = sys.argv[5:]
MUTATION_TABLE = "crates/mutations.tsv"
MUTATION_FILES = ("mutations-append.tsv", "mutations-replacements.tsv", "mutations-delete.txt")
paths = library.collect_paths(directories, [])
if any(os.path.isfile(os.path.join(directory, name)) for directory in directories for name in MUTATION_FILES):
    paths = list(dict.fromkeys(paths + [MUTATION_TABLE]))
outside = [path for path in paths if os.path.isabs(path) or os.path.normpath(path).split(os.sep)[0] == ".."]
if outside:
    raise SystemExit(f"补丁动到树外的路径：{'、'.join(outside)}")
os.makedirs(baseline)
kept = 0
for relative in paths:
    source = os.path.join(tree, relative)
    if os.path.isfile(source) and not os.path.islink(source):
        target = os.path.join(baseline, relative)
        os.makedirs(os.path.dirname(target), exist_ok=True)
        shutil.copy2(source, target)
        kept += 1
with open(paths_file, "w", encoding="utf-8") as handle:
    handle.write("".join(path + "\n" for path in paths))
print(f"补丁动到 {len(paths)} 份路径，打补丁之前在的 {kept} 份存进 {baseline}")
PY_MERGE_BASELINE
)"
# 合进主工作区的那一份补丁目录的 python 段（文件头「先验后合」，verify 起）：python3 - <lib_rebuilt_packages.py> <快照> <原样目录> <路径单子> <补丁目录的位置> <补丁目录…>，
#   照「原样 → 快照此刻」在临时 git 仓里算 crates.patch 与 baseline/，变异表按名字比出三份变异表文件，几份 result.md 照次序接成一份；先写进同一层的临时目录、写全了再改名到位，
#   位置上已有东西就退 1。末行打一句改了几份文件、变异行各几行。弄坏开关 merge-patch-single-directory：只算头一份补丁目录动到的文件；
#   merge-patch-stat-cache：补丁之后那一份写在基线那一份的原 inode 上（只在带 .merge-patch-stat-sample 标记的样本树上生效）；
#   merge-patch-preimage-dropped：变异行替换只写改后的行、删除只写名字，不带前像
MERGE_PATCH_PROGRAM="$(cat <<'PY_MERGE_PATCH'
import importlib.util
import os
import shutil
import subprocess
import sys
import tempfile

specification = importlib.util.spec_from_file_location("lib_rebuilt_packages", sys.argv[1])
library = importlib.util.module_from_spec(specification)
specification.loader.exec_module(library)
tree, baseline, paths_file, out = sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]
directories = sys.argv[6:]
MUTATION_TABLE = "crates/mutations.tsv"
# 弄坏开关 merge-patch-stat-cache：补丁之后那一份写在基线那一份的原 inode 上（修改时刻照源、git 不比改变时刻），git 的 stat 缓存看不出大小与修改时刻都相同的改动；
#   只在要验的树里有 .merge-patch-stat-sample 标记（自证直接喂这一段的那一格造的样本树）时生效，别的格的补丁不受它影响
STAT_CACHE_BROKEN = os.environ.get("MERGE_AND_VERIFY_BREAK") == "merge-patch-stat-cache" and os.path.exists(os.path.join(tree, ".merge-patch-stat-sample"))
with open(paths_file, encoding="utf-8") as handle:
    paths = [line for line in handle.read().split("\n") if line]
files = [path for path in paths if path != MUTATION_TABLE]
if os.environ.get("MERGE_AND_VERIFY_BREAK") == "merge-patch-single-directory" and directories:
    single_directory_paths = set(library.patch_directory_paths(directories[0]))
    files = [path for path in files if path in single_directory_paths]
if os.path.lexists(out):
    raise SystemExit(f"{out} 已经在，不覆盖")

def table_rows(path):
    """变异表里的行：[(名字, 整行)]，空行与 # 起头的行不算；没有这份表交空。"""
    if not os.path.isfile(path):
        return []
    with open(path, encoding="utf-8") as handle:
        return [(line.split("\t", 1)[0], line) for line in handle.read().split("\n") if line.strip() and not line.startswith("#")]

staging = tempfile.mkdtemp(prefix=".merge-patch.", dir=os.path.dirname(out))
try:
    work, result = os.path.join(staging, "work"), os.path.join(staging, "patch")
    os.makedirs(work)
    os.makedirs(result)

    def git(*arguments):
        done = subprocess.run(["git", "-c", "core.autocrlf=false", "-c", "core.fileMode=true", *(["-c", "core.trustctime=false"] if STAT_CACHE_BROKEN else []), "-C", work, *arguments],
                              capture_output=True, stdin=subprocess.DEVNULL)
        if done.returncode != 0:
            raise SystemExit(f"git {' '.join(arguments[:2])} 退 {done.returncode}：{done.stderr.decode('utf-8', 'replace')[-300:]}")
        return done.stdout

    def place(source, relative):
        target = os.path.join(work, relative)
        os.makedirs(os.path.dirname(target), exist_ok=True)
        shutil.copy2(source, target)

    def place_after(source, relative):
        """补丁之后的那一份换进临时仓。基线那一份进索引时 git 记下了它的 inode、大小、修改时刻与改变时刻，git diff 这几样都对得上就不读内容：
        先删后建，文件系统常把刚释放的 inode 又发回来，大小与修改时刻（拷的是快照里的，同一秒）相同的改动就被漏掉。
        所以先写另一个临时文件、再改名换上：旧的那份此刻还在，新的 inode 一定不同。"""
        target = os.path.join(work, relative)
        if STAT_CACHE_BROKEN and os.path.isfile(target):
            with open(source, "rb") as reader, open(target, "wb") as writer:
                shutil.copyfileobj(reader, writer)
            shutil.copystat(source, target)
            return
        staged = os.path.join(staging, "after")   # 临时仓之外、同一个文件系统里：改名是原子的，也不会撞上补丁点到的某个路径
        os.makedirs(os.path.dirname(target), exist_ok=True)
        shutil.copy2(source, staged)
        os.replace(staged, target)

    git("init", "-q")
    before = [path for path in files if os.path.isfile(os.path.join(baseline, path))]
    for relative in before:
        place(os.path.join(baseline, relative), relative)
    if before:
        git("add", "-f", "--", *before)
    created = []
    for relative in files:
        source = os.path.join(tree, relative)
        if os.path.isfile(source) and not os.path.islink(source):
            place_after(source, relative)
            if relative not in before:
                created.append(relative)
        elif os.path.lexists(os.path.join(work, relative)):
            os.remove(os.path.join(work, relative))
    if created:
        git("add", "-f", "-N", "--", *created)
    patch = git("diff", "--binary", "--full-index", "--no-renames", "--no-color", "--no-ext-diff", "--src-prefix=a/", "--dst-prefix=b/") if files else b""
    changed_files = patch.count(b"\ndiff --git ") + (1 if patch.startswith(b"diff --git ") else 0)
    if patch:
        with open(os.path.join(result, "crates.patch"), "wb") as handle:
            handle.write(patch)
        for relative in before:
            target = os.path.join(result, "baseline", relative)
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy2(os.path.join(baseline, relative), target)
    mutation_counts = {"mutations-append.tsv": 0, "mutations-replacements.tsv": 0, "mutations-delete.txt": 0}
    if MUTATION_TABLE in paths:
        old_rows, new_rows = table_rows(os.path.join(baseline, MUTATION_TABLE)), table_rows(os.path.join(tree, MUTATION_TABLE))
        old_by_name, new_by_name = dict(old_rows), dict(new_rows)
        # 替换与删除带前像（基线里同名那一行原样）：apply-writer-patch.py 打之前逐行比对主工作区此刻那一行，对不上整份拒（它文件头「补丁目录里的文件」）
        preimage_dropped = os.environ.get("MERGE_AND_VERIFY_BREAK") == "merge-patch-preimage-dropped"
        sections = {"mutations-append.tsv": [line for name, line in new_rows if name not in old_by_name],
                    "mutations-replacements.tsv": [line if preimage_dropped else f"{line}\t{old_by_name[name]}"
                                                   for name, line in new_rows if name in old_by_name and old_by_name[name] != line],
                    "mutations-delete.txt": [name if preimage_dropped else line for name, line in old_rows if name not in new_by_name]}
        for file_name, lines in sections.items():
            mutation_counts[file_name] = len(lines)
            if lines:
                with open(os.path.join(result, file_name), "w", encoding="utf-8") as handle:
                    handle.write("".join(line + "\n" for line in lines))
    with open(os.path.join(result, "result.md"), "w", encoding="utf-8") as handle:
        handle.write("# 合进主工作区的那一份补丁目录\n\n照给的次序合起来的补丁目录：" + "、".join(directories) + "\n")
        for directory in directories:
            report = os.path.join(directory, "result.md")
            handle.write(f"\n# 补丁目录 {directory} 的报告\n\n")
            if os.path.isfile(report):
                with open(report, encoding="utf-8", errors="replace") as source:
                    handle.write(source.read().rstrip("\n") + "\n")
            else:
                handle.write("（这份补丁目录里没有 result.md）\n")
    os.rename(result, out)
finally:
    shutil.rmtree(staging, ignore_errors=True)
print(f"合起来改 {changed_files} 份文件，变异行追加 {mutation_counts['mutations-append.tsv']}、换 {mutation_counts['mutations-replacements.tsv']}、删 {mutation_counts['mutations-delete.txt']}")
PY_MERGE_PATCH
)"
# 变异锚点核的 python 段（文件头「先验后合」变异锚点核，anchor_check_new_breaks 起）：python3 - <apply-writer-patch.py> <快照> <原样目录> <路径单子> <合进主工作区的补丁目录>，
#   不另写判法：摆出补丁之前的树（快照此刻的拷贝范围，补丁动到的、落在拷贝范围里的文件换回原样目录里的原样，补丁新建的删掉），把 merge-patch 的 crates.patch 里落在拷贝范围里的文件段
#   与快照此刻的变异表交给 apply-writer-patch.py 的 static_check_report（补丁之前的树当基线，补丁之后有、基线上没有的红行才算），把它的拒绝与提醒原样打出来；
#   退 0 没有新红的，1 有新红或 static_check_report 判不了，2 补丁之前的树摆不出来。MERGE_AND_VERIFY_STATIC_CHECK 设了的，static-check 起它（只给自证）。
#   新红的行全数打出（static_check_report 的 shown 给 None）。弄坏开关 anchor-own-table-not-restored：变异表不换回原样；anchor-created-files-kept：补丁新建的文件不删；
#   anchor-check-log-truncated：新红只打前 20 处
MERGE_ANCHOR_CHECK_PROGRAM="$(cat <<'PY_MERGE_ANCHOR_CHECK'
import importlib.util
import os
import re
import shutil
import sys
import tempfile

specification = importlib.util.spec_from_file_location("apply_writer_patch_for_anchor_check", sys.argv[1])
apply_module = importlib.util.module_from_spec(specification)
tree, baseline, paths_file, merge_patch = sys.argv[2:6]
MUTATION_TABLE = "crates/mutations.tsv"
broken = os.environ.get("MERGE_AND_VERIFY_BREAK", "")
scratch = tempfile.mkdtemp(prefix="merge-anchor-check-")

def judge_new_breaks():
    specification.loader.exec_module(apply_module)
    with open(paths_file, encoding="utf-8") as handle:
        paths = [line for line in handle.read().split("\n") if line]
    items = apply_module.static_check_tree_items(tree)

    def inside(relative):
        return any(relative == item or relative.startswith(item + "/") for item in items)

    # 补丁之前的树：快照此刻的拷贝范围，补丁动到的文件换回原样、补丁新建的删掉
    before = os.path.join(scratch, "before")
    apply_module.repository_copy_module().copy_entries(os.path.realpath(tree), before, items)
    for relative in paths:
        if not inside(relative):
            continue
        if relative == MUTATION_TABLE and broken == "anchor-own-table-not-restored":
            continue
        original, target = os.path.join(baseline, relative), os.path.join(before, relative)
        if os.path.isfile(original):
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy2(original, target)
        elif (os.path.isfile(target) or os.path.islink(target)) and broken != "anchor-created-files-kept":
            os.remove(target)
    # merge-patch 的 crates.patch 只收落在拷贝范围里的文件段：范围外的文件临时树里没有，套不上，也不影响变异行
    patch_path, dropped = None, 0
    source_patch = os.path.join(merge_patch, "crates.patch")
    if os.path.isfile(source_patch):
        with open(source_patch, "rb") as handle:
            sections = [part for part in re.split(rb"(?m)^(?=diff --git )", handle.read()) if part.strip()]
        kept = []
        for section in sections:
            rest = section.split(b"\n", 1)[0].decode("utf-8", "surrogateescape")[len("diff --git "):]
            file_path = None if rest.startswith('"') else rest[2:2 + (len(rest) - 5) // 2]   # 「a/<路径> b/<路径>」，不认改名（merge-patch 是 --no-renames 出的）
            if file_path is None or inside(file_path):
                kept.append(section)
            else:
                dropped += 1
        if kept:
            patch_path = os.path.join(scratch, "crates.patch")
            with open(patch_path, "wb") as handle:
                handle.write(b"".join(kept))
    merged = ""
    if os.path.isfile(os.path.join(tree, MUTATION_TABLE)):
        with open(os.path.join(tree, MUTATION_TABLE), encoding="utf-8") as handle:
            merged = handle.read()
    replacement_command = os.environ.get("MERGE_AND_VERIFY_STATIC_CHECK")
    static_check = (lambda checked_tree: [sys.executable or "python3", replacement_command, "static-check", checked_tree]) if replacement_command else None
    refusal, notes = apply_module.static_check_report(before, merged, patch_path, static_check, shown=20 if broken == "anchor-check-log-truncated" else None)
    for line in refusal:
        if not line.startswith("→ 怎么办"):   # 出路由调用方写（static_check_report 的出路说的是 --dry-run）
            print(line)
    for line in notes:
        print(line)
    if dropped:
        print(f"· 合起来的补丁里 {dropped} 份文件落在拷贝范围之外，没带进核（变异行读不到它们）")
    return 1 if refusal else 0

try:
    code = judge_new_breaks()
except (OSError, ValueError, RuntimeError, ImportError) as error:
    print(f"变异锚点核没做成：{error}")
    code = 2
finally:
    shutil.rmtree(scratch, ignore_errors=True)
sys.exit(code)
PY_MERGE_ANCHOR_CHECK
)"
# 变异表倒回补丁之前的 python 段（文件头「先验后合」变异锚点核，带 --skip-apply 的那一趟起）：python3 - <快照的变异表> <写出的变异表> <补丁目录…>，补丁目录写成倒着的次序（后打的在前）。
#   补丁已打在要验的树上，变异表照各份补丁目录里的变异表文件倒着摆回补丁之前：mutations-append.tsv 的行逐字从表里去掉，mutations-replacements.tsv 的改后（前六段）
#   换回前像（后六段），mutations-delete.txt 的行（前像）接回表尾；表里找不到追加的行或改后的行、删掉的行名在表里还在的，都是倒不回（之后被别的改动动过），
#   一处一行点名、写进标准输出、退 1、不写出表。倒回的表只用来当变异锚点核的基线，行的先后与补丁之前不同不影响核（红行比基线时去掉行号）。
#   弄坏开关 anchor-skip-replacement-not-reversed：mutations-replacements.tsv 的改后行不换回前像（补丁之前的表里还是改后的行）
MERGE_TABLE_REVERSE_PROGRAM="$(cat <<'PY_MERGE_TABLE_REVERSE'
import os
import sys

table_path, out_path = sys.argv[1:3]
directories = sys.argv[3:]
broken = os.environ.get("MERGE_AND_VERIFY_BREAK", "")


def patch_rows(directory, name):
    """补丁目录里变异表文件的行（空行与 # 起头的行不算）；没有这份文件交空。"""
    path = os.path.join(directory, name)
    if not os.path.isfile(path):
        return []
    with open(path, encoding="utf-8") as handle:
        return [line for line in handle.read().split("\n") if line.strip() and not line.startswith("#")]


def row_name(line):
    return line.split("\t", 1)[0]


with open(table_path, encoding="utf-8") as handle:
    lines = handle.read().split("\n")
if lines and lines[-1] == "":
    lines.pop()
problems = []
for directory in directories:
    for line in patch_rows(directory, "mutations-append.tsv"):
        if line in lines:
            lines.remove(line)
        else:
            problems.append(f"{directory} 追加的变异行 {row_name(line)} 在要验的树的变异表里找不到原样（补丁之后被改过或删了）")
    for line in patch_rows(directory, "mutations-replacements.tsv"):
        if broken == "anchor-skip-replacement-not-reversed":
            continue
        columns = line.split("\t")
        if len(columns) != 12:
            problems.append(f"{directory} 的 mutations-replacements.tsv 里 {row_name(line)} 不是十二段")
            continue
        replaced, preimage = "\t".join(columns[:6]), "\t".join(columns[6:])
        if replaced in lines:
            lines[lines.index(replaced)] = preimage
        else:
            problems.append(f"{directory} 换掉的变异行 {row_name(line)} 在要验的树的变异表里找不到改后的原样（补丁之后被改过或删了）")
    for line in patch_rows(directory, "mutations-delete.txt"):
        if any(row_name(existing) == row_name(line) for existing in lines if existing.strip() and not existing.startswith("#")):
            problems.append(f"{directory} 删掉的变异行 {row_name(line)} 在要验的树的变异表里还在（补丁之后又加回来了）")
        else:
            lines.append(line)
if problems:
    print("变异表倒不回补丁之前：" + "；".join(problems))
    sys.exit(1)
with open(out_path, "w", encoding="utf-8") as handle:
    handle.write("".join(line + "\n" for line in lines))
PY_MERGE_TABLE_REVERSE
)"
# 开跑那一刻的指纹的 python 段（文件头「开跑那一刻的指纹」）：python3 - <lib_rebuilt_packages.py> <要验的树> <指纹文件> <起点文件> <变异行文件> <做法> <参数…>：
#   record <补丁目录…>：起点照补丁改到的路径算（包目录连同依赖关系上的包与树根那几样、不落在包里的单份文件；登记表与 crates/mutations.tsv 不算），
#     要换掉、删掉的变异行照补丁目录里 mutations-replacements.tsv 的第一段与 mutations-delete.txt 的名字在表里取原样；
#   recheck <那一趟的指纹文件> <那一趟的起点文件> <那一趟的变异行文件>：起点与变异行名取那一趟记的，现算一遍写进这一趟的三份，与那一趟的指纹、变异行逐字节比，对不上退 3。
#   每份普通文件一行 sha256sum 的写法、按字节排序；末行打一句起点、文件与变异行各几个（recheck 对不上时打变了几行）。
#   弄坏开关 main-tree-fingerprint-touched-files-only：起点只取补丁改到的那几份文件；fingerprint-dependencies-skipped：不加依赖关系上的包与树根那几样
MAIN_TREE_FINGERPRINT_PROGRAM="$(cat <<'PY_MAIN_TREE_FINGERPRINT'
import glob
import hashlib
import importlib.util
import os
import sys
import tomllib

specification = importlib.util.spec_from_file_location("lib_rebuilt_packages", sys.argv[1])
library = importlib.util.module_from_spec(specification)
specification.loader.exec_module(library)
# 两种起法：<lib> <树> <指纹> <起点> <变异行> record|recheck …（开跑那一刻的指纹）与 <lib> <树> build-packages <补丁目录…>（build 那一步要编的包，文件头 build 一段）
if sys.argv[3:4] == ["build-packages"]:
    tree, mode, arguments = sys.argv[2], "build-packages", sys.argv[4:]
else:
    tree, fingerprint_path, roots_path, rows_path, mode = sys.argv[2:7]
    arguments = sys.argv[7:]
MUTATION_TABLE = "crates/mutations.tsv"
MUTATION_NAME_FILES = ("mutations-replacements.tsv", "mutations-delete.txt")   # 这一批要换掉、删掉的变异行的名字从这两份取
DEPENDENCY_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")
WORKSPACE_ROOT_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo")   # 有包目录时另加的树根几样（在的才算）
registration_tables = {line for line in os.environ.get("MERGE_AND_VERIFY_REGISTRATION_TABLE_PATHS", "").split("\n") if line}
broken = os.environ.get("MERGE_AND_VERIFY_BREAK", "")

def skipped(name, is_directory):
    """快照不拷的同一组：任意一层的 .git，名叫 target、target-*、__pycache__ 的目录（与核对那一条的 find -prune 同一组）。"""
    if name == ".git":
        return True
    return is_directory and (name in ("target", "__pycache__") or name.startswith("target-"))

def regular_file(path):
    return os.path.isfile(path) and not os.path.islink(path)

def manifest(directory):
    """directory/Cargo.toml 读成的表；没有这份文件交 None。读不了、不是合法的 TOML 照抛（整条退 1）。"""
    path = os.path.join(directory, "Cargo.toml")
    if not os.path.isfile(path):
        return None
    with open(path, "rb") as handle:
        return tomllib.load(handle)

def package_name(directory):
    data = manifest(directory)
    name = data.get("package", {}).get("name") if data else None
    return name if isinstance(name, str) and name else None


def package_directory(relative):
    """路径所在目录往上第一份带 [package] 的 Cargo.toml 所在的目录；到树根还没有交 None。"""
    directory = os.path.dirname(os.path.join(tree, relative))
    while True:
        if package_name(directory):
            return directory
        if directory == tree or os.path.dirname(directory) == directory:
            return None
        directory = os.path.dirname(directory)


def workspace_dependency_entries():
    """树根 Cargo.toml 的 [workspace.dependencies] 那一张表（{依赖名: 那一项}）；没有或不是表交空表。"""
    data = manifest(tree) or {}
    workspace = data.get("workspace")
    table = workspace.get("dependencies") if isinstance(workspace, dict) else None
    return table if isinstance(table, dict) else {}


def patch_paths():
    """树根 Cargo.toml 每个 [patch.<源>] 里写了 path 的项 → {被换掉的包名: 归一的绝对目录}（path 相对树根；项里写了 package 的按它认包名，没写按键名）。"""
    data = manifest(tree) or {}
    sources = data.get("patch")
    found = {}
    for table in sources.values() if isinstance(sources, dict) else []:
        for key, entry in table.items() if isinstance(table, dict) else []:
            if isinstance(entry, dict) and isinstance(entry.get("path"), str):
                name = entry["package"] if isinstance(entry.get("package"), str) else key
                found[name] = os.path.normpath(os.path.join(tree, entry["path"]))
    return found


def path_dependencies(directory):
    """包目录的 Cargo.toml 里 dependencies、dev-dependencies、build-dependencies（连同 target.<平台> 下这三张）指到一个包目录的依赖：
    写了 path 的照它（相对包目录）；写成 { workspace = true } 的照树根 [workspace.dependencies] 里同名那一项（那一项写了 path 的照它、相对树根）；
    别的（registry、git 来源，写成版本串的也算）照依赖的包名（项里 package 改的名字，没写是键名）在树根 [patch.<源>] 里写了 path 的那一项认（patch_paths）。
    弄坏开关 workspace-dependencies-ignored：{ workspace = true } 的不认；patch-paths-ignored：不读 [patch]。"""
    data = manifest(directory) or {}
    tables = [data.get(name) for name in DEPENDENCY_TABLES]
    platforms = data.get("target")
    if isinstance(platforms, dict):
        tables += [platform.get(name) for platform in platforms.values() if isinstance(platform, dict) for name in DEPENDENCY_TABLES]
    inherited = workspace_dependency_entries() if broken != "workspace-dependencies-ignored" else {}
    patched = patch_paths() if broken != "patch-paths-ignored" else {}
    found = set()
    for table in tables:
        if not isinstance(table, dict):
            continue
        for key, entry in table.items():
            base = directory
            if isinstance(entry, dict) and entry.get("workspace") is True:
                if key not in inherited:
                    continue
                entry, base = inherited[key], tree
            if isinstance(entry, dict) and isinstance(entry.get("path"), str):
                candidate = os.path.normpath(os.path.join(base, entry["path"]))
            else:
                name = entry["package"] if isinstance(entry, dict) and isinstance(entry.get("package"), str) else key
                if name not in patched:
                    continue
                candidate = patched[name]
            if package_name(candidate):
                found.add(candidate)
    return found


def workspace_members():
    """树根 Cargo.toml 的 [workspace] members（通配照展开）里带 [package] 的目录。"""
    data = manifest(tree) or {}
    patterns = data.get("workspace", {}).get("members", [])
    found = set()
    for pattern in patterns if isinstance(patterns, list) else []:
        for candidate in glob.glob(os.path.join(tree, pattern)):
            candidate = os.path.normpath(candidate)
            if os.path.isdir(candidate) and package_name(candidate):
                found.add(candidate)
    return found


def closure(starts, neighbours):
    """从 starts 起顺着 neighbours 走得到的全部（含 starts）。"""
    seen, pending = set(starts), list(starts)
    while pending:
        for found in neighbours(pending.pop()):
            if found not in seen:
                seen.add(found)
                pending.append(found)
    return seen


def files_under(start):
    """一个起点底下照 find -P 会列的普通文件：起点是符号链接的不列；目录照 skipped 剪枝，不进符号链接的目录。"""
    if os.path.islink(start):
        return []
    if os.path.isdir(start):
        if skipped(os.path.basename(start), True):
            return []
        found = []
        for current, directories, names in os.walk(start):
            directories[:] = [name for name in directories if not skipped(name, True)]
            for name in names:
                path = os.path.join(current, name)
                if not skipped(name, False) and regular_file(path):
                    found.append(path)
        return found
    if regular_file(start) and not skipped(os.path.basename(start), False):
        return [start]
    return []


def mutation_rows(names):
    """要验的树 crates/mutations.tsv 里第一段在 names 里的行（原样字节，去重、按字节排序）。"""
    table = os.path.join(tree, MUTATION_TABLE)
    if not names or not os.path.isfile(table):
        return []
    with open(table, "rb") as handle:
        lines = handle.read().split(b"\n")
    return sorted({line for line in lines if line and not line.startswith(b"#") and line.split(b"\t", 1)[0].decode("utf-8", "replace") in names})


def read_lines(path):
    with open(path, "rb") as handle:
        return [line for line in handle.read().split(b"\n") if line]


def touched_packages(directories, by_package=True):
    """补丁目录里 crates.patch 改到的路径（新旧两侧；登记表与变异表不算）→（包目录集合, 不落在包里、此刻在的普通文件）。
    by_package 为假时一律当不落在包里（指纹的弄坏开关 main-tree-fingerprint-touched-files-only）。"""
    packages, loose = set(), []
    for directory in directories:
        for relative in library.patch_directory_paths(directory):
            if relative in registration_tables or relative == MUTATION_TABLE:
                continue
            package = package_directory(relative) if by_package else None
            if package is not None:
                packages.add(package)
            elif regular_file(os.path.join(tree, relative)):
                loose.append(os.path.join(tree, relative))
    return packages, loose


def dependents_of(packages):
    """packages 连同工作区里照 path 依赖传递依赖它们的包（在工作区成员与 packages 里找）。"""
    needs_of = {member: path_dependencies(member) for member in workspace_members() | packages}
    return closure(packages, lambda directory: {member for member, needs in needs_of.items() if directory in needs})


if mode == "build-packages":
    # build 那一步要编的包（文件头 build 一段）：改到的包与传递依赖它们的包，只留工作区成员，一行一个包名、按包名排序
    packages, _ = touched_packages(arguments)
    members = workspace_members()
    names = sorted({package_name(directory) for directory in dependents_of(packages) if directory in members})
    sys.stdout.write("".join(name + "\n" for name in names))
    sys.exit(0)
if mode == "record":
    patch_directories = arguments
    names = set()
    packages, starts = touched_packages(patch_directories, by_package=broken != "main-tree-fingerprint-touched-files-only")
    for directory in patch_directories:
        for file_name in MUTATION_NAME_FILES:
            path = os.path.join(directory, file_name)
            if os.path.isfile(path):
                with open(path, encoding="utf-8") as handle:
                    names |= {line.split("\t", 1)[0] for line in handle.read().split("\n") if line.strip() and not line.startswith("#")}
    if packages and broken != "fingerprint-dependencies-skipped":
        packages = closure(packages, path_dependencies) | dependents_of(packages)
        starts += [os.path.join(tree, name) for name in WORKSPACE_ROOT_ENTRIES if os.path.lexists(os.path.join(tree, name))]
    starts += sorted(packages)
    rows = mutation_rows(names)
elif mode == "recheck" and len(arguments) == 3:
    recorded_fingerprint, recorded_roots, recorded_rows = arguments
    starts = [os.fsdecode(line) for line in read_lines(recorded_roots)]
    rows = mutation_rows({line.split(b"\t", 1)[0].decode("utf-8", "replace") for line in read_lines(recorded_rows)})
else:
    raise SystemExit(f"认不出的做法 {mode}（只认 record <补丁目录…>、recheck <指纹> <起点> <变异行> 与 <树> build-packages <补丁目录…>）")
unique = sorted(set(starts), key=os.fsencode)
kept = [start for start in unique if not any(start.startswith(other + os.sep) for other in unique if other != start)]
files = sorted({path for start in kept for path in files_under(start)}, key=os.fsencode)
with open(fingerprint_path, "wb") as handle:
    for path in files:
        digest = hashlib.sha256()
        with open(path, "rb") as source:
            for block in iter(lambda: source.read(1 << 20), b""):
                digest.update(block)
        name = os.fsencode(path)
        if b"\\" in name or b"\n" in name:   # sha256sum 对带反斜杠或换行的名字的写法：行首一个反斜杠、名字里的两样转义
            handle.write(b"\\" + digest.hexdigest().encode() + b"  " + name.replace(b"\\", b"\\\\").replace(b"\n", b"\\n") + b"\n")
        else:
            handle.write(digest.hexdigest().encode() + b"  " + name + b"\n")
with open(roots_path, "wb") as handle:
    for start in kept:
        handle.write(os.fsencode(start) + b"\n")
with open(rows_path, "wb") as handle:
    handle.write(b"".join(line + b"\n" for line in rows))
if mode == "recheck":
    changed = []
    for new_path, old_path in ((fingerprint_path, recorded_fingerprint), (rows_path, recorded_rows)):
        changed += sorted(set(read_lines(new_path)) ^ set(read_lines(old_path)))
    if changed:
        shown = "；".join(line.decode("utf-8", "replace")[-160:] for line in changed[:5])
        print(f"开跑以来主工作区变了：指纹与变异行两份里多出或少了 {len(changed)} 行（头几行：{shown}）")
        sys.exit(3)
print(f"{len(kept)} 个起点、{len(files)} 份文件、{len(rows)} 行要换掉删掉的变异行，写进 {fingerprint_path}")
PY_MAIN_TREE_FINGERPRINT
)"

if [[ "${1:-}" == "--selftest" ]]; then selftest; exit $?; fi
THREADS="$(heavy_thread_default)" || exit 2; skip_apply=0; dirs=(); crash_expand_up_to_argument=""; crash_expand_up_to_given=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-apply) skip_apply=1; shift ;;
    --crash-expand-up-to)
      # 文件头「验证包那一步」：缩崩溃域的展开深度上限，值是非负整数，下面统一核
      [[ $# -ge 2 ]] || die "--crash-expand-up-to 后面要跟一个非负整数（展开深度上限，只往小缩），一样都没做" "写成 --crash-expand-up-to 6 这样的两个词"
      crash_expand_up_to_argument="$2"; crash_expand_up_to_given=1; shift 2 ;;
    # 逐项点名（文件头「逐项点名与按条复用」）
    --list-items) LIST_ITEMS=1; shift ;;
    --with-keys) LIST_WITH_KEYS=1; shift ;;
    # 判不出测试目标的留给提交时整轮门禁（文件头「判不出的留给提交时整轮门禁」）：只在又点了 --item 时生效
    "$UNRESOLVED_OPTION") LEAVE_UNRESOLVED_TO_COMMIT=1; shift ;;
    --item)
      [[ $# -ge 2 && -n "$2" && "$2" != -* ]] || die "--item 后面要跟一项（<包>:<目标根文件> 或 <包>:doc），一样都没做" "写成 --item singlefs-core:src/lib.rs 这样的两个词；可点的项用 --list-items 看"
      NAMED_ITEMS+=("$2"); shift 2 ;;
    --item=*)
      [[ -n "${1#--item=}" ]] || die "--item= 后面要跟一项（<包>:<目标根文件> 或 <包>:doc），一样都没做" "写成 --item=singlefs-core:src/lib.rs；可点的项用 --list-items 看"
      NAMED_ITEMS+=("${1#--item=}"); shift ;;
    --fingerprint-of)
      # 文件头「开跑那一刻的指纹」：那一趟的日志目录，下面开跑时核它记的指纹
      [[ $# -ge 2 && -d "$2" ]] || die "--fingerprint-of 后面要跟那一趟的日志目录（在的目录），一样都没做" "写成 --fingerprint-of <那一趟汇总行里写的日志目录>；只重跑红的那几项的出路自己带上它，照原样交"
      FINGERPRINT_OF="$(cd "$2" && pwd)"; shift 2 ;;
    --*)
      # 弄坏开关 unknown-option-accepted：认不出的选项也照收、当没给（不拒）
      if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "unknown-option-accepted" ]]; then shift; continue; fi
      die "认不出参数 $1" "只认 --skip-apply、--crash-expand-up-to <n>、--item <项>、--fingerprint-of <日志目录>、$UNRESOLVED_OPTION、--list-items [--with-keys]（整包跑归提交时的整轮门禁；提交之外要整包跑先弹窗问用户）" ;;
    *) dirs+=("$1"); shift ;;
  esac
done
((LIST_WITH_KEYS == 0 || LIST_ITEMS)) || die "--with-keys 只跟 --list-items 一起用，一样都没做" "写成 --list-items --with-keys [同一趟的别的参数…]"
[[ -z "$FINGERPRINT_OF" || $skip_apply -eq 0 ]] || die "--fingerprint-of 只跟不带 --skip-apply 的那一趟一起用，一样都没做" "带 --skip-apply 的那一趟补丁已在主工作区上、不合，去掉 --fingerprint-of"
# 缩崩溃域（文件头「验证包那一步」）：参数给了取参数，没给取环境里已设的变量；值要是非负整数（弄坏开关 expand-up-to-unchecked：不核），写坏退 2、一样都不做
if ((crash_expand_up_to_given)); then CRASH_EXPAND_UP_TO="$crash_expand_up_to_argument"; crash_expand_up_to_origin="--crash-expand-up-to"
elif [[ -n "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO+x}" ]]; then CRASH_EXPAND_UP_TO="$SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO"; crash_expand_up_to_origin="环境变量 $CRASH_EXPAND_UP_TO_VARIABLE"; crash_expand_up_to_given=1
fi
if ((crash_expand_up_to_given)) && [[ ! "$CRASH_EXPAND_UP_TO" =~ ^[0-9]+$ ]] && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "expand-up-to-unchecked" ]]; then
  die "$crash_expand_up_to_origin 的值「$CRASH_EXPAND_UP_TO」不是非负整数（展开深度上限，只往小缩），一样都没做" "写成 --crash-expand-up-to 6 这样；不缩就别给，也别在环境里设 $CRASH_EXPAND_UP_TO_VARIABLE"
fi
SOURCE_TREE="$(cd "${MERGE_AND_VERIFY_TREE:-$root}" && pwd)" || die "进不去要验的树 ${MERGE_AND_VERIFY_TREE:-$root}" "在仓里跑；MERGE_AND_VERIFY_TREE 只给自证用，平时不设"
tree="$SOURCE_TREE"
# 补丁目录转成绝对路径（之后进快照里干活，相对路径就指错了）；算改过的包也用这一份（文件头「改过的包都重编了没有」，--skip-apply 时也留着）
absolute_dirs=()
for dir in ${dirs[@]+"${dirs[@]}"}; do
  if [[ -d "$dir" ]]; then absolute_dirs+=("$(cd "$dir" && pwd)"); else absolute_dirs+=("$dir"); fi
done
dirs=(${absolute_dirs[@]+"${absolute_dirs[@]}"}); CHECK_PATCH_DIRS=(${absolute_dirs[@]+"${absolute_dirs[@]}"})
# --list-items（文件头「逐项点名与按条复用」）：不建日志目录、不拷快照、不编，列完就退
((LIST_ITEMS == 0)) || list_items_and_exit
[[ ${#dirs[@]} -gt 0 || $skip_apply -eq 1 ]] || die "没有给补丁目录" "task-server/task submit -- bash scripts/gate/merge-and-verify.sh <补丁目录> …；只验不打用 --skip-apply"
read_slot_table   # 槽位登记（文件头「槽位」）：读不出、写坏退 1，日志目录都不建
if [[ -n "${MERGE_AND_VERIFY_LOG_DIRECTORY:-}" ]]; then logs="$MERGE_AND_VERIFY_LOG_DIRECTORY"
elif work_root="$(python3 "$root/.claude/hooks/lib_work_root.py")"; then logs="$work_root/merge-and-verify/$(TZ=Asia/Tokyo date +%F)-$$"
else die "取不到工作根（.claude/hooks/lib_work_root.py 跑不起来）" "恢复那份库，或设 MERGE_AND_VERIFY_LOG_DIRECTORY 指到日志目录"; fi
# 这一趟的编译目录（文件头）与快照：是这里建的才登记进 RUN_TARGET / SNAPSHOT_TREE，收尾的 EXIT trap 只删登记了的
[[ ! -e "$logs/repo" ]] || die "快照位置 $logs/repo 已经有东西，不往里铺、也不删它" "换一个日志目录（MERGE_AND_VERIFY_LOG_DIRECTORY 不设就是新的 <日期>-<进程号>），或先看清那里是什么"
mkdir -p "$logs" && logs="$(cd "$logs" && pwd)" || die "建不了日志目录 $logs" "看工作根在不在、可不可写"
[[ "$(cd "$logs" && pwd -P)/" != "$(cd "$SOURCE_TREE" && pwd -P)/"* ]] || die "日志目录 $logs 在要验的树 $SOURCE_TREE 里面：快照会把自己拷进自己，什么都没拷" \
  "MERGE_AND_VERIFY_LOG_DIRECTORY 指到树外（不设就是工作根下的 merge-and-verify/<日期>-<进程号>）"
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
# 记下这一趟的任务号与尝试号，再删同一任务更早的尝试留下的快照（文件头「上一次尝试留下的快照」）：在拷快照之前，腾出盘再拷
record_task_attempt "$logs"
remove_earlier_attempt_snapshots "$logs"
# 开跑那一刻的指纹（文件头「先验后合」「开跑那一刻的指纹」）：不带 --skip-apply 的那一趟，拷快照之前记，合进主工作区那一段的核拿它比。
#   给了 --fingerprint-of 的先照那一趟记的起点与变异行现算、与那一趟的逐字节比（弄坏开关 fingerprint-of-unchecked：不核），对不上退 2、一个补丁都不打
if ((skip_apply == 0)); then
  if [[ -n "$FINGERPRINT_OF" && "${MERGE_AND_VERIFY_BREAK:-}" != "fingerprint-of-unchecked" ]]; then
    for recorded_name in "$MAIN_TREE_FINGERPRINT_NAME" "$MAIN_TREE_ROOTS_NAME" "$MAIN_TREE_MUTATION_ROWS_NAME"; do
      [[ -f "$FINGERPRINT_OF/$recorded_name" ]] || die "--fingerprint-of 给的 $FINGERPRINT_OF 里没有 $recorded_name（不是不带 --skip-apply 的那一趟的日志目录），一个补丁都没打、没编" \
        "照原命令去掉 --fingerprint-of、--item 照 --list-items 对这一批补丁目录重列，整批重交（键相同的照复用）" 2
    done
    recheck_rc=0
    python3 - "$REBUILT" "$SOURCE_TREE" "$logs/main-tree-recheck.sha256" "$logs/main-tree-recheck.roots" "$logs/main-tree-recheck.mutation-rows" recheck \
      "$FINGERPRINT_OF/$MAIN_TREE_FINGERPRINT_NAME" "$FINGERPRINT_OF/$MAIN_TREE_ROOTS_NAME" "$FINGERPRINT_OF/$MAIN_TREE_MUTATION_ROWS_NAME" \
      <<<"$MAIN_TREE_FINGERPRINT_PROGRAM" > "$logs/main-tree-recheck.log" 2>&1 || recheck_rc=$?
    case "$recheck_rc" in
      0) echo "  核过那一趟（$FINGERPRINT_OF）开跑以来主工作区在它的起点上没变：$(tail -n 1 "$logs/main-tree-recheck.log")" ;;
      3) die "那一趟（$FINGERPRINT_OF）$(tail -n 1 "$logs/main-tree-recheck.log")（日志 $logs/main-tree-recheck.log），一个补丁都没打、没编" \
           "那一趟判绿的那几项是在变之前的主工作区上绿的，不能接着用：照原命令去掉 --fingerprint-of、--item 照 --list-items 对这一批补丁目录重列，整批重交（键相同的照复用）" 2 ;;
      *) die "核不成那一趟的指纹（日志 $logs/main-tree-recheck.log），一个补丁都没打、没编" "照那份日志修（那一趟的日志目录读不读得了、包里的 Cargo.toml 与文件读不读得了）再跑" 1 ;;
    esac
  fi
  python3 - "$REBUILT" "$SOURCE_TREE" "$logs/$MAIN_TREE_FINGERPRINT_NAME" "$logs/$MAIN_TREE_ROOTS_NAME" "$logs/$MAIN_TREE_MUTATION_ROWS_NAME" record ${dirs[@]+"${dirs[@]}"} \
    <<<"$MAIN_TREE_FINGERPRINT_PROGRAM" > "$logs/main-tree-at-start.log" 2>&1 \
    || die "记不成开跑那一刻主工作区的指纹（日志 $logs/main-tree-at-start.log），一个补丁都没打、没编" "照那份日志修（补丁目录读不读得了、包里的 Cargo.toml 与文件读不读得了）再跑" 1
  echo "  记下开跑那一刻的指纹：$(tail -n 1 "$logs/main-tree-at-start.log")"
fi
# 快照（文件头「快照」「槽位」）：开跑这一刻拷，之后打补丁、编、验都在它上面；占到槽的拷进槽，没占到的一趟一份拷进日志目录。弄坏开关 verify-in-main-tree：不拷、不占槽、都在要验的树上
if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verify-in-main-tree" ]]; then
  echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=verify-in-main-tree：不拷快照，打补丁、编、验都在要验的树 $tree 上"
else
  choose_slot "$logs"
  if [[ -n "$SLOT_DIRECTORY" ]]; then
    SNAPSHOT_TREE="$SLOT_DIRECTORY/repo"
    refresh_slot_snapshot "$SOURCE_TREE" "$SNAPSHOT_TREE" "$logs/$SLOT_CHANGED_SOURCES_NAME" > "$logs/snapshot.log" 2>&1 \
      || die "拷进槽没成（日志 $logs/snapshot.log），一个补丁都没打、没编" "照那份日志修（rsync 在不在、工作根盘满没满）再跑；槽里拷了一半的，下一趟照内容补齐" 1
  else
    echo "  ! 这一趟不占槽：$SLOT_FALLBACK_REASON；快照与编译目录照一趟一份（收尾删，编译从头编）"
    SNAPSHOT_TREE="$logs/repo"
    take_snapshot "$SOURCE_TREE" "$SNAPSHOT_TREE" > "$logs/snapshot.log" 2>&1 \
      || die "拷快照没成（日志 $logs/snapshot.log），一个补丁都没打、没编" "照那份日志修（rsync 在不在、工作根盘满没满）再跑；拷了一半的快照收尾照删" 1
  fi
  # 快照根 git init（文件头「快照」）：判定库按 <git common-dir>/<库名> 取名，不是 git 工作树的读写判定库的脚本退 2；槽里已是 git 工作树的照常 init。
  #   弄坏开关 copy-not-git-initialized：一趟一份那一种不 init
  if [[ -n "$SLOT_DIRECTORY" || "${MERGE_AND_VERIFY_BREAK:-}" != "copy-not-git-initialized" ]]; then
    git init -q "$SNAPSHOT_TREE" > "$logs/snapshot-git-init.log" 2>&1 \
      || die "快照根 git init 没成（日志 $logs/snapshot-git-init.log），一个补丁都没打、没编" "照那份日志修（git 在不在、快照可不可写）再跑；拷了的快照收尾照删" 1
  fi
  # 一趟一份那一种留着快照的那一趟（MERGE_AND_VERIFY_KEEP_COPY=1，文件头「快照」）在日志目录写归属记录 .owner：三行 session、created、purpose，写法同崩溃放量运行目录的
  #   （scripts/hosts/run-build-directory.sh 的 run_directory_owner_record：会话号依次取 WORK_DIRECTORY_OWNER_SESSION、CLAUDE_CODE_SESSION_ID、CLAUDE_SESSION_ID，都没有的不写、已有的不动）。
  #   不留快照的不写；槽位那一种收尾写在槽里（finish）。弄坏开关 keep-owner-unrecorded：留着快照也不写
  if [[ -z "$SLOT_DIRECTORY" && "${MERGE_AND_VERIFY_KEEP_COPY:-}" == 1 && "${MERGE_AND_VERIFY_BREAK:-}" != "keep-owner-unrecorded" ]]; then
    run_directory_owner_record "$logs" "合入后验证留下的快照 $SNAPSHOT_TREE（MERGE_AND_VERIFY_KEEP_COPY=1，用完要删）"
  fi
  if [[ -n "$SLOT_DIRECTORY" ]]; then
    echo "  拷好快照：$SOURCE_TREE → $SNAPSHOT_TREE（$(du -sh -- "$SNAPSHOT_TREE" 2>/dev/null | cut -f1)；不带 .git 与编译目录；按内容拷进槽：写了 $(grep -c '^>f' "$logs/$SLOT_CHANGED_SOURCES_NAME.itemized") 份（内容变了或新加，修改时刻是此刻）、删了 $(grep -c '^\*deleting' "$logs/$SLOT_CHANGED_SOURCES_NAME.itemized") 份，别的留着上一趟的修改时刻；收尾不删）"
  else
    echo "  拷好快照：$SOURCE_TREE → $SNAPSHOT_TREE（$(du -sh -- "$SNAPSHOT_TREE" 2>/dev/null | cut -f1)；不带 .git 与编译目录，修改时刻是此刻，收尾删）"
  fi
  tree="$SNAPSHOT_TREE"
fi
# 验证包那一步（文件头「验证包那一步」）：起快照里那一份门禁，出路那条命令写主工作区那一份
VERIFICATION_STAGE="${MERGE_AND_VERIFY_VERIFICATION_STAGE:-$tree/$VERIFICATION_STAGE_PATH}"
VERIFICATION_RERUN_STAGE="${MERGE_AND_VERIFY_VERIFICATION_STAGE:-$root/$VERIFICATION_STAGE_PATH}"
if [[ -n "$SLOT_DIRECTORY" ]]; then
  # 这一槽的编译目录（文件头「槽位」）：在这一台本地，choose_slot 建好、量过；收尾不删
  CARGO_TARGET_DIR="$SLOT_TARGET"; export CARGO_TARGET_DIR
  echo "  这一槽的编译目录：$SLOT_TARGET（在这一台本地，跟着槽接着上一趟编；超过槽位登记的 slot-target-limit-gib=$SLOT_TARGET_LIMIT_GIB 就清）"
elif [[ -z "${CARGO_TARGET_DIR:-}" ]]; then
  # 编译目录放这一台自己的临时目录下、这一趟新建（文件头「这一趟的编译目录」），不放日志目录：日志目录在工作根下，工作根两台共享（第二台经 NFS 写回本机盘）
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "target-under-work-root" ]]; then   # 弄坏开关：建在 <日志目录>/target
    RUN_TARGET="$logs/target"
    mkdir -- "$RUN_TARGET" || die "建不了这一趟的编译目录 $RUN_TARGET" "看日志目录 $logs 可不可写、盘满没满"
  else
    RUN_TARGET="$(mktemp -d "${TMPDIR:-/tmp}/merge-and-verify-target.XXXXXXXX")" && RUN_TARGET="$(cd "$RUN_TARGET" && pwd -P)" \
      || die "在 ${TMPDIR:-/tmp} 下建不了这一趟的编译目录" "看 ${TMPDIR:-/tmp} 在不在、可不可写、盘满没满；要用现成的编译目录就设 CARGO_TARGET_DIR（那一份收尾不删）"
  fi
  CARGO_TARGET_DIR="$RUN_TARGET"; export CARGO_TARGET_DIR
  echo "  这一趟的编译目录：$RUN_TARGET（这里建的，收尾删）"
else
  echo "  编译目录：$CARGO_TARGET_DIR（调用方设的 CARGO_TARGET_DIR，收尾不删）"
fi
cd "$tree" || die "进不去要编的树 $tree" "看快照 $tree 在不在（拷完就进，进不去多半是盘满或被删）" 1
if [[ $skip_apply -eq 1 ]]; then dirs=(); check_patches_applied "$logs"; fi
verify "$logs" ${dirs[@]+"${dirs[@]}"}; verify_rc=$?
# 先验后合（文件头「先验后合」）：不带 --skip-apply 的那一趟验绿或判不出测试目标（退 3）才打第 2 步那几条与合进主工作区的命令。
#   弄坏开关 merge-in-after-skip-apply：带 --skip-apply 的那一趟也打；merge-in-dropped-on-untargeted：退 3 的不打
if { ((verify_rc == 0)) || { ((verify_rc == 3)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "merge-in-dropped-on-untargeted" ]]; }; } \
   && { ((skip_apply == 0)) || [[ "${MERGE_AND_VERIFY_BREAK:-}" == "merge-in-after-skip-apply" ]]; }; then merge_in_commands "$logs" "$verify_rc"; fi
exit "$verify_rc"
