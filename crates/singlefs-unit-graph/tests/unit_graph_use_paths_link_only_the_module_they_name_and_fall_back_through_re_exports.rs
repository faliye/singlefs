//! R-裸名 use 引进的名字（`src/closure.rs` 的 `candidates_seen_from_the_module_of_the_unit`）：引进它的那一支路径（花括号展开之后）恰好点到候选所在的模块才算点准，
//! 只连那一份；路径点到的模块里没有这个名字的候选（名字是那个模块经 `pub use` 转出来的），落在哪一份看不出，退回全连，不漏转出来的那一份。
//! 漏边的形状：`use crate::prelude::*`，`prelude` 里 `pub use crate::items::*;` 转出 `SHARED_MARK`，根模块又有一个同名的 `SHARED_MARK`。
//! 夹具只切词、不编译；每条用例依次改两处、每改一处跟前一把比键。

mod common_fixture_workspace;

use common_fixture_workspace::{key_of, workspace_of, EntrySpecification, FixtureTree};
use singlefs_unit_graph::rule_switches::RuleSwitches;

const LIBRARY_FILE: &str = "crates/freexport/src/lib.rs";

const LIBRARY: &str = r"pub const SHARED_MARK: u32 = 1;
pub const ROOT_LISTED_MARK: u32 = 2;

pub mod items {
    pub const SHARED_MARK: u32 = 3;
    pub const DEEP_MARK: u32 = 4;
    pub const OTHER_LISTED_MARK: u32 = 5;
}

pub mod prelude {
    pub use crate::items::*;
}

pub mod deep {
    pub const DEEP_MARK: u32 = 6;
    pub mod prelude {
        pub use crate::items::*;
    }
}

pub mod spare {
    const ROOT_LISTED_MARK: u32 = 7;
    const PARENT_MARK: u32 = 8;
    const INNER_MARK: u32 = 9;
    const RELATIVE_MARK: u32 = 10;
}

pub mod through_a_glob {
    use crate::prelude::*;
    pub fn read_mark() -> u32 {
        SHARED_MARK
    }
}

pub mod through_a_named_import {
    use crate::prelude::SHARED_MARK;
    pub fn read_mark() -> u32 {
        SHARED_MARK
    }
}

pub mod through_a_deeper_prelude {
    use crate::deep::prelude::*;
    pub fn read_mark() -> u32 {
        DEEP_MARK
    }
}

pub mod through_braces_at_the_root {
    use crate::{items::OTHER_LISTED_MARK, ROOT_LISTED_MARK};
    pub fn read_mark() -> u32 {
        ROOT_LISTED_MARK + OTHER_LISTED_MARK
    }
}

pub mod through_braces_around_a_prelude {
    use crate::{items::OTHER_LISTED_MARK, prelude::SHARED_MARK};
    pub fn read_mark() -> u32 {
        SHARED_MARK + OTHER_LISTED_MARK
    }
}

pub mod parent_module {
    const PARENT_MARK: u32 = 11;
    pub mod child_module {
        use super::PARENT_MARK;
        pub fn read_mark() -> u32 {
            PARENT_MARK
        }
    }
}

pub mod through_self {
    use self::inner::INNER_MARK;
    pub mod inner {
        pub const INNER_MARK: u32 = 12;
    }
    pub fn read_mark() -> u32 {
        INNER_MARK
    }
}

pub mod through_a_relative_path {
    use layout::RELATIVE_MARK;
    pub mod layout {
        pub const RELATIVE_MARK: u32 = 13;
    }
    pub fn read_mark() -> u32 {
        RELATIVE_MARK
    }
}
";

const RE_EXPORT_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/freexport/Cargo.toml",
        "[package]\nname = \"freexport\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (LIBRARY_FILE, LIBRARY),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyExpectation {
    Changes,
    StaysTheSame,
}

/// 依次把夹具里恰好出现一次的原文换掉（改了不还原），每改一处算一次入口的键、跟前一把比；算不出键就 panic。
fn key_expectations_after_successive_edits(
    entry_path: &'static str,
    edits: &[(&'static str, &'static str)],
) -> Vec<KeyExpectation> {
    let switches = RuleSwitches::every_rule_intact();
    let tree = FixtureTree::materialize("use-paths-through-re-exports", &RE_EXPORT_FIXTURE);
    let entry = EntrySpecification {
        entry_path,
        target: "lib",
        conditions: &[],
    };
    let mut previous_key = key_of(&workspace_of(tree.root(), &switches), &entry, &switches)
        .unwrap_or_else(|reason| panic!("{entry_path} 改前要算得出键：{reason}"));
    edits
        .iter()
        .map(|(original, replacement)| {
            tree.replace_exactly_once(LIBRARY_FILE, original, replacement);
            let key = key_of(&workspace_of(tree.root(), &switches), &entry, &switches)
                .unwrap_or_else(|reason| {
                    panic!("{entry_path} 改了「{original}」之后要算得出键：{reason}")
                });
            let expectation = if key == previous_key {
                KeyExpectation::StaysTheSame
            } else {
                KeyExpectation::Changes
            };
            previous_key = key;
            expectation
        })
        .collect()
}

/// 先改 `items` 里转出来的 `SHARED_MARK`，再改根模块同名的那一份。
const SHARED_MARK_EDITS: [(&str, &str); 2] = [
    (
        "pub const SHARED_MARK: u32 = 3;",
        "pub const SHARED_MARK: u32 = 30;",
    ),
    (
        "pub const SHARED_MARK: u32 = 1;",
        "pub const SHARED_MARK: u32 = 10;",
    ),
];

/// 场景：`use crate::prelude::*`，`prelude` 经 `pub use crate::items::*` 转出 `SHARED_MARK`，根模块又有同名项。预期：两份都连（转出来的那一份不漏）。
#[test]
fn glob_of_a_prelude_links_the_re_exported_item_though_the_root_has_the_same_name() {
    assert_eq!(
        key_expectations_after_successive_edits(
            "freexport::through_a_glob::read_mark",
            &SHARED_MARK_EDITS,
        ),
        [KeyExpectation::Changes, KeyExpectation::Changes],
        "prelude 里没有 SHARED_MARK 的候选，点不准，退回全连"
    );
}

/// 场景：`use crate::prelude::SHARED_MARK`，同样经 `prelude` 转出，根模块有同名项。预期：两份都连。
#[test]
fn named_import_through_a_prelude_links_the_re_exported_item_though_the_root_has_the_same_name() {
    assert_eq!(
        key_expectations_after_successive_edits(
            "freexport::through_a_named_import::read_mark",
            &SHARED_MARK_EDITS,
        ),
        [KeyExpectation::Changes, KeyExpectation::Changes],
        "prelude 里没有 SHARED_MARK 的候选，点不准，退回全连"
    );
}

/// 场景：`use crate::deep::prelude::*`，`deep::prelude` 转出 `items::DEEP_MARK`，路径中间的 `deep` 自己另有同名项。预期：两份都连。
#[test]
fn glob_of_a_deeper_prelude_links_the_re_exported_item_though_a_module_on_the_path_has_the_same_name(
) {
    assert_eq!(
        key_expectations_after_successive_edits(
            "freexport::through_a_deeper_prelude::read_mark",
            &[
                (
                    "pub const DEEP_MARK: u32 = 4;",
                    "pub const DEEP_MARK: u32 = 40;"
                ),
                (
                    "pub const DEEP_MARK: u32 = 6;",
                    "pub const DEEP_MARK: u32 = 60;"
                ),
            ],
        ),
        [KeyExpectation::Changes, KeyExpectation::Changes],
        "路径点到的是 deep::prelude，不是中间的 deep；那里没有候选，退回全连"
    );
}

/// 场景：`use crate::{items::OTHER_LISTED_MARK, ROOT_LISTED_MARK}`，`spare` 里另有私有的同名常量。预期：改 `spare` 键不变，改根模块键变。
#[test]
fn name_directly_in_the_braces_of_a_use_links_the_module_before_the_braces() {
    assert_eq!(
        key_expectations_after_successive_edits(
            "freexport::through_braces_at_the_root::read_mark",
            &[
                (
                    "const ROOT_LISTED_MARK: u32 = 7;",
                    "const ROOT_LISTED_MARK: u32 = 70;"
                ),
                (
                    "pub const ROOT_LISTED_MARK: u32 = 2;",
                    "pub const ROOT_LISTED_MARK: u32 = 20;"
                ),
            ],
        ),
        [KeyExpectation::StaysTheSame, KeyExpectation::Changes],
        "花括号展开之后路径就是 crate：只连根模块那一份"
    );
}

/// 场景：`use crate::{items::OTHER_LISTED_MARK, prelude::SHARED_MARK}`，花括号里一支经 `prelude` 转出、根模块有同名项。预期：两份都连。
#[test]
fn branch_through_a_prelude_inside_braces_links_the_re_exported_item() {
    assert_eq!(
        key_expectations_after_successive_edits(
            "freexport::through_braces_around_a_prelude::read_mark",
            &SHARED_MARK_EDITS,
        ),
        [KeyExpectation::Changes, KeyExpectation::Changes],
        "这一支路径是 crate::prelude，那里没有候选，退回全连"
    );
}

/// 场景：`use super::PARENT_MARK`，`spare` 里另有私有的同名常量。预期：改 `spare` 键不变，改上一层键变。
#[test]
fn use_through_super_links_only_the_parent_module() {
    assert_eq!(
        key_expectations_after_successive_edits(
            "freexport::parent_module::child_module::read_mark",
            &[
                (
                    "const PARENT_MARK: u32 = 8;",
                    "const PARENT_MARK: u32 = 80;"
                ),
                (
                    "const PARENT_MARK: u32 = 11;",
                    "const PARENT_MARK: u32 = 110;"
                ),
            ],
        ),
        [KeyExpectation::StaysTheSame, KeyExpectation::Changes],
        "super 点到 parent_module：只连那一份"
    );
}

/// 场景：`use self::inner::INNER_MARK`，`spare` 里另有私有的同名常量。预期：改 `spare` 键不变，改 `inner` 键变。
#[test]
fn use_through_self_links_only_the_named_child_module() {
    assert_eq!(
        key_expectations_after_successive_edits(
            "freexport::through_self::read_mark",
            &[
                ("const INNER_MARK: u32 = 9;", "const INNER_MARK: u32 = 90;"),
                (
                    "pub const INNER_MARK: u32 = 12;",
                    "pub const INNER_MARK: u32 = 120;"
                ),
            ],
        ),
        [KeyExpectation::StaysTheSame, KeyExpectation::Changes],
        "self::inner 点到本模块下的 inner：只连那一份"
    );
}

/// 场景：相对路径 `use layout::RELATIVE_MARK`，`spare` 里另有私有的同名常量。预期：改 `spare` 键不变，改 `layout` 键变。
#[test]
fn use_through_a_relative_path_links_only_the_module_ending_with_the_path() {
    assert_eq!(
        key_expectations_after_successive_edits(
            "freexport::through_a_relative_path::read_mark",
            &[
                (
                    "const RELATIVE_MARK: u32 = 10;",
                    "const RELATIVE_MARK: u32 = 100;"
                ),
                (
                    "pub const RELATIVE_MARK: u32 = 13;",
                    "pub const RELATIVE_MARK: u32 = 130;"
                ),
            ],
        ),
        [KeyExpectation::StaysTheSame, KeyExpectation::Changes],
        "相对路径 layout 是候选模块路径的后缀：只连 layout 那一份"
    );
}
