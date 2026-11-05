// admission: always 服务常驻、每次起都是一个新进程；自证每次现起真服务现判，上一次的结论不替这一次作保
// run-condition: command python3
//! prune-server：判定库服务（records/2026-11-01-判定库服务化.md 批次 2 的原型，批次 3 加了容错与组提交）。
//!
//! 用法：
//!   prune-server serve [--socket <路径>] [--state-directory <目录>] [--library <目录>，只给自证与压测指临时库]
//!       [--tcp <地址:端口>] [--multi-host-configuration <文件>] [--worker-threads <数>]
//!       [--group-commit-milliseconds <数，缺省 2>] [--group-commit-frames <数，缺省 256>]
//!       [--maximum-pending-per-connection <数，缺省 512>] [--maximum-pending-total <数，缺省 8192>]
//!       [--maximum-pending-bytes <数，缺省 1073741824>] [--maximum-pending-bytes-per-connection <数，缺省 268435456>]
//!       [--maximum-connections <数，缺省 1024>] [--frame-read-timeout-milliseconds <数，缺省 60000>]
//!       [--write-timeout-milliseconds <数，缺省 60000>] [--busy-retry-milliseconds <数，缺省 20>]
//!       socket 缺省 `${XDG_RUNTIME_DIR}/singlefs-prune-server/server.sock`（目录 700、socket 600、全路径至多 107 字节）；
//!       库是服务自己那一份，缺省 `~/.local/state/singlefs/prune-server/library`，不在就建；布局只有一种，与崩溃判定库相同：只有默认列族，
//!       三张表都落在它里面、键原样，两张记账表也在里面、各一个 0xF0 起的键前缀（`src/storage.rs` 文件头）；TCP 地址缺省取仓根 `multi-host.env` 的 `PRUNE_SERVER_TCP_ADDRESS`
//!       （`SINGLEFS_MULTI_HOST_CONFIG` 设了读它指的那一份）；工作线程缺省本机核数。
//!       组提交：写与认领在 `--group-commit-milliseconds` 的窗口里至多收 `--group-commit-frames` 帧，合成一个 WriteBatch、同步一次 WAL，
//!       同步之后才回各自的确认；同步 WAL 失败时回存储出错、服务退 7，重起时重开库（`src/storage.rs` 文件头）。
//!       准入：每条连接与整个服务的待处理帧数、待处理字节数到了上限，当场回「忙、`--busy-retry-milliseconds` 毫秒后再来」；
//!       待处理字节数里记着收着的帧体与没写出去的回应，一帧在期限里收齐、回应写堵住到期限关连接、连接数有上限（`src/server.rs` 文件头）。
//!       状态目录缺省 `~/.local/state/singlefs/prune-server`，库缺省是它下面的 `library`，故障日志是它下面的 `faults.log`（`src/fault_log.rs` 文件头）。
//!       就绪时标准输出打一行 `PRUNE_SERVER_LISTENING unix=… tcp=… library=… faults=… pid=… worker_threads=…`，之后一直跑，SIGTERM 立刻结束。
//!   prune-server admin health|connected-clients|announce-stop [--socket <路径>]   控制命令用（`src/administration.rs` 文件头）
//!   prune-server drop-oversized-values [--state-directory <目录>] [--dry-run]
//!       离线维护，服务停着时在服务那台跑（`control stop` 之后、`control start` 之前）：直接开状态目录下的 library（只开已有的、不建），
//!       扫默认列族，认出键不是段键、计数键形状而值一把键单独一帧点读也回不了的整值行（界与服务整帧拒同一份判法，`src/oversized_values.rs` 文件头）；
//!       每行打 `PRUNE_SERVER_OVERSIZED_VALUE table=<0x表标签或族名> value_bytes=<数> key=<键，非 ASCII 转义>`，
//!       末行 `PRUNE_SERVER_DROP_OVERSIZED_VALUES mode=dry-run|dropped library=… scanned_rows=… oversized_rows=… longest_readable_value_bytes=…`。
//!       `--dry-run` 只打清单、库不动；不带它把清单里的行在一个写批里删掉、同步 WAL、逐把点读核没了。删掉的判定由各入口下次经服务现判、分段写回。
//!       退出码：0 成了；1 删完点读还读得到；2 用法错；3 库打不开（服务开着它、目录不在）或读写出错；6 库对不上；8 文件数上限不够；78 准入不满足。
//!   prune-server --selftest                   自证（每格另起一个进程只跑它，每格起一份真服务挂在临时 socket 上，TCP 走回环）；
//!                                             `PRUNE_SERVER_SELFTEST_CELLS=<格,…>` 只跑点名的格；设了 `PRUNE_SERVER_SELFTEST_WRAPPER_COMPILER=<C 编译器>`
//!                                             时 write-ahead-log-sync-failure-never-acknowledged 多跑真实 fdatasync 失败那条输入（不设时那一格的说明写本次未跑）
//!   prune-server --selftest-break-sweep       不设开关跑一遍自证，再逐项打开下面每个弄坏开关各跑一遍，核红的恰好是点名的那一格
//!   prune-server bench <压测参数>             同 prune-server-bench
//!   prune-server claim-holder --selftest-instance …   只给自证：认领一批键后一直睡，等自证 kill -9 它
//!   prune-server selftest-client --selftest-instance …  只给自证：报到之后一直写，出错时照容错客户端的退出码退出
//!   prune-server selftest-cell --selftest-instance --runner-process <进程号> <格名>   只给自证：在这个进程里只跑这一格，
//!       末行 `PRUNE_SERVER_CELL_RESULT result=passed|failed`（`src/selftest.rs` 文件头「每格各在自己的进程里」）
//! 容错客户端的退出码（`src/reliable_client.rs` 文件头）：75 prune-server 不可用（服务停了、连不上、忙到上限），3 别的错。
//!
//! 协议：长度前缀的二进制帧，一帧一批操作（成批点读走 multi_get、成批写带客户端身份与请求号、按前缀扫、统计、认领），
//! 可流水，回应带请求序号、可能乱序，忙时回状态 5 带「多少毫秒后再来」；格式在 `src/protocol.rs` 文件头。
//! 表「录入、事实、结论」是协议里的分表码，库里同在默认列族、同一个键空间；0xF0–0xFF 起头的键留给服务的记账，客户端碰了回状态 1。
//! 容错客户端（追加日志、重连重放、帧超时、keepalive、退避加抖动、窗口、忙退避）在 `src/reliable_client.rs`。
//!
//! 服务的退出码：1 监听停了；2 用法错（含认不得的参数，例 `--library-layout`）；3 库打不开（被别的进程持有、目录建不了）；4 socket 不能用（已有服务回话、路径太长、目录不是 700）；
//! 5 TCP 不能用（多机配置里没有地址、绑不上）；6 库对不上（目录里有文件却不是 RocksDB、另有列族、RocksDB 因选项对不上拒开）；
//! 7 同步 WAL 失败、服务退出（重起时从 WAL 重开库，`src/storage.rs` 文件头）；
//! 8 文件数上限不够（开库之前 RLIMIT_NOFILE 的软上限抬到硬上限，仍小于库目录的文件数加开着的描述符加余量；标准错误写
//! 「文件数上限不够：要 N、硬上限 M」与出路，`src/storage.rs` 文件头「文件数上限」）；78 准入或运行条件不满足。
//! 自证的退出码：0 全绿；1 有红；2 格名或开关写错。复跑弄坏开关的退出码：0 每项都对上、不设开关那一遍全绿；1 有对不上的。
//!
//! 自证的格（`src/selftest.rs` 的 `CELLS`，次序相同）：
//!   batched-point-read                 一帧 103 个键（3 个不在）读回逐个对上，RocksDB 数到 multi_get 恰好 1 次、读 103 个键
//!   prefix-scan                        前缀扫只回前缀下的键、按升序、到条数上限标截断、用别的表的名义扫回同样的键
//!   tcp-route-from-configuration       TCP 地址取自多机配置的键，经 TCP 三种操作与统计都对
//!   missing-tcp-address-refused        多机配置里没有那个键时服务退 5、点名这个键
//!   pipelined-out-of-order             先发的帧被服务压后，回应乱序到达，客户端按序号认领到各自的值
//!   frame-length-rejected              声称 4 GiB 与短于帧头的帧都回序号 0 的帧长度拒绝、随后关连接
//!   unknown-operation-rejected         认不得的操作码回状态 2，连接接着能用
//!   malformed-payload-rejected         个数对不上、帧尾多字节、认不得的表都回状态 1，一条都不写
//!   single-library-holder              同一个库目录第二个服务打不开、退 3，头一个照常回话（拒的是 RocksDB 自己的 LOCK，没有弄坏开关）
//!   bench-rate-judgment                压测达标判法（95%）与百分位（最近秩法）的已知答案
//!   bench-runs-every-mode              六种负载小速率都跑通（退 0、1 或 4，末行 result 与退出码一致）、末行字段齐、fault-injection 带 faults=3、
//!                                      压测库删掉；不判达标、不设会被同机负载打穿的限时（上限 900 秒，超了打 loadavg）（没有弄坏开关）
//!   bench-interrupted-removes-library  压测器收 SIGTERM 停下、删压测库、停掉它起的服务、退 3
//!   fault-injection-bench-finishes     带故障的压测（每秒 2 万态 6 秒、服务每组提交等满 200 毫秒、追加日志整理门槛 64 KiB，没确认的帧
//!                                      每一趟都超过门槛）十分钟内出末行（超了打 loadavg 与最后一行进度，判了卡住带各线程等在哪）、faults=3、
//!                                      追加日志整理重写的字节至多是追加字节的两倍加门槛、进度行带阶段与客户端停在哪一步
//!   replay-after-disconnect-and-server-kill  不丢：经代理断在一帧中间、吞掉回应再断各一次，40 帧逐键对上；确认过的 30 帧 kill -9 后一个不少；
//!                                      写到一半 kill -9 再重起，60 帧逐键对上；每帧改写的键都是最后一帧
//!   delay-jitter-duplicate-tolerated   网络波动：两个方向每帧延迟 0 到 15 毫秒、迟到的帧重复，结果不变（逐帧改写的键是最后一帧）、不卡死
//!   half-open-connection-recovered     网络波动：TCP keepalive 在内核里开着；连接半开（不转也不关）时帧超时、重连、全部落库
//!   server-kill-while-client-writes    经代理连着的服务 kill -9 再重起之后客户端接着写两帧 4 MiB：看到连接断了、重连、两帧逐字节落库，
//!                                      不堵在写里（限时 120 秒）
//!   reconnect-gives-up-at-limit        网络波动：服务停了，写与读各重连到次数上限回退 78 那一类的错（读不当成键不在），间隔翻倍、有抖动；
//!                                      重开客户端从追加日志重放那一帧
//!   busy-backpressure-bounded          防雪崩：服务变慢（自证专用的提交延迟）时客户端收到忙、照毫秒数退避、全部完成，待处理帧数峰值不超上限
//!   claim-exclusive-and-lease-expiry   防击穿：两个客户端同时认领同一批键每把恰好一个认领到；写了结论的回已有结论；持有者 kill -9 后租期过了别人认领到
//!   group-commit-coalesces-syncs       一帧 100 条只同步 1 次；8 条连接并发 128 帧，RocksDB 数到同步 WAL 至多 32 次
//!   bench-environment-judgment         压测环境判法（loadavg 超阈值不干净）、loadavg 与 diskstats 解析、忙度换算的已知答案
//!   control-install-start-stop-check   control 在假 systemctl 下：install --dry-run 不写不起；服务不通时 install 退 1、通时退 0、单元 ExecStart 不带 --library；
//!                                      start 回读 is-active 打 failed 退 1；check 退 0、打出状态目录下的库目录；stop 回读 cgroup.procs 有进程退 1、正常退 0
//!   stop-lists-connected-clients       两个带不同任务号的自证客户端连着时 control stop：清单两行字段对、停服务记录排他写了一份
//!   intentional-stop-fails-clients-fast 宣布停服务再 SIGTERM：客户端 15 秒内退 75、说「服务停了」
//!   unannounced-server-death-bounded   服务 kill -9 不重起：客户端在重连上限（6 秒）里退 75、说「连不上」
//!   fault-line-on-server-error         自证造的存储写失败：故障日志新增一行，种类与报到身份的字段齐
//!   watch-reports-unit-not-running     control watch：什么都没出时不退；单元不在跑时退 1
//!   watch-reports-server-unreachable   control watch：服务不通时退 1、客户端清单写拿不到
//!   watch-reports-new-fault-line       control watch：起之前的故障行不算；新增一行退 1，打故障行原文、客户端、task status 查法
//!   client-write-timeout-when-peer-stops-reading  代理一个字节都不读时，写帧在写超时处出错、重连重放落库
//!   tcp-proxy-server-kill-client-fails 经 TCP 代理 kill -9 服务：客户端的写出错、到重连上限回「连不上」，不堵在写里
//!   single-library-layout              不带 --library 开的是状态目录下的 library；三张表的名义互读同一行、认领不分表、空前缀扫只回客户端写的行、
//!                                      写 0xF0 起头的键回状态 1；停服务直接开库：只有 default，行是写的 3 条、请求号 1 行（0xF0）、认领 1 行（0xF1）
//!   library-refuses-mismatched         库里另有列族、比较器不同、有文件没有 CURRENT 的目录各退 6 点名原因（列族名、「选项与库对不上」、CURRENT），
//!                                      拒起后库不变；库被另一个服务开着退 3
//!   library-opens-under-low-descriptor-soft-limit  样本库 300 个 SST（文件数多于 128）：服务子进程软硬上限都压到 128 时退 8、
//!                                      标准错误写「文件数上限不够：要 …、硬上限 128」与出路；只把软上限压到 128、硬上限照跑格的进程时
//!                                      开得了库、就绪后 /proc 里的软上限是硬上限、点读最后一个 SST 里的行对上
//!   delete-prefix-only-under-prefix    表标签 0x1C 起头的 5 把键每帧至多 2 把分三帧删（2、2、1，续扫起点取上一帧的最后一把键，最后一帧说没有更多了），
//!                                      前后别的前缀（0x1B、0x1D、ASCII 族名起头）的键一把不少；再删回 0 把；空前缀与记账那一段的前缀被拒、库不动
//!   delete-prefix-counts-before-deleting  前缀下 5 把键每帧至多 2 把只数不删分三帧数到 5、库里一把键不少；先数后删：数到 5、删掉 5、删完再数 0，前缀之外的键还在
//!   journal-high-water-survives-compaction  追加日志整理门槛 1 字节：全确认之后整理、重开再写，与「请求号 1 没确认、2 确认了」之后整理、重开再写，
//!                                      新写都分到没用过的请求号、服务回的不是「早已生效」、新键读得到
//!   write-ahead-log-sync-failure-never-acknowledged  内建造错点造的同步 WAL 失败：写回存储出错、服务退 7；故障没解除时重起、重发同一请求
//!                                      照回存储出错、再退 7；同步恢复、重起之后重发才回做完、读得到，再重发回「早已生效」；
//!                                      设了 PRUNE_SERVER_SELFTEST_WRAPPER_COMPILER 时另跑真实 fdatasync 失败（LD_PRELOAD 包装）：写回存储出错、服务退 7、
//!                                      失败行是包装打的，撤掉之后重起重发回做完、读得到，再重发回「早已生效」
//!   unread-responses-bounded           每条连接额度 4 MiB：小请求读 1 MiB 的值、只发不收 64 次，写出队峰值不超额度加一个回应加不带名额的回应上限、
//!                                      别的客户端照常读写；写期限 3 秒到了慢连接被关、额度放回
//!   incomplete-frames-bounded          额度 1024 字节、待处理 1 帧：四条连接各声明 16 MiB 只发 2 MiB，给没收完的帧体分配的峰值不超额度加一帧；
//!                                      半帧与逐字节慢发的连接在 1.5 秒的帧读取期限后被关；连接数上限 8 时第九条当场关掉；之后照常读写
//!   journal-lock-survives-compaction   第二个打开者卡在开文件与加锁之间、第一个整理（改名换上新文件）之后放行：被同一把锁挡住，原路径重开读回的没确认写帧不多
//!   claim-response-within-frame-limit  255 字节身份的「别人持有」一帧至多 253240 个（已知答案）；混合认领照常；26 万个短键查别人持有的认领整帧回状态 1、
//!                                      说明写超过帧长上限，同一帧里的空闲键没被认领
//!   connect-bounded-by-reconnect-total 重连总时长上限 600 毫秒：unix 连接队列满、TCP 握手不完成、连上之后不读不回各在上限加 2 秒里回「连不上」；
//!                                      连接队列满时调用方要停，2 秒里回「调用方要停」
//!   oversize-frame-reported-at-once    键长超过帧长上限的一条：容错客户端在重连总时长 2 秒之内回协议错、说帧长上限、追加日志里没有这一帧；
//!                                      一问一答的客户端当场回错、说没发，同一条连接随后照常读写
//!   cell-runs-alone-in-own-process     这一格在跑格的进程为它另起的进程里单独跑：父进程是跑格的进程（别的格起子进程复制不到它的描述符）
//!
//! 弄坏开关 PRUNE_SERVER_BREAK=<项>（逗号分隔；只在自证起的服务与压测器带 --selftest-instance 时认，自证进程自己读它弄坏客户端与判法）。
//! 逐项写弄坏哪一处、打开后自证哪一格红（「✗ 自证 <格名>：」那一行）；表与 `src/break_switches.rs` 的 `red_cell` 逐项相同：
//!   PRUNE_SERVER_BREAK=multi-get-per-key                          成批点读退回逐键 get                           batched-point-read
//!   PRUNE_SERVER_BREAK=write-sync-per-entry                       成批写每条各自一个同步写批                     group-commit-coalesces-syncs
//!   PRUNE_SERVER_BREAK=prefix-scan-overruns                       前缀扫走出前缀不停                             prefix-scan
//!   PRUNE_SERVER_BREAK=missing-tcp-address-defaults-to-loopback   多机配置没有 TCP 地址时退回回环、不拒          missing-tcp-address-refused
//!   PRUNE_SERVER_BREAK=response-sequence-unchecked                客户端不核回应序号、按到达先后配给最早的请求   pipelined-out-of-order
//!   PRUNE_SERVER_BREAK=frame-length-unchecked                     服务不判帧长度、照声称的长度读下去             frame-length-rejected
//!   PRUNE_SERVER_BREAK=unknown-operation-accepted                 认不得的操作码回空的成功                       unknown-operation-rejected
//!   PRUNE_SERVER_BREAK=trailing-bytes-accepted                    解帧不判帧尾多出来的字节                       malformed-payload-rejected
//!   PRUNE_SERVER_BREAK=bench-judgment-lenient                     压测达标线从 95% 放到 50%                      bench-rate-judgment
//!   PRUNE_SERVER_BREAK=bench-library-kept-on-interrupt            压测器被打断时不删压测库                       bench-interrupted-removes-library
//!   PRUNE_SERVER_BREAK=unacknowledged-frames-dropped-on-reconnect 对端关连接或读写出错后不重放在途的写帧         replay-after-disconnect-and-server-kill
//!   PRUNE_SERVER_BREAK=acknowledge-before-sync                    服务先回写确认、过 2 秒才同步 WAL              replay-after-disconnect-and-server-kill
//!   PRUNE_SERVER_BREAK=duplicate-requests-reapplied               服务不按（身份，请求号）去重                   delay-jitter-duplicate-tolerated
//!   PRUNE_SERVER_BREAK=duplicate-responses-fatal                  客户端把重复到达的回应当成协议错               delay-jitter-duplicate-tolerated
//!   PRUNE_SERVER_BREAK=frame-timeout-ignored                      客户端不给在途帧设超时                         half-open-connection-recovered
//!   PRUNE_SERVER_BREAK=keepalive-not-set                          客户端的 TCP 连接不开 keepalive                half-open-connection-recovered
//!   PRUNE_SERVER_BREAK=reconnect-backoff-flat                     重连间隔不翻倍                                 reconnect-gives-up-at-limit
//!   PRUNE_SERVER_BREAK=reconnect-without-jitter                   重连间隔不加抖动                               reconnect-gives-up-at-limit
//!   PRUNE_SERVER_BREAK=reconnect-unbounded                        重连没有次数与总时长上限                       reconnect-gives-up-at-limit
//!   PRUNE_SERVER_BREAK=read-failure-reported-as-absent            读失败时回「键都不在」                         reconnect-gives-up-at-limit
//!   PRUNE_SERVER_BREAK=unacknowledged-frames-not-journaled        客户端不把写帧记进追加日志                     reconnect-gives-up-at-limit
//!   PRUNE_SERVER_BREAK=busy-replaced-by-unbounded-queue           服务不设待处理上限、不回忙                     busy-backpressure-bounded
//!   PRUNE_SERVER_BREAK=busy-retry-without-delay                   客户端收到忙立刻重发                           busy-backpressure-bounded
//!   PRUNE_SERVER_BREAK=claim-not-exclusive                        认领不判别人持有                               claim-exclusive-and-lease-expiry
//!   PRUNE_SERVER_BREAK=claim-lease-never-expires                  认领的租期到了也不作废                         claim-exclusive-and-lease-expiry
//!   PRUNE_SERVER_BREAK=claim-kept-after-conclusion                写结论不释放认领、认领不看结论                 claim-exclusive-and-lease-expiry
//!   PRUNE_SERVER_BREAK=group-commit-disabled                      一帧一次 WAL 同步                              group-commit-coalesces-syncs
//!   PRUNE_SERVER_BREAK=bench-load-threshold-ignored               压测器不看 loadavg                             bench-environment-judgment
//!   PRUNE_SERVER_BREAK=journal-compacts-on-every-acknowledgement  追加日志一过门槛就整理、不随整理完的长度翻倍   fault-injection-bench-finishes
//!   PRUNE_SERVER_BREAK=fault-proxy-half-closes-on-server-death    代理在服务那头没了时只半关客户端那头的写       server-kill-while-client-writes
//!   PRUNE_SERVER_BREAK=control-dry-run-writes                     install --dry-run 照样写文件、起 systemctl       control-install-start-stop-check
//!   PRUNE_SERVER_BREAK=control-install-readback-skipped           install 不回读服务通不通                         control-install-start-stop-check
//!   PRUNE_SERVER_BREAK=control-start-readback-skipped             start 不回读 is-active 与服务通不通              control-install-start-stop-check
//!   PRUNE_SERVER_BREAK=control-stop-readback-skipped              stop 只看 systemctl 的退出码                     control-install-start-stop-check
//!   PRUNE_SERVER_BREAK=client-announcement-omits-task             报到时任务号报空                                 stop-lists-connected-clients
//!   PRUNE_SERVER_BREAK=announcement-ignored                       服务不把报到记进登记                             stop-lists-connected-clients
//!   PRUNE_SERVER_BREAK=control-stop-record-skipped                stop 不写停服务记录                              stop-lists-connected-clients
//!   PRUNE_SERVER_BREAK=stop-notice-not-sent                       宣布停服务不给客户端发「服务停了」               intentional-stop-fails-clients-fast
//!   PRUNE_SERVER_BREAK=stop-notice-retried                        客户端收到「服务停了」照常重连                   intentional-stop-fails-clients-fast
//!   PRUNE_SERVER_BREAK=unreachable-exit-code-generic              「连不上」退 3 而不是 75                         unannounced-server-death-bounded
//!   PRUNE_SERVER_BREAK=fault-line-not-written                     服务出错不写故障行                               fault-line-on-server-error
//!   PRUNE_SERVER_BREAK=watch-ignores-unit-state                   watch 不看单元在不在跑                           watch-reports-unit-not-running
//!   PRUNE_SERVER_BREAK=watch-ignores-health                       watch 不看服务通不通                             watch-reports-server-unreachable
//!   PRUNE_SERVER_BREAK=watch-ignores-fault-log                    watch 不看故障日志                               watch-reports-new-fault-line
//!   PRUNE_SERVER_BREAK=watch-counts-old-fault-lines               watch 把起之前的故障行也当新的                   watch-reports-new-fault-line
//!   PRUNE_SERVER_BREAK=client-write-timeout-unset                 容错客户端不设写超时                             client-write-timeout-when-peer-stops-reading
//!   PRUNE_SERVER_BREAK=fault-proxy-only-stops-reading-on-server-death 代理在服务没了时只关客户端那头的读、fd 留着  tcp-proxy-server-kill-client-fails
//!   PRUNE_SERVER_BREAK=bookkeeping-key-tag-dropped                请求号行不带键前缀 F0、落进客户端的键空间        single-library-layout
//!   PRUNE_SERVER_BREAK=service-keys-open-to-clients               客户端碰 0xF0 起头的键不拒、前缀扫走进记账那一段不停  single-library-layout
//!   PRUNE_SERVER_BREAK=library-extra-column-families-accepted     开库不判另有列族、一起打开                       library-refuses-mismatched
//!   PRUNE_SERVER_BREAK=library-open-failure-reported-as-locked    RocksDB 拒开的每种错都报成库被持有（退 3）       library-refuses-mismatched
//!   PRUNE_SERVER_BREAK=library-without-current-file-opened        有文件没有 CURRENT 的目录不拒、在里面建库        library-refuses-mismatched
//!   PRUNE_SERVER_BREAK=control-check-omits-library                check 不打库目录                                 control-install-start-stop-check
//!   PRUNE_SERVER_BREAK=journal-compaction-drops-high-water        追加日志整理不写请求号高水位                     journal-high-water-survives-compaction
//!   PRUNE_SERVER_BREAK=sync-failure-kept-serving                  同步 WAL 失败之后服务不退出、照常回确认          write-ahead-log-sync-failure-never-acknowledged
//!   PRUNE_SERVER_BREAK=response-slot-released-on-enqueue          回应一进写出队就放名额、不记进额度               unread-responses-bounded
//!   PRUNE_SERVER_BREAK=frame-body-read-before-admission           先照声明的长度分配、读完帧体再过准入             incomplete-frames-bounded
//!   PRUNE_SERVER_BREAK=journal-locks-renamed-file                 追加日志的锁加在会被整理改名换掉的日志本身上     journal-lock-survives-compaction
//!   PRUNE_SERVER_BREAK=oversize-frame-sent-and-retried            客户端发帧不核帧长、照发，「帧长度不对」当断线重连重发  oversize-frame-reported-at-once
//!   PRUNE_SERVER_BREAK=claim-response-unbudgeted                  认领不核回应字节、照编越过帧长上限的回应         claim-response-within-frame-limit
//!   PRUNE_SERVER_BREAK=connect-without-deadline                   阻塞的 connect、报到不设写期限、在途帧不受重连总时长管  connect-bounded-by-reconnect-total
//!   PRUNE_SERVER_BREAK=cell-runs-in-runner-process                自证把判这一条的那一格留在跑格的进程的线程里跑  cell-runs-alone-in-own-process
//!   PRUNE_SERVER_BREAK=descriptor-soft-limit-not-raised           开库之前不抬软上限、不核够不够（改前的开法）      library-opens-under-low-descriptor-soft-limit
//!   PRUNE_SERVER_BREAK=delete-prefix-overruns                     删前缀走出前缀不停、越界删到下一个前缀           delete-prefix-only-under-prefix
//!   PRUNE_SERVER_BREAK=delete-prefix-dry-run-deletes              只数不删的删前缀请求照样把数到的键删掉           delete-prefix-counts-before-deleting
//!
//! 没有弄坏开关的格：single-library-holder（拒的是 RocksDB 自己的 LOCK）、tcp-route-from-configuration（读配置那一处由
//! missing-tcp-address-refused 的开关管）、bench-runs-every-mode（跑通性的冒烟格；fault-injection 的逐键核对与它的去重、重放
//! 由上面那几格的开关证红）。
//!
//! 换不掉的：服务的存储是真的 RocksDB（自证用临时库目录）；`/proc` 读数只在 Linux 上成立。

fn main() {
    std::process::exit(prune_server::command_line::run(
        prune_server::command_line::ProgramName::Server,
    ));
}
