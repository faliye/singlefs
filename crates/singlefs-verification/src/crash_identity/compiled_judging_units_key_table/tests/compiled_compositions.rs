//! 编进去的组成清单行：单元的名字不带行号、同名的按次序加序号，按行号起名的单元名字认得出。

use super::*;

/// 编进去的组成清单换成组成清单行：单元写种类、限定名与文件，不带行号；同一份文件里种类与限定名都相同的几个按出现次序另加序号。
#[test]
fn the_compiled_composition_names_units_without_line_numbers_and_tells_repeated_units_apart() {
    let unit = |qualified_name: &str, location: &str, byte: u8| {
        singlefs_unit_graph::entry_groups::CompositionUnit {
            kind: "use".to_string(),
            qualified_name: qualified_name.to_string(),
            location: location.to_string(),
            fingerprint: [byte; DIGEST_BYTES],
        }
    };
    let group = ReadBackEntryGroup {
        group_name: "gpu_judge".to_string(),
        verdict: ReadBackEntryGroupVerdict::Keyed {
            key: [7; DIGEST_BYTES],
        },
        entries_not_compiled: Vec::new(),
        composition: vec![
            unit("crate_a::module_a", "crates/a/src/lib.rs:3-4", 1),
            unit("crate_a::module_a", "crates/a/src/lib.rs:9-9", 2),
        ],
    };
    let names: Vec<String> = composition_of_the_compiled_group(&group)
        .into_iter()
        .map(|composed| composed.unit)
        .collect();
    assert_eq!(
        names,
        [
            "use crate_a::module_a @ crates/a/src/lib.rs",
            "use crate_a::module_a @ crates/a/src/lib.rs #2",
        ]
    );
}

/// 组成清单行里一个单元的名字（[`composition_of_the_compiled_group`] 的写法）认不认得出：`<种类> <限定名> @ <文件>`，同一份文件里种类与限定名
/// 都相同的另接 ` #<第几个>`（从 2 起）；种类与文件里没有空白与 `@`，限定名里每个 `@` 后面紧跟至少一位数字（按行号起名的单元的行号）。
/// [`unit_name_without_line_numbers`] 只去 `@` 后面紧跟的数字、名字与文件之间的 ` @ ` 后面是空格不动，两样都靠这个写法。
///
/// # Errors
/// 认不出：交回哪一处不对。认得出交回限定名里带不带行号。
fn line_numbers_in_a_composition_unit_name(name: &str) -> Result<bool, String> {
    let occurrence_suffix = name.rsplit_once(" #").filter(|(_, number)| {
        !number.is_empty() && number.chars().all(|character| character.is_ascii_digit())
    });
    if let Some((_, number)) = occurrence_suffix {
        match number.parse::<usize>() {
            Ok(occurrence) if occurrence >= 2 => {}
            Ok(_) | Err(_) => return Err(format!("同名序号 #{number} 不是从 2 起的数")),
        }
    }
    let without_the_occurrence = occurrence_suffix.map_or(name, |(before, _)| before);
    let parts: Vec<&str> = without_the_occurrence.split(" @ ").collect();
    let [kind_and_qualified_name, file] = parts.as_slice() else {
        return Err("名字与文件之间的 ` @ ` 不是恰好一处".to_string());
    };
    let Some((kind, qualified_name)) = kind_and_qualified_name.split_once(' ') else {
        return Err("种类与限定名之间没有空格".to_string());
    };
    if kind.is_empty() || qualified_name.is_empty() || file.is_empty() {
        return Err("种类、限定名或文件有一样是空的".to_string());
    }
    if kind.contains('@') || file.contains('@') || file.contains(char::is_whitespace) {
        return Err("种类或文件里有 `@`，或文件里有空白".to_string());
    }
    let is_each_at_sign_followed_by_a_digit =
        qualified_name.match_indices('@').all(|(position, _)| {
            qualified_name[position + 1..].starts_with(|character: char| character.is_ascii_digit())
        });
    if !is_each_at_sign_followed_by_a_digit {
        return Err("限定名里有 `@` 后面紧跟的不是数字".to_string());
    }
    Ok(qualified_name.contains('@'))
}

/// 场景：编进二进制的判器键表里每一组的组成清单（[`compiled_judging_units_keys`] 经 [`composition_of_the_compiled_group`]；构建时编进来，不读活树）。
/// 预期：每个单元的名字都认得出（[`line_numbers_in_a_composition_unit_name`]），至少一个限定名带行号（不然去行号那一步在真清单上没有对象）；
/// 去掉行号（[`unit_name_without_line_numbers`]）之后 ` @ ` 后面的文件与同名序号原样。单元图哪天换了行号的写法（例：`use@L108`）或名字与文件之间的隔法，这条红。
/// 对照（认法自己会红）：`@` 后面接字母的、` @ ` 少了空格的、同名序号从 1 起的现造名字认不出；带行号与带同名序号的现造名字认得出。
/// 判别力（`crates/mutations.tsv`）：组成清单行里名字与文件之间的 ` @ ` 去掉空格，这条红；同名序号从 1 起（每个单元都接 ` #1`），这条红。
#[test]
fn unit_names_in_the_compiled_compositions_keep_line_numbers_as_digits_after_an_at_sign_and_the_file_after_a_spaced_at_sign(
) {
    for (name, expected) in [
        ("use crate_a::use@108 @ crates/a/src/lib.rs", Ok(true)),
        ("fn crate_a::sample @ crates/a/src/lib.rs #2", Ok(false)),
        (
            "use crate_a::use@L108 @ crates/a/src/lib.rs",
            Err("限定名里有 `@` 后面紧跟的不是数字".to_string()),
        ),
        (
            "use crate_a::use@108@crates/a/src/lib.rs",
            Err("名字与文件之间的 ` @ ` 不是恰好一处".to_string()),
        ),
        (
            "fn crate_a::sample @ crates/a/src/lib.rs #1",
            Err("同名序号 #1 不是从 2 起的数".to_string()),
        ),
    ] {
        assert_eq!(
            line_numbers_in_a_composition_unit_name(name),
            expected,
            "对照：{name}"
        );
    }
    let mut unrecognized = Vec::new();
    let mut names_with_line_numbers = 0_usize;
    let mut names_checked = 0_usize;
    let file_and_occurrence =
        |name: &str| name.split_once(" @ ").map(|(_, after)| after.to_string());
    for group in &compiled_judging_units_keys().groups {
        for composed in composition_of_the_compiled_group(group) {
            names_checked += 1;
            match line_numbers_in_a_composition_unit_name(&composed.unit) {
                Ok(has_line_numbers) => {
                    names_with_line_numbers += usize::from(has_line_numbers);
                }
                Err(reason) => {
                    unrecognized.push(format!("{}：{}：{reason}", group.group_name, composed.unit))
                }
            }
            if file_and_occurrence(&unit_name_without_line_numbers(&composed.unit))
                != file_and_occurrence(&composed.unit)
            {
                unrecognized.push(format!(
                    "{}：{}：去掉行号之后文件或同名序号变了",
                    group.group_name, composed.unit
                ));
            }
        }
    }
    assert!(
        unrecognized.is_empty(),
        "编进去的组成清单里有认不出的单元名（共 {names_checked} 个）：{unrecognized:#?}"
    );
    assert!(
            names_with_line_numbers > 0,
            "编进去的组成清单里没有一个限定名带行号（共 {names_checked} 个）：去行号那一步在真清单上没对象"
        );
}

/// 算键与存取判定的三个包（函数单元图、设备库、产品 GPU 对拍的键）不进任何键：编进去的判法单元键表里，进键的那几组（观察者那几组、
/// 判器引到的被检测产品代码那一组）的组成清单一个单元都不落在这三个包里（落进去了，改这三个包就会作废已判的数据）。
/// 判别力（`crates/mutations.tsv`）：这三个包的目录前缀写成别的目录（一个单元都认不出落在里面），对照那一格判红。
#[test]
fn no_keyed_group_holds_a_unit_of_the_crates_that_compute_keys_or_store_verdicts() {
    let crates_kept_out_of_every_key = [
        "crates/singlefs-unit-graph/",
        "crates/singlefs-device-library/",
        "crates/singlefs-background-gpu-pairing-keys/",
    ];
    let lands_in_a_crate_kept_out = |location: &str| {
        crates_kept_out_of_every_key
            .iter()
            .any(|directory| location.starts_with(directory))
    };
    assert!(
        lands_in_a_crate_kept_out("crates/singlefs-unit-graph/src/closure.rs:10-20")
            && !lands_in_a_crate_kept_out("crates/singlefs-core/src/lib.rs:1-2"),
        "对照：落在三个包里的认得出，落在产品包里的不算"
    );
    let groups_in_some_key: Vec<&ReadBackEntryGroup> =
        compiled_judging_units_keys().groups.iter().collect();
    assert!(
        groups_in_some_key
            .iter()
            .any(|group| group.group_name == JUDGED_PRODUCT_CODE_GROUP_NAME),
        "前提：判器引到的被检测产品代码那一组编进来了"
    );
    let units_kept_out: Vec<String> = groups_in_some_key
        .iter()
        .flat_map(|group| {
            group
                .composition
                .iter()
                .filter(|unit| lands_in_a_crate_kept_out(&unit.location))
                .map(move |unit| {
                    format!(
                        "{}：{} {}",
                        group.group_name, unit.qualified_name, unit.location
                    )
                })
        })
        .collect();
    assert!(
        units_kept_out.is_empty(),
        "进键的组里有落在算键与存取判定那三个包里的单元：{units_kept_out:#?}"
    );
}
