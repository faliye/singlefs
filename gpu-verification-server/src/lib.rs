//! GPU 批处理验证服务：崩溃判定的后半段（卡上判）。服务管这一台的卡（起来照多机配置开、一直持着），与统一任务服务之间只经判定库连：
//! 判段把要判的块写成待判行，服务取走、判完在一帧里写回结论、删掉待判行。卡上判没有状态，服务只管一件事：按每张卡给服务的显存额度定
//! 每一波派多少、怎么分，不让卡 OOM。待判的块跨片、跨流放在同一个池里连续取，卡不等某一片录完，也不按片排队；池里的态够把每张空着的卡
//! 各填满一次派活（水位线）、或最老的待判等过配置的限时，就发一波。
//!
//! 模块：
//! - `stream_identity`：一条流的键（打包判表的摘要、三份内核源码的摘要）与调度要的两样（表占多少显存、一共几个态）；
//! - `card_memory_ledger`：一张卡上驻留的判器占多少显存（与验证包 `mebibytes_a_judge_needs` 同一个算法）、按额度收判器的账；
//!   越过额度的判器进不了账，账外造不出一张卡的计划；
//! - `batch_scheduling`：批次调度（纯函数）：给空着的卡、各卡上驻留的判器、池里没派出去的块与此刻，出这一波的计划或不发的理由；
//! - `pending_pool`：待判池的接口与内存实现（批次调度只看这一份）；
//! - `library_pending_pool`：待判池的判定库那一份：不按键挑、在库上现算判到了哪一步（判完而可复用的带守卫删待判行）、认领（认领到之后重读待判行）、
//!   放认领（上一个进程没了放它的、停的时候放自己的）、
//!   一帧带守卫写回结论与删待判行、强制现判写凭据行；验证包那一侧经接口 `VerificationPackageSide`；
//! - `facts_verifier_family`：核对待判行那一族（验证包表 `1F`）自己的几样：块的名字与认领键、上卡核的材料、这一族的行首；
//!   扫表、认领、写回走 `library_pending_pool`，上卡核走 `service_loop` 借显存、`card_worker` 建核对器；
//! - `packed_tables_family`：判表待判行那一族（验证包表 `20`、`21`）自己的几样：一张判表的名字与认领键、写回那一帧要的几样、设备属性、这一族的行首；
//!   扫表、认领、写回走 `library_pending_pool`，上卡走服务主循环的判器路（一张判表是一条流里的一整块）；
//! - `registered_libraries`：登记的临时判定库：判段在真服务里登记的临时服务（认领键 `gpu-claim/temporary-library/…`）、一份登记一份待判池、
//!   各份库按入池序号先来先判、登记的库读写失败只丢那一份；
//! - `verification_package_side`：验证包那一侧的真的那一份：待判行、核对待判行与判表待判行的读写、读回材料、现算判完、核过与结论字在不在、
//!   编结论帧、核对行那一帧与结论字那一帧都调验证包交出的函数；
//! - `cards_of_the_service`：服务持着的那一组卡的接口；
//! - `serving_from_the_library`：服务进程的取活循环（起来开卡、一直持着 → 判定库那一份待判池 → 服务主循环 → 写回），按块、按卡打汇总行；
//!   有人借卡时停认领、手上的块写回之后借出卡键，没人借了收回；
//! - `card_lease`：借卡：请求键与卡键（判定库服务的认领键）、借出与收回那两行，二进制的 `lease take`、`lease return`、`lease renew`；
//! - `card_worker`：每张卡一条工作线程：照计划换出、装上判器，按批判，结论按批交回服务主循环；
//! - `gpu_judges_on_a_card`：工作线程在真卡上用的判器缓存（按流的键缓存 `GpuCrashJudge`）、照多机配置开的那一组真卡，
//!   与服务主循环里一条流的材料（镜像判定那一族读回的、判表那一族读回的，两种都装 `GpuCrashJudge`）；
//! - `service_loop`：服务主循环：收客户端的登记与入池、按调度发波、收结论、整块判完交回客户端；客户端在同一份里（`ServiceClient`）；
//! - `service_configuration`：服务的配置（凑批的限时、扫库间隔、认领租期），原文在 `service-configuration.txt`；
//! - `service_process`：服务进程的身份与「在跑」那一行，下一个进程照它判上一个进程还在不在（没了才放它留下的认领）；
//! - `service_log`：服务进程的标准错误换成过滤管道，验证包逐次派活打的两种行只计数、不进日志；
//! - `break_switches`：弄坏开关 `GPU_VERIFICATION_SERVER_BREAK` 的表，只给测试证红用。

pub mod batch_scheduling;
pub mod break_switches;
pub mod card_lease;
pub mod card_memory_ledger;
pub mod card_worker;
pub mod cards_of_the_service;
pub mod facts_verifier_family;
pub mod gpu_judges_on_a_card;
pub mod library_pending_pool;
pub mod packed_tables_family;
pub mod pending_pool;
pub mod registered_libraries;
pub mod service_configuration;
pub mod service_log;
pub mod service_loop;
pub mod service_process;
pub mod serving_from_the_library;
pub mod stream_identity;
pub mod verification_package_side;
