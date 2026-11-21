//! 键的组成：复用键由入口目标种类、条件原子与真值、闭包单元的（种类，指纹）多重集与数据单元摘要拼成。
use super::*;

/// 复用键（设计 3.1）：入口目标的种类与闭包里各单元真引到的条件原子各带真值（闭包没引到的原子不进键）+ 闭包里每个单元（种类，指纹）的多重集 + 数据单元摘要的多重集
/// 加上入口单元自己（`entry_key_lines`）与调用方给的工具链、依赖解析行。
/// 不带路径、名字、行号、手写版本号。
///
/// # Errors
/// 闭包里有拒算：不交键，交那几条拒算。
pub fn reuse_key(
    workspace: &Workspace,
    outcome: &ClosureOutcome,
    external: &ExternalKeyInputs,
) -> Result<Sha256Digest, Vec<Refusal>> {
    if !outcome.refusals.is_empty() {
        return Err(outcome.refusals.clone());
    }
    let mut lines = vec![format!(
        "entry-target-kind\t{}",
        outcome.entry_target_kind.key_name()
    )];
    lines.extend(
        outcome
            .condition_atoms_with_their_values
            .iter()
            .map(|atom_with_its_value| format!("cfg\t{atom_with_its_value}")),
    );
    let mut unit_lines: Vec<String> = outcome
        .units
        .iter()
        .map(|unit_index| {
            let unit = &workspace.units[unit_index.0];
            format!(
                "unit\t{}\t{}",
                unit.kind.output_name(),
                unit.fingerprint.hexadecimal()
            )
        })
        .collect();
    unit_lines.extend(outcome.wgsl_units.iter().map(|wgsl_index| {
        let wgsl_unit = &workspace.wgsl_units[wgsl_index.0];
        format!(
            "unit\t{}\t{}",
            wgsl_unit.kind.output_name(),
            wgsl_unit.fingerprint.hexadecimal()
        )
    }));
    unit_lines.sort();
    lines.extend(unit_lines);
    let mut data_lines: Vec<String> = outcome
        .data_units
        .iter()
        .map(|data_unit| format!("data\t{}", data_unit.digest.hexadecimal()))
        .collect();
    data_lines.sort();
    lines.extend(data_lines);
    lines.extend(entry_key_lines(workspace, outcome));
    lines.extend(
        external
            .toolchain_lines
            .iter()
            .map(|line| format!("toolchain\t{line}")),
    );
    lines.extend(
        external
            .lock_lines
            .iter()
            .map(|line| format!("lock\t{line}")),
    );
    Ok(sha256_of(lines.join("\n").as_bytes()))
}

/// 键里的 entry 行：每个入口单元连同包它的 impl / trait 头（`Unit::container`）一层层往外，各取（种类，指纹），一个入口一行
/// （`entry\t<种类>\t<指纹>[\t<种类>\t<指纹>…]`），排序去重；不带名字、路径与行号。与 `scripts/gate/lib_function_units.py` 的
/// `entry_key_lines` 同一个口径。闭包是单元的多重集，几个入口互相收进对方的闭包（同一个 impl 里的几个方法、互相调用的两个函数）时
/// 闭包一模一样，没有这几行就共用一把键、一份判定。文档测试取到它自己就停：它挂的条目记在 `documentation_test_of`，
/// `container` 是空的，R-结构也不连它（文档测试另编一个 crate）。
fn entry_key_lines(workspace: &Workspace, outcome: &ClosureOutcome) -> Vec<String> {
    let mut entry_lines: BTreeSet<String> = BTreeSet::new();
    for entry in &outcome.entry_units {
        let mut chain_of_kinds_and_fingerprints: Vec<String> = Vec::new();
        let mut units_on_the_chain: BTreeSet<UnitIndex> = BTreeSet::new();
        let mut owner = Some(*entry);
        while let Some(owner_index) = owner {
            assert!(
                units_on_the_chain.insert(owner_index),
                "container 链不成环：成员指向 impl / trait 头，头自己没有 container（rust_items 切分时只这样填）"
            );
            let owner_unit = &workspace.units[owner_index.0];
            assert!(
                owner_unit.kind != UnitKind::DocumentationTest || owner_unit.container.is_none(),
                "文档测试单元的 container 是空的（所挂条目记在 documentation_test_of）：entry 行取到文档测试自己就停"
            );
            chain_of_kinds_and_fingerprints.push(format!(
                "{}\t{}",
                owner_unit.kind.output_name(),
                owner_unit.fingerprint.hexadecimal()
            ));
            owner = owner_unit.container;
        }
        entry_lines.insert(format!(
            "entry\t{}",
            chain_of_kinds_and_fingerprints.join("\t")
        ));
    }
    entry_lines.into_iter().collect()
}
