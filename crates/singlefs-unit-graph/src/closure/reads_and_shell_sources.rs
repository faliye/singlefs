//! 收集：单元读的文件与构建脚本交的值（R-读）、shell 单元 source 的文件，与最终产物会不会链系统库之外的库。
use super::*;

/// 构建脚本叫 cargo 往产物里多链接库的指令名（接在 [`CARGO_INSTRUCTION_PREFIX`] 后面）：`rustc-link-lib=…`、
/// `rustc-link-arg…=…`（几种后缀都以它起头，`-l` 写在链接器参数里）、`rustc-flags=…`（cargo 只收 `-l` 与 `-L`，写了就当加库）。
const LINK_INSTRUCTIONS_OF_A_BUILD_SCRIPT: [&str; 3] =
    ["rustc-link-lib", "rustc-link-arg", "rustc-flags"];

/// 构建脚本打给 cargo 的指令行的前缀；两冒号的 `cargo::` 是它再接一个 `:`。只有带这个前缀的串才是指令，
/// 单写指令名的串（单元图自己的 [`LINK_INSTRUCTIONS_OF_A_BUILD_SCRIPT`] 这类）不是。
const CARGO_INSTRUCTION_PREFIX: &str = "cargo:";

/// 一个字面量词法单元是串（普通串、原始串，带不带 `b`、`c` 前缀），内容以 [`CARGO_INSTRUCTION_PREFIX`]（或再接一个 `:`）接
/// [`LINK_INSTRUCTIONS_OF_A_BUILD_SCRIPT`] 之一起头：`println!("cargo:rustc-link-lib={}", …)` 这类叫 cargo 加库的那一句。
fn string_literal_starts_a_link_instruction(literal_text: &str) -> bool {
    literal_text
        .trim_start_matches(['b', 'c', 'r', '#'])
        .strip_prefix('"')
        .and_then(|content| content.strip_prefix(CARGO_INSTRUCTION_PREFIX))
        .map(|instruction| instruction.strip_prefix(':').unwrap_or(instruction))
        .is_some_and(|instruction| {
            LINK_INSTRUCTIONS_OF_A_BUILD_SCRIPT
                .iter()
                .any(|instruction_name| instruction.starts_with(instruction_name))
        })
}

/// 闭包判名单块时顺依赖够到的一个工作区包落在哪一边：链进产物，还是只在构建时跑（构建依赖与它的依赖）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrateRole {
    LinkedIntoTheProduct,
    RunAtBuildTime,
}

impl<'workspace> ClosureRun<'workspace> {
    /// 读到的数据文件是 shell 脚本（`shell_sources::is_shell_script`）：它（传递地）`source` / `.` 进来的仓内文件各记一个数据单元，
    /// 落在读这份脚本的那一处（同一个单元、同一行），字节进键；折不出、落在根外、指到不在的文件、bash 规范化不出来的，照 shell 那一份的位置记拒算；bash 规范化跑不下去（bash 不在、退非 0、超时）
    /// 不是拒算，记下头一份、这一趟交 `EntryError::ShellNormalizationFailed`
    /// （`shell_sources::shell_sourced_files`，与 `scripts/gate/lib_function_units.py` 的 `shell_sourced_files` 同一份规则）。
    /// 仓根外面的数据文件不看（显示路径不是从仓根起的）。
    fn follow_shell_sources(
        &mut self,
        site: UnitIndex,
        script_absolute_path: &std::path::Path,
        site_line: u32,
    ) {
        let workspace = self.workspace;
        if self.shell_normalization_failure.is_some() {
            return;
        }
        let script_path_from_the_repository_root = workspace.display_path(script_absolute_path);
        if script_path_from_the_repository_root.starts_with('/')
            || !crate::shell_sources::is_shell_script(
                &workspace.repository_root,
                &script_path_from_the_repository_root,
            )
        {
            return;
        }
        let sourced = match workspace.shell_sourced_files.sourced_files_of(
            &workspace.repository_root,
            &script_path_from_the_repository_root,
        ) {
            Ok(sourced) => sourced,
            Err(failure) => {
                self.shell_normalization_failure = Some(failure);
                return;
            }
        };
        for sourced_file in &sourced.sourced_files {
            match std::fs::read(workspace.repository_root.join(sourced_file)) {
                Ok(bytes) => {
                    self.data_units.insert(DataUnit {
                        site,
                        path_display: sourced_file.clone(),
                        site_line,
                        digest: sha256_of(&bytes),
                        kind: DataUnitKind::File,
                    });
                }
                Err(_) => {
                    self.refusals.insert(Refusal {
                        file: crate::units::RefusalFile::WorkspaceSourceFile(
                            workspace.units[site.0].file,
                        ),
                        line: site_line,
                        form: RefusalForm::DataFileUnreadable,
                        fragment: sourced_file.clone(),
                    });
                }
            }
        }
        for shell_refusal in &sourced.refusals {
            self.refusals.insert(Refusal {
                file: crate::units::RefusalFile::ShellScriptFromTheRepositoryRoot(
                    shell_refusal.file.clone(),
                ),
                line: shell_refusal.line,
                form: shell_refusal.form,
                fragment: shell_refusal.snippet.clone(),
            });
        }
    }

    /// 名单块（[`SourceRead::ExternBlockOfListedSystemLibraryCalls`]）编进的产物链接时可能多出 C 库之外、又看不全的库：「只在链进最终产物的库
    /// 都看得到时放行」。看的包是名单块所在的包与入口所在的包，连同顺依赖（普通、开发、构建三种都顺）够得到的工作区包；其中经构建依赖够到的包
    /// 在构建时跑（构建脚本调它）。有一样就交 `true`（闭包拒算 `extern-function-interface`）：
    /// 依赖里有不在工作区里的包（registry、git 来的，或 `workspace = true` 这类不写 `path` 的；`cc` 这类构建依赖替构建脚本编 C 代码、加库，
    /// 它做了什么看不到）；这些包的任一目标里有一个 `extern` 块的属性里有 `link`（`#[link(…)]`、`cfg_attr` 里套的，不管它进不进闭包：
    /// `link` 对整个产物生效）；这些包的构建脚本、或在构建时跑的包的库里，有一个串字面量是加库的指令
    /// （[`string_literal_starts_a_link_instruction`]；注释与不带 [`CARGO_INSTRUCTION_PREFIX`] 的串不算）。
    fn final_product_may_link_libraries_beyond_the_system_library(
        &self,
        target: TargetIndex,
    ) -> bool {
        let workspace = self.workspace;
        let mut linked_crates: BTreeSet<CrateIndex> = BTreeSet::new();
        let mut crates_run_at_build_time: BTreeSet<CrateIndex> = BTreeSet::new();
        let mut pending: Vec<(CrateIndex, CrateRole)> = vec![
            (
                workspace.targets[target.0].crate_index,
                CrateRole::LinkedIntoTheProduct,
            ),
            (
                workspace.targets[self.entry_target.0].crate_index,
                CrateRole::LinkedIntoTheProduct,
            ),
        ];
        // 每个（包，角色）至多进一次，两个集合只增不减，包数有限，循环到 pending 空为止
        while let Some((crate_index, role)) = pending.pop() {
            let is_new = match role {
                CrateRole::LinkedIntoTheProduct => linked_crates.insert(crate_index),
                CrateRole::RunAtBuildTime => crates_run_at_build_time.insert(crate_index),
            };
            if !is_new {
                continue;
            }
            for dependency in &workspace.crates[crate_index.0].manifest.dependencies {
                let Some(dependency_crate) = dependency
                    .path
                    .as_deref()
                    .and_then(|path| dependency_crate_of(workspace, crate_index, path))
                else {
                    return true;
                };
                let dependency_role = match (role, dependency.section) {
                    (CrateRole::RunAtBuildTime, _) | (_, DependencySection::Build) => {
                        CrateRole::RunAtBuildTime
                    }
                    (
                        CrateRole::LinkedIntoTheProduct,
                        DependencySection::Normal | DependencySection::Development,
                    ) => CrateRole::LinkedIntoTheProduct,
                };
                pending.push((dependency_crate, dependency_role));
            }
        }
        let is_compiled_into_a_target_of = |file: FileIndex, crates: &BTreeSet<CrateIndex>| {
            crates.contains(&workspace.files[file.0].crate_index)
                && !workspace.memberships[file.0].is_empty()
        };
        let every_crate_seen: BTreeSet<CrateIndex> = linked_crates
            .union(&crates_run_at_build_time)
            .copied()
            .collect();
        let has_an_extern_block_naming_link = workspace.units.iter().any(|unit| {
            unit.kind == UnitKind::ExternBlock
                && is_compiled_into_a_target_of(unit.file, &every_crate_seen)
                && unit.token_range.is_some_and(|(first_token, end_token)| {
                    let file = &workspace.files[unit.file.0];
                    extern_block_attributes_name_link(
                        &file.source,
                        &file.tokens[first_token..end_token],
                    )
                })
        });
        let build_scripts_of_the_crates_seen: Vec<TargetIndex> = every_crate_seen
            .iter()
            .filter_map(|crate_index| {
                workspace.target_of_crate(*crate_index, TargetKind::BuildScript, None)
            })
            .collect();
        let libraries_run_at_build_time: Vec<TargetIndex> = crates_run_at_build_time
            .iter()
            .filter_map(|crate_index| {
                workspace.target_of_crate(*crate_index, TargetKind::Library, None)
            })
            .collect();
        let is_run_by_a_build_script = |file: FileIndex| {
            build_scripts_of_the_crates_seen
                .iter()
                .chain(&libraries_run_at_build_time)
                .any(|build_time_target| workspace.membership(file, *build_time_target).is_some())
        };
        let a_build_script_links_a_library = workspace
            .files
            .iter()
            .enumerate()
            .filter(|(file_position, _)| is_run_by_a_build_script(FileIndex(*file_position)))
            .any(|(_, file)| {
                file.tokens.iter().any(|token| {
                    token.kind == TokenKind::Literal
                        && string_literal_starts_a_link_instruction(token_text(&file.source, token))
                })
            });
        has_an_extern_block_naming_link || a_build_script_links_a_library
    }

    pub(super) fn follow_reads(&mut self, unit_index: UnitIndex, target: TargetIndex) {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let crate_index = workspace.targets[target.0].crate_index;
        for read in &unit.reads {
            match read {
                SourceRead::File {
                    absolute_path,
                    line,
                    origin,
                    digest,
                } => {
                    let wgsl_file = workspace
                        .indexes
                        .files_by_path
                        .get(absolute_path)
                        .copied()
                        .filter(|file| workspace.files[file.0].language == SourceLanguage::Wgsl);
                    match (origin, wgsl_file, digest) {
                        (ReadOrigin::Included, Some(file), _) => {
                            self.wgsl_files.insert(file);
                        }
                        (ReadOrigin::Included | ReadOrigin::Runtime, _, Some(digest)) => {
                            self.data_units.insert(DataUnit {
                                site: unit_index,
                                path_display: workspace.display_path(absolute_path),
                                site_line: *line,
                                digest: *digest,
                                kind: DataUnitKind::File,
                            });
                            self.follow_shell_sources(unit_index, absolute_path, *line);
                        }
                        (ReadOrigin::Included | ReadOrigin::Runtime, _, None) => {
                            self.refusals.insert(Refusal {
                                file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                                line: *line,
                                form: RefusalForm::DataFileUnreadable,
                                fragment: workspace.display_path(absolute_path),
                            });
                        }
                    }
                }
                SourceRead::BuildScriptOutput { line } => {
                    match workspace.target_of_crate(crate_index, TargetKind::BuildScript, None) {
                        Some(build_script) => {
                            self.requested_roots.insert(build_script);
                        }
                        None => {
                            self.refusals.insert(Refusal {
                                file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                                line: *line,
                                form: RefusalForm::BuildScriptMissing,
                                fragment: "env!(\"OUT_DIR\")".to_string(),
                            });
                        }
                    }
                }
                SourceRead::BuildScriptEnvironmentValue { .. } => {
                    // 构建脚本经 `cargo::rustc-env` 交的值：连到本 crate 构建脚本的 `main`；没有构建脚本的不连、不拒算
                    // （值来自编译时的进程环境，不在仓里的代码里）。弄坏开关 build-script-environment-edges：不连
                    if self
                        .switches
                        .is_intact(BrokenRule::BuildScriptEnvironmentEdges)
                    {
                        if let Some(build_script) =
                            workspace.target_of_crate(crate_index, TargetKind::BuildScript, None)
                        {
                            self.requested_roots.insert(build_script);
                        }
                    }
                }
                SourceRead::Directory {
                    absolute_path,
                    line,
                    digest,
                } => {
                    self.data_units.insert(DataUnit {
                        site: unit_index,
                        path_display: workspace.display_path(absolute_path),
                        site_line: *line,
                        digest: *digest,
                        kind: DataUnitKind::Directory,
                    });
                }
                // 子进程测试边在闭包收完一趟之后连（`pull_in_child_process_tests`）：参数写法要看闭包里的调用处
                SourceRead::ChildTestProcess { .. } => {}
                // 拒不拒算要看入口，在 `finish` 判
                SourceRead::OwnProgramRestart { .. } => {}
                SourceRead::ExternBlockOfListedSystemLibraryCalls { line, fragment } => {
                    if self.final_product_may_link_libraries_beyond_the_system_library(target) {
                        self.refusals.insert(Refusal {
                            file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                            line: *line,
                            form: RefusalForm::ExternalFunctionInterface,
                            fragment: fragment.clone(),
                        });
                    }
                }
                SourceRead::BinaryExecutable { binary_name, line } => {
                    match workspace.target_of_crate(
                        crate_index,
                        TargetKind::Binary,
                        Some(binary_name),
                    ) {
                        Some(binary) => {
                            self.requested_roots.insert(binary);
                        }
                        None => {
                            self.refusals.insert(Refusal {
                                file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                                line: *line,
                                form: RefusalForm::BinaryTargetMissing,
                                fragment: format!("CARGO_BIN_EXE_{binary_name}"),
                            });
                        }
                    }
                }
            }
        }
    }
}
