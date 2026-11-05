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
#   包装退非 0（命令自己的非 0、被信号杀的 128 + 信号号、245、250、253、254，包装被 TERM / INT 停）时，包装给命令建的临时目录里有东西就留着、stderr 打出路径；
#   退 0 照删（文件头「这一条命令自己的临时目录」一段）。退出码本身不因此变。
#
# 总上限（用户 2026-10-31 定「两台都提到50G 不设置余量 不得超过这里设置的50g」，records/2026-10-30-编译绕过内存总上限.md）：照下面头一样有的定，不按整机内存减余量算：
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
#
# 撞顶的 scope 结局读早了（records/2026-10-23-subagent拆分提案.md 第一百三十七节）：命令退非 0、scope 自己的 memory.events 里 oom_kill 不为 0，而头一次查到的 Result 不是 oom-kill，
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
#   外壳不 exec 命令、留下来读，是因为 scope 一收尾它的 cgroup 就删了、systemd 也不留（2026-09-25 实测命令退出之后 MemoryPeak=[not set]）。
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
# 显存不够（用户 2026-10-06 定：运行时认显存 OOM，records/2026-09-16-subagent拆分提案.md 第五十五、五十六节）：命令退了非 0、或撞了上限、或被总上限挤掉时，
#   包装读 journalctl -k 里它起止之间（起跑前 1 秒到退出后 GPU_JOURNAL_SLACK_SECONDS 秒，--since / --until 按 epoch 秒给）的行，
#   同一行既有 NVRM 又有 NV_ERR_NO_MEMORY（显卡驱动报的显存不够）就报「显存不够」：退非 0 的那种退 245；撞了上限、被挤掉的照旧退 250 / 254，另报一行。
#   驱动的这一行不带进程号，同一段时间里别的进程撞了显存也算进来：宁可把这一条的结果当不算数、重跑，不把显存不够当成测试红。
#   读不到内核日志（没有 journalctl、权限不够、出错、超时、连最近一条内核消息都读不出）报「显存 OOM 没查成」，不当成没撞，退出码照命令自己的。
#   命令退 0 的不查。只给自证换 journalctl：RUN_WITH_MEMORY_CAP_JOURNALCTL=<假的那一份>（传进命令之前清掉）。
#
# 限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT，秒；不设不限）：给 scope 设 RuntimeMaxSec，从起跑算；到点 systemd 给 scope 里每个进程发 TERM，
#   RUN_WITH_MEMORY_CAP_KILL_GRACE（默认 30）秒还不退再发 KILL，退 253。RUN_WITH_MEMORY_CAP_KEY、_TIME_LIMIT、_JOURNALCTL 只管这一条，传进命令之前清掉。
# 这一条命令自己的临时目录（用户 2026-10-23 定案「自己用完自己删」；records/2026-09-16-subagent拆分提案.md 第九十二节末尾）：
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
#   --check、--status、--selftest 不起命令，不建。包装自己被 KILL / HUP 时 EXIT trap 不走，那一份留在 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 下（名字带包装进程号）。
# task-server 给这一层的两样（task-server/task_server_lib.py 文件头「现场」与「资源记录」的退出时峰值；包装开头读了就 unset，不传进命令）：
#   RUN_WITH_MEMORY_CAP_PEAK_OUTPUT=<文件>  外壳在命令退出之后读的 scope 的 memory.peak（文件头「峰值表」一段：scope 收尾之前一定读到一次），是正整数就写进这份文件
#     （先写 <文件>.partial 再改名）：命令比调用方一轮采样还短时，调用方照它记峰值（本机给 attempt_runner.py 读，第二台由 multi-host-run.sh 读了打回本机）。
#     读不到不写；--check 不写；起了 scope 之后包装被 TERM / INT 停的不写（量到的是半截）。
#   RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY=<目录>  这一次尝试的现场目录（共享工作根下，两台同一个绝对路径；这一层不建它的上一层）：上面「退非 0 而那一份里有东西」
#     与设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 这两种要留的，不留在 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 下、不放标记、不进「只留最近 3 个」的轮换：
#     整份 mv 成这个目录，stderr 打「…搬进了这一次尝试的现场目录：<目录>（命令输出里点到 <原路径> 的路径，换成这个目录读）」与一行谁删（task-server 删，不用自己删）。
#     这个目录已在、或 mv 失败：说一句，照没设它那样留在原处、放标记、进轮换。退出码照旧。
# 其余环境变量有默认，只在自证与特殊场合设：
#   RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 这一条命令自己的临时目录建在哪个目录下，默认 /tmp（上一段）
#   RUN_WITH_MEMORY_CAP_SLICE          slice 名，默认 singlefs-heavy.slice（名字里的 - 是 systemd 的层级：它挂在 singlefs.slice 底下）；只许 singlefs 开头的 .slice，别的按用法错退 251
#   RUN_WITH_MEMORY_CAP_SLICE_TOTAL    直接给总上限（systemd 写法，文件头「总上限」①）
#   RUN_WITH_MEMORY_CAP_STATE_DIR      峰值文件与锁的目录，默认 ${XDG_RUNTIME_DIR:-/run/user/<uid>}/singlefs-heavy-admission（tmpfs，重启清空）
#   RUN_WITH_MEMORY_CAP_CGROUP_ROOT    cgroup v2 挂在哪，默认 /sys/fs/cgroup
#   RUN_WITH_MEMORY_CAP_SELFTEST_TARGET       --selftest 测哪一份包装，默认自己：拿改前那一份跑新的自证，看它判错
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
#   RUN_WITH_MEMORY_CAP_BREAK=taskserverleaks  起 scope 时不从命令的环境里摘掉 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER（改前的样子：任务里面再套一层照跑）
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
#   RUN_WITH_MEMORY_CAP_BREAK=hostcolumnignored  记峰值时不留另一台的行（改前一台一份表的写法：整张表只写本台的行）
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
#       退 7、kill -9、撞顶、超时四格与「判红留着的只留最近 3 个」「设了现场目录、命令退 7 而临时目录里有东西时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarykeptempty  包装退非 0 而那一份是空的也留着
#       「命令退非 0 而临时目录里什么都没写时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarykeptunbounded  留现场时不删更早留着的
#       「判红留着的只留最近 3 个」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporaryshared  不给命令建它自己的临时目录
#       临时目录五格（命令拿到的 TMPDIR 不在 root 底下）与「设了现场目录、命令退 7 而临时目录里有东西时」「设了现场目录与 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarykeepignored  不认 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1，照删
#       「设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时」「设了现场目录与 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarycallerreplaced  调用方设了 TMPDIR 也另建一份顶掉它
#       「调用方设了 TMPDIR 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=temporarycallerdeleted  调用方设了 TMPDIR 时照它的，退出时却连它一起删
#       「调用方设了 TMPDIR 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=oom-score-not-raised  scope 里的命令不抬 oom_score_adj（照内核缺省 0；TASK_OOM_SCORE_ADJUST）
#       「scope 里的命令 oom_score_adj 应当是 500」红
#   RUN_WITH_MEMORY_CAP_BREAK=peak-output-unwritten  命令退出之后不往 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 写外壳读到的峰值（改前只靠调用方按轮采）
#       「命令跑 0.1 秒就退时，RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 那一份应当是外壳退出时读到的 scope 峰值」红
#   RUN_WITH_MEMORY_CAP_BREAK=scene-kept-in-temporary-root  设了 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY 也照改前留在 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 下、进轮换
#       「设了现场目录、命令退 7 而临时目录里有东西时」「设了现场目录与 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=memoryhighleft  不清 slice 上别的版本留下的 memory.high：「slice 上留着软上限 47.5G 时」「slice 原先不在、起来之后带着软上限 47.5G 时」
#       「清软上限之后读回不是 max 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=memoryhighunverified  清软上限之后不读回、当清掉了
#       「清软上限之后读回不是 max 时」红
#   RUN_WITH_MEMORY_CAP_BREAK=guard-off  平常跑设了弄坏开关也不拒（探针往下走到「没给要在上限里跑的命令」退 251、不点名）：
#       「平常跑（没有自证的标记）设了 RUN_WITH_MEMORY_CAP_BREAK」标记「空」与标记指到头一行不对的文件两格红
#   RUN_WITH_MEMORY_CAP_BREAK=selftest-cleanup-unhooked  自证不挂收尾（EXIT、TERM、INT 都不挂），只在走到末尾时停自证开的 slice、删临时目录（改前的样子）
#       「自证开出 slice 之后被 TERM 停时」「自证开出 slice 之后被 INT 停时」两格红
# 弄坏开关只在自证里认：RUN_WITH_MEMORY_CAP_BREAK 非空（guard-off 除外）、头一个参数不是 --selftest、又没有自证的标记的，在 preflight 之后、读参数之前退 2，
#   stderr 点名这个变量只在自证（--selftest）里认，命令一行都没跑。自证的标记：RUN_WITH_MEMORY_CAP_SELFTEST_MARK=<文件>，那份文件在、头一行是 SELFTEST_MARK_CONTENT 那一行。
#   --selftest 把它建在自己的临时目录里、export 给起的每一份包装；把这个开关原样传给包装的调用方自证（scripts/hosts/multi-host-run.sh）
#   同样建一份写这一行的标记文件、export 这个变量。标记随环境传给命令；自证收尾删掉标记文件，漏出去的变量指不到文件，不再认。
#
# 发信号的地方，每一处只打得到这一条自己起的进程（用户 2026-09-25 定「后面的脚本不能终止前面的脚本」「不能动 ssh」；
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
set -uo pipefail

MEMORY_CAP_HIT_EXIT=250
MEMORY_CAP_UNAVAILABLE_EXIT=251
TIME_LIMIT_HIT_EXIT=253
SLICE_TOTAL_HIT_EXIT=254
GPU_MEMORY_HIT_EXIT=245               # 显存不够（文件头「显存不够」一段）
STOPPED_FROM_OUTSIDE_SIGNALS=" 1 2 9 15 "   # 命令退 128 加这几个信号号：外面发的 HUP、INT、KILL、TERM（文件头「峰值表」一段）；SEGV、ABRT 这类是命令自己崩的，不在内
GPU_JOURNAL_SLACK_SECONDS=2           # 读内核日志的窗口在命令退出之后再放宽这么多秒：驱动的那一行与命令退出前后脚
JOURNAL_PERMISSION_HINT="not seeing messages from other users and the system"   # journalctl 看不到别人的日志时退 0、只在 stderr 给这句（同 .claude/hooks/session-start.sh）
MEBIBYTE=$((1024 * 1024))
DEFAULT_KILL_GRACE_SECONDS=30
KEPT_TEMPORARY_MARKER=".run-with-memory-cap-kept"   # 退非 0 留着的临时目录里放的标记（文件头「这一条命令自己的临时目录」一段）；只删带它的，别的包装正在用的不碰
KEPT_TEMPORARY_DIRECTORIES_LIMIT=3              # 同一个 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 下留着的只留最近这么多个（照 command-safety.md 里 gate.sh 判红留现场的 3 个）
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
# 弄坏开关只在自证里认（文件头「弄坏开关只在自证里认」一段）：平常跑设了就在读参数之前退 2；弄坏开关 guard-off 下不拒（改前的样子）
SELFTEST_MARK_CONTENT="run-with-memory-cap.sh --selftest 起的包装"
if [[ -n "$BROKEN_JUDGEMENT" && "$BROKEN_JUDGEMENT" != guard-off && "${1:-}" != --selftest ]] \
    && [[ -z "${RUN_WITH_MEMORY_CAP_SELFTEST_MARK:-}" || "$(head -n 1 -- "$RUN_WITH_MEMORY_CAP_SELFTEST_MARK" 2>/dev/null)" != "$SELFTEST_MARK_CONTENT" ]]; then
  echo "  ✗ RUN_WITH_MEMORY_CAP_BREAK=$BROKEN_JUDGEMENT 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次命令一行都没跑（退 2）" >&2
  echo "     → 怎么办：去掉 RUN_WITH_MEMORY_CAP_BREAK 再跑；要证红就跑 RUN_WITH_MEMORY_CAP_BREAK=<项> bash scripts/hosts/run-with-memory-cap.sh --selftest；调用方的自证要把它传给包装的，照文件头那一段建标记文件、export RUN_WITH_MEMORY_CAP_SELFTEST_MARK" >&2
  exit 2
fi
# 只给自证的几个开关（文件头「其余环境变量」）：带着自证的标记才认，平常跑设了不起作用；采样那两个读完就清掉，不传进命令
SELFTEST_SAMPLED_CGROUP="" SELFTEST_SAMPLER_TRACE="" SELFTEST_HOST="" SELFTEST_OOM_SCORE_ADJUST="" SELFTEST_INTERRUPT_PROBE=""
if [[ -n "${RUN_WITH_MEMORY_CAP_SELFTEST_MARK:-}" && "$(head -n 1 -- "$RUN_WITH_MEMORY_CAP_SELFTEST_MARK" 2>/dev/null)" == "$SELFTEST_MARK_CONTENT" ]]; then
  [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_OOM_SCORE_ADJUST:-}" =~ ^[0-9]+$ ]] && SELFTEST_OOM_SCORE_ADJUST="$RUN_WITH_MEMORY_CAP_SELFTEST_OOM_SCORE_ADJUST"
  [[ "${RUN_WITH_MEMORY_CAP_SELFTEST_LATE_ATTEMPTS:-}" =~ ^[0-9]+$ ]] && SCOPE_RESULT_LATE_ATTEMPTS="$RUN_WITH_MEMORY_CAP_SELFTEST_LATE_ATTEMPTS"
  SELFTEST_SAMPLED_CGROUP="${RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLED_CGROUP:-}"
  SELFTEST_SAMPLER_TRACE="${RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE:-}"
  SELFTEST_HOST="${RUN_WITH_MEMORY_CAP_SELFTEST_HOST:-}"
  SELFTEST_INTERRUPT_PROBE="${RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE:-}"
fi
unset RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLED_CGROUP RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE RUN_WITH_MEMORY_CAP_SELFTEST_HOST RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE
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
unset RUN_WITH_MEMORY_CAP_PEAK_OUTPUT RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY
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
# 被总上限挤掉的那一条会被报成撞了自己的上限（2026-09-25 门禁 code-tooling 里撞上过一次）。
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

# move_into_scene <目录> <为什么留> <谁删>：设了 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY（文件头「task-server 给这一层的两样」）就把这一份整个 mv 成那个目录、
# stderr 点名新位置与谁删，交 0；没设、弄坏开关 scene-kept-in-temporary-root 下交 1；那个目录已在或 mv 失败说一句、交 1（调用方照原样留在 TEMPORARY_ROOT 下）
move_into_scene() {
  [[ -n "$SCENE_DIRECTORY" && "$BROKEN_JUDGEMENT" != "scene-kept-in-temporary-root" ]] || return 1
  if [[ -e "$SCENE_DIRECTORY" ]] || ! mv -- "$1" "$SCENE_DIRECTORY" 2>/dev/null; then
    echo "run-with-memory-cap: 这一条命令的临时目录 $1 没搬进这一次尝试的现场目录 $SCENE_DIRECTORY（那里已有东西或 mv 失败），照旧留在 $TEMPORARY_ROOT 下" >&2
    return 1
  fi
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
  # 弄坏开关 temporarydeletedonfailure：不论结局都删（改前的跑法）；temporarykeptempty：空的也留
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

# EXIT trap：收拾这一条命令自己的临时目录（退 0 删、退非 0 而里面有东西留着）；不调 exit，包装的退出码照旧
on_wrapper_exit() {
  end_own_temporary_directory "$?"
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
        if broken == "hostcolumnignored":   # 弄坏开关：照改前一台一份表，只留本台的行
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

# write_peak_output <外壳退出时读到的 memory.peak>：设了 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT（文件头「task-server 给这一层的两样」）而读到的是正整数，写进那一份（先写临时文件再改名）；
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
  rm -f -- "$started_marker"
  trap - TERM INT
  result="$(scope_result "$unit")"
  peak_bytes="$(peak_file_value "$peak_file" peak)"
  [[ "$mode" == run ]] && write_peak_output "$peak_bytes"
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
    # 自证的假 cgroup 目录（文件头「其余环境变量」）：memory.peak 与 oom_kill 改读它的；回收不掉的峰值只认采样子进程从它那里采到的
    IFS= read -r peak_bytes < "$SELFTEST_SAMPLED_CGROUP/memory.peak"
    oom_kill_count="$(awk '$1 == "oom_kill" {print $2}' "$SELFTEST_SAMPLED_CGROUP/memory.events" 2>/dev/null)"
    unreclaimable_peak="$sampled_peak"
  fi
  rm -f -- "$peak_file" "$peak_file.partial"
  if [[ "$mode" != "run" ]]; then
    [[ "$result" != "unreadable" ]] || report_unavailable "--check 起的 scope $unit 查不到结局（systemctl --user show 出错）" "查用户级 systemd 还在不在（systemctl --user status）"
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

# ── 自证 ──
# 吃内存的探针：每次拿 16 MiB 写满（真占常驻页），拿到 MEBIBYTES 就停下正常退出；上限 64M 时它在半路被杀
bounded_memory_hog_code() {
  printf '%s\n' "import sys" "held = []" "for _ in range(int(sys.argv[1]) // 16):" "    held.append(b'\\x01' * (16 * 1024 * 1024))" \
    "print('allocated', len(held) * 16, 'MiB')"
}

# 挤掉那一例用的作业：起跑时刻写进第一个文件，等一会儿、拿够内存、再占一会儿，收尾时刻写进第二个文件
selftest_job_code() {
  printf '%s\n' "import sys, time" "start_path, end_path = sys.argv[1], sys.argv[2]" \
    "delay_seconds, mebibytes, hold_seconds = float(sys.argv[3]), int(sys.argv[4]), float(sys.argv[5])" \
    "open(start_path, 'w').write(repr(time.time()))" "time.sleep(delay_seconds)" \
    "held = [bytes([1]) * (1024 * 1024) for _ in range(mebibytes)]" "time.sleep(hold_seconds)" \
    "open(end_path, 'w').write(repr(time.time()))"
}

# 自证的收尾（文件头「自证的收尾」）：自证开的 slice 逐个 stop 加 revert，再删自证的临时目录；EXIT trap 与自证末尾都调它，调过一次之后再调什么都不做。
# selftest_slices 与 scratch 是全局的：TERM / INT 打断时 trap 照样读得到
selftest_slices=() scratch=""
selftest_cleanup() {
  local own_slice
  for own_slice in ${selftest_slices[@]+"${selftest_slices[@]}"}; do
    systemctl --user stop "$own_slice" >/dev/null 2>&1
    systemctl --user revert "$own_slice" >/dev/null 2>&1
  done
  selftest_slices=()
  [[ -z "$scratch" ]] || rm -rf -- "${scratch:?}"
  scratch=""
}

run_selftest() {
  local failures=0 checked=0 output status marker hog_code job_code target slice_counter=0 slice context_directory
  local first_pid second_pid first_status second_status started_seconds elapsed_seconds cgroup_path refused_before
  local -a context=()
  # 头一件事挂收尾（文件头「自证的收尾」）：TERM / INT 停与中途 exit 都经 EXIT 走到 selftest_cleanup；弄坏开关 selftest-cleanup-unhooked 下不挂（改前只在末尾停）
  if [[ "$BROKEN_JUDGEMENT" != selftest-cleanup-unhooked ]]; then
    trap selftest_cleanup EXIT
    trap 'exit 143' TERM
    trap 'exit 130' INT
  fi
  target="${RUN_WITH_MEMORY_CAP_SELFTEST_TARGET:-$SELF_PATH}"
  # 自证经服务起时环境里带着服务那一层给的两个标记：各例一律清掉
  unset RUN_WITH_MEMORY_CAP_OUTER_UNIT RUN_WITH_MEMORY_CAP_OUTER_CAP
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/run-with-memory-cap-selftest-XXXXXX")"
  # 自证起的每一份包装的标记（文件头「弄坏开关只在自证里认」）：建在临时目录里、export，收尾随 scratch 删
  printf '%s\n' "$SELFTEST_MARK_CONTENT" > "$scratch/selftest-mark"
  export RUN_WITH_MEMORY_CAP_SELFTEST_MARK="$scratch/selftest-mark"
  local selftest_peak_host=192.0.2.1   # 峰值表第七列：自证各例的多机配置过不了配置判法、认不出这一台，经 RUN_WITH_MEMORY_CAP_SELFTEST_HOST 给（文件头「其余环境变量」）
  hog_code="$(bounded_memory_hog_code)"
  job_code="$(selftest_job_code)"
  fail() { echo "  ✗ 自检：$1"; failures=$((failures + 1)); }   # gate-lint:detail
  # 每一例一个新 slice（名字不带 -：- 是 systemd 的层级）、新的峰值表与状态目录；收尾时 stop 加 revert 掉。XDG_STATE_HOME 也指到这一例底下
  fresh_context() { # fresh_context <总上限>
    slice_counter=$((slice_counter + 1))
    slice="singlefs_memory_selftest_$$_${slice_counter}.slice"
    selftest_slices+=("$slice")
    context_directory="$scratch/context-$slice_counter"
    mkdir -p "$context_directory/temporary-root"
    context=(-u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP
             RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER=selftest-task.1 CAPPED_ADMITTED_BY_TASK_SERVER=selftest-task.1
             RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL="$1" RUN_WITH_MEMORY_CAP_STATE_DIR="$context_directory/state"
             RUN_WITH_MEMORY_CAP_PEAKS="$context_directory/peaks.tsv" RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$context_directory/temporary-root"
             XDG_STATE_HOME="$context_directory/state-home" SINGLEFS_MULTI_HOST_CONFIG="$context_directory/multi-host.env"
             RUN_WITH_MEMORY_CAP_PRIVATE_CONFIG="$context_directory/private-config-absent.env" RUN_WITH_MEMORY_CAP_SELFTEST_HOST="$selftest_peak_host")   # 私有配置在家目录下、按台各一份：自证不读它（第二台的家目录是另一个），要的格自己给
  }
  in_context() { env "${context[@]}" "$@"; }
  peak_row() { # peak_row <键> → 峰值表里这个键那一行「列数|峰值|近几次」；没有就空
    awk -F '\t' -v key="$1" '$4 == key {print NF "|" $1 "|" $6}' "$context_directory/peaks.tsv" 2>/dev/null
  }

  # 「自证开出 slice 之后被停」那两格起的这一趟（文件头「其余环境变量」RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE）：收尾已挂好，开一个真 slice、
  # 把进程号、slice 名与临时目录写进探针文件（先写 .partial 再改名，起它的那一格见到文件就是写全了），然后等着被停；前台的 sleep 每次只睡 0.1 秒，trap 至多晚这么久
  if [[ -n "$SELFTEST_INTERRUPT_PROBE" ]]; then
    fresh_context 256M
    in_context bash "$target" --check 64M >/dev/null 2>"$scratch/interrupt-probe-check.err" \
      || fail "探针这一趟的 --check 64M 应当起得来（slice 才开得出来），实际起不来：$(cat -- "$scratch/interrupt-probe-check.err")"
    { printf 'pid %s\n' "$$"; printf 'slice %s\n' "${selftest_slices[@]}"; printf 'scratch %s\n' "$scratch"; } > "$SELFTEST_INTERRUPT_PROBE.partial"
    mv -f -- "$SELFTEST_INTERRUPT_PROBE.partial" "$SELFTEST_INTERRUPT_PROBE"
    local probe_tick
    for ((probe_tick = 0; probe_tick < SELFTEST_INTERRUPT_PROBE_SECONDS * 10; probe_tick++)); do sleep 0.1; done
    fail "设了 RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE 的这一趟自证应当在 $SELFTEST_INTERRUPT_PROBE_SECONDS 秒里被起它的那一格停掉，实际一直没被停"
    echo "  → 怎么办：这一趟只给「自证开出 slice 之后被 TERM / INT 停时」那两格起；手起的去掉 RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE 再跑 --selftest"
    exit 1
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
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-hog bash "$target" 64M python3 -c "$hog_code" 512 >"$scratch/hog.out" 2>"$scratch/hog.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '撞了内存上限 64M' "$scratch/hog.err"; then
    fail "上限 64M 里分配 512 MiB 的探针应当退 $MEMORY_CAP_HIT_EXIT 并报「撞了内存上限 64M」，实际退 $status：$(cat "$scratch/hog.out" "$scratch/hog.err")"
  fi
  # 撞了自己的上限的那一次记成撞顶（峰值记上限）（文件头「峰值表」一段）
  checked=$((checked + 1))
  output="$(peak_row selftest-hog)"
  [[ "$output" == "7|$((64 * MEBIBYTE))|$((64 * MEBIBYTE)):hit:unreclaimable" ]] \
    || fail "撞了自己的上限的那一次应当在峰值表里记成撞顶、峰值记上限（「7|$((64 * MEBIBYTE))|$((64 * MEBIBYTE)):hit:unreclaimable」），实际「$output」（列数|峰值|近几次）"

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
  output="$(peak_row selftest-stopped-control)"
  if [[ $status -ne 0 || ! "$output" =~ ^7\|[0-9]+\| ]]; then
    fail "对照：整段跑完的命令应当把峰值记进表（selftest-stopped-control 那一行），实际退 $status、记的是「$output」：$(cat "$scratch/stopped-control.err")"
  fi
  local stopped_name stopped_number
  for stopped_name in TERM KILL; do
    stopped_number="$(kill -l "$stopped_name")"
    checked=$((checked + 1))
    if in_context env RUN_WITH_MEMORY_CAP_KEY="selftest-stopped-by-$stopped_name" bash "$target" 64M bash -c "kill -$stopped_name \$\$" 2>"$scratch/stopped-$stopped_name.err"; then status=0; else status=$?; fi
    output="$(peak_row "selftest-stopped-by-$stopped_name")"
    if [[ $status -ne $((128 + stopped_number)) || -n "$output" ]] || ! grep -qF "被外面停了（信号 $stopped_number），这一次的峰值不记" "$scratch/stopped-$stopped_name.err" \
        || ! grep -qF "→ 这一条的结果不算数；查是谁发的信号" "$scratch/stopped-$stopped_name.err"; then
      fail "命令被外面的 $stopped_name 停应当原样退 $((128 + stopped_number))、报「被外面停了（信号 $stopped_number），这一次的峰值不记」与出路、峰值表里不留这个键，实际退 $status、峰值表里「$output」：$(cat "$scratch/stopped-$stopped_name.err")"
    fi
  done

  # 正常退出码原样传回
  local wanted
  for wanted in 0 7 101; do
    checked=$((checked + 1))
    if in_context bash "$target" 64M bash -c "exit $wanted"; then status=0; else status=$?; fi
    [[ $status -eq $wanted ]] || fail "命令退 $wanted 时应当原样传回 $wanted，实际 $status"
  done

  # 显存不够（文件头「显存不够」一段）：假 journalctl 按 journal-mode 文件演：oom 打一行 NVRM … NV_ERR_NO_MEMORY，xid 打一行别的 NVRM，
  # failing 退 1（读不到内核日志）；-n 1（看不看得到内核日志）一律打一行。每次调用的参数记进 journal-args
  fresh_context 512M
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

  # 总上限从哪来（文件头「总上限」）：每一例一个新 slice，环境里的 RUN_WITH_MEMORY_CAP_SLICE_TOTAL 设成空，多机配置指到这一例自己写的那一份
  # ② 配置写 SLICE_MEMORY_CAP_BY_HOST='127.0.0.1=300M 192.0.2.71=1G' ⇒ slice 的 memory.max 是这一台那一项 300M；--status 报总上限从多机配置来、不报余量。
  #   127.0.0.1 在每一台的网卡上（lo），是这一台；192.0.2.71（文档用的地址段）是另一台。值整个带一层引号（配置判法去一层成对引号，这里同读法）
  fresh_context 256M
  printf '%s\n' "# 自证写的多机配置" "HOSTS=127.0.0.1 192.0.2.71" "SLICE_MEMORY_CAP_BY_HOST='127.0.0.1=300M 192.0.2.71=1G'" > "$context_directory/multi-host.env"
  checked=$((checked + 1))
  output="$(in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" 64M bash -c 'group="$(cut -d: -f3 /proc/self/cgroup)"; cat "/sys/fs/cgroup$(dirname "$group")/memory.max"' 2>"$scratch/config-total.err")"
  [[ "$output" == "$((300 * MEBIBYTE))" ]] \
    || fail "总上限照多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（300M）：slice 的 memory.max 应当是 $((300 * MEBIBYTE))，实际「$output」：$(cat "$scratch/config-total.err")"
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" --status >"$scratch/config-status.out" 2>&1; then status=0; else status=$?; fi
  if [[ $status -ne 0 ]] || ! grep -qF "的 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项 300M" "$scratch/config-status.out" || grep -q '余量' "$scratch/config-status.out"; then
    fail "--status 应当报总上限从多机配置来（「… 的 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项 300M」）、退 0、不再报余量，实际退 $status：$(head_characters 600 "$scratch/config-status.out")"
  fi
  # ③ 配置里没有这个键、slice 已有总上限（上一例设的 300M）⇒ 照它走、不改；配置换成写坏的（这一台那一项写法不对、没有这一台那一项）⇒ 退 251、报写法不对、命令没跑、slice 不改
  printf '%s\n' "# 自证写的多机配置，没有 SLICE_MEMORY_CAP_BY_HOST" "HOSTS=127.0.0.1 192.0.2.71" > "$context_directory/multi-host.env"
  checked=$((checked + 1))
  output="$(in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" 64M bash -c 'group="$(cut -d: -f3 /proc/self/cgroup)"; cat "/sys/fs/cgroup$(dirname "$group")/memory.max"' 2>"$scratch/existing-total.err")"
  [[ "$output" == "$((300 * MEBIBYTE))" ]] \
    || fail "多机配置里没有 SLICE_MEMORY_CAP_BY_HOST、slice 已有总上限时应当照它走（memory.max 仍是 $((300 * MEBIBYTE))）、命令照跑，实际「$output」：$(cat "$scratch/existing-total.err")"
  local bad_case bad_name bad_value bad_text
  for bad_case in "syntax|这一台那一项写法不对|127.0.0.1=50 192.0.2.71=1G" "absent|没有这一台那一项|192.0.2.71=1G"; do
    IFS='|' read -r bad_name bad_value bad_text <<<"$bad_case"
    printf '%s\n' "# 自证写的多机配置，SLICE_MEMORY_CAP_BY_HOST $bad_value" "HOSTS=127.0.0.1 192.0.2.71" "SLICE_MEMORY_CAP_BY_HOST=$bad_text" > "$context_directory/multi-host.env"
    checked=$((checked + 1))
    marker="$scratch/ran-with-bad-slice-cap-$bad_name"
    if in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/bad-slice-cap-$bad_name.err"; then status=0; else status=$?; fi
    if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q 'SLICE_MEMORY_CAP_BY_HOST 写法不对' "$scratch/bad-slice-cap-$bad_name.err"; then
      fail "多机配置里 SLICE_MEMORY_CAP_BY_HOST $bad_value时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「SLICE_MEMORY_CAP_BY_HOST 写法不对」、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/bad-slice-cap-$bad_name.err")"
    fi
  done
  # 按台取：头一项是另一台（HOSTS 与 SLICE_MEMORY_CAP_BY_HOST 都把 192.0.2.71 写在前面）⇒ 总上限照这一台那一项 320M，不取头一项的 1G（新 slice）
  fresh_context 256M
  printf '%s\n' "# 自证写的多机配置，头一项是另一台" "HOSTS=192.0.2.71 127.0.0.1" "SLICE_MEMORY_CAP_BY_HOST=192.0.2.71=1G 127.0.0.1=320M" > "$context_directory/multi-host.env"
  checked=$((checked + 1))
  output="$(in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" 64M bash -c 'group="$(cut -d: -f3 /proc/self/cgroup)"; cat "/sys/fs/cgroup$(dirname "$group")/memory.max"' 2>"$scratch/config-by-host.err")"
  [[ "$output" == "$((320 * MEBIBYTE))" ]] \
    || fail "SLICE_MEMORY_CAP_BY_HOST 里头一项是另一台时总上限照这一台那一项：slice 的 memory.max 应当是 $((320 * MEBIBYTE))（320M），实际「$output」：$(cat "$scratch/config-by-host.err")"
  # 三样都没有：新 slice、配置不在 ⇒ 退 251、出路写加 SLICE_MEMORY_CAP_BY_HOST、命令没跑
  fresh_context 256M
  checked=$((checked + 1))
  marker="$scratch/ran-without-total"
  if in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/no-total.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q '写一行 SLICE_MEMORY_CAP_BY_HOST=' "$scratch/no-total.err"; then
    fail "三样都定不出总上限时（没给 RUN_WITH_MEMORY_CAP_SLICE_TOTAL、多机配置不在、slice 是新的）应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、出路写在多机配置里写一行 SLICE_MEMORY_CAP_BY_HOST=、命令不跑，\
实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/no-total.err")"
  fi

  # slice 上别的版本留下的 memory.high（文件头「总上限」memory.high 那一段）：不碰真的 slice。假 systemctl 只管这一例的 slice：
  # show -p ControlGroup 答假 cgroup 根下的 /selftest-heavy.slice（起跑标记 <目录>.started 在才答，否则答空 = slice 不在）、set-property MemoryMax 与 MemoryHigh=infinity 写进假目录
  # （FAKE_MEMORY_HIGH_STUCK=1 时 MemoryHigh 写不进去）、start 建起跑标记（给了 FAKE_MEMORY_HIGH_AFTER_START 就把 memory.high 换成它：drop-in 起来就带着）；别的调用交给真的。
  # 假 cgroup 根里 user.slice 是指到真 /sys/fs/cgroup/user.slice 的符号链接：scope 由真的 systemd-run 起，外壳与采样子进程照读真的 scope 目录。
  # 五例：留着 47.5G（51002736640）、已是 max、slice 不在（起来之后是 max）、slice 不在而起来之后带着 47.5G、清了读回仍是 47.5G
  local fake_high_root fake_high_slice fake_high_bin fake_high_case fake_high_before fake_high_after fake_high_stuck high_calls high_now
  local leftover_high=51002736640
  fake_high_root="$scratch/fake-memory-high-root" fake_high_bin="$scratch/fake-memory-high-bin"
  fake_high_slice="$fake_high_root/selftest-heavy.slice"
  mkdir -p "$fake_high_bin" "$fake_high_slice"
  ln -s /sys/fs/cgroup/user.slice "$fake_high_root/user.slice"
  cat > "$fake_high_bin/systemctl" <<'FAKE_MEMORY_HIGH_SYSTEMCTL'
#!/usr/bin/env bash
directory="$FAKE_MEMORY_HIGH_DIRECTORY" slice="$FAKE_MEMORY_HIGH_SLICE"
if [[ "${1:-}" == --user && "${*: -1}" == "$slice" ]]; then
  printf '%s\n' "$*" >> "$FAKE_MEMORY_HIGH_CALLS"
  case "${2:-} ${3:-}" in
    "show -p")
      if [[ "$*" == *ControlGroup* ]]; then
        [[ -e "$directory.started" ]] && printf '/%s\n' "${directory##*/}"
        exit 0
      fi ;;
    "start $slice")
      : > "$directory.started"
      [[ -z "${FAKE_MEMORY_HIGH_AFTER_START:-}" ]] || printf '%s\n' "$FAKE_MEMORY_HIGH_AFTER_START" > "$directory/memory.high"
      exit 0 ;;
  esac
fi
if [[ "${1:-} ${2:-} ${3:-} ${4:-}" == "--user set-property --runtime $slice" ]]; then
  printf '%s\n' "$*" >> "$FAKE_MEMORY_HIGH_CALLS"
  case "${5:-}" in
    MemoryMax=*) printf '%s\n' "${5#MemoryMax=}" > "$directory/memory.max" ;;
    MemoryHigh=infinity) [[ -n "${FAKE_MEMORY_HIGH_STUCK:-}" ]] || printf 'max\n' > "$directory/memory.high" ;;
    *) exit 1 ;;
  esac
  exit 0
fi
exec "$FAKE_SYSTEMCTL_REAL" "$@"
FAKE_MEMORY_HIGH_SYSTEMCTL
  chmod +x "$fake_high_bin/systemctl"
  for fake_high_case in "leftover|started|$leftover_high||" "already-max|started|max||" "absent|absent|max||" \
      "absent-then-leftover|absent|max|$leftover_high|" "stuck|started|$leftover_high||1"; do
    IFS='|' read -r fake_high_case fake_high_before high_now fake_high_after fake_high_stuck <<< "$fake_high_case"
    fresh_context 256M
    rm -f -- "$fake_high_slice.started" "$scratch/memory-high-calls"
    printf '%s\n' $((256 * MEBIBYTE)) > "$fake_high_slice/memory.max"
    printf '%s\n' "$high_now" > "$fake_high_slice/memory.high"
    [[ "$fake_high_before" != started ]] || : > "$fake_high_slice.started"
    marker="$scratch/memory-high-$fake_high_case-ran"
    rm -f -- "$marker"
    checked=$((checked + 1))
    if in_context env PATH="$fake_high_bin:$PATH" FAKE_SYSTEMCTL_REAL="$(command -v systemctl)" FAKE_MEMORY_HIGH_SLICE="$slice" FAKE_MEMORY_HIGH_DIRECTORY="$fake_high_slice" \
        FAKE_MEMORY_HIGH_CALLS="$scratch/memory-high-calls" FAKE_MEMORY_HIGH_AFTER_START="$fake_high_after" FAKE_MEMORY_HIGH_STUCK="$fake_high_stuck" \
        RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$fake_high_root" RUN_WITH_MEMORY_CAP_KEY="selftest-memory-high-$fake_high_case" \
        bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/memory-high-$fake_high_case.err"; then status=0; else status=$?; fi
    high_now="" high_calls=""
    IFS= read -r high_now < "$fake_high_slice/memory.high"
    high_calls="$(grep -c 'MemoryHigh=infinity' "$scratch/memory-high-calls" 2>/dev/null)"
    high_calls="${high_calls:-0}"
    case "$fake_high_case" in
      leftover|absent-then-leftover)
        if [[ $status -ne 0 || ! -e "$marker" || "$high_now" != max || "$high_calls" != 1 ]] || ! grep -qF '清回 max' "$scratch/memory-high-$fake_high_case.err"; then
          fail "$([[ "$fake_high_case" == leftover ]] && echo 'slice 上留着软上限 47.5G 时' || echo 'slice 原先不在、起来之后带着软上限 47.5G 时')应当 set-property MemoryHigh=infinity 一次、\
读回 memory.high 是 max、stderr 打「清回 max」、命令照跑退 0，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')、memory.high「$high_now」、\
MemoryHigh=infinity 调了 ${high_calls:-0} 次：$(head_characters 600 "$scratch/memory-high-$fake_high_case.err")"
        fi ;;
      already-max|absent)
        if [[ $status -ne 0 || ! -e "$marker" || "$high_now" != max || "$high_calls" != 0 ]] || grep -qF '清回 max' "$scratch/memory-high-$fake_high_case.err"; then
          fail "$([[ "$fake_high_case" == absent ]] && echo 'slice 原先不在（起来之后 memory.high 是 max）时' || echo 'slice 的 memory.high 已是 max 时')应当不调 set-property MemoryHigh、\
不打「清回 max」、命令照跑退 0，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')、memory.high「$high_now」、MemoryHigh=infinity 调了 ${high_calls:-0} 次：\
$(head_characters 600 "$scratch/memory-high-$fake_high_case.err")"
        fi ;;
      stuck)
        if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -qF '不是 max；命令一行都没跑' "$scratch/memory-high-$fake_high_case.err" \
            || ! grep -qF '→ 怎么办：systemctl --user cat' "$scratch/memory-high-$fake_high_case.err"; then
          fail "清软上限之后读回不是 max 时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「不是 max；命令一行都没跑」与查 drop-in 的出路、命令不跑，\
实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(head_characters 600 "$scratch/memory-high-$fake_high_case.err")"
        fi ;;
    esac
  done
  rm -f -- "$fake_high_root/user.slice"

  # systemd-run 起不来：退 251、报错，而且那条命令一行都没跑（不退回无上限）
  fresh_context 256M
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

  # 被总上限挤掉（第 ② 层兜住、退 254，不报成撞了自己的上限）：两条各拿 150 MiB，上限各 200M，合起来超总上限 256M；
  # 挤掉的那一次峰值不记（量到的是半截），没挤掉的那一条整段跑完照记
  fresh_context 256M
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-squeeze-1 bash "$target" 200M python3 -c "$job_code" "$scratch/squeeze-1.start" "$scratch/squeeze-1.end" 0.3 150 1.5 \
    >/dev/null 2>"$scratch/squeeze-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-squeeze-2 bash "$target" 200M python3 -c "$job_code" "$scratch/squeeze-2.start" "$scratch/squeeze-2.end" 0.3 150 1.5 \
    >/dev/null 2>"$scratch/squeeze-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne $SLICE_TOTAL_HIT_EXIT && $second_status -ne $SLICE_TOTAL_HIT_EXIT ]] || [[ $first_status -eq $MEMORY_CAP_HIT_EXIT || $second_status -eq $MEMORY_CAP_HIT_EXIT ]] \
      || ! grep -q '总上限挤掉' "$scratch/squeeze-1.err" "$scratch/squeeze-2.err"; then
    fail "两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 $SLICE_TOTAL_HIT_EXIT、报「被总上限挤掉」，实际退 $first_status 与 $second_status：$(cat "$scratch/squeeze-1.err" "$scratch/squeeze-2.err")"
  fi
  checked=$((checked + 1))
  local squeezed_key=""
  [[ $first_status -eq $SLICE_TOTAL_HIT_EXIT ]] && squeezed_key=selftest-squeeze-1
  [[ $second_status -eq $SLICE_TOTAL_HIT_EXIT ]] && squeezed_key=selftest-squeeze-2
  if [[ -n "$squeezed_key" ]]; then
    output="$(peak_row "$squeezed_key")"
    [[ -z "$output" ]] || fail "被总上限挤掉的那一次（$squeezed_key）不该记进峰值表（量到的是半截），实际「$output」"
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
  output="$(peak_row selftest-term-storm)"
  if [[ ! "$output" =~ ^7\|[0-9]+\| ]]; then
    fail "scope 里一直有进程收 TERM 时，外壳照样要把峰值记进表（selftest-term-storm 那一行），实际退 $status、记的是「$output」：$(cat "$scratch/term-storm.err")"
  fi

  # 起了 scope 之后包装收到 TERM / INT（欠账 C801）：先 systemctl --user stop 自己开的那个 scope 单元（在删标记之前），scope 里的命令随之停，
  # 包装退 143 / 130，标记与 scope 都不留。systemctl 是 PATH 前面的一层转接：记下每一次 stop（单元名、那一刻标记还在不在），再交给真的；
  # 包装经一层 python 起，把 INT 设回默认。命令是 exec sleep 60：没被停的话它一直活着（弄坏开关 term-leaves-scope）；
  # 这一例收尾自己停自己的 slice，不让它留到最后。信号只发给这里 & 起、$! 记下的那个包装。
  scope_stop_process_is_live() { # scope_stop_process_is_live <进程号>：在跑（没有、或已是僵尸都算没在跑）交回 0
    local stat_line
    IFS= read -r stat_line 2>/dev/null < "/proc/$1/stat" || return 1
    stat_line="${stat_line##*) }"
    [[ "${stat_line%% *}" != Z ]]
  }
  scope_stop_case() { # scope_stop_case <TERM|INT> <包装该退的码>
    local stop_signal="$1" expected_status="$2" case_directory="$scratch/scope-stop-$1" wrapper_pid command_pid="" poll wrapper_status=0
    local ready=0 wrapper_exited=0 command_exited=0 scope_left="" marker_left="" stop_line=""
    mkdir -p "$case_directory/bin" "$case_directory/tmp"
    : > "$case_directory/calls"
    printf '%s\n' '#!/usr/bin/env bash' \
      'if [[ "${1:-}" == --user && "${2:-}" == stop ]]; then' \
      '  marker=absent' \
      '  compgen -G "$STUB_TMPDIR/run-with-memory-cap-*.started" >/dev/null && marker=present' \
      '  echo "stop $3 marker=$marker" >> "$STUB_CALLS"' \
      'fi' \
      'exec "$STUB_REAL_SYSTEMCTL" "$@"' > "$case_directory/bin/systemctl"
    chmod +x "$case_directory/bin/systemctl"
    python3 -c 'import os, signal, sys; signal.signal(signal.SIGINT, signal.SIG_DFL); signal.signal(signal.SIGHUP, signal.SIG_DFL); os.execvp(sys.argv[1], sys.argv[1:])' \
      env "${context[@]}" PATH="$case_directory/bin:$PATH" TMPDIR="$case_directory/tmp" STUB_TMPDIR="$case_directory/tmp" STUB_CALLS="$case_directory/calls" \
      STUB_REAL_SYSTEMCTL="$(command -v systemctl)" RUN_WITH_MEMORY_CAP_KEY="selftest-scope-stop-$stop_signal" \
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
    stop_line="$(grep -E "^stop singlefs-memory-cap-$wrapper_pid-[0-9]+\.scope marker=present\$" "$case_directory/calls" | head -n 1)"
    systemctl --user stop "$slice" >/dev/null 2>&1
    if ((ready == 0)); then
      fail "起了 scope 之后给包装发 $stop_signal 这一例没判成：命令 15 秒里没在 scope 里起来，包装退 $wrapper_status：$(cat "$case_directory/output")"
    elif [[ -z "$stop_line" || $wrapper_exited -eq 0 || $command_exited -eq 0 || $wrapper_status -ne $expected_status || -n "$scope_left$marker_left" ]]; then
      fail "起了 scope 之后给包装发 $stop_signal：应当先 systemctl --user stop 自己开的 scope 单元（那一刻标记还在）、scope 里的命令 5 秒内随之停、包装 10 秒内退 $expected_status、标记与 scope 都不留；\
实际 stop 调用「$(cat "$case_directory/calls")」、包装$([[ $wrapper_exited -eq 1 ]] && echo '退了' || echo '10 秒里没退')（$wrapper_status）、\
命令 ${command_pid:-没记下}$([[ $command_exited -eq 1 ]] && echo ' 停了' || echo ' 5 秒里没停')、留下的 scope「${scope_left}」标记「${marker_left}」：$(cat "$case_directory/output")"
    fi
  }
  fresh_context 256M
  checked=$((checked + 1))
  scope_stop_case TERM 143
  fresh_context 256M
  checked=$((checked + 1))
  scope_stop_case INT 130

  # 自证开出 slice 之后被停（文件头「自证的收尾」）：起一趟设了 RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE 的自证（经一层 python 把 INT 设回默认，& 起的作业 INT 默认被忽略），
  # 等它开出一个真 slice、写下探针文件，核那个 slice 此刻是 active（不然这一格什么都没证），只给 & 起、$! 记下的它发 TERM / INT，10 秒不退再发 KILL；
  # 退了之后核 systemctl --user list-units --all 里 singlefs_memory_selftest_<它的进程号>_*.slice 一个都不是 active、它的临时目录不在。
  # 弄坏开关只传 selftest-cleanup-unhooked 给它（别的开关不传：只红在点名那几格）；它没收干净的，这里替它停、删，slice 记进 selftest_slices 由这一趟收尾再 stop 加 revert
  interrupt_active_slices() { # interrupt_active_slices <那一趟自证的进程号> → 它开的 slice 里此刻 active 的，各带一个前导空格
    systemctl --user list-units --all --plain --no-legend --type=slice "singlefs_memory_selftest_$1_*.slice" 2>/dev/null | awk '$3 == "active" {printf " %s", $1}'
  }
  interrupt_case() { # interrupt_case <TERM|INT> <那一趟自证该退的码>
    local interrupt_signal="$1" expected_status="$2" probe="$scratch/interrupt-probe-$1" probe_pid poll probe_status=0 ready=0 probe_exited=0
    local probe_key probe_value probe_slices="" probe_scratch="" probe_recorded_pid="" active_before="" active_after="" probe_slice
    rm -f -- "$probe" "$probe.partial"
    python3 -c 'import os, signal, sys; signal.signal(signal.SIGINT, signal.SIG_DFL); os.execvp(sys.argv[1], sys.argv[1:])' \
      env RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE="$probe" \
      RUN_WITH_MEMORY_CAP_BREAK="$([[ "$BROKEN_JUDGEMENT" == selftest-cleanup-unhooked ]] && printf %s "$BROKEN_JUDGEMENT")" \
      bash "$target" --selftest </dev/null >"$scratch/interrupt-$interrupt_signal.out" 2>&1 &
    probe_pid=$!
    for ((poll = 0; poll < 300; poll++)); do   # 至多 30 秒等它写下探针文件
      if [[ -e "$probe" ]]; then ready=1; break; fi
      scope_stop_process_is_live "$probe_pid" || break
      sleep 0.1
    done
    if ((ready)); then
      while read -r probe_key probe_value; do
        case "$probe_key" in
          pid) probe_recorded_pid="$probe_value" ;;
          slice) probe_slices+=" $probe_value" ;;
          scratch) probe_scratch="$probe_value" ;;
        esac
      done < "$probe"
      active_before="$(interrupt_active_slices "$probe_pid")"
      kill -"$interrupt_signal" "$probe_pid"
    fi
    for ((poll = 0; poll < 100; poll++)); do   # 至多 10 秒等它退
      scope_stop_process_is_live "$probe_pid" || { probe_exited=1; break; }
      sleep 0.1
    done
    ((probe_exited)) || kill -KILL "$probe_pid" 2>/dev/null
    wait "$probe_pid"; probe_status=$?
    active_after="$(interrupt_active_slices "$probe_pid")"
    # 它没收干净的替它收：slice 记进这一趟的 selftest_slices 并当场停；临时目录只删 mktemp 起的那种名字
    for probe_slice in $probe_slices $active_after; do
      selftest_slices+=("$probe_slice")
      systemctl --user stop "$probe_slice" >/dev/null 2>&1
    done
    [[ "$probe_scratch" == */run-with-memory-cap-selftest-* && -d "$probe_scratch" ]] && rm -rf -- "${probe_scratch:?}" && probe_scratch+="（还在，这里删了）"
    if ((ready == 0)); then
      fail "自证开出 slice 之后被 $interrupt_signal 停这一格没判成：那一趟自证 30 秒里没写下探针文件，退 $probe_status：$(head_characters 600 "$scratch/interrupt-$interrupt_signal.out")"
    elif [[ "$probe_recorded_pid" != "$probe_pid" || -z "$active_before" ]]; then
      fail "自证开出 slice 之后被 $interrupt_signal 停这一格没判成：探针文件里的进程号「$probe_recorded_pid」应当是起的那一个 $probe_pid、被停之前它开的 slice（${probe_slices# }）应当有 active 的，\
实际 active 的「${active_before# }」：$(head_characters 600 "$scratch/interrupt-$interrupt_signal.out")"
    elif [[ $probe_exited -eq 0 || $probe_status -ne $expected_status || -n "$active_after" || "$probe_scratch" == *还在* ]]; then
      fail "自证开出 slice 之后被 $interrupt_signal 停时应当 10 秒内退 $expected_status、它开的 slice（${probe_slices# }）都停掉不再 active、它的临时目录删掉；\
实际$([[ $probe_exited -eq 1 ]] && echo '退了' || echo '10 秒里没退、发了 KILL')（$probe_status）、还 active 的「${active_after# }」、临时目录「${probe_scratch:-没记下}」：$(head_characters 600 "$scratch/interrupt-$interrupt_signal.out")"
    fi
  }
  checked=$((checked + 1))
  interrupt_case TERM 143
  checked=$((checked + 1))
  interrupt_case INT 130

  # 限时：超过 RUN_WITH_MEMORY_CAP_TIME_LIMIT 退 253，不跑满命令自己的时长
  fresh_context 256M
  checked=$((checked + 1))
  started_seconds="$(date +%s%N)"
  if in_context env RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 RUN_WITH_MEMORY_CAP_KILL_GRACE=2 bash "$target" 64M sleep 4 2>"$scratch/timelimit.err"; then status=0; else status=$?; fi
  elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
  if [[ $status -ne $TIME_LIMIT_HIT_EXIT || $elapsed_seconds -ge 4 ]] || ! grep -q '超过限时 1 秒' "$scratch/timelimit.err"; then
    fail "限时 1 秒跑 sleep 4 应当 4 秒之内退 $TIME_LIMIT_HIT_EXIT、报「超过限时 1 秒」，实际退 $status、用了 $elapsed_seconds 秒：$(cat "$scratch/timelimit.err")"
  fi

  # 峰值表的口径（文件头「峰值表」一段）：采样子进程改采假 cgroup 目录（RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLED_CGROUP），收尾时 memory.peak 与 oom_kill 也读它的；命令是 sleep 0.5，scope 的 Result=success。
  # 页缓存涨满上限：memory.current 等于上限 64M、active_file + inactive_file 占 48M，memory.peak 64M、oom_kill 0 ⇒ 记回收不掉的 16M、clean；
  # 内核在 scope 里杀过进程而 Result 没报 oom-kill：memory.current 40M、页缓存 10M，memory.peak 40M、oom_kill 1 ⇒ 记成撞了顶、峰值记上限 64M
  local fake_cgroup sampled_status case_label
  fresh_context 256M
  for case_label in page-cache oom-kill; do
    fake_cgroup="$scratch/fake-cgroup-$case_label"
    mkdir -p "$fake_cgroup"
    if [[ "$case_label" == page-cache ]]; then
      printf '%s\n' $((64 * MEBIBYTE)) > "$fake_cgroup/memory.current"
      printf 'anon %s\nfile %s\nkernel 0\nactive_file %s\ninactive_file %s\nshmem 0\n' $((16 * MEBIBYTE)) $((48 * MEBIBYTE)) $((32 * MEBIBYTE)) $((16 * MEBIBYTE)) > "$fake_cgroup/memory.stat"
      printf '%s\n' $((64 * MEBIBYTE)) > "$fake_cgroup/memory.peak"
      printf 'low 0\nhigh 0\nmax 40\noom 0\noom_kill 0\n' > "$fake_cgroup/memory.events"
    else
      printf '%s\n' $((40 * MEBIBYTE)) > "$fake_cgroup/memory.current"
      printf 'anon %s\nfile %s\nkernel 0\nactive_file %s\ninactive_file %s\nshmem 0\n' $((30 * MEBIBYTE)) $((10 * MEBIBYTE)) $((6 * MEBIBYTE)) $((4 * MEBIBYTE)) > "$fake_cgroup/memory.stat"
      printf '%s\n' $((40 * MEBIBYTE)) > "$fake_cgroup/memory.peak"
      printf 'low 0\nhigh 0\nmax 3\noom 1\noom_kill 1\n' > "$fake_cgroup/memory.events"
    fi
    if in_context env RUN_WITH_MEMORY_CAP_KEY="selftest-sampled-$case_label" RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLED_CGROUP="$fake_cgroup" bash "$target" 64M sleep 0.5 \
        2>"$scratch/sampled-$case_label.err"; then sampled_status=0; else sampled_status=$?; fi
    output="$(peak_row "selftest-sampled-$case_label")"
    checked=$((checked + 1))
    if [[ "$case_label" == page-cache ]]; then
      [[ $sampled_status -eq 0 && "$output" == "7|$((16 * MEBIBYTE))|$((16 * MEBIBYTE)):clean:unreclaimable" ]] \
        || fail "页缓存涨满上限（memory.current 等于上限 64M、其中 48M 是页缓存，memory.peak 64M、oom_kill 0）的那一趟应当退 0、峰值表记回收不掉的 16M 且 clean（「7|$((16 * MEBIBYTE))|$((16 * MEBIBYTE)):clean:unreclaimable」），不记上限，\
实际退 $sampled_status、记的「$output」（列数|峰值|近几次）：$(cat "$scratch/sampled-$case_label.err")"
    else
      [[ $sampled_status -eq 0 && "$output" == "7|$((64 * MEBIBYTE))|$((64 * MEBIBYTE)):hit:unreclaimable" ]] \
        || fail "scope 的 memory.events 里 oom_kill 1 而 Result 没报 oom-kill（memory.peak 40M、回收不掉的 30M，没到上限 64M）的那一趟应当退 0、峰值表记成撞了顶、峰值记上限（「7|$((64 * MEBIBYTE))|$((64 * MEBIBYTE)):hit:unreclaimable」），\
实际退 $sampled_status、记的「$output」（列数|峰值|近几次）：$(cat "$scratch/sampled-$case_label.err")"
    fi
  done

  # 采样子进程不拖延收尾（文件头「峰值表」一段）：命令结束、超时、撞顶、被停四种收尾，包装退出那一刻它起的采样子进程已经停了，也不是等包装没了才自己退。
  # RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE：包装起采样子进程时记「sampler <进程号> <起始时刻>」，采样子进程自己退时记「exit command-ended」或「exit wrapper-gone」（被包装停的不记）。
  # 被停那一例：命令在 scope 里起来（写下 ran 文件）、记录里有 sampler 那一行之后，只给 & 起、$! 记下的包装（env 直接 exec 成包装）发 TERM
  local sampler_case sampler_label sampler_trace sampler_expected sampler_status sampler_pid sampler_start sampler_now sampler_stat sampler_problem sampler_wrapper_pid sampler_poll
  local -a sampler_line=() sampler_fields=()
  for sampler_case in command-ended time-limit cap-hit stopped; do
    fresh_context 256M
    sampler_trace="$scratch/sampler-$sampler_case.trace"
    checked=$((checked + 1))
    case "$sampler_case" in
      command-ended)
        sampler_label="命令结束（sleep 0.5 退 0）" sampler_expected=0
        if in_context env RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE="$sampler_trace" bash "$target" 64M sleep 0.5 \
            2>"$scratch/sampler-$sampler_case.err"; then sampler_status=0; else sampler_status=$?; fi ;;
      time-limit)
        sampler_label="超时（限时 1 秒跑 sleep 30，退 $TIME_LIMIT_HIT_EXIT）" sampler_expected=$TIME_LIMIT_HIT_EXIT
        if in_context env RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE="$sampler_trace" RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 RUN_WITH_MEMORY_CAP_KILL_GRACE=2 bash "$target" 64M sleep 30 \
            2>"$scratch/sampler-$sampler_case.err"; then sampler_status=0; else sampler_status=$?; fi ;;
      cap-hit)
        sampler_label="撞顶（上限 64M 里分配 512 MiB，退 $MEMORY_CAP_HIT_EXIT）" sampler_expected=$MEMORY_CAP_HIT_EXIT
        if in_context env RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE="$sampler_trace" bash "$target" 64M python3 -c "$hog_code" 512 \
            >/dev/null 2>"$scratch/sampler-$sampler_case.err"; then sampler_status=0; else sampler_status=$?; fi ;;
      stopped)
        sampler_label="被停（命令在 scope 里跑着时给包装发 TERM，退 143）" sampler_expected=143
        env "${context[@]}" RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE="$sampler_trace" bash "$target" 64M bash -c ': > "$1"; sleep 30' sampler-stopped "$scratch/sampler-stopped.ran" \
          >/dev/null 2>"$scratch/sampler-$sampler_case.err" &
        sampler_wrapper_pid=$!
        for ((sampler_poll = 0; sampler_poll < 150; sampler_poll++)); do   # 至多 15 秒等命令在 scope 里起来、记录里有 sampler 那一行
          [[ -e "$scratch/sampler-stopped.ran" ]] && grep -q '^sampler ' "$sampler_trace" 2>/dev/null && break
          sleep 0.1
        done
        kill -TERM "$sampler_wrapper_pid"
        wait "$sampler_wrapper_pid"; sampler_status=$? ;;
    esac
    sampler_problem="" sampler_line=() sampler_now=""
    read -r -a sampler_line 2>/dev/null < <(grep -m 1 '^sampler ' "$sampler_trace" 2>/dev/null)
    sampler_pid="${sampler_line[1]:-}" sampler_start="${sampler_line[2]:-}"
    if [[ ! "$sampler_pid" =~ ^[0-9]+$ ]]; then
      sampler_problem+="记录里没有 sampler 那一行（包装没起采样子进程）；"
    elif IFS= read -r sampler_stat 2>/dev/null < "/proc/$sampler_pid/stat"; then
      sampler_fields=()
      read -r -a sampler_fields <<<"${sampler_stat##*) }"
      [[ "${sampler_fields[0]:-}" == Z ]] || sampler_now="${sampler_fields[19]:-}"
      [[ -z "$sampler_now" || "$sampler_now" != "$sampler_start" ]] || sampler_problem+="包装退出之后采样子进程 $sampler_pid 还在；"
    fi
    ! grep -q '^exit wrapper-gone' "$sampler_trace" 2>/dev/null || sampler_problem+="采样子进程等包装没了才自己退（记录里有 exit wrapper-gone）；"
    [[ -z "$(compgen -G "$context_directory/state/peaks/*.unreclaimable*")" ]] \
      || sampler_problem+="状态目录里留下了采样的输出（$(cd "$context_directory/state/peaks" 2>/dev/null && compgen -G '*.unreclaimable*' | tr '\n' ' ')）；"
    [[ $sampler_status -eq $sampler_expected ]] || sampler_problem+="包装退 $sampler_status（要 $sampler_expected）；"
    [[ "$sampler_case" != stopped || -e "$scratch/sampler-stopped.ran" ]] || sampler_problem+="命令 15 秒里没在 scope 里起来（发 TERM 时还没写下 ran 文件）；"
    [[ -z "$sampler_problem" ]] \
      || fail "采样子进程不拖延收尾：$sampler_label，包装退出那一刻采样子进程应当已经被包装停了、不留到包装退出之后，实际：$sampler_problem记录「$(tr '\n' ' ' 2>/dev/null < "$sampler_trace")」：$(head_characters 600 "$scratch/sampler-$sampler_case.err")"
  done

  # 撞顶的 scope 结局读早了（文件头「撞顶的 scope 结局读早了」）：假的 systemctl 对「查 scope 结局」的 show 头 N 次报「已收尾、Result=success」（读早了），
  # lag 那一格之后报 failed / oom-kill（systemd 后来记上了），stuck 那一格一直是 success（systemd 没记上）；
  # 头 3 次读早了的（lag）要在再查的几次里落到 oom-kill，退 250 报撞了上限、另报「读早了」一行；一直读早了的（stuck，自证把再查的缩到 20 次）按 memory.events 判，
  # 同样退 250、另报「一直是 success」一行；两样都不报成「被外面停了」。
  local fake_directory real_systemctl settle_name settle_unit
  fake_directory="$scratch/fake-systemctl"
  real_systemctl="$(command -v systemctl)"
  mkdir -p "$fake_directory"
  cat > "$fake_directory/systemctl" <<'FAKE_SYSTEMCTL'
#!/usr/bin/env bash
# 假 systemctl：查 scope 结局的那种 show（带 -p Result、末项是 <单元>.scope）设了 FAKE_SYSTEMCTL_RESULT_RESET 的照真的答、Result 换成 success；否则头 FAKE_SYSTEMCTL_LIES 次报「已收尾、Result=success」（读早了），之后报 FAKE_SYSTEMCTL_THEN 点名的 Result（没给就和真的一样）；别的调用原样交给真的
last="${*: -1}"
if [[ -n "${FAKE_SYSTEMCTL_RESULT_RESET:-}" && "${1:-} ${2:-}" == "--user show" && "$*" == *"-p Result"* && "$last" == *.scope ]]; then
  "${FAKE_SYSTEMCTL_REAL:?}" "$@" | sed 's/^Result=.*/Result=success/'
  exit 0
fi
if [[ "${1:-} ${2:-}" == "--user show" && "$*" == *"-p Result"* && "$last" == *.scope ]]; then
  count=0
  [[ -f "${FAKE_SYSTEMCTL_COUNT:?}" ]] && read -r count < "$FAKE_SYSTEMCTL_COUNT"
  count=$((count + 1))
  printf '%s\n' "$count" > "$FAKE_SYSTEMCTL_COUNT"
  if ((count <= ${FAKE_SYSTEMCTL_LIES:?})); then printf 'LoadState=loaded\nActiveState=inactive\nResult=success\n'; exit 0; fi
  if [[ -n "${FAKE_SYSTEMCTL_THEN:-}" ]]; then printf 'LoadState=loaded\nActiveState=failed\nResult=%s\n' "$FAKE_SYSTEMCTL_THEN"; exit 0; fi
fi
exec "${FAKE_SYSTEMCTL_REAL:?}" "$@"
FAKE_SYSTEMCTL
  chmod +x "$fake_directory/systemctl"
  for settle_name in lag stuck; do
    fresh_context 256M
    checked=$((checked + 1))
    if [[ "$settle_name" == lag ]]; then
      in_context env PATH="$fake_directory:$PATH" FAKE_SYSTEMCTL_REAL="$real_systemctl" FAKE_SYSTEMCTL_COUNT="$fake_directory/count-$settle_name" FAKE_SYSTEMCTL_LIES=3 FAKE_SYSTEMCTL_THEN=oom-kill \
        bash "$target" 64M python3 -c "$hog_code" 512 >/dev/null 2>"$scratch/settle-$settle_name.err"; status=$?
    else
      in_context env PATH="$fake_directory:$PATH" FAKE_SYSTEMCTL_REAL="$real_systemctl" FAKE_SYSTEMCTL_COUNT="$fake_directory/count-$settle_name" FAKE_SYSTEMCTL_LIES=100000 \
        RUN_WITH_MEMORY_CAP_SELFTEST_LATE_ATTEMPTS=20 bash "$target" 64M python3 -c "$hog_code" 512 >/dev/null 2>"$scratch/settle-$settle_name.err"; status=$?
    fi
    # 假的把真的 reset-failed 那一步挡掉了（stuck 那一格从头到尾挡着）：撞顶的 scope 停在 failed，收尾时替它清
    settle_unit="$(grep -o 'singlefs-memory-cap-[0-9]*-[0-9]*' "$scratch/settle-$settle_name.err" | head -n 1)"
    [[ -z "$settle_unit" ]] || systemctl --user reset-failed "$settle_unit.scope" >/dev/null 2>&1
    if [[ "$settle_name" == lag ]]; then
      if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '撞了内存上限 64M' "$scratch/settle-lag.err" || ! grep -q 'Result 读早了读到 success' "$scratch/settle-lag.err" \
          || grep -q '被外面停了' "$scratch/settle-lag.err"; then
        fail "撞顶的 scope 被读早了（头 3 次查 Result 都报已收尾、success，真的是 oom-kill）时，包装应当再查几次、退 $MEMORY_CAP_HIT_EXIT 报「撞了内存上限 64M」并另报「Result 读早了读到 success」、不报「被外面停了」，实际退 $status：$(tr '\n' ' ' < "$scratch/settle-lag.err")"
      fi
    elif [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '撞了内存上限 64M' "$scratch/settle-stuck.err" || ! grep -q 'Result 一直是 success' "$scratch/settle-stuck.err" \
        || grep -q '被外面停了' "$scratch/settle-stuck.err"; then
      fail "撞顶的 scope 的 Result 一直读到 success，而它自己的 memory.events 里 oom_kill 不为 0 时，应当按 memory.events 判、退 $MEMORY_CAP_HIT_EXIT 报「撞了内存上限 64M」并另报「Result 一直是 success」、不报「被外面停了」，实际退 $status：$(tr '\n' ' ' < "$scratch/settle-stuck.err")"
    fi
  done
  # failed 而 Result 读到 success（用户级 systemd 重载过配置之后）：假 systemctl 把真 scope 的 Result 换成 success、ActiveState 照真的（failed）。
  # reset-timeout：限时 1 秒被停，journal 里有 Failed with result 'timeout'，应当照限时被停判（退 253）；reset-unknown：journalctl 换成读不了的，原因取不到，应当写明、不冒充限时被停
  printf '#!/usr/bin/env bash\nexit 1\n' > "$fake_directory/journalctl-broken"
  chmod +x "$fake_directory/journalctl-broken"
  for settle_name in reset-timeout reset-unknown; do
    fresh_context 256M
    checked=$((checked + 1))
    if [[ "$settle_name" == reset-timeout ]]; then
      in_context env PATH="$fake_directory:$PATH" FAKE_SYSTEMCTL_REAL="$real_systemctl" FAKE_SYSTEMCTL_RESULT_RESET=1 RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 \
        bash "$target" 64M sleep 30 >/dev/null 2>"$scratch/settle-$settle_name.err"; status=$?
      if [[ $status -ne $TIME_LIMIT_HIT_EXIT ]] || ! grep -q '超过限时 1 秒' "$scratch/settle-reset-timeout.err" || ! grep -q 'journal 里它失败的原因是 timeout' "$scratch/settle-reset-timeout.err" \
          || grep -q '被外面停了' "$scratch/settle-reset-timeout.err"; then
        fail "限时被停的 scope 落到 failed 而 Result 读到 success（重载配置之后）时，包装应当到 journal 里找原因、照限时被停判（退 $TIME_LIMIT_HIT_EXIT 报「超过限时 1 秒」并另报「journal 里它失败的原因是 timeout」、不报「被外面停了」），实际退 $status：$(tr '\n' ' ' < "$scratch/settle-reset-timeout.err")"
      fi
    else
      in_context env PATH="$fake_directory:$PATH" FAKE_SYSTEMCTL_REAL="$real_systemctl" FAKE_SYSTEMCTL_RESULT_RESET=1 RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 \
        RUN_WITH_MEMORY_CAP_JOURNALCTL="$fake_directory/journalctl-broken" bash "$target" 64M sleep 30 >/dev/null 2>"$scratch/settle-$settle_name.err"; status=$?
      if [[ $status -eq $TIME_LIMIT_HIT_EXIT ]] || ! grep -q '原因取不到' "$scratch/settle-reset-unknown.err" || grep -q '超过限时' "$scratch/settle-reset-unknown.err"; then
        fail "限时被停的 scope 落到 failed 而 Result 读到 success、journal 又读不了时，包装应当写明「原因取不到」、不冒充限时被停（不退 $TIME_LIMIT_HIT_EXIT、不报「超过限时」），实际退 $status：$(tr '\n' ' ' < "$scratch/settle-reset-unknown.err")"
      fi
    fi
  done

  # 这一条命令自己的临时目录（文件头「这一条命令自己的临时目录」一段）：假命令把拿到的 TMPDIR 写进 scratch 里的一份文件、往 TMPDIR 里写一份文件，再照各自的结局退；
  # 退 0 的核那一份 TMPDIR 在包装退出之后不在了、RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 底下一个不剩；退非 0 的（退 7、kill -9、撞顶、超时）核那一份与命令写的文件都在、
  # 里面有标记、stderr 打出它的路径（「留着看现场」那一行）。各例的 root 在 scratch 里，留着的随 scratch 删。
  # 自证可能套在外层包装里跑（外层给了 TMPDIR），各例一律 env -u TMPDIR、-u SINGLEFS_KEEP_TEST_TEMPORARY_FILES 起，只在点名的那一例设
  local case_name case_status case_cell root recorded leftover caller code ending kept_age kept_problems now_seconds
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
      nonzero) ending='exit 7'; case_status=7; case_cell="命令退非 0 之后，包装给它的临时目录应当留着、stderr 打出路径" ;;
      killed) ending='kill -9 $$'; case_status=137; case_cell="命令被 kill -9 之后，包装给它的临时目录应当留着、stderr 打出路径" ;;
      cap-hit) ending='python3 -c "$2" 512'; case_status=$MEMORY_CAP_HIT_EXIT; case_cell="命令撞了 64M 上限被杀之后，包装给它的临时目录应当留着、stderr 打出路径" ;;
      time-limit) ending='sleep 30'; case_status=$TIME_LIMIT_HIT_EXIT; case_environment=(RUN_WITH_MEMORY_CAP_TIME_LIMIT=1)
        case_cell="命令超过限时被停之后，包装给它的临时目录应当留着、stderr 打出路径" ;;
    esac
    checked=$((checked + 1))
    if in_context env -u TMPDIR -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" ${case_environment[@]+"${case_environment[@]}"} \
        bash "$target" 64M bash -c "$code$ending" temporary-case "$recorded" "$hog_code" >/dev/null 2>"$scratch/temporary-$case_name.err"; then status=0; else status=$?; fi
    output="$(cat "$recorded" 2>/dev/null)"
    leftover="$(ls -A "$root" 2>/dev/null)"
    if [[ "$case_name" == normal ]]; then
      if [[ $status -ne $case_status || "$output" != "$root"/singlefs-run-with-memory-cap-* || -e "$output" || -n "$leftover" ]]; then
        fail "$case_cell（退 $case_status、命令拿到的 TMPDIR 在 $root 底下、跑完那一份不在了、$root 底下一个不剩），实际退 $status、命令拿到的 TMPDIR 是「$output」、\
那一份$([[ -n "$output" && -e "$output" ]] && echo "还在（里面：$(ls -A "$output" 2>/dev/null | tr '\n' ' ')）" || echo '不在')、$root 底下剩「$leftover」：$(head_characters 600 "$scratch/temporary-$case_name.err")"
      fi
    elif [[ $status -ne $case_status || "$output" != "$root"/singlefs-run-with-memory-cap-* || ! -e "$output/left-by-command" || ! -f "$output/$KEPT_TEMPORARY_MARKER" ]] \
        || ! grep -qF "这一条命令的临时目录留着看现场：$output" "$scratch/temporary-$case_name.err"; then
      fail "$case_cell（退 $case_status、命令拿到的 TMPDIR 在 $root 底下、跑完那一份与命令写的 left-by-command 都在、里面有标记、stderr 打出它的路径），实际退 $status、命令拿到的 TMPDIR 是「$output」、\
那一份$([[ -n "$output" && -e "$output" ]] && echo "在（里面：$(ls -A "$output" 2>/dev/null | tr '\n' ' ')）" || echo '不在了')：$(head_characters 600 "$scratch/temporary-$case_name.err")"
    fi
  done
  # 退非 0 而命令一个临时文件都没写：没有现场可看，照删、不打「留着看现场」
  fresh_context 256M
  root="$scratch/temporary-root-empty"
  recorded="$scratch/temporary-empty.path"
  mkdir -p "$root"
  checked=$((checked + 1))
  if in_context env -u TMPDIR -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" \
      bash "$target" 64M bash -c 'printf "%s\n" "${TMPDIR-}" > "$1"; exit 7' temporary-case "$recorded" >/dev/null 2>"$scratch/temporary-empty.err"; then status=0; else status=$?; fi
  output="$(cat "$recorded" 2>/dev/null)"
  leftover="$(ls -A "$root" 2>/dev/null)"
  if [[ $status -ne 7 || "$output" != "$root"/singlefs-run-with-memory-cap-* || -e "$output" || -n "$leftover" ]] || grep -qF '留着看现场' "$scratch/temporary-empty.err"; then
    fail "命令退非 0 而临时目录里什么都没写时，包装给它的临时目录应当照删（退 7、命令拿到的 TMPDIR 在 $root 底下、跑完那一份不在了、stderr 里没有「留着看现场」），实际退 $status、\
命令拿到的 TMPDIR 是「$output」、那一份$([[ -n "$output" && -e "$output" ]] && echo '还在' || echo '不在')、$root 底下剩「$leftover」：$(head_characters 600 "$scratch/temporary-empty.err")"
  fi
  # 判红留着的只留最近 KEPT_TEMPORARY_DIRECTORIES_LIMIT（3）个：root 下先放三份带标记的旧现场（标记的修改时刻依次是 300、200、100 秒以前）与一份没标记的（别的包装正在用的那种），
  # 一条退 7 的命令跑完：它那一份留着，旧的里最早的那一份删掉，另两份与没标记的那一份都在
  fresh_context 256M
  root="$scratch/temporary-root-limit"
  recorded="$scratch/temporary-limit.path"
  mkdir -p "$root/singlefs-run-with-memory-cap-1-in-use"
  : > "$root/singlefs-run-with-memory-cap-1-in-use/written-by-a-running-wrapper"
  printf -v now_seconds '%(%s)T' -1
  for kept_age in 300 200 100; do
    mkdir -p "$root/singlefs-run-with-memory-cap-1-kept-$kept_age"
    printf '包装退 1 留下的现场（自证造的）\n' > "$root/singlefs-run-with-memory-cap-1-kept-$kept_age/$KEPT_TEMPORARY_MARKER"
    touch -d "@$((now_seconds - kept_age))" "$root/singlefs-run-with-memory-cap-1-kept-$kept_age/$KEPT_TEMPORARY_MARKER"
  done
  checked=$((checked + 1))
  if in_context env -u TMPDIR -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" \
      bash "$target" 64M bash -c "${code}exit 7" temporary-case "$recorded" >/dev/null 2>"$scratch/temporary-limit.err"; then status=0; else status=$?; fi
  output="$(cat "$recorded" 2>/dev/null)"
  kept_problems=""
  [[ $status -eq 7 ]] || kept_problems+="退 $status；"
  [[ "$output" == "$root"/singlefs-run-with-memory-cap-* && -f "$output/$KEPT_TEMPORARY_MARKER" && -e "$output/left-by-command" ]] || kept_problems+="这一条那一份（$output）没留着；"
  [[ ! -e "$root/singlefs-run-with-memory-cap-1-kept-300" ]] || kept_problems+="最早那一份旧的（300 秒以前）还在；"
  [[ -e "$root/singlefs-run-with-memory-cap-1-kept-200" && -e "$root/singlefs-run-with-memory-cap-1-kept-100" ]] || kept_problems+="较新的两份旧的有一份不在了；"
  [[ -e "$root/singlefs-run-with-memory-cap-1-in-use/written-by-a-running-wrapper" ]] || kept_problems+="没标记的那一份（别的包装正在用的）被删了；"
  if [[ -n "$kept_problems" ]]; then
    fail "判红留着的只留最近 3 个（一条退 7 的命令跑完：它那一份留着、三份带标记的旧现场里最早那一份删掉、另两份与没标记的那一份都在），实际：${kept_problems}\
$root 底下剩「$(ls -A "$root" 2>/dev/null | tr '\n' ' ')」：$(head_characters 600 "$scratch/temporary-limit.err")"
  fi
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
  # 设了现场目录（文件头「task-server 给这一层的两样」）：要留的那一份整个搬成那个目录（不放标记、root 底下不剩、原路径不在了）、stderr 点名新位置；
  # 退 7 的与设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 退 0 的各一例。命令里看不到这两个变量
  local scene_case scene_directory scene_environment_seen
  for scene_case in failed kept; do
    fresh_context 256M
    root="$scratch/temporary-root-scene-$scene_case"
    recorded="$scratch/temporary-scene-$scene_case.path"
    scene_directory="$scratch/scene-of-attempt-$scene_case"
    mkdir -p "$root"
    if [[ "$scene_case" == failed ]]; then
      ending='exit 7'; case_status=7; case_environment=(-u SINGLEFS_KEEP_TEST_TEMPORARY_FILES)
      case_cell="设了现场目录、命令退 7 而临时目录里有东西时"
    else
      ending='exit 0'; case_status=0; case_environment=(SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1)
      case_cell="设了现场目录与 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时"
    fi
    checked=$((checked + 1))
    if in_context env -u TMPDIR "${case_environment[@]}" RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY="$scene_directory" \
        bash "$target" 64M bash -c "${code}printf '%s|%s\n' \"\${RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY-没有}\" \"\${RUN_WITH_MEMORY_CAP_PEAK_OUTPUT-没有}\" > \"\$1.environment\"; $ending" \
        temporary-case "$recorded" >/dev/null 2>"$scratch/temporary-scene-$scene_case.err"; then status=0; else status=$?; fi
    output="$(cat "$recorded" 2>/dev/null)"
    scene_environment_seen="$(cat "$recorded.environment" 2>/dev/null)"
    leftover="$(ls -A "$root" 2>/dev/null)"
    if [[ $status -ne $case_status || "$output" != "$root"/singlefs-run-with-memory-cap-* || -e "$output" || -n "$leftover" || ! -e "$scene_directory/left-by-command" \
          || -e "$scene_directory/$KEPT_TEMPORARY_MARKER" || "$scene_environment_seen" != "没有|没有" ]] \
        || ! grep -qF "搬进了这一次尝试的现场目录：$scene_directory" "$scratch/temporary-scene-$scene_case.err"; then
      fail "${case_cell}，包装给它的临时目录应当整个搬成现场目录（退 $case_status、$scene_directory 里有命令写的 left-by-command、没有标记、原来那一份不在了、$root 底下一个不剩、stderr 点名现场目录、命令里看不到两个变量），\
实际退 $status、原来那一份「$output」$([[ -n "$output" && -e "$output" ]] && echo '还在' || echo '不在')、现场目录里「$(ls -A "$scene_directory" 2>/dev/null | tr '\n' ' ')」、$root 底下剩「$leftover」、\
命令看到的「$scene_environment_seen」：$(head_characters 600 "$scratch/temporary-scene-$scene_case.err")"
    fi
  done
  # 退出时读到的峰值（文件头「task-server 给这一层的两样」）：命令跑 0.1 秒就退，比调用方一轮采样短；外壳退出时读的 scope 的 memory.peak 写进 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 那一份
  fresh_context 256M
  checked=$((checked + 1))
  rm -f -- "$scratch/exit-peak"
  if in_context env RUN_WITH_MEMORY_CAP_PEAK_OUTPUT="$scratch/exit-peak" RUN_WITH_MEMORY_CAP_KEY=selftest-exit-peak bash "$target" 64M sleep 0.1 2>"$scratch/exit-peak.err"; then status=0; else status=$?; fi
  output="$(cat "$scratch/exit-peak" 2>/dev/null)"
  if [[ $status -ne 0 || ! "$output" =~ ^[1-9][0-9]*$ || -e "$scratch/exit-peak.partial" ]]; then
    fail "命令跑 0.1 秒就退时，RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 那一份应当是外壳退出时读到的 scope 峰值（退 0、一个正整数、不剩 .partial），实际退 $status、读到「$output」：$(head_characters 600 "$scratch/exit-peak.err")"
  fi
  # scope 里的命令抬 oom_score_adj（TASK_OOM_SCORE_ADJUST）：整机吃紧时先挑任务进程，不挑常驻服务。
  # oom_score_adj 只抬得上去、降不回来，自证经服务起时已继承了抬过的值：目标取自己此刻的值 + 1（RUN_WITH_MEMORY_CAP_SELFTEST_OOM_SCORE_ADJUST），读到它才说明是被测包装设的
  fresh_context 256M
  checked=$((checked + 1))
  local own_oom_score="" wanted_oom_score=""
  IFS= read -r own_oom_score < /proc/self/oom_score_adj
  wanted_oom_score=$(( own_oom_score < 1000 ? own_oom_score + 1 : 1000 ))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-oom-score RUN_WITH_MEMORY_CAP_SELFTEST_OOM_SCORE_ADJUST="$wanted_oom_score" bash "$target" 64M bash -c 'cat /proc/self/oom_score_adj > "$1"' oom "$scratch/oom-score" 2>"$scratch/oom-score.err"; then status=0; else status=$?; fi
  output="$(cat "$scratch/oom-score" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != "$wanted_oom_score" || "$own_oom_score" -ge 1000 ]]; then
    fail "scope 里的命令 oom_score_adj 应当是被测包装设的 $wanted_oom_score（自证自己此刻是 $own_oom_score；到 1000 判不出），实际退 $status、读到「$output」：$(head_characters 600 "$scratch/oom-score.err")"
  fi

  # 峰值表的「线程数」一列：命令里经 capped.sh 3 跑的记 3；经 capped.sh default 跑的记登记值（登记文件第一条非注释行的数），SINGLEFS_THREAD_CAP 设了的记它；
  # 环境里线程变量都是 2 的记 2；不一样的逐个记「名=值」；都没设、命令也没经 capped.sh 的记「未设（整机 N 核）」；
  local -a thread_variable_unsets=()
  local thread_variable_name expectation expected_key expected_threads registered_threads
  while IFS='=' read -r thread_variable_name _; do thread_variable_unsets+=(-u "$thread_variable_name"); done < <(bash "$SCRIPT_DIRECTORY/capped.sh" --print 1)
  fresh_context 256M
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
    [[ "$output" == "7|$expected_threads" ]] \
      || fail "峰值表里 $expected_key 那一行应当是七列、线程数记「$expected_threads」，实际「$output」（列数|线程数）：$(cat "$scratch/threads.err")"
  done

  # 峰值表两台共用一份（文件头「峰值表」一段）：第七列记是哪一台（自证给的 RUN_WITH_MEMORY_CAP_SELFTEST_HOST），另一台同一个键的行原样留着
  fresh_context 256M
  local other_host_row
  other_host_row="$(printf '%s\t%s\t%s\t%s\t%s\t%s\t%s' $((5 * MEBIBYTE)) 2026-09-25T00:00:00Z 200M selftest-host-column 1 "$((5 * MEBIBYTE)):clean:unreclaimable" 192.0.2.99)"  # clock-times:allow 自检输入假时间戳
  printf '%s\n' "$other_host_row" >> "$context_directory/peaks.tsv"
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-host-column bash "$target" 64M true 2>"$scratch/host-column.err"
  output="$(awk -F '\t' '$4 == "selftest-host-column" {print NF "|" $7}' "$context_directory/peaks.tsv" 2>/dev/null | LC_ALL=C sort | tr '\n' ' ')"
  if [[ "$output" != "7|$selftest_peak_host 7|192.0.2.99 " ]] || ! grep -qxF -- "$other_host_row" "$context_directory/peaks.tsv"; then
    fail "峰值表两台共用一份：另一台同一个键的行原样留着、本台那一行第七列记 $selftest_peak_host，实际这个键的行「$output」（列数|哪一台）：$(cat "$scratch/host-column.err")"
  fi

  # 峰值表里有不是这一版的行（这里写一行四列的）：这一趟的峰值记不进，说一句（点名行号与要几列，不是回溯栈），命令的退出码照旧，表一个字节不动
  fresh_context 256M
  printf '%s\t%s\t%s\t%s\n' $((10 * MEBIBYTE)) 2026-09-25T00:00:00Z 200M selftest-four-columns >> "$context_directory/peaks.tsv"  # clock-times:allow 自检输入假时间戳
  checked=$((checked + 1))
  refused_before="$(cat "$context_directory/peaks.tsv")"
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-refused-row bash "$target" 64M bash -c 'exit 7' 2>"$scratch/refused-row.err"; then status=0; else status=$?; fi
  if [[ $status -ne 7 || "$(cat "$context_directory/peaks.tsv")" != "$refused_before" ]] || ! grep -q '峰值没记进表：.*第 1 行不是这一版的形态：列数不对，要 7 列，这一行 4 列' "$scratch/refused-row.err" \
      || grep -q 'Traceback' "$scratch/refused-row.err"; then
    fail "峰值表里有不是这一版的行（四列）：命令退 7 应当照旧退 7、峰值表不动、stderr 说「峰值没记进表」并点名第 1 行与要 7 列（不是回溯栈），实际退 $status：$(head_characters 600 "$scratch/refused-row.err")"
  fi

  # 只认服务直接起的那一层（文件头「只认服务直接起的那一层」一段）：没有准入标记 ⇒ 按用法错退 251、命令没跑、出路写「任务里面不再套包装」
  fresh_context 256M
  checked=$((checked + 1))
  marker="$scratch/unadmitted-ran"
  if in_context env -u RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/unadmitted.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q '任务里面不再套包装：内存、线程、机器、卡由 task-server 定；删掉这一层' "$scratch/unadmitted.err" 2>/dev/null; then
    fail "没有服务给的准入标记时应当按用法错退 $MEMORY_CAP_UNAVAILABLE_EXIT、命令不跑、出路写「任务里面不再套包装」，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unadmitted.err" 2>/dev/null)"
  fi
  # 被服务起的命令里面再调这一份：里层看不到准入标记 ⇒ 按用法错退 251、命令没跑；外层照跑完、退 0
  fresh_context 256M
  checked=$((checked + 1))
  marker="$scratch/nested-ran"
  rm -f -- "$scratch/nested.status"
  if in_context bash "$target" 100M bash -c 'bash "$1" 64M bash -c ": > \"\$1\"" touch-marker "$2" 2>"$3"; echo "$?" > "$4"' nested-outer "$target" "$marker" \
      "$scratch/nested.err" "$scratch/nested.status" 2>/dev/null; then status=0; else status=$?; fi
  output="$(cat "$scratch/nested.status" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != "$MEMORY_CAP_UNAVAILABLE_EXIT" || -e "$marker" ]]; then
    fail "被服务起的命令里面再调这一份应当按用法错退 $MEMORY_CAP_UNAVAILABLE_EXIT、命令不跑，\
实际外层退 $status、里层退 ${output:-（没记下）}、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/nested.err" 2>/dev/null)"
  fi

  # 已由 task-server 准入（文件头「只认服务直接起的那一层」一段）：设了 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER，命令仍在这一例 slice 底下自己的 scope 里，看不到这个变量
  fresh_context 256M
  checked=$((checked + 1))
  rm -f -- "$scratch/admitted-seen.txt" "$scratch/admitted-cgroup.txt"
  if in_context env RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER=selftest-task.1 RUN_WITH_MEMORY_CAP_KEY=selftest-admitted-by-task-server \
      bash "$target" 64M bash -c 'printf "%s" "${RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER-没有}" > "$1"; IFS= read -r line < /proc/self/cgroup; printf "%s\n" "${line#0::}" > "$2"' \
      admitted-probe "$scratch/admitted-seen.txt" "$scratch/admitted-cgroup.txt" 2>"$scratch/admitted.err"; then status=0; else status=$?; fi
  output="$(cat "$scratch/admitted-seen.txt" 2>/dev/null)"
  cgroup_path="$(cat "$scratch/admitted-cgroup.txt" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != 没有 || "$cgroup_path" != */"$slice"/singlefs-memory-cap-*.scope ]]; then
    fail "已由 task-server 准入的一条应当照常在这一例 slice 底下自己的 scope 里跑、退 0，命令里看不到 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER，\
实际退 $status、命令看到「${output:-（没记下）}」、命令的 cgroup「${cgroup_path:-（没记下）}」：$(cat "$scratch/admitted.err" 2>/dev/null)"
  fi

  # 收尾之后不留 scope：每个 slice 底下没有 scope
  checked=$((checked + 1))
  output=""
  for slice in "${selftest_slices[@]}"; do
    if compgen -G "/sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/$slice/*.scope" >/dev/null; then output+=" $slice 底下还有 scope"; fi
  done
  [[ -z "$output" ]] || fail "跑完应当不留 scope，实际：$output"

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
  local foreign_slice
  checked=$((checked + 1))
  foreign_slice="notsinglefs_selftest_$$.slice"
  selftest_slices+=("$foreign_slice")
  if env RUN_WITH_MEMORY_CAP_SLICE="$foreign_slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/foreign-state" \
      RUN_WITH_MEMORY_CAP_PEAKS="$scratch/foreign-peaks.tsv" bash "$target" --check 64M 2>"$scratch/foreign.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT ]] || ! grep -q 'singlefs 开头' "$scratch/foreign.err"; then
    fail "slice 名写成不是 singlefs 开头的 $foreign_slice 时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「singlefs 开头」，实际退 $status：$(cat "$scratch/foreign.err")"
  fi

  selftest_cleanup
  if ((failures)); then
    echo "  ✗ run-with-memory-cap.sh 自检 $failures 处不对（查了 $checked 项；测的是 $target）"
    echo "  → 怎么办：照上面逐条改 run_capped / ensure_slice / clear_leftover_memory_high / resolve_slice_total / configured_slice_cap / record_run / scope_result / check_size_syntax / reject_usage / command_threads / show_status / gpu_memory_verdict / 临时目录那几个函数（取键与峰值口径在 memory_peaks_lib.py）；RUN_WITH_MEMORY_CAP_BREAK 设着、或 RUN_WITH_MEMORY_CAP_SELFTEST_TARGET 指着改前那一份的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ run-with-memory-cap.sh 自检通过（查了 $checked 项：平常跑设了弄坏开关退 2 并点名只在自证里认而带着自证的标记照常判、--check 起得来、64M 上限里分配 512 MiB 的探针与它当孙进程时都判撞顶退 $MEMORY_CAP_HIT_EXIT \
且峰值记成撞顶、上限之内 kill -9 原样退 137 且 stderr 不多一行、被外面的 TERM 与 KILL 停原样退且峰值不记、退出码 0 / 7 / 101 原样传回、\
命令退非 0 而起止之间内核报了显存不够退 $GPU_MEMORY_HIT_EXIT 且按起止给日志窗口、读不到内核日志报没查成、别的 NVRM 行不算、命令退 0 不读日志、撞了上限又报了显存不够照旧退 $MEMORY_CAP_HIT_EXIT 且另报一行、\
scope 里 memory.max 与 memory.swap.max 设上了、scope 挂在 slice 底下且 slice 的总上限设上了、slice 设不上时退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、\
总上限照多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（头一项是另一台也照这一台取）且 --status 报它从哪来、配置里没有这个键时照 slice 已有的走、\
这一台那一项写法不对、没有这一台那一项与三样都没有时退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、\
slice 上留下的 memory.high 清回 max（留着 47.5G 与 slice 起来之后才带上的清一次并打一行、已是 max 与 slice 原先不在的不清、清了读回不是 max 退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑）、\
D-Bus 连不上时跑命令与 --check 都退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、systemd-run 不认上限时 --check 退 $MEMORY_CAP_UNAVAILABLE_EXIT、被总上限挤掉退 $SLICE_TOTAL_HIT_EXIT 且那一次峰值不记、\
scope 里一直收 TERM 时峰值照记、起了 scope 之后给包装发 TERM / INT 时先停自己开的 scope 且标记与 scope 都不留、\
自证开出 slice 之后被 TERM / INT 停时退 143 / 130 且它开的 slice 都不再 active、临时目录删掉、超过限时退 $TIME_LIMIT_HIT_EXIT、\
峰值表的口径（页缓存涨满上限记回收不掉的那一截、oom_kill 大于 0 而 Result 没报的记撞顶）、采样子进程不拖延收尾（四种收尾）、撞顶的 scope 结局读早了与一直读不到都报撞了上限、\
failed 而 Result 读到 success 时到 journal 找原因、这一条命令自己的临时目录（五种结局、空的照删、只留最近 3 个、SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 留着、调用方给的 TMPDIR 照它的且不删）、\
峰值表的线程数一列六种各记对、没有服务的准入标记退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、被服务起的命令里面再调这一份退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、\
已由 task-server 准入的一条命令看不到那个变量、跑完不留 scope、--status 私有配置三种、用法错四种都退 $MEMORY_CAP_UNAVAILABLE_EXIT）"
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
