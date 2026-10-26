# m4-publish-lookup-followup-code-r3：里程碑四七·6 后续批，代码轮第三轮正文（2026-10-23）

<!-- doc-lint:not-numbers T1 T2 T3 T4 NA1 NA2 NA3 NA4 ALT4 ALT5 ALT12 T4a T4b -->

## 一、题面

第二轮判决 `research/prompts/m4-publish-lookup-followup-code-r2-main-verification.md`「三、交给下一批的实现员（按文件聚簇）」第 1–7 条与「五、交用户」两行（用户 2026-10-23 定「并进下一批」）交给第 17 批实现员，补丁已合进主工作区。这一轮攻第 17 批的合入，是这一题的第三轮，也是最终轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第三轮的分工：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核。前两轮攻过的角度不重复：七处改法取的是不是同一项、8 处单次调用的阶数、MA1–MA5、外带 `CARGO_FEATURE_*` 变量、`index_builds == times_entered` 的判法、`None` 作用域、按树分开、第 14342 行与第 3766 行两处线性找、`--target` 加 `RUSTFLAGS`。只有第 17 批改动了它们的前提时才回头攻。这一轮里新冒出来的形态不再为它开一轮，判决里记进交用户表，标「零轮」。

改动全文：`research/prompts/_m4-publish-lookup-followup-code-r3-diff.md`，是第 17 批的 `crates.patch`、`mutations-append.tsv` 与 `mutations-delete.txt` 原样。实现员报告：`~/.local/share/singlefs/work/m4-publish-lookup-followup-3-writer/patch/result.md`（sha256 前缀 `458d089a`）。

## 二、实现今天的样子（主 agent 的观测，2026-10-23）

- 冻结副本 `~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r3/repo/`：此刻的主工作区，含第 17 批与别的会话在这之后合入的改动；去掉了私有配置 `multi-host.env`、`.env`、`.git` 与编译目录。被判文件与引到的 kb 的 sha256 在 `research/prompts/m4-publish-lookup-followup-code-r3-snapshot/sha256sums.txt`。
- 产品代码改动（第 17 批，`git apply --numstat` 加 / 减行数）：`crates/singlefs-core/src/transaction.rs`（49 / 9）、`crates/singlefs-core/src/recovery.rs`（17 / 5）、`crates/singlefs-core/build.rs`（88 / 3）。测试与测试基础设施侧：`crates/singlefs-harness/tests/publish_lookup_by_identity.rs`（433 / 10）、`crates/singlefs-unit-graph/src/build_features.rs`（41 / 0）；`crates/mutations.tsv` 追加 9 行，删 3 行。
- 冻结副本里被改到的位置：
  - `transaction.rs`：`lookup_work` 模块的常数 `ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT` 在第 3287 行；新 `LookupSite` `RewrittenRolesAmongTheDataUnitsOfTheCarriedFirstFile` 在第 3332 行，`InodeRecordsOfTheFilesJudgedByTheRecovery` 在第 3336 行；`publish_admitted` 从第 13213 行起，照抄第一个文件的那一段是第 14397–14425 行（`is_each_data_unit_of_the_carried_first_file_rewritten`）。
  - `recovery.rs`：`data_units_judged_against_the_inode_records` 在第 3779 行。
  - `crates/singlefs-core/build.rs`：`features_opened_by_the_rustflags_but_off_in_this_build_script` 在第 54 行，`main` 在第 97 行。
  - `crates/singlefs-unit-graph/src/build_features.rs`：第 258–268 行用 `#[path]` 把核心层构建脚本编进单测；单测 `the_core_build_script_lists_every_feature_of_its_manifest` 在第 728 行。
  - `publish_lookup_by_identity.rs`：文件头第 31 行起是「已知边界」一段；`assert_the_index_builds_outside_the_named_sites_stay_bounded` 在第 1659 行（常数 `INDEX_BUILDS_OUTSIDE_THE_NAMED_SITES_PER_ASSEMBLY_AT_MOST = 6` 在第 1655 行）；新用例分别在第 2492 行（照抄第一个文件两档）、第 2589 行（重挂 4 与 16 个文件）、第 2660 行（没有带位置的 key）、第 2684 行（构建脚本真开着时报开着）、第 2698 行（`RUSTFLAGS` 判法）。
  - `crates/mutations.tsv`：第 17 批的 9 行在第 5311–5319 行，以「七·6 后续批第二轮」起头。
- 第 17 批的要点，取自实现员报告「改法与新 `LookupSite`」那张表：
  - 第五节 T3 ①：照抄第一个文件时，先给 `rewritten` 建一份 `FirstEntryOfEachIdentity`，逐个数据单元查完收成 `Vec<bool>`，再在装 `units` 的循环里按下标取。
  - 第五节 T3 ②：恢复时按叶容器、记录的次序把记录收成 `Vec<&InodeRecord>`，建一份按 inode 号的 `FirstEntryOfEachIdentity`（同号取最先那条），`inode_record_of` 改查它。
  - 第五节 T4 ①：构建脚本按 0x1f 切开 `CARGO_ENCODED_RUSTFLAGS`，认 `--cfg` 后面一项与 `--cfg=` 那一项里的 `feature="…"`；与自己的 `cfg!(feature = …)` 对不上就 `assert!` 停。
  - 第 7 条：构建脚本另经 `cargo::rustc-env` 把同一个值编进核心层，测试拿它判「cargo 开着时报开着」。
  - 第 3 条：用只供测试的 `CodeTwoTreeNodeCapacities::CappedForTests` 把记账树压小，让三棵树各照抄不止一个节点。
  - 第 2 条：常数 6 是量出来的（5 或 6），不是推出来的。
  - 第 4 条：NA4、ALT4、ALT5 登记成文件头「已知边界」，不进变异表。
  - 第 6 条：删掉三行改后源逐字节相同的变异。
- 合入后验证（量的）：
  - 合入后验证 `~/.local/share/singlefs/work/merge-and-verify/2026-10-23-3331479`：测试红 2 条，是别的会话系统配置读者新判法带出的（`core_review_tree_table_duplicates_and_slot_one_search.rs:958`、`checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs:444`），那个会话已认下；clippy 红在同一批的 E185 装置，主 agent 已改。
  - 变异轻型 `~/.local/share/singlefs/work/m4-b17-postmerge/mutation-quick-2.log` 第 1233 行：「计数：抓到 314 条、没红 0 条、无效 0 条、内存撞顶 0 条、超时 0 条、被总上限挤掉 0 条、排不上没跑 0 条、scope 起不来没跑 0 条、锚点失效 0 行（共 314 条）」，第 17 批的 9 行都点名在内。
  - 形式化轻型 `m4-b17-postmerge/formal-quick.log`：「✓ checker-tier-formal-verification.sh：3 格里 3 格绿（verus-toolchain-pinned verus-proof pinned-red-examples-table），0 格本次未跑（无）」。
  - 真 cargo 核 T4 ①（主 agent 在主工作区开一次性编译目录跑）：带 `--target x86_64-unknown-linux-gnu` 与 `RUSTFLAGS='--cfg feature="lookup-work-counters"'` 时退 101，`crates/singlefs-core/build.rs:110` 停；不带 `--target` 时退 0，构建脚本输出 `cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS`。
  - 两档等价池 `images_sha256` 改前改后逐字相同，见实现员报告「两档等价池」。
  - 崩溃轻型：〔起跑前主 agent 填〕
- 实现员自报没做的（报出来不算打中，放进格里攻后果）：
  - 7 行变异没在副本上证红，靠变异轻型判了。
  - 整工作区 clippy、build 与别的 harness 档测试二进制没在副本上跑，归合入后验证。
  - 没有逐字节比重挂前后重建出来的版本，「恢复结果一字节不变」靠按 inode 号的索引取最先那一条与 `find` 是同一条。
  - harness 慢档增长用例没跑。
  - `None` 的上限 6 是量的常数。
- 压着的条款：`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·6 那一行；`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　「已定」项撑不撑得住：前两轮判成立的，到了第 17 批还成立吗
第二轮判「成立」的有：T1 新索引与改前线性找取同一项、写出的盘镜像逐字节相同；T4 checker 档三项与判法单元键里的 cfg 原子相同。核这两条在第 17 批两处新索引上还成不成立：
- `FirstEntryOfEachIdentity` 建在 `rewritten` 上，`rewritten` 里有重复角色、为空时怎么取；
- `Vec<bool>` 的下标与装 `units` 的循环是不是同一个次序；
- 恢复时按 inode 号的索引取到的记录与改前 `find` 取到的是不是同一条，含同一个 inode 号落在两个叶容器里的情形；
- 恢复出来的版本、挂载结果与改前是不是逐字节相同（实现员没比）；
- 构建脚本新加的 `cargo::rustc-env` 与 `assert!`，有没有让核心层的判法单元键、checker 档构建脚本读到的元数据，或任何一种仓里在用的编法的结果变了。

### T2　未定项清单的完整性：登记的边界与没判的那几样是不是全了
核：
- 文件头「已知边界」一段写的「计数看不见」那一类，是不是只有点名的那几处；有没有别的 `LookupSite` 也照样看不见、没登记；
- `INDEX_BUILDS_OUTSIDE_THE_NAMED_SITES_PER_ASSEMBLY_AT_MOST = 6` 这个量出来的常数，判不判得出它要防的那件事（`None` 随单元数涨）；在别的规模、别的发布形态下它是不是会被正常改动碰红，或者坏写法在它之下也不红；
- `publish_admitted`、恢复、整理、释放几条路径上，按项线性找、`.contains`、`.position` 还有没有剩下的；判决与实现员都没列到的，逐处给位置与随规模怎么涨；
- `CappedForTests` 只供测试的分支，产品编法里进不进得去；
- 前两轮判决「四、复跑与核不动的」两节列的缺口（崩溃状态没逐个枚举、恢复只在干净与不干净两种重挂上比过），第 17 批之后哪几样还开着，有没有新的。

### T3　报出的数字能不能核
在冻结副本的拷贝上照原样重算实现员报告与第二轮判决里引的数，逐个写对上没有：
- 实现员报告「新计数的读数」三段原样摘的 `carried_first_file`、`remount`、`deadlist_pool` 各行；
- `None` 建索引 5 与 6；
- 两档等价池的两个 `images_sha256`；
- 证红表里两行的 left / right；
- 第 17 批 9 行变异各自红在名字说的那一处，换一种同样坏的写法还红不红；
- 第二轮判决里只有正推腿量过、主 agent 没复跑的几个数：恢复那一处 525825 次、ALT12、T4b。

### T4　构建脚本这一链的新代码
核：
- `features_opened_by_the_rustflags_but_off_in_this_build_script` 的切法与认法：0x1f 分隔、`--cfg` 与 `--cfg=` 两种写法、引号与空白、`feature` 以外的键、表里没有的名字；
- 有没有一种合法编法被它误停（例如别的包的特性名与表里重名、`RUSTFLAGS` 带别的 `--cfg`）；
- 有没有一种不符的编法它放过；
- `CARGO_ENCODED_RUSTFLAGS` 没设、为空、不是 UTF-8 时的行为；
- `cargo::rustc-env` 编进去的值在构建脚本不重跑时会不会是旧的。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | T1–T4 | 复核第二轮判决：替被判「打中」的一方辩护。逐条判 NA1–NA4、T3 两处、T4 ①②③ 在第 17 批之后是真修好了，还是只让点名的用例与变异红了；第二轮判「成立」的 T1 与 T4 checker 档三项，到了第 17 批还站不站得住；判决交下去的 7 条里，有没有哪一条的改法同样打中了别的写法（判决够不够得着、是不是同样打中所有替代方案）。每条结论写冻结副本里的文件:行号 |
| 云端攻方（Opus） | T1–T4 | 在冻结副本的拷贝上造反例：<br>T1 造一串合法操作，让第 17 批两处新索引取到与改前不同的项，或让恢复出来的版本与改前不同；<br>T2 找一处没登记、计数看不见的坏写法，或一处还剩的按项线性找，量它随规模怎么涨；<br>T3 照原样重算报出的数，逐个写对没对上；<br>T4 造一种合法编法被构建脚本误停，或一种不符的编法被放过 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷过去的源文件先 `find crates -type f -exec touch {} +` 再编。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，同一时刻只起一条 cargo，用自己的编译目录，跑完删掉。第二台这段时间在跑别的实验的计时，不许用：不起 `scripts/hosts/multi-host-run.sh --peer`，也不往第二台拷东西；本机排队就等。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
