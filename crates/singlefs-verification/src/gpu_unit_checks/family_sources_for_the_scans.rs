//! 扫源码的用例读的卡池这一族：根 `gpu_unit_checks.rs` 连同 `gpu_unit_checks/` 下的每一份，逐份编进测试二进制（不读活树），
//! 拼成扫源码用的那一段（[`source_of_the_family`]）。文件表与根起逐层 `mod` 声明到的文件逐个相同，由根里测试模块的
//! `the_family_table_holds_every_file_the_module_declarations_reach_from_the_root` 核。

/// 族里一份文件归哪一种：编进库的（扫它测试那一段之外的代码），只给测试编的（不扫）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RoleInTheFamily {
    CompiledIntoTheLibrary,
    CompiledOnlyForTests,
}

/// 族的根（路径相对 `crates/singlefs-verification/src`）。
pub(super) const ROOT_OF_THE_FAMILY: &str = "gpu_unit_checks.rs";

/// 子模块里测试模块的声明那两整行（行首起、整行逐字）：拼扫源码的那一段时从编进库的子模块里去掉。
const TESTS_MODULE_DECLARATION: &str = "\n#[cfg(test)]\nmod tests;\n";

/// 卡池这一族的每一份：路径（相对 `crates/singlefs-verification/src`）、归哪一种、源码，按路径的字节序排。
pub(super) const GPU_UNIT_CHECKS_FAMILY_FILES: [(&str, RoleInTheFamily, &str); 11] = [
    (
        "gpu_unit_checks.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("../gpu_unit_checks.rs"),
    ),
    (
        "gpu_unit_checks/card_memory_for_the_judges.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("card_memory_for_the_judges.rs"),
    ),
    (
        "gpu_unit_checks/card_memory_for_the_judges/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("card_memory_for_the_judges/tests.rs"),
    ),
    (
        "gpu_unit_checks/discovering_and_starting_the_cards.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("discovering_and_starting_the_cards.rs"),
    ),
    (
        "gpu_unit_checks/discovering_and_starting_the_cards/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("discovering_and_starting_the_cards/tests.rs"),
    ),
    (
        "gpu_unit_checks/family_sources_for_the_scans.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("family_sources_for_the_scans.rs"),
    ),
    (
        "gpu_unit_checks/gpu_test_start.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("gpu_test_start.rs"),
    ),
    (
        "gpu_unit_checks/gpu_test_start/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("gpu_test_start/tests.rs"),
    ),
    (
        "gpu_unit_checks/unit_checksums.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("unit_checksums.rs"),
    ),
    (
        "gpu_unit_checks/work_on_a_card_and_its_readback.rs",
        RoleInTheFamily::CompiledIntoTheLibrary,
        include_str!("work_on_a_card_and_its_readback.rs"),
    ),
    (
        "gpu_unit_checks/work_on_a_card_and_its_readback/tests.rs",
        RoleInTheFamily::CompiledOnlyForTests,
        include_str!("work_on_a_card_and_its_readback/tests.rs"),
    ),
];

/// 文件表里 `path` 那一份的源码。
///
/// # Panics
/// 文件表里没有 `path`。
pub(super) fn source_in_the_family_table(path: &str) -> &'static str {
    GPU_UNIT_CHECKS_FAMILY_FILES
        .iter()
        .find(|(listed_path, _role, _source)| *listed_path == path)
        .map(|(_path, _role, source)| *source)
        .unwrap_or_else(|| panic!("卡池这一族的文件表里有 {path}"))
}

/// 扫源码的用例读的那一段：编进库的子模块各份去掉测试模块的声明那两整行，照文件表的次序接起来，根这一份整份接在最后。
/// 根的测试模块因此是这一段里唯一一个、在最后：切口（根里测试模块的 `code_before_the_tests_module`）切到它为止，
/// 前面是整族测试那一段之外的代码；钉住的条目「按词恰好一处」数的是整族。
///
/// # Panics
/// 编进库的子模块里测试模块的声明那两整行超过一处。
pub(super) fn source_of_the_family() -> String {
    let mut family = String::new();
    for (path, role, source) in GPU_UNIT_CHECKS_FAMILY_FILES {
        if path == ROOT_OF_THE_FAMILY || role == RoleInTheFamily::CompiledOnlyForTests {
            continue;
        }
        assert!(
            source.matches(TESTS_MODULE_DECLARATION).count() <= 1,
            "{path} 里测试模块的声明那两整行至多一处"
        );
        family.push_str(&source.replacen(TESTS_MODULE_DECLARATION, "\n", 1));
    }
    family.push_str(source_in_the_family_table(ROOT_OF_THE_FAMILY));
    family
}
