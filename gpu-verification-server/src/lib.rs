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
//! - `library_pending_pool`：待判池的判定库那一份：版本对账、在库上现算判到了哪一步（判完而可复用的带守卫删待判行）、认领、
//!   一帧带守卫写回结论与删待判行、强制现判写凭据行；验证包那一侧经接口 `VerificationPackageSide`；
//! - `verification_package_side`：验证包那一侧的真的那一份：待判行的读写、读回材料、现算判完、编结论帧都调验证包交出的函数；
//! - `cards_of_the_service`：服务持着的那一组卡的接口；
//! - `serving_from_the_library`：服务进程的取活循环（起来开卡、一直持着 → 判定库那一份待判池 → 服务主循环 → 写回）；
//! - `card_worker`：每张卡一条工作线程：照计划换出、装上判器，按批判，结论按批交回服务主循环；
//! - `gpu_judges_on_a_card`：工作线程在真卡上用的判器缓存（按流的键缓存 `GpuCrashJudge`）与照多机配置开的那一组真卡；
//! - `service_loop`：服务主循环：收客户端的登记与入池、按调度发波、收结论、整块判完交回客户端；客户端在同一份里（`ServiceClient`）；
//! - `service_configuration`：服务的配置（凑批的限时、扫库间隔、认领租期），原文在 `service-configuration.txt`；
//! - `break_switches`：弄坏开关 `GPU_VERIFICATION_SERVER_BREAK` 的表，只给测试证红用。

pub mod batch_scheduling;
pub mod break_switches;
pub mod card_memory_ledger;
pub mod card_worker;
pub mod cards_of_the_service;
pub mod gpu_judges_on_a_card;
pub mod library_pending_pool;
pub mod pending_pool;
pub mod service_configuration;
pub mod service_loop;
pub mod serving_from_the_library;
pub mod stream_identity;
pub mod verification_package_side;
