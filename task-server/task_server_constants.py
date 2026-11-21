"""task_server.py 拆出的一份：节拍、阈值、名字这些常量，连同打日志、发消息、读写小文件这类不带状态的小函数（总说明在 task_server.py 文件头）。"""
import os
import sys
import task_server_lib as lib
import contextlib
import importlib.util
import json
import math
import re
import resource
import socket
import subprocess
import threading
import time


TICK_SECONDS = 30.0             # 用户（records/2026-10-30-编译绕过内存总上限.md）：「每秒循环太浪费资源了 我30s一次就好了」；提交、取消、尝试结束（runner 调 /wake）都当场叫醒，这个间隔只兜底
PEER_STARTS_PER_PASS = 4         # 一轮至多在第二台起这么多条：一齐起几十条时第二台的 sshd 会重置新连接（实测 100 条一齐交时两条 ssh 被重置）
PEER_REPASS_SECONDS = 2.0        # 第二台这一轮起满了：隔这么久再排一轮，不等满 TICK_SECONDS
UNIT_CHECK_SECONDS = 5.0
LOST_GRACE_SECONDS = 10.0
PEER_RECHECK_SECONDS = 60.0
RESOURCE_CACHE_SECONDS = 2.0
# 第二台的余量由探测线程读（Resources.peer_probe_loop；文件头 ②「第二台的余量不在排队线程里读」）：派活只读它最近一份，不在排队线程里等 ssh
PEER_PROBE_REFRESH_SECONDS = 2.0      # 探测线程隔这么久读一回（同 RESOURCE_CACHE_SECONDS：派活读到的至多旧这么多加一回探测的用时）
FAST_PEER_PROBE_REFRESH_SECONDS = 0.1
PEER_PROBE_TIMEOUT_SECONDS = 20       # 探测、df 与读多机配置（Peer）各一回的时限；超时的当这一回连不上
PEER_PROBE_FAILURE_HOLD_SECONDS = 30.0   # 读不成的记下原因，这么久之内不再去连（派活照这一句当第二台此刻用不了），过了再试
PEER_PROBE_STALE_SECONDS = 30.0       # 最近一份比这更旧（探测线程卡在第二台上）：当第二台此刻用不了，不照旧的那份派
START_TIMEOUT_SECONDS = 60
LEFTOVER_STOP_SECONDS = 10.0     # 尝试正常结束之后停掉单元与 scope 里留下的进程，至多等这么久回读 cgroup 空了没有（stop_leftovers；SIGKILL 当场生效，推的）
UNIT_STOP_SECONDS = 150          # 单元被停时给命令收尾的时限：multi-host-run.sh 停第二台那条命令至多等 30 秒再删第二台这一趟的本地目录，内存包装停自己的 scope，留足（推的）
CANCEL_SETTLE_MARGIN_SECONDS = 20.0   # 取消停了单元之后等对账写回结局：没留结果的过 lost_grace_seconds 才记，再加两次读单元的间隔与这一截（推的；cancel_outcome_reply）
CANCEL_SETTLE_POLL_SECONDS = 0.1      # 等结局写回时隔这么久现读一次任务、叫醒排队线程
STARVED_ROUNDS = 3               # 派活判排头能起而连着这么多轮（秒数按一轮的长短乘）没起成，/health 回不健康；判不能起的、队空不动不算
PROGRESS_STALL_SECONDS = 600.0   # 看进展（Scheduler.progress_problem）：一台照公式放得下排着的最小那条、却这么久一条都没起，/health 回不健康。取 10 分钟的依据：与 STALL_SECONDS
                                 # 同一个数（用户定一条任务日志满 10 分钟不涨才算没进展，服务自己没进展用同一把尺）；派活故意让排头等的几种都短得多（第二台一轮起满等 PEER_REPASS_SECONDS
                                 # 2 秒），等负载（CPU 闸）、等占盘、slice 兜底与照排队先后那两条规则等着的另外不算；队列卡死 4 个多小时那一次 10 分钟就报得出来
STALL_SECONDS = 600.0            # 用户定：一条任务的日志距上一次增长满 10 分钟、这段时间 CPU 也几乎没涨，服务停掉它、记 stalled
PEER_CPU_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_CPU poll=([0-9]+) usage_usec=([0-9]+) read=(yes|no)$")   # multi-host-run.sh 文件头 ⑤ 每轮一行
# 第二台那一层内存包装 scope 这一轮进程用的内存（memory.stat 的 anon 加 shmem）与见过的最大，multi-host-run.sh 每轮一行（文件头 ①「顶在上限上卡住」；不算日志在涨）
PEER_MEMORY_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_MEMORY poll=([0-9]+) current_bytes=([0-9]+) peak_bytes=([0-9]+) read=(yes|no)$")
# 第二台那条命令进程树里这一轮在 D 状态（不可中断的 IO 等待）的进程数，multi-host-run.sh 每轮一行（文件头 ①「卡在 IO 上」；不算日志在涨）
PEER_IO_WAIT_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_IO_WAIT poll=([0-9]+) blocked=([0-9]+) read=(yes|no)$")
# multi-host-run.sh 文件头 ⑤「线程」每轮一行：第二台那一趟进程树里状态 R 的线程数、线程总数、占线程最多的进程（线程数:进程名，名字里别的字换成 _）；文件头 ②「CPU 闸」的可跑线程数
PEER_THREADS_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_THREADS poll=([0-9]+) runnable=([0-9]+) total=([0-9]+) top=([0-9]+):([A-Za-z0-9._-]+) read=(yes|no)$")
# 第二台那一层内存包装 scope 这一轮的 memory.current 与 memory.stat 的五样，multi-host-run.sh 每轮一行（文件头 ①「内存最高那一刻」；不算日志在涨）
PEER_MEMORY_STAT_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_MEMORY_STAT poll=([0-9]+) current_bytes=([0-9]+) anon=([0-9]+) file=([0-9]+) file_dirty=([0-9]+) file_writeback=([0-9]+) shmem=([0-9]+) read=(yes|no)$")
PEER_EXTERNAL_KILL_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=(earlyoom|kernel) pid=([0-9]+) line=(.*)$")   # multi-host-run.sh 命令退非 0 之后读第二台日志打的一行（文件头 ① 与那份文件头 ⑤「被外部信号杀掉」）
SELF_CGROUP_PATH = "/proc/self/cgroup"   # 套在外层单元里的服务认自己所在单元读的那一份（文件头 ③「套在外层单元里的服务」）
IO_PRESSURE_PATH = "/proc/pressure/io"   # 本机此刻的 IO 压力（io-wait 提醒的依据写它的 some avg60；读不到照写读不到）
# 判卡住的窗口里算「有进展」的量（note_activity、activity_text；文件头 ①「卡住」的有进展）：进程内存或这一次的内存峰值这一窗口涨了至少这么多。
# 取 64 MiB（推的）：读数自己的抖动在几 MiB 以内，分配器一回拿的块在 2 MiB 到几十 MiB
ACTIVITY_MEMORY_BYTES = 64 * 1024 ** 2
QUEUE_ATTENTION_MINUTES = 30     # 排队过久的提醒门槛（分钟；文件头 ①「要处置的提醒」）：排满这么久 wait 叫醒一次，之后每翻一倍（60、120 分钟…）再叫一次。取 30 的理由（推的）：常见排队几分钟到十几分钟，
                                 # 满 30 分钟多半是第二台用不了、余量被大任务占着或预测过大；比 AGED_WAIT_SECONDS（两小时升高优先）早，叫醒时人还来得及改命令重交
QUIET_ATTENTION_MINUTES = 30     # 日志长期不涨而服务没停它的提醒门槛（分钟；文件头 ①「要处置的提醒」的 quiet-log-busy）：日志满这么久没涨 wait 叫醒一次，之后每翻一倍（60、120 分钟…）再叫一次。
                                 # 取 30 的理由（推的）：项目规则要单项跑得超过 5 分钟的每 5 分钟至少打一行进度（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」），
                                 # 满 30 分钟没涨是漏了六回；任务 7723 日志 37702 秒没涨，服务一直不停也没人叫
IO_WAIT_ATTENTION_MINUTES = 30   # 卡在 IO 上而服务没停它的提醒门槛（分钟；文件头 ①「要处置的提醒」的 io-wait）：每一轮都有进程在 D 状态满这么久 wait 叫醒一次，之后每翻一倍再叫一次。
LONG_RUN_ATTENTION_SECONDS = 600   # 跑得久而核没用起来（文件头 ①「要处置的提醒」的 long-run-idle-cores）：一次尝试跑满这么久才看它平均在用几个核
LONG_RUN_IDLE_CORE_FRACTION = 24 / 31   # long-run-idle-cores 的界：到此刻平均在用核数不到给的线程数的这么多（线程数 31 时即不到 24 核）才报，只提醒、照跑
LONG_RUN_IDLE_QUARTER_BREAK = 1 / 4     # 弄坏开关 long-run-idle-quarter-threshold 下那一种的界
FILLS_CAP_RUNS = 3                  # fills-any-cap：同一命令键最近这么多次跑成的尝试（只读进程内存口径的行；撞了上限的不算）都贴着各自上限才报
FILLS_CAP_FRACTION = 0.9            # 那一种的「贴着」：峰值不低于那一次上限的这么多（90%）
RUNAWAY_SCAN_SECONDS = 300.0       # 任务之外的进程空转（文件头 ⑥）每隔这么久扫一次两台
FAST_RUNAWAY_SCAN_SECONDS = 0.2    # 自证里的间隔
RUNAWAY_MINIMUM_CPU_SECONDS = 1800  # 累计 CPU 满这么多秒才算
RUNAWAY_MINIMUM_FRACTION = 0.5     # CPU 累计占已跑时长不低于这么多才算
RUNAWAY_RENOTIFY_SECONDS = 1800.0  # 还在的每隔这么久再报一次
RUNAWAY_REPORTS_BEFORE_STOP = 2    # 认得出所属会话的本机进程报过这么多次还在，下一次到点服务按号停掉（共享规则 command-safety.md：报了没响应，报告的一方可以终止）
RUNAWAY_STOP_GRACE_SECONDS = 10.0  # 先 TERM，过这么久还在发 KILL
PROCESS_LIST_SECONDS = 30          # 读一台的进程表至多等这么久；第二台按号停的时限是两段宽限再加这么久
INBOX_ROUND_SECONDS = 30.0       # 发消息那一路（Scheduler.inbox_loop；文件头 ⑥）每隔这么久扫一遍提醒、疑似卡住与服务健康（同排队线程一轮的长短）；结局一有就当场叫醒它，这个间隔只兜底
FAST_INBOX_ROUND_SECONDS = 0.5   # 自证里的间隔（TASK_SERVER_SELFTEST_FAST=1）
INBOX_SEND_SECONDS = 5.0         # 连会话的收件 socket、写完两行的时限：消息先备好再连，一次写完（Claude Code 30 秒里收不到一整行就关连接，cross-session-messaging 文档「The session's inbox socket」）
INBOX_DRAIN_SECONDS = 0.5        # 写完、关了写的那一半之后等对面收完关连接至多这么久（同 socat 读到输入完了之后的缺省等法）
PEER_UNAVAILABLE_NOTIFY_SECONDS = 180.0   # 第二台用不了满这么久才发「服务健康」：余量查不成这类一两轮就好的不叫醒人（note_peer 当场写告警）
FAST_PEER_UNAVAILABLE_NOTIFY_SECONDS = 1.0
PROGRESS_PUSH_MINUTES = 58   # 在跑进度（文件头 ⑥「在跑进度」；用户定的）：任务（分片的连同子任务）有尝试在跑满这么久，给提交它的会话发一条进度，之后每满这么久再发一条
UNCLAIMED_WARNING_MINUTES = 30   # 交任务时会话名下有结局满这么久没交给它（消息没送到、也没起 wait）的，打严厉警告（文件头 ②「提交时警告」②）：消息当场就发，满 30 分钟还没人取的是发不出去而会话也没取；
                                 # 结局满这么久没人取的服务再推一回、之后每翻一倍再推（文件头 ⑥「结局没人取」，异常表 unclaimed-outcome）
CONFIRM_TIMEOUT_SECONDS = 1800   # 交了没确认满这么久服务取消、推「没确认，已取消」（文件头 ⑦；异常表 unconfirmed）：与没人取的结局同一个时限
ANOMALY_REPUSH_SECONDS = 1800    # 门槛不随时长涨的提醒同一道门槛推过之后满这么久还在的再推一回、之后每翻一倍再推（文件头 ⑥「没处置时」；异常表那几行的「没处置时」）
ANOMALY_REPUSHED_KINDS = ("oversized-budget", "fills-any-cap", "long-run-unconfirmed", "cleanup-pending", "shard-advance-failed")   # 提醒门槛不随时长涨的那几种（lib.ANOMALY_TABLE_HEAD 那一段同名）
ACKNOWLEDGED_KINDS = ("oversized-budget", "fills-any-cap")   # 出路是「确要这么多就照跑」的：task confirm <号> --why 记照跑（anomaly_acknowledged），记过的不再再推
ATTENTION_HISTORY_KEPT = 8       # 任务行里每一种提醒记下的推过的那几回（reminder_history）至多留这么多回：再推的正文照它写前几次推了什么
STALL_GRACE_SECONDS = 600.0      # 用户：「至少要延迟十分钟后在判定吧」「如果准备时间或者编译时间长 这不是很容易被杀掉」：尝试起来后先宽限这么久不判，之后才量窗口
CPU_OVERSUBSCRIPTION = 3.4375    # 32 核的台就是 110（用户定：140 降到 110）。CPU 闸：一台此刻的负载（1 分钟负载与可跑进程数的大者）与服务在那台在跑的尝试的线程之和取大，加这一条的线程，不超过核数的这个倍数（cpu_problem）
CPU_OVERSUBSCRIPTION_LOOSE = 6   # 只给弄坏开关 cpu-oversubscription-three：CPU 闸照这个倍数判
PEAK_SEGMENT_CORES_COLUMN = "peak_segment_cores"   # 尝试记录里峰值段核数那一栏（finish_peak_segment_cores 记；文件头 ②「CPU 闸」）
# 尝试记录里 CPU 闸线程数那一栏：min(登记数, max(峰值段核数, 可跑线程数的最大))（finish_cpu_gate_threads 记，threads_by_record 读；没有这一栏的旧尝试不读；文件头 ②「CPU 闸」）
CPU_GATE_THREADS_COLUMN = "cpu_gate_threads"
# 线程总数远超登记数的提醒（文件头 ①「要处置的提醒」的 threads-over-registered；note_threads）：线程总数超过登记数的这么多倍、其中可跑的也超过登记数、占线程最多的那一个进程自己也超过登记数的这么多倍，连着这么多轮才推。
# 两个数是定的、没量过（推的：登记数的两倍以内算开线程时多算了几个辅助线程；连着三轮排除一次编译或链接冒出来的一阵）
THREADS_OVER_REGISTERED_FACTOR = 2
THREADS_OVER_REGISTERED_ROUNDS = 3
CPU_SETTLE_BREAK_SECONDS = 5     # 只给弄坏开关 cpu-threads-settling-only：只记起了不到这么久的尝试的线程
# CPU 闸里在跑的那一条起跑满这么久、量满一段才照实测的峰值段核数记（running_threads），峰值段每段也是这么长（note_peak_segment）；没满的照命令键的核数记录记（threads_by_record）。
# 取 120 秒（推的）：编译前解依赖、等包缓存锁一般在一分钟里（任务 13065 那一次从起到退 25 秒），留一倍余地；各段分开量、取最大那一段，前后用核不一样的任务按用得多的那一段记
CPU_MEASURED_AFTER_SECONDS = 120.0
# CPU 闸里没有跑到头的核数记录的命令（新起的这一条、在跑而没满 CPU_MEASURED_AFTER_SECONDS 的）先按这么多核记（threads_by_record）；起满 CPU_MEASURED_AFTER_SECONDS 之后照实测
# 要核数少的名额（文件头 ②「CPU 闸」；cpu_judgement）：超线的这一条要的核数不超过 CPU_SMALL_TASK_CORES 时照样放行，每台同时在跑的、靠名额起的尝试不超过 CPU_SMALL_TASK_SLOTS；
# 靠名额起的起满 CPU_MEASURED_AFTER_SECONDS、实测超过 CPU_SMALL_TASK_CORES 核的不再占名额（small_slot_released）。
# 两个数是定的、没量过（推的：负载几百的台上一两核的脚本多起几条只多占几个核，名额封顶让排满的台不被小任务越堆越多）
CPU_SMALL_TASK_CORES = 2
CPU_SMALL_TASK_SLOTS = 4
CPU_SMALL_SLOT_TEXT = "要核数少的名额"   # 名额那一半句与起跑说明里的字样（自证照它认）
STALL_CPU_CORES = 0.01           # 「CPU 几乎没涨」：这一个窗口里平均不到一个核的 1%（用户选「日志与 CPU 都不涨才停」）
RUN_HARD_LIMIT_SECONDS = 14400   # 四小时硬上限（文件头 ⑦「四小时硬上限」）：带 --full-gate 的之外累计满这么多秒服务停这一次、记结局 over-four-hours、不重排
RUN_LIMIT_SECONDS = 7200         # 跑满两小时提醒（文件头 ①「跑满两小时提醒」）：从第一次尝试起跑各次尝试在跑的时长累加到这么多秒，提交时的门禁之外记要处置的提醒 long-run-unconfirmed，服务不停
# 日志满窗口没涨而服务没停它的缘由（judge_stall_stop 记进 log_growth；quiet_log_items 只对 SPARED_CPU_UNREAD、SPARED_CPU_BUSY 报 quiet-log-busy，文件头 ①「要处置的提醒」）
SPARED_CPU_UNREAD = "cpu-unread"   # 窗口开头或此刻读不到 CPU
SPARED_CPU_BUSY = "cpu-busy"       # CPU 在涨
SPARED_IO_WAIT = "io-wait"         # 每一轮都有进程在 D 状态（文件头 ①「卡在 IO 上」）
SPARED_ACTIVITY = "activity"       # 进程内存在涨（文件头 ①「卡住」的有进展；activity_text）
STALL_AT_CAP_FRACTION = lib.STALL_AT_CAP_FRACTION   # 判卡住那一刻顶在上限上的那一档（98%；文件头 ①「顶在上限上卡住」）：定义与依据在 task_server_lib.py，
                                                    # 客户端拼打回理由的 loop_leak= 那一段拿同一档判每一次撞上限是不是顶满
AGED_WAIT_SECONDS = 7200.0       # 提交满这么久还在排的升成高优先：排到最前、先判（用户在 records/2026-10-30-编译绕过内存总上限.md 里定）；放不下时不挡后面、不留份（文件头 ②「满时限」）
PEER_SCOPE_LEFTOVERS_MARK = "第二台这一趟内存包装的 scope 停了之后还留着进程："   # confirm_peer_cleanup 写进 cleanup_pending 的开头；cleanup-pending 提醒照它换修改意见
PEER_SCOPE_LEFTOVERS_SHOWN = 10   # 那一句至多列这么多个留下的进程（multi-host-run.sh 记进清理回执的 scope_leftovers）
# 第二台清理恢复按尝试号互斥的锁表（confirm_peer_cleanup；keyed_lock 管，放下就删）：服务进程里只有一个调度器，锁表放在模块里，只备几样的假调度器上照样拿得到
PEER_CLEANUP_LOCKS, PEER_CLEANUP_LOCKS_GUARD = {}, threading.Lock()
# 读日志按尝试号互斥的锁表（log_content；keyed_lock 管，放下就删）：排队线程判卡住与收尾线程都读同一份日志、接着同一个位置读
LOG_CONTENT_LOCKS, LOG_CONTENT_LOCKS_GUARD = {}, threading.Lock()
# 整机内存线（文件头 ②「内存」的整机内存线；machine_memory_problem）：这台整机可用内存占总量的比例与 swap 空闲占总量的比例都低于它，这一轮不往这台派新任务。
# 取 earlyoom 的线（可用内存与 swap 都低于 10% 才杀进程）再加两个百分点（推的：留一截让在跑的涨回去时先撞这一条的停派，不撞 earlyoom）
MACHINE_MEMORY_LINE_FRACTION = 0.12
MACHINE_MEMORY_LINE_TEXT = "整机内存线"   # machine_memory_problem 那一句的起头（自证照它认）
WORKSPACE_WRITER_FLOOR_BYTES = 4 * 1024 ** 3   # 换文件、打补丁的工具内存上限的下限（workspace_writer_floor）
RUNNING_BOOKING_MARGIN_BREAK_BYTES = 1024 ** 3   # 只给弄坏开关 running-booked-with-margin：有 scope 的在跑尝试在此刻实测上加这么多记（running_booking）
# 在跑的记账取的是哪一个（running_booking 交、记账明细的 basis 与 control bookings 打的那一栏）：此刻实测、实测到了它的上限、报告里没有它的 scope 记整份上限
BOOKED_FROM_USAGE, BOOKED_FROM_CAP, BOOKED_WITHOUT_SCOPE, BOOKED_WHILE_YOUNG = "measured", "cap", "no-scope", "young"
RUNNING_PEAK_SAMPLE_SECONDS = 5   # note_running_sample 里同一条尝试这么久之内读到的几回并成一条：窗口里至多攒 窗口秒数 / 5 条
# slice 兜底（文件头 ②「内存」的 slice 兜底；slice_floor_problem）：那台 slice 实测还放得下的（总上限 − slice 里进程用的内存）不到这么多，这一轮不派新任务。
# 取 4 GiB（推的：一次 31 线程编译几十秒里的突增在几 GiB 量级，留出这一截让在跑的涨回峰值时先撞自己的 memory.max，不撞 slice）
SLICE_ROOM_FLOOR_BYTES = 4 * 1024 ** 3
SLICE_FLOOR_TEXT = "slice 兜底"   # slice_floor_problem 那一句的起头（自证照它认）
MEMORY_WAIT_TOP_RUNNING = 2   # 内存放不下的排队原因里点名这台记账最多的几条在跑任务（memory_wait_text）
JUDGE_HOST_MEMORY_PREFIX = "内存：要的空闲"   # host-dispatch.py 的 judge_host 内存放不下那一句的起头：服务照 memory_wait_text 换成写明两个数的那一句（memory_verdict_reworded）
# 记账对账（文件头 ②「内存」的记账对账；audit_bookings）：每隔 BOOKING_AUDIT_SECONDS 逐台比一回「这台在跑尝试的内存记账之和」与「这台实测的用量（报告里各 scope 回收不掉的用量之和）」，
# 记账之和不少于实测的 BOOKING_DRIFT_RATIO 倍、而且多出至少 BOOKING_DRIFT_BYTES，连着 BOOKING_DRIFT_ROUNDS 轮都这样，判「记账漂移」（只报不停任务）。
# 推的：一分钟一轮、三倍、8 GiB、三轮（一次编译冲高之后几十秒回落不报；一台上两三条各多记十来 GiB 的挡着排队的才报）
BOOKING_AUDIT_SECONDS = 60.0
BOOKING_DRIFT_RATIO = 3.0
BOOKING_DRIFT_BYTES = 8 * 1024 ** 3
BOOKING_DRIFT_ROUNDS = 3
BOOKING_DRIFT_PUSH_SECONDS = 600.0   # 判出之后推给被挡住的会话：头一回当场推，之后照 reminder_threshold 以这个数为门槛、每翻一倍再推一回（同一件事同一道门槛只推一次）
BOOKING_DRIFT_ROWS_SHOWN = 8         # 推给会话的正文里逐条列的尝试数上限（/health 与 control check 列全）
RUNNING_BOOKED_SUM_TEXT = "在跑的记账之和"   # 公式那一句里的字样（verdict_within_the_formula、progress_problem；自证照它认）
CPU_UNREAD_TEXT = "CPU：这台此刻读不到"   # cpu_problem 读不到核数或负载时那一句的起头（自证照它认）
WHOLE_FRACTION = 0.85            # 记账贴着整台的界（oversized-budget 提醒，budget_check）：记账不小于那台可用上限（总上限减那台此刻的余量，host_capacity）的这么多就报
IDLE_SHORT_TEXT = "这一条在每一台上都起不来：就算服务在那台跑的任务全收尾、放出它们此刻占的内存，整机空闲也不够要的空闲；它不挡排在它后面的任务"   # 排队原因里 idle_short_text 前面那一句（自证照它认）
WAIT_LIMIT_SECONDS = 60.0
BREAK_FIXED_MARGIN_BYTES = 2 * 1024 ** 3   # 只给弄坏开关 margin-fallback-fixed：哪台量不到余量时照改前退回的固定数（文件头 ②「余量」）
RECORD_WINDOW = 5
LOG_TAIL_BYTES = 65536
PROGRESS_PUSH_LISTED_ATTEMPTS = 6     # 在跑进度那一条里逐次列的在跑尝试至多这么多次，多的只报个数（Scheduler.push_progress）
PROGRESS_PUSH_TAIL_CHARACTERS = 200   # 在跑进度那一条里每次尝试的日志末行至多这么多个字
LOG_SCAN_BYTES = 4 * 1024 * 1024   # log_content 接着读日志时一次至多读这么多
# 用例「本次未跑」的标记行（文件头 ①「要处置的提醒」的 gpu-cases-not-run）：singlefs_verification::gpu_memory_ledger::gpu_not_run_line 打的
# `NOT_RUN_NO_GPU_ADAPTER test=<用例> reason=<为什么>`；记号是整词、一行里每一处都认、一处一条（gpu_not_run_test）：前面是行首、空白或别的非标识符字符，后面是空白或行尾，ANSI 颜色码先去掉再看。
# 整词的边界、颜色码与一行里每一处各是一截（marker_pieces）只在门禁那一份库里写一份，这里按文件路径导入（本服务的 sys.path 不一定有那个目录）；认的名字只有 GPU_NOT_RUN_MARK 一个
GATE_MARKER_LIBRARY_PATH = os.path.join(lib.REPOSITORY_ROOT, "scripts", "gate", "lib_libtest_result_lines.py")
_GATE_MARKER_LIBRARY_SPECIFICATION = importlib.util.spec_from_file_location("lib_libtest_result_lines_for_task_server", GATE_MARKER_LIBRARY_PATH)
gate_marker_library = importlib.util.module_from_spec(_GATE_MARKER_LIBRARY_SPECIFICATION)
_GATE_MARKER_LIBRARY_SPECIFICATION.loader.exec_module(gate_marker_library)
GPU_NOT_RUN_MARK = "NOT_RUN_NO_GPU_ADAPTER"
GPU_NOT_RUN_MARK_FORM_WITHOUT_EDGES = re.compile(re.escape(GPU_NOT_RUN_MARK))   # 只给弄坏开关 gpu-not-run-word-edges-unchecked：两侧都不判词界
GPU_NOT_RUN_TEST = re.compile(r"(?<!\S)test=(\S+)", re.ASCII)   # 记号后面点名用例的那一段：前面是空白（含制表符）；re.ASCII 让 \S 只认 ASCII 空白，与库的记号边界同义
GPU_NOT_RUN_WORD = re.compile(r"[A-Za-z0-9_]+", re.ASCII)   # 库交出的每一截从记号起头：开头连着的标识符字符是记号的全名（NOT_RUN_NO_GPU_ADAPTER_X 与 NOT_RUN_NO_GPU_ADAPTER 是两个词）
GPU_NOT_RUN_BREAK_SWITCHES = ("gpu-not-run-line-start-only", "gpu-not-run-word-edges-unchecked", "gpu-not-run-color-codes-kept", "gpu-not-run-first-mark-only")   # 只给自证：gpu_not_run_test 的 broken 参数认的开关名
GPU_NOT_RUN_UNNAMED = "（这一行没写 test=）"
GPU_NOT_RUN_TESTS_SHOWN = 8   # 结局说明与 gpu-cases-not-run 的依据里逐条点名的用例至多这么多条，余下只报条数
NOT_RUN_MARKS_COLUMN = "gpu_not_run_marks"   # 尝试行里记这一次日志里标记的那一栏：{"count": 记号的处数, "tests": [点名的用例，至多 GPU_NOT_RUN_TESTS_SHOWN 条]}
FAST_TICK_SECONDS = 0.1          # 自证的定时（TASK_SERVER_SELFTEST_FAST=1）
FAST_UNIT_CHECK_SECONDS = 0.3
FAST_LOST_GRACE_SECONDS = 1.0
FAST_STARVED_SECONDS = 1.0
FAST_PROGRESS_STALL_SECONDS = 3.0   # 自证里看进展的时限：比 FAST_STARVED_SECONDS 长，短于各格盯 /health 的 4 秒
NFS_RECHECK_SECONDS = 60.0       # 服务起来之后隔这么久复核一次 NFS 共享的前三项（task_server_lib.py 文件头「NFS 共享」；与 PEER_RECHECK_SECONDS 重读第二台同一个间隔）
FAST_NFS_RECHECK_SECONDS = 0.5
ACTIVE_ATTEMPT_STATES = ("starting", "running")
CLEANUP_RETRY_SECONDS = 5.0   # 清理没确认（cleanup_pending）的那一次隔这么久再重查（finish_attempt）
# 读不到单元状态的提醒门槛（秒；文件头 ①「要处置的提醒」的 unit-unreadable）：对账连着读不到一次尝试的单元状态满这么久推一回，之后每翻一倍再推。
# 取 60 秒（推的）：systemctl --user show 一回至多等 30 秒（unit_states），用户管理器重载、D-Bus 忙这类短时失灵在一两回里就恢复，不为它推；连着一分钟读不到就要人去核用户级 systemd
UNIT_UNREADABLE_ATTENTION_SECONDS = 60.0
FAST_UNIT_UNREADABLE_ATTENTION_SECONDS = 2.0   # 自证里的提醒门槛（TASK_SERVER_SELFTEST_FAST=1）
UNIT_STATES_UNREADABLE_FILE = "unit-states-unreadable"   # 只在自证里：TASK_SERVER_SELFTEST_WORLD 下有这份文件时 unit_states 交 None（演 systemctl --user show 读不到）
UNIT_UNREADABLE_HEALTH_TEXT = "本机读不到在跑尝试的单元状态"   # /health 报读不到单元状态那一句的起头（unit_unreadable_report；自证照它认）
UNIT_UNREADABLE_HEALTH_SHOWN = 5   # /health 那一句至多点名这么多次尝试，余下的报次数（unit_unreadable 栏照列全）
UNIT_CGROUPS_UNREADABLE_FILE = "unit-cgroups-unreadable"   # 只在自证里：TASK_SERVER_SELFTEST_WORLD 下有这份文件时本机核进程的 cgroup 根换成世界目录里不在的一处（演 cgroup 也读不到；LocalMachine.attempt_presence_readings）
ENDED_TAMPERED_FIELDS = ("number", "host", "unit", "label", "started_at", "session", "command_key")   # 核不过而记 ended 的那一次正文只留这几样（end_tampered_attempt）
RESIDENT_RETENTION_SECONDS = 7 * 24 * 3600   # 按任务号记在内存里的那几张表（forget_settled_entries）：有了结局没人取的，结局满这么久也删
RESIDENT_PRUNE_SECONDS = 600.0               # 至多隔这么久清一次那几张表（对账每一轮问一次）
FAST_RESIDENT_PRUNE_SECONDS = 0.5
# forget_settled_entries 清的按任务号记的表：{Scheduler 上的名字: 键里任务号的位置（None 是整个键就是任务号）}
RESIDENT_TASK_TABLES = {"alerted_child_refusals": 0, "reminded_thresholds": 0, "reminder_histories": 0, "workspace_write_checks": None,
                        "advance_failures": None, "advance_episodes": None, "unclaimed_repushed": None}
PAUSE_META_NAME = "pause"        # 暂停态在库 meta 表里的名字（文件头 ⑤「暂停」）
RESTART_SURVIVING_META_PREFIX = "restart-surviving:"   # 换服务不丢的调度判断在库 meta 表里的名字前缀（Store.restart_surviving_state）
# 换服务不丢的调度判断：{库里的名字: 只记内存的弄坏开关}；三样共用一份读写（Scheduler.persist_restart_surviving_state、restore_restart_surviving_state），
# 各自的内存表与库里那一份的互换在 Scheduler.restart_surviving_record、adopt_restart_surviving_record
RESTART_SURVIVING_STATES = {
    "runaway_notified": "runaway-reports-memory-only",        # 任务之外的空转进程报过几次、停没停（watch_runaway_processes）
    "shard_advance": "advance-episodes-memory-only",          # 分片父任务推不动的这一回与开始过几回（note_advance_failure）
    "suspect_reported": "suspect-reported-memory-only",       # 疑似卡住报给会话的是哪一段（push_suspects、unreported_suspect）
    "health_notified": "health-notified-memory-only",         # 服务健康每一样发给了哪几个会话（watch_health）
    "booking_drift_notified": "booking-drift-notified-memory-only",   # 记账漂移每一件推给每个会话推到了哪道提醒门槛（push_booking_drift）
    "gpu_alerted": "gpu-alerted-memory-only",                 # GPU 服务卡坏、判不了写过告警的种类（watch_health）
}
PAUSE_REASON_CHARACTERS = 200    # 暂停原因收的字数（一句话；多的截掉）
LISTING_ONLY_ARGUMENTS = ("--list-items", "--help", "-h")   # 带了就只列项、只打帮助，不编不跑（compiles_declared）
# 判定库读写接完 prune-server 的族：登记表第三列（判定库放哪）写「prune-server」的行（registry_families）照收；别的带判定库的提交时拒（文件头 ②「提交时拒」③）
VERDICT_LIBRARY_REGISTRY = os.path.join(".claude", "gate.d", "verdict-library-scripts.tsv")
REGISTRY_SERVICE_MARK = "prune-server"   # 第三列以它起头的行：那一族的判定库读写全在服务里
REGISTRY_NO_LIBRARY_PREFIXES = ("不算：", "无判定库：")   # 第三列以这两种起头的没有判定库（登记表文件头）
# 只派一台的名单全删了（用户：「我再重申一次 【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：任何命令都派得到两台，哪台由余量定，
# 派到第二台跑红的由脚本那一侧修。下面这几张是删之前的样子，只给弄坏开关 host-lists-restored 照改前放回用（文件头 ②「机器」）
RESTORED_HOST_LISTS = {
    "local_directories": ("scripts/hosts", ".claude/gate.d"),   # 点到这几个目录的只在本机
    "local_entries": (".claude/gate.d/doc-registries.sh", "scripts/crash/crash-point-replay-shard-configuration-check.sh", "prune-server/control",
                      "scripts/experiment-apparatus/e180-reading-shards.sh", "scripts/experiment-apparatus/e183-sweep-history-shards.sh"),   # 读本机才有的东西的入口
    "peer_entries": ("scripts/experiment-apparatus/e185-r2-segment-run.sh",),   # 只在第二台跑的入口
}
VOLATILE_ROOT = "/tmp"   # 重启会清掉的目录：提交时拒的共享根外路径落在它下面的，拒绝说明另说一句重启会清（文件头 ②「提交时拒」①）
# 按子串认词里的绝对路径（find_outside_shared_path：shell -c 脚本词切不开时，与弄坏开关 path-check-substring 下每个带空白的词）；前面紧挨着 \w、/（$VAR/…、a/b 的后半）、
# }（${X:?}/…）、)（$(cmd)/…）的是变量展开、命令替换之后的尾巴，真路径运行时才由展开给，不是命令写死的，不算；前面是空白、引号、=、: 一类的照判
QUOTED_WORD_PATH = re.compile(r"(?<![\w/}).])/[^\s'\"`;|&()<>]+")
# shell 的 -c 脚本那一个词（shell_script_word_indexes）：命令词名（basename）是这几个之一、选项里有带 c 的短选项串（-c、-ec、-lc）的，选项之后头一个词是脚本
SHELL_COMMAND_NAMES = ("bash", "sh", "dash", "zsh", "ksh")
SHELL_OPTIONS_WITH_VALUE = ("-o", "+o", "-O", "+O", "--rcfile", "--init-file")   # 后面跟一个值的 shell 选项（shell_script_word_indexes 跳过那个值）
# 一个 shell 词里的绝对路径（shell_word_paths）：在词头，或紧跟在 =（名字=、--选项= 的值）、挨着写的 ;|& 与重定向 <>、开子 shell 的 ( 后面，到挨着写的 ;|&()<>` 为止；
# 跟在 )、}、$… 后面的（$(pwd)/…、${X:?}/… 展开之后的尾巴）与词中间夹着的不算
SHELL_WORD_PATH = re.compile(r"(?:^|(?<=[=;|&<>(]))/[^;|&()<>`]+")
SHELL_WORD_PATH_WITH_EXPANSION_TAILS = re.compile(r"(?:^|(?<=[=;|&<>()}]))/[^;|&()<>`]+")   # 弄坏开关 expansion-tail-as-path 下用的：跟在 )、} 后面的展开之后的尾巴也取
# 两台相同的系统路径（find_outside_shared_path 不判它们在不在共享根下；与 scripts/hosts/multi-host-run.sh 的 MULTI_HOST_HOST_INDEPENDENT_PATHS 同一组）：以 / 收尾的是整个目录
HOST_INDEPENDENT_PATHS = ("/dev/null", "/dev/stdin", "/dev/stdout", "/dev/stderr", "/dev/fd/", "/usr/", "/bin/")
# 多机配置 SLICE_MEMORY_CAP_BY_HOST 里一台那一项的值（写法同 scripts/hosts/run-with-memory-cap.sh 读它的那一句：正整数加 K / M / G / T）与单位
SLICE_CAP_VALUE = re.compile(r"^([1-9][0-9]*)([KMGT])$")
SIZE_UNITS = {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}
# 改主工作区的工具（modifies_workspace：跑第二遍结果就变，撞上限这类本会重排的结局一律打回、不重排；派哪台不看它，第二台改的就是同一棵共享的树）
LONG_RUN_SPLIT_EXEMPT_FILE = ".claude/long-run-split-exempt"   # 用户点名豁免拆分的项（.claude/rules/long-run-split.md「豁免」）；long_run_split_exempt 读
WORKSPACE_BLOCKED_EXIT_CODE = 7   # scripts/gate/hook-swap.py 的退出码 7：上一趟换上没写完成，拒绝再换
PREVIOUS_SWAP_RUNNING_EXIT_CODE = 9   # scripts/gate/hook-swap.py 的退出码 9（它的 EXIT_PREVIOUS_SWAP_RUNNING）：上一趟没写完成而起它的进程还在这一台上，拒绝再换
SWAP_SCRATCH_BUSY_EXIT_CODE = 10   # scripts/gate/hook-swap.py 的退出码 10（它的 EXIT_SCRATCH_BUSY）：同一个 --scratch 正被另一趟 hook-swap.py 占着，或这个仓的真换正被另一趟占着（两种照日志里的拒绝说明分，scheduler_lifecycle 的 swap_busy_cause），没建树、没写文件
HOOK_SWAP_ENTRY = "scripts/gate/hook-swap.py"   # 带 --feed-only 的只喂不换（modifies_workspace 不算它改工作区）
WRITER_PATCH_ENTRY = "scripts/gate/apply-writer-patch.py"   # 带 --dry-run 的只在临时树上核（modifies_workspace 不算它改工作区）
WORKSPACE_READ_ONLY_ARGUMENTS = ("--selftest", "--list-items", "--dry-run", "--read-outcome", "--pending-status")   # 改工作区的工具带了这几样之一只读不写（自证、列项、干跑、读结局），modifies_workspace 不算（任务 7543、7549）
WORKSPACE_WRITERS = {"scripts/gate/hook-swap.py", "scripts/gate/apply-writer-patch.py", "scripts/gate/stage-mine.py"}
# 提交时拒 ⑤（文件头 ②「提交时拒」⑤）：这几种 cargo 不点名包就编整个工作区；fmt、metadata、tree、clean 不编，不在里面
WHOLE_WORKSPACE_SUBCOMMANDS = {"build", "b", "check", "c", "clippy", "test", "t", "run", "r", "bench", "doc", "d"}
WHOLE_WORKSPACE_FLAGS = ("--workspace", "--all")   # 带了就编整个工作区，点不点名包都一样
WHOLE_WORKSPACE_NEXT_STEP = "点名要编的包：-p <包>（验证包写 -p singlefs-verification）；整仓要跑的走 scripts/gate/merge-and-verify.sh 或提交时的门禁"
COPY_PROOF_STATE_TEXT = {"queued": "在排", "running": "在跑", "sharded": "在跑", lib.AWAITING_CONFIRMATION_STATE: "待确认"}   # 提交时拒 ⑨ 看的任务状态（没进父任务的子任务不看）
COPY_PROOF_COPY_ADVICE = "python3 scripts/hosts/minimal-copy.py <仓根> <草稿目录>/copy --with-test-inputs 另拷一份副本"
MERGE_AND_VERIFY_ENTRY = "scripts/gate/merge-and-verify.sh"   # 合入后验证：只在自己拷的快照上打补丁与验，不算改主工作区（modifies_workspace）
WORKSPACE_WRITER_NO_RETRY_OUTCOMES = ("memory-cap-hit", "squeezed-out", "peer-run-failed", lib.EXTERNAL_KILL_OUTCOME)   # 改主工作区的命令遇到这几种本会重排的结局一律打回、不重排
# 改主工作区的命令以这几种结局收尾（连同被信号杀的 command-failed）：可能写了一半，wait 报 workspace-write-interrupted（文件头 ①「要处置的提醒」）
WORKSPACE_WRITER_INTERRUPTED_OUTCOMES = WORKSPACE_WRITER_NO_RETRY_OUTCOMES + ("memory-cap-at-maximum", "cancelled", "stalled", "launcher-lost", lib.RUN_HARD_LIMIT_OUTCOME)
EXTERNAL_KILL_LOG_AFTER_SECONDS = 60.0   # 读杀进程的日志：尝试结束之后再往后看这么久（被杀的进程退了、构建工具收尾、命令才退）
EXTERNAL_KILL_LOG_SECONDS = 15.0         # 每读一次日志（journalctl）至多等这么久
SIGNAL_EXIT_CODES = range(129, 129 + 64)   # 128 + 信号号：attempt_runner 记被信号杀的命令就是这个码（attempt_runner.py 文件头 ⑥），bash 起的命令被信号杀同样退它
PATCH_WRITER_OUTCOME_NAME = "apply-outcome.tsv"   # WRITER_PATCH_ENTRY 的结局文件：补丁目录里这一份，--read-outcome 判打全了没有
PATCH_WRITER_READ_ONLY_ARGUMENTS = ("--dry-run", "--pending-status", "--read-outcome", "--selftest")   # 带这几样的不写主工作区
PATCH_WRITER_CHECK_SECONDS = 120   # 起一次 --read-outcome 至多等这么久（它只读结局文件、对着主工作区算几份文件的 sha256）
PATCH_WRITER_VERDICTS = {0: "打全了", 1: "已退回", 2: "结局文件读不了", 3: "没打完", 4: "打完之后文件又被改了"}   # --read-outcome 的退出码（apply-writer-patch.py 文件头「结局文件」）
PEER_HOST_SPECIFIC_VARIABLES = ("ROCKSDB_LIB_DIR", "ROCKSDB_STATIC", "SNAPPY_LIB_DIR", "SNAPPY_STATIC")   # 按台取值（灰度会话报 359：第二台的预编库核对红）
MAXIMUM_LAUNCHER_DEPTH = 4
# 盘余量（task_server_lib.py 文件头「挑机器」的「盘余量」）：一台编译目录所在盘不到这么多就这一轮不派那一台，两台同一个下限。一趟的编译目录量过 200G 级
#（records/2026-10-31-task-server巡检.md 问题 72：任务 2665 的合入后验证在第二台留了一份 213G 的编译目录），余量取不到一次合入后验证的四分之一，
# 那一趟编译会在中途写满那台的盘（本机是 TASK_SERVER_BUILD_DIRECTORY 所在盘，第二台是多机配置 BUILD_ROOT_BY_HOST 里它那一项所在盘，multi-host-run.sh 文件头 ⑤）
DISK_FLOOR_BYTES = 50 * 1024 ** 3
DISK_REPORT_KEY = "disk_available"   # 每台余量报告里它编译目录所在盘的余量（字节；Machine.probe 带上，报告里没有时 Machine.disk_free 现量）
PEER_DISK_PROBE_TEXT = "df -B1 --output=avail -- {directory}"   # 在第二台的编译根（BUILD_ROOT_BY_HOST 里它那一项）上起：末行是余量的字节数
IO_PRESSURE_REPORT_KEY = "io_pressure_full_avg300"   # 每台余量报告里这台此刻 /proc/pressure/io 的 full avg300（百分数；Resources.note_io_pressure 记，多机配置没写 IO_PRESSURE_LIMIT_KEY 就不读）
PEER_IO_PRESSURE_PROBE_TEXT = "cat /proc/pressure/io"   # 在第二台上起：输出就是它的 /proc/pressure/io
PEER_BUILD_REMOVE_SECONDS = 120   # multi-host-run.sh --remove-build-directory（第二台留着的编译目录）至多等这么久：核持有者、rm、回读各一次 ssh（推的，同 --recover-cleanup 的量级）
DISK_FLOOR_KIND = "disk-floor"   # note_peer 去重的原因种类：盘不够这一种只告警一次，余量的数变了不另写
# 编译目录占盘（task_server_lib.py 文件头「资源记录」的「编译目录占盘」；records/2026-10-31-task-server巡检.md 问题 76）：派到一台之前那台的盘余量要放得下
# 这条命令量过的编译目录峰值加 BUILD_DISK_FLOOR_BYTES（盘余量照上面的 DISK_REPORT_KEY）
BUILD_DISK_FLOOR_BYTES = DISK_FLOOR_BYTES
# 编译目录组（task_server_lib.py 文件头「编译目录」）：一棵树在一台上至多这么多个编译目录，同时会编的条数多过它的排着、排队原因点名组满。
# 与占盘那一层一起定的：一个编译目录量过 200G 级（上面 DISK_FLOOR_BYTES 那一段），三个是同一棵树上 clippy、点名测试、证红三条并行的量，
# 再多一棵树一台就要几百 G；开新的那一个照样过占盘那一判（这条命令的占盘预测加 BUILD_DISK_FLOOR_BYTES），接着用旧的那一个已占的盘在余量里扣过了
BUILD_POOL_SLOTS_PER_TREE = 3
# 编译目录组满的排队原因都以这一句起头（build_pool_problem 写、pool_only_tree 认：排头只缺编译目录名额的，别的资源都放得下）
BUILD_POOL_FULL_PREFIX = "编译目录组满"
# 树不在了的整组删（sweep_vanished_tree_pools）：服务起来时扫一遍，之后排队线程至多每隔这么久在后台再扫一遍（树：--cwd 解开符号链接后往上第一层有 .git 的那一层，没有就是解开后的 --cwd 本身；它删了，它的组不留到下一次重启）
BUILD_POOL_SWEEP_SECONDS = 600
# 本机编译根下照尝试标签起名、不在组里的目录那一扫（sweep_unpooled_attempt_label_directories）有删的或没删成的时打的汇总行的字样（自证照它认那一扫扫完了）
UNPOOLED_SWEEP_SUMMARY_MARK = "本机编译根下照尝试标签起名、不在组里的目录扫完"
# 启动回执超时（TS-06，records/2026-11-01-task-server深入审核.md）：systemd-run 超时只算「起没起不确定」，照预先定好的单元名读回；
# 读回单元不在、起跑标记也没有，过了这么久才确认没起、记 unit-start-failed
START_UNCERTAIN_GRACE_SECONDS = 120.0
FAST_START_UNCERTAIN_GRACE_SECONDS = 2.0
SELFTEST_PAUSE_SECONDS = 60.0   # 自证的停顿点至多等这么久放行（TASK_SERVER_SELFTEST_PAUSE_POINT）
PRUNE_SERVER_CHECK_SECONDS = 60   # 命令退 75（prune-server 不可用）时起一次 prune-server/control check 的时限
SCENE_KEEP_AFTER_COLLECT_SECONDS = 3600.0   # 现场在 task wait 取走结局之后再留这么久（task_server_lib.py 文件头「现场」：提交方读日志、照日志去读现场，一小时够，推的）
FAST_SCENE_KEEP_AFTER_COLLECT_SECONDS = 1.0
STOP_JOIN_SECONDS = 10.0          # 停服务时等排队线程与 NFS 复核线程停下的时限（两个合计；单元的 TimeoutStopSec 是 30 秒，文件头 ③「停服务」）
STOP_CHILD_GRACE_SECONDS = 3.0    # 停服务时给自己起的子进程发 TERM 之后至多等这么久再 KILL


def say(text):
    print(text, file=sys.stderr, flush=True)


SUBMIT_TRACE_TEXT = "提交进出"       # 一条提交走完打的那一行的字样（SubmitTrace；自证照它认）
SUBMIT_TRACE_SLOW_SECONDS = 5.0      # 提交那一路的一步本身用了这么久，当场另打一行（不等回话）


class SubmitTrace:
    """一条提交（POST /tasks）在服务里走过的几步与各自的时刻：收到（事件循环上，排进线程之前）、进线程、各步做完、回话。
    回话时打一行（以 SUBMIT_TRACE_TEXT 起头）：收到的时刻、每一步做完时距收到多少秒；平常一条提交只这一行。
    一步本身（距上一步）用了 SUBMIT_TRACE_SLOW_SECONDS 以上的，当场另打一行：后面卡住、客户端等不到回话时，日志里也看得到前面慢在哪一步。
    弄坏开关 submit-trace-silent 下一行都不打。"""

    def __init__(self, environment):
        self.silent = lib.broken("submit-trace-silent", environment)
        self.received_wall, self.received = time.time(), time.monotonic()
        self.previous = self.received
        self.steps = []          # [(这一步叫什么, 做完时距收到多少秒)]
        self.task_id = None

    def step(self, name):
        now = time.monotonic()
        self.steps.append((name, now - self.received))
        if now - self.previous >= SUBMIT_TRACE_SLOW_SECONDS and not self.silent:
            say(f"  ! {SUBMIT_TRACE_TEXT}：{name}这一步用了 {now - self.previous:.1f} 秒（收到之后 {now - self.received:.1f} 秒），回话还没出去")
        self.previous = now

    def finish(self, outcome):
        """回话这一刻：记「回话」一步，打那一行；outcome 写这一条的结局（收下、拒了哪一种、出错）。"""
        self.step("回话")
        if self.silent:
            return
        received = time.strftime("%H:%M:%S", time.localtime(self.received_wall)) + f".{int(self.received_wall % 1 * 1000):03d}"
        steps = "、".join(f"{name} +{seconds:.3f}s" for name, seconds in self.steps)
        task = f"任务 {self.task_id}，" if self.task_id is not None else ""
        say(f"  · {SUBMIT_TRACE_TEXT}（{task}{outcome}）：{received} 收到；{steps}")


def append_private_line(path, line):
    lib.ensure_private_directory(os.path.dirname(path))
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    with os.fdopen(descriptor, "a", encoding="utf-8") as handle:
        handle.write(line.rstrip("\n") + "\n")


def nofile_property():
    """尝试单元的打开文件数上限：软、硬都取服务自己的硬上限（用户级 systemd 给单元的软上限是 1024，崩溃判定库开不了；
    records/2026-10-30-编译绕过内存总上限.md #16）。"""
    hard = resource.getrlimit(resource.RLIMIT_NOFILE)[1]
    value = "infinity" if hard == resource.RLIM_INFINITY else str(hard)
    return f"LimitNOFILE={value}:{value}"


def directory_bytes(path):
    """一个目录底下文件的字节数之和（task clean 报大小用；读不到的跳过）。"""
    total = 0
    for root, _, names in os.walk(path):
        for name in names:
            with contextlib.suppress(OSError):
                total += os.lstat(os.path.join(root, name)).st_size
    return total


def earlier_reading(kept, latest):
    """note_running_sample 并读数用：累计量挨得近的两回读数，留先到的（时刻与值对得上）。"""
    return kept


def cpu_reading(value):
    """累计 CPU 读数（Machine.cpu_usec）核成 float：是数（不是布尔）、有限、不小于 0 的交 float，别的（None、接口交来的别的东西）交 None，照读不到办。"""
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0:
        return None
    return float(value)


def read_json_file(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, ValueError):
        return None


def write_text_atomically(path, text):
    temporary = f"{path}.{os.getpid()}.partial"
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.replace(temporary, path)


def write_json_atomically(path, value):
    write_text_atomically(path, json.dumps(value, ensure_ascii=False))


def log_tail(path):
    try:
        with open(path, "rb") as handle:
            handle.seek(0, os.SEEK_END)
            handle.seek(max(0, handle.tell() - LOG_TAIL_BYTES))
            return handle.read().decode("utf-8", "replace")
    except OSError:
        return ""


def gpu_not_run_test(line, broken=()):
    """日志里的一行（bytes，带不带行尾都行）里用例「本次未跑」的标记：一行里每一处 GPU_NOT_RUN_MARK 整词各交一条、次序照出现的；
    每一条是这一处记号后面 test= 点名的用例（这一处没写 test= 的交 GPU_NOT_RUN_UNNAMED，行首的测试名不取）；一处都没有交 []。
    一处的范围是从这一处记号起到同一行下一处 NOT_RUN_ 记号之前（库的 marker_pieces），test= 只在这一截里找：同一行里两条用例各打各的记号，各点各的名。
    整词：前面是行首、空白或别的非标识符字符（标识符字符只算 ASCII 字母、数字、下划线，非 ASCII 的字不算），后面是空白（含制表符）或行尾；
    ANSI 颜色码先去掉再看（两侧照终端上看到的字算）。更长的词（`CANNOT_RUN_…` 里含的、`NOT_RUN_NO_GPU_ADAPTER:`、`NOT_RUN_NO_GPU_ADAPTER_X`）与更短的词不是；
    库认的 NOT_RUN_<原因> 一族里别的名字（NOT_RUN_OUT_OF_GPU_MEMORY 这一类）在这里只用来划截、不算一条。
    broken：这一回认的弄坏开关名（GPU_NOT_RUN_BREAK_SWITCHES，只给自证）：gpu-not-run-line-start-only 只认行首、gpu-not-run-word-edges-unchecked 不判两侧词界、
    gpu-not-run-color-codes-kept 不先去颜色码、gpu-not-run-first-mark-only 一行只算先出现的那一处。"""
    if b"NOT_RUN_" not in line and b"\x1b" not in line:
        return []
    text = line.decode("utf-8", "replace")
    colored = "gpu-not-run-color-codes-kept" in broken
    visible = text if colored else gate_marker_library.ANSI_ESCAPE_FORM.sub("", text)
    if "gpu-not-run-line-start-only" in broken and not visible.startswith("NOT_RUN_"):
        return []
    if "gpu-not-run-word-edges-unchecked" in broken:
        starts = [found.start() for found in GPU_NOT_RUN_MARK_FORM_WITHOUT_EDGES.finditer(visible)]
        pieces = [visible[start:end].rstrip() for start, end in zip(starts, starts[1:] + [len(visible)])]
    else:
        pieces = [piece for kind, piece in gate_marker_library.marker_pieces(text.replace("\x1b", "�") if colored else text)
                  if kind == "not-run" and GPU_NOT_RUN_WORD.match(piece).group() == GPU_NOT_RUN_MARK]
    tests = [named.group(1) if named else GPU_NOT_RUN_UNNAMED for named in (GPU_NOT_RUN_TEST.search(piece) for piece in pieces)]
    return tests[:1] if "gpu-not-run-first-mark-only" in broken else tests


def gpu_not_run_text(marks):
    """尝试行里记的「本次未跑」标记（NOT_RUN_MARKS_COLUMN 那一栏）写成给人看的一段：几条、逐条点名（至多 GPU_NOT_RUN_TESTS_SHOWN 条，余下只报条数）、判段没执行不算过。
    结局说明（Scheduler.finished_preface）与 gpu-cases-not-run 的依据都用这一段。"""
    count, tests = int(marks.get("count") or 0), [str(test) for test in marks.get("tests") or []]
    rest = count - len(tests)
    return (f"这一趟有 {count} 条用例本次未跑（日志里 {count} 处 {GPU_NOT_RUN_MARK} 记号）：{'、'.join(tests)}"
            f"{f'，另 {rest} 条只报条数' if rest > 0 else ''}；这几条的判段没执行、不算过")


def task_scope_text(record):
    """提醒依据里说这是哪一条：分片的一片写第几片（连同子任务号），别的子任务写角色与子任务号，别的写整条任务（long-run-split 与 gpu-cases-not-run 用）。"""
    if record.get("role") == "shard":
        return f"第 {record.get('shard_index')} 片（子任务 {record['id']}）"
    return f"{record.get('role')} 子任务 {record['id']}" if record.get("parent_id") else f"整条任务 {record['id']}"


# multi-host-run.sh 的 ⑥ 删了第二台这一趟的本地目录与 runs/ 下三个文件、回读过之后打的那一行（它的 multi_host_remove_peer_run）里的两段原文：
# 取消时在跑的第二台那一次照它认第二台那一份删了（Scheduler.cancelled_attempt_artifacts）
PEER_TRIP_REMOVED_MARK = "⑥ 删了第二台这一趟的本地目录"
PEER_TRIP_READ_BACK_MARK = "回读不在了"
# multi-host-run.sh 建好第二台这一趟的本地目录 builds/<标签>/ 之后才打的两行（起第二台那条命令那一步的开头、⑤ 开跑）：日志里有其中一行就是那一趟的本地目录建过了
PEER_TRIP_BUILT_MARKS = ("MULTI_HOST_RUN_STEP step=peer-command event=begin", "⑤ 第二台开跑")


def artifact_size_text(byte_count, host):
    """取消之后删产物那几行里的大小：量到的写多大（连同字节数），第二台本地盘上量不到的照写。"""
    if isinstance(byte_count, int):
        return f"{lib.size_text(byte_count)}（{byte_count} 字节）"
    return "量不到（第二台本地盘，服务这一侧看不到）" if host == PEER else "量不到"


def cancelled_artifact_lines(item):
    """取消之后删的一份产物（Scheduler.cancelled_attempt_artifacts 交的）在回话与日志里的行：删了的一行，没删成的「✗」一行接「→ 怎么办：」一行。"""
    where = f"任务 {item['task']} 第 {item['attempt']} 次尝试在{lib.HOST_NAMES.get(item['host'], item['host'])}的{item['kind']} {item['path']}"
    sizes = f"删前 {artifact_size_text(item['bytes_before'], item['host'])}"
    if item["gone"]:
        return [f"  · 取消之后删了{where}：{sizes}、删后 不在（{item['how']}）"]
    return [f"  ✗ 取消之后没删掉{where}：{sizes}、删后 {artifact_size_text(item['bytes_after'], item['host'])}：{item['why']}",
            f"  → 怎么办：{item['howto']}"]


def append_log_lines(path, lines):
    """往一次尝试的日志末尾接几行（取消之后删产物、提交时警告的日志头）；日志路径空着或写不进不报错（回话里照样带这几行）。"""
    if not path:
        return
    with contextlib.suppress(OSError):
        with open(path, "a", encoding="utf-8") as handle:
            handle.write("".join(line + "\n" for line in lines))


def post_to_inbox(socket_path, token, text, environment):
    """往会话的收件 socket 发一条消息（文件头 ⑥）：先发认证行 {"type":"auth","token":…}，再发一行 {"type":"user","message":{"role":"user","content":<正文>}}。
    两行先备好再连、一次写完，写完关掉写的那一半，等对面收完关连接至多 INBOX_DRAIN_SECONDS。→ 发不出去的原因（socket 不在、连不上、写不进），发出去了交空串。
    行的写法取自 Claude Code 二进制自己的调试日志（task-server/README.md「## 发消息给会话」），文档页没写，升级 Claude Code 之后要复核。
    弄坏开关 inbox-auth-omitted 下不发认证行。"""
    lines = [] if lib.broken("inbox-auth-omitted", environment) else [json.dumps({"type": "auth", "token": token}, ensure_ascii=False)]
    lines.append(json.dumps({"type": "user", "message": {"role": "user", "content": text}}, ensure_ascii=False))
    payload = "".join(line + "\n" for line in lines).encode("utf-8")
    connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        connection.settimeout(INBOX_SEND_SECONDS)
        connection.connect(socket_path)
        connection.sendall(payload)
        connection.shutdown(socket.SHUT_WR)
        connection.settimeout(INBOX_DRAIN_SECONDS)
        with contextlib.suppress(OSError):
            while connection.recv(4096):
                pass
    except OSError as error:
        return f"{type(error).__name__}: {error}"
    finally:
        connection.close()
    return ""


def own_unit_state(environment):
    """服务自己那个用户级单元 is-active 打的状态（照输出判，同 control 的 unit_active_state；自证里 TASK_SERVER_CONTROL_SYSTEMCTL 换成假的）。"""
    systemctl = lib.selftest_override("TASK_SERVER_CONTROL_SYSTEMCTL", environment) or "systemctl"
    try:
        completed = subprocess.run([systemctl, "--user", "is-active", lib.UNIT_NAME], capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL, timeout=30)
    except (OSError, subprocess.TimeoutExpired) as error:
        return f"读不到（{type(error).__name__}: {error}）"
    lines = completed.stdout.strip().splitlines()
    return lines[-1].strip() if lines else "（没打状态）"


INBOX_HEADLINE = "task-server："   # 每条消息正文第一行的开头（文件头 ⑥）


def push_round_tail(round_number, previous_at):
    """首行括号里最后一段的「；第 N 回」；重推的（N 大于 1）另写「，上一回在 <日期>」（previous_at 是上一回推出去的时刻，读不到就不写日期）。
    弄坏开关 inbox-headline-round-omitted 下交空串。"""
    if lib.broken("inbox-headline-round-omitted"):
        return ""
    tail = f"；第 {round_number} 回"
    if round_number > 1 and previous_at:
        tail += f"，上一回在 {time.strftime('%Y-%m-%d', time.localtime(float(previous_at)))}"
    return tail


def task_state_phrase(task):
    """推给会话的消息首行里任务此刻的状态：已结束、还在排（没进队的也算）、还在跑；任务记录里没有状态的当还在跑。"""
    state = (task or {}).get("state")
    if state == "finished":
        return "任务已结束"
    if state in ("queued", lib.AWAITING_CONFIRMATION_STATE):
        return "任务还在排"
    return "任务还在跑"


def kind_label(kind):
    """首行里一个种类的中文名（括号里的英文种类名由调用方接）；没登记的退回英文种类名（自证 headline-names-cover-kinds 核登记的种类一个不缺）。
    弄坏开关 inbox-headline-name-omitted 下只写英文种类名。"""
    return kind if lib.broken("inbox-headline-name-omitted") else lib.KIND_NAMES.get(kind, kind)


def headline_class(category, state):
    """首行开头的类别与任务此刻的状态：「结局（任务已结束）」「提醒（任务还在跑，不是结局）」「疑似卡住（任务还在跑、服务没停它）」；弄坏开关 inbox-headline-state-omitted 下不写这一段。"""
    return "" if lib.broken("inbox-headline-state-omitted") else f"{category}（{state}）"


def outcome_headline(task_id, outcome, exit_code, round_number=1, previous_at=None, unclaimed=False):
    """结局那一条的首行：先写类别「结局」与任务此刻的状态（已结束），再写任务号、中文名（英文种类名，退出码；第 N 回）；
    结局没人取而再推的（unclaimed）在状态里另写这条结局还没人取。"""
    state = "任务已结束，这条结局还没人取" if unclaimed else "任务已结束"
    return f"{INBOX_HEADLINE}{headline_class('结局', state)}任务 {task_id} {kind_label(outcome)}（{outcome}，退 {exit_code}{push_round_tail(round_number, previous_at)}）"


def reminder_headline(task, item, round_number, previous_at, tail="", subject=None):
    """要处置的提醒那一条的首行：先写类别「提醒」与任务此刻的状态（还在排 / 还在跑，不是结局；任务其实已结束的写结局另有一条），再写任务号、中文名（英文种类名<tail>；第 N 回）。
    tail 是种类后面接的（没处置、第几回再推，子任务号，片数）。状态照提醒说的那一条记录（subject，说的是子任务的是子任务那一行）：分片的父任务一直是 sharded，
    片都在排的提醒写还在排；subject 不给的照 task。"""
    state = task_state_phrase(subject or task)
    note = "，结局另有一条" if state == "任务已结束" else "，不是结局"
    return (f"{INBOX_HEADLINE}{headline_class('提醒', state + note)}任务 {task['id']} {kind_label(item['kind'])}"
            f"（{item['kind']}{tail}{push_round_tail(round_number, previous_at)}）")


def suspect_headline(task, segment, part=""):
    """疑似卡住那一条的首行：类别「疑似卡住」与任务此刻的状态（还在跑、服务没停它），再写任务号、中文名（英文种类名，第 N 段；第 1 回：一段只推一回）。"""
    return (f"{INBOX_HEADLINE}{headline_class('疑似卡住', '任务还在跑、服务没停它')}任务 {task['id']} {kind_label('suspected-stall')}"
            f"（suspected-stall，第 {segment} 段{part}{push_round_tail(1, None)}）")


GPU_CARD_FAILURE_CHECK_SECONDS = 60   # 服务健康至多隔这么久读一回 GPU 服务的卡坏（gpu_card_failures；cards-failed 经 ssh 问第二台，不每一轮起）
GPU_CARDS_FAILED_READ_SECONDS = 150   # 起一回 gpu-verification-server/control cards-failed 的时限：它问第二台至多等 60 秒、读日志至多 120 秒
GPU_CARD_FAILED_NEXT_STEP = ("主 agent 当场弹窗请用户重置那张卡（问题里写着哪一台、nvidia-smi 的序号与 PCI 总线号），不等、不重试"
                             "（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）；重置好了 python3 gpu-verification-server/control stop、"
                             "python3 gpu-verification-server/control start 让服务重新开卡；python3 gpu-verification-server/control check 列着卡坏那几行")   # GPU 服务卡坏那一条的出路
GPU_BLOCK_LET_GO_NEXT_STEP = ("放掉了认领的 {count} 块：另一台开着 GPU 服务的取得到、会去试；只开了一台的，留在库里、等这一台的 GPU 服务重起之后再试一回"
                              "（判不了行的 claim= 是 released 或 not-held）")   # 判不了的块那一条的出路里放了认领的那一句（gpu_block_not_judgeable_next_step）
GPU_BLOCK_KEPT_NEXT_STEP = ("没放认领的 {count} 块（claim=kept：放的时候读写判定库失败）：留在库里，等认领租期过了才有服务取得到")   # 同上，没放认领的那一句
GPU_BLOCK_NOT_JUDGEABLE_NEXT_STEP_TAIL = ("这一台的 GPU 服务判不了这几块（在腾空的卡上也显存不够，或判法自己 panic），不退回 CPU；等这几块的任务照等；"
                                          "GPU 服务重起之后照常再试一回，同一条流再判不了不重发这一条。两台都报了同一条流的，主 agent 照问题里的族、流键头与块号查那条流"
                                          "（服务日志里那几块的 SERVING_PROBLEM 行与 panic 的那一句），交用户定怎么处置；卡照常用，不要重置卡；"
                                          "python3 gpu-verification-server/control check 列着这几行")   # 判不了的块那一条的出路，接在照认领写的那几句后面
GPU_LINES_UNPARSED_NEXT_STEP = ("gpu-verification-server/control cards-failed 打的卡坏行或判不了的行里，任务服务认的栏（task-server/task_server_lib.py 的 GPU_CARD_FIELD、"
                                "GPU_BLOCK_NOT_JUDGEABLE_KEY_FIELDS、GPU_BLOCK_CLAIM_FIELD）取不到，或认领的值不在 GPU_BLOCK_CLAIMS_LET_GO、GPU_BLOCK_CLAIMS_KEPT 里："
                                "两边的字段对不上，交维护 GPU 服务与任务服务的会话对齐；"
                                "这期间那几行代表的卡坏或判不了照 python3 gpu-verification-server/control check 看")   # GPU 服务的卡坏行认不出那一条的出路
GPU_LINES_UNPARSED_KIND = "gpu-service-lines-unparsed"   # 服务健康的种类：cards-failed 里有认不出的卡坏行或判不了的行（gpu_card_failures）
GPU_HEALTH_KIND_PREFIXES = (f"{lib.GPU_CARD_FAILED_ALERT_KIND}:", f"{lib.GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND}:", GPU_LINES_UNPARSED_KIND)   # 读 GPU 服务得来的服务健康种类的开头（watch_health）


def gpu_block_not_judgeable_next_step(let_go, kept):
    """判不了的块那一条的出路（task_server.py 的 gpu_card_failures）：这条流里放了认领的几块、没放的几块各写一句（GPU_BLOCK_LET_GO_NEXT_STEP、
    GPU_BLOCK_KEPT_NEXT_STEP，块数是 0 的那一句不写），再接 GPU_BLOCK_NOT_JUDGEABLE_NEXT_STEP_TAIL。"""
    said = [text.format(count=count) for text, count in ((GPU_BLOCK_LET_GO_NEXT_STEP, let_go), (GPU_BLOCK_KEPT_NEXT_STEP, kept)) if count]
    return "；".join(said) + "。" + GPU_BLOCK_NOT_JUDGEABLE_NEXT_STEP_TAIL
HEALTH_NEXT_STEP = (f"python3 task-server/control check 看卡在哪；起不来看 journalctl --user -u {lib.UNIT_NAME} -n 100；NFS 共享与第二台挂载照 {lib.NFS_README} 修，"
                    "第二台用不了照上面那一句修（连法、双机开关）；在跑的任务在各自的单元里接着跑，修好之后服务对回它们")   # 服务健康那一条的出路


# ── 库 ──
@contextlib.contextmanager
def keyed_lock(table, guard, key, keep=False):
    """拿着锁表 table（{键: [锁, 拿着与在等的个数]}，改它要拿 guard）里 key 那一把跑 with 里那一段（Store.task_lock 与 confirm_peer_cleanup 用）：
    拿着与在等的都算一个；最后一个放下时把它从表里删掉（在等的那几个手里是同一把，删之前没有人能另建一把顶替它），表的大小不随用过的键数涨。keep 为真时放下也不删。"""
    with guard:
        entry = table.setdefault(key, [threading.Lock(), 0])
        entry[1] += 1
    try:
        with entry[0]:
            yield
    finally:
        with guard:
            entry[1] -= 1
            if entry[1] == 0 and not keep:
                del table[key]


def association_id(value):
    """正文里的关联号（task_id、parent_id）：是正整数的交它，别的（0、空、不是整数的）交 0。校验不过的行判不出归哪一件时照 0 写，不挂在任何一件名下。"""
    return value if isinstance(value, int) and not isinstance(value, bool) and value > 0 else 0


# ── 两台的余量 ──
LOCAL, PEER = "local", "peer"   # 服务记尝试的视角（task_server_lib.HOST_NAMES）：本机是服务所在这一台，第二台是多机配置 HOSTS 里另一台
