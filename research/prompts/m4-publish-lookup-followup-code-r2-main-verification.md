# m4-publish-lookup-followup-code-r2：里程碑四七·6 后续批，代码轮第二轮判决（2026-10-23）

<!-- doc-lint:not-numbers T1 T2 T3 T4 NA1 NA2 NA3 NA4 ALT4 ALT5 ALT12 T4b -->

被判的产品代码：`crates/singlefs-core/src/transaction.rs` 与 `crates/singlefs-core/build.rs`（第 11 批，`research/prompts/_m4-publish-lookup-followup-code-r2-diff.md` 的 `crates.patch`）。测试与测试基础设施 `crates/singlefs-unit-graph/src/build_features.rs`、`crates/singlefs-checker-tier/build.rs`、`crates/singlefs-harness/tests/publish_lookup_by_identity.rs` 与 `crates/mutations.tsv` 追加的 17 行、换掉的 3 行一并判 T2、T4。这是这一题的第二轮。

开工快照：`research/prompts/m4-publish-lookup-followup-code-r2-snapshot/`

## 一、核查

| 核了什么 | 怎么核 | 结果 |
|---|---|---|
| 两条腿的引文 | `python3 scripts/agents/cite-check.py research/prompts/m4-publish-lookup-followup-code-r2-sonnet-output.md research/prompts/m4-publish-lookup-followup-code-r2-opus-output.md --root ~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r2/repo --background research/prompts/_m4-publish-lookup-followup-code-r2-background.md` | 「✓ 核了 24 处引文、69 处裸行号，对上 24 处，没判 69 处（其中裸行号没有引文可比 63 处；逐处列在上面）」。判决要用的几行主 agent 在冻结副本里逐行读：`transaction.rs` 第 14342 行 `Some(_) if !rewritten.contains(&identity) => carried_unit(`，`rewritten` 是 `&resolved.rewritten_roles`，类型 `Vec<TransactionUnit>`（第 6729 行 `pub rewritten_roles: Vec<TransactionUnit>,`）；`recovery.rs` 第 3716–3722 行 `let inode_record_of = \|inode: u64\| {` 在全部 inode 叶容器的记录上 `.find(\|inode_record\| inode_record.inode == inode)`，第 3766 行在逐个「第一个文件之外的文件」的循环里调它；`crates/singlefs-core/build.rs` 第 14 行文档点名的单测 `the_core_build_script_lists_every_feature_of_its_manifest`，`grep -rn` 在冻结副本 `crates/` 下只命中这一行注释本身 |
| 开工快照 | `sha256sum -c research/prompts/m4-publish-lookup-followup-code-r2-snapshot/sha256sums.txt` | 主工作区 `crates/mutations.tsv` 对不上，其余 14 份 OK；冻结副本里 9 份 crates 与 kb 全部 OK。`crates/mutations.tsv` 的差是别的会话改的 6 行（`reversal/mutations-tsv-since-snapshot.diff`，12 行 `<`/`>`），以「七·6 后续批」起头的 17 行冻结副本与主工作区各 17 行。两条腿只引冻结副本 |
| 攻方的复跑数 | 模型目录拷到 `~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r2-main-check/model/`（9 个文件 sha256 与攻方报告开头那张表逐个相同），`bash scripts/hosts/run-with-memory-cap.sh 16G bash <那份>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r2/repo --work <草稿>/run --model <那份>`（头一趟本机根分区满、编不过，盘空出来后重跑） | 退 0。原样：`name=job job=base … test result: ok. 12 passed; 0 failed; 1 ignored`，`job=NA1`、`NA2`、`NA3`、`NA4` 四行同为 `rc=0 test result: ok. 12 passed; 0 failed; 1 ignored`；`name=none_site job=base None: LookupWork { index_builds: 5`、`name=none_site job=NA2 None: LookupWork { index_builds: 29`；`name=remount files=256 units=2354 mapped=2337 mount=ok times_entered=1 index_builds=1 index_lookups=2337`（基线）与 `… times_entered=2337 index_builds=2337 index_lookups=2337`（NA3）；`name=carried deadlist_pool_rewrite data_units=8 carried_accounting_nodes=0`、`data_units=32 … carried_accounting_nodes=0`；`name=probe` 五行 `first_file_contains_comparisons=2816`、`12544`、`37888`、`62464`、`165888`；`name=differential seeds=32 steps=48 lines=1600 before_vs_after=identical`；`name=t4 cell=rustflags_with_target library_rustc_cfg_lookup_work_counters=on build_script_rustc_cfg_lookup_work_counters=off build_script_output=[cargo::metadata=enabled_features=]` 与 `cell=rustflags_without_target … build_script_output=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS]`。与攻方 `out/rerun-expected.txt` 逐行相同，只差一格：第一个文件 1024 个单元、重写 32 个单元那一行攻方报 `211968`，复跑是 `62464`（= 1024 × 61，`rewritten_roles=61`，与其余四行的算术同形）；攻方报告自述第二台共用编译目录让一趟跑了旧变异体，判决用复跑的 `62464` |
| 正推腿的数 | 正推 T3.2 恢复那一处 `inode_record_find`（1024 个文件 525825 次）、T2.5 自造变异 ALT4、ALT5、ALT12、T4b 的读数没有单独复跑 | 恢复那一处主 agent 读码核过形状（逐个文件在全部记录上线性找）；ALT4/ALT5 与攻方 NA4 同类、两条腿各自独立量到；ALT12、T4b 只有正推一条腿量过，判决里标「正推量的，没复跑」 |

## 二、判定与去向

| 格 | 正推 | 攻方 | 判定 |
|---|---|---|---|
| T1 修补本身 | 成立（取最先一项；几支与改前同；256 种子 × 64 步 16896 行逐字节相同，塞故障后 379 行不同） | 没打中（128 段 × 48 步写流、镜像、槽串逐行相同，两种弄坏各 1568、728 行不同） | **成立**。两条腿各写各的差分，不共用代码与采样；主 agent 复跑攻方那一份 `before_vs_after=identical`。崩溃状态没有逐个枚举（两条腿都是从写流逐条相同推的）；`units` 里重复角色那一支两条腿的随机历史都没走到，只靠读码（稳定排序取最先一项） |
| T2 计数与用例的判法 | 部分成立 | 打中四处（NA1–NA4） | **打中，坐实**：① `index_builds == times_entered` 分不开「每项进一次、每项建一次」：恢复那一侧每把 key 调一次（NA3）`times_entered=2337 index_builds=2337`，12 条用例全绿；恢复那一侧没有一条用例判它（复跑）。② 建在作用域外的重建记进 `None`，没有用例判 `None` 的建索引次数：NA2 把 `None` 的建索引次数从 5 抬到 29，12 条全绿（复跑）。③ 按树分开的记账树那一处在每个取样里照抄 0 个节点，只让记账树退回线性不红（NA1，复跑 `carried_accounting_nodes=0`）；今天两块盘上记账树装得进一片叶、这条变异等价是推的。④ 计数只数经索引的动作，索引照用、旁边再加一遍线性找不红（NA4 复跑；正推 ALT4、ALT5 同类）。⑤ 「一把有位置的 key 都没有就不建」没有用例钉（正推 ALT12，正推量的，没复跑）。⑥ 20 行变异里三对改后的源逐字节相同（第 12 行同第 7 行、第 13 行同第 9 行、第 14 行同第 8 行，正推量的）：17 种不同的改法。20 行各自红在名字说的那一处（正推逐行列的表）；整行换掉的 C491、C394 两行改前改后红在同一条断言（正推在改前副本上补跑了），实现员报告里没对照的那一处补上了 |
| T3 没看过的同路径查找与副作用 | 部分成立（第 11 批没有可量的新代价；还剩两处平方） | 打中一处 | **打中，坐实**：① `transaction.rs` 第 14342 行每次照抄第一个文件的发布，在 `Vec` 上为第一个文件的每个数据单元线性找一遍，比较次数 = 第一个文件的数据单元数 × 这次重写的角色数（复跑：64 × 44 = 2816、256 × 49 = 12544、1024 × 37 = 37888、1024 × 61 = 62464、1024 × 162 = 165888）；第一个文件占池的大头又整份重写一个大文件时两样都到池大小的量级。② `recovery.rs` 第 3766 行恢复时逐个文件在全部 inode 记录上线性找，文件数的平方（正推量的、没复跑：1024 个文件 525825 次；主 agent 读码核过形状）。两处都不在七·6 点名的位置里、第 11 批没碰。第 11 批自己的副作用：恢复那一侧每次重挂建 1 份索引（复跑基线行），小发布与不开特性时的 `Drop` 两条腿都没量出可见代价（正推量的挂钟只量了一次，攻方没量），判不打中 |
| T4 构建脚本与判法单元键 | 部分成立 | 打中一种编法 | **打中，坐实**：① 同时带 `--target` 与 `RUSTFLAGS='--cfg feature="lookup-work-counters"'`、不开特性时，库按开着编、`singlefs-core` 构建脚本交出空的 `enabled_features`（复跑两行）；仓里的脚本今天这样编没有，推的，没全仓搜。② `crates/singlefs-core/build.rs` 第 14 行文档点名的单测不存在（主 agent grep 核过）；核心层清单加一个特性而表没加时，`cargo build` 与 checker 档构建都不报，只有跑 harness 档 `publish_lookup_by_identity` 那一个目标才红（两条腿读码一致）。③ 把核心层构建脚本里那一格 cfg 写死成假（开着也报没开），登记的用例与变异都不红（正推 T4b 量的，没复跑）：harness 档用例经 `#[path]` 把构建脚本编进测试包，测试包里这个 cfg 恒为假，只钉得住「没开别报开着」那一半。checker 档三项与键里 cfg 原子判成立（两条腿读码一致） |

## 三、交给下一批的实现员（按文件聚簇）

落在 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs`、`crates/mutations.tsv`、`crates/singlefs-core/build.rs`（文档那一句）与 `crates/singlefs-unit-graph/src/build_features.rs`（要补单测就在这里）；T3 两处与 T4 ① 要不要改产品代码见第五节用户定。

1. **T2 ①**：恢复那一侧加一条重挂用例，判 (进 1、建 1、查 = 有位置的进映射角色数)，NA3 那条变异进 `crates/mutations.tsv`、点名这条用例红。用 `assert_one_index_built_per_entry` 的各处逐处核旁边有没有一条判查次数的断言，没有的补上。
2. **T2 ②**：发布路径各用例另判 `None` 的建索引次数在两档单元数下不涨；NA2 进变异表、点名的用例红。
3. **T2 ③**：加一个让记账树照抄到不止一个节点的取样（多盘或别的造法，实现员读码定），按树分开那条用例在三棵树上都判整数遍；NA1 进变异表、点名的用例红。造不出的停下交主 agent，写明为什么。
4. **T2 ④**：计数看不见的「索引照用、旁边再加一遍线性」：给五处（映射位置、要写的、点名的、留给快照的两处、记录点名的数据角色）各加一条按比较次数或按规模判的用例，或者写明为什么不判、登记成已知边界；NA4、ALT4、ALT5 三条按实现员的取舍进变异表或进登记的边界说明。
5. **T2 ⑤**：「一把有位置的 key 都没有就不建」那一支钉一条用例，ALT12 进变异表。
6. **T2 ⑥**：第 12、13、14 行与第 7、9、8 行改后源逐字节相同：删掉重复的三行，或改成真不同的写法；改了的变异行证红。
7. **T4 ②③**：`crates/singlefs-core/build.rs` 第 14 行文档改成指真正判它的地方，或在 `crates/singlefs-unit-graph/src/build_features.rs` 补上它点名的那条单测（读核心层清单 `[features]` 与表逐个比）；「开着要报开着」那一半补一条会红的检查（真开特性编构建脚本，读它交出的元数据），T4b 进变异表、点名的检查红。

改法都是零轮形态（两条腿各自的改法只在它们自己的模型上推过）；这一批改完照验证段合入、轻型三条，再开这一题的第三轮（最终轮）。

## 四、复跑与核不动的

- 正推腿的读数（恢复那一处 525825 次、ALT12、T4b、20 行逐行红在哪、三对重复）没有单独复跑，见第一节；判决里用到的地方逐处标了「正推量的」。
- 攻方第一个文件 1024 × 32 那一格的 211968 不用，用复跑的 62464。
- 崩溃状态没有逐个枚举；恢复代码只在干净与不干净两种重挂上比过，没在中途崩溃的状态上比过（攻方报告自述）。
- 挂钟没有一个数进判决：两台满载，两条腿量的挂钟都只量了一次。

## 五、交用户

| 岔路 | 事实 | 候选 |
|---|---|---|
| T3 的两处按项线性找怎么办 | `transaction.rs` 第 14342 行比较次数 = 第一个文件数据单元数 × 重写的角色数（主 agent 复跑：1024 × 162 = 165888）；`recovery.rs` 第 3766 行恢复时文件数的平方（正推量的、没复跑：1024 个文件 525825 次）。两处都不在七·6 点名的位置里，第 11 批没碰 | 并进下一批 / 只并发布那一处 / 记进里程碑四优化项不改。**用户 2026-10-23 定：并进下一批**（弹窗选「并进下一批 (Recommended)」） |
| T4 ① `--target` 加 `RUSTFLAGS` 带特性 cfg 时键与真编不符怎么办 | 复跑：`cell=rustflags_with_target library_rustc_cfg_lookup_work_counters=on … build_script_output=[cargo::metadata=enabled_features=]`；仓里脚本今天这样编没有（推的，没全仓搜） | 并进下一批（构建脚本另比 `CARGO_ENCODED_RUSTFLAGS` 里的 `--cfg feature=`，对不上就停）/ 登记成已知边界不改。**用户 2026-10-23 定：并进下一批**（弹窗选「并进下一批 (Recommended)」） |

## 回看决策

不涉及决策：这一轮判的是七·6 后续批第 11 批的查法、计数与测试判别力和 `singlefs-core` 构建脚本交出的元数据，没有改动或确立 `.claude/kb/decisions/` 下任何一条的说法；压着的条款是 `.claude/kb/milestone/04-fourth-txn.md` 七·6 那一行与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。构建脚本那一格压着 D13（验证路线） 已定项 17 的键组成，这一轮只报出两种编法下键与真编的不符，不改它的说法。
