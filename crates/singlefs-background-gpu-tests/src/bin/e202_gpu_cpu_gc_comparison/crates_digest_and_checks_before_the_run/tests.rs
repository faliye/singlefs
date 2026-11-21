use super::*;

/// 停机条款第 2 条：产物里早先那几条命令的 crates 摘要有一份与这一次的不同就停。
#[test]
fn the_crates_change_is_seen_when_an_earlier_command_ran_on_another_digest() {
    let version_line = |digest: &str| {
        (
            "crates_version".to_string(),
            BTreeMap::from([("combined_sha256".to_string(), digest.to_string())]),
        )
    };
    assert!(!the_crates_changed_since_the_earlier_commands(&[], "aaaa"));
    assert!(!the_crates_changed_since_the_earlier_commands(
        &[version_line("aaaa"), version_line("aaaa")],
        "aaaa"
    ));
    assert!(the_crates_changed_since_the_earlier_commands(
        &[version_line("aaaa"), version_line("bbbb")],
        "aaaa"
    ));
}

/// 第四节第 3a 步第 2 项：1 ms 里前进 100 次才够细。
#[test]
fn the_thread_cpu_clock_check_needs_a_hundred_advances_in_a_millisecond() {
    assert!(the_thread_cpu_clock_is_fine_enough(100));
    assert!(!the_thread_cpu_clock_is_fine_enough(99));
}

/// 场景：现造一棵小树（只在内存里）：摘要要走的八处各有文件（产品 GPU 包的 `src/` 底下另有一层目录），测试包里另有一份不在摘要里的测试文件。
/// 预期：摘要的文件清单正好是那八处底下的文件（装置自己与它经 `#[path]` 编进来的两份共用模块都在），不在那八处的测试文件不在，按路径的各段排。
#[test]
fn the_crates_digest_covers_the_apparatus_and_the_two_modules_it_compiles_in_by_path() {
    let files_of_the_tree: BTreeMap<&str, &[u8]> = [
        "crates/singlefs-background-gpu/src/lib.rs",
        "crates/singlefs-background-gpu/src/nested/kernel.wgsl",
        "crates/singlefs-mount-runtime/src/lib.rs",
        "crates/singlefs-replay-prefix/src/lib.rs",
        "crates/singlefs-format/src/lib.rs",
        "crates/singlefs-core/src/lib.rs",
        "crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs",
        "crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs",
        "crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs",
        "crates/singlefs-background-gpu-tests/tests/real_gpu_tests_open_only_the_gpus_the_task_server_handed_over.rs",
    ]
    .into_iter()
    .map(|path| (path, path.as_bytes()))
    .collect();
    let what_the_path_is = |path: &str| -> Result<PathWalkedForTheCratesDigest, String> {
        if files_of_the_tree.contains_key(path) {
            return Ok(PathWalkedForTheCratesDigest::File);
        }
        let prefix_of_the_entries = format!("{path}/");
        let entry_names: BTreeSet<String> = files_of_the_tree
            .keys()
            .filter_map(|file| file.strip_prefix(prefix_of_the_entries.as_str()))
            .filter_map(|rest| rest.split('/').next())
            .map(str::to_string)
            .collect();
        if entry_names.is_empty() {
            return Err(format!("小树里没有 {path}"));
        }
        Ok(PathWalkedForTheCratesDigest::Directory {
            entry_names: entry_names.into_iter().collect(),
        })
    };
    let bytes_of_the_file = |path: &str| -> Result<Vec<u8>, String> {
        files_of_the_tree
            .get(path)
            .map(|bytes| bytes.to_vec())
            .ok_or_else(|| format!("小树里没有文件 {path}"))
    };
    let files = source_files_and_digests_walked_with(
        &STARTING_PATHS_OF_THE_CRATES_DIGEST,
        &what_the_path_is,
        &bytes_of_the_file,
    )
    .expect("八处都在小树里");
    assert_eq!(
        files
            .iter()
            .map(|(path, _digest)| path.as_str())
            .collect::<Vec<&str>>(),
        [
            "crates/singlefs-background-gpu/src/lib.rs",
            "crates/singlefs-background-gpu/src/nested/kernel.wgsl",
            "crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs",
            "crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs",
            "crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs",
            "crates/singlefs-core/src/lib.rs",
            "crates/singlefs-format/src/lib.rs",
            "crates/singlefs-mount-runtime/src/lib.rs",
            "crates/singlefs-replay-prefix/src/lib.rs",
        ]
    );
}

/// 装置这份二进制经普通路径依赖链进来的工作区包的清单原文（包目录从仓根起），编进测试二进制：测试包自己，与从它起经普通依赖走得到、
/// 又不在 [`PACKAGES_LEFT_OUT_OF_THE_CRATES_DIGEST`] 里的每个包。走到一个这里没有的包，
/// [`packages_reached_by_normal_path_dependencies`] 停下点名它：加了路径依赖就要在这里加它的清单，再核它的 `src` 进没进摘要。
const MANIFESTS_OF_THE_PACKAGES_LINKED_INTO_THE_APPARATUS: [(&str, &str); 6] = [
    (
        "crates/singlefs-background-gpu-tests",
        include_str!("../../../../Cargo.toml"),
    ),
    (
        "crates/singlefs-background-gpu",
        include_str!("../../../../../singlefs-background-gpu/Cargo.toml"),
    ),
    (
        "crates/singlefs-core",
        include_str!("../../../../../singlefs-core/Cargo.toml"),
    ),
    (
        "crates/singlefs-format",
        include_str!("../../../../../singlefs-format/Cargo.toml"),
    ),
    (
        "crates/singlefs-mount-runtime",
        include_str!("../../../../../singlefs-mount-runtime/Cargo.toml"),
    ),
    (
        "crates/singlefs-replay-prefix",
        include_str!("../../../../../singlefs-replay-prefix/Cargo.toml"),
    ),
];

/// 不进 crates 摘要的工作区包（包目录从仓根起）与为什么：路径依赖走到它收进来，不再往它的依赖里走。
const PACKAGES_LEFT_OUT_OF_THE_CRATES_DIGEST: [(&str, &str); 1] = [(
    "crates/singlefs-verification",
    "验证包只管按需领卡与判定库，不在计时窗里决定读数",
)];

/// 清单原文里写在普通依赖段（`[dependencies]`、`[target.<条件>.dependencies]`）的路径依赖，交回 `path = "…"` 引号里的原样。
/// 开发依赖与构建依赖不链进装置这份二进制，不算。
///
/// # Errors
/// 普通依赖写成这里认不出的样子（一个依赖一段的 `[dependencies.<名>]`、`workspace = true` 的）：说不出它有没有路径。
fn path_dependencies_in_the_normal_dependency_sections(
    manifest_text: &str,
) -> Result<Vec<String>, String> {
    let mut is_in_a_normal_dependency_section = false;
    let mut paths = Vec::new();
    for line in manifest_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') {
            let section_name = trimmed.trim_start_matches('[').trim_end_matches(']');
            let is_a_target_dependency_section =
                section_name.starts_with("target.") && section_name.contains(".dependencies");
            if section_name.starts_with("dependencies.")
                || (is_a_target_dependency_section && section_name.contains(".dependencies."))
            {
                return Err(format!(
                    "认不出一个依赖一段的写法 [{section_name}]：说不出它有没有路径"
                ));
            }
            is_in_a_normal_dependency_section = section_name == "dependencies"
                || (is_a_target_dependency_section && section_name.ends_with(".dependencies"));
            continue;
        }
        if !is_in_a_normal_dependency_section {
            continue;
        }
        let Some((dependency_name, value)) = trimmed.split_once('=') else {
            return Err(format!("普通依赖段里认不出这一行：{trimmed}"));
        };
        let (Some(table_start), Some(table_end)) = (value.find('{'), value.rfind('}')) else {
            continue;
        };
        for key_and_value in value[table_start + 1..table_end].split(',') {
            let Some((key, key_value)) = key_and_value.split_once('=') else {
                continue;
            };
            match key.trim() {
                "path" => paths.push(key_value.trim().trim_matches('"').to_string()),
                "workspace" => {
                    return Err(format!(
                        "认不出依赖 {} 的 workspace 写法：说不出它有没有路径",
                        dependency_name.trim()
                    ))
                }
                _other_key => {}
            }
        }
    }
    Ok(paths)
}

/// 从仓根起的包目录 `package_directory` 照清单里写的相对路径 `relative_path` 走到的目录（从仓根起；`..` 退一层，`.` 不动）。
///
/// # Errors
/// 退出了仓根。
fn directory_reached_from(package_directory: &str, relative_path: &str) -> Result<String, String> {
    let mut segments: Vec<&str> = package_directory.split('/').collect();
    for segment in relative_path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if segments.pop().is_none() {
                    return Err(format!("{package_directory} 照 {relative_path} 退出了仓根"));
                }
            }
            directory_name => segments.push(directory_name),
        }
    }
    Ok(segments.join("/"))
}

/// 从 `starting_package` 起经普通依赖段的路径依赖逐层走得到的包（包目录从仓根起，不含起点自己）；走到 `left_out` 里的包收进来，
/// 不往它的依赖里走。`manifest_of` 交包目录的清单原文。
///
/// # Errors
/// 走到一个交不出清单的包，清单里有认不出的依赖写法，或相对路径退出了仓根。
fn packages_reached_by_normal_path_dependencies(
    starting_package: &str,
    manifest_of: &dyn Fn(&str) -> Option<&'static str>,
    left_out: &BTreeSet<&str>,
) -> Result<BTreeSet<String>, String> {
    let mut reached: BTreeSet<String> = BTreeSet::new();
    let mut packages_to_read: Vec<String> = vec![starting_package.to_string()];
    while let Some(package_directory) = packages_to_read.pop() {
        let manifest_text = manifest_of(&package_directory).ok_or_else(|| {
            format!(
                "走到 {package_directory}，MANIFESTS_OF_THE_PACKAGES_LINKED_INTO_THE_APPARATUS 里没有它的清单：加上它，再核它的 src 进没进摘要"
            )
        })?;
        for relative_path in path_dependencies_in_the_normal_dependency_sections(manifest_text)
            .map_err(|reason| format!("{package_directory}/Cargo.toml：{reason}"))?
        {
            let dependency_directory = directory_reached_from(&package_directory, &relative_path)?;
            if reached.insert(dependency_directory.clone())
                && !left_out.contains(dependency_directory.as_str())
            {
                packages_to_read.push(dependency_directory);
            }
        }
    }
    Ok(reached)
}

/// 场景：从测试包的清单起，经普通依赖段的路径依赖逐层走到工作区里的每个包（验证包登记为不进摘要，走到它就停）。
/// 预期：走到的包除登记不进摘要的之外，`src` 都在 crates 摘要的起点里（这些包的代码都链进装置、在计时窗里跑）；
/// 登记不进摘要的包与编进来的每份清单都真走到了（登记不是空摆着的）。
/// 判别力（`crates/mutations.tsv`）：「E202 crates 摘要起点漏 singlefs-mount-runtime 的 src」让这一条红。
#[test]
fn every_package_linked_into_the_apparatus_by_path_has_its_sources_in_the_crates_digest() {
    let manifests: BTreeMap<&str, &'static str> =
        MANIFESTS_OF_THE_PACKAGES_LINKED_INTO_THE_APPARATUS
            .into_iter()
            .collect();
    let left_out: BTreeSet<&str> = PACKAGES_LEFT_OUT_OF_THE_CRATES_DIGEST
        .iter()
        .map(|(package_directory, _why)| *package_directory)
        .collect();
    let starting_package = "crates/singlefs-background-gpu-tests";
    let reached = packages_reached_by_normal_path_dependencies(
        starting_package,
        &|package_directory: &str| manifests.get(package_directory).copied(),
        &left_out,
    )
    .unwrap_or_else(|reason| panic!("{reason}"));
    let starting_paths: BTreeSet<&str> = STARTING_PATHS_OF_THE_CRATES_DIGEST.into_iter().collect();
    let packages_whose_sources_are_missing: Vec<&str> = reached
        .iter()
        .map(String::as_str)
        .filter(|package_directory| !left_out.contains(package_directory))
        .filter(|package_directory| {
            !starting_paths.contains(format!("{package_directory}/src").as_str())
        })
        .collect();
    assert!(
        packages_whose_sources_are_missing.is_empty(),
        "这几个包经路径依赖链进装置，src 却不在 crates 摘要的起点里（它们改了，停机条款第 2 条看不见）：{packages_whose_sources_are_missing:?}"
    );
    let registrations_not_reached: Vec<&str> = left_out
        .iter()
        .copied()
        .chain(
            manifests
                .keys()
                .copied()
                .filter(|package_directory| *package_directory != starting_package),
        )
        .filter(|package_directory| !reached.contains(*package_directory))
        .collect();
    assert!(
        registrations_not_reached.is_empty(),
        "登记不进摘要的包与编进来的清单都该经路径依赖走得到，这几个没走到（登记过时了，或走法没读出依赖）：{registrations_not_reached:?}"
    );
}
