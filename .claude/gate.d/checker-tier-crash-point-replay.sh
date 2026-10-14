#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 scripts/gate/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判
# gate-stage: 崩溃点重放与登记的崩溃枚举用例（快档跑 checker 档包不标 ignored 的用例、核每条用例的全绿标记；--full 逐条照复用判定跑全量）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: code-source-discipline.sh 与 harness-model-differential-and-scenarios.sh 也调 scripts/crash/crash-case-check.py，但调的是 file-names、one-scenario 两样，分归代码类与 harness 类；这里开跑前只调 placement、modules 两样，判的是崩溃枚举用例与 checker 档测试文件，红了就不起 cargo
# gate-covers: 崩溃点重放
# gate-cell: crash-case-replay 快档跑 checker 档包不标 ignored 的用例、逐条核崩溃枚举用例的全绿标记；--full 逐条照复用判定跑
# gate-cell: feature-gated-tests 快档带上 checker 档 tests/ 里 cfg 点到的特性另跑一趟：挂特性的测试文件逐个跑到的用例不为 0，GPU 对拍没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑
#
# 用法（参数顺序不限）：
#   bash .claude/gate.d/checker-tier-crash-point-replay.sh [项目根]                                     快档，两格都跑；gate.sh 只传项目根
#   bash <worktree>/.claude/gate.d/checker-tier-crash-point-replay.sh --full [--start-over] <worktree>   全量，只有 crash-case-replay 一格
#   --list                     打格名表（格名、制表符、判什么），不起 cargo、不起 python
#   --check <格名>[,<格名>…]   只跑点名的格
#   --list-items               逐行打被判那棵树 stage-inputs.tsv 里键是 crash-case: 的行与它们的 test=，不起 cargo、不起 python
#   --item <用例名>            只看点名的崩溃枚举用例（crash-case: 可写可不写，给几次取并集）：快档照跑 checker 档包、只核这几条的标记；--full 只跑这几条
#   --start-over               只跟 --full：每条用例换一份新的空库（<库的家目录>/crash-point-replay-start-over/<输入指纹>，开跑前清空、跑完留着），判过的不复用、从头判；
#                              不带时用例照家目录下崩溃枚举用例那一类的长期库，判过的块复用（调用方环境里的 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY 清掉）
# 退出码：
#   0   绿；快档有标记不作数、带特性那一格有本次未跑时也退 0，逐条报「本次未跑」
#   1   红；快档有一格红就退 1
#   2   用法错：参数认不出，格名或用例名写错或缺，--start-over 没跟 --full，--full 只点了 feature-gated-tests，项目根进不去，
#       样本档标记目录里的假 cargo 或假 rustc 不在或不可执行，PATH 上的 cargo 没解到标记目录里那一份
#   3   样本档流程判绿：样本档没跑过真的崩溃点重放，不与真跑的绿混在一起
#   77  本次未跑：快档可复用上一次整轮全绿的判定、改动没碰登记的路径，或没有 checker 档包
#   78  准入与运行条件没满足（preflight）
# 不用共用库 lib/stage-cells.sh：它的汇总把 0、77 之外的退出码并成 1（样本档要退 3）；admission.py --selftest 拷这一份去跑的临时仓里也没有 lib/。
# 解析参数的判法与共用库同一套（写错退 2、列出格名）。
#
# 覆盖：两条流的崩溃点重放全量由登记的崩溃枚举用例跑，新池新建文件与 E142 产物逐字比对、「覆盖写、释放、回退与复用」固定脚本到 E 与闭式比对，都由用例自己断言；
#   只做过 mkfs 的池可写挂载再发第一个文件版本那条流与新池新建文件逐项相同，由 cargo test 里的快用例钉住，不另枚举。
#   用例平时标 ignored，--full 在 release 下带 --include-ignored --exact 逐条跑；用例的判别力在用例自己（计数断言与同文件的靶向阳性对照）。
#
# --full：在 HEAD + 暂存区的 worktree 里跑（主工作区读得到别的会话没暂存的改动）；建法与 gate.sh --staged 相同，命令在快档的出路句里。
#   逐条崩溃枚举用例，照登记表的次序。指纹由 admission.py crash-case-manifest 按内容算；这一份 checker-tier-crash-point-replay 不进指纹，改它不废旧标记。
#   全绿标记在 git common-dir 的 singlefs-crash-case-green.<用例名>.<输入指纹>，各 worktree 读写同一组：在且作数就复用；
#   不在才照 crash-case-command 交的命令跑，开跑与跑完各算一次指纹、相同才写。判红删这批输入那一格；「跑的过程中输入变了」判红不删：它说不出开跑那一批的好坏。
#   两趟 --full 同时跑同一条用例、同一个指纹时不加锁：后跑完的判红会删掉先跑完的绿标记，只会假红、不会假绿（.claude/skills/crash-test/commit-stages.md 里「另有 --full 在跑时不起」挡着）。
#   续跑靠判定库：判过的块存在家目录下崩溃枚举用例那一类的长期库里（crates/singlefs-checker-tier/src/crash_state_enumeration.rs、crash_library.rs），重跑同一条用例只判没判过的块；库目录由 crash-case-command 定。
#   线程数 SINGLEFS_CRASH_POINT_REPLAY_THREADS 没设取每台默认重活线程数（.claude/heavy-thread-default 的登记值，admission.py 的 crash_case_worker_threads 经 registered_heavy_threads 取；本机核数只用来判本机多于 1 核与记进标记）；
#   只起 1 个线程怎么判红、标记里线程怎么记，判法在 admission.py crash-case-judge。
#   双机分片默认关：本地配置 ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}（模板 multi-host.env.example）经 scripts/crash/crash-point-replay-shard-configuration-check.sh
#   判得过才开；开着时登记了 shard=across-machines 的用例交 scripts/crash/crash-point-replay-shard-run.sh --merged-log（它经崩溃放量的双机驱动：本机 0/2、第二台 1/2 各写各的库，取回，本机带导入再跑一趟），
#   导入那一趟的日志照单机判、写同一格标记。
#   日志：每趟一个新目录 <common-dir>/singlefs-crash-point-replay-logs/<开跑时刻>-<pid>/，跑完不删；log.<用例名> 是全量日志，同名加 .findings.tsv 是发现日志
#   （行格式以 crates/singlefs-checker-tier/src/crash_point_replay_progress.rs 为准）。发现日志判红：某节没有 summary、red_states 不是 0、定稿行数与 signatures= 对不上，
#   或发现日志空而全量日志里有 CRASH_POINT_REPLAY_FINDINGS 行。发现表超过 30 个签名只打前 30 行。全绿标记不带发现表。
#   这一份里定着结论的只有流程次序（开跑与跑完各算一次指纹、先判日志再写标记、判红删那一格）与两处第二道判红（cargo 退非 0、发现日志）；
#   第一道（test result 恰好 1 passed、登记的计数行）在准入模块的判法摘要里。所以改它时快档照样核标记（范围那一问不摘掉它）。
#
# 快档 feature-gated-tests 格（C604（快档不开特性，崩溃放量与 GPU 对拍编进来零条））：特性从 checker 档 tests/ 顶层每个 .rs 的 cfg 现读，不另抄名单；
#   挂特性的文件跑到 0 条、每处 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍没卡时打这一行后返回成功）都报「本次未跑」、不判红；一个挂特性的文件都没有，记无对象可判。
#   清掉调用方的 SINGLEFS_CRASH_AMPLIFICATION_* 与 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE：快档不写真的判定库、不按全量的规模跑。
#   特性里有 gpu 时这一趟经 scripts/crash/gpu-quiesce-run.sh 起：照多机配置清场并回读、跑，跑完不复原（包装打 step=restore result=not-restored；
#   要复原由用户手动跑私有复原脚本），它打的 GPU_QUIESCE 行逐行转打；清场回读没过（包装退 78、打 step=command result=not-started）判红、写出路，
#   卡上有不归清场白名单的计算进程时出路是等它跑完再只重跑这一格，这里不停它。样本档里清场配置只认样本根 .crash-point-replay-sample-tools/multi-host.env
#   （只写假清场命令），没有就判红、不起包装；包装的登记与本机日志放这一趟的临时目录，不落进真工作根。
#   特性里有 gpu 时这一趟的测试参数带 `-- --nocapture --test-threads 1`（GPU 对拍的测试各占卡的显存，不并行起）；特性里没有 gpu 的不加。
#   还没带这个参数的（不是无例外）：准入模块 admission.py 拼的 crash-case 命令（崩溃枚举用例）、崩溃放量 crash-amplification.sh、双机驱动 crash-amplification-two-hosts.sh（含 crash-point-replay.sh 的双机分支）；
#   等代码批与下一批改动同时改成带。这个参数只管同时起几个测试，不管一个测试里 CPU 判器自己开的线程：GPU 对拍的测试读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度
#   （available_parallelism，不照 SINGLEFS_THREAD_CAP 与登记值；改成取登记值归代码批）。样本里的假 cargo 把收到的参数打成一行 CRASH_POINT_REPLAY_PROGRESS sample-cargo-arguments: …，阶段转出来，quick-features-green 的 expect 核这一行带不带 --test-threads 1。
#
# 多机配置开了 GPU（ENABLE_GPU=1）：快档两趟都在这份配置下跑，不靠仓根配置关 GPU。配置照 scripts/crash/crash-amplification-lib.sh 的
#   crash_amplification_multi_host_configuration_file 找（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，样本档只认样本根
#   .crash-point-replay-sample-tools/multi-host.env），找到的那一份在就经 SINGLEFS_MULTI_HOST_CONFIG=<它的绝对路径> 交给两趟 cargo，用例与这里判的是同一份。
#   不带特性那一趟（crash-case-replay 格）：编不进 gpu 的用例遇到 ENABLE_GPU=1 打一行 NOT_RUN_<原因> test=<用例> reason=<一句> 报本次未跑、照常返回成功
#     （crates 那一侧的约定；GPU 对拍没卡时的 NOT_RUN_NO_GPU_ADAPTER 是同一种写法），这一格把日志里每一处这样的行逐条往 $GATE_NOT_RUN_FILE 报一行
#     （不判红、不算通过，逐格汇总里这一格记「不带特性那一趟有 N 处本次未跑」）；它要判的那一截归带特性那一趟。日志里有判段判在 CPU 的行
#     （崩溃枚举那一类的 CRASH_POINT_REPLAY_JUDGE route=cpu，或 CRASH_POINT_REPLAY_PARALLEL_START / _FINISHED 带 judge=cpu）判红：配了 GPU 不退回 CPU。
#   带特性那一趟（feature-gated-tests 格）：特性里有 gpu 时照旧经 scripts/crash/gpu-quiesce-run.sh 起；配置写了 ENABLE_GPU=1 而日志里有判段判在 CPU 的行
#     （同上两种）判红：配了 GPU、又编进了 gpu，判段必须在 GPU。
#   判得出的只有崩溃枚举那一类打的这两种行；给定状态与放量流水线的用例有的另写一份 ENABLE_GPU=0 的配置（GPU 对拍的 CPU 那一侧），它们的 mode= 不看。
#   配置没写 ENABLE_GPU=1（关着、没写、文件不在）两趟都不判判段在哪。
# 自证：这一份的流程（逐条复用、只重跑输入变了的那一条、库目录与分片变量的清与设、--start-over、快档缺一格判红、跑的过程中输入变了不写标记）由
#   scripts/gate/admission.py --selftest 的「checker-tier-crash-point-replay」那几格核：把这一份拷进临时仓的 .claude/stage-under-test/（不在 .claude/gate.d/ 下）、拿打合成日志的假 cargo 跑；
#   日志与标记怎么判、输入指纹怎么算，由同一份自证的「崩溃枚举用例」那几格拿合成日志与临时仓核。
# 样本档：样本根上有 .crash-point-replay-sample-tools/ 就进样本档，把它放到 PATH 最前面（里面的假 cargo、假 rustc 照 cases/<测试目标>.log 打日志，
#   设了 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 时把 cases/<测试目标>.findings.tsv 追加进去），不管带没带 --full 都走 --full 那一路；判绿退 3，判红照常退 1。
#   里面另放一个 quick-tier 文件就留在快档（复用与范围两问不问）：假 cargo 不带 --features 打 cases/default-pass.log，
#   带了打 cases/feature-pass.<--features 的值>.log、退出码照同名 .exit（没有那一份日志退 101）。
#   样本由 .claude/singlefs-ai-sop/scripts/stage-selftest.sh 跑，setup.sh 在临时目录里 git init；进库的假工具 git 模式是 100755。
# 样本（fixtures/checker-tier-crash-point-replay.sh/）：
#   green                        一条用例的发现日志一节、0 个签名，一条用例不走发现日志
#   red                          两个签名 red_states=3、没有 summary、32 个签名三条用例
#   quick-gpu-configured-plain-pass-not-run      样本的多机配置写了 ENABLE_GPU=1，不带特性那一趟一条用例打了 NOT_RUN_GPU_FEATURE_NOT_COMPILED_IN：记本次未跑、不判红（退 3）
#   quick-gpu-configured-plain-pass-judged-on-cpu  同一份配置，不带特性那一趟的日志里有 CRASH_POINT_REPLAY_JUDGE route=cpu：退回 CPU，判红
#   quick-features-gpu-configured-green          同 quick-features-green，多机配置写了 ENABLE_GPU=1：不带特性那一趟报本次未跑、带特性那一趟判段在 GPU，判绿（退 3）
#   quick-features-gpu-configured-judged-on-cpu  同一份配置，带 gpu 特性那一趟的日志里有 CRASH_POINT_REPLAY_JUDGE route=cpu：判段落在 CPU，判红
#   static-check-red             一条标了 #[ignore]、调 enumerate_crash_states 却没登记 crash-case: 的 checker 档用例，一份第一行没声明模块的 checker 档测试文件：
#                                开跑前的静态判据判红、不起 cargo（red、green 的测试文件第一行都声明了「无」，静态判据判绿、往下走）
#   quick-features-green         三个挂特性的文件都跑到了用例
#   quick-features-gpu-not-run   两处 NOT_RUN_NO_GPU_ADAPTER（一处跟在 test 行后面、一处在库那一段）
#   quick-features-zero          一个文件只有 ignored、一个文件 cargo 输出里没有它那一段
#   quick-features-red           带特性那一趟退 101（不复原）
#   quick-features-quiesce-red   假清场配置的清场回读退 1：带特性那一趟不起、判红（不复原）
#   quick-features-interrupted   假 cargo 打完日志给自己发 TERM（退 143）：判红（不复原）
#   quick-features-no-quiesce-config  样本档里没放假的清场配置：不起包装、判红
#   quick-features-* 的 .crash-point-replay-sample-tools/multi-host.env 是只写假清场命令（true / false）的多机配置，quick-features-no-quiesce-config 没有它。
#   sample-cargo-not-executable  标记目录里只有假 cargo 与假 rustc，setup.sh 把假 cargo 的执行位去掉、给假 rustc 补上：退 2
#   四份 quick-features-* 的 tests/ 里挂特性的写法各有一处：整份 #![cfg(feature)]、折成几行的 #![cfg(all(…))]、逐条 #[cfg(feature)]，
#   与一个只在 // 注释里出现的特性名（读进来就找不到那组特性的日志）。
#   每份样本根的 .gate-cells 点名它算给哪一格：green、red、static-check-red、sample-cargo-not-executable、quick-gpu-configured-* 算给 crash-case-replay，
#   quick-features-* 算给 feature-gated-tests。这一道不用共用库、自己不读 .gate-cells，它只给 code-tooling 的 fixture-claims 格逐格数红绿。
# 弄坏开关（只给证红用）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/checker-tier-crash-point-replay.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   GATE_CRASH_POINT_REPLAY_BREAK=no-findings-file  不设 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE
#       green（期望退出 3，实测 1）；red（输出里找不到「第 1 节…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-unfinished  没有 summary 的节当跑完了
#       red（输出里找不到「第 1 节…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-red-states  不看 red_states
#       red（输出里找不到「问题：第 1 节 red_states=3：有状态判红」）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-truncation  签名全打不截
#       red（输出里找不到「其余 2 个见」）
#   GATE_CRASH_POINT_REPLAY_BREAK=static-check-skipped  跳过开跑前的静态判据
#       static-check-red（输出里找不到「── 开跑前的静态判据」）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-feature-pass  不起带特性那一趟
#       quick-features-gpu-not-run（输出里找不到「本次未跑（这不是通过）：GPU 对拍没跑…」）；quick-features-green（输出里找不到「✓ feature-gated-tests：3 个挂特性的测…」）；
#       quick-features-interrupted、quick-features-quiesce-red、quick-features-red（期望退出 1，实测 3）；quick-features-zero（输出里找不到「! feature-gated-tests 不算全绿：3 个…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=features-not-from-source  特性不从源码读、写死 verdict-store,gpu（假 cargo 没有这组特性的日志、退 101）
#       quick-features-gpu-not-run、quick-features-green、quick-features-zero（期望退出 3，实测 1）；quick-features-interrupted（输出里找不到「GPU_QUIESCE step=command result=exit-143」）；
#       quick-features-red（输出里找不到「找不到 GPU 适配器…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-feature-pass-exit  带特性那一趟退非 0 当成跑过
#       quick-features-interrupted、quick-features-red（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-zero-feature-files  不判挂特性的文件跑到 0 条
#       quick-features-zero（输出里找不到「本次未跑（这不是通过）：checker 档 crates/s…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-gpu-not-run  不读 NOT_RUN_NO_GPU_ADAPTER
#       quick-features-gpu-not-run（输出里找不到「本次未跑（这不是通过）：GPU 对拍没跑…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=gpu-test-threads-unset  特性里有 gpu 也不加 --test-threads 1（照改前，测试线程取 RUST_TEST_THREADS 即登记值）
#       quick-features-green（输出里找不到「…-- --nocapture --test-threads 1」与「sample-cargo-arguments: … -- --nocapture --test-threads 1」两处）
#   GATE_CRASH_POINT_REPLAY_BREAK=gpu-pass-unquiesced  特性里有 gpu 也不经清场包装、直接起 cargo
#       quick-features-green、quick-features-red、quick-features-interrupted（输出里找不到「GPU_QUIESCE step=…」）；
#       quick-features-quiesce-red、quick-features-no-quiesce-config（期望退出 1，实测 3）
#   GPU_QUIESCE_BREAK=run-despite-failed-quiesce  清场包装（scripts/crash/gpu-quiesce-run.sh）清场回读没过也起命令
#       quick-features-quiesce-red（期望退出 1，实测 3）
#   GPU_QUIESCE_BREAK=restore-after-run  清场包装出去时又照多机配置复原
#       quick-features-green、quick-features-red、quick-features-interrupted、quick-features-quiesce-red（输出里找不到「GPU_QUIESCE step=restore result=not-restored holders=0」）
#   GATE_CRASH_POINT_REPLAY_BREAK=sample-tools-unchecked  不核标记目录里的假工具
#       sample-cargo-not-executable（期望退出 2，实测 1）
#   GATE_CRASH_POINT_REPLAY_BREAK=cpu-judging-accepted  配置写了 ENABLE_GPU=1 也不看判段判在哪
#       quick-gpu-configured-plain-pass-judged-on-cpu、quick-features-gpu-configured-judged-on-cpu（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=plain-pass-not-run-unreported  不带特性那一趟日志里的 NOT_RUN_<原因> 行不收、不报本次未跑
#       quick-gpu-configured-plain-pass-not-run、quick-features-gpu-configured-green（输出里找不到「本次未跑（这不是通过）：checker 档快档不带特性那一趟…」）
#   CRASH_CASE_CHECK_BREAK=rows-ignored  判法那一份（scripts/crash/crash-case-check.py）不报没登记的那一条
#       static-check-red（输出里找不到「forgot_to_register_full：标了 #[i…」）
set -uo pipefail
crash_point_replay_tier="quick"
crash_point_replay_start_over=0
root_argument=""
crash_point_replay_list_cells=0
crash_point_replay_list_items=0
crash_point_replay_named_cases=()
# 格名表：与文件头两行 `# gate-cell:` 逐字相同
CRASH_CASE_REPLAY_CELL_NAME="crash-case-replay"
CRASH_CASE_REPLAY_CELL_TITLE="快档跑 checker 档包不标 ignored 的用例、逐条核崩溃枚举用例的全绿标记；--full 逐条照复用判定跑"
FEATURE_GATED_TESTS_CELL_NAME="feature-gated-tests"
FEATURE_GATED_TESTS_CELL_TITLE="快档带上 checker 档 tests/ 里 cfg 点到的特性另跑一趟：挂特性的测试文件逐个跑到的用例不为 0，GPU 对拍没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑"
# --check 点名的格（空着就是两格都跑）
crash_point_replay_selected_cells=()
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
crash_point_replay_check_cells() { # <逗号隔开的格名>：认 crash-case-replay、feature-gated-tests，记进 crash_point_replay_selected_cells；写错、空着退 2 并列出可用的格名
  local requested_cell requested_cells=()
  IFS=, read -r -a requested_cells <<< "$1"
  ((${#requested_cells[@]})) || requested_cells=("")
  for requested_cell in "${requested_cells[@]}"; do
    if [[ "$requested_cell" != "$CRASH_CASE_REPLAY_CELL_NAME" && "$requested_cell" != "$FEATURE_GATED_TESTS_CELL_NAME" ]]; then
      echo "  ✗ checker-tier-crash-point-replay.sh：--check 里有没登记的格名：「$requested_cell」"
      echo "     → 怎么办：格名照下面写（也可以 bash .claude/gate.d/checker-tier-crash-point-replay.sh --list 看）；按崩溃枚举用例点名用 --item <用例名>："
      printf '       %s  %s\n' "$CRASH_CASE_REPLAY_CELL_NAME" "$CRASH_CASE_REPLAY_CELL_TITLE" "$FEATURE_GATED_TESTS_CELL_NAME" "$FEATURE_GATED_TESTS_CELL_TITLE"
      exit 2
    fi
    crash_point_replay_selected_cells+=("$requested_cell")
  done
}
crash_point_replay_runs_cell() { # <格名>：这一趟跑不跑这一格（没给 --check 两格都跑）
  local selected_cell
  ((${#crash_point_replay_selected_cells[@]})) || return 0
  for selected_cell in "${crash_point_replay_selected_cells[@]}"; do [[ "$selected_cell" == "$1" ]] && return 0; done
  return 1
}
while (($#)); do
  stage_argument="$1"
  case "$stage_argument" in
    --full) crash_point_replay_tier="full" ;;
    --start-over) crash_point_replay_start_over=1 ;;
    --list) crash_point_replay_list_cells=1 ;;
    --list-items) crash_point_replay_list_items=1 ;;
    --check|--item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then
        echo "  ✗ $stage_argument 后面缺名字"
        echo "     → 怎么办：写成 --check $CRASH_CASE_REPLAY_CELL_NAME，或 --item <崩溃枚举用例名>（crash-case: 可写可不写，几条就给几次；可点的用 --list-items 看）。"
        exit 2
      fi
      if [[ "$stage_argument" == --check ]]; then crash_point_replay_check_cells "$2"; else crash_point_replay_named_cases+=("crash-case:${2#crash-case:}"); fi
      shift ;;
    --check=*) crash_point_replay_check_cells "${stage_argument#--check=}" ;;
    --item=*) crash_point_replay_named_value="${stage_argument#--item=}"; crash_point_replay_named_cases+=("crash-case:${crash_point_replay_named_value#crash-case:}") ;;
    -*)
      echo "  ✗ 认不出的参数：$stage_argument"
      echo "     → 怎么办：只认 --full、--start-over、--list、--check $CRASH_CASE_REPLAY_CELL_NAME、--list-items、--item <用例名> 与项目根，顺序不限：bash .claude/gate.d/checker-tier-crash-point-replay.sh [--full [--start-over]] [--item <用例名>] [项目根]"
      exit 2 ;;
    *) root_argument="$stage_argument" ;;
  esac
  shift
done
if ((crash_point_replay_list_cells)); then
  printf '%s\t%s\n' "$CRASH_CASE_REPLAY_CELL_NAME" "$CRASH_CASE_REPLAY_CELL_TITLE" "$FEATURE_GATED_TESTS_CELL_NAME" "$FEATURE_GATED_TESTS_CELL_TITLE"
  exit 0
fi
if ((crash_point_replay_list_items)); then
  crash_point_replay_list_table="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}/.claude/gate.d/stage-inputs.tsv"
  if [[ ! -f "$crash_point_replay_list_table" ]]; then
    echo "  ✗ 读不到 $crash_point_replay_list_table：列不出可点名的崩溃枚举用例"
    echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"
    exit 1
  fi
  awk -F'\t' '$1 ~ /^crash-case:/ { test = ""; n = split($3, tokens, " "); for (i = 1; i <= n; i++) if (tokens[i] ~ /^test=/) test = substr(tokens[i], 6); printf "%s\t%s\n", $1, test }' "$crash_point_replay_list_table"
  exit 0
fi
if [[ "$crash_point_replay_start_over" == 1 && "$crash_point_replay_tier" != full ]]; then
  echo "  ✗ --start-over 只跟 --full 一起用：快档不跑全量，没有库可换"
  echo "     → 怎么办：要换一份新的空库、从头跑全量，写成 bash .claude/gate.d/checker-tier-crash-point-replay.sh --full --start-over <根>。"
  exit 2
fi
if [[ "$crash_point_replay_tier" == full ]] && ! crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
  echo "  ✗ --full 只有 $CRASH_CASE_REPLAY_CELL_NAME 一格：$FEATURE_GATED_TESTS_CELL_NAME 只在快档跑"
  echo "     → 怎么办：带特性那一格不带 --full 跑：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME [项目根]；全量写成 --full（或 --full --check $CRASH_CASE_REPLAY_CELL_NAME）。"
  exit 2
fi
ROOT="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
# 相对的 $0 在 cd 之后会指错，在 cd 之前取成绝对路径
crash_point_replay_stage_script_path="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
# 准入模块：读登记表、算指纹、判前提、判日志、读写全绿标记都经它（子命令见它的文件头）
crash_point_replay_stage_repository="$(cd "$(dirname "$0")/../.." && pwd)"
crash_point_replay_admission_module="$crash_point_replay_stage_repository/scripts/gate/admission.py"
# 快档的两趟 cargo test 不吃 cargo 自己的缺省（核数）：经 capped.sh default 起，编译与测试线程取登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它）。
# 随仓走的 scripts/hosts/capped.sh 不在时（admission.py 自证建的临时仓、只放了准入模块与本阶段）不加前缀。
crash_point_replay_thread_prefix=()
[[ -f "$crash_point_replay_stage_repository/scripts/hosts/capped.sh" ]] && crash_point_replay_thread_prefix=(bash "$crash_point_replay_stage_repository/scripts/hosts/capped.sh" default)
cd "$ROOT" 2>/dev/null || exit 2
# 样本档（写法见文件头「样本档」）
crash_point_replay_sample_mode=0
if [[ -d "$PWD/.crash-point-replay-sample-tools" ]]; then
  crash_point_replay_sample_mode=1
  PATH="$PWD/.crash-point-replay-sample-tools:$PATH"
  export PATH
  if [[ -f "$PWD/.crash-point-replay-sample-tools/quick-tier" ]]; then
    echo "  ! 样本档（快档）：$PWD/.crash-point-replay-sample-tools 在、里面有 quick-tier，起 cargo 的是那里面的假 cargo，留在快档、复用与范围两问不问；这一趟不算真跑过 checker 档"
  else
    crash_point_replay_tier="full"
    echo "  ! 样本档：$PWD/.crash-point-replay-sample-tools 在，起用例的是那里面的假 cargo，走 --full 那一路；这一趟不算真跑过崩溃点重放"
  fi
fi
# 弄坏开关（只给证红用，清单在文件头）
crash_point_replay_break="${GATE_CRASH_POINT_REPLAY_BREAK:-}"
# 多机配置与 GPU 开关（文件头「多机配置开了 GPU」）：样本档只认样本根的假配置，真跑照 crash-amplification-lib.sh 的找法；找到的那一份交给两趟 cargo
crash_point_replay_configuration_file=""
crash_point_replay_amplification_library="$crash_point_replay_stage_repository/scripts/crash/crash-amplification-lib.sh"
if [[ "$crash_point_replay_sample_mode" == 1 ]]; then
  if [[ -f "$PWD/.crash-point-replay-sample-tools/multi-host.env" ]]; then crash_point_replay_configuration_file="$PWD/.crash-point-replay-sample-tools/multi-host.env"; fi
elif [[ -f "$crash_point_replay_amplification_library" ]]; then
  crash_point_replay_configuration_file="$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_multi_host_configuration_file "$ROOT")"
fi
crash_point_replay_gpu_configured=0
crash_point_replay_configuration_setting=()
if [[ -n "$crash_point_replay_configuration_file" && -f "$crash_point_replay_configuration_file" ]]; then
  crash_point_replay_configuration_file="$(cd "$(dirname "$crash_point_replay_configuration_file")" && pwd)/$(basename "$crash_point_replay_configuration_file")"
  crash_point_replay_configuration_setting=("SINGLEFS_MULTI_HOST_CONFIG=$crash_point_replay_configuration_file")
  if [[ "$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_configuration_value "$crash_point_replay_configuration_file" ENABLE_GPU)" == 1 ]]; then
    crash_point_replay_gpu_configured=1
  fi
fi
# 样本档先核假工具：不可执行时 PATH 解到真 cargo，就在样本目录里起了真编译、判的也不再是样本；有一样不对退 2
if [[ "$crash_point_replay_sample_mode" == 1 && "$crash_point_replay_break" != sample-tools-unchecked ]]; then
  crash_point_replay_sample_tools_directory="$PWD/.crash-point-replay-sample-tools"
  crash_point_replay_sample_tool_problems=()
  for crash_point_replay_sample_tool in cargo rustc; do
    crash_point_replay_sample_tool_path="$crash_point_replay_sample_tools_directory/$crash_point_replay_sample_tool"
    if [[ ! -e "$crash_point_replay_sample_tool_path" ]]; then
      crash_point_replay_sample_tool_problems+=("假 $crash_point_replay_sample_tool 不在：$crash_point_replay_sample_tool_path")
    elif [[ ! -f "$crash_point_replay_sample_tool_path" || ! -x "$crash_point_replay_sample_tool_path" ]]; then
      crash_point_replay_sample_tool_mode="$(stat -c '%A' "$crash_point_replay_sample_tool_path")"
      crash_point_replay_sample_tool_problems+=("假 $crash_point_replay_sample_tool 不可执行（$crash_point_replay_sample_tool_mode）：$crash_point_replay_sample_tool_path")
    fi
  done
  if crash_point_replay_resolved_cargo="$(type -P cargo)"; then :; else crash_point_replay_resolved_cargo=""; fi
  if [[ "$crash_point_replay_resolved_cargo" != "$crash_point_replay_sample_tools_directory/cargo" ]]; then
    crash_point_replay_sample_tool_problems+=("PATH 最前面放了标记目录，cargo 却解到 ${crash_point_replay_resolved_cargo:-（哪里都解不到）}")
  fi
  if (( ${#crash_point_replay_sample_tool_problems[@]} > 0 )); then
    echo "  ✗ 样本档：$crash_point_replay_sample_tools_directory 里的假工具有 ${#crash_point_replay_sample_tool_problems[@]} 处不对，不往下走（往下走 cargo 就是真的那一个，在样本目录里起真编译）："
    printf '      · %s\n' "${crash_point_replay_sample_tool_problems[@]}"
    echo "     → 怎么办：给标记目录里的假 cargo、假 rustc 补上执行位（chmod 755；进库的样本 git 模式写 100755：git update-index --chmod=+x <文件>），"
    echo "                缺的照 .claude/gate.d/fixtures/checker-tier-crash-point-replay.sh/green/.crash-point-replay-sample-tools 补上，再跑；不是有意跑样本档的，把 $crash_point_replay_sample_tools_directory 挪走再跑。"
    exit 2
  fi
fi

# 开跑前的静态判据（placement、modules；判法在 scripts/crash/crash-case-check.py）：快档、--full、样本档都在复用判定之前判，红了不起 cargo。
# 判法脚本不在（admission.py --selftest 拷这一份去的临时仓没有它）报一行本次未跑、往下走，不判绿也不判红。
crash_point_replay_static_judge="$crash_point_replay_stage_repository/scripts/crash/crash-case-check.py"
if [[ -d crates && "$crash_point_replay_break" != static-check-skipped && ! -f "$crash_point_replay_static_judge" ]]; then
  echo "  ! 开跑前的静态判据没跑：判法脚本 $crash_point_replay_static_judge 不在（这一份 checker-tier-crash-point-replay 被拷到了不带 scripts/ 的树里），往 GATE_NOT_RUN_FILE 报一行"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    printf '%s\n' "checker-tier-crash-point-replay.sh 开跑前的静态判据（placement、modules）没跑：$crash_point_replay_static_judge 不在" >> "$GATE_NOT_RUN_FILE"
  fi
elif [[ -d crates && "$crash_point_replay_break" != static-check-skipped ]]; then
  echo "── 开跑前的静态判据：崩溃枚举用例住哪、登记没有，checker 档测试文件声明模块"
  if python3 "$crash_point_replay_static_judge" --only placement,modules "$ROOT"; then crash_point_replay_static_exit=0; else crash_point_replay_static_exit=$?; fi
  if [[ "$crash_point_replay_static_exit" != 0 && "$crash_point_replay_static_exit" != 77 ]]; then
    echo "  ✗ 开跑前的静态判据：crash-case-check --only placement,modules 退 $crash_point_replay_static_exit（逐处列在上面），这一趟不起 cargo"
    echo "     → 怎么办：照上面那一句出路改（崩溃枚举用例挪进 crates/singlefs-checker-tier/tests/、全量那条登记进 .claude/gate.d/stage-inputs.tsv 的 crash-case: 行；"
    echo "                checker 档测试文件第一行写「//! checker 档模块：…」）；改完先单跑 python3 scripts/crash/crash-case-check.py --only placement,modules 到它退 0，再跑这一道。"
    exit 1
  fi
fi

crash_point_replay_stage_file_name="$(basename "$0")"
crash_point_replay_input_table="$ROOT/.claude/gate.d/stage-inputs.tsv"
crash_point_replay_registered_input_paths=()
if crash_point_replay_registered_paths_text="$(python3 "$crash_point_replay_admission_module" paths "$ROOT" "$crash_point_replay_stage_file_name" 2>/dev/null)" \
   && [[ -n "$crash_point_replay_registered_paths_text" ]]; then
  mapfile -t crash_point_replay_registered_input_paths <<< "$crash_point_replay_registered_paths_text"
fi
if (( ${#crash_point_replay_registered_input_paths[@]} == 0 )); then
  echo "  ✗ $crash_point_replay_input_table 里没有 $crash_point_replay_stage_file_name 这一行（或读不到这份表）：判不出这一道读哪些路径"
  echo "     → 怎么办：在 .claude/gate.d/stage-inputs.tsv 里给本阶段登记它读的路径（制表符分隔，照别的行写）。"
  exit 1
fi
crash_point_replay_input_paths_text="${crash_point_replay_registered_input_paths[*]}"

# 快档先问能不能复用上一次整轮全绿的判定、这次改动碰没碰登记的路径，任一可跳过退 77 不退 0（show-me-test.md「门禁不许假装通过」）；--full 不问。
# 两问之前先核登记的每条路径 git 列得出文件：写错的路径两问都答「没碰」，这一道就一直退 77、一次都不跑。
# 范围那一问另加登记表、这一份 checker-tier-crash-point-replay 与准入模块：登记行与判法摘要进指纹、checker-tier-crash-point-replay 定流程次序，只改它们也要核标记。C8（门禁范围判不出来）还欠着。
# 快档样本档不问：样本是刚 git init 的临时仓，判的是后面的流程。
if [[ "$crash_point_replay_tier" == quick && "$crash_point_replay_sample_mode" == 0 ]]; then
  crash_point_replay_unlisted_input_paths=()
  for registered_input_path in "${crash_point_replay_registered_input_paths[@]}"; do
    registered_input_listing="$(git -c core.quotepath=false -C "$ROOT" ls-files -co --exclude-standard -- "$registered_input_path" 2>/dev/null)"
    if [[ -z "$registered_input_listing" ]]; then crash_point_replay_unlisted_input_paths+=("$registered_input_path"); fi
  done
  if (( ${#crash_point_replay_unlisted_input_paths[@]} > 0 )); then
    echo "  ✗ stage-inputs.tsv 登记给 $crash_point_replay_stage_file_name 的路径里，有 ${#crash_point_replay_unlisted_input_paths[@]} 条 git 一个文件都列不出来：${crash_point_replay_unlisted_input_paths[*]}（登记的是 ${crash_point_replay_input_paths_text}）"
    echo "     → 怎么办：多半是 .claude/gate.d/stage-inputs.tsv 里那一条写错了（拼错、目录少了斜杠、指到不存在的路径）；"
    echo "                在项目根跑 git ls-files -co --exclude-standard -- <那一条>，改到列得出文件为止。全部列不出时先看这里是不是 git 工作树。"
    exit 1
  fi
  source "$crash_point_replay_stage_repository/scripts/gate/stage-run-or-skip.sh" || { echo "  ✗ source 不进 scripts/gate/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
  stage_run_or_skip "$crash_point_replay_stage_repository/scripts/gate/stage-must-run.sh" "复用上一次整轮全绿的判定" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")"
  crash_point_replay_judge_paths=(.claude/gate.d/stage-inputs.tsv "$(realpath -m --relative-to="$ROOT" "$crash_point_replay_stage_script_path")" "$(realpath -m --relative-to="$ROOT" "$crash_point_replay_admission_module")")
  stage_run_or_skip "$crash_point_replay_stage_repository/scripts/gate/change-touches-crates.sh" "这次改动没碰它判的东西" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 ${crash_point_replay_input_paths_text}，加 ${crash_point_replay_judge_paths[*]}，判据见 scripts/gate/change-touches-crates.sh。" \
    -- "$ROOT" "${crash_point_replay_registered_input_paths[@]}" "${crash_point_replay_judge_paths[@]}"
fi
[[ -f Cargo.toml && -d crates/singlefs-checker-tier ]] || { echo "  ! 没有 crates/singlefs-checker-tier（checker 档包），本阶段跳过"; exit 77; }
python3 "$crash_point_replay_admission_module" gate-preconditions "$crash_point_replay_stage_repository" "$crash_point_replay_stage_file_name" || exit 1

if ! git_common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$git_common_directory" ]]; then
  echo "  ✗ $ROOT 不是 git 工作树（取不到 git common-dir）：崩溃枚举用例的全绿标记与续跑的进度文件没处放、也没处读"
  echo "     → 怎么办：在这个项目的 git 仓里跑，或把仓库根作为参数传进来；标记在 \$(git rev-parse --git-common-dir)/singlefs-crash-case-green.<用例名>.<输入指纹>。"
  exit 1
fi
crash_point_replay_scratch_directory="$(mktemp -d)"
trap 'rm -rf -- "${crash_point_replay_scratch_directory:?}"' EXIT

# crash-cases 一条一行：键、包、测试目标、用例函数（制表符分隔），照登记表的次序；登记有错退非 0
if ! crash_case_listing="$(python3 "$crash_point_replay_admission_module" crash-cases "$ROOT")"; then
  printf '%s\n' "$crash_case_listing" | sed 's/^/       /'
  echo "  ✗ .claude/gate.d/stage-inputs.tsv 里 crash-case: 那几行登记有错（上面逐条列出）：判不出要跑、要核哪几条崩溃枚举用例"
  echo "     → 怎么办：照 scripts/gate/admission.py 文件头「崩溃枚举用例行」改那几行（test=<包>:<测试目标>:<用例函数> 恰好一条，用例函数在测试目标里找得到）；"
  echo "                单跑 python3 scripts/gate/admission.py crash-cases <项目根>，改到它退 0 为止。"
  exit 1
fi
crash_case_rows=()
if [[ -n "$crash_case_listing" ]]; then mapfile -t crash_case_rows <<< "$crash_case_listing"; fi
if (( ${#crash_case_rows[@]} == 0 )); then
  echo "  ✗ .claude/gate.d/stage-inputs.tsv 里一条崩溃枚举用例（键是 crash-case: 的行）都没登记：崩溃点重放全量没有东西可跑、可核"
  echo "     → 怎么办：两条流的崩溃点重放全量至少各登记一行，写法见 scripts/gate/admission.py 文件头「崩溃枚举用例行」。"
  exit 1
fi
if (( ${#crash_point_replay_named_cases[@]} > 0 )); then
  crash_point_replay_kept_rows=()
  crash_point_replay_unknown_cases=()
  for crash_point_replay_named_case in "${crash_point_replay_named_cases[@]}"; do
    crash_point_replay_found=0
    for crash_case_row in "${crash_case_rows[@]}"; do [[ "${crash_case_row%%$'\t'*}" == "$crash_point_replay_named_case" ]] && crash_point_replay_found=1; done
    (( crash_point_replay_found )) || crash_point_replay_unknown_cases+=("$crash_point_replay_named_case")
  done
  if (( ${#crash_point_replay_unknown_cases[@]} > 0 )); then
    echo "  ✗ --item 点名的崩溃枚举用例没登记：${crash_point_replay_unknown_cases[*]}"
    echo "     → 怎么办：用例名照 bash .claude/gate.d/checker-tier-crash-point-replay.sh --list-items 列的写（登记表里有 ${#crash_case_rows[@]} 条）。"
    exit 2
  fi
  for crash_case_row in "${crash_case_rows[@]}"; do
    for crash_point_replay_named_case in "${crash_point_replay_named_cases[@]}"; do
      if [[ "${crash_case_row%%$'\t'*}" == "$crash_point_replay_named_case" ]]; then crash_point_replay_kept_rows+=("$crash_case_row"); break; fi
    done
  done
  echo "  · 只看 --item 点名的 ${#crash_point_replay_kept_rows[@]} 条崩溃枚举用例（登记表里共 ${#crash_case_rows[@]} 条，别的这一趟不跑、不核）"
  crash_case_rows=("${crash_point_replay_kept_rows[@]}")
fi

# write_crash_case_manifest <键> <清单文件>：这条用例这批输入的逐文件清单写进清单文件，指纹、文件数、减去的文件数放进
# case_fingerprint / case_file_count / case_excluded_count。算不出返回 1，原因放进 case_manifest_problem。
write_crash_case_manifest() {
  local manifest_summary
  if ! manifest_summary="$(python3 "$crash_point_replay_admission_module" crash-case-manifest "$ROOT" "$1" "$2" \
      --judging-digest --toolchain --build-environment)"; then
    case_manifest_problem="$manifest_summary"
    return 1
  fi
  read -r case_fingerprint case_file_count case_excluded_count <<< "$manifest_summary"
  if [[ ! "$case_fingerprint" =~ ^[0-9a-f]{64}$ || ! "$case_file_count" =~ ^[1-9][0-9]*$ || ! "$case_excluded_count" =~ ^[0-9]+$ ]]; then
    case_manifest_problem="准入模块打的不是「<指纹> <文件数> <减去的文件数>」：$manifest_summary"
    return 1
  fi
  return 0
}

# delete_crash_case_marker <键> <指纹>：删这条用例这批输入那一格全绿标记
delete_crash_case_marker() {
  local marker_path
  marker_path="$(python3 "$crash_point_replay_admission_module" crash-case-marker-path "$ROOT" "$1" "$2")" || return 0
  if [[ -n "$marker_path" ]]; then rm -f -- "${marker_path:?}"; fi
  return 0
}

# run_checker_package_tests <日志>：只跑不标 ignored 的用例（--tests 连 src/bin/ 的内联单测一起带上），cargo 的整段输出进日志；
# 快档不写发现日志，清掉调用方的 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE。
run_checker_package_tests() {
  env -u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
    ${crash_point_replay_thread_prefix[@]+"${crash_point_replay_thread_prefix[@]}"} cargo test --release --no-fail-fast -p singlefs-checker-tier --lib --tests -- --nocapture 2>&1 \
    | tee "$1" \
    | { grep --line-buffered '^CRASH_POINT_REPLAY_PROGRESS ' || true; } \
    | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# read_crash_case_command <键> <输入指纹>：准入模块 crash-case-command 交出的这一条的命令与环境（以 NUL 分隔）读进 case_machine_cores、
# case_threads、case_threads_origin（这三个只打进开跑那一行，不转给 crash-case-judge / record）、case_library_directory（--start-over 的空库，或 default）与数组 case_command。
# 交不出、写法不对返回 1，原因放进 case_command_problem。
read_crash_case_command() {
  local command_file="$crash_point_replay_scratch_directory/command.${1#crash-case:}" command_output
  local -a start_over_option=()
  if [[ "$crash_point_replay_start_over" == 1 ]]; then start_over_option=(--start-over); fi
  if ! python3 "$crash_point_replay_admission_module" crash-case-command "$ROOT" "$1" "$2" "${start_over_option[@]}" > "$command_file"; then
    case_command_problem="$(tr '\0' ' ' < "$command_file")"
    return 1
  fi
  local -a command_words=()
  mapfile -d '' -t command_words < "$command_file"
  if (( ${#command_words[@]} < 5 )) || [[ ! "${command_words[0]}" =~ ^[1-9][0-9]*$ || ! "${command_words[1]}" =~ ^[1-9][0-9]*$ \
      || ! "${command_words[2]}" =~ ^(explicit|default)$ ]]; then
    command_output="$(tr '\0' ' ' < "$command_file")"
    case_command_problem="准入模块交的不是「核数、线程数、explicit|default、库目录、命令…」：${command_output}"
    return 1
  fi
  case_machine_cores="${command_words[0]}"
  case_threads="${command_words[1]}"
  case_threads_origin="${command_words[2]}"
  case_library_directory="${command_words[3]}"
  case_command=("${command_words[@]:4}")
  return 0
}

# run_crash_case <日志> <发现日志> <命令的词…>：环境里设 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE=<发现日志>；整段输出进日志，CRASH_POINT_REPLAY_PROGRESS 行边跑边转出来。
run_crash_case() {
  local log_file="$1"
  local -a findings_setting=(SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE="$2")
  if [[ "$crash_point_replay_break" == no-findings-file ]]; then findings_setting=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE); fi
  shift 2
  env "${findings_setting[@]}" "$@" 2>&1 \
    | tee "$log_file" \
    | { grep --line-buffered '^CRASH_POINT_REPLAY_PROGRESS ' || true; } \
    | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# run_crash_case_in_two_shards <键> <日志> <输入指纹> <测试目标> <用例函数>：双机跑一条（crash-point-replay-shard-run.sh --merged-log，它经崩溃放量的双机驱动），
# 导入那一趟的日志由驱动抄进 <日志>；库目录照 crash-case-command 交的（--start-over 时是那一份空库，不然清掉调用方的、用例照长期库），线程数照 case_threads。
run_crash_case_in_two_shards() {
  local -a library_setting=(-u SINGLEFS_CRASH_AMPLIFICATION_LIBRARY)
  if [[ "$case_library_directory" != default ]]; then library_setting=("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=$case_library_directory"); fi
  env "${library_setting[@]}" "SINGLEFS_CRASH_POINT_REPLAY_THREADS=$case_threads" bash "$crash_point_replay_shard_driver" --merged-log "$1" "$ROOT" "$3" "$2" "$4" "$5" 2>&1 | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# report_crash_case_findings <发现日志> <全量日志>：逐节打发现表（行格式与节的写法以 crates/singlefs-checker-tier/src/crash.rs 的发现日志为准：
# 字段制表符分，第一个字段是行的种类，其余 key=value）。每一处不对各打一行「问题：…」，有问题返回 1；发现日志不在又不该有时返回 0。
report_crash_case_findings() {
  local findings_file="$1" log_file="$2" stdout_findings_lines
  stdout_findings_lines="$(grep -c '^CRASH_POINT_REPLAY_FINDINGS ' "$log_file" 2>/dev/null)"
  stdout_findings_lines="${stdout_findings_lines:-0}"
  if [[ ! -s "$findings_file" ]]; then
    if [[ "$stdout_findings_lines" == 0 ]]; then
      echo "    发现日志：没有（$findings_file 不在或是空的，全量日志里也没有 CRASH_POINT_REPLAY_FINDINGS 行：这条用例没走读 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 的枚举入口）"
      return 0
    fi
    echo "    发现日志：$findings_file 不在或是空的"
    echo "      问题：全量日志里有 ${stdout_findings_lines} 行 CRASH_POINT_REPLAY_FINDINGS（枚举跑完了 ${stdout_findings_lines} 趟），发现日志却一节都没写：SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 没传进用例"
    return 1
  fi
  echo "    发现日志：$findings_file（全量日志里 CRASH_POINT_REPLAY_FINDINGS ${stdout_findings_lines} 行）"
  awk -F'\t' -v shown_limit=30 -v findings_path="$findings_file" -v break_item="$crash_point_replay_break" '
    function field(name,   position, equals) {
      for (position = 2; position <= NF; position++) {
        equals = index($position, "=")
        if (equals > 0 && substr($position, 1, equals - 1) == name) return substr($position, equals + 1)
      }
      return ""
    }
    function signature_text(tail_name) {
      return "#" field("finding") " pass=" field("pass") " violated=" field("violated") " segment=" field("segment") " publish=" field("publish") " " tail_name
    }
    $1 == "crash_point_replay_findings_begin" {
      sections++
      section_name[sections] = "stream=" field("stream") (field("shard") != "" ? " shard=" field("shard") : "") " states=" field("states")
      next
    }
    sections == 0 { lines_before_first_section++; next }
    $1 == "crash_point_replay_finding" { final_rows[sections, ++final_count[sections]] = signature_text("states=" field("states") " sample_states=" field("sample_states")); next }
    $1 == "crash_point_replay_finding_new" { new_rows[sections, ++new_count[sections]] = signature_text("first_state=" field("first_state")); next }
    $1 == "crash_point_replay_finding_threshold" { next }
    $1 == "crash_point_replay_findings_summary" {
      summary_count[sections]++
      summary_signatures[sections] = field("signatures"); summary_red_states[sections] = field("red_states"); summary_states[sections] = field("states")
      next
    }
    { unknown_kinds[$1]++ }
    function print_rows(kind, section, count,   row, limit) {
      limit = (break_item == "no-truncation") ? count : shown_limit
      for (row = 1; row <= count && row <= limit; row++) print "        " (kind == "final" ? final_rows[section, row] : new_rows[section, row])
      if (count > limit) print "        其余 " (count - limit) " 个见 " findings_path
    }
    END {
      problems = 0
      if (sections == 0) { print "      问题：一行 crash_point_replay_findings_begin 都没有，认不出节"; problems++ }
      if (lines_before_first_section > 0) { print "      问题：第一行 crash_point_replay_findings_begin 之前有 " lines_before_first_section " 行"; problems++ }
      for (kind in unknown_kinds) print "      （认不出的行种类 " kind "：" unknown_kinds[kind] " 行，没判）"
      for (section = 1; section <= sections; section++) {
        if (summary_count[section] == 0) {
          if (break_item == "ignore-unfinished") continue
          print "      第 " section " 节（" section_name[section] "）：这一趟没跑完（没有 crash_point_replay_findings_summary 行）；死之前找到 " (new_count[section] + 0) " 个签名："
          print_rows("new", section, new_count[section] + 0)
          print "      问题：第 " section " 节这一趟没跑完（没有 crash_point_replay_findings_summary 行：还在跑、被杀或 panic 判红）"
          problems++
          continue
        }
        print "      第 " section " 节（" section_name[section] "）：signatures=" summary_signatures[section] " red_states=" summary_red_states[section] " states=" summary_states[section]
        if (final_count[section] + 0 == 0) print "        没有签名"
        print_rows("final", section, final_count[section] + 0)
        if (summary_count[section] > 1) { print "      问题：第 " section " 节有 " summary_count[section] " 行 crash_point_replay_findings_summary"; problems++ }
        if (summary_signatures[section] != ((final_count[section] + 0) "")) {
          print "      问题：第 " section " 节定稿 " (final_count[section] + 0) " 行 crash_point_replay_finding，summary 却说 signatures=" summary_signatures[section]; problems++
        }
        if (break_item != "ignore-red-states" && summary_red_states[section] != "0") {
          print "      问题：第 " section " 节 red_states=" summary_red_states[section] "：有状态判红（不是 0 都判红；读不出数也判红）"; problems++
        }
      }
      exit (problems > 0 ? 1 : 0)
    }
  ' "$findings_file"
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

# print_staged_worktree_full_commands：出路里建「HEAD + 暂存区」worktree 并在里面跑 --full 的命令；建法与共享 gate.sh --staged 相同。
print_staged_worktree_full_commands() {
  echo '                在项目根、暂存之后，把下面三行命令放进同一次 Bash 调用（三行共用 crash_point_replay_full_base 这个变量，分开跑它就是空的；这次调用的退出码是经内存包装的那条 --full 命令的，250–254 是包装自己的结局）：'
  echo '                crash_point_replay_tree_ready=; crash_point_replay_full_base="$(mktemp -d)"; git diff --cached --binary > "$crash_point_replay_full_base/staged.patch"'
  echo '                git worktree add --detach "$crash_point_replay_full_base/tree" HEAD && { [ ! -s "$crash_point_replay_full_base/staged.patch" ] || git -C "$crash_point_replay_full_base/tree" apply --index "$crash_point_replay_full_base/staged.patch"; } && crash_point_replay_tree_ready=1'
  echo '                if [ "$crash_point_replay_tree_ready" = 1 ]; then SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh 16G bash "${crash_point_replay_full_base:?}/tree/.claude/gate.d/checker-tier-crash-point-replay.sh" --full "${crash_point_replay_full_base:?}/tree"; crash_point_replay_full_rc=$?; else echo "worktree 没建好或暂存区的 diff 套不上，--full 没跑"; crash_point_replay_full_rc=1; fi; git worktree remove --force "${crash_point_replay_full_base:?}/tree" 2>/dev/null; rm -rf "${crash_point_replay_full_base:?}"; ( exit "$crash_point_replay_full_rc" )'
}

# run_checker_quick_tier：快档跑 checker 档包不标 ignored 的用例。判红打出路、返回 1；判绿把每个测试二进制那一行 test result 的 passed / ignored 加总进 quick_tier_report。
run_checker_quick_tier() {
  local quick_log result_lines passed_count ignored_count passed_total ignored_total binary_count
  quick_log="$crash_point_replay_scratch_directory/quick-singlefs-checker-tier.log"
  if ! run_checker_package_tests "$quick_log"; then
    tail -40 "$quick_log"
    # --no-fail-fast：一个测试二进制红了其余照跑，红的用例与测试目标在这里一次列全
    grep -E '^test .* FAILED$|^error: test failed, to rerun pass' "$quick_log" | sed 's/^/    /' || true
    echo "  ✗ checker 档快档判红（上面是 cargo test 的尾部与全部红的用例、测试目标）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-checker-tier -- --nocapture（只看一个测试二进制加 --test <名>）；"
    echo "                断言消息里是第一处对不上的计数或违例；改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
    return 1
  fi
  result_lines="$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$quick_log")"
  passed_total=0; ignored_total=0; binary_count=0
  while read -r passed_count ignored_count; do
    [[ -n "${passed_count:-}" ]] || continue
    passed_total=$((passed_total + passed_count)); ignored_total=$((ignored_total + ignored_count)); binary_count=$((binary_count + 1))
  done <<< "$result_lines"
  if (( passed_total == 0 )); then
    echo "  ✗ checker 档快档跑过了，却读不到 cargo 的 test result 行，或一条用例都没通过：扫到 0 条不是通过"
    echo "     → 怎么办：cargo test --release -p singlefs-checker-tier -- --list 看这个包里还剩几条不标 ignored 的用例；"
    echo "                一条都没有，就是快用例被整批标了 ignored 或删掉了，补回来。"
    return 1
  fi
  quick_tier_report="checker 档包 ${binary_count} 个测试二进制，${passed_total} 条通过、${ignored_total} 条 ignored（全量那几条留给 --full）"
  # 配置开了 GPU（文件头「多机配置开了 GPU」）：判段判在 CPU 的判红；用例报的本次未跑逐条报，不判红、不算通过
  local cpu_judging=() plain_where plain_line
  if (( crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-accepted ]]; then
    mapfile -t cpu_judging < <(cpu_judging_lines "$quick_log")
    if (( ${#cpu_judging[@]} > 0 )); then
      printf '      %s\n' "${cpu_judging[@]:0:20}"
      echo "  ✗ checker 档快档不带 gpu 特性那一趟：多机配置 $crash_point_replay_configuration_file 写了 ENABLE_GPU=1，日志里有 ${#cpu_judging[@]} 处判段判在 CPU（上面列了前 20 处），配了 GPU 不退回 CPU"
      echo "     → 怎么办：不带 gpu 特性那一趟遇到 ENABLE_GPU=1 打一行 NOT_RUN_<原因> test=<用例> reason=<一句> 报本次未跑、不判（crates 那一侧改），判归带特性那一趟（feature-gated-tests 格）；改完只重跑这一格：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME"
      return 1
    fi
  fi
  crash_point_replay_plain_not_run_total=0
  if [[ "$crash_point_replay_break" != plain-pass-not-run-unreported ]]; then
    while IFS=$'\t' read -r plain_where plain_line; do
      [[ -n "$plain_line" ]] || continue
      crash_point_replay_plain_not_run_total=$((crash_point_replay_plain_not_run_total + 1))
      crash_point_replay_report_not_run "checker 档快档不带特性那一趟（${plain_where}）报了本次未跑：${plain_line}；这一截归带特性那一趟（$FEATURE_GATED_TESTS_CELL_NAME 格）判，这一趟不算跑过"
    done < <(test_not_run_lines "$quick_log")
  fi
  return 0
}

# cpu_judging_lines <日志>：崩溃枚举那一类判段判在 CPU 的行（CRASH_POINT_REPLAY_JUDGE route=cpu，或 CRASH_POINT_REPLAY_PARALLEL_START / _FINISHED 带 judge=cpu；
# 行首可带空白），一处一行「<它所在的 tests/<名>.rs，或「库与别的二进制」>：<那一行>」
cpu_judging_lines() {
  awk '
    /^[[:space:]]*Running / {
      current = "库与别的二进制"
      if (match($0, /tests\/[^\/ ]+\.rs \(/)) current = "tests/" substr($0, RSTART + 6, RLENGTH - 8)
      next
    }
    {
      line = $0
      sub(/^[[:space:]]+/, "", line)
      if (line ~ /^CRASH_POINT_REPLAY_JUDGE route=cpu( |$)/ || (line ~ /^CRASH_POINT_REPLAY_PARALLEL_(START|FINISHED) / && line ~ / judge=cpu( |$)/))
        print ((current == "") ? "库与别的二进制" : current) "：" line
    }
  ' "$1"
}

# test_not_run_lines <日志>：用例报的本次未跑行（NOT_RUN_<原因> test=<用例> …，跟在「test <名> ... 」后面的也认，按子串找），
# 一处一行「<它所在的 tests/<名>.rs，或「库与别的二进制」><TAB><从 NOT_RUN_ 起到下一处 NOT_RUN_ 之前>」
test_not_run_lines() {
  awk '
    /^[[:space:]]*Running / {
      current = "库与别的二进制"
      if (match($0, /tests\/[^\/ ]+\.rs \(/)) current = "tests/" substr($0, RSTART + 6, RLENGTH - 8)
      next
    }
    {
      rest = $0
      while (match(rest, /NOT_RUN_[A-Z][A-Z0-9_]* test=/) > 0) {
        rest = substr(rest, RSTART)
        tail = substr(rest, 2)
        if (match(tail, /NOT_RUN_[A-Z][A-Z0-9_]* test=/) > 0) { piece = substr(rest, 1, RSTART); rest = substr(tail, RSTART) } else { piece = rest; rest = "" }
        sub(/[[:space:]]+$/, "", piece)
        print ((current == "") ? "库与别的二进制" : current) "\t" piece
      }
    }
  ' "$1"
}

# crash_point_replay_report_not_run <哪一部分没跑：为什么、怎么补>：照共享 lib.sh 的 report_not_run（这一份不 source lib.sh）。
crash_point_replay_report_not_run() {
  echo "  ! 本次未跑（这不是通过）：$1"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$1" >> "$GATE_NOT_RUN_FILE"; fi
}

# check_crash_case_markers：crash-case-replay 格快档的后一半，逐条核全绿标记；不作数的报本次未跑、不判红。结论放进 crash_point_replay_cell_state。
check_crash_case_markers() {
  present_report="$crash_point_replay_scratch_directory/present-report"
  missing_report="$crash_point_replay_scratch_directory/missing-report"
  : > "$present_report"
  : > "$missing_report"
  missing_cases=()
  for crash_case_row in "${crash_case_rows[@]}"; do
    IFS=$'\t' read -r case_key _case_package _case_target _case_function <<< "$crash_case_row"
    case_manifest="$crash_point_replay_scratch_directory/quick-manifest.${case_key#crash-case:}"
    if ! write_crash_case_manifest "$case_key" "$case_manifest"; then
      missing_cases+=("$case_key")
      echo "       $case_key：算不出这批输入的指纹：$case_manifest_problem" >> "$missing_report"
      continue
    fi
    if marker_check_output="$(python3 "$crash_point_replay_admission_module" crash-case-marker-check "$ROOT" "$case_key" "$case_fingerprint" "$case_manifest")"; then
      read -r _ok_word _marker_path marker_finished_utc <<< "$(head -1 <<< "$marker_check_output")"
      {
        echo "    $case_key：那一格跑完于 ${marker_finished_utc}（这批输入的指纹 ${case_fingerprint:0:16}…，${case_file_count} 个文件，减去用例读不到的 ${case_excluded_count} 个）"
        tail -n +2 <<< "$marker_check_output" | sed 's/^/      /'
      } >> "$present_report"
    else
      missing_cases+=("$case_key")
      {
        echo "       $case_key（这批输入的指纹 ${case_fingerprint:0:16}…，${case_file_count} 个文件，减去用例读不到的 ${case_excluded_count} 个）："
        sed 's/^/         /' <<< "$marker_check_output"
      } >> "$missing_report"
    fi
  done
  echo "  ✓ checker 档快档跑完（release，只跑不标 ignored 的用例，全量留给 --full）：${quick_tier_report}"
  if (( ${#missing_cases[@]} > 0 )); then
    echo "  ! ${#missing_cases[@]} 条崩溃枚举用例没有作数的全绿标记，全量这一轮没跑，记「本次未跑」、不判红（全量默认不在提交时跑，D13 已定项 15）：${missing_cases[*]}"
    cat "$missing_report"
    legacy_marker_total="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green*' 2>/dev/null | grep -c .)"
    if [[ "$legacy_marker_total" -gt 0 ]]; then
      echo "       common-dir 里还有 ${legacy_marker_total} 格按整批输入分格的旧标记（singlefs-layer0-full-green*）：分成逐条用例之后不再认，可以删掉。"
    fi
    echo "     → 要跑全量（用户要求时）：暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 checker-tier-crash-point-replay 跑 --full <它的根>"
    echo "                （与 gate.sh --staged 同一建法；它只跑没有作数标记的那几条，别的复用）："
    print_staged_worktree_full_commands
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
      for missing_case in "${missing_cases[@]}"; do
        echo "崩溃枚举用例 ${missing_case} 这批输入没有作数的全绿标记，全量没跑（要跑：checker-tier-crash-point-replay --full）" >> "$GATE_NOT_RUN_FILE"
      done
    fi
  fi
  echo "  ✓ 登记 ${#crash_case_rows[@]} 条崩溃枚举用例，$(( ${#crash_case_rows[@]} - ${#missing_cases[@]} )) 条的全绿标记与各自这批输入的指纹相同（登记路径 ${crash_point_replay_input_paths_text}，逐条减去用例读不到的文件），标记里的计数行原样："
  cat "$present_report"
  crash_point_replay_cell_state="绿"
  if (( ${#missing_cases[@]} > 0 )); then crash_point_replay_cell_state="有 ${#missing_cases[@]} 条崩溃枚举用例全量本次未跑"; fi
}

# list_feature_gated_test_files <tests 目录>：目录顶层每个 .rs（一个文件一个测试二进制）里 #[cfg(…)] / #![cfg(…)] 点到的特性，
# 一个文件一行「文件名<TAB>whole-file|items<TAB>特性,特性」（有 #![cfg(…)] 点到特性的记 whole-file），照文件名排序；挂特性的一个都没有时不打。
# 括号按层数配对（rustfmt 把长的 cfg 折成几行也认）；// 起头的行（注释与文档注释）不读。
list_feature_gated_test_files() {
  python3 - "$1" <<'PYTHON'
import pathlib
import re
import sys

attribute_start = re.compile(r"#(!?)\[\s*cfg\s*\(")
feature_name = re.compile(r"\bfeature\s*=\s*\"([^\"]+)\"")
for path in sorted(pathlib.Path(sys.argv[1]).glob("*.rs")):
    text = "\n".join(line for line in path.read_text(encoding="utf-8").splitlines() if not line.lstrip().startswith("//"))
    features, whole_file = set(), False
    for match in attribute_start.finditer(text):
        depth, position = 1, match.end()
        while position < len(text) and depth > 0:
            depth += {"(": 1, ")": -1}.get(text[position], 0)
            position += 1
        named = feature_name.findall(text[match.end():position])
        features.update(named)
        if named and match.group(1) == "!":
            whole_file = True
    if features:
        print(f"{path.name}\t{'whole-file' if whole_file else 'items'}\t{','.join(sorted(features))}")
PYTHON
}

# count_tests_run_by_file <日志…>：cargo test 输出里逐个集成测试文件跑到的用例数，几份日志相加；一行「文件名<TAB>条数」。
# 一个测试二进制从 cargo 的「Running …tests/<名>.rs (…)」那一行起、到下一行 Running 为止，条数是它 test result 行的 passed 加 failed（ignored 不算跑到）；
# 库（Running unittests …）与别的二进制不打。
count_tests_run_by_file() {
  awk '
    FNR == 1 { current = "" }
    /^[[:space:]]*Running / {
      current = ""
      if (match($0, /tests\/[^\/ ]+\.rs \(/)) { current = substr($0, RSTART + 6, RLENGTH - 8); seen[current] = 1 }
      next
    }
    current != "" && /^test result: / {
      if (match($0, /[0-9]+ passed/)) ran[current] += substr($0, RSTART, RLENGTH) + 0
      if (match($0, /[0-9]+ failed/)) ran[current] += substr($0, RSTART, RLENGTH) + 0
    }
    END { for (name in seen) printf "%s\t%d\n", name, ran[name] + 0 }
  ' "$@"
}

# gpu_adapter_not_run_lines <日志>：日志里每一处 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍找不到 GPU 适配器时打的那一行：test=<用例名> reason=<一句>），
# 一处一行「<它所在的 tests/<名>.rs，或「库与别的二进制」><TAB>NOT_RUN_NO_GPU_ADAPTER …」；--nocapture 下它可能跟在「test <名> ... 」后面，按子串找。
gpu_adapter_not_run_lines() {
  awk '
    /^[[:space:]]*Running / {
      current = "库与别的二进制"
      if (match($0, /tests\/[^\/ ]+\.rs \(/)) current = "tests/" substr($0, RSTART + 6, RLENGTH - 8)
      next
    }
    {
      rest = $0
      while ((at = index(rest, "NOT_RUN_NO_GPU_ADAPTER ")) > 0) {
        rest = substr(rest, at)
        following = index(substr(rest, 2), "NOT_RUN_NO_GPU_ADAPTER ")
        piece = (following > 0) ? substr(rest, 1, following) : rest
        sub(/[[:space:]]+$/, "", piece)
        print ((current == "") ? "库与别的二进制" : current) "\t" piece
        rest = (following > 0) ? substr(rest, following + 1) : ""
      }
    }
  ' "$1"
}

# run_feature_gated_tests_cell <crash-case-replay 格那一趟的日志，没跑传空串>：判红打出路、返回 1；不红返回 0，结论放进 feature_cell_state。
run_feature_gated_tests_cell() {
  local tests_directory="crates/singlefs-checker-tier/tests" listing feature_list feature_log feature_exit=0 bindgen_arguments=""
  local amplification_library="$crash_point_replay_stage_repository/scripts/crash/crash-amplification-lib.sh"
  local counts_file="$crash_point_replay_scratch_directory/feature-gated-counts"
  local row file_name file_scope file_features scope_text count ran_total=0 environment_name gpu_where gpu_line not_run_total
  local -a gated_rows=() features=() target_arguments=() bindgen_setting=() cleared_environment=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE) zero_files=() gpu_not_run=() gpu_test_arguments=()
  if ! listing="$(list_feature_gated_test_files "$tests_directory")"; then
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：读不出 $tests_directory 下哪几个测试文件挂特性 cfg（上面是 python 的报错）"
    echo "     → 怎么办：看报错里点名的文件（多半不是 UTF-8 或读不了），修好之后只重跑这一格：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
    return 1
  fi
  if [[ -z "$listing" ]]; then
    feature_cell_state="无对象可判"
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：$tests_directory 下 0 个测试文件挂特性 cfg，无对象可判，带特性那一趟不起（往 GATE_NOT_RUN_FILE 记一行）"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
      printf '%s\n' "checker-tier-crash-point-replay.sh 的 $FEATURE_GATED_TESTS_CELL_NAME 格无对象可判：$tests_directory 下没有挂特性 cfg 的测试文件" >> "$GATE_NOT_RUN_FILE"
    fi
    return 0
  fi
  mapfile -t gated_rows <<< "$listing"
  mapfile -t features < <(cut -f3 <<< "$listing" | tr ',' '\n' | LC_ALL=C sort -u)
  feature_list="$(IFS=,; printf '%s' "${features[*]}")"
  if [[ "$crash_point_replay_break" == features-not-from-source ]]; then feature_list="verdict-store,gpu"; fi
  for row in "${gated_rows[@]}"; do target_arguments+=(--test "${row%%.rs$'\t'*}"); done
  # 特性里有 gpu：测试线程只开 1 个（GPU 对拍的几个测试各占一张卡的显存，并行起会互相抢；一个测试里 CPU 判器自己的线程不受这个参数管：读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度）
  if [[ ",$feature_list," == *,gpu,* && "$crash_point_replay_break" != gpu-test-threads-unset ]]; then gpu_test_arguments=(--test-threads 1); fi
  # 特性里有 gpu：这一趟经清场包装起（文件头「快档 feature-gated-tests 格」）；样本档只认样本里只写假清场命令的那份多机配置
  local -a quiesce_prefix=() quiesce_environment=()
  if [[ ",$feature_list," == *,gpu,* && "$crash_point_replay_break" != gpu-pass-unquiesced ]]; then
    quiesce_prefix=(bash "$crash_point_replay_stage_repository/scripts/crash/gpu-quiesce-run.sh" --)
    if [[ "$crash_point_replay_sample_mode" == 1 ]]; then
      if [[ ! -f "$PWD/.crash-point-replay-sample-tools/multi-host.env" ]]; then
        echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：样本档里没有假的清场配置 $PWD/.crash-point-replay-sample-tools/multi-host.env，带 gpu 特性那一趟不起（起了就是照真的多机配置清场）"
        echo "     → 怎么办：照 .claude/gate.d/fixtures/checker-tier-crash-point-replay.sh/quick-features-green/.crash-point-replay-sample-tools/multi-host.env 放一份只写假清场命令的配置，再跑"
        return 1
      fi
      quiesce_environment=("SINGLEFS_MULTI_HOST_CONFIG=$PWD/.crash-point-replay-sample-tools/multi-host.env" "GPU_QUIESCE_STATE_DIRECTORY=$crash_point_replay_scratch_directory/gpu-quiesce-state"
        "GPU_QUIESCE_LOG_DIRECTORY=$crash_point_replay_scratch_directory/gpu-quiesce-logs")
    fi
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：特性里有 gpu，这一趟经 scripts/crash/gpu-quiesce-run.sh 照多机配置清场再起，跑完不复原（要复原手动跑私有复原脚本，多机配置的 QUIESCE_START_COMMAND）"
  fi
# librocksdb-sys 的 bindgen 要的头文件目录：与崩溃放量取同一个值，编译产物才共用。在子 shell 里 source 那份库、只读这一个变量。
  if [[ -f "$amplification_library" ]]; then
    bindgen_arguments="$(source "$amplification_library" >/dev/null 2>&1; printf '%s' "${CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS:-}")"
  fi
  if [[ -n "$bindgen_arguments" ]]; then
    bindgen_setting=("BINDGEN_EXTRA_CLANG_ARGS=$bindgen_arguments")
  else
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：从 $amplification_library 读不到 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS，BINDGEN_EXTRA_CLANG_ARGS 照调用方环境（${BINDGEN_EXTRA_CLANG_ARGS:-没设}）；librocksdb-sys 编不过时这一格判红"
  fi
  while IFS= read -r environment_name; do
    if [[ "$environment_name" == SINGLEFS_CRASH_AMPLIFICATION_* ]]; then cleared_environment+=(-u "$environment_name"); fi
  done < <(compgen -e)
  feature_log="$crash_point_replay_scratch_directory/quick-singlefs-checker-tier-features.log"
  echo "── $FEATURE_GATED_TESTS_CELL_NAME：带特性那一趟（特性从 $tests_directory 下 ${#gated_rows[@]} 个挂特性的测试文件现读）：cargo test --release -p singlefs-checker-tier --features $feature_list --lib ${target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}"
  if [[ "$crash_point_replay_break" == no-feature-pass ]]; then
    : > "$feature_log"
  else
    env "${cleared_environment[@]}" ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
      ${quiesce_environment[@]+"${quiesce_environment[@]}"} ${bindgen_setting[@]+"${bindgen_setting[@]}"} \
      ${quiesce_prefix[@]+"${quiesce_prefix[@]}"} ${crash_point_replay_thread_prefix[@]+"${crash_point_replay_thread_prefix[@]}"} cargo test --release --no-fail-fast -p singlefs-checker-tier --features "$feature_list" --lib "${target_arguments[@]}" -- --nocapture ${gpu_test_arguments[@]+"${gpu_test_arguments[@]}"} 2>&1 \
      | tee "$feature_log" \
      | { grep --line-buffered '^CRASH_POINT_REPLAY_PROGRESS ' || true; } \
      | sed -u 's/^/    /'
    feature_exit="${PIPESTATUS[0]}"
  fi
  if (( ${#quiesce_prefix[@]} > 0 )); then
    grep '^GPU_QUIESCE ' "$feature_log" | sed 's/^/    /'
    if [[ "$feature_exit" == 78 ]] && grep -q '^GPU_QUIESCE step=command result=not-started' "$feature_log"; then
      echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：带 gpu 特性那一趟没起：清场回读没过或多机配置没写全 QUIESCE_*（清场那两个键；清场包装的原话在 $feature_log）"
      echo "     → 怎么办：卡上有不归清场白名单的计算进程（别的会话的 GPU 活）时清场回读本来就不过：等那个进程跑完再只重跑这一格，不停它：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME；多机配置缺键的照仓根 multi-host.env.example 写全"
      return 1
    fi
  fi
  if [[ "$feature_exit" != 0 && "$crash_point_replay_break" != ignore-feature-pass-exit ]]; then
    tail -40 "$feature_log"
    grep -E '^test .* FAILED$|^error: test failed, to rerun pass' "$feature_log" | sed 's/^/    /' || true
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：带特性那一趟 cargo test 退 $feature_exit（上面是它的尾部与全部红的用例、测试目标）"
    echo "     → 怎么办：单跑看细节：BINDGEN_EXTRA_CLANG_ARGS=\"$bindgen_arguments\" cargo test --release -p singlefs-checker-tier --features $feature_list --lib ${target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}；"
    echo "                编不过多半是 librocksdb-sys 现编 C++ 缺 libclang、C++ 编译器或这个变量；用例红照断言消息改；没卡的机器上 GPU 对拍该打一行 NOT_RUN_NO_GPU_ADAPTER 之后返回成功，panic 就是那条用例没接这个约定。"
    return 1
  fi
  # 配置开了 GPU、这一趟编进了 gpu（文件头「多机配置开了 GPU」）：判段必须在 GPU，日志里有判在 CPU 的行判红
  if [[ ",$feature_list," == *,gpu,* ]] && (( crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-accepted ]]; then
    local feature_cpu_judging=()
    mapfile -t feature_cpu_judging < <(cpu_judging_lines "$feature_log")
    if (( ${#feature_cpu_judging[@]} > 0 )); then
      printf '      %s\n' "${feature_cpu_judging[@]:0:20}"
      echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：带 gpu 特性那一趟判在 CPU：多机配置 $crash_point_replay_configuration_file 写了 ENABLE_GPU=1、这一趟编进了 gpu，日志里有 ${#feature_cpu_judging[@]} 处判段判在 CPU（上面列了前 20 处）"
      echo "     → 怎么办：看那几行的 why_not_the_gpu=（选路在 crates/singlefs-checker-tier/src/crash_state_enumeration.rs 的 crash_point_replay_judge_route）：用例读的配置与这里判的不是同一份，就对齐 SINGLEFS_MULTI_HOST_CONFIG；改完只重跑这一格：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
      return 1
    fi
  fi
  count_tests_run_by_file "$feature_log" ${1:+"$1"} > "$counts_file"
  for row in "${gated_rows[@]}"; do
    IFS=$'\t' read -r file_name file_scope file_features <<< "$row"
    count="$(awk -F'\t' -v name="$file_name" '$1 == name { print $2 }' "$counts_file")"
    count="${count:-0}"
    ran_total=$((ran_total + count))
    if [[ "$count" == 0 && "$crash_point_replay_break" != ignore-zero-feature-files ]]; then
      zero_files+=("tests/$file_name")
      scope_text="逐条挂"; if [[ "$file_scope" == whole-file ]]; then scope_text="整份挂"; fi
      crash_point_replay_report_not_run "checker 档 $tests_directory/$file_name ${scope_text}特性 $file_features，快档跑到它的用例 0 条（cargo 输出里没有它那一段，或只有 ignored）：给它补一条不标 ignored 的快用例，或查 cfg 里的特性名与 Cargo.toml 的 [features] 对不对得上"
    fi
  done
  if [[ "$crash_point_replay_break" != ignore-gpu-not-run ]]; then
    while IFS=$'\t' read -r gpu_where gpu_line; do
      [[ -n "$gpu_line" ]] || continue
      gpu_not_run+=("$gpu_line")
      crash_point_replay_report_not_run "GPU 对拍没跑（${gpu_where}；这台机器找不到 GPU 适配器）：${gpu_line}；在有卡的机器上跑这一格补上"
    done < <(gpu_adapter_not_run_lines "$feature_log")
  fi
  not_run_total=$(( ${#zero_files[@]} + ${#gpu_not_run[@]} ))
  if (( not_run_total > 0 )); then
    feature_cell_state="有 ${not_run_total} 处本次未跑"
    echo "  ! $FEATURE_GATED_TESTS_CELL_NAME 不算全绿：${#gated_rows[@]} 个挂特性的测试文件（特性 $feature_list）跑到 ${ran_total} 条，本次未跑 ${not_run_total} 处：编进来 0 条的文件 ${#zero_files[@]} 个（${zero_files[*]:-无}），没卡没跑的 GPU 对拍 ${#gpu_not_run[@]} 条"
    echo "     → 编进来 0 条的给它补不标 ignored 的快用例；GPU 对拍在有卡的机器上补跑：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
  else
    feature_cell_state="绿"
    echo "  ✓ $FEATURE_GATED_TESTS_CELL_NAME：${#gated_rows[@]} 个挂特性的测试文件（特性 $feature_list）每个都编进来、跑到了用例，共 ${ran_total} 条"
  fi
  return 0
}

# ── 快档：crash-case-replay 格与 feature-gated-tests 格 ─────────
if [[ "$crash_point_replay_tier" == quick ]]; then
  quick_red_cells=()
  quick_cell_states=()
  quick_tier_log=""
  if crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
    quick_tier_report=""
    if run_checker_quick_tier; then
      quick_tier_log="$crash_point_replay_scratch_directory/quick-singlefs-checker-tier.log"
      check_crash_case_markers
      # 不带特性那一趟用例报的本次未跑（文件头「多机配置开了 GPU」）：逐格汇总里点出来，不算全绿
      if (( crash_point_replay_plain_not_run_total > 0 )); then
        if [[ "$crash_point_replay_cell_state" == 绿 ]]; then crash_point_replay_cell_state="不带特性那一趟有 ${crash_point_replay_plain_not_run_total} 处本次未跑"
        else crash_point_replay_cell_state+="；不带特性那一趟有 ${crash_point_replay_plain_not_run_total} 处本次未跑"; fi
      fi
      quick_cell_states+=("$CRASH_CASE_REPLAY_CELL_NAME：$crash_point_replay_cell_state")
    else
      quick_red_cells+=("$CRASH_CASE_REPLAY_CELL_NAME")
      quick_cell_states+=("$CRASH_CASE_REPLAY_CELL_NAME：红")
    fi
  fi
  if crash_point_replay_runs_cell "$FEATURE_GATED_TESTS_CELL_NAME"; then
    if run_feature_gated_tests_cell "$quick_tier_log"; then
      quick_cell_states+=("$FEATURE_GATED_TESTS_CELL_NAME：$feature_cell_state")
    else
      quick_red_cells+=("$FEATURE_GATED_TESTS_CELL_NAME")
      quick_cell_states+=("$FEATURE_GATED_TESTS_CELL_NAME：红")
    fi
  fi
  quick_cell_summary=""
  for quick_cell_state in "${quick_cell_states[@]}"; do quick_cell_summary+="${quick_cell_summary:+；}${quick_cell_state}"; done
  echo "  · 逐格：${quick_cell_summary}"
  if (( ${#quick_red_cells[@]} > 0 )); then
    echo "  ✗ 快档有 ${#quick_red_cells[@]} 格判红：${quick_red_cells[*]}（各自的出路在上面）"
    echo "     → 怎么办：照上面那几格各自的「→ 怎么办」改完，只重跑红的格：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $(IFS=,; printf '%s' "${quick_red_cells[*]}") [项目根]"
    exit 1
  fi
  if [[ "$crash_point_replay_sample_mode" == 1 ]]; then
    echo "  ! 样本档（.crash-point-replay-sample-tools/quick-tier，假 cargo）：快档流程判绿，退 3 不退 0——这一趟没跑过真的 checker 档"
    echo "     → 真跑要在不带 .crash-point-replay-sample-tools 的树里跑快档。"
    exit 3
  fi
  exit 0
fi

# ── --full：逐条崩溃枚举用例照复用判定跑 ─────────
full_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
start_over_note="不带 --start-over：判过的块照家目录下崩溃枚举用例那一类的长期库复用"
if [[ "$crash_point_replay_start_over" == 1 ]]; then start_over_note="带 --start-over：每条用例换一份新的空库（<库的家目录>/crash-point-replay-start-over/<输入指纹>，开跑前清空、跑完留着）、从头判"; fi
echo "  · --full 开跑（${full_started_utc}）：${#crash_case_rows[@]} 条崩溃枚举用例逐条照复用判定跑（这批输入那一格全绿标记在就复用）；${start_over_note}"
# 每趟一个新目录，不拿上一趟的输出顶上
crash_point_replay_log_directory="$git_common_directory/singlefs-crash-point-replay-logs/$(date -u +%Y%m%dT%H%M%SZ)-$$"
if ! mkdir -p -- "$crash_point_replay_log_directory"; then
  echo "  ✗ 建不了这一趟的日志目录 $crash_point_replay_log_directory：全量日志与发现日志没处放"
  echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full。"
  exit 1
fi
echo "  · 这一趟的全量日志与发现日志放 ${crash_point_replay_log_directory}/（log.<用例名> 与 log.<用例名>.findings.tsv，跑完不删）"
crash_point_replay_shard_driver="$ROOT/scripts/crash/crash-point-replay-shard-run.sh"
if crash_point_replay_shard_configuration_note="$(bash "$ROOT/scripts/crash/crash-point-replay-shard-configuration-check.sh" "$ROOT" 2>&1)"; then
  crash_point_replay_sharded=1
  echo "  · 双机分片：开（${crash_point_replay_shard_configuration_note//$'\n'/；}）；登记了 shard=across-machines 的用例两台各跑一片、本机 merge，别的单机跑"
else
  crash_point_replay_sharded=0
  echo "  · 双机分片：关（${crash_point_replay_shard_configuration_note//$'\n'/；}）；每条用例单机跑"
fi
red_cases=()
green_cases=()
reused_cases=()
for crash_case_row in "${crash_case_rows[@]}"; do
  IFS=$'\t' read -r case_key case_package case_target case_function <<< "$crash_case_row"
  case_label="${case_key#crash-case:}"
  manifest_at_start="$crash_point_replay_scratch_directory/manifest-at-start.$case_label"
  if ! write_crash_case_manifest "$case_key" "$manifest_at_start"; then
    echo "  ✗ $case_key：算不出这批输入的指纹：$case_manifest_problem"
    echo "     → 怎么办：在项目根跑 git ls-files -co --exclude-standard -- ${crash_point_replay_input_paths_text} 看列不列得出文件，再跑 cargo -V && rustc -V；"
    echo "                单跑 python3 scripts/gate/admission.py crash-case-manifest <根> $case_key <清单文件> --judging-digest --toolchain --build-environment 看它报什么。"
    red_cases+=("$case_key")
    continue
  fi
  fingerprint_at_start="$case_fingerprint"
  file_count_at_start="$case_file_count"
  excluded_count_at_start="$case_excluded_count"
  if marker_check_output="$(python3 "$crash_point_replay_admission_module" crash-case-marker-check "$ROOT" "$case_key" "$fingerprint_at_start" "$manifest_at_start")"; then
    read -r _ok_word reused_marker_path reused_finished_utc <<< "$(head -1 <<< "$marker_check_output")"
    echo "  · $case_key 复用：这批输入（指纹 ${fingerprint_at_start:0:16}…）那一格全绿标记跑完于 ${reused_finished_utc}，这一趟不跑；要重跑就删掉 ${reused_marker_path}"
    reused_cases+=("$case_key")
    continue
  fi
  if ! read_crash_case_command "$case_key" "$fingerprint_at_start"; then
    echo "  ✗ $case_key：准入模块交不出起用例的命令：$case_command_problem"
    echo "     → 怎么办：单跑 python3 scripts/gate/admission.py crash-case-command <根> $case_key <指纹> 看它报什么（取不到 git common-dir、"
    echo "                本机核数读不到、SINGLEFS_CRASH_POINT_REPLAY_THREADS 不是正整数、每台默认重活线程数的登记取不了都在这里报）；修好之后重跑 --full。"
    red_cases+=("$case_key")
    continue
  fi
  if [[ "$case_library_directory" != default ]]; then
    rm -rf -- "${case_library_directory:?}"
    if ! mkdir -p -- "$case_library_directory"; then
      echo "  ✗ $case_key：建不了 --start-over 的空库 $case_library_directory"
      echo "     → 怎么办：看那一目录可不可写、盘满没满；修好之后重跑 --full --start-over。"
      red_cases+=("$case_key")
      continue
    fi
    echo "  · $case_key --start-over：换一份新的空库 $case_library_directory（跑完留着，要清就删它）"
  fi
  case_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  case_log="$crash_point_replay_log_directory/log.$case_label"
  case_findings_file="$case_log.findings.tsv"
  rm -f -- "$case_log" "$case_findings_file"
  case_threads_origin_text="没设，取每台默认重活线程数的登记值"
  if [[ "$case_threads_origin" == explicit ]]; then case_threads_origin_text="显式设的"; fi
  case_threads_note="SINGLEFS_CRASH_POINT_REPLAY_THREADS=${case_threads}（${case_threads_origin_text}），本机 ${case_machine_cores} 核"
  case_sharded=0
  if [[ "$crash_point_replay_sharded" == 1 ]] && python3 "$crash_point_replay_admission_module" crash-case-shardable "$ROOT" "$case_key" >/dev/null; then case_sharded=1; fi
  case_way="单机跑：${case_command[*]}"
  if [[ "$case_sharded" == 1 ]]; then
    case_way="双机跑（本机 0/2、第二台 1/2 各写各的库，取回，本机带导入再跑一趟；bash scripts/crash/crash-point-replay-shard-run.sh --merged-log $case_key <树根> <指纹> <日志> $case_target $case_function）"
    case_threads_note="本机那两趟 ${case_threads_note}，第二台那一趟的线程由双机驱动取登记的重活默认并行度（CRASH_AMPLIFICATION_PEER_THREADS 设了照它）"
  fi
  echo "  · $case_key 开跑（${case_started_utc}；${case_way}；${case_threads_note}；这批输入的指纹 ${fingerprint_at_start:0:16}…，${file_count_at_start} 个文件，减去用例读不到的 ${excluded_count_at_start} 个）："
  sed 's/^/      /' <<< "$marker_check_output"
  echo "    全量日志：${case_log}；发现日志：${case_findings_file}"
  if [[ "$case_sharded" == 1 ]]; then
    run_crash_case_in_two_shards "$case_key" "$case_log" "$fingerprint_at_start" "$case_target" "$case_function"
  else
    run_crash_case "$case_log" "$case_findings_file" "${case_command[@]}"
  fi
  case_run_exit=$?
# 发现表跑完就打（判红的那一趟最要看它），判定留到下面
  if report_crash_case_findings "$case_findings_file" "$case_log"; then case_findings_exit=0; else case_findings_exit=$?; fi
  if [[ "$case_run_exit" != 0 ]]; then
    [[ -f "$case_log" ]] && tail -40 "$case_log"
    delete_crash_case_marker "$case_key" "$fingerprint_at_start"
    if [[ "$case_sharded" == 1 ]]; then
      echo "  ✗ $case_key 判红：双机那一趟退 $case_run_exit（上面是驱动脚本的输出与导入那一趟日志的尾部）"
      echo "     → 怎么办：驱动脚本输出里判红的那一句说清卡在哪一步（工具链、指纹、某一片、账本、merge）；要单机复核，挪开本地配置 multi-host.env 再跑 --full"
      red_cases+=("$case_key")
      continue
    fi
    echo "  ✗ $case_key 判红：cargo test 退非 0（上面是它的尾部）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p $case_package --test $case_target -- --include-ignored --exact $case_function --nocapture"
    echo "                断言消息里是第一处对不上的计数或违例。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
    red_cases+=("$case_key")
    continue
  fi
  judged_lines_file="$crash_point_replay_scratch_directory/judged.$case_label"
  if ! judge_output="$(python3 "$crash_point_replay_admission_module" crash-case-judge "$ROOT" "$case_key" "$case_log" "$judged_lines_file")"; then
    delete_crash_case_marker "$case_key" "$fingerprint_at_start"
    printf '%s\n' "$judge_output" | sed 's/^/       /'
    echo "  ✗ $case_key 的用例跑过了，日志却判不绿（上面逐条列出）"
    echo "     → 怎么办：计数行不是恰好一行、过滤之后没跑到恰好一条用例，对一对 stage-inputs.tsv 里 $case_key 那一行第三列与用例打印的行；"
    echo "                不是全量去 crates/singlefs-checker-tier/src/crash_state_enumeration.rs 的 enumerate_crash_states 看；只起了 1 个线程看 crash_point_replay_worker_threads 读没读到 SINGLEFS_CRASH_POINT_REPLAY_THREADS，"
    echo "                真要单线程跑（比对单进程读数），显式写 SINGLEFS_CRASH_POINT_REPLAY_THREADS=1 bash .claude/gate.d/checker-tier-crash-point-replay.sh --full <根>。"
    red_cases+=("$case_key")
    continue
  fi
  if [[ "$case_findings_exit" != 0 ]]; then
    delete_crash_case_marker "$case_key" "$fingerprint_at_start"
    echo "  ✗ $case_key 的用例跑过了、日志也判得绿，发现日志却判红（上面发现表里的「问题：」逐条列出）"
    echo "     → 怎么办：「这一趟没跑完」看全量日志 $case_log 的尾部是 panic 还是被杀；red_states 不是 0 是有状态判红而用例没红，"
    echo "                对一对用例的计数断言与 crates/singlefs-checker-tier/src/crash.rs 的发现表；发现日志不在而全量日志里有 CRASH_POINT_REPLAY_FINDINGS 行，"
    echo "                是 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 没传进用例（单机看本阶段 run_crash_case，双机看 scripts/crash/crash-point-replay-shard-run.sh 给导入那一趟的设置）。"
    red_cases+=("$case_key")
    continue
  fi
  manifest_at_finish="$crash_point_replay_scratch_directory/manifest-at-finish.$case_label"
  if ! write_crash_case_manifest "$case_key" "$manifest_at_finish"; then
    echo "  ✗ $case_key 跑完之后算不出这批输入的指纹：$case_manifest_problem；不写全绿标记"
    echo "     → 怎么办：多半是跑的过程中有人删了登记路径下的文件或工具链坏了；等改动停下，在 HEAD + 暂存区的 worktree 里重跑 --full。"
    red_cases+=("$case_key")
    continue
  fi
  if [[ "$case_fingerprint" != "$fingerprint_at_start" ]]; then
    echo "  ✗ $case_key 跑的过程中它的输入变了（开跑 ${fingerprint_at_start:0:16}…，跑完 ${case_fingerprint:0:16}…）：读到的不一定是同一版，不写全绿标记；开跑那一批已有的那一格不删"
    report_manifest_differences "$manifest_at_start" "$manifest_at_finish" "开跑时" "跑完时"
    echo "     → 怎么办：别在有人改这些路径的树里跑。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 checker-tier-crash-point-replay 跑 --full <它的根>（与 gate.sh --staged 同一建法）："
    print_staged_worktree_full_commands
    red_cases+=("$case_key")
    continue
  fi
  threads_text="${judge_output//$'\n'/；}${judge_output:+；}${case_threads_note}"
  if ! record_output="$(python3 "$crash_point_replay_admission_module" crash-case-record "$ROOT" "$case_key" "$fingerprint_at_start" "$manifest_at_start" "$judged_lines_file" \
      --files "$file_count_at_start" --excluded "$excluded_count_at_start" --started "$case_started_utc" --judged-root "$ROOT")"; then
    echo "  ✗ $case_key 判绿，全绿标记却没写成：$record_output"
    echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full（没有标记，整轮门禁的快档会一直红）。"
    red_cases+=("$case_key")
    continue
  fi
  echo "  ✓ $case_key 判绿（${threads_text}）：全绿标记写进 ${record_output}，记下的行原样："
  sed 's/^/      /' "$judged_lines_file"
  green_cases+=("$case_key")
done
full_finished_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if (( ${#red_cases[@]} > 0 )); then
  echo "  ✗ --full 有 ${#red_cases[@]} 条崩溃枚举用例判红：${red_cases[*]}（这一趟判绿 ${#green_cases[@]} 条、复用 ${#reused_cases[@]} 条；开跑 ${full_started_utc}，跑完 ${full_finished_utc}）"
  echo "     → 怎么办：逐条照它自己那一句「→ 怎么办」改；改完暂存，再在 HEAD + 暂存区的 worktree 里跑 --full（这一趟判绿的与复用的那几条下一趟照样复用）。"
  exit 1
fi
echo "  ✓ --full 跑完（开跑 ${full_started_utc}，跑完 ${full_finished_utc}）：${#crash_case_rows[@]} 条崩溃枚举用例，这一趟跑了判绿 ${#green_cases[@]} 条（${green_cases[*]:-无}），复用 ${#reused_cases[@]} 条（${reused_cases[*]:-无}）；日志在 ${crash_point_replay_log_directory}/"
if [[ "$crash_point_replay_sample_mode" == 1 ]]; then
  echo "  ! 样本档（.crash-point-replay-sample-tools，假 cargo）：流程判绿，退 3 不退 0——这一趟没跑过真的崩溃点重放"
  echo "     → 真跑要在不带 .crash-point-replay-sample-tools 的树里跑 --full（建法见快档的出路句）。"
  exit 3
fi
