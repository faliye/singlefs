//! 只键几个包的单元：闭包照八条规则从入口收（经别的包才够到的这几个包的单元照收；依赖图上一个点名的包都够不到的 crate 不沿它收，
//! [`units_of_the_crates_reaching_none_of_the_listed_packages`]），键、组成清单与拒算只取落在点名的包里的单元；
//! 入口单元、入口目标的文件与别的包的单元连同它们读的数据与 WGSL 都不进键。
//! 判器与观察者引到的被检测产品代码那一把这样取（[`outcome_keeping_only_the_listed_packages`]）：入口是判器与观察者，它们自己的代码不进。

use super::*;

/// 键里留哪几个包的单元（清单 `[package] name` 的写法）；入口单元与入口目标的文件不留。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagesKeptInTheKey {
    pub package_names: BTreeSet<String>,
}

/// `kept` 点名的每个包在工作区里的 crate。
///
/// # Errors
/// 点名的包工作区里没有（[`EntryError::CrateNotFound`]：登记表写错了，调用方应当停下）。
fn crates_of_the_listed_packages(
    workspace: &Workspace,
    kept: &PackagesKeptInTheKey,
) -> Result<BTreeSet<CrateIndex>, EntryError> {
    kept.package_names
        .iter()
        .map(|package_name| {
            workspace
                .crate_named(package_name)
                .ok_or_else(|| EntryError::CrateNotFound(package_name.clone()))
        })
        .collect()
}

/// 依赖图上一个点名的包都够不到的 crate 里每个单元的限定名：只键点名的包的那一把收闭包时，这几个 crate 不沿它收
/// （[`crate::entry_groups::entry_group_key_keeping_only_the_listed_packages`] 交给闭包的「不沿它收」登记）。
/// 够得到：自己就是点名的包，或清单里一条带 `path` 的依赖（普通、开发、构建三种都顺）落到一个够得到的工作区 crate；
/// 不带 `path` 的依赖落在工作区外，不连。
/// 够不到的 crate（判定库服务 prune-server、函数单元图这类测试服务代码）里的代码调不到点名的包：沿它收只多出照名字连的边
/// （R-方法照方法名连、R-裸名照名字连，都不看依赖），收进来的点名的包的单元并不会从那里跑到；它们的文件因此也不进要看的文件。
///
/// # Errors
/// 点名的包工作区里没有（[`EntryError::CrateNotFound`]）。
pub fn units_of_the_crates_reaching_none_of_the_listed_packages(
    workspace: &Workspace,
    kept: &PackagesKeptInTheKey,
) -> Result<BTreeSet<String>, EntryError> {
    let mut crates_reaching_a_listed_package = crates_of_the_listed_packages(workspace, kept)?;
    // 每一轮至少多认下一个 crate 才再来一轮：至多 crate 数那么多轮
    loop {
        let crates_newly_reaching: Vec<CrateIndex> = (0..workspace.crates.len())
            .map(CrateIndex)
            .filter(|crate_index| !crates_reaching_a_listed_package.contains(crate_index))
            .filter(|crate_index| {
                workspace.crates[crate_index.0]
                    .manifest
                    .dependencies
                    .iter()
                    .filter_map(|dependency| dependency.path.as_deref())
                    .filter_map(|path| dependency_crate_of(workspace, *crate_index, path))
                    .any(|dependency_crate| {
                        crates_reaching_a_listed_package.contains(&dependency_crate)
                    })
            })
            .collect();
        if crates_newly_reaching.is_empty() {
            break;
        }
        crates_reaching_a_listed_package.extend(crates_newly_reaching);
    }
    Ok(workspace
        .units
        .iter()
        .filter(|unit| {
            !crates_reaching_a_listed_package.contains(&workspace.files[unit.file.0].crate_index)
        })
        .map(|unit| unit.qualified_name.clone())
        .collect())
}

/// `outcome` 只留 `kept` 点名的包里的单元：
/// - 条件原子取留下的单元各自引到的（[`ClosureOutcome::condition_atoms_of_each_unit`]）；
/// - 数据单元取读它的那个单元留下了的；WGSL 单元取所在文件落在点名的包里的；
/// - 拒算里说闭包收没收全的（[`RefusalForm::leaves_the_edges_of_the_closure_uncertain`]）不论落在哪份文件都留：别的包里切不全、宏拼名字、
///   条件编译判不了的那一处后面可能还有边连到点名的包；只说那一处自己读的输入定不下来的，落在留下的文件里才留；
/// - 入口单元一个都不记（复用键里入口单元那几行 entry 也没有：入口是判器与观察者），落在点名的包里的单元照留；头一条边只留留下的单元的；
///   不沿它收的登记照抄。
///
/// # Errors
/// 点名的包工作区里没有（[`EntryError::CrateNotFound`]：登记表写错了，调用方应当停下）。
pub fn outcome_keeping_only_the_listed_packages(
    workspace: &Workspace,
    outcome: &ClosureOutcome,
    kept: &PackagesKeptInTheKey,
) -> Result<ClosureOutcome, EntryError> {
    let kept_crates = crates_of_the_listed_packages(workspace, kept)?;
    let is_kept_file = |file: FileIndex| kept_crates.contains(&workspace.files[file.0].crate_index);
    let kept_crate_directories: Vec<String> = kept_crates
        .iter()
        .filter_map(|crate_index| {
            workspace.crates[crate_index.0]
                .directory
                .strip_prefix(&workspace.repository_root)
                .ok()
                .map(|relative| format!("{}/", relative.display()))
        })
        .collect();
    let units: Vec<UnitIndex> = outcome
        .units
        .iter()
        .copied()
        .filter(|unit_index| is_kept_file(workspace.units[unit_index.0].file))
        .collect();
    let kept_units: BTreeSet<UnitIndex> = units.iter().copied().collect();
    let condition_atoms_of_each_unit: BTreeMap<UnitIndex, BTreeSet<String>> = outcome
        .condition_atoms_of_each_unit
        .iter()
        .filter(|(unit_index, _atoms)| kept_units.contains(unit_index))
        .map(|(unit_index, atoms)| (*unit_index, atoms.clone()))
        .collect();
    let condition_atoms_with_their_values: BTreeSet<String> = condition_atoms_of_each_unit
        .values()
        .flatten()
        .cloned()
        .collect();
    let is_kept_refusal = |refusal: &Refusal| {
        refusal.form.leaves_the_edges_of_the_closure_uncertain()
            || match &refusal.file {
                crate::units::RefusalFile::WorkspaceSourceFile(file) => is_kept_file(*file),
                crate::units::RefusalFile::ShellScriptFromTheRepositoryRoot(path) => {
                    kept_crate_directories
                        .iter()
                        .any(|directory| path.starts_with(directory.as_str()))
                }
            }
    };
    Ok(ClosureOutcome {
        entry_target_kind: outcome.entry_target_kind,
        entry_units: Vec::new(),
        condition_atoms_with_their_values: condition_atoms_with_their_values.into_iter().collect(),
        condition_atoms_of_each_unit,
        units,
        wgsl_units: outcome
            .wgsl_units
            .iter()
            .copied()
            .filter(|wgsl_index| is_kept_file(workspace.wgsl_units[wgsl_index.0].file))
            .collect(),
        data_units: outcome
            .data_units
            .iter()
            .filter(|data_unit| kept_units.contains(&data_unit.site))
            .cloned()
            .collect(),
        refusals: outcome
            .refusals
            .iter()
            .filter(|refusal| is_kept_refusal(refusal))
            .cloned()
            .collect(),
        first_edges: outcome
            .first_edges
            .iter()
            .filter(|(unit_index, _edge)| kept_units.contains(unit_index))
            .map(|(unit_index, edge)| (*unit_index, edge.clone()))
            .collect(),
        units_reached_and_not_followed: outcome.units_reached_and_not_followed.clone(),
    })
}
