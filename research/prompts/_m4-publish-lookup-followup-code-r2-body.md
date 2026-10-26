# m4-publish-lookup-followup-code-r2：里程碑四七·6 后续批，代码轮第二轮正文（2026-10-23）

<!-- doc-lint:not-numbers T1 T2 T3 T4 MA1 MA2 MA3 MA4 MA5 -->

## 一、题面

第一轮判决 `research/prompts/m4-publish-lookup-followup-code-r1-main-verification.md`「三、交给下一批的实现员（按文件聚簇）」第 1–4 条交给第 11 批实现员，补丁已合进主工作区。这一轮攻第 11 批的合入，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮的分工：第一轮的修补本身、产生结论的方法、从未被看过的地方。第一轮攻过的角度（七处改法取的是不是同一项、8 处单次调用的阶数、MA1–MA5 那几种每项重建、外带 `CARGO_FEATURE_*` 变量）不重复，除非第 11 批改动了它们的前提。

改动全文：`research/prompts/_m4-publish-lookup-followup-code-r2-diff.md`（第 11 批的 `crates.patch`、`mutations-append.tsv`、`mutations-replacements.tsv` 原样）。实现员报告：`~/.local/share/singlefs/work/m4-publish-lookup-followup-2-writer/patch/report.md`（sha256 前缀 `9aaa93ef`）。

## 二、实现今天的样子（主 agent 的观测，2026-10-23）

- 冻结副本 `~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r2/repo/`（此刻主工作区，含第 11 批与别的会话这之后合入的改动；去掉了私有配置 `multi-host.env`、`.env`、`.git` 与编译目录）。被判文件与引到的 kb 的 sha256 在 `research/prompts/m4-publish-lookup-followup-code-r2-snapshot/sha256sums.txt`。
- 产品代码改动（第 11 批，`git apply --numstat` 加 / 减行数）：`crates/singlefs-core/src/transaction.rs`（183 / 40）与 `crates/singlefs-core/build.rs`（30 / 9）。测试与测试基础设施侧：`crates/singlefs-unit-graph/src/build_features.rs`（95 / 26）、`crates/singlefs-checker-tier/build.rs`（16 / 2）、`crates/singlefs-harness/tests/publish_lookup_by_identity.rs`（519 / 30）、`crates/mutations.tsv`（追加 17 行、整行换 3 行）。
- 冻结副本里被改到的位置（`transaction.rs` 的行号）：`lookup_work` 模块 `LookupSite` 第 3286 行起、`SortedIdentityIndex` 第 3503 行、`FirstEntryOfEachIdentity` 第 3590 行；`take_the_mapping_positions_over_the_location_hints` 第 3730 行；`placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots` 第 4401 行；新函数 `roles_other_than_the_ones_kept_for_the_snapshots` 第 4580 行；`mapped_roles_released_by_this_publish` 第 4917 行；`deadlist_changes_of_this_publish` 第 6219 行；`roles_named_by_each_record_of_the_publish` 第 11814 行与新函数 `roles_other_than_the_data_roles_named_before_the_last_record` 第 11857 行；`build_extent_tree` 第 12065 行；`publish_admitted` 第 13161 行；新函数 `lookup_site_of_the_carried_nodes_of` 第 15006 行；`build_multi_level_tree` 第 15060 行。`crates/singlefs-core/Cargo.toml` 第 10 行 `links = "singlefs-core"`、第 22 行 `lookup-work-counters = []`。
- 第 11 批的要点（实现员报告「改法」那张表）：第 1 条 ① 映射位置换进 `units` 改成先问完位置、再建一份按角色的 `SortedIdentityIndex`（同一角色取最先那一项）逐项改槽；② `publish_admitted` 装完 `units` 建一份 `FirstEntryOfEachIdentity`，`written_units`、`named_unit_of` 都查它。第 2 条三处 `.contains` 改成每遍建一次按角色的索引查。第 3 条：计数加 `times_entered`，测试判「建索引次数恰好等于走了几遍」就是 `index_builds == times_entered`；删快照进那一处挪到循环外；按 inode 号找文件的读者不再自己进一处、调用方各进各的；`build_extent_tree` 开头进 `CarriedExtentTreeNodes`；`build_multi_level_tree` 按树进三处（换掉 `CarriedNodesOfTheMultiLevelTrees`）；整理发批用例加第三批（3 项、没有留存项）。第 4 条：`crates/singlefs-core/build.rs` 交出的特性改成只看 `cfg!(feature = …)`、不读环境；`build_features::features_of_the_crate_being_built` 改收这张表、与清单 `[features]` 对不上交新错误 `FeatureTableDisagreesWithTheManifest`；checker 档构建脚本照同一取法交本包三项。
- 合入后验证（量的）：第 11 批 2026-10-22 打进主工作区。全包合入后验证在含第 11 批的主工作区上绿（`~/.local/share/singlefs/work/merge-and-verify/2026-10-22-78544`：「TEST_TARGETS_PARALLEL dispatched=299 collected=299 failed=0 jobs=31 test_threads=1 doc=yes」，`publish_lookup_by_identity`「test result: ok. 12 passed; 0 failed; 1 ignored」）。变异轻型（`~/.local/share/singlefs/work/m4-followup-2-mutation-local/mutation-quick-4.log` 第 323 行「计数：抓到 268 条、没红 1 条、无效 1 条、内存撞顶 0 条、超时 0 条、被总上限挤掉 0 条、排不上没跑 0 条、scope 起不来没跑 0 条、锚点失效 1 行（共 273 条）」）里第 11 批的 20 行全抓到，没红、无效与锚点失效那几行都不在第 11 批。形式化轻型「✓ checker-tier-formal-verification.sh：3 格里 3 格绿（verus-toolchain-pinned verus-proof pinned-red-examples-table），0 格本次未跑（无）」。真 cargo 核第 4 条（`~/.local/share/singlefs/work/multi-host-run/t4-links-metadata-b11-2026-10-22/t4-result.txt`）：`name=t4 cell=environment build_script_ran=yes rustc_cfg_lookup_work_counters=off build_script_output=[cargo::metadata=enabled_features=]`、`cell=config_env` 同、`name=t4 cell=feature build_script_ran=yes rustc_cfg_lookup_work_counters=on build_script_output=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS]`、`name=t4 cell=stale build_script_ran=no rustc_cfg_lookup_work_counters=not-recompiled build_script_output=[cargo::metadata=enabled_features=]`、`name=t4 cell=checker_tier_keys_compare groups_with_a_different_line=0 of 66`。崩溃轻型（`~/.local/share/singlefs/work/m4-b11-postmerge/crash-quick-2.log`）「CRASH_POINT_REPLAY_TOTAL tier=quick dispatched=39 collected=39 green=36 red=2 not_run=1 …」，两条红主 agent 查过都不在第 11 批（一条是别的会话改池级 checker 新根段判法带出的 GPU 对拍用例，一条是跑的过程中源码被别的会话换掉、导入那一趟找不到录下的流），不当前提。
- 实现员自报没做的（报出来不算打中，放进格里攻后果）：「外带变量、不开特性」实现员没用真 cargo 编过，主 agent 合入后用攻方第一轮的脚本补跑了（上一条）；cargo 文档原文没读到，依据是 `librocksdb-sys` 构建脚本的同一取法与本机构建脚本输出；整行替换的「释放之前读盘核校验和只走这次重写的角色」那一行替换之后红在产品里那条对账断言，改前红在哪条没对照；harness 慢档增长用例没跑。
- 压着的条款：`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·6 那一行，与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　修补本身：新索引与改前的线性找取不取同一项、写不写同一份字节
核：`SortedIdentityIndex`、`FirstEntryOfEachIdentity` 在同一角色有几项时取哪一项，与改前的 `iter().find` 是不是同一项；`take_the_mapping_positions_over_the_location_hints` 改成「先问完位置、有位置的才建索引」之后，位置为空、部分有位置、`mapped_units` 里有 `units` 里找不到的角色这几支，行为与改前是不是一样（含 panic 文案与条件）；`publish_admitted` 里索引建好之后到最后一次查之间 `units` 会不会被改、索引会不会过期；三处 `.contains` 换成索引之后，清单里有重复角色、清单为空时取舍是不是一样；同一串合法操作写出的盘镜像改前改后是不是逐字节相同。

### T2　产生结论的方法：计数与用例判的是不是那件事
核：`times_entered` 与 `index_builds == times_entered` 这条判法，会不会让「每遍建一份」和「每项建一份但每项都进一次」分不开；挪到循环外、调用方各进各的之后，有没有一处查找落在任何一个 `LookupSite` 的作用域之外（记进 `None`），或者两个作用域套着、一次查记进两处；按树分开的三处在只照抄一棵树、三棵都照抄、一棵都不照抄时，整数遍判法会不会空真；两档（8 与 32 单元）与三批（2、8、3 项）取样分不分得出每遍重建、每项重建、退回线性三类；20 行变异各自红的理由是不是它名字说的那件事，换一种同样坏的写法还红不红。

### T3　从未被看过的地方：第 11 批之外、同一条路径上的查找与第 11 批改动的副作用
核：`publish_admitted`、整理发批、恢复、释放这几条路径上，第 11 批与七·6 前几批都没碰过的按项线性找、`.contains`、`.position` 还有没有（随池大小或随一次写的单元数平方涨的）；新建的索引在恢复路径（`recovery.rs` 调 `take_the_mapping_positions_over_the_location_hints`）上的内存与次数；`FirstEntryOfEachIdentity` 建索引排序本身的代价在小发布（1–2 个单元）上是不是比改前线性找更慢到可量；`lookup-work-counters` 特性关着时 `InsideALookupSite` 也实现 `Drop` 之后，产品构建里有没有多出的分支或代价。

### T4　构建脚本与判法单元键这一链
核：`crates/singlefs-core/build.rs` 只看 `cfg!(feature = …)` 之后，cargo 在 harness 档 dev 依赖打开 `lookup-work-counters` 而普通依赖没开时，两份构建脚本输出各报的是什么、checker 档读到的是哪一份；`FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG` 这张表手写，清单 `[features]` 加一个特性而表没加时谁会红（`FeatureTableDisagreesWithTheManifest` 在哪一步报、会不会只在某一条编法上才报）；`cell=stale` 那一行（构建脚本不重跑、元数据留着空）是不是正确行为；checker 档构建脚本交本包三项（`default`、`gpu`、`verdict-store`）与判法单元键里 cfg 原子的真值是不是同一份。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T4 | 核「代码做的是不是第一轮判决第三节 1–4 条要的那件事」：逐格对补丁原文与冻结副本的行，每条结论写文件:行号；T1 逐处列出新索引的建处、查处与改前改后各取哪一项；T2 逐条列 20 行变异改的是哪一行、点名的用例红在哪条断言、为什么那条断言判的是这一处 |
| 云端攻方（Opus） | T1–T4 | 在冻结副本的拷贝上造反例：一串合法操作让新索引取到不同的项、或改前改后写出不同的盘镜像、或改后 panic 而改前不 panic；T2 造一个改坏查法（每项重建、退回线性、建在作用域外、只照抄一棵树）而 20 行与五条快档用例都不红的变异；T3 找一处第 11 批没碰、在同一条路径上按项调的线性查找，量它随表长怎么涨；T4 造一种编法让判法单元键与真编的特性不符 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷过去的源文件先 `find crates -type f -exec touch {} +` 再编。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，同一时刻只起一条 cargo；用自己的编译目录。本机资源紧张就尝试其他可用的机器（`scripts/hosts/multi-host-run.sh --peer`）。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
