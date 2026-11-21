//! 判段在真服务里登记它的临时库：登记键与持有者的写法、一把续着租的认领、续租、放掉与列出。

use super::*;

// ============================== 判段在真服务里登记它的临时库 ==============================
//
// 判段的库在临时 prune-server 里（一次性库；或进程环境里设着临时服务的标记、全仓那一份开在临时服务里）、配了 GPU 时，
// GPU 批处理服务连的是真服务，看不见那份临时服务里的待判行。判段在写第一行待判行之前往真服务认领一把登记键
// （[`TemporaryLibraryRegistration::claim_key`]：键里是临时服务连哪、键空间前缀与标签），一条线程每隔
// [`TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_INTERVAL`] 续一次租（同一持有者再认领一次），判段交回时守卫放掉
// （[`TemporaryLibraryRegisteredWithTheRealService`]）；进程被杀，租期 [`TEMPORARY_LIBRARY_REGISTRATION_LEASE`] 一过自己失效。
// 登记只是一把认领，真服务里一行数据行都不写。GPU 批处理服务每一圈列认领（prune-server 操作码 13，前缀
// [`TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX`]），照 [`TemporaryLibraryRegistration::from_claim_key`] 解出连哪、用哪段键空间，
// 去那份临时服务取待判行、把结论写回那一份。

/// 登记键的族前缀：GPU 批处理服务按它列认领，得到这一圈要连的临时库。
pub const TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX: &[u8] = b"gpu-claim/temporary-library/";

/// 登记的持有者身份的头：身份是 `temporary-library/<这一台>/<登记者>`（[`TemporaryLibraryRegistration::holder_identity`]）。
const TEMPORARY_LIBRARY_REGISTRATION_HOLDER_IDENTITY_PREFIX: &str = "temporary-library/";

/// 登记键、载荷与持有者身份的写法，逐字（[`TemporaryLibraryRegistration`] 照它编解）。它的 SHA-256 是登记写法那把键
/// （[`temporary_library_registration_protocol_key`]）：写法一变这把键跟着变。
pub const TEMPORARY_LIBRARY_REGISTRATION_LAYOUT: &str = "登记键：gpu-claim/temporary-library/<这一台在 HOSTS 里的地址>/<进程号>-<进程开机以来的启动刻，时钟滴答数>/<载荷的小写十六进制>；\
载荷依次四段，每段是 u16 大端字节长接那么多字节：临时 prune-server 的 unix socket 路径（没有是空）、它的 TCP 地址 主机:端口（没有是空）、\
键空间前缀（键原样那一份是空，别的头一个字节是 0xE0）、给人看的标签；\
持有者身份：temporary-library/<这一台在 HOSTS 里的地址>/<进程号>-<进程开机以来的启动刻，时钟滴答数>";

/// 登记写法那把键：[`TEMPORARY_LIBRARY_REGISTRATION_LAYOUT`] 的 SHA-256（GPU 批处理服务照它认登记的写法，判段不拿它问服务收不收）。
#[must_use]
pub fn temporary_library_registration_protocol_key() -> [u8; DIGEST_BYTES] {
    sha256_digest(TEMPORARY_LIBRARY_REGISTRATION_LAYOUT.as_bytes())
}

/// 登记的租期：判段活着时每隔 [`TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_INTERVAL`] 续一次；进程被杀，这么久之后列认领不再列出它。
pub const TEMPORARY_LIBRARY_REGISTRATION_LEASE: std::time::Duration =
    std::time::Duration::from_secs(30);

/// 续租的间隔：租期的三分之一，中间丢一次续租照样续得上。
pub const TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_INTERVAL: std::time::Duration =
    std::time::Duration::from_secs(10);

const _: () = assert!(
    TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_INTERVAL.as_secs() * 3
        <= TEMPORARY_LIBRARY_REGISTRATION_LEASE.as_secs(),
    "续租间隔不长过租期的三分之一"
);

/// 续租没续上时打的那一行（标准错误）的头：`<前缀> key=<登记键> <为什么>`。
pub const TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_FAILED_LINE_PREFIX: &str =
    "CRASH_LIBRARY_TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_FAILED";

/// 这个进程里登记过几份临时库：标签后面接这个号，同一个进程里同一份库同时登记两回也是两把键，一回放手不撤掉另一回。
static TEMPORARY_LIBRARY_REGISTRATIONS_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 登记那一份临时库之前判段这一侧先拒的几种（交回为什么，一行待判行都不写）；都不是交 `None`：
/// 连不上（socket 与 TCP 地址都没有）；键空间前缀不空而头一个字节不是 [`DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE`]；
/// 指到真服务（socket 解开符号链接之后就是 `real_service` 那个 socket，或 TCP 地址就是 `real_service` 的地址、多机配置里真服务的
/// `real_network_address`）：GPU 批处理服务照这样的登记连过去，结论就写进真库了。
#[must_use]
pub fn problem_of_a_temporary_library_registration(
    service: &TemporaryServiceOfTheLibrary,
    real_service: &RouteAddress,
    real_network_address: Option<&str>,
) -> Option<String> {
    if service.socket_path.is_none() && service.network_address.is_none() {
        return Some(
            "判段的库在临时 prune-server 里，却既没有它的 socket 也没有它的 TCP 地址：登记了 GPU 批处理服务也连不上，不登记、不写待判行"
                .to_string(),
        );
    }
    if service
        .key_space_prefix
        .first()
        .is_some_and(|leading_byte| *leading_byte != DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE)
    {
        return Some(format!(
            "键空间前缀 {} 不空而头一个字节不是 {DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE:#04x}：不是一次性库的名字空间，不登记、不写待判行",
            hexadecimal_text(&service.key_space_prefix)
        ));
    }
    let resolved = |path: &Path| std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let names_the_real_socket = match (real_service, &service.socket_path) {
        (RouteAddress::UnixSocket(real_socket), Some(socket_path)) => {
            resolved(real_socket) == resolved(socket_path)
        }
        (RouteAddress::UnixSocket(_), None) | (RouteAddress::Network(_), Some(_) | None) => false,
    };
    let names_the_real_network_address = match (real_service, &service.network_address) {
        (RouteAddress::Network(real_address), Some(address)) => real_address == address,
        (RouteAddress::Network(_), None) | (RouteAddress::UnixSocket(_), Some(_) | None) => false,
    } || service
        .network_address
        .as_deref()
        .is_some_and(|address| real_network_address == Some(address));
    if names_the_real_socket || names_the_real_network_address {
        return Some(format!(
            "判段的库指到真服务（socket {:?}、TCP {:?}，真服务是 {real_service:?}、多机配置里的地址 {real_network_address:?}）：登记了结论就写进真库，\
             不登记、不写待判行",
            service.socket_path, service.network_address
        ));
    }
    None
}

/// 一份登记：哪一台（`HOSTS` 里的地址）、哪个进程登记的（`<进程号>-<启动刻>`）、临时服务连哪与键空间前缀、给人看的标签。
/// 编成登记键与持有者身份照 [`TEMPORARY_LIBRARY_REGISTRATION_LAYOUT`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporaryLibraryRegistration {
    pub host: String,
    pub registrant: String,
    pub service: TemporaryServiceOfTheLibrary,
    pub label: String,
}

impl TemporaryLibraryRegistration {
    /// 这个进程此刻登记 `service`：登记者是这个进程号连同它的启动刻（`/proc/self/stat` 第 22 项，进程号被别的进程重用也分得开），
    /// 标签后面接 `#<这个进程里第几份登记>`。
    ///
    /// # Errors
    /// `/proc/self/stat` 读不了或认不出启动刻。
    pub fn of_this_process(
        this_host: &crate::gpu_card_configuration::HostAddress,
        service: TemporaryServiceOfTheLibrary,
        label: &str,
    ) -> Result<Self, String> {
        let process_status_line = std::fs::read_to_string("/proc/self/stat").map_err(|error| {
            format!("读不了 /proc/self/stat（登记者带这个进程的启动刻）：{error}")
        })?;
        let start_time =
            crate::gpu_card_configuration::start_time_since_boot_in_clock_ticks_from_the_process_status_line(
                &process_status_line,
            )?;
        let serial = TEMPORARY_LIBRARY_REGISTRATIONS_IN_THIS_PROCESS.fetch_add(1, Ordering::SeqCst);
        Ok(Self {
            host: this_host.as_str().to_string(),
            registrant: format!("{}-{start_time}", std::process::id()),
            service,
            label: format!("{label}#{serial}"),
        })
    }

    /// 持有者身份：`temporary-library/<这一台>/<登记者>`。
    #[must_use]
    pub fn holder_identity(&self) -> Vec<u8> {
        format!(
            "{TEMPORARY_LIBRARY_REGISTRATION_HOLDER_IDENTITY_PREFIX}{}/{}",
            self.host, self.registrant
        )
        .into_bytes()
    }

    /// 载荷：socket 路径、TCP 地址、键空间前缀、标签四段，每段 u16 大端字节长接那么多字节。
    ///
    /// # Errors
    /// 有一段长过 u16 装得下的字节数。
    pub fn payload(&self) -> Result<Vec<u8>, String> {
        use std::os::unix::ffi::OsStrExt;
        let socket_path_bytes = self
            .service
            .socket_path
            .as_ref()
            .map_or(&[][..], |socket_path| socket_path.as_os_str().as_bytes());
        let network_address_bytes = self
            .service
            .network_address
            .as_deref()
            .map_or(&[][..], str::as_bytes);
        let mut payload = Vec::new();
        for (field_name, field) in [
            ("socket 路径", socket_path_bytes),
            ("TCP 地址", network_address_bytes),
            ("键空间前缀", self.service.key_space_prefix.as_slice()),
            ("标签", self.label.as_bytes()),
        ] {
            let length = u16::try_from(field.len()).map_err(|_too_long| {
                format!(
                    "登记载荷的{field_name}有 {} 字节，长过 u16 装得下的",
                    field.len()
                )
            })?;
            payload.extend_from_slice(&length.to_be_bytes());
            payload.extend_from_slice(field);
        }
        Ok(payload)
    }

    /// 登记键：`gpu-claim/temporary-library/<这一台>/<登记者>/<载荷的小写十六进制>`。
    ///
    /// # Errors
    /// 这一台或登记者里有 `/`、是空的；载荷编不出（[`Self::payload`]）。
    pub fn claim_key(&self) -> Result<Vec<u8>, String> {
        for (field_name, field) in [("这一台", &self.host), ("登记者", &self.registrant)] {
            if field.is_empty() || field.contains('/') {
                return Err(format!(
                    "登记键里的{field_name}「{field}」是空的或带 /：解的一方按 / 切不开"
                ));
            }
        }
        let mut key = TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX.to_vec();
        key.extend_from_slice(self.host.as_bytes());
        key.push(b'/');
        key.extend_from_slice(self.registrant.as_bytes());
        key.push(b'/');
        key.extend_from_slice(hexadecimal_text(&self.payload()?).as_bytes());
        Ok(key)
    }

    /// [`Self::claim_key`] 的逆（GPU 批处理服务列出认领之后照它解）。
    ///
    /// # Errors
    /// 不是登记键的族前缀起头；前缀之后不是 UTF-8、切不成三段（这一台、登记者、载荷）；载荷不是小写十六进制、段长对不上、末尾多字节；
    /// TCP 地址或标签不是 UTF-8。
    pub fn from_claim_key(key: &[u8]) -> Result<Self, String> {
        use std::os::unix::ffi::OsStringExt;
        let not_a_registration = |why: &str| {
            format!(
                "认领键 {} 不是一份临时库登记：{why}",
                String::from_utf8_lossy(key)
            )
        };
        let after_the_prefix = key
            .strip_prefix(TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX)
            .ok_or_else(|| not_a_registration("不是登记键的族前缀起头"))?;
        let text = std::str::from_utf8(after_the_prefix)
            .map_err(|_not_utf8| not_a_registration("前缀之后不是 UTF-8"))?;
        let mut parts = text.splitn(3, '/');
        let (Some(host), Some(registrant), Some(payload_text)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return Err(not_a_registration("切不成这一台、登记者、载荷三段"));
        };
        if host.is_empty() || registrant.is_empty() || payload_text.contains('/') {
            return Err(not_a_registration("这一台或登记者是空的，或载荷里带 /"));
        }
        let payload =
            bytes_of_lowercase_hexadecimal(payload_text).map_err(|why| not_a_registration(&why))?;
        let mut position = 0usize;
        let mut fields: Vec<Vec<u8>> = Vec::with_capacity(4);
        // 迭代上界：四段
        for _field in 0..4 {
            let length_bytes = payload
                .get(position..position + 2)
                .ok_or_else(|| not_a_registration("载荷读不满一段的字节长"))?;
            let length = usize::from(u16::from_be_bytes([length_bytes[0], length_bytes[1]]));
            position += 2;
            let field = payload
                .get(position..position + length)
                .ok_or_else(|| not_a_registration("载荷读不满一段的字节"))?;
            fields.push(field.to_vec());
            position += length;
        }
        if position != payload.len() {
            return Err(not_a_registration("载荷四段之后还有字节"));
        }
        let mut fields = fields.into_iter();
        let mut next_field = || fields.next().expect("上面的循环恰好读了四段");
        let socket_path_bytes = next_field();
        let network_address_bytes = next_field();
        let key_space_prefix = next_field();
        let label_bytes = next_field();
        let text_of = |bytes: Vec<u8>, field_name: &str| {
            String::from_utf8(bytes)
                .map_err(|_not_utf8| not_a_registration(&format!("{field_name}不是 UTF-8")))
        };
        let network_address = text_of(network_address_bytes, "TCP 地址")?;
        Ok(Self {
            host: host.to_string(),
            registrant: registrant.to_string(),
            service: TemporaryServiceOfTheLibrary {
                socket_path: (!socket_path_bytes.is_empty())
                    .then(|| PathBuf::from(std::ffi::OsString::from_vec(socket_path_bytes))),
                network_address: (!network_address.is_empty()).then_some(network_address),
                key_space_prefix,
            },
            label: text_of(label_bytes, "标签")?,
        })
    }
}

/// 小写十六进制文本还原成字节（[`hexadecimal_text`] 的逆）。
///
/// # Errors
/// 字符数是奇数；有不是 `0`–`9`、`a`–`f` 的字符。
fn bytes_of_lowercase_hexadecimal(text: &str) -> Result<Vec<u8>, String> {
    let digit_value = |digit: u8| match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        _ => Err(format!("「{}」不是小写十六进制数字", char::from(digit))),
    };
    let (pairs, odd_digit) = text.as_bytes().as_chunks::<2>();
    if !odd_digit.is_empty() {
        return Err(format!("十六进制文本有 {} 个字符，不是偶数", text.len()));
    }
    pairs
        .iter()
        .map(|[high, low]| Ok(digit_value(*high)? * 16 + digit_value(*low)?))
        .collect()
}

/// 判段手里的那一份登记（[`register_the_temporary_library_with_the_service`] 交回）：一条线程每隔续租间隔以登记的持有者身份
/// 再认领一次；守卫丢掉时停那条线程，它放掉登记键（只放这个持有者名下的）、关掉客户端再收尾。
pub struct TemporaryLibraryRegisteredWithTheRealService {
    registration: TemporaryLibraryRegistration,
    claim_key: Vec<u8>,
    /// 丢掉它就是叫续租线程停：线程等着它，等到断开就放登记、收尾。
    stop_renewing: Option<mpsc::Sender<()>>,
    renewer: Option<std::thread::JoinHandle<()>>,
    /// 续租线程续不上时记下的那一句（交回错误的处置办法下；退出进程的处置办法下进程已经退了）。
    renewal_failure: Arc<Mutex<Option<String>>>,
}

impl TemporaryLibraryRegisteredWithTheRealService {
    #[must_use]
    pub fn registration(&self) -> &TemporaryLibraryRegistration {
        &self.registration
    }

    #[must_use]
    pub fn claim_key(&self) -> &[u8] {
        &self.claim_key
    }

    /// 续租线程续不上时记下的那一句；一直续得上是 `None`。
    #[must_use]
    pub fn renewal_failure(&self) -> Option<String> {
        self.renewal_failure
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl Drop for TemporaryLibraryRegisteredWithTheRealService {
    fn drop(&mut self) {
        drop(self.stop_renewing.take());
        if let Some(renewer) = self.renewer.take() {
            if renewer.join().is_err() {
                eprintln!(
                    "{TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_FAILED_LINE_PREFIX} key={} 续租登记的线程 panic 了：登记没放，租期一过自己失效",
                    String::from_utf8_lossy(&self.claim_key)
                );
            }
        }
    }
}

/// 容错客户端报的错：照 `failure_handling` 打一行 [`PRUNE_SERVER_FAILURE_LINE_PREFIX`]、照客户端的退出码退出进程，或交回那一行。
fn registration_client_failed(
    failure_handling: PruneServerFailureHandling,
    operation: &str,
    failure: &ClientFailure,
) -> String {
    let exit_code = failure.exit_code();
    let line =
        format!("{PRUNE_SERVER_FAILURE_LINE_PREFIX} exit_code={exit_code} {operation}：{failure}");
    match failure_handling {
        PruneServerFailureHandling::ExitTheProcess => {
            eprintln!("{line}");
            std::process::exit(exit_code)
        }
        PruneServerFailureHandling::ReturnTheError => line,
    }
}

/// 以 `holder_identity` 认领登记键 `claim_key`、租期 `lease`（头一回与每一回续租都是这一句：同一持有者再认领一次，到期往后挪）。
///
/// # Errors
/// 租期装不进 u32 毫秒；客户端报错（照 `failure_handling`）；登记键被别的持有者认领着、或回的结局不是认领到。
fn claim_the_registration(
    client: &mut ReliableClient,
    holder_identity: &[u8],
    claim_key: &[u8],
    lease: std::time::Duration,
    failure_handling: PruneServerFailureHandling,
) -> Result<(), String> {
    let lease_milliseconds = u32::try_from(lease.as_millis())
        .map_err(|_too_long| format!("登记的租期 {lease:?} 装不进 u32 毫秒"))?;
    let outcomes = client
        .claim_as(
            holder_identity,
            &[(Table::Conclusions, claim_key.to_vec())],
            lease_milliseconds,
        )
        .map_err(|failure| {
            registration_client_failed(failure_handling, "在真服务里认领临时库的登记键", &failure)
        })?;
    match outcomes.as_slice() {
        [ClaimOutcome::Claimed] => Ok(()),
        [ClaimOutcome::HeldByOther {
            holder_identity: other_holder,
            ..
        }] => Err(format!(
            "临时库的登记键 {} 被别的持有者 {} 认领着",
            String::from_utf8_lossy(claim_key),
            String::from_utf8_lossy(other_holder)
        )),
        [ClaimOutcome::AlreadyConcluded] => Err(format!(
            "临时库的登记键 {} 服务答已经有结论：登记键不该有值",
            String::from_utf8_lossy(claim_key)
        )),
        outcomes_other_than_one => Err(format!(
            "认领一把临时库的登记键，服务回了 {} 个结局",
            outcomes_other_than_one.len()
        )),
    }
}

/// 以 `holder_identity` 放掉登记键 `claim_key`（只放这个持有者名下的）。
///
/// # Errors
/// 客户端报错。
fn release_the_registration(
    client: &mut ReliableClient,
    holder_identity: &[u8],
    claim_key: &[u8],
) -> Result<(), ClientFailure> {
    client
        .release_claims_as(holder_identity, &[(Table::Conclusions, claim_key.to_vec())])
        .map(|_outcome_of_each_key| ())
}

/// 续租线程：每隔 `renewal_interval` 以 `holder_identity` 再认领一次 `claim_key`，直到 `stop_requested` 断开（守卫丢掉）；
/// 断开之后放掉登记键、关掉客户端与它的追加日志目录。续不上就记进 `renewal_failure`、打一行
/// [`TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_FAILED_LINE_PREFIX`]、不再续（客户端报的错在退出进程的处置办法下当场退出）。
#[allow(
    clippy::too_many_arguments,
    reason = "客户端、它的追加日志目录、持有者、登记键、租期、间隔、处置办法、叫停与记失败的地方，各是各的东西"
)]
#[allow(
    clippy::needless_pass_by_value,
    reason = "客户端与它的追加日志目录归这条线程，线程收尾时放"
)]
fn renew_the_registration_until_told_to_stop(
    mut client: ReliableClient,
    journal_directory: ExclusiveTemporaryDirectory,
    holder_identity: &[u8],
    claim_key: &[u8],
    lease: std::time::Duration,
    renewal_interval: std::time::Duration,
    failure_handling: PruneServerFailureHandling,
    stop_requested: &mpsc::Receiver<()>,
    renewal_failure: &Mutex<Option<String>>,
) {
    // 迭代上界：守卫丢掉之前每个续租间隔一圈；续不上就出圈、不再续
    loop {
        match stop_requested.recv_timeout(renewal_interval) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Err(problem) = claim_the_registration(
                    &mut client,
                    holder_identity,
                    claim_key,
                    lease,
                    failure_handling,
                ) {
                    eprintln!(
                        "{TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_FAILED_LINE_PREFIX} key={} {problem}",
                        String::from_utf8_lossy(claim_key)
                    );
                    *renewal_failure
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner) = Some(problem);
                    return;
                }
            }
        }
    }
    if let Err(failure) = release_the_registration(&mut client, holder_identity, claim_key) {
        eprintln!(
            "{TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_FAILED_LINE_PREFIX} key={} 放不掉登记：{failure}；租期一过自己失效",
            String::from_utf8_lossy(claim_key)
        );
    }
    drop(client);
    drop(journal_directory);
}

/// 在 `route_of_the_real_service` 那份服务里登记 `registration`：认领登记键（租期 `lease`），起续租线程（每隔 `renewal_interval` 续一次），
/// 交回守卫（丢掉时放掉登记）。连不上、认领不成的客户端错照 `failure_handling`（判段是退出进程，照客户端的退出码退：75 服务不可用）。
///
/// # Errors
/// 登记键编不出、持有者身份长过客户端身份的上限；建不了追加日志的一次性目录；连不上、认领不成（交回错误的处置办法下）；
/// 登记键被别的持有者认领着；起不了续租线程。
pub fn register_the_temporary_library_with_the_service(
    route_of_the_real_service: &RouteAddress,
    registration: TemporaryLibraryRegistration,
    lease: std::time::Duration,
    renewal_interval: std::time::Duration,
    failure_handling: PruneServerFailureHandling,
) -> Result<TemporaryLibraryRegisteredWithTheRealService, String> {
    let claim_key = registration.claim_key()?;
    let holder_identity = registration.holder_identity();
    if holder_identity.len() > prune_server::protocol::MAXIMUM_CLIENT_IDENTITY_BYTES {
        return Err(format!(
            "临时库登记的持有者身份 {} 有 {} 字节，长过 {} 字节",
            String::from_utf8_lossy(&holder_identity),
            holder_identity.len(),
            prune_server::protocol::MAXIMUM_CLIENT_IDENTITY_BYTES
        ));
    }
    let journal_directory =
        ExclusiveTemporaryDirectory::create("temporary-library-registration-journal")?;
    let mut client = ReliableClient::open(ReliableClientSettings::new(
        route_of_the_real_service.clone(),
        journal_directory.join(DEFAULT_JOURNAL_FILE_NAME),
    ))
    .map_err(|failure| {
        registration_client_failed(failure_handling, "连真服务登记临时库", &failure)
    })?;
    claim_the_registration(
        &mut client,
        &holder_identity,
        &claim_key,
        lease,
        failure_handling,
    )?;
    let (stop_renewing, stop_requested) = mpsc::channel::<()>();
    let renewal_failure = Arc::new(Mutex::new(None));
    let renewal_failure_of_the_renewer = Arc::clone(&renewal_failure);
    let claim_key_of_the_renewer = claim_key.clone();
    let renewer = std::thread::Builder::new()
        .name("temporary-library-registration-renewer".to_string())
        .spawn(move || {
            renew_the_registration_until_told_to_stop(
                client,
                journal_directory,
                &holder_identity,
                &claim_key_of_the_renewer,
                lease,
                renewal_interval,
                failure_handling,
                &stop_requested,
                &renewal_failure_of_the_renewer,
            );
        })
        .map_err(|error| {
            format!("起不了续租临时库登记的线程：{error}；登记没放，租期一过自己失效")
        })?;
    Ok(TemporaryLibraryRegisteredWithTheRealService {
        registration,
        claim_key,
        stop_renewing: Some(stop_renewing),
        renewer: Some(renewer),
        renewal_failure,
    })
}

/// 判段登记它的临时库（产品那一路）：真服务是忽略改连接地址的那几个变量算出的那一处（[`prune_server_route_of_the_variables`]
/// 三个变量都给空，平常跑连的那一处，也就是 GPU 批处理服务起来时连的那一处）；先照 [`problem_of_a_temporary_library_registration`]
/// 拒指到真服务、前缀不合、连不上的；这一台取多机配置判法认出的那一项；登记照 [`register_the_temporary_library_with_the_service`]，
/// 租期 [`TEMPORARY_LIBRARY_REGISTRATION_LEASE`]、每 [`TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_INTERVAL`] 续一次，客户端错退出进程。
///
/// # Errors
/// 真服务的地址算不出；判段这一侧先拒的那几种；这一台认不出；同 [`register_the_temporary_library_with_the_service`]。
pub fn register_the_temporary_library_with_the_real_service(
    service: &TemporaryServiceOfTheLibrary,
    label: &str,
) -> Result<TemporaryLibraryRegisteredWithTheRealService, String> {
    let configuration_path = default_multi_host_configuration_path();
    let not_set = std::ffi::OsStr::new("");
    let real_service =
        prune_server_route_of_the_variables(not_set, not_set, not_set, &configuration_path)
            .map_err(|refusal| refusal.failure_line())?;
    let real_network_address = read_configuration_value(&configuration_path, NETWORK_ADDRESS_KEY)
        .ok()
        .flatten()
        .filter(|configured_address| !configured_address.is_empty());
    if let Some(problem) = problem_of_a_temporary_library_registration(
        service,
        &real_service,
        real_network_address.as_deref(),
    ) {
        return Err(problem);
    }
    let addresses = crate::gpu_card_configuration::multi_host_addresses_of_the_caller()?;
    let registration = TemporaryLibraryRegistration::of_this_process(
        addresses.this_host(),
        service.clone(),
        label,
    )?;
    register_the_temporary_library_with_the_service(
        &real_service,
        registration,
        TEMPORARY_LIBRARY_REGISTRATION_LEASE,
        TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_INTERVAL,
        PruneServerFailureHandling::ExitTheProcess,
    )
}

/// `route` 那份服务里此刻列得出的临时库登记（列认领，前缀 [`TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX`]，租期没到的）：
/// 键、持有者身份、剩余租期原样，解法照 [`TemporaryLibraryRegistration::from_claim_key`]。
///
/// # Errors
/// 建不了追加日志的一次性目录；连不上、列不成。
pub fn temporary_library_registrations_listed_by_the_service(
    route: &RouteAddress,
) -> Result<Vec<ListedClaim>, String> {
    let journal_directory =
        ExclusiveTemporaryDirectory::create("temporary-library-registration-listing-journal")?;
    let mut client = ReliableClient::open(ReliableClientSettings::new(
        route.clone(),
        journal_directory.join(DEFAULT_JOURNAL_FILE_NAME),
    ))
    .map_err(|failure| format!("连不上 {route:?}：{failure}"))?;
    client
        .list_claims(TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX)
        .map_err(|failure| format!("在 {route:?} 列临时库登记：{failure}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一份写死的登记：两样连接地址、一次性库那种名字空间、标签都有。
    fn a_temporary_library_registration(
        host: &crate::gpu_card_configuration::HostAddress,
    ) -> TemporaryLibraryRegistration {
        let mut key_space_prefix = vec![DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE];
        key_space_prefix
            .extend_from_slice(b"disposable-crash-library/registration-unit-test/1-2-3/");
        TemporaryLibraryRegistration::of_this_process(
            host,
            TemporaryServiceOfTheLibrary {
                socket_path: Some(PathBuf::from(
                    "/tmp/singlefs-prune-server-registration-unit-test/socket",
                )),
                network_address: Some("127.0.0.2:40123".to_string()),
                key_space_prefix,
            },
            "registration-unit-test",
        )
        .unwrap_or_else(|reason| panic!("{reason}"))
    }

    /// 登记键照写法读回原样：族前缀、这一台、登记者（进程号与启动刻）、载荷的小写十六进制；持有者身份是 `temporary-library/<这一台>/<登记者>`。
    /// 没了 TCP 地址的那一份同样读回原样（那一段长度 0）；前缀不是登记族的、载荷末尾多一个字节的、十六进制里有大写的读不出。
    /// 判别力（`crates/mutations.tsv`）：载荷里 TCP 地址与键空间前缀两段的次序对调，这条红。
    #[test]
    fn a_temporary_library_registration_key_reads_back_as_written() {
        let addresses = addresses_whose_host_is_not_the_usual_loopback();
        let registration = a_temporary_library_registration(addresses.this_host());
        let key = registration.claim_key().expect("编得出登记键");
        assert!(
            key.starts_with(b"gpu-claim/temporary-library/127.0.0.2/"),
            "登记键以族前缀与这一台起头：{}",
            String::from_utf8_lossy(&key)
        );
        assert_eq!(
            TemporaryLibraryRegistration::from_claim_key(&key),
            Ok(registration.clone()),
            "读回原样"
        );
        assert_eq!(
            registration.holder_identity(),
            format!("temporary-library/127.0.0.2/{}", registration.registrant).into_bytes(),
            "持有者身份"
        );
        let without_a_network_address = TemporaryLibraryRegistration {
            service: TemporaryServiceOfTheLibrary {
                network_address: None,
                ..registration.service.clone()
            },
            ..registration.clone()
        };
        assert_eq!(
            TemporaryLibraryRegistration::from_claim_key(
                &without_a_network_address.claim_key().expect("编得出登记键")
            ),
            Ok(without_a_network_address),
            "没有 TCP 地址的那一份同样读回原样"
        );
        let mut with_a_foreign_prefix = b"gpu-claim/other-family/".to_vec();
        with_a_foreign_prefix
            .extend_from_slice(&key[TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX.len()..]);
        let mut with_one_more_payload_byte = key.clone();
        with_one_more_payload_byte.extend_from_slice(b"00");
        let with_an_upper_case_digit: Vec<u8> = {
            let mut changed = key.clone();
            let position_of_the_final_digit = changed.len() - 1;
            changed[position_of_the_final_digit] = b'A';
            changed
        };
        for (what, unreadable) in [
            ("前缀不是登记族的", with_a_foreign_prefix),
            ("载荷末尾多一个字节", with_one_more_payload_byte),
            ("十六进制里有大写", with_an_upper_case_digit),
        ] {
            assert!(
                TemporaryLibraryRegistration::from_claim_key(&unreadable).is_err(),
                "{what}：读不出"
            );
        }
    }

    /// 判段这一侧先拒的几种：登记的 socket 就是真服务的 socket、TCP 地址就是多机配置里真服务的地址、TCP 地址就是真服务走的那个地址、
    /// 键空间前缀不空而头一个字节不是一次性库那一个、两样连接地址都没有；一次性库那种（别的 socket、别的地址、0xE0 起头）不拒，
    /// 键原样那一份（前缀空）也不拒。
    /// 判别力（`crates/mutations.tsv`）：不核 socket 是不是真服务那一个，这条红。
    #[test]
    fn a_registration_naming_the_real_service_or_with_a_foreign_prefix_is_refused() {
        let real_socket = PathBuf::from("/tmp/singlefs-registration-unit-test-real/socket");
        let real_service = RouteAddress::UnixSocket(real_socket.clone());
        let mut disposable_prefix = vec![DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE];
        disposable_prefix.extend_from_slice(b"disposable-crash-library/refusal-unit-test/");
        let temporary = TemporaryServiceOfTheLibrary {
            socket_path: Some(PathBuf::from(
                "/tmp/singlefs-registration-unit-test-temporary/socket",
            )),
            network_address: Some("127.0.0.2:40124".to_string()),
            key_space_prefix: disposable_prefix,
        };
        let real_network_address = Some("127.0.0.2:7450");
        assert_eq!(
            problem_of_a_temporary_library_registration(
                &temporary,
                &real_service,
                real_network_address
            ),
            None,
            "一次性库那种：不拒"
        );
        assert_eq!(
            problem_of_a_temporary_library_registration(
                &TemporaryServiceOfTheLibrary {
                    key_space_prefix: Vec::new(),
                    ..temporary.clone()
                },
                &real_service,
                real_network_address
            ),
            None,
            "键原样那一份（前缀空）：不拒"
        );
        for (what, refused, route) in [
            (
                "socket 就是真服务的 socket",
                TemporaryServiceOfTheLibrary {
                    socket_path: Some(real_socket.clone()),
                    ..temporary.clone()
                },
                real_service.clone(),
            ),
            (
                "TCP 地址就是多机配置里真服务的地址",
                TemporaryServiceOfTheLibrary {
                    network_address: real_network_address.map(str::to_string),
                    ..temporary.clone()
                },
                real_service.clone(),
            ),
            (
                "TCP 地址就是真服务走的那个地址",
                TemporaryServiceOfTheLibrary {
                    socket_path: None,
                    network_address: Some("127.0.0.2:7451".to_string()),
                    ..temporary.clone()
                },
                RouteAddress::Network("127.0.0.2:7451".to_string()),
            ),
            (
                "键空间前缀不是一次性库那一个字节起头",
                TemporaryServiceOfTheLibrary {
                    key_space_prefix: b"foreign-prefix/".to_vec(),
                    ..temporary.clone()
                },
                real_service.clone(),
            ),
            (
                "两样连接地址都没有",
                TemporaryServiceOfTheLibrary {
                    socket_path: None,
                    network_address: None,
                    ..temporary.clone()
                },
                real_service.clone(),
            ),
        ] {
            assert!(
                problem_of_a_temporary_library_registration(&refused, &route, real_network_address)
                    .is_some(),
                "{what}：拒"
            );
        }
    }

    /// 场景：起两份一次性库，一份当「真服务」，另一份是判段的库；照判段那一份的临时服务在「真服务」里登记（租期 30 秒）。
    /// 预期：持着守卫时「真服务」里列得出恰好这一份登记（键就是守卫的登记键、持有者是登记的持有者身份、解出来连的就是判段那一份临时服务）；
    /// 守卫丢掉之后列不出（放掉了，不靠租期过期）；「真服务」里一行待判行都没有（登记只是一把认领）。
    /// 判别力（`crates/mutations.tsv`）：放手时不放登记，这条红。
    #[test]
    fn a_temporary_library_registration_is_listed_while_held_and_released_when_its_guard_is_dropped(
    ) {
        let addresses = addresses_whose_host_is_not_the_usual_loopback();
        // 两份一次性库留着：一份替真服务，一份是判段的库
        let standing_in_for_the_real_service = DisposableLibrary::new_under(
            "registration-real-service",
            &std::env::temp_dir(),
            addresses.this_host(),
        )
        .unwrap_or_else(|refusal| panic!("{refusal}"));
        let library_of_the_judging_stage = DisposableLibrary::new_under(
            "registration-judging-stage",
            &std::env::temp_dir(),
            addresses.this_host(),
        )
        .unwrap_or_else(|refusal| panic!("{refusal}"));
        let temporary_service = library_of_the_judging_stage
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("{refusal}"))
            .temporary_service_of_the_library()
            .expect("一次性库在临时服务里");
        let registration = TemporaryLibraryRegistration::of_this_process(
            addresses.this_host(),
            temporary_service.clone(),
            "listed-while-held",
        )
        .unwrap_or_else(|reason| panic!("{reason}"));
        let holder_identity = registration.holder_identity();
        let guard = register_the_temporary_library_with_the_service(
            standing_in_for_the_real_service.route(),
            registration,
            TEMPORARY_LIBRARY_REGISTRATION_LEASE,
            TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_INTERVAL,
            PruneServerFailureHandling::ReturnTheError,
        )
        .unwrap_or_else(|reason| panic!("{reason}"));
        let listed_while_held = temporary_library_registrations_listed_by_the_service(
            standing_in_for_the_real_service.route(),
        )
        .unwrap_or_else(|reason| panic!("{reason}"));
        assert_eq!(
            listed_while_held
                .iter()
                .map(|claim| (claim.key.clone(), claim.holder_identity.clone()))
                .collect::<Vec<_>>(),
            vec![(guard.claim_key().to_vec(), holder_identity)],
            "持着守卫：列得出恰好这一份登记"
        );
        assert_eq!(
            TemporaryLibraryRegistration::from_claim_key(guard.claim_key())
                .map(|registration_read_back| registration_read_back.service),
            Ok(temporary_service),
            "解出来连的就是判段那一份临时服务"
        );
        drop(guard);
        assert_eq!(
            temporary_library_registrations_listed_by_the_service(
                standing_in_for_the_real_service.route()
            )
            .unwrap_or_else(|reason| panic!("{reason}")),
            Vec::new(),
            "守卫丢掉之后列不出：放掉了"
        );
        assert_eq!(
            standing_in_for_the_real_service
                .open_store(PruneServerFailureHandling::ReturnTheError)
                .unwrap_or_else(|refusal| panic!("{refusal}"))
                .every_gpu_pending_row()
                .expect("扫得了库"),
            Vec::new(),
            "「真服务」里一行待判行都没有：登记只是一把认领"
        );
    }

    /// 场景：同上起两份一次性库，登记的租期压成 300 毫秒、每 100 毫秒续一次，持着守卫等 1 秒（三个多租期）。
    /// 预期：等完照样列得出这一份登记，续租没记下失败。
    /// 判别力（`crates/mutations.tsv`）：续租线程一次都不续（只等放手），这条红。
    #[test]
    fn a_temporary_library_registration_is_renewed_while_it_is_held() {
        const LEASE_OF_THE_UNIT_TEST: std::time::Duration = std::time::Duration::from_millis(300);
        const RENEWAL_INTERVAL_OF_THE_UNIT_TEST: std::time::Duration =
            std::time::Duration::from_millis(100);
        const HELD_FOR: std::time::Duration = std::time::Duration::from_secs(1);
        let addresses = addresses_whose_host_is_not_the_usual_loopback();
        // 两份一次性库留着：一份替真服务，一份是判段的库
        let standing_in_for_the_real_service = DisposableLibrary::new_under(
            "renewal-real-service",
            &std::env::temp_dir(),
            addresses.this_host(),
        )
        .unwrap_or_else(|refusal| panic!("{refusal}"));
        let library_of_the_judging_stage = DisposableLibrary::new_under(
            "renewal-judging-stage",
            &std::env::temp_dir(),
            addresses.this_host(),
        )
        .unwrap_or_else(|refusal| panic!("{refusal}"));
        let temporary_service = library_of_the_judging_stage
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("{refusal}"))
            .temporary_service_of_the_library()
            .expect("一次性库在临时服务里");
        let guard = register_the_temporary_library_with_the_service(
            standing_in_for_the_real_service.route(),
            TemporaryLibraryRegistration::of_this_process(
                addresses.this_host(),
                temporary_service,
                "renewed-while-held",
            )
            .unwrap_or_else(|reason| panic!("{reason}")),
            LEASE_OF_THE_UNIT_TEST,
            RENEWAL_INTERVAL_OF_THE_UNIT_TEST,
            PruneServerFailureHandling::ReturnTheError,
        )
        .unwrap_or_else(|reason| panic!("{reason}"));
        std::thread::sleep(HELD_FOR);
        let listed_after_waiting = temporary_library_registrations_listed_by_the_service(
            standing_in_for_the_real_service.route(),
        )
        .unwrap_or_else(|reason| panic!("{reason}"));
        assert_eq!(
            (
                listed_after_waiting
                    .iter()
                    .map(|claim| claim.key.clone())
                    .collect::<Vec<_>>(),
                guard.renewal_failure()
            ),
            (vec![guard.claim_key().to_vec()], None),
            "持着守卫等过三个多租期：续着租，照样列得出；续租没失败过"
        );
        drop(guard);
    }
}
