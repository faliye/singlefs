//! 产品行为判定与形式化判定分开取键（m4-exit5-relocation-proof-r3 判决第二节 Z4 与第三节「判器键」）：产品行为视图剥掉 `verus!` 里的
//! 规格与证明块，形式化判定照源码原样。
//!
//! 夹具上照辩方腿原型（`research/prompts/m4-exit5-relocation-proof-r3-sonnet-model/tools/key_variants_stripped.sh`）的八种改法，
//! 改的都是同一个 `key_bytes_width`：只加注释、只用 `verus!` 包一下、加 ensures、加 ensures 与 proof 块、加 spec 函数与 proof 函数与
//! `let ghost` 与 `assert` 五种只改证明的，产品行为的键不变；改执行代码里的串、表达式、加一个属性三种，产品行为的键变——
//! 后三种比的是同样包在 `verus!` 里、只差那一处执行代码的两份，剥到执行代码（两份一起剥没）时它们红。
//! `Ghost<…>` / `Tracked<…>` 参数与 ghost 字段（主 agent 2026-10-26 定也剥）：夹具上两个函数一起换成都包在 `verus!` 里的两种写法比，
//! 只加一个 ghost 参数（连调用处的 `Ghost(…)` 实参）、ghost 字段（连初值与模式），产品行为的键不变；同一处换成普通参数、改执行代码的实参、
//! 改执行字段的类型，键变——剥过头（普通的也剥）时它们红。
//! 判器组（`entry_groups`）与命令行各一格：判器组在产品行为视图上取键；命令行不给 `--judgment-kind` 时照源码原样取键。
//! 真仓上（测试自己拷一份）`crates/singlefs-replay-prefix/src/lib.rs` 一条引理多一句 `assert(true);`：CPU 判器的产品行为键不变，
//! CPU 判器与那条引理的形式化键都变。

mod common_fixture_workspace;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use common_fixture_workspace::{
    closure_with_triple, key_of, key_of_outcome, workspace_of, EntrySpecification, FixtureTree,
    HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::entry_groups::{entry_group_key, EntryGroupRequest, EntryGroupVerdict};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::{ExternalKeyInputs, JudgmentKind, TargetSelector, Workspace};

/// 夹具：一个 crate，判法入口经一个 exec 函数用到一个常量（辩方原型改的是核心层 `allocation_record_tree` 里同名的那一个）。
const STRIPPING_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/fstripped/Cargo.toml",
        "[package]\nname = \"fstripped\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "crates/fstripped/src/lib.rs",
        "const ALLOCATION_RECORD_KEY_BYTES: u64 = 10;\n\
         \n\
         fn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n\
         \n\
         pub fn judge_record_key_width() -> usize {\n    key_bytes_width()\n}\n",
    ),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

const FIXTURE_LIBRARY: &str = "crates/fstripped/src/lib.rs";

/// 原型里被改的那一段，原样。
const ORIGINAL_FUNCTION: &str = "fn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}";

const FIXTURE_JUDGE: EntrySpecification = EntrySpecification {
    entry_path: "fstripped::judge_record_key_width",
    target: "lib",
    conditions: &[],
};

/// 夹具上两个函数连在一起的那一段，原样（ghost 参数与 ghost 字段的用例连调用处、结构体一起换）。
const BOTH_FUNCTIONS: &str = "fn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n\npub fn judge_record_key_width() -> usize {\n    key_bytes_width()\n}";

/// 夹具上把那一段换成 `replacement`（`None` 是不改）之后，判法入口在这类判定的视图上的键。
fn fixture_judge_key(replacement: Option<&str>, kind: JudgmentKind) -> String {
    fixture_judge_key_replacing(ORIGINAL_FUNCTION, replacement, kind)
}

/// 夹具上把 `anchor` 那一段换成 `replacement`（`None` 是不改）之后，判法入口在这类判定的视图上的键。
fn fixture_judge_key_replacing(
    anchor: &str,
    replacement: Option<&str>,
    kind: JudgmentKind,
) -> String {
    let tree = FixtureTree::materialize("strip-verus-specifications", &STRIPPING_FIXTURE);
    if let Some(replacement) = replacement {
        tree.replace_exactly_once(FIXTURE_LIBRARY, anchor, replacement);
    }
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    key_of(workspace.view_for_judgment(kind), &FIXTURE_JUDGE, &switches)
        .unwrap_or_else(|reason| panic!("夹具的判法入口交得出键：{reason}"))
        .hexadecimal()
}

/// 这种改法前后，判法入口的产品行为键变没变（变为 true）。
fn product_behavior_key_changes_under(replacement: &str) -> bool {
    fixture_judge_key(None, JudgmentKind::ProductBehavior)
        != fixture_judge_key(Some(replacement), JudgmentKind::ProductBehavior)
}

/// 两种改法之间（都包在 `verus!` 里、只差一处执行代码），判法入口的产品行为键变没变（变为 true）。
fn product_behavior_key_changes_between(reference: &str, replacement: &str) -> bool {
    fixture_judge_key(Some(reference), JudgmentKind::ProductBehavior)
        != fixture_judge_key(Some(replacement), JudgmentKind::ProductBehavior)
}

/// 这种改法前后，判法入口的形式化键变没变（变为 true）。
fn formal_verification_key_changes_under(replacement: &str) -> bool {
    fixture_judge_key(None, JudgmentKind::FormalVerification)
        != fixture_judge_key(Some(replacement), JudgmentKind::FormalVerification)
}

/// 辩方原型把注释加在函数前面；这里加在函数体里：切词规则被弄坏、注释成了词法单元时，函数前面那一行先让切分拒算，
/// 断言碰不到键，写在函数体里才由「键不变」这一条断言红。
const COMMENT_ONLY: &str = "fn key_bytes_width() -> usize {\n    // 只加一行注释。\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}";

const WRAPPED_IN_VERUS_WITHOUT_GHOST_CODE: &str = "verus! {\nfn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n}";

const WRAPPED_WITH_AN_ENSURES: &str = "verus! {\nfn key_bytes_width() -> (width: usize)\n    ensures width == 10,\n{\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n}";

const WRAPPED_WITH_AN_ENSURES_AND_A_PROOF_BLOCK: &str = "verus! {\nfn key_bytes_width() -> (width: usize)\n    ensures width == 10,\n{\n    proof { assert(true); }\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n}";

const WRAPPED_WITH_SPEC_AND_PROOF_FUNCTIONS_A_GHOST_LET_AND_AN_ASSERT: &str = "verus! {\nspec fn width_in_the_specification() -> int { 10 }\nproof fn lemma_width() ensures width_in_the_specification() == 10 { }\nfn key_bytes_width() -> (width: usize)\n    requires true,\n    ensures width == width_in_the_specification(),\n{\n    proof { lemma_width(); }\n    let ghost width_in_ghost_code = width_in_the_specification();\n    assert(width_in_ghost_code == 10);\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n}";

const EXECUTABLE_STRING_CHANGED: &str = "verus! {\nfn key_bytes_width() -> (width: usize)\n    ensures width == 10,\n{\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"11\")\n}\n}";

const EXECUTABLE_EXPRESSION_CHANGED: &str = "verus! {\nfn key_bytes_width() -> (width: usize)\n    ensures width == 10,\n{\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES + 1).expect(\"10\")\n}\n}";

const EXECUTABLE_ATTRIBUTE_ADDED: &str = "verus! {\n#[inline(never)]\nfn key_bytes_width() -> (width: usize)\n    ensures width == 10,\n{\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n}";

#[test]
fn a_comment_only_change_keeps_the_product_behavior_key() {
    assert!(
        !product_behavior_key_changes_under(COMMENT_ONLY),
        "只加注释：去注释之后逐字相同"
    );
}

#[test]
fn wrapping_the_function_in_verus_without_ghost_code_keeps_the_product_behavior_key() {
    assert!(
        !product_behavior_key_changes_under(WRAPPED_IN_VERUS_WITHOUT_GHOST_CODE),
        "只用 verus! 包一下：包装不进单元"
    );
}

#[test]
fn an_added_ensures_clause_keeps_the_product_behavior_key_and_changes_the_formal_verification_key()
{
    assert!(
        !product_behavior_key_changes_under(WRAPPED_WITH_AN_ENSURES),
        "加 ensures 与具名返回值：产品行为视图剥掉它们，键回到原键"
    );
    assert!(
        formal_verification_key_changes_under(WRAPPED_WITH_AN_ENSURES),
        "形式化判定的键保留规格：加一条 ensures 键要变，不然旧判定顶新规格"
    );
}

#[test]
fn an_added_ensures_clause_and_proof_block_keep_the_product_behavior_key() {
    assert!(
        !product_behavior_key_changes_under(WRAPPED_WITH_AN_ENSURES_AND_A_PROOF_BLOCK),
        "加 ensures 与 proof 块：都剥掉"
    );
}

#[test]
fn added_spec_and_proof_functions_a_ghost_let_and_an_assert_keep_the_product_behavior_key() {
    assert!(
        !product_behavior_key_changes_under(
            WRAPPED_WITH_SPEC_AND_PROOF_FUNCTIONS_A_GHOST_LET_AND_AN_ASSERT
        ),
        "spec 函数、proof 函数、requires、ensures、proof 块里调引理、let ghost、assert 都剥掉，剥掉的部分点到的名字也不再是边"
    );
}

#[test]
fn changing_a_string_in_the_executable_code_changes_the_product_behavior_key() {
    assert!(
        product_behavior_key_changes_between(WRAPPED_WITH_AN_ENSURES, EXECUTABLE_STRING_CHANGED),
        "执行代码里的串改了：产品行为变了，键要变"
    );
}

#[test]
fn changing_an_expression_in_the_executable_code_changes_the_product_behavior_key() {
    assert!(
        product_behavior_key_changes_between(
            WRAPPED_WITH_AN_ENSURES,
            EXECUTABLE_EXPRESSION_CHANGED
        ),
        "执行代码里的表达式改了：键要变"
    );
}

#[test]
fn adding_an_attribute_to_the_executable_function_changes_the_product_behavior_key() {
    assert!(
        product_behavior_key_changes_between(WRAPPED_WITH_AN_ENSURES, EXECUTABLE_ATTRIBUTE_ADDED),
        "exec 函数上加 #[inline(never)]：属性照进指纹，键要变"
    );
}

/// 两个函数一起换成 `reference` 与 `replacement` 两种写法（都包在 `verus!` 里），判法入口在这类判定的视图上的键变没变（变为 true）。
fn key_changes_between_rewrites_of_both_functions(
    kind: JudgmentKind,
    reference: &str,
    replacement: &str,
) -> bool {
    fixture_judge_key_replacing(BOTH_FUNCTIONS, Some(reference), kind)
        != fixture_judge_key_replacing(BOTH_FUNCTIONS, Some(replacement), kind)
}

const BOTH_FUNCTIONS_WRAPPED_IN_VERUS: &str = "verus! {\nfn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n\npub fn judge_record_key_width() -> usize {\n    key_bytes_width()\n}\n}";

const JUDGE_WITH_A_GHOST_PARAMETER: &str = "verus! {\nfn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n\npub fn judge_record_key_width(Ghost(expected_width): Ghost<int>) -> usize {\n    key_bytes_width()\n}\n}";

const JUDGE_WITH_A_TRACKED_PARAMETER: &str = "verus! {\nfn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n\npub fn judge_record_key_width(Tracked(expected_width): Tracked<u64>) -> usize {\n    key_bytes_width()\n}\n}";

const JUDGE_WITH_AN_ORDINARY_PARAMETER: &str = "verus! {\nfn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n\npub fn judge_record_key_width(expected_width: u64) -> usize {\n    key_bytes_width()\n}\n}";

const CALL_WITH_AN_EXECUTABLE_ARGUMENT: &str = "verus! {\nfn key_bytes_width(multiplier: usize) -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\") * multiplier\n}\n\npub fn judge_record_key_width() -> usize {\n    key_bytes_width(1)\n}\n}";

const CALL_WITH_AN_EXECUTABLE_ARGUMENT_CHANGED: &str = "verus! {\nfn key_bytes_width(multiplier: usize) -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\") * multiplier\n}\n\npub fn judge_record_key_width() -> usize {\n    key_bytes_width(2)\n}\n}";

const CALL_WITH_AN_EXECUTABLE_ARGUMENT_AND_A_GHOST_ARGUMENT: &str = "verus! {\nfn key_bytes_width(multiplier: usize, Ghost(expected_width): Ghost<int>) -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\") * multiplier\n}\n\npub fn judge_record_key_width() -> usize {\n    key_bytes_width(1, Ghost(10))\n}\n}";

const METHOD_CALL_WITHOUT_ARGUMENTS: &str = "verus! {\npub struct RecordKeyWidth {\n    pub multiplier: usize,\n}\n\nimpl RecordKeyWidth {\n    fn key_bytes_width(&self) -> usize {\n        usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\") * self.multiplier\n    }\n}\n\npub fn judge_record_key_width() -> usize {\n    RecordKeyWidth { multiplier: 1 }.key_bytes_width()\n}\n}";

const METHOD_CALL_WITH_A_GHOST_ARGUMENT: &str = "verus! {\npub struct RecordKeyWidth {\n    pub multiplier: usize,\n}\n\nimpl RecordKeyWidth {\n    fn key_bytes_width(&self, Ghost(expected_width): Ghost<int>) -> usize {\n        usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\") * self.multiplier\n    }\n}\n\npub fn judge_record_key_width() -> usize {\n    RecordKeyWidth { multiplier: 1 }.key_bytes_width(Ghost(10))\n}\n}";

const STRUCTURE_WITH_AN_EXECUTABLE_FIELD: &str = "verus! {\npub struct RecordKeyWidth {\n    pub width: usize,\n}\n\nfn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n\npub fn judge_record_key_width() -> usize {\n    let RecordKeyWidth { width } = RecordKeyWidth { width: key_bytes_width() };\n    width\n}\n}";

const STRUCTURE_WITH_THE_EXECUTABLE_FIELD_TYPE_CHANGED: &str = "verus! {\npub struct RecordKeyWidth {\n    pub width: u64,\n}\n\nfn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n\npub fn judge_record_key_width() -> usize {\n    let RecordKeyWidth { width } = RecordKeyWidth { width: key_bytes_width() };\n    width\n}\n}";

const STRUCTURE_WITH_A_GHOST_FIELD_ITS_INITIALIZATION_AND_ITS_PATTERN: &str = "verus! {\npub struct RecordKeyWidth {\n    pub width: usize,\n    pub expected_width: Ghost<int>,\n}\n\nfn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n\npub fn judge_record_key_width() -> usize {\n    let RecordKeyWidth { width, expected_width: Ghost(expected) } = RecordKeyWidth { width: key_bytes_width(), expected_width: Ghost(10) };\n    width\n}\n}";

const STRUCTURE_WITH_A_GHOST_MODE_FIELD: &str = "verus! {\npub struct RecordKeyWidth {\n    pub width: usize,\n    pub ghost expected_width: int,\n}\n\nfn key_bytes_width() -> usize {\n    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect(\"10\")\n}\n\npub fn judge_record_key_width() -> usize {\n    let RecordKeyWidth { width } = RecordKeyWidth { width: key_bytes_width() };\n    width\n}\n}";

#[test]
fn an_added_ghost_parameter_keeps_the_product_behavior_key_and_changes_the_formal_verification_key()
{
    assert!(
        !key_changes_between_rewrites_of_both_functions(
            JudgmentKind::ProductBehavior,
            BOTH_FUNCTIONS_WRAPPED_IN_VERUS,
            JUDGE_WITH_A_GHOST_PARAMETER
        ),
        "判法入口加一个 Ghost(名): Ghost<int> 参数：产品行为视图连模式剥掉，键不变"
    );
    assert!(
        key_changes_between_rewrites_of_both_functions(
            JudgmentKind::FormalVerification,
            BOTH_FUNCTIONS_WRAPPED_IN_VERUS,
            JUDGE_WITH_A_GHOST_PARAMETER
        ),
        "形式化判定照源码原样：加一个 ghost 参数键要变"
    );
}

#[test]
fn an_added_tracked_parameter_keeps_the_product_behavior_key() {
    assert!(
        !key_changes_between_rewrites_of_both_functions(
            JudgmentKind::ProductBehavior,
            BOTH_FUNCTIONS_WRAPPED_IN_VERUS,
            JUDGE_WITH_A_TRACKED_PARAMETER
        ),
        "判法入口加一个 Tracked(名): Tracked<u64> 参数：剥掉，键不变"
    );
}

#[test]
fn an_ordinary_parameter_added_in_the_same_place_changes_the_product_behavior_key() {
    assert!(
        key_changes_between_rewrites_of_both_functions(
            JudgmentKind::ProductBehavior,
            BOTH_FUNCTIONS_WRAPPED_IN_VERUS,
            JUDGE_WITH_AN_ORDINARY_PARAMETER
        ),
        "同一处加的是普通参数 expected_width: u64：产品签名变了，键要变"
    );
}

#[test]
fn a_ghost_parameter_and_its_ghost_argument_added_to_a_call_keep_the_product_behavior_key() {
    assert!(
        !key_changes_between_rewrites_of_both_functions(
            JudgmentKind::ProductBehavior,
            CALL_WITH_AN_EXECUTABLE_ARGUMENT,
            CALL_WITH_AN_EXECUTABLE_ARGUMENT_AND_A_GHOST_ARGUMENT
        ),
        "被调函数在普通参数后面加一个 ghost 参数、调用处跟着加 Ghost(10)：两处连前面的逗号一起剥，键不变"
    );
}

#[test]
fn a_ghost_parameter_and_its_ghost_argument_added_to_a_method_call_keep_the_product_behavior_key() {
    assert!(
        !key_changes_between_rewrites_of_both_functions(
            JudgmentKind::ProductBehavior,
            METHOD_CALL_WITHOUT_ARGUMENTS,
            METHOD_CALL_WITH_A_GHOST_ARGUMENT
        ),
        "方法在 &self 后面加一个 ghost 参数、方法调用处跟着加 Ghost(10)：都剥，键不变"
    );
}

#[test]
fn changing_an_executable_argument_changes_the_product_behavior_key() {
    assert!(
        key_changes_between_rewrites_of_both_functions(
            JudgmentKind::ProductBehavior,
            CALL_WITH_AN_EXECUTABLE_ARGUMENT,
            CALL_WITH_AN_EXECUTABLE_ARGUMENT_CHANGED
        ),
        "调用处的普通实参从 1 改成 2：执行代码变了，键要变"
    );
}

#[test]
fn a_ghost_field_added_with_its_initialization_and_its_pattern_keeps_the_product_behavior_key() {
    assert!(
        !key_changes_between_rewrites_of_both_functions(
            JudgmentKind::ProductBehavior,
            STRUCTURE_WITH_AN_EXECUTABLE_FIELD,
            STRUCTURE_WITH_A_GHOST_FIELD_ITS_INITIALIZATION_AND_ITS_PATTERN
        ),
        "结构体加一个 Ghost<int> 字段、字面量里加它的初值 Ghost(10)、模式里加 Ghost(名)：三处连前面的逗号一起剥，键不变"
    );
}

#[test]
fn an_added_ghost_mode_field_keeps_the_product_behavior_key() {
    assert!(
        !key_changes_between_rewrites_of_both_functions(
            JudgmentKind::ProductBehavior,
            STRUCTURE_WITH_AN_EXECUTABLE_FIELD,
            STRUCTURE_WITH_A_GHOST_MODE_FIELD
        ),
        "结构体加一个 ghost 模式的字段 ghost expected_width: int：剥掉，键不变"
    );
}

#[test]
fn changing_the_type_of_an_executable_field_changes_the_product_behavior_key() {
    assert!(
        key_changes_between_rewrites_of_both_functions(
            JudgmentKind::ProductBehavior,
            STRUCTURE_WITH_AN_EXECUTABLE_FIELD,
            STRUCTURE_WITH_THE_EXECUTABLE_FIELD_TYPE_CHANGED
        ),
        "普通字段的类型从 usize 改成 u64：产品的结构体变了，键要变"
    );
}

/// 判器组（崩溃判定库判器组那一种取法）在夹具上的键：交原样装出来的那一份工作区。
fn fixture_judge_group_key(replacement: Option<&str>) -> String {
    let tree = FixtureTree::materialize("strip-verus-judge-group", &STRIPPING_FIXTURE);
    if let Some(replacement) = replacement {
        tree.replace_exactly_once(FIXTURE_LIBRARY, ORIGINAL_FUNCTION, replacement);
    }
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let group = entry_group_key(
        &workspace,
        &EntryGroupRequest {
            group_name: "judge_record_key_width".to_string(),
            target: TargetSelector::Library,
            entry_paths: vec![FIXTURE_JUDGE.entry_path.to_string()],
            features: Vec::new(),
        },
        &[format!("host: {HOST_TARGET_TRIPLE}")],
        &[],
        &switches,
    )
    .unwrap_or_else(|error| panic!("夹具的判器组入口写对了：{error}"));
    match group.verdict {
        EntryGroupVerdict::Keyed { key } => key.hexadecimal(),
        EntryGroupVerdict::Refused { refusal_lines } => {
            panic!("夹具的判器组不该拒算：{refusal_lines:?}")
        }
        EntryGroupVerdict::NotCompiled { reason } => {
            panic!("夹具的判器组入口编得进来：{reason}")
        }
    }
}

#[test]
fn a_judge_group_key_is_taken_on_the_product_behavior_view() {
    assert_eq!(
        fixture_judge_group_key(None),
        fixture_judge_group_key(Some(WRAPPED_WITH_AN_ENSURES_AND_A_PROOF_BLOCK)),
        "判器组是产品行为判定：交原样装出来的工作区，加 ensures 与 proof 块照样不换键"
    );
}

/// 命令行 `unit-graph key` 在夹具（换成 `replacement`）上打的键；`judgment_kind` 是 `None` 时不给 `--judgment-kind`。
fn command_line_key(replacement: &str, judgment_kind: Option<JudgmentKind>) -> String {
    let tree = FixtureTree::materialize("strip-verus-command-line", &STRIPPING_FIXTURE);
    tree.replace_exactly_once(FIXTURE_LIBRARY, ORIGINAL_FUNCTION, replacement);
    let root = tree.root().to_string_lossy().into_owned();
    let host_line = format!("host: {HOST_TARGET_TRIPLE}");
    let mut arguments = vec![
        "key",
        root.as_str(),
        FIXTURE_JUDGE.entry_path,
        "--toolchain-line",
        host_line.as_str(),
    ];
    if let Some(kind) = judgment_kind {
        arguments.extend(["--judgment-kind", kind.option_value()]);
    }
    let output = Command::new(env!("CARGO_BIN_EXE_unit-graph"))
        .args(&arguments)
        .output()
        .expect("测试二进制旁边的 unit-graph 起得来");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    stdout
        .strip_prefix("key\t")
        .map(|rest| rest.trim_end().to_string())
        .unwrap_or_else(|| panic!("key 只打一行键行：{stdout}"))
}

/// 库在这一类判定的视图上、带同一行工具链算的键（命令行那一把的对照）。
fn library_key_with_the_host_line(replacement: &str, kind: JudgmentKind) -> String {
    let tree = FixtureTree::materialize("strip-verus-library-key", &STRIPPING_FIXTURE);
    tree.replace_exactly_once(FIXTURE_LIBRARY, ORIGINAL_FUNCTION, replacement);
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let view = workspace.view_for_judgment(kind);
    let outcome = closure_with_triple(view, &FIXTURE_JUDGE, &switches, HOST_TARGET_TRIPLE)
        .unwrap_or_else(|error| panic!("夹具的判法入口找得到：{error}"));
    key_of_outcome(
        view,
        &outcome,
        &ExternalKeyInputs {
            toolchain_lines: vec![format!("host: {HOST_TARGET_TRIPLE}")],
            lock_lines: Vec::new(),
        },
    )
    .unwrap_or_else(|reason| panic!("夹具的判法入口交得出键：{reason}"))
    .hexadecimal()
}

#[test]
fn the_command_line_keeps_the_specifications_unless_asked_for_the_product_behavior_view() {
    assert_eq!(
        command_line_key(WRAPPED_WITH_AN_ENSURES, None),
        library_key_with_the_host_line(WRAPPED_WITH_AN_ENSURES, JudgmentKind::FormalVerification),
        "不给 --judgment-kind：照源码原样取键（规格进键，原先不带它的调用照旧得同一把键）"
    );
    assert_eq!(
        command_line_key(WRAPPED_WITH_AN_ENSURES, Some(JudgmentKind::ProductBehavior)),
        library_key_with_the_host_line(WRAPPED_WITH_AN_ENSURES, JudgmentKind::ProductBehavior),
        "给 --judgment-kind product-behavior：在产品行为视图上取键"
    );
}

static NEXT_COPY_NUMBER: AtomicUsize = AtomicUsize::new(0);

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("本 crate 在仓根的 crates/ 下")
        .to_path_buf()
}

fn copy_directory(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap_or_else(|error| panic!("建 {}：{error}", to.display()));
    let entries =
        std::fs::read_dir(from).unwrap_or_else(|error| panic!("读 {}：{error}", from.display()));
    for entry in entries {
        let entry = entry.expect("目录项读得出");
        let name = entry.file_name();
        if name == "target" || name == ".git" {
            continue;
        }
        let source = entry.path();
        let destination = to.join(&name);
        if source.is_dir() {
            copy_directory(&source, &destination);
        } else {
            std::fs::copy(&source, destination)
                .unwrap_or_else(|error| panic!("拷 {}：{error}", source.display()));
        }
    }
}

/// 把仓里的 `crates/`、`litmus/`、根上的 `Cargo.toml`、`Cargo.lock` 与 `.claude/heavy-thread-default` 拷进一个新的临时目录。
fn repository_copy() -> FixtureTree {
    let repository = repository_root();
    let copy_number = NEXT_COPY_NUMBER.fetch_add(1, Ordering::SeqCst);
    // 先认领、再拷：拷到一半 panic 时守卫删掉拷了的那一半
    let tree = FixtureTree::adopt(std::env::temp_dir().join(format!(
        "singlefs-unit-graph-strip-verus-repository-{}-{copy_number}",
        std::process::id()
    )));
    let root = tree.root();
    copy_directory(&repository.join("crates"), &root.join("crates"));
    copy_directory(&repository.join("litmus"), &root.join("litmus"));
    for file in ["Cargo.toml", "Cargo.lock", ".claude/heavy-thread-default"] {
        let destination = root.join(file);
        std::fs::create_dir_all(destination.parent().expect("文件在某个目录里")).expect("建目录");
        std::fs::copy(repository.join(file), destination)
            .unwrap_or_else(|error| panic!("拷 {file}：{error}"));
    }
    tree
}

/// CPU 判器入口（崩溃判定库判器组的定义那一组）。
const CENTRAL_PROCESSOR_JUDGE: EntrySpecification = EntrySpecification {
    entry_path: "singlefs_checker_tier::crash::judge_crash_image",
    target: "lib",
    conditions: &[],
};

/// 被加 `assert(true);` 的那条引理（形式化判定按条验的一项）。
const LEMMA_OF_APPLYING_A_RECORD_TWICE: EntrySpecification = EntrySpecification {
    entry_path:
        "singlefs_replay_prefix::lemma_applying_a_record_twice_in_a_row_is_applying_it_once",
    target: "lib",
    conditions: &[],
};

const REPLAY_PREFIX_LIBRARY: &str = "crates/singlefs-replay-prefix/src/lib.rs";
const LEMMA_BODY_ANCHOR: &str = "    let twice = applied_records.insert(position + 1, applied_records[position]);\n    if position == applied_records.len() - 1 {";
const LEMMA_BODY_WITH_AN_ASSERT_TRUE: &str = "    let twice = applied_records.insert(position + 1, applied_records[position]);\n    assert(true);\n    if position == applied_records.len() - 1 {";

/// 树上三把键：CPU 判器的产品行为键、CPU 判器的形式化键、那条引理的形式化键。
fn replay_prefix_keys(tree: &FixtureTree) -> [String; 3] {
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let key = |view: &Workspace, entry: &EntrySpecification| {
        key_of(view, entry, &switches)
            .unwrap_or_else(|reason| panic!("{} 交得出键：{reason}", entry.entry_path))
            .hexadecimal()
    };
    [
        key(
            workspace.view_for_judgment(JudgmentKind::ProductBehavior),
            &CENTRAL_PROCESSOR_JUDGE,
        ),
        key(
            workspace.view_for_judgment(JudgmentKind::FormalVerification),
            &CENTRAL_PROCESSOR_JUDGE,
        ),
        key(
            workspace.view_for_judgment(JudgmentKind::FormalVerification),
            &LEMMA_OF_APPLYING_A_RECORD_TWICE,
        ),
    ]
}

#[test]
fn an_assert_true_added_in_a_replay_prefix_lemma_keeps_the_judge_product_behavior_key_and_changes_the_formal_verification_keys(
) {
    let tree = repository_copy();
    let [product_before, formal_judge_before, formal_lemma_before] = replay_prefix_keys(&tree);
    tree.replace_exactly_once(
        REPLAY_PREFIX_LIBRARY,
        LEMMA_BODY_ANCHOR,
        LEMMA_BODY_WITH_AN_ASSERT_TRUE,
    );
    let [product_after, formal_judge_after, formal_lemma_after] = replay_prefix_keys(&tree);
    assert_eq!(
        product_before, product_after,
        "引理里多一句 assert(true)：CPU 判器的产品行为键不变（引理整条剥掉，不进判器闭包）"
    );
    assert_ne!(
        formal_judge_before, formal_judge_after,
        "CPU 判器的形式化键保留证明，照旧变"
    );
    assert_ne!(
        formal_lemma_before, formal_lemma_after,
        "那条引理的形式化键要变：改了证明的那一项要重验"
    );
}
