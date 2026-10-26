# m4-publish-lookup-followup-code-r1：里程碑四七·6 后续批，代码轮第一轮判决（2026-10-22）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

被判的产品代码：`crates/singlefs-core/src/transaction.rs`（七·6 后续批，`research/prompts/_m4-publish-lookup-followup-code-r1-diff.md` 第一个 diff 围栏）、`crates/singlefs-core/Cargo.toml` 与 `crates/singlefs-core/build.rs`（判法单元键那一批在 `crates/singlefs-core/` 下的两份）。测试代码 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs` 与 `crates/mutations.tsv` 追加的 19 行一并判 T3。这是另立一题（七·6 第三轮判决「五、交用户」第二行用户定）的第一轮。

开工快照：`research/prompts/m4-publish-lookup-followup-code-r1-snapshot/`

## 一、核查

| 核了什么 | 怎么核 | 结果 |
|---|---|---|
| 两条腿的引文 | `python3 scripts/agents/cite-check.py research/prompts/m4-publish-lookup-followup-code-r1-sonnet-output.md research/prompts/m4-publish-lookup-followup-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r1/repo --background research/prompts/_m4-publish-lookup-followup-code-r1-background.md` | 「✓ 核了 82 处引文、49 处裸行号，对上 82 处，没判 49 处（其中裸行号没有引文可比 49 处；逐处列在上面）」。判决要用的几行主 agent 在冻结副本里逐行读：`transaction.rs` 第 3762 行 `for (identity, key) in &version.mapped_units {`、第 3767 行 `.find(\|unit\| unit.identity == *identity)`（在 `take_the_mapping_positions_over_the_location_hints` 里，整理发批经第 13143 行、恢复经 `recovery.rs` 第 4147 行调它）；第 14576、14583 行 `publish_admitted` 里 `written_units` 与 `named_unit_of` 两处 `units.iter().find(…)`；第 4338、4820 行 `.filter(\|identity\| !kept_for_the_snapshots.contains(identity))`；第 11713 行 `.filter(\|identity\| !data_roles_named_before_the_last_record.contains(identity))`；第 6209 行 `.get_or_insert_with(\|\| previous_version.mapping_entries_kept_for_the_snapshots_by_key())`；第 11975 行 `let _inside_the_carried_nodes = lookup_work::entering(LookupSite::CarriedExtentTreeNodes);`；`mount.rs` 第 5529 行 `.map(\|role\| (*role, output.unit(*role).slot))`；`crates/singlefs-core/build.rs` 第 9–18 行按 `CARGO_FEATURE_` 前缀收进程环境、只 `rerun-if-changed=build.rs` |
| 开工快照 | `sha256sum -c research/prompts/m4-publish-lookup-followup-code-r1-snapshot/sha256sums.txt` | 主工作区 `transaction.rs`、`crates/mutations.tsv` 对不上，其余 12 份 OK；冻结副本里 14 份全部 OK。倒推：`transaction.rs` 被预开发会话打了标签 0 补丁（`research/prompts/m4-publish-lookup-followup-code-r1-snapshot/reversal/tag-zero-crates.patch`，sha256 `20511eb6…`），它打之前备的那一份 sha256 `3b81509b74d72a6f…` 与快照相同、与冻结副本逐字相同（`reversal/transaction-before-tag-zero.sha256`）；`crates/mutations.tsv` 是别的会话改与追加的行（`reversal/mutations-tsv-since-snapshot.diff`），以「七·6」起头的 41 行一字没变。两条腿只引冻结副本，腿引的行都不受这两处影响 |
| 攻方的复跑数 | 模型目录拷到 `~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r1-main-check/model/`（12 个文件与攻方报告开头列的 sha256 逐个相同），`bash scripts/hosts/run-with-memory-cap.sh 16G bash <那份>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r1/repo --work <草稿>/run --model <那份>` | 退 0。原样：`name=mutant mutant=baseline build_rc=0 rc=0 test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out failed=[]`，MA1–MA5 五行同为 `rc=0 test result: ok. 10 passed`；`name=slow_mutant mutant=MA5 build_rc=0 rc=101 test result: FAILED. 0 passed; 1 failed`；`name=slow_mutant mutant=MA1+MA2+MA3+MA4 build_rc=0 rc=0 test result: ok. 1 passed`；`name=differential seeds=512 steps=64 lines=33792 before_vs_after=identical`；`name=probe files=256 units=2354 mapped=2337 op=relocation_batch_of_8_from_the_last_file mapping_positions_find=16364 …`；`name=probe files=256 units=2354 mapped=2337 op=remount mapping_positions_find=2741311 …`；`name=t4 cell=checker_tier_keys_compare groups_with_a_different_line=56 of 64`；`name=rerun_compare 54 of 54 lines of out/rerun-expected.txt found`。与攻方报告逐项对得上（抬 F 那一行 `mount_placements_find` 本机重跑是 84292、攻方是 126438，不在预期文件里，判决不用这个数） |
| 正推腿的数 | 正推报告 T2 的计数（整份重写 n = 64→256 时 `written_find` 3159→37949）与 T3 自打的三条变异没有单独复跑 | T2 那两处（第 14576、14583 行）主 agent 读码核过是按重写的每个角色在 `units` 上线性找；T3 三条缺口与攻方 MA1–MA3 改的是同几处，两条腿各自独立量到同一结论，判决只引攻方复跑的数 |

## 二、判定与去向

| 格 | 正推 | 攻方 | 判定 |
|---|---|---|---|
| T1 七处改法的行为 | 成立（取最先一项、挪早前后同一份上一版、删参数之后同一版、文案与错误条件不变；倒打补丁比两串池的镜像相同） | 没打中（512 段随机合法历史、33792 行改前改后逐行相同，4000 次搬留存单元、零 panic） | **成立**。两条腿各走一条路（读码与两串固定池、随机历史差分），不共用代码与采样。查不到的几支差分没走到，只靠读码 |
| T2 射程判得对不对 | 部分不成立（8 处单次调用成立；`mount.rs` 第 5529 行不只挂载；发布路径漏了 `publish_admitted` 两处按角色找与几处 `.contains`） | 打中一处（`take_the_mapping_positions_over_the_location_hints` 每搬一项在 `units` 上找一遍，恢复时 P²/2），另报两处随写大小平方 | **打中，坐实**：① 第 3767 行整理发批每项 O(P)、恢复时 P²/2；② 第 14576、14583 行发布时每个重写角色 O(P)；两处与七·6 去掉的同形，不在七·6 点名的位置里。③ 第 4338、4820、11713 行三处 `.contains` 随一次写的单元数平方涨、不随池大小。8 处单次调用判成立（两条腿都量或读到每次发布常数次、每次 O(P)）；`mount.rs` 第 5529 行阶数判断成立、名字判错（也经会话准入推的抬 F 走到，`mounted_session.rs` 第 3494 行）。去向见第五节用户定 |
| T3 计数与新用例 | 部分成立（三条用例数产品调用点；19 行在终稿上逐行重证全红；自打三条变异快档全绿） | 打中（MA1–MA5 快档全绿，MA1–MA4 慢档也绿，只有 MA5 慢档红） | **打中，坐实**：① 删快照与按 inode 号找文件两处在发布路径上只判「至少」或不判建索引次数，每项重建不红（MA1、MA2、MA3）；② 建在计数作用域之外的索引记进没点名的那一处，没有用例判它（MA5）；③ 多层码 2 树只照抄 1 个节点，整数遍判法空真，记账树与 deadlist 树退回线性不红（MA4）；④ 核源落点「没有留存项就建 2 份」那一支没取样（正推）。19 行原有变异在终稿上全红（正推重证），不受证红之后两处改动影响 |
| T4 `links` 与构建脚本 | 成立（只交元数据；单编与 dev 依赖一起编各交对的那一份；`links` 只此一处） | 打中一种编法（环境或 `--config env` 里带 `CARGO_FEATURE_LOOKUP_WORK_COUNTERS`、不开特性：元数据报开着，64 组判器里 56 组的键变了；去掉变量再编构建脚本不重跑） | **打中，坐实**：构建脚本把进程环境里所有 `CARGO_FEATURE_*` 当 cargo 开的特性，外带的同名变量分不出（判别子观测不到，`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」那张表第二行）。仓里脚本今天不这样设变量（推的，没全仓搜）；错的是复用键，红不会被判成绿这一步是推的。产品与发布构建、`links` 撞名两问成立 |

## 三、交给下一批的实现员（按文件聚簇）

落在 `crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/build.rs`（连同 `crates/singlefs-unit-graph/` 读元数据的那一侧，要改就同一批）、`crates/singlefs-harness/tests/publish_lookup_by_identity.rs` 与 `crates/mutations.tsv`：

1. **T2 ①②**（用户定并进）：`take_the_mapping_positions_over_the_location_hints` 的按角色找改成一次建、逐项查（整理发批与恢复两处调用都受益）；`publish_admitted` 里 `written_units`、`named_unit_of` 两处改走这一次发布已有或一次建的按角色索引。每处进计数（`LookupSite` 新成员），每处一条改回线性的变异，点名的用例红；行为一字节不变（两档等价池 `images_sha256` 照旧）。验收：攻方探针那几格（一批 8 项、单元 159/599/2354）的比较次数不再随单元数涨。
2. **T2 ③**（用户定并进）：第 4338、4820、11713 行三处 `.contains` 改成按集合查，同样每处一条变异、一条会红的用例（取样要让一次写的单元数在两档之间变）。
3. **T3 ①–④**：删快照、按 inode 号找文件两处在发布路径上判建索引次数恰好等于走了几遍；没点名那一处（`None`）的建索引次数在两档大小下不涨，或把照抄 extent 树那一处的作用域包住建；整份重写那条用例让记账树或 deadlist 树照抄到不止一个节点、按树分开数；核源落点取样补一批没有留存项的。验收：攻方 MA1–MA5 与正推自打的三条照各自改法加进 `crates/mutations.tsv`，每条点名的用例红。
4. **T4**：构建脚本交出的特性名只认 cargo 这一次真开的（攻方改法丁提的 `CARGO_CFG_FEATURE`，或别的取法，实现员读 cargo 文档定并写依据），带一条会红的检查：外带 `CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1` 而不开特性时元数据不报开着。
5. **措辞**：里程碑四「优化项」与调度记录里把 `mount.rs` 第 5529 行那一串写成「挂载路径」的地方，改成「抬回退下限那一串（挂载、卸载、会话准入推的抬 F）」（主 agent 改，不归实现员）。

改法都是零轮形态（攻方改法甲到丁只在它自己的模型上推过）；这一批改完照验证段合入、轻型三条，再开这一题的第二轮攻它。

## 四、复跑与核不动的

- 正推腿的读数没有单独复跑，见第一节。
- T1 判成立靠两条腿各自的路子，不是只凭攻方一次「没打中」。
- 崩溃状态没有逐个枚举（两条腿比的是每一步之后写出的镜像）；崩溃轻型这一段在本机跑，结论另报。
- 恢复时那一半（P²/2）只在攻方探针的重挂场景量过一次；挂载路径的总挂钟没量。

## 五、交用户

| 岔路 | 事实 | 候选 |
|---|---|---|
| 整理与发布路径上漏的两处按项线性查找（T2 ①②）怎么办 | 第 3767 行一批 8 项、单元数 159/599/2354 时比较 1004/4076/16364 次，恢复时 2741311 次（攻方复跑，主 agent 重跑对上）；第 14576、14583 行整份重写 n = 64→256 时 3159→37949 次（正推量的，没复跑） | 并进下一批 / 只并整理与发布那半 / 记进优化项不改。**用户 2026-10-22 定：并进下一批**（弹窗选「并进下一批」） |
| 随一次写的单元数平方涨的三处 `.contains`（T2 ③）怎么办 | 快照下整份重写 N = 8/32/128 时 882/3636/49962 次（攻方复跑，主 agent 重跑对上）；不随池大小，不在七·6 题面里 | 记进优化项 / 一起并进下一批。**用户 2026-10-22 定：一起并进下一批**（弹窗选「一起并进下一批」） |

## 回看决策

不涉及决策：这一轮判的是七·6 后续批的查法、计数与测试判别力和 `singlefs-core` 构建脚本交出的元数据，没有改动或确立 `.claude/kb/decisions/` 下任何一条的说法；压着的条款是 `.claude/kb/milestone/04-fourth-txn.md` 七·6 那一行与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。构建脚本那一格压着 D13（验证路线） 已定项 17 的键组成，这一轮只报出外带环境变量时键与真编的不符，不改它的说法。
