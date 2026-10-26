# m3-close-gov-b17-r3 正推腿（Sonnet）报告：K1、K4、K5（2026-10-24）

腿：云端正推（`three-way-forward`）。没读禁读清单里的另一条腿的报告与模型。所有数是这一趟现跑的；机器上有别的会话的 e165 装置占满 CPU（负载 100–285），所以计时只作参考。

## 复跑与文件指纹

模型目录 `research/prompts/m3-close-gov-b17-r3-sonnet-model/`（只算 python，不起 cargo、不编译；整套约 35 分钟，机器空闲时）。判决复跑：把目录拷进草稿目录再跑，不在原目录里跑：

    bash scripts/hosts/run-with-memory-cap.sh 16G bash <副本>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-gov-b17-r3-snapshot --work <新的草稿目录>

末尾应打八行 `name=rerun_compare file=… result=identical`（`k1-judging-digests`、`k5-units-by-file`、`k5-units-by-file-included`、`k1-resolve-summary`、`k1-direct-compare`、`k1-plan-summary`、`k1-wgsl-probe`、`k4-classify`）。内存上限 16G 没量过峰值（plan 起 16 个算键子进程）。`RERUN_PARTS="trees k4"` 可以只挑几部分冒烟。读主工作区现文：开工后被别的会话改过的文件（下面「没做什么」第 6 条）会让对应的数变，比对出 DIFFERENT 时先看是不是那几份。整趟 `rerun.sh` 我没从头到尾跑过一遍：它调的 `k1_run_all.sh` 是分段（每段一次前台命令）跑的，`out/` 就是这样得来的；整趟只冒烟过 `trees k4`（八行里 `k4-classify` 一行 identical，其余 SKIPPED）。

文件 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里）：

    2a06e459b518eb155355fa5d55d43e0b558ae7f665b3020ceb1a6adf4ee76774  ./out/evidence-k5-citecheck-r2.txt
    265519e1272c69201a1adf9c6c68e4da15d7e649aa712b842a43c055a4d3d891  ./out/evidence-k5-guard-selftest.txt
    3d23309c786fd06989f9d53796961cc2de59c54a1348d98804466d882b433422  ./out/evidence-k5-judging-units.txt
    170f4d9f23cc10d7750325d03d679b42c53e13d4e9899a29044f053c66c9d005  ./out/evidence-k5-lib-heavy-selftest.txt
    b3fd705d78763f6346b1669acba69fd9406cc32c9303d2298328eb229f0f353e  ./out/evidence-k5-r2-opus-rerun.txt
    7a2023bf39b7666ecb00d172d0c54790b8d52cca790f49994fa32f032c135e2c  ./out/evidence-k5-r2-sonnet-rerun.txt
    2729375f8831b4151d0ee888ee988785a95f773e8bc7895c782f3cbf7a56fd5d  ./out/evidence-k5-selftest-main.txt
    d26337d9b2fce989369e214cff8ec82417a76e5d3cb227a8c033eeafa3e952a1  ./out/evidence-k5-snapshot-tree-r2.txt
    1dc5ee7539f69a46bd05d8c69191f8639c19b633fa25d60a5fae608b999d09d2  ./out/k1-direct-compare.txt
    5298e9620e83a04e54dcd5e5b96a6c2d7fdcd9755c15270f804da508748626ab  ./out/k1-judging-digests.txt
    1ec0ea43008a4e1ad3e661719b039dd099b495e438f9e25263273d1f6e4c6cd3  ./out/k1-plan-summary.txt
    6daece46cf59e2dc630450cb1e58b5c47804c3f745e6c35852d934e51fb3f4b1  ./out/k1-resolve-summary.txt
    b5b2d91bb9fc8bd2343576768f80e44b424a793ccde0a128e4329efc56d02b57  ./out/k1-wgsl-probe.txt
    737c9f8ba627e04a66384cc5f715ad5774eb52cf3227f77ede17ad1be76302e2  ./out/k4-classify.txt
    cc72d60dd456dc47e355f92581bf477bc0eab6d82b40ea5fb9566fc9dcb38a8c  ./out/k4-patch-reverse.txt
    2bdbec7c7b66721d340572c6f664d0131f0bd8e1fa8b7ac0e17e95635f1d559d  ./out/k5-units-by-file-included.txt
    d058fd83baeb165e5f120ea029212a96341e16a3a21882098f35a2acc43c4106  ./out/k5-units-by-file.txt
    c26eeb08352b5ecb586125d4ed0d2a768adc7b34d2c84cf55de1c3f39e404ba1  ./tools/k1_common.py
    0917336af7a628f76e2888e51196f233d18cef13fabd05a51076cf30f03fb5f1  ./tools/k1_compare.py
    8fd0661f3b4ea4d69fa09a325c1279ab8aba6d1b1950fcc7748c8236a02be6fd  ./tools/k1_direct.py
    ccae5d8906c95b13c83d6c1f6786a0dfcb88efce2ae509b7c7e19c3191107b01  ./tools/k1_edits.py
    027073c447f3e4fe494f11b73bce07e71f621f23317da980d18e17402c4340cf  ./tools/k1_pick.py
    533ecbfbe85ca4523aaccc3c561f60d326824da0311893ccbba32ed169cb7cf4  ./tools/k1_plan_summary.py
    a0761dc999576ca720876e426cf638765d745d8f36b0649e5b9a76802d9ef725  ./tools/k1_resolve_all.py
    cddd200c29d853b2c6eb03d4c444104c10857508395a7e00872f2027184db0b8  ./tools/k1_resolve_summary.py
    34eaec7edf9298c8387a8dba2a3243f785d0ca717f8e918c76eb5bdd0e9bff24  ./tools/k1_run_all.sh
    a9518ba04650ace8d0852a4273a67102f07f8df51a1d7a16d3d74dca0560226d  ./tools/k1_seed.py
    561e80beb14eba46b0c5019cb2e8da2f8147d1f5f1d1c4f7e6a34ea2b33555f7  ./tools/k1_trees.sh
    373f96f65b818cea930a60b28978a7a61e6313ae0d57686e6ff536d62c33abf2  ./tools/k1_wgsl_probe.py
    5a532754ad579cd5671bbf9ef2f8fe970daaaacefc093f447c0c8d8959d065b7  ./tools/k4_classify.py
    feeb6bc4cee9e4c23ac3154100d715e7955d1e194b8b651088814e9921d9337b  ./tools/k4_patch_reverse.sh
    54bc58d3c614da9aabe60520153744cbb8d164d82feac31b1a914890de42dfb4  ./tools/rerun.sh

本报告引的、不由 `rerun.sh` 重新生成的实测原样输出在 `out/evidence-*.txt`（草稿目录路径已换成 <草稿目录>，家目录写成 ~）。

## K1 文件级排除在 crates 变异判法上成不成立

**结论：成立**（取的判据：D13 已定项 17 文件级排除的角色句 + 「单改 lfu 一处、树不动，判定或复用不许变而键不变」）。有一处代价不推翻它、但要登记：`lib_function_units.py` 写窄了指纹（量到的例子 E5）时，库层看不见「这次改动与同窗的树改动」（下面「代价」）。翻面的观测、推翻它的现象在本节末。

定案原文（`.claude/kb/decisions/13-验证路线.md:435`）：「管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键」。同一条里「键过窄」归违反（`.claude/kb/decisions/13-验证路线.md:433`：「键过窄（引用的常量、类型、宏漏收）」）。

### 路径 1：读代码，lfu 在判法里干了哪几件事（没有改动、没有运行）

lfu = `scripts/gate/lib_function_units.py`。`lib_mutation_units.py` 按路径载入它（`scripts/mutation/lib_mutation_units.py:78` 的 `FUNCTION_UNITS_PATH`、`:128` 的 `spec_from_file_location("lib_function_units_for_mutation_units", FUNCTION_UNITS_PATH)`）。

| 件事 | lfu 入口 | 调用处 | 输出流到哪 |
|---|---|---|---|
| 切文件成单元、取去注释指纹 | `RustFile`（scripts/gate/lib_function_units.py:1036）；指纹取词法单元 `pieces = self.texts[start:end]`（scripts/gate/lib_function_units.py:1082） | `mutated_units`（scripts/mutation/lib_mutation_units.py:310）、`resolve_test`（:406）、`executed_units`（:536）、`current_fingerprints`（:518） | 行键「被改坏的单元」组（scripts/mutation/crates-mutation-rows.py:1277）；基线键「test-unit」（:1284）与覆盖号（:1287）；覆盖记录；`executed_set_is_current`（:1366） |
| 工作区、目标、模块路径 | `RustWorkspace`（scripts/gate/lib_function_units.py:1985） | `test_units`（scripts/mutation/lib_mutation_units.py:394）→ `resolve_test` 的名字匹配（`:417`） | 点名的测试是哪个单元 → 基线键、覆盖号、行键的闭包起点 |
| 从入口收 Rust 闭包 | `rust_closure_of`（scripts/gate/lib_function_units.py:2419） | `UnitKeys.closure`（scripts/mutation/lib_mutation_units.py:502） | 行键「test」组；基线复用里「闭包号相同」那一支（scripts/mutation/crates-mutation-rows.py:1380） |
| 收 python 闭包（判法单元） | `closure_of`（scripts/gate/lib_function_units.py:7511） | `judging_units`（scripts/mutation/lib_mutation_units.py:768） | 判法单元指纹（`judging`，行键 :1277 与基线键 :1284 里都有）；排除就在这里：`:774`、`:779` |

排除的实现：`scripts/mutation/lib_mutation_units.py:774`「excluded = () if break_is("judging-files-included") else not_judging_files()」，`:779`「if unit.file not in excluded:」；名单 `scripts/crash/lib_verdict_library.py:88`「NOT_JUDGING_FILES = ("scripts/crash/lib_verdict_library.py", "scripts/gate/lib_function_units.py")」。

**判结局的函数一处不碰单元键**：`baseline_problem`（scripts/mutation/crates-mutation-rows.py:1501）、`verdict_of_run`（:1540）到下一个 `def caught_text`（:1577）之前，命令与输出：

    $ awk 'NR>=1501 && NR<1577 && /mutation_units|function_units|\bkeys\b|lfu|UnitKeys/ {print NR": "$0}' scripts/mutation/crates-mutation-rows.py
    （无输出）

所以「抓到 / 没红」只由 cargo 输出与 `verdict_of_run` 定，lfu 的输出不进去。lfu 的输出只进三样：行键、基线键 / 覆盖号、覆盖记录的逐单元比对。其中**只有第三样不是「按键取记录」**：`executed_set_is_current`（:1359）拿记录里存的（种类, 文件, 限定名, 指纹）去问 `current_fingerprints`（`:1366`「current = keys.current_fingerprints(references)」），`baseline_reuse_problem` 里它在前、「闭包号相同」在后（`:1378`、`:1380`）；覆盖号（:1287）不含判法单元指纹。这就是第二轮 H3-b 担心的缝：lfu 经 `executed_units` / `current_fingerprints` 决定基线复不复用，而这一路不挂在任何含 lfu 的键上。路径 2、3 就是去量这条缝。

此路径的前提（读的，不是跑的）：行号是主工作区现文；`broad-mutation.py` 之外的被判文件 sha256 开工后没变（见「没做什么」第 6 条）。

### 路径 2：副本里改 lfu 一处，只算 python（不起 cargo）

做法：从主工作区拷判法与 crates 要的几样成 `base`（`tools/k1_trees.sh`），变体用硬链接拷出后只换 `scripts/gate/lib_function_units.py` 一个文件的一处（`tools/k1_edits.py` 的 `EDITS`，旧串恰好命中一次才改）：

| 名 | 改的是什么（lfu 里的一处切单元的逻辑） | 预期 |
|---|---|---|
| E1 | 单元起始行取属性之后的首个词（scripts/gate/lib_function_units.py:1085 `self.lines[start]` → `self.lines[max(self.current_header, start)]`），指纹不动 | 只动行号 |
| E2 | 限定名的连接符 `::` → `.`（scripts/gate/lib_function_units.py:1085 `"::".join(context + [name])`） | 点名测试的名字匹配、覆盖记录的单元身份变 |
| E3 | scan 里「下一项从哪起」换一种等价写法（scripts/gate/lib_function_units.py:1311 `i = max(end, i + 1)`） | 输出不变 |
| E4 | 指纹不连前面的外属性（scripts/gate/lib_function_units.py:1082 `pieces = self.texts[start:end]`） | 带属性的单元指纹全变 |
| E5 | 指纹不连 `#[inline…]` 这一种外属性（scripts/gate/lib_function_units.py:1082 同一行之后加过滤） | 只有带 inline 的单元指纹变；其余窄了 |
| t1 | 树改动（不是 lfu）：`crates/singlefs-core/src/checksum.rs` 的 `fn table_index` 上加一行 `#[inline(never)]` | 该单元的指纹随属性变；它在 3838 行（plan 数）的闭包里 |

步骤 A，整张变异表问每个变体的 `UnitKeys.resolve_test` 与 `mutated_units`（`tools/k1_resolve_all.py`，命令与摘要在 `out/k1-resolve-summary.txt`）：

    base: rows=5513 resolve_refused=24 mutated_refused=0
    v-E1: resolve_changed=0 (其中新拒算 0) mutated_changed=0
    v-E2: resolve_changed=2127 (其中新拒算 2125) mutated_changed=1522
    v-E3: resolve_changed=0 (其中新拒算 0) mutated_changed=0
    v-E4: resolve_changed=5489 (其中新拒算 0) mutated_changed=1371
    v-E5: resolve_changed=0 (其中新拒算 0) mutated_changed=0

E2 的 `mutated_changed` 是单元名变了、指纹没变（行键里被改坏的单元只取「种类 + 指纹」，scripts/mutation/crates-mutation-rows.py:1277 那一句上方的 `mutated_lines`）；它的「新拒算 2125」是 `resolve_test` 在 `scripts/mutation/lib_mutation_units.py:417` 的 `full_name.endswith("::" + expected)` 对不上。E4 的 resolve 全变是 `#[test]` 单元的指纹变了。

步骤 B，50 行取样（变异表每隔 100 行取一行），`tools/k1_direct.py`：在 `base` 上给每个覆盖号造一份合成的 `llvm-cov export`（这条测试闭包里每个 fn 单元一个函数，第一段代码区域的行取 `fn` 关键字所在行），照 `judge_coverage`（scripts/mutation/crates-mutation-rows.py:3065 `executed, unplaced = keys.executed_units(export, state["work"])`）的写法造覆盖记录；各变体用同一份 export 与记录问 `executed_units`、`executed_set_is_current`、`baseline_reuse_problem`。每格是「与 base-base 不同的行数」（共 50 行）：

    label	resolve	mutated	row_key	baseline_key	coverage_id	test_closure	executed_functions_now	executed_functions_same_as_recorded	set_current	reuse_problem	reuse_problem_realistic
    base-base	0	0	0	0	0	0	0	0	0	0	0
    E1-base	0	0	0	0	0	0	0	49	0	0	0
    E2-base	23	0	22	22	22	22	22	49	43	21	0
    E3-base	0	0	0	0	0	0	0	0	0	0	0
    E4-base	50	14	49	49	49	49	49	49	49	6	0
    E5-base	0	0	0	0	0	0	0	0	0	0	0
    base-t1	0	0	33	0	0	33	49	49	27	27	33
    E5-t1	0	0	0	0	0	0	49	49	0	0	0
    E4-t1	50	14	49	49	49	49	49	49	49	6	0

后面的汇总（`out/k1-direct-compare.txt` 第 12 行起）：

    label=base-base set_current_true=43 baseline_reused_by_coverage_only=44 baseline_reused_given_record=50 baseline_reused_real_flow=49 set_current_flipped_rows=0
    label=E1-base set_current_true=43 baseline_reused_by_coverage_only=44 baseline_reused_given_record=50 baseline_reused_real_flow=49 set_current_flipped_rows=0
    label=E2-base set_current_true=0 baseline_reused_by_coverage_only=23 baseline_reused_given_record=50 baseline_reused_real_flow=27 set_current_flipped_rows=43
    label=E3-base set_current_true=43 baseline_reused_by_coverage_only=44 baseline_reused_given_record=50 baseline_reused_real_flow=49 set_current_flipped_rows=0
    label=E4-base set_current_true=0 baseline_reused_by_coverage_only=50 baseline_reused_given_record=50 baseline_reused_real_flow=0 set_current_flipped_rows=49
    label=E5-base set_current_true=43 baseline_reused_by_coverage_only=44 baseline_reused_given_record=50 baseline_reused_real_flow=49 set_current_flipped_rows=0
    label=base-t1 set_current_true=16 baseline_reused_by_coverage_only=17 baseline_reused_given_record=17 baseline_reused_real_flow=16 set_current_flipped_rows=27
    label=E5-t1 set_current_true=43 baseline_reused_by_coverage_only=44 baseline_reused_given_record=50 baseline_reused_real_flow=49 set_current_flipped_rows=0
    label=E4-t1 set_current_true=0 baseline_reused_by_coverage_only=50 baseline_reused_given_record=50 baseline_reused_real_flow=0 set_current_flipped_rows=49

读法（`set_current_true_in_base=43`：base 上 50 行里 43 行的记录马上就是「当前」；其余 7 行里 6 行的闭包带 wgsl 单元、1 行（变异表第 4906 行）行键拒算、没有覆盖记录，见「旁注」）：

- E1、E3、E5（树不动）：解析、单元身份、行键、基线键、覆盖号、基线复用，与 base 逐格相同（全 0）。E1 唯一不同的是覆盖记录里存的行号区间（`executed_functions_same_as_recorded` 49 行不同），比较时不用行号。
- E2：22 行点名测试认不出，行键、基线键、覆盖号变成空（拒算）；其余 28 行基线键不变，但覆盖那一支翻了（只问覆盖的 `reuse_problem` 21 行变），**真实写法**（记录里带闭包号，覆盖不对再比闭包号）下 `reuse_problem_realistic` 0 行变；先按基线键取记录的真实流程里，复用的基线 49 → 27，少掉的 22 条就是拒算的那 22 行。
- E4：50 行 resolve 全变，行键、基线键、覆盖号全变，真实流程复用 49 → 0。
- t1，lfu 不动（base-t1）：33 行行键与闭包号变了，真实流程里复用的基线 49 → 16。
- **E5 + t1：行键、基线键、覆盖号、闭包号 0 行变，`set_current` 0 行翻，真实流程里复用的基线仍 49**——比 base-t1 多沿用 33 条，这 33 条的点名测试执行到（或闭包够得到）被改的 `table_index`。

### 路径 3：副本里用真的 `plan` 子命令（`tools/k1_seed.py` + `plan`）

做法：用 `base` 的判法给整张表造一份「上一趟每行都判成抓到」的库（字段照 `judge_row`；5413 条，`out/` 里没存库本身，`tools/k1_seed.py` 每次现造）；再在各变体里跑真的 `crates-mutation-rows.py plan <树>`，读每行 复用 / 现判 / 拒算。另造一份「改前」的库（`GATE_MUTATION_BREAK=judging-files-included`，判法单元指纹带 lfu 那 496 个单元，961 个）作对照。判法单元指纹（`out/k1-judging-digests.txt`，节选）：

    base flag=[] c63379ff285eba00ca251e038844715f9057189fb3fca205c2985172c9d0dccc 465
    base flag=[judging-files-included] c7bc2303eee3e0ddf8d33199c67b490fa4ac48bb55ba428ac75c954c9b47da40 961

五个变体不开关时都是 `c63379ff…dccc 465`（lfu 怎么改都不动它），开关下五个变体各是一个不同的值（`out/k1-judging-digests.txt` 全文 16 行）。plan 摘要（`out/k1-plan-summary.txt`，「相对参照」= 相对 `seedbase-base-base`，同一份库、同一棵树、未改的 lfu）：

    seedbase-base-base: {'复用': 5413, '拒算': 100} 相对参照变了 {}
    seedbase-E1-base: {'复用': 5413, '拒算': 100} 相对参照变了 {}
    seedbase-E2-base: {'复用': 3352, '拒算': 2161} 相对参照变了 {'复用->拒算': 2061}
    seedbase-E3-base: {'复用': 5413, '拒算': 100} 相对参照变了 {}
    seedbase-E4-base: {'拒算': 100, '现判': 5413} 相对参照变了 {'复用->现判': 5413}
    seedbase-E5-base: {'复用': 5413, '拒算': 100} 相对参照变了 {}
    seedbase-base-t1: {'复用': 1575, '拒算': 100, '现判': 3838} 相对参照变了 {'复用->现判': 3838}
    seedbase-E5-t1: {'复用': 5413, '拒算': 100} 相对参照变了 {}
    seedbase-E4-t1: {'拒算': 100, '现判': 5413} 相对参照变了 {'复用->现判': 5413}
    seedincl-E5-t1: {'拒算': 100, '现判': 5413} 相对参照变了 {'复用->现判': 5413}
    seedincl-E3-base: {'拒算': 100, '现判': 5413} 相对参照变了 {'复用->现判': 5413}
    seedincl-base-t1: {'复用': 1575, '拒算': 100, '现判': 3838} 相对参照变了 {'复用->现判': 3838}

前缀 `seedbase` = 库按 465 个单元的指纹造；`seedincl` = 库按 961 个单元（带 lfu）造，plan 也带开关。`-t1` = 树上加了那一行属性；`-base` = 树不动。

读法：

- E1、E3、E5（树不动）：5413 行全复用，与未改的 lfu 一样。E3 是纯等价改写：排除让它**零作废**；带 lfu 的指纹（`seedincl-E3-base`）则 5413 行全现判。这是第一轮 G1-g 讲的「多作废」那一边的量。
- E2：2061 行从复用变拒算（点名测试认不出，键算不出，照跑），没有一行变成「复用却不该复用」。
- E4：5413 行全现判（键整体动了）。
- 正确的 lfu + t1：3838 行现判、1575 行复用（`seedbase-base-t1`，`seedincl-base-t1` 同数）。
- **E5 + t1：5413 行全复用**（`seedbase-E5-t1`）：该现判的 3838 行沿用了旧判定。带 lfu 的指纹下同一个变体、同一棵树是 5413 行全现判（`seedincl-E5-t1`），没漏。这是第二轮 H3-b 讲的「少作废」那一边的量。

### 三条路径有没有共用前提

- 路径 1 只读主工作区现文和 grep，不依赖任何改动、不运行。
- 路径 2 把 lfu 当黑盒改一处，直接调 `UnitKeys` 的函数与 `executed_set_is_current` / `baseline_reuse_problem`；不经 `KeyContext`、判定库、`plan`。
- 路径 3 经真的 `plan` 子命令（`KeyContext`、`row_decision`、我造的库）。
- **共用的前提**：路径 2、3 用同一组改动（E1–E5、t1）和同一棵 `base` 树。所以「lfu 还有别的改法能让某一行判定变而键不变」这一点，三条路径同样量不到；它们互不矛盾，但这一格上不算三条独立的证据。要补独立证据，得有另一个人（攻方腿）从别的改法下手，不是我再多造几种。

### 结论与依据

成立，依据每条路径一条：

1. 路径 1：lfu 的输出不进 `verdict_of_run` / `baseline_problem`（上面的 awk 无输出）；进的只有行键、基线键与覆盖号、覆盖记录的逐单元比对。
2. 路径 2：单改 lfu 一处、树不动，凡是基线键 / 覆盖号 / 行键没变的行，真实流程里的基线复用结果也没变（E1、E3、E5 全 0 行；E2 键没变的 28 行里覆盖那一支翻了 21 行、闭包号那一支兜住，`reuse_problem_realistic` 0 行变；E4 键全变）。
3. 路径 3：单改 lfu 一处、树不动，`plan` 的 5413 行里没有一行是「键没变而复用结果变」：E1、E3、E5 0 行变；E2 2061 行复用 → 拒算（键算不出）；E4 5413 行复用 → 现判（键变了）。

### 代价（不推翻，要登记）

E5 + t1 那一行：lfu 把指纹写窄（D13 已定项 17 自己归为违反，`.claude/kb/decisions/13-验证路线.md:433` 的「键过窄」）、同一窗口里树上又加了一行该进指纹的属性，库层看不见：plan 沿用 3838 行、直接调用沿用 33 条基线。带 lfu 的指纹（改前）下这一行 5413 行全现判；但同一个指纹下纯等价改写 E3 也 5413 行全现判。两头是同一张表上的量：多作废 5413 行（等价改写）对少作废 3838 行（指纹写窄 + 同窗树改动）。这一条排除在崩溃判定库上是同一个名单（`scripts/crash/lib_verdict_library.py:223`「excluded = () if "judging-files-included" in BREAK else NOT_JUDGING_FILES」），不是 crates 变异独有。lfu 写窄指纹该由 lfu 自己的自证拦下：我想核「lfu 的 `--selftest` 在 E5 下红不红」，但我起它被 `heavy-test-guard.sh` 拒了（拒绝说明写的是：命令词写成变量或当前目录认不出，解不出真实路径；它同时写明改过正文的脚本的自证不在副本里跑），没验；只做了静态 grep：

    $ awk 'NR>=9925 && NR<=10262 && /外属性|inline 属性|属性进指纹/' scripts/gate/lib_function_units.py
    （无输出）

### 翻面的观测

会让结论翻成「不成立」的，任一条：

- 单改 lfu 一处、树不动，出现 判法单元指纹、行键、基线键、覆盖号全不变而 `plan` 里某行复用 / 现判变了，或真实流程里的基线复用变了。本轮 E1–E5 出现 0 次（路径 2、3）。
- lfu 的输出进了 `verdict_of_run` / `baseline_problem` / cargo 命令行。路径 1 的 awk 无输出。
- 把「排除不得让同窗的树改动被漏掉」当作排除的前提：那 E5 + t1 的 3838 行就是翻面的观测。这一条是取舍，不是实测能定的；本腿按定案原文的角色句取判据，没按这条取。

**什么现象会推翻「成立」**：上面第一条里的任何一次；或攻方腿给出一种单改 lfu 的改法让 `executed_units` 把执行过的函数落进别的单元、而 `current_fingerprints` 仍认旧记录（我只量了 E1–E5 五种）。

## K4 修补二批对第二轮判决第二节、第三节逐项比，加实验复跑入口那一句对 `classify`

**结论**：补丁里每一处都对得上判决要的；有两处出入、一处句子比 `classify` 宽、两处复核不了。

判决（`research/prompts/m3-close-gov-b17-r2-main-verification.md`）要的：第 25 行 H1 格「改法明确的进修补批：两处重型清单补实验复跑入口；依赖版本照定案补「闭包真引到的」；G1-c 那两句：「理由不同、分开记」删（论证），「管怎么存、怎么算键，不管判对错」留（判据，H3 的裁断见下）」；第 27 行 H3 格「自证那一半写明「`lib_verdict_library.py` 对 crates 判法闭包贡献 0，改它只核指纹不变」，或换一处真在闭包里的函数，进修补批」；第 35 行去向「`implementation-workflow.md:49` 与 `heavy-test-guard.sh` 文件头类条目补实验复跑入口；`implementation-workflow.md:31` 与 `verification.md:50` 依赖版本照定案写「闭包真引到的」、删「理由不同、分开记」；H3-c 自证那一半写明或换函数」。

### 补丁文件 vs 现文（补丁倒着打，`tools/k4_patch_reverse.sh`，原样输出在 `out/k4-patch-reverse.txt`）

    patching file .claude/rules/implementation-workflow.md
    patching file .claude/rules/verification.md
    patching file scripts/mutation/crates-mutation-rows.py
    55525e09512a3b0089e1308015a28b86c78863eac942c4e6a3cb7ff142adb944  .claude/rules/implementation-workflow.md
    4fe26261d1084425696baf83cb260e9d26059df4c130998c6b0cd9781ce71a5f  .claude/rules/verification.md
    3b54387d1500a1804c0b0d2c40ac5f1842acd5117b135b0ce28171323113342b  scripts/mutation/crates-mutation-rows.py
    第二轮判决时记的 sha256：
    55525e09512a3b0089e1308015a28b86c78863eac942c4e6a3cb7ff142adb944  .claude/rules/implementation-workflow.md
    e59b30f6723738dd9e49e8c5c350a8853fac1bff43b36a41169a17a97d6b4339  .claude/rules/verification.md
    69e1bce7ed7b3cbf8032088c6a90ffde65dbf1b60fff212660ee7ccaebc830ec  scripts/mutation/crates-mutation-rows.py
    verification.md 第 35 行含「解析出的闭包真引到的依赖版本」的次数： 1
    verification.md 倒回补丁、再把第 35 行倒回「解析出的依赖版本」： e59b30f6723738dd9e49e8c5c350a8853fac1bff43b36a41169a17a97d6b4339
    等于第二轮判决时记的： True

读法：`implementation-workflow.md` 倒回补丁就是第二轮判决时那一份，所以第二轮到现在它只多了补丁里那两处，别的没动。`verification.md` 倒回补丁还差一处才回到第二轮那一份：第 35 行也是同一个替换，**不在 `governance-fix-2.patch` 里**（补丁文件只有第 50 行一处，`@@ -47,7 +47,7 @@`）。`crates-mutation-rows.py` 倒回补丁后不等于第二轮那一份（第二轮判决第一节第 1 条已记：开工后另一会话又换过一版），补丁以外的差我不核。

### 逐处

| 处 | 判决要的 | 现文 | 判定 |
|---|---|---|---|
| `.claude/rules/implementation-workflow.md:31` 依赖版本 | 补「闭包真引到的」 | 「解析出的闭包真引到的依赖版本」（grep 命中第 31 行） | 对得上 |
| `.claude/rules/implementation-workflow.md:31` 论证句 | 「理由不同、分开记」删 | 现文「算键时有两种排除。文件级」；全仓 `.claude/rules` 下「理由不同」0 命中 | 对得上 |
| `.claude/rules/implementation-workflow.md:31` 判据句 | 「管怎么存、怎么算键，不管判对错」留 | 第 31 行还在 | 对得上 |
| `.claude/rules/verification.md:50` 同上三句 | 同上 | 「两种排除：文件级是」、「解析出的闭包真引到的依赖版本」、「管怎么存、怎么算键，不管判对错」都在第 50 行 | 对得上 |
| **`.claude/rules/verification.md:35`** | 判决点名的是 `:50`，没点名第 35 行 | 第 35 行也由「解析出的依赖版本」改成「解析出的闭包真引到的依赖版本」（上面倒推出的） | **出入 1**：判决没点名的一行，同一个替换、同一个口径（快档那一行讲的也是键的组成）；不是新说法 |
| `.claude/rules/implementation-workflow.md:49` 补实验复跑入口 | 补 | 加了「实验复跑入口 `scripts/experiment-apparatus/replay.sh`（不带参数、点名实验号、`--admission-only`；`--list-items` 与登记为 light 的单独 `--selftest` 不算）」 | 补上了；与 `classify` 逐项对见下 |
| `heavy-test-guard.sh` 文件头 | 补 | 第 49 行有同一句（还多「按名字判、不读正文」） | **复核不了这一份只改了这一句**：补丁文件里没有这份文件，开工快照里只有现文的 sha256（r2 记的 `d196737f…`，r3 记的 `fa7d8acf…`），r2 那一份原样不在手 |
| `scripts/mutation/lib_mutation_units.py:36` 引文同步 | （判决没单列；H3-c 的跟改） | 同一句 15 字以上的格名在三处逐字相同（`scripts/mutation/crates-mutation-rows.py:206`、`:6885`、`scripts/mutation/lib_mutation_units.py:36`），旧格名全仓 0 命中 | 对得上；**复核不了这份文件的差只有这一行**（同上，补丁文件里没有它，r2 `8da38607…` → r3 `2ae1813f…`） |
| `crates-mutation-rows.py` 自证那一格（H3-c） | 写明「…贡献 0，改它只核指纹不变」，或换一处真在闭包里的函数 | 删掉改 `lib_verdict_library.py` 的那一半，格名里写「lib_verdict_library.py 对 crates 判法闭包贡献 0，没有函数可改」，只留改 `RustFile.row` 那一半 | **出入 2**：走的不是判决列的两条路，是第三条（删掉空测的一半）。效力核了，见下 |

**出入 2 的效力核（`out/k1-judging-digests.txt` 里 `v-cell`、`v-lvl` 四行）**：`v-cell` = 自证现在留的那一半（改 lfu 的 `RustFile.row` 一处：`0 < line` → `1 <= line`），`v-lvl` = 删掉的那一半（改 `lib_verdict_library.py` 的 `digest_text` 一处）。

    v-cell flag=[] c63379ff285eba00ca251e038844715f9057189fb3fca205c2985172c9d0dccc 465
    v-cell flag=[judging-files-included] d9788bbccb591d07cedd7265caa65bd9f6dee5282d327cc088284458b8c60bb1 961
    v-lvl flag=[] c63379ff285eba00ca251e038844715f9057189fb3fca205c2985172c9d0dccc 465
    v-lvl flag=[judging-files-included] c7bc2303eee3e0ddf8d33199c67b490fa4ac48bb55ba428ac75c954c9b47da40 961

- 留下的一半：不开关时指纹仍是 `c63379ff…dccc 465`（格要的「不变」），开 `judging-files-included` 时变成 `d9788bbc…`、不等于 base 开关下的 `c7bc2303…`（格要的「必红」）。有牙。
- 删掉的一半：两种模式下指纹都不动（`v-lvl` 两行与 base 的 `c63379ff…`、`c7bc2303…` 相同）——它在弄坏开关下也不会红，是空测，删它不丢牙。判决说的「空测」坐实。
- 代价：现在没有任何一格守「`NOT_JUDGING_FILES` 的第一项 `lib_verdict_library.py` 没被收进 crates 判法的闭包」；它今天对闭包贡献 0（`out/k5-units-by-file-included.txt` 里开关下 961 个单元也没有它），所以守不住也不需要，将来判法入口真引到它时没有格会红。判决文字里「写明」那条路也有同样的缺口，不是补丁独有。
- 整份 `crates-mutation-rows.py --selftest` 现文是 159 格全绿（`out/evidence-k5-selftest-main.txt`）。带 `GATE_MUTATION_BREAK=judging-files-included` 的整份自证我起过一次，机器被别的会话占满，235 秒超时没出结果，没有输出可贴；所以「这一格在整份自证里红」我没有整份验过，只有上面指纹层的等价核。

### 实验复跑入口那一句对 `classify`

句子（`.claude/rules/implementation-workflow.md:49`）：「实验复跑入口 `scripts/experiment-apparatus/replay.sh`（不带参数、点名实验号、`--admission-only`；`--list-items` 与登记为 light 的单独 `--selftest` 不算）」。`classify` 现判法的文字在 `.claude/hooks/lib_heavy_tests.py:621`「不算的只有摘掉 --force 之后恰好一个 --list-items 的」，实现在 `:3287` 起（比对的是 `EXPERIMENT_REPLAY_QUERY_ARGUMENTS`，`:1041` 定为 `["--list-items"]`）。逐项问 `classify`（直接问命令词）与 `classify_process`（前面加 `bash`）（`tools/k4_classify.py`，原样在 `out/k4-classify.txt`），表里 `replay.sh` = `scripts/experiment-apparatus/replay.sh`：

| 参数 | `classify` | `classify_process` |
|---|---|---|
| `replay.sh` | replay-all-stage | replay-all-stage |
| `replay.sh E142` | replay-all-stage | replay-all-stage |
| `replay.sh E142 E143` | replay-all-stage | replay-all-stage |
| `replay.sh --admission-only` | replay-all-stage | replay-all-stage |
| `replay.sh --admission-only E142` | replay-all-stage | replay-all-stage |
| `replay.sh --full` | replay-all-stage | replay-all-stage |
| `replay.sh --list-items` | None | None |
| `replay.sh --force --list-items` | None | None |
| `replay.sh --list-items E142` | replay-all-stage | replay-all-stage |
| `replay.sh --list-items --full` | replay-all-stage | replay-all-stage |
| `replay.sh --list-items --list-items` | replay-all-stage | replay-all-stage |
| `replay.sh --selftest` | None | None |
| `replay.sh --selftest E142` | commit-check-selftest-unregistered | commit-check-selftest-unregistered |
| `replay.sh E142 --list-items` | replay-all-stage | replay-all-stage |
| `replay.sh --help` | replay-all-stage | replay-all-stage |

逐项：

| 句子里的项 | 判 |
|---|---|
| 不带参数 | 对得上（重型 `replay-all-stage`） |
| 点名实验号 | 对得上 |
| `--admission-only` | 对得上（带不带实验号都重型） |
| `--list-items` 不算 | 单独（摘掉 `--force` 之后恰好这一个词）对得上，放行 |
| 登记为 light 的单独 `--selftest` 不算 | 对得上，放行；`--selftest` 后面再带实验号则不是单独：判成 `commit-check-selftest-unregistered`（提交时才跑的检查，子 agent 照拒） |
| **`--list-items` 后面还带别的词** | **句子比 `classify` 宽**：`--list-items E142`、`--list-items --full`、`--list-items --list-items`、`E142 --list-items` 全是 `replay-all-stage` 重型；句子写「`--list-items` … 不算」没有「单独」二字（同一句里 `--selftest` 前面有「单独」）。`lib_heavy_tests.py` 自己的文字是「恰好一个」。读句子的人可能把 `--list-items E142` 当放行 |
| 句子没列的 `--full`、`--help` | 都是重型；句子的括号是举例，不是封口，不算出入，但读起来像封口 |

`heavy-test-guard.sh` 文件头第 49 行的同一句同样没有「单独」二字。`classify` 这一侧没有出错，错的（宽的）是复述。

### 去向表里「写回」那一行（D13 已定项 17 现状句，中英日）

判决第 36 行：「D13 已定项 17 定案（中英日）现状句「现在只有……那一条」改成指登记位 `EXCLUDED_PROGRAM_EDGES`」。现文：`.claude/kb/decisions/13-验证路线.md:435`「边的清单以 `scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES` 登记为准，登记了的都只出提示，按这些边排出键。」；英文 `.claude/kb/decisions/13-verification-route.en.md:432`、日文 `.claude/kb/decisions/13-検証ルート.ja.md:433` 都含 `EXCLUDED_PROGRAM_EDGES`；三份里「现在只有」「only the memory wrapper」「今はメモリ」命中 0。登记位今天三条边（`scripts/gate/lib_function_units.py:7154` `EXCLUDED_PROGRAM_EDGES = {` 起，三个键）。对得上。kb 本身还写着「两种排除不进键，理由不同、分开记」（`.claude/kb/decisions/13-验证路线.md:435`）：判决只要求规则复述里删这句论证，kb 是定案原文，不动，不算出入。

### K4 小结与推翻条件

| 项 | 判定 |
|---|---|
| 补丁每一处 vs 判决 H1、去向表 | 对得上（依赖版本三处、论证句两处、判据句保留、`:49` 补入口） |
| 判决没要的 | `.claude/rules/verification.md:35` 同一替换（出入 1）；H3-c 走第三条路、删空测一半（出入 2，效力核了） |
| 实验复跑入口句 vs `classify` | 三个项、`--selftest` 项对得上；`--list-items` 不带「单独」比 `classify` 宽（后面带别的词仍重型） |
| 复核不了 | `heavy-test-guard.sh` 头这一份、`lib_mutation_units.py` 这一份「只改了点名的那一处」（开工快照没存原样）；整份自证带弄坏开关没跑出结果 |

什么现象推翻：补丁倒打后 `implementation-workflow.md` 的 sha256 不等于 `55525e09…`（现在等）；`verification.md` 倒打并倒回第 35 行后不等于 `e59b30f6…`（现在等）；任何一个项在 `classify` 里与上表不同；v-cell 在开关下指纹不变（现在变）。

## K5 第二轮报出的数，逐个现跑

数的出处：第二轮判决 `m3-close-gov-b17-r2-main-verification.md` 第 7、8、17、18 行；第二轮两腿报告；第三轮正文 `_m3-close-gov-b17-r3-body.md` 第 11、12、16 行。命令都在主工作区（没动任何文件），输出原样见 `out/evidence-k5-*.txt`、`out/k1-judging-digests.txt`、`out/k5-units-by-file*.txt`。

| 数 | 出处 | 现跑命令（范围声明略） | 现跑输出 | 判 |
|---|---|---|---|---|
| 465（判法单元指纹首行） | 正文第 12 行、r2 攻方报告第 88 行 | `python3 scripts/mutation/crates-mutation-rows.py judging-units .` | `c63379ff285eba00ca251e038844715f9057189fb3fca205c2985172c9d0dccc 465` | 核到 |
| 961 / 496 | 正文第 12 行 | 同上，前缀 `GATE_MUTATION_BREAK=judging-files-included`，加 `--units`，按文件数单元（`out/k5-units-by-file-included.txt`） | 首行 `c7bc2303…da40 961`；`496 scripts/gate/lib_function_units.py`、222、70、64、43、38、19、9，合计 961；不带开关 465 = 222+70+64+43+38+19+9（`out/k5-units-by-file.txt`） | 核到；`lib_function_units.py`、`lib_verdict_library.py` 不带开关各 0 个（后者开关下也 0） |
| 1442 | 正文第 16 行 | `bash .claude/hooks/heavy-test-guard.sh --selftest` | 「✓ 自检通过（查了 1442 种，其中该拒 274 种）」 | 核到 |
| 801 | 同上 | `python3 .claude/hooks/lib_heavy_tests.py --selftest` | 「✓ lib_heavy_tests 自检通过（查了 801 种：…」 | 核到 |
| 159 | 同上 | `python3 scripts/mutation/crates-mutation-rows.py --selftest`（经 `capped.sh 31`） | 「✓ crates-mutation-rows.py 自证 159 格都对（…」；另一行 `REUSE_SELFTEST script=scripts/mutation/crates-mutation-rows.py same-inputs=reused unrelated-change=reused unit-change=rejudged-only-affected items=5 affected=2` | 核到 |
| identical=6 differs=0 | r2 判决第 17 行、攻方报告第 13 行 | 把 r2 攻方模型目录拷进草稿目录，`bash scripts/hosts/run-with-memory-cap.sh 2G bash <副本>/opus-model/tools/rerun.sh --repo <仓根> --frozen <r2 快照> --work <新目录>` | 六行 `result=identical`，末行 `name=rerun_summary identical=6 differs=0`（`out/evidence-k5-r2-opus-rerun.txt`） | 核到（模型读的是冻结副本，今天的修补不影响它） |
| 八行 `result=identical` | r2 判决第 18 行、辩方报告第 11 行 | 同上，`sonnet-model`，内存上限 4G | 退 1；只有两行：`name=rerun_roots …` 与「.claude/rules/verification.md 里找不到「两种排除理由不同」」（`out/evidence-k5-r2-sonnet-rerun.txt`） | **复现不了**。这个模型第 1 步读主工作区现文检查补丁片段还在不在，修补二批恰好删了它检查的「理由不同、分开记」，所以今天必然退 1。不是数错了；要再现 r2 当时的八行得有 r2 当时的 `verification.md` 与另外四份文件的原样，我只还原得出其中两份（见下） |
| 13 / 14 倒推 | r2 判决第 7 行 | `python3 scripts/agents/snapshot-tree.py m3-close-gov-b17-r2 --out <草稿目录>/r2-tree` | 首行「快照目录里没有 judged-vs-head.diff 与 judged-vs-index.diff：每份都取此刻 HEAD」；末行「清单点名 14 份，倒推出 9 份、sha256 对上 9 份；倒推不出 5 份、对不上 0 份」，退 1 | **复现不了 13**，今天是 9。5 份倒推不出：`implementation-workflow.md`、`verification.md`、`heavy-test-guard.sh`、`lib_mutation_units.py`、`crates-mutation-rows.py`——前四份是修补二批动过的，第五份 r2 就已经倒推不出。所以 13 → 9 的差正好是修补二批 |
| 53 / 27 / 81（cite-check） | r2 判决第 8 行 | `python3 scripts/agents/cite-check.py <r2 两腿报告> --root <r2-tree> --background research/prompts/_m3-close-gov-b17-r2-background.md` | 「✓ 核了 53 处引文、55 处裸行号，对上 23 处，没判 85 处（其中裸行号没有引文可比 28 处；逐处列在上面）」 | 53、55、28 核到；**27 / 81 复现不了**，得 23 / 85（`out/evidence-k5-citecheck-r2.txt`）。倒推树只有 9 份原样 + 我用补丁倒推出的 2 份（`implementation-workflow.md`、`verification.md` 的 sha256 都对上第二轮记的），比 r2 当时少 `heavy-test-guard.sh`、`lib_mutation_units.py` 两份，原来对上的引文落到它们身上就变成「没判」；这个差我没逐处对到 4 处，只核了方向 |

主工作区的行号（正文第 11–12 行点的）：`scripts/mutation/crates-mutation-rows.py:510`「JUDGING_ENTRIES = (」、`:1378`「if executed_set_is_current(keys, coverage):」、`.claude/hooks/lib_heavy_tests.py:3375`「return resolved is not None and os.path.realpath(resolved)」、`scripts/experiment-apparatus/replay.sh:99`、`scripts/gate/lib_function_units.py:7154`「EXCLUDED_PROGRAM_EDGES = {」（三个键）都核到。`broad-mutation.py` 的 4322 / 4324 / 4454 在开工时的版本里核到（sha256 `7d530d22…` 的那一份：第二轮倒推树里的拷贝，`row["key"] not in current_keys` 在 4322、`gpu_key != pass_key` 在 4324、`conflicts or gpu_plan.rejected` 在 4454）；**这份文件在我跑的时候被别的会话改了**（现文 sha256 `49273352…`，同三句在 4347 / 4349 / 4479），所以主工作区现文的行号已经不是 4322 / 4324 / 4454。第二轮判决第 2 条引的 `.claude/hooks/lib_heavy_tests.py:3387` 同条件下没再核。

### 冲突并排（K4 里「句子比 `classify` 宽」那一处，整句抄）

    .claude/rules/implementation-workflow.md:49
      实验复跑入口 `scripts/experiment-apparatus/replay.sh`（不带参数、点名实验号、`--admission-only`；`--list-items` 与登记为 light 的单独 `--selftest` 不算）
    .claude/hooks/lib_heavy_tests.py:621
      不算的只有摘掉 --force 之后恰好一个 --list-items 的（逐行打可点名的实验号，
    .claude/hooks/lib_heavy_tests.py:623
      --list-items 后面还带别的词，都照重型算。

## 旁注（不在 K1 / K4 / K5 的问里，量到了，交主 agent 判去向）

1. **覆盖那一支对带 wgsl 单元的测试永远不「当前」**。`scripts/mutation/lib_mutation_units.py:532`「units = self.rust_file(reference.file).units」对记录里每个非数据文件的单元都按 Rust 文件去找；覆盖记录把闭包里不是 fn 的单元一起存（`scripts/mutation/crates-mutation-rows.py:3067`「for reference in executed + [member for member in closure.members if member.kind != "fn"]}」），wgsl 单元（`wgsl_compute`、`wgsl_fn` …）在 Rust 文件里找不回。`tools/k1_wgsl_probe.py` 对那 49 条取样覆盖记录数：

        name=k1_wgsl_probe coverage_records=49 with_wgsl_members=6 (executed_set_is_current 真 0) without_wgsl_members=43 (executed_set_is_current 真 43) missing_by_kind_in_wgsl_records={'wgsl_compute': 30, 'wgsl_const': 4476, 'wgsl_const_assert': 270, 'wgsl_fn': 3810, 'wgsl_struct': 390, 'wgsl_var': 1206}

   闭包带 wgsl 的 6 条一条都不「当前」，别的 43 条都「当前」。这些测试（GPU 的）的基线复用就只剩「闭包号相同」那一支（`scripts/mutation/crates-mutation-rows.py:1380`），「没执行到的函数变了、基线照复用」对它们不成立。偏保守（多重判基线），不是少作废。取样是 50 行里的 6 条，不是全表的比例；覆盖记录是我照 `judge_coverage` 的写法在合成 export 上造的，不是真插桩跑出来的。不属 K1，也和 lfu 无关（`current_fingerprints` 在 `lib_mutation_units.py` 里）。
2. **开工后 `scripts/mutation/broad-mutation.py` 被别的会话改了**（见 K5 末段）。K3（GPU 导入退码）不归我这条腿，没核它。
3. 本机此刻负载 100–285（别的会话的 e165 装置，每个 30 多个线程）：我所有计时只作参考；`crates-mutation-rows.py --selftest` 现文版 2 分钟内出结果，带弄坏开关那一次在 235 秒超时。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 文件级排除在 crates 变异判法上 | 一致（成立） | 三条路径：lfu 输出不进结局判法（路径 1 awk 无输出）；单改 lfu 一处、树不动，五种改动里没有「键不变而复用变」（路径 2 全表 5513 行 + 50 行取样；路径 3 的 `plan` 5413 行）；代价：E5 指纹写窄 + 同窗树改动 plan 沿用 3838 行、基线沿用 33 条，路径 2、3 共用同一组改动，这一格上不独立 |
| K1 翻面观测 | — | 单改 lfu 出现键不变而复用或结局变（0 次）；或把「同窗树改动不许漏」当排除的前提（E5 + t1 即观测） |
| K4 补丁 vs 判决第二、三节 | 一致（带两处出入） | 依赖版本、论证句、判据句、`:49` 入口都对得上；出入 1：`.claude/rules/verification.md:35` 判决没点名；出入 2：H3-c 走第三条路（删空测一半，效力核了） |
| K4 入口那一句 vs `classify` | 冲突一处 | 不带参数、点名实验号、`--admission-only`、`--list-items`（单独）、`--selftest`（单独）对得上；`--list-items` 后面带别的词仍重型，句子没写「单独」 |
| K4 复核不了 | 复核不了 | `heavy-test-guard.sh` 头、`lib_mutation_units.py` 的开工原样不在手；整份自证带弄坏开关 235 秒超时 |
| K5 465 / 961 / 496 | 一致 | 现跑核到 |
| K5 1442 / 801 / 159 | 一致 | 现跑核到 |
| K5 identical=6 | 一致 | 攻方模型在冻结副本上复跑 6 行 identical |
| K5 八行 identical | 冲突（复现不了） | 辩方模型读活仓，修补二批删了它查的句子，今天退 1 |
| K5 倒推 13 / 14 | 冲突（复现不了） | 今天 9 / 14，差的 4 份是修补二批动过的文件 |
| K5 cite-check 53 / 27 / 81 | 部分 | 53、55、28 核到；27 / 81 得 23 / 85（倒推树少两份） |
| K5 行号 | 一致（除 broad-mutation） | 510、1378、3375、99、7154 核到；`broad-mutation.py` 的行号在开工版本里核到，现文被改 |

## 要主 agent 跑的命令

整套复跑（约 35 分钟；只算 python，不起 cargo；主 agent 经 `run_in_background` 起，别的会话占满机器时更久）：

    SINGLEFS_RUN_SCOPE='种类=items; 跑什么=三方腿模型 rerun.sh（crates-mutation-rows.py 的 judging-units 与 plan、只读 python 探针）; 范围=整张变异表加取样; 为什么=判决复跑 K1/K4/K5' bash scripts/hosts/run-with-memory-cap.sh 16G bash <副本>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-gov-b17-r3-snapshot --work <新的草稿目录>

跑完看末尾八行 `name=rerun_compare … result=identical`；`out/` 里的 `evidence-*.txt` 与 `k4-patch-reverse.txt` 不进比对。机器上别的会话改过的文件会让对应的数变（先看 `out/judged-sha256-now.txt` 里 FAILED 的是哪几份）。

## 没做什么

1. 没验：lfu 自己的 `--selftest` 在 E5 下红不红（起它被 `heavy-test-guard.sh` 拒）。
2. 没验：整份 `crates-mutation-rows.py --selftest` 带 `GATE_MUTATION_BREAK=judging-files-included` 时那一格红（超时），只做了指纹层的等价核。
3. 没验：`heavy-test-guard.sh` 头、`lib_mutation_units.py` 这一批改动只有点名的那一处（开工快照没存原样）。
4. 没跑：K2、K3（不归这条腿）；cargo、重型测试、门禁全量一概没跑。
5. 只量了 E1–E5 五种 lfu 改法；攻方腿专找「改 lfu 而判定变键不变」的输入，我不替它找。
6. 开工后被别的会话改过的文件：`scripts/mutation/broad-mutation.py`（我的 `sha256sum -c` 前后各一次，前一次 OK、后一次 FAILED）。别的 11 份被判文件与 kb 在最后一次核时仍 OK。
7. 不判别的腿的格，不替主 agent 采纳或出判决。

## 清理

交回前删掉我自己建的仓副本与库拷贝：`run2`（204M，含 `w1` 各变体与 `base`、五份判定库、`plan` 输出）、`r2-tree`（2.1M）、`r2-rerun`（392K，第二轮两个模型目录的拷贝与它们的输出）、`smoke`（20K）；更早删过的 `w1`（约 95M）、`base`（约 74M）与几份 `lib-*` 判定库（没记大小）。草稿目录 `~/.local/share/singlefs/work/m3-close-gov-b17-r3-sonnet` 现在只剩 `out/` 里的文本（1.5M）与一份 sha256 清单。
