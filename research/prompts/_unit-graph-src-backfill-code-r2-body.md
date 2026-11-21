# unit-graph-src-backfill-code-r2：单元图第一轮改法与工作区成员对齐合入之后的复核（2026-11-19）

代码轮第二轮。第一轮判决 `research/prompts/unit-graph-src-backfill-code-r1-main-verification.md` 第三节第 1 到 6 条的改法已合进主工作区（Rust 一侧随单元图与重放前缀那一批补丁合入，Python 一侧 `scripts/gate/lib_function_units.py` 等五份经 `scripts/gate/hook-swap.py` 换上）；之后为门禁 code-tooling 的 function-units-two-implementations-agree 格（换上 Python 那一份之后红：「dump 有 1509 处单元两份不同」，全是 `gpu-verification-server/` 下只有 Python 那一份切出的单元）又合了两份 Rust 补丁：工作区成员照根 `Cargo.toml` 的 `[workspace]` 收、数据目录只认仓根 `crates/` 之下。这一轮把这些一起攻。

开工快照 `research/prompts/unit-graph-src-backfill-code-r2-snapshot/`：
- `crates-sha256.txt` 十三份：`crates/singlefs-unit-graph` 下改过的六份源文件与五份测试，加 `crates/singlefs-verification/build.rs`、`crates/singlefs-background-gpu-tests/build.rs`。
- `changes.patch`：这十三份相对 HEAD 的差（HEAD 那一版与第一轮开工快照十六份逐份 sha256 相同，主 agent 现核），即第一轮改法加两份对齐补丁的全部合入。
- `mutation-rows-new.tsv`：`crates/mutations.tsv` 相对 HEAD 新加的以「单元图」起头的 69 行（变异表本身不进清单，冻结成这一份）。
- `governance-sha256.txt` 四份 Python：`scripts/gate/lib_function_units.py`、`lib_function_units_selftest_fixtures.py`、`admission.py`、`scripts/mutation/lib_mutation_units.py`；`python-scripts.patch` 是换上的那一份 Python 补丁原样（五份，另一份 `scripts/mutation/crates-mutation-rows.py` 归别的会话在改，不进清单，这一轮只判补丁里它那几段）；`python-admission-break-switch.patch` 是之后补的 admission.py 开关修补。之后另换的 `lib_function_units.py` 修补（`TAIL_READ_SCOPES` 补 12 行、一个夹具改成别的模块里的同名）没留补丁，腿读清单里那一版的现文。

方案按 `crates/` 今天的实现来谈。

## 实现今天的样子（主 agent 的观测，2026-11-19 现读）

- 第一轮改法第 1 条：`crates/singlefs-unit-graph/src/build_toolchain.rs` 新加 `INCREMENTAL_SETTING_KEY`、`CODEGEN_UNITS_SETTING_KEY`、`PROFILE_SETTINGS_READ`（7 项）、`enum BuiltinProfile` 与 `builtin_profile`；生效的 incremental 依次由根清单、`CARGO_PROFILE_<名>_INCREMENTAL`、`CARGO_INCREMENTAL` 盖，定 `codegen-units` 没写时的默认值，键里只出 `codegen-units` 一行。Python 一侧 `admission.py` 的 `profile_lines` 同口径，另有构建环境行一路（两处各进一回编译档环境变量，开关 `profile-environment-unread` 两处都不读）。
- 第 2 条：`closure.rs` 新加 `STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE`（16 个），宏实参里的名字除这几个宏外照连同名函数。
- 第 3 条：use 路径点到的候选带 cfg、或那个模块另有引进同名的 use、或相对路径首段被 use 引进（含 `as`）时退回全连。
- 第 4、5 条：起自己要求用法与 `let` 同一层花括号；extern 块属性里任何位置的 `link`、同目标另一个带 `link` 的块、构建脚本里的 `RUSTC_LINK_LIBRARY_INSTRUCTION`（`rustc-link-lib`）都不放行。
- 工作区成员（`workspace.rs`）：有 `[workspace]`（表头、点分键、内联表三种写法都认，只写子表头如 `[workspace.metadata]` 也算有）时照 `members` 逐条 glob（`*`、`?`、`[…]`，带通配的段不配点起头的名字）、去 `exclude`、要有 `Cargo.toml`；没有 `[workspace]` 退回 `crates/` 下；根清单带 `[package]`（含只写表头）时仓根算成员；缺包名取目录名，仓根取 `root`；收不到成员交空（`WorkspaceError::NoCrates` 删了）；根下有 `crates/` 或 `Cargo.toml` 当仓根，都没有当 `crates/` 本身。`CrateOrigin::UnderTheCratesDirectory` 改名 `WorkspaceMember`。`manifest.rs` 新加 `parse_toml_document`（交键值与每个表头）与 `parse_crate_manifest_with_default_package_name`。
- 数据目录（`rust_items.rs` 的 `directory_fold`）：折出的目录只在 `<仓根>/crates/`（`DATA_DIRECTORIES_PARENT_UNDER_THE_REPOSITORY_ROOT`）之下、不是它本身时收成数据目录，仓根经 `RustFileInput`、`RustScanner` 新字段 `repository_root` 交进来。
- 已知没对齐、交这一轮的两处（实现员报告 `~/.local/share/singlefs/work/unit-graph-workspace-members/result-third.md`「没对齐、交主 agent 的」）：内联空表 `workspace = {}` Python 当没有 workspace、Rust 当有；`members` 写成 `"."` 这类时缺名默认名两边可能不同（推的，没量）。

## 这一轮攻什么

1. 第一轮改法第 1 到 5 条合进来的样子：每条是不是关掉了第一轮判决第二节点名的那一形（夹具在 `~/.local/share/singlefs/work/unit-graph-src-backfill-code-r1-attack/fixtures`），有没有新开的少收或误拒。
2. 第 6 条：Rust 与 Python 两份在第 1、4、5 件与工作区成员、数据目录上是不是同一个判法；找一个两份给出不同闭包或不同键的输入就是打中。上面列的两处没对齐的判它们是不是真差、真实仓碰不碰得到。
3. 工作区成员与数据目录那两份补丁：glob 的读法与 Python `glob.glob` 对不对得上；把 `gpu-verification-server` 收成成员之后，验证包 `build.rs` 算的判器组键（只有 `observer_conclusion_composition` 一组变了，ca8c8e5e… 到 104a3724…，主 agent 量的）为什么变、变得对不对。
4. 第一轮判决第二节第 8 条点名的没有变异行的判定分支，这一批补了没有；69 行新变异（`mutation-rows-new.tsv`）每行红在不在名字说的那条断言上（第一轮第 78、79 行那一类）。
5. Python 一侧补的 12 行 `TAIL_READ_SCOPES` 登记：每一处是不是真只读调用方的输入或自证临时目录，有没有该拒算而登进去的。

第一轮已攻过而没打中的角度不重复：第 6 件首边与 `why`、第 7 件不沿点名单元收、第 8 件 `build_features.rs`、第 9 件纯改名、四处 `expect` 的「栈不空」。

## 验证读数（主 agent 经任务服务跑出的）

- 合入后验证（`scripts/gate/merge-and-verify.sh`，两份工作区与数据目录补丁，点 4 个测试目标）：「验证红 0 项」；在它的快照上：两份实现一致那一格「dump 比了 60341 个单元（Rust 58086、WGSL 2082、数据 173），逐个相同；登记入口 10 个（stage-inputs.tsv 18 个），闭包与键逐个相同」；crates-mutation-replay 点 22 行「抓到 22 条、没红 0 条、无效 0 条、锚点失效 0 行」、基线不绿 0 行；两条命名检查与 `crash-case-check.py`、`refused-test-registry.py list` 退 0。之后合进主工作区，`apply-writer-patch.py --read-outcome` 退 0，`code-source-discipline.sh --check mutation-tables` 退 0。
- Python 一侧：`lib_function_units.py --selftest`「691 格都对……弄坏开关 298 个各红在点名的格」；`admission.py` 自证退 0、break-switch-replay 点它「复跑了 99 条开关（1 份），自证红且点名的格与实际红的格逐格相同 99 条」；`crates-mutation-rows.py --selftest` 退 0、它的两个新开关各只红点名那一格；夹具对拍「喂了 39 条夹具，两份不同 0 条、预期不符 0 条」（单元图换上 Python 之后、工作区成员补丁之前那一版）。

### 检查单：走不到的理由二次查实

本轮改动没有这一类：三份实现员报告「走不到的理由」都写「无」（`unit-graph-workspace-members/result-third.md`、`unit-graph-data-directory-under-crates/result.md`）；单元图是测试支撑包（`.claude/gate.d/crate-roles.tsv:14`），没有新加 `todo!`、`unreachable!`、`assert!`。腿在 `changes.patch` 非测试段里搜一遍这几样。

### 检查单：类型与名字

新加的名字见「实现今天的样子」；`changes.patch` 非测试段新加 `expect` 三处（`build_toolchain.rs`：「内置根 profile 的默认值里有 incremental，上面只往里盖、不删」、「算得出」、「codegen-units 是进键的设置，每份生效值都有这一行」），全补丁新加 `_ =>` 5 处。腿判「算得出」这条消息说没说出依赖哪条不变量、5 处 `_ =>` 匹配的是不是开放集合、新名字说不说得出它是什么。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：没有认旧版本的分支或旧布局号；`NoCrates` 错误成员删了而不是留着兼容。腿在补丁里搜。

### 检查单：不新建判定库

本轮改动没有这一类：单元图只算键、不存判定；Python 一侧判定照旧经 prune-server。腿搜新开的目录、TSV、sqlite、RocksDB。

### 检查单：所有任务多机可用

新测试的夹具都是测试源码里的字面量，写进 `FixtureTree` 的临时目录（实现员报告「拒算的测试」一节：无）；生效 incremental 读 `CARGO_INCREMENTAL` 与 `CARGO_PROFILE_<名>_INCREMENTAL`，两台由服务交下来的环境是否相同，腿判这会不会让两台算出不同键。

### 检查单：测试优先多线程

新测试是小夹具，每条几毫秒（点名跑「9 passed … finished in 0.00s」「10 passed … finished in 0.05s」），不走崩溃枚举。腿判有没有该切片并行而没并行的。

### 检查单：带 --item 的命令与项

实现员报告里的 crates-mutation-replay `--item` 主 agent 照原样交了（21 行、22 行两趟）；腿核 `mutation-rows-new.tsv` 69 行能不能被 `--list-items` 列出、两趟有没有漏点这一批新写的行。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新加崩溃点重放流或崩溃枚举用例（补丁里没有 `crash-case:` 登记行，也没有调 `enumerate_crash_states` 一族的测试）。

### 检查单：放的位置

新增与改动的文件都在 `crates/singlefs-unit-graph/`（`src/` 与 `tests/`）与两份既有的 `build.rs`；单元图在 `.claude/gate.d/crate-roles.tsv:14` 登「测试支撑」；新测试住它自己的 `tests/`，没有跨包 `include!` 或 `#[path]`。腿核合不合 `.claude/rules/directory-layout.md`「crates/：各包放什么」。
