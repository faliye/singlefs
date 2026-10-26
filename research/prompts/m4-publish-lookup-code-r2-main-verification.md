# m4-publish-lookup-code-r2：里程碑四七·6 发布路径按身份查，代码轮第二轮判决（2026-10-21）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

被判的产品代码：`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mounted_session.rs`（七·6 第二批补丁，`research/prompts/_m4-publish-lookup-code-r2-diff.md`）。测试代码 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs` 与 `crates/mutations.tsv` 的五行追加、两行换名一并判 T2、T4。

开工快照：`research/prompts/m4-publish-lookup-code-r2-snapshot/`（crates-sha256.txt 于 2026-10-21 事后补：两行原样抄自同目录 sha256sums.txt，那份是腿派出时记的）

两条腿都在开工几分钟后因客户端关闭被停，各新派一条同立场的腿接着做（`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 当天那一条）；判决只引接续那两条腿的报告。

## 一、核查

| 核了什么 | 怎么核 | 结果 |
|---|---|---|
| 两条腿的引文 | `python3 scripts/agents/cite-check.py research/prompts/m4-publish-lookup-code-r2-sonnet-output.md research/prompts/m4-publish-lookup-code-r2-opus-output.md --root ~/.local/share/singlefs/work/m4-publish-lookup-code-r2/repo --background research/prompts/_m4-publish-lookup-code-r2-background.md` | 「✓ 核了 66 处引文、80 处裸行号，对上 66 处，没判 80 处（其中裸行号没有引文可比 80 处；逐处列在上面）」。判决要用的裸行号主 agent 在冻结副本里逐个读：`transaction.rs` 第 9543–9551 行是 `relocation_checked_against_the_version` 里 `.mapped_units.iter().find(\|(_, key)\| *key == mapping_key)` 加 `central_mapping_locations_in_the_version(previous, &mapping_key)`；第 4412–4414 行 `leaves_decoded` 返回 `self.leaves_read.len()`，上面第 4400–4404 行是 `self.leaves_read.entry(leaf_role).or_insert_with(…)`；第 3305–3309 行 `mapping_key_of` 的 `.expect("照抄进来的角色在上一版里进映射、有一把 key")`；`mounted_session.rs` 公开函数只有第 2830 行一个调用点，它所在的私有函数只在第 3018 行被调用；测试第 915–943 行 `comparisons_of_looking_up_every_identity` 在测试自己包了计数身份（第 872–892 行 `IdentityCountingItsComparisons`）的表上另建索引来数 |
| 开工快照 | `sha256sum -c research/prompts/m4-publish-lookup-code-r2-snapshot/sha256sums.txt` | 11 个文件全部 OK，没有要倒推的 |
| 攻方的复跑数 | 模型目录拷到 `~/.local/share/singlefs/work/m4-publish-lookup-code-r2-main-check/model/`（`SHA256SUMS` 逐个核过），`bash scripts/hosts/run-with-memory-cap.sh 16G bash <那份>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m4-publish-lookup-code-r2/repo --work <草稿>/run --model <那份>` | 退 0。原样输出：`name=run mutant=baseline test=publish_lookup_by_identity rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=118 growth_in_percent=110`；MA、MC、MD、MF、MI 五条在 `publish_lookup_by_identity` 上 `rc=0 test result: ok. 6 passed`、读数同为 118、110，MA、MD、MG 在 `snapshot_release_and_destroy_through_the_deadlist` 上 `rc=0 test result: ok. 15 passed`；ME `rc=101 test result: FAILED. 0 passed; 1 failed`，MH `rc=101 test result: FAILED. 6 passed; 9 failed`，探针 P1 `rc=101 test result: FAILED. 0 passed; 1 failed`；`name=differential before_vs_after=identical lines=15`；`name=rerun_compare 24 of 33 lines found in out/rerun-expected.txt`——多出的 9 行是脚本打变异时印的 `applied <变异名> <文件>`，预期文件的 24 行全在；攻方报告写「24 of 24」是没整条跑过脚本、分母写错，结果行不受影响 |
| 正推腿的数 | 正推报告 T2 的读数（未变异 110%、118%；两行增长变异与「从大到小逐项找」391%、398%；变体 V1 六条全绿）没有单独复跑 | 未变异读数与攻方复跑同值；V1 与攻方 MC 改的是同一处（批量读者每把 key 重解叶），两条腿各自独立量到同一结论，判决只引攻方复跑的数 |

## 二、判定与去向

| 格 | 正推 | 攻方 | 判定 |
|---|---|---|---|
| T1 公开的那一半是不是整理真用的那一半 | 成立（三个整理入口汇到唯一调用点；抽出前后原函数体一字未动；另报 `transaction.rs:9548` 那处线性找不经公开函数） | 打中一处（同一处 `relocation_checked_against_the_version`，换成 panic 的探针在十六文件那档历史上撞到；抽出前后镜像摘要逐行相同；没找到绕开不变量的路） | **成立，另有一处范围外的线性查找坐实**：整理读版本取映射条目只经公开函数，抽出前后行为相同。整理发批核源落点那一步（第 9543–9551 行）按计划里每把 key 在 `mapped_units` 上线性找、再逐 key 走一遍映射，补丁前后都没动，不在七·6 点名的五处里，第一轮与背景材料的「已知」清单都没列。它不是这一批补丁的行为反例，是整理路径上第六处随表长涨的查找，去向交用户（第五节） |
| T2 增长判据数的是不是增长本身 | 部分成立（两个数都不是产品调用点的数） | 打中（判别子观测不到） | **打中，坐实**：解叶数读的是缓存表的键数（第 4412–4414 行），不是解叶的次数；比较次数取自测试自己另建的索引（测试第 915–943 行），产品调用点怎么查都进不了这个数。批量读者每把 key 重解叶（攻方 MC、正推 V1）与几处查法退回线性（攻方 MA、MD、MI），点名用例全绿、两个读数一直是 118、110。判据要改成数产品调用点上的次数 |
| T3 快照判定那一圈换了查法之后的行为 | 成立（八种构造改前改后逐行相同；取最先一项；panic 文案逐字相同，只是位置从第 7074 行挪到第 3308 行） | 没打中（快照历史差分 14 行逐行相同；判别力不只靠产品断言：变异 MH 红在测试自己的留存条目断言上，15 条红 9 条） | 成立。两条腿各走一条路（读码与构造、差分探针），不共用代码与采样 |
| T4 新加的五行与换名的两行 | 部分成立（七行各自红；找到两处改回去没有任何一行红：V1、V4） | 打中两处（「每把 key 现建一个批量读者」那行红只因外层读者没人用、片数读成 0，代价相同而保住计数的写法 MF 不红；改回去不红的有 MG（快照圈退回线性）与 MC（每把 key 重解叶）） | **打中，坐实**：七行变异都红在点名的测试上；但第二批的两处改法改回去没有一行红——快照判定那一圈退回线性（攻方 MG、正推 V4）、批量读者每把 key 重解叶（攻方 MC、正推 V1）。「每把 key 现建一个批量读者」那一行判的是计数没对上，不是重解叶本身。根子与 T2 同一个：判据数不到产品调用点 |

## 三、交给下一批的实现员（按文件聚簇）

落在 `transaction.rs`、`mounted_session.rs`、`publish_lookup_by_identity.rs` 与 `crates/mutations.tsv`，一个实现员一次改完：

1. **T2、T4 同一个根子**：增长判据与变异要判到产品调用点。在产品里数七·6 五处查找与整理读版本实际做的事——每次查走过几项、建了几次索引、解了几次叶（不是解过几片）——计数只在测试构建开（一个默认关的特性，harness 档的 dev 依赖打开它；写明加在哪、在哪个特性下开），不读挂钟与 CPU 时间。测试改成读这些计数判增长与「每片叶至多解一次」。验收：攻方 MA、MC、MD、MF、MG、MI 六条（攻方模型目录 `mutants.tsv`）照它们的改法加进 `crates/mutations.tsv`，每条点名的用例红；未变异的用例绿；「七·6 整理读版本：每把 key 现建一个批量读者」那一行照新计数改点名，仍然红。
2. **增长慢档用例两档并行**：增长用例在一个线程里依次量 512 与 2048 两档（看门狗报过只用 0.9 核），两档互不依赖，照 `.claude/rules/test-script-discipline.md`「编译与算法都要快」切开并行。

产品代码有改动，改完照验证段整体再验，下一轮代码轮再攻（第三轮，照 `.claude/rules/three-way-inference.md`「第三轮之后停」，是七·6 的最终轮）。

## 四、复跑与核不动的

- 正推腿的读数没有单独复跑，见第一节。
- 「没打中」只抽一次样：T3 判成立靠两条腿各自的路子（正推读码与八种构造，攻方差分探针），不是只凭攻方一次「没打中」。
- 攻方 MB（解叶退回平方）慢档跑了约 13 分钟没跑完、被腿停了，不在复跑里，判决不用它。
- 崩溃状态没有逐个枚举（两条腿的写流逐条相同，崩溃状态集合由写流定，这一步是推的）；崩溃轻型在验证段跑过。
- 攻方提的改法（产品里数次数、只在测试构建开）只在它自己的模型上推过、没实现，被攻过零轮；第三节照它交实现员，改完由第三轮攻。

## 五、交用户

| 岔路 | 事实 | 候选 |
|---|---|---|
| 整理发批核源落点的线性查找（`transaction.rs` 第 9543–9551 行，冻结副本行号）并不并进七·6 | 七·6 点名五处（`.claude/kb/milestone/04-fourth-txn.md` 七·6 那一行，用户 2026-10-20 定「先修四处线性查找」），这一处不在其内；它每批对计划里每把 key 线性找一遍 `mapped_units`、再逐 key 走一遍映射，代价随表长与批大小相乘，没量过（两条腿都写的是推的）。同类的还有背景材料已列的 `mapping_locations_of_a_mapped_unit` 与快照判定那一圈进 deadlist 那一支（正推报 5748→4295 仍线性） | 并进第三节那一批一起改（同一个实现员、同一批文件，七·6 的射程从五处扩到这三处）；或记进里程碑四「优化项」表，等 E185 第二段重跑量过再定。**用户 2026-10-21 定：并进下一批**（弹窗选「并进下一批」） |

## 回看决策

不涉及决策：这一轮判的是七·6 第二批的查法与测试判别力，没有改动或确立 `.claude/kb/decisions/` 下任何一条的说法；压着的条款是 `.claude/kb/milestone/04-fourth-txn.md` 七·6 那一行与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。
