//! 一次编译里各包实际开了哪些特性（D13（验证路线） 已定项 17：键里闭包真引到的 cfg 原子各带「这一次编译」的真值）。
//!
//! cargo 在同一次构建里把各包对同一个依赖要的特性并到唯一那一份上（特性统一）：`cargo test` 一起编 harness 档与 checker 档时，
//! harness 档的 dev 依赖替 `singlefs-core` 打开的特性，checker 档链进来的那一份 `singlefs-core` 同样开着。只看正在编的包自己的
//! `CARGO_FEATURE_*` 与清单里的依赖边，看不到别的包并上来的那一部分。
//!
//! 取法：依赖包带 `links`，它自己的构建脚本把 cargo 这一次给它开的特性（并过之后的）照 [`ENABLED_FEATURES_METADATA_KEY`]
//! 交出去（`cargo::metadata=enabled_features=<逗号隔开的特性名，写成 cargo 环境变量名后缀的样子>`）；cargo 只把它交给**直接**依赖这个包的构建脚本，变量名
//! `DEP_<links>_ENABLED_FEATURES`（[`features_enabled_on_the_dependencies`] 读）。间接依赖的、带特性的包这样取不到，
//! [`features_enabled_on_the_dependencies`] 遇到就交错误，不静默当成没开。
//!
//! 「cargo 这一次开没开」一律从构建脚本自己的 `cfg!(feature = "<名>")` 取（cargo 编构建脚本时带着本包这一次开了的特性的 cfg），
//! 不读进程环境里的 `CARGO_FEATURE_*`：外面带进来的同名变量（不开特性、环境里设 `CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1`）与 cargo 设的分不出，
//! 键会说开着而编出来的代码没开（七·6 后续批代码轮第一轮判决第二节 T4）。正在编的包自己的那一份同样照这个取法交进来
//! （[`features_of_the_crate_being_built`]）。
//!
//! 取到的特性交给 `closure::compute_closure_of_entries_with_the_features_of_the_build`：解特性时这些包一被链进来，就照这一次编译的
//! 样子再请求这些特性（与把它们写进入口包对那个依赖的 `features` 之后解出来的相同）。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::closure::dependency_crate_of;
use crate::manifest::DependencySection;
use crate::workspace::{CrateIndex, Workspace};

/// 依赖包的构建脚本交出特性用的元数据键（`singlefs-core` 的 `build.rs` 打的是同一个键）。
pub const ENABLED_FEATURES_METADATA_KEY: &str = "enabled_features";

/// 这一次编译里 cargo 给各依赖包打开的特性：包名 → 特性名（清单 `[features]` 里的写法）。一个都没并上的包不在表里。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FeaturesEnabledByTheBuild {
    pub features_of_each_package: BTreeMap<String, BTreeSet<String>>,
}

/// 依赖包开了哪些特性取不全：构建脚本应当停下，不按「没开」算键。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildFeaturesError {
    /// 带特性的包不是正在编的包的直接普通依赖（间接依赖、只在 dev 依赖里）：cargo 不把它的元数据交给这个构建脚本。
    NotADirectNormalDependency { package_name: String },
    /// 带特性的直接依赖没写 `links`：它的构建脚本交不出元数据。
    DependencyWithoutLinks { package_name: String },
    /// 带 `links` 的直接依赖没交出特性那一行（构建脚本没打，或键写的不是 [`ENABLED_FEATURES_METADATA_KEY`]）。
    MetadataMissing {
        package_name: String,
        variable_name: String,
    },
    /// 交出的特性名对不上清单 `[features]` 里的任何一个（可选依赖的隐式特性也落在这里）。
    UnrecognizedFeature {
        package_name: String,
        environment_name: String,
    },
    /// 依赖链上有可选的工作区依赖：判不出这一次编没编它。
    OptionalWorkspaceDependency {
        dependent_package_name: String,
        dependency_key: String,
    },
    /// 正在编的包的构建脚本交进来的特性表（特性名与它的 `cfg!(feature = …)`）与清单 `[features]` 对不上：
    /// 清单里有、表里没写的那几个这一次开没开取不到；表里有、清单里没有的是写错的名字。
    FeatureTableDisagreesWithTheManifest {
        package_name: String,
        missing_from_the_table: Vec<String>,
        missing_from_the_manifest: Vec<String>,
    },
}

impl fmt::Display for BuildFeaturesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildFeaturesError::NotADirectNormalDependency { package_name } => write!(
                formatter,
                "包「{package_name}」声明了特性，却不是正在编的包的直接普通依赖：cargo 不把它的 links 元数据交给这个构建脚本，\
                 这一次编译里它开了哪些特性取不到（把它写成直接依赖，或让中间的包转交）"
            ),
            BuildFeaturesError::DependencyWithoutLinks { package_name } => write!(
                formatter,
                "包「{package_name}」声明了特性却没写 [package] links：它的构建脚本交不出这一次编译开了哪些特性"
            ),
            BuildFeaturesError::MetadataMissing {
                package_name,
                variable_name,
            } => write!(
                formatter,
                "包「{package_name}」的构建脚本没交出 {variable_name}（应当打 cargo::metadata={ENABLED_FEATURES_METADATA_KEY}=…）"
            ),
            BuildFeaturesError::UnrecognizedFeature {
                package_name,
                environment_name,
            } => write!(
                formatter,
                "包「{package_name}」交出的特性 {environment_name} 对不上它清单 [features] 里的任何一个"
            ),
            BuildFeaturesError::OptionalWorkspaceDependency {
                dependent_package_name,
                dependency_key,
            } => write!(
                formatter,
                "包「{dependent_package_name}」把工作区里的「{dependency_key}」写成可选依赖：判不出这一次编没编它"
            ),
            BuildFeaturesError::FeatureTableDisagreesWithTheManifest {
                package_name,
                missing_from_the_table,
                missing_from_the_manifest,
            } => write!(
                formatter,
                "包「{package_name}」构建脚本里的特性表与清单 [features] 对不上：清单里有、表里没写的 {missing_from_the_table:?}，\
                 表里有、清单里没有的 {missing_from_the_manifest:?}（表里每个特性写一行 (名字, cfg!(feature = \"名字\"))）"
            ),
        }
    }
}

/// 特性名写进 cargo 环境变量名的样子：大写，`-` 换成 `_`。
fn environment_name_of(name: &str) -> String {
    name.to_ascii_uppercase().replace('-', "_")
}

/// 正在编的包自己开了哪些特性（清单 `[features]` 里、`features_with_their_cfg` 说开着的那几个，按清单里的名字排好）。
/// `features_with_their_cfg` 是它构建脚本里逐个写的（特性名、`cfg!(feature = "<名>")`）：cargo 编构建脚本时带着本包这一次开了的特性的 cfg，
/// 并过之后的，别的包并上来的也在里面；不读进程环境里的 `CARGO_FEATURE_*`（外面带进来的同名变量分不出）。
///
/// # Errors
/// 表与清单 `[features]` 不是同一组名字 ⇒ [`BuildFeaturesError::FeatureTableDisagreesWithTheManifest`]。
pub fn features_of_the_crate_being_built(
    workspace: &Workspace,
    crate_being_built: CrateIndex,
    features_with_their_cfg: &[(&str, bool)],
) -> Result<Vec<String>, BuildFeaturesError> {
    let workspace_crate = &workspace.crates[crate_being_built.0];
    let features_of_the_manifest = &workspace_crate.manifest.features;
    let names_in_the_table: BTreeSet<&str> = features_with_their_cfg
        .iter()
        .map(|(feature, _)| *feature)
        .collect();
    let missing_from_the_table: Vec<String> = features_of_the_manifest
        .keys()
        .filter(|feature| !names_in_the_table.contains(feature.as_str()))
        .cloned()
        .collect();
    let missing_from_the_manifest: Vec<String> = names_in_the_table
        .iter()
        .filter(|feature| !features_of_the_manifest.contains_key(**feature))
        .map(|feature| (*feature).to_string())
        .collect();
    if !missing_from_the_table.is_empty() || !missing_from_the_manifest.is_empty() {
        return Err(BuildFeaturesError::FeatureTableDisagreesWithTheManifest {
            package_name: workspace_crate.package_name.clone(),
            missing_from_the_table,
            missing_from_the_manifest,
        });
    }
    Ok(features_of_the_manifest
        .keys()
        .filter(|feature| {
            features_with_their_cfg
                .iter()
                .any(|(name, is_enabled_by_cargo)| {
                    *name == feature.as_str() && *is_enabled_by_cargo
                })
        })
        .cloned()
        .collect())
}

/// 正在编的包依赖的工作区包（直接的普通依赖、dev 依赖，与它们传递的普通依赖）里，声明了特性的那几个这一次编译各开了哪些：
/// 每个都要是直接普通依赖、带 `links`、它的构建脚本交出了 `DEP_<links>_ENABLED_FEATURES`。
///
/// # Errors
/// 有一个声明了特性的包取不到（[`BuildFeaturesError`] 各成员）；依赖链上有可选的工作区依赖。
pub fn features_enabled_on_the_dependencies(
    workspace: &Workspace,
    crate_being_built: CrateIndex,
    read_environment: &dyn Fn(&str) -> Option<String>,
) -> Result<FeaturesEnabledByTheBuild, BuildFeaturesError> {
    let mut direct_normal_dependencies: BTreeSet<CrateIndex> = BTreeSet::new();
    let mut reached: BTreeSet<CrateIndex> = BTreeSet::new();
    let mut pending: Vec<(CrateIndex, bool)> = vec![(crate_being_built, true)];
    // 每个包至多展开一次：迭代次数不超过工作区里的包数
    while let Some((dependent, is_the_crate_being_built)) = pending.pop() {
        let dependent_crate = &workspace.crates[dependent.0];
        for dependency in &dependent_crate.manifest.dependencies {
            let follows_this_section = match dependency.section {
                DependencySection::Normal => true,
                DependencySection::Development => is_the_crate_being_built,
                DependencySection::Build => false,
            };
            if !follows_this_section {
                continue;
            }
            let Some(path) = dependency.path.as_deref() else {
                continue;
            };
            let Some(dependency_crate) = dependency_crate_of(workspace, dependent, path) else {
                continue;
            };
            if dependency.is_optional {
                return Err(BuildFeaturesError::OptionalWorkspaceDependency {
                    dependent_package_name: dependent_crate.package_name.clone(),
                    dependency_key: dependency.key.clone(),
                });
            }
            if is_the_crate_being_built && dependency.section == DependencySection::Normal {
                direct_normal_dependencies.insert(dependency_crate);
            }
            if dependency_crate != crate_being_built && reached.insert(dependency_crate) {
                pending.push((dependency_crate, false));
            }
        }
    }
    let mut features_enabled_by_the_build = FeaturesEnabledByTheBuild::default();
    for dependency_crate in reached {
        let workspace_crate = &workspace.crates[dependency_crate.0];
        let manifest = &workspace_crate.manifest;
        if manifest.features.is_empty() {
            continue;
        }
        let package_name = workspace_crate.package_name.clone();
        if !direct_normal_dependencies.contains(&dependency_crate) {
            return Err(BuildFeaturesError::NotADirectNormalDependency { package_name });
        }
        let Some(links) = manifest.links.as_deref() else {
            return Err(BuildFeaturesError::DependencyWithoutLinks { package_name });
        };
        let variable_name = format!(
            "DEP_{}_{}",
            environment_name_of(links),
            environment_name_of(ENABLED_FEATURES_METADATA_KEY)
        );
        let Some(reported) = read_environment(&variable_name) else {
            return Err(BuildFeaturesError::MetadataMissing {
                package_name,
                variable_name,
            });
        };
        let mut enabled = BTreeSet::new();
        for environment_name in reported.split(',').filter(|name| !name.is_empty()) {
            let Some(feature) = manifest
                .features
                .keys()
                .find(|feature| environment_name_of(feature) == environment_name)
            else {
                return Err(BuildFeaturesError::UnrecognizedFeature {
                    package_name,
                    environment_name: environment_name.to_string(),
                });
            };
            enabled.insert(feature.clone());
        }
        if !enabled.is_empty() {
            features_enabled_by_the_build
                .features_of_each_package
                .insert(package_name, enabled);
        }
    }
    Ok(features_enabled_by_the_build)
}

/// `singlefs-core` 的构建脚本本身，只在单测里编进来读它的特性表（`tests::the_core_build_script_lists_every_feature_of_its_manifest`）。
/// 本包没有 `lookup-work-counters`，表里那一格的 cfg 在这里恒为假；构建脚本的 `main` 在这里用不上。写在文件顶层：`#[path]` 从本文件所在的
/// `src/` 起算（写在内联的 `mod tests` 里要从不存在的 `src/build_features/tests/` 起算）。
#[cfg(test)]
#[allow(
    dead_code,
    unexpected_cfgs,
    reason = "构建脚本的 main 在单测里不调；它写的 cfg!(feature = \"lookup-work-counters\") 是 singlefs-core 的特性，本包没有这个特性"
)]
#[path = "../../singlefs-core/build.rs"]
mod singlefs_core_build_script;

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::closure::{
        compute_closure_of_entries_with_the_features_of_the_build, reuse_key, ClosureOutcome,
        ClosureRequest, ExternalKeyInputs, RequestedCondition,
    };
    use crate::rule_switches::RuleSwitches;
    use crate::sha256::Sha256Digest;
    use crate::workspace::{load_workspace, TargetSelector};

    const HOST_TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";

    /// 计数的包：`counting` 特性开着编带计数的那一份（与 `singlefs-core` 的 `lookup-work-counters` 同形）。
    const COUNTING_MANIFEST: &str = "[package]\nname = \"counting-basis\"\nversion = \"0.1.0\"\n\
        edition = \"2021\"\nlinks = \"counting-basis\"\n\n[features]\ncounting = []\n";
    const COUNTING_LIBRARY: &str = "#[cfg(feature = \"counting\")]\npub fn count_one_lookup(counter: &mut u64) {\n    \
        *counter += 1;\n}\n\n#[cfg(not(feature = \"counting\"))]\npub fn count_one_lookup(_counter: &mut u64) {}\n";
    /// 判器的包：普通依赖计数的包，不要它的特性。
    const JUDGING_MANIFEST: &str =
        "[package]\nname = \"judging\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
        [dependencies]\ncounting-basis = { path = \"../counting-basis\" }\n";
    /// 判器的包照「一起编」的样子把计数特性写进依赖（攻方量「开」那一把键的做法）。
    const JUDGING_MANIFEST_ASKING_FOR_COUNTING: &str = "[package]\nname = \"judging\"\nversion = \"0.1.0\"\n\
        edition = \"2021\"\n\n[dependencies]\ncounting-basis = { path = \"../counting-basis\", features = [\"counting\"] }\n";
    const JUDGING_LIBRARY: &str =
        "pub fn judge_one_image() -> u64 {\n    let mut counter = 0;\n    \
        counting_basis::count_one_lookup(&mut counter);\n    counter\n}\n";
    /// 脚手架的包：dev 依赖替计数的包打开 `counting`（与 harness 档同形）。
    const SCAFFOLD_MANIFEST: &str = "[package]\nname = \"scaffold\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
        [dependencies]\ncounting-basis = { path = \"../counting-basis\" }\n\n\
        [dev-dependencies]\ncounting-basis = { path = \"../counting-basis\", features = [\"counting\"] }\n";
    const SCAFFOLD_LIBRARY: &str = "pub fn scaffold_nothing() {}\n";

    static FIXTURE_SERIAL: AtomicUsize = AtomicUsize::new(0);

    /// 临时目录里的一棵小工作区，用完删掉：用例做完、断言失败、panic 都删，写到一半 panic 留下的半棵也删。
    struct FixtureWorkspace {
        root: PathBuf,
    }

    impl FixtureWorkspace {
        /// 先认领根、再写：写到一半 panic 时 Drop 删掉写了的那一半。
        fn new(files: &[(&str, &str)]) -> Self {
            let workspace = FixtureWorkspace {
                root: std::env::temp_dir().join(format!(
                    "singlefs-unit-graph-build-features-{}-{}",
                    std::process::id(),
                    FIXTURE_SERIAL.fetch_add(1, Ordering::SeqCst)
                )),
            };
            for (relative_path, text) in files {
                let path = workspace.root.join(relative_path);
                std::fs::create_dir_all(path.parent().expect("夹具文件都在 crates/<包>/ 下"))
                    .expect("临时目录写得了");
                std::fs::write(&path, text).expect("临时目录写得了");
            }
            workspace
        }

        fn load(&self) -> Workspace {
            load_workspace(
                &self.root,
                NonZeroUsize::MIN,
                &RuleSwitches::every_rule_intact(),
            )
            .expect("夹具工作区装得起来")
        }
    }

    impl Drop for FixtureWorkspace {
        /// 删不掉只打一行，不在 Drop 里 panic（panic 途中再 panic 会直接中止进程）。
        fn drop(&mut self) {
            if let Err(error) = std::fs::remove_dir_all(&self.root) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("删不掉自己建的临时目录 {}：{error}", self.root.display());
                }
            }
        }
    }

    /// 临时目录里这个进程建的小工作区：名字以 `singlefs-unit-graph-build-features-<进程号>-` 起头的那几个。
    fn fixture_workspaces_of_this_process_left_in_the_temporary_directory() -> Vec<PathBuf> {
        let prefix = format!("singlefs-unit-graph-build-features-{}-", std::process::id());
        std::fs::read_dir(std::env::temp_dir())
            .expect("临时目录读得了")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
            .map(|entry| entry.path())
            .collect()
    }

    /// 场景：建一棵小工作区，作用域照常走完。预期：走完之前根在，走完之后根不在。
    /// 判别力（`crates/mutations.tsv`）：小工作区的 Drop 不删，根还在，这条红。
    #[test]
    fn a_fixture_workspace_is_gone_once_the_scope_ends_normally() {
        let root = {
            let workspace = FixtureWorkspace::new(&[("crates/judging/src/lib.rs", "")]);
            assert!(
                workspace.root.join("crates/judging/src/lib.rs").exists(),
                "走完之前夹具文件在"
            );
            workspace.root.clone()
        };
        assert!(!root.exists(), "走完之后小工作区 {} 不在了", root.display());
    }

    /// 写到一半 panic 的那棵小工作区里头一份夹具的内容：别的用例的夹具里没有这一行，按它认出这一棵。
    const HALF_WRITTEN_FIXTURE_TEXT: &str = "// 写到一半就 panic 的小工作区\n";

    /// 场景：小工作区写到一半 panic（后一份夹具要把前一份文件当目录用，建目录那一步失败）。
    /// 预期：panic 接住之后，写了的那一半不在了（同一进程里别的用例的小工作区并行建着，按头一份夹具的内容认这一棵）。
    /// 判别力（`crates/mutations.tsv`）：小工作区的 Drop 不删，半棵还在，这条红。
    #[test]
    fn a_fixture_workspace_half_written_when_a_panic_hits_is_gone() {
        let panicked = std::panic::catch_unwind(|| {
            FixtureWorkspace::new(&[
                ("crates/judging/src/lib.rs", HALF_WRITTEN_FIXTURE_TEXT),
                ("crates/judging/src/lib.rs/inner", "写不进：上一级是文件\n"),
            ])
        });
        assert!(panicked.is_err(), "写夹具途中真 panic 了");
        let half_written: Vec<PathBuf> =
            fixture_workspaces_of_this_process_left_in_the_temporary_directory()
                .into_iter()
                .filter(|root| {
                    std::fs::read_to_string(root.join("crates/judging/src/lib.rs"))
                        .is_ok_and(|text| text == HALF_WRITTEN_FIXTURE_TEXT)
                })
                .collect();
        assert_eq!(
            half_written,
            Vec::<PathBuf>::new(),
            "panic 之后写了一半的小工作区不在了"
        );
    }

    fn fixture_with_judging_manifest(judging_manifest: &str) -> FixtureWorkspace {
        FixtureWorkspace::new(&[
            ("crates/counting-basis/Cargo.toml", COUNTING_MANIFEST),
            ("crates/counting-basis/src/lib.rs", COUNTING_LIBRARY),
            ("crates/judging/Cargo.toml", judging_manifest),
            ("crates/judging/src/lib.rs", JUDGING_LIBRARY),
            ("crates/scaffold/Cargo.toml", SCAFFOLD_MANIFEST),
            ("crates/scaffold/src/lib.rs", SCAFFOLD_LIBRARY),
        ])
    }

    fn closure_and_key_of_the_judge(
        workspace: &Workspace,
        features_enabled_by_the_build: &FeaturesEnabledByTheBuild,
    ) -> (ClosureOutcome, Sha256Digest) {
        let request = ClosureRequest {
            entry_path: "judging::judge_one_image".to_string(),
            target: TargetSelector::Library,
            requested_conditions: Vec::<RequestedCondition>::new(),
            uses_default_features: false,
            target_triple: HOST_TARGET_TRIPLE.to_string(),
        };
        let outcome = compute_closure_of_entries_with_the_features_of_the_build(
            workspace,
            &request,
            &[],
            features_enabled_by_the_build,
            &RuleSwitches::every_rule_intact(),
        )
        .expect("判器入口找得到");
        let key = reuse_key(workspace, &outcome, &ExternalKeyInputs::default())
            .expect("夹具闭包里没有拒算");
        (outcome, key)
    }

    fn counting_atom_of(outcome: &ClosureOutcome) -> Vec<String> {
        outcome
            .condition_atoms_with_their_values
            .iter()
            .filter(|atom| atom.starts_with("feature=\"counting\""))
            .cloned()
            .collect()
    }

    fn counting_enabled_by_the_build() -> FeaturesEnabledByTheBuild {
        FeaturesEnabledByTheBuild {
            features_of_each_package: [(
                "counting-basis".to_string(),
                ["counting".to_string()].into_iter().collect(),
            )]
            .into_iter()
            .collect(),
        }
    }

    /// 只编判器的包（计数没开）与和脚手架一起编（脚手架的 dev 依赖把计数并上来）两种编法：判器闭包里计数那个原子的真值分别是
    /// 假与真，键各是那一种编法编出来的代码该有的那一把——开的那一把等于把计数特性写进判器对依赖的 `features` 之后算的键。
    #[test]
    fn a_feature_another_package_unified_onto_the_dependency_flips_its_atom_and_the_key() {
        let fixture = fixture_with_judging_manifest(JUDGING_MANIFEST);
        let workspace = fixture.load();
        let (alone, alone_key) =
            closure_and_key_of_the_judge(&workspace, &FeaturesEnabledByTheBuild::default());
        let (together, together_key) =
            closure_and_key_of_the_judge(&workspace, &counting_enabled_by_the_build());
        assert_eq!(
            counting_atom_of(&alone),
            ["feature=\"counting\"\tfalse"],
            "只编判器的包时计数没开"
        );
        assert_eq!(
            counting_atom_of(&together),
            ["feature=\"counting\"\ttrue"],
            "和脚手架一起编时计数并上来、开着"
        );
        assert_ne!(alone_key, together_key, "两种编法两把键");
        let asking_fixture = fixture_with_judging_manifest(JUDGING_MANIFEST_ASKING_FOR_COUNTING);
        let asking_workspace = asking_fixture.load();
        let (_, asking_key) =
            closure_and_key_of_the_judge(&asking_workspace, &FeaturesEnabledByTheBuild::default());
        assert_eq!(
            together_key, asking_key,
            "并上来的特性算出的键，等于判器自己要这个特性时的那一把"
        );
    }

    /// 没有任何包被并上特性时，键与不带这份输入算的逐字相同；交出空集的包也一样。
    #[test]
    fn no_feature_unified_onto_any_dependency_keeps_the_key_as_without_the_input() {
        let fixture = fixture_with_judging_manifest(JUDGING_MANIFEST);
        let workspace = fixture.load();
        let (_, without_the_input) =
            closure_and_key_of_the_judge(&workspace, &FeaturesEnabledByTheBuild::default());
        let empty_report = FeaturesEnabledByTheBuild {
            features_of_each_package: [("counting-basis".to_string(), BTreeSet::new())]
                .into_iter()
                .collect(),
        };
        let (_, with_an_empty_report) = closure_and_key_of_the_judge(&workspace, &empty_report);
        let without_the_input_through_the_old_entry = reuse_key(
            &workspace,
            &crate::closure::compute_closure(
                &workspace,
                &ClosureRequest {
                    entry_path: "judging::judge_one_image".to_string(),
                    target: TargetSelector::Library,
                    requested_conditions: Vec::new(),
                    uses_default_features: false,
                    target_triple: HOST_TARGET_TRIPLE.to_string(),
                },
                &RuleSwitches::every_rule_intact(),
            )
            .expect("判器入口找得到"),
            &ExternalKeyInputs::default(),
        )
        .expect("夹具闭包里没有拒算");
        assert_eq!(without_the_input, with_an_empty_report);
        assert_eq!(without_the_input, without_the_input_through_the_old_entry);
    }

    fn environment_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
            .collect();
        move |name: &str| {
            pairs
                .iter()
                .find(|(pair_name, _)| pair_name == name)
                .map(|(_, value)| value.clone())
        }
    }

    fn judging_crate(workspace: &Workspace) -> CrateIndex {
        workspace.crate_named("judging").expect("夹具里有判器的包")
    }

    /// 计数的包经 links 交出 `COUNTING`：读成清单里的 `counting`；交出空串就是一个都没开（不进表）。
    #[test]
    fn the_metadata_of_a_direct_dependency_reads_back_as_its_manifest_feature_names() {
        let fixture = fixture_with_judging_manifest(JUDGING_MANIFEST);
        let workspace = fixture.load();
        let read = features_enabled_on_the_dependencies(
            &workspace,
            judging_crate(&workspace),
            &environment_of(&[("DEP_COUNTING_BASIS_ENABLED_FEATURES", "COUNTING")]),
        )
        .expect("直接依赖带 links、交出了特性");
        assert_eq!(read, counting_enabled_by_the_build());
        let read_none = features_enabled_on_the_dependencies(
            &workspace,
            judging_crate(&workspace),
            &environment_of(&[("DEP_COUNTING_BASIS_ENABLED_FEATURES", "")]),
        )
        .expect("交出空串也是交了");
        assert_eq!(read_none, FeaturesEnabledByTheBuild::default());
    }

    /// 带特性的直接依赖没交出那一行：交错误，不当成没开。
    #[test]
    fn a_direct_dependency_with_features_that_reported_nothing_is_an_error() {
        let fixture = fixture_with_judging_manifest(JUDGING_MANIFEST);
        let workspace = fixture.load();
        assert_eq!(
            features_enabled_on_the_dependencies(
                &workspace,
                judging_crate(&workspace),
                &environment_of(&[]),
            ),
            Err(BuildFeaturesError::MetadataMissing {
                package_name: "counting-basis".to_string(),
                variable_name: "DEP_COUNTING_BASIS_ENABLED_FEATURES".to_string(),
            })
        );
    }

    /// 交出的名字对不上清单里的特性：交错误。
    #[test]
    fn a_reported_feature_missing_from_the_manifest_is_an_error() {
        let fixture = fixture_with_judging_manifest(JUDGING_MANIFEST);
        let workspace = fixture.load();
        assert_eq!(
            features_enabled_on_the_dependencies(
                &workspace,
                judging_crate(&workspace),
                &environment_of(&[("DEP_COUNTING_BASIS_ENABLED_FEATURES", "COUNTING,WIDE")]),
            ),
            Err(BuildFeaturesError::UnrecognizedFeature {
                package_name: "counting-basis".to_string(),
                environment_name: "WIDE".to_string(),
            })
        );
    }

    /// 带特性的包只经别的包间接依赖：cargo 不把它的元数据交给这个构建脚本，交错误。
    #[test]
    fn a_dependency_with_features_reached_only_through_another_package_is_an_error() {
        let fixture = FixtureWorkspace::new(&[
            ("crates/counting-basis/Cargo.toml", COUNTING_MANIFEST),
            ("crates/counting-basis/src/lib.rs", COUNTING_LIBRARY),
            ("crates/scaffold/Cargo.toml", SCAFFOLD_MANIFEST),
            ("crates/scaffold/src/lib.rs", SCAFFOLD_LIBRARY),
            (
                "crates/judging/Cargo.toml",
                "[package]\nname = \"judging\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
                 [dependencies]\nscaffold = { path = \"../scaffold\" }\n",
            ),
            ("crates/judging/src/lib.rs", "pub fn judge_one_image() {}\n"),
        ]);
        let workspace = fixture.load();
        assert_eq!(
            features_enabled_on_the_dependencies(
                &workspace,
                judging_crate(&workspace),
                &environment_of(&[("DEP_COUNTING_BASIS_ENABLED_FEATURES", "COUNTING")]),
            ),
            Err(BuildFeaturesError::NotADirectNormalDependency {
                package_name: "counting-basis".to_string(),
            })
        );
    }

    /// 带特性的直接依赖没写 links：交错误。
    #[test]
    fn a_direct_dependency_with_features_but_without_links_is_an_error() {
        let manifest_without_links = COUNTING_MANIFEST.replace("links = \"counting-basis\"\n", "");
        let fixture = FixtureWorkspace::new(&[
            (
                "crates/counting-basis/Cargo.toml",
                manifest_without_links.as_str(),
            ),
            ("crates/counting-basis/src/lib.rs", COUNTING_LIBRARY),
            ("crates/judging/Cargo.toml", JUDGING_MANIFEST),
            ("crates/judging/src/lib.rs", JUDGING_LIBRARY),
        ]);
        let workspace = fixture.load();
        assert_eq!(
            features_enabled_on_the_dependencies(
                &workspace,
                judging_crate(&workspace),
                &environment_of(&[("DEP_COUNTING_BASIS_ENABLED_FEATURES", "COUNTING")]),
            ),
            Err(BuildFeaturesError::DependencyWithoutLinks {
                package_name: "counting-basis".to_string(),
            })
        );
    }

    /// 正在编的包自己的特性照它构建脚本交进来的特性表取（名字与 `cfg!(feature = …)`）：表说开着的取到、说没开的不取；
    /// 没有特性的脚手架交空表取到空。
    #[test]
    fn the_features_of_the_crate_being_built_come_from_the_feature_table_of_its_build_script() {
        let fixture = fixture_with_judging_manifest(JUDGING_MANIFEST);
        let workspace = fixture.load();
        let counting_crate = workspace
            .crate_named("counting-basis")
            .expect("夹具里有计数的包");
        assert_eq!(
            features_of_the_crate_being_built(&workspace, counting_crate, &[("counting", true)]),
            Ok(vec!["counting".to_string()])
        );
        assert_eq!(
            features_of_the_crate_being_built(&workspace, counting_crate, &[("counting", false)]),
            Ok(Vec::new())
        );
        let scaffold_crate = workspace
            .crate_named("scaffold")
            .expect("夹具里有脚手架的包");
        assert_eq!(
            features_of_the_crate_being_built(&workspace, scaffold_crate, &[]),
            Ok(Vec::new())
        );
    }

    /// 特性表与清单 `[features]` 不是同一组名字：清单里有、表里漏写的，表里写了、清单里没有的，都交错误、逐个点名，不按「没开」算。
    #[test]
    fn a_feature_table_that_disagrees_with_the_manifest_is_an_error() {
        let fixture = fixture_with_judging_manifest(JUDGING_MANIFEST);
        let workspace = fixture.load();
        let counting_crate = workspace
            .crate_named("counting-basis")
            .expect("夹具里有计数的包");
        assert_eq!(
            features_of_the_crate_being_built(&workspace, counting_crate, &[("wide", true)]),
            Err(BuildFeaturesError::FeatureTableDisagreesWithTheManifest {
                package_name: "counting-basis".to_string(),
                missing_from_the_table: vec!["counting".to_string()],
                missing_from_the_manifest: vec!["wide".to_string()],
            })
        );
    }

    /// `singlefs-core` 那一侧的约定与这里读的对得上：清单写了 links、构建脚本照 [`ENABLED_FEATURES_METADATA_KEY`] 打元数据。
    /// 构建脚本不依赖本 crate，键名在两处各写一次，这一条钉住它们相同。
    #[test]
    fn the_core_build_script_reports_under_the_key_read_here() {
        let core_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../singlefs-core");
        let manifest_text =
            std::fs::read_to_string(core_directory.join("Cargo.toml")).expect("读得了核心层清单");
        let manifest =
            crate::manifest::parse_crate_manifest(&manifest_text).expect("核心层清单读得了");
        assert_eq!(manifest.links.as_deref(), Some("singlefs-core"));
        let build_script_text =
            std::fs::read_to_string(core_directory.join("build.rs")).expect("读得了核心层构建脚本");
        assert!(
            build_script_text.contains(&format!(
                "\"cargo::metadata={ENABLED_FEATURES_METADATA_KEY}={{}}\""
            )),
            "核心层构建脚本照 {ENABLED_FEATURES_METADATA_KEY} 交出特性"
        );
    }

    /// `singlefs-core` 清单 `[features]` 里的每个特性在它构建脚本的特性表（`FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG`）里恰好一行、
    /// 表里没有清单之外的名字（`crates/singlefs-core/build.rs` 文件头点名的这一条）。构建脚本只读 cfg、构建时自己不比：清单加了特性而表没加，
    /// 那个特性开着时交出去的元数据里没有它，依赖方的判法单元键按没开算，`cargo build` 与 checker 档的构建都不报。
    /// 表编进来读（同一份源码），清单照原文解（`manifest::parse_crate_manifest`）。
    #[test]
    fn the_core_build_script_lists_every_feature_of_its_manifest() {
        let core_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../singlefs-core");
        let manifest_text =
            std::fs::read_to_string(core_directory.join("Cargo.toml")).expect("读得了核心层清单");
        let manifest =
            crate::manifest::parse_crate_manifest(&manifest_text).expect("核心层清单读得了");
        let names_in_the_manifest: Vec<&str> =
            manifest.features.keys().map(String::as_str).collect();
        let mut names_in_the_table: Vec<&str> =
            super::singlefs_core_build_script::FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG
                .iter()
                .map(|(feature, _)| *feature)
                .collect();
        names_in_the_table.sort_unstable();
        assert!(
            !names_in_the_manifest.is_empty(),
            "核心层清单 [features] 里至少有 lookup-work-counters：{names_in_the_manifest:?}"
        );
        assert_eq!(
            names_in_the_table, names_in_the_manifest,
            "核心层构建脚本的特性表与清单 [features] 逐个相同"
        );
    }
}
