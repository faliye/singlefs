//! 服务的起停：准备套接字与状态目录、开库、开两处监听、起工作线程池、接受循环，与测试进程里起的一份服务。

use super::*;

fn prepare_socket_directory(settings: &ServerSettings) -> Result<(), ServeFailure> {
    if let Some(problem) = socket_path_problem(&settings.socket_path) {
        return Err(ServeFailure::SocketUnavailable(problem));
    }
    let Some(directory) = settings.socket_path.parent() else {
        return Err(ServeFailure::SocketUnavailable(format!(
            "socket 路径 {} 没有上一层目录",
            settings.socket_path.display()
        )));
    };
    if !directory.exists() {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(directory)
            .map_err(|error| {
                ServeFailure::SocketUnavailable(format!(
                    "建不了 socket 目录 {}：{error}",
                    directory.display()
                ))
            })?;
    }
    let metadata = std::fs::metadata(directory).map_err(|error| {
        ServeFailure::SocketUnavailable(format!(
            "读不了 socket 目录 {}：{error}",
            directory.display()
        ))
    })?;
    if !metadata.is_dir() {
        return Err(ServeFailure::SocketUnavailable(format!(
            "socket 目录 {} 不是目录",
            directory.display()
        )));
    }
    // socket 目录归服务管：谁建的、umask 多宽都不管，服务起来时把它收成 700（第二台 umask 002 建出来的是 775）
    let mode = metadata.permissions().mode() & 0o777;
    if mode != 0o700 {
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700)).map_err(
            |error| {
                ServeFailure::SocketUnavailable(format!(
                    "socket 目录 {} 的权限是 {mode:o}，改成 700 没成：{error}",
                    directory.display()
                ))
            },
        )?;
    }
    if settings.socket_path.exists() {
        if unix_socket_has_listener(&settings.socket_path) {
            return Err(ServeFailure::SocketUnavailable(format!(
                "已有服务在 {} 上回话",
                settings.socket_path.display()
            )));
        }
        std::fs::remove_file(&settings.socket_path).map_err(|error| {
            ServeFailure::SocketUnavailable(format!(
                "删不了没人回话的旧 socket {}：{error}",
                settings.socket_path.display()
            ))
        })?;
    }
    Ok(())
}

/// 状态目录（故障日志在里面）：没有就建成 700。
fn prepare_state_directory(settings: &ServerSettings) -> Result<(), ServeFailure> {
    if settings.state_directory.exists() {
        return Ok(());
    }
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&settings.state_directory)
        .map_err(|error| {
            ServeFailure::LibraryUnavailable(format!(
                "建不了状态目录 {}：{error}",
                settings.state_directory.display()
            ))
        })
}

/// 服务开好了库、绑好了两处监听、起好了提交线程与工作线程池，还没开始接连接的样子（[`serve`] 与 [`start_in_this_process`] 共用这一段）。
struct OpenedService {
    shared: Arc<SharedState>,
    dispatch: Dispatch,
    unix_listener: UnixListener,
    network_listener: TcpListener,
    bound_network_address: SocketAddr,
}

/// 线程 panic 记不记进故障日志：服务进程记（装进程级的 panic 钩子）；在调用方进程里起的那一份不装，那个进程的 panic 钩子是它自己的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PanicRecording {
    IntoTheFaultLog,
    LeftToTheHostProcess,
}

/// 开库、绑监听、起提交线程与工作线程池（[`OpenedService`]）；`release_notice` 随共享状态最后放手（库关完之后）时回话。
fn open_the_service(
    settings: ServerSettings,
    panic_recording: PanicRecording,
    release_notice: ReleaseNotice,
) -> Result<OpenedService, ServeFailure> {
    prepare_socket_directory(&settings)?;
    prepare_state_directory(&settings)?;
    let fault_log = Arc::new(
        FaultLog::open(&settings.state_directory, &settings.break_switches)
            .map_err(ServeFailure::LibraryUnavailable)?,
    );
    match panic_recording {
        PanicRecording::IntoTheFaultLog => install_panic_recorder(Arc::clone(&fault_log)),
        PanicRecording::LeftToTheHostProcess => {
            // 宿主进程的 panic 钩子归它自己，这一份不动
        }
    }
    if let Some(parent) = settings.library_directory.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            ServeFailure::LibraryUnavailable(format!(
                "建不了库目录的上一层 {}：{error}",
                parent.display()
            ))
        })?;
    }
    let mut library = VerdictLibrary::open(
        &settings.library_directory,
        settings.worker_threads,
        settings.break_switches.clone(),
    )
    .map_err(|failure| match failure {
        LibraryOpenFailure::Unavailable(message) => ServeFailure::LibraryUnavailable(message),
        LibraryOpenFailure::Mismatched(message) => ServeFailure::LibraryMismatched(message),
        LibraryOpenFailure::DescriptorLimitTooLow(message) => {
            ServeFailure::DescriptorLimitTooLow(message)
        }
    })?;
    library.set_selftest_sync_failure_file(settings.selftest_sync_failure_file.clone());
    let unix_listener = UnixListener::bind(&settings.socket_path).map_err(|error| {
        ServeFailure::SocketUnavailable(format!(
            "绑不上 socket {}：{error}",
            settings.socket_path.display()
        ))
    })?;
    std::fs::set_permissions(
        &settings.socket_path,
        std::fs::Permissions::from_mode(0o600),
    )
    .map_err(|error| ServeFailure::SocketUnavailable(format!("改不了 socket 权限：{error}")))?;
    let network_listener = TcpListener::bind(&settings.network_address).map_err(|error| {
        ServeFailure::NetworkUnavailable(format!(
            "绑不上 TCP 地址 {}：{error}",
            settings.network_address
        ))
    })?;
    let bound_network_address = network_listener.local_addr().map_err(|error| {
        ServeFailure::NetworkUnavailable(format!("读不出 TCP 监听的地址：{error}"))
    })?;

    let shared = Arc::new(SharedState {
        library,
        break_switches: settings.break_switches.clone(),
        selftest_delay: settings.selftest_delay,
        counters: ServerCounters::default(),
        write_ahead_log_sync_timings: WriteAheadLogSyncTimings::new(),
        admission: Arc::new(Admission::new(
            settings.admission,
            settings
                .break_switches
                .is_enabled(BreakSwitch::BusyReplacedByUnboundedQueue),
        )),
        connections: ConnectionRegistry::default(),
        outbound_totals: Arc::new(OutboundTotals::default()),
        fault_log,
        is_stopping: AtomicBool::new(false),
        release_notice,
    });
    let committer =
        start_committer(&shared, settings.committer).map_err(ServeFailure::LibraryUnavailable)?;
    let dispatch = Dispatch {
        workers: start_worker_pool(&shared, settings.worker_threads),
        committer,
    };
    Ok(OpenedService {
        shared,
        dispatch,
        unix_listener,
        network_listener,
        bound_network_address,
    })
}

/// 两个接连接的线程（unix socket、TCP）；`stop_requested` 置上之后各自接到的下一条连接不收、线程收尾。
fn start_accepting(
    opened: OpenedService,
    stop_requested: &Arc<AtomicBool>,
) -> [thread::JoinHandle<AcceptLoopEnd>; 2] {
    let OpenedService {
        shared,
        dispatch,
        unix_listener,
        network_listener,
        bound_network_address: _,
    } = opened;
    let unix_accept = {
        let shared = Arc::clone(&shared);
        let dispatch = dispatch.clone();
        let stop_requested = Arc::clone(stop_requested);
        thread::spawn(move || {
            accept_unix_connections(unix_listener, &shared, &dispatch, &stop_requested)
        })
    };
    let network_accept = {
        let stop_requested = Arc::clone(stop_requested);
        thread::spawn(move || {
            accept_network_connections(network_listener, &shared, &dispatch, &stop_requested)
        })
    };
    [unix_accept, network_accept]
}

/// 起服务；正常情况下不返回（由 SIGTERM 结束：写都是同步的，被杀不丢已回应的写）。
pub fn serve(settings: ServerSettings) -> Result<(), ServeFailure> {
    let socket_path = settings.socket_path.clone();
    let worker_threads = settings.worker_threads;
    let library_directory = settings.library_directory.clone();
    let opened = open_the_service(
        settings,
        PanicRecording::IntoTheFaultLog,
        ReleaseNotice::Unwatched,
    )?;
    println!(
        "{LISTENING_LINE_PREFIX} unix={} tcp={} library={} faults={} pid={} worker_threads={worker_threads}",
        socket_path.display(),
        opened.bound_network_address,
        library_directory.display(),
        opened.shared.fault_log.path().display(),
        std::process::id(),
    );
    io::stdout()
        .flush()
        .map_err(|error| ServeFailure::ListenerStopped(format!("就绪行写不出去：{error}")))?;

    // 服务进程里没有停服句柄：这个标记一直不置，接连接的线程只在监听出错时收尾
    let never_stopped = Arc::new(AtomicBool::new(false));
    let [unix_accept, network_accept] = start_accepting(opened, &never_stopped);
    let unix_result = unix_accept.join();
    let network_result = network_accept.join();
    let describe = |name: &str, result: thread::Result<AcceptLoopEnd>| match result {
        Ok(AcceptLoopEnd::ListenerFailed(error)) => format!("{name} 监听停了：{error}"),
        Ok(AcceptLoopEnd::StopRequested) => format!("{name} 监听被停服句柄叫停了"),
        Err(_) => format!("{name} 监听线程 panic 了"),
    };
    Err(ServeFailure::ListenerStopped(format!(
        "{}；{}",
        describe("unix socket", unix_result),
        describe("TCP", network_result)
    )))
}

/// 在调用方进程里起的那一份用几个工作线程（也是交给 RocksDB 的后台线程数）：只给测试自己起的临时服务，一个进程里同时开着好几份，各开少几个。
pub const WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS: usize = 2;
/// 停服时等库关完的上限：接连接的线程收尾、连着的连接都关掉之后，提交线程与工作线程放手共享状态、库关上，正常在毫秒级。
pub const STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT: Duration = Duration::from_secs(30);
/// 停服时等库关完期间，每过这么久再关一遍连着的连接（刚接进来、还没登记的那几条登记之后才关得到）。
const STOP_OF_A_SERVICE_IN_THIS_PROCESS_POLL: Duration = Duration::from_millis(10);

/// 共享状态放手（库关完）时回话的一端：放在 [`SharedState`] 的最后一个字段，字段按声明的次序放手，回话时库已经关上。
pub(super) enum ReleaseNotice {
    /// 服务进程：没人等（进程由 SIGTERM 结束）。
    Unwatched,
    /// 在调用方进程里起的那一份：停服句柄等这一声才删目录。
    Watched(Mutex<Sender<()>>),
}

impl Drop for ReleaseNotice {
    fn drop(&mut self) {
        match self {
            ReleaseNotice::Unwatched => {
                // 服务进程由 SIGTERM 结束，没人等这一声
            }
            ReleaseNotice::Watched(sender) => {
                // 停服句柄已经不等了（等过了上限）的，回话没人收，丢掉
                let _ = sender
                    .get_mut()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .send(());
            }
        }
    }
}

/// 在调用方这个进程里起的一份服务（测试自己起的临时服务：状态目录、库与 socket 都在调用方给的位置，协议与服务进程同一份）。
/// 停服（[`ServiceInThisProcess::stop`]，丢掉时同样停）：不再接连接、关掉连着的连接、删 socket，等提交线程与工作线程放手、库关上才回。
/// 与服务进程不同的只有三处：不装进程级的 panic 钩子（`PanicRecording::LeftToTheHostProcess`）、不打就绪行、有停服句柄。
/// 同步 WAL 失败时与服务进程一样退出整个进程（`storage.rs` 文件头「同步 WAL 失败之后」），这一份不另改。
pub struct ServiceInThisProcess {
    socket_path: PathBuf,
    bound_network_address: SocketAddr,
    stop_requested: Arc<AtomicBool>,
    accept_threads: Vec<thread::JoinHandle<AcceptLoopEnd>>,
    shared: Weak<SharedState>,
    released: Receiver<()>,
    is_stopped: bool,
}

/// 在这个进程里起一份服务：状态目录 `state_directory`（故障日志与库 `library_directory_in(state_directory)` 都在里面，没有就建）、
/// socket `socket_path`（全路径受 `configuration::MAXIMUM_SOCKET_PATH_BYTES` 限）、TCP 绑 `network_address`（`<地址>:<端口>`；调用方给多机配置 `HOSTS` 里这一台的地址
/// 加端口 0，端口由内核挑、几份同时起互不撞端口，两台的 GPU 批处理服务都连得到；这里不读多机配置、不补缺省，读不到地址由调用方报错）；不读环境、
/// 不认弄坏开关。各项上限取命令行的缺省（`command_line` 的 `DEFAULT_*`），工作线程 [`WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS`] 个。
///
/// # Errors
/// 同 [`serve`] 起不来的几种（[`ServeFailure`]）：socket 路径太长、已有服务在那个 socket 上回话、库打不开、端口绑不上（含地址不在这台的网卡上）。
pub fn start_in_this_process(
    state_directory: &Path,
    socket_path: &Path,
    network_address: &str,
) -> Result<ServiceInThisProcess, ServeFailure> {
    start_in_this_process_listening_on(
        state_directory,
        socket_path,
        network_address,
        CommitterSettings {
            group_window: Duration::from_millis(DEFAULT_GROUP_COMMIT_MILLISECONDS),
            group_maximum_frames: DEFAULT_GROUP_COMMIT_FRAMES,
            selftest_commit_delay: None,
            selftest_storage_failure_key: None,
            selftest_write_failure_file: None,
        },
    )
}

/// [`start_in_this_process`]，TCP 绑 `network_address`、提交线程取 `committer`（测试要别的绑定地址、别的组提交窗口与自证造错时用）。
pub(crate) fn start_in_this_process_listening_on(
    state_directory: &Path,
    socket_path: &Path,
    network_address: &str,
    committer: CommitterSettings,
) -> Result<ServiceInThisProcess, ServeFailure> {
    let settings = ServerSettings {
        socket_path: socket_path.to_path_buf(),
        network_address: network_address.to_string(),
        state_directory: state_directory.to_path_buf(),
        library_directory: library_directory_in(state_directory),
        worker_threads: WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS,
        break_switches: BreakSwitches::none(),
        selftest_delay: None,
        selftest_sync_failure_file: None,
        admission: AdmissionLimits {
            maximum_pending_per_connection: DEFAULT_MAXIMUM_PENDING_PER_CONNECTION,
            maximum_pending_total: DEFAULT_MAXIMUM_PENDING_TOTAL,
            maximum_pending_bytes: DEFAULT_MAXIMUM_PENDING_BYTES,
            maximum_pending_bytes_per_connection: DEFAULT_MAXIMUM_PENDING_BYTES_PER_CONNECTION,
            maximum_connections: DEFAULT_MAXIMUM_CONNECTIONS,
            frame_read_timeout: Duration::from_millis(DEFAULT_FRAME_READ_TIMEOUT_MILLISECONDS),
            write_timeout: Duration::from_millis(DEFAULT_WRITE_TIMEOUT_MILLISECONDS),
            busy_retry: Duration::from_millis(DEFAULT_BUSY_RETRY_MILLISECONDS),
        },
        committer,
    };
    let (release_sender, released) = mpsc::channel();
    let opened = open_the_service(
        settings,
        PanicRecording::LeftToTheHostProcess,
        ReleaseNotice::Watched(Mutex::new(release_sender)),
    )?;
    let bound_network_address = opened.bound_network_address;
    let shared = Arc::downgrade(&opened.shared);
    let stop_requested = Arc::new(AtomicBool::new(false));
    let accept_threads = start_accepting(opened, &stop_requested).into();
    Ok(ServiceInThisProcess {
        socket_path: socket_path.to_path_buf(),
        bound_network_address,
        stop_requested,
        accept_threads,
        shared,
        released,
        is_stopped: false,
    })
}

impl ServiceInThisProcess {
    /// 客户端连的 socket。
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// TCP 实际绑上的地址（本机回环与内核挑的端口）。
    #[must_use]
    pub fn bound_network_address(&self) -> SocketAddr {
        self.bound_network_address
    }

    /// 停服：不再接连接、删 socket、关掉连着的连接，等库关上才回。
    ///
    /// # Errors
    /// 接连接的线程 panic 了；等满 [`STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT`] 库还没关上（还有线程拿着共享状态）。
    pub fn stop(mut self) -> Result<(), String> {
        self.stop_and_wait_for_the_library_to_close()
    }

    fn stop_and_wait_for_the_library_to_close(&mut self) -> Result<(), String> {
        self.is_stopped = true;
        self.stop_requested.store(true, Ordering::SeqCst);
        // 接连接的线程堵在 accept 上：各连一下叫醒它，它看到停服标记就收尾（这一条连接不收）
        let _ = UnixStream::connect(&self.socket_path);
        let _ = TcpStream::connect(loopback_address_to_wake_the_listener(
            self.bound_network_address,
        ));
        let mut problems = Vec::new();
        for accept_thread in self.accept_threads.drain(..) {
            match accept_thread.join() {
                Ok(AcceptLoopEnd::StopRequested) => {}
                Ok(AcceptLoopEnd::ListenerFailed(error)) => {
                    problems.push(format!("停服之前监听就停了：{error}"));
                }
                Err(_) => problems.push("接连接的线程 panic 了".to_string()),
            }
        }
        if let Err(error) = std::fs::remove_file(&self.socket_path) {
            if error.kind() != io::ErrorKind::NotFound {
                problems.push(format!(
                    "删不了 socket {}：{error}",
                    self.socket_path.display()
                ));
            }
        }
        let deadline = Instant::now() + STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT;
        // 迭代上界：STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT / STOP_OF_A_SERVICE_IN_THIS_PROCESS_POLL 轮；库关上（回话）或回话的一端已经放手就出
        loop {
            if let Some(shared) = self.shared.upgrade() {
                shared.connections.shut_down_every_connection();
            }
            match self
                .released
                .recv_timeout(STOP_OF_A_SERVICE_IN_THIS_PROCESS_POLL)
            {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if Instant::now() >= deadline {
                problems.push(format!(
                    "等了 {} 秒库还没关上：还有 {} 处拿着服务的共享状态（连接、提交线程或工作线程没收尾）",
                    STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT.as_secs(),
                    self.shared.strong_count()
                ));
                break;
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems.join("；"))
        }
    }
}

/// 叫醒堵在 accept 上的接连接线程要连的地址：监听绑在未指定地址（`0.0.0.0`、`::`）上的，换成同一地址族的回环
/// （绑在 `::` 上、`bindv6only` 开着的主机连 `127.0.0.1` 会被拒），别的原样。
fn loopback_address_to_wake_the_listener(bound_address: SocketAddr) -> SocketAddr {
    let mut wake_address = bound_address;
    if bound_address.ip().is_unspecified() {
        let loopback = match bound_address {
            SocketAddr::V4(_) => std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            SocketAddr::V6(_) => std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),
        };
        wake_address.set_ip(loopback);
    }
    wake_address
}

impl Drop for ServiceInThisProcess {
    fn drop(&mut self) {
        if self.is_stopped {
            return;
        }
        if let Err(problem) = self.stop_and_wait_for_the_library_to_close() {
            eprintln!(
                "  ✗ 停这个进程里起的 prune-server（{}）没停干净：{problem}",
                self.socket_path.display()
            );
        }
    }
}

/// 接连接的线程为什么收尾。
enum AcceptLoopEnd {
    /// 监听出错（`accept` 交回的错）。
    ListenerFailed(io::Error),
    /// 停服句柄叫停（[`ServiceInThisProcess::stop`]）。
    StopRequested,
}

/// 工作队列的通道本身不设容量：排进去的帧都过了准入，条数受待处理上限所限。
fn start_worker_pool(shared: &Arc<SharedState>, worker_threads: usize) -> Sender<Job> {
    let (job_sender, job_receiver) = mpsc::channel::<Job>();
    let job_receiver = Arc::new(Mutex::new(job_receiver));
    for _ in 0..worker_threads.max(1) {
        let shared = Arc::clone(shared);
        let job_receiver: Arc<Mutex<Receiver<Job>>> = Arc::clone(&job_receiver);
        thread::spawn(move || loop {
            let received = job_receiver
                .lock()
                .expect("工作队列锁只在 recv 时持有，持有时不会 panic")
                .recv();
            let Ok(job) = received else {
                return;
            };
            let Job {
                body,
                responder,
                mut slot,
                connection,
            } = job;
            let reserves_while_building =
                !shared.is_enabled(BreakSwitch::ResponseBuiltBeforeBudget);
            let (mut response, reserved_while_building) = {
                let mut budget =
                    ResponseBudget::new(&mut slot, &shared.counters, reserves_while_building);
                let response = process_frame(&shared, &body, &connection, &mut budget);
                (response, budget.reserved_bytes)
            };
            // 回应记进额度才交出去（文件头「额度」）：边编边记过的只再记余下的帧头那几个字节；记不进就换成忙、已经记下的放回，
            // 这一帧做过、回应没留，重发即可（读、扫、统计都不改库）。
            if !slot.reserve_response(response.len().saturating_sub(reserved_while_building)) {
                slot.release_response(reserved_while_building);
                let (sequence, operation_code) = peek_request_header(&body).unwrap_or_default();
                let reason = shared
                    .admission
                    .describe_bytes_refusal(&slot.budget, response.len());
                shared
                    .counters
                    .busy_rejections
                    .fetch_add(1, Ordering::Relaxed);
                let retry_after = shared.retry_after_milliseconds();
                response = encode_busy(
                    sequence,
                    operation_code,
                    retry_after,
                    &format!("忙：回应{reason}；这一帧做过、回应没留，{retry_after} 毫秒后再来"),
                );
            }
            drop(body);
            // 连接已经断了的，回应没人收，丢掉；名额随回应写出去或丢掉时放手。
            let _ = responder.send_response(response, slot);
        });
    }
    job_sender
}

fn accept_unix_connections(
    listener: UnixListener,
    shared: &Arc<SharedState>,
    job_sender: &Dispatch,
    stop_requested: &AtomicBool,
) -> AcceptLoopEnd {
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                // 停服句柄叫停之后接到的（多半是叫醒这个线程的那一条）不收：放手 stream、线程收尾
                if stop_requested.load(Ordering::SeqCst) {
                    return AcceptLoopEnd::StopRequested;
                }
                // 到了连接数上限的当场关掉（放手 stream）。
                let Some(open_connection) = shared.open_connection() else {
                    continue;
                };
                let shared = Arc::clone(shared);
                let job_sender = job_sender.clone();
                let origin = ConnectionOrigin {
                    transport: Transport::UnixSocket,
                    peer_address: String::new(),
                };
                thread::spawn(move || {
                    handle_connection(stream, &origin, &shared, &job_sender);
                    drop(open_connection);
                });
            }
            Err(error) => return AcceptLoopEnd::ListenerFailed(error),
        }
    }
}

fn accept_network_connections(
    listener: TcpListener,
    shared: &Arc<SharedState>,
    job_sender: &Dispatch,
    stop_requested: &AtomicBool,
) -> AcceptLoopEnd {
    loop {
        match listener.accept() {
            Ok((stream, peer)) => {
                // 停服句柄叫停之后接到的（多半是叫醒这个线程的那一条）不收：放手 stream、线程收尾
                if stop_requested.load(Ordering::SeqCst) {
                    return AcceptLoopEnd::StopRequested;
                }
                // 到了连接数上限的当场关掉（放手 stream）。
                let Some(open_connection) = shared.open_connection() else {
                    continue;
                };
                // 小帧（小块一块一帧）不等 Nagle 攒包；设不上只影响延迟，不影响对错。
                let _ = stream.set_nodelay(true);
                // 半开的连接由内核认出来关掉；设不上时这条连接要等对端自己断，照样服务。
                if let Err(message) = enable_keepalive(&stream, KeepaliveSettings::default()) {
                    eprintln!("  ✗ 给接进来的 TCP 连接开 keepalive 失败：{message}");
                }
                let shared = Arc::clone(shared);
                let job_sender = job_sender.clone();
                let origin = ConnectionOrigin {
                    transport: Transport::Network,
                    peer_address: peer.to_string(),
                };
                thread::spawn(move || {
                    handle_connection(stream, &origin, &shared, &job_sender);
                    drop(open_connection);
                });
            }
            Err(error) => return AcceptLoopEnd::ListenerFailed(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{RouteAddress, SimpleClient};
    use crate::in_process_service_for_tests::{
        committer_settings_grouping, ServiceUnderTest, TemporaryDirectoryOfTheTest,
        LOOPBACK_NETWORK_ADDRESS,
    };
    use crate::protocol::Table;

    /// 在这个进程里起的一份：客户端经 socket 写得进、读得回；停服之后 socket 不在、连不上，库已经关上——同一个状态目录马上再起一份
    /// 开得了库（RocksDB 的 LOCK 放了）、读得回停服之前写的那一行。
    /// 判别力：停服不关连着的连接（`shut_down_every_connection` 什么都不做），客户端还连着，库等不到关上，`stop` 交回错误，这条红。
    #[test]
    fn service_started_in_this_process_serves_a_client_and_after_the_stop_releases_its_library_and_socket(
    ) {
        let directory = TemporaryDirectoryOfTheTest::create("in-process-stop");
        let state_directory = directory.path.join("state");
        let socket_path = directory.path.join("socket");
        let service =
            start_in_this_process(&state_directory, &socket_path, LOOPBACK_NETWORK_ADDRESS)
                .unwrap_or_else(|failure| panic!("起不来：{failure:?}"));
        let route = RouteAddress::UnixSocket(service.socket_path().to_path_buf());
        let mut client =
            SimpleClient::connect(&route, BreakSwitches::none()).expect("连得上这一份");
        client
            .write(&[(Table::Facts, b"key".to_vec(), b"value".to_vec())])
            .expect("写得进");
        assert_eq!(
            client
                .read(&[(Table::Facts, b"key".to_vec())])
                .expect("读得了"),
            vec![Some(b"value".to_vec())]
        );
        // 客户端还连着就停服：停服关掉连着的连接，不等客户端自己走
        service.stop().expect("停得干净：库关上了");
        assert!(!socket_path.exists(), "停服之后 socket 不在");
        assert!(UnixStream::connect(&socket_path).is_err(), "停服之后连不上");
        drop(client);
        let restarted =
            start_in_this_process(&state_directory, &socket_path, LOOPBACK_NETWORK_ADDRESS)
                .unwrap_or_else(|failure| {
                    panic!("停服之后同一个状态目录再起一份，库应当开得了：{failure:?}")
                });
        let mut client_of_the_restarted =
            SimpleClient::connect(&route, BreakSwitches::none()).expect("连得上重起的那一份");
        assert_eq!(
            client_of_the_restarted
                .read(&[(Table::Facts, b"key".to_vec())])
                .expect("读得了"),
            vec![Some(b"value".to_vec())],
            "停服之前写的那一行在库里"
        );
        drop(client_of_the_restarted);
        restarted.stop().expect("重起的那一份同样停得干净");
    }

    /// 在这个进程里起的那一份绑的是调用方给的地址，不是写死的回环地址：给 `127.0.0.2:0`（回环段里另一个地址）与 `0.0.0.0:0`，
    /// 绑上的 IP 就是给的那一个（后一个是未指定地址）。
    /// 判别力：不看 `network_address` 参数、照旧绑 `127.0.0.1:0`，两个地址的断言都红。
    #[test]
    fn service_started_in_this_process_binds_the_network_address_it_is_given() {
        for given_address in ["127.0.0.2:0", "0.0.0.0:0"] {
            let given_ip = given_address
                .parse::<SocketAddr>()
                .expect("写死的地址合规")
                .ip();
            let directory = TemporaryDirectoryOfTheTest::create("in-process-bind-address");
            let service = start_in_this_process(
                &directory.path.join("state"),
                &directory.path.join("socket"),
                given_address,
            )
            .unwrap_or_else(|failure| panic!("绑 {given_address} 起不来：{failure:?}"));
            assert_eq!(
                service.bound_network_address().ip(),
                given_ip,
                "给的是 {given_address}，绑上的却是 {}",
                service.bound_network_address()
            );
            service.stop().expect("停得干净");
        }
    }

    fn assert_wake_address_is(bound_address: &str, expected_wake_address: &str) {
        let bound_address: SocketAddr = bound_address.parse().expect("写死的地址合规");
        let expected_wake_address: SocketAddr =
            expected_wake_address.parse().expect("写死的地址合规");
        assert_eq!(
            loopback_address_to_wake_the_listener(bound_address),
            expected_wake_address,
            "绑在 {bound_address} 的监听，停服时该连 {expected_wake_address} 叫醒接连接的线程"
        );
    }

    /// 判别力：未指定地址一律换成 IPv4 回环（不分地址族），绑在 `::` 上的监听被指到 `127.0.0.1`，下面的 IPv6 那条红。
    #[test]
    fn listener_bound_to_the_ipv4_unspecified_address_is_woken_through_the_ipv4_loopback() {
        assert_wake_address_is("0.0.0.0:4321", "127.0.0.1:4321");
    }

    #[test]
    fn listener_bound_to_the_ipv6_unspecified_address_is_woken_through_the_ipv6_loopback() {
        assert_wake_address_is("[::]:4321", "[::1]:4321");
    }

    #[test]
    fn listener_bound_to_a_specific_ipv4_address_is_woken_at_that_address() {
        assert_wake_address_is("10.1.2.3:4321", "10.1.2.3:4321");
    }

    #[test]
    fn listener_bound_to_a_specific_ipv6_address_is_woken_at_that_address() {
        assert_wake_address_is("[2001:db8::1]:4321", "[2001:db8::1]:4321");
    }

    /// 绑在 IPv6 未指定地址上的服务，停服在时限内回来、socket 删掉。本机绑不了 `[::1]`（没有 IPv6 回环）时这条没法判：
    /// 打一行 `NOT_RUN_NO_IPV6_LOOPBACK` 报本次未跑、直接返回，不算判过。
    /// 判别力：在 `bindv6only` 开着的主机上，叫醒连 `127.0.0.1` 连不上，`stop` 等不到接连接的线程收尾，这条超时红；
    /// 双栈主机上 `127.0.0.1` 也连得进 `::`，这条绿与否不区分，区分靠上面四条纯函数用例。
    #[test]
    fn service_listening_on_the_ipv6_unspecified_address_stops_within_the_time_limit() {
        const STOP_TIME_LIMIT: Duration = Duration::from_secs(10);
        if TcpListener::bind("[::1]:0").is_err() {
            println!(
                "NOT_RUN_NO_IPV6_LOOPBACK 本次未跑：本机绑不了 [::1]（没有 IPv6 回环），绑 [::]:0 的服务停服那条路没判"
            );
            return;
        }
        let running = ServiceUnderTest::start_listening_on(
            "ipv6-stop",
            "[::]:0",
            committer_settings_grouping(
                Duration::from_millis(DEFAULT_GROUP_COMMIT_MILLISECONDS),
                DEFAULT_GROUP_COMMIT_FRAMES,
            ),
        );
        let bound_address = running.service.bound_network_address();
        assert!(
            bound_address.is_ipv6() && bound_address.ip().is_unspecified(),
            "该绑在 IPv6 未指定地址上：{bound_address}"
        );
        let socket_path = running.service.socket_path().to_path_buf();
        let ServiceUnderTest {
            service,
            directory: directory_kept_until_the_end,
        } = running;
        let stop_started = Instant::now();
        service.stop().expect("绑在 [::] 上的服务停得干净");
        let stop_took = stop_started.elapsed();
        assert!(
            stop_took < STOP_TIME_LIMIT,
            "停服用了 {stop_took:?}，该在 {STOP_TIME_LIMIT:?} 之内"
        );
        assert!(!socket_path.exists(), "停服之后 socket 不在");
        drop(directory_kept_until_the_end);
    }
}
