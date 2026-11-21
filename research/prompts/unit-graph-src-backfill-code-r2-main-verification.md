# unit-graph-src-backfill-code-r2：判决（2026-11-19）

正文 `research/prompts/_unit-graph-src-backfill-code-r2-body.md`，开工快照 `research/prompts/unit-graph-src-backfill-code-r2-snapshot/`。这一题的第二轮；第一轮判决 `research/prompts/unit-graph-src-backfill-code-r1-main-verification.md`，它第三节第 1 到 6 条的改法与之后为两份实现一致那一格合的工作区成员、数据目录两份补丁，合进主工作区后这一轮一起攻。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方 | `research/prompts/unit-graph-src-backfill-code-r2-opus-output.md`（模型目录同名 `-opus-model/`） | 第一轮八形「推的关了」；改法新开三处少收（同一条 use 里改名再走相对路径、与标准库宏同名的 `macro_rules!`、extern 只看名单块所在的目标）；profile 链少 `build.incremental` 与 `CARGO_BUILD_INCREMENTAL`、非路径依赖不增量；工作区成员七形两份不同；组键为什么变推不出；69 行变异抽 37 行都红在点名断言上；`TAIL_READ_SCOPES` 加 15 删 3、两行登法不合文件头；检查单「多机」不成立、「带 --item」推不出 |
| 辩方 | `research/prompts/unit-graph-src-backfill-code-r2-sonnet-output.md` | 第一轮判决十条站得住，第 7 条写宽一点；改法够得着被打中的那一形，邻形没够着（增量邻通道、写死 `codegen-units` 时增量不进键、标准宏同名、glob 引进的首段、`rustc-link-arg`、`rustc-flags`、依赖里的 `#[link]`）；工作区成员两处真差另加一处；检查单「放的位置」不成立、「多机」与「带 --item」推不出 |

主 agent 的核查（2026-11-19 现读、现跑）：

- 开工快照 `sha256sum -c` 两份清单（`crates-sha256.txt` 十三份、`governance-sha256.txt` 四份）都 OK。
- 引文现读：`scripts/gate/admission.py:873` 原文「"CARGO_BUILD_INCREMENTAL": "只定增量编译的缓存，编出来的代码一样"」；`crates/singlefs-unit-graph/src/build_toolchain.rs:538`、`:542` 两处 `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../singlefs-verification/build.rs"))`、`…/../singlefs-background-gpu-tests/build.rs`，前一处登在 `.claude/gate.d/crate-cross-references-allowed.tsv`，后一处登在只缩不涨的 `.claude/gate.d/crate-cross-references-lag.tsv`（理由「拆包时改指产品 GPU 包的 build.rs」）。
- 判决引的读数都由主 agent 经任务服务跑出：
  - 攻方复跑包 `research/prompts/unit-graph-src-backfill-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh`：type=succeeded exit=0，「frozen_check files_ok=17 verdict=ok」。
  - 辩方两份探针 `cargo-incremental-channels-probe.sh`、`codegen-units-explicit-probe.sh`：都 type=succeeded exit=0。
  - 辩方夹具驱动 `run-defense-fixtures.sh`（主 agent 把辩方第 14 到 28 条并成一份）：type=succeeded exit=0；其中两条 `why`（第 27、28 条）退 2，原文「入口……在这组 cfg 下不编进来」，命令写错，组键那一问改看攻方的成员 A/B。
  - `crates-mutation-replay.sh --list-items`：type=succeeded exit=0，冻结的 69 个名字逐个列得出、「构建档键」14 处命中、「锚点失效 0 行」。
  - crates-mutation-replay 点 13 行「构建档键」：「抓到 13 条、没红 0 条、无效 0 条、锚点失效 0 行」，基线不绿 0 行。

## 二、判决

1. **第一轮八形**：关了。攻方复跑两份各喂一遍：`r1-macro-path`、`r1-macro-nested`、`r1-cfg-swap`、`r1-module-alias`、`r1-alias-leaf` 两份都 `found_lines=1`；`r1-relaunch`、`r1-fallback` 两份都拒算 `current-exe`（Rust 退 3、Python 退 4）。第一轮第 7 条「`use … as` 那一支两份不一致」对齐了（`r1-alias-leaf` 两份相同）。
2. **同一条 use 里先改名再走相对路径**（攻方）：打中，坐实。`case=same-use-one-use`、`case=same-use-macro` 两份都 `found_lines=0`，对照 `same-use-two-uses` 两份 `found_lines=1`；运行时探针三种写法都调到那个函数（「through_two_uses=7」「through_one_use=7」「through_a_macro=7」）。两份同一个洞。
3. **与标准库宏同名的工作区 `macro_rules!`**（攻方、辩方）：闭包两份都不连（`std-name-format`、`std-name-vec` 两份 `found_lines=0`，对照 1）；运行时那一半没量到：攻方的运行时探针编不过（「runtime_compile probe=shadow-through-a-macro-named-like-a-standard-one exit=1」）。核不动，不采；记进第三节第 6 条。
4. **extern 只看名单块所在的目标**（攻方，辩方另报两形）：打中，坐实。`extern-test-target`、`extern-dependency`、`extern-build-helper`、`extern-cc-crate` 两份都不拒算；链接探针「fexternfinal …4242」「fexterndep …4242」「fbuildlink …4242」「fcclink …4242」对照「fexternclean … -1」，运行时调到了别的库。辩方的 `link-through-rustc-link-arg`、`link-through-rustc-flags`、`link-in-a-dependency-crate` 三棵同样不拒算（闭包里只有 `extern_block` 与入口）。
5. **profile 第 1 件**（两条腿）：打中，坐实三处。
   - 增量的开关通道：辩方探针「release-CARGO_BUILD_INCREMENTAL=true incremental-flags=1」「release-config-build.incremental=true incremental-flags=1」「release-config-profile.release.incremental=true incremental-flags=1」「dev-CARGO_BUILD_INCREMENTAL=false incremental-flags=0」；攻方「setting=cargo-build-incremental-false … member_incremental=no … key=41923825…」与默认那一行键相同。Rust 与 Python 两份都不读这几条；`admission.py:873` 还把 `CARGO_BUILD_INCREMENTAL` 写成「编出来的代码一样」。
   - 写死 `codegen-units` 时增量照样改产物：辩方探针「explicit-codegen-units opt-level=0 incremental objects=32」对「plain objects=16」，「opt-level=3 incremental objects=32」对「plain objects=1」。第一轮用户定案第 38 条「incremental 不单独出行」的前提（写了 `codegen-units` 就与增量无关）在这一形上不成立。
   - 非路径依赖的切法：攻方「setting=explicit-codegen-units-256 … dependency_codegen_units=256 … key=41923825…」，与默认（`dependency_codegen_units=unset`）键相同。
   - 攻方另报的 cargo 配置 `build.incremental=false` 那一行「member_incremental=yes」，与默认相同，不采。
   检查单「所有任务多机可用」因此不成立。
6. **工作区成员两份不同**（两条腿）：打中，坐实六形，都是今天的根清单碰不到的。攻方 dump 对拍：内联空表（Rust 0 行、Python 1 行）、多行串（0 对 1）、`"."` 成员缺名（`identical=no`，`case=dot-unnamed` Python 退 2）、根路径带方括号（1 对 0）、绝对路径成员（0 对 1）、目录同名（`case=same-name-library` Python 退 2）、嵌套成员的文件 Rust 收两遍（「nested_member_file_lines rust=2 python=1」）。辩方四棵里只有内联空表那一棵单元行不同；`"."` 成员加根 package、成员乱序、`[!…]` 取反三棵单元行两份逐行相同，只差汇总行，辩方「Python 收两份」那一形不采。
7. **glob 引进的首段**（辩方）：没打中。夹具闭包两个同名 `helper_total` 都收了（多收方向）。
8. **`observer_conclusion_composition` 组键为什么变**：推不出，记进第三节第 6 条。攻方成员 A/B：「members_ab arm=with-the-server … key_line=be4e3891…」与「without-the-server … key_line=be4e3891…」相同、「only_with_the_server=0 only_without_the_server=0」，`gpu-verification-server` 进不进成员不改这一组的键；主 agent 量的「前」那一份之后主工作区被别的会话改过，哪一处改动让它变的没找出。
9. **变异行**：第一轮第二节第 8 条七处分支都补了（攻方逐处对上）；69 行全部列得出、锚点失效 0 行；13 行「构建档键」抓到 13 条。攻方备注三处成立：第 60、62 行红在测试辅助的 `expect` 上，第 28 行名字说的与测试抓到的坏法不同，第 16 行的测试分不出点准与退回全连。
10. **`TAIL_READ_SCOPES`**（攻方）：13 行成立；`crate-roles-check.py` 那两行读的是被判那棵树的清单，登成空元组与文件头「读的是调用方交进来的输入的登空元组」不合，实害推的没有。正文写「补 12 行」不对，相对 HEAD 加 15 行、删 3 行。
11. **检查单**：走不到的理由、不兼容旧代码、不新建判定库、测试优先多线程、新崩溃流五格两条腿都判成立。类型与名字成立，正文写错一处：新加 `expect` 在非测试段只有 `build_toolchain.rs` 一处，另两处在 `mod tests` 里。所有任务多机可用不成立（第 5 条）。带 --item 的命令与项成立（69 行与 13 行都列得出，13 行这一趟复跑抓到 13 条）。放的位置：`build_toolchain.rs:542` 引产品 GPU 测试包的 `build.rs` 是这一批加的跨包引用，登在只缩不涨的滞后表，去向是拆包时改指产品 GPU 包；辩方判不成立、攻方判成立，主 agent 判「成立，已登滞后表」，不另改。

## 三、改法

都改在单元图与 Python 两份算键实现上，两份同一个判法；改完照代码轮收法再攻一轮（第三轮）。

1. 同一条 use 里先改名再走相对路径、宏展开出的改名：首段检查不把路径所在的那条 use 自己排掉；第 2 条三棵夹具两份都 `found_lines=1`。量过。
2. extern：链进最终产物的库算全，同一个包的任一目标、它的依赖（含构建依赖，如 `cc`）里有 `#[link]` 或构建脚本打 `rustc-link-lib`、`rustc-link-arg`、`rustc-flags` 带 `-l` 的，不放行；第 4 条七棵夹具两份都拒算 `extern-function-interface`，`extern-control` 照旧不拒。量过。
3. 工作区成员：六形两份对齐，取多收或拒算的那一边（宁可多收、不静默少收）；夹具两份 dump 逐行相同。量过。
4. profile：生效的增量补读 `CARGO_BUILD_INCREMENTAL` 与 cargo 配置的 `build.incremental`、`profile.<名>.incremental`；`admission.py:873` 那一句与钉它的自证格改掉。写死 `codegen-units` 时增量怎么进键、非路径依赖的切法怎么进键，交用户（与第一轮用户定案第 38 条冲突）。
5. 变异：第 60、62、28、16 行改到红在名字说的那条断言上。`TAIL_READ_SCOPES` 两行照文件头的登法改。
6. 记账不改的：与标准库宏同名的 `macro_rules!` 运行时调不调得到同名函数（核不动）；`observer_conclusion_composition` 组键变化的来由；两处登进下一轮正文「这一轮攻什么」。

## 交用户的

| 问题 | 依据 |
|---|---|
| 写死 `codegen-units` 时，增量开关还要不要进键（第一轮定案第 38 条说「incremental 不单独出行」） | 第二节第 5 条：辩方探针 opt-level=0 32 对 16、opt-level=3 32 对 1 个目标文件 |
| 非路径依赖（registry 包）不走增量、它的 `codegen-units` 默认值随档变，依赖一侧的切法要不要另出一行进键 | 第二节第 5 条：攻方「dependency_codegen_units=256」与默认键相同 |

用户定案（2026-11-19 弹窗）：写死 `codegen-units` 时增量照样进键，选「生效增量也单独出一行」；依赖一侧选「另出一行进键」。第三节第 4 条照这两条改：键里出生效 `codegen-units`、生效 `incremental`、依赖一侧生效 `codegen-units` 三行，第一轮用户定案第 38 条「incremental 不单独出行」作废。

## 这一轮被判过的文件

- crates/singlefs-background-gpu-tests/build.rs：判了它把 profile 读到的环境变量逐个报给 cargo（第一轮第 1 条那一路）；打中缺 `CARGO_BUILD_INCREMENTAL` 与 cargo 配置那几条（第二节第 5 条）。
- crates/singlefs-unit-graph/src/build_toolchain.rs：判了生效 incremental 与 `codegen-units` 默认值；打中第二节第 5 条三处；`:542` 跨包 `include_str!` 已登滞后表。
- crates/singlefs-unit-graph/src/closure.rs：判了宏实参照连、use 路径点准与退回全连；打中同一条 use 改名那一形（第二节第 2 条），标准宏同名核不动。
- crates/singlefs-unit-graph/src/manifest.rs：判了 `parse_toml_document` 与缺名默认名；打中多行串、内联空表两形（第二节第 6 条）。
- crates/singlefs-unit-graph/src/rust_items.rs：判了起自己、extern 与数据目录；打中 extern 只看名单块所在目标（第二节第 4 条），数据目录没打中。
- crates/singlefs-unit-graph/src/units.rs：判了拒算种类的新成员，没打中。
- crates/singlefs-unit-graph/src/workspace.rs：判了成员收法、glob、仓根判法；打中第二节第 6 条六形。
- crates/singlefs-unit-graph/tests/unit_graph_local_bindings_shadow_same_named_functions.rs：判了宏实参那几条用例，没打中。
- crates/singlefs-unit-graph/tests/unit_graph_manifest_directory_reads_under_crates_become_directory_data_units.rs：判了数据目录用例，没打中。
- crates/singlefs-unit-graph/tests/unit_graph_own_program_restart_is_keyed_only_from_the_main_of_a_bin.rs：判了起自己那几条用例，没打中。
- crates/singlefs-unit-graph/tests/unit_graph_shapes_that_hide_the_callee_widen_the_closure_or_refuse_the_key.rs：判了点准、extern 用例；同一条 use 改名与 extern 跨目标两形没有用例（第二节第 2、4 条）。
- crates/singlefs-unit-graph/tests/unit_graph_workspace_members_come_from_the_root_manifest_as_in_the_python_implementation.rs：判了成员用例；第二节第 6 条六形没有用例。
- crates/singlefs-verification/build.rs：同产品 GPU 测试包那一份，打中缺增量通道。
- scripts/gate/lib_function_units.py：判了 Python 一侧同一批判法与 `TAIL_READ_SCOPES`；打中第二节第 2、4、6 条（两份同一个洞或两份不同）与两行登法。
- scripts/gate/lib_function_units_selftest_fixtures.py：判了新夹具，没打中。
- scripts/gate/admission.py：判了 `profile_lines` 与构建环境行；打中 `:873` 那一句与第二节第 5 条。
- scripts/mutation/lib_mutation_units.py：判了编译档设置的取舍，打中同第二节第 5 条（辩方第 7 条写宽那一句：原文行的收法仍是黑名单，`:56`）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 不影响 | 2026-11-19 不受影响：打中的都是键算法的实现没做到已定项 17 写的「生效构建环境」与「不静默少收」，条款不改；交用户的两条是第一轮用户定案第 38 条的口径 |
