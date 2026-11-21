//! 入口：入口路径解到哪个目标、哪几个单元，入口的条件与是不是一个程序的 `main`。
use super::*;

/// 这一趟的入口恰好是一个 bin 目标自己的 `fn main`（目标根文件顶层那一个，一组入口里没有别的）、不按测试编：跑起来的进程从这个 `main` 起。
pub(super) fn entry_is_the_main_of_a_program(
    workspace: &Workspace,
    entry_target: TargetIndex,
    entry_units: &[UnitIndex],
    is_entry_compiled_as_test: bool,
) -> bool {
    workspace.targets[entry_target.0].kind == TargetKind::Binary
        && !is_entry_compiled_as_test
        && workspace
            .main_function_of(entry_target)
            .is_some_and(|main| entry_units == [main])
}

pub(super) fn entry_conditions_of(
    request: &ClosureRequest,
    target_kind: TargetKind,
    features_enabled_on_the_dependencies_by_the_build: BTreeMap<CrateIndex, BTreeSet<String>>,
) -> EntryConditions {
    let mut conditions = EntryConditions {
        features: BTreeSet::new(),
        is_test: target_kind.is_compiled_as_test(),
        extra_names: BTreeSet::new(),
        extra_key_values: BTreeSet::new(),
        uses_default_features: request.uses_default_features,
        features_enabled_on_the_dependencies_by_the_build,
    };
    for requested in &request.requested_conditions {
        match requested {
            RequestedCondition::Name(name) if name == "test" => conditions.is_test = true,
            RequestedCondition::Name(name) => {
                conditions.extra_names.insert(name.clone());
            }
            RequestedCondition::Feature(feature) => {
                conditions.features.insert(feature.clone());
            }
            RequestedCondition::KeyValue { key, value } => {
                conditions
                    .extra_key_values
                    .insert((key.clone(), value.clone()));
            }
        }
    }
    conditions
}

pub(super) fn resolve_entry_target(
    workspace: &Workspace,
    request: &ClosureRequest,
) -> Result<TargetIndex, EntryError> {
    let segments: Vec<&str> = request.entry_path.split("::").collect();
    let Some(crate_name) = segments.first().filter(|_| segments.len() >= 2) else {
        return Err(EntryError::MalformedEntryPath(request.entry_path.clone()));
    };
    let crate_index = workspace
        .crate_named(crate_name)
        .ok_or_else(|| EntryError::CrateNotFound((*crate_name).to_string()))?;
    let (kind, name) = match &request.target {
        TargetSelector::Library => (TargetKind::Library, None),
        TargetSelector::Binary(name) => (TargetKind::Binary, Some(name.as_str())),
        TargetSelector::IntegrationTest(name) => (TargetKind::IntegrationTest, Some(name.as_str())),
        TargetSelector::Example(name) => (TargetKind::Example, Some(name.as_str())),
        TargetSelector::Benchmark(name) => (TargetKind::Benchmark, Some(name.as_str())),
        TargetSelector::BuildScript => (TargetKind::BuildScript, None),
        TargetSelector::DocumentationTest => (TargetKind::DocumentationTest, None),
    };
    workspace
        .target_of_crate(crate_index, kind, name)
        .ok_or_else(|| EntryError::TargetNotFound {
            crate_name: (*crate_name).to_string(),
            target: format!(
                "{}{}",
                kind.key_name(),
                name.map(|target_name| format!(":{target_name}"))
                    .unwrap_or_default()
            ),
        })
}

/// 一份文件的单元（工作区里的单元按文件次序排着，同一份文件的单元连在一起）。
pub(super) fn units_of_file(
    workspace: &Workspace,
    file: crate::units::FileIndex,
) -> impl Iterator<Item = UnitIndex> {
    let start = workspace.units.partition_point(|unit| unit.file < file);
    let end = workspace.units.partition_point(|unit| unit.file <= file);
    (start..end).map(UnitIndex)
}

pub(super) fn unit_module_path(
    workspace: &Workspace,
    unit_index: UnitIndex,
    target: TargetIndex,
) -> Option<Vec<String>> {
    let unit = &workspace.units[unit_index.0];
    let membership = workspace.membership(unit.file, target)?;
    let mut path = membership.module_path.clone();
    path.extend(unit.local_module_path.iter().cloned());
    Some(path)
}

fn matches_entry_path(
    workspace: &Workspace,
    unit_index: UnitIndex,
    target: TargetIndex,
    path: &[&str],
) -> bool {
    let unit = &workspace.units[unit_index.0];
    let Some((last, module_part)) = path.split_last() else {
        return false;
    };
    if unit.name != *last {
        return false;
    }
    let Some(module_path) = unit_module_path(workspace, unit_index, target) else {
        return false;
    };
    match unit.container {
        None => module_path
            .iter()
            .map(String::as_str)
            .eq(module_part.iter().copied()),
        Some(container) => {
            let Some((type_name, outer)) = module_part.split_last() else {
                return false;
            };
            let container_unit = &workspace.units[container.0];
            let names_the_type = container_unit.name == *type_name
                || container_unit.implementation.as_ref().is_some_and(|shape| {
                    shape.self_type_names.iter().any(|name| name == type_name)
                });
            names_the_type
                && module_path
                    .iter()
                    .map(String::as_str)
                    .eq(outer.iter().copied())
        }
    }
}

pub(super) fn resolve_entry_units(
    workspace: &Workspace,
    entry_path: &str,
    entry_target: TargetIndex,
    link: &Link,
) -> Result<Vec<UnitIndex>, EntryError> {
    let segments: Vec<&str> = entry_path.split("::").collect();
    let path = &segments[1..];
    let target = &workspace.targets[entry_target.0];
    if target.kind == TargetKind::DocumentationTest {
        let library = workspace
            .target_of_crate(target.crate_index, TargetKind::Library, None)
            .ok_or_else(|| EntryError::EntryNotFound(entry_path.to_string()))?;
        let mut tests = Vec::new();
        for file in &workspace.files_of_target[library.0] {
            if let Some(membership) = workspace.membership(*file, library) {
                for ((module_file, local_path), module_tests) in
                    &workspace.indexes.documentation_tests_of_module
                {
                    if module_file == file {
                        let mut full = membership.module_path.clone();
                        full.extend(local_path.iter().cloned());
                        if full.iter().map(String::as_str).eq(path.iter().copied()) {
                            tests.extend(module_tests.iter().copied());
                        }
                    }
                }
            }
        }
        for (owner, owner_tests) in &workspace.indexes.documentation_tests_of_item {
            if matches_entry_path(workspace, *owner, library, path) {
                tests.extend(owner_tests.iter().copied());
            }
        }
        tests.sort();
        tests.dedup();
        if tests.is_empty() {
            return Err(EntryError::NoDocumentationTests(entry_path.to_string()));
        }
        return Ok(tests);
    }
    let candidates: Vec<UnitIndex> = workspace.files_of_target[entry_target.0]
        .iter()
        .flat_map(|file| units_of_file(workspace, *file))
        .filter(|unit_index| {
            workspace.units[unit_index.0].kind == UnitKind::Function
                && matches_entry_path(workspace, *unit_index, entry_target, path)
        })
        .collect();
    if candidates.is_empty() {
        return Err(EntryError::EntryNotFound(entry_path.to_string()));
    }
    let set = link.condition_set(entry_target);
    let active: Vec<UnitIndex> = candidates
        .iter()
        .copied()
        .filter(
            |candidate| match unit_activity(workspace, &set, *candidate, entry_target) {
                Activity::Inactive => false,
                Activity::Active | Activity::ActiveWithUnknownCondition(_) => true,
            },
        )
        .collect();
    match active.len() {
        0 => Err(EntryError::EntryInactive(entry_path.to_string())),
        1 => Ok(active),
        _ => Err(EntryError::EntryAmbiguous {
            path: entry_path.to_string(),
            candidates: active
                .iter()
                .map(|candidate| workspace.units[candidate.0].qualified_name.clone())
                .collect(),
        }),
    }
}
