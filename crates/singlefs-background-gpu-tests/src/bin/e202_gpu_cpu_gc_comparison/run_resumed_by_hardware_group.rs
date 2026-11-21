//! 续跑按硬件组归组：产物里已有的重复归组，这一趟只补自己那一组。

use super::*;

/// 续跑归组的是哪一种跑：做完一次重复的那一行与结果行的种类、这一档这一组设定的行怎么认、`name=resume` 一行开头的字段、下一个重复号怎么取。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RunResumedByGroup {
    /// 交替跑（敏感度的交替跑那一段同）：这一档的 `name=repeat_done` 与 `name=run`。
    Lockstep { tier: UnitAreaTier },
    /// 并发跑（敏感度的并发跑那一段同）：这一档、这个前台间隔的 `name=concurrent_repeat_done` 与 `name=concurrent_run`。
    Concurrent {
        tier: UnitAreaTier,
        foreground_interval_milliseconds: u128,
    },
}

impl RunResumedByGroup {
    pub(super) const fn tier(self) -> UnitAreaTier {
        match self {
            RunResumedByGroup::Lockstep { tier } | RunResumedByGroup::Concurrent { tier, .. } => {
                tier
            }
        }
    }

    const fn kind_of_the_repeat_done_line(self) -> &'static str {
        match self {
            RunResumedByGroup::Lockstep { .. } => "repeat_done",
            RunResumedByGroup::Concurrent { .. } => "concurrent_repeat_done",
        }
    }

    const fn kind_of_the_run_line(self) -> &'static str {
        match self {
            RunResumedByGroup::Lockstep { .. } => "run",
            RunResumedByGroup::Concurrent { .. } => "concurrent_run",
        }
    }

    /// 这一行是不是这一档、这一组设定的。
    fn is_of_this_setting(self, fields: &BTreeMap<String, String>) -> bool {
        let is_of_this_tier = fields.get("tier").map(String::as_str) == Some(self.tier().name());
        match self {
            RunResumedByGroup::Lockstep { .. } => is_of_this_tier,
            RunResumedByGroup::Concurrent {
                foreground_interval_milliseconds,
                ..
            } => {
                is_of_this_tier
                    && fields.get("foreground_interval_milliseconds")
                        == Some(&foreground_interval_milliseconds.to_string())
            }
        }
    }

    /// `name=resume` 一行开头那几个原有的设定字段。
    fn leading_fields_of_the_resume_line(self) -> String {
        match self {
            RunResumedByGroup::Lockstep { tier } => format!("tier={}", tier.name()),
            RunResumedByGroup::Concurrent {
                tier,
                foreground_interval_milliseconds,
            } => format!(
                "command=concurrent tier={} foreground_interval_milliseconds={foreground_interval_milliseconds}",
                tier.name()
            ),
        }
    }

    /// 下一个重复号：全产物统一往后编（最大号加 1），不按组各编。交替跑数整份产物里的 `name=run` 与 `name=repeat_done`；
    /// 并发跑数这一档、这个间隔的 `name=concurrent_run` 与 `name=concurrent_repeat_done`。
    pub(super) fn next_repeat(self, lines: &[KindAndFieldsOfAResultLine]) -> u32 {
        lines
            .iter()
            .filter(|(name, fields)| match self {
                RunResumedByGroup::Lockstep { .. } => name == "run" || name == "repeat_done",
                RunResumedByGroup::Concurrent { .. } => {
                    (name == "concurrent_run" || name == "concurrent_repeat_done")
                        && self.is_of_this_setting(fields)
                }
            })
            .filter_map(|(_name, fields)| fields.get("repeat")?.parse::<u32>().ok())
            .max()
            .map_or(0, |repeat| repeat + 1)
    }
}

/// 取卡之后、开跑之前那一段的输入（交替跑与并发跑共用，登记第十二节修订第 16 条「装置的结构要求」）。
pub(super) struct InputsBeforeTheRun<'inputs> {
    pub(super) lines_already_in_the_product: &'inputs [KindAndFieldsOfAResultLine],
    pub(super) session_number: u64,
    pub(super) run: RunResumedByGroup,
    /// 点了的臂：名字与挂载用的后台计算，照命令行的次序。
    pub(super) arms_requested: &'inputs [(String, ComputeOfTheArm)],
    /// 全部卡臂开几张（`--cards`）；并发跑没有全部卡臂，交 [`CardsOfTheEveryCardArm::EveryHandedOverCard`]。
    pub(super) cards_of_the_every_card_arm: CardsOfTheEveryCardArm,
    /// 交来的卡：每张的地址、型号、驱动、驱动信息（按 PCI 地址次序）与交来的先后。
    pub(super) cards: &'inputs CardsSeenByThisProcess,
    pub(super) host_hardware: &'inputs HostHardware,
}

/// 取卡之后、开跑之前那一段算出的：要打的行（`not_run`、记卡、`hardware_group`、`resume`，按这个次序）、要跑的臂（`arms_requested` 里的下标）、
/// 这一趟的硬件组、全部卡臂开的那几张、这一组已做完的重复（重复号 → 做完它的那一趟）与读回的结果行、下一个重复号。
#[derive(Debug, PartialEq)]
pub(super) struct PlanOfTheRun {
    pub(super) lines: Vec<(&'static str, String)>,
    pub(super) indices_of_the_arms_run: Vec<usize>,
    pub(super) group: HardwareGroup,
    pub(super) discrete_gpus_of_the_every_card_arm: Vec<DiscreteGpuSeenByThisProcess>,
    pub(super) repeats_of_the_group_done: BTreeMap<u32, String>,
    /// 这一组做完的重复里 `void=no` 的结果行（`name=run` 或 `name=concurrent_run`）的字段。
    pub(super) run_lines_of_the_group: Vec<BTreeMap<String, String>>,
    pub(super) next_repeat: u32,
}

/// 取卡之后、开跑之前那一段的拒绝：要打的行与退出码（这一趟一条臂都不开跑）。
#[derive(Debug, PartialEq, Eq)]
pub(super) struct RefusalBeforeTheRun {
    lines: Vec<(&'static str, String)>,
    exit_code: i32,
}

/// 产物里每一趟（`session=`）的组标识，取那一趟的 `name=hardware_group` 行。
fn hardware_groups_of_the_sessions(
    lines_already_in_the_product: &[KindAndFieldsOfAResultLine],
) -> BTreeMap<String, String> {
    lines_already_in_the_product
        .iter()
        .filter(|(name, _fields)| name == "hardware_group")
        .filter_map(|(_name, fields)| {
            Some((fields.get("session")?.clone(), fields.get("group")?.clone()))
        })
        .collect()
}

/// 产物里有结果行（`name=run`、`name=concurrent_run`）而没有 `name=hardware_group` 行的头一趟：那几次重复认不出归哪一组。
fn first_session_with_results_and_without_a_hardware_group(
    lines_already_in_the_product: &[KindAndFieldsOfAResultLine],
    hardware_groups_of_the_sessions: &BTreeMap<String, String>,
) -> Option<String> {
    lines_already_in_the_product
        .iter()
        .filter(|(name, _fields)| name == "run" || name == "concurrent_run")
        .filter_map(|(_name, fields)| fields.get("session"))
        .find(|session| !hardware_groups_of_the_sessions.contains_key(*session))
        .cloned()
}

/// 取卡之后、开跑之前那一段（登记第十二节修订第 16 条「装置的结构要求」「续跑核什么」）：不读环境、不碰卡、不写文件。
/// 次序：产物里有一趟有结果行而没有硬件组行就拒（退 2，`product_session_without_a_hardware_group`）；`--cards` 多于交来的拒（退 2）；
/// 逐条臂判跑不跑（报本次未跑的打 `not_run`）；这一台一张卡都没交，要跑的臂也打 `not_run`、退 77；再只给这一趟真跑的用卡的臂记卡、
/// 算这一趟的硬件组；产物里已有的做完的重复按它那一趟的组归组，这一趟只接着补自己那一组（别的组的重复留在产物里，不改、不判、不并进）。
/// 卡与硬件不与产物里已有的比对，不因换了卡、换了台而停。
///
/// # Errors
/// 上面那三种拒绝，带要打的行与退出码。
pub(super) fn plan_of_the_run_after_the_cards(
    inputs: &InputsBeforeTheRun<'_>,
) -> Result<PlanOfTheRun, RefusalBeforeTheRun> {
    let tier = inputs.run.tier();
    let lines_already_in_the_product = inputs.lines_already_in_the_product;
    let mut hardware_groups_of_the_sessions =
        hardware_groups_of_the_sessions(lines_already_in_the_product);
    if let Some(session) = first_session_with_results_and_without_a_hardware_group(
        lines_already_in_the_product,
        &hardware_groups_of_the_sessions,
    ) {
        return Err(RefusalBeforeTheRun {
            lines: vec![(
                "stopped",
                format!(
                    "clause=product_session_without_a_hardware_group session={} way_out=换一个--product",
                    field_value(&session)
                ),
            )],
            exit_code: EXIT_USAGE,
        });
    }
    let discrete_gpus_of_the_every_card_arm = discrete_gpus_named_for_the_every_card_arm(
        inputs.cards,
        inputs.cards_of_the_every_card_arm,
    )
    .map_err(|reason| RefusalBeforeTheRun {
        lines: vec![(
            "stopped",
            format!(
                "clause=cards_option_exceeds_the_cards_handed_over reason={} way_out=--cards改小或不给",
                field_value(&reason)
            ),
        )],
        exit_code: EXIT_USAGE,
    })?;
    let mut lines: Vec<(&'static str, String)> = Vec::new();
    let mut indices_of_the_arms_run: Vec<usize> = Vec::new();
    for (index, (name, compute)) in inputs.arms_requested.iter().enumerate() {
        match reason_the_arm_is_not_run(
            *compute,
            discrete_gpus_of_the_arm(
                *compute,
                &inputs.cards.discrete_gpus,
                &discrete_gpus_of_the_every_card_arm,
            )
            .len(),
        ) {
            Some(reason) => lines.push((
                "not_run",
                format!("tier={} arm={name} reason={reason}", tier.name()),
            )),
            None => indices_of_the_arms_run.push(index),
        }
    }
    if inputs.cards.discrete_gpus.is_empty() {
        for index in &indices_of_the_arms_run {
            lines.push((
                "not_run",
                format!(
                    "tier={} arm={} reason=服务交的卡里没有这一台的独立显卡",
                    tier.name(),
                    inputs.arms_requested[*index].0
                ),
            ));
        }
        return Err(RefusalBeforeTheRun {
            lines,
            exit_code: EXIT_NO_ARM_RAN,
        });
    }
    let is_run = |compute: ComputeOfTheArm| {
        indices_of_the_arms_run
            .iter()
            .any(|index| inputs.arms_requested[*index].1 == compute)
    };
    let one_card = if is_run(ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress) {
        inputs.cards.discrete_gpus.first()
    } else {
        None
    };
    let every_card = is_run(ComputeOfTheArm::GpuOnEveryCard)
        .then_some(discrete_gpus_of_the_every_card_arm.as_slice());
    if let Some(gpu) = one_card {
        lines.push((
            "one_card_arm_card",
            one_card_arm_card_fields(inputs.session_number, tier, gpu),
        ));
    }
    if let Some(gpus) = every_card {
        lines.push((
            "every_card_arm_cards",
            every_card_arm_cards_fields(
                inputs.session_number,
                tier,
                inputs.cards_of_the_every_card_arm,
                gpus,
            ),
        ));
    }
    let group = hardware_group_of(inputs.host_hardware, one_card, every_card);
    lines.push((
        "hardware_group",
        group.fields_of_the_line(inputs.session_number, tier),
    ));
    hardware_groups_of_the_sessions
        .insert(inputs.session_number.to_string(), group.identifier.clone());
    let mut repeats_done_of_each_group: BTreeMap<String, BTreeMap<u32, String>> =
        lines_already_in_the_product
            .iter()
            .filter(|(name, _fields)| name == "hardware_group")
            .filter(|(_name, fields)| fields.get("tier").map(String::as_str) == Some(tier.name()))
            .filter_map(|(_name, fields)| fields.get("group"))
            .chain(std::iter::once(&group.identifier))
            .map(|identifier| (identifier.clone(), BTreeMap::new()))
            .collect();
    let mut repeats_done_in_the_product: BTreeMap<u32, String> = BTreeMap::new();
    for (_name, fields) in lines_already_in_the_product
        .iter()
        .filter(|(name, fields)| {
            name == inputs.run.kind_of_the_repeat_done_line()
                && inputs.run.is_of_this_setting(fields)
        })
    {
        let (Some(repeat), Some(session)) = (
            fields
                .get("repeat")
                .and_then(|repeat| repeat.parse::<u32>().ok()),
            fields.get("session"),
        ) else {
            continue;
        };
        repeats_done_in_the_product.insert(repeat, session.clone());
        if let Some(identifier) = hardware_groups_of_the_sessions.get(session) {
            repeats_done_of_each_group
                .entry(identifier.clone())
                .or_default()
                .insert(repeat, session.clone());
        }
    }
    let repeats_of_the_group_done = repeats_done_of_each_group
        .get(&group.identifier)
        .cloned()
        .unwrap_or_default();
    let run_lines_of_the_group: Vec<BTreeMap<String, String>> = lines_already_in_the_product
        .iter()
        .filter(|(name, fields)| {
            name == inputs.run.kind_of_the_run_line()
                && inputs.run.is_of_this_setting(fields)
                && fields.get("void").map(String::as_str) == Some("no")
                && fields
                    .get("repeat")
                    .and_then(|repeat| repeat.parse::<u32>().ok())
                    .and_then(|repeat| repeats_of_the_group_done.get(&repeat))
                    == fields.get("session")
        })
        .map(|(_name, fields)| fields.clone())
        .collect();
    let next_repeat = inputs.run.next_repeat(lines_already_in_the_product);
    let groups_in_the_product = repeats_done_of_each_group
        .iter()
        .map(|(identifier, done)| format!("{identifier}:{}", done.len()))
        .collect::<Vec<String>>()
        .join(",");
    lines.push((
        "resume",
        format!(
            "{} repeats_already_done={} next_repeat={next_repeat} group={} repeats_of_the_group_already_done={} groups_in_the_product={groups_in_the_product}",
            inputs.run.leading_fields_of_the_resume_line(),
            repeats_done_in_the_product.len(),
            group.identifier,
            repeats_of_the_group_done.len()
        ),
    ));
    Ok(PlanOfTheRun {
        lines,
        indices_of_the_arms_run,
        group,
        discrete_gpus_of_the_every_card_arm,
        repeats_of_the_group_done,
        run_lines_of_the_group,
        next_repeat,
    })
}

/// 交替跑与并发跑取卡之后、开跑之前调：照 [`plan_of_the_run_after_the_cards`] 打要打的行、交回算出的那一份；拒绝时打拒绝的行、
/// 交回它的退出码（退 2 的另把那一行与用法打到标准错误）。产物由拥有它的调用方收尾（[`ProductWriter::finish`]）。
pub(super) fn emit_the_plan_of_the_run_or_the_refusal(
    product: &mut ProductWriter,
    inputs: &InputsBeforeTheRun<'_>,
) -> Result<PlanOfTheRun, i32> {
    match plan_of_the_run_after_the_cards(inputs) {
        Ok(plan) => {
            for (kind, fields) in &plan.lines {
                product.emit(kind, fields);
            }
            Ok(plan)
        }
        Err(refusal) => {
            for (kind, fields) in &refusal.lines {
                product.emit(kind, fields);
                if refusal.exit_code == EXIT_USAGE {
                    eprintln!("{kind} {fields}\n{USAGE}");
                }
            }
            Err(refusal.exit_code)
        }
    }
}

/// 判读行（`judgement_of_a_lockstep_tier`、`judgement_of_a_concurrent_tier` 交回的与 `name=verdict`）带上这一趟的组标识：
/// 收尾判读只判这一趟所在的那一组（登记第十二节修订第 16 条「判读怎么分组」第 5 条）。
pub(super) fn fields_with_the_group(fields: &str, group: &HardwareGroup) -> String {
    format!("{fields} group={}", group.identifier)
}

/// 这条臂开的卡从哪一份里取：全部卡臂是 `--cards` 挑过的那一份，点名一张卡臂（取头一张）与 CPU 臂是交来的全部。
pub(super) fn discrete_gpus_of_the_arm<'cards>(
    compute: ComputeOfTheArm,
    handed_over: &'cards [DiscreteGpuSeenByThisProcess],
    of_the_every_card_arm: &'cards [DiscreteGpuSeenByThisProcess],
) -> &'cards [DiscreteGpuSeenByThisProcess] {
    match compute {
        ComputeOfTheArm::Cpu | ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress => handed_over,
        ComputeOfTheArm::GpuOnEveryCard => of_the_every_card_arm,
    }
}

/// 打 `name=gpus` 一行：枚举过没有、几张、各自的 PCI 地址与型号。
pub(super) fn emit_the_gpus(
    product: &mut ProductWriter,
    is_enumerated: bool,
    discrete_gpus: &[DiscreteGpuSeenByThisProcess],
) {
    product.emit(
        "gpus",
        &format!(
            "enumerated={is_enumerated} count={} pci_bus_addresses={} models={}",
            discrete_gpus.len(),
            field_value(
                &discrete_gpus
                    .iter()
                    .map(|gpu| gpu.pci_bus_address.as_str())
                    .collect::<Vec<&str>>()
                    .join(",")
            ),
            field_value(
                &discrete_gpus
                    .iter()
                    .map(|gpu| gpu.model.as_str())
                    .collect::<Vec<&str>>()
                    .join(",")
            )
        ),
    );
}

#[cfg(test)]
mod tests;
