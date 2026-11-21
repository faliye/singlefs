//! 条件：入口要的特性怎么解、链接集里有哪几个目标、每个目标看得见哪几个目标，与单元在这个目标里生效与否（R-目标）。
use super::*;

/// 一次解析特性：每个被链进来的 crate 开了哪些特性、依赖边。
struct FeatureResolution {
    features: BTreeMap<CrateIndex, BTreeSet<String>>,
    root_edges: BTreeSet<CrateIndex>,
    normal_edges: BTreeMap<CrateIndex, BTreeSet<CrateIndex>>,
}

fn request_feature(
    workspace: &Workspace,
    crate_index: CrateIndex,
    feature: &str,
    features: &mut BTreeMap<CrateIndex, BTreeSet<String>>,
    enabled_optional: &mut BTreeMap<CrateIndex, BTreeSet<String>>,
) {
    if workspace.crates[crate_index.0]
        .manifest
        .features
        .contains_key(feature)
    {
        features
            .entry(crate_index)
            .or_default()
            .insert(feature.to_string());
    } else {
        enabled_optional
            .entry(crate_index)
            .or_default()
            .insert(feature.to_string());
    }
}

/// 照 cargo 的特性统一（在这一次链接里）：根 crate 的请求起，展开 `[features]`（`dep:x`、`x/y`、`x?/y`），顺依赖边把请求传下去，到不再变为止。
/// 非根的 crate 只顺普通依赖（它们的 dev 依赖不编）；同一次构建里别的根（别的包的测试与它们的 dev 依赖）并上来的特性从这一根的清单里看不出，
/// 由 `features_enabled_by_the_build` 交进来：哪个 crate 一被链进来，就照这一次编译的样子再请求它那几个特性。
fn resolve_features(
    workspace: &Workspace,
    root_crate: CrateIndex,
    root_sections: &[DependencySection],
    requested: &BTreeSet<String>,
    uses_default_features: bool,
    features_enabled_by_the_build: &BTreeMap<CrateIndex, BTreeSet<String>>,
) -> FeatureResolution {
    let mut features: BTreeMap<CrateIndex, BTreeSet<String>> = BTreeMap::new();
    let mut enabled_optional: BTreeMap<CrateIndex, BTreeSet<String>> = BTreeMap::new();
    let mut root_edges = BTreeSet::new();
    let mut normal_edges: BTreeMap<CrateIndex, BTreeSet<CrateIndex>> = BTreeMap::new();
    features.insert(root_crate, BTreeSet::new());
    for feature in requested {
        request_feature(
            workspace,
            root_crate,
            feature,
            &mut features,
            &mut enabled_optional,
        );
    }
    if uses_default_features {
        request_feature(
            workspace,
            root_crate,
            "default",
            &mut features,
            &mut enabled_optional,
        );
    }
    loop {
        let size_before: usize = features.values().map(BTreeSet::len).sum::<usize>()
            + enabled_optional.values().map(BTreeSet::len).sum::<usize>()
            + features.len();
        let crates_now: Vec<CrateIndex> = features.keys().copied().collect();
        for crate_index in crates_now {
            for feature_enabled_by_the_build in features_enabled_by_the_build
                .get(&crate_index)
                .into_iter()
                .flatten()
            {
                request_feature(
                    workspace,
                    crate_index,
                    feature_enabled_by_the_build,
                    &mut features,
                    &mut enabled_optional,
                );
            }
            let manifest = &workspace.crates[crate_index.0].manifest;
            let enabled_now: Vec<String> = features
                .get(&crate_index)
                .map(|set| set.iter().cloned().collect())
                .unwrap_or_default();
            for feature in enabled_now {
                let Some(values) = manifest.features.get(&feature) else {
                    continue;
                };
                for value in values {
                    if let Some(dependency_key) = value.strip_prefix("dep:") {
                        enabled_optional
                            .entry(crate_index)
                            .or_default()
                            .insert(dependency_key.to_string());
                    } else if let Some((dependency_part, dependency_feature)) =
                        value.split_once('/')
                    {
                        let is_weak = dependency_part.ends_with('?');
                        let dependency_key = dependency_part.trim_end_matches('?');
                        if !is_weak {
                            enabled_optional
                                .entry(crate_index)
                                .or_default()
                                .insert(dependency_key.to_string());
                        }
                        let dependency_crate = manifest
                            .dependencies
                            .iter()
                            .find(|dependency| dependency.key == dependency_key)
                            .and_then(|dependency| dependency.path.as_deref())
                            .and_then(|path| dependency_crate_of(workspace, crate_index, path));
                        if let Some(dependency_crate) = dependency_crate {
                            request_feature(
                                workspace,
                                dependency_crate,
                                dependency_feature,
                                &mut features,
                                &mut enabled_optional,
                            );
                        }
                    } else {
                        request_feature(
                            workspace,
                            crate_index,
                            value,
                            &mut features,
                            &mut enabled_optional,
                        );
                    }
                }
            }
            let sections: &[DependencySection] = if crate_index == root_crate {
                root_sections
            } else {
                &[DependencySection::Normal]
            };
            for dependency in &manifest.dependencies {
                if !sections.contains(&dependency.section) {
                    continue;
                }
                let Some(path) = dependency.path.as_deref() else {
                    continue;
                };
                let Some(dependency_crate) = dependency_crate_of(workspace, crate_index, path)
                else {
                    continue;
                };
                let is_enabled = !dependency.is_optional
                    || enabled_optional
                        .get(&crate_index)
                        .is_some_and(|enabled| enabled.contains(&dependency.key))
                    || features
                        .get(&crate_index)
                        .is_some_and(|enabled| enabled.contains(&dependency.key));
                if !is_enabled {
                    continue;
                }
                features.entry(dependency_crate).or_default();
                for feature in &dependency.features {
                    request_feature(
                        workspace,
                        dependency_crate,
                        feature,
                        &mut features,
                        &mut enabled_optional,
                    );
                }
                if dependency.uses_default_features {
                    request_feature(
                        workspace,
                        dependency_crate,
                        "default",
                        &mut features,
                        &mut enabled_optional,
                    );
                }
                if crate_index == root_crate {
                    root_edges.insert(dependency_crate);
                }
                if dependency.section == DependencySection::Normal {
                    normal_edges
                        .entry(crate_index)
                        .or_default()
                        .insert(dependency_crate);
                }
            }
        }
        let size_after: usize = features.values().map(BTreeSet::len).sum::<usize>()
            + enabled_optional.values().map(BTreeSet::len).sum::<usize>()
            + features.len();
        if size_after == size_before {
            return FeatureResolution {
                features,
                root_edges,
                normal_edges,
            };
        }
    }
}

fn dependency_sections_of(kind: TargetKind, is_test: bool) -> Vec<DependencySection> {
    match kind {
        TargetKind::BuildScript => vec![DependencySection::Build],
        TargetKind::Library | TargetKind::Binary if !is_test => vec![DependencySection::Normal],
        TargetKind::Library
        | TargetKind::Binary
        | TargetKind::IntegrationTest
        | TargetKind::Example
        | TargetKind::Benchmark
        | TargetKind::DocumentationTest => {
            vec![DependencySection::Normal, DependencySection::Development]
        }
    }
}

/// 一个根目标（入口，或闭包要求补进来的构建脚本、bin）看得见的目标：它自己、本 crate 的库（链得到时）、依赖 crate 的库（传递）。
fn visible_from_root(
    workspace: &Workspace,
    root: TargetIndex,
    resolution: &FeatureResolution,
) -> Vec<TargetIndex> {
    let root_target = &workspace.targets[root.0];
    let mut visible = vec![root];
    if root_target.kind.links_its_own_library() {
        if let Some(library) =
            workspace.target_of_crate(root_target.crate_index, TargetKind::Library, None)
        {
            visible.push(library);
        }
    }
    let mut reached: BTreeSet<CrateIndex> = BTreeSet::new();
    let mut pending: Vec<CrateIndex> = resolution.root_edges.iter().copied().collect();
    if root_target.kind.links_its_own_library() {
        if let Some(own_edges) = resolution.normal_edges.get(&root_target.crate_index) {
            pending.extend(own_edges.iter().copied());
        }
    }
    while let Some(crate_index) = pending.pop() {
        if !reached.insert(crate_index) {
            continue;
        }
        if let Some(edges) = resolution.normal_edges.get(&crate_index) {
            pending.extend(edges.iter().copied());
        }
    }
    for crate_index in reached {
        if let Some(library) = workspace.target_of_crate(crate_index, TargetKind::Library, None) {
            if !visible.contains(&library) {
                visible.push(library);
            }
        }
    }
    visible
}

/// 一个依赖库看得见的目标：它自己与它传递依赖的库。
fn visible_from_library(
    workspace: &Workspace,
    library: TargetIndex,
    resolution: &FeatureResolution,
) -> Vec<TargetIndex> {
    let crate_of_library = workspace.targets[library.0].crate_index;
    let mut visible = vec![library];
    let mut reached: BTreeSet<CrateIndex> = BTreeSet::new();
    let mut pending: Vec<CrateIndex> = resolution
        .normal_edges
        .get(&crate_of_library)
        .map(|edges| edges.iter().copied().collect())
        .unwrap_or_default();
    while let Some(crate_index) = pending.pop() {
        if !reached.insert(crate_index) {
            continue;
        }
        if let Some(edges) = resolution.normal_edges.get(&crate_index) {
            pending.extend(edges.iter().copied());
        }
    }
    for crate_index in reached {
        if let Some(dependency_library) =
            workspace.target_of_crate(crate_index, TargetKind::Library, None)
        {
            visible.push(dependency_library);
        }
    }
    visible
}

/// R-目标：入口目标与要补进来的根目标各解一次特性，合成一个链接集。
pub(super) fn build_link(
    workspace: &Workspace,
    entry_target: TargetIndex,
    entry_conditions: &EntryConditions,
    platform: Option<&TargetPlatform>,
    extra_roots: &BTreeSet<TargetIndex>,
    switches: &RuleSwitches,
) -> Link {
    let mut roots: Vec<(TargetIndex, bool)> = vec![(entry_target, true)];
    roots.extend(extra_roots.iter().map(|root| (*root, false)));
    let mut link = Link {
        targets: BTreeSet::new(),
        visible: HashMap::new(),
        condition_sets: HashMap::new(),
        included_crates: BTreeSet::new(),
    };
    let mut merged_features: BTreeMap<CrateIndex, BTreeSet<String>> = BTreeMap::new();
    for (root, is_entry) in roots {
        let root_target = &workspace.targets[root.0];
        let is_test = is_entry && entry_conditions.is_test;
        let (requested, uses_default) = if is_entry {
            (
                entry_conditions.features.clone(),
                entry_conditions.uses_default_features,
            )
        } else {
            (BTreeSet::new(), true)
        };
        // 构建脚本是给宿主编的，它的依赖不与目标那一侧并特性（resolver 2）：并上来的特性只给目标那一侧的根
        let no_features_enabled_by_the_build = BTreeMap::new();
        let features_enabled_by_the_build = match root_target.kind {
            TargetKind::BuildScript => &no_features_enabled_by_the_build,
            TargetKind::Library
            | TargetKind::Binary
            | TargetKind::IntegrationTest
            | TargetKind::Example
            | TargetKind::Benchmark
            | TargetKind::DocumentationTest => {
                &entry_conditions.features_enabled_on_the_dependencies_by_the_build
            }
        };
        let resolution = resolve_features(
            workspace,
            root_target.crate_index,
            &dependency_sections_of(root_target.kind, is_test),
            &requested,
            uses_default,
            features_enabled_by_the_build,
        );
        let root_visible = visible_from_root(workspace, root, &resolution);
        for visible_target in &root_visible {
            link.targets.insert(*visible_target);
            if *visible_target != root
                && workspace.targets[visible_target.0].kind == TargetKind::Library
            {
                link.visible.entry(*visible_target).or_insert_with(|| {
                    visible_from_library(workspace, *visible_target, &resolution)
                });
            }
        }
        link.visible.insert(root, root_visible);
        for (crate_index, features) in resolution.features {
            link.included_crates.insert(crate_index);
            merged_features
                .entry(crate_index)
                .or_default()
                .extend(features);
        }
        if is_entry {
            link.condition_sets.insert(
                root,
                ConditionalCompilationSet::for_target(
                    platform,
                    &CompilationConditions {
                        features: merged_features
                            .get(&root_target.crate_index)
                            .cloned()
                            .unwrap_or_default(),
                        is_test: entry_conditions.is_test,
                        extra_names: entry_conditions.extra_names.clone(),
                        extra_key_values: entry_conditions.extra_key_values.clone(),
                    },
                ),
            );
        }
    }
    if !switches.is_intact(BrokenRule::TargetScope) {
        let every_target: Vec<TargetIndex> =
            (0..workspace.targets.len()).map(TargetIndex).collect();
        link.targets = every_target.iter().copied().collect();
        for target in &every_target {
            link.visible.insert(*target, every_target.clone());
        }
    }
    let linked: Vec<TargetIndex> = link.targets.iter().copied().collect();
    for target in linked {
        if link.condition_sets.contains_key(&target) {
            continue;
        }
        let crate_index = workspace.targets[target.0].crate_index;
        link.condition_sets.insert(
            target,
            ConditionalCompilationSet::for_target(
                platform,
                &CompilationConditions {
                    features: merged_features
                        .get(&crate_index)
                        .cloned()
                        .unwrap_or_default(),
                    is_test: false,
                    extra_names: BTreeSet::new(),
                    extra_key_values: BTreeSet::new(),
                },
            ),
        );
    }
    if !switches.is_intact(BrokenRule::ConditionalCompilation) {
        for set in link.condition_sets.values_mut() {
            *set = set.clone().holding_every_predicate();
        }
    }
    link
}

pub(super) enum Activity {
    Inactive,
    Active,
    ActiveWithUnknownCondition(String),
}

/// 一个单元在一个目标、一组 cfg 下编不编进来：它自己（连外层模块、impl 头）的条件，再加它所在文件从目标根经哪几条 `mod` 连进来的条件
/// （文件的哪一条连法成立都行；文档测试单元不看文件那一层）。
pub(super) fn unit_activity(
    workspace: &Workspace,
    set: &ConditionalCompilationSet,
    unit_index: UnitIndex,
    target: TargetIndex,
) -> Activity {
    let unit = &workspace.units[unit_index.0];
    let mut unknown_fragment = None;
    match set.evaluate_conjunction(&unit.conditions) {
        Evaluation::DoesNotHold => return Activity::Inactive,
        Evaluation::Unknown => unknown_fragment = Some(rendered_conjunction(&unit.conditions)),
        Evaluation::Holds => {}
    }
    if unit.kind != UnitKind::DocumentationTest {
        let Some(membership) = workspace.membership(unit.file, target) else {
            return Activity::Inactive;
        };
        let mut any_holds = false;
        let mut unknown_alternative = None;
        for alternative in &membership.condition_alternatives {
            match set.evaluate_conjunction(alternative) {
                Evaluation::Holds => any_holds = true,
                Evaluation::Unknown => {
                    unknown_alternative = Some(rendered_conjunction(alternative))
                }
                Evaluation::DoesNotHold => {}
            }
        }
        if !any_holds {
            match unknown_alternative {
                Some(fragment) => unknown_fragment = unknown_fragment.or(Some(fragment)),
                None => return Activity::Inactive,
            }
        }
    }
    match unknown_fragment {
        Some(fragment) => Activity::ActiveWithUnknownCondition(fragment),
        None => Activity::Active,
    }
}

fn rendered_conjunction(predicates: &[Predicate]) -> String {
    predicates
        .iter()
        .map(rendered_predicate)
        .collect::<Vec<String>>()
        .join(" && ")
}
