# m4-publish-lookup-code-r3：里程碑四七·6 发布路径按身份查，代码轮第三轮判决（2026-10-21）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

被判的产品代码：`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mounted_session.rs`（七·6 第三批补丁，`research/prompts/_m4-publish-lookup-code-r3-diff.md`），连同 `crates/singlefs-core/Cargo.toml` 的特性声明。测试代码 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs`、`crates/singlefs-harness/Cargo.toml` 与 `crates/mutations.tsv` 末尾七·6 那几行一并判 T3、T4。这是七·6 代码轮的第三轮、最终轮。

开工快照：`research/prompts/m4-publish-lookup-code-r3-snapshot/`

## 一、核查

| 核了什么 | 怎么核 | 结果 |
|---|---|---|
| 两条腿的引文 | `python3 scripts/agents/cite-check.py research/prompts/m4-publish-lookup-code-r3-sonnet-output.md research/prompts/m4-publish-lookup-code-r3-opus-output.md --root ~/.local/share/singlefs/work/m4-publish-lookup-code-r3/repo --background research/prompts/_m4-publish-lookup-code-r3-background.md` | 「✓ 核了 142 处引文、68 处裸行号，对上 142 处，没判 68 处（其中裸行号没有引文可比 68 处；逐处列在上面）」。判决要用的裸行号主 agent 在冻结副本里逐个读：`transaction.rs` 第 4053–4058 行 `TransactionOutput::unit` 仍是 `self.units.iter().find(\|unit\| unit.identity == identity)`；第 9986 行 `relocation_checked_against_the_version` 里 `RelocatedUnitHolder::RoleOfThisVersion(role) => previous.unit(role).bytes.clone()`；第 9917 行同一函数里 `.find(\|(key, _)\| *key == mapping_key)`；第 3972 行 `file_other_than_the_first` 的 `.find(\|file\| file.inode == inode)`；第 6116 行 `deadlist_changes_of_this_publish` 里 `.find(\|(key, _)\| *key == entry.mapping_key)`；第 11854 行 `build_extent_tree` 里 `previous_version.unit(role_in_the_previous_version)`；`crates/singlefs-checker-tier/build.rs` 第 73–82 行只按本包 `CARGO_FEATURE_*` 收特性；`crates/singlefs-unit-graph/src/closure.rs` 第 348–351 行非根的包只顺 `DependencySection::Normal` |
| 开工快照 | `sha256sum -c research/prompts/m4-publish-lookup-code-r3-snapshot/sha256sums.txt` | 13 个文件全部 OK，没有要倒推的 |
| 攻方模型 | 模型目录拷到 `~/.local/share/singlefs/work/m4-publish-lookup-code-r3-main-check/model/`，8 个文件与攻方报告开头列的 sha256 逐个相同 | 一致 |
| 攻方的复跑数 | `bash scripts/hosts/run-with-memory-cap.sh 16G bash <那份>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m4-publish-lookup-code-r3/repo --work <草稿>/run --model <那份>` | 退 0。原样输出：`name=mutant mutant=baseline rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100`；RA、RB、RC、RD、RE、RF、RG、RG+MA 八行都是 `rc=0 test result: ok. 6 passed`、读数与基线相同；`name=mutant mutant=MA rc=101 test result: FAILED. 3 passed; 3 failed`；`name=differential_compare histories=192 before_vs_after=identical endings=ending=completed:192`；`name=rerun_compare 37 of 37 lines of out/rerun-expected.txt found`。与攻方报告逐项对得上 |
| 正推腿的数 | 正推报告的读数（边界用例 2033 次比对、改前改后三串操作镜像摘要 9 行相同、M7 慢档增长 123%）没有单独复跑 | 与攻方各自独立量到同一侧结论（T1、T2 行为相同；改回一半与每项重建不红），判决只引攻方复跑的数 |

## 二、判定与去向

| 格 | 正推 | 攻方 | 判定 |
|---|---|---|---|
| T1 前两轮判成立的，在新索引下还成立吗 | 成立（三种身份类型 `Ord` 与 `==` 都派生、一致；边界用例 2033 次比对与线性找相同；panic 文案逐字不变）；另报等价改法「稳定排序换不稳定排序」快档全绿，同一身份取最先一项在大表上没有测试守 | 没打中（192 段随机历史改前改后每一步镜像摘要、结局、录制流步数逐行相同） | **成立**。两条腿各走一条路（读码与边界比对、随机历史差分），不共用代码与采样。正推报的「不稳定排序不红」是测试判别力的缺口，并进第三节 |
| T2 并进来的两处查法换了之后的行为 | 成立（两处调用点逐处列出，改前改后取同一项；三串操作镜像摘要 9 行相同；错误路径只读码） | 行为上没打中（48 段里 46 段走到释放三处与快照判定圈；核源落点只走到 6 段、共 128 次查，偏薄；查不到时的 panic 与 `Err` 逐字照抄） | **行为成立**。核源落点那一处覆盖偏薄，记进第三节的测试补强 |
| T3 计数判的是不是产品调用点上的事 | 部分不成立（核心层没有起线程、用例并行不串数、200% 门槛站得住；但 M2、M3、M4、M7 四类改坏快档全绿，另有五处线性找不在计数里） | 打中（RA–RF 六条改回一半或每项重建，`publish_lookup_by_identity` 六条连慢档全绿：逐 key 查里那次 `version.unit` 线性找不记数；核源落点那一处 K 定死、每项重建在两档上同倍涨，比值不动；释放三处合记一个点、断言只要「至少」） | **打中，坐实**：① 用户定并进的整理发批核源落点没有改完：第 9917 行按 key 线性找、第 9986 行经 `TransactionOutput::unit` 线性找，后者每批 K 项各走一遍 `units`，都不进计数；② 七·6 点名的几处只有批量形态改成了索引，单次调用的 `TransactionOutput::unit`（第 4053 行）、`file_other_than_the_first`（第 3972 行）仍线性，发布与整理路径上还在调（第 9986、11854 行等）；③ 判据数不到「改回一半」：释放三处合记一个点、只断言「至少」，核源落点的批大小在测试里定死，每项重建在两档上同倍涨，比值判据看不出 |
| T4 特性与变异 | 部分不成立（不开特性时计数编没了；判法单元键不走 dev 边、特性不进键；第三批改了 9 组判器里 5 组的键；改回去无一行红的 5 处） | 打中两件：计数本身改坏（RG，删掉逐项走的那一句计数）六条全绿、叠上 MA 后 MA 不再红；一起编 harness 测试时 `singlefs-core` 开着计数编，checker 档构建脚本却按不开那一把键算（unit-graph 量的：不开 `128ef4…`、开 `e7b303…`） | **打中，坐实**：① 计数器自己没有会红的检查；② 判法单元键看不到别的包并上来的特性：`crates/singlefs-checker-tier/build.rs` 只读本包的 `CARGO_FEATURE_*`，`crates/singlefs-unit-graph/src/closure.rs` 对非根的包只顺普通依赖，harness 的 dev 依赖替 `singlefs-core` 打开的 `lookup-work-counters` 它看不到——一起编出的二进制跑的是开计数的代码、顶着不开那一把键，是错误复用那一侧（正推说「特性不进键」，判的是同一件事的另一面：它没进键，所以开不开键都一样）。今天计数对行为中性，红不会判成绿，这一步是推的；这是判别子观测不到（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」那张表第二行），不是七·6 的行为错 |

## 三、交给下一批的实现员（按文件聚簇）

落在 `transaction.rs`、`mounted_session.rs`、`singlefs-core/Cargo.toml`、`singlefs-checker-tier/Cargo.toml`（看第五节用户怎么定）、`publish_lookup_by_identity.rs` 与 `crates/mutations.tsv`：

1. **T3 ①②：七·6 射程里剩下的线性查找改完**。整理发批核源落点第 9917、9986 行；`TransactionOutput::unit` 与 `file_other_than_the_first` 的单次调用在发布与整理路径上的调用点改走已有索引（第 11854 行起的 `build_extent_tree` 等，实现员逐处列出调用点、判它在不在发布与整理路径上）；`deadlist_changes_of_this_publish` 第 6116 行找留存条目。每处进计数。验收：每处一条改回线性的变异，点名的用例红；行为一字节不变。
2. **T3 ③：判据判得到「改回一半」**。释放三处分开记、各自断言；核源落点那一处的批大小在两档上随池大小变或另加一条按批大小取样的用例，让每项重建与线性找在比值上分得出。验收：攻方 RA–RF、正推 M2、M3、M4、M7 照各自改法加进 `crates/mutations.tsv`，每条点名的用例红。
3. **T1 附带**：同一身份取最先一项在大表上要有测试守，验收是正推 M1（稳定排序换不稳定排序）点名的用例红。
4. **T4 ①**：计数器自己要有会红的检查，验收是攻方 RG 点名的用例红。
5. **T4 ②**：照第五节用户的选择改。

增长慢档用例两档并行、七·6 已有的其余行不动。

## 四、复跑与核不动的

- 正推腿的读数没有单独复跑，见第一节。
- T1、T2 判成立靠两条腿各自的路子，不是只凭攻方一次「没打中」。
- 崩溃状态没有逐个枚举（两条腿都写明了，差分比的是每一步之后的镜像）。合入后崩溃轻型这一趟判完 24 项全绿，七·6 点名的四项在内；`make-filesystem-over-a-used-pool` 及其后各项本次未跑，归里程碑三收尾会话批 14，批 14 进主工作区后整趟重跑。
- 攻方提的改法甲到戊、第三节各条改法都是零轮形态（第三轮之后不再开第四轮，`.claude/rules/three-way-inference.md`「第三轮之后停」）。

## 五、交用户

| 岔路 | 事实 | 候选 |
|---|---|---|
| 判法单元键看不到并上来的特性（T4 ②）怎么收 | 一起编 harness 测试时 `singlefs-core` 开着 `lookup-work-counters` 编，checker 档的键按不开算（第一节、第二节 T4）；今天计数对行为中性（推的，没量过） | 修键算法认并上来的特性 / checker 档也开这个特性 / 去掉特性计数常开。**用户 2026-10-21 定：修键算法，认并上来的特性**（弹窗选「修键算法，认并上来的特性」）：改 `crates/singlefs-unit-graph` 与 `crates/singlefs-checker-tier/build.rs`，按 cargo 实际解析出的每个包的特性算 cfg 原子的真值，单派一个实现员、与七·6 那一批文件不相交 |
| 第三节这一批改完之后怎么验 | 七·6 代码轮已到第三轮、最终轮；第三节改的是产品代码，照 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」要一轮代码轮，门禁 doc-process-records 的 crates-adversarial-review 格要按路径点名 | 另立一题开新代码轮 / 只验不攻、用户豁免。**用户 2026-10-21 定：另立一题开新代码轮**（弹窗选「另立一题开新代码轮」）：这一批改完照验证段合入、轻型三条，再开一个新名字的代码轮第一轮攻它 |

## 回看决策

不涉及决策：这一轮判的是七·6 第三批的查法、计数与测试判别力，没有改动或确立 `.claude/kb/decisions/` 下任何一条的说法；压着的条款是 `.claude/kb/milestone/04-fourth-txn.md` 七·6 那一行与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。判法单元键的那一件压着 D13（验证路线） 已定项 17 的键组成，这一轮只报出它在这一种编法下看不到并上来的特性，不改它的说法。
