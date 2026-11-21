//! 子命令 judge-trend：量 T 的分类与三档产物的 crates 版本相同。

use super::*;

// ---- 子命令 judge-trend（登记第六节量 T；停机条款第 2 条：三档产物的 crates 版本相同）----

/// 三档产物里读回的 `saving_per_batch`，按（对比的臂、组标识、档）分（登记第十二节修订第 16 条「判读怎么分组」第 6 条）。
type SavingsPerBatchByArmGroupAndTier = BTreeMap<(String, String, UnitAreaTier), Vec<f64>>;

/// 量 T 的判定行（`name=trend` 那一行 `name=` 之后的字段），每个（对比的臂、组标识）一行：同一组三档都有才判它的走向，
/// 三档凑不齐同一组的打 `trend=not_judged` 与它有几档，不跨组拼三档。
fn trend_lines_by_arm_and_group(
    savings_per_batch: &SavingsPerBatchByArmGroupAndTier,
) -> Vec<String> {
    let arms_and_groups: BTreeSet<(String, String)> = savings_per_batch
        .keys()
        .map(|(against, group, _tier)| (against.clone(), group.clone()))
        .collect();
    arms_and_groups
        .into_iter()
        .map(|(against, group)| {
            let by_tier: Vec<Vec<f64>> = UnitAreaTier::EVERY_TIER
                .iter()
                .filter_map(|tier| {
                    savings_per_batch
                        .get(&(against.clone(), group.clone(), *tier))
                        .cloned()
                })
                .collect();
            let tiers_with_a_saving = by_tier.len();
            match <[Vec<f64>; 3]>::try_from(by_tier) {
                Ok(by_tier) => format!(
                    "against={against} group={group} tiers_with_a_saving={tiers_with_a_saving} trend={}",
                    trend_over_the_tiers(&by_tier).name()
                ),
                Err(_fewer_than_three_tiers) => format!(
                    "against={against} group={group} tiers_with_a_saving={tiers_with_a_saving} trend=not_judged"
                ),
            }
        })
        .collect()
}

pub(super) fn judge_trend_command(product_path: &Path, lockstep_products: &[PathBuf]) -> i32 {
    let mut product = match ProductWriter::appending_to(product_path) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let mut crates_versions: BTreeSet<String> = BTreeSet::new();
    let mut savings_per_batch: SavingsPerBatchByArmGroupAndTier = BTreeMap::new();
    for lockstep_product in lockstep_products {
        let lines = match result_lines_already_in_the_product(lockstep_product) {
            Ok(lines) => lines,
            Err(reason) => {
                product.emit(
                    "input_unreadable",
                    &format!("reason={}", field_value(&reason)),
                );
                product.finish();
                return EXIT_RUN_CONDITION_NOT_MET;
            }
        };
        for (name, fields) in &lines {
            match name.as_str() {
                "crates_version" => crates_versions.extend(fields.get("combined_sha256").cloned()),
                "saving_per_batch" => {
                    let tier = fields
                        .get("tier")
                        .and_then(|tier| UnitAreaTier::of_the_name(tier));
                    let values: Option<Vec<f64>> = fields.get("values").map(|values| {
                        values
                            .split(',')
                            .filter_map(|value| value.parse::<f64>().ok())
                            .collect()
                    });
                    if let (Some(tier), Some(against), Some(group), Some(values)) =
                        (tier, fields.get("against"), fields.get("group"), values)
                    {
                        savings_per_batch.insert((against.clone(), group.clone(), tier), values);
                    }
                }
                _other_result_lines => {}
            }
        }
    }
    product.emit(
        "crates_versions_of_the_inputs",
        &format!("distinct={}", crates_versions.len()),
    );
    if crates_versions.len() != 1 {
        product.emit(
            "stopped",
            "clause=crates_changed_between_the_commands_of_the_segment",
        );
        product.finish();
        return EXIT_STOPPED;
    }
    for fields in trend_lines_by_arm_and_group(&savings_per_batch) {
        product.emit("trend", &fields);
    }
    product.finish();
    EXIT_SUCCESS
}

#[cfg(test)]
mod tests;
