#!/usr/bin/env bash
# admission: always 被 scripts/experiment-apparatus/replay.sh 在同目录 source 进去的那一段，判的是此刻的仓与这一趟的参数，上一次的结论不替这一次作保
# run-condition: none 不单独执行：由 replay.sh 的 source 载入，环境要求同 replay.sh（文件头「GPU」「编译目录」）
# replay-selftest.sh：replay.sh 的一段（--selftest 的全部自证（replay_selftest 与它调的各格））。只定义函数与载入时赋值的全局量，不改 shell 选项。
# 这一段按 replay.sh 里 source 它的那一行的位置载入。文件头的用法、复跑表、弄坏开关表、退出码都在 replay.sh。
# 自证（bash scripts/experiment-apparatus/replay.sh --selftest）：PATH 最前面放一个假 cargo，只演 build_research_binaries 的两种结局，不编、不跑任何实验。
#   build-red    假 cargo 打一行 error[E0425] 退 101：要返回 2，stderr 有第一处错、日志末尾、日志路径与重现命令，日志落了盘
#   build-green  假 cargo 退 0：要返回 0，stderr 一个字都没有
#   gpu-features-on / -off / -garbled / -no-such / -unreadable / -build-red  配置开了 GPU 的特性（resolve_checker_tier_features，假 cargo 记参数，配置文件放临时目录）：
#     写 ENABLE_GPU=1 → 试编一次、CHECKER_TIER_FEATURES 是 --features gpu；写 0 或没写 → 不带、不起 cargo；写 yes、点名的文件不在、文件不可读 → 78 点名原因；
#     开了而试编红 → 78、第一处错、日志路径、「不退回不带 gpu 特性去跑」
#   run-lines-with-features  起 实验包装置的 cargo run 行（本文件里 grep 出来）每一行都带数组
#   gpu-probe-takes-no-cards  开了 GPU、环境里没有 GPU_CARDS_BY_PRIORITY（多机配置里的清场命令是假的、只往临时目录的日志记一笔）：resolve_checker_tier_features 返回 0，
#     照装置那样直接起的命令看不到卡（不领卡、不设卡串），假清场命令一笔都没记（不另套清场）
#   run-lines-without-quiesce-wrapper  起 实验包装置的 cargo run 行没有一行另套清场，本文件不是注释的行里也不起清场包装
#   row-key      一张两行的假键表（E1 第二行指明 E1/second）：admission_key_of_row 给第二行的键要是 E1/second，不是实验号
#   jobs-default 没设 SINGLEFS_THREAD_CAP 时 default_replay_jobs 要等于登记文件第一条非注释行的数；设成 7 时要是 7
#   整条按行认键（replay-row-keys 判红、按行问准入、产物头由这里写）在临时仓里跑的那几格在 python3 scripts/gate/admission.py --selftest 的「复跑表按行的准入键」
#   负载关（假产物放临时目录，阈值钉 0.5，timing_load_decide 判，不起任何实验）：
#     load-high    收尾行 loadavg1=20.50 cpus=32（比值 0.640625）→ skip，点名比值
#     load-low     loadavg1=2.50 cpus=32 → judge
#     load-none    收尾行没有字段、不在滞后表 → red，点名产物与「收尾行没有 loadavg1 / cpus」
#     load-lagged  收尾行没有字段、产物登记在滞后表 → judge 并记滞后
#     load-exact   判据写 exact 的行不看字段 → judge
#     lag-table-bad  滞后表登记复跑表里没有的名字、exact 行的产物、重复名字、理由不足 8 个字、用空格而不是制表符隔开理由 → 逐条点名（load_timing_lag）
#     lag-table-stale  滞后表登记的留存产物自己已经有字段 → 点名叫人删这一行
#     strip-load   strip_load_fields 只删 name=done 行上的 loadavg1= 与 cpus=
#   按行复用（replay_reuse_selftest：临时仓里两行的合成复跑表，E1 留存产物在、E2 的留存产物只在 git 历史里；这一份 replay.sh 的副本、假 cargo（记下每一次收到的参数）、假 rustc、
#   两个假 research 二进制，真仓的 .claude/scripts、.claude/singlefs-ai-sop、scripts/gate 链进来，判定库在临时 prune-server 里、库名取临时仓自己的 git common-dir
#   （键带临时目录的命名空间；库里有几条经 admission.py library-files fetch 拉进这一格的临时目录数），接连跑五趟：
#     reuse-first-run      头一趟两行都跑：E1 字节一致、E2 字节一致（留存产物从 git 历史取回），判定库记两份（各带组成清单），临时仓里不建 singlefs-unit-verdicts/
#     reuse-second-run     第二趟两行都复用、一个装置都没跑、cargo build 一次都不起（全复用就不编），REPLAY_TOTAL 报 reused=2 judged=0
#     reuse-device-source  改 E1 装置 main 的一行代码：只重跑 E1、只编 --bin e1-sample，开跑前报出变了 fn e1_sample.rs::main，E2 照旧复用
#     reuse-comment-only   只改 E2 装置源码的一行注释：两行都复用（行键取单元的去注释指纹，注释不算）
#     reuse-three-questions  复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：上面头一趟、第二趟（①）、只改注释（②）、改 E1 装置 main（③，affected E1）
#                          照共用库判、打规范输出「REUSE_SELFTEST script=scripts/experiment-apparatus/replay.sh …」；reuse-second-run、reuse-comment-only、reuse-device-source 三格在它里面点名
#     last-run-list        上次红的清单（文件头「三」那一份共用库）：全绿的几趟之后是空文件
#     reuse-sentinel       复用哨兵（共用库文件头「二」）：种子 5、哨兵 1 条那一趟绿、抽中一行强制现判；开着 REPLAY_BREAK=reuse-ignored 判红「复用失灵」
#                          （这两趟与它们前面清空开关垫的一趟，REPLAY_BREAK 自己定，不接调用方设的开关：别的开关打开时这一格不跟着红）
#     reuse-red-withdraws-green  记红、撤绿、下趟现判：清空开关垫一趟（E1 字节一致入库）；再跑一趟，复用哨兵（哨兵 1 条、种子 5：E2 当哨兵、E1 被抽成强制现判）让 E1 同键现判，E1 的假装置（REPLAY_SELFTEST_DRIFT_DEVICE=e1，装置在仓外不进键）演出另一份产物
#                          ⇒ E1 对不上、整趟退 1，库里这把行键记成 drift；下一趟：只起 e1、开跑前写「E1 要现跑：上次判红」、判回字节一致，E2 复用；再下一趟两行全复用
#     reuse-broken-withdraws-green  跑不了也记红、撤绿：清空开关垫一趟（E1 字节一致入库）；再跑一趟，复用哨兵（哨兵 1 条、种子 5）让 E1 同键现判，E1 的假装置退 3（REPLAY_SELFTEST_BROKEN_DEVICE=e1，装置在仓外不进键）
#                          ⇒ E1 跑不了、整趟退 1，库里这把行键记成 broken；下一趟：只起 e1、开跑前写「E1 要现跑：上次判红」、判回字节一致，E2 复用；再下一趟两行全复用
#     reuse-outside-git-exits-2  另搭一棵样本仓、删掉它的 .git，跑一趟：整趟退 2、点名不是 git 工作树、一个装置都没起、不打 REPLAY_TOTAL
#     reuse-unreachable-exits-75  服务连不上（socket 不在、TCP 端口刚放掉、重连总时长 2 秒）：整趟退 75、点名 prune-server 不可用、一个装置都没起、不打 REPLAY_TOTAL
#     reuse-gpu-only-for-feature-rows  另搭一棵样本仓、复跑表加一行点到 CHECKER_TIER_FEATURES 的驱动 E3，多机配置 ENABLE_GPU=1、环境里没有卡串：
#                          只跑 E1 E2 退 0、两行都起、不试编 gpu 特性、stderr 写「都不点 CHECKER_TIER_FEATURES」；样本仓里放 E3 的留存产物 e3-sample.out（与驱动打的字节相同），
#                          只跑 E3 退 0、试编了 gpu 特性、E3 的驱动带着 --features gpu 起了（驱动的 stderr 落在这一趟输出目录里这一行的 .err）
#   这几趟的哨兵数设 1000（键相同的全当哨兵、不抽强制现判），reuse-sentinel 那两趟除外；上次红的清单写临时目录（SINGLEFS_LAST_RUNS_DIRECTORY）
#   这份自证里起样本仓的副本，每一处都带字面实验号或行项（样本表只有 E1、E2 的趟写 E1 E2，派行各格写这一格表里的全部实验号）：不带实验号、只带选项或参数折不出的起法
#   让 --selftest 的单元闭包连整张真复跑表与表里的驱动（scripts/gate/lib_function_units.py 文件头「shell 起法带字面参数」）。所以不给实验号那一趟才走的几支
#   （want 在 ONLY 为空时放行每一行、上次红的清单整份换）与经入口起的 --list-items 不在自证里起
#   留存产物取回与装置读不到输入（replay_archived_selftest：同一份临时仓，E2 的留存产物只在 git 历史里）三格：
#     archived-retrieved-green    E2 字节一致、结果行写出处「取自 git show <提交>^:research/results/e2-sample.out」、整趟退 0
#     archived-unrecoverable-red  E2 的留存产物名改成历史里也没有的：判「取不回」、结果行点名这个产物、整趟退 1
#     device-not-run-red          E1 的假装置打一行 NOT_RUN_SAMPLE_INPUT 退 77：记跑不了（退出码 77）、整趟退 1，没有「本次未跑」的一档
#   留存产物只以 .out.gz 存在时的比对（replay_gzip_selftest：同一份临时仓，E1 的留存产物改成 e1-sample.out.gz）：
#     gz-stored-green      先解压再比：E1 字节一致、整趟退 0
#     gz-stored-red        换成内容不同的 gzip：E1 对不上、整趟退 1
#   驱动开跑前核输入（replay_inputs_present）：driver_e142、E170 两行判镜像、E171 第一段读冻结参照的七行、E171 第 1 次重跑三个比对器、
#     E174 三个 judge、E158 r3 / r4 比对、E165 七个变异杀伤界驱动，从本文件取出定义，在没有 research/、冻结根与导出根都没给的空目录里各起一次：
#     都要退 1、stderr 里点名自己的「✗ 输入不在 test=<驱动名>」行数恰好是缺的样数（E158 r4 六十六份臂产物逐份报），stdout 不打
#   E171 整表那四行只核账（replay_e171_library_selftest，一格 e171-missing-row-refused）：样本仓里放假的取行程序（shared_steps.py 的 take、take-rows）与假比对器，
#     库里缺一行时 driver_e171_s2_comparator、driver_e171_s4_comparator、driver_e171_s4_f4_comparator、driver_e171_s4_f4_substitutes_comparator 各起一次：
#     都要退非 0、stderr 有分片入口的整条命令（带范围声明）、stdout 不打，取行程序之外一笔都没记（假 cargo 与比对器一次没起）
#   编译目录（文件头「编译目录」；replay_build_directory_selftest：同一份临时仓，假 cargo 编的时候把假装置拷进 CARGO_TARGET_DIR 的 release/）六格：
#     跑绿、编不过、装置开跑之后被 TERM 三种结局之后这一格的 TMPDIR 底下都不剩这一趟的编译目录，调用方给了 CARGO_TARGET_DIR 照它编且不删，
#     服务给的 CARGO_TARGET_DIR 里有上一次尝试编出的产物：照它编、那份产物照留、不删（service-build-directory-kept），
#     装置在本轮输出目录里造的带 CACHEDIR.TAG 的副本编译目录跑完不在、本轮输出照留
#   派行与进度（文件头「派行」「进度」；replay_dispatch_selftest：每格一棵样本仓，复跑表换成这一格的几行，假装置记起止与看到的卡并在 REPLAY_SELFTEST_BARRIER 条起来之前等着，上限经 TASK_SERVER_MEMORY_CAP_BYTES 交，峰值记录放这一格的临时目录；
#   要卡那几格在环境里给卡串 GPU_CARDS_BY_PRIORITY=selftest-card，没有卡串那一格不给）十七格：
#     dispatch-memory-bound         三行量过、各 1 GiB，上限 3 GiB，多机配置没写这一台能给的内存：同时至多两行，也真有两行一起跑，标准错误写读数脚本 host-cap 那一行报的取不到的原因
#     dispatch-task-cap-not-binding  三行量过、各 1 GiB，服务交的上限 3 GiB、多机配置这一台那一项 64G：三行一起跑（比这一台能给的内存，不拿这一次拿到的上限压）
#     dispatch-threads-unbounded    三行各量过 31 线程、REPLAY_JOBS=8：三行一起跑，线程不累计
#     dispatch-row-count-bound      REPLAY_JOBS=2，三行单线程：同时恰好两行
#     dispatch-exclusive-rows       timing 行单独跑、排在别的行之后，没量过的一行与量过的两行一起跑，结果行照旧按表序；
#                                   样本仓的每台默认重活线程数取法钉成 16（大于这一格的 REPLAY_JOBS=8）、设成只读函数（副本读多机配置时 source 的库盖不掉），
#                                   没量过的那一行照它算线程，它至少要被调到一次
#     dispatch-gpu-rows-serial      要卡的两行（驱动，表里排在不要卡的那一行前面）挪到最后：不要卡的一行先跑完、看不到卡，两行要卡的不同时跑、都看到环境里给的那一串，整趟退 0
#     dispatch-gpu-rows-without-cards  同形的三行，环境里没有卡串：照样不拦，要卡的两行一行一行跑、看不到卡，三行都判字节一致，整趟退 0
#     dispatch-driver-leased-rows   要卡的两行（一行登记成本脚本不交卡、排在表里最前，一行照常交卡）加不要卡的一行：卡串只交给照常交卡的那一行，
#                                   登记的那一行在那一段跑完之后跑、看不到卡，三行都字节一致，整趟退 0
#     dispatch-oversized-row-refused  量过 40 GiB、上限 3 GiB：退 250、点名那一行、不编、一个装置都不起
#     row-progress-lines            每一行现跑的起止各一行、峰值是正整数，在跑的报进度，标准输出里没有这几行
#     row-peak-recorded-then-shared  跑一趟量到的峰值进记录，改一处装置源码（行键变、照现跑）再跑一趟：这两行一起跑
#     row-failed-peak-not-recorded  装置退 3 的那一行不记峰值
#     stopped-keeps-finished-rows   一行判完、下一行在跑时发 TERM：退 143，标准输出有判完那一行的结果行与 REPLAY_STOPPED，没有在跑那一行的
#     dispatch-item-selects-one-row  一个实验号三行：--item E1.2 只起第 2 行、E1 --item E1.3 只起第 3 行，派出都是 1 行，退 0
#     dispatch-item-unknown-refused  --item E1.9 与 E1 --item（缺项名）退 2、点名项名、列出 E1 的行项、不建本轮输出目录、一个装置都不起
#     dispatch-deferred-row-not-run  登记成输入缺的行（REPLAY_DEFERRED_ROW_ROOT，导出根不在）记「本次未跑」、装置不起、整趟退 0，REPLAY_TOTAL 带 not_run=1；只剩这一行时整趟退 77
#     e181-goes-through-long-run    E181 这一行不带参数起长跑脚本（这一格里是假的）、经 E181_PRODUCT_DIRECTORY 取它拼出的产物交给判法，不起 cargo run；驱动正文字面点名装置（行键收装置 main 靠它）
# 弄坏开关：
#   REPLAY_BREAK=build-output-dropped  构建输出丢进 /dev/null、红了只说「replay: 构建失败」
#       build-red（「✗ 自检：构建失败时应当返回 2、打第一处错」）
#       连带红的格：「✗ 自检：按行复用 reuse-device-source：改了 E1 装置 main 的一行代码」、「✗ 自检：构建失败时」、「✗ 自检：构建成功时错误警告日志也要在、末行写收了 0 行（build-green.problems.log）；实际：cat: … No s…」
#   REPLAY_BREAK=build-log-single  构建日志（build.log、gpu-build.log）只写一份全量的（不经 two-logs.sh，没有 .problems.log）
#       build-two-logs（「✗ 自检：构建失败时应当出两份日志…」）、构建成功那一格的错误警告日志（「✗ 自检：构建成功时错误警告日志也要在、末行写收了 0 行」）
#   REPLAY_BREAK=gz-read-raw  留存产物 .gz 不解压、原样读（read_stored 的 *.gz 分支退成 cat）
#       gz-stored-green（「✗ 自检：留存产物 .gz gz-stored-green：…」）
#   REPLAY_BREAK=driver-build-output-dropped  驱动开跑前编研究二进制不指 scripts/experiment-apparatus 工作区、输出丢进 /dev/null
#       driver-build（「✗ 自检：驱动开跑前编研究二进制（build_research_driver_binary）要经 --manifest-path …」）
#       与 E162、E163 那三个驱动各一格（「✗ 自检：driver_e162_anchors 开跑前要经 build_research_driver_binary 编…」，driver_e162_power_cut_selftest、driver_e163_r1_merge 同）
#       连带红的格：「✗ 自检：driver_e162_power_cut_selftest 开跑前要经 build_research_driver_binary…」、「✗ 自检：driver_e163_r1_merge 开跑前要经 build_research_driver_binary 编（cargo 只…」
#   REPLAY_BREAK=row-key-by-experiment  每一行一律按实验号问准入
#       row-key（「✗ 自检：复跑表那一行指明了准入键 E1/second」）；admission.py --selftest 设着它跑，『复跑表按行的准入键』那几格转红
#   REPLAY_BREAK=load-field-ignored  不看负载字段：超阈值的与没字段又不在滞后表的，都照判
#       load-high（「✗ 自检：负载 20.5 / 32 核超过阈值 0.5，这一行应当记本次不判并点名比值」）、load-none（「✗ 自检：收尾行没有字段又不在滞后表，这一行应当判红…」）两格转红
#   REPLAY_BREAK=gpu-features-dropped  配置开了 GPU 也不带 gpu 特性
#       gpu-features-on（「✗ 自检：配置写了 ENABLE_GPU=1，应当先试编 gpu 特性…」）
#   REPLAY_BREAK=run-lines-without-features  自检里核「cargo run 行都带特性数组」时当没有数组（模拟新加的一行忘了带）
#       run-lines-with-features（「✗ 自检：起 实验包装置的 cargo run 行…应当每一行都带…」）
#   REPLAY_BREAK=run-line-quiesce-wrapped  自检里核「不另套清场」时多塞一行经清场包装起的 cargo run（模拟新加的一行又套了清场）
#       run-lines-without-quiesce-wrapper（「✗ 自检：起 实验包装置不另套清场」）
#   REPLAY_BREAK=reuse-ignored  按行复用的判定库只记不读（每一趟全跑）
#       reuse-second-run、reuse-device-source、reuse-comment-only、reuse-three-questions（『✗ 自检：按行复用 <格名>：』）
#       连带红的格：「✗ 自检：按行复用 reuse-second-run：第二趟行键没变」、「✗ 自检：按行复用 reuse-device-source：改了 E1 装置 main 的一行代码」、「✗ 自检：按行复用 reuse-comment-only：只改 E2 装置源码的一行注释」、「✗ 自检：按行复用 reuse-three-questions：reuse-three-questions：① 同一份输入跑两趟：第二趟现判…」
#   REUSE_CHECKS_BREAK=rejudged-sentinel-passes（共用库 scripts/gate/lib_reuse_checks.py 的开关）  哨兵被现判不红
#       reuse-sentinel（「✗ 自检：按行复用 reuse-sentinel：」）
#   REPLAY_BREAK=red-not-recorded  对不上的行不记红（只记绿，判红的不撤同一把行键原有的绿）
#       reuse-red-withdraws-green（「✗ 自检：按行复用 reuse-red-withdraws-green：」）
#   REPLAY_BREAK=broken-not-recorded  现跑了而跑不了的行不记红（不撤同一把行键原有的绿，下一趟照复用）
#       reuse-broken-withdraws-green（「✗ 自检：按行复用 reuse-broken-withdraws-green：」）
#   REPLAY_BREAK=row-library-failure-tolerated  按行复用的库没法用（不在 git 工作树里、行键整份算不出、中转目录建不了、记录拉不下来）时只打一句、往下全跑
#       reuse-outside-git-exits-2（「✗ 自检：按行复用 reuse-outside-git-exits-2：」）
#   REPLAY_BREAK=prune-server-failure-swallowed  判定库服务连不上（replay-row-inputs、library-files 退 75）不按 75 办（照别的失败退出或照记不下往下跑），不退 75
#       reuse-unreachable-exits-75（「✗ 自检：按行复用 reuse-unreachable-exits-75：」）
#   REPLAY_BREAK=reuse-key-by-row-only  按行复用的键只按复跑表那一行算，不看装置源码与工具链
#       reuse-first-run（「✗ 自检：按行复用 reuse-first-run：」：键不经准入模块算，判定库那两份不带组成清单）、
#       reuse-device-source（「✗ 自检：按行复用 reuse-device-source：」）
#       连带红的格：「✗ 自检：按行复用 reuse-three-questions：reuse-three-questions：③ 改闭包里一个函数单元：闭包含…」；
#       另外十格：键不看装置源码，同一份临时库里同名的行键相同，这几格本该真起装置的那一趟照复用、装置不跑：
#       「✗ 自检：留存产物取回 device-not-run-red：」、「✗ 自检：留存产物取回 archived-unrecoverable-red：」、「✗ 自检：留存产物 .gz gz-stored-red：」、
#       「✗ 自检：入口与完整性闸 integrity-previous-output：」、「✗ 自检：编译目录 build-directory-removed-when-build-fails：」、
#       「✗ 自检：编译目录 build-directory-removed-when-stopped：」、「✗ 自检：编译目录 caller-target-directory-kept：」、
#       「✗ 自检：编译目录 service-build-directory-kept：」、「✗ 自检：编译目录 copy-build-directories-removed：」、「✗ 自检：派行 row-peak-recorded-then-shared：」
#   REPLAY_BREAK=gpu-for-every-run  开了 GPU 时要现跑的行一律当要卡（不看 admission.py replay-row-inputs 第 4 列，有行要现跑就试编 gpu 特性）
#       reuse-gpu-only-for-feature-rows（「✗ 自检：按行复用 reuse-gpu-only-for-feature-rows：」）
#   REPLAY_BREAK=build-everything  有行要现跑时一律整编（不按要现跑的行只编那几个 bin）
#       reuse-device-source（「✗ 自检：按行复用 reuse-device-source：」）
#   REPLAY_BREAK=archived-restore-off  留存产物不在树里也不从 git 历史取回（replay_archived_product_copy 一律取不回）
#       archived-retrieved-green（「✗ 自检：留存产物取回 archived-retrieved-green：」）
#       连带红的格（E2 的留存产物只在 git 历史里，取不回就整趟退 1）：
#       「✗ 自检：按行复用 reuse-first-run：」、「✗ 自检：按行复用 reuse-second-run：」、「✗ 自检：按行复用 reuse-device-source：」、「✗ 自检：按行复用 reuse-comment-only：」、
#       「✗ 自检：按行复用 reuse-three-questions：」、「✗ 自检：按行复用 last-run-list：」、「✗ 自检：留存产物取回 device-not-run-red：」、「✗ 自检：入口与完整性闸 integrity-previous-output：」
#   REPLAY_BREAK=unrecoverable-as-green  取不回的留存产物当字节一致（改错的样子）
#       archived-unrecoverable-red（「✗ 自检：留存产物取回 archived-unrecoverable-red：」）
#   REPLAY_BREAK=inputs-not-checked  驱动开跑前不核输入、照起
#       驱动开跑前核输入那二十五格（「✗ 自检：driver_e142 在没有 research/、冻结根与导出根都没给的目录里」）
#       连带红的格：「✗ 自检：driver_e170_r2_v2_487_z3_images 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e170_r2_v2_487_z4_images 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s2_images_o3 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s2_o3_control_baseline 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s2_comparator 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s3_comparator_mutations 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s4_comparator 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s4_f4_comparator 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s4_f4_substitutes_comparator 在没有 research/、冻结根与导出根都没给…」、「✗ 自检：driver_e171_r1_s1_revision_6_comparator 在没有 research/、冻结根与导出根都没给的…」、「✗ 自检：driver_e171_r1_s1_revision_6_cross_host_check 在没有 research/、冻结根与导…」、「✗ 自检：driver_e171_r1_s2_revision_6_comparator 在没有 research/、冻结根与导出根都没给的…」、「✗ 自检：driver_e174_simulated_judge 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e174_segment_one_judge 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e174_segment_two_judge 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e158_r3_seg1_compare 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e158_r4_compare 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e165_segment2 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e165_r3_segment3_root_choice_kill_bounds 在没有 research/、冻结根…」、「✗ 自检：driver_e165_r4_segment2_root_choice_faults_kill_bounds 在没有 resear…」、「✗ 自检：driver_e165_r5_segment2_r2_mode_kill_bounds 在没有 research/、冻结根与导出根…」、「✗ 自检：driver_e165_r5_segment2_r3_mode_kill_bounds 在没有 research/、冻结根与导出根…」、「✗ 自检：driver_e165_r5_segment2_r4_mode_kill_bounds 在没有 research/、冻结根与导出根…」、「✗ 自检：driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds 在没有 research…」
#   REPLAY_BREAK=thread-registry-not-copied  拷 crates/ 去编译的副本不带 .claude/heavy-thread-default
#       thread-registry-copied（「✗ 自检：拷 crates/ 去编译的副本要带上每台默认重活线程数的登记文件…」）
#   REPLAY_BREAK=copy-lines-without-registry  自检里核「拷 crates 的行都接着拷登记文件」时当没有那一句（模拟新加的一行忘了带）
#       copy-lines-with-registry（「✗ 自检：拷 crates/ 去编译的行…应当每一行都接着 copy_thread_registry_into…」）
#   REPLAY_BREAK=copy-list-fixed  拷今天这棵树去编译的副本（e168_patched_tree、driver_e171_s4_comparator）照写死的四样加登记文件拷，不取共用库的闭包（没有 prune-server/）
#       copies-take-the-shared-closure（「✗ 自检：拷今天这棵树去编译的副本要照共用库的闭包拷全…」）
#   REPLAY_BREAK=copy-configuration-unset  副本里起命令不交多机配置（副本不带 multi-host.env，照副本的位置找不到配置）
#       copy-configuration-from-the-main-tree（「✗ 自检：副本里起命令的多机配置要取主工作区那一份…」）
#   REPLAY_BREAK=build-directory-kept  收尾不删这一趟自己建的编译目录与运行目录（编进固定的 scripts/experiment-apparatus/target/，跨趟留着）
#       build-directory-removed-when-green、build-directory-removed-when-build-fails、build-directory-removed-when-stopped、
#       copy-build-directories-removed（这一格也核 TMPDIR 底下不剩）（『✗ 自检：编译目录 <格名>：』）
#       连带红的格：「✗ 自检：编译目录 build-directory-removed-when-green：跑绿」、「✗ 自检：编译目录 build-directory-removed-when-build-fails：编不过」、「✗ 自检：编译目录 build-directory-removed-when-stopped：装置开跑之后给副本发 TERM」、「✗ 自检：编译目录 copy-build-directories-removed：装置在本轮输出目录里造的副本编译目录（带 CACHEDIR…」
#   REPLAY_BREAK=term-not-forwarded  被停时 replay_one 不把 TERM 转给手里那一步（装置照跑、开着编译目录里的文件，编译目录删不掉）
#       build-directory-removed-when-stopped（「✗ 自检：编译目录 build-directory-removed-when-stopped：」）
#   REPLAY_BREAK=copy-build-directories-kept  收尾不删驱动在本轮输出目录里的副本编出来的编译目录
#       copy-build-directories-removed（「✗ 自检：编译目录 copy-build-directories-removed：」）
#   REPLAY_BREAK=service-build-directory-ignored  不看环境里给的 CARGO_TARGET_DIR（调用方或服务给的），照自己在 TMPDIR 底下建、跑完删
#       caller-target-directory-kept、service-build-directory-kept（「✗ 自检：编译目录 caller-target-directory-kept：」、「✗ 自检：编译目录 service-build-directory-kept：」）
#   REPLAY_BREAK=e181-whole-process  E181 这一行照旧起一个不带段名的整进程（cargo run 跑全部 11 段、默认每台重活线程数），不走长跑脚本
#       e181-goes-through-long-run（「✗ 自检：派行 e181-goes-through-long-run：」）
#   REPLAY_BREAK=memory-bound-ignored  派行不比内存上限
#       dispatch-memory-bound（「✗ 自检：派行 dispatch-memory-bound：」）
#   REPLAY_BREAK=memory-by-task-cap  同时放得下几行照这一次拿到的上限比，不比这一台能给的内存
#       dispatch-task-cap-not-binding（「✗ 自检：派行 dispatch-task-cap-not-binding：」）
#   REPLAY_BREAK=host-cap-unreported  读数脚本 read 带 --no-host-cap、不报这一台能给的内存那一行（取得到的照取不到办，取不到的原因换成「没报」）
#       dispatch-task-cap-not-binding（「✗ 自检：派行 dispatch-task-cap-not-binding：」）、dispatch-memory-bound（「✗ 自检：派行 dispatch-memory-bound：」）
#   REPLAY_BREAK=host-cap-fallback-dropped  这一台能给的内存取不到时不退回这一次拿到的上限，派行不比内存
#       dispatch-memory-bound（「✗ 自检：派行 dispatch-memory-bound：」）
#   REPLAY_BREAK=threads-bound-kept  线程数（量过的取量到的，没量过的取每台默认重活线程数）超过 REPLAY_JOBS 的行独占（线程还在限制同时开几行的样子）
#       dispatch-threads-unbounded（「✗ 自检：派行 dispatch-threads-unbounded：」）
#       dispatch-exclusive-rows（「✗ 自检：派行 dispatch-exclusive-rows：」）
#   REPLAY_BREAK=row-count-unbound  派行不比 REPLAY_JOBS，放得下就起
#       dispatch-row-count-bound（「✗ 自检：派行 dispatch-row-count-bound：」）
#   REPLAY_BREAK=exclusive-row-shared  timing 行与 E181 不独占
#       dispatch-exclusive-rows（「✗ 自检：派行 dispatch-exclusive-rows：」）
#   REPLAY_BREAK=unmeasured-row-exclusive  没量过的行独占
#       dispatch-exclusive-rows（「✗ 自检：派行 dispatch-exclusive-rows：」）
#   REPLAY_BREAK=load-settle-builtin-name  自证读到的文本里等负载那一段的 awk 变量换成 gawk 的内建名字 load（gawk 起不来、照满上限等下去，mawk 不报）
#       load-settle-awk-names（「✗ 自检：静态核 load-settle-awk-names：」）
#   REPLAY_BREAK=mutation-driver-modules-unchecked  变异驱动的表与单测模块不核，一律当配得上
#       mutation-driver-tables（「✗ 自检：静态核 mutation-driver-tables：」）
#   REPLAY_BREAK=gpu-rows-not-serial  要卡的那一段几行一起起，不一行一行跑
#       dispatch-gpu-rows-serial（「✗ 自检：派行 dispatch-gpu-rows-serial：」）、dispatch-gpu-rows-without-cards（「✗ 自检：派行 dispatch-gpu-rows-without-cards：」）
#       （环境里没有卡串的那一格同样核要卡的两行一行一行跑）
#   REPLAY_BREAK=cards-shown-to-every-row  派行之前就把环境里的卡串交出去：不要卡的行也看得到卡
#       dispatch-gpu-rows-serial（「✗ 自检：派行 dispatch-gpu-rows-serial：」）、dispatch-driver-leased-rows（「✗ 自检：派行 dispatch-driver-leased-rows：」）
#   REPLAY_BREAK=driver-leased-rows-held-by-replay  不看 REPLAY_DRIVERS_LEASING_THEIR_OWN_CARDS：本脚本不交卡的行也进交卡的那一批、看得到卡串
#       dispatch-driver-leased-rows（「✗ 自检：派行 dispatch-driver-leased-rows：」）
#   REPLAY_BREAK=oversized-row-not-refused  一行比上限还大也不退 250，照起
#       dispatch-oversized-row-refused（「✗ 自检：派行 dispatch-oversized-row-refused：」）
#   REPLAY_BREAK=row-progress-silent  不报每一行的起止与在跑的进度
#       row-progress-lines（「✗ 自检：派行 row-progress-lines：」）
#   REPLAY_BREAK=row-peak-unrecorded  量到的峰值不进记录（下一趟这些行照没量过办）
#       row-peak-recorded-then-shared（「✗ 自检：派行 row-peak-recorded-then-shared：」）
#   REPLAY_BREAK=failed-row-peak-recorded  装置退非 0 的那一次也记峰值
#       row-failed-peak-not-recorded（「✗ 自检：派行 row-failed-peak-not-recorded：」）
#   REPLAY_BREAK=finished-rows-lost-when-stopped  被停时不打已判完的行的结果行
#       stopped-keeps-finished-rows（「✗ 自检：派行 stopped-keeps-finished-rows：」）
#   REPLAY_BREAK=guard-off  平常跑设了弄坏开关也不拒（认不得的选项先被「复跑表里没有」当用法错拒，不点名变量）
#       break-switch-refused-outside-selftest（「✗ 自检：平常跑设了弄坏开关应当退 2 并点名」）
# 弄坏开关 REPLAY_BREAK 只在 --selftest 起的那几趟里认（自证设 REPLAY_SELFTEST_MARK 指到它建的标记文件，子进程继承）：平常跑设了就在一切之前退 2、点名它只在自证里认。
# 按行复用的自证（文件头「按行复用」，replay_selftest 调）：<目录> 下建一个临时仓（git init），复跑表换成两行（E1 留存产物在、E2 的留存产物只在 git 历史里），
# 真仓的 .claude/scripts、.claude/singlefs-ai-sop、scripts/gate 链进来，PATH 前面放假 cargo、假 rustc；
# 拿这一份 replay.sh 的副本接连跑五趟，每一格往 stdout 打一行「<格名> ok」或「<格名> bad <一句>」。
# 按行复用与留存产物取回两份自证共用的临时仓（replay_reuse_selftest、replay_archived_selftest 调）：<目录>/root 下 git init，复跑表换成两行（E1 留存产物在、E2 的留存产物先入库再删掉、只在 git 历史里），
# 真仓的 .claude/scripts、.claude/singlefs-ai-sop、scripts/gate 链进来，PATH 前面放假 cargo、假 rustc（<目录>/bin）；
# 两个假 research 二进制往 $REPLAY_SELFTEST_RUN_LOG 记一笔，REPLAY_SELFTEST_NOT_RUN_DEVICE 点名它时打一行 NOT_RUN_SAMPLE_INPUT 退 77；
# REPLAY_SELFTEST_BROKEN_DEVICE 点名它时往 stderr 打一句、退 3（跑不了）。
# 设了 REPLAY_SELFTEST_DEVICE_HOLD 的，先建旁边那个 .fifo、再把自己的进程号写进它点名的文件，之后在 fifo 上 read -t 8 拿住 8 秒（开着编译目录里自己那份脚本）：
# 写下进程号之后不再起子进程：TERM 只停装置自己，它 fork 出来、还没 exec 的子进程照样开着同一份脚本，副本收尾核开着的文件时就不删编译目录（所以不一趟趟起 sleep 0.1）。
replay_selftest_sample_repository() { # <目录>
  local work="$1" real_root="$PWD" root name
  root="$work/root"
  mkdir -p "$root/.claude/gate.d" "$root/scripts/experiment-apparatus" "$work/devices" "$work/tmp" "$root/scripts/hosts" "$root/research/results" "$work/bin" || return 1
  git init -q "$root" || return 1
  ln -s "$real_root/.claude/scripts" "$root/.claude/scripts"
  ln -s "$real_root/.claude/singlefs-ai-sop" "$root/.claude/singlefs-ai-sop"
  ln -s "$real_root/scripts/gate" "$root/scripts/gate"
  ln -s "$real_root/scripts/crash" "$root/scripts/crash"   # resolve_checker_tier_features 读多机配置的共用库
  ln -s "$real_root/scripts/hosts/two-logs.sh" "$root/scripts/hosts/two-logs.sh"   # 构建日志出两份（build_research_binaries）
  ln -s "$real_root/scripts/hosts/two_logs.py" "$root/scripts/hosts/two_logs.py"
  ln -s "$real_root/scripts/hosts/run-build-directory.sh" "$root/scripts/hosts/run-build-directory.sh"   # 这一趟的编译目录（文件头「编译目录」）
  ln -s "$real_root/scripts/hosts/run_build_directory.py" "$root/scripts/hosts/run_build_directory.py"
  ln -s "$real_root/scripts/experiment-apparatus/replay_row_costs.py" "$root/scripts/experiment-apparatus/replay_row_costs.py"   # 派行的读数（文件头「派行」），它按自己的真实路径找真仓的 scripts/hosts/task_memory.py（上限与这一台能给的内存的取法），样本仓里不另放
  printf '%s\n' '.claude/scripts' '.claude/singlefs-ai-sop' 'scripts/gate' 'scripts/crash' 'scripts/hosts/two-logs.sh' 'scripts/hosts/two_logs.py' \
    'scripts/hosts/run-build-directory.sh' 'scripts/hosts/run_build_directory.py' 'scripts/experiment-apparatus/replay_row_costs.py' >"$root/.gitignore"
  printf '%s\n' '# 自证的登记表：一行都不登记准入' >"$root/.claude/gate.d/stage-inputs.tsv"
  printf '%s\n' '# 自证的滞后表' >"$root/.claude/gate.d/timing-load-field-lag.tsv"
  printf '%s\n' '[package]' 'name = "e7-index-bench"' 'version = "0.1.0"' '' '[lib]' 'path = "lib.rs"' '' '[[bin]]' 'name = "e1-sample"' 'path = "e1_sample.rs"' \
    '' '[[bin]]' 'name = "e2-sample"' 'path = "e2_sample.rs"' >"$root/scripts/experiment-apparatus/Cargo.toml"
  printf '%s\n' '# 自证' >"$root/scripts/experiment-apparatus/Cargo.lock"
  printf '%s\n' 'pub fn shared() -> u32 { 1 }' >"$root/scripts/experiment-apparatus/lib.rs"
  printf '%s\n' 'fn main() { println!("one"); }' >"$root/scripts/experiment-apparatus/e1_sample.rs"
  printf '%s\n' 'fn main() { println!("two"); }' >"$root/scripts/experiment-apparatus/e2_sample.rs"
  for name in e1 e2; do
    printf '%s\n' '#!/usr/bin/env bash' "echo $name >>\"\${REPLAY_SELFTEST_RUN_LOG:?}\"" \
      "if [[ \"\${REPLAY_SELFTEST_NOT_RUN_DEVICE:-}\" == $name ]]; then echo 'NOT_RUN_SAMPLE_INPUT test=$name-sample reason=自证造的输入不在' >&2; exit 77; fi" \
      "if [[ \"\${REPLAY_SELFTEST_BROKEN_DEVICE:-}\" == $name ]]; then echo '自证造的跑不了' >&2; exit 3; fi" \
      '[[ -z "${REPLAY_SELFTEST_DEVICE_PATHS:-}" ]] || echo "$0" >>"$REPLAY_SELFTEST_DEVICE_PATHS"' \
      'if [[ -n "${REPLAY_SELFTEST_DEVICE_HOLD:-}" ]]; then [[ -p "$REPLAY_SELFTEST_DEVICE_HOLD.fifo" ]] || mkfifo -- "$REPLAY_SELFTEST_DEVICE_HOLD.fifo" || exit 1; echo "$$" >"$REPLAY_SELFTEST_DEVICE_HOLD"; read -r -t 8 _ <>"$REPLAY_SELFTEST_DEVICE_HOLD.fifo"; fi' \
      'if [[ -n "${REPLAY_SELFTEST_COPY_TARGET:-}" ]]; then copy="$(dirname "$REPLAY_DEV")/selftest-copy"; mkdir -p "$copy/target/release" "$copy/source" && printf "Signature: 8a477f597d28d172789f06886806bc55\n" >"$copy/target/CACHEDIR.TAG"; fi' \
      "if [[ \"\${REPLAY_SELFTEST_DRIFT_DEVICE:-}\" == $name ]]; then printf 'E7RESULT name=config arm=$name-drifted\\nE7RESULT name=done emitted=2\\n'; exit 0; fi" \
      "printf 'E7RESULT name=config arm=$name\\nE7RESULT name=done emitted=2\\n'" \
      >"$work/devices/$name-sample"
    chmod +x "$work/devices/$name-sample"
  done
  printf 'E7RESULT name=config arm=e1\nE7RESULT name=done emitted=2\n' >"$root/research/results/e1-sample.out"
  # E2 的留存产物只在 git 历史里：先入库、再删掉（按「每次提交删上一次的实验记录」归档的样子），复跑从 git 历史取回再比
  printf 'E7RESULT name=config arm=e2\nE7RESULT name=done emitted=2\n' >"$root/research/results/e2-sample.out"
  git -C "$root" add -f research/results/e2-sample.out \
    && git -C "$root" -c user.name=selftest -c user.email=selftest@example.invalid -c commit.gpgsign=false commit -q -m 'e2 sample product' \
    && git -C "$root" rm -q research/results/e2-sample.out \
    && git -C "$root" -c user.name=selftest -c user.email=selftest@example.invalid -c commit.gpgsign=false commit -q -m 'archive e2 sample product' || return 1
  # 假 cargo：记下收到的参数（与 CARGO_TARGET_DIR，另一份 .target）；编（build）的时候把 $work/devices/ 下的假装置拷进 CARGO_TARGET_DIR 的 release/（装置从那里起）；
  # REPLAY_SELFTEST_BUILD_FAILS=1 时编不过、退 101
  printf '%s\n' '#!/usr/bin/env bash' '[[ "${1:-}" == -V ]] && { echo "cargo 0.0.0-replay-reuse-selftest"; exit 0; }' 'echo "$*" >>"${REPLAY_SELFTEST_CARGO_LOG:?}"' \
    'echo "${CARGO_TARGET_DIR-}" >>"$REPLAY_SELFTEST_CARGO_LOG.target"' \
    '[[ "${REPLAY_SELFTEST_BUILD_FAILS:-}" == 1 ]] && { echo "error: 自证造的编译失败"; exit 101; }' \
    "if [[ \"\${1:-}\" == build ]]; then mkdir -p \"\${CARGO_TARGET_DIR:?}/release\" && cp -- \"$work/devices/\"* \"\$CARGO_TARGET_DIR/release/\" || exit 101; fi" 'exit 0' >"$work/bin/cargo"
  printf '%s\n' '#!/usr/bin/env bash' 'case "${1:-}" in -V) echo "rustc 0.0.0-replay-reuse-selftest";; -vV) printf "rustc 0.0.0-replay-reuse-selftest\nhost: x86_64-unknown-linux-gnu\n";; esac' 'exit 0' >"$work/bin/rustc"
  chmod +x "$work/bin/cargo" "$work/bin/rustc"
  python3 - "$real_root/scripts/experiment-apparatus/replay.sh" "$root/scripts/experiment-apparatus/replay.sh" <<'PY' || return 1
import sys
text = open(sys.argv[1], encoding="utf-8").read()
table_start = text.index("TABLE=$(cat <<'TSV'\n") + len("TABLE=$(cat <<'TSV'\n")
table_end = text.index("\nTSV\n)\n", table_start)
text = text[:table_start] + "E1|e1-sample||e1-sample.out|exact\nE2|e2-sample||e2-sample.out|exact" + text[table_end:]
keys_start = text.index("ADMISSION_ROW_KEYS=$(cat <<'TSV'\n") + len("ADMISSION_ROW_KEYS=$(cat <<'TSV'\n")
keys_end = text.index("TSV\n)\n", keys_start)
text = text[:keys_start] + text[keys_end:]
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    handle.write(text)
PY
  chmod +x "$root/scripts/experiment-apparatus/replay.sh"
  cp -- "${REPLAY_SOURCE_FILES[@]:1}" "$root/scripts/experiment-apparatus/" || return 1   # 入口按自己所在目录 source 的几份库，样本仓的副本旁边带着
}
# 自证里临时仓的按行复用库名：与入口「库名」那一段同一种取法（<git common-dir>/singlefs-unit-verdicts/replay-row，文件头「按行复用」），
# 只当名字交给 admission.py library-files，本地不建；取不到 git common-dir 退 1、什么都不打
replay_selftest_row_library_name() { # <临时仓>
  local common_directory
  common_directory="$(git -C "$1" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]] || return 1
  printf '%s/singlefs-unit-verdicts/replay-row\n' "$common_directory"
}
replay_reuse_selftest() {
  local work="$1" real_root="$PWD" root library calls cargo_calls out total run_number=0 reuse_exit=0 name
  root="$work/root"; calls="$work/devices-ran.log"; cargo_calls="$work/cargo-calls.log"
  replay_selftest_sample_repository "$work" || return 1
  library="$(replay_selftest_row_library_name "$root")" || { echo "reuse-first-run bad 临时仓 $root 取不到 git common-dir，库名拼不出"; return 1; }
  reuse_run() { # <参数…>：在临时仓里跑一趟副本；输出进 $out、这一趟起了哪几个装置进 $calls（每趟先清空）、末行 REPLAY_TOTAL 进 $total、退出码进 reuse_exit
    # 参数只认 case 里那几组，各写成一条带字面实验号或行项的起法：闭包按起法里的字面词收复跑表的行，参数折不出（"$@" 这类）或不带实验号就连整张复跑表
    # （scripts/gate/lib_function_units.py 文件头「shell 起法带字面参数」）。样本仓的表只有 E1、E2 两行，要两行都跑的写 E1 E2
    run_number=$((run_number + 1)); out="$work/out.$run_number"; : >"$calls"; : >"$cargo_calls"; : >"$work/not-run.$run_number"
    # 复用哨兵：键相同的全当哨兵、不抽强制现判（reuse_sentinel 设了的那几趟照它）；上次红的清单写临时目录
    if (cd "$root" && unset SINGLEFS_MULTI_HOST_CONFIG SINGLEFS_REUSE_SENTINEL_SEED CARGO_TARGET_DIR \
          && export TMPDIR="$work/tmp" PATH="$work/bin:$PATH" REPLAY_OUT="$work/replay-out.$run_number" REPLAY_JOBS=1 \
               REPLAY_SELFTEST_RUN_LOG="$calls" REPLAY_SELFTEST_CARGO_LOG="$cargo_calls" GATE_NOT_RUN_FILE="$work/not-run.$run_number" \
               SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" SINGLEFS_REUSE_SENTINEL_COUNT=1000 \
          && { [[ -z "${reuse_sentinel[*]-}" ]] || export "${reuse_sentinel[@]}"; } \
          && case "$*" in
               "E1 E2") bash scripts/experiment-apparatus/replay.sh E1 E2 ;;
               E1) bash scripts/experiment-apparatus/replay.sh E1 ;;
               E3) bash scripts/experiment-apparatus/replay.sh E3 ;;
               *) echo "reuse_run：参数「$*」没有字面起法（只认 E1 E2、E1、E3）" >&2; exit 2 ;;
             esac) >"$out" 2>&1; then
      reuse_exit=0
    else
      reuse_exit=$?
    fi
    total="$(grep '^REPLAY_TOTAL ' "$out")"
    for name in E1 E2; do   # 复用三问那一格读的：这一趟每一行复用还是现判
      if grep -qE "^$name .*复用 " "$out"; then printf '%s\t复用\n' "$name"; else printf '%s\t现判\n' "$name"; fi
    done >"$work/states.$run_number"
  }
  local reuse_sentinel=()
  ran() { sort "$calls" | tr '\n' ' '; }
  reuse_run E1 E2
  if [[ "$reuse_exit" == 0 && "$(ran)" == "e1 e2 " && "$total" == *" reused=0 "* ]] && grep -qE '^E1 .*字节一致' "$out" && grep -qE '^E2 .*字节一致.*取自 git show' "$out" \
     && [[ "$(replay_selftest_library_counts "$library" "$work/fetched")" == "2 2" && ! -e "${library%/*}" ]] \
     && [[ ! -s "$work/not-run.$run_number" ]]; then
    echo "reuse-first-run ok"
  else
    echo "reuse-first-run bad 头一趟应当两行都跑、E1 字节一致、E2 字节一致（留存产物从 git 历史取回）、判定库记两份（各带组成清单）、临时仓里不建 singlefs-unit-verdicts/、没有逐字节比对本次未跑的报告；退 $reuse_exit，起了 $(ran)，$total，判定库「判定 组成清单」$(replay_selftest_library_counts "$library" "$work/fetched") 份，临时仓里 singlefs-unit-verdicts/ $([[ -e "${library%/*}" ]] && echo 建了 || echo 没建)，本次未跑「$(tr '\n' ' ' <"$work/not-run.$run_number")」，输出尾部 $(tail -n 4 "$out" | tr '\n' ' ')"
  fi
  reuse_run E1 E2
  if [[ "$reuse_exit" == 0 && -z "$(ran)" && "$total" == *" judged=0 reused=2 "* ]] && grep -qE '^E1 .*复用 +字节一致：行键相同' "$out" && grep -qE '^E2 .*复用 +字节一致：行键相同' "$out" \
     && ! grep -q '^build' "$cargo_calls" && grep -qF '要现跑的 0 行，不试编 gpu 特性、不编' "$out"; then
    echo "reuse-second-run ok"
  else
    echo "reuse-second-run bad 第二趟行键没变应当两行都复用、一个装置都不起、cargo build 一次都不起（不编）；退 $reuse_exit，起了「$(ran)」，cargo 收到「$(tr '\n' ';' <"$cargo_calls")」，$total"
  fi
  printf '%s\n' 'fn main() { println!("uno"); }' >"$root/scripts/experiment-apparatus/e1_sample.rs"
  reuse_run E1 E2
  if [[ "$reuse_exit" == 0 && "$(ran)" == "e1 " && "$total" == *" reused=1 "* ]] && grep -qE '^E1 .*字节一致' "$out" && grep -qE '^E2 .*复用' "$out" \
     && grep -q -- '^build .*--bin e1-sample' "$cargo_calls" && ! grep -q -- '--bin e2-sample' "$cargo_calls" && grep -q 'E1 要现跑：键对不上.*变了 fn e1_sample.rs::main' "$out"; then
    echo "reuse-device-source ok"
  else
    echo "reuse-device-source bad 改了 E1 装置 main 的一行代码应当只重跑 E1（只编 --bin e1-sample、报出变了 fn e1_sample.rs::main）、E2 复用；退 $reuse_exit，起了「$(ran)」，cargo 收到「$(tr '\n' ';' <"$cargo_calls")」，$total，输出 $(grep '要现跑' "$out" | head -n 2 | tr '\n' ' ')"
  fi
  printf '%s\n' '// 只改一行注释：行键取单元的去注释指纹，注释不算' 'fn main() { println!("two"); }' >"$root/scripts/experiment-apparatus/e2_sample.rs"
  reuse_run E1 E2
  if [[ "$reuse_exit" == 0 && -z "$(ran)" && "$total" == *" reused=2 "* ]]; then
    echo "reuse-comment-only ok"
  else
    echo "reuse-comment-only bad 只改 E2 装置源码的一行注释应当两行都复用；退 $reuse_exit，起了「$(ran)」，$total"
  fi
  # 复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：上面四趟就是那四问——头一趟、第二趟（①）、只改注释（②，排在闭包内那一趟之后，库里是那一趟记下的）、
  # 改 E1 装置 main 的一行（③，affected 是 E1）；已有的 reuse-second-run、reuse-comment-only、reuse-device-source 三格在这一格里点名，判法照共用库
  if python3 "$real_root/scripts/gate/lib_reuse_checks.py" three-questions --script scripts/experiment-apparatus/replay.sh --first "$work/states.1" --second "$work/states.2" \
       --unrelated "$work/states.4" --unit-change "$work/states.3" --affected E1 >"$work/three.log" 2>&1; then
    echo "reuse-three-questions ok"
  else
    echo "reuse-three-questions bad $(tr '\n' ' ' <"$work/three.log")"
  fi
  grep '^REUSE_SELFTEST ' "$work/three.log"
  # 上次红的清单（lib_reuse_checks.py 文件头「三」）：这几趟都绿，清单是空文件
  if [[ -f "$work/last-runs/scripts__experiment-apparatus__replay.sh.tsv" && ! -s "$work/last-runs/scripts__experiment-apparatus__replay.sh.tsv" ]]; then
    echo "last-run-list ok"
  else
    echo "last-run-list bad 全绿的几趟之后 $work/last-runs/scripts__experiment-apparatus__replay.sh.tsv 应当在、是空文件：$(ls -l "$work/last-runs" 2>&1 | tr '\n' ' ')"
  fi
  # 复用哨兵（lib_reuse_checks.py 文件头「二」）：种子 5、哨兵 1 条：一行当哨兵复用、另一行强制现判、判定与库里相同，整趟绿；
  # 开着 REPLAY_BREAK=reuse-ignored（只记不读）再跑，哨兵判红「复用失灵」（共用库的开关 REUSE_CHECKS_BREAK=rejudged-sentinel-passes 下这一格判错）。
  # 这两趟的 REPLAY_BREAK 自己定、不接调用方设的开关：照常那一趟清空，只记不读那一趟只设 reuse-ignored（REPLAY_BROKEN 按整串比，接成「<别的>,reuse-ignored」就认不出），
  # 别的开关打开时这一格不跟着红（弄坏开关只红在文件头点名的格）。前面几趟是在调用方的开关下记的库（reuse-key-by-row-only 下键的算法都不同），
  # 所以先清空开关垫一趟（哨兵数 1000、不抽强制现判），库里是照常算的键，再起那两趟
  reuse_sentinel=(REPLAY_BREAK=)
  reuse_run E1 E2
  local sentinel_primed_exit="$reuse_exit"
  reuse_sentinel=(SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5 REPLAY_BREAK=)
  reuse_run E1 E2
  local sentinel_green_exit="$reuse_exit" sentinel_green_out="$out"
  reuse_sentinel=(SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5 REPLAY_BREAK=reuse-ignored)
  reuse_run E1 E2
  reuse_sentinel=()
  if [[ "$sentinel_primed_exit" == 0 && "$sentinel_green_exit" == 0 ]] && grep -qF '复用哨兵（复跑表）：种子 5' "$sentinel_green_out" && grep -qF '复用哨兵抽中强制现判' "$sentinel_green_out" \
     && [[ "$reuse_exit" == 1 ]] && grep -qF '复用失灵：' "$out"; then
    echo "reuse-sentinel ok"
  else
    echo "reuse-sentinel bad 垫的那一趟与种子 5 那一趟应当绿、抽中一行强制现判，只记不读那一趟应当判红「复用失灵」；垫的那一趟退 $sentinel_primed_exit；照常那一趟退 $sentinel_green_exit：$(grep -E '复用哨兵|复用失灵' "$sentinel_green_out" | tr '\n' ' ')；只记不读那一趟退 $reuse_exit：$(grep -E '复用哨兵|复用失灵' "$out" | tr '\n' ' ')"
  fi
  # 记红、撤绿、下趟现判（文件头「按行复用」，reuse-red-withdraws-green）：清空开关垫一趟（E1 字节一致入库）；再跑一趟走复用哨兵强制现判（哨兵 1 条、种子 5：E2 当哨兵复用、E1 被抽成强制现判），E1 的假装置演出另一份产物
  # （REPLAY_SELFTEST_DRIFT_DEVICE=e1：装置在仓外，不进这一行的行键）⇒ E1 对不上、整趟退 1、库里这把行键记成 drift；下一趟：E1 现判（开跑前写「上次判红」）、判回字节一致，
  # E2 复用；再下一趟两行全复用（盖回绿）。弄坏开关 red-not-recorded（对不上的不记）让下一趟 E1 照旧复用；这一格只接这一个开关，别的开关打开时不跟着红
  local red_break=""
  [[ "$REPLAY_BROKEN" == red-not-recorded ]] && red_break=red-not-recorded
  reuse_sentinel=(REPLAY_BREAK=)
  reuse_run E1 E2
  local red_primed_exit="$reuse_exit"
  reuse_sentinel=(REPLAY_BREAK="$red_break" REPLAY_SELFTEST_DRIFT_DEVICE=e1 SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5)
  reuse_run E1 E2
  local red_drift_exit="$reuse_exit" red_drift_out="$out"
  reuse_sentinel=(REPLAY_BREAK=)
  reuse_run E1 E2
  local red_rejudge_exit="$reuse_exit" red_rejudge_out="$out" red_rejudge_ran
  red_rejudge_ran="$(ran)"
  reuse_run E1 E2
  if [[ "$red_primed_exit" == 0 && "$red_drift_exit" == 1 ]] && grep -qE '^E1 .*对不上' "$red_drift_out" \
     && [[ "$red_rejudge_exit" == 0 && "$red_rejudge_ran" == "e1 " ]] && grep -qF 'E1 要现跑：上次判红' "$red_rejudge_out" \
     && grep -qE '^E1 .*字节一致' "$red_rejudge_out" && grep -qE '^E2 .*复用' "$red_rejudge_out" \
     && [[ "$reuse_exit" == 0 && -z "$(ran)" && "$total" == *" judged=0 reused=2 "* ]]; then
    echo "reuse-red-withdraws-green ok"
  else
    echo "reuse-red-withdraws-green bad 同一把行键 E1 先字节一致入库、哨兵强制现判判成对不上，下一趟应当现判 E1（只起 e1、开跑前写上次判红、判回字节一致）、E2 复用，再下一趟两行全复用；垫的那一趟退 $red_primed_exit；强制现判那一趟退 $red_drift_exit；下一趟退 $red_rejudge_exit、起了「$red_rejudge_ran」、$(grep -F 'E1 要现跑' "$red_rejudge_out" | head -n 1 | clip_text 200)；再下一趟退 $reuse_exit、起了「$(ran)」、$total"
  fi
  # 跑不了也记红、撤绿（文件头「按行复用」，reuse-broken-withdraws-green）：清空开关垫一趟（E1 字节一致入库）；再跑一趟走复用哨兵强制现判（哨兵 1 条、种子 5：E1 被抽成强制现判），E1 的假装置退 3
  # （REPLAY_SELFTEST_BROKEN_DEVICE=e1：装置在仓外，不进这一行的行键）⇒ E1 跑不了、整趟退 1、库里这把行键记成 broken；下一趟：E1 现判（开跑前写「上次判红」）、判回字节一致，
  # E2 复用；再下一趟两行全复用。弄坏开关 broken-not-recorded（跑不了的不记）让下一趟 E1 照旧复用；这一格只接这一个开关，别的开关打开时不跟着红
  local broken_break=""
  [[ "$REPLAY_BROKEN" == broken-not-recorded ]] && broken_break=broken-not-recorded
  reuse_sentinel=(REPLAY_BREAK=)
  reuse_run E1 E2
  local broken_primed_exit="$reuse_exit"
  reuse_sentinel=(REPLAY_BREAK="$broken_break" REPLAY_SELFTEST_BROKEN_DEVICE=e1 SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5)
  reuse_run E1 E2
  local broken_run_exit="$reuse_exit" broken_run_out="$out"
  reuse_sentinel=(REPLAY_BREAK=)
  reuse_run E1 E2
  local broken_rejudge_exit="$reuse_exit" broken_rejudge_out="$out" broken_rejudge_ran
  broken_rejudge_ran="$(ran)"
  reuse_run E1 E2
  if [[ "$broken_primed_exit" == 0 && "$broken_run_exit" == 1 ]] && grep -qE '^E1 .*跑不了 +退出码 3' "$broken_run_out" \
     && [[ "$broken_rejudge_exit" == 0 && "$broken_rejudge_ran" == "e1 " ]] && grep -qF 'E1 要现跑：上次判红' "$broken_rejudge_out" \
     && grep -qE '^E1 .*字节一致' "$broken_rejudge_out" && grep -qE '^E2 .*复用' "$broken_rejudge_out" \
     && [[ "$reuse_exit" == 0 && -z "$(ran)" && "$total" == *" judged=0 reused=2 "* ]]; then
    echo "reuse-broken-withdraws-green ok"
  else
    echo "reuse-broken-withdraws-green bad 同一把行键 E1 先字节一致入库、哨兵强制现判跑成跑不了（退 3），下一趟应当现判 E1（只起 e1、开跑前写上次判红、判回字节一致）、E2 复用，再下一趟两行全复用；垫的那一趟退 $broken_primed_exit；强制现判那一趟退 $broken_run_exit；下一趟退 $broken_rejudge_exit、起了「$broken_rejudge_ran」、$(grep -F 'E1 要现跑' "$broken_rejudge_out" | head -n 1 | clip_text 200)；再下一趟退 $reuse_exit、起了「$(ran)」、$total"
  fi
  # 不在 git 工作树里（文件头「按行复用」，reuse-outside-git-exits-2）：另搭一棵样本仓、删掉它的 .git（样本仓在临时目录里，往上找不到别的仓），跑一趟：
  # 整趟应当退 2、点名不是 git 工作树、一个装置都不起、不打 REPLAY_TOTAL。弄坏开关 row-library-failure-tolerated（不复用、不记、全跑）让它往下跑；这一格只接这一个开关
  local outside_break="" first_root="$root"
  [[ "$REPLAY_BROKEN" == row-library-failure-tolerated ]] && outside_break=row-library-failure-tolerated
  replay_selftest_sample_repository "$work/outside-git-cell" || return 1
  root="$work/outside-git-cell/root"
  replay_remove_directory_below "$work" "$root/.git" || return 1
  reuse_sentinel=(REPLAY_BREAK="$outside_break")
  reuse_run E1 E2
  reuse_sentinel=()
  root="$first_root"
  if [[ "$reuse_exit" == 2 && -z "$(ran)" && -z "$total" ]] && grep -qF '不是 git 工作树' "$out"; then
    echo "reuse-outside-git-exits-2 ok"
  else
    echo "reuse-outside-git-exits-2 bad 不在 git 工作树里应当整趟退 2、点名不是 git 工作树、一个装置都不起、不打 REPLAY_TOTAL；退 $reuse_exit，起了「$(ran)」，$total，$(grep -E '✗|!' "$out" | head -n 2 | tr '\n' ' ' | clip_text 300)"  # gate-lint:detail
  fi
  # 服务连不上（文件头「按行复用」，reuse-unreachable-exits-75）：socket 不在、TCP 端口刚放掉、重连总时长 2 秒；库里有判过的记录（前面几趟记的），
  # 整趟应当退 75、点名 prune-server 不可用、一个装置都不起、不打 REPLAY_TOTAL。弄坏开关 prune-server-failure-swallowed（照库里没有全跑）让它退 0；这一格只接这一个开关
  local unreachable_address unreachable_break=""
  [[ "$REPLAY_BROKEN" == prune-server-failure-swallowed ]] && unreachable_break=prune-server-failure-swallowed
  unreachable_address="$(python3 -B -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print("127.0.0.1:%d" % s.getsockname()[1]); s.close()')"
  reuse_sentinel=(REPLAY_BREAK="$unreachable_break" SINGLEFS_PRUNE_SERVER_SOCKET="$work/no-such.sock" SINGLEFS_PRUNE_SERVER_TCP_ADDRESS="$unreachable_address"
    SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=2)
  reuse_run E1 E2
  reuse_sentinel=()
  if [[ "$reuse_exit" == 75 && -z "$(ran)" && -z "$total" ]] && grep -qF 'prune-server 不可用' "$out"; then
    echo "reuse-unreachable-exits-75 ok"
  else
    echo "reuse-unreachable-exits-75 bad 判定库服务连不上应当整趟退 75、点名 prune-server 不可用、一个装置都不起、不打 REPLAY_TOTAL；退 $reuse_exit，起了「$(ran)」，$total，$(grep -F 'prune-server' "$out" | head -n 2 | tr '\n' ' ' | clip_text 300)"
  fi
  # 开了 GPU 只在要现跑的行里有要卡的行时才试编（文件头「GPU」，reuse-gpu-only-for-feature-rows）：另搭一棵样本仓，复跑表加一行
  # E3|@replay_selftest_gpu_driver（驱动闭包点到 CHECKER_TIER_FEATURES，admission.py replay-row-inputs 第 4 列给 1），多机配置写 ENABLE_GPU=1、环境里没有卡串：
  # 只跑 E1 E2（都不点）整趟退 0、两行都起、cargo 一次 --features gpu 都没收到、stderr 写「都不点 CHECKER_TIER_FEATURES」；
  # 只跑 E3 整趟退 0（不领卡、环境里没有卡串也不拦；样本仓里放了 E3 的留存产物 e3-sample.out，与驱动打的字节相同，判字节一致）、cargo 收到试编 gpu 特性那一条、
  # E3 的驱动带着 --features gpu 起了（驱动的 stderr 由 replay_one 落进这一趟输出目录里这一行的 <tag>.err，里面有它打的 features=--features gpu；不进这一趟的输出）。
  # 弄坏开关 gpu-for-every-run（要现跑的行一律当要卡）让只跑 E1 E2 那一趟试编 gpu 特性；这一格只接这一个开关
  local gpu_break="" gpu_plain_exit gpu_plain_out gpu_plain_ran gpu_plain_cargo gpu_feature_cargo gpu_feature_driver_err
  [[ "$REPLAY_BROKEN" == gpu-for-every-run ]] && gpu_break=gpu-for-every-run
  replay_selftest_sample_repository "$work/gpu-cell" || return 1
  root="$work/gpu-cell/root"
  python3 - "$root/scripts/experiment-apparatus/replay.sh" <<'PY' || return 1
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
table = "TABLE=$(cat <<'TSV'\n"
driver = ("replay_selftest_gpu_driver() {\n"
          "  echo \"features=${CHECKER_TIER_FEATURES[*]-}\" >&2\n"
          "  printf 'E7RESULT name=config arm=e3\\nE7RESULT name=done emitted=2\\n'\n"
          "}\n")
start = text.index(table)
row = "E2|e2-sample||e2-sample.out|exact"
row_end = text.index(row, start) + len(row)
text = text[:start] + driver + text[start:row_end] + "\nE3|@replay_selftest_gpu_driver||e3-sample.out|exact" + text[row_end:]
open(path, "w", encoding="utf-8").write(text)
PY
  printf '%s\n' '# 自证的多机配置' 'ENABLE_GPU=1' >"$work/gpu-cell/multi-host.env"
  printf 'E7RESULT name=config arm=e3\nE7RESULT name=done emitted=2\n' >"$root/research/results/e3-sample.out"
  reuse_sentinel=(REPLAY_BREAK="$gpu_break" SINGLEFS_MULTI_HOST_CONFIG="$work/gpu-cell/multi-host.env")
  reuse_run E1 E2
  gpu_plain_exit="$reuse_exit"; gpu_plain_out="$out"; gpu_plain_ran="$(ran)"; gpu_plain_cargo="$(tr '\n' ';' <"$cargo_calls")"
  reuse_run E3
  gpu_feature_cargo="$(tr '\n' ';' <"$cargo_calls")"
  gpu_feature_driver_err="$(cat -- "$work/replay-out.$run_number"/E3.*.err 2>/dev/null)"
  reuse_sentinel=()
  root="$first_root"
  if [[ "$gpu_plain_exit" == 0 && "$gpu_plain_ran" == "e1 e2 " && "$gpu_plain_cargo" != *"--features gpu"* ]] && grep -qF '都不点 CHECKER_TIER_FEATURES' "$gpu_plain_out" \
     && [[ "$reuse_exit" == 0 && "$gpu_feature_cargo" == *"--features gpu"* && "$gpu_feature_driver_err" == *"features=--features gpu"* ]]; then
    echo "reuse-gpu-only-for-feature-rows ok"
  else
    echo "reuse-gpu-only-for-feature-rows bad 开了 GPU、环境里没有卡串：只跑不点 CHECKER_TIER_FEATURES 的 E1 E2 应当退 0、两行都起、不试编 gpu 特性，只跑点到的 E3 应当退 0（E3 字节一致）、试编 gpu 特性、驱动带着 --features gpu 起（它那一行的 .err 里有 features=--features gpu）；E1 E2 那一趟退 $gpu_plain_exit、起了「$gpu_plain_ran」、cargo 收到「$gpu_plain_cargo」、$(grep -E '✗|都不点' "$gpu_plain_out" | head -n 2 | tr '\n' ' ' | clip_text 300)；E3 那一趟退 $reuse_exit、cargo 收到「$gpu_feature_cargo」、结果行「$(grep -E '^E3 ' "$out" | head -n 1 | clip_text 200)」、驱动的 stderr「$(tr '\n' ' ' <<<"$gpu_feature_driver_err" | clip_text 200)」、$(grep -E '✗' "$out" | head -n 2 | tr '\n' ' ' | clip_text 300)"
  fi
}

# 自证读判定库（replay_reuse_selftest、replay_archived_selftest 调）：<库名> 底下的记录经 admission.py library-files 拉进 <落到哪>（先清空），
# 打「<判定份数> <组成清单份数>」；拉不成打「拉不成」。在真仓的工作目录里起（自证就在那里）
replay_selftest_library_counts() { # <库名> <落到哪>
  rm -rf -- "${2:?}"
  if python3 scripts/gate/admission.py library-files fetch "$1" "$2" keys/ >/dev/null 2>&1; then
    printf '%s %s\n' "$(find "$2/keys" -mindepth 2 -maxdepth 2 -name verdict 2>/dev/null | wc -l)" "$(find "$2/keys" -mindepth 2 -maxdepth 2 -name composition 2>/dev/null | wc -l)"
  else
    echo 拉不成
  fi
}

# 留存产物取回与装置读不到输入的自证（文件头「驱动要读的输入」，replay_selftest 调）：同一份临时仓（replay_selftest_sample_repository：E1 的留存产物在树里，
# E2 的留存产物只在 git 历史里——先入库、再删掉），接连跑三趟（每趟之前把两个假装置的源码各改一个字，行键各不相同，互不复用），每一格往 stdout 打一行「<格名> ok」或「<格名> bad <一句>」：
#   archived-retrieved-green    E2 的留存产物从 git 历史取回再比：E2 字节一致、结果行写出处「取自 git show <提交>^:research/results/e2-sample.out」、整趟退 0
#   device-not-run-red          E1 的假装置打一行 NOT_RUN_SAMPLE_INPUT 退 77：记跑不了（退出码 77）、整趟退 1，没有「本次未跑」的一档；E2 照旧字节一致
#   archived-unrecoverable-red  E2 的留存产物名改成历史里也没有的 e2-never-recorded.out：E2 判「取不回」、结果行点名这个产物、整趟退 1
replay_archived_selftest() {
  local work="$1" root calls out total exit_code=0
  root="$work/root"; calls="$work/devices-ran.log"
  replay_selftest_sample_repository "$work" || return 1
  archived_round() { # <趟号> [环境变量赋值…]：在临时仓里跑一趟副本（样本表的两行 E1、E2 都跑，起法写字面实验号：闭包按它收复跑表的行）；输出进 $out、起了哪几个装置进 $calls、末行 REPLAY_TOTAL 进 $total、退出码进 exit_code
    out="$work/out.$1"; : >"$calls"; : >"$work/not-run.$1"
    printf 'fn main() { println!("one-archived-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e1_sample.rs"
    printf 'fn main() { println!("two-archived-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e2_sample.rs"
    if (cd "$root" && env -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR TMPDIR="$work/tmp" PATH="$work/bin:$PATH" REPLAY_OUT="$work/replay-out.$1" REPLAY_JOBS=1 \
          REPLAY_SELFTEST_RUN_LOG="$calls" REPLAY_SELFTEST_CARGO_LOG="$work/cargo-calls.log" GATE_NOT_RUN_FILE="$work/not-run.$1" \
          SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" SINGLEFS_REUSE_SENTINEL_COUNT=1000 "${@:2}" bash scripts/experiment-apparatus/replay.sh E1 E2) >"$out" 2>&1; then
      exit_code=0
    else
      exit_code=$?
    fi
    total="$(grep '^REPLAY_TOTAL ' "$out")"
  }
  archived_round 1
  if [[ "$exit_code" == 0 ]] && grep -qE '^E1 +e1-sample +字节一致' "$out" \
     && grep -qE '^E2 +e2-sample +字节一致 +e2-sample\.out（取自 git show [0-9a-f]{12}\^:research/results/e2-sample\.out）' "$out"; then
    echo "archived-retrieved-green ok"
  else
    echo "archived-retrieved-green bad E2 的留存产物只在 git 历史里，应当取回再比、判字节一致并写出处、整趟退 0；退 $exit_code，$(grep -E '^E[12] ' "$out" | tr '\n' ';' | clip_text 400)"
  fi
  archived_round 2 REPLAY_SELFTEST_NOT_RUN_DEVICE=e1
  if [[ "$exit_code" == 1 ]] && grep -qE '^E1 +e1-sample +跑不了 +退出码 77' "$out" && grep -qE '^E2 +e2-sample +字节一致' "$out" && ! grep -q '本次未跑 ' "$out"; then
    echo "device-not-run-red ok"
  else
    echo "device-not-run-red bad E1 的装置打了 NOT_RUN 行退 77，应当记跑不了（退出码 77）、整趟退 1、不再有本次未跑的一档；退 $exit_code，$(grep -E '^E[12] ' "$out" | tr '\n' ';' | clip_text 400)"
  fi
  python3 - "$root/scripts/experiment-apparatus/replay.sh" <<'PY' || return 1
import sys
path = sys.argv[1]
lines = open(path, encoding="utf-8").read().split("\n")
rows = [index for index, line in enumerate(lines) if line == "E2|e2-sample||e2-sample.out|exact"]   # 整行相等：只有复跑表那一行，代码里的字面量不算
if len(rows) != 1:
    sys.exit("复跑表里找不到 E2 那一行（整行相等的有 %d 行）" % len(rows))
lines[rows[0]] = "E2|e2-sample||e2-never-recorded.out|exact"
with open(path, "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines))
PY
  archived_round 3
  if [[ "$exit_code" == 1 ]] && grep -qE '^E2 +e2-sample +取不回 +留存产物 e2-never-recorded\.out' "$out"; then
    echo "archived-unrecoverable-red ok"
  else
    echo "archived-unrecoverable-red bad E2 的留存产物树里没有、git 历史里也取不回，应当判取不回、点名 e2-never-recorded.out、整趟退 1；退 $exit_code，$(grep -E '^E[12] ' "$out" | tr '\n' ';' | clip_text 400)"
  fi
}

# 留存产物只以 .out.gz 存在时的比对（replay_one 里的 read_stored：*.gz 先 gzip -dc，E186 两份压成 .gz 入库）的自证，replay_selftest 调：同一份临时仓
# （replay_selftest_sample_repository），副本的复跑表里 E1 的留存产物改成 e1-sample.out.gz（临时仓里没有 .out，只有 .out.gz）；两格各一行收回来：
#   gz-stored-green  留存产物是装置输出的 gzip：整趟退 0，E1 判字节一致、产物名写 e1-sample.out.gz
#   gz-stored-red    留存产物换成内容不同的 gzip：整趟退 1，E1 判对不上、diff 命令点名 e1-sample.out.gz
replay_gzip_selftest() {
  local work="$1" root out exit_code=0
  root="$work/root"
  replay_selftest_sample_repository "$work" || return 1
  python3 - "$root/scripts/experiment-apparatus/replay.sh" <<'PY' || return 1
import sys
path = sys.argv[1]
lines = open(path, encoding="utf-8").read().split("\n")
rows = [index for index, line in enumerate(lines) if line == "E1|e1-sample||e1-sample.out|exact"]   # 整行相等：只有复跑表那一行，代码里的字面量不算
if len(rows) != 1:
    sys.exit("复跑表里找不到 E1 那一行（整行相等的有 %d 行）" % len(rows))
lines[rows[0]] = "E1|e1-sample||e1-sample.out.gz|exact"
with open(path, "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines))
PY
  rm -f -- "$root/research/results/e1-sample.out"
  printf 'E7RESULT name=config arm=e1\nE7RESULT name=done emitted=2\n' | gzip -c >"$root/research/results/e1-sample.out.gz"
  gz_round() { # <趟号>：在临时仓里只跑 E1（起法写字面实验号：闭包按它收复跑表的行）；输出进 $out、退出码进 exit_code。先把 E1 的装置源码改一个字（行键变，不读上一趟记的判定）
    out="$work/out.$1"; : >"$work/devices-ran.log"; : >"$work/not-run.$1"
    printf 'fn main() { println!("one-gz-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e1_sample.rs"
    if (cd "$root" && env -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR TMPDIR="$work/tmp" PATH="$work/bin:$PATH" REPLAY_OUT="$work/replay-out.$1" REPLAY_JOBS=1 \
          REPLAY_SELFTEST_RUN_LOG="$work/devices-ran.log" REPLAY_SELFTEST_CARGO_LOG="$work/cargo-calls.log" GATE_NOT_RUN_FILE="$work/not-run.$1" \
          SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" SINGLEFS_REUSE_SENTINEL_COUNT=1000 bash scripts/experiment-apparatus/replay.sh E1) >"$out" 2>&1; then
      exit_code=0
    else
      exit_code=$?
    fi
  }
  gz_round 1
  if [[ "$exit_code" == 0 ]] && grep -qE '^E1 +e1-sample +字节一致 +e1-sample\.out\.gz' "$out"; then
    echo "gz-stored-green ok"
  else
    echo "gz-stored-green bad 留存产物只以 e1-sample.out.gz 存在（内容是装置输出的 gzip）时应当先解压再比、E1 判字节一致、整趟退 0；退 $exit_code，$(grep -E '^E1 ' "$out" | head -n 1)"
  fi
  printf 'E7RESULT name=config arm=other\nE7RESULT name=done emitted=2\n' | gzip -c >"$root/research/results/e1-sample.out.gz"
  gz_round 2
  if [[ "$exit_code" == 1 ]] && grep -qE '^E1 +e1-sample +对不上 +e1-sample\.out\.gz，[0-9]+ 行不同 → diff .* research/results/e1-sample\.out\.gz' "$out"; then
    echo "gz-stored-red ok"
  else
    echo "gz-stored-red bad 留存产物 e1-sample.out.gz 换成内容不同的 gzip 应当判对不上、整趟退 1；退 $exit_code，$(grep -E '^E1 ' "$out" | head -n 1)"
  fi
}

# 入口脚本格式与三条完整性闸（文件头「入口脚本」「三条完整性闸」）的自证，replay_selftest 调：同一份临时仓（replay_selftest_sample_repository），
# 每一格往 stdout 打一行「<格名> ok」或「<格名> bad <一句>」，每一趟之前先把 E1、E2 的装置源码各改一个字（entry_round：装置源码进行键，各格键不同，互不复用）：
#   list-items              --list-items 那一支打的两个函数在临时仓里直接调（不经入口起：--list-items 带不了实验号）：逐行打 E1、E2（几行、二进制），再逐行打每一行的行项 E1.1、E2.1，都退 0
#   unknown-experiment      点名表里没有的 E9：退 2、点名 E9，本轮输出目录不建、一个装置都不起
#   integrity-exit-code     闸 3：E1 的装置照常打两行却退 3，E1 记「跑不了 退出码 3」、整趟退 1（不当成跑过了）
#   integrity-emitted       闸 2：E2 的装置收尾行说 emitted=3、实收 2 条，E2 记「跑不了」、整趟退 1
#   integrity-previous-output  同一个 REPLAY_OUT 接连两趟：前一趟 E2 的留存产物从 git 历史取回（取回的那一份留在本轮输出目录里），后一趟树里补上了内容不同的留存产物：
#                           E2 判对不上、diff 点名树里的 research/results/e2-sample.out（读树里的，不读上一趟取回的那一份）
replay_entry_selftest() {
  local work="$1" root calls out total exit_code=0
  root="$work/root"; calls="$work/devices-ran.log"
  replay_selftest_sample_repository "$work" || return 1
  entry_round() { # <趟号> <REPLAY_OUT 的名字> <参数>：在临时仓里跑一趟副本；输出进 $out、起了哪几个装置进 $calls、末行 REPLAY_TOTAL 进 $total、退出码进 exit_code。先把 E1、E2 的装置源码各改一个字（行键变，各趟互不复用）
    # 参数只认 E9、E1、E2，各写成一条带字面实验号的起法：闭包按起法里的字面实验号收复跑表的行，参数折不出（"${@:3}" 这类）或不带实验号就连整张复跑表
    out="$work/out.$1"; : >"$calls"; : >"$work/not-run.$1"
    printf 'fn main() { println!("one-entry-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e1_sample.rs"
    printf 'fn main() { println!("two-entry-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e2_sample.rs"
    if (cd "$root" && unset SINGLEFS_MULTI_HOST_CONFIG SINGLEFS_REUSE_SENTINEL_SEED CARGO_TARGET_DIR \
          && export TMPDIR="$work/tmp" PATH="$work/bin:$PATH" REPLAY_OUT="$work/$2" REPLAY_JOBS=1 \
               REPLAY_SELFTEST_RUN_LOG="$calls" REPLAY_SELFTEST_CARGO_LOG="$work/cargo-calls.log" GATE_NOT_RUN_FILE="$work/not-run.$1" \
               SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" SINGLEFS_REUSE_SENTINEL_COUNT=1000 \
          && case "$3" in
               E9) bash scripts/experiment-apparatus/replay.sh E9 ;;
               E1) bash scripts/experiment-apparatus/replay.sh E1 ;;
               E2) bash scripts/experiment-apparatus/replay.sh E2 ;;
               *) echo "entry_round：参数 $3 没有字面起法（只认 E9、E1、E2）" >&2; exit 2 ;;
             esac) >"$out" 2>&1; then
      exit_code=0
    else
      exit_code=$?
    fi
    total="$(grep '^REPLAY_TOTAL ' "$out")"
  }
  device() { # <装置名> <退出码> <收尾行的 emitted=>：把临时仓里那个假装置换成打 config 与 done 两行、退给定退出码的一份（假 cargo 编的时候拷进这一趟的编译目录）
    printf '%s\n' '#!/usr/bin/env bash' "echo $1 >>\"\${REPLAY_SELFTEST_RUN_LOG:?}\"" \
      "printf 'E7RESULT name=config arm=$1\\nE7RESULT name=done emitted=$3\\n'" "exit $2" >"$work/devices/$1-sample"
    chmod +x "$work/devices/$1-sample"
  }
  # list-items：--list-items 只能单独带（主体里 --list-items 那一判见了别的参数就退 2），起法里带不了实验号，只带选项起副本又让闭包连整张复跑表
  # （scripts/gate/lib_function_units.py 文件头「shell 起法带字面参数」）。这一格不经入口起：在临时仓里直接调 --list-items 那一支打的两个函数
  # （replay_table_experiments、replay_table_row_items 从当前目录下的 scripts/experiment-apparatus/replay.sh 原文读复跑表，读到的是样本仓那一份），整份比它们打的行；
  # 经入口起 --list-items 时退 0、不建本轮输出目录、不起装置，这一格不判
  out="$work/out.list-items"
  if (cd "$root" && replay_table_experiments && replay_table_row_items) >"$out" 2>&1; then exit_code=0; else exit_code=$?; fi
  if [[ "$exit_code" == 0 && "$(cat -- "$out")" == $'E1\t1\te1-sample\nE2\t1\te2-sample\nE1.1\t1\te1-sample\nE2.1\t1\te2-sample' ]]; then
    echo "list-items ok"
  else
    echo "list-items bad 样本仓里 replay_table_experiments 与 replay_table_row_items 应当逐行打 E1、E2（几行、二进制）与行项 E1.1、E2.1、都退 0；退 $exit_code，输出「$(tr '\n' ';' <"$out" | clip_text 300)」"
  fi
  entry_round 2 unknown-out E9
  if [[ "$exit_code" == 2 ]] && grep -qF '复跑表里没有：E9' "$out" && [[ ! -e "$work/unknown-out" && ! -s "$calls" ]]; then
    echo "unknown-experiment ok"
  else
    echo "unknown-experiment bad 点名表里没有的 E9 应当退 2、点名 E9、不建本轮输出目录、不起装置；退 $exit_code，输出尾部「$(tail -n 3 "$out" | tr '\n' ' ' | clip_text 300)」"
  fi
  device e1 3 2
  entry_round 3 exit-code-out E1
  if [[ "$exit_code" == 1 ]] && grep -qE '^E1 +e1-sample +跑不了 +退出码 3' "$out"; then
    echo "integrity-exit-code ok"
  else
    echo "integrity-exit-code bad E1 的装置照常打两行却退 3，应当记跑不了、整趟退 1；退 $exit_code，$(grep -E '^E1 ' "$out" | head -n 1)"
  fi
  device e1 0 2
  device e2 0 3
  entry_round 4 emitted-out E2
  if [[ "$exit_code" == 1 ]] && grep -qE '^E2 +e2-sample +跑不了 +第 1 段收尾行说 3 条，实收 2 条' "$out"; then
    echo "integrity-emitted ok"
  else
    echo "integrity-emitted bad E2 的收尾行说 emitted=3、实收 2 条，应当记跑不了、整趟退 1；退 $exit_code，$(grep -E '^E2 ' "$out" | head -n 1)"
  fi
  device e2 0 2
  entry_round 5 previous-out E2
  local earlier_round_exit="$exit_code" earlier_round_line
  earlier_round_line="$(grep -E '^E2 ' "$out" | head -n 1)"
  printf 'E7RESULT name=config arm=other\nE7RESULT name=done emitted=2\n' >"$root/research/results/e2-sample.out"
  entry_round 6 previous-out E2
  rm -f -- "$root/research/results/e2-sample.out"
  if [[ "$earlier_round_exit" == 0 && "$exit_code" == 1 ]] && grep -qE '^E2 +e2-sample +字节一致 +e2-sample\.out（取自 git show' <<<"$earlier_round_line" \
     && grep -qE '^E2 +e2-sample +对不上 +e2-sample\.out，[0-9]+ 行不同 → diff .* research/results/e2-sample\.out' "$out"; then
    echo "integrity-previous-output ok"
  else
    echo "integrity-previous-output bad 同一个 REPLAY_OUT 接连两趟，前一趟 E2 取回留存产物判字节一致，后一趟树里补上了内容不同的留存产物，应当判对不上、diff 点名树里的那一份；前一趟退 $earlier_round_exit（$earlier_round_line），后一趟退 $exit_code（$(grep -E '^E2 ' "$out" | head -n 1)）"
  fi
}

# 派行与进度（文件头「派行」「进度」）的自证，replay_selftest 调：每格一棵样本仓（replay_selftest_sample_repository），复跑表换成这一格的几行（都是假装置 e1-sample，参数是行的名字），
# 峰值记录放这一格的 records/（REPLAY_ROW_COSTS_DIRECTORY），上限经 TASK_SERVER_MEMORY_CAP_BYTES 交、cgroup 那一档指到不在的文件（不收窄），负载关阈值放宽、不等负载（这几格不判负载关）。
# 假装置往 $REPLAY_SELFTEST_RUN_LOG 记「start <名字>」「end <名字>」，起来之后等到 REPLAY_SELFTEST_BARRIER 行起来了（至多 3 秒）再拿住 REPLAY_SELFTEST_DEVICE_SECONDS 秒：放得下的行真的一起跑，放不下的等不到也照走。
# 每一格往 stdout 打一行「<格名> ok」或「<格名> bad <一句>」。
replay_dispatch_selftest() {
  local work="$1" real_root="$PWD" cell root calls exit_code=0 replay_pid
  local costs_script="$real_root/scripts/experiment-apparatus/replay_row_costs.py"
  local -a dispatch_env=()
  local e1=e1-sample.out one_gibibyte=1073741824 big_cap=68719476736
  dispatch_tree() { # <格名> <多机配置里多写的一行，没有写空串> <复跑表的行…>
    cell="$work/$1"; root="$cell/root"; calls="$cell/devices-ran.log"
    local configuration_line="$2"
    shift 2
    replay_selftest_sample_repository "$cell" || return 1
    mkdir -p "$cell/records"
    printf '%s\n' '# 自证的多机配置' "$configuration_line" >"$cell/multi-host.env"
    printf '%s\n' '#!/usr/bin/env bash' \
      'echo "start ${1:-none} cards=${GPU_CARDS_BY_PRIORITY:-none}" >>"$REPLAY_SELFTEST_RUN_LOG"' \
      'if [[ "${1:-}" == hold ]]; then [[ -p "$REPLAY_SELFTEST_DEVICE_HOLD.fifo" ]] || mkfifo -- "$REPLAY_SELFTEST_DEVICE_HOLD.fifo" || exit 1; echo "$$" >"$REPLAY_SELFTEST_DEVICE_HOLD"; read -r -t 8 _ <>"$REPLAY_SELFTEST_DEVICE_HOLD.fifo"; fi' \
      'for _ in {1..60}; do (( $(grep -c "^start " "$REPLAY_SELFTEST_RUN_LOG") >= ${REPLAY_SELFTEST_BARRIER:-1} )) && break; sleep 0.05; done' \
      'sleep "${REPLAY_SELFTEST_DEVICE_SECONDS:-0.4}"' \
      'echo "end ${1:-none}" >>"$REPLAY_SELFTEST_RUN_LOG"' \
      'printf "E7RESULT name=config arm=e1\n"' \
      'if [[ "${1:-}" == timing ]]; then printf "E7RESULT name=done emitted=2 loadavg1=0.00 cpus=32\n"; else printf "E7RESULT name=done emitted=2\n"; fi' \
      '[[ "${1:-}" != fail ]] || exit 3' >"$cell/devices/e1-sample"
    chmod +x "$cell/devices/e1-sample"
    printf 'E7RESULT name=config arm=e1\nE7RESULT name=done emitted=2 loadavg1=0.00 cpus=32\n' >"$root/research/results/e1-timing.out"
    python3 - "$root/scripts/experiment-apparatus/replay.sh" "$@" <<'PY' || return 1
import sys
path, rows = sys.argv[1], sys.argv[2:]
text = open(path, encoding="utf-8").read()
table = "TABLE=$(cat <<'TSV'\n"
table_start = text.index(table) + len(table)
table_end = text.index("\nTSV\n)\n", table_start)
drivers = ("replay_selftest_gpu_row_one() {\n  : \"${CHECKER_TIER_FEATURES[*]-}\"\n  \"$REPLAY_RELEASE_DIRECTORY/e1-sample\" gpu-one\n}\n"
           "replay_selftest_gpu_row_two() {\n  : \"${CHECKER_TIER_FEATURES[*]-}\"\n  \"$REPLAY_RELEASE_DIRECTORY/e1-sample\" gpu-two\n}\n"
           "replay_selftest_driver_leased_row() {\n  : \"${CHECKER_TIER_FEATURES[*]-}\"\n"
           "  \"$REPLAY_RELEASE_DIRECTORY/e1-sample\" own-cards\n}\n")
text = text[:table_start - len(table)] + drivers + table + "\n".join(rows) + text[table_end:]
open(path, "w", encoding="utf-8").write(text)
PY
    chmod +x "$root/scripts/experiment-apparatus/replay.sh"
  }
  dispatch_seed() { # <身份> <峰值字节> <线程数>：造一份量过的记录
    python3 "$costs_script" record "$cell/records" "$1" "$2" "$3"
  }
  dispatch_settings() { # 这一格每一趟共用的环境（dispatch_env 接在后面）
    printf '%s\0' SINGLEFS_MULTI_HOST_CONFIG="$cell/multi-host.env" GATE_MUTATION_CGROUP_LISTING="$cell/no-such-cgroup-listing" TMPDIR="$cell/tmp" PATH="$cell/bin:$PATH" \
      REPLAY_OUT="$cell/replay-out" REPLAY_SELFTEST_RUN_LOG="$calls" REPLAY_SELFTEST_CARGO_LOG="$cell/cargo-calls.log" GATE_NOT_RUN_FILE="$cell/not-run" \
      SINGLEFS_LAST_RUNS_DIRECTORY="$cell/last-runs" SINGLEFS_REUSE_SENTINEL_COUNT=1000 REPLAY_ROW_COSTS_DIRECTORY="$cell/records" REPLAY_TIMING_SETTLE_SECONDS=0 REPLAY_TIMING_LOAD_RATIO=1000 \
      ${dispatch_env[@]+"${dispatch_env[@]}"}
  }
  dispatch_run() { # <参数…>：在这一格的样本仓里跑一趟：标准输出与标准错误分开进 stdout、stderr，起止记录先清空，退出码进 exit_code
    # 参数只认 case 里那几组，各写成一条带字面实验号或行项的起法（要整张样本表都跑的，写这一格表里的全部实验号）：闭包按起法里的字面词收复跑表的行，
    # 参数折不出（"$@" 这类）或不带实验号就连整张复跑表（scripts/gate/lib_function_units.py 文件头「shell 起法带字面参数」）
    local -a settings=()
    mapfile -d '' -t settings < <(dispatch_settings)
    : >"$calls"
    if (cd "$root" && unset SINGLEFS_REUSE_SENTINEL_SEED CARGO_TARGET_DIR TASK_SERVER_MEMORY_CAP_BYTES && export "${settings[@]}" \
          && case "$*" in
               "E1A E1B E1C E1D") bash scripts/experiment-apparatus/replay.sh E1A E1B E1C E1D ;;
               "E1A E1B E1C") bash scripts/experiment-apparatus/replay.sh E1A E1B E1C ;;
               "E1A E1B") bash scripts/experiment-apparatus/replay.sh E1A E1B ;;
               E1A) bash scripts/experiment-apparatus/replay.sh E1A ;;
               E1) bash scripts/experiment-apparatus/replay.sh E1 ;;
               E181) bash scripts/experiment-apparatus/replay.sh E181 ;;
               "--item E1.2") bash scripts/experiment-apparatus/replay.sh --item E1.2 ;;
               "E1 --item E1.3") bash scripts/experiment-apparatus/replay.sh E1 --item E1.3 ;;
               "--item E1.9") bash scripts/experiment-apparatus/replay.sh --item E1.9 ;;
               "E1 --item") bash scripts/experiment-apparatus/replay.sh E1 --item ;;
               *) echo "dispatch_run：参数「$*」没有字面起法（只认 E1A E1B E1C E1D、E1A E1B E1C、E1A E1B、E1A、E1、E181、--item E1.2、E1 --item E1.3、--item E1.9、E1 --item）" >&2; exit 2 ;;
             esac) >"$cell/stdout" 2>"$cell/stderr"; then
      exit_code=0
    else
      exit_code=$?
    fi
  }
  dispatch_most_at_once() { # 同时在跑的行数最多几行
    awk '$1 == "start" { running++; if (running > most) most = running } $1 == "end" { running-- } END { print most + 0 }' "$calls"
  }
  dispatch_company_of() { # <名字>：这一行在跑的时候，同时在跑的行数（连它自己）最多几行
    awk -v row="$1" '$1 == "start" { running++; on[$2] = 1; if (on[row] && running > most) most = running } $1 == "end" { running--; delete on[$2] } END { print most + 0 }' "$calls"
  }
  dispatch_seen() { # 这一趟的起止记录，一行写完（报错用）
    tr '\n' ' ' <"$calls" | clip_text 300
  }
  dispatch_tail() { # 这一趟标准错误的末尾（报错用）
    tail -n 4 "$cell/stderr" | tr '\n' ' ' | clip_text 400
  }

  dispatch_tree dispatch-memory-bound "" "E1A|e1-sample|a|$e1|exact" "E1B|e1-sample|b|$e1|exact" "E1C|e1-sample|c|$e1|exact" || return 1
  dispatch_seed 'E1A|e1-sample|a' "$one_gibibyte" 1 && dispatch_seed 'E1B|e1-sample|b' "$one_gibibyte" 1 && dispatch_seed 'E1C|e1-sample|c' "$one_gibibyte" 1 || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES=3221225472 REPLAY_SELFTEST_BARRIER=2)
  dispatch_run E1A E1B E1C
  if [[ "$exit_code" == 0 && "$(dispatch_most_at_once)" == 2 ]] && grep -q '这一台能给的内存取不到（多机配置 .* 没写 SLICE_MEMORY_CAP_BY_HOST）' "$cell/stderr"; then
    echo "dispatch-memory-bound ok"
  else
    echo "dispatch-memory-bound bad 三行各量过 1 GiB、上限 3 GiB、多机配置没写这一台能给的内存，应当退 0、同时恰好两行、标准错误写读数脚本报的取不到的原因（没写 SLICE_MEMORY_CAP_BY_HOST）；退 $exit_code，同时至多 $(dispatch_most_at_once) 行，起止「$(dispatch_seen)」，$(dispatch_tail)"
  fi

  dispatch_tree dispatch-task-cap-not-binding "$(printf '%s\n' "HOSTS=127.0.0.1 192.0.2.70" \
      "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.70=/selftest/other-cargo" \
      "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.70=/selftest/other-builds" \
      "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=64G 192.0.2.70=1G" \
      "QUIESCE_STOP_COMMAND=" "QUIESCE_STOPPED_CHECK_COMMAND=" "QUIESCE_START_COMMAND=" "QUIESCE_STARTED_CHECK_COMMAND=" \
      "ENABLE_ACROSS_MACHINES=0" "ENABLE_GPU=0" "NFS_SHARED_ROOTS=$work/dispatch-task-cap-not-binding")" \
    "E1A|e1-sample|a|$e1|exact" "E1B|e1-sample|b|$e1|exact" "E1C|e1-sample|c|$e1|exact" || return 1
  dispatch_seed 'E1A|e1-sample|a' "$one_gibibyte" 1 && dispatch_seed 'E1B|e1-sample|b' "$one_gibibyte" 1 && dispatch_seed 'E1C|e1-sample|c' "$one_gibibyte" 1 || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES=3221225472 REPLAY_SELFTEST_BARRIER=3)
  dispatch_run E1A E1B E1C
  if [[ "$exit_code" == 0 && "$(dispatch_most_at_once)" == 3 ]] && grep -qF '比这一台能给的内存 68719476736 字节' "$cell/stderr"; then
    echo "dispatch-task-cap-not-binding ok"
  else
    echo "dispatch-task-cap-not-binding bad 三行各量过 1 GiB、服务交的上限 3 GiB（只够两行）、多机配置这一台那一项 64G：同时放得下几行比这一台能给的内存，应当三行一起跑、标准错误写比的是 64G 的字节数；退 $exit_code，同时至多 $(dispatch_most_at_once) 行，起止「$(dispatch_seen)」，$(dispatch_tail)"
  fi

  dispatch_tree dispatch-threads-unbounded "" "E1A|e1-sample|a|$e1|exact" "E1B|e1-sample|b|$e1|exact" "E1C|e1-sample|c|$e1|exact" || return 1
  dispatch_seed 'E1A|e1-sample|a' "$one_gibibyte" 31 && dispatch_seed 'E1B|e1-sample|b' "$one_gibibyte" 31 && dispatch_seed 'E1C|e1-sample|c' "$one_gibibyte" 31 || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap" REPLAY_SELFTEST_BARRIER=3)
  dispatch_run E1A E1B E1C
  if [[ "$exit_code" == 0 && "$(dispatch_most_at_once)" == 3 ]]; then
    echo "dispatch-threads-unbounded ok"
  else
    echo "dispatch-threads-unbounded bad 三行各量过 31 线程、REPLAY_JOBS=8、内存放得下：线程不累计，应当三行一起跑；退 $exit_code，同时至多 $(dispatch_most_at_once) 行，起止「$(dispatch_seen)」，$(dispatch_tail)"
  fi

  dispatch_tree dispatch-row-count-bound "" "E1A|e1-sample|a|$e1|exact" "E1B|e1-sample|b|$e1|exact" "E1C|e1-sample|c|$e1|exact" || return 1
  dispatch_seed 'E1A|e1-sample|a' "$one_gibibyte" 1 && dispatch_seed 'E1B|e1-sample|b' "$one_gibibyte" 1 && dispatch_seed 'E1C|e1-sample|c' "$one_gibibyte" 1 || return 1
  dispatch_env=(REPLAY_JOBS=2 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap" REPLAY_SELFTEST_BARRIER=2)
  dispatch_run E1A E1B E1C
  if [[ "$exit_code" == 0 && "$(dispatch_most_at_once)" == 2 ]]; then
    echo "dispatch-row-count-bound ok"
  else
    echo "dispatch-row-count-bound bad REPLAY_JOBS=2、内存放得下：应当同时恰好两行；退 $exit_code，同时至多 $(dispatch_most_at_once) 行，起止「$(dispatch_seen)」，$(dispatch_tail)"
  fi

  dispatch_tree dispatch-exclusive-rows "" "E1A|e1-sample|timing|e1-timing.out|timing" "E1B|e1-sample|a|$e1|exact" "E1C|e1-sample|unmeasured|$e1|exact" "E1D|e1-sample|b|$e1|exact" || return 1
  # 没量过的那一行按每台默认重活线程数算线程（派行那一段调 heavy_thread_default，弄坏开关 threads-bound-kept 拿它比 REPLAY_JOBS）。这一格往样本仓放一份钉 16 的取法
  #   （大于这一格的 REPLAY_JOBS=8），开关下红不红与外层交的 SINGLEFS_THREAD_CAP 无关（break-switch-replay 按外层路数分，外层 31 路时交 1）。
  #   钉的取法设成只读函数：副本在 read_gpu_switch 里 source scripts/crash/crash-amplification-lib.sh，那份库再 source 它旁边的 ../hosts/heavy-threads.sh；
  #   样本仓的 scripts/crash 是指到真仓的符号链接，那一句读进来的是真仓那一份（照 SINGLEFS_THREAD_CAP 取），不设只读就把钉的这一份盖掉。设了只读，那一句报一行 readonly function、照往下读。
  #   不改 SINGLEFS_THREAD_CAP：副本起的 admission.py replay-row-inputs 照它分子进程，要守外层给的线程额度。
  #   钉的取法每调一次往起止记录写一行「threads 16 <调它的函数>」（派行的几个 awk 只认 start、end 行），这一格要它至少调过一次：盖掉了就红在这一格，不悄悄照 SINGLEFS_THREAD_CAP 算
  printf '%s\n' '# 自证样本仓的每台默认重活线程数取法：钉成 16、只读，每调一次往起止记录写一行（replay.sh 的 dispatch-exclusive-rows 那一格）' \
    'heavy_thread_default() { echo "threads 16 ${FUNCNAME[1]:-main}" >>"${REPLAY_SELFTEST_RUN_LOG:?}"; echo 16; }' 'readonly -f heavy_thread_default' \
    >"$root/scripts/hosts/heavy-threads.sh" || return 1
  dispatch_seed 'E1A|e1-sample|timing' "$one_gibibyte" 1 && dispatch_seed 'E1B|e1-sample|a' "$one_gibibyte" 1 && dispatch_seed 'E1D|e1-sample|b' "$one_gibibyte" 1 || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap" REPLAY_SELFTEST_BARRIER=3)
  dispatch_run E1A E1B E1C E1D
  local exclusive_rows_threads_calls
  exclusive_rows_threads_calls="$(grep -c '^threads 16 ' "$calls")"
  if [[ "$exit_code" == 0 && "$(dispatch_company_of a)" == 3 && "$(dispatch_company_of timing)" == 1 && "$(dispatch_company_of unmeasured)" == 3 ]] \
     && [[ "$(awk '$1 == "start" { last = $2 } END { print last }' "$calls")" == timing ]] \
     && [[ "$(grep -E '^E1[A-D] ' "$cell/stdout" | awk '{ printf "%s ", $1 }')" == "E1A E1B E1C E1D " ]] && (( exclusive_rows_threads_calls > 0 )); then
    echo "dispatch-exclusive-rows ok"
  else
    echo "dispatch-exclusive-rows bad a、b 与没量过的那一行应当一起跑，timing 行单独跑、排在后面，结果行照旧按表序，样本仓钉 16 的取法要调到；退 $exit_code，a 那时 $(dispatch_company_of a) 行、timing 那时 $(dispatch_company_of timing) 行、没量过的那时 $(dispatch_company_of unmeasured) 行，钉的取法调了 $exclusive_rows_threads_calls 次，起止「$(dispatch_seen)」，$(dispatch_tail)"
  fi

  # 要卡的那一段（文件头「GPU」「派行」）：要卡的两行在表里排在不要卡的那一行前面，派行把它们挪到最后；起止记录里逐条记「start <参数> cards=<看到的卡>」「end <参数>」，
  #   dispatch_card_events 把它们连成一串（start 带看到的卡），这三格比的是这一串
  dispatch_card_events() {
    awk '$1 == "start" { sub(/^cards=/, "", $3); printf "%s:%s:%s ", $1, $2, $3; next } { printf "%s:%s ", $1, $2 }' "$calls"
  }
  local card_rows_events gpu_rows_serial_events="start:cpu:none end:cpu start:gpu-one:selftest-card end:gpu-one start:gpu-two:selftest-card end:gpu-two "
  dispatch_tree dispatch-gpu-rows-serial "ENABLE_GPU=1" "E1A|@replay_selftest_gpu_row_one||$e1|exact" "E1B|@replay_selftest_gpu_row_two||$e1|exact" "E1C|e1-sample|cpu|$e1|exact" || return 1
  dispatch_seed 'E1A|@replay_selftest_gpu_row_one|' "$one_gibibyte" 1 && dispatch_seed 'E1B|@replay_selftest_gpu_row_two|' "$one_gibibyte" 1 && dispatch_seed 'E1C|e1-sample|cpu' "$one_gibibyte" 1 || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap" GPU_CARDS_BY_PRIORITY=selftest-card)
  dispatch_run E1A E1B E1C
  card_rows_events="$(dispatch_card_events)"
  if [[ "$exit_code" == 0 && "$card_rows_events" == "$gpu_rows_serial_events" ]] \
     && [[ "$(grep -E '^E1[A-C] ' "$cell/stdout" | awk '{ printf "%s:%s ", $1, $3 }')" == "E1A:字节一致 E1B:字节一致 E1C:字节一致 " ]]; then
    echo "dispatch-gpu-rows-serial ok"
  else
    echo "dispatch-gpu-rows-serial bad 不要卡的一行应当先跑完、看不到卡，要卡的两行一行一行跑、都看到环境里给的卡串，三行都字节一致、结果行照表序，整趟退 0（应为「$gpu_rows_serial_events」）；退 $exit_code，实得「$card_rows_events」，结果行「$(grep -E '^E1[A-C] ' "$cell/stdout" | awk '{ printf "%s:%s ", $1, $3 }')」，$(dispatch_tail)"
  fi

  local gpu_rows_without_cards_events="start:cpu:none end:cpu start:gpu-one:none end:gpu-one start:gpu-two:none end:gpu-two "
  dispatch_tree dispatch-gpu-rows-without-cards "ENABLE_GPU=1" "E1A|@replay_selftest_gpu_row_one||$e1|exact" "E1B|@replay_selftest_gpu_row_two||$e1|exact" "E1C|e1-sample|cpu|$e1|exact" || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap")
  dispatch_run E1A E1B E1C
  card_rows_events="$(dispatch_card_events)"
  if [[ "$exit_code" == 0 && "$card_rows_events" == "$gpu_rows_without_cards_events" ]] \
     && [[ "$(grep -E '^E1[A-C] ' "$cell/stdout" | awk '{ printf "%s:%s ", $1, $3 }')" == "E1A:字节一致 E1B:字节一致 E1C:字节一致 " ]]; then
    echo "dispatch-gpu-rows-without-cards ok"
  else
    echo "dispatch-gpu-rows-without-cards bad 环境里没有卡串：照样不拦，要卡的两行一行一行跑、看不到卡，三行都字节一致，整趟退 0（应为「$gpu_rows_without_cards_events」）；退 $exit_code，实得「$card_rows_events」，结果行「$(grep -E '^E1[A-C] ' "$cell/stdout" | awk '{ printf "%s:%s ", $1, $3 }')」，$(dispatch_tail)"
  fi

  # 本脚本不交卡的行（文件头「GPU」「派行」，REPLAY_DRIVERS_LEASING_THEIR_OWN_CARDS）：样本仓里那张登记加上 replay_selftest_driver_leased_row（它起假装置 own-cards），
  #   排在表里最前；照常交卡的 gpu-two 与不要卡的 cpu 各一行。卡串只交给 gpu-two，own-cards 在那一段跑完之后跑、看不到卡
  local driver_leased_events="start:cpu:none end:cpu start:gpu-two:selftest-card end:gpu-two start:own-cards:none end:own-cards "
  dispatch_tree dispatch-driver-leased-rows "ENABLE_GPU=1" "E1A|@replay_selftest_driver_leased_row||$e1|exact" "E1B|@replay_selftest_gpu_row_two||$e1|exact" "E1C|e1-sample|cpu|$e1|exact" || return 1
  python3 - "$root/scripts/experiment-apparatus/replay.sh" <<'PY' || return 1
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
head = "\nREPLAY_DRIVERS_LEASING_THEIR_OWN_CARDS=("
assert text.count(head) == 1
text = text.replace(head, head + "replay_selftest_driver_leased_row ")
open(path, "w", encoding="utf-8").write(text)
PY
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap" GPU_CARDS_BY_PRIORITY=selftest-card)
  dispatch_run E1A E1B E1C
  card_rows_events="$(dispatch_card_events)"
  if [[ "$exit_code" == 0 && "$card_rows_events" == "$driver_leased_events" ]] \
     && [[ "$(grep -E '^E1[A-C] ' "$cell/stdout" | awk '{ printf "%s:%s ", $1, $3 }')" == "E1A:字节一致 E1B:字节一致 E1C:字节一致 " ]]; then
    echo "dispatch-driver-leased-rows ok"
  else
    echo "dispatch-driver-leased-rows bad 登记成本脚本不交卡的一行应当在交卡那一段跑完之后跑、看不到卡，卡串只交给照常交卡的那一行，三行都字节一致，整趟退 0（应为「$driver_leased_events」）；退 $exit_code，实得「$card_rows_events」，结果行「$(grep -E '^E1[A-C] ' "$cell/stdout" | awk '{ printf "%s:%s ", $1, $3 }')」，$(dispatch_tail)"
  fi

  dispatch_tree dispatch-oversized-row-refused "" "E1A|e1-sample|a|$e1|exact" || return 1
  dispatch_seed 'E1A|e1-sample|a' 42949672960 1 || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES=3221225472)
  dispatch_run E1A
  if [[ "$exit_code" == 250 && ! -s "$calls" ]] && grep -qF 'E1A.1（量过 42949672960 字节）' "$cell/stderr" && ! grep -qs '^build ' "$cell/cargo-calls.log"; then
    echo "dispatch-oversized-row-refused ok"
  else
    echo "dispatch-oversized-row-refused bad 量过 40 GiB、上限 3 GiB，应当退 250、点名那一行、不编、不起装置；退 $exit_code，起止「$(dispatch_seen)」，$(dispatch_tail)"
  fi

  dispatch_tree row-progress-lines "" "E1A|e1-sample|a|$e1|exact" "E1B|e1-sample|b|$e1|exact" || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap" REPLAY_PROGRESS_SECONDS=1 REPLAY_SELFTEST_DEVICE_SECONDS=1.6)
  dispatch_run E1A E1B
  if [[ "$exit_code" == 0 ]] \
     && (( $(grep -c '^REPLAY_ROW E1A\.1 开跑$' "$cell/stderr") == 1 && $(grep -c '^REPLAY_ROW E1B\.2 开跑$' "$cell/stderr") == 1 )) \
     && (( $(grep -cE '^REPLAY_ROW E1A\.1 判 字节一致 用时 [0-9]+ 秒 峰值 [1-9][0-9]* 线程 [1-9][0-9]*｜E1A ' "$cell/stderr") == 1 )) \
     && (( $(grep -cE '^REPLAY_ROW E1B\.2 判 字节一致 用时 [0-9]+ 秒 峰值 [1-9][0-9]* 线程 [1-9][0-9]*｜E1B ' "$cell/stderr") == 1 )) \
     && grep -qE '^REPLAY_ROW E1[AB]\.[12] 在跑 用时 [0-9]+ 秒 峰值 ' "$cell/stderr" && ! grep -q '^REPLAY_ROW' "$cell/stdout"; then
    echo "row-progress-lines ok"
  else
    echo "row-progress-lines bad 现跑的每一行应当起止各一行、峰值是正整数、在跑的报进度、标准输出里没有这几行；退 $exit_code，标准错误里的进度「$(grep '^REPLAY_ROW' "$cell/stderr" | clip_text 90 | tr '\n' ';' | clip_text 500)」，$(dispatch_tail)"
  fi

  dispatch_tree row-peak-recorded-then-shared "" "E1A|e1-sample|a|$e1|exact" "E1B|e1-sample|b|$e1|exact" || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap")
  dispatch_run E1A E1B
  local records_after_unshared_run
  records_after_unshared_run="$(find "$cell/records" -type f | wc -l)"
  printf 'fn main() { println!("one-second-round"); }\n' >"$root/scripts/experiment-apparatus/e1_sample.rs"
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap" REPLAY_SELFTEST_BARRIER=2)
  dispatch_run E1A E1B
  if [[ "$exit_code" == 0 && "$(dispatch_most_at_once)" == 2 ]] && (( records_after_unshared_run >= 2 )); then
    echo "row-peak-recorded-then-shared ok"
  else
    echo "row-peak-recorded-then-shared bad 跑一趟两行量到的峰值都进记录，下一趟这两行应当一起跑；头一趟之后记录 $records_after_unshared_run 份，下一趟退 $exit_code、同时至多 $(dispatch_most_at_once) 行，起止「$(dispatch_seen)」，$(dispatch_tail)"
  fi

  dispatch_tree row-failed-peak-not-recorded "" "E1A|e1-sample|fail|$e1|exact" || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap")
  dispatch_run E1A
  if [[ "$exit_code" == 1 ]] && grep -qE '^E1A +e1-sample +跑不了 +退出码 3' "$cell/stdout" && [[ -z "$(find "$cell/records" -type f)" ]]; then
    echo "row-failed-peak-not-recorded ok"
  else
    echo "row-failed-peak-not-recorded bad 装置退 3 应当记跑不了、整趟退 1、不记峰值；退 $exit_code，记录 $(find "$cell/records" -type f | wc -l) 份，$(dispatch_tail)"
  fi

  dispatch_tree stopped-keeps-finished-rows "" "E1A|e1-sample|a|$e1|exact" "E1B|e1-sample|hold|$e1|exact" || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap" REPLAY_SELFTEST_DEVICE_HOLD="$cell/device-hold.pid")
  local -a stop_settings=()
  mapfile -d '' -t stop_settings < <(dispatch_settings)
  : >"$calls"
  # 起法写这一格表里的全部实验号（同 dispatch_run 那一句：闭包按字面实验号收复跑表的行）
  env -C "$root" -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR -u TASK_SERVER_MEMORY_CAP_BYTES "${stop_settings[@]}" bash scripts/experiment-apparatus/replay.sh E1A E1B >"$cell/stdout" 2>"$cell/stderr" &
  replay_pid=$!
  for _ in {1..150}; do
    [[ -s "$cell/device-hold.pid" && -f "$cell/replay-out/E1A.1.line" ]] && break
    sleep 0.1
  done
  kill -TERM "$replay_pid" 2>/dev/null
  wait "$replay_pid"
  exit_code=$?
  if [[ "$exit_code" == 143 ]] && grep -qE '^E1A +e1-sample +字节一致' "$cell/stdout" && grep -qx 'REPLAY_STOPPED finished=1 dispatched=2' "$cell/stdout" && ! grep -qE '^E1B ' "$cell/stdout"; then
    echo "stopped-keeps-finished-rows ok"
  else
    echo "stopped-keeps-finished-rows bad E1A 判完、E1B 在跑时发 TERM，应当退 143、标准输出有 E1A 的结果行与 REPLAY_STOPPED，没有 E1B 的；退 $exit_code，标准输出「$(tr '\n' ';' <"$cell/stdout" | clip_text 300)」，$(dispatch_tail)"
  fi

  local e181_product=e181-landing-check-fixed-point-and-mount-lower-bound-landing.out
  dispatch_tree e181-goes-through-long-run "" "E181|@driver_e181||$e181_product|timing" || return 1
  printf 'E7RESULT name=row\nE7RESULT name=done emitted=2 loadavg1=0.00 cpus=32\n' >"$root/research/results/$e181_product"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "long-run arguments=[$*]" >>"$REPLAY_SELFTEST_RUN_LOG"' 'echo "片 checks：开跑"' 'mkdir -p -- "$E181_PRODUCT_DIRECTORY"' \
    "printf 'E7RESULT name=row\nE7RESULT name=done emitted=2 loadavg1=0.00 cpus=32\n' >\"\$E181_PRODUCT_DIRECTORY/$e181_product\"" >"$root/scripts/experiment-apparatus/e181-landing-check-long-run.sh"
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap")
  dispatch_run E181
  local e181_driver_text
  e181_driver_text="$(sed -n '/^driver_e181() {$/,/^}$/p' "$root/scripts/experiment-apparatus/replay.sh")"
  if [[ "$exit_code" == 0 ]] && grep -qE '^E181 +@driver_e181 +字节一致' "$cell/stdout" \
     && grep -qx 'long-run arguments=\[\]' "$calls" && ! grep -q '^run ' "$cell/cargo-calls.log" \
     && grep -qF -e '-p singlefs-implementation-experiments --bin e181_landing_check_fixed_point_and_mount_lower_bound' <<<"$e181_driver_text"; then
    echo "e181-goes-through-long-run ok"
  else
    echo "e181-goes-through-long-run bad E181 应当不带参数起长跑脚本、把它拼出的产物交给判法（判字节一致、整趟退 0），不起 cargo run，驱动正文字面点名装置；退 $exit_code，结果行「$(grep -E '^E181 ' "$cell/stdout" | clip_text 200)」，长跑脚本「$(cat "$calls")」，cargo「$(tr '\n' ';' <"$cell/cargo-calls.log" | clip_text 300)」，$(dispatch_tail)"
  fi

  # --item（文件头「入口脚本」）：一个实验号三行（参数 a、b、c，假装置往起止记录里写自己的参数）；点 E1.2 只起 b，E1 加 --item E1.3 只起 c，
  #   起的行数、派出数（REPLAY_TOTAL dispatched=）都是 1；行项写错与 --item 缺项名都退 2、点名项名、列出 E1 的行项、不建本轮输出目录、一个装置都不起
  #   （缺项名那一趟写成 E1 --item：只带 --item 的起法里没有实验号，闭包照只带选项连整张复跑表）
  dispatch_tree dispatch-item-selects-one-row "" "E1|e1-sample|a|$e1|exact" "E1|e1-sample|b|$e1|exact" "E1|e1-sample|c|$e1|exact" || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap")
  dispatch_run --item E1.2
  local item_alone_exit="$exit_code" item_alone_started item_alone_total
  item_alone_started="$(awk '$1 == "start" { printf "%s ", $2 }' "$calls")"; item_alone_total="$(grep -c '^REPLAY_TOTAL dispatched=1 ' "$cell/stdout")"
  dispatch_run E1 --item E1.3
  if [[ "$item_alone_exit" == 0 && "$item_alone_started" == "b " && "$item_alone_total" == 1 ]] \
     && [[ "$exit_code" == 0 && "$(awk '$1 == "start" { printf "%s ", $2 }' "$calls")" == "c " ]] && grep -q '^REPLAY_TOTAL dispatched=1 ' "$cell/stdout"; then
    echo "dispatch-item-selects-one-row ok"
  else
    echo "dispatch-item-selects-one-row bad 一个实验号三行：--item E1.2 应当只起 b、派出 1 行，E1 --item E1.3 应当只起 c、派出 1 行，都退 0；--item E1.2 退 $item_alone_exit，起了「$item_alone_started」；E1 --item E1.3 退 $exit_code，起止「$(dispatch_seen)」，$(dispatch_tail)"
  fi
  dispatch_tree dispatch-item-unknown-refused "" "E1|e1-sample|a|$e1|exact" "E1|e1-sample|b|$e1|exact" "E1|e1-sample|c|$e1|exact" || return 1
  dispatch_run --item E1.9
  local item_unknown_exit="$exit_code" item_unknown_listed item_unknown_named item_missing_exit
  item_unknown_listed="$(grep -c 'E1.1 E1.2 E1.3' "$cell/stderr")"
  item_unknown_named="$(grep -cF 'E1.9' "$cell/stderr")"
  dispatch_run E1 --item
  item_missing_exit="$exit_code"
  if [[ "$item_unknown_exit" == 2 && "$item_missing_exit" == 2 && "$item_unknown_listed" == 1 && "$item_unknown_named" -ge 1 && ! -s "$calls" && ! -e "$cell/replay-out" ]]; then
    echo "dispatch-item-unknown-refused ok"
  else
    echo "dispatch-item-unknown-refused bad --item E1.9 与 E1 --item（缺项名）应当都退 2、点名项名、列出 E1 的行项 E1.1 E1.2 E1.3、不建本轮输出目录、一个装置都不起；E1.9 退 $item_unknown_exit（列项 $item_unknown_listed 行），缺项名退 $item_missing_exit，起止「$(dispatch_seen)」，输出目录 $([[ -e "$cell/replay-out" ]] && echo 建了 || echo 没建)，$(dispatch_tail)"
  fi

  # 输入缺时本次未跑（REPLAY_DEFERRED_ROW_ROOT）：样本仓里没有工作根，登记了的行要读的导出根一定不在。
  #   两行（a 没登记、deferred 登记）：deferred 记「本次未跑」、装置不起（起止记录里只有 a）、整趟退 0、REPLAY_TOTAL 带 not_run=1、dispatched=2；
  #   只有登记了的一行：整趟退 77（一项都没跑到）、一个装置都不起、REPLAY_TOTAL 带 not_run=1
  deferred_register() { # <行的身份…>：样本仓里 replay.sh 的 REPLAY_DEFERRED_ROW_ROOT 加上这几行（都要读 selftest-missing-root）
    python3 - "$root/scripts/experiment-apparatus/replay.sh" "$@" <<'PY'
import sys
path, identities = sys.argv[1], sys.argv[2:]
text = open(path, encoding="utf-8").read()
head = "declare -A REPLAY_DEFERRED_ROW_ROOT=(\n"
assert text.count(head) == 1
text = text.replace(head, head + "".join(f'  ["{identity}"]=selftest-missing-root\n' for identity in identities))
open(path, "w", encoding="utf-8").write(text)
PY
  }
  dispatch_tree dispatch-deferred-row-not-run "" "E1|e1-sample|a|$e1|exact" "E1|e1-sample|deferred|$e1|exact" || return 1
  deferred_register "E1|e1-sample|deferred" || return 1
  dispatch_env=(REPLAY_JOBS=8 TASK_SERVER_MEMORY_CAP_BYTES="$big_cap")
  dispatch_run E1
  local deferred_mixed_exit="$exit_code" deferred_mixed_started deferred_mixed_total deferred_mixed_line
  deferred_mixed_started="$(awk '$1 == "start" { printf "%s ", $2 }' "$calls")"
  deferred_mixed_total="$(grep -c '^REPLAY_TOTAL dispatched=2 .* not_run=1$' "$cell/stdout")"
  deferred_mixed_line="$(grep -c '^E1 .*本次未跑 ' "$cell/stdout")"
  dispatch_tree dispatch-deferred-row-not-run-alone "" "E1|e1-sample|deferred|$e1|exact" || return 1
  deferred_register "E1|e1-sample|deferred" || return 1
  dispatch_run E1
  if [[ "$deferred_mixed_exit" == 0 && "$deferred_mixed_started" == "a " && "$deferred_mixed_total" == 1 && "$deferred_mixed_line" == 1 ]] \
     && [[ "$exit_code" == 77 && ! -s "$calls" ]] && grep -q '^REPLAY_TOTAL dispatched=1 .* not_run=1$' "$cell/stdout"; then
    echo "dispatch-deferred-row-not-run ok"
  else
    echo "dispatch-deferred-row-not-run bad 登记成输入缺的行应当记本次未跑、装置不起、不判红：两行（a、deferred）整趟退 0、只起 a、REPLAY_TOTAL 带 not_run=1；只剩 deferred 一行整趟退 77、一个装置都不起；两行那趟退 $deferred_mixed_exit，起了「$deferred_mixed_started」，REPLAY_TOTAL 命中 $deferred_mixed_total、本次未跑行 $deferred_mixed_line；单行那趟退 $exit_code，起止「$(dispatch_seen)」，$(dispatch_tail)"
  fi
}

# 编译目录（文件头「编译目录」）的自证，replay_selftest 调：同一份临时仓（replay_selftest_sample_repository），假 cargo 编的时候把假装置拷进 CARGO_TARGET_DIR 的 release/；
# 每一格往 stdout 打一行「<格名> ok」或「<格名> bad <一句>」，每一趟之前先把 E1 的装置源码改一个字（行键变了，各格互不复用）、只跑 E1，各格一个自己的 TMPDIR（$work/tmp-<格名>，这一趟的编译目录建在它底下）
# 与自己的本轮输出目录（REPLAY_OUT=$work/out-<格名>）：
#   build-directory-removed-when-green        E1 字节一致、整趟退 0：假 cargo 拿到的 CARGO_TARGET_DIR 是 TMPDIR 底下的 singlefs-replay-build-*/target，
#                                             E1 的装置是从它的 release/ 起的，跑完 TMPDIR 底下不剩它
#   build-directory-removed-when-build-fails  假 cargo 编不过（退 101）：整趟退 2、装置没起，跑完 TMPDIR 底下不剩编译目录
#   build-directory-removed-when-stopped      E1 的装置开跑之后只给副本那一个进程（$!）发 TERM：5 秒内退 143，装置跟着停（它开着编译目录里自己那份文件），TMPDIR 底下不剩编译目录；
#                                             副本退了之后不在一个读的时刻上判：限时 5 秒，按装置的进程号等它真退出、轮询到 TMPDIR 底下不剩编译目录，两样都成了才不再等
#   caller-target-directory-kept              调用方设了 CARGO_TARGET_DIR：假 cargo 拿到的就是它、装置从它的 release/ 起，跑完它与里面的装置都在，TMPDIR 底下不另建
#   service-build-directory-kept              服务给的 CARGO_TARGET_DIR 里有上一次尝试编出的产物（撞上限重排接着用的那一份）：照它编、装置从它的 release/ 起，
#                                             跑完它、里面的装置与上一次的产物都在，TMPDIR 底下不另建
#   copy-build-directories-removed            E1 的装置在本轮输出目录里造一份副本（target/ 带 CACHEDIR.TAG，旁边一个 source/）：跑完 target/ 不在、source/ 与 E1 的产物照留
replay_build_directory_selftest() {
  local work="$1" root out exit_code=0 tmp leftover replay_pid waited settled stat_line device_pid device_state device_command
  root="$work/root"
  replay_selftest_sample_repository "$work" || return 1
  build_round() { # <格名> [环境变量赋值…]：在临时仓里只跑 E1（先把 E1 的装置源码改一个字：行键变，不读上一格记的判定），前台；输出进 $out、退出码进 exit_code、TMPDIR 底下剩的编译目录进 leftover
    tmp="$work/tmp-$1"; out="$work/out.$1"; mkdir -p "$tmp"; : >"$work/devices-ran.log"; : >"$work/cargo-calls.log"; : >"$work/cargo-calls.log.target"; : >"$work/device-paths.$1"
    printf 'fn main() { println!("one-build-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e1_sample.rs"
    if env -C "$root" -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR PATH="$work/bin:$PATH" TMPDIR="$tmp" \
         REPLAY_OUT="$work/out-$1" REPLAY_JOBS=1 REPLAY_SELFTEST_RUN_LOG="$work/devices-ran.log" REPLAY_SELFTEST_CARGO_LOG="$work/cargo-calls.log" \
         REPLAY_SELFTEST_DEVICE_PATHS="$work/device-paths.$1" GATE_NOT_RUN_FILE="$work/not-run.$1" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
         SINGLEFS_REUSE_SENTINEL_COUNT=1000 "${@:2}" bash scripts/experiment-apparatus/replay.sh E1 >"$out" 2>&1; then
      exit_code=0
    else
      exit_code=$?
    fi
    leftover="$(find "$tmp" -mindepth 1 -maxdepth 1 -name 'singlefs-replay-build-*' 2>/dev/null | tr '\n' ' ')"
  }
  build_round build-directory-removed-when-green
  if [[ "$exit_code" == 0 && -z "$leftover" ]] && grep -qE '^E1 +e1-sample +字节一致' "$out" \
     && grep -qxE "$tmp/singlefs-replay-build-[0-9]+-[A-Za-z0-9]+/target" "$work/cargo-calls.log.target" \
     && grep -qxE "$tmp/singlefs-replay-build-[0-9]+-[A-Za-z0-9]+/target/release/e1-sample" "$work/device-paths.build-directory-removed-when-green"; then
    echo "build-directory-removed-when-green ok"
  else
    echo "build-directory-removed-when-green bad 跑绿应当退 0、假 cargo 拿到的 CARGO_TARGET_DIR 与装置的路径都在 $tmp 下这一趟的编译目录里、跑完不剩它；退 $exit_code，cargo 拿到「$(sort -u "$work/cargo-calls.log.target" | tr '\n' ' ')」，装置在「$(tr '\n' ' ' <"$work/device-paths.build-directory-removed-when-green")」，剩「$leftover」，$(grep -E '^E1 ' "$out" | head -n 1)"
  fi
  build_round build-directory-removed-when-build-fails REPLAY_SELFTEST_BUILD_FAILS=1
  if [[ "$exit_code" == 2 && -z "$leftover" && ! -s "$work/devices-ran.log" ]]; then
    echo "build-directory-removed-when-build-fails ok"
  else
    echo "build-directory-removed-when-build-fails bad 编不过应当退 2、装置不起、跑完 $tmp 底下不剩这一趟的编译目录；退 $exit_code，起了「$(tr '\n' ' ' <"$work/devices-ran.log")」，剩「$leftover」，输出尾部 $(tail -n 4 "$out" | tr '\n' ' ')"
  fi
  # 被停：副本放后台起（env -C 进临时仓再 exec 成 bash，$! 就是副本那一个进程），等 E1 的装置写下它的进程号再发 TERM
  tmp="$work/tmp-build-directory-removed-when-stopped"; out="$work/out.build-directory-removed-when-stopped"; mkdir -p "$tmp"; : >"$work/devices-ran.log"
  printf 'fn main() { println!("one-build-stopped"); }\n' >"$root/scripts/experiment-apparatus/e1_sample.rs"
  env -C "$root" -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR PATH="$work/bin:$PATH" TMPDIR="$tmp" \
    REPLAY_OUT="$work/out-build-directory-removed-when-stopped" REPLAY_JOBS=1 REPLAY_SELFTEST_RUN_LOG="$work/devices-ran.log" REPLAY_SELFTEST_CARGO_LOG="$work/cargo-calls.log" \
    REPLAY_SELFTEST_DEVICE_HOLD="$work/device-hold.pid" GATE_NOT_RUN_FILE="$work/not-run.stopped" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
    SINGLEFS_REUSE_SENTINEL_COUNT=1000 bash scripts/experiment-apparatus/replay.sh E1 >"$out" 2>&1 &
  replay_pid=$!
  for _ in $(seq 150); do
    [[ -s "$work/device-hold.pid" ]] && break
    sleep 0.1
  done
  kill -TERM "$replay_pid" 2>/dev/null
  waited=0
  # 副本退了之后在 wait 之前是僵尸，kill -0 照样成功：看 /proc 里的状态
  while ((waited < 50)) && IFS= read -r stat_line 2>/dev/null <"/proc/$replay_pid/stat" && [[ "${stat_line##*) }" != Z* ]]; do
    sleep 0.1
    waited=$((waited + 1))
  done
  wait "$replay_pid"
  exit_code=$?
  # 不在一个读的时刻上判：限时 5 秒，按进程号等装置真退出（/proc 里不在、或已是僵尸；进程号被别的进程复用了的不算装置）、轮询到 TMPDIR 底下不剩编译目录，两样都成了才不再等
  device_pid="$(cat "$work/device-hold.pid" 2>/dev/null)"
  settled=0
  while :; do
    leftover="$(find "$tmp" -mindepth 1 -maxdepth 1 -name 'singlefs-replay-build-*' 2>/dev/null | tr '\n' ' ')"
    device_state=gone
    if [[ -n "$device_pid" ]] && IFS= read -r stat_line 2>/dev/null <"/proc/$device_pid/stat" && [[ "${stat_line##*) }" != Z* ]] \
       && device_command="$(tr '\0' ' ' 2>/dev/null <"/proc/$device_pid/cmdline")" && [[ "$device_command" == */release/e1-sample* ]]; then device_state=running; fi
    [[ "$device_state" == gone && -z "$leftover" ]] && break
    ((settled < 50)) || break
    sleep 0.1
    settled=$((settled + 1))
  done
  if [[ -n "$device_pid" && "$exit_code" == 143 && "$waited" -lt 50 && -z "$leftover" && "$device_state" == gone ]]; then
    echo "build-directory-removed-when-stopped ok"
  else
    echo "build-directory-removed-when-stopped bad 装置开跑之后给副本发 TERM，应当 5 秒内退 143、装置跟着停、跑完 $tmp 底下不剩这一趟的编译目录；装置进程号「$device_pid」（$device_state），退 $exit_code、等了 $((waited / 10)) 秒多，副本退了之后又等了 $((settled / 10)) 秒多（限时 5 秒），剩「$leftover」，输出尾部 $(tail -n 6 "$out" | tr '\n' ' ')"
  fi
  mkdir -p "$work/caller-target"
  build_round caller-target-directory-kept CARGO_TARGET_DIR="$work/caller-target"
  if [[ "$exit_code" == 0 && -z "$leftover" && -x "$work/caller-target/release/e1-sample" ]] && grep -qxF "$work/caller-target" "$work/cargo-calls.log.target" \
     && grep -qxF "$work/caller-target/release/e1-sample" "$work/device-paths.caller-target-directory-kept"; then
    echo "caller-target-directory-kept ok"
  else
    echo "caller-target-directory-kept bad 调用方给了 CARGO_TARGET_DIR，应当照它编、装置从它的 release/ 起、跑完它还在、不另建编译目录；退 $exit_code，cargo 拿到「$(sort -u "$work/cargo-calls.log.target" | tr '\n' ' ')」，装置在「$(tr '\n' ' ' <"$work/device-paths.caller-target-directory-kept")」，剩「$leftover」"
  fi
  # service-build-directory-kept（文件头「编译目录」）：服务给的 CARGO_TARGET_DIR 里有上一次尝试编出的产物：照它编、装置从它的 release/ 起、跑完它与上一次的产物都在、TMPDIR 底下不另建
  mkdir -p "$work/service-build/release" && echo earlier >"$work/service-build/release/earlier-attempt"
  build_round service-build-directory-kept CARGO_TARGET_DIR="$work/service-build"
  if [[ "$exit_code" == 0 && -z "$leftover" && -x "$work/service-build/release/e1-sample" && -f "$work/service-build/release/earlier-attempt" ]] \
     && grep -qxF "$work/service-build" "$work/cargo-calls.log.target" \
     && grep -qxF "$work/service-build/release/e1-sample" "$work/device-paths.service-build-directory-kept"; then
    echo "service-build-directory-kept ok"
  else
    echo "service-build-directory-kept bad 服务给了 CARGO_TARGET_DIR，应当照它编、装置从它的 release/ 起、跑完它与上一次尝试的产物都在、不另建编译目录；退 $exit_code，cargo 拿到「$(sort -u "$work/cargo-calls.log.target" | tr '\n' ' ')」，装置在「$(tr '\n' ' ' <"$work/device-paths.service-build-directory-kept")」，剩「$leftover」"
  fi
  build_round copy-build-directories-removed REPLAY_SELFTEST_COPY_TARGET=1
  if [[ "$exit_code" == 0 && -z "$leftover" && ! -e "$work/out-copy-build-directories-removed/selftest-copy/target" && -d "$work/out-copy-build-directories-removed/selftest-copy/source" \
        && -s "$work/out-copy-build-directories-removed/E1.1.out" ]]; then
    echo "copy-build-directories-removed ok"
  else
    echo "copy-build-directories-removed bad 装置在本轮输出目录里造的副本编译目录（带 CACHEDIR.TAG）跑完应当不在、副本的 source/ 与 E1 的产物照留；退 $exit_code，本轮输出目录里有「$(find "$work/out-copy-build-directories-removed" -maxdepth 3 2>/dev/null | sed "s#$work/##" | tr '\n' ' ')」"
  fi
}

#   REPLAY_BREAK=jobs-from-nproc  同时跑几条实验的缺省取核数的一半
#       jobs-default（「✗ 自检：没设 REPLAY_JOBS 时同时跑几条实验要取登记的重活默认」）
#   REPLAY_BREAK=unknown-experiment-accepted  表里没有的实验号照收
#       unknown-experiment（「✗ 自检：入口与完整性闸 unknown-experiment：」）
#   REPLAY_BREAK=deferred-rows-ignored  不看 REPLAY_DEFERRED_ROW_ROOT，登记成输入缺的行照旧起驱动
#       dispatch-deferred-row-not-run（「✗ 自检：派行 dispatch-deferred-row-not-run：」）
#   REPLAY_BREAK=item-unknown-accepted  表里没有的行项照收（点不到任何一行）
#       dispatch-item-unknown-refused（「✗ 自检：派行 dispatch-item-unknown-refused：」）
#   REPLAY_BREAK=item-row-ignored  --item 不剪行，点到的实验号整个跑
#       dispatch-item-selects-one-row（「✗ 自检：派行 dispatch-item-selects-one-row：」）
#   REPLAY_BREAK=emitted-unchecked  闸 2 不核收尾行的 emitted= 与实收条数
#       integrity-emitted（「✗ 自检：入口与完整性闸 integrity-emitted：」）
#   REPLAY_BREAK=exit-code-ignored  闸 3 不看装置的退出码
#       integrity-exit-code（「✗ 自检：入口与完整性闸 integrity-exit-code：」）
#       连带红的格（装置退非 0 照往下判，本该判跑不了的两格判不出）：「✗ 自检：留存产物取回 device-not-run-red：E1 的装置打了 NOT_RUN 行退 77，应当记跑不了（退出码 77）」、「✗ 自检：派行 row-failed-peak-not-recorded：」
#   REPLAY_BREAK=e182-transit-unset  driver_e182 与 driver_e182_units_kept 起装置时不设 E182_VERDICT_LIBRARY（装置删了缺省本地库之后退 2）
#       e182-transit 两个驱动各一格（「✗ 自检：driver_e182 起装置时要把 E182_VERDICT_LIBRARY 设成这一趟的中转目录」
#       「✗ 自检：driver_e182_units_kept 起装置时要把 E182_VERDICT_LIBRARY 设成这一趟的中转目录」）
#   REPLAY_BREAK=load-errors-unchecked  command_not_found_handle 不记找不到的命令（载入期一句注释丢了 # 只在 stderr 打一句、自证照判通过）
#       load-commands-not-found 的第二格（「✗ 自检：现调一个不存在的命令，command_not_found_handle 应当记下它的名字」）
#   REPLAY_BREAK=missing-row-driver-passes  E171 整表四行的驱动取行时库里缺行（take-rows 退 4）打了出路照往下走（e171_take_rows 返回 0，驱动接着起比对器、退 0）
#       e171-missing-row-refused（「✗ 自检：E171 整表行只核账 e171-missing-row-refused：」）
#   REPLAY_BREAK=missing-row-remedy-silent  E171 整表四行库里缺项缺行时不打出路句（e171_library_remedy 什么都不打、照返回 1）
#       e171-missing-row-refused（「✗ 自检：E171 整表行只核账 e171-missing-row-refused：」）
# 变异驱动点的变异表与单测模块配得上吗（驱动 python3 mutation_driver.py --table … 里的表，这一族是 mutation_driver.py 认 --unit-test-module 的目录）：<仓根> <读哪份脚本的驱动>。
#   一个驱动给的单测模块（没给是 mutation_driver.py 的 TEST_MODULES）顺 import 够不着的源文件上改坏的行，单测一条都不红，产物是 survived= 等于表的行数、caught=0：
#   表里每一行的原文落在的源文件，要有一份在这些模块的 import 闭包里；表头写了 --unit-test-module <模块> 的，驱动要给这个模块。原文一处都没命中的归 mutation_driver.py 自己的锚点核对（退 3），这里不判。
#   配不上的逐行打「<驱动>：…」、退 1；全配得上打「drivers=<核了几个驱动> rows=<几行>」、退 0。只读文本，不起驱动、不跑单测。弄坏开关 mutation-driver-modules-unchecked：不核、退 0、打 drivers=0。
replay_mutation_driver_table_problems() {
  if [[ "$REPLAY_BROKEN" == mutation-driver-modules-unchecked ]]; then echo "drivers=0 rows=0"; return 0; fi
  python3 -B - "$1" "$2" <<'PY'
import ast, os, re, sys

root, script = sys.argv[1:3]
with open(script, encoding="utf-8") as handle:
    script_text = handle.read()


def imports_of(path):
    with open(path, encoding="utf-8") as handle:
        tree = ast.parse(handle.read())
    named = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            named |= {alias.name.split(".")[0] for alias in node.names}
        elif isinstance(node, ast.ImportFrom) and node.module and not node.level:
            named.add(node.module.split(".")[0])
    return named


problems, drivers, rows_checked = [], 0, 0
for found in re.finditer(r"^(driver_\w+)\(\) \{\n(.*?)^\}", script_text, re.M | re.S):
    name, body = found.groups()
    if "mutation_driver.py" not in body:
        continue
    directory_named = re.search(r'cd "\$repository_root/([^"]+)"', body)
    table_named = re.search(r'--table "\$repository_root/([^"]+)"', body)
    if not directory_named or not table_named:
        problems.append(f'{name}：驱动里认不出 cd "$repository_root/<目录>" 或 --table "$repository_root/<表>"，核不了它的表与单测模块')
        continue
    directory = os.path.join(root, directory_named.group(1))
    table = os.path.join(root, table_named.group(1))
    driver_path = os.path.join(directory, "mutation_driver.py")
    if not os.path.isfile(driver_path):
        problems.append(f"{name}：{driver_path} 不在")
        continue
    with open(driver_path, encoding="utf-8") as handle:
        driver_text = handle.read()
    if "--unit-test-module" not in driver_text:
        continue   # 这个目录的驱动不认这个选项：表与单测是一一对应的一份，没有配错的余地
    if not os.path.isfile(table):
        problems.append(f"{name}：变异表 {table} 不在")
        continue
    given = re.findall(r"--unit-test-module (\S+)", body)
    default_named = re.search(r"^TEST_MODULES = (.*)$", driver_text, re.M)
    modules = given or (list(ast.literal_eval(default_named.group(1))) if default_named else [])
    module_names = {entry[:-3] for entry in os.listdir(directory) if entry.endswith(".py")}
    closure, pending = set(), list(modules)
    while pending:
        module = pending.pop()
        if module in closure or module not in module_names:
            continue
        closure.add(module)
        pending += imports_of(os.path.join(directory, module + ".py")) & module_names
    sources = sorted(entry for entry in module_names if entry != "mutation_driver" and not entry.startswith("test_"))
    source_texts = {}
    for source in sources:
        with open(os.path.join(directory, source + ".py"), encoding="utf-8") as handle:
            source_texts[source] = handle.read()
    with open(table, encoding="utf-8") as handle:
        table_lines = handle.read().splitlines()
    declared = set(re.findall(r"--unit-test-module (test_\w+)", "\n".join(line for line in table_lines if line.startswith("#"))))
    if declared - set(modules):
        problems.append(f"{name}：{table_named.group(1)} 表头写的单测模块 {sorted(declared - set(modules))} 驱动没给（驱动给的是 {modules}）")
    drivers += 1
    unreached = []
    for line in table_lines:
        if not line or line.startswith("#"):
            continue
        fields = line.split("	")
        rows_checked += 1
        anchor = fields[1].replace("\\n", "\n") if len(fields) > 1 else ""
        hit = [source for source in sources if anchor and anchor in source_texts[source]]
        if hit and not closure & set(hit):
            unreached.append(fields[0])
    if unreached:
        problems.append(f"{name}：{table_named.group(1)} 有 {len(unreached)} 行改的源文件不在单测模块 {modules} 的 import 闭包里，这些行单测不会红（头一行 {unreached[0]}）")
for problem in problems:
    print(problem)
if problems:
    sys.exit(1)
print(f"drivers={drivers} rows={rows_checked}")
PY
}
# 两格静态核（replay_selftest 的一组，一格一行「<格名> ok」或「<格名> bad <说明>」）：只读这份脚本与它点到的表和单测，不编、不跑实验
#   load-settle-awk-names  这份脚本里每处 awk -v 的变量名都不是 gawk 的内建名字，等负载那一段（replay_wait_for_load_to_settle）照 gawk 起：比值在阈值以内不睡、超了睡（sleep 换成只记一笔的假的）
#     （gawk 见到内建名字直接退 2：比值判不出来，这一段照满上限等，stderr 一句 fatal，mawk 不报，所以名字另按名单静态判，两台判的一样）
#   mutation-driver-tables  真仓这份脚本里的变异驱动都配得上；样本（两张表、两份单测）里漏了 --unit-test-module 的驱动判红、点名它，给对了的不点名，表头写的模块没给的判红
replay_static_selftest() {
  local work="$1" script_text settle_text awk_names name reserved_used=() settle_path sleep_log settle_status
  mkdir -p "$work/bin"
  script_text="$(cat -- "${REPLAY_SOURCE_FILES[@]}")"
  settle_text="$(sed -n '/^replay_wait_for_load_to_settle() {$/,/^}$/p' "${REPLAY_SOURCE_FILES[@]}")"
  if [[ "$REPLAY_BROKEN" == load-settle-builtin-name ]]; then
    script_text="${script_text//load_average/load}"
    settle_text="${settle_text//load_average/load}"
  fi
  awk_names="$(grep -E '(^|[^[:alnum:]_])awk[[:space:]]' <<<"$script_text" | grep -oE -- '-v [A-Za-z_]+=' | sed -E 's/^-v //; s/=$//' | sort -u)"
  for name in $awk_names; do
    case " BEGIN BEGINFILE END ENDFILE and asort asorti atan2 bindtextdomain break case close compl continue cos dcgettext dcngettext default delete do else eval exit exp fflush for func function gensub getline gsub if in include index int intdiv intdiv0 isarray length load log lshift match mkbool mktime namespace next nextfile or patsplit print printf rand return rshift sin split sprintf sqrt srand stopme strftime strtonum sub substr switch system systime tolower toupper typeof while xor " in
      *" $name "*) reserved_used+=("$name") ;;
    esac
  done
  printf '%s\n' '#!/usr/bin/env bash' 'echo slept >>"$REPLAY_SELFTEST_SLEEP_LOG"' >"$work/bin/sleep"
  chmod +x "$work/bin/sleep"
  settle_path="$work/bin:$PATH"
  if command -v gawk >/dev/null 2>&1; then ln -s "$(command -v gawk)" "$work/bin/awk"; fi
  sleep_log="$work/slept.log"
  : >"$sleep_log"
  # 等负载那一段在子 shell 里照 settle_text 定义一遍（弄坏开关 load-settle-builtin-name 换过名字的那一份）、export -f 交给子 bash；
  # -c 的串整个是字面的：单元闭包认得出它起什么，串里拼函数正文的折不出、会拒算。replay_selftest 在仓根调这一组，heavy-threads.sh 照仓根的相对路径 source
  (eval "$settle_text"; export -f replay_wait_for_load_to_settle
   PATH="$settle_path" REPLAY_SELFTEST_SLEEP_LOG="$sleep_log" TIMING_LOAD_RATIO=1000000 REPLAY_TIMING_SETTLE_SECONDS=1 bash -c 'source scripts/hosts/heavy-threads.sh; replay_wait_for_load_to_settle') >"$work/settle-within.out" 2>&1
  settle_status=$?
  local slept_within slept_beyond
  slept_within="$(grep -c . "$sleep_log")"
  : >"$sleep_log"
  (eval "$settle_text"; export -f replay_wait_for_load_to_settle
   PATH="$settle_path" REPLAY_SELFTEST_SLEEP_LOG="$sleep_log" TIMING_LOAD_RATIO=-1 REPLAY_TIMING_SETTLE_SECONDS=1 bash -c 'source scripts/hosts/heavy-threads.sh; replay_wait_for_load_to_settle') >"$work/settle-beyond.out" 2>&1
  slept_beyond="$(grep -c . "$sleep_log")"
  if (( ${#reserved_used[@]} == 0 )) && [[ "$settle_status" == 0 && "$slept_within" == 0 && "$slept_beyond" == 1 && ! -s "$work/settle-within.out" && ! -s "$work/settle-beyond.out" ]]; then
    echo "load-settle-awk-names ok"
  else
    echo "load-settle-awk-names bad awk -v 不许用 gawk 的内建名字（用了：${reserved_used[*]-无}），比值在阈值以内不睡、超了睡一次；实际退 $settle_status、阈值以内睡 $slept_within 次、超了睡 $slept_beyond 次，输出「$(cat "$work/settle-within.out" "$work/settle-beyond.out" | tr '\n' ' ' | clip_text 300)」"
  fi

  local sample="$work/driver-sample" real_output real_status sample_output sample_status
  mkdir -p "$sample/device"
  printf '%s\n' 'TEST_MODULES = ("test_alpha",)' '# 认 --unit-test-module' >"$sample/device/mutation_driver.py"
  printf '%s\n' 'import alpha' >"$sample/device/test_alpha.py"
  printf '%s\n' 'import beta' >"$sample/device/test_beta.py"
  printf '%s\n' 'def alpha_answer():' '    return 41' >"$sample/device/alpha.py"
  printf '%s\n' 'def beta_answer():' '    return 42' >"$sample/device/beta.py"
  printf '# 样本表\nS1 改 beta\treturn 42\treturn 43\n' >"$sample/beta.tsv"
  printf '# 样本表，经 mutation_driver.py 带 --unit-test-module test_beta 跑\nF1 改 alpha\treturn 41\treturn 40\n' >"$sample/alpha-declared.tsv"
  printf '%s\n' 'driver_sample_missing() {' \
    '  (cd "$repository_root/device" && python3 mutation_driver.py --table "$repository_root/beta.tsv" --scratch s --output o)' '}' \
    'driver_sample_given() {' \
    '  (cd "$repository_root/device" && python3 mutation_driver.py --table "$repository_root/beta.tsv" --unit-test-module test_beta --scratch s --output o)' '}' \
    'driver_sample_declared_other() {' \
    '  (cd "$repository_root/device" && python3 mutation_driver.py --table "$repository_root/alpha-declared.tsv" --unit-test-module test_alpha --scratch s --output o)' '}' >"$sample/script.sh"
  if real_output="$(replay_mutation_driver_table_problems "$PWD" "$REPLAY_ENTRY_SCRIPT" 2>&1)"; then real_status=0; else real_status=$?; fi
  if sample_output="$(replay_mutation_driver_table_problems "$sample" "$sample/script.sh" 2>&1)"; then sample_status=0; else sample_status=$?; fi
  if [[ "$real_status" == 0 && "$sample_status" == 1 && "$real_output" =~ ^drivers=[1-9] ]] \
     && grep -q '^driver_sample_missing：beta.tsv 有 1 行改的源文件不在单测模块' <<<"$sample_output" \
     && grep -q '^driver_sample_declared_other：alpha-declared.tsv 表头写的单测模块' <<<"$sample_output" \
     && ! grep -q '^driver_sample_given：' <<<"$sample_output"; then
    echo "mutation-driver-tables ok"
  else
    echo "mutation-driver-tables bad 真仓的变异驱动要都配得上、样本里漏 --unit-test-module 与表头写的模块没给的两个驱动要判红、给对了的不点名；真仓退 $real_status「$(tr '\n' ' ' <<<"$real_output" | clip_text 400)」，样本退 $sample_status「$(tr '\n' ' ' <<<"$sample_output" | clip_text 400)」"
  fi
}
# E171 整表那四行只核账的一格（replay_selftest 的一组，一格一行「<格名> ok」或「<格名> bad <说明>」）：
#   e171-missing-row-refused  样本仓（replay_selftest_sample_repository：假 cargo 记下每一次起它）里放假的取行程序 shared_steps.py（take 照取到；take-rows 点到
#     REPLAY_SELFTEST_MISSING_ROWS_ROOT 那张表退 4、别的表照取到）、假比对器 comparator.py 与假的 scripts/hosts/minimal-copy.py（只建目录），三样每起一回往
#     REPLAY_SELFTEST_RUN_LOG 记一笔（拷副本的不记）；从本文件取出四个驱动与它们用的函数，在样本仓里各起一次，每个驱动缺的是它自己那张表的行
#     （F4、补位两行先把前面的段记成这一趟做完：driver_e171_s4_comparator 照记号直接交它留下的产物）：都要退非 0、stderr 有分片入口的整条命令（带范围声明）、
#     stdout 不打，记录里只有取行程序那几笔（共用前置一笔、缺的那张表一笔；F4、补位两行只有缺的那张表一笔），假 cargo 一次没起
#   REPLAY_BREAK 在这一格里只认 missing-row-driver-passes、missing-row-remedy-silent 两条，别的开关打开时这一格照清空开关跑（不跟着红）
replay_e171_library_selftest() {
  local work="$1" root tools definitions e171_case driver rest missing stages out status expected remedy_line problems=""
  if ! replay_selftest_sample_repository "$work" >"$work.sample.log" 2>&1; then
    echo "e171-missing-row-refused bad 样本仓搭不起来（replay_selftest_sample_repository 退非 0）：$(tail -n 3 "$work.sample.log" | tr '\n' ' ' | clip_text 300)"
    return 0
  fi
  root="$work/root"
  tools="$root/scripts/experiment-apparatus/e171_three_oracle_collusion"
  mkdir -p "$tools" || return 1
  cat >"$tools/shared_steps.py" <<'FAKE_TAKE'
import os
import sys

arguments = sys.argv[1:]
command = arguments[0] if arguments else "-"
output_root = os.path.basename(arguments[arguments.index("--output-root") + 1]) if "--output-root" in arguments else "-"
with open(os.environ["REPLAY_SELFTEST_RUN_LOG"], "a", encoding="utf-8") as log:
    log.write(f"shared_steps {command} {output_root}\n")
if command == "take-rows" and output_root == os.environ.get("REPLAY_SELFTEST_MISSING_ROWS_ROOT"):
    print(f"row=5 reason=库里没有这把键 output_root={output_root}", file=sys.stderr)
    sys.exit(4)
if command == "take-rows":
    print("table lanes=0 selftest")
FAKE_TAKE
  cat >"$tools/comparator.py" <<'FAKE_COMPARATOR'
import os
import sys

arguments = sys.argv[1:]
with open(os.environ["REPLAY_SELFTEST_RUN_LOG"], "a", encoding="utf-8") as log:
    log.write("comparator\n")
if "--print-q3-core-rows" in arguments:
    print("q3_core_rows=8")
elif "--output" in arguments:
    with open(arguments[arguments.index("--output") + 1], "w", encoding="utf-8") as handle:
        handle.write("E7RESULT name=selftest-comparator\n")
FAKE_COMPARATOR
  printf '%s\n' 'import os' 'import sys' 'os.makedirs(sys.argv[2], exist_ok=True)' >"$root/scripts/hosts/minimal-copy.py"
  # 驱动定义在这个函数被调用之后才读到：从本文件里取出来，在子 shell 里 eval（同「驱动开跑前核输入」那几格）
  definitions="$(awk '
    /^E171_FREEZE_REFERENCE=/ { print; next }
    /^e171_s4_stage_(done|mark)\(\) \{/ { print; next }
    /^(e171_library_remedy|e171_take_shared_steps|e171_take_rows|driver_e171_s2_comparator|driver_e171_s4_comparator|driver_e171_s4_f4_comparator|driver_e171_s4_f4_substitutes_comparator)\(\) \{/ { inside = 1 }
    inside { print; if ($0 ~ /^\}/) inside = 0 }' "$REPLAY_ENTRY_SCRIPT")"
  # <驱动>:<缺行的那张表（输出根的名字）>:<先记成做完的段>
  for e171_case in driver_e171_s2_comparator:rows-s2-trial: driver_e171_s4_comparator:rows-s4: \
    driver_e171_s4_f4_comparator:rows-s4-f4:s4 driver_e171_s4_f4_substitutes_comparator:rows-s4-f4-substitutes:s4,f4; do
    driver="${e171_case%%:*}"; rest="${e171_case#*:}"; missing="${rest%%:*}"; stages="${rest#*:}"
    out="$work/out-$driver"
    mkdir -p "$out" || return 1
    if (
      cd "$root" || exit 2
      eval "$definitions" || exit 2
      case "$REPLAY_BROKEN" in missing-row-driver-passes|missing-row-remedy-silent) ;; *) REPLAY_BROKEN= ;; esac
      OUT_DIR="$out"
      CHECKER_TIER_FEATURES=()   # 驱动里点到这个数组；这一格不开 GPU，给空数组（没设时 set -u 下驱动第一行就退）
      export PATH="$work/bin:$PATH" REPLAY_SELFTEST_RUN_LOG="$out.run.log" REPLAY_SELFTEST_CARGO_LOG="$out.cargo.log" REPLAY_SELFTEST_MISSING_ROWS_ROOT="$missing"
      : >"$E171_FREEZE_REFERENCE" && mkdir -p "$OUT_DIR/e171-s4-comparator" || exit 2
      printf 'E7RESULT name=selftest-s4\n' >"$OUT_DIR/e171-s4-comparator/comparator.out"
      printf 'E7RESULT name=selftest-s4-f4\n' >"$OUT_DIR/e171-s4-comparator/comparator-f4.out"
      for stage in ${stages//,/ }; do e171_s4_stage_mark "$OUT_DIR/e171-s4-comparator" "$stage" || exit 2; done
      "$driver"
    ) >"$out.stdout" 2>"$out.stderr"; then status=0; else status=$?; fi
    expected="shared_steps take-rows $missing"
    [[ -n "$stages" ]] || expected="shared_steps take -"$'\n'"$expected"
    remedy_line="$(grep -F -- "task-server/task submit --cwd $root -- bash scripts/experiment-apparatus/e171-row-shard.sh --root " "$out.stderr" | head -n 1)"
    if [[ "$status" == 0 || "$(cat -- "$out.run.log" 2>/dev/null)" != "$expected" || -s "$out.cargo.log" || -s "$out.stdout" \
          || "$remedy_line" != *"SINGLEFS_RUN_SCOPE='种类=whole; 跑什么="*"; 范围="*"; 为什么="* ]]; then
      problems+="${driver} 退 $status、出路句「${remedy_line:-没有}」、记录「$(tr '\n' ' ' <"$out.run.log" 2>/dev/null)」（要「$(tr '\n' ' ' <<<"$expected")」）、cargo「$(tr '\n' ' ' <"$out.cargo.log" 2>/dev/null)」、stdout「$(tr '\n' ' ' <"$out.stdout")」、stderr「$(tr '\n' ' ' <"$out.stderr" | clip_text 200)」；"
    fi
  done
  if [[ -z "$problems" ]]; then
    echo "e171-missing-row-refused ok"
  else
    echo "e171-missing-row-refused bad 库里缺一行时四个驱动都要退非 0、stderr 打带范围声明的分片入口整条命令、stdout 不打、只起取行程序：$(clip_text 1500 <<<"$problems")"
  fi
}
# 递归删目录前的守卫（目标是几段变量拼出来的，如「$OUT_DIR/名字」，其中一段为空就删到别处，「${变量:?}」只挡整串为空）：
# <根> 非空，且每个 <目标> 都严格落在 <根>/ 底下（比「<根>/」长），才删；不满足的什么都不删，退 1、stderr 点名。
replay_remove_directory_below() { # <根> <目标>…
  local root_directory="${1:-}" directory_to_remove
  if [[ -z "$root_directory" ]]; then
    echo "  ✗ replay_remove_directory_below：根是空的，不删（目标 ${2:-}）" >&2
    echo "     → 怎么办：调用方先把根（\$OUT_DIR 这类）定下来再删" >&2
    return 1
  fi
  shift
  for directory_to_remove in "$@"; do
    if [[ "$directory_to_remove" != "$root_directory"/?* ]]; then
      echo "  ✗ replay_remove_directory_below：「$directory_to_remove」不在「$root_directory」底下，不删" >&2
      echo "     → 怎么办：目标由几段变量拼成，查哪一段是空的或指到了别处" >&2
      return 1
    fi
    rm -rf -- "${directory_to_remove:?}"
  done
}
replay_selftest() {
  local scratch failures=0 checked=0 status problems_last_line
  # 载入期找不到的命令（command_not_found_handle 记的），在自证别的步骤跑之前先存一份：load-commands-not-found 那一格判它
  local load_commands_not_found=()
  mapfile -t load_commands_not_found < <(replay_commands_not_found)
  # 按行复用、留存产物取回、.gz、编译目录那几组起的副本都读写判定库：只在临时 prune-server 上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」
  # --with-temporary-service），不是临时服务就一格都不跑、退 2，不写真服务（判据只在客户端的 selftest_service_problem 一处）
  local service_problem
  if ! service_problem="$(python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem())' "$PWD/scripts/prune-server" 2>&1)" \
     || [[ -n "$service_problem" ]]; then
    echo "  ✗ replay.sh 自检没跑：${service_problem:-载入 scripts/prune-server/prune_server_client.py 没成}"
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）"
    return 2
  fi
  # 服务交给这一条自证的编译目录（CARGO_TARGET_DIR）与调用方的卡串（文件头「编译目录」「GPU」）不传给自证起的副本：要它们的那几格各自在环境里给
  unset CARGO_TARGET_DIR GPU_CARDS_BY_PRIORITY
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/replay-selftest-XXXXXX")"
  # 中途报错、被打断也删（成功、失败都删；local 变量到退出时已不在，路径在这里就展开）
  trap "rm -rf -- '$scratch'" EXIT
  mkdir -p "$scratch/bin"
  # 自证起的子进程的标记（文件头附近那一道拒）：自证里起的每一份 replay.sh 都继承它，平常跑设了 REPLAY_BREAK 的没有
  printf '%s\n' replay-selftest >"$scratch/mark"
  export REPLAY_SELFTEST_MARK="$scratch/mark"
  # 平常跑设了弄坏开关（门禁 code-tooling 的 break-switch-outside-selftest 格就这样探）：标记清掉，要退 2 并点名变量只在自证里认；guard-off 下这一道不拒，探针往下走到「复跑表里没有」退 2，不点名
  checked=$((checked + 1))
  local guard_probe_break=thread-registry-not-copied guard_probe_output guard_probe_status
  [[ "$REPLAY_BROKEN" != guard-off ]] || guard_probe_break=guard-off
  # 探针带一个表里没有的实验号 break-guard-probe-unknown-experiment（不以 - 起头）：闭包按这个字面实验号收复跑表的行（一行都不收）；只带选项就连整张复跑表。
  # 守卫在读参数之前就拒，带不带它退出码与拒绝说明都一样；guard-off 下它与那个选项一起照「复跑表里没有」退 2
  if guard_probe_output="$(env REPLAY_SELFTEST_MARK= REPLAY_BREAK="$guard_probe_break" bash scripts/experiment-apparatus/replay.sh --force --break-guard-probe-unknown-option break-guard-probe-unknown-experiment 2>&1 </dev/null)"; then guard_probe_status=0; else guard_probe_status=$?; fi
  if [[ "$guard_probe_status" != 2 ]] || ! grep -qE 'REPLAY_BREAK.*(自证|selftest)' <<<"$guard_probe_output"; then
    echo "  ✗ 自检：平常跑设了弄坏开关应当退 2 并点名 REPLAY_BREAK 只在自证里认；实际退 $guard_probe_status：$(head -n 3 <<<"$guard_probe_output" | tr '\n' ' ')"
    echo "     → 怎么办：看文件头附近那一道拒（REPLAY_BREAK 非空而没有自证的标记就退 2）；REPLAY_BREAK=guard-off 设着的话这里本来就该红"
    failures=$((failures + 1))
  fi
  printf '%s\n' '#!/usr/bin/env bash' 'echo "   Compiling e7-index-bench v0.1.0"' \
    'if [[ "${FAKE_CARGO_OUTCOME:-}" == red ]]; then echo "error[E0425]: cannot find value \`missing\` in this scope"; echo "error: could not compile \`e7-index-bench\`"; exit 101; fi' \
    'echo "    Finished release"; exit 0' >"$scratch/bin/cargo"
  chmod +x "$scratch/bin/cargo"
  checked=$((checked + 1))
  if (PATH="$scratch/bin:$PATH" FAKE_CARGO_OUTCOME=red build_research_binaries "$scratch/build-red.log") 2>"$scratch/build-red.stderr"; then status=0; else status=$?; fi
  if [[ "$status" != 2 ]] || ! grep -qF "第一处错：error[E0425]: cannot find value" "$scratch/build-red.stderr" \
     || ! grep -qF "    error: could not compile" "$scratch/build-red.stderr" || ! grep -qF "整份构建日志：$scratch/build-red.log" "$scratch/build-red.stderr" \
     || ! grep -qF "单独重现：(cd $PWD && cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml)" "$scratch/build-red.stderr" \
     || [[ ! -s "$scratch/build-red.log" ]]; then
    echo "  ✗ 自检：构建失败时应当返回 2、打第一处错、日志末尾、日志路径与重现命令，日志要落盘；实际返回 $status：$(cat "$scratch/build-red.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # build-two-logs：构建日志出两份（全量 build-red.log，旁边同名的 build-red.problems.log 带行号收 error 行、末行写退出码 101），stderr 点名错误警告日志的路径
  checked=$((checked + 1))
  problems_last_line="$(tail -n 1 "$scratch/build-red.problems.log" 2>/dev/null)"   # 管道收尾的 grep -q 在 pipefail 下会把命中读成没命中：先落到变量
  if [[ ! -s "$scratch/build-red.problems.log" ]] || ! grep -qE '^[0-9]+:error\[E0425\]: cannot find value' "$scratch/build-red.problems.log" \
     || ! grep -qE '^[0-9]+:error: could not compile' "$scratch/build-red.problems.log" || ! grep -qF '被包的命令退出码 101' <<<"$problems_last_line" \
     || ! grep -qF "错误警告日志：$scratch/build-red.problems.log" "$scratch/build-red.stderr"; then
    echo "  ✗ 自检：构建失败时应当出两份日志（build-red.log 全量；build-red.problems.log 带行号收两条 error、末行写退出码 101），并在 stderr 点名错误警告日志的路径；实际 $(ls "$scratch" | grep '^build-red' | tr '\n' ' ')：$(cat "$scratch/build-red.problems.log" 2>&1 | tr '\n' '|')"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  if (PATH="$scratch/bin:$PATH" FAKE_CARGO_OUTCOME=green build_research_binaries "$scratch/build-green.log") 2>"$scratch/build-green.stderr"; then status=0; else status=$?; fi
  if [[ "$status" != 0 || -s "$scratch/build-green.stderr" ]]; then
    echo "  ✗ 自检：构建成功时应当返回 0、一个字都不打；实际返回 $status：$(cat "$scratch/build-green.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  problems_last_line="$(tail -n 1 "$scratch/build-green.problems.log" 2>/dev/null)"
  if ! grep -qF '收了 0 行' <<<"$problems_last_line"; then
    echo "  ✗ 自检：构建成功时错误警告日志也要在、末行写收了 0 行（build-green.problems.log）；实际：$(cat "$scratch/build-green.problems.log" 2>&1 | tr '\n' '|')"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 驱动开跑前编研究二进制（build_research_driver_binary）：假 cargo 记收到的参数、照仓根编那种报错退 101；cargo 的输出进 stderr（这一行的 .err），stdout 一个字不打
  mkdir -p "$scratch/driver-bin"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*" >>"${FAKE_CARGO_LOG:?}"' 'echo "error: cannot specify features for packages outside of workspace"' 'exit 101' >"$scratch/driver-bin/cargo"
  chmod +x "$scratch/driver-bin/cargo"
  : >"$scratch/driver-cargo.log"
  checked=$((checked + 1))
  if (PATH="$scratch/driver-bin:$PATH" FAKE_CARGO_LOG="$scratch/driver-cargo.log" build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return) \
       >"$scratch/driver-build.stdout" 2>"$scratch/driver-build.err"; then status=0; else status=$?; fi
  if [[ "$status" != 1 || -s "$scratch/driver-build.stdout" ]] || ! grep -qF 'error: cannot specify features for packages outside of workspace' "$scratch/driver-build.err" \
     || ! grep -qxF 'build -q --release --manifest-path scripts/experiment-apparatus/Cargo.toml --features e174-gpu --bin e174-lease-drop-wait-memory-return' "$scratch/driver-cargo.log"; then
    echo "  ✗ 自检：驱动开跑前编研究二进制（build_research_driver_binary）要经 --manifest-path scripts/experiment-apparatus/Cargo.toml 编、编不过返回 1、cargo 的输出进 stderr（这一行的 .err 非空）、stdout 不打；实际返回 $status，cargo 收到「$(tr '\n' '|' <"$scratch/driver-cargo.log")」，stderr「$(tr '\n' ' ' <"$scratch/driver-build.err" | clip_text 300)」，stdout「$(tr '\n' ' ' <"$scratch/driver-build.stdout" | clip_text 200)」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # E162、E163 那三个驱动同样经它编：起驱动本身，假 cargo 另记 BINDGEN_EXTRA_CLANG_ARGS（E162 两个要把它设进编那一步），编不过就停在编那一步。
  # 驱动定义在这个函数被调用之后才读到，这里从本文件里取出那三段定义、在子 shell 里 eval
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*|BINDGEN=${BINDGEN_EXTRA_CLANG_ARGS-没设}" >>"${FAKE_CARGO_LOG:?}"' 'echo "error: cannot specify features for packages outside of workspace"' 'exit 101' >"$scratch/driver-bin/cargo"
  local driver_case driver_name driver_expected driver_definitions
  driver_definitions="$(awk '/^(driver_e162_anchors|driver_e162_power_cut_selftest|driver_e163_r1_merge)\(\) \{/,/^}/' "$REPLAY_ENTRY_SCRIPT")"
  checked=$((checked + 1))
  if [[ "$(grep -cE '^driver_e16[23]_[a-z0-9_]+\(\) \{' <<<"$driver_definitions")" != 3 ]]; then
    echo "  ✗ 自检：从 $REPLAY_ENTRY_SCRIPT 里取不全 driver_e162_anchors、driver_e162_power_cut_selftest、driver_e163_r1_merge 三段定义（取到的头几行：$(head -n 3 <<<"$driver_definitions" | tr '\n' '|')）"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  for driver_case in \
    "driver_e162_anchors|build -q --release --manifest-path scripts/experiment-apparatus/Cargo.toml --features e162-block-stores --bin e162-crash-verdict-block-store|BINDGEN=-I/usr/lib/gcc/x86_64-linux-gnu/13/include" \
    "driver_e162_power_cut_selftest|build -q --release --manifest-path scripts/experiment-apparatus/Cargo.toml --features e162-block-stores --bin e162-verdict-store-power-cut|BINDGEN=-I/usr/lib/gcc/x86_64-linux-gnu/13/include" \
    "driver_e163_r1_merge|build -q --release --manifest-path scripts/experiment-apparatus/Cargo.toml --features e163-gpu --bin e163-gpu-multicard-crc32c|BINDGEN=没设"; do
    driver_name="${driver_case%%|*}"; driver_expected="${driver_case#*|}"
    : >"$scratch/driver-cargo.log"
    checked=$((checked + 1))
    if (unset BINDGEN_EXTRA_CLANG_ARGS; OUT_DIR="$scratch"; eval "$driver_definitions"; PATH="$scratch/driver-bin:$PATH" FAKE_CARGO_LOG="$scratch/driver-cargo.log" "$driver_name") \
         >"$scratch/driver-build.stdout" 2>"$scratch/driver-build.err"; then status=0; else status=$?; fi
    if [[ "$status" != 1 || -s "$scratch/driver-build.stdout" ]] || ! grep -qF 'error: cannot specify features for packages outside of workspace' "$scratch/driver-build.err" \
       || [[ "$(cat "$scratch/driver-cargo.log")" != "$driver_expected" ]]; then
      echo "  ✗ 自检：$driver_name 开跑前要经 build_research_driver_binary 编（cargo 只起一次：$driver_expected）、编不过返回 1、cargo 的输出进 .err、stdout 不打；实际返回 $status，cargo 收到「$(tr '\n' '|' <"$scratch/driver-cargo.log")」，stderr「$(tr '\n' ' ' <"$scratch/driver-build.err" | clip_text 300)」，stdout「$(tr '\n' ' ' <"$scratch/driver-build.stdout" | clip_text 200)」"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done
  # E182 两个驱动（文件头「E182 的片结果库」）：起装置时 E182_VERDICT_LIBRARY 设成这一趟中转目录里各自那一种；从本文件取出两段定义、在子 shell 里 eval，假 cargo 记收到的值
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*|E182_VERDICT_LIBRARY=${E182_VERDICT_LIBRARY-没设}" >>"${FAKE_CARGO_LOG:?}"' 'exit 0' >"$scratch/driver-bin/cargo"
  # 驱动名写成 for 的字面词（不从带 $scratch 的串里切）：单元闭包把下面命令位置上的 "$driver_name" 折成这几个驱动函数（lib_function_units.py 文件头「起代码的写法逐种认」），
  # 带 $scratch 的串折不出，同一个函数里两处 "$driver_name" 都会拒算
  local e182_definitions e182_mode
  e182_definitions="$(awk '/^(driver_e182|driver_e182_units_kept)\(\) \{/,/^}/' "$REPLAY_ENTRY_SCRIPT")"
  for driver_name in driver_e182 driver_e182_units_kept; do
    case "$driver_name" in
      driver_e182) e182_mode=single-device ;;
      driver_e182_units_kept) e182_mode=single-device-units-kept ;;
    esac
    driver_expected="run -q --release -p singlefs-implementation-experiments --bin e182_unreadable_abandoned_root_forks -- $e182_mode|E182_VERDICT_LIBRARY=$scratch/e182-shards/e182-$e182_mode"
    : >"$scratch/driver-cargo.log"
    checked=$((checked + 1))
    (unset E182_VERDICT_LIBRARY; CHECKER_TIER_FEATURES=(); REPLAY_E182_SHARDS="$scratch/e182-shards"; eval "$e182_definitions"
     PATH="$scratch/driver-bin:$PATH" FAKE_CARGO_LOG="$scratch/driver-cargo.log" "$driver_name") >/dev/null 2>&1
    if [[ "$(cat "$scratch/driver-cargo.log")" != "$driver_expected" ]]; then
      echo "  ✗ 自检：$driver_name 起装置时要把 E182_VERDICT_LIBRARY 设成这一趟的中转目录（cargo 只起一次：$driver_expected）；实际 cargo 收到「$(tr '\n' '|' <"$scratch/driver-cargo.log")」"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done
  checked=$((checked + 1))
  local -A ADMISSION_ROW_KEY=(["E1|@driver_e1|"]="E1" ["E1|@driver_e1_second|"]="E1/second")
  local ROW_ADMISSION_KEY=""
  admission_key_of_row E1 @driver_e1_second ""
  if [[ "$ROW_ADMISSION_KEY" != E1/second ]]; then
    echo "  ✗ 自检：复跑表那一行指明了准入键 E1/second，问准入就该用它；实际用的是「$ROW_ADMISSION_KEY」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 配置开了 GPU 时 验证包装置带 gpu 特性编（resolve_checker_tier_features）：假 cargo 把收到的参数记进 gpu-calls.log，配置文件放临时目录
  mkdir -p "$scratch/gpu-bin"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "cargo $*" >>"${FAKE_CARGO_LOG:?}"' \
    'if [[ "${FAKE_CARGO_OUTCOME:-}" == red ]]; then echo "error[E0432]: unresolved import wgpu"; exit 101; fi' 'exit 0' >"$scratch/gpu-bin/cargo"
  chmod +x "$scratch/gpu-bin/cargo"
  gpu_probe() { # <配置文件路径> <假 cargo 的结局>：子 shell 里跑一遍，打「状态 特性数组」一行；stderr 进 gpu-probe.stderr，cargo 的调用进 gpu-calls.log
    : >"$scratch/gpu-calls.log"
    ( export SINGLEFS_MULTI_HOST_CONFIG="$1" FAKE_CARGO_LOG="$scratch/gpu-calls.log" FAKE_CARGO_OUTCOME="$2" PATH="$scratch/gpu-bin:$PATH" \
        QUIESCE_CALLS="$scratch/quiesce-calls.log"
      resolve_checker_tier_features "$scratch/gpu-probe.log" 2>"$scratch/gpu-probe.stderr"; probe_status=$?
      echo "$probe_status ${CHECKER_TIER_FEATURES[*]:-}" )
  }
  local gpu_reading
  # 开了 GPU 的多机配置：清场命令是假的（往 $QUIESCE_CALLS 记一笔）；本脚本不另套清场，这一笔不该有
  printf '%s\n' '# 自证的多机配置' 'ENABLE_GPU=1' \
    "QUIESCE_STOP_COMMAND='echo quiesce-stop >> \"\$QUIESCE_CALLS\"'" 'QUIESCE_STOPPED_CHECK_COMMAND=true' >"$scratch/gpu-on.env"
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-on.env" green)"
  if [[ "$gpu_reading" != "0 --features gpu" ]] \
     || ! grep -qxF "cargo build --release -p singlefs-verification --features gpu --lib" "$scratch/gpu-calls.log"; then
    echo "  ✗ 自检：配置写了 ENABLE_GPU=1，应当先试编 gpu 特性并把「--features gpu」放进 CHECKER_TIER_FEATURES；实际「$gpu_reading」，cargo 调用：$(tr '\n' '|' <"$scratch/gpu-calls.log")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  printf '%s\n' 'ENABLE_GPU=0' >"$scratch/gpu-off.env"
  printf '%s\n' 'ENABLE_ACROSS_MACHINES=1' >"$scratch/gpu-unset.env"
  checked=$((checked + 1))
  if [[ "$(gpu_probe "$scratch/gpu-off.env" green)" != "0 " || -s "$scratch/gpu-calls.log" || "$(gpu_probe "$scratch/gpu-unset.env" green)" != "0 " || -s "$scratch/gpu-calls.log" ]]; then
    echo "  ✗ 自检：配置写 ENABLE_GPU=0、或没写这个键，应当不带特性、一次 cargo 都不起；实际 $(gpu_probe "$scratch/gpu-off.env" green)，cargo 调用：$(tr '\n' '|' <"$scratch/gpu-calls.log")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  printf '%s\n' 'ENABLE_GPU=yes' >"$scratch/gpu-garbled.env"
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-garbled.env" green)"
  if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "ENABLE_GPU 写成了「yes」" "$scratch/gpu-probe.stderr" || [[ -s "$scratch/gpu-calls.log" ]]; then
    echo "  ✗ 自检：ENABLE_GPU 写了 0、1 之外的值，应当退 78、点名那个值、不起 cargo、不悄悄不带特性；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-no-such.env" green)"
  if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "点名的多机配置 $scratch/gpu-no-such.env 不在" "$scratch/gpu-probe.stderr"; then
    echo "  ✗ 自检：SINGLEFS_MULTI_HOST_CONFIG 点名的文件不在，应当退 78 并点名那个文件；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  if (( EUID != 0 )); then
    cp -- "$scratch/gpu-on.env" "$scratch/gpu-unreadable.env"; chmod 000 "$scratch/gpu-unreadable.env"
    checked=$((checked + 1))
    gpu_reading="$(gpu_probe "$scratch/gpu-unreadable.env" green)"
    if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "gpu-unreadable.env 不可读" "$scratch/gpu-probe.stderr" || [[ -s "$scratch/gpu-calls.log" ]]; then
      echo "  ✗ 自检：配置文件不可读，应当退 78、不起 cargo（不当成没开 GPU）；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  fi
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-on.env" red)"
  if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "第一处错：error[E0432]: unresolved import wgpu" "$scratch/gpu-probe.stderr" \
     || ! grep -qF "不退回不带 gpu 特性去跑" "$scratch/gpu-probe.stderr" || ! grep -qF "整份编译日志：$scratch/gpu-probe.log" "$scratch/gpu-probe.stderr"; then
    echo "  ✗ 自检：开了 GPU 而 gpu 特性编不过，应当退 78、打第一处错与日志路径、不带特性数组；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # gpu-probe-takes-no-cards（文件头「GPU」：不领卡）：开了 GPU、环境里没有卡串，resolve_checker_tier_features 返回 0，
  #   照装置那样直接起的一条命令看到的是「cards=unset」（不领卡、不设卡串）；多机配置里的清场命令是假的、往 quiesce-calls.log 记一笔（不另套清场，一笔都不该有）
  local cards_reading
  checked=$((checked + 1))
  : >"$scratch/gpu-calls.log"; : >"$scratch/quiesce-calls.log"
  cards_reading="$( ( export SINGLEFS_MULTI_HOST_CONFIG="$scratch/gpu-on.env" FAKE_CARGO_LOG="$scratch/gpu-calls.log" FAKE_CARGO_OUTCOME=green PATH="$scratch/gpu-bin:$PATH" \
                      QUIESCE_CALLS="$scratch/quiesce-calls.log"
                    unset GPU_CARDS_BY_PRIORITY
                    resolve_checker_tier_features "$scratch/gpu-probe.log" 2>"$scratch/gpu-probe.stderr"; probe_status=$?
                    echo "resolve=$probe_status"
                    if (( probe_status == 0 )); then bash -c 'echo "cards=${GPU_CARDS_BY_PRIORITY-unset}"'; fi ) | tr '\n' ' ')"
  if [[ "$cards_reading" != "resolve=0 cards=unset " || -s "$scratch/quiesce-calls.log" ]]; then
    echo "  ✗ 自检：开了 GPU、环境里没有卡串，应当返回 0、照装置那样起的命令看不到卡（不领卡）、一条清场命令都不跑；实际「$cards_reading」，假清场命令记了「$(tr '\n' '|' <"$scratch/quiesce-calls.log")」：$(tail -n 3 "$scratch/gpu-probe.stderr" | tr '\n' ' ')"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 起 实验包装置的 cargo run 行（用到给定状态引擎的那几个装置）每一行都带特性数组：新加一行忘了带，开了 GPU 时那一行拿不到判段结论
  local engine_lines unmarked_lines wrapper_name wrapped_lines
  engine_lines="$(grep -hE 'cargo run .*-p singlefs-implementation-experiments .*--bin (e156_|e158_|e161_|e168_|e170_|e171_|"\$bin")' -- "${REPLAY_SOURCE_FILES[@]}" | grep -vE '^[[:space:]]*#' || true)"
  if [[ "$REPLAY_BROKEN" == run-lines-without-features ]]; then engine_lines="${engine_lines//CHECKER_TIER_FEATURES/}"; fi
  unmarked_lines="$(grep -vF 'CHECKER_TIER_FEATURES' <<<"$engine_lines" || true)"
  checked=$((checked + 1))
  if [[ -z "$engine_lines" || -n "$unmarked_lines" ]]; then
    echo "  ✗ 自检：起 实验包装置的 cargo run 行（共 $(grep -c . <<<"$engine_lines") 行）应当每一行都带 \${CHECKER_TIER_FEATURES[@]…}；缺的：$(head -n 3 <<<"$unmarked_lines" | clip_text 200)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 起 实验包装置不另套清场（清场与分卡归统一任务服务，任务里面再调清场包装由包装自己拒）：同一批行没有一行带清场包装或清场前缀，
  #   本文件不是注释的行里也不起清场包装（包装名拆开写，这一格自己的行不算命中）
  wrapper_name="gpu-quiesce-run"'.sh'
  if [[ "$REPLAY_BROKEN" == run-line-quiesce-wrapped ]]; then
    engine_lines+=$'\n'"bash scripts/crash/$wrapper_name -- cargo ""run -q --release -p singlefs-implementation-experiments --bin e156_allocation_basis_counts"   # cargo 与 run 拆开写：不让上面那一格扫装置行时把这一行当成装置行
  fi
  wrapped_lines="$( { grep -hvE '^[[:space:]]*#' -- "${REPLAY_SOURCE_FILES[@]}" | grep -F "$wrapper_name"; grep -E "QUIESCE|$wrapper_name" <<<"$engine_lines"; } || true)"
  checked=$((checked + 1))
  if [[ -z "$engine_lines" || -n "$wrapped_lines" ]]; then
    echo "  ✗ 自检：起 实验包装置不另套清场：cargo run 行（共 $(grep -c . <<<"$engine_lines") 行）不该带清场前缀，本文件不是注释的行里也不该起 $wrapper_name；命中的：$(head -n 3 <<<"$wrapped_lines" | clip_text 200)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 拷 crates/ 去编译的副本带上每台默认重活线程数的登记文件（copy_thread_registry_into）：拷进去的与仓里那一份逐字节相同
  checked=$((checked + 1))
  copy_thread_registry_into "$scratch/registry-tree"
  if ! cmp -s -- .claude/heavy-thread-default "$scratch/registry-tree/.claude/heavy-thread-default"; then
    echo "  ✗ 自检：拷 crates/ 去编译的副本要带上每台默认重活线程数的登记文件 .claude/heavy-thread-default（harness 经 include_str! 读它），副本里实际有「$(ls -A "$scratch/registry-tree/.claude" 2>/dev/null | tr '\n' ' ')」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 拷 crates 的每一行（「Cargo.lock .cargo crates」那几行）都在同一行里接着 copy_thread_registry_into：新加一行忘了带，那个副本编不过 harness
  local copy_lines unregistered_copy_lines
  copy_lines="$(grep -hE 'Cargo\.lock \.cargo crates' -- "${REPLAY_SOURCE_FILES[@]}" | grep -vE '^[[:space:]]*#' || true)"
  if [[ "$REPLAY_BROKEN" == copy-lines-without-registry ]]; then copy_lines="${copy_lines//copy_thread_registry_into/}"; fi
  unregistered_copy_lines="$(grep -vF 'copy_thread_registry_into' <<<"$copy_lines" || true)"
  checked=$((checked + 1))
  if [[ -z "$copy_lines" || -n "$unregistered_copy_lines" ]]; then
    echo "  ✗ 自检：拷 crates/ 去编译的行（共 $(grep -c . <<<"$copy_lines") 行）应当每一行都接着 copy_thread_registry_into <副本>；缺的：$(head -n 3 <<<"$unregistered_copy_lines" | clip_text 200)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 拷今天这棵树去编译的副本照共用库的闭包拷全（replay_copy_closure_into）：lib_repository_copy.py entries 列的每一条（path 依赖的 prune-server/、
  # 编译期读的 .claude/heavy-thread-default、运行期起的脚本）在副本里都在；e168_patched_tree 与 driver_e171_s4_comparator 都经它拷
  local closure_entries closure_entry closure_missing="" patched_tree_definition frozen_tree_definition positive_controls_definition
  # 驱动定义在这个函数被调用之后才读到：从本文件里取出那三段定义，落进变量再比（管道收尾的 grep -q 在 pipefail 下会把命中读成没命中）
  patched_tree_definition="$(awk '/^e168_patched_tree\(\) \{/,/^}/' "$REPLAY_ENTRY_SCRIPT")"
  frozen_tree_definition="$(awk '/^driver_e171_s4_comparator\(\) \{/,/^}/' "$REPLAY_ENTRY_SCRIPT")"
  positive_controls_definition="$(awk '/^driver_e168_positive_controls\(\) \{/,/^}/' "$REPLAY_ENTRY_SCRIPT")"
  checked=$((checked + 1))
  closure_entries="$(python3 scripts/hosts/lib_repository_copy.py entries . | awk -F'\t' '/^REPOSITORY_COPY entry /{ sub(/^REPOSITORY_COPY entry /, "", $1); print $1 }')"
  replay_copy_closure_into "$scratch/closure-tree" 2>"$scratch/closure-tree.err"
  while IFS= read -r closure_entry; do
    [[ -z "$closure_entry" || -e "$scratch/closure-tree/$closure_entry" ]] || closure_missing+=" $closure_entry"
  done <<<"$closure_entries"
  if [[ -z "$closure_entries" || -n "$closure_missing" || "$patched_tree_definition" != *replay_copy_closure_into* \
        || "$frozen_tree_definition" != *replay_copy_closure_into* ]]; then
    echo "  ✗ 自检：拷今天这棵树去编译的副本要照共用库的闭包拷全（lib_repository_copy.py entries 列的 $(grep -c . <<<"$closure_entries") 条都在，e168_patched_tree 与 driver_e171_s4_comparator 都经 replay_copy_closure_into 拷）；副本里缺的：${closure_missing:- 没有}；$(tail -n 2 "$scratch/closure-tree.err" 2>/dev/null | tr '\n' ' ' | clip_text 300)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  replay_remove_directory_below "${TMPDIR:-/tmp}" "$scratch/closure-tree"
  # 副本里起命令的多机配置取主工作区那一份（replay_configuration_for_the_copy）：没设 SINGLEFS_MULTI_HOST_CONFIG 时打 <主工作区>/multi-host.env，设了的照原样；
  # driver_e168_positive_controls 在副本里起 cargo 之前把它交给 SINGLEFS_MULTI_HOST_CONFIG
  local configuration_unset configuration_given main_tree
  checked=$((checked + 1))
  if main_tree="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"; then main_tree="$(cd "$(dirname "$main_tree")" && pwd -P)"; else main_tree="$(pwd -P)"; fi
  configuration_unset="$(unset SINGLEFS_MULTI_HOST_CONFIG; replay_configuration_for_the_copy)"
  configuration_given="$(export SINGLEFS_MULTI_HOST_CONFIG=/given/multi-host.env; replay_configuration_for_the_copy)"
  if [[ "$configuration_unset" != "$main_tree/multi-host.env" || "$configuration_given" != /given/multi-host.env \
        || "$positive_controls_definition" != *'export SINGLEFS_MULTI_HOST_CONFIG="$configuration"'* ]]; then
    echo "  ✗ 自检：副本里起命令的多机配置要取主工作区那一份（没设时应当是 $main_tree/multi-host.env、设了的照原样，driver_e168_positive_controls 起 cargo 前交给 SINGLEFS_MULTI_HOST_CONFIG）；实际没设得「$configuration_unset」、设了得「$configuration_given」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 计时行的负载关：假产物放进 $scratch/results/，复跑表是下面这几行，滞后表登记 e-lagged.out
  mkdir -p "$scratch/results"
  local -A TIMING_LAG=()
  local TIMING_LOAD_RATIO=0.5 lag_table="$scratch/lag.tsv" lag_problems
  local selftest_table
  selftest_table="$(printf '%s\n' 'E1|e1-high||e-high.out|timing' 'E2|e2-low||e-low.out|timing' 'E3|e3-none||e-none.out|timing' 'E4|e4-lagged||e-lagged.out|timing' 'E5|e5-exact||e-exact.out|exact')"
  printf '%s\n' 'E7RESULT name=config threads=4' 'E7RESULT name=done emitted=2 loadavg1=20.50 cpus=32' >"$scratch/results/e-high.out"
  printf '%s\n' 'E7RESULT name=config threads=4' 'E7RESULT name=done emitted=2 loadavg1=2.50 cpus=32' >"$scratch/results/e-low.out"
  printf '%s\n' 'E7RESULT name=config threads=4' 'E7RESULT name=done emitted=2' >"$scratch/results/e-none.out"
  cp -- "$scratch/results/e-none.out" "$scratch/results/e-lagged.out"
  printf '%s\n' '# 样本滞后表' '' $'e-lagged.out\t2026-10-13 立 C350 之前的产物，装置还没加收尾字段' >"$lag_table"
  checked=$((checked + 1))
  if ! load_timing_lag "$lag_table" "$selftest_table" "$scratch/results" >"$scratch/lag.problems"; then   # 不经命令替换：TIMING_LAG 要留在这一层
    echo "  ✗ 自检：只登记了复跑表里 timing 行的留存产物、理由写全的滞后表应当读得过；实际报：$(cat "$scratch/lag.problems")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-high.out "$scratch/results/e-high.out"
  if [[ "$TIMING_LOAD_DECISION" != skip || "$TIMING_LOAD_NOTE" != *"比值 0.640625"* ]]; then
    echo "  ✗ 自检：负载 20.5 / 32 核超过阈值 0.5，这一行应当记本次不判并点名比值；实际 $TIMING_LOAD_DECISION：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-low.out "$scratch/results/e-low.out"
  if [[ "$TIMING_LOAD_DECISION" != judge || "$TIMING_LOAD_LAGGED" != 0 ]]; then
    echo "  ✗ 自检：负载 2.5 / 32 核没超阈值，这一行应当照判；实际 $TIMING_LOAD_DECISION（滞后 $TIMING_LOAD_LAGGED）：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-none.out "$scratch/results/e-none.out"
  if [[ "$TIMING_LOAD_DECISION" != red || "$TIMING_LOAD_NOTE" != *"e-none.out"* || "$TIMING_LOAD_NOTE" != *"收尾行没有 loadavg1 / cpus"* ]]; then
    echo "  ✗ 自检：收尾行没有字段又不在滞后表，这一行应当判红并点名产物与「收尾行没有 loadavg1 / cpus」；实际 $TIMING_LOAD_DECISION：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-lagged.out "$scratch/results/e-lagged.out"
  if [[ "$TIMING_LOAD_DECISION" != judge || "$TIMING_LOAD_LAGGED" != 1 ]]; then
    echo "  ✗ 自检：收尾行没有字段但登记在滞后表里，这一行应当照判并记滞后；实际 $TIMING_LOAD_DECISION（滞后 $TIMING_LOAD_LAGGED）：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide exact e-none.out "$scratch/results/e-none.out"
  if [[ "$TIMING_LOAD_DECISION" != judge ]]; then
    echo "  ✗ 自检：判据写 exact 的行不看负载字段，没字段也照判；实际 $TIMING_LOAD_DECISION：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  printf '%s\n' $'e-ghost.out\t2026-10-13 不在复跑表里的名字' $'e-exact.out\t2026-10-13 复跑表里是 exact 的行' $'e-lagged.out\t2026-10-13 重复登记' $'e-short.out\t短' 'e-spaced.out  # 用空格隔开理由的旧写法，不是两列' >>"$lag_table"
  if lag_problems="$(load_timing_lag "$lag_table" "$selftest_table" "$scratch/results")" \
     || [[ "$lag_problems" != *"e-ghost.out 不是复跑表任何一条 timing 行"* || "$lag_problems" != *"e-exact.out 不是复跑表任何一条 timing 行"* \
        || "$lag_problems" != *"重复登记 e-lagged.out"* || "$lag_problems" != *"理由不足 8 个字"* \
        || "$lag_problems" != *"没有用制表符隔开的理由"* ]]; then
    echo "  ✗ 自检：滞后表登记了复跑表里没有的名字、exact 行的产物、重复名字、没写够理由、用空格隔开理由，应当判红并逐条点名；实际：$lag_problems"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  printf '%s\n' '# 样本滞后表' $'e-high.out\t2026-10-13 这份已经有字段了却还登记着' >"$lag_table"
  if lag_problems="$(load_timing_lag "$lag_table" "$selftest_table" "$scratch/results")" || [[ "$lag_problems" != *"e-high.out 自己的收尾行已经有 loadavg1 / cpus"* ]]; then
    echo "  ✗ 自检：滞后表登记的留存产物自己已经有字段，应当判红叫人删这一行；实际：$lag_problems"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  if [[ "$(printf '%s\n' 'E7RESULT name=config cpus=4' 'E7RESULT name=done emitted=2 loadavg1=2.50 cpus=32' | strip_load_fields)" != $'E7RESULT name=config cpus=4\nE7RESULT name=done emitted=2' ]]; then
    echo "  ✗ 自检：比对前要删掉 name=done 行上的 loadavg1= 与 cpus=、别的行上的同名字段不动"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 按行复用（文件头「按行复用」）：十二格各一行收回来，收回的格数对不上整个算一处不对。
  #   收格的 grep 带 -a：哪一格的说明里混进了不合法的 UTF-8（按字节截的半个字），GNU grep 照二进制只报「binary file matches」、那一格的行不交出来（少收一格），
  #   -a 照文本收；各格说明里按字节截的那几处另经 iconv -c 去掉半个字（本次未跑与分组的收法同此）
  local reuse_cell reuse_verdict reuse_detail reuse_collected=0
  replay_reuse_selftest "$scratch/reuse" >"$scratch/reuse.report" 2>&1
  while read -r reuse_cell reuse_verdict reuse_detail; do
    checked=$((checked + 1)); reuse_collected=$((reuse_collected + 1))
    if [[ "$reuse_verdict" != ok ]]; then
      echo "  ✗ 自检：按行复用 $reuse_cell：$reuse_detail"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done < <(grep -aE '^(reuse-[a-z0-9-]+|last-run-list) (ok|bad)' "$scratch/reuse.report")
  grep '^REUSE_SELFTEST ' "$scratch/reuse.report"
  if (( reuse_collected != 12 )); then
    echo "  ✗ 自检：按行复用的自证只收回 $reuse_collected 格（要 12 格）：$(tail -n 6 "$scratch/reuse.report" | tr '\n' ' ' | clip_text 800)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 留存产物取回与装置读不到输入（文件头「驱动要读的输入」）：三格各一行收回来，收回的格数对不上整个算一处不对
  local archived_cell archived_verdict archived_detail archived_collected=0
  replay_archived_selftest "$scratch/archived" >"$scratch/archived.report" 2>&1
  while read -r archived_cell archived_verdict archived_detail; do
    checked=$((checked + 1)); archived_collected=$((archived_collected + 1))
    if [[ "$archived_verdict" != ok ]]; then
      echo "  ✗ 自检：留存产物取回 $archived_cell：$archived_detail"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done < <(grep -aE '^(archived|device)-[a-z-]+ (ok|bad)' "$scratch/archived.report")
  if (( archived_collected != 3 )); then
    echo "  ✗ 自检：留存产物取回的自证只收回 $archived_collected 格（要 3 格）：$(tail -n 6 "$scratch/archived.report" | tr '\n' ' ' | clip_text 800)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 留存产物只以 .gz 存在时的比对（replay_gzip_selftest，两格）与入口脚本格式、三条完整性闸（replay_entry_selftest，五格）：每格一行收回来，
  # 收回的格数对不上整个算一处不对。一组一条 case 分支：报告名、格名的式样、要几格、报的时候怎么称呼。
  # 两组共用这一段收法：这份文件里程序读的注释（gate-lint 的明细标记这一类）进复跑表每一行的行键（lib_function_units.py 的 kept-comments），多一处就让每一行都现跑
  # 组的函数名写成 for 的字面词：单元闭包把命令位置上的 "$group_function" 折成这几个函数（read 读进来的值折不出，会拒算）
  local group_function group_report group_cells group_wanted group_title group_cell group_verdict group_detail group_collected
  for group_function in replay_gzip_selftest replay_entry_selftest replay_build_directory_selftest replay_dispatch_selftest replay_static_selftest replay_e171_library_selftest; do
    case "$group_function" in
      replay_gzip_selftest) group_report=gzip group_cells='gz-stored-[a-z]+' group_wanted=2 group_title='留存产物 .gz' ;;
      replay_entry_selftest) group_report=entry group_cells='list-items|unknown-experiment|integrity-[a-z-]+' group_wanted=5 group_title='入口与完整性闸' ;;
      replay_build_directory_selftest) group_report=build-directory group_cells='build-directory-[a-z-]+|caller-target-directory-kept|service-build-directory-kept|copy-build-directories-removed' group_wanted=6 group_title='编译目录' ;;
      replay_dispatch_selftest) group_report=dispatch group_cells='dispatch-[a-z-]+|row-[a-z-]+|stopped-keeps-finished-rows|e181-goes-through-long-run' group_wanted=17 group_title='派行' ;;
      replay_static_selftest) group_report=static group_cells='load-settle-awk-names|mutation-driver-tables' group_wanted=2 group_title='静态核' ;;
      replay_e171_library_selftest) group_report=e171-library group_cells='e171-missing-row-refused' group_wanted=1 group_title='E171 整表行只核账' ;;
    esac
    group_collected=0
    "$group_function" "$scratch/$group_report" >"$scratch/$group_report.report" 2>&1
    while read -r group_cell group_verdict group_detail; do
      checked=$((checked + 1)); group_collected=$((group_collected + 1))
      if [[ "$group_verdict" != ok ]]; then
        echo "  ✗ 自检：$group_title $group_cell：$group_detail"  # gate-lint:detail
        failures=$((failures + 1))
      fi
    done < <(grep -aE "^($group_cells) (ok|bad)" "$scratch/$group_report.report")
    if (( group_collected != group_wanted )); then
      echo "  ✗ 自检：$group_title 的自证只收回 $group_collected 格（要 $group_wanted 格）：$(tail -n 6 "$scratch/$group_report.report" | tr '\n' ' ' | clip_text 800)"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done
  # 驱动开跑前核输入（replay_inputs_present）：从本文件取出这几个驱动与它们用的函数、全局量，在一个没有 research/ 的空目录里起，
  # 冻结根与导出根在固定的位置（工作根下 replay-frozen-roots/，这里没建）；每个都要退 1、stderr 里打「✗ 输入不在 test=<驱动名>」行恰好那么多行（缺几样报几样），stdout 不打。
  # 弄坏开关 REPLAY_BREAK=inputs-not-checked 下它们往下去起编不存在的脚本，退的不是 1，这几格转红。
  local input_driver_definitions input_case input_driver input_expected input_lines input_status
  input_driver_definitions="$(awk '
    /^(E171_FREEZE_REFERENCE|E171_R1_DUMPS_REMEDY|E158_[A-Z0-9_]*ARMS)=/ { print; next }
    /^(replay_remove_directory_below|driver_e142|e170_r2_judge_images|driver_e170_r2_v2_487_z3_images|driver_e170_r2_v2_487_z4_images|driver_e171_s2_images_o3|driver_e171_s2_o3_control_baseline|driver_e171_s2_comparator|driver_e171_s3_comparator_mutations|driver_e171_s4_comparator|driver_e171_s4_f4_comparator|driver_e171_s4_f4_substitutes_comparator|e171_r1_revision_6_python|e171_r1_inputs_present|e171_r1_view|driver_e171_r1_s1_revision_6_comparator|driver_e171_r1_s1_revision_6_cross_host_check|driver_e171_r1_s2_revision_6_comparator|driver_e174_simulated_judge|driver_e174_segment_one_judge|driver_e174_segment_two_judge|driver_e158_r3_seg1_compare|driver_e158_r4_compare|driver_e165_segment2|driver_e165_r3_segment3_root_choice_kill_bounds|driver_e165_r4_segment2_root_choice_faults_kill_bounds|driver_e165_r5_segment2_r2_mode_kill_bounds|driver_e165_r5_segment2_r3_mode_kill_bounds|driver_e165_r5_segment2_r4_mode_kill_bounds|driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds)\(\) \{/ {
      if ($0 ~ /; \}[[:space:]]*$/) { print; next }
      inside = 1
    }
    inside { print; if ($0 ~ /^\}/) inside = 0 }' "${REPLAY_SOURCE_FILES[@]}")"
  mkdir -p "$scratch/inputs-empty-root" "$scratch/inputs-out"
  for input_case in driver_e142:1 driver_e170_r2_v2_487_z3_images:1 driver_e170_r2_v2_487_z4_images:1 \
    driver_e171_s2_images_o3:1 driver_e171_s2_o3_control_baseline:1 driver_e171_s2_comparator:1 driver_e171_s3_comparator_mutations:1 \
    driver_e171_s4_comparator:1 driver_e171_s4_f4_comparator:1 driver_e171_s4_f4_substitutes_comparator:1 \
    driver_e171_r1_s1_revision_6_comparator:14 driver_e171_r1_s1_revision_6_cross_host_check:3 driver_e171_r1_s2_revision_6_comparator:3 \
    driver_e174_simulated_judge:1 driver_e174_segment_one_judge:1 driver_e174_segment_two_judge:1 driver_e158_r3_seg1_compare:9 driver_e158_r4_compare:66 \
    driver_e165_segment2:1 driver_e165_r3_segment3_root_choice_kill_bounds:1 driver_e165_r4_segment2_root_choice_faults_kill_bounds:1 \
    driver_e165_r5_segment2_r2_mode_kill_bounds:1 driver_e165_r5_segment2_r3_mode_kill_bounds:1 driver_e165_r5_segment2_r4_mode_kill_bounds:1 \
    driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds:1; do
    input_driver="${input_case%%:*}"; input_expected="${input_case#*:}"
    checked=$((checked + 1))
    if (cd "$scratch/inputs-empty-root" && OUT_DIR="$scratch/inputs-out" && eval "$input_driver_definitions" && "$input_driver") \
         >"$scratch/inputs.stdout" 2>"$scratch/inputs.stderr"; then input_status=0; else input_status=$?; fi
    input_lines="$(grep -c "^  ✗ 输入不在 test=$input_driver " -- "$scratch/inputs.stderr")"
    if [[ "$input_status" != 1 || "$input_lines" != "$input_expected" || -s "$scratch/inputs.stdout" ]]; then
      echo "  ✗ 自检：$input_driver 在没有 research/、冻结根与导出根都没给的目录里应当退 1、打 $input_expected 行点名它的「✗ 输入不在」、stdout 不打；实际退 $input_status、$input_lines 行，stderr「$(tr '\n' ' ' <"$scratch/inputs.stderr" | clip_text 400)」"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done
  checked=$((checked + 1))
  local registered_default capped_jobs unset_jobs
  registered_default="$(grep -vE '^[[:space:]]*(#|$)' .claude/heavy-thread-default | head -n 1 | awk '{print $1}')"
  # default_replay_jobs 经 export -f 交给子 bash（只在这一层命令替换里导出），-c 的串整个是字面的：单元闭包认得出它起什么，串里拼函数正文的折不出、会拒算
  unset_jobs="$(export -f default_replay_jobs; env -u SINGLEFS_THREAD_CAP bash -c 'source scripts/hosts/heavy-threads.sh; REPLAY_BROKEN="$1"; default_replay_jobs' _ "$REPLAY_BROKEN")"
  capped_jobs="$(export -f default_replay_jobs; SINGLEFS_THREAD_CAP=7 bash -c 'source scripts/hosts/heavy-threads.sh; REPLAY_BROKEN="$1"; default_replay_jobs' _ "$REPLAY_BROKEN")"
  if [[ -z "$registered_default" || "$unset_jobs" != "$registered_default" || "$capped_jobs" != 7 ]]; then
    echo "  ✗ 自检：没设 REPLAY_JOBS 时同时跑几条实验要取登记的重活默认（登记「$registered_default」，没设上限得「$unset_jobs」，上限 7 得「$capped_jobs」）"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # load-commands-not-found：这一趟载入 replay.sh 时执行到的行一个找不到的命令都没有；现调一个不存在的命令，要被记下（证明这一格认得出）
  checked=$((checked + 1))
  if ((${#load_commands_not_found[@]})); then
    echo "  ✗ 自检：载入 replay.sh 时有行找不到命令（多半是一句注释丢了 #）：${load_commands_not_found[*]}；照 bash 打的那几行「line N: … command not found」找到那一行补上 #"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  replay_selftest_probe_command_that_does_not_exist 2>/dev/null
  local commands_not_found_now
  commands_not_found_now="$(replay_commands_not_found)"
  if ! grep -qx replay_selftest_probe_command_that_does_not_exist <<<"$commands_not_found_now"; then
    echo "  ✗ 自检：现调一个不存在的命令，command_not_found_handle 应当记下它的名字，实际没记：载入期找不到命令的行这一格认不出"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  replay_remove_directory_below "${TMPDIR:-/tmp}" "$scratch"
  if ((failures)); then
    echo "  ✗ replay.sh 自检 $failures 处不对（查了 $checked 项）"
    echo "    → 怎么办：看 build_research_binaries、admission_key_of_row 与计时行的负载关（timing_load_decide、load_timing_lag）；「按行复用 <格名>」那几处看 replay_reuse_selftest 里那一格的条件与它打的那一句（不在 git 工作树里那一格看取库名那一段对 replay_row_library_failure 的调用）；REPLAY_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ replay.sh 自检通过（查了 $checked 项：构建失败返回 2 并打第一处错、日志末尾、日志路径与重现命令，构建成功一个字都不打，驱动开跑前编研究二进制经 --manifest-path 指到研究工作区、编不过的输出进 .err（build_research_driver_binary 与 E162、E163 那三个驱动），指明了准入键的行按它问准入，按行复用接连五趟（头一趟两行都跑并记进判定库、第二趟都复用、改一行装置源码只重跑那一行、只改注释照样复用），开了 GPU 而要现跑的行都不点特性数组时不试编，点到的才试编、环境里没有卡串也照起，计时行负载超阈值记本次不判、没超照判、没字段判红、没字段但在滞后表照判，滞后表的登记错误逐条点名，配置开了 GPU 时 验证包装置带 gpu 特性（开了先试编、没开不起 cargo、配置读不出或试编红退 78 点名原因、起装置的 cargo run 行都带特性数组，开了 GPU 不另套清场、不领卡，要卡的行挪到最后一段、一行一行跑、看到环境里给的卡串（没有的不设）、不要卡的行看不到卡，起装置的 cargo run 行都不带清场前缀、本文件不起清场包装），同时跑几条实验的缺省取登记的重活默认，拷 crates/ 去编译的副本都带上每台默认重活线程数的登记文件，留存产物只在 git 历史里的行取回再比（字节一致并写出处）、取不回判「取不回」，装置退 77 打了 NOT_RUN 行的记跑不了、没有本次未跑的一档，读别的实验产物与留存产物的二十五个驱动缺输入时退 1、缺几样报几样，E171 整表四行库里缺一行时各退非 0、打带范围声明的分片入口整条命令、只起取行程序，--list-items 只列实验号、表里没有的实验号退 2，三条完整性闸各一格：退出码非 0、收尾行 emitted= 与实收对不上、同一输出目录里上一趟取回的留存产物不压过树里的，编译目录六格：跑绿、编不过、装置开跑之后被 TERM 三种结局之后都不剩这一趟的编译目录，调用方或服务给了 CARGO_TARGET_DIR 都照它编且不删（服务给的里面上一次尝试的产物照留），副本编出来的编译目录跑完删、产物照留）"
  return 0
}
