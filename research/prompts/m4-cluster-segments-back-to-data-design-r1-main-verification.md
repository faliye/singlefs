# m4-cluster-segments-back-to-data-design-r1：判决（2026-11-18）

正文 `research/prompts/_m4-cluster-segments-back-to-data-design-r1-body.md`，开工快照 `research/prompts/m4-cluster-segments-back-to-data-design-r1-snapshot/`。设计推论轮第一轮；这一题此前没有判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m4-cluster-segments-back-to-data-design-r1-sonnet-output.md` | 新加候选戊并推荐：用户数据的候选序改成这次挂载开过的聚簇段之外先、之内后，之内有活槽的段先于全空段，开放段永不给；被拒那一刻还得回至少 49265 对；甲不够还把全空段送给数据；丙在被拒那一刻回 0；丁留作第二步；准入的读数与上界改成总槽对数减开放段 |
| 攻方（Opus） | `research/prompts/m4-cluster-segments-back-to-data-design-r1-opus-output.md`，模型 `research/prompts/m4-cluster-segments-back-to-data-design-r1-opus-model/` | 四条候选全部打中；规则暗中护着三样条款没写的东西（全空段储备、上界靠集合不缩、整理的资格与产出谓词）；丙最脆弱；「整理在本次挂载里也还不回来」成立，另有三道独立的挡 |

核查：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/m4-cluster-segments-back-to-data-design-r1-model-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-cluster-segments-back-to-data-design-r1 --out ~/.local/share/singlefs/work/m4-cluster-segments-back-to-data-design-r1-model-rerun/m4-cluster-segments-back-to-data-design-r1-tree --repository /home/fy5090/code/singlefs`，任务 20215 退 0。
- `python3 scripts/agents/cite-check.py research/prompts/m4-cluster-segments-back-to-data-design-r1-sonnet-output.md research/prompts/m4-cluster-segments-back-to-data-design-r1-opus-output.md --root ~/.local/share/singlefs/work/m4-cluster-segments-back-to-data-design-r1-model-rerun/m4-cluster-segments-back-to-data-design-r1-tree --background research/prompts/_m4-cluster-segments-back-to-data-design-r1-background.md`，末行「✓ 核了 83 处引文、122 处裸行号，对上 29 处，没判 176 处（其中裸行号没有引文可比 16 处；逐处列在上面）」；快照只记 kb，引 `crates/` 的行它列成没判，判决只引主 agent 现读过的行与复跑出的读数。
- 攻方的计数模型（纯 python，四段历史 × 八条臂 × 两档回收滞后）：主 agent 拷到 `~/.local/share/singlefs/work/m4-cluster-segments-back-to-data-design-r1-model-rerun/model` 复跑，任务 20212 退 0，64 行 `name=cluster_result`，`two_runs_identical=yes`。它量的是腿自写的模型，不是核心层：模型里今日臂在造池形的历史上第 20834 次用户发布被拒、拒因是 `empty_cluster_segments_below_the_floor`，核心层量到的是第 21987 次发布、拒因 `SlotPairsOutsideTheClusterSegmentsForDataUnits`（任务 20071）；次数相近、拒因不同，模型的读数只拿来比臂与臂之间的次序，不当绝对值。
- 主 agent 现读（主工作区）：`crates/singlefs-core/src/compaction_plan.rs:240`「is_opened_in_this_mount: cluster_segments.contains(&start_slot),」；`crates/singlefs-core/src/allocator.rs:887`「只会比真放开之后成的对数多，不会少」；`allocator.rs:2289`–`:2293` 与 `:3075`（集合只进不出，上一份调度记录里核过）。
- 核不动：正推的「还得回至少 49265 对」「需求约 3040–3214 对」是从任务 20071 的探针行与装置常量推的，主 agent 没复算；攻方「丙放宽资格之后 R 可以是开放段、形成活锁」「签发前判空洞数为 0」是读码推的；整理的选段资格在 Verus 证过的 spec 里（两腿都引 `crates/singlefs-replay-prefix` 下的 `compaction_plan_proofs.rs`），主 agent 没读。

## 二、判决

模型的读数（任务 20212，回收滞后取 4 的那一档；数是被拒时的用户发布序号，越大撑得越久）：

| 历史 | 今天 | 甲 全空即还 | 乙 至多一个活槽即还 | 乙 至多两个 | 只关开放段（近似戊） | 丙 整理搬空再还 | 丁 复用稀疏段 | 丁加甲 |
|---|---|---|---|---|---|---|---|---|
| 造池形 | 20834 | 21025 | 28869 | 28982 | 28705 | 54985 | 68418 | 68418 |
| 长寿的提交内生块多 | 18121 | 18121 | 18121 | 18121 | 20828 | 26452 | 43906 | 43906 |
| 映射节点钉住段 | 20546 | 20546 | 23701 | 24303 | 24303 | 20546 | 66754 | 66754 |
| 先建一阵元数据再删 | 6535 | 6553 | 7069 | 7422 | 7519 | 7506 | 7203 | 7227 |

逐条判（第一轮，打中的先挂起）：

- 甲（全空即还）：打中，两腿一致。被拒那一刻全空的聚簇段只有 8 个（任务 20071 的直方图 `0:8`，量的），模型里四段历史上甲与今天几乎一样；这 8 段还是提交内生块开新段的储备。甲单独不解这件事。
- 乙（低于门槛即还）：打中。门槛是负载的函数：每段钉住的活槽数超过门槛的历史上一段都不还（模型「长寿的提交内生块多」那一行，乙两档与今天同为 18121）。
- 丙（整理搬空再还）：打中，两腿一致判它最脆弱。整理今天不选这次挂载开过的聚簇段（`compaction_plan.rs:240` 主 agent 现读属实）；映射节点钉住段的历史上丙与今天相同（20546）；要改 Verus 证过的选段资格、补「R 不是开放段」。模型里它在造池形的历史上撑到 54985，是在模型假设整理做得到的前提下。
- 丁（提交内生块复用稀疏的聚簇段）：模型里四段历史中三段撑得最久（68418、43906、66754），但它不还空间：「先建一阵元数据再删」的历史上丁（7203）不如只关开放段（7519），删空的段在本次挂载里照旧对数据关着；它与 D3（空间分配） 已定项 10 ①「段起点 = 最低的全空段」的字面不能同真（攻方读的，主 agent 没核那一句的原文）。丁管的是聚簇段个数怎么涨，不管已经积下的。
- 戊（正推新加：数据候选序改成集合外先、集合内后，开放段永不给）：零轮，攻方没攻到它本身；模型里最接近的臂「只关开放段」在四段历史上都比今天久（28705、20828、24303、7519），但只多出三到四成，而正推算的「被拒那一刻还得回至少 49265 对」是这一刻的存量。两个读数量的不是一件事：一个是这一刻放出多少，一个是放出之后还能撑多久。
- 规则护着的东西里条款没写的三样（攻方，采为第二轮必答）：这次挂载开过、已经回收空的聚簇段是提交内生块开新段的全空段储备；`allocator.rs:887` 那个上界「只会多不会少」靠集合不缩成立，集合会缩之后会出现判不下、推一串空发布之后却落得下；整理的资格与产出谓词（数全空段）与「还给数据」方向相反。
- 「整理在本次挂载里也还不回来」：两腿一致判成立，主 agent 现读坐实其中一道（选段不选这次挂载开过的段）；其余两道（签发前判、映射节点钉住）是读码推的。
- 崩溃一致性：甲、乙、丁、戊只动内存里的集合与候选序，重挂之后的落点规则与今天相同（两腿一致，推的）；丙多一类整理意图。

第一轮两腿结论不一致、有打中，照设计推论轮开第二轮。第二轮攻的方向：戊与丁各自的修补与组合（戊管存量、丁管增量，合起来够不够）、全空段储备与上界那两样暗中的保证在戊与丁下怎么保、「先建一阵元数据再删」那段历史。

## 三、改法

- 戊（数据候选序改成集合外先、集合内后，开放段永不给；准入的读数与上界改成总槽对数减开放段）：零轮。
- 丁（提交内生块优先落进这次挂载开过、有空槽的聚簇段）：零轮；模型量过一个自写模型。
- 戊加丁的组合：零轮，没有模型臂。
- 丙：三处要先解（选段资格、签发前判、产出谓词），这一轮不采。

## 交用户的

暂无；第三轮之后列。

## 这一轮被判过的文件

- .claude/kb/decisions/03-空间分配.md：判了已定项 8 第 2 条「聚簇段在挂载期间对用户数据关着」护着的东西里哪几样在稀疏段、全空段上还成立，四条候选加正推新加的一条都挂起，条款这一轮不改。
- .claude/kb/decisions/26-后台整理与放置回收.md：判了已定项 1 的选段资格与产出谓词和「靠整理搬空再还」那条候选的关系（今天的整理不选这次挂载开过的聚簇段），条款这一轮不改。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 8 | 备料 | 2026-11-18 不受影响：候选都挂起，等后两轮，条款不改 |
| D3（空间分配） 已定项 10 | 备料 | 2026-11-18 不受影响：复用稀疏段那条候选与「段起点取最低的全空段」的关系等第二轮，条款不改 |
| D26（后台整理与放置回收） 已定项 1 | 备料 | 2026-11-18 不受影响：靠整理那条候选这一轮不采，条款不改 |
