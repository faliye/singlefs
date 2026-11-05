// admission: always 每跑一次都是新的吞吐与延迟观测，压测库每趟新建、跑完删，上一次的数不替这一次作保
// run-condition: command python3
//! prune-server-bench：prune-server 的压测器（records/2026-11-01-判定库服务化.md 批次 2 起，批次 3 加了容错客户端、故障注入与环境诊断）。
//!
//! 用法：
//!   prune-server-bench --mode <full-block|small-block-per-frame|small-block-batched|recording|two-routes|fault-injection>
//!       --target-states-per-second <数> --duration-seconds <数>
//!       [--states-per-block <数，full-block 缺省 65536、其余缺省 6>] [--batch-blocks <数，缺省 512>] [--batch-milliseconds <数，缺省 20>]
//!       [--layout <per-state|per-block，缺省 per-state>] [--tcp <服务的 TCP 监听地址，缺省 127.0.0.1:0>]
//!       [--worker-threads <服务的工作线程数，缺省本机核数>] [--maximum-in-flight-frames <数，缺省 256>] [--drain-seconds <数，缺省 30>]
//!       [--maximum-load-average <数，缺省本机核数的一半>] [--group-commit-milliseconds <数>] [--group-commit-frames <数>]
//!       [--progress-seconds <数，缺省 5>] [--stall-seconds <数，缺省 300>] [--journal-compaction-bytes <数，缺省 67108864>]
//!   每种负载是什么、怎么计量、诊断怎么量，在 `src/bench.rs` 文件头；进度行与卡住诊断在 `src/bench_progress.rs` 文件头。
//!
//! 它在 `${TMPDIR:-/tmp}` 下建 `prune-server-bench-<进程号>-<纳秒>/`（打一行 `PRUNE_SERVER_BENCH_LIBRARY <路径>`），起自己这个可执行文件的
//! `serve` 当服务，库与 socket 都在那个目录里；跑完（成功、失败、收 SIGTERM / SIGINT）停服务、删目录。被 SIGKILL 的删不了，目录名带进程号，手动删。
//!
//! 输出：跑的过程中每隔 `--progress-seconds` 秒与阶段一变时每一路一行 `PRUNE_SERVER_BENCH_PROGRESS …`（阶段、已完成块数、帧数、
//! 客户端停在哪一步）；`--stall-seconds` 秒里计数一点不动判卡住，打 `PRUNE_SERVER_BENCH_STALLED …` 与按线程的
//! `PRUNE_SERVER_BENCH_THREAD …`（各线程在内核里等在哪），这一趟判失败。跑完：fault-injection 每次故障一行
//! `PRUNE_SERVER_BENCH_FAULT kind=… injected_at_seconds=… recovery_ms=…`，每一路一行
//! `PRUNE_SERVER_BENCH_ROUTE …`（带 reconnections、duplicate_responses、busy_responses、journal_appended_bytes、journal_compactions、
//! journal_rewritten_bytes）、服务端计数一行 `PRUNE_SERVER_BENCH_SERVER_COUNTERS …`，末行
//! `PRUNE_SERVER_BENCH mode=… target_states_per_second=… achieved_states_per_second=… frames_per_second=… p50_ms=… p99_ms=…
//!  server_peak_rss_mib=… server_cpu_seconds=… records_per_second=… layout=… states_per_block=… batch_blocks=… batch_milliseconds=…
//!  duration_seconds=… elapsed_seconds=… routes=… required_percent=… load_average_start=… load_average_end=… load_average_threshold=…
//!  disk_device=… disk_busy_percent=… wal_syncs=… wal_sync_p50_ms=… wal_sync_p99_ms=… wal_sync_max_ms=… faults=… fault_recovery_max_ms=…
//!  client_reconnections=… duplicate_responses=… busy_responses=… journal_appended_bytes=… journal_compactions=… journal_rewritten_bytes=…
//!  result=<met|missed|environment-dirty|failed>`。
//!   达标：achieved_states_per_second ≥ target_states_per_second × 95%（required_percent）。
//!   环境不干净：起跑前或跑完时 /proc/loadavg 的 1 分钟值超过 load_average_threshold，不判达标没达标。
//!
//! 退出码：0 达标；1 没达标；2 用法错；3 没跑成（服务起不来、回应不对、读回逐字节对不上、完整性闸对不上、帧序键不对、故障没注入上或没恢复、
//! 判了卡住、被打断、压测库删不掉）；4 环境不干净（result=environment-dirty）；78 准入或运行条件不满足。
//! 结局的先后：没跑成 > 环境不干净 > 达标 / 没达标。
//!
//! 弄坏开关：压测器带 --selftest-instance 时认 PRUNE_SERVER_BREAK 里的 bench-judgment-lenient、bench-library-kept-on-interrupt、
//! bench-load-threshold-ignored，并把 journal-compacts-on-every-acknowledgement 交给它的容错客户端（别的客户端开关不交），
//! 逐项弄坏哪一处、自证哪一格红，表在 `src/bin/prune_server.rs` 文件头；自证由 `prune-server --selftest` 跑。

fn main() {
    std::process::exit(prune_server::command_line::run(
        prune_server::command_line::ProgramName::Bench,
    ));
}
