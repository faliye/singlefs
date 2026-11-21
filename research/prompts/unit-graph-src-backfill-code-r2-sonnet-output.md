# unit-graph-src-backfill-code-r2：辩方腿报告（2026-11-19）

被复核：第一轮判决 `research/prompts/unit-graph-src-backfill-code-r1-main-verification.md`（第二节十条、第三节六条改法）。
开工核对：`sha256sum -c` 过 `crates-sha256.txt` 十三份与 `governance-sha256.txt` 四份，全部 OK（量的，工作区现文与开工快照同一份）。
我没往任务服务交任务、没编没跑单元图；下面凡标「推的」都是读码推的，待主 agent 跑「要主 agent 跑的命令」一节。
草稿（夹具、静态核对脚本、两个探针）在 `/home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/`，各文件 sha256 在本节末。

## 结论一览

1. 判决第二节十条：对「判决要说的那一形」十条都站得住；没有哪一条判写宽到靠不住。判决第 7 条里「Rust 六项白名单、Python 两处黑名单」今天只在有效值行上对齐了，原文行的收法仍是黑名单，见表。
2. 第三节六条改法：对被打中的那一形都够得着；但各有一个邻形没被够着（推的，各有夹具或探针可验）：
   - 增量编译的邻通道：`CARGO_BUILD_INCREMENTAL`、cargo 配置里的 `[build] incremental` 与 `[profile.<名>] incremental` 不读，而 admission.py 里还写着它们「编出来的代码一样」。
   - 标准宏同名：宏名单按名字认，工作区里自己写一个叫 `write` 的宏就当标准宏。
   - glob 引进的首段：相对路径首段由 glob 引进的别名，没算进「别的 use 引进」。
   - 链接的邻形：`rustc-link-arg`、`rustc-flags` 里的 `-l`，与依赖 crate 里的 `#[link]` 没算进「多链接了库」。
3. 工作区成员与数据目录两份补丁：规则读法与 Python 同一条，只在边角处不同。正文列的两处：内联空表是真差、真实仓碰不到；`"."` 成员的缺名默认名是真差（Python 取 `.`、Rust 取 `root`），还多一个正文没列的差（根清单带 `[package]` 又列 `"."` 时 Python 收两个 crate、Rust 收一个），真实仓碰不到。另有两处 Rust 与 Python 次序不同、`[!…]` 取反分支没有测试与变异行（见第三段）。
4. 检查单九格：成立 5 格；「放的位置」不成立（跨包 `include_str!` 两处）；「所有任务多机可用」推不出（邻通道未读）；「类型与名字」成立但正文写错一处事实（非测试段只有一处 `expect`）；「带 --item」推不出（冻结的 69 行漏了 13 行）。

## 第二节十条逐条复核

| 判决条目 | 辩方判定 | 证据与推翻条件 |
|---|---|---|
| 第 1 条：profile 的 incremental 不进键，而 codegen-units 没写时的切法随增量变（判决第 26 行） | 站得住（打中的是「codegen-units 没写」那一形）；改法够得着这一形、够不着邻通道 | 判决第 18 行记着探针读数 `codegen-units-probe incremental objects=128`、`codegen-units-probe plain objects=1`，只坐实了「开不开增量，代码生成单元数不同」，没坐实缺省值 256 与 16（`build_toolchain.rs` 里自己写「本机没有文档可核」）。邻通道：`build_toolchain.rs:237` 只读 `CARGO_PROFILE_<名>_<键>`，Rust 模块头 `build_toolchain.rs:7` 写明不认 cargo 配置；`scripts/gate/admission.py:873`：「"CARGO_BUILD_INCREMENTAL": "只定增量编译的缓存，编出来的代码一样"}」仍这样写着；`scripts/gate/admission.py:876`：「BUILD_KEYS_LEFT_OUT = frozenset(("jobs", "target-dir", "incremental"))」仍含 incremental。这句「编出来的代码一样」正是判决第 1 条用 128 对 1 推翻的前提。cargo 文档说 `build.incremental` 与 `CARGO_BUILD_INCREMENTAL` 盖过各 profile（凭记忆，推的）。推翻：探针 `cargo-incremental-channels-probe.sh` 里通道 4、5、6、8 的 rustc 命令行 `-C incremental=` 个数与通道 1、7 相同，就是这些通道不改增量，我撤回。 |
| 第 1 条的「同样打中别的臂」问题 | 同样打中别的臂 | 替代臂「incremental 单独出一行」也读不到配置通道，邻通道缺口与臂的选择无关，是各臂共有的「cargo 配置不读」缺口，不是这一臂的缺点。选定的臂另有一处别的臂没有的缺口：codegen-units 写死时增量开关不进键（`build_toolchain.rs` 里 `if !effective.contains_key(CODEGEN_UNITS_SETTING_KEY)` 才看增量）。写死时增量还改不改切法没人量过：探针 `codegen-units-explicit-probe.sh` 量，打开与不开的 `.o` 数或 `value_1` 符号数不同才算这一臂窄了。 |
| 第 2 条：宏实参里的名字一律按局部算（判决第 27 行） | 站得住；改法够得着判决点名的那一形 | 判决第 19 行记的夹具读数：夹具 `fshadow::entry_through_the_module_path`、`entry_inside_a_nested_function` 清单里没有 `fshadow::helper_total`。现在 `closure.rs:1138` 的 16 项名单之外的宏括号里照连。邻形：`crates/singlefs-unit-graph/src/closure.rs:1223`：「.contains(&identifier_name(text_at(position - 2)));」按宏名字面认，不看宏是谁定义的；全仓 `grep -rnE 'macro_rules!\s+(format|…|vec)\b'`（crates、gpu-verification-server、prune-server、task-server、scripts，量的）现在没有同名宏，所以今天没有实例，只是边角（推的）。夹具 `std-named-macro-of-the-workspace` 喂 `entry_through_the_std_named_macro`，清单里没有 `fstdmacro::helper_total` 就是邻形成立。 |
| 第 3 条：use 路径点准候选，两形（cfg 互斥的转出、`as` 改名的首段，判决第 28 行） | 站得住（两形）；改法够得着两形；邻形推的 | 判决里「HEAD 判法下同样漏」一句是主 agent 自己标了推的。改法三条分支（候选带条件、候选所在模块另有 use 引进同名、相对路径首段被别的 use 引进）在 `closure.rs:1881` 那处 `if exact_module.is_none()` 之后生效；`is_imported_by_another_use` 只数「按名字引进」，glob 不算。邻形：首段别名由 `use hub::*` 引进、同时另处有同名真模块，Rust 点不准也不退回全连。夹具 `glob-imported-module-alias`：清单只有 `fglobhop::other::short::helper_total`、没有 `fglobhop::deep::real::helper_total` 就是邻形成立（推的）。 |
| 第 4 条：程序起自己，判形状的那一半（判决第 29 行） | 站得住；改法够得着；入口那一半已对齐 | 改法在 `rust_items.rs:1839` 起的「用法与 `let` 同层花括号」与 `rust_items.rs:1681` 的「块前是 `|` 或 `=>`」。Python 同形：`lib_function_units.py:3545` 的 `innermost_brace_around` 与 `expression_ending_at_diverges`。入口：Rust `workspace.rs:2025` 的 `main_function_of` 与 Python `lib_function_units.py:5179` 的 `rust_entry_is_the_main_of_a_program` 都取目标根文件里排头的顶层 `fn main`，读法同形（读码）。判决第 29 行记正推腿「Rust 比第一个顶层 main、Python 比入口本身」：现在两边都比「入口恰是那个 main」。推翻：夹具喂两份实现，清单不同。 |
| 第 5 条：extern 块只声明登记的系统库调用，两形（`cfg_attr` 里的 `link`、同包另一个带 `#[link]` 的块，判决第 30 行） | 站得住（两形）；改法够得着判决写的字面；邻形够不着（推的） | 改法：`rust_items.rs:511` 的 `extern_block_attributes_name_link` 取 `extern` 之前任何位置的 `link`；`closure.rs:2355` 的 `target_may_link_libraries_beyond_the_system_library` 看同目标另一个 extern 块与构建脚本里的那个常量（`crates/singlefs-unit-graph/src/closure.rs:911`：「const RUSTC_LINK_LIBRARY_INSTRUCTION: &str = "rustc-link-lib";」）。邻形一：构建脚本打 `cargo:rustc-link-arg=-lfakesync` 或 `cargo:rustc-flags=-l fakesync` 同样多链接一个库，名单里没有这两个词。邻形二：`crates/singlefs-unit-graph/src/closure.rs:2359`：「&& workspace.membership(unit.file, target).is_some()」只看本目标的文件，依赖 crate 里的 `#[link]` 看不到，而 `#[link]` 对最终链接的库集合生效。夹具 `link-through-rustc-link-arg`、`link-through-rustc-flags`、`link-in-a-dependency-crate` 各喂入口，没有 `extern-function-interface` 拒算行即邻形成立。Python 同形：`lib_function_units.py:2481` 的 `extern_block_attributes_name_link`。 |
| 第 6 条：首边与 `why`、不沿点名单元收、`build_features.rs`、纯改名，两条腿都没打中（判决第 31 行） | 站得住 | 按 `three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 那一条，两条腿都没打中就记没打中。判决里关于返回类型的更正我现查：`crates/singlefs-unit-graph/src/entry_groups.rs:116`：「) -> Result<EntryGroupKey, EntryError> {」，那是 `entry_group_key_with_the_features_of_the_build` 的签名末行，没换类型；新函数在 `entry_groups.rs:141`。纯改名 12 块我没逐块读。 |
| 第 7 条：Rust 与 Python 的 `use … as` 不一致，与第 1 件的白名单对黑名单（判决第 32 行） | 站得住；「白名单对黑名单」写宽了一点 | `use … as` 一支：Python 现在有同名分支 `lib_function_units.py:4722` 的 `is_imported_by_another_use`，弄坏开关在 `lib_function_units.py:718`，两边同形（读码）。白名单对黑名单：有效值行（Python `lib_function_units.py:15741` 的 `effective_profile_lines` 对 Rust `build_toolchain.rs`）已是同一个六键白名单；原文行的收法 Python 仍是黑名单，`scripts/mutation/lib_mutation_units.py:56` 自己写「比 Rust 那一份收得宽」，`admission.py:25` 同。两条是不同的键家族，不要求逐字相同，所以不算新差；但判决第三节第 1 条「同一口径一起改」只对有效值行成立。推翻：主 agent 喂含 `[profile.release.package.x]` 的夹具，两边键相同。 |
| 第 8 条：80 行变异没罩全，五处少收、起自己与 extern 各一处不拒（判决第 33 行） | 站得住（读码推的）；新写的代码上同一毛病重现 | 本轮补的行：冻结表 69 行（`mutation-rows-new.tsv`）之外还有 13 行 `构建档键…`（`build_toolchain.rs`，冻结表没收，现查 `crates/mutations.tsv`，见第三段）。新写的方括号取反分支：`grep -rn '\[!' crates/singlefs-unit-graph/tests/*.rs` 命中 0 行，`workspace.rs:1509` 起的 `bracket_expression_after_the_opening` 里 `!` 取反、`a-z` 区间、`]` 排头当字面三处，没有测试也没有变异行（行 53 只改 `bracket.admits` 的真假，不碰解析）。推翻：补一个 `[!a]?` 夹具与对应变异行后红在名字说的断言上。 |
| 第 9 条：检查单八格，多机可用不成立（判决第 34 行） | 站得住；改后残余推不出 | 判决当时的根据是 `CARGO_INCREMENTAL` 两台不同键相同，改后已读。残余是 cargo 配置通道：用户级配置是每台各自的状态，两台的 `[build] incremental` 不同时键仍相同（推的，待探针）。 |
| 第 10 条：事实表与正文写错的（判决第 35 行） | 站得住 | 现查：`crates/singlefs-unit-graph/src/rule_switches.rs:94`：「pub const EVERY_RULE: [BrokenRule; 29] = [」（判决说 28 变 29）；`build_toolchain.rs:20` 在 HEAD 里已有 `PROFILE_SETTING_KEYS`（`git show HEAD:` 量的），不是新加；`closure.rs:936` 现在是 `use_path_leading_to`，行号随改动已动，判决当时的 928 行我没法在快照树上核（没倒推快照树）。 |

## 第三节六条改法：合进来的样子

| 改法 | 做到了没有 | 证据（文件:行）与多做、少做 |
|---|---|---|
| 1：incremental 不单独出行，用生效值定 codegen-units 缺省，只出一行；断言两种编法行不同；Python 与 `lib_mutation_units.py` 同口径 | 做到；两处不及判决字面 | 键只出 `codegen-units`：`build_toolchain.rs:21` 的 `PROFILE_SETTING_KEYS` 六项无 incremental；`build_toolchain.rs:391` 起的测试断言 `assert_ne!`，并断言 256 与 16 两行。Python：`lib_function_units.py:15741` 起三层盖（内置、清单继承链、`CARGO_PROFILE_*`）再 `CARGO_INCREMENTAL`，与 Rust 同序。`admission.py` 的 `profile_lines` 与 `lib_mutation_units.py` 的 `effective_profile_lines_of_every_profile` 调同一份。少做：`scripts/mutation/lib_mutation_units.py:110` 的常量仍叫 `PROFILE_SETTINGS_NOT_AFFECTING_OUTPUT` 且含 incremental，名字说的就是被推翻的那句话（只作「原文行不进」用，行为没错，名字过时）。多做：Rust 构建脚本报 `rerun-if-env-changed` 改成对 `environment_variables_the_profile_lines_read` 逐个报，两份构建脚本各一处，有测试钉住（`build_toolchain.rs:538`、`:542` 的 `include_str!`，见检查单「放的位置」）。 |
| 2：宏括号里的名字照连，只有标准格式化与断言一族照算局部 | 做到；名单比判决点名的宽一点 | `closure.rs:1138` 的 16 项含 `panic`、`debug_assert*`、`eprint*`，判决写「格式化与断言那几个宏（`format!`、`println!`、`write!`、`assert!` 一族、`vec!`）」；多出的都是同类就地求值，不算越界。Python 同名单：`lib_function_units.py:2394`，自证读 Rust 源码逐项比，弄坏开关 `rust-macro-list-short`。16 个宏各一行变异（冻结表行 30 到 45）各点自己的测试。 |
| 3：use 路径三条分支，Rust 与 Python 对齐取多收的 | 做到 | `closure.rs:1881` 起三处退回全连；Python `lib_function_units.py:4722` 起同形。两份对拍靠主 agent 的「喂了 39 条夹具，两份不同 0 条」，那一趟在工作区成员补丁之前（正文第 36 行），补丁之后未重跑，见「要主 agent 跑的命令」。 |
| 4：起自己与 extern 两条改法，Python 同步 | 做到 | 起自己：`rust_items.rs:1839` 与 `:1681`，Python `lib_function_units.py:3545`；extern：`rust_items.rs:511` 与 `closure.rs:2355`，Python `lib_function_units.py:2481`。 |
| 5：没有变异行的分支补行，78、79 行改到红在名字说的断言上 | 部分做到；没法静态判「红在哪条断言」 | 静态核（我的 `rows_static_check.py`，量的）：冻结的 69 行里原文在文件里恰好命中一次的 69 行、点名的测试函数都在的 69 行；`--exact` 缺的 2 行（行 1 与行 69）。红不红、红在哪条断言，要靠 crates-mutation-replay 复跑；正文说跑了 21 与 22 行两趟共 43 行，剩下 26 行与那 13 行 `构建档键…` 没有读数。 |
| 6：Rust 与 Python 在第 1、4 件入口判法上对齐，夹具两边各喂一遍 | 做到（读码）；工作区成员补丁之后的夹具读数没有 | 第 4 件入口两边同形（见第二节第 4 条）；第 1 件有效值行同形。夹具对拍读数是工作区成员补丁之前的版本。 |

## 工作区成员与数据目录两份对齐补丁：同不同一条规则

先答：是同一条规则，读法逐处对得上；差只在边角，其中正文列的两处都是真差、真实仓碰不到。

| 对象 | 判定 | 证据 |
|---|---|---|
| 数据目录：折出的目录只在 `<仓根>/crates/` 之下才收 | 同一条规则 | Python `scripts/gate/lib_function_units.py:3870`：「return path.startswith("crates/") and not path.startswith("../") and os.path.isdir(os.path.join(self.root, path))」；Rust `crates/singlefs-unit-graph/src/rust_items.rs:1239`：「if folded.starts_with(&crates_directory) && folded != crates_directory {」，前面 `rust_items.rs:1333` 到 `:1347` 先判 `is_file`、`is_dir` 再进 `directory_fold`，与 Python 的 `isdir` 同序。传 `crates/` 目录当根时 Rust 取上一层作仓根，Python 没有这个入口（给它 `crates/` 收不到成员），是 Rust 多出的入口，不影响对拍。 |
| 工作区成员：`[workspace]` 有没有 | 同一条，内联空表一处真差 | Python `scripts/gate/lib_function_units.py:4141`：「if workspace:」，字典真假；`[workspace]` 空表头与 `[workspace.metadata]` 的处理 Rust 已按这条写（`workspace.rs:1388`、`:1442`）。内联空表 `workspace = {}`：Python 的 `workspace` 是空字典、判假、退回 `crates/` 下收；Rust `crates/singlefs-unit-graph/src/workspace.rs:1469`：「([], TomlValue::InlineTable(pairs)) => pairs」之前，`get_or_insert_with` 已先造出 `Some`，判有、`members` 为空、一个都不收。真差：同一棵树 Python 收 `crates/*`，Rust 收 0 个。真实仓碰不到：根清单第 4 行是带 `members` 的 `[workspace]` 表头（`grep -n '^\[workspace' Cargo.toml` 量的）。改法方向只有一条能与 Python 一致：内联表 `pairs` 为空时不建 `Some`（与同文件对空表头的注释同一条规则）；Cargo 自己对 `workspace = {}` 的读法没查，不当依据。夹具 `workspace-inline-empty-table`，两边 `dump` 的 crate 与单元行不同即差成立。 |
| 成员：`members` 通配 | 读法同一条；次序与绝对路径、`..` 不同 | 我用自己的 Python 在草稿目录建树、直接调 `glob.glob`（量的，没走项目代码）：`crates/[!a]*` 配 `zeta`、`crates/[^a]*` 的 `^` 是字面（配 `a`、`a^b`）、`crates/**` 与 `crates/*` 同、`crates/[z-a]*` 空、`[a-]` 里 `-` 是字面、`.` 起头的名字只在这一节也以 `.` 起头时才配。这些与 Rust `workspace.rs:1509` 起的 `bracket_expression_after_the_opening`、`component_pattern_matches`、`entries_matching_the_component` 逐条读下来同义。不同：Python `scripts/gate/lib_function_units.py:4144`：「for match in sorted(glob.glob(os.path.join(root, pattern))):」次序是「members 写的次序、每条通配内部按字符串排」，Rust `crates/singlefs-unit-graph/src/workspace.rs:1636`：「let mut member_directories: BTreeSet<PathBuf> = BTreeSet::new();」一律按路径排；真实仓 `Cargo.toml` 第 7 行起的成员不是字母序，而主 agent 量的「dump 逐个相同、闭包与键逐个相同」没分出次序，所以这棵树上次序不改结果；次序会不会改 crate 下标进而改输出，没有读数。测试 `unit_graph_workspace_members_come_from_the_root_manifest_as_in_the_python_implementation.rs` 里 `crates_by_package_name` 先按包名排序再比，次序不在被测范围。绝对路径成员（Python 的 `os.path.join` 丢掉仓根）与带 `..` 的成员，Rust 把前导 `/` 滤掉当相对路径，真实仓没有。夹具 `workspace-members-unsorted`、`workspace-members-negated-bracket` 喂两边。 |
| 成员：缺名的默认名（正文第二处没对齐） | 真差；真实仓碰不到 | Python `scripts/gate/lib_function_units.py:4091`：「self.package = package.get("name", posixpath.basename(directory) or "root")」；成员写成 `"."` 时 `os.path.relpath` 交 `.`、`posixpath.basename(".")` 交 `.`（量的，我的 Python 片段打出 `relpath of '.' member: . | basename: .`），名字是 `.`；Rust `crates/singlefs-unit-graph/src/workspace.rs:1684`：「Some(_) | None => "root".to_string(),」，仓根取 `root`。 |
| 成员：根清单带 `[package]` 又列 `"."`（正文没列的一处） | 真差；真实仓碰不到 | Python `scripts/gate/lib_function_units.py:4153`：「directories.insert(0, "")」，`"."` 是另一个串，`dict.fromkeys` 不合并，同一个目录收成两个 crate；Rust 以路径为键，合成一个。夹具 `workspace-dot-member-and-root-package`。真实仓根清单只有 `[workspace]`、`[profile.release]`、`[profile.dev]` 三个表头（`grep -nE '^\[workspace|^\[package|^\[profile' Cargo.toml` 量的），没有 `[package]`，成员也没有 `"."`。 |
| 两处没对齐「真实仓碰不碰得到」 | 碰不到 | 同上两行；`grep -n '"\."' Cargo.toml` 我没单跑，成员清单是第 7 行起十二条（读文件看的），没有 `"."`。 |

补充：对「同一条规则」这个判定，两份实现的一致性证据在主 agent 那一趟：合入后验证里 dump 60341 个单元逐个相同；那是对真实仓一棵树的读数，上面这些边角没有一条被它罩着，要靠下面夹具读数。

### 验证包 build.rs 的判器组键为什么变、变得对不对

只有 `observer_conclusion_composition` 一组变了（主 agent 量的，ca8c8e5e… 到 104a3724…），我读码推不出变的原因，不拿猜测当结论：

- 事实（量的）：根清单第 18 行把 `gpu-verification-server` 列进成员；Python 一直读根清单，所以那个目录在 Python 一侧一直是成员，Rust 在补丁前只收 `crates/` 下的目录，正文第 3 行记的 1509 处单元差都是只有 Python 切出的 `gpu-verification-server/` 单元。补丁之后 Rust 与 Python 在 dump 上相同。
- 事实（量的）：该组入口是 `judging_units_entries.rs` 里 `target: "lib"` 的四个 `crash_amplification` 函数；`singlefs-verification/Cargo.toml` 不依赖 `gpu-verification-server`（反过来是它依赖验证包）。所以这组闭包里出现 `gpu-verification-server` 的单元，只能经「名字或路径字面量」边，不能经依赖边，具体哪一条边我没读到。
- 方向（推的）：若 Python 一侧这组闭包本来就含 `gpu-verification-server` 的单元，Rust 是补上了 D13（验证路线）要求的「键过窄」，变得对；若 Python 一侧也不含，键变的原因在别处（例如工作区多一个依赖验证包的成员改了「依赖包开了哪些特性」那一行），要另查。分辨它的是 `why` 子命令与 Python 一侧的 `closure --explain`（见「要主 agent 跑的命令」）。推翻我这句：Python 一侧闭包不含 `gpu-verification-server/`，而 Rust 含。

## 检查单九格

| 格 | 判 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 在 `changes.patch` 与两份 Python 补丁里搜新加行（量的）：`todo!`、`unreachable!`、`unimplemented!` 0 处；`assert!` 一族只在 `#[cfg(test)]` 之后的测试代码与 `tests/` 新文件里，非测试代码里出现的 `debug_assert` 三处是宏名单里的字符串（`changes.patch` 第 555 到 557 行）。 |
| 类型与名字 | 成立，附两处更正 | 一：正文说「非测试段新加 `expect` 三处」不对，非测试段只有一处，`crates/singlefs-unit-graph/src/build_toolchain.rs:243`：「.expect("内置根 profile 的默认值里有 incremental，上面只往里盖、不删");」，消息写了依赖的不变量与理由，成立；另两处（`算得出`、`codegen-units 是进键的设置，每份生效值都有这一行`）在 `build_toolchain.rs:312` 的 `#[cfg(test)]` 之后的测试辅助函数里。二：新加 `_ =>` 五处匹配对象：`builtin_profile` 的 `&str`、`workspace.rs` 里三处（字段名 `&str`、键路径切片、`(Option<&char>, Option<&char>)`），都是开放集合，不会因新增成员漏判。三：名字，`scripts/mutation/lib_mutation_units.py:110`：「PROFILE_SETTINGS_NOT_AFFECTING_OUTPUT = frozenset(("debug", "incremental"))」名字说 incremental 不影响输出，正是第 1 件推翻的话，应改成说「原文行不进」；命名检查我没跑（归主 agent）。 |
| 不兼容旧代码与旧格式 | 成立 | 新加行里搜「早先、原先、此前、起初、最初、后来、改成了、换成了、修过、第一轮、第二轮、攻方、判决、三方、用户定案、legacy、兼容」（量的）：`changes.patch`、`python-scripts.patch`、`python-admission-break-switch.patch` 都是 0 处。`CrateOrigin::UnderTheCratesDirectory` 改名 `WorkspaceMember` 没留旧名别名，`NoCrates` 删了没留兼容。 |
| 不新建判定库 | 成立 | 新加行里搜 `sqlite`、`RocksDB`、`.tsv`、`create_dir`、`OpenOptions`、`File::create`、`fs::write`、`write_text`、`open(…"w")`（量的）：0 处；单元图只算键，不存判定。变异表 `crates/mutations.tsv` 是既有登记表。 |
| 所有任务多机可用 | 推不出 | 读环境的两处都经调用方交进来的 `environment`（`build_toolchain.rs:237`；Python `lib_function_units.py:15773`），测试用假环境，没有 `set_var`（新加行搜 `set_var`、`remove_var` 0 处）。`grep` 任务服务、`scripts/hosts`、`scripts/gate`（除两份键脚本）、`.claude/hooks`、`multi-host.env.example`、`.cargo`：没有一处设 `CARGO_INCREMENTAL` 或 `CARGO_PROFILE_*`，唯一提到 `CARGO_PROFILE_` 的是 `scripts/hosts/rocksdb-prebuilt.py:140`，读 `:354` 的字典推导看，是它编 RocksDB 时把这些变量从那一次的子进程环境里去掉、不设值（推的）。两台任务服务交下的环境因此相同；会让两台键不同的残余是用户级 cargo 配置的 `[build] incremental`、`[profile.*] incremental`（每台各自的状态，两份实现都不读，见第二节第 1 条邻通道）。本机 `ls ~/.cargo/config*` 无命中（量的），第二台没查，待主 agent 查。 |
| 测试优先多线程 | 成立 | 新加 `#[test]` 60 条（`grep -c '^+.*#\[test\]' changes.patch` 量的），都是小夹具；夹具根名带标签、进程号与进程内序号（`crates/singlefs-unit-graph/tests/common_fixture_workspace/mod.rs:154` 的 `fresh_root`），并行不撞；没有 `set_var` 与 `--test-threads` 一类串行手段；没有该切片并行而没并行的大循环。 |
| 带 --item 的命令与项 | 推不出 | 静态核（我的 `rows_static_check.py`，量的）：冻结 69 行的原文在对应文件里都恰好命中一次，点名的测试函数都在；行 1 与行 69 命令里没有 `--exact`，用的是名字过滤（会连带跑同前缀的别的测试，不是错）。漏处：冻结表只收了名字以「单元图」起头的 69 行，`构建档键…` 13 行（`build_toolchain.rs`，第三节第 1 条的变异）不在 `mutation-rows-new.tsv` 里（我用 HEAD 的 `crates/mutations.tsv` 对当前表按前四列比，量的：单元图包里新而不在冻结表里的 13 行，原文各命中一次、测试函数都在、命令里都没有 `--exact`，脚本是我草稿目录的 `rows_static_check.py`）。正文说 `--item` 交了 21 行与 22 行两趟共 43 行，冻结 69 行里另 26 行与这 13 行没有读数；`--list-items` 能不能列出它们没跑。 |
| 新崩溃流多罩了什么 | 成立 | 补丁里搜 `crash-case:`、`enumerate_crash_states`、`crash_point`（量的）：0 处；没有新崩溃枚举用例。 |
| 放的位置 | 不成立（正文「没有跨包 `include!`」写错） | 新文件都在 `crates/singlefs-unit-graph/` 的 `src/` 与 `tests/`，`.claude/gate.d/crate-roles.tsv:14` 登「测试支撑」，合 `directory-layout.md` 第 20 行的放法。但 `crates/singlefs-unit-graph/src/build_toolchain.rs:538`：「const VERIFICATION_BUILD_SCRIPT_TEXT: &str = include_str!(concat!(」与 `crates/singlefs-unit-graph/src/build_toolchain.rs:542`：「const BACKGROUND_GPU_TESTS_BUILD_SCRIPT_TEXT: &str = include_str!(concat!(」是两处跨包 `include_str!` 别的包的 `build.rs`（`.rs`，HEAD 的该文件没有，`git show HEAD:` 量的，新写的）。`.claude/rules/directory-layout.md@crates/：各包放什么`：「不跨包用 `#[path = …]`、`include!`、`include_str!`、`include_bytes!` 引另一个包目录下的源码（`.rs`、`.wgsl`）：要用另一个包的代码，在 `Cargo.toml` 里依赖它，或在自己的包里写一份。读另一个包的数据文件（`Cargo.toml`、`.md`、`.tsv`）不在此列。」登记情况：验证包那一处有放行行 `.claude/gate.d/crate-cross-references-allowed.tsv:5`：「单元图的单测读验证包的构建脚本原文，核它把 profile 读到的环境变量逐个报给 cargo」；产品 GPU 测试包那一处只在只缩不涨的 `.claude/gate.d/crate-cross-references-lag.tsv:4`：「换成放行表的一行」。门禁字面路径判不到 `concat!(env!(…))` 拼出的路径，靠人核（`directory-layout.md` 写明「拼出来的路径（运行时读文件、宏里嵌宏）跨没跨包」靠 review）。 |

## 要主 agent 跑的命令

没写模型，没有 `rerun.sh`。下面两个探针各收一个运行目录参数、开头先删那一个目录、只认 `singlefs/work` 之下以 `-run` 收尾的目录（脚本头的 `case` 判，别的退 2）。
探针的内存与时间：没跑过，推的：各是几个单文件的小编译，几秒、内存几十 MB 量级，待主 agent 填读数。
全部在仓根起（submit 不给 `--cwd`）。夹具在 `/home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures`；`<build=>` 是第 1 条 `task wait` 末行 `build=` 给的目录。夹具在这个草稿目录里，跑完这些命令之前不要清它。每条看末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`，再看日志里点名的行。

1. `task-server/task submit -- cargo build -p singlefs-unit-graph --bin unit-graph`。看：`exit=0`、末行有 `build=`。

增量编译的邻通道（第二节第 1 条）：

2. `task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/cargo-incremental-channels-probe.sh /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense-incremental-run`。看：`exit=0`；日志里九行 `probe-channel=… incremental-flags=<n>`。我推：普通 release 为 0，`CARGO_INCREMENTAL=1`、`CARGO_PROFILE_RELEASE_INCREMENTAL=true`、`CARGO_BUILD_INCREMENTAL=true`、`--config build.incremental=true`、`--config profile.release.incremental=true` 各 ≥1，普通 dev ≥1，dev 加 `CARGO_BUILD_INCREMENTAL=false` 为 0，最后一行 test 档读到 `CARGO_PROFILE_DEV_OPT_LEVEL=2` 时 `opt-level-2-flags` ≥1。后四条通道（`CARGO_BUILD_INCREMENTAL`、两条 `--config`、dev 关掉）的数与普通那一条不同，就证实邻通道成立；最后一行 ≥1 说明 cargo 让 test 档继承 dev 的环境变量设置，而 Rust 与 Python 只读叶子档名的变量（`build_toolchain.rs:237`、`lib_function_units.py:15773`）是共有的窄口径，这一轮没把它单列成打中；为 0 就撤回这句推测。
3. `task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/codegen-units-explicit-probe.sh /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense-codegen-run`。看：`exit=0`；六行 `explicit-codegen-units …`。我没有推的方向，没量过：写死 `codegen-units=16` 时开不开增量的 `.o` 数或 `value_1` 符号数不同，才说明选定的臂在写死时窄；相同则撤回第二节第 1 条第二行。

第一轮判决那八棵夹具在改法之后的读数（够不够得着被打中的形；夹具在 `/home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures`，开工时现查在；原样喂不改）：

4. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/shadow-through-macro fshadow::entry_through_the_module_path --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：组成清单里现在有 `fshadow::helper_total`（改前没有，判决第 19 行）。
5. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/shadow-through-macro fshadow::entry_inside_a_nested_function --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：组成清单里现在有 `fshadow::helper_total`。
6. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/use-cfg-swapped-re-export fswap::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：现在有 `fswap::portable::checksum`。
7. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/use-relative-path-through-module-alias falias::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：`falias::deep::real::helper_total` 与 `falias::other::short::helper_total` 都在。
8. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/relaunch-shadowed-binding frelaunch::main --target bin:frelaunch --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：`exit=3` 且拒算行点名 `current-exe`。
9. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/restart-with-argument-fallback ffallback::main --target bin:ffallback --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：`exit=3` 且拒算行点名 `current-exe`。
10. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/extern-link-through-cfg-attr fexternattr::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：`exit=3` 且有 `extern-function-interface` 拒算行。
11. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/extern-link-on-another-block fexternother::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：`exit=3` 且有 `extern-function-interface` 拒算行。
12. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/use-alias-leaf-divergence fdiverge::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：`fdiverge::first::helper_total` 与 `fdiverge::second::helper_total` 都在。
13. `task-server/task submit -- python3 scripts/gate/lib_function_units.py closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures/use-alias-leaf-divergence fdiverge::entry_point`。看：与第 12 条清单相同（Python 入口认不出就换 `crates/fdiverge/src/lib.rs::entry_point`）。

邻形夹具（第二节第 2、3、5 条；喂 Rust，读到的与「我推」不同就撤回那一形）：

14. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/glob-imported-module-alias fglobhop::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。我推：只有 `fglobhop::other::short::helper_total`、没有 `fglobhop::deep::real::helper_total`（邻形成立）。
15. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/std-named-macro-of-the-workspace fstdmacro::entry_through_the_std_named_macro --toolchain-line 'host: x86_64-unknown-linux-gnu'`。我推：没有 `fstdmacro::helper_total`（邻形成立）；对照入口 `fstdmacro::entry_without_a_binding` 有。
16. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/link-through-rustc-link-arg flinkarg::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。我推：`exit=0`、没有 `extern-function-interface` 拒算行（邻形成立）。
17. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/link-through-rustc-flags flinkflags::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。我推：`exit=0`、没有 `extern-function-interface` 拒算行（邻形成立）。
18. `task-server/task submit -- <build=>/target/debug/unit-graph closure /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/link-in-a-dependency-crate fdepmain::entry_point --toolchain-line 'host: x86_64-unknown-linux-gnu'`。我推：`exit=0`、没有 `extern-function-interface` 拒算行（邻形成立）。

工作区成员夹具（第三段；每棵两条，Rust 一条 Python 一条，比两份输出的 crate 与单元行，不同即差成立）：

19. `task-server/task submit -- <build=>/target/debug/unit-graph dump /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/workspace-inline-empty-table`。我推：没有 `falone` 的单元。
20. `task-server/task submit -- python3 scripts/gate/lib_function_units.py dump /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/workspace-inline-empty-table`。我推：有 `falone` 的单元（`workspace = {}` 判假，退回 `crates/`）。
21. `task-server/task submit -- <build=>/target/debug/unit-graph dump /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/workspace-dot-member-and-root-package`。我推：`src/lib.rs` 的单元一份。
22. `task-server/task submit -- python3 scripts/gate/lib_function_units.py dump /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/workspace-dot-member-and-root-package`。我推：同一份文件的单元出两份或 crate 名带 `.`。
23. `task-server/task submit -- <build=>/target/debug/unit-graph dump /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/workspace-members-unsorted`。看：输出行次序。
24. `task-server/task submit -- python3 scripts/gate/lib_function_units.py dump /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/workspace-members-unsorted`。我推：与第 23 条逐行相同（dump 按文件排序，成员次序不进输出）；不同即次序会改输出。
25. `task-server/task submit -- <build=>/target/debug/unit-graph dump /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/workspace-members-negated-bracket`。我推：有 `fbb`、`fcc` 的单元，没有 `faa` 的。
26. `task-server/task submit -- python3 scripts/gate/lib_function_units.py dump /home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/fixtures/workspace-members-negated-bracket`。我推：与第 25 条逐行相同。

判器组键为什么变（第三段末）：

27. `task-server/task submit -- <build=>/target/debug/unit-graph why /home/fy5090/code/singlefs singlefs_verification::crash_amplification::judge_the_images_after_the_observers_on_the_gpu --unit gpu-verification-server --toolchain-line 'host: x86_64-unknown-linux-gnu'`。看：有没有行、经哪条边头一回进闭包。
28. `task-server/task submit -- python3 scripts/gate/lib_function_units.py closure /home/fy5090/code/singlefs singlefs_verification::crash_amplification::judge_the_images_after_the_observers_on_the_gpu --explain`。看：日志里 `gpu-verification-server/` 的 `via` 行有没有；有就是 Python 一侧一直收着、Rust 补上了，方向对。

变异行（第二节第 8 条、检查单「带 --item」）：

29. `task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items`。看：能列出冻结表 69 个名字与 13 个 `构建档键…` 名字（名字原样 `cut -f1` 自 `research/prompts/unit-graph-src-backfill-code-r2-snapshot/mutation-rows-new.tsv`）；列不出的逐个报。
30. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=crates-mutation-replay 点名 13 行构建档键变异; 范围=13 行; 为什么=冻结表没收这 13 行、复核第三节第 1 条改法的变异是否各红在名字说的断言上' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --item '构建档键把 debug 放回进键的 profile 设置' --item '构建档键把 incremental 放回进键的 profile 设置' --item '构建档键 dev 内置不开增量' --item '构建档键 release 内置开增量' --item '构建档键根清单里的 incremental 不读' --item '构建档键 CARGO_PROFILE 的 INCREMENTAL 不读' --item '构建档键 CARGO_INCREMENTAL 设 1 不强开' --item '构建档键 CARGO_INCREMENTAL 设 0 不强关' --item '构建档键认不得的 CARGO_INCREMENTAL 当关' --item '构建档键认不得的 incremental 当关' --item '构建档键写了 codegen-units 也按增量换掉' --item '构建档键增量开关取反定 codegen-units' --item '构建档键读的环境变量不报 CARGO_INCREMENTAL'`。看：成功句「抓到 13 条」，没红 0、无效 0、锚点失效 0。冻结表 69 行里 21 与 22 两趟没点到的那几行，另按名字补点一趟（点哪几行要对 `--list-items` 与两趟的命令）。

## 推翻条件（上面表里没单写的）

- 第二节第 6、9、10 条「站得住」：第 6 条被推翻要一条腿或主 agent 的夹具在那四处里喂出不同的清单；第 9 条被推翻要第 2 条命令里 `CARGO_BUILD_INCREMENTAL`、两条 `--config` 不改 `-C incremental=` 个数（那样第二节第 1 条的邻通道与检查单「所有任务多机可用」的残余一并撤回）；第 10 条被推翻要 `grep -nF` 在当前文件里命不中我引的那几行。
- 第三节 1 到 6 条「做到」：每一条被推翻要对应夹具读数与我读码的结论相反（第 4 到 13 条命令，改前后清单没翻过来）；第 5 条的「部分做到」被推翻要第 29、30 条命令列出全部名字且逐行红在点名测试上。
- 工作区成员「同一条规则」：被推翻要第 19 到 26 条命令里出现我没列的差，或我列的差不出现。
- 检查单「放的位置不成立」：被推翻要主 agent 认定 `include_str!` 读另一个包的 `build.rs` 属 `directory-layout.md` 第 24 行末句「读另一个包的数据文件」的范围；我读它的 `.rs` 后缀与那句括号里列的 `Cargo.toml`、`.md`、`.tsv` 不同，按「源码（`.rs`、`.wgsl`）」判。

## 没做什么

- 没替主 agent 撤销或维持判决，只交复核结果。
- 没往统一任务服务交任务，没编、没跑单元图与两份 Python；凡「我推」都是读码推的，判决要引的数（探针、夹具、变异复跑）一个都没复跑，命令在上一节。
- 没读禁读清单里的文件（`unit-graph-src-backfill-code-r2-opus-output.md` 与 `unit-graph-src-backfill-code-r2-opus-model/`）。
- 没攻第 5 项「`TAIL_READ_SCOPES` 补的 12 行」（不在这一份派发的四条出口里）；没逐行现判 69 行变异红不红、红在哪条断言（要复跑）；没读纯改名 12 块的原文；没跑命名检查、门禁与自证。
- 没核 cargo 文档里 `build.incremental` 与 `CARGO_BUILD_INCREMENTAL` 的优先级、`codegen-units` 缺省值 256 与 16、`CARGO_INCREMENTAL` 取 0 与 1 之外的值时 cargo 的读法：本机没有 cargo 文档（`build_toolchain.rs` 自己写了），一律是凭记忆的推，待第 2、3 条命令。
- 没改 `crates/`、`scripts/`、治理文件；`crates/mutations.tsv` 现在有别的会话在改（行号会变），我给的行号与名字以复跑时的 `--list-items` 为准。

## 草稿文件 sha256（`/home/fy5090/.local/share/singlefs/work/unit-graph-src-backfill-code-r2-defense/`）

```
2af780ccb005a30dc09577b76b3620377ee8e3b4977d9e4ede7efebf954f0363  cargo-incremental-channels-probe.sh
44129a2e556be3c48b4a21389d4904016cc76c0fa46bb9218b9caa79e5bebf98  codegen-units-explicit-probe.sh
7822f616273d3210325cdf75fdc3b8ee7cc67fbfa9bd8f4b7f44877899e4cbb2  rows_static_check.py
3dc7cdcec0b23d9fb68c2bb11e32d5924e8c159c2e7ab18c928578b9a01221f4  fixtures.sha256
```

夹具 33 个文件逐个的 sha256 在 `fixtures.sha256`（`sha256sum -c` 全部 OK）。`rows_static_check.py` 的复跑：`python3 -I rows_static_check.py <HEAD 的 crates/mutations.tsv 副本>`，读仓内文件，没跑项目代码。
