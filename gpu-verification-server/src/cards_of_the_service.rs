//! 服务持着的那一组卡：服务起来就照多机配置点名的这一台的全部卡开设备，一直持着到进程停（`crate::serving_from_the_library`），
//! 不经统一任务服务领卡、不还（卡归这个服务管：领、持、判、清场；统一任务服务只往判定库写待判行、读结论）。
//! 真卡是 `crate::gpu_judges_on_a_card::OpenedGpuCards`；不上卡的测试换成替身。

use singlefs_verification::crash_amplification::CardThatJudgedAPendingBlock;

use crate::card_memory_ledger::{CardIndex, CardOfTheService};
use crate::card_worker::JudgesOnACard;

/// 服务持着的一组卡：每张给服务的号与额度、在每张卡上造判器缓存的办法、写进结论块属性的那张卡是什么卡。
pub trait CardsOfTheService: Send + Sync + 'static {
    /// 装一个判器要的材料（与待判池配出的流同一种）。
    type Materials: Send + Sync + 'static;
    /// 一张卡上的判器缓存（在那张卡的工作线程里造，可以借着这组卡）。
    type Judges<'cards>: JudgesOnACard<StreamMaterials = Self::Materials>
    where
        Self: 'cards;

    fn cards_of_the_service(&self) -> Vec<CardOfTheService>;

    fn judges_on(&self, card: CardIndex) -> Self::Judges<'_>;

    /// 判了一块的那张卡：卡型与驱动版本进结论块属性里在哪判的那一格（验证包 `conclusion_frame_of_a_gpu_pending_block`）。
    fn card_that_judged(&self, card: CardIndex) -> CardThatJudgedAPendingBlock;
}
