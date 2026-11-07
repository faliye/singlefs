//! R-裸名在 `crates/` 下的包同一个目标里按模块筛候选（`src/closure.rs` 文件头「R-裸名同一个目标里按模块筛」）：单元只连本模块定义的、
//! 经 use 引进的、glob use 的模块里的、写成 `模块::名` 的那几份，别的模块里同名的私有条目不连；use 或路径的根名是看得见的别的 crate 的，
//! 不点本目标里的哪一份；`super::super::名` 往上数两层；`macro_rules!` 体里的名字在调用处解，照旧全连。
//! 真仓的形状：验证包里 CPU 判器的单元经 `use singlefs_format::{…SLOT_BYTES…}` 用 `SLOT_BYTES`，同一目标里 `gpu_unit_checks` 另有私有的
//! `const SLOT_BYTES`，不该连进 CPU 判器的闭包。夹具是 `crates/` 下的 `fmodules`（几个行内模块）与它依赖的 `fformat`；
//! 每条用例依次改两处、每改一处跟前一把比键：别的模块里同名的那一份改了键不变，真用到的那一份改了键变。
//! 弄坏开关 `crates-directory-names-by-module`（`src/rule_switches.rs`）打开时，别的模块里同名的私有常量改了键也变。

mod common_fixture_workspace;

use common_fixture_workspace::{key_of, workspace_of, EntrySpecification, FixtureTree};
use singlefs_unit_graph::rule_switches::{BrokenRule, RuleSwitches};

const MODULES_FILE: &str = "crates/fmodules/src/lib.rs";
const FORMAT_FILE: &str = "crates/fformat/src/lib.rs";

/// `fmodules` 的库：`cards` 与 `judges` 各有一个私有的 `SLOT_BYTES`；`spare` 里放着与别处同名、谁都不该连到的私有常量。
const MODULES_LIBRARY: &str = r"macro_rules! scaled_by_the_caller {
    () => {
        SCALE * 3
    };
}

pub mod cards {
    const SLOT_BYTES: usize = 32;
    pub fn card_slots() -> usize {
        SLOT_BYTES * 2
    }
}

pub mod judges {
    const SLOT_BYTES: usize = 4096;
    pub fn judge_slots() -> usize {
        SLOT_BYTES / 2
    }
}

pub mod layout {
    pub const PAGE_BYTES: usize = 512;
    pub const LINE_BYTES: usize = 64;
}

pub mod readers {
    use crate::layout::PAGE_BYTES;
    pub fn page_count() -> usize {
        PAGE_BYTES / 8
    }
}

pub mod globbed {
    use crate::layout::*;
    pub fn line_count() -> usize {
        LINE_BYTES / 8
    }
}

pub mod framed {
    use fformat::FRAME_BYTES;
    pub fn frame_count() -> usize {
        FRAME_BYTES / 8
    }
}

pub mod pathed {
    pub fn frame_count_by_path() -> usize {
        fformat::FRAME_BYTES / 4
    }
}

pub mod spare {
    const PAGE_BYTES: usize = 7;
    const LINE_BYTES: usize = 11;
    const FRAME_BYTES: usize = 13;
    const SCALE: u32 = 17;
}

pub mod outer {
    const DEPTH: u32 = 1;
    pub mod middle {
        const DEPTH: u32 = 2;
        pub mod inner {
            pub fn depth() -> u32 {
                super::super::DEPTH
            }
        }
    }
}

pub mod scaled {
    const SCALE: u32 = 5;
    pub fn scaled_value() -> u32 {
        scaled_by_the_caller!()
    }
}
";

const FORMAT_LIBRARY: &str = "pub const FRAME_BYTES: usize = 1024;\n";

const BARE_NAMES_BY_MODULE_FIXTURE: [(&str, &str); 5] = [
    (
        "crates/fmodules/Cargo.toml",
        "[package]\nname = \"fmodules\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nfformat = { path = \"../fformat\" }\n",
    ),
    (MODULES_FILE, MODULES_LIBRARY),
    (
        "crates/fformat/Cargo.toml",
        "[package]\nname = \"fformat\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (FORMAT_FILE, FORMAT_LIBRARY),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyExpectation {
    Changes,
    StaysTheSame,
}

/// 在夹具里改的那一处：哪份文件、恰好出现一次的原文、换成什么。
#[derive(Clone, Copy, Debug)]
struct FixtureEdit {
    file: &'static str,
    original: &'static str,
    replacement: &'static str,
}

const fn library_entry(entry_path: &'static str) -> EntrySpecification {
    EntrySpecification {
        entry_path,
        target: "lib",
        conditions: &[],
    }
}

/// 依次改夹具里的几处（改了不还原），每改一处算一次入口的键、跟前一把比；每一趟都要算得出键，算不出就 panic（带入口错或拒算行）。
fn key_expectations_after_successive_edits(
    entry: &EntrySpecification,
    edits: &[FixtureEdit],
    switches: &RuleSwitches,
) -> Vec<KeyExpectation> {
    let tree = FixtureTree::materialize("bare-names-by-module", &BARE_NAMES_BY_MODULE_FIXTURE);
    let mut previous_key = key_of(&workspace_of(tree.root(), switches), entry, switches)
        .unwrap_or_else(|reason| panic!("{} 改前要算得出键：{reason}", entry.entry_path));
    edits
        .iter()
        .map(|edit| {
            tree.replace_exactly_once(edit.file, edit.original, edit.replacement);
            let key = key_of(&workspace_of(tree.root(), switches), entry, switches).unwrap_or_else(
                |reason| {
                    panic!(
                        "{} 改了「{}」之后要算得出键：{reason}",
                        entry.entry_path, edit.original
                    )
                },
            );
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

const JUDGE_SLOTS_ENTRY: EntrySpecification = library_entry("fmodules::judges::judge_slots");

/// 先改 `cards` 里私有的 `SLOT_BYTES`，再改 `judges` 自己的那一个。
const OWN_MODULE_EDITS: [FixtureEdit; 2] = [
    FixtureEdit {
        file: MODULES_FILE,
        original: "const SLOT_BYTES: usize = 32;",
        replacement: "const SLOT_BYTES: usize = 33;",
    },
    FixtureEdit {
        file: MODULES_FILE,
        original: "const SLOT_BYTES: usize = 4096;",
        replacement: "const SLOT_BYTES: usize = 4097;",
    },
];

/// 场景：`judges::judge_slots` 用自己模块里私有的 `SLOT_BYTES`，`cards` 模块里另有一个同名的私有常量。
/// 预期：改 `cards` 那一个键不变，改 `judges` 那一个键变。
#[test]
fn a_unit_using_the_private_constant_of_its_own_module_does_not_link_the_same_name_in_another_module(
) {
    assert_eq!(
        key_expectations_after_successive_edits(
            &JUDGE_SLOTS_ENTRY,
            &OWN_MODULE_EDITS,
            &RuleSwitches::every_rule_intact(),
        ),
        [KeyExpectation::StaysTheSame, KeyExpectation::Changes],
        "judge_slots 只连 judges::SLOT_BYTES；cards::SLOT_BYTES 在别的模块里、没经 use 引进，不连"
    );
}

/// 场景：弄坏开关 crates-directory-names-by-module，同一组改动。预期：两处改动键都变（同一目标里照旧按同名全连）。
#[test]
fn with_crates_directory_names_by_module_broken_the_private_constant_in_another_module_links_too() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &JUDGE_SLOTS_ENTRY,
            &OWN_MODULE_EDITS,
            &RuleSwitches::with_broken(&[BrokenRule::CratesDirectoryNamesByModule]),
        ),
        [KeyExpectation::Changes, KeyExpectation::Changes],
        "不按模块筛时 judge_slots 的 SLOT_BYTES 连到 cards 与 judges 两份"
    );
}

/// 场景：`readers::page_count` 经 `use crate::layout::PAGE_BYTES` 用它，`spare` 里另有私有的同名常量。
/// 预期：改 `spare` 那一个键不变，改 `layout` 那一个键变。
#[test]
fn a_name_imported_by_a_use_links_only_the_module_the_use_points_at() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &library_entry("fmodules::readers::page_count"),
            &[
                FixtureEdit {
                    file: MODULES_FILE,
                    original: "const PAGE_BYTES: usize = 7;",
                    replacement: "const PAGE_BYTES: usize = 8;",
                },
                FixtureEdit {
                    file: MODULES_FILE,
                    original: "pub const PAGE_BYTES: usize = 512;",
                    replacement: "pub const PAGE_BYTES: usize = 513;",
                },
            ],
            &RuleSwitches::every_rule_intact(),
        ),
        [KeyExpectation::StaysTheSame, KeyExpectation::Changes],
        "use crate::layout::PAGE_BYTES 点到 layout：只连 layout::PAGE_BYTES，不连 spare::PAGE_BYTES"
    );
}

/// 场景：`globbed::line_count` 经 `use crate::layout::*` 用 `LINE_BYTES`，`spare` 里另有私有的同名常量。
/// 预期：改 `spare` 那一个键不变，改 `layout` 那一个键变。
#[test]
fn a_name_brought_in_by_a_glob_use_links_only_the_globbed_module() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &library_entry("fmodules::globbed::line_count"),
            &[
                FixtureEdit {
                    file: MODULES_FILE,
                    original: "const LINE_BYTES: usize = 11;",
                    replacement: "const LINE_BYTES: usize = 12;",
                },
                FixtureEdit {
                    file: MODULES_FILE,
                    original: "pub const LINE_BYTES: usize = 64;",
                    replacement: "pub const LINE_BYTES: usize = 65;",
                },
            ],
            &RuleSwitches::every_rule_intact(),
        ),
        [KeyExpectation::StaysTheSame, KeyExpectation::Changes],
        "use crate::layout::* 整个引进 layout：只连 layout::LINE_BYTES，不连 spare::LINE_BYTES"
    );
}

/// 先改 `spare` 里私有的 `FRAME_BYTES`，再改 `fformat` 的那一个。
const OTHER_CRATE_EDITS: [FixtureEdit; 2] = [
    FixtureEdit {
        file: MODULES_FILE,
        original: "const FRAME_BYTES: usize = 13;",
        replacement: "const FRAME_BYTES: usize = 14;",
    },
    FixtureEdit {
        file: FORMAT_FILE,
        original: "pub const FRAME_BYTES: usize = 1024;",
        replacement: "pub const FRAME_BYTES: usize = 1025;",
    },
];

/// 场景：`framed::frame_count` 经 `use fformat::FRAME_BYTES` 从别的 crate 引进它（真仓里 `use singlefs_format::{…SLOT_BYTES…}` 的形状），
/// `fmodules` 的 `spare` 里另有私有的同名常量。预期：改 `spare` 那一个键不变，改 `fformat` 那一个键变。
#[test]
fn a_name_imported_from_another_crate_does_not_link_a_same_named_item_of_this_crate() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &library_entry("fmodules::framed::frame_count"),
            &OTHER_CRATE_EDITS,
            &RuleSwitches::every_rule_intact(),
        ),
        [KeyExpectation::StaysTheSame, KeyExpectation::Changes],
        "use 的根名 fformat 是看得见的别的 crate：这条 use 不点 fmodules 里的哪一份，spare::FRAME_BYTES 不连"
    );
}

/// 场景：`pathed::frame_count_by_path` 写成 `fformat::FRAME_BYTES`，`spare` 里另有私有的同名常量。
/// 预期：改 `spare` 那一个键不变，改 `fformat` 那一个键变。
#[test]
fn a_path_rooted_at_another_crate_does_not_link_a_same_named_item_of_this_crate() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &library_entry("fmodules::pathed::frame_count_by_path"),
            &OTHER_CRATE_EDITS,
            &RuleSwitches::every_rule_intact(),
        ),
        [KeyExpectation::StaysTheSame, KeyExpectation::Changes],
        "路径的根名 fformat 是看得见的别的 crate：这一处不点 fmodules 里的哪一份，spare::FRAME_BYTES 不连"
    );
}

/// 场景：`outer::middle::inner::depth` 写 `super::super::DEPTH`，`outer` 与 `outer::middle` 各有一个私有的 `DEPTH`。
/// 预期：改 `middle` 那一个键不变，改 `outer` 那一个键变。
#[test]
fn a_path_with_two_supers_links_the_grandparent_module_and_not_the_parent() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &library_entry("fmodules::outer::middle::inner::depth"),
            &[
                FixtureEdit {
                    file: MODULES_FILE,
                    original: "const DEPTH: u32 = 2;",
                    replacement: "const DEPTH: u32 = 3;",
                },
                FixtureEdit {
                    file: MODULES_FILE,
                    original: "const DEPTH: u32 = 1;",
                    replacement: "const DEPTH: u32 = 4;",
                },
            ],
            &RuleSwitches::every_rule_intact(),
        ),
        [KeyExpectation::StaysTheSame, KeyExpectation::Changes],
        "super::super 从 inner 往上两层是 outer：只连 outer::DEPTH，不连 middle::DEPTH"
    );
}

/// 场景：根模块的 `macro_rules! scaled_by_the_caller` 体里写裸名 `SCALE`，在 `scaled` 模块里调用，`SCALE` 由调用处的模块供。
/// 预期：改 `scaled::SCALE` 键变（宏体里的名字不按宏定义所在的根模块筛）。
#[test]
fn names_in_a_macro_rules_body_are_resolved_at_the_call_site_and_link_every_module() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &library_entry("fmodules::scaled::scaled_value"),
            &[FixtureEdit {
                file: MODULES_FILE,
                original: "const SCALE: u32 = 5;",
                replacement: "const SCALE: u32 = 6;",
            }],
            &RuleSwitches::every_rule_intact(),
        ),
        [KeyExpectation::Changes],
        "宏体里的 SCALE 在调用处 scaled 模块里解：macro_rules! 单元的裸名不按模块筛，scaled::SCALE 连上"
    );
}
