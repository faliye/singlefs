//! 弄坏开关 `PRUNE_SERVER_BREAK=<项>[,<项>…]`：只给自证证红用（`.claude/rules/test-script-discipline.md`「测试脚本自己也要被测试证明有效」）。
//! 服务只在带 `--selftest-instance` 起时读它，压测器同样只在带 `--selftest-instance` 时读，自证进程自己读它弄坏客户端与判法；平常跑时不认。
//! 每一项弄坏哪一处、打开后自证哪一格红，表在 `BreakSwitch::red_cell` 与两个二进制的文件头，两处逐项相同。

/// 环境变量名。
pub const BREAK_VARIABLE: &str = "PRUNE_SERVER_BREAK";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BreakSwitch {
    /// 成批点读退回逐键 `get`，不调 `multi_get`。
    MultiGetPerKey,
    /// 成批写的每一条各自一个同步写批。
    WriteSyncPerEntry,
    /// 按前缀扫走出前缀之后不停，扫到条数上限为止。
    PrefixScanOverruns,
    /// 多机配置里没有 TCP 地址时不拒，悄悄退回回环地址。
    MissingNetworkAddressDefaultsToLoopback,
    /// 客户端不核回应序号，回应按到达先后配给最早发出的请求。
    ResponseSequenceUnchecked,
    /// 服务不判帧长度，照声称的长度读下去。
    FrameLengthUnchecked,
    /// 服务把认不得的操作码当成完成、回一个空的成功。
    UnknownOperationAccepted,
    /// 服务解帧时不判帧尾多出来的字节。
    TrailingBytesAccepted,
    /// 压测达标线从目标的 95% 放到 50%。
    BenchJudgmentLenient,
    /// 压测器被打断时不删压测库。
    BenchLibraryKeptOnInterrupt,
    /// 对端关了连接或读写出错之后（帧超时那一种不算），客户端不重放在途没确认的写帧，当成已确认从追加日志里勾掉（悄悄丢帧）。
    UnacknowledgedFramesDroppedOnReconnect,
    /// 服务先回写确认、过 2 秒才同步 WAL（确认早于落盘）。
    AcknowledgeBeforeSync,
    /// 服务不按（身份，请求号）去重，重复的写帧再写一遍。
    DuplicateRequestsReapplied,
    /// 客户端把重复到达的回应（序号已经回过）当成协议错误。
    DuplicateResponsesFatal,
    /// 客户端不给在途帧设超时，半开的连接一直等下去。
    FrameTimeoutIgnored,
    /// 客户端的 TCP 连接不开 keepalive。
    KeepaliveNotSet,
    /// 重连的间隔不往上翻倍，一直是起始间隔。
    ReconnectBackoffFlat,
    /// 重连的间隔不加抖动。
    ReconnectWithoutJitter,
    /// 重连没有次数与总时长上限。
    ReconnectUnbounded,
    /// 读失败（连不上、忙到上限）时回「键都不在」，不报错。
    ReadFailureReportedAsAbsent,
    /// 客户端不把写帧记进本地追加日志。
    UnacknowledgedFramesNotJournaled,
    /// 服务不设待处理帧数的上限，不回忙，帧都排进队列。
    BusyReplacedByUnboundedQueue,
    /// 客户端收到忙立刻重发，不等服务给的毫秒数。
    BusyRetryWithoutDelay,
    /// 认领不判别人持有，谁来都认领到。
    ClaimNotExclusive,
    /// 认领的租期到了也不作废。
    ClaimLeaseNeverExpires,
    /// 写结论时不释放认领，认领也不看结论在不在。
    ClaimKeptAfterConclusion,
    /// 服务不跨帧合并同步，一帧一次 WAL 同步。
    GroupCommitDisabled,
    /// 压测器不看 loadavg，环境不干净也照判达标没达标。
    BenchLoadThresholdIgnored,
    /// 客户端追加日志一过整理门槛就整理，不看整理完还剩多少：没确认的帧本身超过门槛时，此后每个确认都整份重写一遍
    /// 再同步（批次 3 的写法；按压测 2138 带故障那一条的参数复跑时，客户端线程就停在整理文件的 fsync 上）。
    JournalCompactsOnEveryAcknowledgement,
    /// 故障注入代理在服务那头没了（读到流尾、写出错）时只半关客户端那头的写、不再读客户端的请求、连接不关：
    /// 客户端接着写一帧大过发送缓冲的帧就一直堵着（批次 3 的代理就是这样）。
    FaultProxyHalfClosesOnServerDeath,
    /// `control install --dry-run` 照样写文件（单元、目录、密钥）。
    ControlDryRunWrites,
    /// `control install` 起了单元之后不回读服务通不通，直接说装好了。
    ControlInstallReadbackSkipped,
    /// `control start` 只看 systemctl 的退出码，不回读 is-active 与服务通不通。
    ControlStartReadbackSkipped,
    /// `control stop` 只看 systemctl 的退出码，不回读 is-active 与 cgroup.procs。
    ControlStopReadbackSkipped,
    /// 客户端报到时任务号报空，不读 `TASK_SERVER_TASK`。
    ClientAnnouncementOmitsTask,
    /// 服务不把报到记进连接的登记（列出来的客户端没有身份）。
    AnnouncementIgnored,
    /// `control stop` 不写停服务记录。
    ControlStopRecordSkipped,
    /// 服务宣布停服务时不给连着的客户端发「服务停了」。
    StopNoticeNotSent,
    /// 客户端收到「服务停了」当成连接断了，照常重连重试。
    StopNoticeRetried,
    /// 连不上服务（重连到上限）时退出码给 3，不给「服务不可用」那一个码。
    UnreachableExitCodeGeneric,
    /// 服务出了自身的错不写故障行。
    FaultLineNotWritten,
    /// `control watch` 不看单元在不在跑。
    WatchIgnoresUnitState,
    /// `control watch` 不看服务通不通。
    WatchIgnoresHealth,
    /// `control watch` 不看故障日志。
    WatchIgnoresFaultLog,
    /// `control watch` 把起之前就有的故障行也当成新的。
    WatchCountsOldFaultLines,
    /// 容错客户端的连接不设写超时，对端不读时一直堵在写里。
    ClientWriteTimeoutUnset,
    /// 故障注入代理在服务那头没了时只关客户端那头的读（shutdown(SHUT_RD)）、连接留着：
    /// TCP 上客户端接着写不会出错，写满发送窗口就堵住（unix socket 上对端的写照样出错）。
    FaultProxyOnlyStopsReadingOnServerDeath,
    /// 已生效的请求号那几行不带键前缀 `F0`，落进客户端的键空间。
    BookkeepingKeyTagDropped,
    /// 客户端读写、认领 0xF0–0xFF 起头的键不拒，按前缀扫走到记账那一段也不停。
    ServiceKeysOpenToClients,
    /// 开库不判库里有没有默认之外的列族，有就一起打开。
    LibraryExtraColumnFamiliesAccepted,
    /// 把 RocksDB 拒开的每一种错都报成「库被持有」（退 3），不分选项对不上。
    LibraryOpenFailureReportedAsLocked,
    /// 库目录里有文件却没有 CURRENT 时不拒，交给 RocksDB 在里面建库。
    LibraryWithoutCurrentFileOpened,
    /// `control check` 不打库目录。
    ControlCheckOmitsLibrary,
    /// 客户端追加日志整理时不写请求号高水位：全确认之后整理、重开，请求号从小号分起，新写被服务当成早已生效的重发。
    JournalCompactionDropsHighWater,
    /// 服务同步 WAL 失败之后不退出、照常服务：去重查到只在内存里生效的请求号，重发同一请求回成功。
    SyncFailureKeptServing,
    /// 服务把回应交给写线程的队就放待处理名额、回应不记进额度：对端不读时回应在队里无界地积。
    ResponseSlotReleasedOnEnqueue,
    /// 服务照声明的长度先分配、读完帧体再过准入：没收完的帧占的内存不记进额度。
    FrameBodyReadBeforeAdmission,
    /// 客户端追加日志的排他锁加在日志本身上：整理改名换了 inode，卡在开与加锁之间的第二个打开者锁上旧 inode、写进没人读的旧文件。
    JournalLocksRenamedFile,
    /// 客户端发帧之前不核帧长，超过帧长上限照发；服务回「帧长度不对」当连接断了，重连重发（改前的样子）。
    OversizeFrameSentAndRetried,
    /// 服务不核认领回应的字节数：大批「别人持有」带长身份的回应越过帧长上限。
    ClaimResponseUnbudgeted,
    /// 客户端建连用阻塞的 `connect`、报到不设写期限、在途帧不受这一轮重连的总时长上限管。
    ConnectWithoutDeadline,
    /// 自证不给格 cell-runs-alone-in-own-process 另起进程，留在跑格的进程自己的线程里跑（`selftest.rs` 文件头「每格各在自己的进程里」）。
    /// 只弄坏这一格的放法：全部格都退回同一个进程的话，追加日志那两格随别的格起子进程的时机时红时绿，开关轮判不出该红哪一格。
    CellRunsInRunnerProcess,
    /// 开库之前不把 RLIMIT_NOFILE 的软上限抬到硬上限、也不核够不够（改前的开法）：库的文件数多于软上限时 RocksDB 报「Too many open files」拒开、退 3。
    DescriptorSoftLimitNotRaised,
    /// 删前缀走出前缀之后不停，接着删到条数上限（越界删到下一个前缀）。
    DeletePrefixOverruns,
    /// 只数不删的删前缀请求照样把数到的键删掉（干跑不干）。
    DeletePrefixDryRunDeletes,
}

impl BreakSwitch {
    pub const ALL: [BreakSwitch; 65] = [
        BreakSwitch::MultiGetPerKey,
        BreakSwitch::WriteSyncPerEntry,
        BreakSwitch::PrefixScanOverruns,
        BreakSwitch::MissingNetworkAddressDefaultsToLoopback,
        BreakSwitch::ResponseSequenceUnchecked,
        BreakSwitch::FrameLengthUnchecked,
        BreakSwitch::UnknownOperationAccepted,
        BreakSwitch::TrailingBytesAccepted,
        BreakSwitch::BenchJudgmentLenient,
        BreakSwitch::BenchLibraryKeptOnInterrupt,
        BreakSwitch::UnacknowledgedFramesDroppedOnReconnect,
        BreakSwitch::AcknowledgeBeforeSync,
        BreakSwitch::DuplicateRequestsReapplied,
        BreakSwitch::DuplicateResponsesFatal,
        BreakSwitch::FrameTimeoutIgnored,
        BreakSwitch::KeepaliveNotSet,
        BreakSwitch::ReconnectBackoffFlat,
        BreakSwitch::ReconnectWithoutJitter,
        BreakSwitch::ReconnectUnbounded,
        BreakSwitch::ReadFailureReportedAsAbsent,
        BreakSwitch::UnacknowledgedFramesNotJournaled,
        BreakSwitch::BusyReplacedByUnboundedQueue,
        BreakSwitch::BusyRetryWithoutDelay,
        BreakSwitch::ClaimNotExclusive,
        BreakSwitch::ClaimLeaseNeverExpires,
        BreakSwitch::ClaimKeptAfterConclusion,
        BreakSwitch::GroupCommitDisabled,
        BreakSwitch::BenchLoadThresholdIgnored,
        BreakSwitch::JournalCompactsOnEveryAcknowledgement,
        BreakSwitch::FaultProxyHalfClosesOnServerDeath,
        BreakSwitch::ControlDryRunWrites,
        BreakSwitch::ControlInstallReadbackSkipped,
        BreakSwitch::ControlStartReadbackSkipped,
        BreakSwitch::ControlStopReadbackSkipped,
        BreakSwitch::ClientAnnouncementOmitsTask,
        BreakSwitch::AnnouncementIgnored,
        BreakSwitch::ControlStopRecordSkipped,
        BreakSwitch::StopNoticeNotSent,
        BreakSwitch::StopNoticeRetried,
        BreakSwitch::UnreachableExitCodeGeneric,
        BreakSwitch::FaultLineNotWritten,
        BreakSwitch::WatchIgnoresUnitState,
        BreakSwitch::WatchIgnoresHealth,
        BreakSwitch::WatchIgnoresFaultLog,
        BreakSwitch::WatchCountsOldFaultLines,
        BreakSwitch::ClientWriteTimeoutUnset,
        BreakSwitch::FaultProxyOnlyStopsReadingOnServerDeath,
        BreakSwitch::BookkeepingKeyTagDropped,
        BreakSwitch::ServiceKeysOpenToClients,
        BreakSwitch::LibraryExtraColumnFamiliesAccepted,
        BreakSwitch::LibraryOpenFailureReportedAsLocked,
        BreakSwitch::LibraryWithoutCurrentFileOpened,
        BreakSwitch::ControlCheckOmitsLibrary,
        BreakSwitch::JournalCompactionDropsHighWater,
        BreakSwitch::SyncFailureKeptServing,
        BreakSwitch::ResponseSlotReleasedOnEnqueue,
        BreakSwitch::FrameBodyReadBeforeAdmission,
        BreakSwitch::JournalLocksRenamedFile,
        BreakSwitch::OversizeFrameSentAndRetried,
        BreakSwitch::ClaimResponseUnbudgeted,
        BreakSwitch::ConnectWithoutDeadline,
        BreakSwitch::CellRunsInRunnerProcess,
        BreakSwitch::DescriptorSoftLimitNotRaised,
        BreakSwitch::DeletePrefixOverruns,
        BreakSwitch::DeletePrefixDryRunDeletes,
    ];

    pub fn name(self) -> &'static str {
        match self {
            BreakSwitch::MultiGetPerKey => "multi-get-per-key",
            BreakSwitch::WriteSyncPerEntry => "write-sync-per-entry",
            BreakSwitch::PrefixScanOverruns => "prefix-scan-overruns",
            BreakSwitch::MissingNetworkAddressDefaultsToLoopback => {
                "missing-tcp-address-defaults-to-loopback"
            }
            BreakSwitch::ResponseSequenceUnchecked => "response-sequence-unchecked",
            BreakSwitch::FrameLengthUnchecked => "frame-length-unchecked",
            BreakSwitch::UnknownOperationAccepted => "unknown-operation-accepted",
            BreakSwitch::TrailingBytesAccepted => "trailing-bytes-accepted",
            BreakSwitch::BenchJudgmentLenient => "bench-judgment-lenient",
            BreakSwitch::BenchLibraryKeptOnInterrupt => "bench-library-kept-on-interrupt",
            BreakSwitch::UnacknowledgedFramesDroppedOnReconnect => {
                "unacknowledged-frames-dropped-on-reconnect"
            }
            BreakSwitch::AcknowledgeBeforeSync => "acknowledge-before-sync",
            BreakSwitch::DuplicateRequestsReapplied => "duplicate-requests-reapplied",
            BreakSwitch::DuplicateResponsesFatal => "duplicate-responses-fatal",
            BreakSwitch::FrameTimeoutIgnored => "frame-timeout-ignored",
            BreakSwitch::KeepaliveNotSet => "keepalive-not-set",
            BreakSwitch::ReconnectBackoffFlat => "reconnect-backoff-flat",
            BreakSwitch::ReconnectWithoutJitter => "reconnect-without-jitter",
            BreakSwitch::ReconnectUnbounded => "reconnect-unbounded",
            BreakSwitch::ReadFailureReportedAsAbsent => "read-failure-reported-as-absent",
            BreakSwitch::UnacknowledgedFramesNotJournaled => "unacknowledged-frames-not-journaled",
            BreakSwitch::BusyReplacedByUnboundedQueue => "busy-replaced-by-unbounded-queue",
            BreakSwitch::BusyRetryWithoutDelay => "busy-retry-without-delay",
            BreakSwitch::ClaimNotExclusive => "claim-not-exclusive",
            BreakSwitch::ClaimLeaseNeverExpires => "claim-lease-never-expires",
            BreakSwitch::ClaimKeptAfterConclusion => "claim-kept-after-conclusion",
            BreakSwitch::GroupCommitDisabled => "group-commit-disabled",
            BreakSwitch::BenchLoadThresholdIgnored => "bench-load-threshold-ignored",
            BreakSwitch::JournalCompactsOnEveryAcknowledgement => {
                "journal-compacts-on-every-acknowledgement"
            }
            BreakSwitch::FaultProxyHalfClosesOnServerDeath => {
                "fault-proxy-half-closes-on-server-death"
            }
            BreakSwitch::ControlDryRunWrites => "control-dry-run-writes",
            BreakSwitch::ControlInstallReadbackSkipped => "control-install-readback-skipped",
            BreakSwitch::ControlStartReadbackSkipped => "control-start-readback-skipped",
            BreakSwitch::ControlStopReadbackSkipped => "control-stop-readback-skipped",
            BreakSwitch::ClientAnnouncementOmitsTask => "client-announcement-omits-task",
            BreakSwitch::AnnouncementIgnored => "announcement-ignored",
            BreakSwitch::ControlStopRecordSkipped => "control-stop-record-skipped",
            BreakSwitch::StopNoticeNotSent => "stop-notice-not-sent",
            BreakSwitch::StopNoticeRetried => "stop-notice-retried",
            BreakSwitch::UnreachableExitCodeGeneric => "unreachable-exit-code-generic",
            BreakSwitch::FaultLineNotWritten => "fault-line-not-written",
            BreakSwitch::WatchIgnoresUnitState => "watch-ignores-unit-state",
            BreakSwitch::WatchIgnoresHealth => "watch-ignores-health",
            BreakSwitch::WatchIgnoresFaultLog => "watch-ignores-fault-log",
            BreakSwitch::WatchCountsOldFaultLines => "watch-counts-old-fault-lines",
            BreakSwitch::ClientWriteTimeoutUnset => "client-write-timeout-unset",
            BreakSwitch::FaultProxyOnlyStopsReadingOnServerDeath => {
                "fault-proxy-only-stops-reading-on-server-death"
            }
            BreakSwitch::BookkeepingKeyTagDropped => "bookkeeping-key-tag-dropped",
            BreakSwitch::ServiceKeysOpenToClients => "service-keys-open-to-clients",
            BreakSwitch::LibraryExtraColumnFamiliesAccepted => {
                "library-extra-column-families-accepted"
            }
            BreakSwitch::LibraryOpenFailureReportedAsLocked => {
                "library-open-failure-reported-as-locked"
            }
            BreakSwitch::LibraryWithoutCurrentFileOpened => "library-without-current-file-opened",
            BreakSwitch::ControlCheckOmitsLibrary => "control-check-omits-library",
            BreakSwitch::JournalCompactionDropsHighWater => "journal-compaction-drops-high-water",
            BreakSwitch::SyncFailureKeptServing => "sync-failure-kept-serving",
            BreakSwitch::ResponseSlotReleasedOnEnqueue => "response-slot-released-on-enqueue",
            BreakSwitch::FrameBodyReadBeforeAdmission => "frame-body-read-before-admission",
            BreakSwitch::JournalLocksRenamedFile => "journal-locks-renamed-file",
            BreakSwitch::OversizeFrameSentAndRetried => "oversize-frame-sent-and-retried",
            BreakSwitch::ClaimResponseUnbudgeted => "claim-response-unbudgeted",
            BreakSwitch::ConnectWithoutDeadline => "connect-without-deadline",
            BreakSwitch::CellRunsInRunnerProcess => "cell-runs-in-runner-process",
            BreakSwitch::DescriptorSoftLimitNotRaised => "descriptor-soft-limit-not-raised",
            BreakSwitch::DeletePrefixOverruns => "delete-prefix-overruns",
            BreakSwitch::DeletePrefixDryRunDeletes => "delete-prefix-dry-run-deletes",
        }
    }

    /// 打开这一项之后自证里该红、而且只该红的那一格。
    pub fn red_cell(self) -> &'static str {
        match self {
            BreakSwitch::MultiGetPerKey => "batched-point-read",
            BreakSwitch::WriteSyncPerEntry => "group-commit-coalesces-syncs",
            BreakSwitch::PrefixScanOverruns => "prefix-scan",
            BreakSwitch::MissingNetworkAddressDefaultsToLoopback => "missing-tcp-address-refused",
            BreakSwitch::ResponseSequenceUnchecked => "pipelined-out-of-order",
            BreakSwitch::FrameLengthUnchecked => "frame-length-rejected",
            BreakSwitch::UnknownOperationAccepted => "unknown-operation-rejected",
            BreakSwitch::TrailingBytesAccepted => "malformed-payload-rejected",
            BreakSwitch::BenchJudgmentLenient => "bench-rate-judgment",
            BreakSwitch::BenchLibraryKeptOnInterrupt => "bench-interrupted-removes-library",
            BreakSwitch::UnacknowledgedFramesDroppedOnReconnect
            | BreakSwitch::AcknowledgeBeforeSync => "replay-after-disconnect-and-server-kill",
            BreakSwitch::DuplicateRequestsReapplied | BreakSwitch::DuplicateResponsesFatal => {
                "delay-jitter-duplicate-tolerated"
            }
            BreakSwitch::FrameTimeoutIgnored | BreakSwitch::KeepaliveNotSet => {
                "half-open-connection-recovered"
            }
            BreakSwitch::ReconnectBackoffFlat
            | BreakSwitch::ReconnectWithoutJitter
            | BreakSwitch::ReconnectUnbounded
            | BreakSwitch::ReadFailureReportedAsAbsent
            | BreakSwitch::UnacknowledgedFramesNotJournaled => "reconnect-gives-up-at-limit",
            BreakSwitch::BusyReplacedByUnboundedQueue | BreakSwitch::BusyRetryWithoutDelay => {
                "busy-backpressure-bounded"
            }
            BreakSwitch::ClaimNotExclusive
            | BreakSwitch::ClaimLeaseNeverExpires
            | BreakSwitch::ClaimKeptAfterConclusion => "claim-exclusive-and-lease-expiry",
            BreakSwitch::GroupCommitDisabled => "group-commit-coalesces-syncs",
            BreakSwitch::BenchLoadThresholdIgnored => "bench-environment-judgment",
            BreakSwitch::JournalCompactsOnEveryAcknowledgement => "fault-injection-bench-finishes",
            BreakSwitch::FaultProxyHalfClosesOnServerDeath => "server-kill-while-client-writes",
            BreakSwitch::ControlDryRunWrites
            | BreakSwitch::ControlInstallReadbackSkipped
            | BreakSwitch::ControlStartReadbackSkipped
            | BreakSwitch::ControlStopReadbackSkipped => "control-install-start-stop-check",
            BreakSwitch::ClientAnnouncementOmitsTask
            | BreakSwitch::AnnouncementIgnored
            | BreakSwitch::ControlStopRecordSkipped => "stop-lists-connected-clients",
            BreakSwitch::StopNoticeNotSent | BreakSwitch::StopNoticeRetried => {
                "intentional-stop-fails-clients-fast"
            }
            BreakSwitch::UnreachableExitCodeGeneric => "unannounced-server-death-bounded",
            BreakSwitch::FaultLineNotWritten => "fault-line-on-server-error",
            BreakSwitch::WatchIgnoresUnitState => "watch-reports-unit-not-running",
            BreakSwitch::WatchIgnoresHealth => "watch-reports-server-unreachable",
            BreakSwitch::WatchIgnoresFaultLog | BreakSwitch::WatchCountsOldFaultLines => {
                "watch-reports-new-fault-line"
            }
            BreakSwitch::ClientWriteTimeoutUnset => "client-write-timeout-when-peer-stops-reading",
            BreakSwitch::FaultProxyOnlyStopsReadingOnServerDeath => {
                "tcp-proxy-server-kill-client-fails"
            }
            BreakSwitch::BookkeepingKeyTagDropped | BreakSwitch::ServiceKeysOpenToClients => {
                "single-library-layout"
            }
            BreakSwitch::LibraryExtraColumnFamiliesAccepted
            | BreakSwitch::LibraryOpenFailureReportedAsLocked
            | BreakSwitch::LibraryWithoutCurrentFileOpened => "library-refuses-mismatched",
            BreakSwitch::ControlCheckOmitsLibrary => "control-install-start-stop-check",
            BreakSwitch::JournalCompactionDropsHighWater => {
                "journal-high-water-survives-compaction"
            }
            BreakSwitch::SyncFailureKeptServing => {
                "write-ahead-log-sync-failure-never-acknowledged"
            }
            BreakSwitch::ResponseSlotReleasedOnEnqueue => "unread-responses-bounded",
            BreakSwitch::FrameBodyReadBeforeAdmission => "incomplete-frames-bounded",
            BreakSwitch::JournalLocksRenamedFile => "journal-lock-survives-compaction",
            BreakSwitch::OversizeFrameSentAndRetried => "oversize-frame-reported-at-once",
            BreakSwitch::ClaimResponseUnbudgeted => "claim-response-within-frame-limit",
            BreakSwitch::ConnectWithoutDeadline => "connect-bounded-by-reconnect-total",
            BreakSwitch::CellRunsInRunnerProcess => "cell-runs-alone-in-own-process",
            BreakSwitch::DescriptorSoftLimitNotRaised => {
                "library-opens-under-low-descriptor-soft-limit"
            }
            BreakSwitch::DeletePrefixOverruns => "delete-prefix-only-under-prefix",
            BreakSwitch::DeletePrefixDryRunDeletes => "delete-prefix-counts-before-deleting",
        }
    }

    fn from_name(text: &str) -> Option<BreakSwitch> {
        BreakSwitch::ALL
            .into_iter()
            .find(|switch| switch.name() == text)
    }
}

/// 这一次打开的弄坏开关；空集合是平常跑。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreakSwitches {
    enabled: Vec<BreakSwitch>,
}

impl BreakSwitches {
    pub fn none() -> BreakSwitches {
        BreakSwitches {
            enabled: Vec::new(),
        }
    }

    /// 逗号分隔的项名；认不得的项名拒，不悄悄忽略（写错一个字母的开关会让证红变成假绿）。
    pub fn parse(text: &str) -> Result<BreakSwitches, String> {
        let mut enabled = Vec::new();
        for item in text
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
        {
            match BreakSwitch::from_name(item) {
                Some(switch) => enabled.push(switch),
                None => {
                    let known: Vec<&str> = BreakSwitch::ALL
                        .iter()
                        .map(|switch| switch.name())
                        .collect();
                    return Err(format!(
                        "{BREAK_VARIABLE} 里的项「{item}」认不得；认得的：{}",
                        known.join("、")
                    ));
                }
            }
        }
        Ok(BreakSwitches { enabled })
    }

    /// 读环境变量；没设或是空串就是平常跑。
    pub fn from_environment() -> Result<BreakSwitches, String> {
        match std::env::var(BREAK_VARIABLE) {
            Ok(text) => BreakSwitches::parse(&text),
            Err(std::env::VarError::NotPresent) => Ok(BreakSwitches::none()),
            Err(std::env::VarError::NotUnicode(_)) => Err(format!("{BREAK_VARIABLE} 不是 UTF-8")),
        }
    }

    pub fn is_enabled(&self, switch: BreakSwitch) -> bool {
        self.enabled.contains(&switch)
    }

    /// 只留 `switch` 这一项（打开了才留）：压测器只把自己那一格的开关交给它的客户端，别的格的客户端开关不碰压测。
    pub fn only(&self, switch: BreakSwitch) -> BreakSwitches {
        self.only_these(&[switch])
    }

    /// 只留 `switches` 里的几项（打开了才留）：自证起子进程（控制命令、自证客户端）时只交这一格要证红的开关。
    pub fn only_these(&self, switches: &[BreakSwitch]) -> BreakSwitches {
        BreakSwitches {
            enabled: self
                .enabled
                .iter()
                .copied()
                .filter(|enabled| switches.contains(enabled))
                .collect(),
        }
    }

    /// 写回环境变量的值（逗号分隔的项名；空串是平常跑）。
    pub fn environment_text(&self) -> String {
        self.enabled
            .iter()
            .map(|switch| switch.name())
            .collect::<Vec<_>>()
            .join(",")
    }
}
