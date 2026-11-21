//! 借卡：判段之外要自己开卡的真卡验证（只有借卡白名单里登记的，见下「借卡白名单」）在要开卡的那一刻向这一台的 GPU 批处理验证服务借这一台的全部卡，
//! 用完还；借着的那一段服务不认领新的待判块。借与还只经判定库服务的认领帧与放认领帧走（`prune_server::protocol` 文件头「认领」「放认领」），
//! 不写数据行、不另开通道、不读写服务的状态目录。
//!
//! 借卡白名单：`gpu-verification-server/src/card-lease-borrowers.tsv`（[`CARD_LEASE_BORROWERS_PATH_IN_THE_REPOSITORY`]），一行
//! `<可执行文件名>  # <理由>`，`#` 起头的行与空行不算；经 `include_str!` 编进这个二进制（[`CARD_LEASE_BORROWERS_TEXT`]），改了要重编、
//! `control install` 重装服务才生效。此刻只有 E202（GPU与CPU两种模式GC对照组）的装置二进制 `e202_gpu_cpu_gc_comparison` 一项。
//! 认借的一方只看 `/proc/<借的进程号>/exe` 指到的那个文件的文件名（[`executable_name_of_the_borrower`]）：不看目录、不看包名、不看命令行，
//! 与表里一项逐字相同才放行（二进制从一个包搬到另一个包、同名照认；cargo 测试二进制带 `-<十六位哈希>` 后缀，对不上）。读之前、读之后各认一回
//! 借的进程（进程号、启动刻、开机号，`crate::service_process` 的判法）：两回都在，读到的才是它的，不是进程号被复用之后那一个的。
//! 读不到、借的进程不在或启动刻对不上、文件名不在表里，一律照拒：一把键都不认领，往标准错误先打一行 [`CARD_LEASE_REFUSED_LINE_PREFIX`]
//! （`borrower_process=<借的进程号> executable=<带引号的可执行文件名，认不出写 none>`），再打 `✗` 与出路两行，退
//! [`BORROWER_OFF_THE_WHITELIST_EXIT`]。只有 `lease take` 认：`lease return` 只放借的一方自己名下的键，`lease renew` 只由借到了的 `lease take` 起，
//! `lease list` 只读、一把键都不认领。
//!
//! 三种认领键（只记认领、不当数据行写；以判定库服务的 GPU 认领键前缀起头，放认领帧只收这种键）：
//! - 请求键 `gpu-claim/card-lease-request/<这一台>`（[`CardLeaseKeys::request_key`]）：借的一方持着它，表示有人要借这一台的卡；
//! - 卡键 `gpu-claim/card-lease/<这一台>`（[`CardLeaseKeys::card_key`]）：谁持着它，这一台的卡这一段归谁用；
//! - 等号 `gpu-claim/card-lease-waiting/<这一台>/<排序段>`（[`CardLeaseWaitingTicket::key`]）：一个在等的 `lease take` 一张，排先来后到（下面「先来后到」）。
//!
//! `<这一台>` 是多机配置主机数组里这一台那一项（多机配置判法认的，与卡串里的地址逐字相同）。三种键的租期都取服务配置的认领租期
//! （`service-configuration.txt` 的 `CLAIM_LEASE_MILLISECONDS`）。
//!
//! 先来后到：`lease take` 过了借卡白名单之后、认领请求键之前先认领一张等号，持有者身份是借的一方的持有者身份。排序段是
//! `<开始等的时刻：UNIX 纪元以来的毫秒数，20 位十进制补零>-<借的进程号，10 位补零>-<启动刻，20 位补零>`（[`CardLeaseWaitingTicket`]）：
//! 按键升序列出来的次序就是开始等的次序，同一毫秒的按借的进程号、启动刻分先后。只在同一台里比：同一台的 `lease take` 都在这一台起、读同一个钟。
//! 之后每一回要去认领请求键之前，先按前缀 `gpu-claim/card-lease-waiting/<这一台>/`（[`CardLeaseKeys::waiting_ticket_prefix`]）列这一台的等号
//! （判定库服务的列认领帧），自己那一张排在最前（第 [`FIRST_POSITION_IN_THE_QUEUE`] 张）才去认领请求键，不在最前隔 [`BORROWER_RETRY`] 再列；
//! 等的这一段每过三分之一租期续一次等号。排在前面的借方不在了（它的 `lease take` 被杀、没人续），它的等号到租期失效、自然让出。
//! 请求键与卡键都认领到、起续租进程之前放掉自己的等号；中途退出的每一支（借的进程没了、等号或请求键续不上、读写判定库失败）同样放掉
//! （读写判定库失败的那一支放不掉的，到租期自己失效）。列出来的等号里没有自己那一张（租期过了没续上），当续不上办：放掉认领到的、退 1。
//!
//! 查借着哪几张卡（`lease list`）：对多机配置主机数组里点名了卡的每一台（卡串里有地址是它的项，卡串的取法同 `lease take`），照主机数组的次序，
//! 按前缀列它的卡键、请求键与等号（列认领帧；卡键、请求键只取键逐字相同的那一条），往标准输出打：
//! - 一行 `GPU_CARD_LEASE_HELD host=<台> held_by=<service|borrower|nobody> holder=<带引号的卡键的持有者身份；没人持着写 none>
//!   cards=<带引号的这一台的卡串> lease_left_milliseconds=<卡键的剩余租期；没人持着写 0>`（[`CARD_LEASE_HELD_LINE_PREFIX`]）：持有者身份以
//!   `card-lease/` 起头的是借方持着（`borrower`），别的持有者是服务持着（`service`：服务以它认领待判块的身份持卡键），列不到卡键是没人持着（`nobody`）；
//! - 请求键有人持着时一行 `GPU_CARD_LEASE_REQUESTED host=<台> holder=<带引号的持有者身份> lease_left_milliseconds=<剩余租期>`
//!   （[`CARD_LEASE_REQUESTED_LINE_PREFIX`]）；
//! - 每一张等号照次序一行 `GPU_CARD_LEASE_QUEUED host=<台> position=<第几，从 1 起> holder=<带引号的持有者身份> lease_left_milliseconds=<剩余租期>`
//!   （[`CARD_LEASE_QUEUED_LINE_PREFIX`]）。
//!
//! 持有者身份照打给人看的写法（[`holder_text`]：可见的 ASCII 照原样，别的打小写十六进制）再加引号。读到几台都退 0（一台都没点名卡的不打行）。
//!
//! 服务那一半在取活循环里（`crate::serving_from_the_library` 文件头「借卡」）：服务以它认领待判块的那个持有者身份持卡键。
//!
//! 借的一方是这个二进制的 `lease` 子命令（[`run_the_lease_subcommand`]；`control lease take`、`control lease return <卡串>`、`control lease list`
//! 原样转来）：
//! - 持有者身份 `card-lease/<这一台>/<借的进程号>-<它的启动刻>`（[`CardLeaseBorrower::holder_identity`]）。借的进程号与启动刻
//!   （`/proc/<进程号>/stat` 第 22 项）由借的进程经环境变量 `GPU_CARD_LEASE_BORROWER=<进程号>:<启动刻>` 交给 take、return、renew 每一条子命令，
//!   子命令不看自己的父进程：还卡那一条在借的进程退出之后才跑，那时父进程已经换了。变量没给或写法不对退 2（`lease list` 不读它）。
//! - `lease take`：先照借卡白名单认借的一方（不在表里的照「借卡白名单」那一段拒，不往下走）；再认领一张等号、排到最前（上面「先来后到」），
//!   以这个身份认领请求键（别人持着就每 [`BORROWER_RETRY`] 再列一次等号、再试，一直等），再认领卡键（同样一直等，期间每过三分之一租期续一次请求键与等号）；
//!   等的时候头一回、在等的那一把或自己排第几变了的那一回与之后每 [`WAITING_LINE_PERIOD`] 往标准错误打一行 [`CARD_LEASE_WAITING_LINE_PREFIX`]
//!   （`key=<等号|请求键|卡键> host=<这一台> holder=<在等的那一把的持有者；排在别人后面时是排在最前那一张等号的持有者> remaining_lease_milliseconds=<它的剩余租期>
//!   position=<自己排第几，从 1 起>`）；借的进程不在了就放掉认领到的、退 1。两把都认领到之后放掉等号、起续租进程（`lease renew`：放进它自己的进程组，借的进程所在的组被整组停时它还在、
//!   照常放键；标准输入、标准错误接空，标准输出只回一行就绪行 [`CARD_LEASE_RENEWING_LINE_PREFIX`]，之后不再写），读到就绪行才往标准错误打一行
//!   [`CARD_LEASE_TAKEN_LINE_PREFIX`]（这一台、持有者、续租进程的 `<进程号>:<启动刻>`），往标准输出打这一台的卡串、退 0。
//!   卡串：多机配置 `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`（没写取 `GPU_CARDS_BY_PRIORITY`，与服务开卡读的同一项，
//!   `crate::gpu_judges_on_a_card`）里地址是这一台的那几项，照原样、空白隔开；一项都没有不借、退 78。
//! - `lease renew`（续租进程，只由 `lease take` 起）：每 [`RENEWER_BEAT`] 一拍，先看借的进程还在不在（进程号、启动刻、开机号，
//!   `crate::service_process` 的判法；判不了当不在），不在了放掉两把键、退出；再以 [`PROBE_LEASE_MILLISECONDS`] 毫秒的租期探停止键
//!   `gpu-claim/card-lease-returned/<持有者身份>`（[`returned_key_of`]），别人持着就是 `lease return` 叫停了：放掉两把键、退出；
//!   都不是、上一次续过了三分之一租期才续两把键（续不上的放掉余下的、退出）。
//! - `lease return <卡串>`：以另一个身份（持有者身份接 `/return`，[`returning_identity_of`]）认领停止键，租期取续租间隔（三分之一租期）的两倍；
//!   等续租进程放掉卡键（以那个身份、探的租期试认领卡键：不再是借的一方持着就是放了），至多等 [`RETURN_WAIT_FOR_THE_RENEWER`]；
//!   再以借的一方的身份放两把键（续租进程已不在、没放的也放得掉），退 0；两把键已经不在借的一方名下也退 0（还过了）。
//!   等到续租进程放了的，停止键随即放掉；没等到的留着，租期过了自己失效（续租进程卡住、过后才看停止键的那一种照样停）。
//!   卡串参数给了就行，空串也收（借的命令退 0 而卡串是空的，借的一方当场还，交的就是空串）：不校验，只打进标准错误那一行
//!   [`CARD_LEASE_RETURNING_LINE_PREFIX`]（这一台、持有者、带引号的卡串）；键由这一台与借的一方的身份定。
//! - `lease list`：上面「查借着哪几张卡」。
//!
//! 连判定库服务的路与 `serve` 同一套取法（验证包 `crash_library::prune_server_route_of_the_configuration`）；多机配置照
//! `SINGLEFS_MULTI_HOST_CONFIG`，没设照编这份二进制的那棵仓找（验证包 `gpu_card_configuration::multi_host_configuration_file`），
//! 这一台是主机数组里哪一项照多机配置判法认（`MultiHostAddresses::emitted_by_the_configuration_check`）。每条子命令的判定库客户端
//! 追加日志在它自己的一次性临时目录里，退出时删掉（被信号打死的留在临时目录里）。
//!
//! 退出码：0 借到、还了、续租进程收了、列完了；2 用法错（`lease` 后面认不出、`lease return` 后面一个参数都没给、借的一方的变量没给或写法不对、
//! 弄坏开关设在临时判定库服务之外）；78 多机配置找不到、判法认不出这一台、两个显卡键都没写或写坏（`lease take`、`lease list`）、
//! 这一台一张卡都没点名（`lease take`）；3 连判定库的路走不通（配置错）；连不上、读写判定库失败照判定库客户端的退出码表（75 服务不可用、3 别的错）；
//! 4 借的一方不在借卡白名单里、或认不出它是哪个可执行文件（`lease take`；一把键都不认领）；
//! 1 编进来的服务配置或借卡白名单解不开、建不了临时目录、续租进程起不来或没回就绪行、等的时候借的进程没了、续请求键或等号没续上、自己的等号不在了。
//!
//! 弄坏开关（`crate::break_switches`；card-lease-return-refuses-an-empty-card-string 下 `lease return` 的卡串是空的或只有空白退 2，
//! card-lease-borrower-whitelist-unchecked 下 `lease take` 不认借的一方、谁都借给，card-lease-waiting-order-ignored 下 `lease take` 不看等号次序、
//! 每一回都直接去认领请求键）：这几条子命令只在临时判定库服务底下（设了 `PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET`）读
//! `GPU_VERIFICATION_SERVER_BREAK`，别处设了退 2；续租进程照继承的环境读同一份。

use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use prune_server::gpu_judging_rows::GPU_CLAIM_FAMILY_PREFIX;
use prune_server::protocol::{ClaimOutcome, ClaimReleaseOutcome, ListedClaim, Table};
use prune_server::reliable_client::{ClientFailure, ReliableClient, ReliableClientSettings};
use singlefs_verification::crash_library::{
    prune_server_route_of_the_configuration, ExclusiveTemporaryDirectory,
    PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::gpu_card_configuration::{
    configuration_value, multi_host_configuration_file, parse_cards_by_priority,
    MultiHostAddresses, CARDS_BY_PRIORITY_KEY, CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY,
};
use singlefs_verification::hexadecimal::hexadecimal_text;

use crate::break_switches::{BreakSwitch, BreakSwitches, BREAK_VARIABLE};
use crate::service_configuration::{ServiceConfiguration, SERVICE_CONFIGURATION_TEXT};
use crate::service_process::{process_status_fields_of, ProcessLiveness, ServiceProcessIdentity};

/// 借的一方交给每条子命令的变量：`<借的进程号>:<它的启动刻>`。
pub const BORROWER_VARIABLE: &str = "GPU_CARD_LEASE_BORROWER";
/// 二进制的子命令名：`gpu-verification-server lease <take|return <卡串>|renew|list>`。
pub const LEASE_SUBCOMMAND: &str = "lease";
const TAKE_SUBCOMMAND: &str = "take";
const RETURN_SUBCOMMAND: &str = "return";
const RENEW_SUBCOMMAND: &str = "renew";
const LIST_SUBCOMMAND: &str = "list";
/// 请求键在 GPU 认领键前缀之后的那一段。
const REQUEST_KEY_SEGMENT: &[u8] = b"card-lease-request/";
/// 卡键在 GPU 认领键前缀之后的那一段。
const CARD_KEY_SEGMENT: &[u8] = b"card-lease/";
/// 等号在 GPU 认领键前缀之后的那一段（后面接 `<这一台>/<排序段>`，文件头「先来后到」）。
const WAITING_TICKET_KEY_SEGMENT: &[u8] = b"card-lease-waiting/";
/// 等号键里这一台与排序段之间、排序段里三项之间的分隔。
const WAITING_TICKET_HOST_SEPARATOR: &str = "/";
const WAITING_TICKET_FIELD_SEPARATOR: &str = "-";
/// 排序段里开始等的时刻（UNIX 纪元以来的毫秒数）补零到的位数：`u64` 最多 20 位。
const WAITING_START_DIGITS: usize = 20;
/// 排序段里借的进程号补零到的位数：`u32` 最多 10 位。
const WAITING_PROCESS_NUMBER_DIGITS: usize = 10;
/// 排序段里借的进程的启动刻补零到的位数：`u64` 最多 20 位。
const WAITING_START_TICKS_DIGITS: usize = 20;
/// 排在最前的那一张等号是第几张：排第几从 1 起数（等的那一行的 `position=`、`lease list` 的排队行）。
pub const FIRST_POSITION_IN_THE_QUEUE: usize = 1;
/// 停止键在 GPU 认领键前缀之后的那一段（后面接持有者身份）。
const RETURNED_KEY_SEGMENT: &[u8] = b"card-lease-returned/";
/// 借的一方的持有者身份的开头（后面接 `<这一台>/<进程号>-<启动刻>`）。
const HOLDER_PREFIX: &str = "card-lease/";
/// `lease return` 认领停止键用的身份：借的一方的持有者身份接这一段。
const RETURNING_IDENTITY_SUFFIX: &[u8] = b"/return";
/// 探请求键、停止键与 `lease return` 试卡键的租期：认领到也只占 1 毫秒，过了自己失效。
pub const PROBE_LEASE_MILLISECONDS: u32 = 1;
/// 借的一方等一把键时隔多久再试。
pub const BORROWER_RETRY: Duration = Duration::from_secs(2);
/// 等的时候隔多久往标准错误打一行在等谁。
pub const WAITING_LINE_PERIOD: Duration = Duration::from_secs(30);
/// 续租进程一拍多长：借的进程不在了、被叫停，至多这么久之后放键。
pub const RENEWER_BEAT: Duration = Duration::from_secs(1);
/// `lease return` 等续租进程放卡键的上限：续租进程一拍一秒，三拍等得到；到了照放（续租进程已不在的那种）。
pub const RETURN_WAIT_FOR_THE_RENEWER: Duration = Duration::from_secs(3);
/// `lease return` 隔多久看一次续租进程放了没有。
const RETURN_POLL: Duration = Duration::from_millis(100);
/// 续不续租按三分之一租期：租期过一次续不上，还有两次机会。
const RENEWALS_PER_LEASE: u32 = 3;
/// 停止键的租期是续租间隔的几倍。
const STOP_KEY_LEASE_IN_RENEWAL_INTERVALS: u32 = 2;
/// 服务放出卡键那一行的开头。
pub const CARDS_LENT_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CARDS_LENT";
/// 服务收回卡键那一行的开头。
pub const CARDS_TAKEN_BACK_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CARDS_TAKEN_BACK";
/// `lease take` 等一把键时往标准错误打的那一行的开头。
pub const CARD_LEASE_WAITING_LINE_PREFIX: &str = "GPU_CARD_LEASE_WAITING";
/// `lease take` 借到、打卡串之前往标准错误打的那一行的开头。
pub const CARD_LEASE_TAKEN_LINE_PREFIX: &str = "GPU_CARD_LEASE_TAKEN";
/// 续租进程起来、开始续之前往它的标准输出打的就绪行的开头。
pub const CARD_LEASE_RENEWING_LINE_PREFIX: &str = "GPU_CARD_LEASE_RENEWING";
/// 每条子命令的判定库客户端追加日志在一次性临时目录里的名字。
const JOURNAL_FILE_NAME: &str = "library-journal";
/// 一次性临时目录名里的标签。
const TEMPORARY_DIRECTORY_LABEL: &str = "gpu-card-lease";
/// `lease return` 收到卡串、开始还之前往标准错误打的那一行的开头。
pub const CARD_LEASE_RETURNING_LINE_PREFIX: &str = "GPU_CARD_LEASE_RETURNING";
/// `lease take` 照借卡白名单拒了借的一方时，往标准错误先打的那一行的开头（文件头「借卡白名单」）。
pub const CARD_LEASE_REFUSED_LINE_PREFIX: &str = "GPU_CARD_LEASE_REFUSED";
/// `lease list` 每一台的卡键那一行的开头（文件头「查借着哪几张卡」）。
pub const CARD_LEASE_HELD_LINE_PREFIX: &str = "GPU_CARD_LEASE_HELD";
/// `lease list` 请求键有人持着时那一行的开头。
pub const CARD_LEASE_REQUESTED_LINE_PREFIX: &str = "GPU_CARD_LEASE_REQUESTED";
/// `lease list` 每一张等号那一行的开头。
pub const CARD_LEASE_QUEUED_LINE_PREFIX: &str = "GPU_CARD_LEASE_QUEUED";
/// `lease list` 那一行里卡键没人持着时 `holder=` 后面写的（认得出的身份带引号，与它分得开）。
const HOLDER_NOT_HELD: &str = "none";
/// 借卡白名单在仓里的路径（拒绝那两行点名它）。
pub const CARD_LEASE_BORROWERS_PATH_IN_THE_REPOSITORY: &str =
    "gpu-verification-server/src/card-lease-borrowers.tsv";
/// 编进这个二进制的借卡白名单全文。
pub const CARD_LEASE_BORROWERS_TEXT: &str = include_str!("card-lease-borrowers.tsv");
/// 拒绝那一行里认不出可执行文件名时 `executable=` 后面写的（认得出的名字带引号，与它分得开）。
const EXECUTABLE_NOT_READ: &str = "none";
const SUCCESS_EXIT: u8 = 0;
const USAGE_EXIT: u8 = 2;
const RUN_CONDITION_EXIT: u8 = 78;
/// 连判定库的路走不通（配置错）：与 `serve` 的同一个码。
const LIBRARY_ROUTE_REFUSED_EXIT: u8 = 3;
const LEASE_FAILURE_EXIT: u8 = 1;
/// `lease take` 的借的一方不在借卡白名单里、或认不出它是哪个可执行文件。
pub const BORROWER_OFF_THE_WHITELIST_EXIT: u8 = 4;
const LEASE_USAGE: &str = "gpu-verification-server lease take | lease return <卡串，可以是空串> | lease list（由 python3 gpu-verification-server/control lease … 转来；\
lease renew 只由 lease take 起），take、return 的环境里带 GPU_CARD_LEASE_BORROWER=<借的进程号>:<它的启动刻，/proc/<进程号>/stat 第 22 项>";

/// 这一台的两把键。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardLeaseKeys {
    host: String,
}

impl CardLeaseKeys {
    /// `host` 是多机配置主机数组里这一台那一项（判法认出的原样）。
    #[must_use]
    pub fn of_this_host(host: &str) -> Self {
        Self {
            host: host.to_string(),
        }
    }

    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// 请求键 `gpu-claim/card-lease-request/<这一台>`。
    #[must_use]
    pub fn request_key(&self) -> Vec<u8> {
        self.key_under(REQUEST_KEY_SEGMENT)
    }

    /// 卡键 `gpu-claim/card-lease/<这一台>`。
    #[must_use]
    pub fn card_key(&self) -> Vec<u8> {
        self.key_under(CARD_KEY_SEGMENT)
    }

    /// 这一台的等号都以它起头：`gpu-claim/card-lease-waiting/<这一台>/`（末尾的 `/` 让 `127.0.0.1` 的前缀不罩住 `127.0.0.10` 的等号）。
    #[must_use]
    pub fn waiting_ticket_prefix(&self) -> Vec<u8> {
        [
            self.key_under(WAITING_TICKET_KEY_SEGMENT),
            WAITING_TICKET_HOST_SEPARATOR.as_bytes().to_vec(),
        ]
        .concat()
    }

    fn key_under(&self, segment: &[u8]) -> Vec<u8> {
        [GPU_CLAIM_FAMILY_PREFIX, segment, self.host.as_bytes()].concat()
    }
}

/// 停止键 `gpu-claim/card-lease-returned/<持有者身份>`。
#[must_use]
pub fn returned_key_of(holder: &[u8]) -> Vec<u8> {
    [GPU_CLAIM_FAMILY_PREFIX, RETURNED_KEY_SEGMENT, holder].concat()
}

/// `lease return` 认领停止键用的身份：持有者身份接 `/return`。
#[must_use]
pub fn returning_identity_of(holder: &[u8]) -> Vec<u8> {
    [holder, RETURNING_IDENTITY_SUFFIX].concat()
}

/// 借的进程：进程号与它的启动刻（`/proc/<进程号>/stat` 第 22 项，开机以来的时钟滴答）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardLeaseBorrower {
    pub process_number: u32,
    pub start_ticks: u64,
}

impl CardLeaseBorrower {
    /// 照变量的值认（`None` 是没给）：恰好 `<进程号>:<启动刻>`，两段都是十进制数、进程号不是 0。
    ///
    /// # Errors
    /// 没给、写法不对：交回为什么（带变量名与写法）。
    pub fn of_the_variable_value(value: Option<&str>) -> Result<Self, String> {
        let value =
            value.ok_or_else(|| format!("环境里没有 {BORROWER_VARIABLE}：{LEASE_USAGE}"))?;
        let malformed = || format!("{BORROWER_VARIABLE}「{value}」写法不对：{LEASE_USAGE}");
        let (process_number, start_ticks) = value.split_once(':').ok_or_else(malformed)?;
        let is_decimal =
            |text: &str| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
        if !is_decimal(process_number) || !is_decimal(start_ticks) {
            return Err(malformed());
        }
        let process_number: u32 = process_number.parse().map_err(|_| malformed())?;
        let start_ticks: u64 = start_ticks.parse().map_err(|_| malformed())?;
        if process_number == 0 {
            return Err(malformed());
        }
        Ok(Self {
            process_number,
            start_ticks,
        })
    }

    /// 此刻在跑的那个进程（它的 `/proc/<进程号>/stat` 照 `crate::service_process` 的读法读）。
    ///
    /// # Errors
    /// 那个进程不在、stat 读不了或认不出。
    pub fn of_the_running_process(process_number: u32) -> Result<Self, String> {
        let status_fields = process_status_fields_of(process_number)?.ok_or_else(|| {
            format!("/proc/{process_number}/stat 不在：进程 {process_number} 不在了")
        })?;
        Ok(Self {
            process_number,
            start_ticks: status_fields.start_ticks,
        })
    }

    /// 变量的写法 `<进程号>:<启动刻>`。
    #[must_use]
    pub fn variable_value(&self) -> String {
        format!("{}:{}", self.process_number, self.start_ticks)
    }

    /// 持有者身份 `card-lease/<这一台>/<进程号>-<启动刻>`。
    #[must_use]
    pub fn holder_identity(&self, keys: &CardLeaseKeys) -> Vec<u8> {
        format!(
            "{HOLDER_PREFIX}{}/{}-{}",
            keys.host(),
            self.process_number,
            self.start_ticks
        )
        .into_bytes()
    }

    /// 这个进程此刻还在不在：进程号在、不是僵尸、启动刻相同（同一次开机里认，`crate::service_process` 的判法）。
    ///
    /// # Errors
    /// 开机号或 `/proc/<进程号>/stat` 读不了、认不出。
    pub fn liveness(&self) -> Result<ProcessLiveness, String> {
        let this_process = ServiceProcessIdentity::of_this_process()?;
        ServiceProcessIdentity {
            process_number: self.process_number,
            start_ticks: self.start_ticks,
            boot: this_process.boot,
        }
        .liveness()
    }
}

/// 一张等号（文件头「先来后到」）：哪个借的进程、从哪一刻起等。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardLeaseWaitingTicket {
    pub started_waiting_milliseconds_since_the_epoch: u64,
    pub borrower: CardLeaseBorrower,
}

impl CardLeaseWaitingTicket {
    /// `borrower` 此刻开始等的那一张。
    ///
    /// # Errors
    /// 系统钟早于 UNIX 纪元，或毫秒数装不进 `u64`。
    pub fn starting_now(borrower: CardLeaseBorrower) -> Result<Self, String> {
        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("系统钟早于 UNIX 纪元，排不了等号：{error}"))?;
        let started_waiting_milliseconds_since_the_epoch =
            u64::try_from(since_the_epoch.as_millis())
                .map_err(|_| "系统钟的毫秒数装不进 u64，排不了等号".to_string())?;
        Ok(Self {
            started_waiting_milliseconds_since_the_epoch,
            borrower,
        })
    }

    /// 等号键 `gpu-claim/card-lease-waiting/<这一台>/<开始等的毫秒数>-<借的进程号>-<启动刻>`，三项各补零到定宽：
    /// 按键升序就是开始等的次序，同一毫秒的按借的进程号、启动刻分先后。
    #[must_use]
    pub fn key(&self, keys: &CardLeaseKeys) -> Vec<u8> {
        let sorting_segment = format!(
            "{started:0start_width$}{WAITING_TICKET_FIELD_SEPARATOR}{process:0process_width$}{WAITING_TICKET_FIELD_SEPARATOR}{ticks:0ticks_width$}",
            started = self.started_waiting_milliseconds_since_the_epoch,
            start_width = WAITING_START_DIGITS,
            process = self.borrower.process_number,
            process_width = WAITING_PROCESS_NUMBER_DIGITS,
            ticks = self.borrower.start_ticks,
            ticks_width = WAITING_START_TICKS_DIGITS,
        );
        [keys.waiting_ticket_prefix(), sorting_segment.into_bytes()].concat()
    }
}

/// 自己那一张等号在 `queue`（这一台的等号，按键升序列出来的）里排第几，从 [`FIRST_POSITION_IN_THE_QUEUE`] 起；不在里面是 `None`。
#[must_use]
pub fn position_in_the_queue_of(queue: &[ListedClaim], own_ticket_key: &[u8]) -> Option<usize> {
    queue
        .iter()
        .position(|ticket| ticket.key == own_ticket_key)
        .map(|index| FIRST_POSITION_IN_THE_QUEUE + index)
}

/// 借卡白名单（文件头「借卡白名单」）：借得到卡的进程的可执行文件名。不变量：每一项不空、不带空白与 `/`、互不相同。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardLeaseBorrowerWhitelist {
    executable_names: BTreeSet<String>,
}

impl CardLeaseBorrowerWhitelist {
    /// 照白名单的写法认 `text`：每一行 `<可执行文件名>  # <理由>`，`#` 起头的行与空行不算。
    ///
    /// # Errors
    /// 一行没有 `#` 后面的理由、名字带空白或 `/`、同一个名字写了两行：交回第几行、为什么。
    pub fn of_the_text(text: &str) -> Result<Self, String> {
        let mut executable_names = BTreeSet::new();
        for (line_index, line) in text.lines().enumerate() {
            let line_number = line_index + 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (written_name, reason) = line.split_once('#').ok_or_else(|| {
                format!(
                    "第 {line_number} 行「{line}」没有 `# <理由>`：写成 <可执行文件名>  # <理由>"
                )
            })?;
            let executable_name = written_name.trim();
            if reason.trim().is_empty() {
                return Err(format!("第 {line_number} 行「{line}」的理由是空的"));
            }
            // 名字不会是空的：这一行去掉首尾空白之后不空、又不以 `#` 起头，`#` 前面至少有一个不是空白的字
            if executable_name.contains(char::is_whitespace) || executable_name.contains('/') {
                return Err(format!(
                    "第 {line_number} 行的可执行文件名「{executable_name}」带空白或 `/`：只写文件名"
                ));
            }
            if !executable_names.insert(executable_name.to_string()) {
                return Err(format!(
                    "第 {line_number} 行的可执行文件名「{executable_name}」前面已写过"
                ));
            }
        }
        Ok(Self { executable_names })
    }

    /// 编进这个二进制的那一份（[`CARD_LEASE_BORROWERS_TEXT`]）。
    ///
    /// # Errors
    /// 同 [`Self::of_the_text`]。
    pub fn compiled_in() -> Result<Self, String> {
        Self::of_the_text(CARD_LEASE_BORROWERS_TEXT)
    }

    /// 白名单里的可执行文件名，按字典序。
    pub fn executable_names(&self) -> impl Iterator<Item = &str> {
        self.executable_names.iter().map(String::as_str)
    }

    /// 可执行文件名是 `executable_name` 的进程借不借得到：与表里一项逐字相同。
    #[must_use]
    pub fn lets_borrow(&self, executable_name: &str) -> bool {
        self.executable_names.contains(executable_name)
    }
}

/// 借的进程的可执行文件名：`/proc/<进程号>/exe` 指到的那个文件的文件名（文件头「借卡白名单」）。读之前、读之后各认一回借的进程。
///
/// # Errors
/// 判不了借的进程在不在、它不在了或启动刻对不上、读不了那个链接、链接没有文件名或文件名不是 UTF-8：交回为什么。
pub fn executable_name_of_the_borrower(borrower: &CardLeaseBorrower) -> Result<String, String> {
    let process_number = borrower.process_number;
    let confirm_the_borrower = |when: &str| match borrower.liveness() {
        Ok(ProcessLiveness::Running) => Ok(()),
        Ok(ProcessLiveness::Gone) => Err(format!(
            "{when}借的进程 {} 不在了（进程号不在、已结束或启动刻对不上）",
            borrower.variable_value()
        )),
        Err(problem) => Err(format!("{when}判不了借的进程在不在：{problem}")),
    };
    confirm_the_borrower("读可执行文件之前")?;
    let executable_link_path = format!("/proc/{process_number}/exe");
    let executable = std::fs::read_link(&executable_link_path)
        .map_err(|error| format!("读不了 {executable_link_path}：{error}"))?;
    let executable_name = executable
        .file_name()
        .ok_or_else(|| {
            format!(
                "{executable_link_path} 指到的「{}」没有文件名",
                executable.display()
            )
        })?
        .to_str()
        .ok_or_else(|| {
            format!(
                "{executable_link_path} 指到的「{}」的文件名不是 UTF-8",
                executable.display()
            )
        })?
        .to_string();
    confirm_the_borrower("读完可执行文件之后")?;
    Ok(executable_name)
}

/// 认领一把借卡的键的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardLeaseKeyAnswer {
    /// 这个持有者持着（新认领到，或自己持着、续上了）。
    Claimed,
    /// 别的持有者持着、租期没到。
    HeldByOther {
        holder: Vec<u8>,
        remaining_lease_milliseconds: u64,
    },
}

/// 以 `holder` 认领（或续上）一把借卡的键。
///
/// # Errors
/// 读写失败；回的结局条数不是 1；库里有这把键（认领键不当数据行写，出现就是别处写坏了，当协议错报）。
pub fn claim_a_card_lease_key_as(
    client: &mut ReliableClient,
    holder: &[u8],
    key: Vec<u8>,
    lease_milliseconds: u32,
) -> Result<CardLeaseKeyAnswer, ClientFailure> {
    let mut outcomes = client.claim_as(
        holder,
        &[(Table::Conclusions, key.clone())],
        lease_milliseconds,
    )?;
    if outcomes.len() != 1 {
        return Err(ClientFailure::Protocol(format!(
            "认领一把借卡的键回了 {} 个结局",
            outcomes.len()
        )));
    }
    match outcomes.remove(0) {
        ClaimOutcome::Claimed => Ok(CardLeaseKeyAnswer::Claimed),
        ClaimOutcome::HeldByOther {
            holder_identity,
            remaining_lease_milliseconds,
        } => Ok(CardLeaseKeyAnswer::HeldByOther {
            holder: holder_identity,
            remaining_lease_milliseconds,
        }),
        ClaimOutcome::AlreadyConcluded => Err(ClientFailure::Protocol(format!(
            "判定库里有借卡的键 {} 这一行（认领键不当数据行写，库被别处写坏了）",
            String::from_utf8_lossy(&key)
        ))),
    }
}

/// 以 `holder` 放掉这几把借卡的键（只放它名下的）；交回逐键的结局。
///
/// # Errors
/// 读写失败；回的结局个数与键数不同。
pub fn release_card_lease_keys_as(
    client: &mut ReliableClient,
    holder: &[u8],
    keys: &[Vec<u8>],
) -> Result<Vec<ClaimReleaseOutcome>, ClientFailure> {
    let keys_in_the_table: Vec<(Table, Vec<u8>)> = keys
        .iter()
        .map(|key| (Table::Conclusions, key.clone()))
        .collect();
    let outcomes = client.release_claims_as(holder, &keys_in_the_table)?;
    if outcomes.len() != keys.len() {
        return Err(ClientFailure::Protocol(format!(
            "放 {} 把借卡的键回了 {} 个结局",
            keys.len(),
            outcomes.len()
        )));
    }
    Ok(outcomes)
}

/// 卡串里地址是 `this_host` 的那几项，照原样、按原来的先后（每项从右切两刀取地址，地址可以是 IPv6、自带冒号）；切不出三段的项不算。
#[must_use]
pub fn entries_of_this_host<'card_string>(
    card_string: &'card_string str,
    this_host: &str,
) -> Vec<&'card_string str> {
    card_string
        .split_whitespace()
        .filter(|entry| {
            entry
                .rsplit_once(':')
                .and_then(|(address_and_index, _quota)| address_and_index.rsplit_once(':'))
                .is_some_and(|(address, _index)| address == this_host)
        })
        .collect()
}

/// 服务放出卡键那一行。
#[must_use]
pub fn cards_lent_line(keys: &CardLeaseKeys, blocks_finished_first: u64) -> String {
    format!(
        "{CARDS_LENT_LINE_PREFIX} host={} blocks_finished_first={blocks_finished_first}",
        keys.host()
    )
}

/// 服务收回卡键那一行。
#[must_use]
pub fn cards_taken_back_line(keys: &CardLeaseKeys, seconds_lent: u64) -> String {
    format!(
        "{CARDS_TAKEN_BACK_LINE_PREFIX} host={} seconds_lent={seconds_lent}",
        keys.host()
    )
}

/// 持有者身份打给人看：可见的 ASCII 照原样，别的打小写十六进制。
#[must_use]
pub fn holder_text(holder: &[u8]) -> String {
    if !holder.is_empty() && holder.iter().all(u8::is_ascii_graphic) {
        String::from_utf8_lossy(holder).into_owned()
    } else {
        hexadecimal_text(holder)
    }
}

/// 卡键此刻在谁手里（`lease list` 那一行的 `held_by=`，文件头「查借着哪几张卡」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardKeyHolding {
    /// 持有者身份不以 `card-lease/` 起头：服务以它认领待判块的身份持着。
    Service,
    /// 持有者身份以 `card-lease/` 起头：借的一方持着。
    Borrower,
    /// 列不到卡键。
    Nobody,
}

impl CardKeyHolding {
    fn of_the_card_key(card_key_claim: Option<&ListedClaim>) -> Self {
        match card_key_claim {
            None => Self::Nobody,
            Some(claim) if claim.holder_identity.starts_with(HOLDER_PREFIX.as_bytes()) => {
                Self::Borrower
            }
            Some(_) => Self::Service,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Borrower => "borrower",
            Self::Nobody => "nobody",
        }
    }
}

/// `lease list` 的卡键那一行（文件头「查借着哪几张卡」）：`cards` 是这一台的卡串。
fn card_lease_held_line(
    keys: &CardLeaseKeys,
    card_key_claim: Option<&ListedClaim>,
    cards: &str,
) -> String {
    let holding = CardKeyHolding::of_the_card_key(card_key_claim);
    let (holder_written, lease_left_milliseconds) = card_key_claim.map_or_else(
        || (HOLDER_NOT_HELD.to_string(), 0),
        |claim| {
            (
                format!("{:?}", holder_text(&claim.holder_identity)),
                claim.remaining_lease_milliseconds,
            )
        },
    );
    format!(
        "{CARD_LEASE_HELD_LINE_PREFIX} host={} held_by={} holder={holder_written} cards={cards:?} lease_left_milliseconds={lease_left_milliseconds}",
        keys.host(),
        holding.name()
    )
}

/// `lease list` 的请求键那一行（请求键有人持着时才打）。
fn card_lease_requested_line(keys: &CardLeaseKeys, request_key_claim: &ListedClaim) -> String {
    format!(
        "{CARD_LEASE_REQUESTED_LINE_PREFIX} host={} holder={:?} lease_left_milliseconds={}",
        keys.host(),
        holder_text(&request_key_claim.holder_identity),
        request_key_claim.remaining_lease_milliseconds
    )
}

/// `lease list` 的一张等号那一行：`position` 是它排第几（从 [`FIRST_POSITION_IN_THE_QUEUE`] 起）。
fn card_lease_queued_line(keys: &CardLeaseKeys, position: usize, ticket: &ListedClaim) -> String {
    format!(
        "{CARD_LEASE_QUEUED_LINE_PREFIX} host={} position={position} holder={:?} lease_left_milliseconds={}",
        keys.host(),
        holder_text(&ticket.holder_identity),
        ticket.remaining_lease_milliseconds
    )
}

/// 等的时候那一行打到哪一回了（文件头 `lease take`）：头一回、在等的那一把或自己排第几变了的那一回与之后每 [`WAITING_LINE_PERIOD`] 打一回。
struct WaitingLinePrinting {
    /// 上一回打的：在等的那一把、自己排第几、什么时候打的。
    last_printed: Option<(&'static str, usize, Instant)>,
}

impl WaitingLinePrinting {
    fn not_printed_yet() -> Self {
        Self { last_printed: None }
    }

    /// 在等 `key_said`（`holder` 持着、剩 `remaining_lease_milliseconds`），自己排第 `position_in_the_queue`：到了该打的那一回就往标准错误打。
    fn print_when_due(
        &mut self,
        keys: &CardLeaseKeys,
        key_said: &'static str,
        holder: &[u8],
        remaining_lease_milliseconds: u64,
        position_in_the_queue: usize,
    ) {
        let is_due = self
            .last_printed
            .is_none_or(|(printed_key, printed_position, printed_at)| {
                printed_key != key_said
                    || printed_position != position_in_the_queue
                    || printed_at.elapsed() >= WAITING_LINE_PERIOD
            });
        if is_due {
            eprintln!(
                "{CARD_LEASE_WAITING_LINE_PREFIX} key={key_said} host={} holder={} remaining_lease_milliseconds={remaining_lease_milliseconds} position={position_in_the_queue}",
                keys.host(),
                holder_text(holder)
            );
            self.last_printed = Some((key_said, position_in_the_queue, Instant::now()));
        }
    }
}

/// 一条子命令没做成：打两行（`✗` 与出路），退 `exit`；照借卡白名单拒的，两行之前另打一行 [`CARD_LEASE_REFUSED_LINE_PREFIX`]。
struct LeaseRefusal {
    refused_line: Option<String>,
    problem: String,
    next_step: String,
    exit: u8,
}

impl LeaseRefusal {
    fn new(problem: String, next_step: &str, exit: u8) -> Self {
        Self {
            refused_line: None,
            problem,
            next_step: next_step.to_string(),
            exit,
        }
    }

    fn of_the_library(failure: &ClientFailure, what: &str) -> Self {
        Self {
            refused_line: None,
            problem: format!("{what}时读写判定库失败：{failure}"),
            next_step:
                "prune-server/control check 看判定库服务在不在跑；借着的键到租期过了自己失效"
                    .to_string(),
            exit: u8::try_from(failure.exit_code()).unwrap_or(LEASE_FAILURE_EXIT),
        }
    }

    /// 借的一方不在借卡白名单里（`executable_name` 是读到的可执行文件名，认不出是 `None`）：文件头「借卡白名单」那三行、退
    /// [`BORROWER_OFF_THE_WHITELIST_EXIT`]。
    fn of_a_borrower_off_the_whitelist(
        borrower: &CardLeaseBorrower,
        executable_name: Option<&str>,
        problem: String,
    ) -> Self {
        let executable_written = executable_name.map_or_else(
            || EXECUTABLE_NOT_READ.to_string(),
            |name| format!("{name:?}"),
        );
        Self {
            refused_line: Some(format!(
                "{CARD_LEASE_REFUSED_LINE_PREFIX} borrower_process={} executable={executable_written}",
                borrower.process_number
            )),
            problem,
            next_step: format!(
                "卡只归 GPU 服务：崩溃判段写待判行由服务判，别的测试不借卡；确要借卡的真卡验证，经用户同意往 \
{CARD_LEASE_BORROWERS_PATH_IN_THE_REPOSITORY} 加一行后重编、`control install` 重装服务"
            ),
            exit: BORROWER_OFF_THE_WHITELIST_EXIT,
        }
    }

    fn print(&self) -> ExitCode {
        if let Some(refused_line) = &self.refused_line {
            eprintln!("{refused_line}");
        }
        eprintln!("  ✗ {}", self.problem);
        eprintln!("  → 怎么办：{}", self.next_step);
        ExitCode::from(self.exit)
    }
}

const CONFIGURATION_HOWTO: &str = "照仓根 multi-host.env.example 写多机配置（HOSTS 写这一台的地址，GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 或 GPU_CARDS_BY_PRIORITY 点名这一台的卡），\
或设 SINGLEFS_MULTI_HOST_CONFIG 指到它";

/// 这几条子命令的弄坏开关：只在临时判定库服务底下认（文件头「弄坏开关」）。
fn break_switches_of_the_lease_subcommand() -> Result<BreakSwitches, LeaseRefusal> {
    let value = std::env::var(BREAK_VARIABLE)
        .ok()
        .filter(|value| !value.trim().is_empty());
    let Some(value) = value else {
        return Ok(BreakSwitches::none());
    };
    let under_the_temporary_service =
        std::env::var_os(PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE)
            .is_some_and(|marked| !marked.is_empty());
    if !under_the_temporary_service {
        return Err(LeaseRefusal::new(
            format!("{BREAK_VARIABLE} 只在临时判定库服务底下认（{PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE} 没设）"),
            &format!("不设 {BREAK_VARIABLE} 再借"),
            USAGE_EXIT,
        ));
    }
    BreakSwitches::from_the_variable_value(Some(&value)).map_err(|problem| {
        LeaseRefusal::new(problem, "照 src/break_switches.rs 的项名改", USAGE_EXIT)
    })
}

/// 借的一方：照变量认；弄坏开关 card-lease-borrower-from-the-parent-process 下变量没给时取这个进程的父进程。
fn borrower_of_the_environment(
    break_switches: &BreakSwitches,
) -> Result<CardLeaseBorrower, LeaseRefusal> {
    let value = std::env::var(BORROWER_VARIABLE).ok();
    if value.is_none() && break_switches.is_on(BreakSwitch::CardLeaseBorrowerFromTheParentProcess) {
        return CardLeaseBorrower::of_the_running_process(std::os::unix::process::parent_id())
            .map_err(|problem| LeaseRefusal::new(problem, LEASE_USAGE, USAGE_EXIT));
    }
    CardLeaseBorrower::of_the_variable_value(value.as_deref()).map_err(|problem| {
        LeaseRefusal::new(
            problem,
            "借的进程把自己的进程号与启动刻（/proc/self/stat 第 22 项）写成 GPU_CARD_LEASE_BORROWER=<进程号>:<启动刻> 交给每一条子命令",
            USAGE_EXIT,
        )
    })
}

/// `lease take` 认借的一方（文件头「借卡白名单」）：可执行文件名逐字在借卡白名单里才放行，在任何一把键认领之前。
/// 弄坏开关 card-lease-borrower-whitelist-unchecked 下不认，谁都放行。
fn admit_only_a_borrower_on_the_whitelist(
    borrower: &CardLeaseBorrower,
    break_switches: &BreakSwitches,
) -> Result<(), LeaseRefusal> {
    if break_switches.is_on(BreakSwitch::CardLeaseBorrowerWhitelistUnchecked) {
        return Ok(());
    }
    let whitelist = CardLeaseBorrowerWhitelist::compiled_in().map_err(|problem| {
        LeaseRefusal::new(
            format!("编进来的 {CARD_LEASE_BORROWERS_PATH_IN_THE_REPOSITORY} 解不开：{problem}"),
            "照那份文件头的写法改对，重编、重装",
            LEASE_FAILURE_EXIT,
        )
    })?;
    let listed = whitelist
        .executable_names()
        .collect::<Vec<&str>>()
        .join("、");
    let process_number = borrower.process_number;
    match executable_name_of_the_borrower(borrower) {
        Ok(executable_name) => {
            if whitelist.lets_borrow(&executable_name) {
                return Ok(());
            }
            Err(LeaseRefusal::of_a_borrower_off_the_whitelist(
                borrower,
                Some(&executable_name),
                format!(
                    "借的进程 {process_number} 的可执行文件名「{executable_name}」不在借卡白名单 \
{CARD_LEASE_BORROWERS_PATH_IN_THE_REPOSITORY} 里（表里的：{listed}）：不借"
                ),
            ))
        }
        Err(problem) => Err(LeaseRefusal::of_a_borrower_off_the_whitelist(
            borrower,
            None,
            format!(
                "认不出借的进程 {process_number} 是哪个可执行文件（{problem}）；借卡白名单 \
{CARD_LEASE_BORROWERS_PATH_IN_THE_REPOSITORY} 只认表里的文件名（表里的：{listed}）：不借"
            ),
        )),
    }
}

/// 多机配置在哪、这一台是主机数组里哪一项（文件头）。
fn configuration_and_this_host(
    repository: &Path,
) -> Result<(std::path::PathBuf, MultiHostAddresses), LeaseRefusal> {
    let configuration = multi_host_configuration_file(repository)
        .map_err(|problem| LeaseRefusal::new(problem, CONFIGURATION_HOWTO, RUN_CONDITION_EXIT))?
        .ok_or_else(|| {
            LeaseRefusal::new(
                format!(
                    "找不到多机配置（SINGLEFS_MULTI_HOST_CONFIG 没设，编这份二进制的仓 {} 的主工作树根下也没有）：借哪几张卡只照它",
                    repository.display()
                ),
                CONFIGURATION_HOWTO,
                RUN_CONDITION_EXIT,
            )
        })?;
    let addresses = MultiHostAddresses::emitted_by_the_configuration_check(&configuration)
        .map_err(|problem| LeaseRefusal::new(problem, CONFIGURATION_HOWTO, RUN_CONDITION_EXIT))?;
    Ok((configuration, addresses))
}

/// 多机配置里那一份显卡先后（文件头「卡串」：清场之后那一份，没写取平常那一份），照判法解过。
fn configured_card_string(
    configuration: &Path,
    addresses: &MultiHostAddresses,
) -> Result<String, LeaseRefusal> {
    let written = |key: &str| {
        configuration_value(configuration, key).filter(|value| !value.trim().is_empty())
    };
    let card_string = written(CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY)
        .or_else(|| written(CARDS_BY_PRIORITY_KEY))
        .ok_or_else(|| {
            LeaseRefusal::new(
                format!(
                    "多机配置 {} 里 {CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY} 与 {CARDS_BY_PRIORITY_KEY} 都没写：这一台没有卡可借",
                    configuration.display()
                ),
                CONFIGURATION_HOWTO,
                RUN_CONDITION_EXIT,
            )
        })?;
    parse_cards_by_priority(&card_string, addresses)
        .map_err(|problem| LeaseRefusal::new(problem, CONFIGURATION_HOWTO, RUN_CONDITION_EXIT))?;
    Ok(card_string)
}

/// 这一台的卡串（文件头「卡串」）；弄坏开关 card-lease-taken-without-cards-of-this-host 下一项都没有也交（空串）。
fn card_string_of_this_host(
    configuration: &Path,
    addresses: &MultiHostAddresses,
    break_switches: &BreakSwitches,
) -> Result<String, LeaseRefusal> {
    let card_string = configured_card_string(configuration, addresses)?;
    let entries = entries_of_this_host(&card_string, addresses.this_host().as_str());
    if entries.is_empty()
        && !break_switches.is_on(BreakSwitch::CardLeaseTakenWithoutCardsOfThisHost)
    {
        return Err(LeaseRefusal::new(
            format!(
                "多机配置的卡串「{card_string}」里没有这一台（{}）的卡：不借",
                addresses.this_host().as_str()
            ),
            "在没有卡的这一台上不借卡：要卡的那一条照常交统一任务服务（它挑机器），或在多机配置里点名这一台的卡",
            RUN_CONDITION_EXIT,
        ));
    }
    Ok(entries.join(" "))
}

/// 连上判定库的客户端、它的追加日志所在的一次性临时目录与认领租期。字段次序：客户端在临时目录之前丢掉（追加日志先关、目录再删）。
struct LibraryConnection {
    client: ReliableClient,
    journal_directory: ExclusiveTemporaryDirectory,
    lease_milliseconds: u32,
}

impl LibraryConnection {
    fn open() -> Result<Self, LeaseRefusal> {
        let configuration = ServiceConfiguration::from_the_text(SERVICE_CONFIGURATION_TEXT)
            .map_err(|problem| {
                LeaseRefusal::new(
                    format!("编进来的 gpu-verification-server/service-configuration.txt 解不开：{problem}"),
                    "照那份文件头的写法改对，重编、重装",
                    LEASE_FAILURE_EXIT,
                )
            })?;
        let route = prune_server_route_of_the_configuration().map_err(|refusal| {
            LeaseRefusal::new(
                refusal.failure_line(),
                "照上面那一行的出路改环境或多机配置",
                LIBRARY_ROUTE_REFUSED_EXIT,
            )
        })?;
        let journal_directory = ExclusiveTemporaryDirectory::create(TEMPORARY_DIRECTORY_LABEL)
            .map_err(|problem| {
                LeaseRefusal::new(
                    problem,
                    "看临时目录（TMPDIR）在不在、写不写得进",
                    LEASE_FAILURE_EXIT,
                )
            })?;
        let client = ReliableClient::open(ReliableClientSettings::new(
            route,
            journal_directory.join(JOURNAL_FILE_NAME),
        ))
        .map_err(|failure| LeaseRefusal::of_the_library(&failure, "连判定库"))?;
        Ok(Self {
            client,
            journal_directory,
            lease_milliseconds: configuration.claim_lease_milliseconds(),
        })
    }

    /// `prefix` 下此刻租期没到的认领，按键升序（列认领帧）。
    fn claims_listed_under(&mut self, prefix: &[u8]) -> Result<Vec<ListedClaim>, LeaseRefusal> {
        self.client
            .list_claims(prefix)
            .map_err(|failure| LeaseRefusal::of_the_library(&failure, "列借卡的认领"))
    }

    /// 键逐字是 `key` 的那一把此刻的认领（列认领帧按前缀列，同一前缀下别的键不算）；没人持着是 `None`。
    fn claim_of_exactly(&mut self, key: &[u8]) -> Result<Option<ListedClaim>, LeaseRefusal> {
        Ok(self
            .claims_listed_under(key)?
            .into_iter()
            .find(|claim| claim.key == key))
    }
}

/// 一条子命令连上判定库之后的样子。字段次序：客户端在临时目录之前丢掉（追加日志先关、目录再删）。
struct LeaseSession {
    keys: CardLeaseKeys,
    borrower: CardLeaseBorrower,
    holder: Vec<u8>,
    lease_milliseconds: u32,
    break_switches: BreakSwitches,
    /// `lease take` 认领到、还没放的等号（文件头「先来后到」）；没有是 `None`。
    waiting_ticket: Option<Vec<u8>>,
    client: ReliableClient,
    _journal_directory: ExclusiveTemporaryDirectory,
}

impl LeaseSession {
    fn open(
        keys: CardLeaseKeys,
        borrower: CardLeaseBorrower,
        break_switches: BreakSwitches,
    ) -> Result<Self, LeaseRefusal> {
        let LibraryConnection {
            client,
            journal_directory,
            lease_milliseconds,
        } = LibraryConnection::open()?;
        let holder = borrower.holder_identity(&keys);
        Ok(Self {
            keys,
            borrower,
            holder,
            lease_milliseconds,
            break_switches,
            waiting_ticket: None,
            client,
            _journal_directory: journal_directory,
        })
    }

    fn renewal_interval(&self) -> Duration {
        Duration::from_millis(u64::from(self.lease_milliseconds / RENEWALS_PER_LEASE))
    }

    /// 等号的租期：取认领租期（文件头「先来后到」）。
    fn waiting_ticket_lease_milliseconds(&self) -> u32 {
        self.lease_milliseconds
    }

    fn claim_as_the_borrower(
        &mut self,
        key: Vec<u8>,
        lease_milliseconds: u32,
    ) -> Result<CardLeaseKeyAnswer, LeaseRefusal> {
        claim_a_card_lease_key_as(&mut self.client, &self.holder, key, lease_milliseconds)
            .map_err(|failure| LeaseRefusal::of_the_library(&failure, "认领借卡的键"))
    }

    /// 以借的一方的身份放掉两把键，还没放的等号一起放（文件头「先来后到」）。
    fn release_the_lease_keys(&mut self) -> Result<(), LeaseRefusal> {
        let mut keys = vec![self.keys.request_key(), self.keys.card_key()];
        keys.extend(self.waiting_ticket.clone());
        release_card_lease_keys_as(&mut self.client, &self.holder, &keys)
            .map_err(|failure| LeaseRefusal::of_the_library(&failure, "放借卡的键"))?;
        self.waiting_ticket = None;
        Ok(())
    }

    /// 请求键与卡键都认领到之后放掉自己的等号（文件头「先来后到」）：后面排着的那一张随即排到最前。
    fn release_the_waiting_ticket(&mut self) -> Result<(), LeaseRefusal> {
        let Some(ticket) = self.waiting_ticket.clone() else {
            return Ok(());
        };
        release_card_lease_keys_as(&mut self.client, &self.holder, &[ticket])
            .map_err(|failure| LeaseRefusal::of_the_library(&failure, "放等号"))?;
        self.waiting_ticket = None;
        Ok(())
    }

    /// 这一台此刻的等号，按键升序（文件头「先来后到」）。
    fn waiting_queue(&mut self) -> Result<Vec<ListedClaim>, LeaseRefusal> {
        self.client
            .list_claims(&self.keys.waiting_ticket_prefix())
            .map_err(|failure| LeaseRefusal::of_the_library(&failure, "列这一台的等号"))
    }

    /// 认领一张此刻开始等的等号（文件头「先来后到」）；交回它的键。
    fn take_a_waiting_ticket(&mut self) -> Result<Vec<u8>, LeaseRefusal> {
        let ticket = CardLeaseWaitingTicket::starting_now(self.borrower)
            .map_err(|problem| LeaseRefusal::new(problem, "核这一台的系统钟", LEASE_FAILURE_EXIT))?
            .key(&self.keys);
        match self
            .claim_as_the_borrower(ticket.clone(), self.waiting_ticket_lease_milliseconds())?
        {
            CardLeaseKeyAnswer::Claimed => {
                self.waiting_ticket = Some(ticket.clone());
                Ok(ticket)
            }
            CardLeaseKeyAnswer::HeldByOther { holder, .. } => Err(LeaseRefusal::new(
                format!(
                    "等号 {} 被 {} 持着：键里带着这个借的一方的进程号与启动刻，只有它自己认领",
                    String::from_utf8_lossy(&ticket),
                    holder_text(&holder)
                ),
                "prune-server/control check 看判定库服务；隔一毫秒以上重借一次",
                LEASE_FAILURE_EXIT,
            )),
        }
    }

    /// 等的时候借的进程不在了：放掉认领到的、交回拒绝。
    fn refusal_once_the_borrower_is_gone(&mut self, key_said: &str) -> LeaseRefusal {
        let refusal = LeaseRefusal::new(
            format!(
                "等{key_said}的时候借的进程 {} 不在了：放掉认领到的、不借",
                self.borrower.variable_value()
            ),
            "借的进程等到 lease take 退出再退出",
            LEASE_FAILURE_EXIT,
        );
        self.release_the_lease_keys().err().unwrap_or(refusal)
    }

    /// 续 `renewed` 那几把（键与各自的租期）：有一把没续上就放掉认领到的、交回拒绝。
    fn renew_while_waiting(
        &mut self,
        key_said: &str,
        renewed: &[(Vec<u8>, u32)],
    ) -> Result<(), LeaseRefusal> {
        for (renewed_key, lease_milliseconds) in renewed {
            if self.claim_as_the_borrower(renewed_key.clone(), *lease_milliseconds)?
                != CardLeaseKeyAnswer::Claimed
            {
                self.release_the_lease_keys()?;
                return Err(LeaseRefusal::new(
                    format!(
                        "等{key_said}的时候续 {} 没续上（租期过了、被别人认领走了）",
                        String::from_utf8_lossy(renewed_key)
                    ),
                    "重借一次",
                    LEASE_FAILURE_EXIT,
                ));
            }
        }
        Ok(())
    }

    /// 排到最前、认领到请求键（文件头「先来后到」）：每一回去认领请求键之前先列这一台的等号，自己那一张排在最前才去认领；
    /// 交回认领到那一回自己排第几。弄坏开关 card-lease-waiting-order-ignored 下不看次序、每一回都去认领；
    /// card-lease-keys-held-by-another-taken-as-claimed 下别人持着请求键也当认领到。
    fn wait_for_the_turn_and_the_request_key(
        &mut self,
        ticket: &[u8],
    ) -> Result<usize, LeaseRefusal> {
        let request_key = self.keys.request_key();
        let renewed_meanwhile = [(ticket.to_vec(), self.waiting_ticket_lease_milliseconds())];
        let mut waiting_line = WaitingLinePrinting::not_printed_yet();
        let mut renewed_at = Instant::now();
        // 迭代上界：不设，一直等到排到最前、认领到请求键；借的进程不在了、等号不在了、续不上、读写判定库失败时交回
        loop {
            let queue = self.waiting_queue()?;
            let Some(position_in_the_queue) = position_in_the_queue_of(&queue, ticket) else {
                self.release_the_lease_keys()?;
                return Err(LeaseRefusal::new(
                    format!(
                        "这一台的等号里没有自己那一张 {}（租期过了没续上）",
                        String::from_utf8_lossy(ticket)
                    ),
                    "重借一次",
                    LEASE_FAILURE_EXIT,
                ));
            };
            let is_at_the_front_of_the_queue = position_in_the_queue == FIRST_POSITION_IN_THE_QUEUE
                || self
                    .break_switches
                    .is_on(BreakSwitch::CardLeaseWaitingOrderIgnored);
            let (key_said, holder, remaining_lease_milliseconds) = if is_at_the_front_of_the_queue {
                match self.claim_as_the_borrower(request_key.clone(), self.lease_milliseconds)? {
                    CardLeaseKeyAnswer::Claimed => return Ok(position_in_the_queue),
                    CardLeaseKeyAnswer::HeldByOther { .. }
                        if self
                            .break_switches
                            .is_on(BreakSwitch::CardLeaseKeysHeldByAnotherTakenAsClaimed) =>
                    {
                        return Ok(position_in_the_queue)
                    }
                    CardLeaseKeyAnswer::HeldByOther {
                        holder,
                        remaining_lease_milliseconds,
                    } => ("请求键", holder, remaining_lease_milliseconds),
                }
            } else {
                let front_of_the_queue = queue
                    .first()
                    .expect("自己那一张在列出来的等号里，列出来的不空");
                (
                    "等号",
                    front_of_the_queue.holder_identity.clone(),
                    front_of_the_queue.remaining_lease_milliseconds,
                )
            };
            if !self.borrower_is_running() {
                return Err(self.refusal_once_the_borrower_is_gone(key_said));
            }
            waiting_line.print_when_due(
                &self.keys,
                key_said,
                &holder,
                remaining_lease_milliseconds,
                position_in_the_queue,
            );
            if renewed_at.elapsed() >= self.renewal_interval() {
                self.renew_while_waiting(key_said, &renewed_meanwhile)?;
                renewed_at = Instant::now();
            }
            std::thread::sleep(BORROWER_RETRY);
        }
    }

    /// 借的进程还在不在（只有明确认出进程已不在才当不在；短暂读不到时不误放键）。
    fn borrower_is_running(&self) -> bool {
        !matches!(self.borrower.liveness(), Ok(ProcessLiveness::Gone))
    }

    /// 等到认领到 `key`（文件头 `lease take`）：`renewed_meanwhile` 是已认领到、等的时候要续着的那几把（键与各自的租期），
    /// `position_in_the_queue` 是排到这一步时自己排第几（等的那一行照打）。
    /// 弄坏开关 card-lease-keys-held-by-another-taken-as-claimed 下别人持着也当认领到。
    fn wait_until_claimed(
        &mut self,
        key: &[u8],
        key_said: &'static str,
        renewed_meanwhile: &[(Vec<u8>, u32)],
        position_in_the_queue: usize,
    ) -> Result<(), LeaseRefusal> {
        let mut waiting_line = WaitingLinePrinting::not_printed_yet();
        let mut renewed_at = Instant::now();
        // 迭代上界：不设，一直等到认领到；借的进程不在了、续不上、读写判定库失败时交回
        loop {
            let answer = self.claim_as_the_borrower(key.to_vec(), self.lease_milliseconds)?;
            let CardLeaseKeyAnswer::HeldByOther {
                holder,
                remaining_lease_milliseconds,
            } = answer
            else {
                return Ok(());
            };
            if self
                .break_switches
                .is_on(BreakSwitch::CardLeaseKeysHeldByAnotherTakenAsClaimed)
            {
                return Ok(());
            }
            if !self.borrower_is_running() {
                return Err(self.refusal_once_the_borrower_is_gone(key_said));
            }
            waiting_line.print_when_due(
                &self.keys,
                key_said,
                &holder,
                remaining_lease_milliseconds,
                position_in_the_queue,
            );
            if renewed_at.elapsed() >= self.renewal_interval() {
                self.renew_while_waiting(key_said, renewed_meanwhile)?;
                renewed_at = Instant::now();
            }
            std::thread::sleep(BORROWER_RETRY);
        }
    }

    /// 起续租进程、等它的就绪行；交回它的 `<进程号>:<启动刻>`（读不到启动刻的只交进程号）。
    fn start_the_renewer(&mut self) -> Result<String, LeaseRefusal> {
        let refused = |problem: String| {
            LeaseRefusal::new(
                problem,
                "看上面那一句；两把键已放掉，重借一次",
                LEASE_FAILURE_EXIT,
            )
        };
        let program = std::env::current_exe()
            .map_err(|error| refused(format!("取不到这个二进制的路径（续租进程照它起）：{error}")));
        let spawned = program.and_then(|program| {
            Command::new(program)
                .args([LEASE_SUBCOMMAND, RENEW_SUBCOMMAND])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .process_group(0)
                .spawn()
                .map_err(|error| refused(format!("起不了续租进程：{error}")))
        });
        let mut renewer = match spawned {
            Ok(renewer) => renewer,
            Err(refusal) => {
                self.release_the_lease_keys()?;
                return Err(refusal);
            }
        };
        let mut ready_line = String::new();
        let read = renewer
            .stdout
            .take()
            .map(|output| BufReader::new(output).read_line(&mut ready_line));
        if !matches!(read, Some(Ok(_))) || !ready_line.starts_with(CARD_LEASE_RENEWING_LINE_PREFIX)
        {
            let status = renewer.wait().map_or_else(
                |error| format!("等不到它：{error}"),
                |status| status.to_string(),
            );
            self.release_the_lease_keys()?;
            return Err(refused(format!(
                "续租进程没回就绪行（读到「{}」，它 {status}）",
                ready_line.trim_end()
            )));
        }
        Ok(identity_of_the_renewer_left_running(renewer))
    }

    /// `lease take` 排队、认领两把键（文件头「先来后到」与 `lease take`）：认领到之后放掉等号。
    fn take_both_keys_in_turn(&mut self) -> Result<(), LeaseRefusal> {
        let ticket = self.take_a_waiting_ticket()?;
        let position_in_the_queue = self.wait_for_the_turn_and_the_request_key(&ticket)?;
        if !self
            .break_switches
            .is_on(BreakSwitch::CardLeasePrintedBeforeTheCardKey)
        {
            let card_key = self.keys.card_key();
            let renewed_meanwhile = [
                (self.keys.request_key(), self.lease_milliseconds),
                (ticket, self.waiting_ticket_lease_milliseconds()),
            ];
            self.wait_until_claimed(&card_key, "卡键", &renewed_meanwhile, position_in_the_queue)?;
        }
        let returning = returning_identity_of(&self.holder);
        let stop_key = returned_key_of(&self.holder);
        release_card_lease_keys_as(&mut self.client, &returning, &[stop_key])
            .map_err(|failure| LeaseRefusal::of_the_library(&failure, "清上一次的停止键"))?;
        self.release_the_waiting_ticket()
    }

    /// `lease take`（文件头）。
    fn take(&mut self, card_string: &str) -> Result<(), LeaseRefusal> {
        if let Err(refusal) = self.take_both_keys_in_turn() {
            // 中途退出的每一支都放掉认领到的与等号（文件头「先来后到」）；读写判定库失败的那一支多半放不掉，照交回原来那一句，键到租期自己失效
            let _ = self.release_the_lease_keys();
            return Err(refusal);
        }
        let renewer = self.start_the_renewer()?;
        eprintln!(
            "{CARD_LEASE_TAKEN_LINE_PREFIX} host={} holder={} renewer={renewer}",
            self.keys.host(),
            holder_text(&self.holder)
        );
        let mut output = std::io::stdout().lock();
        writeln!(output, "{card_string}")
            .and_then(|()| output.flush())
            .map_err(|error| {
                LeaseRefusal::new(
                    format!("卡串写不进标准输出：{error}"),
                    "借的进程照读 lease take 的标准输出；键由续租进程在借的进程退出之后放掉",
                    LEASE_FAILURE_EXIT,
                )
            })
    }

    /// `lease renew`（文件头）：叫停、借的进程不在、续不上时放掉两把键、交回。
    fn renew_until_returned_or_the_borrower_is_gone(&mut self) -> Result<(), LeaseRefusal> {
        let mut output = std::io::stdout().lock();
        writeln!(
            output,
            "{CARD_LEASE_RENEWING_LINE_PREFIX} host={} holder={}",
            self.keys.host(),
            holder_text(&self.holder)
        )
        .and_then(|()| output.flush())
        .map_err(|error| {
            LeaseRefusal::new(
                format!("就绪行写不出去：{error}"),
                "续租进程只由 lease take 起",
                LEASE_FAILURE_EXIT,
            )
        })?;
        drop(output);
        let stop_key = returned_key_of(&self.holder);
        let mut renewed_at = Instant::now();
        // 迭代上界：不设，借的进程在、没被叫停就一直续
        loop {
            std::thread::sleep(RENEWER_BEAT);
            if !self
                .break_switches
                .is_on(BreakSwitch::CardLeaseRenewerIgnoresTheBorrower)
                && !self.borrower_is_running()
            {
                return self.release_the_lease_keys();
            }
            if matches!(
                self.claim_as_the_borrower(stop_key.clone(), PROBE_LEASE_MILLISECONDS),
                Ok(CardLeaseKeyAnswer::HeldByOther { .. })
            ) {
                return self.release_the_lease_keys();
            }
            if renewed_at.elapsed() >= self.renewal_interval() {
                let mut renewed_all = true;
                for key in [self.keys.request_key(), self.keys.card_key()] {
                    match self.claim_as_the_borrower(key, self.lease_milliseconds) {
                        Ok(CardLeaseKeyAnswer::Claimed) => {}
                        Ok(CardLeaseKeyAnswer::HeldByOther { .. }) => {
                            return self.release_the_lease_keys();
                        }
                        Err(_) => {
                            renewed_all = false;
                            break;
                        }
                    }
                }
                if renewed_all {
                    renewed_at = Instant::now();
                }
            }
        }
    }

    /// `lease return`（文件头）：`card_string` 只打进标准错误那一行、不校验。
    /// 弄坏开关 card-lease-return-leaves-the-renewer-running 下不认领停止键、不等续租进程。
    fn return_the_cards(&mut self, card_string: &str) -> Result<(), LeaseRefusal> {
        eprintln!(
            "{CARD_LEASE_RETURNING_LINE_PREFIX} host={} holder={} card_string={card_string:?}",
            self.keys.host(),
            holder_text(&self.holder)
        );
        let returning = returning_identity_of(&self.holder);
        let card_key = self.keys.card_key();
        let mut released_by_returning = vec![card_key.clone()];
        if !self
            .break_switches
            .is_on(BreakSwitch::CardLeaseReturnLeavesTheRenewerRunning)
        {
            let stop_key = returned_key_of(&self.holder);
            let stop_key_lease_milliseconds = (self.lease_milliseconds / RENEWALS_PER_LEASE)
                * STOP_KEY_LEASE_IN_RENEWAL_INTERVALS;
            let deadline = Instant::now() + RETURN_WAIT_FOR_THE_RENEWER;
            // 迭代上界：RETURN_WAIT_FOR_THE_RENEWER 到点就停
            loop {
                let stop_key_held = claim_a_card_lease_key_as(
                    &mut self.client,
                    &returning,
                    stop_key.clone(),
                    stop_key_lease_milliseconds,
                )
                .map_err(|failure| LeaseRefusal::of_the_library(&failure, "认领停止键"))?
                    == CardLeaseKeyAnswer::Claimed;
                let card_key_answer = claim_a_card_lease_key_as(
                    &mut self.client,
                    &returning,
                    card_key.clone(),
                    PROBE_LEASE_MILLISECONDS,
                )
                .map_err(|failure| LeaseRefusal::of_the_library(&failure, "看卡键"))?;
                let renewer_let_go = !matches!(
                    &card_key_answer,
                    CardLeaseKeyAnswer::HeldByOther { holder, .. } if *holder == self.holder
                );
                if stop_key_held && renewer_let_go {
                    // 续租进程放了、退出了：停止键也放掉（同一个借的进程再借时它不该还在）；没等到的留着，租期过了自己失效
                    released_by_returning.push(stop_key);
                    break;
                }
                if Instant::now() >= deadline {
                    break;
                }
                std::thread::sleep(RETURN_POLL);
            }
        }
        self.release_the_lease_keys()?;
        release_card_lease_keys_as(&mut self.client, &returning, &released_by_returning)
            .map(|_| ())
            .map_err(|failure| LeaseRefusal::of_the_library(&failure, "放试过的卡键与停止键"))
    }
}

/// 续租进程脱离出去照跑：这个进程不等它（借到之后当场退出），它退出时由收养孤儿进程的那一方收尸。交回它的 `<进程号>:<启动刻>`
/// （读不到启动刻的只交进程号）。
fn identity_of_the_renewer_left_running(renewer: Child) -> String {
    CardLeaseBorrower::of_the_running_process(renewer.id()).map_or_else(
        |_| renewer.id().to_string(),
        |identity| identity.variable_value(),
    )
}

/// `lease list`（文件头「查借着哪几张卡」）：多机配置主机数组里点名了卡的每一台，照主机数组的次序，卡键那一行、请求键有人持着时那一行、
/// 每一张等号一行；列完才一起打到标准输出（中途读写判定库失败的一行都不打）。
fn print_the_leases_of_the_hosts_with_cards(
    configuration: &Path,
    addresses: &MultiHostAddresses,
) -> Result<(), LeaseRefusal> {
    let card_string = configured_card_string(configuration, addresses)?;
    let mut connection = LibraryConnection::open()?;
    let mut lines: Vec<String> = Vec::new();
    for host in addresses.hosts() {
        let cards_of_the_host = entries_of_this_host(&card_string, host.as_str());
        if cards_of_the_host.is_empty() {
            continue;
        }
        let keys = CardLeaseKeys::of_this_host(host.as_str());
        let card_key_claim = connection.claim_of_exactly(&keys.card_key())?;
        lines.push(card_lease_held_line(
            &keys,
            card_key_claim.as_ref(),
            &cards_of_the_host.join(" "),
        ));
        if let Some(request_key_claim) = connection.claim_of_exactly(&keys.request_key())? {
            lines.push(card_lease_requested_line(&keys, &request_key_claim));
        }
        let queue = connection.claims_listed_under(&keys.waiting_ticket_prefix())?;
        for (index, ticket) in queue.iter().enumerate() {
            lines.push(card_lease_queued_line(
                &keys,
                FIRST_POSITION_IN_THE_QUEUE + index,
                ticket,
            ));
        }
    }
    let mut output = std::io::stdout().lock();
    lines
        .iter()
        .try_for_each(|line| writeln!(output, "{line}"))
        .and_then(|()| output.flush())
        .map_err(|error| {
            LeaseRefusal::new(
                format!("lease list 的行写不进标准输出：{error}"),
                "核标准输出接到了哪里",
                LEASE_FAILURE_EXIT,
            )
        })
}

/// 借的一方的那几条 `lease` 子命令要做的（都要 `GPU_CARD_LEASE_BORROWER`）。
enum BorrowerStep {
    Take,
    /// `lease return <卡串>`：卡串原样（可以是空串）。
    Return {
        card_string: String,
    },
    Renew,
}

/// 一条 `lease` 子命令要做的。
enum LeaseStep {
    OfTheBorrower(BorrowerStep),
    /// `lease list`：只读，不要借的一方（文件头「查借着哪几张卡」）。
    List,
}

/// 跑 `gpu-verification-server lease …`（`arguments` 是 `lease` 之后的那几个词；`repository` 是编这份二进制的那棵仓，多机配置从那里找）。
#[must_use]
pub fn run_the_lease_subcommand(arguments: &[String], repository: &Path) -> ExitCode {
    let step = match arguments {
        [subcommand] if subcommand == TAKE_SUBCOMMAND => {
            LeaseStep::OfTheBorrower(BorrowerStep::Take)
        }
        [subcommand] if subcommand == RENEW_SUBCOMMAND => {
            LeaseStep::OfTheBorrower(BorrowerStep::Renew)
        }
        [subcommand] if subcommand == LIST_SUBCOMMAND => LeaseStep::List,
        [subcommand, card_string] if subcommand == RETURN_SUBCOMMAND => {
            LeaseStep::OfTheBorrower(BorrowerStep::Return {
                card_string: card_string.clone(),
            })
        }
        _ => {
            return LeaseRefusal::new(
                format!("认不出 lease 后面的「{}」", arguments.join(" ")),
                &format!("写成 {LEASE_USAGE}"),
                USAGE_EXIT,
            )
            .print();
        }
    };
    let ran = break_switches_of_the_lease_subcommand().and_then(|break_switches| match &step {
        LeaseStep::OfTheBorrower(borrower_step) => {
            run_a_borrower_step(borrower_step, repository, break_switches)
        }
        LeaseStep::List => {
            let (configuration, addresses) = configuration_and_this_host(repository)?;
            print_the_leases_of_the_hosts_with_cards(&configuration, &addresses)
        }
    });
    match ran {
        Ok(()) => ExitCode::from(SUCCESS_EXIT),
        Err(refusal) => refusal.print(),
    }
}

fn run_a_borrower_step(
    step: &BorrowerStep,
    repository: &Path,
    break_switches: BreakSwitches,
) -> Result<(), LeaseRefusal> {
    if matches!(step, BorrowerStep::Return { card_string } if card_string.trim().is_empty())
        && break_switches.is_on(BreakSwitch::CardLeaseReturnRefusesAnEmptyCardString)
    {
        return Err(LeaseRefusal::new(
            "lease return 的卡串是空的".to_string(),
            &format!("写成 {LEASE_USAGE}"),
            USAGE_EXIT,
        ));
    }
    let borrower = borrower_of_the_environment(&break_switches)?;
    if matches!(step, BorrowerStep::Take) {
        admit_only_a_borrower_on_the_whitelist(&borrower, &break_switches)?;
    }
    let (configuration, addresses) = configuration_and_this_host(repository)?;
    let keys = CardLeaseKeys::of_this_host(addresses.this_host().as_str());
    match step {
        BorrowerStep::Take => {
            let card_string =
                card_string_of_this_host(&configuration, &addresses, &break_switches)?;
            LeaseSession::open(keys, borrower, break_switches)?.take(&card_string)
        }
        BorrowerStep::Renew => LeaseSession::open(keys, borrower, break_switches)?
            .renew_until_returned_or_the_borrower_is_gone(),
        BorrowerStep::Return { card_string } => {
            LeaseSession::open(keys, borrower, break_switches)?.return_the_cards(card_string)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 变量只认 `<进程号>:<启动刻>`：两段都是十进制数、进程号不是 0；别的写法与没给都拒，拒的那一句带变量名。
    #[test]
    fn the_borrower_variable_takes_only_a_process_number_and_a_start_tick_count() {
        assert_eq!(
            CardLeaseBorrower::of_the_variable_value(Some("4242:987654")),
            Ok(CardLeaseBorrower {
                process_number: 4242,
                start_ticks: 987_654,
            })
        );
        for written in [
            None,
            Some(""),
            Some("4242"),
            Some("4242:"),
            Some(":987654"),
            Some("0:987654"),
            Some("4242:987654:1"),
            Some("-1:987654"),
            Some("4242:+5"),
            Some(" 4242:987654"),
            Some("abc:def"),
            Some("99999999999:1"),
        ] {
            let Err(refusal) = CardLeaseBorrower::of_the_variable_value(written) else {
                panic!("{written:?} 该拒");
            };
            assert!(
                refusal.contains(BORROWER_VARIABLE),
                "拒的那一句带变量名：{refusal}"
            );
        }
    }

    /// 两把键、停止键与持有者身份的写法逐字照文件头；变量的写法认得回。
    #[test]
    fn the_keys_and_the_holder_identity_are_written_as_the_header_says() {
        let keys = CardLeaseKeys::of_this_host("192.0.2.1");
        assert_eq!(
            keys.request_key(),
            b"gpu-claim/card-lease-request/192.0.2.1"
        );
        assert_eq!(keys.card_key(), b"gpu-claim/card-lease/192.0.2.1");
        let borrower = CardLeaseBorrower {
            process_number: 12,
            start_ticks: 34,
        };
        let holder = borrower.holder_identity(&keys);
        assert_eq!(holder, b"card-lease/192.0.2.1/12-34");
        assert_eq!(
            returned_key_of(&holder),
            b"gpu-claim/card-lease-returned/card-lease/192.0.2.1/12-34"
        );
        assert_eq!(
            returning_identity_of(&holder),
            b"card-lease/192.0.2.1/12-34/return"
        );
        assert_eq!(
            CardLeaseBorrower::of_the_variable_value(Some(&borrower.variable_value())),
            Ok(borrower)
        );
    }

    /// 卡串里只取地址逐字是这一台的那几项，照原样、照先后；IPv6 地址自带冒号也切得对。
    #[test]
    fn only_the_entries_of_this_host_are_kept_verbatim_in_their_order() {
        let card_string = "192.0.2.1:0:8G 192.0.2.2:0:16G 192.0.2.1:2:512M 192.0.2.10:1:4G";
        assert_eq!(
            entries_of_this_host(card_string, "192.0.2.1"),
            ["192.0.2.1:0:8G", "192.0.2.1:2:512M"]
        );
        assert_eq!(
            entries_of_this_host("fd00::1:0:8G fd00::2:1:8G", "fd00::1"),
            ["fd00::1:0:8G"]
        );
        assert!(entries_of_this_host(card_string, "192.0.2.3").is_empty());
    }

    /// 这个进程自己认得出、算在跑；进程号相同而启动刻不同算不在。
    #[test]
    fn this_process_is_running_and_the_same_number_with_another_start_is_gone() {
        let this_process = CardLeaseBorrower::of_the_running_process(std::process::id())
            .expect("读得到这个进程的 stat");
        assert_eq!(this_process.liveness(), Ok(ProcessLiveness::Running));
        let reused_number = CardLeaseBorrower {
            process_number: this_process.process_number,
            start_ticks: this_process.start_ticks + 1,
        };
        assert_eq!(reused_number.liveness(), Ok(ProcessLiveness::Gone));
    }

    /// 编进来的借卡白名单解得开，恰好只有 E202（GPU与CPU两种模式GC对照组）的装置二进制一项：多一项、少一项都要先改这里（加一行要用户同意）。
    #[test]
    fn the_compiled_in_whitelist_names_only_the_e202_comparison_binary() {
        let whitelist = CardLeaseBorrowerWhitelist::compiled_in()
            .unwrap_or_else(|problem| panic!("编进来的借卡白名单解不开：{problem}"));
        assert_eq!(
            whitelist.executable_names().collect::<Vec<&str>>(),
            ["e202_gpu_cpu_gc_comparison"]
        );
    }

    /// 只有逐字相同的文件名借得到：带 cargo 测试二进制那种哈希后缀的、只是前缀的、大小写不同的、空的都借不到。
    #[test]
    fn only_an_executable_name_equal_to_an_entry_lets_borrow() {
        let whitelist =
            CardLeaseBorrowerWhitelist::of_the_text("# 说明\n\nlisted_binary  # 测试用的一项\n")
                .expect("写法对");
        assert!(whitelist.lets_borrow("listed_binary"));
        for written in [
            "listed_binary-0123456789abcdef",
            "listed_binar",
            "Listed_binary",
            "listed_binary ",
            "",
        ] {
            assert!(!whitelist.lets_borrow(written), "「{written}」不该借得到");
        }
    }

    /// 白名单的写法：注释行与空行不算（去掉首尾空白之后以 `#` 起头的也是注释）；没写理由、理由是空的、名字带空白或 `/`、同一个名字写两行都拒，
    /// 拒的那一句点名第几行。
    #[test]
    fn whitelist_lines_without_a_reason_or_with_a_malformed_or_repeated_name_are_refused() {
        assert_eq!(
            CardLeaseBorrowerWhitelist::of_the_text("  # 只有注释\n\n")
                .expect("只有注释与空行写法对")
                .executable_names()
                .count(),
            0
        );
        for (text, line_number) in [
            ("listed_binary\n", 1),
            ("# 说明\nlisted_binary  #  \n", 2),
            ("  # 一行理由\n", 0),
            ("listed binary  # 理由\n", 1),
            ("bin/listed_binary  # 理由\n", 1),
            ("listed_binary  # 理由\nlisted_binary  # 又一回\n", 2),
        ] {
            let outcome = CardLeaseBorrowerWhitelist::of_the_text(text);
            if line_number == 0 {
                assert!(
                    outcome.is_ok(),
                    "{text:?} 里以 # 起头的行是注释：{outcome:?}"
                );
                continue;
            }
            let Err(refusal) = outcome else {
                panic!("{text:?} 该拒");
            };
            assert!(
                refusal.starts_with(&format!("第 {line_number} 行")),
                "{text:?} 拒的那一句点名第 {line_number} 行：{refusal}"
            );
        }
    }

    /// 这个进程的可执行文件名照 `/proc/<进程号>/exe` 读得出、与 `current_exe` 的文件名相同；启动刻对不上的照拒、点名不在了。
    #[test]
    fn the_executable_name_of_a_running_borrower_is_read_and_a_reused_number_is_refused() {
        let this_process = CardLeaseBorrower::of_the_running_process(std::process::id())
            .expect("读得到这个进程的 stat");
        let current_executable = std::env::current_exe().expect("读得到这个进程的可执行文件");
        let expected_name = current_executable
            .file_name()
            .and_then(|name| name.to_str())
            .expect("测试二进制的文件名是 UTF-8");
        assert_eq!(
            executable_name_of_the_borrower(&this_process).as_deref(),
            Ok(expected_name)
        );
        let reused_number = CardLeaseBorrower {
            process_number: this_process.process_number,
            start_ticks: this_process.start_ticks + 1,
        };
        let Err(refusal) = executable_name_of_the_borrower(&reused_number) else {
            panic!("启动刻对不上该拒");
        };
        assert!(refusal.contains("不在了"), "拒的那一句说不在了：{refusal}");
    }

    /// 等号键逐字照文件头「先来后到」：这一台的前缀带末尾的 `/`，排序段三项各补零到定宽；按键升序就是开始等的次序
    /// （位数不同的毫秒数照数的大小排），同一毫秒的按借的进程号、再按启动刻排；别的台的等号不落在这一台的前缀下。
    #[test]
    fn waiting_ticket_keys_sort_by_the_start_of_the_wait_then_by_the_borrowing_process() {
        let keys = CardLeaseKeys::of_this_host("192.0.2.1");
        assert_eq!(
            keys.waiting_ticket_prefix(),
            b"gpu-claim/card-lease-waiting/192.0.2.1/"
        );
        let ticket = |started: u64, process_number: u32, start_ticks: u64| CardLeaseWaitingTicket {
            started_waiting_milliseconds_since_the_epoch: started,
            borrower: CardLeaseBorrower {
                process_number,
                start_ticks,
            },
        };
        assert_eq!(
            ticket(1_700_000_000_123, 4242, 987_654).key(&keys),
            b"gpu-claim/card-lease-waiting/192.0.2.1/00000001700000000123-0000004242-00000000000000987654"
        );
        let in_the_order_they_started_waiting = [
            ticket(999, 4_000_000_000, 7),
            ticket(1_000, 9, 9),
            ticket(1_000, 10, 1),
            ticket(1_000, 10, 2),
            ticket(1_000_000, 1, 1),
        ];
        let ticket_keys: Vec<Vec<u8>> = in_the_order_they_started_waiting
            .iter()
            .map(|waiting| waiting.key(&keys))
            .collect();
        let mut sorted_keys = ticket_keys.clone();
        sorted_keys.sort();
        assert_eq!(sorted_keys, ticket_keys, "按键升序就是开始等的次序");
        let ticket_of_a_longer_address =
            ticket(1, 1, 1).key(&CardLeaseKeys::of_this_host("192.0.2.10"));
        assert!(
            !ticket_of_a_longer_address.starts_with(&keys.waiting_ticket_prefix()),
            "192.0.2.10 的等号不落在 192.0.2.1 的前缀下"
        );
    }

    /// 自己那一张排第几：照列出来的次序从 1 起数；列出来的里没有自己那一张是 `None`。
    #[test]
    fn the_position_in_the_queue_counts_from_one_in_the_listed_order() {
        fn listed(key: &[u8]) -> ListedClaim {
            ListedClaim {
                key: key.to_vec(),
                holder_identity: b"card-lease/192.0.2.1/1-1".to_vec(),
                remaining_lease_milliseconds: 1,
            }
        }
        let queue = [listed(b"front"), listed(b"middle"), listed(b"back")];
        assert_eq!(position_in_the_queue_of(&queue, b"front"), Some(1));
        assert_eq!(position_in_the_queue_of(&queue, b"back"), Some(3));
        assert_eq!(position_in_the_queue_of(&queue, b"absent"), None);
        assert_eq!(position_in_the_queue_of(&[], b"front"), None);
    }
}
