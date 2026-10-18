#!/usr/bin/env bash
# detached-run-script-signature: singlefs 脱离会话长跑入口（.claude/hooks/ 的子 agent 闸与重型测试闸按这一行认改名、拷走的一份，别删、别挪出文件开头 4 KiB）
# 长跑脚本脱离会话跑：挂到用户级 systemd 的一个临时服务单元下，不在调用方的进程树里，调用方被停、会话的后台时限到点、会话关掉都不影响它；
# 会话里只上报（list / watch），不停它（records/2026-09-16-subagent拆分提案.md 第四十二节，用户 2026-10-01 定）。
#
#   detached-run.sh start --wall-clock <within-2h|over-2h> --label <名> [--log <文件>] [--stop-timeout <秒>] -- <命令与参数…>
#   detached-run.sh long-run-check -- <命令与参数…>
#   detached-run.sh list
#   detached-run.sh watch [--every <秒>]
#   detached-run.sh stop <名>
#   detached-run.sh --selftest
#
# start：单元名是 singlefs-detached-<名>.service，挂在 singlefs-detached.slice 下（systemd-run --user，--service-type=exec，--collect，
#   --expand-environment=no：命令里的 $ 与 % 原样交给命令）。名只用字母、数字、点、减号与下划线，字母或数字起头，至多 64 个字。
#   同名的单元还在跑（ActiveState 是 active / activating / deactivating / reloading / refreshing）就拒、退 3，什么都不起；
#   systemctl --user show 查不到同名单元的状态（失败、或什么都没打：用户级 systemd 连不上）就拒、退 1，什么都不起、旧登记不动；
#   同名的登记已经结束（或单元已不在、没留退出码），先把那个登记目录改名成 <名>@<起跑的秒数> 留着，再起新的。判与起在同一把锁里（登记目录的 .lock，flock）。
#   起跑秒数是旧登记挪去的名字、也是 watch 记观察的键：同名旧登记的起跑秒数不早于此刻（同一秒里同名重起）的，先等到它之后那一秒再起，
#   至多等 DETACHED_RUN_SAME_LABEL_WAIT_SECONDS（5）秒，还不行（时钟往回拨过）就拒、退 1，什么都没起、旧登记不动。
#   带进单元的环境：调用方的 SINGLEFS_HEAVY_TESTS、PATH、HOME、CARGO_HOME、RUSTUP_HOME（设了的才带，逐个 --setenv=名=值），工作目录是调用方的当前目录。
#   别的变量不带：单元里是用户级 systemd 给的那一份环境（systemctl --user show-environment）加上这几样；要别的，写成 -- env 名=值 <命令>。
#   TMPDIR 不带：会话的临时目录随会话清掉，单元里落回 /tmp。
#   命令经 env 起：-- 之后开头的 名=值 是给命令设的变量（与在 shell 里写在命令前一样）。名字要合 shell 变量名的写法（字母、数字、下划线，不以数字起头）：
#   开头那一串带 = 的词里有一个名字不合写法（a-b=1、--force=1、1=1、a.b=1），start 拒、退 2、什么都不起——env 照样把它当赋值、跑后面那条，
#   两道闸切词时却把它当命令词、看不到后面那条。判的是 preflight 摘掉整词 --force 之后、start 真收到的参数。
#   preflight 摘掉参数里每一个 --force（连 -- 之后的、连写在 --label 这类选项值位置上的），单元里跑的是摘掉之后的命令；两道闸切 start 的参数时
#   先照同一个摘法摘（.claude/hooks/lib_shell_words.py 的 detached_run_arguments_after_preflight），闸看的与真跑的逐字相同。要把 --force 交给命令，
#   写成 -- bash -c '<命令> --force'。
#   单元的主进程（MainPID）是一个 bash：它把自己与命令的进程号写进登记，命令的标准输出与标准错误追加进日志，命令退出后把退出码写进登记再退出；
#   单元被停（stop、systemctl --user stop）时它等命令退出，照样写退出码（被 TERM 停的是 143）。登记在单元结束之后还在，退出码随时读得到。
#   命令退出之后（自己退出、或单元被停）它找命令留在别的 scope 里的进程：内存包装（run-with-memory-cap.sh）经 systemd-run --user --scope 起的那一层
#   不在这个单元的 cgroup 里，systemd 停单元时不停它。认法是环境里带着这一次的 DETACHED_RUN_INSTANCE（单元里那个 bash 设给命令的，子孙都继承）、
#   cgroup 是 .scope 的进程；逐个 systemctl --user stop --no-block 它们所在的 scope（不等，scope 自己的停机时限到点 systemd 发 KILL），往日志里各记一行。
#   命令自己整个挪进 scope（单元的 cgroup 里一个命令的进程都不剩）的，单元被停时那个 bash 等不到命令退出，到停机时限被 KILL，这一步来不及做。
#   命令退出之后单元里写登记失败（登记目录写不进、盘满）的：它把「哪几项写不进、命令退出码」写进旁路文件（${DETACHED_RUN_SIDE_DIRECTORY:-$XDG_RUNTIME_DIR/
#   singlefs-detached-runs}/<名>@<起跑的秒数>.record-write-failed，路径记在登记的 write-failure-file；XDG_RUNTIME_DIR 没设时用 /tmp），再往日志与单元的
#   标准错误各打一行；list 与 watch 读旁路文件，单列成「登记写不进」，不报成被 KILL。旁路也写不进的，照旧列成「没留退出码」。
#   日志默认是登记目录里的 log；--log 给的相对路径按调用方的当前目录算，所在目录要已经在。
#   起单元之前先核写得进：--log 所在的目录写得进、日志已经在时它写得进；登记目录建得了、新登记的各项与日志开头那几行写得进。
#   有一样不成就退 1、不起单元，这一次建的登记删掉、挪走的同名旧登记挪回原处；建不了这一次的登记目录时同样退 1、挪回旧登记。
#   挪不回的（原处已有东西、mv 失败）在 stderr 打一行：旧登记还在 <名>@<起跑的秒数>，怎么挪回。
#   单元的停机时限（TimeoutStopSec）由 --stop-timeout <秒> 给，默认 DETACHED_RUN_DEFAULT_STOP_TIMEOUT_SECONDS（180 秒），写没写都设进单元：
#   被停时命令有这么久收尾，到点还没退出 systemd 发 KILL（连主进程那个 bash 一起，退出码写不出来）。默认取 180 秒，是因为要给收到 TERM 之后收尾的脚本留够时间
#   （broad-mutation.py 先停在途那一步的子进程再退；multi-host-run.sh 经 SSH 停第二台那条命令、自己至多等 30 秒再 KILL，再删第二台这一趟的树；
#   这几段收尾要多久是推的，没量过）；要更长的写 --stop-timeout。时限写进登记的 stop-timeout。stop 怎么等、前台调用给多长的超时，见 stop 那一段。
#   systemd-run 退非 0 时回读单元的 ActiveState：单元在跑（systemd-run 报错而单元照起，例如 D-Bus 调用超时），登记里写 systemd-run-failed
#   （日期与它的退出码），stderr 打一行警告，照在跑往下回读、退 0；单元不在跑，登记里写 start-failed（日期与 systemd-run 的退出码）再退 1，
#   list 把它列成「没起成」，不当成被 KILL 的；回读也查不到单元状态的，同样写 start-failed、退 1，list 之后照单元状态改判。
#   起之前读 loginctl show-user <用户> -p Linger：为 no、或读不到时，stderr 打一行警告，照起（不拒）。
#   起完回读：单元的 ActiveState 与 MainPID、登记里命令的进程号（至多等 10 秒），两样都读不到就判没起成、退 1；登记里有退出码而没有命令的进程号，
#   是命令没跑起来（单元里写不进登记或日志，退出码是单元里那个 bash 的），判没起成、退 1。
# start 的 --wall-clock（必填，也可写成 --wall-clock=<值>）：这一次的预计挂钟在 2 小时内写 within-2h，超过写 over-2h。没写、写了别的值，用法错退 2、什么都不起。
#   over-2h 的：从 -- 之后的词里认出被起的那份脚本，读它文件头注释块里的长活声明——# long-run-shards:（分片单元）、# long-run-resume:（续跑入口：
#   同一条命令重起只补没做完的）、# long-run-hosts:（这一行与派发提示里的「多机：」一行同一种读法，按登记值读：只认 .claude/hooks/lib_long_run.py 的 CANONICAL_HOSTS_SENTENCE 那一句规范写法，
#   逐字（可带句末句号），后面不许接任何说明，说明另起一行写普通注释，别的写法一律拒，不再用现算词表；规范句只写在那一处，这里不抄。文件头写的读法与脚本真做的事是否一致，钩子看不到，由写文件头的人保证。
#   只有 .claude/long-run-live-hosts-exempt 登记、登记的 sha256 又对得上这份脚本现在文件的脚本（真按此刻空闲现算的 E171 六份入口）这一行按现算读：点名 --emit-hosts 与 host-resources.py，不写死；
#   登记的 sha256 对不上（脚本改过）判红；那张表只缩不涨，读法的细节见 .claude/hooks/lib_long_run.py 文件头），
#   分不了的写 # long-run-unshardable: 点名缺的那几段与为什么（至少 8 个字）。判据 .claude/rules/test-script-discipline.md
#   「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」；认被起的脚本、读声明与判法都在 .claude/hooks/lib_long_run.py（写法以它的文件头为准）。
#   声明不齐、多机一行写死额度、判不出被起的是哪份脚本（bash -c '…' 一长串、直接起 cargo 或编出来的二进制）、判法起不来，都拒、退 5，
#   什么都不起、旧登记不动（判在拿锁之前）；出路：把命令写进一份脚本、文件头补声明、改成 -- bash <那份脚本> 起。齐了在 stderr 打一行「长活声明齐了」。
#   within-2h 的不读声明。挂钟估得对不对这里判不了。
# long-run-check：照 over-2h 判 -- 之后那条命令的长活声明，不起单元、不碰登记；齐了退 0（stdout 一行），不齐退 5，用法错退 2。
#   谁都能跑（子 agent 交回长跑法之前自查；.claude/hooks/bash-command-detector.sh 只拒子 agent 的 start 与 stop）。
# list：逐条列登记——还在跑的：名、单元名、主进程号、命令的进程号、起跑日期、已跑时长、日志大小；已结束的：退出码（stop 停的另注一句；没有命令进程号的
#   注「命令没跑起来」）、结束日期、用时；单元已不在又没留退出码的单列（stop 停时到点被 KILL 的另注一句），其中旁路文件在的单列成「登记写不进」；
#   systemctl --user show 查不到状态的单列成「查不到」（不知道还在不在跑，不算结束）；记了 start-failed 的先按单元名查状态，单元在跑的列成还在跑，
#   查不到的列成查不到，单元不在的才列成「没起成」（挪走的旧登记 <名>@<秒数> 不查：单元名已归新一次）。每条带命令与日志的最后一行。
#   另读 systemctl --user list-units 'singlefs-detached-*'，单元在、这个登记目录里却没有它的（登记被删了、或在别的登记目录里）逐个列成「登记里没有」，
#   读不到单元清单的打一句「查不到单元清单」。成功句报「还在跑、没结束的 N 条，已结束 M 条，没留退出码 K 条，登记写不进 W 条，查不到 Q 条，没起成 F 条；
#   登记里没有的单元 U 个」。
# watch：只上报的观察者，给会话里 run_in_background 用。每 --every 秒（默认 300）打一次 list；还在跑的与查不到的一条都不剩时打观察到的各条退出码、退 0；
#   查不到的不当结束，接着观察（清单里写明是查不到）。观察到的每一条按「登记目录 + 起跑秒数」记，连同它的单元名：观察之后同名又起过一次的，
#   旧一次的退出码到挪走的 <名>@<起跑的秒数> 里读，照样报；观察到的登记不见了（被删、被挪走）而同名没有新登记的，按记下的单元名查状态：
#   单元还在跑的列成「还在跑（登记里没有）」、查不到的列成「查不到（登记里没有）」，都不算结束，接着等；单元不在了才算结束（退出码读不到）。
#   watch 开跑之后起跑的登记（起跑秒数不早于 watch 开跑那一秒，连挪走的 <名>@<秒数>）每一轮都记成观察到的，一轮清单都没见它在跑的也算：
#   两次清单之间起了又跑完的短命令照报退出码。
#   同一条连续 DETACHED_RUN_UNKNOWN_ALARM_ROUNDS（3）次查不到，每一次清单之后单列一行「报警」，仍不退出、不停单元：总线一两秒的抖动连 3 次碰上的少，
#   默认每 300 秒一次时 3 次是 10 分钟以上连不上，该让人看一眼；报警只是一行字，报早了代价小（与 scripts/hosts/peer-host-lib.sh 判 SSH 连不上
#   的「连续 60 次」同一种写法，那边每几秒查一次，这里每次间隔长，次数取小）。登记里没有、也没观察到的单元只在清单里列，不等它们。
#   收到 TERM / INT / HUP（会话的后台时限到点、任务被停）先打最后一次清单（「还在跑」那几条就是还在跑、没结束的脚本），再退 143 / 130 / 129。
#   一轮轮列清单的是 watch 起的子进程，信号由 watch 进程在 wait 里接：记下信号、停子进程、等它退，再打最后一次清单（detached_run_watch 里的说明）。
#   它只读 systemctl --user show / list-units 与登记，不停、不给任何单元或命令发信号（只停它自己起的那一个 sleep）；被 KILL 时最后那一次清单打不出来，定时那几次照打。
# stop：只按登记里的单元名停那一个（systemctl --user stop <单元名>），不按模式找进程。systemctl --user stop 等单元停下（命令在停机时限里退出，或到点被 KILL），
#   命令收尾要多久就等多久，至多停机时限；停完回读 ActiveState，再至多等 5 秒读登记里的退出码，读不到就是到点被 KILL 了，登记里写 stop-killed（日期与停机时限）。
#   所以一次 stop 至多要停机时限加 5 秒多一点（默认 180 秒的单元是 185 秒）。前台调用 stop 时，Bash 工具的 timeout 至少给到（停机时限 + 30）× 1000 毫秒
#   （默认时限给 210000）：工具不写 timeout 时默认 120 秒，比默认时限短，到点 stop 被截断，单元照停（停机任务在 systemd 里），但退出码与 stop-killed
#   这一次报不出来，之后用 list 看。停机时限超过 210 秒的（（时限 + 30）× 1000 超过前台命令 timeout 的上限 240000 毫秒），stop 用 run_in_background 起、等完成通知。
#   240000 毫秒出自 .claude/agent-common.md「长活交回主 agent」那一条：前台命令的 timeout 不超过 240000 毫秒（是规则定的数，不是 Bash 工具自己的上限）。
#   不改成只发停机、不在前台等：同名重起要旧一次的单元已经停下（start 判同名在跑就拒），stop 回来时单元已停下、结局已报，重起才接得上。
#   查不到单元状态（用户级 systemd 连不上）就拒、退 1，什么都没停。
#
# 谁能用：start 与 stop 只给主 agent（.claude/hooks/bash-command-detector.sh 拒子 agent 的；list、watch 只读，谁都放）。
#   .claude/hooks/heavy-test-guard.sh 把 start 当包装看穿，-- 之后那条命令照重型测试那张表判，前缀照认（.claude/hooks/lib_heavy_tests.py 的 command_under_detached_run）。
#   两道闸按内容认这个脚本（.claude/hooks/lib_shell_words.py 的 detached_run_script）：名字叫 detached-run.sh、指到它的符号链接、文件开头带签名行的改名拷贝，
#   bash / sh 起的、source / . 读进来的、直接执行的都认；同一条命令里先 cp（子 agent 闸另认 install、ln、mv）再起的按目标认。
#   罩不住的写法：删掉签名行的拷贝；eval、变量里拼出来的路径（S=…; bash "$S"）、python 的 subprocess、xargs 与 find -exec 起的；
#   命令词不带斜杠、从 PATH 里找到的拷贝；/usr/bin/time、flock、systemd-run --wait 这类包装后面的（子 agent 闸不剥它们）；
#   重型闸里同一条命令里先 ln -s / install / mv 再起的；主 agent 经改名拷贝起时 -- 之后那条命令（③ 的放行口只按名字剥，里面的放出追踪认不出）。
# 用户级 systemd 开着 linger（loginctl show-user <用户> 的 Linger=yes）时，单元跨登录会话活着；Linger=no 时这个用户的最后一个登录会话结束，
#   用户级 systemd 连同这里起的单元一起停，list 里记成「没留退出码」。start 每次现读 Linger，为 no 时警告（见 start 那一段）。
#   会话的后台时限到点、Claude Code 会话关掉不在此列。
#
# 登记目录：${DETACHED_RUN_STATE_DIRECTORY:-${XDG_STATE_HOME:-~/.local/state}/singlefs/detached-runs}/<名>/（DETACHED_RUN_STATE_DIRECTORY 给自证与临时换目录用）。
#   每条一个目录：unit、command、cwd、environment（带进去的变量名）、started（秒数）、started-date、log（日志路径）、runner-pid（主进程号）、
#   pid（命令的进程号）、ended、ended-date、exit（退出码）、stop-timeout（停机时限的秒数）、stop-requested（stop 停的）、
#   stop-killed（stop 停时到点被 KILL）、start-failed（systemd-run 没起成）、systemd-run-failed（systemd-run 退非 0 而单元照起）、
#   write-failure-file（单元里写登记失败时的旁路文件路径；DETACHED_RUN_SIDE_DIRECTORY 给自证换目录用）。
#
# 退出码：0 成功（watch：观察的全部结束）；1 systemd 起不来、起了回读不到、登记目录建不了、查不到单元状态；2 用法错（什么都没起）；
#   3 拒绝（同名还在跑、stop 的名没有登记）；4 弄坏开关 DETACHED_RUN_BREAK 设在 --selftest 之外（什么都没做；出路：去掉它再跑）；
#   5 长活声明不齐（start --wall-clock over-2h 与 long-run-check；什么都没起）；
#   78 准入或运行条件不满足（preflight）；watch 被停退 143 / 130 / 129。
#
# 自证（--selftest）：在临时登记目录里起名为 selftest-<进程号>-<格> 的单元，只跑 sleep、true 与几行 bash；收尾逐个按单元名停、
#   再用 systemctl --user list-units 核这个前缀一条不剩（不剩也是一格）。替用户级 systemd 失灵、systemd-run 失败、Linger 的几格
#   在 PATH 最前面放替身的 systemctl、systemd-run、loginctl（只在那一格的命令上），不动真的用户级 systemd。
# 弄坏开关 DETACHED_RUN_BREAK=<项>（只给证红用，逗号分隔可给几个；打开时 --selftest 退 1）。只在 --selftest 起的进程里生效：
#   自证在它的临时登记目录里放 .selftest-token、把同一个值放进 DETACHED_RUN_SELFTEST_TOKEN 带给它起的命令，两样对得上才认开关；
#   别处设了它（两样对不上）就拒绝执行、退 4，什么都不做。
# 逐项写开关名、弄坏哪一处、打开后自证哪几格红：项名是自证 detached_run_expect 的格名，一行一项；失败行就是「✗ 自证格不对：<格名>」，
#   句首即格名，所以格名后不再加括号。格名里随跑变的写成 <进程号>（单元名前缀 selftest-<进程号>-）。
#   DETACHED_RUN_BREAK=in-caller-tree  start 不经 systemd，把命令起在调用方的进程树里
#       Ⓕ watch 在观察的全部结束时自己退 0、报 selftest-<进程号>-f 退出码 5
#       Ⓒ 同名还在跑就拒、退 3，原登记的命令进程号不变、还活着，list 报还在跑 1 条
#       Ⓒ 没写 --stop-timeout：单元的 TimeoutStopUSec 是默认的 180 秒，登记里同一个数
#       Ⓖ stop 停下登记的那一个单元：退 0、单元不在跑、退出码 143、记了 stop-requested、命令进程没了
#       Ⓝ stop 等命令收尾完（TERM 之后 7 秒退 9）：退 0、报退出码 9、回来时单元已停下、没记 stop-killed
#       Ⓐ 调用方的单元被停之后，命令的单元还在跑、sleep 还活着
#       Ⓔ watch 被 TERM 停：退 143、最后一次清单里有还在跑的 selftest-<进程号>-e，e 的单元与 sleep 都还在
#       Ⓔ watch 被 INT 停：退 130、最后一次清单里有还在跑的 selftest-<进程号>-i，i 的单元与 sleep 都还在
#       Ⓔ watch 被 HUP 停：退 129、最后一次清单里有还在跑的 selftest-<进程号>-u，u 的单元与 sleep 都还在
#       Ⓔ watch 在一轮清单列到一半时收到 TERM：一轮清单在 watch 起的一个子进程里列、不在接信号的 watch 进程里，退 143、最后一次清单里有还在跑的 selftest-<进程号>-e2，e2 的单元与 sleep 都还在
#       Ⓢ systemd-run 没起成：start 退 1、登记里有 start-failed、list 列成「没起成」
#       Ⓡ 同名重起：watch 退 0，照报观察到的那一次的退出码 143（stop 停的，登记已挪走）
#       Ⓧ 登记被挪走而单元还在跑：list 列成「登记里没有  单元 singlefs-detached-selftest-<进程号>-x.service」
#       Ⓦ 观察到的登记被挪走而单元还在跑：watch 列成「还在跑（登记里没有）  selftest-<进程号>-w」，不报都结束了（退 124）
#       Ⓦ systemd-run 退 1 而单元照起：start 退 0、登记里有 systemd-run-failed 没有 start-failed、list 列成还在跑
#       Ⓦ 登记里记了 start-failed 而单元在跑：list 按单元名查，列成还在跑，不列「没起成」
#       Ⓣ --stop-timeout 2 设进单元、命令不理 TERM：stop 到点 KILL、退 0、没留退出码、登记里有 stop-killed、list 注明被 KILL
#   DETACHED_RUN_BREAK=exit-dropped  单元里不写退出码
#       Ⓑ 退出码在单元结束之后读得到（start 退 0、登记里 exit 是 7、单元已收掉、list 报退出码 7 与最后一行）
#       Ⓕ watch 在观察的全部结束时自己退 0、报 selftest-<进程号>-f 退出码 5
#       Ⓖ stop 停下登记的那一个单元：退 0、单元不在跑、退出码 143、记了 stop-requested、命令进程没了
#       Ⓝ stop 等命令收尾完（TERM 之后 7 秒退 9）：退 0、报退出码 9、回来时单元已停下、没记 stop-killed
#       Ⓡ 同名重起：watch 退 0，照报观察到的那一次的退出码 143（stop 停的，登记已挪走）
#       Ⓥ 起单元之前没核日志时：起完回读认得出命令没跑起来（有退出码没有命令进程号）、退 1，list 注明没跑起来
#   DETACHED_RUN_BREAK=same-label-allowed  同名还在跑也照起
#       Ⓒ 同名还在跑就拒、退 3，原登记的命令进程号不变、还活着，list 报还在跑 1 条
#   DETACHED_RUN_BREAK=environment-dropped  不带调用方的环境变量
#       Ⓓ SINGLEFS_HEAVY_TESTS、CARGO_HOME、RUSTUP_HOME、PATH、HOME 与工作目录带进单元，$ 与 % 原样
#   DETACHED_RUN_BREAK=path-home-dropped  不带 PATH 与 HOME
#       Ⓓ SINGLEFS_HEAVY_TESTS、CARGO_HOME、RUSTUP_HOME、PATH、HOME 与工作目录带进单元，$ 与 % 原样
#   DETACHED_RUN_BREAK=watch-stops-units  watch 收到 TERM / INT / HUP 时停掉还在跑的单元
#       Ⓔ watch 被 TERM 停：退 143、最后一次清单里有还在跑的 selftest-<进程号>-e，e 的单元与 sleep 都还在
#       Ⓔ watch 被 INT 停：退 130、最后一次清单里有还在跑的 selftest-<进程号>-i，i 的单元与 sleep 都还在
#       Ⓔ watch 被 HUP 停：退 129、最后一次清单里有还在跑的 selftest-<进程号>-u，u 的单元与 sleep 都还在
#       Ⓔ watch 在一轮清单列到一半时收到 TERM：一轮清单在 watch 起的一个子进程里列、不在接信号的 watch 进程里，退 143、最后一次清单里有还在跑的 selftest-<进程号>-e2，e2 的单元与 sleep 都还在
#   DETACHED_RUN_BREAK=watch-stops-in-trap  watch 在自己的进程里一轮轮列清单，trap 里当场列最后一次清单、退出（改前的写法）
#       Ⓔ watch 在一轮清单列到一半时收到 TERM：一轮清单在 watch 起的一个子进程里列、不在接信号的 watch 进程里，退 143、最后一次清单里有还在跑的 selftest-<进程号>-e2，e2 的单元与 sleep 都还在
#   DETACHED_RUN_BREAK=watch-stops-units-on-hup  watch 只在收到 HUP 时停掉还在跑的单元
#       Ⓔ watch 被 HUP 停：退 129、最后一次清单里有还在跑的 selftest-<进程号>-u，u 的单元与 sleep 都还在
#   DETACHED_RUN_BREAK=watch-stops-units-on-int  watch 只在收到 INT 时停掉还在跑的单元
#       Ⓔ watch 被 INT 停：退 130、最后一次清单里有还在跑的 selftest-<进程号>-i，i 的单元与 sleep 都还在
#   DETACHED_RUN_BREAK=query-failure-as-gone  systemctl --user show 失败当成单元不在
#       Ⓠ 查不到单元状态：list 单列「查不到」，watch 不当结束（到 6 秒被 timeout 停、退 124，没报「都结束了」）
#       Ⓠ 连续 3 次查不到：watch 报警一行「报警  selftest-<进程号>-q」、带出路，仍不退出（退 124）
#       Ⓠ 查不到单元状态：start 拒、退 1、没建登记；stop 拒、退 1、没记 stop-requested
#       Ⓠ 查不到单元状态时 start 同名：拒、退 1，旧登记留在原处（没挪走）
#       Ⓙ 查不到之后又查得到、那条已结束：watch 退 0、照报 selftest-<进程号>-j 退出码 0
#   DETACHED_RUN_BREAK=start-failure-unmarked  systemd-run 没起成时不写 start-failed
#       Ⓢ systemd-run 没起成：start 退 1、登记里有 start-failed、list 列成「没起成」
#   DETACHED_RUN_BREAK=watch-keys-by-path  watch 按登记目录记观察到的，不带起跑秒数
#       Ⓡ 同名重起：watch 退 0，照报观察到的那一次的退出码 143（stop 停的，登记已挪走）
#   DETACHED_RUN_BREAK=unregistered-units-hidden  list 不读单元清单
#       Ⓧ 登记被挪走而单元还在跑：list 列成「登记里没有  单元 singlefs-detached-selftest-<进程号>-x.service」
#   DETACHED_RUN_BREAK=stop-timeout-ignored  start 不给单元设 TimeoutStopSec
#       Ⓒ 没写 --stop-timeout：单元的 TimeoutStopUSec 是默认的 180 秒，登记里同一个数
#       Ⓣ --stop-timeout 2 设进单元、命令不理 TERM：stop 到点 KILL、退 0、没留退出码、登记里有 stop-killed、list 注明被 KILL
#   DETACHED_RUN_BREAK=stop-kill-unrecorded  stop 读不到退出码时不写 stop-killed
#       Ⓣ --stop-timeout 2 设进单元、命令不理 TERM：stop 到点 KILL、退 0、没留退出码、登记里有 stop-killed、list 注明被 KILL
#   DETACHED_RUN_BREAK=linger-unchecked  start 不读 Linger
#       Ⓛ Linger=no 时 start 警告一行、照起（退 0）；Linger=yes 时不警告
#   DETACHED_RUN_BREAK=break-honored-anywhere  弄坏开关在 --selftest 之外也认
#       Ⓚ DETACHED_RUN_BREAK 设在 --selftest 之外：start 与 list 都拒、退 4，没建登记
#   DETACHED_RUN_BREAK=default-stop-timeout-unset  只有写了 --stop-timeout 才设进单元，默认 180 只写进登记
#       Ⓒ 没写 --stop-timeout：单元的 TimeoutStopUSec 是默认的 180 秒，登记里同一个数
#   DETACHED_RUN_BREAK=stop-nonblocking  stop 不等停机任务（--no-block），至多等 5 秒退出码与单元停下，等不到就记 stop-killed
#       Ⓝ stop 等命令收尾完（TERM 之后 7 秒退 9）：退 0、报退出码 9、回来时单元已停下、没记 stop-killed
#   DETACHED_RUN_BREAK=query-check-after-archive  查不到单元状态的判挪到「挪走同名旧登记」之后
#       Ⓠ 查不到单元状态时 start 同名：拒、退 1，旧登记留在原处（没挪走）
#   DETACHED_RUN_BREAK=watch-skips-unknown  watch 只记还在跑的、不记查不到的
#       Ⓠ 连续 3 次查不到：watch 报警一行「报警  selftest-<进程号>-q」、带出路，仍不退出（退 124）
#       Ⓙ 查不到之后又查得到、那条已结束：watch 退 0、照报 selftest-<进程号>-j 退出码 0
#   DETACHED_RUN_BREAK=watch-trusts-registry  watch 不按记下的单元名查登记不见了的那几条
#       Ⓦ 观察到的登记被挪走而单元还在跑：watch 列成「还在跑（登记里没有）  selftest-<进程号>-w」，不报都结束了（退 124）
#   DETACHED_RUN_BREAK=observe-overwrites-unit  watch 记观察时读到什么记什么：登记刚好被挪走、单元名读不到，就拿空的盖掉上一次记下的（改前的写法）
#       Ⓦ 观察到的登记被挪走而单元还在跑：watch 列成「还在跑（登记里没有）  selftest-<进程号>-w」，不报都结束了（退 124）
#   DETACHED_RUN_BREAK=unknown-alarm-missing  连续查不到不报警
#       Ⓠ 连续 3 次查不到：watch 报警一行「报警  selftest-<进程号>-q」、带出路，仍不退出（退 124）
#   DETACHED_RUN_BREAK=start-failure-not-rechecked  systemd-run 退非 0 时不回读单元状态，一律记 start-failed、退 1
#       Ⓦ systemd-run 退 1 而单元照起：start 退 0、登记里有 systemd-run-failed 没有 start-failed、list 列成还在跑
#   DETACHED_RUN_BREAK=start-failed-trusted  list 见到 start-failed 就列「没起成」，不按单元名查状态
#       Ⓦ 登记里记了 start-failed 而单元在跑：list 按单元名查，列成还在跑，不列「没起成」
#   DETACHED_RUN_BREAK=log-unchecked  起单元之前不核 --log 写不写得进
#       Ⓥ --log 写不进：start 退 1、不留登记、命令没跑、没打「✓ 起了」
#   DETACHED_RUN_BREAK=ran-without-pid-trusted  起完回读见到退出码就报起了，不看有没有命令的进程号
#       Ⓥ 起单元之前没核日志时：起完回读认得出命令没跑起来（有退出码没有命令进程号）、退 1，list 注明没跑起来
#   DETACHED_RUN_BREAK=record-write-failure-as-vanished  单元里写登记失败时不写旁路文件
#       Ⓥ 命令退出后登记写不进：list 列成「登记写不进  selftest-<进程号>-v3」、报退出码 3，不列成没留退出码
#   DETACHED_RUN_BREAK=assignment-name-unchecked  start 不判 -- 之后开头那串带 = 的词的名字
#       Ⓜ -- 之后开头的赋值名不合写法（a-b=1、--force=1、1=1、a.b=1、FOO=1 a-b=1、--force a-b=1）：start 拒、退 2、带出路、没建登记、命令没跑；合法的 FOO=… 照起、设进命令
#   DETACHED_RUN_BREAK=archive-not-restored  起不成时（建不了这一次的登记目录、登记或日志写不进）不把挪走的同名旧登记挪回、挪不回也不出声
#       Ⓞ 建不了这一次的登记：start 退 1、挪走的同名旧登记挪回原处；挪回也失败时退 1、打一行「没挪回」带出路、旧登记留在 <名>@<秒数>
#   DETACHED_RUN_BREAK=same-second-restart-allowed  同名旧登记的起跑秒数不早于此刻也不等，当场起
#       Ⓨ 同名旧登记的起跑秒数不早于此刻：start 等到它之后再起、退 0，新登记的起跑秒数比旧的大，旧登记挪到 <名>@<旧秒数>
#   DETACHED_RUN_BREAK=watch-misses-short-runs  watch 只记清单里见到在跑的，不记它开跑之后起跑的登记
#       Ⓩ watch 开跑之后起了又跑完的短命令：一轮清单都没见它在跑，watch 退 0 时照报 selftest-<进程号>-z2 退出码 6
#   DETACHED_RUN_BREAK=scopes-left-running  单元里那个 bash 命令退出之后不找、不停命令留在别的 scope 里的进程
#       Ⓤ 命令留在别的 scope 里的进程：stop 停单元之后，那个 scope 也停了、sleep 不在了，日志里记了一行
#   DETACHED_RUN_BREAK=wall-clock-optional  start 没写 --wall-clock 时当 within-2h 照起
#       长活① 没写 --wall-clock、写了认不出的值：用法错退 2、带出路、没建登记
#   DETACHED_RUN_BREAK=long-run-unchecked  start --wall-clock over-2h 不判长活声明、照起
#       长活② over-2h、被起的脚本一样声明都没有（不分片的长跑法）：拒、退 5、带出路、没建登记
#       长活③ over-2h、只写了分片一样（半截声明）：拒、退 5、点名缺续跑与多机、没建登记
#       长活④ over-2h、多机一行写死线程与内存：拒、退 5、点名写死的额度、没建登记
#       长活⑤ over-2h、bash -c 一长串判不出被起的脚本：拒、退 5、出路写进一份脚本、没建登记
#       长活⑧ over-2h 三样齐了照起、命令跑了；within-2h 不读声明照起
#   长活声明判法那一半的开关在 .claude/hooks/lib_long_run.py（LIB_LONG_RUN_BREAK；只在自证起的进程里认，别处设了 start 与 long-run-check 调判法前摘掉它）：
#   LIB_LONG_RUN_BREAK=partial-accepted  三样写了一样就当齐了
#       长活③ over-2h、只写了分片一样（半截声明）：拒、退 5、点名缺续跑与多机、没建登记
#       长活⑥ long-run-check：分不了一行不到 8 个字、没点名缺的多机都拒（退 5），点名了的放（退 0）
#   LIB_LONG_RUN_BREAK=hosts-unchecked  多机一行不看内容
#       长活④ over-2h、多机一行写死线程与内存：拒、退 5、点名写死的额度、没建登记
#       长活⑨ long-run-check 多机一行按登记值读：规范句放（退 0），带句末句号的规范句放、规范句后另起一行写普通注释放；现算写法、否定句里提 host-resources.py、没点名登记值、除以此刻在跑的重活数、规范句后接任何说明（含数字的、不含数字的、写按此刻空闲现算 / 按负载降 / 除以此刻同时在跑的重活数 / host-resources.py 的）都拒（退 5）
#       长活⑩ 豁免表里登记的脚本文件头按现算读（退 0），没登记的同一份文件头照拒（没登记的也放了）
#   LIB_LONG_RUN_BREAK=hosts-canonical-unchecked  登记值读法下多机一行不核是不是规范句（写了就放，只剩写死的额度那一条）
#       长活⑨（现算写法、否定句里提 host-resources.py、没点名登记值、除以此刻在跑的重活数四份与规范句后接说明的六份都放了，退 0 而不是 5）；长活⑩（没登记的同一份文件头放了）
#   LIB_LONG_RUN_BREAK=hosts-suffix-accepted  登记值读法下规范句后面接任何说明都放行（只核以规范句起头）
#       长活⑨（规范句后接说明的六份放了，退 0 而不是 5）
#   LIB_LONG_RUN_BREAK=exempt-sha256-unchecked  豁免表里登记的 sha256 不比（在表里的脚本一律按现算读）
#       长活⑩（登记在表里、脚本改过一个字节的那份放了，退 0 而不是 1）
#   LIB_LONG_RUN_BREAK=header-live-reading  脚本文件头退回按现算读（点名 host-resources.py 才齐，登记值那一句反而拒）
#       长活⑦（包着的 full.sh 退 5）；长活⑧（over-2h 起 full.sh 被拒）；长活⑨（规范句退 5、现算写法退 0）；长活⑩（没登记的、登记了而 sha256 对不上的同一份文件头也放了）
#   LIB_LONG_RUN_BREAK=live-exempt-ignored  不查 .claude/long-run-live-hosts-exempt，登记了的脚本也按登记值读
#       长活⑩ 登记在豁免表里的脚本文件头按现算读（退 0），没登记的同一份文件头照拒（退 5）
#   LIB_LONG_RUN_BREAK=script-unresolved-accepted  判不出被起的脚本时当成齐了
#       长活⑤ over-2h、bash -c 一长串判不出被起的脚本：拒、退 5、出路写进一份脚本、没建登记
#   LIB_LONG_RUN_BREAK=unshardable-unchecked  分不了一行不看字数、不看点没点名缺的那几样
#       长活⑥ long-run-check：分不了一行不到 8 个字、没点名缺的多机都拒（退 5），点名了的放（退 0）
#   LIB_LONG_RUN_BREAK=wrappers-not-skipped  不跳过包装，第一份脚本就算被起的那份
#       长活⑦ long-run-check 跳过包装：run-with-memory-cap.sh 与 capped.sh 包着的三样齐的脚本退 0；包着没声明的脚本退 5、点名的是那份脚本
#   两道闸切参数那一半的开关在 .claude/hooks/lib_shell_words.py（不经令牌，自证把它带给自己起的 python3）：
#   LIB_SHELL_WORDS_BREAK=detached-run-force-kept  切 start 的参数时 -- 之后的 --force 不照 preflight 摘
#       Ⓟ -- 之后的 --force 照 preflight 摘：命令收到 a b，闸切出的词与登记里的命令逐字相同
#
# gate-similar: multi-host-run.sh 它把一条命令挪到第二台跑、调用方在前台等到结束，收到 TERM 连第二台那条一起停；这里只在本机把命令挂到用户级 systemd 下、调用方不等也不随它停，两份的生命周期相反
# gate-similar: run-with-memory-cap.sh 它也用用户级 systemd（scope 加 singlefs-heavy.slice），scope 里的命令仍是调用方的子进程、跟着调用方的进程树走；这里要的正是不跟着调用方走的 service
#
# admission: always 每次调都是对此刻的用户级 systemd 与登记目录做一件事，上一次的结论不替这一次作保
# run-condition: command bash systemd-run systemctl flock timeout python3
# run-condition: check systemctl --user show-environment >/dev/null 2>&1 :: 用户级 systemd 连不上（没有 XDG_RUNTIME_DIR 或 D-Bus）：在登录会话里跑，或请用户开 linger
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

detached_run_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
DETACHED_RUN_UNIT_PREFIX="singlefs-detached-"
DETACHED_RUN_SLICE="singlefs-detached.slice"
DETACHED_RUN_CARRIED_VARIABLES=(SINGLEFS_HEAVY_TESTS PATH HOME CARGO_HOME RUSTUP_HOME)
DETACHED_RUN_LABEL_PATTERN='^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$'
DETACHED_RUN_DEFAULT_EVERY_SECONDS=300
DETACHED_RUN_DEFAULT_STOP_TIMEOUT_SECONDS=180
DETACHED_RUN_QUERY_FAILED="query-failed"   # detached_run_unit_state 查不到时打的（不是 systemd 的 ActiveState 取值）
DETACHED_RUN_UNKNOWN_ALARM_ROUNDS=3         # watch 里同一条连续这么多次查不到就报警（理由在文件头 watch 那一段）
DETACHED_RUN_SAME_LABEL_WAIT_SECONDS=5      # 同名旧登记的起跑秒数不早于此刻时至多等这么久（文件头 start 那一段）
detached_run_state="${DETACHED_RUN_STATE_DIRECTORY:-${XDG_STATE_HOME:-$HOME/.local/state}/singlefs/detached-runs}"
detached_run_side_directory="${DETACHED_RUN_SIDE_DIRECTORY:-${XDG_RUNTIME_DIR:-/tmp}/singlefs-detached-runs}"
DETACHED_RUN_RUNNING_COUNT=0
DETACHED_RUN_RUNNING_RECORDS=()
DETACHED_RUN_UNKNOWN_COUNT=0
DETACHED_RUN_UNKNOWN_RECORDS=()
DETACHED_RUN_LOST_RUNNING_COUNT=0
DETACHED_RUN_LOST_UNKNOWN_KEYS=()
DETACHED_RUN_WATCH_SLEEP_PID=""
DETACHED_RUN_WATCH_CHILD_PID=""     # watch 起的、一轮轮列清单的那个子进程
DETACHED_RUN_WATCH_SIGNAL_NAME=""   # watch 收到的第一个信号（trap 只记它）
DETACHED_RUN_WATCH_SIGNAL_EXIT=""   # 与它对应的退出码
declare -A DETACHED_RUN_OBSERVED=()
declare -A DETACHED_RUN_OBSERVED_UNITS=()
declare -A DETACHED_RUN_UNKNOWN_ROUNDS=()

# 单元里那个 bash 的代码（四段拼起来；弄坏开关 exit-dropped 下不拼写退出码那一段，record-write-failure-as-vanished 下不拼写旁路那一段）：
#   参数是 <登记目录> <日志> <旁路文件> <命令…>。写登记失败的项记进 write_failed
read -r -d '' DETACHED_RUN_RUNNER_HEAD <<'RUNNER'
record="$1"; log="$2"; side="$3"; shift 3
write_failed=""
trap 'stop_requested=1' TERM INT HUP
export DETACHED_RUN_INSTANCE="$record#$$#$(date +%s%N)"
printf '%s\n' "$$" > "$record/.runner-pid.tmp" && mv -f -- "$record/.runner-pid.tmp" "$record/runner-pid" || write_failed+=" runner-pid"
"$BASH" -c 'printf "%s\n" "$$" > "$0/.pid.tmp" && mv -f -- "$0/.pid.tmp" "$0/pid" && exec env -- "$@"' "$record" "$@" >> "$log" 2>&1 < /dev/null
rc=$?
date +%s > "$record/.ended.tmp" && mv -f -- "$record/.ended.tmp" "$record/ended" || write_failed+=" ended"
TZ=Asia/Tokyo date +%F > "$record/.ended-date.tmp" && mv -f -- "$record/.ended-date.tmp" "$record/ended-date" || write_failed+=" ended-date"
RUNNER
read -r -d '' DETACHED_RUN_RUNNER_SWEEP <<'RUNNER'
declare -A left_scopes=()
own_cgroup="$(sed -n 's/^0:://p' "/proc/$$/cgroup")"
for environ in /proc/[0-9]*/environ; do
  left_pid="${environ#/proc/}"; left_pid="${left_pid%/environ}"
  [[ "$left_pid" != "$$" && -r "$environ" ]] || continue
  grep -qzxF -- "DETACHED_RUN_INSTANCE=$DETACHED_RUN_INSTANCE" "$environ" 2>/dev/null || continue
  left_cgroup="$(sed -n 's/^0:://p' "/proc/$left_pid/cgroup" 2>/dev/null)"
  if [[ -n "$left_cgroup" && "$left_cgroup" != "$own_cgroup" && "${left_cgroup##*/}" == *.scope ]]; then left_scopes["${left_cgroup##*/}"]=1; fi
done
for left_scope in "${!left_scopes[@]}"; do
  if systemctl --user stop --no-block -- "$left_scope"; then
    printf '== detached-run.sh：命令退出时还有进程留在 %s 里（不在这个单元的 cgroup 里），连同那个 scope 停了\n' "$left_scope" >> "$log"
  else
    printf '== detached-run.sh：命令退出时还有进程留在 %s 里，systemctl --user stop 没成，要自己停它\n' "$left_scope" >> "$log"
  fi
done
RUNNER
read -r -d '' DETACHED_RUN_RUNNER_EXIT <<'RUNNER'
printf '%s\n' "$rc" > "$record/.exit.tmp" && mv -f -- "$record/.exit.tmp" "$record/exit" || write_failed+=" exit"
RUNNER
read -r -d '' DETACHED_RUN_RUNNER_SIDE <<'RUNNER'
if [[ -n "$write_failed" ]]; then
  note="$(TZ=Asia/Tokyo date +%F) 单元里写登记失败（写不进：${write_failed# }），命令退出码 $rc"
  mkdir -p -- "${side%/*}" && printf '%s\n' "$note" > "$side"
  printf '== detached-run.sh：%s\n' "$note" >> "$log"
  printf 'detached-run.sh：%s（登记 %s）\n' "$note" "$record" >&2
fi
RUNNER
DETACHED_RUN_RUNNER_TAIL='exit "$rc"'

detached_run_break_is() { [[ ",${DETACHED_RUN_BREAK:-}," == *",$1,"* ]]; }

detached_run_inside_selftest() { # 这个进程是不是 --selftest 起的：登记目录里的 .selftest-token 与 DETACHED_RUN_SELFTEST_TOKEN 对得上
  [[ -n "${DETACHED_RUN_SELFTEST_TOKEN:-}" && -f "$detached_run_state/.selftest-token" ]] \
    && [[ "$(head -n 1 -- "$detached_run_state/.selftest-token")" == "$DETACHED_RUN_SELFTEST_TOKEN" ]]
}

detached_run_usage_error() { # <原因>：用法错，什么都没起，退 2
  echo "  ✗ detached-run.sh 用法错：$1" >&2
  echo "     → 怎么办：detached-run.sh start --wall-clock <within-2h|over-2h> --label <名> [--log <文件>] [--stop-timeout <秒>] -- <命令与参数…>、long-run-check -- <命令与参数…>、list、watch [--every <秒>]、stop <名>，或 --selftest；名只用字母、数字、点、减号与下划线，字母或数字起头，至多 64 个字" >&2
  exit 2
}

detached_run_query_failed() { # <单元名> <这一次什么都没做的说明>：systemctl --user show 查不到单元状态，退 1
  echo "  ✗ systemctl --user show 查不到单元 $1 的状态（失败、或什么都没打：用户级 systemd 连不上），$2" >&2
  echo "     → 怎么办：systemctl --user show-environment 看连不连得上（要 XDG_RUNTIME_DIR 与 D-Bus）；在登录会话里重跑，连上了再做" >&2
  exit 1
}

detached_run_bad_assignment_word() { # <命令与参数…>：开头那一串带 = 的词（单元里 env 把它们都当 名=值）里，第一个名字不合 shell 变量名写法的打出来、返回 0；都合写法返回 1
  local word
  for word in "$@"; do
    [[ "$word" == *=* ]] || return 1
    if [[ ! "${word%%=*}" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
      printf '%s\n' "$word"
      return 0
    fi
  done
  return 1
}

detached_run_restore_archive() { # <挪走的旧登记> <原处>：把这一次挪走的同名旧登记挪回原处，挪不回在 stderr 打一行带出路的提示；没挪过（第一个参数空）什么都不做
  [[ -n "$1" ]] || return 0
  detached_run_break_is archive-not-restored && return 0
  if [[ ! -e "$2" && ! -L "$2" ]] && mv -T -- "$1" "$2"; then
    echo "  · 挪走的同名旧登记挪回了 $2" >&2
    return 0
  fi
  echo "  ! 挪走的同名旧登记没挪回 $2（原处已有东西，或 mv 失败、报错在上面）：旧登记还在 $1，list 照读得到它的退出码" >&2
  echo "     → 怎么办：看登记目录 ${2%/*} 的权限与剩余空间（df -h）；原处是空的就 mv -T -- $1 $2 挪回，原处有东西先看它是什么再定" >&2
  return 0
}

detached_run_today() { TZ=Asia/Tokyo date +%F; }

detached_run_field() { # <登记目录> <项>：打出那一项的第一行，没有就什么都不打
  if [[ -f "$1/$2" ]]; then head -n 1 -- "$1/$2"; fi
  return 0
}

detached_run_write_field() { # <登记目录> <项> <值>：先写临时文件再改名换上
  printf '%s\n' "$3" > "$1/.$2.tmp" && mv -f -- "$1/.$2.tmp" "$1/$2"
}

detached_run_unit_state() { # <单元名>：打出它的 ActiveState（单元不在时 systemd 报 inactive）；systemctl 失败或什么都没打时打 query-failed
  local state
  if state="$(systemctl --user show -p ActiveState --value "$1" 2>/dev/null)" && [[ -n "$state" ]]; then
    printf '%s\n' "$state"
  elif ! detached_run_break_is query-failure-as-gone; then
    printf '%s\n' "$DETACHED_RUN_QUERY_FAILED"
  fi
  return 0
}

detached_run_linger_warning() { # 用户级 systemd 没开 linger、或读不到时 stderr 打一行警告；不拒
  local user linger
  detached_run_break_is linger-unchecked && return 0
  user="$(id -un)"
  if ! linger="$(loginctl show-user "$user" -p Linger --value 2>/dev/null)" || [[ -z "$linger" ]]; then
    echo "  ! 读不到 loginctl show-user $user 的 Linger：判不出最后一个登录会话结束时用户级 systemd 会不会连这里起的单元一起停；照起" >&2
  elif [[ "$linger" == no ]]; then
    echo "  ! 用户 $user 没开 linger（loginctl show-user 的 Linger=no）：最后一个登录会话结束时，用户级 systemd 连这里起的单元一起停，list 里记成「没留退出码」；照起。要它跨会话活着，请用户开 loginctl enable-linger $user" >&2
  fi
  return 0
}

detached_run_is_active_state() { # <ActiveState>
  case "$1" in active|activating|deactivating|reloading|refreshing) return 0 ;; esac
  return 1
}

detached_run_duration() { # <秒数>：打成 <天>d<时>h<分>m
  local seconds="$1" days hours minutes
  if ((seconds < 0)); then seconds=0; fi
  days=$((seconds / 86400)); hours=$((seconds % 86400 / 3600)); minutes=$((seconds % 3600 / 60))
  if ((days > 0)); then printf '%dd%02dh%02dm' "$days" "$hours" "$minutes"; else printf '%dh%02dm' "$hours" "$minutes"; fi
}

detached_run_size() { # <文件>：打出大小（1024 进制）
  local bytes
  if ! bytes="$(stat -c %s -- "$1" 2>/dev/null)"; then printf '（没有日志）'; return 0; fi
  numfmt --to=iec --suffix=B "$bytes" 2>/dev/null || printf '%sB' "$bytes"
}

detached_run_last_line() { # <日志>：打出最后一行不空的（回车切开的进度行取最后一段），至多 200 个字
  local line
  [[ -f "$1" ]] || return 0
  line="$(tail -c 8192 -- "$1" | tr '\r' '\n' | sed '/^[[:space:]]*$/d' | tail -n 1)"
  printf '%s' "${line:0:200}"
}

detached_run_list() { # 逐条列登记；DETACHED_RUN_RUNNING_COUNT / _RECORDS 记下还在跑的，DETACHED_RUN_UNKNOWN_COUNT / _RECORDS 记下查不到的
  local record label unit state status now started started_date ended rc log command main_pid pid line note side records=()
  local ended_count=0 vanished_count=0 write_failed_count=0 start_failed_count=0 unregistered_count=0 total=0 units_listing name load active sub
  local -A registered_units=()
  DETACHED_RUN_RUNNING_COUNT=0
  DETACHED_RUN_RUNNING_RECORDS=()
  DETACHED_RUN_UNKNOWN_COUNT=0
  DETACHED_RUN_UNKNOWN_RECORDS=()
  now="$(date +%s)"
  if [[ -d "$detached_run_state" ]]; then
    shopt -s nullglob
    records=("$detached_run_state"/*/)
    shopt -u nullglob
  fi
  for record in ${records[@]+"${records[@]}"}; do
    record="${record%/}"
    label="${record##*/}"
    [[ -f "$record/unit" ]] || continue
    total=$((total + 1))
    unit="$(detached_run_field "$record" unit)"
    if [[ -n "$unit" ]]; then registered_units["$unit"]=1; fi
    started="$(detached_run_field "$record" started)"
    [[ "$started" =~ ^[0-9]+$ ]] || started="$now"
    started_date="$(detached_run_field "$record" started-date)"
    log="$(detached_run_field "$record" log)"
    command="$(detached_run_field "$record" command)"
    rc="$(detached_run_field "$record" exit)"
    state=""
    if [[ -n "$rc" ]]; then
      status=ended
    elif [[ "$label" == *@* ]]; then   # 挪走的旧登记：单元名已归同名的新一次，不查
      if [[ -f "$record/start-failed" ]]; then status=start-failed; else status=vanished; fi
    elif [[ -f "$record/start-failed" ]] && detached_run_break_is start-failed-trusted; then
      status=start-failed
    else
      state="$(detached_run_unit_state "$unit")"
      if detached_run_is_active_state "$state"; then
        status=running
      elif [[ "$state" == "$DETACHED_RUN_QUERY_FAILED" ]]; then
        status=unknown
      elif [[ -f "$record/start-failed" ]]; then
        status=start-failed
      else
        status=vanished
      fi
    fi
    side="$(detached_run_field "$record" write-failure-file)"
    if [[ "$status" == vanished && -n "$side" && -f "$side" ]]; then status=write-failed; fi
    case "$status" in
      running)
        DETACHED_RUN_RUNNING_COUNT=$((DETACHED_RUN_RUNNING_COUNT + 1))
        DETACHED_RUN_RUNNING_RECORDS+=("$record")
        main_pid="$(systemctl --user show -p MainPID --value "$unit" 2>/dev/null)"
        pid="$(detached_run_field "$record" pid)"
        note=""
        if [[ -f "$record/start-failed" || -f "$record/systemd-run-failed" ]]; then
          note="  （systemd-run 报过错而单元照起：$(detached_run_field "$record" systemd-run-failed)$(detached_run_field "$record" start-failed)）"
        fi
        echo "  · 还在跑  $label  单元 $unit（$state）  主进程 ${main_pid:-?}  命令进程 ${pid:-?}  起跑 ${started_date:-?}  已跑 $(detached_run_duration $((now - started)))  日志 $(detached_run_size "$log") $log$note"
        ;;
      ended)
        ended_count=$((ended_count + 1))
        ended="$(detached_run_field "$record" ended)"
        [[ "$ended" =~ ^[0-9]+$ ]] || ended="$now"
        note=""
        if [[ -f "$record/stop-requested" ]]; then note="（detached-run.sh stop 停的）"; fi
        if [[ ! -f "$record/pid" ]] && ! detached_run_break_is ran-without-pid-trusted; then
          note+="（命令没跑起来：单元里写不进登记或日志，这是单元里那个 bash 的退出码）"
        fi
        echo "  · 已结束  $label  退出码 $rc$note  单元 $unit  起跑 ${started_date:-?}  结束 $(detached_run_field "$record" ended-date)  用时 $(detached_run_duration $((ended - started)))  日志 $(detached_run_size "$log") $log"
        ;;
      write-failed)
        write_failed_count=$((write_failed_count + 1))
        echo "  · 登记写不进  $label  单元 $unit 已不在：$(head -n 1 -- "$side")（旁路文件 $side）  起跑 ${started_date:-?}  日志 $(detached_run_size "$log") $log"
        ;;
      vanished)
        vanished_count=$((vanished_count + 1))
        note="主进程被 KILL、用户级 systemd 退出（没开 linger 时最后一个登录会话结束）或机器重启"
        if [[ -f "$record/stop-killed" ]]; then note="detached-run.sh stop 停的：$(detached_run_field "$record" stop-killed)"; fi
        echo "  · 没留退出码  $label  单元 $unit 已不在${state:+（ActiveState=$state）}：$note  起跑 ${started_date:-?}  日志 $(detached_run_size "$log") $log"
        ;;
      unknown)
        DETACHED_RUN_UNKNOWN_COUNT=$((DETACHED_RUN_UNKNOWN_COUNT + 1))
        DETACHED_RUN_UNKNOWN_RECORDS+=("$record")
        echo "  · 查不到  $label  单元 $unit：systemctl --user show 失败（用户级 systemd 连不上），不知道还在不在跑，不算结束  起跑 ${started_date:-?}  日志 $(detached_run_size "$log") $log"
        ;;
      start-failed)
        start_failed_count=$((start_failed_count + 1))
        echo "  · 没起成  $label  单元 $unit：$(detached_run_field "$record" start-failed)，命令一行都没跑  起跑 ${started_date:-?}  日志 $(detached_run_size "$log") $log"
        ;;
    esac
    echo "      命令：${command:0:200}"
    line="$(detached_run_last_line "$log")"
    echo "      日志最后一行：${line:-（空）}"
  done
  if ! detached_run_break_is unregistered-units-hidden; then
    if units_listing="$(systemctl --user list-units --all --plain --no-legend "${DETACHED_RUN_UNIT_PREFIX}*" 2>/dev/null)"; then
      while read -r name load active sub _; do
        [[ "$name" == "${DETACHED_RUN_UNIT_PREFIX}"*.service && -z "${registered_units[$name]:-}" ]] || continue
        unregistered_count=$((unregistered_count + 1))
        echo "  · 登记里没有  单元 $name（$load / $active / $sub）：它的登记被删了、或在别的登记目录里；detached-run.sh 停不到它，看它用 systemctl --user status $name，怎么停交主 agent 判断"
      done <<< "$units_listing"
    else
      echo "  · 查不到单元清单：systemctl --user list-units 失败（用户级 systemd 连不上），登记里没有的单元列不出来"
    fi
  fi
  echo "  ✓ detached-run.sh list：还在跑、没结束的 $DETACHED_RUN_RUNNING_COUNT 条，已结束 $ended_count 条，没留退出码 $vanished_count 条，登记写不进 $write_failed_count 条，查不到 $DETACHED_RUN_UNKNOWN_COUNT 条，没起成 $start_failed_count 条；登记里没有的单元 $unregistered_count 个（登记 $total 条，登记目录 $detached_run_state）"
}

detached_run_long_run_check() { # <命令与参数…>：照 over-2h 判被起脚本的长活声明（判法 .claude/hooks/lib_long_run.py，它的 ✓ 打 stdout、✗ 与出路打 stderr）；
  # 齐了返回 0；不齐、判不出、判法起不来返回 5。LIB_LONG_RUN_BREAK 只在自证起的进程里带给判法，别处摘掉
  local lib="${detached_run_script%/*}/../../.claude/hooks/lib_long_run.py" rc unset_break=(-u LIB_LONG_RUN_BREAK)
  if detached_run_inside_selftest; then unset_break=(); fi
  if env ${unset_break[@]+"${unset_break[@]}"} python3 "$lib" script "$PWD" -- "$@"; then return 0; else rc=$?; fi
  if ((rc != 1)); then
    echo "  ✗ 长活声明判不了：$lib 退 $rc（判法起不来，或 LIB_LONG_RUN_BREAK 写错），这一次什么都没起" >&2
    echo "     → 怎么办：看它上面打的那几行，修好 .claude/hooks/lib_long_run.py（或去掉写错的 LIB_LONG_RUN_BREAK）再起" >&2
  fi
  return 5
}

detached_run_start() { # start 的参数
  local label="" log="" separator_seen=0 record unit state archive="" started name runner main_pid pid tries=0 carried=() setenv=() properties=() rc waited=0
  local stop_timeout="$DETACHED_RUN_DEFAULT_STOP_TIMEOUT_SECONDS" stop_timeout_given="" unwritable="" side query_deferred="" bad_assignment wall_clock=""
  while (($#)); do
    case "$1" in
      --label) (($# >= 2)) || detached_run_usage_error "--label 后面要一个名"; label="$2"; shift 2 ;;
      --label=*) label="${1#--label=}"; shift ;;
      --log) (($# >= 2)) || detached_run_usage_error "--log 后面要一个文件"; log="$2"; shift 2 ;;
      --log=*) log="${1#--log=}"; shift ;;
      --stop-timeout) (($# >= 2)) || detached_run_usage_error "--stop-timeout 后面要秒数"; stop_timeout_given=1; stop_timeout="$2"; shift 2 ;;
      --stop-timeout=*) stop_timeout_given=1; stop_timeout="${1#--stop-timeout=}"; shift ;;
      --wall-clock) (($# >= 2)) || detached_run_usage_error "--wall-clock 后面要 within-2h 或 over-2h"; wall_clock="$2"; shift 2 ;;
      --wall-clock=*) wall_clock="${1#--wall-clock=}"; shift ;;
      --) separator_seen=1; shift; break ;;
      *) detached_run_usage_error "start 认不出 $1（命令要写在 -- 之后）" ;;
    esac
  done
  ((separator_seen)) || detached_run_usage_error "start 没有 --：命令要写在 -- 之后"
  (($#)) || detached_run_usage_error "start 的 -- 之后没有命令"
  # 这里的 "$@" 已经是 preflight 摘掉整词 --force 之后、start 真收到的参数
  if ! detached_run_break_is assignment-name-unchecked && bad_assignment="$(detached_run_bad_assignment_word "$@")"; then
    echo "  ✗ detached-run.sh start：-- 之后开头的「$bad_assignment」带 =、而 = 前面不是合法的 shell 变量名；单元里经 env 起命令，env 会把它当 名=值、照跑后面那条，两道闸却把它当命令词、看不到后面那条。这一次什么都没起（退 2）" >&2
    echo "     → 怎么办：要设环境变量写成 -- env 名=值 <命令>，名字只许字母、数字、下划线、不以数字起头" >&2
    exit 2
  fi
  [[ -n "$label" ]] || detached_run_usage_error "start 没给 --label"
  [[ "$label" =~ $DETACHED_RUN_LABEL_PATTERN ]] || detached_run_usage_error "名「$label」不合写法"
  [[ "$stop_timeout" =~ ^[1-9][0-9]*$ ]] || detached_run_usage_error "--stop-timeout 要正整数秒，给的是 $stop_timeout"
  # 预计挂钟与长活声明（文件头 start 的 --wall-clock 一段）：判在拿锁、挪旧登记之前
  if [[ -z "$wall_clock" ]] && detached_run_break_is wall-clock-optional; then wall_clock=within-2h; fi
  case "$wall_clock" in
    within-2h) ;;
    over-2h) if ! detached_run_break_is long-run-unchecked; then detached_run_long_run_check "$@" >&2 || exit 5; fi ;;
    "") detached_run_usage_error "start 没写 --wall-clock：预计挂钟 2 小时内写 --wall-clock within-2h；超过的写 --wall-clock over-2h，被起的脚本文件头要写长活声明（detached-run.sh 文件头 start 的 --wall-clock 一段）" ;;
    *) detached_run_usage_error "--wall-clock 只认 within-2h 或 over-2h，给的是「$wall_clock」" ;;
  esac
  record="$detached_run_state/$label"
  unit="${DETACHED_RUN_UNIT_PREFIX}${label}.service"
  detached_run_linger_warning
  if [[ -z "$log" ]]; then
    log="$record/log"
  else
    [[ "$log" == /* ]] || log="$PWD/$log"
    [[ -d "$(dirname -- "$log")" ]] || detached_run_usage_error "日志 $log 所在的目录不在"
    if ! detached_run_break_is log-unchecked && { [[ ! -w "$(dirname -- "$log")" ]] || [[ -e "$log" && ! -w "$log" ]]; }; then
      echo "  ✗ 日志 $log 写不进（所在目录或日志本身不可写），什么都没起" >&2
      echo "     → 怎么办：--log 换一个写得进的地方（不写 --log 就落在登记目录里），或修好那个目录的权限再起" >&2
      exit 1
    fi
  fi
  if ! mkdir -p -- "$detached_run_state" || ! : >> "$detached_run_state/.lock"; then
    echo "  ✗ 建不了登记目录 $detached_run_state 或它的锁文件，什么都没起" >&2
    echo "     → 怎么办：看 \$XDG_STATE_HOME（没设是 ~/.local/state）可不可写、盘满没满，修好再起" >&2
    exit 1
  fi
  exec 9>>"$detached_run_state/.lock"
  if ! flock -w 60 9; then
    echo "  ✗ 60 秒没拿到登记目录的锁（$detached_run_state/.lock），什么都没起" >&2
    echo "     → 怎么办：有别的 detached-run.sh start 卡在里面；用 detached-run.sh list 看一眼，等它起完再起" >&2
    exit 1
  fi
  state="$(detached_run_unit_state "$unit")"
  if [[ "$state" == "$DETACHED_RUN_QUERY_FAILED" ]] && detached_run_break_is query-check-after-archive; then query_deferred="$state"; state=""; fi
  if [[ "$state" == "$DETACHED_RUN_QUERY_FAILED" ]]; then
    detached_run_query_failed "$unit" "判不出同名的还在不在跑：这一次什么都没起，旧登记没动"
  fi
  if detached_run_is_active_state "$state" && ! detached_run_break_is same-label-allowed; then
    echo "  ✗ 名「$label」还在跑（单元 $unit，ActiveState=$state，命令进程 $(detached_run_field "$record" pid)），这一次什么都没起" >&2
    echo "     → 怎么办：换一个名再起；看它跑到哪了用 detached-run.sh list；确实要停它用 detached-run.sh stop $label（只停登记的那一个单元）" >&2
    exit 3
  fi
  if [[ "$state" == failed ]]; then systemctl --user reset-failed "$unit" 2>/dev/null; fi
  if [[ -e "$record" ]]; then
    started="$(detached_run_field "$record" started)"
    # 同一秒里同名重起：起跑秒数是挪去的名字与 watch 记观察的键，等到旧一次之后那一秒再起（文件头 start 那一段）
    if [[ "$started" =~ ^[0-9]+$ ]] && ! detached_run_break_is same-second-restart-allowed; then
      while (($(date +%s) <= started)) && ((waited < DETACHED_RUN_SAME_LABEL_WAIT_SECONDS * 10)); do
        sleep 0.1
        waited=$((waited + 1))
      done
      if (($(date +%s) <= started)); then
        echo "  ✗ 同名旧登记 $record 的起跑秒数 $started 不早于此刻 $(date +%s)（等了 $DETACHED_RUN_SAME_LABEL_WAIT_SECONDS 秒还是），什么都没起，旧登记没动" >&2
        echo "     → 怎么办：看机器时钟是不是往回拨过；换一个名起，或旧登记没用了自己挪走它再起" >&2
        exit 1
      fi
    fi
    archive="$record@${started:-$(date +%s)}"
    [[ ! -e "$archive" ]] || archive="$archive.$$"
    if ! mv -- "$record" "$archive"; then
      echo "  ✗ 同名的旧登记 $record 挪不走，什么都没起" >&2
      echo "     → 怎么办：看登记目录的权限；旧登记没用了就自己删掉它，再起" >&2
      exit 1
    fi
    echo "  · 同名的旧登记已结束，挪到 $archive 留着" >&2
  fi
  if [[ -n "$query_deferred" ]]; then   # 只有弄坏开关 query-check-after-archive 下走到这里（旧登记已经挪走了）
    detached_run_query_failed "$unit" "判不出同名的还在不在跑：这一次什么都没起"
  fi
  if ! mkdir -- "$record"; then
    detached_run_restore_archive "$archive" "$record"
    echo "  ✗ 建不了登记 $record，什么都没起" >&2
    echo "     → 怎么办：看登记目录的权限与剩余空间，修好再起" >&2
    exit 1
  fi
  if ! detached_run_break_is environment-dropped; then
    for name in "${DETACHED_RUN_CARRIED_VARIABLES[@]}"; do
      if detached_run_break_is path-home-dropped && [[ "$name" == PATH || "$name" == HOME ]]; then continue; fi
      if [[ -n "${!name+set}" ]]; then
        setenv+=("--setenv=$name=${!name}")
        carried+=("$name")
      fi
    done
  fi
  started="$(date +%s)"
  side="$detached_run_side_directory/$label@$started.record-write-failed"
  detached_run_write_field "$record" unit "$unit" || unwritable+=" unit"
  detached_run_write_field "$record" command "$(printf '%q ' "$@")" || unwritable+=" command"
  detached_run_write_field "$record" cwd "$PWD" || unwritable+=" cwd"
  detached_run_write_field "$record" environment "${carried[*]:-（一样都没带）}" || unwritable+=" environment"
  detached_run_write_field "$record" started "$started" || unwritable+=" started"
  detached_run_write_field "$record" started-date "$(detached_run_today)" || unwritable+=" started-date"
  detached_run_write_field "$record" log "$log" || unwritable+=" log"
  detached_run_write_field "$record" stop-timeout "$stop_timeout" || unwritable+=" stop-timeout"
  detached_run_write_field "$record" write-failure-file "$side" || unwritable+=" write-failure-file"
  if ! printf '== detached-run.sh start %s（%s，单元 %s）\n   命令：%s\n   目录：%s\n   带进去的环境：%s\n   停机时限：%s 秒\n' \
      "$label" "$(detached_run_today)" "$unit" "$(printf '%q ' "$@")" "$PWD" "${carried[*]:-（一样都没带）}" "$stop_timeout" >> "$log" \
      && ! detached_run_break_is log-unchecked; then
    unwritable+=" 日志 $log"
  fi
  if [[ -n "$unwritable" ]]; then
    rm -rf -- "${record:?}"
    detached_run_restore_archive "$archive" "$record"
    echo "  ✗ 写不进（${unwritable# }），什么都没起；这一次建的登记 $record 删了" >&2
    echo "     → 怎么办：看登记目录 $detached_run_state 与日志所在目录的权限、剩余空间（df -h），修好再起" >&2
    exit 1
  fi
  runner="$DETACHED_RUN_RUNNER_HEAD"
  if ! detached_run_break_is scopes-left-running; then runner+=$'\n'"$DETACHED_RUN_RUNNER_SWEEP"; fi
  if ! detached_run_break_is exit-dropped; then runner+=$'\n'"$DETACHED_RUN_RUNNER_EXIT"; fi
  if ! detached_run_break_is record-write-failure-as-vanished; then runner+=$'\n'"$DETACHED_RUN_RUNNER_SIDE"; fi
  runner+=$'\n'"$DETACHED_RUN_RUNNER_TAIL"
  if ! detached_run_break_is stop-timeout-ignored; then properties+=("--property=TimeoutStopSec=$stop_timeout"); fi
  if detached_run_break_is default-stop-timeout-unset && [[ -z "$stop_timeout_given" ]]; then properties=(); fi
  if detached_run_break_is in-caller-tree; then
    ( exec "$BASH" -c "$runner" detached-run-runner "$record" "$log" "$side" "$@" ) < /dev/null > /dev/null 2>&1 9>&- &
  elif systemd-run --user --quiet --collect --service-type=exec --expand-environment=no \
      --unit="$unit" --slice="$DETACHED_RUN_SLICE" --description="singlefs detached-run $label" \
      --working-directory="$PWD" ${properties[@]+"${properties[@]}"} ${setenv[@]+"${setenv[@]}"} \
      -- "$BASH" -c "$runner" detached-run-runner "$record" "$log" "$side" "$@" 9>&-; then
    :
  else
    rc=$?
    state=""
    if ! detached_run_break_is start-failure-not-rechecked; then state="$(detached_run_unit_state "$unit")"; fi
    if detached_run_is_active_state "$state"; then
      detached_run_write_field "$record" systemd-run-failed "$(detached_run_today) systemd-run 退 $rc，单元照样起来了（ActiveState=$state）"
      echo "  ! systemd-run 退 $rc（它的报错在上面），但单元 $unit 起来了（ActiveState=$state）：照在跑记，登记里写了 systemd-run-failed" >&2
    else
      if ! detached_run_break_is start-failure-unmarked; then
        detached_run_write_field "$record" start-failed "$(detached_run_today) systemd-run 退 $rc${state:+，回读单元 ActiveState=$state}"
      fi
      echo "  ✗ systemd-run 没起成单元 $unit（退 $rc，它的报错在上面；回读单元 ActiveState=${state:-?}），命令一行都没跑；登记 $record 里记了 start-failed，list 列成「没起成」" >&2
      echo "     → 怎么办：bash $detached_run_script list 看同名的还在不在（list 按单元名再查一次，单元其实起来了的列成还在跑）；systemctl --user status $unit 看单元；用户级 systemd 连不上就在登录会话里重起" >&2
      exit 1
    fi
  fi
  exec 9>&-
  while [[ ! -f "$record/pid" && ! -f "$record/exit" ]] && ((tries < 100)); do
    sleep 0.1
    tries=$((tries + 1))
  done
  pid="$(detached_run_field "$record" pid)"
  state="$(detached_run_unit_state "$unit")"
  main_pid="$(systemctl --user show -p MainPID --value "$unit" 2>/dev/null)"
  if [[ -z "$pid" && ! -f "$record/exit" ]]; then
    echo "  ✗ 单元 $unit 起了，10 秒里登记 $record 没出现命令的进程号（ActiveState=${state:-?}），判没起成" >&2
    echo "     → 怎么办：看日志 $log 与 systemctl --user status $unit；单元还在就用 detached-run.sh stop $label 停它再查" >&2
    exit 1
  fi
  if [[ -z "$pid" ]] && ! detached_run_break_is ran-without-pid-trusted; then
    echo "  ✗ 单元 $unit 起了，但命令没跑起来：登记 $record 里有退出码 $(detached_run_field "$record" exit)、没有命令的进程号（单元里写不进登记或日志 $log），判没起成" >&2
    echo "     → 怎么办：看日志 $log 所在目录与登记目录的权限、剩余空间（df -h），修好再起；单元里那个 bash 的报错用 journalctl --user -u $unit 看" >&2
    exit 1
  fi
  echo "  ✓ 起了「$label」：单元 $unit（ActiveState=${state:-?}），主进程 ${main_pid:-?}，命令进程 ${pid:-（已经跑完）}，日志 $log"
  echo "    带进去的环境：${carried[*]:-（一样都没带）}；工作目录 $PWD；停机时限 $stop_timeout 秒；登记 $record"
  echo "    → 会话里只上报：run_in_background 起 bash $detached_run_script watch；看一眼 bash $detached_run_script list；要停 bash $detached_run_script stop $label"
  echo "      （stop 等命令收尾完才回来，至多停机时限加 5 秒：前台调用时 Bash 的 timeout 给到 $(((stop_timeout + 30) * 1000)) 毫秒；超过 240000 的用 run_in_background 起 stop）"
}

detached_run_observe() { # <登记目录>…：记下观察到的这几条，键是「登记目录@起跑秒数」（与同名重起时旧登记挪去的名字同一个写法），连同单元名。
  #   列完清单到记观察之间登记可能被挪走：读不到起跑秒数而登记目录已经不在的，这一次不记；读不到单元名的，不拿空的盖掉上一次记下的
  #   （盖掉了，watch 就按不出单元名去查、把还在跑的当成结束）。弄坏开关 observe-overwrites-unit 下照改前的写法读到什么记什么
  local record key started unit
  for record in "$@"; do
    started="$(detached_run_field "$record" started)"
    if [[ -z "$started" && ! -d "$record" ]] && ! detached_run_break_is observe-overwrites-unit; then continue; fi
    key="$record@$started"
    DETACHED_RUN_OBSERVED["$key"]=1
    unit="$(detached_run_field "$record" unit)"
    if [[ -n "$unit" || -z "${DETACHED_RUN_OBSERVED_UNITS[$key]:-}" ]] || detached_run_break_is observe-overwrites-unit; then
      DETACHED_RUN_OBSERVED_UNITS["$key"]="$unit"
    fi
  done
}

detached_run_observe_started_since() { # <秒数>：起跑秒数不早于它的登记（连挪走的 <名>@<秒数>）都记成观察到的，键与 detached_run_observe 同一个写法。
  #   两次清单之间起了又跑完的短命令一轮清单都没见它在跑，不这样记就报不出它的退出码（文件头 watch 那一段）
  local since="$1" record base primary started key
  [[ "$since" =~ ^[0-9]+$ ]] || return 0
  shopt -s nullglob
  local records=("$detached_run_state"/*/)
  shopt -u nullglob
  for record in ${records[@]+"${records[@]}"}; do
    record="${record%/}"
    [[ -f "$record/unit" ]] || continue
    started="$(detached_run_field "$record" started)"
    [[ "$started" =~ ^[0-9]+$ ]] && ((started >= since)) || continue
    base="${record##*/}"
    primary="$detached_run_state/${base%%@*}"
    key="$primary@$started"
    [[ -z "${DETACHED_RUN_OBSERVED[$key]:-}" ]] || continue
    DETACHED_RUN_OBSERVED["$key"]=1
    DETACHED_RUN_OBSERVED_UNITS["$key"]="$(detached_run_field "$record" unit)"
  done
  return 0
}

detached_run_watch_lost() { # 观察到的各条里登记不见了（被删、被挪走）而同名没有新登记的：按记下的单元名查状态。
  #   还在跑的数进 DETACHED_RUN_LOST_RUNNING_COUNT，查不到的键记进 DETACHED_RUN_LOST_UNKNOWN_KEYS，各打一行；单元不在了的不打（结局里报）
  local key record unit state label
  DETACHED_RUN_LOST_RUNNING_COUNT=0
  DETACHED_RUN_LOST_UNKNOWN_KEYS=()
  detached_run_break_is watch-trusts-registry && return 0
  while IFS= read -r key; do
    [[ -n "$key" ]] || continue
    record="$(detached_run_observed_record "$key")"
    [[ -f "$record/unit" || -f "${key%@*}/unit" ]] && continue   # 登记还在（原处或挪走的那一份），或同名已有新一次（单元名归它）
    unit="${DETACHED_RUN_OBSERVED_UNITS[$key]:-}"
    [[ -n "$unit" ]] || continue
    label="${key%@*}"
    label="${label##*/}"
    state="$(detached_run_unit_state "$unit")"
    if detached_run_is_active_state "$state"; then
      DETACHED_RUN_LOST_RUNNING_COUNT=$((DETACHED_RUN_LOST_RUNNING_COUNT + 1))
      echo "  · 还在跑（登记里没有）  $label  单元 $unit（$state）：观察到的登记 ${key%@*} 不见了（被删或挪走），单元还在跑；watch 接着等它，退出码读不到"
    elif [[ "$state" == "$DETACHED_RUN_QUERY_FAILED" ]]; then
      DETACHED_RUN_LOST_UNKNOWN_KEYS+=("$key")
      echo "  · 查不到（登记里没有）  $label  单元 $unit：观察到的登记 ${key%@*} 不见了，systemctl --user show 也查不到，不知道还在不在跑，不算结束"
    fi
  done < <(printf '%s\n' "${!DETACHED_RUN_OBSERVED[@]}" | sort)
  return 0
}

detached_run_watch_alarm() { # <这一次查不到的键>…：观察到的各条连续查不到的次数加一（这一次查得到的清零），到 DETACHED_RUN_UNKNOWN_ALARM_ROUNDS 次的报警；不退出、不停单元
  local key alarmed=0 label
  local -A unknown_now=()
  for key in "$@"; do unknown_now["$key"]=1; done
  for key in "${!DETACHED_RUN_OBSERVED[@]}"; do
    if [[ -n "${unknown_now[$key]:-}" ]]; then
      DETACHED_RUN_UNKNOWN_ROUNDS["$key"]=$((${DETACHED_RUN_UNKNOWN_ROUNDS[$key]:-0} + 1))
    else
      DETACHED_RUN_UNKNOWN_ROUNDS["$key"]=0
    fi
  done
  detached_run_break_is unknown-alarm-missing && return 0
  while IFS= read -r key; do
    [[ -n "$key" ]] || continue
    ((${DETACHED_RUN_UNKNOWN_ROUNDS[$key]:-0} >= DETACHED_RUN_UNKNOWN_ALARM_ROUNDS)) || continue
    label="${key%@*}"
    label="${label##*/}"
    echo "  ! 报警  $label  单元 ${DETACHED_RUN_OBSERVED_UNITS[$key]:-?}：连续 ${DETACHED_RUN_UNKNOWN_ROUNDS[$key]} 次查不到单元状态（用户级 systemd 一直连不上），watch 接着查，不退出、不停单元"
    alarmed=$((alarmed + 1))
  done < <(printf '%s\n' "${!DETACHED_RUN_OBSERVED[@]}" | sort)
  if ((alarmed > 0)); then
    echo "     → 怎么办：systemctl --user show-environment 看连不连得上（要 XDG_RUNTIME_DIR 与 D-Bus）；连不上请用户在登录会话里看用户级 systemd，单元与命令可能还在跑，别当它们结束了"
  fi
  return 0
}

detached_run_observed_record() { # <键>：观察到的那一次现在的登记目录：同名没重起过就是原处，重起过就是挪走的 <名>@<起跑秒数>（撞名时带 .<进程号>）
  local record="${1%@*}" started="${1##*@}" archive
  if detached_run_break_is watch-keys-by-path || [[ "$(detached_run_field "$record" started)" == "$started" ]]; then
    printf '%s\n' "$record"
    return 0
  fi
  for archive in "$record@$started" "$record@$started".*; do
    if [[ -d "$archive" ]]; then printf '%s\n' "$archive"; return 0; fi
  done
  printf '%s\n' "$record@$started"
}

detached_run_watch_outcomes() { # 观察到的各条的退出码
  local key record label rc note moved side count=0
  if ((${#DETACHED_RUN_OBSERVED[@]} == 0)); then
    echo "  ✓ detached-run.sh watch：开跑时就没有还在跑的，观察到 0 条"
    return 0
  fi
  echo "  · 观察到的各条："
  while IFS= read -r key; do
    [[ -n "$key" ]] || continue
    count=$((count + 1))
    record="$(detached_run_observed_record "$key")"
    label="${key%@*}"
    label="${label##*/}"
    if [[ ! -f "$record/unit" ]]; then
      echo "  · $label 没留退出码（观察到的登记 ${key%@*} 不在了：被删或挪走；单元 ${DETACHED_RUN_OBSERVED_UNITS[$key]:-?} 已不在跑，退出码读不到）"
      continue
    fi
    moved=""
    if [[ "$record" != "${key%@*}" ]]; then moved="（之后同名又起过一次，这一次的登记挪到了 $record）"; fi
    rc="$(detached_run_field "$record" exit)"
    side="$(detached_run_field "$record" write-failure-file)"
    note=""
    if [[ -f "$record/stop-requested" ]]; then note="（detached-run.sh stop 停的）"; fi
    if [[ -n "$rc" ]]; then
      echo "  · $label 退出码 $rc$note$moved"
    elif [[ -n "$side" && -f "$side" ]]; then
      echo "  · $label 登记写不进：$(head -n 1 -- "$side")$note$moved"
    elif [[ -f "$record/stop-killed" ]]; then
      echo "  · $label 没留退出码（detached-run.sh stop 停的：$(detached_run_field "$record" stop-killed)）$moved"
    else
      echo "  · $label 没留退出码（单元已不在：主进程被 KILL、用户级 systemd 退出或机器重启）$moved"
    fi
  done < <(printf '%s\n' "${!DETACHED_RUN_OBSERVED[@]}" | sort)
  echo "  ✓ detached-run.sh watch：观察到的 $count 条都结束了，各自的退出码在上面"
}

detached_run_watch_on_signal() { # <信号名> <退出码>：watch 进程的 trap，只记下收到的第一个信号；停子进程、打最后一次清单在主流程里做
  if [[ -z "$DETACHED_RUN_WATCH_SIGNAL_NAME" ]]; then
    DETACHED_RUN_WATCH_SIGNAL_NAME="$1"
    DETACHED_RUN_WATCH_SIGNAL_EXIT="$2"
  fi
  return 0
}

detached_run_watch_stopped() { # <信号名> <退出码>：先打最后一次清单再退；不停任何单元
  local record unit stops=0
  trap '' TERM INT HUP
  if [[ -n "$DETACHED_RUN_WATCH_SLEEP_PID" ]]; then kill "$DETACHED_RUN_WATCH_SLEEP_PID" 2>/dev/null; fi
  echo "  · detached-run.sh watch 收到 $1：退出前最后一次清单，「还在跑」那几条就是还在跑、没结束的脚本（它们照跑，watch 没停任何单元）"
  if detached_run_break_is "watch-stops-units-on-${1,,}"; then stops=1; fi
  if detached_run_break_is watch-stops-units; then
    stops=1
  fi
  if ((stops)); then
    detached_run_list > /dev/null   # 还在跑的那几条：列清单的是子进程时，这个进程里没有它记下的
    for record in ${DETACHED_RUN_RUNNING_RECORDS[@]+"${DETACHED_RUN_RUNNING_RECORDS[@]}"}; do
      unit="$(detached_run_field "$record" unit)"
      systemctl --user stop "$unit"
    done
  fi
  detached_run_list
  exit "$2"
}

detached_run_watch_loop() { # <秒> <in-process|child>：一轮轮列清单、记观察；还在跑的与查不到的一条都不剩时报各条退出码、退 0。
  #   child：在 watch 起的子进程里跑，不接信号（watch 进程接），两轮之间用 read -t 等、不起 sleep（子进程被停时不留下没人管的 sleep）；
  #   in-process：弄坏开关 watch-stops-in-trap 下在 watch 进程里跑（改前的写法），两轮之间起一个 sleep 等
  local every="$1" how="$2" since="${3:-}" round=0 record unknown_keys
  while :; do
    round=$((round + 1))
    echo "  == 第 $round 次（$(detached_run_today)）"
    detached_run_list
    # 弄坏开关 watch-skips-unknown 下只记还在跑的；记两样的那两行保持原文（第二轮攻方 v4_mutations.py 按原文改它造错实现，见文件头的开关表）
    if detached_run_break_is watch-skips-unknown; then detached_run_observe ${DETACHED_RUN_RUNNING_RECORDS[@]+"${DETACHED_RUN_RUNNING_RECORDS[@]}"}; else
    detached_run_observe ${DETACHED_RUN_RUNNING_RECORDS[@]+"${DETACHED_RUN_RUNNING_RECORDS[@]}"} \
      ${DETACHED_RUN_UNKNOWN_RECORDS[@]+"${DETACHED_RUN_UNKNOWN_RECORDS[@]}"}
    fi
    detached_run_break_is watch-misses-short-runs || detached_run_observe_started_since "$since"
    detached_run_watch_lost
    unknown_keys=(${DETACHED_RUN_LOST_UNKNOWN_KEYS[@]+"${DETACHED_RUN_LOST_UNKNOWN_KEYS[@]}"})
    for record in ${DETACHED_RUN_UNKNOWN_RECORDS[@]+"${DETACHED_RUN_UNKNOWN_RECORDS[@]}"}; do
      unknown_keys+=("$record@$(detached_run_field "$record" started)")
    done
    detached_run_watch_alarm ${unknown_keys[@]+"${unknown_keys[@]}"}
    if ((DETACHED_RUN_UNKNOWN_COUNT + ${#DETACHED_RUN_LOST_UNKNOWN_KEYS[@]} > 0)); then
      echo "  · 查不到的 $((DETACHED_RUN_UNKNOWN_COUNT + ${#DETACHED_RUN_LOST_UNKNOWN_KEYS[@]})) 条不知道还在不在跑，不当结束，$every 秒后接着查"
    fi
    if ((DETACHED_RUN_LOST_RUNNING_COUNT > 0)); then
      echo "  · 登记不见了而单元还在跑的 $DETACHED_RUN_LOST_RUNNING_COUNT 条不当结束，$every 秒后接着查"
    fi
    if ((DETACHED_RUN_RUNNING_COUNT == 0 && DETACHED_RUN_UNKNOWN_COUNT == 0 && DETACHED_RUN_LOST_RUNNING_COUNT == 0 \
         && ${#DETACHED_RUN_LOST_UNKNOWN_KEYS[@]} == 0)); then
      detached_run_watch_outcomes
      exit 0
    fi
    if [[ "$how" == child ]]; then
      read -r -t "$every" _ <> <(:)   # 读一条只有自己握着写端的管道，到点超时
    else
      sleep "$every" &
      DETACHED_RUN_WATCH_SLEEP_PID=$!
      wait "$DETACHED_RUN_WATCH_SLEEP_PID"
      DETACHED_RUN_WATCH_SLEEP_PID=""
    fi
  done
}

detached_run_watch() { # watch 的参数
  local every="$DETACHED_RUN_DEFAULT_EVERY_SECONDS" child_exit watch_started
  watch_started="$(date +%s)"   # 这一秒起跑的登记都算观察到的（文件头 watch 那一段）
  while (($#)); do
    case "$1" in
      --every) (($# >= 2)) || detached_run_usage_error "--every 后面要秒数"; every="$2"; shift 2 ;;
      --every=*) every="${1#--every=}"; shift ;;
      *) detached_run_usage_error "watch 认不出 $1" ;;
    esac
  done
  [[ "$every" =~ ^[1-9][0-9]*$ ]] || detached_run_usage_error "--every 要正整数秒，给的是 $every"
  if detached_run_break_is watch-stops-in-trap; then
    trap 'detached_run_watch_stopped TERM 143' TERM
    trap 'detached_run_watch_stopped INT 130' INT
    trap 'detached_run_watch_stopped HUP 129' HUP
    echo "  · detached-run.sh watch：每 $every 秒打一次清单，只读、不停任何单元；还在跑的与查不到的一条都不剩时报各条退出码、退出"
    detached_run_watch_loop "$every" in-process "$watch_started"
  fi
  # 信号由这个进程接，一轮轮列清单的是它起的子进程：信号落在一轮清单中间时，在列清单的那个 bash 里跑 trap，负载高时偶发 bash 报
  #   「unexpected EOF while looking for matching `)'」、watch 退 1、最后一次清单打不出来（trap 里只记一个变量也一样）。这个进程只在 wait 里接信号：
  #   记下信号，停子进程（只停它这一个进程号）、等它退，再打最后一次清单、退出
  trap 'detached_run_watch_on_signal TERM 143' TERM
  trap 'detached_run_watch_on_signal INT 130' INT
  trap 'detached_run_watch_on_signal HUP 129' HUP
  echo "  · detached-run.sh watch：每 $every 秒打一次清单，只读、不停任何单元；还在跑的与查不到的一条都不剩时报各条退出码、退出"
  detached_run_watch_loop "$every" child "$watch_started" &
  DETACHED_RUN_WATCH_CHILD_PID=$!
  if wait "$DETACHED_RUN_WATCH_CHILD_PID"; then child_exit=0; else child_exit=$?; fi
  if [[ -z "$DETACHED_RUN_WATCH_SIGNAL_NAME" ]]; then exit "$child_exit"; fi
  kill "$DETACHED_RUN_WATCH_CHILD_PID" 2>/dev/null
  wait "$DETACHED_RUN_WATCH_CHILD_PID" 2>/dev/null
  detached_run_watch_stopped "$DETACHED_RUN_WATCH_SIGNAL_NAME" "$DETACHED_RUN_WATCH_SIGNAL_EXIT"
}

detached_run_stop() { # stop 的参数
  local label record unit state rc tries=0 stop_timeout
  (($# == 1)) || detached_run_usage_error "stop 只要一个名"
  label="$1"
  [[ "$label" =~ $DETACHED_RUN_LABEL_PATTERN ]] || detached_run_usage_error "名「$label」不合写法"
  record="$detached_run_state/$label"
  if [[ ! -f "$record/unit" ]]; then
    echo "  ✗ 登记目录 $detached_run_state 里没有「$label」，什么都没停" >&2
    echo "     → 怎么办：bash $detached_run_script list 看登记了哪些名；只停经 detached-run.sh start 起的、按登记的名停" >&2
    exit 3
  fi
  unit="$(detached_run_field "$record" unit)"
  stop_timeout="$(detached_run_field "$record" stop-timeout)"
  state="$(detached_run_unit_state "$unit")"
  if [[ "$state" == "$DETACHED_RUN_QUERY_FAILED" ]]; then
    detached_run_query_failed "$unit" "判不出它还在不在跑：什么都没停"
  fi
  if ! detached_run_is_active_state "$state"; then
    echo "  ✓ 「$label」的单元 $unit 已经不在跑（ActiveState=${state:-?}），没停什么；退出码 $(detached_run_field "$record" exit)"
    return 0
  fi
  detached_run_write_field "$record" stop-requested "$(detached_run_today)"
  echo "  · 停「$label」：单元 $unit，停机时限 ${stop_timeout:-?} 秒；等命令收尾完才回来，至多停机时限加 5 秒"
  # 弄坏开关 stop-nonblocking：不等停机任务，至多等 5 秒退出码与单元停下。else 里那几段保持原文、不缩进
  #   （第二轮攻方 v4_mutations.py 按原文改它们造同一种错实现，见文件头的开关表）
  if detached_run_break_is stop-nonblocking; then
    systemctl --user stop --no-block "$unit"
    while ((tries < 50)); do
      state="$(detached_run_unit_state "$unit")"
      if [[ -f "$record/exit" ]] && ! detached_run_is_active_state "$state"; then break; fi
      sleep 0.1
      tries=$((tries + 1))
    done
    state="$(detached_run_unit_state "$unit")"
    rc="$(detached_run_field "$record" exit)"
  else
  if ! systemctl --user stop "$unit"; then
    echo "  ✗ systemctl --user stop $unit 失败（它的报错在上面）" >&2
    echo "     → 怎么办：systemctl --user status $unit 看它卡在哪；别按名字或进程组去停，交主 agent 判断" >&2
    exit 1
  fi
  state="$(detached_run_unit_state "$unit")"
  if detached_run_is_active_state "$state"; then
    echo "  ✗ 停过之后单元 $unit 还是 $state" >&2
    echo "     → 怎么办：systemctl --user status $unit 看它卡在哪；别按名字或进程组去停，交主 agent 判断" >&2
    exit 1
  fi
  # systemctl --user stop 等到单元停下才回来（命令在停机时限里退出，或到点被 KILL）；主进程退出前写好退出码，这里再至多等 5 秒
  while [[ ! -f "$record/exit" ]] && ((tries < 50)); do
    sleep 0.1
    tries=$((tries + 1))
  done
  rc="$(detached_run_field "$record" exit)"
  fi
  if [[ -z "$rc" ]]; then
    if ! detached_run_break_is stop-kill-unrecorded; then
      detached_run_write_field "$record" stop-killed "$(detached_run_today) 停机时限 ${stop_timeout:-?} 秒里没退出，被 KILL，没留退出码"
    fi
    echo "  ✓ 停了「$label」：单元 $unit 现在是 ${state:-?}；停机时限 ${stop_timeout:-?} 秒里命令没退出，被 KILL，没留退出码（登记里记了 stop-killed）"
    return 0
  fi
  echo "  ✓ 停了「$label」：单元 $unit 现在是 ${state:-?}，退出码 $rc"
}

# ── 自证 ──
detached_run_selftest_checked=0
detached_run_selftest_failed=0
detached_run_selftest_units=()
detached_run_selftest_work=""
detached_run_selftest_r_archive=""   # Ⓡ 那一格旧一次的登记挪去的地方（弄坏开关 in-caller-tree 下收尾按它停那一个 sleep）

detached_run_expect() { # <格名> <0 对 / 别的 错> [<输出文件>]
  detached_run_selftest_checked=$((detached_run_selftest_checked + 1))
  if [[ "$2" == 0 ]]; then
    echo "  · 对：$1"
    return 0
  fi
  detached_run_selftest_failed=$((detached_run_selftest_failed + 1))
  echo "  ✗ 自证格不对：$1"
  echo "     → 怎么办：看这一格下面的输出与 detached-run.sh 里对应的那一段；DETACHED_RUN_BREAK 设着的话这一格本来就该红"
  if [[ -n "${3:-}" && -f "$3" ]]; then sed 's/^/       | /' -- "$3"; fi
  return 0
}

detached_run_selftest_wait_for_file() { # <文件> <至多几个 0.1 秒>
  local tries=0
  while [[ ! -f "$1" ]] && ((tries < $2)); do
    sleep 0.1
    tries=$((tries + 1))
  done
  [[ -f "$1" ]]
}

detached_run_selftest_wait_for_output() { # <文件> <要等的一行（grep -F 的字面）> <至多几个 0.1 秒> [<进程号>]：等到那一行出现退 0；
  #   给了进程号的，那个进程先退了就不再等、退 1（这一格判下去）。上限只防卡死，给足：负载高时 watch 一轮清单要好几秒
  local tries=0
  until grep -qF -- "$2" "$1" 2>/dev/null; do
    if [[ -n "${4:-}" ]] && ! kill -0 "$4" 2>/dev/null; then grep -qF -- "$2" "$1" 2>/dev/null; return; fi
    ((tries < $3)) || return 1
    sleep 0.1
    tries=$((tries + 1))
  done
}

detached_run_selftest_wait_for_round_after() { # <文件> <那一行（字面）> <至多几个 0.1 秒> <进程号>：等那一行之后 watch 又打了一次「== 第 N 次」（没当结束、接着看），
  #   或那个进程退了；到上限退 1
  local tries=0
  until awk -v line="$2" 'index($0, line) { seen = 1; next } seen && /^  == 第 [0-9]+ 次/ { found = 1 } END { exit found ? 0 : 1 }' "$1" 2>/dev/null; do
    kill -0 "$4" 2>/dev/null || return 1
    ((tries < $3)) || return 1
    sleep 0.1
    tries=$((tries + 1))
  done
}

detached_run_selftest_stop_leftover() { # <登记目录>：弄坏开关 in-caller-tree 下命令不在单元里，按登记的进程号停这一个（它是这份自证起的 sleep；t 那一格的 sleep 不理 TERM，所以发 KILL）
  local pid
  pid="$(detached_run_field "$1" pid)"
  [[ "$pid" =~ ^[0-9]+$ ]] || return 0
  [[ -f "$1/exit" ]] && return 0
  if [[ "$(cat "/proc/$pid/comm" 2>/dev/null)" == sleep ]]; then kill -KILL "$pid" 2>/dev/null; fi
  return 0
}

detached_run_selftest_cleanup() { # 停这份自证起的每个单元（按单元名，一个一个停），删临时目录
  local unit work="$detached_run_selftest_work" prefix="selftest-$$-"
  for unit in ${detached_run_selftest_units[@]+"${detached_run_selftest_units[@]}"}; do
    if detached_run_is_active_state "$(detached_run_unit_state "$unit")"; then systemctl --user stop "$unit" 2>/dev/null; fi
    systemctl --user reset-failed "$unit" 2>/dev/null
  done
  if [[ -n "$work" ]]; then
    detached_run_selftest_stop_leftover "$work/state/${prefix}c"
    detached_run_selftest_stop_leftover "$work/state/${prefix}a"
    detached_run_selftest_stop_leftover "$work/state/${prefix}e"
    detached_run_selftest_stop_leftover "$work/state/${prefix}e1"
    detached_run_selftest_stop_leftover "$work/state/${prefix}e2"
    detached_run_selftest_stop_leftover "$work/state/${prefix}u"
    detached_run_selftest_stop_leftover "$work/state/${prefix}i"
    detached_run_selftest_stop_leftover "$work/state-r/${prefix}r"
    if [[ -n "$detached_run_selftest_r_archive" ]]; then detached_run_selftest_stop_leftover "$detached_run_selftest_r_archive"; fi
    detached_run_selftest_stop_leftover "$work/state-t/${prefix}t"
    detached_run_selftest_stop_leftover "$work/moved-x"
    detached_run_selftest_stop_leftover "$work/state/${prefix}n"
    detached_run_selftest_stop_leftover "$work/moved-w"
    detached_run_selftest_stop_leftover "$work/state-w2/${prefix}w2"
    # Ⓥ 那一格让命令把自己的登记目录改成不可写：删临时目录之前改回来
    chmod -R u+w -- "$work" 2>/dev/null
  fi
  return 0
}

detached_run_selftest_timespan_seconds() { # <systemctl show 打的时长，如 3min、1min 30s、2s>：打出秒数；认不出打空
  local part total=0
  for part in $1; do
    [[ "$part" =~ ^([0-9]+)(h|min|s)$ ]] || return 0
    case "${BASH_REMATCH[2]}" in
      h) total=$((total + BASH_REMATCH[1] * 3600)) ;;
      min) total=$((total + BASH_REMATCH[1] * 60)) ;;
      s) total=$((total + BASH_REMATCH[1])) ;;
    esac
  done
  printf '%s\n' "$total"
}

detached_run_selftest_state_directory() { # <目录>：建一个自证用的登记目录，放进 .selftest-token（弄坏开关在它里面才认）
  mkdir -p -- "$1" && printf '%s\n' "$DETACHED_RUN_SELFTEST_TOKEN" > "$1/.selftest-token"
}

detached_run_selftest_shim() { # <目录> <命令名> <脚本正文>：在目录里放一个替身命令（只放进那一格命令的 PATH 最前面）
  mkdir -p -- "$1" && printf '#!/usr/bin/env bash\n%s\n' "$3" > "$1/$2" && chmod +x -- "$1/$2"
}

detached_run_selftest_watch_signal() { # <信号> <期望的退出码> <格的字母> <格名>：起一条 sleep 300 的单元，watch 收到这个信号之后
  #   退期望的码、最后一次清单里有它、它的单元与 sleep 都还在。work、prefix、state_directory 用 detached_run_selftest 里的
  local signal="$1" wanted_rc="$2" letter="$3" label="${prefix}$3" unit pid watch_pid rc tries=0 state
  unit="${DETACHED_RUN_UNIT_PREFIX}${label}.service"
  bash "$detached_run_script" start --wall-clock within-2h --label "$label" -- sleep 300 > "$work/$letter.out" 2>&1
  detached_run_selftest_wait_for_file "$state_directory/$label/pid" 100
  pid="$(detached_run_field "$state_directory/$label" pid)"
  # 后台作业的 INT 默认被忽略（nohup 下 HUP 也是），忽略着进来的信号 trap 不上：用 env --default-signal 把三种都放回默认，watch 才接得到
  env --default-signal=INT,HUP,TERM bash "$detached_run_script" watch --every 1 > "$work/$letter-watch.out" 2>&1 &
  watch_pid=$!
  # 等 watch 列出这一条再发信号：上限给足（负载高时 watch 起来、列到这一条要十几秒），watch 先退了就不等
  detached_run_selftest_wait_for_output "$work/$letter-watch.out" "还在跑  $label" 1200 "$watch_pid"
  kill -"$signal" "$watch_pid"
  wait "$watch_pid"; rc=$?
  sleep 0.5
  state="$(detached_run_unit_state "$unit")"
  echo "watch 退出码 $rc；$label 的单元 $state；命令进程 ${pid:-（没读到）}" >> "$work/$letter-watch.out"
  detached_run_expect "$4" \
    "$( { ((rc == wanted_rc)) && grep -q "watch 收到 $signal" "$work/$letter-watch.out" \
          && sed -n "/watch 收到 $signal/,\$p" "$work/$letter-watch.out" > "$work/$letter-last.out" && grep -q "还在跑  $label" "$work/$letter-last.out" \
          && detached_run_is_active_state "$state" && [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; } && echo 0 || echo 1)" "$work/$letter-watch.out"
}

detached_run_selftest_watch_signal_mid_list() { # <格名>：watch 在一轮清单列到一半时收到 TERM。替身 systemctl 每被 show -p 调一次，记下自己往上的一串祖先进程号；
  #   第一次 show e2 的单元时再记一行 MARK、建标记、停 2 秒，自证见到标记就给 watch 发 TERM（信号一定落在这一轮清单中间）。
  #   要：MARK 之前每一次调替身，祖先里 watch 进程底下那一个都是同一个进程（一轮清单在 watch 起的一个子进程里列，接信号的 watch 进程不列清单），
  #   退 143，最后一次清单里有 e2，e2 的单元与 sleep 都还在。work、prefix、state_directory 用 detached_run_selftest 里的
  local label="${prefix}e2" unit pid watch_pid rc state shim="$work/shim-mid-list" marker="$work/e2-listing" real_systemctl lister_check
  unit="${DETACHED_RUN_UNIT_PREFIX}${label}.service"
  real_systemctl="$(command -v systemctl)"
  detached_run_selftest_shim "$shim" systemctl "chain=\"\" ancestor=\"\$PPID\" depth=0
while [[ \"\$ancestor\" =~ ^[0-9]+\$ ]] && (( ancestor > 1 && depth < 16 )); do
  chain+=\"\$ancestor \"
  ancestor=\"\$(sed -n 's/^.*) [A-Za-z] \\([0-9]*\\) .*/\\1/p' \"/proc/\$ancestor/stat\" 2>/dev/null)\"
  depth=\$((depth + 1))
done
if [[ \" \$* \" == *' show -p '* ]]; then printf '%s\n' \"\$chain\" >> $(printf '%q' "$work/e2-listers"); fi
if [[ \" \$* \" == *' show '* && \" \$* \" == *$(printf '%q' " $unit ")* && ! -e $(printf '%q' "$marker") ]]; then
  printf 'MARK\n' >> $(printf '%q' "$work/e2-listers")
  : > $(printf '%q' "$marker")
  sleep 2
fi
exec $(printf '%q' "$real_systemctl") \"\$@\""
  # 先起一条 e1（名字排在 e2 前面）：别的格的单元被弄坏开关停了也保证这一轮在 e2 之前至少调两次 show
  bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}e1" -- sleep 300 > "$work/e1.out" 2>&1
  bash "$detached_run_script" start --wall-clock within-2h --label "$label" -- sleep 300 > "$work/e2.out" 2>&1
  detached_run_selftest_wait_for_file "$state_directory/$label/pid" 100
  pid="$(detached_run_field "$state_directory/$label" pid)"
  PATH="$shim:$PATH" env --default-signal=INT,HUP,TERM bash "$detached_run_script" watch --every 1 > "$work/e2-watch.out" 2>&1 &
  watch_pid=$!
  detached_run_selftest_wait_for_file "$marker" 1200
  kill -TERM "$watch_pid"
  wait "$watch_pid"; rc=$?
  sleep 0.5
  state="$(detached_run_unit_state "$unit")"
  # MARK 之前的每一行：祖先里 watch 进程底下那一个
  lister_check="$(awk -v watch="$watch_pid" '
    $0 == "MARK" { exit }
    { n = split($0, chain, " "); below = ""
      for (i = 2; i <= n; i++) if (chain[i] == watch) { below = chain[i - 1]; break }
      calls++
      if (below == "") missing++; else if (first == "") first = below; else if (below != first) differ++ }
    END { if (calls >= 2 && !missing && !differ) print "ok"; else printf "MARK 之前调替身 %d 次，祖先里没有 watch 进程的 %d 次，watch 底下那一个与第一次不同的 %d 次\n", calls, missing, differ }' "$work/e2-listers" 2>/dev/null)"
  echo "watch 退出码 $rc；$label 的单元 $state；命令进程 ${pid:-（没读到）}；watch 进程 $watch_pid；列清单的进程：${lister_check:-（替身没被调到）}" >> "$work/e2-watch.out"
  detached_run_expect "$1" \
    "$( { ((rc == 143)) && [[ -e "$marker" && "$lister_check" == ok ]] \
          && sed -n "/watch 收到 TERM/,\$p" "$work/e2-watch.out" > "$work/e2-last.out" && grep -q "还在跑  $label" "$work/e2-last.out" \
          && detached_run_is_active_state "$state" && [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; } && echo 0 || echo 1)" "$work/e2-watch.out"
}

detached_run_selftest() {
  local work prefix="selftest-$$-" rc pid pid_before state caller_unit wanted got tries leftover
  work="$(mktemp -d "${TMPDIR:-/tmp}/detached-run-selftest.XXXXXX")" || {
    echo "  ✗ 建不了自证的临时目录"
    echo "     → 怎么办：看 \$TMPDIR（没设是 /tmp）可不可写"
    exit 1
  }
  detached_run_selftest_work="$work"
  trap 'detached_run_selftest_cleanup; rm -rf -- "${detached_run_selftest_work:?}"' EXIT
  export DETACHED_RUN_STATE_DIRECTORY="$work/state"
  # 弄坏开关只在这份自证起的进程里认：临时登记目录里的 .selftest-token 与带下去的 DETACHED_RUN_SELFTEST_TOKEN 对得上
  DETACHED_RUN_SELFTEST_TOKEN="selftest-$$-$(cat /proc/sys/kernel/random/uuid 2>/dev/null || date +%s%N)"
  export DETACHED_RUN_SELFTEST_TOKEN
  local state_directory="$work/state"
  detached_run_selftest_state_directory "$state_directory"
  local unit_b="${DETACHED_RUN_UNIT_PREFIX}${prefix}b.service" unit_d="${DETACHED_RUN_UNIT_PREFIX}${prefix}d.service"
  local unit_f="${DETACHED_RUN_UNIT_PREFIX}${prefix}f.service" unit_c="${DETACHED_RUN_UNIT_PREFIX}${prefix}c.service"
  local unit_a="${DETACHED_RUN_UNIT_PREFIX}${prefix}a.service" unit_e="${DETACHED_RUN_UNIT_PREFIX}${prefix}e.service"
  local unit_t="${DETACHED_RUN_UNIT_PREFIX}${prefix}t.service" unit_x="${DETACHED_RUN_UNIT_PREFIX}${prefix}x.service" letter
  caller_unit="${DETACHED_RUN_UNIT_PREFIX}${prefix}caller.service"
  detached_run_selftest_units=("$unit_b" "$unit_d" "$unit_f" "$unit_c" "$unit_a" "$unit_e" "$caller_unit" "$unit_t" "$unit_x")
  for letter in e1 e2 u i k l l2 q2 r s n w w2 v1 v2 v3 p q m0 m1 m2 m3 m4 m5 m6 y z1 z2 us lw lw2 lr1 lr2 lr3 lr4 lr-ok lr-within; do detached_run_selftest_units+=("${DETACHED_RUN_UNIT_PREFIX}${prefix}${letter}.service"); done
  detached_run_selftest_units+=("${DETACHED_RUN_UNIT_PREFIX}${prefix}us-inner.scope")   # Ⓤ 那一格照内存包装起的 scope
  # 单元里写登记失败时的旁路文件落进临时目录，不进真的 $XDG_RUNTIME_DIR
  export DETACHED_RUN_SIDE_DIRECTORY="$work/side"

  # Ⓗ 用法错与拒绝：名不合写法、没有 --、stop 没登记的名
  bash "$detached_run_script" start --wall-clock within-2h --label 'a/b' -- true > "$work/h1.out" 2>&1; rc=$?
  detached_run_expect "Ⓗ 名里带斜杠 ⇒ 用法错退 2" "$((rc == 2 ? 0 : 1))" "$work/h1.out"
  bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}h" true > "$work/h2.out" 2>&1; rc=$?
  detached_run_expect "Ⓗ 命令前面没写 -- ⇒ 用法错退 2，什么都没起" \
    "$( { ((rc == 2)) && [[ ! -d "$state_directory/${prefix}h" ]]; } && echo 0 || echo 1)" "$work/h2.out"
  bash "$detached_run_script" stop "${prefix}nowhere" > "$work/h3.out" 2>&1; rc=$?
  detached_run_expect "Ⓗ stop 没登记的名 ⇒ 拒绝退 3" "$((rc == 3 ? 0 : 1))" "$work/h3.out"

  # Ⓚ 弄坏开关设在自证之外（不带 DETACHED_RUN_SELFTEST_TOKEN，登记目录里的令牌对不上）：start 与 list 都拒、退 4，什么都没起
  env -u DETACHED_RUN_SELFTEST_TOKEN DETACHED_RUN_BREAK="${DETACHED_RUN_BREAK:+$DETACHED_RUN_BREAK,}exit-dropped" \
    bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}k" -- true > "$work/k.out" 2>&1; rc=$?
  env -u DETACHED_RUN_SELFTEST_TOKEN DETACHED_RUN_BREAK="${DETACHED_RUN_BREAK:+$DETACHED_RUN_BREAK,}exit-dropped" \
    bash "$detached_run_script" list >> "$work/k.out" 2>&1; got=$?
  detached_run_expect "Ⓚ DETACHED_RUN_BREAK 设在 --selftest 之外：start 与 list 都拒、退 4，没建登记" \
    "$( { ((rc == 4)) && [[ "$got" == 4 ]] && [[ ! -d "$state_directory/${prefix}k" ]] && grep -q "设在 --selftest 之外" "$work/k.out"; } && echo 0 || echo 1)" \
    "$work/k.out"

  # Ⓜ -- 之后开头那一串带 = 的词里有名字不合 shell 变量名写法的：start 拒、退 2、没建登记、命令没跑（单元里 env 会把它当赋值、照跑后面那条，闸却看不到）；
  #   preflight 先摘整词 --force（「--force a-b=1」判的是 a-b=1）。合法的 名=值 照起、设进命令的环境（对照）
  local m_index=0 m_sample m_words=() m_failed=0
  : > "$work/m.out"
  for m_sample in 'a-b=1' '--force=1' '1=1' 'a.b=1' 'FOO=1 a-b=1' '--force a-b=1'; do
    m_index=$((m_index + 1))
    read -r -a m_words <<< "$m_sample"
    bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}m$m_index" -- "${m_words[@]}" touch "$work/m$m_index.marker" >> "$work/m.out" 2>&1; rc=$?
    echo "样本「$m_sample」：start 退 $rc；登记 $([[ -d "$state_directory/${prefix}m$m_index" ]] && echo 在 || echo 不在)；marker $([[ -e "$work/m$m_index.marker" ]] && echo 在 || echo 不在)" >> "$work/m.out"
    { ((rc == 2)) && [[ ! -d "$state_directory/${prefix}m$m_index" && ! -e "$work/m$m_index.marker" ]]; } || m_failed=$((m_failed + 1))
  done
  bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}m0" -- FOO=selftest-m0 bash -c 'printf "%s\n" "$FOO" > "$0"' "$work/m0.env" >> "$work/m.out" 2>&1; rc=$?
  detached_run_selftest_wait_for_file "$state_directory/${prefix}m0/exit" 100
  echo "对照「FOO=selftest-m0」：start 退 $rc；命令读到 FOO=$(cat "$work/m0.env" 2>/dev/null)；拒的样本里带出路的 $(grep -c '要设环境变量写成 -- env 名=值' "$work/m.out") 条" >> "$work/m.out"
  detached_run_expect "Ⓜ -- 之后开头的赋值名不合写法（a-b=1、--force=1、1=1、a.b=1、FOO=1 a-b=1、--force a-b=1）：start 拒、退 2、带出路、没建登记、命令没跑；合法的 FOO=… 照起、设进命令" \
    "$( { ((m_failed == 0)) && ((rc == 0)) && [[ "$(cat "$work/m0.env" 2>/dev/null)" == selftest-m0 ]] \
          && [[ "$(grep -c '要设环境变量写成 -- env 名=值' "$work/m.out")" == "$m_index" ]]; } && echo 0 || echo 1)" "$work/m.out"

  # Ⓞ 同名的旧登记已结束、挪走之后建不了这一次的登记目录（替身 mkdir 只对这一次的登记目录退 1）：start 退 1、旧登记挪回原处、没留 <名>@<秒数>；
  #   挪回也失败（替身 mv 对从 <名>@… 挪出的退 1）：start 退 1、stderr 打一行「没挪回」带出路，旧登记留在 <名>@<秒数>
  local state_o="$work/state-o" shim_o="$work/shim-o" shim_o2="$work/shim-o2" real_mkdir real_mv shim_mkdir_body started_o
  real_mkdir="$(command -v mkdir)"
  real_mv="$(command -v mv)"
  detached_run_selftest_state_directory "$state_o"
  for letter in o o2; do
    mkdir -p "$state_o/${prefix}$letter"
    detached_run_write_field "$state_o/${prefix}$letter" unit "${DETACHED_RUN_UNIT_PREFIX}${prefix}$letter.service"
    detached_run_write_field "$state_o/${prefix}$letter" started "$(($(date +%s) - 100))"
    detached_run_write_field "$state_o/${prefix}$letter" exit 0
  done
  started_o="$(detached_run_field "$state_o/${prefix}o" started)"
  shim_mkdir_body="for argument in \"\$@\"; do
  case \"\$argument\" in $(printf '%q' "$state_o/${prefix}o")|$(printf '%q' "$state_o/${prefix}o2")) echo 'mkdir: 建不了（自证的替身）' >&2; exit 1 ;; esac
done
exec $(printf '%q' "$real_mkdir") \"\$@\""
  detached_run_selftest_shim "$shim_o" mkdir "$shim_mkdir_body"
  detached_run_selftest_shim "$shim_o2" mkdir "$shim_mkdir_body"
  detached_run_selftest_shim "$shim_o2" mv "seen=0
for argument in \"\$@\"; do
  if ((seen)); then
    case \"\$argument\" in $(printf '%q' "$state_o/${prefix}o2@")*) echo 'mv: 挪不动（自证的替身）' >&2; exit 1 ;; esac
    break
  fi
  [[ \"\$argument\" == -- ]] && seen=1
done
exec $(printf '%q' "$real_mv") \"\$@\""
  PATH="$shim_o:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_o" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}o" -- true > "$work/o1.out" 2>&1; rc=$?
  PATH="$shim_o2:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_o" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}o2" -- true > "$work/o2.out" 2>&1; got=$?
  { echo "建不了登记、挪回成：start 退 $rc"; cat "$work/o1.out"; echo "建不了登记、挪回也失败：start 退 $got"; cat "$work/o2.out"; ls -a "$state_o"; } > "$work/o.out" 2>&1
  detached_run_expect "Ⓞ 建不了这一次的登记：start 退 1、挪走的同名旧登记挪回原处；挪回也失败时退 1、打一行「没挪回」带出路、旧登记留在 <名>@<秒数>" \
    "$( { ((rc == 1)) && [[ "$(detached_run_field "$state_o/${prefix}o" started)" == "$started_o" ]] && ! compgen -G "$state_o/${prefix}o@*" > /dev/null \
          && grep -qF "挪回了 $state_o/${prefix}o" "$work/o1.out" && grep -q "建不了登记" "$work/o1.out" \
          && [[ "$got" == 1 && ! -e "$state_o/${prefix}o2" ]] && compgen -G "$state_o/${prefix}o2@*" > /dev/null \
          && grep -qF "没挪回 $state_o/${prefix}o2" "$work/o2.out" && grep -qF "→ 怎么办：看登记目录 $state_o 的权限" "$work/o2.out"; } && echo 0 || echo 1)" "$work/o.out"

  # Ⓑ 退出码在单元结束之后读得到：命令退 7，单元收掉（--collect）之后登记里与 list 里都是 7，list 带日志最后一行
  bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}b" -- bash -c 'echo last-line-b; exit 7' > "$work/b.out" 2>&1; rc=$?
  detached_run_selftest_wait_for_file "$state_directory/${prefix}b/exit" 200
  tries=0
  while [[ "$(systemctl --user show -p LoadState --value "$unit_b" 2>/dev/null)" == loaded ]] && ((tries < 50)); do
    sleep 0.1
    tries=$((tries + 1))
  done
  bash "$detached_run_script" list > "$work/b-list.out" 2>&1
  cat "$work/b.out" >> "$work/b-list.out"
  got="$(detached_run_field "$state_directory/${prefix}b" exit)"
  detached_run_expect "Ⓑ 退出码在单元结束之后读得到（start 退 0、登记里 exit 是 7、单元已收掉、list 报退出码 7 与最后一行）" \
    "$( { ((rc == 0)) && [[ "$got" == 7 ]] \
          && [[ "$(systemctl --user show -p LoadState --value "$unit_b" 2>/dev/null)" != loaded ]] \
          && grep -q "已结束  ${prefix}b  退出码 7" "$work/b-list.out" \
          && grep -q "日志最后一行：last-line-b" "$work/b-list.out"; } && echo 0 || echo 1)" "$work/b-list.out"

  # Ⓓ 调用方的环境与工作目录带进单元，命令里的 $ 与 % 原样交给命令
  #   PATH 的第一段换成一个空目录、HOME 换成临时目录，单元里读到的要是这两样，不是用户级 systemd 自己的那一份
  mkdir -p "$work/cwd-d" "$work/path-d" "$work/home-d"
  ( cd "$work/cwd-d" && SINGLEFS_HEAVY_TESTS=selftest-carried CARGO_HOME="$work/cargo-home" RUSTUP_HOME="$work/rustup-home" \
      PATH="$work/path-d:$PATH" HOME="$work/home-d" \
      bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}d" -- \
      bash -c 'printf "%s|%s|%s|%s|%s|%s|%s\n" "${SINGLEFS_HEAVY_TESTS-unset}" "${CARGO_HOME-unset}" "${RUSTUP_HOME-unset}" "${PATH%%:*}" "${HOME-unset}" "$(pwd -P)" "$1" > "$0"' \
      "$work/d.env" 'pct%n-dollar$HOME' ) > "$work/d.out" 2>&1; rc=$?
  detached_run_selftest_wait_for_file "$state_directory/${prefix}d/exit" 200
  wanted="selftest-carried|$work/cargo-home|$work/rustup-home|$work/path-d|$work/home-d|$(cd "$work/cwd-d" && pwd -P)|pct%n-dollar\$HOME"
  got="$(cat "$work/d.env" 2>/dev/null)"
  printf '要的：%s\n读到：%s\n' "$wanted" "$got" >> "$work/d.out"
  detached_run_expect "Ⓓ SINGLEFS_HEAVY_TESTS、CARGO_HOME、RUSTUP_HOME、PATH、HOME 与工作目录带进单元，\$ 与 % 原样" \
    "$( { ((rc == 0)) && [[ "$got" == "$wanted" ]]; } && echo 0 || echo 1)" "$work/d.out"

  # Ⓕ watch 在观察的全部结束时自己退 0，报各自的退出码
  bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}f" -- bash -c 'sleep 2; exit 5' > "$work/f.out" 2>&1
  timeout 60 bash "$detached_run_script" watch --every 1 > "$work/f-watch.out" 2>&1; rc=$?
  cat "$work/f.out" >> "$work/f-watch.out"
  detached_run_expect "Ⓕ watch 在观察的全部结束时自己退 0、报 ${prefix}f 退出码 5" \
    "$( { ((rc == 0)) && grep -q "^  · ${prefix}f 退出码 5\$" "$work/f-watch.out"; } && echo 0 || echo 1)" "$work/f-watch.out"

  # Ⓒ 同名还在跑就拒、退 3，原登记与命令不动；list 数到还在跑 1 条
  bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}c" -- sleep 300 > "$work/c.out" 2>&1
  detached_run_selftest_wait_for_file "$state_directory/${prefix}c/pid" 100
  pid_before="$(detached_run_field "$state_directory/${prefix}c" pid)"
  bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}c" -- sleep 301 > "$work/c2.out" 2>&1; rc=$?
  pid="$(detached_run_field "$state_directory/${prefix}c" pid)"
  bash "$detached_run_script" list > "$work/c-list.out" 2>&1
  cat "$work/c.out" "$work/c2.out" >> "$work/c-list.out"
  detached_run_expect "Ⓒ 同名还在跑就拒、退 3，原登记的命令进程号不变、还活着，list 报还在跑 1 条" \
    "$( { ((rc == 3)) && [[ -n "$pid_before" && "$pid" == "$pid_before" ]] && kill -0 "$pid_before" 2>/dev/null \
          && grep -q "还在跑、没结束的 1 条" "$work/c-list.out"; } && echo 0 || echo 1)" "$work/c-list.out"

  # Ⓒ 没写 --stop-timeout：默认停机时限（DETACHED_RUN_DEFAULT_STOP_TIMEOUT_SECONDS）设进单元，与登记里的 stop-timeout 一致（不是用户级 systemd 的默认 90 秒）
  local default_timeout_property
  default_timeout_property="$(systemctl --user show -p TimeoutStopUSec --value "$unit_c" 2>&1)"
  got="$(detached_run_selftest_timespan_seconds "$default_timeout_property")"
  echo "单元 $unit_c 的 TimeoutStopUSec：$default_timeout_property（$got 秒）；登记 stop-timeout：$(detached_run_field "$state_directory/${prefix}c" stop-timeout)" > "$work/c-timeout.out"
  detached_run_expect "Ⓒ 没写 --stop-timeout：单元的 TimeoutStopUSec 是默认的 $DETACHED_RUN_DEFAULT_STOP_TIMEOUT_SECONDS 秒，登记里同一个数" \
    "$( { [[ "$got" == "$DETACHED_RUN_DEFAULT_STOP_TIMEOUT_SECONDS" ]] \
          && [[ "$(detached_run_field "$state_directory/${prefix}c" stop-timeout)" == "$DETACHED_RUN_DEFAULT_STOP_TIMEOUT_SECONDS" ]]; } && echo 0 || echo 1)" \
    "$work/c-timeout.out"

  # Ⓖ stop 只按登记的单元名停那一个：单元停下、登记里退出码 143、注 stop 停的
  bash "$detached_run_script" stop "${prefix}c" > "$work/g.out" 2>&1; rc=$?
  state="$(detached_run_unit_state "$unit_c")"
  got="$(detached_run_field "$state_directory/${prefix}c" exit)"
  detached_run_expect "Ⓖ stop 停下登记的那一个单元：退 0、单元不在跑、退出码 143、记了 stop-requested、命令进程没了" \
    "$( { ((rc == 0)) && ! detached_run_is_active_state "$state" && [[ "$got" == 143 ]] \
          && [[ -f "$state_directory/${prefix}c/stop-requested" ]] && ! kill -0 "$pid_before" 2>/dev/null; } && echo 0 || echo 1)" "$work/g.out"

  # Ⓝ stop 等命令收尾完：命令收到 TERM 之后 7 秒才退 9（比 stop 回读退出码的 5 秒长、比停机时限 30 秒短）；stop 退 0、报退出码 9、
  #   回来时单元已停下、没记 stop-killed
  bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}n" --stop-timeout 30 -- bash -c 'trap "sleep 7; exit 9" TERM; : > "$0"; sleep 30 & wait' "$work/n.ready" \
    > "$work/n.out" 2>&1
  detached_run_selftest_wait_for_file "$work/n.ready" 100
  bash "$detached_run_script" stop "${prefix}n" >> "$work/n.out" 2>&1; rc=$?
  state="$(detached_run_unit_state "${DETACHED_RUN_UNIT_PREFIX}${prefix}n.service")"
  echo "stop 退 $rc；stop 回来时单元 $state；登记 exit=$(detached_run_field "$state_directory/${prefix}n" exit)；stop-killed=$(detached_run_field "$state_directory/${prefix}n" stop-killed)" >> "$work/n.out"
  detached_run_expect "Ⓝ stop 等命令收尾完（TERM 之后 7 秒退 9）：退 0、报退出码 9、回来时单元已停下、没记 stop-killed" \
    "$( { ((rc == 0)) && grep -q "退出码 9" "$work/n.out" && ! detached_run_is_active_state "$state" \
          && [[ "$(detached_run_field "$state_directory/${prefix}n" exit)" == 9 ]] && [[ ! -f "$state_directory/${prefix}n/stop-killed" ]]; } && echo 0 || echo 1)" \
    "$work/n.out"

  # Ⓐ 调用方被停、命令照跑：调用方本身放在一个单元里（停它就是停它整个 cgroup，比停一棵进程树还狠），由它起 sleep 300，再停调用方
  systemd-run --user --quiet --collect --service-type=exec --expand-environment=no --unit="$caller_unit" --slice="$DETACHED_RUN_SLICE" \
    --working-directory="$work" --setenv=PATH="$PATH" --setenv=HOME="$HOME" --setenv=DETACHED_RUN_STATE_DIRECTORY="$state_directory" \
    --setenv=DETACHED_RUN_SELFTEST_TOKEN="$DETACHED_RUN_SELFTEST_TOKEN" ${DETACHED_RUN_BREAK:+"--setenv=DETACHED_RUN_BREAK=$DETACHED_RUN_BREAK"} \
    --setenv=DETACHED_RUN_SIDE_DIRECTORY="$DETACHED_RUN_SIDE_DIRECTORY" \
    -- "$BASH" -c '"$BASH" "$0" start --wall-clock within-2h --label "$1" -- sleep 300 > "$2" 2>&1; exec sleep 300' "$detached_run_script" "${prefix}a" "$work/a.out" \
    > "$work/a-caller.out" 2>&1
  detached_run_selftest_wait_for_file "$state_directory/${prefix}a/pid" 150
  pid="$(detached_run_field "$state_directory/${prefix}a" pid)"
  systemctl --user stop "$caller_unit" >> "$work/a-caller.out" 2>&1
  sleep 0.5
  state="$(detached_run_unit_state "$unit_a")"
  {
    echo "调用方单元 $caller_unit 停过之后：$(detached_run_unit_state "$caller_unit")；命令单元 $unit_a：$state；命令进程 ${pid:-（没读到）}"
    cat "$work/a.out" 2>/dev/null
  } >> "$work/a-caller.out"
  detached_run_expect "Ⓐ 调用方的单元被停之后，命令的单元还在跑、sleep 还活着" \
    "$( { [[ -n "$pid" ]] && ! detached_run_is_active_state "$(detached_run_unit_state "$caller_unit")" \
          && detached_run_is_active_state "$state" && kill -0 "$pid" 2>/dev/null \
          && [[ "$(cat "/proc/$pid/comm" 2>/dev/null)" == sleep ]]; } && echo 0 || echo 1)" "$work/a-caller.out"

  # Ⓔ watch 被 TERM / INT / HUP 停：先打最后一次清单（里面有还在跑的那一条），退 143 / 130 / 129；那一条的单元与 sleep 都还在
  detached_run_selftest_watch_signal TERM 143 e "Ⓔ watch 被 TERM 停：退 143、最后一次清单里有还在跑的 ${prefix}e，e 的单元与 sleep 都还在"
  detached_run_selftest_watch_signal INT 130 i "Ⓔ watch 被 INT 停：退 130、最后一次清单里有还在跑的 ${prefix}i，i 的单元与 sleep 都还在"
  detached_run_selftest_watch_signal HUP 129 u "Ⓔ watch 被 HUP 停：退 129、最后一次清单里有还在跑的 ${prefix}u，u 的单元与 sleep 都还在"
  detached_run_selftest_watch_signal_mid_list "Ⓔ watch 在一轮清单列到一半时收到 TERM：一轮清单在 watch 起的一个子进程里列、不在接信号的 watch 进程里，退 143、最后一次清单里有还在跑的 ${prefix}e2，e2 的单元与 sleep 都还在"

  # Ⓠ 查不到单元状态（替身 systemctl：show 与 list-units 退 1、什么都不打，show-environment 转给真的，preflight 要它）：
  #   list 单列「查不到」、不算没留退出码；watch 不当结束、接着看（5 秒后被 timeout 停）；start 与 stop 拒、退 1、什么都没做
  local real_systemctl shim_bus="$work/shim-bus" state_q="$work/state-q"
  real_systemctl="$(command -v systemctl)"
  detached_run_selftest_shim "$shim_bus" systemctl \
    "for argument in \"\$@\"; do [[ \"\$argument\" == show-environment ]] && exec $(printf '%q' "$real_systemctl") \"\$@\"; done
echo 'Failed to connect to bus: No such file or directory（自证的替身）' >&2
exit 1"
  detached_run_selftest_state_directory "$state_q"
  mkdir -p "$state_q/${prefix}q"
  detached_run_write_field "$state_q/${prefix}q" unit "${DETACHED_RUN_UNIT_PREFIX}${prefix}q.service"
  detached_run_write_field "$state_q/${prefix}q" started "$(date +%s)"
  detached_run_write_field "$state_q/${prefix}q" command "sleep 300"
  PATH="$shim_bus:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_q" bash "$detached_run_script" list > "$work/q-list.out" 2>&1
  # timeout 给足（负载高时三轮清单可能超过 6 秒）：报警那一行打出来、watch 又列完一轮仍不退出，就给 timeout 发 ALRM 让它当场到点（与到点同一个结局：
  #   停 watch、退 124）；watch 先退了（当成结束）就不等
  local watch_pid_q
  PATH="$shim_bus:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_q" timeout 600 bash "$detached_run_script" watch --every 1 > "$work/q-watch.out" 2>&1 &
  watch_pid_q=$!
  detached_run_selftest_wait_for_output "$work/q-watch.out" "报警  ${prefix}q" 600 "$watch_pid_q" \
    && detached_run_selftest_wait_for_round_after "$work/q-watch.out" "报警  ${prefix}q" 600 "$watch_pid_q"
  kill -ALRM "$watch_pid_q" 2>/dev/null
  wait "$watch_pid_q"; rc=$?
  { echo "watch 退出码 $rc"; cat "$work/q-watch.out"; } >> "$work/q-list.out"
  detached_run_expect "Ⓠ 查不到单元状态：list 单列「查不到」，watch 不当结束（到 6 秒被 timeout 停、退 124，没报「都结束了」）" \
    "$( { grep -q "查不到  ${prefix}q" "$work/q-list.out" && grep -q "查不到 1 条" "$work/q-list.out" \
          && ((rc == 124)) && ! grep -q "都结束了" "$work/q-watch.out"; } && echo 0 || echo 1)" "$work/q-list.out"
  # 同一次 watch：每秒一次、连续 3 次查不到之后报警一行（带出路），仍不退出
  detached_run_expect "Ⓠ 连续 $DETACHED_RUN_UNKNOWN_ALARM_ROUNDS 次查不到：watch 报警一行「报警  ${prefix}q」、带出路，仍不退出（退 124）" \
    "$( { grep -q "报警  ${prefix}q .*连续 [0-9]* 次查不到" "$work/q-watch.out" && grep -q "→ 怎么办：systemctl --user show-environment" "$work/q-watch.out" \
          && ((rc == 124)); } && echo 0 || echo 1)" "$work/q-list.out"
  PATH="$shim_bus:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_q" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}q2" -- true > "$work/q2.out" 2>&1; rc=$?
  PATH="$shim_bus:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_q" bash "$detached_run_script" stop "${prefix}q" >> "$work/q2.out" 2>&1; got=$?
  detached_run_expect "Ⓠ 查不到单元状态：start 拒、退 1、没建登记；stop 拒、退 1、没记 stop-requested" \
    "$( { ((rc == 1)) && [[ ! -d "$state_q/${prefix}q2" ]] && [[ "$got" == 1 ]] && [[ ! -f "$state_q/${prefix}q/stop-requested" ]] \
          && grep -q "查不到单元" "$work/q2.out"; } && echo 0 || echo 1)" "$work/q2.out"
  # 同名的旧登记在（q 没有退出码、不知道还在不在跑）：start 同名照拒、退 1，旧登记留在原处、没挪成 <名>@<秒数>
  wanted="$(detached_run_field "$state_q/${prefix}q" started)"
  PATH="$shim_bus:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_q" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}q" -- true > "$work/q3.out" 2>&1; rc=$?
  ls -a "$state_q" >> "$work/q3.out" 2>&1
  detached_run_expect "Ⓠ 查不到单元状态时 start 同名：拒、退 1，旧登记留在原处（没挪走）" \
    "$( { ((rc == 1)) && [[ "$(detached_run_field "$state_q/${prefix}q" started)" == "$wanted" ]] \
          && ! compgen -G "$state_q/${prefix}q@*" > /dev/null; } && echo 0 || echo 1)" "$work/q3.out"

  # Ⓙ 查不到之后又查得到：watch 开跑时用户级 systemd 连不上（替身 systemctl 在 bus-down-j 在时退 1，不在时转给真的），这期间那条跑完、写了退出码；
  #   连上之后 watch 照报它的退出码 0，不报「观察到 0 条」
  local shim_toggle="$work/shim-toggle" state_j="$work/state-j" watch_pid_j
  detached_run_selftest_shim "$shim_toggle" systemctl \
    "if [[ -e $(printf '%q' "$work/bus-down-j") ]]; then
  for argument in \"\$@\"; do [[ \"\$argument\" == show-environment ]] && exec $(printf '%q' "$real_systemctl") \"\$@\"; done
  echo 'Failed to connect to bus: No such file or directory（自证的替身）' >&2
  exit 1
fi
exec $(printf '%q' "$real_systemctl") \"\$@\""
  detached_run_selftest_state_directory "$state_j"
  mkdir -p "$state_j/${prefix}j"
  detached_run_write_field "$state_j/${prefix}j" unit "${DETACHED_RUN_UNIT_PREFIX}${prefix}j.service"
  detached_run_write_field "$state_j/${prefix}j" started "$(date +%s)"
  detached_run_write_field "$state_j/${prefix}j" command "sleep 300"
  : > "$work/bus-down-j"
  PATH="$shim_toggle:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_j" timeout 300 bash "$detached_run_script" watch --every 1 > "$work/j-watch.out" 2>&1 &
  watch_pid_j=$!
  detached_run_selftest_wait_for_output "$work/j-watch.out" "查不到  ${prefix}j" 1200 "$watch_pid_j"
  detached_run_write_field "$state_j/${prefix}j" ended "$(date +%s)"
  detached_run_write_field "$state_j/${prefix}j" exit 0
  rm -f -- "$work/bus-down-j"
  wait "$watch_pid_j"; rc=$?
  echo "watch 退出码 $rc" >> "$work/j-watch.out"
  detached_run_expect "Ⓙ 查不到之后又查得到、那条已结束：watch 退 0、照报 ${prefix}j 退出码 0" \
    "$( { ((rc == 0)) && grep -q "^  · ${prefix}j 退出码 0\$" "$work/j-watch.out"; } && echo 0 || echo 1)" "$work/j-watch.out"

  # Ⓢ systemd-run 没起成（替身 systemd-run 退 1）：start 退 1，登记里记 start-failed，list 列成「没起成」、不当成被 KILL 的
  local shim_run="$work/shim-run" state_s="$work/state-s"
  detached_run_selftest_shim "$shim_run" systemd-run "echo 'Failed to start transient service unit（自证的替身）' >&2
exit 1"
  detached_run_selftest_state_directory "$state_s"
  PATH="$shim_run:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_s" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}s" -- true > "$work/s.out" 2>&1; rc=$?
  DETACHED_RUN_STATE_DIRECTORY="$state_s" bash "$detached_run_script" list >> "$work/s.out" 2>&1
  detached_run_expect "Ⓢ systemd-run 没起成：start 退 1、登记里有 start-failed、list 列成「没起成」" \
    "$( { ((rc == 1)) && [[ -f "$state_s/${prefix}s/start-failed" ]] && grep -q "没起成  ${prefix}s" "$work/s.out" \
          && grep -q "没起成 1 条" "$work/s.out"; } && echo 0 || echo 1)" "$work/s.out"

  # Ⓡ 同名重起：watch 观察到 r 在跑；r 被 stop（143）之后隔一秒同名又起一次（退 6）；watch 结束时照报观察到的那一次的 143
  local state_r="$work/state-r" watch_pid
  detached_run_selftest_state_directory "$state_r"
  DETACHED_RUN_STATE_DIRECTORY="$state_r" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}r" -- sleep 300 > "$work/r.out" 2>&1
  detached_run_selftest_wait_for_file "$state_r/${prefix}r/pid" 100
  DETACHED_RUN_STATE_DIRECTORY="$state_r" timeout 90 bash "$detached_run_script" watch --every 8 > "$work/r-watch.out" 2>&1 &
  watch_pid=$!
  detached_run_selftest_wait_for_output "$work/r-watch.out" "还在跑  ${prefix}r" 1200 "$watch_pid"
  DETACHED_RUN_STATE_DIRECTORY="$state_r" bash "$detached_run_script" stop "${prefix}r" >> "$work/r.out" 2>&1
  detached_run_selftest_r_archive="$state_r/${prefix}r@$(detached_run_field "$state_r/${prefix}r" started)"
  sleep 1.1   # 起跑秒数要与上一次不同，旧登记才挪得到 <名>@<上一次的秒数>
  DETACHED_RUN_STATE_DIRECTORY="$state_r" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}r" -- bash -c 'sleep 1; exit 6' >> "$work/r.out" 2>&1
  wait "$watch_pid"; rc=$?
  { echo "watch 退出码 $rc"; cat "$work/r.out"; } >> "$work/r-watch.out"
  detached_run_expect "Ⓡ 同名重起：watch 退 0，照报观察到的那一次的退出码 143（stop 停的，登记已挪走）" \
    "$( { ((rc == 0)) && grep -q "^  · ${prefix}r 退出码 143（detached-run.sh stop 停的）" "$work/r-watch.out"; } && echo 0 || echo 1)" \
    "$work/r-watch.out"

  # Ⓧ 登记被挪走、单元还在跑：list 列成「登记里没有」
  local state_x="$work/state-x"
  detached_run_selftest_state_directory "$state_x"
  DETACHED_RUN_STATE_DIRECTORY="$state_x" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}x" -- sleep 300 > "$work/x.out" 2>&1
  detached_run_selftest_wait_for_file "$state_x/${prefix}x/pid" 100
  mv -- "$state_x/${prefix}x" "$work/moved-x"
  DETACHED_RUN_STATE_DIRECTORY="$state_x" bash "$detached_run_script" list >> "$work/x.out" 2>&1
  detached_run_expect "Ⓧ 登记被挪走而单元还在跑：list 列成「登记里没有  单元 $unit_x」" \
    "$(grep -q "登记里没有  单元 $unit_x" "$work/x.out" && echo 0 || echo 1)" "$work/x.out"

  # Ⓦ watch 观察到 w 在跑之后，w 的登记被挪走而单元还在跑：watch 按记下的单元名查到它还在跑，列成「还在跑（登记里没有）」，
  #   不报「都结束了」（之后又列完一轮仍不退出，自证给 timeout 发 ALRM 让它当场到点、退 124）
  local state_w="$work/state-w" watch_pid_w
  detached_run_selftest_state_directory "$state_w"
  DETACHED_RUN_STATE_DIRECTORY="$state_w" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}w" -- sleep 300 > "$work/w.out" 2>&1
  detached_run_selftest_wait_for_file "$state_w/${prefix}w/pid" 100
  # 「观察到之后」挪走，挪在最容易出岔子的那一下：替身 head 数 watch 读 w 的单元名的次数（每一轮清单一次、记观察一次），第 4 次（第二轮记观察，
  #   起跑秒数刚读完）先把登记挪走再读——单元名读不到，watch 要用第一轮记下的那一个去查
  local real_head shim_w="$work/shim-w"
  real_head="$(command -v head)"
  detached_run_selftest_shim "$shim_w" head "if [[ \" \$* \" == *$(printf '%q' " $state_w/${prefix}w/unit ")* ]]; then
  printf 'x\n' >> $(printf '%q' "$work/w-unit-reads")
  if [[ \"\$(wc -l < $(printf '%q' "$work/w-unit-reads"))\" == 4 ]]; then
    mv -- $(printf '%q' "$state_w/${prefix}w") $(printf '%q' "$work/moved-w") && : > $(printf '%q' "$work/w-moved")
  fi
fi
exec $(printf '%q' "$real_head") \"\$@\""
  PATH="$shim_w:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_w" timeout 600 bash "$detached_run_script" watch --every 1 > "$work/w-watch.out" 2>&1 &
  watch_pid_w=$!
  detached_run_selftest_wait_for_file "$work/w-moved" 1200
  if [[ ! -e "$work/w-moved" ]]; then mv -- "$state_w/${prefix}w" "$work/moved-w"; fi   # 替身没挪成（watch 先退了）：照样挪走，收尾按 moved-w 停
  # 挪走之后 watch 列出「还在跑（登记里没有）」、又列完一轮仍不退出，就给 timeout 发 ALRM 让它当场到点（与到点同一个结局：停 watch、退 124）；
  #   watch 先退了（当成都结束了）就不等
  detached_run_selftest_wait_for_output "$work/w-watch.out" "还在跑（登记里没有）  ${prefix}w " 1200 "$watch_pid_w" \
    && detached_run_selftest_wait_for_round_after "$work/w-watch.out" "还在跑（登记里没有）  ${prefix}w " 600 "$watch_pid_w"
  kill -ALRM "$watch_pid_w" 2>/dev/null
  wait "$watch_pid_w"; rc=$?
  { echo "watch 退出码 $rc"; cat "$work/w.out"; } >> "$work/w-watch.out"
  detached_run_expect "Ⓦ 观察到的登记被挪走而单元还在跑：watch 列成「还在跑（登记里没有）  ${prefix}w」，不报都结束了（退 124）" \
    "$( { ((rc == 124)) && grep -q "还在跑（登记里没有）  ${prefix}w " "$work/w-watch.out" && ! grep -q "都结束了" "$work/w-watch.out"; } && echo 0 || echo 1)" \
    "$work/w-watch.out"

  # Ⓦ systemd-run 退 1 而单元照起（替身 systemd-run 先转给真的、再退 1）：start 回读到单元在跑，记 systemd-run-failed、不记 start-failed、退 0，
  #   list 列成还在跑；之后在同一份登记里补写 start-failed（start 当时没查到单元），list 照按单元名查、仍列成还在跑
  local shim_run_w2="$work/shim-run-w2" state_w2="$work/state-w2" real_systemd_run
  real_systemd_run="$(command -v systemd-run)"
  detached_run_selftest_shim "$shim_run_w2" systemd-run "$(printf '%q' "$real_systemd_run") \"\$@\"
echo \"自证的替身：真的 systemd-run 退 \$?，这里报 1\" >&2
exit 1"
  detached_run_selftest_state_directory "$state_w2"
  PATH="$shim_run_w2:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_w2" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}w2" -- sleep 300 > "$work/w2.out" 2>&1; rc=$?
  DETACHED_RUN_STATE_DIRECTORY="$state_w2" bash "$detached_run_script" list >> "$work/w2.out" 2>&1
  detached_run_expect "Ⓦ systemd-run 退 1 而单元照起：start 退 0、登记里有 systemd-run-failed 没有 start-failed、list 列成还在跑" \
    "$( { ((rc == 0)) && [[ -f "$state_w2/${prefix}w2/systemd-run-failed" && ! -f "$state_w2/${prefix}w2/start-failed" ]] \
          && grep -q "还在跑  ${prefix}w2 " "$work/w2.out"; } && echo 0 || echo 1)" "$work/w2.out"
  detached_run_write_field "$state_w2/${prefix}w2" start-failed "自证补写的 start-failed（模拟 start 当时没查到单元）"
  DETACHED_RUN_STATE_DIRECTORY="$state_w2" bash "$detached_run_script" list > "$work/w3.out" 2>&1
  detached_run_expect "Ⓦ 登记里记了 start-failed 而单元在跑：list 按单元名查，列成还在跑，不列「没起成」" \
    "$( { grep -q "还在跑  ${prefix}w2 " "$work/w3.out" && ! grep -q "没起成  ${prefix}w2" "$work/w3.out"; } && echo 0 || echo 1)" "$work/w3.out"

  # Ⓥ --log 写不进（所在目录 555）：start 退 1、不起单元、不留登记、没打「✓ 起了」
  local state_v="$work/state-v"
  detached_run_selftest_state_directory "$state_v"
  mkdir -p "$work/ro"
  chmod 555 "$work/ro"
  DETACHED_RUN_STATE_DIRECTORY="$state_v" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}v1" --log "$work/ro/run.log" -- touch "$work/v1.marker" \
    > "$work/v1.out" 2>&1; rc=$?
  sleep 0.5
  echo "start 退 $rc；单元 $(detached_run_unit_state "${DETACHED_RUN_UNIT_PREFIX}${prefix}v1.service")；marker $([[ -e "$work/v1.marker" ]] && echo 在 || echo 不在)" >> "$work/v1.out"
  detached_run_expect "Ⓥ --log 写不进：start 退 1、不留登记、命令没跑、没打「✓ 起了」" \
    "$( { ((rc == 1)) && [[ ! -d "$state_v/${prefix}v1" && ! -e "$work/v1.marker" ]] && ! grep -q "✓ 起了" "$work/v1.out" \
          && grep -q "写不进" "$work/v1.out"; } && echo 0 || echo 1)" "$work/v1.out"
  # 起单元之前的核关掉（这一格自己带弄坏开关 log-unchecked）：单元起了而命令没跑起来，起完回读照样认得出、退 1，list 注明没跑起来
  DETACHED_RUN_BREAK="${DETACHED_RUN_BREAK:+$DETACHED_RUN_BREAK,}log-unchecked" DETACHED_RUN_STATE_DIRECTORY="$state_v" \
    bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}v2" --log "$work/ro/run.log" -- touch "$work/v2.marker" > "$work/v2.out" 2>&1; rc=$?
  detached_run_selftest_wait_for_file "$state_v/${prefix}v2/exit" 100
  DETACHED_RUN_STATE_DIRECTORY="$state_v" bash "$detached_run_script" list >> "$work/v2.out" 2>&1
  detached_run_expect "Ⓥ 起单元之前没核日志时：起完回读认得出命令没跑起来（有退出码没有命令进程号）、退 1，list 注明没跑起来" \
    "$( { ((rc == 1)) && [[ ! -e "$work/v2.marker" ]] && grep -q "命令没跑起来" "$work/v2.out" && ! grep -q "✓ 起了" "$work/v2.out" \
          && grep -q "已结束  ${prefix}v2 .*命令没跑起来" "$work/v2.out"; } && echo 0 || echo 1)" "$work/v2.out"
  # 命令退出之后单元里写登记失败（命令把自己的登记目录改成不可写、退 3）：旁路文件记下退出码，list 列成「登记写不进」、报退出码 3，不列「没留退出码」
  local state_v3="$work/state-v3" side_v3
  detached_run_selftest_state_directory "$state_v3"
  mkdir -p "$work/v3-log"
  DETACHED_RUN_STATE_DIRECTORY="$state_v3" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}v3" --log "$work/v3-log/run.log" -- \
    bash -c 'chmod a-w "$0"; exit 3' "$state_v3/${prefix}v3" > "$work/v3.out" 2>&1
  side_v3="$(detached_run_field "$state_v3/${prefix}v3" write-failure-file)"
  tries=0
  while detached_run_is_active_state "$(detached_run_unit_state "${DETACHED_RUN_UNIT_PREFIX}${prefix}v3.service")" && ((tries < 100)); do
    sleep 0.1
    tries=$((tries + 1))
  done
  DETACHED_RUN_STATE_DIRECTORY="$state_v3" bash "$detached_run_script" list >> "$work/v3.out" 2>&1
  echo "旁路文件 ${side_v3:-（登记里没写）}：$(cat "$side_v3" 2>/dev/null || echo 不在)" >> "$work/v3.out"
  detached_run_expect "Ⓥ 命令退出后登记写不进：list 列成「登记写不进  ${prefix}v3」、报退出码 3，不列成没留退出码" \
    "$( { [[ -n "$side_v3" && -f "$side_v3" ]] && grep -q "登记写不进  ${prefix}v3 .*命令退出码 3" "$work/v3.out" && grep -q "登记写不进 1 条" "$work/v3.out" \
          && ! grep -q "没留退出码  ${prefix}v3" "$work/v3.out"; } && echo 0 || echo 1)" "$work/v3.out"

  # Ⓟ -- 之后的 --force：preflight 摘掉每一个，单元里跑的是 bash -c … a b；两道闸切参数用的 lib_shell_words.detached_run_start_command
  #   交回的词与登记里的命令逐字相同（LIB_SHELL_WORDS_BREAK=detached-run-force-kept 下这一格红）
  local hooks_directory code_p hook_view=() hook_text recorded_text
  hooks_directory="$(cd "$(dirname "$detached_run_script")/../../.claude/hooks" 2>/dev/null && pwd)"
  code_p='printf "%s\n" "$@" > "$0"'
  bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}p" -- --force bash -c "$code_p" "$work/p.args" a --force b > "$work/p.out" 2>&1; rc=$?
  detached_run_selftest_wait_for_file "$state_directory/${prefix}p/exit" 100
  if [[ -n "$hooks_directory" ]]; then
    mapfile -d '' -t hook_view < <(python3 -B -c 'import sys
sys.path.insert(0, sys.argv[1])
import lib_shell_words
words = lib_shell_words.detached_run_start_command(sys.argv[2:]) or []
sys.stdout.write("".join(word + "\0" for word in words))' "$hooks_directory" start --wall-clock within-2h --label "${prefix}p" -- --force bash -c "$code_p" "$work/p.args" a --force b)
  fi
  hook_text="$(printf '%q ' ${hook_view[@]+"${hook_view[@]}"})"
  recorded_text="$(detached_run_field "$state_directory/${prefix}p" command)"
  printf 'start 退 %s\n闸看到的：%s\n登记里的：%s\n命令收到的参数：%s\n' "$rc" "$hook_text" "$recorded_text" "$(tr '\n' ' ' < "$work/p.args" 2>/dev/null)" >> "$work/p.out"
  detached_run_expect "Ⓟ -- 之后的 --force 照 preflight 摘：命令收到 a b，闸切出的词与登记里的命令逐字相同" \
    "$( { ((rc == 0)) && [[ "$(cat "$work/p.args" 2>/dev/null)" == $'a\nb' ]] && [[ -n "$recorded_text" && "$hook_text" == "$recorded_text" ]]; } && echo 0 || echo 1)" \
    "$work/p.out"

  # Ⓣ 停机时限：--stop-timeout 2 设进单元（TimeoutStopUSec=2s）；命令不理 TERM，stop 到点被 KILL：退 0、没留退出码、登记里有 stop-killed、list 注明
  local state_t="$work/state-t" timeout_property
  detached_run_selftest_state_directory "$state_t"
  DETACHED_RUN_STATE_DIRECTORY="$state_t" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}t" --stop-timeout 2 -- bash -c 'trap "" TERM; exec sleep 300' \
    > "$work/t.out" 2>&1
  detached_run_selftest_wait_for_file "$state_t/${prefix}t/pid" 100
  timeout_property="$(systemctl --user show -p TimeoutStopUSec --value "$unit_t" 2>&1)"
  echo "单元的 TimeoutStopUSec：$timeout_property" >> "$work/t.out"
  if [[ "$timeout_property" == 2s ]]; then
    DETACHED_RUN_STATE_DIRECTORY="$state_t" bash "$detached_run_script" stop "${prefix}t" >> "$work/t.out" 2>&1; rc=$?
  else
    rc=1
    echo "停机时限没设进单元，不跑 stop（照默认时限要等 90 秒），收尾按单元名停" >> "$work/t.out"
  fi
  DETACHED_RUN_STATE_DIRECTORY="$state_t" bash "$detached_run_script" list >> "$work/t.out" 2>&1
  detached_run_expect "Ⓣ --stop-timeout 2 设进单元、命令不理 TERM：stop 到点 KILL、退 0、没留退出码、登记里有 stop-killed、list 注明被 KILL" \
    "$( { ((rc == 0)) && [[ "$timeout_property" == 2s ]] && [[ ! -f "$state_t/${prefix}t/exit" && -f "$state_t/${prefix}t/stop-killed" ]] \
          && grep -q "没留退出码  ${prefix}t .*被 KILL" "$work/t.out"; } && echo 0 || echo 1)" "$work/t.out"

  # Ⓛ Linger=no 时 start 在 stderr 警告一行、照起；Linger=yes 时不警告（替身 loginctl 打 no / yes）
  local state_l="$work/state-l"
  detached_run_selftest_state_directory "$state_l"
  detached_run_selftest_shim "$work/shim-linger-no" loginctl "echo no"
  detached_run_selftest_shim "$work/shim-linger-yes" loginctl "echo yes"
  PATH="$work/shim-linger-no:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_l" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}l" -- true \
    > "$work/l-no.out" 2>&1; rc=$?
  PATH="$work/shim-linger-yes:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state_l" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}l2" -- true \
    > "$work/l-yes.out" 2>&1; got=$?
  cat "$work/l-no.out" "$work/l-yes.out" > "$work/l.out"
  detached_run_expect "Ⓛ Linger=no 时 start 警告一行、照起（退 0）；Linger=yes 时不警告" \
    "$( { ((rc == 0)) && [[ "$got" == 0 ]] && grep -q "没开 linger" "$work/l-no.out" && ! grep -q "linger" "$work/l-yes.out"; } && echo 0 || echo 1)" \
    "$work/l.out"

  # Ⓨ 同名旧登记的起跑秒数不早于此刻（同一秒里同名重起；这里把旧登记的起跑秒数写成此刻之后 2 秒，免得赶上秒的边界判不出）：
  #   start 等到它之后那一秒再起，新登记的起跑秒数比旧的大，旧登记挪到 <名>@<旧秒数>
  local state_y="$work/state-y" old_started_y new_started_y
  detached_run_selftest_state_directory "$state_y"
  mkdir -p "$state_y/${prefix}y"
  old_started_y=$(($(date +%s) + 2))
  printf '%s\n' "${DETACHED_RUN_UNIT_PREFIX}${prefix}y.service" > "$state_y/${prefix}y/unit"
  printf '%s\n' "$old_started_y" > "$state_y/${prefix}y/started"
  printf '0\n' > "$state_y/${prefix}y/exit"
  DETACHED_RUN_STATE_DIRECTORY="$state_y" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}y" -- true > "$work/y.out" 2>&1; rc=$?
  new_started_y="$(detached_run_field "$state_y/${prefix}y" started)"
  echo "旧登记的起跑秒数 $old_started_y，新登记的 ${new_started_y:-（没有）}；挪走的：$(cd "$state_y" && ls -d "${prefix}y@"* 2>/dev/null | tr '\n' ' ')" >> "$work/y.out"
  detached_run_expect "Ⓨ 同名旧登记的起跑秒数不早于此刻：start 等到它之后再起、退 0，新登记的起跑秒数比旧的大，旧登记挪到 <名>@<旧秒数>" \
    "$( { ((rc == 0)) && [[ "$new_started_y" =~ ^[0-9]+$ ]] && ((new_started_y > old_started_y)) && [[ -d "$state_y/${prefix}y@$old_started_y" ]]; } && echo 0 || echo 1)" \
    "$work/y.out"

  # Ⓩ watch 开跑之后起了又跑完的短命令：长的 z1 在跑时起 watch（每 2 秒一轮），等它打完第一轮清单再起 z2（退 6，一下就完），
  #   等 z2 留下退出码再停 z1；watch 退 0、报 z2 的退出码 6（一轮清单都没见 z2 在跑，照报）
  local state_z="$work/state-z" watch_pid_z
  detached_run_selftest_state_directory "$state_z"
  DETACHED_RUN_STATE_DIRECTORY="$state_z" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}z1" -- sleep 300 > "$work/z.out" 2>&1
  detached_run_selftest_wait_for_file "$state_z/${prefix}z1/pid" 100
  DETACHED_RUN_STATE_DIRECTORY="$state_z" bash "$detached_run_script" watch --every 2 > "$work/z-watch.out" 2>&1 &
  watch_pid_z=$!
  detached_run_selftest_wait_for_output "$work/z-watch.out" "还在跑  ${prefix}z1" 1200 "$watch_pid_z"
  DETACHED_RUN_STATE_DIRECTORY="$state_z" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}z2" -- bash -c 'exit 6' >> "$work/z.out" 2>&1
  detached_run_selftest_wait_for_file "$state_z/${prefix}z2/exit" 100
  DETACHED_RUN_STATE_DIRECTORY="$state_z" bash "$detached_run_script" stop "${prefix}z1" >> "$work/z.out" 2>&1
  # watch 每 2 秒一轮，负载高时一轮清单要好几秒：至多等 60 秒，到点还没退就停它（只停这一格自己起的那个进程号）
  for ((tries = 0; tries < 600; tries++)); do
    kill -0 "$watch_pid_z" 2>/dev/null || break
    sleep 0.1
  done
  if kill -0 "$watch_pid_z" 2>/dev/null; then kill "$watch_pid_z"; fi
  if wait "$watch_pid_z"; then rc=0; else rc=$?; fi
  cat "$work/z-watch.out" >> "$work/z.out"
  detached_run_expect "Ⓩ watch 开跑之后起了又跑完的短命令：一轮清单都没见它在跑，watch 退 0 时照报 selftest-<进程号>-z2 退出码 6" \
    "$( { ((rc == 0)) && grep -q "^  · ${prefix}z2 退出码 6" "$work/z-watch.out"; } && echo 0 || echo 1)" "$work/z.out"

  # Ⓤ 命令留在别的 scope 里的进程（照内存包装：外面一层 bash 留在单元的 cgroup 里（后面还有一句 exit，bash 不把 systemd-run exec 掉），经 systemd-run --user --scope 起一层 sleep，它不在单元的 cgroup 里）：
  #   stop 停单元之后，那个 scope 也停了、sleep 不在了，日志里记了一行
  local state_us="$work/state-us" scope_us="${DETACHED_RUN_UNIT_PREFIX}${prefix}us-inner.scope" sleep_us="" scope_state_us=""
  detached_run_selftest_state_directory "$state_us"
  DETACHED_RUN_STATE_DIRECTORY="$state_us" bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}us" -- \
    bash -c 'systemd-run --user --scope --quiet --unit="$1" bash -c "echo \$\$ > \"\$0\"; exec sleep 300" "$2"; exit "$?"' wrapper-like "$scope_us" "$work/us.sleep" \
    > "$work/us.out" 2>&1
  detached_run_selftest_wait_for_file "$work/us.sleep" 100
  sleep_us="$(cat "$work/us.sleep" 2>/dev/null)"
  echo "scope 里的 sleep：${sleep_us:-（没起来）}，cgroup $(sed -n 's/^0:://p' "/proc/${sleep_us:-0}/cgroup" 2>/dev/null)" >> "$work/us.out"
  DETACHED_RUN_STATE_DIRECTORY="$state_us" bash "$detached_run_script" stop "${prefix}us" >> "$work/us.out" 2>&1
  for ((tries = 0; tries < 100; tries++)); do
    scope_state_us="$(detached_run_unit_state "$scope_us")"
    detached_run_is_active_state "$scope_state_us" || break
    sleep 0.1
  done
  echo "stop 之后 scope $scope_us 是 ${scope_state_us:-?}，sleep $([[ -n "$sleep_us" && "$(cat "/proc/$sleep_us/comm" 2>/dev/null)" == sleep ]] && echo 还在 || echo 不在)" >> "$work/us.out"
  detached_run_expect "Ⓤ 命令留在别的 scope 里的进程：stop 停单元之后，那个 scope 也停了、sleep 不在了，日志里记了一行" \
    "$( { [[ -n "$sleep_us" ]] && ! detached_run_is_active_state "$scope_state_us" && [[ "$(cat "/proc/$sleep_us/comm" 2>/dev/null)" != sleep ]] \
          && grep -qF "留在 $scope_us 里" "$state_us/${prefix}us/log"; } && echo 0 || echo 1)" "$work/us.out"

  # 长活①–⑧ --wall-clock 与长活声明（判法 .claude/hooks/lib_long_run.py）：样本脚本放 $work/long-run/，被起时只 touch 第一个参数给的标记；
  #   拒的几格核退出码、没建登记、标记不在、stderr 点名与出路；放的几格起真单元（lr-ok、lr-within）或只跑 long-run-check
  local long_run="$work/long-run" scripts_directory="${detached_run_script%/*}" lr_rc=() lr_index
  mkdir -p "$long_run"
  cat > "$long_run/bare.sh" <<'SAMPLE'
#!/usr/bin/env bash
# 样本：一样长活声明都没有（不分片的长跑法）
touch "$1"
SAMPLE
  cat > "$long_run/half.sh" <<'SAMPLE'
#!/usr/bin/env bash
# long-run-shards: 按行号 mod 片数切
touch "$1"
SAMPLE
  cat > "$long_run/hard.sh" <<'SAMPLE'
#!/usr/bin/env bash
# long-run-shards: 按行号 mod 片数切
# long-run-resume: 结果表里已有的行跳过，重起同一条命令只补没做完的
# long-run-hosts: 本机 31 线程 30G、第二台 31 线程 50G（--emit-hosts 与 heavy-thread-default 只当参考）
touch "$1"
SAMPLE
  cat > "$long_run/full.sh" <<'SAMPLE'
#!/usr/bin/env bash
# long-run-shards: 按行号 mod 片数切
# long-run-resume: 结果表里已有的行跳过，重起同一条命令只补没做完的
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
touch "$1"
SAMPLE
  # 多机一行的该拒的写法（长活⑨）：现算（点名 host-resources.py）、否定句里提 host-resources.py、没点名登记值、除以此刻在跑的重活数，
  # 以及规范句后接任何说明（后接「；」与说明、说明里有数字、写按此刻空闲现算、写机器忙时按负载降、写除以此刻同时在跑的重活数、写 host-resources.py 的 available_threads）；
  # 该放的两种：规范句带句末句号、规范句另起一行写普通注释（规范句取 lib_long_run.py 的 CANONICAL_HOSTS_SENTENCE，不在这里另抄）
  local lr_canonical lr_host_names lr_host_lines
  lr_canonical="$(python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import lib_long_run; print(lib_long_run.CANONICAL_HOSTS_SENTENCE)' "$scripts_directory/../../.claude/hooks")" \
    || { echo "  ✗ 读不到 .claude/hooks/lib_long_run.py 的 CANONICAL_HOSTS_SENTENCE（长活⑨ 的样本要它）"; exit 1; }
  lr_host_names=(hosts-live hosts-negated hosts-unnamed hosts-divide hosts-note hosts-digit hosts-note-idle hosts-note-load hosts-note-divide hosts-note-resources hosts-period hosts-note-line)
  lr_host_lines=(
    "台数取 crash-point-replay-shard-configuration-check.sh --emit-hosts，每台的线程与内存取 host-resources.py --json 现算"
    "台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不调 host-resources.py 现算"
    "台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程按多机配置里写的"
    "台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值除以此刻同时在跑的重活数"
    "$lr_canonical；双机时两台各取这一个数，机器忙也不降"
    "$lr_canonical；见第 2 节"
    "$lr_canonical；按此刻空闲现算每台槽数与线程"
    "$lr_canonical；机器忙时按负载降"
    "$lr_canonical；除以此刻同时在跑的重活数"
    "$lr_canonical；每台线程按 host-resources.py 的 available_threads 取"
    "$lr_canonical。"
    "$lr_canonical"$'\n# 说明：双机时两台各取这一个数，机器忙也不降（另起一行的普通注释，不带 long-run-hosts:）')
  for lr_index in "${!lr_host_names[@]}"; do
    printf '#!/usr/bin/env bash\n# 样本：多机一行的写法\n# long-run-shards: 按行号 mod 片数切\n# long-run-resume: 结果表里已有的行跳过，重起同一条命令只补没做完的\n# long-run-hosts: %s\ntouch "$1"\n' \
      "${lr_host_lines[$lr_index]}" > "$long_run/${lr_host_names[$lr_index]}.sh"
  done
  cat > "$long_run/short.sh" <<'SAMPLE'
#!/usr/bin/env bash
# long-run-shards: 按行号 mod 片数切
# long-run-resume: 结果表里已有的行跳过
# long-run-unshardable: 多机不行
touch "$1"
SAMPLE
  cat > "$long_run/unnamed.sh" <<'SAMPLE'
#!/usr/bin/env bash
# long-run-shards: 按行号 mod 片数切
# long-run-resume: 结果表里已有的行跳过
# long-run-unshardable: 装置只认本机那块盘，第二台上没有这块盘
touch "$1"
SAMPLE
  cat > "$long_run/named.sh" <<'SAMPLE'
#!/usr/bin/env bash
# long-run-shards: 按行号 mod 片数切
# long-run-resume: 结果表里已有的行跳过
# long-run-unshardable: 多机这一段缺：装置只认本机那块盘，第二台上没有这块盘
touch "$1"
SAMPLE

  # 长活① 没写 --wall-clock、写了认不出的值：用法错退 2、带出路、没建登记（DETACHED_RUN_BREAK=wall-clock-optional 下没写的那一次照起，这一格红）
  bash "$detached_run_script" start --label "${prefix}lw" -- bash "$long_run/full.sh" "$long_run/lw.marker" > "$work/lw.out" 2>&1; rc=$?
  bash "$detached_run_script" start --wall-clock 3h --label "${prefix}lw2" -- bash "$long_run/full.sh" "$long_run/lw2.marker" >> "$work/lw.out" 2>&1; got=$?
  echo "没写：start 退 $rc；写 3h：start 退 $got" >> "$work/lw.out"
  detached_run_expect "长活① 没写 --wall-clock、写了认不出的值：用法错退 2、带出路、没建登记" \
    "$( { ((rc == 2)) && [[ "$got" == 2 ]] && [[ ! -d "$state_directory/${prefix}lw" && ! -d "$state_directory/${prefix}lw2" ]] \
          && grep -q "没写 --wall-clock" "$work/lw.out" && grep -q "只认 within-2h 或 over-2h" "$work/lw.out" && grep -q "→ 怎么办" "$work/lw.out"; } && echo 0 || echo 1)" \
    "$work/lw.out"

  # 长活②–⑤ --wall-clock over-2h 的四种该拒的：没声明（nice 包一层，照样认出 bare.sh）、半截、写死额度、bash -c 一长串
  local lr_samples=("lr1|nice -n 19 bash $long_run/bare.sh $long_run/lr1.marker" "lr2|bash $long_run/half.sh $long_run/lr2.marker"
                    "lr3|bash $long_run/hard.sh $long_run/lr3.marker")
  local lr_words=()
  for lr_index in "${!lr_samples[@]}"; do
    read -r -a lr_words <<< "${lr_samples[$lr_index]#*|}"
    bash "$detached_run_script" start --wall-clock over-2h --label "${prefix}${lr_samples[$lr_index]%%|*}" -- "${lr_words[@]}" \
      > "$work/${lr_samples[$lr_index]%%|*}.out" 2>&1
    lr_rc[lr_index]=$?
  done
  bash "$detached_run_script" start --wall-clock=over-2h --label "${prefix}lr4" -- bash -c "touch $(printf '%q' "$long_run/lr4.marker"); true" > "$work/lr4.out" 2>&1
  lr_rc[3]=$?
  for lr_index in 1 2 3 4; do echo "lr$lr_index：start 退 ${lr_rc[lr_index - 1]}" >> "$work/lr$lr_index.out"; done
  detached_run_expect "长活② over-2h、被起的脚本一样声明都没有（不分片的长跑法）：拒、退 5、带出路、没建登记" \
    "$( { [[ "${lr_rc[0]}" == 5 ]] && [[ ! -d "$state_directory/${prefix}lr1" && ! -e "$long_run/lr1.marker" ]] && grep -q "缺 分片" "$work/lr1.out" \
          && grep -qF "$long_run/bare.sh" "$work/lr1.out" && grep -q "→ 怎么办" "$work/lr1.out"; } && echo 0 || echo 1)" "$work/lr1.out"
  detached_run_expect "长活③ over-2h、只写了分片一样（半截声明）：拒、退 5、点名缺续跑与多机、没建登记" \
    "$( { [[ "${lr_rc[1]}" == 5 ]] && [[ ! -d "$state_directory/${prefix}lr2" && ! -e "$long_run/lr2.marker" ]] && grep -q "缺 续跑.*多机" "$work/lr2.out" \
          && ! grep -q "缺 分片" "$work/lr2.out"; } && echo 0 || echo 1)" "$work/lr2.out"
  detached_run_expect "长活④ over-2h、多机一行写死线程与内存：拒、退 5、点名写死的额度、没建登记" \
    "$( { [[ "${lr_rc[2]}" == 5 ]] && [[ ! -d "$state_directory/${prefix}lr3" && ! -e "$long_run/lr3.marker" ]] \
          && grep -q "写死了额度：31 线程、30G" "$work/lr3.out"; } && echo 0 || echo 1)" "$work/lr3.out"
  detached_run_expect "长活⑤ over-2h、bash -c 一长串判不出被起的脚本：拒、退 5、出路写进一份脚本、没建登记" \
    "$( { [[ "${lr_rc[3]}" == 5 ]] && [[ ! -d "$state_directory/${prefix}lr4" && ! -e "$long_run/lr4.marker" ]] \
          && grep -q "判不出被起的是哪份脚本" "$work/lr4.out" && grep -q "把这条命令写进一份脚本" "$work/lr4.out"; } && echo 0 || echo 1)" "$work/lr4.out"

  # 长活⑥ long-run-check：分不了一行不到 8 个字、没点名缺的多机都拒（退 5），点名了的放（退 0）
  bash "$detached_run_script" long-run-check -- bash "$long_run/short.sh" x > "$work/lr6.out" 2>&1; rc=$?
  bash "$detached_run_script" long-run-check -- bash "$long_run/unnamed.sh" x >> "$work/lr6.out" 2>&1; got=$?
  bash "$detached_run_script" long-run-check -- bash "$long_run/named.sh" x >> "$work/lr6.out" 2>&1; lr_index=$?
  echo "short.sh 退 $rc；unnamed.sh 退 $got；named.sh 退 $lr_index" >> "$work/lr6.out"
  detached_run_expect "长活⑥ long-run-check：分不了一行不到 8 个字、没点名缺的多机都拒（退 5），点名了的放（退 0）" \
    "$( { ((rc == 5)) && [[ "$got" == 5 && "$lr_index" == 0 ]] && grep -q "只有 4 个字" "$work/lr6.out" && grep -q "没点名缺的 多机" "$work/lr6.out"; } && echo 0 || echo 1)" \
    "$work/lr6.out"

  # 长活⑦ long-run-check 跳过包装：run-with-memory-cap.sh 与 capped.sh 包着的 full.sh 退 0；包着 bare.sh 的退 5、点名的是 bare.sh（只判、不起包装）
  bash "$detached_run_script" long-run-check -- bash "$scripts_directory/run-with-memory-cap.sh" 1G bash "$scripts_directory/capped.sh" 1 \
    bash "$long_run/full.sh" x > "$work/lr7.out" 2>&1; rc=$?
  bash "$detached_run_script" long-run-check -- bash "$scripts_directory/run-with-memory-cap.sh" 1G bash "$long_run/bare.sh" x >> "$work/lr7.out" 2>&1; got=$?
  echo "包着 full.sh 退 $rc；包着 bare.sh 退 $got" >> "$work/lr7.out"
  detached_run_expect "长活⑦ long-run-check 跳过包装：run-with-memory-cap.sh 与 capped.sh 包着的三样齐的脚本退 0；包着没声明的脚本退 5、点名的是那份脚本" \
    "$( { ((rc == 0)) && [[ "$got" == 5 ]] && grep -qF "长活声明齐了（$long_run/full.sh" "$work/lr7.out" && grep -qF "长活声明（$long_run/bare.sh）" "$work/lr7.out"; } && echo 0 || echo 1)" \
    "$work/lr7.out"

  # 长活⑨ long-run-check 多机一行按登记值读：规范句（full.sh）放（退 0），带句末句号的规范句（hosts-period.sh）与规范句另起一行写普通注释（hosts-note-line.sh）放；
  # 现算写法、否定句里提 host-resources.py、没点名登记值、除以此刻在跑的重活数都拒（退 5，点名「不是规范写法」），规范句后接任何说明的六份
  # （「；」加说明、说明里有数字、按此刻空闲现算、按负载降、除以此刻同时在跑的重活数、host-resources.py 的 available_threads）都拒（退 5，点名「接了说明」）
  local lr9_names=(full hosts-period hosts-note-line hosts-live hosts-negated hosts-unnamed hosts-divide hosts-note hosts-digit hosts-note-idle hosts-note-load hosts-note-divide hosts-note-resources)
  local lr9_rc=() lr9_got="" lr9_ok=1
  for lr_index in "${!lr9_names[@]}"; do
    bash "$detached_run_script" long-run-check -- bash "$long_run/${lr9_names[$lr_index]}.sh" x > "$work/lr9-${lr9_names[$lr_index]}.out" 2>&1; lr9_rc[lr_index]=$?
    lr9_got+="${lr9_names[$lr_index]} ${lr9_rc[lr_index]}；"
    if ((lr_index < 3)); then
      ((lr9_rc[lr_index] == 0)) || lr9_ok=0
    else
      ((lr9_rc[lr_index] == 5)) || lr9_ok=0
      if ((lr_index < 7)); then
        grep -q "一行不是规范写法" "$work/lr9-${lr9_names[$lr_index]}.out" || lr9_ok=0
      else
        { grep -q "接了说明" "$work/lr9-${lr9_names[$lr_index]}.out" && ! grep -q "不是规范写法" "$work/lr9-${lr9_names[$lr_index]}.out"; } || lr9_ok=0
      fi
    fi
  done
  { echo "$lr9_got"; cat "$work"/lr9-*.out; } > "$work/lr9.out"
  detached_run_expect "长活⑨ long-run-check 多机一行按登记值读：规范句放、带句末句号的规范句放、规范句另起一行写普通注释放（退 0）；现算写法、否定句里提 host-resources.py、没点名登记值、除以此刻在跑的重活数、规范句后接任何说明（含数字的与不含数字的）都拒（退 5）" \
    "$( ((lr9_ok == 1)) && echo 0 || echo 1)" \
    "$work/lr9.out"

  # 长活⑩ 豁免表：登记在 .claude/long-run-live-hosts-exempt 里、sha256 对得上的脚本，文件头按现算读（hosts-live.sh 的同一份内容放在 listed.sh，退 0）；没登记的同一份内容（unlisted.sh）照拒（退 5）；
  #   登记了而脚本改过一个字节（edited.sh：登记的是改之前那一版的 sha256）也拒，点名 sha256 对不上（退 1）。
  #   样本树里放一份判法库的拷贝与一张登记 listed.sh、edited.sh 的表（带 sha256），库按自己所在的仓根找表
  local exempt_tree="$work/exempt-tree" exempt_listed_rc exempt_unlisted_rc exempt_edited_rc exempt_listed_sha256
  mkdir -p "$exempt_tree/.claude/hooks" "$exempt_tree/scripts"
  cp "$scripts_directory/../../.claude/hooks/lib_long_run.py" "$scripts_directory/../../.claude/hooks/lib_shell_words.py" "$scripts_directory/../../.claude/hooks/lib_heavy_tests.py" "$exempt_tree/.claude/hooks/"
  cp "$long_run/hosts-live.sh" "$exempt_tree/scripts/listed.sh"
  cp "$long_run/hosts-live.sh" "$exempt_tree/scripts/unlisted.sh"
  cp "$long_run/hosts-live.sh" "$exempt_tree/scripts/edited.sh"
  exempt_listed_sha256="$(sha256sum "$exempt_tree/scripts/listed.sh" | cut -d' ' -f1)"
  printf '\n# 自证：登记之后又改了一个字节\n' >> "$exempt_tree/scripts/edited.sh"   # 追加在 touch 那一行之后，文件头不变、sha256 变了
  printf '# 自证用的豁免表\nscripts/listed.sh  %s  # 自证：文件头按现算写、不能改\nscripts/edited.sh  %s  # 自证：登记的是改之前那一版的 sha256\n' \
    "$exempt_listed_sha256" "$exempt_listed_sha256" > "$exempt_tree/.claude/long-run-live-hosts-exempt"
  python3 "$exempt_tree/.claude/hooks/lib_long_run.py" script "$exempt_tree" -- bash "$exempt_tree/scripts/listed.sh" x > "$work/lr10.out" 2>&1; exempt_listed_rc=$?
  python3 "$exempt_tree/.claude/hooks/lib_long_run.py" script "$exempt_tree" -- bash "$exempt_tree/scripts/unlisted.sh" x >> "$work/lr10.out" 2>&1; exempt_unlisted_rc=$?
  python3 "$exempt_tree/.claude/hooks/lib_long_run.py" script "$exempt_tree" -- bash "$exempt_tree/scripts/edited.sh" x >> "$work/lr10.out" 2>&1; exempt_edited_rc=$?
  echo "listed.sh 退 $exempt_listed_rc；unlisted.sh 退 $exempt_unlisted_rc；edited.sh 退 $exempt_edited_rc" >> "$work/lr10.out"
  detached_run_expect "长活⑩ 豁免表里登记的脚本（sha256 对得上）文件头按现算读（退 0），没登记的同一份文件头照拒、登记了而 sha256 对不上的照拒（都退 1，detached-run.sh 把它转成退 5）" \
    "$( { ((exempt_listed_rc == 0)) && [[ "$exempt_unlisted_rc" == 1 && "$exempt_edited_rc" == 1 ]] && grep -qF "长活声明齐了（$exempt_tree/scripts/listed.sh" "$work/lr10.out" \
          && grep -qF "长活声明（$exempt_tree/scripts/unlisted.sh）" "$work/lr10.out" && grep -qF "长活声明（$exempt_tree/scripts/edited.sh）：scripts/edited.sh 在 .claude/long-run-live-hosts-exempt 里登记的 sha256 是 $exempt_listed_sha256" "$work/lr10.out"; } && echo 0 || echo 1)" \
    "$work/lr10.out"

  # 长活⑧ over-2h 三样齐了照起、命令跑了、stderr 有「长活声明齐了」；within-2h 不读声明（bare.sh）照起（对照）
  bash "$detached_run_script" start --wall-clock over-2h --label "${prefix}lr-ok" -- bash "$long_run/full.sh" "$long_run/lr-ok.marker" > "$work/lr8.out" 2>&1; rc=$?
  bash "$detached_run_script" start --wall-clock within-2h --label "${prefix}lr-within" -- bash "$long_run/bare.sh" "$long_run/lr-within.marker" >> "$work/lr8.out" 2>&1; got=$?
  detached_run_selftest_wait_for_file "$long_run/lr-ok.marker" 100
  detached_run_selftest_wait_for_file "$long_run/lr-within.marker" 100
  echo "over-2h 齐了：start 退 $rc；within-2h：start 退 $got" >> "$work/lr8.out"
  detached_run_expect "长活⑧ over-2h 三样齐了照起、命令跑了；within-2h 不读声明照起" \
    "$( { ((rc == 0)) && [[ "$got" == 0 ]] && [[ -e "$long_run/lr-ok.marker" && -e "$long_run/lr-within.marker" ]] && grep -q "长活声明齐了" "$work/lr8.out"; } && echo 0 || echo 1)" \
    "$work/lr8.out"

  # 收尾：a、e、i、u 用 stop 停（stop 那一格的第二遍），再逐个按单元名停剩下的，最后核这个前缀的单元一条不剩
  bash "$detached_run_script" stop "${prefix}a" > "$work/stop-a.out" 2>&1
  bash "$detached_run_script" stop "${prefix}e" > "$work/stop-e.out" 2>&1
  bash "$detached_run_script" stop "${prefix}i" > "$work/stop-i.out" 2>&1
  bash "$detached_run_script" stop "${prefix}u" > "$work/stop-u.out" 2>&1
  detached_run_selftest_cleanup
  leftover="$(systemctl --user list-units --all --plain --no-legend "${DETACHED_RUN_UNIT_PREFIX}${prefix}*" 2>&1)"
  printf '%s\n' "$leftover" > "$work/leftover.out"
  detached_run_expect "收尾之后 systemctl --user list-units 里 ${DETACHED_RUN_UNIT_PREFIX}${prefix}* 一条不剩" \
    "$( [[ -z "$leftover" ]] && echo 0 || echo 1)" "$work/leftover.out"

  if ((detached_run_selftest_failed > 0)); then
    echo "  ✗ detached-run.sh 自证 $detached_run_selftest_checked 格里 $detached_run_selftest_failed 格不对（逐格在上面）"
    echo "     → 怎么办：按不对的那一格修 detached-run.sh 再跑 --selftest；DETACHED_RUN_BREAK 设着的话本来就该红"
    exit 1
  fi
  echo "  ✓ detached-run.sh 自证 $detached_run_selftest_checked 格都对（用法错与拒绝、弄坏开关设在自证之外就拒、-- 之后开头的赋值名不合写法就拒、建不了登记时挪回同名旧登记且挪不回出声、退出码在单元收掉之后读得到、环境（连 PATH、HOME）与工作目录带进单元、watch 全部结束自己退、同名在跑就拒、默认停机时限设进单元、stop 只停登记的那一个、stop 等命令收尾完、调用方被停命令照跑、watch 被 TERM / INT / HUP 都不停单元、查不到单元状态时 list 单列 watch 不退且连续 3 次报警 start 与 stop 拒 start 同名不挪旧登记、查不到之后又查得到照报退出码、没起成的单列、同名重起照报旧一次的退出码、登记被挪走的单元照列、观察到的登记被挪走而单元在跑 watch 不算结束、systemd-run 报错而单元照起记成在跑、日志写不进不起单元、命令没跑起来认得出、单元里写登记失败单列、--force 摘法与闸一致、停机时限设进单元且到点 KILL 记进登记、Linger=no 时警告、同一秒里同名重起等到下一秒、watch 开跑之后起了又跑完的短命令照报、命令留在别的 scope 里的进程随单元停、--wall-clock 必填且 over-2h 判长活声明（没声明、半截、写死额度、判不出脚本、分不了没点名都拒，多机一行按登记值读：只认规范句（可带句末句号，后面不接任何说明），别的写法都拒，豁免表里登记且 sha256 对得上的脚本按现算读、sha256 对不上的拒，包装跳过，齐了照起）、收尾不剩单元）"
}

if [[ -n "${DETACHED_RUN_BREAK:-}" && "${1:-}" != --selftest ]] && ! detached_run_break_is break-honored-anywhere && ! detached_run_inside_selftest; then
  echo "  ✗ DETACHED_RUN_BREAK=$DETACHED_RUN_BREAK 设在 --selftest 之外：弄坏开关只在自证起的进程里生效，这一次什么都没做（退 4）" >&2
  echo "     → 怎么办：去掉它再跑（env -u DETACHED_RUN_BREAK bash $detached_run_script ${1:-<子命令>} …）；要证红就跑 DETACHED_RUN_BREAK=<项> bash $detached_run_script --selftest" >&2
  exit 4
fi
case "${1:-}" in
  start) shift; detached_run_start "$@" ;;
  long-run-check)
    shift
    [[ "${1:-}" == -- ]] || detached_run_usage_error "long-run-check 要写成 long-run-check -- <命令与参数…>"
    shift
    (($#)) || detached_run_usage_error "long-run-check 的 -- 之后没有命令"
    detached_run_long_run_check "$@" || exit 5 ;;
  list) shift; (($# == 0)) || detached_run_usage_error "list 不带参数"; detached_run_list ;;
  watch) shift; detached_run_watch "$@" ;;
  stop) shift; detached_run_stop "$@" ;;
  --selftest) shift; detached_run_selftest ;;
  *) detached_run_usage_error "认不出子命令「${1:-（没给）}」" ;;
esac
