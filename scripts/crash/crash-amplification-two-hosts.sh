#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上两台各跑一份、拷回、导入、对账，上一次的结论不替这一次作保
# run-condition: command cargo ssh rsync git
# 崩溃放量流水线的双机跑法（里程碑二「增补 4」第一、四、十一项；用户 2026-09-28 定四种跑法只由配置的 ENABLE_ACROSS_MACHINES、ENABLE_GPU 两个开关定）：
#
#   crash-amplification-two-hosts.sh <测试目标> [--test-filter <用例函数名>] [--library <本机库目录>] [--expand-up-to <N>] [--threads <N>] [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>]
#                                    [--host-weights <本机>,<第二台>] [--local-states-per-second <本机量到的速度>] [--quiesce] [--quiesced] [--given-states]
#   --given-states：测试目标经给定状态引擎判（`crash_amplification::GivenStatesEngine`：每张表在引擎里先写进库、再从库里读回来判），不另起录入段（也不分着录）；
#                   第二台那一趟也不设选段（不给它 SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge），第二台的引擎同样先录再判：送过去的库快照照在本机这一趟之前，
#                   第二台只判就读不到这一趟要判的清单（报「先跑录入段」）；它录的表进它自己的库，取回后由步骤 ④ 导入
#   crash-amplification-two-hosts.sh --selftest      拿假的本机那一趟、假的第二台（换掉 multi-host-run.sh）与假的清场走一遍分份、快照、取回、导入、对账、
#                                                    先判着再按速度分、出错的几种与退出码的映射（不编、不起 cargo、不连第二台、不上卡；格名见下面「弄坏开关」）；
#                                                    末尾一轮（开关轮）从下面「弄坏开关」那张表现读开关与点名的格，逐个打开再跑一遍 --selftest，核红的格与点名的逐格相等
#
# 测试目标是 crates/singlefs-checker-tier/tests/ 下一条走 crash_amplification::run_from_environment 的用例（例 crash_amplification_fixed_script_through_rollback），
# 或经给定状态引擎判的随机历史用例（崩溃点重放入口 scripts/crash/crash-point-replay.sh 的快档开了双机时照这三步起它们：引擎读同样的 SHARE、LIBRARY、IMPORT）。
# 步骤：① 本机 SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2、库写到 --library（单独起时先跑的录入段与之后本机每一趟判段——先判着、分份、导入——都用这一份；
#          单独起（选段没设成 judge、不带 --given-states）时录入段分着录：清单、流程、事实表与判表（配了 GPU）照旧整条录（几十秒，录入键不含份），观察者的录入段逐状态在 CPU 上
#          恢复、观察（挂了观察者的用例要几个小时），只录这一台领的块，领哪几块与判段同一个分法（份加 SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS、按块序号），
#          所以本机录入段带 SHARE=0/2 与判段也带的权重（--host-weights 给了就带），本机那一份在后台子 shell 里先录入段（record.log）、录成了再判 0/2，与第二台同时开跑；
#          没给用 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/crash-state-verdicts，
#          crash-amplification-lib.sh 文件头「判定库」：一份库、几类测试同一份；块键带名字的旧库引擎开库就拒，这里不删、不读；
#          在 /tmp 之外：重启不清、重跑同一条命令就是续跑；用例里的 run_from_environment 拒绝落在 /tmp、/var/tmp、/dev/shm、/run 下的库），后台跑（先 6 路并行只编、再按线程数跑）；
#       ② 第二台 SHARE=1/2、库写到那棵树副本树根下的 crash-amplification-library/（引擎只收绝对路径：命令最前面的 bash -c 用第二台的 $PWD 拼成 <树根>/crash-amplification-library，
#          见 peer_library_path_script），经 scripts/hosts/multi-host-run.sh --peer --fetch 跑并取回
#          （树怎么拷、内存上限、工具链比对、脱离 ssh 会话、私有字换成「第二台」都归它）；树副本不带 multi-host.env，
#          两个开关 ENABLE_ACROSS_MACHINES / ENABLE_GPU 随命令的环境送过去（测试在配置不在时读同名环境变量）；
#          分着录时（单独起、不带 --given-states，权重录入时就有：给了 --host-weights，或什么都没给等分）第二台不设选段，引擎先录它领的那一份观察者块再判，
#          清单从送过去的库快照里读（快照照在本机开录之前；库里没有就自己整条录），录的块和判的块都写进它树里的 crash-amplification-library/，随 --fetch 取回，
#          本机那一趟判段读的是本机库里本机录的，第二台判段读的是它库里它录的，各写各的库；取回的库里第二台录的观察者块不并进本机库，并进去的是它判的结论（步骤 ④）；
#          只给 --local-states-per-second（权重要等第二台量完速度才有）时录入段还没有权重，仍由本机整份录（SHARE=0/1），第二台只判（SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge：
#          清单与观察者块在送过去的快照里）；分项驱动 crash-amplification-item.sh 交过来时选段已经是 judge，录入段在它那一步整份录完，同样第二台只判；
#          带 --given-states 时不设选段，第二台的引擎先录再判，录的表写进它树里的 crash-amplification-library/，随 --fetch 取回；
#       ③ 等本机那一份；④ 本机再跑一趟 SHARE=0/2 带 SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库>：导入第二台的块、对账、读违例——
#          用例在导入之后断言对账为空、违例为空，它退 0 这一趟才算全绿。② 失败（第二台被停、掉线、被杀）而库取回了一部分，也先导入再退：
#          第二台判到一半的块不丢，下一次同一条命令接着跑。--test-filter 只跑测试目标里点名的那一条用例（cargo test 的 --exact）。
# 跑法：配置 ENABLE_GPU=1 两台都带 gpu 特性编，各用多机配置里显卡先后（GPU_CARDS_BY_PRIORITY）点名给自己的卡、各在额度之内（用例里的 run_from_environment 做；
#       第二台的树副本不带配置，那个键与 SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer 随命令的环境送过去）；不是 1 只带 verdict-store。
#       走 GPU 的计算类：开了 GPU 引擎就三类全走 GPU，它不读 GPU_COMPUTATION_CLASSES（crash-amplification-lib.sh 文件头「配置」），
#       两台的每一趟（本机那几趟与第二台排计划、分份那两趟）都不送它，开不开 GPU 都一样；本机那几趟清掉调用方环境里漏进来的同名变量。
#       GPU 对拍的测试各占卡的显存，不并行起：开了 GPU 时两台每一趟 cargo test 的参数（共用库拼的，本脚本在 crash_amplification_prepare 之后另加）带 --test-threads 1，没开的不加。
#       两台怎么分：--host-weights 给了照它；给了 --local-states-per-second（本机排派活计划时试判量到的速度）就先让第二台也排一趟计划、量它的速度，
#       按两个速度分；都没给就等分。第二台量速度的那段时间（它要拷树、从头编、编内核）本机不空等：本机先领整份判着，
#       第二台量完就叫它停（建停下文件，它判完手里那一块、写进库就收尾），再按速度分工——判过的块在库里，不重判。
#       这里默认不清场（卡在额度之内用）；全量由 crash-amplification.sh 开跑就清场、带 --quiesced 调到这里。单独要清场显式带 --quiesce：照配置里的清场命令先停，跑完不复原、退出时报一句
#       「清了场、没复原」（peer-host-lib.sh 的 peer_host_quiesce / peer_host_quiesce_left；要复原由用户手动跑私有复原脚本）。--quiesced：调用方已经清过场；这里只把显卡的先后与额度
#       换成清场之后那一份（多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）。
# 往引擎送的变量（crash-point-replay.sh 文件头「往引擎送的变量」）：
#   SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY（本机设了就把那份按段登记文件拷一份快照经 --send 送进第二台那棵树的 crash-amplification-segments-registry/，
#   变量值在第二台上是那一份的绝对路径）：第二台排计划、分份那两趟都送；本机那几趟从调用方环境继承。
#   SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES（按段登记里分片的段一趟判几个态）：本机环境里设了（空串也算，照原样交、引擎拒）就把 <变量>=<原值> 放进
#   第二台排计划、分份那两趟的环境，没设不送（第二台照引擎的默认 2^32 判）；本机那几趟从调用方环境继承。
#   SINGLEFS_CRASH_LIBRARY_WRITE_ONLY（只写不读：库不读、全部块现判、照键写回；崩溃点重放入口 --full 设）：本机环境里设了就照原值放进第二台那两趟的环境，
#   没设不送（第二台照读库复用）；本机那几趟从调用方环境继承。
# 线程（crash-amplification-lib.sh 文件头「线程数的缺省」）：第二台排计划、分份那两趟起 capped.sh 的线程数与那两趟判的线程数（SINGLEFS_CRASH_AMPLIFICATION_THREADS）
#   取同一个数：CRASH_AMPLIFICATION_PEER_THREADS 给了就用它（正整数），没给取登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它；
#   用户 2026-10-12 定：默认每台 31、机器忙时也固定、不看忙闲）；--threads 只管本机跑的那一步（没给同样取登记值），不带给第二台。
#   本机只编那一步同样取登记值（CRASH_AMPLIFICATION_BUILD_THREADS 给了照它）。显式给的不是正整数退 78，那一趟与本机先判着、本机那一份都还没起。
# 第二台的编译目录：第二台排计划、分份那两趟共用树副本上一层的 crash-amplification-target（peer_environment 里的 CARGO_TARGET_DIR；每趟的树副本是新拷的，不共用就每趟从头编依赖）。
#   树副本经 multi-host-run.sh 的 rsync -a 拷过去、源文件带着本机的旧 mtime，而 cargo 按 mtime 判本地包新不新鲜：内容换了（别的树、别的会话在那个目录里编过）而 mtime
#   不比上一次编的新，就悄悄照用旧产物。所以两趟都把同一个目录交 multi-host-run.sh 的 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY（它的文件头 ⑤）：第二台起 cargo 之前
#   在 <编译目录>.lock 上持锁、scripts/hosts/source-stamp.py refresh 把内容或 mtime 与戳里不一样的源文件 touch 到此刻，cargo 打出 Running 再放锁。
#   代价：每趟新拷的树上一趟 touch 过的文件 mtime 又回到本机的旧值，refresh 照样再 touch，第二台每趟都重编本地包（依赖照旧复用）。
# ENABLE_ACROSS_MACHINES 不是 1 由 multi-host-run.sh 拒（退 255），这里不偷偷退回单机。
# 输出：三趟各自的 CRASH_AMPLIFICATION 行原样打出（第二台那一趟经 multi-host-run.sh 已换掉私有的字；先判着那一趟行尾加 warm_up=true、导入那一趟行尾加 import_run=true，见下）；日志放 <库目录的上层>/runs/<标签>/（--selftest 的全在一个 mktemp 目录里，退出时删掉）。
#   判段没跑的那一种（配置写了 ENABLE_GPU=1 而二进制没编进 gpu 特性：引擎每一趟判段都打一行 CRASH_AMPLIFICATION_NOT_RUN step=judge … flow=<流> reason=…、录入段照录、退 0）
#   那几行只从导入那一趟转打一次，崩溃放量入口据它把这一项记本次未跑；退出码照导入那一趟。
# 退出码：④ 那一趟 cargo test 的退出码；① 或 ② 失败退它们的退出码（先失败的那一个），一步都没跑退 2；250–254 是内存包装自己的结局；
#         分着录时 ① 里本机的录入段红了：退 crash-amplification-lib.sh 的 crash_amplification_record_on_this_host 照 record.log 分类之后交回的码（78、248、245 或原样），
#            本机判段不起，第二台那一趟照常跑完取回，再带停下文件导入它判过的块（不重新录观察者）；
#         78 运行条件不满足（出路：修好设备，或改配置）：说是清过场却没在多机配置里写清场之后那一份显卡先后、
#            --quiesce 时多机配置判不过或清场没过、
#            第二台那一趟 multi-host-run.sh 自己没跑成（它退 255：配置、工具链、拷树、连不上、等不到结局）、第二台的库没取回、
#            第二台那两趟或本机只编那一步显式给的线程数不是正整数（文件头「线程」），
#            或哪一趟的日志里有卡用不了的原话、给定状态表的判器借不到显存那一行（crash-amplification-lib.sh 的 crash_amplification_failure_kind 判 device），
#            或这份二进制跑不了这一趟（结论版本与树对不上、开了 GPU 而没编进 gpu 特性；同一个函数判 build）；
#            或录入段之后到导入那一趟之间测试二进制里的观察者版本变了而导入那一趟红了（上面「观察者版本」；出路：两趟之间判器单元变了，重跑整趟）；
#         248 算法上判不了：哪一趟的日志里有有状态 GPU 判不了、内核展开超预算、建不出表的原话（同一个函数判 algorithm）；
#         249 给了 --local-states-per-second 而第二台没量出速度、日志里认不出原因（打一行 CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED；本机先判着的块在库里，第二台一块都还没判）；
#         77  本次未跑：第二台排派活计划那一趟退 0、没打计划行，日志里有行首为 NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason= 的那一行
#             （第二台起过卡池、池里的卡等满了也借不到显存；行的写法同 crash-amplification-item.sh 文件头步骤 ②）。打一行
#             CRASH_AMPLIFICATION_NOT_RUN step=peer-placement test=crash_amplification::run_from_environment reason=<那一行 reason= 之后原样>，不分份、不改在本机一台跑；
#             本机先判着的块在库里。别的用例名的那一行、那一趟退了非 0 的，都不算这一种，照上面 249 那几种分类。
#         143 分份途中收到 TERM（见下「被停」）：先导入第二台取回的库再退。
#         分类只在退非 0 的那一趟上做；这几处映射由 --selftest 的各格证红（下面「弄坏开关」），分类函数本身另由 crash-amplification-item.sh --selftest 的第 8、9、11 格证红。
# 被停：分份那一段（本机 0/2 与第二台 1/2 两趟都在后台跑、驱动在 wait 里等）收到 TERM，把 TERM 转给这两趟（只发给起它们时记下的两个进程号，
#   发之前核它还是这个驱动的子进程），等第二台那一趟返回（multi-host-run.sh 收到 TERM 先停第二台那条命令、再把 --fetch 点名的库取回 fetched/），再等本机那一趟；
#   fetched/crash-amplification-library/CURRENT 在就照「先导入再退」带 IMPORT 跑一趟本机 0/2——带着先建好的停下文件（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE），
#   只导入第二台判过的块、本机这一份没判的块不再领——转打它的行，退 143。导入那一趟起着时再来的 TERM 只记一笔，不打断导入。
#   本机那一趟的 TERM 先到起它的那个子 shell，再由共用库 crash_amplification_run_on_this_host 转给它后台起的内存包装（包装收到 TERM 停自己开的 scope，cargo 随 scope 停），并等它退出、库的锁放掉才返回
#   （共用库 crash-amplification-lib.sh 文件头「被停」一段）；导入那一趟开库仍按 SINGLEFS_CRASH_LIBRARY_LOCK_WAIT_SECONDS 等锁。分份之前那几段（录入段、先判着、第二台排计划）收到 TERM 照旧当场退。
# 本机先判着、第二台量完速度就叫停的那一趟（warm-local.log）：它的 CRASH_AMPLIFICATION 行带 stopped_early=true，是有意叫停、剩下的由两台分份判完，
#   转打时在行尾加 warm_up=true，崩溃放量入口据此不把这一项报成「本次未跑」，复用数也只取这一行的（入口文件头「复用数怎么取」）。
# 观察者版本（录入段之后记下、导入那一趟对一遍）：挂了观察者的用例，录入段的块按观察者版本存、判段按同一把版本读（crash_amplification.rs 的 observer_versions_of_the_keys）；
#   录入段到导入那一趟之间测试二进制重编、判器单元变了（别的会话中途往 crates 里换了文件），导入那一趟的版本和录入段的不是一把，读不到录入段录的块，
#   用例报「录入段那一块不在库里：先跑录入段」——叫人去补录入段，其实录入段是好的。单独起（选段没设成 judge）、本机录入段成了之后，这里从 record.log 里 cargo 打的
#   Running … (<测试二进制>) 那一行找到二进制，读出编进去的判器键表里观察者那几组（observer_code:…、observer_conclusion_composition、judgement_of_the_crash_images）与工具链行
#   取 sha256，写进 <日志目录>/observer-version；导入那一趟之后对 import.log 指到的二进制再取一遍：对不上而且导入那一趟红了，退 78，
#   出路写「两趟之间判器单元变了，重跑整趟」；对不上而导入那一趟判绿，只打一行说明；录入段在分项驱动那边跑过（选段交过来就是 judge）、日志里找不到二进制、二进制里读不出表，都不核，打一行说明。
#   另读的输入（extra_inputs_digest）跑起来才算，二进制里没有，不在这一份摘要里。弄坏开关 observer-version-unchecked：导入那一趟不核。
# 导入那一趟（import.log，含本机或第二台那一份红了之后先导入的那一趟）：它的 CRASH_AMPLIFICATION 行转打时在行尾加 import_run=true。
#   那一趟的 states_reused= 是导入之前本机那一份已判的，崩溃放量入口据此不把它算进复用数（flow= 行不带 share= 与 imported_blocks=，只认得这个标记）。
# 重型：跑 checker 档用例，只在用户要求时跑（SINGLEFS_HEAVY_TESTS=user-request 由这里带给三趟 cargo）；--selftest 不起 cargo，不算重型。
#
# 只供测试（--selftest 给它起的每一趟里层设这几个；与 crash-amplification-item.sh 的同类变量一样，平常跑时设了也照认）：
#   CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN=<脚本>      换掉本机那一趟（调法同 crash-amplification-lib.sh 的 crash_amplification_run_on_this_host：
#                                                       <脚本> <share> <库目录> <日志> [<附加环境 键=值>…]，自己写日志）；换掉时这一趟的 cargo test 参数
#                                                       （共用库拼的 cargo_test_arguments，空格连起来）放进它环境里的 CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_TEST_ARGUMENTS；
#   CRASH_AMPLIFICATION_TWO_HOSTS_PEER_RUN=<脚本>       换掉 multi-host-run.sh（调法与它相同：--peer --label … [--fetch …] [--send …] [--memory-cap …] -- env <键=值…> <命令…>，
#                                                       取回的目录落 MULTI_HOST_RUN_FETCH_DIRECTORY）；
#   CRASH_AMPLIFICATION_TWO_HOSTS_PEER_HOST_LIB=<文件>  换掉 peer-host-lib.sh（--quiesce 用的 peer_host_load_configuration、peer_host_quiesce、peer_host_quiesce_left）。
#   CRASH_AMPLIFICATION_TWO_HOSTS_SWITCH_TABLE=<文件>   --selftest 末尾的开关轮从这份文件的文件头读开关表（不设读本脚本自己的），代码里认的开关照旧从本脚本取：
#                                                       拿一份改坏了表的拷贝（删一格、多一格、格名写错、漏一个开关）证开关轮会红。只在 --selftest 的开关轮里读。
#   CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED=<文件>   第二台量完速度、叫停本机先判着那一趟的那一步（建停下文件，或不建）走过之后建这份文件：
#                                                       假的先判着那一趟等「停下文件在」或「这份文件在」，后者先到就是没被叫停、当场退 101，不等满 10 秒。
#                                                       --selftest 给每一格设成格目录下的一份；停下文件在它之前建，所以两份都在时照叫停了算。
#   换不掉的：真的那一趟由 crash-amplification-lib.sh 起 cargo、multi-host-run.sh 拷树连第二台，自证不碰它们。
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」「测试目标」，.claude/hooks/run-scope-guard.sh 的拒绝说明读它）：
#   ~/.local/share/singlefs/last-runs/scripts__crash__crash-amplification-two-hosts.sh.tsv（SINGLEFS_LAST_RUNS_DIRECTORY 盖过），红的测试名一行一个：
#   「<测试目标>」，带 --test-filter 的「<测试目标>::<用例函数>」；退 0 是绿、退 2 / 77 / 130 / 143 照留，别的退出码是红；整个目标跑的那一趟换掉这个目标的全部项，
#   点名用例的那一趟只换那一项。--selftest 之后每一种退法都经 EXIT 钩子 two_hosts_on_exit 写；测试目标的源文件不在、又没显式设 SINGLEFS_LAST_RUNS_DIRECTORY 的
#   （别的自证拿假目标起真驱动）不写。自证那一格 last-run-list；自证里各格的里层都写进自证自己的临时目录。
# 弄坏开关 CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=<项>（几个用逗号连；只给 --selftest 证红用，自证起的每一趟里层都带着调用方设的值）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几格红（格名是 run_selftest 里 run_cell 的第一个参数，或不起里层的 switch-wheel-judges；判错的那一格打「✗ 自证 <格名>：」）。
# 写法：一行「#   <变量>=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（、隔开；括号里是说明，开关轮不读，括号外的每一段都算一格）。
# --selftest 末尾的开关轮从这一段与下面「共用库」那一段现读（crash-amplification-lib.sh 的 crash_amplification_read_switch_table），判法是同一份库的 crash_amplification_switch_wheel
# （与崩溃放量入口共用）：表里的双机开关与代码里 two_hosts_break_is 认的要相等、表里的共用库开关要是共用库里认的，每个逐个打开、要退 1、红的格与点名的逐格相等；
# 共用库认而表里没列的开关也逐个打开，要退 0、一格不红。并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS（没给取 SINGLEFS_THREAD_CAP，再没给按 nproc）。
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=same-share-on-both-hosts  第二台那一趟也领 0/2（两台判同一半）
#       observer-version-unrelated-group-keeps-the-exit、observer-version-changed-import-green-stays-green（两台判同一半，导入之后对账不为空，退 101；一个是退 101 的输出里没有「unjudged_blocks=1」，一个不退 0）
#       peer-engine-variables-absent-when-unset、peer-segments-registry-sent、peer-slice-states-sent、peer-write-only-sent（导入之后对账不为空、退 101）、
#       both-hosts-green、library-snapshot-sent、calibrated-weights（调用记录里没有第二台领 1/2 的那一趟）、
#       reconciliation-not-empty（没人判的是 3 块、不是漏判的那 1 块）、quiesce-not-restored（导入之后对账不为空、退 101）、
#       gpu-on-classes-not-sent、gpu-cards-from-multi-host-configuration、gpu-off-classes-not-sent（导入之后对账不为空、退 101，调用记录里也没有第二台领 1/2 的那一趟）、
#       peer-threads-default、peer-threads-given、peer-threads-cap、
#       peer-checking-threads-default、peer-checking-threads-given、peer-checking-threads-cap、records-when-not-judging、given-states-without-record（导入之后对账不为空、退 101）、
#       named-test-runs-even-if-ignored、records-library-marker、records-into-the-given-library、records-into-the-default-library-when-not-given、
#       crash-case-shaped-records-and-imports、judge-not-run-forwarded-once（导入之后对账不为空、退 101）
#       peer-build-guarded、peer-library-absolute（导入之后对账不为空、退 101）
#       observers-recorded-by-share、observers-recorded-by-share-with-weights、split-record-fails、split-term-imports-the-fetched-library（第二台也领 0/2：1 3 5 没人领，导入之后对账不为空，不是退 0；被停那一格里第二台库里的观察者块是 0 2 4，不是 1 3 5）、
#       gpu-test-threads-one、gpu-off-test-threads-not-added（导入之后对账不为空、退 101，调用记录里没有第二台领 1/2 的那一趟）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=skip-import               取回之后不跑导入那一趟、当它退 0
#       observer-version-unrelated-group-keeps-the-exit、observer-version-changed-exits-78、observer-version-unreadable-keeps-the-exit（导入那一趟没跑，漏判那一块没人发现，退 0，不退 101 / 78）
#       peer-segments-registry-sent、peer-slice-states-sent、peer-write-only-sent（调用记录里没有带 IMPORT 的本机 0/2 那一趟）、
#       both-hosts-green（最后一趟不是带 IMPORT 的本机 0/2）、reconciliation-not-empty（退 0，不退 101）、
#       gpu-on-classes-not-sent、gpu-cards-from-multi-host-configuration、gpu-off-classes-not-sent（调用记录里没有带 IMPORT 的本机 0/2 那一趟）、
#       records-into-the-given-library、records-into-the-default-library-when-not-given（.library 里本机只有两笔，少了导入那一趟）、
#       crash-case-shaped-records-and-imports（调用记录里没有带 IMPORT 的本机 0/2 那一趟）、judge-not-run-forwarded-once（导入那一趟没跑，转打 0 行）、
#       observers-recorded-by-share、observers-recorded-by-share-with-weights（最后一趟不是带 IMPORT 的本机 0/2）、
#       gpu-test-threads-one（本机只有先判着与分份两趟，少了导入那一趟的 --test-threads 1 记录）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=term-skips-import         分份途中收到 TERM 照改前不导入第二台取回的库就退（文件头「被停」）
#       term-imports-the-fetched-library、split-term-imports-the-fetched-library（调用记录里没有带 IMPORT 的本机 0/2 那一趟，第二台判过的块没进本机库）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=record-skipped-when-not-judging  单独起（选段没设成 judge）也不先在本机跑录入段
#       observer-version-unrelated-group-keeps-the-exit、observer-version-changed-exits-78、observer-version-unreadable-keeps-the-exit（没有录入段，没有 record.log，记不下版本，退 101 但 <日志目录>/observer-version 不在、也没有「观察者版本取不到」那一行）
#       records-when-not-judging、records-library-marker、records-into-the-given-library、records-into-the-default-library-when-not-given、
#       crash-case-shaped-records-and-imports（.stage 或 .library 里第一笔不是本机录入段那一趟）、
#       observers-recorded-by-share、observers-recorded-by-share-with-weights、observers-recorded-whole-when-weights-wait-for-speed、split-record-fails、split-term-imports-the-fetched-library（.record-share 里没有本机录入段那一笔）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=record-for-given-states  带了 --given-states 也先起录入段
#       given-states-without-record（.stage 里有一笔 record）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-judges-only-for-given-states  带了 --given-states 也照改前给第二台设 SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge（第二台只判、不录）
#       given-states-without-record（.stage 里第二台那一笔是 stage=judge）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=record-whole-on-this-host  不分着录：照改前本机用 0/1 把整份录完（观察者整份只有本机录）、第二台选段设成 judge 只判
#       records-when-not-judging、records-library-marker、records-into-the-given-library、records-into-the-default-library-when-not-given、crash-case-shaped-records-and-imports、
#       observers-recorded-by-share、observers-recorded-by-share-with-weights、split-record-fails、split-term-imports-the-fetched-library
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-judges-only-after-split-record  分着录时第二台也照改前设 SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge（只判，它那一份观察者没人录）
#       records-when-not-judging、records-library-marker、records-into-the-given-library、records-into-the-default-library-when-not-given、crash-case-shaped-records-and-imports、
#       observers-recorded-by-share、observers-recorded-by-share-with-weights、split-record-fails、split-term-imports-the-fetched-library
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=record-weights-dropped  分着录时本机录入段不带 --host-weights 给的权重（录的是等分的那一份，判段按权重领，读不到录入块）
#       observers-recorded-by-share-with-weights
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=split-record-failure-ignored  分着录时本机录入段红了也接着判
#       split-record-fails
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=record-into-the-default-library  给了 --library 时录入段照改前写进默认库，--library 只管之后那几趟判段（改前的样子：判段在给的库里找不到录入段写下的清单）
#       records-into-the-given-library（.library 里录入段那一笔、转打的 CRASH_LIBRARY_RECORDED 行的 library= 是默认库，不是给的那一个）、
#       observers-recorded-by-share、observers-recorded-by-share-with-weights、observers-recorded-whole-when-weights-wait-for-speed（录入段写进默认库，本机判段在给的库里读不到观察者录入块，退 101）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=no-library-snapshot       本机库已有块也不照快照送给第二台
#       library-snapshot-sent（第二台那一趟没带 --send、库里那一块没复用）、library-snapshot-fails（不照快照也就拷不坏，退 0，不退 2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=ignore-snapshot-failure   快照拷不成也照跑
#       library-snapshot-fails（不退 2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=local-failure-ignored     本机那一份退非 0 也往下走（导入、退 0）
#       local-share-fails（退 0，不退 101）、local-share-card-unusable（退 0，不退 78）、local-share-given-states-run-condition（退 0，不退 78）、local-share-gpu-memory（退 0，不退 245）、
#       split-record-fails（本机录入段红了也退 0，不退 101）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=gpu-memory-unexplained    本机或第二台那一份退 245（内存包装认出的显存不够）时打通用出路、不说显存不够
#       local-share-gpu-memory、peer-share-gpu-memory
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=local-failure-skips-salvage  本机那一份红了就不导入第二台取回的库
#       local-share-fails、split-record-fails（最后一趟不是带 IMPORT 的本机 0/2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-unreachable-as-run-failure  multi-host-run.sh 退 255 不映射成 78，照日志分类
#       peer-unreachable（退 255）、peer-placement-unreachable（退 249）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-failure-ignored      第二台那一份退非 0 也往下走（导入、退 0）
#       peer-share-card-unusable（退 0，不退 78）、peer-share-gpu-memory（退 0，不退 245）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-failure-skips-salvage  第二台那一份红了就不导入它取回的半份库
#       peer-share-card-unusable（最后一趟不是带 IMPORT 的本机 0/2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=accept-missing-fetched-library  第二台的库没取回也照导入
#       peer-library-not-fetched（导入那一趟找不到库、退 101，不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=import-failure-ignored    导入那一趟退非 0（对账不为空）也报绿
#       observer-version-unrelated-group-keeps-the-exit、observer-version-unreadable-keeps-the-exit（导入那一趟红了也报绿，退 0，不退 101）
#       reconciliation-not-empty（退 0，不退 101）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=drop-host-weights         算出两台的分工也不带给两趟分份
#       calibrated-weights（分份那两趟的调用记录里 weights=none）、observers-recorded-by-share-with-weights（第二台那一趟没拿到权重，调用记录里 weights=none）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=no-warm-up-mark           先判着的那一趟转打时不加 warm_up=true
#       calibrated-weights（没有带 warm_up=true 的 CRASH_AMPLIFICATION 行）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=no-import-mark            导入那一趟转打时不加 import_run=true
#       both-hosts-green、local-share-fails、peer-share-card-unusable（导入那一行行尾没有 import_run=true）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=never-stop-warm-run       第二台量完速度不建停下文件
#       peer-engine-variables-absent-when-unset、peer-segments-registry-sent、peer-slice-states-sent、peer-write-only-sent（同下退 101，调用记录里没有分份那一趟）、
#       calibrated-weights、observers-recorded-whole-when-weights-wait-for-speed、peer-placement-unreachable、peer-placement-card-unusable、peer-placement-algorithm、peer-placement-unrecognized、
#       peer-placement-not-run、peer-placement-foreign-not-run-line、gpu-on-classes-not-sent、gpu-cards-from-multi-host-configuration、gpu-off-classes-not-sent（假的先判着那一趟看到叫停那一步走过了、停下文件却不在，退 101；各格退 101）、
#       peer-threads-default、peer-threads-given、peer-threads-cap（同上退 101，分份那一趟没起、调用记录里只有排计划那一笔线程数）、
#       peer-checking-threads-default、peer-checking-threads-given、peer-checking-threads-cap（同上退 101，第二台只记下排计划那一笔）
#       peer-build-guarded（同上退 101，.shared-target 里只有排计划那一笔）、peer-library-absolute（同上退 101）、
#       gpu-test-threads-one、gpu-off-test-threads-not-added（同上退 101，分份那一趟没起、调用记录里只有先判着与排计划两笔）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=warm-failure-ignored      本机先判着的那一趟退非 0 也往下走
#       warm-run-card-unusable（退 0，不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-placement-failure-ignored  第二台没量出速度也往下走（分工写成「本机速度,」）
#       peer-placement-unreachable、peer-placement-card-unusable、peer-placement-algorithm、peer-placement-unrecognized、peer-placement-foreign-not-run-line（都退 0）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-placement-not-run-as-failure  第二台排计划那一趟借不到显存的那一行不认，照没量出速度退 249（改前的写法）
#       peer-placement-not-run（退 249，不退 77）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-placement-accept-any-not-run-line  别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行也当第二台借不到显存
#       peer-placement-foreign-not-run-line（退 77，不退 249）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-placement-unclassified  第二台没量出速度时不照日志分类（一律 249）
#       peer-placement-card-unusable（不退 78）、peer-placement-algorithm（不退 248）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=accept-bad-host-weights   --host-weights 写错也照收
#       host-weights-malformed（不退 2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=accept-bad-local-speed    --local-states-per-second 写错也照收
#       local-speed-malformed（不退 2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=usage-exits-1             用法错退 1（不退 2）
#       usage-missing-target、usage-unknown-option
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=prepare-failure-ignored   crash_amplification_prepare 交回非 0 也照跑（说是清过场却没在多机配置里写清场之后那一份显卡先后）
#       quiesced-without-card-list（不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=ignore-peer-configuration  --quiesce 时多机配置判不过也照跑
#       quiesce-configuration-fails（不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=ignore-quiesce-failure    --quiesce 时清场没过也照跑
#       quiesce-fails（不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=restore-after-run         --quiesce 清过场、退出时又照配置跑了复原命令
#       quiesce-not-restored（调用记录里有 restore）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=classes-sent-to-peer  开了 GPU 照改前把三类 GPU_COMPUTATION_CLASSES 送给第二台
#       gpu-on-classes-not-sent（第二台那几笔调用记录里 classes= 不是 [unset]）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=gpu-test-threads-unset  开了 GPU 照改前不给 cargo test 参数加 --test-threads 1（本机三趟与第二台两趟都不带）
#       gpu-test-threads-one
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=segments-registry-not-sent-to-peer  本机环境里设了 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY 也不把那份文件送给第二台、不送变量（改前的样子）
#       peer-segments-registry-sent（第二台那两趟的 registry= 不是第二台树里那份快照的路径、或文件内容对不上）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=slice-states-not-sent-to-peer  本机环境里设了 SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES 也不送给第二台（改前的样子：第二台照 2^32 判）
#       peer-slice-states-sent（第二台那两趟的 slice_states= 是 [unset]，不是本机设的 131072）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=write-only-not-sent-to-peer  本机环境里设了 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY 也不送给第二台（改前的样子：第二台照读库复用）
#       peer-write-only-sent（第二台那两趟的 write_only= 是 [unset]，不是本机设的 1）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-threads-fixed        第二台那两趟照改前写死 capped.sh 8，不取登记值、也不认 CRASH_AMPLIFICATION_PEER_THREADS
#       peer-threads-default、peer-threads-given、peer-threads-cap（第二台那两趟的线程数是 8）、peer-threads-given-invalid（退 0，不退 78）、
#       peer-checking-threads-default、peer-checking-threads-given、peer-checking-threads-cap（第二台那两趟判的线程数也是 8）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-checking-threads-from-this-host  第二台那两趟判的线程数 SINGLEFS_CRASH_AMPLIFICATION_THREADS 照改前沿用本机的（common_environment 里那一份）
#       peer-checking-threads-default、peer-checking-threads-given、peer-checking-threads-cap（第二台那两趟判的线程数是本机 --threads 的 9）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-cap-from-configuration  没给 --peer-memory-cap 时第二台不带上限、落到配置的 PEER_MEMORY_CAP（改前的样子）
#       peer-cap-follows-this-host（第二台那一趟没拿到本机的 --memory-cap）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=wheel-ignores-cells       开关轮只比退出码、不比红的格（crash_amplification_switch_wheel 的第三个参数、crash_amplification_switch_wheel_verdict 的第五个参数给 1）
#       switch-wheel-judges（造出来的输出上删一格、多一格、格名写错、没列的开关红了都判成对上）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=judge-not-run-dropped     导入那一趟打的 CRASH_AMPLIFICATION_NOT_RUN step=judge 行不转打（入口认不出这一项判段没跑）
#       judge-not-run-forwarded-once
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-shared-target-unguarded  第二台那两趟照改前不交 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY（共用编译目录不持锁、不刷源码戳，旧 mtime 新内容的树照用别的树编的旧产物）
#       peer-build-guarded（第二台那两趟的 shared_target= 是 [unset]）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=last-run-unwritten        跑完不写上次红的清单（改前的样子）
#       last-run-list（清单一直不在，红的那两趟之后对不上）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-library-relative     第二台那两趟照改前在命令的 env 里交相对路径 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=crash-amplification-library（引擎拒相对路径，第二台那一份当场红）
#       peer-library-absolute、records-into-the-given-library、records-into-the-default-library-when-not-given（.library 里第二台那几笔是相对路径 crash-amplification-library）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=observer-version-unchecked  导入那一趟不核观察者版本（改前的样子：版本变了落成假用例「先跑录入段」那一句，退导入那一趟的 101）
#       observer-version-changed-exits-78（退 101，不退 78）
# 共用库 crash-amplification-lib.sh 的开关 CRASH_AMPLIFICATION_ITEM_BREAK（开关表在 crash-amplification-item.sh 文件头）打开时这里也红（共用库认而这里没列的，开关轮要它一格不红）：
#   CRASH_AMPLIFICATION_ITEM_BREAK=device-unusable-as-run-failure
#       local-share-card-unusable、peer-share-card-unusable、warm-run-card-unusable（退 101，不退 78）、peer-placement-card-unusable（退 249，不退 78）
#   CRASH_AMPLIFICATION_ITEM_BREAK=algorithm-refusal-as-run-failure
#       peer-placement-algorithm（退 249，不退 248）
#   CRASH_AMPLIFICATION_ITEM_BREAK=given-states-run-condition-as-run-failure  给定状态表的判器借不到显存那一行（CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET）不认
#       local-share-given-states-run-condition（退 101，不退 78）
#   CRASH_AMPLIFICATION_ITEM_BREAK=cards-read-from-cards-file  显卡先后照改前读退役的私有显卡配置，不读多机配置
#       gpu-cards-from-multi-host-configuration（调用记录里 cards= 是私有那一份的 local:1:3G peer:1:3G）、quiesced-without-card-list（私有那一份写了清场之后那一份，不退 78）
#   CRASH_AMPLIFICATION_ITEM_BREAK=inherited-classes-kept  本机那一趟不清掉调用方环境里漏进来的 GPU_COMPUTATION_CLASSES
#       gpu-on-classes-not-sent、gpu-off-classes-not-sent（本机那几笔调用记录里 classes=[crash_judge_gpu]）
#   CRASH_AMPLIFICATION_ITEM_BREAK=classes-sent-again  开了 GPU 照改前把三类 GPU_COMPUTATION_CLASSES 交给本机那一趟
#       gpu-on-classes-not-sent（本机那几笔调用记录里 classes= 是三类）
#   CRASH_AMPLIFICATION_ITEM_BREAK=build-threads-fixed  本机只编那一步照改前写死 capped.sh 6，不取登记值、也不认 CRASH_AMPLIFICATION_BUILD_THREADS
#       this-host-threads-default、this-host-threads-given、this-host-threads-cap（只编那一步的线程数是 6）、this-host-threads-given-invalid（交回 0，不交回 78）
#   CRASH_AMPLIFICATION_ITEM_BREAK=threads-default-cores  没给线程数时取本机核数减一（改前的样子），不取登记值、不认 SINGLEFS_THREAD_CAP（共用库 crash_amplification_default_threads）
#       peer-threads-cap、peer-checking-threads-cap、this-host-threads-cap（设了 SINGLEFS_THREAD_CAP=5 还是核数减一；本机核数减一不等于登记值的机器上 -default 三格也红，开关轮在那种机器上对不上）
#   CRASH_AMPLIFICATION_ITEM_BREAK=include-ignored-dropped  点名了用例（--test-filter）的那一趟照改前只带 --exact、不带 --include-ignored：标 ignore 的用例（崩溃点重放的全量用例都是）一条都不跑、退 0，录入段空着
#       named-test-runs-even-if-ignored（第二台分份那一趟的调用记录里 include_ignored=no）、this-host-named-test-runs-even-if-ignored（共用库拼出的 cargo test 参数里没有 --include-ignored）、
#       crash-case-shaped-records-and-imports（照崩溃点重放用例的样子造的假用例标着 ignore，本机录入段那一趟一条都不跑、一行录入行都没有，退 1）
#   CRASH_AMPLIFICATION_ITEM_BREAK=library-marker-not-accepted  录入段照改前只认 CRASH_AMPLIFICATION_RECORDED 行，不认崩溃枚举用例经判定库两段跑时打的 CRASH_LIBRARY_RECORDED 行：崩溃点重放双机那几条录入段跑成了也判「一行都没有」、退 1
#       records-library-marker、records-into-the-given-library、records-into-the-default-library-when-not-given、crash-case-shaped-records-and-imports（退 1、一趟判段都不起）
#   CRASH_AMPLIFICATION_ITEM_BREAK=library-in-the-old-directory  没给 --library 时默认库照改前打 <家目录>/library
#       records-into-the-default-library-when-not-given（.library 里本机那几笔是 <家目录>/library）
#
# gate-similar: crash-point-replay-shard-run.sh 它按 SINGLEFS_CRASH_AMPLIFICATION_SHARE 切片跑登记的崩溃枚举用例、写 checker-tier-crash-point-replay 全绿标记，账本与 merge 是崩溃点重放那一套；这里跑的是判定存储那一套（各写各的库、导入并成一个）
# gate-similar: multi-host-run.sh 它只管把一条命令送到第二台跑并取回产物，不知道分片；这里调它送第二台那一份，本机那一份与导入那一趟自己跑
# gate-similar: mutation-shard-run.sh 它按用时把变异表分两堆、两台各跑门禁 checker-tier-crates-mutation-replay；分的是变异行，不是崩溃状态块
# gate-similar: crash-amplification-item.sh 它排派活计划、定一个分项在本机一台跑还是动用第二台；动用第二台时调这里
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

two_hosts_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
two_hosts_script="$two_hosts_script_directory/$(basename "${BASH_SOURCE[0]}")"
repository_root="$(cd "$two_hosts_script_directory/../.." && pwd)"
source "$two_hosts_script_directory/crash-amplification-lib.sh"

two_hosts_break_is() { [[ ",${CRASH_AMPLIFICATION_TWO_HOSTS_BREAK:-}," == *",$1,"* ]]; }
two_hosts_exit_code_is_explained() { # <要退的码>：78、248 与 245（内存包装认出的显存不够）由 crash_amplification_explain_exit_code 打出路
  [[ "$1" == 78 || "$1" == 248 ]] || { [[ "$1" == 245 ]] && ! two_hosts_break_is gpu-memory-unexplained; }
}

# 只供测试（文件头「只供测试」）：换掉本机那一趟、第二台那一趟与清场。本机那一趟的环境照真的那一趟组（crash_amplification_environment_for_this_host），只把起 cargo 换成假的
if [[ -n "${CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN:-}" ]]; then
  crash_amplification_run_on_this_host() {
    crash_amplification_environment_for_this_host "$1" "$2" "${@:4}"
    env "${this_host_environment[@]}" CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_TEST_ARGUMENTS="${cargo_test_arguments[*]}" bash "$CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN" "$@"
  }
fi
multi_host_runner="${CRASH_AMPLIFICATION_TWO_HOSTS_PEER_RUN:-$two_hosts_script_directory/../hosts/multi-host-run.sh}"
peer_host_library="${CRASH_AMPLIFICATION_TWO_HOSTS_PEER_HOST_LIB:-$two_hosts_script_directory/../hosts/peer-host-lib.sh}"

usage() {
  echo "  ✗ 用法：$0 <测试目标> [--test-filter <用例函数名>] [--expand-up-to <N>] [--threads <本机跑的那一步的核对线程数；第二台的取 CRASH_AMPLIFICATION_PEER_THREADS 或现算>] [--library <本机库目录，跨次复用>] [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>] [--host-weights <本机>,<第二台>] [--local-states-per-second <数>] [--quiesce] [--given-states]，或 --selftest" >&2
  echo "  → 怎么办：测试目标写 crates/singlefs-checker-tier/tests/ 下的文件名（不带 .rs），例 crash_amplification_fixed_script_through_rollback" >&2
  if two_hosts_break_is usage-exits-1; then exit 1; fi
  exit 2
}



# 自证：三份假的（本机那一趟、第二台那一趟的 multi-host-run.sh、清场用的 peer-host-lib.sh）加一份假的用例，每一格起一趟里层 bash <本脚本>，
# 看退出码、转打的行与调用记录。假的用例：一共 6 块，SHARE=i/n 判序号除以 n 余 i 的那几块；库是一个目录（CURRENT 加一份 blocks，一行一块的序号），
# 已在库里的块记复用、不重判；带 IMPORT 时把那个库的块并进来再对账：没人判的块数不是 0 就 panicked、退 101（真用例在导入之后断言对账为空）。
# 每一格各用自己的格目录（库、第二台的假树、调用记录、输出）与自己的标签（日志目录），不读别的格留下的东西。
run_selftest() {
  local work failures=0 cells=0 cell_directory="" cell_exit=0
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-amplification-two-hosts-selftest.XXXXXX")" || {
    echo "  ✗ --selftest：建不了临时目录（TMPDIR=${TMPDIR:-/tmp}）" >&2
    echo "  → 怎么办：看 TMPDIR 可不可写" >&2
    exit 2
  }
  trap 'rm -rf -- "${work:?}"' EXIT
  cat >"$work/case.sh" <<'STUB'
#!/usr/bin/env bash
# 假的用例（自证用）：6 块；SHARE=i/n 判序号除以 n 余 i 的块，库里已有的记复用；带 IMPORT 并进来再对账；带停下文件的那一趟判一块、等停下文件；每次调记一笔进 STUB_CALLS
# （classes=[…] 是这一趟环境里 GPU_COMPUTATION_CLASSES 的值，cards=[…] 是 GPU_CARDS_BY_PRIORITY 的值，没设记 [unset]）
set -uo pipefail
share="${SINGLEFS_CRASH_AMPLIFICATION_SHARE:?}" library="${SINGLEFS_CRASH_AMPLIFICATION_LIBRARY:?}"
# STUB_BINARY_RECORD / STUB_BINARY_JUDGE / STUB_BINARY_IMPORT（观察者版本那几格）：照 cargo test 在日志里打的那一行 Running … (<测试二进制>)，路径取这一趟
#   （录入段 / 判段 / 带 IMPORT 的导入那一趟）对应的变量；没设不打
running_kind=judge
[[ "${SINGLEFS_CRASH_AMPLIFICATION_STAGE-}" == record ]] && running_kind=record
[[ -n "${SINGLEFS_CRASH_AMPLIFICATION_IMPORT:-}" ]] && running_kind=import
running_variable="STUB_BINARY_${running_kind^^}"
[[ -z "${!running_variable:-}" ]] || echo "     Running tests/stub.rs (${!running_variable})"
# STUB_CASE_SHAPE=crash-case：照崩溃点重放双机那几条用例的样子——标 #[ignore]：这一趟 cargo test 的参数（STUB_TEST_ARGUMENTS：本机那几趟是驱动交给假的本机那一趟的
#   CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_TEST_ARGUMENTS，第二台那几趟是假的 multi-host-run.sh 收到的命令）里没有 --include-ignored，就像 libtest 那样一条都不跑、
#   退 0（调用记录里记一笔 <台> ignored-not-run）；录入段只打 CRASH_LIBRARY_RECORDED、不打崩溃放量那一行；判段的汇总行打 CRASH_POINT_REPLAY_PARALLEL_FINISHED，不打 CRASH_AMPLIFICATION
crash_case_shape=0 summary_line=CRASH_AMPLIFICATION
if [[ "${STUB_CASE_SHAPE:-amplification}" == crash-case ]]; then crash_case_shape=1; summary_line=CRASH_POINT_REPLAY_PARALLEL_FINISHED; fi
if (( crash_case_shape )) && [[ " ${STUB_TEST_ARGUMENTS:-} " != *" --include-ignored "* ]]; then
  echo "${STUB_HOST:?} ignored-not-run" >>"${STUB_CALLS:?}"
  echo "running 0 tests"
  echo "test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out"
  exit 0
fi
# 这一趟跑哪一段另记进 <调用记录>.stage（录入段那一趟只记这一笔、打 RECORDED 行就退 0；没设记 unset）
echo "${STUB_HOST:?} stage=${SINGLEFS_CRASH_AMPLIFICATION_STAGE-unset}" >>"${STUB_CALLS:?}.stage"
# 这一趟 cargo test 的参数里有没有 --test-threads 1 另记进 <调用记录>.test-threads（gpu-test-threads-one、gpu-off-test-threads-not-added 两格读）
test_threads_given=no; [[ " ${STUB_TEST_ARGUMENTS:-} " == *" --test-threads 1 "* ]] && test_threads_given=yes
echo "$STUB_HOST test_threads=$test_threads_given" >>"${STUB_CALLS:?}.test-threads"
# 这一趟收到的库目录（SINGLEFS_CRASH_AMPLIFICATION_LIBRARY，原样）另记进 <调用记录>.library，录入段那一趟也记（records-into-the-given-library 那两格读）
echo "$STUB_HOST stage=${SINGLEFS_CRASH_AMPLIFICATION_STAGE-unset} library=$library" >>"$STUB_CALLS.library"
# STUB_OBSERVERS=1：模拟挂了观察者的用例——每一块有一份观察者录入块，录进库里的 observer-records（一行一块的序号）；
#   谁领哪一块：没权重按份 i/n 取余，带权重 w0,w1 时序号对 w0+w1 取余小于 w0 归 0、否则归 1（录入段与判段同一个算法，真引擎里是平滑加权轮转）；
#   录入段（STAGE=record）与先录再判（STAGE 没设、没有停下文件）把自己领的块录进自己的库；判段读自己领的、还没判的块时库里没有录入块就 panic、退 101（真引擎报「先跑录入段」）。
#   录入段那一趟另把「<台> share=… weights=…」记进 <调用记录>.record-share（observers-recorded-by-share 那几格读）
share_index="${share%/*}" share_count="${share#*/}" weights_text="${SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS:-none}"
owner_of_block() { # <块序号>：打领它的那一台的序号
  local block="$1" first_weight second_weight
  if [[ "${STUB_OBSERVERS:-}" == 1 && "$weights_text" != none ]]; then
    IFS=, read -r first_weight second_weight <<<"$weights_text"
    if (( block % (first_weight + second_weight) < first_weight )); then echo 0; else echo 1; fi
  else
    echo $(( block % share_count ))
  fi
}
record_my_observers() {
  [[ "${STUB_OBSERVERS:-}" == 1 ]] || return 0
  { mkdir -p -- "$library" && touch -- "$library/CURRENT" "$library/blocks" "$library/observer-records"; } 2>/dev/null || { echo "假的用例建不了库 $library"; exit 3; }
  local block
  for (( block = 0; block < 6; block++ )); do
    [[ "$(owner_of_block "$block")" == "$share_index" ]] || continue
    grep -qx "$block" "$library/observer-records" || echo "$block" >>"$library/observer-records"
  done
}
fail_if_told() { # STUB_FAILURE=plain|device：照点名失败，退 101；gpu-memory：演经内存包装跑的那一趟测试红了、包装认出显存不够，退 245
  case "${STUB_FAILURE:-}" in
    gpu-memory)
      echo "test result: FAILED"
      echo "run-with-memory-cap: 显存不够（假的用例）"
      exit 245
      ;;
    device) echo "GPU_CARD_NOT_USABLE index=0：配置点名了这张卡，这台机器却用不了它（假的用例）" ;;
    given-states) echo "CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET table=stub-table reason=配了 GPU，给定状态表的判器借不到显存（假的用例）" ;;
    plain) ;;
    *) return 0 ;;
  esac
  echo "thread 'the_case' panicked at crates/singlefs-checker-tier/src/crash_amplification.rs:1:1:"
  echo "假的用例照 STUB_FAILURE=$STUB_FAILURE 失败"
  echo "test result: FAILED"
  exit 101
}
if [[ "${SINGLEFS_CRASH_AMPLIFICATION_STAGE-}" == record ]]; then
  echo "$STUB_HOST share=$share weights=$weights_text" >>"$STUB_CALLS.record-share"
  fail_if_told
  record_my_observers
  # STUB_RECORD_MARKER=library：照崩溃枚举用例经判定库两段跑的样子只打 CRASH_LIBRARY_RECORDED（crash_library.rs 的 record_the_item），不打崩溃放量那一行
  if [[ "${STUB_RECORD_MARKER:-amplification}" == library ]] || (( crash_case_shape )); then
    echo "CRASH_LIBRARY_RECORDED class=crash_state_enumeration item=stub::full manifest=0000000000000000 library=$library"
  else
    echo "CRASH_AMPLIFICATION_RECORDED flow=stub recorded_flow=00 facts=00 packed_judge_tables=- blocks=6 exhaustive=true"
  fi
  exit 0
fi
# STUB_JUDGE_NOT_RUN=1（judge-not-run-forwarded-once 那一格）：照没编进 gpu 特性的二进制遇到 ENABLE_GPU=1 那样，判段每一趟（排计划那一趟除外）
#   先打 CRASH_AMPLIFICATION_NOT_RUN step=judge 与 NOT_RUN_NO_GPU_ADAPTER 两行，之后照常
if [[ "${STUB_JUDGE_NOT_RUN:-}" == 1 && "${SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY:-0}" != 1 ]]; then
  echo "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=stub reason=配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（假的用例）"
  echo "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（假的用例）"
fi
import="${SINGLEFS_CRASH_AMPLIFICATION_IMPORT:-}" stop_file="${SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE:-}"
placement="${SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY:-0}" weights="${SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS:-none}" total=6
import_mark=no; [[ -n "$import" ]] && import_mark=yes
stop_mark=no; [[ -n "$stop_file" ]] && stop_mark=yes
echo "${STUB_HOST:?} share=$share import=$import_mark placement=$placement stop=$stop_mark sent=${STUB_SENT:-no} classes=[${GPU_COMPUTATION_CLASSES-unset}] cards=[${GPU_CARDS_BY_PRIORITY-unset}] weights=$weights" >>"${STUB_CALLS:?}"
# 这一趟判的线程数（SINGLEFS_CRASH_AMPLIFICATION_THREADS）另记进 <调用记录>.checking-threads（线程数的缺省那几格读）
echo "$STUB_HOST placement=$placement checking_threads=${SINGLEFS_CRASH_AMPLIFICATION_THREADS-unset}" >>"$STUB_CALLS.checking-threads"
# 往引擎送的变量（SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY）在这一趟环境里的值另记进 <调用记录>.engine-environment；registry_sum 是那份文件的 cksum（没设写 unset，指到的文件不在写 missing）；
# 同一行的 write_only=[…] 与末尾 slice_states=[…] 是 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY、SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES 在这一趟环境里的值（没设写 unset）
registry="${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" registry_sum=unset
if [[ "$registry" != unset ]]; then registry_sum=missing; [[ -f "$registry" ]] && registry_sum="$(cksum <"$registry" | cut -d' ' -f1)"; fi
echo "$STUB_HOST placement=$placement registry=[$registry] registry_sum=[$registry_sum] write_only=[${SINGLEFS_CRASH_LIBRARY_WRITE_ONLY-unset}] slice_states=[${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES-unset}]" >>"$STUB_CALLS.engine-environment"
if [[ "$placement" == 1 ]]; then
  case "${STUB_PLACEMENT:-ok}" in
    device) echo "GPU_CARD_NOT_USABLE index=2：配置点名了这张卡，这台机器却用不了它（假的用例）"; echo "test result: FAILED"; exit 101 ;;
    algorithm)
      echo "thread 'the_case' panicked at crates/singlefs-checker-tier/src/crash_amplification.rs:1:1:"
      echo "卡 2 判不了点 p 块内第 5 个状态（恢复内核第 1 字 7、checker 内核第 3 字 9：表越过了内核的容量）。配置开了 GPU，不退回 CPU 判器"
      echo "test result: FAILED"; exit 101 ;;
    unrecognized) echo "test result: FAILED"; exit 101 ;;
    not-run) # 第二台起过卡池、等满了也借不到显存：照 Rust 那一行打，不打计划行，退 0
      echo "running 1 test"
      echo "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB（假的用例）"
      echo "test result: ok. 1 passed; 0 failed"; exit 0 ;;
    foreign-not-run) # 别的用例名的那一行（不是产品路径的），退 0、不打计划行
      echo "NOT_RUN_NO_GPU_ADAPTER test=some_gpu_parity_test reason=本机没有 NVIDIA 卡（假的用例）"
      echo "test result: ok. 1 passed; 0 failed"; exit 0 ;;
  esac
  rate="${STUB_RATE:-50}"
  echo "CRASH_AMPLIFICATION_PLACEMENT host=$STUB_HOST judge=gpu pending_blocks=$total pending_states=6000 states_per_second_on_this_host=$rate estimated_seconds_on_this_host=$(( 6000 / rate ))"
  exit 0
fi
{ mkdir -p -- "$library" && touch -- "$library/CURRENT" "$library/blocks"; } 2>/dev/null || { echo "假的用例建不了库 $library"; exit 3; }
if [[ -n "$stop_file" && -z "$import" ]]; then
  fail_if_told
  grep -qx 0 "$library/blocks" || echo 0 >>"$library/blocks"
  # 等驱动叫停：停下文件在就是叫停了；驱动走过叫停那一步（STOP_DECIDED 那份文件在）而停下文件不在，就是没叫停、当场判红；两样都等不到，等满 10 秒判红
  waited=0 decided="${CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED:-}"
  while [[ ! -e "$stop_file" ]] && [[ -z "$decided" || ! -e "$decided" ]] && (( waited < 100 )); do sleep 0.1; waited=$((waited + 1)); done
  if [[ ! -e "$stop_file" && -n "$decided" && -e "$decided" ]]; then
    echo "thread 'the_case' panicked: 驱动走过了叫停那一步（$decided 在），停下文件 $stop_file 却没建"
    echo "test result: FAILED"
    exit 101
  fi
  if [[ ! -e "$stop_file" ]]; then
    echo "thread 'the_case' panicked: 等了 10 秒，停下文件 $stop_file 也没建出来"
    echo "test result: FAILED"
    exit 101
  fi
  echo "CRASH_AMPLIFICATION share=$share states_checked_here=1 stopped_early=true weights=$weights"
  exit 0
fi
index="${share%/*}" count="${share#*/}" judged=0 reused=0
# 带 IMPORT、停下文件已经在（被停之后先导入的那一趟）：一块都不领，只导入
stopped_before_judging=0
[[ -n "$import" && -n "$stop_file" && -e "$stop_file" ]] && stopped_before_judging=1
# 先录再判（选段没设）：没有停下文件在时，先把自己领的观察者块录进自己的库（停下文件在就停在块边上，一块也不录）
if [[ -z "${SINGLEFS_CRASH_AMPLIFICATION_STAGE:-}" ]] && (( ! stopped_before_judging )); then record_my_observers; fi
for (( block = 0; block < total; block++ )); do
  (( stopped_before_judging )) && break
  [[ "$(owner_of_block "$block")" == "$index" ]] || continue
  [[ "$block" == "${STUB_DROP_BLOCK:-}" ]] && continue
  if grep -qx "$block" "$library/blocks"; then reused=$((reused + 1)); else
    if [[ "${STUB_OBSERVERS:-}" == 1 ]] && ! grep -qx "$block" "$library/observer-records" 2>/dev/null; then
      echo "thread 'the_case' panicked: 观察者在块 $block 的录入段那一块不在库里 $library：先跑录入段（假的用例）"
      echo "test result: FAILED"
      exit 101
    fi
    echo "$block" >>"$library/blocks"; judged=$((judged + 1))
  fi
done
fail_if_told
if [[ -z "$import" ]]; then
  echo "$summary_line share=$share states_checked_here=$judged reused=$reused weights=$weights"
  exit 0
fi
if [[ ! -f "$import/CURRENT" ]]; then
  echo "thread 'the_case' panicked: 要导入的库 $import 不在"
  echo "test result: FAILED"
  exit 101
fi
imported=$(( $(sort -u "$library/blocks" "$import/blocks" | wc -l) - $(sort -u "$library/blocks" | wc -l) ))
sort -u -o "$library/blocks" "$library/blocks" "$import/blocks"
unjudged=$(( total - $(wc -l <"$library/blocks") ))
echo "$summary_line share=$share states_checked_here=$judged reused=$reused imported=$imported unjudged_blocks=$unjudged red=0 weights=$weights"
if (( unjudged > 0 && stopped_before_judging )); then
  echo "thread 'the_case' panicked: 叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE）：$unjudged 块还没判，判过的在库里"
  echo "test result: FAILED"
  exit 101
fi
if (( unjudged > 0 )); then
  echo "thread 'the_case' panicked: 导入之后对账不为空：$unjudged 块没人判"
  echo "test result: FAILED"
  exit 101
fi
exit 0
STUB
  cat >"$work/local-run.sh" <<'STUB'
#!/usr/bin/env bash
# 假的本机那一趟（自证用）：调法同 crash_amplification_run_on_this_host（<share> <库目录> <日志> [<附加环境 键=值>…]），跑假的用例、写日志。
# 照点名失败：带停下文件的那一趟看 STUB_WARM_FAILURE，带 IMPORT 的导入那一趟不失败，录入段那一趟看 STUB_LOCAL_RECORD_FAILURE，别的（分份那一趟）看 STUB_LOCAL_FAILURE
share="$1" library="$2" log="$3"
shift 3
# 带 IMPORT 的先认（被停之后先导入的那一趟 IMPORT 与停下文件都带）；分份那一趟（两样都不带）跑完建 <调用记录>.local-share-done
share_run=0
case " $* " in
  *" SINGLEFS_CRASH_AMPLIFICATION_IMPORT="*) failure="" ;;
  *" SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE="*) failure="${STUB_WARM_FAILURE:-}" ;;
  *" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "*) failure="${STUB_LOCAL_RECORD_FAILURE:-}" ;;
  *) failure="${STUB_LOCAL_FAILURE:-}"; share_run=1 ;;
esac
# 这一趟的 cargo test 参数（驱动交的 CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_TEST_ARGUMENTS）交给假的用例：照崩溃点重放用例的样子造的那种据此判标了 ignore 的跑不跑
env "$@" STUB_HOST=local STUB_SENT=no STUB_FAILURE="$failure" STUB_DROP_BLOCK= STUB_PLACEMENT=ok STUB_TEST_ARGUMENTS="${CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_TEST_ARGUMENTS:-}" \
  SINGLEFS_CRASH_AMPLIFICATION_SHARE="$share" SINGLEFS_CRASH_AMPLIFICATION_LIBRARY="$library" bash "${STUB_CASE:?}" >"$log" 2>&1
case_exit=$?
(( share_run )) && : >"${STUB_CALLS:?}.local-share-done"
exit "$case_exit"
STUB
  cat >"$work/multi-host-run.sh" <<'STUB'
#!/usr/bin/env bash
# 假的 multi-host-run.sh（自证用）：参数与它相同；不拷树、不连第二台，在 STUB_PEER_TREE 这棵假树根下跑假的用例，--send 的目录先拷进树，
# --fetch 点名的目录跑完拷进 MULTI_HOST_RUN_FETCH_DIRECTORY（跑红了也取回，与真的一样）。
# STUB_PEER_EXIT=255（分份那一趟）/ STUB_PEER_PLACEMENT=unreachable（排计划那一趟）：连不上，什么都不跑、不取回，退 255；
# STUB_PEER_FAILURE=plain|device：分份那一趟判完照点名失败；STUB_PEER_DROP_BLOCK=<序号>：漏判那一块；STUB_PEER_NO_FETCH=1：跑完不取回；
# STUB_PEER_PLACEMENT=device|algorithm|unrecognized：排计划那一趟照点名失败；STUB_PEER_RATE：排计划那一趟报的速度（默认 50）。
# 真的第二台那一趟是新起的 ssh 会话、不继承本机的环境：假的照此清掉 GPU_COMPUTATION_CLASSES、选段 SINGLEFS_CRASH_AMPLIFICATION_STAGE 与往引擎送的变量，第二台只看得见命令里送过去的那一份。
# 命令最前面可以是一段或几段 bash -c <脚本> <名> <变量> <树内相对路径>（双机驱动的 peer_library_path_script、peer_tree_path_script）：假的在假树根里逐段真跑那段脚本
# （拼 $PWD；按段登记文件不在退 2）取变量值，脚本退非 0 就当那一趟命令退 2、什么都不跑
fetch_name="" send_sources=() send_names=()
while (($# > 0)); do
  case "$1" in
    --peer) shift ;;
    --label) shift 2 ;;
    --memory-cap) echo "memory-cap=$2" >>"${STUB_CALLS:?}.memory-cap"; shift 2 ;;
    --fetch) fetch_name="$2"; shift 2 ;;
    --send) send_sources+=("$2"); send_names+=("$3"); shift 3 ;;
    --) shift; break ;;
    *) echo "假的 multi-host-run.sh：认不得参数 $1"; exit 2 ;;
  esac
done
tree_path_scripts=() tree_path_names=() tree_path_variables=() tree_path_relatives=() tree_path_value=""
while [[ "${1:-}" == bash && "${2:-}" == -c ]]; do
  tree_path_scripts+=("$3"); tree_path_names+=("$4"); tree_path_variables+=("$5"); tree_path_relatives+=("$6")
  shift 6
done
[[ "${1:-}" == env ]] && shift
assignments=()
while (($# > 0)) && [[ "$1" == *=* ]]; do assignments+=("$1"); shift; done
placement=0
case " ${assignments[*]} " in *" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 "*) placement=1 ;; esac
# 第二台共用编译目录的守卫：调用方环境里的 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY 与命令里 CARGO_TARGET_DIR 的值记进 <调用记录>.shared-target（peer-build-guarded 那一格读）
cargo_target=unset
for assignment in "${assignments[@]}"; do [[ "$assignment" == CARGO_TARGET_DIR=* ]] && cargo_target="${assignment#CARGO_TARGET_DIR=}"; done
echo "placement=$placement shared_target=[${MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY-unset}] cargo_target=[$cargo_target]" >>"${STUB_CALLS:?}.shared-target"
# 送过去的命令是 bash scripts/hosts/capped.sh <线程数> cargo …：线程数记进 <调用记录>.threads（线程数的缺省那几格读）
[[ "${1:-}" == bash && "${2:-}" == */capped.sh ]] && echo "placement=$placement threads=${3:-}" >>"${STUB_CALLS:?}.threads"
# 点名了用例的那一趟（-- 之后带 --exact）：--include-ignored 带没带记进 <调用记录>.test-arguments（点名的用例标 ignore 时不带就一条都不跑）
case " $* " in *" --exact "*) include_ignored=no; [[ " $* " == *" --include-ignored "* ]] && include_ignored=yes; echo "peer placement=$placement include_ignored=$include_ignored" >>"${STUB_CALLS:?}.test-arguments" ;; esac
if { ((placement)) && [[ "${STUB_PEER_PLACEMENT:-ok}" == unreachable ]]; } || { ((!placement)) && [[ "${STUB_PEER_EXIT:-}" == 255 ]]; }; then
  echo "peer unreachable placement=$placement" >>"${STUB_CALLS:?}"
  echo "multi-host-run.sh（假的）：第二台连不上"
  exit 255
fi
tree="${STUB_PEER_TREE:?}"
mkdir -p -- "$tree" || exit 3
sent=no
for index in "${!send_sources[@]}"; do
  rm -rf -- "${tree:?}/${send_names[index]:?}"
  cp -a -- "${send_sources[index]}" "$tree/${send_names[index]}" || exit 3
  [[ "${send_names[index]}" == crash-amplification-library ]] && sent=yes
done
for index in "${!tree_path_variables[@]}"; do
  tree_path_value="$(cd "$tree" && bash -c "${tree_path_scripts[index]}" "${tree_path_names[index]}" "${tree_path_variables[index]}" "${tree_path_relatives[index]}" printenv "${tree_path_variables[index]}")" || exit 2
  assignments+=("${tree_path_variables[index]}=$tree_path_value")
done
( cd "$tree" && env -u GPU_COMPUTATION_CLASSES -u SINGLEFS_CRASH_AMPLIFICATION_STAGE -u SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY -u SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES -u SINGLEFS_CRASH_LIBRARY_WRITE_ONLY "${assignments[@]}" STUB_HOST=peer STUB_SENT="$sent" STUB_FAILURE="${STUB_PEER_FAILURE:-}" STUB_DROP_BLOCK="${STUB_PEER_DROP_BLOCK:-}" \
    STUB_PLACEMENT="${STUB_PEER_PLACEMENT:-ok}" STUB_RATE="${STUB_PEER_RATE:-50}" STUB_TEST_ARGUMENTS="$*" bash "${STUB_CASE:?}" )
case_exit=$?
# STUB_PEER_WAIT_FOR_TERM=1（分份那一趟）：演第二台那条命令还在跑——判完自己那一份之后建 <调用记录>.peer-running、等驱动转来的 TERM（至多 10 秒）；
#   收到 TERM 照真的 multi-host-run.sh 先取回 --fetch 点名的库再退 143（调用记录里记 peer term）；等不到记 peer no-term、不取回、退 1
if [[ "${STUB_PEER_WAIT_FOR_TERM:-0}" == 1 && "$placement" == 0 ]]; then
  peer_term=0 waited=0
  trap 'peer_term=1' TERM
  : >"${STUB_CALLS:?}.peer-running"
  while (( ! peer_term && waited < 100 )); do sleep 0.1; waited=$((waited + 1)); done
  if (( ! peer_term )); then echo "peer no-term" >>"$STUB_CALLS"; exit 1; fi
  echo "peer term" >>"$STUB_CALLS"
  case_exit=143
fi
if [[ -n "$fetch_name" && "${STUB_PEER_NO_FETCH:-0}" != 1 && -e "$tree/$fetch_name" ]]; then
  mkdir -p -- "${MULTI_HOST_RUN_FETCH_DIRECTORY:?}"
  rm -rf -- "${MULTI_HOST_RUN_FETCH_DIRECTORY:?}/${fetch_name:?}"
  cp -a -- "$tree/$fetch_name" "$MULTI_HOST_RUN_FETCH_DIRECTORY/$fetch_name" || exit 3
fi
exit "$case_exit"
STUB
  cat >"$work/peer-host-lib.sh" <<'STUB'
# 假的 peer-host-lib.sh（自证用，被 source）：STUB_QUIESCE=configuration-fails|quiesce-fails 照点名失败；每次调记一笔进 STUB_CALLS
# 配置里的复原命令只记一笔 restore（这里不该调它，弄坏开关 restore-after-run 下才调）
peer_host_load_configuration() { echo "load-configuration" >>"${STUB_CALLS:?}"; QUIESCE_START_COMMAND='echo restore >>"$STUB_CALLS"'; [[ "${STUB_QUIESCE:-ok}" != configuration-fails ]]; }
peer_host_quiesce() { echo "quiesce" >>"${STUB_CALLS:?}"; [[ "${STUB_QUIESCE:-ok}" != quiesce-fails ]]; }
peer_host_quiesce_left() { echo "quiesce-left" >>"${STUB_CALLS:?}"; }
STUB
  printf '%s\n' 'GPU_CARDS_BY_PRIORITY="local:0:15G"' >"$work/multi-host-without-quiesced.env"
  # 退役的私有显卡配置（SINGLEFS_GPU_CARDS_CONFIG）指到一份写了别的卡与一类计算的假文件：读到它就是读错了（
  #   gpu-cards-from-multi-host-configuration、quiesced-without-card-list；弄坏开关下照它读，也不会去读这台机器真的私有配置）
  printf '%s\n' 'GPU_CARDS_BY_PRIORITY="local:1:3G peer:1:3G"' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="local:1:3G peer:1:3G"' "GPU_COMPUTATION_CLASSES='crash_judge_gpu'" >"$work/retired-gpu-cards.env"
  # 里层读的多机配置指到不在的文件（两个开关都关、一份卡都没写），库与日志都在临时目录下，本机、第二台与清场都换成上面那几份假的
  export SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" SINGLEFS_GPU_CARDS_CONFIG="$work/retired-gpu-cards.env" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home"
  export SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs"   # 上次红的清单：各格的里层都写这里，不写共用的那一份（last-run-list 那一格另给自己的目录）
  export CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_PEER_RUN="$work/multi-host-run.sh" \
    CRASH_AMPLIFICATION_TWO_HOSTS_PEER_HOST_LIB="$work/peer-host-lib.sh" STUB_CASE="$work/case.sh"

  run_cell() { # <格名> [--without-library] [--term-when <文件>]… [<环境 键=值>…] -- <参数…>：起一趟里层（标签是格名、库在格目录下，带 --library <格目录>/library；
    #           给了 --without-library 就不带 --library，库落默认库；给了 --term-when，里层放后台起，等给的几份文件都在（至多 10 秒）就只给它自己（$!）发 TERM），
    #           输出落 <格目录>/output、调用记录落 <格目录>/calls，退出码放 cell_exit
    local name="$1" environment=() library_arguments=() term_when=() cell_command=() driver_pid="" waited=0 term_file="" term_ready=0
    shift
    cell_directory="$work/$name"
    library_arguments=(--library "$cell_directory/library")
    while (($# > 0)) && [[ "$1" != -- ]]; do
      if [[ "$1" == --without-library ]]; then library_arguments=()
      elif [[ "$1" == --term-when ]]; then term_when+=("$2"); shift
      else environment+=("$1"); fi
      shift
    done
    shift
    mkdir -p "$cell_directory"
    : >"$cell_directory/calls"; : >"$cell_directory/calls.stage"; : >"$cell_directory/calls.library"; : >"$cell_directory/calls.record-share"; : >"$cell_directory/calls.test-threads"
    cells=$((cells + 1))
    cell_command=(env -u SINGLEFS_CRASH_AMPLIFICATION_IMPORT -u SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE -u SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY -u SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS -u GPU_COMPUTATION_CLASSES -u GPU_CARDS_BY_PRIORITY
      -u SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY -u SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES -u SINGLEFS_CRASH_LIBRARY_WRITE_ONLY
      -u CRASH_AMPLIFICATION_PEER_THREADS -u CRASH_AMPLIFICATION_BUILD_THREADS -u SINGLEFS_THREAD_CAP
      STUB_LOCAL_FAILURE= STUB_WARM_FAILURE= STUB_PEER_EXIT= STUB_PEER_FAILURE= STUB_PEER_DROP_BLOCK= STUB_PEER_NO_FETCH= STUB_PEER_PLACEMENT= STUB_PEER_RATE= STUB_QUIESCE= STUB_RECORD_MARKER=
      STUB_CASE_SHAPE= STUB_PEER_WAIT_FOR_TERM= STUB_JUDGE_NOT_RUN= STUB_OBSERVERS= STUB_LOCAL_RECORD_FAILURE= STUB_BINARY_RECORD= STUB_BINARY_JUDGE= STUB_BINARY_IMPORT=
      STUB_CALLS="$cell_directory/calls" STUB_PEER_TREE="$cell_directory/peer-tree" CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED="$cell_directory/stop-decided"
      SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge "${environment[@]+"${environment[@]}"}"
      bash "$two_hosts_script" "$@" "${library_arguments[@]+"${library_arguments[@]}"}" --label "$name")
    if (( ${#term_when[@]} == 0 )); then
      "${cell_command[@]}" >"$cell_directory/output" 2>&1
      cell_exit=$?
      return 0
    fi
    "${cell_command[@]}" >"$cell_directory/output" 2>&1 &
    driver_pid=$!
    while (( waited < 100 )) && kill -0 "$driver_pid" 2>/dev/null; do
      term_ready=1
      for term_file in "${term_when[@]}"; do [[ -e "$term_file" ]] || term_ready=0; done
      (( term_ready )) && break
      sleep 0.1; waited=$((waited + 1))
    done
    kill -TERM "$driver_pid" 2>/dev/null
    wait "$driver_pid"; cell_exit=$?
  }
  cell_failed() { # <格名> <判错的一句> <出路>
    echo "  ✗ 自证 $1：$2（退 $cell_exit；调用记录「$(tr '\n' ';' <"$work/$1/calls")」）" >&2
    echo "     → 怎么办：$3" >&2
    tail -n 12 "$work/$1/output" | sed 's/^/       | /' >&2
    failures=$((failures + 1))
  }
  output_has() { grep -qE -- "$2" "$work/$1/output"; }   # <格名> <正则>
  calls_have() { grep -qE -- "$2" "$work/$1/calls"; }    # <格名> <正则>
  last_call_is() { [[ "$(tail -n 1 "$work/$1/calls")" =~ $2 ]]; }   # <格名> <正则>
  # <格名>：转打的 CRASH_AMPLIFICATION 行里恰好一行带 import_run=true，就是导入那一趟的那一行（假的用例导入时报 imported=）
  only_the_import_line_is_marked() {
    [[ "$(grep -c 'import_run=true' "$work/$1/output")" == 1 ]] && output_has "$1" '^CRASH_AMPLIFICATION share=0/2 .*imported=[0-9]+ .*import_run=true$'
  }

  # ── 分份、取回、导入、对账
  run_cell both-hosts-green -- some-target
  if [[ "$cell_exit" != 0 ]] || ! calls_have both-hosts-green '^local share=0/2 import=no ' || ! calls_have both-hosts-green '^peer share=1/2 import=no ' \
     || ! last_call_is both-hosts-green '^local share=0/2 import=yes ' || ! output_has both-hosts-green 'unjudged_blocks=0 red=0' \
     || ! only_the_import_line_is_marked both-hosts-green; then
    cell_failed both-hosts-green "本机领 0/2、第二台领 1/2、取回之后本机带 IMPORT 导入、对账为空，应当退 0；转打的行里只有导入那一行带 import_run=true" \
      "看主路径：第二台那一趟的 SINGLEFS_CRASH_AMPLIFICATION_SHARE 是 1/2，取回之后导入那一趟带 SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库>，转打经 print_the_import_lines"
  fi
  # 单独起这个驱动（选段没设成 judge）：分着录（文件头「步骤」①②）——本机先跑录入段（.stage 里本机第一笔是 local stage=record），之后本机每一趟都是 stage=judge；
  #   第二台自己先录再判（选段不设，假的记 unset），一趟 stage=judge 都没有。本机与第二台的 .stage 各自按台看（两台同时开跑，笔与笔之间的先后不定）
  stage_lines_of() { grep "^$2 " "$work/$1/calls.stage"; }   # <格名> <local|peer>
  run_cell records-when-not-judging SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target
  if [[ "$cell_exit" != 0 ]] || [[ "$(stage_lines_of records-when-not-judging local | head -n 1)" != "local stage=record" ]] \
     || [[ "$(stage_lines_of records-when-not-judging local | tail -n +2 | grep -cv ' stage=judge$')" != 0 ]] \
     || [[ "$(stage_lines_of records-when-not-judging peer | sort -u | tr '\n' ';')" != "peer stage=unset;" ]]; then
    cell_failed records-when-not-judging "单独起（选段没设成 judge）应当先在本机跑录入段、之后本机每一趟只跑判段，第二台自己先录再判（选段不设）、一趟 stage=judge 都没有，退 0（退 $cell_exit；.stage「$(tr '\n' ';' <"$work/records-when-not-judging/calls.stage")」）" \
      "看 mkdir -p 建日志目录之后「分着录」record_split 那一段与 peer_environment 后面选段那一段：分着录时第二台不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge"
  fi
  # 判段没跑（文件头「输出」：没编进 gpu 特性的二进制遇到 ENABLE_GPU=1，每一趟判段都打 CRASH_AMPLIFICATION_NOT_RUN step=judge 行）：
  #   那一行只从导入那一趟转打一次（本机分份那一趟也打了，不转），退 0
  run_cell judge-not-run-forwarded-once STUB_JUDGE_NOT_RUN=1 -- some-target
  if [[ "$cell_exit" != 0 ]] || [[ "$(grep -c '^CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=stub reason=' "$work/judge-not-run-forwarded-once/output")" != 1 ]]; then
    cell_failed judge-not-run-forwarded-once "判段没跑的那一种（每一趟判段都打 CRASH_AMPLIFICATION_NOT_RUN step=judge 行）应当只从导入那一趟转打那一行、恰好一行、退 0（实转 $(grep -c '^CRASH_AMPLIFICATION_NOT_RUN step=judge ' "$work/judge-not-run-forwarded-once/output") 行）" \
      "看 print_the_import_lines：只转 import.log 里行首 CRASH_AMPLIFICATION_NOT_RUN step=judge 的那几行"
  fi
  # 崩溃枚举用例经判定库两段跑的（崩溃点重放双机那几条）录入段只打 CRASH_LIBRARY_RECORDED、没有崩溃放量那一行：照样算录入成了，之后本机与第二台跑判段、退 0，那一行转打出来
  run_cell records-library-marker STUB_RECORD_MARKER=library SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target
  if [[ "$cell_exit" != 0 ]] || [[ "$(stage_lines_of records-library-marker local | head -n 1)" != "local stage=record" ]] \
     || ! grep -q '^peer stage=unset$' "$work/records-library-marker/calls.stage" \
     || ! grep -q '^CRASH_LIBRARY_RECORDED class=crash_state_enumeration ' "$work/records-library-marker/output"; then
    cell_failed records-library-marker "录入段只打 CRASH_LIBRARY_RECORDED 行（崩溃枚举用例经判定库两段跑的样子）应当照样算录入成了：之后本机跑判段、第二台先录再判、退 0、那一行转打出来（退 $cell_exit；.stage「$(tr '\n' ';' <"$work/records-library-marker/calls.stage")」）" \
      "看共用库 crash_amplification_record_on_this_host 认录入行的那一句：CRASH_AMPLIFICATION_RECORDED 与 CRASH_LIBRARY_RECORDED 两种都算"
  fi
  # 录入段与判段用的库（.library）：<格名> <本机每一趟该收到的库目录>。退 0；本机第一笔是本机录入段那一趟；本机的每一笔（录入段、分份、导入，至少 3 笔）都是那个库目录；
  #   第二台的每一笔（至少 1 笔，先录再判、选段不设）照旧是它树里的 crash-amplification-library，交的是绝对路径 <假树根>/crash-amplification-library
  libraries_are() {
    local record="$work/$1/calls.library" local_calls peer_calls
    local_calls="$(grep -c '^local ' "$record")"; peer_calls="$(grep -c '^peer ' "$record")"
    [[ "$cell_exit" == 0 ]] && [[ "$(grep '^local ' "$record" | head -n 1)" == "local stage=record library=$2" ]] && (( local_calls >= 3 && peer_calls >= 1 )) \
      && [[ "$(grep -cxF -e "local stage=record library=$2" -e "local stage=judge library=$2" "$record")" == "$local_calls" ]] \
      && [[ "$(grep -cxF "peer stage=unset library=$work/$1/peer-tree/crash-amplification-library" "$record")" == "$peer_calls" ]]
  }
  # 给了 --library（崩溃点重放双机那几条由 crash-point-replay-shard-run.sh 给 <家目录>/crash-state-verdicts 的样子）：单独起时先跑的录入段写进给的那个库，
  #   之后本机每一趟判段也读它，转打的 CRASH_LIBRARY_RECORDED 行的 library= 是它
  run_cell records-into-the-given-library STUB_RECORD_MARKER=library SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target
  if ! libraries_are records-into-the-given-library "$work/records-into-the-given-library/library" \
     || ! grep -qxF "CRASH_LIBRARY_RECORDED class=crash_state_enumeration item=stub::full manifest=0000000000000000 library=$work/records-into-the-given-library/library" "$work/records-into-the-given-library/output"; then
    cell_failed records-into-the-given-library "给了 --library，录入段与之后本机每一趟判段（分份、导入）都应当用给的那个库目录、第二台照旧用它树里的 crash-amplification-library、退 0（.library「$(tr '\n' ';' <"$work/records-into-the-given-library/calls.library")」）" \
      "看主路径 local_library 那几行：--library 的覆盖要在 crash_amplification_record_on_this_host 之前，录入段与 run_local_share 收同一个 \$local_library"
  fi
  # 没给 --library：录入段与本机每一趟判段都用默认库 <家目录>/crash-state-verdicts（家目录是自证设的 SINGLEFS_CRASH_AMPLIFICATION_HOME）
  run_cell records-into-the-default-library-when-not-given --without-library STUB_RECORD_MARKER=library SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target
  libraries_are records-into-the-default-library-when-not-given "$work/home/crash-state-verdicts" \
    || cell_failed records-into-the-default-library-when-not-given "没给 --library，录入段与之后本机每一趟判段都应当用默认库 $work/home/crash-state-verdicts、第二台照旧用它树里的 crash-amplification-library、退 0（.library「$(tr '\n' ';' <"$work/records-into-the-default-library-when-not-given/calls.library")」）" \
      "看主路径 local_library 那一行与 crash-amplification-lib.sh 的 crash_amplification_default_library"
  # 带 --given-states（经给定状态引擎判的测试目标）：本机不起录入段，.stage 里一笔 record 都没有；第二台至少一笔、每一笔都不是 stage=judge
  #   （引擎自己先录再判，不设选段，假的记 unset；第二台只判就读不到本机这一趟才录的清单）
  run_cell given-states-without-record SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target --given-states
  if [[ "$cell_exit" != 0 ]] || grep -q ' stage=record$' "$work/given-states-without-record/calls.stage" \
     || ! grep -q '^peer stage=' "$work/given-states-without-record/calls.stage" || grep -q '^peer stage=judge$' "$work/given-states-without-record/calls.stage"; then
    cell_failed given-states-without-record "带 --given-states 应当一趟录入段都不起、第二台至少跑一趟而且那一笔不是 stage=judge（先录再判）、退 0（退 $cell_exit；.stage「$(tr '\n' ';' <"$work/given-states-without-record/calls.stage")」）" \
      "看选段那一段的条件：given_states 是 1 时不调 crash_amplification_record_on_this_host；看 peer_environment 后面选段那一段：given_states 是 1 时不给第二台设 SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge"
  fi
  mkdir -p "$work/library-snapshot-sent/library"
  : >"$work/library-snapshot-sent/library/CURRENT"; echo 1 >"$work/library-snapshot-sent/library/blocks"
  run_cell library-snapshot-sent -- some-target
  if [[ "$cell_exit" != 0 ]] || ! calls_have library-snapshot-sent '^peer share=1/2 import=no placement=0 stop=no sent=yes ' \
     || ! output_has library-snapshot-sent '^CRASH_AMPLIFICATION share=1/2 states_checked_here=2 reused=1 '; then
    cell_failed library-snapshot-sent "本机库里已有一块，应当照快照送给第二台（--send）、第二台复用那一块、退 0" \
      "看主路径 \$local_library/CURRENT 在时 cp -a 照快照、peer_arguments 加 --send 那一段"
  fi
  mkdir -p "$work/library-snapshot-fails/library" "$work/home/runs/library-snapshot-fails"
  : >"$work/library-snapshot-fails/library/CURRENT"; : >"$work/library-snapshot-fails/library/blocks"
  : >"$work/home/runs/library-snapshot-fails/library-for-peer"   # 占住快照的落点：一份同名的普通文件，cp -a 拷不过去
  run_cell library-snapshot-fails -- some-target
  [[ "$cell_exit" == 2 ]] \
    || cell_failed library-snapshot-fails "拷不了给第二台的库快照，应当退 2" "看主路径 cp -a 那一行：拷不成就退 2，不带着半份快照去跑"
  run_cell local-share-fails STUB_LOCAL_FAILURE=plain -- some-target
  if [[ "$cell_exit" != 101 ]] || ! last_call_is local-share-fails '^local share=0/2 import=yes ' || ! only_the_import_line_is_marked local-share-fails; then
    cell_failed local-share-fails "本机那一份退 101、认不出原因，应当先把第二台取回的库导进本机库（那一行带 import_run=true）、再原样退 101" \
      "看「本机那一份退」那一段：取回的库在就先导入、经 print_the_import_lines 转打，退出码经 crash_amplification_exit_code_of_a_failed_run 照 share-0.log 分类"
  fi
  run_cell local-share-card-unusable STUB_LOCAL_FAILURE=device -- some-target
  [[ "$cell_exit" == 78 ]] \
    || cell_failed local-share-card-unusable "本机那一份日志里有卡用不了的原话，应当退 78" "看「本机那一份退」那一段与 crash-amplification-lib.sh 的 crash_amplification_failure_kind 认不认 GPU_CARD_NOT_USABLE"
  # 给定状态表的判器借不到显存：引擎打 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 那一行后报错（cargo 退 101），是运行条件不满足
  run_cell local-share-given-states-run-condition STUB_LOCAL_FAILURE=given-states -- some-target
  [[ "$cell_exit" == 78 ]] \
    || cell_failed local-share-given-states-run-condition "本机那一份日志里有给定状态表借不到显存那一行（CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET），应当退 78" \
      "看 crash-amplification-lib.sh 的 crash_amplification_failure_kind 的 device 那一组认不认 CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX 起头的那一行"
  run_cell local-share-gpu-memory STUB_LOCAL_FAILURE=gpu-memory -- some-target
  { [[ "$cell_exit" == 245 ]] && output_has local-share-gpu-memory '显存不够（那一趟退了非 0'; } \
    || cell_failed local-share-gpu-memory "本机那一份经内存包装退 245（显存不够），应当原样退 245、打显存不够的出路" \
      "看「本机那一份退」那一段与 two_hosts_exit_code_is_explained：245 走 crash-amplification-lib.sh 的 crash_amplification_explain_exit_code 那一支"
  run_cell peer-share-gpu-memory STUB_PEER_FAILURE=gpu-memory -- some-target
  { [[ "$cell_exit" == 245 ]] && output_has peer-share-gpu-memory '显存不够（那一趟退了非 0'; } \
    || cell_failed peer-share-gpu-memory "第二台那一份经那边的内存包装退 245（显存不够），应当原样退 245、打显存不够的出路" \
      "看「第二台那一份退」那一段与 two_hosts_exit_code_is_explained：245 不是 255，照 share-1.log 分类之后原样退、走显存不够那一支"
  run_cell peer-unreachable STUB_PEER_EXIT=255 -- some-target
  [[ "$cell_exit" == 78 ]] \
    || cell_failed peer-unreachable "multi-host-run.sh 退 255（第二台连不上），应当退 78" "看「第二台那一份退」那一段：255 是第二台用不上，映射成 78"
  run_cell peer-share-card-unusable STUB_PEER_FAILURE=device -- some-target
  if [[ "$cell_exit" != 78 ]] || ! last_call_is peer-share-card-unusable '^local share=0/2 import=yes ' || ! only_the_import_line_is_marked peer-share-card-unusable; then
    cell_failed peer-share-card-unusable "第二台那一份判完卡用不了（退 101，库取回了），应当先导入它判过的块（那一行带 import_run=true）、再退 78" \
      "看「第二台那一份退」那一段：取回的库在就先导入、经 print_the_import_lines 转打，退出码照 share-1.log 分类"
  fi
  run_cell peer-library-not-fetched STUB_PEER_NO_FETCH=1 -- some-target
  [[ "$cell_exit" == 78 ]] \
    || cell_failed peer-library-not-fetched "第二台退 0 而库没取回，应当退 78" "看「第二台的库没取回」那一段：\$fetched_library 不在就退 78，不拿空的去导入"
  run_cell reconciliation-not-empty STUB_PEER_DROP_BLOCK=3 -- some-target
  if [[ "$cell_exit" != 101 ]] || ! output_has reconciliation-not-empty 'unjudged_blocks=1 '; then
    cell_failed reconciliation-not-empty "第二台漏判一块、导入之后对账剩 1 块，应当原样退导入那一趟的 101" \
      "看末尾导入那一趟：它退非 0 就照 import.log 分类退出，不报绿"
  fi
  # 观察者版本（文件头「观察者版本」）：单独起（选段空）本机先录入段，驱动从 record.log 里的 Running 行找到假二进制、取出里面观察者那几组的摘要记下；第二台漏判一块、
  #   导入那一趟红（101）：版本没变（含只有无关的 gpu_judge 组变了）原样退 101；变了退 78、写「两趟之间判器单元变了，重跑整趟」、不是假用例那句「先跑录入段」；
  #   录入段的二进制读不出表：说明一行、原样退 101；导入那一趟判绿而版本变了：退 0
  fake_observer_binary() { # <文件> <观察者那几组的键> <无关的 gpu_judge 组的键>
    { printf 'garbage-before\0'
      printf '# singlefs entry group keys\ntoolchain\trustc fake\ngroup\tgpu_judge\tkeyed\t%s\ngroup\tjudgement_of_the_crash_images\tkeyed\t%s\ngroup\tobserver_conclusion_composition\tkeyed\t%s\n' "$3" "$2" "$2"
      printf 'group\tobserver_code:tests/stub.rs:stub\tkeyed\t%s\nunit\tobserver_code:tests/stub.rs:stub\tfn\nwatched\ta.rs\n' "$2"
      printf 'trailer\0'; } >"$1"
  }
  mkdir -p "$work/observer-version-binaries"
  fake_observer_binary "$work/observer-version-binaries/recorded" key-one gpu-one
  fake_observer_binary "$work/observer-version-binaries/other-gpu-group" key-one gpu-two
  fake_observer_binary "$work/observer-version-binaries/other-observer" key-two gpu-one
  printf 'not-a-test-binary\n' >"$work/observer-version-binaries/without-table"
  local observer_binaries="$work/observer-version-binaries"
  run_cell observer-version-unrelated-group-keeps-the-exit SINGLEFS_CRASH_AMPLIFICATION_STAGE= STUB_PEER_DROP_BLOCK=3 \
    STUB_BINARY_RECORD="$observer_binaries/recorded" STUB_BINARY_IMPORT="$observer_binaries/other-gpu-group" -- some-target
  if [[ "$cell_exit" != 101 ]] || ! output_has observer-version-unrelated-group-keeps-the-exit 'unjudged_blocks=1 ' || output_has observer-version-unrelated-group-keeps-the-exit '判器单元变了' \
     || [[ ! -s "$work/home/runs/observer-version-unrelated-group-keeps-the-exit/observer-version" ]]; then
    cell_failed observer-version-unrelated-group-keeps-the-exit "观察者那几组没变、只有无关的 gpu_judge 组变了：导入那一趟红了应当原样退 101（不报判器单元变了），录入段之后记下了版本" \
      "看 two_hosts_observer_version_of_binary：只取 observer_code:… 、observer_conclusion_composition、judgement_of_the_crash_images 与工具链行；记下的版本写在 <日志目录>/observer-version"
  fi
  run_cell observer-version-changed-exits-78 SINGLEFS_CRASH_AMPLIFICATION_STAGE= STUB_PEER_DROP_BLOCK=3 \
    STUB_BINARY_RECORD="$observer_binaries/recorded" STUB_BINARY_IMPORT="$observer_binaries/other-observer" -- some-target
  if [[ "$cell_exit" != 78 ]] || ! output_has observer-version-changed-exits-78 '两趟之间判器单元变了，重跑整趟' || output_has observer-version-changed-exits-78 '先跑录入段' \
     || ! last_call_is observer-version-changed-exits-78 '^local share=0/2 import=yes '; then
    cell_failed observer-version-changed-exits-78 "录入段之后导入那一趟的二进制里观察者版本变了而导入那一趟红了，应当退 78、出路写「两趟之间判器单元变了，重跑整趟」、输出里不出现「先跑录入段」" \
      "看末尾导入那一趟之后的 two_hosts_observer_version_changed：对不上而 import_exit 非 0 就退 78；录入段成了之后经 two_hosts_note_observer_version 记下版本"
  fi
  run_cell observer-version-unreadable-keeps-the-exit SINGLEFS_CRASH_AMPLIFICATION_STAGE= STUB_PEER_DROP_BLOCK=3 \
    STUB_BINARY_RECORD="$observer_binaries/without-table" STUB_BINARY_IMPORT="$observer_binaries/other-observer" -- some-target
  if [[ "$cell_exit" != 101 ]] || ! output_has observer-version-unreadable-keeps-the-exit '观察者版本取不到，导入那一趟不核版本' \
     || [[ -e "$work/home/runs/observer-version-unreadable-keeps-the-exit/observer-version" ]]; then
    cell_failed observer-version-unreadable-keeps-the-exit "录入段的二进制里读不出判器键表：应当打一行说明、不记版本、导入那一趟红了原样退 101" \
      "看 two_hosts_note_observer_version：取不到时只打说明，不写 observer-version"
  fi
  run_cell observer-version-changed-import-green-stays-green SINGLEFS_CRASH_AMPLIFICATION_STAGE= \
    STUB_BINARY_RECORD="$observer_binaries/recorded" STUB_BINARY_IMPORT="$observer_binaries/other-observer" -- some-target
  if [[ "$cell_exit" != 0 ]] || output_has observer-version-changed-import-green-stays-green '重跑整趟'; then
    cell_failed observer-version-changed-import-green-stays-green "版本变了而导入那一趟判绿：应当退 0、不退 78（对账为空、0 红，结论有效）" \
      "看末尾导入那一趟之后：只在 import_exit 非 0 时退 78，判绿只打一行说明"
  fi
  # 分份途中被停（文件头「被停」）：本机那一份跑完、第二台那一份判完还没返回（假的 multi-host-run.sh 等 TERM）时只给驱动发 TERM ⇒
  #   驱动把 TERM 转给第二台那一趟（调用记录里有 peer term）、第二台照真的那样取回库再退，驱动带 IMPORT 跑一趟本机 0/2 把取回的块导进本机库、退 143
  run_cell term-imports-the-fetched-library --term-when "$work/term-imports-the-fetched-library/calls.peer-running" \
    --term-when "$work/term-imports-the-fetched-library/calls.local-share-done" STUB_PEER_WAIT_FOR_TERM=1 -- some-target
  local term_fetched_blocks="$work/home/runs/term-imports-the-fetched-library/fetched/crash-amplification-library/blocks"
  if [[ "$cell_exit" != 143 ]] || ! calls_have term-imports-the-fetched-library '^peer term$' \
     || ! calls_have term-imports-the-fetched-library '^local share=0/2 import=yes ' || [[ ! -s "$term_fetched_blocks" ]] \
     || [[ -n "$(LC_ALL=C comm -13 <(LC_ALL=C sort -u "$work/term-imports-the-fetched-library/library/blocks" 2>/dev/null) <(LC_ALL=C sort -u "$term_fetched_blocks"))" ]]; then
    cell_failed term-imports-the-fetched-library "分份途中给驱动发 TERM：应当把 TERM 转给第二台那一趟（调用记录里有 peer term）、等它取回库，再带 IMPORT 跑一趟本机 0/2 把取回的块导进本机库、退 143（本机库「$(tr '\n' ' ' <"$work/term-imports-the-fetched-library/library/blocks" 2>/dev/null)」，取回的「$(tr '\n' ' ' <"$term_fetched_blocks" 2>/dev/null)」）" \
      "看主路径分份那一段的 trap … TERM、forward_term_to_own_run 与 wait_for_own_run，和收到 TERM 之后 two_hosts_term_received 那一段：fetched 库的 CURRENT 在就经 run_local_share 带 IMPORT 导入再 exit 143"
  fi
  # 照崩溃点重放双机那几条用例的样子（假的用例 STUB_CASE_SHAPE=crash-case：标 ignore、录入段只打 CRASH_LIBRARY_RECORDED）、单独起、点名用例：本机录入段要真跑
  #   （驱动给本机那一趟的 cargo test 参数带 --include-ignored）、CRASH_LIBRARY_RECORDED 那一行要算录入成了，之后两台判段、导入、对账为空、退 0，本机库里 6 块齐
  run_cell crash-case-shaped-records-and-imports STUB_CASE_SHAPE=crash-case SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target --test-filter the_full_case
  if [[ "$cell_exit" != 0 ]] || [[ "$(stage_lines_of crash-case-shaped-records-and-imports local | head -n 1)" != "local stage=record" ]] \
     || calls_have crash-case-shaped-records-and-imports ' ignored-not-run$' || grep -q '^peer stage=judge$' "$work/crash-case-shaped-records-and-imports/calls.stage" \
     || ! grep -q '^CRASH_LIBRARY_RECORDED class=crash_state_enumeration ' "$work/crash-case-shaped-records-and-imports/output" \
     || output_has crash-case-shaped-records-and-imports '^CRASH_AMPLIFICATION_RECORDED ' \
     || ! calls_have crash-case-shaped-records-and-imports '^peer share=1/2 import=no ' || ! last_call_is crash-case-shaped-records-and-imports '^local share=0/2 import=yes ' \
     || [[ "$(LC_ALL=C sort -u "$work/crash-case-shaped-records-and-imports/library/blocks" 2>/dev/null | wc -l)" != 6 ]]; then
    cell_failed crash-case-shaped-records-and-imports "照崩溃点重放用例的样子造的用例（标 ignore、录入段只打 CRASH_LIBRARY_RECORDED）给了 --test-filter：本机录入段应当真跑（不记 ignored-not-run）、那一行算录入成了，之后第二台（先录再判、选段不设）领 1/2、本机带 IMPORT 导入、本机库 6 块齐、退 0（.stage「$(tr '\n' ';' <"$work/crash-case-shaped-records-and-imports/calls.stage")」）" \
      "看共用库 crash_amplification_prepare 拼的 cargo test 参数（test_filter 非空要带 --include-ignored）与 crash_amplification_record_on_this_host 认的录入行（CRASH_LIBRARY_RECORDED 也算）"
  fi

  # ── 观察者录入段按份分给两台（文件头「步骤」①②；假的用例 STUB_OBSERVERS=1：每一块有一份观察者录入块，谁领哪一块录哪一块，判段读自己领的块时库里没有录入块就红）
  #   本机录入段只录本机那一份（SHARE=0/2，.record-share 里本机一笔、带判段也带的权重）、第二台自己先录它那一份再判（选段不设）；两台各写各的库
  #   （本机库的 observer-records 只有本机那几块，第二台树里的只有第二台那几块）；两台判段都读到自己库里录的，对账为空、退 0
  observer_blocks_of() { sort -n "$1" 2>/dev/null | tr '\n' ' '; }   # <库目录下 observer-records 的路径>：一行一块的序号，排好序、空格隔开
  run_cell observers-recorded-by-share STUB_OBSERVERS=1 SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target
  if [[ "$cell_exit" != 0 ]] || [[ "$(tr '\n' ';' <"$work/observers-recorded-by-share/calls.record-share")" != "local share=0/2 weights=none;" ]] \
     || [[ "$(stage_lines_of observers-recorded-by-share local | head -n 1)" != "local stage=record" ]] \
     || [[ "$(stage_lines_of observers-recorded-by-share peer | sort -u | tr '\n' ';')" != "peer stage=unset;" ]] \
     || [[ "$(observer_blocks_of "$work/observers-recorded-by-share/library/observer-records")" != "0 2 4 " ]] \
     || [[ "$(observer_blocks_of "$work/observers-recorded-by-share/peer-tree/crash-amplification-library/observer-records")" != "1 3 5 " ]] \
     || ! output_has observers-recorded-by-share 'unjudged_blocks=0 red=0'; then
    cell_failed observers-recorded-by-share "不带 --given-states 单独起：本机录入段只录本机那一份（.record-share 只有 local share=0/2 一笔）、第二台自己先录它那一份再判（选段不设）、两台各写各的库（本机 0 2 4、第二台 1 3 5）、两台判段读到的都是自己库里录的、对账为空、退 0（退 $cell_exit；.record-share「$(tr '\n' ';' <"$work/observers-recorded-by-share/calls.record-share")」；本机库「$(observer_blocks_of "$work/observers-recorded-by-share/library/observer-records")」；第二台库「$(observer_blocks_of "$work/observers-recorded-by-share/peer-tree/crash-amplification-library/observer-records")」）" \
      "看 mkdir -p 建日志目录之后「分着录」record_split 那一段、run_local_record_then_share（录入段收 0/2）与 peer_environment 后面选段那一段（分着录时第二台不设 judge）"
  fi
  # 给了 --host-weights：权重在录入时就定，本机录入段也带它（录入段与判段同一个分法），本机只录权重分给它的那几块；第二台那一份同理
  run_cell observers-recorded-by-share-with-weights STUB_OBSERVERS=1 SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target --host-weights 3,1
  if [[ "$cell_exit" != 0 ]] || [[ "$(tr '\n' ';' <"$work/observers-recorded-by-share-with-weights/calls.record-share")" != "local share=0/2 weights=3,1;" ]] \
     || [[ "$(observer_blocks_of "$work/observers-recorded-by-share-with-weights/library/observer-records")" != "0 1 2 4 5 " ]] \
     || [[ "$(observer_blocks_of "$work/observers-recorded-by-share-with-weights/peer-tree/crash-amplification-library/observer-records")" != "3 " ]] \
     || ! output_has observers-recorded-by-share-with-weights 'unjudged_blocks=0 red=0'; then
    cell_failed observers-recorded-by-share-with-weights "给了 --host-weights 3,1：本机录入段应当带着同一份权重（.record-share 是 local share=0/2 weights=3,1）、只录权重分给本机的那几块（0 1 2 4 5）、第二台只录它那一块（3）、两台判段都读到自己库里录的、退 0（退 $cell_exit；.record-share「$(tr '\n' ';' <"$work/observers-recorded-by-share-with-weights/calls.record-share")」；本机库「$(observer_blocks_of "$work/observers-recorded-by-share-with-weights/library/observer-records")」）" \
      "看 run_local_record_then_share：录入段的附加环境带 weights_environment（record_extra），与判段同一份"
  fi
  # 给了 --local-states-per-second（权重要等第二台量完速度才有）：录入段还没有权重，照旧本机整份录（SHARE=0/1）、第二台只判（读送过去的快照）；
  #   假的用例不模拟排计划的速度之外的东西，这一格只看录入段那一趟的份与第二台的选段
  run_cell observers-recorded-whole-when-weights-wait-for-speed STUB_OBSERVERS=1 SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || [[ "$(tr '\n' ';' <"$work/observers-recorded-whole-when-weights-wait-for-speed/calls.record-share")" != "local share=0/1 weights=none;" ]] \
     || [[ "$(stage_lines_of observers-recorded-whole-when-weights-wait-for-speed peer | sort -u | tr '\n' ';')" != "peer stage=judge;" ]]; then
    cell_failed observers-recorded-whole-when-weights-wait-for-speed "只给了 --local-states-per-second（权重要等第二台量完速度）：录入段应当本机整份录（.record-share 是 local share=0/1）、第二台只判（stage=judge）、退 0（退 $cell_exit；.record-share「$(tr '\n' ';' <"$work/observers-recorded-whole-when-weights-wait-for-speed/calls.record-share")」；.stage「$(tr '\n' ';' <"$work/observers-recorded-whole-when-weights-wait-for-speed/calls.stage")」）" \
      "看「分着录」record_split 的条件：local_states_per_second 非空而 host_weights 空时不分着录"
  fi
  # 本机录入段红了：判段不起（没有 share=0/2 的判段那一趟），第二台那一趟照常跑完取回，带停下文件导入它判过的块（不设选段、不重新录观察者），退本机录入段的退出码（101）
  run_cell split-record-fails STUB_OBSERVERS=1 STUB_LOCAL_RECORD_FAILURE=plain SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target
  if [[ "$cell_exit" != 101 ]] || calls_have split-record-fails '^local share=0/2 import=no ' || ! calls_have split-record-fails '^peer share=1/2 import=no ' \
     || ! last_call_is split-record-fails '^local share=0/2 import=yes placement=0 stop=yes ' || [[ "$(stage_lines_of split-record-fails local | tail -n 1)" != "local stage=unset" ]] \
     || [[ -s "$work/split-record-fails/library/observer-records" ]] || ! output_has split-record-fails '观察者录入段退 101' \
     || [[ "$(observer_blocks_of "$work/split-record-fails/peer-tree/crash-amplification-library/observer-records")" != "1 3 5 " ]]; then
    cell_failed split-record-fails "分着录时本机录入段红了（退 101）：判段不起、第二台那一趟照常跑完（先录它那一份观察者 1 3 5 再判）、带停下文件导入它取回的库（不设选段、不重新录观察者）、退 101、点名是录入段（退 $cell_exit；调用记录「$(tr '\n' ';' <"$work/split-record-fails/calls")」）" \
      "看 run_local_record_then_share 录入段红了留 local-record-failed 标记、退它的退出码；主流程「导入那一趟的选段」那一段与本机那一份退非 0 那一段"
  fi
  # 分份途中被停（分着录）：同 term-imports-the-fetched-library，只是选段不设（本机录入段跑完、判段跑完才收到 TERM）：带 IMPORT 的那一趟本机不设选段、带着停下文件
  run_cell split-term-imports-the-fetched-library --term-when "$work/split-term-imports-the-fetched-library/calls.peer-running" \
    --term-when "$work/split-term-imports-the-fetched-library/calls.local-share-done" STUB_OBSERVERS=1 STUB_PEER_WAIT_FOR_TERM=1 SINGLEFS_CRASH_AMPLIFICATION_STAGE= -- some-target
  if [[ "$cell_exit" != 143 ]] || ! calls_have split-term-imports-the-fetched-library '^peer term$' \
     || ! last_call_is split-term-imports-the-fetched-library '^local share=0/2 import=yes placement=0 stop=yes ' \
     || [[ "$(stage_lines_of split-term-imports-the-fetched-library local | tail -n 1)" != "local stage=unset" ]] \
     || [[ "$(observer_blocks_of "$work/split-term-imports-the-fetched-library/peer-tree/crash-amplification-library/observer-records")" != "1 3 5 " ]]; then
    cell_failed split-term-imports-the-fetched-library "分着录时分份途中给驱动发 TERM：应当把 TERM 转给第二台那一趟、等它取回库，再不设选段、带停下文件跑一趟本机 0/2 带 IMPORT、退 143（退 $cell_exit；调用记录「$(tr '\n' ';' <"$work/split-term-imports-the-fetched-library/calls")」）" \
      "看主流程「导入那一趟的选段」：分着录时收到 TERM 就 unset 选段，被停的导入那一趟带停下文件"
  fi

  # ── 先判着、量第二台的速度、按速度分
  run_cell calibrated-weights -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! output_has calibrated-weights '^CRASH_AMPLIFICATION share=0/1 .*stopped_early=true.* warm_up=true$' \
     || ! output_has calibrated-weights '^CRASH_AMPLIFICATION_HOST_WEIGHTS local,peer=150,50$' \
     || ! calls_have calibrated-weights '^peer share=0/1 import=no placement=1 ' \
     || ! calls_have calibrated-weights '^local share=0/2 import=no .* weights=150,50$' || ! calls_have calibrated-weights '^peer share=1/2 import=no .* weights=150,50$'; then
    cell_failed calibrated-weights "本机先判着（转打带 warm_up=true）、第二台排计划量出每秒 50、两趟分份都带 SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=150,50，应当退 0" \
      "看「给了本机的速度」那一段：量完建停下文件、转打加 warm_up=true、host_weights 写进 weights_environment"
  fi
  run_cell warm-run-card-unusable STUB_WARM_FAILURE=device -- some-target --local-states-per-second 150
  [[ "$cell_exit" == 78 ]] \
    || cell_failed warm-run-card-unusable "本机先判着的那一趟卡用不了，应当退 78" "看「本机先判着的那一趟退」那一段：经 crash_amplification_exit_code_of_a_failed_run 照 warm-local.log 分类"
  run_cell peer-placement-unreachable STUB_PEER_PLACEMENT=unreachable -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 78 ]] || ! output_has peer-placement-unreachable '^CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED exit=255$'; then
    cell_failed peer-placement-unreachable "第二台排计划那一趟 multi-host-run.sh 退 255，应当打 CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED 并退 78" \
      "看「第二台排派活计划那一趟没量出速度」那一段：255 映射成 78"
  fi
  run_cell peer-placement-card-unusable STUB_PEER_PLACEMENT=device -- some-target --local-states-per-second 150
  [[ "$cell_exit" == 78 ]] \
    || cell_failed peer-placement-card-unusable "第二台排计划那一趟卡用不了，应当退 78" "看「第二台排派活计划那一趟没量出速度」那一段的 case：device 退 78"
  run_cell peer-placement-algorithm STUB_PEER_PLACEMENT=algorithm -- some-target --local-states-per-second 150
  [[ "$cell_exit" == 248 ]] \
    || cell_failed peer-placement-algorithm "第二台排计划那一趟算法上判不了，应当退 248" "看「第二台排派活计划那一趟没量出速度」那一段的 case：algorithm 退 248"
  run_cell peer-placement-unrecognized STUB_PEER_PLACEMENT=unrecognized -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 249 ]] || calls_have peer-placement-unrecognized '^peer share=1/2 '; then
    cell_failed peer-placement-unrecognized "第二台没量出速度、日志里认不出原因，应当退 249、不起分份那两趟" \
      "看「第二台排派活计划那一趟没量出速度」那一段：认不出的退 249，不往下分份"
  fi
  run_cell peer-placement-not-run STUB_PEER_PLACEMENT=not-run -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 77 ]] || calls_have peer-placement-not-run '^peer share=1/2 ' \
     || ! output_has peer-placement-not-run '^CRASH_AMPLIFICATION_NOT_RUN step=peer-placement test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB（假的用例）$' \
     || ! output_has peer-placement-not-run '本次未跑（这不是通过）：第二台排派活计划那一趟借不到显存' || output_has peer-placement-not-run '^CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED '; then
    cell_failed peer-placement-not-run "第二台排计划那一趟借不到显存（退 0、没打计划行、有产品路径那一行），应当打 CRASH_AMPLIFICATION_NOT_RUN step=peer-placement、报本次未跑、退 77、不起分份那两趟" \
      "看「第二台排派活计划那一趟没量出速度」之前认 peer_not_run_line 那一段：那一趟退 0、没有计划行、行首是产品路径那一行才认"
  fi
  run_cell peer-placement-foreign-not-run-line STUB_PEER_PLACEMENT=foreign-not-run -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 249 ]] || output_has peer-placement-foreign-not-run-line '^CRASH_AMPLIFICATION_NOT_RUN '; then
    cell_failed peer-placement-foreign-not-run-line "第二台排计划那一趟只有别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行，应当照没量出速度、认不出原因退 249，不打 CRASH_AMPLIFICATION_NOT_RUN" \
      "看认 peer_not_run_line 那一段：只认行首是「NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=」的那一行"
  fi

  # ── 参数、配置与清场
  run_cell host-weights-malformed -- some-target --host-weights 3
  [[ "$cell_exit" == 2 ]] || cell_failed host-weights-malformed "--host-weights 3（缺第二台那一个数）应当退 2" "看参数之后 --host-weights 的格式判"
  run_cell local-speed-malformed -- some-target --local-states-per-second 0
  [[ "$cell_exit" == 2 ]] || cell_failed local-speed-malformed "--local-states-per-second 0 应当退 2" "看参数之后 --local-states-per-second 的格式判"
  run_cell usage-missing-target --
  [[ "$cell_exit" == 2 ]] || cell_failed usage-missing-target "没给测试目标应当退 2" "看 usage：用法错退 2"
  run_cell usage-unknown-option -- some-target --no-such-option
  [[ "$cell_exit" == 2 ]] || cell_failed usage-unknown-option "认不得的选项应当退 2" "看参数那一段的 -*) 分支与 usage"
  run_cell quiesced-without-card-list SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-without-quiesced.env" -- some-target --quiesced
  [[ "$cell_exit" == 78 ]] \
    || cell_failed quiesced-without-card-list "说是清过场、多机配置里却没写 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（退役的私有显卡配置里写了也不算），应当退 78" "看 crash_amplification_prepare 那一行：交回非 0 就退 78"
  run_cell quiesce-configuration-fails STUB_QUIESCE=configuration-fails -- some-target --quiesce
  [[ "$cell_exit" == 78 ]] \
    || cell_failed quiesce-configuration-fails "--quiesce 时多机配置判不过，应当退 78" "看 --quiesce 那一段 peer_host_load_configuration 之后的退出"
  run_cell quiesce-fails STUB_QUIESCE=quiesce-fails -- some-target --quiesce
  [[ "$cell_exit" == 78 ]] || cell_failed quiesce-fails "--quiesce 时清场没过，应当退 78" "看 --quiesce 那一段 peer_host_quiesce 之后的退出"
  run_cell quiesce-not-restored -- some-target --quiesce
  if [[ "$cell_exit" != 0 ]] || ! last_call_is quiesce-not-restored '^quiesce-left$' || grep -qx restore "$work/quiesce-not-restored/calls"; then
    cell_failed quiesce-not-restored "--quiesce 清过场、跑完退 0，退出时应当只报「清了场、没复原」、不跑复原命令（调用记录最后一笔是 quiesce-left、没有 restore）" "看 --quiesce 那一段 trap 'peer_host_quiesce_left' EXIT"
  fi

  # ── 走 GPU 的计算类与显卡先后（crash-amplification-lib.sh 文件头「配置」）：引擎不读 GPU_COMPUTATION_CLASSES，开不开 GPU 两台的每一趟都不送它
  #   （调用方环境里漏进来的那一份本机那几趟清掉；第二台那一趟是新起的 ssh 会话，只看得见命令里送过去的）；显卡先后读多机配置，两台的每一趟都交到。
  #   退役的私有显卡配置（上面 export 的 SINGLEFS_GPU_CARDS_CONFIG）写了别的卡与一类计算，读到它就红。
  #   各格带 --local-states-per-second，五种调用（本机先判着、第二台排计划、两台分份、本机导入）都走到；所以主路径上的开关（never-stop-warm-run、
  #   same-share-on-both-hosts、skip-import）打开时这几格也红：少了哪一种调用就核不到它的环境，不为脱开它们放掉退 0 与五种调用都在这两条
  printf '%s\n' ENABLE_GPU=1 ENABLE_ACROSS_MACHINES=1 'GPU_CARDS_BY_PRIORITY="local:0:15G peer:0:10G"' >"$work/multi-host-gpu-on.env"
  printf '%s\n' ENABLE_GPU=0 ENABLE_ACROSS_MACHINES=1 >"$work/multi-host-gpu-off.env"
  every_call_has_classes() { ! grep -qvF " classes=[$2] " "$work/$1/calls"; }   # <格名> <值>：每一笔调用记录都是这个 classes
  every_call_has_cards() { ! grep -qvF " cards=[$2] " "$work/$1/calls"; }       # <格名> <值>：每一笔调用记录都是这个 cards
  five_calls_seen() { # <格名>：先判着、第二台排计划、两台分份、本机导入五种调用都在，最后一趟是导入
    calls_have "$1" '^local share=0/1 import=no placement=0 stop=yes ' && calls_have "$1" '^peer share=0/1 import=no placement=1 ' \
      && calls_have "$1" '^local share=0/2 import=no ' && calls_have "$1" '^peer share=1/2 import=no ' && last_call_is "$1" '^local share=0/2 import=yes '
  }
  run_cell gpu-on-classes-not-sent SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" \
    GPU_COMPUTATION_CLASSES=crash_judge_gpu -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! every_call_has_classes gpu-on-classes-not-sent unset || ! five_calls_seen gpu-on-classes-not-sent; then
    cell_failed gpu-on-classes-not-sent "开了 GPU（调用方环境里漏进来一份 GPU_COMPUTATION_CLASSES）：本机三趟与第二台两趟的环境里都不该有 GPU_COMPUTATION_CLASSES（引擎不读它，经 cleared_then_set 设它的用例会 panic），退 0" \
      "本机那几趟看 crash-amplification-lib.sh 的 crash_amplification_environment_for_this_host：开头 -u GPU_COMPUTATION_CLASSES、不再加这个变量；第二台那两趟看 peer_environment：不加这个变量"
  fi
  run_cell gpu-cards-from-multi-host-configuration SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! every_call_has_cards gpu-cards-from-multi-host-configuration 'local:0:15G peer:0:10G' \
     || ! five_calls_seen gpu-cards-from-multi-host-configuration; then
    cell_failed gpu-cards-from-multi-host-configuration "开了 GPU、多机配置写了显卡先后「local:0:15G peer:0:10G」（退役的私有显卡配置里是 local:1:3G peer:1:3G）：两台每一趟的环境里 GPU_CARDS_BY_PRIORITY 都应当是多机配置那一份，退 0" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare：显卡先后从 crash_amplification_cards_source_file 给的多机配置读"
  fi
  run_cell gpu-off-classes-not-sent SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" \
    GPU_COMPUTATION_CLASSES=crash_judge_gpu -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! every_call_has_classes gpu-off-classes-not-sent unset \
     || ! calls_have gpu-off-classes-not-sent '^local share=0/2 import=yes ' || ! calls_have gpu-off-classes-not-sent '^peer share=1/2 import=no '; then
    cell_failed gpu-off-classes-not-sent "没开 GPU（调用方环境里漏进来一份 GPU_COMPUTATION_CLASSES）：两台每一趟的环境里都不该有 GPU_COMPUTATION_CLASSES，退 0" \
      "看 crash_amplification_environment_for_this_host 开头的 -u GPU_COMPUTATION_CLASSES"
  fi

  # ── gpu 那一趟的 cargo 参数带 --test-threads 1（GPU 对拍的测试各占卡的显存，不并行起；.claude/rules/verification.md「两档：轻型与重型」）：
  #   开了 GPU，本机三趟与第二台排计划、分份两趟的 cargo test 参数都带；没开的不加。假的用例把收到的参数里有没有它记进 <调用记录>.test-threads
  run_cell gpu-test-threads-one SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! five_calls_seen gpu-test-threads-one || grep -qv '^[a-z]* test_threads=yes$' "$work/gpu-test-threads-one/calls.test-threads" \
     || (( $(grep -c '^local ' "$work/gpu-test-threads-one/calls.test-threads") < 3 )) || (( $(grep -c '^peer ' "$work/gpu-test-threads-one/calls.test-threads") < 2 )); then
    cell_failed gpu-test-threads-one "开了 GPU：本机三趟与第二台排计划、分份两趟的 cargo test 参数都应当带 --test-threads 1，退 0（实记「$(tr '\n' ';' <"$work/gpu-test-threads-one/calls.test-threads")」）" \
      "看 crash_amplification_prepare 之后那一行：gpu_switch 为 1 时 cargo_test_arguments 要加 --test-threads 1"
  fi
  run_cell gpu-off-test-threads-not-added SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || grep -qv '^[a-z]* test_threads=no$' "$work/gpu-off-test-threads-not-added/calls.test-threads" \
     || (( $(grep -c '^local ' "$work/gpu-off-test-threads-not-added/calls.test-threads") < 1 )) || (( $(grep -c '^peer ' "$work/gpu-off-test-threads-not-added/calls.test-threads") < 1 )); then
    cell_failed gpu-off-test-threads-not-added "没开 GPU：本机与第二台的 cargo test 参数都不加 --test-threads 1，退 0（实记「$(tr '\n' ';' <"$work/gpu-off-test-threads-not-added/calls.test-threads")」）" \
      "看 crash_amplification_prepare 之后那一行：只在 gpu_switch 为 1 时加"
  fi

  # ── 往引擎送的变量（文件头「步骤」之后 peer_environment 那一段）：本机设了，第二台排计划、分份那两趟也要在环境里拿到；
  #   按段登记文件随快照送进第二台那棵树，变量值是第二台上那一份的绝对路径（假的 multi-host-run.sh 在假树根里真跑 peer_tree_path_script 拼出）。
  #   各格带 --local-states-per-second，排计划与分份两趟都走到；调用记录旁的 <调用记录>.engine-environment 逐趟记 registry=[…] registry_sum=[…]
  printf '%s\n' '# 自证用的按段登记文件' 'item-a	0	3	0000000000000000000000000000000000000000000000000000000000000000' >"$work/crash-point-replay-segments.tsv"
  registry_sum_of_local="$(cksum <"$work/crash-point-replay-segments.tsv" | cut -d' ' -f1)"
  peer_engine_lines() { grep '^peer ' "$work/$1/calls.engine-environment" 2>/dev/null; }   # <格名>
  peer_engine_lines_are() { # <格名> <每一趟都要含的子串>：第二台至少有排计划与分份两趟，每一趟都含它
    local lines
    lines="$(peer_engine_lines "$1")"
    [[ "$(grep -c . <<<"$lines")" -ge 2 ]] && ! grep -qvF -- "$2" <<<"$lines"
  }
  run_cell peer-segments-registry-sent SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY="$work/crash-point-replay-segments.tsv" -- some-target --local-states-per-second 150
  peer_registry_path="$work/peer-segments-registry-sent/peer-tree/crash-amplification-segments-registry/crash-point-replay-segments.tsv"
  if [[ "$cell_exit" != 0 ]] || ! peer_engine_lines_are peer-segments-registry-sent " registry=[$peer_registry_path] registry_sum=[$registry_sum_of_local]" \
     || ! five_calls_seen peer-segments-registry-sent || ! grep -q "^local .* registry=\[$work/crash-point-replay-segments.tsv\] " "$work/peer-segments-registry-sent/calls.engine-environment"; then
    cell_failed peer-segments-registry-sent "本机环境里设了 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY：第二台排计划、分份那两趟的环境里要有它，值是第二台树里 crash-amplification-segments-registry/ 下那份快照的路径、内容与本机那份相同（实记「$(peer_engine_lines peer-segments-registry-sent | tr '\n' ';')」），本机那几趟照旧继承，退 0" \
      "看 peer_environment 后面那一段：快照拷进 <日志目录>/segments-registry-for-peer/、两趟 multi-host-run.sh 都带 --send、命令最前面加 peer_tree_path_prefix"
  fi
  run_cell peer-engine-variables-absent-when-unset -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! peer_engine_lines_are peer-engine-variables-absent-when-unset ' registry=[unset] registry_sum=[unset]' \
     || [[ -e "$work/peer-engine-variables-absent-when-unset/peer-tree/crash-amplification-segments-registry" ]]; then
    cell_failed peer-engine-variables-absent-when-unset "本机环境里没设这个变量：第二台那几趟也不该有、树里也不该多出登记文件快照（实记「$(peer_engine_lines peer-engine-variables-absent-when-unset | tr '\n' ';')」），退 0" \
      "看 peer_environment 后面那一段的条件：本机没设（空串也不送登记文件）就什么都不加"
  fi
  # peer-slice-states-sent：本机设了 SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=131072 ⇒ 第二台排计划（placement=1）、分份（placement=0）两趟的环境里都是 131072，本机那几趟照旧继承；
  #   本机没设 ⇒ 两趟都没有：读上一格 peer-engine-variables-absent-when-unset 那一趟的记录（run_cell 清掉了这个变量，同一种起法）
  run_cell peer-slice-states-sent SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=131072 -- some-target --local-states-per-second 150
  slice_states_local_lines="$(grep -c '^local ' "$work/peer-slice-states-sent/calls.engine-environment" 2>/dev/null)"
  if [[ "$cell_exit" != 0 ]] || ! five_calls_seen peer-slice-states-sent || ! peer_engine_lines_are peer-slice-states-sent ' slice_states=[131072]' \
     || ! grep -q '^peer placement=1 .* slice_states=\[131072\]$' "$work/peer-slice-states-sent/calls.engine-environment" \
     || ! grep -q '^peer placement=0 .* slice_states=\[131072\]$' "$work/peer-slice-states-sent/calls.engine-environment" \
     || (( ${slice_states_local_lines:-0} == 0 )) || [[ "$(grep -c '^local .* slice_states=\[131072\]$' "$work/peer-slice-states-sent/calls.engine-environment")" != "$slice_states_local_lines" ]] \
     || ! peer_engine_lines_are peer-engine-variables-absent-when-unset ' slice_states=[unset]'; then
    cell_failed peer-slice-states-sent "本机环境里设了 SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=131072：第二台排计划、分份那两趟的环境里都要有它、值照原样，本机那几趟照旧继承，退 0；本机没设：第二台那两趟都不该有（实记设了的「$(tr '\n' ';' <"$work/peer-slice-states-sent/calls.engine-environment")」、没设的第二台「$(peer_engine_lines peer-engine-variables-absent-when-unset | tr '\n' ';')」）" \
      "看 peer_environment 后面 SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES 那一段：本机设了（空串也算）就照原值加进 peer_environment，peer_threads_now 再拷进两趟用的 peer_run_environment"
  fi
  # peer-write-only-sent：本机设了 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1（崩溃点重放入口 --full）⇒ 第二台排计划、分份两趟的环境里都是 1，本机那几趟照旧继承；
  #   本机没设 ⇒ 第二台两趟都没有：读 peer-engine-variables-absent-when-unset 那一趟的记录
  run_cell peer-write-only-sent SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1 -- some-target --local-states-per-second 150
  write_only_local_lines="$(grep -c '^local ' "$work/peer-write-only-sent/calls.engine-environment" 2>/dev/null)"
  if [[ "$cell_exit" != 0 ]] || ! five_calls_seen peer-write-only-sent || ! peer_engine_lines_are peer-write-only-sent ' write_only=[1] ' \
     || ! grep -q '^peer placement=1 .* write_only=\[1\] ' "$work/peer-write-only-sent/calls.engine-environment" \
     || ! grep -q '^peer placement=0 .* write_only=\[1\] ' "$work/peer-write-only-sent/calls.engine-environment" \
     || (( ${write_only_local_lines:-0} == 0 )) || [[ "$(grep -c '^local .* write_only=\[1\] ' "$work/peer-write-only-sent/calls.engine-environment")" != "$write_only_local_lines" ]] \
     || ! peer_engine_lines_are peer-engine-variables-absent-when-unset ' write_only=[unset] '; then
    cell_failed peer-write-only-sent "本机环境里设了 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1：第二台排计划、分份那两趟的环境里都要有它、值照原样，本机那几趟照旧继承，退 0；本机没设：第二台那两趟都不该有（实记设了的「$(tr '\n' ';' <"$work/peer-write-only-sent/calls.engine-environment")」、没设的第二台「$(peer_engine_lines peer-engine-variables-absent-when-unset | tr '\n' ';')」）" \
      "看 peer_environment 后面 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY 那一段：本机设了就照原值加进 peer_environment，peer_threads_now 再拷进两趟用的 peer_run_environment"
  fi

  # ── 线程数的缺省（crash-amplification-lib.sh 文件头「线程数的缺省」）：没给就取登记的重活默认并行度（.claude/heavy-thread-default），SINGLEFS_THREAD_CAP 设了取它
  # 第二台那两趟（排计划、分份）：送过去的 capped.sh 线程数由假的 multi-host-run.sh 记进 <调用记录>.threads
  local registered_threads
  registered_threads="$(env -u SINGLEFS_THREAD_CAP bash -c 'source "$1"; heavy_thread_default' two-hosts-registered "$two_hosts_script_directory/../hosts/heavy-threads.sh")" \
    || { echo "  ✗ --selftest：读不出登记的重活默认并行度（scripts/hosts/heavy-threads.sh 的 heavy_thread_default）" >&2; echo "     → 怎么办：先跑 bash scripts/hosts/heavy-threads.sh --selftest，把 .claude/heavy-thread-default 修好" >&2; exit 1; }
  peer_threads_are() { # <格名> <线程数>：排计划、分份那两趟各一笔、线程数都是它
    [[ "$(LC_ALL=C sort "$work/$1/calls.threads" 2>/dev/null | tr '\n' ';')" == "placement=0 threads=$2;placement=1 threads=$2;" ]]
  }
  run_cell peer-threads-default -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! peer_threads_are peer-threads-default "$registered_threads"; then
    cell_failed peer-threads-default "没给 CRASH_AMPLIFICATION_PEER_THREADS、没设 SINGLEFS_THREAD_CAP：第二台排计划、分份那两趟都应当 capped.sh 登记值 $registered_threads、退 0（实记「$(tr '\n' ';' <"$work/peer-threads-default/calls.threads" 2>/dev/null)」）" \
      "看 peer_threads_now：没给 CRASH_AMPLIFICATION_PEER_THREADS 时取 crash_amplification_threads_now peer，它打登记的重活默认并行度"
  fi
  run_cell peer-threads-given CRASH_AMPLIFICATION_PEER_THREADS=3 -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! peer_threads_are peer-threads-given 3; then
    cell_failed peer-threads-given "调用方给了 CRASH_AMPLIFICATION_PEER_THREADS=3：第二台那两趟都应当 capped.sh 3（不按登记值 $registered_threads）、退 0（实记「$(tr '\n' ';' <"$work/peer-threads-given/calls.threads" 2>/dev/null)」）" \
      "看 peer_threads_now 把 CRASH_AMPLIFICATION_PEER_THREADS 交给 crash_amplification_threads_now 的第二个参数"
  fi
  run_cell peer-threads-cap SINGLEFS_THREAD_CAP=5 -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! peer_threads_are peer-threads-cap 5; then
    cell_failed peer-threads-cap "设了 SINGLEFS_THREAD_CAP=5、没给 CRASH_AMPLIFICATION_PEER_THREADS：第二台那两趟都应当 capped.sh 5（上限顶掉登记值）、退 0（实记「$(tr '\n' ';' <"$work/peer-threads-cap/calls.threads" 2>/dev/null)」）" \
      "看 crash_amplification_default_threads：它经 heavy_thread_default 取，SINGLEFS_THREAD_CAP 设了照它"
  fi
  # 第二台那一份的上限：没给 --peer-memory-cap 时与本机 --memory-cap 相同（只看假的 multi-host-run.sh 记下的上限，不看退出码）
  run_cell peer-cap-follows-this-host -- some-target --memory-cap 6G
  if [[ "$(LC_ALL=C sort -u "$work/peer-cap-follows-this-host/calls.memory-cap" 2>/dev/null | tr '\n' ';')" != "memory-cap=6G;" ]]; then
    cell_failed peer-cap-follows-this-host "没给 --peer-memory-cap、本机 --memory-cap 6G：第二台那一趟应当拿到 --memory-cap 6G（实记「$(tr '\n' ';' <"$work/peer-cap-follows-this-host/calls.memory-cap" 2>/dev/null)」）" \
      "看参数解析之后那一行：peer_memory_cap 为空时取 memory_cap，不落到配置的 PEER_MEMORY_CAP"
  fi
  run_cell peer-threads-given-invalid CRASH_AMPLIFICATION_PEER_THREADS=abc -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 78 ]] || [[ -s "$work/peer-threads-given-invalid/calls.threads" ]] || calls_have peer-threads-given-invalid '^(local|peer) ' \
     || ! output_has peer-threads-given-invalid '不是正整数'; then
    cell_failed peer-threads-given-invalid "调用方给了 CRASH_AMPLIFICATION_PEER_THREADS=abc：应当退 78、报不是正整数，本机先判着与第二台那两趟都不起（退 $cell_exit）" \
      "看 peer_threads_now：crash_amplification_threads_now 交回非 0 时 exit 78，要在起本机先判着那一趟之前调"
  fi
  # 第二台那两趟判的线程数（SINGLEFS_CRASH_AMPLIFICATION_THREADS，假的用例记进 <调用记录>.checking-threads）：取第二台那一份，不沿用本机的。
  #   各格给本机 --threads 9，本机那几趟判 9；第二台的与它不同才分得出
  checking_threads_are() { # <格名> <第二台的线程数>：第二台排计划、分份两趟各一笔、都是它；本机那几趟至少一笔、都是 9
    local recorded="$work/$1/calls.checking-threads" local_lines
    local_lines="$(grep '^local ' "$recorded" 2>/dev/null)"
    [[ "$(grep '^peer ' "$recorded" 2>/dev/null | LC_ALL=C sort | tr '\n' ';')" == "peer placement=0 checking_threads=$2;peer placement=1 checking_threads=$2;" ]] \
      && [[ -n "$local_lines" ]] && ! grep -qv ' checking_threads=9$' <<<"$local_lines"
  }
  run_cell peer-checking-threads-default -- some-target --threads 9 --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! checking_threads_are peer-checking-threads-default "$registered_threads"; then
    cell_failed peer-checking-threads-default "没给 CRASH_AMPLIFICATION_PEER_THREADS、本机 --threads 9：第二台排计划、分份那两趟判的线程数都应当是登记值 $registered_threads、本机那几趟是 9，退 0（实记「$(tr '\n' ';' <"$work/peer-checking-threads-default/calls.checking-threads" 2>/dev/null)」）" \
      "看 peer_threads_now：peer_run_environment 换掉 common_environment 带来的本机那一份 SINGLEFS_CRASH_AMPLIFICATION_THREADS，两趟都用 peer_run_environment"
  fi
  run_cell peer-checking-threads-given CRASH_AMPLIFICATION_PEER_THREADS=3 -- some-target --threads 9 --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! checking_threads_are peer-checking-threads-given 3; then
    cell_failed peer-checking-threads-given "调用方给了 CRASH_AMPLIFICATION_PEER_THREADS=3：第二台那两趟判的线程数都应当是 3（不按登记值、不沿用本机的 9）、退 0（实记「$(tr '\n' ';' <"$work/peer-checking-threads-given/calls.checking-threads" 2>/dev/null)」）" \
      "看 peer_threads_now：SINGLEFS_CRASH_AMPLIFICATION_THREADS 取 peer_threads，与 capped.sh 那一个数同源"
  fi
  run_cell peer-checking-threads-cap SINGLEFS_THREAD_CAP=5 -- some-target --threads 9 --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! checking_threads_are peer-checking-threads-cap 5; then
    cell_failed peer-checking-threads-cap "设了 SINGLEFS_THREAD_CAP=5、本机 --threads 9：第二台那两趟判的线程数都应当是 5、本机那几趟是 9，退 0（实记「$(tr '\n' ';' <"$work/peer-checking-threads-cap/calls.checking-threads" 2>/dev/null)」）" \
      "看 crash_amplification_default_threads 经 heavy_thread_default 认 SINGLEFS_THREAD_CAP，与 peer_threads_now 把它写进 peer_run_environment"
  fi
  # 本机只编那一步：另起一个 bash 只 source 共用库，调真的 crash_amplification_run_on_this_host；仓根换成格目录下的假树，
  # 里面的 run-with-memory-cap.sh 去掉上限照起、capped.sh 只把「线程数 只编还是跑」记进调用记录
  this_host_run() { # <格名> [<环境 键=值>…]：交回值放 cell_exit
    local name="$1"
    shift
    cell_directory="$work/$name"
    mkdir -p "$cell_directory/root/scripts" "$cell_directory/root/scripts/hosts"
    : >"$cell_directory/calls"; : >"$cell_directory/calls.stage"
    cells=$((cells + 1))
    printf '%s\n' '#!/usr/bin/env bash' 'shift; exec "$@"' >"$cell_directory/root/scripts/hosts/run-with-memory-cap.sh"
    printf '%s\n' '#!/usr/bin/env bash' 'step=test; [[ " $* " == *" --no-run "* ]] && step=build' 'echo "threads=$1 step=$step" >>"${STUB_CALLS:?}"' >"$cell_directory/root/scripts/hosts/capped.sh"
    env -u CRASH_AMPLIFICATION_BUILD_THREADS -u SINGLEFS_THREAD_CAP STUB_CALLS="$cell_directory/calls" "$@" \
      bash -c 'repository_root="$1"; source "$2" || exit 3; threads=7; memory_cap=1G; common_environment=(); gpu_cards_by_priority=""
               cargo_build_arguments=(test --no-run); cargo_test_arguments=(test)
               crash_amplification_run_on_this_host 0/1 "$3/library" "$3/run.log"' \
      this-host-run "$cell_directory/root" "$two_hosts_script_directory/crash-amplification-lib.sh" "$cell_directory" >"$cell_directory/output" 2>&1
    cell_exit=$?
  }
  this_host_run this-host-threads-default
  if [[ "$cell_exit" != 0 || "$(tr '\n' ';' <"$work/this-host-threads-default/calls")" != "threads=$registered_threads step=build;threads=7 step=test;" ]]; then
    cell_failed this-host-threads-default "没给 CRASH_AMPLIFICATION_BUILD_THREADS、没设 SINGLEFS_THREAD_CAP：只编那一步应当 capped.sh 登记值 $registered_threads、跑的那一步照 threads（7）、交回 0（实记「$(tr '\n' ';' <"$work/this-host-threads-default/calls")」）" \
      "看 crash_amplification_run_on_this_host：只编那一步的线程数取 crash_amplification_threads_now local，它打登记的重活默认并行度"
  fi
  this_host_run this-host-threads-given CRASH_AMPLIFICATION_BUILD_THREADS=3
  if [[ "$cell_exit" != 0 || "$(tr '\n' ';' <"$work/this-host-threads-given/calls")" != "threads=3 step=build;threads=7 step=test;" ]]; then
    cell_failed this-host-threads-given "调用方给了 CRASH_AMPLIFICATION_BUILD_THREADS=3：只编那一步应当 capped.sh 3（不按登记值）、交回 0（实记「$(tr '\n' ';' <"$work/this-host-threads-given/calls")」）" \
      "看 crash_amplification_run_on_this_host 把 CRASH_AMPLIFICATION_BUILD_THREADS 交给 crash_amplification_threads_now 的第二个参数"
  fi
  this_host_run this-host-threads-cap SINGLEFS_THREAD_CAP=5
  if [[ "$cell_exit" != 0 || "$(tr '\n' ';' <"$work/this-host-threads-cap/calls")" != "threads=5 step=build;threads=7 step=test;" ]]; then
    cell_failed this-host-threads-cap "设了 SINGLEFS_THREAD_CAP=5：只编那一步应当 capped.sh 5（上限顶掉登记值）、跑的那一步照 threads（7）、交回 0（实记「$(tr '\n' ';' <"$work/this-host-threads-cap/calls")」）" \
      "看 crash_amplification_default_threads 经 heavy_thread_default 认 SINGLEFS_THREAD_CAP"
  fi
  this_host_run this-host-threads-given-invalid CRASH_AMPLIFICATION_BUILD_THREADS=abc
  if [[ "$cell_exit" != 78 || -s "$work/this-host-threads-given-invalid/calls" ]] || ! output_has this-host-threads-given-invalid '不是正整数'; then
    cell_failed this-host-threads-given-invalid "调用方给了 CRASH_AMPLIFICATION_BUILD_THREADS=abc：只编那一步应当交回 78、报不是正整数，一趟 cargo 都不起（退 $cell_exit，实记「$(tr '\n' ';' <"$work/this-host-threads-given-invalid/calls")」）" \
      "看 crash_amplification_run_on_this_host：crash_amplification_threads_now 交回非 0 时 return 78，在起 capped.sh 之前"
  fi
  # 点名了用例（--test-filter）：第二台排计划、分份那两趟的 cargo test 都要带 --include-ignored（崩溃点重放的全量用例标 ignore，不带就一条都不跑、退 0、录入段空着）
  # 第二台共用编译目录的守卫（文件头「第二台的编译目录」）：排计划、分份那两趟都交 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY，值与命令里 CARGO_TARGET_DIR 是同一个目录
  #   （假的 multi-host-run.sh 记进 <调用记录>.shared-target）
  local shared_target_lines shared_target_pattern='^placement=0 shared_target=\[([^]]+)\] cargo_target=\[([^]]+)\];placement=1 shared_target=\[([^]]+)\] cargo_target=\[([^]]+)\];$'
  run_cell peer-build-guarded -- some-target --local-states-per-second 150
  shared_target_lines="$(LC_ALL=C sort "$work/peer-build-guarded/calls.shared-target" 2>/dev/null | tr '\n' ';')"
  if [[ "$cell_exit" != 0 ]] || ! [[ "$shared_target_lines" =~ $shared_target_pattern ]] || [[ "${BASH_REMATCH[1]}" == unset ]] \
     || [[ "${BASH_REMATCH[2]}" != "${BASH_REMATCH[1]}" || "${BASH_REMATCH[3]}" != "${BASH_REMATCH[1]}" || "${BASH_REMATCH[4]}" != "${BASH_REMATCH[1]}" ]]; then
    cell_failed peer-build-guarded "第二台排计划、分份那两趟都应当交 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY、值与命令里的 CARGO_TARGET_DIR 相同，退 0（实记「$shared_target_lines」）" \
      "看「给了本机的速度」那一段之前的 peer_shared_target_environment：从 peer_environment 的 CARGO_TARGET_DIR 取，两处起 multi-host-run.sh 都带上它"
  fi
  # 第二台那两趟的库目录（文件头步骤②）：引擎只收绝对路径，排计划、分份那两趟交的都是第二台树根拼出的 <假树根>/crash-amplification-library
  local peer_library_lines peer_library_expected="peer stage=judge library=$work/peer-library-absolute/peer-tree/crash-amplification-library"
  run_cell peer-library-absolute -- some-target --local-states-per-second 150
  peer_library_lines="$(grep '^peer ' "$work/peer-library-absolute/calls.library" 2>/dev/null | tr '\n' ';')"
  if [[ "$cell_exit" != 0 || "$peer_library_lines" != "$peer_library_expected;$peer_library_expected;" ]]; then
    cell_failed peer-library-absolute "第二台排计划、分份那两趟都应当交绝对路径的库目录 <第二台树根>/crash-amplification-library、退 0（退 $cell_exit；.library 里第二台那几笔「$peer_library_lines」）" \
      "看 peer_library_prefix：两处起 multi-host-run.sh 的命令最前面都带 bash -c \"\$peer_library_path_script\"，env 里不再交相对路径 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY"
  fi
  run_cell named-test-runs-even-if-ignored -- some-target --test-filter the_full_case
  if [[ "$cell_exit" != 0 ]] || ! grep -q '^peer placement=0 include_ignored=yes$' "$work/named-test-runs-even-if-ignored/calls.test-arguments" 2>/dev/null \
     || grep -q 'include_ignored=no' "$work/named-test-runs-even-if-ignored/calls.test-arguments"; then
    cell_failed named-test-runs-even-if-ignored "给了 --test-filter：第二台分份那一趟（placement=0）的 cargo test 应当带 --include-ignored、退 0，记下的每一趟都不许 include_ignored=no（实记「$(tr '\n' ';' <"$work/named-test-runs-even-if-ignored/calls.test-arguments" 2>/dev/null)」）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare：test_filter 非空时 cargo_test_arguments 要加 --include-ignored --exact <用例>"
  fi
  # 共用库拼出来的参数本身：另起一个 bash 只 source 共用库，调真的 crash_amplification_prepare，-- 之后要是 --nocapture --include-ignored --exact <用例>
  cell_directory="$work/this-host-named-test-runs-even-if-ignored"; mkdir -p "$cell_directory"; : >"$cell_directory/calls"; cells=$((cells + 1))
  bash -c 'repository_root="$1"; source "$2" || exit 3; test_target=some-target; test_filter=the_full_case; threads=2; expand_up_to=""
           crash_amplification_prepare || exit 4; printf "%s\n" "${cargo_test_arguments[@]}" >"$3/calls"' \
    this-host-named-test "$work" "$two_hosts_script_directory/crash-amplification-lib.sh" "$cell_directory" >"$cell_directory/output" 2>&1
  cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(tr '\n' ' ' <"$cell_directory/calls")" != *"-- --nocapture --include-ignored --exact the_full_case "* ]]; then
    cell_failed this-host-named-test-runs-even-if-ignored "共用库 crash_amplification_prepare 给了 test_filter 时拼出的 cargo test 参数应当以「-- --nocapture --include-ignored --exact the_full_case」收尾、交回 0" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare：test_filter 非空时加 --include-ignored --exact <用例>"
  fi

  # ── 上次红的清单（文件头「上次红的清单」；scripts/gate/lib_reuse_checks.py 文件头「三」「测试目标」）：四趟写进这一格自己的目录——
  #   beta-target 整个红、点名 one_case 绿、点名 one_case 红、整个绿；每一趟之后清单要等于照这一趟的退出码推出来的那一份
  #   （退 0 绿、退 2 / 77 / 130 / 143 照留、别的红；整个目标那一趟换掉这个目标的全部项，点名的只换那一项）。
  #   没开弄坏开关时另要四趟的退出码依次是 101、0、101、0（红的那两趟真红了，推出来的清单才不是空的）
  local last_run_directory="$work/last-runs-cell" last_run_expected="" last_run_actual="" last_run_problems="" last_run_exits=""
  local last_run_step last_run_name last_run_target last_run_filter last_run_setting last_run_key
  for last_run_step in "last-run-list-whole-red:beta-target::STUB_LOCAL_FAILURE=plain" "last-run-list-filter-green:beta-target:one_case:" \
                       "last-run-list-filter-red:beta-target:one_case:STUB_LOCAL_FAILURE=plain" "last-run-list:beta-target::"; do
    IFS=: read -r last_run_name last_run_target last_run_filter last_run_setting <<<"$last_run_step"
    run_cell "$last_run_name" SINGLEFS_LAST_RUNS_DIRECTORY="$last_run_directory" ${last_run_setting:+"$last_run_setting"} -- \
      "$last_run_target" ${last_run_filter:+--test-filter "$last_run_filter"}
    last_run_exits+="$cell_exit "
    last_run_key="$last_run_target${last_run_filter:+::$last_run_filter}"
    case "$cell_exit" in
      2|77|130|143) ;;
      *)
        if [[ -n "$last_run_filter" ]]; then
          last_run_expected="$(grep -vxF -- "$last_run_key" <<<"$last_run_expected" || true)"
        else
          last_run_expected="$(grep -vxF -- "$last_run_target" <<<"$last_run_expected" | grep -vF -- "$last_run_target::" || true)"
        fi
        (( cell_exit != 0 )) && last_run_expected+=$'\n'"$last_run_key"
        ;;
    esac
    last_run_expected="$(sed '/^$/d' <<<"$last_run_expected" | LC_ALL=C sort -u)"
    last_run_actual="$(sed '/^$/d' "$last_run_directory/scripts__crash__crash-amplification-two-hosts.sh.tsv" 2>/dev/null | LC_ALL=C sort -u)"
    [[ "$last_run_actual" == "$last_run_expected" ]] \
      || last_run_problems+="$last_run_name 退 $cell_exit 之后清单是「$(tr '\n' ' ' <<<"$last_run_actual")」，应为「$(tr '\n' ' ' <<<"$last_run_expected")」；"
  done
  if [[ -z "${CRASH_AMPLIFICATION_TWO_HOSTS_BREAK:-}${CRASH_AMPLIFICATION_ITEM_BREAK:-}" && "$last_run_exits" != "101 0 101 0 " ]]; then
    last_run_problems+="四趟退 $last_run_exits，应为 101 0 101 0；"
  fi
  [[ -z "$last_run_problems" ]] \
    || cell_failed last-run-list "上次红的清单：$last_run_problems" \
      "看主流程开头装的 EXIT 钩子 two_hosts_on_exit（经 scripts/gate/lib_reuse_checks.py last-run-target：红的测试目标一行一个，点名用例的只换那一项）"

  # ── 开关轮的判法（crash-amplification-lib.sh 的 crash_amplification_switch_wheel_verdict）：不起里层，拿一份造出来的输出（红在 cell-a、cell-b 两格）
  #   比几种点名：照实点名的、没列而一格不红的要判对上；删一格、多一格、格名写错、退出码不对、表里没写格、没列而红了的要判对不上
  cells=$((cells + 1)); cell_exit="（不起里层）"
  local wheel_sample="$work/switch-wheel-judges" exit_only=0 verdicts
  two_hosts_break_is wheel-ignores-cells && exit_only=1
  mkdir -p "$wheel_sample"; : >"$wheel_sample/calls"; : >"$wheel_sample/empty"
  printf '%s\n' '  ✗ 自证 cell-b：假的' '     → 怎么办：假的' '  ✗ 自证 cell-a：假的' '  ✗ 自证 cell-a：又一次' >"$wheel_sample/red"
  verdict_of() { # <点名的格> <退出码> <造出来的输出：red|empty>
    if crash_amplification_switch_wheel_verdict "$1" "$2" "$wheel_sample/$3" '✗ 自证 ([a-z0-9-]+)：' "$exit_only" >/dev/null; then echo 对上; else echo 对不上; fi  # gate-lint:detail 判法读的模式串，不是拒绝
  }
  verdicts="照实点名=$(verdict_of 'cell-a cell-b' 1 red) 删一格=$(verdict_of cell-a 1 red) 多一格=$(verdict_of 'cell-a cell-b cell-c' 1 red)"
  verdicts+=" 格名写错=$(verdict_of 'cell-a cell-bb' 1 red) 退出码不对=$(verdict_of 'cell-a cell-b' 0 red) 没写格=$(verdict_of '' 1 red)"
  verdicts+=" 没列而一格不红=$(verdict_of - 0 empty) 没列而红了=$(verdict_of - 1 red)"
  printf '%s\n' "$verdicts" >"$wheel_sample/output"
  if [[ "$verdicts" != "照实点名=对上 删一格=对不上 多一格=对不上 格名写错=对不上 退出码不对=对不上 没写格=对不上 没列而一格不红=对上 没列而红了=对不上" ]]; then
    cell_failed switch-wheel-judges "开关轮的判法应当只认照实点名的（退 1、红的格逐格相等）与没列而一格不红的（退 0），别的都判对不上；实判「$verdicts」" \
      "看 crash-amplification-lib.sh 的 crash_amplification_switch_wheel_verdict：第五个参数是 0 时要逐格比，点名的格是空串一律对不上"
  fi

  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  if crash_amplification_switch_wheel_inner; then echo "  ✓ --selftest（开关轮里起的一份，不再起开关轮）：$cells 格都对"; exit 0; fi

  # 开关轮（文件头「弄坏开关」）：从文件头现读开关与点名的格、与代码里认的开关比齐，逐个打开再跑一遍 --selftest，红的格要与点名的逐格相等
  local table assignment named in_code in_library in_table_two="" in_table_item="" duplicated stray switch round_failures=0 listed_count unlisted_count=0
  switch_wheel_assignments=(); switch_wheel_expected=()
  local switch_table_source="${CRASH_AMPLIFICATION_TWO_HOSTS_SWITCH_TABLE:-$two_hosts_script}"
  [[ "$switch_table_source" == "$two_hosts_script" ]] || echo "  · 开关轮：开关表读自 $switch_table_source（CRASH_AMPLIFICATION_TWO_HOSTS_SWITCH_TABLE），代码里认的开关取自 $two_hosts_script"
  table="$(crash_amplification_read_switch_table "$switch_table_source" CRASH_AMPLIFICATION_TWO_HOSTS_BREAK CRASH_AMPLIFICATION_ITEM_BREAK)"
  while IFS=$'\t' read -r assignment named; do
    [[ -n "$assignment" ]] || continue
    switch_wheel_assignments+=("$assignment"); switch_wheel_expected+=("$named")
    case "$assignment" in
      CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=*) in_table_two+="${assignment#*=}"$'\n' ;;
      *) in_table_item+="${assignment#*=}"$'\n' ;;
    esac
  done <<<"$table"
  listed_count="${#switch_wheel_assignments[@]}"
  if (( listed_count == 0 )); then
    echo "  ✗ 开关轮：文件头「弄坏开关」那张表一个开关都没读到" >&2
    echo "     → 怎么办：表里一行「#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（文件头「弄坏开关」一段的写法）" >&2
    exit 1
  fi
  duplicated="$(printf '%s\n' "${switch_wheel_assignments[@]}" | LC_ALL=C sort | uniq -d | tr '\n' ' ')"
  if [[ -n "$duplicated" ]]; then
    echo "  ✗ 开关轮：文件头表里同一个开关列了不止一次：$duplicated" >&2
    echo "     → 怎么办：一个开关只写一行，点名的格都写在它下面" >&2
    round_failures=$((round_failures + 1))
  fi
  in_code="$(grep -oE 'two_hosts_break_is [a-z0-9-]+' "$two_hosts_script" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table_two="$(printf '%s' "$in_table_two" | LC_ALL=C sort -u)"
  if [[ "$in_code" != "$in_table_two" ]]; then
    echo "  ✗ 开关轮：代码里 two_hosts_break_is 认的开关与文件头表里列的双机开关不一样（只在代码里：$(LC_ALL=C comm -23 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table_two") | tr '\n' ' ')；只在表里：$(LC_ALL=C comm -13 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table_two") | tr '\n' ' ')）" >&2
    echo "     → 怎么办：代码里每一处 two_hosts_break_is 认的开关都在表里写一行（连同点名的格），表里每一行都在代码里有对应的一处" >&2
    round_failures=$((round_failures + 1))
  fi
  in_library="$(grep -oE 'crash_amplification_item_break_is [a-z0-9-]+' "$two_hosts_script_directory/crash-amplification-lib.sh" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table_item="$(printf '%s' "$in_table_item" | LC_ALL=C sort -u)"
  stray="$(LC_ALL=C comm -13 <(printf '%s\n' "$in_library") <(printf '%s\n' "$in_table_item") | sed '/^$/d' | tr '\n' ' ')"
  if [[ -n "$stray" ]]; then
    echo "  ✗ 开关轮：文件头表里列的共用库开关，共用库里不认：$stray" >&2
    echo "     → 怎么办：照 crash-amplification-lib.sh 里 crash_amplification_item_break_is 认的名字写，不认的那一行删掉" >&2
    round_failures=$((round_failures + 1))
  fi
  # 共用库认而表里没列的：也逐个打开，要退 0、一格不红（点名写「-」）
  for switch in $(LC_ALL=C comm -23 <(printf '%s\n' "$in_library") <(printf '%s\n' "$in_table_item")); do
    switch_wheel_assignments+=("CRASH_AMPLIFICATION_ITEM_BREAK=$switch"); switch_wheel_expected+=(-); unlisted_count=$((unlisted_count + 1))
  done
  crash_amplification_switch_wheel "$work" '✗ 自证 ([a-z0-9-]+)：' "$exit_only" -- bash "$two_hosts_script" --selftest
  round_failures=$((round_failures + switch_wheel_failures))
  if (( round_failures > 0 )); then
    echo "  ✗ --selftest：$cells 格都对，开关轮有 $round_failures 处对不上" >&2
    echo "     → 怎么办：红的格与表里点名的不同，要么改那一处判法让开关红在点名的格，要么照实改文件头那一行；单看一个开关：CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=<开关> $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：两台分份取回导入对账为空退 0、录入段与本机每一趟判段用同一个库（给了 --library 用它、没给用默认库）、导入那一行（含红了之后先导入的）带 import_run=true、快照送第二台并复用、快照拷不成退 2、本机红先导入再原样退、本机卡用不了与给定状态表借不到显存退 78、本机与第二台那一份显存不够原样退 245、第二台连不上退 78、"
  echo "    分份途中被停先把 TERM 转给第二台那一趟、导入它取回的库再退 143、照崩溃点重放用例的样子造的用例（标 ignore、录入段只打 CRASH_LIBRARY_RECORDED）录入段真跑并导入对账为空、"
  echo "    第二台卡用不了先导入再退 78、库没取回退 78、对账不为空原样退、先判着按速度分、先判着那一趟卡用不了退 78、判段没跑的那一行只从导入那一趟转打一次、第二台量不出速度按日志退 78 / 78 / 248 / 249、第二台排计划借不到显存记本次未跑退 77（别的用例名的那一行照退 249）、"
  echo "    参数写错与用法错退 2、清过场没写那一份显卡配置退 78、清场判不过或没过退 78、清过场退出时不复原、"
  echo "    开不开 GPU 两台每一趟都不送 GPU_COMPUTATION_CLASSES（漏进来的也清掉）、显卡先后读多机配置（不读退役的私有显卡配置）、开了 GPU 两台每一趟 cargo test 都带 --test-threads 1（没开的不加）、"
  echo "    观察者录入按份分给两台（单独起、权重录入时就有：本机录入段只录本机那一份且带判段也带的权重、第二台先录它那一份再判、两台各写各的库、判段都读到自己库里录的；录入段红了判段不起、带停下文件导入第二台取回的；权重要等速度时仍由本机整份录）、"
  echo "    本机设了的按段登记文件（快照）与分片的段一趟判几个态（原值）都送进第二台排计划、分份那两趟、没设不送、"
  echo "    第二台那两趟与本机只编那一步的线程数取登记的重活默认并行度（显式给的优先、SINGLEFS_THREAD_CAP 设了取它、显式给的不是正整数退 78 不起；第二台那两趟判的线程数也取同一个数，不沿用本机的）"
  echo "    开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读：双机 $(grep -c . <<<"$in_table_two") 个、共用库 $(( listed_count - $(grep -c . <<<"$in_table_two") )) 个；共用库另有 $unlisted_count 个表里没列、要它一格不红）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的格"
  exit 0
}

# 本机上限默认按机器空着的量给（用户 2026-09-28：有 40G 就给 40G，不给 12G）——但要留在 slice 总上限（整机 − 余量 20G = 40.1G）之内：
# 给满 40G 时 slice 里残留的几百 MiB 就让它排不进队、等满 3600 秒退 252（2026-09-28 到 E 全域第一次起跑本机那一半就这样卡了一小时），所以给 38G
test_target=""; expand_up_to=""; memory_cap="38G"; peer_memory_cap=""; label="crash-amplification-$(date -u +%Y%m%dT%H%M%SZ)-$$"; selftest=0; threads=""; library_directory=""; test_filter=""; quiesce=0
host_weights=""; local_states_per_second=""; quiesced=0; given_states=0
while (($# > 0)); do
  case "$1" in
    --expand-up-to) expand_up_to="${2:?--expand-up-to 要一个数}"; shift 2 ;;
    --test-filter) test_filter="${2:?--test-filter 要一个用例函数名}"; shift 2 ;;
    --quiesce) quiesce=1; quiesced=1; shift ;;
    --quiesced) quiesced=1; shift ;;
    --given-states) given_states=1; shift ;;
    --host-weights) host_weights="${2:?--host-weights 要 <本机>,<第二台> 两个正整数}"; shift 2 ;;
    --local-states-per-second) local_states_per_second="${2:?--local-states-per-second 要一个正整数}"; shift 2 ;;
    --library) library_directory="${2:?--library 要一个目录}"; shift 2 ;;
    --threads) threads="${2:?--threads 要一个数}"; shift 2 ;;
    --memory-cap) memory_cap="${2:?--memory-cap 要一个上限}"; shift 2 ;;
    --peer-memory-cap) peer_memory_cap="${2:?--peer-memory-cap 要一个上限}"; shift 2 ;;
    --label) label="${2:?--label 要一个标签}"; shift 2 ;;
    --selftest) selftest=1; shift ;;
    -*) usage ;;
    *) if [[ -n "$test_target" ]]; then usage; fi; test_target="$1"; shift ;;
  esac
done

if [[ -n "$host_weights" ]] && ! [[ "$host_weights" =~ ^[1-9][0-9]*,[1-9][0-9]*$ ]] && ! two_hosts_break_is accept-bad-host-weights; then
  echo "  ✗ --host-weights 要写 <本机>,<第二台> 两个正整数，读到「$host_weights」" >&2
  echo "  → 怎么办：例 --host-weights 3,1（本机领四分之三）" >&2
  exit 2
fi
if [[ -n "$local_states_per_second" ]] && ! [[ "$local_states_per_second" =~ ^[1-9][0-9]*$ ]] && ! two_hosts_break_is accept-bad-local-speed; then
  echo "  ✗ --local-states-per-second 要一个正整数，读到「$local_states_per_second」" >&2
  echo "  → 怎么办：给本机排派活计划那一趟 CRASH_AMPLIFICATION_PLACEMENT 行里 states_per_second_on_this_host 的合计" >&2
  exit 2
fi
# 第二台那一份的上限：没给 --peer-memory-cap 就取本机那一份同一条命令的上限（两台同一套口径；配置的 PEER_MEMORY_CAP 由 multi-host-run.sh 当最大值判）
if [[ -z "$peer_memory_cap" ]] && ! two_hosts_break_is peer-cap-from-configuration; then peer_memory_cap="$memory_cap"; fi
if [[ "$selftest" == 1 ]]; then run_selftest; fi
# 退出时（每一种退法）：写上次红的清单（文件头「上次红的清单」），再做 --quiesce 那一段登记的收尾（two_hosts_exit_extra：报一句清了场、没复原）
two_hosts_exit_extra=""
two_hosts_on_exit() { # <这一趟的退出码>
  if ! two_hosts_break_is last-run-unwritten && [[ -n "$test_target" ]]; then   # 弄坏开关 last-run-unwritten：跑完不写（改前的样子）
    python3 "$repository_root/scripts/gate/lib_reuse_checks.py" last-run-target scripts/crash/crash-amplification-two-hosts.sh "$test_target" "$1" \
      ${test_filter:+--test-filter "$test_filter"} --target-file "$repository_root/crates/singlefs-checker-tier/tests/$test_target.rs" >/dev/null 2>&1 \
      || echo "  ! 上次红的清单没写成（scripts/gate/lib_reuse_checks.py last-run-target），这一趟的判定不受影响" >&2
  fi
  [[ -z "$two_hosts_exit_extra" ]] || eval "$two_hosts_exit_extra"
}
trap 'two_hosts_on_exit "$?"' EXIT
library_is_temporary=0
crash_amplification_prepare || two_hosts_break_is prepare-failure-ignored || exit 78
# GPU 对拍的测试各占卡的显存，不并行起：开了 GPU 时本机与第二台每一趟 cargo test（共用库拼的 cargo_test_arguments，本机三趟与第二台排计划、分份两趟都用它）带 --test-threads 1
# （.claude/rules/verification.md「两档：轻型与重型」）；没开的不加。弄坏开关 gpu-test-threads-unset 照改前不加
if (( gpu_switch )) && ! two_hosts_break_is gpu-test-threads-unset; then cargo_test_arguments+=(--test-threads 1); fi

# 库与日志的落点：/tmp 开机会清空（tmpfiles 的 D /tmp），2026-09-29 到 E 全域的库与树就丢在那里、续不上；正式跑一律放家目录下的固定位置，
# 重跑同一条命令就是续跑（同一判法单元指纹下判过的块全部复用）。
crash_amplification_home="${SINGLEFS_CRASH_AMPLIFICATION_HOME:-$HOME/.local/share/singlefs/crash-amplification}"
run_directory="$crash_amplification_home/runs/$label"
local_library="$(crash_amplification_default_library "$crash_amplification_home")"
# --library：本机的库跨次复用（已核过的块下一次不再核；第二台开跑前拿到它的一份快照，同样复用）。录入段与之后本机每一趟判段都用这同一个目录：
# 判段从库里读录入段写下的清单、流程与判表，录入段写进别处，判段就找不到（崩溃点重放双机那几条由 crash-point-replay-shard-run.sh 给 --library）
[[ -n "$library_directory" ]] && local_library="$library_directory"
record_library="$local_library"
# 弄坏开关 record-into-the-default-library：照改前录入段写进默认库，--library 的覆盖落在录入段之后、只管判段
two_hosts_break_is record-into-the-default-library && record_library="$(crash_amplification_default_library "$crash_amplification_home")"
mkdir -p "$run_directory" || { echo "  ✗ 建不了 $run_directory" >&2; echo "  → 怎么办：看 ${crash_amplification_home} 可不可写" >&2; exit 2; }
# 观察者版本（文件头「观察者版本」）：录入段之后记下、导入那一趟对一遍。版本取自这一趟 cargo test 在日志里打的 Running … (<测试二进制>) 那一行指到的二进制：
# 二进制里编进去一份判器键表（crates/singlefs-checker-tier/src/crash_identity.rs 的 COMPILED_JUDGING_UNITS_KEYS_TEXT，构建脚本写的 judging_units_keys.tsv，表头
# 「# singlefs entry group keys」），观察者版本由其中观察者代码那一组（observer_code:…）、观察者结论拼块那一组（observer_conclusion_composition）、池级 checker 那一组
# （judgement_of_the_crash_images）的键与工具链行拼出来（crash_amplification.rs 的 observer_versions_of_the_keys）；这里把这几组的行取出来取 sha256。
# 另读的输入（extra_inputs_digest）是跑起来才算的，二进制里没有，不在这一份摘要里。
two_hosts_observer_version_of_binary() { # <测试二进制>：打「<摘要> groups=<组数> tables=<份数> binary=<路径>」一行；读不出打原因到 stderr、退 1
  python3 -I - "$1" <<'OBSERVER_VERSION_PYTHON'
import hashlib
import mmap
import sys

path = sys.argv[1]
start = b"# singlefs entry group keys\ntoolchain\t"
recognized = (b"toolchain\t", b"group\t", b"refusal\t", b"not-compiled-entry\t", b"unit\t", b"watched\t")
relevant_groups = (b"observer_conclusion_composition", b"judgement_of_the_crash_images")
try:
    with open(path, "rb") as handle:
        view = mmap.mmap(handle.fileno(), 0, access=mmap.ACCESS_READ)
except (OSError, ValueError) as error:
    print(f"读不了测试二进制 {path}：{error}", file=sys.stderr)
    sys.exit(1)
selected, tables, at = [], 0, view.find(start)
while at >= 0:
    tables += 1
    view.seek(at)
    view.readline()
    while True:
        line = view.readline()
        if not line.endswith(b"\n") or not line.startswith(recognized):
            break
        fields = line.rstrip(b"\n").split(b"\t")
        if line.startswith(b"toolchain\t"):
            selected.append(line)
        elif line.startswith(b"group\t") and len(fields) > 1 and (fields[1].startswith(b"observer_code:") or fields[1] in relevant_groups):
            selected.append(line)
    at = view.find(start, at + 1)
if tables == 0:
    print(f"测试二进制 {path} 里找不到编进去的判器键表（# singlefs entry group keys）", file=sys.stderr)
    sys.exit(1)
groups = sum(1 for line in selected if line.startswith(b"group\t"))
if groups == 0:
    print(f"测试二进制 {path} 的判器键表里没有观察者那几组（observer_code:…、observer_conclusion_composition、judgement_of_the_crash_images）", file=sys.stderr)
    sys.exit(1)
print(f"{hashlib.sha256(b''.join(selected)).hexdigest()} groups={groups} tables={tables} binary={path}")
OBSERVER_VERSION_PYTHON
}
two_hosts_observer_version_of_log() { # <日志>：日志里最后一行 Running … (<测试二进制>) 指到的二进制的观察者版本；相对路径按树根算
  local binary
  binary="$(grep -E '^[[:space:]]+Running .* \(.+\)$' "$1" 2>/dev/null | tail -n 1 | sed -E 's/^.*\((.+)\)$/\1/')"
  if [[ -z "$binary" ]]; then echo "日志 $1 里没有 cargo 打的 Running … (<测试二进制>) 那一行（日志不在，或这一趟没经 cargo test）" >&2; return 1; fi
  [[ "$binary" == /* ]] || binary="$repository_root/$binary"
  two_hosts_observer_version_of_binary "$binary"
}
# 录入段成了之后记下（本机录入段那一趟的日志 record.log）；取不到打一行说明、导入那一趟不核
two_hosts_note_observer_version() {
  local version
  rm -f -- "$run_directory/observer-version"
  if version="$(two_hosts_observer_version_of_log "$run_directory/record.log" 2>&1)"; then
    printf '%s\n' "$version" >"$run_directory/observer-version"
    echo "  · 录入段之后记下观察者版本 ${version%% *}（$run_directory/observer-version）"
  else
    echo "  ! 观察者版本取不到，导入那一趟不核版本：$version" >&2
  fi
}
# 导入那一趟的二进制里观察者版本和录入段之后记下的对不上：退 0（置 observer_version_recorded、observer_version_now）；没记下、没法核、没变都退 1。
# 弄坏开关 observer-version-unchecked：不核（永远退 1）
observer_version_recorded="" observer_version_now=""
two_hosts_observer_version_changed() {
  two_hosts_break_is observer-version-unchecked && return 1
  [[ -s "$run_directory/observer-version" ]] || return 1
  observer_version_recorded="$(<"$run_directory/observer-version")"
  if ! observer_version_now="$(two_hosts_observer_version_of_log "$run_directory/import.log" 2>&1)"; then
    echo "  ! 导入那一趟的观察者版本取不到，没法核：$observer_version_now" >&2
    return 1
  fi
  [[ "${observer_version_recorded%% *}" != "${observer_version_now%% *}" ]]
}

# 两段（用户 2026-10-11 定）：分项驱动已经在本机跑过录入段、把选段设成 judge 才交过来；单独起这个驱动时它自己先在本机跑录入段。
# 之后本机先判、两台分份、导入每一趟（连同第二台那几趟）都只跑判段，第二台从送过去的库快照里读录入段写下的那几样；
# 单独起且权重录入时就有的，录入段不在这里先整份跑完，改成下面「分着录」：本机录自己那一份观察者、第二台自己录它那一份。
# 带 --given-states 的例外：引擎每张表先写进库再读回来判，驱动不起录入段，两台的引擎各自先录再判（本机那几趟不设选段；第二台见下面 peer_environment 后面选段那一段）
record_wanted=0
if [[ "${SINGLEFS_CRASH_AMPLIFICATION_STAGE:-}" != judge ]] && { (( ! given_states )) || two_hosts_break_is record-for-given-states; } && ! two_hosts_break_is record-skipped-when-not-judging; then
  record_wanted=1
fi
# 分着录（文件头「步骤」①②）：观察者的录入段逐状态在 CPU 上恢复、观察，挂了观察者的用例整段要几个小时；引擎录入段带 SHARE=k/n 时清单、流程、事实表照旧整条录（几十秒），
# 只有观察者的块按份筛，筛法与判段同一个（份加权重、按块序号），所以两台能各录各的那一份：本机录 0/2 那一份，第二台自己先录 1/2 那一份再判（不设选段），两台同时开跑，各写各的库。
# 权重要在录入时就定：给了 --host-weights 或什么都没给（等分）可以；只给了 --local-states-per-second（权重要等第二台量完速度才有，量速度的那一趟要本机先判着整份）
# 录入时还没有权重，仍由本机整份录、第二台只判（旧做法）。带 --given-states 的引擎自己先录再判，不走这里。
record_split=0
if (( record_wanted && ! given_states )) && [[ -z "$local_states_per_second" || -n "$host_weights" ]] && ! two_hosts_break_is record-whole-on-this-host; then
  record_split=1
fi
if (( record_wanted && ! record_split )); then
  crash_amplification_record_on_this_host "$record_library" "$run_directory/record.log" || exit $?
  two_hosts_note_observer_version
  export SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge
fi

# 本机的一趟（crash-amplification-lib.sh 的 crash_amplification_run_on_this_host）。<share> <库目录> <日志> [IMPORT=<库…>] [<附加环境 键=值>…]；两台的分工随 weights_environment 带
weights_environment=()
run_local_share() {
  local share="$1" library="$2" log="$3" import="${4:-}"
  local extra=("${weights_environment[@]+"${weights_environment[@]}"}")
  [[ -n "$import" ]] && extra+=("SINGLEFS_CRASH_AMPLIFICATION_IMPORT=$import")
  (( $# > 4 )) && extra+=("${@:5}")
  crash_amplification_run_on_this_host "$share" "$library" "$log" "${extra[@]+"${extra[@]}"}"
}

# 转打导入那一趟的 CRASH_AMPLIFICATION 行，行尾加 import_run=true（文件头「导入那一趟」那一段）；三处导入都经这里
print_the_import_lines() {
  local import_mark=" import_run=true"
  two_hosts_break_is no-import-mark && import_mark=""
  grep -h '^CRASH_AMPLIFICATION ' "$run_directory/import.log" 2>/dev/null | sed "s/\$/$import_mark/"
  # 判段没跑的那一种（文件头「输出」：配置写了 ENABLE_GPU=1 而二进制没编进 gpu 特性）打的本次未跑行只从导入那一趟转一次，入口据它记这一项本次未跑；
  #   弄坏开关 judge-not-run-dropped 下不转
  two_hosts_break_is judge-not-run-dropped && return 0
  grep -h '^CRASH_AMPLIFICATION_NOT_RUN step=judge ' "$run_directory/import.log" 2>/dev/null
}

[[ -n "$test_target" ]] || usage

# --quiesce：照配置里的清场命令先停，跑完不复原、退出时报一句「清了场、没复原」（做法同 crash-point-replay-shard-run.sh；命令与输出不打，里面有服务名）。不带就不清场：卡在额度之内用。
if [[ "$quiesce" == 1 ]]; then
  source "$peer_host_library"
  peer_host_load_configuration "$two_hosts_script_directory/crash-point-replay-shard-configuration-check.sh" "$repository_root" || two_hosts_break_is ignore-peer-configuration || { echo "  ✗ 多机配置判不过或没开双机（上面是判法的原因）" >&2; echo "  → 怎么办：修好设备，或改配置：照判法打出的出路改 multi-host.env" >&2; exit 78; }
  if ! peer_host_quiesce && ! two_hosts_break_is ignore-quiesce-failure; then
    echo "  ✗ 第二台清场没过（回读没通过）" >&2
    echo "  → 怎么办：修好设备，或改配置：看配置里 QUIESCE_STOP_COMMAND 与 QUIESCE_STOPPED_CHECK_COMMAND 在第二台上跑不跑得通" >&2
    exit 78
  fi
  if two_hosts_break_is restore-after-run; then
    two_hosts_exit_extra='peer_host_quiesce_left; bash -c "${QUIESCE_START_COMMAND:-}" >/dev/null 2>&1'
  else
    two_hosts_exit_extra='peer_host_quiesce_left'
  fi
fi

# 第二台那一趟的环境：两个开关、它是第二台、显卡配置里的优先级（树副本不带配置）；三个内核一个一个编（驱动编大内核吃内存，一起编在第二台上撞过内存上限）；
# 编译目录共用树副本上一层的 crash-amplification-target：每一趟的树副本是新拷的，不共用就每趟从头编 rocksdb 等依赖（约 3 分钟）
peer_environment=("${common_environment[@]}" ENABLE_ACROSS_MACHINES=1 "ENABLE_GPU=$gpu_switch" SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1 CARGO_TARGET_DIR=../crash-amplification-target)
# 选段：录入段已在本机整份跑完（旧做法：分项驱动交过来的、权重要等速度的、record-whole-on-this-host）、清单与观察者块在送过去的库快照里，第二台只判；
# 分着录（record_split）时第二台自己的观察者那一份只有它自己录，快照里没有（快照照在本机开录之前），只判就读不到（报「先跑录入段」），所以不设选段，第二台的引擎先录它那一份再判；
# 带 --given-states 时驱动不起录入段（上面「两段」那一段），这一趟要判的清单只有引擎判之前自己录，同样不设，第二台的引擎先录再判。
# 弄坏开关 peer-judges-only-for-given-states 下带 --given-states 也照改前设 judge；peer-judges-only-after-split-record 下分着录时也设 judge
peer_judges_only=0
if (( given_states )); then
  two_hosts_break_is peer-judges-only-for-given-states && peer_judges_only=1
elif (( record_split )); then
  two_hosts_break_is peer-judges-only-after-split-record && peer_judges_only=1
else
  peer_judges_only=1
fi
if (( peer_judges_only )); then
  peer_environment+=(SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge)
fi
[[ -n "$gpu_cards_by_priority" ]] && peer_environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_by_priority")
# 走 GPU 的计算类不送（文件头「跑法」：引擎不读 GPU_COMPUTATION_CLASSES）；弄坏开关 classes-sent-to-peer 下照改前送三类
if (( gpu_switch )) && two_hosts_break_is classes-sent-to-peer; then
  peer_environment+=("GPU_COMPUTATION_CLASSES=gpu_unit_checks crash_verify_gpu crash_judge_gpu")
fi
# 分片的段一趟判几个态（文件头「往引擎送的变量」）：本机设了（空串也算，照原样交、引擎拒）就照原值放进第二台两趟（排计划、分份）的环境，没设不送；
# 两趟用的 peer_run_environment 由 peer_threads_now 从这里拷。弄坏开关 slice-states-not-sent-to-peer 下不送
if [[ -n "${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES+set}" ]] && ! two_hosts_break_is slice-states-not-sent-to-peer; then
  peer_environment+=("SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=$SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES")
fi
# 只写不读（文件头「往引擎送的变量」）：本机设了就照原值放进第二台两趟的环境，没设不送。弄坏开关 write-only-not-sent-to-peer 下不送
if [[ -n "${SINGLEFS_CRASH_LIBRARY_WRITE_ONLY+set}" ]] && ! two_hosts_break_is write-only-not-sent-to-peer; then
  peer_environment+=("SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=$SINGLEFS_CRASH_LIBRARY_WRITE_ONLY")
fi

# 往引擎送的变量（crash-point-replay.sh 文件头「往引擎送的变量」）：本机那几趟从调用方环境继承（env 不清它们）；第二台那一趟是新起的 ssh 会话，
# 只看得见命令里送过去的，所以两趟（排计划、分份）都照送：
#   按段登记 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY：本机设了就把那份文件拷一份快照到 <日志目录>/segments-registry-for-peer/，随库快照一样经 --send 送进第二台那棵树的
#     crash-amplification-segments-registry/（树副本每趟新拷、跑完删，所以排计划、分份两趟各送一次）；变量值换成第二台上那一份的绝对路径——那个路径只有第二台自己的 shell 知道，
#     由命令最前面的 bash -c（peer_tree_path_script）用第二台的 $PWD 拼、文件不在就退 2 不起 cargo。
# 弄坏开关 segments-registry-not-sent-to-peer 下不送。
peer_tree_path_script='registry="$PWD/$2"; [[ -f "$registry" ]] || { echo "  ✗ 第二台那棵树里没有 $2（本机送过去的按段登记文件快照）" >&2; echo "  → 怎么办：看 multi-host-run.sh 的 --send 那几行有没有送到" >&2; exit 2; }; export "$1=$registry"; shift 2; exec "$@"'
peer_tree_path_prefix=(); peer_registry_send=()
if [[ -n "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY:-}" ]] && ! two_hosts_break_is segments-registry-not-sent-to-peer; then
  segments_registry_for_peer="$run_directory/segments-registry-for-peer"
  segments_registry_file_name="$(basename -- "$SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY")"
  if ! { mkdir -p -- "$segments_registry_for_peer" && cp -- "$SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY" "$segments_registry_for_peer/$segments_registry_file_name"; }; then
    echo "  ✗ 拷不了按段登记文件 $SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY 的快照给第二台（落点 $segments_registry_for_peer）" >&2
    echo "  → 怎么办：看 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY 指的文件在不在、$run_directory 可不可写" >&2
    exit 2
  fi
  peer_registry_send=(--send "$segments_registry_for_peer" crash-amplification-segments-registry)
  peer_tree_path_prefix=(bash -c "$peer_tree_path_script" peer-tree-path SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY "crash-amplification-segments-registry/$segments_registry_file_name")
fi
# 第二台那两趟的库目录（文件头步骤②）：引擎只收绝对路径（crash_library.rs 的 crash_library_directory 拒相对路径），第二台树副本的绝对路径同样只有第二台自己的 shell 知道，
#   由命令最前面的 bash -c（peer_library_path_script）用第二台的 $PWD 拼成 <树根>/crash-amplification-library（库快照没送过去时目录还不在，引擎自己建）。
#   弄坏开关 peer-library-relative 下照改前在 env 里交相对路径
peer_library_path_script='export "$1=$PWD/$2"; shift 2; exec "$@"'
peer_library_prefix=(bash -c "$peer_library_path_script" peer-tree-library SINGLEFS_CRASH_AMPLIFICATION_LIBRARY crash-amplification-library)
peer_library_environment=()
if two_hosts_break_is peer-library-relative; then
  peer_library_prefix=(); peer_library_environment=(SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=crash-amplification-library)
fi

# 第二台那一趟的线程数（文件头「线程」）：起 capped.sh 的放进 peer_threads；算不出退 78（这时这一趟还什么都没起）。
# 那一趟判的线程数 SINGLEFS_CRASH_AMPLIFICATION_THREADS 也取 peer_threads：peer_environment 里随 common_environment 带着的那一份是本机的
# （--threads 或本机核数减一），换掉它，组成这一趟的环境 peer_run_environment；弄坏开关 peer-checking-threads-from-this-host 下照改前留着本机那一份
peer_threads_now() {
  local entry
  if two_hosts_break_is peer-threads-fixed; then
    peer_threads=8
  elif ! peer_threads="$(crash_amplification_threads_now peer "${CRASH_AMPLIFICATION_PEER_THREADS:-}")"; then
    echo "  ✗ 第二台那一趟没起：它的线程数不是正整数（上面一句是原因）" >&2
    echo "  → 怎么办：CRASH_AMPLIFICATION_PEER_THREADS 写成正整数，或不设、取登记的重活默认并行度（退 78：运行条件不满足）" >&2
    exit 78
  fi
  peer_run_environment=()
  if two_hosts_break_is peer-checking-threads-from-this-host; then
    peer_run_environment=("${peer_environment[@]}")
    return 0
  fi
  for entry in "${peer_environment[@]}"; do
    [[ "$entry" == SINGLEFS_CRASH_AMPLIFICATION_THREADS=* ]] || peer_run_environment+=("$entry")
  done
  peer_run_environment+=("SINGLEFS_CRASH_AMPLIFICATION_THREADS=$peer_threads")
}

# 第二台共用编译目录的守卫（文件头「第二台的编译目录」）：目录取 peer_environment 里的 CARGO_TARGET_DIR（同一个值，不另写一份），起 multi-host-run.sh 的两处都带
#   MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY=<它>。弄坏开关 peer-shared-target-unguarded 下照改前不带
peer_target_directory=""
for peer_environment_entry in "${peer_environment[@]}"; do
  [[ "$peer_environment_entry" == CARGO_TARGET_DIR=* ]] && peer_target_directory="${peer_environment_entry#CARGO_TARGET_DIR=}"
done
peer_shared_target_environment=()
if [[ -n "$peer_target_directory" ]] && ! two_hosts_break_is peer-shared-target-unguarded; then
  peer_shared_target_environment=("MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY=$peer_target_directory")
fi

# 给了本机的速度：第二台先排一趟派活计划、量它的速度，两台按速度分（四舍五入到整数，至少 1）
if [[ -z "$host_weights" && -n "$local_states_per_second" ]]; then
  peer_threads_now
  # 第二台量速度的那段时间本机先判着（整份），量完叫它停
  warm_stop_file="$run_directory/stop-the-run-on-this-host"
  rm -f -- "$warm_stop_file"
  # 第二台排计划那一趟同样只跑判段：本机先判着之前照一份库的快照（录入段写下的流程、事实表与判表在里面）送过去
  placement_library_send=()
  if [[ -f "$local_library/CURRENT" ]] && ! two_hosts_break_is no-library-snapshot; then
    library_for_peer_placement="$run_directory/library-for-peer-placement"
    cp -a -- "$local_library" "$library_for_peer_placement" || two_hosts_break_is ignore-snapshot-failure || { echo "  ✗ 拷不了库的快照给第二台排计划那一趟" >&2; echo "  → 怎么办：看 $run_directory 可不可写" >&2; exit 2; }
    placement_library_send=(--send "$library_for_peer_placement" crash-amplification-library)
  fi
  crash_amplification_run_on_this_host 0/1 "$local_library" "$run_directory/warm-local.log" "SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=$warm_stop_file" &
  warm_job=$!
  placement_arguments=(--peer --label "$label-placement" "${peer_registry_send[@]+"${peer_registry_send[@]}"}" "${placement_library_send[@]+"${placement_library_send[@]}"}")
  [[ -n "$peer_memory_cap" ]] && placement_arguments+=(--memory-cap "$peer_memory_cap")
  env "${peer_shared_target_environment[@]+"${peer_shared_target_environment[@]}"}" bash "$multi_host_runner" "${placement_arguments[@]}" -- \
    "${peer_library_prefix[@]+"${peer_library_prefix[@]}"}" "${peer_tree_path_prefix[@]+"${peer_tree_path_prefix[@]}"}" \
    env "${peer_run_environment[@]}" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/1 "${peer_library_environment[@]+"${peer_library_environment[@]}"}" \
    bash scripts/hosts/capped.sh "$peer_threads" cargo "${cargo_test_arguments[@]}" >"$run_directory/peer-placement.log" 2>&1
  peer_placement_exit=$?
  two_hosts_break_is never-stop-warm-run || : > "$warm_stop_file"
  # 只供测试（文件头「只供测试」CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED）：叫停这一步走过了，在停下文件之后建
  [[ -n "${CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED:-}" ]] && : > "$CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED"
  wait "$warm_job"; warm_exit=$?
  rm -f -- "$warm_stop_file"
  # 有意叫停的那一趟：CRASH_AMPLIFICATION 行尾加 warm_up=true（文件头「本机先判着」那一段）
  warm_up_mark=" warm_up=true"
  two_hosts_break_is no-warm-up-mark && warm_up_mark=""
  grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL|_STOPPED)? ' "$run_directory/warm-local.log" 2>/dev/null | sed "/^CRASH_AMPLIFICATION /s/\$/$warm_up_mark/"
  if [[ "$warm_exit" != 0 ]] && ! two_hosts_break_is warm-failure-ignored; then
    warm_exit_code="$(crash_amplification_exit_code_of_a_failed_run "$warm_exit" "$run_directory/warm-local.log")"
    echo "  ✗ 本机先判着的那一趟退 $warm_exit；日志 $run_directory/warm-local.log" >&2
    if two_hosts_exit_code_is_explained "$warm_exit_code"; then crash_amplification_explain_exit_code "$warm_exit_code"; else
      echo "  → 怎么办：看日志里 test result、panicked 与 run-with-memory-cap 那几行；判过的块已经在库里，重跑同一条命令接着判" >&2
    fi
    exit "$warm_exit_code"
  fi
  grep -h '^CRASH_AMPLIFICATION_PLACEMENT ' "$run_directory/peer-placement.log" 2>/dev/null
  peer_states_per_second="$(crash_amplification_placement_rate "$run_directory/peer-placement.log")"
  # 第二台排计划那一趟借不到显存（文件头「退出码」77）：那一趟退 0、没打计划行、行首是产品路径借不到显存的那一行。
  #   弄坏开关 peer-placement-not-run-as-failure 下不认（照没量出速度退 249），peer-placement-accept-any-not-run-line 下别的用例名的也认
  peer_not_run_prefix='NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason='
  two_hosts_break_is peer-placement-accept-any-not-run-line && peer_not_run_prefix='NOT_RUN_NO_GPU_ADAPTER test='
  peer_not_run_line=""
  if [[ "$peer_placement_exit" == 0 && -z "$(grep -m 1 '^CRASH_AMPLIFICATION_PLACEMENT ' "$run_directory/peer-placement.log" 2>/dev/null)" ]] \
     && ! two_hosts_break_is peer-placement-not-run-as-failure; then
    peer_not_run_line="$(awk -v prefix="$peer_not_run_prefix" 'index($0, prefix) == 1 { print; exit }' "$run_directory/peer-placement.log" 2>/dev/null)"
  fi
  if [[ -n "$peer_not_run_line" ]]; then
    echo "CRASH_AMPLIFICATION_NOT_RUN step=peer-placement test=crash_amplification::run_from_environment reason=${peer_not_run_line#*reason=}"
    echo "  ! 本次未跑（这不是通过）：第二台排派活计划那一趟借不到显存，两台都没分份判：${peer_not_run_line#*reason=}；日志 $run_directory/peer-placement.log" >&2
    echo "  · 等第二台上占着卡的 GPU 用例或测试进程跑完，再重跑同一条命令（本机先判着的块在库里，接着判）；配置了两台就要两台都跑，不改在本机一台跑" >&2
    exit 77
  fi
  if [[ "$peer_placement_exit" != 0 || -z "$peer_states_per_second" || "$peer_states_per_second" == 0 ]] && ! two_hosts_break_is peer-placement-failure-ignored; then
    echo "CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED exit=$peer_placement_exit"
    echo "  ✗ 第二台排派活计划那一趟没量出速度（退 $peer_placement_exit）；日志 $run_directory/peer-placement.log" >&2
    # 照日志分类：multi-host-run.sh 自己没跑成（255）或卡用不了 78，算法上判不了 248，认不出 249
    peer_placement_exit_code=249
    if [[ "$peer_placement_exit" == 255 ]] && ! two_hosts_break_is peer-unreachable-as-run-failure; then peer_placement_exit_code=78
    elif [[ "$peer_placement_exit" == 245 ]] && ! two_hosts_break_is gpu-memory-unexplained; then peer_placement_exit_code=245   # 那边的内存包装认出显存不够
    elif ! two_hosts_break_is peer-placement-unclassified; then
      case "$(crash_amplification_failure_kind "$run_directory/peer-placement.log")" in
        device|build) peer_placement_exit_code=78 ;;
        algorithm) peer_placement_exit_code=248 ;;
      esac
    fi
    if two_hosts_exit_code_is_explained "$peer_placement_exit_code"; then crash_amplification_explain_exit_code "$peer_placement_exit_code"; else
      echo "  → 怎么办：看日志里 GPU_CARD_NOT_USABLE、panicked、run-with-memory-cap 与 multi-host-run.sh 那几行（250 是第二台那一趟撞了内存上限）；配置了两台就要两台都跑成，修好第二台再重跑同一条命令" >&2
    fi
    exit "$peer_placement_exit_code"
  fi
  host_weights="$local_states_per_second,$peer_states_per_second"
fi
if [[ -n "$host_weights" ]]; then
  two_hosts_break_is drop-host-weights || weights_environment=("SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=$host_weights")
  echo "CRASH_AMPLIFICATION_HOST_WEIGHTS local,peer=$host_weights"
fi

peer_arguments=(--peer --label "$label" --fetch crash-amplification-library "${peer_registry_send[@]+"${peer_registry_send[@]}"}")
if [[ -f "$local_library/CURRENT" ]] && ! two_hosts_break_is no-library-snapshot; then
  # 库开着的时候不能拷（RocksDB 的文件在变）：本机那一份开跑之前先照一份快照，送给第二台当它的起点
  library_for_peer="$run_directory/library-for-peer"
  cp -a -- "$local_library" "$library_for_peer" || two_hosts_break_is ignore-snapshot-failure || { echo "  ✗ 拷不了库的快照给第二台" >&2; echo "  → 怎么办：看 $run_directory 可不可写" >&2; exit 2; }
  peer_arguments+=(--send "$library_for_peer" crash-amplification-library)
fi
peer_threads_now
[[ -n "$peer_memory_cap" ]] && peer_arguments+=(--memory-cap "$peer_memory_cap")
fetch_directory="$run_directory/fetched"
fetched_library="$fetch_directory/crash-amplification-library"
peer_share=1/2
two_hosts_break_is same-share-on-both-hosts && peer_share=0/2

# 被停（文件头「被停」）：分份那两趟都放后台、驱动在 wait 里等，收到 TERM 时 trap 当场就跑——记一笔、把 TERM 只转给这里记下的两个进程号
forward_term_to_own_run() { # <进程号>：它还是这个驱动的子进程（/proc 里父进程号是 $$；收过之后进程号可能被别的进程重用）才发 TERM，只发这一个
  local pid="$1" process_status="" after_name=""
  [[ -n "$pid" ]] || return 0
  { read -r process_status <"/proc/$pid/stat"; } 2>/dev/null || return 0
  after_name="${process_status##*) }"   # 第二段（进程名）可以带空格与括号：取最后一个「) 」之后，依次是状态、父进程号
  after_name="${after_name#* }"
  [[ "${after_name%% *}" == "$$" ]] || return 0
  kill -TERM "$pid" 2>/dev/null
  return 0
}
wait_for_own_run() { # <进程号>：等这一趟真退出，退出码放 run_exit。wait 被 TERM 的 trap 打断（trap 置 two_hosts_wait_interrupted）就接着等；
  # 接着等时它已经收过（wait 退 127）就留上一次的码
  local status
  run_exit=""
  while :; do
    two_hosts_wait_interrupted=0
    wait "$1"; status=$?
    if [[ -n "$run_exit" && "$status" == 127 ]]; then return 0; fi
    run_exit="$status"
    (( two_hosts_wait_interrupted )) || return 0
  done
}
two_hosts_term_received=0; two_hosts_wait_interrupted=0; local_pid=""; peer_pid=""
# 本机那一份的整条（后台子 shell 里跑）：分着录时先录本机那一份观察者（SHARE=0/2、带判段也带的权重，日志 record.log），录成了再判 0/2；没分着录就只判 0/2。
#   录入段没成：留一份标记 local-record-failed（主流程据此知道退出码已经是照日志分类之后的、没有判段日志），退它的退出码，不判。
#   弄坏开关 record-weights-dropped 下录入段不带权重（录的是等分的那一份，判段按权重领别的块，读不到）；split-record-failure-ignored 下录入段红了也接着判
local_record_failed_marker="$run_directory/local-record-failed"
rm -f -- "$local_record_failed_marker"
run_local_record_then_share() {
  local record_exit record_extra=()
  if (( record_split )); then
    two_hosts_break_is record-weights-dropped || record_extra=("${weights_environment[@]+"${weights_environment[@]}"}")
    crash_amplification_record_on_this_host "$record_library" "$run_directory/record.log" 0/2 "${record_extra[@]+"${record_extra[@]}"}"; record_exit=$?
    if (( record_exit != 0 )) && ! two_hosts_break_is split-record-failure-ignored; then
      : > "$local_record_failed_marker"
      return "$record_exit"
    fi
    two_hosts_note_observer_version
    export SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge
  fi
  run_local_share 0/2 "$local_library" "$run_directory/share-0.log"
}
trap 'two_hosts_term_received=1; two_hosts_wait_interrupted=1; forward_term_to_own_run "$local_pid"; forward_term_to_own_run "$peer_pid"' TERM
run_local_record_then_share &
local_pid=$!
MULTI_HOST_RUN_FETCH_DIRECTORY="$fetch_directory" env "${peer_shared_target_environment[@]+"${peer_shared_target_environment[@]}"}" bash "$multi_host_runner" "${peer_arguments[@]}" -- \
  "${peer_library_prefix[@]+"${peer_library_prefix[@]}"}" "${peer_tree_path_prefix[@]+"${peer_tree_path_prefix[@]}"}" \
  env "${peer_run_environment[@]}" "${weights_environment[@]+"${weights_environment[@]}"}" "SINGLEFS_CRASH_AMPLIFICATION_SHARE=$peer_share" "${peer_library_environment[@]+"${peer_library_environment[@]}"}" \
  bash scripts/hosts/capped.sh "$peer_threads" cargo "${cargo_test_arguments[@]}" >"$run_directory/share-1.log" 2>&1 &
peer_pid=$!
# 第二台那一趟起之前就收到了 TERM（trap 跑时 peer_pid 还空着）：补发给它
if (( two_hosts_term_received )); then forward_term_to_own_run "$peer_pid"; fi
wait_for_own_run "$peer_pid"; peer_exit="$run_exit"; peer_pid=""
wait_for_own_run "$local_pid"; local_exit="$run_exit"; local_pid=""
grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL)? ' "$run_directory/share-0.log" "$run_directory/share-1.log" 2>/dev/null
# 导入那一趟的选段（分着录时）：本机那一份的录入段成了就只判（跟旧做法一样，判段只从库里读）；录入段没成，或被停了（录入段可能停在半路），导入那一趟不设选段，
#   由引擎先核清单与观察者块再导入——这两种导入都带停下文件（下面），录入段在块边上立刻停下，不会重新录几个小时
local_record_failed=0
[[ -e "$local_record_failed_marker" ]] && local_record_failed=1
if (( record_split )); then
  if (( local_record_failed || two_hosts_term_received )); then unset SINGLEFS_CRASH_AMPLIFICATION_STAGE; else export SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge; fi
fi
stop_the_import_environment=()
if (( local_record_failed )); then
  : > "$run_directory/stop-before-the-import-judges"
  stop_the_import_environment=("SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=$run_directory/stop-before-the-import-judges")
fi
if (( two_hosts_term_received )); then
  trap 'two_hosts_term_received=1' TERM   # 导入那一趟起着时再来的 TERM 只记一笔，不打断导入
  echo "  ✗ 分份途中收到 TERM：本机那一份退 $local_exit、第二台那一份退 $peer_exit；日志 $run_directory/" >&2
  if [[ ! -f "$fetched_library/CURRENT" ]]; then
    echo "  · 第二台的库没取回（$fetched_library/CURRENT 不在），没有可导入的" >&2
  elif ! two_hosts_break_is term-skips-import; then
    # 先导入再退：停下文件先建好，导入那一趟只导入第二台判过的块，本机这一份没判的块不再领
    : > "$run_directory/stop-before-the-import-judges"
    run_local_share 0/2 "$local_library" "$run_directory/import.log" "$fetched_library" "SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=$run_directory/stop-before-the-import-judges"; term_import_exit=$?
    print_the_import_lines
    echo "  · 第二台取回的库已导入本机库（导入那一趟退 $term_import_exit；带着停下文件，本机这一份没判的块没领）" >&2
  fi
  echo "  → 怎么办：被停的这一趟不算结果（退 143）；判过的块都在库里，重跑同一条命令接着判" >&2
  exit 143
fi
trap - TERM
if [[ "$local_exit" != 0 ]] && ! two_hosts_break_is local-failure-ignored; then
  if (( local_record_failed )); then
    echo "  ✗ 本机那一份的观察者录入段退 $local_exit（分着录，SHARE=0/2；退出码已经照 record.log 分类）；日志 $run_directory/record.log，判段没起" >&2
    echo "  → 怎么办：看 $run_directory/record.log 里录入段红在哪一步，修好后照原样重跑同一条命令（判过的块照键续跑）" >&2
  else
    echo "  ✗ 本机那一份退 $local_exit；日志 $run_directory/share-0.log" >&2
    echo "  → 怎么办：看 $run_directory/share-0.log 末尾的红行，修好后照原样重跑同一条命令（判过的块照键续跑）" >&2
  fi
  if [[ -f "$fetched_library/CURRENT" ]] && ! two_hosts_break_is local-failure-skips-salvage; then
    # 第二台那一份判过的块已经取回来了：先导进本机库再退，不因为本机这边红了就把第二台判的丢掉；下一次同一条命令接着跑
    #   （本机录入段红了的：带停下文件导入，只导入第二台判过的块，不重新录观察者）
    run_local_share 0/2 "$local_library" "$run_directory/import.log" "$fetched_library" "${stop_the_import_environment[@]+"${stop_the_import_environment[@]}"}"; salvage_exit=$?
    print_the_import_lines
    echo "  · 第二台取回的库已导入本机库（导入那一趟退 $salvage_exit）" >&2
  fi
  if (( local_record_failed )); then local_exit_code="$local_exit"; else
    local_exit_code="$(crash_amplification_exit_code_of_a_failed_run "$local_exit" "$run_directory/share-0.log")"
  fi
  if two_hosts_exit_code_is_explained "$local_exit_code"; then crash_amplification_explain_exit_code "$local_exit_code"; else
    echo "  → 怎么办：看日志里 test result、panicked 与 run-with-memory-cap 那几行（250–254 是内存包装自己的结局）" >&2
  fi
  exit "$local_exit_code"
fi
if [[ "$peer_exit" != 0 ]] && ! two_hosts_break_is peer-failure-ignored; then
  echo "  ✗ 第二台那一份退 $peer_exit；日志 $run_directory/share-1.log" >&2
  if [[ -f "$fetched_library/CURRENT" ]] && ! two_hosts_break_is peer-failure-skips-salvage; then
    # 第二台判到一半的库取回来了：先导进本机库再退，它判过的块不丢；下一次同一条命令接着跑
    run_local_share 0/2 "$local_library" "$run_directory/import.log" "$fetched_library"; salvage_exit=$?
    print_the_import_lines
    echo "  · 第二台判到一半的库已导入本机库（导入那一趟退 $salvage_exit）：重跑同一条命令接着判" >&2
  fi
  # multi-host-run.sh 自己没跑成（255：配置、工具链、拷树、连不上、等不到结局）是第二台用不上，退 78；别的照 share-1.log 分类
  if [[ "$peer_exit" == 255 ]] && ! two_hosts_break_is peer-unreachable-as-run-failure; then peer_exit_code=78; else
    peer_exit_code="$(crash_amplification_exit_code_of_a_failed_run "$peer_exit" "$run_directory/share-1.log")"
  fi
  if two_hosts_exit_code_is_explained "$peer_exit_code"; then crash_amplification_explain_exit_code "$peer_exit_code"; else
    echo "  → 怎么办：看日志里第二台的 test result 与 panicked（250–254 是内存包装自己的结局）" >&2
  fi
  exit "$peer_exit_code"
fi
if [[ ! -d "$fetched_library" ]] && ! two_hosts_break_is accept-missing-fetched-library; then
  echo "  ✗ 第二台的库没取回：$fetched_library 不在" >&2
  echo "  → 怎么办：修好设备，或改配置：看 share-1.log 里 multi-host-run.sh --fetch 那几行，第二台那一趟的库落在哪、取回时连没连上（退 78）" >&2
  exit 78
fi
if two_hosts_break_is skip-import; then
  : > "$run_directory/import.log"; import_exit=0
else
  run_local_share 0/2 "$local_library" "$run_directory/import.log" "$fetched_library"; import_exit=$?
fi
print_the_import_lines
# 观察者版本（文件头「观察者版本」）：导入那一趟用的二进制和录入段之后记下的对不上：红了不是漏录，退 78 叫人重跑整趟；判绿了只说一声
if two_hosts_observer_version_changed; then
  if [[ "$import_exit" != 0 ]]; then
    echo "  ✗ 录入段之后到导入那一趟之间，测试二进制里编进去的判器单元变了：观察者版本录入段 ${observer_version_recorded%% *}，导入那一趟 ${observer_version_now%% *}（二进制见 $run_directory/observer-version 与 import.log 里 Running 那一行）；导入那一趟退 $import_exit，读不到录入段按旧版本录的块，不是录入段漏录" >&2
    echo "  → 怎么办：两趟之间判器单元变了，重跑整趟（同一条命令，从录入段起，别只补录入段）；树在中途被改过的，在冻住的树里跑" >&2
    exit 78
  fi
  echo "  ! 录入段之后到导入那一趟之间观察者版本变了（录入段 ${observer_version_recorded%% *}，导入那一趟 ${observer_version_now%% *}），导入那一趟照样对账为空、0 红：结论按导入那一趟的二进制算" >&2
fi
if [[ "$import_exit" != 0 ]] && ! two_hosts_break_is import-failure-ignored; then
  echo "  ✗ 导入第二台的库之后那一趟退 $import_exit（对账不为空或有红）；日志 $run_directory/import.log" >&2
  import_exit_code="$(crash_amplification_exit_code_of_a_failed_run "$import_exit" "$run_directory/import.log")"
  if two_hosts_exit_code_is_explained "$import_exit_code"; then crash_amplification_explain_exit_code "$import_exit_code"; else
    echo "  → 怎么办：看 import.log 里 CRASH_AMPLIFICATION 那一行的 unjudged_blocks 与 red，与 panicked 那几行" >&2
  fi
  exit "$import_exit_code"
fi
run_logs=("$run_directory"/*.log)
echo "  ✓ 双机各录各核、第二台的库拷回导入、对账为空、0 红（${#run_logs[@]} 份日志在 $run_directory）"
