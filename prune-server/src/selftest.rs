//! 自证（`prune-server --selftest`）与弄坏开关的复跑（`prune-server --selftest-break-sweep`）。
//!
//! 每格起自己的真服务（这个可执行文件的 `serve --selftest-instance`），挂在 `$TMPDIR` 下这一格自己的目录里的 socket 上，
//! TCP 走回环；格与格之间不共用目录、库与服务，并行跑，按登记次序打结果。`PRUNE_SERVER_SELFTEST_CELLS=<格,…>` 只跑点名的格。
//! 每格各在自己的进程里：跑格的进程给每格另起一个 `prune-server selftest-cell --selftest-instance --runner-process <跑格的进程号> <格名>`，
//! 那个进程里只跑这一格，标准输出先打这一格的说明，末行 `PRUNE_SERVER_CELL_RESULT result=passed|failed`（绿退 0，红退 1）；
//! 点名单跑一格与全量里跑它走的是同一条路。格挤在同一个进程的线程里时，进程的描述符表是各格共用的：别的格起服务
//! （`spawned.rs` 设了 `pre_exec`，走 fork 再 exec）fork 出来、还没 exec 的子进程复制着全部描述符，带 close-on-exec 的也在，
//! 到 exec 才关；追加日志的锁文件（`journal.rs`，`flock` 记在打开的那份文件上）这时关掉再开，锁还在那份复制上，
//! 重开报「被别的客户端开着」：单跑绿、全量里红。另起进程之后别的格的 fork 复制不到这一格的描述符。
//! 格 cell-runs-alone-in-own-process 判这一条（弄坏开关 cell-runs-in-runner-process 把它留在跑格的进程的线程里）。
//! 末行 `PRUNE_SERVER_SELFTEST cells=… passed=… failed=… failed_cells=<逗号分隔，全绿是 ->`；全绿退 0，有红退 1，格名或开关写错退 2。
//!
//! 复跑弄坏开关：先不设开关跑一遍自证（要全绿），再对 `BreakSwitch::ALL` 逐项设 `PRUNE_SERVER_BREAK=<项>` 跑一遍，
//! 红的格要恰好是那一项点名的那一格（`BreakSwitch::red_cell`）；末行 `PRUNE_SERVER_BREAK_SWEEP switches=… matched=… baseline=green|red`，全对退 0。

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use rocksdb::statistics::Ticker;

use crate::bench::{
    judge_achieved_rate, percentile_microseconds, BenchMode, RateJudgment, BENCH_LINE_PREFIX,
    ENVIRONMENT_DIRTY_EXIT_CODE, LIBRARY_LINE_PREFIX, SERVER_READY_TIMEOUT, STARTED_LINE_PREFIX,
};
use crate::bench_progress::{PROGRESS_LINE_PREFIX, STALLED_LINE_PREFIX, THREAD_LINE_PREFIX};
use crate::break_switches::{BreakSwitch, BreakSwitches, BREAK_VARIABLE};
use crate::client::{connect, ReceiveFailure, RouteAddress, SimpleClient};
use crate::configuration::{create_directory_holding_a_socket, NETWORK_ADDRESS_KEY};
use crate::protocol::{
    begin_point_read_frame, encode_raw_request_frame, statistics_value, OperationCode,
    RejectionStatus, ResponseOutcome, StatisticsCounter, Table,
};
use crate::selftest_fault_cells::{
    cell_bench_environment_judgment, cell_busy_backpressure_bounded,
    cell_claim_exclusive_and_lease_expiry, cell_delay_jitter_duplicate_tolerated,
    cell_group_commit_coalesces_syncs, cell_half_open_connection_recovered,
    cell_reconnect_gives_up_at_limit, cell_replay_after_disconnect_and_server_kill,
    cell_server_kill_while_client_writes,
};
use crate::selftest_hardening_cells::{
    cell_claim_lease_covers_the_holder_after_the_answer, cell_claim_response_within_frame_limit,
    cell_connect_bounded_by_reconnect_total, cell_delete_prefix_response_within_pending_budget,
    cell_incomplete_frames_bounded, cell_journal_high_water_survives_compaction,
    cell_journal_lock_survives_compaction, cell_oversize_frame_reported_at_once,
    cell_unread_responses_bounded, cell_worker_responses_within_pending_budget,
    cell_write_ahead_log_sync_failure_never_acknowledged, cell_write_storage_failure_replayed,
};
use crate::selftest_library_layout_cells::{
    cell_delete_prefix_counts_before_deleting, cell_delete_prefix_only_under_prefix,
    cell_delete_prefix_releases_claims, cell_library_opens_under_low_descriptor_soft_limit,
    cell_library_refuses_mismatched, cell_multi_host_configuration_from_the_main_worktree,
    cell_prefix_scan_resumes_after_the_last_key, cell_segmented_rewrite_survives_writer_death,
    cell_single_library_layout,
};
use crate::selftest_service_cells::{
    cell_client_write_timeout_when_peer_stops_reading, cell_control_install_start_stop_check,
    cell_fault_line_on_server_error, cell_intentional_stop_fails_clients_fast,
    cell_stop_lists_connected_clients, cell_tcp_proxy_server_kill_client_fails,
    cell_unannounced_server_death_bounded, cell_watch_reports_new_fault_line,
    cell_watch_reports_server_unreachable, cell_watch_reports_unit_not_running,
};
use crate::server::{FRAMES_HANDLED_COUNTER, LISTENING_LINE_PREFIX};
use crate::spawned::{SpawnedProcess, StandardError};

pub const SELFTEST_LINE_PREFIX: &str = "PRUNE_SERVER_SELFTEST";
pub const SWEEP_LINE_PREFIX: &str = "PRUNE_SERVER_BREAK_SWEEP";
pub const CELL_SELECTION_VARIABLE: &str = "PRUNE_SERVER_SELFTEST_CELLS";
/// 自证起的服务的工作线程数：至少 2 个才乱序得起来。
pub(crate) const SELFTEST_WORKER_THREADS: &str = "4";

/// 每格进程的末行开头（文件头「每格各在自己的进程里」）。
pub const CELL_RESULT_LINE_PREFIX: &str = "PRUNE_SERVER_CELL_RESULT";
/// `selftest-cell` 子命令里跑格的进程号前面那个参数。
const RUNNER_PROCESS_ARGUMENT: &str = "--runner-process";
/// 判「每格各在自己的进程里」的那一格；弄坏开关 cell-runs-in-runner-process 只把它留在跑格的进程里。
const OWN_PROCESS_CELL_NAME: &str = "cell-runs-alone-in-own-process";

/// 这一格在哪个进程里跑（文件头「每格各在自己的进程里」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CellPlacement {
    /// 跑格的进程为这一格另起的进程；`runner` 是跑格的进程号。
    Own { runner: u32 },
    /// 跑格的进程自己的一个线程，同一个进程里跑着 `cells_in_process` 格（只在弄坏开关 cell-runs-in-runner-process 下）。
    RunnerThread { cells_in_process: usize },
}

pub(crate) struct CellContext {
    pub(crate) executable: PathBuf,
    pub(crate) break_switches: BreakSwitches,
    pub(crate) placement: CellPlacement,
}

type CellFunction = fn(&CellContext) -> Result<String, String>;

struct Cell {
    name: &'static str,
    remedy: &'static str,
    function: CellFunction,
}

const CELLS: [Cell; 54] = [
    Cell {
        name: "batched-point-read",
        remedy: "看 storage.rs 的 read_points：同一张表的键要一次 batched_multi_get_cf 查完，次序与请求相同",
        function: cell_batched_point_read,
    },
    Cell { name: "prefix-scan", remedy: "看 storage.rs 的 scan_prefix_into：走出前缀就停、到条数上限标截断", function: cell_prefix_scan },
    Cell {
        name: "tcp-route-from-configuration",
        remedy: "看 command_line.rs 的 run_server 读多机配置那一段与 server.rs 的 TCP 监听",
        function: cell_tcp_route_from_configuration,
    },
    Cell {
        name: "missing-tcp-address-refused",
        remedy: "看 command_line.rs 的 run_server：多机配置里没有 PRUNE_SERVER_TCP_ADDRESS 要退 5、点名这个键，不退回回环",
        function: cell_missing_tcp_address_refused,
    },
    Cell {
        name: "pipelined-out-of-order",
        remedy: "看 client.rs 的 ConnectionReceiver::receive：回应按序号认领，不按到达先后",
        function: cell_pipelined_out_of_order,
    },
    Cell {
        name: "frame-length-rejected",
        remedy: "看 server.rs 的 handle_connection：帧体长度不在 [帧头, 上限] 之内回序号 0 的拒绝、关连接",
        function: cell_frame_length_rejected,
    },
    Cell {
        name: "unknown-operation-rejected",
        remedy: "看 server.rs 的 process_frame：认不得的操作码回状态 2 的拒绝，连接照常可用",
        function: cell_unknown_operation_rejected,
    },
    Cell {
        name: "malformed-payload-rejected",
        remedy: "看 protocol.rs 的 decode_request_body：个数对不上、认不得的表、帧尾多字节都回状态 1，一条都不写",
        function: cell_malformed_payload_rejected,
    },
    Cell {
        name: "single-library-holder",
        remedy: "同一个库目录第二个服务要打不开（RocksDB 的 LOCK）、退 3；看 storage.rs 的 open 有没有加退回只读或副本的路",
        function: cell_single_library_holder,
    },
    Cell {
        name: "bench-rate-judgment",
        remedy: "看 bench.rs 的 judge_achieved_rate（达标线 95%）与 percentile_microseconds（最近秩法）",
        function: cell_bench_rate_judgment,
    },
    Cell {
        name: "bench-runs-every-mode",
        remedy: "看压测器自己打的 ✗ 与末行；六种负载小速率要跑通（达标、没达标、环境不干净都算，读回对不上、完整性闸、帧序键不对、服务起不来算红）、末行字段齐、压测库删掉",
        function: cell_bench_runs_every_mode,
    },
    Cell {
        name: "bench-interrupted-removes-library",
        remedy: "看 bench.rs 的 TemporaryLibraryDirectory 与 interruption.rs：收 SIGTERM 要停下、删压测库、停掉服务、退 3",
        function: cell_bench_interrupted_removes_library,
    },
    Cell {
        name: "fault-injection-bench-finishes",
        remedy: "卡住先看压测器的 PRUNE_SERVER_BENCH_PROGRESS 行停在哪个阶段、client_activity 是什么，判了卡住看 PRUNE_SERVER_BENCH_THREAD 行各线程等在哪；重写超量看 journal.rs 的整理门槛（要随上次整理完的长度翻倍，不许每个确认都整份重写）",
        function: cell_fault_injection_bench_finishes,
    },
    Cell {
        name: "replay-after-disconnect-and-server-kill",
        remedy: "断线那一半看 reliable_client.rs 的 mark_broken 与 send_unsent（在途的写帧要标成重发、重连之后按原次序重发，不许当成已确认）；kill -9 那一半看 committer.rs 的 commit（先写、flush_wal(true) 同步，同步之后才回确认）",
        function: cell_replay_after_disconnect_and_server_kill,
    },
    Cell {
        name: "delay-jitter-duplicate-tolerated",
        remedy: "看 committer.rs 的 add_write 去重（身份加请求号）与 reliable_client.rs 的 handle_response：序号已经回过的回应记一笔丢掉",
        function: cell_delay_jitter_duplicate_tolerated,
    },
    Cell {
        name: "half-open-connection-recovered",
        remedy: "看 reliable_client.rs 的 pump_once（在途帧到点没回应要断开重连）与 keepalive.rs（TCP 连接要开 keepalive）",
        function: cell_half_open_connection_recovered,
    },
    Cell {
        name: "server-kill-while-client-writes",
        remedy: "看 fault_proxy.rs 的 server_side_gone：服务那头读到流尾或写出错时客户端那头要整个关掉（shutdown Both），客户端的写才会出错、才会重连",
        function: cell_server_kill_while_client_writes,
    },
    Cell {
        name: "reconnect-gives-up-at-limit",
        remedy: "看 reliable_client.rs 的 ensure_connected 与 backoff_delay（翻倍、封顶、0.5 到 1 倍抖动、次数与总时长上限）、read（失败回错误）、journal.rs（写帧先记日志）",
        function: cell_reconnect_gives_up_at_limit,
    },
    Cell {
        name: "busy-backpressure-bounded",
        remedy: "看 server.rs 的 Admission（待处理帧数到上限当场回忙、不排队）与 reliable_client.rs 的 defer_after_busy（照服务给的毫秒数加抖动再重发）",
        function: cell_busy_backpressure_bounded,
    },
    Cell {
        name: "claim-exclusive-and-lease-expiry",
        remedy: "看 committer.rs 的 add_claim（别人持有且租期没到回别人持有、已有结论回已有结论）与 add_write（写结论同一批释放认领）",
        function: cell_claim_exclusive_and_lease_expiry,
    },
    Cell {
        name: "group-commit-coalesces-syncs",
        remedy: "看 committer.rs 的 run（窗口里收到的几帧合成一组、只同步一次 WAL）与 storage.rs 的 commit_group（一组一个 WriteBatch、flush_wal 一次）",
        function: cell_group_commit_coalesces_syncs,
    },
    Cell {
        name: "bench-environment-judgment",
        remedy: "看 environment.rs 的 judge_environment（起止 loadavg 有一个超阈值就不干净）、parse_load_averages、parse_input_output_ticks、busy_percent",
        function: cell_bench_environment_judgment,
    },
    Cell {
        name: "control-install-start-stop-check",
        remedy: "看 prune-server/control 的 install（--dry-run 一个文件不写、systemctl 一条不起；装完回读服务通不通）、start（回读 is-active 与服务通不通）、stop（回读 is-active 与 cgroup.procs）、check",
        function: cell_control_install_start_stop_check,
    },
    Cell {
        name: "stop-lists-connected-clients",
        remedy: "看 connections.rs 的 announcement_from_environment（报到带 TASK_SERVER_TASK、TASK_SERVER_ATTEMPT、主机名、进程号）、server.rs 的 handle_reader_request（报到记进登记、宣布停服务回清单）、control 的 stop（打清单、排他写停服务记录）",
        function: cell_stop_lists_connected_clients,
    },
    Cell {
        name: "intentional-stop-fails-clients-fast",
        remedy: "看 server.rs 宣布停服务那一段（给每条连接发「服务停了」，之后来的帧也只回它）与 reliable_client.rs 的 handle_connection_level_response（收到「服务停了」回 ServerStopped，不重试）",
        function: cell_intentional_stop_fails_clients_fast,
    },
    Cell {
        name: "unannounced-server-death-bounded",
        remedy: "看 reliable_client.rs 的 ensure_connected（重连总时长上限）与 ClientFailure::exit_code（「连不上」退 75）",
        function: cell_unannounced_server_death_bounded,
    },
    Cell {
        name: "fault-line-on-server-error",
        remedy: "看 fault_log.rs 的 FaultLog::record 与 committer.rs 的 fail_group（组提交失败每条连接记一行，带连接的报到身份）",
        function: cell_fault_line_on_server_error,
    },
    Cell {
        name: "watch-reports-unit-not-running",
        remedy: "看 prune-server/control 的 watch：单元 is-active 不是 active 要打出来退 1",
        function: cell_watch_reports_unit_not_running,
    },
    Cell {
        name: "watch-reports-server-unreachable",
        remedy: "看 prune-server/control 的 watch：admin health 不通要打出来退 1",
        function: cell_watch_reports_server_unreachable,
    },
    Cell {
        name: "watch-reports-new-fault-line",
        remedy: "看 prune-server/control 的 watch：从起的那一刻的长度往后读故障日志，新增的故障行打出来退 1，起之前就有的不算；退出时打连着的客户端与 task-server/task status 的查法",
        function: cell_watch_reports_new_fault_line,
    },
    Cell {
        name: "client-write-timeout-when-peer-stops-reading",
        remedy: "看 reliable_client.rs 的 open_connection：连接要设写超时（write_timeout），对端不读时写出错、断开重连重放",
        function: cell_client_write_timeout_when_peer_stops_reading,
    },
    Cell {
        name: "tcp-proxy-server-kill-client-fails",
        remedy: "看 fault_proxy.rs 的 server_side_gone：服务那头没了要把客户端那头真关掉（RST、shutdown、放掉留着的复制），TCP 上客户端的写才出错",
        function: cell_tcp_proxy_server_kill_client_fails,
    },
    Cell {
        name: "single-library-layout",
        remedy: "看 storage.rs 文件头与 applied_request_key、claim_key（记账行在默认列族、0xF0 与 0xF1 起头）、scan_prefix_into（走到 0xF0 那一段就停），\
                 protocol.rs 的 service_key_refusal（客户端碰 0xF0–0xFF 起头的键回状态 1），command_line.rs 不带 --library 取状态目录下的 library",
        function: cell_single_library_layout,
    },
    Cell {
        name: "library-refuses-mismatched",
        remedy: "看 storage.rs 的 open_single_column_family_library（有文件没有 CURRENT、另有列族退 6）与 classify_rocksdb_failure（锁与读写出错退 3，选项对不上退 6）",
        function: cell_library_refuses_mismatched,
    },
    Cell {
        name: "library-opens-under-low-descriptor-soft-limit",
        remedy: "看 storage.rs 的 raise_descriptor_limit_for（开库之前软上限抬到硬上限、回读，抬完小于库目录文件数加开着的描述符加余量就退 8、\
                 说「文件数上限不够：要 N、硬上限 M」）与 open_single_column_family_library 有没有在开库之前调它",
        function: cell_library_opens_under_low_descriptor_soft_limit,
    },
    Cell {
        name: "delete-prefix-only-under-prefix",
        remedy: "看 storage.rs 的 scan_prefix_for_deletion（走出前缀、走到记账那一段就停，一帧至多碰 maximum_keys 把，前缀下还有键时说没有更多=假）与 protocol.rs 的 \
                 DeletePrefix 解帧（空前缀、记账那一段的前缀、不以前缀起头的续扫起点、本帧上限越界都回状态 1）",
        function: cell_delete_prefix_only_under_prefix,
    },
    Cell {
        name: "delete-prefix-counts-before-deleting",
        remedy: "看 storage.rs 的 scan_prefix_for_deletion 的 count_only（只数不删时不交回要删的键）与 committer.rs 的 add_delete_prefix（只在要删时往写批里记删除），数的口径和删一样；客户端 delete_prefix 的续扫起点取上一帧回的最后一把键",
        function: cell_delete_prefix_counts_before_deleting,
    },
    Cell {
        name: "journal-high-water-survives-compaction",
        remedy: "看 journal.rs 的 compact（整理要写请求号高水位记录）与 open（下一个请求号取高水位、写帧、已确认记录里最大的加一）",
        function: cell_journal_high_water_survives_compaction,
    },
    Cell {
        name: "write-ahead-log-sync-failure-never-acknowledged",
        remedy: "看 committer.rs 的 commit 与 exit_after_sync_failure（同步 WAL 失败回存储出错、服务退 7，不留在进程里接着回确认）、storage.rs 的 commit_group（分写失败与同步失败）",
        function: cell_write_ahead_log_sync_failure_never_acknowledged,
    },
    Cell {
        name: "unread-responses-bounded",
        remedy: "看 connections.rs 的 Responder::send_response（回应带着名额进写出队，写出去或丢掉才放手）、server.rs 的工作线程（回应先记进额度，记不进换成忙）与 write_responses（写堵住到期限关连接）",
        function: cell_unread_responses_bounded,
    },
    Cell {
        name: "incomplete-frames-bounded",
        remedy: "看 server.rs 的 FrameReader::read_request_frame（分配帧体之前先记进额度，记不进读完丢掉回忙；一帧在期限里收齐）与 SharedState::open_connection（连接数上限）",
        function: cell_incomplete_frames_bounded,
    },
    Cell {
        name: "journal-lock-survives-compaction",
        remedy: "看 journal.rs 的 open_pausing_before_lock（排他锁加在不改名的 <日志>.lock 上，持锁之后才开日志）与 compact",
        function: cell_journal_lock_survives_compaction,
    },
    Cell {
        name: "claim-response-within-frame-limit",
        remedy: "看 committer.rs 的 add_claim（先逐键判、数回应字节，越过帧长上限整帧回状态 1、一个键都不认领）与 protocol.rs 的 claim_outcomes_payload_bytes",
        function: cell_claim_response_within_frame_limit,
    },
    Cell {
        name: "connect-bounded-by-reconnect-total",
        remedy: "看 client.rs 的 open_announced_stream（非阻塞连接、每一步用剩下的期限、报到前设写期限）与 reliable_client.rs 的 ensure_connected、reconnect_cycle_deadline（在途帧受这一轮重连的总时长上限管）",
        function: cell_connect_bounded_by_reconnect_total,
    },
    Cell {
        name: "oversize-frame-reported-at-once",
        remedy: "看 reliable_client.rs 的 oversize_frame_failure（写、读、认领的帧拼好先核帧长，超了回协议错、不记日志、不发）与 mark_broken（接收缓冲里的「帧长度不对」回协议错、不重连），client.rs 的 ConnectionSender::send_frame",
        function: cell_oversize_frame_reported_at_once,
    },
    Cell {
        name: "claim-lease-covers-the-holder-after-the-answer",
        remedy: "看 reliable_client.rs 的 write_unsent_frames（认领每次发出去都给整段租期，不换成剩下的）与 committer.rs 的 add_claim\
                 （同一身份、租期没到的认领到期取已记下的与现在加帧里租期里晚的那一个，不往前挪）",
        function: cell_claim_lease_covers_the_holder_after_the_answer,
    },
    Cell {
        name: "write-storage-failure-replayed",
        remedy: "看 reliable_client.rs 的 handle_response 与 retry_after_storage_failure（写帧回存储出错不从操作表拿掉，退避之后照同一请求号再交，\
                 连着出错到重连次数上限回 StorageFailing 退 75），committer.rs 的 commit（写失败照常服务、同步失败退 7）",
        function: cell_write_storage_failure_replayed,
    },
    Cell {
        name: "delete-prefix-response-within-pending-budget",
        remedy: "看 committer.rs 的 add_delete_prefix（回应先 reserve_response 记进额度，记不进回忙、一把都不删，记得进才把删除记进写批）与 server.rs 文件头「额度」",
        function: cell_delete_prefix_response_within_pending_budget,
    },
    Cell {
        name: "delete-prefix-releases-claims",
        remedy: "看 committer.rs 的 add_delete_prefix 与 release_claim_on（删掉的每一把键同时从内存认领表与写批里去掉认领；只数不删不动认领）",
        function: cell_delete_prefix_releases_claims,
    },
    Cell {
        name: "worker-responses-within-pending-budget",
        remedy: "看 server.rs 的 ResponseBudget 与 execute_request（成批点读先数回应多大、记进额度再分配，按前缀扫每读出一条先记再放进回应，记不进整帧回忙、放回已记的）与 storage.rs 的 scan_prefix_into",
        function: cell_worker_responses_within_pending_budget,
    },
    Cell {
        name: "multi-host-configuration-from-the-main-worktree",
        remedy: "看 configuration.rs 的 multi_host_configuration_path_from 与 main_worktree_root_of（没设 SINGLEFS_MULTI_HOST_CONFIG 时照 git rev-parse --git-common-dir 的上一层取主工作树根下那一份）",
        function: cell_multi_host_configuration_from_the_main_worktree,
    },
    Cell {
        name: "segmented-rewrite-survives-writer-death",
        remedy: "看 segmented_values.rs 的 plan_segmented_write（段键带整值的 sha256、版本登记键并上此刻那一版）与 reliable_client.rs 的 write_segments_of_large_values、read_missing_segments（段键取清单的版本，写前删过期的版本）",
        function: cell_segmented_rewrite_survives_writer_death,
    },
    Cell {
        name: "prefix-scan-resumes-after-the-last-key",
        remedy: "看 protocol.rs 的按前缀扫帧（续扫起点）与 storage.rs 的 scan_prefix_into（续扫起点不空时从它后面接一个 0 字节起扫）",
        function: cell_prefix_scan_resumes_after_the_last_key,
    },
    Cell {
        name: OWN_PROCESS_CELL_NAME,
        remedy: "看 selftest.rs 的 run_cell_isolated 与 run_cell_in_own_process（每格另起一个 selftest-cell 进程只跑它，父进程是跑格的进程）",
        function: cell_runs_alone_in_own_process,
    },
];

/// 一格里放 socket 的最长相对路径（控制命令那一格的 `run/server.sock`；别的格是目录下的 `server.sock`、`proxy.sock`）。
const LONGEST_SOCKET_RELATIVE_PATH_IN_A_CELL: &str = "run/server.sock";
const CELL_DIRECTORY_NAME_PREFIX: &str = "prune-selftest";

/// 一格自己的临时目录：放手时删。建法见 [`create_directory_holding_a_socket`]，里面的 socket 全路径不超 107 字节。
pub(crate) struct CellDirectory {
    pub(crate) path: PathBuf,
}

impl CellDirectory {
    pub(crate) fn create() -> Result<CellDirectory, String> {
        CellDirectory::create_under(std::env::var_os("TMPDIR"))
    }

    /// `temporary_directory_variable` 是 `TMPDIR` 的值。
    fn create_under(
        temporary_directory_variable: Option<OsString>,
    ) -> Result<CellDirectory, String> {
        let path = create_directory_holding_a_socket(
            temporary_directory_variable,
            CELL_DIRECTORY_NAME_PREFIX,
            LONGEST_SOCKET_RELATIVE_PATH_IN_A_CELL,
        )?;
        Ok(CellDirectory { path })
    }
}

impl Drop for CellDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.path) {
            eprintln!("  ✗ 删不了自证的临时目录 {}：{error}", self.path.display());
            eprintln!("  → 怎么办：手动删掉它");
        }
    }
}

/// 一格起的服务。字段次序就是放手次序：先停服务，再删目录。
pub(crate) struct TestServer {
    #[allow(
        dead_code,
        reason = "只为放手时停掉服务：SpawnedProcess 的 Drop 照进程号 SIGKILL 再收尸"
    )]
    process: SpawnedProcess,
    unix_socket: PathBuf,
    pub(crate) network_address: String,
    library: PathBuf,
    #[allow(
        dead_code,
        reason = "只为放手时删掉这一格的临时目录（CellDirectory 的 Drop）"
    )]
    directory: CellDirectory,
}

impl TestServer {
    pub(crate) fn unix_route(&self) -> RouteAddress {
        RouteAddress::UnixSocket(self.unix_socket.clone())
    }

    pub(crate) fn network_route(&self) -> RouteAddress {
        RouteAddress::Network(self.network_address.clone())
    }
}

pub(crate) fn line_field(line: &str, name: &str) -> Option<String> {
    line.split_whitespace()
        .find_map(|field| {
            field
                .strip_prefix(name)
                .and_then(|rest| rest.strip_prefix('='))
        })
        .map(str::to_string)
}

/// 起服务的命令：socket、状态目录（故障日志在里面）、库都在这一格自己的目录里。
pub(crate) fn server_command(
    context: &CellContext,
    directory: &Path,
    library: &Path,
) -> Result<Command, String> {
    let mut command = Command::new(&context.executable);
    command
        .arg("serve")
        .arg("--selftest-instance")
        .arg("--socket")
        .arg(directory.join("server.sock"))
        .arg("--state-directory")
        .arg(directory)
        .arg("--library")
        .arg(library)
        .arg("--worker-threads")
        .arg(SELFTEST_WORKER_THREADS);
    Ok(command)
}

/// 起服务；`network_arguments` 是 TCP 那几个参数（`--tcp 127.0.0.1:0` 或 `--multi-host-configuration <文件>`），`extra` 另加。
pub(crate) fn start_test_server(
    context: &CellContext,
    network_arguments: &[String],
    extra: &[&str],
) -> Result<TestServer, String> {
    let directory = CellDirectory::create()?;
    let library = directory.path.join("library");
    let mut command = server_command(context, &directory.path, &library)?;
    command.args(network_arguments).args(extra);
    let mut process = SpawnedProcess::spawn(command, StandardError::Capture)?;
    let line = process
        .wait_for_line_starting_with(LISTENING_LINE_PREFIX, SERVER_READY_TIMEOUT)
        .map_err(|message| {
            format!(
                "服务没起来：{message}；标准错误：{}",
                process.standard_error_text()
            )
        })?;
    let unix_socket = PathBuf::from(
        line_field(&line, "unix").ok_or_else(|| format!("就绪行没有 unix=：{line}"))?,
    );
    let network_address =
        line_field(&line, "tcp").ok_or_else(|| format!("就绪行没有 tcp=：{line}"))?;
    Ok(TestServer {
        process,
        unix_socket,
        network_address,
        library,
        directory,
    })
}

pub(crate) fn loopback_arguments() -> Vec<String> {
    vec!["--tcp".to_string(), "127.0.0.1:0".to_string()]
}

pub(crate) fn counter(counters: &[StatisticsCounter], name: &str) -> Result<u64, String> {
    statistics_value(counters, name).ok_or_else(|| format!("统计回应里没有 {name}"))
}

pub(crate) fn counter_delta(
    before: &[StatisticsCounter],
    after: &[StatisticsCounter],
    ticker: Ticker,
) -> Result<u64, String> {
    Ok(counter(after, ticker.name())?.saturating_sub(counter(before, ticker.name())?))
}

pub(crate) fn bytes(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

fn cell_batched_point_read(context: &CellContext) -> Result<String, String> {
    let server = start_test_server(context, &loopback_arguments(), &[])?;
    let mut client = SimpleClient::connect(&server.unix_route(), context.break_switches.clone())?;
    let entries: Vec<(Table, Vec<u8>, Vec<u8>)> = (0..100)
        .map(|index| {
            (
                Table::Conclusions,
                bytes(&format!("point-read/{index:03}")),
                bytes(&format!("value-{index}-").repeat(3)),
            )
        })
        .collect();
    let written = client.write(&entries)?;
    if written != 100 {
        return Err(format!("写确认说写了 {written} 条，该是 100 条"));
    }
    let mut keys: Vec<(Table, Vec<u8>)> =
        vec![(Table::Conclusions, bytes("point-read/absent-front"))];
    for (position, (table, key, _)) in entries.iter().enumerate() {
        if position == 50 {
            keys.push((Table::Conclusions, bytes("point-read/absent-middle")));
        }
        keys.push((*table, key.clone()));
    }
    keys.push((Table::Conclusions, bytes("zzzz/absent-back")));
    let before = client.statistics()?;
    let values = client.read(&keys)?;
    let after = client.statistics()?;
    if values.len() != keys.len() {
        return Err(format!("{} 个键读回 {} 个值", keys.len(), values.len()));
    }
    let expected: HashMap<&[u8], &[u8]> = entries
        .iter()
        .map(|(_, key, value)| (key.as_slice(), value.as_slice()))
        .collect();
    for ((_, key), value) in keys.iter().zip(&values) {
        let wanted = expected.get(key.as_slice()).copied();
        if value.as_deref() != wanted {
            return Err(format!(
                "键 {} 读回 {value:?}，该是 {wanted:?}",
                String::from_utf8_lossy(key)
            ));
        }
    }
    let multi_get_calls = counter_delta(&before, &after, Ticker::NumberMultigetCalls)?;
    let multi_get_keys = counter_delta(&before, &after, Ticker::NumberMultigetKeysRead)?;
    if multi_get_calls != 1 || multi_get_keys != 103 {
        return Err(format!("一帧 103 个键，RocksDB 数到 multi_get 调了 {multi_get_calls} 次、经它读了 {multi_get_keys} 个键；该是 1 次、103 个"));
    }
    Ok("一帧 103 个键（3 个不在）读回逐个对上；RocksDB 数到 multi_get 1 次、103 个键".to_string())
}

/// 统计里的 WAL 同步计数比 `before` 涨了至少 1 次才回（至多等 `SYNC_SETTLE_TIMEOUT`）：
/// 同步落在回确认之后的写法（弄坏开关 acknowledge-before-sync）也要数得全，判它的是「不丢」那一格，不是这里。
pub(crate) fn statistics_after_sync(
    client: &mut SimpleClient,
    before: &[StatisticsCounter],
) -> Result<Vec<StatisticsCounter>, String> {
    let deadline = std::time::Instant::now() + SYNC_SETTLE_TIMEOUT;
    loop {
        let after = client.statistics()?;
        if counter_delta(before, &after, Ticker::WalFileSynced)? >= 1
            || std::time::Instant::now() >= deadline
        {
            return Ok(after);
        }
        thread::sleep(Duration::from_millis(50));
    }
}

/// 等同步落下来的上限（弄坏开关 acknowledge-before-sync 回确认之后 2 秒才同步）。
const SYNC_SETTLE_TIMEOUT: Duration = Duration::from_secs(10);

/// 一帧 100 条写只同步一次；「组提交」那一格的一半。
pub(crate) fn single_frame_single_sync(context: &CellContext) -> Result<String, String> {
    let server = start_test_server(context, &loopback_arguments(), &[])?;
    let mut client = SimpleClient::connect(&server.unix_route(), context.break_switches.clone())?;
    let entries: Vec<(Table, Vec<u8>, Vec<u8>)> = (0..100)
        .map(|index| {
            let table = if index % 2 == 0 {
                Table::Recording
            } else {
                Table::Facts
            };
            (
                table,
                bytes(&format!("single-sync/{index:03}")),
                bytes(&format!("payload-{index}")),
            )
        })
        .collect();
    let before = client.statistics()?;
    let written = client.write(&entries)?;
    let after = statistics_after_sync(&mut client, &before)?;
    if written != 100 {
        return Err(format!("写确认说写了 {written} 条，该是 100 条"));
    }
    let synced = counter_delta(&before, &after, Ticker::WalFileSynced)?;
    let keys_written = counter_delta(&before, &after, Ticker::NumberKeysWritten)?;
    // 101 个键：100 条数据加同一批里那一条「这个请求号已生效」的去重记录。
    if synced != 1 || keys_written != 101 {
        return Err(format!("一帧 100 条写，RocksDB 数到同步 WAL {synced} 次、写 {keys_written} 个键；该是 1 次、101 个（100 条加一条请求号记录）"));
    }
    let keys: Vec<(Table, Vec<u8>)> = entries
        .iter()
        .map(|(table, key, _)| (*table, key.clone()))
        .collect();
    let values = client.read(&keys)?;
    for ((_, key, value), received) in entries.iter().zip(&values) {
        if received.as_deref() != Some(value.as_slice()) {
            return Err(format!(
                "键 {} 读回 {received:?}",
                String::from_utf8_lossy(key)
            ));
        }
    }
    Ok("一帧 100 条写（跨录入、事实两张表）只同步 1 次 WAL、写 101 个键（100 条加一条请求号记录），读回逐条对上".to_string())
}

fn cell_prefix_scan(context: &CellContext) -> Result<String, String> {
    let server = start_test_server(context, &loopback_arguments(), &[])?;
    let mut client = SimpleClient::connect(&server.unix_route(), context.break_switches.clone())?;
    let mut entries: Vec<(Table, Vec<u8>, Vec<u8>)> = (0..5)
        .map(|index| {
            (
                Table::Conclusions,
                bytes(&format!("scan/alpha/{index}")),
                bytes(&format!("alpha-{index}")),
            )
        })
        .collect();
    entries.push((
        Table::Conclusions,
        bytes("scan/alphabet"),
        bytes("neighbour"),
    ));
    entries.extend((0..5).map(|index| {
        (
            Table::Conclusions,
            bytes(&format!("scan/beta/{index}")),
            bytes(&format!("beta-{index}")),
        )
    }));
    client.write(&entries)?;
    let (is_truncated, scanned) = client.scan(Table::Conclusions, b"scan/alpha/", 100)?;
    let scanned_keys: Vec<String> = scanned
        .iter()
        .map(|entry| String::from_utf8_lossy(&entry.key).into_owned())
        .collect();
    let expected_keys: Vec<String> = (0..5).map(|index| format!("scan/alpha/{index}")).collect();
    if is_truncated || scanned_keys != expected_keys {
        return Err(format!("扫 scan/alpha/ 回 {scanned_keys:?}（截断 {is_truncated}），该是 {expected_keys:?}（不截断）"));
    }
    for (index, entry) in scanned.iter().enumerate() {
        if entry.value != bytes(&format!("alpha-{index}")) {
            return Err(format!(
                "扫回的 {} 值不对：{}",
                scanned_keys[index],
                String::from_utf8_lossy(&entry.value)
            ));
        }
    }
    let (limited_truncated, limited) = client.scan(Table::Conclusions, b"scan/alpha/", 3)?;
    if !limited_truncated || limited.len() != 3 {
        return Err(format!(
            "条数上限 3 扫回 {} 条（截断 {limited_truncated}），该是 3 条、截断",
            limited.len()
        ));
    }
    let (_, other_table) = client.scan(Table::Facts, b"scan/alpha/", 100)?;
    let other_table_keys: Vec<String> = other_table
        .iter()
        .map(|entry| String::from_utf8_lossy(&entry.key).into_owned())
        .collect();
    if other_table_keys != expected_keys {
        return Err(format!(
            "用事实表的名义扫 scan/alpha/ 回 {other_table_keys:?}，该与结论表一样是 {expected_keys:?}（三张表同在默认列族、同一个键空间）"
        ));
    }
    Ok("前缀 scan/alpha/ 只回它下面的 5 条（不越到 scan/alphabet）、上限 3 时标截断、用别的表的名义扫回同样 5 条（三张表同一个键空间）".to_string())
}

fn cell_tcp_route_from_configuration(context: &CellContext) -> Result<String, String> {
    let configuration_directory = CellDirectory::create()?;
    let configuration = configuration_directory.path.join("multi-host.env");
    std::fs::write(
        &configuration,
        format!("# 自证用的多机配置\nHOSTS=\"127.0.0.1\"\n{NETWORK_ADDRESS_KEY}=\"127.0.0.1:0\"\n"),
    )
    .map_err(|error| format!("写不了自证的多机配置：{error}"))?;
    let network_arguments = vec![
        "--multi-host-configuration".to_string(),
        configuration.display().to_string(),
    ];
    let server = start_test_server(context, &network_arguments, &[])?;
    if !server.network_address.starts_with("127.0.0.1:") || server.network_address.ends_with(":0") {
        return Err(format!(
            "服务报的 TCP 地址是 {}，该是配置里的 127.0.0.1 加系统分的端口",
            server.network_address
        ));
    }
    let mut client =
        SimpleClient::connect(&server.network_route(), context.break_switches.clone())?;
    let entries: Vec<(Table, Vec<u8>, Vec<u8>)> = (0..10)
        .map(|index| {
            (
                Table::Facts,
                bytes(&format!("network/{index}")),
                bytes(&format!("fact-{index}")),
            )
        })
        .collect();
    if client.write(&entries)? != 10 {
        return Err("经 TCP 写 10 条，写确认的条数不对".to_string());
    }
    let mut keys: Vec<(Table, Vec<u8>)> = entries
        .iter()
        .map(|(table, key, _)| (*table, key.clone()))
        .collect();
    keys.push((Table::Facts, bytes("network/absent")));
    let values = client.read(&keys)?;
    for ((_, key), value) in keys.iter().zip(&values) {
        let wanted = entries
            .iter()
            .find(|(_, entry_key, _)| entry_key == key)
            .map(|(_, _, entry_value)| entry_value.as_slice());
        if value.as_deref() != wanted {
            return Err(format!(
                "经 TCP 读 {} 回 {value:?}，该是 {wanted:?}",
                String::from_utf8_lossy(key)
            ));
        }
    }
    let (_, scanned) = client.scan(Table::Facts, b"network/", 100)?;
    if scanned.len() != 10 {
        return Err(format!(
            "经 TCP 扫 network/ 回 {} 条，该是 10 条",
            scanned.len()
        ));
    }
    let frames_handled = counter(&client.statistics()?, FRAMES_HANDLED_COUNTER)?;
    if frames_handled < 4 {
        return Err(format!(
            "服务数到处理了 {frames_handled} 帧，经 TCP 至少发了 4 帧"
        ));
    }
    Ok(format!("TCP 地址取自多机配置的 {NETWORK_ADDRESS_KEY}（{}），经 TCP 成批写、成批点读、前缀扫、统计都对", server.network_address))
}

fn cell_missing_tcp_address_refused(context: &CellContext) -> Result<String, String> {
    let directory = CellDirectory::create()?;
    let configuration = directory.path.join("multi-host.env");
    std::fs::write(
        &configuration,
        "# 自证用的多机配置：没有服务的 TCP 地址\nHOSTS=\"127.0.0.1\"\n",
    )
    .map_err(|error| format!("写不了自证的多机配置：{error}"))?;
    let library = directory.path.join("library");
    let mut command = server_command(context, &directory.path, &library)?;
    command
        .arg("--multi-host-configuration")
        .arg(&configuration);
    let mut process = SpawnedProcess::spawn(command, StandardError::Capture)?;
    if let Ok(line) =
        process.wait_for_line_starting_with(LISTENING_LINE_PREFIX, Duration::from_secs(15))
    {
        return Err(format!(
            "多机配置里没有 {NETWORK_ADDRESS_KEY}，服务却起来了：{line}"
        ));
    }
    let status = process
        .wait_for_exit(Duration::from_secs(15))
        .ok_or("服务既没起来也没退出")?;
    let standard_error = process.standard_error_text();
    if status.code() != Some(5) || !standard_error.contains(NETWORK_ADDRESS_KEY) {
        return Err(format!(
            "服务退出码 {:?}（该是 5），标准错误：{standard_error}",
            status.code()
        ));
    }
    Ok(format!(
        "多机配置里没有 {NETWORK_ADDRESS_KEY} 时服务退 5、点名这个键"
    ))
}

fn cell_pipelined_out_of_order(context: &CellContext) -> Result<String, String> {
    let server = start_test_server(
        context,
        &loopback_arguments(),
        &["--selftest-delay", "1:400"],
    )?;
    let mut setup = SimpleClient::connect(&server.unix_route(), BreakSwitches::none())?;
    setup.write(&[
        (Table::Conclusions, bytes("pipe/slow"), bytes("slow-value")),
        (Table::Conclusions, bytes("pipe/fast"), bytes("fast-value")),
    ])?;
    let (mut sender, mut receiver) = connect(&server.unix_route(), context.break_switches.clone())
        .map_err(|error| format!("连不上服务：{error}"))?;
    receiver
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|error| format!("设不上读超时：{error}"))?;
    let read_one = |key: &'static str| {
        move |sequence: u64| {
            let mut builder = begin_point_read_frame(sequence, 1, 32);
            builder.key(Table::Conclusions, key.as_bytes());
            builder.finish()
        }
    };
    let slow_sequence = sender
        .send_frame(read_one("pipe/slow"))
        .map_err(|error| format!("发不出去：{error}"))?;
    let fast_sequence = sender
        .send_frame(read_one("pipe/fast"))
        .map_err(|error| format!("发不出去：{error}"))?;
    if slow_sequence != 1 {
        return Err(format!(
            "这条连接的第一帧序号是 {slow_sequence}，服务的延迟钉在序号 1 上"
        ));
    }
    let expected: HashMap<u64, &[u8]> = HashMap::from([
        (slow_sequence, b"slow-value".as_slice()),
        (fast_sequence, b"fast-value".as_slice()),
    ]);
    let mut arrival_sequences = Vec::new();
    for _ in 0..2 {
        let matched = receiver
            .receive()
            .map_err(|failure| format!("收回应失败：{failure}"))?;
        arrival_sequences.push(matched.response.sequence);
        let wanted = expected
            .get(&matched.request_sequence)
            .copied()
            .ok_or_else(|| format!("认领到没发过的序号 {}", matched.request_sequence))?;
        let is_expected = matches!(
            &matched.response.outcome,
            ResponseOutcome::PointValues(values)
                if values.len() == 1 && values[0].as_deref() == Some(wanted)
        );
        if !is_expected {
            return Err(format!(
                "序号 {} 的请求认领到的回应是 {:?}，该是 {}",
                matched.request_sequence,
                matched.response.outcome,
                String::from_utf8_lossy(wanted)
            ));
        }
    }
    if arrival_sequences != vec![fast_sequence, slow_sequence] {
        return Err(format!(
            "回应到达的序号次序是 {arrival_sequences:?}，服务没有乱序回应，这一格没测到认领"
        ));
    }
    Ok(format!("先发的帧（序号 {slow_sequence}）被服务压后 400 毫秒，回应按 {arrival_sequences:?} 乱序到达，按序号认领到各自的值"))
}

fn expect_connection_rejection(
    server: &TestServer,
    raw_bytes: &[u8],
    description: &str,
) -> Result<(), String> {
    let (mut sender, mut receiver) = connect(&server.unix_route(), BreakSwitches::none())
        .map_err(|error| format!("连不上服务：{error}"))?;
    receiver
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("设不上读超时：{error}"))?;
    sender
        .write_raw_bytes(raw_bytes)
        .map_err(|error| format!("写不出去：{error}"))?;
    let response = receiver
        .receive_unmatched()
        .map_err(|failure| format!("{description}：没收到拒绝（{failure}）"))?;
    let is_length_rejection = matches!(
        &response.outcome,
        ResponseOutcome::Rejected {
            status: RejectionStatus::FrameLengthRejected,
            ..
        }
    );
    if !is_length_rejection || response.sequence != 0 {
        return Err(format!(
            "{description}：回应是序号 {} 的 {:?}，该是序号 0 的帧长度拒绝",
            response.sequence, response.outcome
        ));
    }
    match receiver.receive_unmatched() {
        Err(ReceiveFailure::EndOfStream) => Ok(()),
        Ok(extra) => Err(format!("{description}：拒了之后连接没关，又收到 {extra:?}")),
        Err(failure) => Err(format!("{description}：拒了之后该读到流尾，读到 {failure}")),
    }
}

fn cell_frame_length_rejected(context: &CellContext) -> Result<String, String> {
    let server = start_test_server(context, &loopback_arguments(), &[])?;
    expect_connection_rejection(&server, &u32::MAX.to_le_bytes(), "声称 4 GiB 的帧")?;
    let mut short = 3_u32.to_le_bytes().to_vec();
    short.extend_from_slice(b"abc");
    expect_connection_rejection(&server, &short, "短于帧头的帧")?;
    let mut client = SimpleClient::connect(&server.unix_route(), context.break_switches.clone())?;
    client.statistics()?;
    Ok(
        "声称 4 GiB 与短于帧头的两种帧都回序号 0 的帧长度拒绝、随后关连接；新连接照常可用"
            .to_string(),
    )
}

fn cell_unknown_operation_rejected(context: &CellContext) -> Result<String, String> {
    let server = start_test_server(context, &loopback_arguments(), &[])?;
    let mut client = SimpleClient::connect(&server.unix_route(), context.break_switches.clone())?;
    let outcome = client.call(|sequence| encode_raw_request_frame(sequence, 0x7F, b"x"))?;
    let is_unknown_operation_rejection = matches!(
        &outcome,
        ResponseOutcome::Rejected {
            status: RejectionStatus::UnknownOperation,
            ..
        }
    );
    if !is_unknown_operation_rejection {
        return Err(format!(
            "操作码 0x7F 回的是 {outcome:?}，该是认不得的操作码拒绝"
        ));
    }
    client
        .statistics()
        .map_err(|message| format!("拒了认不得的操作码之后同一条连接不能用了：{message}"))?;
    Ok("操作码 0x7F 回状态 2 的拒绝，同一条连接接着能用".to_string())
}

fn cell_malformed_payload_rejected(context: &CellContext) -> Result<String, String> {
    let server = start_test_server(context, &loopback_arguments(), &[])?;
    let mut client = SimpleClient::connect(&server.unix_route(), context.break_switches.clone())?;
    let mut short_count = 5_u32.to_le_bytes().to_vec();
    for key in [b"one".as_slice(), b"two".as_slice()] {
        short_count.push(Table::Facts.code());
        short_count.extend_from_slice(&u32::try_from(key.len()).expect("三个字节").to_le_bytes());
        short_count.extend_from_slice(key);
    }
    let identity = b"malformed-payload-client";
    let mut trailing = vec![u8::try_from(identity.len()).expect("短身份")];
    trailing.extend_from_slice(identity);
    trailing.extend_from_slice(&1_u64.to_le_bytes());
    trailing.extend_from_slice(&1_u32.to_le_bytes());
    trailing.push(Table::Facts.code());
    for field in [b"malformed/trailing".as_slice(), b"v".as_slice()] {
        trailing.extend_from_slice(&u32::try_from(field.len()).expect("短字段").to_le_bytes());
        trailing.extend_from_slice(field);
    }
    trailing.extend_from_slice(b"junk");
    let mut unknown_table = 1_u32.to_le_bytes().to_vec();
    unknown_table.push(9);
    unknown_table.extend_from_slice(&1_u32.to_le_bytes());
    unknown_table.push(b'k');
    let cases = [
        (
            "声明 5 个键只给了 2 个",
            OperationCode::BatchedPointRead,
            short_count,
        ),
        ("成批写帧尾多 4 字节", OperationCode::BatchedWrite, trailing),
        ("表号 9", OperationCode::BatchedPointRead, unknown_table),
    ];
    for (description, operation, payload) in cases {
        let outcome = client
            .call(|sequence| encode_raw_request_frame(sequence, operation.code(), &payload))?;
        let is_malformed_rejection = matches!(
            &outcome,
            ResponseOutcome::Rejected {
                status: RejectionStatus::MalformedPayload,
                ..
            }
        );
        if !is_malformed_rejection {
            return Err(format!(
                "{description}：回的是 {outcome:?}，该是内容解不开的拒绝"
            ));
        }
    }
    let values = client.read(&[(Table::Facts, bytes("malformed/trailing"))])?;
    if values != vec![None] {
        return Err(format!(
            "帧尾多字节的那一帧被拒了，库里却有它的键：{values:?}"
        ));
    }
    Ok(
        "个数对不上、帧尾多字节、认不得的表三种都回状态 1，被拒的写一条都没落库，连接接着能用"
            .to_string(),
    )
}

fn cell_single_library_holder(context: &CellContext) -> Result<String, String> {
    let holder = start_test_server(context, &loopback_arguments(), &[])?;
    let second_directory = CellDirectory::create()?;
    let mut command = server_command(context, &second_directory.path, &holder.library)?;
    command.args(loopback_arguments());
    let mut second = SpawnedProcess::spawn(command, StandardError::Capture)?;
    if let Ok(line) =
        second.wait_for_line_starting_with(LISTENING_LINE_PREFIX, SERVER_READY_TIMEOUT)
    {
        return Err(format!("同一个库目录第二个服务也起来了：{line}"));
    }
    let status = second
        .wait_for_exit(Duration::from_secs(15))
        .ok_or("第二个服务既没起来也没退出")?;
    if status.code() != Some(3) {
        return Err(format!(
            "第二个服务退出码 {:?}，该是 3；标准错误：{}",
            status.code(),
            second.standard_error_text()
        ));
    }
    let mut client = SimpleClient::connect(&holder.unix_route(), context.break_switches.clone())?;
    client
        .statistics()
        .map_err(|message| format!("第二个服务被拒之后头一个不回话了：{message}"))?;
    Ok("同一个库目录第二个服务打不开库、退 3，头一个照常回话".to_string())
}

fn cell_bench_rate_judgment(context: &CellContext) -> Result<String, String> {
    let cases: [(f64, u64, RateJudgment); 6] = [
        (95.0, 100, RateJudgment::Met),
        (94.99, 100, RateJudgment::Missed),
        (0.0, 100, RateJudgment::Missed),
        (160_000.0, 160_000, RateJudgment::Met),
        (151_999.0, 160_000, RateJudgment::Missed),
        (60.0, 100, RateJudgment::Missed),
    ];
    for (achieved, target, expected) in cases {
        let judged = judge_achieved_rate(achieved, target, &context.break_switches);
        if judged != expected {
            return Err(format!(
                "达到 {achieved}、目标 {target} 判成 {judged:?}，该是 {expected:?}"
            ));
        }
    }
    let hundred: Vec<u32> = (1..=100).collect();
    let percentile_cases = [
        (hundred.as_slice(), 50, 50),
        (hundred.as_slice(), 99, 99),
        (&[7_u32][..], 99, 7),
        (&[][..], 50, 0),
    ];
    for (samples, percent, expected) in percentile_cases {
        let computed = percentile_microseconds(samples, percent);
        if computed != expected {
            return Err(format!(
                "{} 个样本的 p{percent} 算成 {computed}，该是 {expected}",
                samples.len()
            ));
        }
    }
    Ok(format!(
        "达标判法 {} 例与百分位 {} 例都对（达标线 95%）",
        cases.len(),
        percentile_cases.len()
    ))
}

fn bench_mode_arguments(mode: BenchMode) -> Vec<String> {
    let (target, states_per_block, duration_seconds) = match mode {
        BenchMode::FullBlock => ("2560", "256", "1"),
        BenchMode::SmallBlockPerFrame | BenchMode::SmallBlockBatched | BenchMode::Recording => {
            ("600", "6", "1")
        }
        BenchMode::TwoRoutes => ("1200", "6", "1"),
        BenchMode::FaultInjection => ("600", "6", "3"),
    };
    [
        "bench",
        "--mode",
        mode.name(),
        "--target-states-per-second",
        target,
        "--duration-seconds",
        duration_seconds,
        "--states-per-block",
        states_per_block,
        "--batch-blocks",
        "16",
        "--batch-milliseconds",
        "10",
        "--worker-threads",
        "2",
        "--drain-seconds",
        BENCH_CHILD_DRAIN_SECONDS,
    ]
    .iter()
    .map(|argument| (*argument).to_string())
    .collect()
}

/// 自证里的小压测不设会被同机负载打穿的限时：持续秒数之后等在途的块做完给到 600 秒，等压测器退出给到 900 秒；
/// 都只是上限，跑得快就不等。超了的失败信息里打当时的 loadavg。
const BENCH_CHILD_DRAIN_SECONDS: &str = "600";
const BENCH_CHILD_EXIT_TIMEOUT: Duration = Duration::from_secs(900);

const REQUIRED_BENCH_FIELDS: [&str; 20] = [
    "mode",
    "target_states_per_second",
    "achieved_states_per_second",
    "frames_per_second",
    "p50_ms",
    "p99_ms",
    "server_peak_rss_mib",
    "server_cpu_seconds",
    "load_average_start",
    "load_average_end",
    "load_average_threshold",
    "disk_device",
    "disk_busy_percent",
    "wal_syncs",
    "wal_sync_p50_ms",
    "wal_sync_p99_ms",
    "wal_sync_max_ms",
    "faults",
    "fault_recovery_max_ms",
    "result",
];

fn load_average_text() -> String {
    std::fs::read_to_string("/proc/loadavg").map_or_else(
        |error| format!("读不了 /proc/loadavg：{error}"),
        |text| text.trim().to_string(),
    )
}

fn run_bench_child(context: &CellContext, mode: BenchMode) -> Result<String, String> {
    let mut command = Command::new(&context.executable);
    command.args(bench_mode_arguments(mode));
    let mut process = SpawnedProcess::spawn(command, StandardError::Capture)?;
    let library_line =
        process.wait_for_line_starting_with(LIBRARY_LINE_PREFIX, SERVER_READY_TIMEOUT)?;
    let library = PathBuf::from(library_line.trim_start_matches(LIBRARY_LINE_PREFIX).trim());
    let status = process
        .wait_for_exit(BENCH_CHILD_EXIT_TIMEOUT)
        .ok_or_else(|| {
            format!(
                "{} {} 秒没跑完（此刻 /proc/loadavg：{}）",
                mode.name(),
                BENCH_CHILD_EXIT_TIMEOUT.as_secs(),
                load_average_text()
            )
        })?;
    let lines = process.remaining_lines(Duration::from_secs(5));
    let last = lines.last().cloned().unwrap_or_default();
    // 跑通的三种结局都认：达标退 0、没达标退 1、环境不干净退 4。小速率达不达标取决于这一刻盘的同步延迟与同机负载，
    // 达标判法与环境判法各由 bench-rate-judgment、bench-environment-judgment 拿已知答案核；这一格只判跑得通、末行齐、压测库删掉。
    let expected_result = match status.code() {
        Some(0) => "met",
        Some(1) => "missed",
        Some(ENVIRONMENT_DIRTY_EXIT_CODE) => "environment-dirty",
        other => {
            return Err(format!(
                "{} 退出码 {other:?}（该是 0 达标、1 没达标或 4 环境不干净），末行 {last}；此刻 /proc/loadavg：{}；标准错误：{}",
                mode.name(),
                load_average_text(),
                process.standard_error_text()
            ))
        }
    };
    if mode == BenchMode::FaultInjection && line_field(&last, "faults").as_deref() != Some("3") {
        return Err(format!("fault-injection 末行该是 faults=3：{last}"));
    }
    if !last.starts_with(&format!("{BENCH_LINE_PREFIX} "))
        || line_field(&last, "result").as_deref() != Some(expected_result)
    {
        return Err(format!(
            "{} 退出码 {:?}，末行却不是 result={expected_result} 的 {BENCH_LINE_PREFIX}：{last}",
            mode.name(),
            status.code()
        ));
    }
    for field in REQUIRED_BENCH_FIELDS {
        if line_field(&last, field).is_none() {
            return Err(format!("{} 末行缺 {field}=：{last}", mode.name()));
        }
    }
    if library.exists() {
        return Err(format!(
            "{} 跑完压测库 {} 还在",
            mode.name(),
            library.display()
        ));
    }
    Ok(format!(
        "{} {}/{} {expected_result}",
        mode.name(),
        line_field(&last, "achieved_states_per_second").unwrap_or_default(),
        line_field(&last, "target_states_per_second").unwrap_or_default()
    ))
}

fn cell_bench_runs_every_mode(context: &CellContext) -> Result<String, String> {
    let results: Vec<Result<String, String>> = thread::scope(|scope| {
        let handles: Vec<_> = BenchMode::ALL
            .iter()
            .map(|mode| scope.spawn(move || run_bench_child(context, *mode)))
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err("跑压测的线程 panic 了".to_string()))
            })
            .collect()
    });
    if results.len() != BenchMode::ALL.len() {
        return Err(format!(
            "派出 {} 种负载，收回 {} 个结果",
            BenchMode::ALL.len(),
            results.len()
        ));
    }
    let mut summaries = Vec::new();
    for result in results {
        summaries.push(result?);
    }
    Ok(format!(
        "{} 种负载都跑通（fault-injection 3 秒带 3 次故障，其余各 1 秒）、末行字段齐、压测库删掉（达到/目标每秒态数与结局：{}）",
        BenchMode::ALL.len(),
        summaries.join("、")
    ))
}

fn cell_bench_interrupted_removes_library(context: &CellContext) -> Result<String, String> {
    let mut command = Command::new(&context.executable);
    command.args([
        "bench",
        "--selftest-instance",
        "--mode",
        "small-block-per-frame",
        "--target-states-per-second",
        "600",
        "--duration-seconds",
        "60",
        "--worker-threads",
        "2",
    ]);
    let mut process = SpawnedProcess::spawn(command, StandardError::Capture)?;
    let library_line =
        process.wait_for_line_starting_with(LIBRARY_LINE_PREFIX, Duration::from_secs(30))?;
    let library = PathBuf::from(library_line.trim_start_matches(LIBRARY_LINE_PREFIX).trim());
    let started_line =
        process.wait_for_line_starting_with(STARTED_LINE_PREFIX, Duration::from_secs(30))?;
    let server_process = line_field(&started_line, "server_process")
        .ok_or_else(|| format!("开压行没有 server_process=：{started_line}"))?;
    thread::sleep(Duration::from_millis(300));
    process.send_signal(libc::SIGTERM)?;
    let status = process
        .wait_for_exit(Duration::from_secs(20))
        .ok_or("收了 SIGTERM 20 秒还没退出")?;
    let last = process
        .remaining_lines(Duration::from_secs(5))
        .last()
        .cloned()
        .unwrap_or_default();
    if library.exists() {
        // 弄坏开关打开时留下的压测库，这一格自己删掉，不留给下一趟。
        let _ = std::fs::remove_dir_all(&library);
        return Err(format!(
            "收了 SIGTERM 之后压测库 {} 还在",
            library.display()
        ));
    }
    if Path::new(&format!("/proc/{server_process}")).exists() {
        return Err(format!(
            "压测器退出之后它起的服务（进程 {server_process}）还在"
        ));
    }
    if status.code() != Some(3) || !last.contains(" result=failed") {
        return Err(format!(
            "收了 SIGTERM 之后退出码 {:?}（该是 3）、末行 {last}（该带 result=failed）",
            status.code()
        ));
    }
    Ok("压测器收了 SIGTERM：停下、删压测库、停掉它起的服务、末行 result=failed、退 3".to_string())
}

/// 带故障负载那一格给客户端追加日志的整理门槛：比三帧（每帧约 27 KB）还小。
const FAULT_BENCH_JOURNAL_COMPACTION_BYTES: u64 = 64 * 1024;
/// 带故障负载那一格让服务每组提交等满的毫秒数：确认要等满这个窗口才一批回来，每秒约 100 帧时在途总有约 20 帧没确认，
/// 远过整理门槛——「服务慢了、没确认的帧本身超过门槛」的局面这一格每一趟都走到，不靠这一刻盘忙不忙。
/// 每个确认都整份重写的写法在这里重写量约是追加量的几倍（窗口 100 毫秒时量到 3.8 倍，200 毫秒拉开得更远），
/// 修好的写法按门槛翻倍的算法至多两倍。
const FAULT_BENCH_GROUP_COMMIT_MILLISECONDS: &str = "200";
/// 带故障负载那一格等压测器出末行的上限：正常几十秒跑完，上限放宽到十分钟，只防真卡住；超了打 loadavg。
const FAULT_BENCH_EXIT_TIMEOUT: Duration = Duration::from_secs(600);

/// 带故障的压测在能走到「没确认的帧超过整理门槛」的规模上跑：限时内出末行、三次故障都注入并恢复、
/// 追加日志整理重写的字节至多是追加字节的两倍再加一个门槛（每个确认都整份重写的写法远超这个数）、进度行按阶段打出来。
fn cell_fault_injection_bench_finishes(context: &CellContext) -> Result<String, String> {
    let compaction_bytes = FAULT_BENCH_JOURNAL_COMPACTION_BYTES.to_string();
    let mut command = Command::new(&context.executable);
    command.args([
        "bench",
        "--selftest-instance",
        "--mode",
        "fault-injection",
        "--target-states-per-second",
        "20000",
        "--duration-seconds",
        "6",
        "--states-per-block",
        "6",
        "--batch-blocks",
        "64",
        "--batch-milliseconds",
        "10",
        "--worker-threads",
        "2",
        "--drain-seconds",
        BENCH_CHILD_DRAIN_SECONDS,
        "--progress-seconds",
        "1",
        "--journal-compaction-bytes",
        &compaction_bytes,
        "--group-commit-milliseconds",
        FAULT_BENCH_GROUP_COMMIT_MILLISECONDS,
        // 两分钟里计数一点不动才判卡住：正常的恢复（重连、重起开库）在机器忙时也是几秒到十几秒。
        "--stall-seconds",
        "120",
    ]);
    let mut process = SpawnedProcess::spawn(command, StandardError::Capture)?;
    let library_line =
        process.wait_for_line_starting_with(LIBRARY_LINE_PREFIX, SERVER_READY_TIMEOUT)?;
    let library = PathBuf::from(library_line.trim_start_matches(LIBRARY_LINE_PREFIX).trim());
    let Some(status) = process.wait_for_exit(FAULT_BENCH_EXIT_TIMEOUT) else {
        let seen = process.remaining_lines(Duration::from_secs(1));
        let last_progress = seen
            .iter()
            .rev()
            .find(|line| line.starts_with(PROGRESS_LINE_PREFIX))
            .cloned()
            .unwrap_or_default();
        drop(process);
        // 压测器被放手时收 SIGKILL，它的压测库这一格自己删掉，不留给下一趟。
        let _ = std::fs::remove_dir_all(&library);
        return Err(format!(
            "带故障的压测 {} 秒没出末行，卡住了（此刻 /proc/loadavg：{}）；最后一行进度：{last_progress}",
            FAULT_BENCH_EXIT_TIMEOUT.as_secs(),
            load_average_text()
        ));
    };
    let lines = process.remaining_lines(Duration::from_secs(5));
    let last = lines.last().cloned().unwrap_or_default();
    let expected_result = match status.code() {
        Some(0) => "met",
        Some(1) => "missed",
        Some(ENVIRONMENT_DIRTY_EXIT_CODE) => "environment-dirty",
        other => {
            // 压测器判了卡住的，带上它打的卡住行与不在 futex 上睡的线程（各线程等在哪）。
            let stall_lines: Vec<&str> = lines
                .iter()
                .map(String::as_str)
                .filter(|line| {
                    line.starts_with(STALLED_LINE_PREFIX)
                        || (line.starts_with(THREAD_LINE_PREFIX) && !line.contains("syscall=futex"))
                })
                .collect();
            return Err(format!(
                "退出码 {other:?}（该是 0、1 或 4），末行 {last}；此刻 /proc/loadavg：{}；标准错误：{}；卡住诊断：{}",
                load_average_text(),
                process.standard_error_text(),
                stall_lines.join(" | ")
            ));
        }
    };
    if line_field(&last, "result").as_deref() != Some(expected_result)
        || line_field(&last, "faults").as_deref() != Some("3")
    {
        return Err(format!(
            "末行该是 result={expected_result}、faults=3：{last}"
        ));
    }
    let field_number = |name: &str| -> Result<u64, String> {
        line_field(&last, name)
            .and_then(|text| text.parse().ok())
            .ok_or_else(|| format!("末行没有数字的 {name}=：{last}"))
    };
    let appended = field_number("journal_appended_bytes")?;
    let rewritten = field_number("journal_rewritten_bytes")?;
    let compactions = field_number("journal_compactions")?;
    let rewrite_limit = appended * 2 + FAULT_BENCH_JOURNAL_COMPACTION_BYTES;
    if rewritten > rewrite_limit {
        return Err(format!(
            "追加日志整理了 {compactions} 次、重写 {rewritten} 字节，超过追加的 {appended} 字节的两倍再加门槛（{rewrite_limit}）：\
             没确认的帧超过门槛之后每个确认都在整份重写（满速率时客户端整个时间花在重写与同步上，带故障的压测就卡住）"
        ));
    }
    if compactions == 0 {
        return Err(format!(
            "追加日志一次都没整理（追加 {appended} 字节、门槛 {FAULT_BENCH_JOURNAL_COMPACTION_BYTES}）：这一格没走到要判的局面"
        ));
    }
    let progress_lines: Vec<&String> = lines
        .iter()
        .filter(|line| line.starts_with(PROGRESS_LINE_PREFIX))
        .collect();
    let phases: Vec<String> = progress_lines
        .iter()
        .filter_map(|line| line_field(line, "phase"))
        .collect();
    if progress_lines.is_empty()
        || progress_lines.iter().any(|line| {
            line_field(line, "client_activity").is_none()
                || line_field(line, "completed_blocks").is_none()
        })
        || !phases
            .iter()
            .any(|phase| phase == "faults-done" || phase == "draining")
    {
        return Err(format!(
            "进度行不齐（{} 行，阶段：{phases:?}）：每行要有 phase=、completed_blocks=、client_activity=，走完故障调度要打到 faults-done 或 draining",
            progress_lines.len()
        ));
    }
    if library.exists() {
        return Err(format!("跑完压测库 {} 还在", library.display()));
    }
    let mut distinct_phases = phases.clone();
    distinct_phases.dedup();
    Ok(format!(
        "带故障的压测出了末行 result={expected_result}、faults=3、fault_recovery_max_ms={}；追加日志整理 {compactions} 次、\
         重写 {rewritten} 字节 ≤ 追加 {appended} 字节的两倍加门槛；进度 {} 行，阶段依次 {}",
        line_field(&last, "fault_recovery_max_ms").unwrap_or_default(),
        progress_lines.len(),
        distinct_phases.join("→")
    ))
}

fn selected_cells() -> Result<Vec<&'static Cell>, String> {
    let Ok(selection) = std::env::var(CELL_SELECTION_VARIABLE) else {
        return Ok(CELLS.iter().collect());
    };
    let mut selected = Vec::new();
    for name in selection
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        let cell = CELLS.iter().find(|cell| cell.name == name).ok_or_else(|| {
            let known: Vec<&str> = CELLS.iter().map(|cell| cell.name).collect();
            format!(
                "{CELL_SELECTION_VARIABLE} 里的格「{name}」认不得；认得的：{}",
                known.join("、")
            )
        })?;
        selected.push(cell);
    }
    Ok(selected)
}

/// 自证入口。
pub fn run_selftest() -> i32 {
    let break_switches = match BreakSwitches::from_environment() {
        Ok(switches) => switches,
        Err(message) => {
            eprintln!("  ✗ {message}");
            eprintln!("  → 怎么办：改对 {BREAK_VARIABLE} 的值，或不设它");
            return 2;
        }
    };
    let cells = match selected_cells() {
        Ok(cells) => cells,
        Err(message) => {
            eprintln!("  ✗ {message}");
            eprintln!("  → 怎么办：照上面列的格名改 {CELL_SELECTION_VARIABLE}，或不设它跑全部格");
            return 2;
        }
    };
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("  ✗ 找不到自己的可执行文件：{error}");
            eprintln!("  → 怎么办：在能读 /proc/self/exe 的环境里跑");
            return 2;
        }
    };
    let context = CellContext {
        executable,
        break_switches,
        placement: CellPlacement::RunnerThread {
            cells_in_process: cells.len(),
        },
    };
    let runner_process = std::process::id();
    let results: Vec<Result<String, String>> = thread::scope(|scope| {
        let handles: Vec<_> = cells
            .iter()
            .map(|cell| scope.spawn(|| run_cell_isolated(&context, cell, runner_process)))
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err("跑这一格的线程 panic 了".to_string()))
            })
            .collect()
    });
    if results.len() != cells.len() {
        eprintln!("  ✗ 派出 {} 格，收回 {} 个结果", cells.len(), results.len());
        eprintln!("  → 怎么办：自证自己的收束坏了，看 selftest.rs 的 run_selftest");
        return 1;
    }
    let mut failed_cells = Vec::new();
    for (cell, result) in cells.iter().zip(&results) {
        match result {
            Ok(summary) => println!("  ✓ 自证 {}：{summary}", cell.name),
            Err(why) => {
                println!("  ✗ 自证 {}：{why}", cell.name);
                println!("  → 怎么办：{}", cell.remedy);
                failed_cells.push(cell.name);
            }
        }
    }
    let passed = cells.len() - failed_cells.len();
    if failed_cells.is_empty() {
        println!("  ✓ prune-server 自证 {} 格全绿", cells.len());
    } else {
        println!(
            "  ✗ prune-server 自证 {} 格里红了 {} 格：{}",
            cells.len(),
            failed_cells.len(),
            failed_cells.join("、")
        );
        println!("  → 怎么办：照各红格的「怎么办」改；只重跑红的格：{CELL_SELECTION_VARIABLE}={} prune-server --selftest", failed_cells.join(","));
    }
    let failed_list = if failed_cells.is_empty() {
        "-".to_string()
    } else {
        failed_cells.join(",")
    };
    println!(
        "{SELFTEST_LINE_PREFIX} cells={} passed={passed} failed={} failed_cells={failed_list}",
        cells.len(),
        failed_cells.len()
    );
    i32::from(!failed_cells.is_empty())
}

/// 跑一格：另起一个进程只跑它（文件头「每格各在自己的进程里」）；弄坏开关 cell-runs-in-runner-process 下，
/// 判这一条的那一格留在这个线程里跑。
fn run_cell_isolated(
    context: &CellContext,
    cell: &Cell,
    runner_process: u32,
) -> Result<String, String> {
    if cell.name == OWN_PROCESS_CELL_NAME
        && context
            .break_switches
            .is_enabled(BreakSwitch::CellRunsInRunnerProcess)
    {
        return (cell.function)(context);
    }
    run_cell_in_own_process(context, cell, runner_process)
}

/// 起 `selftest-cell` 进程跑这一格，收齐它的标准输出、等它退出，按末行与退出码回这一格的结局。
/// 不另设限时：各格自己带限时（会卡住的格到点置停标记、判红）。它的标准错误接在自证的标准错误上。
fn run_cell_in_own_process(
    context: &CellContext,
    cell: &Cell,
    runner_process: u32,
) -> Result<String, String> {
    let mut command = Command::new(&context.executable);
    command
        .arg("selftest-cell")
        .arg("--selftest-instance")
        .arg(RUNNER_PROCESS_ARGUMENT)
        .arg(runner_process.to_string())
        .arg(cell.name);
    let mut process = SpawnedProcess::spawn(command, StandardError::Inherit)?;
    let (status, lines) = process.wait_for_exit_with_all_lines()?;
    let Some((last, explanation_lines)) = lines.split_last() else {
        return Err(format!(
            "这一格的进程 {status} 退出，一行标准输出都没打（它的标准错误在上面）"
        ));
    };
    let explanation = explanation_lines.join("\n");
    let verdict = if last.starts_with(CELL_RESULT_LINE_PREFIX) {
        line_field(last, "result")
    } else {
        None
    };
    if verdict.as_deref() == Some("passed") && status.code() == Some(0) {
        return Ok(explanation);
    }
    if verdict.as_deref() == Some("failed") && status.code() == Some(1) {
        return Err(explanation);
    }
    Err(format!(
        "这一格的进程 {status} 退出，末行该是「{CELL_RESULT_LINE_PREFIX} result=passed」退 0 或「… result=failed」退 1，\
         是「{last}」；之前打的：{explanation}（它的标准错误在上面）"
    ))
}

/// `selftest-cell` 的参数：跑格的进程号与格名。
fn parse_cell_arguments(arguments: &[String]) -> Result<(u32, String), String> {
    let mut runner_process = None;
    let mut name = None;
    let mut remaining = arguments.iter();
    while let Some(argument) = remaining.next() {
        match argument.as_str() {
            "--selftest-instance" => {}
            RUNNER_PROCESS_ARGUMENT => {
                let value = remaining
                    .next()
                    .ok_or_else(|| format!("{RUNNER_PROCESS_ARGUMENT} 后面缺进程号"))?;
                runner_process =
                    Some(value.parse::<u32>().map_err(|_| {
                        format!("{RUNNER_PROCESS_ARGUMENT} 的进程号不是整数：{value}")
                    })?);
            }
            other if name.is_none() && !other.starts_with("--") => name = Some(other.to_string()),
            other => return Err(format!("认不得的参数：{other}")),
        }
    }
    let runner_process =
        runner_process.ok_or_else(|| format!("缺 {RUNNER_PROCESS_ARGUMENT} <跑格的进程号>"))?;
    let name = name.ok_or_else(|| "缺格名".to_string())?;
    Ok((runner_process, name))
}

/// 每格进程的入口（`prune-server selftest-cell --selftest-instance --runner-process <跑格的进程号> <格名>`，只由 `--selftest` 起）：
/// 在这个进程里只跑这一格，标准输出先打这一格的说明，末行 `PRUNE_SERVER_CELL_RESULT result=passed|failed`；
/// 绿退 0，红退 1，参数、格名或开关不对退 2。
pub fn run_selftest_cell(arguments: &[String]) -> i32 {
    let (runner_process, name) = match parse_cell_arguments(arguments) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("  ✗ selftest-cell：{message}");
            eprintln!("  → 怎么办：这个子命令只由 prune-server --selftest 起，参数照 selftest.rs 的 run_cell_in_own_process");
            return 2;
        }
    };
    let Some(cell) = CELLS.iter().find(|cell| cell.name == name) else {
        eprintln!("  ✗ selftest-cell：格「{name}」认不得");
        eprintln!("  → 怎么办：格名照 selftest.rs 的 CELLS");
        return 2;
    };
    let break_switches = match BreakSwitches::from_environment() {
        Ok(switches) => switches,
        Err(message) => {
            eprintln!("  ✗ {message}");
            eprintln!("  → 怎么办：改对 {BREAK_VARIABLE} 的值，或不设它");
            return 2;
        }
    };
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("  ✗ 找不到自己的可执行文件：{error}");
            eprintln!("  → 怎么办：在能读 /proc/self/exe 的环境里跑");
            return 2;
        }
    };
    let context = CellContext {
        executable,
        break_switches,
        placement: CellPlacement::Own {
            runner: runner_process,
        },
    };
    match (cell.function)(&context) {
        Ok(summary) => {
            println!("{summary}");
            println!("{CELL_RESULT_LINE_PREFIX} result=passed");
            0
        }
        Err(why) => {
            println!("{why}");
            println!("{CELL_RESULT_LINE_PREFIX} result=failed");
            1
        }
    }
}

/// 这一格在跑格的进程为它另起的进程里单独跑：父进程是跑格的进程，自己不是它（文件头「每格各在自己的进程里」）。
fn cell_runs_alone_in_own_process(context: &CellContext) -> Result<String, String> {
    let own = std::process::id();
    match context.placement {
        CellPlacement::Own { runner } => {
            // SAFETY: getppid 不收参数、总是成功。
            let parent = unsafe { libc::getppid() };
            if u32::try_from(parent).ok() != Some(runner) || own == runner {
                return Err(format!(
                    "这一格的进程 {own} 的父进程是 {parent}，该是跑格的进程 {runner}、且不是它自己"
                ));
            }
            Ok(format!(
                "这一格在进程 {own} 里单独跑，父进程是跑格的进程 {runner}：别的格起子进程时复制不到这个进程的描述符"
            ))
        }
        CellPlacement::RunnerThread { cells_in_process } => Err(format!(
            "这一格跑在跑格的进程 {own} 自己的线程里，这个进程里跑着 {cells_in_process} 格：别的格 fork 出来、还没 exec 的子进程\
             复制着这个进程的描述符，追加日志的锁文件关掉再开会被那份复制挡住、报「被别的客户端开着」"
        )),
    }
}

/// 起一遍子自证，回它报的红格（`-` 是全绿）与它的 ✗ 行。
fn run_child_selftest(
    executable: &Path,
    break_switch: Option<BreakSwitch>,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut command = Command::new(executable);
    command
        .arg("--selftest")
        .env_remove(CELL_SELECTION_VARIABLE);
    match break_switch {
        Some(switch) => command.env(BREAK_VARIABLE, switch.name()),
        None => command.env_remove(BREAK_VARIABLE),
    };
    let output = command
        .output()
        .map_err(|error| format!("起不了子自证：{error}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let failure_lines: Vec<String> = text
        .lines()
        .filter(|line| line.trim_start().starts_with('✗'))
        .map(str::to_string)
        .collect();
    let last = text.lines().last().unwrap_or_default();
    if !last.starts_with(SELFTEST_LINE_PREFIX) {
        return Err(format!(
            "子自证的末行不是 {SELFTEST_LINE_PREFIX}：{last}；标准错误：{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let failed = line_field(last, "failed_cells")
        .ok_or_else(|| format!("子自证末行没有 failed_cells=：{last}"))?;
    let failed_cells = if failed == "-" {
        Vec::new()
    } else {
        failed.split(',').map(str::to_string).collect()
    };
    Ok((failed_cells, failure_lines))
}

/// 弄坏开关复跑入口。
pub fn run_break_sweep() -> i32 {
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("  ✗ 找不到自己的可执行文件：{error}");
            eprintln!("  → 怎么办：在能读 /proc/self/exe 的环境里跑");
            return 2;
        }
    };
    for switch in BreakSwitch::ALL {
        if !CELLS.iter().any(|cell| cell.name == switch.red_cell()) {
            eprintln!(
                "  ✗ 弄坏开关 {} 点名的格 {} 不在自证的格表里",
                switch.name(),
                switch.red_cell()
            );
            eprintln!("  → 怎么办：改 break_switches.rs 的 red_cell 或 selftest.rs 的 CELLS，两边格名逐字相同");
            return 1;
        }
    }
    let baseline_green = match run_child_selftest(&executable, None) {
        Ok((failed_cells, _)) if failed_cells.is_empty() => {
            println!("  ✓ 不设开关：自证 {} 格全绿", CELLS.len());
            true
        }
        Ok((failed_cells, failure_lines)) => {
            println!("  ✗ 不设开关自证就红了：{}", failed_cells.join("、"));
            for line in failure_lines {
                println!("      {line}");
            }
            println!("  → 怎么办：先让 prune-server --selftest 全绿，再复跑弄坏开关");
            false
        }
        Err(message) => {
            println!("  ✗ 不设开关的那一遍没跑成：{message}");
            println!("  → 怎么办：看上面的末行与标准错误");
            false
        }
    };
    let mut matched = 0_usize;
    for switch in BreakSwitch::ALL {
        match run_child_selftest(&executable, Some(switch)) {
            Ok((failed_cells, _)) if failed_cells == vec![switch.red_cell().to_string()] => {
                matched += 1;
                println!(
                    "  ✓ 弄坏开关 {}：只红在点名的 {}",
                    switch.name(),
                    switch.red_cell()
                );
            }
            Ok((failed_cells, failure_lines)) => {
                println!(
                    "  ✗ 弄坏开关 {}：红的格是 {:?}，该是只红 {}",
                    switch.name(),
                    failed_cells,
                    switch.red_cell()
                );
                for line in failure_lines {
                    println!("      {line}");
                }
                println!("  → 怎么办：没红是那一格判不出这一处坏；多红了是开关弄坏的不止一处，或别的格也依赖这一处");
            }
            Err(message) => {
                println!("  ✗ 弄坏开关 {}：子自证没跑成：{message}", switch.name());
                println!("  → 怎么办：看子自证的末行与标准错误");
            }
        }
    }
    let all_matched = matched == BreakSwitch::ALL.len() && baseline_green;
    if all_matched {
        println!(
            "  ✓ 弄坏开关 {} 项逐项打开，各自只红在点名的那一格",
            BreakSwitch::ALL.len()
        );
    } else {
        println!(
            "  ✗ 弄坏开关 {} 项里对上 {matched} 项（不设开关那一遍 {}）",
            BreakSwitch::ALL.len(),
            if baseline_green { "绿" } else { "红" }
        );
        println!("  → 怎么办：照上面每一项的说明改");
    }
    println!(
        "{SWEEP_LINE_PREFIX} switches={} matched={matched} baseline={}",
        BreakSwitch::ALL.len(),
        if baseline_green { "green" } else { "red" }
    );
    i32::from(!all_matched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::{MAXIMUM_SOCKET_PATH_BYTES, SHORT_TEMPORARY_ROOT};

    /// `TMPDIR` 设得比 socket 路径上限还长时，一格的目录退到短根底下，最长的那条 socket 路径仍不超上限。
    #[test]
    fn cell_directory_falls_back_to_the_short_root_when_the_temporary_root_is_longer_than_a_socket_path_may_be(
    ) {
        let long_root =
            Path::new(SHORT_TEMPORARY_ROOT).join("r".repeat(MAXIMUM_SOCKET_PATH_BYTES + 20));

        let directory = CellDirectory::create_under(Some(long_root.clone().into_os_string()))
            .unwrap_or_else(|message| panic!("临时根放不下，短根放得下，却建不成：{message}"));

        assert!(
            !directory.path.starts_with(&long_root),
            "临时根 {} 放不下 socket 全路径，目录该改建在 {SHORT_TEMPORARY_ROOT} 底下，实际是 {}",
            long_root.display(),
            directory.path.display()
        );
        let socket_path = directory.path.join(LONGEST_SOCKET_RELATIVE_PATH_IN_A_CELL);
        assert!(
            socket_path.as_os_str().len() <= MAXIMUM_SOCKET_PATH_BYTES,
            "socket 全路径 {} 超过 {MAXIMUM_SOCKET_PATH_BYTES} 字节",
            socket_path.display()
        );
    }
}
