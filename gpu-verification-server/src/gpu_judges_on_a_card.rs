//! 真卡上的判器缓存：一张卡上按流的键（打包判表的摘要、内核源码的摘要）各留一个 `GpuCrashJudge`。
//! 同一张卡上同一套源码的管线只编一次（`GpuCard::pipelines_compiled_once_on_this_device`，判器装上时照源码摘要取），
//! 换出再装回只重传表、重开草稿区。服务持着的那一组真卡（[`OpenedGpuCards`]）在这里照多机配置开、在这里给每张卡造判器缓存。
//! 核对待判行那一族的一组块在同一张卡上照材料建一个 `GpuFactsVerifier`（验证包 `crash_verify_gpu`）、逐块核、核完丢掉，不缓存。
//! 服务进程里卡上装判器的材料是 [`MaterialsOfAServedStream`]：镜像判定那一族与判表待判行那一族各一臂。
//! 判的时候 panic 之后（`crate::card_worker` 文件头「接住 Out of Memory」）读这张卡的设备情况（验证包 `DeviceConditionOfACard`：
//! wgpu 没接住的 Out of Memory 先记进它再 panic，设备丢失的回调也记进它），照 [`device_after_a_panic`] 分四种：
//! - 设备丢了、验证包认得出是显存不够丢的（`DeviceConditionOfACard::out_of_memory_explaining_a_failure_of_the_work` 照设备丢失回调里认出的缘由：
//!   wgpu 因显存不够丢设备时给的那一句，或上一回 panic 之后记过 Out of Memory）：丢掉全部判器，重开这张卡的设备（下面「重开设备」）；
//! - 设备为别的原因丢了：交丢失原因（卡坏）；
//! - 设备没丢而记的 Out of Memory 次数比上一回多：丢掉全部判器；丢掉的缓冲与上一波新表的暂存挂在 wgpu 的待写列表里，要等下一次提交做完才还给驱动，
//!   工作线程随后调 [`JudgesOnACard::settle_after_running_out_of_memory`]：在这张卡上做一次提交、等它做完。服务不直接依赖 wgpu，那一次提交借验证包
//!   `GpuCard::crc32c_of_ranges` 算一段空字节的单元校验和（它提交一次、等这一次做完）；
//! - 设备没丢、也没记过 Out of Memory（判法自己的 panic）：丢掉全部判器（panic 停在哪一步不知道，缓存与主循环的账一起清掉）。
//!
//! 重开设备：服务持着的那一组卡（[`OpenedGpuCards`]）每张卡留 [`DEVICE_REOPENINGS_OF_A_CARD`] 个空格，重开时照验证包 `gpu_cards_by_priority`
//! 按这张卡的 `nvidia-smi` 序号与配置的额度另开一个设备，放进头一个空格，之后这张卡的判器都装在新设备上；丢了的那一个设备与它之前开的照旧持着，
//! 到这组卡丢掉（服务停）才放。格用完了、这张卡没有配置的额度（不是照多机配置开的）、另开没成，都交设备丢了（卡坏）。

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::ops::Range;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::{Arc, OnceLock};

use singlefs_verification::crash_amplification::{
    CardThatJudgedAPendingBlock, GpuPendingStreamMaterial,
};
use singlefs_verification::crash_judge_dispatch::{GpuCrashJudge, PackedJudgeTablesReadBack};
use singlefs_verification::crash_judge_gpu::JudgeKernelSources;
use singlefs_verification::crash_verify_gpu::GpuFactsVerifier;
use singlefs_verification::gpu_card_configuration::{
    configuration_value, multi_host_addresses_of_the_caller, multi_host_configuration_file,
    parse_cards_by_priority, CARDS_BY_PRIORITY_KEY, CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY,
};
use singlefs_verification::gpu_memory_ledger::{
    DeviceConditionOfACard, DeviceWhenTheWorkReturned, WorkOnTheCardOfThisThread,
};
use singlefs_verification::gpu_unit_checks::{gpu_cards_by_priority, GpuCard};

use crate::card_memory_ledger::{CardIndex, CardOfTheService};
use crate::card_worker::{panic_message, CardAfterAPanic, JudgedOrdinals, JudgesOnACard};
use crate::cards_of_the_service::CardsOfTheService;
use crate::facts_verifier_family::FactsVerifierMaterial;
use crate::stream_identity::{StreamFacts, StreamKey};

/// 装一条流的判器要的两样：从判定库读回、核过摘要的打包判表，与编进判器的三份内核源码。
pub trait JudgeMaterialsOnACard: Send + Sync + 'static {
    fn packed_judge_tables(&self) -> &PackedJudgeTablesReadBack;

    fn kernel_sources(&self) -> &JudgeKernelSources;

    /// 这条流的键。
    fn stream_key(&self) -> StreamKey {
        StreamKey::of_the_tables_under_the_kernel_sources(
            self.packed_judge_tables(),
            self.kernel_sources(),
        )
    }

    /// 批次调度要的这条流的事实。
    fn stream_facts(&self) -> StreamFacts {
        StreamFacts::of_the_tables(self.packed_judge_tables())
    }
}

/// 只有判表与内核源码的材料：判表待判行那一族上卡用它（[`MaterialsOfAServedStream::PackedTables`]），不经判定库待判行、直接交服务主循环的对拍也用它。
pub struct GpuStreamMaterials {
    pub tables: PackedJudgeTablesReadBack,
    pub kernel_sources: JudgeKernelSources,
}

impl JudgeMaterialsOnACard for GpuStreamMaterials {
    fn packed_judge_tables(&self) -> &PackedJudgeTablesReadBack {
        &self.tables
    }

    fn kernel_sources(&self) -> &JudgeKernelSources {
        &self.kernel_sources
    }
}

/// 待判行那条流的材料（验证包 `material_of_a_gpu_pending_stream` 读回的）：判表与内核源码照它装判器，编结论帧另用它别的几样。
impl JudgeMaterialsOnACard for GpuPendingStreamMaterial {
    fn packed_judge_tables(&self) -> &PackedJudgeTablesReadBack {
        &self.packed_judge_tables
    }

    fn kernel_sources(&self) -> &JudgeKernelSources {
        &self.kernel_sources
    }
}

/// 服务进程的主循环里一条流的材料：两族上卡判的都是 `GpuCrashJudge`（同一条 `register_stream` / `submit_block` 路、同一本判器显存账），
/// 只差材料从哪一族的行读回；主循环与卡只经 [`JudgeMaterialsOnACard`] 看它。
pub enum MaterialsOfAServedStream {
    /// 镜像判定那一族（表 `1D`）：编结论帧另用这一份别的几样，验证包那一侧留着同一份。
    ImageJudgement(Arc<GpuPendingStreamMaterial>),
    /// 判表待判行那一族（表 `20`）：照判表摘要读回的判表与照旋钮栏取的内核源码（`crate::packed_tables_family` 文件头「内核源码」）。
    PackedTables(GpuStreamMaterials),
}

impl JudgeMaterialsOnACard for MaterialsOfAServedStream {
    fn packed_judge_tables(&self) -> &PackedJudgeTablesReadBack {
        match self {
            Self::ImageJudgement(material) => material.packed_judge_tables(),
            Self::PackedTables(material) => material.packed_judge_tables(),
        }
    }

    fn kernel_sources(&self) -> &JudgeKernelSources {
        match self {
            Self::ImageJudgement(material) => material.kernel_sources(),
            Self::PackedTables(material) => material.kernel_sources(),
        }
    }
}

/// 一张卡在这个进程里至多重开几次设备（文件头「重开设备」）：丢了的设备到服务停才放，次数有上限，一张卡反复因显存不够丢设备时不无限另开。
pub const DEVICE_REOPENINGS_OF_A_CARD: usize = 4;

/// 服务持着的真卡：按多机配置里这一台的卡的先后排号，额度照每张卡的显存额度；每张卡另留重开设备的空格（文件头「重开设备」）。
pub struct OpenedGpuCards<Materials> {
    cards: Vec<GpuCard>,
    reopened_devices: Vec<Vec<OnceLock<GpuCard>>>,
    materials: PhantomData<fn() -> Materials>,
}

impl<Materials> OpenedGpuCards<Materials> {
    /// 照 `repository` 那棵仓的多机配置开这一台点名的全部卡：显卡先后取 `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`，没写取
    /// `GPU_CARDS_BY_PRIORITY`（与统一任务服务分卡读的同一个次序：卡归这个服务独占，照独占时那一份），主机数组与这一台照多机配置判法认的。
    ///
    /// # Errors
    /// 找不到多机配置；两个键都没写；卡串写法不对、认不出这一台；这一台一张卡都没点名；点名的卡有开不成的（原因在 `GPU_CARD_*` 那几行）。
    pub fn open_the_cards_named_for_this_host(repository: &Path) -> Result<Self, String> {
        let configuration = multi_host_configuration_file(repository)?.ok_or_else(|| {
            format!(
                "仓 {} 找不到多机配置（multi-host.env）：服务开哪几张卡只照它",
                repository.display()
            )
        })?;
        let written = |key: &str| {
            configuration_value(&configuration, key).filter(|value| !value.trim().is_empty())
        };
        let card_string = written(CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY)
            .or_else(|| written(CARDS_BY_PRIORITY_KEY))
            .ok_or_else(|| {
                format!(
                    "多机配置 {} 里 {CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY} 与 {CARDS_BY_PRIORITY_KEY} 都没写：服务不猜开哪几张卡",
                    configuration.display()
                )
            })?;
        let addresses = multi_host_addresses_of_the_caller()?;
        let configured = parse_cards_by_priority(&card_string, &addresses)?.cards_of_this_host();
        if configured.is_empty() {
            return Err(format!("多机配置的卡串「{card_string}」里没有这一台的卡"));
        }
        let cards = gpu_cards_by_priority(&configured);
        if cards.len() != configured.len() {
            return Err(format!(
                "卡串「{card_string}」点名这一台 {} 张卡，开成了 {} 张（开不成的原因在上面 GPU_CARD_* 那几行）：点名的每一张都要开得成",
                configured.len(),
                cards.len()
            ));
        }
        Ok(Self::of_the_cards(cards))
    }

    /// 已经开好的卡（上卡的测试从卡池借来的那一张）。
    #[must_use]
    pub fn of_the_cards(cards: Vec<GpuCard>) -> Self {
        let reopened_devices = cards
            .iter()
            .map(|_| {
                (0..DEVICE_REOPENINGS_OF_A_CARD)
                    .map(|_| OnceLock::new())
                    .collect()
            })
            .collect();
        Self {
            cards,
            reopened_devices,
            materials: PhantomData,
        }
    }

    fn card(&self, card: CardIndex) -> &GpuCard {
        let position = usize::try_from(card.0).expect("装得进 usize");
        &self.cards[position]
    }
}

impl<Materials: JudgeMaterialsOnACard> CardsOfTheService for OpenedGpuCards<Materials> {
    type Materials = Materials;
    type Judges<'cards> = GpuJudgesOnACard<'cards, Materials>;

    fn cards_of_the_service(&self) -> Vec<CardOfTheService> {
        self.cards
            .iter()
            .enumerate()
            .map(|(position, card)| CardOfTheService {
                card: CardIndex(u32::try_from(position).expect("卡数装得进 u32")),
                service_quota_mebibytes: card.memory_budget_mebibytes(),
            })
            .collect()
    }

    fn judges_on(&self, card: CardIndex) -> GpuJudgesOnACard<'_, Materials> {
        let position = usize::try_from(card.0).expect("装得进 usize");
        GpuJudgesOnACard::on_the_card_reopening_into(
            &self.cards[position],
            &self.reopened_devices[position],
        )
    }

    fn card_that_judged(&self, card: CardIndex) -> CardThatJudgedAPendingBlock {
        let gpu_card = self.card(card);
        CardThatJudgedAPendingBlock {
            card_index: gpu_card.index,
            card_name: gpu_card.name.clone(),
            card_driver_version: gpu_card.driver_version.clone(),
        }
    }
}

/// panic 之后这张卡的设备怎样（文件头的四种）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceAfterAPanic {
    /// 设备丢了，验证包认得出是显存不够：`reason` 是它说的那一句。
    LostForWantOfMemory { reason: String },
    /// 设备为别的原因丢了：`reason` 是记下的丢失原因。
    LostOtherwise { reason: String },
    /// 设备没丢，记的 Out of Memory 比 `out_of_memory_reports_seen` 多。
    RanOutOfMemory,
    /// 设备没丢、也没多记 Out of Memory（判法自己的 panic）。
    PanickedOtherwise,
}

/// 照设备情况认 panic 之后这张卡的设备怎样（文件头）：`out_of_memory_reports_seen` 是上一回 panic 之后（或造判器缓存时）这个设备记过几次 Out of Memory。
/// 设备丢了的，是不是显存不够照验证包的判法认（`DeviceConditionOfACard::out_of_memory_explaining_a_failure_of_the_work`，交回那一刻设备已丢，
/// 活另起一个号：这之后记的 Out of Memory 都不算在它上面，只看丢失的缘由与这之后记没记过），服务不另比 wgpu 的那一句。
#[must_use]
pub fn device_after_a_panic(
    condition: &DeviceConditionOfACard,
    out_of_memory_reports_seen: usize,
) -> DeviceAfterAPanic {
    if let Some(lost) = condition.lost_reason() {
        let work = WorkOnTheCardOfThisThread::begin();
        let for_want_of_memory = condition.out_of_memory_explaining_a_failure_of_the_work(
            work.identifier(),
            out_of_memory_reports_seen,
            DeviceWhenTheWorkReturned::Lost,
        );
        drop(work);
        return match for_want_of_memory {
            Some(reason) => DeviceAfterAPanic::LostForWantOfMemory { reason },
            None => DeviceAfterAPanic::LostOtherwise { reason: lost },
        };
    }
    if condition.out_of_memory_reports() > out_of_memory_reports_seen {
        DeviceAfterAPanic::RanOutOfMemory
    } else {
        DeviceAfterAPanic::PanickedOtherwise
    }
}

/// 一张真卡上的判器缓存。
pub struct GpuJudgesOnACard<'card, Materials> {
    /// 判器装在它上面的那个设备：起先是服务开的那一个，重开之后是新开的那一个（文件头「重开设备」）。
    card: &'card GpuCard,
    /// 这张卡重开设备的空格（[`OpenedGpuCards`] 留的；不经它造的是空的，重开不了）。
    reopening_slots: &'card [OnceLock<GpuCard>],
    judges: BTreeMap<StreamKey, GpuCrashJudge<'card>>,
    /// 造这一份时、或上一回 panic 之后这张卡的设备上记过几次 Out of Memory（文件头）。
    out_of_memory_reports_seen: usize,
    materials: PhantomData<fn() -> Materials>,
}

impl<'card, Materials> GpuJudgesOnACard<'card, Materials> {
    /// 一张卡上的判器缓存，设备因显存不够丢了不重开（上卡的对拍直接造的那一种）。
    #[must_use]
    pub fn on_the_card(card: &'card GpuCard) -> Self {
        Self::on_the_card_reopening_into(card, &[])
    }

    /// 一张卡上的判器缓存，设备因显存不够丢了时照 `reopening_slots` 重开（文件头「重开设备」）。
    #[must_use]
    pub fn on_the_card_reopening_into(
        card: &'card GpuCard,
        reopening_slots: &'card [OnceLock<GpuCard>],
    ) -> Self {
        Self {
            card,
            reopening_slots,
            judges: BTreeMap::new(),
            out_of_memory_reports_seen: card.device_condition().out_of_memory_reports(),
            materials: PhantomData,
        }
    }

    /// 重开这张卡的设备（文件头「重开设备」）：判器已丢掉；开好的放进头一个空格，之后判器装在它上面。
    ///
    /// # Errors
    /// 空格用完了；这张卡没有配置的额度；照序号另开没成。
    fn reopen_the_device(&mut self) -> Result<(), String> {
        let slots: &'card [OnceLock<GpuCard>] = self.reopening_slots;
        let Some(slot) = slots.iter().find(|slot| slot.get().is_none()) else {
            return Err(format!(
                "这张卡在这个进程里已重开过 {} 次设备（DEVICE_REOPENINGS_OF_A_CARD），不再重开",
                slots.len()
            ));
        };
        let quota = self.card.memory_quota_mebibytes.ok_or_else(|| {
            "这张卡没有配置的显存额度（不是照多机配置开的），重开不了".to_string()
        })?;
        let reopened = gpu_cards_by_priority(&[(self.card.index, quota)])
            .pop()
            .ok_or_else(|| {
                format!(
                    "照 nvidia-smi 序号 {} 另开设备没成（原因在上面 GPU_CARD_* 那几行）",
                    self.card.index
                )
            })?;
        slot.set(reopened).map_err(|_| {
            "重开设备的那一格已被占了（同一张卡只有一条工作线程，不该有）".to_string()
        })?;
        let card = slot
            .get()
            .ok_or_else(|| "重开设备的那一格放进去之后读不到".to_string())?;
        self.card = card;
        self.out_of_memory_reports_seen = card.device_condition().out_of_memory_reports();
        Ok(())
    }
}

impl<Materials: JudgeMaterialsOnACard> JudgesOnACard for GpuJudgesOnACard<'_, Materials> {
    type StreamMaterials = Materials;

    /// # Panics
    /// 这条流已经装着（计划与工作线程的缓存对不上）；同 `GpuCrashJudge::on_card_compiling_these_kernel_sources`。
    fn load_the_judge(
        &mut self,
        stream: StreamKey,
        materials: &Materials,
        states_per_dispatch: u64,
    ) {
        assert!(
            !self.judges.contains_key(&stream),
            "{stream:?} 已经装在卡 {} 上",
            self.card.name
        );
        let judge = GpuCrashJudge::on_card_compiling_these_kernel_sources(
            self.card,
            materials.packed_judge_tables(),
            usize::try_from(states_per_dispatch).expect("一次派活的态数装得进 usize"),
            materials.kernel_sources(),
        );
        self.judges.insert(stream, judge);
    }

    /// # Panics
    /// 这条流没装着。
    fn evict_the_judge(&mut self, stream: StreamKey) {
        let evicted = self.judges.remove(&stream);
        assert!(
            evicted.is_some(),
            "要换出的 {stream:?} 不在卡 {} 上",
            self.card.name
        );
    }

    /// 一段装得进判器一次派活的就一次派活判；装不下的交判器自己按宽度分几次派（判器开出来的草稿区可能比计划要的小：
    /// 它另按这张卡的显存额度压过一次）。派了几次照判器自己的派活计数（`GpuCrashJudge::time_spent` 的 `dispatches`）前后之差。
    ///
    /// # Panics
    /// 这条流没装着；同 `GpuCrashJudge::judge_states`。
    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> JudgedOrdinals {
        let judge = self
            .judges
            .get(&stream)
            .unwrap_or_else(|| panic!("要判的 {stream:?} 不在卡 {} 上", self.card.name));
        let states = usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize");
        let dispatches_before = judge.time_spent().dispatches;
        let verdicts = if states <= judge.states_per_dispatch() {
            judge.judge_states_in_one_dispatch(ordinals)
        } else {
            judge.judge_states(ordinals)
        };
        JudgedOrdinals {
            verdicts,
            dispatches: judge
                .time_spent()
                .dispatches
                .saturating_sub(dispatches_before),
        }
    }

    /// 照材料建一个核对器（验证包 `GpuFactsVerifier::new_by_the_segment_expansions`：事实表与每段的拆法上卡一次），逐块
    /// `verify_states`（一块一次派活），交回之后核对器随这一次调用丢掉。
    ///
    /// # Panics
    /// 同 `GpuFactsVerifier::new_by_the_segment_expansions` 与 `GpuFactsVerifier::verify_states`。
    fn verify_the_facts(
        &mut self,
        material: &dyn FactsVerifierMaterial,
        blocks: &[Range<u64>],
    ) -> Vec<Vec<u32>> {
        let verifier = GpuFactsVerifier::new_by_the_segment_expansions(
            self.card,
            material.facts(),
            material.segment_expansions(),
        );
        blocks
            .iter()
            .map(|ordinals| verifier.verify_states(ordinals.clone()))
            .collect()
    }

    /// 照设备情况认（文件头，[`device_after_a_panic`]）：因显存不够丢了的丢掉全部判器、重开设备（重开没成交设备丢了）；为别的原因丢了交丢失原因；
    /// 没丢的丢掉全部判器，交显存不够或判法自己的 panic。
    fn after_a_panic(&mut self) -> CardAfterAPanic {
        let condition = self.card.device_condition();
        let seen_before = self.out_of_memory_reports_seen;
        self.out_of_memory_reports_seen = condition.out_of_memory_reports();
        let judges_dropped = self.judges.len();
        match device_after_a_panic(condition, seen_before) {
            DeviceAfterAPanic::LostOtherwise { reason } => CardAfterAPanic::DeviceLost { reason },
            DeviceAfterAPanic::LostForWantOfMemory { reason } => {
                self.judges.clear();
                match self.reopen_the_device() {
                    Ok(()) => CardAfterAPanic::DeviceReopenedAfterRunningOutOfMemory {
                        judges_dropped,
                        reason: format!("设备因显存不够丢了（{reason}），已重开这张卡的设备"),
                    },
                    Err(not_reopened) => CardAfterAPanic::DeviceLost {
                        reason: format!("{reason}；设备因显存不够丢的，重开没成：{not_reopened}"),
                    },
                }
            }
            DeviceAfterAPanic::RanOutOfMemory => {
                self.judges.clear();
                CardAfterAPanic::RanOutOfMemory { judges_dropped }
            }
            DeviceAfterAPanic::PanickedOtherwise => {
                self.judges.clear();
                CardAfterAPanic::PanickedOtherwise { judges_dropped }
            }
        }
    }

    /// 在这张卡上做一次提交、等它做完（文件头：借验证包 `GpuCard::crc32c_of_ranges` 算一段空字节）；
    /// 那一次提交若因 wgpu 提交序号失步等原因 panic，且这张卡留有重开设备的空格，就重开设备来清掉待写列表与失步状态。
    fn settle_after_running_out_of_memory(&mut self) -> Result<(), String> {
        let card = self.card;
        let nothing: &[u8] = &[];
        match catch_unwind(AssertUnwindSafe(|| card.crc32c_of_ranges(&[nothing]))) {
            Ok(checksums) if checksums.len() == 1 => Ok(()),
            Ok(checksums) => Err(format!(
                "算一段空字节的单元校验和交回 {} 个结果，不是 1 个",
                checksums.len()
            )),
            Err(panic) => {
                let message = panic_message(panic.as_ref());
                if !self.reopening_slots.is_empty() {
                    self.reopen_the_device().map_err(|not_reopened| {
                        format!("{message}；重开这张卡的设备也没成：{not_reopened}")
                    })
                } else {
                    Err(message)
                }
            }
        }
    }

    fn pci_bus_identifier(&self) -> Option<String> {
        Some(self.card.pci_bus_identifier.clone())
    }
}
