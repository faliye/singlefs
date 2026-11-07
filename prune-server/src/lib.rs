//! prune-server：判定库服务的原型。服务进程是 RocksDB 的唯一持有者，两台的任务都经它读写同一份库
//! （records/2026-11-01-判定库服务化.md「用户定案」那几条）。
//!
//! 模块：
//! - `protocol`：长度前缀的二进制帧，一帧一批操作（成批点读、成批写、按前缀扫、统计、认领、按前缀删），回应带请求序号；
//! - `storage`：RocksDB 那一层，服务只有自己那一份库、只有默认列族：三类表「录入、事实、结论」只是分表码、键原样，
//!   已生效的请求号与认领记录两张记账表也在默认列族、各一个客户端碰不到的键前缀；
//! - `server`：两处监听（unix socket、TCP）、每条连接一个读线程一个写线程、准入（待处理有界，满了回忙）、
//!   读交工作线程池，回应按完成先后写回（可乱序）；`committer`：写与认领的组提交线程（跨帧合并同步、去重、认领）；
//! - `client`：流水客户端与一问一答的客户端；`reliable_client`：容错客户端（追加日志、重连重放、超时、退避、窗口、忙退避）；
//! - `segmented_values`：客户端一侧的大值分段（照 Python 客户端的分段格式写、读回拼接核 sha256、按前缀扫时段键与版本登记键不当条目）；
//!   `sha256`：分段清单记的整值摘要；
//! - `journal`：容错客户端的本地追加日志；`keepalive`：TCP keepalive 的设置与读回；
//! - `fault_proxy`：自证与压测器用的故障注入代理（断帧、吞回应、半开、帧重复、延迟抖动）；
//! - `configuration`：仓根 `multi-host.env` 里服务的 TCP 地址、socket 与库的缺省位置；
//! - `bench`：压测器的负载与计量；`bench_progress`：压测器的进度行与卡住诊断（按线程读 `/proc` 打各线程等在哪）；`environment`：压测器的环境诊断（loadavg、块设备忙度）；`selftest` 与 `selftest_fault_cells`：自证各格与弄坏开关的复跑；
//! - `break_switches`：弄坏开关 `PRUNE_SERVER_BREAK` 的表；`command_line`：两个二进制共用的入口；
//! - `preflight`：准入与运行条件的现判；`process_watch`：读 `/proc` 取服务的内存峰值与 CPU 秒数；
//! - `interruption`：压测器收 SIGTERM / SIGINT 之后收尾删压测库；`spawned`：起子进程、读它报的就绪行。
//! - `connections`：服务这一侧的连接登记与客户端报到的身份；`fault_log`：服务自身出错时追加的故障行；
//! - `oversized_values`：离线子命令 `drop-oversized-values`，服务停着时开库删掉一把键单独一帧也读不回的整值（分段之前写进来的旧行）；
//! - `administration`：控制命令调的 `admin` 子命令（服务通不通、列连着的客户端、宣布停服务）与自证起的客户端子进程；
//!   `selftest_service_cells`：自证里常驻单元、控制命令、停服务、故障行、监听、写超时与经 TCP 代理杀服务这几格；
//!   `selftest_library_layout_cells`：自证里库布局那三格（只有默认列族与记账前缀、拒开对不上的库、软上限低时抬到硬上限开库）与按键前缀删行那两格；
//!   `selftest_hardening_cells`：自证里加固那七格（追加日志的请求号高水位与排他锁、同步失败不回确认、回应与收帧受额度与期限管、
//!   认领回应不越过帧长上限、建连受重连总时长上限管）。

pub mod administration;
pub mod bench;
pub mod bench_progress;
pub mod break_switches;
pub mod client;
pub mod command_line;
pub mod committer;
pub mod configuration;
pub mod connections;
pub mod environment;
pub mod fault_log;
pub mod fault_proxy;
pub mod interruption;
pub mod journal;
pub mod keepalive;
pub mod oversized_values;
pub mod preflight;
pub mod process_watch;
pub mod protocol;
pub mod reliable_client;
pub mod segmented_values;
pub mod selftest;
pub mod selftest_fault_cells;
pub mod selftest_hardening_cells;
pub mod selftest_library_layout_cells;
pub mod selftest_service_cells;
pub mod server;
pub mod sha256;
pub mod spawned;
pub mod storage;
