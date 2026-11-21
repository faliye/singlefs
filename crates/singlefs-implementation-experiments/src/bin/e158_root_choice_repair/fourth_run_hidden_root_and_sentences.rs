//! 第 4 次跑被藏的根落到哪里、对照句子与判词。

use super::*;

// ---------------------------------------------------------------------------
// 12.2 Q0 的去向栏（登记「读法写死」表「被藏那条根的去向」一行）：被测挂载交回（或崩在中途）之后、撤故障之后，按孪生读法解。
//      判的次序（执行员修订第 4 条写死）：被择或被施加 → 环在且被抛弃 → 根槽被盖 → 原样留着 → 其它。
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum HiddenRootDestination {
    InRingAndAbandoned,
    RootSlotOverwritten,
    ChosenOrApplied,
    LeftAsItWas,
    Other(String),
}

impl HiddenRootDestination {
    pub(super) fn name(&self) -> String {
        match self {
            HiddenRootDestination::InRingAndAbandoned => "in_ring_and_abandoned".to_string(),
            HiddenRootDestination::RootSlotOverwritten => "root_slot_overwritten".to_string(),
            HiddenRootDestination::ChosenOrApplied => "chosen_or_applied".to_string(),
            HiddenRootDestination::LeftAsItWas => "left_as_it_was".to_string(),
            HiddenRootDestination::Other(detail) => format!("other:{detail}"),
        }
    }
}

/// 被藏那条根：键 K_h 与它在起始镜像上住的根槽 s_h = (区域, 区域内槽)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HiddenRootPosition {
    pub(super) key: TimelineRoot,
    pub(super) region: u64,
    pub(super) slot: u64,
}

pub(super) fn hidden_root_position(
    base: &MemoryPool,
    key: TimelineRoot,
) -> Option<HiddenRootPosition> {
    let geometry = independent_geometry(base).ok()?;
    checker_image::valid_roots(base, &geometry)
        .into_iter()
        .find(|(_, _, view)| (view.instance, view.checkpoint_txg) == key)
        .map(|(region, slot, _)| HiddenRootPosition { key, region, slot })
}

/// 登记 D22（单元原子性怎么合成） 已定项 2「槽序」：txg 的根落在 (txg mod 3, (txg div 3) mod S)（PC-N-盖 的点预言，变异 M21）。
pub(super) fn predicted_root_ring_slot_of_a_publish(
    checkpoint_txg: u64,
    slots_per_region: u64,
) -> (u64, u64) {
    (
        checkpoint_txg % ROOT_RING_REGIONS_LOCAL,
        (checkpoint_txg / ROOT_RING_REGIONS_LOCAL) % slots_per_region,
    )
}

/// 按内容号表（装置的账，外加被测挂载写下的根沿用 E 的内容号）判去向；`tested_ok_effective` = 被测挂载交回 `Ok` 时 E 的键。
pub(super) fn hidden_root_destination(
    position: &HiddenRootPosition,
    content_numbers: &BTreeMap<TimelineRoot, u64>,
    hidden_content_number: u64,
    after: &MemoryPool,
    tested_ok_effective: Option<TimelineRoot>,
) -> HiddenRootDestination {
    let Ok(geometry) = independent_geometry(after) else {
        return HiddenRootDestination::Other("geometry".to_string());
    };
    if tested_ok_effective
        .and_then(|effective| content_numbers.get(&effective))
        .is_some_and(|number| *number >= hidden_content_number)
    {
        return HiddenRootDestination::ChosenOrApplied;
    }
    let roots = checker_image::valid_roots(after, &geometry);
    let at_the_hidden_slot = roots
        .iter()
        .find(|(region, slot, _)| (*region, *slot) == (position.region, position.slot))
        .map(|(_, _, view)| (view.instance, view.checkpoint_txg));
    let abandoned = abandoned_roots_on(after);
    let hidden_abandoned = abandoned
        .as_ref()
        .is_ok_and(|set| set.contains(&position.key));
    let slot_holds_the_hidden_root = at_the_hidden_slot == Some(position.key);
    if slot_holds_the_hidden_root && hidden_abandoned {
        return HiddenRootDestination::InRingAndAbandoned;
    }
    let ring_has_the_hidden_content_or_newer = roots.iter().any(|(_, _, view)| {
        content_numbers
            .get(&(view.instance, view.checkpoint_txg))
            .is_some_and(|number| *number >= hidden_content_number)
    });
    if at_the_hidden_slot.is_some()
        && !slot_holds_the_hidden_root
        && !ring_has_the_hidden_content_or_newer
    {
        return HiddenRootDestination::RootSlotOverwritten;
    }
    if tested_ok_effective.is_none() && slot_holds_the_hidden_root && !hidden_abandoned {
        return HiddenRootDestination::LeftAsItWas;
    }
    HiddenRootDestination::Other(format!(
        "slot_holds={}:abandoned={}:abandoned_reading={}:ring_has_content_at_least_hidden={ring_has_the_hidden_content_or_newer}:tested_ok={}",
        key_text(at_the_hidden_slot),
        hidden_abandoned,
        abandoned.as_ref().map_or("error", |_| "ok"),
        tested_ok_effective.is_some()
    ))
}

/// 装置的内容号表，外加一次可写挂载写下的根（写行、暖机、抬 F）沿用 E 的内容号（「读法写死」表「内容号跟根走」一行）。
pub(super) fn content_numbers_with_the_publishes_of(
    history: &ThirdRunHistory,
    mounted: Option<&Mounted>,
) -> BTreeMap<TimelineRoot, u64> {
    let mut numbers = history.content_number_by_root.clone();
    if let Some(mounted) = mounted {
        let carried = numbers
            .get(&root_pair(&mounted.output.effective_root))
            .copied()
            .unwrap_or(0);
        let mut roots = vec![root_pair(mounted.output.row_publish.root())];
        roots.extend(
            mounted
                .output
                .warm_up_publishes
                .iter()
                .map(|publish| root_pair(publish.root())),
        );
        for raised in mounted.output.space_admission.floor_raises() {
            roots.extend(
                raised
                    .publishes
                    .iter()
                    .map(|publish| root_pair(&publish.root)),
            );
        }
        for root in roots {
            numbers.insert(root, carried);
        }
    }
    numbers
}

/// 一份镜像的指纹（Q5-同成 要「起始镜像逐字节相同」；也给 H1f 的前缀跨臂核同一份）：每块盘写过的扇区按序号与字节混进 FNV-1a。
pub(super) fn image_fingerprint(pool: &MemoryPool) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for (identity, device) in &pool.devices {
        mix(&identity.0.to_le_bytes());
        for sector in device.written_sectors_in(DeviceOffsetInBytes(0), pool.device_size_in_bytes) {
            mix(&sector.to_le_bytes());
            mix(&device.read(DeviceOffsetInBytes(sector * SECTOR_BYTES_LOCAL), 512));
        }
    }
    hash
}

// ---------------------------------------------------------------------------
// 12.3 阳性对照的句子（登记 5.4：每一句带标签、各句单独打一个字段；判定词 pass / fail / not_constructible，
//      条件式的 [测] 句前件不成立记 vacuous）。V1 按标签路由（11.1），由 `r4-compare` 机械算。
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SentenceTag {
    /// [测]：只由装置自己的账决定真假。
    Measured,
    /// [臂]：由臂的定义推出的这一臂的结局。
    ArmDefinition,
    /// [今]：对今天那一臂的底座怎么表现的描述。
    TodayDescription,
}

impl SentenceTag {
    pub(super) fn name(self) -> &'static str {
        match self {
            SentenceTag::Measured => "measure",
            SentenceTag::ArmDefinition => "arm",
            SentenceTag::TodayDescription => "today",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SentenceVerdict {
    Pass,
    Fail,
    NotConstructible,
    Vacuous,
}

impl SentenceVerdict {
    pub(super) fn name(self) -> &'static str {
        match self {
            SentenceVerdict::Pass => "pass",
            SentenceVerdict::Fail => "fail",
            SentenceVerdict::NotConstructible => "not_constructible",
            SentenceVerdict::Vacuous => "vacuous",
        }
    }

    pub(super) fn of(holds: bool) -> SentenceVerdict {
        if holds {
            SentenceVerdict::Pass
        } else {
            SentenceVerdict::Fail
        }
    }

    /// 条件式：前件不成立 ⇒ vacuous，成立 ⇒ 后件定 pass / fail。
    pub(super) fn conditional(antecedent: bool, consequent: bool) -> SentenceVerdict {
        if antecedent {
            SentenceVerdict::of(consequent)
        } else {
            SentenceVerdict::Vacuous
        }
    }
}

pub(super) struct ControlSentence {
    pub(super) tag: SentenceTag,
    pub(super) name: &'static str,
    pub(super) verdict: SentenceVerdict,
    /// 打不中走哪一条（[臂] 句：F9 / F10 / F12 / F16 / F18；[今] 句：F1 / F2 / F4 / F15 / F17 / F19；[测] 句：V1 作废）。
    pub(super) clause: &'static str,
}

pub(super) fn sentence(
    tag: SentenceTag,
    name: &'static str,
    verdict: SentenceVerdict,
    clause: &'static str,
) -> ControlSentence {
    ControlSentence {
        tag,
        name,
        verdict,
        clause,
    }
}

/// 一条对照的总判定：有 fail ⇒ fail；否则有 not_constructible ⇒ not_constructible；否则 pass。
pub(super) fn overall_verdict(sentences: &[ControlSentence]) -> &'static str {
    if sentences
        .iter()
        .any(|sentence| sentence.verdict == SentenceVerdict::Fail)
    {
        "fail"
    } else if sentences
        .iter()
        .any(|sentence| sentence.verdict == SentenceVerdict::NotConstructible)
    {
        "not_constructible"
    } else {
        "pass"
    }
}

pub(super) fn emit_fourth_run_positive_control(
    arm: &str,
    control: &str,
    coordinates: &str,
    sentences: &[ControlSentence],
    detail: &str,
) {
    let fields: Vec<String> = sentences
        .iter()
        .map(|sentence| {
            format!(
                "sentence_{}_{}={}",
                sentence.tag.name(),
                sentence.name,
                sentence.verdict.name()
            )
        })
        .collect();
    let routes: Vec<&str> = sentences
        .iter()
        .filter(|sentence| {
            matches!(
                sentence.verdict,
                SentenceVerdict::Fail | SentenceVerdict::NotConstructible
            )
        })
        .map(|sentence| sentence.clause)
        .collect();
    emit_result(&format!(
        "name=r4_positive_control arm={arm} control={control} {coordinates} verdict={} sentences={} {} missed_routes={} {detail}",
        overall_verdict(sentences),
        sentences.len(),
        fields.join(" "),
        if routes.is_empty() { "none".to_string() } else { routes.join(",") }
    ));
}

/// 阳性对照与 L 族那几格顺带做的 7.3 第一行对拍与 F13（今天那一臂）打成结果行（第 3 次跑 `emit_scratch_findings` 的第 4 次跑那一份）。
pub(super) fn emit_fourth_run_scratch_findings(
    family: &str,
    arm: &str,
    scratch: &ThirdRunFamilySummary,
) {
    emit_result(&format!(
        "name=r4_cross_checks family={family} arm={arm} s4_mismatches={} f13_checked={} f13_failures={}",
        scratch.stop_condition_four_mismatches.len(),
        scratch.failure_clause_thirteen_checks,
        scratch.failure_clause_thirteen_findings.len()
    ));
    for (kind, list) in [
        ("s4", &scratch.stop_condition_four_mismatches),
        ("f13", &scratch.failure_clause_thirteen_findings),
    ] {
        for detail in list {
            emit_result(&format!(
                "name=r4_family_finding family={family} arm={arm} kind={kind} detail={detail:?}"
            ));
        }
    }
}

#[cfg(test)]
mod fourth_run_tests;
