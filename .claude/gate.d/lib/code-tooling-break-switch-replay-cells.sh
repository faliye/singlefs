#!/usr/bin/env bash
# 门禁 code-tooling 的格函数库：break-switch-replay、kb-directories-write-scope、prune-server-selftest。
# 由 .claude/gate.d/code-tooling.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法库变量、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# 各格共用的全局变量由入口设好，别份库里的函数与变量在调用时才取。

# ── 格 break-switch-replay：还 checks-owed.md C625（弄坏开关文件头三样没有门禁）复跑方那一半（格式那一半是 break-switch-headers 格）。
# 判据：登记表 .claude/gate.d/break-switch-replay-files.tsv（被判的根里那一份，「<相对仓根的路径><TAB><每条开关最多几秒>[<TAB><基准趟最多几秒>]<TAB># 理由>」）里每份脚本或钩子，
#   要在被判的根里读弄坏开关、文件头开关表每条三样写全（break-switch-headers 格的读法）、又在 .claude/hooks/selftest-registry.tsv 里登记成自证（selftest）：
#   先不设开关起一趟这份的 `--selftest`（基准趟，其余名字以 _BREAK 结尾的变量摘掉），要退 0；它打的失败行（带 ✗ 的行）是自证本来就有的
#   （被测的那一段该打的拒绝行），不算开关弄红的。再对表里每一条开关，在环境里设上它、照样起 `--selftest`：
#   这一趟的失败行减去基准趟的（整行去掉临时路径与数字之后比，按次数抵），剩下的是这条开关弄红的行；
#   点名的项取下一行起 「…」 里的原文（嵌套的括号配平，原文里的 … 是中间省掉的一段，各段按次序出现在同一行里）。
#   点名的格与实际红的格逐格相同：点名的每一项都要在剩下的行里照出来（少红的判「点名的项没在失败行里」）；剩下的行里以 ✗ 起头的每一行都要被某一项照出来
#   （多红的判「失败行没被点名」，逐行列）。不以 ✗ 起头、只在行中间带 ✗ 的行（自证转述被测程序输出的那几行）只拿来照点名的项，不要求被点名。
#   汇总行不要求点名：同一份脚本里红了的开关有一半以上（至少两条）下都多红的同一行（去掉临时路径与数字之后相同），是自证的汇总句（「N 处判错」这类），不当一格。
#   输出按字节收、照 UTF-8 解（stdout 与 stderr 各解各的）；解不了的那一路照 U+FFFD 换掉坏字节接着照判，另记一条「坏字节」红、点名哪一路第几字节；
#   单独的 \r 与 \r\n 都当换行（与按文本收时同一个认法），不留在行里把两行接成一行。
#   结局：点名的与实际红的逐格相同是对；退 2 并点名这个变量只在自证里认的是「自证自带开关轮」（C622 的守卫在外面设不进去，开关轮在自证里面，不在这里复跑）；
#   退 0（开关没弄坏什么）、点名的项里没有可比的「…」原文、少红或多红、超时、有坏字节，都判红；基准趟退非 0 或超时，那一份判红、开关趟不起。
#   不是对的先单独重跑一次再定，防并行下计时类自证偶然红（基准趟到点的重不重跑、拿几个线程照下面「重跑」）。
#   各条互不依赖，按每台默认重活线程数（至多 8）并行（基准趟先齐，再起开关趟；一份的开关趟齐了就当场判这一份、不等别的份），
#   收回的条数与派出去的对不上整格红。
# 重跑：要重跑的那几条（这一趟自己起的头一趟判得不是对的；开关趟取回的落盘趟判得不是对的不在内，那几趟先并行重起，见下面「开关趟落盘」）先记下，
#   别的趟（基准趟、全部开关趟）都收完之后一次一条地起（按开关在登记里的次序）；基准趟要重跑的同样等全部基准趟收完、一次一份。
#   重跑时没有别的趟在跑：并行下超时的那一批不再一起起步、叠在一起撞内存上限，重跑的结局也不受并行拖慢。
#   基准趟要重跑的起因与那一趟的线程数：
#     到点停整组、而头一趟拿到的里层线程数小于登记的线程数（基准趟那一段按份数分过；取回的落盘趟拿几个线程不落盘，也算这一种）：拿满登记的线程数、同一个限时重跑，
#     起跑前打一行「基准趟拿满线程重跑：<路径>（头一趟里层 N 个线程、跑了 S 秒到点停整组；这一趟里层 T 个线程、同一个限时 L 秒）」；结局取重跑那一趟：
#     过了照常起它的开关趟、照常按键记库，还到点判红，失败行接「头一趟里层 N 个线程、跑了 S 秒到点，拿满 T 个线程重跑那一趟跑了 S2 秒」；
#     到点停整组、头一趟已经拿满登记的线程数：不重跑、判红，失败行接「头一趟里层已拿满登记的 T 个线程、跑了 S 秒到点，不重跑」；
#     退非 0 或留下进程（没到点）：照头一趟那一段的线程数单独重跑一次再定。
#   样本 break-switch-replay-baseline-split-timeout-rerun-green（样本根 .break-switch-replay-jobs 给上限 100 GiB、线程 8；slow.sh、quick.sh 各一条开关，每条开关最多 3 秒：
#   基准趟外层 2 条 × 里层 4；slow.sh 的自证在基准趟里读到的 SINGLEFS_THREAD_CAP 小于 8 就睡过限时、等于 8 当场过：必须判绿，输出里有 slow.sh 那一行「基准趟拿满线程重跑」，
#   成功句报复跑了 2 条开关、逐格相同 2 条）；break-switch-replay-baseline-full-timeout-red（线程 4、一份 stall.sh 一条开关，每条开关最多 3 秒：基准趟拿满 4 个线程，
#   头一趟在样本根留一个记号再睡过限时、再起一趟看到记号当场过：必须判红，失败行带「不重跑」那一句；重跑了就判成绿）。
#   弄坏开关 CODE_TOOLING_BREAK=replay-baseline-timeout-rerun-same-threads：到点的基准趟一律照头一趟那一段的线程数重跑一次（分过的不拿满，拿满的也再跑一趟），
#   break-switch-replay-baseline-split-timeout-rerun-green 判红（期望退出 0，实测 1：slow.sh 重跑那一趟仍拿 4 个线程、又到点）、
#   break-switch-replay-baseline-full-timeout-red 判绿（期望退出 1，实测 0：重跑那一趟看到记号就过了）。
#   样本 break-switch-replay-reruns-serial-green（样本根 .break-switch-replay-jobs 给上限 100 GiB、线程 31：flaky.sh 两条开关头一趟都没红、要重跑，
#   steady.sh 一条开关趟要跑 3 秒；每一趟起跑时在 running/ 下登一个文件、退出前删掉，重跑的那一趟起跑与睡 1 秒之后各看一次 running/ 里有没有别的趟，
#   看到过就记下、此后每一趟都多打一行没点名的 ✗：必须判绿，成功句报复跑了 3 条开关、逐格相同 3 条）。
#   弄坏开关 CODE_TOOLING_BREAK=replay-reruns-batched：一份的开关趟齐了、要重跑的当场派进池子（与还在跑的 steady.sh、同一批要重跑的另一条一起起），
#   break-switch-replay-reruns-serial-green 判红（期望退出 0，实测 1：「scripts/gate/flaky.sh FLAKY_BREAK=prefix：不符」，失败行「✗ 自检：重跑时另有别的趟在跑」没被点名）。
#   开关趟的重跑一次一条、那一段只它一条在跑：里层线程数照那一段同时在跑的条数分（线程数 ÷ 1，拿满登记的线程数），SINGLEFS_THREAD_CAP 照它设；
#   弄坏开关 replay-reruns-batched 下重跑与别的趟一起在池子里起，照开关趟那一段分过的数。
#   样本 break-switch-replay-rerun-threads-full-green（样本根 .break-switch-replay-jobs 给上限 100 GiB、线程 8；again.sh 两条开关，开关趟那一段外层 3 条 × 里层 2：
#   其中 AGAIN_BREAK=late 头一趟没红、要重跑，AGAIN_BREAK=plain 照常红；自证在基准趟里核 SINGLEFS_THREAD_CAP 是 8、头一趟开关趟里核是 2、重跑那一趟核是 8，
#   哪一趟不对就记下、此后每一趟多打一行没点名的 ✗：必须判绿，成功句报复跑了 2 条开关、逐格相同 2 条）。
#   弄坏开关 CODE_TOOLING_BREAK=replay-rerun-threads-split：一次一条的重跑仍拿开关趟那一段分过的 2 个线程，
#   break-switch-replay-rerun-threads-full-green 判红（期望退出 0，实测 1：「scripts/gate/again.sh AGAIN_BREAK=late：不符」，失败行「✗ 自检：这一趟核到的线程数不对」没被点名）。
# 每趟的进程组（.claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」；写法在 SWITCH_REPLAY_PARALLELISM_LIBRARY 的 TripLauncher，selftest-cells-alone 格同一份）：
#   基准趟、开关趟与重跑的每一趟放进它自己的会话与进程组；到点对整组先 TERM，过 TRIP_STOP_GRACE_SECONDS（3）秒组里还有活的再 KILL，打一行「到点停整组」；
#   直接起的那一个退出之后组里还有进程（等 TRIP_EXIT_SETTLE_SECONDS 秒还在）：逐个记进这一趟的失败信息，再照样停整组；门禁收到 TERM、INT、HUP 时在跑的各趟照样停整组。
#   停完按进程组号在 /proc 里核组里一个不剩，那一趟才返回、名额才还；还剩停不掉的，记进失败信息，这一趟的内存额度不还（位置照还）。
#   留下进程的趟：开关趟对上的也记「留下进程」、判红，别的结局在说明后面接上；基准趟照没过办（开关一条不起）；都先单独重跑一次再定，且不落盘（取回的都是收干净了的趟）。
#   到点没跑完的趟量到的峰值是被停之前的，不换进每趟内存的估计。
#   样本 break-switch-replay-trip-group-stopped-green（hangs.sh 一条开关，每条开关限时 3 秒：那条开关的头一趟派一个不理 TERM 的孙进程、自己睡 10 秒，到点；
#   重跑那一趟起跑时看那个孙进程还活着没有，活着就多打一行没点名的 ✗：必须判绿，且输出里有「到点停整组：scripts/gate/hangs.sh HANGS_BREAK=prefix」
#   与「发了 KILL；组里一个不剩之后才还这一趟的名额」。重跑那一趟一定在头一趟返回、名额还了之后才起，看到的就是还名额那一刻组里收没收干净）。
#   弄坏开关 CODE_TOOLING_BREAK=replay-trip-leader-only-stopped：每趟不开新会话，到点只 kill 直接起的那一个、不收组、不核，名额当场还：
#   重跑那一趟起跑时孙进程还活着，break-switch-replay-trip-group-stopped-green 判红（期望退出 0，实测 1：「scripts/gate/hangs.sh HANGS_BREAK=prefix：不符」，
#   失败行「✗ 自检：前一趟派的孙进程到这一趟起跑时还活着」没被点名）。
# 只在临时 prune-server 上跑的自证（要临时服务的表 .claude/hooks/selftest-temporary-service.tsv 里登记成 required：裸跑退 2、不写真服务，research-script-selftests 格同一个认法）：
#   基准趟与开关趟都照「$SELFTEST_TEMPORARY_SERVICE_PREFIX」那一句包一层起（--server-binary 给这一格开跑前取的那一份，code_tooling_prune_server_binary；
#   包装起的临时服务一趟一份，各趟互不串），开关照样设在环境里传进去；表里有这种的才取二进制，取不到的那几份判红、开关一条都不起。
#   样本 break-switch-replay-temporary-service-green：样本根自己的要临时服务的表里登记成 required 的一份样本自证（源码里不写那几个字，只靠表认），不在样本的假包装（scripts/prune-server/prune_server_client_selftest.py，
#   只设一个变量再起那条命令）底下就退 2；登记一条开关，必须判绿（二进制取 PRUNE_SERVER_BINARY，样本档在包装底下起时由包装设）。
#   弄坏开关 CODE_TOOLING_BREAK=replay-temporary-service-unwrapped：不包，那一份样本判红（期望退出 0，实测 1：「不设开关的 --selftest 就没过（退 2）」）。
# 两个限时分开读（拆分提案第一百二十二节 ③）：
#   第二列「每条开关最多几秒」只管开关趟（设上一条开关起的那一趟 --selftest）；第三列「基准趟最多几秒」只管基准趟（不设开关起的那一趟），不写就照第二列。
#   自证自带开关轮的那几份（crash-point-replay.sh、crash-amplification.sh 这类，外面设开关被守卫当场拒）：开关趟一起来就退 2，基准趟却要把自证与它自己的
#   整轮开关都跑完，比单条开关趟慢得多，第三列照「单跑一条开关的秒数 ×（1 + 开关条数 ÷ 开关轮并行度，向上取整）」写。
# 同一张文件头开关表的两种读法：这一格读每条开关下一行起「」里的失败行原文（点名的项）；自证自带开关轮的脚本，它自己的开关轮读括号外的格名、不读「」里的原文。
#   两种读法各管各的：这一格对自带开关轮的只核守卫（退 2 并点名只在自证里认），格名对不对归它自己的开关轮。
# 没接复跑的（文件头开关表写全、登记成自证、又不在复跑登记表里的）：成功句逐个列名，往 $GATE_NOT_RUN_FILE 报一行；接进来的办法是往登记表加一行，
#   先拿原型把那份脚本的每一条开关跑一遍，点名的格与实际红的格逐格相同再加。登记表只往里加、不往外删，除非那份脚本不再读开关。
# 拆分族（写法与认法在 BREAK_SWITCH_HEADER_LIBRARY 的 SplitFamilies）：登记的路径是入口，入口拆出的部分文件（同目录的 <名>_<部分>.py、<名>-<部分>.sh、<名>-<部分>.py，
#   门禁阶段的 lib/ 下 <阶段名>-<部分>.sh，且入口的代码里提到它）也算这一份：「代码里读不读弄坏开关」按入口加部分文件的代码算，读开关的代码在部分文件里、入口自己不读也算读；
#   文件头开关表（switch_entries）、写没写全（header_gaps）、汇总行声明（summary_declaration）也按整族读：开关表可以写在入口或部分文件的文件头，每条开关与它点名的项取它写在的那一份，
#   红句里的文件:行是那一份的；整族没有一条开关表的，点名入口。选格变量（<名字>_SELFTEST_CELLS）只在入口那一份的文件头里找，找不到就整份起（判得更严、不更松）。
#   收尾数「没接复跑」的文件跳过部分文件，开关归它的入口那一份算；登记表点名的文件各自是入口，不当别人的部分文件。
#   样本 break-switch-replay-split-family-green（入口 scripts/gate/divided.py 自己不读开关，import 的 divided_core.py 读它，开关表在 divided_core.py 的文件头；登记表一行点名入口：必须判绿，复跑了 1 条开关）、
#   break-switch-replay-split-family-item-green（同一份脚本，样本根的 .gate-items 点部分文件里开关表的那一条开关；可点的项（code-tooling.sh 的 code_tooling_items_of）按整族列）、
#   break-switch-replay-split-family-unrelated-red（入口 scripts/gate/lonely.py 不读开关，同目录 lonely_extra.py 读开关而 lonely.py 的代码里没提它：不是它的部分文件，判红「代码里不读弄坏开关了」）。
#   弄坏开关 CODE_TOOLING_BREAK=replay-split-family-unread：只读登记路径那一份的代码与文件头，break-switch-replay-split-family-green 判红（期望退出 0，实测 1：「代码里不读弄坏开关了，那一行删掉」）；
#   =replay-split-family-table-entry-only：读开关按整族、开关表与汇总行声明只读入口那一份的文件头，break-switch-replay-split-family-green 判红（期望退出 0，实测 1：「文件头没有一条…」）；
#   =replay-split-family-by-name：只按命名认部分文件、不核入口的代码提没提到，break-switch-replay-split-family-unrelated-red 判绿（期望退出 1，实测 1，「代码里不读弄坏开关了」那条 want 找不到：lonely_extra.py 被当成部分文件，
#   lonely.py 照读开关往下复跑、设上开关自证照样退 0）。
# 管不到的：失败行不带 ✗ 的自证（点名的项只照带 ✗ 的行）；汇总行的认法是按次数推的，一格真被一半以上的开关弄红时它也当汇总行、不要求点名；
#   自证自带开关轮的那一份开关轮判得对不对（归它自己的自证）。
# 进度（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔 5 分钟；写法在 SWITCH_REPLAY_PROGRESS_LIBRARY）：开跑打一行「break-switch-replay 开跑：」
#   （份数、开关条数、并行度）；每条开关判完打一行「break-switch-replay 第 k / N 条<对上、对上（自证自带开关轮）或没对上（结局）>：<路径> <开关>，用了 S 秒」
#   （S 是这条开关趟加重跑的秒数）；每份的开关轮判完（或基准趟没过、开关一条没起）打一行「break-switch-replay 跑完一份：<路径> 开关 N 条、对上 M 条、用了 S 秒」
#   （S 从这份第一趟起跑算起）；另起一条线程每过 300 秒打一行总进度（判完几条、在跑几趟）、跑得最久的三趟、一行「按份：」（跑完几份、还在跑几份），
#   再给起跑了还没跑完的每份打一行「break-switch-replay 还在跑：<路径> 开关 N 条、已跑完 T 趟、判完 K 条、用了 S 秒」，末了一行内存（后代进程数、Pss 合计、cgroup 记账、到此刻的峰值），
#   每 5 秒采一次内存；收尾打一行「break-switch-replay 逐条进度 R 行、派出去 N 条；」加内存峰值、峰值那一刻在跑的按份计数与单个最大的进程。
#   样本根（judged_root_is_sample）里的 .break-switch-replay-progress-seconds（一个正整数秒数）顶替 300 秒，样本不必等满 5 分钟；写的不是正整数判红、一条都不跑。
#   真仓上一趟 58 份、1386 条开关；并行 8 时第二台 8689 秒跑完 854 趟（任务 3542 的日志，被取消前最后一次报的进度），照这个速度整格约四小时（推的）。
# 机读进度：开跑时与每跑完一趟时，另打一行行首顶格的「TASK_SERVER_PROGRESS done=<跑完几趟> total=<要跑几趟>」（读日志的人与脚本认它，统一任务服务不读；不带 ✗；别的进度行照旧）。
#   趟是基准趟、开关趟与要重跑的那一趟，取回的落盘趟当场算跑完；total 开跑时是这一趟要起的份数（基准趟）加开关趟数，要重跑的那一趟派出去之前加一，
#   基准趟都跑完时改成「跑完的加还要跑的开关趟」（基准趟没过、开关一条没起的那几份扣掉）。按趟报，不按判完的条报：一份的开关趟齐了才集中判，
#   按条报时 done 一阵一阵涨、看不出跑到哪了（ReplayProgress 文档串）。
#   样本 break-switch-replay-progress-green（一份、两条开关，三趟）里有 done=0、1、2、3 四行，total 都是 3。
#   弄坏开关 CODE_TOOLING_BREAK=replay-machine-progress-silent：不打这一行，break-switch-replay-progress-green 里四条 TASK_SERVER_PROGRESS 的 want 找不到。
# 范围（.claude/rules/implementation-workflow.md「复用门禁的判定：按每条判定的单元键判，不按「改了哪个目录」或「哪棵树绿过」判」）：这一趟给了 --check 或 --item（样本根的 .gate-cells 与 .gate-items；
#   范围声明种类 items）是指定范围，什么都没给（提交时的整轮门禁；种类 whole）是不指定范围；样本根的 .break-switch-replay-whole-scope 把 .gate-cells 点名的这一趟改按不指定范围判。
#   指定范围：文件里写着「<名字>_SELFTEST_CELLS=<格名…」的自证（有选格手段：那个环境变量设成逗号隔开的格名，自证只跑点名的格，单跑一格与它在全部格里跑判定相同）
#   每条开关趟只起那一条点名的格：点名的项（「…」里的失败行句首）里「自证：<格名>：」或「✗ 自证 <格名>：」里的那一段，一条开关写几处取几处；基准趟起这份每条开关点名的格的并集（开关趟的失败行减的是同一批格的）。
#   判法一字不变：点名的每一项都要在失败行里照出来（点名的格没红判红），没点名的 ✗ 行判「失败行没被点名」；只是没点名的格不起，红不到它们。
#   一条开关读不出格名的，它那一趟整份起，这份的基准趟也整份起；没有选格手段的自证整份起（scripts/mutation/broad-mutation.py 的 BROAD_MUTATION_SELFTEST_CELLS 选的是代码里的格组、
#   名字写的是「<组名」，失败行里的格名对不到组，不算有选格手段）。不指定范围：每条开关趟与基准趟都起整份自证，逐格相同。
#   两种范围的判定不共用单元键：只起点名的格的那几趟，键里另带 line:cell-scope=<选格变量名>:<格名的 sha256>，不指定范围的键里没有这一行（只起点名的格看不到点名之外的格红了什么，
#   判的不是同一件事，指定范围判出的绿不许被不指定范围那一趟复用）；整份起的趟两种范围判的是同一件事，共用同一把键。成功句末尾写「范围：指定范围（…）」或「范围：不指定范围（…）」，开跑前另打一行。
#   弄坏开关 CODE_TOOLING_BREAK=replay-scope-ignored：指定范围也整份起、键里不带选格那一行，break-switch-replay-scoped-cells-green 判红；
#   =replay-whole-scoped：不指定范围也只起点名的格，break-switch-replay-whole-scope-red 判绿；=replay-scoped-key-shared：只起点名的格的那几趟不带选格那一行，
#   break-switch-replay-scoped-key-separate-green 第二趟复用第一趟的。
# 并行度：并行条数取登记的每台默认重活线程数（.claude/heavy-thread-default；样本根的 .break-switch-replay-jobs 写 threads 顶替），不按这一趟的内存上限收
#   （准许按内存上限定并发的入口登记在 .claude/memory-bounded-parallelism.tsv，这一格不在里面）。内存上限由 task-server 照这条命令的记录给，这一格不自报要多少内存：
#   照上限收并行时，上限又是照以往一路跑的峰值给的，只够一路，并行永远收在一路、也永远撞不上上限、服务也就不放大。撞了上限服务照乘 1.5 放大、重排这一条，
#   第二趟只补没落盘的开关趟（下面「开关趟落盘」；样本 break-switch-replay-interrupted-resume）。
#   内存上限只写进开跑那一行：这一趟所在的 cgroup 一路到根的 memory.max 里数字的最小值（读法 cgroup_memory_limit；样本根的 memory-max 顶替），读不到或是 max 照写原因。
#   外层路数 × 里层线程数 ≤ 登记的线程数，基准趟与开关趟两段各算各的（基准趟都跑完、要重跑的基准趟一次一条重跑完，开关趟才派出去，两段不同时在跑）：
#     开关趟那一段：外层路数的上限 = min(并行条数, 要起的趟数（每份一趟基准趟加全部开关趟）)，里层线程数 = ⌊线程数 ÷ 外层路数的上限⌋（至少 1）；
#     基准趟那一段：同时起的只有份数条基准趟，外层路数 = min(开关趟那一段外层路数的上限, 份数)，里层线程数 = ⌊线程数 ÷ 它⌋（至少 1；一份就拿满登记的线程数）；
#     要重跑的基准趟一次一份、只它一趟在跑：分过而到点停整组的拿满登记的线程数，别的照拿基准趟那一段的数（这一格文件头「重跑」）；
#     要重跑的开关趟（头一趟是这一趟自己起的）在全部开关趟收完之后一次一条、只它一条在跑：外层路数 1，里层线程数 = 线程数（这一格文件头「重跑」）；
#     取回的、判得不是对的开关趟不在此列，按开关趟那一段的外层路数并行重起（这一格文件头「开关趟落盘」）。
#   每一趟起的自证都带环境变量 SINGLEFS_THREAD_CAP=<它那一段的里层线程数>（自证里的并行度经登记位取，登记位认它）；只起点名的格的那几趟，格数的单位数也不超过它那一段的里层线程数。
#   同时在跑的趟数不超过开关趟那一段外层路数的上限（Admission 不带内存份额，只数在跑的趟；TripMeter 量的各趟峰值只进内存那一行进度）。
#   开跑那一行写「并行 N 条（登记的每台默认重活线程数 T 条；内存上限 …）」与两段各自的「基准趟：外层 N 条 × 里层 M = P（不超过每台默认重活线程数 T）」
#   「开关趟：外层 N 条 × 里层 M = P（不超过每台默认重活线程数 T）」，再写一句分过而到点的基准趟拿满 T 个线程、同一个限时一次一份重跑，
#   一句要重跑的开关趟一次一条、拿满 T 个线程。
#   自证里不经登记位取并行度的（登记表 .claude/heavy-thread-exempt 点名的、自己读别的线程变量的）收不住这一层。
#   样本：break-switch-replay-scoped-cells-green（.gate-cells 点名这一格是指定范围：一份有选格手段的脚本四格，两条开关各弄坏点名的一格与没点名的一格，只起点名的格所以绿，成功句写「范围：指定范围」）；
#     break-switch-replay-scoped-cells-space-form-green（同一种脚本，失败行与点名的项写成「✗ 自证 <格名>：」，照样读出格名、只起点名的格）；
#     break-switch-replay-whole-scope-red（同一份，样本根放 .break-switch-replay-whole-scope 按不指定范围判：没点名的两格红了，各点名「失败行没被点名」）；
#     break-switch-replay-scoped-named-quiet-red（指定范围：脚本退 1，只在行中间转述一句不是点名的 ✗ 行，点名的格没红判红）；
#     break-switch-replay-scoped-key-separate-green（样本根放 .break-switch-replay-second-pass 与 .break-switch-replay-second-pass-whole：同一份判定库上第一趟指定范围、第二趟不指定范围，第二趟三条判定全部现判）；
#     break-switch-replay-jobs-registered-green（样本根的 .break-switch-replay-jobs 写 memory-max=1536 MiB、threads=8；一份脚本 32 条开关、各红一格，开关趟起跑时在 running/ 下登一个文件、
#     等同时有 8 趟登着（哪一趟看到过就记下，之后的照放），等满 10 秒还没等到就记下、此后每一趟多打一行带自己开关值的没点名的 ✗：必须判绿，开跑那一行写并行 8 条，
#     收尾逐条进度 32 行、派出去 32 条）、-jobs-registered-red（同一组并行度、同样 32 条开关，其中一条点名的项写的是另一格：只那一条判不符，1 处不对（复跑了 32 条、1 份））、
#     -jobs-threads-capped-green（上限 100 GiB、线程 2、一份两条开关：并行 2 条，基准趟外层 1 条 × 里层 2 = 2，开关趟外层 2 条 × 里层 1 = 2）、
#     -jobs-inner-threads-green（上限 100 GiB、线程 8、一份两条开关三趟：基准趟外层 1 条 × 里层 8 = 8，开关趟外层 3 条 × 里层 2 = 6；样本自证在基准趟里核 SINGLEFS_THREAD_CAP 是 8、
#     在开关趟里核是 2，不是就打一行没点名的 ✗ 并退 1：基准趟没过、或开关趟红在没点名的一格，判红）、
#     -jobs-baseline-threads-by-count-green（上限 100 GiB、线程 8、两份各一条开关四趟：基准趟外层 2 条 × 里层 4 = 8，开关趟外层 4 条 × 里层 2 = 8；
#     两份样本自证在基准趟里核 SINGLEFS_THREAD_CAP 是 4、在开关趟里核是 2，不是就同上判红）。
#   弄坏开关 CODE_TOOLING_BREAK=replay-jobs-by-memory：并行条数照内存上限算（parallel_library.jobs_for：⌊上限的 SAFE_SHARE_PERCENT 份额 ÷ 每趟内存⌋、不超过线程数），
#   Admission 带那一份份额，-registered-green、-registered-red、-threads-capped 三份判红；
#   =replay-inner-threads-uncapped：不把里层线程数收到线程数 ÷ 外层路数的上限、不设 SINGLEFS_THREAD_CAP，-threads-capped、-inner-threads、-baseline-threads-by-count
#   与 break-switch-replay-baseline-split-timeout-rerun-green（slow.sh 读不到这个变量）判红；
#   =replay-baseline-threads-split：基准趟那一段也拿开关趟那一段分过的里层线程数，-threads-capped（开跑那一行基准趟写的是外层 1 条 × 里层 1）、
#   -inner-threads（基准趟核到的是 2、不是 8）、-baseline-threads-by-count（基准趟核到的是 2、不是 4）、break-switch-replay-baseline-split-timeout-rerun-green
#   （重跑那一行写的是头一趟里层 2 个线程）与 break-switch-replay-baseline-full-timeout-red（头一趟拿到 2、不是登记的 4，拿满重跑就过了）判错。
# 登记表空或不在：退 77。
# 样本：break-switch-replay-red（登记表里四份：一份有一条开关点名的项没在失败行里，一份有一条开关设上之后自证照样退 0，一份点名的项里没有「…」原文；
#   另一行指到不在的文件）；break-switch-replay-unnamed-red（一份开关弄红两格、点名一格；一份红的一格前面是单独的 \r 接着的进度行，没点名：两处多红）；
#   break-switch-replay-green（一份全对、一份自证自带开关轮、一份基准趟本来就打一行 ✗ 拒绝行且三条开关各红一格加一句汇总行、一份只有一条开关而基准趟也打那一行，
#   另有一份文件头写全却没登记，报「复跑了 6 条开关……没接复跑的 1 份」）；
#   break-switch-replay-undecodable-red（一份两条开关、一条下自证吐坏字节，一份全对，一份退 0，一份吐坏字节而点名的项也不在失败行里：
#   坏字节、退 0、坏字节加少红各记，报「4 处不对（复跑了 5 条、4 份）」）。
# 开跑前预检登记的文件头开关表写没写全：与 break-switch-headers 格同一份取法（BREAK_SWITCH_HEADER_LIBRARY 的 header_gaps、gap_sentence），缺点名的项（点名行缩进不够读不出来的也算）、
#   缺弄坏哪一处、一条开关都读不出的，逐条点名「复跑登记表:行号 路径:行 开关=值 缺…」，那一份不起；样本 break-switch-header-gap-red（与 break-switch-headers 格一起跑，那一格的说明里）。
# 判红的明细：日志里每条「不符」只列前三项点名的项（各截 60 字）、前三行没被点名的失败行（各截 80 字），逐条对点名不够用。判红时把每条不符的开关完整的点名的项与这一趟全部多出来的失败行
#   落到一份文件里（真仓上在工作根下新建 break-switch-replay-mismatches.<随机串>/mismatches.txt，工作根取 .claude/hooks/lib_work_root.py 不带参数打的那个，两台读得到；
#   取不到或建不了时放本机临时目录并在回读那一行写明；样本根放本机临时目录、回读核过就删），路径与那一条开关的起始行接在「不符」行后面（「；完整的失败行与点名的项在 <文件>:<行>」），
#   日志里照旧只列前几项、前几行。写完回读一遍：点名的项逐项、多出来的失败行逐行都得整段在文件里，缺了判红；核过打「不符明细回读核过：N 条开关、点名的项 A 项、失败行 B 行（其中没被点名的 C 行）都在文件里」。
#   样本 break-switch-replay-mismatch-detail-red（一份开关红五格：文件头点名一格、另点名了一格没红，多红的四行每行都超过八十个字）。
#   弄坏开关 CODE_TOOLING_BREAK=replay-mismatch-detail-truncated：明细文件照日志的样子截（失败行只写前三行、每行前八十个字，点名的项只写前三项、每项前六十个字），回读缺那几行，
#   break-switch-replay-mismatch-detail-red 里「不符明细回读核过」那条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=replay-header-gaps-unjudged：不做这一步预检（文件头缺点名的项也照样复跑），break-switch-header-gap-red 里复跑登记表那条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=replay-items-unchecked：不比点名的项，break-switch-replay-red、break-switch-replay-undecodable-red、break-switch-replay-mismatch-detail-red 里「点名的项没在失败行里」那条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=replay-output-strict-decode：按文本收输出、不容坏字节，break-switch-replay-undecodable-red 整格抛 UnicodeDecodeError，
#   「坏字节，stdout 里有不合法的 UTF-8」与「4 处不对（复跑了 5 条、4 份）」那几条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=replay-unnamed-rows-unchecked：不查多红（失败行比点名的多不判），break-switch-replay-unnamed-red 判绿（期望退出 1，实测 0），
#   break-switch-replay-mismatch-detail-red 里「失败行没被点名」那条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=replay-baseline-unsubtracted：不起基准趟、不减自证本来就有的失败行，break-switch-replay-green 判红（期望退出 0，实测 1：只有一条开关的那一份，基准趟那一行没被点名）。
# 弄坏开关 CODE_TOOLING_BREAK=replay-summary-rows-required：汇总行也要点名，break-switch-replay-green 判红（期望退出 0，实测 1：「✗ 自检：… 格不对」没被点名）。
# 弄坏开关 CODE_TOOLING_BREAK=replay-carriage-return-kept：单独的 \r 留在行里，break-switch-replay-unnamed-red 里「scripts/gate/carriage.py CARRIAGE_BREAK=progress」那条 want 找不到
#   （回车后面那一格接在进度行后面、不以 ✗ 起头，不要求点名；只剩一处不对）。
# 弄坏开关 CODE_TOOLING_BREAK=replay-bad-bytes-first：有坏字节就只记坏字节、不再比点名，break-switch-replay-undecodable-red 里
#   「scripts/gate/garbled-mismatch.py GARBLED_MISMATCH_BREAK=bytes：不符」与「4 处不对」两条 want 找不到。
# 样本 break-switch-replay-slow-baseline-green：一份自证自带开关轮的脚本（外面设开关当场退 2 并点名；不设开关时自证先睡 3 秒、算它自己那一轮），
#   登记成每条开关 2 秒、基准趟 8 秒，必须判绿（开关趟只要一瞬，基准趟要 3 秒，同用 2 秒就超时）。
# 弄坏开关 CODE_TOOLING_BREAK=replay-baseline-shares-switch-limit：基准趟也照第二列的限时跑，break-switch-replay-slow-baseline-green 判红
#   （期望退出 0，实测 1：「不设开关的 --selftest 就没过（超时：基准趟限时 2 秒）」）。
# 样本 break-switch-replay-progress-green：样本根里 .break-switch-replay-progress-seconds 写 1；一份脚本两条开关，自证每一趟（基准趟与开关趟）都先睡 3 秒，
#   两条开关各红一格、点名都对，必须判绿，且输出里有「break-switch-replay 还在跑：scripts/gate/sleepy.sh」（一趟 3 秒、间隔 1 秒，基准趟里就至少报两次）
#   与「break-switch-replay 跑完一份：scripts/gate/sleepy.sh 开关 2 条、对上 2 条」，另有逐条的「第 1 / 2 条对上」「第 2 / 2 条对上」、内存一行与「逐条进度 2 行、派出去 2 条」。
# 弄坏开关 CODE_TOOLING_BREAK=replay-progress-silent：不打进度、不起报进度的线程，break-switch-replay-progress-green 里那几条进度的 want 找不到
#   （与 stage-break-switch-replay 格同一条开关，各红各的进度样本）。
# 按条复用（.claude/rules/implementation-workflow.md「复用重型门禁的判定：按每条判定的单元键判」；共用段在 UNIT_VERDICT_ROUNDS_LIBRARY）：
#   一条判定 = 登记表里的一份脚本加它的一条开关（名字「<路径> <开关名>=<值>」），外加那一份不设开关的基准趟（名字「<路径> 基准趟」）；
#   种类 code-tooling.sh:break-switch-replay（prune-server 那一份库，unit-verdicts 族 singlefs-unit-verdicts/code-tooling.sh:break-switch-replay，库根不进键）。
#   键（scripts/gate/admission.py verdict-plan 算，不带路径、行号与限时）由这几样组成：
#   ① 那份脚本自证的单元闭包（入口就是那份脚本：python 的程序入口、shell 的 script-body，连同 import、source 到的模块与函数、读的数据文件、解释器版本，
#      与 research-script-selftests 格算单元键的是同一路，admission.py 文件头「单元键」；python 与 shell 的都另带 program-argument:--selftest，只收 --selftest 真走到的那一支，
#      lib_function_units.py 文件头「程序入口带参数」）；
#   ② 判法：这一道所在仓里 break_switch_replay_judge_code 的闭包（令牌 BREAK_SWITCH_REPLAY_JUDGE_TOKEN：开关表的读法与 BREAK_SWITCH_REPLAY_JUDGE_LIBRARY）；
#   ③ line:command=<起它的命令样子>；④ line:switch=<开关名>=<值>（基准趟写「基准趟」）；
#   ⑤ 开关那几条另加 line:named-sha256=<那一条点名的项原文的 sha256>（文件头是注释，不进单元指纹，另进这一行）；
#   ⑥ 只在临时 prune-server 上跑的另加 file:scripts/prune-server/prune_server_client_selftest.py 与 line:temporary-service=wrapped；
#   ⑦ 文件头声明了汇总行的（下面「汇总行的认法」），开关那几条另加 line:summary-sha256=<声明的原文的 sha256>（声明决定哪几行多红不要求点名，文件头是注释，另进这一行）。
#   拒算的照共用段开跑前判红、一条都不起（UNIT_VERDICT_ROUNDS_LIBRARY 的 ItemVerdicts.stop_on_refusals；样本 break-switch-replay-refused-red）。
#   起哪几份：点到的条里有一条要现判（键对不上、库里记着红、复用哨兵强制现判）的整份照起——基准趟与全部开关趟，不是只起那几条：
#   汇总行要整份的开关趟，只起几条判出来的与整格里的不同；别的条这一趟陪跑，判完照记库、判红照报（成功句与按条复用那一行报陪跑几条）。点到的条都复用的整份跳过。
#   只点了单条的那几份（下面「逐项点名」）不在此列：整份就是基准趟加点到的那几条。
#   键相同、库里记着绿的复用；每趟开跑前 verdict-plan 抽复用哨兵、强制现判一条（种子打进日志；被判的根下有 .unit-verdict-sample/ 的样本不抽），失灵交 plan-failure 判红。
#   判完就记：一份的开关趟齐了当场判、当场记，要重跑的那几条判完各记一次。基准趟判绿（没超时、退 0、没有坏字节）、开关对上（对、守卫，没有坏字节）记绿，
#   别的照同一把键记红、撤掉原有的绿（verdict-record --red-names）；基准趟没过、开关一条没起的那一份只记基准趟；收尾把这一趟判完的整份再交一次（哨兵在那一趟比）。
#   开关趟落盘（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」：被打断后重跑同一条命令就是续跑，只补没判的）：一条开关的判定要这一份全部开关趟的汇总行，
#   开关趟齐之前判不了，所以落盘的单位是一趟：基准趟与开关趟每跑完一趟，当场把这一趟的结局（超时、退出码、坏字节，与判法读的那几行：带 ✗ 的行、点到开关名的自证行）照这一条的键
#   写进这一格种类底下的 trips/<键>.json（TripJournal，经 admission.py library-files，同一份判定库、不另开）。要起的那几份里键相同、落了盘的趟取回、不再起，只补没落盘的；
#   取回的趟与现跑的趟一起算汇总行、一起判，判法读到的与一趟跑完的相同；现跑的头一趟判得不是对的，照旧先单独重跑一次再定（重跑的结局盖掉落盘的那一份）。
#   取回的开关趟判得不是对的（落盘的趟只保证收干净了，不保证是正常跑完的：被内存上限杀掉的自证也落盘）不当头一趟，不进一次一条的重跑：
#   一份的头一趟（现跑的、取回的）收齐之后，那几趟照没落盘办，与别的开关趟一样按开关趟那一段的外层路数并行重起一趟（里层线程数也是那一段的），
#   打一行「取回的落盘趟里 N 趟判得不是对的（取回时：退 C：K 趟…）：<路径> 这几趟不当头一趟，与别的开关趟一样按外层 P 条 × 里层 M 并行重起…」；
#   重起的趟收齐之后才判这一份，重起的趟当头一趟：判得不是对的再进上面「重跑」那一段，认不出汇总行的（整份性质，重起改不了）不重起。重起的趟照常落盘、盖掉落盘的那一份，机读进度的 total 每趟加一。
#   复用哨兵抽中强制现判的那一条不取回，照起。取回时 prune-server 不可用这一格退 75；别的取不回判红，要起的照全部起。「起哪几份」不变，取回了的不算陪跑。
#   样本 break-switch-replay-resumed-bad-trips-parallel-green（样本根 .break-switch-replay-jobs 给上限 100 GiB、线程 4，.unit-verdict-sample/interrupted-resume/stop-after 写 3；lane.sh 四条开关各红一格：
#   被打断的那一趟里（CODE_TOOLING_SAMPLE_STOP_ARMED 设着）每条开关趟像被内存上限杀掉、退 137 不打失败行，基准趟照常；判完基准趟与前两条开关趟就停，落盘的前两条是退 137 的；
#   第二趟取回 3 趟、现起 2 趟，取回的两条判得不是对的，并行重起，每一趟起跑后等 6 秒、看见别的趟同时在跑就记下、都没看见就多打一行没点名的 ✗：
#   必须判绿，输出里有「取回的落盘趟里 2 趟判得不是对的」「开关趟：现跑 4 趟、取回 3 趟」，第三趟全部复用）。
#   弄坏开关 CODE_TOOLING_BREAK=replay-resumed-bad-trips-serial：取回的判得不是对的不挑出来，照头一趟判、进一次一条的重跑，
#   break-switch-replay-resumed-bad-trips-parallel-green 判红（期望退出 0，实测 1：「scripts/gate/lane.sh LANE_BREAK=prefix：不符」，失败行「✗ 自检：这一趟起跑后 6 秒内没有别的趟同时在跑」没被点名）。
#   样本 break-switch-replay-interrupted-resume（两份脚本：alpha 一条守卫、两条各红一格加一句汇总行，beta 一条；stop-after 是 3：判完两份基准趟与守卫那一条就停，
#   两份样本脚本同一趟起第二次就判错；第二趟取回 3 趟、现跑 3 趟，判绿；第三趟全部复用）；break-switch-replay-forced-trip-rerun-green（样本根的 .break-switch-replay-second-pass 写种子：
#   一份脚本四条开关连基准趟五条判定，同一份判定库上跑两趟，第二趟复用哨兵照那个种子强制现判一条开关，取回 4 趟、现跑 1 趟）。
#   成功句报「按条复用：现判 N、复用 M、拒算 R、陪跑 C」与「开关趟：现跑 A 趟、取回 B 趟」；全部复用时报「全部复用，这一格一条都没起」、退 0。算不出（verdict-plan 退非 0、没打 PLAN）照全部现判跑完、判红；
#   prune-server 不可用（退 75）一份都不起、这一格退 75。
#   汇总行的认法：汇总行是自证在各格失败行之外多打的总结句（例「✗ … N 格里 M 格不对」），一条开关趟里多出来的这一行不要求点名。两种认法：
#     声明了的：被测脚本文件头写一行 break-summary-rows：后面接一个或几个「✗ …原文…」（… 是中间省掉的一段，各段按次序出现在同一行里，读法同点名的项；
#       与行尾带 # gate-lint:summary 的那一行同一件事的另一头：那一头给 gate-lint 豁免，这一头给复跑认汇总行）。原文要以 ✗ 起头、去掉 … 之后至少 8 个字，
#       写得不对的开跑前逐条点名「复跑登记表:行号 路径 第 N 行 break-summary-rows …」，那一份不起。照声明认：各条红了的开关趟多出来的 ✗ 行里照得出声明的就是汇总行，
#       一条开关趟里出现就算，整份起与只点单条的认法相同，不靠统计；多出来的行里没点名、声明也认不出的判不符（照常记库，说明末尾点名声明）。
#       声明进每条开关的键（上面 ⑦），改了声明那几条现判。
#     没声明的：一半以上开关下都多红的同一行（至少两条开关趟），要整份的开关趟才认得出，只点单条的认不出（下面「逐项点名」）。
#     样本 break-switch-replay-item-single-summary-declared-green（tally.py 同 -single-summary-unresolved-red，文件头声明了汇总行，只点 TALLY_BREAK=prefix：汇总行认得出，判绿）、
#       break-switch-replay-item-single-summary-declared-wrong-red（声明的原文与汇总行对不上：照声明仍认不出，判不符并点名声明，不是「认不出汇总行」）、
#       break-switch-replay-summary-declared-in-key（两份脚本五条，alpha 声明了汇总行；change.sh 只改那一行声明的写法（照样照得出）：alpha 的两条开关现判，基准趟与 beta 复用）。
#     弄坏开关 CODE_TOOLING_BREAK=replay-summary-declaration-ignored：不读声明，-single-summary-declared-green 判成「认不出汇总行」；
#       =replay-summary-declaration-unkeyed：声明不进键，-summary-declared-in-key 里改声明那一趟 alpha 的两条照复用、三问第三问红。
#   逐项点名（这一道「逐项点名」一节）：项是 <脚本路径>（整份）或 <脚本路径>:<开关名>=<值>（单条），给几次取并集；同一份既点了整份又点了单条的按整份。
#     只点了单条的那一份：基准趟与点到的那几条是这一趟的全部判定（请求、计划、开关趟、按条复用那一行与「复跑了 N 条、M 份」都只数它们），没点的开关不起、不算判过；
#     基准趟键相同、库里记着绿的照复用（这一份有开关趟要起时，基准趟照起或取回落盘的那一趟，失败行要减它的，算陪跑），每条开关各自按键复用、各自记库，键与整份起时相同。
#     没声明汇总行的脚本（下面「汇总行的认法」）：汇总行（一半以上开关下都多红的同一行）要整份的开关趟才认得出，只点单条的不认：判成不符、而点名的项都照出来、
#     红的只是没被点名的 ✗ 行的，那几行在整份里可能是汇总行，判不定，结局「认不出汇总行」，判红、不记库（库里原有的判定不动；整份点 --item <路径> 判，或在文件头声明汇总行），
#     这样单条判出的绿在整份里也是绿，键可以共用。
#     成功句与汇总行末尾接「只点了单条的 K 份：<路径> 点了 k / n 条…（没点的 M 条这一趟没起、不算判过）」。
#   样本：break-switch-replay-reuse-three-questions（两份脚本五条；复用三问：change.sh 改 alpha.py 的 expected_cells，affected 是 alpha 的三条；四趟都绿、三问都过）；
#     break-switch-replay-named-in-key（同样两份；change.sh 只改 alpha.py 头注里一条开关点名的项的写法（照样照得出）：那一条现判，同一份别的两条陪跑、仍算复用）；
#     break-switch-replay-red-over-green（.unit-verdict-sample/prior/ 照判绿记四条；red-over-green 下一律现判，样本根的 .red-switch 让 flaky.py 那条开关红在没点名的一格，
#     判红记红、撤绿，再算一次计划：复用 3 条、现判 1 条、原因「上次判红」）；break-switch-replay-item-co-run-green（tally.py 三条开关各红一格加一句汇总行，点它整份又点它一条开关：
#     按整份，基准趟与三条开关四条都是这一趟的判定；复用三问：change.sh 只改 prefix 那一条点名的项的写法（照样照得出），unit 那一趟那一条现判，
#     基准趟与另两条复用、整份照起（落了盘的趟取回），汇总行认得出，四趟都绿、三问都过；
#     .unit-verdict-sample/prior/ 造不出「只一条的键变」：上一版的树只换单元闭包，点名的项的原文是这一版的请求里的 line:，只差头注的上一版算出的四把键与这一版的一样）、
#     break-switch-replay-item-unknown（项写错）；
#     break-switch-replay-item-single-switch-green（trio.py 三条开关，只点 TRIO_BREAK=alpha：只起基准趟与那一条，没点的 TRIO_BREAK=gamma 点名的项写的是另一格、一起就判不符；
#     判绿，按条复用那一行「2 条判定」「陪跑 0 条」，成功句「复跑了 1 条开关（1 份）」与「只点了单条的 1 份：scripts/gate/trio.py 点了 1 / 3 条」）；
#     break-switch-replay-item-single-summary-unresolved-red（tally.py 同上，只点 TALLY_BREAK=prefix：汇总行那一行没被点名，判「认不出汇总行」、退 1）；
#     break-switch-replay-item-unknown-switch（点 trio.py 文件头里没有的一条开关：退 2、一格都不跑，列出的可点的项里有 trio.py 整份与三条开关项）。
#     弄坏开关 CODE_TOOLING_BREAK=replay-single-picks-co-run：只点单条也照整份起、没点的陪跑，-single-switch-green 判红（gamma 判不符）、-single-summary-unresolved-red 判绿；
#     =replay-single-pick-summary-guessed：只点单条的照点到的几条认汇总行、照常判不符并记库，-single-summary-unresolved-red 里「认不出汇总行」那条 want 找不到。
#     break-switch-replay-record-skips-forwarded-green（一份一条开关，基准趟往它自己读的 rewritten.tsv 末尾加一行：收尾记库两条都「跑的过程中键变了」，
#     .unit-verdict-sample/record-skip-lines-shown 写 1，转一行「!」、另一条报「另有 1 条」；共用段 ItemVerdicts.report_record_skips）。
#   弄坏开关（文件头开关表）CODE_TOOLING_BREAK=replay-plan-ignored、replay-red-not-recorded、replay-key-whole-registry、replay-named-unkeyed、replay-siblings-not-co-run、
#     items-ignored、item-unknown-accepted、replay-trips-not-stored、replay-trips-not-loaded、replay-forced-trip-loaded、replay-scope-ignored、replay-cell-name-colon-only、replay-whole-scoped、
#     replay-scoped-key-shared、replay-jobs-by-memory、
#     replay-machine-progress-silent、record-skip-lines-dropped、replay-trip-leader-only-stopped、replay-reruns-batched、replay-mismatch-detail-truncated、
#     replay-rerun-threads-split、replay-single-picks-co-run、replay-single-pick-summary-guessed、replay-summary-declaration-ignored、replay-summary-declaration-unkeyed、
#     replay-resumed-bad-trips-serial、admission-run-unbounded，各红在开关表点名的样本。
#   admission.py 的子命令卡住（共用段上方的注释：整组连着 300 秒不涨 CPU 滴答、也没有进程等盘就停整组、退 124）：照「算不出」那一句全部现判跑完、判红。
#   样本 break-switch-replay-admission-stall-red（样本根的 .unit-verdict-sample/admission-stall：verdict-plan 那一趟换成只睡 20 秒的命令、限时改成 2 秒；
#   必须判红，输出里有「admission.py verdict-plan 卡住」那一行）；弄坏开关 CODE_TOOLING_BREAK=admission-run-unbounded 下睡满才退、那一行没有。
# 判成「不符」的开关，汇总前重算键（ItemVerdicts.key_changes，写法在共用段上方的注释）：与开跑前算的不同，那一条的失败行接「跑的过程中键变了（开跑前算的键、现在的键）：
#   组成清单有 N 处不同：…；出路：在冻结快照树里重判…」，键没变的失败行不变（句子接在「完整的失败行与点名的项在 <文件>:<行>」之后）；这一趟没有「不符」的开关就一个进程都不多起。
#   样本 break-switch-replay-key-changed-red（基准趟往自己读的 shifted.tsv 末尾加一行，开关那一条失败行没点名：键在跑的过程中变了）；键没变的在 break-switch-replay-mismatch-detail-red 里报「键变了 0 条」。
#   弄坏开关 CODE_TOOLING_BREAK=key-change-not-rechecked、key-change-noted-always、replay-baseline-unsubtracted（基准趟不起，表没人改），各红在开关表点名的样本。
cell_break_switch_replay() {
  local registry=".claude/gate.d/break-switch-replay-files.tsv" jobs
  [[ -f "$registry" ]] || { echo "  ! 没有 $registry，这一格无对象可判"; exit 77; }
  if ! jobs="$(heavy_thread_default)"; then
    echo "  ✗ 取不了每台默认重活线程数（原因在上一行），复跑的并行度定不了，一条都没跑"
    echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
    exit 1
  fi
  # 只在临时 prune-server 上跑的自证（这一格文件头那一段）：表里有就开跑前取一次二进制，取不到留空（python 那一侧把那几份判红、点名原因在上一行）
  local replay_path replay_needs_binary=0 replay_binary="" replay_scratch="" replay_exit replay_required=""
  temporary_service_table_load || exit 1
  while IFS=$'\t' read -r replay_path _; do
    if [[ -n "$replay_path" && "$replay_path" != \#* && -f "$replay_path" ]] && temporary_service_is_required "$replay_path"; then
      replay_needs_binary=1
      replay_required+="$replay_path"$'\n'
    fi
  done < "$registry"
  if (( replay_needs_binary )) && [[ "${CODE_TOOLING_BREAK:-}" != replay-temporary-service-unwrapped ]]; then
    replay_scratch="$(mktemp -d)"
    replay_binary="$(code_tooling_prune_server_binary "$replay_scratch")" || replay_binary=""
  fi
  # 进度间隔（这一格文件头「进度」）：样本根里的 .break-switch-replay-progress-seconds 顶替 300 秒
  local replay_progress_seconds
  if ! replay_progress_seconds="$(switch_replay_progress_seconds .break-switch-replay-progress-seconds)"; then
    printf '%s\n' "$replay_progress_seconds"
    [[ -z "$replay_scratch" ]] || rm -rf -- "$replay_scratch"
    exit 1
  fi
  # 范围（这一格文件头「范围」）：给了 --check 或 --item（样本根是 .gate-cells 与 .gate-items）是指定范围，什么都没给是不指定范围；
  # 样本根的 .break-switch-replay-whole-scope 把它改成不指定范围，.break-switch-replay-jobs 给并行度的输入（这一格文件头「并行度」）
  local replay_scope=whole replay_knobs="" replay_work_root=""
  [[ "$STAGE_CELLS_SELECTION_SOURCE" == "全部登记的格" ]] || replay_scope=named
  # 判红的明细文件放工作根下（这一格文件头「判红的明细」）：真仓上取 lib_work_root.py 不带参数打的工作根，取不到留空（python 那一侧改放本机临时目录并写明）；样本根留空
  if ! judged_root_is_sample; then
    replay_work_root="$(python3 "$STAGE_REPOSITORY/.claude/hooks/lib_work_root.py" 2>/dev/null)" || replay_work_root=""
  fi
  if judged_root_is_sample; then
    [[ ! -f .break-switch-replay-whole-scope ]] || replay_scope=whole
    [[ ! -f .break-switch-replay-jobs ]] || replay_knobs="$(<.break-switch-replay-jobs)"
  fi
  # 复用三问（这一格文件头「按条复用」的样本）：样本根带 .unit-verdict-sample/three-questions/ 的，同一份判定库上跑四趟、判三问
  if judged_root_is_sample && [[ -d .unit-verdict-sample/three-questions ]]; then
    code_tooling_three_question_rounds break-switch-replay break_switch_replay_pass; replay_exit=$?
  elif judged_root_is_sample && [[ -f .unit-verdict-sample/interrupted-resume/stop-after ]]; then
    code_tooling_interrupted_rounds break-switch-replay break_switch_replay_pass; replay_exit=$?
  elif judged_root_is_sample && [[ -f .break-switch-replay-second-pass ]]; then
    # 强制现判那一条不取回（这一格文件头「开关趟落盘」）：第二趟键都相同，复用哨兵照样本写的种子强制现判一条；
    # 样本根另放 .break-switch-replay-second-pass-whole 的，第二趟改按不指定范围判（指定范围判出的不许被不指定范围那一趟复用）
    break_switch_replay_pass ""; replay_exit=$?
    if ((replay_exit == 0)); then
      echo "── 第二趟（break-switch-replay）"
      [[ ! -f .break-switch-replay-second-pass-whole ]] || replay_scope=whole
      SINGLEFS_REUSE_SENTINEL_SEED="$(cat .break-switch-replay-second-pass)" break_switch_replay_pass ""; replay_exit=$?
    fi
  else
    break_switch_replay_pass ""; replay_exit=$?
  fi
  [[ -z "$replay_scratch" ]] || rm -rf -- "$replay_scratch"
  return "$replay_exit"
}

# break_switch_replay_pass <各条结局写到哪，空是不写>：这一格的一趟（算键、查库、起键对不上的那几份、记库、判）；
# registry、jobs、replay_binary、replay_required、replay_progress_seconds 取调用它的那一格的
break_switch_replay_pass() {
  python3 - "$(break_switch_replay_judge_code)" "$STAGE_REPOSITORY/.claude/hooks/lib_project_code_runs.py" "$registry" "${CODE_TOOLING_BREAK:-}" "$jobs" "${GATE_NOT_RUN_FILE:-}" \
    "$replay_binary" "$replay_required" "$SELFTEST_TEMPORARY_SERVICE_PREFIX" "$replay_progress_seconds" "$SWITCH_REPLAY_PROGRESS_LIBRARY" \
    "$UNIT_VERDICT_ROUNDS_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/admission.py" "$STAGE_NAME:break-switch-replay" "$BREAK_SWITCH_REPLAY_JUDGE_TOKEN" "$STAGE_REPOSITORY" \
    "${CODE_TOOLING_PICKED[break-switch-replay]:-}" "$(judged_root_is_sample && echo 1)" "$1" "$replay_scope" "$replay_knobs" "$SWITCH_REPLAY_PARALLELISM_LIBRARY" \
    "$BREAK_SWITCH_HEADER_LIBRARY" "$replay_work_root" "$TREE_LISTING_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py" <<'PY'
import collections, concurrent.futures, glob, hashlib, importlib.util, os, re, shutil, sys, tempfile, threading, time, types
judge_library = types.ModuleType("lib_break_switch_replay_judge")
exec(compile(sys.argv[1], "break-switch-replay-judge-library", "exec"), judge_library.__dict__)
table = judge_library   # 开关表的读法与判法在同一个模块里（break_switch_replay_judge_code 打的两段）
exec(compile(sys.argv[23], "break-switch-header-library", "exec"), table.__dict__)   # 读没读开关、写全没写全、复跑登记表的取法：与 break-switch-headers 格同一份（不进判定的键）
runs_library_path, registry_path, broken, default_threads, not_run_file = sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]), sys.argv[6]
server_binary, service_required, service_prefix = sys.argv[7], set(sys.argv[8].split("\n")) - {""}, sys.argv[9].split()
progress_seconds = int(sys.argv[10])     # 进度间隔（这一格文件头「进度」）
progress_library = types.ModuleType("lib_switch_replay_progress")
exec(compile(sys.argv[11], "switch-replay-progress-library", "exec"), progress_library.__dict__)
rounds = types.ModuleType("lib_unit_verdict_rounds")
exec(compile(sys.argv[12], "unit-verdict-rounds-library", "exec"), rounds.__dict__)
admission, kind, judge_token, judge_root = sys.argv[13], sys.argv[14], sys.argv[15], sys.argv[16]
picked = [item for item in sys.argv[17].split("\n") if item]      # 逐项点名（这一道「逐项点名」一节）：空是整格
sample, states_path = sys.argv[18] == "1", sys.argv[19]
work_root_text = sys.argv[24]    # 判红的明细文件放在这个工作根下（这一格文件头「判红的明细」）：样本根与取不到时是空
parallel_library = types.ModuleType("lib_switch_replay_parallelism")
exec(compile(sys.argv[22], "switch-replay-parallelism-library", "exec"), parallel_library.__dict__)
# 范围（这一格文件头「范围」）：scoped 是指定范围；selecting 是指定范围且有选格手段的自证只起点名的格（弄坏开关 replay-scope-ignored 下不选，replay-whole-scoped 下不指定范围也选）
scoped = sys.argv[20] == "named" or broken == "replay-whole-scoped"
selecting = scoped and broken != "replay-scope-ignored"
try:
    knobs = parallel_library.parse_knobs(sys.argv[21])
except ValueError as error:
    print(f"  ✗ {error}，一条都没跑")
    print(f"     → 怎么办：样本根的 {parallel_library.KNOB_FILE} 一行一项，写 memory-max=<字节数|max|unreadable> 或 threads=<正整数>。")
    sys.exit(1)
threads = int(knobs.get("threads", default_threads))                           # 登记的每台默认重活线程数（样本根的 threads 顶替）
memory_limit, memory_limit_source = parallel_library.memory_limit_of(knobs)    # 这一趟的内存上限（字节，读不到或是 max 是 None）与它的出处
needs_service = set()   # 只在临时 prune-server 上跑的那几份（要临时服务的表里登记成 required，shell 一侧读好交进来的 service_required）：基准趟与开关趟都包一层起
spec = importlib.util.spec_from_file_location("lib_project_code_runs", runs_library_path)
runs_library = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runs_library)
selftests, _ = runs_library.read_selftest_registry(".claude/hooks/selftest-registry.tsv")
problems, registered, queue, baseline_seconds_of = [], {}, [], {}
selection_of = {}   # 路径 → 选格的环境变量名（有选格手段的自证，指定范围时才记）
declared_summary_of = {}   # 路径 → 文件头 break-summary-rows 声明的汇总行原文（这一格文件头「汇总行的认法」）；弄坏开关 replay-summary-declaration-ignored 下不读
BASELINE = "基准趟"
picked_paths, picked_switches = set(), {}
for item in picked:
    item_path, separator, item_switch = item.partition(":")
    if separator:
        picked_switches.setdefault(item_path, set()).add(item_switch)
    else:
        picked_paths.add(item_path)


def path_picked(path):
    return not picked or path in picked_paths or path in picked_switches


# 只点了单条的那几份（这一格文件头「逐项点名」）：路径 → 点到的开关标签；同一份也点了整份的按整份。这几份只起基准趟与点到的开关趟，
# 汇总行认不出（共用 judge 的那一句「认不出汇总行」）；弄坏开关 replay-single-picks-co-run：照整份起、没点的开关陪跑
single_picked_of = ({} if broken == "replay-single-picks-co-run"
                    else {path: switches for path, switches in picked_switches.items() if path not in picked_paths})
UNRESOLVED_SUMMARY = "认不出汇总行"


def named_by_item(path, label):
    """这一条是不是点到的：整格、点了整份的，各条都是；点了 <路径>:<开关> 的，那几条与那一份的基准趟是（只点单条的那几份没点的开关不起；
    弄坏开关 replay-single-picks-co-run 下同一份别的开关趟陪跑）。"""
    return not picked or path in picked_paths or label == BASELINE or label in picked_switches.get(path, ())


registry_rows = table.replay_registry_paths(registry_path)
# 拆分族（这一格文件头「拆分族」）：登记的路径各自是入口；弄坏开关 replay-split-family-unread 下只认路径自己，replay-split-family-by-name 下部分文件只按命名认，
# replay-split-family-table-entry-only 下开关表与汇总行声明只读入口那一份的文件头
families = table.SplitFamilies(table.load_split_family(sys.argv[26]), independent=registry_rows, alone=broken == "replay-split-family-unread",
                               by_name=broken == "replay-split-family-by-name", entry_table_only=broken == "replay-split-family-table-entry-only")


def reads_switch(path):
    """→ (入口的文本, 入口加部分文件的代码读没读弄坏开关)。"""
    return families.text_of(path), bool(families.switch_names(path))

for number, line in enumerate(open(registry_path, encoding="utf-8"), 1):
    if not line.strip() or line.startswith("#"):
        continue
    columns = line.rstrip("\n").split("\t")
    if not path_picked(columns[0]):
        continue
    baseline_column = columns[2] if len(columns) == 4 else columns[1] if len(columns) == 3 else ""
    if (len(columns) not in (3, 4) or not columns[1].isdigit() or int(columns[1]) < 1 or not baseline_column.isdigit() or int(baseline_column) < 1
            or len(columns[-1].lstrip("#").strip()) < 4):
        problems.append(f"{registry_path}:{number} 不是「<路径><TAB><每条开关最多几秒>[<TAB><基准趟最多几秒>]<TAB># 理由>」（秒数是正整数，理由至少 4 个字）")
        continue
    path, seconds = columns[0], int(columns[1])
    registered[path] = seconds
    baseline_seconds_of[path] = int(baseline_column)
    if not os.path.isfile(path):
        problems.append(f"{registry_path}:{number} 指到的 {path} 不在，那一行删掉")
        continue
    text, reads = reads_switch(path)
    entries = families.family_entries(path)
    gaps = [] if broken == "replay-header-gaps-unjudged" else families.family_gaps(path)
    declared_items, declaration_problems = families.family_declaration(path)
    row = selftests.get(path)
    if not reads:
        problems.append(f"{registry_path}:{number} {path} 代码里不读弄坏开关了，那一行删掉")
    elif gaps:
        problems.extend(f"{registry_path}:{number} {table.gap_sentence(where, gap)}；复跑要文件头开关表三样写全、点名的项里「」成对（break-switch-headers 格点名同一条），先改好再复跑" for where, gap in gaps)
    elif declaration_problems:
        problems.extend(f"{registry_path}:{number} {path} {why}；汇总行声明写不对，复跑认不了汇总行，改成 break-summary-rows：「✗ …原文…」" for why in declaration_problems)
    elif row is None or row.kind != runs_library.SELFTEST_KIND:
        problems.append(f"{registry_path}:{number} {path} 在 .claude/hooks/selftest-registry.tsv 里不是自证的登记（{'没登记' if row is None else row.kind}），先登记成 selftest 再复跑")
    elif path in service_required and broken != "replay-temporary-service-unwrapped" and not server_binary:
        problems.append(f"{registry_path}:{number} {path} 只在临时 prune-server 上跑（要临时服务的表里登记成 required），这一趟取不到 prune-server 二进制（原因在上面一行），开关一条都没起")
    else:
        if path in service_required and broken != "replay-temporary-service-unwrapped":
            needs_service.add(path)
        variable = parallel_library.selection_variable(text) if selecting else None
        if variable:
            selection_of[path] = variable
        if declared_items and broken != "replay-summary-declaration-ignored":
            declared_summary_of[path] = declared_items
        queue.extend((path, seconds, entry) for entry in entries)


# 按条复用（这一格文件头「按条复用」）：一条判定 = 一份脚本加它的一条开关，外加那一份不设开关的基准趟；
# 键 = 那份脚本的单元闭包（入口就是那份脚本，照 research-script-selftests 格的算法）、判法（judge_token）、起它的命令样子、开关名与值、那一条点名的项的原文（文件头是注释、
# 不进单元指纹，另进一行 line:）；只在临时 prune-server 上跑的另加包装那一份；不带路径、行号与限时
def label_of(entry):
    return f"{entry['name']}={entry['value']}"


def item_name(path, label):
    return f"{path} {label}"


def command_text(path):
    return f"{'python3 -B' if path.endswith('.py') else 'bash'} {path} --selftest"


# 只点了单条的那几份（single_picked_of）：开关趟只留点到的那几条，文件头开关表的条数另记（成功句报「点了 k / n 条」）
switch_total_of = {path: len({label_of(entry) for job_path, _seconds, entry in queue if job_path == path}) for path in single_picked_of}
queue = [job for job in queue if job[0] not in single_picked_of or label_of(job[2]) in single_picked_of[job[0]]]
labels_of, entry_of = {}, {}
for path, _seconds, entry in queue:
    labels_of.setdefault(path, [BASELINE])
    if label_of(entry) not in labels_of[path]:
        labels_of[path].append(label_of(entry))
    entry_of[(path, label_of(entry))] = entry
all_paths = list(labels_of)
# 指定范围（这一格文件头「范围」）：有选格手段的自证，每条开关起点名的格——点名的项（「…」里的失败行句首）里「自证：<格名>：」之间那一段；
# 基准趟起这份每条开关点名的格的并集（开关趟的失败行减的是同一批格的）。一条开关读不出格名的，它那一趟整份起，这份的基准趟也整份起。
named_cells_of = {}      # (路径, 开关标签) → 点名的格名，排好序
for path, _seconds, entry in queue:
    cells = sorted({cell for item in judge_library.quoted_items(entry["items"]) for cell in parallel_library.cells_in(item, broken)})
    if path in selection_of and cells:
        named_cells_of[(path, label_of(entry))] = cells
baseline_cells_of = {}   # 路径 → 基准趟起的格名（并集）
for path in all_paths:
    switch_labels = [label for label in labels_of[path] if label != BASELINE]
    if path in selection_of and all((path, label) in named_cells_of for label in switch_labels):
        baseline_cells_of[path] = sorted({cell for label in switch_labels for cell in named_cells_of[(path, label)]})


def cells_of_trip(path, label):
    """这一趟（基准趟或一条开关）只起的格名；整份起的交 None。"""
    return baseline_cells_of.get(path) if label == BASELINE else named_cells_of.get((path, label))


def request_line(path, label):
    tokens = [judge_token, f"judge-root:{judge_root}", f"line:command={command_text(path)}", f"line:switch={label}"]
    cells = cells_of_trip(path, label)
    if cells and broken != "replay-scoped-key-shared":
        # 只起点名的格判出的，与整份判出的不是同一件事：键里另带选格变量与格名，不指定范围那一趟的键里没有这一行，复用不到
        tokens.append(f"line:cell-scope={selection_of[path]}:" + hashlib.sha256(",".join(cells).encode("utf-8")).hexdigest())
    # 判的是「这份脚本带 --selftest 跑」：python 与 shell 的程序入口都只收 --selftest 真走到的那一支（admission.py 文件头「判定请求」program-argument:）
    tokens.append("program-argument:--selftest")
    if path in needs_service:
        tokens += ["file:scripts/prune-server/prune_server_client_selftest.py", "line:temporary-service=wrapped"]
    if label != BASELINE and broken != "replay-named-unkeyed":
        tokens.append("line:named-sha256=" + hashlib.sha256(entry_of[(path, label)]["items"].encode("utf-8")).hexdigest())
    if label != BASELINE and path in declared_summary_of and broken != "replay-summary-declaration-unkeyed":
        # 声明的汇总行决定哪几行多红不要求点名，文件头是注释、不进单元指纹，另进这一行
        tokens.append("line:summary-sha256=" + hashlib.sha256("\n".join(declared_summary_of[path]).encode("utf-8")).hexdigest())
    if broken == "replay-key-whole-registry":
        tokens += [f"file:{other}" for other in sorted(set(registry_rows)) if os.path.isfile(other)]
    return "\t".join([item_name(path, label), path, *tokens])


request_lines = [request_line(path, label) for path in all_paths for label in labels_of[path]]
plan_scratch = tempfile.mkdtemp(prefix="break-switch-replay-plan.")
verdicts = rounds.ItemVerdicts(admission, os.getcwd(), kind, plan_scratch, "code-tooling.sh 的 break-switch-replay 格", sample,
                               "bash .claude/gate.d/code-tooling.sh --check break-switch-replay（逐条设上开关起那一份的 --selftest）", broken != "replay-red-not-recorded")
if request_lines:
    verdicts.plan(request_lines)
    if verdicts.unavailable:
        shutil.rmtree(plan_scratch, ignore_errors=True)
        print(f"  ✗ 脚本与钩子的弄坏开关复跑这一格没判：prune-server 不可用（上面那一句），{len(all_paths)} 份一条都没起，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check break-switch-replay")
        sys.exit(75)
    if not verdicts.ok:
        print(f"  ! 按条复用算不出（{verdicts.problem}）：这一趟每一份都起、一条都不记库")
named_items = [(path, label) for path in all_paths for label in labels_of[path] if named_by_item(path, label)]


def needs_judging(path, label):
    return verdicts.decision(item_name(path, label))[0] != rounds.REUSE


# 起哪几份：点到的条里有一条要现判（或拒算）的整份照起（基准趟与全部开关趟：汇总行要整份的开关趟，点名单跑一条与整格里判的相同）；点到的条都复用的整份跳过。
# 只点了单条的那几份，「整份」就是基准趟加点到的那几条（queue 与 labels_of 里只有它们）
run_paths = [path for path in all_paths
             if broken == "replay-plan-ignored" or any(needs_judging(path, label) for label in labels_of[path] if named_by_item(path, label))]
# 开关趟落盘（这一格文件头「开关趟落盘」）：要起的那几份里键相同、落了盘的趟取回，不再起；强制现判那一条不取回
trip_journal = rounds.TripJournal(verdicts, broken != "replay-trips-not-stored")
stopper = rounds.SampleStop(sample, os.getcwd(), plan_scratch)     # 样本档造中途被停（code_tooling_interrupted_rounds）：那一趟只开一个线程
if stopper.armed:
    threads = 1
trip_names = [item_name(path, label) for path in run_paths for label in labels_of[path]]
loaded_trips = {}
if verdicts.ok and broken not in ("replay-plan-ignored", "replay-trips-not-loaded"):
    loaded_trips = trip_journal.load([name for name in trip_names if broken == "replay-forced-trip-loaded" or not verdicts.forced(name)])
    if trip_journal.unavailable:
        shutil.rmtree(plan_scratch, ignore_errors=True)
        print(f"  ✗ 脚本与钩子的弄坏开关复跑这一格没判：取回落了盘的开关趟时 prune-server 不可用（{trip_journal.problem}），一条都没起，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check break-switch-replay")
        sys.exit(75)
    if trip_journal.problem:
        problems.append(f"落了盘的开关趟取不回（{trip_journal.problem}），这一趟要起的照全部起；照括号里那一句修 admission.py library-files 或判定库")
decision_count = collections.Counter(verdicts.decision(item_name(path, label))[0] for path, label in named_items)
judged_count, refused_count, reused_count = decision_count[rounds.JUDGE], decision_count[rounds.REFUSED], decision_count[rounds.REUSE]
co_run_count = sum(1 for path in run_paths for label in labels_of[path]
                   if not (named_by_item(path, label) and needs_judging(path, label)) and item_name(path, label) not in loaded_trips)
if request_lines:
    print(f"  · 按条复用：{len(all_paths)} 份脚本、{len(named_items)} 条判定（基准趟 {sum(1 for _path, label in named_items if label == BASELINE)} 条、"
          f"开关 {sum(1 for _path, label in named_items if label != BASELINE)} 条），现判 {judged_count}、复用 {reused_count}、拒算 {refused_count}；"
          f"起 {len(run_paths)} 份（陪跑 {co_run_count} 条：同一份里有条要现判就整份照起，汇总行要整份的开关趟）、跳过 {len(all_paths) - len(run_paths)} 份（点到的条都复用）", flush=True)
    print(f"  · 开关趟落盘：取回 {len(loaded_trips)} 趟（键相同、跑完就落了盘，不再起），这一趟要起 {len(trip_names) - len(loaded_trips)} 趟", flush=True)
    shown = 0
    for path, label in named_items:
        decision, reason = verdicts.decision(item_name(path, label))
        if decision == rounds.REUSE:
            continue
        shown += 1
        if shown <= 20:
            print(f"    {'拒算' if decision == rounds.REFUSED else '现判'}「{item_name(path, label)}」：{reason}")
    if shown > 20:
        print(f"    …现判与拒算另有 {shown - 20} 条没列")
    if reused_count <= 20 or sample:
        for path, label in named_items:
            decision, reason = verdicts.decision(item_name(path, label))
            if decision == rounds.REUSE:
                print(f"    复用「{item_name(path, label)}」：{reason}")
# 弄坏开关 replay-siblings-not-co-run：同一份只起要现判的那几条开关趟（改错的样子：汇总行少了陪跑的开关趟，点名单跑一条与整格里判的不同）
queue = [job for job in queue if job[0] in run_paths
         and (broken != "replay-siblings-not-co-run" or (named_by_item(job[0], label_of(job[2])) and needs_judging(job[0], label_of(job[2]))))]
# 这一趟的范围（成功句与开跑前一行都写，这一格文件头「范围」）：指定范围写有几条开关趟、几份基准趟只起点名的格，几条整份起；不指定范围写每条整份起
restricted_switches = sum(1 for path, _seconds, entry in queue if cells_of_trip(path, label_of(entry)))
restricted_baselines = sum(1 for path in {job[0] for job in queue} if baseline_cells_of.get(path))
if scoped:
    scope_text = f"范围：指定范围（只起点名的格：开关趟 {restricted_switches} 条、基准趟 {restricted_baselines} 份；整份起：开关趟 {len(queue) - restricted_switches} 条）"
else:
    scope_text = "范围：不指定范围（每条开关趟与基准趟都起整份自证，逐格相同）"
if queue:
    print(f"  · {scope_text}" + (f"；选格变量：{'、'.join(sorted(set(selection_of.values())))}" if selection_of else ""), flush=True)
green_of = {}            # 判完的条 → 判绿没有（记库用；判完就记，收尾再整份交一次）
recorded_names = set()   # 这一趟当场记过的条


def record_path(path):
    """这一份刚判完的条当场照键记（这一格文件头「按条复用」判完就记）：一份的开关趟齐了判完那一批记一次，要重跑的那几条判完各记一次。"""
    with progress_lock:
        mine = [(name, green) for name, green in green_of.items() if name.startswith(path + " ") and name not in recorded_names]
        recorded_names.update(name for name, _green in mine)
    verdicts.record([name for name, green in mine if green], [name for name, green in mine if not green])


def journaled_trip(record, entry):
    """一趟落盘的那一份（这一格文件头「开关趟落盘」）：判法读超时、退出码、坏字节、失败行与点到开关名的自证行，别的行不存。
    留下进程或停不掉的趟（record["left"] 不空）不落盘：下一趟照起，取回的趟都是收干净了的。"""
    switch = entry["name"] if entry is not None else None
    rows = [row for row in record["output"].split("\n")
            if "✗" in row or (switch and switch in row and ("自证" in row or "selftest" in row.lower()))]
    return {"timeout": record["timeout"], "code": record["code"], "bad": record["bad"], "output": "\n".join(rows)}


progress_lock = threading.Lock()         # 管下面几张按份记的表；拿着它不调 tracker（tracker 自己的锁在里面拿打印锁，不反过来）
started_of = {}                          # 路径 → 这份第一趟起跑的时刻（time.monotonic）
trips_of = collections.Counter()         # 路径 → 跑完的趟数（基准趟、开关趟与重跑都算）
settled_of = collections.Counter()       # 路径 → 判完的开关条数
matched_of = collections.Counter()       # 路径 → 判完而对上的开关条数（对、守卫，且没有坏字节）
finished = set()                         # 开关轮跑完（或基准趟没过、开关一条没起）的路径


def seconds_since(path):
    return int(time.monotonic() - started_of.get(path, time.monotonic()))


def running_paths_lines():
    """报总进度时另打的几行：几份里跑完几份，起跑了还没跑完的每份一行（跑得长的那份中途也有进度）。"""
    with progress_lock:
        running = [path for path in started_of if path not in finished]
        snapshot = [(path, trips_of[path], switch_count_of.get(path, 0), settled_of[path], seconds_since(path)) for path in running]
        done = len(finished)
    return ([f"break-switch-replay 按份：{len(paths)} 份里跑完 {done} 份、还在跑 {len(snapshot)} 份"]
            + [f"break-switch-replay 还在跑：{path} 开关 {switch_count} 条、已跑完 {trips} 趟、判完 {settled} 条、用了 {elapsed} 秒"
               for path, trips, switch_count, settled, elapsed in snapshot])


def finish(path, text):
    """一份的开关轮跑完：打一行（脚本名、开关数、对上几条、用了几秒）。"""
    with progress_lock:
        finished.add(path)
    tracker.say(text)


def trip_inner_threads(label):
    """这一趟拿到的里层线程数（这一格文件头「并行度」）：基准趟取基准趟那一段的，开关趟取开关趟那一段的。"""
    return baseline_inner_threads if label == BASELINE else inner_threads


def trip_units(path, label, threads_of_trip):
    """这一趟占几个内存单位：只起点名的格的趟，起几格就几个（自证自己并行起格，至多这一趟拿到的里层线程数 threads_of_trip 个同时在跑）；整份起的趟一个。"""
    cells = cells_of_trip(path, label)
    return min(len(cells), threads_of_trip) if cells else 1


def run_selftest(path, seconds, entry=None, threads_of_trip=None):
    """起一趟这份的 --selftest（entry 是 None 时不设开关，是基准趟）：→ {timeout, code, output, bad, seconds, peak_kib, left, threads}。
    threads_of_trip 是这一趟拿到的里层线程数，不给就取它那一段的（trip_inner_threads）；基准趟拿满线程重跑那一趟给登记的线程数（这一格文件头「重跑」）。
    先过内存这一关（admission，这一格文件头「并行度」），占到位置才起；用的秒数从占到位置算起。
    run_selftest_once 核完组里一个不剩才返回，名额在那之后还（这一格文件头「每趟的进程组」）；组里有停不掉的，内存额度不还。
    到点没跑完的趟量到的峰值是被停之前的，不换进估计。"""
    label = BASELINE if entry is None else label_of(entry)
    if threads_of_trip is None:
        threads_of_trip = trip_inner_threads(label)
    units = trip_units(path, label, threads_of_trip)
    with admission.slot(path, units) as hold:
        with progress_lock:
            started_of.setdefault(path, time.monotonic())
        started = time.monotonic()
        try:
            with tracker.running(path, f"{path} {'基准趟' if entry is None else entry['name'] + '=' + entry['value']}"):
                record = run_selftest_once(path, seconds, entry, hold, threads_of_trip)
            record["seconds"] = time.monotonic() - started
            record["threads"] = threads_of_trip
            if not record["timeout"]:
                admission.observe(path, entry is None, units, record["peak_kib"])
            if not record["left"]:
                trip_journal.store(item_name(path, BASELINE if entry is None else label_of(entry)), journaled_trip(record, entry))
            stopper.passed()
            return record
        finally:
            with progress_lock:
                trips_of[path] += 1
            tracker.trip_finished()


def first_trip(path, seconds, entry=None):
    """这一条这一趟的头一趟：落了盘、取回了的用取回的那一份（用了 0 秒、机读进度当场算跑完一趟），没有的现起。
    取回的那一份 threads 是 None：它是哪一趟、拿了几个线程跑的不落盘，到点的基准趟照「分过而到点」拿满线程重跑（这一格文件头「重跑」）；
    retrieved 记它是取回的，开关趟取回的判得不是对的，一份的头一趟收齐之后不当头一趟、并行重起（refetch_stale_trips，这一格文件头「开关趟落盘」）。"""
    loaded = loaded_trips.get(item_name(path, BASELINE if entry is None else label_of(entry)))
    if loaded is None:
        return run_selftest(path, seconds, entry)
    tracker.trip_finished()
    return dict(loaded, seconds=0.0, left="", threads=None, retrieved=True)   # 落盘的都是收干净了的趟（journaled_trip）


def run_selftest_once(path, seconds, entry, hold, threads_of_trip):
    environment = {name: value for name, value in os.environ.items() if not name.endswith("_BREAK")}
    if broken != "replay-inner-threads-uncapped":
        environment["SINGLEFS_THREAD_CAP"] = str(threads_of_trip)   # 里层线程数：这一趟拿到的并行度上限
    if entry is not None:
        environment[entry["name"]] = entry["value"]
    interpreter = ["python3", "-B"] if path.endswith(".py") else ["bash"]
    strict = {"text": True} if broken == "replay-output-strict-decode" else {}
    cells = cells_of_trip(path, BASELINE if entry is None else label_of(entry))
    if cells:   # 指定范围、有选格手段的自证：只起点名的格（这一格文件头「范围」）
        environment[selection_of[path]] = ",".join(cells)
    command = [*interpreter, path, "--selftest"]
    if path in needs_service:   # 包装写法：-- 前插 --server-binary <这一趟那一份>（这一格文件头「只在临时 prune-server 上跑的自证」）
        command = [*service_prefix[:-1], "--server-binary", server_binary, "--", *command]
    # 每趟自成一组、到点与收尾停整组、核完组里一个不剩才返回（这一格文件头「每趟的进程组」）
    trip = launcher.run(command, environment, seconds, f"{path} {BASELINE if entry is None else label_of(entry)}", meter, **strict)
    if trip["stuck"]:
        hold.keep = True
    if trip["timeout"]:
        return {"timeout": True, "code": None, "output": "", "bad": [], "peak_kib": trip["peak_kib"], "left": trip["problem"]}
    output, bad = judge_library.decoded_output((("stdout", trip["stdout"]), ("stderr", trip["stderr"])), broken)
    return {"timeout": False, "code": trip["code"], "output": output, "bad": bad, "peak_kib": trip["peak_kib"], "left": trip["problem"]}


def judge(path, entry, record):
    """判法照 judge_library.judge（基准趟的失败行与汇总行取这一份的 baseline_rows_of、summary_of）；这一趟没收干净的（record["left"] 不空，这一格文件头「每趟的进程组」）：
    对上的也记「留下进程」，别的结局在说明后面接上。
    声明了汇总行的脚本（文件头 break-summary-rows）照声明认汇总行，只点单条也认得出，多出来的行照声明认不出的就是判不符（照常记库）、说明末尾点名声明。
    没声明、只点了单条的那几份（这一格文件头「逐项点名」）没认汇总行：判成不符、而点名的项都照出来了、红的只是没被点名的 ✗ 行的（把多出来的 ✗ 行都当汇总行再判一次是对），
    那几行在整份里可能是汇总行，判不定：结局「认不出汇总行」，不记库；弄坏开关 replay-single-pick-summary-guessed 下照点到的几条认汇总行、照常判。"""
    baseline_rows = baseline_rows_of[path]
    kind, why = judge_library.judge(entry, record, baseline_rows, summary_of[path], broken)
    if kind == "不符" and path in declared_summary_of and "失败行没被点名" in why:
        why += (f"；文件头 break-summary-rows 声明了 {len(declared_summary_of[path])} 项汇总行，上面没被点名的行照声明都认不出：声明写得对不上这些行就改声明，"
                f"确是没点名的格就照实测改头注")
    if (kind == "不符" and path in single_picked_of and path not in declared_summary_of and broken != "replay-single-pick-summary-guessed" and not record["bad"] and not record["left"]
            and judge_library.judge(entry, record, baseline_rows, frozenset(judge_library.VOLATILE.sub("#", row) for row in judge_library.extra_rows(
                judge_library.failure_rows(record["output"]), baseline_rows)), broken)[0] == "对"):
        return UNRESOLVED_SUMMARY, (f"{why}；这一份只点了 {switch_count_of[path]} / {switch_total_of[path]} 条开关，这几行在整份里可能是汇总行（一半以上开关下都多红的同一行不要求点名），"
                                    f"只点单条认不出，这一条不记库、库里原有的判定不动；整份点 --item {path} 判这一条，或这几行确是没点名的格就照实测改头注")
    if not record["left"]:
        return kind, why
    if kind in ("对", "守卫"):
        return "留下进程", record["left"]
    return kind, f"{why}；{record['left']}"


def summary_keys_of(path, first):
    """这一份这一趟的各条开关趟（同一把键只留表里排在后面的那一趟）交判法认汇总行：声明了汇总行的（declared_summary_of）照声明认，没声明的靠整份统计；
    没声明、只点了单条的那几份不认（judge 的文档串），弄坏开关 replay-single-pick-summary-guessed 下照点到的几条认。"""
    records = [record for (job_path, _seconds, _switch), record in first.items() if job_path == path]
    if path in declared_summary_of:
        return declared_summary_keys(path, records)
    if path in single_picked_of and broken != "replay-single-pick-summary-guessed":
        return frozenset()
    return judge_library.summary_keys_of(records, baseline_rows_of[path], broken)


def declared_summary_keys(path, records):
    """声明了汇总行的脚本：各条红了的开关趟多出来的 ✗ 行里，照声明写的某一项照得出的行（去掉临时路径与数字之后）；有几条开关趟里出现都算，一条也算。
    弄坏开关 replay-summary-rows-required 下汇总行也要点名，同判法里那一条。"""
    if broken == "replay-summary-rows-required":
        return frozenset()
    keys = set()
    for record in records:
        if record["timeout"] or record["code"] in (0, None):
            continue
        for row in judge_library.extra_rows(judge_library.failure_rows(record["output"]), baseline_rows_of[path]):
            if row.startswith("✗") and any(judge_library.appears_in(item, row) for item in declared_summary_of[path]):
                keys.add(judge_library.VOLATILE.sub("#", row))
    return frozenset(keys)


paths = sorted({path for path, _seconds, _entry in queue})
seconds_of = {path: seconds for path, seconds, _entry in queue}
# 基准趟的限时读第三列（这一格文件头「两个限时分开读」）；弄坏开关 replay-baseline-shares-switch-limit：基准趟也照第二列
baseline_limit_of = {path: seconds_of[path] if broken == "replay-baseline-shares-switch-limit" else baseline_seconds_of[path] for path in paths}
baseline_rows_of, results = {}, []
switch_count_of = collections.Counter(path for path, _seconds, _entry in queue)
# 并行度（这一格文件头「并行度」）：并行条数取登记的每台默认重活线程数，内存上限只写进开跑那一行。
#   弄坏开关 replay-jobs-by-memory：照内存上限算并行条数（jobs_for）、Admission 带上限的份额（上限读不到或是 max 时这个开关不起作用）
fallback_trip_bytes = parallel_library.TRIP_MEMORY_FALLBACK_BYTES
by_memory = broken == "replay-jobs-by-memory" and memory_limit is not None
jobs = parallel_library.jobs_for(threads, memory_limit, fallback_trip_bytes) if by_memory else threads
if by_memory:
    plan_text = (f"内存上限 {memory_limit // parallel_library.MEBIBYTE} MiB、每趟 {fallback_trip_bytes // parallel_library.MEBIBYTE} MiB、"
                 f"份额 {parallel_library.SAFE_SHARE_PERCENT}%")
else:
    limit_text = f"内存上限{memory_limit_source}" if memory_limit is None else f"内存上限 {memory_limit // parallel_library.MEBIBYTE} MiB（{memory_limit_source}）"
    plan_text = f"登记的每台默认重活线程数 {threads} 条；{limit_text}，由服务定、撞了服务照乘 1.5 放大重排，不按它收并行"
# 外层同时在跑的趟数 × 每趟里层的线程数不超过登记的线程数，基准趟与开关趟两段各算各的（这一格文件头「并行度」）：
#   开关趟那一段外层路数的上限取并行条数与要起的趟数里小的，里层线程数取线程数除以它；同时在跑的不超过这个上限（Admission 照它数）。
#   基准趟那一段开关趟一条还没起，同时起的只有份数条基准趟（要重跑的一次一条），外层路数取上限与份数里小的，里层线程数取线程数除以它（一份就拿满）；
#   分过而到点的基准趟重跑那一趟只它一趟在跑，拿满线程数（这一格文件头「重跑」）。
#   两段的里层线程数都经环境变量 SINGLEFS_THREAD_CAP 交给那一趟起的自证（自证里的并行度都经登记位取，登记位认这个变量）
outer_cap = max(1, min(jobs, len(queue) + len(paths)))
inner_threads = threads if broken == "replay-inner-threads-uncapped" else max(1, threads // outer_cap)
baseline_outer = max(1, min(outer_cap, len(paths)))
# 弄坏开关 replay-baseline-threads-split：基准趟那一段也拿开关趟那一段分过的数
baseline_inner_threads = (inner_threads if broken in ("replay-inner-threads-uncapped", "replay-baseline-threads-split")
                          else max(1, threads // baseline_outer))
# 要重跑的开关趟：全部开关趟收完之后一次一条，那一段同时在跑的只它一条，里层线程数取线程数 ÷ 1（这一格文件头「重跑」）；
# 弄坏开关 replay-rerun-threads-split：仍拿开关趟那一段分过的数
RERUN_OUTER = 1
rerun_inner_threads = inner_threads if broken == "replay-rerun-threads-split" else max(1, threads // RERUN_OUTER)
admission = parallel_library.Admission(parallel_library.budget_of(memory_limit) if by_memory else None, outer_cap, paths, fallback_trip_bytes)
meter = parallel_library.TripMeter(progress_library)


def stop_line(text):
    """「到点停整组」与「基准趟拿满线程重跑」那两种行：不随进度一起关（弄坏开关 replay-progress-silent 只关进度），拿进度库的打印锁打。"""
    with tracker.print_lock:
        print(f"  · break-switch-replay {text}", flush=True)


# 每趟自成一组、到点与收尾停整组（这一格文件头「每趟的进程组」）；门禁收到 TERM、INT、HUP 时各线程收自己那一组，处理在开池之前于主线程装上
launcher = parallel_library.TripLauncher(stop_line, broken == "replay-trip-leader-only-stopped")
launcher.install_stop_on_signals()
# 机读进度按趟报（这一格文件头「机读进度」）：开跑时要跑的是每份一趟基准趟（不起基准趟的弄坏开关下没有）加全部开关趟
tracker = progress_library.ReplayProgress("break-switch-replay", len(queue), progress_seconds, broken == "replay-progress-silent", running_paths_lines,
                                          broken != "replay-machine-progress-silent",
                                          (0 if broken == "replay-baseline-unsubtracted" else len(paths)) + len(queue))
if queue:
    tracker.start(f"break-switch-replay 开跑：{len(paths)} 份、{len(queue)} 条开关，并行 {jobs} 条（{plan_text}）；"
                  f"基准趟：外层 {baseline_outer} 条 × 里层 {baseline_inner_threads} = {baseline_outer * baseline_inner_threads}（不超过每台默认重活线程数 {threads}）；"
                  f"开关趟：外层 {outer_cap} 条 × 里层 {inner_threads} = {outer_cap * inner_threads}（不超过每台默认重活线程数 {threads}）；"
                  f"基准趟到点停整组而里层被分过的，全部基准趟收完之后拿满 {threads} 个线程、同一个限时一次一份重跑，已拿满还到点的不重跑；"
                  f"要重跑的开关趟全部开关趟收完之后一次一条、拿满 {rerun_inner_threads} 个线程；每条判完打一行、每份开关轮跑完打一行，"
                  f"每过 {progress_seconds} 秒打一行总进度并给还没跑完的每份打一行")


def settled(index, kind, why, record, seconds):
    """第 index 条开关判完：记结局，打逐条进度一行；这一份的最后一条判完时另打「跑完一份」。"""
    path, _seconds, entry = queue[index]
    with progress_lock:
        outcome_of[index] = (kind, why, record["bad"])
        outcome_record_of[index] = record
        settled_of[path] += 1
        matched_of[path] += kind in ("对", "守卫") and not record["bad"]
        last = settled_of[path] == switch_count_of[path]
        matched = matched_of[path]
        if kind != UNRESOLVED_SUMMARY:   # 认不出汇总行的判不定，不记库（judge 的文档串）
            green_of[item_name(path, label_of(entry))] = judge_library.switch_green(kind, record)
    if kind not in ("对", "守卫"):
        verdict = f"没对上（{kind}{'、坏字节' if record['bad'] else ''}）"
    elif record["bad"]:
        verdict = "没对上（坏字节）"
    else:
        verdict = "对上（自证自带开关轮）" if kind == "守卫" else "对上"
    tracker.item_done(verdict, f"{path} {entry['name']}={entry['value']}", seconds)
    if last:
        finish(path, f"break-switch-replay 跑完一份：{path} 开关 {switch_count_of[path]} 条、对上 {matched} 条、用了 {seconds_since(path)} 秒")


def rerun_and_settle(index, first_record, threads_of_trip=None):
    """不是对的先单独重跑一次再定（这一格文件头「判据」末尾）；用了几秒算两趟的和。
    threads_of_trip 是重跑那一趟的里层线程数：全部开关趟收完之后一次一条的重跑给 rerun_inner_threads，与别的趟一起在池子里跑的不给（取开关趟那一段的）。"""
    path, seconds, entry = queue[index]
    record = run_selftest(path, seconds, entry, threads_of_trip)
    kind, why = judge(path, entry, record)
    settled(index, kind, why, record, first_record["seconds"] + record["seconds"])
    record_path(path)


def baseline_first_trip_text(record):
    """基准趟头一趟拿了几个线程、跑了几秒（「基准趟拿满线程重跑」那一行与没过的失败行用）；取回的落盘趟这两样都不落盘。"""
    if record["threads"] is None:
        return "头一趟是取回的落盘趟（拿几个线程、跑了几秒都不落盘）"
    return f"头一趟里层 {record['threads']} 个线程、跑了 {record['seconds']:.0f} 秒"


outcome_of, outcome_record_of, summary_of = {}, {}, {}
with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(admission.max_running, 2 * len(queue) + len(paths)))) as pool:
    # 基准趟：不设开关起一趟；要重跑的等全部基准趟收完、主线程里一次一份（这一格文件头「重跑」）：
    #   到点停整组而头一趟里层被分过（或是取回的、拿几个线程不明）的，拿满登记的线程数、同一个限时重跑，结局取重跑那一趟；
    #   头一趟已经拿满还到点的不重跑、判红；退非 0、留下进程的照头一趟那一段的线程数单独重跑一次再定。
    #   弄坏开关 replay-baseline-timeout-rerun-same-threads：到点的一律照头一趟那一段的线程数重跑一次（分过的不拿满，拿满的也再跑一趟）
    if broken != "replay-baseline-unsubtracted":
        baselines = dict(zip(paths, pool.map(lambda path: first_trip(path, baseline_limit_of[path]), paths)))
        for path in paths:
            record, limit, rerun_note = baselines[path], baseline_limit_of[path], ""
            split = record["threads"] is None or record["threads"] < threads
            if record["timeout"] and broken == "replay-baseline-timeout-rerun-same-threads":
                tracker.trips_added(1)
                record = run_selftest(path, limit)
            elif record["timeout"] and split:
                first_text = baseline_first_trip_text(record)
                stop_line(f"基准趟拿满线程重跑：{path}（{first_text}到点停整组；这一趟里层 {threads} 个线程、同一个限时 {limit} 秒）")
                tracker.trips_added(1)
                record = run_selftest(path, limit, threads_of_trip=threads)
                rerun_note = f"；{first_text}到点，拿满 {threads} 个线程重跑那一趟跑了 {record['seconds']:.0f} 秒"
            elif record["timeout"]:
                rerun_note = f"；头一趟里层已拿满登记的 {threads} 个线程、跑了 {record['seconds']:.0f} 秒到点，不重跑"
            elif record["code"] != 0 or record["left"]:
                tracker.trips_added(1)
                record = run_selftest(path, limit)
            with progress_lock:
                green_of[item_name(path, BASELINE)] = judge_library.baseline_green(record) and not record["left"]
            if record["timeout"] or record["code"] != 0 or record["left"]:
                reason = "；".join(part for part in ("超时：基准趟限时 " + str(limit) + " 秒" if record["timeout"] else
                                                      "退 " + str(record["code"]) if record["code"] != 0 else "", record["left"]) if part)
                problems.append(f"{path}：不设开关的 --selftest 就没过（{reason}{rerun_note}），开关下的失败行比不出哪些是开关弄红的，这一份的开关一条都没起")
                finish(path, f"break-switch-replay 跑完一份：{path} 开关 {switch_count_of[path]} 条、对上 0 条（基准趟没过，开关一条没起）、用了 {seconds_since(path)} 秒")
                record_path(path)
                continue
            problems.extend(f"{path} 不设开关：坏字节，{why}，点名的项比不准；改那份脚本让它只吐 UTF-8" for why in record["bad"])
            baseline_rows_of[path] = judge_library.failure_rows(record["output"])
    else:
        # 弄坏开关 replay-baseline-unsubtracted：不起基准趟，基准趟那一条这一趟没判、不记库
        baseline_rows_of = {path: [] for path in paths}
    queue = [job for job in queue if job[0] in baseline_rows_of]
    tracker.total = len(queue)
    tracker.trips_left(len(queue))
    # 开关趟全部派出去；一份的开关趟齐了就当场判这一份（汇总行要这一份的全部开关趟），不等别的份，逐条进度随判随打。
    # 同一把键（路径、限时、开关）只留表里排在后面的那一趟，汇总行与判定都读这一份；要重跑的另派出去。
    key_of = [(path, seconds, entry["name"] + "=" + entry["value"]) for path, seconds, entry in queue]
    indexes_of = collections.defaultdict(list)
    for index, (path, _seconds, _entry) in enumerate(queue):
        indexes_of[path].append(index)
    waiting_of = {path: len(indexes) for path, indexes in indexes_of.items()}
    # 要重跑的（这一格文件头「判据」末尾「重跑」；取回的判得不是对的先并行重起，这一格文件头「开关趟落盘」）：记下来，别的趟都收完之后一次一条地起；
    # 弄坏开关 replay-reruns-batched：当场派进池子，与还在跑的趟、同一批要重跑的一起起
    record_of, first, reruns, rerun_later = {}, {}, [], []
    refetched = set()   # 取回的落盘趟判得不是对的、已并行重起过的那几份（一份只重起一次）

    def first_records_of(path):
        """这一份各条开关趟手上的头一趟结局记进 first（同一把键只留表里排在后面的那一趟），汇总行照它认。"""
        for sibling in indexes_of[path]:
            first[key_of[sibling]] = record_of[sibling]
        summary_of[path] = summary_keys_of(path, first)

    def stale_retrieved_trips(path):
        """这一份取回的、判得不是对的开关趟（queue 里的下标）：落盘的那一趟不是这一趟自己起的头一趟，不进「先单独重跑一次再定」那一段（一次一条）。
        判法照 judge；认不出汇总行的（UNRESOLVED_SUMMARY）是这一份的整体性质，重起一趟改不了它，不算。
        弄坏开关 replay-resumed-bad-trips-serial：不挑，取回的判得不是对的照头一趟判、进一次一条的重跑。"""
        if broken == "replay-resumed-bad-trips-serial":
            return []
        first_records_of(path)
        stale = []
        for sibling in indexes_of[path]:
            record = record_of[sibling]
            if not record.get("retrieved"):
                continue
            kind, _why = judge(path, queue[sibling][2], record)
            if kind != UNRESOLVED_SUMMARY and (kind not in ("对", "守卫") or record["bad"]):
                stale.append(sibling)
        return stale

    # 头一趟都是派出去的：现跑的与取回的；一份的头一趟收齐，取回而判得不是对的先并行重起（与别的开关趟同一个外层路数、同一个里层线程数），
    # 重起的这几趟收齐之后才判这一份；重起的还不是对的，才进下面一次一条的重跑
    pending = {pool.submit(first_trip, path, seconds, entry): index for index, (path, seconds, entry) in enumerate(queue)}
    while pending:
        done, _unfinished = concurrent.futures.wait(pending, return_when=concurrent.futures.FIRST_COMPLETED)
        for future in done:
            index = pending.pop(future)
            record_of[index] = future.result()
            path = queue[index][0]
            waiting_of[path] -= 1
            if waiting_of[path]:
                continue
            stale = [] if path in refetched else stale_retrieved_trips(path)
            refetched.add(path)
            if stale:
                outcomes = collections.Counter("到点" if record_of[sibling]["timeout"] else f"退 {record_of[sibling]['code']}" for sibling in stale)
                stop_line(f"取回的落盘趟里 {len(stale)} 趟判得不是对的（取回时：{'、'.join(f'{what}：{count} 趟' for what, count in sorted(outcomes.items()))}）：{path} 这几趟不当头一趟，"
                          f"与别的开关趟一样按外层 {outer_cap} 条 × 里层 {inner_threads} 并行重起，重起的还不是对的才一次一条重跑")
                for sibling in stale:
                    tracker.trips_added(1)
                    waiting_of[path] += 1
                    pending[pool.submit(run_selftest, path, queue[sibling][1], queue[sibling][2])] = sibling
                continue
            first_records_of(path)
            for sibling in indexes_of[path]:
                record = first[key_of[sibling]]
                kind, why = judge(path, queue[sibling][2], record)
                if kind not in ("对", "守卫") or record["bad"]:
                    tracker.trips_added(1)
                    if broken == "replay-reruns-batched":
                        reruns.append(pool.submit(rerun_and_settle, sibling, record))
                    else:
                        rerun_later.append((sibling, record))
                else:
                    settled(sibling, kind, why, record, record["seconds"])
            record_path(path)
    for future in concurrent.futures.as_completed(reruns):
        future.result()
    for sibling, record in sorted(rerun_later, key=lambda pair: pair[0]):
        rerun_and_settle(sibling, record, rerun_inner_threads)   # 一次一条、那一段只它在跑：拿满（这一格文件头「重跑」）
    results = [outcome_of[index] for index in range(len(queue)) if index in outcome_of]
meter.close()
tracker.close()
if len(results) != len(queue):
    problems.append(f"派出去 {len(queue)} 条、收回 {len(results)} 条，对不上，这一趟不算数")


def mismatch_detail(index):
    """一条判成「不符」的开关的完整明细 → (点名的项, 这一趟多出来的失败行, 点名的项里没在失败行里的, 多出来的失败行里没被点名的)，一个字都不截。
    取法与 judge_library.judge 的「不符」那一支同一份：点名的项取 quoted_items，失败行取这一趟的减基准趟的，汇总行不要求点名；
    弄坏开关 replay-items-unchecked、replay-unnamed-rows-unchecked 下照 judge 的样子少算。"""
    path, _seconds, entry = queue[index]
    items = judge_library.quoted_items(entry["items"])
    extra = judge_library.extra_rows(judge_library.failure_rows(outcome_record_of[index]["output"]), baseline_rows_of[path])
    missing = [item for item in items if broken != "replay-items-unchecked" and not any(judge_library.appears_in(item, row) for row in extra)]
    unnamed = [] if broken == "replay-unnamed-rows-unchecked" else [
        row for row in extra if row.startswith("✗") and judge_library.VOLATILE.sub("#", row) not in summary_of[path] and not any(judge_library.appears_in(item, row) for item in items)]
    return items, extra, missing, unnamed


def write_mismatch_file(indexes):
    """判成「不符」的各条开关（queue 里的下标）的完整明细落到一份文件里（这一格文件头「判红的明细」）→ (文件路径, {下标: 它那一节在文件里的起始行})；没有不符的是 (None, {})。
    写完回读：点名的项逐项、多出来的失败行逐行都得在文件里，缺了记一处不对；核过的打一行。真仓上放工作根下新建的目录（两台读得到），样本根放本机临时目录、核完就删。
    弄坏开关 replay-mismatch-detail-truncated：照日志截（失败行只写前三行、每行前八十个字，点名的项只写前三项、每项前六十个字）。"""
    if not indexes:
        return None, {}
    note, directory = "", None
    if work_root_text and not sample:
        try:
            os.makedirs(work_root_text, exist_ok=True)
            directory = tempfile.mkdtemp(prefix="break-switch-replay-mismatches.", dir=work_root_text)
        except OSError as error:
            note = f"（工作根 {work_root_text} 用不了：{error}；明细落在本机临时目录，第二台读不到）"
    elif not sample:
        note = "（取不到工作根；明细落在本机临时目录，第二台读不到）"
    directory = directory or tempfile.mkdtemp(prefix="break-switch-replay-mismatches.")
    truncated = broken == "replay-mismatch-detail-truncated"
    lines, start_of, details = ["# break-switch-replay 判成「不符」的开关的完整明细（点名的项与失败行都不截断）"], {}, {}
    for index in indexes:
        path, _seconds, entry = queue[index]
        items, extra, missing, unnamed = mismatch_detail(index)
        details[index] = (items, extra, missing, unnamed)
        start_of[index] = len(lines) + 1
        written_items = [item[:60] for item in items[:3]] if truncated else items
        written_rows = [row[:80] for row in extra[:3]] if truncated else extra
        lines.append(f"## {path} {entry['name']}={entry['value']}")
        lines.append(f"点名的项 {len(items)} 项（没在失败行里的标「没红」）：")
        lines.extend(f"  [{'没红' if item in missing else '红了'}] {text}" for item, text in zip(items, written_items))
        lines.append(f"这一趟多出来的失败行 {len(extra)} 行（没被点名的标「没点名」）：")
        lines.extend(f"  [{'没点名' if row in unnamed else '已点名'}] {text}" for row, text in zip(extra, written_rows))
    file_path = os.path.join(directory, "mismatches.txt")
    with open(file_path, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    with open(file_path, encoding="utf-8") as handle:
        content = handle.read()
    absent = [text for items, extra, _missing, _unnamed in details.values() for text in [*items, *extra] if text not in content]
    if absent:
        problems.append(f"不符明细文件 {file_path} 回读时缺 {len(absent)} 处完整的点名的项或失败行（比如：{absent[0][:80]}）{note}")
    else:
        print(f"  · 不符明细回读核过：{len(indexes)} 条开关、点名的项 {sum(len(detail[0]) for detail in details.values())} 项、"
              f"失败行 {sum(len(detail[1]) for detail in details.values())} 行（其中没被点名的 {sum(len(detail[3]) for detail in details.values())} 行）都在文件里{note}"
              f"{'（样本根：核完就删）' if sample else ''}", flush=True)
    if sample and not absent:
        shutil.rmtree(directory, ignore_errors=True)
    return file_path, start_of


result_indexes = [index for index in range(len(queue)) if index in outcome_of]
mismatch_file, mismatch_start = write_mismatch_file([index for index in result_indexes if outcome_of[index][0] == "不符"])
# 判成「不符」的那几条才重算键：跑的过程中键变了的，失败行里多一句（ItemVerdicts.key_changes）
key_change_notes = verdicts.key_changes([item_name(queue[index][0], label_of(queue[index][2])) for index in result_indexes if outcome_of[index][0] == "不符"])
for index, (kind, why, bad) in zip(result_indexes, results):
    path, _seconds, entry = queue[index]
    if kind not in ("对", "守卫"):
        where = f"；完整的失败行与点名的项在 {mismatch_file}:{mismatch_start[index]}" if index in mismatch_start else ""
        key_changed = key_change_notes.get(item_name(path, label_of(entry)))
        problems.append(f"{path} {entry['name']}={entry['value']}：{kind}，{why}{where}" + (f"；{key_changed}" if key_changed else ""))
    problems.extend(f"{path} {entry['name']}={entry['value']}：坏字节，{reason}，改那份脚本让它在这条开关下只吐 UTF-8" for reason in bad)
# 收尾把这一趟判完的条整份再交一次（同一把键写同样的内容），复用哨兵强制现判的那一条在这一趟比
verdicts.record([name for name, green in green_of.items() if green], [name for name, green in green_of.items() if not green], final=True)
if any(not green for green in green_of.values()):
    verdicts.reread_after_red()
verdicts.write_states(states_path, [item_name(path, label) for path, label in named_items])
shutil.rmtree(plan_scratch, ignore_errors=True)
unregistered = []
tree = types.ModuleType("lib_tree_listing")
exec(compile(sys.argv[25], "tree-listing-library", "exec"), tree.__dict__)
selftest_files = [] if picked else sorted(path for directory in ("scripts", ".claude/hooks") for path in tree.files_under(directory, (".sh", ".py"), skipped=("target", "fixtures")))
for path in selftest_files:
    if path in registered or families.entry_of(path) != path:
        continue   # 登记了的这一趟已判；部分文件读的开关归它的入口那一份算
    _text, reads = reads_switch(path)
    row = selftests.get(path)
    if reads and not families.family_gaps(path) and row is not None and row.kind == runs_library.SELFTEST_KIND:
        unregistered.append(path)
if unregistered and not_run_file:
    with open(not_run_file, "a", encoding="utf-8") as handle:
        handle.write(f"code-tooling.sh break-switch-replay：{len(unregistered)} 份文件头写全的脚本还没接复跑\n")
guarded = sum(1 for kind, _why, _bad in results if kind == "守卫")
# 只点了单条的那几份（这一格文件头「逐项点名」）：成功句与汇总行写明各点了几条，没点的那几条这一趟没起、不算判过
single_paths = [path for path in all_paths if path in single_picked_of]
single_text = ""
if single_paths:
    single_text = (f"；只点了单条的 {len(single_paths)} 份："
                   + "、".join(f"{path} 点了 {len(labels_of[path]) - 1} / {switch_total_of[path]} 条" for path in single_paths)
                   + f"（没点的 {sum(switch_total_of[path] - len(labels_of[path]) + 1 for path in single_paths)} 条这一趟没起、不算判过）")
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 脚本与钩子的弄坏开关复跑：{len(problems)} 处不对（复跑了 {len(queue)} 条、{len(set(path for path, _s, _e in queue))} 份）{single_text}")  # gate-lint:summary
    print("     → 怎么办：点名的项没在失败行里、失败行没被点名的，照实测改那份文件头那一条下面「」里的失败行原文，让点名的格与实际红的格逐格相同"
          "（实测是对的才改头注；自证判错是因为判法坏了就修判法）；"
          f"设上之后自证照样退 0 的，开关没弄坏它说的那一处，改开关或改头注；不设开关自证就没过的，先修那份自证；输出带不合法 UTF-8 的（坏字节），改那份脚本让它在那条开关下只吐 UTF-8"
          f"（截断的多字节字符、原样转发的二进制都会这样）；留下进程或停不掉的，改那份自证让它起的进程在它退出之前都收干净"
          f"（.claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」）；基准趟超时的（自证自带开关轮的最常见），在 {registry_path} 那一行第三列写基准趟限时；"
          f"这份还接不进复跑的，从 {registry_path} 删掉那一行；判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
if verdicts.unavailable:
    print("  ✗ 脚本与钩子的弄坏开关复跑：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75")
    print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check break-switch-replay")
    sys.exit(75)
if verdicts.red:
    print(f"  ✗ 脚本与钩子的弄坏开关复跑判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；起了的 {len(queue)} 条开关都对上")
    print("     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check break-switch-replay；不能复用就判红，不退回全部现判当绿")
    sys.exit(1)
if not request_lines:
    print("  ! 登记表里没有一条可复跑的开关，这一格无对象可判")
    sys.exit(77)
reuse_text = (f"按条复用：现判 {judged_count}、复用 {reused_count}、拒算 {refused_count}、陪跑 {co_run_count}；开关趟：现跑 {sum(trips_of.values())} 趟、取回 {len(loaded_trips)} 趟"
              if verdicts.ok else "按条复用算不出、全部现判")
if not paths:
    print(f"  ✓ 脚本与钩子的弄坏开关复跑：点到的 {len(named_items)} 条都有键相同的判定，全部复用，这一格一条都没起（{reuse_text}）；"
          f"没接复跑的 {len(unregistered)} 份文件头写全的脚本：{'、'.join(unregistered) or '（没有）'}；{scope_text}{single_text}")
    sys.exit(0)
print(f"  ✓ 脚本与钩子的弄坏开关复跑：复跑了 {len(queue)} 条开关（{len(paths)} 份），自证红且点名的格与实际红的格逐格相同 {len(queue) - guarded} 条、自证自带开关轮 {guarded} 条；"
      f"没接复跑的 {len(unregistered)} 份文件头写全的脚本：{'、'.join(unregistered) or '（没有）'}；{reuse_text}；{scope_text}{single_text}")
PY
}

# ── 格 kb-directories-write-scope：还拆分提案 records/2026-09-16-subagent拆分提案.md 第九十节 ⑧（候选扫描改读 .claude/kb-directories 之后，写回员的写范围仍是写死的模式，
#   载体再搬家时写范围那一侧照样漏）。
# 判据：被判的根里 .claude/kb-directories 登记的每个 kb 目录（读法是这一道所在仓里 scripts/kb/quote-kb.py 的 kb_candidate_roots，不另写一份：登记是替换不是追加，
#   没登记、或登记的都不在、都被拒就是 .claude/kb），写回员（kb-scribe）在被判的根的 .claude/hooks/agent-write-scope.tsv 里那几行路径模式，要盖住这个目录下一层的 .md
#   与任意一层子目录里的 .md。探两条造出来的路径：<目录>/kb-directories-write-scope-probe.md 与 <目录>/kb-directories-write-scope-probe/kb-directories-write-scope-probe.md，
#   不看目录里此刻有没有子目录（载体搬家之后才长出来的子目录同样要写得到）。哪一条没有一条模式配上，判红、逐个目录点名缺哪一层。
# 「盖住」照写范围闸本身的配法：这一道所在仓里 .claude/hooks/lib_write_scope.py 的 expanded_scope_patterns 与 path_in_scope（写闸 write-guard.sh 与 Bash 检出钩子用的同一份），
#   ** 跨目录（配成 .*）、* 不跨目录（配成 [^/]*）：<目录>/*.md 只盖下一层，子目录里的 .md 写不到；<目录>/**/*.md 只盖子目录，下一层写不到；<目录>/** 两样都盖。
#   以 $SINGLEFS_WORK_ROOT/ 起头的模式只管工作根，不展开、不参与。要子目录也算：候选扫描（quote-kb.py 的 invariant_class_files、门禁 doc-registries 的
#   kb_candidate_md_files）认登记目录下各层子目录里的 .md，写回员要能把去向写回那些文件。别的 agent 的行盖住了不算。
# 被判的根里没有 .claude/hooks/agent-write-scope.tsv：退 77（写范围闸没有表，无对象可判）。表里没有 kb-scribe 的行：每个登记目录都判红。
# 样本：kb-directories-write-scope-red（登记三个目录：.claude/kb 由 ** 盖全；notes/invariants 只有 *.md，子目录写不到；docs/extra 只被 clerk 的 docs/** 盖住、
#   kb-scribe 一条都没有；后两个逐个点名判红）；kb-directories-write-scope-green（三个目录 kb-scribe 都由 ** 盖全，报「盖住登记的 3 个 kb 目录」）。
# 弄坏开关 CODE_TOOLING_BREAK=kb-scope-subdirectory-unchecked：只探下一层、不探子目录，kb-directories-write-scope-red 照样退 1（docs/extra 那一个照红），
#   「notes/invariants：写回员（kb-scribe）写不到子目录里的 .md」与「登记的 3 个 kb 目录里 2 个写回员写不全」两条 want 找不到。
cell_kb_directories_write_scope() {
  local table=".claude/hooks/agent-write-scope.tsv"
  [[ -f "$table" ]] || { echo "  ! 被判的根里没有 $table，写范围闸无表可判，这一格无对象可判"; exit 77; }
  python3 - "$STAGE_REPOSITORY/scripts/kb/quote-kb.py" "$STAGE_REPOSITORY/.claude/hooks/lib_write_scope.py" "$table" "$ROOT" "${CODE_TOOLING_BREAK:-}" <<'PY'
import importlib.util, os, sys
sys.dont_write_bytecode = True
quote_kb_path, write_scope_path, table, root, broken = sys.argv[1:6]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


quote_kb = load("quote_kb", quote_kb_path)
write_scope = load("lib_write_scope", write_scope_path)
WRITER = "kb-scribe"
PROBE = "kb-directories-write-scope-probe"
directories = quote_kb.kb_candidate_roots(root)
patterns = write_scope.load_scopes(table).get(WRITER, [])
judged_patterns = write_scope.expanded_scope_patterns(patterns, None)
probes = [("下一层的 .md", f"{PROBE}.md")]
if broken != "kb-scope-subdirectory-unchecked":
    probes.append(("子目录里的 .md", f"{PROBE}/{PROBE}.md"))
problems = []
for directory in directories:
    missing = []
    for what, tail in probes:
        probe = os.path.join(directory, tail)
        if not write_scope.path_in_scope(judged_patterns, os.path.normpath(os.path.join(root, probe)), root):
            missing.append(f"{what}（探 {probe}）")
    if missing:
        problems.append(f"{quote_kb.KB_DIRECTORIES_REGISTRY} 登记的 {directory}：写回员（{WRITER}）写不到{'、'.join(missing)}")
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    written = "、".join(patterns) if patterns else f"表里没有 {WRITER} 的行"
    print(f"  ✗ 登记的 {len(directories)} 个 kb 目录里 {len(problems)} 个写回员写不全（{table} 里 {WRITER} 的模式：{written}；** 跨目录、* 不跨目录）")  # gate-lint:summary
    print(f"     → 怎么办：在 {table} 给 {WRITER} 加一行 <目录>/**（只许 .md 的写成 <目录>/*.md 与 <目录>/**/*.md 两行），定义 .claude/agents/{WRITER}.md「写范围」同步改；"
          f"那个目录不该写回的，从 {quote_kb.KB_DIRECTORIES_REGISTRY} 撤掉；判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ 写回员（{WRITER}）的写范围盖住登记的 {len(directories)} 个 kb 目录（每个探下一层与子目录里的 .md 各一条）：{'、'.join(directories)}")
PY
}

# ── 格 prune-server-selftest：仓根 prune-server/（判定库服务，records/2026-11-01-判定库服务化.md）的自证过。
# 判据：被判的根里 prune-server/Cargo.toml 在，就编 prune-server 的服务二进制、直接起 `prune-server --selftest`（每格起一份真服务与真 RocksDB，约 22 秒；
#   与 `cargo run --release --offline --bin prune-server -- --selftest` 同一个入口），读它的标准输出：恰好一行、且在最后一行的
#   `PRUNE_SERVER_SELFTEST cells=<N> passed=<P> failed=<K> failed_cells=<格名,…|->`，N 大于 0、P+K 等于 N、failed_cells 的个数等于 K，退出码 0 且 K 为 0 才绿；
#   退 1 且 K 大于 0 是自证有格红（逐格点名、判红）；退出码与 K 对不上、末行缺或不在最后、N 为 0、退 2 或别的码（格名或开关写错、没跑成），都是输出认不出，判红。
#   prune-server/Cargo.toml 不在：这一格无对象可判，退 77。每一趟都跑，不读判定库：判的是判定库服务自己，不入库（判定入库的射程之外，与 task-server 自己的自证同）。
# 起法：真仓与 --staged 的临时树上，先照这一道所在仓的 scripts/hosts/rocksdb-prebuilt.py entry --root <根> 判 RocksDB 预编库的取值（prune-server 依赖 rocksdb），判红这一格红；再在这一格自己的临时目录里 cargo build --release --offline --manifest-path <根>/prune-server/Cargo.toml --bin prune-server
#   （经 capped.sh default），在 <根>/prune-server 下直接起编出来的二进制，TMPDIR 指到临时目录里（自证的库与 socket 随临时目录一起删），临时目录连同编译目录跑完删（编不过时日志另拷一份留着）。
#   样本根（judged_root_is_sample）里放了 .prune-server-selftest-program 的，起它（python3，假的自证：只认 --selftest，打样本根里 .prune-server-selftest-lines 的内容再退样本根里 .prune-server-selftest-exit 写的码），不编。
# 样本（假的自证照样本根里的两份文件打行）：
#   prune-server-selftest-green（20 格都过，末行 failed=0、退 0）；prune-server-selftest-absent-green（根里没有 prune-server/，无对象退 77）；
#   prune-server-selftest-failed-red（末行 cells=20 passed=19 failed=1 failed_cells=batched-point-read、退 1：验收要的 failed=1 那一份）；
#   prune-server-selftest-empty-red（末行 cells=0 passed=0 failed=0、退 0：一格都没跑）；
#   prune-server-selftest-no-summary-red（只打各格的行、没有末行、退 0）；prune-server-selftest-exit-red（末行 failed=0，却退 1）。
# 弄坏开关 CODE_TOOLING_BREAK=prune-server-failed-ignored：不看末行的 failed= 与退出码，failed-red、exit-red 判绿；
#   弄坏开关 CODE_TOOLING_BREAK=prune-server-empty-run-accepted：N 为 0 也算过，empty-red 判绿；
#   弄坏开关 CODE_TOOLING_BREAK=prune-server-last-line-unchecked：末行缺或不在最后不判（有末行照旧核，没有就只看退出码），no-summary-red 判绿；
#   弄坏开关 CODE_TOOLING_BREAK=prune-server-exit-unchecked：不核退出码与 failed= 对不对得上，exit-red 判绿。
cell_prune_server_selftest() {
local scratch program=() run_exit kept_build_log program_binary build_log
[[ -f prune-server/Cargo.toml ]] || { echo "  ! 被判的根里没有 prune-server/Cargo.toml（判定库服务），这一格无对象可判"; exit 77; }
scratch="$(mktemp -d "${TMPDIR:-/tmp}/code-tooling-prune-server-selftest.XXXXXX")" || { echo "  ✗ 建不了这一格的临时目录"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。"; exit 1; }
# shellcheck disable=SC2064  # 路径在这里就展开（%q 过）
trap "rm -rf -- $(printf '%q' "$scratch")" EXIT   # 这一格的编译目录、自证的库与 socket 与输出，收尾删
mkdir "$scratch/tmp" || { echo "  ✗ 建不了 $scratch/tmp"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。"; exit 1; }
if judged_root_is_sample && [[ -f .prune-server-selftest-program ]]; then
  program=(python3 "$ROOT/.prune-server-selftest-program")
else
  if ! python3 "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" entry --root "$ROOT" >"$scratch/rocksdb-entry.txt" 2>&1; then
    sed 's/^/     /' "$scratch/rocksdb-entry.txt" | tail -n 8   # gate-lint:detail
    echo "  ✗ 编 prune-server 之前 RocksDB 预编库的取值没判过（上面是 rocksdb-prebuilt.py entry 的输出尾部），这一格没编、没跑自证"
    echo "     → 怎么办：照上面那几行的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再跑这一格。"
    exit 1
  fi
  # 这一格判的是编得出：不认环境里的 PRUNE_SERVER_BINARY（置空），照源码编；编的是本趟共用的那一份（code_tooling_prune_server_binary：别的格先编过就直接用，编译日志在共用目录里）
  if ! program_binary="$(PRUNE_SERVER_BINARY="" code_tooling_prune_server_binary "$scratch" 2>"$scratch/build.err")"; then
    build_log="$scratch/prune-server-build.log"
    [[ -f "${CODE_TOOLING_RUN_SCRATCH:-}/prune-server-build/prune-server-build.log" ]] && build_log="$CODE_TOOLING_RUN_SCRATCH/prune-server-build/prune-server-build.log"
    [[ -f "$build_log" ]] || { cp -- "$scratch/build.err" "$scratch/prune-server-build.log" && build_log="$scratch/prune-server-build.log"; }
    kept_build_log="$(mktemp "${TMPDIR:-/tmp}/code-tooling-prune-server-selftest-build.XXXXXX.log")" && cp -- "$build_log" "$kept_build_log" || kept_build_log="（没留下：拷不出来）"
    echo "  ✗ 编不出 prune-server（cargo build --release --manifest-path prune-server/Cargo.toml --bin prune-server，日志留在 $kept_build_log）：$(tail -n 3 "$build_log" | tr '\n' ' ' | clip_text 300)"
    echo "     → 怎么办：看那份日志，看完自己删；编译错照报错修 prune-server/（归工具实现员），别的会话正改着的就等它改完再跑这一格。"
    exit 1
  fi
  # 拷到这一格自己的临时目录里再起：命令位置上的词要是闭包折得出的写法（「$目录/<不带脚本后缀的字面尾巴>」），直接起 "$program_binary" 这一格的键算不出
  cp -- "$program_binary" "$scratch/prune-server-under-test" || { echo "  ✗ 拷不出编好的 prune-server（$program_binary）"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  program=("$scratch/prune-server-under-test")
fi
if (cd "$ROOT/prune-server" 2>/dev/null || cd "$ROOT" || { echo "进不去 $ROOT，自证一格都没起" >&2; exit 2; }; TMPDIR="$scratch/tmp" "${program[@]}" --selftest) >"$scratch/selftest.txt" 2>"$scratch/selftest.err"; then
  run_exit=0
else
  run_exit=$?
fi
python3 - "$scratch/selftest.txt" "$scratch/selftest.err" "$run_exit" "${CODE_TOOLING_BREAK:-}" <<'PY'
import sys
output_file, error_file, run_exit, broken = sys.argv[1:5]
PREFIX = "PRUNE_SERVER_SELFTEST"
lines = [line for line in open(output_file, encoding="utf-8", errors="replace").read().split("\n") if line.strip()]
summaries = [line for line in lines if line.startswith(PREFIX + " ")]
red_lines = [line.strip() for line in lines if line.lstrip().startswith("✗")]
faults = []
fields = {}
if (len(summaries) != 1 or lines[-1] != summaries[0]) and broken != "prune-server-last-line-unchecked":
    faults.append(f"末行不是恰好一行 {PREFIX}（实有 {len(summaries)} 行，最后一行是「{lines[-1].strip()[:120] if lines else '（没有输出）'}」）")
if summaries:
    fields = dict(part.split("=", 1) for part in summaries[-1].split()[1:] if "=" in part)
cells, passed, failed = (int(fields[name]) if fields.get(name, "").isdigit() else None for name in ("cells", "passed", "failed"))
failed_cells = [] if fields.get("failed_cells", "-") == "-" else fields["failed_cells"].split(",")
if summaries and (None in (cells, passed, failed) or "failed_cells" not in fields):
    faults.append(f"末行缺 cells=、passed=、failed=、failed_cells= 之一，或前三个不是整数：{summaries[-1][:200]}")
elif summaries:
    if broken == "prune-server-failed-ignored":
        failed, failed_cells, run_exit = 0, [], "0"
        passed = cells
    if cells == 0 and broken != "prune-server-empty-run-accepted":
        faults.append("末行 cells=0：一格都没跑")
    if passed + failed != cells:
        faults.append(f"末行 passed={passed} 加 failed={failed} 不等于 cells={cells}")
    if failed != len(failed_cells):
        faults.append(f"末行 failed={failed}，failed_cells 却列了 {len(failed_cells)} 格")
if broken != "prune-server-exit-unchecked":
    if run_exit not in ("0", "1"):
        tail = " | ".join(open(error_file, encoding="utf-8", errors="replace").read().strip().split("\n")[-3:])
        faults.append(f"prune-server --selftest 退 {run_exit}（格名或开关写错、起不来或没跑成：{tail[:300]}）")
    elif summaries and failed is not None:
        if run_exit == "0" and failed:
            faults.append(f"退 0，末行却报 failed={failed}")
        elif run_exit == "1" and not failed:
            faults.append("退 1，末行却报 failed=0")
if failed_cells or faults or (failed and failed > 0):
    for cell in failed_cells:
        print(f"  ✗ prune-server 自证的格 {cell} 红了")  # gate-lint:detail
    for line in red_lines[:20]:
        print(f"     {line[:300]}")  # gate-lint:detail
    for fault in faults:
        print(f"  ✗ prune-server --selftest 的输出：{fault}")  # gate-lint:detail
    print(f"  ✗ prune-server 自证没过：{cells if cells is not None else '?'} 格里红 {failed if failed is not None else '?'} 格（{'、'.join(failed_cells) or '无'}）、{len(faults)} 处输出不对（退 {run_exit}）")  # gate-lint:summary
    print("     → 怎么办：单跑看细节：task-server/task submit --cwd <仓根>/prune-server -- cargo run --release --offline --bin prune-server -- --selftest；"
          "只跑红的格：同一条命令改成 task-server/task submit --cwd <仓根>/prune-server -- env PRUNE_SERVER_SELFTEST_CELLS=<格名> cargo run --release --offline --bin prune-server -- --selftest；"
          "格的写法与弄坏开关在 prune-server/src/bin/prune_server.rs 文件头，判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ prune-server 自证：{cells} 格都过（{summaries[-1] if summaries else '无末行'}，退 {run_exit}）")
PY
}
