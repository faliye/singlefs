#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 起不了 user systemd scope 时它自己退 251，调用方（checker-tier-crates-mutation-replay、check.sh 的前缀）按这个码分流；这里拒绝退 78 会让它们认错结局
# 给一条命令定内存上限，起跑之前先判整机放不放得下、放不下就排队（records/2026-09-16-subagent拆分提案.md 第四十节第 28、30 行）。三层：
#   ① 这一条的上限：放进 systemd 的临时 scope（MemoryMax=<上限>、MemorySwapMax=0、OOMPolicy=stop），撞上限只杀这个 scope 里的进程；
#   ② 总量兜底：每个 scope 都挂进同一个 slice（默认 singlefs-heavy.slice），slice 的 MemoryMax = 整机内存 − 余量，MemoryHigh = MemoryMax 的 95%。几条合起来撞顶也只在 slice 里杀，
#      杀不到 slice 外面的本地模型服务与 Claude Code 会话；快撞顶时由超出记账最多的那一条的包装先停它自己（「盯账」一段）。slice 设不上就报错退出，不退回无总上限；
#   ③ 起跑前判内存量：拿一把锁，算「slice 里已占的 + 这一条按预测要的」，不超过总上限才起；放不下就放锁、隔一会儿再判，等满上限报「排不上」退出；
#      账上没有在跑的而放不下（没有哪一条跑完会放出来），不排队、当场报「排不上」退出。
#   只判「这台会不会 OOM」（用户 2026-10-06 定，records/2026-09-16-subagent拆分提案.md 第五十五节）：开跑前只在按预测这台放不下时排队、不为手续拒；
#   运行时认撞了这一条的上限、被总上限挤掉、超时与显存不够。选哪台机器、按历史预测要多少归统一派发器 scripts/hosts/host-dispatch.py。
# 变异跑道 scripts/mutation/mutate.sh 与门禁 checker-tier-crates-mutation-replay 每条变异都经它跑；子 agent 跑 cargo test / cargo run / 实验二进制也要经它
# （.claude/hooks/heavy-test-guard.sh 在执行前拒不经它的）。提交时的重阶段整条经它跑，例：
#   SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh 16G bash <根>/.claude/gate.d/checker-tier-crash-point-replay.sh --full <根>
#
#   run-with-memory-cap.sh <上限> <命令> [参数…]   排队，在上限里跑那条命令；退出码见下
#   run-with-memory-cap.sh auto <命令> [参数…]     上限取峰值表里这条命令实测峰值的 1.5 倍（按 MiB 向上取整）；表里没有这条取保守默认值
#                                                  （scripts/hosts/memory_peaks_lib.py 的 DEFAULT_MEMORY_PREDICTION_TEXT，8G），stderr 报一行
#   run-with-memory-cap.sh --check <上限>          只试 slice 的总上限设不设得上、带同样上限的 scope 起不起得来（在 slice 里跑 true，不排队）；起不来退 251；
#                                                  slice 上有余量不同的别的调用者时照已有的总上限试、不改 slice、不打「照 slice … 已有的总上限」那一行
#   run-with-memory-cap.sh --status                现量：整机内存、余量、slice 总上限与已占、账上在跑的每一条、slice 外面的用量
#   run-with-memory-cap.sh --selftest              自证（每一例在自己的临时 slice 里，总上限几百 M，吃不满机器）
#
# 上限照 systemd 的 MemoryMax 写法：正整数加 K / M / G / T（1024 进制），例 16G、512M；单位必写，不带单位的「16」在 systemd 那里是 16 字节，这里按用法错拒绝。默认值由调用方定，写在调用方的脚本头。
# 退出码：
#   0–244、246–249  那条命令自己的退出码（cargo 只退 0、101、126、127 与 128 + 信号号，碰不到 245 与 250–254）；包装自己的结局一个都不落在这一段，用法错也不落（退 251）；
#                   只有一处例外：平常跑设了只给自证用的 RUN_WITH_MEMORY_CAP_BREAK，包装退 2、命令一行都没跑（文件头「弄坏开关只在自证里认」一段）；
#   245    显存不够：命令退了非 0，而它起止之间的内核日志里有显卡驱动报的显存不够（NVRM … NV_ERR_NO_MEMORY，文件头「显存不够」一段）；那一次的输出不算结果。
#          命令退 0 的不查、照退 0；内核日志查不成的照退命令自己的码，另报一行「显存 OOM 没查成」；
#   250    撞了这一条自己的上限：scope 的 Result 是 oom-kill，scope 自己的 memory.events 里 oom 不为 0（或读不到）。
#          只认 Result、不认退出码：被杀的是 cargo 起的测试二进制时 cargo 退 101 或被停时退 143，被杀的是命令本身时退 137，外面 kill -9 同样退 137；
#   251    用法错（上限、余量、总上限不是 正整数加 K / M / G / T 的写法或没写单位，限时、等待上限不是整数秒，slice 名不是 singlefs 开头，没给命令，
#          套在外层包装里而这一条的上限比外层还大：文件头「套在包装里」一段），
#          slice 的总上限设不上（读回不是正整数：max、读不到。读回是别的调用者设的另一个正整数，或 slice 上有活着的 scope / 账而总上限与自己算的不同，不算设不上，照已有的值走，退不了 251：
#          「余量」一段「并发调用者」）、systemd-run 起不来（没有用户级 systemd、D-Bus 连不上、上限写法 systemd 不认）、账与锁的目录建不了，或查不到 scope 的结局：
#          那条命令一行都没跑（或结局判不了），不退回无上限、无总上限去跑——那正是要防的事；
#   252    内存不够排不上：等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 秒还放不下，或这一条要的量比总上限还大、或账上没有在跑的而这一条要的量加 slice 里回收不掉的用量已超总上限（这两种等多久都放不下，不等）。命令一行都没跑；退之前现查别的机器放不放得下这一条（「排队时去找别的机器」一段）；
#   253    超过限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT）：scope 的 Result 是 timeout，systemd 停掉了 scope 里的进程；
#   254    被总上限挤掉：slice 满了或快满了，这一条被停，不是它自己撞了它的上限；它的结果不算数，重跑它。两种来路，退出码相同、stderr 与告警记录分得开：
#          包装盯账停的（「盯账」一段）：slice 回收不掉的用量到了 MemoryHigh，这一条超出记账最多、超出量过了容差，包装在撞总上限之前往它的 scope 写 cgroup.kill；
#            stderr 报「超出记账最多…包装在撞总上限之前停了它」，告警记录写 kind=memory-squeezed、stopped_by=over-booking-watch，这个键记成撞过顶，重跑时按它的上限排队；
#          内核挤掉的：scope 的 Result 是 oom-kill，而 scope 自己的 oom 计数是 0、oom_kill 不为 0——slice 撞了顶，内核在 slice 里挑了这一条杀（告警记录 stopped_by=kernel）；
#            内核按超出记账的量挑（「排队」一段的 oom_score_adj），只有峰值超出记账过了容差的才把这个键记成撞过顶，没超账而被挑中的不记。
#
# 余量（RUN_WITH_MEMORY_CAP_RESERVE，默认 20G）留给 slice 外面：本地模型服务、各个 Claude Code 会话与它们起的工具、门禁在包装外面的进程、内核。
#   2026-09-25 量的（只在本机成立；换机器、本地模型服务换模型或换并行度都要重量）：MemTotal 60.1 GiB；slice 外面回收不掉的合计 13.4 GiB
#   （MemTotal − MemAvailable，那时 slice 里几乎是空的），其中本地模型服务 7.4 GiB anon + 0.6 GiB shmem（它 cgroup 的 memory.stat；
#   systemctl status 报它的 Memory 37.9G，其中 30.3 GiB 是模型文件的页缓存，回收得掉），user.slice 2.4 GiB anon（Claude Code 会话与它们起的进程），
#   其余约 3 GiB 是内核（Slab 1.9 GiB、页表）与别的服务。20G 比 13.4 GiB 多约 6.6 GiB，给会话变多、本地模型服务在负载下涨、门禁在包装外面的 python 进程留着；
#   本机 slice 总上限因此约 40 GiB。怎么现量：bash scripts/hosts/run-with-memory-cap.sh --status 打出上面每一项此刻的数。
#   本地模型服务的 cgroup 不写进仓（单元名是本机的私有信息）：--status 从仓外的私有配置读，文件是 RUN_WITH_MEMORY_CAP_PRIVATE_CONFIG，
#   默认 ${XDG_CONFIG_HOME:-~/.config}/singlefs/run-with-memory-cap.env；一行一条「键=值」，# 起头的行与空行不看，不 source。
#   读的键是 OUTSIDE_SERVICE_CGROUP=<它的 cgroup，相对 cgroup 根，形如 system.slice/<单元名>.service>，读那里 memory.stat 的 anon + shmem。
#   文件不在或没有这个键，--status 里这一项报「没配」、其余照算、退出码不受影响；配了而读不到 memory.stat 报「读不到」。
#   这一项只进 --status 的明细，排队与总上限都不用它。--status 不打印配的值。
#   slice 外面涨过了余量，总上限挡不住那一部分，整机照样可能被外面涨满：那一类靠看门狗的常驻内存告警与 .claude/hooks/session-start.sh 的 OOM 报告。
#
# 并发调用者（checks-owed C787）：总上限 = 整机内存 − 余量，调用者的 RUN_WITH_MEMORY_CAP_RESERVE 不同（例：轻型变异复跑给 10G、全量那一份用默认 20G）就算出不同的总上限，
#   而 slice 只有一个 MemoryMax。起 scope 之前（ensure_slice）先看 slice 此刻的 MemoryMax：
#   与自己算的相同，不动；不同而 slice 上有活着的使用者（它底下还在的 *.scope 的 cgroup 目录，加账上包装还活着的每一笔，刚放行、scope 还没起的也算）、MemoryMax 是正整数，
#   照已有的值走：不去改 slice 的 MemoryMax，这一趟的准入、排队、「要的量比总上限还大」的判都按它算，往 stderr 打一行「run-with-memory-cap: 照 slice … 已有的总上限 … 字节走（自己按余量 … 算的是 … 字节…）…；来自：…」，
#   写明自己算的、已有的、来自哪个 scope / 账上哪一笔（单元名、包装进程号、会话、上限）；不退 251。
#   slice 上没有活着的使用者：照旧按自己的余量设。设完读回的是别的调用者刚设的另一个正整数（两个调用者在设与读回之间交错），同样照读回的走并打同一行；读回是 max 或读不到才是控制器没下放，退 251。
#   --check（只试上限设不设得上，调用方如 scripts/mutation/mutate.sh 开跑先调一次）也照已有的值试、不改 slice，但不打这一行：说明只在真要跑命令的那一支打，
#   免得它占了调用方自己 stderr 的头一行。
#   照已有的值走的代价：这一趟的总上限可能比自己的余量算出来的小（排队等得久）或大（余量留得少），由先设的那一边的余量定；同时在跑的几条用同一个总上限，账才对得上。
#
# 排队（第 ③ 层）怎么算，判与记账在同一把锁（${状态目录}/lock，flock）里，几条同时来的一条一条判：
#   这一条要的量（记账按预测的实际峰值，不按声明的上限）：峰值表里这条命令（键见下）近几次实测里最大的峰值（「峰值表」一段「近几次」），不超过它的上限；
#     表里没有的按 scripts/hosts/host-resources.py 对这条命令的预测与它的上限取小（.claude/kb/milestone/03-third-txn.md「待优化」第 11 行：写 32G 的新命令照上限排队，
#     在约 40 GiB 的总上限里排着不起）。预测取的是 host-resources.py 逐台判用的那一份：scripts/hosts/host-dispatch.py 的 predict（按文件路径导入，不另写一份），
#     读这个包装的峰值表、键是这个包装的键；表里没有时它给 memory_peaks_lib.py 的 DEFAULT_MEMORY_PREDICTION_TEXT（8G）。预测算不出来（导入或调用出错）按它的上限算，
#     原因写进排队时报的那一句。scope 的 MemoryMax 照旧是写的上限，不跟着改：预测少算了，涨过去的那一截撞的是 slice 的总上限；
#     按预测放进来的（要的量小于上限）与按峰值表放进来的同样处置：oom_score_adj 按记账算、超出记账被停或被挤掉之后这个键记成撞过顶，下一次按上限排队。
#     几条按峰值表或预测放进来的，预测之和放得下就一起起、不排队。
#     这两种也按它这一次的上限算（不按峰值表）：设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP=<为什么>（广谱变异驱动升档重跑的那一趟这样起）；
#     这个键撞过顶（峰值表里这一行的近几次里有一次撞了顶：撞了自己的上限记成上限，被总上限挤掉的也记成上限，见「峰值表」一段）。
#     按上限算比总上限还大的当场退 252（等多久都放不下），放不下就照下面排队、等满退 252，不放进去再撞 slice 的总上限。
#   撞 slice 总上限时内核杀谁（盯账没赶上时的兜底，records/2026-10-23-subagent拆分提案.md 第四十八节 ⑩c，用户 2026-10-26 定「改」）：外壳把自己的 oom_score_adj 写好、命令随之继承，
#     内核按进程打分，在 slice 里挑「常驻页 + 页表页 + oom_score_adj × 分母页数 / 1000」最大的那个进程杀；分母 = slice 的 memory.max + min(slice 的 memory.swap.max, SwapTotal)。
#     分母里的 swap 那一项是 2026-10-26 在 400M 的小 slice 上量的：两个进程常驻差约 23,200 页，slice 的 memory.swap.max 是 max 时 adj 差 10 与 11 之间翻面，
#     设成 0 之后在 200 与 260 之间翻面，分别落在分母含 swap（每点约 2,200 页）与不含 swap（每点约 102 页）的预测上；memory.low、memory.min 与 cgroup 的记账量都不进这个分数。
#     账上有记账的（按峰值表、按预测、按上限放进来的都算）写 oom_score_adj = 1000 − 记账 × 1000 / 分母（字节算，起 scope 之前现读，小于 0 取 0），
#     代进去分数 = 常驻 − 记账 + 分母，内核挑的就是常驻超出自己记账最多的那个进程；设了 RUN_WITH_MEMORY_CAP_OOM_FIRST=<为什么>（广谱变异驱动起变异那几步时这样起：
#     一条无界分配的变异）的写 1000，比任何一条超出记账的都先挨杀；套在外层里的里层一条不写，照旧继承外层那一条的。
#     已知的局限：内核按进程打分，一条 scope 里几个进程分摊了超账（并行的 rustc 这类），每个进程都要减掉整条 scope 的记账，内核仍可能挑没超账的那一条；
#     adj 是整数，本机正式 slice 上 1 点约 49 MiB（按上面的式子算的，没在正式 slice 上量），超账量差不到这么多的两条分不清。这两样由盯账接住（「盯账」一段），这里只是盯账没赶上时的次序。
#   已占 = slice 里回收不掉的用量（memory.current 减 active_file 与 inactive_file：页缓存回收得掉，而结束了的 scope 的页缓存挂回 slice、一直算在 slice 的
#          memory.current 里；2026-09-25 实测 scope 里 dd 写 120 MiB 退出之后，slice 的 memory.current 还是 131 MiB，其中 inactive_file 126 MiB）
#        + 账上每条放行了还在跑的 max(0, 它要的量 − 它的 scope 此刻回收不掉的用量)：刚起跑的 cargo 还没涨到量，只看 slice 的用量，前后脚来的两条都判得下、合起来超。
#   已占 + 这一条要的量 ≤ 总上限就放行，在账上记一笔（${状态目录}/ledger/<unit>.json：包装的进程号与起始时刻、要的量、上限、键），scope 收尾之后删掉；
#   放行那一刻读不到包装的起始时刻（包装已经不在）就不记、不放行，排队那一步退出。
#   包装被 KILL、没删掉的那一笔，下一次判的时候按进程号与起始时刻认出来删掉：此刻读不到它的起始时刻、账上记的起始时刻是空的、两者对不上，都算死账。
#   放不下而账上没有在跑的：已占的全是 slice 里回收不掉的用量，没有哪一条跑完会放出来，等多久都放不下；当场退 252、不排队，出路写这一条要多少、
#   此刻放得下的最多多少（总上限减回收不掉的用量）。账上有在跑的才排队（records/2026-10-01-对接上游测试脚本规范.md）。
#   账上有在跑的、但这一条要的量加 slice 里不属于任何 scope 的常驻用量（memory_peaks_lib.py 的 resident_bytes：slice 回收不掉的用量减去它底下每个 scope 回收不掉的用量）
#   已超总上限：账上在跑的与别的 scope 都跑完，这一截也不会少，等多久都放不下；同样当场退 252、不排队，出路写这一条要多少、放得下的最多多少（总上限减常驻）。
#   （.claude/kb/milestone/03-third-txn.md「待优化」第 19 行：第二台总上限 50.2G、空闲时 slice 常驻约 640M，按上限 50G 记账的一条会白排满 3600 秒才退 252。）
#   要的量取峰值表里量过的峰值与写的上限中较小的那个（上面「这一条要的量」）：峰值表里记着 1.8G 的命令写 24G 上限，账上记 1.8G、不记 24G。
#   放不下就放锁、隔 ADMISSION_POLL_SECONDS 秒再判，第一次等与之后每 WAITING_REPORT_EVERY_SECONDS 秒往 stderr 报一行在等什么；
#   等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS（默认 3600）秒还放不下，列出账上占着的每一条，退 252。默认 3600 秒的依据：门禁 checker-tier-crates-mutation-replay 一条变异限时 1800 秒，
#   排在后面的一般等前面一条跑完就轮到；再宽一倍，给别的重活占着 slice 的时候。
#   排队时包装被 TERM / INT 停（只停包装进程本身、不停它的进程组也一样）：排队那一步的子进程跟着停、调用方的管道随之关上，账上不留这一笔，
#   命令没跑，退 143 / 130（stop_queued_admission）。
#   排队时包装被 KILL / HUP 停：排队那一步每轮重判之前先核自己的父进程还是不是包装，不是就退出、不记账，调用方的管道随之关上，至多晚一个重判间隔。
#   HUP 不设 trap：这一核照样接得住；设了还得在起 scope 时换掉，改动就伸进放行之后那一段，省下的只是一个重判间隔。
#   起了 scope 之后包装被 TERM / INT 停（欠账 C801；调用方是 scripts/crash/crash-amplification-lib.sh 这类把 TERM 只发给包装一个进程号的）：
#   先 systemctl --user stop 自己开的那个 scope 单元（$unit.scope），scope 里的命令随之收到 TERM、停下；再删标记、账与峰值文件，退 143 / 130（stop_own_scope_then_exit）。
#   起 scope 的 systemd-run 放后台、用 wait 等，所以 trap 当场就跑，不等命令退出；命令读的标准输入照旧是调用方的（<&0），INT 照旧不被忽略（env --default-signal=INT；调用方进来时就忽略 INT 的照旧忽略）。
#   scope 还没建出来（systemd-run 还在起，stop 找不到单元）：改给 systemd-run 那一个进程发 TERM。
#
# 盯账（records/2026-10-23-subagent拆分提案.md 第四十八节 ⑩c，用户 2026-10-26 定「改」）：几条按峰值表放进来的合起来撞总上限时，只停超出记账的那一条，不让内核按常驻挑中没超账、常驻更大的那一条
#   （改之前 2026-10-26 在 400M 的小 slice 上 6 次里 6 次如此：没超账的那一条退 254、还被记成撞过顶，超账的那一条退 0）。三样合起来：
#   ① 账上有记账的每一条，起 scope 之后包装另起一个盯账的子进程（watch_over_booking，python，读法复用 memory_peaks_lib.py 的 unreclaimable_bytes 与 live_ledger_entries），
#     每 OVER_BOOKING_POLL_SECONDS（0.05）秒看一次：slice 回收不掉的用量（与排队的「已占」同一个定义）到了 slice 的 memory.max 的 SLICE_HIGH_PERMILLE（950）‰（触发线），
#     就算账上每一条此刻超出记账多少（它的 scope 回收不掉的用量 − 它要的量）；自己超出的量大于 OVER_BOOKING_TOLERANCE_BYTES（16 MiB）、又是账上最多的（一样多的按单元名小的算最多），
#     先把此刻的数写进标记文件（${状态目录}/peaks/<unit>.over-booking），再往自己的 scope 的 cgroup.kill 写 1。只动自己那一条，别的包装各管各的；写标记与写 cgroup.kill 那一段里 TERM 挡着，
#     包装收尾时停它不会只写了一半。命令退出、包装被停或换了父进程，它在下一轮自己退。
#     容差 16 MiB：2026-10-26 在 400M 的小 slice 上两条都不超账、合计压过触发线时，没超账那一条此刻用量离记账只差约 1 MiB（记账取的峰值与此刻用量差一点噪声就翻正）；正式 slice 上的噪声没量过。
#   ② slice 的 memory.high 设成触发线（ensure_slice_high，与 MemoryMax 同一处设，照已有总上限走的那一支也设）：越过它的分配被内核节流，盯账赶得上一口气拿满的那种分配
#     （2026-10-26 实测：不设它，一口气拿满的分配 5 次里 5 次抢在盯账前面撞了 memory.max、内核杀的是没超账的那一条；设了 5 次都由盯账停对，内核一次都没 OOM）。
#     代价：越过触发线之后 slice 里正在分配的命令一律被节流，没超账的也一样。2026-10-26 在 400M 的小 slice 上量的：两条都不超账、合计越过 memory.high 约 10 MiB 时，
#     后一条拿 140 MiB 用了 1.08–1.49 秒（独自拿 0.031 秒），slice 的 memory.events 里 high 计了 530–632 次（三趟）；正式 slice 与真实负载上没量过。
#   ③ oom_score_adj 按记账算（「排队」一段「撞 slice 总上限时内核杀谁」）：盯账没赶上时内核按超出记账的量挑。
#   包装认标记：命令退出之后标记在，就是盯账停的这一条，退 254（文件头「退出码」），这个键记成撞过顶（峰值记成上限），告警记录 kind=memory-squeezed、stopped_by=over-booking-watch，
#   量到的峰值取标记里记的 scope 的 memory.peak（外壳随 cgroup.kill 一起没了，写不出峰值文件）；不落到「被外面停了（信号 9）」那一支。
#   2026-10-26 在 400M 的小 slice 上量过的四种场景（单进程超账、同一条 scope 里 8 个进程分摊超账、一口气拿满、两条都不超账而合计压过触发线）各 5 次都只停了超账那一条或谁都没停；
#   正式 slice（约 40 GiB）与真实负载上没量过。已知的局限：套在外层里的里层一条不记账，它的 scope 不在账上，盯账算不到它头上，它涨过头只由内核兜底。
#
# 排队时去找别的机器（用户 2026-10-20「提醒一下 去找一下可用的电脑」，records/2026-09-16-subagent拆分提案.md 第七十五节）：第一次报「排队」那一刻与退 252 之前
#   （等满、要的量比总上限还大、账上没有在跑的而放不下、常驻加要的量超总上限，四处都算），现查别的机器此刻放不放得下这一条。
#   判法是 scripts/hosts/host-resources.py --json -- <这条命令>（键照传 RUN_WITH_MEMORY_CAP_KEY；逐台判是 host-dispatch.py 的 predict 与 judge_host，这里不另写一份「哪台放得下」），
#   只看多机配置里的第二台（PEER_SSH_HOST 那一台：multi-host-run.sh --peer 只去得了它）。
#   放得下：往 stderr 打一行「  → 第二台此刻放得下（可用 X、要 Y）：改用 bash scripts/hosts/multi-host-run.sh --peer --memory-cap <上限> --label <名> -- <这条命令>」
#     （X 是第二台此刻可用的内存，Y 是 host-resources.py 对这条命令的预测；上限是这一条的，比第二台的 PEER_MEMORY_CAP 大就照后者写），
#     命令里以本机仓根（这份包装所在的树，符号链接解开前后两种写法都认）起头的参数改写成树内相对路径、仓根本身写 .（第二台那棵树不在本机仓根那个路径上，
#     命令在那棵树的树根起；multi-host-run.sh --peer 见到这类参数会拒，退 2）：整词是仓根或以「仓根/」起头的才改，仓根之外的绝对路径与名字只是以仓根起头的路径（<仓根>-旁边）原样留着；
#     告警记录的 command 一栏照旧是本机跑的原命令，出路（next_step）与那一行里是改写后的，
#     再往告警记录（scripts/hosts/core_usage.py 文件头「告警记录」：同一种写法，经它的 append_line 写；写到哪照「峰值表」一段「告警跟着峰值表走」）追加一条：
#     kind=memory-queue-peer-fits、phase=queued 或 refused、会话号、告警进程号（这个包装）、命令、这一条的上限、本机的数（要的量、slice 已占或回收不掉的、总上限）、
#     第二台的数（可用、要的、上限）、那一行原文与出路。scripts/agents/agent-watch.py 读它叫醒主 agent（「本机内存排队、别的机器放得下」）：
#     queued 那条要包装还活着，refused 那条包装退了也报（core_usage.py 的新鲜窗口内）；排了队之后在本机放行的补一条 phase=admitted，看门狗见了就不再报。
#   放不下：打一行「现查别的机器：第二台此刻也放不下（…）」，照旧排队、退 252。
#   查不到（多机配置没开或判不过、第二台连不上、超时、host-resources.py 退别的码或输出读不出）：照实打一行「没查到别的机器：<原因>」。
#   不改排队本身：不自动改去第二台（两台结论口径不一样时混用分不清），退出码照旧。
#   不拖慢准入：排队开始那一趟放到后台线程查，排队照旧每 ADMISSION_POLL_SECONDS 秒重判，放行时停掉在查的那一趟（只停自己起的 host-resources.py 那一个进程号，
#   它起的 ssh 由 ssh 自己收尾）；排队时包装被 TERM 停，排队那一步同样先停它再退。退 252 之前那一趟在前台查（先等排队开始那一趟查完），每趟至多 HOST_LOOKUP_SECONDS 秒。
#   HOST_LOOKUP_SECONDS 15：host-resources.py 读第二台实测两三秒（它文件头「常量的依据」）；这里给它的每台时限 HOST_LOOKUP_PEER_TIMEOUT_SECONDS 8 秒、
#     采样间隔 HOST_LOOKUP_SAMPLE_SECONDS 0.2 秒（只看内存，不为核占用等满一秒），再加本机那一份与 python 起动；推的，没量过忙时。
#     退 252 那一趟最多多等这么久，比起排满的 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 可以不计；排队开始那一趟在后台，不占准入的时间。
#   同一台机器上几条一起排队（变异门禁一批几十条）时每 HOST_LOOKUP_EVERY_SECONDS（60）秒只查一次：免得几十条一齐 ssh 第二台、一齐叫醒主 agent；
#     轮不到的打「没查到别的机器：N 秒前包装 <进程号> 刚查过」。谁上一次查的记在状态目录的 host-lookup.json（排队那把锁里读写）。
#   RUN_WITH_MEMORY_CAP_HOST_LOOKUP 设成任何非空值（例 off）这一条不查，只打一行「没查到别的机器：RUN_WITH_MEMORY_CAP_HOST_LOOKUP=…」；自证里除了这几例都这样设。
#   RUN_WITH_MEMORY_CAP_HOST_LOOKUP_SECONDS 换掉 HOST_LOOKUP_SECONDS（只给自证）。
#   整体兜底（research/prompts/m3-close-reuse-closure-r3-main-verification.md 末节「用户定案」第 1 条）：HostLookup 的 begin、finish、run、query、cancel 整段包在 try 里，
#     出了任何异常（起线程、建临时文件、读输出、拼那一行……）只打一行「没查到别的机器：<哪一趟>出了 <异常>…」，排队、放行与退出码照旧；被 TERM 停的 SystemExit 照旧往外走。
#     会卡住的两处不靠 try：排队锁在这里不阻塞地取（LOCK_NB，至多试 HOST_LOOKUP_LOCK_SECONDS 0.5 秒，避开排队那一步每轮占锁的那一瞬），取不到这一趟不查，
#     打「没查到别的机器：排队锁 … 被别的进程占着…」；finish、cancel 等排队开始那一趟收尾至多等它的时限加 HOST_LOOKUP_JOIN_MARGIN_SECONDS（2）秒，
#     过了打「没查到别的机器：排队开始那一趟 N 秒还没查完，不再等它」、不再等（daemon 线程，包装退出时随之没了）。
#     这几行只出提示、不改结局：判法读包装输出前去掉它们（scripts/gate/verus-proof-check.py、scripts/experiment-apparatus/e165-pin-red-examples.py、
#     scripts/mutation/crates-mutation-rows.py 各一份的 HOST_LOOKUP_LINE，只认 HostLookup 打的四种行首），判法单元的闭包也不走 HostLookup.query
#     起 host-resources.py 那一条边（scripts/gate/lib_function_units.py 的 EXCLUDED_PROGRAM_EDGES）。HostLookup 里新加一处 say 而那三份正则不认它，三份的自证各红一格。
#     自证（只跑这几项照旧是 RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=host-lookup）六格各造一处出事：四处异常由只给自证的 RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_FAULT 在真世界会抛的那一处抛；
#     「锁被别的进程占着」起一个真占着排队锁的进程（RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_LOCK_HANDOFF 只管叫它来占、等它占上），「线程里的 query 睡着」让后台那一趟真睡
#     （RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_SLEEP），两格都是真卡住、不抛异常；每格包装的退出码与关掉 HostLookup 的那一臂逐个相同、30 秒内退。
#
# 资源释放广播（用户 2026-10-23 定「叫醒并重判，不自动挪」，records/2026-09-16-subagent拆分提案.md 第九十九节；发与收都在 scripts/hosts/resource_release_broadcast.py）：
#   发：命令真跑过之后（scope 里的外壳起来过：命令正常退出、退非 0、撞了这一条的上限 250、超时 253、被总上限挤掉 254、查不到结局 251、起了 scope 之后包装被 TERM / INT 停，都算），
#     包装退出那一刻（EXIT trap 的 on_wrapper_exit 调 broadcast_memory_released）发一条 memory-released：标签是这一条的 scope 单元名，带包装的退出码与这一条的峰值
#     （没量到取账上记的要的量）。排队没放行就退的（252、排队时被停、用法错、scope 起不来）不发：它没占过资源。--check、--status、--selftest 不发。
#     套在外层包装里的不发：外层账上那一笔照旧占着，里层跑完放不出排队的余量。账与锁的目录不是默认那一份（设了 RUN_WITH_MEMORY_CAP_STATE_DIR：自证与别的脚本的假世界）
#     不发，除非显式给了 RESOURCE_BROADCAST_DIRECTORY（事件说的是这台机器共用的那一份账）。
#     推不出去的那几行写进本机事件目录的 push-problems.log（emit --problems-to-log），广播那一步的输出一行都不进包装的 stderr：判法读包装的 stderr。
#     广播那一步出了什么事都不改包装的退出码（EXIT trap 里不调 exit），至多晚退那份库的 PUSH_SECONDS（4 秒）加起 python 与读配置的零点几秒。
#   收：排队开始那一刻（第一次报「排队」）HostLookup.watch_events 起一个那份库的 follow --since <这一刻>（只停自己起的这一个；follow 起得慢也不漏掉起动那一会儿来的），排队每一轮的等（ADMISSION_POLL_SECONDS）
#     换成「新事件或到点」先到者（HostLookup.pause）；收到别的机器推来的（source=pushed）或第二台那一趟跑完的（kind=peer-run-finished），当场在后台
#     再跑一次上面「排队时去找别的机器」那一套查法（HostLookup.wake）；本机自己发的内存放出、卡放出、脱离单元结束不另查：第二台的余量不因它们变，
#     本机放出的照旧由本机准入每 ADMISSION_POLL_SECONDS 秒重判接住。放得下照旧打改去第二台的那一行、写 kind=memory-queue-peer-fits、phase=queued，
#     几种行尾都注「（被资源释放广播叫醒：第 N 号，<种类>）」。不改排队本身：不自动改去第二台，退出码照旧。
#     去重：几条一起排队时一个事件只查一次：查了第几号事件记在 host-lookup.json 的 event（排队那把锁里读写，与 60 秒那一条共用这一份）；
#     轮不到的打「没查到别的机器：第 N 号资源释放广播包装 <进程号> 已经查过…」。叫醒时排队开始那一趟或上一次叫醒那一趟还在查的，记下最新的号，查完再查。
#     叫醒那一趟与 follow 出的任何异常照「整体兜底」只打一行「没查到别的机器：…」，排队、放行与退出码照旧；follow 起不来或退了，排队照旧每 ADMISSION_POLL_SECONDS 秒重判。
#     RUN_WITH_MEMORY_CAP_HOST_LOOKUP 设了（这一条不查别的机器）不起 follow。只给自证：RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_FAULT=wake-thread 在叫醒那一趟起线程时抛。
#   这两条子进程边（broadcast_memory_released 与 HostLookup.watch_events 起 resource_release_broadcast.py）只出提示、不改结局，判法单元的闭包不走它们
#     （scripts/gate/lib_function_units.py 的 EXCLUDED_PROGRAM_EDGES）；自证 RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=broadcast 那几格核：那份库出事时包装的退出码照旧、
#     包装的 stderr 里没有广播那一步打的行。自证里读写事件只用 grep 与 printf，不起那份库（起了就是另一条要排的边）。
#
# 峰值表（RUN_WITH_MEMORY_CAP_PEAKS，默认本机的 ${XDG_DATA_HOME:-~/.local/share}/singlefs/memory-peaks.tsv；从 git worktree、第二台的副本里跑的也读写本机那一份）：一行一条命令，文件头的 # 行写口径，
#   由这个脚本建、读、写（写在同一把锁里，先写临时文件再改名；按 MiB 向上取整，与表里记的相同就不写）。口径：scope 里那个外壳 bash 在命令退出之后读自己 cgroup 的 memory.peak（字节）。
#   外壳不 exec 命令、留下来读，是因为 scope 一收尾它的 cgroup 就删了、systemd 也不留（2026-09-25 实测命令退出之后 MemoryPeak=[not set]）。
#   含页缓存（cargo 写编译产物的那些）与外壳 bash 自己，偏大不偏小。撞了这一条自己的上限记成上限；盯账停的、与按峰值表放进来（要的量小于上限）又被内核挤掉而峰值超出记账过了容差的也记成上限
#   （它要的比排队时按的多，下一次按上限排）；超时、被停、没超出记账而被挤掉的（照上限放进来的超不了账）不记（量到的是半截）；
#   命令被外面的信号停（退 128 加 HUP、INT、KILL、TERM 的信号号 1、2、9、15）而 scope 没撞自己的上限、也没被总上限挤掉的同样不记（量到的也是半截），
#   另报一句「被外面停了（信号 N），这一次的峰值不记」与出路，退出码照命令自己的。
#   放在仓外：数只在本机成立，是运行状态不是仓的内容；门禁 checker-tier-crates-mutation-replay 跑的时候每条都写它，放进工作区 gate.sh 开跑与收尾的工作区指纹就对不上。
#   键：RUN_WITH_MEMORY_CAP_KEY 给了就用它；不给就把命令的各个词用空格接起来（制表符、换行换成空格，测试二进制名里 -<16 位十六进制> 的哈希去掉，截到 300 字）。
#   取键、读表在 scripts/hosts/memory_peaks_lib.py（command_key、read_peak_table），统一派发器 scripts/hosts/host-dispatch.py 按同一份取键、预测。
#   线程数（第五列）：命令里经 capped.sh <N> 跑的记 N（它把线程变量一律设成 N，盖过外面的），经 capped.sh default 跑的记 heavy_thread_default 的值
#   （SINGLEFS_THREAD_CAP 设了照它，否则 .claude/heavy-thread-default 的登记值；取不到记「判不了」）；否则看包装自己环境里 capped.sh 设的那几个线程变量
#   （名单取 capped.sh --print）：设了的都相同记那个数，不同就逐个记「名=值」，一个都没设记「未设（整机 N 核）」。「量过的」上限只认线程数相同的那一行
#   （.claude/skills/crash-test/commit-stages.md「跑之前定下来」一节）；排队照旧只按键取峰值。加这一列之前记的行照原样留四列写回：只认四列的旧包装还在跑时读得到它们，
#   它收尾时写回的表里没有五列的行，那几条下一次按上限排队、重新记。
#   近几次（第六列）：这个键最近 RECENT_MEASUREMENTS（memory_peaks_lib.py，5）次记进表的实测，新的在前，逗号隔开，每次写成「<峰值字节>:hit」或「<峰值字节>:clean」
#   （hit 是撞了顶：撞了自己的上限、被盯账停了、或按峰值表放进来超出记账又被内核挤掉，峰值记成上限；clean 是整段跑完量到的）；前三列是最近那一次。
#   排队记账、auto、派发器 host-dispatch.py 的 predict 与 host-resources.py 都经 memory_peaks_lib.recorded_peak 读近几次：峰值取近几次里最大的，
#   近几次里有一次撞顶就算撞过顶、按上限排队。所以撞顶之后连跑 4 次干净的仍按上限排，第 5 次干净的跑完之后才按峰值排；峰值一路变小时记账取近几次里最大的，不跟着变小。
#   超时、被停、没超出记账而被挤掉的照旧不记，近几次不动。没有第六列的旧行（四列、五列）照读，当成只有一次：那一行的峰值，到了那一次的上限就算撞了顶。
#   加这一列之前就在跑的旧包装（收尾时读写的是四段：峰值、时刻、上限、线程数）写回时丢掉第六列：它写回的表里六列的行都只剩五列，
#   那几条的近几次又从只有一次记起（memory_peaks_lib.read_peak_table 照旧交四段，它读得到这几行）。没动到的四列、五列行照原样写回。
#   峰值涨了（工作量可能变了；键照旧是命令文本，代码改动让工作量变大时键不变，在结局上认）：
#   一次跑完（命令自己退出，不是 250、253、254，也不是被外面停了），量到的峰值是这一次排队时记的账的 CAP_FACTOR（memory_peaks_lib.py，1.5，
#   派发器的上限倍数读同一个）倍以上，而记账取自峰值表（账上那一笔的 need_source 是 peak-table；按 host-resources.py 的预测或默认值记的 prediction、按上限记的 cap 都不算）：
#   stderr 打一行「run-with-memory-cap: 这一次峰值 X，是记账 Y 的 Z 倍：工作量可能变了（…）」与一行出路，再往告警记录（scripts/hosts/core_usage.py 文件头「告警记录」，
#   经它的 append_line 写；写到哪见下面「告警跟着峰值表走」）追加一条 kind=memory-peak-grew：会话号、告警进程号（这个包装）、键、上限、记账与它从哪来、实测、倍数、那一行与出路。
#   撞顶与被挤掉：退 250（撞了自己的上限）与退 254（被总上限挤掉）各往同一份告警记录追加一条 kind=memory-own-cap 与 kind=memory-squeezed，
#   带会话号、告警进程号、键、上限、记账（套在外层里的不记账，写 null）与它从哪来、实测、是不是按峰值表放进来的（by_peak_table）；退出码不变。
#   memory-squeezed 另带 stopped_by：over-booking-watch 是盯账停的（「盯账」一段，实测取标记里记的 scope 峰值），kernel 是内核挤掉的；两种都是「结果不算数，重跑」，看门狗照同一种报。
#   设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条（有意撞顶的跑法：广谱变异驱动起无界分配的变异步）退 250、254 不写 memory-own-cap、memory-squeezed，
#   退出码、峰值表、撞顶标记与 stderr 照旧；它的 memory-peak-grew 照写。
#   scripts/agents/agent-watch.py 读这三种叫醒主 agent，包装退了也报（core_usage.py 的新鲜窗口内）。告警记录写不进只往 stderr 说一句，退出码照旧。
#   告警跟着峰值表走（这三种与「排队时去找别的机器」的 memory-queue-peer-fits 都照这一条）：显式设了 SINGLEFS_CORE_USAGE_ALERTS 的照它写；
#   RUN_WITH_MEMORY_CAP_PEAKS 指到默认峰值表（memory_peaks_lib.default_peak_table()）之外的（按真路径比：经符号链接指到默认那一份的算默认），
#   写进那份峰值表旁边的 <峰值表路径>.alerts.jsonl（内嵌 python 的 PEAK_TABLE_ALERTS_SUFFIX），不写工作根那份真记录：看门狗只读真记录，
#   拿临时峰值表跑的自证与别的脚本撞顶、挨挤不叫醒主 agent；没设、或指到默认那一份的写真记录（core_usage.alerts_path：工作根下的 core-usage-alerts.jsonl）。
#
# 上限对实测峰值（2026-10-06 撤了：用户定包装只判这台会不会 OOM，records/2026-09-16-subagent拆分提案.md 第五十五节）：上限超过峰值表里实测峰值 2 倍不再拒，
#   照给的上限跑、照预测排队；RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON 不再看，设了无害（照旧在传进命令之前清掉）。auto 照常认，表里没有这条取保守默认值。
#
# 日常一条（2026-10-06 撤了，同上）：从 Claude Code 会话里起的一条与别的一样排队（等待上限默认同样是 3600 秒），同一个会话已有一条在跑也照常放行、
#   照预测记账；RUN_WITH_MEMORY_CAP_PARALLEL_REASON 不再看，设了无害（照旧在传进命令之前清掉）。账上每一笔照旧记下会话号（CLAUDE_CODE_SESSION_ID）。
#
# 显存不够（用户 2026-10-06 定：运行时认显存 OOM，records/2026-09-16-subagent拆分提案.md 第五十五、五十六节）：命令退了非 0、或撞了上限、或被总上限挤掉时，
#   包装读 journalctl -k 里它起止之间（起跑前 1 秒到退出后 GPU_JOURNAL_SLACK_SECONDS 秒，--since / --until 按 epoch 秒给）的行，
#   同一行既有 NVRM 又有 NV_ERR_NO_MEMORY（显卡驱动报的显存不够）就报「显存不够」：退非 0 的那种退 245；撞了上限、被挤掉的照旧退 250 / 254，另报一行。
#   驱动的这一行不带进程号，同一段时间里别的进程撞了显存也算进来：宁可把这一条的结果当不算数、重跑，不把显存不够当成测试红。
#   读不到内核日志（没有 journalctl、权限不够、出错、超时、连最近一条内核消息都读不出）报「显存 OOM 没查成」，不当成没撞，退出码照命令自己的。
#   命令退 0 的不查：结果作数，也省一次读日志。只给自证换 journalctl：RUN_WITH_MEMORY_CAP_JOURNALCTL=<假的那一份>（与 _KEY 等一样在传进命令之前清掉）。
#
# 限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT，秒；不设不限）：给 scope 设 RuntimeMaxSec，从起跑算、不算排队的时间；到点 systemd 给 scope 里每个进程发 TERM，
#   RUN_WITH_MEMORY_CAP_KILL_GRACE（默认 30）秒还不退再发 KILL，退 253。要限时就用这个变量，别在包装外面套 timeout：套在外面的把排队的时间也算进去。
# 等待上限（RUN_WITH_MEMORY_CAP_WAIT_SECONDS，秒，默认 3600）。这几个只管这一条：RUN_WITH_MEMORY_CAP_KEY、_TIME_LIMIT、_WAIT_SECONDS、_QUEUE_BY_CAP、_OOM_FIRST 传进命令之前清掉，
#   命令里再经包装跑的不会接着用。
# 这一条命令自己的临时目录（用户 2026-10-23 定案「自己用完自己删」；records/2026-09-16-subagent拆分提案.md 第九十二节末尾）：
#   调用方没设 TMPDIR（或设成空串）时，起命令之前在 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT（默认 /tmp）下 mktemp -d 一个 singlefs-run-with-memory-cap-<包装进程号>-XXXXXX，
#   export 成 TMPDIR 交给命令；包装退出时（命令正常退、退非 0、被信号杀、撞了这一条的上限、超时、被总上限挤掉、排不上退 252、包装被 TERM / INT 停，都走 EXIT trap）
#   整个删掉，删完回读还在就往 stderr 报一行、退出码照旧。测试进程被 SIGKILL 时 Drop 守卫不走，留在它 TMPDIR 里的东西由这一步删。
#   SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1（名字与含义同 crates/singlefs-harness 的 KEEP_TEMPORARY_FILES_ENVIRONMENT_VARIABLE：只认 1）时不删，往 stderr 打一行
#   「run-with-memory-cap: SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：这一条命令的临时目录留着看现场：<路径>」，看完自己删。
#   调用方设了 TMPDIR 的照它的：不另建、不删（那一份归调用方；套在外层包装里的里层一条读到的就是外层给的那一份）。
#   --check、--status、--selftest 不起命令，不建。包装自己被 KILL / HUP 时 EXIT trap 不走，那一份留在 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 下（名字带包装进程号）。
# 其余环境变量有默认，只在自证与特殊场合设：
#   RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 这一条命令自己的临时目录建在哪个目录下，默认 /tmp（上一段）；自证指到自己的临时目录里，核跑完那里一个不剩
#   RUN_WITH_MEMORY_CAP_SLICE          slice 名，默认 singlefs-heavy.slice（名字里的 - 是 systemd 的层级：它挂在 singlefs.slice 底下）；只许 singlefs 开头的 .slice，别的按用法错退 251
#   RUN_WITH_MEMORY_CAP_SLICE_TOTAL    直接给总上限（systemd 写法），顶掉「整机内存 − 余量」；自证用它压到几百 M
#   RUN_WITH_MEMORY_CAP_STATE_DIR      锁与账的目录，默认 ${XDG_RUNTIME_DIR:-/run/user/<uid>}/singlefs-heavy-admission（tmpfs，重启清空，与 slice 同寿）
#   RUN_WITH_MEMORY_CAP_CGROUP_ROOT    cgroup v2 挂在哪，默认 /sys/fs/cgroup
#   RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE  判完放得下、记账之前停这么多秒：只给自证把「几条同时来」的竞态撑开
#   RUN_WITH_MEMORY_CAP_SELFTEST_TARGET       --selftest 测哪一份包装，默认自己：拿改前那一份跑新的自证，看它判错
#   RUN_WITH_MEMORY_CAP_SELFTEST_SLICE_PAUSE  slice 的总上限设完、读回之前停这么多秒；RUN_WITH_MEMORY_CAP_SELFTEST_SLICE_PAUSE_FILE 给了就在停之前先写这个文件：只给自证把「并发调用者」的交错撑开
#   RUN_WITH_MEMORY_CAP_HOST_LOOKUP           设了（任何非空值，例 off）排队与退 252 时不查别的机器；RUN_WITH_MEMORY_CAP_HOST_LOOKUP_SECONDS 换掉查的时限（文件头「排队时去找别的机器」）
#   RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=host-lookup  --selftest 只跑排队时去找别的机器那几项（看它红在哪用；只跑这几项的那一趟不算整份自证通过）
#   RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=peak-history  --selftest 只跑峰值表近几次、峰值涨了、撞顶与被挤掉的告警、有意撞顶不写告警、告警跟着峰值表走那几项（同上，不算整份自证通过）
#   RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=temporary-directory  --selftest 只跑这一条命令自己的临时目录那几项（同上，不算整份自证通过）
#   RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=broadcast  --selftest 只跑资源释放广播那几项（同上，不算整份自证通过）
#   RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=slice-total  --selftest 只跑被总上限挤掉、内核挑谁与盯账那几项（同上，不算整份自证通过）
#   RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY  设了（非空）这一条不起盯账、不设 slice 的 memory.high，撞总上限全由内核按 oom_score_adj 挑：只给自证判内核兜底那几例；
#                                             带着自证的标记（文件头「弄坏开关只在自证里认」）才认，平常跑设了不起作用
#   RUN_WITH_MEMORY_CAP_SELFTEST_WATCH_TRACE  盯账每判一次（slice 到了触发线）往这个文件追加一行此刻的数：只给自证证明「两条都不超账」那一例真判到了；同样带着自证的标记才认
# 套在包装里（包装里再经包装跑的：主 agent 照规则 run-with-memory-cap.sh 16G bash scripts/crash/crash-point-replay.sh --quick 起，里面放量那几项再经它；
#   run-with-memory-cap.sh <上限> bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick 里每条变异再经它）：外层起 scope 时给命令设两个标记
#   RUN_WITH_MEMORY_CAP_OUTER_UNIT=<这一条的 scope 单元名>、RUN_WITH_MEMORY_CAP_OUTER_CAP=<这一条的上限>。里层这一条两个标记都在、上限写法认得、
#   而且自己此刻就在那个 scope 的 cgroup 里（/proc/self/cgroup 的末段是 <单元名>.scope；标记随环境漏到别处的进程不算），就算套在外层里：
#   外层已经照它要的量占着账，里层不另记账、不排队（不再因外层占着账而自己排不进、等满退 252），照样另起自己的 scope、挪出外层、挂在同一个 slice 里：
#   自己的 MemoryMax、限时、撞了自己的上限退 250、被总上限挤掉退 254、超时退 253、显存不够退 245 与峰值表都照旧（门禁 checker-tier-crates-mutation-replay、
#   scripts/mutation/mutate.sh、scripts/gate/verus-proof-check.py、scripts/mutation/broad-mutation.py 这些调用方靠每一条自己的 scope 分 250 / 253 / 254，
#   不并进外层的 scope）；它起的命令拿到的两个标记换成它自己的。里层的上限比外层大（外层装不下它）：按用法错退 251、命令没跑，出路写把上限写到外层以内。
#   外层的结局判法不变。里层不排队，几条里层同时跑、合起来超过外层上限时只有 slice 的总上限兜着（被挤掉的退 254）：同时起几条、各给多少，由调用方按外层的上限分。
#   别的不套着的一条照旧排队：外层账上那一笔与里层 scope 实占的都算进 slice 已占（偏多不偏少）。
#   调用方要知道外层给了多少（例 scripts/crash/crash-point-replay.sh 把它转给放量入口与双机驱动的 --memory-cap），读 RUN_WITH_MEMORY_CAP_OUTER_CAP。
#
# 弄坏开关 RUN_WITH_MEMORY_CAP_BREAK=<项>（只给 --selftest 证明它会红用，一次给一个；打开时自证退 1）。逐项写开关名、弄坏哪一处、打开后自证哪几项红
# （自证的例没有标签，项名直接写那条「✗ 自检：」失败行的句首、到「，实际」之前，长的截到能认出是哪一例；括号里写那一行接着报的。
#   红的项多的，同一批几项写在一行，「」逐项隔开）：
#   RUN_WITH_MEMORY_CAP_BREAK=nocap  不进 scope 直接跑（第一版之前的跑法），命令跑在调用方的 cgroup 里；60 项红
#       「单进程超账（记账 40 MiB 的那一条涨到 200 MiB）时盯账应当只停它」「盯账停的那一条应当往告警记录写一条 memory-squeezed」「多进程超账（记账 40 MiB 的那一条在同一条 scope 里起 8 个进程各涨到 24 MiB）时盯账应当只停它」
#       「一口气拿满（记账 40 MiB 的那一条起来就拿 200 MiB）时 slice 的 memory.high 应当是总上限的 95%」「两条都不超账（记账 240 MiB 与 140 MiB、各超出约 5 MiB，合计约 390 MiB 过了触发线 380 MiB）时盯账应当判到」
#       「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」「被内核挤掉而没超出记账的那一条应当退 254」（盯账那几项：命令不进 scope，超账那一条退 0、谁都没被停，slice 不在、读不到它的 oom_kill，峰值表里两行照旧是记好的那一次）
#       「峰值表里记着 1.8G（1843 MiB）、这一次写 24G 上限的一条应当按 1843 MiB 记账」（实际 memory.max 是 max）
#       「峰值表里没有、写 32G 的那一条应当按 host-resources.py 的预测 8G 记账」（实际账上没有它那一笔、memory.max 不是 32G）
#       「上限 64M 里分配 512 MiB 的探针应当退 250 并报「撞了内存上限 64M」」（实际退 0：allocated 512 MiB）
#       「吃内存的是孙进程时应当照样退 250」（实际退 101）
#       「上限之内被 kill -9 的命令应当原样退 137、不算撞顶，stderr 里不多出外壳补的「Killed」那一行」（实际退 137，多了 …Killed 那一行）
#       「auto 应当取实测峰值 10M 的 1.5 倍」「auto 而峰值表里没有这条时应当取保守默认值 8G 跑」（实际「max」）
#       「scope 里的 memory.max、memory.swap.max 应当是 67108864 与 0」「scope 应当挂在 slice singlefs_memory_selftest_….slice 底下」（实际是调用方的 session-….scope）
#       「从会话里起的一条此刻放不下时应当排队、等前一条跑完再起」（实际两条同时在跑）
#       「命令退 3、起止之间内核报了 NVRM … NV_ERR_NO_MEMORY 时应当退 245」「读内核日志应当按命令起止给 --since @<起> 与 --until @<止>」
#       「内核日志读不出来时应当照退命令自己的 3、报「显存 OOM 没查成」」「撞了自己的上限、起止之间内核又报了显存不够时应当照旧退 250」（实际不读内核日志）
#       「slice 的总上限设不上时应当退 251」「slice 的总上限设不上时 --check 应当退 251」
#       「余量 1024T 比整机内存还大时应当退 251」「D-Bus 连不上时应当退 251」「D-Bus 连不上时 --check 应当退 251」
#       「slice 设得上而 systemd-run 不认上限 99999999999999T 时 --check 应当退 251」「两条同时来（各要 200M、总上限 256M）应当一条一条跑」
#       「前一条还没涨到量时来的后一条应当等它跑完」「峰值表里没有的那一条应当按上限 200M 算、排不上退 252」「要 512M 而总上限 256M 时应当不等、立刻退 252」
#       「两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 254」「撞过顶的键（上一次撞了 200M）这一次上限 300M」
#       「设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP 的那一条应当按自己的上限 200M 排队」「按峰值表放进来的那一条涨过 slice 的总上限时」
#       「按峰值表放进来又被总上限挤掉之后，峰值表里这个键应当记成它的上限 200M」「撞过顶的键重跑时应当按自己的上限排队」
#       「两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条涨过总上限时」「限时 1 秒跑 sleep 4 应当 4 秒之内退 253」
#       「账上那一笔的包装已经死了，应当删掉它」「账上起始时刻记成 null、包装不在的那一笔应当算死账」（这一批实际都退 0、命令照跑）
#       「排队中给包装发 TERM 这一例没判成」「排队中给包装发 INT 这一例没判成」「排队中给包装发 KILL 这一例没判成」「排队中给包装发 HUP 这一例没判成」
#       「放行那一刻包装已经不在这一例没判成」（包装没排队、退 0）
#       「账上没有在跑的而放不下这一例没判成」（--check 没起 slice，查不到它的 ControlGroup）
#       「跑完一条拿 30 MiB 的，峰值表里 selftest-small 应当记下 30 MiB 到 200M 之间的峰值」「scope 里一直有进程收 TERM 时，外壳照样要把峰值记进表」
#       「峰值表里 selftest-threads-capped 那一行应当是六列」「峰值表里 selftest-threads-same 那一行应当是六列」
#       「峰值表里 selftest-threads-differing 那一行应当是六列」「峰值表里 selftest-threads-unset 那一行应当是六列」（峰值表里记的是「」）
#       「峰值表里 selftest-threads-default 那一行应当是六列」「峰值表里 selftest-threads-default-cap 那一行应当是六列」（同上）
#       「跑完应当不留账、不留 scope」（实际 context-…的账没清空）
#       「套在外层里起的一条应当不另记账、不排队」「套在外层里而上限比外层大时应当按用法错退 251」「套在外层里的一条撞了自己的上限应当退 250」
#       「标记变量漏到外层 scope 之外时应当照没套跑」（不开 scope 就没有标记、没有账：它退 0、命令跑了，账目录不在）
#   RUN_WITH_MEMORY_CAP_BREAK=reject-foreign-reserve  余量不同的并发调用者照旧拒（改前的跑法）：读回不是自己设的值就退 251，slice 上有活着的 scope 也照样去改 slice 的 MemoryMax；3 项红
#       「余量不同的两个调用者并发、先到的在设完总上限读回之前被后到的改了值时，两条都应当退 0、先到的照读回的值走并报「照 slice … 已有的总上限」」（实际先到的退 251：设了 MemoryMax=…，读回是后到那一条的值）
#       「slice 上有余量 10G 的一条活着时，余量默认的后到一条应当照已有的总上限走、不改 slice 的 MemoryMax、报「照 slice … 已有的总上限 … 字节走」并写明来自哪个 scope、退 0」（实际退 0，但 slice 的 MemoryMax 被改成了后到那一条的）
#       「后到一条自己算的总上限 512M 比 slice 上在跑那一条设的 256M 大时，排队应当照已有的 256M 算、等前一条跑完、两条都退 0」（实际两条同时在跑：按自己的 512M 判，放得下）
#   RUN_WITH_MEMORY_CAP_BREAK=check-reports-foreign-reserve  --check 也打「照 slice … 已有的总上限」那一行（改前的跑法）；1 项红
#       「slice 上有余量 10G 的一条活着时，默认余量的 --check 64M 应当照已有的总上限试、不改 slice 的 MemoryMax、stderr 里不打「照 slice」那一行、退 0」（实际退 0、slice 没改，但 stderr 多了那一行）
#   RUN_WITH_MEMORY_CAP_BREAK=fallback  systemd-run 起不来时退回无上限跑
#       「slice 设得上而 systemd-run 不认上限 99999999999999T 时 --check 应当退 251、报 systemd-run」（实际退 0：Failed to parse MemoryMax=…）
#   RUN_WITH_MEMORY_CAP_BREAK=exitcode  按退出码 137 / 143 判撞顶，不看 scope 的 Result（也不读内核日志）；13 项红
#       「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」「被内核挤掉而没超出记账的那一条应当退 254」（实际被内核挤掉的那一条退 250；盯账停的那一支先认标记，不受这个开关影响）
#       「吃内存的是孙进程时应当照样退 250」（实际退 101）
#       「上限之内被 kill -9 的命令应当原样退 137、不算撞顶」（实际退 250）
#       「命令退 3、起止之间内核报了 NVRM … NV_ERR_NO_MEMORY 时应当退 245」「内核日志读不出来时应当照退命令自己的 3、报「显存 OOM 没查成」」（实际退 3、没报）
#       「读内核日志应当按命令起止给 --since @<起> 与 --until @<止>」（实际参数「」）
#       「撞了自己的上限、起止之间内核又报了显存不够时应当照旧退 250、另报一行「显存不够」」（实际退 250、没报）
#       「两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 254」（实际退 250 与 0）
#       「按峰值表放进来的那一条涨过 slice 的总上限时」「两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条涨过总上限时」（实际它退 250）
#       「按峰值表放进来又被总上限挤掉之后，峰值表里这个键应当记成它的上限 200M」（实际记的是量到的峰值）
#       「限时 1 秒跑 sleep 4 应当 4 秒之内退 253」（实际退 250）
#   RUN_WITH_MEMORY_CAP_BREAK=noresult  不认 scope 的 Result；12 项红（另一项「套在外层里的一条撞了自己的上限应当退 250」：实际它退 137）
#       「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」「被内核挤掉而没超出记账的那一条应当退 254」（实际被内核挤掉的那一条退 137、报「被外面停了」）
#       「上限 64M 里分配 512 MiB 的探针应当退 250」（实际退 137）
#       「吃内存的是孙进程时应当照样退 250」（实际退 101）
#       「撞了自己的上限、起止之间内核又报了显存不够时应当照旧退 250」（实际退 245：落进命令退非 0 那一支）
#       「两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 254」（实际退 0 与 137）
#       「设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP 的那一条应当按自己的上限 200M 排队」「按峰值表放进来的那一条涨过 slice 的总上限时」
#       「撞过顶的键重跑时应当按自己的上限排队」「两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条涨过总上限时」（实际它退 137）
#       「按峰值表放进来又被总上限挤掉之后，峰值表里这个键应当记成它的上限 200M」（实际记的是量到的峰值）
#   RUN_WITH_MEMORY_CAP_BREAK=noslice  不挂 slice 也不排队（加总上限之前的跑法）；35 项红
#       「单进程超账（记账 40 MiB 的那一条涨到 200 MiB）时盯账应当只停它」「盯账停的那一条应当往告警记录写一条 memory-squeezed」「多进程超账（记账 40 MiB 的那一条在同一条 scope 里起 8 个进程各涨到 24 MiB）时盯账应当只停它」
#       「一口气拿满（记账 40 MiB 的那一条起来就拿 200 MiB）时 slice 的 memory.high 应当是总上限的 95%」「两条都不超账（记账 240 MiB 与 140 MiB、各超出约 5 MiB，合计约 390 MiB 过了触发线 380 MiB）时盯账应当判到」
#       「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」「被内核挤掉而没超出记账的那一条应当退 254」（盯账那几项：不挂 slice 就不盯账，两条各在自己的 scope 里跑完）
#       「峰值表里记着 1.8G（1843 MiB）、这一次写 24G 上限的一条应当按 1843 MiB 记账」（实际账上没有它那一笔）
#       「峰值表里没有、写 32G 的那一条应当按 host-resources.py 的预测 8G 记账」（实际账上没有它那一笔）
#       （其中「套在外层里起的一条应当不另记账、不排队」「标记变量漏到外层 scope 之外时应当照没套跑」两项：不排队也就不记账，账上 0 笔）
#       「从会话里起的一条此刻放不下时应当排队、等前一条跑完再起」（实际两条同时在跑）
#       「scope 应当挂在 slice singlefs_memory_selftest_….slice 底下」（实际挂在 app.slice 底下）
#       「slice 的总上限设不上时应当退 251」「slice 的总上限设不上时 --check 应当退 251」「余量 1024T 比整机内存还大时应当退 251」
#       「两条同时来（各要 200M、总上限 256M）应当一条一条跑」「前一条还没涨到量时来的后一条应当等它跑完」「峰值表里没有的那一条应当按上限 200M 算、排不上退 252」
#       「要 512M 而总上限 256M 时应当不等、立刻退 252」「两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 254」
#       「撞过顶的键（上一次撞了 200M）这一次上限 300M」「设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP 的那一条应当按自己的上限 200M 排队」
#       「按峰值表放进来的那一条涨过 slice 的总上限时」「撞过顶的键重跑时应当按自己的上限排队」「两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条涨过总上限时」
#       「账上那一笔的包装已经死了，应当删掉它」「账上起始时刻记成 null、包装不在的那一笔应当算死账」（这一批实际都退 0）
#       「排队中给包装发 TERM 这一例没判成」「排队中给包装发 INT 这一例没判成」「排队中给包装发 KILL 这一例没判成」「排队中给包装发 HUP 这一例没判成」
#       「放行那一刻包装已经不在这一例没判成」（包装没排队、退 0）
#       「账上没有在跑的而放不下这一例没判成」（--check 没起 slice，查不到它的 ControlGroup）
#       「跑完应当不留账、不留 scope」（实际 context-… 的账没清空）
#   RUN_WITH_MEMORY_CAP_BREAK=slicefallback  slice 设不上时不挂 slice 照跑
#       「slice 的总上限设不上时应当退 251」「slice 的总上限设不上时 --check 应当退 251」「余量 1024T 比整机内存还大时应当退 251」（实际退 0、命令跑了）
#   RUN_WITH_MEMORY_CAP_BREAK=nolock  判与记账不拿锁
#       「两条同时来（各要 200M、总上限 256M）应当一条一条跑、都退 0」（实际退 254 与 0）
#       「放行那一刻包装已经不在这一例没判成」（没找到停在记账之前的那一步）
#   RUN_WITH_MEMORY_CAP_BREAK=noledger  已占只看 slice 的用量，不算账上还没涨到量的；8 项红
#       「从会话里起的一条此刻放不下时应当排队、等前一条跑完再起」（实际两条同时在跑）
#       「两条同时来（各要 200M、总上限 256M）应当一条一条跑、都退 0」「前一条还没涨到量时来的后一条应当等它跑完、两条都退 0」（有一条退 254）
#       「峰值表里没有的那一条应当按上限 200M 算、排不上退 252」（实际退 0、命令跑了）
#       「排队中给包装发 TERM 这一例没判成」「排队中给包装发 INT 这一例没判成」「排队中给包装发 KILL 这一例没判成」「排队中给包装发 HUP 这一例没判成」
#   RUN_WITH_MEMORY_CAP_BREAK=nopredict  峰值表里没有的照旧按它的上限算，不看 host-resources.py 的预测（改前的跑法）
#       「峰值表里没有、写 32G 的那一条应当按 host-resources.py 的预测 8G 记账，在总上限 40G 里与按上限 24G 记账的那一条同时在跑」
#       （实际它退 252：按 32G 记账，24G + 32G 超总上限、排满 2 秒）
#   RUN_WITH_MEMORY_CAP_BREAK=nodefault  峰值表里没有的按 0 算；11 项红
#       「从会话里起的一条此刻放不下时应当排队、等前一条跑完再起」（实际两条同时在跑）
#       「两条同时来（各要 200M、总上限 256M）应当一条一条跑、都退 0」「前一条还没涨到量时来的后一条应当等它跑完、两条都退 0」（有一条退 254）
#       「峰值表里没有的那一条应当按上限 200M 算、排不上退 252」「要 512M 而总上限 256M 时应当不等、立刻退 252」（实际退 0）
#       「按峰值表放进来的那一条涨过 slice 的总上限时，内核应当杀它（退 254）、不杀照上限放进来的常驻那一条」（实际常驻那一条退 254）
#       「排队中给包装发 TERM 这一例没判成」「排队中给包装发 INT 这一例没判成」「排队中给包装发 KILL 这一例没判成」「排队中给包装发 HUP 这一例没判成」
#       「峰值表里没有、写 32G 的那一条应当按 host-resources.py 的预测 8G 记账」（实际账上记的是 0）
#   RUN_WITH_MEMORY_CAP_BREAK=ignoretable  不看峰值表、一律按上限算；16 项红
#       「单进程超账（记账 40 MiB 的那一条涨到 200 MiB）时盯账应当只停它」「盯账停的那一条应当往告警记录写一条 memory-squeezed」「多进程超账（记账 40 MiB 的那一条在同一条 scope 里起 8 个进程各涨到 24 MiB）时盯账应当只停它」
#       「一口气拿满（记账 40 MiB 的那一条起来就拿 200 MiB）时 slice 的 memory.high 应当是总上限的 95%」「两条都不超账（记账 240 MiB 与 140 MiB、各超出约 5 MiB，合计约 390 MiB 过了触发线 380 MiB）时盯账应当判到」
#       「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」「被内核挤掉而没超出记账的那一条应当退 254」（盯账那几项：两条都按上限 300M 记账、放不进 400M，后一条等前一条跑完才起，谁都没被停）
#       「峰值表里记着 1.8G（1843 MiB）、这一次写 24G 上限的一条应当按 1843 MiB 记账，在总上限 40G 里与按上限 24G 记账的那一条同时在跑」（实际它退 252：按 24G 记账）
#       「峰值表里没有、写 32G 的那一条应当按 host-resources.py 的预测 8G 记账」（实际它退 252：按 32G 记账）
#       「峰值表里没有的那一条应当按上限 200M 算」（实际退 252，报的是「按它的上限 200M 算（弄坏开关 ignoretable）」）
#       「峰值表里记着约 30 MiB 的两条（上限各 200M、总上限 256M）应当按记下的量一起放行、同时在跑」（实际一条一条跑的）
#       「两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 254」（实际退 0 与 0）
#       「撞过顶的键（上一次撞了 200M）这一次上限 300M、总上限 256M 时应当不放进去、当场退 252、报「这个键撞过顶」」（实际退 252，报的不是这一句）
#       「设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP 的那一条应当按自己的上限 200M 排队」「按峰值表放进来的那一条涨过 slice 的总上限时」
#       「两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条涨过总上限时」（实际它退 250：按上限排队，等常驻那一条跑完才起、撞了自己的上限）
#   RUN_WITH_MEMORY_CAP_BREAK=noslicehit  被内核挤掉的也报成撞了自己的上限（盯账停的那一支先认标记，不受这个开关影响）；6 项红
#       「两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 254、报「被总上限挤掉」」（实际退 250 与 0）
#       「按峰值表放进来的那一条涨过 slice 的总上限时，内核应当杀它」「两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条涨过总上限时」
#       「按峰值表放进来又被总上限挤掉的那一条应当退 254、往告警记录写一条 memory-squeezed」「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」「被内核挤掉而没超出记账的那一条应当退 254」（实际挨挤的那一条退 250）
#   RUN_WITH_MEMORY_CAP_BREAK=notimelimit  不设限时
#       「限时 1 秒跑 sleep 4 应当 4 秒之内退 253」（实际退 0、用了 4 秒）
#   RUN_WITH_MEMORY_CAP_BREAK=keepstale  账上包装已经死了的那一笔不删
#       「账上那一笔的包装已经死了，应当删掉它、放行这一条退 0」（实际退 252、那一笔还在）
#       「账上起始时刻记成 null、包装不在的那一笔应当算死账」（实际退 252、null 那一笔还在）
#       「跑完应当不留账、不留 scope」（实际 context-14、context-9 的账没清空）
#   RUN_WITH_MEMORY_CAP_BREAK=noprivate  --status 不读私有配置，本地模型服务那一项一律当没配
#       「--status 私有配置 set 那一例应当退 0、「slice 外面」那一行含「本地模型服务的 anon + shmem 5 MiB、」」（实际退 0、不含那一句）
#   下面四个开关把 2026-10-06 撤了的手续拒绝装回去（证明自证认得出它们又出现了）：
#   RUN_WITH_MEMORY_CAP_BREAK=overpeakreject  上限超过峰值表里实测峰值 2 倍就按用法错退 251（撤之前的判法）；18 项红
#       「单进程超账（记账 40 MiB 的那一条涨到 200 MiB）时盯账应当只停它」「盯账停的那一条应当往告警记录写一条 memory-squeezed」「多进程超账（记账 40 MiB 的那一条在同一条 scope 里起 8 个进程各涨到 24 MiB）时盯账应当只停它」
#       「一口气拿满（记账 40 MiB 的那一条起来就拿 200 MiB）时 slice 的 memory.high 应当是总上限的 95%」「两条都不超账（记账 240 MiB 与 140 MiB、各超出约 5 MiB，合计约 390 MiB 过了触发线 380 MiB）时盯账应当判到」
#       「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」「被内核挤掉而没超出记账的那一条应当退 254」（盯账那几项：记账 40 MiB、上限 300M 的那一条退 251、命令没跑）
#       「峰值表里记着 1.8G（1843 MiB）、这一次写 24G 上限的一条应当按 1843 MiB 记账」（实际退 251：24G 超过 1.8G 的 2 倍）
#       「上限 64M 超过峰值表里实测峰值 10M 的 2 倍时应当照常跑、不拒」（实际退 251、命令没跑）
#       另外九项是峰值表里记着、上限比峰值大 2 倍以上的那几例（显存那两例的键在头一例跑完之后记了 3M）：
#       「内核日志读不出来时应当照退命令自己的 3」「起止之间只有别的 NVRM 行（Xid）时应当照退命令自己的 3」
#       「峰值表里记着约 30 MiB 的两条（上限各 200M、总上限 256M）应当按记下的量一起放行」「两条合起来超总上限 256M（各自没超上限 200M）时」
#       「设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP 的那一条应当按自己的上限 200M 排队」「按峰值表放进来的那一条涨过 slice 的总上限时」
#       「按峰值表放进来又被总上限挤掉之后，峰值表里这个键应当记成它的上限 200M」「撞过顶的键重跑时应当按自己的上限排队」
#       「两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条涨过总上限时」（实际退 251）
#   RUN_WITH_MEMORY_CAP_BREAK=autonodefault  auto 而峰值表里没有这条就按用法错退 251（撤之前的判法）
#       「auto 而峰值表里没有这条时应当取保守默认值 8G 跑」（实际退 251）
#   RUN_WITH_MEMORY_CAP_BREAK=sessiononejob  同一个会话账上已有一条在跑就退 252（撤之前的判法）
#       「同一个会话已有一条在跑时第二条应当照常放行、跟它同时跑」（实际退 252、命令没跑）
#   RUN_WITH_MEMORY_CAP_BREAK=dailynoqueue  从会话里起的默认不排队、放不下当场退 252（撤之前的判法）
#       「从会话里起的一条此刻放不下时应当排队、等前一条跑完再起，不当场退 252」（实际退 252、命令没跑）
#   显存不够那几项（文件头「显存不够」一段）：
#   RUN_WITH_MEMORY_CAP_BREAK=nogpuoom  命令退出之后不读内核日志
#       「命令退 3、起止之间内核报了 NVRM … NV_ERR_NO_MEMORY 时应当退 245、报「显存不够」」（实际退 3）
#       「撞了自己的上限、起止之间内核又报了显存不够时应当照旧退 250、另报一行「显存不够」」（实际退 250、没报）
#   RUN_WITH_MEMORY_CAP_BREAK=journalasclean  读不到内核日志当成没撞
#       「内核日志读不出来时应当照退命令自己的 3、报「显存 OOM 没查成」」（实际退 3、没报）
#   RUN_WITH_MEMORY_CAP_BREAK=anynvrm  只要是 NVRM 行就当显存不够（不看 NV_ERR_NO_MEMORY）
#       「起止之间只有别的 NVRM 行（Xid）时应当照退命令自己的 3、不报显存不够」（实际退 245）
#   RUN_WITH_MEMORY_CAP_BREAK=wholejournal  读内核日志不按命令起止给 --since / --until
#       「读内核日志应当按命令起止给 --since @<起> 与 --until @<止>」（实际参数里没有 --since）
#   RUN_WITH_MEMORY_CAP_BREAK=gpuoomonsuccess  命令退 0 也查内核日志、查到显存不够就退 245
#       「命令退 0 时应当照退 0、不读内核日志」（实际退 245）
#   RUN_WITH_MEMORY_CAP_BREAK=queuehopeless  账上没有在跑的、这一条要的量加 slice 里回收不掉的用量已超总上限时照旧排队（改前的跑法）
#       「账上没有在跑的、要 40G 而回收不掉 420M、总上限 40.1G 时应当不等、当场退 252」（实际退 252、用了 4 秒，stderr 是「排队（已等 0 秒，至多 4 秒）」）
#   RUN_WITH_MEMORY_CAP_BREAK=queuebaseline  账上有在跑的、这一条要的量加 slice 里不属于任何 scope 的常驻用量已超总上限时照旧排队（待优化第 19 行改前的跑法）
#       「账上有一条在跑、而 slice 里 420M 常驻不属于任何 scope、要 40G 加常驻超总上限 40.1G 时应当不等、当场退 252」（实际退 252、用了 4 秒，stderr 是「排队（已等 0 秒，至多 4 秒）」）
#   RUN_WITH_MEMORY_CAP_BREAK=nopeakmin  峰值表里有这条也按它的上限记账，不取较小的（待优化第 19 行改前的跑法）；12 项红
#       「单进程超账（记账 40 MiB 的那一条涨到 200 MiB）时盯账应当只停它」「盯账停的那一条应当往告警记录写一条 memory-squeezed」「多进程超账（记账 40 MiB 的那一条在同一条 scope 里起 8 个进程各涨到 24 MiB）时盯账应当只停它」
#       「一口气拿满（记账 40 MiB 的那一条起来就拿 200 MiB）时 slice 的 memory.high 应当是总上限的 95%」「两条都不超账（记账 240 MiB 与 140 MiB、各超出约 5 MiB，合计约 390 MiB 过了触发线 380 MiB）时盯账应当判到」
#       「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」「被内核挤掉而没超出记账的那一条应当退 254」（盯账那几项：同 ignoretable，后一条等前一条跑完才起）
#       「峰值表里记着 1.8G（1843 MiB）、这一次写 24G 上限的一条应当按 1843 MiB 记账，在总上限 40G 里与按上限 24G 记账的那一条同时在跑」（实际它退 252：按 24G 记账）
#       「峰值表里记着约 30 MiB 的两条（上限各 200M、总上限 256M）应当按记下的量一起放行、同时在跑」（实际一条一条跑的）
#       「两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 254」「按峰值表放进来的那一条涨过 slice 的总上限时」
#       「两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条涨过总上限时」（实际它退 250：按上限排队）
#   RUN_WITH_MEMORY_CAP_BREAK=peakqueue  撞过顶的键照旧按峰值表里记的量排队（改前的跑法）
#       「撞过顶的键（上一次撞了 200M）这一次上限 300M、总上限 256M 时应当不放进去、当场退 252」（实际退 254：按 200M 放进去、涨过了 slice 的总上限）
#   RUN_WITH_MEMORY_CAP_BREAK=ignorequeuebycap  不认 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP，升档那趟照旧按峰值表排队
#       「设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP 的那一条应当按自己的上限 200M 排队、等常驻那一条跑完再起」（实际按 20 MiB 放进去，两条同时在跑、它退 254）
#       「账上没有在跑的、要 40G 而回收不掉 420M、总上限 40.1G 时应当不等、当场退 252」「账上有一条在跑时（要 40G、回收不掉 420M、总上限 40.1G）应当照旧排队」
#       「账上有一条在跑、而 slice 里 420M 常驻不属于任何 scope、要 40G 加常驻超总上限 40.1G 时应当不等、当场退 252」
#       （实际都退 0、命令跑了：峰值表里没有，按预测 8G 放进去）
#   RUN_WITH_MEMORY_CAP_BREAK=nooomadjust  账上有记账的一律不写 oom_score_adj（命令照旧继承调用方的，自证里是 0）；3 项红
#       「按峰值表放进来的那一条涨过 slice 的总上限时，内核应当杀它（退 254）、不杀照上限放进来的常驻那一条」（实际常驻那一条退 254）
#       「按峰值表放进来又被总上限挤掉的那一条应当退 254、往告警记录写一条 memory-squeezed」（实际常驻那一条退 254）「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」（实际没超账的常驻那一条退 254）
#   RUN_WITH_MEMORY_CAP_BREAK=nooomfirst  不认 RUN_WITH_MEMORY_CAP_OOM_FIRST，设了它的照记账算 oom_score_adj；1 项红
#       「被内核挤掉而没超出记账的那一条应当退 254」（实际它退 0：内核按记账挑了超账的那一条。OOM_FIRST 那一例设不设都是 OOM_FIRST 那一条超出记账最多，照记账算也先杀它，不再红）
#   RUN_WITH_MEMORY_CAP_BREAK=noslicemark  按峰值表放进来又被总上限挤掉的不把这个键记成撞过顶
#       「按峰值表放进来又被总上限挤掉之后，峰值表里这个键应当记成它的上限 200M」（实际还是 20 MiB）、
#       「撞过顶的键重跑时应当按自己的上限排队、等常驻那一条跑完再起」（实际又按 20 MiB 放进去、退 254）
#       「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」（实际超账那一条的键没记成撞过顶）
#   盯账那几项（文件头「盯账」一段；RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=slice-total 连同被总上限挤掉、内核挑谁那几项只跑这几项）：
#   RUN_WITH_MEMORY_CAP_BREAK=nowatch  不起盯账（只留按记账算的 oom_score_adj 与 slice 的 memory.high）；5 项红
#       「单进程超账（记账 40 MiB 的那一条涨到 200 MiB）时盯账应当只停它」「盯账停的那一条应当往告警记录写一条 memory-squeezed」
#       「多进程超账（记账 40 MiB 的那一条在同一条 scope 里起 8 个进程各涨到 24 MiB）时盯账应当只停它」
#       「一口气拿满（记账 40 MiB 的那一条起来就拿 200 MiB）时 slice 的 memory.high 应当是总上限的 95%」
#       「两条都不超账（记账 240 MiB 与 140 MiB、各超出约 5 MiB，合计约 390 MiB 过了触发线 380 MiB）时盯账应当判到」
#       （实际没有盯账那一句、判法记录 0 行；多进程那一例内核杀的是没超账的常驻那一条）
#   RUN_WITH_MEMORY_CAP_BREAK=noslicehigh  slice 不设 memory.high（盯账与按记账算的 oom_score_adj 照旧）；1 项红
#       「一口气拿满（记账 40 MiB 的那一条起来就拿 200 MiB）时 slice 的 memory.high 应当是总上限的 95%」（实际 memory.high 是 max，内核 OOM 了一次、由它杀了超账那一条）
#       多进程那一例这个开关下时红时不红（2026-10-26 两趟里一趟红：内核杀了没超账的常驻那一条、盯账又停了超账那一条），不点名
#   RUN_WITH_MEMORY_CAP_BREAK=oomadjustflat  oom_score_adj 照改前的两档写（按峰值表或预测放进来的 500、按上限放进来的不写）；1 项红
#       「不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条」（实际没超账、常驻更大的那一条退 254，超账那一条退 0）
#   RUN_WITH_MEMORY_CAP_BREAK=slicemarkall  按峰值表放进来而被内核挤掉的一律记成撞过顶，不看峰值超没超出记账（改前的判法）；1 项红
#       「被内核挤掉而没超出记账的那一条应当退 254」（实际没超账的那一条的键记成了撞过顶）
#   套在包装里那几项（文件头「套在包装里」一段）：
#   RUN_WITH_MEMORY_CAP_BREAK=nestedqueues  套在外层里也照没套那样另记账、排队（改前的跑法）
#       「套在外层里起的一条应当不另记账、不排队」（实际它退 252、用了 4 秒，2026-10-10 崩溃点重放轻型里放量那几项就这样等满 3600 秒退 252）、
#       「套在外层里而上限比外层大时应当按用法错退 251」（实际它排队等满 4 秒退 252）
#   RUN_WITH_MEMORY_CAP_BREAK=nestedlargercap  套在外层里而上限比外层大也照跑
#       「套在外层里而上限比外层大时应当按用法错退 251」（实际它退 0、命令跑了）
#   RUN_WITH_MEMORY_CAP_BREAK=nestedbyenvonly  只看两个标记变量、不核自己在不在那个 scope 的 cgroup 里
#       「标记变量漏到外层 scope 之外时应当照没套跑」（实际退 251：标记里的 16M 比 64M 小）
#   RUN_WITH_MEMORY_CAP_BREAK=term-leaves-scope  起了 scope 之后包装被 TERM / INT 停时不 systemctl --user stop 自己的 scope，只删标记与账就退（改前的跑法，命令在 scope 里照跑）
#       「起了 scope 之后给包装发 TERM」「起了 scope 之后给包装发 INT」（实际 stop 调用为空、scope 里的命令 5 秒里没停、留下 scope）
#   RUN_WITH_MEMORY_CAP_BREAK=nestedinouterscope  套在外层里时不另起 scope、命令直接在外层的 scope 里跑
#       「套在外层里的一条撞了自己的上限应当退 250」（实际外层的 scope 撞顶、外层退 250，它没记下退出码）
#   RUN_WITH_MEMORY_CAP_BREAK=threadsdefaultunset  command_threads 不认 capped.sh default，当没经 capped.sh 跑（改前的跑法）；1 项红
#       「峰值表里 selftest-threads-default 那一行应当是六列、线程数记「…」」（实际记「未设（整机 N 核）」，括号里的 N 是本机核数）
#   RUN_WITH_MEMORY_CAP_BREAK=recordkilledpeak  命令被外面的信号停（退 128 加 1、2、9、15）而 scope 没撞自己的上限、也没被总上限挤掉时，照旧把半截的峰值记进表、不报「被外面停了」（改前的跑法）；2 项红
#       「命令被外面的 TERM 停应当原样退 143、报「被外面停了（信号 15），这一次的峰值不记」、峰值表里不留这个键」「命令被外面的 KILL 停应当原样退 137、报「被外面停了（信号 9），这一次的峰值不记」、峰值表里不留这个键」
#   排队时去找别的机器那几项（文件头「排队时去找别的机器」；RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=host-lookup 只跑这几项）：
#   RUN_WITH_MEMORY_CAP_BREAK=nohostlookup  排队与退 252 时不查别的机器（改前的跑法）；12 项红：下面列的前六项，加整体兜底那六格（四处抛异常、排队锁被占着、线程里的 query 睡着：关掉查之后走不到、没有那一行）
#       「第二台放得下时排队开始与退 252 时各打一行改去第二台的命令」「改去第二台的命令里以本机仓根起头的参数改成树内相对路径」「几条一起排队时 60 秒内只查一次」「第二台放不下时各打一行第二台此刻也放不下」
#       「第二台连不上时各打一行没查到别的机器」「查别的机器超过时限时各打一行没查到别的机器」（实际一行都没打）
#   RUN_WITH_MEMORY_CAP_BREAK=hostlookupnorecord  放得下也不写告警记录；2 项红
#       「第二台放得下时排队开始与退 252 时各打一行改去第二台的命令、各写一条告警记录」「改去第二台的命令里以本机仓根起头的参数改成树内相对路径」（实际告警记录读不了：文件不在）
#   RUN_WITH_MEMORY_CAP_BREAK=hostlookupabsolutepath  改去第二台的命令里不把以本机仓根起头的参数改成树内相对路径（改前的跑法）；1 项红
#       「改去第二台的命令里以本机仓根起头的参数改成树内相对路径（仓根本身写 .），仓根之外的绝对路径与名字只是以仓根起头的路径原样留着」
#       （实际 stderr 里还是 bash <仓根>/.claude/gate.d/x.sh --check a <仓根>，告警记录的出路里也是）
#   RUN_WITH_MEMORY_CAP_BREAK=hostlookupblocking  排队开始那一趟在前台查（查完才重判）
#       「排队开始时在查别的机器（第二台 6 秒才回话）不拖慢准入」（实际挡路那一笔删掉之后 7 秒多才起跑：要等排队开始那一趟查完）
#   RUN_WITH_MEMORY_CAP_BREAK=hostlookupnotimeout  查别的机器不设时限
#       「查别的机器超过时限时各打一行没查到别的机器：2 秒没查完」（实际等到 host-resources.py 自己的 8 秒，报的是第二台没读到）
#   RUN_WITH_MEMORY_CAP_BREAK=hostlookupunthrottled  几条一起排队时各查各的
#       「几条一起排队时 60 秒内只查一次」（实际后一条也查了、打了改去第二台的命令）
#   整体兜底那六格（文件头「整体兜底」；下面三个开关打开之后的红是推的，交回时还没在真包装上跑过）：
#   RUN_WITH_MEMORY_CAP_BREAK=hostlookupnocatch  HostLookup 接住异常也照旧抛出去（拿掉整体兜底的 try）；4 项红
#       「HostLookup 在「thread-start」那一处抛异常时只打一行没查到别的机器、退出码与关掉它的那一臂相同」与 temp-file、read-output、format 三处的同一项
#       （应当报退 251：排队那一步带着回溯退 1，包装照「排队那一步退 1」退 251）
#   RUN_WITH_MEMORY_CAP_BREAK=hostlookuplockblocking  查别的机器那一趟照旧阻塞地取排队锁
#       「排队锁被别的进程占着时这一趟不查、打一行「没查到别的机器：排队锁 … 被别的进程占着」」（应当报退 124：等那把锁等到 30 秒被停）
#   RUN_WITH_MEMORY_CAP_BREAK=hostlookupjoinunbounded  等排队开始那一趟收尾不设上限
#       「线程里的 query 睡着时退 252 之前不一直等它」（应当报退 124：等后台那一趟睡完等到 30 秒被停）
#   峰值表近几次、峰值涨了、撞顶与被挤掉的告警、有意撞顶不写告警、告警跟着峰值表走那几项（文件头「峰值表」一段；RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=peak-history 只跑这几项）：
#   RUN_WITH_MEMORY_CAP_BREAK=lastpeakonly  排队记账只看峰值表里最近一次实测（改前的判法）；2 项红
#       「撞顶之后连跑 4 次干净的应当仍按上限 64M 排，第 5 次干净的也按上限、跑完之后第 6 次才按近 5 次实测里最大的峰值」（实际第 2 次起按最近一次的峰值记账）
#       「峰值依次是 100M、80M、60M、40M 时，记账应当取近几次里最大的」（实际记的是最近一次拿 40 MiB 那一趟的峰值）
#   RUN_WITH_MEMORY_CAP_BREAK=nogrowthalert  峰值涨了也不打那一行、不写告警记录
#       「记账 N MiB（峰值表）、实测约 M MiB 的那一趟应当打「这一次峰值 X，是记账 Y 的 Z 倍：工作量可能变了」与出路、写一条 memory-peak-grew」（实际记录 0 条、没打那一行）
#   RUN_WITH_MEMORY_CAP_BREAK=growthanysource  判峰值涨了不看记账从哪来：按预测或默认值记账的也报
#       「记账来自预测的默认值（峰值表里没有这条；跑起来把账上的 need 改成 1 MiB）的那一趟应当不打那一行、不写记录」（实际写了 1 条、打了那一行）
#   RUN_WITH_MEMORY_CAP_BREAK=nooutcomealert  退 250、254 不写告警记录；3 项红
#       「撞了自己的上限应当退 250、往告警记录写一条 memory-own-cap」「按峰值表放进来又被总上限挤掉的那一条应当退 254、往告警记录写一条 memory-squeezed」
#       「盯账停的那一条应当往告警记录写一条 memory-squeezed」（这一项不在 peak-history 那几项里）
#       （实际退出码照旧、记录 0 条）
#   RUN_WITH_MEMORY_CAP_BREAK=oomfirstalerts  设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的退 250、254 照旧写 memory-own-cap、memory-squeezed（改前的写法）；2 项红
#       「设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的一条撞了自己的上限应当退 250、不写 memory-own-cap」（实际退 250、写了 1 条）
#       「两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条挨挤之后应当不写 memory-squeezed」（实际写了 1 条；这一项不在 peak-history 那几项里）
#   RUN_WITH_MEMORY_CAP_BREAK=alertsignorepeaks  不看峰值表在哪，告警一律写真记录（改前的写法）；6 项红
#       「盯账停的那一条应当往告警记录写一条 memory-squeezed」（实际峰值表旁边 0 条；这一项不在 peak-history 那几项里）
#       「峰值表指到默认之外（这一例的临时峰值表）的一条撞顶应当往峰值表旁边的 peaks.tsv.alerts.jsonl 写一条 memory-own-cap、真记录不多一行」（实际旁边 0 条、真记录 1 条）
#       读峰值表旁边那一份的这几项同样红（实际旁边 0 条，记录写进了这一例的真记录）：「撞了自己的上限应当退 250、往告警记录写一条 memory-own-cap」
#       「记账 N MiB（峰值表）、实测约 M MiB 的那一趟应当打「这一次峰值 X，是记账 Y 的 Z 倍：工作量可能变了」与出路、写一条 memory-peak-grew」
#       「按峰值表放进来又被总上限挤掉的那一条应当退 254、往告警记录写一条 memory-squeezed」
#       「设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的一条撞了自己的上限应当退 250、不写 memory-own-cap」（没设的那一条旁边 0 条）
#   这一条命令自己的临时目录那几项（文件头「这一条命令自己的临时目录」一段；RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=temporary-directory 只跑这几项）：
#   RUN_WITH_MEMORY_CAP_BREAK=temporaryleft  包装退出时不删它给命令建的临时目录（改前没有这一步，命令被 SIGKILL 时留下的就一直留着）；5 项红
#       「命令正常退出之后，包装给它的临时目录应当删掉」「命令退非 0 之后，包装给它的临时目录应当删掉」「命令被 kill -9 之后，包装给它的临时目录应当删掉」
#       「命令撞了 64M 上限被杀之后，包装给它的临时目录应当删掉」「命令超过限时被停之后，包装给它的临时目录应当删掉」（实际那一份与命令写的文件都还在）
#   RUN_WITH_MEMORY_CAP_BREAK=temporaryshared  不给命令建它自己的临时目录，调用方没设 TMPDIR 就照没设交给命令（改前的跑法）；6 项红：同上 5 项与
#       「设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时临时目录应当留着、stderr 打出它的路径」（实际命令拿到的 TMPDIR 是空的）
#   RUN_WITH_MEMORY_CAP_BREAK=temporarykeepignored  不认 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1，照删
#       「设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时临时目录应当留着、stderr 打出它的路径」（实际那一份删了、没打那一行）
#   RUN_WITH_MEMORY_CAP_BREAK=temporarycallerreplaced  调用方设了 TMPDIR 也另建一份顶掉它
#       「调用方设了 TMPDIR 时命令应当拿到调用方那一份、跑完那一份与里面的文件都还在」（实际命令拿到的是包装另建的那一份）
#   RUN_WITH_MEMORY_CAP_BREAK=temporarycallerdeleted  调用方设了 TMPDIR 时照它的，退出时却连它一起删
#       「调用方设了 TMPDIR 时命令应当拿到调用方那一份、跑完那一份与里面的文件都还在」（实际调用方那一份删了）
#   RUN_WITH_MEMORY_CAP_BREAK=alertspathtext  按字面比峰值表路径、不按真路径：经符号链接指到默认那一份的也当默认之外
#       「经符号链接 … 指到默认峰值表的一条撞顶应当把 memory-own-cap 写进真记录、默认峰值表旁边不留」（实际真记录 0 条、旁边 1 条）
#   资源释放广播那几项（文件头「资源释放广播」；RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=broadcast 只跑这几项）：
#   RUN_WITH_MEMORY_CAP_BREAK=broadcast-none  包装退出时不发（改前的样子）；3 项红
#       「命令真跑过、退 3 之后应当发一条 memory-released」「起了 scope 之后包装被 TERM 停应当退 143、发一条 memory-released exit=143」
#       「套在外层包装里跑完的一条应当不发、外层照发一条」（实际事件目录里 0 条）
#   RUN_WITH_MEMORY_CAP_BREAK=broadcast-unadmitted  排队没放行就退的也发
#       「排不上当场退 252 的一条应当不发资源释放广播」（实际事件目录里 1 条 exit=252）
#   RUN_WITH_MEMORY_CAP_BREAK=broadcast-exit-swallowed  EXIT trap 里照广播那一步的退出码 exit（广播失败吞掉包装的退出码）
#       「资源释放广播那份库出事（自证造的崩溃）时包装应当照退命令的 3」（实际退 1）
#   RUN_WITH_MEMORY_CAP_BREAK=broadcast-lines-leak  广播那一步打的行照转到包装的 stderr
#       「资源释放广播那份库出事（自证造的崩溃）时包装应当照退命令的 3、stderr 里没有广播那一步的行」（实际 stderr 里有「自证造的崩溃」那一行）
#   RUN_WITH_MEMORY_CAP_BREAK=broadcast-nested  套在外层包装里的也发
#       「套在外层包装里跑完的一条应当不发、外层照发一条」（实际 2 条）
#   RUN_WITH_MEMORY_CAP_BREAK=broadcast-wake-ignored  排队时收到广播不叫醒、不重查（改前的样子：只每 ADMISSION_POLL_SECONDS 秒重判本机）
#       「排队中收到第二台那一趟跑完的广播应当当场再查别的机器」（实际没有被资源释放广播叫醒的那一行）
#   RUN_WITH_MEMORY_CAP_BREAK=broadcast-wake-any  本机自己发的内存放出也叫醒、另查别的机器
#       「排队中收到本机自己发的内存放出应当不另查别的机器」（实际多了一行被资源释放广播叫醒的查法）
#   RUN_WITH_MEMORY_CAP_BREAK=broadcast-wake-unthrottled  几条一起排队时一个事件各查各的
#       「几条一起排队时一个资源释放广播只查一次」（实际两条都打了改去第二台的命令）
#   RUN_WITH_MEMORY_CAP_BREAK=hostlookupnocatch 另红两项：「被资源释放广播叫醒那一趟起线程时抛异常应当只打一行没查到别的机器、退出码照旧」
#       「排队中资源释放广播的 follow 出事（自证造的崩溃）应当只打一行没查到别的机器、退出码照旧」（排队那一步带着回溯退 1、包装退 251）
#   RUN_WITH_MEMORY_CAP_BREAK=guard-off  平常跑设了弄坏开关也不拒（改前的样子：探针往下走到「没给要在上限里跑的命令」退 251，不点名变量）；2 项红
#       「平常跑（没有自证的标记）设了 RUN_WITH_MEMORY_CAP_BREAK 应当退 2、点名它只在自证里认」标记「空」与标记指到头一行不对的文件各一项（实际退 251，报的是没给命令）
#
# 弄坏开关只在自证里认：RUN_WITH_MEMORY_CAP_BREAK 非空（guard-off 除外）、头一个参数不是 --selftest、又没有自证的标记的，在 preflight 之后、读参数之前退 2，
#   stderr 点名这个变量只在自证（--selftest）里认，命令一行都没跑。自证的标记：RUN_WITH_MEMORY_CAP_SELFTEST_MARK=<文件>，那份文件在、头一行是 SELFTEST_MARK_CONTENT 那一行。
#   --selftest 把它建在自己的临时目录里、export 给起的每一份包装；把这个开关原样传给包装的调用方自证（scripts/mutation/mutate.sh、scripts/hosts/multi-host-run.sh）
#   同样建一份写这一行的标记文件、export 这个变量。标记随环境传给命令（套在外层里那几例的里层包装要认它）；自证收尾删掉标记文件，漏出去的变量指不到文件，不再认。
#
# 发信号的地方，每一处只打得到这一条自己起的进程（用户 2026-09-25 定「后面的脚本不能终止前面的脚本」「不能动 ssh」；
# records/2026-09-16-subagent拆分提案.md 第四十节第 32 行）：
#   撞顶（OOMPolicy=stop）与超时（RuntimeMaxSec）由 systemd 停这一条自己开的 scope，scope 里只有这一条的外壳与它的命令；
#   查完结局 reset-failed 的是这一条自己的 unit；外壳里的 trap 只收信号、不发；起了 scope 之后被停时的 trap（stop_own_scope_then_exit）先 systemctl --user stop 点名自己开的那个 $unit.scope，
#   再删自己的标记、账与峰值文件，scope 还没建出来时只给自己 `&` 起、`$!` 记下的那个 systemd-run 发 TERM（发之前核 /proc 里它的父进程是这个包装）；
#   排队时被停的 trap（stop_queued_admission）只给自己起的那一个排队子进程发 TERM：进程号是 `&` 当场记下的，发之前核 /proc 里它的父进程是这个包装；
#   slice 名只许 singlefs 开头（case 前面那句核对）：包装给它设 MemoryMax、自证收尾停的是自证自己开的 singlefs_memory_selftest_<进程号>_<序号>.slice，
#   写成 user.slice、app.slice 这类就等于给别人的进程设上限；
#   自证里 kill -9 $$ 停的是那条命令自己；「排队中停包装」「放行那一刻包装已经不在」「起了 scope 之后停包装」几例停的是它自己 `&` 起、`$!` 记下的那个包装（最后一例收尾停的是自证自己的 slice）；TERM 风暴那一例先核 cgroup 路径在这一例自己的 slice 底下、是自己开的 singlefs-memory-cap-*.scope，
#   不对就一个信号都不发——弄坏开关 nocap、fallback 不开 scope，命令跑在调用方的 cgroup 里（会话里就是 SSH 会话的 session-*.scope）。
set -uo pipefail

MEMORY_CAP_HIT_EXIT=250
MEMORY_CAP_UNAVAILABLE_EXIT=251
MEMORY_ADMISSION_REFUSED_EXIT=252
TIME_LIMIT_HIT_EXIT=253
SLICE_TOTAL_HIT_EXIT=254
GPU_MEMORY_HIT_EXIT=245               # 显存不够（文件头「显存不够」一段）
STOPPED_FROM_OUTSIDE_SIGNALS=" 1 2 9 15 "   # 命令退 128 加这几个信号号：外面发的 HUP、INT、KILL、TERM（文件头「峰值表」一段）；SEGV、ABRT 这类是命令自己崩的，不在内
GPU_JOURNAL_SLACK_SECONDS=2           # 读内核日志的窗口在命令退出之后再放宽这么多秒：驱动的那一行与命令退出前后脚
JOURNAL_PERMISSION_HINT="not seeing messages from other users and the system"   # journalctl 看不到别人的日志时退 0、只在 stderr 给这句（同 .claude/hooks/session-start.sh）
MEBIBYTE=$((1024 * 1024))
DEFAULT_RESERVE="20G"                 # 依据见文件头「余量」一段
DEFAULT_WAIT_SECONDS=3600             # 依据见文件头「排队」一段
DEFAULT_KILL_GRACE_SECONDS=30
SLICE_HIGH_PERMILLE=950                         # slice 的 memory.high 与盯账的触发线 = memory.max 的这么多千分之一（文件头「盯账」一段）
OVER_BOOKING_TOLERANCE_BYTES=$((16 * 1024 * 1024))   # 超出记账过了这么多才算超账：盯账停谁、内核挤掉的记不记撞过顶都按它（文件头「盯账」一段）
OVER_BOOKING_POLL_SECONDS=0.05                  # 盯账隔多久看一次
BROKEN_JUDGEMENT="${RUN_WITH_MEMORY_CAP_BREAK:-}"
# 起 scope 的 systemd-run 放后台（文件头「被停」一段），脚本里 & 起的命令一律忽略 INT、进了 bash 也 trap 不上：包装自己进来时 INT 没被忽略的，给它设回默认；
# 进来时就被忽略的（调用方有意如此）照旧忽略
SCOPE_RUNNER_INT_OPTION=(--default-signal=INT)
[[ "$(trap -p INT)" != *"''"* ]] || SCOPE_RUNNER_INT_OPTION=()
SCOPE_RESULT_POLL_ATTEMPTS=50        # 查 scope 结局最多查这么多次，每次隔 0.1 秒：命令刚退出时 scope 可能还没收尾
CAP_MODE=run                          # run_capped 的 mode（run：真要跑命令；check：--check 只试上限）。adopt_existing_slice_total 据此决定打不打「照 slice … 已有的总上限」那一行
NESTED_IN_OUTER_SCOPE=0               # 1：这一条套在外层包装的 scope 里（文件头「套在包装里」一段），不另记账、不排队
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# 弄坏开关只在自证里认（文件头「弄坏开关只在自证里认」一段）：平常跑设了就在读参数之前退 2；弄坏开关 guard-off 下不拒（改前的样子）
SELFTEST_MARK_CONTENT="run-with-memory-cap.sh --selftest 起的包装"
if [[ -n "$BROKEN_JUDGEMENT" && "$BROKEN_JUDGEMENT" != guard-off && "${1:-}" != --selftest ]] \
    && [[ -z "${RUN_WITH_MEMORY_CAP_SELFTEST_MARK:-}" || "$(head -n 1 -- "$RUN_WITH_MEMORY_CAP_SELFTEST_MARK" 2>/dev/null)" != "$SELFTEST_MARK_CONTENT" ]]; then
  echo "  ✗ RUN_WITH_MEMORY_CAP_BREAK=$BROKEN_JUDGEMENT 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次命令一行都没跑（退 2）" >&2
  echo "     → 怎么办：去掉 RUN_WITH_MEMORY_CAP_BREAK 再跑；要证红就跑 RUN_WITH_MEMORY_CAP_BREAK=<项> bash scripts/hosts/run-with-memory-cap.sh --selftest；调用方的自证要把它传给包装的，照文件头那一段建标记文件、export RUN_WITH_MEMORY_CAP_SELFTEST_MARK" >&2
  exit 2
fi
# 只给自证的两个开关（文件头「其余环境变量」）：带着自证的标记才认，平常跑设了不起作用
SELFTEST_KERNEL_ONLY=0 SELFTEST_WATCH_TRACE=""
if [[ -n "${RUN_WITH_MEMORY_CAP_SELFTEST_MARK:-}" && "$(head -n 1 -- "$RUN_WITH_MEMORY_CAP_SELFTEST_MARK" 2>/dev/null)" == "$SELFTEST_MARK_CONTENT" ]]; then
  [[ -z "${RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY:-}" ]] || SELFTEST_KERNEL_ONLY=1
  SELFTEST_WATCH_TRACE="${RUN_WITH_MEMORY_CAP_SELFTEST_WATCH_TRACE:-}"
fi
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIRECTORY/heavy-threads.sh"     # heavy_thread_default：峰值表的线程数一列认 capped.sh default
SELF_PATH="$SCRIPT_DIRECTORY/$(basename "${BASH_SOURCE[0]}")"
SLICE="${RUN_WITH_MEMORY_CAP_SLICE:-singlefs-heavy.slice}"
RESERVE="${RUN_WITH_MEMORY_CAP_RESERVE:-$DEFAULT_RESERVE}"
SLICE_TOTAL_OVERRIDE="${RUN_WITH_MEMORY_CAP_SLICE_TOTAL:-}"
CGROUP_ROOT="${RUN_WITH_MEMORY_CAP_CGROUP_ROOT:-/sys/fs/cgroup}"
STATE_DIRECTORY="${RUN_WITH_MEMORY_CAP_STATE_DIR:-${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/singlefs-heavy-admission}"
TIME_LIMIT_SECONDS="${RUN_WITH_MEMORY_CAP_TIME_LIMIT:-}"
KILL_GRACE_SECONDS="${RUN_WITH_MEMORY_CAP_KILL_GRACE:-$DEFAULT_KILL_GRACE_SECONDS}"
JOURNALCTL="${RUN_WITH_MEMORY_CAP_JOURNALCTL:-journalctl}"   # 只给自证换成假的（文件头「显存不够」一段）
TEMPORARY_ROOT="${RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT:-/tmp}"   # 这一条命令自己的临时目录建在它下面（文件头「这一条命令自己的临时目录」一段）
# 弄坏开关 dailynoqueue：照撤之前那样，从会话里起、不带 SINGLEFS_HEAVY_TESTS 的默认不排队（文件头「日常一条」一段）
if [[ "$BROKEN_JUDGEMENT" == "dailynoqueue" && -z "${SINGLEFS_HEAVY_TESTS:-}" && -n "${CLAUDE_CODE_SESSION_ID:-}" ]]; then DEFAULT_WAIT_SECONDS=0; fi
WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-$DEFAULT_WAIT_SECONDS}"
PRIVATE_CONFIG="${RUN_WITH_MEMORY_CAP_PRIVATE_CONFIG:-${XDG_CONFIG_HOME:-${HOME:-}/.config}/singlefs/run-with-memory-cap.env}"   # 文件头「余量」一段
# 在 scope 里跑的外壳：命令不 exec，跑完读自己 cgroup 的 memory.peak 与 memory.events 写进峰值文件（scope 一收尾 cgroup 就删了）。
# TERM / INT 设成记一笔的处理函数而不是忽略：忽略会被命令继承（exec 之后忽略的信号仍忽略），处理函数不会；
# 撞顶之后 systemd 停 scope 发的 TERM 于是只打断命令，外壳读完、写完再退，退出码照传命令的。
# 命令退出之后改成忽略 TERM / INT（后面不再起子进程，没有谁会继承它），只用 bash 的内建命令读、写（read、printf），不起 cut / cat / grep / mv：
# systemd 停 scope 时给 scope 里每个进程都发 TERM，那一刻正在跑的外部命令会被它杀掉、内建的 read 会被打断，峰值文件就写不全，
# 被总上限挤掉的那一条会被报成撞了自己的上限（2026-09-25 门禁 code-tooling 里撞上过一次）。
# 外壳自己的 stderr 指到 /dev/null、命令的 stderr 经 fd 4 接回原处：命令被信号杀掉时 bash 会补一行「line 6: … Killed "$@"」，那一行不是命令的输出。
# 第 4 个参数不空：起命令之前把外壳自己的 oom_score_adj 写成它，命令随之继承（文件头「排队」一段「撞 slice 总上限时内核杀谁」）。
SCOPE_HOLDER='trap "stop_requested=1" TERM INT
: > "$1"
peak_file="$2"
cgroup_root="$3"
oom_score_adjust="$4"
shift 4
exec 4>&2 2>/dev/null
[[ -z "$oom_score_adjust" ]] || printf "%s\n" "$oom_score_adjust" > /proc/self/oom_score_adj
"$@" 2>&4 4>&-
command_exit=$?
trap "" TERM INT
own_cgroup_line="" peak="" own_oom="" oom_kill=""
IFS= read -r own_cgroup_line < /proc/self/cgroup
own_group="$cgroup_root${own_cgroup_line#0::}"
IFS= read -r peak < "$own_group/memory.peak"
while read -r event_name event_count; do
  case "$event_name" in
    oom) own_oom="$event_count" ;;
    oom_kill) oom_kill="$event_count" ;;
  esac
done < "$own_group/memory.events"
printf "peak %s\noom %s\noom_kill %s\n" "$peak" "$own_oom" "$oom_kill" > "$peak_file"
exit "$command_exit"'

# 用法错退 251（「命令一行都没跑」那一类）：退 2 就落进 0–249，与那条命令自己的退出码分不开（文件头「退出码」一段）
reject_usage() {
  echo "  ✗ $1" >&2
  echo "  → 怎么办：$2" >&2
  exit "$MEMORY_CAP_UNAVAILABLE_EXIT"
}

report_unavailable() {
  echo "  ✗ 带内存上限的 scope 起不来：$1" >&2
  echo "  → 怎么办：$2" >&2
  exit "$MEMORY_CAP_UNAVAILABLE_EXIT"
}

# 这一条命令自己的临时目录（文件头「这一条命令自己的临时目录」一段）：own_temporary_directory 记着包装建的那一份，EXIT trap 里删
own_temporary_directory=""
begin_own_temporary_directory() {
  if [[ -n "${TMPDIR:-}" && "$BROKEN_JUDGEMENT" != "temporarycallerreplaced" ]]; then
    [[ "$BROKEN_JUDGEMENT" == "temporarycallerdeleted" ]] && own_temporary_directory="$TMPDIR"
    return 0
  fi
  [[ "$BROKEN_JUDGEMENT" == "temporaryshared" ]] && return 0
  if ! own_temporary_directory="$(mktemp -d "$TEMPORARY_ROOT/singlefs-run-with-memory-cap-$$-XXXXXX")"; then
    own_temporary_directory=""
    report_unavailable "建不了这一条命令自己的临时目录（mktemp -d 在 $TEMPORARY_ROOT 下失败），命令一行都没跑" \
      "看 $TEMPORARY_ROOT 在不在、可不可写、盘满没满（df -h $TEMPORARY_ROOT）；或由调用方自己设 TMPDIR（那一份归调用方删）"
  fi
  TMPDIR="$own_temporary_directory"
  export TMPDIR
}

end_own_temporary_directory() {
  local directory="$own_temporary_directory"
  [[ -n "$directory" ]] || return 0
  own_temporary_directory=""
  if [[ "${SINGLEFS_KEEP_TEST_TEMPORARY_FILES:-}" == 1 && "$BROKEN_JUDGEMENT" != "temporarykeepignored" ]]; then
    echo "run-with-memory-cap: SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：这一条命令的临时目录留着看现场：$directory" >&2
    return 0
  fi
  [[ "$BROKEN_JUDGEMENT" == "temporaryleft" ]] && return 0
  rm -rf -- "${directory:?}"
  if [[ -e "$directory" ]]; then
    echo "run-with-memory-cap: 这一条命令的临时目录 $directory 删不干净（rm -rf 之后回读还在），退出码照命令的结局" >&2
    echo "run-with-memory-cap: → 怎么办：ls -la $directory 看剩下什么、权限对不对；有进程还开着里面的文件就先 python3 scripts/hosts/run_build_directory.py holders $directory 找出来按进程号停，再手删" >&2
  fi
}

# 资源释放广播（文件头「资源释放广播」）：run_capped 在命令真跑过之后置这三样，包装退出时 broadcast_memory_released 读
memory_command_ran=0          # 1：scope 里的外壳起来过（命令跑过）
released_unit=""              # 这一条的 scope 单元名（事件的标签）
released_memory_bytes=""      # 这一条量到的峰值，没量到取账上记的要的量
broadcast_memory_released() { # broadcast_memory_released <包装的退出码>：发一条 memory-released；广播那一步的输出不往包装的 stdout / stderr 带，退出码交回给弄坏开关用
  local -a memory_option=()
  [[ "$BROKEN_JUDGEMENT" != "broadcast-none" ]] || return 0
  ((memory_command_ran)) || return 0
  if ((NESTED_IN_OUTER_SCOPE)) && [[ "$BROKEN_JUDGEMENT" != "broadcast-nested" ]]; then return 0; fi
  [[ -z "${RUN_WITH_MEMORY_CAP_STATE_DIR:-}" || -n "${RESOURCE_BROADCAST_DIRECTORY:-}" ]] || return 0
  [[ "$released_memory_bytes" =~ ^[0-9]+$ ]] && memory_option=(--memory-bytes "$released_memory_bytes")
  if [[ "$BROKEN_JUDGEMENT" == "broadcast-lines-leak" ]]; then
    python3 "$SCRIPT_DIRECTORY/resource_release_broadcast.py" emit --kind memory-released --label "${released_unit:-singlefs-memory-cap}" --exit-code "$1" \
      ${memory_option[@]+"${memory_option[@]}"} --problems-to-log < /dev/null
    return
  fi
  python3 "$SCRIPT_DIRECTORY/resource_release_broadcast.py" emit --kind memory-released --label "${released_unit:-singlefs-memory-cap}" --exit-code "$1" \
    ${memory_option[@]+"${memory_option[@]}"} --problems-to-log < /dev/null > /dev/null 2>&1
}

# EXIT trap：先删这一条命令自己的临时目录，再发资源释放广播；不调 exit，包装的退出码照旧（弄坏开关 broadcast-exit-swallowed 下照广播那一步的退出码退）
on_wrapper_exit() {
  local wrapper_exit=$? broadcast_exit=0
  end_own_temporary_directory
  broadcast_memory_released "$wrapper_exit" || broadcast_exit=$?
  if [[ "$BROKEN_JUDGEMENT" == "broadcast-exit-swallowed" ]]; then exit "$broadcast_exit"; fi
}

check_size_syntax() { # check_size_syntax <写法> <是什么>
  [[ "$1" =~ ^[1-9][0-9]*[KMGT]$ ]] \
    || reject_usage "$2要写成 正整数加 K / M / G / T，单位必写（例 16G、512M；不带单位的数 systemd 当字节数），收到的是「$1」" "照 systemd 的 MemoryMax 写法给；默认值与依据写在文件头"
}

size_in_bytes() { # size_in_bytes <正整数[KMGT]> → 字节（1024 进制，与 systemd 相同）；写法先由 check_size_syntax 验过，超过 64 位交非 0
  local number="${1%[KMGT]}" suffix="${1##*[0-9]}" multiplier=1
  case "$suffix" in
    K) multiplier=1024 ;;
    M) multiplier=$MEBIBYTE ;;
    G) multiplier=$((MEBIBYTE * 1024)) ;;
    T) multiplier=$((MEBIBYTE * MEBIBYTE)) ;;
  esac
  ((${#number} <= 18 && number <= 9223372036854775807 / multiplier)) || return 1
  echo $((number * multiplier))
}

# 套在外层包装里没有（文件头「套在包装里」一段）：两个标记都在、上限写法认得、这个进程此刻就在那个 scope 的 cgroup 里，才打外层的上限写法、交回 0；
# 不算套着交回 1。弄坏开关 nestedqueues：一律当没套（改前的跑法）；nestedbyenvonly：只看两个标记、不核 cgroup
outer_scope_cap() {
  local own_cgroup_line="" own_group
  [[ "$BROKEN_JUDGEMENT" == "nestedqueues" ]] && return 1
  [[ -n "${RUN_WITH_MEMORY_CAP_OUTER_UNIT:-}" && -n "${RUN_WITH_MEMORY_CAP_OUTER_CAP:-}" ]] || return 1
  [[ "$RUN_WITH_MEMORY_CAP_OUTER_CAP" =~ ^[1-9][0-9]*[KMGT]$ ]] && size_in_bytes "$RUN_WITH_MEMORY_CAP_OUTER_CAP" >/dev/null || return 1
  if [[ "$BROKEN_JUDGEMENT" != "nestedbyenvonly" ]]; then
    IFS= read -r own_cgroup_line < /proc/self/cgroup 2>/dev/null || return 1
    own_group="${own_cgroup_line#0::}"
    [[ "${own_group##*/}" == "$RUN_WITH_MEMORY_CAP_OUTER_UNIT.scope" ]] || return 1
  fi
  printf '%s' "$RUN_WITH_MEMORY_CAP_OUTER_CAP"
}

# slice 的总上限（字节，按 MiB 向下取整：cgroup 的 memory.max 按页取整，取整到 MiB 之后读回来与写进去的逐字相同）；
# 算不出来（MemTotal 读不到、余量比整机还大）交非 0
slice_total_bytes() {
  local memory_total_kibibytes reserve_bytes total
  if [[ -n "$SLICE_TOTAL_OVERRIDE" ]]; then
    total="$(size_in_bytes "$SLICE_TOTAL_OVERRIDE")"
  else
    memory_total_kibibytes="$(awk '$1 == "MemTotal:" {print $2}' /proc/meminfo 2>/dev/null)"
    [[ "$memory_total_kibibytes" =~ ^[0-9]+$ ]] || return 1
    reserve_bytes="$(size_in_bytes "$RESERVE")"
    total=$((memory_total_kibibytes * 1024 - reserve_bytes))
  fi
  total=$((total / MEBIBYTE * MEBIBYTE))
  ((total > 0)) || return 1
  echo "$total"
}

# slice 上还活着的使用者（文件头「余量」一段「并发调用者」）：scope 的 cgroup 目录还在的，加账上包装还活着的每一笔（刚放行、scope 还没起的也算）；
# 一行一个，没有就空。只读，不删死账（删账要在锁里，归 admission）
slice_live_users() { # slice_live_users <slice 的 cgroup 目录>
  python3 -B /dev/fd/3 "$SCRIPT_DIRECTORY" "$STATE_DIRECTORY/ledger" "$1" 2>/dev/null 3<<'PY'
import os, sys
sys.path.insert(0, sys.argv[1])
import memory_peaks_lib as peaks
seen = set()
for entry in peaks.live_ledger_entries(sys.argv[2], remove_dead=False):
    seen.add(entry.get("unit"))
    print(f"{entry.get('unit')}（账上：包装进程号 {entry.get('pid')}、会话 {entry.get('session') or '无'}、上限 {entry.get('cap')}）")
for name in sorted(os.listdir(sys.argv[3])):
    if name.endswith(".scope") and os.path.isdir(os.path.join(sys.argv[3], name)) and name[:-6] not in seen:
        print(f"{name[:-6]}（scope 还在，账上没有）")
PY
}

# 照 slice 此刻已有的总上限走（文件头「余量」一段「并发调用者」）：设好 SLICE_TOTAL_BYTES、SLICE_DIRECTORY；真要跑命令的那一支（CAP_MODE=run）往 stderr 打一行说明，
# --check（CAP_MODE=check）只试上限设不设得上、不打：mutate.sh 这类调用方开跑先调一次 --check，打了这一行就占了它自己 stderr 的头一行。
# 弄坏开关 check-reports-foreign-reserve：--check 也打这一行（改前的跑法）。
adopt_existing_slice_total() { # adopt_existing_slice_total <slice 的 cgroup 目录> <已有的总上限，字节> <原因> <来自谁，多行>
  SLICE_DIRECTORY="$1"
  if [[ "$CAP_MODE" == "run" || "$BROKEN_JUDGEMENT" == "check-reports-foreign-reserve" ]]; then
    echo "run-with-memory-cap: 照 slice $SLICE 已有的总上限 $2 字节走（自己按余量 $RESERVE 算的是 $SLICE_TOTAL_BYTES 字节，不改 slice 的 MemoryMax）：$3；来自：$(printf '%s' "$4" | paste -sd '；')" >&2
  fi
  SLICE_TOTAL_BYTES="$2"
}

# 把 slice 的总上限设上、确认 cgroup 里真的是这个数：成功时设好 SLICE_TOTAL_BYTES、SLICE_DIRECTORY；失败时设 SLICE_PROBLEM、交非 0。
# 已经是这个数就不再设（每条变异都经这里，免得一条一次 D-Bus 写）。
# 余量不同的调用者并发（文件头「余量」一段「并发调用者」）：slice 此刻的 MemoryMax 与自己算的不同、而 slice 上有活着的 scope 或账，照已有的值走、不去改它；
# 自己设完读回的是别人刚设的另一个正整数，同样照读回的走。读回不是正整数（max、读不到）才是控制器没下放，照旧报错。
# 弄坏开关 reject-foreign-reserve：两处都不采纳，读回不是自己设的就报错（改前的跑法）。
# 总上限定下来之后再设 slice 的 memory.high（ensure_slice_high）。
ensure_slice() {
  ensure_slice_total && ensure_slice_high
}

# slice 的 memory.high = 总上限的 SLICE_HIGH_PERMILLE ‰（按 MiB 向下取整，文件头「盯账」②）：越过它的分配被内核节流，盯账赶得上一口气拿满的那种分配。
# 照已有总上限走的那一支（总上限是别的调用者设的）也按那个总上限设。已经是这个数就不再设；设不上（set-property 失败、读回不是正整数）设 SLICE_PROBLEM、交非 0；
# 读回是别的正整数（另一个调用者在设与读回之间按它的总上限设了）照读回的走。
# 自证的 RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY 与弄坏开关 noslicehigh 下不设。
ensure_slice_high() {
  local wanted read_back="" control_output
  [[ "$BROKEN_JUDGEMENT" != "noslicehigh" ]] || return 0
  ((SELFTEST_KERNEL_ONLY)) && return 0
  wanted=$((SLICE_TOTAL_BYTES * SLICE_HIGH_PERMILLE / 1000 / MEBIBYTE * MEBIBYTE))
  IFS= read -r read_back < "$SLICE_DIRECTORY/memory.high" 2>/dev/null
  [[ "$read_back" == "$wanted" ]] && return 0
  if ! control_output="$(systemctl --user set-property --runtime "$SLICE" MemoryHigh="$wanted" 2>&1)"; then
    SLICE_PROBLEM="systemctl --user set-property --runtime $SLICE MemoryHigh=$wanted 失败：$control_output"
    return 1
  fi
  read_back=""
  IFS= read -r read_back < "$SLICE_DIRECTORY/memory.high" 2>/dev/null
  if [[ ! "$read_back" =~ ^[1-9][0-9]*$ ]]; then
    SLICE_PROBLEM="设了 MemoryHigh=$wanted，读回 $SLICE_DIRECTORY/memory.high 是「${read_back:-读不到}」"
    return 1
  fi
}

ensure_slice_total() {
  local cgroup_path control_output read_back existing_total live_users
  SLICE_PROBLEM=""
  if ! SLICE_TOTAL_BYTES="$(slice_total_bytes)"; then
    SLICE_PROBLEM="总上限算不出来：/proc/meminfo 的 MemTotal 读不到，或整机内存减余量 $RESERVE 已不剩（RUN_WITH_MEMORY_CAP_SLICE_TOTAL=${SLICE_TOTAL_OVERRIDE:-没设}）"
    return 1
  fi
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  if [[ -n "$cgroup_path" ]]; then
    existing_total="$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)"
    if [[ "$existing_total" == "$SLICE_TOTAL_BYTES" ]]; then
      SLICE_DIRECTORY="$CGROUP_ROOT$cgroup_path"
      return 0
    fi
    if [[ "$BROKEN_JUDGEMENT" != "reject-foreign-reserve" && "$existing_total" =~ ^[1-9][0-9]*$ ]]; then
      live_users="$(slice_live_users "$CGROUP_ROOT$cgroup_path")"
      if [[ -n "$live_users" ]]; then
        adopt_existing_slice_total "$CGROUP_ROOT$cgroup_path" "$existing_total" "slice 上有还活着的 scope 或账，总上限是它们那边的调用者设的" "$live_users"
        return 0
      fi
    fi
  fi
  if ! control_output="$(systemctl --user set-property --runtime "$SLICE" MemoryMax="$SLICE_TOTAL_BYTES" 2>&1)"; then
    SLICE_PROBLEM="systemctl --user set-property --runtime $SLICE MemoryMax=$SLICE_TOTAL_BYTES 失败：$control_output"
    return 1
  fi
  if ! control_output="$(systemctl --user start "$SLICE" 2>&1)"; then
    SLICE_PROBLEM="systemctl --user start $SLICE 失败：$control_output"
    return 1
  fi
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  if [[ -z "$cgroup_path" ]]; then
    SLICE_PROBLEM="$SLICE 起了却查不到它的 cgroup（systemctl --user show -p ControlGroup 是空的）"
    return 1
  fi
  if [[ -n "${RUN_WITH_MEMORY_CAP_SELFTEST_SLICE_PAUSE:-}" ]]; then   # 只给自证撑开「设完、读回之前」的竞态：先写标记文件，再停这么多秒
    [[ -z "${RUN_WITH_MEMORY_CAP_SELFTEST_SLICE_PAUSE_FILE:-}" ]] || : > "$RUN_WITH_MEMORY_CAP_SELFTEST_SLICE_PAUSE_FILE"
    sleep "$RUN_WITH_MEMORY_CAP_SELFTEST_SLICE_PAUSE"
  fi
  read_back="$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)"
  if [[ "$BROKEN_JUDGEMENT" != "reject-foreign-reserve" && "$read_back" != "$SLICE_TOTAL_BYTES" && "$read_back" =~ ^[1-9][0-9]*$ ]]; then
    live_users="$(slice_live_users "$CGROUP_ROOT$cgroup_path")"
    adopt_existing_slice_total "$CGROUP_ROOT$cgroup_path" "$read_back" "设完读回的是别的调用者刚设的另一个总上限" "${live_users:-还在设总上限的另一个调用者（账与 scope 都还没有）}"
    return 0
  fi
  if [[ "$read_back" != "$SLICE_TOTAL_BYTES" ]]; then
    SLICE_PROBLEM="设了 MemoryMax=$SLICE_TOTAL_BYTES，读回 $CGROUP_ROOT$cgroup_path/memory.max 是「${read_back:-读不到}」（cgroup 的 memory 控制器没下放给用户级 systemd？）"
    return 1
  fi
  SLICE_DIRECTORY="$CGROUP_ROOT$cgroup_path"
}

prepare_state_directory() {
  mkdir -p "$STATE_DIRECTORY/ledger" "$STATE_DIRECTORY/peaks" 2>/dev/null && : >> "$STATE_DIRECTORY/lock" 2>/dev/null
}

default_peak_table() {
  echo "${XDG_DATA_HOME:-$HOME/.local/share}/singlefs/memory-peaks.tsv"
}

# 峰值表的键（文件头「峰值表」一段）：取法只在 memory_peaks_lib.py 的 command_key，统一派发器取同一份
command_key() {
  python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import memory_peaks_lib as peaks; sys.stdout.write(peaks.command_key(sys.argv[3:], sys.argv[2]))' \
    "$SCRIPT_DIRECTORY" "${RUN_WITH_MEMORY_CAP_KEY:-}" "$@"
}

# 峰值表的「线程数」一列（文件头「峰值表」一段）：命令里经 capped.sh <N> 跑的取 N，经 capped.sh default 跑的取 heavy_thread_default 的值（登记值或 SINGLEFS_THREAD_CAP）；
# 否则看包装自己环境里 capped.sh 设的那几个线程变量。弄坏开关 threadsdefaultunset 下不认 default（改前的跑法）
command_threads() {
  local previous_word="" word names name value first_value="" differing=0 listing="" registered
  for word in "$@"; do
    if [[ "${previous_word##*/}" == "capped.sh" && "$word" =~ ^[1-9][0-9]*$ ]]; then
      printf '%s' "$word"
      return
    fi
    if [[ "${previous_word##*/}" == "capped.sh" && "$word" == default && "$BROKEN_JUDGEMENT" != threadsdefaultunset ]]; then
      if registered="$(heavy_thread_default 2>/dev/null)"; then printf '%s' "$registered"; else printf '判不了（取不到每台默认重活线程数）'; fi
      return
    fi
    previous_word="$word"
  done
  if ! names="$(bash "$SCRIPT_DIRECTORY/capped.sh" --print 1 2>/dev/null)"; then
    printf '判不了（读不到 %s 的线程变量名单）' "$SCRIPT_DIRECTORY/capped.sh"
    return
  fi
  while IFS='=' read -r name _; do
    value="${!name:-}"
    [[ -n "$value" ]] || continue
    listing+="${listing:+,}$name=$value"
    if [[ -z "$first_value" ]]; then first_value="$value"; elif [[ "$value" != "$first_value" ]]; then differing=1; fi
  done <<<"$names"
  if [[ -z "$listing" ]]; then
    printf '未设（整机 %s 核）' "$(nproc)"
  elif ((differing)); then
    printf '%s' "$listing"
  else
    printf '%s' "$first_value"
  fi
}

# 排队时包装收到 TERM / INT（trap 在 run_capped 里设，从排队起管到起 scope 之前）：给排队那一步的子进程发 TERM、等它退，
# 删掉账上这一笔（放行那一刻刚记上的），退 143 / 130。只发给自己起的那一个：进程号是 `&` 当场记下的（`&` 与记下之间就到的，拿 $!），
# 发之前核 /proc 里它的父进程是这个包装；已经退出被回收、进程号被别人复用的都不发。
queued_admission_pid=""        # 排队那一步的子进程号：放后台那一刻记下，wait 等到它退出就清空
queued_admission_forking=0     # 正在起排队那一步
stop_queued_admission() { # stop_queued_admission <退出码>
  local child_pid="$queued_admission_pid" stat_line
  local -a stat_fields=()
  if [[ -z "$child_pid" ]] && ((queued_admission_forking)); then
    child_pid="${!:-}"
  fi
  if [[ -n "$child_pid" ]] && IFS= read -r stat_line 2>/dev/null < "/proc/$child_pid/stat"; then
    read -r -a stat_fields <<<"${stat_line##*) }"
    if [[ "${stat_fields[1]:-}" == "$$" ]]; then
      kill -TERM "$child_pid" 2>/dev/null
      wait "$child_pid"
    fi
  fi
  rm -f -- "$STATE_DIRECTORY/ledger/$unit.json" "$STATE_DIRECTORY/ledger/.$unit.partial"
  exit "$1"
}

# 起了 scope 之后包装收到 TERM / INT（trap 在 run_capped 里设，从起 scope 管到命令退出）：先 systemctl --user stop 自己开的那个 scope 单元
# （单元名是这一条自己起的 $unit.scope），scope 里的命令随之收到 TERM；再删标记、账上这一笔与峰值文件，退 143 / 130。
# systemd-run 放后台、用 wait 等：前台等着时 bash 不跑 trap，命令在 scope 里照跑到底。
# scope 还没建出来（stop 找不到单元，systemd-run 还在起）：改给 systemd-run 那一个进程发 TERM，它在 scope 建出来之前自己就是被停的对象；
# 进程号是 `&` 当场记下的（`&` 与记下之间就到的，拿 $!），发之前核 /proc 里它的父进程是这个包装。
scope_runner_pid=""        # systemd-run 的进程号：放后台那一刻记下，wait 等到它退出就清空
scope_runner_forking=0     # 正在起 systemd-run
over_booking_watcher_pid=""   # 盯账的子进程号（文件头「盯账」①）：放后台那一刻记下，stop_over_booking_watcher 停完清空

# 停盯账的子进程：只给 `&` 当场记下的那一个进程号发 TERM（它写标记与 cgroup.kill 那一段挡着 TERM，写完才退），再等它退出
stop_over_booking_watcher() {
  [[ -n "$over_booking_watcher_pid" ]] || return 0
  kill -TERM "$over_booking_watcher_pid" 2>/dev/null
  wait "$over_booking_watcher_pid" 2>/dev/null
  over_booking_watcher_pid=""
}

stop_own_scope_then_exit() { # stop_own_scope_then_exit <退出码>
  local runner_pid="$scope_runner_pid" stat_line
  local -a stat_fields=()
  if [[ -z "$runner_pid" ]] && ((scope_runner_forking)); then
    runner_pid="${!:-}"
  fi
  stop_over_booking_watcher
  # 资源释放广播（文件头「资源释放广播」）：scope 里的外壳起来过（标记在）才算命令跑过，包装退出时照发
  if [[ "$CAP_MODE" == "run" && -e "$started_marker" ]]; then
    memory_command_ran=1 released_unit="$unit" released_memory_bytes="${admitted_need:-}"
  fi
  if [[ "$BROKEN_JUDGEMENT" != "term-leaves-scope" ]] && ! systemctl --user stop "$unit.scope" >/dev/null 2>&1; then
    if [[ -n "$runner_pid" ]] && IFS= read -r stat_line 2>/dev/null < "/proc/$runner_pid/stat"; then
      read -r -a stat_fields <<<"${stat_line##*) }"
      [[ "${stat_fields[1]:-}" == "$$" ]] && kill -TERM "$runner_pid" 2>/dev/null
    fi
  fi
  rm -f -- "$started_marker" "$STATE_DIRECTORY/ledger/$unit.json" "$peak_file" "$peak_file.partial" "$peak_file.over-booking" "$peak_file.over-booking.partial"
  exit "$1"
}

# 排队、记账、峰值表与 --status 的那一半（锁、JSON、算术在 python 里写得清楚）。第一个参数是动作：admit、finish、status。
# admit 放后台、用 wait 等：排队可能等上一小时，而前台子进程没退之前 bash 不跑 trap，包装被停时排队那一步会留成孤儿、占着调用方管道的写端
# （records/2026-09-16-subagent拆分提案.md 第四十节第 38 行）；在 wait 里 trap 当场就跑。trap 接不住的 KILL 与不设 trap 的 HUP，
# 由 admit 每轮核自己的父进程还是不是包装（第 41 行），所以 admit 必须由包装进程自己起，不能放进子 shell。finish、status 照旧在前台跑。
admission() {
  local admission_exit
  local -a python_command=(env RUN_WITH_MEMORY_CAP_STATE_DIR_RESOLVED="$STATE_DIRECTORY" RUN_WITH_MEMORY_CAP_PEAKS_RESOLVED="$PEAK_TABLE" RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED="$SCRIPT_DIRECTORY"
                           RUN_WITH_MEMORY_CAP_WAIT_SECONDS_RESOLVED="$WAIT_SECONDS" RUN_WITH_MEMORY_CAP_THREADS_RESOLVED="${command_threads_text:-}"
                           python3 /dev/fd/3 "$@")
  {
    if [[ "$1" == "admit" ]]; then
      queued_admission_forking=1
      "${python_command[@]}" &
      queued_admission_pid=$!
      wait "$queued_admission_pid"
      admission_exit=$?
      queued_admission_pid="" queued_admission_forking=0
    else
      "${python_command[@]}"
      admission_exit=$?
    fi
  } 3<<'PY'
import contextlib, dataclasses, fcntl, importlib.util, json, os, select, shlex, signal, subprocess, sys, tempfile, threading, time
from datetime import datetime, timezone
from zoneinfo import ZoneInfo

sys.dont_write_bytecode = True
sys.path.insert(0, os.environ["RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED"])
import memory_peaks_lib as peaks   # 读峰值表、读账、算回收不掉的用量与还没涨到量的那一截（统一派发器用同一份）

MEMORY_CAP_UNAVAILABLE_EXIT = 251
MEMORY_ADMISSION_REFUSED_EXIT = 252
ADMISSION_POLL_SECONDS = 1.0            # 放不下时隔多久再判一次
WAITING_REPORT_EVERY_SECONDS = 60       # 排队时隔多久往 stderr 报一行
HOST_LOOKUP_SECONDS = 15.0              # 查别的机器一趟至多等这么久；这几个常量的依据见文件头「排队时去找别的机器」
HOST_LOOKUP_PEER_TIMEOUT_SECONDS = 8    # 交给 host-resources.py 的每台时限（HOST_RESOURCES_HOST_TIMEOUT_SECONDS）
HOST_LOOKUP_SAMPLE_SECONDS = 0.2        # 交给 host-resources.py 的核占用采样间隔（HOST_RESOURCES_SAMPLE_SECONDS）
HOST_LOOKUP_EVERY_SECONDS = 60.0        # 几条一起排队时每这么多秒只查一次
HOST_LOOKUP_LOCK_SECONDS = 0.5          # 查的那一趟不阻塞地取排队锁，至多试这么久（排队那一步每轮只占锁一瞬，避开它）；取不到这一趟不查（文件头「整体兜底」）
HOST_LOOKUP_JOIN_MARGIN_SECONDS = 2.0   # 等在查的那一趟收尾至多等它的时限再加这么久，过了不再等它（文件头「整体兜底」）
HOST_LOOKUP_SELFTEST_HANDOFF_SECONDS = 10.0   # 只给自证：叫自证的占锁进程来占排队锁之后至多等它这么久
# 只给自证（「整体兜底」那几格）：RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_FAULT 点名的那一处抛它在真世界里会抛的那一种
HOST_LOOKUP_SELFTEST_FAULTS = {
    "thread-start": lambda: RuntimeError("can't start new thread（自证造的）"),
    "temp-file": lambda: OSError(28, "No space left on device（自证造的）"),
    "read-output": lambda: UnicodeDecodeError("utf-8", b"\xff", 0, 1, "invalid start byte（自证造的）"),
    "format": lambda: TypeError("unsupported format string passed to NoneType.__format__（自证造的）"),
    "wake-thread": lambda: RuntimeError("can't start new thread（自证造的，被资源释放广播叫醒那一趟）"),
}
BROADCAST_WAKE_KINDS = ("peer-run-finished",)   # 本机自己发的事件里叫醒另查的种类；别的机器推来的（source=pushed）一律叫醒（文件头「资源释放广播」）
MEMORY_QUEUE_RECORD_KIND = "memory-queue-peer-fits"   # 告警记录的种类：scripts/agents/agent-watch.py 认它
MEBIBYTE = 1024 * 1024
state_directory = os.environ["RUN_WITH_MEMORY_CAP_STATE_DIR_RESOLVED"]
ledger_directory = os.path.join(state_directory, "ledger")
lock_path = os.path.join(state_directory, "lock")
peak_table_path = os.environ["RUN_WITH_MEMORY_CAP_PEAKS_RESOLVED"]
broken = os.environ.get("RUN_WITH_MEMORY_CAP_BREAK", "")
PEAK_TABLE_HEADER = [
    "# 每条命令近几次实测的内存峰值：scripts/hosts/run-with-memory-cap.sh 建、读、写（排队时按它算「这条要的量」：近几次里最大的，近几次里撞过顶就按上限），别手改。",
    "# 口径：scope 里的外壳 bash 在命令退出之后读自己 cgroup 的 memory.peak，按 MiB 向上取整的字节数；含页缓存（cargo 写编译产物的那些）与外壳 bash 自己，偏大不偏小。",
    "#   撞了这一条自己的上限的、被盯账停了的、超出记账又被总上限挤掉的记成上限；超时、被停、没超出记账而被挤掉的不记（量到的是半截）。只在本机成立，换机器要重量，所以放在仓外的本机数据目录里。",
    "# 列（制表符分隔）：峰值字节、量的时刻（ISO 8601，Z 结尾）、那一次的上限、键（命令的各个词用空格接起来，测试二进制名里的 16 位哈希去掉；RUN_WITH_MEMORY_CAP_KEY 可以指定）、"
    "线程数（命令里 capped.sh 的 N，否则包装环境里 capped.sh 那几个线程变量的值，都没设写「未设（整机 N 核）」；加这一列之前记的行没有这一列）、"
    f"近几次（最近 {peaks.RECENT_MEASUREMENTS} 次实测，新的在前，逗号隔开，每次「<峰值字节>:hit」撞了顶或「<峰值字节>:clean」整段跑完；加这一列之前记的行没有这一列）。"
    "前三列是最近那一次。",
]
MEMORY_PEAK_GREW_RECORD_KIND = "memory-peak-grew"   # 告警记录的种类（文件头「峰值表」一段）：scripts/agents/agent-watch.py 认这三种
MEMORY_OWN_CAP_RECORD_KIND = "memory-own-cap"
MEMORY_SQUEEZED_RECORD_KIND = "memory-squeezed"
PEAK_TABLE_ALERTS_SUFFIX = ".alerts.jsonl"   # 峰值表指到默认之外时告警记录写在 <峰值表路径> 加这个后缀（文件头「峰值表」一段「告警跟着峰值表走」）


human = peaks.human
unreclaimable_bytes = peaks.unreclaimable_bytes
process_start_time = peaks.process_start_time
size_bytes = peaks.size_bytes


@contextlib.contextmanager
def admission_lock(within_seconds=None):
    """排队那把锁，交拿没拿到。within_seconds 是 None 时等到拿到为止（排队与记账）；给了秒数就不阻塞地试（LOCK_NB），至多试这么久，
    拿不到交 False、不等（查别的机器那一趟：HostLookup.take_turn，文件头「排队时去找别的机器」「整体兜底」）。弄坏开关 hostlookuplockblocking 下照旧等到拿到。"""
    if broken == "nolock":
        yield True
        return
    with open(lock_path, "a", encoding="utf-8") as handle:
        if within_seconds is None or broken == "hostlookuplockblocking":
            fcntl.flock(handle, fcntl.LOCK_EX)
        else:
            deadline = time.monotonic() + within_seconds
            while True:
                try:
                    fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    break
                except BlockingIOError:
                    if time.monotonic() >= deadline:
                        yield False
                        return
                    time.sleep(0.05)
        try:
            yield True
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def live_ledger_entries():
    """账上还在跑的每一笔；包装已经死了（memory_peaks_lib.ledger_entry_is_dead：进程号与起始时刻对不上、记成 null 的）的那一笔删掉。
    要在 admission_lock 里调（records/2026-09-16-subagent拆分提案.md 第四十节第 41 行）。"""
    return peaks.live_ledger_entries(ledger_directory, remove_dead=True, keep_dead=(broken == "keepstale"))


def read_peak_rows():
    return peaks.read_peak_rows(peak_table_path)


def peak_table_line(key, row):
    """一行：rows 里每个键是 memory_peaks_lib.read_peak_rows 那五段。没有近几次的（加第六列之前记的、没动到的）照原样写四列或五列；
    有近几次而没有线程数的，线程数写「未记」占住第五列。"""
    peak, measured, cap, threads, recent = row
    fields = [str(peak), measured, cap, key]
    if recent is not None:
        fields += [threads if threads is not None else "未记", peaks.format_recent(recent)]
    elif threads is not None:
        fields.append(threads)
    return "\t".join(fields)


def write_peak_table(rows):
    lines = PEAK_TABLE_HEADER + [peak_table_line(key, row) for key, row in sorted(rows.items())]
    temporary = f"{peak_table_path}.{os.getpid()}.partial"
    os.makedirs(os.path.dirname(os.path.abspath(peak_table_path)), exist_ok=True)
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    os.replace(temporary, peak_table_path)


def predicted_bytes(key):
    """峰值表里没有这条时 scripts/hosts/host-resources.py 对它的内存预测：它逐台判用的 host-dispatch.py 的 predict（按文件路径导入，同一份判法，
    读的是这个包装的峰值表、键就是这个包装的键）。交 (字节, "")；算不出来交 (None, 为什么)，调用方按上限算。依据见文件头「排队」一段。"""
    try:
        spec = importlib.util.spec_from_file_location("host_dispatch", os.path.join(os.environ["RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED"], "host-dispatch.py"))
        dispatch = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(dispatch)
        settings = dispatch.settings_from_environment()
        settings = dataclasses.replace(settings, peak_table=peak_table_path, environment=dict(settings.environment, RUN_WITH_MEMORY_CAP_KEY=key))
        predicted = dispatch.predict(settings, [key], False).memory_predicted
    except Exception as error:   # 导入或调用出错：不当成 0，按上限算（宁可多排队，不少记账）
        return None, f"{type(error).__name__}: {str(error)[:200]}"
    if not isinstance(predicted, int) or predicted < 0:
        return None, f"预测值不是非负整数：{predicted!r}"
    return predicted, ""


def need_of(key, cap_bytes, cap_text):
    """(这一条要的量, 一句为什么, 记账从哪来：peak-table、prediction、cap，弄坏开关 nodefault 下是 zero)；依据见文件头「排队」「峰值表」两段。
    峰值表按近几次读（memory_peaks_lib.recorded_peak，派发器的 predict 读同一份）。"""
    if broken == "ignoretable":
        return cap_bytes, f"按它的上限 {cap_text} 算（弄坏开关 ignoretable）", "cap"
    queue_by_cap = os.environ.get("RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP", "")
    if queue_by_cap and broken != "ignorequeuebycap":
        return cap_bytes, f"设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP（{queue_by_cap}），按它的上限 {cap_text} 算", "cap"
    row = read_peak_rows().get(key)
    if row is None:
        if broken == "nodefault":
            return 0, "峰值表里没有这条，按 0 算（弄坏开关 nodefault）", "zero"
        if broken == "nopredict":
            return cap_bytes, f"峰值表里没有这条，按它的上限 {cap_text} 算（弄坏开关 nopredict：不看 host-resources.py 的预测）", "cap"
        predicted, why_not = predicted_bytes(key)
        if predicted is None:
            return cap_bytes, f"峰值表里没有这条，host-resources.py 对它的预测没算出来（{why_not}），按它的上限 {cap_text} 算", "cap"
        if predicted < cap_bytes:
            return predicted, f"峰值表里没有这条，按 host-resources.py 对它的预测 {human(predicted)} 算（比它的上限 {cap_text} 小；scope 的上限照旧是 {cap_text}）", "prediction"
        return cap_bytes, f"峰值表里没有这条，按它的上限 {cap_text} 算（host-resources.py 对它的预测 {human(predicted)} 不比它小）", "cap"
    # 弄坏开关 lastpeakonly：只看最近一次实测（改前的判法）
    recorded, hit = peaks.recorded_peak(peak_table_path, key, latest_only=(broken == "lastpeakonly"))
    recent_text = "最近一次" if broken == "lastpeakonly" else f"近 {len(peaks.recent_of(row))} 次实测里"
    if hit and broken != "peakqueue":
        return cap_bytes, f"这个键撞过顶（峰值表里{recent_text}有一次撞了顶；最近一次 {row[1]} 记的 {human(row[0])}、那一次上限 {row[2]}），按它这一次的上限 {cap_text} 算", "cap"
    if broken == "nopeakmin":
        return cap_bytes, f"按它的上限 {cap_text} 算（弄坏开关 nopeakmin：峰值表里有这条也不取较小的）", "cap"
    if recorded > cap_bytes:
        return cap_bytes, f"按它的上限 {cap_text} 算（峰值表里{recent_text}最大的峰值 {human(recorded)} 比它大）", "cap"
    return recorded, f"按峰值表里{recent_text}最大的峰值 {human(recorded)}（最近一次 {row[1]} 量的）算", "peak-table"


def occupants_text(entries, slice_directory):
    if not entries:
        return ["      （账上没有在跑的；已占的全是 slice 里别的用量，多半是刚收尾还没放干净的，或不经这个包装挂进 slice 的进程）"]
    return [f"      {entry['unit']}：要 {human(entry['need'])}，此刻占 {human(unreclaimable_bytes(os.path.join(slice_directory, entry['unit'] + '.scope')) or 0)}，"
            f"包装进程 {entry['pid']}，{entry['admitted_at']} 起跑，键「{entry['key'][:120]}」" for entry in entries]


def leave_because_wrapper_gone(wrapper_pid, what_was_seen):
    """包装已经不在（被 KILL / HUP 停了）：不排了、不记账，退出——留下来就成了孤儿，占着调用方管道的写端，放行时还会在账上记一笔
    （records/2026-09-16-subagent拆分提案.md 第四十节第 41 行）。调用方的管道已经关了就不写这一行。"""
    with contextlib.suppress(OSError):
        print(f"run-with-memory-cap: 包装进程 {wrapper_pid} 已经不在（{what_was_seen}），排队这一步退出，账上不记这一笔，命令没跑", file=sys.stderr)
    return MEMORY_CAP_UNAVAILABLE_EXIT


def say(text):
    with contextlib.suppress(OSError, ValueError):   # 调用方的管道已经关了就不写
        print(text, file=sys.stderr, flush=True)


def selftest_fault(point):
    """只给自证（「整体兜底」那几格）：RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_FAULT 点名这一处时，在这里抛它在真世界里会抛的那一种（HOST_LOOKUP_SELFTEST_FAULTS）。"""
    if os.environ.get("RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_FAULT", "") == point:
        raise HOST_LOOKUP_SELFTEST_FAULTS[point]()


def tree_relative_arguments(arguments):
    """被包的命令里整词是本机仓根、或以「仓根/」起头的参数改写成树内相对路径（仓根本身写 .），别的参数原样：改去第二台的命令在第二台那棵树的树根起，
    本机仓根的绝对路径在那边不存在（文件头「排队时去找别的机器」）。仓根是这份包装所在的树，逻辑路径与符号链接解开后的路径都认。
    弄坏开关 hostlookupabsolutepath 下原样不改。"""
    if broken == "hostlookupabsolutepath":
        return list(arguments)
    top = os.path.normpath(os.path.join(os.environ["RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED"], "..", ".."))
    roots = list(dict.fromkeys([top, os.path.realpath(top)]))
    rewritten = []
    for argument in arguments:
        for root in roots:
            if argument == root or argument.startswith(root + "/"):
                argument = argument[len(root):].lstrip("/") or "."
                break
        rewritten.append(argument)
    return rewritten


class HostLookup:
    """排队时去找别的机器（文件头「排队时去找别的机器」）：现查第二台此刻放不放得下这一条，判法是 scripts/hosts/host-resources.py 的逐台判
    （host-dispatch.py 的 predict 与 judge_host），这里不另写；放得下就打改去那台的命令、写告警记录，查不到照实打一行原因。"""

    def __init__(self, unit, cap_bytes, cap_text, key, wrapper_pid, command):
        self.unit, self.cap_bytes, self.cap_text, self.key, self.wrapper_pid, self.command = unit, cap_bytes, cap_text, key, wrapper_pid, command
        self.guard = threading.Lock()
        self.process = None      # 在查的那个 host-resources.py：自己起的子进程，放行或被停时按它的进程号停
        self.thread = None
        self.stopped = False
        self.alerted = False     # 写过 phase=queued 的告警记录：放行时补一条 admitted
        self.events = None       # 收资源释放广播的 follow（文件头「资源释放广播」）：自己起的子进程，收尾时按它的进程号停
        self.event_buffer = b""
        self.wake_thread = None  # 被资源释放广播叫醒那一趟
        self.pending_wake = None # 叫醒时还有一趟在查：记下最新的（号, 种类），查完再查
        self.queued_at = time.time()   # 排队开始那一刻（begin 里改成那一刻）：交给 follow 的 --since

    # 整体兜底（文件头「整体兜底」）：begin、finish、run、query、cancel 整段包在 try 里，出了任何异常只打一行（warn），排队与退出码照旧；
    # 被 TERM 停的 SystemExit 不是 Exception，照旧往外走。弄坏开关 hostlookupnocatch 下接住了也照旧抛出去（拿掉 try 的样子）。

    def warn(self, where, error):
        """出了异常只打一行，打不出来也不往外抛。"""
        try:
            say(f"run-with-memory-cap: 没查到别的机器：{where}出了 {type(error).__name__}（{str(error)[:200]}），这一趟不查；排队与退出码照旧")
        except Exception:
            pass

    def limit(self):
        """查一趟的时限（秒）：RUN_WITH_MEMORY_CAP_HOST_LOOKUP_SECONDS 换掉 HOST_LOOKUP_SECONDS（只给自证）；写的不是正的秒数交 None。"""
        limit_text = os.environ.get("RUN_WITH_MEMORY_CAP_HOST_LOOKUP_SECONDS", "")
        try:
            limit = float(limit_text) if limit_text else HOST_LOOKUP_SECONDS
        except ValueError:
            return None
        return limit if limit > 0 else None

    def wait_thread(self):
        """等排队开始那一趟收尾：至多它的时限加 HOST_LOOKUP_JOIN_MARGIN_SECONDS 秒，过了不再等它（daemon 线程，包装退出时随之没了）。
        弄坏开关 hostlookupjoinunbounded 下等到它收尾为止。"""
        thread, self.thread = self.thread, None
        self.join_lookup(thread, "排队开始那一趟")

    def wait_wake_thread(self):
        """等被资源释放广播叫醒那一趟收尾（文件头「资源释放广播」），时限与 wait_thread 相同。"""
        thread, self.wake_thread = self.wake_thread, None
        self.join_lookup(thread, "被资源释放广播叫醒那一趟")

    def join_lookup(self, thread, what):
        """wait_thread 与 wait_wake_thread 共用的等法（不按属性名取线程：闭包认不出 getattr 变量名，判法单元会拒算）。"""
        if thread is None:
            return
        if broken == "hostlookupjoinunbounded":
            thread.join()
            return
        bound = (self.limit() or HOST_LOOKUP_SECONDS) + HOST_LOOKUP_JOIN_MARGIN_SECONDS
        thread.join(bound)
        if thread.is_alive():
            say(f"run-with-memory-cap: 没查到别的机器：{what} {bound:g} 秒还没查完，不再等它")

    def begin(self, local):
        """排队开始那一刻：放到后台线程查，排队照旧重判（不拖慢准入）。"""
        try:
            if broken == "nohostlookup":
                return
            self.queued_at = time.time()
            self.watch_events()
            if broken == "hostlookupblocking":
                self.run("queued", local)
                return
            selftest_fault("thread-start")
            thread = threading.Thread(target=self.run, args=("queued", local), daemon=True)
            thread.start()
            self.thread = thread
        except Exception as error:
            if broken == "hostlookupnocatch":
                raise
            self.warn("排队开始那一趟起线程时", error)

    def watch_events(self):
        """排队开始那一刻起一个资源释放广播的 follow（文件头「资源释放广播」）：只起这一个，收尾时只停这一个。
        RUN_WITH_MEMORY_CAP_HOST_LOOKUP 设了不起（这一条不查别的机器，叫醒了也不查）。起不来只打一行，排队照旧。"""
        try:
            if os.environ.get("RUN_WITH_MEMORY_CAP_HOST_LOOKUP", "") or self.events is not None:
                return
            script = os.path.join(os.environ["RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED"], "resource_release_broadcast.py")
            with self.guard:
                if self.stopped:
                    return
                # --since 排队开始这一刻：follow 起得慢也不漏掉起动那一会儿来的事件
                self.events = subprocess.Popen([sys.executable, "-B", script, "follow", "--since", f"{self.queued_at:.3f}"], stdout=subprocess.PIPE,
                                               stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL)
        except Exception as error:
            if broken == "hostlookupnocatch":
                raise
            self.events = None
            self.warn("起资源释放广播的 follow 时", error)

    def stop_events(self):
        """停自己起的 follow（按它的进程号），等它退出。"""
        events, self.events = self.events, None
        if events is None:
            return
        if events.poll() is None:
            events.kill()   # 只停自己起的这一个（按它的进程号）
        with contextlib.suppress(subprocess.TimeoutExpired):
            events.wait(timeout=5)
        with contextlib.suppress(OSError):
            events.stdout.close()

    def pause(self, seconds, local):
        """排队这一轮的等（文件头「资源释放广播」）：「新事件或 seconds 到点」先到者；收到该叫醒的就在后台再查一次别的机器。
        follow 没起、退了、读出了异常：只打一行（退了与异常），照旧睡满这一轮。弄坏开关 broadcast-wake-ignored 下只睡、不看广播。"""
        started = time.monotonic()
        try:
            events = self.events
            if events is None or broken == "broadcast-wake-ignored":
                time.sleep(seconds)
                return
            ready, _, _ = select.select([events.stdout.fileno()], [], [], seconds)
            chunk = os.read(events.stdout.fileno(), 65536) if ready else None
            if chunk == b"":
                code = events.poll()
                self.stop_events()
                raise RuntimeError(f"资源释放广播的 follow 退了（退出码 {code}），之后只每 {ADMISSION_POLL_SECONDS:g} 秒重判本机")
            if chunk:
                self.event_buffer += chunk
                *lines, self.event_buffer = self.event_buffer.split(b"\n")
                wake = None
                for line in lines:
                    fields = dict(word.split("=", 1) for word in line.decode("utf-8", "replace").split() if "=" in word)
                    if not fields.get("seq", "").isdigit():
                        continue
                    if fields.get("source") == "pushed" or fields.get("kind") in BROADCAST_WAKE_KINDS or broken == "broadcast-wake-any":
                        wake = (int(fields["seq"]), fields.get("kind", "?"))
                if wake is not None:
                    self.wake(wake, local)
            if self.pending_wake is not None:
                self.wake(self.pending_wake, local)
        except Exception as error:
            if broken == "hostlookupnocatch":
                raise
            self.stop_events()
            self.warn("等资源释放广播时", error)
            remaining = seconds - (time.monotonic() - started)
            if remaining > 0:
                time.sleep(remaining)

    def wake(self, woken, local):
        """收到该叫醒的资源释放广播（woken 是（号, 种类））：后台再跑一次查法；排队开始那一趟或上一次叫醒那一趟还在查的，记下最新的号，查完再查。"""
        try:
            with self.guard:
                if self.stopped:
                    return
                busy = any(thread is not None and thread.is_alive() for thread in (self.thread, self.wake_thread))
                if busy:
                    self.pending_wake = max(self.pending_wake or woken, woken)
                    return
                self.pending_wake = None
            selftest_fault("wake-thread")
            thread = threading.Thread(target=self.run, args=("queued", local, woken), daemon=True)
            thread.start()
            self.wake_thread = thread
        except Exception as error:
            if broken == "hostlookupnocatch":
                raise
            self.warn("被资源释放广播叫醒那一趟起线程时", error)

    def finish(self, local):
        """退 252 之前：先等排队开始那一趟与叫醒那一趟查完（各至多它的时限加 HOST_LOOKUP_JOIN_MARGIN_SECONDS），再现查一趟。"""
        try:
            if broken == "nohostlookup":
                return
            self.stop_events()
            self.wait_thread()
            self.wait_wake_thread()
            self.run("refused", local)
        except Exception as error:
            if broken == "hostlookupnocatch":
                raise
            self.warn("退 252 之前那一趟", error)

    def cancel(self, admitted):
        """排队这一步要退了：停掉在查的那一趟；写过 queued 告警的、又在本机放行了，补一条 admitted。"""
        try:
            with self.guard:
                self.stopped = True
                process = self.process
            if process is not None and process.poll() is None:
                process.kill()   # 只停自己起的这一个（按它的进程号）；它起的 ssh 由 ssh 自己收尾
            self.stop_events()
            self.wait_thread()
            self.wait_wake_thread()
            if admitted and self.alerted:
                self.record("admitted", {}, {}, f"run-with-memory-cap: 排了队之后在本机起跑了（{self.unit}）", "")
        except Exception as error:
            if broken == "hostlookupnocatch":
                raise
            self.warn("收尾时", error)

    def take_turn(self, woken=None):
        """几条一起排队时每 HOST_LOOKUP_EVERY_SECONDS 秒只查一次：轮到交空串，轮不到交原因。同一个包装的两趟不互相挡。
        被资源释放广播叫醒的那一趟（woken 是（号, 种类））不看 60 秒，按事件去重：同一个号别的包装已经查过就不查（文件头「资源释放广播」）。
        排队锁不阻塞地取（至多试 HOST_LOOKUP_LOCK_SECONDS 秒），被别的进程占着就交「锁被占着」的原因，这一趟不查（文件头「整体兜底」）。"""
        if broken == "hostlookupunthrottled" and woken is None:
            return ""
        path = os.path.join(state_directory, "host-lookup.json")
        handoff = os.environ.get("RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_LOCK_HANDOFF", "")
        if handoff:   # 只给自证「锁被别的进程占着」那一格：叫自证的占锁进程来占住排队锁，等它占上了再取（真卡住的世界，不抛异常冒充）
            with open(os.path.join(handoff, "request"), "w", encoding="utf-8"):
                pass
            deadline = time.monotonic() + HOST_LOOKUP_SELFTEST_HANDOFF_SECONDS
            while not os.path.exists(os.path.join(handoff, "taken")) and time.monotonic() < deadline:
                time.sleep(0.05)
        try:
            with admission_lock(within_seconds=HOST_LOOKUP_LOCK_SECONDS) as locked:
                if not locked:
                    return (f"排队锁 {lock_path} 被别的进程占着（{HOST_LOOKUP_LOCK_SECONDS:g} 秒内没拿到），这一趟不查；"
                            "要现查：python3 scripts/hosts/host-resources.py -- <这条命令>")
                try:
                    with open(path, encoding="utf-8") as handle:
                        last = json.load(handle)
                except (OSError, ValueError):
                    last = {}
                if not isinstance(last, dict):
                    last = {}
                now = time.time()
                last_event = last.get("event") if isinstance(last.get("event"), int) else 0
                someone_else = isinstance(last.get("pid"), int) and last["pid"] != self.wrapper_pid
                if woken is not None:
                    if someone_else and last_event >= woken[0] and broken != "broadcast-wake-unthrottled":
                        return (f"第 {woken[0]} 号资源释放广播包装 {last['pid']} 已经查过（几条一起排队时一个事件只查一次，看那一条打的；"
                                f"要现查：python3 scripts/hosts/host-resources.py -- <这条命令>）")
                    last_event = max(last_event, woken[0])
                elif someone_else and isinstance(last.get("at"), (int, float)) and 0 <= now - last["at"] < HOST_LOOKUP_EVERY_SECONDS:
                    return (f"{now - last['at']:.0f} 秒前包装 {last['pid']} 刚查过（几条一起排队时每 {HOST_LOOKUP_EVERY_SECONDS:g} 秒只查一次，看那一条打的；"
                            f"要现查：python3 scripts/hosts/host-resources.py -- <这条命令>）")
                temporary = f"{path}.{os.getpid()}.partial"
                with open(temporary, "w", encoding="utf-8") as handle:
                    json.dump({"pid": self.wrapper_pid, "at": now, "event": last_event}, handle)
                os.replace(temporary, path)
        except OSError:
            pass   # 记不下谁查过：照查，宁可多查一次
        return ""

    def query(self, woken=None):
        """(查不到的原因, 第二台那一份)：原因是空串时第二台那一份是 dict（name、available、need、fits、memory_cap、why）。
        woken 是被资源释放广播叫醒时的（号, 种类），交给 take_turn 按事件去重。
        整段包在 try 里（文件头「整体兜底」）：出了异常交「查的时候出了 …」当原因，由 run 打成一行。"""
        try:
            switch = os.environ.get("RUN_WITH_MEMORY_CAP_HOST_LOOKUP", "")
            if switch:
                return f"RUN_WITH_MEMORY_CAP_HOST_LOOKUP={switch}，这一条不查", None
            sleep_text = os.environ.get("RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_SLEEP", "")
            if sleep_text and threading.current_thread() is not threading.main_thread():
                time.sleep(float(sleep_text))   # 只给自证「线程里的 query 睡着」那一格：后台那一趟真睡着（不抛异常冒充）
            limit = self.limit()
            if limit is None:
                return f"RUN_WITH_MEMORY_CAP_HOST_LOOKUP_SECONDS 要写正的秒数，收到的是「{os.environ.get('RUN_WITH_MEMORY_CAP_HOST_LOOKUP_SECONDS', '')}」", None
            problem = self.take_turn(woken)
            if problem:
                return problem, None
            script = os.path.join(os.environ["RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED"], "host-resources.py")
            environment = dict(os.environ, RUN_WITH_MEMORY_CAP_KEY=self.key, HOST_RESOURCES_HOST_TIMEOUT_SECONDS=str(HOST_LOOKUP_PEER_TIMEOUT_SECONDS),
                               HOST_RESOURCES_SAMPLE_SECONDS=str(HOST_LOOKUP_SAMPLE_SECONDS))
            # 输出落进临时文件：到点停它时不被它留下的子进程（ssh）握着的管道卡住
            selftest_fault("temp-file")
            with tempfile.TemporaryFile(mode="w+", encoding="utf-8") as output, tempfile.TemporaryFile(mode="w+", encoding="utf-8") as errors:
                with self.guard:
                    if self.stopped:
                        return "排队这一步已经要退了，不查", None
                    try:
                        process = self.process = subprocess.Popen([sys.executable, "-B", script, "--json", "--", *self.command], stdout=output, stderr=errors,
                                                                  stdin=subprocess.DEVNULL, env=environment)
                    except OSError as error:
                        return f"起不了 {script}（{error}）", None
                try:
                    code = process.wait(timeout=None if broken == "hostlookupnotimeout" else limit)
                except subprocess.TimeoutExpired:
                    process.kill()   # 只停自己起的这一个（按它的进程号）
                    process.wait()
                    return f"{limit:g} 秒没查完（停了 host-resources.py；它起的 ssh 由 ssh 自己收尾）", None
                output.seek(0)
                errors.seek(0)
                selftest_fault("read-output")
                stdout, stderr = output.read(), errors.read().strip()
            if code not in (0, 1):
                tail = stderr.splitlines()[-1][:200] if stderr else ""
                return f"host-resources.py 退 {code}" + (f"：{tail}" if tail else ""), None
            try:
                result = json.loads(stdout)
                if result.get("local_only_because"):
                    return result["local_only_because"], None
                peer = next((host for host in result["hosts"] if host.get("is_peer")), None)
                if peer is None:
                    return "多机配置里没有第二台（PEER_SSH_HOST）", None
                verdict = next(verdict for verdict in result["command"]["verdicts"] if verdict["name"] == peer["name"])
                if not verdict.get("read"):
                    return f"{peer['name']}{verdict['why']}", None
                return "", {"name": peer["name"], "available": peer["memory"]["available"], "need": result["command"]["prediction"]["memory"],
                            "fits": bool(verdict["fits"]), "memory_cap": peer.get("memory_cap"), "why": verdict.get("why", "")}
            except (ValueError, KeyError, TypeError, AttributeError, StopIteration) as error:
                return f"host-resources.py 的输出读不出（{type(error).__name__}）", None
        except Exception as error:
            if broken == "hostlookupnocatch":
                raise
            return f"查的时候出了 {type(error).__name__}（{str(error)[:200]}）", None

    def run(self, phase, local, woken=None):
        try:
            self.run_unguarded(phase, local, woken)
        except Exception as error:
            if broken == "hostlookupnocatch":
                raise
            with self.guard:
                if self.stopped:
                    return
            self.warn("被资源释放广播叫醒那一趟" if woken else f"{'排队开始' if phase == 'queued' else '退 252 之前'}那一趟", error)

    def run_unguarded(self, phase, local, woken=None):
        problem, peer = self.query(woken)
        with self.guard:
            if self.stopped:
                return
        cause = f"（被资源释放广播叫醒：第 {woken[0]} 号，{woken[1]}）" if woken else ""
        if problem:
            say(f"run-with-memory-cap: 没查到别的机器：{problem}{cause}")
            return
        if not peer["fits"]:
            say(f"run-with-memory-cap: 现查别的机器：{peer['name']}此刻也放不下（{peer['why']}）{cause}")
            return
        cap_text, cap_note = self.cap_text, ""
        peer_cap = size_bytes(peer["memory_cap"] or "")
        if peer_cap is not None and peer_cap < self.cap_bytes:
            cap_text, cap_note = peer["memory_cap"], f"（第二台的上限至多 {peer['memory_cap']}，照它写）"
        label = self.unit.replace("singlefs-memory-cap-", "moved-from-queue-", 1)
        command = f"bash scripts/hosts/multi-host-run.sh --peer --memory-cap {cap_text} --label {label} -- {shlex.join(tree_relative_arguments(self.command))}"
        selftest_fault("format")
        line = f"  → {peer['name']}此刻放得下（可用 {human(peer['available'])}、要 {human(peer['need'])}）：改用 {command}{cap_note}{cause}"
        say(line)
        if broken != "hostlookupnorecord":
            self.record(phase, local, peer, line, f"本机内存排队、{peer['name']}放得下：改用 {command}{cap_note}；还要在本机跑就等它排上（退了 252 的那条要重起）")

    def record(self, phase, local, peer, line, next_step):
        """告警记录（写法照 scripts/hosts/core_usage.py 文件头「告警记录」，经它的 alerts_path 与 append_line 写），agent-watch.py 读它叫醒主 agent。"""
        try:
            import core_usage
            path = alerts_record_path()
            if path is None:
                raise OSError("读不到工作根那份库，不知道告警记录放哪")
            entry = {"time": round(time.time(), 3), "date": datetime.now(ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d"),
                     "session_id": os.environ.get("CLAUDE_CODE_SESSION_ID", ""), "source": "run-with-memory-cap.sh", "kind": MEMORY_QUEUE_RECORD_KIND,
                     "phase": phase, "subject": "本机内存排队", "label": self.unit, "alert_pid": self.wrapper_pid, "command": shlex.join(self.command)[:300],
                     "cap": self.cap_text, "local": local, "peer": peer, "line": line.strip(), "next_step": next_step}
            core_usage.append_line(path, json.dumps(entry, ensure_ascii=False, separators=(",", ":")))
        except Exception as error:   # 导入失败、写不进：只说一句，排队照旧
            say(f"  ! 告警记录写不进（{type(error).__name__}: {error}）：看门狗读不到这一条，靠上面那一行叫人看；设 SINGLEFS_CORE_USAGE_ALERTS 指到可写的文件")
            return
        if phase == "queued":
            self.alerted = True


def admit(unit, cap_bytes, cap_text, total_bytes, slice_directory, key, wrapper_pid, lookup):
    os.makedirs(ledger_directory, exist_ok=True)
    wait_seconds = float(os.environ["RUN_WITH_MEMORY_CAP_WAIT_SECONDS_RESOLVED"])
    pause_seconds = float(os.environ.get("RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE") or 0)
    with admission_lock():
        need, need_text, need_source = need_of(key, cap_bytes, cap_text)
    if need > total_bytes:
        print(f"  ✗ 内存不够排不上：这一条要 {human(need)}（{need_text}），比 slice 的总上限 {human(total_bytes)} 还大，等多久都放不下；命令没跑", file=sys.stderr)
        print("  → 怎么办：上限写小一点再跑（真要这么多就先调小余量 RUN_WITH_MEMORY_CAP_RESERVE，依据写在 scripts/hosts/run-with-memory-cap.sh 文件头）", file=sys.stderr)
        lookup.finish({"need": need, "slice_total": total_bytes})
        return MEMORY_ADMISSION_REFUSED_EXIT
    started = time.monotonic()
    last_report = None
    session = os.environ.get("CLAUDE_CODE_SESSION_ID", "")
    if broken == "sessiononejob" and session:   # 弄坏开关：把 2026-10-06 撤了的「同一个会话同时只许一条」装回去（文件头「日常一条」一段）
        with admission_lock():
            same_session = [entry for entry in live_ledger_entries() if entry.get("session") == session]
        if same_session:
            print(f"  ✗ 这个会话（{session}）已经有一条在跑（弄坏开关 sessiononejob 装回的手续拒绝）；命令没跑", file=sys.stderr)
            print("  → 怎么办：这个开关只给自证判红用，去掉 RUN_WITH_MEMORY_CAP_BREAK 再跑", file=sys.stderr)
            return MEMORY_ADMISSION_REFUSED_EXIT
    while True:
        if os.getppid() != wrapper_pid:
            return leave_because_wrapper_gone(wrapper_pid, f"排队这一步的父进程成了 {os.getppid()}")
        refused = None   # 等多久都放不下、当场退 252 的那两种：本机的数（锁外现查别的机器，查的那一步也要拿这把锁）
        with admission_lock():
            entries = live_ledger_entries()
            slice_used = unreclaimable_bytes(slice_directory)
            if slice_used is None:
                print(f"  ✗ 读不了 slice 的用量（{slice_directory} 下的 memory.current / memory.stat），判不了放不放得下；命令没跑", file=sys.stderr)
                print("  → 怎么办：systemctl --user status 看用户级 systemd 还在不在、slice 还在不在；修好再跑，别绕开排队去跑", file=sys.stderr)
                return MEMORY_CAP_UNAVAILABLE_EXIT
            reserved = 0 if broken == "noledger" else peaks.pending_bytes(entries, slice_directory)
            occupied = slice_used + reserved
            if occupied + need <= total_bytes:
                if pause_seconds:
                    time.sleep(pause_seconds)
                wrapper_start_time = process_start_time(wrapper_pid)
                if wrapper_start_time is None:
                    return leave_because_wrapper_gone(wrapper_pid, "放行那一刻读不到它的起始时刻")
                entry = {"unit": unit, "pid": wrapper_pid, "pid_start_time": wrapper_start_time, "need": need, "cap": cap_text,
                         "key": key, "admitted_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
                         "session": session, "need_source": need_source}
                temporary = os.path.join(ledger_directory, f".{unit}.partial")
                with open(temporary, "w", encoding="utf-8") as handle:
                    json.dump(entry, handle, ensure_ascii=False)
                os.replace(temporary, os.path.join(ledger_directory, unit + ".json"))
                if last_report is not None:
                    print(f"run-with-memory-cap: 排了 {time.monotonic() - started:.0f} 秒，起跑（{unit}）", file=sys.stderr)
                return 0
            if not entries and broken != "queuehopeless":
                fits_at_most = max(0, total_bytes - slice_used)
                print(f"  ✗ 内存不够排不上：这一条要 {human(need)}（{need_text}），slice 里回收不掉的用量 {human(slice_used)}，合起来超过总上限 {human(total_bytes)}；"
                      "账上没有在跑的，没有哪一条跑完会放出来，等多久都放不下，不排队；命令没跑", file=sys.stderr)
                print(f"  → 怎么办：上限写小一点再跑（要 {human(need)}、放得下的最多 {human(fits_at_most)}，即上限不超过 {fits_at_most // MEBIBYTE}M）；"
                      f"slice 里那 {human(slice_used)} 不在账上，bash scripts/hosts/run-with-memory-cap.sh --status 看是谁；"
                      "真要这么多就先调小余量 RUN_WITH_MEMORY_CAP_RESERVE（依据写在文件头）", file=sys.stderr)
                refused = {"need": need, "slice_unreclaimable": slice_used, "slice_total": total_bytes}
            resident = None if refused else peaks.resident_bytes(slice_directory, slice_used)
            if not refused and broken != "queuebaseline" and need + resident > total_bytes:
                fits_at_most = max(0, total_bytes - resident)
                print(f"  ✗ 内存不够排不上：这一条要 {human(need)}（{need_text}），slice 里不属于任何 scope 的常驻用量 {human(resident)}（账上在跑的 {len(entries)} 条跑完它也不会少），"
                      f"合起来超过总上限 {human(total_bytes)}，等多久都放不下，不排队；命令没跑", file=sys.stderr)
                print(f"  → 怎么办：上限写小一点再跑（要 {human(need)}、放得下的最多 {human(fits_at_most)}，即上限不超过 {fits_at_most // MEBIBYTE}M）；"
                      f"slice 里那 {human(resident)} 常驻是谁，bash scripts/hosts/run-with-memory-cap.sh --status 看；真要这么多就先调小余量 RUN_WITH_MEMORY_CAP_RESERVE（依据写在文件头）", file=sys.stderr)
                refused = {"need": need, "slice_resident": resident, "slice_total": total_bytes}
        if refused:
            lookup.finish(refused)
            return MEMORY_ADMISSION_REFUSED_EXIT
        now = time.monotonic()
        if now - started >= wait_seconds:
            print(f"  ✗ 内存不够排不上：等了 {now - started:.0f} 秒（上限 RUN_WITH_MEMORY_CAP_WAIT_SECONDS={wait_seconds:g}），slice 已占 {human(occupied)}"
                  f"（回收不掉的用量 {human(slice_used)} + 账上还没涨到量的 {human(reserved)}），这一条要 {human(need)}（{need_text}），总上限 {human(total_bytes)}；命令没跑。账上占着的：",
                  file=sys.stderr)
            for line in occupants_text(entries, slice_directory):
                print(line, file=sys.stderr)   # gate-lint:detail
            print("  → 怎么办：等上面那几条跑完再跑这一条（或把 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 设长）；这一条的上限写得比它真要的大，就写小一点；"
                  "别绕开这个包装去跑——排不上说明此刻整机放不下，硬起就是再来一次整机 OOM", file=sys.stderr)
            lookup.finish({"need": need, "slice_occupied": occupied, "slice_total": total_bytes})
            return MEMORY_ADMISSION_REFUSED_EXIT
        if last_report is None or now - last_report >= WAITING_REPORT_EVERY_SECONDS:
            first_report = last_report is None
            print(f"run-with-memory-cap: 排队（已等 {now - started:.0f} 秒，至多 {wait_seconds:g} 秒）：slice 已占 {human(occupied)}，这一条要 {human(need)}"
                  f"（{need_text}），总上限 {human(total_bytes)}；账上在跑 {len(entries)} 条", file=sys.stderr)
            last_report = now
            if first_report:   # 排队开始那一刻：后台现查别的机器（文件头「排队时去找别的机器」），起收资源释放广播的 follow（文件头「资源释放广播」）
                lookup.begin({"need": need, "slice_occupied": occupied, "slice_total": total_bytes})
        # 等「新事件或 ADMISSION_POLL_SECONDS 到点」先到者；收到该叫醒的就在后台再查一次别的机器（文件头「资源释放广播」）
        lookup.pause(ADMISSION_POLL_SECONDS, {"need": need, "slice_occupied": occupied, "slice_total": total_bytes})


def alerts_record_path():
    """这个包装写的告警记录放哪（文件头「峰值表」一段「告警跟着峰值表走」）：显式设了 SINGLEFS_CORE_USAGE_ALERTS 的照它；
    峰值表指到默认那一份之外的（按真路径比）写进它旁边的 <峰值表路径>.alerts.jsonl；否则是工作根那份真记录。读不到工作根那份库交 None。"""
    import core_usage
    if os.environ.get(core_usage.ALERTS_VARIABLE) or broken == "alertsignorepeaks":
        return core_usage.alerts_path()
    if broken == "alertspathtext":
        beside = os.path.abspath(peak_table_path) != os.path.abspath(peaks.default_peak_table())
    else:
        beside = os.path.realpath(peak_table_path) != os.path.realpath(peaks.default_peak_table())
    return os.path.abspath(peak_table_path) + PEAK_TABLE_ALERTS_SUFFIX if beside else core_usage.alerts_path()


def append_outcome_record(kind, subject, unit, wrapper_pid, fields, line, next_step):
    """告警记录（写法照 scripts/hosts/core_usage.py 文件头「告警记录」，经它的 alerts_path 与 append_line 写；文件头「峰值表」一段），
    agent-watch.py 读它叫醒主 agent。写不进只说一句，退出码照旧。"""
    try:
        import core_usage
        path = alerts_record_path()
        if path is None:
            raise OSError("读不到工作根那份库，不知道告警记录放哪")
        entry = {"time": round(time.time(), 3), "date": datetime.now(ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d"),
                 "session_id": os.environ.get("CLAUDE_CODE_SESSION_ID", ""), "source": "run-with-memory-cap.sh", "kind": kind,
                 "subject": subject, "label": unit, "alert_pid": wrapper_pid, **fields, "line": line.strip(), "next_step": next_step}
        core_usage.append_line(path, json.dumps(entry, ensure_ascii=False, separators=(",", ":")))
    except Exception as error:   # 导入失败、写不进
        say(f"  ! 告警记录写不进（{type(error).__name__}: {error}）：看门狗读不到这一条，靠上面那一行叫人看；设 SINGLEFS_CORE_USAGE_ALERTS 指到可写的文件")


def outcome_alerts(unit, key, cap_text, outcome, measured, entry, wrapper_pid):
    """文件头「峰值表」一段：一次跑完而峰值是记账的 CAP_FACTOR 倍以上（记账取自峰值表）打一行、写 memory-peak-grew；
    退 250（own-cap）写 memory-own-cap，退 254（squeezed、squeezed-marked、over-booking-stopped）写 memory-squeezed，stopped_by 分内核挤掉的（kernel）与盯账停的（over-booking-watch）。
    entry 是账上这一笔（套在外层里的没有，交 None）。"""
    booked = entry.get("need") if isinstance(entry, dict) and isinstance(entry.get("need"), int) else None
    source = entry.get("need_source") if isinstance(entry, dict) else None
    by_peak_table = source == "peak-table"
    booked_text = human(booked) if booked is not None else "没记账（套在外层包装里）"
    measured_text = human(measured) if measured is not None else "没量到"
    fields = {"key": key, "cap": cap_text, "booked": booked, "booked_source": source, "measured": measured, "by_peak_table": by_peak_table}
    if outcome == "clean":
        if broken == "nogrowthalert" or booked is None or not booked or measured is None:
            return
        if not by_peak_table and broken != "growthanysource":
            return
        ratio = measured / booked
        if ratio < peaks.CAP_FACTOR:
            return
        line = (f"run-with-memory-cap: 这一次峰值 {measured_text}，是记账 {booked_text} 的 {ratio:.1f} 倍：工作量可能变了"
                f"（键「{key[:120]}」，记账取峰值表里近几次实测最大的那个；退出码照命令自己的）")
        next_step = ("查这条命令最近的改动是不是让它要的内存变多了；变多了就照这一次的峰值给上限（峰值表已记下这一次，下一次按它记账），"
                     "同时在跑的几条按新的量分；没变多是这一次偶然涨了，不用处置")
        say(line)
        say(f"run-with-memory-cap: → {next_step}")
        append_outcome_record(MEMORY_PEAK_GREW_RECORD_KIND, "内存峰值比记账大", unit, wrapper_pid, dict(fields, ratio=round(ratio, 2)), line, next_step)
        return
    if broken == "nooutcomealert":
        return
    if os.environ.get("RUN_WITH_MEMORY_CAP_OOM_FIRST") and broken != "oomfirstalerts":
        return   # 有意撞顶的跑法不叫醒主 agent（文件头「峰值表」一段）：退出码、峰值表、撞顶标记与 stderr 照旧
    admitted_text = "按峰值表放进来的" if by_peak_table else "不是按峰值表放进来的"
    if outcome == "own-cap":
        line = f"撞了自己的上限 {cap_text}（键「{key[:120]}」，记账 {booked_text}、实测 {measured_text}，{admitted_text}）"
        next_step = ("上限给小了就照实测给大一点再跑；最近的改动让工作量变了的，先查改动；峰值表里这个键已记成撞过顶，"
                     f"之后连着 {peaks.RECENT_MEASUREMENTS} 次整段跑完之前都按上限排队")
        append_outcome_record(MEMORY_OWN_CAP_RECORD_KIND, "撞了自己的上限", unit, wrapper_pid, fields, line, next_step)
    elif outcome in ("squeezed", "squeezed-marked"):
        line = f"被总上限挤掉（键「{key[:120]}」，上限 {cap_text}、记账 {booked_text}、实测 {measured_text}，{admitted_text}）"
        next_step = "结果不算数，重跑它；常这样就 bash scripts/hosts/run-with-memory-cap.sh --status 看 slice 里同时在跑的是谁"
        append_outcome_record(MEMORY_SQUEEZED_RECORD_KIND, "被总上限挤掉", unit, wrapper_pid, dict(fields, stopped_by="kernel"), line, next_step)
    elif outcome == "over-booking-stopped":
        line = (f"超出记账最多，包装在撞总上限之前停了它（键「{key[:120]}」，上限 {cap_text}、记账 {booked_text}、实测 {measured_text}，{admitted_text}）")
        next_step = ("结果不算数，重跑它：峰值表里这个键已记成撞过顶，重跑时按上限排队；常这样说明它要的内存比峰值表里记的多了，"
                     "bash scripts/hosts/run-with-memory-cap.sh --status 看 slice 里同时在跑的是谁")
        append_outcome_record(MEMORY_SQUEEZED_RECORD_KIND, "超出记账最多被包装停了", unit, wrapper_pid, dict(fields, stopped_by="over-booking-watch"), line, next_step)


def finish(unit, key, cap_text, peak_text, outcome="", measured_text="", wrapper_pid_text=""):
    """scope 收尾之后：账上这一笔删掉，量到了峰值就记进峰值表（连同 run_capped 经环境交来的线程数），近几次往前推一次（文件头「峰值表」一段）。
    outcome（run_capped 交来的结局）：clean 命令自己退出、own-cap 撞了自己的上限、squeezed 被总上限挤掉、squeezed-marked 按峰值表放进来超出记账又被挤掉、
    over-booking-stopped 盯账停的（后两种 peak_text 是上限）、空串（弄坏开关 exitcode 那一支）不写告警；measured_text 是外壳量到的峰值，wrapper_pid_text 是包装进程号。
    峰值按 MiB 向上取整记，这一行写出来与表里记的相同就不写：整份重写要先写临时文件再改名，盘忙的时候一次要一两秒
    （2026-09-25 实测 IO 压力 full avg10 约 14% 时，写临时文件加改名 0.03–1.8 秒），而撞了上限的那几条每次记的都是上限。"""
    ledger_path = os.path.join(ledger_directory, unit + ".json")
    entry = None
    with contextlib.suppress(OSError, ValueError):
        with open(ledger_path, encoding="utf-8") as handle:
            entry = json.load(handle)
    with contextlib.suppress(FileNotFoundError):
        os.remove(ledger_path)
    measured = -(-int(measured_text) // MEBIBYTE) * MEBIBYTE if measured_text.isdigit() else None
    wrapper_pid = int(wrapper_pid_text) if wrapper_pid_text.isdigit() else os.getppid()
    if outcome:
        outcome_alerts(unit, key, cap_text, outcome, measured, entry, wrapper_pid)
    if not peak_text:
        return 0
    peak = -(-int(peak_text) // MEBIBYTE) * MEBIBYTE
    cap_bytes = size_bytes(cap_text)
    hit = outcome in ("own-cap", "squeezed-marked", "over-booking-stopped") if outcome else cap_bytes is not None and peak >= cap_bytes
    threads = os.environ.get("RUN_WITH_MEMORY_CAP_THREADS_RESOLVED") or None
    try:
        with admission_lock():
            rows = read_peak_rows()
            old = rows.get(key)
            recent = ([(peak, hit)] + (peaks.recent_of(old) if old is not None else []))[:peaks.RECENT_MEASUREMENTS]
            measured_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
            if old is not None and peak_table_line(key, old) == peak_table_line(key, (peak, old[1], cap_text, threads, recent)):
                return 0
            rows[key] = (peak, measured_at, cap_text, threads, recent)
            write_peak_table(rows)
    except OSError as error:
        print(f"run-with-memory-cap: 峰值没记进 {peak_table_path}（{error}），这一条的退出码照旧；下一次它按上限排队", file=sys.stderr)
    return 0


def meminfo():
    return peaks.meminfo()


def cgroup_stat(path, names):
    try:
        with open(path, encoding="utf-8") as handle:
            stat = dict(line.split() for line in handle)
        return sum(int(stat.get(name, 0)) for name in names)
    except (OSError, ValueError):
        return None


def private_setting(config_path, key):
    """私有配置里 key 的值；文件不在、读不了、没有这个键都返回 (None, 为什么没配)。"""
    if broken == "noprivate":
        return None, "弄坏开关 noprivate 设着"
    try:
        with open(config_path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except FileNotFoundError:
        return None, "私有配置文件不在"
    except (OSError, UnicodeDecodeError) as error:
        return None, f"私有配置文件读不了（{type(error).__name__}）"
    for line in lines:
        name, separator, value = line.strip().partition("=")
        if line.strip().startswith("#") or not separator or name.strip() != key:
            continue
        value = value.strip().strip("'\"")
        if value:
            return value, ""
    return None, f"私有配置里没有 {key}"


def status(slice_name, total_text, reserve_text, slice_directory, cgroup_root, private_config):
    memory = meminfo()
    total_memory, available = memory["MemTotal"], memory["MemAvailable"]
    print(f"  … 整机内存 {human(total_memory)}（/proc/meminfo 的 MemTotal），此刻 MemAvailable {human(available)}")
    total_bytes = int(total_text) if total_text.isdigit() else None
    print(f"  … slice {slice_name} 的总上限 {human(total_bytes) if total_bytes else '算不出来'}（整机内存 − 余量 {reserve_text}；RUN_WITH_MEMORY_CAP_RESERVE 改余量）")
    slice_used = unreclaimable_bytes(slice_directory) if slice_directory else 0
    entries = live_ledger_entries() if os.path.isdir(ledger_directory) else []
    print(f"  … slice 里回收不掉的用量 {human(slice_used or 0)}{'' if slice_directory else '（slice 还没起）'}；账上在跑 {len(entries)} 条"
          f"{'：' if entries else ''}")
    for line in occupants_text(entries, slice_directory) if entries else []:
        print(line)
    outside = total_memory - available - (slice_used or 0)
    service_cgroup, why_unset = private_setting(private_config, "OUTSIDE_SERVICE_CGROUP")
    if service_cgroup is None:
        service_text = f"没配（{why_unset}；配法见文件头「余量」一段）"
    else:
        service = cgroup_stat(os.path.join(cgroup_root, service_cgroup.lstrip("/"), "memory.stat"), ("anon", "shmem"))
        service_text = human(service) if service is not None else "读不到（私有配置里的 OUTSIDE_SERVICE_CGROUP 底下没有可读的 memory.stat）"
    user = cgroup_stat(os.path.join(cgroup_root, "user.slice/memory.stat"), ("anon",))
    print(f"  … slice 外面回收不掉的合计约 {human(outside)}（MemTotal − MemAvailable − slice 里的用量），其中本地模型服务的 anon + shmem "
          f"{service_text}、user.slice 的 anon {human(user) if user is not None else '读不到'}")
    if total_bytes is not None:
        reserve_bytes = total_memory - total_bytes
        if outside > reserve_bytes:
            print(f"  ✗ slice 外面的用量已经超过余量 {human(reserve_bytes)}：slice 的总上限挡不住整机被外面涨满")
            print("  → 怎么办：查外面是谁在涨（systemd-cgtop、ps -eo rss,pid,cmd --sort=-rss）；是常态就调大 RUN_WITH_MEMORY_CAP_RESERVE 并改文件头的依据")
            return 1
        print(f"  ✓ 余量比 slice 外面此刻的用量多 {human(reserve_bytes - outside)}")
    return 0


operation, arguments = sys.argv[1], sys.argv[2:]
if operation == "admit":
    unit, cap_bytes, cap_text, total_bytes, slice_directory, key, wrapper_pid, *command = arguments
    host_lookup = HostLookup(unit, int(cap_bytes), cap_text, key, int(wrapper_pid), command)
    # 排队时包装被 TERM 停（stop_queued_admission 只给这一步发 TERM）：先停自己在查别的机器的那个子进程再退，不留给它跑完
    signal.signal(signal.SIGTERM, lambda signal_number, frame: sys.exit(128 + signal_number))
    admit_status = MEMORY_CAP_UNAVAILABLE_EXIT
    try:
        admit_status = admit(unit, int(cap_bytes), cap_text, int(total_bytes), slice_directory, key, int(wrapper_pid), host_lookup)
    finally:
        host_lookup.cancel(admitted=(admit_status == 0))
    sys.exit(admit_status)
if operation == "finish":
    sys.exit(finish(*arguments))
if operation == "status":
    sys.exit(status(*arguments))
print(f"  ✗ admission 不认识的动作 {operation}", file=sys.stderr)
print("  → 怎么办：这是 run-with-memory-cap.sh 自己的内部调用写错了，看调 admission 的那一行", file=sys.stderr)
sys.exit(2)
PY
  return "$admission_exit"
}

# scope 的结局：命令退出之后查 systemd 记的 Result，直到 scope 收尾（不在了、inactive、failed）或查满次数。
# 打印 Result（oom-kill、timeout、success 等）；systemctl 本身出错时打印 unreadable。
# 读到 oom-kill 也要等 scope 收尾再走：它那时多半还在 deactivating，不等它落到 failed 就 reset-failed 不掉，撞顶的 scope 会一直挂着。
scope_result() {
  local unit="$1" attempt properties load_state active_state result="" oom_seen=""
  for ((attempt = 1; attempt <= SCOPE_RESULT_POLL_ATTEMPTS; attempt++)); do
    if ! properties="$(systemctl --user show -p LoadState -p ActiveState -p Result "$unit.scope" 2>/dev/null)"; then
      echo "unreadable"
      return
    fi
    load_state="$(sed -n 's/^LoadState=//p' <<<"$properties")"
    active_state="$(sed -n 's/^ActiveState=//p' <<<"$properties")"
    result="$(sed -n 's/^Result=//p' <<<"$properties")"
    [[ "$result" == "oom-kill" ]] && oom_seen="oom-kill"
    if [[ "$load_state" == "not-found" || "$active_state" == "inactive" || "$active_state" == "failed" ]]; then
      break
    fi
    sleep 0.1
  done
  [[ -n "$oom_seen" ]] && result="$oom_seen"
  if [[ "$active_state" == "failed" ]]; then
    systemctl --user reset-failed "$unit.scope" >/dev/null 2>&1   # 撞顶、超时的 scope 停在 failed，不清掉就一直挂在用户级 systemd 里
  fi
  echo "${result:-unreadable}"
}

peak_file_value() { # peak_file_value <峰值文件> <名字> → 那一行的数；没有就空
  [[ -f "$1" ]] && awk -v wanted="$2" '$1 == wanted {print $2}' "$1"
}

# run_capped <上限> <run|check> <命令…>：check 只试起不起得来，不排队、不记峰值、不限时
recorded_peak_bytes() { # recorded_peak_bytes <键> → 峰值表里这条命令的实测峰值（字节）；没有就空（读法在 memory_peaks_lib.py）
  python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import memory_peaks_lib as peaks; peak, _ = peaks.recorded_peak(sys.argv[2], sys.argv[3]); print("" if peak is None else peak)' \
    "$SCRIPT_DIRECTORY" "$PEAK_TABLE" "$1"
}

cap_judged_against_the_recorded_peak() { # cap_judged_against_the_recorded_peak <上限写法> <命令…> → 这一次用的上限写法：auto 换成数，别的原样（文件头「上限对实测峰值」一段）
  local cap="$1" key peak peak_mebibytes cap_bytes default_cap
  shift
  [[ "$cap" == auto || "$BROKEN_JUDGEMENT" == "overpeakreject" ]] || { printf '%s' "$cap"; return; }
  key="$(command_key "$@")"
  peak="$(recorded_peak_bytes "$key")"
  if [[ "$cap" == auto ]]; then
    if [[ -n "$peak" ]]; then
      printf '%sM' $(( (peak * 3 / 2 + MEBIBYTE - 1) / MEBIBYTE ))
      return
    fi
    [[ "$BROKEN_JUDGEMENT" != "autonodefault" ]] || reject_usage "上限写 auto，而峰值表 $PEAK_TABLE 里没有这条命令（键「$key」）（弄坏开关 autonodefault 装回的手续拒绝）" \
      "这个开关只给自证判红用，去掉 RUN_WITH_MEMORY_CAP_BREAK 再跑"
    default_cap="$(python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import memory_peaks_lib as peaks; print(peaks.DEFAULT_MEMORY_PREDICTION_TEXT)' "$SCRIPT_DIRECTORY")"
    echo "run-with-memory-cap: 上限写 auto，峰值表里没有这条命令（键「${key:0:120}」），取保守默认值 $default_cap（memory_peaks_lib.py 的 DEFAULT_MEMORY_PREDICTION_TEXT）" >&2
    printf '%s' "$default_cap"
    return
  fi
  # 弄坏开关 overpeakreject：把 2026-10-06 撤了的「上限超过实测峰值 2 倍就拒」装回去
  check_size_syntax "$cap" "内存上限"
  cap_bytes="$(size_in_bytes "$cap")" || reject_usage "内存上限 $cap 太大，换成字节超过 64 位" "写成真要的量，例 16G"
  if [[ -n "$peak" ]] && (( cap_bytes > 2 * peak )); then
    peak_mebibytes=$(( (peak + MEBIBYTE - 1) / MEBIBYTE ))
    reject_usage "上限 $cap 超过峰值表里这条命令实测峰值 ${peak_mebibytes}M 的 2 倍（弄坏开关 overpeakreject 装回的手续拒绝），命令一行都没跑" \
      "这个开关只给自证判红用，去掉 RUN_WITH_MEMORY_CAP_BREAK 再跑"
  fi
  printf '%s' "$cap"
}

head_characters() { # head_characters <字数> [<文件>]：按字（UTF-8）取开头这么多个字，不截出半个汉字（按字节截的 head -c、cut -c 会，输出就解不成 UTF-8）；
  # 不给文件读标准输入；文件读不了交空。解不开的字节换成 U+FFFD
  python3 -c 'import sys
data = open(sys.argv[2], "rb").read() if len(sys.argv) > 2 else sys.stdin.buffer.read()
sys.stdout.write(data.decode("utf-8", "replace")[:int(sys.argv[1])])' "$@" 2>/dev/null
}

# 显存不够（文件头「显存不够」一段）：读 journalctl -k 里 <起> 到 <止>（epoch 秒）的行，打一行结论：
#   hit <行数> <头一行>     有 NVRM … NV_ERR_NO_MEMORY 行
#   clean                  看得到内核日志、窗口里没有
#   unreadable <为什么>    读不到内核日志（不当成没撞）
gpu_memory_verdict() { # gpu_memory_verdict <起> <止>
  local probe_output="" journal_output="" journal_status=0 hits problem="" error_file pattern='NVRM.*NV_ERR_NO_MEMORY|NV_ERR_NO_MEMORY.*NVRM'
  local -a window=(--since "@$(($1 - 1))" --until "@$(($2 + GPU_JOURNAL_SLACK_SECONDS))")
  [[ "$BROKEN_JUDGEMENT" != "wholejournal" ]] || window=()
  [[ "$BROKEN_JUDGEMENT" != "anynvrm" ]] || pattern='NVRM'
  if ! command -v "$JOURNALCTL" >/dev/null 2>&1; then
    problem="这台机器上没有 journalctl"
  elif ! error_file="$(mktemp "${TMPDIR:-/tmp}/run-with-memory-cap-journal-XXXXXX" 2>/dev/null)"; then
    problem="建不了放 journalctl 报错的临时文件（${TMPDIR:-/tmp} 不可写？）"
  else
    if probe_output="$(timeout 20 "$JOURNALCTL" -k -n 1 -q --no-pager 2>"$error_file")"; then journal_status=0; else journal_status=$?; fi
    if ((journal_status != 0)); then
      problem="journalctl -k -n 1 退 $journal_status：$(head_characters 200 "$error_file" | tr '\n' ' ')"
    elif grep -qF "$JOURNAL_PERMISSION_HINT" "$error_file"; then
      problem="权限不够，journalctl 看不到内核日志（要在 adm 或 systemd-journal 组里）"
    elif [[ -z "$probe_output" ]]; then
      problem="journalctl -k 连最近一条内核消息都读不出来"
    else
      if journal_output="$(timeout 20 "$JOURNALCTL" -k -q --no-pager -o short-iso "${window[@]}" 2>"$error_file")"; then journal_status=0; else journal_status=$?; fi
      ((journal_status == 0)) || problem="journalctl -k --since/--until 退 $journal_status：$(head_characters 200 "$error_file" | tr '\n' ' ')"
    fi
    rm -f -- "$error_file"
  fi
  if [[ -n "$problem" ]]; then
    if [[ "$BROKEN_JUDGEMENT" == "journalasclean" ]]; then echo "clean"; else echo "unreadable $problem"; fi
    return
  fi
  hits="$(grep -cE "$pattern" <<<"$journal_output")"
  if ((hits > 0)); then
    echo "hit $hits $(grep -m 1 -E "$pattern" <<<"$journal_output" | head_characters 300)"
  else
    echo "clean"
  fi
}

# 撞了上限、被总上限挤掉的那几种：另报一行起止之间有没有显存不够，不改退出码
gpu_memory_note() { # gpu_memory_note <起> <止>
  local verdict
  [[ "$BROKEN_JUDGEMENT" != "nogpuoom" ]] || return 0
  verdict="$(gpu_memory_verdict "$1" "$2")"
  case "$verdict" in
    hit\ *) echo "run-with-memory-cap: 这一条起止之间内核还报了显存不够（${verdict#hit }）：显存也撞了，查清之前别照原样重派" >&2 ;;
    unreadable\ *) echo "run-with-memory-cap: 显存 OOM 没查成（${verdict#unreadable }）：不当成没撞" >&2 ;;
  esac
}

# 命令的退出码是 128 加外面常发的信号号时，交回信号号（退 0）；不是交 1
stopped_from_outside_signal() { # stopped_from_outside_signal <退出码>
  local code="$1"
  [[ "$code" =~ ^[0-9]+$ ]] && ((code > 128 && code < 160)) || return 1
  [[ "$STOPPED_FROM_OUTSIDE_SIGNALS" == *" $((code - 128)) "* ]] || return 1
  echo "$((code - 128))"
}

# 按记账算的 oom_score_adj（文件头「排队」一段「撞 slice 总上限时内核杀谁」）：1000 − 记账 × 1000 / 分母，分母 = slice 的 memory.max + min(slice 的 memory.swap.max, SwapTotal)，
# 起 scope 之前现读（swap 开关变了跟着变），小于 0 取 0。分母读不出来交空：外壳不写，命令照旧继承调用方的
booked_oom_score_adjust() { # booked_oom_score_adjust <记账，字节>
  local slice_max="" swap_max="" swap_kibibytes denominator adjust
  IFS= read -r slice_max < "$SLICE_DIRECTORY/memory.max" 2>/dev/null
  IFS= read -r swap_max < "$SLICE_DIRECTORY/memory.swap.max" 2>/dev/null
  swap_kibibytes="$(awk '$1 == "SwapTotal:" {print $2}' /proc/meminfo 2>/dev/null)"
  [[ "$slice_max" =~ ^[1-9][0-9]*$ && "$swap_kibibytes" =~ ^[0-9]+$ ]] || return 0
  denominator=$((swap_kibibytes * 1024))
  [[ "$swap_max" =~ ^[0-9]+$ ]] && ((swap_max < denominator)) && denominator=$swap_max   # max（不限）就是 SwapTotal
  denominator=$((denominator + slice_max))
  adjust=$((1000 - $1 * 1000 / denominator))
  ((adjust < 0)) && adjust=0
  echo "$adjust"
}

# 盯账（文件头「盯账」①）：放后台起，命令退出、包装被停或换了父进程时自己退；包装收尾时只给它这一个进程号发 TERM。
# 停自己那一刻先写标记文件（run_capped 认它判结局），写标记与写 cgroup.kill 那一段挡着 TERM。读法复用 memory_peaks_lib.py，不另写一份「回收不掉的用量」
watch_over_booking() { # watch_over_booking <unit> <记账，字节> <systemd-run 的进程号> <标记文件>
  exec python3 -B /dev/fd/3 "$SCRIPT_DIRECTORY" "$SLICE_DIRECTORY" "$STATE_DIRECTORY/ledger" "$1" "$2" "$3" "$$" "$4" \
    "$OVER_BOOKING_TOLERANCE_BYTES" "$SLICE_HIGH_PERMILLE" "$OVER_BOOKING_POLL_SECONDS" "$SELFTEST_WATCH_TRACE" 3<<'PY'
import os, signal, sys, time
sys.dont_write_bytecode = True
(script_directory, slice_directory, ledger_directory, unit, need_text, runner_text, wrapper_text, marker_path,
 tolerance_text, permille_text, poll_text, trace_path) = sys.argv[1:13]
sys.path.insert(0, script_directory)
import memory_peaks_lib as peaks
need, runner, wrapper = int(need_text), int(runner_text), int(wrapper_text)
tolerance, permille, poll = int(tolerance_text), int(permille_text), float(poll_text)
own = os.path.join(slice_directory, unit + ".scope")


def alive(process_id):
    try:
        os.kill(process_id, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def read_number(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return int(handle.read().strip())
    except (OSError, ValueError):
        return None


def stop_own_scope(own_used, own_excess, slice_used, trigger):
    """先写标记（run_capped 认它），再往自己的 scope 的 cgroup.kill 写 1；这一段挡着 TERM，包装收尾时停这个进程不会只做一半。"""
    signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGTERM})
    peak = read_number(os.path.join(own, "memory.peak"))
    with open(marker_path + ".partial", "w", encoding="utf-8") as handle:
        handle.write(f"excess {own_excess}\nneed {need}\nused {own_used}\nslice {slice_used}\ntrigger {trigger}\npeak {peak if peak is not None else own_used}\n")
    os.replace(marker_path + ".partial", marker_path)
    try:
        with open(os.path.join(own, "cgroup.kill"), "w", encoding="utf-8") as handle:
            handle.write("1")
    except OSError as error:
        os.remove(marker_path)
        print(f"run-with-memory-cap: 盯账停不了自己的 scope {unit}（写 cgroup.kill：{error}），撞总上限时照旧由内核按 oom_score_adj 挑", file=sys.stderr)


def watch():
    while os.getppid() == wrapper and alive(runner):
        slice_max = read_number(os.path.join(slice_directory, "memory.max"))
        slice_used = peaks.unreclaimable_bytes(slice_directory)
        if os.path.isdir(own) and slice_max and slice_used is not None and slice_used >= slice_max * permille // 1000:
            trigger = slice_max * permille // 1000
            own_used = peaks.unreclaimable_bytes(own) or 0
            own_excess = own_used - need
            rival = None   # 超出记账比自己多的（一样多的按单元名小的算多）
            for entry in peaks.live_ledger_entries(ledger_directory, remove_dead=False):
                other, other_need = entry.get("unit"), entry.get("need")
                if other == unit or not isinstance(other, str) or not isinstance(other_need, int):
                    continue
                other_directory = os.path.join(slice_directory, other + ".scope")
                if not os.path.isdir(other_directory):
                    continue
                other_excess = (peaks.unreclaimable_bytes(other_directory) or 0) - other_need
                if other_excess > own_excess or (other_excess == own_excess and other < unit):
                    rival = f"{other} 超出 {other_excess}"
                    break
            if trace_path:
                with open(trace_path, "a", encoding="utf-8") as handle:
                    handle.write(f"{unit} slice {slice_used} trigger {trigger} own {own_used} need {need} excess {own_excess} rival {rival or '无'}\n")
            if own_excess > tolerance and rival is None:
                stop_own_scope(own_used, own_excess, slice_used, trigger)
                return
        time.sleep(poll)


try:
    watch()
except Exception as error:   # 读 cgroup、读账、写标记出了没想到的事：盯账停下，撞总上限时照旧由内核按 oom_score_adj 挑
    print(f"run-with-memory-cap: 盯账出了 {type(error).__name__}: {error}，这一条不再盯账，撞总上限时照旧由内核按 oom_score_adj 挑", file=sys.stderr)
PY
}

run_capped() {
  local cap="$1" mode="$2" cap_bytes unit started_marker peak_file key command_threads_text exit_code result use_slice=1 admission_status
  local peak_bytes own_oom_count oom_kill_count scope_properties admitted_need over_admitted=0 oom_score_adjust="" command_started command_ended gpu_verdict stopped_signal
  local over_booking_marker watch_excess watch_used watch_slice watch_trigger watch_peak squeezed_marked
  shift 2
  if [[ "$BROKEN_JUDGEMENT" == "nocap" ]]; then
    "$@"
    return
  fi
  if ((NESTED_IN_OUTER_SCOPE)) && [[ "$BROKEN_JUDGEMENT" == "nestedinouterscope" ]]; then
    "$@"
    return
  fi
  CAP_MODE="$mode"
  cap_bytes="$(size_in_bytes "$cap")" || cap_bytes=0   # 只有 --check 走得到 0：跑命令的入口先判过上限不超过 64 位
  prepare_state_directory || report_unavailable "账与锁的目录 $STATE_DIRECTORY 建不了（mkdir 或写 lock 失败），命令一行都没跑" \
    "查 \$XDG_RUNTIME_DIR（$STATE_DIRECTORY 的上一层）在不在、可写不可写；或设 RUN_WITH_MEMORY_CAP_STATE_DIR 到一个可写的目录。不许绕开排队去跑"
  if [[ "$BROKEN_JUDGEMENT" == "noslice" ]]; then
    use_slice=0
  elif ! ensure_slice; then
    if [[ "$BROKEN_JUDGEMENT" == "slicefallback" ]]; then
      use_slice=0
    else
      report_unavailable "slice $SLICE 的总上限设不上：$SLICE_PROBLEM；命令一行都没跑" \
        "照上面那句修（systemctl --user status、cat /sys/fs/cgroup/user.slice/user-\$(id -u).slice/user@\$(id -u).service/cgroup.subtree_control 里要有 memory）；修好之前别跑这一步，不许退回无总上限去跑"
    fi
  fi
  unit="singlefs-memory-cap-$$-$RANDOM$RANDOM"
  key="$(command_key "$@")"
  command_threads_text="$(command_threads "$@")"
  # 套在外层包装里的（文件头「套在包装里」一段）不另记账、不排队：外层已经照它要的量占着账
  if [[ "$mode" == "run" && $use_slice -eq 1 && $NESTED_IN_OUTER_SCOPE -eq 0 ]]; then
    # 排队时被停：排队那一步跟着停、账上不留这一笔（stop_queued_admission）；起 scope 时换成下面那两个 trap
    trap 'stop_queued_admission 143' TERM
    trap 'stop_queued_admission 130' INT
    admission admit "$unit" "$cap_bytes" "$cap" "$SLICE_TOTAL_BYTES" "$SLICE_DIRECTORY" "$key" "$$" "$@"
    admission_status=$?
    if [[ $admission_status -eq $MEMORY_ADMISSION_REFUSED_EXIT ]]; then
      trap - TERM INT
      # 弄坏开关 broadcast-unadmitted：排队没放行就退的也当命令跑过、照发资源释放广播
      [[ "$BROKEN_JUDGEMENT" != "broadcast-unadmitted" ]] || { memory_command_ran=1; released_unit="$unit"; }
      return "$MEMORY_ADMISSION_REFUSED_EXIT"
    fi
    if [[ $admission_status -ne 0 ]]; then
      report_unavailable "排队那一步退 $admission_status（上面是它的报错），命令一行都没跑" "照上面那几行修；不许绕开排队去跑"
    fi
    # 撞 slice 总上限时内核挑谁（文件头「排队」一段）：账上有记账的按记账算（booked_oom_score_adjust），设了 OOM_FIRST 的 1000。
    # over_admitted：按峰值表或预测放进来的（账上记的要的量小于上限）。弄坏开关 oomadjustflat 照改前的两档（按峰值表放进来的 500、按上限放进来的不写），nooomadjust 一律不写
    admitted_need="$(sed -n 's/.*"need": \([0-9][0-9]*\).*/\1/p' "$STATE_DIRECTORY/ledger/$unit.json" 2>/dev/null)"
    if [[ "$admitted_need" =~ ^[0-9]+$ ]]; then
      ((admitted_need < cap_bytes)) && over_admitted=1
      if [[ "$BROKEN_JUDGEMENT" == "oomadjustflat" ]]; then
        ((over_admitted)) && oom_score_adjust=500
      elif [[ "$BROKEN_JUDGEMENT" != "nooomadjust" ]]; then
        oom_score_adjust="$(booked_oom_score_adjust "$admitted_need")"
      fi
    fi
    if [[ -n "${RUN_WITH_MEMORY_CAP_OOM_FIRST:-}" && "$BROKEN_JUDGEMENT" != "nooomfirst" ]]; then
      oom_score_adjust=1000
    fi
  fi
  started_marker="${TMPDIR:-/tmp}/run-with-memory-cap-$$-$RANDOM$RANDOM.started"
  peak_file="$STATE_DIRECTORY/peaks/$unit"
  over_booking_marker="$peak_file.over-booking"   # 盯账停这一条时写（文件头「盯账」一段）
  # 起了 scope 之后被停（包括被外层 timeout 连同进程组一起停）：先停自己开的 scope，再把标记、账上这一笔与峰值文件删掉（stop_own_scope_then_exit）
  trap 'stop_own_scope_then_exit 143' TERM
  trap 'stop_own_scope_then_exit 130' INT
  scope_properties=(-p MemoryMax="$cap" -p MemorySwapMax=0 -p OOMPolicy=stop)
  ((use_slice)) && scope_properties+=(--slice="$SLICE")
  if [[ "$mode" == "run" && -n "$TIME_LIMIT_SECONDS" && "$BROKEN_JUDGEMENT" != "notimelimit" ]]; then
    scope_properties+=(-p RuntimeMaxSec="$TIME_LIMIT_SECONDS" -p TimeoutStopSec="$KILL_GRACE_SECONDS")
  fi
  # 外壳里先建标记再跑命令：标记在，说明命令确实是在上限里起的；不在，说明 systemd-run 自己没起来。
  # 外面那层把本 shell 的 stderr 临时指到 /dev/null：外壳被 SIGKILL 时 bash 会补一行「… Killed  systemd-run …」，那一行不是命令的输出；
  # 命令自己的 stderr 经 fd 3 照旧接回原来的 stderr。只管这一条的三个变量在这里清掉，不传进命令；
  # 套在包装里的两个标记换成这一条自己的（文件头「套在包装里」一段）：命令里再经包装跑的认它们
  printf -v command_started '%(%s)T' -1
  # 放后台、用 wait 等（stop_own_scope_then_exit 前的注释）；<&0 让命令照旧读调用方的标准输入（后台作业默认读 /dev/null）
  scope_runner_pid=""; scope_runner_forking=1
  { env "${SCOPE_RUNNER_INT_OPTION[@]}" -u RUN_WITH_MEMORY_CAP_KEY -u RUN_WITH_MEMORY_CAP_TIME_LIMIT -u RUN_WITH_MEMORY_CAP_WAIT_SECONDS -u RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON -u RUN_WITH_MEMORY_CAP_PARALLEL_REASON \
      -u RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP -u RUN_WITH_MEMORY_CAP_OOM_FIRST -u RUN_WITH_MEMORY_CAP_JOURNALCTL \
      -u RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY -u RUN_WITH_MEMORY_CAP_SELFTEST_WATCH_TRACE \
      RUN_WITH_MEMORY_CAP_OUTER_UNIT="$unit" RUN_WITH_MEMORY_CAP_OUTER_CAP="$cap" \
      systemd-run --user --scope --unit="$unit" "${scope_properties[@]}" --quiet \
      bash -c "$SCOPE_HOLDER" run-with-memory-cap "$started_marker" "$peak_file" "$CGROUP_ROOT" "$oom_score_adjust" "$@" 2>&3 3>&-; } 3>&2 2>/dev/null <&0 &
  scope_runner_pid=$!; scope_runner_forking=0
  # 盯账（文件头「盯账」①）：账上有记账的才盯（套在外层里的、--check 不盯）；自证的 KERNEL_ONLY 与弄坏开关 nowatch 下不起
  over_booking_watcher_pid=""
  if [[ "$mode" == "run" && $use_slice -eq 1 && "${admitted_need:-}" =~ ^[0-9]+$ && "$BROKEN_JUDGEMENT" != "nowatch" ]] && ((SELFTEST_KERNEL_ONLY == 0)); then
    watch_over_booking "$unit" "$admitted_need" "$scope_runner_pid" "$over_booking_marker" &
    over_booking_watcher_pid=$!
  fi
  wait "$scope_runner_pid" 2>/dev/null
  exit_code=$?
  scope_runner_pid=""
  stop_over_booking_watcher
  printf -v command_ended '%(%s)T' -1
  if [[ ! -e "$started_marker" ]]; then
    rm -f -- "$STATE_DIRECTORY/ledger/$unit.json" "$peak_file" "$peak_file.partial" "$over_booking_marker" "$over_booking_marker.partial"
    trap - TERM INT
    if [[ "$BROKEN_JUDGEMENT" == "fallback" ]]; then
      "$@"
      return
    fi
    report_unavailable "systemd-run --user --scope 退出码 $exit_code，命令一行都没跑（上面是 systemd-run 自己的报错）" \
      "查用户级 systemd 与 D-Bus（systemctl --user status、echo \$DBUS_SESSION_BUS_ADDRESS \$XDG_RUNTIME_DIR）；起不来就别跑这一步，不许退回无上限去跑"
  fi
  rm -f -- "$started_marker"
  trap - TERM INT
  [[ "$mode" == "run" ]] && { memory_command_ran=1; released_unit="$unit"; }   # 资源释放广播：命令跑过（文件头「资源释放广播」）
  result="$(scope_result "$unit")"
  peak_bytes="$(peak_file_value "$peak_file" peak)"
  released_memory_bytes="${peak_bytes:-${admitted_need:-}}"
  own_oom_count="$(peak_file_value "$peak_file" oom)"
  oom_kill_count="$(peak_file_value "$peak_file" oom_kill)"
  rm -f -- "$peak_file" "$peak_file.partial"
  if [[ "$mode" != "run" ]]; then
    rm -f -- "$STATE_DIRECTORY/ledger/$unit.json"
  fi
  # 盯账停的（文件头「盯账」一段）：标记在就是包装自己在撞总上限之前停了这一条，退 254、这个键记成撞过顶，不落到「被外面停了」那一支
  if [[ "$mode" == "run" && -f "$over_booking_marker" ]]; then
    watch_excess="$(peak_file_value "$over_booking_marker" excess)" watch_used="$(peak_file_value "$over_booking_marker" used)"
    watch_slice="$(peak_file_value "$over_booking_marker" slice)" watch_trigger="$(peak_file_value "$over_booking_marker" trigger)"
    watch_peak="$(peak_file_value "$over_booking_marker" peak)"
    rm -f -- "$over_booking_marker" "$over_booking_marker.partial"
    released_memory_bytes="${watch_peak:-${admitted_need:-}}"
    admission finish "$unit" "$key" "$cap" "$cap_bytes" over-booking-stopped "$watch_peak" "$$"
    echo "run-with-memory-cap: 超出记账最多，包装在撞总上限之前停了它（slice $SLICE 回收不掉的 $watch_slice 字节到了触发线 $watch_trigger，scope $unit 此刻 $watch_used 字节、记账 ${admitted_need:-?}、超出 $watch_excess，是账上超出最多的；命令退出码 $exit_code）" >&2
    echo "run-with-memory-cap: → 这一条的结果不算数，重跑它：峰值表里这个键已记成撞过顶，重跑时按它的上限 $cap 排队，放不下就等或退 $MEMORY_ADMISSION_REFUSED_EXIT；常这样说明它要的内存比峰值表里记的多了，bash $SELF_PATH --status 看 slice 里是谁" >&2
    gpu_memory_note "$command_started" "$command_ended"
    return "$SLICE_TOTAL_HIT_EXIT"
  fi
  rm -f -- "$over_booking_marker.partial"
  if [[ "$BROKEN_JUDGEMENT" == "exitcode" ]]; then
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" "$peak_bytes"
    if [[ $exit_code -eq 137 || $exit_code -eq 143 ]]; then
      echo "run-with-memory-cap: 撞了内存上限 $cap（弄坏开关 exitcode：按退出码判）" >&2
      return "$MEMORY_CAP_HIT_EXIT"
    fi
    return "$exit_code"
  fi
  if [[ "$result" == "unreadable" ]]; then
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" ""
    report_unavailable "命令退出码 $exit_code，之后查不到 scope $unit 的结局（systemctl --user show 出错），判不了是不是撞了上限" \
      "查用户级 systemd 还在不在（systemctl --user status）；这一条的结果不算数，修好再重跑"
  fi
  if [[ "$result" == "oom-kill" && "$BROKEN_JUDGEMENT" != "noresult" ]]; then
    if ((use_slice)) && [[ "$own_oom_count" == "0" && "${oom_kill_count:-0}" != "0" && "$BROKEN_JUDGEMENT" != "noslicehit" ]]; then
      # 按峰值表放进来、峰值又超出了排队时按的量（过了容差；峰值没量到的照超了算）：这个键记成撞过顶。没超账而被内核挑中的不记（文件头「退出码」254）。
      # 弄坏开关 slicemarkall：按峰值表放进来而被挤掉的一律记（改前的判法）
      squeezed_marked=0
      if ((over_admitted)); then
        if [[ "$BROKEN_JUDGEMENT" == "slicemarkall" || ! "$peak_bytes" =~ ^[0-9]+$ ]] || ((peak_bytes > admitted_need + OVER_BOOKING_TOLERANCE_BYTES)); then
          squeezed_marked=1
        fi
      fi
      if ((squeezed_marked)) && [[ "$mode" == "run" && "$BROKEN_JUDGEMENT" != "noslicemark" ]]; then
        # 这个键记成撞过顶（峰值记成上限），重跑时按它的上限排队（文件头「峰值表」一段）
        admission finish "$unit" "$key" "$cap" "$cap_bytes" squeezed-marked "$peak_bytes" "$$"
        echo "run-with-memory-cap: 被 slice $SLICE 的总上限挤掉了（scope $unit 的 Result=oom-kill，而它自己没撞它的上限 $cap：它按峰值表放进来、涨过了排队时按的量，slice 满了，内核先挑了它；命令退出码 $exit_code）" >&2
        echo "run-with-memory-cap: → 这一条的结果不算数，重跑它：峰值表里这个键已记成撞过顶，重跑时按它的上限 $cap 排队，放不下就等或退 $MEMORY_ADMISSION_REFUSED_EXIT；bash $SELF_PATH --status 看 slice 里是谁" >&2
        gpu_memory_note "$command_started" "$command_ended"
        return "$SLICE_TOTAL_HIT_EXIT"
      fi
      [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" "" squeezed "$peak_bytes" "$$"
      if ((over_admitted && squeezed_marked == 0)); then
        echo "run-with-memory-cap: 被 slice $SLICE 的总上限挤掉了（scope $unit 的 Result=oom-kill，而它自己没撞它的上限 $cap：它按峰值表放进来，峰值 $peak_bytes 字节没超出记账 ${admitted_need:-?}（容差 $OVER_BOOKING_TOLERANCE_BYTES），是 slice 满了、内核挑中了它；峰值表里这个键不记撞过顶；命令退出码 $exit_code）" >&2
      else
        echo "run-with-memory-cap: 被 slice $SLICE 的总上限挤掉了（scope $unit 的 Result=oom-kill，而它自己没撞它的上限 $cap：整个 slice 满了，内核在 slice 里挑了这一条杀；命令退出码 $exit_code）" >&2
      fi
      echo "run-with-memory-cap: → 这一条的结果不算数，重跑它；常这样说明排队按峰值表放进来的几条实际占的比表里记的多，bash $SELF_PATH --status 看 slice 里是谁" >&2
      gpu_memory_note "$command_started" "$command_ended"
      return "$SLICE_TOTAL_HIT_EXIT"
    fi
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" "$cap_bytes" own-cap "$peak_bytes" "$$"
    echo "run-with-memory-cap: 撞了内存上限 $cap（scope $unit 的 Result=oom-kill，命令退出码 $exit_code）" >&2
    gpu_memory_note "$command_started" "$command_ended"
    return "$MEMORY_CAP_HIT_EXIT"
  fi
  if [[ "$result" == "timeout" ]]; then
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" ""
    echo "run-with-memory-cap: 超过限时 ${TIME_LIMIT_SECONDS} 秒（scope $unit 的 Result=timeout，systemd 停掉了 scope 里的进程；命令退出码 $exit_code）" >&2
    return "$TIME_LIMIT_HIT_EXIT"
  fi
  stopped_signal=""
  if [[ "$mode" == "run" && "$BROKEN_JUDGEMENT" != "recordkilledpeak" ]]; then
    stopped_signal="$(stopped_from_outside_signal "$exit_code")" || stopped_signal=""
  fi
  if [[ -n "$stopped_signal" ]]; then
    # 命令被外面的信号停了（earlyoom 发的 TERM 这一类），而 scope 没撞自己的上限、也没被 slice 挤掉：量到的是半截，不记进峰值表（文件头「峰值表」一段）
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" ""
    echo "run-with-memory-cap: 被外面停了（信号 $stopped_signal），这一次的峰值不记（命令退 $exit_code；scope $unit 没撞它自己的上限 $cap，也没被 slice $SLICE 的总上限挤掉，量到的是半截）" >&2
    echo "run-with-memory-cap: → 这一条的结果不算数；查是谁发的信号（系统日志里 earlyoom 的记录、手动 kill、看门狗），腾出整机内存或等发信号的一方停手，再重跑；峰值表里这个键仍是上一次整段跑完的量" >&2
  else
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" "$peak_bytes" clean "$peak_bytes" "$$"
  fi
  # 显存不够（文件头「显存不够」一段）：命令退了非 0 才读内核日志
  if [[ "$mode" == "run" && "$BROKEN_JUDGEMENT" != "nogpuoom" ]] && [[ $exit_code -ne 0 || "$BROKEN_JUDGEMENT" == "gpuoomonsuccess" ]]; then
    gpu_verdict="$(gpu_memory_verdict "$command_started" "$command_ended")"
    case "$gpu_verdict" in
      hit\ *)
        echo "run-with-memory-cap: 显存不够（命令退 $exit_code；它起止之间内核日志里有 NVRM … NV_ERR_NO_MEMORY：${gpu_verdict#hit }）" >&2
        echo "run-with-memory-cap: → 这一条的结果不算数（退 $GPU_MEMORY_HIT_EXIT）：看同一段时间谁在用卡（nvidia-smi），少给这一条或同时在跑的几条一些显存再重跑；经统一派发器 scripts/hosts/host-dispatch.py --gpu 起的按显存余量挑机器" >&2
        return "$GPU_MEMORY_HIT_EXIT" ;;
      unreadable\ *)
        echo "run-with-memory-cap: 显存 OOM 没查成（${gpu_verdict#unreadable }）：命令退 $exit_code，判不了是不是显存不够，不当成没撞；手动 journalctl -k | grep NV_ERR_NO_MEMORY 看一眼" >&2 ;;
    esac
  fi
  return "$exit_code"
}

show_status() {
  local total cgroup_path slice_directory=""
  check_size_syntax "$RESERVE" "余量 RUN_WITH_MEMORY_CAP_RESERVE "
  [[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
  total="$(slice_total_bytes)" || total="unknown"
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  [[ -n "$cgroup_path" ]] && slice_directory="$CGROUP_ROOT$cgroup_path"
  admission status "$SLICE" "$total" "$RESERVE" "$slice_directory" "$CGROUP_ROOT" "$PRIVATE_CONFIG"
}

# ── 自证 ──
# 吃内存的探针：每次拿 16 MiB 写满（真占常驻页），拿到 MEBIBYTES 就停下正常退出；上限 64M 时它在半路被杀
bounded_memory_hog_code() {
  printf '%s\n' "import sys" "held = []" "for _ in range(int(sys.argv[1]) // 16):" "    held.append(b'\\x01' * (16 * 1024 * 1024))" \
    "print('allocated', len(held) * 16, 'MiB')"
}

# 排队那几例用的作业：起跑时刻写进第一个文件，等一会儿、拿够内存、再占一会儿，收尾时刻写进第二个文件
selftest_job_code() {
  printf '%s\n' "import sys, time" "start_path, end_path = sys.argv[1], sys.argv[2]" \
    "delay_seconds, mebibytes, hold_seconds = float(sys.argv[3]), int(sys.argv[4]), float(sys.argv[5])" \
    "open(start_path, 'w').write(repr(time.time()))" "time.sleep(delay_seconds)" \
    "held = [bytes([1]) * (1024 * 1024) for _ in range(mebibytes)]" "time.sleep(hold_seconds)" \
    "open(end_path, 'w').write(repr(time.time()))"
}

run_selftest() {
  local failures=0 checked=0 scratch output status marker hog_code job_code target slice_counter=0 slice context_directory
  local first_pid second_pid first_status second_status started_seconds elapsed_seconds cgroup_path first_total second_total_after
  local -a selftest_slices=() context=() host_lookup_payload=()   # host_lookup_payload：host_lookup_command 要换掉被包的命令时设（改去第二台的命令里改写路径那一格），平常空
  local selftest_admitted_at='2026-09-25T00:00:00Z'   # clock-times:allow 自检造的账本与峰值表时刻，格式照真账本
  target="${RUN_WITH_MEMORY_CAP_SELFTEST_TARGET:-$SELF_PATH}"
  # 自证本身也可能套在外层包装里跑（照规则经 run-with-memory-cap.sh <上限> 起）：各例只在「套在包装里」那几例里自己造外层，别的一律当没套
  unset RUN_WITH_MEMORY_CAP_OUTER_UNIT RUN_WITH_MEMORY_CAP_OUTER_CAP
  # 外层按峰值表放进来时它写的 oom_score_adj 500 会传给这里起的每一条（文件头「排队」一段「撞 slice 总上限时内核杀谁」），
  # 「按上限放进来的不写」那几例就不是 0、内核挑错人：先把自证自己的写回 0（不提权也写得回：只往下写到 0）
  { printf '0\n' > /proc/self/oom_score_adj; } 2>/dev/null
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/run-with-memory-cap-selftest-XXXXXX")"
  # 自证起的每一份包装的标记（文件头「弄坏开关只在自证里认」）：建在临时目录里、export，收尾随 scratch 删
  printf '%s\n' "$SELFTEST_MARK_CONTENT" > "$scratch/selftest-mark"
  export RUN_WITH_MEMORY_CAP_SELFTEST_MARK="$scratch/selftest-mark"
  hog_code="$(bounded_memory_hog_code)"
  job_code="$(selftest_job_code)"
  fail() { echo "  ✗ 自检：$1"; failures=$((failures + 1)); }   # gate-lint:detail
  # 每一例一个新 slice（名字不带 -：- 是 systemd 的层级）、新的账与峰值表；收尾时 stop 加 revert 掉
  fresh_context() { # fresh_context <总上限>
    slice_counter=$((slice_counter + 1))
    slice="singlefs_memory_selftest_$$_${slice_counter}.slice"
    selftest_slices+=("$slice")
    context_directory="$scratch/context-$slice_counter"
    mkdir -p "$context_directory/temporary-root"
    # 各例起的包装给命令建的临时目录也落在这一例底下（文件头「这一条命令自己的临时目录」一段）：给包装发 KILL / HUP 的那几例 EXIT trap 不走、留下的那一份随 scratch 删
    context=(-u SINGLEFS_HEAVY_TESTS -u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL="$1" RUN_WITH_MEMORY_CAP_STATE_DIR="$context_directory/state"
             RUN_WITH_MEMORY_CAP_PEAKS="$context_directory/peaks.tsv" RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$context_directory/temporary-root")
    # 告警记录跟着峰值表走（文件头「峰值表」一段「告警跟着峰值表走」）：写进这一例峰值表旁边的 peaks.tsv.alerts.jsonl（context_alerts），不写工作根那一份
    # （看门狗读那一份叫醒主 agent）；工作根换成这一例的 work-root，弄坏开关让告警写回真记录时也落在这里，它底下的 core-usage-alerts.jsonl 当「真记录」认
    context_alerts="$context_directory/peaks.tsv.alerts.jsonl"
    context+=(SINGLEFS_WORK_ROOT="$context_directory/work-root")
  }
  in_context() { env "${context[@]}" "$@"; }
  intervals_overlap() { # intervals_overlap <前一条起> <前一条收> <后一条起> <后一条收>：四个文件都在、两段时间有交集才退 0
    python3 -c 'import sys; a0, a1, b0, b1 = (float(open(path).read()) for path in sys.argv[1:5]); sys.exit(0 if a0 < b1 and b0 < a1 else 1)' "$@" 2>/dev/null
  }
  all_exist() { local path; for path in "$@"; do [[ -f "$path" ]] || return 1; done; }
  seed_peak() { # seed_peak <键> <字节> [<那一次的上限写法，默认 200M>]：往这一例的峰值表里写一行
    printf '%s\t%s\t%s\t%s\n' "$2" "$selftest_admitted_at" "${3:-200M}" "$1" >> "$context_directory/peaks.tsv"
  }
  # 排队时去找别的机器（文件头「排队时去找别的机器」）：判法是真的 scripts/hosts/host-resources.py，换成假的只有多机配置、PATH 最前面的 ssh
  # （第二台那一头读假的 /proc）与 nvidia-smi（报没有卡）。挡路的是账上一笔假的（包装进程号写这个自证自己、要 250M），这一条要 200M、总上限 256M，只能排队。
  # 告警记录写这一例自己的文件（SINGLEFS_CORE_USAGE_ALERTS），会话号是假的。第二台慢的那两例里，假 ssh 睡着的那一个由它自己睡完退出（至多 10 秒），不发信号。
  host_lookup_world() { # host_lookup_world <目录> <第二台此刻可用，GiB>：一份假的多机配置与第二台
    local world="$1"
    mkdir -p "$world/bin" "$world/peer/proc" "$world/peer/state/ledger"
    cat > "$world/bin/ssh" <<'FAKE_SSH'
#!/usr/bin/env bash
# 假 ssh：去掉 -o 选项与主机名；第二台目录里有 unreachable 就照连不上退 255，有 slow 就先睡它写的秒数；命令照 ssh 那样交给 bash -c，
# /proc/meminfo、包装的 slice、余量与账换成第二台那一份
while [[ "${1:-}" == -o ]]; do shift 2; done
shift
here="$HOST_LOOKUP_SELFTEST_WORLD/peer"
if [[ -e "$here/unreachable" ]]; then echo "ssh: connect to host selftest-peer-host-name port 22: Connection refused" >&2; exit 255; fi
[[ -e "$here/slow" ]] && sleep "$(cat "$here/slow")"
exec env -u RUN_WITH_MEMORY_CAP_SLICE_TOTAL RUN_WITH_MEMORY_CAP_SLICE=singlefs_host_lookup_selftest_absent.slice RUN_WITH_MEMORY_CAP_RESERVE=20G \
  HOST_RESOURCES_PROC_ROOT="$here/proc" SINGLEFS_DISPATCH_PROBE_MEMINFO="$here/proc/meminfo" RUN_WITH_MEMORY_CAP_STATE_DIR="$here/state" bash -c "$*"
FAKE_SSH
    printf '#!/usr/bin/env bash\necho "假 nvidia-smi：没有卡" >&2\nexit 9\n' > "$world/bin/nvidia-smi"
    chmod +x "$world/bin/ssh" "$world/bin/nvidia-smi"
    printf 'MemTotal: %d kB\nMemAvailable: %d kB\n' $((60 * 1024 * 1024)) $(($2 * 1024 * 1024)) > "$world/peer/proc/meminfo"
    printf 'cpu  0 0 0 800 0 0 0 0 0 0\ncpu0 0 0 0 800 0 0 0 0 0 0\n' > "$world/peer/proc/stat"
    cp -- "$world/peer/proc/stat" "$world/peer/proc/stat.later"
    printf '0.00 0.00 0.00 1/100 1\n' > "$world/peer/proc/loadavg"
    printf '%s\n' PEER_SSH_HOST=selftest-peer-host-name PEER_REPOSITORY_DIRECTORY=/selftest/peer/home PEER_CARGO_BIN_DIRECTORY=/selftest/peer/bin PEER_MEMORY_CAP=40G \
      QUIESCE_STOP_COMMAND=true QUIESCE_STOPPED_CHECK_COMMAND=true QUIESCE_START_COMMAND=true QUIESCE_STARTED_CHECK_COMMAND=true ENABLE_ACROSS_MACHINES=1 \
      > "$world/multi-host.env"
  }
  host_lookup_blocker() { # host_lookup_blocker：账上记一笔挡路的假账（包装进程号写这个自证自己，起始时刻对得上，要 250M）
    local stat_line
    local -a stat_fields=()
    IFS= read -r stat_line < "/proc/$$/stat"
    read -r -a stat_fields <<<"${stat_line##*) }"
    mkdir -p "$context_directory/state/ledger"
    printf '{"unit": "singlefs-memory-cap-selftest-blocker", "pid": %s, "pid_start_time": "%s", "need": %s, "cap": "250M", "key": "blocker", "admitted_at": "%s"}' \
      "$$" "${stat_fields[19]:-}" $((250 * MEBIBYTE)) "$selftest_admitted_at" > "$context_directory/state/ledger/singlefs-memory-cap-selftest-blocker.json"
  }
  host_lookup_command() { # host_lookup_command <世界> <名> <等待上限秒> [变量=值…]：host_lookup_run 设成在这一例的环境里起包装（要 200M）的整条命令
    local world="$1" name="$2" wait_seconds="$3"
    local -a payload=(bash -c ': > "$1"' touch-marker "$scratch/host-lookup-$name.ran")   # 被包的命令；host_lookup_payload 非空就换成它（改去第二台的命令里改写路径那一格）
    shift 3
    if ((${#host_lookup_payload[@]})); then payload=("${host_lookup_payload[@]}"); fi
    host_lookup_run=(env "${context[@]}" env -u RUN_WITH_MEMORY_CAP_HOST_LOOKUP HOST_LOOKUP_SELFTEST_WORLD="$world" PATH="$world/bin:$PATH"
      SINGLEFS_MULTI_HOST_CONFIG="$world/multi-host.env" SINGLEFS_CORE_USAGE_ALERTS="$world/alerts.jsonl" CLAUDE_CODE_SESSION_ID=selftest-host-lookup-session
      RUN_WITH_MEMORY_CAP_KEY="selftest-host-lookup-$name" RUN_WITH_MEMORY_CAP_WAIT_SECONDS="$wait_seconds" "$@"
      bash "$target" 200M "${payload[@]}")
  }
  host_lookup_case() { # host_lookup_case <世界> <名> <等待上限秒> [变量=值…]：挡着路前台跑一条，退出码进 status、用了几秒进 elapsed_seconds，stderr 进 host-lookup-<名>.err
    local started=$SECONDS
    host_lookup_blocker
    host_lookup_command "$@"
    if "${host_lookup_run[@]}" 2>"$scratch/host-lookup-$2.err"; then status=0; else status=$?; fi
    elapsed_seconds=$((SECONDS - started))
    rm -f -- "$context_directory/state/ledger/singlefs-memory-cap-selftest-blocker.json"
  }
  host_lookup_cells() {
    local world records lines_count wrapper_pid poll err tree_top relative_tail
    local -a host_lookup_run=()
    # 第二台放得下（可用 43 GiB，这条命令没有峰值、预测 8G）：排队开始与退 252 时各打一行改去第二台的命令，告警记录 queued、refused 各一条
    fresh_context 256M
    world="$scratch/host-lookup-fits"
    host_lookup_world "$world" 43
    checked=$((checked + 1))
    host_lookup_case "$world" fits 4
    err="$scratch/host-lookup-fits.err"
    lines_count="$(grep -cF '→ 第二台此刻放得下（可用 43.0 GiB、要 8.0 GiB）：改用 bash scripts/hosts/multi-host-run.sh --peer --memory-cap 200M --label moved-from-queue-' "$err")"
    records="$(python3 -c 'import json, sys
try:
    rows = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8")]
except OSError as error:
    print(f"读不了：{error}")
    sys.exit(0)
good = [row.get("phase") for row in rows] == ["queued", "refused"] and all(
    row.get("kind") == "memory-queue-peer-fits" and row.get("session_id") == "selftest-host-lookup-session" and isinstance(row.get("alert_pid"), int)
    and "touch-marker" in row.get("command", "") and row.get("local", {}).get("need") and row.get("local", {}).get("slice_total")
    and row.get("peer", {}).get("available") == 43 * 1024 ** 3 and row.get("peer", {}).get("need") == 8 * 1024 ** 3
    and "multi-host-run.sh --peer --memory-cap 200M" in row.get("next_step", "") for row in rows)
print("ok" if good else json.dumps(rows, ensure_ascii=False)[:800])' "$world/alerts.jsonl" 2>&1)"
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || -e "$scratch/host-lookup-fits.ran" || "$lines_count" != 2 || "$records" != ok ]] \
        || ! grep -qF -- "-- bash -c ': > \"\$1\"' touch-marker" "$err"; then
      fail "第二台放得下时排队开始与退 $MEMORY_ADMISSION_REFUSED_EXIT 时各打一行改去第二台的命令、各写一条告警记录（queued、refused，带会话号、命令与两台的数），\
退 $MEMORY_ADMISSION_REFUSED_EXIT、命令没跑，实际退 $status、命令$([[ -e "$scratch/host-lookup-fits.ran" ]] && echo '跑了' || echo '没跑')、那一行 ${lines_count:-0} 行、告警记录：$records；stderr：$(cat "$err")"
    fi
    # 几条一起排队：同一个状态目录里紧接着来的另一条不再查，打「没查到别的机器：… 刚查过」
    checked=$((checked + 1))
    host_lookup_case "$world" throttled 2
    err="$scratch/host-lookup-throttled.err"
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT ]] || grep -qF '此刻放得下' "$err" || ! grep -q '没查到别的机器：.*刚查过' "$err"; then
      fail "几条一起排队时 60 秒内只查一次：紧接着来的一条应当打「没查到别的机器：… 刚查过」、不打改去第二台的命令、照旧退 $MEMORY_ADMISSION_REFUSED_EXIT，实际退 $status：$(cat "$err")"
    fi
    # 改去第二台的命令里，以本机仓根起头的参数改成树内相对路径（仓根本身写 .），仓根之外的绝对路径与名字只是以仓根起头的路径原样留着（命令没跑：挡着路、等满退 252）
    fresh_context 256M
    world="$scratch/host-lookup-relative"
    host_lookup_world "$world" 43
    checked=$((checked + 1))
    tree_top="$(cd "$(dirname "$target")/../.." && pwd)"
    host_lookup_payload=(bash "$tree_top/.claude/gate.d/x.sh" --check a "$tree_top" /tmp/outside-the-tree "$tree_top-beside/file")
    host_lookup_case "$world" relative 4
    host_lookup_payload=()
    err="$scratch/host-lookup-relative.err"
    relative_tail="-- bash .claude/gate.d/x.sh --check a . /tmp/outside-the-tree $tree_top-beside/file"
    lines_count="$(grep -cF -- "$relative_tail" "$err")"
    records="$(python3 -c 'import json, sys
try:
    rows = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8")]
except OSError as error:
    print(f"读不了：{error}")
    sys.exit(0)
tail = sys.argv[2]
good = len(rows) == 2 and all(tail in row.get("next_step", "") and tail in row.get("line", "") and sys.argv[3] + "/.claude" not in row.get("next_step", "") for row in rows)
print("ok" if good else json.dumps(rows, ensure_ascii=False)[:800])' "$world/alerts.jsonl" "$relative_tail" "$tree_top" 2>&1)"
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || "$lines_count" != 2 || "$records" != ok ]] || grep -F "→ 第二台此刻放得下" "$err" | grep -qF -- "bash $tree_top/.claude"; then
      fail "改去第二台的命令里以本机仓根起头的参数改成树内相对路径（仓根本身写 .），仓根之外的绝对路径与名字只是以仓根起头的路径原样留着：排队开始与退 $MEMORY_ADMISSION_REFUSED_EXIT 时各一行、告警记录的出路与那一行里也是，\
应当有「$relative_tail」，实际退 $status、那一行 ${lines_count:-0} 行、告警记录：$records；stderr：$(cat "$err")"
    fi
    # 第二台放不下（可用 1 GiB）：各打一行「第二台此刻也放不下」，不写告警记录
    fresh_context 256M
    world="$scratch/host-lookup-nofit"
    host_lookup_world "$world" 1
    checked=$((checked + 1))
    host_lookup_case "$world" nofit 3
    err="$scratch/host-lookup-nofit.err"
    lines_count="$(grep -c '现查别的机器：第二台此刻也放不下' "$err")"
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || "$lines_count" != 2 || -s "$world/alerts.jsonl" ]] || grep -qF '此刻放得下' "$err"; then
      fail "第二台放不下时各打一行第二台此刻也放不下、不打改去第二台的命令、不写告警记录、照旧退 $MEMORY_ADMISSION_REFUSED_EXIT，\
实际退 $status、那一行 ${lines_count:-0} 行、告警记录$([[ -s "$world/alerts.jsonl" ]] && echo '写了' || echo '没写')：$(cat "$err")"
    fi
    # 第二台连不上（假 ssh 退 255）：各打一行「没查到别的机器：第二台没读到…」
    fresh_context 256M
    world="$scratch/host-lookup-unreachable"
    host_lookup_world "$world" 43
    : > "$world/peer/unreachable"
    checked=$((checked + 1))
    host_lookup_case "$world" unreachable 3
    err="$scratch/host-lookup-unreachable.err"
    lines_count="$(grep -c '没查到别的机器：第二台没读到' "$err")"
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || "$lines_count" != 2 || -s "$world/alerts.jsonl" ]] || grep -qF '此刻放得下' "$err"; then
      fail "第二台连不上时各打一行没查到别的机器（第二台没读到）、不写告警记录、照旧退 $MEMORY_ADMISSION_REFUSED_EXIT，实际退 $status、那一行 ${lines_count:-0} 行：$(cat "$err")"
    fi
    # 超过时限：第二台 10 秒才回话、这一条的时限 2 秒
    fresh_context 256M
    world="$scratch/host-lookup-timeout"
    host_lookup_world "$world" 43
    printf '10\n' > "$world/peer/slow"
    checked=$((checked + 1))
    host_lookup_case "$world" timeout 3 RUN_WITH_MEMORY_CAP_HOST_LOOKUP_SECONDS=2
    err="$scratch/host-lookup-timeout.err"
    lines_count="$(grep -c '没查到别的机器：2 秒没查完' "$err")"
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || "$lines_count" != 2 ]] || ((elapsed_seconds >= 12)); then
      fail "查别的机器超过时限时各打一行没查到别的机器：2 秒没查完、照旧退 $MEMORY_ADMISSION_REFUSED_EXIT（等待上限 3 秒，12 秒内退），\
实际退 $status、那一行 ${lines_count:-0} 行、用了 $elapsed_seconds 秒：$(cat "$err")"
    fi
    # 不拖慢准入：第二台 6 秒才回话；看到「排队」那一行就删掉挡路那一笔，这一条要在 4 秒内起跑（排队每 1 秒重判一次）
    fresh_context 256M
    world="$scratch/host-lookup-prompt"
    host_lookup_world "$world" 43
    printf '6\n' > "$world/peer/slow"
    checked=$((checked + 1))
    err="$scratch/host-lookup-prompt.err"
    host_lookup_blocker
    host_lookup_command "$world" prompt 30
    "${host_lookup_run[@]}" 2>"$err" &
    wrapper_pid=$!
    for ((poll = 0; poll < 200; poll++)); do
      grep -q '排队（已等' "$err" 2>/dev/null && break
      sleep 0.1
    done
    rm -f -- "$context_directory/state/ledger/singlefs-memory-cap-selftest-blocker.json"
    for ((poll = 0; poll < 100; poll++)); do
      [[ -e "$scratch/host-lookup-prompt.ran" ]] && break
      sleep 0.1
    done
    if wait "$wrapper_pid"; then status=0; else status=$?; fi
    if [[ $status -ne 0 || ! -e "$scratch/host-lookup-prompt.ran" ]] || ((poll >= 40)); then
      fail "排队开始时在查别的机器（第二台 6 秒才回话）不拖慢准入：挡路那一笔删掉之后 4 秒内起跑、退 0，\
实际退 $status、删掉之后 $((poll / 10)) 秒多$([[ -e "$scratch/host-lookup-prompt.ran" ]] && echo '才起跑' || echo '还没起跑')：$(cat "$err")"
    fi
    host_lookup_fallback_cells
  }
  # 整体兜底（文件头「排队时去找别的机器」「整体兜底」）：六处出事，包装的退出码与关掉 HostLookup 的那一臂逐个相同、30 秒内退。
  # 四处异常由 RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_FAULT 在那一处抛；锁与线程两格造真卡住的世界：一个真进程占住排队锁、后台那一趟真睡着，不抛异常冒充
  host_lookup_fallback_run() { # host_lookup_fallback_run <名> <上限> <等待上限秒> [变量=值…]：在 world 里挡着路跑一条（限 30 秒），退出码进 status，stderr 进 host-lookup-<名>.err
    local name="$1" cap="$2" wait_seconds="$3"
    shift 3
    host_lookup_blocker
    rm -f -- "$context_directory/state/host-lookup.json"   # 几条一起排队时每 60 秒只查一次：每一格都要现查，先清掉上一格记的谁查过
    if timeout 30 env "${context[@]}" env -u RUN_WITH_MEMORY_CAP_HOST_LOOKUP HOST_LOOKUP_SELFTEST_WORLD="$world" PATH="$world/bin:$PATH" \
        SINGLEFS_MULTI_HOST_CONFIG="$world/multi-host.env" SINGLEFS_CORE_USAGE_ALERTS="$world/alerts.jsonl" CLAUDE_CODE_SESSION_ID=selftest-host-lookup-session \
        RUN_WITH_MEMORY_CAP_KEY="selftest-host-lookup-$name" RUN_WITH_MEMORY_CAP_WAIT_SECONDS="$wait_seconds" "$@" \
        bash "$target" "$cap" bash -c ': > "$1"' touch-marker "$scratch/host-lookup-$name.ran" 2>"$scratch/host-lookup-$name.err"; then status=0; else status=$?; fi
    rm -f -- "$context_directory/state/ledger/singlefs-memory-cap-selftest-blocker.json"
  }
  host_lookup_fallback_cells() {
    local world off_status lock_off_status point handoff holder_pid err
    fresh_context 256M
    world="$scratch/host-lookup-fallback"
    host_lookup_world "$world" 43
    # 关掉 HostLookup 的两臂：排队等满 3 秒退 252；这一条要 300M、比总上限 256M 还大，当场退 252
    host_lookup_fallback_run fallback-off 200M 3 RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off
    off_status=$status
    host_lookup_fallback_run fallback-lock-off 300M 3 RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off
    lock_off_status=$status
    # 起线程、建临时文件、读输出、拼改去第二台那一行（第二台放得下才走到）四处抛异常：只打一行「没查到别的机器：…出了 …」，退出码照旧
    for point in thread-start temp-file read-output format; do
      checked=$((checked + 1))
      err="$scratch/host-lookup-fallback-$point.err"
      host_lookup_fallback_run "fallback-$point" 200M 3 RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_FAULT="$point"
      if [[ $status -ne $off_status || $off_status -ne $MEMORY_ADMISSION_REFUSED_EXIT ]] || grep -q '^Traceback' "$err" || ! grep -q '没查到别的机器：.*出了 ' "$err"; then
        fail "HostLookup 在「$point」那一处抛异常时只打一行没查到别的机器、退出码与关掉它的那一臂相同（$off_status），实际退 $status：$(cat "$err")"
      fi
    done
    # 锁被别的进程占着：当场退 252 之前现查的那一趟要取排队锁时，叫一个真进程来占住它（占上了写 taken，一直占着，到这一格收尾才停它）
    handoff="$scratch/host-lookup-lock-handoff"
    mkdir -p "$handoff" "$context_directory/state"
    python3 -c 'import fcntl, os, sys, time
handoff, lock_path = sys.argv[1], sys.argv[2]
deadline = time.monotonic() + 30
while not os.path.exists(os.path.join(handoff, "request")) and time.monotonic() < deadline:
    time.sleep(0.05)
handle = open(lock_path, "a")
fcntl.flock(handle, fcntl.LOCK_EX)
open(os.path.join(handoff, "taken"), "w").close()
time.sleep(120)' "$handoff" "$context_directory/state/lock" &
    holder_pid=$!
    checked=$((checked + 1))
    err="$scratch/host-lookup-fallback-lock.err"
    host_lookup_fallback_run fallback-lock 300M 3 RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_LOCK_HANDOFF="$handoff"
    kill "$holder_pid" 2>/dev/null
    wait "$holder_pid" 2>/dev/null || :
    if [[ $status -ne $lock_off_status || $lock_off_status -ne $MEMORY_ADMISSION_REFUSED_EXIT || ! -e "$handoff/taken" ]] \
        || ! grep -q '没查到别的机器：排队锁 .* 被别的进程占着' "$err"; then
      fail "排队锁被别的进程占着时这一趟不查、打一行「没查到别的机器：排队锁 … 被别的进程占着」、退出码与关掉它的那一臂相同（$lock_off_status），\
实际退 $status（124 是 30 秒到了被停）、占锁进程$([[ -e "$handoff/taken" ]] && echo '占上了' || echo '没占上')：$(cat "$err")"
    fi
    # 线程里的 query 睡着：排队开始那一趟在后台真睡 120 秒，等满 3 秒退 252 之前至多等它 2 + 2 秒，不再等
    checked=$((checked + 1))
    err="$scratch/host-lookup-fallback-sleep.err"
    host_lookup_fallback_run fallback-sleep 200M 3 RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_SLEEP=120 RUN_WITH_MEMORY_CAP_HOST_LOOKUP_SECONDS=2
    if [[ $status -ne $off_status ]] || ! grep -q '没查到别的机器：排队开始那一趟 .* 秒还没查完，不再等它' "$err"; then
      fail "线程里的 query 睡着时退 252 之前不一直等它、打一行「排队开始那一趟 … 秒还没查完，不再等它」、退出码与关掉它的那一臂相同（$off_status），\
实际退 $status（124 是 30 秒到了被停）：$(cat "$err")"
    fi
  }
  # 只跑这几项（RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=host-lookup，看它红在哪用）：跑完收尾、退出，不算整份自证通过
  if [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_ONLY:-}" == host-lookup ]]; then
    host_lookup_cells
    for slice in "${selftest_slices[@]}"; do
      systemctl --user stop "$slice" >/dev/null 2>&1
      systemctl --user revert "$slice" >/dev/null 2>&1
    done
    rm -rf -- "${scratch:?}"
    if ((failures)); then
      echo "  ✗ run-with-memory-cap.sh 自检（只跑排队时去找别的机器那几项）$failures 处不对（查了 $checked 项；测的是 $target）"
      echo "  → 怎么办：照上面逐条改 admission 里的 HostLookup 与 admit 调它的那几处；RUN_WITH_MEMORY_CAP_BREAK 设着的话本来就该红"
      exit 1
    fi
    echo "  ✓ run-with-memory-cap.sh 自检（只跑排队时去找别的机器那几项，不算整份自证通过）查了 $checked 项都对"
    exit 0
  fi

  # 峰值表近几次、峰值涨了、撞顶与被挤掉的告警（文件头「峰值表」一段）。记账看命令自己在 scope 里读到的账（账上这一笔的 need），
  # 告警记录写这一例峰值表旁边的那一份（fresh_context 设的 context_alerts），会话号是假的
  local peak_session=selftest-peak-history-session
  local ledger_need_code ledger_shrink_code alert_check_code dispatch_prediction_code
  ledger_need_code='import glob, json, sys
entries = [json.load(open(path)) for path in glob.glob(sys.argv[1] + "/*.json")]
open(sys.argv[2], "w").write(" ".join(str(entry["need"]) for entry in entries if entry.get("key") == sys.argv[3]))'
  # 记账之后把自己账上那一笔的 need 改成 1 MiB（记账从哪来不动），再拿 48 MiB：造一条「按预测记账、实测比记账大得多」的
  ledger_shrink_code='import glob, json, os, sys
for path in glob.glob(sys.argv[1] + "/*.json"):
    entry = json.load(open(path))
    if entry.get("key") == sys.argv[2]:
        entry["need"] = 1024 * 1024
        open(path + ".shrunk", "w").write(json.dumps(entry))
        os.replace(path + ".shrunk", path)
held = [bytes([1]) * (1024 * 1024) for _ in range(int(sys.argv[3]))]'
  alert_check_code='import json, sys
path, kind, key, wanted = sys.argv[1], sys.argv[2], sys.argv[3], json.loads(sys.argv[4])
try:
    rows = [json.loads(line) for line in open(path, encoding="utf-8")]
except OSError:
    rows = []
found = [row for row in rows if row.get("kind") == kind and row.get("key") == key]
count, minimum = wanted.pop("count", 1), wanted.pop("measured_at_least", None)
problems = [] if len(found) == count else [f"{len(found)} 条（要 {count} 条）"]
for row in found:
    problems += [f"{name}={row.get(name)!r}（要 {value!r}）" for name, value in wanted.items() if row.get(name) != value]
    if not isinstance(row.get("alert_pid"), int) or not row.get("line") or not row.get("next_step"):
        problems.append(f"告警进程号、那一行或出路缺了：{row}")
    measured = row.get("measured")
    if minimum is not None and not (isinstance(measured, int) and measured >= minimum):
        problems.append(f"measured={measured!r}（要不小于 {minimum}）")
print("ok" if not problems else "；".join(problems))'
  dispatch_prediction_code='import dataclasses, importlib.util, os, sys
spec = importlib.util.spec_from_file_location("host_dispatch", os.path.join(sys.argv[1], "host-dispatch.py"))
dispatch = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dispatch)
settings = dispatch.settings_from_environment()
settings = dataclasses.replace(settings, peak_table=sys.argv[2], environment=dict(settings.environment, RUN_WITH_MEMORY_CAP_KEY=sys.argv[3]))
prediction = dispatch.predict(settings, [sys.argv[3]], False)
print(prediction.memory_history, prediction.memory_predicted)'
  alert_check() { python3 -c "$alert_check_code" "$@" 2>&1; }   # alert_check <告警记录> <kind> <键> <要的字段 JSON>
  dispatch_prediction() { python3 -B -c "$dispatch_prediction_code" "$(dirname "$target")" "$@" 2>&1; }   # dispatch_prediction <峰值表> <键>：派发器 predict 取的「来历 字节」
  recent_column() { awk -F '\t' -v key="$1" '$4 == key {print NF "|" $6}' "$context_directory/peaks.tsv" 2>/dev/null; }   # recent_column <键> → 「列数|第六列」
  peak_history_cells() {
    local round seen seen_all expected_need recent max_recent latest grew_err calibrated booked_grew booked_small records predicted
    local mebibytes resident_pid resident_status expectation expected_key expected_need
    local plain_status plain_records real_alerts real_records alias_table default_table explicit_records case_label
    local -a peaks_setting=()
    # 撞顶之后的滞回：上限 64M 里分配 512 MiB 撞顶（退 250，写一条 memory-own-cap），之后连跑干净的：头 5 次按上限 64M 记账，
    # 第 5 次跑完近几次里没有撞顶的了，第 6 次按近 5 次实测里最大的记账；派发器的 predict 在同一张表上同样认（第 4 次之后是撞过顶，第 5 次之后取同一个数）
    fresh_context 256M
    checked=$((checked + 1))
    if in_context env CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_KEY=selftest-hysteresis bash "$target" 64M python3 -c "$hog_code" 512 \
        >/dev/null 2>"$scratch/hysteresis-hit.err"; then status=0; else status=$?; fi
    records="$(alert_check "$context_alerts" memory-own-cap selftest-hysteresis \
      "{\"session_id\": \"$peak_session\", \"source\": \"run-with-memory-cap.sh\", \"cap\": \"64M\", \"booked\": $((64 * MEBIBYTE)), \"booked_source\": \"cap\", \"by_peak_table\": false, \"measured_at_least\": $((48 * MEBIBYTE))}")"
    if [[ $status -ne $MEMORY_CAP_HIT_EXIT || "$records" != ok ]]; then
      fail "撞了自己的上限应当退 $MEMORY_CAP_HIT_EXIT、往告警记录写一条 memory-own-cap（会话号、键、上限 64M、记账 64M 按上限、实测、不是按峰值表放进来的），实际退 $status、记录：$records：$(cat "$scratch/hysteresis-hit.err")"
    fi
    seen_all=""
    for round in 1 2 3 4 5 6; do
      if ((round == 6)); then
        recent="$(recent_column selftest-hysteresis)"
        max_recent="$(tr ',' '\n' <<<"${recent#*|}" | awk -F ':' '$2 == "clean" {if ($1 > max) max = $1; clean++} END {if (clean == 5) print max}')"
        predicted="$(dispatch_prediction "$context_directory/peaks.tsv" selftest-hysteresis)"
      fi
      rm -f -- "$scratch/hysteresis-$round.seen"
      in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-hysteresis bash "$target" 64M python3 -c "$ledger_need_code" "$context_directory/state/ledger" \
        "$scratch/hysteresis-$round.seen" selftest-hysteresis 2>>"$scratch/hysteresis.err"
      seen="$(cat "$scratch/hysteresis-$round.seen" 2>/dev/null)"
      seen_all+="${seen_all:+、}第 $round 次 ${seen:-（没读到）}"
      if ((round == 4)); then
        predicted="$(dispatch_prediction "$context_directory/peaks.tsv" selftest-hysteresis)"
        checked=$((checked + 1))
        [[ "$predicted" == "hit $((64 * MEBIBYTE * 3 / 2))" ]] \
          || fail "撞顶之后跑了 4 次干净的，派发器的 predict 在同一张表上应当仍按撞过顶预测 $((64 * MEBIBYTE * 3 / 2))（「hit 字节」），实际「$predicted」"
      fi
    done
    checked=$((checked + 1))
    expected_need=""
    for round in 1 2 3 4 5; do expected_need+="${expected_need:+、}第 $round 次 $((64 * MEBIBYTE))"; done
    if [[ "${seen_all%、第 6 次*}" != "$expected_need" || -z "$max_recent" || "${seen_all##*第 6 次 }" != "$max_recent" ]] || ((max_recent >= 64 * MEBIBYTE)); then
      fail "撞顶之后连跑 4 次干净的应当仍按上限 64M 排，第 5 次干净的也按上限、跑完之后第 6 次才按近 5 次实测里最大的峰值（${max_recent:-近几次里不是 5 次干净的}）排，\
实际账上记的：$seen_all（近几次那一列跑第 6 次之前是「$recent」）：$(tail -n 5 "$scratch/hysteresis.err")"
    fi
    checked=$((checked + 1))
    [[ "$predicted" == "peak $max_recent" ]] \
      || fail "撞顶之后跑了 5 次干净的，派发器的 predict 在同一张六列表上应当与包装第 6 次记的账相同（peak ${max_recent:-?}），实际「$predicted」"

    # 峰值一路变小（依次拿 100、80、60、40 MiB）：记账取近几次里最大的（拿 100 MiB 那一次的峰值），不跟着变小；派发器取同一个数
    fresh_context 256M
    for mebibytes in 100 80 60 40; do
      in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-descending bash "$target" 200M python3 -c "$job_code" "$scratch/descending-$mebibytes.start" "$scratch/descending-$mebibytes.end" 0 "$mebibytes" 0 \
        >/dev/null 2>>"$scratch/descending.err"
    done
    recent="$(recent_column selftest-descending)"
    max_recent="$(tr ',' '\n' <<<"${recent#*|}" | awk -F ':' '{if ($1 > max) max = $1} END {print max + 0}')"
    latest="$(tr ',' '\n' <<<"${recent#*|}" | awk -F ':' 'NR == 1 {print $1 + 0}')"
    predicted="$(dispatch_prediction "$context_directory/peaks.tsv" selftest-descending)"
    rm -f -- "$scratch/descending.seen"
    in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-descending bash "$target" 200M python3 -c "$ledger_need_code" "$context_directory/state/ledger" "$scratch/descending.seen" selftest-descending \
      2>>"$scratch/descending.err"
    seen="$(cat "$scratch/descending.seen" 2>/dev/null)"
    checked=$((checked + 1))
    if [[ "${recent%%|*}" != 6 || "$seen" != "$max_recent" ]] || ((max_recent < 100 * MEBIBYTE || latest >= max_recent)); then
      fail "峰值依次是 100M、80M、60M、40M 时，记账应当取近几次里最大的（拿 100 MiB 那一次，不小于 100 MiB），不取最近一次的 $latest，\
实际账上记 ${seen:-（没读到）}、近几次那一列「$recent」：$(tail -n 5 "$scratch/descending.err")"
    fi
    checked=$((checked + 1))
    [[ "$predicted" == "peak $max_recent" ]] \
      || fail "峰值依次是 100M、80M、60M、40M 时，派发器的 predict 在同一张六列表上应当取与包装记账相同的 $max_recent（「peak 字节」），实际「$predicted」"

    # 加近几次之前记的四列、五列旧行：照读，当成只有一次，记账等于那一行的峰值；跑完之后四列那一行变成六列、近几次是这一次加旧的那一次
    fresh_context 256M
    seed_peak selftest-old-four $((30 * MEBIBYTE))
    printf '%s\t%s\t%s\t%s\t%s\n' $((50 * MEBIBYTE)) "$selftest_admitted_at" 200M selftest-old-five 4 >> "$context_directory/peaks.tsv"
    for expectation in "selftest-old-four|$((30 * MEBIBYTE))" "selftest-old-five|$((50 * MEBIBYTE))"; do
      expected_key="${expectation%%|*}" expected_need="${expectation#*|}"
      rm -f -- "$scratch/$expected_key.seen"
      in_context env RUN_WITH_MEMORY_CAP_KEY="$expected_key" bash "$target" 200M python3 -c "$ledger_need_code" "$context_directory/state/ledger" "$scratch/$expected_key.seen" "$expected_key" \
        2>>"$scratch/old-rows.err"
      seen="$(cat "$scratch/$expected_key.seen" 2>/dev/null)"
      checked=$((checked + 1))
      [[ "$seen" == "$expected_need" ]] \
        || fail "加近几次之前记的旧行（$expected_key）应当照读、记账等于那一行的峰值 $expected_need，实际账上记 ${seen:-（没读到）}：$(tail -n 3 "$scratch/old-rows.err")"
    done
    checked=$((checked + 1))
    recent="$(recent_column selftest-old-four)"
    [[ "${recent%%|*}" == 6 && "${recent#*|}" == *",$((30 * MEBIBYTE)):clean" ]] \
      || fail "四列旧行（selftest-old-four）跑过一次之后应当是六列、近几次是「这一次,$((30 * MEBIBYTE)):clean」，实际「$recent」（列数|第六列）"

    # 峰值涨了：先量一次拿 48 MiB 的峰值 P，记账 P/3 的那一条（倍数约 3）打那一行、写 memory-peak-grew；记账 P×4/5 的（倍数约 1.25）不打不写；
    # 按预测（默认值 8G）记账、跑起来把自己账上的 need 改成 1 MiB 的那一条（倍数几十）也不打不写：记账不是取自峰值表
    fresh_context 256M
    in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-grew-calibration bash "$target" 200M python3 -c "$job_code" "$scratch/grew-0.start" "$scratch/grew-0.end" 0 48 0 \
      >/dev/null 2>"$scratch/grew-calibration.err"
    calibrated="$(awk -F '\t' '$4 == "selftest-grew-calibration" {print $1}' "$context_directory/peaks.tsv" 2>/dev/null)"
    calibrated="${calibrated:-0}"
    booked_grew=$(( (calibrated / 3 + MEBIBYTE - 1) / MEBIBYTE * MEBIBYTE )) booked_small=$(( (calibrated * 4 / 5 + MEBIBYTE - 1) / MEBIBYTE * MEBIBYTE ))
    seed_peak selftest-grew "$booked_grew"
    seed_peak selftest-grew-small "$booked_small"
    checked=$((checked + 1))
    grew_err="$scratch/grew.err"
    in_context env CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_KEY=selftest-grew bash "$target" 200M python3 -c "$job_code" "$scratch/grew-1.start" "$scratch/grew-1.end" 0 48 0 \
      >/dev/null 2>"$grew_err"
    records="$(alert_check "$context_alerts" memory-peak-grew selftest-grew \
      "{\"session_id\": \"$peak_session\", \"cap\": \"200M\", \"booked\": $booked_grew, \"booked_source\": \"peak-table\", \"by_peak_table\": true, \"measured_at_least\": $((booked_grew * 3 / 2))}")"
    if ((calibrated < 48 * MEBIBYTE)) || [[ "$records" != ok ]] || ! grep -q '^run-with-memory-cap: 这一次峰值 .*，是记账 .* 的 [0-9.]* 倍：工作量可能变了' "$grew_err" \
        || ! grep -q '^run-with-memory-cap: → 查这条命令最近的改动' "$grew_err"; then
      fail "记账 $((booked_grew / MEBIBYTE)) MiB（峰值表）、实测约 $((calibrated / MEBIBYTE)) MiB 的那一趟应当打「这一次峰值 X，是记账 Y 的 Z 倍：工作量可能变了」与出路、写一条 memory-peak-grew，\
实际记录：$records；stderr：$(cat "$grew_err")"
    fi
    checked=$((checked + 1))
    in_context env CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_KEY=selftest-grew-small bash "$target" 200M python3 -c "$job_code" "$scratch/grew-2.start" "$scratch/grew-2.end" 0 48 0 \
      >/dev/null 2>"$scratch/grew-small.err"
    records="$(alert_check "$context_alerts" memory-peak-grew selftest-grew-small '{"count": 0}')"
    if [[ "$records" != ok ]] || grep -q '工作量可能变了' "$scratch/grew-small.err"; then
      fail "记账 $((booked_small / MEBIBYTE)) MiB、实测约 $((calibrated / MEBIBYTE)) MiB（不到 1.5 倍）的那一趟应当不打那一行、不写记录，实际记录：$records；stderr：$(cat "$scratch/grew-small.err")"
    fi
    fresh_context 40G
    checked=$((checked + 1))
    in_context env CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_KEY=selftest-grew-default bash "$target" 16G python3 -c "$ledger_shrink_code" "$context_directory/state/ledger" \
      selftest-grew-default 48 >/dev/null 2>"$scratch/grew-default.err"
    records="$(alert_check "$context_alerts" memory-peak-grew selftest-grew-default '{"count": 0}')"
    if [[ "$records" != ok ]] || grep -q '工作量可能变了' "$scratch/grew-default.err"; then
      fail "记账来自预测的默认值（峰值表里没有这条；跑起来把账上的 need 改成 1 MiB）的那一趟应当不打那一行、不写记录，实际记录：$records；stderr：$(cat "$scratch/grew-default.err")"
    fi

    # 被总上限挤掉：常驻那一条按上限 160M 放进来、占 150 MiB；这一条峰值表里记 20 MiB、上限 200M，按峰值表放进来、涨到 400 MiB 挨杀（退 254），
    # 写一条 memory-squeezed（按峰值表放进来的、记账 20 MiB、stopped_by=kernel）；常驻那一条退 0、不写。
    # 两条都设 RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY：不盯账、slice 不设 memory.high，判的是内核挤掉那一支的告警（盯账停的那一支见 over_booking_cells）
    fresh_context 256M
    checked=$((checked + 1))
    seed_peak selftest-squeezed $((20 * MEBIBYTE))
    in_context env CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_KEY=selftest-squeeze-resident bash "$target" 160M python3 -c "$job_code" \
      "$scratch/squeezed-1.start" "$scratch/squeezed-1.end" 0 150 4 >/dev/null 2>"$scratch/squeezed-1.err" &
    resident_pid=$!
    sleep 1
    if in_context env CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_KEY=selftest-squeezed bash "$target" 200M python3 -c "$hog_code" 400 \
        >/dev/null 2>"$scratch/squeezed-2.err"; then status=0; else status=$?; fi
    wait "$resident_pid"; resident_status=$?
    records="$(alert_check "$context_alerts" memory-squeezed selftest-squeezed \
      "{\"session_id\": \"$peak_session\", \"cap\": \"200M\", \"booked\": $((20 * MEBIBYTE)), \"booked_source\": \"peak-table\", \"by_peak_table\": true, \"stopped_by\": \"kernel\", \"measured_at_least\": $((20 * MEBIBYTE))}")"
    if [[ $status -ne $SLICE_TOTAL_HIT_EXIT || $resident_status -ne 0 || "$records" != ok ]] \
        || [[ "$(alert_check "$context_alerts" memory-squeezed selftest-squeeze-resident '{"count": 0}')" != ok ]]; then
      fail "按峰值表放进来又被总上限挤掉的那一条应当退 $SLICE_TOTAL_HIT_EXIT、往告警记录写一条 memory-squeezed（会话号、键、上限 200M、记账 20 MiB 取自峰值表、实测、按峰值表放进来的），\
常驻那一条退 0、不写，实际它退 $status、常驻退 $resident_status、记录：$records：$(cat "$scratch/squeezed-2.err" "$scratch/squeezed-1.err")"
    fi

    # 有意撞顶的跑法（文件头「峰值表」一段）：设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的一条撞自己的上限退 250、不写 memory-own-cap，
    # 退出码、stderr 与峰值表的撞顶标记照旧；没设的同样撞顶照旧写一条
    fresh_context 256M
    checked=$((checked + 1))
    if in_context env CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_KEY=selftest-oom-first-hit RUN_WITH_MEMORY_CAP_OOM_FIRST='有意撞顶的变异步（自检）' \
        bash "$target" 64M python3 -c "$hog_code" 512 >/dev/null 2>"$scratch/oom-first-hit.err"; then status=0; else status=$?; fi
    if in_context env CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_KEY=selftest-plain-hit bash "$target" 64M python3 -c "$hog_code" 512 \
        >/dev/null 2>"$scratch/plain-hit.err"; then plain_status=0; else plain_status=$?; fi
    records="$(alert_check "$context_alerts" memory-own-cap selftest-oom-first-hit '{"count": 0}')"
    plain_records="$(alert_check "$context_alerts" memory-own-cap selftest-plain-hit "{\"session_id\": \"$peak_session\", \"cap\": \"64M\", \"booked_source\": \"cap\"}")"
    recent="$(recent_column selftest-oom-first-hit)"
    if [[ $status -ne $MEMORY_CAP_HIT_EXIT || $plain_status -ne $MEMORY_CAP_HIT_EXIT || "$records" != ok || "$plain_records" != ok || "${recent#*|}" != *:hit ]] \
        || ! grep -qF '撞了内存上限 64M' "$scratch/oom-first-hit.err"; then
      fail "设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的一条撞了自己的上限应当退 $MEMORY_CAP_HIT_EXIT、不写 memory-own-cap、stderr 照报「撞了内存上限 64M」、峰值表照记撞顶，没设的同样撞顶照旧写一条，\
实际它退 $status、记录：$records、近几次「$recent」（列数|第六列），没设的退 $plain_status、记录：$plain_records：$(cat "$scratch/oom-first-hit.err" "$scratch/plain-hit.err")"
    fi

    # 告警跟着峰值表走（文件头「峰值表」一段）：工作根是这一例的 work-root（fresh_context 设的 SINGLEFS_WORK_ROOT），它底下的 core-usage-alerts.jsonl 当「真记录」认。
    # 峰值表指到这一例的临时峰值表的一条撞顶写进旁边那一份、真记录不多一行；不设 RUN_WITH_MEMORY_CAP_PEAKS（默认峰值表随 XDG_DATA_HOME 换到这一例底下）的
    # 与经符号链接指到默认那一份的都写真记录、峰值表旁边不留；显式设了 SINGLEFS_CORE_USAGE_ALERTS 的照显式的写
    fresh_context 256M
    real_alerts="$context_directory/work-root/core-usage-alerts.jsonl"
    checked=$((checked + 1))
    if in_context env CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_KEY=selftest-alerts-beside-peaks bash "$target" 64M python3 -c "$hog_code" 512 \
        >/dev/null 2>"$scratch/alerts-beside.err"; then status=0; else status=$?; fi
    records="$(alert_check "$context_alerts" memory-own-cap selftest-alerts-beside-peaks "{\"session_id\": \"$peak_session\", \"cap\": \"64M\"}")"
    real_records="$(alert_check "$real_alerts" memory-own-cap selftest-alerts-beside-peaks '{"count": 0}')"
    if [[ $status -ne $MEMORY_CAP_HIT_EXIT || "$records" != ok || "$real_records" != ok ]]; then
      fail "峰值表指到默认之外（这一例的临时峰值表）的一条撞顶应当往峰值表旁边的 peaks.tsv.alerts.jsonl 写一条 memory-own-cap、真记录不多一行，\
实际退 $status、旁边那一份：$records、真记录：$real_records：$(cat "$scratch/alerts-beside.err")"
    fi
    default_table="$context_directory/data/singlefs/memory-peaks.tsv"
    mkdir -p "$context_directory/data/singlefs"
    ln -s "$context_directory/data" "$context_directory/data-alias"
    for expectation in "selftest-alerts-default-unset|" "selftest-alerts-default-alias|$context_directory/data-alias/singlefs/memory-peaks.tsv"; do
      expected_key="${expectation%%|*}" alias_table="${expectation#*|}"
      if [[ -n "$alias_table" ]]; then
        peaks_setting=(RUN_WITH_MEMORY_CAP_PEAKS="$alias_table") case_label="经符号链接 $alias_table 指到默认峰值表"
      else
        peaks_setting=(-u RUN_WITH_MEMORY_CAP_PEAKS) case_label="不设 RUN_WITH_MEMORY_CAP_PEAKS（默认峰值表在这一例的 XDG_DATA_HOME 底下）"
      fi
      checked=$((checked + 1))
      if in_context env "${peaks_setting[@]}" XDG_DATA_HOME="$context_directory/data" CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_KEY="$expected_key" \
          bash "$target" 64M python3 -c "$hog_code" 512 >/dev/null 2>"$scratch/$expected_key.err"; then status=0; else status=$?; fi
      real_records="$(alert_check "$real_alerts" memory-own-cap "$expected_key" "{\"session_id\": \"$peak_session\", \"cap\": \"64M\"}")"
      records="$(alert_check "$default_table.alerts.jsonl" memory-own-cap "$expected_key" '{"count": 0}')"
      if [[ $status -ne $MEMORY_CAP_HIT_EXIT || "$real_records" != ok || "$records" != ok ]] || ! grep -qF "$expected_key" "$default_table" 2>/dev/null; then
        fail "${case_label}的一条撞顶应当把 memory-own-cap 写进真记录、\
默认峰值表旁边不留、峰值记进默认峰值表，实际退 $status、真记录：$real_records、默认峰值表旁边：$records、默认峰值表里$(grep -qF "$expected_key" "$default_table" 2>/dev/null && echo '有' || echo '没有')这一行：$(cat "$scratch/$expected_key.err")"
      fi
    done
    checked=$((checked + 1))
    if in_context env SINGLEFS_CORE_USAGE_ALERTS="$context_directory/explicit-alerts.jsonl" CLAUDE_CODE_SESSION_ID="$peak_session" RUN_WITH_MEMORY_CAP_KEY=selftest-alerts-explicit \
        bash "$target" 64M python3 -c "$hog_code" 512 >/dev/null 2>"$scratch/alerts-explicit.err"; then status=0; else status=$?; fi
    explicit_records="$(alert_check "$context_directory/explicit-alerts.jsonl" memory-own-cap selftest-alerts-explicit "{\"session_id\": \"$peak_session\"}")"
    records="$(alert_check "$context_alerts" memory-own-cap selftest-alerts-explicit '{"count": 0}')"
    real_records="$(alert_check "$real_alerts" memory-own-cap selftest-alerts-explicit '{"count": 0}')"
    if [[ $status -ne $MEMORY_CAP_HIT_EXIT || "$explicit_records" != ok || "$records" != ok || "$real_records" != ok ]]; then
      fail "显式设了 SINGLEFS_CORE_USAGE_ALERTS 的一条撞顶应当照显式的写一条 memory-own-cap、峰值表旁边与真记录都不写，\
实际退 $status、显式那一份：$explicit_records、峰值表旁边：$records、真记录：$real_records：$(cat "$scratch/alerts-explicit.err")"
    fi
  }
  # 只跑这几项（RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=peak-history，看它红在哪用）：跑完收尾、退出，不算整份自证通过
  if [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_ONLY:-}" == peak-history ]]; then
    peak_history_cells
    for slice in "${selftest_slices[@]}"; do
      systemctl --user stop "$slice" >/dev/null 2>&1
      systemctl --user revert "$slice" >/dev/null 2>&1
    done
    rm -rf -- "${scratch:?}"
    if ((failures)); then
      echo "  ✗ run-with-memory-cap.sh 自检（只跑峰值表近几次、峰值涨了、撞顶与被挤掉的告警那几项）$failures 处不对（查了 $checked 项；测的是 $target）"
      echo "  → 怎么办：照上面逐条改 admission 里的 need_of、finish、outcome_alerts、alerts_record_path 与 memory_peaks_lib.py 的 recorded_peak；RUN_WITH_MEMORY_CAP_BREAK 设着的话本来就该红"
      exit 1
    fi
    echo "  ✓ run-with-memory-cap.sh 自检（只跑峰值表近几次、峰值涨了、撞顶与被挤掉的告警那几项，不算整份自证通过）查了 $checked 项都对"
    exit 0
  fi

  # 这一条命令自己的临时目录（文件头「这一条命令自己的临时目录」一段）：假命令把拿到的 TMPDIR 写进 scratch 里的一份文件、往 TMPDIR 里写一份文件，再照各自的结局退；
  # 核包装的退出码、那一份 TMPDIR 在包装退出之后不在了、RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 底下一个不剩。
  # 自证可能套在外层包装里跑（外层给了 TMPDIR），各例一律 env -u TMPDIR、-u SINGLEFS_KEEP_TEST_TEMPORARY_FILES 起，只在点名的那一例设
  temporary_directory_cells() {
    local case_name case_status case_cell root recorded leftover caller code ending
    local -a case_environment=()
    code='printf "%s\n" "${TMPDIR-}" > "$1"; if [[ -n "${TMPDIR-}" ]]; then : > "$TMPDIR/left-by-command"; fi; '
    for case_name in normal nonzero killed cap-hit time-limit; do
      fresh_context 256M
      root="$scratch/temporary-root-$case_name"
      recorded="$scratch/temporary-$case_name.path"
      mkdir -p "$root"
      case_environment=()
      case "$case_name" in
        normal) ending='exit 0'; case_status=0; case_cell="命令正常退出之后，包装给它的临时目录应当删掉" ;;
        nonzero) ending='exit 7'; case_status=7; case_cell="命令退非 0 之后，包装给它的临时目录应当删掉" ;;
        killed) ending='kill -9 $$'; case_status=137; case_cell="命令被 kill -9 之后，包装给它的临时目录应当删掉" ;;
        cap-hit) ending='python3 -c "$2" 512'; case_status=$MEMORY_CAP_HIT_EXIT; case_cell="命令撞了 64M 上限被杀之后，包装给它的临时目录应当删掉" ;;
        time-limit) ending='sleep 30'; case_status=$TIME_LIMIT_HIT_EXIT; case_environment=(RUN_WITH_MEMORY_CAP_TIME_LIMIT=1)
          case_cell="命令超过限时被停之后，包装给它的临时目录应当删掉" ;;
      esac
      checked=$((checked + 1))
      if in_context env -u TMPDIR -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" ${case_environment[@]+"${case_environment[@]}"} \
          bash "$target" 64M bash -c "$code$ending" temporary-case "$recorded" "$hog_code" >/dev/null 2>"$scratch/temporary-$case_name.err"; then status=0; else status=$?; fi
      output="$(cat "$recorded" 2>/dev/null)"
      leftover="$(ls -A "$root" 2>/dev/null)"
      if [[ $status -ne $case_status || "$output" != "$root"/singlefs-run-with-memory-cap-* || -e "$output" || -n "$leftover" ]]; then
        fail "$case_cell（退 $case_status、命令拿到的 TMPDIR 在 $root 底下、跑完那一份不在了、$root 底下一个不剩），实际退 $status、命令拿到的 TMPDIR 是「$output」、\
那一份$([[ -n "$output" && -e "$output" ]] && echo "还在（里面：$(ls -A "$output" 2>/dev/null | tr '\n' ' ')）" || echo '不在')、$root 底下剩「$leftover」：$(head_characters 600 "$scratch/temporary-$case_name.err")"
      fi
    done
    # 设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：那一份留着（命令写的文件也在），stderr 打出它的路径；看完由这里删
    fresh_context 256M
    root="$scratch/temporary-root-kept"
    recorded="$scratch/temporary-kept.path"
    mkdir -p "$root"
    checked=$((checked + 1))
    if in_context env -u TMPDIR SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" \
        bash "$target" 64M bash -c "${code}exit 0" temporary-case "$recorded" >/dev/null 2>"$scratch/temporary-kept.err"; then status=0; else status=$?; fi
    output="$(cat "$recorded" 2>/dev/null)"
    if [[ $status -ne 0 || "$output" != "$root"/singlefs-run-with-memory-cap-* || ! -e "$output/left-by-command" ]] \
        || ! grep -qF "SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：这一条命令的临时目录留着看现场：$output" "$scratch/temporary-kept.err"; then
      fail "设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时临时目录应当留着、stderr 打出它的路径（退 0、$root 底下那一份与命令写的 left-by-command 都在），\
实际退 $status、命令拿到的 TMPDIR 是「$output」、那一份$([[ -n "$output" && -e "$output" ]] && echo '在' || echo '不在')：$(head_characters 600 "$scratch/temporary-kept.err")"
    fi
    rm -rf -- "${root:?}"
    # 调用方自己给了 TMPDIR：命令拿到的就是它，包装不另建、不删；那一份与调用方放进去的、命令写的文件都还在
    fresh_context 256M
    root="$scratch/temporary-root-caller"
    caller="$scratch/temporary-given-by-caller"
    recorded="$scratch/temporary-caller.path"
    mkdir -p "$root" "$caller"
    : > "$caller/placed-by-caller"
    checked=$((checked + 1))
    if in_context env -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES TMPDIR="$caller" RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" \
        bash "$target" 64M bash -c "${code}exit 0" temporary-case "$recorded" >/dev/null 2>"$scratch/temporary-caller.err"; then status=0; else status=$?; fi
    output="$(cat "$recorded" 2>/dev/null)"
    leftover="$(ls -A "$root" 2>/dev/null)"
    if [[ $status -ne 0 || "$output" != "$caller" || ! -e "$caller/placed-by-caller" || ! -e "$caller/left-by-command" || -n "$leftover" ]]; then
      fail "调用方设了 TMPDIR 时命令应当拿到调用方那一份、跑完那一份与里面的文件都还在（退 0、拿到 $caller、$root 底下不另建），\
实际退 $status、命令拿到的 TMPDIR 是「$output」、调用方那一份$([[ -e "$caller" ]] && echo "在（里面：$(ls -A "$caller" 2>/dev/null | tr '\n' ' ')）" || echo '不在了')、\
$root 底下剩「$leftover」：$(head_characters 600 "$scratch/temporary-caller.err")"
    fi
  }
  # 只跑这几项（RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=temporary-directory，看它红在哪用）：跑完收尾、退出，不算整份自证通过
  if [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_ONLY:-}" == temporary-directory ]]; then
    temporary_directory_cells
    for slice in "${selftest_slices[@]}"; do
      systemctl --user stop "$slice" >/dev/null 2>&1
      systemctl --user revert "$slice" >/dev/null 2>&1
    done
    rm -rf -- "${scratch:?}"
    if ((failures)); then
      echo "  ✗ run-with-memory-cap.sh 自检（只跑这一条命令自己的临时目录那几项）$failures 处不对（查了 $checked 项；测的是 $target）"
      echo "  → 怎么办：照上面逐条改 begin_own_temporary_directory / end_own_temporary_directory 与主流程里挂 EXIT trap 的那两行；RUN_WITH_MEMORY_CAP_BREAK 设着的话本来就该红"
      exit 1
    fi
    echo "  ✓ run-with-memory-cap.sh 自检（只跑这一条命令自己的临时目录那几项，不算整份自证通过）查了 $checked 项都对"
    exit 0
  fi

  # 资源释放广播（文件头「资源释放广播」）：事件目录是这一例自己的（RESOURCE_BROADCAST_DIRECTORY），不推（RESOURCE_BROADCAST_PUSH=off）；
  # 读写事件只用 grep 与 printf，不起那份库（文件头那一段末尾）。排队那几格借「排队时去找别的机器」的假世界（host_lookup_world）
  broadcast_count() { # broadcast_count <事件目录> <grep -E 的式子>：事件文件里合这个式子的行数
    cat "$1"/*.event 2>/dev/null | grep -cE -- "$2"
  }
  broadcast_write_event() { # broadcast_write_event <事件目录> <序号> <种类> <来源>：照那份库的写法放一条事件（先写临时文件再改名，last-sequence 跟着改）
    mkdir -p "$1"
    printf 'seq=%s kind=%s host=第二台 source=%s label=selftest-broadcast exit=0 time=%s date=%s\n' "$2" "$3" "$4" "${EPOCHREALTIME/,/.}" "$(TZ=Asia/Tokyo date +%F)" \
      > "$1/.selftest-$2.partial"
    mv -f -- "$1/.selftest-$2.partial" "$1/$(printf '%012d' "$2").event"
    printf '%s\n' "$2" > "$1/.selftest-last-sequence.partial"
    mv -f -- "$1/.selftest-last-sequence.partial" "$1/last-sequence"
  }
  broadcast_wait_for() { # broadcast_wait_for <文件> <grep -E 的式子> <至多几个 0.1 秒>：等到那一行出现退 0
    local poll
    for ((poll = 0; poll < $3; poll++)); do
      grep -qE -- "$2" "$1" 2>/dev/null && return 0
      sleep 0.1
    done
    return 1
  }
  broadcast_cells() {
    local events err status wrapper_pid world lines_count records woke other_pid other_err other_status
    local -a broadcast_context=()
    # 命令真跑过、退 3：发一条 memory-released，标签是 scope 单元名、带退出码与峰值
    fresh_context 256M
    events="$context_directory/events"
    broadcast_context=(RESOURCE_BROADCAST_DIRECTORY="$events" RESOURCE_BROADCAST_PUSH=off)
    checked=$((checked + 1))
    if in_context "${broadcast_context[@]}" bash "$target" 64M bash -c 'exit 3' 2>"$scratch/broadcast-ran.err"; then status=0; else status=$?; fi
    lines_count="$(broadcast_count "$events" '^seq=[0-9]+ kind=memory-released host=本机 source=local label=singlefs-memory-cap-[0-9]+-[0-9]+ exit=3 memory_bytes=[0-9]+ ')"
    if [[ $status -ne 3 || "$lines_count" != 1 ]]; then
      fail "命令真跑过、退 3 之后应当发一条 memory-released（标签是 scope 单元名、exit=3、带峰值）、包装照退 3，实际退 $status、合的事件 ${lines_count:-0} 条：$(cat "$events"/*.event 2>/dev/null)"
    fi
    # 排不上当场退 252（要 512M、总上限 256M）：它没占过资源，不发
    checked=$((checked + 1))
    rm -rf -- "${events:?}"
    if in_context "${broadcast_context[@]}" bash "$target" 512M true 2>"$scratch/broadcast-refused.err"; then status=0; else status=$?; fi
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT ]] || compgen -G "$events/*.event" > /dev/null; then
      fail "排不上当场退 $MEMORY_ADMISSION_REFUSED_EXIT 的一条应当不发资源释放广播，实际退 $status、事件：$(cat "$events"/*.event 2>/dev/null)"
    fi
    # 起了 scope 之后包装被 TERM 停（只停这里 & 起、\$! 记下的那个包装）：退 143、发一条 exit=143
    checked=$((checked + 1))
    rm -rf -- "${events:?}"
    env "${context[@]}" "${broadcast_context[@]}" bash "$target" 64M bash -c ': > "$1"; sleep 30' term-probe "$scratch/broadcast-term.started" 2>"$scratch/broadcast-term.err" &
    wrapper_pid=$!
    for ((woke = 0; woke < 100; woke++)); do [[ -e "$scratch/broadcast-term.started" ]] && break; sleep 0.1; done
    kill -TERM "$wrapper_pid" 2>/dev/null
    if wait "$wrapper_pid"; then status=0; else status=$?; fi
    lines_count="$(broadcast_count "$events" ' kind=memory-released .* exit=143 ')"
    if [[ $status -ne 143 || "$lines_count" != 1 ]]; then
      fail "起了 scope 之后包装被 TERM 停应当退 143、发一条 memory-released exit=143，实际退 $status、合的事件 ${lines_count:-0} 条：$(cat "$events"/*.event 2>/dev/null)；stderr：$(cat "$scratch/broadcast-term.err")"
    fi
    # 那份库出事（只给自证的 RESOURCE_BROADCAST_SELFTEST_FAULT=crash，带那份库认的标记）：包装照退命令的 3，广播那一步打的一行都不进包装的 stderr
    checked=$((checked + 1))
    rm -rf -- "${events:?}"
    printf '%s\n' "resource_release_broadcast.py 自证的标记" > "$scratch/broadcast-mark"
    if in_context "${broadcast_context[@]}" RESOURCE_BROADCAST_SELFTEST_FAULT=crash RESOURCE_BROADCAST_SELFTEST_MARK="$scratch/broadcast-mark" \
        bash "$target" 64M bash -c 'exit 3' 2>"$scratch/broadcast-fault.err"; then status=0; else status=$?; fi
    if [[ $status -ne 3 ]] || grep -qE '自证造的崩溃|没广播到|资源释放广播' "$scratch/broadcast-fault.err"; then
      fail "资源释放广播那份库出事（自证造的崩溃）时包装应当照退命令的 3、stderr 里没有广播那一步的行，实际退 $status：$(cat "$scratch/broadcast-fault.err")"
    fi
    # 套在外层包装里：里层跑完不发，外层照发一条
    checked=$((checked + 1))
    rm -rf -- "${events:?}"
    if in_context "${broadcast_context[@]}" bash "$target" 128M bash "$target" 32M true 2>"$scratch/broadcast-nested.err"; then status=0; else status=$?; fi
    lines_count="$(broadcast_count "$events" ' kind=memory-released ')"
    if [[ $status -ne 0 || "$lines_count" != 1 ]]; then
      fail "套在外层包装里跑完的一条应当不发、外层照发一条，实际退 $status、事件 ${lines_count:-0} 条：$(cat "$events"/*.event 2>/dev/null)"
    fi
    # 排队中收到广播（第二台先放不下）：本机自己发的内存放出不另查；第二台那一趟跑完、第二台此刻放得下：当场再查、打改去第二台的命令、写 phase=queued
    fresh_context 256M
    events="$context_directory/events"
    world="$scratch/broadcast-wake"
    host_lookup_world "$world" 1
    err="$scratch/host-lookup-broadcast-wake.err"
    host_lookup_blocker
    host_lookup_command "$world" broadcast-wake 40 RESOURCE_BROADCAST_DIRECTORY="$events" RESOURCE_BROADCAST_PUSH=off
    "${host_lookup_run[@]}" 2>"$err" &
    wrapper_pid=$!
    broadcast_wait_for "$err" '现查别的机器：第二台此刻也放不下' 200
    broadcast_write_event "$events" 1 memory-released local
    sleep 3
    checked=$((checked + 1))
    if grep -q '被资源释放广播叫醒' "$err"; then
      fail "排队中收到本机自己发的内存放出应当不另查别的机器，实际 stderr：$(cat "$err")"
    fi
    printf 'MemTotal: %d kB\nMemAvailable: %d kB\n' $((60 * 1024 * 1024)) $((43 * 1024 * 1024)) > "$world/peer/proc/meminfo"
    broadcast_write_event "$events" 2 peer-run-finished local
    checked=$((checked + 1))
    if broadcast_wait_for "$err" '此刻放得下.*被资源释放广播叫醒：第 2 号，peer-run-finished' 150; then woke=0; else woke=1; fi
    rm -f -- "$context_directory/state/ledger/singlefs-memory-cap-selftest-blocker.json"
    if wait "$wrapper_pid"; then status=0; else status=$?; fi
    records="$(python3 -c 'import json, sys
try:
    rows = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8")]
except OSError as error:
    print(f"读不了：{error}")
    sys.exit(0)
print("ok" if any(row.get("phase") == "queued" and row.get("kind") == "memory-queue-peer-fits" and "被资源释放广播叫醒" in row.get("line", "") for row in rows)
      else json.dumps(rows, ensure_ascii=False)[:600])' "$world/alerts.jsonl" 2>&1)"
    if [[ $woke -ne 0 || $status -ne 0 || "$records" != ok ]]; then
      fail "排队中收到第二台那一趟跑完的广播应当当场再查别的机器：第二台放得下时打改去第二台的命令（行尾注被资源释放广播叫醒）、写 phase=queued、照旧排队到放行退 0，\
实际$([[ $woke -eq 0 ]] && echo '打了' || echo '没打')那一行、退 $status、告警记录：$records；stderr：$(cat "$err")"
    fi
    # 几条一起排队（第二台一直放得下）：一个事件只查一次，另一条打「第 1 号资源释放广播包装 … 已经查过」
    fresh_context 256M
    events="$context_directory/events"
    world="$scratch/broadcast-throttled"
    host_lookup_world "$world" 43
    err="$scratch/host-lookup-broadcast-throttled-a.err"
    other_err="$scratch/host-lookup-broadcast-throttled-b.err"
    host_lookup_blocker
    host_lookup_command "$world" broadcast-throttled-a 40 RESOURCE_BROADCAST_DIRECTORY="$events" RESOURCE_BROADCAST_PUSH=off
    "${host_lookup_run[@]}" 2>"$err" &
    wrapper_pid=$!
    broadcast_wait_for "$err" '此刻放得下' 200
    host_lookup_command "$world" broadcast-throttled-b 40 RESOURCE_BROADCAST_DIRECTORY="$events" RESOURCE_BROADCAST_PUSH=off
    "${host_lookup_run[@]}" 2>"$other_err" &
    other_pid=$!
    broadcast_wait_for "$other_err" '刚查过' 200
    broadcast_write_event "$events" 1 peer-run-finished local
    broadcast_wait_for "$err" '被资源释放广播叫醒' 150
    broadcast_wait_for "$other_err" '被资源释放广播叫醒' 150
    sleep 1
    checked=$((checked + 1))
    lines_count="$(cat "$err" "$other_err" | grep -c '此刻放得下.*被资源释放广播叫醒：第 1 号')"
    records="$(cat "$err" "$other_err" | grep -c '没查到别的机器：第 1 号资源释放广播包装 [0-9]* 已经查过')"
    rm -f -- "$context_directory/state/ledger/singlefs-memory-cap-selftest-blocker.json"
    if wait "$wrapper_pid"; then status=0; else status=$?; fi
    if wait "$other_pid"; then other_status=0; else other_status=$?; fi
    if [[ "$lines_count" != 1 || "$records" != 1 || $status -ne 0 || $other_status -ne 0 ]]; then
      fail "几条一起排队时一个资源释放广播只查一次：一条打改去第二台的命令、另一条打「第 1 号资源释放广播包装 … 已经查过」，两条照旧放行退 0，\
实际查了 ${lines_count:-0} 次、已经查过 ${records:-0} 行、两条退 $status 与 $other_status；stderr：$(cat "$err")｜$(cat "$other_err")"
    fi
    # 叫醒那一趟起线程时抛异常（只给自证的 RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_FAULT=wake-thread）：只打一行没查到别的机器，排队照旧等满 6 秒退 252
    fresh_context 256M
    events="$context_directory/events"
    world="$scratch/broadcast-wake-fault"
    host_lookup_world "$world" 1
    err="$scratch/host-lookup-broadcast-wake-fault.err"
    host_lookup_blocker
    host_lookup_command "$world" broadcast-wake-fault 6 RESOURCE_BROADCAST_DIRECTORY="$events" RESOURCE_BROADCAST_PUSH=off RUN_WITH_MEMORY_CAP_SELFTEST_LOOKUP_FAULT=wake-thread
    timeout 30 "${host_lookup_run[@]}" 2>"$err" &
    wrapper_pid=$!
    broadcast_wait_for "$err" '现查别的机器：第二台此刻也放不下' 200
    broadcast_write_event "$events" 1 peer-run-finished local
    checked=$((checked + 1))
    if wait "$wrapper_pid"; then status=0; else status=$?; fi
    rm -f -- "$context_directory/state/ledger/singlefs-memory-cap-selftest-blocker.json"
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT ]] || grep -q '^Traceback' "$err" || ! grep -q '没查到别的机器：被资源释放广播叫醒那一趟起线程时出了 RuntimeError' "$err"; then
      fail "被资源释放广播叫醒那一趟起线程时抛异常应当只打一行没查到别的机器、退出码照旧（等满 6 秒退 $MEMORY_ADMISSION_REFUSED_EXIT），实际退 $status（124 是 30 秒到了被停）：$(cat "$err")"
    fi
    # 排队中 follow 出事（那份库认的自证标记加 RESOURCE_BROADCAST_SELFTEST_FAULT=crash：follow 一起来就退 1）：HostLookup.watch_events 起的那条子进程边上的异常
    #   只打一行没查到别的机器，排队照旧每 ADMISSION_POLL_SECONDS 秒重判、等满 6 秒退 252（判法单元的闭包不走这条边：scripts/gate/lib_function_units.py 的 EXCLUDED_PROGRAM_EDGES）
    fresh_context 256M
    events="$context_directory/events"
    world="$scratch/broadcast-follow-fault"
    host_lookup_world "$world" 1
    checked=$((checked + 1))
    host_lookup_case "$world" broadcast-follow-fault 6 RESOURCE_BROADCAST_DIRECTORY="$events" RESOURCE_BROADCAST_PUSH=off RESOURCE_BROADCAST_SELFTEST_FAULT=crash \
      RESOURCE_BROADCAST_SELFTEST_MARK="$scratch/broadcast-mark"
    err="$scratch/host-lookup-broadcast-follow-fault.err"
    lines_count="$(grep -c '没查到别的机器：等资源释放广播时出了 RuntimeError（资源释放广播的 follow 退了' "$err")"
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || "$lines_count" != 1 ]] || grep -qE '^Traceback|自证造的崩溃' "$err"; then
      fail "排队中资源释放广播的 follow 出事（自证造的崩溃）应当只打一行没查到别的机器、退出码照旧（等满 6 秒退 $MEMORY_ADMISSION_REFUSED_EXIT），实际退 $status、那一行 ${lines_count:-0} 条：$(cat "$err")"
    fi
  }
  # 只跑这几项（RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=broadcast，看它红在哪用）：跑完收尾、退出，不算整份自证通过
  if [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_ONLY:-}" == broadcast ]]; then
    broadcast_cells
    for slice in "${selftest_slices[@]}"; do
      systemctl --user stop "$slice" >/dev/null 2>&1
      systemctl --user revert "$slice" >/dev/null 2>&1
    done
    rm -rf -- "${scratch:?}"
    if ((failures)); then
      echo "  ✗ run-with-memory-cap.sh 自检（只跑资源释放广播那几项）$failures 处不对（查了 $checked 项；测的是 $target）"
      echo "  → 怎么办：照上面逐条改 broadcast_memory_released / on_wrapper_exit 与 HostLookup 的 watch_events、pause、wake、take_turn；RUN_WITH_MEMORY_CAP_BREAK 设着的话本来就该红"
      exit 1
    fi
    echo "  ✓ run-with-memory-cap.sh 自检（只跑资源释放广播那几项，不算整份自证通过）查了 $checked 项都对"
    exit 0
  fi

  # 被总上限挤掉、内核挑谁那几项（RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=slice-total 连同 over_booking_cells 只跑这几项）。
  # 判内核那一支的几例（挤掉、按峰值表放进来涨过总上限、OOM_FIRST）各条都设 RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY：不盯账、slice 不设 memory.high，
  # 撞总上限全由内核按 oom_score_adj（按记账算）挑；盯账停的那一支见 over_booking_cells
  slice_total_cells() {
  # 被总上限挤掉（第 ② 层兜住、退 254，不报成撞了自己的上限）：表里把这两条记成 10 MiB，排队一起放行，实际各拿 150 MiB，合起来超总上限 256M
  fresh_context 256M
  checked=$((checked + 1))
  seed_peak selftest-underrecorded $((10 * MEBIBYTE))
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_KEY=selftest-underrecorded RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='自检排队这一例故意上限远大于峰值' bash "$target" 200M python3 -c "$job_code" "$scratch/squeeze-1.start" "$scratch/squeeze-1.end" 0.3 150 1.5 \
    >/dev/null 2>"$scratch/squeeze-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_KEY=selftest-underrecorded RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='自检排队这一例故意上限远大于峰值' bash "$target" 200M python3 -c "$job_code" "$scratch/squeeze-2.start" "$scratch/squeeze-2.end" 0.3 150 1.5 \
    >/dev/null 2>"$scratch/squeeze-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne $SLICE_TOTAL_HIT_EXIT && $second_status -ne $SLICE_TOTAL_HIT_EXIT ]] || [[ $first_status -eq $MEMORY_CAP_HIT_EXIT || $second_status -eq $MEMORY_CAP_HIT_EXIT ]] \
      || ! grep -q '总上限挤掉' "$scratch/squeeze-1.err" "$scratch/squeeze-2.err"; then
    fail "两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 $SLICE_TOTAL_HIT_EXIT、报「被总上限挤掉」，实际退 $first_status 与 $second_status：$(cat "$scratch/squeeze-1.err" "$scratch/squeeze-2.err")"
  fi

  # 撞过顶的键按自己的上限排队（文件头「排队」一段）：上一次撞了 200M 的键，这一次上限 300M、总上限 256M，按 300M 算放不下，当场退 252，
  # 不按峰值表里的 200M 放进去再涨过 slice 的总上限（改前退 254）
  fresh_context 256M
  checked=$((checked + 1))
  seed_peak selftest-hit-before $((200 * MEBIBYTE))
  started_seconds="$(date +%s%N)"
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-hit-before bash "$target" 300M python3 -c "$hog_code" 400 >/dev/null 2>"$scratch/hit-over-slice.err"; then status=0; else status=$?; fi
  elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
  if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || $elapsed_seconds -ge 5 ]] || ! grep -q '这个键撞过顶' "$scratch/hit-over-slice.err"; then
    fail "撞过顶的键（上一次撞了 200M）这一次上限 300M、总上限 256M 时应当不放进去、当场退 $MEMORY_ADMISSION_REFUSED_EXIT、报「这个键撞过顶」，实际退 $status、用了 $elapsed_seconds 秒：$(cat "$scratch/hit-over-slice.err")"
  fi

  # 升档那趟（设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP）按自己的上限排队：常驻那一条按上限 160M 放进来、占着 150 MiB，升档那趟峰值表里记 20 MiB、上限 200M，
  # 按 200M 算放不下，等常驻那一条跑完再起，撞的是它自己的上限（退 250），常驻那一条照常退 0
  fresh_context 256M
  checked=$((checked + 1))
  seed_peak selftest-escalated $((20 * MEBIBYTE))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-resident bash "$target" 160M python3 -c "$job_code" "$scratch/queue-cap-1.start" "$scratch/queue-cap-1.end" 0 150 3 \
    >/dev/null 2>"$scratch/queue-cap-1.err" &
  first_pid=$!
  sleep 1
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-escalated RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP='升档那趟（自检）' RUN_WITH_MEMORY_CAP_WAIT_SECONDS=30 \
      RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='自检排队这一例故意上限远大于峰值' bash "$target" 200M python3 -c "$job_code" "$scratch/queue-cap-2.start" "$scratch/queue-cap-2.end" 0 300 0 \
      >/dev/null 2>"$scratch/queue-cap-2.err"; then status=0; else status=$?; fi
  wait "$first_pid"; first_status=$?
  if [[ $first_status -ne 0 || $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! all_exist "$scratch/queue-cap-1.end" "$scratch/queue-cap-2.start" \
      || ! python3 -c 'import sys; sys.exit(0 if float(open(sys.argv[2]).read()) >= float(open(sys.argv[1]).read()) else 1)' "$scratch/queue-cap-1.end" "$scratch/queue-cap-2.start" 2>/dev/null \
      || ! grep -q 'RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP' "$scratch/queue-cap-2.err"; then
    fail "设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP 的那一条应当按自己的上限 200M 排队、等常驻那一条跑完再起、撞自己的上限退 $MEMORY_CAP_HIT_EXIT，常驻那一条退 0，实际常驻退 $first_status、它退 $status：$(cat "$scratch/queue-cap-1.err" "$scratch/queue-cap-2.err")"
  fi

  # 按峰值表放进来的那一条涨过 slice 的总上限：内核杀它（它记账 20 MiB、超出记账最多，按记账算的 oom_score_adj 让它分数最高），不杀照上限放进来的常驻那一条；
  # 这个键随之记成撞过顶（峰值记成上限 200M），重跑时按 200M 排队，等常驻那一条跑完再起、撞自己的上限退 250
  fresh_context 256M
  checked=$((checked + 1))
  seed_peak selftest-grower $((20 * MEBIBYTE))
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_KEY=selftest-resident bash "$target" 160M python3 -c "$job_code" "$scratch/collateral-1.start" "$scratch/collateral-1.end" 0 150 4 \
    >/dev/null 2>"$scratch/collateral-1.err" &
  first_pid=$!
  sleep 1
  if in_context env RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_KEY=selftest-grower RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='自检排队这一例故意上限远大于峰值' bash "$target" 200M python3 -c "$hog_code" 400 \
      >/dev/null 2>"$scratch/collateral-2.err"; then status=0; else status=$?; fi
  wait "$first_pid"; first_status=$?
  if [[ $first_status -ne 0 || $status -ne $SLICE_TOTAL_HIT_EXIT ]]; then
    fail "按峰值表放进来的那一条涨过 slice 的总上限时，内核应当杀它（退 $SLICE_TOTAL_HIT_EXIT）、不杀照上限放进来的常驻那一条（退 0），实际常驻退 $first_status、它退 $status：$(cat "$scratch/collateral-1.err" "$scratch/collateral-2.err")"
  fi
  checked=$((checked + 1))
  output="$(awk -F '\t' '$4 == "selftest-grower" {print $1 "\t" $3}' "$context_directory/peaks.tsv" 2>/dev/null)"
  [[ "$output" == "$((200 * MEBIBYTE))"$'\t'"200M" ]] \
    || fail "按峰值表放进来又被总上限挤掉之后，峰值表里这个键应当记成它的上限 200M（$((200 * MEBIBYTE))），实际「$output」"
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_KEY=selftest-resident bash "$target" 160M python3 -c "$job_code" "$scratch/collateral-3.start" "$scratch/collateral-3.end" 0 150 3 \
    >/dev/null 2>"$scratch/collateral-3.err" &
  first_pid=$!
  sleep 1
  if in_context env RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_KEY=selftest-grower RUN_WITH_MEMORY_CAP_WAIT_SECONDS=30 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='自检排队这一例故意上限远大于峰值' \
      bash "$target" 200M python3 -c "$job_code" "$scratch/collateral-4.start" "$scratch/collateral-4.end" 0 300 0 >/dev/null 2>"$scratch/collateral-4.err"; then status=0; else status=$?; fi
  wait "$first_pid"; first_status=$?
  if [[ $first_status -ne 0 || $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! all_exist "$scratch/collateral-3.end" "$scratch/collateral-4.start" \
      || ! python3 -c 'import sys; sys.exit(0 if float(open(sys.argv[2]).read()) >= float(open(sys.argv[1]).read()) else 1)' "$scratch/collateral-3.end" "$scratch/collateral-4.start" 2>/dev/null; then
    fail "撞过顶的键重跑时应当按自己的上限排队、等常驻那一条跑完再起、撞自己的上限退 $MEMORY_CAP_HIT_EXIT，常驻那一条退 0，实际常驻退 $first_status、它退 $status：$(cat "$scratch/collateral-3.err" "$scratch/collateral-4.err")"
  fi

  # 两条都按峰值表放进来（常驻那一条记 140 MiB、上限 160M、占 150 MiB；另一条记 20 MiB、上限 200M）：设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条
  # （广谱变异的变异步）涨过总上限时内核杀它（1000 对按记账算的那个数），不杀更大的常驻那一条
  fresh_context 256M
  checked=$((checked + 1))
  seed_peak selftest-resident-recorded $((140 * MEBIBYTE))
  seed_peak selftest-oom-first $((20 * MEBIBYTE))
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_KEY=selftest-resident-recorded bash "$target" 160M python3 -c "$job_code" "$scratch/oom-first-1.start" "$scratch/oom-first-1.end" 0 150 4 \
    >/dev/null 2>"$scratch/oom-first-1.err" &
  first_pid=$!
  sleep 1
  if in_context env RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_KEY=selftest-oom-first RUN_WITH_MEMORY_CAP_OOM_FIRST='广谱变异的变异步（自检）' RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='自检排队这一例故意上限远大于峰值' \
      bash "$target" 200M python3 -c "$hog_code" 400 >/dev/null 2>"$scratch/oom-first-2.err"; then status=0; else status=$?; fi
  wait "$first_pid"; first_status=$?
  if [[ $first_status -ne 0 || $status -ne $SLICE_TOTAL_HIT_EXIT ]]; then
    fail "两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条涨过总上限时，内核应当杀它（退 $SLICE_TOTAL_HIT_EXIT）、不杀更大的常驻那一条（退 0），实际常驻退 $first_status、它退 $status：$(cat "$scratch/oom-first-1.err" "$scratch/oom-first-2.err")"
  fi
  # 有意撞顶的跑法挨挤也不写 memory-squeezed（文件头「峰值表」一段）：只判记录，退出码由上面那一格判
  checked=$((checked + 1))
  records="$(alert_check "$context_alerts" memory-squeezed selftest-oom-first '{"count": 0}')"
  [[ "$records" == ok ]] \
    || fail "两条都按峰值表放进来、设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的那一条挨挤之后应当不写 memory-squeezed，实际它退 $status、记录：$records"
  }

  # 盯账（文件头「盯账」一段）：每一例一个总上限 400M 的新 slice（触发线与 memory.high 都是 380 MiB）。常驻那一条（键 selftest-booked-steady）记账 240 MiB、
  # 上限 300M、拿 240 MiB（这台上 python 外加外壳约 5 MiB，此刻约 245 MiB，超出记账约 5 MiB，在容差 16 MiB 以内）占 8 秒；它拿够之后起另一条，上限 300M，
  # 记账 40 MiB 的涨到 200 MiB（超账）、记账 140 MiB 的拿 140 MiB（不超账）。两条的记账合计都放得进 400M，排队一起放行。
  # 判：谁退什么、谁的 stderr 有盯账那一句、slice 的 memory.events 里 oom_kill 还是不是 0（内核一次都没 OOM）、峰值表里谁记成了撞过顶。
  over_booking_cells() {
    local grower_code steady_status other_status steady_err other_err kills high_now steady_recent other_row trace
    local -a steady_env=() other_env=()
    # 涨内存的作业（argv：收尾时刻写到哪、几个进程、每个进程涨到几 MiB、每步几 MiB、每步隔几秒、涨够之后占几秒）：几个进程是 fork 出来的，同在这一条的 scope 里
    grower_code='import os, sys, time
end_path, processes, per_process, step, step_seconds, hold_seconds = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]), float(sys.argv[5]), float(sys.argv[6])
def grow():
    held = []
    while len(held) * step < per_process:
        held.append(bytes([1]) * (step * 1024 * 1024))
        time.sleep(step_seconds)
    time.sleep(hold_seconds)
children = []
for _ in range(processes - 1):
    child = os.fork()
    if child == 0:
        grow()
        os._exit(0)
    children.append(child)
grow()
codes = [os.waitpid(child, 0)[1] for child in children]
if not any(codes):
    open(end_path, "w").write(repr(time.time()))
sys.exit(1 if any(codes) else 0)'
    slice_event() { # slice_event <名字> → 这一例 slice 的 memory.events 里那一项（含它底下的 scope）；读不到交空
      local group
      group="$(systemctl --user show -p ControlGroup --value "$slice" 2>/dev/null)"
      [[ -n "$group" ]] && awk -v wanted="$1" '$1 == wanted {print $2}' "/sys/fs/cgroup$group/memory.events" 2>/dev/null
    }
    # over_booking_pair <名> <另一条的键> <另一条的命令…>：起常驻那一条，等它拿够再起另一条（前台），之后等常驻那一条；
    # 退出码进 steady_status、other_status，stderr 进 steady_err、other_err。两条各带 steady_env、other_env（各例自己设）
    over_booking_pair() {
      local name="$1" other_key="$2" poll
      shift 2
      steady_err="$scratch/over-booking-$name-steady.err" other_err="$scratch/over-booking-$name-other.err"
      in_context env CLAUDE_CODE_SESSION_ID=selftest-over-booking-session RUN_WITH_MEMORY_CAP_KEY=selftest-booked-steady ${steady_env[@]+"${steady_env[@]}"} \
        bash "$target" 300M python3 -c "$job_code" "$scratch/over-booking-$name-steady.start" "$scratch/over-booking-$name-steady.end" 0 240 8 >/dev/null 2>"$steady_err" &
      first_pid=$!
      for ((poll = 0; poll < 100; poll++)); do
        [[ -e "$scratch/over-booking-$name-steady.start" ]] && break
        sleep 0.1
      done
      sleep 1
      if in_context env CLAUDE_CODE_SESSION_ID=selftest-over-booking-session RUN_WITH_MEMORY_CAP_KEY="$other_key" ${other_env[@]+"${other_env[@]}"} \
          bash "$target" 300M "$@" >/dev/null 2>"$other_err"; then other_status=0; else other_status=$?; fi
      wait "$first_pid"; steady_status=$?
    }
    over_booking_context() { # over_booking_context：总上限 400M 的新 slice，峰值表里记好三条的记账（上限都是 300M）
      fresh_context 400M
      seed_peak selftest-booked-steady $((240 * MEBIBYTE)) 300M
      seed_peak selftest-booked-grower $((40 * MEBIBYTE)) 300M
      seed_peak selftest-booked-within $((140 * MEBIBYTE)) 300M
    }
    # 判常驻那一条照常跑完、另一条被盯账停了（退 254、stderr 有盯账那一句）、内核没 OOM、另一条的键记成撞过顶而常驻那一条的不记；不对就交回非 0，问题写进 over_booking_problem
    over_booking_stopped_right() { # over_booking_stopped_right <名>
      over_booking_problem=""
      kills="$(slice_event oom_kill)"
      steady_recent="$(recent_column selftest-booked-steady)"
      other_row="$(awk -F '\t' '$4 == "selftest-booked-grower" {print $1 "|" $6}' "$context_directory/peaks.tsv" 2>/dev/null)"
      [[ $steady_status -eq 0 && -e "$scratch/over-booking-$1-steady.end" ]] || over_booking_problem+="常驻那一条退 $steady_status$([[ -e "$scratch/over-booking-$1-steady.end" ]] || echo '、没跑完')；"
      [[ $other_status -eq $SLICE_TOTAL_HIT_EXIT ]] && grep -q '超出记账最多，包装在撞总上限之前停了它' "$other_err" || over_booking_problem+="超账那一条退 $other_status、stderr 里$(grep -q '超出记账最多' "$other_err" && echo '有' || echo '没有')盯账那一句；"
      ! grep -q '超出记账最多' "$steady_err" || over_booking_problem+="常驻那一条被盯账停了；"
      [[ "$kills" == 0 ]] || over_booking_problem+="slice 的 oom_kill 是「$kills」（不是 0：内核 OOM 过，或读不到）；"
      [[ "$steady_recent" != *:hit* ]] || over_booking_problem+="常驻那一条的键记成了撞过顶（近几次「$steady_recent」）；"
      [[ "${other_row%%|*}" == "$((300 * MEBIBYTE))" && "${other_row#*|}" == *:hit* ]] || over_booking_problem+="超账那一条的键没记成撞过顶（峰值|近几次「$other_row」）；"
      [[ -z "$over_booking_problem" ]]
    }
    local over_booking_problem=""

    # 单进程超账：另一条记账 40 MiB、每 0.15 秒涨 10 MiB 涨到 200 MiB；到触发线时它超出记账约 95 MiB，盯账停它
    over_booking_context
    checked=$((checked + 1))
    steady_env=() other_env=()
    over_booking_pair single selftest-booked-grower python3 -c "$grower_code" "$scratch/over-booking-single-other.end" 1 200 10 0.15 1
    over_booking_stopped_right single \
      || fail "单进程超账（记账 40 MiB 的那一条涨到 200 MiB）时盯账应当只停它（退 $SLICE_TOTAL_HIT_EXIT、报「超出记账最多，包装在撞总上限之前停了它」、键记成撞过顶），\
没超账的常驻那一条退 0、键不记撞过顶，内核一次都不 OOM，实际：$over_booking_problem$(cat "$other_err" "$steady_err")"
    # 盯账停的那一条写一条 memory-squeezed，stopped_by=over-booking-watch；常驻那一条不写
    checked=$((checked + 1))
    records="$(alert_check "$context_alerts" memory-squeezed selftest-booked-grower \
      "{\"session_id\": \"selftest-over-booking-session\", \"cap\": \"300M\", \"booked\": $((40 * MEBIBYTE)), \"booked_source\": \"peak-table\", \"by_peak_table\": true, \"stopped_by\": \"over-booking-watch\", \"measured_at_least\": $((40 * MEBIBYTE))}")"
    if [[ "$records" != ok || "$(alert_check "$context_alerts" memory-squeezed selftest-booked-steady '{"count": 0}')" != ok ]]; then
      fail "盯账停的那一条应当往告警记录写一条 memory-squeezed（stopped_by=over-booking-watch、记账 40 MiB 取自峰值表、实测不小于记账），常驻那一条不写，实际：$records"
    fi

    # 多进程超账：另一条是同一条 scope 里 8 个进程、各每 0.15 秒涨 6 MiB 涨到 24 MiB（单个进程都不大，合起来超账）；内核按进程打分挑不对，盯账按 scope 算挑得对
    over_booking_context
    checked=$((checked + 1))
    over_booking_pair multiple selftest-booked-grower python3 -c "$grower_code" "$scratch/over-booking-multiple-other.end" 8 24 6 0.15 1
    over_booking_stopped_right multiple \
      || fail "多进程超账（记账 40 MiB 的那一条在同一条 scope 里起 8 个进程各涨到 24 MiB）时盯账应当只停它（退 $SLICE_TOTAL_HIT_EXIT、报「超出记账最多，包装在撞总上限之前停了它」、键记成撞过顶），\
没超账的常驻那一条退 0、键不记撞过顶，内核一次都不 OOM，实际：$over_booking_problem$(cat "$other_err" "$steady_err")"

    # 一口气拿满：另一条起来一口气拿 200 MiB；越过 slice 的 memory.high（380 MiB）之后分配被节流，盯账赶在撞 memory.max 之前停它
    over_booking_context
    checked=$((checked + 1))
    over_booking_pair at-once selftest-booked-grower python3 -c "$grower_code" "$scratch/over-booking-at-once-other.end" 1 200 200 0 1
    high_now="$(systemctl --user show -p ControlGroup --value "$slice" 2>/dev/null)"
    high_now="$(cat "/sys/fs/cgroup$high_now/memory.high" 2>/dev/null)"
    if ! over_booking_stopped_right at-once || [[ "$high_now" != "$((380 * MEBIBYTE))" ]]; then
      fail "一口气拿满（记账 40 MiB 的那一条起来就拿 200 MiB）时 slice 的 memory.high 应当是总上限的 95%（$((380 * MEBIBYTE))）、盯账应当只停它（退 $SLICE_TOTAL_HIT_EXIT、报「超出记账最多，包装在撞总上限之前停了它」、键记成撞过顶），\
没超账的常驻那一条退 0、键不记撞过顶，内核一次都不 OOM，实际：$over_booking_problem slice 的 memory.high「$high_now」：$(cat "$other_err" "$steady_err")"
    fi

    # 两条都不超账、合计压过触发线：另一条记账 140 MiB、拿 140 MiB（约 145 MiB），两条合计约 390 MiB，过了 380 MiB 的触发线、在 400M 以内；
    # 盯账判到了（两条的判法记录里有行）、谁都不停，两条都退 0，内核不 OOM
    over_booking_context
    checked=$((checked + 1))
    trace="$scratch/over-booking-within.trace"
    steady_env=(RUN_WITH_MEMORY_CAP_SELFTEST_WATCH_TRACE="$trace") other_env=(RUN_WITH_MEMORY_CAP_SELFTEST_WATCH_TRACE="$trace")
    over_booking_pair within selftest-booked-within python3 -c "$job_code" "$scratch/over-booking-within-other.start" "$scratch/over-booking-within-other.end" 0 140 3
    steady_env=() other_env=()
    kills="$(slice_event oom_kill)"
    if [[ $steady_status -ne 0 || $other_status -ne 0 || "$kills" != 0 || ! -s "$trace" ]] || grep -q '超出记账最多' "$steady_err" "$other_err"; then
      fail "两条都不超账（记账 240 MiB 与 140 MiB、各超出约 5 MiB，合计约 390 MiB 过了触发线 380 MiB）时盯账应当判到（判法记录有行）、谁都不停、两条都退 0、内核不 OOM，\
实际常驻退 $steady_status、另一条退 $other_status、slice 的 oom_kill「$kills」、判法记录 $(wc -l < "$trace" 2>/dev/null || echo 0) 行：$(tail -n 2 "$trace" 2>/dev/null)；$(cat "$other_err" "$steady_err")"
    fi

    # 内核兜底：两条都设 RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY（不盯账、slice 不设 memory.high），单进程超账撞了 memory.max，
    # 内核按记账算的 oom_score_adj 杀超账那一条（退 254、记成撞过顶），不杀没超账、常驻更大的那一条（改前两条同为 500 时杀的是它，2026-10-26 实测 6 次里 6 次）
    over_booking_context
    checked=$((checked + 1))
    steady_env=(RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1) other_env=(RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1)
    over_booking_pair kernel selftest-booked-grower python3 -c "$grower_code" "$scratch/over-booking-kernel-other.end" 1 200 10 0.15 1
    steady_env=() other_env=()
    kills="$(slice_event oom_kill)"
    steady_recent="$(recent_column selftest-booked-steady)"
    other_row="$(awk -F '\t' '$4 == "selftest-booked-grower" {print $1 "|" $6}' "$context_directory/peaks.tsv" 2>/dev/null)"
    if [[ $steady_status -ne 0 || $other_status -ne $SLICE_TOTAL_HIT_EXIT || "$kills" == 0 || -z "$kills" || "$steady_recent" == *:hit* ]] \
        || [[ "${other_row%%|*}" != "$((300 * MEBIBYTE))" || "${other_row#*|}" != *:hit* ]] || ! grep -q '被 slice .* 的总上限挤掉了' "$other_err"; then
      fail "不盯账、单进程超账撞了总上限时内核应当按记账算的 oom_score_adj 杀超账那一条（退 $SLICE_TOTAL_HIT_EXIT、报「被 slice … 的总上限挤掉了」、键记成撞过顶），没超账、常驻更大的那一条退 0、键不记撞过顶，\
实际常驻退 $steady_status、超账那一条退 $other_status、slice 的 oom_kill「$kills」、常驻那一条近几次「$steady_recent」、超账那一条峰值|近几次「$other_row」：$(cat "$other_err" "$steady_err")"
    fi

    # 被内核挤掉而没超账的那一条不记撞过顶：两条都设 KERNEL_ONLY，常驻那一条另设 RUN_WITH_MEMORY_CAP_OOM_FIRST（oom_score_adj 1000，内核先挑它），
    # 它没超出记账（约 5 MiB，在容差以内）：退 254、stderr 写明没超出记账、峰值表里它那一行照旧（不记撞过顶）；超账那一条随之跑完退 0
    over_booking_context
    checked=$((checked + 1))
    steady_env=(RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1 RUN_WITH_MEMORY_CAP_OOM_FIRST='自证：让内核先挑没超账的那一条') other_env=(RUN_WITH_MEMORY_CAP_SELFTEST_KERNEL_ONLY=1)
    over_booking_pair innocent selftest-booked-grower python3 -c "$grower_code" "$scratch/over-booking-innocent-other.end" 1 200 10 0.15 1
    steady_env=() other_env=()
    steady_recent="$(recent_column selftest-booked-steady)"
    if [[ $steady_status -ne $SLICE_TOTAL_HIT_EXIT || $other_status -ne 0 || "$steady_recent" == *:hit* ]] || ! grep -q '没超出记账' "$steady_err"; then
      fail "被内核挤掉而没超出记账的那一条应当退 $SLICE_TOTAL_HIT_EXIT、stderr 写明「没超出记账」、峰值表里这个键不记撞过顶，超账那一条跑完退 0，\
实际它退 $steady_status、近几次「$steady_recent」（列数|第六列）、超账那一条退 $other_status：$(cat "$steady_err" "$other_err")"
    fi
  }
  # 只跑这几项（RUN_WITH_MEMORY_CAP_SELFTEST_ONLY=slice-total，看它红在哪用）：跑完收尾、退出，不算整份自证通过
  if [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_ONLY:-}" == slice-total ]]; then
    slice_total_cells
    over_booking_cells
    for slice in "${selftest_slices[@]}"; do
      systemctl --user stop "$slice" >/dev/null 2>&1
      systemctl --user revert "$slice" >/dev/null 2>&1
    done
    rm -rf -- "${scratch:?}"
    if ((failures)); then
      echo "  ✗ run-with-memory-cap.sh 自检（只跑被总上限挤掉、内核挑谁与盯账那几项）$failures 处不对（查了 $checked 项；测的是 $target）"
      echo "  → 怎么办：照上面逐条改 run_capped 里按记账算 oom_score_adj、认盯账标记与记不记撞过顶的几处，watch_over_booking、ensure_slice_high；RUN_WITH_MEMORY_CAP_BREAK 设着的话本来就该红"
      exit 1
    fi
    echo "  ✓ run-with-memory-cap.sh 自检（只跑被总上限挤掉、内核挑谁与盯账那几项，不算整份自证通过）查了 $checked 项都对"
    exit 0
  fi

  # 弄坏开关只在自证里认（文件头那一段；门禁 code-tooling 的 break-switch-outside-selftest 格就这样探）：
  # 没有标记（变量清掉、或指到头一行不对的文件）而设了开关 ⇒ 退 2、点名它只在自证里认；弄坏开关 guard-off 下这一道不拒，探针往下走到「没给命令」退 251、不点名。
  # 带着标记 ⇒ 不拒，往下走到同一处退 251
  local guard_probe_break=nocap guard_probe_mark guard_probe_output guard_probe_status
  [[ "$BROKEN_JUDGEMENT" != guard-off ]] || guard_probe_break=guard-off
  printf '%s\n' "不是 $SELFTEST_MARK_CONTENT" > "$scratch/selftest-mark-wrong"
  for guard_probe_mark in "" "$scratch/selftest-mark-wrong"; do
    checked=$((checked + 1))
    if guard_probe_output="$(env RUN_WITH_MEMORY_CAP_SELFTEST_MARK="$guard_probe_mark" RUN_WITH_MEMORY_CAP_BREAK="$guard_probe_break" \
        bash "$target" --force --break-guard-probe-unknown-option 2>&1 </dev/null)"; then guard_probe_status=0; else guard_probe_status=$?; fi
    if [[ "$guard_probe_status" != 2 ]] || ! grep -qE 'RUN_WITH_MEMORY_CAP_BREAK.*只在自证（--selftest）里认' <<<"$guard_probe_output"; then
      fail "平常跑（没有自证的标记）设了 RUN_WITH_MEMORY_CAP_BREAK 应当退 2、点名它只在自证里认（标记「${guard_probe_mark:-空}」），实际退 $guard_probe_status：$(head -n 3 <<<"$guard_probe_output" | tr '\n' ' ')"
    fi
  done
  checked=$((checked + 1))
  if guard_probe_output="$(env RUN_WITH_MEMORY_CAP_BREAK=nocap bash "$target" --force --break-guard-probe-unknown-option 2>&1 </dev/null)"; then guard_probe_status=0; else guard_probe_status=$?; fi
  if [[ "$guard_probe_status" != "$MEMORY_CAP_UNAVAILABLE_EXIT" ]] || grep -qF '只在自证（--selftest）里认' <<<"$guard_probe_output"; then
    fail "带着自证的标记设了 RUN_WITH_MEMORY_CAP_BREAK 应当不拒、照常往下判（这一条没给命令，退 $MEMORY_CAP_UNAVAILABLE_EXIT），实际退 $guard_probe_status：$(head -n 3 <<<"$guard_probe_output" | tr '\n' ' ')"
  fi

  fresh_context 512M
  checked=$((checked + 1))
  if ! in_context bash "$target" --check 64M 2>"$scratch/check.err"; then
    fail "--check 64M 在这台机器上应当起得来，实际起不来：$(cat "$scratch/check.err")"
  fi

  checked=$((checked + 1))
  if in_context bash "$target" 64M python3 -c "$hog_code" 512 >"$scratch/hog.out" 2>"$scratch/hog.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '撞了内存上限 64M' "$scratch/hog.err"; then
    fail "上限 64M 里分配 512 MiB 的探针应当退 $MEMORY_CAP_HIT_EXIT 并报「撞了内存上限 64M」，实际退 $status：$(cat "$scratch/hog.out" "$scratch/hog.err")"
  fi

  # 吃内存的是孙进程、父进程被杀之后自己还要退 101：照样按 Result 判撞顶
  checked=$((checked + 1))
  if in_context bash "$target" 64M bash -c 'python3 -c "$1" 512; exit 101' hog-parent "$hog_code" >"$scratch/grandchild.out" 2>"$scratch/grandchild.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]]; then
    fail "吃内存的是孙进程时应当照样退 $MEMORY_CAP_HIT_EXIT（按 scope 的 Result 判，不按父进程的退出码），实际退 $status：$(cat "$scratch/grandchild.err")"
  fi

  # 不是撞顶的 SIGKILL（自己 kill -9 自己）退 137，不许判成撞顶
  checked=$((checked + 1))
  if in_context bash "$target" 64M bash -c 'kill -9 $$' 2>"$scratch/sigkill.err"; then status=0; else status=$?; fi
  if [[ $status -ne 137 || "$(cat "$scratch/sigkill.err")" == *Killed* ]]; then
    fail "上限之内被 kill -9 的命令应当原样退 137、不算撞顶，stderr 里不多出外壳补的「Killed」那一行，实际退 $status：$(cat "$scratch/sigkill.err")"
  fi

  # 命令被外面的信号停（退 128 加 1、2、9、15）而 scope 没撞自己的上限、也没被 slice 挤掉：退出码原样，报「被外面停了（信号 N），这一次的峰值不记」与出路，
  # 峰值表里不留这个键（量到的是半截）；对照：同样的上限里整段跑完的一条照记。弄坏开关 recordkilledpeak 下 TERM、KILL 两格红
  fresh_context 256M
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-stopped-control bash "$target" 64M bash -c : 2>"$scratch/stopped-control.err"; then status=0; else status=$?; fi
  output="$(awk -F '\t' '$4 == "selftest-stopped-control" {print $1}' "$context_directory/peaks.tsv" 2>/dev/null)"
  if [[ $status -ne 0 || ! "$output" =~ ^[0-9]+$ ]]; then
    fail "对照：整段跑完的命令应当把峰值记进表（selftest-stopped-control 那一行），实际退 $status、记的是「$output」：$(cat "$scratch/stopped-control.err")"
  fi
  for stopped_name in TERM KILL; do
    stopped_number="$(kill -l "$stopped_name")"
    checked=$((checked + 1))
    if in_context env RUN_WITH_MEMORY_CAP_KEY="selftest-stopped-by-$stopped_name" bash "$target" 64M bash -c "kill -$stopped_name \$\$" 2>"$scratch/stopped-$stopped_name.err"; then status=0; else status=$?; fi
    output="$(awk -F '\t' -v key="selftest-stopped-by-$stopped_name" '$4 == key {print $1}' "$context_directory/peaks.tsv" 2>/dev/null)"
    if [[ $status -ne $((128 + stopped_number)) || -n "$output" ]] || ! grep -qF "被外面停了（信号 $stopped_number），这一次的峰值不记" "$scratch/stopped-$stopped_name.err" \
        || ! grep -qF "→ 这一条的结果不算数；查是谁发的信号" "$scratch/stopped-$stopped_name.err"; then
      fail "命令被外面的 $stopped_name 停应当原样退 $((128 + stopped_number))、报「被外面停了（信号 $stopped_number），这一次的峰值不记」与出路、峰值表里不留这个键，实际退 $status、表里记的是「$output」：$(cat "$scratch/stopped-$stopped_name.err")"
    fi
  done

  # 正常退出码原样传回
  for wanted in 0 7 101; do
    checked=$((checked + 1))
    if in_context bash "$target" 64M bash -c "exit $wanted"; then status=0; else status=$?; fi
    [[ $status -eq $wanted ]] || fail "命令退 $wanted 时应当原样传回 $wanted，实际 $status"
  done

  # 上限对实测峰值（文件头「上限对实测峰值」一段，2026-10-06 撤了那道拒绝）：表里这条实测 10M，给 64M（超过 2 倍）照常跑、退 0、命令跑了；
  # auto 取 1.5 倍（15M，scope 的 memory.max 读得到 15728640）；表里没有这条时 auto 取保守默认值 8G（memory.max 8589934592）
  marker="$scratch/ran-over-peak"
  seed_peak "bash -c : > \"\$1\" touch-marker $marker" 10485760
  checked=$((checked + 1))
  if in_context bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/overpeak.err"; then status=0; else status=$?; fi
  if [[ $status -ne 0 || ! -e "$marker" ]]; then
    fail "上限 64M 超过峰值表里实测峰值 10M 的 2 倍时应当照常跑、不拒，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/overpeak.err")"
  fi
  seed_peak 'bash -c group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"; cat "$group/memory.max"' 10485760
  checked=$((checked + 1))
  output="$(in_context bash "$target" auto bash -c 'group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"; cat "$group/memory.max"' 2>&1)"
  [[ "$output" == "15728640" ]] || fail "auto 应当取实测峰值 10M 的 1.5 倍（15M，memory.max 15728640），实际「$output」"
  fresh_context 9G   # 默认值 8G 要放得进总上限（slice 的 MemoryMax 只是个上限，不真占内存）
  checked=$((checked + 1))
  output="$(in_context bash "$target" auto bash -c 'group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"; cat "$group/memory.max"' auto-without-a-recorded-peak 2>"$scratch/auto-unknown.err")"
  if [[ "$output" != "8589934592" ]] || ! grep -q '取保守默认值 8G' "$scratch/auto-unknown.err"; then
    fail "auto 而峰值表里没有这条时应当取保守默认值 8G 跑（memory.max 8589934592、stderr 报「取保守默认值 8G」），实际「$output」：$(cat "$scratch/auto-unknown.err")"
  fi

  # 日常一条（文件头「日常一条」一段，2026-10-06 撤了那两道拒绝）：同一个会话已有一条在跑，第二条照常放行、跟它同时跑；
  # 从会话里起的一条此刻放不下（两条都按上限 200M 算、总上限 256M），排队、等前一条跑完再起、退 0，不当场退 252（等待上限是默认的，不明写）
  fresh_context 256M
  marker="$scratch/second-session-ran"
  in_context env CLAUDE_CODE_SESSION_ID=selftest-session-one RUN_WITH_MEMORY_CAP_KEY=selftest-session-first bash "$target" 64M bash -c 'sleep 4' 2>"$scratch/session-first.err" &
  first_pid=$!
  sleep 1.5
  checked=$((checked + 1))
  if in_context env CLAUDE_CODE_SESSION_ID=selftest-session-one RUN_WITH_MEMORY_CAP_KEY=selftest-session-second bash "$target" 16M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/session-second.err"; then status=0; else status=$?; fi
  if [[ $status -ne 0 || ! -e "$marker" ]] || ! kill -0 "$first_pid" 2>/dev/null; then
    fail "同一个会话已有一条在跑时第二条应当照常放行、跟它同时跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')、前一条$(kill -0 "$first_pid" 2>/dev/null && echo '还在跑' || echo '已经跑完')：$(cat "$scratch/session-second.err")"
  fi
  wait "$first_pid" || true   # shell-lint:exit-collected 第一条只是占着位置，它自己的退出码不判
  fresh_context 256M
  checked=$((checked + 1))
  in_context env CLAUDE_CODE_SESSION_ID=selftest-session-one RUN_WITH_MEMORY_CAP_KEY=selftest-daily-holder bash "$target" 200M python3 -c "$job_code" "$scratch/daily-1.start" "$scratch/daily-1.end" 0 20 2.5 \
    >/dev/null 2>"$scratch/daily-1.err" &
  first_pid=$!
  sleep 1
  if in_context env CLAUDE_CODE_SESSION_ID=selftest-session-two RUN_WITH_MEMORY_CAP_KEY=selftest-daily-queued bash "$target" 200M python3 -c "$job_code" "$scratch/daily-2.start" "$scratch/daily-2.end" 0 1 0 \
      >/dev/null 2>"$scratch/daily-queued.err"; then status=0; else status=$?; fi
  wait "$first_pid"; first_status=$?
  if [[ $first_status -ne 0 || $status -ne 0 ]] || ! all_exist "$scratch/daily-1.end" "$scratch/daily-2.start" \
      || ! python3 -c 'import sys; sys.exit(0 if float(open(sys.argv[2]).read()) >= float(open(sys.argv[1]).read()) else 1)' "$scratch/daily-1.end" "$scratch/daily-2.start" 2>/dev/null; then
    fail "从会话里起的一条此刻放不下时应当排队、等前一条跑完再起，不当场退 $MEMORY_ADMISSION_REFUSED_EXIT，实际前一条退 $first_status、它退 $status：$(cat "$scratch/daily-1.err" "$scratch/daily-queued.err")"
  fi
  fresh_context 512M

  # 显存不够（文件头「显存不够」一段）：假 journalctl 按 journal-mode 文件演：oom 打一行 NVRM … NV_ERR_NO_MEMORY，xid 打一行别的 NVRM，
  # failing 退 1（读不到内核日志）；-n 1（看不看得到内核日志）一律打一行。每次调用的参数记进 journal-args
  local fake_journal="$scratch/fake-journal/journalctl" journal_args since_epoch until_epoch
  mkdir -p "$scratch/fake-journal"
  cat > "$fake_journal" <<'FAKE_JOURNAL'
#!/usr/bin/env bash
here="$(dirname "$0")"
printf '%s\n' "$*" >> "$here/journal-args"
mode="$(cat "$here/journal-mode")"
if [[ "$mode" == failing ]]; then echo "Failed to open journal" >&2; exit 1; fi
case " $* " in *" -n 1 "*) echo "2026-10-06 host kernel: probe"; exit 0 ;; esac
case "$mode" in
  oom) echo "2026-10-06 host kernel: NVRM: GPU1 nvAssertOkFailedNoLog: Assertion failed: Out of memory [NV_ERR_NO_MEMORY] (0x00000051) returned from pmaAllocatePages() @ pool_alloc.c:270" ;;
  xid) echo "2026-10-06 host kernel: NVRM: Xid (PCI:0000:10:00): 109, pid=441575, name=selftest, channel 0x00000003, errorString CTX SWITCH TIMEOUT, Info 0xc002" ;;
esac
exit 0
FAKE_JOURNAL
  chmod +x "$fake_journal"
  journal_case() { # journal_case <模式> <名字> <命令…>：status 与 $scratch/<名字>.err
    printf '%s\n' "$1" > "$scratch/fake-journal/journal-mode"
    : > "$scratch/fake-journal/journal-args"
    local case_name="$2"
    shift 2
    if in_context env RUN_WITH_MEMORY_CAP_JOURNALCTL="$fake_journal" bash "$target" 64M "$@" 2>"$scratch/$case_name.err"; then status=0; else status=$?; fi
  }
  checked=$((checked + 1))
  journal_case oom gpu-oom bash -c 'exit 3'
  if [[ $status -ne $GPU_MEMORY_HIT_EXIT ]] || ! grep -q '显存不够' "$scratch/gpu-oom.err"; then
    fail "命令退 3、起止之间内核报了 NVRM … NV_ERR_NO_MEMORY 时应当退 $GPU_MEMORY_HIT_EXIT、报「显存不够」，实际退 $status：$(cat "$scratch/gpu-oom.err")"
  fi
  checked=$((checked + 1))
  journal_args="$(grep -v -- ' -n 1 ' "$scratch/fake-journal/journal-args" | head -n 1)"
  since_epoch="$(sed -n 's/.*--since @\([0-9][0-9]*\).*/\1/p' <<<"$journal_args")"
  until_epoch="$(sed -n 's/.*--until @\([0-9][0-9]*\).*/\1/p' <<<"$journal_args")"
  if [[ -z "$since_epoch" || -z "$until_epoch" ]] || ((since_epoch > until_epoch || until_epoch - since_epoch > 60)); then
    fail "读内核日志应当按命令起止给 --since @<起> 与 --until @<止>（两者相差不过 60 秒），实际参数「$journal_args」"
  fi
  checked=$((checked + 1))
  journal_case failing gpu-unreadable bash -c 'exit 3'
  if [[ $status -ne 3 ]] || ! grep -q '显存 OOM 没查成' "$scratch/gpu-unreadable.err"; then
    fail "内核日志读不出来时应当照退命令自己的 3、报「显存 OOM 没查成」，实际退 $status：$(cat "$scratch/gpu-unreadable.err")"
  fi
  checked=$((checked + 1))
  journal_case xid gpu-other-nvrm bash -c 'exit 3'
  if [[ $status -ne 3 ]] || grep -q '显存不够' "$scratch/gpu-other-nvrm.err"; then
    fail "起止之间只有别的 NVRM 行（Xid）时应当照退命令自己的 3、不报显存不够，实际退 $status：$(cat "$scratch/gpu-other-nvrm.err")"
  fi
  checked=$((checked + 1))
  journal_case oom gpu-success bash -c 'exit 0'
  if [[ $status -ne 0 || -s "$scratch/fake-journal/journal-args" ]]; then
    fail "命令退 0 时应当照退 0、不读内核日志，实际退 $status、journalctl 被调了 $(wc -l < "$scratch/fake-journal/journal-args") 次：$(cat "$scratch/gpu-success.err")"
  fi
  checked=$((checked + 1))
  journal_case oom gpu-with-cap-hit python3 -c "$hog_code" 512
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '显存不够' "$scratch/gpu-with-cap-hit.err"; then
    fail "撞了自己的上限、起止之间内核又报了显存不够时应当照旧退 $MEMORY_CAP_HIT_EXIT、另报一行「显存不够」，实际退 $status：$(cat "$scratch/gpu-with-cap-hit.err")"
  fi

  # 上限真的设上了：scope 里读自己 cgroup 的 memory.max 与 memory.swap.max
  checked=$((checked + 1))
  output="$(in_context bash "$target" 64M bash -c 'group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"; cat "$group/memory.max" "$group/memory.swap.max"' 2>&1)"
  if [[ "$output" != $'67108864\n0' ]]; then
    fail "scope 里的 memory.max、memory.swap.max 应当是 67108864 与 0，实际「$output」"
  fi

  # 总上限（第 ② 层）真的设上了：scope 挂在这一例的 slice 底下，slice 的 memory.max 是 512M
  checked=$((checked + 1))
  output="$(in_context bash "$target" 64M bash -c 'group="$(cut -d: -f3 /proc/self/cgroup)"; echo "$group"; cat "/sys/fs/cgroup$(dirname "$group")/memory.max"' 2>&1)"
  if [[ "$output" != *"/$slice/"*$'\n536870912' ]]; then
    fail "scope 应当挂在 slice $slice 底下、slice 的 memory.max 是 536870912（512M），实际「$output」"
  fi

  # slice 设不上（cgroup 里读回来的不是那个数，这里用一个不存在的 cgroup 根演）：退 251、报 slice、命令没跑，不退回无总上限
  checked=$((checked + 1))
  marker="$scratch/ran-without-slice"
  if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$scratch/no-cgroup-here" bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/noslice.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q 'slice' "$scratch/noslice.err"; then
    fail "slice 的总上限设不上时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报 slice、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/noslice.err")"
  fi
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$scratch/no-cgroup-here" bash "$target" --check 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "slice 的总上限设不上时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  # 余量比整机内存还大：总上限算不出来，同样退 251、命令没跑
  checked=$((checked + 1))
  marker="$scratch/ran-with-oversized-reserve"
  if in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= RUN_WITH_MEMORY_CAP_RESERVE=1024T bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/reserve.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]]; then
    fail "余量 1024T 比整机内存还大时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/reserve.err")"
  fi

  # systemd-run 起不来：退 251、报错，而且那条命令一行都没跑（不退回无上限）
  checked=$((checked + 1))
  marker="$scratch/ran-without-cap"
  if in_context env DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent \
      bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/unavailable.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q '起不来' "$scratch/unavailable.err"; then
    fail "D-Bus 连不上时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「起不来」、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unavailable.err")"
  fi
  checked=$((checked + 1))
  if in_context env DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent \
      bash "$target" --check 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "D-Bus 连不上时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  # slice 设得上、而 systemd-run 自己起不来（systemd 不认这个上限：换成字节超过 64 位）：--check 退 251，不退回无上限去跑
  checked=$((checked + 1))
  if in_context bash "$target" --check 99999999999999T 2>"$scratch/scope-refused.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT ]] || ! grep -q 'systemd-run' "$scratch/scope-refused.err"; then
    fail "slice 设得上而 systemd-run 不认上限 99999999999999T 时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报 systemd-run，实际退 $status：$(cat "$scratch/scope-refused.err")"
  fi

  # 两条同时来、都判放得下（第 ③ 层的锁）：总上限 256M，两条各要 200M（峰值表里没有、按上限算），判完停 0.6 秒再记账把竞态撑开；
  # 应当一条跑完另一条才起，两条都退 0。判与记账不在一把锁里时两条都放进来、合起来 300 MiB 超总上限
  fresh_context 256M
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE=0.6 bash "$target" 200M python3 -c "$job_code" "$scratch/lock-1.start" "$scratch/lock-1.end" 0.2 150 1.0 \
    >/dev/null 2>"$scratch/lock-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE=0.6 bash "$target" 200M python3 -c "$job_code" "$scratch/lock-2.start" "$scratch/lock-2.end" 0.2 150 1.0 \
    >/dev/null 2>"$scratch/lock-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! all_exist "$scratch"/lock-{1,2}.{start,end} \
      || intervals_overlap "$scratch/lock-1.start" "$scratch/lock-1.end" "$scratch/lock-2.start" "$scratch/lock-2.end"; then
    fail "两条同时来（各要 200M、总上限 256M）应当一条一条跑、都退 0，实际退 $first_status 与 $second_status、$(intervals_overlap "$scratch/lock-1.start" "$scratch/lock-1.end" "$scratch/lock-2.start" "$scratch/lock-2.end" && echo '两条同时在跑' || echo '没有同时在跑')：$(cat "$scratch/lock-1.err" "$scratch/lock-2.err")"
  fi

  # 余量不同的调用者并发（文件头「余量」一段「并发调用者」，checks-owed C787）。总上限 = 整机内存 − 余量，两个调用者的余量不同就算出两个总上限；
  # 这几例用真的 RUN_WITH_MEMORY_CAP_RESERVE（不设 SLICE_TOTAL），别的例子才用 SLICE_TOTAL 压到几百 M。
  # ① 先到的一条设完总上限、停在读回之前（SELFTEST_SLICE_PAUSE 3 秒，停之前先写标记），后到的余量不同的那一条这时把总上限设成自己的：
  #    先到的读回是别人的值，应当照读回的走并报一行，不退 251；两条都退 0
  fresh_context 256M
  checked=$((checked + 1))
  rm -f -- "$scratch/reserve-pause.mark"
  in_context env -u RUN_WITH_MEMORY_CAP_SLICE_TOTAL RUN_WITH_MEMORY_CAP_RESERVE=10G RUN_WITH_MEMORY_CAP_SELFTEST_SLICE_PAUSE=3 \
    RUN_WITH_MEMORY_CAP_SELFTEST_SLICE_PAUSE_FILE="$scratch/reserve-pause.mark" bash "$target" 64M bash -c 'sleep 1' >/dev/null 2>"$scratch/reserve-race-first.err" &
  first_pid=$!
  for _ in $(seq 1 50); do [[ -e "$scratch/reserve-pause.mark" ]] && break; sleep 0.1; done
  if in_context env -u RUN_WITH_MEMORY_CAP_SLICE_TOTAL bash "$target" 64M bash -c ':' >/dev/null 2>"$scratch/reserve-race-second.err"; then second_status=0; else second_status=$?; fi
  wait "$first_pid"; first_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! grep -q '照 slice .* 已有的总上限' "$scratch/reserve-race-first.err"; then
    fail "余量不同的两个调用者并发、先到的在设完总上限读回之前被后到的改了值时，两条都应当退 0、先到的照读回的值走并报「照 slice … 已有的总上限」，实际退 $first_status 与 $second_status：$(cat "$scratch/reserve-race-first.err" "$scratch/reserve-race-second.err")"
  fi

  # ② slice 上有余量 10G 的一条活着（在账上、scope 在跑），余量默认的一条后到：应当照 slice 已有的总上限走、不改 slice 的 MemoryMax、报一行说明（写明已有的值与来自哪个 scope），
  #    退 0；它自己的 scope 读到的 slice MemoryMax 还是先到那一条设的
  fresh_context 256M
  checked=$((checked + 1))
  in_context env -u RUN_WITH_MEMORY_CAP_SLICE_TOTAL RUN_WITH_MEMORY_CAP_RESERVE=10G bash "$target" 64M bash -c 'sleep 4' >/dev/null 2>"$scratch/reserve-live-first.err" &
  first_pid=$!
  for _ in $(seq 1 50); do compgen -G "$context_directory/state/ledger/*.json" >/dev/null && break; sleep 0.1; done
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$slice" 2>/dev/null)"
  first_total="$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)"
  output="$(in_context env -u RUN_WITH_MEMORY_CAP_SLICE_TOTAL bash "$target" 64M bash -c 'cat "$(dirname "/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)")/memory.max"' 2>"$scratch/reserve-live-second.err")"
  second_status=$?
  second_total_after="$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)"
  wait "$first_pid"
  if [[ ! "$first_total" =~ ^[1-9][0-9]*$ || $second_status -ne 0 || "$output" != "$first_total" || "$second_total_after" != "$first_total" ]] \
      || ! grep -q "照 slice .* 已有的总上限 $first_total 字节走" "$scratch/reserve-live-second.err" || ! grep -q 'singlefs-memory-cap-' "$scratch/reserve-live-second.err"; then
    fail "slice 上有余量 10G 的一条活着时，余量默认的后到一条应当照已有的总上限 ${first_total:-读不到} 走、不改 slice 的 MemoryMax、报「照 slice … 已有的总上限 … 字节走」并写明来自哪个 scope、退 0，实际退 $second_status、它读到的 slice MemoryMax「$output」、之后 slice 的是「$second_total_after」：$(cat "$scratch/reserve-live-second.err")"
  fi

  # ③ 排队也按已有的总上限算：先到的总上限 256M、在跑的一条占 200M；后到的自己算出来的总上限是 512M（放得下两条），照已有的 256M 走就得等前一条跑完，两条不重叠、都退 0
  fresh_context 256M
  checked=$((checked + 1))
  in_context bash "$target" 200M python3 -c "$job_code" "$scratch/foreign-total-1.start" "$scratch/foreign-total-1.end" 0.2 150 2.0 >/dev/null 2>"$scratch/foreign-total-1.err" &
  first_pid=$!
  for _ in $(seq 1 50); do compgen -G "$context_directory/state/ledger/*.json" >/dev/null && break; sleep 0.1; done
  in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL=512M bash "$target" 200M python3 -c "$job_code" "$scratch/foreign-total-2.start" "$scratch/foreign-total-2.end" 0 150 0.5 \
    >/dev/null 2>"$scratch/foreign-total-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! all_exist "$scratch"/foreign-total-{1,2}.{start,end} \
      || intervals_overlap "$scratch/foreign-total-1.start" "$scratch/foreign-total-1.end" "$scratch/foreign-total-2.start" "$scratch/foreign-total-2.end" \
      || ! grep -q '照 slice .* 已有的总上限 268435456 字节走' "$scratch/foreign-total-2.err"; then
    fail "后到一条自己算的总上限 512M 比 slice 上在跑那一条设的 256M 大时，排队应当照已有的 256M 算、等前一条跑完、两条都退 0，实际退 $first_status 与 $second_status、$(intervals_overlap "$scratch/foreign-total-1.start" "$scratch/foreign-total-1.end" "$scratch/foreign-total-2.start" "$scratch/foreign-total-2.end" && echo '两条同时在跑' || echo '没有同时在跑')：$(cat "$scratch/foreign-total-2.err")"
  fi

  # ④ --check 只试上限设不设得上：slice 上有余量 10G 的一条活着，默认余量的 --check 64M 照已有的总上限试、不改 slice 的 MemoryMax、stderr 里不打「照 slice」那一行、退 0
  #    （mutate.sh 开跑先调一次 --check，它自证钉「stderr 头一行是这一轮开 N 个工作进程」）。占着的那一条自己睡到头退出，不发信号
  fresh_context 256M
  checked=$((checked + 1))
  in_context env -u RUN_WITH_MEMORY_CAP_SLICE_TOTAL RUN_WITH_MEMORY_CAP_RESERVE=10G bash "$target" 64M bash -c 'sleep 4' >/dev/null 2>"$scratch/check-foreign-first.err" &
  first_pid=$!
  for _ in $(seq 1 50); do compgen -G "$context_directory/state/ledger/*.json" >/dev/null && break; sleep 0.1; done
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$slice" 2>/dev/null)"
  first_total="$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)"
  if in_context env -u RUN_WITH_MEMORY_CAP_SLICE_TOTAL bash "$target" --check 64M >/dev/null 2>"$scratch/check-foreign-second.err"; then second_status=0; else second_status=$?; fi
  second_total_after="$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)"
  wait "$first_pid"; first_status=$?
  if [[ ! "$first_total" =~ ^[1-9][0-9]*$ || $first_status -ne 0 || $second_status -ne 0 || "$second_total_after" != "$first_total" ]] \
      || grep -q '照 slice' "$scratch/check-foreign-second.err"; then
    fail "slice 上有余量 10G 的一条活着时，默认余量的 --check 64M 应当照已有的总上限 ${first_total:-读不到} 试、不改 slice 的 MemoryMax、stderr 里不打「照 slice」那一行、退 0，实际退 $second_status（占着的那一条退 $first_status）、之后 slice 的是「$second_total_after」：$(cat "$scratch/check-foreign-second.err")"
  fi

  # 前一条放行了、还没涨到量（先睡 1 秒再拿 150 MiB）时后一条来：只看 slice 的用量两条都判得下、合起来超；
  # 账上还没涨到量的那部分算进已占，后一条就等前一条跑完
  fresh_context 256M
  checked=$((checked + 1))
  in_context bash "$target" 200M python3 -c "$job_code" "$scratch/ramp-1.start" "$scratch/ramp-1.end" 1.0 150 0.8 >/dev/null 2>"$scratch/ramp-1.err" &
  first_pid=$!
  sleep 0.4
  in_context bash "$target" 200M python3 -c "$job_code" "$scratch/ramp-2.start" "$scratch/ramp-2.end" 0 150 0.8 >/dev/null 2>"$scratch/ramp-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! all_exist "$scratch"/ramp-{1,2}.{start,end} \
      || intervals_overlap "$scratch/ramp-1.start" "$scratch/ramp-1.end" "$scratch/ramp-2.start" "$scratch/ramp-2.end"; then
    fail "前一条还没涨到量时来的后一条应当等它跑完、两条都退 0，实际退 $first_status 与 $second_status、$(intervals_overlap "$scratch/ramp-1.start" "$scratch/ramp-1.end" "$scratch/ramp-2.start" "$scratch/ramp-2.end" && echo '两条同时在跑' || echo '没有同时在跑')：$(cat "$scratch/ramp-1.err" "$scratch/ramp-2.err")"
  fi

  # 峰值表里没有这条：按 host-resources.py 的预测（8G）与它的上限取小，即上限。前一条（表里记 100 MiB）在跑，后一条表里没有、上限 200M：100 + 200 放不进 256M，
  # 等 1 秒排不上，退 252、命令没跑、说清是按上限算的
  fresh_context 256M
  checked=$((checked + 1))
  seed_peak selftest-recorded $((100 * MEBIBYTE))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-recorded bash "$target" 200M python3 -c "$job_code" "$scratch/table-1.start" "$scratch/table-1.end" 0 10 2.5 \
    >/dev/null 2>"$scratch/table-1.err" &
  first_pid=$!
  sleep 0.6
  marker="$scratch/ran-unrecorded"
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-unrecorded RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1 bash "$target" 200M bash -c ': > "$1"' touch-marker "$marker" \
      2>"$scratch/unrecorded.err"; then status=0; else status=$?; fi
  wait "$first_pid"; first_status=$?
  if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || -e "$marker" ]] || ! grep -q '排不上' "$scratch/unrecorded.err" || ! grep -q '峰值表里没有这条，按它的上限 200M 算' "$scratch/unrecorded.err"; then
    fail "峰值表里没有的那一条应当按上限 200M 算、排不上退 $MEMORY_ADMISSION_REFUSED_EXIT、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unrecorded.err")"
  fi
  [[ $first_status -eq 0 ]] || fail "表里记着 100 MiB 的那一条应当照常跑完退 0，实际退 $first_status：$(cat "$scratch/table-1.err")"

  # 要的量比总上限还大：等多久都放不下，不等，立刻退 252
  checked=$((checked + 1))
  started_seconds="$(date +%s%N)"
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-oversized bash "$target" 512M true 2>"$scratch/oversized.err"; then status=0; else status=$?; fi
  elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
  if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || $elapsed_seconds -ge 5 ]] || ! grep -q '比 slice 的总上限' "$scratch/oversized.err"; then
    fail "要 512M 而总上限 256M 时应当不等、立刻退 $MEMORY_ADMISSION_REFUSED_EXIT，实际退 $status、用了 $elapsed_seconds 秒：$(cat "$scratch/oversized.err")"
  fi

  # 峰值记进表，下一次按它算：跑一条拿 30 MiB 的，表里这个键记下的峰值在 30 MiB 与上限 200M 之间；
  # 之后两条同样的同时来，各按记下的量算，都放得下、同时在跑
  fresh_context 256M
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-small RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='自检排队这一例故意上限远大于峰值' bash "$target" 200M python3 -c "$job_code" "$scratch/small-0.start" "$scratch/small-0.end" 0 30 0 \
      2>"$scratch/small-0.err"; then status=0; else status=$?; fi
  output="$(awk -F '\t' '$4 == "selftest-small" {print $1}' "$context_directory/peaks.tsv" 2>/dev/null)"
  if [[ $status -ne 0 || ! "$output" =~ ^[0-9]+$ ]] || ((output < 30 * MEBIBYTE || output > 200 * MEBIBYTE)); then
    fail "跑完一条拿 30 MiB 的，峰值表里 selftest-small 应当记下 30 MiB 到 200M 之间的峰值，实际退 $status、记的是「$output」：$(cat "$scratch/small-0.err")"
  fi
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-small RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='自检排队这一例故意上限远大于峰值' bash "$target" 200M python3 -c "$job_code" "$scratch/small-1.start" "$scratch/small-1.end" 0 30 1.2 \
    >/dev/null 2>"$scratch/small-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-small RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='自检排队这一例故意上限远大于峰值' bash "$target" 200M python3 -c "$job_code" "$scratch/small-2.start" "$scratch/small-2.end" 0 30 1.2 \
    >/dev/null 2>"$scratch/small-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! intervals_overlap "$scratch/small-1.start" "$scratch/small-1.end" "$scratch/small-2.start" "$scratch/small-2.end"; then
    fail "峰值表里记着约 30 MiB 的两条（上限各 200M、总上限 256M）应当按记下的量一起放行、同时在跑，实际退 $first_status 与 $second_status、$(intervals_overlap "$scratch/small-1.start" "$scratch/small-1.end" "$scratch/small-2.start" "$scratch/small-2.end" && echo '同时在跑' || echo '一条一条跑的')：$(cat "$scratch/small-1.err" "$scratch/small-2.err")"
  fi

  # 外壳写峰值那一步挨得住 systemd 停 scope 时发给 scope 里每个进程的 TERM：命令在 scope 里留一个进程，等命令退出之后 1 到 2 秒里
  # 不停地（不 sleep）给 scope 里除它以外的每个进程发 TERM——外壳那时起的任何外部命令活不过一轮；峰值照样要记进表。
  # 只在这一例自己开的 scope 里发：cgroup 路径要是 …/<这一例的 slice>/singlefs-memory-cap-*.scope（slice 名经 $1 传进去），不对就不发信号、退 0。
  # 不开 scope 的路径（弄坏开关 nocap）下命令就在调用方的 cgroup 里：2026-09-25 在 SSH 会话的 session-1.scope 里发过一轮 TERM，
  # 打掉了 VSCode 扩展宿主与 Claude 会话；只核最后一段名字时，外面包着一层别的 singlefs-memory-cap scope（整条门禁经包装跑）会打到外层那一整条
  fresh_context 256M
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-term-storm bash "$target" 64M bash -c '
      group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"
      case "$group" in
        */"$1"/singlefs-memory-cap-*.scope) ;;
        *) echo "不在这一例自己的 slice $1 底下自己开的 scope 里（$group），不发信号" >&2; exit 0 ;;
      esac
      command_pid=$$
      ( while kill -0 "$command_pid" 2>/dev/null; do :; done
        storm_end=$((SECONDS + 2))
        while ((SECONDS < storm_end)); do
          while read -r process_id; do [[ "$process_id" != "$BASHPID" ]] && kill -TERM "$process_id" 2>/dev/null; done < "$group/cgroup.procs"  # process-safety:own-scope 上面 case 核过路径在这一例自己的 slice 底下、是自己开的 singlefs-memory-cap-*.scope
        done ) >/dev/null 2>&1 &
      exit 0' term-storm "$slice" 2>"$scratch/term-storm.err"; then status=0; else status=$?; fi
  output="$(awk -F '\t' '$4 == "selftest-term-storm" {print $1}' "$context_directory/peaks.tsv" 2>/dev/null)"
  if [[ ! "$output" =~ ^[0-9]+$ ]]; then
    fail "scope 里一直有进程收 TERM 时，外壳照样要把峰值记进表（selftest-term-storm 那一行），实际退 $status、记的是「$output」：$(cat "$scratch/term-storm.err")"
  fi

  # 起了 scope 之后包装收到 TERM / INT（欠账 C801）：先 systemctl --user stop 自己开的那个 scope 单元（在删标记与账之前），scope 里的命令随之停，
  # 包装退 143 / 130，标记、账与 scope 都不留。systemctl 是 PATH 前面的一层转接：记下每一次 stop（单元名、那一刻标记与账还在不在），再交给真的；
  # 包装经一层 python 起，把 INT 设回默认（同下面「排队中停包装」）。命令是 exec sleep 60：没被停的话它一直活着（弄坏开关 term-leaves-scope）；
  # 这一例收尾自己停自己的 slice，不让它留到最后。信号只发给这里 & 起、$! 记下的那个包装。
  scope_stop_process_is_live() { # scope_stop_process_is_live <进程号>：在跑（没有、或已是僵尸都算没在跑）交回 0
    local stat_line
    IFS= read -r stat_line 2>/dev/null < "/proc/$1/stat" || return 1
    stat_line="${stat_line##*) }"
    [[ "${stat_line%% *}" != Z ]]
  }
  scope_stop_case() { # scope_stop_case <TERM|INT> <包装该退的码>
    local stop_signal="$1" expected_status="$2" case_directory="$scratch/scope-stop-$1" wrapper_pid command_pid="" poll wrapper_status=0
    local ready=0 wrapper_exited=0 command_exited=0 scope_left="" marker_left="" ledger_left="" stop_line=""
    mkdir -p "$case_directory/bin" "$case_directory/tmp"
    : > "$case_directory/calls"
    printf '%s\n' '#!/usr/bin/env bash' \
      'if [[ "${1:-}" == --user && "${2:-}" == stop ]]; then' \
      '  marker=absent; ledger=absent' \
      '  compgen -G "$STUB_TMPDIR/run-with-memory-cap-*.started" >/dev/null && marker=present' \
      '  [[ -e "$STUB_LEDGER_DIRECTORY/${3%.scope}.json" ]] && ledger=present' \
      '  echo "stop $3 marker=$marker ledger=$ledger" >> "$STUB_CALLS"' \
      'fi' \
      'exec "$STUB_REAL_SYSTEMCTL" "$@"' > "$case_directory/bin/systemctl"
    chmod +x "$case_directory/bin/systemctl"
    python3 -c 'import os, signal, sys; signal.signal(signal.SIGINT, signal.SIG_DFL); signal.signal(signal.SIGHUP, signal.SIG_DFL); os.execvp(sys.argv[1], sys.argv[1:])' \
      env "${context[@]}" PATH="$case_directory/bin:$PATH" TMPDIR="$case_directory/tmp" STUB_TMPDIR="$case_directory/tmp" STUB_CALLS="$case_directory/calls" \
      STUB_LEDGER_DIRECTORY="$context_directory/state/ledger" STUB_REAL_SYSTEMCTL="$(command -v systemctl)" RUN_WITH_MEMORY_CAP_KEY="selftest-scope-stop-$stop_signal" \
      bash "$target" 64M bash -c 'echo $$ > "$1"; exec sleep 60' scope-stop "$case_directory/command-pid" >"$case_directory/output" 2>&1 &
    wrapper_pid=$!
    for ((poll = 0; poll < 150; poll++)); do   # 至多 15 秒等命令在 scope 里起来（它把自己的进程号写进文件）
      if [[ -s "$case_directory/command-pid" ]]; then ready=1; break; fi
      sleep 0.1
    done
    command_pid="$(cat -- "$case_directory/command-pid" 2>/dev/null)"
    if ((ready)); then
      kill -"$stop_signal" "$wrapper_pid"
      for ((poll = 0; poll < 100; poll++)); do   # 至多 10 秒
        scope_stop_process_is_live "$wrapper_pid" || { wrapper_exited=1; break; }
        sleep 0.1
      done
      ((wrapper_exited)) || kill -KILL "$wrapper_pid" 2>/dev/null
      if [[ "$command_pid" =~ ^[0-9]+$ ]]; then
        for ((poll = 0; poll < 50; poll++)); do   # 至多 5 秒
          scope_stop_process_is_live "$command_pid" || { command_exited=1; break; }
          sleep 0.1
        done
      fi
    else
      kill -KILL "$wrapper_pid" 2>/dev/null
    fi
    wait "$wrapper_pid"; wrapper_status=$?
    scope_left="$(compgen -G "/sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/$slice/*.scope")"
    marker_left="$(compgen -G "$case_directory/tmp/run-with-memory-cap-*.started")"
    ledger_left="$(compgen -G "$context_directory/state/ledger/singlefs-memory-cap-$wrapper_pid-*.json")"
    stop_line="$(grep -E "^stop singlefs-memory-cap-$wrapper_pid-[0-9]+\.scope marker=present ledger=present\$" "$case_directory/calls" | head -n 1)"
    systemctl --user stop "$slice" >/dev/null 2>&1
    if ((ready == 0)); then
      fail "起了 scope 之后给包装发 $stop_signal 这一例没判成：命令 15 秒里没在 scope 里起来，包装退 $wrapper_status：$(cat "$case_directory/output")"
    elif [[ -z "$stop_line" || $wrapper_exited -eq 0 || $command_exited -eq 0 || $wrapper_status -ne $expected_status || -n "$scope_left$marker_left$ledger_left" ]]; then
      fail "起了 scope 之后给包装发 $stop_signal：应当先 systemctl --user stop 自己开的 scope 单元（那一刻标记与账还在）、scope 里的命令 5 秒内随之停、包装 10 秒内退 $expected_status、标记账与 scope 都不留；\
实际 stop 调用「$(cat "$case_directory/calls")」、包装$([[ $wrapper_exited -eq 1 ]] && echo '退了' || echo '10 秒里没退')（$wrapper_status）、\
命令 ${command_pid:-没记下}$([[ $command_exited -eq 1 ]] && echo ' 停了' || echo ' 5 秒里没停')、留下的 scope「${scope_left}」标记「${marker_left}」账「${ledger_left}」：$(cat "$case_directory/output")"
    fi
  }
  fresh_context 256M
  checked=$((checked + 1))
  scope_stop_case TERM 143
  fresh_context 256M
  checked=$((checked + 1))
  scope_stop_case INT 130

  slice_total_cells

  # 限时：超过 RUN_WITH_MEMORY_CAP_TIME_LIMIT 退 253，不跑满命令自己的时长
  checked=$((checked + 1))
  started_seconds="$(date +%s%N)"
  if in_context env RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 RUN_WITH_MEMORY_CAP_KILL_GRACE=2 bash "$target" 64M sleep 4 2>"$scratch/timelimit.err"; then status=0; else status=$?; fi
  elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
  if [[ $status -ne $TIME_LIMIT_HIT_EXIT || $elapsed_seconds -ge 4 ]] || ! grep -q '超过限时 1 秒' "$scratch/timelimit.err"; then
    fail "限时 1 秒跑 sleep 4 应当 4 秒之内退 $TIME_LIMIT_HIT_EXIT、报「超过限时 1 秒」，实际退 $status、用了 $elapsed_seconds 秒：$(cat "$scratch/timelimit.err")"
  fi

  # 账上包装已经死了的那一笔（进程号是 pid_max，这台机器上不会有这个进程）不算：记着要 250M 的死账不挡一条要 200M 的
  checked=$((checked + 1))
  mkdir -p "$context_directory/state/ledger"
  printf '{"unit": "singlefs-memory-cap-dead", "pid": %s, "pid_start_time": "1", "need": %s, "cap": "250M", "key": "dead", "admitted_at": "'"$selftest_admitted_at"'"}' \
    "$(cat /proc/sys/kernel/pid_max)" $((250 * MEBIBYTE)) > "$context_directory/state/ledger/singlefs-memory-cap-dead.json"
  if in_context env RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1 bash "$target" 200M true 2>"$scratch/stale.err"; then status=0; else status=$?; fi
  if [[ $status -ne 0 || -e "$context_directory/state/ledger/singlefs-memory-cap-dead.json" ]]; then
    fail "账上那一笔的包装已经死了，应当删掉它、放行这一条退 0，实际退 $status、那一笔$([[ -e "$context_directory/state/ledger/singlefs-memory-cap-dead.json" ]] && echo '还在' || echo '删了')：$(cat "$scratch/stale.err")"
  fi

  # 盯账那几项（over_booking_cells，定义在上面）：各例自己开新 slice
  over_booking_cells

  # 排队中停包装（信号只发给包装进程本身、不发给它的进程组，2026-09-26 撞见孤儿的就是这样停的）：排队那一步的子进程 5 秒内跟着退、
  # 调用方的管道 5 秒内读到 EOF、包装退 143 / 130（TERM / INT，trap 接住）或 137 / 129（KILL / HUP，被信号杀掉）、命令没跑、账上与 --status 里都没有这一条。
  # 挡路的是账上一笔假的：包装进程号写这个自证自己（活着、起始时刻对得上），要 250M，它的 scope 不存在、此刻占 0；这一条要 200M，250 + 200 放不进 256M，只能排队。
  # 停完删掉挡路那一笔、等 1.5 秒（多于排队重判的间隔 1 秒）：排队那一步没跟着退的话，它这时放行、在账上记一笔，账上那一格判得出来。
  # 包装经一层 python 起：脚本里 & 起的命令忽略 INT、进了 bash 也 trap 不上，先把 INT 设回默认再 exec 包装，进程号不变；
  # HUP 也设回默认：自证在 nohup 底下跑时 HUP 是忽略的，包装停不下来。
  # KILL、HUP 两例包装没有 trap（KILL trap 不上，HUP 不设），排队那一步靠每轮核父进程退出，至多晚一个重判间隔（1 秒），5 秒够。
  # 信号只发给这里 & 起、$! 记下的那个包装；还没 wait 过它，它退了也只是僵尸，进程号不会被别人复用
  start_time_of_live_process() { # start_time_of_live_process <进程号> → /proc/<进程号>/stat 第 22 段；进程不在或已成僵尸交空
    local stat_line
    local -a stat_fields=()
    IFS= read -r stat_line 2>/dev/null < "/proc/$1/stat" || return 0
    read -r -a stat_fields <<<"${stat_line##*) }"
    [[ "${stat_fields[0]:-}" == "Z" ]] || printf '%s' "${stat_fields[19]:-}"
  }
  queued_stop_case() { # queued_stop_case <TERM|INT> <包装该退的码>
    local stop_signal="$1" expected_status="$2" case_directory="$scratch/queued-stop-$1" wrapper_pid reader_pid poll wrapper_status leftover status_output
    local queued_child_pid="" queued_child_start="" queued_line_seen=0 child_exited=0 end_of_file_seen=0
    local -a child_pids=()
    mkdir -p "$case_directory" "$context_directory/state/ledger"
    printf '{"unit": "singlefs-memory-cap-selftest-blocker", "pid": %s, "pid_start_time": "%s", "need": %s, "cap": "250M", "key": "blocker", "admitted_at": "'"$selftest_admitted_at"'"}' \
      "$$" "$(start_time_of_live_process "$$")" $((250 * MEBIBYTE)) > "$context_directory/state/ledger/singlefs-memory-cap-selftest-blocker.json"
    mkfifo "$case_directory/pipe"
    { cat "$case_directory/pipe" > "$case_directory/output"; : > "$case_directory/end-of-file"; } &
    reader_pid=$!
    python3 -c 'import os, signal, sys; signal.signal(signal.SIGINT, signal.SIG_DFL); signal.signal(signal.SIGHUP, signal.SIG_DFL); os.execvp(sys.argv[1], sys.argv[1:])' \
      env "${context[@]}" RUN_WITH_MEMORY_CAP_KEY="selftest-queued-stop-$stop_signal" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=30 \
      bash "$target" 200M bash -c ': > "$1"' touch-marker "$case_directory/command-ran" >"$case_directory/pipe" 2>&1 &
    wrapper_pid=$!
    for ((poll = 0; poll < 200; poll++)); do   # 至多 20 秒等它排上队：python 那一半第一次判放不下就报一行「排队」
      if grep -q '排队（已等' "$case_directory/output" 2>/dev/null; then queued_line_seen=1; break; fi
      sleep 0.1
    done
    if ((queued_line_seen)); then   # children 那一行末尾没有换行，read 读到它也退 1，所以不拿 read 的退出码判
      read -r -a child_pids 2>/dev/null < "/proc/$wrapper_pid/task/$wrapper_pid/children"
      queued_child_pid="${child_pids[0]:-}"
      [[ -n "$queued_child_pid" ]] && queued_child_start="$(start_time_of_live_process "$queued_child_pid")"
    fi
    kill -"$stop_signal" "$wrapper_pid"
    for ((poll = 0; poll < 50; poll++)); do   # 至多 5 秒
      [[ -n "$queued_child_start" && "$(start_time_of_live_process "$queued_child_pid")" != "$queued_child_start" ]] && child_exited=1
      [[ -e "$case_directory/end-of-file" ]] && end_of_file_seen=1
      ((child_exited && end_of_file_seen)) && break
      sleep 0.1
    done
    rm -f -- "$context_directory/state/ledger/singlefs-memory-cap-selftest-blocker.json"
    sleep 1.5
    leftover="$(compgen -G "$context_directory/state/ledger/singlefs-memory-cap-$wrapper_pid-*.json")"
    status_output="$(in_context bash "$target" --status 2>&1)"
    wait "$wrapper_pid"; wrapper_status=$?
    wait "$reader_pid"
    if ((queued_line_seen == 0)); then
      fail "排队中给包装发 $stop_signal 这一例没判成：包装 20 秒里没排上队（挡路那一笔要 250M、这一条要 200M、总上限 256M），包装退 $wrapper_status：$(cat "$case_directory/output")"
    elif ((child_exited == 0 || end_of_file_seen == 0)) || [[ $wrapper_status -ne $expected_status || -n "$leftover" || -e "$case_directory/command-ran" ]] \
        || [[ "$status_output" == *"selftest-queued-stop-$stop_signal"* || "$status_output" == *"singlefs-memory-cap-$wrapper_pid-"* ]]; then
      fail "排队中给包装发 $stop_signal：排队那一步的子进程应当 5 秒内跟着退、调用方的管道 5 秒内读到 EOF、包装退 $expected_status、命令不跑、账上与 --status 里都没有这一条；\
实际子进程 ${queued_child_pid:-没找到} $([[ $child_exited -eq 1 ]] && echo '跟着退了' || echo '5 秒里没退')、管道$([[ $end_of_file_seen -eq 1 ]] && echo '读到了 EOF' || echo '5 秒里没读到 EOF')、包装退 $wrapper_status、\
命令$([[ -e "$case_directory/command-ran" ]] && echo '跑了' || echo '没跑')、账上${leftover:+留着 }${leftover:-没有这一条}、--status 里$([[ "$status_output" == *"selftest-queued-stop-$stop_signal"* || "$status_output" == *"singlefs-memory-cap-$wrapper_pid-"* ]] && echo '有' || echo '没有')这一条：$(cat "$case_directory/output")"
    fi
  }
  fresh_context 256M
  checked=$((checked + 1))
  queued_stop_case TERM 143
  checked=$((checked + 1))
  queued_stop_case INT 130
  # KILL / HUP 两例与下面「放行那一刻包装已经不在」一例的包装是被信号杀掉的：bash 在它死后的下一条前台命令之后往自己的 stderr 补一行
  # 「<脚本>: line N: <进程号> Killed|Hangup <命令>」，那不是自证的输出。这几例的 stderr 先接到文件，只把别的行照原样打回 stderr
  without_job_notices() { # without_job_notices <函数> <参数…>
    local notices_file="$scratch/job-notices-$slice_counter"
    "$@" 2>"$notices_file"
    grep -Ev '^[^:]+: line [0-9]+: +[0-9]+ (Killed|Hangup) ' "$notices_file" >&2
    return 0
  }
  fresh_context 256M
  checked=$((checked + 1))
  without_job_notices queued_stop_case KILL 137
  fresh_context 256M
  checked=$((checked + 1))
  without_job_notices queued_stop_case HUP 129

  # 放行那一刻包装已经不在（这一轮开头核父进程时包装还在，判完放得下、记账之前被 KILL 并回收）：不记账、排队那一步退出、命令没跑。
  # 判完放得下之后排队那一步拿着锁停 RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE 秒：锁隔 0.3 秒连着两次被占着，就是停在那里了（判放不放得下只要几毫秒）。
  # 信号只发给这里 & 起、$! 记下的那个包装，发完当场 wait 回收它
  lock_held_by_someone() { # lock_held_by_someone <锁文件>：锁文件在、而此刻拿不到它才退 0
    local lock_status
    [[ -e "$1" ]] || return 1
    if flock -n -E 75 "$1" true 2>/dev/null; then lock_status=0; else lock_status=$?; fi
    [[ $lock_status -eq 75 ]]
  }
  gone_at_admission_case() {
    local case_directory="$scratch/gone-at-admission" lock_path="$context_directory/state/lock" wrapper_pid wrapper_status poll paused=0
    local admission_child_pid="" admission_child_start="" child_exited=0 leftover
    local -a child_pids=()
    mkdir -p "$case_directory"
    # 不经 in_context（shell 函数放后台是一层子 shell，$! 记下的就不是包装）：env 直接 exec 包装，进程号不变
    env "${context[@]}" RUN_WITH_MEMORY_CAP_KEY=selftest-gone-at-admission RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE=3 \
      bash "$target" 200M bash -c ': > "$1"' touch-marker "$case_directory/command-ran" >"$case_directory/output" 2>&1 &
    wrapper_pid=$!
    for ((poll = 0; poll < 100; poll++)); do   # 至多约 10 秒
      if lock_held_by_someone "$lock_path"; then
        sleep 0.3
        if lock_held_by_someone "$lock_path"; then paused=1; break; fi
      fi
      sleep 0.1
    done
    if ((paused)); then   # children 那一行末尾没有换行，read 读到它也退 1，所以不拿 read 的退出码判
      read -r -a child_pids 2>/dev/null < "/proc/$wrapper_pid/task/$wrapper_pid/children"
      admission_child_pid="${child_pids[0]:-}"
      [[ -n "$admission_child_pid" ]] && admission_child_start="$(start_time_of_live_process "$admission_child_pid")"
    fi
    kill -KILL "$wrapper_pid"
    wait "$wrapper_pid"; wrapper_status=$?
    for ((poll = 0; poll < 80; poll++)); do   # 至多 8 秒：停顿还剩不到 3 秒
      if [[ -n "$admission_child_start" && "$(start_time_of_live_process "$admission_child_pid")" != "$admission_child_start" ]]; then child_exited=1; break; fi
      sleep 0.1
    done
    leftover="$(compgen -G "$context_directory/state/ledger/singlefs-memory-cap-$wrapper_pid-*.json")"
    if ((paused == 0)) || [[ -z "$admission_child_start" ]]; then
      fail "放行那一刻包装已经不在这一例没判成：10 秒里没看到排队那一步拿着锁停在记账之前、或没找到它（${admission_child_pid:-没找到}），包装退 $wrapper_status：$(cat "$case_directory/output")"
    elif ((child_exited == 0)) || [[ -n "$leftover" || -e "$case_directory/command-ran" ]]; then
      fail "判完放得下、记账之前包装被 KILL 并回收：排队那一步应当不记账、退出，命令不跑；实际排队那一步 $admission_child_pid $([[ $child_exited -eq 1 ]] && echo '退了' || echo '8 秒里没退')、\
账上${leftover:+留着 }${leftover:-没有这一条}${leftover:+（$(cat "$leftover" 2>/dev/null)）}、命令$([[ -e "$case_directory/command-ran" ]] && echo '跑了' || echo '没跑')、包装退 $wrapper_status：$(cat "$case_directory/output")"
    fi
  }
  fresh_context 256M
  checked=$((checked + 1))
  without_job_notices gone_at_admission_case

  # 账上起始时刻记成空（null：包装不在时放行记下的）、进程号此刻也不在（pid_max，这台机器上不会有这个进程）的那一笔算死账：要 250M 也不挡一条要 200M 的，而且删掉；
  # 同一本账上起始时刻是字符串、包装活着（进程号写这个自证自己，起始时刻对得上）的那一笔（要 20M）照旧算活的、不删：20 + 200 放得进 256M
  fresh_context 256M
  checked=$((checked + 1))
  mkdir -p "$context_directory/state/ledger"
  printf '{"unit": "singlefs-memory-cap-null-start", "pid": %s, "pid_start_time": null, "need": %s, "cap": "250M", "key": "null-start", "admitted_at": "'"$selftest_admitted_at"'"}' \
    "$(cat /proc/sys/kernel/pid_max)" $((250 * MEBIBYTE)) > "$context_directory/state/ledger/singlefs-memory-cap-null-start.json"
  printf '{"unit": "singlefs-memory-cap-selftest-live", "pid": %s, "pid_start_time": "%s", "need": %s, "cap": "20M", "key": "live", "admitted_at": "'"$selftest_admitted_at"'"}' \
    "$$" "$(start_time_of_live_process "$$")" $((20 * MEBIBYTE)) > "$context_directory/state/ledger/singlefs-memory-cap-selftest-live.json"
  if in_context env RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1 bash "$target" 200M true 2>"$scratch/null-start.err"; then status=0; else status=$?; fi
  if [[ $status -ne 0 || -e "$context_directory/state/ledger/singlefs-memory-cap-null-start.json" || ! -e "$context_directory/state/ledger/singlefs-memory-cap-selftest-live.json" ]]; then
    fail "账上起始时刻记成 null、包装不在的那一笔应当算死账、删掉、放行这一条退 0，起始时刻是字符串、包装活着的那一笔应当留着；实际退 $status、\
null 那一笔$([[ -e "$context_directory/state/ledger/singlefs-memory-cap-null-start.json" ]] && echo '还在' || echo '删了')、\
活着的那一笔$([[ -e "$context_directory/state/ledger/singlefs-memory-cap-selftest-live.json" ]] && echo '留着' || echo '被当成死账删了')：$(cat "$scratch/null-start.err")"
  fi
  rm -f -- "$context_directory/state/ledger/singlefs-memory-cap-selftest-live.json"

  # 账上没有在跑的而放不下（文件头「排队」一段）：当场退 252、不排队、命令没跑，出路写要多少、放得下的最多多少；账上有一条在跑时照旧排队、等满再退 252。
  # slice 是假的：这一例的 slice 先经 --check 在真的 cgroup 里起一次（总上限 41062M，约 40.1 GiB），再把 cgroup 根指到自己造的目录，
  # 在那个 slice 的 cgroup 路径下放 memory.max（同一个总上限）、memory.current 与 memory.stat（回收不掉的 420 MiB，没有页缓存）。这一条上限 40G、设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP，按 40G 算
  # （峰值表里没有的按 host-resources.py 的预测 8G 算，就放得下了；这两例判的是放不下时排不排队）。
  # 两例都排不上，scope 一个都不起。当场那一例限等 4 秒：判错照旧排队的话等满 4 秒，「用了几秒」那一格判得出来；排队那一例限等 2 秒。
  # 挡路那一笔的包装进程号写这个自证自己（活着、起始时刻对得上），要 100M，它的 scope 不在假根里、此刻占 0
  fresh_context 41062M
  local hopeless_root="$scratch/hopeless-cgroup" hopeless_group
  in_context bash "$target" --check 64M 2>"$scratch/hopeless-check.err"
  hopeless_group="$(systemctl --user show -p ControlGroup --value "$slice" 2>/dev/null)"
  checked=$((checked + 1))
  if [[ -z "$hopeless_group" ]]; then
    fail "账上没有在跑的而放不下这一例没判成：--check 之后查不到 slice $slice 的 ControlGroup：$(cat "$scratch/hopeless-check.err")"
  else
    mkdir -p "$hopeless_root$hopeless_group" "$context_directory/state/ledger"
    echo $((41062 * MEBIBYTE)) > "$hopeless_root$hopeless_group/memory.max"
    echo $((41062 * MEBIBYTE * SLICE_HIGH_PERMILLE / 1000 / MEBIBYTE * MEBIBYTE)) > "$hopeless_root$hopeless_group/memory.high"   # 假根里 slice 的 memory.high 已是该设的数，不去设真的 slice
    echo $((420 * MEBIBYTE)) > "$hopeless_root$hopeless_group/memory.current"
    printf 'anon %s\nactive_file 0\ninactive_file 0\n' $((420 * MEBIBYTE)) > "$hopeless_root$hopeless_group/memory.stat"
    marker="$scratch/ran-hopeless"
    started_seconds="$(date +%s%N)"
    if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$hopeless_root" RUN_WITH_MEMORY_CAP_KEY=selftest-hopeless RUN_WITH_MEMORY_CAP_WAIT_SECONDS=4 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP='自证：按上限 40G 记账' \
        bash "$target" 40G bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/hopeless.err"; then status=0; else status=$?; fi
    elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || -e "$marker" || $elapsed_seconds -ge 3 ]] || grep -q '排队（已等' "$scratch/hopeless.err" \
        || ! grep -qF '没有哪一条跑完会放出来' "$scratch/hopeless.err" || ! grep -qF '要 40.0 GiB、放得下的最多 39.7 GiB' "$scratch/hopeless.err"; then
      fail "账上没有在跑的、要 40G 而回收不掉 420M、总上限 40.1G 时应当不等、当场退 $MEMORY_ADMISSION_REFUSED_EXIT、出路写「要 40.0 GiB、放得下的最多 39.7 GiB」、命令不跑，\
实际退 $status、用了 $elapsed_seconds 秒、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/hopeless.err")"
    fi
    checked=$((checked + 1))
    printf '{"unit": "singlefs-memory-cap-selftest-running", "pid": %s, "pid_start_time": "%s", "need": %s, "cap": "100M", "key": "running", "admitted_at": "'"$selftest_admitted_at"'"}' \
      "$$" "$(start_time_of_live_process "$$")" $((100 * MEBIBYTE)) > "$context_directory/state/ledger/singlefs-memory-cap-selftest-running.json"
    # 账上有一条在跑，而 slice 里那 420 MiB 常驻不属于任何 scope（假根里 slice 底下没有子 cgroup）：要 40G 加 420M 常驻超总上限 40.1G，账上那一条跑完也不会少，
    # 等多久都放不下，应当不等、当场退 252，出路写放得下的最多 39.7 GiB（待优化第 19 行：第二台总上限 50.2G、常驻约 640M、上限 50G 白排 3600 秒）
    marker="$scratch/ran-resident"
    started_seconds="$(date +%s%N)"
    if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$hopeless_root" RUN_WITH_MEMORY_CAP_KEY=selftest-resident RUN_WITH_MEMORY_CAP_WAIT_SECONDS=4 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP='自证：按上限 40G 记账' \
        bash "$target" 40G bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/resident.err"; then status=0; else status=$?; fi
    elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
    checked=$((checked + 1))
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || -e "$marker" || $elapsed_seconds -ge 3 ]] || grep -q '排队（已等' "$scratch/resident.err" \
        || ! grep -qF '不属于任何 scope 的常驻用量' "$scratch/resident.err" || ! grep -qF '要 40.0 GiB、放得下的最多 39.7 GiB' "$scratch/resident.err"; then
      fail "账上有一条在跑、而 slice 里 420M 常驻不属于任何 scope、要 40G 加常驻超总上限 40.1G 时应当不等、当场退 $MEMORY_ADMISSION_REFUSED_EXIT、出路写「要 40.0 GiB、放得下的最多 39.7 GiB」、命令不跑，\
实际退 $status、用了 $elapsed_seconds 秒、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/resident.err")"
    fi
    # 下一例的 420M 是账上在跑的那一条的 scope 占着的（假根里 slice 底下有它的 scope 目录）：它跑完就放出来，没有常驻，照旧排队
    mkdir -p "$hopeless_root$hopeless_group/singlefs-memory-cap-selftest-running.scope"
    echo $((420 * MEBIBYTE)) > "$hopeless_root$hopeless_group/singlefs-memory-cap-selftest-running.scope/memory.current"
    printf 'anon %s\nactive_file 0\ninactive_file 0\n' $((420 * MEBIBYTE)) > "$hopeless_root$hopeless_group/singlefs-memory-cap-selftest-running.scope/memory.stat"
    marker="$scratch/ran-hopeful"
    started_seconds="$(date +%s%N)"
    if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$hopeless_root" RUN_WITH_MEMORY_CAP_KEY=selftest-hopeful RUN_WITH_MEMORY_CAP_WAIT_SECONDS=2 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP='自证：按上限 40G 记账' \
        bash "$target" 40G bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/hopeful.err"; then status=0; else status=$?; fi
    elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
    rm -f -- "$context_directory/state/ledger/singlefs-memory-cap-selftest-running.json"
    if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || -e "$marker" || $elapsed_seconds -lt 2 ]] || ! grep -q '排队（已等' "$scratch/hopeful.err" \
        || ! grep -q '等了 [0-9]* 秒' "$scratch/hopeful.err" || grep -qF '没有哪一条跑完会放出来' "$scratch/hopeful.err"; then
      fail "账上有一条在跑时（要 40G、回收不掉 420M、总上限 40.1G）应当照旧排队、等满 2 秒退 $MEMORY_ADMISSION_REFUSED_EXIT、命令不跑，\
实际退 $status、用了 $elapsed_seconds 秒、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/hopeful.err")"
    fi
  fi

  # 下面两例共用：在自己的 scope 里抄下账上自己那一笔要的量与自己 cgroup 的 memory.max（argv：起、收、账目录、抄到哪个文件、自己的键），睡半秒
  local booked_probe_code
  booked_probe_code="$(cat <<'BOOKED_PROBE'
import glob, json, sys, time
start_path, end_path, ledger_directory, seen_path, own_key = sys.argv[1:6]
open(start_path, "w").write(repr(time.time()))
entries = [json.load(open(path)) for path in glob.glob(ledger_directory + "/*.json")]
needs = [entry["need"] for entry in entries if entry.get("key") == own_key]
group = open("/proc/self/cgroup").read().strip().split(":", 2)[2]
try:
    memory_max = open("/sys/fs/cgroup" + group + "/memory.max").read().strip()
except OSError as error:
    memory_max = type(error).__name__
open(seen_path, "w").write(" ".join(str(need) for need in needs) + "|" + memory_max)
time.sleep(0.5)
open(end_path, "w").write(repr(time.time()))
BOOKED_PROBE
)"

  # 峰值表里没有这条而上限写得比预测大（文件头「排队」一段）：记账按 host-resources.py 对它的预测与上限取小，scope 的上限照旧是写的那个。
  # 总上限 40G：前一条按上限 24G 记账（设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP）、在跑 4 秒；后一条峰值表里没有、写 32G，按预测 8G 记账，
  # 24G + 8G 放得下，两条同时在跑。照上限记账的话 24G + 32G 超总上限，后一条排满 2 秒退 252（或等前一条跑完才起，两条不同时）。
  # 后一条在自己的 scope 里抄下账上自己那一笔要的量与自己 cgroup 的 memory.max。两条只睡、不占内存，scope 的上限只是个数
  fresh_context 40G
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-booked-by-cap RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP='自证：前一条按上限 24G 记账' bash "$target" 24G \
    python3 -c "$job_code" "$scratch/predicted-1.start" "$scratch/predicted-1.end" 0 0 4 >/dev/null 2>"$scratch/predicted-1.err" &
  first_pid=$!
  sleep 0.6
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-predicted RUN_WITH_MEMORY_CAP_WAIT_SECONDS=2 bash "$target" 32G python3 -c "$booked_probe_code" \
      "$scratch/predicted-2.start" "$scratch/predicted-2.end" "$context_directory/state/ledger" "$scratch/predicted-2.seen" selftest-predicted >/dev/null 2>"$scratch/predicted-2.err"; then status=0; else status=$?; fi
  wait "$first_pid"; first_status=$?
  output="$(cat "$scratch/predicted-2.seen" 2>/dev/null)"
  if [[ $status -ne 0 || $first_status -ne 0 || "$output" != "$((8 * 1024 * MEBIBYTE))|$((32 * 1024 * MEBIBYTE))" ]] \
      || ! intervals_overlap "$scratch/predicted-1.start" "$scratch/predicted-1.end" "$scratch/predicted-2.start" "$scratch/predicted-2.end"; then
    fail "峰值表里没有、写 32G 的那一条应当按 host-resources.py 的预测 8G 记账，在总上限 40G 里与按上限 24G 记账的那一条同时在跑、都退 0，scope 的 memory.max 照旧是 32G，\
实际退 $status 与 $first_status（前一条）、$(intervals_overlap "$scratch/predicted-1.start" "$scratch/predicted-1.end" "$scratch/predicted-2.start" "$scratch/predicted-2.end" && echo '同时在跑' || echo '没有同时在跑')、\
账上要的量|memory.max 是「$output」：$(cat "$scratch/predicted-2.err" "$scratch/predicted-1.err")"
  fi

  # 峰值表里量过的峰值比写的上限小很多（待优化第 19 行：合入后验证写 24G、实际只用 1.8G，排在后面的 24G 测试白排了 1500 秒）：记账取峰值与上限中较小的。
  # 总上限 40G：前一条按上限 24G 记账（设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP）、在跑 4 秒；后一条峰值表里记着 1843 MiB（约 1.8G，那一次的上限也是 24G，没撞顶）、这一次写 24G，
  # 应当按 1843 MiB 记账，24G + 1.8G 放得下，两条同时在跑，scope 的 memory.max 照旧是 24G。按上限记账的话 24G + 24G 超总上限，后一条排满 2 秒退 252。
  fresh_context 40G
  checked=$((checked + 1))
  seed_peak selftest-peak-booked $((1843 * MEBIBYTE)) 24G
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-peak-holder RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP='自证：前一条按上限 24G 记账' bash "$target" 24G \
    python3 -c "$job_code" "$scratch/peak-1.start" "$scratch/peak-1.end" 0 0 4 >/dev/null 2>"$scratch/peak-1.err" &
  first_pid=$!
  sleep 0.6
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-peak-booked RUN_WITH_MEMORY_CAP_WAIT_SECONDS=2 bash "$target" 24G python3 -c "$booked_probe_code" \
      "$scratch/peak-2.start" "$scratch/peak-2.end" "$context_directory/state/ledger" "$scratch/peak-2.seen" selftest-peak-booked >/dev/null 2>"$scratch/peak-2.err"; then status=0; else status=$?; fi
  wait "$first_pid"; first_status=$?
  output="$(cat "$scratch/peak-2.seen" 2>/dev/null)"
  if [[ $status -ne 0 || $first_status -ne 0 || "$output" != "$((1843 * MEBIBYTE))|$((24 * 1024 * MEBIBYTE))" ]] \
      || ! intervals_overlap "$scratch/peak-1.start" "$scratch/peak-1.end" "$scratch/peak-2.start" "$scratch/peak-2.end"; then
    fail "峰值表里记着 1.8G（1843 MiB）、这一次写 24G 上限的一条应当按 1843 MiB 记账，在总上限 40G 里与按上限 24G 记账的那一条同时在跑、都退 0，scope 的 memory.max 照旧是 24G，\
实际退 $status 与 $first_status（前一条）、$(intervals_overlap "$scratch/peak-1.start" "$scratch/peak-1.end" "$scratch/peak-2.start" "$scratch/peak-2.end" && echo '同时在跑' || echo '没有同时在跑')、\
账上要的量|memory.max 是「$output」：$(cat "$scratch/peak-2.err" "$scratch/peak-1.err")"
  fi

  # 峰值表的「线程数」一列：命令里经 capped.sh 3 跑的记 3；经 capped.sh default 跑的记登记值（登记文件第一条非注释行的数），SINGLEFS_THREAD_CAP 设了的记它；
  # 环境里线程变量都是 2 的记 2；不一样的逐个记「名=值」；都没设、命令也没经 capped.sh 的记「未设（整机 N 核）」（没限线程的命令，实际能用整机的核）；
  # 加这一列之前记的四列那一行写回之后照原样是四列
  local -a thread_variable_unsets=()
  local thread_variable_name expectation expected_key expected_threads registered_threads
  while IFS='=' read -r thread_variable_name _; do thread_variable_unsets+=(-u "$thread_variable_name"); done < <(bash "$SCRIPT_DIRECTORY/capped.sh" --print 1)
  fresh_context 256M
  seed_peak selftest-four-columns $((10 * MEBIBYTE))
  in_context env "${thread_variable_unsets[@]}" RUN_WITH_MEMORY_CAP_KEY=selftest-threads-capped bash "$target" 64M bash "$SCRIPT_DIRECTORY/capped.sh" 3 true 2>>"$scratch/threads.err"
  in_context env "${thread_variable_unsets[@]}" CARGO_BUILD_JOBS=2 SINGLEFS_THREAD_CAP=2 RUN_WITH_MEMORY_CAP_KEY=selftest-threads-same bash "$target" 64M true 2>>"$scratch/threads.err"
  in_context env "${thread_variable_unsets[@]}" CARGO_BUILD_JOBS=2 SINGLEFS_CRASH_POINT_REPLAY_THREADS=1 RUN_WITH_MEMORY_CAP_KEY=selftest-threads-differing bash "$target" 64M true \
    2>>"$scratch/threads.err"
  in_context env "${thread_variable_unsets[@]}" RUN_WITH_MEMORY_CAP_KEY=selftest-threads-unset bash "$target" 64M true 2>>"$scratch/threads.err"
  registered_threads="$(awk '!/^[[:space:]]*(#|$)/ {print $1; exit}' "$SCRIPT_DIRECTORY/../../.claude/heavy-thread-default" 2>/dev/null)"
  in_context env "${thread_variable_unsets[@]}" RUN_WITH_MEMORY_CAP_KEY=selftest-threads-default bash "$target" 64M bash "$SCRIPT_DIRECTORY/capped.sh" default true 2>>"$scratch/threads.err"
  in_context env "${thread_variable_unsets[@]}" SINGLEFS_THREAD_CAP=5 RUN_WITH_MEMORY_CAP_KEY=selftest-threads-default-cap bash "$target" 64M bash "$SCRIPT_DIRECTORY/capped.sh" default true \
    2>>"$scratch/threads.err"
  for expectation in "selftest-threads-capped|3" "selftest-threads-same|2" "selftest-threads-differing|CARGO_BUILD_JOBS=2,SINGLEFS_CRASH_POINT_REPLAY_THREADS=1" \
      "selftest-threads-unset|未设（整机 $(nproc) 核）" "selftest-threads-default|$registered_threads" "selftest-threads-default-cap|5"; do
    expected_key="${expectation%%|*}" expected_threads="${expectation#*|}"
    checked=$((checked + 1))
    output="$(awk -F '\t' -v key="$expected_key" '$4 == key {print NF "|" $5}' "$context_directory/peaks.tsv" 2>/dev/null)"
    [[ "$output" == "6|$expected_threads" ]] \
      || fail "峰值表里 $expected_key 那一行应当是六列、线程数记「$expected_threads」，实际「$output」（列数|线程数）：$(cat "$scratch/threads.err")"
  done
  checked=$((checked + 1))
  output="$(awk -F '\t' '$4 == "selftest-four-columns" {print NF}' "$context_directory/peaks.tsv" 2>/dev/null)"
  [[ "$output" == 4 ]] || fail "加线程数一列之前记的四列那一行（selftest-four-columns）写回之后应当仍是四列，实际「$output」列"

  # 套在包装里（文件头「套在包装里」一段）：外层（真的包装、真的 scope）里再起一条，看它记不记账、排不排队、上限比外层大拒不拒、撞了自己的上限还是不是 250
  local inner_status inner_seconds ledger_count
  # 外层要 200M（峰值表里没有，按上限算）占着账、总上限 256M；套在里面的一条要 150M，没套的话按上限算放不下、排队等满 4 秒退 252。
  # 套着：不另记账、不排队、当场起、退 0，它的命令看到的账上只有外层那一笔
  fresh_context 256M
  checked=$((checked + 1))
  rm -f -- "$scratch/nested-inner.status" "$scratch/nested-ledger.txt"
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-nested-outer bash "$target" 200M bash -c '
      started=$SECONDS
      env RUN_WITH_MEMORY_CAP_KEY=selftest-nested-inner RUN_WITH_MEMORY_CAP_WAIT_SECONDS=4 bash "$1" 150M bash -c "ls -- \"\$1\" > \"\$2\"" list-ledger "$2" "$3" 2>"$4"
      printf "%s %s\n" "$?" "$((SECONDS - started))" > "$5"' nested-outer "$target" "$context_directory/state/ledger" "$scratch/nested-ledger.txt" \
      "$scratch/nested-inner.err" "$scratch/nested-inner.status" 2>"$scratch/nested-outer.err"; then status=0; else status=$?; fi
  inner_status="" inner_seconds=""
  [[ -f "$scratch/nested-inner.status" ]] && read -r inner_status inner_seconds < "$scratch/nested-inner.status"
  ledger_count="$(grep -c '\.json$' "$scratch/nested-ledger.txt" 2>/dev/null)"
  if [[ $status -ne 0 || "$inner_status" != 0 || "$ledger_count" != 1 || ! "$inner_seconds" =~ ^[0-2]$ ]]; then
    fail "套在外层里起的一条应当不另记账、不排队（外层要 200M 占着账、总上限 256M、它要 150M）：当场起、退 0，它的命令看到的账上只有外层那一笔，\
实际外层退 $status、它退 ${inner_status:-（没记下）}、用了 ${inner_seconds:-?} 秒、账上 ${ledger_count:-?} 笔：$(cat "$scratch/nested-inner.err" "$scratch/nested-outer.err" 2>/dev/null)"
  fi
  # 外层 100M，套在里面的一条要 200M：外层装不下它，按用法错退 251、报「比外层还大」、命令不跑
  fresh_context 256M
  checked=$((checked + 1))
  marker="$scratch/nested-larger-ran"
  rm -f -- "$scratch/nested-larger.status"
  # 里面那条的等待上限给 4 秒：弄坏开关 nestedqueues 下它照没套那样排队，不等满默认的 3600 秒
  if in_context bash "$target" 100M bash -c 'RUN_WITH_MEMORY_CAP_WAIT_SECONDS=4 bash "$1" 200M bash -c ": > \"\$1\"" touch-marker "$2" 2>"$3"; echo "$?" > "$4"' nested-outer "$target" "$marker" \
      "$scratch/nested-larger.err" "$scratch/nested-larger.status" 2>/dev/null; then status=0; else status=$?; fi
  output="$(cat "$scratch/nested-larger.status" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != "$MEMORY_CAP_UNAVAILABLE_EXIT" || -e "$marker" ]] || ! grep -q '比外层还大' "$scratch/nested-larger.err" 2>/dev/null; then
    fail "套在外层里而上限比外层大时应当按用法错退 $MEMORY_CAP_UNAVAILABLE_EXIT（外层 100M、它 200M）、报「比外层还大」、命令不跑，\
实际外层退 $status、它退 ${output:-（没记下）}、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/nested-larger.err" 2>/dev/null)"
  fi
  # 外层 300M，套在里面的一条 64M 分配 512 MiB：它在自己的 scope 里撞自己的上限、退 250，外层的 scope 不撞、外层照跑完退 0
  # （变异门禁这类调用方靠每一条自己的 scope 分 250 / 253 / 254）
  fresh_context 512M
  checked=$((checked + 1))
  rm -f -- "$scratch/nested-oom.status"
  if in_context bash "$target" 300M bash -c 'RUN_WITH_MEMORY_CAP_WAIT_SECONDS=4 bash "$1" 64M python3 -c "$2" 512 >/dev/null 2>"$3"; echo "$?" > "$4"' nested-outer "$target" "$hog_code" \
      "$scratch/nested-oom.err" "$scratch/nested-oom.status" 2>"$scratch/nested-oom-outer.err"; then status=0; else status=$?; fi
  output="$(cat "$scratch/nested-oom.status" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != "$MEMORY_CAP_HIT_EXIT" ]] || ! grep -q '撞了内存上限 64M' "$scratch/nested-oom.err" 2>/dev/null; then
    fail "套在外层里的一条撞了自己的上限应当退 $MEMORY_CAP_HIT_EXIT、报「撞了内存上限 64M」，外层照跑完退 0（外层 300M、它 64M、分配 512 MiB），\
实际外层退 $status、它退 ${output:-（没记下）}：$(cat "$scratch/nested-oom.err" "$scratch/nested-oom-outer.err" 2>/dev/null)"
  fi
  # 两个标记随环境漏进来、而这个进程不在那个 scope 里：照没套跑（记账、上限 64M 比标记里的 16M 大也不拒）
  fresh_context 256M
  checked=$((checked + 1))
  rm -f -- "$scratch/leaked-ledger.txt"
  if in_context env RUN_WITH_MEMORY_CAP_OUTER_UNIT=singlefs-memory-cap-selftest-not-this-scope RUN_WITH_MEMORY_CAP_OUTER_CAP=16M \
      bash "$target" 64M bash -c 'ls -- "$1" > "$2"' list-ledger "$context_directory/state/ledger" "$scratch/leaked-ledger.txt" 2>"$scratch/leaked.err"; then status=0; else status=$?; fi
  ledger_count="$(grep -c '\.json$' "$scratch/leaked-ledger.txt" 2>/dev/null)"
  if [[ $status -ne 0 || "$ledger_count" != 1 ]]; then
    fail "标记变量漏到外层 scope 之外时应当照没套跑（记账、上限 64M 比标记里的 16M 大也不拒、退 0），实际退 $status、账上 ${ledger_count:-?} 笔：$(cat "$scratch/leaked.err" 2>/dev/null)"
  fi

  host_lookup_cells
  peak_history_cells
  temporary_directory_cells
  broadcast_cells

  # 收尾之后不留账、不留 scope：每一例的账是空的，每个 slice 底下没有 scope
  checked=$((checked + 1))
  output=""
  for context_directory in "$scratch"/context-*; do
    if compgen -G "$context_directory/state/ledger/*.json" >/dev/null; then output+=" $context_directory 的账没清空"; fi
  done
  for slice in "${selftest_slices[@]}"; do
    if compgen -G "/sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/$slice/*.scope" >/dev/null; then output+=" $slice 底下还有 scope"; fi
  done
  [[ -z "$output" ]] || fail "跑完应当不留账、不留 scope，实际：$output"

  # --status 的本地模型服务那一项：私有配置里配了 OUTSIDE_SERVICE_CGROUP 就读那里的 memory.stat；文件不在、没有这个键都报「没配」、不读、退出码照旧。
  # cgroup 根指到这一例自己造的假目录，私有配置写在 scratch 里；失败信息不贴 --status 的整段输出（改前那一份会在里面打出单元名）
  fresh_context 256M
  local fake_cgroup_root="$scratch/fake-cgroup" private_case private_config expected_text
  mkdir -p "$fake_cgroup_root/system.slice/selftest-outside.service" "$fake_cgroup_root/user.slice"
  printf 'anon %s\nshmem %s\nfile %s\n' $((3 * MEBIBYTE)) $((2 * MEBIBYTE)) $((900 * MEBIBYTE)) > "$fake_cgroup_root/system.slice/selftest-outside.service/memory.stat"
  printf 'anon %s\n' $((7 * MEBIBYTE)) > "$fake_cgroup_root/user.slice/memory.stat"
  printf '%s\n' "# 自证用的私有配置" "OTHER_KEY=system.slice/other.service" "OUTSIDE_SERVICE_CGROUP=system.slice/selftest-outside.service" > "$scratch/private-set.env"
  printf '%s\n' "# 自证用的私有配置，没有 OUTSIDE_SERVICE_CGROUP" "OTHER_KEY=system.slice/selftest-outside.service" > "$scratch/private-keyless.env"
  for private_case in "set|$scratch/private-set.env|本地模型服务的 anon + shmem 5 MiB、" \
      "absent|$scratch/private-absent.env|本地模型服务的 anon + shmem 没配（" \
      "keyless|$scratch/private-keyless.env|本地模型服务的 anon + shmem 没配（"; do
    IFS='|' read -r private_case private_config expected_text <<< "$private_case"
    checked=$((checked + 1))
    if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$fake_cgroup_root" RUN_WITH_MEMORY_CAP_PRIVATE_CONFIG="$private_config" bash "$target" --status \
        >"$scratch/private-$private_case.out" 2>&1; then status=0; else status=$?; fi
    if [[ $status -ne 0 ]] || ! grep -qF -- "$expected_text" "$scratch/private-$private_case.out" \
        || { [[ "$private_case" != set ]] && grep -qF -- "5 MiB、" "$scratch/private-$private_case.out"; }; then
      fail "--status 私有配置 $private_case 那一例应当退 0、「slice 外面」那一行含「$expected_text」$([[ "$private_case" != set ]] && echo '、不含假 cgroup 里的 5 MiB')，\
实际退 $status、$(grep -qF -- "$expected_text" "$scratch/private-$private_case.out" && echo '含' || echo '不含')那一句（整段输出不贴：改前那一份会在里面打出单元名）"
    fi
  done

  # 用法错退 $MEMORY_CAP_UNAVAILABLE_EXIT（「命令一行都没跑」那一类），不落在 0–249 里与命令自己的退出码混在一起
  checked=$((checked + 1))
  if bash "$target" abc true 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "上限写成 abc 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  checked=$((checked + 1))
  if bash "$target" 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "没给命令应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  # 上限不带单位（「16」）：systemd 当 16 字节；按用法错拒绝，不起 scope、命令不跑、报「单位必写」
  fresh_context 256M
  checked=$((checked + 1))
  marker="$scratch/ran-unitless"
  if in_context bash "$target" 16 bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/unitless.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q '单位必写' "$scratch/unitless.err"; then
    fail "上限写成不带单位的 16 时应当按用法错退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「单位必写」、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unitless.err")"
  fi
  # slice 名不是 singlefs 开头（别人的 slice）：退 $MEMORY_CAP_UNAVAILABLE_EXIT，不设上限、不起 scope。改前那一份会真去设，名字取一个不碍事的，收尾照样停掉、还原
  checked=$((checked + 1))
  foreign_slice="notsinglefs_selftest_$$.slice"
  selftest_slices+=("$foreign_slice")
  if env RUN_WITH_MEMORY_CAP_SLICE="$foreign_slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/foreign-state" \
      RUN_WITH_MEMORY_CAP_PEAKS="$scratch/foreign-peaks.tsv" bash "$target" --check 64M 2>"$scratch/foreign.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT ]] || ! grep -q 'singlefs 开头' "$scratch/foreign.err"; then
    fail "slice 名写成不是 singlefs 开头的 $foreign_slice 时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「singlefs 开头」，实际退 $status：$(cat "$scratch/foreign.err")"
  fi

  for slice in "${selftest_slices[@]}"; do
    systemctl --user stop "$slice" >/dev/null 2>&1
    systemctl --user revert "$slice" >/dev/null 2>&1
  done
  rm -rf -- "${scratch:?}"
  if ((failures)); then
    echo "  ✗ run-with-memory-cap.sh 自检 $failures 处不对（查了 $checked 项；测的是 $target）"
    echo "  → 怎么办：照上面逐条改 run_capped / ensure_slice / ensure_slice_high / watch_over_booking / booked_oom_score_adjust / scope_result / admission / check_size_syntax / reject_usage / command_threads / private_setting / cap_judged_against_the_recorded_peak / gpu_memory_verdict（取键与读账在 memory_peaks_lib.py）；RUN_WITH_MEMORY_CAP_BREAK 设着、或 RUN_WITH_MEMORY_CAP_SELFTEST_TARGET 指着改前那一份的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ run-with-memory-cap.sh 自检通过（查了 $checked 项：平常跑设了弄坏开关退 2 并点名只在自证里认而带着自证的标记照常判、--check 起得来、64M 上限里分配 512 MiB 的探针与它当孙进程时都判撞顶退 $MEMORY_CAP_HIT_EXIT、\
上限之内 kill -9 原样退 137 且 stderr 不多一行、退出码 0 / 7 / 101 原样传回、上限超过实测峰值 2 倍照常跑、auto 取实测峰值 1.5 倍、auto 而表里没有取保守默认值 8G、\
同一个会话已有一条在跑时第二条照常放行、从会话里起的一条放不下时排队等前一条跑完、\
余量不同的并发调用者不拒（先到的读回是后到的值照读回的走、slice 上有活着的使用者时后到的照已有的总上限走且不改 slice 的 MemoryMax、排队也按已有的算，这几例都打「照 slice … 已有的总上限」一行；slice 上有别的余量的调用者活着时 --check 照已有的试、不改 slice、不打这一行、退 0）、\
命令退非 0 而起止之间内核报了显存不够退 $GPU_MEMORY_HIT_EXIT 且按起止给日志窗口、读不到内核日志报没查成、别的 NVRM 行不算、命令退 0 不读日志、撞了上限又报了显存不够照旧退 $MEMORY_CAP_HIT_EXIT 且另报一行、\
scope 里 memory.max 与 memory.swap.max 设上了、scope 挂在 slice 底下且 slice 的总上限设上了、\
slice 设不上与余量比整机还大时退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、D-Bus 连不上时跑命令与 --check 都退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、systemd-run 不认上限时 --check 退 $MEMORY_CAP_UNAVAILABLE_EXIT、\
两条同时来与前一条还没涨到量时都一条一条跑、峰值表里没有而上限比预测小的按上限算排不上退 $MEMORY_ADMISSION_REFUSED_EXIT、峰值表里没有而写 32G 的按 host-resources.py 的预测 8G 记账、与按上限 24G 记账的一条在 40G 里同时跑且 scope 上限照旧 32G、要的量比总上限大立刻退 $MEMORY_ADMISSION_REFUSED_EXIT、\
账上没有在跑的而要的量加回收不掉的用量超总上限时当场退 $MEMORY_ADMISSION_REFUSED_EXIT 且出路写放得下的最多多少、账上有一条在跑而要的量加不属于任何 scope 的常驻用量超总上限时同样当场退 $MEMORY_ADMISSION_REFUSED_EXIT、\
常驻是在跑那一条的 scope 占着的则照旧排队、峰值表里记着 1.8G 的写 24G 上限按 1.8G 记账、与按 24G 记账的一条在 40G 里同时跑、\
峰值记进表之后两条按记下的量一起跑、scope 里一直收 TERM 时峰值照记（TERM 只在这一例自己的 slice 底下自己开的 scope 里发）、被总上限挤掉退 $SLICE_TOTAL_HIT_EXIT、\
撞过顶的键按上限算放不下当场退 $MEMORY_ADMISSION_REFUSED_EXIT、设了 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP 的按上限排队等常驻的跑完、按峰值表放进来涨过总上限的先挨杀且键记成撞过顶、重跑按上限排队、两条都按峰值表放进来时设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的先挨杀、\
盯账（单进程超账、同一条 scope 里 8 个进程分摊超账、一口气拿满三种都只停超账那一条、退 $SLICE_TOTAL_HIT_EXIT、键记成撞过顶、写 memory-squeezed 且 stopped_by=over-booking-watch、内核不 OOM，\
slice 的 memory.high 是总上限的 95%，两条都不超账而合计压过触发线时判到了也谁都不停）、不盯账时内核按记账算的 oom_score_adj 杀超账那一条、被内核挤掉而没超出记账的不记撞过顶、\
超过限时退 $TIME_LIMIT_HIT_EXIT、死账不挡路、\
套在外层里起的一条不另记账不排队、套在外层里上限比外层大退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、套在外层里撞了自己的上限退 $MEMORY_CAP_HIT_EXIT 而外层照跑完、标记变量漏到外层之外照没套跑、\
排队中给包装本身发 TERM / INT / KILL / HUP 时排队那一步 5 秒内跟着退、调用方管道读到 EOF、包装退 143 / 130 / 137 / 129、命令没跑、账上与 --status 里没有这一条、\
起了 scope 之后给包装发 TERM / INT 时先 systemctl --user stop 自己开的 scope（标记与账还在的时候）、scope 里的命令随之停、包装退 143 / 130、标记账与 scope 都不留、\
判完放得下而记账之前包装被 KILL 并回收时不记账且命令没跑、账上起始时刻是 null 的死账不挡路且被删、起始时刻是字符串而包装活着的那一笔不被当成死账、跑完不留账不留 scope、\
峰值表的线程数一列（经 capped.sh N 跑、经 capped.sh default 跑、环境里线程变量相同、不同、都没设五种各记对）且加这一列之前的四列行照原样留四列、\
--status 私有配置里配了本地模型服务的 cgroup 就读它、文件不在与没有这个键都报没配且退 0、\
上限写错、上限不带单位、没给命令与 slice 名不是 singlefs 开头都按用法错退 $MEMORY_CAP_UNAVAILABLE_EXIT、不带单位的那一例命令没跑、\
峰值表近几次（撞顶之后连跑 5 次干净的都按上限记账、第 6 次才按近 5 次里最大的，峰值一路变小时记账取最大的，四列五列旧行照读，派发器的 predict 在同一张表上取同一个数）、\
峰值涨了（记账取自峰值表、实测是它 1.5 倍以上的打一行并写 memory-peak-grew，不到 1.5 倍的与按预测记账的不打）、退 $MEMORY_CAP_HIT_EXIT 与 $SLICE_TOTAL_HIT_EXIT 各写 memory-own-cap 与 memory-squeezed、\
设了 RUN_WITH_MEMORY_CAP_OOM_FIRST 的撞顶与挨挤不写这两种而退出码与撞顶标记照旧、告警跟着峰值表走（峰值表指到默认之外的写它旁边那一份而真记录不多一行、不设与经符号链接指到默认的写真记录、显式给了告警记录的照显式的写）、\
排队时去找别的机器（第二台放得下时排队开始与退 $MEMORY_ADMISSION_REFUSED_EXIT 时各打一行改去第二台的命令并写告警记录、放不下与连不上与超过时限各打一行、几条一起排队 60 秒只查一次、在查的那一趟不拖慢准入）、\
这一条命令自己的临时目录（命令正常退、退非 0、被 kill -9、撞了上限、超过限时五种结局之后都删掉、设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 留着并打出路径、调用方给了 TMPDIR 照它的且不删）、\
资源释放广播（命令跑过退 3 与起了 scope 之后被 TERM 停各发一条 memory-released、排不上不发、套在外层里不发、那份库出事时退出码照旧且 stderr 里没有它的行、排队中本机自己的内存放出不另查、第二台那一趟跑完当场再查且放得下写 phase=queued、几条一起排队一个事件只查一次、叫醒那一趟抛异常与 follow 出事各只打一行））"
}

# slice 名只许 singlefs 开头的 .slice（文件头「发信号的地方」一段）
[[ "$SLICE" =~ ^singlefs[-_A-Za-z0-9]*\.slice$ ]] \
  || reject_usage "slice 名 RUN_WITH_MEMORY_CAP_SLICE 要写成 singlefs 开头的 .slice，收到的是「$SLICE」" \
       "不设它（默认 singlefs-heavy.slice），或写成 singlefs_<用途>.slice；别的 slice 里是别人的进程，给它设上限、停它就打到别人"
case "${1:-}" in
  --selftest)
    run_selftest
    exit 0 ;;
  --status)
    PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-$(default_peak_table)}"
    show_status
    exit $? ;;
  --check)
    check_size_syntax "${2:-}" "内存上限"
    check_size_syntax "$RESERVE" "余量 RUN_WITH_MEMORY_CAP_RESERVE "
    [[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
    PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-$(default_peak_table)}"
    run_capped "$2" check true
    exit $? ;;
  "")
    reject_usage "没给上限与命令" "写成 run-with-memory-cap.sh <上限> <命令> [参数…]，上限例 16G" ;;
esac

cap="$1"
shift
[[ $# -gt 0 ]] || reject_usage "没给要在上限里跑的命令" "写成 run-with-memory-cap.sh $cap <命令> [参数…]"
PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-$(default_peak_table)}"
cap="$(cap_judged_against_the_recorded_peak "$cap" "$@")" || exit $?
check_size_syntax "$cap" "内存上限"
size_in_bytes "$cap" >/dev/null || reject_usage "内存上限 $cap 太大，换成字节超过 64 位" "写成真要的量，例 16G"
check_size_syntax "$RESERVE" "余量 RUN_WITH_MEMORY_CAP_RESERVE "
[[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
[[ -z "$TIME_LIMIT_SECONDS" || "$TIME_LIMIT_SECONDS" =~ ^[1-9][0-9]*$ ]] \
  || reject_usage "限时 RUN_WITH_MEMORY_CAP_TIME_LIMIT 要写成正整数秒，收到的是「$TIME_LIMIT_SECONDS」" "写成例 1800；不限时就别设"
[[ "$WAIT_SECONDS" =~ ^[0-9]+$ ]] || reject_usage "等待上限 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 要写成非负整数秒，收到的是「$WAIT_SECONDS」" "写成例 3600"
[[ $# -gt 0 ]] || reject_usage "没给要在上限里跑的命令" "写成 run-with-memory-cap.sh $cap <命令> [参数…]"
PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-$(default_peak_table)}"
# 套在外层包装里（文件头「套在包装里」一段）：不另记账、不排队；上限比外层大就按用法错拒
if outer_cap="$(outer_scope_cap)"; then
  NESTED_IN_OUTER_SCOPE=1
  if [[ "$BROKEN_JUDGEMENT" != "nestedlargercap" ]] && (( $(size_in_bytes "$cap") > $(size_in_bytes "$outer_cap") )); then
    reject_usage "这一条套在外层包装的 scope $RUN_WITH_MEMORY_CAP_OUTER_UNIT（上限 $outer_cap）里，它的上限 $cap 比外层还大：外层装不下它，命令一行都没跑" \
      "把这一条的上限写到 $outer_cap 以内（调用方脚本的上限选项或变量，例 --memory-cap $outer_cap；外层给了多少读 RUN_WITH_MEMORY_CAP_OUTER_CAP）；真要这么多就把外层那一条的上限调大，或不套在外层里跑"
  fi
fi
# 这一条命令自己的临时目录：先挂 EXIT trap 再建，report_unavailable、stop_queued_admission、stop_own_scope_then_exit 里的 exit 都走得到它；
# 同一个 trap 里接着发资源释放广播（on_wrapper_exit，文件头「资源释放广播」）
trap on_wrapper_exit EXIT
begin_own_temporary_directory
run_capped "$cap" run "$@"
exit $?
