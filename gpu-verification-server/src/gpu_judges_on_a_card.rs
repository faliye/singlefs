//! 真卡上的判器缓存：一张卡上按流的键（打包判表的摘要、内核源码的摘要）各留一个 `GpuCrashJudge`。
//! 同一张卡上同一套源码的管线只编一次（`GpuCard::pipelines_compiled_once_on_this_device`，判器装上时照源码摘要取），
//! 换出再装回只重传表、重开草稿区。服务持着的那一组真卡（[`OpenedGpuCards`]）在这里照多机配置开、在这里给每张卡造判器缓存。

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::ops::Range;
use std::path::Path;

use singlefs_verification::crash_amplification::{
    CardThatJudgedAPendingBlock, GpuPendingStreamMaterial,
};
use singlefs_verification::crash_judge_dispatch::{GpuCrashJudge, PackedJudgeTablesReadBack};
use singlefs_verification::crash_judge_gpu::JudgeKernelSources;
use singlefs_verification::gpu_card_configuration::{
    configuration_value, multi_host_addresses_of_the_caller, multi_host_configuration_file,
    parse_cards_by_priority, CARDS_BY_PRIORITY_KEY, CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY,
};
use singlefs_verification::gpu_unit_checks::{gpu_cards_by_priority, GpuCard};

use crate::card_memory_ledger::{CardIndex, CardOfTheService};
use crate::card_worker::JudgesOnACard;
use crate::cards_of_the_service::CardsOfTheService;
use crate::pending_pool::StateVerdict;
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

/// 只有判表与内核源码的材料：不经判定库待判行、直接交服务主循环的对拍用它。
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

/// 服务持着的真卡：按多机配置里这一台的卡的先后排号，额度照每张卡的显存额度。
pub struct OpenedGpuCards<Materials> {
    cards: Vec<GpuCard>,
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
        Self {
            cards,
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
        GpuJudgesOnACard::on_the_card(self.card(card))
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

/// 一张真卡上的判器缓存。
pub struct GpuJudgesOnACard<'card, Materials> {
    card: &'card GpuCard,
    judges: BTreeMap<StreamKey, GpuCrashJudge<'card>>,
    materials: PhantomData<fn() -> Materials>,
}

impl<'card, Materials> GpuJudgesOnACard<'card, Materials> {
    #[must_use]
    pub fn on_the_card(card: &'card GpuCard) -> Self {
        Self {
            card,
            judges: BTreeMap::new(),
            materials: PhantomData,
        }
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
    /// 它另按这张卡的显存额度压过一次）。
    ///
    /// # Panics
    /// 这条流没装着；同 `GpuCrashJudge::judge_states`。
    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> Vec<StateVerdict> {
        let judge = self
            .judges
            .get(&stream)
            .unwrap_or_else(|| panic!("要判的 {stream:?} 不在卡 {} 上", self.card.name));
        let states = usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize");
        if states <= judge.states_per_dispatch() {
            judge.judge_states_in_one_dispatch(ordinals)
        } else {
            judge.judge_states(ordinals)
        }
    }
}
