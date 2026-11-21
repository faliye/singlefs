//! 进键的判法单元的类型：哪一组判法单元、组成清单的一行、判法单元键，库里那一把与这一趟比出变了的单元，与拒算。

use super::*;

// ============================== 进键的判法单元：观察者与判器引到的被检测产品代码 ==============================

/// 哪一组判法单元：入口登记在 `crate::judging_units_entries`，构建脚本收出闭包、算键编进二进制，跑时只经 [`judging_units_key`] 取。
/// 判器（池级 checker、记录核对器、CPU 参照判器、GPU 判器）、核对、观察者结论拼块与对拍的比法自己的代码不进任何键；
/// 进键的是它们引到的被检测产品代码（[`Self::JudgedProductCode`]、[`Self::ObserverCode`] 那一组只留产品包的单元）。录入键不是判法单元：
/// 它只由录下的写流摘要与场景标识组成（`crate::crash_library::recording_key_of_the_scenario_and_the_inputs`）。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JudgingUnits {
    /// 一个观察者：（观察者代码文件，观察者名）是它的判定项标识；它那一组只留产品包的单元（观察者引到的被检测产品代码），
    /// 键进它的版本，观察者自己的代码不进。
    ObserverCode {
        observer_code_file: String,
        observer_name: String,
    },
    /// 判器引到的被检测产品代码（`crate::judging_units_entries::JUDGED_PRODUCT_CODE_ENTRIES` 那几个入口收的闭包只留产品包的单元）：
    /// 进判器判出的三种结论的版本与对拍结论的键，改产品恢复里一处，已判的块现判。
    JudgedProductCode,
}

impl JudgingUnits {
    /// 行与组成清单里的种类名（不带用例名、文件与观察者名）。
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::ObserverCode { .. } => "observer_code",
            Self::JudgedProductCode => "judged_product_code",
        }
    }

    /// 编进二进制的那份表里这一组的组名（只当索引）：观察者是
    /// `crate::judging_units_entries::observer_group_name` 拼的那一个，判器引到的被检测产品代码是
    /// `crate::judging_units_entries::JUDGED_PRODUCT_CODE_GROUP_NAME`。
    #[must_use]
    pub fn group_name(&self) -> String {
        match self {
            Self::ObserverCode {
                observer_code_file,
                observer_name,
            } => {
                crate::judging_units_entries::observer_group_name(observer_code_file, observer_name)
            }
            Self::JudgedProductCode => {
                crate::judging_units_entries::JUDGED_PRODUCT_CODE_GROUP_NAME.to_string()
            }
        }
    }
}

/// 一组判法单元里的一个单元与它的指纹：组成清单的一行，随键落盘，库对不上时照它点名变了的单元。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JudgingUnitFingerprint {
    pub unit: String,
    pub fingerprint: [u8; DIGEST_BYTES],
}

/// 一组判法单元的键：只按内容取值（这一组判法单元的复用键），不带路径、名字、手写版本号。组成清单随键落盘
/// （判定库的组成清单表），作废时点名变了的单元。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgingUnitsKey {
    fingerprint: [u8; DIGEST_BYTES],
    composition: Vec<JudgingUnitFingerprint>,
}

impl JudgingUnitsKey {
    /// 从指纹与组成清单建（编进二进制的表读回的、库里读回的组成清单、测试里造的）。
    #[must_use]
    pub fn of_the_parts(
        fingerprint: [u8; DIGEST_BYTES],
        composition: Vec<JudgingUnitFingerprint>,
    ) -> Self {
        Self {
            fingerprint,
            composition,
        }
    }

    #[must_use]
    pub fn fingerprint(&self) -> &[u8; DIGEST_BYTES] {
        &self.fingerprint
    }

    #[must_use]
    pub fn composition(&self) -> &[JudgingUnitFingerprint] {
        &self.composition
    }

    /// 组成清单的文字：一个单元一行 `<单元>\t<指纹十六进制>`，按单元排。
    #[must_use]
    pub fn composition_text(&self) -> String {
        let mut sorted = self.composition.clone();
        sorted.sort();
        sorted
            .iter()
            .map(|unit| format!("{}\t{}\n", unit.unit, hexadecimal_text(&unit.fingerprint)))
            .collect()
    }

    /// [`Self::composition_text`] 的逆；有一行认不出交回 `None`。
    #[must_use]
    pub fn composition_from_text(text: &str) -> Option<Vec<JudgingUnitFingerprint>> {
        text.lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let (unit, hexadecimal) = line.split_once('\t')?;
                let bytes: Vec<u8> = (0..hexadecimal.len())
                    .step_by(2)
                    .map(|start| {
                        hexadecimal
                            .get(start..start + 2)
                            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                    })
                    .collect::<Option<Vec<u8>>>()?;
                Some(JudgingUnitFingerprint {
                    unit: unit.to_string(),
                    fingerprint: bytes.try_into().ok()?,
                })
            })
            .collect()
    }
}

/// 库里存着的那一把与这一趟的组成清单比：两边指纹不同、或只有一边有的单元，按单元排、不重。两边一样是空的。
/// 两边各取一个配成一对、不报的：先配名字与指纹都相同的，再配去掉限定名里的行号（[`unit_name_without_line_numbers`]）之后名字相同、
/// 指纹也相同的——按行号起名的单元（`use@<行>`、`<宏>!@<行>`、WGSL 的 `const_assert@<行>` 这类）只挪了行，键不看名字与行号、不变，
/// 不算变了。配不上对的照原名报：挪了行又改了内容的，旧名新名各报一个。
#[must_use]
pub fn units_changed_between(
    stored: &[JudgingUnitFingerprint],
    now: &[JudgingUnitFingerprint],
) -> Vec<String> {
    let mut stored_unpaired: Vec<&JudgingUnitFingerprint> = stored.iter().collect();
    let mut now_unpaired: Vec<&JudgingUnitFingerprint> = now.iter().collect();
    pair_off_units_with_the_same_compared_name_and_fingerprint(
        &mut stored_unpaired,
        &mut now_unpaired,
        &str::to_string,
    );
    pair_off_units_with_the_same_compared_name_and_fingerprint(
        &mut stored_unpaired,
        &mut now_unpaired,
        &unit_name_without_line_numbers,
    );
    stored_unpaired
        .iter()
        .chain(&now_unpaired)
        .map(|unit| unit.unit.clone())
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect()
}

/// 两边还没配上对的单元里，`compared_name_of` 取出的名字相同、指纹也相同的一边一个配成一对，从两边各拿掉；
/// 同一个名字与指纹在两边各有几个，就配上两边里少的那么多对。
fn pair_off_units_with_the_same_compared_name_and_fingerprint(
    stored_unpaired: &mut Vec<&JudgingUnitFingerprint>,
    now_unpaired: &mut Vec<&JudgingUnitFingerprint>,
    compared_name_of: &dyn Fn(&str) -> String,
) {
    let mut now_positions_of_each_name_and_fingerprint: BTreeMap<
        (String, [u8; DIGEST_BYTES]),
        Vec<usize>,
    > = BTreeMap::new();
    for (position, unit) in now_unpaired.iter().enumerate() {
        now_positions_of_each_name_and_fingerprint
            .entry((compared_name_of(unit.unit.as_str()), unit.fingerprint))
            .or_default()
            .push(position);
    }
    let mut is_now_unit_paired = vec![false; now_unpaired.len()];
    stored_unpaired.retain(|unit| {
        let paired_now_position = now_positions_of_each_name_and_fingerprint
            .get_mut(&(compared_name_of(unit.unit.as_str()), unit.fingerprint))
            .and_then(Vec::pop);
        match paired_now_position {
            Some(position) => {
                is_now_unit_paired[position] = true;
                false
            }
            None => true,
        }
    });
    *now_unpaired = now_unpaired
        .iter()
        .zip(&is_now_unit_paired)
        .filter(|(_unit, is_paired)| !**is_paired)
        .map(|(unit, _is_paired)| *unit)
        .collect();
}

/// 组成清单里一个单元的名字去掉限定名里的行号：`@` 后面紧跟的那串数字去掉，别的字原样。按行号起名的单元
/// （`singlefs-unit-graph` 切单元时把所在的行号拼进限定名：`use@<行>`、`extern@<行>`、`<宏>!@<行>`、WGSL 的 `const_assert@<行>` 这类）
/// 只挪了行，两边去掉之后同名；名字与文件之间的 ` @ ` 后面是空格，不动。
pub(super) fn unit_name_without_line_numbers(unit: &str) -> String {
    let mut name = String::with_capacity(unit.len());
    let mut follows_an_at_sign = false;
    for character in unit.chars() {
        if follows_an_at_sign && character.is_ascii_digit() {
            continue;
        }
        follows_an_at_sign = character == '@';
        name.push(character);
    }
    name
}

/// 库对不上时红句里点名变了的单元那一截：以「变了的单元：」起头，接逗号隔开的单元清单；组成清单两边都空、比不出时写明。
#[must_use]
pub fn changed_units_sentence(changed_units: &[String]) -> String {
    if changed_units.is_empty() {
        "变了的单元：（组成清单比不出：库里那一把或这一趟的组成清单是空表）".to_string()
    } else {
        format!("变了的单元：{}", changed_units.join("，"))
    }
}

/// 一组判法单元的键算不出：哪一组、什么形态。调用方照判、不入库，计入拒算（不退回整文件或整表指纹，不跳过）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgingUnitsKeyRefused {
    /// 编进二进制的表里没有这一组（观察者没登记），或这一组的入口在构建脚本收它时照的特性下一个都不编进来。
    NotComputedAtTheBuild { units: JudgingUnits, reason: String },
    /// 编译时这一组的闭包里有认不出的形态（`singlefs-unit-graph` 的拒算行，点名单元与形态）。
    RefusedAtTheBuild {
        units: JudgingUnits,
        refusal_lines: Vec<String>,
    },
}

impl JudgingUnitsKeyRefused {
    /// 拒算的是哪一组。
    #[must_use]
    pub fn units(&self) -> &JudgingUnits {
        match self {
            Self::NotComputedAtTheBuild { units, .. } | Self::RefusedAtTheBuild { units, .. } => {
                units
            }
        }
    }

    /// 拒算的形态名（行里 `form=` 那一截）。
    #[must_use]
    pub fn form_name(&self) -> &'static str {
        match self {
            Self::NotComputedAtTheBuild { .. } => "not_computed_at_the_build",
            Self::RefusedAtTheBuild { .. } => "refused_at_the_build",
        }
    }

    /// 红句：哪一组、什么形态、为什么；闭包里认不出的点名拒算行。
    #[must_use]
    pub fn sentence(&self) -> String {
        match self {
            Self::NotComputedAtTheBuild { units, reason } => format!(
                "判法单元 {} 拒算（{}）：{reason}",
                units.group_name(),
                self.form_name()
            ),
            Self::RefusedAtTheBuild {
                units,
                refusal_lines,
            } => format!(
                "判法单元 {} 拒算（{}）：闭包里有认不出的形态：{}",
                units.group_name(),
                self.form_name(),
                refusal_lines.join(" | ").replace('\t', " ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 组成清单写成文字再读回逐行相同；两把清单比下来点名指纹不同与只有一边有的单元；红句以「变了的单元：」起头、接清单，
    /// 两边都空时写明比不出；编译时拒算的那一种红句带拒算行。
    #[test]
    fn the_composition_reads_back_and_the_red_sentence_names_the_changed_units() {
        let unit = |name: &str, byte: u8| JudgingUnitFingerprint {
            unit: name.to_string(),
            fingerprint: [byte; DIGEST_BYTES],
        };
        let stored = JudgingUnitsKey::of_the_parts(
            [1; DIGEST_BYTES],
            vec![unit("recover", 1), unit("check_unit", 2), unit("walk", 3)],
        );
        assert_eq!(
            JudgingUnitsKey::composition_from_text(&stored.composition_text()),
            Some({
                let mut sorted = stored.composition().to_vec();
                sorted.sort();
                sorted
            }),
            "组成清单读得回"
        );
        assert_eq!(
            JudgingUnitsKey::composition_from_text("recover\tzz\n"),
            None
        );
        let now = vec![unit("recover", 1), unit("check_unit", 9), unit("crc32", 4)];
        let changed = units_changed_between(stored.composition(), &now);
        assert_eq!(changed, vec!["check_unit", "crc32", "walk"]);
        let sentence = changed_units_sentence(&changed);
        assert!(
            sentence.starts_with("变了的单元：") && sentence.contains("check_unit，crc32，walk"),
            "{sentence}"
        );
        assert!(
            changed_units_sentence(&units_changed_between(&[], &[])).starts_with("变了的单元："),
            "组成清单是空表也照打这一截"
        );
        let refused = JudgingUnitsKeyRefused::RefusedAtTheBuild {
            units: JudgingUnits::JudgedProductCode,
            refusal_lines: vec![
                "refused\tcurrent-exe\tcrates/a/src/lib.rs:3\tcurrent_exe".to_string()
            ],
        };
        assert!(
            refused.sentence().contains("refused_at_the_build")
                && refused
                    .sentence()
                    .contains("current-exe crates/a/src/lib.rs:3"),
            "{}",
            refused.sentence()
        );
    }

    /// 场景：库里那一把与这一趟的组成清单（组成清单行的写法 `<种类> <限定名> @ <文件>`），按行号起名的单元挪了行。
    /// 只挪行、指纹不变的四种：一条 `use` 往下挪一行；条目级宏调用挪了行；同一份文件里两条 `use` 一起往下挪两行，挪完一条落在另一条
    /// 原来的行号上（同一个名字两边指纹不同，要按去掉行号之后的名字与指纹配对才配得上）；WGSL 的 `const_assert` 挪了行。
    /// 预期：只挪行的这几种比下来一个都不报；不按行号起名的单元照旧按名字比，没变的不报。
    /// 另一份：一条 `use` 挪了行又改了内容（指纹变），照报，旧名新名各报一个；另有一个不按行号起名、指纹变了的单元照报。
    /// 判别力（`crates/mutations.tsv`）：去行号那一步不去 `@` 后面的数字，只挪行的照旧以旧名新名各报一次，这条红；
    /// 去掉行号之后配对不看指纹（只按名字配），挪了行又改了内容的旧名新名与指纹变了的单元都配上对、一个都不报，这条红。
    #[test]
    fn units_named_by_their_line_that_only_moved_are_not_named_as_changed_and_edited_ones_are() {
        let unit = |name: &str, byte: u8| JudgingUnitFingerprint {
            unit: name.to_string(),
            fingerprint: [byte; DIGEST_BYTES],
        };
        let history = "crates/singlefs-verification/src/history.rs";
        let kernel = "crates/singlefs-verification/src/crash_judge_checker.wgsl";
        let stored = [
            unit(&format!("use singlefs_verification::history::use@108 @ {history}"), 1),
            unit(
                &format!(
                    "item_macro singlefs_verification::history::enum_listing_every_member!@134 @ {history}"
                ),
                2,
            ),
            unit(&format!("use singlefs_verification::history::use@200 @ {history}"), 3),
            unit(&format!("use singlefs_verification::history::use@202 @ {history}"), 4),
            unit(
                &format!("wgsl_const_assert singlefs_verification::crash_judge_checker.wgsl::const_assert@285 @ {kernel}"),
                5,
            ),
            unit(
                &format!("fn singlefs_verification::history::small_device_unit_area_start_slot @ {history}"),
                6,
            ),
        ];
        let only_moved = [
            unit(&format!("use singlefs_verification::history::use@109 @ {history}"), 1),
            unit(
                &format!(
                    "item_macro singlefs_verification::history::enum_listing_every_member!@135 @ {history}"
                ),
                2,
            ),
            unit(&format!("use singlefs_verification::history::use@202 @ {history}"), 3),
            unit(&format!("use singlefs_verification::history::use@204 @ {history}"), 4),
            unit(
                &format!("wgsl_const_assert singlefs_verification::crash_judge_checker.wgsl::const_assert@286 @ {kernel}"),
                5,
            ),
            unit(
                &format!("fn singlefs_verification::history::small_device_unit_area_start_slot @ {history}"),
                6,
            ),
        ];
        assert_eq!(
            units_changed_between(&stored, &only_moved),
            Vec::<String>::new(),
            "只挪行、指纹不变的单元不报"
        );
        let moved_and_edited = [
            unit(&format!("use singlefs_verification::history::use@109 @ {history}"), 9),
            unit(
                &format!(
                    "item_macro singlefs_verification::history::enum_listing_every_member!@135 @ {history}"
                ),
                2,
            ),
            unit(&format!("use singlefs_verification::history::use@202 @ {history}"), 3),
            unit(&format!("use singlefs_verification::history::use@204 @ {history}"), 4),
            unit(
                &format!("wgsl_const_assert singlefs_verification::crash_judge_checker.wgsl::const_assert@286 @ {kernel}"),
                5,
            ),
            unit(
                &format!("fn singlefs_verification::history::small_device_unit_area_start_slot @ {history}"),
                7,
            ),
        ];
        assert_eq!(
            units_changed_between(&stored, &moved_and_edited),
            [
                format!("fn singlefs_verification::history::small_device_unit_area_start_slot @ {history}"),
                format!("use singlefs_verification::history::use@108 @ {history}"),
                format!("use singlefs_verification::history::use@109 @ {history}"),
            ],
            "挪了行又改了内容的旧名新名各报一个，不按行号起名、指纹变了的照报，只挪行的不报"
        );
    }

    /// 场景：库里那一把与这一趟的组成清单里，一条按行号起名的 `use` 删了、另一条往上挪了两行（指纹不变）、又新加一条（指纹是新的）。
    /// 预期：删了的那一条（库里的名字）与新加的那一条（这一趟的名字）各报一个，只挪了行的不报，没变的不按行号起名的单元不报。
    /// 判别力（`crates/mutations.tsv`）：只报库里那一边没配上的单元（新加的单元不报），这条红。
    #[test]
    fn a_unit_named_by_its_line_that_is_removed_or_added_is_named_and_one_that_only_moved_is_not() {
        let unit = |name: &str, byte: u8| JudgingUnitFingerprint {
            unit: name.to_string(),
            fingerprint: [byte; DIGEST_BYTES],
        };
        let history = "crates/singlefs-verification/src/history.rs";
        let stored = [
            unit(&format!("use singlefs_verification::history::use@108 @ {history}"), 1),
            unit(&format!("use singlefs_verification::history::use@120 @ {history}"), 2),
            unit(
                &format!("fn singlefs_verification::history::small_device_unit_area_start_slot @ {history}"),
                6,
            ),
        ];
        let now = [
            unit(&format!("use singlefs_verification::history::use@110 @ {history}"), 2),
            unit(&format!("use singlefs_verification::history::use@140 @ {history}"), 3),
            unit(
                &format!("fn singlefs_verification::history::small_device_unit_area_start_slot @ {history}"),
                6,
            ),
        ];
        assert_eq!(
            units_changed_between(&stored, &now),
            [
                format!("use singlefs_verification::history::use@108 @ {history}"),
                format!("use singlefs_verification::history::use@140 @ {history}"),
            ],
            "删了的与新加的按行号起名的单元各报一个，只挪了行的与没变的不报"
        );
    }

    /// 场景：库里两条内容逐字相同（指纹相同）的条目级宏调用（行 10 与行 20），这一趟只剩一条、落在行 12。
    /// 预期：报一个，报的是库里那两条之一：只挪行的配对按去掉行号的名字与指纹配，两条一模一样，配给哪一条都对，名字归属可以落在内容相同的另一条上；
    /// 个数不多不少（内容相同的两条各配一次，不把这一趟那一条配给两条）。
    /// 判别力（`crates/mutations.tsv`）：配对时不把配上的那一条从这一趟那一边拿掉（这一趟一条配给库里两条，一个都不报），这条红。
    #[test]
    fn units_with_the_same_content_named_by_their_line_pair_off_one_for_one_and_the_name_left_is_one_of_them(
    ) {
        let unit = |name: &str, byte: u8| JudgingUnitFingerprint {
            unit: name.to_string(),
            fingerprint: [byte; DIGEST_BYTES],
        };
        let history = "crates/singlefs-verification/src/history.rs";
        let stored_names = [
            format!("item_macro singlefs_verification::history::enum_listing_every_member!@10 @ {history}"),
            format!("item_macro singlefs_verification::history::enum_listing_every_member!@20 @ {history}"),
        ];
        let stored = [unit(&stored_names[0], 2), unit(&stored_names[1], 2)];
        let now = [unit(
            &format!("item_macro singlefs_verification::history::enum_listing_every_member!@12 @ {history}"),
            2,
        )];
        let changed = units_changed_between(&stored, &now);
        assert!(
            changed.len() == 1 && stored_names.contains(&changed[0]),
            "内容相同的两条剩一条：报一个，是库里那两条之一：{changed:?}"
        );
    }
}
