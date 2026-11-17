//! 把这一次编译里 cargo 给本包打开的特性交给直接依赖本包的构建脚本：`links` 的元数据，依赖方读 `DEP_SINGLEFS_BACKGROUND_GPU_ENABLED_FEATURES`
//! （`singlefs_unit_graph::build_features::features_enabled_on_the_dependencies`，键名是那里的 `ENABLED_FEATURES_METADATA_KEY`）。
//! 做法与取法同 `crates/singlefs-core/build.rs`：cargo 编构建脚本时带着本包这一次开了的特性，`cfg!(feature = "<名>")` 就是这一次真开没开；
//! 不读进程环境里的 `CARGO_FEATURE_*`。只做交出：不生成代码，值是开着的特性名写成 cargo 环境变量名的样子（大写、`-` 换成 `_`），逗号隔开。
//! 清单 `[features]` 里的每个特性都要在 [`FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS`] 里有一行：`singlefs-unit-graph` 的
//! `build_features` 单测把两边逐个比（`the_background_gpu_build_script_lists_every_feature_of_its_manifest`）。

/// 本包清单 `[features]` 里的每个特性（清单里的写法，按名字排好）与 cargo 这一次开没开它。
pub(crate) const FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS: [(&str, bool);
    1] = [("tests-on-a-real-gpu", cfg!(feature = "tests-on-a-real-gpu"))];

fn main() {
    println!(
        "cargo::metadata=enabled_features={}",
        FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CONDITIONAL_COMPILATION_FLAGS
            .iter()
            .filter(|(_, is_enabled_by_cargo)| *is_enabled_by_cargo)
            .map(|(feature, _)| feature.to_ascii_uppercase().replace('-', "_"))
            .collect::<Vec<String>>()
            .join(",")
    );
    println!("cargo::rerun-if-changed=build.rs");
}
