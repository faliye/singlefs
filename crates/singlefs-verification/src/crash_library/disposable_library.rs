//! 一次性库：在这个进程里自己起的临时 prune-server（socket 太长时另放一份短目录）与它里面的一段名字空间，
//! 临时服务绑这一台的地址；经产品入口跑的用例把它设成这个进程开的那一份。

use super::*;

/// 这个进程里建过几份一次性库（[`DisposableLibrary::new`] 每次取一个号，名字空间里带它）。
static DISPOSABLE_LIBRARIES_CREATED_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 一次性库自己起的那一份临时 prune-server：在这个进程里起（`prune_server::server::start_in_this_process`），状态目录在它独占的
/// 一次性临时目录里；socket 也放那里，全路径超了客户端认的上限 `prune_server::configuration::MAXIMUM_SOCKET_PATH_BYTES`（`TMPDIR` 嵌得深，
/// 变异门禁每行一个的临时目录那种）时另放进 [`SHORT_SOCKET_DIRECTORY_ROOT`] 下一份只有本用户进得去的短一次性目录。
/// 丢掉时先停服（不再接连接、关掉还连着的连接、等库关上），再删那一两份目录；停不干净是缺陷（还有线程拿着服务），不在 panic 中就 panic。
struct TemporaryPruneServer {
    service: Option<ServiceInThisProcess>,
    service_directory: ExclusiveTemporaryDirectory,
    /// socket 在 `service_directory` 里放不下时另开的那份短目录（随服务放手一起删）；放得下时没有。
    #[expect(dead_code, reason = "只为持有到放手：丢掉时删那份短目录，不读它")]
    short_socket_directory: Option<ExclusiveTemporaryDirectory>,
}

/// 这个进程里起着的临时 prune-server（[`TemporaryPruneServer`]）：socket 路径到 TCP 绑上的地址（`<地址>:<端口>`，IPv6 带方括号）。
/// 判段登记一次性库时照库的 socket 查它的 TCP 地址（[`temporary_service_of_a_library`]）：另一台的 GPU 批处理服务只连得到 TCP。
pub(super) static NETWORK_ADDRESSES_OF_THE_TEMPORARY_SERVICES_IN_THIS_PROCESS: Mutex<
    BTreeMap<PathBuf, String>,
> = Mutex::new(BTreeMap::new());

/// 临时服务的 TCP 绑哪个地址：`this_host`（多机配置 `HOSTS` 里这一台）的地址接内核挑的端口 `:0`，IPv6 地址带方括号。
/// 不读别的变量、不补缺省：这一台是哪一项只由多机配置判法认（[`crate::gpu_card_configuration::multi_host_addresses_of_the_caller`]）。
#[must_use]
pub fn network_address_of_a_temporary_service_on(
    this_host: &crate::gpu_card_configuration::HostAddress,
) -> String {
    let address = this_host.as_str();
    if address.contains(':') {
        format!("[{address}]:0")
    } else {
        format!("{address}:0")
    }
}

/// 临时服务的 socket 在它那份一次性临时目录（或另开的短目录）里的文件名。
const TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME: &str = "socket";
/// 临时服务的状态目录（故障日志与库）在它那份一次性临时目录里的名字。
const TEMPORARY_PRUNE_SERVER_STATE_DIRECTORY_NAME: &str = "state";
/// socket 放不进临时服务自己的目录时，另开短目录的地方（与 `scripts/prune-server/prune_server_client_selftest.py` 的 `selftest_root`
/// 在 `TMPDIR` 太长时退到同一处）。
const SHORT_SOCKET_DIRECTORY_ROOT: &str = "/tmp";
/// 那份短目录的权限：只有本用户进得去（[`SHORT_SOCKET_DIRECTORY_ROOT`] 人人可写，socket 不放进别人进得去的目录）。
const SHORT_SOCKET_DIRECTORY_MODE: u32 = 0o700;

impl TemporaryPruneServer {
    /// 服务的一次性临时目录建在 `temporary_root` 下；socket 放进去超上限时另开 [`SHORT_SOCKET_DIRECTORY_ROOT`] 下的短目录放 socket。
    /// TCP 绑 `this_host`（多机配置 `HOSTS` 里这一台的地址，[`network_address_of_a_temporary_service_on`]）上内核挑的端口：
    /// 判段登记了这份临时服务之后，两台的 GPU 批处理服务都照这个地址连它。起好之后 socket 与绑上的地址记进
    /// [`NETWORK_ADDRESSES_OF_THE_TEMPORARY_SERVICES_IN_THIS_PROCESS`]，丢掉时撤掉。
    ///
    /// # Errors
    /// 建不了一次性临时目录或短目录；服务起不来（socket 路径超了上限、库开不了、端口绑不上，说明连同 prune-server 写的出路）。
    fn start_under(
        temporary_root: &Path,
        this_host: &crate::gpu_card_configuration::HostAddress,
    ) -> Result<Self, String> {
        use std::os::unix::fs::DirBuilderExt;
        let service_directory = ExclusiveTemporaryDirectory::create_in(
            temporary_root,
            "prune-server",
            &std::fs::DirBuilder::new(),
        )?;
        let socket_in_the_service_directory =
            service_directory.join(TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME);
        let short_socket_directory =
            if socket_path_problem(&socket_in_the_service_directory).is_some() {
                let mut owner_only_directory_builder = std::fs::DirBuilder::new();
                owner_only_directory_builder.mode(SHORT_SOCKET_DIRECTORY_MODE);
                Some(ExclusiveTemporaryDirectory::create_in(
                    Path::new(SHORT_SOCKET_DIRECTORY_ROOT),
                    "prune-server-socket",
                    &owner_only_directory_builder,
                )?)
            } else {
                None
            };
        let socket_path = short_socket_directory
            .as_ref()
            .map_or(socket_in_the_service_directory, |short_directory| {
                short_directory.join(TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME)
            });
        let network_address = network_address_of_a_temporary_service_on(this_host);
        let service = start_in_this_process(
            &service_directory.join(TEMPORARY_PRUNE_SERVER_STATE_DIRECTORY_NAME),
            &socket_path,
            &network_address,
        )
        .map_err(|failure| {
            let (problem, next_step) = failure.describe();
            format!(
                "在这个进程里起不来临时 prune-server（{}，TCP {network_address}）：{problem}；{next_step}",
                socket_path.display()
            )
        })?;
        NETWORK_ADDRESSES_OF_THE_TEMPORARY_SERVICES_IN_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(
                socket_path.clone(),
                service.bound_network_address().to_string(),
            );
        Ok(Self {
            service: Some(service),
            service_directory,
            short_socket_directory,
        })
    }

    fn service(&self) -> &ServiceInThisProcess {
        self.service
            .as_ref()
            .expect("服务在放手之前一直在：只有 Drop 取走它")
    }
}

impl Drop for TemporaryPruneServer {
    fn drop(&mut self) {
        let Some(service) = self.service.take() else {
            return;
        };
        NETWORK_ADDRESSES_OF_THE_TEMPORARY_SERVICES_IN_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(service.socket_path());
        if let Err(problem) = service.stop() {
            let message = format!(
                "临时 prune-server（{}）没停干净：{problem}",
                self.service_directory.display()
            );
            if std::thread::panicking() {
                eprintln!("  ✗ {message}");
            } else {
                panic!("{message}");
            }
        }
    }
}

/// 一份一次性库：它自己在这个进程里起的那份临时 prune-server（[`TemporaryPruneServer`]）里的一段名字空间（[`LibraryKeySpace::DisposableLibrary`]），
/// 从空库起；另带一份独占的一次性临时目录（用例自己的配置文件、日志这类放这里）。不读连哪一处的环境变量、不连真服务：测试写不到真服务里去。
/// 丢掉时停掉那份临时服务、删它的目录（名字空间里的行随之删掉），再删那份临时目录。
pub struct DisposableLibrary {
    route: RouteAddress,
    key_space: LibraryKeySpace,
    scratch_directory: ExclusiveTemporaryDirectory,
    temporary_service: TemporaryPruneServer,
    /// 单测构建里这份临时服务占的那一个名额（[`TemporaryServerPlaceInTheTestProcess`]）：排在最后，服务停了、目录删了才还
    #[cfg(test)]
    _place_in_the_test_process: TemporaryServerPlaceInTheTestProcess,
}

impl DisposableLibrary {
    /// 临时服务的 TCP 绑多机配置 `HOSTS` 里这一台的地址（[`crate::gpu_card_configuration::multi_host_addresses_of_the_caller`]）。
    ///
    /// # Errors
    /// 多机配置判法认不出这一台；建不了一次性临时目录；临时 prune-server 起不来。
    pub fn new(label: &str) -> Result<Self, String> {
        let addresses = crate::gpu_card_configuration::multi_host_addresses_of_the_caller()
            .map_err(|reason| {
                format!(
                    "一次性库的临时 prune-server 的 TCP 绑多机配置 HOSTS 里这一台的地址，这一台认不出：{reason}"
                )
            })?;
        Self::new_under(label, &std::env::temp_dir(), addresses.this_host())
    }

    /// 同 [`Self::new`]，只是临时服务的目录与那份临时目录建在 `temporary_root` 下（单测拿一条长目录当 `TMPDIR` 用，不改进程环境：
    /// 同一进程里并行跑的别的用例照旧读 `TMPDIR`），TCP 绑 `this_host` 的地址。
    ///
    /// # Errors
    /// 建不了一次性临时目录；临时 prune-server 起不来。
    pub(super) fn new_under(
        label: &str,
        temporary_root: &Path,
        this_host: &crate::gpu_card_configuration::HostAddress,
    ) -> Result<Self, String> {
        #[cfg(test)]
        let place_in_the_test_process = TemporaryServerPlaceInTheTestProcess::take();
        let temporary_service = TemporaryPruneServer::start_under(temporary_root, this_host)?;
        let socket_path = temporary_service.service().socket_path().to_path_buf();
        let scratch_directory = ExclusiveTemporaryDirectory::create_in(
            temporary_root,
            &format!("disposable-library-{label}"),
            &std::fs::DirBuilder::new(),
        )?;
        let serial_in_this_process =
            DISPOSABLE_LIBRARIES_CREATED_IN_THIS_PROCESS.fetch_add(1, Ordering::SeqCst);
        let nanoseconds_since_the_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since_the_epoch| since_the_epoch.as_nanos());
        let mut namespace = vec![DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE];
        namespace.extend_from_slice(
            format!(
                "{DISPOSABLE_LIBRARY_NAMESPACE_NAME_PREFIX}{label}/{}-{nanoseconds_since_the_epoch}-{serial_in_this_process}/",
                std::process::id()
            )
            .as_bytes(),
        );
        Ok(Self {
            route: RouteAddress::UnixSocket(socket_path),
            key_space: LibraryKeySpace::DisposableLibrary(namespace),
            scratch_directory,
            temporary_service,
            #[cfg(test)]
            _place_in_the_test_process: place_in_the_test_process,
        })
    }

    /// 它自己起的那份临时 prune-server 的目录（状态目录在里面；socket 放得下时也在里面，[`TemporaryPruneServer`]），丢掉之后不在。
    #[must_use]
    pub fn temporary_service_directory(&self) -> &Path {
        self.temporary_service.service_directory.path()
    }

    /// 那份临时 prune-server 的 TCP 实际绑上的地址（多机配置 `HOSTS` 里这一台的地址），停服之后连不上。
    #[must_use]
    pub fn temporary_service_network_address(&self) -> std::net::SocketAddr {
        self.temporary_service.service().bound_network_address()
    }

    /// 那份独占的一次性临时目录。
    #[must_use]
    pub fn directory(&self) -> &Path {
        self.scratch_directory.path()
    }

    /// 这一份在服务里的键空间。
    #[must_use]
    pub fn key_space(&self) -> &LibraryKeySpace {
        &self.key_space
    }

    /// 那份临时服务连哪一处（用例另连同一份临时服务里别的键空间时用）。
    #[must_use]
    pub fn route(&self) -> &RouteAddress {
        &self.route
    }

    /// 连这一份（一个新的客户端；追加日志在这一端自己的一次性目录里，随这一端放手删掉，[`PruneServerStorage::connect_with_a_journal_of_its_own`]）。
    ///
    /// # Errors
    /// 连不上（交回错误的那一种处置时）；建不了一次性目录。
    pub fn connect(
        &self,
        failure_handling: PruneServerFailureHandling,
    ) -> Result<Arc<PruneServerStorage>, String> {
        PruneServerStorage::connect_with_a_journal_of_its_own(
            self.route.clone(),
            failure_handling,
            self.key_space.clone(),
        )
        .map_err(|error| format!("连不上临时 prune-server（{:?}）：{error}", self.route))
    }

    /// 开这一份（[`VerdictStore::open_the_prune_server`]）。同一份开几次读到的是同一批行（重开、另一个进程开的样子）。
    ///
    /// # Errors
    /// 同 [`Self::connect`]；开库不成（读不成、判定向量表对不上）。
    pub fn open_store(
        &self,
        failure_handling: PruneServerFailureHandling,
    ) -> Result<VerdictStore, String> {
        VerdictStore::open_the_prune_server(self.connect(failure_handling)?)
            .map_err(|error| format!("开不了一次性库：{error:?}"))
    }

    /// 把这一份设成这个进程开的那一份（经产品入口跑的用例用）：交回的守卫在期间，这个进程里 [`CrashLibrary::open`]、
    /// [`LibraryOpenInThisProcess::open_or_share`] 开的都是这一份；守卫丢掉时换回全仓那一份。同一时刻一个进程只设一份：几条用例同时设，
    /// 后来的等先设的那一份的守卫丢掉。是进程内的设置，不是环境变量，不传给子进程。
    /// 测试里经产品入口开库的每一处都要先设自己那一份：没设就开的，开到的是此刻同一个进程里别的用例设着的那一份（那条用例收尾时停掉它的临时服务，
    /// 这一边接着读写就连不上），没人设着时开到全仓那一份（测试不往真服务写数据行）。
    /// 有人持着全仓那一份的守卫（[`TheOneLibraryKeptForThisProcess`]）时，等它们都放手才设。
    #[must_use]
    pub fn make_it_the_library_of_this_process(&self) -> DisposableLibraryOfThisProcess<'_> {
        let serializing = DISPOSABLE_LIBRARY_SETTING
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        *DISPOSABLE_LIBRARY_OF_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(DisposableLibraryChosen {
            route: self.route.clone(),
            key_space: self.key_space.clone(),
        });
        DisposableLibraryOfThisProcess {
            library: self,
            _serializing: serializing,
        }
    }
}

/// 一次性库当路径用（`directory.join(…)`、交给收 `&Path` 的函数）：就是它那份一次性临时目录 [`DisposableLibrary::directory`]。
impl std::ops::Deref for DisposableLibrary {
    type Target = Path;

    fn deref(&self) -> &Path {
        self.scratch_directory.path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一次性库不设任何临时服务的变量也建得出：它在这个进程里自己起一份临时 prune-server，socket 在那份服务独占的目录里（不是
    /// `XDG_RUNTIME_DIR` 下真服务的 socket），经它开得了库（另一个客户端也开得了同一份）；另一个客户端还连着就丢掉
    /// 一次性库，服务照样停掉（TCP 连不上）、那份目录不在。
    /// 判别力（`crates/mutations.tsv`）：一次性库连真服务的 socket、丢掉一次性库时不停临时服务，各让这一条红。
    #[test]
    fn a_disposable_library_starts_its_own_prune_server_in_this_process_and_removes_it_when_dropped(
    ) {
        // 一次性库留着：自证一次性库自己起、自己停临时服务
        let library = DisposableLibrary::new("own-temporary-service")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let service_directory = library.temporary_service_directory().to_path_buf();
        let network_address = library.temporary_service_network_address();
        let socket_path = match library.route() {
            RouteAddress::UnixSocket(socket_path) => socket_path.clone(),
            RouteAddress::Network(address) => {
                panic!("一次性库走它自己那份临时服务的 socket，不走 TCP：{address}")
            }
        };
        // TMPDIR 嵌得深（变异门禁每行一个的临时目录）时 socket 放不进临时服务自己的目录，另放在短目录里
        // （那一路由 a_disposable_library_under_a_temporary_root_too_long_for_the_socket_puts_the_socket_in_a_short_directory_and_removes_both 钉）
        let socket_fits_in_the_service_directory =
            socket_path_problem(&service_directory.join(TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME))
                .is_none();
        if socket_fits_in_the_service_directory {
            assert!(
                socket_path.starts_with(&service_directory),
                "socket 在临时服务自己的目录里：{}",
                socket_path.display()
            );
        } else {
            assert_eq!(
                socket_path.parent().and_then(Path::parent),
                Some(Path::new(SHORT_SOCKET_DIRECTORY_ROOT)),
                "socket 放不进临时服务自己的目录时在 {SHORT_SOCKET_DIRECTORY_ROOT} 下另开的短目录里：{}",
                socket_path.display()
            );
        }
        assert_ne!(
            default_socket_path().ok(),
            Some(socket_path.clone()),
            "不是真服务的 socket"
        );
        assert!(
            std::os::unix::net::UnixStream::connect(&socket_path).is_ok(),
            "建库之后临时服务在，socket 上有人听：{}",
            socket_path.display()
        );
        let this_host = crate::gpu_card_configuration::multi_host_addresses_of_the_caller()
            .unwrap_or_else(|reason| panic!("前提：多机配置判法认得出这一台：{reason}"));
        assert_eq!(
            network_address.ip().to_string(),
            this_host.this_host().as_str(),
            "临时服务的 TCP 绑多机配置 HOSTS 里这一台的地址：{network_address}"
        );
        let first_client = library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let client_still_connected = library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("另一个客户端开同一份库：{refusal}"));
        assert_eq!(
            first_client.library_name(),
            client_still_connected.library_name(),
            "两个客户端开的是同一份"
        );
        drop(first_client);
        drop(library);
        assert!(
            !service_directory.exists(),
            "丢掉之后临时服务的目录不在：{}",
            service_directory.display()
        );
        assert!(
            std::net::TcpStream::connect(network_address).is_err(),
            "丢掉之后临时服务停了，TCP 连不上：{network_address}"
        );
        drop(client_still_connected);
    }

    /// 临时目录的根嵌得深、socket 放进临时服务自己的目录就超上限时（变异门禁每行一个的 `TMPDIR` 那种；这里把那条长目录交给
    /// [`DisposableLibrary::new_under`]，不改进程环境）：一次性库照样起得来，状态目录照旧在长目录下，socket 另放在
    /// [`SHORT_SOCKET_DIRECTORY_ROOT`] 下一份只有本用户进得去的短目录里、不超上限；经它开得了库（另一个客户端也开得了同一份）；
    /// 丢掉之后两份目录都不在。
    /// 判别力（`crates/mutations.tsv`）：去掉退到短目录那一步（socket 照旧放进长目录），临时服务起不来，这条红。
    #[test]
    fn a_disposable_library_under_a_temporary_root_too_long_for_the_socket_puts_the_socket_in_a_short_directory_and_removes_both(
    ) {
        let long_root_parent = ExclusiveTemporaryDirectory::create("long-temporary-root")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let long_temporary_root =
            long_root_parent.join("deeply-nested-temporary-directory-".repeat(4));
        std::fs::create_dir(&long_temporary_root).unwrap_or_else(|error| {
            panic!("建不了长目录 {}：{error}", long_temporary_root.display())
        });
        assert!(
            socket_path_problem(
                &long_temporary_root
                    .join("singlefs-prune-server-0-0")
                    .join(TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME)
            )
            .is_some(),
            "取样点要让 socket 放进临时服务自己的目录就超上限：{}",
            long_temporary_root.display()
        );
        // 一次性库留着：自证 socket 放不下时另开短目录
        let addresses = crate::gpu_card_configuration::multi_host_addresses_of_the_caller()
            .unwrap_or_else(|reason| panic!("前提：多机配置判法认得出这一台：{reason}"));
        let library = DisposableLibrary::new_under(
            "long-temporary-root",
            &long_temporary_root,
            addresses.this_host(),
        )
        .unwrap_or_else(|refusal| panic!("{refusal}"));
        let service_directory = library.temporary_service_directory().to_path_buf();
        let socket_path = match library.route() {
            RouteAddress::UnixSocket(socket_path) => socket_path.clone(),
            RouteAddress::Network(address) => {
                panic!("一次性库走它自己那份临时服务的 socket，不走 TCP：{address}")
            }
        };
        let short_socket_directory = socket_path
            .parent()
            .expect("socket 路径是一份目录里的一个文件，有上一级")
            .to_path_buf();
        assert!(
            service_directory.starts_with(&long_temporary_root),
            "状态目录照旧在长目录下：{}",
            service_directory.display()
        );
        assert_eq!(
            short_socket_directory.parent(),
            Some(Path::new(SHORT_SOCKET_DIRECTORY_ROOT)),
            "socket 在 {SHORT_SOCKET_DIRECTORY_ROOT} 下另开的短目录里：{}",
            socket_path.display()
        );
        assert_eq!(
            socket_path_problem(&socket_path),
            None,
            "短目录里的 socket 不超上限"
        );
        let short_directory_mode = {
            use std::os::unix::fs::PermissionsExt;
            /// `st_mode` 里权限那几位（去掉文件种类那几位）。
            const PERMISSION_BITS_OF_THE_MODE: u32 = 0o777;
            std::fs::metadata(&short_socket_directory)
                .unwrap_or_else(|error| {
                    panic!(
                        "短目录 {} 读不了：{error}",
                        short_socket_directory.display()
                    )
                })
                .permissions()
                .mode()
                & PERMISSION_BITS_OF_THE_MODE
        };
        assert_eq!(
            short_directory_mode,
            SHORT_SOCKET_DIRECTORY_MODE,
            "短目录只有本用户进得去：{}",
            short_socket_directory.display()
        );
        let first_client = library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let second_client = library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("另一个客户端开同一份库：{refusal}"));
        assert_eq!(
            first_client.library_name(),
            second_client.library_name(),
            "两个客户端开的是同一份"
        );
        drop(first_client);
        drop(second_client);
        drop(library);
        assert!(
            !service_directory.exists(),
            "丢掉之后临时服务的目录不在：{}",
            service_directory.display()
        );
        assert!(
            !short_socket_directory.exists(),
            "丢掉之后放 socket 的短目录不在：{}",
            short_socket_directory.display()
        );
    }

    /// 两份一次性库是两段名字空间，头一个字节都是一次性库那一个（与全仓那一份的表标签、服务记账那一段都不撞）。
    #[test]
    fn two_disposable_libraries_take_two_namespaces_led_by_the_disposable_byte() {
        // 一次性库留着：自证两份一次性库两段名字空间
        let one = DisposableLibrary::new("namespace").unwrap_or_else(|refusal| panic!("{refusal}"));
        let another =
            DisposableLibrary::new("namespace").unwrap_or_else(|refusal| panic!("{refusal}"));
        let prefix_of = |library: &DisposableLibrary| match library.key_space() {
            LibraryKeySpace::DisposableLibrary(namespace) => namespace.clone(),
            LibraryKeySpace::TheOneLibrary => panic!("一次性库的键空间不会是全仓那一份"),
        };
        assert_eq!(
            prefix_of(&one).first(),
            Some(&DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE)
        );
        assert_ne!(
            prefix_of(&one),
            prefix_of(&another),
            "两份一次性库两段名字空间"
        );
    }

    /// 场景：一次性库建在单测给的「这一台」（`HOSTS` 里的 127.0.0.2）上。
    /// 预期：它那份临时 prune-server 的 TCP 绑的就是 127.0.0.2（内核挑的端口），不是 127.0.0.1；这个进程里照 socket 查得到这个地址，
    /// 丢掉之后查不到。
    /// 判别力（`crates/mutations.tsv`）：绑的地址写回 `127.0.0.1:0`，这条红。
    #[test]
    fn the_temporary_service_of_a_disposable_library_binds_the_address_of_this_host() {
        let addresses = addresses_whose_host_is_not_the_usual_loopback();
        // 一次性库留着：自证临时服务绑这一台在 HOSTS 里的地址
        let library = DisposableLibrary::new_under(
            "binds-the-address-of-this-host",
            &std::env::temp_dir(),
            addresses.this_host(),
        )
        .unwrap_or_else(|refusal| panic!("{refusal}"));
        let bound = library.temporary_service_network_address();
        assert_eq!(
            bound.ip().to_string(),
            "127.0.0.2",
            "临时服务的 TCP 绑 HOSTS 里这一台的地址：{bound}"
        );
        let socket_path = match library.route() {
            RouteAddress::UnixSocket(socket_path) => socket_path.clone(),
            RouteAddress::Network(address) => {
                panic!("一次性库走它自己那份临时服务的 socket，不走 TCP：{address}")
            }
        };
        let temporary_service = library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("{refusal}"))
            .temporary_service_of_the_library();
        assert_eq!(
            temporary_service,
            Some(TemporaryServiceOfTheLibrary {
                socket_path: Some(socket_path.clone()),
                network_address: Some(bound.to_string()),
                key_space_prefix: library.key_space().key_prefix().to_vec(),
            }),
            "照库查得到它那份临时服务：socket、绑上的 TCP 地址、名字空间"
        );
        drop(library);
        assert_eq!(
            NETWORK_ADDRESSES_OF_THE_TEMPORARY_SERVICES_IN_THIS_PROCESS
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .get(&socket_path),
            None,
            "丢掉之后这个进程里查不到它的 TCP 地址"
        );
    }
}
