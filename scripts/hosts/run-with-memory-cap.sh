#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 起不了 user systemd scope 时它自己退 251，调用方（task-server 的尝试、crates-mutation-replay）按这个码分流；这里拒绝退 78 会让它们认错结局
# 给一条命令定内存上限的内核层兜底，两层（排队、记账、挑机器、定额度都归仓根 task-server/，这里一样都不做；records/2026-10-30-编译绕过内存总上限.md「收尾批」）：
#   ① 这一条的上限：放进 systemd 的临时 scope（MemoryMax=<上限>、MemorySwapMax=0、OOMPolicy=stop），撞上限只杀这个 scope 里的进程；
#   ② 总量兜底：每个 scope 都挂进同一个 slice（默认 singlefs-heavy.slice），slice 的 MemoryMax 是总上限（下面「总上限」一段），
#      几条合起来撞顶也只在 slice 里杀，杀不到 slice 外面的本地模型服务与 Claude Code 会话。slice 设不上就报错退出，不退回无总上限。
#   另记峰值表（每条命令近几次的内存峰值），见下面「峰值表」一段。
# 起法：task-server 每一次尝试经它起（task-server/task_server.py 文件头「起任务」：本机 bash scripts/hosts/run-with-memory-cap.sh <上限> bash <仓根>/scripts/hosts/capped.sh <线程数> <命令…>，
#   第二台经 multi-host-run.sh --peer 在共享的同一棵树里（同一个绝对路径）经仓里这一份起）。人手与 agent 不直接起它：一律 task-server/task submit -- <命令…>，上限由服务按记录给、撞了放大重排。
#
#   run-with-memory-cap.sh <上限> <命令> [参数…]   在上限里跑那条命令；退出码见下
#   run-with-memory-cap.sh --check <上限>          只试 slice 的总上限设不设得上、带同样上限的 scope 起不起得来（在 slice 里跑 true）；起不来退 251
#   run-with-memory-cap.sh --status                现量：slice 的总上限与它从哪来、slice 里回收不掉的用量与每个 scope、整机内存、slice 外面的用量
#   run-with-memory-cap.sh --selftest              自证（每一例在自己的临时 slice 里，总上限几百 M，吃不满机器）
#
# 上限照 systemd 的 MemoryMax 写法：正整数加 K / M / G / T（1024 进制），例 16G、512M；单位必写，不带单位的「16」在 systemd 那里是 16 字节，这里按用法错拒绝。
# 退出码：
#   0–244、246–249  那条命令自己的退出码（cargo 只退 0、101、126、127 与 128 + 信号号，碰不到 245 与 250–254）；包装自己的结局一个都不落在这一段，用法错也不落（退 251）；
#                   只有一处例外：平常跑设了只给自证用的 RUN_WITH_MEMORY_CAP_BREAK，包装退 2、命令一行都没跑（文件头「弄坏开关只在自证里认」一段）；
#   245    显存不够：命令退了非 0，而它起止之间的内核日志里有显卡驱动报的显存不够（NVRM … NV_ERR_NO_MEMORY，文件头「显存不够」一段）；那一次的输出不算结果。
#          命令退 0 的不查、照退 0；内核日志查不成的照退命令自己的码，另报一行「显存 OOM 没查成」；
#   250    撞了这一条自己的上限：scope 的 Result 是 oom-kill，scope 自己的 memory.events 里 oom 不为 0（或读不到）。
#          只认 Result、不认退出码：被杀的是 cargo 起的测试二进制时 cargo 退 101 或被停时退 143，被杀的是命令本身时退 137，外面 kill -9 同样退 137；
#   251    用法错（上限、总上限不是 正整数加 K / M / G / T 的写法或没写单位，限时不是整数秒，slice 名不是 singlefs 开头，没给命令，
#          环境里没有服务给的 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER：文件头「只认服务直接起的那一层」一段），总上限定不下来（文件头「总上限」一段）、slice 的总上限设不上（读回不是设的那个数）、
#          systemd-run 起不来（没有用户级 systemd、D-Bus 连不上、上限写法 systemd 不认）、峰值文件的目录建不了，或查不到 scope 的结局：
#          那条命令一行都没跑（或结局判不了），不退回无上限、无总上限去跑——那正是要防的事；
#   253    超过限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT）：scope 的 Result 是 timeout，systemd 停掉了 scope 里的进程；
#   254    被总上限挤掉：scope 的 Result 是 oom-kill，而 scope 自己的 oom 计数是 0、oom_kill 不为 0——slice 撞了顶，内核在 slice 里挑了这一条杀；
#          它的结果不算数，重跑它（task-server 照同一上限重排）。
#   252    命令退出之后停了自己的 scope、回读没过（文件头「命令退出之后停自己的 scope」）：is-active 读到的不是 inactive 或 failed，或 cgroup.procs 里还有进程；
#          stderr 逐个列留下的进程（进程号与命令行前 120 字）。只换掉命令自己的退出码（0–244、246–249）；245、250、253、254 照原样退，留下的进程照列。
#   包装退非 0（命令自己的非 0、被信号杀的 128 + 信号号、245、250、253、254，包装被 TERM / INT 停）时，包装给命令建的临时目录里有东西就留着、stderr 打出路径；
#   退 0 照删（文件头「这一条命令自己的临时目录」一段）。退出码本身不因此变。设了现场目录的搬进现场目录，搬齐、到总时限只搬了一部分、搬失败留在原处，
#   三种都不改退出码（文件头「现场目录搬多少」）。
#
# 总上限（两台都是 50G，不设余量，不得超过这里设置的 50G；records/2026-10-30-编译绕过内存总上限.md）：照下面头一样有的定，不按整机内存减余量算：
#   ① 环境里给了 RUN_WITH_MEMORY_CAP_SLICE_TOTAL（systemd 写法）：照它。只给自证与别的脚本的假世界压到几百 M 用；task-server 不设它。
#   ② 多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项：配置找法同多机配置，SINGLEFS_MULTI_HOST_CONFIG，没设取这份脚本所在那棵树的根上的 multi-host.env
#      （第二台在同一个绝对路径的共享树里读本机这一份：multi-host-run.sh 把 SINGLEFS_MULTI_HOST_CONFIG 设成它，那份文件头 ⑤「带到第二台的环境」）。
#      值是空白分隔的 <地址>=<总上限>（模板 multi-host.env.example「按台的三个键」）；这一台照同一份配置的 HOSTS 与这一台网卡上的地址认
#      （scripts/hosts/peer-host-lib.sh 的 peer_host_resolve_addresses，同 scripts/hosts/multi_host_addresses.py 的 resolve），取这一台那一项（configured_slice_cap）。
#      写了这个键而认不出这一台、没有这一台那一项、或那一项写法不对：退 251，不退回下一样。
#   ③ 配置读不到或没写这个键，slice 此刻已有正整数的 MemoryMax（真机上是之前照 ② 设的）：照它走、不改 slice；只为不带这个键的假世界（别的脚本自证里自己写的多机配置）。
#   三样都没有：退 251，命令一行都没跑，出路写在多机配置里加 SLICE_MEMORY_CAP_BY_HOST。
#   定下来之后与 slice 此刻的 MemoryMax 不同就设成它（systemctl --user set-property --runtime），设完读回；读回不是这个数退 251。
#   slice 的 memory.high（软上限）包装自己不设；别的版本留下的（用户级 systemd 的 runtime drop-in，slice 停了再起照样带上）每一次起都清回 max（ensure_slice 的 clear_leftover_memory_high）：
#     总上限设好、slice 的 cgroup 目录定下来之后，用 bash 的内建 read 读它的 memory.high（不多起进程）；是 max 照跑；读不到（slice 的 cgroup 目录不在）当没有、照跑；
#     别的值就 systemctl --user set-property --runtime <slice> MemoryHigh=infinity，再读一次 memory.high 的内容：是 max 往 stderr 打一行「清回 max」、照跑；
#     set-property 失败或读回不是 max 退 251、命令一行都没跑，出路写手清的命令与查 drop-in 的地方。软上限撞线时进程卡在 mem_cgroup_handle_over_high（D 状态）、不 OOM，所以不带着它跑。
#     --check 同样清；--status 不清。
#   --status 打总上限从哪来（上面哪一样）与 slice 此刻读回的 memory.max。本地模型服务的 cgroup 不写进仓（单元名是本机的私有信息）：--status 从仓外的私有配置读，
#   文件是 RUN_WITH_MEMORY_CAP_PRIVATE_CONFIG，默认 ${XDG_CONFIG_HOME:-~/.config}/singlefs/run-with-memory-cap.env；一行一条「键=值」，# 起头的行与空行不看，不 source。
#   读的键是 OUTSIDE_SERVICE_CGROUP=<它的 cgroup，相对 cgroup 根，形如 system.slice/<单元名>.service>，读那里 memory.stat 的 anon + shmem。
#   文件不在或没有这个键，--status 里这一项报「没配」、其余照算、退出码不受影响；配了而读不到 memory.stat 报「读不到」。--status 不打印配的值。
#
# 起了 scope 之后包装被 TERM / INT 停（欠账 C801；调用方是 task-server 停尝试单元、crash-amplification-lib.sh 这类把 TERM 只发给包装一个进程号的）：
#   先 systemctl --user stop 自己开的那个 scope 单元（$unit.scope），scope 里的命令随之收到 TERM、停下；再删标记与峰值文件，退 143 / 130（stop_own_scope_then_exit）。
#   起 scope 的 systemd-run 放后台、用 wait 等，所以 trap 当场就跑，不等命令退出；命令读的标准输入照旧是调用方的（<&0），INT 照旧不被忽略（env --default-signal=INT；调用方进来时就忽略 INT 的照旧忽略）。
#   scope 还没建出来（systemd-run 还在起，stop 找不到单元）：改给 systemd-run 那一个进程发 TERM。这一次的峰值不记（量到的是半截）。
# 命令退出之后停自己的 scope（命令退出之后 scope 里留着进程就一直活着；records/2026-10-31-task-server巡检.md）：外壳退出之后（不论命令退几、
#   --check 那一次也算），读一次 scope 的 cgroup.procs（目录是外壳写进起跑标记的那一条），还有进程的往 stderr 打一行「还留着 N 个进程」并逐个列；再一律
#   systemctl --user stop 自己的 $unit.scope， .claude/rules/command-safety-by-review.md 回读：is-active 的输出是 inactive 或 failed、cgroup.procs 的内容是空的（目录已不在算空）才算停了；
#   回读没过退 252（文件头「退出码」），stderr 列留下的进程（stop_scope_after_command）。查结局（Result、读早了的 oom-kill）在停之前做完，停不改结局。
#   弄坏开关 scope-left-after-exit 下不停（scope 里还有进程就一直活着）。
# multi-host-run.sh 给这一层的两样（它文件头 ⑤⑥；读了就 unset，不传进命令；写不进说一句、照跑）：
#   RUN_WITH_MEMORY_CAP_UNIT_OUTPUT=<文件>，起 scope 之前把这一条的 scope 单元名（不带 .scope）写进去（先写 <文件>.partial 再改名），第二台清理时照它先停这一个 scope。
#   RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD=<文件>，建了这一条命令自己的临时目录（下面「这一条命令自己的临时目录」）就把它的全路径写进去（同样先写 <文件>.partial 再改名），
#   包装退出时 EXIT trap 收拾完那一份之后删掉这份文件：文件还在就是包装没走到 EXIT（被 KILL / HUP），第二台清理时照它删留下的那一份。
#   调用方给了 TMPDIR 的不写（那一份归调用方）。弄坏开关 temporary-record-unwritten 下不写，temporary-record-kept-at-exit 下退出时不删。
#
# 撞顶的 scope 结局读早了（records/2026-10-29-subagent拆分提案.md 第一百三十七节 ⑧）：命令退非 0、scope 自己的 memory.events 里 oom_kill 不为 0，而头一次查到的 Result 不是 oom-kill，
#   再查至多 SCOPE_RESULT_LATE_ATTEMPTS 次（每次隔 0.1 秒）；查到 oom-kill 照撞顶办，另报一行「Result 读早了读到 …」；一直不是就按内核的计数判成 oom-kill，另报一行「Result 一直是 …」。
#   两种都不报成「被外面停了」。
# failed 而 Result 读到 success（同一节的另一种读错）：用户级 systemd 重载过配置（别的趟自证收尾的 systemctl --user revert 会触发 daemon-reload），failed 的 scope 的 Result
#   被清成 success；到这个 scope 的 journal 里找 Failed with result '<原因>' 那一行，取得到照它办（timeout 退 253、oom-kill 照上面），取不到写明「原因取不到」、照读到的 success 办。
#
# 峰值表（RUN_WITH_MEMORY_CAP_PEAKS，默认共享数据根下的 memory-peaks.tsv：memory_peaks_lib.py 的 default_peak_table，经 .claude/hooks/lib_work_root.py 的 shared_data_path 取，
#   两台读写同一份、不经 ~ 取；取不到这一趟的峰值不记、说一句，命令的退出码照旧）：一行一条命令在一台上的实测，文件头的 # 行写口径，
#   由这个脚本建、写（写在表旁 <峰值表>.lock 那把锁里——两台都写这一份，锁跟着表在共享根下——先写临时文件再改名；按 MiB 向上取整，与表里记的相同就不写）；
#   读它的见 memory_peaks_lib.py 文件头「峰值表的口径与列」。
#   口径：回收不掉的用量的峰值，不含页缓存。回收不掉的用量 = scope 的 memory.current 减 memory.stat 的 active_file、inactive_file（memory_peaks_lib.py 的 unreclaimable_bytes）。
#   命令跑的过程中包装另起一个采样子进程（sample_unreclaimable_peak，python，在 scope 外、包装的进程树里），每 0.1 秒（memory_peaks_lib.py 的 PEAK_SAMPLE_SECONDS）读一次 scope 的这两份文件，
#   最大值写进峰值文件旁边的 <峰值文件>.unreclaimable；scope 的 cgroup 目录由外壳起命令之前写进起跑标记（外壳读自己的 /proc/self/cgroup）。外壳在命令退出之后再按同一算法采一次
#   （只用 bash 的内建命令），两者取大的记进表。一次都没采到的不记。
#   采样子进程不拖延收尾：命令结束、超时、撞顶、被停（包装收到 TERM / INT）四种收尾，包装都在退出之前只给它的进程号发 TERM、等它退（stop_peak_sampler）；
#   包装被 KILL 时它看父进程换了自己退、删掉 <峰值文件>.unreclaimable。
#   外壳不 exec 命令、留下来读，是因为 scope 一收尾它的 cgroup 就删了、systemd 也不留（命令退出之后 MemoryPeak=[not set]）。
#   撞顶只认真的 OOM：scope 的 Result=oom-kill 而不是被挤掉（撞了自己的上限，记成上限），或 scope 的 memory.events 里 oom_kill 大于 0（Result 没报 oom-kill、命令照常退出的，
#   也记成撞了顶、记上限）；峰值到了上限而 oom_kill 是 0 的（页缓存涨满上限）记 clean、记回收不掉的那一截。记什么只在 memory_peaks_lib.py 的 judged_measurement 一处判。
#   超时、被总上限挤掉、命令被外面的信号停（退 128 加 HUP、INT、KILL、TERM 的信号号 1、2、9、15）而 scope 没撞自己的上限的不记（量到的是半截）；
#   被外面停的另报一句「被外面停了（信号 N），这一次的峰值不记」与出路，退出码照命令自己的。
#   放在仓外的共享数据根下：是运行状态不是仓的内容；数只对量它的那一台成立，所以每行记是哪一台（第七列），读的时候只认本台的行。
#   键：RUN_WITH_MEMORY_CAP_KEY 给了就用它；不给就把命令的各个词按 memory_peaks_lib.py 的 command_key 取（剥 capped.sh N、本机路径换成记号、测试二进制名里的哈希去掉）。
#   线程数（第五列）：命令里经 capped.sh <N> 跑的记 N，经 capped.sh default 跑的记 heavy_thread_default 的值（SINGLEFS_THREAD_CAP 设了照它，否则 .claude/heavy-thread-default 的登记值；
#   取不到记「判不了」）；否则看包装自己环境里 capped.sh 设的那几个线程变量（名单取 capped.sh --print）：设了的都相同记那个数，不同就逐个记「名=值」，一个都没设记「未设（整机 N 核）」。
#   近几次（第六列）：这个键最近 RECENT_MEASUREMENTS（memory_peaks_lib.py，5）次记进表的实测，新的在前，逗号隔开，每次写成「<峰值字节>:hit:unreclaimable」或「<峰值字节>:clean:unreclaimable」；
#   前三列是最近那一次。哪一台（第七列）：这一台在多机配置 HOSTS 里的地址（memory_peaks_lib.py 的 this_host，照 scripts/hosts/multi_host_addresses.py 认）；取不到这一趟的峰值不记、说一句。
#   写的时候只换本台这个键那一行，另一台的行原样写回。表里的行只认这一版的七列：不是的（列数不对、第六列写不对、第七列空），读表抛 memory_peaks_lib.py 的 TableRowRefused、点名行号与要几列，记峰值的这一趟说一句、命令的退出码照旧；
#   出路是删掉那一行或把整张表删了，下次跑会重新记。
#
# 显存不够（运行时认显存 OOM，records/2026-09-16-subagent拆分提案.md 第五十五、五十六节）：命令退了非 0、或撞了上限、或被总上限挤掉时，
#   包装读 journalctl -k 里它起止之间（起跑前 1 秒到退出后 GPU_JOURNAL_SLACK_SECONDS 秒，--since / --until 按 epoch 秒给）的行，
#   同一行既有 NVRM 又有 NV_ERR_NO_MEMORY（显卡驱动报的显存不够）就报「显存不够」：退非 0 的那种退 245；撞了上限、被挤掉的照旧退 250 / 254，另报一行。
#   驱动的这一行不带进程号，同一段时间里别的进程撞了显存也算进来：宁可把这一条的结果当不算数、重跑，不把显存不够当成测试红。
#   读不到内核日志（没有 journalctl、权限不够、出错、超时、连最近一条内核消息都读不出）报「显存 OOM 没查成」，不当成没撞，退出码照命令自己的。
#   命令退 0 的不查。只给自证换 journalctl：RUN_WITH_MEMORY_CAP_JOURNALCTL=<假的那一份>（传进命令之前清掉）。
#
# 限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT，秒；不设不限）：给 scope 设 RuntimeMaxSec，从起跑算；到点 systemd 给 scope 里每个进程发 TERM，
#   RUN_WITH_MEMORY_CAP_KILL_GRACE（默认 30）秒还不退再发 KILL，退 253。RUN_WITH_MEMORY_CAP_KEY、_TIME_LIMIT、_JOURNALCTL 只管这一条，传进命令之前清掉。
# 这一条命令自己的临时目录（自己用完自己删；records/2026-09-16-subagent拆分提案.md 第九十二节末尾）：
#   调用方没设 TMPDIR（或设成空串）时，起命令之前在 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT（默认 /tmp）下 mktemp -d 一个 singlefs-run-with-memory-cap-<包装进程号>-XXXXXX，
#   export 成 TMPDIR 交给命令；包装退出时都走 EXIT trap，照包装的退出码分两样（.claude/singlefs-ai-sop/rules/command-safety.md「测试镜像一律放临时目录」）：
#     退 0：整个删掉，删完回读还在就往 stderr 报一行、退出码照旧。测试进程被 SIGKILL 时 Drop 守卫不走，留在它 TMPDIR 里的东西由这一步删。
#     退非 0（命令自己的非 0、被信号杀、245、250、253、254、起了 scope 之后包装被 TERM / INT 停）而那一份里有东西：留着看现场，里面放一份标记 KEPT_TEMPORARY_MARKER
#       （一行：包装的退出码与包装进程号），stderr 打「run-with-memory-cap: 包装退 <码>（…）：这一条命令的临时目录留着看现场：<路径>」与一行出路（看完自己删）。
#     退非 0 而那一份是空的（命令没起：用法错退 251，或命令一个临时文件都没写）：照删，没有现场可看。
#   留着的不越积越多：同一个 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 下带标记的只留最近 KEPT_TEMPORARY_DIRECTORIES_LIMIT（3）个（按标记的修改时刻），
#     更早的由之后留现场的那一条包装删掉；只删带标记的，别的包装正在用的（没有标记）不碰。
#   SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1（名字与含义同 crates/singlefs-verification 的 KEEP_TEMPORARY_FILES_ENVIRONMENT_VARIABLE：只认 1）时不删，往 stderr 打一行
#   「run-with-memory-cap: SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：这一条命令的临时目录留着看现场：<路径>」，看完自己删。
#   调用方设了 TMPDIR 的照它的：不另建、不删（那一份归调用方）。
#   --check、--status、--selftest 不起命令，不建。包装自己被 KILL / HUP 时 EXIT trap 不走，那一份留在 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 下（名字带包装进程号）；
#   第二台那一层与本机一样建在默认的 /tmp 下（命令拿到的 TMPDIR 短，测试在它下面绑得上 Unix 套接字）；multi-host-run.sh 另给 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD，
#   它的 ⑥ 照那份文件删留下的那一份（上面「multi-host-run.sh 给这一层的两样」；它文件头 ⑤「这一条命令自己的临时目录」）。
# task-server 给这一层的两样（task-server/task_server_lib.py 文件头「现场」与「资源记录」的退出时峰值；包装开头读了就 unset，不传进命令）：
#   RUN_WITH_MEMORY_CAP_PEAK_OUTPUT=<文件>  这一条回收不掉的用量的峰值（文件头「峰值表」一段：扣了页缓存，采样子进程每 0.1 秒采的与外壳收尾时采的取大的，与峰值表记的同一个数），是正整数就写进这份文件
#     （先写 <文件>.partial 再改名）：调用方一轮采样之间冒出来的峰值在这里，调用方与按轮采的取大的记（本机给 attempt_runner.py 读，第二台由 multi-host-run.sh 读了打回本机；撞上限的那一次同样写）。
#     都没采到不写；--check 不写；起了 scope 之后包装被 TERM / INT 停的不写（量到的是半截）。弄坏开关 peak-output-with-page-cache 下写 scope 的 memory.peak（含页缓存）。
#   RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY=<目录>  这一次尝试的现场目录（共享工作根下，两台同一个绝对路径；这一层不建它的上一层）：上面「退非 0 而那一份里有东西」
#     与设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 这两种要留的，不留在 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 下、不放标记、不进「只留最近 3 个」的轮换：
#     搬成这个目录（下面「现场目录搬多少」），stderr 打「…搬进了这一次尝试的现场目录：<目录>（命令输出里点到 <原路径> 的路径，换成这个目录读）」与一行谁删（task-server 删，不用自己删），
#     原来那一份删掉。这个目录已在、或搬的过程失败：说一句，照没设它那样留在原处、放标记、进轮换。退出码照旧。
#   现场目录搬多少（这个目录在共享根下，第二台经 NFS 写，一个一个建文件；任务 13229 第 2 次在第二台上搬约 8.9 万个文件 600 多秒没搬完、被服务判卡住，
#     第 1 次搬了约 210 秒 mv 失败、留下 603M 半份；个数是那两次现场目录里量的条目数，秒数照日志推的）：
#     先数一遍（不跟符号链接）：普通文件、符号链接要搬；表观至少 SCENE_SPARSE_IMAGE_MIN_BYTES、占盘不到表观一半的普通文件是稀疏的大镜像（例 replay.sh 自证 truncate 出来的 e9.img），
#     不搬、只记路径与表观、占盘字节；套接字、管道、设备文件不搬、只记。要搬的个数过 SCENE_MOVE_ENTRY_LIMIT 或字节数过 SCENE_MOVE_BYTE_LIMIT 的，只搬日志
#     （后缀在 SCENE_LOG_SUFFIXES 里、每份不过 SCENE_LOG_FILE_MAX_BYTES，按路径次序搬到两个上限为止），别的只记。
#     并行与总时限（第二台往共享根逐个写小文件慢在每个文件的写等待，两个上限挡不住这种慢）：要搬的排成日志在前（后缀与大小照上一句）、别的在后，各自照路径次序；
#     目录照路径先建好，再按这个次序派给 SCENE_MOVE_WORKERS 路的线程池并行搬，各路搬完一个在同一把锁里记数、打进度行；任何一个失败照整趟失败办（下面「中途失败」）。
#     从开搬起过了 SCENE_MOVE_TIME_LIMIT_SECONDS 秒还没搬完的：停派、在搬的等它搬完，已搬的留下，没搬的在清单里记 time-limit-not-moved（排在后面的先落到这一种），
#     照常写清单、改名成这个目录，照搬成了办（原来那一份删掉、不放标记，包装的退出码照旧），stderr 打一句「现场搬到总时限 N 秒没搬完：搬了 X 个，到时限没搬的 Y 个…」。
#     每一份都写一张清单 SCENE_MANIFEST_NAME（放在现场目录里；一行一项：状态 moved / sparse-not-moved / over-limit-not-moved / special-not-moved / time-limit-not-moved、
#     种类、表观字节、占盘字节、相对路径）。
#     搬的过程往 stderr 打进度行「run-with-memory-cap: 搬现场进度：…」（开搬一行、每 SCENE_PROGRESS_EVERY_ENTRIES 个或隔 SCENE_PROGRESS_EVERY_SECONDS 秒一行、搬完或到总时限停一行）：
#     服务判卡住看日志涨不涨；结局句「现场搬了 N 个（用了 S 秒）；没搬：…」不论进度行打不打都打。先搬进同一层的临时名 <这个目录>.partial-<进程号>
#     （与这个目录在同一个文件系统），搬齐或到总时限、清单写完再改名成这个目录；
#     中途失败（读不了、写不了、被 TERM / INT 停）先停派、等在搬的搬完，再删掉那份临时名、这个目录不建，原来那一份照旧留着。
# 其余环境变量有默认，只在自证与特殊场合设：
#   RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 这一条命令自己的临时目录建在哪个目录下，默认 /tmp（上一段）
#   RUN_WITH_MEMORY_CAP_SLICE          slice 名，默认 singlefs-heavy.slice（名字里的 - 是 systemd 的层级：它挂在 singlefs.slice 底下）；只许 singlefs 开头的 .slice，别的按用法错退 251
#   RUN_WITH_MEMORY_CAP_SLICE_TOTAL    直接给总上限（systemd 写法，文件头「总上限」①）
#   RUN_WITH_MEMORY_CAP_STATE_DIR      峰值文件与锁的目录，默认 ${XDG_RUNTIME_DIR:-/run/user/<uid>}/singlefs-heavy-admission（tmpfs，重启清空）
#   RUN_WITH_MEMORY_CAP_CGROUP_ROOT    cgroup v2 挂在哪，默认 /sys/fs/cgroup
#   RUN_WITH_MEMORY_CAP_SELFTEST_TARGET       --selftest 测哪一份包装，默认自己：指到另一份包装就拿它跑这份自证，看它判错
#   RUN_WITH_MEMORY_CAP_SELFTEST_OOM_SCORE_ADJUST  scope 里的命令抬到的 oom_score_adj（默认 TASK_OOM_SCORE_ADJUST，500）：只给自证取一个比自己此刻大的数（oom_score_adj 只抬得上去、降不回来，自证经服务起时已继承了 500）
#   RUN_WITH_MEMORY_CAP_SELFTEST_LATE_ATTEMPTS  scope 结局读早了时再查 Result 的次数（默认 SCOPE_RESULT_LATE_ATTEMPTS，100）：只给自证把「一直读不到 oom-kill」那一例缩短；
#                                             带着自证的标记才认，传进命令之前清掉
#   RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLED_CGROUP  采样子进程改采这个目录（假的 cgroup 目录：memory.current、memory.stat），收尾时 memory.peak 与 oom_kill 也改读它的
#                                             memory.peak、memory.events（文件头「峰值表」一段）：只给自证造「页缓存涨满上限」「oom_kill 而 Result 没报」那几例；带着自证的标记才认，读完就清掉、不传进命令
#   RUN_WITH_MEMORY_CAP_SELFTEST_HOST  峰值表第七列（哪一台）写它、不照多机配置认这一台：只给自证（它各例的多机配置只写几个键，过不了配置判法，峰值表认不出这一台）；带着自证的标记才认，读完就清掉、不传进命令
#   RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE  包装起采样子进程时往这个文件追加「sampler <进程号> <起始时刻>」，采样子进程自己退（不是被包装停）时追加「exit command-ended」或
#                                             「exit wrapper-gone」：只给自证核「采样子进程不拖延收尾」那四例；带着自证的标记才认，读完就清掉、不传进命令
#   RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE  设了的 --selftest 挂好收尾之后只开一个真 slice（一例 --check 64M），把「pid <进程号>」「slice <名>」「scratch <目录>」写进这个文件，
#                                             再等着被停（至多 SELFTEST_INTERRUPT_PROBE_SECONDS 秒，到点退 1）：只给自证「自证开出 slice 之后被 TERM / INT 停时」那两格（文件头「自证的收尾」）；
#                                             带着自证的标记才认，读完就清掉、不传进命令
#   RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_ENTRY_LIMIT  搬现场的个数上限换成这个数（默认 SCENE_MOVE_ENTRY_LIMIT，文件头「现场目录搬多少」）：只给自证「搬现场超上限时」那一例用几十个文件造超上限；
#                                             带着自证的标记才认，读完就清掉、不传进命令
#   RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_TIME_LIMIT  搬现场的总时限换成这个数（秒，可带小数；默认 SCENE_MOVE_TIME_LIMIT_SECONDS）：只给自证「搬现场到总时限时」那一例用 1 秒造到时限；
#                                             带着自证的标记才认，读完就清掉、不传进命令
#   RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_COPY_DELAY  搬现场每个文件搬之前睡这么久（秒，可带小数；默认 0）：只给自证造慢拷贝（「搬现场 80 个每个慢 0.2 秒的文件时」「搬现场到总时限时」）；
#                                             带着自证的标记才认，读完就清掉、不传进命令
# 交给命令的两个标记：起 scope 时给命令设 RUN_WITH_MEMORY_CAP_OUTER_UNIT=<这一条的 scope 单元名>、RUN_WITH_MEMORY_CAP_OUTER_CAP=<这一条的上限>；
#   命令要知道这一条给了多少，读 RUN_WITH_MEMORY_CAP_OUTER_CAP。
# 只认服务直接起的那一层：task-server 起本机的尝试时给这一层设 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER=<任务号>.<第几次尝试>
#   （task-server/task_server_lib.py 的 ADMITTED_VARIABLE；派第二台的那一次由 multi-host-run.sh --peer 照服务给它的那一份设给第二台这一层；非空就算设了，值不打、不记）。
#   <上限> <命令> 这一种没有它就按用法错退 251、命令没跑；起 scope 时从命令的环境里摘掉它，于是任务里面的脚本再经这一份起一条当场被拒：
#   内存上限由服务定，脚本照读服务交的 TASK_SERVER_MEMORY_CAP_BYTES，不再套这一层。--check、--status、--selftest 不看它。
#
# 弄坏开关 RUN_WITH_MEMORY_CAP_BREAK=<项>（只给 --selftest 证明它会红用，一次给一个；打开时自证退 1）。逐项写开关名、弄坏哪一处、打开后自证哪几项红
# （项名是那条「✗ 自检：」失败行的句首，到「应当」为止；格名与红法是照代码读出来的，交主 agent 打开开关逐个跑一遍核）：
#   RUN_WITH_MEMORY_CAP_BREAK=nocap  不进 scope 直接跑，命令跑在调用方的 cgroup 里
#       上限那几格（「上限 64M 里分配 512 MiB 的探针」「scope 里的 memory.max」「scope 应当挂在 slice」…）与「命令跑 0.1 秒就退时，RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 那一份应当…」红
#   RUN_WITH_MEMORY_CAP_BREAK=noslice  不挂 slice
#       「scope 应当挂在 slice」「两条合起来超总上限 256M」「总上限照多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项」红
#   RUN_WITH_MEMORY_CAP_BREAK=slicefallback  slice 设不上时不挂 slice 照跑
#       「slice 的总上限设不上时」红
#   RUN_WITH_MEMORY_CAP_BREAK=fallback  systemd-run 起不来时退回无上限跑
#       「D-Bus 连不上时」红
#   RUN_WITH_MEMORY_CAP_BREAK=exitcode  按退出码 137 / 143 判撞顶，不看 scope 的 Result
#       「吃内存的是孙进程时」「上限之内被 kill -9 的命令」「命令被外面的 TERM 停」…红
#   RUN_WITH_MEMORY_CAP_BREAK=noresult  不认 scope 的 Result
#       「上限 64M 里分配 512 MiB 的探针」「两条合起来超总上限 256M」…红
#   RUN_WITH_MEMORY_CAP_BREAK=noslicehit  被内核挤掉的也报成撞了自己的上限
#       「两条合起来超总上限 256M」红
#   RUN_WITH_MEMORY_CAP_BREAK=squeezedrecorded  被内核挤掉的那一次照撞顶记进峰值表
#       「被总上限挤掉的那一次」红
#   RUN_WITH_MEMORY_CAP_BREAK=notimelimit  不设限时
#       「限时 1 秒跑 sleep 4」「命令超过限时被停之后」红
#   RUN_WITH_MEMORY_CAP_BREAK=recordkilledpeak  命令被外面的信号停也记峰值
#       「命令被外面的 TERM 停」「命令被外面的 KILL 停」红
#   RUN_WITH_MEMORY_CAP_BREAK=term-leaves-scope  起了 scope 之后包装被 TERM / INT 停时不 systemctl --user stop 自己的 scope
#       「起了 scope 之后给包装发 TERM」「… INT」红
#   RUN_WITH_MEMORY_CAP_BREAK=taskserverleaks  起 scope 时不从命令的环境里摘掉 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER：任务里面再套一层照跑
#       「已由 task-server 准入的一条」「被服务起的命令里面再调这一份」红
#   RUN_WITH_MEMORY_CAP_BREAK=threadsdefaultunset  command_threads 不认 capped.sh default
#       「峰值表里 selftest-threads-default 那一行」「… selftest-threads-default-cap 那一行」红
#   RUN_WITH_MEMORY_CAP_BREAK=slicecapconfigignored  不读多机配置的 SLICE_MEMORY_CAP_BY_HOST
#       「总上限照多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项」「--status 应当报总上限从多机配置来」红
#   RUN_WITH_MEMORY_CAP_BREAK=slicecapsyntaxlenient  SLICE_MEMORY_CAP_BY_HOST 写坏时（这一台那一项写法不对、没有这一台那一项）当没写、往下一样退
#       「多机配置里 SLICE_MEMORY_CAP_BY_HOST 这一台那一项写法不对时」「多机配置里 SLICE_MEMORY_CAP_BY_HOST 没有这一台那一项时」红
#   RUN_WITH_MEMORY_CAP_BREAK=slicecapfirstentry  SLICE_MEMORY_CAP_BY_HOST 不按这一台的地址取，一律取头一项（写死次序的样子）
#       「SLICE_MEMORY_CAP_BY_HOST 里头一项是另一台时总上限照这一台那一项」红（连带「多机配置里 SLICE_MEMORY_CAP_BY_HOST 没有这一台那一项时」：取了另一台那一项照跑）
#   RUN_WITH_MEMORY_CAP_BREAK=existingtotalrefused  配置里没有这个键时不照 slice 已有的总上限走、当场退 251
#       「多机配置里没有 SLICE_MEMORY_CAP_BY_HOST、slice 已有总上限时」红
#   RUN_WITH_MEMORY_CAP_BREAK=unboundedwithoutcap  三样都定不出总上限时不挂 slice 照跑
#       「三样都定不出总上限时」红
#   RUN_WITH_MEMORY_CAP_BREAK=hostcolumnignored  记峰值时不留另一台的行：整张表只写本台的行
#       「峰值表两台共用一份：另一台同一个键的行原样留着」红
#   RUN_WITH_MEMORY_CAP_BREAK=refusalunhandled  峰值表里有不是这一版的行时不接 TableRowRefused、回溯栈打出来
#       「峰值表里有不是这一版的行」红
#   RUN_WITH_MEMORY_CAP_BREAK=recordmemorypeak  整段跑完的照旧把 scope 的 memory.peak（含页缓存）记成峰值
#       「页缓存涨满上限」红
#   RUN_WITH_MEMORY_CAP_BREAK=hitbypeak  整段跑完的照旧按 memory.peak 到没到上限判撞顶
#       「页缓存涨满上限」「scope 的 memory.events 里 oom_kill 1 而 Result 没报 oom-kill」红
#   RUN_WITH_MEMORY_CAP_BREAK=samplerleft  包装收尾时不停采样子进程，采样子进程只在包装没了之后才自己退
#       「采样子进程不拖延收尾」四格红
#   RUN_WITH_MEMORY_CAP_BREAK=nogpuoom  命令退出之后不读内核日志
#       「命令退 3、起止之间内核报了」「撞了自己的上限、起止之间内核又报了显存不够时」红
#   RUN_WITH_MEMORY_CAP_BREAK=journalasclean  读不到内核日志当成没撞
#       「内核日志读不出来时」红
#   RUN_WITH_MEMORY_CAP_BREAK=anynvrm  只要是 NVRM 行就当显存不够
#       「起止之间只有别的 NVRM 行」红
#   RUN_WITH_MEMORY_CAP_BREAK=wholejournal  读内核日志不按命令起止给 --since / --until
#       「读内核日志应当按命令起止给」红
#   RUN_WITH_MEMORY_CAP_BREAK=gpuoomonsuccess  命令退 0 也查内核日志
#       「命令退 0 时应当照退 0、不读内核日志」红
#   RUN_WITH_MEMORY_CAP_BREAK=oomresultlag  不看 memory.events 里的 oom_kill、不为读早了的 Result 多查
#       「撞顶的 scope 被读早了」「撞顶的 scope 的 Result 一直读到 success」红
#   RUN_WITH_MEMORY_CAP_BREAK=scoperesultonly  ActiveState 是 failed 而 Result 读到 success 时不到 journal 找原因：「限时被停的 scope 落到 failed 而 Result 读到 success（重载配置之后）时」
#       「… journal 又读不了时」红
#   RUN_WITH_MEMORY_CAP_BREAK=noprivate  --status 不读私有配置
#       「--status 私有配置 set 那一例」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporaryleft  包装退出时该删的也不删
#       「命令正常退出之后，包装给它的临时目录应当删掉」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarydeletedonfailure  包装退非 0 也照删
#       退 7、kill -9、撞顶、超时四格与「判红留着的只留最近 3 个」「设了现场目录、命令退 7 而临时目录里有东西时」
#       「搬现场超上限时」「搬现场的过程」「现场里稀疏的大镜像」「搬现场中途失败时」「搬现场 80 个每个慢 0.2 秒的文件时」「搬现场到总时限时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarykeptempty  包装退非 0 而那一份是空的也留着
#       「命令退非 0 而临时目录里什么都没写时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarykeptunbounded  留现场时不删更早留着的
#       「判红留着的只留最近 3 个」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporaryshared  不给命令建它自己的临时目录
#       临时目录五格（命令拿到的 TMPDIR 不在 root 底下）与「给了 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD 时」「设了现场目录、命令退 7 而临时目录里有东西时」「设了现场目录与 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时」
#       「搬现场超上限时」「搬现场的过程」「现场里稀疏的大镜像」「搬现场中途失败时」「搬现场 80 个每个慢 0.2 秒的文件时」「搬现场到总时限时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarykeepignored  不认 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1，照删
#       「设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时」「设了现场目录与 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarycallerreplaced  调用方设了 TMPDIR 也另建一份顶掉它
#       「调用方设了 TMPDIR 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarycallerdeleted  调用方设了 TMPDIR 时照它的，退出时却连它一起删
#       「调用方设了 TMPDIR 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporary-record-unwritten  给了 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD 也不把临时目录的全路径写进去
#       「给了 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporary-record-kept-at-exit  包装退出时不删记着临时目录路径的那份文件（第二台清理会照它删掉留着看现场的那一份）
#       「给了 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=oom-score-not-raised  scope 里的命令不抬 oom_score_adj（照内核缺省 0；TASK_OOM_SCORE_ADJUST）
#       「scope 里的命令 oom_score_adj 应当是 500」红
#   RUN_WITH_MEMORY_CAP_BREAK=peak-output-unwritten  命令退出之后不往 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 写峰值（只靠调用方按轮采）
#       「命令跑 0.1 秒就退时，RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 那一份应当是回收不掉的用量的峰值」与「页缓存涨满上限的那一趟，RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 那一份应当是回收不掉的 16M」红
#   RUN_WITH_MEMORY_CAP_BREAK=scene-kept-in-temporary-root  设了 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY 也留在 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 下、进轮换
#       「设了现场目录、命令退 7 而临时目录里有东西时」「设了现场目录与 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时」
#       「搬现场超上限时」「搬现场的过程」「现场里稀疏的大镜像」「搬现场中途失败时」「搬现场 80 个每个慢 0.2 秒的文件时」「搬现场到总时限时」红
#   RUN_WITH_MEMORY_CAP_BREAK=memoryhighleft  不清 slice 上别的版本留下的 memory.high：「slice 上留着软上限 47.5G 时」「slice 原先不在、起来之后带着软上限 47.5G 时」  history-kept: 自证失败行原文，按原文匹配
#       「清软上限之后读回不是 max 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=memoryhighunverified  清软上限之后不读回、当清掉了
#       「清软上限之后读回不是 max 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=scope-left-after-exit  命令退出之后不停自己的 scope：scope 里留着进程就一直活着
#       「命令退出之后 scope 里留着一个 setsid 出去、清了环境的进程时」「跑完」两格红（后一格是收尾那一格：留着的那个 scope 收尾之后还在，任务 9486 量的）
#   RUN_WITH_MEMORY_CAP_BREAK=guard-off  平常跑设了弄坏开关也不拒（探针往下走到「没给要在上限里跑的命令」退 251、不点名）：
#       「平常跑（没有自证的标记）设了 RUN_WITH_MEMORY_CAP_BREAK」标记「空」与标记指到头一行不对的文件两格红
#   RUN_WITH_MEMORY_CAP_BREAK=selftest-cleanup-unhooked  自证不挂收尾（EXIT、TERM、INT 都不挂），只在走到末尾时停自证开的 slice、删临时目录
#       「自证开出 slice 之后被 TERM 停时」「自证开出 slice 之后被 INT 停时」两格红
#   RUN_WITH_MEMORY_CAP_BREAK=peak-output-with-page-cache  往 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 写 scope 的 memory.peak（含页缓存），不是回收不掉的那一截
#       「页缓存涨满上限的那一趟，RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 那一份应当是回收不掉的 16M」红
#   RUN_WITH_MEMORY_CAP_BREAK=scene-move-unbounded  搬现场不看个数与字节上限，整份照搬（文件头「现场目录搬多少」）
#       「搬现场超上限时」红
#   RUN_WITH_MEMORY_CAP_BREAK=scene-progress-silent  搬现场不打进度行
#       「搬现场的过程」红
#   RUN_WITH_MEMORY_CAP_BREAK=scene-sparse-moved  稀疏的大镜像照普通文件搬
#       「现场里稀疏的大镜像」红
#   RUN_WITH_MEMORY_CAP_BREAK=scene-partial-left  搬现场中途失败不删搬了一半的临时名
#       「搬现场中途失败时」红
#   RUN_WITH_MEMORY_CAP_BREAK=scene-move-serial  搬现场只开 1 路，逐个搬
#       「搬现场 80 个每个慢 0.2 秒的文件时」「搬现场到总时限时」红（后一格：1 秒时限里逐个搬只搬得完两份日志，排在日志后面的那一份也到时限没搬）
#   RUN_WITH_MEMORY_CAP_BREAK=scene-move-time-unlimited  搬现场不设总时限，一直搬到搬齐
#       「搬现场到总时限时」红
#   RUN_WITH_MEMORY_CAP_BREAK=scene-move-logs-not-first  搬现场不把日志排在前面，照路径次序派
#       「搬现场到总时限时」红
# 弄坏开关只在自证里认：RUN_WITH_MEMORY_CAP_BREAK 非空（guard-off 除外）、头一个参数不是 --selftest、又没有自证的标记的，在 preflight 之后、读参数之前退 2，
#   stderr 点名这个变量只在自证（--selftest）里认，命令一行都没跑。自证的标记：RUN_WITH_MEMORY_CAP_SELFTEST_MARK=<文件>，那份文件在、头一行是 SELFTEST_MARK_CONTENT 那一行。
#   --selftest 把它建在自己的临时目录里、export 给起的每一份包装；把这个开关原样传给包装的调用方自证（scripts/hosts/multi-host-run.sh）
#   同样建一份写这一行的标记文件、export 这个变量。标记随环境传给命令；自证收尾删掉标记文件，漏出去的变量指不到文件，不再认。
#
# 发信号的地方，每一处只打得到这一条自己起的进程（后面的脚本不能终止前面的脚本，不能动 ssh；
# records/2026-09-16-subagent拆分提案.md 第四十节第 32 行）：
#   撞顶（OOMPolicy=stop）与超时（RuntimeMaxSec）由 systemd 停这一条自己开的 scope，scope 里只有这一条的外壳与它的命令；
#   查完结局 reset-failed 的是这一条自己的 unit；外壳里的 trap 只收信号、不发；起了 scope 之后被停时的 trap（stop_own_scope_then_exit）先 systemctl --user stop 点名自己开的那个 $unit.scope，
#   scope 还没建出来时只给自己 `&` 起、`$!` 记下的那个 systemd-run 发 TERM（发之前核 /proc 里它的父进程是这个包装）；采样子进程同样只停 `&` 记下的那一个；
#   slice 名只许 singlefs 开头（case 前面那句核对）：包装给它设 MemoryMax、自证收尾停的是自证自己开的 singlefs_memory_selftest_<进程号>_<序号>.slice，
#   写成 user.slice、app.slice 这类就等于给别人的进程设上限；
#   自证里 kill -9 $$ 停的是那条命令自己；「起了 scope 之后停包装」「采样子进程被停」两例停的是它自己 `&` 起、`$!` 记下的那个包装（收尾停的是自证自己的 slice）；TERM 风暴那一例先核 cgroup 路径在这一例自己的 slice 底下、
#   是自己开的 singlefs-memory-cap-*.scope，不对就一个信号都不发——弄坏开关 nocap、fallback 不开 scope，命令跑在调用方的 cgroup 里（会话里就是 SSH 会话的 session-*.scope）；
#   「自证开出 slice 之后被停」两格只给自己 `&` 起、`$!` 记下的那一趟自证发 TERM / INT，10 秒不退再给它发 KILL。
#
# 自证的收尾（selftest_cleanup）：--selftest 进来头一件事挂 EXIT 与 TERM、INT（TERM 退 143、INT 退 130，都经 EXIT），收尾逐个 systemctl --user stop 加 revert
#   自证开的 slice（selftest_slices：singlefs_memory_selftest_<进程号>_<序号>.slice 与「slice 名不是 singlefs 开头」那一例的），再删自证的临时目录；
#   走到末尾、中途 exit、被服务取消（TERM）、被 INT 都经它，一个 slice 都不留。正在跑的那一条包装是前台子进程时，bash 等它退了才跑 trap；
#   包装自己收到同一个 TERM 时照「起了 scope 之后包装被 TERM / INT 停」先停自己的 scope。
#   自证里起包装一律是子进程 bash "$target"，run_capped 设、清的 TERM / INT trap 在包装那一层，盖不到自证这一层；包装模式的 EXIT trap（on_wrapper_exit）只在
#   <上限> <命令> 那一种挂，--selftest 不挂。自证被 KILL 时 trap 不走，留下的 slice 要手清（systemctl --user stop 加 revert 那个名字）。
#
# 分在两份文件里：自证（run_selftest、它用的探针函数 bounded_memory_hog_code 与 selftest_job_code、收尾 selftest_cleanup）写在同目录 run-with-memory-cap-selftest.sh 里，
#   本文件在 --selftest 那一支里 source 它（路径照本文件自己所在的目录 SCRIPT_DIRECTORY 找，不照工作目录），别的调用不载入；自证读的全局变量与各 *_EXIT 常量仍在本文件定义。
#   弄坏开关 selftest-cleanup-unhooked 与 guard-off 在自证一侧的判断在 run-with-memory-cap-selftest.sh 里，别的开关的代码都在本文件；开关表仍写在本文件。
set -uo pipefail

MEMORY_CAP_HIT_EXIT=250
MEMORY_CAP_UNAVAILABLE_EXIT=251
TIME_LIMIT_HIT_EXIT=253
SLICE_TOTAL_HIT_EXIT=254
SCOPE_LEFT_EXIT=252                   # 命令退出之后停了自己的 scope、回读没过（文件头「命令退出之后停自己的 scope」）
SCOPE_LEFTOVER_COMMAND_CHARACTERS=120 # 列留下的进程时每个命令行取前这么多个字
GPU_MEMORY_HIT_EXIT=245               # 显存不够（文件头「显存不够」一段）
STOPPED_FROM_OUTSIDE_SIGNALS=" 1 2 9 15 "   # 命令退 128 加这几个信号号：外面发的 HUP、INT、KILL、TERM（文件头「峰值表」一段）；SEGV、ABRT 这类是命令自己崩的，不在内
GPU_JOURNAL_SLACK_SECONDS=2           # 读内核日志的窗口在命令退出之后再放宽这么多秒：驱动的那一行与命令退出前后脚
JOURNAL_PERMISSION_HINT="not seeing messages from other users and the system"   # journalctl 看不到别人的日志时退 0、只在 stderr 给这句（同 .claude/hooks/session-start.sh）
MEBIBYTE=$((1024 * 1024))
DEFAULT_KILL_GRACE_SECONDS=30
KEPT_TEMPORARY_MARKER=".run-with-memory-cap-kept"   # 退非 0 留着的临时目录里放的标记（文件头「这一条命令自己的临时目录」一段）；只删带它的，别的包装正在用的不碰
KEPT_TEMPORARY_DIRECTORIES_LIMIT=3              # 同一个 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 下留着的只留最近这么多个（照 command-safety.md 里 gate.sh 判红留现场的 3 个）
# 搬现场的上限（文件头「现场目录搬多少」）：任务 13229 在第二台经 NFS 搬 62277 个约 210 秒、89048 个 629 秒没搬完，照慢的那次推每秒约 140 个；
#   5000 个照这个速率约 36 秒，远在服务判卡住的 10 分钟（task-server/task_server_lib.py 错误类型表 stalled 一行）之内
SCENE_MOVE_ENTRY_LIMIT=5000
# 字节上限推的、没量：直连千兆 NFS（sync 导出）写按 50 MB/s 粗估，512 MiB 约 11 秒
SCENE_MOVE_BYTE_LIMIT=$((512 * MEBIBYTE))
# 搬现场的总时限（秒，从开搬算）：到点停派、在搬的搬完，没搬的记进清单、照搬成了办。第二台经 NFS 往共享根逐个写小文件量到过每秒不到 1 个
#   （任务 14496：1022 秒搬了 937 个、9557315 字节），两个上限挡不住这种慢；120 秒远在服务判卡住的 10 分钟之内
SCENE_MOVE_TIME_LIMIT_SECONDS=120
SCENE_MOVE_WORKERS=16                             # 搬现场并行几路：慢在每个文件往共享盘写的等待，不在 CPU
SCENE_SPARSE_IMAGE_MIN_BYTES=$((16 * MEBIBYTE))  # 表观至少这么大、占盘不到表观一半的普通文件算稀疏的大镜像，不搬（replay.sh 自证 truncate -s 512M 的 e9.img 占盘 0）
SCENE_LOG_SUFFIXES=".log .out .err .report .txt .json .tsv .md"   # 超上限时只搬的日志：这几个后缀
SCENE_LOG_FILE_MAX_BYTES=$MEBIBYTE                # 超上限时只搬的日志：每份不过这么大（再大的是数据，不是日志）
SCENE_PROGRESS_EVERY_ENTRIES=100                  # 搬现场每搬这么多个打一行进度
SCENE_PROGRESS_EVERY_SECONDS=5                    # 搬现场至多隔这么久打一行进度：服务判卡住要日志 10 分钟没涨
SCENE_MANIFEST_NAME=".run-with-memory-cap-scene-manifest.tsv"   # 现场目录里的清单（文件头「现场目录搬多少」）
SLICE_MEMORY_CAP_KEY="SLICE_MEMORY_CAP_BY_HOST" # 多机配置里各台总上限的键：<地址>=<总上限> 空白分隔，取这一台那一项（文件头「总上限」②）
BROKEN_JUDGEMENT="${RUN_WITH_MEMORY_CAP_BREAK:-}"
# 起 scope 的 systemd-run 放后台（文件头「起了 scope 之后包装被 TERM / INT 停」），脚本里 & 起的命令一律忽略 INT、进了 bash 也 trap 不上：包装自己进来时 INT 没被忽略的，给它设回默认；
# 进来时就被忽略的（调用方有意如此）照旧忽略
SCOPE_RUNNER_INT_OPTION=(--default-signal=INT)
[[ "$(trap -p INT)" != *"''"* ]] || SCOPE_RUNNER_INT_OPTION=()
SCOPE_RESULT_POLL_ATTEMPTS=50        # 查 scope 结局最多查这么多次，每次隔 0.1 秒：命令刚退出时 scope 可能还没收尾
SCOPE_REASON_ATTEMPTS=30              # failed 的 scope 读到 success 时，到 journal 里找失败原因最多读这么多次，每次隔 0.1 秒（文件头「failed 而 Result 读到 success」）
SCOPE_RESULT_LATE_ATTEMPTS=100        # 撞顶的 scope 结局读早了时再查 Result 最多这么多次，每次隔 0.1 秒（文件头「撞顶的 scope 结局读早了」）
SELFTEST_INTERRUPT_PROBE_SECONDS=60   # 设了 RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE 的那一趟自证开出 slice 之后至多等这么久被停（文件头「其余环境变量」）；起它的那一格 10 秒内就停
CAP_MODE=run                          # run_capped 的 mode（run：真要跑命令；check：--check 只试上限）
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# 弄坏开关只在自证里认（文件头「弄坏开关只在自证里认」一段）：平常跑设了就在读参数之前退 2；弄坏开关 guard-off 下不拒
SELFTEST_MARK_CONTENT="run-with-memory-cap.sh --selftest 起的包装"
if [[ -n "$BROKEN_JUDGEMENT" && "$BROKEN_JUDGEMENT" != guard-off && "${1:-}" != --selftest ]] \
    && [[ -z "${RUN_WITH_MEMORY_CAP_SELFTEST_MARK:-}" || "$(head -n 1 -- "$RUN_WITH_MEMORY_CAP_SELFTEST_MARK" 2>/dev/null)" != "$SELFTEST_MARK_CONTENT" ]]; then
  echo "  ✗ RUN_WITH_MEMORY_CAP_BREAK=$BROKEN_JUDGEMENT 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次命令一行都没跑（退 2）" >&2
  echo "     → 怎么办：去掉 RUN_WITH_MEMORY_CAP_BREAK 再跑；要证红就跑 RUN_WITH_MEMORY_CAP_BREAK=<项> bash scripts/hosts/run-with-memory-cap.sh --selftest；调用方的自证要把它传给包装的，照文件头那一段建标记文件、export RUN_WITH_MEMORY_CAP_SELFTEST_MARK" >&2
  exit 2
fi
# 只给自证的几个开关（文件头「其余环境变量」）：带着自证的标记才认，平常跑设了不起作用；采样那两个读完就清掉，不传进命令
SELFTEST_SAMPLED_CGROUP="" SELFTEST_SAMPLER_TRACE="" SELFTEST_HOST="" SELFTEST_OOM_SCORE_ADJUST="" SELFTEST_INTERRUPT_PROBE="" SELFTEST_SCENE_COPY_DELAY=0
if [[ -n "${RUN_WITH_MEMORY_CAP_SELFTEST_MARK:-}" && "$(head -n 1 -- "$RUN_WITH_MEMORY_CAP_SELFTEST_MARK" 2>/dev/null)" == "$SELFTEST_MARK_CONTENT" ]]; then
  [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_OOM_SCORE_ADJUST:-}" =~ ^[0-9]+$ ]] && SELFTEST_OOM_SCORE_ADJUST="$RUN_WITH_MEMORY_CAP_SELFTEST_OOM_SCORE_ADJUST"
  [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_LATE_ATTEMPTS:-}" =~ ^[0-9]+$ ]] && SCOPE_RESULT_LATE_ATTEMPTS="$RUN_WITH_MEMORY_CAP_SELFTEST_LATE_ATTEMPTS"
  SELFTEST_SAMPLED_CGROUP="${RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLED_CGROUP:-}"
  SELFTEST_SAMPLER_TRACE="${RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE:-}"
  SELFTEST_HOST="${RUN_WITH_MEMORY_CAP_SELFTEST_HOST:-}"
  SELFTEST_INTERRUPT_PROBE="${RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE:-}"
  [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_ENTRY_LIMIT:-}" =~ ^[0-9]+$ ]] && SCENE_MOVE_ENTRY_LIMIT="$RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_ENTRY_LIMIT"
  [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_TIME_LIMIT:-}" =~ ^[0-9]+([.][0-9]+)?$ ]] && SCENE_MOVE_TIME_LIMIT_SECONDS="$RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_TIME_LIMIT"
  [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_COPY_DELAY:-}" =~ ^[0-9]+([.][0-9]+)?$ ]] && SELFTEST_SCENE_COPY_DELAY="$RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_COPY_DELAY"
fi
unset RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLED_CGROUP RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE RUN_WITH_MEMORY_CAP_SELFTEST_HOST RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE \
  RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_ENTRY_LIMIT RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_TIME_LIMIT RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_COPY_DELAY
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIRECTORY/heavy-threads.sh"     # heavy_thread_default：峰值表的线程数一列认 capped.sh default
source "$SCRIPT_DIRECTORY/peer-host-lib.sh"     # peer_host_resolve_addresses、peer_host_configured_value、peer_host_value_for_host：总上限取这一台那一项（文件头「总上限」②）
SELF_PATH="$SCRIPT_DIRECTORY/$(basename "${BASH_SOURCE[0]}")"
SLICE="${RUN_WITH_MEMORY_CAP_SLICE:-singlefs-heavy.slice}"
SLICE_TOTAL_OVERRIDE="${RUN_WITH_MEMORY_CAP_SLICE_TOTAL:-}"
SLICE_CONFIGURATION="${SINGLEFS_MULTI_HOST_CONFIG:-$(cd "$SCRIPT_DIRECTORY/../.." 2>/dev/null && pwd)/multi-host.env}"   # 文件头「总上限」②
CGROUP_ROOT="${RUN_WITH_MEMORY_CAP_CGROUP_ROOT:-/sys/fs/cgroup}"
STATE_DIRECTORY="${RUN_WITH_MEMORY_CAP_STATE_DIR:-${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/singlefs-heavy-admission}"
TIME_LIMIT_SECONDS="${RUN_WITH_MEMORY_CAP_TIME_LIMIT:-}"
KILL_GRACE_SECONDS="${RUN_WITH_MEMORY_CAP_KILL_GRACE:-$DEFAULT_KILL_GRACE_SECONDS}"
JOURNALCTL="${RUN_WITH_MEMORY_CAP_JOURNALCTL:-journalctl}"   # 只给自证换成假的（文件头「显存不够」一段）
TEMPORARY_ROOT="${RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT:-/tmp}"   # 这一条命令自己的临时目录建在它下面（文件头「这一条命令自己的临时目录」一段）
# task-server 给这一层的两样（文件头「task-server 给这一层的两样」）：读了就 unset，不传进命令
PEAK_OUTPUT="${RUN_WITH_MEMORY_CAP_PEAK_OUTPUT:-}"
SCENE_DIRECTORY="${RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY:-}"
# multi-host-run.sh 给的两样（文件头「multi-host-run.sh 给这一层的两样」）：scope 单元名、这一条命令自己的临时目录的全路径各写进一份
UNIT_OUTPUT="${RUN_WITH_MEMORY_CAP_UNIT_OUTPUT:-}"
TEMPORARY_RECORD="${RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD:-}"
unset RUN_WITH_MEMORY_CAP_PEAK_OUTPUT RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY RUN_WITH_MEMORY_CAP_UNIT_OUTPUT RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD
# 只认服务直接起的那一层（文件头「只认服务直接起的那一层」）：起 scope 时从命令的环境里摘掉；taskserverleaks 下不摘
ADMITTED_BY_TASK_SERVER_UNSET=(-u RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER)
if [[ "$BROKEN_JUDGEMENT" == taskserverleaks ]]; then ADMITTED_BY_TASK_SERVER_UNSET=(); fi
PRIVATE_CONFIG="${RUN_WITH_MEMORY_CAP_PRIVATE_CONFIG:-${XDG_CONFIG_HOME:-${HOME:-}/.config}/singlefs/run-with-memory-cap.env}"   # 文件头「总上限」末尾
# 在 scope 里跑的外壳：命令不 exec，跑完读自己 cgroup 的 memory.peak 与 memory.events 写进峰值文件（scope 一收尾 cgroup 就删了）。
# 起命令之前把自己的 cgroup 路径（/proc/self/cgroup 去掉「0::」）写进起跑标记：包装的采样子进程照它找 scope 的 cgroup 目录（文件头「峰值表」一段）；
# 命令退出之后另按 memory_peaks_lib.py 的 unreclaimable_bytes 同一算法采一次回收不掉的用量（memory.current 减 active_file、inactive_file），写进峰值文件的 unreclaimable 一行。
# TERM / INT 设成记一笔的处理函数而不是忽略：忽略会被命令继承（exec 之后忽略的信号仍忽略），处理函数不会；
# 撞顶之后 systemd 停 scope 发的 TERM 于是只打断命令，外壳读完、写完再退，退出码照传命令的。
# 命令退出之后改成忽略 TERM / INT（后面不再起子进程，没有谁会继承它），只用 bash 的内建命令读、写（read、printf），不起 cut / cat / grep / mv：
# systemd 停 scope 时给 scope 里每个进程都发 TERM，那一刻正在跑的外部命令会被它杀掉、内建的 read 会被打断，峰值文件就写不全，
# 被总上限挤掉的那一条会被报成撞了自己的上限。
# 外壳自己的 stderr 指到 /dev/null、命令的 stderr 经 fd 4 接回原处：命令被信号杀掉时 bash 会补一行「line 10: … Killed "$@"」，那一行不是命令的输出。
# 命令进 scope 之后 oom_score_adj 抬到这个数（scope 里的命令与它起的子进程都继承）：整机内存吃紧时 earlyoom 与内核先挑任务进程，不挑常驻服务（prune-server、task-server、会话）
TASK_OOM_SCORE_ADJUST=500
SCOPE_HOLDER='trap "stop_requested=1" TERM INT
own_cgroup_line=""; IFS= read -r own_cgroup_line < /proc/self/cgroup
printf "%s\n" "${own_cgroup_line#0::}" > "$1"
if [[ -n "${RUN_WITH_MEMORY_CAP_OOM_SCORE_ADJUST:-}" ]]; then printf "%s\n" "$RUN_WITH_MEMORY_CAP_OOM_SCORE_ADJUST" > /proc/self/oom_score_adj 2>/dev/null || :; fi
unset RUN_WITH_MEMORY_CAP_OOM_SCORE_ADJUST
peak_file="$2"
cgroup_root="$3"
shift 3
exec 4>&2 2>/dev/null
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
current="" file_pages=0 unreclaimable=""
IFS= read -r current < "$own_group/memory.current"
while read -r stat_name stat_value; do
  case "$stat_name" in
    active_file|inactive_file) [[ "$stat_value" =~ ^[0-9]+$ ]] && file_pages=$((file_pages + stat_value)) ;;
  esac
done < "$own_group/memory.stat"
[[ "$current" =~ ^[0-9]+$ ]] && unreclaimable=$((current > file_pages ? current - file_pages : 0))
printf "peak %s\noom %s\noom_kill %s\nunreclaimable %s\n" "$peak" "$own_oom" "$oom_kill" "$unreclaimable" > "$peak_file"
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
  record_own_temporary_directory
}

# record_own_temporary_directory：给了 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD（文件头「multi-host-run.sh 给这一层的两样」）就把这一条命令自己的临时目录的全路径写进它
# （先写 .partial 再改名），写成了置 temporary_record_written，EXIT trap 收拾完那一份之后删；写不进说一句、照跑。弄坏开关 temporary-record-unwritten 下不写
temporary_record_written=0
record_own_temporary_directory() {
  [[ -n "$TEMPORARY_RECORD" && "$BROKEN_JUDGEMENT" != "temporary-record-unwritten" ]] || return 0
  if { printf '%s\n' "$own_temporary_directory" > "$TEMPORARY_RECORD.partial" && mv -f -- "$TEMPORARY_RECORD.partial" "$TEMPORARY_RECORD"; } 2>/dev/null; then
    temporary_record_written=1
  else
    rm -f -- "$TEMPORARY_RECORD.partial" 2>/dev/null
    echo "run-with-memory-cap: 这一条命令的临时目录的路径写不进 $TEMPORARY_RECORD（照跑；包装被 KILL 时留下的 $own_temporary_directory 没人照它删）" >&2
  fi
}

# prune_kept_temporary_directories <这一次留下的目录>：同一个 TEMPORARY_ROOT 下带标记的（退非 0 留着的）只留最近 KEPT_TEMPORARY_DIRECTORIES_LIMIT 个，
# 按标记的修改时刻、新的在前，更早的删掉；这一次留下的那一份不删，没有标记的（别的包装正在用的）不碰。弄坏开关 temporarykeptunbounded 下一份都不删
prune_kept_temporary_directories() {
  local marker kept_directory kept_count=0
  local -a markers=()
  [[ "$BROKEN_JUDGEMENT" == "temporarykeptunbounded" ]] && return 0
  mapfile -t markers < <(ls -1t -- "$TEMPORARY_ROOT"/singlefs-run-with-memory-cap-*/"$KEPT_TEMPORARY_MARKER" 2>/dev/null)
  for marker in ${markers[@]+"${markers[@]}"}; do
    kept_directory="${marker%/"$KEPT_TEMPORARY_MARKER"}"
    kept_count=$((kept_count + 1))
    ((kept_count > KEPT_TEMPORARY_DIRECTORIES_LIMIT)) || continue
    [[ "$kept_directory" != "$1" ]] || continue
    rm -rf -- "${kept_directory:?}"
  done
}

# move_into_scene <目录> <为什么留> <谁删>：设了 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY（文件头「task-server 给这一层的两样」）就把这一份照「现场目录搬多少」搬成那个目录、
# 删掉原来那一份，stderr 点名新位置与谁删，交 0；没设、弄坏开关 scene-kept-in-temporary-root 下交 1；那个目录已在或搬的过程失败说一句、交 1（调用方照原样留在 TEMPORARY_ROOT 下）
move_into_scene() {
  [[ -n "$SCENE_DIRECTORY" && "$BROKEN_JUDGEMENT" != "scene-kept-in-temporary-root" ]] || return 1
  local mover_exit=0
  if [[ -e "$SCENE_DIRECTORY" ]]; then
    echo "run-with-memory-cap: 这一条命令的临时目录 $1 没搬进这一次尝试的现场目录 $SCENE_DIRECTORY（那里已有东西），照旧留在 $TEMPORARY_ROOT 下" >&2
    return 1
  fi
  python3 -B /dev/fd/3 "$1" "$SCENE_DIRECTORY" "$BROKEN_JUDGEMENT" "$SCENE_MOVE_ENTRY_LIMIT" "$SCENE_MOVE_BYTE_LIMIT" "$SCENE_SPARSE_IMAGE_MIN_BYTES" \
    "$SCENE_LOG_SUFFIXES" "$SCENE_LOG_FILE_MAX_BYTES" "$SCENE_PROGRESS_EVERY_ENTRIES" "$SCENE_PROGRESS_EVERY_SECONDS" "$SCENE_MANIFEST_NAME" \
    "$SCENE_MOVE_WORKERS" "$SCENE_MOVE_TIME_LIMIT_SECONDS" "$SELFTEST_SCENE_COPY_DELAY" 3<<'PY' || mover_exit=$?
# 搬现场（文件头「现场目录搬多少」）：数一遍、分出稀疏的大镜像与特殊文件、超上限只挑日志，日志排在前面，目录先建好、再派给线程池并行搬进同一层的临时名，
# 到总时限停派、等在搬的搬完，清单写完再改名；搬齐与到总时限都交 0。中途失败停派、等在搬的搬完、删掉临时名、交 1（删不掉交 2）。进度行与结局句打在 stderr 上（服务判卡住看日志涨不涨）
import os, shutil, signal, stat, sys, threading, time
from concurrent.futures import FIRST_EXCEPTION, ThreadPoolExecutor, wait
(source, scene, broken, entry_limit, byte_limit, sparse_minimum, log_suffixes, log_file_maximum,
 progress_every_entries, progress_every_seconds, manifest_name, workers, time_limit, copy_delay) = sys.argv[1:15]
entry_limit, byte_limit, sparse_minimum = int(entry_limit), int(byte_limit), int(sparse_minimum)
log_file_maximum, progress_every_entries, progress_every_seconds = int(log_file_maximum), int(progress_every_entries), float(progress_every_seconds)
log_suffixes = tuple(log_suffixes.split())
workers = 1 if broken == "scene-move-serial" else int(workers)
time_limit = None if broken == "scene-move-time-unlimited" else float(time_limit)
copy_delay = float(copy_delay)   # 只给自证的慢拷贝：每个文件搬之前睡这么久（文件头「其余环境变量」）

def say(text):
    sys.stderr.write("run-with-memory-cap: " + text + "\n")
    sys.stderr.flush()

class Stopped(Exception):
    pass

def on_signal(number, frame):
    raise Stopped(f"收到信号 {signal.Signals(number).name}")

signal.signal(signal.SIGTERM, on_signal)
signal.signal(signal.SIGINT, on_signal)

def escaped(path):
    return path.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n")

def is_log(entry):
    return entry[1] == "file" and entry[0].endswith(log_suffixes) and entry[2] <= log_file_maximum

# 数一遍：(相对路径, 种类, 表观字节, 占盘字节)；目录不单列（搬文件之前照路径建），符号链接不跟；数不清（读不了、被停）就不开搬、交 1
def walk_error(error):
    raise error

entries = []
try:
    for directory, subdirectories, files in os.walk(source, onerror=walk_error):
        subdirectories.sort()
        for name in sorted(files + [entry for entry in subdirectories if os.path.islink(os.path.join(directory, entry))]):
            full = os.path.join(directory, name)
            status = os.lstat(full)
            relative = os.path.relpath(full, source)
            if stat.S_ISLNK(status.st_mode):
                entries.append((relative, "link", 0, 0))
            elif stat.S_ISREG(status.st_mode):
                allocated = status.st_blocks * 512
                sparse = status.st_size >= sparse_minimum and allocated * 2 < status.st_size and broken != "scene-sparse-moved"
                entries.append((relative, "sparse" if sparse else "file", status.st_size, allocated))
            else:
                entries.append((relative, "special", status.st_size, status.st_blocks * 512))
except (OSError, Stopped) as error:
    say(f"搬现场没开搬：数不清 {source} 里有什么（{error}）")
    sys.exit(1)
movable = [entry for entry in entries if entry[1] in ("file", "link")]
movable_bytes = sum(entry[2] for entry in movable)
over_limit = (len(movable) > entry_limit or movable_bytes > byte_limit) and broken != "scene-move-unbounded"
chosen = set()
if over_limit:
    chosen_bytes = 0
    for entry in movable:
        if len(chosen) >= entry_limit:
            break
        if is_log(entry) and chosen_bytes + entry[2] <= byte_limit:
            chosen.add(entry[0])
            chosen_bytes += entry[2]
else:
    chosen = {entry[0] for entry in movable}
to_move = [entry for entry in movable if entry[0] in chosen]
if broken != "scene-move-logs-not-first":
    to_move.sort(key=lambda entry: not is_log(entry))   # 稳定排序：日志在前、别的在后，各自照路径次序；到总时限没搬的是排在后面的
total_bytes = sum(entry[2] for entry in to_move)
silent = broken == "scene-progress-silent"
if over_limit:
    say(f"现场要搬 {len(movable)} 个、{movable_bytes} 字节，过了上限（{entry_limit} 个、{byte_limit} 字节）：只搬其中的日志 {len(to_move)} 个，别的只记进清单 {manifest_name}")

staging = f"{scene}.partial-{os.getpid()}"
started = time.monotonic()
deadline = None if time_limit is None else started + time_limit
progress_lock = threading.Lock()   # 管 moved 与 progress：各路搬完一个都在锁里记数、判要不要打进度行
moved = set()
progress = {"count": 0, "bytes": 0, "said": started}

def move_one(entry):
    relative, kind, size, allocated = entry
    if copy_delay > 0:
        time.sleep(copy_delay)
    source_path = os.path.join(source, relative)
    target_path = os.path.join(staging, relative)
    if kind == "link":
        os.symlink(os.readlink(source_path), target_path)
    else:
        shutil.copy2(source_path, target_path, follow_symlinks=False)
    with progress_lock:
        moved.add(relative)
        progress["count"] += 1
        progress["bytes"] += size
        now = time.monotonic()
        if not silent and (progress["count"] % progress_every_entries == 0 or now - progress["said"] >= progress_every_seconds):
            say(f"搬现场进度：{progress['count']}/{len(to_move)} 个、{progress['bytes']}/{total_bytes} 字节（已用 {now - started:.0f} 秒）")
            progress["said"] = now

executor = None
try:
    os.mkdir(staging)
    if not silent:
        say(f"搬现场进度：开搬 {len(to_move)} 个、{total_bytes} 字节、{workers} 路并行、总时限 {'不设' if time_limit is None else f'{time_limit:g} 秒'} → {staging}（搬齐改名成 {scene}）")
    for directory in sorted({os.path.dirname(entry[0]) for entry in to_move} - {""}):
        os.makedirs(os.path.join(staging, directory), exist_ok=True)
    executor = ThreadPoolExecutor(max_workers=workers)
    futures = [executor.submit(move_one, entry) for entry in to_move]
    # 等到搬齐、有一份失败或到总时限；每次至多等 1 秒，TERM / INT 在主线程上照常打断
    while True:
        remaining = None if deadline is None else deadline - time.monotonic()
        if remaining is not None and remaining <= 0:
            break
        done, pending = wait(futures, timeout=1.0 if remaining is None else min(remaining, 1.0), return_when=FIRST_EXCEPTION)
        for future in done:
            future.result()
        if not pending:
            break
    executor.shutdown(wait=True, cancel_futures=True)   # 到总时限：没开搬的撤掉，在搬的等它搬完
    for future in futures:
        if not future.cancelled():
            future.result()
    elapsed = time.monotonic() - started
    states = {"sparse": "sparse-not-moved", "special": "special-not-moved"}
    with open(os.path.join(staging, manifest_name), "w", encoding="utf-8", errors="surrogateescape") as manifest:
        manifest.write("# 状态\t种类\t表观字节\t占盘字节\t相对路径（run-with-memory-cap.sh 文件头「现场目录搬多少」）\n")
        for relative, kind, size, allocated in entries:
            if relative in moved:
                state = "moved"
            elif relative in chosen:
                state = "time-limit-not-moved"
            else:
                state = states.get(kind, "over-limit-not-moved")
            manifest.write(f"{state}\t{kind}\t{size}\t{allocated}\t{escaped(relative)}\n")
    os.rename(staging, scene)
except (OSError, shutil.Error, Stopped) as error:
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
    signal.signal(signal.SIGINT, signal.SIG_IGN)
    if executor is not None:
        executor.shutdown(wait=True, cancel_futures=True)   # 先停派、等在搬的搬完，再删临时名：不然删的同时还有线程往里写
    if broken != "scene-partial-left":
        shutil.rmtree(staging, ignore_errors=True)
    if os.path.lexists(staging):
        say(f"✗ 搬现场失败（{error}），搬了一半的临时名 {staging} 还在")
        sys.exit(2)
    say(f"搬现场失败（{error}）：搬了一半的临时名已删，{scene} 没建")
    sys.exit(1)
sparse_count = sum(1 for entry in entries if entry[1] == "sparse")
special_count = sum(1 for entry in entries if entry[1] == "special")
moved_bytes = sum(entry[2] for entry in to_move if entry[0] in moved)
time_limit_count = len(to_move) - len(moved)
if not silent:
    if time_limit_count:
        say(f"搬现场进度：到总时限停 {len(moved)}/{len(to_move)} 个、{moved_bytes}/{total_bytes} 字节（用了 {elapsed:.0f} 秒）")
    else:
        say(f"搬现场进度：搬完 {len(to_move)} 个、{total_bytes} 字节（用了 {elapsed:.0f} 秒）")
if time_limit_count:
    say(f"现场搬到总时限 {time_limit:g} 秒没搬完：搬了 {len(moved)} 个，到时限没搬的 {time_limit_count} 个（清单里记成 time-limit-not-moved），已搬的留在现场目录里")
say(f"现场搬了 {len(moved)} 个（用了 {elapsed:.1f} 秒）；没搬：稀疏的大镜像 {sparse_count} 个、套接字管道设备 {special_count} 个、超上限的 {len(movable) - len(to_move)} 个、"
    f"到总时限的 {time_limit_count} 个；清单 {os.path.join(scene, manifest_name)}")
PY
  if (( mover_exit != 0 )); then
    echo "run-with-memory-cap: 这一条命令的临时目录 $1 没搬进这一次尝试的现场目录 $SCENE_DIRECTORY（搬的过程失败，上一行是原因），照旧留在 $TEMPORARY_ROOT 下" >&2
    return 1
  fi
  rm -rf -- "${1:?}"
  [[ ! -e "$1" ]] || echo "run-with-memory-cap: 搬完现场之后原来那一份 $1 删不干净（rm -rf 之后回读还在）" >&2
  echo "run-with-memory-cap: $2：这一条命令的临时目录搬进了这一次尝试的现场目录：$SCENE_DIRECTORY（命令输出里点到 $1 的路径，换成这个目录读）" >&2
  echo "run-with-memory-cap: → 不用自己删：$3（task-server/task_server_lib.py 文件头「现场」）" >&2
  return 0
}

# end_own_temporary_directory <包装的退出码>：包装退 0 删掉这一条命令自己的临时目录；退非 0 而里面有东西就留着看现场、stderr 打出路径（文件头「这一条命令自己的临时目录」一段）
end_own_temporary_directory() {
  local directory="$own_temporary_directory" wrapper_exit="$1"
  [[ -n "$directory" ]] || return 0
  own_temporary_directory=""
  if [[ "${SINGLEFS_KEEP_TEST_TEMPORARY_FILES:-}" == 1 && "$BROKEN_JUDGEMENT" != "temporarykeepignored" ]]; then
    move_into_scene "$directory" "SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1" "留到提交方 task-server/task clean <任务号> 时删" && return 0
    echo "run-with-memory-cap: SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：这一条命令的临时目录留着看现场：$directory" >&2
    return 0
  fi
  # 弄坏开关 temporarydeletedonfailure：不论结局都删；temporarykeptempty：空的也留
  if [[ "$wrapper_exit" != 0 && "$BROKEN_JUDGEMENT" != "temporarydeletedonfailure" ]] \
      && { [[ -n "$(ls -A -- "$directory" 2>/dev/null)" ]] || [[ "$BROKEN_JUDGEMENT" == "temporarykeptempty" ]]; }; then
    move_into_scene "$directory" "包装退 ${wrapper_exit}（命令判红、被信号杀、撞了上限或超时，不是 0）" "提交方 task wait 取走结局之后一段时间由 task-server 删" && return 0
    printf '包装退 %s 留下的现场（包装进程号 %s）\n' "$wrapper_exit" "$$" > "$directory/$KEPT_TEMPORARY_MARKER"
    echo "run-with-memory-cap: 包装退 ${wrapper_exit}（命令判红、被信号杀、撞了上限或超时，不是 0）：这一条命令的临时目录留着看现场：$directory" >&2
    echo "run-with-memory-cap: → 看完自己删（rm -rf $directory）；同一个 $TEMPORARY_ROOT 下这样留着的只留最近 $KEPT_TEMPORARY_DIRECTORIES_LIMIT 个，更早的由之后留现场的那一条包装删掉" >&2
    prune_kept_temporary_directories "$directory"
    return 0
  fi
  [[ "$BROKEN_JUDGEMENT" == "temporaryleft" ]] && return 0
  rm -rf -- "${directory:?}"
  if [[ -e "$directory" ]]; then
    echo "run-with-memory-cap: 这一条命令的临时目录 $directory 删不干净（rm -rf 之后回读还在），退出码照命令的结局" >&2
    echo "run-with-memory-cap: → 怎么办：ls -la $directory 看剩下什么、权限对不对；有进程还开着里面的文件就先 python3 scripts/hosts/run_build_directory.py holders $directory 找出来按进程号停，再手删" >&2
  fi
}

# EXIT trap：收拾这一条命令自己的临时目录（退 0 删、退非 0 而里面有东西留着），再删记着它路径的那份文件（文件头「multi-host-run.sh 给这一层的两样」；
# 弄坏开关 temporary-record-kept-at-exit 下不删）；不调 exit，包装的退出码照旧
on_wrapper_exit() {
  end_own_temporary_directory "$?"
  if (( temporary_record_written )) && [[ "$BROKEN_JUDGEMENT" != "temporary-record-kept-at-exit" ]]; then
    rm -f -- "$TEMPORARY_RECORD"
  fi
}

check_size_syntax() { # check_size_syntax <写法> <是什么>
  [[ "$1" =~ ^[1-9][0-9]*[KMGT]$ ]] \
    || reject_usage "$2要写成 正整数加 K / M / G / T，单位必写（例 16G、512M；不带单位的数 systemd 当字节数），收到的是「$1」" "照 systemd 的 MemoryMax 写法给"
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

# 多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（文件头「总上限」②）：写对了打那一项的写法、交 0；配置读不到或没这个键交 1；
# 有这个键而认不出这一台、没有这一台那一项、那一项写法不对，打哪里不对、交 2。这一台照同一份配置的 HOSTS 认（peer-host-lib.sh 的 peer_host_resolve_addresses）。
# 弄坏开关 slicecapfirstentry 下不看地址取头一项；slicecapsyntaxlenient 下写坏的当没写
configured_slice_cap() {
  local text hosts_text this_address entry value problem=""
  [[ -r "$SLICE_CONFIGURATION" && "$BROKEN_JUDGEMENT" != "slicecapconfigignored" ]] || return 1
  text="$(peer_host_configured_value "$SLICE_CONFIGURATION" "$SLICE_MEMORY_CAP_KEY")" || return 1
  if [[ "$BROKEN_JUDGEMENT" == "slicecapfirstentry" ]]; then
    entry="${text%%[[:space:]]*}"
    value="${entry#*=}"
  else
    hosts_text="$(peer_host_configured_value "$SLICE_CONFIGURATION" HOSTS)" || hosts_text=""
    if ! peer_host_resolve_addresses "$hosts_text" >/dev/null 2>&1 || [[ -z "$THIS_HOST_ADDRESS" ]]; then
      problem="认不出这一台：同一份配置的 HOSTS 里没有恰好一项是这一台网卡上的地址（或没写 HOSTS、ip 跑不起来）"
    else
      this_address="$THIS_HOST_ADDRESS"
      value="$(peer_host_value_for_host "$text" "$this_address")" || problem="没有这一台那一项（HOSTS 里每一台恰好一项）"
    fi
  fi
  if [[ -z "$problem" && ! "$value" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
    problem="这一台那一项「$value」不是正整数加 K / M / G / T"
  fi
  if [[ -n "$problem" ]]; then
    [[ "$BROKEN_JUDGEMENT" == "slicecapsyntaxlenient" ]] && return 1
    printf '%s' "$problem"
    return 2
  fi
  printf '%s' "$value"
}

# 定总上限（文件头「总上限」）：设好 SLICE_TOTAL_BYTES、SLICE_TOTAL_SOURCE、SLICE_TOTAL_ADOPTED（1：照 slice 已有的走，不改 slice）；定不下来设 SLICE_PROBLEM、交非 0。
# 按 MiB 向下取整：cgroup 的 memory.max 按页取整，取整到 MiB 之后读回来与写进去的逐字相同
resolve_slice_total() { # resolve_slice_total <slice 此刻的 memory.max，读不到给空>
  local existing="$1" configured configured_status total
  SLICE_TOTAL_ADOPTED=0 SLICE_PROBLEM=""
  if [[ -n "$SLICE_TOTAL_OVERRIDE" ]]; then
    total="$(size_in_bytes "$SLICE_TOTAL_OVERRIDE")" || { SLICE_PROBLEM="RUN_WITH_MEMORY_CAP_SLICE_TOTAL=$SLICE_TOTAL_OVERRIDE 换成字节超过 64 位"; return 1; }
    SLICE_TOTAL_SOURCE="环境里的 RUN_WITH_MEMORY_CAP_SLICE_TOTAL=$SLICE_TOTAL_OVERRIDE"
  else
    configured="$(configured_slice_cap)"; configured_status=$?
    if ((configured_status == 2)); then
      SLICE_PROBLEM="多机配置 $SLICE_CONFIGURATION 里 $SLICE_MEMORY_CAP_KEY 写法不对（$configured），要写成 $SLICE_MEMORY_CAP_KEY=<地址>=<正整数加 K / M / G / T> <地址>=<…>（HOSTS 里每一台恰好一项，例 <地址>=50G）"
      return 1
    fi
    if ((configured_status == 0)); then
      total="$(size_in_bytes "$configured")" || { SLICE_PROBLEM="多机配置里 $SLICE_MEMORY_CAP_KEY=$configured 换成字节超过 64 位"; return 1; }
      SLICE_TOTAL_SOURCE="多机配置 $SLICE_CONFIGURATION 的 $SLICE_MEMORY_CAP_KEY 里这一台那一项 $configured"
    elif [[ "$existing" =~ ^[1-9][0-9]*$ && "$BROKEN_JUDGEMENT" != "existingtotalrefused" ]]; then
      SLICE_TOTAL_BYTES="$existing" SLICE_TOTAL_ADOPTED=1
      SLICE_TOTAL_SOURCE="slice $SLICE 已有的总上限（多机配置 $SLICE_CONFIGURATION 读不到或没写 $SLICE_MEMORY_CAP_KEY，照已有的走、不改）"
      return 0
    else
      SLICE_PROBLEM="定不出总上限：环境里没给 RUN_WITH_MEMORY_CAP_SLICE_TOTAL，多机配置 $SLICE_CONFIGURATION 读不到或没写 $SLICE_MEMORY_CAP_KEY，slice $SLICE 也还没有总上限"
      return 1
    fi
  fi
  total=$((total / MEBIBYTE * MEBIBYTE))
  ((total > 0)) || { SLICE_PROBLEM="总上限按 MiB 取整之后是 0（$SLICE_TOTAL_SOURCE）"; return 1; }
  SLICE_TOTAL_BYTES="$total"
}

# 把 slice 的总上限设上、确认 cgroup 里真的是这个数：成功时设好 SLICE_TOTAL_BYTES、SLICE_DIRECTORY；失败时设 SLICE_PROBLEM、交非 0。
# 已经是这个数就不再设（每一次尝试都经这里，免得一条一次 D-Bus 写）。定不出总上限时交 2（弄坏开关 unboundedwithoutcap 认它）
ensure_slice() {
  local cgroup_path control_output read_back existing=""
  SLICE_PROBLEM=""
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  [[ -n "$cgroup_path" ]] && existing="$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)"
  resolve_slice_total "$existing" || return 2
  if [[ -n "$cgroup_path" && "$existing" == "$SLICE_TOTAL_BYTES" ]]; then
    SLICE_DIRECTORY="$CGROUP_ROOT$cgroup_path"
    clear_leftover_memory_high
    return 0
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
  read_back="$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)"
  if [[ "$read_back" != "$SLICE_TOTAL_BYTES" ]]; then
    SLICE_PROBLEM="设了 MemoryMax=$SLICE_TOTAL_BYTES（$SLICE_TOTAL_SOURCE），读回 $CGROUP_ROOT$cgroup_path/memory.max 是「${read_back:-读不到}」（cgroup 的 memory 控制器没下放给用户级 systemd？或别的调用者刚按另一个数设了）"
    return 1
  fi
  SLICE_DIRECTORY="$CGROUP_ROOT$cgroup_path"
  clear_leftover_memory_high
}

# 别的版本留在 slice 上的 memory.high 清回 max（文件头「总上限」memory.high 那一段）：读用内建 read，是 max 或读不到（cgroup 目录不在）什么都不做；
# 清不掉（set-property 失败、读回不是 max）退 251，命令一行都没跑。弄坏开关 memoryhighleft：不清；memoryhighunverified：清了不读回
clear_leftover_memory_high() {
  local high_file="$SLICE_DIRECTORY/memory.high" memory_high="" read_back="" control_output
  IFS= read -r memory_high 2>/dev/null < "$high_file"
  [[ -n "$memory_high" && "$memory_high" != max && "$BROKEN_JUDGEMENT" != "memoryhighleft" ]] || return 0
  if ! control_output="$(systemctl --user set-property --runtime "$SLICE" MemoryHigh=infinity 2>&1)"; then
    report_unavailable "slice $SLICE 上留着软上限 memory.high=$memory_high（别的版本的包装设的），systemctl --user set-property --runtime $SLICE MemoryHigh=infinity 失败：$control_output；命令一行都没跑" \
      "手动跑 systemctl --user set-property --runtime $SLICE MemoryHigh=infinity，再 cat $high_file 看是不是 max；清回 max 之前别跑：带着软上限，撞线的进程卡在 D 状态（mem_cgroup_handle_over_high）、不 OOM"
  fi
  if [[ "$BROKEN_JUDGEMENT" == "memoryhighunverified" ]]; then
    read_back=max
  else
    IFS= read -r read_back 2>/dev/null < "$high_file"
  fi
  if [[ "$read_back" != max ]]; then
    report_unavailable "slice $SLICE 上留着软上限 memory.high=$memory_high（别的版本的包装设的），set-property --runtime MemoryHigh=infinity 之后读回 $high_file 是「${read_back:-读不到}」，不是 max；命令一行都没跑" \
      "systemctl --user cat $SLICE 看哪一份 drop-in 还写着 MemoryHigh（runtime 的在 \${XDG_RUNTIME_DIR:-/run/user/\$(id -u)}/systemd/user.control/$SLICE.d/ 下），改成 MemoryHigh=infinity 或删掉、systemctl --user daemon-reload，再 cat $high_file 读到 max 才跑；不许带着软上限跑"
  fi
  echo "run-with-memory-cap: slice $SLICE 上留着软上限 memory.high=$memory_high（别的版本的包装设的），已 set-property --runtime MemoryHigh=infinity 清回 max（读回 $high_file 是 max）" >&2
}

prepare_state_directory() {
  mkdir -p "$STATE_DIRECTORY/peaks" 2>/dev/null && : >> "$STATE_DIRECTORY/lock" 2>/dev/null
}

# 峰值表的键（文件头「峰值表」一段）：取法只在 memory_peaks_lib.py 的 command_key，task-server 取同一份
command_key() {
  python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import memory_peaks_lib as peaks; sys.stdout.write(peaks.command_key(sys.argv[3:], sys.argv[2]))' \
    "$SCRIPT_DIRECTORY" "${RUN_WITH_MEMORY_CAP_KEY:-}" "$@"
}

# 峰值表的「线程数」一列（文件头「峰值表」一段）：命令里经 capped.sh <N> 跑的取 N，经 capped.sh default 跑的取 heavy_thread_default 的值（登记值或 SINGLEFS_THREAD_CAP）；
# 否则看包装自己环境里 capped.sh 设的那几个线程变量。弄坏开关 threadsdefaultunset 下不认 default
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

# 起了 scope 之后包装收到 TERM / INT（trap 在 run_capped 里设，从起 scope 管到命令退出）：先 systemctl --user stop 自己开的那个 scope 单元
# （单元名是这一条自己起的 $unit.scope），scope 里的命令随之收到 TERM；再删标记与峰值文件，退 143 / 130。
# systemd-run 放后台、用 wait 等：前台等着时 bash 不跑 trap，命令在 scope 里照跑到底。
# scope 还没建出来（stop 找不到单元，systemd-run 还在起）：改给 systemd-run 那一个进程发 TERM，它在 scope 建出来之前自己就是被停的对象；
# 进程号是 `&` 当场记下的（`&` 与记下之间就到的，拿 $!），发之前核 /proc 里它的父进程是这个包装。
scope_runner_pid=""        # systemd-run 的进程号：放后台那一刻记下，wait 等到它退出就清空
scope_runner_forking=0     # 正在起 systemd-run
peak_sampler_pid=""        # 采样子进程的进程号（文件头「峰值表」一段）
peak_sampler_forking=0
stop_own_scope_then_exit() { # stop_own_scope_then_exit <退出码>
  local runner_pid="$scope_runner_pid" stat_line
  local -a stat_fields=()
  if [[ -z "$runner_pid" ]] && ((scope_runner_forking)); then
    runner_pid="${!:-}"
  fi
  stop_peak_sampler   # 采样子进程（文件头「峰值表」一段）跟着停、等它退，不留到包装退出之后
  rm -f -- "$peak_file.unreclaimable" "$peak_file.unreclaimable.partial"
  if [[ "$BROKEN_JUDGEMENT" != "term-leaves-scope" ]] && ! systemctl --user stop "$unit.scope" >/dev/null 2>&1; then
    if [[ -n "$runner_pid" ]] && IFS= read -r stat_line 2>/dev/null < "/proc/$runner_pid/stat"; then
      read -r -a stat_fields <<<"${stat_line##*) }"
      [[ "${stat_fields[1]:-}" == "$$" ]] && kill -TERM "$runner_pid" 2>/dev/null
    fi
  fi
  rm -f -- "$started_marker" "$peak_file" "$peak_file.partial"
  exit "$1"
}

# 列 scope 里的进程（文件头「命令退出之后停自己的 scope」）：<进程号，一行一个> → 每个一行「<进程号> <命令行前 SCOPE_LEFTOVER_COMMAND_CHARACTERS 字>」；读不到命令行的写「（命令行读不到）」
scope_process_lines() { # scope_process_lines <cgroup.procs 的内容>
  local pid command_line
  while IFS= read -r pid; do
    [[ "$pid" =~ ^[0-9]+$ ]] || continue
    command_line="$(tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null | head_characters "$SCOPE_LEFTOVER_COMMAND_CHARACTERS")"
    printf '%s %s\n' "$pid" "${command_line:-（命令行读不到）}"
  done <<<"$1"
}

# 命令退出之后停自己的 scope（文件头「命令退出之后停自己的 scope」）：先读一次 cgroup.procs，还有进程的说一句、逐个列；一律 systemctl --user stop 这一条自己的 $unit.scope，
# 回读 is-active 的输出是 inactive 或 failed、cgroup.procs 的内容是空的（目录已不在算空）才退 0；没过打原因、逐个列留下的进程与出路，退 1。
# 只停这一条自己开的单元（单元名是 run_capped 现起的、带包装的进程号）。弄坏开关 scope-left-after-exit 下不停、退 0
stop_scope_after_command() { # stop_scope_after_command <单元名> <scope 的 cgroup 目录，读不到给空>
  local unit="$1" group="$2" before="" after="" state
  [[ "$BROKEN_JUDGEMENT" != "scope-left-after-exit" ]] || return 0
  [[ -n "$group" && -d "$group" ]] && before="$(cat -- "$group/cgroup.procs" 2>/dev/null)"
  if [[ -n "$before" ]]; then
    echo "run-with-memory-cap: 命令退出之后 scope $unit 里还留着 $(grep -c . <<<"$before") 个进程，停掉自己的 scope：$(scope_process_lines "$before" | tr '\n' '；')" >&2
  fi
  systemctl --user stop "$unit.scope" >/dev/null 2>&1   # 判的是下面回读的输出，不看 stop 的退出码（单元已收尾时它报找不到单元）
  state="$(systemctl --user is-active "$unit.scope" 2>/dev/null)"
  [[ -n "$group" && -d "$group" ]] && after="$(cat -- "$group/cgroup.procs" 2>/dev/null)"
  if [[ ( "$state" == inactive || "$state" == failed ) && -z "$after" ]]; then
    return 0
  fi
  echo "  ✗ run-with-memory-cap: 命令退出之后停了自己的 scope $unit，回读 is-active 是「${state:-空}」、cgroup.procs 里还有 $(grep -c . <<<"$after") 个进程（退 $SCOPE_LEFT_EXIT）：" >&2
  scope_process_lines "$after" | sed 's/^/      /' >&2   # gate-lint:detail
  echo "  → 怎么办：这条命令退出后还留着子进程，核实脚本有没有不等子进程就退；照上面的进程号逐个看它们为什么停不掉（D 状态等 IO 的等 IO 缓过来），停掉之后 systemctl --user is-active $unit.scope 回读不在了再重跑" >&2
  return 1
}

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
  # ActiveState 是 failed 而 Result 读到 success：用户级 systemd 重载过配置，failed 的 scope 的 Result 被清成 success，
  # 原因只剩 journal 里那一行 Failed with result。取得到就照它办，取不到写明、Result 照读到的（文件头「failed 而 Result 读到 success」）
  if [[ "$active_state" == "failed" && "$result" == "success" && "$BROKEN_JUDGEMENT" != "scoperesultonly" ]]; then
    local failed_reason
    failed_reason="$(scope_failed_reason "$unit")"
    if [[ -n "$failed_reason" ]]; then
      echo "run-with-memory-cap: scope $unit 已落到 failed 而 Result 读到 success（用户级 systemd 重载过配置，Result 被清掉了），journal 里它失败的原因是 $failed_reason，照 $failed_reason 办" >&2
      result="$failed_reason"
    else
      echo "run-with-memory-cap: scope $unit 已落到 failed 而 Result 读到 success（用户级 systemd 重载过配置，Result 被清掉了），journal 里也查不到它失败的原因（没有 Failed with result 那一行，或 journalctl 读不了）：原因取不到，Result 照读到的 success 办，退出码照命令自己的" >&2
    fi
  fi
  if [[ "$active_state" == "failed" ]]; then
    systemctl --user reset-failed "$unit.scope" >/dev/null 2>&1   # 撞顶、超时的 scope 停在 failed，不清掉就一直挂在用户级 systemd 里
  fi
  echo "${result:-unreadable}"
}

# failed 的 scope 的失败原因：读它的 journal 里最后一行 Failed with result '<原因>'，打印原因；查不到什么都不打印。journal 的行落在 scope 失败的那一刻，
# 但读它的时候写入可能还在路上，没读到就再读，最多 SCOPE_REASON_ATTEMPTS 次、每次隔 0.1 秒；journalctl 本身出错或不在就不再读。
scope_failed_reason() { # scope_failed_reason <单元名>
  local unit="$1" attempt journal_text reason=""
  command -v "$JOURNALCTL" >/dev/null 2>&1 || return 0
  for ((attempt = 1; attempt <= SCOPE_REASON_ATTEMPTS; attempt++)); do
    journal_text="$(timeout 20 "$JOURNALCTL" --user -u "$unit.scope" -o cat --no-pager -n 50 2>/dev/null)" || return 0
    reason="$(sed -n "s/.*Failed with result '\\([a-z-]*\\)'.*/\\1/p" <<<"$journal_text" | tail -n 1)"
    [[ -z "$reason" ]] || break
    sleep 0.1
  done
  printf '%s' "$reason"
}

# 撞顶的 scope 结局读早了（文件头「撞顶的 scope 结局读早了」）：命令退非 0、scope 自己的 memory.events 里 oom_kill 不为 0，而 scope_result 读到的 Result 不是 oom-kill 时，
# 再查最多 SCOPE_RESULT_LATE_ATTEMPTS 次。查到 oom-kill 打印「oom-kill <第几次查到>」（并等它落到 failed 再 reset-failed，同 scope_result 的理由），一直没有打印「missed <最后读到的 Result>」
scope_late_oom_result() { # scope_late_oom_result <单元名>
  local unit="$1" attempt properties active_state result="" seen_at=""
  for ((attempt = 1; attempt <= SCOPE_RESULT_LATE_ATTEMPTS; attempt++)); do
    properties="$(systemctl --user show -p ActiveState -p Result "$unit.scope" 2>/dev/null)" || break
    active_state="$(sed -n 's/^ActiveState=//p' <<<"$properties")"
    result="$(sed -n 's/^Result=//p' <<<"$properties")"
    if [[ "$result" == "oom-kill" ]]; then
      [[ -n "$seen_at" ]] || seen_at="$attempt"
      if [[ "$active_state" == "inactive" || "$active_state" == "failed" ]]; then
        [[ "$active_state" != "failed" ]] || systemctl --user reset-failed "$unit.scope" >/dev/null 2>&1
        break
      fi
    fi
    sleep 0.1
  done
  if [[ -n "$seen_at" ]]; then echo "oom-kill $seen_at"; else echo "missed ${result:-空}"; fi
}

peak_file_value() { # peak_file_value <峰值文件> <名字> → 那一行的数；没有就空
  [[ -f "$1" ]] && awk -v wanted="$2" '$1 == wanted {print $2}' "$1"
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

# 采回收不掉的用量的峰值（文件头「峰值表」一段）：放后台起，命令退出、包装被停或换了父进程时自己退；包装收尾时只给它这一个进程号发 TERM。
# 读法复用 memory_peaks_lib.py，不另写一份「回收不掉的用量」
sample_unreclaimable_peak() { # sample_unreclaimable_peak <起跑标记> <systemd-run 的进程号> <输出文件>
  exec python3 -B /dev/fd/3 "$SCRIPT_DIRECTORY" "$CGROUP_ROOT" "$1" "$2" "$$" "$3" "$SELFTEST_SAMPLED_CGROUP" "$SELFTEST_SAMPLER_TRACE" "$BROKEN_JUDGEMENT" 3<<'PY'
import contextlib, os, sys
sys.dont_write_bytecode = True
script_directory, cgroup_root, marker_path, runner_text, wrapper_text, output_path, fixed_directory, trace_path, broken = sys.argv[1:10]
sys.path.insert(0, script_directory)
import memory_peaks_lib as peaks
runner, wrapper = int(runner_text), int(wrapper_text)
left = broken == "samplerleft"   # 弄坏开关 samplerleft
found = [fixed_directory]


def alive(process_id):
    try:
        os.kill(process_id, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def scope_directory():
    """外壳起命令之前写进起跑标记的 cgroup 路径（整行写完才认）；还没写就交空串，下一轮再看。"""
    if not found[0]:
        try:
            with open(marker_path, encoding="utf-8") as handle:
                line = handle.read()
        except OSError:
            return ""
        if line.endswith("\n") and line.strip().startswith("/"):
            found[0] = cgroup_root.rstrip("/") + line.strip()
    return found[0]


def keep_going():
    if left:
        return os.getppid() == wrapper
    return os.getppid() == wrapper and alive(runner)


try:
    peaks.sample_unreclaimable_peak(scope_directory, output_path, keep_going)
except Exception as error:   # 读 cgroup、写输出出了没想到的事：这一次只剩外壳收尾时采的那一次
    print(f"run-with-memory-cap: 采回收不掉的用量出了 {type(error).__name__}: {error}，这一次的峰值只按外壳收尾时采的那一次记", file=sys.stderr)
wrapper_gone = os.getppid() != wrapper
if wrapper_gone:
    for path in (output_path, output_path + ".partial"):
        with contextlib.suppress(FileNotFoundError):
            os.remove(path)
if trace_path:
    with contextlib.suppress(OSError):
        with open(trace_path, "a", encoding="utf-8") as handle:
            handle.write("exit wrapper-gone\n" if wrapper_gone else "exit command-ended\n")
PY
}

# 停采样子进程：只给 `&` 当场记下的那一个进程号发 TERM，再等它退出。`&` 与记下进程号之间被停的（被停那一支，stop_own_scope_then_exit），
# 拿 $!，发之前核 /proc 里它的父进程是这个包装。弄坏开关 samplerleft：不停、不等
stop_peak_sampler() {
  local sampler_pid="$peak_sampler_pid" stat_line
  local -a stat_fields=()
  if [[ -z "$sampler_pid" ]] && ((peak_sampler_forking)); then
    sampler_pid="${!:-}"
    if [[ -n "$sampler_pid" ]] && IFS= read -r stat_line 2>/dev/null < "/proc/$sampler_pid/stat"; then
      read -r -a stat_fields <<<"${stat_line##*) }"
      [[ "${stat_fields[1]:-}" == "$$" ]] || sampler_pid=""
    else
      sampler_pid=""
    fi
  fi
  peak_sampler_pid="" peak_sampler_forking=0
  [[ -n "$sampler_pid" && "$BROKEN_JUDGEMENT" != samplerleft ]] || return 0
  kill -TERM "$sampler_pid" 2>/dev/null
  wait "$sampler_pid" 2>/dev/null
}

# 记峰值表（文件头「峰值表」一段）：outcome 是 clean（命令自己退出，第四个参数给 scope 的 memory.peak）、own-cap（撞了自己的上限，第四个参数给上限的字节数）、空串（不记峰值）；
# 记不进只说一句，退出码照旧
record_run() { # record_run <outcome> <键> <上限写法> <memory.peak> <回收不掉的峰值> <oom_kill> <线程数>
  env RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED="$SCRIPT_DIRECTORY" RUN_WITH_MEMORY_CAP_PEAKS_RESOLVED="$PEAK_TABLE" RUN_WITH_MEMORY_CAP_HOST_RESOLVED="$SELFTEST_HOST" \
    python3 -B /dev/fd/3 "$@" 3<<'PY'
import contextlib, fcntl, os, sys
from datetime import datetime, timezone

sys.dont_write_bytecode = True
sys.path.insert(0, os.environ["RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED"])
import memory_peaks_lib as peaks

MEBIBYTE = 1024 * 1024
explicit_peak_table = os.environ["RUN_WITH_MEMORY_CAP_PEAKS_RESOLVED"]   # RUN_WITH_MEMORY_CAP_PEAKS；空串是默认那一份（文件头「峰值表」一段）
peak_table_path = explicit_peak_table or "默认的峰值表"                    # 记不进时那一句里说的名字；默认那一份在 record_peak 里现取
selftest_host = os.environ["RUN_WITH_MEMORY_CAP_HOST_RESOLVED"]          # 只给自证的第七列（文件头「其余环境变量」）；空串照多机配置认这一台
broken = os.environ.get("RUN_WITH_MEMORY_CAP_BREAK", "")
PEAK_TABLE_HEADER = [
    "# 每条命令近几次实测的内存峰值：scripts/hosts/run-with-memory-cap.sh 建、写，读它的见 scripts/hosts/memory_peaks_lib.py 文件头「峰值表的口径与列」，别手改。",
    "# 口径：回收不掉的用量（scope 的 memory.current 减 memory.stat 的 active_file、inactive_file）的峰值，跑的过程中每 "
    f"{peaks.PEAK_SAMPLE_SECONDS:g} 秒采一次、连同外壳收尾时采的一次取最大，按 MiB 向上取整的字节数；不含页缓存。",
    "#   撞顶只认真的 OOM（scope 的 Result=oom-kill 而不是被总上限挤掉，或 memory.events 里 oom_kill 大于 0），记成上限；超时、被总上限挤掉、被停的不记（量到的是半截）。"
    "数只对量它的那一台成立：两台共用这一份（共享数据根下），每行第七列记是哪一台，读的时候只认本台的行。",
    "# 列（制表符分隔）：峰值字节、量的时刻（ISO 8601，Z 结尾）、那一次的上限、键（memory_peaks_lib.py 的 command_key；RUN_WITH_MEMORY_CAP_KEY 可以指定）、"
    "线程数（命令里 capped.sh 的 N，否则包装环境里 capped.sh 那几个线程变量的值，都没设写「未设（整机 N 核）」）、"
    f"近几次（最近 {peaks.RECENT_MEASUREMENTS} 次实测，新的在前，逗号隔开，每次「<峰值字节>:hit:unreclaimable」撞了顶或「<峰值字节>:clean:unreclaimable」整段跑完）、"
    "哪一台（多机配置 HOSTS 里那一台的地址）。前三列是最近那一次。",
]


def write_peak_table(path, rows):
    """rows 是 (哪一台, 键) → 五段（memory_peaks_lib.read_all_peak_rows 那一种）；两台的行都写回。"""
    lines = PEAK_TABLE_HEADER + [peaks.peak_table_line(key, row, host) for (host, key), row in sorted(rows.items())]
    temporary = f"{path}.{os.getpid()}.partial"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    os.replace(temporary, path)


@contextlib.contextmanager
def table_lock(path):
    """表旁的 <表>.lock（文件头「峰值表」一段：两台都写共享根下这一份，锁跟着表）。"""
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path + ".lock", "a", encoding="utf-8") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def record_peak(outcome, key, cap_text, peak_text, unreclaimable_text, oom_kill_text, threads):
    cap_bytes = peaks.size_bytes(cap_text)
    if not outcome or not peak_text.isdigit():
        return
    measurement = peaks.judged_measurement(outcome, cap_bytes, int(peak_text), int(unreclaimable_text) if unreclaimable_text.isdigit() else None,
                                           int(oom_kill_text) if oom_kill_text.isdigit() else 0,
                                           record_memory_peak=(broken == "recordmemorypeak"), hit_by_peak=(broken == "hitbypeak"))
    if measurement is None:   # clean 而一次都没采到（文件头「峰值表」一段）：不记
        return
    peak = -(-measurement[0] // MEBIBYTE) * MEBIBYTE
    hit = measurement[1]
    path = explicit_peak_table or peaks.default_peak_table()   # 取不到抛 PeakTableUnavailable（文件头「峰值表」一段）
    host = selftest_host or peaks.this_host()                  # 第七列；取不到同样抛 PeakTableUnavailable
    with table_lock(path):
        rows = peaks.read_all_peak_rows(path)
        if broken == "hostcolumnignored":   # 弄坏开关：一台一份表，只留本台的行
            rows = {row_key: row for row_key, row in rows.items() if row_key[0] == host}
        old = rows.get((host, key))
        recent = ([(peak, hit, peaks.MEASURED_UNRECLAIMABLE)] + (old[4] if old is not None else []))[:peaks.RECENT_MEASUREMENTS]
        measured_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        if (old is not None and broken != "hostcolumnignored"
                and peaks.peak_table_line(key, old, host) == peaks.peak_table_line(key, (peak, old[1], cap_text, threads, recent), host)):
            return
        rows[(host, key)] = (peak, measured_at, cap_text, threads, recent)
        write_peak_table(path, rows)


outcome, key, cap_text, peak_text, unreclaimable_text, oom_kill_text, threads_text = sys.argv[1:8]
try:
    record_peak(outcome, key, cap_text, peak_text, unreclaimable_text, oom_kill_text, threads_text or None)
except OSError as error:
    print(f"run-with-memory-cap: 峰值没记进 {peak_table_path}（{error}），这一条的退出码照旧", file=sys.stderr)
except peaks.PeakTableUnavailable as error:
    print(f"run-with-memory-cap: 峰值没记进表：{error}；这一条的退出码照旧", file=sys.stderr)
except peaks.TableRowRefused as error:
    if broken == "refusalunhandled":   # 弄坏开关：不接这一种，回溯栈打出来
        raise
    print(f"run-with-memory-cap: 峰值没记进表：{error}；这一条的退出码照旧", file=sys.stderr)
sys.exit(0)
PY
}

# write_peak_output <回收不掉的用量的峰值>：设了 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT（文件头「task-server 给这一层的两样」）而给的是正整数，写进那一份（先写临时文件再改名）；
# 写不成说一句、退出码照旧；弄坏开关 peak-output-unwritten 下不写
write_peak_output() {
  [[ -n "$PEAK_OUTPUT" && "$1" =~ ^[1-9][0-9]*$ && "$BROKEN_JUDGEMENT" != "peak-output-unwritten" ]] || return 0
  if ! { printf '%s\n' "$1" > "$PEAK_OUTPUT.partial" && mv -f -- "$PEAK_OUTPUT.partial" "$PEAK_OUTPUT"; } 2>/dev/null; then
    echo "run-with-memory-cap: 退出时读到的峰值没写进 $PEAK_OUTPUT（这一条的退出码照旧）" >&2
  fi
  return 0
}

# run_capped <上限> <run|check> <命令…>：check 只试起不起得来，不记峰值、不限时
run_capped() {
  local cap="$1" mode="$2" unit started_marker peak_file key command_threads_text exit_code result use_slice=1 slice_status
  local peak_bytes own_oom_count oom_kill_count scope_properties command_started command_ended gpu_verdict stopped_signal
  local late_oom_note="" late_result_was late_oom_outcome
  shift 2
  if [[ "$BROKEN_JUDGEMENT" == "nocap" ]]; then
    "$@"
    return
  fi
  CAP_MODE="$mode"
  prepare_state_directory || report_unavailable "峰值文件与锁的目录 $STATE_DIRECTORY 建不了（mkdir 或写 lock 失败），命令一行都没跑" \
    "查 \$XDG_RUNTIME_DIR（$STATE_DIRECTORY 的上一层）在不在、可写不可写；或设 RUN_WITH_MEMORY_CAP_STATE_DIR 到一个可写的目录"
  if [[ "$BROKEN_JUDGEMENT" == "noslice" ]]; then
    use_slice=0
  else
    ensure_slice; slice_status=$?
    if ((slice_status == 2)); then
      if [[ "$BROKEN_JUDGEMENT" == "unboundedwithoutcap" ]]; then
        use_slice=0
      else
        report_unavailable "slice $SLICE 的总上限定不下来：$SLICE_PROBLEM；命令一行都没跑" \
          "在多机配置（$SLICE_CONFIGURATION，模板是仓根 multi-host.env.example）里写一行 $SLICE_MEMORY_CAP_KEY=<地址>=<总上限> <地址>=<总上限>（HOSTS 里每一台恰好一项，例 <地址>=50G）；不许退回无总上限去跑"
      fi
    elif ((slice_status != 0)); then
      if [[ "$BROKEN_JUDGEMENT" == "slicefallback" ]]; then
        use_slice=0
      else
        report_unavailable "slice $SLICE 的总上限设不上：$SLICE_PROBLEM；命令一行都没跑" \
          "照上面那句修（systemctl --user status、cat /sys/fs/cgroup/user.slice/user-\$(id -u).slice/user@\$(id -u).service/cgroup.subtree_control 里要有 memory）；修好之前别跑这一步，不许退回无总上限去跑"
      fi
    fi
  fi
  unit="singlefs-memory-cap-$$-$RANDOM$RANDOM"
  if [[ "$mode" == run && -n "$UNIT_OUTPUT" ]]; then   # multi-host-run.sh 要的 scope 单元名（文件头「multi-host-run.sh 给这一层的两样」）
    { printf '%s\n' "$unit" > "$UNIT_OUTPUT.partial" && mv -f -- "$UNIT_OUTPUT.partial" "$UNIT_OUTPUT"; } 2>/dev/null \
      || echo "run-with-memory-cap: scope 单元名写不进 $UNIT_OUTPUT（照跑；第二台清理时只能照会话号与标签找留下的进程）" >&2
  fi
  key="$(command_key "$@")"
  command_threads_text="$(command_threads "$@")"
  started_marker="${TMPDIR:-/tmp}/run-with-memory-cap-$$-$RANDOM$RANDOM.started"
  peak_file="$STATE_DIRECTORY/peaks/$unit"
  # 起了 scope 之后被停（包括被外层 timeout 连同进程组一起停）：先停自己开的 scope，再把标记与峰值文件删掉（stop_own_scope_then_exit）
  trap 'stop_own_scope_then_exit 143' TERM
  trap 'stop_own_scope_then_exit 130' INT
  scope_properties=(-p MemoryMax="$cap" -p MemorySwapMax=0 -p OOMPolicy=stop)
  ((use_slice)) && scope_properties+=(--slice="$SLICE")
  if [[ "$mode" == "run" && -n "$TIME_LIMIT_SECONDS" && "$BROKEN_JUDGEMENT" != "notimelimit" ]]; then
    scope_properties+=(-p RuntimeMaxSec="$TIME_LIMIT_SECONDS" -p TimeoutStopSec="$KILL_GRACE_SECONDS")
  fi
  # 外壳里先建标记再跑命令：标记在，说明命令确实是在上限里起的；不在，说明 systemd-run 自己没起来。
  # 外面那层把本 shell 的 stderr 临时指到 /dev/null：外壳被 SIGKILL 时 bash 会补一行「… Killed  systemd-run …」，那一行不是命令的输出；
  # 命令自己的 stderr 经 fd 3 照旧接回原来的 stderr。只管这一条的几个变量在这里清掉，不传进命令；
  # 交给命令的两个标记（文件头「交给命令的两个标记」）
  printf -v command_started '%(%s)T' -1
  # 放后台、用 wait 等（stop_own_scope_then_exit 前的注释）；<&0 让命令照旧读调用方的标准输入（后台作业默认读 /dev/null）
  scope_runner_pid=""; scope_runner_forking=1
  { env "${SCOPE_RUNNER_INT_OPTION[@]}" -u RUN_WITH_MEMORY_CAP_KEY -u RUN_WITH_MEMORY_CAP_TIME_LIMIT -u RUN_WITH_MEMORY_CAP_JOURNALCTL -u RUN_WITH_MEMORY_CAP_SELFTEST_LATE_ATTEMPTS -u RUN_WITH_MEMORY_CAP_SELFTEST_OOM_SCORE_ADJUST \
      ${ADMITTED_BY_TASK_SERVER_UNSET[@]+"${ADMITTED_BY_TASK_SERVER_UNSET[@]}"} \
      RUN_WITH_MEMORY_CAP_OUTER_UNIT="$unit" RUN_WITH_MEMORY_CAP_OUTER_CAP="$cap" \
      RUN_WITH_MEMORY_CAP_OOM_SCORE_ADJUST="$([[ "$BROKEN_JUDGEMENT" == "oom-score-not-raised" ]] || printf %s "${SELFTEST_OOM_SCORE_ADJUST:-$TASK_OOM_SCORE_ADJUST}")" \
      systemd-run --user --scope --unit="$unit" "${scope_properties[@]}" --quiet \
      bash -c "$SCOPE_HOLDER" run-with-memory-cap "$started_marker" "$peak_file" "$CGROUP_ROOT" "$@" 2>&3 3>&-; } 3>&2 2>/dev/null <&0 &
  scope_runner_pid=$!; scope_runner_forking=0
  # 采回收不掉的用量的峰值（文件头「峰值表」一段）：真要跑命令的才采（--check 不记峰值、不采）
  local unreclaimable_peak="" sampled_peak="" peak_sample_file="$peak_file.unreclaimable"
  peak_sampler_pid=""
  if [[ "$mode" == "run" ]]; then
    peak_sampler_forking=1
    sample_unreclaimable_peak "$started_marker" "$scope_runner_pid" "$peak_sample_file" </dev/null >/dev/null &
    peak_sampler_pid=$!; peak_sampler_forking=0
    if [[ -n "$SELFTEST_SAMPLER_TRACE" ]]; then   # 自证：记下采样子进程的进程号与起始时刻，自证核包装退出那一刻它已经不在
      local sampler_stat_line=""
      local -a sampler_stat_fields=()
      IFS= read -r sampler_stat_line 2>/dev/null < "/proc/$peak_sampler_pid/stat"
      read -r -a sampler_stat_fields <<<"${sampler_stat_line##*) }"
      printf 'sampler %s %s\n' "$peak_sampler_pid" "${sampler_stat_fields[19]:-}" >> "$SELFTEST_SAMPLER_TRACE"
    fi
  fi
  wait "$scope_runner_pid" 2>/dev/null
  exit_code=$?
  scope_runner_pid=""
  stop_peak_sampler
  sampled_peak="$(peak_file_value "$peak_sample_file" peak)"
  rm -f -- "$peak_sample_file" "$peak_sample_file.partial"
  printf -v command_ended '%(%s)T' -1
  if [[ ! -e "$started_marker" ]]; then
    rm -f -- "$peak_file" "$peak_file.partial"
    trap - TERM INT
    if [[ "$BROKEN_JUDGEMENT" == "fallback" ]]; then
      "$@"
      return
    fi
    report_unavailable "systemd-run --user --scope 退出码 $exit_code，命令一行都没跑（上面是 systemd-run 自己的报错）" \
      "查用户级 systemd 与 D-Bus（systemctl --user status、echo \$DBUS_SESSION_BUS_ADDRESS \$XDG_RUNTIME_DIR）；起不来就别跑这一步，不许退回无上限去跑"
  fi
  local scope_cgroup_line="" scope_cgroup_directory="" scope_left=0
  IFS= read -r scope_cgroup_line 2>/dev/null < "$started_marker"   # 外壳起命令之前写的自己的 cgroup（停 scope 之后照它回读 cgroup.procs）
  [[ "$scope_cgroup_line" == /* ]] && scope_cgroup_directory="$CGROUP_ROOT$scope_cgroup_line"
  rm -f -- "$started_marker"
  trap - TERM INT
  result="$(scope_result "$unit")"
  peak_bytes="$(peak_file_value "$peak_file" peak)"
  own_oom_count="$(peak_file_value "$peak_file" oom)"
  oom_kill_count="$(peak_file_value "$peak_file" oom_kill)"
  # 回收不掉的用量的峰值（文件头「峰值表」一段）：采样子进程采到的与外壳收尾时采的那一次取大的；都没采到是空
  unreclaimable_peak="$(peak_file_value "$peak_file" unreclaimable)"
  [[ "$unreclaimable_peak" =~ ^[0-9]+$ ]] || unreclaimable_peak=""
  [[ "$sampled_peak" =~ ^[0-9]+$ ]] || sampled_peak=""
  if [[ -n "$sampled_peak" ]] && { [[ -z "$unreclaimable_peak" ]] || ((sampled_peak > unreclaimable_peak)); }; then
    unreclaimable_peak="$sampled_peak"
  fi
  if [[ -n "$SELFTEST_SAMPLED_CGROUP" ]]; then
    # 自证的假 cgroup 目录（文件头「其余环境变量」）：memory.peak、oom_kill 改读它的；回收不掉的峰值只认采样子进程从它那里采到的
    IFS= read -r peak_bytes < "$SELFTEST_SAMPLED_CGROUP/memory.peak"
    oom_kill_count="$(awk '$1 == "oom_kill" {print $2}' "$SELFTEST_SAMPLED_CGROUP/memory.events" 2>/dev/null)"
    unreclaimable_peak="$sampled_peak"
  fi
  if [[ "$mode" == run ]]; then   # 交给调用方的峰值（文件头「task-server 给这一层的两样」）；弄坏开关 peak-output-with-page-cache 下写 memory.peak
    if [[ "$BROKEN_JUDGEMENT" == "peak-output-with-page-cache" ]]; then write_peak_output "$peak_bytes"; else write_peak_output "$unreclaimable_peak"; fi
  fi
  rm -f -- "$peak_file" "$peak_file.partial"
  if [[ "$mode" != "run" ]]; then
    [[ "$result" != "unreadable" ]] || report_unavailable "--check 起的 scope $unit 查不到结局（systemctl --user show 出错）" "查用户级 systemd 还在不在（systemctl --user status）"
    stop_scope_after_command "$unit" "$scope_cgroup_directory" || return "$SCOPE_LEFT_EXIT"
    return "$exit_code"
  fi
  # 撞顶的 scope 结局读早了（文件头「撞顶的 scope 结局读早了」）：内核在这条 scope 里杀过进程、命令退非 0，而 Result 读到的不是 oom-kill：再查几次；一直不是就按内核的计数办
  if [[ $exit_code -ne 0 && "$BROKEN_JUDGEMENT" != "oomresultlag" && "${oom_kill_count:-0}" =~ ^[1-9][0-9]*$ ]] \
      && [[ "$result" != "oom-kill" && "$result" != "timeout" && "$result" != "unreadable" ]]; then
    late_result_was="$result"
    late_oom_outcome="$(scope_late_oom_result "$unit")"
    if [[ "$late_oom_outcome" == "oom-kill "* ]]; then
      late_oom_note="scope $unit 的 Result 读早了读到 $late_result_was（它自己的 memory.events oom=${own_oom_count:-?} oom_kill=$oom_kill_count），再查第 ${late_oom_outcome#oom-kill } 次落到了 oom-kill"
    else
      late_oom_note="scope $unit 的 Result 一直是 ${late_oom_outcome#missed }，而它自己的 memory.events oom=${own_oom_count:-?} oom_kill=$oom_kill_count：内核杀过它里面的进程，按 memory.events 判成 oom-kill"
    fi
    result="oom-kill"
  fi
  # 结局查完了：一律停自己的 scope、回读（文件头「命令退出之后停自己的 scope」）；回读没过的，命令自己的退出码换成 SCOPE_LEFT_EXIT（下面最后那一处 return）
  stop_scope_after_command "$unit" "$scope_cgroup_directory" || scope_left=1
  if [[ "$BROKEN_JUDGEMENT" == "exitcode" ]]; then
    record_run clean "$key" "$cap" "$peak_bytes" "$unreclaimable_peak" "$oom_kill_count" "$command_threads_text"
    if [[ $exit_code -eq 137 || $exit_code -eq 143 ]]; then
      echo "run-with-memory-cap: 撞了内存上限 $cap（弄坏开关 exitcode：按退出码判）" >&2
      return "$MEMORY_CAP_HIT_EXIT"
    fi
    return "$exit_code"
  fi
  if [[ "$result" == "unreadable" ]]; then
    report_unavailable "命令退出码 $exit_code，之后查不到 scope $unit 的结局（systemctl --user show 出错），判不了是不是撞了上限" \
      "查用户级 systemd 还在不在（systemctl --user status）；这一条的结果不算数，修好再重跑"
  fi
  if [[ "$result" == "oom-kill" && "$BROKEN_JUDGEMENT" != "noresult" ]]; then
    [[ -z "$late_oom_note" ]] || echo "run-with-memory-cap: $late_oom_note" >&2
    if ((use_slice)) && [[ "$own_oom_count" == "0" && "${oom_kill_count:-0}" != "0" && "$BROKEN_JUDGEMENT" != "noslicehit" ]]; then
      # 被总上限挤掉：量到的是半截，峰值不记（弄坏开关 squeezedrecorded 照撞顶记）
      [[ "$BROKEN_JUDGEMENT" != "squeezedrecorded" ]] \
        || record_run own-cap "$key" "$cap" "$(size_in_bytes "$cap")" "$unreclaimable_peak" "$oom_kill_count" "$command_threads_text"
      echo "run-with-memory-cap: 被 slice $SLICE 的总上限挤掉了（scope $unit 的 Result=oom-kill，而它自己没撞它的上限 $cap：整个 slice 满了，内核在 slice 里挑了这一条杀；命令退出码 $exit_code）" >&2
      echo "run-with-memory-cap: → 这一条的结果不算数，重跑它；bash $SELF_PATH --status 看 slice 里同时在跑的是谁" >&2
      gpu_memory_note "$command_started" "$command_ended"
      return "$SLICE_TOTAL_HIT_EXIT"
    fi
    record_run own-cap "$key" "$cap" "$(size_in_bytes "$cap")" "$unreclaimable_peak" "$oom_kill_count" "$command_threads_text"
    echo "run-with-memory-cap: 撞了内存上限 $cap（scope $unit 的 Result=oom-kill，命令退出码 $exit_code）" >&2
    gpu_memory_note "$command_started" "$command_ended"
    return "$MEMORY_CAP_HIT_EXIT"
  fi
  if [[ "$result" == "timeout" ]]; then
    echo "run-with-memory-cap: 超过限时 ${TIME_LIMIT_SECONDS} 秒（scope $unit 的 Result=timeout，systemd 停掉了 scope 里的进程；命令退出码 $exit_code）" >&2
    return "$TIME_LIMIT_HIT_EXIT"
  fi
  stopped_signal=""
  if [[ "$BROKEN_JUDGEMENT" != "recordkilledpeak" ]]; then
    stopped_signal="$(stopped_from_outside_signal "$exit_code")" || stopped_signal=""
  fi
  if [[ -n "$stopped_signal" ]]; then
    # 命令被外面的信号停了（earlyoom 发的 TERM 这一类），而 scope 没撞自己的上限、也没被 slice 挤掉：量到的是半截，峰值不记（文件头「峰值表」一段）
    echo "run-with-memory-cap: 被外面停了（信号 $stopped_signal），这一次的峰值不记（命令退 $exit_code；scope $unit 已收尾，Result=${result:-空}、它自己的 memory.events oom=${own_oom_count:-?} oom_kill=${oom_kill_count:-?}，没撞它自己的上限 $cap，也没被 slice $SLICE 的总上限挤掉，量到的是半截）" >&2
    echo "run-with-memory-cap: → 这一条的结果不算数；查是谁发的信号（系统日志里 earlyoom 的记录、手动 kill、看门狗），腾出整机内存或等发信号的一方停手，再重跑；峰值表里这个键仍是上一次整段跑完的量" >&2
  else
    # 记什么（回收不掉的峰值、oom_kill 大于 0 记撞顶、采不到不记）由 memory_peaks_lib.judged_measurement 定
    record_run clean "$key" "$cap" "$peak_bytes" "$unreclaimable_peak" "$oom_kill_count" "$command_threads_text"
  fi
  # 显存不够（文件头「显存不够」一段）：命令退了非 0 才读内核日志
  if [[ "$BROKEN_JUDGEMENT" != "nogpuoom" ]] && [[ $exit_code -ne 0 || "$BROKEN_JUDGEMENT" == "gpuoomonsuccess" ]]; then
    gpu_verdict="$(gpu_memory_verdict "$command_started" "$command_ended")"
    case "$gpu_verdict" in
      hit\ *)
        echo "run-with-memory-cap: 显存不够（命令退 $exit_code；它起止之间内核日志里有 NVRM … NV_ERR_NO_MEMORY：${gpu_verdict#hit }）" >&2
        echo "run-with-memory-cap: → 这一条的结果不算数（退 $GPU_MEMORY_HIT_EXIT）：看同一段时间谁在用卡（nvidia-smi），交 task-server 重排（它对 gpu-memory-short 照同一上限重排一次）" >&2
        return "$GPU_MEMORY_HIT_EXIT" ;;
      unreadable\ *)
        echo "run-with-memory-cap: 显存 OOM 没查成（${gpu_verdict#unreadable }）：命令退 $exit_code，判不了是不是显存不够，不当成没撞；手动 journalctl -k | grep NV_ERR_NO_MEMORY 看一眼" >&2 ;;
    esac
  fi
  if ((scope_left)); then
    return "$SCOPE_LEFT_EXIT"   # 停了自己的 scope 回读还有进程（stop_scope_after_command 已列）：命令自己的退出码换成它
  fi
  return "$exit_code"
}

# --status（文件头「总上限」一段）：总上限从哪来、slice 此刻读回的 memory.max、slice 里回收不掉的用量与每个 scope、整机内存与 slice 外面的用量
show_status() {
  local cgroup_path slice_directory="" existing=""
  [[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  [[ -n "$cgroup_path" ]] && slice_directory="$CGROUP_ROOT$cgroup_path" && existing="$(cat "$slice_directory/memory.max" 2>/dev/null)"
  SLICE_TOTAL_BYTES="" SLICE_TOTAL_SOURCE=""
  resolve_slice_total "$existing" || SLICE_TOTAL_SOURCE="定不下来：$SLICE_PROBLEM"
  env RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED="$SCRIPT_DIRECTORY" python3 -B /dev/fd/3 "$SLICE" "${SLICE_TOTAL_BYTES:-}" "$SLICE_TOTAL_SOURCE" "$existing" "$slice_directory" \
    "$CGROUP_ROOT" "$PRIVATE_CONFIG" "$BROKEN_JUDGEMENT" 3<<'PY'
import os, sys
sys.dont_write_bytecode = True
sys.path.insert(0, os.environ["RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED"])
import memory_peaks_lib as peaks

slice_name, total_text, source, existing_text, slice_directory, cgroup_root, private_config, broken = sys.argv[1:9]
human = peaks.human


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


total_bytes = int(total_text) if total_text.isdigit() else None
print(f"  … slice {slice_name} 的总上限 {human(total_bytes) if total_bytes else '定不下来'}（{source}）；"
      f"slice 此刻读回的 memory.max {human(int(existing_text)) if existing_text.isdigit() else (existing_text or '读不到（slice 还没起）')}")
slice_used = (peaks.unreclaimable_bytes(slice_directory) or 0) if slice_directory else 0
scopes = []
if slice_directory and os.path.isdir(slice_directory):
    for name in sorted(os.listdir(slice_directory)):
        child = os.path.join(slice_directory, name)
        if name.endswith(".scope") and os.path.isdir(child):
            scopes.append((name[:-len(".scope")], peaks.unreclaimable_bytes(child)))
print(f"  … slice 里回收不掉的用量 {human(slice_used)}{'' if slice_directory else '（slice 还没起）'}；在跑的 scope {len(scopes)} 个{'：' if scopes else ''}")
for unit, used in scopes:
    print(f"      {unit}：回收不掉的 {human(used) if used is not None else '读不到'}")
memory = peaks.meminfo()
total_memory, available = memory["MemTotal"], memory["MemAvailable"]
outside = total_memory - available - slice_used
service_cgroup, why_unset = private_setting(private_config, "OUTSIDE_SERVICE_CGROUP")
if service_cgroup is None:
    service_text = f"没配（{why_unset}；配法见文件头「总上限」一段末尾）"
else:
    service = cgroup_stat(os.path.join(cgroup_root, service_cgroup.lstrip("/"), "memory.stat"), ("anon", "shmem"))
    service_text = human(service) if service is not None else "读不到（私有配置里的 OUTSIDE_SERVICE_CGROUP 底下没有可读的 memory.stat）"
user = cgroup_stat(os.path.join(cgroup_root, "user.slice/memory.stat"), ("anon",))
print(f"  … 整机内存 {human(total_memory)}（/proc/meminfo 的 MemTotal），此刻 MemAvailable {human(available)}；"
      f"slice 外面回收不掉的合计约 {human(outside)}（MemTotal − MemAvailable − slice 里的用量），其中本地模型服务的 anon + shmem "
      f"{service_text}、user.slice 的 anon {human(user) if user is not None else '读不到'}")
sys.exit(0)
PY
}


# slice 名只许 singlefs 开头的 .slice（文件头「发信号的地方」一段）
[[ "$SLICE" =~ ^singlefs[-_A-Za-z0-9]*\.slice$ ]] \
  || reject_usage "slice 名 RUN_WITH_MEMORY_CAP_SLICE 要写成 singlefs 开头的 .slice，收到的是「$SLICE」" \
       "不设它（默认 singlefs-heavy.slice），或写成 singlefs_<用途>.slice；别的 slice 里是别人的进程，给它设上限、停它就打到别人"
case "${1:-}" in
  --selftest)
    source "$SCRIPT_DIRECTORY/run-with-memory-cap-selftest.sh"   # 自证的函数只在这一支用，别的调用不载入
    run_selftest
    exit 0 ;;
  --status)
    show_status
    exit $? ;;
  --check)
    check_size_syntax "${2:-}" "内存上限"
    [[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
    PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-}"   # 空串是默认那一份，记的时候现取（文件头「峰值表」一段）
    run_capped "$2" check true
    exit $? ;;
  "")
    reject_usage "没给上限与命令" "写成 run-with-memory-cap.sh <上限> <命令> [参数…]，上限例 16G" ;;
esac

cap="$1"
shift
[[ $# -gt 0 ]] || reject_usage "没给要在上限里跑的命令" "写成 run-with-memory-cap.sh $cap <命令> [参数…]"
check_size_syntax "$cap" "内存上限"
size_in_bytes "$cap" >/dev/null || reject_usage "内存上限 $cap 太大，换成字节超过 64 位" "写成真要的量，例 16G"
[[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
[[ -z "$TIME_LIMIT_SECONDS" || "$TIME_LIMIT_SECONDS" =~ ^[1-9][0-9]*$ ]] \
  || reject_usage "限时 RUN_WITH_MEMORY_CAP_TIME_LIMIT 要写成正整数秒，收到的是「$TIME_LIMIT_SECONDS」" "写成例 1800；不限时就别设"
PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-}"   # 空串是默认那一份，记的时候现取（文件头「峰值表」一段）
# 只认服务直接起的那一层（文件头「只认服务直接起的那一层」一段）：没有服务给的准入标记就按用法错拒，命令没跑
[[ -n "${RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER:-}" ]] \
  || reject_usage "run-with-memory-cap.sh 只认 task-server 起尝试时直接套的那一层，环境里没有服务给的 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER（任务里面又套了一层，或不经服务直接起），命令一行都没跑" \
       "任务里面不再套包装：内存、线程、机器、卡由 task-server 定；删掉这一层（这一次的上限照读服务交的 TASK_SERVER_MEMORY_CAP_BYTES），整条经 task-server/task submit -- <命令…> 交"
# 这一条命令自己的临时目录：先挂 EXIT trap 再建，report_unavailable、stop_own_scope_then_exit 里的 exit 都走得到它
trap on_wrapper_exit EXIT
begin_own_temporary_directory
run_capped "$cap" run "$@"



exit $?
