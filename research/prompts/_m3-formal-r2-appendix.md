**出处 `.claude/kb/milestone/03-third-txn.md:1-10`（整段抄，未转述）**

```markdown
# 里程碑「删除与多次 COW」：作业点、验收与次序

<!-- doc-lint:not-numbers M1 M2 M3 Z3 -->

**口径**：规划文件，2026-09-26 建档、2026-09-28 按用户定案改成作业点表。**它不要求正确**：每项开工前回来改，改完在「历史版本」记一行。前一个里程碑是 [02-second-txn.md](02-second-txn.md)（收尾九步做到第 9 步，固定脚本到 E 全域在跑）；用户 2026-09-28 定里程碑三与里程碑二收尾并行开工（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 开头），里程碑二收口不再是这份开工的前置。

**读法**：每项四格——「原话」照录用户口述；「仓里已有的」写这一项今天在仓里压着哪些条款、欠账、代码（现查的，带出处）；「作业点」一张表：编号、做什么、压着的条款与欠账、验收（每条先证会红）、依赖；「开工前要先定的」列还没定、开工前要走三方或问用户的问题。凡写「推的」的是主 agent 没量过的判断。作业点的编号写成「项·点」（一·1、三·2），只在这份文件里用。

**新事务是删除**（`.claude/rules/fs-design.md`「从事务开始，不从功能开始」：一个里程碑一种新事务）。**这份文件只管文件系统本身、形式化验证、崩溃点与欠账**：崩溃放量的装置在 [02-second-txn.md](02-second-txn.md)「增补 4」，这里的流只是**跑在**那套装置上。

```

**出处 `.claude/kb/milestone/03-third-txn.md:11-24`（整段抄，未转述）**

```markdown
## 出口（用户 2026-09-28 定：删除事务、多次 COW、崩溃点审查与系统验证；2026-09-29 加形式化验证并定为优先）

五件，全部达成才算完，每件的验收在对应项的作业点表里：

| # | 出口 | 归哪一项 |
|---|---|---|
| 1 | 删除作为新事务落地：单文件 unlink（第一版没有目录，按 inode 号删）走意图、分批回收与 `deleted_inodes`；写出的新盘上形态登记进 `layout/03-third-txn.md`；固定脚本加删除段、崩在回收中间的崩溃点跑在崩溃放量上全域 0 红；I-4.4（意图只描述目标态）、I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-9.5（删除队列里的号仍在树里）、I-3.4（可用空间扣待删占用） 实现 | 三 |
| 2 | 多次 COW：至少三条新流跑在崩溃放量上，全域 0 红 | 一 |
| 3 | 项二点名的不变量全部实现（I-4.1（重放后树可遍历）、I-8.4（重放幂等）、I-8.1（环几何够大）、I-7.5（根槽按判定宽度对齐）、树 ID 水位、指针 key 与单元头相等、代字段），项二欠账簇还清或改写成别的里程碑的账 | 二 |
| 4 | 崩溃点审查和补充、系统调优和验证：写路径每个提交步骤与每个入口都被至少一条崩溃流罩住（覆盖表由脚本从录制流与登记表算出、门禁一格比对）；每条流至少一个必红世界；ignore 开着的节点清单只减不增；旋钮登记表带取值依据 | 六 |
| 5 | 形式化验证（用户 2026-09-29 定优先做）：crash refinement 的 M1（全性）与 M2（前缀不切发布）在选定的工具上证出，证明产物入库、门禁一道三种结果（证明 / 反例 / timeout）且 timeout 判红；每条性质各有一个必出反例的世界；M3 与 O3（独立规约执行器） 进不进出口按七·1 的代价数定 | 七 |

**不在出口**：崩溃放量装置本身（[02-second-txn.md](02-second-txn.md)「增补 4」）、并发（项四，暂定）、目录与 POSIX 面、快照与克隆、加密、压缩、第三方管道、扫描重建、后台整理（D26（后台整理与放置回收））、小数据打包容器（D27（小数据打包容器））、zoned、真设备崩溃注入之外的最终判据。

```

**出处 `.claude/kb/milestone/03-third-txn.md:25-44`（整段抄，未转述）**

```markdown
## 分组与次序

| 组 | 项 |
|---|---|
| 乙　文件系统本身 | 一至四 |
| 丙　欠账 | 五 |
| 丁　崩溃点与验证系统 | 六 |
| 戊　形式化验证（优先） | 七 |

| 段 | 做什么 | 为什么排在这 | 状态 |
|---|---|---|---|
| 0 开工 | 项五的清单；出口定稿 | 清单决定还有哪些活并进来 | 2026-09-28 做了（`records/2026-09-28-里程碑三第十项清单.md`；出口就是这一版） |
| 1 设计 | **七·1（选路与题面，优先，用户 2026-09-29 定）** ∥ 三·1（删除的设计轮）∥ 二·1 的不变量措辞 ∥ 六·1 崩溃点覆盖表（审查） | 四边互不依赖；七·1 先派、资源先给它 | 未开 |
| 2 实现 | **七·2 到 七·5（M1、M2、M3、门禁一道，优先）** ∥ 三·2、三·3 ∥ 二·2 到 二·5 按文件聚簇 ∥ 六·3 每条流的必红世界 | 四边写的文件不相交（形式化的 spec 与门禁、core 的 unlink 与模型、checker 的不变量、checker 档的用例与变异行） | 未开 |
| 3 覆盖 | 一·1、三·4、六·2 补的流跑在崩溃放量上，全域跑；七·6（O3（独立规约执行器））若按七·1 定做，排这一段 | 出口 1、2、4 的流在这一段满足 | 未开 |
| 4 优化 | 二·6（C567（分配与发布的代价随盘容量线性））∥ 六·4 旋钮登记与扫值 | core 的算法改动等删除与不变量落地之后再动，量了再改；旋钮要在补齐的流上扫才有意义 | 未开 |
| 5 暂定 | 项四 | 不在出口里；做不到转给里程碑四 | 未开 |

**估计**：不含项四三到四周，推的，没量过（参照：里程碑一建档到出口 4–5 天，里程碑二 2026-09-16 开工、2026-09-28 收尾九步做到第 9 步）。

```

**出处 `.claude/kb/milestone/03-third-txn.md:45-57`（整段抄，未转述）**

```markdown
## 一　多次 COW 下的读写一致，与崩溃重放测试

- **原话**：「多次cow下的读写一致，与崩溃重放测试」
- **仓里已有的**：崩溃放量今天两条流：固定脚本到 D 全域 51 773 558 态 0 红、到 E 再正常卸载（78 段）在跑（`briefs/2026-09-28.md`「崩溃放量」一节）。随机历史（`crates/singlefs-harness/src/history.rs` 的 `HistoryOperation`：第一个文件版本、覆盖写、空发布、关闭再可写挂载、挂着回退、抬 F、冷启动恢复、抛弃最新根的崩溃恢复）有多次覆盖写与多次挂载，但不做全量崩溃点重放；崩溃注入（`crates/singlefs-checker-tier/src/crash_injection.rs`）按点注入，不枚举。新流怎么写：公共前缀模块 `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs`，节点编码 `<流程>.<周期>.<步>`（用户 2026-09-28 定，[02-second-txn.md](02-second-txn.md)「增补 4」第一项）。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 一·1 | 定「多次 COW」的流清单并写成固定脚本流，至少三条：同一文件覆盖写不少于 3 次、中间关闭再可写挂载 2 次（跨三个实例）；多个 inode 交错覆盖写（并行线三的形态）；跨多个单元的大文件覆盖写（并行线一的形态）。每条流用公共前缀模块，节点按 `<流程>.<周期>.<步>` 编码，登记 `crash-case:` 行带 `exhaustive=` 计数行 | D13（验证路线） 已定项 4 段模型；D16（发布语义） 已定项 4 发布顺序；C445（模型对拍的取样点不随写路径增长） | 每条流全域 0 红；每条流有一处必红世界（改坏覆盖写的释放判定，那条流的判器红） | 崩溃放量装置已在，节点编码已定 |
| 一·2 | 随机历史加多次覆盖写与删除的操作种类，模型对拍的取样点随写路径增长；归项一的里程碑二欠账逐条现查还开不开：C143（inode 号水位在回退后会退回去重发）、C282（环里最旧根没有定义）、C314（回退可以复用被抛弃的根引用的单元）、C329（写行那次发布之前推抬 F 的空发布没有检查）、C330（中间实例那一行的 T_pub 取所选根的 txg）、C42（残留记录冒充合法前缀）、C77（重放起点未定义）、C464（I-1.8 对码 1 的全序键写窄了）、C551（`build_pool` 的分配器没有根环表） | C445（模型对拍的取样点不随写路径增长） | 快档里新增形态各有种子命中（`KNOWN_RED_FORMS` 不新增）；C445（模型对拍的取样点不随写路径增长） 还清或改写 | 删除那一种等 三·2 |

- **开工前要先定的**：一·1 的三条流是不是够「多次」。验证装置一侧的欠账（C8（门禁范围判不出来）、C353（journal 在恢复路径上的承重没有会红的检查） 那一批）归项六。

```

**出处 `.claude/kb/milestone/03-third-txn.md:58-74`（整段抄，未转述）**

```markdown
## 二　索引树更新与 journal 环与根的一致性

- **原话**：「索引树更新与journal环与根 的一致性」
- **仓里已有的**：发布顺序定在 D16（发布语义） 已定项 4（COW 单元 → 屏障 → journal 记录 → 屏障 → 根槽 → 系统配置槽）；checker 判 49 条不变量（`grep -c '^| I-.*已实现' .claude/kb/invariants.md`）；[invariants.md](../invariants.md) 里未实现且归这一项的：I-4.1（重放后树可遍历）、I-8.4（重放幂等）、I-8.1（环几何够大）、I-7.5（根槽按判定宽度对齐）、I-8.5（tail 槽有效集非空）（C367（I-8.5 的字面后半句在合法镜像上不成立） 说它字面后半句在合法镜像上不成立）、I-4.3（提交原子）（对一份镜像判不出，没硬写）；代字段不变量的措辞草稿已入库 `research/prompts/c395-invariants-draft.md`；多子叶的树分裂在里程碑二做了。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 二·1 | 不变量补齐：I-4.1（重放后树可遍历）、I-8.4（重放幂等）、I-8.1（环几何够大）、I-7.5（根槽按判定宽度对齐）、I-8.5（tail 槽有效集非空） 改措辞、树 ID 水位、指针里的 key 各段与被指单元头相等、代字段；主 agent 先给措辞，书记员写进 [invariants.md](../invariants.md)，实现员做 checker 判定与坏镜像 | C28（环几何不变量未实现）、C47（根槽没按判定宽度对齐）、C367（I-8.5 的字面后半句在合法镜像上不成立）、C147（树 ID 水位的不变量还没有）、C294（指针里的 key 各段与被指单元头相等没有不变量）、C395（代字段只有验收断言盯着）、C446（未实现的不变量没有前置登记，也没有到期） | 每条：checker 判定、坏镜像用例只红自己、`crates/mutations.tsv` 一行；I-4.3（提交原子） 仍判不出就在状态列写明为什么 | 无 |
| 二·2 | C585（会话发布放进发布途中取下的旧快照） 用户选的改法 C：四个入口同一判——落后且缺现行那一版的单元，或现行那一版的根写在这块盘上而那一槽里不是它，才拒 | D18（块里携带什么信息） 已定项 11「不落后」；改法被攻过零轮 | 攻方那 1268 格里的 58 格转成拒绝；用例与变异行；代码三方一轮 | 无 |
| 二·3 | C586（回退取树 ID 水位的那一半没有用例）：会红的用例与变异行；litmus 注释里一处过期路径；`crates/mutations.tsv` 第 736 行的靶名 | C586（回退取树 ID 水位的那一半没有用例） | 把 `.fold(current.root.tree_identifier_watermark, u64::max)` 改成 `.fold(0, u64::max)` 用例红 | 无 |
| 二·4 | 环与根的条款空白：C566（环上有洞时多扣的槽随洞数无界增长）——用户 2026-09-28 定「重排」是**重排根环的槽位**（可写挂载时把洞补上，让环上没有洞），动写路径与崩溃点，走一轮三方（形态、写几次、崩在补洞中间的状态）再实现；C380（环上有洞时释放代判不出唯一值） 随它一起判（洞补上之后还有没有对象）；C583（双故障形：每盘一槽系统配置坏加最新根一时读不出）——用户 2026-09-28 定写成「不罩」的条款，同日已写回 D22（单元原子性怎么合成） 已定项 8 射程、已钉的两条用例当登记、已挪进已还清；C596（撕裂的系统配置轮换之后只剩只读挂载）、C597（实例表行只增不删，删行的条件没有条款） | 各自的登记行 | C566（环上有洞时多扣的槽随洞数无界增长）：判决 + 补洞的必红世界（补洞那次发布崩在中间，判器红）+ 用例；C583（双故障形：每盘一槽系统配置坏加最新根一时读不出）：条款写回、C583（双故障形：每盘一槽系统配置坏加最新根一时读不出） 挪已还清；其余两笔条款 + 会红用例，或用户定不做并写明依据 | C566（环上有洞时多扣的槽随洞数无界增长） 的三方与三·1 分开开，可以并行 |
| 二·5 | 其余归项二的欠账簇，开工日逐条现查还开不开、按要动的文件聚簇派：里程碑二留下的 C37（事务与记录的对应无定义）、C80（记账更新必须原子地随根发布）、C196（中央映射树的根住哪全仓无条款）、C199（实例代号递增与 jsn 断号即止互相矛盾）、C274（重放收养的对象不推计数器水位）、C310（事务切分纪律与记录数口径打架）、C331（择根倒挂压过已确认的写）（还欠多次挂载崩溃点重放里的注入）、C342（树 ID 水位在根读不出时退回去重发）、C365（恢复路径的链首不锚在所选根覆盖的最后一条）、C411（分配记录的跨度段与单元类别不符没检查）、C469（I-8.7 的两处射程今天判不了）、C548（记账树每次发布整批换代与每统计量每代一条不符）、C560（满环时根槽写坏的恢复前镜像上记账多于遍历）、C569（前一槽躺着残留记录时反向链两边判法不同）；次要归项二的 C496（环里一条记录都读不出的那道拒绝的前提不承重）、C555（两盘最新系统配置槽都坏时取号撞号）、C563（读根环一槽读坏就当没有根，别的调用方没判）；不是里程碑二留下但归项二的 C72（extent 和单元什么关系没人答）、C83（提交固定点没人回答）、C85（发布计数器的四个等号没写死）、C88（根环的时间线判别未实现）、C163（固定点各轮写序相同使平局破除落空）、C211（节点落盘形态没有正文）、C296（journal 重放时凑不齐映射 key）、C388（checker 的判定宽度不自己探测）、C389（Merkle 根链差分零代码）；收口表第 28 行的 inode 叶容器末尾分裂（并行线三跨记录层 0 流） | 各自的登记行 | 每笔按登记行的「怎么拦」；关不了的写去向 | 二·1 |
| 二·6 | C567（分配与发布的代价随盘容量线性）：先量（本机能造的最大稀疏盘上一次分配与一次发布的耗时与内存，里程碑二只量一次），改法走三方；用户 2026-09-28 定先量、改动排段 4（删除与不变量落地之后），「core 不优化算法、checker 先优化」在这个里程碑只让位这一笔 | `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」那张表第一行 | 量出的数进实验页；改法的代价数交用户 | 段 4 |

- **开工前要先定的**：二·4 与二·6 的三问用户 2026-09-28 已定（写在各自那一行）；剩 C596（撕裂的系统配置轮换之后只剩只读挂载）、C597（实例表行只增不删，删行的条件没有条款） 做不做。

```

**出处 `.claude/kb/milestone/03-third-txn.md:75-91`（整段抄，未转述）**

```markdown
## 三　删除（新事务）

- **原话**：「加删除」（用户 2026-09-27，里程碑三的新事务）
- **仓里已有的**：概念层已定：D8（核心索引结构） 已定项 5（可能修改无界数量 extent 的操作先把意图写进 logged_ops 树再分批推进、崩溃后续做；删除中的 inode 单独进 `deleted_inodes` 树、由后台回收；意图只声明目标态、续做是每次恢复都跑的常规动作、每个中间态都是合法状态、删除意图与最后一批工作同一事务提交）与已定项 6（回收完成前 inode 记录留在 inode 树里、nlink = 0，已发布的 (inode 号, 对象出生代) 不复用）。格式层没落点：C118（`deleted_inodes` 树的形态无落点）（key、value、树 ID、新池新建文件建不建空树都没写），意图记录住哪棵树全仓没写。代码里零删除：`crates/singlefs-core/src/inode_tree.rs` 第 16 行与 `crates/singlefs-checker/src/walk.rs` 第 6133 行都写明「这一版没有删除」；`crates/singlefs-harness/src/model.rs` 的操作枚举只有第一个文件版本、覆盖写、写行、空发布、零单元发布。未实现的不变量：I-4.4（意图只描述目标态）、I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-9.5（删除队列里的号仍在树里）、I-3.4（可用空间扣待删占用）。意图机制还有两个消费者：D5（快照 / 空间记账机制） 已定项 6 的稀疏旁表、D26（后台整理与放置回收） 的整理。安全设计轮第三轮「崩在删除类发布中间」（近满之后截断成 0 那一次发布，一条流 16 777 223 个状态）没跑（`research/prompts/m2-safety-r3-main-verification.md` 第四节）。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 三·1 | 设计轮（三方，最多三轮）：`deleted_inodes` 的 key / value / 树 ID、mkfs 建不建空树；意图记录住哪（单独一棵 logged_ops 树，还是记账或树表里一条条目）、单文件 unlink 的意图目标态怎么写（「inode N：无 extent、记录删除」这类，推的）；分批的粒度与续做入口（每次可写挂载先续做），以及**分批必须能被测试强制进入**——第一版单文件的 extent 数有界、按字面可能一批做完，要一个只供测试的批大小旋钮，固定脚本的删除段至少两批（`.claude/rules/fs-design.md` 五条硬要求第 2 条）；意图未完成期间可用空间怎么扣（I-3.4（可用空间扣待删占用））；崩在回收中间的崩溃点怎么编码进放量的节点（`unlink.<段>.<步>`，推的）；新树落 D15（格式冻结政策） 已定项 7 的哪一层、态别（登记进 [freeze-layer-membership.md](../freeze-layer-membership.md)）；与稀疏旁表、整理共用意图机制的接口 | D8（核心索引结构） 已定项 5 / 6；D5（快照 / 空间记账机制） 已定项 6；D21（权威态与派生态的分界）；D15（格式冻结政策）；C118（`deleted_inodes` 树的形态无落点） | 判决 `research/prompts/m3-delete-r<轮>-main-verification.md`，岔路带代价数交用户；定案写回 D8（核心索引结构） 新分项、C118（`deleted_inodes` 树的形态无落点） 还清；`layout/03-third-txn.md` 建档 | 无 |
| 三·2 | 实现 core：`unlink` 单文件（无目录，按 inode 号）——意图写入 → 分批释放 extent（走 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的释放判定）→ inode 记录 nlink = 0 留树、`deleted_inodes` 加条目 → 最后一批与删意图同一事务；每次可写挂载先续做；`crates/singlefs-harness/src/model.rs` 加删除操作与删后读到「没有文件」的答案 | D8（核心索引结构） 已定项 5 的四条硬要求 | harness 用例：删后读回「没有文件」、崩在每两批之间的中间态 checker 全绿、同一批续做两次盘上字节相同（幂等）、删除意图不与最后一批同事务时用例红；变异行；代码三方一轮 | 三·1 |
| 三·3 | checker：I-4.4（意图只描述目标态）、I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-9.5（删除队列里的号仍在树里）、I-3.4（可用空间扣待删占用） 实现，坏镜像各一 | D8（核心索引结构） 已定项 5 / 6；三·1 的定案 | 每条只红自己；`crates/mutations.tsv` 各一行 | 三·1 |
| 三·4 | 覆盖：固定脚本到 E 之后加 unlink 段（或新流 mkfs → A → unlink → 再建第一个文件版本，推的，三·1 定），跑在崩溃放量上；安全轮第四节没跑的那一格（截断成 0）也一起跑 | D13（验证路线） 已定项 4 | 两条流全域 0 红；改坏「最后一批与删意图同事务」那一处，那条流的判器红 | 三·2、三·3、一·1 的公共前缀 |
| 三·5 | 连带欠账：C116（叶容器的分裂 / 合并纪律没有会失败的检查）（删除引入合并）、C375（待删占用与已承诺预留按份还是按和记没有条款）、C479（inode 号复用已发过的号今天造不出来）（删除之后号不重发才造得出）、C536（inode 树的「左从右借」让容器号条件为假）；意图与墓碑类：C1（意图续做做成常规路径）、C2（无界操作做成单事务）、C84（墓碑单元的粒度没人定）、C109（哪些树的 key 死亡要写墓碑）；C103（容器索引无落点）、C105（打包容器按发布频率重写的写放大）、C106（开放容器数上界） 归 D27（小数据打包容器），不做 | 各自的登记行 | 每笔按登记行的「怎么拦」 | 三·2 |
| 三·6 | 字节：删除写出的新盘上形态（意图记录、`deleted_inodes` 叶、nlink = 0 的 inode 记录、回收那几次发布的段序列）登记进 `layout/03-third-txn.md`，照 E142（新池新建文件的干跑） 的形态跑一次干跑核实装与登记逐字段相等；门禁 doc-registries 加一格照 [02-second-txn.md](02-second-txn.md) 那道挂钩 | D15（格式冻结政策）；[layout/01-first-txn.md](../layout/01-first-txn.md) 的宽度与落点 | 干跑产物与登记表全等；挂钩格判红的样本 | 三·1 |

- **开工前要先定的**：三·1 那一轮要答的全部问题。第一版只做 unlink、不带 truncate（用户 2026-09-28 定；截断到 N 字节留给下一个里程碑，同一套意图机制）。

```

**出处 `.claude/kb/milestone/03-third-txn.md:92-103`（整段抄，未转述）**

```markdown
## 四　并发读写稳定（暂定，优先级最低）

- **原话**：「并发读写稳定」；2026-09-27 补一句：「现在没有实现真正的并发 在读写端预期的那种 可以释放ssd限制的多个事务并行提交场景 我还是比较期待的。」
- **仓里已有的**：D13（验证路线） 已定项 2「先串行，越过交点再开并行；将来并行提交时记录按事务号顺序追加，事务号 N 的 fsync 返回前要等 < N 的事务全部已追加或已作废或已确定实例切换」；串行的天花板一次 checkpoint 约 1.1 万条条目（算式在 C58（串行提交撞天花板没人报警））；里程碑二把并行提交列在不做的那一串里。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 四·1 | 只做前置：C58（串行提交撞天花板没人报警） 的交点报警；并行提交要不要为它加 litmus（门禁 checker-tier-lkmm）；D13（验证路线） 已定项 2 那条前置约束在并行下的实现形态与崩溃语义（前缀可能同时含完整的事务 B 与残缺的事务 A）写成问题单 | D13（验证路线） 已定项 2；C58（串行提交撞天花板没人报警）；归项四的 C87（准入读的合成值无定义）、C214（遍历跳的窗口余量没人守）、C319（请求内单元按 key 升序发出没有条款也没有检查）、C383（同一次发布里同 key 的中间态折叠没有检查）、C488（组提交摊薄了写字节，没摊薄的那一半没量过） | 报警会红；问题单写成 | 出口 1–3 之后才开 |

- **开工前要先定的**：「并发」是不是真的并行提交（用户 2026-09-27 那句是）；做不到转给里程碑四（用户 2026-09-27 定）。

```

**出处 `.claude/kb/milestone/03-third-txn.md:104-136`（整段抄，未转述）**

```markdown
## 五　里程碑二未收口的，与欠账表里相关的

- **原话**：「里程碑2中未收口需要测试和实现的」；「检查还清与本里程碑3相关的内容 .claude/kb/checks-owed.md」（原话里的本机绝对路径写成仓内相对路径）
- **仓里已有的**：清单 2026-09-28 列完（`records/2026-09-28-里程碑三第十项清单.md`）：欠账表开着的 479 行逐行终判，里程碑二留下的 144 行里 45 行「第十项自留」（就是这一项自留）、38 行疑似已还（核实后核销 30 行、留 22 行部分已还）、22 行挪到里程碑三之外、其余归项一至项四（各项作业点表里已点名；清单里归崩溃放量装置的随「增补 4」走，不在这份文件里）；不是里程碑二留下但与里程碑三相关的 50 行里归项的已写进各项，单列的 C30（链式保留池量化未验）、C70（跨统计量的和没人盯）、C122（分配记录不点删之后的条目数口径）、C191（同源性计数器只盖了一对）、C284（施加一条记录在指针层上做什么没有定义）、C397（反向判据的 N 没声明，事件数没人计） 在这一项判。用户 2026-09-28 定照终判写进各项。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收 | 依赖 |
|---|---|---|---|---|
| 五·1 | 「第十项自留」45 行，开工日逐条判「这个里程碑关 / 留 / 挪出」：C146（无空段时的回落政策全仓无定义）、C355（checkpoint 保留池池级扣而固定点每块要落两列）、C356（第九项按统计量之差算，净释放时为负且没有钳位）、C357（同一根轴上的两条重开判据没人合过）（要的是 E160（随机小读占比：三个来源与重开判据的合成） 第二段）、C368（分配器落点只看盘 0，盘不等大时断言失败）、C370（需求、可用与 df 没有共同单位）、C371（活元数据没有定义）、C372（全空聚簇段数两份定义相反）、C379（记账的全空聚簇段数与分配器能开的段两个口径）、C455（挂钟最贵的七道阶段没有判别力样本）、C461（checker 判「挂不上的镜像」整片报不适用）、C462（空池恢复报的错名不副实）、C463（一块盘的系统配置两份全废之后静默成功）、C466（几块盘的系统配置设备数对不上时 checker 判不出）、C489（singlefs-core 那 1786 条广谱变异没跑）（全跑 5.7–11.5 天）、C493（回退候选集条文与实现说反话）、C538（分配记录树释放链让保留池兜不住）、C542（取号前预演不读盘核，坏盘加回退到最老根会烧号）、C543（重建上一版只带实例表第 0 片）、C546（抬 F 被拒时扣住的槽不退回）、C549（分配记录树固定点 64 轮的上限没有依据）、C550（对拍模型里分配记录树节点数的上界偏松）、C553（盘在两次挂载之间改大小没有条款）、C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）、C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷）、C570（抬 F 回收扣住的槽计数上空闲但分配器不发）、C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判）、C572（回退目标引用隔离单元不进逐盘验）、C573（固定点上界没算有普通分配那次发布多出的删插）、C574（落得下那一判不核两槽对齐与各盘同槽）、C575（一个错误成员装两种判据的载荷）、C576（三条用例被准入先拒连带红，改法待选）、C578（乙的读缓存代价没量）、C581（extent 偏移非 0 没有判据）、C582（挂载盘容量下界没条款该取哪个）；已在项二、项三、项六表里点名的自留行（C476（盘上内容可控时走得到的 panic 有 21 处）、C496（环里一条记录都读不出的那道拒绝的前提不承重）、C541（原样重发把一次失败放大成整条发布流阻塞）、C555（两盘最新系统配置槽都坏时取号撞号）、C558（回退目标是环里最旧的根时回退那次发布会写坏它）、C563（读根环一槽读坏就当没有根，别的调用方没判）、C566（环上有洞时多扣的槽随洞数无界增长）、C567（分配与发布的代价随盘容量线性）、C583（双故障形：每盘一槽系统配置坏加最新根一时读不出））在那几项判，这里不重复 | 各自的登记行 | 每行一个去向，写进这张表的备注列或 [checks-owed.md](../checks-owed.md) 那一行 | 里程碑二收口那天按收口表与欠账表再核一遍增量 |
| 五·2 | 部分已还、还欠一截的 22 行（`records/2026-09-28-里程碑三第十项清单.md`「疑似已还的 52 行」表里处置为「留着」的：C66（格式级空白没有分项号） 到 C564（另外 6 条层 0 全量没人跑）），按核实表「还欠」一列改写登记行，欠的那一截各归一项或关；C564（另外 6 条层 0 全量没人跑） 那一截是崩溃放量的活，随那一部分走 | 各自的登记行 | 22 行登记行只写还欠的那一截 | 无 |
| 五·3 | 多次发布的字节清单（收口表第 25 行，用户 2026-09-23 定判不收口、挪后）：照 E142（新池新建文件的干跑） 的形态建一份新实验，核固定脚本每次发布写出的字节与 [layout/02-second-txn.md](../layout/02-second-txn.md) 逐字段相等，段序列登记表补发布 B 自己那一行 | [layout/02-second-txn.md](../layout/02-second-txn.md) | 干跑产物与登记表全等 | 无；三·6 用同一形态 |
| 五·4 | 单列的六笔各判去向：C30（链式保留池量化未验）、C70（跨统计量的和没人盯）、C122（分配记录不点删之后的条目数口径）、C191（同源性计数器只盖了一对）、C284（施加一条记录在指针层上做什么没有定义）、C397（反向判据的 N 没声明，事件数没人计） | 各自的登记行 | 每笔一个去向 | 无 |
| 五·5 | 挪到里程碑三之外的 22 行（第三方管道、冗余、扫描重建、三块盘以上、统计量标签、后台整理、修复与 scrub、性能、要挂载内实例切换的那一族）：只在 [checks-owed.md](../checks-owed.md) 登记行的前置列写明归哪个主题，这份文件不再点名（清单在 `records/2026-09-28-里程碑三第十项清单.md`「甲一」表） | — | 22 行各写了归属 | 无 |
| 五·6 | 大档故障注入 2 条新发现（用户 2026-09-30 加进这个里程碑）：设备吞掉起点段一次系统配置槽写之后，模型说可写挂载该成，实现以 `MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)` 拒（种子 7463871032432355222，release 单跑 12.9 秒复现；那个拒法是提交 dbcc0d2f 进的）。先定条款「设备吞写之后这种拒算不算允许的结局」，再二选一：模型在设备吞写之后认这种拒，或改挂载的判法 | C600（吞写之后可写挂载被拒而模型说该成） | 种子 7463871032432355222 那段钉成一条 harness 用例，先证它按条款会红；改完照 C600（吞写之后可写挂载被拒而模型说该成） 登记行那组环境变量重跑大档，0 条新发现 | 条款先定 |

- **五·1 逐行去向**（主 agent 2026-09-30 判；「三方」指这一簇先走一轮三方定条款，判决之后再并进实现批；调度在 `records/2026-09-30-里程碑三开工第一轮.md`）：

| 去向 | 行 | 依据 |
|---|---|---|
| 已还 | C379（记账的全空聚簇段数与分配器能开的段两个口径） | 已在 [checks-owed.md](../checks-owed.md)「已还清」表 |
| 关，准入条款簇（新开一轮三方 `m3-admission-r1`，判决后并进实现批） | C356（第九项按统计量之差算，净释放时为负且没有钳位）、C370（需求、可用与 df 没有共同单位）、C371（活元数据没有定义）、C546（抬 F 被拒时扣住的槽不退回）（剩两格：扣住位怎么放）、C570（抬 F 回收扣住的槽计数上空闲但分配器不发）、C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判）、C573（固定点上界没算有普通分配那次发布多出的删插）、C574（落得下那一判不核两槽对齐与各盘同槽）、C576（三条用例被准入先拒连带红，改法待选）、C582（挂载盘容量下界没条款该取哪个） | 各行的前置列都是「条款没定 / 改法待选 / 要先定单位」，同落在 D28（挂载期承诺量） 与 D3（空间分配） 的准入式子上；C370 那一格判决之后照原行交用户定单位 |
| 关，实现（条款已定，按 `crates/` 文件聚簇，排在二·2 那一批合入之后，因为都动 `mount.rs` 或准入） | C355（checkpoint 保留池池级扣而固定点每块要落两列）（丁已定：准入式子逐设备合取 + E141（切换预留的挂载准入自证） 加第八项）、C368（分配器落点只看盘 0，盘不等大时断言失败）（剩：三种落点拒绝分开报、随机历史加不等盘取样点）、C493（回退候选集条文与实现说反话）（剩一条变异）、C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）（剩取号改写见证值与模型跟上乙两半）、C572（回退目标引用隔离单元不进逐盘验）（C554 乙落地后）、C575（一个错误成员装两种判据的载荷） | 前置列写着条款已定或「无」 |
| 关，并进二·1 的不变量轮 | C461（checker 判「挂不上的镜像」整片报不适用）（剩：哪块盘系统配置全废用哪条编号报）、C466（几块盘的系统配置设备数对不上时 checker 判不出）、C581（extent 偏移非 0 没有判据） | 三行缺的都是一条判定或一条不变量的措辞 |
| 关，并进二·4 那一轮 | C462（空池恢复报的错名不副实）、C463（一块盘的系统配置两份全废之后静默成功） | 与 C596（撕裂的系统配置轮换之后只剩只读挂载） 同为系统配置冗余的条款空白（D22（单元原子性怎么合成） 已定项 8） |
| 关，归项六 | C455（挂钟最贵的七道阶段没有判别力样本）（六·5）、C549（分配记录树固定点 64 轮的上限没有依据）（六·4 旋钮登记表）、C550（对拍模型里分配记录树节点数的上界偏松）（六·3，与 C445（模型对拍的取样点不随写路径增长） 同簇） | 都是验证系统自己的判别力或旋钮依据 |
| 关，段 4 与二·6 同批量 | C578（乙的读缓存代价没量） | 与 C567（分配与发布的代价随盘容量线性） 同一次量挂载读阶段的耗时与内存 |
| 关，先造格再交用户 | C542（取号前预演不读盘核，坏盘加回退到最老根会烧号） | 挂载时回退那一支已没了；普通可写挂载那一支先由实现员造（进实现批），造不出按原行挪已还清，造得出交用户定改不改 |
| 交用户 | C489（singlefs-core 那 1786 条广谱变异没跑） | 重型、串行约 11.5 天；用户 2026-09-30 问 GPU 能不能参与，答复之后再定 |
| 关，并进准入条款簇那一轮（几何一格） | C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷） | 用户 2026-09-30 定「不接受 但是要推一下」：方向是不接受，「io_min 大到多少 mkfs 拒」怎么写进 D2（RAID 条带策略） 已定项 19 射程要先推一轮，与 C582 同属几何下界 |
| 挪出里程碑三 | C146（无空段时的回落政策全仓无定义）（② 已还，剩 ① 归后台整理）、C357（同一根轴上的两条重开判据没人合过）（等 D25（目标负载优先级） 的权重）、C372（全空聚簇段数两份定义相反）（记账那一侧 C379 已定，剩与 D26（后台整理与放置回收） 停机谓词的冲突，归后台整理）、C538（分配记录树释放链让保留池兜不住）（用户 2026-09-24 定挪后）、C543（重建上一版只带实例表第 0 片）（用户 2026-09-25 定不改，没有路径靠它）、C553（盘在两次挂载之间改大小没有条款）（原行写挪后） | 前置在里程碑三出口之外，或用户已定挪后 |

- **开工前要先定的**：五·1 里哪些用户要现在拍板（每一行都要一个去向，主 agent 先判、拿不准的弹窗）；五·6 设备吞写之后可写挂载该成还是该拒（设计判断，走三方或交用户）。

```

**出处 `.claude/kb/milestone/03-third-txn.md:137-153`（整段抄，未转述）**

```markdown
## 六　崩溃点审查和补充；崩溃点测试系统调优和验证

- **原话**：「再加一条 崩溃点审查和补充 崩溃点测试系统调优和验证」（用户 2026-09-28）；同日早些时候的一句是它的题面：「量过会漏 说明量的不好，说明我们现在的崩溃重放 还是设计的有问题。」
- **与放量装置的分界**：装置本身（身份、KV、双机、剪枝、批量派活）归 [02-second-txn.md](02-second-txn.md)「增补 4」与做那一部分的会话；这一项管**点**（写路径的哪些地方设了崩溃点、哪些该设没设、ignore 开着的哪些该开）与**系统的判别力和旋钮**（它抓不抓得到已知的坏法、每个参数取什么值凭什么、跑一次多贵）。做这一项时冒出的装置改动记给那边，不在这里做。
- **仓里已有的**：崩溃点由记录器按屏障段切（`crates/singlefs-checker-tier/src/crash_amplification.rs` 的 `CrashPointRecorder`，节点 `<入口>.<写的种类>.<第几段>`），共用步骤 6 个函数在 `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs`（今天一处 ignore 都没写；ignore 的处理在 `crash_amplification.rs` 里 14 处，展开上限 `expand_up_to`）；身份与复用键在 `crash_identity.rs`。枚举域定义是 D13（验证路线） 已定项 4（前若干段全持久、当前段任意整写子集、按设备切段、撕裂粒度不进模型）；三层分档是 D13（验证路线） 已定项 9，今天只有层 0（C409（层 1、层 2 在门禁里没有阶段））。登记的流 8 条（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行：新池新建文件、固定脚本到 E、会话推的抬 F、C561（记录核对器的复用豁免在复用只落一半时假红） 的 sigma、多记录发布、树分裂、位置寻址树、崩溃注入快档），放量今天跑的两条（固定脚本到 D 全域、到 E 再卸载）。系统自己的判别力：checker 档里带会红形态的用例文件（`crash_enumeration_new_pool_file_creation_stream.rs`、`bad_disk_input_campaign.rs`、`crash_points_tree_identifier_watermark_orphans.rs`，另有两份是装置那边的对拍用例），`crates/mutations.tsv` 1452 行里 416 行点名 checker 档，[verification-build.md](../verification-build.md)「harness 自己要证明会红的用例」表九条。已知的漏：层 0 规模第二轮攻方量过按流列的输入清单漏共用文件（[02-second-txn.md](02-second-txn.md)「增补 4」第一项）；C467（新补的四条不变量的覆盖率没被穷举用例钉住） 量到 I-8.7（实例内事务号不重号） 在新池新建文件那条流上 0 次评估、I-1.8（归并后版本全序） ② 在单一 checkpoint 上一条判定都不产出；C331（择根倒挂压过已确认的写） 还欠多次挂载崩溃点重放里的注入；删除类发布崩在中间没跑（三·4 接）。挂钟：每状态 15–20 毫秒、两台 4 千态/秒（`briefs/2026-09-28.md`）。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 六·1 | **审查**：列崩溃点覆盖表——写路径的每个提交步骤（D17（实现分层与第三方管道） 已定项 2 登记的那个封闭枚举的每个成员）与每个入口（mkfs、取号写行暖机、首个文件版本、覆盖写、空发布、多记录发布、树分裂、回退、抬 F、卸载、崩溃恢复重放、系统配置轮换、被抛弃根的账现算）对着 8 条登记流与放量的流：哪个写序列在哪条流里被枚举、哪个一条流都没有、ignore 开着的节点各是哪些。表由脚本从录制流与 `crash-case:` 行算出，不手列（手列清单正是量过会漏的那一处），落 [verification-build.md](../verification-build.md) 一节 | D13（验证路线） 已定项 4 / 9；D17（实现分层与第三方管道） 已定项 2；C467（新补的四条不变量的覆盖率没被穷举用例钉住） | 门禁一格比对：有提交步骤成员或入口没被任何流罩到就红；判别力：加一个枚举成员不加流，那一格红 | 无 |
| 六·2 | **补充**：按六·1 的表补流——多次挂载里的崩溃注入（C331（择根倒挂压过已确认的写） 欠的那一半）、系统配置轮换坏一槽之后再崩、删除类（三·4）、真设备上的崩溃注入（QEMU 里断电，负载归这里、装置归「增补 4」第一项，补「最终判据」）；ignore 开着的节点按用户「慢慢开」逐个开，每开一个记状态数 | D13（验证路线） 已定项 4；C331（择根倒挂压过已确认的写）；C564（另外 6 条层 0 全量没人跑） 的对象 | 新流全域 0 红；ignore 清单条数只减不增（清单进产物，门禁比上一次） | 六·1 |
| 六·3 | **验证（判别力）**：每条流至少一个必红世界（改坏实现或改坏判器，那条流的判器必须红），进 `crates/mutations.tsv` 或 checker 档用例；系统级：每条流报每条不变量被评估到的次数，0 次的显式列出（C467（新补的四条不变量的覆盖率没被穷举用例钉住） 那一形），门禁判「0 次的不变量数不许涨」（C13（checker 判定失效）：跑了全绿但不做事）；journal 承重（抹掉记录段恢复结果必须变，C353（journal 在恢复路径上的承重没有会红的检查））；模型对拍的取样点随写路径增长（C445（模型对拍的取样点不随写路径增长）） | C13（checker 判定失效）、C353（journal 在恢复路径上的承重没有会红的检查）、C467（新补的四条不变量的覆盖率没被穷举用例钉住）、C445（模型对拍的取样点不随写路径增长） | 每条流的必红世界跑红；评估次数表进产物、门禁格红过一次 | 六·1 |
| 六·4 | **调优**：旋钮登记表（展开上限、线程数、内存上限、批大小、双机分工、ignore、快档取样点）——每个旋钮写取值与依据，凭感觉取的标「推的」；每个旋钮至少一次扫值（挂钟或状态数对旋钮的曲线，计数实验形态）；什么时候跑快档、什么时候全域、快档的取样点够不够，写成判据；层 1、层 2 要不要在这个里程碑立阶段（C409（层 1、层 2 在门禁里没有阶段））开工前定。每状态耗时的构成（择根 / 走树 / 记录核对器 / oracle）的剖析归「增补 4」第二项，这里只用它的数 | D13（验证路线） 已定项 9；C8（门禁范围判不出来）、C409（层 1、层 2 在门禁里没有阶段） | 旋钮登记表带出处；每个旋钮一份扫值产物；「推的」条数只减 | 六·2 |
| 六·5 | 归这一项的其余欠账：C20（罕见布局无人跑）、C447（独立规约执行器与 RefFS 都零代码）；次要归这一项的 C476（盘上内容可控时走得到的 panic 有 21 处）、C541（原样重发把一次失败放大成整条发布流阻塞）、C558（回退目标是环里最旧的根时回退那次发布会写坏它） | 各自的登记行 | 各自登记行写的会红检查；开工日逐条判「这个里程碑关 / 留 / 不做」 | 无 |

- **开工前要先定的**：覆盖表的粒度（按提交步骤成员、按入口、还是按屏障段——推的：三列都要，行是入口 × 步骤）；真设备崩溃注入进不进这个里程碑；层 1、层 2 立不立阶段；C8（门禁范围判不出来） 在放量按块复用之后是不是已由复用键还清（推的，没核）。

```

**出处 `.claude/kb/milestone/03-third-txn.md:154-170`（整段抄，未转述）**

```markdown
## 七　形式化验证：crash refinement 与独立规约执行器（优先）

- **原话**：「我们代码的验证 还没有走 形式化验证是吧」「需要进里程碑3 而且要优先做」（用户 2026-09-29）。
- **仓里已有的**：文件系统代码本身没走过形式化验证。有的只有内存序那一截：`litmus/` 8 份（4 对 Never 与 nofence 对照），门禁 checker-tier-lkmm 起 herd7（`~/.opam/default/bin/herd7`）判——验的是几段同步模式翻成的形式模型，不是 `crates/` 的代码，「写下的模型与代码今天做的是不是一回事」没有东西保证。决策已定、零代码：D13（验证路线） 已定项 1 定 crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」，三个里程碑 M1 全性（`Recover` 在已定项 4 枚举的每个崩溃状态上有定义、终止、输出过 I-7.1（根槽有效集非空） 与 I-8.3（重放前缀严格连续） 的镜像）、M2 前缀不切发布（任一 `jsn` 连续前缀的重放结果等于某个整数个已发布 checkpoint 的结果，停在一次发布中间那次整体不施加）、M3 幂等与单调（`Recover(Recover(c)) = Recover(c)`，段内两个崩溃状态 `c ⊆ d` 推 `Recover(c)` 施加的事务集 ⊆ `Recover(d)` 的——后半全仓无落点）；射程写明 refinement 的边际价值只在层 0 枚举取不到的几维（载荷字节、记录条数 / 段数 / txg 数含环回绕、上一条时间线的残留、槽数与设备数）。已定项 13：refinement 的 spec 与模型对拍的 model 是同一个东西的两种写法，同源性要观测；已定项 14：哈希与 AEAD 在形式验证里只能当 uninterpreted function，日志结构三层绑架符号执行；已定项 6：O3（独立规约执行器）要用另一种语言、只照 decisions 与 invariants 的文字重新推导、配阳性对照。已定项 13 自己写着「crash refinement 今天没有代码」，`grep -rln refinement crates research --include=*.rs --include=*.py --include=*.sh` 2026-09-29 现查 0 命中；`Cargo.toml` 里 kani / verus / prusti / creusot / loom 一个都没有。开着的账：C5（Z3 超时当通过）、C191（同源性计数器只盖了一对）、C407（三 oracle 合谋测试没有代码）、C447（独立规约执行器与 RefFS 都零代码）。调研：[prior-art.md](../prior-art.md) 7.4 的 Yggdrasil（crash refinement 的出处，Python + Z3；别家怎么做是线索不是证据）。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 七·1 | **选路与题面**（三方一轮，岔路单 `research/prompts/m3-formal-r1-forks.md`，判决 `m3-formal-r1-main-verification.md`）：候选至少三条，各给代价数（要另写多少行、能证到 M1–M3 的哪一条、一次跑多久、哈希与 AEAD 怎么当 uninterpreted function、spec 从哪来）——① Yggdrasil 形态：Python + Z3，实现另写一份可符号执行的副本，证 `Recover` 的关系性质；② 在 `crates/` 的 Rust 代码上直接做有界模型检查（Kani 这一类），不重写实现、bounded；③ 协议模型（TLA+ / Apalache 这一类）证 M2 / M3，与代码靠 trace 对拍绑定。每条候选的代价是推的、要量了再交（`.claude/rules/three-way-inference.md`「岔路交用户定之前，每条路要有代价数」）。同一轮定 spec 的来源（只照 [invariants.md](../invariants.md) 与决策文字手写，不从 checker 源码派生——D13（验证路线） 已定项 12、13 的两个冲突）与 O3（独立规约执行器） 进不进这个里程碑 | D13（验证路线） 已定项 1 / 6 / 12 / 13 / 14；C5（Z3 超时当通过） | 岔路单每行带代价数交用户；选定后 D13（验证路线） 已定项 1 三个里程碑的「落点」列填上工具与文件 | 无；用户定优先，第一个派 |
| 七·2 | **M1 全性**：在选定工具上证 `Recover` 对 D13（验证路线） 已定项 4 枚举域里每个崩溃状态有定义、终止、输出过 I-7.1（根槽有效集非空） 与 I-8.3（重放前缀严格连续） 的镜像 | D13（验证路线） 已定项 1 M1；D13（验证路线） 已定项 4 | 证明产物入库、复跑相同；必出反例的世界：把 `Recover` 改成跳过 I-8.3（重放前缀严格连续） 那一判，反例出现 | 七·1 |
| 七·3 | **M2 前缀不切发布**：任一 `jsn` 连续前缀的重放结果等于某个整数个已发布 checkpoint；停在一次发布中间那次整体不施加（D16（发布语义） 已定项 4 的顺序、D23（journal 的角色与格式） 已定项 14 的末条标志） | D13（验证路线） 已定项 1 M2；D16（发布语义）；D23（journal 的角色与格式） | 证明产物入库；必出反例的世界：把「末条标志位」判定改成任意一条都算末条，反例出现 | 七·2 |
| 七·4 | **M3 幂等与单调**：`Recover(Recover(c)) = Recover(c)` 就是 I-8.4（重放幂等）——与二·1 里它的 checker 判定是同一条性质的两种形式，两边对拍；单调那一半全仓无落点，先写条款再证 | D13（验证路线） 已定项 1 M3；I-8.4（重放幂等） | 幂等：证明产物 + checker 判定在同一批崩溃状态上不矛盾；单调：条款写进决策正文，证不出就写明卡在哪 | 七·3、二·1 |
| 七·5 | **门禁一道**（checker-tier 类，命令带 `SINGLEFS_HEAVY_TESTS`）：三种结果（证明 / 反例 / timeout），timeout 判红（C5（Z3 超时当通过） 还清）；输入登记进 `.claude/gate.d/stage-inputs.tsv` 按输入复用；spec 与 model 不一致事件的计数（C191（同源性计数器只盖了一对））；格结构照 `.claude/rules/verification.md`「门禁的结构」 | C5（Z3 超时当通过）、C191（同源性计数器只盖了一对） | 判别力样本：一份会超时的 spec 判红、一份带反例的 spec 判红、证明过的判绿；`--list` / `--check` 格结构过 `research/scripts/gate-structure-check.py` | 七·2 |
| 七·6 | **O3（独立规约执行器） 独立规约执行器与 RefFS**（C447（独立规约执行器与 RefFS 都零代码））：另一种语言、只照 decisions 与 invariants 的文字重新推导、配阳性对照；三 oracle 合谋测试（C407（三 oracle 合谋测试没有代码））。做不做、做到哪，按七·1 的代价数由用户定 | D13（验证路线） 已定项 6；C447（独立规约执行器与 RefFS 都零代码）、C407（三 oracle 合谋测试没有代码） | 按登记行的「怎么拦」：阳性对照抓到规约错、合谋测试抓到三 oracle 同错 | 七·1 |

- **开工前要先定的**：工具与形态（七·1 交用户）；O3（独立规约执行器） 进不进这个里程碑；spec 的来源与它和 checker 的同源性怎么观测（已定项 13）；哈希与 AEAD 在选定工具里怎么当 uninterpreted function（已定项 14）。

```

**出处 `.claude/kb/milestone/03-third-txn.md:171-176`（整段抄，未转述）**

```markdown
## 与里程碑二收尾的关系

- 里程碑二到 E 全域在两台机器上跑；一·1、三·4 的新流跑在同一套崩溃放量上，装置那边的改动（前缀共享、KV、剪枝、批量派活）由里程碑二「增补 4」与做那一部分的会话管，这份文件只记流本身。
- 甲组（验证装置重做，含新结构、屏障判红 C6（块层语义假设写错）、C26（屏障数从没量过）、KV 存储）整组在里程碑二「增补 4」收口，这份文件不再管。
- 里程碑二收口那天，项五按收口表与欠账表再核一遍增量（清单是 2026-09-28 的仓）。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1-17`（整段抄，未转述）**

```markdown
## E165 恢复重放有界穷举与变异杀伤界 —— 第一段已跑（2026-10-01），第二段已跑（2026-10-01）

<!-- doc-lint:not-numbers M1 M2 M3 K2 K3 K4 K5 K6 K7 K8 K9 K10 J0 J1 J2 J3 J4 J5 J6 H1 H2 H3 H4 H5 V1 V2 V3 V4 V5 V6 V7 V8 V9 F1 F2 F3 F4 F5 F6 F8 F9 F10 F11 F12 F13 F14 F15 F16 F17 F18 F19 Q6 Q9 Q10 Q12 Q13 Q14 Q15 L6 L9 C6 E1 E2 A6 R0 S3 Z1 Z2 Z3 Z4 IA1 IA2 -->

跑前登记 `research/prompts/e165-preregistration.md`（第十二节修订 1–12）；岔路单 `research/prompts/m3-formal-r1-forks.md` 的 K3–K9。被测对象是 `crates/singlefs-core/src/recovery.rs` 今天的 `replay_journal`（2026-10-01 的 sha256 `f04814901d9521a1f72bf64aae530157c643fe47f3b0a37ac06695feb9418fdb`），一个字没改。装置 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`（入库装置：直接调 `replay_journal`、`JournalRecord::to_bytes` / `parse`、真 CRC、真反向链，读者是内存里的两块盘）。

产物：

- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out`：`--bounds segment1 --family legal`，910 行 `E7RESULT`，49 753 566 态。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out`：`--bounds light --family legal`，876 行，6 923 358 态。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family-timing.out`：两次跑的 `E7TIMING` 耗时行与量的时候的 `uptime` 原样输出，不进逐字比对。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out`：`--bounds segment1 --family adversarial`，172 行，64 513 748 态（登记修订第 9 条收窄停机条款 H4 之后跑的）。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family-timing.out`：对抗一族两次跑（16 线程、7 线程）的耗时行与 `uptime`，不进逐字比对。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out`：第二段，26 个变体（基线 1、`crates/mutations.tsv` 里 21 种不同的替换即 23 行、自造 4 条）各一行或两行，32 行。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv`：第二段的红例账，151 行（每个红的变体在每个红了的域首红的 N 上一个反例状态），格式见它的表头。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds-timing.out`：第二段驱动的计时、负载与单变体量的那一次，不进逐字比对。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:18-25`（整段抄，未转述）**

```markdown
### 这一段做了什么

- **合法历史一族**（登记 5.2 的 L、L 小环 6 与 9、U 验证开与关、C 大环与小环 6、X、E1、E2、R）：生成器照 D23（journal 的角色与格式） 已定项 4、已定项 14 与 D13（验证路线） 已定项 1 引的条款造合法历史，每次发布几条记录、提交标记怎么摆全扫，记录的任意落盘子集全扫；每个状态调一次 `replay_journal`，照条款文字手写的判据 J0–J4、J6 与实现锚点 IA1、IA2 各数各的违例；C 域另在全部 s ⊆ u 的状态对上数 J5a（施加集读法）、J5b（落到的版本读法），五个对集合各报各的，并与「理想恢复」（落到照条款写的最远一版 j\*）并排。
- **对抗一族**（A、A 小环 6、A 链）：第一次跑停在登记停机条款 H4（登记 5.2 的字母表有序号 0，`crates/singlefs-core/src/journal.rs:304`「序号从 1 起（已定项 4）：读到 0 当这条记录损坏」，`parse` 拒收；修订 7）。主 agent 写了修订 9：序号不是 0 的记录照 H4 原文断言解得出，序号 0 的记录断言 `parse` 拒收、没拒就停。**修订 9 是看过对抗一族的草稿读数之后写的**；装置照它改了断言之后正式跑、入库，新产物的 `E7RESULT` 行与草稿那一份逐字节相同。序号 0 的记录进不了表，S 的 (e)「序号 0」在建出来的表上走不到（修订 7）。
- **第二段**（登记 5.5、5.6，修订 10–12）：驱动在每个变体自己的副本里改 `crates/singlefs-core/src/recovery.rs`、冷编译装置、跑 `--bounds segment2`，逐 (域, N) 与基线比；界固定为 5.6 的上沿（修订 10 去掉了放宽规则），每个域的「首红的 N」取第一个与基线不同的 N。红了的域在首红的 N 上找一个反例状态，原实现与变体的二进制各单条重放一遍。
- 装置为第二段加了两个参数（修订 12 允许，判据一条没改）：`--replay-state <域>:<N>:<序号>` 只判枚举次序里那一个状态，`--list-states <域>:<N>` 逐态各打一行；两种都单线程、不跑自检与阳性对照。加了之后第一段合法历史一族整份重跑，`E7RESULT` 行与入库那份逐字节相同。
- 照登记 5.5 的次序：① 常量回比断言与装置自检 13 项；② 阳性对照（合法历史一族 35 个读数、对抗一族 10 个）；③–⑥ 各域。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:26-80`（整段抄，未转述）**

````markdown
### 结果整行抄自产物

全部抄自 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out`，行首是那份文件自己的行号（`grep -n 'name=verdict'` 取的）。各域合计的判决行：

```
173:E7RESULT name=verdict domain=L ring_slots=64 smallest_n=1 largest_n=10 states=24186470 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=6046106 applied_ge2=1509478 domain_is_nonempty=true
246:E7RESULT name=verdict domain=L6 ring_slots=6 smallest_n=1 largest_n=6 states=3654 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=354 j6_premise_fails_judged=1272 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=1362 applied_ge2=230 domain_is_nonempty=true
355:E7RESULT name=verdict domain=L9 ring_slots=9 smallest_n=1 largest_n=9 states=1171430 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=51998 j6_premise_fails_judged=557792 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=372042 applied_ge2=87166 domain_is_nonempty=true
428:E7RESULT name=verdict domain=U_verify_on ring_slots=64 smallest_n=1 largest_n=6 states=12004314 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=750246 applied_ge2=46842 domain_is_nonempty=true
489:E7RESULT name=verdict domain=U_verify_off ring_slots=64 smallest_n=1 largest_n=6 states=12004314 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=6726648 j3_violations=0 j4_violations=6726648 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=7086030 applied_ge2=3410250 domain_is_nonempty=true
604:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 states=149796 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=24987 applied_ge2=4197 domain_is_nonempty=true
709:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 states=9700 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=215 j6_premise_fails_judged=596 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=1967 applied_ge2=229 domain_is_nonempty=true
791:E7RESULT name=verdict domain=X ring_slots=64 smallest_n=2 largest_n=7 states=216484 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=40156 applied_ge2=6628 domain_is_nonempty=true
840:E7RESULT name=verdict domain=E1 ring_slots=64 smallest_n=1 largest_n=4 states=398 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=70 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=70 applied_ge2=0 domain_is_nonempty=true
871:E7RESULT name=verdict domain=E2 ring_slots=64 smallest_n=1 largest_n=3 states=42 j0_violations=0 failed_states=42 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=0 applied_ge2=0 domain_is_nonempty=true
908:E7RESULT name=verdict domain=R ring_slots=64 smallest_n=2 largest_n=5 states=6964 j0_violations=0 failed_states=0 j1_violations=956 j2_violations=956 j3_violations=956 j4_violations=956 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=1808 applied_ge2=492 domain_is_nonempty=true
909:E7RESULT name=verdict scope=run bounds=segment1 family=legal states=49753566 self_checks_passed=13 control_readings=35 controls_not_red=0 void_conditions=0 halt_conditions=0
```

C 域状态对（J5a、J5b；`today` 是今天的实现，`ideal` 是理想恢复；`cut_own` 是切法甲、根槽那次写自成一段，`cut_with` 是切法乙、根槽那次写与记录同段）：

```
605:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=wide pairs=7322544 today_j5a_violations=830685 today_j5b_violations=0 ideal_j5a_violations=830685 ideal_j5b_violations=0
606:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_own_legal pairs=71579 today_j5a_violations=4969 today_j5b_violations=0 ideal_j5a_violations=4969 ideal_j5b_violations=0
607:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_own_same_segment pairs=49644 today_j5a_violations=911 today_j5b_violations=0 ideal_j5a_violations=911 ideal_j5b_violations=0
608:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_own_same_segment_stopped_in_a_record_segment pairs=48369 today_j5a_violations=0 today_j5b_violations=0 ideal_j5a_violations=0 ideal_j5b_violations=0
609:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_own_same_segment_stopped_in_a_root_segment pairs=2733 today_j5a_violations=911 today_j5b_violations=0 ideal_j5a_violations=911 ideal_j5b_violations=0
610:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_with_legal pairs=190025 today_j5a_violations=8146 today_j5b_violations=0 ideal_j5a_violations=8146 ideal_j5b_violations=0
611:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_with_same_segment pairs=144560 today_j5a_violations=911 today_j5b_violations=0 ideal_j5a_violations=911 ideal_j5b_violations=0
612:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_with_same_segment_root_write_differs pairs=48369 today_j5a_violations=911 today_j5b_violations=0 ideal_j5a_violations=911 ideal_j5b_violations=0
613:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_with_same_segment_root_write_agrees pairs=96191 today_j5a_violations=0 today_j5b_violations=0 ideal_j5a_violations=0 ideal_j5b_violations=0
710:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=wide pairs=319689 today_j5a_violations=41511 today_j5b_violations=0 ideal_j5a_violations=41511 ideal_j5b_violations=0
711:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_own_legal pairs=2020 today_j5a_violations=475 today_j5b_violations=0 ideal_j5a_violations=475 ideal_j5b_violations=0
712:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_own_same_segment pairs=829 today_j5a_violations=119 today_j5b_violations=0 ideal_j5a_violations=119 ideal_j5b_violations=0
713:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_own_same_segment_stopped_in_a_record_segment pairs=669 today_j5a_violations=0 today_j5b_violations=0 ideal_j5a_violations=0 ideal_j5b_violations=0
714:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_own_same_segment_stopped_in_a_root_segment pairs=357 today_j5a_violations=119 today_j5b_violations=0 ideal_j5a_violations=119 ideal_j5b_violations=0
715:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_with_legal pairs=4507 today_j5a_violations=701 today_j5b_violations=0 ideal_j5a_violations=701 ideal_j5b_violations=0
716:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_with_same_segment pairs=1929 today_j5a_violations=119 today_j5b_violations=0 ideal_j5a_violations=119 ideal_j5b_violations=0
717:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_with_same_segment_root_write_differs pairs=669 today_j5a_violations=119 today_j5b_violations=0 ideal_j5a_violations=119 ideal_j5b_violations=0
718:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_with_same_segment_root_write_agrees pairs=1260 today_j5a_violations=0 today_j5b_violations=0 ideal_j5a_violations=0 ideal_j5b_violations=0
```

每个不为 0 的格里 N 最小的那一行与它的见证（`shape` 是每次发布几条记录与提交标记的摆法，`on_disk` 逐条记录在不在 0 号盘、点名单元哪一态，`first` / `second` 是第一、二次调用的结局）：

```
217:E7RESULT name=violations domain=L6 ring_slots=6 n=4 criterion=J6_premise_fails violations=10 judged=10 witness=shape=1c0+1c0+1c0+1c0;on_disk=-,good,good,good;residue_at=none;chosen=0;farthest=2;verify=true;first=returned:txg102:watermark1002:applied2;second=returned:txg103:watermark1003:applied1
302:E7RESULT name=violations domain=L9 ring_slots=9 n=5 criterion=J6_premise_fails violations=38 judged=38 witness=shape=1c0+1c0+1c0+1c0+1c0;on_disk=-,good,good,good,good;residue_at=none;chosen=0;farthest=3;verify=true;first=returned:txg103:watermark1003:applied3;second=returned:txg104:watermark1004:applied1
442:E7RESULT name=violations domain=U_verify_off ring_slots=64 n=2 criterion=J2 violations=24 judged=108 witness=shape=1c0+1c0;on_disk=-,d0_unreadable;residue_at=none;chosen=0;farthest=0;verify=false;first=returned:txg101:watermark1001:applied1
444:E7RESULT name=violations domain=U_verify_off ring_slots=64 n=2 criterion=J4 violations=24 judged=108 witness=shape=1c0+1c0;on_disk=-,d0_unreadable;residue_at=none;chosen=0;farthest=0;verify=false;first=returned:txg101:watermark1001:applied1
659:E7RESULT name=violations domain=C6 ring_slots=6 n=3 criterion=J6_premise_fails violations=5 judged=5 witness=shape=1c0+1c0+1c0+1c0;on_disk=good,good,good,good;residue_at=none;chosen=0;farthest=2;verify=true;first=returned:txg102:watermark1002:applied2;second=returned:txg103:watermark1003:applied1
810:E7RESULT name=violations domain=E1 ring_slots=64 n=2 criterion=J6_premise_holds violations=2 judged=12 witness=shape=1c0+1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=1;verify=true;first=returned:txg18446744073709551615:watermark1001:applied1;second=failed:chosen_root_txg_has_no_successor
874:E7RESULT name=violations domain=R ring_slots=64 n=2 criterion=J1 violations=2 judged=4 witness=shape=1c0+1c0;on_disk=-,-;residue_at=1;chosen=0;farthest=0;verify=true;first=returned:txg101:watermark9000001:applied1
500:E7RESULT name=pairs domain=C ring_slots=64 n=1 set=wide pairs=9 today_j5a=1 today_j5b=0 ideal_j5a=1 ideal_j5b=0
```

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:81-103`（整段抄，未转述）**

````markdown
### 对抗一族（第一段；登记修订 9 是看过对抗一族的草稿读数之后写的）

抄自 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out`（`grep -n 'name=verdict'`）：

```
84:E7RESULT name=verdict domain=A ring_slots=64 smallest_n=1 largest_n=4 states=57585320 a0_violations=0 failed_states=12435792 as_whole_table_violations=330240 as_consecutive_gap_continues_violations=154624 as_consecutive_gap_stops_violations=488224 readings_differing_whole_table_vs_consecutive_gap_continues=175616 readings_differing_whole_table_vs_consecutive_gap_stops=509216 readings_differing_consecutive_gap_continues_vs_consecutive_gap_stops=333600 a_structure_violations=0 a_limit_violations=0 j6_premise_holds_violations=85400 j6_premise_fails_violations=0 j6_premise_fails_judged=0 applied_ge1=1473356 states_holding_a_record_rejected_by_parse=45815520 domain_is_nonempty=true
127:E7RESULT name=verdict domain=A6 ring_slots=6 smallest_n=1 largest_n=3 states=788838 a0_violations=0 failed_states=128560 as_whole_table_violations=2048 as_consecutive_gap_continues_violations=1024 as_consecutive_gap_stops_violations=3424 readings_differing_whole_table_vs_consecutive_gap_continues=1024 readings_differing_whole_table_vs_consecutive_gap_stops=3424 readings_differing_consecutive_gap_continues_vs_consecutive_gap_stops=2400 a_structure_violations=0 a_limit_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=496 j6_premise_fails_judged=65536 applied_ge1=25058 states_holding_a_record_rejected_by_parse=548640 domain_is_nonempty=true
170:E7RESULT name=verdict domain=AC ring_slots=64 smallest_n=1 largest_n=3 states=6139590 a0_violations=0 failed_states=1009760 as_whole_table_violations=10240 as_consecutive_gap_continues_violations=4096 as_consecutive_gap_stops_violations=16640 readings_differing_whole_table_vs_consecutive_gap_continues=6144 readings_differing_whole_table_vs_consecutive_gap_stops=18688 readings_differing_consecutive_gap_continues_vs_consecutive_gap_stops=12544 a_structure_violations=0 a_limit_violations=0 j6_premise_holds_violations=2904 j6_premise_fails_violations=0 j6_premise_fails_judged=0 applied_ge1=133024 states_holding_a_record_rejected_by_parse=4295232 domain_is_nonempty=true
171:E7RESULT name=verdict scope=run bounds=segment1 family=adversarial states=64513748 self_checks_passed=13 control_readings=10 controls_not_red=0 void_conditions=0 halt_conditions=0
```

不为 0 的 AS 与 J6 格里 N 最小的在 n = 3（n ≤ 2 全 0）；A 域 n = 3 的见证：

```
59:E7RESULT name=violations domain=A ring_slots=64 n=3 criterion=AS_consecutive_gap_continues violations=1024 judged=778034 witness=anchor=absent;slots=t+1:o1:L0:C0:x1:chain_true,t+0:o1:L1:C0:x1:chain_true,t+1:o1:L1:C1:x1:chain_true;first=returned:txg100:watermark1000:applied0;expected=txg101:watermark9003038/txg101:watermark9003038/txg101:watermark9003038
63:E7RESULT name=violations domain=A ring_slots=64 n=3 criterion=J6_premise_holds violations=464 judged=650650 witness=anchor=absent;slots=t+1:o1:L1:C1:x1:chain_true,t+2:o1:L0:C0:x1:chain_true,t+1:o1:L1:C0:x1:chain_true;first=returned:txg101:watermark9001038:applied1;expected=txg100:watermark1000/txg101:watermark9001038/txg101:watermark9001038;second=failed:more_than_one_last_record_flag
```

- 三种读法两两分得开（三个 A-互 数在三个域都不为 0），AS 三条在同一个域都不为 0。第 59 行的见证：槽 2 与所选根同 txg、带末条标志，S 认它当锚点、施加槽 3；实现的 `above` 按计数器排，第一条是槽 1（txg t + 1），不等于期望的计数器就停，一条不施加——水位之上的记录计数器比锚点小（同实例里 txg 随计数器回退）才走得到。
- A-结构、A-限 在三个域都是 0：带上前提之后，实现返回的根要么是所选根、要么取自一条同实例、txg 更大、带提交标记与末条标志的记录，报告的施加条数不超过在飞上限。
- J6 前提真的违例（A 464 起，见证第 63 行）：第一次落到的那一版，表里同 txg 带末条标志的有两条，第二次调用报 `more_than_one_last_record_flag`。
- 被 `parse` 拒收（序号 0）的记录至少一条的态数照报（`states_holding_a_record_rejected_by_parse`），A 域 57 585 320 态里 45 815 520 态。

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:104-157`（整段抄，未转述）**

````markdown
### 第二段：变异杀伤界

收尾三行抄自 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out`：

```
E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows=863 repository_recovery_sha256_at_start=f04814901d9521a1f72bf64aae530157c643fe47f3b0a37ac06695feb9418fdb repository_recovery_sha256_at_end=f04814901d9521a1f72bf64aae530157c643fe47f3b0a37ac06695feb9418fdb crates_git_status_unchanged=true copies_restored=true
E7RESULT name=verdict scope=segment2 variants=26 variants_excluding_baseline=25 red=23 red_only_by_implementation_anchor=1 not_red_within_bounds=1 invalid=0 run_did_not_finish=0 ledger_sha256=155cdb1a0882432e943d5f18b9d425b7265f7e22884d8741c900151f292b5724
E7RESULT name=done emitted=32
```

基线那一行记 `rows_found_verbatim_in_segment1_products`：基线每个 (域, N) 的行都逐字出现在第一段的两份产物里（作废条款 V7 没触发）。逐变体的结局与两族各自的首红最小值，命令与原样输出（行数按表行数，第 23 与 355 行、第 406 与 414 行是同一个变体、各一行）：

```
$ grep 'name=kill_bound' research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out | grep -v item=baseline | awk '{o="";for(i=1;i<=NF;i++){if($i~/^(item|table_row|outcome|legal_family_minimum|adversarial_family_minimum)=/)o=o" "$i};print substr(o,2)}'
item=tsv-23 table_row=23 outcome=red legal_family_minimum=2 adversarial_family_minimum=none
item=tsv-23 table_row=355 outcome=red legal_family_minimum=2 adversarial_family_minimum=none
item=tsv-24 table_row=24 outcome=red legal_family_minimum=1 adversarial_family_minimum=1
item=tsv-36 table_row=36 outcome=red legal_family_minimum=3 adversarial_family_minimum=1
item=tsv-226 table_row=226 outcome=red_only_by_implementation_anchor legal_family_minimum=2 adversarial_family_minimum=none
item=tsv-354 table_row=354 outcome=red legal_family_minimum=2 adversarial_family_minimum=none
item=tsv-406 table_row=406 outcome=red legal_family_minimum=2 adversarial_family_minimum=1
item=tsv-406 table_row=414 outcome=red legal_family_minimum=2 adversarial_family_minimum=1
item=tsv-448 table_row=448 outcome=not_red_within_bounds legal_family_minimum=none adversarial_family_minimum=none
item=tsv-512 table_row=512 outcome=red legal_family_minimum=3 adversarial_family_minimum=1
item=tsv-513 table_row=513 outcome=red legal_family_minimum=none adversarial_family_minimum=1
item=tsv-514 table_row=514 outcome=red legal_family_minimum=none adversarial_family_minimum=2
item=tsv-515 table_row=515 outcome=red legal_family_minimum=none adversarial_family_minimum=2
item=tsv-516 table_row=516 outcome=red legal_family_minimum=none adversarial_family_minimum=1
item=tsv-517 table_row=517 outcome=red legal_family_minimum=2 adversarial_family_minimum=2
item=tsv-518 table_row=518 outcome=red legal_family_minimum=2 adversarial_family_minimum=1
item=tsv-613 table_row=613 outcome=red legal_family_minimum=none adversarial_family_minimum=1
item=tsv-614 table_row=614 outcome=red legal_family_minimum=none adversarial_family_minimum=2
item=tsv-920 table_row=920 outcome=red legal_family_minimum=3 adversarial_family_minimum=1
item=tsv-921 table_row=921 outcome=red legal_family_minimum=2 adversarial_family_minimum=1
item=tsv-922 table_row=922 outcome=red legal_family_minimum=1 adversarial_family_minimum=1
item=tsv-1003 table_row=1003 outcome=red legal_family_minimum=1 adversarial_family_minimum=none
item=tsv-1004 table_row=1004 outcome=red legal_family_minimum=1 adversarial_family_minimum=1
item=Z1 table_row=3 outcome=red legal_family_minimum=1 adversarial_family_minimum=1
item=Z2 table_row=4 outcome=red legal_family_minimum=3 adversarial_family_minimum=3
item=Z3 table_row=5 outcome=red legal_family_minimum=2 adversarial_family_minimum=none
item=Z4 table_row=6 outcome=red legal_family_minimum=3 adversarial_family_minimum=3
```

界内没被条款判据抓到的两条，整行抄（第 7、11 行）：

```
7:E7RESULT name=kill_bound item=tsv-226 table=crates/mutations.tsv table_row=226 rows_sharing_this_replacement=226 outcome=red_only_by_implementation_anchor first_clause_red=none first_anchor_red=U_verify_off:2:IA2 other_difference_before_any_red=none not_red_domains=L(≤8),L6(≤6),U_verify_on(≤5),C(≤5),C6(≤5),A(≤3),A6(≤3),AC(≤3),X(≤6),E1(≤4),E2(≤3),R(≤5) legal_family_minimum=2 adversarial_family_minimum=none run_exit=0 run_finished=true domains_without_rows=none abort=none red_examples=1 red_examples_confirmed_by_single_state_replay=1 copy_restored=true rerun="bash research/scripts/e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --output research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out --ledger research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv --recheck --item tsv-226" title=只供测试的开关：关掉点名验证时_verification_passed_照加（两臂在健康镜像上报同一个数，运行时看不出走了哪一条） complete=yes
11:E7RESULT name=kill_bound item=tsv-448 table=crates/mutations.tsv table_row=448 rows_sharing_this_replacement=448 outcome=not_red_within_bounds first_clause_red=none first_anchor_red=none other_difference_before_any_red=none not_red_domains=L(≤8),L6(≤6),U_verify_on(≤5),U_verify_off(≤5),C(≤5),C6(≤5),A(≤3),A6(≤3),AC(≤3),X(≤6),E1(≤4),E2(≤3),R(≤5) legal_family_minimum=none adversarial_family_minimum=none run_exit=0 run_finished=true domains_without_rows=none abort=none red_examples=0 red_examples_confirmed_by_single_state_replay=0 copy_restored=true rerun="bash research/scripts/e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --output research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out --ledger research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv --recheck --item tsv-448" title=P6_后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮_K4-b） complete=yes
```

- 「红」照登记 5.5 的口径是「与基线不同」，不分方向。红例账第 7 列带方向（`+` 是变体多加的违例、`-` 是少加的），只是首红那个状态上的，不是整域合计。几处「红」是变体比基线少违例：R 域的 J1–J4（今天的实现在残留态上落到时间线之外，变体不施加或少施加就不落了；tsv-24、512、920、921、922、1004、Z1 的反例状态都是 `-J1`），E1 的 J6 前提真（tsv-24、512、517、921、922、Z1 是 `-J6_premise_holds`），Z2、Z4 小环上的 J6 前提假（`-J6_premise_fails`）。tsv-406（同 414）在 E1、N = 3 上是 `+J6_premise_holds`。
- 红例账 151 行，原实现与变体的二进制各单条重放，判定与逐态行相同的 151 行：`awk -F'\t' '!/^#/{t++; if($8=="true"&&$9=="true")c++} END{print t,c}'` 输出 `151 151`。
- 登记第九节甲表「推的最先改变输出的取样点」逐行对过：每一行在它推的那个 (域, N) 上都红了、判据名也对得上；另外多数变体在别的域红得更早（例 tsv-406 在 C 域 Nc = 2），tsv-1004 在 E1、N = 1 上先被「合法历史一族报错态数」这条实现锚点抓到。推 7（448 可能每个域都不红）、推 8（226 只有 IA2）量到了。

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:158-180`（整段抄，未转述）**

```markdown
### 逐行判定

| 登记的量 | 读数（合法历史一族） | 照登记第六、十节记成什么 |
|---|---|---|
| Q1 原实现 J1（返回的根） | L、L6、L9、U 验证开、X、E1、C、C6 全 0 | F1 没触发：界内原实现在合法历史域里过 M2 的「返回的根」取法 |
| Q2 原实现 J2（根加盘） | 同一批域全 0 | 原实现那一半 J1 与 J2 逐域同为 0：界内两种取法分不开（F2 的前一半；后一半要第二段的变异） |
| Q3 J3（在飞上限） | 大环、6 槽、9 槽全 0 | 原实现过 I-8.3（重放前缀严格连续） 的上限那一句 |
| Q4 J4（最远一版） | L、L6、L9、U 验证开、X、E1、C、C6 全 0 | F6 没触发：「加」那一题面对今天的实现在界内为真 |
| Q5 J0 与报错态数 | J0 全 0；报错态数只有 E2 的 42（42 态全报错，就是 IA1） | 合法历史一族（L、L 小环、U 验证开、X、E1）报错态数 0，与登记第七节甲类锚点相符，F11 没触发 |
| Q6 J5a、J5b | J5b 在五个对集合、两个环上全 0；J5a 在「宽」「甲合法」「甲同段」「乙合法」「乙同段」都不为 0（大环 830 685 / 4 969 / 911 / 8 146 / 911）；甲同段的 911 全在「停在根槽段」那一行，「停在记录段」是 0；理想恢复与原实现逐格同值 | 施加集读法的违例是读法的性质、不是实现的：任何落到最远一版的恢复都有。哪个集合算「段内」由主 agent 判；不管判哪一个，J5a 都不为 0，J5b 都为 0 |
| Q7 J6（不动点） | 前提真：L、L6、L9、U、C、C6、X 全 0，E1 是 70（第二次调用在 txg 顶格的根上报错）；前提假：大环上一个态都没有（上限咬不到），L6 354 / 1 272，L9 51 998 / 557 792，C6 215 / 596 | F8 触发在 E1：不动点读法带前提也为假，见证是第一次落到 txg = u64 最大值的那一版、第二次报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`。F9 没触发：小环上前提假的态里 J6 不为 0，前提承重。随几何翻面：前提假的格大环没有态、小环不为 0，两点都写 |
| Q8 关掉点名验证 | U 验证关：J1 是 0，J2 是 6 726 648，J4 是 6 726 648，IA2 是 0 | F12 按「根加盘」取法触发、按「返回的根」取法不触发：关掉点名验证那一种策略自己过 J1、不过 J2 |
| Q11 R 域 | J1 是 956（6 964 态里），见证 N = 2 | F17 没触发：不带「同一 (实例代号, 计数器) 只对应一份内容」这个前提时 M2 字面为假，前提承重 |
| Q12 域非空 | 每个域 `domain_is_nonempty=true`（E2 除外照登记不要求）；C 域所选根不是 R_0 的态数大环 N = 6 是 115 520 | V4 没触发 |
| Q9 A-互、AS 三条 | 三个 A-互 数在 A、A6、AC 都不为 0（A：175 616 / 509 216 / 333 600）；AS 三条在三个域都不为 0（A：330 240 / 154 624 / 488 224） | F13 没触发：三种读法界内分得开。F14 触发：今天的实现与三种读法都对不上（同一个域里三条都不为 0） |
| Q10 A-结构、A-限 | 三个域全 0 | F16 没触发：对抗域带前提之后原实现过结构与上限两条 |
| Q13 变异表的杀伤界（25 个变体） | 条款判据红 23；只被实现锚点抓到 1（表第 226 行，U 验证关 N = 2 上 IA2）；界内不红 1（表第 448 行，每个域到上沿都与基线逐行相同）；无效 0 | F18 触发：第 448 行界内不红；第 226 行另列「条款判据界内不红」。F19 不适用（修订 10 去掉了放宽）。F15 触发：第 613、614、514、448 行在合法历史一族每个域都界内不红；其中 613（A、n = 1）、614、514（A、n = 2）在对抗一族红，448 两族都不红 |
| Q13 看 K3 的四行 | 第 518、406、414、36 行：J2 在 L、L6、U、C、X、E1 上红（N = 2–4；第 36 行在 E1 上到上沿都不红），同一个 N 上 J1 不红；J1 只在 R 域红（518、406、414、36 都是 N = 3） | F2 没触发：J1 与 J2 的首红 N 在合法历史域上不等，界内分得开。F3 没触发：这几行在 R 域的 J1 上红。注意：每个域红了就不往上扫，这些域里 J1 在更大的 N 上红不红没量 |
| Q13 看 K5 的 Z1 | Z1 在 L、L6、U 验证开、X（N = 2–3）只有 J4 与基线不同；在 R（N = 2）J1、J2、J3 不同，在 U 验证关（N = 2）J2 不同，在 C、C6（Nc = 1）J5a 不同，在 E1（N = 2）J6 前提真不同，在 A、A6、AC（n = 1）AS 三条不同 | 照登记「与基线不同」的字面，F5 触发（J4 之外还在 J1、J2、J3、J5a、J6 上红）。这些差别在红例账的反例状态上都是变体少了违例（R 的 `-J1`、E1 的 `-J6_premise_holds`），C 域 J5a 的方向装置没报，读 `judge_a_legal_state` 推的是变体永远落在所选根、施加集为空、J5a 违例变 0（没量过）；按「多出违例」读，合法历史一族里只有 J4 红，那是 F4 的形态。取哪种读法由主 agent 定 |
| Q14 J6 分不分辨变异表 | 表第 406、414 行在 E1、N = 3 上多出 J6 前提真的违例（反例状态 `+J6_premise_holds`）；第 24、512、517、921、922 行与 Z1 在 E1 的反例状态上少了 J6 违例；Z2、Z4 在小环的反例状态上少了 J6 前提假的违例（都只看首红那一个状态，不是整域合计） | F10 没触发：有变异在 J6 上红，其中 406、414 是多出违例 |

推 1–推 11 里第一段量得到的：推 1（施加集读法在「记录全落、根槽落没落」那一对上为假）量到了（C 域 Nc = 1「宽」集合 9 对里 1 对）；推 2（小环上前提假时不动点为假）量到了；推 3（关掉验证过「返回的根」、不过「根加盘」）量到了；推 9（残留域里不过「返回的根」）量到了。登记没推到的一条：E1 上前提真的 J6 违例 70。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:181-186`（整段抄，未转述）**

```markdown
### 钉绝对值的断言与阳性对照

- 登记第七节乙类锚点（各域每个 N 的状态数、L 域 N ≤ 7 的见证数、C 域的对数与合法崩溃状态数，含第十二节补的小环两组）在装置里逐格 `assert_eq!`，一格不等进程当场退出；两份产物都跑到了 `name=done`。
- S 自己的对照（登记 5.4、作废条款 V3）：L、L6、L9、U 验证开、X、E1 里 S 三种读法的期望落点与 j\* 不等的态数，123 行 `name=specification_self_check` 全是 `mismatches=0`。
- 阳性对照 35 个读数全红（`controls_not_red=0`）：J0 是 1、J1 是 762、J4 是 762（都等于对照界里 j\* ≥ 1 的态数，登记第七节见证数 N = 1..5 之和）、J2 是 1 032 而 J1 是 0、J3 小环 354 而大环 0、J5a 是 225、J5b 是 35、J5a 与 J5b 在九个对集合行上逐行相等、J6 是 3 110、IA1 是 42、IA2 是 2 058。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:187-198`（整段抄，未转述）**

```markdown
### 几何敏感性

| 旋钮 | 取样点 | 判定翻不翻 |
|---|---|---|
| 环槽数（在飞上限） | 64 槽（上限 21）、9 槽（3）、6 槽（2） | J1–J4 三个点都是 0，不翻；J6 前提假那一格大环没有态、小环不为 0，翻；J5a、J5b 大环与 6 槽小环同判定 |
| 历史的起始计数器 | 大环 7（不回绕）、小环等于环槽数（历史中间回绕） | 随环一起，J1–J4 不翻 |
| 所选根的 txg | 100、u64 最大值 − 1、u64 最大值 | J6 前提真那一格在 u64 最大值 − 1 上翻成不为 0 |

在飞上限的分布：大环上表里所选根之上的记录条数最大 9（L 域 N = 10），大于上限的态数 0——上限在大环上咬不到，不是「不动点在大环上成立」的证据。J6 为假的态上第三遍与第二遍相不相等（只报不判）：L6 的 N = 6 是 210 相等、62 不等，L9 的 N = 9 是 33 904 相等、7 600 不等。

没扫的旋钮：盘数（两块，记录只在 0 号盘）、记录字节数与 F、实例代号的取值、一条记录点名几个单元。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:199-202`（整段抄，未转述）**

```markdown
### `light` 档

用户 2026-10-01 定这个装置当轻量工具。`--bounds light` 的界：L 到 9、U 到 5、A 到 3，其余各域同第一段。取法：草稿试跑量到 10 线程上每态挂钟 0.24–1.2 微秒，每个域取到 3 秒之内。入库的是 `--family legal` 那一份（6 923 358 态，进程内单调钟 2.671 秒）；状态数与见证数钉的是与第一段同一张常量表。量的时候同机有别的会话在跑 31 线程的广谱变异与求解器，开跑之前 `uptime` 报的三个负载均值是 10.19、29.98、34.04（原样输出在计时产物的头两行）；第一段 `--family legal` 在同一负载下 29.061 秒。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:203-208`（整段抄，未转述）**

```markdown
### 变异

单测 16 条（`grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`；第二段加的两条管单状态重放的参数与逐态行的写法）；装置变异 17 条追加在 `crates/mutations.tsv` 第 1477–1493 行（名字以「E165 」开头）。单测与变异都没跑：checker 档包的测试归提交时的门禁 checker-tier-crates-mutation-replay。装置每次开跑先做 13 项自检，其中每一项与一条单测调同一个函数，三份第一段产物里 13 行 `name=self_check` 都是 `passed=true`。

第二段量的变异表是登记第九节甲表（`crates/mutations.tsv` 里动 `replay_journal` 那五个函数的 23 行，加自造表 `research/mutations/e165_replay_bounded_enumeration_custom.tsv` 的 Z1–Z4），驱动开工照登记的取法重选一遍、行号与登记相同才跑。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:209-214`（整段抄，未转述）**

```markdown
### 复跑

`bash research/scripts/replay.sh E165`（登记行 `driver_e165`、`driver_e165_light`、`driver_e165_adversarial`、`driver_e165_segment2`，判据 exact）。计数与线程数无关：第一段 `--family legal` 用 10 线程与 7 线程、`--family adversarial` 用 16 线程与 7 线程各跑一遍，`E7RESULT` 行逐字节相同。

第二段的驱动 `research/scripts/e165-replay-mutation-kill-bounds.sh`：`--list-items` 列 26 个变体；每行产物带只复跑那一条的命令（`--recheck --item <变体>`，重跑之后拿新行与产物里的行比）；中途断了重跑同一条命令只补没跑的。红例账每行带单条重放的命令 `target/release/e165_replay_bounded_enumeration --replay-state <域>:<N>:<序号>`，原实现上重放的判定就是那一行第 12 列。2026-10-01 在 `--jobs 4 --threads 4` 上整段 32 分钟，同机负载均值 17–90（原样在第二段计时产物里）。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:215-224`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 今天的实现 | 调 `replay_journal`，取返回的根 | `crates/singlefs-core/src/recovery.rs:3057 replay_journal`；装置里 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:927 call_replay_journal` | 生成器造的记录字节与 `records` 表；`JournalRecord::parse` |
| 2 | 照条款写的最远一版 j\* | 声明式：从所选根起连续完整、记录条数之和不超过在飞上限的最远一版；只读生成器记下的真相 | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:827 farthest_version_of` | 本地常量（环槽数、F）；生成器的真相（哪条记录属于哪次发布、哪条在盘上）；D23（journal 的角色与格式） 已定项 14 的五条口径 |
| 3 | 照条款写的全函数 S（三种读法） | 逐条取前缀：登记 5.3 的第 1–4 条与 (a)–(k) | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1082 specification_expectation` | `records` 表（与路径 1 同一张）；本地常量；生成器记下的链值与点名单元真相；同一批条款 |

连起来比的断言：路径 1 对路径 2 是 J4（`judge_a_legal_state`，第 1941 行起）；路径 3 对路径 2 是 S 自己的对照（第 2028 行）。三条路径共用生成器与同一批条款：生成器造错或条款被三处一起读错，三条会一起错而逐格相等；钉它的是登记第七节的状态数、见证数锚点（另一份 python 脚本算的）与阳性对照。取值是扫遍界内的定义域，不是枚举点。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:225-236`（整段抄，未转述）**

```markdown
### 它答不了的

- 界外的输入：N 大于各域上沿、一条记录点名多个单元、空发布记录、多于两个实例、跨环一圈以上的覆盖，都没扫（登记 5.1 末段）。「0 违例」只说界内。
- 对抗域里序号 0 的记录：`parse` 拒收、进不了表，S 的 (e)「序号 0」在建出来的表上走不到；读者拿到序号 0 以外的坏记录怎么判，不在这个字母表里。
- 第二段的「首红的 N」是逐行与基线不同的第一个 N，不分方向：变体少了违例也记红（「逐行判定」Q13 的 Z1 那一行）。每个域红了就不往上看，红了之后别的判据在更大的 N 上红不红没量。
- 第 448 行界内不红，是真盲区、等价变异还是取样点不敏感，登记写明不在这个实验里判（F18）。
- 红例账的方向只看首红那一个状态；整域上多了还是少了违例没有单独数。
- S 第 3 条 (b)「链上遇到水位之下的记录」取「停」：条款没逐字写，主 agent 2026-10-01 推的。合法历史一族里这一格走不走得到没有单独数。
- J6 只量了「不动点」这一种读法；K7 另外三种读法不在装置里。
- Q6 的乙同段「按停在记录段 / 停在根槽段分行」照字面分不出来（切法乙没有单独的根槽段），装置报的是「根槽那次写在两个状态上取值同不同」两行，够不够由主 agent 定。
- `replay_journal` 在 `crates/` 里的调用方除登记 5.3 写的三处之外，还有入库装置 `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`（登记第十三节列过）；三处调用除布尔开关与读者之外有没有别的差别没逐行核（停机条款 H5 只核到 grep 这一步）。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:237-246`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 备料 | 2026-10-01（第二段跑完后重看）不受影响：M1–M3 的题面取哪种读法（K3、K5、K6、K7）归三方判决，这一轮不改决策；第二段给了 K3（J1 与 J2 界内分得开）、K5（Z1 两种读法下各落 F4 / F5）的变异读数，在「逐行判定」一节 |
| D13（验证路线） 已定项 4 | 备料 | 2026-10-01（第二段跑完后重看）不受影响：C 域两种切法只是照已定项 4 的字面各摆一种，根槽那次写算不算同段由主 agent 判，条款没动 |
| D23（journal 的角色与格式） 已定项 4 | 备料 | 2026-10-01（对抗一族入库后重看）不受影响：读者规则第四格的三种读法（K9）界内分得开，今天的实现与三种都对不上（F14），而 A-结构、A-限 为 0；读者规则那几条变异（613、614、514、448）在合法历史一族界内全不红（F15）。这些是给三方的读数，条款没动 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-01（第二段跑完后重看）不受影响：五条口径与注 1 照字面写成 j\* 与 S，界内合法历史一族上今天的实现与它们逐态相同，条款没动 |
| D16（发布语义） 已定项 7 | 备料 | 2026-10-01（第二段跑完后重看）不受影响：关掉点名验证那一臂过「返回的根」、不过「根加盘」；第二段里「点名单元验证」的变异（表第 354 行、Z3）只在 U 验证开 N = 2 的 J2、J4 上红，与已定项 7 同向，条款没动 |

```

**出处 `.claude/kb/decisions/13-验证路线.md:1-4`（整段抄，未转述）**

```markdown
## D13 验证路线 —— 已定

D13（验证路线） 管本工程拿什么验正确性：三类现成办法全做——自写参照实现做差分对拍、有界穷举崩溃测试、crash refinement 形式验证（调查见 [prior-art.md](../prior-art.md) 7.4）——并且不照抄，针对本工程特有的四样（COW 每事务发新根、Merkle 自验证、整卷 AEAD 的 nonce 唯一性、没有参照实现）自己设计验证手段。它定 oracle 怎么分、自研哪两样、崩溃点重放怎么分层与抽样、崩溃状态集合怎么定义、checker 与实现共享什么。不管的：每条不变量写什么在 [invariants.md](../invariants.md)；三样验证手段怎么落地、谁挡着谁在 [verification-build.md](../verification-build.md)；提交步骤登记在哪、结构等价类怎么分、今天几个，归 D17（实现分层与第三方管道） 已定项 2。

```

**出处 `.claude/kb/decisions/13-验证路线.md:25-42`（整段抄，未转述）**

```markdown
#### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」

**定案**：crash refinement 的第一个验证对象是恢复函数 `Recover` 的关系性质，最小封闭单元是「一次发布 + 一次崩溃 + 一次恢复」。三个里程碑依次是：

| 里程碑 | 义务 | 落点 |
|---|---|---|
| **M1 全性** | `Recover` 在 D13（验证路线） 已定项 4 枚举的**每一个**崩溃状态上有定义、终止、输出一个能过 I-7.1（根槽有效集非空） 与 I-8.3（重放前缀严格连续） 的镜像 | 脚手架。**完成判据不是「证出来了」，是「M2 能在它上面写出来」**——层 0 崩溃点重放本就全量枚举（[verification-build.md](../verification-build.md)「层 0 不抽样、全量枚举」），恢复卡住当场暴露，单独证它没有边际价值 |
| **M2 前缀不切发布** | **任一 `jsn` 连续前缀，其重放结果等于某个整数个已发布的 checkpoint 的结果——不存在被前缀切开的发布；前缀停在一次发布中间时，那次发布整体不施加**（D16（发布语义） 已定项 4「一次发布整体施加或整体不施加」） | 落点是 I-8.3（重放前缀严格连续）——它的判据补句逐字就是这一条（「前缀不得停在一次发布的中间——一次发布整体施加或整体不施加」），不要在 [invariants.md](../invariants.md) 里另立一条；crash refinement 侧还欠的是把它证成 `Recover` 的全称性质 |
| **M3 幂等与单调** | `Recover(Recover(c)) = Recover(c)`（I-8.4（重放幂等））；段内 `c ⊆ c'` ⇒ `Recover(c)` 施加的事务集 ⊆ `Recover(c')` 施加的事务集（后半全仓无落点） | O2（独立解析器 + checker） 判不了：I-4.5（已停用·需两镜像） 退役理由逐字「幂等要比对两个镜像，单镜像判不了」；记录核对器的入参里没有第二次恢复的结果，`crates/` 里没有判它的检查 |

**射程**：refinement 的边际价值只在层 0 枚举取不到的那几维——载荷字节、记录条数 / 段数 / txg 数（含环回绕）、上一条时间线的残留、槽数与设备数；崩溃点那一维层 0 已全量枚举。「校验和过了」在模型里是一个布尔标志，**不许读成 refinement 验了完整性**——与已定项 14 里 D4（校验和位置） 那一行「不许读成『验了 Merkle』」同形，也与 D13（验证路线） 已定项 4 对 CRC32C 碰撞概率的「知情接受」是同一笔账。D13（验证路线） 已定项 4 是 refinement 左边的 `Crash`（模型层的枚举域），不是右边的 `Spec`，拿它当 spec 会让 refinement 恒真。I-6.5（nonce 水位单调） 的形式化撤下；D18（块里携带什么信息） 已定项 14 定了、且 nonce 分配规则（单调计数器 + 预支水位、水位在使用前持久）被立成一条分项之后，这一格重开——重开时它唯一的真优势是 nonce 对 POSIX 不可见，它的 spec 与 O1（内存模型对拍（HashMap 理想文件系统）） 的 model 结构上不可能同源，不受冲突 2（D13（验证路线） 已定项 13） 影响。

**依据**：
- E42（一事务几条记录）：为了让「被切开 ⇒ 非法」成为恒等式而钉死了载荷，「被切开却碰巧合法」这一类它结构上看不见——M2 把载荷符号化补的正是这一格。
- 三方：材料 `_d13-item1-background.md`，三条腿的产出 `d13-item1-opus-forward-output.md`、`d13-item1-sonnet-reverse-output.md`、`d13-item1-local-attack-v2-output.md`；三腿一致把原候选排最后，两条云端腿各自收敛到「前缀判定与提交原子性」这道接缝，主 agent 按封闭性把择新与发布原子性收进同一个单元（判决与撤下原候选的三条门槛原文在变更史）。

**欠**：C37（事务与记录的对应无定义）：M2 的可判定形式；C77（重放起点未定义）：在择新与前缀判定那道接缝上。

```

**出处 `.claude/kb/decisions/13-验证路线.md:43-56`（整段抄，未转述）**

```markdown
#### 已定项 2：先串行提交

**定案**：事务层先承诺串行提交——`.claude/rules/fs-design.md`「判据写不出来时，先只做一条，而且**先做慢而正确的那条**」落到实处，串行是本工程明令的默认值，不是在两条路里挑了一条。越过交点再开并行，而不是现在就承诺并行。对将来并行提交的一条前置约束：记录按事务号顺序追加，事务号 N 的 fsync 在返回前必须等 < N 的事务全部已追加 ∨ 已作废 ∨ 已确定实例切换或转只读（作废是提交管道的内存状态，对并发后继可见）；串行提交下这条规则空转。

**射程**：串行的天花板在一次 checkpoint 约 1.1 万条条目（单线程合并耗时追上一次设备同步的交点，算式在 C58（串行提交撞天花板没人报警））：低于交点设备同步是瓶颈、串行不吃亏，高于交点单线程合并变成天花板，没有安全余量。已定项 2 不决定记录头的字段清单、不改变新池新建文件写出的字节，它决定性能天花板与 crash refinement 的范围。并行提交仍然会动崩溃语义：「前缀」是全序上的概念，并行下一个前缀可能同时含有完整的事务 B 与残缺的事务 A；记录头已带事务边界字段（D23（journal 的角色与格式） 已定项 7：事务号 8 + 提交标记 1）。

**依据**：
- E17（write buffer 合并上限）：单线程 sort-merge 的上限是交点的一半；按叶分区的无锁并行是越过交点之后的逃生口，但不许默认它够。
- E44（序号位宽的本机实测与代价）：本机 fsync 率是交点的另一半。
- E42（一事务几条记录）：串行下一个事务跨多条记录同样会被前缀切开——决定记录头字段清单的是「一个事务恰不恰好产生一条 journal 记录」，不是串行 / 并行。
- 用户定案，记在变更史；将来并行提交那条前置约束来自 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1。没走三方论证：这是确认一条已经写明的默认值，真正要走三方的是何时改成并行，那一步没做。

**欠**：C58（串行提交撞天花板没人报警）；C37（事务与记录的对应无定义）：记录与事务对应的不变量与崩溃点重放。

```

**出处 `.claude/kb/decisions/13-验证路线.md:57-69`（整段抄，未转述）**

```markdown
#### 已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义

**定案**：不做只管功能的内存模型，做一个自己处理持久化与崩溃语义的参照实现（RefFS）。这条定案接受「工程量接近再写一个文件系统」这个量级，把 RefFS 从「储备候选」升为**要做的东西**。RefFS 与实现不许共用崩溃点枚举的那段代码：做到极致之后「三 oracle 合谋测试」（已定项 6）防的那个共同误解，最危险的落点是「RefFS 与实现是否共享同一个崩溃语义的错误理解」。

**射程**：只定做到什么程度，不定什么时候做（排期未定）；不许读成「先做 RefFS 再做实现」——RefFS 是验证手段不是里程碑（`.claude/rules/format-evolution.md`「从写开始，不从读开始」）。D17（实现分层与第三方管道） 已定项 4 据此把「入口」的可执行定义交给 RefFS 的操作面，fsync 的持久性承诺由 RefFS 的崩溃语义定义；O3（独立规约执行器） 仍是崩溃点集合的独立执行器，职责不变。`crates/` 里没有 RefFS（`grep -rn RefFs crates/` 零命中）。

**依据**：
- 无实验：做到什么程度是验证投入的取舍，没有可量的量。
- 用户定案，记在变更史。
- D17（实现分层与第三方管道） 已定项 4：O1（内存模型对拍（HashMap 理想文件系统））明写「不管崩溃」，定义不了 fsync 的持久性承诺，而第一个可运行目标恰恰是「正确提交一个事务」——做到极致的 RefFS 补的正是这一半。

**欠**：C447（独立规约执行器与 RefFS 都零代码）；C191（同源性计数器只盖了一对）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:70-85`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，全量与快档都枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。**同一块盘上连着的屏障并成一道**：录制流里一道屏障与这块盘上一道屏障之间没有任何一次写（别的盘的屏障不算写）时，这一道不记（`crates/singlefs-harness/src/lib.rs` 的 `push`），独立模型照同一条规矩记；中间没有写，这一段不产生崩溃状态，合并与否不改变崩溃状态集合，只改变操作数与种类串。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。层 0 的扇区取 512 字节；两条层 0 流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条层 0 流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；层 0 的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（新池新建文件的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案，2026-09-26 用户定照实现改成「关掉它所在的那一段，与前面没被屏障隔开的写同段」），记在变更史。
- 按设备切段（有一块盘没放行就不关段）、FUA 只放行它那块盘、原地覆写多取「新旧都读不出」：用户定案（代码审阅第 1、3、4 条；2026-09-27 弹窗选「A 合并，按设备记屏障」「照补，全量也跑」与「原地覆写补第三态」），第三态按字节撕是主 agent 同日定的、被攻过零轮，记在变更史；实现与判别力（只吞盘 1 一道屏障，段尾 `26,2,1,2` → `26,5`，层 0 小流里记录核对器报出来）在 `research/prompts/m2-rev-b3a2-implementer-report.md` 第一、三节。
- 同一块盘上连着的屏障并成一道：用户 2026-09-28 定「并，跟今天的录制器」（E142（新池新建文件的干跑） 第十九次跑撞上登记的停机 S19-crates：独立模型两道都记、录制器并掉第二道，暖机与整条流四格的操作数与种类串不同、崩溃状态数相同，报告 `research/prompts/e142-r19-runner3-report.md` 结论段，登记 `research/prompts/e142-r19-prereg.md` 第 85 行 R40），原话在变更史。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/13-验证路线.md:86-102`（整段抄，未转述）**

```markdown
#### 已定项 5：只共享一份从 kb 生成的常量

**定案**：checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改（`.claude/singlefs-ai-sop/rules/machine-first.md`：重复要生成，不能手抄）。**其余一律不共享**：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空（C12（增量语义共用） 由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格定的判据，形态是 crate 粒度的依赖闭包交集、不是符号级，「格式解析与常量除外」那条例外就是这一项在用）。

- **判定宽度不许从别处拿**：I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 的判据是挂载时探测的 `physical_block_size`，它不在镜像里。checker 自己探一次，再与系统配置记录的 mkfs 时取值比对（D22（单元原子性怎么合成） 已定项 2「mkfs 按池内最大值划、挂载时逐设备复算比对」的 checker 侧形态）。**探不到时报「声明值，未探测」，不许当成探到的**——文件当设备的镜像属于这一类，崩溃点重放的绝大多数镜像都是文件，所以这条不是边角。异构池里判定宽度是一个集合不是一个数（C17（设备几何当全局参数）），checker 要复算 mkfs 那步取最大值的聚合，不然健康的异构池上会假红。
- **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。
- **CRC32C 各写一份的前提是参数钉死**（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性。

**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 code-source-discipline 的 format-constants 格按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源只有类型（`crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`）、没有接到判定上，I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。

**依据**：
- 无实验：共享边界是保验证独立性的政策，没有可量的量；它的判别力靠两边各写一份的检查逐条红给人看。
- 三方对抗：材料 `_checker-sharing-vehicle-background.md`，腿的产出 `research/prompts/checker-sharing-vehicle-*`，腿况在 [verification-build.md](../verification-build.md)——正推腿从 D13（验证路线）、C12（增量语义共用） 推出「常量在例外内、其余禁止」；替代反推腿与本地腿各自构造出同一条：kb 里写错的值共享与各抄都抓不住。
- 用户定案，记在变更史。

**欠**：C94（登记的格式常量与后来的定案对不上）；C17（设备几何当全局参数）；C387（常量模块手写，不是从 kb 生成）；C388（checker 的判定宽度不自己探测）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:103-126`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构 | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导 | 规约本身可能写错——**必须配阳性对照才有意义** |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
本工程功能正确性的全部故事就是这三个 oracle。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 今天的形态是 `crates/singlefs-harness/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 harness-model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 零代码；RefFS（已定项 3）没有；「三 oracle 合谋测试」没有（`grep -rn 合谋 crates .claude/gate.d` 零命中）。

**依据**：
- 无实验：三个 oracle 的分工与各自独立性的来处是验证体系的结构设计，没有可量的量；独立性要靠「三 oracle 合谋测试」观测，那个测试还没有。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C445（模型对拍的取样点不随写路径增长）；C446（未实现的不变量没有前置登记，也没有到期）；C447（独立规约执行器与 RefFS 都零代码）；「三 oracle 合谋测试」没有 C 编号（C407（三 oracle 合谋测试没有代码） 记着它零代码，那一格本身仍没有编号）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:127-145`（整段抄，未转述）**

```markdown
#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像

**定案**：

> **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
> 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`——持久集合是枚举器给出的这个崩溃状态里哪些写落了盘（层 0 的段内整写子集）；只看盘上字节分不开「写没落、而原地内容恰好相同」与「写落了」，判复用与缺席要拿持久集合判；它与自研第 1 项「Merkle 根链差分」（已定项 8）同属二元检查。⚠️ **故意不给它编号**——O2（独立解析器 + checker） 那场混乱就是编号当称呼造成的，再加一个 O4 只会给下一次漂移多一个落脚点。⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。

**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-checker-tier/src/crash.rs`，接在层 0 的每个崩溃状态上（门禁 checker-tier-layer0-replay）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 层 0 不覆盖可写挂载：层 0 在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在层 0 的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-checker-tier/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。

**依据**：
- 无实验：**依据是类型，不是成本**：「逐字节比对」是二元关系 `R(镜像₁, 镜像₂)`，**没有任何投入能把一元谓词变成二元关系**；I-4.5（已停用·需两镜像） 因同一条门槛退役。
- 「崩溃后镜像」是两份：随 D16（发布语义） 已定项 4 收窄问法那一轮三方论证（材料 `_d16-items134-background.md`）由用户定案，记在变更史。
- 持久集合那一样入参：用户定案，原话在变更史；判别子观测不到的实据在 `research/prompts/m2-layer0-scale-r3-main-verification.md` 第二节按扇区豁免那一行。
- 层 0 不覆盖可写挂载、可写挂载与二次崩溃由崩溃注入覆盖：用户定案（代码审阅第 2 条，2026-09-27 弹窗选「崩溃注入层补可写挂载」），记在变更史；崩溃注入那一侧的实现在 `research/prompts/m2-rev-b3b-implementer-report.md`。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:146-169`（整段抄，未转述）**

```markdown
#### 已定项 8：自研的两样：Merkle 根链差分与代际增量 scrub

**定案**：现成工具没有、本工程自研的两样：

**1. Merkle 根链差分 —— COW + 每事务发新根独有的杠杆。**
历史根在被回收前物理存在过，构成一条可回放的链。对相邻两代根递归比较「子树指针 + 校验和/MAC」，
**指针没变的地方直接跳过不下钻**，复杂度只正比于两代之间实际改动的部分。
它能检出常规遍历式 checker 检不出的一类错误：**同步篡改后自洽的原地写**——
某次写绕过 COW、原地改了块内容并同步更新了校验和，单点看它是自洽的（I-2.1（校验和与内容匹配） 全绿），
而根链差分会发现矛盾：**没走 COW 就不该有新分配，父指针理应没变，校验和却变了**。
⚠️ **必须配阴性对照**：正常 COW 提交后跑同一检查必须全绿，否则它会把每次正常更新都当成篡改。

**2. 代际增量 scrub。** 靠 I-1.2（块头写序已发布） 与 D5（快照 / 空间记账机制） 的 birth txg
维护一个「上次 scrub 扫到的 generation 水位」，只遍历 `birth generation > 水位` 的块——
老子树的校验和链没变就必然还自洽。**它让「每次提交都全量验证本次改动」在挂钟上可行，不必抽样。**

**射程**：被搬过的副本 `birth` 不变、会永远落在水位之下，代际增量 scrub 看不见它们。`crates/` 里没有根链差分（`grep -rn 根链 crates` 零命中），也没有代际增量 scrub（`grep -rn scrub crates` 零命中）。

**依据**：
- 无实验：两样都靠 COW 每事务发新根与 birth txg 的结构性质成立，今天都还没有实现，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C389（Merkle 根链差分零代码）；C194（代际增量 scrub 看不见被搬过的副本）；代际增量 scrub 零代码没有 C 编号。

```

**出处 `.claude/kb/decisions/13-验证路线.md:170-198`（整段抄，未转述）**

```markdown
#### 已定项 9：崩溃点重放要分三层，否则挂钟必炸

**定案**：

| 层 | 范围 | 触发 |
|---|---|---|
| 层 0 冒烟 | 固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模（规模见射程），精确全量 | checker 档的全量（已定项 15）：用户要求或夜间跑；提交时默认只跑快档 |
| 层 1 常规 | 中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore，覆盖报告与门禁逐个列「本次未跑」 | 每次提交 |
| 层 2 全量 | 层 1 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量 | 夜间 / 发布前 |

**剪枝加 ignore 之后敢说什么**：敢说「开着的节点在全部崩溃状态上通过」；
**不敢说**「ignore 的节点通过」。

**只能夜间跑的也算准入判据，但只能是异步准入判据**：
**提交前 = 快速档全绿（必要条件）；合并前 = 夜间档全绿（准入条件）。**
夜间档红了，处置不是回退，是「之后所有提交都带着一个未结的红」直到修掉。
**夜间档必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个夜间档 bug 的挂钟 ≈ `log2(N) × 一天`，一个不可复现的夜间失败价值接近 0，
甚至为负，它训练人忽略红灯。

**射程**：今天只有层 0（门禁 checker-tier-layer0-replay）；层 0 分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；全量带断点续跑（门禁 checker-tier-layer0-replay `--full`），登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档，全量按已定项 15 由用户要求或夜间跑；层 1、层 2 在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；节点怎么精确剪枝、ignore 节点什么时候开归已定项 10、11。

**依据**：
- 无实验：分层是门禁挂钟的设计约束，层 1、层 2 的负载与挂钟都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 平时快档用甲二：用户定案，原话在变更史；层 0 规模三轮判决 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- 用户定案：「我们不是抽样 我们是剪枝 加 初期ignore」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定案：不抽样，剪枝加初期 ignore」那一行）、「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）、「1 该枚举该设计的设计完 设计全」「2 枚举不完 正是剪枝的用处」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节里材料员被门禁 doc-process-records 的 implementation-premise 格挡下、用户定『设计全、枚举不完靠剪枝、粒度合适』的那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 9 推翻。

**欠**：层 1、层 2 在门禁里没有阶段，没有 C 编号。

```

**出处 `.claude/kb/decisions/13-验证路线.md:199-228`（整段抄，未转述）**

```markdown
#### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」

**定案**：层 1 精确覆盖先缩问题，再谈剪枝。崩溃点的定义域是**事务层状态机**，而 `.claude/rules/fs-design.md` 有两句顶层规则：
「一个事务层，所有结构共用」与「唯一不许多样化的是事务层」。所以布局能改的只有两类：

| 差异 | 改变的是 | 判定 |
|---|---|---|
| 节点大小、条带宽度 | 一步写多少字节、某步重复几次 | **参数**，不新增步骤种类 |
| 旋转 vs SSD | seek 代价、IO 调度 | 落在设备层，**不落在事务层协议里** |
| **zoned（ZNS）** | **能不能原地覆盖根指针** | **结构性**：只能追加、根位置漂移，且多出 zone finish/reset |

**落地手段是一个封闭枚举 + 穷尽 match**：`crates/singlefs-core/src/transaction.rs` 的 `CommitStep`，步骤种类的登记位是 [layout/01-first-txn.md](../layout/01-first-txn.md) 八的段序列登记表。
任何布局要引入新的结构性步骤**必须改这个共享定义 ⇒ 编译器强制所有引用处补分支 ⇒
这个 diff 门禁 grep 得到**，可以自动打标「本次引入了新的结构等价类，需为它新开一轮独立层 2 重放」，
**不靠人记得申报**（按 diff 算门禁范围欠在 C8（门禁范围判不出来））。再加 newtype 隔离（`RotationalPtr` / `ZonedPtr`），
把 D12（目标介质） 已定项 2 里「不许共用」那一桶（记账事实的判定路径）变成编译期约束。
⚠️ **它有第二个职责（D17（实现分层与第三方管道） 加的）：它是第三方管道的接入点。**
协议归本工程所有，一条管道要引入新的结构性提交步骤必须往本仓发这个 diff。

**射程**：旋转与 SSD 在提交协议层面同构，由 D12（目标介质） 那一轮三方论证承担；zoned 那一行只说明它至少多出一个等价类。等价关系按录制流的段序列同构、今天几个，归 D17（实现分层与第三方管道） 已定项 2。按种类串认节点够不够，第一轮判决打中（挂起，待第二、三轮攻）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节。
⚠️ **它钉住的只是「有哪些步骤」，钉不住「每一步的失败原子性宽度」**——
持久内存与对象存储改变的是后者，枚举接不住（缺口与补法见 D17（实现分层与第三方管道））。
`RotationalPtr` / `ZonedPtr` 这层 newtype 隔离 `crates/` 里没有（`grep -rn "RotationalPtr\|ZonedPtr" crates` 零命中）。

**依据**：
- 无实验：N 是什么由 `.claude/rules/fs-design.md` 那两句顶层规则推出，没有可量的量；等价关系怎么判、今天几个的依据在 D17（实现分层与第三方管道） 已定项 2。
- 三方论证：层 1 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。

**欠**：C8（门禁范围判不出来）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:229-263`（整段抄，未转述）**

```markdown
#### 已定项 11：ignore 的节点什么时候开：否决按欠账轮转的账本，留下三条

**定案**：

**「按欠账轮转的账本」调度：否决。** 格子 = `(布局, 子阶段桶)`、按欠账降序取队首、声称任一格子最多 `K_max = ⌈N×M/C⌉` 次提交必被覆盖的那个调度被否决：它保证不了的恰恰是它要防的那一类——慢的、罕见的分支（实测验算与另外四条否决理由在变更史）。

**留下的三条**：

1. **方向对**：要一个**确定性的、能被门禁判红的覆盖记录**，而不是概率论证——
   概率保证在长期维护里等于没有保证。

<!-- doc-lint:registry name-col=2 -->

| 符号 | 简称 | 含义 |
|---|---|---|
| T1 | 层 1 挂钟预算 | 层 1 本地挂钟预算，是输入不是输出 |

2. **`T1`（层 1 挂钟预算） 是输入，不是输出**（`.claude/rules/fs-design.md`：门禁运行时长本身是设计约束）。
   正确顺序是「先声明预算 → 用它约束一次提交能开几个 ignore 节点」，不是「量出耗时 → 推出能开多少」。
   **这一条不需要任何测量，现在就该定。**
3. **判别力必须与覆盖分开检查**：每个 ignore 节点被打开的那次提交里，额外对一份「已知坏的镜像」跑它自己的 checker，
   不报错即判红——这是 D13（验证路线）已有的阳性对照纪律的直接复用。见 C13（checker 判定失效）。

门禁里没有「层 1 崩溃点重放」这个阶段，量不出 `T1`（层 1 挂钟预算） / `C` / 每格耗时。
⚠️ **因此 `C`、`K_max`、`T1`（层 1 挂钟预算） 这三个数一个都不许现在写进 kb。**

**射程**：只管 ignore 的节点什么时候开；层 0 全量枚举不需要 `T1`（层 1 挂钟预算）。第 2 条的「现在就该定」与末句「一个都不许现在写进 kb」说反话，记在 [verification-build.md](../verification-build.md)「待定案」第 6 问，没判。节点登记表带状态字段（开着 / ignore / 等实现），另加三条——每个 ignore 行点名一个会变的对象、那个对象变绿而节点仍 ignore 时判红、覆盖报告列 ignore 的状态数与占比：这三条被攻过零轮（`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节），不写成定案，等第二、三轮再攻。

**依据**：
- 无实验：否决账本靠的是一次调度规则的模拟验算（200 轮、取后半程稳态最大值、严格前缀贪心，三行数在变更史），没有登记成实验；留下的三条是门禁设计原则，没有可量的量。
- 三方论证：层 1 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。
- 用户定案：「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 11 推翻，「逐格判决」一节给节点登记表带状态字段的形态与三条待攻的补充条件。

**欠**：C8（门禁范围判不出来）；C13（checker 判定失效）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:264-279`（整段抄，未转述）**

```markdown
#### 已定项 12：冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具`

**定案**：**冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具` 直接打架。**
在用的不变量（条数的权威登记位在 [invariants.md](../invariants.md) 文件头）若既是 checker 源码又是形式验证的 spec，就成了单点。
**可行的折中，而且本工程已经付过这笔钱**：checker 做两个方向——**遍历式**（从系统配置走下去）
与**扫描式**（全盘扫，靠「元数据块必须自描述」的块头重建），两者对同一条不变量独立判断，
共享的只有格式解析。

**射程**：管 checker 内部两个方向之间共享什么；checker 与实现之间共享什么归已定项 5。池级 checker 今天两个方向都有（`crates/singlefs-checker/src/walk.rs` 文件头）。

**依据**：
- 无实验：这是「checker 即规范」与校验路径独立之间的取舍，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：两个方向「共享的只有格式解析」这一条没有会红的检查（`.claude/gate.d/` 里没有判它的阶段），没有 C 编号。

```

**出处 `.claude/kb/decisions/13-验证路线.md:280-296`（整段抄，未转述）**

```markdown
#### 已定项 13：冲突 2：crash refinement 的 spec 与模型对拍的 model

**定案**：**冲突 2：crash refinement 的 spec 与模型对拍的 model 是同一个东西的两种写法。**
合并成一份 → 省一半工作量，但**直接违反独立性要求**；各写一份 → 名义独立，
但通常是同一个人同一周写的。缓解（只降低相关性、不消除）：让两份**在表述层面就不同**——
spec 写成「对镜像的状态谓词」，model 写成「op → 状态转移」，不一致时至少能互相定位。
**同源性要被观测**：统计「spec 与 model 不一致」的事件数及每次判定哪一方错；
若累计 ≥ 5 次不一致而判定「spec 错」的次数为 0，说明两者实际同源，应把两条腿合并、预算移给穷举。

**射程**：缓解只降低相关性、不消除。已定项 1 撤下的那条原候选不受这条影响，理由在已定项 1 的射程。crash refinement 今天没有代码（`grep -rln refinement crates research --include=*.rs` 零命中），不一致事件还没有来源。

**依据**：
- 无实验：同源性要靠将来 spec 与 model 的不一致事件计数观测，spec 与计数器都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C191（同源性计数器只盖了一对）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:297-315`（整段抄，未转述）**

```markdown
#### 已定项 14：与其他决策的连锁

**定案**：

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） | 根链差分把 Merkle 从「要付的代价」翻转成验证杠杆；但形式验证里哈希是 uninterpreted function，**碰撞抗性完全在验证范围之外，不许读成「验了 Merkle」** |
| D8（核心索引结构） | 日志结构节点、write buffer、key cache 三层都会绑架符号执行（循环上界、滞后窗口、缓存一致性），使「Yggdrasil 一分钟内」的口径在本工程不成立 |
| D9（加密） | AEAD 在 SMT 里不可解，只能建模成 uninterpreted function，**「nonce 重用泄漏明文」这个后果验不出来**，只剩「nonce 不重复」这条纯组合性质——而后者恰是这条腿最值钱的产出 |
| D11（索引节点要不要留消息缓冲区） | 已定项 1 取「留」；D8（核心索引结构） 已定项 1 取幂等完整值之后 checker 的 I-3.1（已分配统计对得上） 不必解析消息语义（D11（索引节点要不要留消息缓冲区） 据此判它的代价 1 在本工程不成立），冲突 1 不因此变成「无解」 |

**射程**：连锁的是形式验证与 checker 碰到这几条决策时的边界，不改那几条决策的条款；根链差分与冲突 1 本身归已定项 8、12。

**依据**：
- 无实验：连锁是从那几条决策的已定条款推出来的边界，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:316-327`（整段抄，未转述）**

```markdown
#### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑

**定案**：`crates/` 下的验证代码按「什么时候跑」分两档，各住一个包，与池级 checker 一共三个名字：**harness 档**是 `crates/singlefs-harness`，单元测试与集成测试加上它们用的脚手架库（录制器、内存池、理想模型、随机历史、故障注入设备、场景），改了代码随时跑；包内再分轻重：debug 下单条跑到 60 秒及以上的是重，标 `#[ignore]`，单条用时看平常跑的时候量到的、不另外单独量，要跑随时跑，一律经内存包装；一条用例一个场景，测试文件按测什么起名、不按里程碑起名。**checker 档**是 `crates/singlefs-checker-tier`，崩溃态枚举引擎与记录核对器、断点续跑与双机分片、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对、实验与真设备装置二进制，以及它们的全部用例，连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档里的崩溃点重放（门禁 checker-tier-layer0-replay）分两档，checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 不分：**快档**是 `cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（装置二进制的内联单测不在快档里）（门禁 checker-tier-layer0-replay 在整轮门禁与提交时都跑它），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（checker-tier-layer0-replay `--full`），提交时不默认跑，由用户要求或夜间跑，跑法按已定项 9 分层；GPU 只接校验和那一截（E163（GPU多卡算单元校验和））、要不要接归 D24（后台重活能不能卸给 GPU）。**池级 checker** 是 `crates/singlefs-checker`，判决器库，被两档调用。依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ④ 条判，dev-dependencies 也算）；两档都要的一小段代码宁可各留一份。崩溃枚举用例一律写在 `crates/singlefs-checker-tier/tests/` 里（`research/scripts/crash-case-check.py` 判），重型测试闸按包判：跑到 `singlefs-checker-tier` 的测试就是 checker 档。

**射程**：管 `crates/` 下代码与测试住哪个包、什么时候跑、闸按什么认、三个包怎么称呼（`.claude/rules/verification.md`「定义与名字」）；不管每条不变量判什么（[invariants.md](../invariants.md)）、不管枚举域怎么定（已定项 4）、不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`），上游那一批已同步进副本、未发版。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。

**依据**：
- 无实验：什么时候跑、住哪个包是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
- 用户定案（2026-09-27），原话在变更史；三方 `verification-split-r1`（判决 `research/prompts/verification-split-r1-main-verification.md`）攻的是拆出 `singlefs-checker-tier` 之前的那一版，拆包之后的改动被攻过零轮。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:1-13`（整段抄，未转述）**

```markdown
## D23 journal 的角色与格式 —— 已定

<!-- doc-lint:not-numbers CRC-64 SSE4.2 CRC32C SHA-256 CJ2 -->

D23（journal 的角色与格式） 管 journal 是什么、写什么、崩溃之后怎么用它：记录承载什么与分配的先后（已定项 20）、每次 fsync 写什么（已定项 1）、环的几何、记录尺寸与在飞上限（已定项 2 / 12 / 18 / 19）、tail（已定项 3 / 18）、记录头与点名项的字节（已定项 4 / 7 / 8 / 9 / 10 / 11 / 13 / 15 / 17 / 21）、前缀判定与重放下界（已定项 14 / 22）、管理员回退、失败处置与实例切换（已定项 14 / 16）、与 journal 相关的三个死锁和保留池（已定项 23 / 24 / 25）、记录头的解析纪律（已定项 26）。不管的：发布的持久顺序、checkpoint 的触发与暖机归 D16（发布语义）；实例表的行格式与写行规则归 D18（块里携带什么信息） 已定项 11；根记录与根环归 D22（单元原子性怎么合成）；保留池与切换预留怎么进准入不等式归 D28（挂载期承诺量）；checker 与记录核对器各是什么归 D13（验证路线） 已定项 6 / 7；目标负载归 D25（目标负载优先级）。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| T6 | 维持拒绝时拒不拒取决于池的历史 |
| L8 | 并行线三跨记录层0流 |

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:45-90`（整段抄，未转述）**

```markdown
#### 已定项 1：journal 的角色本身

**定案**：**取甲：每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录。祖先不延后。** 脏叶是全部脏叶，不只被 fsync 的那个文件的：fsync = 一次全量发布。

- **两根轴不正交**：轴一（每次 fsync 发不发根）取「发根」之后，轴二（祖先落盘延不延后）只剩一个合法值。**延后祖先 ⇒ 新根不可能自洽 ⇒ 发不出一个反映本次 fsync 的根。** COW 下，要让本次 fsync 的叶从根可达，整条脊柱必须已在盘上；而「自洽」是 [invariants.md](../invariants.md) I-7（根环） 类的要求（从代号最大的根递归校验能走完权威态）。2×2 只有三格可实现：

  | | 不延后祖先 | 延后祖先 |
  |---|---|---|
  | **每次 fsync 发根** | **甲** | **不存在** |
  | 只在 checkpoint 发根 | `wal_full` | 乙 / 丙 |

  ⇒ **轴一已定的「每次 fsync 发根」单独就把乙与丙一起判掉了**，且轴一那四条依据（环截断不了 / 多一条挂载期重放路径 / journal 进验证链 / 屏障一个没省）对乙**逐条原样成立**，不需要重新论证。丙（E22（记录点名祖先落点） 那个形态）同样不写祖先内容，只在轴一被重开时才有意义。
- **甲下 redo 为空**，不是「甲下恢复无动作」：恢复期仍有「续做未完成意图」这条常规路径（[checks-owed.md](../checks-owed.md) C1（意图续做做成常规路径））。
- **记录头的类型字段现在就是必需的，与轴二日后重不重开无关。** 类型字段的登记表（登记位 D22（单元原子性怎么合成） 已定项 17）：0 无效（判损坏）、1 普通记录；新类型只追加不重用码，未知类型按 A 条拒绝挂载。把它写成「为轴二留的期权」会让人以为付了钱、买到了轴二的可逆性——**它一分钱也没买到那个。**
- **真正让轴二可逆的是四项，缺一不可**：

  | # | 预留什么 | 为什么 |
  |---|---|---|
  | A | 记录头**类型字段** + **未知类型 ⇒ 拒绝挂载**，且该政策由**系统配置 incompat 位**承载 | 光有字段而政策是「跳过不认识的记录」，失败模式是**静默漏放一条发布指令**，比没有字段更糟 |
  | B | 记录**自述长度**，且**先边界检查再算校验和** | 类型字段只说「这是什么」，不说「这有多长」。丙的记录长度正比于树高，装不进按甲的尺寸钉死的预算 |
  | C | **journal 几何进系统配置**（环大小、最大记录字节、在飞记录数上限），**不许是代码常量** | 换轴二 ⇒ 最坏记录尺寸变 ⇒ 死锁 3 的几何不变量要跟着变。焊进代码就只能重建文件系统 |
  | D | 项的大小**由记录类型决定** | D9（加密）已定 MAC 满 128 位不截断，丙的项更宽 |

  ⚠️ **A 与 B 本来就欠着**（B 是已定项 26 的第二条），把它们记成「为轴二付的钱」是重复记账。
- **重开轴二的可判据。** ⚠️ **这是设计期判据，不是运行期开关。** `.claude/rules/fs-design.md` 硬要求 5「分支变量不许在操作中途改变」——把「延不延后」做成按负载切换的运行时分支，会让崩溃语义在一次操作中途改变，当场违规。**轴二只能整体切换，且走 incompat 位。**

<!-- doc-lint:registry name-col=2 -->

| 闸 | 简称 | 判什么 | 现在测得了吗 |
|---|---|---|---|
| G23.0 | 轴一被重开 | 轴一被重开（否则后面几条不必看） | — |
| G23.1 | 祖先链占比达标 | 单流同步写（QD=1、单线程）的 fsync 服务时间里，「写祖先链 + 根槽」占比 ≥ X%，**X 在实验注册时声明，不许事后定** | **不能**，E16（journal 的角色：WAL vs 意图日志） 口径明写只算块数不算时间。前置：事务层 + 真机 harness |
| G23.2 | 甲丙差不收敛 | 攒批变大时，甲与丙的差**不**收敛 | **「差」取甲 / 丙每批块数之比**（用户 2026-09-28 定；今天用乙的数代丙）：E16（journal 的角色：WAL vs 意图日志） 第五次跑显示，按今天的宽度收不收敛随树高变，不是「批 ≤ 流数」的界——真实基线 1 TiB（4 层、根下 11 个孩子）上流数 ≤ 16 先涨后收敛（峰值在批 ≈ 流数），流数 ≥ 24 从批 = 1 起就收敛（`research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out` 的 `row_summary` 行，流数 64 那一行第 1818、1840 行：批 1 到批 64 比值从 3.500000 降到 3.169231）；树高 ≤ 3 全部流数从批 = 1 起收敛，树高 ≥ 5 扫到的流数全部先涨后收敛，树高 4 时门槛约为 2 × 根下孩子数（只在三方攻方副本上量过，主 agent 复跑逐字节相同，入库装置没重做，判决 `research/prompts/d23-e16r5-r1-main-verification.md` 第一节汇总表）。「3.40× → 3.71×」那组数是旧宽度（扇出恒 128、树高恒 4）下的。新写法被攻过零轮。⚠️ 丙尚未测 |
| G23.3 | 四项盘上齐备 | 第 A–D 四项在盘上齐备 | **能，现在就该定** |

- **反向判据（D23（journal 的角色与格式）必须自带）**：若记录核对器（D13（验证路线） 已定项 7：需要第二个输入的核对归它）上线后，累计 N 次全量验证里「记录核对器抓到而 checker + O1（内存模型对拍（HashMap 理想文件系统）） + 崩溃点重放三者都没抓到」的事件数仍为 **0**（N 在上线时先声明），则「乙毁掉记录核对器的逐字节比对」这条依据的增量价值实测为零，轴二应当重开评估。没有这一条，那条依据就是不可证伪的。

**射程**：管每次 fsync 写什么、轴二能不能重开；目标负载归 D25（目标负载优先级）。定案只压在「两根轴不正交」这一条结构性依据上，与收益多少无关；定案当时给出的三条依据都降了级、不承重：「乙毁掉记录核对器的逐字节比对」暂不承重，「乙买不到『用错了变成挂不上』」射程用错撤下，「乙的收益形状」只剩奖金上界。乙的收益只在单流同步写上存在，而那恰是数据库与虚拟机镜像（D10（随机写 / 碎片化的真答案） 记为 COW 文件系统的共同软肋）⇒ 只能说「乙的奖金有界」，不能说「乙的收益不值一提」。轴一的写放大前提在里程碑「覆盖写、释放、回退与复用」增补 1 的三方第一轮被打中，第一轮那处打中在第二轮被翻掉（记录头里 188 字节的新根段今天已经在，施加就是换四个字段，所以「wal_full 自己改不掉」只对「每次 fsync 另写一个树表单元」那一形成立），两轮判完的结论是三条岔路收成甲、正文不动（`research/prompts/m2-s1-r1-main-verification.md`、`m2-s1-r2-main-verification.md`）。

**依据**：

- E16（journal 的角色：WAL vs 意图日志）：延后祖先的臂只在 checkpoint 发根、从没在 fsync 时发过根；攒批吃不掉延后祖先的收益随树高变（第五次跑：树高 ≤ 3 全部流数从批 = 1 起收敛，树高 4 门槛约为 2 × 根下孩子数，树高 ≥ 5 扫到的流数全部先涨后收敛）——它改的是「乙的收益形状」那条已降级依据的强度，不是结论。
- 三方：2026-08-28 三方论证（材料 `_axis2-background.md`），正推腿给出「轴一 ⇒ 轴二」这条结构性依据，三条原依据怎么降级、腿况在变更史。

**欠**：C1（意图续做做成常规路径）；反向判据的 N 没有声明，记录核对器已接在层 0 上（`crates/singlefs-checker-tier/src/crash.rs` 的 `check_records`），事件数没人计。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:91-115`（整段抄，未转述）**

```markdown
#### 已定项 2：journal 是定长环还是链式

**定案**：**取定长环。** ⚠️ 候选理由「链式把 journal 写路径变成分配路径，于是进入死锁 2」过强，不采用：链式的增量是有界可算的，抬保留池就断开——它不是一个真环。真正判掉链式的是三条，都不经过死锁 2：

1. **保留池必须涨一个量级，而涨保留池会加重假性 ENOSPC。** 链式下 journal 块本身要分配 ⇒ 保留池必须从 `ckpt_cost` 涨到 `ckpt_cost + 一个 checkpoint 窗口内的 journal 块`；而保留池对普通分配是纯税，假性 ENOSPC 只有准入治得了（已定项 24）⇒ **链式不是把代价变成死锁，是把它变成假性 ENOSPC。**
2. **前缀判定与空间归属被绑死。** 定长环下「有效前缀到哪」只决定重放哪些记录；链式下还决定**哪些块分配器可以发出去**——前缀规则与 D21（权威态与派生态的分界）的记账权威是两个独立判定，却必须逐块一致，不一致就是静默泄漏或双分配；断号即止的截断点之后的链块要么泄漏、要么被重发而重放仍指着它。
3. **死锁 3 不是被消掉，是从静态判据换成动态判据——这条是决定性的。**

   | | 判据 | 什么时候算得出 |
   |---|---|---|
   | 定长环 | 最坏操作 ≤ 环大小 / F，F ≥ 2 | **mkfs 时算、mount 时校验**（同 XFS `XFS_MIN_LOG_FACTOR`） |
   | 链式 | 最坏操作 ≤ **当前空闲** | 算不出——**当前空闲在操作中途会变** |

   `.claude/rules/fs-design.md` 硬要求 5 逐字写着：「**只在操作期间不会变的输入上分支**……「当前填充率」会在操作中途越过阈值，**那是危险的分支变量**」。⇒ **链式把一条可判定的几何不变量，换成了本工程明令不许用的运行期变量。** 这不是权衡，是撞上已定的硬要求。

**射程**：管 journal 的空间形态。与已定项 3 耦合：定长环下 tail 答错**只是多做功**（可降级），链式下是**泄漏或双分配**，tail 必须是权威态、要 tail-before-reuse 硬序、FUA 落在关键路径；**内联 tail + 链式是最坏格**（内联只能搭记录便车推进，链式下空闲期不写记录 ⇒ journal 块一直回收不了，空转变成空间泄漏）。⚠️ 环这一侧的几何不变量已写成 [invariants.md](../invariants.md) 的 I-8.1（环几何够大），**而它的 checker 未实现**——「写进清单」不等于「有东西在拦」。

**依据**：

- E25（环与链各要多大保留池）：checkpoint 卡死恰好在预测的那一档归零——环在 reserve = ckpt_cost，链在 ckpt_cost 加一个窗口的 journal 块。
- E23（journal 几何）：环与链只差一项——链式每个 journal 块都要向分配器要一次块，环 0 次；链式没有死锁 3、环要几何不变量。
- 三方：2026-08-29 三条腿一致判「链式进入死锁 2」过强（材料 `_d23-background.md`）。

**欠**：C28（环几何不变量未实现）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:116-147`（整段抄，未转述）**

```markdown
#### 已定项 3：tail 住系统配置槽，改 jbd2 两处

**定案**：**取系统配置形态**（tail 的权威副本住在一个固定位置，不内联在记录头），**并改两处**：

| # | 改动 | 为什么 |
|---|---|---|
| 1 | **不许单副本原地覆写**，改 ≥2 槽轮换 + 代号 + 先逐个验证全部候选再择新 | D22（单元原子性怎么合成）已对根定过同一条纪律：一次撕裂会抹掉唯一见证者 |
| 2 | **随每次发布写一次，是发布序列的最后一步** | D16（发布语义） 已定项 7 的注：根槽持久之后再更新系统配置槽、进层 0 枚举；D16（发布语义） 已定项 6 逐字「每次发布把 `checkpoint_txg` 加一——**fsync 触发的发布也是发布**」⇒ 「每个 checkpoint 一次」与「每次发布一次」是同一个频率，fsync 触发的发布同样要写系统配置槽 |

- 原地覆写与 COW 不冲突：tail 与根槽、journal 记录头同属自证单元（发根本身就是覆写根环的一个槽），归 D22（单元原子性怎么合成）已定项 21 管；COW 对没有父的单元买不到东西。**tail 语义上是派生态**：权威判定仍是 `jsn` 连续前缀 + `checkpoint_txg`。
- **由此逼出三条硬要求**：
  1. **恢复不许先信 tail 再往前走：必须先全环扫描、逐条验证，再择最长合法前缀。** tail 陈旧时，`[tail_陈旧, tail_真实)` 这段已经被新一圈记录写掉了 ⇒ 从陈旧 tail 起往前走，**第一条就 `jsn != expected`** ⇒ 断号即止 ⇒ 连 `tail_真实` 之后仍然需要的记录也不重放。**那是数据丢失方向，不是「多做功」方向。** ⇒ 「陈旧 tail 只让恢复多做功」这句话**只在全环扫描式恢复下成立**。与 D22（单元原子性怎么合成）对根槽的「先逐个验证全部候选、再择新」是同一条纪律。
  2. **重放必须幂等**：

     > **每条 journal 记录的重放必须幂等**——同一条记录被重放两次与重放一次，结果相同。

     「陈旧 tail 只是多做功」这句话的前提是它：没有幂等，陈旧 tail 就从「多做功」变成「重复施加已生效的记录」。与已定项 20「记录承载指针层的**目标态**」相容：目标态天然幂等（把指针写成某个值，写两次结果一样），逻辑意图不是（「计数加一」执行两次结果不同）。
  3. **恢复算法救不了几何**：窗口撑爆环时两种恢复一起丢；环大小 ≥ 窗口是 [checks-owed.md](../checks-owed.md) C28（环几何不变量未实现）的职责，**不是恢复算法的**，两者不许互相顶替。
- 块重用叠上来之后「陈旧 tail 只是多做功」不成立，恢复需要重放下界，取根的水位（已定项 14）；这对已定项 3 是收窄不是推翻：「先全环扫描、逐条验证、不许先信 tail」原样成立，变的是验证出的合法前缀里哪一段被施加。

**射程**：管 tail 住哪、多久写一次、恢复怎么用它。字节表 [layout/01-first-txn.md](../layout/01-first-txn.md) 零与一两节按「每次发布一次」写：两次暖机空发布与新池新建文件各推进一次 tail。⚠️ 取系统配置形态时用过的代价表（E23（journal 几何） 的「两条臂差 21 : 500」）已作废：第一版每次发布都是 checkpoint，稳态多写从 21 次变成每次发布两盘各一槽；重放那一维今天不存在（恢复不读 tail、每次挂载全环扫描）。「每次发布轮换系统配置槽、返回前等它持久」今天的依据是判据 N-配置（已定项 14「可写挂载读到的更新状态读不出」，C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙、C577（系统配置没见证到的最新根，乙罩不到））：只减轮换次数或不等它持久就返回，「已确认的发布 + 根槽一时读不出 + 一份点名单元坏」时丢已确认的发布，而且那一格与「根槽 FUA 之前崩 + 一份单元坏」盘面逐字节相同、改读者分不开。这条依据成立在那一类双故障在容错射程里（用户 2026-09-28 定在）；落后支（D18（块里携带什么信息） 已定项 11）读 tail 不是这条依据。

**依据**：

- E24（恢复算法：先信 tail 会不会丢记录）：先信 tail 漏掉的条数恰等于窗口内写入的条数；全环扫描不丢，但把陈旧 tail 以来的记录全部重放一遍（逼出幂等）；窗口撑爆环时两条臂一起丢（恢复救不了几何）。
- E23（journal 几何）：jbd2 形态与 XFS 内联形态在稳态额外写与空闲崩溃后的重放量上方向相反——按的是 tail 每 checkpoint 推进一次的旧前提，见射程。
- 三方：2026-08-29（材料 `_d23-background.md`）。
- 三方判决 `research/prompts/c245-r3-main-verification.md`（C245（系统配置槽的写频率，两条条款说反话） 第三轮）：每次发布轮换的依据换成判据 N-配置；攻方探针量出乙、丙在「已确认的发布 + 根槽一时读不出 + 一份点名单元坏」上丢已确认的发布、甲要四处故障才丢。
- 用户定案 2026-09-28：那一类双故障在容错射程里（原话在变更史）。

**欠**：C28（环几何不变量未实现）；C77（重放起点未定义）；C594（每次轮换之前现算整池 F 的读开销没量）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:148-175`（整段抄，未转述）**

```markdown
#### 已定项 4：记录头约束在单个原子单元内，头 311 字节

**定案**：**记录头完整落在一个原子单元内**（[invariants.md](../invariants.md) I-8.2（记录头不跨原子单元））。头 **311 字节**：

- **十个字段 78 字节**：magic 4 + 类型 2 + 算法类型 1 + 记录标志 1（位 0 = 本次发布末条，已定项 17；其余位写 0，读到非 0 当损坏） + 自述长度 4 + 点名项数 4 + `jsn` **10** + `checkpoint_txg` 8 + nonce 12 + 头部校验和 32。`jsn` 的 10 字节 = 实例代号 32 位 + 计数器 48 位（已定项 9）。`tail_lsn` 不在头里：已定项 3 逐字定了「tail 的权威副本住在一个固定位置，**不内联在记录头**」，而「内联在每条记录头的 `tail_lsn`」正是被否掉的那个 XFS 形态的定义。
- **三笔已定增量 17 字节，不在那十个字段的 78 里**：已定项 7 的事务号 + 提交标记 **9 字节**、已定项 8 的反向链 **4 字节**、已定项 13 的载荷校验和 **4 字节**。
- **本次发布内序号 4 字节**（无符号 32 位，从 1 起：一次发布切成 N 条记录时依次是 1..N，只有一条时是 1；空发布记录也写 1）：已定项 14 注 1 在所选根那条记录读不出时靠它接链首。4 字节的宽度与紧跟事务号、提交标记的落点是主 agent 按「不为省空间牺牲自包含」取的，可推翻。字段在头里的偏移：事务号 78、提交标记 86、本次发布内序号 87、反向链 91、载荷校验和 95、新根段 99、fsid 287、MAC 295、头末 311。一次发布的 N 条记录序号依次 1..N、与 jsn 同步；读者遇到这四格，都当那条记录损坏、断链即止，那次发布整体不施加：序号 0；一次发布之内跳号；锚点读得出时，下一次发布的首条序号不是 1（断在这一条）；同一 (实例代号, checkpoint_txg) 里带末条标志的那条之后还有记录（断在带标志的那条）。与 I-8.9（一次发布的记录序号连续且只有末条带标志） 判的是同一件事。
- **已定项 15 的新根段 188 字节**（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8）。
- **fsid 8 与 MAC 16，都排在新根段之后**：
  - **fsid 8**（系统配置 fsid 的低 8 字节，与单元头那 8 字节同口径，D18（块里携带什么信息） 已定项 7）。不带的话 I-1.4（块头 fsid 一致） 要给 journal 记录写一条豁免，而全环扫描就再也挡不住「上一个文件系统留在这块盘上的记录」——mkfs 整环写 0 只在 mkfs 真跑完时成立，重 mkfs 中途崩掉就不成立。带 8 字节之后不用豁免。
  - **MAC 16**（放在 fsid 之后；nonce 12 与算法类型 1 十个字段里已有）。理由与根记录那 29 字节同一条（D22（单元原子性怎么合成） 已定项 7）：journal 记录头是自证结构，没有父指针带 MAC ⇒ 不 day-1 留位，开加密那天要么整条走明文豁免（泄漏 `checkpoint_txg`、新根段里的两条指针、点名项位置），要么就是一次布局变更；D9（加密） 已定项 3 的原则逐字是「为加密留的位所有卷都留、不挪作他用」。I-6（加密不变量）不列它。
- 登记值就是 311（`JOURNAL_HEADER_BYTES`）；十个字段的 78 另登记为 `JOURNAL_HEADER_TEN_FIELD_BYTES`。**说 78 时要写明指的是十个字段，连同这一句一起引。** 311 占 512 扇区的 61%，其后还余 **201 字节**（装得下 3 个点名项，每项 56 字节）；4096 上余 **3785**（67 个点名项，⌊(4096 − 311) ÷ 56⌋ = ⌊3785 / 56⌋）。
   <!-- format-const: JOURNAL_HEADER_BYTES = 311 stale=头 277 字节|的 277 字节头|合计 **277 字节**|JOURNAL_HEADER_BYTES: u64 = 277|JOURNAL_HEADER_BYTES: u64 = 84|JOURNAL_HEADER_BYTES: u64 = 86|JOURNAL_HEADER_BYTES: u64 = 78|512), 428|512), 426|512), 434|4096), 4012|4096), 4010|4096), 4018|512 上 428|512 上 426|头 86 字节|头 **307 字节**|登记值就是 307|⌊3789 / 56⌋ -->
   <!-- format-const: JOURNAL_HEADER_TEN_FIELD_BYTES = 78 -->

**射程**：⚠️ **那个原子宽度不由本工程定**：D20（承重面：单元的原子性与自包含）已定自证单元依赖**运行时探测**的 `physical_block_size`，不许把 512 焊进格式。⚠️ **这个值是格式常量，改它要三处一起动**：kb 标记与正文、实验源码的 `const` 与钉死它的单测、重跑实验并更新产物；由门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/code-source-discipline.sh` 的 format-constants 格）判红。记录尺寸取 4 KiB 定长之后对齐浪费恒为 0（已定项 12）；变长记录的对齐代价表只对变长记录成立。

**依据**：

- E23（journal 几何）：十个字段（跑时还带 `tail_lsn`）的头装得进一个 512 单元，不对齐、多条记录挤同一个原子单元时除第一条外都判不了撕裂——头要完整落在一个原子单元内。
- E75（记录尺寸与环几何）：各个头部读法在 512 与 4096 两档 `physical_block_size` 上都装得进一个单元；定长 2 的幂尺寸下头永远不跨单元。
- fsid 8 与 MAC 16：用户弹窗定案，没有走三方论证，可推翻，原话在变更史。
- 本次发布内序号 4 字节：用户 2026-09-23 定（`research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 的 Z6），是推的、被攻过零轮，原话在变更史。
- 用户定案 2026-09-24：头 311 落地、末条标志放在原「填充 1」那个字节、序号的读者规则（原话在变更史）。
- 用户定案 2026-09-24：锚点读得出时首条序号不是 1、末条标志之后还有同 txg 的记录，两格都当损坏断链（实二十报告 `research/prompts/m2-lastflag-implementer-report.md` 第六节 2、3），原话在变更史；无实验：两格都要 CRC-32C 碰巧对上或改出来的镜像才走得到。

**欠**：C94（登记的格式常量与后来的定案对不上）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:176-191`（整段抄，未转述）**

```markdown
#### 已定项 5：O2（独立解析器 + checker） 是一元谓词，逐字节比对归记录核对器

**定案**：**O2（独立解析器 + checker） 是一元谓词，只判单个镜像**，不是「逐字节比对」——那是二元关系，需要第二个入参；需要第二个输入的核对归**记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像)`，权威记录在 D13（验证路线） 已定项 7。

- ⇒ D23（journal 的角色与格式）否掉逻辑意图日志的第二条理由（「落点不确定使核对退化」）按字面是空的，**改写为「核对从句法变成语义」，且它不是独立的第二条理由，是第一条（重放要第二个实现）的一条封堵**：逻辑意图记录不点名任何盘上对象 ⇒ 核对方要自行实现树语义 + 记账 + 分配合法性 ⇒ 第二实现的成本原样搬到核对侧。
- ⇒ **「否掉逻辑意图日志」这个结论不动**，第一条理由独立成立（已定项 20）。
- ⇒ D23（journal 的角色与格式）的反向判据主语是**记录核对器**（已定项 1；主语写成 O2（独立解析器 + checker） 时按字面恒为 0——O2（独立解析器 + checker） 就是 checker——不可证伪）。

**射程**：只管 D23（journal 的角色与格式） 怎么称呼这两个验证角色、它们各否掉什么；两者是什么、入参是什么、代码在哪，权威在 D13（验证路线） 已定项 6 / 7。

**依据**：

- 无实验：一元谓词不能变成二元关系，这是类型之争，没有可量的量；2026-08-29 三方论证（材料 `_o2-background.md`），逐条依据见 D13（验证路线） 已定项 7。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:192-203`（整段抄，未转述）**

```markdown
#### 已定项 6：目标负载优先级移出 journal 这条线

**定案**：目标负载优先级移出 D23（journal 的角色与格式），见 D25（目标负载优先级）。它跨 D8（核心索引结构） / D10（随机写 / 碎片化的真答案） / D23（journal 的角色与格式），不是 journal 的问题；留在这里会被当成 journal 细节。**它不阻塞 D23（journal 的角色与格式） 任何一项**：轴二是被轴一结构性排除的，目标负载只影响已定项 1 里「乙的收益形状」那条已降级依据的强度，不影响结论。

**射程**：只是一条指针，定了什么看 D25（目标负载优先级）。

**依据**：

- 无实验：这是一次挪位，理由是它跨三条决策、不是 journal 的问题，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:204-223`（整段抄，未转述）**

```markdown
#### 已定项 7：一个事务可以跨多条记录，记录头带事务边界字段

**定案**：**可以跨多条，记录头带事务边界字段**：事务号 8 + 提交标记 1 共 9 字节（已定项 4 的三笔增量之一；与已定项 8 的反向链是**不同的字段**，不许合并计价）。

- 事务号按实例计数、从 1 起，同一事务的全部记录共享它，与 jsn 是两个计数；单元头写序存它的低 48 位、高 16 位恒 0，不新开字段（C113（扫描重建时多版单元的现行版本判定无输入） 定案复用这个字段）。事务号 0 留给不承载事务的空发布记录（已定项 19 ①）。
- 记录按事务号顺序追加，一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加——这是实例表行的 W 能当精确前缀的依据（第一版串行提交下 W 的前缀性本来就成立，D13（验证路线） 已定项 2；这条纪律买的是并行提交打开之后它仍然成立）。
- **恢复算法**：在「jsn 严格连续 + 校验和 + 反向链」之后**再加一步**——**丢掉提交标记还没出现的那个事务的全部记录**。⚠️ **它不能靠反向链代劳**：被丢的尾巴在链上是**连续**的（它们本来就是顺序写的），链检查一条都拦不住。**两条闸管的是两件事**：链管「是不是同一条时间线」，提交标记管「这个事务写完了没有」。
- ⚠️ **丢尾巴是常态不是边角**：前缀落在事务中间的是绝大多数崩溃点 ⇒ 恢复路径里那一步会被绝大多数崩溃点走到，它必须和主路径一样被崩溃点重放覆盖，不许当异常分支。
- 判据已写成 I-8.8（前缀里的事务不被切开）（单张镜像那一半）与层 0 崩溃点重放（整数个事务那一半）；单张镜像那一半池级 checker 已判（`walk::check_pool_image` 的 `judge_commit_markers_per_transaction`），层 0 那一半仍欠，记在 C37（事务与记录的对应无定义）。

**射程**：管事务与记录的对应；记录与时间线的对应是已定项 8，两条不是同一条——已定项 7 取任一答案，时间线那一格照样漏。⚠️ 它不是 D13（验证路线） 已定项 2（串行 / 并行）的推论——串行下同样会切开。I-8.2（记录头不跨原子单元） 只约束**记录头**，没有任何规则禁止一条大记录；一个事务写几个单元的用户数据归 D16（发布语义） 已定项 5 末段的切分纪律，它与记录数口径的冲突归 C310（事务切分纪律与记录数口径打架）。「一事务一条」那一侧还有两条没建模的代价：环回绕要连续空间、整条记录要先在内存里缓冲齐才能开始写。

**依据**：

- E42（一事务几条记录）：一事务一条与「跨多条 + 事务号 + 提交标记」都满足 I-4.3（提交原子），跨多条而没有边界字段时非法前缀恰等于被切开的前缀数、占绝大多数崩溃点——实验分不出对错，只分得出代价落在哪。
- E45（事务跨多条记录的环占用）：它量出的「跨多条把环顶大 12 倍」是一条比 I-8.2（记录头不跨原子单元） 严的对齐规则造出来的，按字面倍数是 1.00×，拿它撤回已定项 7 依据的那一步不成立（实验结论作废）。
- 用户定案「跨多条 + 边界字段」，理由「它严格更一般」，攻击轮复核维持；定案带来的三条义务（格式、恢复算法、不变量）用户口径是「算法要增强」，原话与经过在变更史。

**欠**：C37（事务与记录的对应无定义）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:224-244`（整段抄，未转述）**

```markdown
#### 已定项 8：记录头带一条 32 位反向链

**定案**：**记录头加一条反向链（前一条记录的 hash），宽度 32 位。** 覆盖哪些字段是已定项 10，hash 算法是已定项 11（完整 32 位 CRC32C），写路径上的取值口径是已定项 19 ②。

- **它补的缺口**：D23（journal 的角色与格式）三条已定项（定长环 / 断号即止 / 全环扫描）**各自成立，凑在一起漏**：定长环的槽位由 `jsn` 决定 ⇒ 断号之后的残留记录不会消失、只会等着被覆盖；而恢复之后从「前缀末 + 1」接着写，**那正好接在残留序列的前面**（已定项 22）。
- **同一个字段两个职责，两个都记**：分辨时间线；以及让记录核对器能用**与正向扫描完全不同的遍历**复核同一个前缀边界（正合 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验的两条路径不许共享同一段代码」）。只按验证独立性记，它会在「为了省字节」的权衡里第一个被砍掉。
- **一旦 hash 覆盖了实例代号（已定项 10），反向链对「分辨时间线」这个职责就是冗余的**——实例代号那条出路是精确的，不是概率的；那时链剩下的职责是遍历独立性。
- ⚠️ **一次误接受没有第二道防线**：I-8.3（重放前缀严格连续）、在飞记录数上限、`checkpoint_txg` 三条闸都不触发（两条时间线在同一个 checkpoint 窗口内，那个字段一模一样）⇒ 后果是**把一条已丢弃时间线的指针层目标态装进现行状态**，落在 I-7.4（近 K 代块未被复用） 保护窗口内时它完全合法、静默为旧数据。

**射程**：⚠️ **误接受的分母是「空洞 → 恢复 → 续写 → 再崩」这个事故序列发生的次数，不是时间**；取的场景点（每天崩 10 次、十年）不是测量，本仓还没有崩溃频率遥测。链宽不随文件变大而需要变宽：一次事故里链只在一个接缝上被考验，随文件变大受压的是序号计数器（已定项 9）与环的几何（I-8.1（环几何够大））。⚠️ 接缝数口径压在「一次事故一个接缝」这条没建模的假设上，一次恢复若留下多个断点，分母跟着变 ⇒ **32 位与已定项 11 的 CRC32C 是同一个单点**，那个口径一倒两条一起倒，重开条件对两项同时生效（已定项 11 重开条件第 1 条）。⚠️ 代价侧（2 / 4 / 8 字节链在盘上多不多占单元，E49（反向链宽度 32 还是 64） 扫过六档头宽）只取决于记录头总字节 mod 8，**是偶然不是结构**，头宽再动就要重算；现行不含反向链的头是 307，不在扫过的档位里 ⇒ **重算之前不许把「选 16 不省东西」「64 位也免费」「两侧都不分岔」当结论引用。** ⚠️ XFS 的 `h_prev_block` 只在写入侧赋值、恢复侧不比对它，引它时只能引「字段形态」，不能引「现役这么防」。

**依据**：

- E32（上一条时间线的残留）：定长环 + 断号即止 + 从前缀末接着写，残留记录会被下一次恢复接上去重放；两条自然实现各违反一条判据、方向相反——缺口在规范里，不在实现里。
- E36（槽位映射那一维）：槽位与序号解耦救不了，新旧记录等长时与按序号那一支逐格相同，只在记录长度恰好不同时碰巧挡住。
- E39（反向链挡不挡得住残留记录）：开链的全部格子残留为 0 且一条自己的记录都没误杀；误接受率逐档贴住 2⁻ⁿ。
- E51（反向链的碰撞机会有多少次）：碰撞机会的正确分母是接缝数（同源记录的链值构造性相等），按它 32 位全部档位够用；把每一次相邻比较都算一次机会的遍历分母是过计。
- 用户定案：加反向链「字节代价小，而它同时买到两样」；宽度「如果 32 位够用 那么就 32」，原话在变更史；三方 2026-08-30 三条腿一致指向 32 位。

**欠**：C42（残留记录冒充合法前缀）；C200（记录头改宽之后代价侧没重算）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:245-263`（整段抄，未转述）**

```markdown
#### 已定项 9：jsn = 实例代号 32 位 + 计数器 48 位

**定案**：**实例代号 32 位 + 计数器 48 位，两个字段共 10 字节，不挤进一个 64 位。** 分开还多买一样：**比大小不必先解位域**，恢复路径少一处可以写错的移位。

- 短板在代号那一格：代号与 fsync 率无关，每天崩 10 次、十年 = 36 500 次，16 位容量 65 535 只有 1.8 倍余量 ⇒ 加宽到 32 位（容量 4.29×10⁹，用不完）；计数器 48 位不加宽。
- 计数器全池接着走、不按实例归零（已定项 14 第 3 条），48 位的寿命按全卷算。
- ⚠️ **随之而来的一条义务：代号不许回滚，且要有人保证。** 代号一回滚，这条出路逐格退回没有代号时的样子（残留全数重放）。**这与 D9（加密） 已定项 8 对 nonce 水位的要求同型**——那里逐字要求水位「不许住在裸明文系统配置里……必须落在一个被 MAC 覆盖的字段上」。
- **代号住哪儿、谁保护它**：住 journal 记录里是循环（恢复要先读记录才知道代号，而读记录正需要代号来判）；住系统配置则落进「不许回滚」那个要求 ⇒ 与 D9（加密） 已定项 8 的 nonce 水位**同一个载体问题**，两者应当一起定，不许各定各的。今天的做法：取号取 max(系统配置, 根环) + 1、写进每一份系统配置（已定项 16）。

**射程**：⚠️ **敲定的两个数各有一处未测**：① 计数器那一档只在本机测过，**本机盘没有掉电保护（PLP）**，带 PLP 的企业盘 fsync 率可以高一到两个数量级，那一档本机测不到，仍是假设；本机测的还是 ext4 的 fsync 路径，是本工程 fsync 率的**上界代理**。② 代号那一档的「每天崩 10 次」是**取的场景点，不是测量**。⇒ **重开条件可判定**：拿到现场 fsync 率且它超过 10⁵/秒，计数器那一半重开。已定项 14 之后前缀规则不改：重放从所选根覆盖的最后一条记录之后接，所选根是最新实例的根时新记录都在同一实例里、严格连续；只有所选根退到更早实例时才碰到实例边界，那一格按严格前缀停，代价写在 D16（发布语义） 已定项 7 的注，要不要补是 D16（发布语义） 已定项 8。

**依据**：

- E44（序号位宽的本机实测与代价）：本机 fsync 率下 48 位计数器远超需要，「48 位不够」那条推理的前提在本机不成立；`jsn` 加宽到 10 / 12 字节在 512 与 4096 两种单元下点名项容量不变、一格都不多占单元。
- E37（日志实例代号）：带实例代号的那一臂两条判据同时满足，前提是代号不许回滚——回滚一格就逐格退回残留全数重放。
- 用户定案，原话在变更史。

**欠**：C199（实例代号递增与 jsn 断号即止互相矛盾）；代号的载体与防回滚保护没有编号，挂在 D9（加密） 已定项 8 的 nonce 水位同一个载体问题上。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:264-280`（整段抄，未转述）**

```markdown
#### 已定项 10：`previous_hash` 覆盖前一条记录的整个记录头

**定案**：**覆盖前一条记录的整个记录头，其中含 `jsn`。** `jsn` 是「实例代号 32 位 + 计数器 48 位」（已定项 9）⇒ **实例代号在链值的输入里** ⇒ 两条时间线的链值构造性不同，误接受率回到 2⁻ⁿ 那条曲线上，32 位够用的判断因此成立。

- ⚠️ **取「整个记录头」而不是「只取 jsn」的理由**：多覆盖的字段**一个字节都不多花**（hash 的输入长度不进盘），而少覆盖一个字段就多一条「这个字段被改了链还对得上」的缝。
- ⚠️ **一条实现纪律**：`header_csum` 那 32 字节**自己不能进 hash 输入**（它是头的一部分而又依赖头）——写死为按零参与（已定项 19 ②）。
- **链值必须有一个「两条时间线保证不同」的输入**，否则误接受率是 **100%，与位宽无关**：若 `previous_hash` 取前一条的 `header_csum`，两条竞争记录的 magic / 类型 / `jsn`（都等于 expected）/ `checkpoint_txg`（同一窗口内一模一样）在点名项数相同、加密关闭时**构造性相等**。它与已定项 8 的宽度定案是两件事：宽度答「碰撞概率多大」，已定项 10 答「有没有东西可碰」。

**射程**：管 hash 的输入；函数是已定项 11，宽度是已定项 8，本实例第一条恒 0、写路径不读盘是已定项 19 ②。

**依据**：

- E37（日志实例代号）：带实例代号的恢复在残留与新写两条判据上都是 0，精确而不是概率——把实例代号放进链值的输入，链就有了「两条时间线保证不同」的那个字段。
- 用户定案，原话在变更史；三方 2026-08-30 反推腿读实验源码时指出链值的输入里没有标识时间线的字段，经过在变更史。

**欠**：C42（残留记录冒充合法前缀）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:281-310`（整段抄，未转述）**

```markdown
#### 已定项 11：反向链的 hash 取完整 32 位 CRC32C

**定案**：**取完整 32 位 CRC32C**，与 D19（块指针的结构与宽度预算） 已定项 2 已定的密文校验和同一个函数。

- **主依据是「这个选择不承重」，不是「CRC32C 更好」**：32 位已判够用，且余量是**三个数量级** ⇒ 在 2⁻³² 这条曲线上，两个候选（完整 CRC32C、32 位截断的密码学 hash）**都够用**，选谁不改变风险量级；即使真损失 1 整比特（碰撞率翻倍），余量仍够。
- **在「都够用」之上，按两条次要理由选 CRC32C**：

  | 理由 | 它买到什么 |
  |---|---|
  | 与 D19（块指针的结构与宽度预算） 已定项 2 同函数 | 少一份实现、少一份 checker、少一类崩溃点重放 |
  | 有 SSE4.2 的 `crc32` 指令 | 提交路径上少一次密码学计算。⚠️ **本工程一个周期都没测**，这条只作次要理由，不许当依据 |

- **取「完整」不取「截断」**：截断的 CRC 不是短一点的 CRC，若链值取截断 CRC，2⁻ⁿ 那条曲线不适用。
- ⚠️ **一条已撤回的理由**：「CRC 线性 ⇒ 差异落在 ≤32 比特窗口内时碰撞可证为 0，而截断的密码学 hash 只能给不可证的 2⁻³²」。**它不成立**：那个「窗口受限」的前提是被模型硬编码出来的——两条竞争记录的 `named_count` / `record_length` / `nonce` 被建成构造性相同，而它们是**两个不同的事务**，且 I-6.1（nonce 不重用） 要求 nonce 全盘唯一。
- ⚠️ **由此还得到一条要记的反向风险**：**窗口条件不成立时，线性臂的尾部比密码学 hash 差。** CRC 的碰撞差异集是一个线性子空间——同一个差异模式**每次都碰**；截断的密码学 hash 是每对独立的 2⁻³²。在现有余量下这不改变结论，**但它是唯一一条指向另一边的证据，不许在引用定案时略过。**
- **重开条件（两条，任一成立即重开）**：
  1. 接缝数那个分母口径被推翻，余量掉到 10 倍以内；
  2. 已定项 10 的覆盖范围变化，使「两条时间线保证不同」这个前件不再成立——那时误接受率 100%，与函数和位宽都无关。

**射程**：只管反向链的函数；`header_csum` 的算法（CRC32C 4 字节 + 28 字节零）归 D18（块里携带什么信息） 已定项 17，`payload_csum` 的算法在已定项 13。

**依据**：

- E51（反向链的碰撞机会有多少次）：32 位按接缝数的分母全部档位够用，余量三个数量级——函数选择不承重。
- E61（反向链 hash 算法的均匀性）：量三只排除得掉「有效位宽损失 ≥ 1 整比特」，两个候选都在其内；量二作废，连带撤回了「线性可证为 0」那条理由。
- E40（密文校验和位宽）：截断的 CRC 把「单比特翻转必检」这条结构性质整个丢掉——取完整、不取截断。
- 用户定案（改正后的量二不跑），原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:311-328`（整段抄，未转述）**

```markdown
#### 已定项 12：记录尺寸取 4 KiB

**定案**：**记录尺寸定为 4 KiB，定长，写进系统配置**（XFS 把同一个参数写进系统配置的 `h_size`）。

- 与已定项 4 相容：记录头 311 字节要完整落在一个 `physical_block_size` 内，而 4096 被 512 与 4096 两种探测值都整除。
- 定长记录取 2 的幂尺寸之后，I-8.2（记录头不跨原子单元） 的对齐浪费**恒为 0**：记录背靠背摆下去，起始偏移恒是单元的整数倍。
- 4 KiB 记录装 67 个点名项 <!-- format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD = 67 -->（已定项 4）；再大只抬高 I-8.1（环几何够大） 里那个「最坏事务占用」，进而抬高环大小下界，买不到别的。
- **用户口径**：先按这个数落地，写代码时再调整。

**射程**：⚠️ 余量那一格按的是「一次 fsync 一个事务」的读法（目标负载 12 项事务恰占 1 条记录），与 D16（发布语义） 已定项 5 末段的切分纪律（一个事务最多写一个单元的用户数据）不在同一口径：按切分纪律加已定项 7 一条记录一个事务号，seq 批 1 是 8 条记录、每条只装 1 到 5 个点名项，「12 项恰占 1 条」那一格不存在；余量口径与环几何怎么改归 C310（事务切分纪律与记录数口径打架），要用户定。目标负载（D25（目标负载优先级））若长大一个量级，这个数要重推，那时另立实验。环回绕与「装成一条要先在内存里缓冲整条记录」两条代价没建模。

**依据**：

- E75（记录尺寸与环几何）：4 KiB 上目标负载恰占一条记录、跨记录数 0，几个头部读法给的容量相同；环下界与另一个二进制已发表的那一格逐格相等；定长 2 的幂尺寸下对齐浪费恒 0。
- 用户定案，原话在变更史。

**欠**：C310（事务切分纪律与记录数口径打架）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:329-352`（整段抄，未转述）**

```markdown
#### 已定项 13：载荷完整性，`header_csum` 罩整条记录

**定案**：`header_csum` **覆盖整条记录 `[0, 4096)`**，含点名项数组与其后的补齐，`header_csum` 自身那 32 字节按零参与（32 字节里放 CRC32C 4 字节 + 28 字节零，D18（块里携带什么信息） 已定项 17）；`payload_csum` 那 4 字节字段**保留**，仍罩点名项数组，算法取 CRC32C（与已定项 11 的反向链同一个，不引入第二种）。**第一版不再单独分辨「头落了、载荷只落一半」**——任一校验和不过就是该记录无效，恢复断链即止（已定项 18 / I-8.3（重放前缀严格连续) 同一句口径）。

- 与 D18（块里携带什么信息） 已定项 17 逐字「自证结构（根记录、系统配置槽、journal 记录头）的校验和覆盖整个槽含补齐」一致：journal 记录是自证结构、槽就是那 4096 字节；只罩头的话，这条通用规则要为 journal 记录单开一个例外，而根记录与系统配置槽都没有这个例外。
- **四个候选里另外三个各被一条已定条款挡掉，这是排除不是取舍**：

  | 候选 | 挡它的 |
  |---|---|
  | 甲（一个校验和覆盖整条记录）| 会失去「头落尾没落」的可分辨性。第一版取的正是「整条」这个覆盖范围，同时保留 `payload_csum` 字段——也就是接受了这一行写的代价 |
  | 丙（每个点名项自带它所指目标的校验和）| D4（校验和位置） 的校验和证明「指向的那个单元完好」，**不证明「这条点名项本身完整落盘」**，而载荷完整性问的是后者 |
  | 丁（丙 + `named_count`）| 丙那个洞没补，只多一道数量核对 |

- `payload_csum` 是头里的字段，**必须落在 `header_csum` 的覆盖范围之内**，否则改掉它本身是静默的（载荷与校验和一起改，两个都过）；`header_csum` 罩整条 `[0, 4096)` 之后这条天然满足。

**射程**：⚠️ **代价是明写的**：分辨「头落了、载荷只落一半」与「载荷被改一个字节」原本靠两个校验和的**差异**，整条罩之后这两格与「整条没落全」在第一版判成同一种结局（记录无效）；买回来的是「与另外两个自证结构同一条规则」。要重开「只罩头」时，那张四格判据表原样可引。

**依据**：

- E76（载荷校验和的判别力）：另加的载荷校验和在检查、不是在回声头那一份（头落了载荷只落一半、载荷被改一个字节，都只有它红）；只有一个校验和的形态在类型上答不了「头落尾没落」；判决对两个摘要函数相同，是覆盖范围的产物。
- 用户弹窗定案取整条（主 agent 推荐只罩头 `[0, 307)`），没有走三方论证，可推翻，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:353-440`（整段抄，未转述）**

```markdown
#### 已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换

**定案**：**恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录；陈旧 tail 只是「从哪开始扫环」的优化，不再决定重放集合。**

**前缀判定的完整口径是五条，缺一不可**（引用 I-8.3（重放前缀严格连续）时连这句一起引）：
jsn 严格连续（断号即止）、**`(实例代号, checkpoint_txg)` 大于根的水位**、
提交标记齐全的事务才施加（D23（journal 的角色与格式）已定项 7）、
施加前逐项验证点名单元（D16（发布语义）已定项 7，E77（发布的持久顺序）证明承重）——**一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加**，有一份读不出或对不上，这条记录与它之后的都不施加。
⚠️ **第五条：施加的单位是一次发布——合法前缀停在一次发布的两个事务之间时，那次发布整体不施加，前四条判出来的前缀再按发布边界截短一次；引用这四条时连这一条一起引。**（D16（发布语义） 已定项 4）**发布边界怎么认**：一次发布的末条 = 记录标志位 0 为 1 的那一条（已定项 4、17）；前缀里一次发布的末条没到，那次发布整体不施加。

两份镜像何时算「记录在」：**任一份自证校验和过即在**（崩溃点重放里的检查由层 0 全量做出——jsn 3 只有一份持久的两个状态都读回文件）。点名单元不同：**两份都验过才施加**（前缀第四条）。代价：两块盘的池上一块盘整块掉了，记录在幸存盘上、点名单元有一份在掉了的盘上，那条记录不施加——恢复退到更早那一版，已确认（fsync 已返回）的最后一版丢掉；D16（发布语义） 已定项 8 的暖机管不到这一格。用户知情认下（C519（丢一整块盘时恢复丢掉刚确认的那一版） 那一格：「任一份验过」会不会放过 E77（发布的持久顺序） 要挡的一份新一份旧的嫁接没量过）。

**跨实例边界、链首、比较序、计数器与切换时的 W**（注 1–4）：

1. **前缀规则不跨实例边界**：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停（五条口径第一条原样）。**所选根覆盖的最后一条** = 与所选根同实例、同 `checkpoint_txg` 的记录里 jsn 最大的那条——一次发布切成多条记录时它们共享一个 `checkpoint_txg`（D16（发布语义） 已定项 6），那次发布的最后一条才是根覆盖到的末端。**那条读不出时**，链首接水位之上第一条可读记录，前提是它的「本次发布内序号」为 1（已定项 4）、`checkpoint_txg` = 所选根的 txg + 1；序号不是 1 说明下一次发布的开头缺了，断号即止。**「那条」按末条标志认**——所选根那次发布读得出的几条里没有一条带末条标志，就算「那条读不出」，走「序号为 1 的第一条可读记录当链首」那一支；不取读得出的同 txg 记录里 jsn 最大的那条（用户 2026-09-24 定，读法乙；三方 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 那一支四条臂同中、没分出候选，这一读法被攻过零轮）。所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。代价写在 D16（发布语义） 已定项 7 的注，要不要让新实例先暖机是 D16（发布语义） 已定项 8。
2. **记录水位 `(实例代号, checkpoint_txg)` 按实例代号为主比**，与 jsn 同序；择根照旧 txg 为主、实例代号破平局（D22（单元原子性怎么合成） 已定项 7），两处比的东西不同，各守各的序。按 txg 为主时，新实例的第一个根取「根环最大 + 1」，而被抛弃实例开放 checkpoint 里已提交的记录 txg 可以更大，会被当成水位之后施加到新实例的根上。
3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。前缀末的计数器取环里读得出的最大那一个：所选根自己那条记录两份都读不出时，新实例的第一条编在读得出的最大号 + 1，落回那条读不出的记录原来的槽（被重写的本来就读不出）；计数器从记录头读，不按 txg 推。定长环的槽位由 jsn 决定（槽位与序号解耦那条出路已被 E36（槽位映射那一维） 判掉），归零会让新实例的记录落到旧前缀还要用的槽上；已定项 9 的 48 位本来就按全卷寿命算。⚠️ **checkpoint_txg 也一样**：新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1；普通挂载本来就全环扫描，切换不为算这个下界重扫 journal 环（内存里的 txg 已不小于这次挂载见过的一切；切换的所选根要读盘择，见实例切换那一句）。它与 C331（择根倒挂压过已确认的写） 的修法候选「记录扫描水位」是同一个量，C331（择根倒挂压过已确认的写） 还清时一起写。代价：跳号比今天大（最多到被抛弃时间线的长度），D5（快照 / 空间记账机制） 已定项 2 的点删随之改成「删掉 ≤ 当前代 − K 的全部代」。
4. **切换时的 W 取被重发的那个 checkpoint 里的最大事务号**：开放 checkpoint 里已完成的事务全部 > W、按新写序重做。按第 2 条，它们的记录不在所选根之后；W 若取字面的「最后施加的事务号」，它们的单元会被谓词判成已发布而谁都不施加它们（C287（切换收养开放 checkpoint 的事务后再崩））。被重发的 checkpoint 里没有事务号非 0 的记录（重发的是空发布）时 W = 0（已定项 19 ①：事务号 0 不进 W 的 max）；W 写在每个被照旧事务的写序实例那一行，切换的所选根按实例切换那一句取。

**第一版没有干净关闭标记**：每次挂载一律走恢复（全环扫描、逐条验证、择根，按五条口径与注 1–4 施加），不按上一次是不是干净关闭分支；准入、暖机、回退候选集、F 的生效都不读这样一个标记。

**第一个文件版本接在哪条记录之后**：写第一个文件版本之前，要接的那一版之后的上一条记录必须解得出、且它的 `checkpoint_txg` 等于那一版的 txg——新记录的计数器取它的计数器 + 1、反向链取它的记录头（已定项 8、已定项 9）；不满足就在任何写之前拒绝（`crates/singlefs-core/src/transaction.rs` 的 `publish_first_file` 返回 `FirstFileVersionDoesNotFollowTheVersionItBuildsOn`），一个字节不写。

**管理员回退是挂着时的一次向前发布**：管理员在一次可写挂载里从回退候选集选一个旧根 R_old，文件系统发一次普通发布（D16（发布语义） 已定项 7 的持久顺序），新根的用户可见树（inode 树、extent 树；`deleted_inodes` 树落地之后也算，C118（`deleted_inodes` 树的形态无落点））的根指针取 R_old 那一版的；树表单元、分配记录树、中央映射树与记账统计量（D5（快照 / 空间记账机制） 已定项 4）按这次发布照调整之后的分配器现算，不从 R_old 那棵账载入（inode 号与树 ID 水位见表里「水位」一格）；实例表照 cur 的；checkpoint_txg 照常加一，**不新增实例、不取号、不写实例表行、不施加也不删除任何记录**。记号：cur = 现行那一版，new = 回退发出的新根，用户可见单元 = 数据单元与 extent 树、inode 树的单元。

| 项 | 定案 |
|---|---|
| 候选集 | 根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效（D16（发布语义） 已定项 1）∧ 带文件（那一版写过文件，不是还没写过文件的那一版）的根：(i, T) 可选 ⟺ cur 的实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被崩溃恢复抛弃的时间线上的根一条都选不中 |
| 释放 | cur 那一版的账里仍分配、而 new 不引用的落点，释放代 = new 的 txg；隔离单元（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1：任一份核出对不上、两份都留在已分配的）除外，留在已分配（用户 2026-09-27 定） |
| 复活 | R_old 引用的用户可见单元里，cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的分配代 |
| 水位 | inode 号水位与树 ID 水位 = max(cur 那一版内存里的水位, 环里读得出的根的水位)，不跟着 R_old 退回去 |
| 记录 | 回退那次发布的记录事务号取 0、不推进实例内计数（与空发布同：不承载用户事务，D23（journal 的角色与格式） 已定项 19 ①）；新根带的 F 取现行那一版根上的，不算抬 F；jsn 接现行那一条（主 agent 2026-09-26 定，推的，被攻过零轮） |
| 在任何写之前拒 | R_old 引用的用户可见单元在 cur 的账里没有记录，或仍分配而分配代与 R_old 那一版账里的不同；R_old 那一版或 cur 那一版的账有一个节点读不出；复活集里的每个单元（cur 的账记成已释放、这一次改回已分配的那些）在每块盘上的那一份都要读得出、校验和对得上，有一份不行就拒——它们的槽在 cur 的时间线上释放过，释放之后可能被一次没发布成的写盖掉一块盘上的那一份；cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内（与前缀第四条「两份都验过才施加」同一方向；主 agent 2026-09-26 定，推的，被攻过零轮） |

崩在回退那次发布的任何一点，恢复落到的与任何一次普通发布没成时相同；回退的根持久之后才向管理员确认（D16（发布语义） 已定项 7）。回退深度由候选集定：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。**被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账、只隔离其中**只被被抛弃根引用**的槽——查账的集合取窄读法：按主语「被抛弃时间线的根」读，仍被有效根引用的槽不在其内——「有效根」指回退候选集里的根（按实例表判仍然有效 ∧ txg ≥ F_生效），每次挂载与每次抬 F 按当时的候选集重算；被抛弃根的账（树表或分配记录树）读不出时不跳过：从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报，映射也现算不成就重读一次，仍不成在取号之前拒可写、只读照常（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定，被攻过零轮，判决 `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）；宽读法（凡被环里任一可读根引用过的槽都隔离）会让抬 F 在第一版 24 槽里买不到任何东西（影子账；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 同一次定案（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）把事务按单元切分 ⇒ 崩溃原子性的单位是一个单元、不是一次写请求（仓里没有任何决策承诺过写请求级原子性，这是一次对外语义的收缩）。

**可写挂载读到的更新状态读不出（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续，用户 2026-09-27 定；判据 N-配置续 照 `research/prompts/e158-r3-prereg.md` 第 348 行与 `research/prompts/e158-r4-prereg.md` 第 413 行）**：读阶段（择根、扫 journal、重放）读完，读池里每块盘两槽的系统配置：有一槽读不出或本池自证不过 ⇒ 判不出、按真（C331（择根倒挂压过已确认的写） 取甲，用户 2026-09-28 定，被攻过两轮，判决 `research/prompts/unreadable-at-mount-r2-main-verification.md` L2）；两槽都自证得过时取全部槽 journal tail 的最大值 c_见证，0 ⇒ 不判；所选那一版那次发布带末条标志的记录读得出（任一份）⇒ c_见证 > 它的计数器即为真；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) 大于所选那一版即为真；两条都读不出按真。为真就在这次挂载内有效的读缓存（只收读成且不是全零的落点）上把读阶段重做一遍（R = 1，取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」，两次读之间不等），重做那一遍为假就用它往下走，仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常。重建分配器判抛弃用的最新那条根的实例表读不出，同样重读一次、仍读不出拒可写（代码审阅第 22 条）。**「续」**：取号那一写（`crates/singlefs-core/src/transaction.rs` 的 `write_acquired_instance`）不再把 tail 写成 0，改写取号那一刻按同一取法读到的见证值 c_见证（用户 2026-09-27 定「乙-配置续（推荐）」；实现员 2026-09-27 交回并打上，落点是 `write_acquired_instance` 里的 `highest_system_configuration_journal_tail`：每块盘两槽里全部自证过、fsid 与本池相同的系统配置槽 journal_tail 取最大，一份都没有时 0，在第一道屏障之后、第一个取号写之前读一次）。取号失败的回卷写（D18（块里携带什么信息） 已定项 11）带同一个见证值，不退回 0。mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：新池新建文件的字节不变。取号那一刻某块盘两槽里有一槽读不出或本池自证不过：在第一个取号写之前拒（C331（择根倒挂压过已确认的写） 取甲，用户 2026-09-28 定；`crates/singlefs-core/src/transaction.rs` 的 `self_verified_system_configurations_of_every_device`）。系统配置没见证到的最新根，「续」落地之后由下一次挂载在自己的 c_见证 里看到；这一形记欠见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 的 Q1。

**失败的处置按失败的性质分两支**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 失败表）。**判别子是当下判得出的量，不是次数**：失败发生时先对**目标设备的固定落点做一次探针写**——写不进去 ⇒ 持续失败，走**转只读到下次挂载**；写得进去，再**对这次失败的那个落点做一次只读复核**：读得出 ⇒ 瞬时失败，走**实例切换**；读不出 ⇒ 落点级的持续坏，同样走**转只读到下次挂载**。⚠️ 复核这一步**只读不写**：写探针进失败落点会抹掉那里的权威数据（8 个失败点里 4 格的落点是根槽或系统配置槽），而固定落点这一步不许改——`D2`（RAID 条带策略） **已定项 13** 逐字「与 `D23`（journal 的角色与格式） 已定项 14 的两支判别子同一个动作」靠它成立（主 agent 现查：那句话在 `.claude/kb/decisions/02-RAID条带策略.md:225`，归属已定项 13「降级期间只读」，不是已定项 15）。**失败点是屏障时没有落点可复核**，那几格只有固定落点探针写这一步。可写设备数掉到 w 的下限以下、切换自己的预留拿不到，同样直接走只读支；连续切换次数越过 **N_switch = 3**（具名可调参数）也转只读，它是兜底的兜底，真正的判别子是探针写加落点只读复核。**收敛靠的是切换的第一步就是一次会失败的写**：切换要先取号，取号要写独占打开成功的每一份系统配置、全或无 ⇒「写不出去」这类故障最多让切换走一步（探针写先拦，侥幸过了全或无也立刻判失败）。

- 切换要用的块在挂载准入时预留：**它是内存里的一个空间量**，每次挂载重算、崩溃即丢、**不落盘**——分配记录条目只有「已分配」与「已释放 + 释放代」两个态（D3（空间分配） 已定项 7），盘上没法表达第三个态，而落盘表达它本身就要写单元、回到「要写才能写」。预留量按 **(N_switch + 1) × 一次切换的最坏量**（一次挂载允许 N_switch 次切换，且实例表链每切一次长一行、写行要 COW 重写整条链，第 k 次比第 1 次贵；多的一份给这次挂载写行那次发布的元数据）；**这个量的口径与式子的权威正文在 D28（挂载期承诺量） 已定项 3**：一次切换的最坏量 = 实例表链重写（副本数 2 × 32768 × 片数，按设备算）+ 暖机空发布（至多 R 次 × 现算 c_max），**固定点重做与号 > W 的数据单元重写不另要空间**——切换是挂载内一次恢复，重建态里失败那次的分配不存在，重做走 checkpoint 的保留池；引这一段时引 D28（挂载期承诺量） 已定项 3。
- **预留挡的是分配失败那一路，不挡写失败那一路**——写失败由探针写与取号全或无挡。
- **「分配失败」按它发生在哪里拆三支**（不拆的话池一满就整卷只读，而释放空间要写盘、重挂又被预留合取挡住，撞了 `.claude/rules/fs-design.md` 那句「释放空间这个操作本身不需要申请空间」）——前台事务的用户数据分配失败 ⇒ **向调用者返回 ENOSPC**，checkpoint 照常发布、不动挂载状态；后台、可续做、非决策路径（重平衡、墓碑回收、scrub、整理）的分配失败 ⇒ **暂停那个活并记进意图**，不动挂载状态（`.claude/rules/fs-design.md` 第三格「无戒律」、D2（RAID 条带策略） 已定项 4 c「前台不停」）；**切换自己的预留拿不到** ⇒ 才转只读。
- 根槽写失败重发时 checkpoint_txg 推进一格再发（根环区域 = `txg mod R`，D22（单元原子性怎么合成） 已定项 2，不推进就是反复重写同一个槽、把 R 个失败域用成一个）；⚠️ 由此「同一个根槽连续失败」不再是一条判据——槽位随 txg 轮转，N_switch ≤ R 时它永远够不着，留着它读起来像一道闸、实际是空的。
- **实例切换** = 挂载内做一次恢复：取新实例代号、写行、重发在飞 checkpoint——所选根**重新读盘择根**：对着仍打开的设备句柄按恢复择根的同一套规则现选（`crates/singlefs-core/src/recovery.rs` 的 `choose_root`），不取调用方内存里记的根——根已落盘而系统配置槽那一步失败时，内存里记的根比盘上的现行根旧（C458（实例切换取内存里的根，不重新读盘））；行只落在 [这个根的实例, 新实例)，每个被照旧事务的写序实例 k 写 (k, k 最后发布的 txg（没发布过根时 0）, 被照旧的、写序属于 k 的最大事务号)，其余按 D18（块里携带什么信息） 已定项 11（旧实例发布过根、被照旧的事务都是它自己的时候，就是 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号) 这一行），号 ≤ W 的事务照旧（开放 checkpoint 里已完成的事务全部 > W，C287（切换收养开放 checkpoint 的事务后再崩） 按这个 W 封死，注 4）、号 > W 的按新写序重做或向还没返回的调用者报错，固定点单元全部按新实例重写；不转只读、不等下次挂载，代价 = 一条行 + 整条实例表链重写 + 号 > W 的数据单元重写 + 一次固定点重做，频率未测。

**这一版的失败处置**：发布不接受失败。一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它（checkpoint_txg、计数器、本次发布内序号、记录标志、单元的位置与字节都不变），重发成功才建下一次发布；于是合法历史里同一 (实例代号, checkpoint_txg) 不会有两条带末条标志的记录，恢复遇到它就是盘坏了，在任何写之前停下（`crates/singlefs-core/src/recovery.rs` 的 `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`）。原样重发管这一版的串行提交；探针写、两支分流、实例切换与根槽写失败推进 txg 仍在本项，随失败处置的代码在后面的里程碑做。原样重发在串行提交下会把一次失败放大成整条发布流阻塞，欠 C541（原样重发把一次失败放大成整条发布流阻塞）。

**射程**：管恢复施加哪一段、回退与切换怎么做、失败怎么分流；实例表的行格式与写行规则归 D18（块里携带什么信息） 已定项 11，切换预留的量归 D28（挂载期承诺量） 已定项 3，defer 窗口与 F 归 D16（发布语义） 已定项 1。已知边角：

- ⚠️ 它对已定项 3 是收窄不是推翻：「先全环扫描、逐条验证、不许先信 tail」原样成立——变的是验证出的合法前缀里**哪一段被施加**。
- ⚠️ **影子账与按实例表判抛弃（`abandoned_by_table`）留着，理由是崩溃恢复，不是管理员回退**：一段没有任何管理员回退的历史里，崩溃恢复因暂时读错落到旧根、实例表抛弃了较新的根，关掉影子账，被抛弃的根引用的数据单元被复用，之后恢复落在它上面读不出（第一轮 H6）。「只在崩溃恢复落到旧根的那次挂载里隔离」挡不住它：被复用的写发生在下一次干净的挂载里（第二轮辩方 G6，读代码的判断、没量）。落 [checks-owed.md](../checks-owed.md) C314（回退可以复用被抛弃的根引用的单元）。 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续（用户 2026-09-27 定）之后，系统配置见证过的较新的根暂时读不出不再被抛弃：可写挂载重读一次，仍读不出就拒可写；影子账留着的理由收窄成系统配置没见证到的那一形（那次发布的系统配置轮换没落盘）。
- ⚠️ **C331（择根倒挂压过已确认的写） 取甲的代价（量的，`research/prompts/unreadable-at-mount-r2-main-verification.md` L1）**：撕裂的系统配置轮换（普通断电就造得出，被撕裂的那次发布没确认过）之后、一槽系统配置持续读错之后，可写挂载每次都拒，只读挂载不写盘，那一槽没有机会被重写——池只剩只读挂载（撕裂 5 态连挂 6 次）；「只读挂载不写盘」对今天的代码量实（2016 条轨迹），条款上没有，见 C32（无密钥的一侧写不写得出字节没有闸）、C33（scrub 检出之后做什么没有条款）、C75（降级挂载这道闸自己不会红）。一次可写挂载都做不成时怎么走出来，没有条款（C596（撕裂的系统配置轮换之后只剩只读挂载））。C393（被抛弃根的账读不出时修复没有条款） 从映射也现算不成的那一形同样只剩只读挂载。
- ⚠️ **挂着时回退的已知边角三样**：① 「R_old 仍分配的落点在 cur 的账里是同槽同分配代」这道核不在定案里，也补不上——已释放的记录记的是释放代（`crates/singlefs-core/src/allocator.rs:38`），核看不出「这个槽被复用过又释放了」；它放过的读不对的回退只在 F 回落时走得到（第三轮判决第二节 K1：F 生效取最大值的最小值时 15 格中 12 格，取 SysPre 时 0），靠 F 不回落挡。② 崩溃恢复抛弃的根在重开时全部暂时读不出，影子账读不到它们的账、隔离 0，之后别的实例复用了它们的槽，撤故障再把新实例的根全改坏，恢复落在被抛弃的根上读不出——与回退形态无关的多重故障，见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）——乙-配置续之后这一形只剩被抛弃根是系统配置没见证到的那一种：见证过的在崩溃恢复那次挂载里重读一次仍读不出就拒可写，抛弃不了。③ 两个状态来回回退超过根环一圈时，可退到的不同状态少于 4 个（D16（发布语义） 已定项 1 的根环容量边界）。
- 实现今天的链首不锚在所选根覆盖的最后一条（`crates/singlefs-core/src/recovery.rs` 的 `replay_journal`），欠在 C365（恢复路径的链首不锚在所选根覆盖的最后一条）。
- ⚠️ **只读复核买的是落点持续坏那一类，代价是它自己零轮。** 固定落点探针写单独用时，落点持续坏那一类会反复白切换（攻方在副本装置上量到 24 个持续故障场景合计 46 次，那个数没在入库装置重做过、不进 kb）；只读复核能分辨它。这条改法是攻方腿自己提的，**被攻过零轮**，用户 2026-09-21 知情接受。
- **切换重新读盘择根带出的三件**：读盘选出的根比内存里记的新时，不重发在飞 checkpoint，以读盘选出的根为准接着走；切换写的实例表行与 W 按读盘选出的根写；回退那次发布里，管理员选的 R_old 必须在「以读盘选出的根为最新根」算出的回退候选集里，不在就拒绝、盘上逐字节不变。挂载内实例切换的实现随里程碑「覆盖写、释放、回退与复用」收口表第 16、40 行在后面的里程碑（C458（实例切换取内存里的根，不重新读盘））。
- ⚠️ **零单元发布这一支不冻结**：发布末尾系统配置轮换之后那道屏障报错时，带单元 / 带记录的发布与轮换报错同一处置（冻结、原样重发）；零单元发布（`crates/singlefs-core/src/transaction.rs` 的 `publish_without_units`，末尾走 `persist_the_root_then_rotate_the_system_configuration` 那一路）这一支不冻结、整个挂载返回错误（失败经 `PublishError::BlockDevice`），下一次发布落在下一次挂载、换了实例代号；这是替条款没写的一支做的选择（判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节「最新根读不出、见证与屏障」那一格；`research/prompts/m2-impl-c577-barrier-implementer-report.md` 第 106–111 行「停下交主 agent 的设计问题」第 2 条）。钉它的用例是 `crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs` 的 `a_zero_unit_publish_whose_post_rotation_barrier_fails_is_not_frozen_and_the_whole_mount_returns_the_error`：只做过 mkfs 的池第一次可写挂载，写行那次零单元发布在盘 1 轮换之后那道屏障报错一次 ⇒ 挂载报 `MountError::Publish`、原因 `PublishError::BlockDevice`，写账已落盘的发布 0 份、失败的 1 份；录制流落到盘 0 那道屏障为止，盘 1 那一道没进录制流；换回不报错的设备之后再可写挂载，取号 2、择到 (实例 1, txg 1)。它只钉文件镜像上的结局：harness 的文件设备没有写缓存，盘 1 的轮换写在屏障报错之前已经进了镜像，「屏障报错时轮换其实没持久」那一种崩溃状态它不罩（`research/prompts/m2-impl-r2-fixes-a-implementer-report.md`「结论」第 4 行、「停下交主 agent 的」第 5 条）。
- ⚠️ **C366（暖机路径把 txg 写进计数器与 tail） 那条定案在可写挂载这一层没有可达盘面**：注 3「前缀末的计数器取环里读得出的最大那一个」只在所选根自己那条记录两份都读不出时与 txg 分得开；那一形在可写挂载的读阶段被 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙判成「两条都读不出按真」，重读一次仍为真就在取号之前拒可写（C579（判据在单故障合法状态上也拒可写） 用户定维持乙字面），走不到取号、写行与暖机。今天钉住的是：函数层 `crates/singlefs-harness/tests/warm_up_journal_counter.rs` 的 `warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two`（环里末条 jsn 40 时暖机写 jsn 41、42，txg 仍是 1、2）；同一份文件的 `c366_when_the_chosen_root_own_record_is_unreadable_the_writable_mount_is_refused_by_the_undecidable_witness_and_the_read_only_mount_opens_the_chosen_root`：可写挂载拒在任何写之前、两块盘逐字节不变，只读挂载照常择 (实例 1, txg 4)、环里读得出的 jsn 1–3 一条不施加（`research/prompts/m2-investigate-three-reds-report.md` 第一节第 2 条、第七节；`research/prompts/m2-impl-r2-fixes-b-implementer-report.md` 第三节 5c 行）。

**依据**：

- E78（重放的起点）：陈旧 tail 叠上块重用之后，「全环扫描 + 断号即止 + 幂等」三件套不闭合——失配即中止的恢复在健康镜像上自我中止，失配跳过的撕裂判别力归零；水位臂与尾删臂都过全部判据，取水位臂是因为水位就是根记录已有的 `(实例代号, checkpoint_txg)`（D22（单元原子性怎么合成） 已定项 7），零新增格式字段，而尾删臂要在恢复路径里加一个重试环。
- E77（发布的持久顺序）：拿掉施加前的点名单元验证，单元与记录同段那一臂当场出静默嫁接——第四条承重。
- E104（扫描重建的现行版本判定）：新根取 T_old + 1 时压不过被抛弃的根——注 2、注 3 新实例的第一次发布从环里最大 + 1 起靠它；它的另一半「不写回退行时被回退抛弃的单元在全规则臂上复活」撑的是回退行，回退行删了，那一半不再承重。
- E150（回退复用被抛弃的根引用的单元）：影子账在模型里两格零违例，多隔离的单元到被抛弃的最新根被轮转覆写为止。模型里的被抛弃时间线是管理员回退造出来的；今天影子账留着的理由是崩溃恢复抛弃的时间线（第一轮 H6），机理相同，模型没重跑那一形。
- E142（新池新建文件的干跑）：干跑时装置取「两份镜像任一份合法即在」，改坏一份记录仍读到全部合法记录——「记录在」的口径由此收口。
- 三方：`research/prompts/c199-r1-main-verification.md`、`research/prompts/c199-r2-main-verification.md`（注 1–4）；`research/prompts/c322-r3-main-verification.md`（取号屏障与跳过的号）；`research/prompts/c329-c330-r3-main-verification.md`（切换的所选根与 W 那一行）；`research/prompts/c143-r3-main-verification.md`（第一个新根的 txg 取 CJ2）；`research/prompts/m2-r1-main-verification.md` 第三节与 `research/prompts/m2-step45-code-r3-main-verification.md` 第六节 5（影子账查谁的账、有效根）；C113（扫描重建时多版单元的现行版本判定无输入） 定案（`_c113-r9-final.md`）。
- 用户定案（重放下界、回退、失败表、切换预留口径、两份镜像的口径各一次），原话在变更史。
- `research/prompts/c381-r3-main-verification.md` 第三节：候选乙（探针写改成写失败的那个落点）被三处独立打中——屏障上没有落点、8 个失败点里 4 格的落点是权威结构、与用户第二轮定的第 1 题（落点进地址空间表，买的就是「写坏了也不心疼」）对撞、`D2`（RAID 条带策略） 已定项 13 那句话会当场为假。
- 用户定案 2026-09-21（改法 B），原话在变更史。
- 用户定案 2026-09-23：切换的所选根重新读盘择根（`research/prompts/c381-r3-main-verification.md` 第二节记的零轮形态、第四节第 3 题），原话在变更史；被攻过零轮。
- 用户定案 2026-09-23：注 3 的前缀末计数器取环里读得出的最大那一个（C366（暖机路径把 txg 写进计数器与 tail） 实现员查出所选根自己那条记录读不出时条款没写），原话在变更史；这条定案在可写挂载这一层没有可达盘面（射程「C366（暖机路径把 txg 写进计数器与 tail） 那条定案在可写挂载这一层没有可达盘面」那一条），C579（判据在单故障合法状态上也拒可写） 用户 2026-09-27 定维持乙字面、拒可写（`research/prompts/m2-investigate-three-reds-report.md`），原话在变更史。
- 用户定案 2026-09-23：第五条的发布边界按已定项 17 那条约定认末条（C245（系统配置槽的写频率，两条条款说反话） 第二轮 `research/prompts/c245-r2-main-verification.md` 第四节列的三个判别量之一，被攻过零轮），原话在变更史。末条按记录标志位 0 认（已定项 17）；一次发布的记录里没读到带标志的那条，整次不施加。
- 用户定案 2026-09-23：注 1 的锚点取同 txg 里 jsn 最大的那条、锚点读不出时靠本次发布内序号接链首，第一版没有干净关闭标记（三方第一轮 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K3、K4；K4 的两个改法都被攻过零轮，本次发布内序号那一个是推的、没实现），原话在变更史。
- 用户定案 2026-09-24：第一个文件版本写之前核上一条记录，照今天的实现写成条款（C495（第一个文件版本核上一版记录没有条款），三方 `research/prompts/m2-refusals-presumed-r1-main-verification.md` 第三节 R5：拒之前零字节写、被拒那一族零故障不可达），原话在变更史。
- 用户定案 2026-09-24：切换重新读盘择根带出的三件照主 agent 推荐定（判决 `research/prompts/m2-newq-r1-main-verification.md` 第三节 N5–N7，原话在变更史）；无实验：三件都不改盘上格式、实现不在这个里程碑，差别只在语义。
- 用户定案 2026-09-24：末条再跨记录、最后一条带标志位（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T6（维持拒绝时拒不拒取决于池的历史），按今天的字面认末条会施加半次发布，带标志位那一格 0 / 0）。
- 用户定案 2026-09-24：末条按标志认、注 1 的锚点读法乙（原话在变更史；实十三报告 `research/prompts/m2-p6-header-implementer-report.md` 第四节 ① 列的两种读法）。
- 用户定案 2026-09-24：这一版发布失败原样重发（实二十报告 `research/prompts/m2-lastflag-implementer-report.md` 第六节 1：同一实例失败后接着发，会留下两条同 (实例代号, checkpoint_txg) 都带末条标志的记录），原话在变更史；无实验：只定失败之后发什么，不改盘上格式。
- `research/prompts/m2-final-code-r2-main-verification.md` 第三节 Z8（这一版发布失败原样重发，攻方没打中）；那一份的 Z9 与第四节改法 1–4 是回退见证的，随见证删掉不再承重。
- 管理员回退改成挂着时的一次向前发布（候选集、释放、复活、水位、拒）、删前缀第五条与回退见证、影子账依据改指 H6：无实验，三方原型（`research/prompts/m2-rollback-forward-r1-opus-model/`、`m2-rollback-forward-r2-opus-model/`、`m2-rollback-forward-r3-opus-model/`）在冻结副本上量过，没立实验号，原型上的数不是入库装置上的数；三轮判决 `research/prompts/m2-rollback-forward-r1-main-verification.md`、`m2-rollback-forward-r2-main-verification.md`、`m2-rollback-forward-r3-main-verification.md`。水位取 max(内存, 环) 的收严（第三轮判决第二节 K1 的水位分句）被攻过零轮；用户定案 2026-09-25（向前发布、只在挂着的时候做）与 2026-09-26（第三轮交用户的四问），原话在变更史。
- 主 agent 2026-09-26 定：回退那次发布之前逐盘验复活集里的每个单元（「在任何写之前拒」那一格最后一句），照用户 2026-09-25 原话「1 就是fsync失败后 两个盘掉一个盘。这个不能认，这个违背我们数据安全的承诺」（`records/2026-09-24-里程碑二收尾调度.md` 第三节那一时刻的一行），与 C519（丢一整块盘时恢复丢掉刚确认的那一版） 那一格「两份都验过才施加」同一方向；无实验，推的，被攻过零轮；多读的量随 R_old 之后释放的用户可见单元数涨，没量。
- 用户定案 2026-09-25：点名单元两份都验过才施加、认下丢一整块盘时丢掉刚确认的那一版（C519（丢一整块盘时恢复丢掉刚确认的那一版） 实一复现：`crates/singlefs-harness/tests/whole_device_loss_after_warm_up.rs`），原话在变更史；无实验：照今天的实现写成条款，另一读法「任一份验过」会不会放过 E77（发布的持久顺序） 那种嫁接没量。
- 用户定案 2026-09-27（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续，先选乙、再补选「配置续（推荐）」）：可写挂载读到系统配置见证过的更新状态读不出，重读一次仍读不出就拒可写；取号那一写不再写 tail = 0，改写取号那一刻按同一取法读到的见证值（实现已合入，`crates/singlefs-core/src/transaction.rs` 的 `highest_system_configuration_journal_tail`）；判据 N-配置续 定义在 `research/prompts/e158-r3-prereg.md` 第 348 行与 `research/prompts/e158-r4-prereg.md` 第 413 行，原话在变更史。E158（择根与修复四岔路） 第 4 次跑：第三段 H1g 把乙-配置与乙-配置续分开（16/32 对 0/32），第二段（以装置撞键修过的 seg2b 为准）四族两者逐族相同，页上「2026-09-27：第 4 次跑」一节的岔路表第 1、2 行。

**欠**：C77（重放起点未定义）；C314（回退可以复用被抛弃的根引用的单元）；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）；C331（择根倒挂压过已确认的写）；C334（切换的所选根没有会红的检查）；C365（恢复路径的链首不锚在所选根覆盖的最后一条）；C287（切换收养开放 checkpoint 的事务后再崩）；C126（切换预留的最坏量没有口径）；C493（回退候选集条文与实现说反话）；C381（根已落盘之后发布失败，分配器仍退回）——三方第二、三轮打中失败表那两支判别子（探针写在盘上没有落点、屏障类失败点连「失败的那个落点」都没有定义），用户已定「落点进地址空间表、射程逐个失败点列一张表」，第 2、4 题已定（`D16`（发布语义） 新立一条已定项、失败表改两步判别子），「实例切换取的是内存里的根」那一题定成切换重新读盘择根、带出的三件待定（`research/prompts/c381-r2-main-verification.md`、`c381-r3-main-verification.md`）；代码那一半（探针写、只读复核、实例切换、转只读）跟里程碑「覆盖写、释放、回退与复用」收口表第 16 行一起挪到后面的里程碑；C458（实例切换取内存里的根，不重新读盘）：切换重新读盘择根那条会红的检查；C539（锚点读得出时下一次发布的首条序号不是 1）；C540（末条标志坏在一次发布中间，读者切出两次发布）；C541（原样重发把一次失败放大成整条发布流阻塞）；C558（回退目标是环里最旧的根时回退那次发布会写坏它）；C596（撕裂的系统配置轮换之后只剩只读挂载）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:441-453`（整段抄，未转述）**

```markdown
#### 已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根

**定案**：**由记录重建那次发布的根：记录头加「新根段」= 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 188 字节 <!-- format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES = 188 -->（实例表单元指针照所选根），4096 记录装 67 个点名项。施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义。已定项 14 条 2「严格大于所选根就施加」原样成立。**

**射程**：管「记录在、根槽不在」那一格恢复施加什么；两条指针的宽度随 D19（块指针的结构与宽度预算） 已定项 7 / 11 走。施加一条记录时指针层上其余的事（分配器登记、overlay）不在这一格，归 C284（施加一条记录在指针层上做什么没有定义） 与 D28（挂载期承诺量） 已定项 2。

**依据**：

- E142（新池新建文件的干跑）：新根段加进记录头之后，「施加一条记录」在指针层上有了定义，忽略 journal 与查 journal 的恢复结果在一批状态上不同——journal 承重。
- 用户定案，原话在变更史。

**欠**：C284（施加一条记录在指针层上做什么没有定义）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:454-469`（整段抄，未转述）**

```markdown
#### 已定项 16：mkfs 写实例代号 0，第一次可写挂载取 1

**定案**：**mkfs 写 0，0 = 「mkfs、尚无实例」，不是有效实例；第一次可写挂载按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 取 max(系统配置, 根环) + 1 = 1，并写进每一份系统配置（一次系统配置槽写，世代号 +1）之后才动单元。mkfs 写出的单元写序 (0, 0)、第 0 代根实例代号 0；D18（块里携带什么信息） 已定项 11 那句「从 1 起、0 无效」读成「实例从 1 起，0 保留给 mkfs」。**

- **取号那一步的屏障**：取号那两次系统配置写之后、本实例第一个非系统配置写之前至少一道完成了的屏障，首次挂载路径上暖机开场那道就算、不另发；那道屏障完成之前本实例的全部写者都不发单元、记录、根；那道屏障在任一块盘上报错 ⇒ 取号全或无失败，不许只重发屏障就继续。取号的 max 读法与回卷写带的号写在 D18（块里携带什么信息） 已定项 11，世代号按 D22（单元原子性怎么合成） 已定项 16 逐盘 + 1。

**射程**：管实例代号从几起、取号那一步的次序；实例表的行怎么写归 D18（块里携带什么信息） 已定项 11。⚠️ 取号之后，写行或暖机那几次发布因块设备报错失败时，已写进系统配置的新代号**不回卷**——回卷只管取号自己那几次系统配置写（D18（块里携带什么信息） 已定项 11）；这个号就此烧掉，下一次可写挂载按 max + 1 取下一个，被烧掉的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行。这是已知行为，不当缺陷修。

**依据**：

- 无实验：实例代号的起点是编码约定；取号屏障的判别力在层 0 崩溃点重放里，不是计数实验。
- 三方：`research/prompts/c322-r3-main-verification.md` 第五节（取号那一步的屏障，三轮三方之后用户定案）；实例代号的起点是用户定案。原话都在变更史。
- 用户定案 2026-09-23：取号之后写行或暖机失败不回卷，认了、写成已知行为（里程碑「覆盖写、释放、回退与复用」收口表第 57 行），原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:470-489`（整段抄，未转述）**

```markdown
#### 已定项 17：点名项 56 字节的字段表

**定案**：**一项一个单元（新池新建文件 8 项）：位置条目 14 × 2（偏移 0）+ 单元类型标签 1（28）+ 出生树 8（29）+ 出生 txg 8（37）+ key 尾段 10（45：码 1 写序；码 2 / 码 3 实例代号 4 + 出生序号 4 + 补零 2）+ flags 1（55）= 56 <!-- format-const: JOURNAL_NAMED_ENTRY_BYTES = 56 -->；不另带载荷 CRC（位置条目各带 4 字节校验和）。重放从点名项直接凑出映射 key。**

- **记录点名的每一项自带校验和**：D4（校验和位置） 已定校验和内联进父指针，而记录点名的东西在记录持久时它的父还没发出 ⇒ 记录临时充当父，否则重放时那些单元没有任何完整性凭据；恢复重放施加前必须逐项验证点名单元（已定项 14 第四条）。
- **一次发布切成 N 个事务、N 条记录时，这次发布共享的提交内生块（extent 树节点、inode 叶容器与根、分配记录树、记账树、中央映射树、树表单元）只在最后一条记录里点名**，前面几条不重复点名它们。
- **共享的提交内生块多于最后一条记录装得下（4096 记录 67 项）时，末条再跨记录**：从最后一个事务那条记录起依次多写几条，装满一条再开下一条，只有真正的最后一条带「本次发布末条」标志；恢复按这个标志认一次发布的边界，一次发布的记录里没读到带标志的那条，整次不施加（已定项 14 第五条）。标志放在记录头十个字段里「填充 1」那个字节（改名「记录标志 1」），位 0 = 本次发布末条；每一次只有一条记录的发布（含空发布记录），那一条的位 0 也写 1——改新池新建文件的字节（w1、w4、t9 三条记录的这一字节与头部校验和 [46, 50)；w4、t9 两条的反向链 [91, 95) 也跟着变，E142（新池新建文件的干跑） 空白 G26 逐字节核过）。用户 2026-09-24 定。

**射程**：管点名项的字节；位置条目的构成归 D19（块指针的结构与宽度预算） 已定项 4 / 11，映射 key 的形态归 D19（块指针的结构与宽度预算） 已定项 6 / 10。⚠️ 共享的提交内生块只在最后一条点名时，崩在最后一条持久之前，那些单元没有任何记录点名——重放的回收路径要认得出这一格，层 0 的小负载要覆盖它（C491（多条记录时共享内生块在哪条点名没定））。

**依据**：

- E16（journal 的角色：WAL vs 意图日志）：记录点名的东西其父此刻还没写到盘上，记录必须临时充当父——逼出「点名的每一项自带校验和」。
- E77（发布的持久顺序）：点名项自带校验和、施加前逐项验证，是单元与记录之间那道屏障得以省掉的交换条件。
- 用户定案，原话在变更史。
- 用户定案 2026-09-23：共享的提交内生块只在最后一条记录里点名，原话在变更史。
- 用户定案 2026-09-24：末条再跨记录、最后一条带标志位（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T6（维持拒绝时拒不拒取决于池的历史），按今天的字面认末条会施加半次发布，带标志位那一格 0 / 0）。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:490-504`（整段抄，未转述）**

```markdown
#### 已定项 18：在飞记录数上限的取值与语义，tail 字段的编码

**定案**：在飞记录数上限 = 环槽数 ÷ F（`F` 是 I-8.1（环几何够大） 的安全系数），默认环下 196608 ÷ 3 = **65536**，写进系统配置；tail 8 字节存 jsn 的 48 位计数器，靠系统配置自己的两槽轮换取冗余，不另设 tail 槽；记录 n 落在环内偏移 `(计数器 − 1) mod 槽数 × 4096`。

- **语义**：这个数的含义是**一次恢复重放的前缀最多 N 条**。读到一条校验和不过的记录时，恢复**断链即止**——停在它之前那一条，不追问它是「撕裂」还是「损坏」，也不因此拒绝只读挂载；可写挂载另按 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续判（已定项 14）：系统配置见证过比断链处那一版新的发布、重读一次仍判真，拒可写。不是 XFS 那种「最后 N 条内的坏校验和算撕裂、之外算损坏拒绝挂载」。

**射程**：⚠️ 代价：一条真正的「盘中段损坏」与一条「尾部撕裂」在第一版判成同一种结局，池照常挂上、损坏那一条之后的记录全部不施加。第一版串行提交（D13（验证路线） 已定项 2），任一时刻只有一条记录在飞 ⇒ XFS 那套在第一版里 N 恒 1，而它要的那个「N = 并发写出的记录数」与「环槽数 ÷ F」是两个量纲不同的数，同一个字段装不下两种语义。

**依据**：

- 无实验：取值是 I-8.1（环几何够大） 的算术（环槽数 ÷ F），语义是一次口径裁定——断链即止已经是 I-8.3（重放前缀严格连续) 的判据，这一句只是把「校验和不过」也并进同一条出口，不新增分支。
- 用户弹窗定案，没有走三方论证，可推翻，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:505-521`（整段抄，未转述）**

```markdown
#### 已定项 19：空发布记录的事务号与提交标记、`previous_hash`、环长

**定案**：

- **① 空发布记录**：事务号 0 保留给不承载事务的记录，提交标记写 1；不进实例表 W 的 max，不进「丢掉提交标记没出现的尾巴」判定。
- **② `previous_hash`** = CRC32C(**本实例内逻辑前一条**记录的头，其中 `header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录恒 0**；写路径**不读盘**。反向链是**按实例**的——I-8.3（重放前缀严格连续) 逐字「跨实例边界即停」，重放本来就不会从一个实例的记录接到另一个实例的记录上，实例边界另有实例代号挡着；取「环上前一格的记录头」那一种，写路径每写一条记录就要先把环上那一格读回来，买到的只是「跨实例也能连上」，而那条链没有任何判定在用。字节表里 w1 那一行写 0 的理由是「本实例第一条」，不是「环上无前一条」。
- **③ 环长**：环长是 **mkfs 参数**，**默认 768 MiB**；约束 **环 ≤ 设备容量 ÷ 4**，越界**拒绝 mkfs**。系统配置里仍只存「环长度（字节）」这一个字段，不另存容量或比例。**挂载时有效 `T_dirty` = min(`T_dirty` 字段, 环长 ÷ F)**——系统配置里 `T_dirty` 仍写 2 GiB，默认 768 MiB 环下有效值 = min(2 GiB, 256 MiB) = 256 MiB。单元区起始槽号随环长走，默认环下是 784 MiB（槽 50176）。`T_dirty` 的夹取是 I-8.1（环几何够大） 的直接推论——环装不下 `F ×` 最坏占用时，能脏的字节数必须跟着降，否则一次发布发不出去。

**射程**：③ 的「参数 + 上界」让大盘上 768 MiB 只占容量的百分之几、小镜像按参数缩，`环 ≤ 容量 ÷ 4` 挡住「环吃掉整个盘」；⚠️ 连带：`.claude/kb/layout/01-first-txn.md` 的预想镜像是 **4 GiB**（1 GiB 的镜像在这条约束下装不下 768 MiB 的环）。实现：`crates/singlefs-core/src/make_filesystem.rs` 收环长参数、按最小那块盘的容量 ÷ 4 拒绝越界，默认值是 `crates/singlefs-format/src/lib.rs` 的 `JOURNAL_RING_DEFAULT_BYTES`。 挂载读者一侧另判环长：在飞上限（环槽数 ÷ F，已定项 18）≥ 1，且环末端不越过编译期单元区起点（768 MiB，`crates/singlefs-format/src/lib.rs` 的 `UNIT_AREA_START_SLOT` 与 `JOURNAL_RING_START_SLOT`）；越界整池拒绝挂载（实审 A3a 落地：`crates/singlefs-core/src/recovery.rs` 的 `SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange`）。

**依据**：

- E143（一事务一单元下的 journal 账）：按切分纪律，`T_dirty` 满窗要的环是预想 64 MiB 环的 4 倍，要么环 ≥ 768 MiB（F = 3）、要么 `T_dirty` 压小——默认环长与 `T_dirty` 夹取的来处。
- ①、②、③ 都是用户弹窗定案，没有走三方论证，可推翻；主 agent 推荐的也是这几档，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:522-546`（整段抄，未转述）**

```markdown
#### 已定项 20：记录承载指针层的目标态，分配先于 journal

**定案**：**journal 记录承载「指针层的目标态」，不是逻辑意图，也不是全量物理重做。** 单元由 COW 写到各自最终位置并自带校验和（D22（单元原子性怎么合成） 已定项 18 里「COW + D4（校验和位置） 够」那一类），journal 只记「哪些子树根换到了哪个块 + 新根」。

- **由此得到一条写路径的硬顺序：分配先于 journal。** journal 记录不是「事务的全部」，是「**已经写到盘上的东西的发布指令**」。
- 判据：

  | 判据 | 逻辑意图 | 全量物理重做 | **指针层目标态** |
  |---|---|---|---|
  | 幂等（I-4.4（意图只描述目标态）「只描述目标状态」） | 只有写成目标态才幂等 | 天然幂等 | **天然幂等** |
  | 重放要不要分配器 | **要** | 不要 | **不要** |
  | 记录大小（D16（发布语义）「小、顺序写」） | 小 | **大**（要带全部字节） | **小** |
  | 记录核对器（D13（验证路线） 已定项 7）核对一条记录要什么 | 自行实现树语义 + 记账 + 分配合法性（核对从句法变成语义） | 逐字节比对 | **格式解析 + 校验和 + 根环择新** |

- **否掉逻辑意图的第一条理由（独立成立）**：O2（独立解析器 + checker） 明令不许调用被测 crate 任何代码（D13（验证路线）），而逻辑意图的重放要重新实现分配器 + 树操作 + 校验和 + 记账，**约等于第二个文件系统**。第二条理由（落点不确定使核对退化）是第一条的一条封堵，不是独立理由（已定项 5）。
- 记录形态与 D13（验证路线） 的验证手段互相挟持：验证手段的可行性决定记录形态，记录形态决定验证手段的产出等级，两者要一起定。

**射程**：管记录里装什么、分配与记录谁先；施加一条记录在指针层上换哪几个字段是已定项 15，其余归 C284（施加一条记录在指针层上做什么没有定义）；重放幂等这条硬要求在已定项 3。

**依据**：

- 无实验：方向由 2026-08-28 三方论证按幂等、重放要不要分配器、记录大小、核对要什么四条判据推出（主 agent 裁决），四条都是结构性质，没有可量的对照；ZFS 的 ZIL 靠「把主写路径再调一遍」回避独立实现的成本，是线索，不是依据，在变更史。

**欠**：C284（施加一条记录在指针层上做什么没有定义）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:547-567`（整段抄，未转述）**

```markdown
#### 已定项 21：世代号：记录粒度的全局单调号，不是 D5（快照 / 空间记账机制） 的 txg

**定案**：**记录头要两个计数字段，职责不同、不可合并**：

| 字段 | 粒度 | 用途 |
|---|---|---|
| `jsn` | **每条记录**，全局单调，10 字节（实例代号 32 位 + 计数器 48 位，已定项 9）| D22（单元原子性怎么合成） 要求②的世代号；前缀判定靠它 |
| `checkpoint_txg` | 每个 checkpoint | 归属哪个 checkpoint，供 D5（快照 / 空间记账机制） 的 birth/death 记账 |

- 理由：D5（快照 / 空间记账机制） 是「一个 checkpoint 一个 txg 号」⇒ 同一 checkpoint 窗口内的多条 journal 记录**共享同一个 txg**，彼此分不出先后，D22（单元原子性怎么合成） 要求的「被实际检查的世代号」就失去意义。
- ⚠️ jsn 取到不回绕的宽度，**买到的是「免掉回绕比较」，不是新能力**：jbd2 的 tid 是 32 位且会回绕，靠 `tid_gt` 的有符号差比较；回绕比较正是独立实现最容易实现得不一样的地方。
- ⚠️ **「用 txg 就不需要 cycle」只对轴①成立**，见 D22（单元原子性怎么合成） 那两个正交轴的表。

**射程**：管两个计数的职责；两者在水位与择根里谁为主比，归已定项 14 第 2 条；「每次发布 checkpoint_txg + 1」归 D16（发布语义） 已定项 6。

**依据**：

- 无实验：职责划分由 D5（快照 / 空间记账机制） 的粒度直接推出，是定义之间的推导，没有可量的量。

**欠**：C85（发布计数器的四个等号没写死）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:568-579`（整段抄，未转述）**

```markdown
#### 已定项 22：前缀判定：严格连续、断号即止

**定案**：**前缀判据是 `jsn == expected`，不等即止。** 只用「`jsn` > 上次已应用」过滤**不够**：若序列里有空洞（5849 写成、5850 没写、5851 写成），「>」会把 5851 错接到 5849 后面，**破坏因果序**。还要一个**显式的在飞记录数上限**（取值与语义见已定项 18）。前缀判定的完整口径（五条）在已定项 14。

**射程**：⚠️ 断号即止在**那一次**恢复里是对的（前缀停在 5849），它单独挡不住**上一条时间线的残留**：恢复之后接着写时，5851 那个槽里还躺着上一条时间线的记录，**校验和是好的、`jsn` 恰好等于 expected** ⇒ 第二次崩溃后的恢复把它接上去重放。在 D23（journal 的角色与格式）唯一测过的那种槽位映射（`slot = jsn % 槽数`）下这由构造必然发生，位置与序号解耦时退化成巧合，而解耦救不了。现有的三道闸（I-8.3（重放前缀严格连续）、在飞记录数上限、同一窗口里一模一样的 `checkpoint_txg`）都不触发；tail 推进不是出路（tail 只在 checkpoint 持久之后前移、残留从未被应用 ⇒ `tail ≤ min(残留 jsn)` 恒成立）；重放幂等也救不了（它是自反的：「**同一条记录**被重放两次与重放一次，结果相同」，残留是**另一条**记录，它点名的那些单元在被丢弃之后可能已被新时间线重分配）。这一格由已定项 8 / 9 / 10 挡：反向链、序号高位带实例代号、链值覆盖实例代号；恢复时物理作废残留槽要求原地覆写，对 zoned 不成立。

**依据**：

- 无实验：「>」会把空洞之后的记录错接到空洞之前，这是序列定义上的反例，不需要量；残留那一格的实验在已定项 8 的依据里。

**欠**：C42（残留记录冒充合法前缀）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:580-600`（整段抄，未转述）**

```markdown
#### 已定项 23：三个死锁各自怎么破

**定案**：journal 相关的三个死锁，两个落在格式上、事后补不进去：

| # | 环 | 对本工程 |
|---|---|---|
| 1 | journal 满 → checkpoint → 发根 → **发根要不要写 journal** | D16（发布语义）那句「不绕过 journal」取读法甲（已定项 25）：发根本身不是一条 journal 追加 |
| 2 | **COW 独有**：checkpoint → COW 写新节点 → 要分配 → 盘满 → 发不出根 | **保留池治这一条**，`reserve ≥ ckpt_cost` 即可（已定项 24）。⚠️ **D22（单元原子性怎么合成） 的 K 加重它**：盘快满时最该救命的空间正是被 K 扣住的 ⇒ **K 必须是运行时策略，不能是格式常量** |
| 3 | 单条操作大于环 | 需要等价的几何不变量：**任一操作的最坏 journal 占用 ≤ 环大小 / F，F ≥ 2**（[invariants.md](../invariants.md) I-8.1（环几何够大）），mkfs 时算、mount 时校验 |

- **死锁 3 推出一条依赖关系**：**D8（核心索引结构） 的意图分批机制不是「删大文件的便利设施」，它是 journal 满死锁的必要条件之一**——算不下的操作必须能拆成若干个各自合法的事务。

**射程**：管这三个环怎么断；保留池与准入的分治在已定项 24，读法甲在已定项 25，环几何的具体取值在已定项 18 / 19。I-8.1（环几何够大） 的「占用」是空间量纲，允许一个事务跨多条记录（已定项 7）。⚠️ I-8.1（环几何够大） 的 checker 未实现。死锁 2 是 COW 独有的：原地写的 jbd2 / XFS 回家不需要新空间，没有这个环。

**依据**：

- E23（journal 几何）：定长环有死锁 3、要几何不变量，环太小时强制 checkpoint 的次数断崖式上升；链式没有死锁 3，但每个 journal 块都要向分配器要块、进入死锁 2 的环。
- E19（defer 窗口下的假性 ENOSPC）：保留池恰等于一次 checkpoint 的开销就把 checkpoint 卡死打到 0——死锁 2 由保留池断开。

**欠**：C28（环几何不变量未实现）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:601-625`（整段抄，未转述）**

```markdown
#### 已定项 24：死锁 2 与假性 ENOSPC 是两种故障，要分治

**定案**：

| 故障 | 谁治得了 |
|---|---|
| **checkpoint 卡死**（D23（journal 的角色与格式）死锁 2） | **保留池**，且 `reserve ≥ ckpt_cost` 就够；本工程的 ckpt_cost 按 D28（挂载期承诺量） 已定项 4 现算（各记录树高之和 + 记账节点），不设常数 |
| **假性 ENOSPC** | **只有准入**。保留池反而加重它 |

- **机制**：保留池是给 checkpoint 开的一道地板，对**普通分配**是纯税——前提是保留池已经大到不卡死；前提之外符号是反的，卡死之后空间根本不回收，小保留池反而拒绝得更多。而假性 ENOSPC 的成因是「defer 窗口与根环 K 扣住的量超过剩余空间」，**那不在保留池的射程内**。
- **假性 ENOSPC 的破法是一条可计算的准入规则**：

  > **剩余空间必须 > 每 checkpoint 搅动量 × (延迟代数 + 1) + 一次 checkpoint 的开销**

  ⇒ 不许把盘填到 `容量 − 搅动量 × (延迟+1) − checkpoint 开销` 以上。这与 `.claude/rules/fs-design.md`「准入控制要在进门前先算最坏情况」同构——**「最坏情况」里必须含 defer 窗口与 K 扣住的那一块。**

**射程**：保留池怎么进准入不等式、算不算挂载期承诺量归 D28（挂载期承诺量） 已定项 4，准入不等式的权威形式在 D28（挂载期承诺量） 已定项 1。⚠️ **搅动量在真实负载里是变量**，建模时当常数；怎么估它是真机压测那一半的问题。

**依据**：

- E19（defer 窗口下的假性 ENOSPC）：预留恰等于一次 checkpoint 的开销就把卡死打到 0；再加大预留只让假性 ENOSPC 按固定步长上涨；那条准入规则的预测与实测逐格一致。
- E25（环与链各要多大保留池）：「保留池对普通分配是纯税」只在保留池大到不卡死时成立，卡死区间里小保留池反而拒绝得更多。

**欠**：C83（提交固定点没人回答）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:626-648`（整段抄，未转述）**

```markdown
#### 已定项 25：D16（发布语义）「不绕过 journal」取读法甲

**定案**：D16（发布语义）那句

> 「没有任何操作可以绕过 journal 直接发根。」

**取读法甲**：

| 读法 | 含义 | 结果 |
|---|---|---|
| **甲（采用）** | 任何最终反映到根里的**语义效果**必须先经由 journal；**发根这个动作本身不是一条 journal 追加** | 断开死锁 1 |
| 乙（字面） | 发根也要走 journal ⇒ 发根要追加记录 ⇒ journal 满时发不出根 | **环闭合，死锁** |

⇒ 它与 D22（单元原子性怎么合成） 兼容：**根环是 journal 之外的固定 K 槽，发根 = 覆写根环的一个槽，不是追加。** checkpoint 不是「操作」，是 journal 机制本身的一部分；普通用户操作仍然只能进 journal。

**射程**：只管 D16（发布语义）那一句怎么读；发布的其余语义归 D16（发布语义）。

**依据**：

- 无实验：这是一句措辞的读法裁定，两种读法里一种直接闭合死锁环，没有可量的量；jbd2 / XFS 的破法（释放空间的路径消耗零日志空间）与读法甲相容，是线索，在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:649-665`（整段抄，未转述）**

```markdown
#### 已定项 26：记录头的两条解析纪律

**定案**：

1. **校验和覆盖的字节范围必须是显式的盘上常量，绝不能由语言的 `sizeof(struct)` 推出。** 独立解析器不会碰巧复制同一个 `sizeof`，这正是 checker 能抓、单一实现抓不住的错误类型。
2. **记录自述长度必须先做边界检查，再用于校验和计算。**

反向链那一条（让记录核对器能用与正向扫描完全不同的遍历复核同一个前缀边界）是已定项 8。

**射程**：管解析 journal 记录头的次序与常量来源；覆盖范围本身（整条 `[0, 4096)`）在已定项 13；已定项 1 的 B 条（自述长度，先边界检查再算校验和）就是第 2 条。

**依据**：

- 无实验：两条都是从现役实现的缺陷现查出来的纪律（XFS 因为用了 `sizeof`，i386 与其他架构算出不同 CRC，至今背着 `XLOG_REC_SIZE` / `XLOG_REC_SIZE_OTHER` 两个常量和两遍校验的兼容疤；ZFS 的 ZILOG2 从块内读 `zc_nused` 决定校验范围，读之前先 `if (size < nused) return ECKSUM`），约束的是解析次序与常量来源，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:1-4`（整段抄，未转述）**

```markdown
## D16 发布语义 —— 已定

D16（发布语义） 管发布这一格：一个新根什么时候发、一次发布按什么顺序落盘、发布计数器是哪一个数、这一批释放什么时候回可分配集合、新实例的第一个根怎么先覆盖两块盘、一次空发布写不写单元、journal 记录格式在 D15（格式冻结政策） 的冻结分层里算什么。不管的：journal 的记录字段、环几何与重放前缀在 D23（journal 的角色与格式）；根环参数、根记录与系统配置的字段表在 D22（单元原子性怎么合成）；快照的记账模型与生死边界表在 D5（快照 / 空间记账机制） 与 D6（快照实现模型）；准入不等式与挂载期承诺量在 D3（空间分配） 与 D28（挂载期承诺量）；一次发布事件内部的内存序在 `litmus/commit-publish.litmus`，那份钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。

```

**出处 `.claude/kb/decisions/16-发布语义.md:27-74`（整段抄，未转述）**

```markdown
#### 已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读

**定案**：回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算；同一个状态只数一次（去重的判法在「「非空」从盘上怎么认」那一段）。**根环容量是这句的边界**：两个状态来回交替超过根环一圈时，环里的根全是这两个状态，保留的就只有环里有的那几个，任何机制都给不出环里没有的状态。**窗口的单位是发布，不是秒**：defer 窗口保护的是「环里的根还能不能用」，普查过的消费者没有一个按秒计价。形态取 E139（按盘回退下界的收严形态） 的臂 G2S，与装置里跑的一字不差：

| 项 | 定案 |
|---|---|
| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |
| 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算 |
| 回退下界 F | 根记录带 8 字节（D22（单元原子性怎么合成） 已定项 7，242 → 250），系统配置另记一份 8 字节（D22（单元原子性怎么合成） 已定项 9）。平时不动，窗口就是整个根环；准入不够时抬，正常卸载时抬到现行那一版的 txg |
| 抬 F 的上限 | **准入抬 F**：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)。**卸载抬 F**：抬到现行那一版的 txg，不另判上限（等价于上限取最新的持久有效根，不按盘取小） |
| 根槽这一次读坏 | 算准入抬 F 的上限、挂着时管理员回退判候选（根环里有没有目标）与算 F_生效 时读根环：挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，当没有根；挂载那一刻读得出、或这个进程写过且 FUA 返回过的槽，这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退、报它自己的错误成员，不按有根或没根猜（抬 F 那一半用户 2026-09-26 定，推的，被攻过零轮；回退那一半主 agent 2026-09-28 定，被攻过一轮：代码三方第三轮判决 `research/prompts/m2-closeout-code-r3-main-verification.md` 第一节判替没写的条款做了选择、回退扩写拒错或放错那一格没打中（一次抽样））。可写挂载的择根按 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续处置（D23（journal 的角色与格式） 已定项 14：系统配置见证过比所选那一版新的发布就把读阶段重读一次，仍判真拒可写）；只读挂载的择根、「环里最旧有效根」怎么处置这一格没判，见 C563（读根环一槽读坏就当没有根，别的调用方没判） |
| 生效 | F 另记进每块盘的系统配置（「抬 F 那一串」那一行写明先写后抬）；恢复后生效值 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 系统配置里读得出的 F 的最大值)；系统配置里读得出的取每块幸存盘两槽里全部自证过（校验和过且 fsid 与本池相同）的槽，与取号式子同一取法（D18（块里携带什么信息） 已定项 11） |
| 回退候选集 | 按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的挂着时回退取同一个集合） |
| 抬 F 那一串 | 准入与卸载共用。动盘之前在分配器的拷贝上把这一串空发布整串预演：任一次取不到落点，就一次都不发、分配器原样换回抬 F 之前那一份（`MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`）；**预演过了、第一次发布之前，先把新 F 写进每块盘的系统配置、过一道屏障**，之后才发第一条带新 F 的根；**先写那一步任何一块盘失败，这一串的根一条都不发、返回错误；已写进别的盘系统配置的新 F 留着、不回卷，F 的生效值只增不减**（主 agent 2026-09-26 定，推的，被攻过零轮）；空发布推到每块盘上都有一条带新 F 的根为止；真发时落盘途中写错，交回已落盘几次（`MountError::RaiseFloorSequencePublishFailed`），那一格里扣住的槽怎么放没有条款，见 C546（抬 F 被拒时扣住的槽不退回） |
| 正常卸载 | 推一串空发布把 F 抬到现行那一版的 txg（「抬 F 那一串」照准入的走法，上限见「抬 F 的上限」那一行）；这一串写的根在根记录 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）；第一版几何（R = 3、区域归属 0 / 1 / 0）下空发布 2 或 3 次（现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次）；不加干净关闭标记，下次挂载照旧走恢复（D23（journal 的角色与格式） 已定项 14） |
| 准入 | 可分配 = min(可再分配 + 活元数据 − 保留池, `df`)；准入不够、或准入放行而落点取不到时，先推空发布抬 F 再判（写行那次发布之前不推：它是新实例的第一次发布，元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC）；**可写挂载的准入不够时，写行那次发布照走切换预留，写完行之后推抬 F 的空发布、再判**；一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC；挂载处推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常（D2（RAID 条带策略） 已定项 13 那一格的例外）（用户 2026-09-26 定，被攻过零轮） |
| `df` | 没用过的 + 已释放的 + 活元数据 − 保留池 − 推空最坏残留 − 滞后量；保留池 = 2 × 5 + (B − 1) × c_max = 10 + 7 c_max，残留 = 5 + (B − 2) × c_max = 5 + 6 c_max；滞后量 = 释放代 > max(第 4 新的非空有效根, 环里最旧有效根) 且不是空发布放掉的已释放块 |
| 环深下限 | 每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随之抬） |

**「非空」从盘上怎么认**：一条有效根算非空 ⟺ 它的树表里用户可见的树（inode 树、extent 树）的根指针，与它前一条有效根（按 txg 排；有效 = 按实例表判仍然有效 ∧ txg ≥ 当前的 F）树表里的不同。比的是树表里这两棵树的条目、不是树表单元自己的落点——每一次发布（连空发布）都重写树表单元（`crates/singlefs-core/src/transaction.rs`），按落点比会把空发布全算成非空。**数「第 N 新的非空有效根」与「4 个不同状态」时去重**：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入——管理员回退发出的新根与它前一条有效根不同、按「「非空」从盘上怎么认」那一判法算非空，而它的状态与更早的某一条相同，不去重就会把同一个状态数两次（第二轮 X7：来回回退之后按内容只剩 2 个不同状态）。反方向只让 F 停住，直到第 4 个状态的根离开根环。

**改新池新建文件的字节：是**：根记录加 F 8 字节；新池新建文件里 F 恒 0（mkfs 与第一次发布时非空有效根不足 4 个，上限取最旧有效根 txg 0）。系统配置加 F 8 字节，新池新建文件里写 0（D22（单元原子性怎么合成） 已定项 9）。

**连带改的**：D22（单元原子性怎么合成） 已定项 7 的根记录加 F（242 → 250）、flags 占一位卸载记号，已定项 9 的系统配置加 F 8 字节，已定项 2 的 S 下界；D23（journal 的角色与格式） 已定项 14 的回退候选集加「txg ≥ F_生效」；D3（空间分配） 已定项 9 第 2 条写明界；I-7.4（近 K 代块未被复用） 的 K 改由 F 表达；I-7.9（回退下界 F 不高于抬 F 的上限） 按卸载记号分两支。

**射程**：定的是 defer 窗口与回退下界怎么算，不定环几何、不定 c_max 的口径（c_max 是声明的最坏空发布开销，按 D28（挂载期承诺量） 已定项 4 按当时结构现算）。已知边角七样：

- **「生效」取 SysPre**：F 只住根记录、根槽不镜像（已定项 7）时，带新 F 的根坏掉 F_生效 就回落，更早的根重新进回退候选集、而它们引用的块已被合法复用——I-7.4（近 K 代块未被复用） 在这段历史上不成立（`research/prompts/m2-step45-code-r2-main-verification.md` 第四节、`research/prompts/m2-step6-checker-r1-main-verification.md` 第四节）；病根是 F 只住根记录，与生效取哪条规则无关（第三轮 K4：带 F 的根全坏那一格，今天的规则、MAX、MAX+HOLD 候选坏 12 / 140 / 12，SysPre 0）。代价：系统配置多 8 字节（动格式）；每次抬 F（准入与卸载）每块盘多一次系统配置写、加一道屏障；checker 与层 0 要跟着读系统配置里的 F——今天的 checker 按根上带的 F 判，带 F 的根全坏时会误红 I-7.4（近 K 代块未被复用） 等五条（第三轮 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。「先写后抬」是第三轮攻方提的，被攻过零轮，用户 2026-09-26 知情定。被抛弃时间线上的根带的 F 算进 F_生效（用户 2026-09-28 定「算进」，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M2：被抛弃根带着比别处高的 F 时，不算的话 I-7.12（系统配置 F 不低于同盘根上的 F） 红、回退退得到那个 F 之下，而被抛弃时间线抬 F 时已按它回收过槽；实现 `crates/singlefs-core/src/recovery.rs` 的 `effective_rollback_floor_of_the_roots_read`，checker 另算一份 `crates/singlefs-checker/src/walk.rs`）；C331（择根倒挂压过已确认的写） 取甲之后，被抛弃根带着更高 F 的盘面只在构造的镜像上有（那一形要可写挂载时一槽系统配置一时读不出，甲 拒这样的可写挂载）。与 `research/prompts/alloc-basis-r3-main-verification.md` 的 F 口径岔路一起判的那一件仍开着。
- F 回落之后第 0 代根重新进有效根、它的树表单元已被合法复用、此后每一次抬 F 都被拒到根环把它盖掉为止（`research/prompts/m2-emptypool-nonempty-r1-main-verification.md` 第一节 Z1）：前提是 F 回落，取 SysPre 之后在带 F 的根全坏那一格上不再出现（推的，第三轮没造那一形）。一次挂载里连续发布超过根环容量时挂载内不回收，checker 在合法状态上判 I-3.1（已分配统计对得上） 红（E153（账本形态与环上有洞的代价） 在 `crates/` 副本上量到），仍开着，归里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ② 行。
- **正常卸载的代价**：每次卸载多 2 或 3 次空发布，第一版几何上每次 8 个单元、两盘合计 21 次写调用、279040 字节（第一轮攻方 t10、t5a，副本上量的）；卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版。用户选时知情的代价是「停机时崩溃兜底只剩最新一条根」（按条款推的）；第二轮在卸载之后再挂、最新 j 条根坏的 75 行上恢复全读对、可写挂载全挂上，这一条没出现：F 之下的根还在环里，它们的记录能把状态重放回这一串的末尾（第二轮判决 G4）。
- **用户知情接受的代价三条**：删掉的空间要再发生 3 次改变用户可见状态的发布才回可分配集合，在那之前 `df` 不把它算空闲、按 `df` 写它是真 ENOSPC；`df` 少报 = 保留池 + 残留 = 15 + 13 c_max（c_max = 5 / 10 时 80 / 145 块）再加滞后量，而滞后量 = 4 × 最近四个窗口的释放量、**没有上界**（一窗删一个 8 块对象时平均约 34 块、最大 44 / 52，一窗删 32 个时 1044 块），删掉 s 字节 `df` 一个字节都不涨；c_max 按现算取，第一版池规模 9 块 ⇒ 保留池 73 块。
- **最少保留 4 个状态只在一次处置里根槽写失败 ≤ k_tol 次时兑现**：S = 4、c₀ = 5、3 次失败那一格违反 24 次，按字面记一次输、按 k_tol 的射程在外；全枚举下 S = 4、c₀ = 1 也违反 160 次（同为超过容忍的格）。
- **代价是选项本身的，不是这一族的缺陷**：一个被保留的状态钉住的是在它那一刻活着、后来才死的全部块，选最近 4 个就是每次删除都撞上——滞后量与「删了要等 3 次状态」都从这里来。
- **E139（按盘回退下界的收严形态） 答不了的**：计数模型，按发布计不按墙钟；只建根一级的引用、没建 journal 记录重放；实例切换的挂载时预留由 D28（挂载期承诺量） 已定项 3 扣；容量 4000 块、两块盘；k_tol 固定 2；滞后量口径是写登记的人替状态数读法构造的；保留池扫描的最小值落在扫描下沿、真正要多少没量到；环深下限是探针的数。两轮前身 E135（动态回退下界） 与 E138（按盘回退下界与推空的空间要求） 的结论已被 E139（按盘回退下界的收严形态） 取代，不再当依据用。

**依据**：

- E139（按盘回退下界的收严形态）：臂 G2S 在全部故障世界（活化区间内崩溃不丢盘、先跳号再抬 F、推空期间 1..3 次根槽写失败、坏槽、开销升到 c_max）零假候选、零不可恢复，各格按自己的 `df` 假性 ENOSPC 为 0，一次准入的发布数实测最大 7 ≤ 8，跑前保留池下不卡死——定案取的就是这条臂（跑前登记 `e139-preregistration.md`，产物 `e139-tightened-floor-2026-09-12.out`）。
- 四轮三方论证，第四轮判决 `research/prompts/d16-item1-r4-main-verification.md`；第三轮 `research/prompts/d16-item1-r3-main-verification.md`。
- 用户三次定案（候选集可以一直缩、最少保留 4 个、「4」数的是可退到的不同状态、三种「哪 4 个」里选最近 4 个），原话在变更史；「非空」怎么认与「写行那次发布之前不推」同为用户定案，原话在变更史。
- 用户定案 2026-09-25：抬 F 那一串改成整串预演，推翻 2026-09-24「维持逐次发布」（原话在变更史）；实现是实二六（`research/prompts/m2-e26-implementer-report.md`）。
- 生效取 SysPre、卸载抬 F 到现行那一版的 txg（B1）、「4 个不同状态」去重与根环容量边界、候选按现行那一版的实例表判：无实验，三方原型（`research/prompts/m2-rollback-forward-r1-opus-model/`、`m2-rollback-forward-r2-opus-model/`、`m2-rollback-forward-r3-opus-model/`）在冻结副本上量过，没立实验号，原型上的数不是入库装置上的数；三轮判决 `research/prompts/m2-rollback-forward-r1-main-verification.md`、`m2-rollback-forward-r2-main-verification.md`、`m2-rollback-forward-r3-main-verification.md`（第三轮第三节站住的、第四节交用户的）。SysPre 的「先写后抬」被攻过零轮。系统配置里读得出的 F 取每块盘两槽里全部自证过的槽、先写那一步失败时一条根都不发且已写进的 F 不回卷，这两处是主 agent 2026-09-26 定的，推的，被攻过零轮。
- 用户定案 2026-09-25（正常卸载时抬 F、取 B1）与 2026-09-26（C419（F 生效之后靠什么保持不回落） 取 SysPre，推翻 2026-09-25 定的「生效值取各盘 F 的最大值」；「4 个不同状态」加根环容量边界），原话在变更史。
- 「根槽这一次读坏」那一行：抬 F 那一半是用户定案，原话在变更史，构造的反例在 `records/2026-09-24-里程碑二收尾调度.md` 第三节「实四丙交回」那一行；回退那一半是主 agent 2026-09-28 定（同一份调度记录第 309 行），经过在变更史。实现已合入：`crates/singlefs-core/src/recovery.rs` 的 `readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once` 对这个进程知道住着根的槽「读不出或自证不过」都重读一次，仍坏报 `RootRingSlotKnownToHoldARootStillBadAfterOneReread`（带 `first_reading` / `reread`）；算上限那一遍是 `crates/singlefs-core/src/mount.rs` 的 `roots_of_the_ring_for_the_ceiling`（仍坏报 `MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），抬 F 与卸载重算影子账与回收门槛那一遍仍坏报 `MountError::NewerStateStillUnreadableAfterOneReread`（`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`），管理员回退的 `rollback_candidate` 判候选与算 F_生效（`recovery.rs` 的 `effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`）仍坏报 `RollbackError::CandidateJudgementStillUnreadableAfterOneReread`、在任何写之前。用例：`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的 `raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_not_self_verified_is_refused_before_any_write`、`a_root_ring_slot_known_to_hold_a_root_not_self_verified_once_is_read_on_the_reread_and_the_floor_is_raised`、`rolling_back_while_the_targets_root_ring_slot_stays_unreadable_is_refused_before_any_write`；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 的 `rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write` 与 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 的 `rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write`（这个进程写过的根被改坏之后回退拒、盘上不变）（`research/prompts/m2-impl-r2-fixes-b-implementer-report.md` 第三节第 3 行；合入后验证一收窄成只管读不出、之后由改法 B 还原成全句，见 `research/prompts/m2-impl-merge-verify-1-implementer-report.md` 第三节第 1 小节）。
- 「准入」那一行的落点被拒也推、挂载处写行之后推：用户定案，原话在变更史；安全设计轮第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`（会话里落点取不到 C283（准入失败时不先推发布就报 ENOSPC） 不管、只崩了再挂就把小盘挂死）。

**欠**：C282（环里最旧根没有定义）——定义与条款写在已定项 1 里，会红的检查仍欠；C314（回退可以复用被抛弃的根引用的单元）（与 defer 窗口取多大无关，病根在 D23（journal 的角色与格式） 已定项 14 的影子账）；C329（写行那次发布之前推抬 F 的空发布没有检查）；C563（读根环一槽读坏就当没有根，别的调用方没判）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:75-87`（整段抄，未转述）**

```markdown
#### 已定项 2：checkpoint 的触发判据取时间与脏数据量，两者取先到

**定案**：`触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)`。两个阈值是**系统配置声明值**，挂载时求值一次（形态与 D21（权威态与派生态的分界）「单元几何挂载时求值一次」同型）。

**射程**：定的是什么时候触发，不定两个阈值取多少（已定项 5），也不定触发之后怎么切分事务（同样在已定项 5）。已知边角一样：`.claude/rules/fs-design.md` 硬要求 5 说的分支变量是那两个阈值、它们在挂载期间恒定，`now` 与脏字节数是被比较的量、会变，不是分支变量。

**依据**：

- 无实验：`∨` 这个形态是两条失败模式互相堵出来的，没有可量的量——只按脏数据量，空闲系统上最后一批写可以无限期不落、崩溃丢窗口无界；只按时间，突发写会把一个 checkpoint 的 journal 占用撑过环几何，而 I-8.1（环几何够大） 要的是「任一事务的最坏 journal 占用」有界。那个 `∨` 式子就是全部，没有魔数以外的自由度，`.claude/rules/fs-design.md` 硬要求 1「判据显式、可计算、写下来」因此满足。可量的量是两个阈值的取值与它们的可行域，在已定项 5。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:88-108`（整段抄，未转述）**

```markdown
#### 已定项 3：journal 记录格式是四层之外的第三个独立冻结组件

**定案**：journal 不进 D15（格式冻结政策） 的四层链。它的记录格式——记录头（含已定待落地的事务号 + 提交标记、反向链、载荷校验和三笔增量）、点名项、载荷校验和的覆盖范围、事务边界——立成四层之外的第三个独立冻结组件；住系统配置的 journal 几何字段（记录尺寸、环大小、在飞记录数上限、tail 槽）是系统配置字段，随第 1 层冻。新记录格式走「新记录类型 + 系统配置 log-incompat 位」：环里有这种格式的记录时置位、环干净时清掉，不认识就拒绝挂载；永久要读得懂的只有判定「这条记录不施加」要用的那几个头字段（magic、类型、自述长度、`jsn`、`checkpoint_txg`、头校验和）。

**冻结前置**（不早于它嵌进来的每一个外部定义）：第 2 层（点名项里的位置条目）；落点粒度那一格（D3（空间分配） 已定项 7 欠着的处置）；D18（块里携带什么信息） 已定项 11 的单元类登记表（点名项里的单元类型标签）；树 ID 所在的树表单元——它今天没有冻结层归属，所以这一条前置今天就挂着。**冻结时刻是提交 `format-frozen/journal-record.toml` 的那个 commit**，不许写成「第一个写出记录的镜像」。

**射程**：只答 journal。树表单元、实例表单元与根记录的冻结层归属仍是 C246（树表单元没有冻结层归属） 那一格。三样已知边角：

- **置位与清位是运行期动作**，不走提交这条时间线，要另配一道运行期的闸（写第一条新格式记录之前先持久置位，缺位即拒绝写），随组件的 spec 一起落。
- **今天还冻不了**，挡着五样：点名项 56 字节只定了总宽，构成只有 [layout/01-first-txn.md](../layout/01-first-txn.md)「六、journal」的预想表，也没给加密字段留位（D23（journal 的角色与格式） 已定项 1 的 D 条「项的大小由记录类型决定」管着这件事）；记录头登记的格式常量是 78，三笔增量落地后是 95；spec 文件不存在；log-incompat 位还没有位号，也没进 D15（格式冻结政策） 那张 feature bit 记账表；冻结前清单第 1 项量宽度、第 7 项量未定分项，两项都看不见 `jsn` 解释口径打架那笔欠账，按今天的清单冻这个组件时两项都不红。
- **「什么时候算环干净、可以清位」要按本工程的重放下界定**（D23（journal 的角色与格式） 已定项 14 的水位），不照搬现役实现。

**依据**：

- 无实验：冻结层归属是格式政策，能核的是各层之间的依赖顺序，没有可量的量；记录头宽度那笔字节账在 C94（登记的格式常量与后来的定案对不上）。
- 三方论证（材料 `_d16-items134-background.md`，产物同目录 `d16-items134-*`：`d16-items134-sonnet-output.md`、`d16-items134-opus-output.md`、`d16-items134-local-output-s1.md` 与 `-s2.md`）：正推腿判「独立组件」是唯一与 D15（格式冻结政策） 已定项 1 自洽的答案，反推腿补上 log-incompat 位之后演进那一格赢，并打中两处前置；本地腿两次抽样的反例把字段挂错了层，主 agent 核后不采纳。判决取独立组件 + log-incompat 位，冻结前置按反推腿收严。
- 现役参照：XFS 有 `XFS_SB_FEAT_INCOMPAT_LOG_ALL`，在主系统配置里置位、日志里没有受保护的条目时清掉（本机树 `fs/xfs/libxfs/xfs_format.h:426-441`、`fs/xfs/xfs_mount.c:1560`、`:1629-1639`），**未在本项目验证**；一处已知差异：XFS 的日志要重做逻辑与物理混合的条目，本工程的记录只承载指针层目标态。
- 用户定案，原话在变更史。

**欠**：C94（登记的格式常量与后来的定案对不上）、C119（冻结组件没有 spec 文件）（journal 记录格式是第三个，那一行的简称还写「两个」）、C246（树表单元没有冻结层归属）、C199（实例代号递增与 jsn 断号即止互相矛盾）（冻它之前要先还）、C225（组件的冻结时刻绕开门禁触发）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:109-128`（整段抄，未转述）**

```markdown
#### 已定项 4：一次发布整体施加或整体不施加，第二份 replay 停在句法层

**定案**：「施加一条记录在指针层上做什么」那个二选一取「一次发布整体施加或整体不施加」。记录核对器那第二份 replay 实现因此停在句法层：前缀判定 + 择根 + 续做，不实现树语义、不重新生成祖先。与 D13（验证路线） 的关系照收窄后的四样：

| # | 定了什么 | 依据 |
|---|---|---|
| ① | **共享**：与实现只共享 D13（验证路线） 已定项 5 那份生成常量；与 checker **默认不共享**，要共享得另行论证；不与 RefFS 共享恢复代码 | D13（验证路线） 冲突 1 的折中管的是 checker 自己遍历、扫描两个方向之间，不授权两个判决器之间共享（两条云端腿一致） |
| ② | **入参拆成三份**：崩溃态镜像（择根与前缀判定看它）、记录流（崩溃态镜像环里的 journal 记录）、实现恢复后的镜像（比对的对象） | 恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14），D13（验证路线） 定案写的「崩溃后镜像」在 harness 先恢复、后核对的顺序下是两份 |
| ③ | **缺一个数**：前缀第一条要的「在飞记录数上限」本仓没有值（I-8.3（重放前缀严格连续）；D23（journal 的角色与格式） 已定项 1 的 C 条要它进系统配置） | 那个数在本仓零命中 |
| ④ | **合谋测试**：记录核对器是第四个判决器；每个判决器都要有一个只有它抓得住、而且对可见状态或结构有害的坏实现。「摘掉根槽前的屏障」不够格——它抓到的缺失只伤记录核对器自己；候选待找：原来的候选回退行 W 截断随管理员回退改成挂着时的向前发布一起删掉（D23（journal 的角色与格式） 已定项 14），第二轮量到截断在今天的代码上本来就走不到。O3（独立规约执行器） 今天不在 [verification-build.md](../verification-build.md) 的落地步骤里 | 反推腿 4-D |

**射程**：定的是第二份 replay 做什么、不做什么，**不定它要多少人工**——零代码时那个数没有可核的答案，只有 research 里三样工作的模型等价物当下界替身（`e42_transaction_records.rs` 240 行、`e77_publish_order.rs` 506 行、`e78_replay_start.rs` 556 行，反推腿 `wc -l`），第四样（把期望态算出来）一行模型都没有。指针层其余部分（允许部分施加时换哪个指针、祖先谁来 COW、落点谁分配）仍归 C284（施加一条记录在指针层上做什么没有定义）；D8（核心索引结构） 已定项 13（同一次发布里同 key 的中间态折叠）靠这一支：施加只落在发布边界上，同一次发布里的中间态才不会被 replay 出来。

**依据**：

- 无实验：第二份实现必须独立实现什么、允许共享什么，是验证路线的分工，没有可量的量；「要多少人工」在零代码时也没有可核的答案，只有行数下界替身。
- 三方论证之后的用户定案收窄问法（问的是「那第二份实现必须独立实现什么、允许共享什么」加「它在合谋测试里站什么位置」），原话在变更史。

**欠**：C284（施加一条记录在指针层上做什么没有定义）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:129-153`（整段抄，未转述）**

```markdown
#### 已定项 5：`T_time` 与 `T_dirty` 取多少，丢失窗口的口径

**定案**：`T_time` 默认值 5 s，取可行区间 [200 ms, 5 s] 的上端——默认丢失窗口是 5 秒，取更小只减少摊销、不多买任何东西，拦住丢失的是那 5 秒本身。**`T_time` 用户可设，不限制**：挂载时传的值覆盖盘上的字段并回写；超出 [200 ms, 5 s] 不拒绝，只给警告（用户 2026-09-20）。`T_dirty` = 2 GiB（初值，写代码时再调）；它不由丢失窗口定——`∨` 规则下时间那一支已经保证了窗口，`T_dirty` 管的是**量**，它就是脏页内存预算本身。**`T_dirty` 是可调值，不是格式常量**：系统配置里那个字段是格式，字段里放什么数不是。**挂载时还要再夹一次**：有效 `T_dirty` = min(系统配置里那个字段, 环长 ÷ F)；环长是 mkfs 参数（默认 768 MiB、约束环 ≤ 设备容量 ÷ 4），所以「环装不装得下 `F ×` 最坏占用」不由一条 mkfs 时的下界保证，改成挂载时把脏字节预算压下去；默认环下有效值 = min(2 GiB, 256 MiB) = **256 MiB**，系统配置里 `T_dirty` 仍写 2 GiB、字段不动。

**丢失窗口的口径**：取号 = 事务被分配进某个 checkpoint 那一刻，checkpoint 的切分只关闭成员资格——触发之后不再分配事务进它，已分配的事务把各自的单元写完，发布阶段等全部单元落盘（任何 COW checkpoint 都要等最慢的那次 I/O，ZFS 的 txg sync 同样等最慢的 vdev，未在本项目验证）。事务切分纪律是一事务一单元。落到取号上，事务切分纪律把一次写请求按单元切成若干事务各自取号，一个事务最多写一个单元的用户数据，**同一次请求切出的若干事务按其单元的 key 升序取号**（提交路径要记「同一请求内取号序与 key 序不一致的次数」、非 0 判红）。单元写的重试预算是墙钟 T_retry（具名可调参数，不是格式常量；用尽即按失败的性质走实例切换或转只读，D23（journal 的角色与格式） 已定项 14——它同时是丢失窗口的加项、挂载内实例切换的判据、以及连续切换计数的时间基准）。**由此崩溃最多丢的是 T_time + N_switch × (T_retry + T_redo)，不是恰好 5 秒**——一次挂载允许 N_switch 次实例切换，每次都要熬完 T_retry 再把在飞 checkpoint 重做一遍（T_redo），而重做期间开放的那个 checkpoint 发布不了、只能一直攒。这里的 `T_time` 是本次挂载生效的那个值，不是 5 秒这个常数；`T_time` 可调之后，2026-08 用户定案的「崩溃最多丢 5 秒的写」降级成**默认值下的承诺**，不是全局承诺。**「上次 checkpoint 的时刻」在重发时取重发那一刻**（取原来那一刻的话，切换期间触发条件一直成立而在飞深度只有两个，触发只能空转）；在飞 checkpoint 深度 = 开放 + 发布中两个；并发 fsync 的返回时刻被同 checkpoint 的其他事务拖住是组提交固有的代价。这是已定项 14 那条「快照发布点钉在这个 checkpoint 全部数据变更之后」的可执行形态。

**射程**：定的是两个阈值取什么数与崩溃最多丢多少，不定环几何本身、也不定重做期间开放的那个 checkpoint 会不会堵住写者（D28（挂载期承诺量） 已定项 3 点名的那一格口径空白，D16（发布语义） 没写）。七样已知边角：

- **`T_time` 的下界只适用于非 fsync 触发的发布**：那步推导以「一窗多次操作共享一次发布」为前提，fsync 密集负载下窗口由 fsync 自身触发。引用这条下界时连这句一起引；上界 5 s（用户定的丢失窗口）不受影响。
- **「向上取整到 200 ms」与「预算取 0.1」都是选的，不是算出来的**：0.2 给 94 ms、0.05 给 410 ms（外推），下界随预算线性挪，改预算就要重算。
- **由 `T_dirty` 推出的两个派生数按已定记录格式算**：D23（journal 的角色与格式） 已定项 12 的 4 KiB 定长记录（头 95 B、每项 56 B、71 项/条）下，同一个 `T_dirty` 满窗是 1847 条记录、占用 **7388 KiB**，I-8.1（环几何够大） 的环下界 F = 2 时 **14776 KiB**、F = 3 时 **22164 KiB**。两个阈值的定案值不变，变的只是这两个派生数的口径。
- **一事务一单元下的记录账与这个派生数打架**：按六个负载点算，每次 fsync 的记录数等于数据单元数（seq 批 1 是 8 条、批 10 是 80 条），journal 恒占用户数据 12.5%，`T_dirty` 满窗 65 536 条记录、256 MiB，是字节表预想的 64 MiB 环的 4 倍。两条口径原样并列，改法要用户定。
- **本机持续写带宽没有实测**（E58（校验和粒度的端到端代价） 那个 2045 MiB/s 是校验和实验里的顺序口径，不是写带宽，不许拿来换算），所以「5 秒能写多少字节」算不出来。
- **`T_dirty` 与 `T_time` 是两种规则**：`T_dirty` 走夹取——有效值 = min(字段, 环长 ÷ F)，不动；`T_time` 走警告——越界不拒绝，只给警告。同一段里两个字段两种规则，不按同一规则读。
- **`T_time` 默认值后期按盘给推荐**（用户 2026-09-20）：要先有一批盘的实测与探测输入，落在 D12（目标介质） 那条线上（第一条布局线是纯 SSD）。记成 `T_time` 默认值的重开条件，不是现在要做的事。

**依据**：

- E70（checkpoint 两个阈值的可行域）：两个阈值的可行域——`T_dirty` 的上界绑环几何但不承重（64 MiB 环就允许 16 GiB 脏数据，实际卡住它的是脏页内存与崩溃丢失窗口），`T_time` 的下界由摊销预算反解，按最差负载 multistream 定而不按主负载 seq 定，取摊销预算 `B = 0.1` 块/操作 ⇒ 反解 547 次操作 ⇒ 197 ms、向上取整记 200 ms，seq 负载一点下界都不需要。
- E44（序号位宽的本机实测与代价）：本机 2785 fsync/秒，是把摊销预算反解成 `T_time` 下界的那个速率。
- E91（已定格式下的环账与准入账）：按已定记录格式重算 `T_dirty` 满窗的 journal 占用与环下界，旧的 32 B/条目口径少算 1.80 倍。
- 用户定案三次（两个阈值、丢失窗口口径的收口、挂载时再夹一次），原话在变更史。

**欠**：C310（事务切分纪律与记录数口径打架）、C319（请求内单元按 key 升序发出没有条款也没有检查）、C425（T_time 小于一次发布耗时时触发空转）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:154-176`（整段抄，未转述）**

```markdown
#### 已定项 6：发布计数器，四个等号写成一条

**定案**：每次发布把 checkpoint_txg 加一——fsync 触发的发布也是发布，没有「小发布不记号」的例外；根槽的轮转键（D22（单元原子性怎么合成） 已定项 2 的「区域 = txg mod R」里那个 txg）、记账的代（D5（快照 / 空间记账机制） 已定项 2）、恢复重放的水位（D23（journal 的角色与格式） 已定项 14）都是这同一个计数。不另设「发布代号」字段——它与 checkpoint_txg 是同一个东西。**四个等号**：fsync = 提前发布、区域 = txg mod R、一个 txg 一个号、记账代 = checkpoint 号，四处读的是同一个数，哪个实现读岔一处就会撞根槽或读错账。

**四条骑手条款**（与定案同日采纳，不是可选项）：

1. **根记录必须带 32 位实例代号**（与 D23（journal 的角色与格式） 已定项 9 的 journal 实例代号是同一个计数），择新在 checkpoint_txg 平局时按实例代号高者赢。机制：丢盘回退 + 盘失而复得会造出两条同 txg 的自证合法根——journal 靠实例代号分辨时间线，根环此前什么都没有。字段落点在 D22（单元原子性怎么合成） 已定项 7。
2. **记账 key 的「代」段宽度 ≥ 48 位**：按本机 2785 发布/秒，32 位 17.8 天回绕。记账 key 的宽度算术自陈是假设，要按此重算。
3. **`T_time` 下界的推导（197 ms）只适用于非 fsync 触发的发布**——那步推导以「一窗多次操作共享一次发布」为前提，fsync 密集负载下窗口由 fsync 自身触发。引用已定项 5 的下界时连这句一起引；上界 5 s（用户定的丢失窗口）不受影响。
4. **记账条目的跨发布合并不再可行**（代进了 key，「后者胜」并不掉不同代的两条）⇒ 每代「统计量数 × 2」次写就是实付值，不是上界。代价，照实记。

**射程**：定的是这个计数怎么加、四处怎么共用一个数，不定根环几何本身（D22（单元原子性怎么合成） 已定项 2）、也不定 journal 实例代号什么时候递增（D23（journal 的角色与格式） 已定项 9）。已知边角一样：这条定案当天的对抗测试实际跑在两条腿上——本地腿那一份后来被收窄后的字词损坏检查判红（`proposed proposed`，实词自复读那一类），按 `.claude/rules/three-way-inference.md`「闸判红之后那一轮作废」，**不许再拿它当「三方一致」引用**；定案本身不因此推翻，两条云端腿各自的判定性产出仍在。

**依据**：

- E48（根环几何的可行点）：轮转键就叫 txg 且逐次发布递增，9 个可行点全部建在这个读法上。
- E50（根环槽数的上下界）：源码逐字「一圈时长（秒）= 槽总数 ÷ fsync 率。每次 fsync 发一次根」——已入库的根环算术只在「每次发布 checkpoint_txg 加一」这条等式下为真，不钉它那批实测追溯性作废。
- E53（丢一整块盘之后根环还挂不挂得上）：丢盘回退 + 盘失而复得的那条历史，就是骑手条款 1（根记录带实例代号）要分辨的东西。
- E44（序号位宽的本机实测与代价）：本机 2785 发布/秒，32 位 17.8 天回绕——骑手条款 2 的「≥ 48 位」是从这个速率来的。
- 对抗测试（用户要求先攻再定）的原始输出 `c85-publication-counter-forward 那一轮的腿报告（产物在 git 里）`、`c85-publication-counter-reverse 那一轮的腿报告（产物在 git 里）`、`c85-publication-counter-local.out`；用户定案，原话在变更史。

**欠**：C85（发布计数器的四个等号没写死）（条款已写成一条，会红的检查仍欠）、C88（根环的时间线判别未实现）（骑手条款 1 的 checker 侧，反推腿那一轮找出来的可达破绽）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:177-193`（整段抄，未转述）**

```markdown
#### 已定项 7：发布的持久顺序

**定案**：一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。**系统配置槽在根槽持久之后再更新，每个 checkpoint 一次**，它是发布序列的最后一步、进层 0 枚举，一次发布的段序列因此唯一。**抬 F 那一串（准入与卸载）在它的第一次发布之前多一步**：先把新 F 写进每块盘的系统配置、过一道屏障（已定项 1「抬 F 那一串」）；这一次系统配置写不属于任何一次发布，每次发布内部的顺序照旧。**fsync 返回条件多一句**：新实例在本实例写成的根覆盖两块盘之前，fsync 不返回（做法与次数在已定项 8）；管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14），它的根持久之后才向管理员确认，与 fsync 同一个返回条件；实现待派，排在实审 A3c 之后（同在 `transaction.rs`），代价另登记一个小实验量。

**射程**：定的是一次发布内部这几步的先后与 fsync 什么时候返回，不定屏障的真机代价、也不定根槽本身的冗余形态（D22（单元原子性怎么合成） 已定项 2）。三样已知边角：

- **「fsync 等系统配置轮换持久之后才返回」的射程**：返回之后这一代的根只由那一个根槽罩着（根槽不镜像），见证它的系统配置已落盘。同一实例里退一代，靠 journal 重放追得上；**每个新实例（每次可写挂载、恢复、切换）的第一个根在下一次发布之前是单点**——那个槽读不出或它所在的盘掉了，恢复退到上一个实例的根，重放按严格前缀停在实例边界（D23（journal 的角色与格式） 已定项 14 的注 1），这次挂载里 fsync 已返回的事务丢，一个故障就够。要不要让新实例先把根写到两块盘上再确认，是已定项 8。
- **屏障口径**：两道 FLUSH + 根槽 FUA + 系统配置轮换之后一道屏障 = 每次发布四个序点，比 D25（目标负载优先级） 推导的两个多两个——多的两个是第二道屏障与「发布返回前加屏障」的价钱，知情接受（后者用户 2026-09-27 定，起因是 C577（系统配置没见证到的最新根，乙罩不到） 那条用例在抬 F 之前更早红：发布路径根槽 FUA 之后轮换系统配置就返回，没有屏障）。
- **第二道屏障买的是记录流完整性**，不是数据完整性：数据完整性的必要屏障恰一道（根槽之前）；不上第二道有 7 类「根在案而记录缺席」的状态，记录核对器与反向链的输入有洞。用户定案「需要」，两道都上。

**依据**：

- E77（发布的持久顺序）：崩溃子集穷举——不上根槽之前那道屏障，1024 个崩溃状态里 504 个违例；拿掉「重放施加前验证点名单元」这一步，b_rs 臂 63 处静默嫁接，所以它是承重步骤。
- 用户定案，原话在变更史；「发布返回前加屏障」（系统配置轮换之后那一道，序点从三个变四个）是用户 2026-09-27 定的，起因、弹窗事实与岔路在 `research/prompts/c554-fix-forks.md` 第 4 行，实现已合入（`crates/singlefs-core/src/transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration`，实 C577（系统配置没见证到的最新根，乙罩不到） 报告 `research/prompts/m2-impl-c577-barrier-implementer-report.md`），代价仍另登记小实验量。

**欠**：C26（屏障数从没量过）（真机每次 fsync 的屏障数仍没量）。崩溃点重放 harness 的两条自证用例（摘掉根槽前的屏障必须红、关掉重放验证必须红）随 C76（发布时序没有权威落点） 已还清。

```

**出处 `.claude/kb/decisions/16-发布语义.md:194-215`（整段抄，未转述）**

```markdown
#### 已定项 8：暖机取戊，新实例的根先覆盖两块盘

**定案**：新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回；做法是连推空发布，第一版几何至多 R = 3 次。**第一版实付 2 次，而且它是格式常量不是运行时谓词**：根环三个区域的设备归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7），于是 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定。登记成 `format-const` WARM_UP_EMPTY_PUBLISHES = 2 与 NEW_POOL_FILE_CREATION_TXG = 3（[layout/01-first-txn.md](../layout/01-first-txn.md) 八），条款在 D22（单元原子性怎么合成） 已定项 16 第 5 句。**后续挂载与挂载中途崩了再挂**：次数现算——写行那次发布的 txg 取 D23（journal 的角色与格式） 已定项 14 注 3 那个下界本身（max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1），之后每次空发布加一，按根环落点看落哪块盘，推到本实例的根覆盖每块盘为止，至多 R 次；取号之前的准入与取号之后推发布用同一份计划（`crates/singlefs-core/src/mount.rs` 的 `warm_up_publish_txgs`）。

**射程**：定的是新实例确认之前要做什么，不定实例切换本身的判据（D23（journal 的角色与格式） 已定项 14）、不定空发布写出多少单元（已定项 9）。四样已知边角：

- **前置 C314（回退可以复用被抛弃的根引用的单元） 的条款那一半已满足**（用户取影子账，条款写进 D23（journal 的角色与格式） 已定项 14 与 I-7.4（近 K 代块未被复用）），暖机已生效；**检查那一半仍欠**：多次挂载的录制流已有（层 0 第二条流），逐句核仍差格 1 的专门断言，C314（回退可以复用被抛弃的根引用的单元） 因此仍在未还表。
- **归属改了那两个格式常量要重算**（加盘或开条带表时）；「至多 R = 3 次」那句仍是几何上界，不是第一版走得到的次数。第一版实付 2 次这一档没有走三方论证，可推翻。
- **代价**：改已定项 7 的 fsync 返回条件（新实例确认前多一个条件）；新池新建文件的 checkpoint_txg 从 1 变 3，多两条空记录与两次根写；D28（挂载期承诺量） 已定项 3 的切换预留加「每次切换至多 R = 3 次空发布」，按 D28（挂载期承诺量） 已定项 4 现算的 c_max 取（池规模 9 块 ⇒ 27 块）；空发布占根环槽位，把旧的非空根更早挤出环。
- **另两臂**：甲（不补）判据 2 一个故障就丢；丁（重放跨实例边界）出局。
- **白推**：写行 txg ≡ 2 (mod 3) 时第一次空发布落在已经覆盖的盘 0 上，白推一次——副本装置上量到每次多写 147968 字节、多 2 道屏障、多 1 次 FUA、多占根环一槽，只重开不写时每次挂载都落在这一档，24 槽的根环 8 次挂载就整圈转完（入库装置上没重做）。这是写行 txg 取下界本身的直接后果，认下；「普通挂载的写行 txg 往后挪到只推 1 次的值」那条改法被攻过零轮，没采纳。

**依据**：

- E150（回退复用被抛弃的根引用的单元）：无暖机时 1 个故障就落到被抛弃的根上，暖机之后要 4 个；影子账与墓碑两种修法都零违例，用户取影子账。
- E148（提交固定点按两棵记录树重算）：空发布在池规模、提交块聚簇时 9 块，切换预留那一半按它算。
- E142（新池新建文件的干跑）：checkpoint_txg 1 → 3 的改动落到新池新建文件的字节上之后，262162 个状态零违例。
- 两轮三方论证 `research/prompts/c199-r2-main-verification.md`；用户定案，原话在变更史。
- 后续挂载那一句与白推：三方第一轮 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K2（推到覆盖为止在后续挂载、回退、挂载中途崩了再挂三种起点上都构造不出不够），用户 2026-09-23 定照今天写、认白推，原话在变更史。

**欠**：C314（回退可以复用被抛弃的根引用的单元）（检查那一半）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:216-230`（整段抄，未转述）**

```markdown
#### 已定项 9：一次空发布也写单元

**定案**：记账树存在时就写——空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2「每行每发布重写」重写记账行，连带记账树节点、分配记录、映射条目与树表单元。第一次可写挂载的暖机时树表 0 条 ⇒ 零单元；以后的空发布按 D28（挂载期承诺量） 已定项 4 现算的 c_max 块。**树表 0 条的一版上写行的那次发布写两个单元**：实例表与一片分配记录节点，分配记录树根指针住根记录（D22（单元原子性怎么合成） 已定项 7）；被换下的上一版实例表与上一片分配记录节点记成已释放，重开时从这片节点重建这一版的账。根记录那一项只有树表 0 条、写过行的那一版写非零，mkfs 的第 0 代与带文件的一版写全 0，零单元发布照抄上一版。

**射程**：定的是空发布写不写单元、写哪几样，不定 c_max 怎么算（D28（挂载期承诺量） 已定项 4 按当时结构现算）、也不定空发布什么时候推（推抬 F 在已定项 1，暖机在已定项 8）。写行那次发布写的那片分配记录节点，单元头树 ID 写 0（实现今天的取法：那一版的树 ID 水位是 mkfs 的 11，写分配记录树的 13 会让 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 当场红），没有条款；它进不进中央映射也没有条款——那一版没有映射树，D19（块指针的结构与宽度预算） 已定项 8 的自举豁免三类里没有它。

**依据**：

- E148（提交固定点按两棵记录树重算）：空发布的固定点按两棵记录树重算是池规模 9 块、新池新建文件规模 4 块，「以后的空发布按现算的 c_max 块」那句的数出自这里。
- E142（新池新建文件的干跑）：第一次可写挂载的暖机那两次空发布逐字节写出来，树表 0 条时零单元。
- 用户定案 2026-09-23（写行那次发布在树表 0 条的一版上写实例表与分配记录节点两个单元、分配记录树根指针住根记录），原话在变更史；实现与用例点名在 [checks-owed.md](../checks-owed.md) 已还清表 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 那一行。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:231-246`（整段抄，未转述）**

```markdown
#### 已定项 10：发布粒度取攒批，不是每个事务发一次

**定案**：攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。

**射程**：定的是频率，不定一次发布事件内部的顺序——那在 `litmus/commit-publish.litmus`，它钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。两样已知边角：

- **净代价分布**：格式侧几乎为零（journal 本来就要有），验证侧一大笔——中间态语义改写（已定项 12）、第二份 replay 实现（已定项 4 与 13），再加 D13（验证路线） 的「Merkle 根链差分」粒度变粗：一代根里塞了 K 个事务的改动，定位精度从「哪个事务」降到「哪个 checkpoint」，而 D13（验证路线） 已写明夜间档的真代价是定位成本。
- **发布语义决定「走几次」**：每事务发根 → K 批走 K 次；checkpoint → 同一窗口内的 K 批只走约一次，各批只留一条 journal 记录（小、顺序写）⇒ 分批大删除那一格的元数据写放大从 `K × M × H` 降为 `checkpoint 数 × M × H`。

**依据**：

- 无实验：定案靠两条互相独立的推理，当时没有可量的量。① 它与 D5（快照 / 空间记账机制） 已定的 ZFS birth txg 模型天然相容——ZFS 的 txg 本身就是 checkpoint（`zfs_txg_timeout` 默认 5 秒，或攒够一定脏数据量），一个 txg 批量大量操作、txg 结束才更新一次 uberblock；采纳 checkpoint 之后 birth txg 的号粒度问题自动消失，一个 txg 一个号，deadlist 的比较粒度就是 checkpoint 粒度、不必重算（未在本项目验证）。② **根是丢失写的最终见证者**：检测「盘上这块是不是它应该是的那个版本」需要一个外部见证者，块自身办不到（合法的旧版本与合法的新版本在块内层面同构，D18（块里携带什么信息） 有完整论证），而见证者必须在不同的失败域上——父节点若在同一次事件里也没写成就要祖父，递归到根 ⇒ checkpoint 发布的那个根是这条递归的终点，它的持久化必须比任何单个块更可靠（ZFS 的形态是 4 个 label × 每个 128 KiB 的 uberblock 环、且偶数与奇数 label 分两阶段写；本工程根自身的冗余形态由 D22（单元原子性怎么合成） 已定项 2 定）。频率的可量代价（两个阈值的可行域）在已定项 5。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:247-259`（整段抄，未转述）**

```markdown
#### 已定项 11：没有任何操作可以绕过 journal 直接发根

**定案**：所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏。fsync 的效果是「提前触发一次发布」，**不是另开一条日志格式、另一套恢复逻辑**。判据是：journal 是所有提交的必经之路，还是某些路径的旁路——前者是一套机制、持久化分两级，合规；只有 fsync（或只有元数据、只有小写）走 journal、别的直接发根，就是 `.claude/rules/fs-design.md` 点名的「为 fsync 单开一套日志树」。

**射程**：说的是挂载之后的操作；mkfs 种子写第 0 代根不经 journal（D13（验证路线） 已定项 1 自己列的根槽三种写者之一，mkfs 一条记录都不写），不在这句的射程里。已知边角一样：D23（journal 的角色与格式） 已定项 1 的轴一取甲压在这一条上，重开它要回答的依据里有这一条。

**依据**：

- 无实验：它是 `.claude/rules/fs-design.md`「一个事务层，所有结构共用」在发布这一格的形态，判的是机制的形状（必经之路还是旁路），不是一个能量出来的数；不写死就会退化成「为某个路径开第二套事务机制」。每次持久化的写量是另一个问题，归 D23（journal 的角色与格式） 已定项 1。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:260-272`（整段抄，未转述）**

```markdown
#### 已定项 12：中间状态的合法性语义改写成「根 + journal 任意前缀」

**定案**：D8（核心索引结构）「中间状态必须每一个都是合法的文件系统状态」改写为「**根 + journal 任意前缀，replay 之后合法**」。

**射程**：定的是合法性怎么判，不定前缀本身怎么截——前缀判定的完整口径是 D23（journal 的角色与格式） 已定项 14 的五条，缺一不可。已知边角一样：块复用那一侧的真正约束比 checkpoint 边界更严，按 journal 位置说（已定项 15 与 D22（单元原子性怎么合成） 已定项 4）。

**依据**：

- E42（一事务几条记录）：1601 个前缀里 1400 个被前缀切开（87.4%），一事务一条那一臂非法前缀恒 0——那是「任意前缀 replay 之后合法」的直接形式；跨多条而无边界字段时非法前缀恰等于被切开的前缀数，加边界字段之后 0。这条语义在模型上是可数的，而且数得出它什么时候破。
- 用户定案，原话在变更史。

**欠**：C42（残留记录冒充合法前缀）（上一条时间线留在环里的记录会被下一次恢复当作合法前缀重放）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:273-285`（整段抄，未转述）**

```markdown
#### 已定项 13：这份 journal replay 归记录核对器，不归 O2（独立解析器 + checker）

**定案**：D13（验证路线） 已定 O2（独立解析器 + checker） 是一元谓词、只判单个镜像，所以这份 journal replay 归「记录核对器」。那是第二次实现 replay，而 replay 正是崩溃语义最容易出**共同误解**的地方——D13（验证路线） 的「三 oracle 合谋测试」要防的就是这个。**这是 D16（发布语义） 最贵的一笔。**

**射程**：定的是归谁，做什么与允许共享什么在已定项 4。已知边角一样：D23（journal 的角色与格式） 轴一定甲之后「redo 为空」，最贵那一笔的大头随之消失，只剩前缀判定 + 择根 + 续做。⚠️ **「redo 为空」只对「根已持久」的发布成立**：E42（一事务几条记录） 实测 1601 个前缀里 1400 个被前缀切开（87.4%），被切开的比例与边界字段在不在无关——变的是这 1400 个非法还是合法（无边界字段臂 1400 非法，有边界字段臂 0）⇒ **恢复期重放非空是常态，不是边角。**

**依据**：

- 无实验：哪个判决器实现 replay 是验证路线的分工，没有可量的量；第二份实现的成本量级与它必须独立实现什么在已定项 4。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:286-298`（整段抄，未转述）**

```markdown
#### 已定项 14：快照发布点与 checkpoint 边界对齐，钉在全部数据变更之后

**定案**：快照发布点必须与 checkpoint 边界对齐，且位置钉死在「本 checkpoint 全部数据变更之后」，快照不许出现在 checkpoint 中间。同一 checkpoint 内建两个快照是允许的：它们捕获同一状态、其间 deadlist 为空；但 checker 里的「上一个快照」要用 `(txg, 快照 id)` 打破并列。

**射程**：定的是快照点落在哪，不定快照本身的记账模型（D5（快照 / 空间记账机制） 与 D6（快照实现模型））。已知边角一样：这条规则的可执行形态是已定项 5 那段丢失窗口口径——发布阶段等全部单元落盘，快照点就落在那之后。

**依据**：

- 无实验：理由是机制性的，没有可量的量——`birth` 的分辨率只到 checkpoint，**块无法表达「我生在本 checkpoint 里快照点之前还是之后」**；不钉死它，D5（快照 / 空间记账机制） 那张边界表里 `birth == S.txg` 与 `death == S.txg` 两行会同时失去确定答案，而它们一个是**数据丢失**、一个是**空间泄漏**。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:299-318`（整段抄，未转述）**

```markdown
#### 已定项 15：checkpoint C 中产生的释放，C 发布之前不进可分配集合

**定案**：checkpoint C 中产生的一切释放，在 C 被发布之前不得进入可分配集合。两个理由，第二个是 D16（发布语义） 独有且更严的：

1. **回滚**：崩溃后可能回到 C−1 的根，而生于 C 之前、死于 C 的块在那时仍然活着。
2. **已定项 12 把 D8（核心索引结构） 的中间态语义改写成「根 + journal 任意前缀，replay 后合法」**：若一个块在 journal 位置 p 被释放、位置 q > p 被重新分配并写入新内容，那么重放到 `p < r < q` 的前缀会得到一棵指向该地址的树，**而盘上的字节已经是 q 的内容了**（物理写不受 journal 位置约束）。**这不是理论风险，是必然的错。**

**射程**：定的是「什么时候可以回可分配集合」这条下界，不定窗口取多大（已定项 1）。三样已知边角：

- **第二条理由是按 journal 位置说的** ⇒ 真正的约束是整个 journal 重放窗口，不是 checkpoint 边界；D22（单元原子性怎么合成） 已定项 4 收在那一形，缺了它的失败模式是「可检测（父指针校验和失配），不可恢复」。
- **窗口容量与 ENOSPC 直接相关**（D3（空间分配）：释放空间这个操作本身不许申请空间），而攒批让「建了又删」在同一窗口内闭合的概率大幅上升 ⇒ **defer 窗口落在热路径上**。
- **外部线索**：ZFS 有同型机制且更保守，`include/sys/txg.h` 的 `#define TXG_DEFER_SIZE 2 /* Number of txgs worth of frees we defer adding to in-core spacemaps */`，**未在本项目验证**；⚠️ 那个 2 为什么是 2，没查到一手解释，本工程的 defer 窗口要按自己的重放语义推，**不许照抄**。

**依据**：

- E82（准入的在飞合成）：把窗口内产生的释放记成可用的那条臂，20 格吃进 defer 该扣住的块——这条规则被静默绕过时的现象是可数的，准入侧的形态因此写进了 D28（挂载期承诺量） 已定项 2。
- 用户定案，原话在变更史。

**欠**：无（块复用那一侧的会红检查由 I-7.4（近 K 代块未被复用） 承担，欠账在已定项 1 列的那几笔）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:319-331`（整段抄，未转述）**

```markdown
#### 已定项 16：checkpoint 与消息缓冲买同一样东西，E7（离线索引 harness） 的对照组钉死

**定案**：checkpoint 与 D11（索引节点要不要留消息缓冲区） 的消息缓冲区买的是同一样东西，差别只在**摊销窗口住在哪**：checkpoint 的窗口住**内存**、不进格式；消息缓冲的窗口住**盘**、必须进格式。⇒ **D11（索引节点要不要留消息缓冲区） 的边际收益 = 「跨 checkpoint 的摊销」减去「checkpoint 内摊销」**。**E7（离线索引 harness） 的对照组必须是「有 checkpoint、无消息缓冲」。**

**射程**：定的是那个对照组怎么设，不定 D11（索引节点要不要留消息缓冲区） 取哪一边——那由 D11（索引节点要不要留消息缓冲区） 自己定。已知边角一样：BetrFS 那组数字里这两块从没被拆开过（它同时有 4 MiB 节点的消息缓冲**和** 5 秒 checkpoint），**未在本项目验证**。

**依据**：

- 无实验：这是「对照组怎么设」的规定，不是实验结论；文献里没有一个对照组带 write buffer 前端，消息缓冲相对本工程现方向的边际收益，现有文献一个数字都没给，本项目也还没有可引的量。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:332-354`（整段抄，未转述）**

```markdown
#### 已定项 17：fsync 报错之后，调用方能假设什么

**定案**：

- fsync 报错之后，这次发布的数据**可以出现也可以不出现**——调用方不许把「报错」读成「这次没成立」。
- **报错之后不许假设可以照旧复用同一份内存态重试。**（`close(2)` 的 NOTES 段明写「Retrying the close() after a failure return is the wrong thing to do」；本工程取同一条语义。）
- 要知道到底成没成立，调用方自己去读现行根：`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`、`choose_root` 对着仍打开的设备句柄就判得出，判据是「这次的 checkpoint_txg 是不是现行根」，不用卸载重挂。今天 `transaction.rs` 没有调它们的路径，缺的是一段胶水。
- 这一条不定 fsync 什么时候返回，那是已定项 7。

**射程**：

- **只在「转只读」与「重做又失败」那几格上生效。** `D23`（journal 的角色与格式） 已定项 14 失败表判「切换」的那几格，调用方根本收不到错误（实例切换会把在飞 checkpoint 重发掉），这一条对那几格无话可说。
- 管的是「报错之后调用方能假设什么」，不管失败本身怎么分流（那是 `D23`（journal 的角色与格式） 已定项 14），也不管持久顺序（已定项 7）。

**依据**：

- 无实验：这一条是对外语义的承诺，没有可量的量。撑它的是 `research/prompts/c381-r3-main-verification.md` 第一节 K1 那一格——H1（第六步系统配置槽轮换失败、前五步全成）上，「报错即没成立」这条语义**本身就是 `C381`（根已落盘之后发布失败，分配器仍退回） 的成因之一**：它让调用方以为没成立、拿旧状态接着发。
- `research/prompts/c381-r3-main-verification.md` 第三节（候选甲站住，两样收严的来处：K2 的 POSIX 线索、K4-e 的射程限定）。
- 用户定案 2026-09-21，原话在变更史。
- ⚠️ **依据里要写明**：定案的后两句（禁止性条款、射程限定）**被攻过零轮**（`.claude/rules/three-way-inference.md`：攻方腿自己提的收严算没被攻过），用户 2026-09-21 知情接受。

**欠**：`C381`（根已落盘之后发布失败，分配器仍退回）（改法落地之前这一格还开着）。

```

**出处 `.claude/kb/experiments/01-节点大小vs可恢复性.md:1-52`（整段抄，未转述）**

```markdown
## E1 节点大小 vs 可恢复性 —— 部分已跑（爆炸半径那一半由 E29（坏一个节点的爆炸半径）还清，且判据被它纠正；恢复时间那一半仍欠 checker）

**产物**：爆炸半径那一半的原始输出在 `e29-blast-radius-2026-08-29.out`，
复跑 `cargo run --release --bin e29_blast_radius`（正文在 E29（坏一个节点的爆炸半径））。
⚠️ **恢复时间那一半没有产物，因为它一次都没跑过**——前置是 mkfs、checker、定点破坏工具。


**背景**：大节点省 seek、树更浅（bcachefs 用 128K–256K），
但**一个坏节点的重扫代价随节点大小线性放大**（E29（坏一个节点的爆炸半径） 实测：第 1 层节点的重扫量恰等于扇出，
2 KiB 的 49 到 64 KiB 的 1636），与本工程的可重建性目标直接冲突。
⚠️ **放大的是恢复时间，不是永久损失**——索引是派生态（D21（权威态与派生态的分界）），坏了只是慢。
两边都要量，不能只挑一边说话。

**自变量**：节点大小 16K / 32K / 64K / 128K / 256K。

**测什么**

| 指标 | 口径 |
|---|---|
| 单节点损坏的爆炸半径 | 随机破坏 1 个元数据块后，**变得不可达的 key 数**（不是字节数——字节数会被节点大小本身带偏）。算术那一版已由 E29（坏一个节点的爆炸半径） 给出，这里要的是真镜像上的复现 |
| **全盘扫描重建的重扫代价** | 破坏 K 个节点后，把它们重建回来要扫多少个单元。**不测「成功率」**——E29（坏一个节点的爆炸半径） 实测索引侧的永久损失恒 0，成功率那一维没有判别力 |
| **可达但值陈旧或错误的 key 数** | 把重建出的 keyspace 与模型（那个理想文件系统 HashMap）逐 key 比对，**与「不可达」分开报两个数** |
| 查找代价 | 一次点查平均读多少个块；树高 |
| 元数据总量 | 同一份数据集下的元数据字节数 |

**判据**：在「**重扫代价 ≤ 目标线**」的前提下，取查找代价最低的那个节点大小
（2026-09-06 用户定案，与 D8（核心索引结构） 已定项 4 的口径改写同一次）。
目标线是一个恢复时间预算，**要先定**——这一步不能等实验结果出来再定。
⚠️ **判据不是「全盘扫描重建成功率 ≥ 目标线」**：E29（坏一个节点的爆炸半径） 实测
索引坏的永久损失恒 0 ⇒ 成功率与节点大小无关，拿它当判据夹不出上限。

**怎样才算失败**：如果各档之间爆炸半径没有可分辨的差异，
说明破坏注入的方式不对（比如只破坏了叶子、或者破坏点没落在被引用的节点上），
**整轮作废重做**，不许拿「看起来差不多」当结论。

⚠️ **「可达但值陈旧」这一列不是锦上添花，是E1（节点大小 vs 可恢复性）的判别力缺口。**
纯路由的内部节点丢了，损失的只是路由，叶子自带块头，扫描重建是**幂等**的；
而一旦内部节点里存了别处没有第二份的数据（例如消息缓冲区里还没下刷的更新），
丢了它损失的就是**数据**，重建出来的树可以块头对、校验和对、Merkle 链对——
**I-1（块自描述）.x 与 I-2（校验和）.x 全绿，内容却是旧的**。
[invariants.md](../invariants.md) 里现有的不变量没有一条能检出这一类。
不加这一列，E1（节点大小 vs 可恢复性） 会对「内部节点存数据」这类设计报出一个漂亮的重建成功率，
而那个数字什么也没保证。

**前置**：D8（核心索引结构） 定案、mkfs、checker、以及一个能定点破坏镜像的工具。

⚠️ **不产出「按设备特性算节点大小」的选取规则**：D8（核心索引结构） 已定项 2 已定
16 KiB **钉成常量**——「格式里是一个常量，不是一条公式」，依据是 E20（指针变宽的 CPU 缓存代价）
那条曲线上 8 KiB 的反常六轮稳定且无人解释。
⇒ E1（节点大小 vs 可恢复性） 要产出的是**那个常量的上限校验**：恢复时间目标线之下 16 KiB 站不站得住，
而不是一条按设备算出节点大小的公式。**测试一律显式钉死取值，不许用自动值。**

```

**出处 `.claude/kb/experiments/01-节点大小vs可恢复性.md:53-59`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 4 | 备料 | 2026-09-20 不受影响：D8（核心索引结构） 已定项 4 逐字写「取值靠 E1（节点大小 vs 可恢复性） 定：在「重扫代价 ≤ 目标线」的前提下取查找代价最低的那档」，而恢复时间那一半一次没跑、目标线本身也还没定，今天没有哪条分项压在 E1（节点大小 vs 可恢复性） 的数上 |
| D21（权威态与派生态的分界） 已定项 9 | 不影响 | 2026-09-20 不受影响：E1（节点大小 vs 可恢复性） 只把「索引是派生态、丢了只是慢」当输入，用来说明随节点大小放大的是恢复时间不是永久损失；D21（权威态与派生态的分界） 已定项 9 的定案不压在 E1（节点大小 vs 可恢复性） 上 |

```

**出处 `.claude/kb/invariants.md:1-8`（整段抄，未转述）**

```markdown
# 不变量清单

**checker 是 invariants.md 的可执行形式。** invariants.md 每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../rules/format-evolution.md)「硬约束」）。

每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。
写不成可判定形式的，说明还没想清楚，不许进 invariants.md。

```

**出处 `.claude/kb/invariants.md:9-18`（整段抄，未转述）**

```markdown
## 状态

**新池新建文件阶段（2026-09-14 起）。** 磁盘格式已落成字节：mkfs、新池新建文件的写路径与冷启动恢复在 `crates/singlefs-core`；checker 在 `crates/singlefs-checker`，与实现只共享常量模块（D13（验证路线） 已定项 5）。
池级 checker（`walk::check_pool_image`）判 49 条（数它的命令：`grep -c '^| I-.*已实现' .claude/kb/invariants.md`，与 `crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 长度相等；第一版 23 条，2026-09-16 里程碑「覆盖写、释放、回退与复用」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），2026-09-18 加 I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 与 I-5.4（分配记录罩住的槽互不相交），2026-09-21 加 I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号），2026-09-22 加 I-1.10（码 2 条目宽等于字段表宽）、I-9.6（水位大于两处最大号）、I-9.12（分隔 key 落在孩子区间之外）、I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块），2026-09-23 加 I-8.8（前缀里的事务不被切开） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号），2026-09-24 加 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读） 与 I-8.9（一次发布的记录序号连续且只有末条带标志），2026-09-25 加过 I-7.10（已停用·回退见证表删了） 与 I-7.11（已停用·见证择根删了）、2026-09-26 随回退见证删掉退役，同日加 I-7.12（系统配置 F 不低于同盘根上的 F），2026-09-27 加 I-1.11（映射 key 与单元头相符），同日加 I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序），2026-09-28 加 I-7.14（系统配置本盘设备号等于盘在池里的身份）），状态列写「已实现」的就是这 49 条：每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`），层 0 的每个崩溃状态都跑它。
I-7.7（系统配置实例代号不低于根环） 2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 定案改成 ① ②，层 0 里 0 个状态判红；I-3.1（已分配统计对得上） 与 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 的 checker 读法 2026-09-14 由用户收尾弹窗定甲，写在各自那一行。
其余条目仍写「未实现」：它们要覆盖写、释放、回退与复用（释放、重用、残留）、journal 一侧的那一类检查、快照或加密，新池新建文件的镜像上判不出东西。

<!-- invariant-count -->
现共 82 条在用（编号至 I-9.16（树表条目按树 ID 严格升序且合发号次序），另有六条退役，编号不回收）。这一句是当前条数的权威登记位；「## 历史版本」里那几句「现共 N 条在用」记的是当时的数，不跟着改。

```

**出处 `.claude/kb/invariants.md:19-44`（整段抄，未转述）**

```markdown
## I-1 块自描述 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-1.1 | 块头自述逻辑地址 | 任一单元，其头记录的**类内身份**与实际引用一致——数据单元的身份是五元组（锚点偏移即它的逻辑地址）；索引节点是树 ID + 层级（+ 该树已定携带的 key 区间，D18（块里携带什么信息）已定项 2 口径）；打包记录单元（码 3）是（出生树 ID, 打包记录类型, 容器号, 容器出生代）四元组，「父级」按打包记录类型分——类型 1 走容器索引的身份引用（C103（容器索引无落点）），类型 2 走 inode 树内部节点条目的身份引用（I-9.2（条目身份与子头相符）），**类型 4（实例表）走根记录的物理指针，不属于任何树、不进容器索引**。**自证单元（journal 记录 / 根槽 / 系统配置槽 / 类型 4 实例表单元）显式豁免**：无父引用无树身份，自证纪律在 I-7（根环）/ I-8（journal）两类与 D22（单元原子性怎么合成）已定项 21；多层码 2 树（记账树、中央映射树）的索引节点，身份里的层级与 key 区间按引用它的父条目核：层级 = 父 − 1、区间落在父条目给的那一段、内部节点区间是子树覆盖区间；按位置寻址的两棵树（分配记录树、extent 树），节点头里的层级与 key 区间等于它的位置规定的那一层、那一段；内部条目的 key 是某个孩子那一段的起点，孩子按位置严格递增；分配记录叶里每条记录的起点与末槽都落在叶罩的那一段里；extent 上段叶条目的 inode 号、下段叶记录的单元号落在叶罩的那一段里，上段叶条目的标签只能是 0、1、2（标签 0 载荷全零、标签 1 载荷末 2 字节为零）；根之下没有空节点（这一半 checker 由实二五补）。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；`walk::Walk::walk_code_two_subtree`；坏镜像用例 `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`；`crates/singlefs-checker/src/position_addressed.rs`） |
| I-1.2 | 块头写序已发布 | 任一单元，其头记录的写序 (i, n)（D18（块里携带什么信息）已定项 7 / 11 的字段）与诞生代号 b 按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的全局已发布谓词判成已发布：i < 挂载根实例且无该实例的行 ⇒ 已发布，有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub（所选根那个实例的 T_pub = 重放之后那个根的 txg，严格介于所选根的实例与新实例之间、又不是切换照旧事务写序主人的实例写 (i, 0, 0)，切换的所选根按 D23（journal 的角色与格式） 已定项 14 取被重发的在飞 checkpoint 所基于的根；W = **属于实例 i 的**、被这次重放施加的最大事务号，切换时是被照旧的、写序属于 i 的最大事务号；同一次恢复写出的那批行里只有所选根那个实例的 W 可能非 0，同一次切换写出的那批里只有被照旧事务的写序主人的 W 可能非 0，表上允许有多行非 0，它们来自不同次的恢复或切换；2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；i == 挂载根实例 ⇒ b ≤ 挂载根的 checkpoint_txg；i > 挂载根实例 ⇒ 判损坏。**例外**：打包记录类型 4 的实例表单元不走这条谓词，它的现行版本由根记录持有的物理指针唯一确定。**判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留**——这是扫描方向独立于遍历方向判它的依据（D13（验证路线） 已定项 12 双方向要求）；2026-09-05 前它只比诞生代号与当前 txg，崩溃后 txg 重发让那条判据对同 txg 两条时间线失效（E104（扫描重建的现行版本判定）：无实例表时崩溃世界复活 6） | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`，**只判遍历方向那一半**：走读到的每个被引用单元，头里的诞生代号 b、写序的实例代号 i 与尾段（码 1 是写序的事务号 n、码 2 / 码 3 是出生序号）要与**引用它的那条指针**记的相同——不同则「头记录的写序」不是这个单元的真实身份，谓词判的就是假输入；口径同恢复路径今天的前置检查（`crates/singlefs-core/src/recovery.rs` 五处报 I-1.2（块头写序已发布） 的 `InvariantViolated` 里「出生身份与指针不符」那三处；「诞生于根之后」那两处是谓词那一半，归 I-4.2（无被引用未提交块）），checker 按字段表自己解一份。⚠️ **扫描方向那一半（「任一单元」都按谓词判、i > 挂载根实例 ⇒ 判损坏）对着一份崩溃镜像判不了，没写**：① 判不成已发布本身不是违例——本行判据自己写着那是未发布的撕裂事务垃圾或被抛弃时间线的残留；② 「i > 挂载根实例 ⇒ 判损坏」只在恢复刚发布完新根的那一刻成立，而上一次挂载取了号、写出单元、一个根都没发布就崩时，盘上合法地留着 i > 最新根实例的在飞单元，那一支今天由 I-7.7（系统配置实例代号不低于根环） ① 按各盘系统配置的实例代号兜着。谓词本身照旧在用：`PublishedPredicate` 过滤 I-1.8（归并后版本全序） 的射程、判 I-4.2（无被引用未提交块）。打包记录类型 4 的实例表单元按本行的**例外**不进这一条。坏镜像是 extent 树根头里的出生序号 +1、引用它的指针一个字节不动（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它，走读一个被引用单元都读不到的状态报「不适用」） |
| I-1.3 | 块头树 ID 一致 | 任一单元，其头记录的所属树 ID 与实际引用它的树一致。中央映射树不进树表，「实际引用它的树」取根记录（或记录新根段）里中央映射树根指针带的出生树，不写死 15：回退到无文件那一版之后再发第一个文件版本，八棵树从水位起重新发号（D8（核心索引结构） 已定项 8 ②），映射树的号随之变（主 agent 2026-09-23 定，C511（回退到无文件那一版之后诞生代怎么接） 第 3 步实现员查出写死 15 的读法在合法的回退历史上误红）。自证单元（含类型 4 实例表单元）与 I-1.1（块头自述逻辑地址）同口径显式豁免。⚠️ 打包记录单元（码 3）与被快照共享的数据单元头里的树 ID 是出生树，跨头共享时与引用树不同，那一情形的判法随 D9（加密） 那次「绑出生快照」的重开（C110（跨头共享与加密后元数据类的块头一致性读法））。这一射程覆盖码 3 全部打包记录类型，含类型 2（inode 树的叶，D8（核心索引结构） 已定项 6）：克隆头共享 origin 的叶时，叶头的出生树是 origin | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.4 | 块头 fsid 一致 | 任一单元，其头记录的 fsid == 系统配置 fsid（D18（块里携带什么信息）已定项 7 的字段；加密开启后与 KDF 绑定冗余，接受）。scrub 半边「本卷全部单元归属指向本卷」靠它才写得出（C74（无加密期的跨卷错向写没防线）补救的另一半） journal 记录头 2026-09-14 起也带 fsid 8（系统配置 fsid 低 8 字节，D23（journal 的角色与格式） 已定项 4）：fsid 不符的记录整条不进重放前缀、按断链处理（E142（新池新建文件的干跑） 第七次跑记的 G25） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.5 | 系统配置记管道身份 | 系统配置记录写出该镜像的管道身份（实现标识 + 版本），且非空。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：自举头段「写入者身份 20」= 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32，第一版写 `singlefs-rs` / 1；实现标识全零判红 | 未实现（条款已写、检查仍欠） |
| I-1.6 | 类标签登记且三处相等 | 任一单元，共同前缀偏移 6 的单元类型标签 ∈ D18（块里携带什么信息） 已定项 11 的登记表（码 0 与未登记码即判损坏）；偏移 7 的 8 个 flag 位第一版恒 0，非 0 一律拒收（前向兼容政策，D15（格式冻结政策） 判定表）；数据单元（码 1）五元组首字节 == 偏移 6；打包记录单元（码 3）类身份段首字节 == 偏移 6 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.7 | 打包容器合法与判定顺序 | 任一打包记录单元（码 3）：记录数 × 记录宽 ≤ 声明长度且声明长度 ≤ 32768 − 136（含 29 字节预留位的头，D18（块里携带什么信息） 已定项 16；2026-09-14 预留位从 28 加到 29，`cap(136, 56) = ⌊32632 / 56⌋ = 582` 与按 135 算同值），**先于**载荷校验和判（D23（journal 的角色与格式） 已定项 1 的 B 条同一纪律）；判定顺序恒为头校验和 → 类身份段 → 记录宽 == 登记表为该打包记录类型登记的宽（类型 2 是 140，不等判损坏，只需头字段，D8（核心索引结构） 已定项 6）→ I-1.2（块头写序已发布） / I-1.3（块头树 ID 一致） / I-1.4（块头 fsid 一致） → 边界 → 载荷校验和 → 逐条记录，任一步失败即中止；打包记录类型 4 的两种 kind 记录另按 D18（块里携带什么信息） 已定项 11 的实例表字段表判，不套用「记录宽 == 登记表的宽」那一步（一片记录宽恒 88，条数上限由片容量给——D18（块里携带什么信息） 已定项 11 2026-09-13 从 64 改宽）（对象级的判定——I-9.7（记录三段恒零）、I-9.8（记录 inode 等于 key）——在这条中止链之外，只让那条记录 EIO）；容器头声明一种打包记录类型、一棵出生树；打包记录类型 ∈ 第二级登记表，未登记类型的容器其物理指针不得出现在任何回收 / 复用操作的输入集合 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.8 | 归并后版本全序 | 码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组（字段表在 D18（块里携带什么信息） 已定项 7（码 1）与已定项 11（码 3）；归并键取这些字段但去掉载荷校验和那一段——它正是 ①「同一组的成员载荷相同」要比的那个量，进了键 ① 就恒真）之后，同 key 的各组两两全序键不等——码 1 的键是写序，码 3 的键是 (诞生代号, 实例代号, 事务号) 三元——诞生代号打头（同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同），事务号留在末位做同一 checkpoint 内的平局破除（固定点会迭代，同一容器在一个 checkpoint 里不保证只写一次）；同一组的成员载荷相同（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 副本归并，射程 w = 2；码 2 不进这条） | 已实现（2026-09-21，池级 checker `walk::check_pool_image` 判 ① 同一组的成员载荷相同；② 同 key 的各组两两全序键不等**只判码 3**——码 1 的键停着等 C464（I-1.8 对码 1 的全序键写窄了） 定案，checker 里显式写成 `TotalOrderKey::UndecidedForDataUnit`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.9 | 清扫水位内无可读垃圾 | 清扫水位（D5（快照 / 空间记账机制） 已定项 4 第 13 项）以内不存在可读垃圾；其外的可读垃圾每个 key 都被更大写序的可见版本或可见墓碑压住（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4；扫描重建的产物未晋升期间清扫禁止，这条在重建之后失效） | 未实现 |
| I-1.10 | 码 2 条目宽等于字段表宽 | 任一码 2 索引节点，头里自述的**条目宽**（D18（块里携带什么信息） 已定项 7 的自描述三段之一：key 宽 1 + 条目数 2 + 条目宽 2，偏移表在 D18（块里携带什么信息） 已定项 18、载荷布局在 D8（核心索引结构） 已定项 11）**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）。条目宽是**盘上读来的值**，`index_node_view` 只判了它 ≥ key 宽（extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」——不判就按字段表的固定偏移切，切片当场越界（增补 3 第 6 件 panic 面普查第二族，C476（盘上内容可控时走得到的 panic 有 21 处） R1 / R3 / R4 / R13）。条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它），否则判该节点损坏（2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做）。判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目。⚠️ **中央映射树不判**：它的条目宽由 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树） 定，那一条还开着；**树表条目（200 字节）两边都没写**，而走读照样按固定偏移切树表节点的条目——要不要把射程扩到它，账在 C476（盘上内容可控时走得到的 panic 有 21 处） | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::walk_tree_table_entry`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；条目宽为 0 时条目数非 0 那一条 2026-09-27 实审 A3-checker-2 落地：`index_node_view_judging_a_zero_entry_width`（`crates/singlefs-checker/src/walk.rs:747`）在解码失败理由是 `Verdict::EntryWidthZeroWithEntries`、且这棵树登记了条目字段表（`EntryFieldTableRegistration::Registered`）时报违例；树表条目与中央映射树不在射程内，只记走读失败；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
| I-1.11 | 映射 key 与单元头相符 | 任一中央映射条目，key 的类标签属于 D18（块里携带什么信息） 已定项 11 的登记表，且等于它的位置条目指的单元头里的类标签；key 里的出生 txg、实例代号、尾段（码 1 是写序的事务号 6 字节；码 2 / 码 3 是出生序号 4 字节，其后 2 字节补零恒 0）分别等于那个单元头里的诞生代号、写序实例代号、事务号或出生序号；key 里的出生树等于那个单元头里的出生树（码 1 / 码 3 取类身份段偏移 43，码 2 取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定）。不等判损坏。 | 已实现（2026-09-27，池级 checker `walk::check_pool_image`，判定在 `crates/singlefs-checker/src/walk.rs` 的 `walk_central_mapping_entries`；key 各段的偏移按 D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10 的字段表另写一份，不用实现的解析；判到的射程：类标签在登记表里且等于被指单元头的，出生 txg / 实例代号 / 尾段等于单元头的，码 2 / 码 3 尾段后 2 字节补零恒 0，码 1 / 码 3 的出生树等于单元头偏移 43，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不判；映射根全零（无文件的一版）时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（映射里数据单元那条 key 的出生 txg 3 → 4）与 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`。编号已落到代码：`crates/singlefs-checker/src/image.rs:38` 的常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 现在写的值就是这个编号本身，I-1.11（映射 key 与单元头相符）） |

> I-1（块自描述） 类的适用范围是**任一单元**（数据与元数据都在内，
> D20（承重面：单元的原子性与自包含） 口径），判定字段的落点与宽度在
> [decisions.md](decisions.md) D18（块里携带什么信息） 已定项 7 的字段表（码 1 / 2）与已定项 11 的类身份段表（码 3）；
> 自证单元（journal 记录 / 根槽 / 系统配置槽）对 I-1.1（块头自述逻辑地址） 与
> I-1.3（块头树 ID 一致） 显式豁免，豁免的依据是它们的自证纪律另有落点
> （I-7（根环） / I-8（journal） 两类与 D22（单元原子性怎么合成） 已定项 21）。
> ⚠️ **豁免行是穷举的，按 D18（块里携带什么信息） 已定项 11 的登记表**：码 1 / 2 / 3 在字段表里有类身份段，码 16 / 17 / 18 在豁免行里点名，
> 其余码即判红（I-1.6（类标签登记且三处相等））——这是「穷举的显式分支」作用在不变量射程上。

```

**出处 `.claude/kb/invariants.md:45-76`（整段抄，未转述）**

```markdown
## I-7 根环 —— 类

**依赖 D22（单元原子性怎么合成），形态未定。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-7.1 | 根槽有效集非空 | 全部根槽位中，**自证校验和通过**的记录集合 S 非空 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.2 | 最新根可完整遍历 | S 中代号最大者，其 checkpoint 树根指针出发按 D4（校验和位置） 递归校验，能走完全部权威态（[decisions.md](decisions.md) D21（权威态与派生态的分界） 定义：单元 + 记账），无一处失配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.3 | 环健康性 | **环健康性**：S 中除代号最大者外，至少还存在一条更早代号的记录。不存在即判红——它意味着某次提交把上一代直接覆盖了，轮换逻辑已失效，**下一次撕裂将无路可退**。**例外只有一个**：S 全部是第 0 代（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成）已定项 8）时判绿，崩溃后回退到这个态的镜像同样判绿。⚠️ 射程：种子没被覆盖完之前，它们自己就充当「更早代号的记录」，连续两代落进同一槽的轮换 bug 要等 R × S 次发布把种子全部覆盖之后才会被这一条抓到 | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，例外（自证过的根全部是第 0 代）照判据原样实现；坏镜像建在 mkfs 刚写完那份上（三条第 0 代根的 checkpoint_txg 一起改成 1），例外那一支另有一份阳性对照；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。**「K 代」2026-09-13 起就指这个候选集**：平时回退下界 F 不动时是整个根环，盘紧时抬 F 缩到最近 4 个可退到的状态、且每块盘上最新的持久有效根都在内（D16（发布语义） 已定项 1）；原「K 是运行时策略、下限 ≥ 2」作废。[decisions.md](decisions.md) D22（单元原子性怎么合成）：**根环深度是块重用延迟的上界**（D22（单元原子性怎么合成）同时明令块复用的界不能是格式常量），不满足则环里的旧根是假的回退候选。「有效」按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的回退候选集判：(i, T) 有效 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被抛弃时间线的根不在候选集里，这只管回退目标的选择；被抛弃时间线今天只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。它们引用的块在离开根环之前同样不许重新分配、不许抹头（2026-09-13 用户定案，C314（回退可以复用被抛弃的根引用的单元） 取影子账，D23（journal 的角色与格式） 已定项 14；留着的理由是第一轮判决 H6：崩溃恢复落到旧根之后，被抛弃的根引用的单元被复用、之后恢复落在它上面读不出）。⚠️ F_生效 按 D16（发布语义） 已定项 1「生效」取根上与系统配置里读得出的最大值；checker 今天只读根上带的 F，带 F 的根全坏时会在合法状态上误红这一条与别的按回退候选集判的几条（第三轮判决 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。崩溃后 defer 队列的内存态丢失，接手的是清扫准入「释放代 ≤ max(F_生效, 环里最旧有效根)」那条闸（谓词由 D16（发布语义） 已定项 1 给，2026-09-13；有效性按实例表判），保护宽度随之变 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，按每条回退候选集里更早的根各判一格：走读它引用的单元时校验和对不上或头用不了，就是它指着的块被复用或抹头；坏镜像是 mkfs 树表单元（只有第 0 代根还引用）改内容重封；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 被抛弃、还在根环里的根也判（2026-09-27，用户定「checker 现在就补被抛弃根那一半」，代码审阅第 7 条）：最新根的实例表有行时，根环里每条被判抛弃的根（有行 (i, Ti, ·) 且 T > Ti）各用一份自己的走读走一遍，只看读回来对不对——走出 I-2.1（校验和与内容匹配） 违例或走读失败就按这条根判红，判定丢掉、引用不进 I-3.1（已分配统计对得上） 的并集（`walk::judge_blocks_referenced_by_abandoned_roots`，用例在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`）；根环有读不出的槽且最新根的实例表有行时，没有违例就整条报不适用、不报成立；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形在这一半上照实红，等 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 修好转绿。 |
| I-7.5 | 根槽按判定宽度对齐 | 每个根槽的**起始偏移**是挂载时探测的 `physical_block_size` 的整数倍。**逐槽判，不是只判第 0 个**——偏移半个槽的槽照样跨 2 个原子单元（E41（根环槽几何 2×4×256） 实测：对齐时风险槽数 1，偏移 256 时是 2），撕裂会连带打掉相邻的槽而没有任何东西报警 | 未实现 ⚠️ **射程注（2026-09-13，D2（RAID 条带策略） 已定项 15）**：`io_min` 大于 `physical_block_size` 时按 `io_min` 判——已定项 17 收窄后根槽与系统配置槽在它的射程内。 |
| I-7.6 | 根环区域落在互不相同的盘上 | R 个区域**在 mkfs 时逐区域写下的设备身份**（D2（RAID 条带策略） 已定项 7 用户定案逐字「归属不再按 `(r × P) mod devs` 算，mkfs 时逐区域把设备身份写下来」）取满 min(R, devs) 个不同值，且没有一块盘背超过 ⌈R / devs⌉ 个区域（鸽笼下界，D22（单元原子性怎么合成） 已定项 14，2026-09-13 用户定案）：devs ≥ R 时就是两两不同；devs < R 时不拒绝挂载，容错承诺按盘数降级（E87（固定结构的放置） 量过的 2 盘 R = 3：掉盘 1 最坏 1 代、掉盘 0 最坏 2 代）。取值不满或某盘超过下界即**拒绝挂载**——名义容错 F 个失败域而实际区域挤在同一块盘上，是 E47（根环失败域的损失） 实测过的形态（`even_span` 在 18 个几何格里只有 3/18 真的落到 4 块不同盘） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.7 | 系统配置实例代号不低于根环 | 可写挂载独占打开成功的那个集合里（它至少是过半，过半是准入门槛不是写入范围），每块盘的系统配置实例代号按它两槽里全部自证过（校验和过且 fsid 与本池相同）的槽读，取号用同一个读法（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者，任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」；② 各盘的实例代号（每盘取它全部自证过的槽里最大的那个）不等时，较大者不出现在任何根记录、journal 记录、单元写序里，镜像记不下这次挂载独占打开的集合时报「不适用」，第一版两盘就是全部盘。取号没做完、全或无失败回卷期间的代号不等由 ② 判成立，不另开例外（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的取号规则）。只在 devs ≥ R 时根环才是系统配置之外的独立见证，第一版 R = 3 > devs = 2 时两路见证的失败域是同一对盘 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 用户定案改成 ① ②**：checker 读每盘两槽里全部自证过的系统配置，扫描方向加读 journal 环与三类单元头的写序；层 0 全量 262165 个状态 I-7.7（系统配置实例代号不低于根环） 违例 0（旧定义「各盘择到的相等且 ≥ 根环」下取号只落一块盘的 2 个状态判红）；坏镜像三份：没人引用的实例 2 孤儿（①）、journal 环里实例 2 的记录（①）、盘 0 最大号 2 而孤儿带 2（②） |
| I-7.8 | 根记录树 ID 水位不低于全池最大树 ID | **根环里全部根记录的「树 ID 水位」字段取 max，必须 > 该池中出现过的最大树 ID。**「出现过的」由 checker 全盘扫描算出：全部单元头的索引节点类身份段里那个树 ID（D18（块里携带什么信息） 已定项 7），并上树表条目里的树 ID；**不许与运行时共用同一段代码**（D13（验证路线） 已定项 5）。取 max 的范围是「根环里**全部**根记录」不是「回退候选集」——被抛弃时间线的根仍然承载它那一代发过的号，形态与 D23（journal 的角色与格式） 已定项 14 给 checkpoint_txg 取值那句同源。**判别力**：抓两类——① 某次发布没更新水位；② 水位被实现成「本 checkpoint 发出的最高号」而不是**累计高水位**（后者的镜像是连发 R + 1 个 checkpoint 各建一棵树，第一棵的号会随根轮出环而被重发）。⚠️ **只许写成不等式**：水位是「下一个可用号」的上界，写成等式会把正常镜像判红 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 崩溃孤儿两条流（第一个文件版本那次发布的每个前缀 → 可写挂载 → 挂载之后六种动作）在 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs`；中间实例行（T = 0）不排除的坏镜像是 `checker_known_bad_images.rs` 的 `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`（实例 2 的根槽全清零之后实例 3 写中间实例行 (2, 0, 0)，根环水位压低之后只红 I-7.8（根记录树 ID 水位不低于全池最大树 ID）；回退行那一道随回退行删掉，D18（块里携带什么信息） 已定项 11）；实现在 `walk::written_after_its_instance_was_last_applied` ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲；2026-09-24 主 agent 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 1 条补第二道，被攻过零轮）**：崩在发布之前的那个事务写出的孤儿节点不算「出现过」。扫描方向不数两类码 2 节点：① 诞生代号超过根环里最大 checkpoint_txg 的；② 写序实例 i 在最新根指着的实例表里有行 (i, Ti, ·)、Ti > 0、而诞生代号 > Ti 的——行说实例 i 只被施加到 Ti，之后写的没发布过；下一个实例第一次发布取的 txg 可以等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 取等号），只靠 ① 挡不住（第一个文件版本崩在记录落盘之前、再可写挂载的两条流上 130 格在合法状态上判红）。Ti = 0 的行（恢复写的中间实例行）不排除：中间实例发布过的号仍要算（D8（核心索引结构） 已定项 8 ②）。节点头 fsid 不是本池的照旧数。按字面数孤儿，层 0 全量 262165 个状态里 261891 个判红（水位 11 而盘上已有树 ID 11–15 的孤儿节点），见 records 2026-09-13-总审核 十一·七 |
| I-7.9 | 回退下界 F 不高于抬 F 的上限 | **抬 F 的那一条根**（它带的 F 比同一实例里 txg 比它小的最新那条根带的高；新实例的第一条根没有同实例的前一条，不算抬——它带的是恢复算出的 F_生效；管理员回退是现行实例里的一次向前发布，带的 F 与前一条相同，不算抬；一次抬 F 推的几次空发布只有第一条算）根记录里那个回退下界 F（D22（单元原子性怎么合成） 已定项 7 的字段），按这条根 flags 里的**卸载记号**（D22（单元原子性怎么合成） 已定项 7 flags 位 0）分两支判：**带卸载记号的**，F 不高于根环里 txg 比它小的最新那条根的 txg（正常卸载把 F 抬到现行那一版的 txg、不另判上限，D16（发布语义） 已定项 1「抬 F 的上限」）；**不带的**（准入抬 F），F 不高于**用它之前的根算出的** D16（发布语义） 已定项 1 的准入抬 F 上限：min(每块盘上最新的持久有效根的 txg, 第 4 新的非空持久有效根的 txg)，非空有效根不足 4 个时取最旧有效根的 txg。有效 = 按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F；「之前的根」取根环里 txg 比它小的那些。只判抬 F 的那一条，不在后来每张镜像上重算：F 抬过之后再回退，后来的根会把当初撑着 F 的非空根判成无效，现算的上限会低于 F，而 F 不回落（用户定案 2026-09-24，理由见实二报告：按「有效根不要求 txg ≥ F」的读法，固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与两个随机种子在合法状态上判红）；非空 = 它的树表里 inode 树、extent 树的根指针与它前一条有效根（按 txg 排）树表里的不同——比的是树表里这两棵树的条目，不比树表单元自己的落点（D16（发布语义） 已定项 1 表格后那段「「非空」从盘上怎么认」）；数「第 4 新的非空」时去重：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入（同一段）。⚠️ 它判的是一张镜像上 F 有没有抬过头，不判 F 该不该抬（那是准入的事，D28（挂载期承诺量） 已定项 1）。⚠️ **卸载记号盘上没有别的东西核它**：准入入口误打记号、F 又抬过头，按带记号那一支放过（第三轮判决第二节 K6：(b) 丢的正是「准入抬过头却带了卸载记号」那一格）——这一格归层 0 录制流另核「卸载记号只由卸载入口打」（C557（卸载记号只由卸载入口打没有检查）），不归这一条。⚠️ F 写进了系统配置而带新 F 的根没落盘或坏了、恢复后生效值怎么取的那一类，归层 0 与 C556（checker 与层 0 不读系统配置里的 F），不归这一条。算上限要比的有效根树表读不出时，它空不空判不了、不猜：F 不高于把这些根全算空得出的上限就成立，高于全算非空得出的上限就违例，落在两者之间这条根不判 | 已实现，两支（不带记号那一支 2026-09-24、带卸载记号那一支 2026-09-26）：池级 checker `walk::check_pool_image` 的 `judge_rollback_floor_raises_against_their_ceilings`，根环里每条抬 F 的根各判一格，按根记录读出的卸载记号分支（checker 读成 `UnmountMarkerView`）。**不带记号那一支**：坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（步 5 那段历史上抬到上限 11 与之后再发一版都成立、抬到 12 只红它）；固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与随机历史各档不红；层 0 `crash_enumeration_fixed_script_stream.rs` 抬过 F 的三条流要求真被评估过；`crates/mutations.tsv` 7 条变异（判法拿掉、上限取大、第 4 新改第 3 新、有效按新 F 取、不限同一实例、一条都不认、「4 个不同状态」不去重）；用户 2026-09-24 定义，里程碑「覆盖写、释放、回退与复用」收口表第 26 行。**带卸载记号那一支**：坏镜像同一文件的 `marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant`（正常卸载之后的镜像按带记号那一支判 6 ≤ 6 成立、按准入那一支会红；把那条带记号的根的 F 改成 7 只红它）；变异「checker 把每条根都读成不带卸载记号」「带记号那一支把不高于前一条根的 txg 判成低于」；记号的写读在 `crates/singlefs-core/src/root_record.rs`（写者照 `unmount_marker` 写 flags 位 0、读者拒记号以外的位），写者、读者各有一条变异（实二） |
| I-7.10 | 已停用·回退见证表删了 | **此编号不再使用**——2026-09-26 管理员回退改成挂着时的一次向前发布，系统配置里的回退见证表删掉（D23（journal 的角色与格式） 已定项 14、D22（单元原子性怎么合成） 已定项 9）⇒ 这一格**无对象**。曾要求：每块盘两槽里自证过的系统配置槽，槽内偏移 481 起的回退见证表解得开（条数不超过 R × S − 1、条目严格升序、目标实例代号小于新实例代号、条数之后全 0），同一个新实例代号在各盘各槽里记的回退目标相同 | 不适用 |
| I-7.11 | 已停用·见证择根删了 | **此编号不再使用**——2026-09-26 回退见证表删掉（D23（journal 的角色与格式） 已定项 14），择根不再跳过被见证抛弃的根 ⇒ 这一格**无对象**。曾要求：所选根是根环里自证过、不被回退见证表抛弃的根里 (txg, 实例代号) 最大的那一条；根环里至少一条自证过的根不被见证表抛弃；见证表里新实例代号不大于所选根实例代号的每一条，所选根指着的实例表罩得住它 | 不适用 |
| I-7.12 | 系统配置 F 不低于同盘根上的 F | 每块盘分开判：这块盘两槽里自证过（校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）的系统配置槽中回退下界 F（D22（单元原子性怎么合成） 已定项 9 的字段）的最大值，不低于这块盘根槽里每一条自证过的根带的 F（D22（单元原子性怎么合成） 已定项 7 的字段）。抬 F 那一串先把新 F 写进每块盘的系统配置、过一道屏障，才发第一条带新 F 的根（D16（发布语义） 已定项 1「抬 F 那一串」），所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里。某块盘两槽都自证不过时，那块盘不判；一块盘都判不了时整条报「不适用」。判别力：把「先写系统配置」挪到第一条根之后，崩在两者之间的状态必须红 | 已实现（2026-09-26，池级 checker `walk::check_pool_image` 的 `judge_system_configuration_floor_against_the_roots_on_each_device`：每块有自证过的系统配置槽的盘判一格，那块盘上一条根都没有时那一格成立；两槽都自证不过的盘不判，一块盘都判不了时整条报「不适用」。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant`：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12（系统配置 F 不低于同盘根上的 F） 真被评估过且成立，盘 1 两槽的 F 改回 0 只红它、红在盘 1 txg 16 那条带 11 的根上；变异「不比系统配置里的 F（每块盘恒成立）」「checker 不读系统配置里的 F（读成 0）」（实二）。条款仍是主 agent 2026-09-26 按 SysPre 的写序推的，没三方；层 0 每个崩溃状态都跑池级 checker，按判法每个状态都评估得到它（推的，层 0 没跑） |
| I-7.13 | 系统配置池级字段在读者收的范围里 | 任一自证过（magic 与整槽校验和对，`check_system_configuration_slot`）、incompat 位认得的系统配置槽（字段住 D22（单元原子性怎么合成） 已定项 9 的字段表，读者收的界在 D22（单元原子性怎么合成） 已定项 2 与已定项 16）：格式版本 = 1（`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）、加密类型 = 0（第一版恒关，`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`）、固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号（`geometry_of` 的 `fixed_structure_slot_spacing_lies_in_the_format_range` / `physical_block_size_fits_a_root_slot` / `journal_ring_bytes_lie_in_the_supported_range`）。任一盘任一槽不满足其中一项即判红：checker 报违例、不作保，且该池其余不变量一律报「不适用」，与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）。**判的字段与 core 收同一张表**：格式版本、加密类型、槽距（上界用同一槽自述的根环起点）、根槽宽、环长、单元区起始槽号（偏移 417）= 环长现算的起点且在 64 槽段边界上、journal 环起点（偏移 325）与根环起点（偏移 371）等于第一版常量，共八项（D22（单元原子性怎么合成） 已定项 16 第 1 条）。R（区域数）与 S（每区槽数）越界不在这一条里，仍归「这一槽不可择」 | 已实现（2026-09-27，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_system_configuration_values_the_reader_accepts` 在 `walk::check_pool_image` 里调用，任一盘任一槽带越界值时该池其余不变量整批报「不适用」并提前返回。**八项 checker 与 core 两边都判**：checker 解槽时判格式版本与加密类型（`crates/singlefs-checker/src/lib.rs`），其余六项在 `geometry_of` 判——先判 journal 环起点（偏移 325）= `JOURNAL_RING_START_SLOT`（1024）、根环起点（偏移 371）= `ROOT_RING_BASE_SLOT`（64）（两个起点先于槽距判：槽距上界按同一槽自述的根环起点算），再判槽距、根槽宽、环长，最后判单元区起始槽号（偏移 417）= 同一槽自述的环起点 + ⌈环长 ÷ 16384⌉（`unit_area_start_slot_is_the_slot_after_the_journal_ring`）且是 `CLUSTER_SEGMENT_SLOTS`（64）的整数倍；新加的四判各报一个 `Verdict` 成员（`JournalRingStartSlotNotTheFirstVersionConstant`、`RootRingBaseSlotNotTheFirstVersionConstant`、`UnitAreaStartNotTheSlotAfterTheJournalRing`、`UnitAreaStartOffTheClusterSegmentBoundary`），都归 `value_refused_by_this_reader`（整池不作保）；常量取自 `singlefs-format`，算式 checker 自己写。core 一侧 `crates/singlefs-core/src/system_configuration.rs` 的 `parse_slot` 读出偏移 325、371 两个字段，`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 按格式版本、加密类型、槽距（上界用同一槽自述的根环起点，`largest_fixed_structure_slot_spacing_under_the_root_ring_base`）、根槽宽、根环起点、journal 环起点、环长、单元区起点的次序判，不收的值报 `SystemConfigurationValueOutsideWhatThisReaderAccepts` 的一个成员（两个起点是 `RootRingBaseNotTheFirstVersionSlot`、`JournalRingStartNotTheFirstVersionSlot`），经 `RecoveryFailure::SystemConfigurationValueRefused` 整池拒。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：`a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`（盘 1 槽 0 加密类型改成 1、重算整槽校验和，只红 I-7.13（系统配置池级字段在读者收的范围里）、其余不变量全报不适用），`each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`（几何字段各一格，各只红 I-7.13（系统配置池级字段在读者收的范围里）、违例说明点名被拒的那个成员）。两边收同一张表由 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 钉住：新池新建文件之后的镜像上两块盘四个系统配置槽改同一个几何字段、整槽校验和重封，五格各一条用例，每格 core 的只读与可写挂载都拒成同一个 `SystemConfigurationValueRefused` 值、两块盘逐字节不变，checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、其余不变量报不适用；根环起点写 0 那一格两边点名的成员不同（core 先判槽距上界、报槽距，checker 先判根环起点、报根环起点），照两边各自的判定次序钉（用户 2026-09-27 定「读字段，不等于常量就整池拒」，`records/2026-09-24-里程碑二收尾调度.md` 第 293 行；代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节「core 与 checker 对系统配置几何字段的读法」那一格）|
| I-7.14 | 系统配置本盘设备号等于盘在池里的身份 | 每块盘两个系统配置槽里，每个自证过（magic 与整槽校验和对）、fsid 与本池相同的槽，自举头里的本盘设备号等于这块盘在池里的身份。字段住 D22（单元原子性怎么合成） 已定项 9 的系统配置字段表（自举头「本盘设备号 4」）；实现一侧是 D18（块里携带什么信息） 已定项 11 里挂载核盘表那一句：可写挂载比盘表（`crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`，`OwnDeviceNumberDiffersFromTheIdentityHandedIn`）。别的池的槽归 I-1.4（块头 fsid 一致） 判，不在这一条里 | 已实现（2026-09-28，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_own_device_number_is_the_device_identity` 在 `walk::check_pool_image` 里调用，每块盘每个自证过、fsid 属于本池的系统配置槽判一次，本盘设备号与 `ImageReader::devices` 给的这块盘的身份比。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：两块盘四个系统配置槽的本盘设备号互换，`devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant` 钉只红 I-7.14（系统配置本盘设备号等于盘在池里的身份），这份坏镜像也在 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 的清单里；变异表 `crates/mutations.tsv` 里对着这一判的三行（判定恒真、池级走读不调这一判、坏镜像写回原值）改法 A 的 `prove-red.sh` 各抓到（`research/prompts/m2-impl-r2-fixes-a-implementer-report.md` 第 77–78 行）。合法历史上不判违例：门禁 harness-model-differential-and-scenarios 随机历史那一趟 I-7.14（系统配置本盘设备号等于盘在池里的身份） 判绿 4155 次、不适用 0 次（同一报告第 189 行）；层 0 快档与全量没跑）|

> ⚠️ **I-7.3（环健康性） 的价值在于它不需要真的崩溃一次才能发现 bug。**
> 轮换逻辑写错（例如连续两代落到同一槽位）在**下一次正常提交后的 checker 扫描**里就会被抓到，
> 而不是等到某次撕裂真的发生、发现无处可退时才暴露。
> 这与「校验和会不会红」是两类判别力：前者查机制活着没有，后者查数据对不对。

> ⚠️ **I-7（根环） 类验不了「失败域独立」这个假设。** 根环的多区域布局依赖
> 「盘首与盘尾的物理故障 / 掉电撕裂事件统计独立」，而对着镜像判不出这一条——
> 它只能验证「机制按设计跑了」，不能验证「设计的假设是真的」。
> 该假设**本机无证据**，见 [decisions.md](decisions.md) D22（单元原子性怎么合成） 已定项 2。

```

**出处 `.claude/kb/invariants.md:77-115`（整段抄，未转述）**

```markdown
## I-8 journal —— 类

**依赖 D23（journal 的角色与格式），形态已定（定长环 + tail 住系统配置槽），checker 未实现。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-8.1 | 环几何够大 | `环大小 ≥ F × 任一事务的最坏 journal 占用`，F ≥ 2。最坏占用与 F 写进系统配置，mkfs 时算、mount 时复算比对 | 未实现 ⚠️ **口径注（2026-09-13，D28（挂载期承诺量） 已定项 4）**：ckpt_cost 按当时各记录树的高现算、挂载期内会长，mkfs 时声明的最坏占用要么取上界、要么挂载期内也复算比对——哪一种没定。 |
| I-8.2 | 记录头不跨原子单元 | 任一 journal 记录的头完整落在一个 `physical_block_size` 单元内。该宽度**运行时探测**，不许硬编码 | 未实现 |
| I-8.3 | 重放前缀严格连续 | 被重放的记录集合，其 `jsn` 严格连续（`jsn == expected`，断号即止；链从所选根覆盖的最后一条记录之后接，跨实例边界即停，记录水位按实例代号为主比——D23（journal 的角色与格式） 已定项 14 的注，2026-09-13），且整段落在在飞记录数上限之内。⚠️ **这一条不是前缀纪律的全部**：D23（journal 的角色与格式） 已定项 14 的前缀口径共五条，这里只落了其中三条半——**「提交标记齐全的事务才施加」与「施加前逐项验证点名单元」两条不在这条判据里**，而两者的处境不同：前者 2026-09-23 已落成 I-8.8（前缀里的事务不被切开），checker 判定与崩溃点重放那一半仍欠，记在 C37（事务与记录的对应无定义）；后者**已经有了**，只是不住这一条——它住 D16（发布语义） 已定项 7，两条故障注入自证 2026-09-14 在层 0 做出（C76（发布时序没有权威落点））。⇒ 照这一条实现恢复施加逻辑的人会漏掉那两道防线，而它们正是挡住「半个事务」与「点名单元已损坏的记录」的 | 未实现 ⚠️ **判据补一句（2026-09-13，D16（发布语义） 已定项 4）**：前缀不得停在一次发布的中间——一次发布整体施加或整体不施加。 ⚠️ **再补一句（2026-09-14，D23（journal 的角色与格式） 已定项 18 用户定案）**：「在飞记录数上限」的语义就是**这个前缀最多 N 条**（N = 环槽数 ÷ F），不是 XFS 那种「最后 N 条内算撕裂、之外算损坏」；**任一校验和不过的记录一律断链即止**——停在它之前那一条，第一版不区分撕裂与损坏、不因此拒绝挂载。 |
| I-8.4 | 重放幂等 | 任一 journal 记录被重放两次与重放一次，产生的盘上状态相同 | 未实现 |
| I-8.5 | tail 槽有效集非空 | 全部 tail 槽位中自证校验和通过的记录集合非空，且其中除代号最大者外至少还有一条更早的 | 未实现 |
| I-8.6 | 反向链算法 | 任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 311 字节头，`header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录反向链恒 0**（D23（journal 的角色与格式） 已定项 10 / 已定项 19，2026-09-13 用户定案，链按实例算是 2026-09-14 用户定案，头宽随 D23（journal 的角色与格式） 已定项 4 现行 311）。不等的记录不进重放前缀。jsn 的计数器为 0 的记录判违例：它没有本实例内的逻辑前一条，反向链无从定义（D23（journal 的角色与格式） 已定项 18 的落点式 `(计数器 − 1) mod 槽数 × 4096` 从计数器 1 起；用户 2026-09-27 定）。⚠️ 链按实例算，所以跨实例边界的两条记录之间**不判**这一条——实例边界由实例代号挡着（I-8.3（重放前缀严格连续)） | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，逐盘判：计数器 1 与跨实例边界的第一条判反向链恒 0，同实例前一条在盘上时判链值，前一条不在盘上不判；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 里环还空着的 4 个状态报不适用，其余每个崩溃状态都判）；计数器 0 判违例（2026-09-27，`walk.rs` 的 `ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`，用例 `a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant` 在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`） |
| I-8.7 | 实例内事务号不重号 | 判环里全部自证过的记录，同一实例按 `jsn` 的计数器看：① 按计数器相邻的两条非 0 记录，事务号不减（事务号 0 是不承载事务的空发布记录，不进这个比较）；② 一个事务号的记录在计数器上连续，被别的记录（含事务号 0）隔开之后不再出现。D23（journal 的角色与格式） 已定项 7 逐字：「事务号按实例计数、从 1 起」「记录按事务号顺序追加……这是实例表行的 W 能当精确前缀的依据」；同一项定一个事务可以跨多条记录，所以相邻两条可以同号。 | 已实现（2026-09-21 立时判「非 0 事务号严格递增」；2026-09-23 按 D23（journal 的角色与格式） 已定项 7 改成不减加同号连续。池级 checker `walk::check_pool_image` 的 `judge_transaction_numbers_per_instance`：逐盘按实例判，同一实例按计数器相邻的两条非 0 记录，号不等时后一条严格大于前一条（①），号相等时两条之间不许坐着同一实例的别的记录（②，事务号 0 的也算；中间只有读不出的槽或别的实例的记录不判红）；「被别的非 0 号隔开之后又出现」一定伴着一处 ① 红，不另判；同一实例非 0 的记录不足两条时报不适用。坏镜像三份（交错两份、事务号倒退）在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，每份只红这一条，违例说明标出红的是 ① 还是 ②） |
| I-8.8 | 前缀里的事务不被切开 | 判环里全部自证过的记录：③ 同一实例同一事务号的一组记录里，**带提交标记的至多一条，且它是这一组计数器最大的那一条**；提交标记那一字节只许 0 或 1；④ **一组里一条提交标记都没有时，这个实例在它后面不许再有任何记录**（事务号 0 的空发布也算；别的实例的记录不算）：D23（journal 的角色与格式） 已定项 7 逐字写「一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加」，实例切换之后这个实例不再写任何记录。它与 I-8.7（实例内事务号不重号） 的 ① ② 合起来，保证 D23（journal 的角色与格式） 已定项 7 原话那个恢复算法「丢掉提交标记还没出现的那个事务的全部记录」**恰好丢掉、且只丢掉**被崩溃切开的那个尾巴；`crates/singlefs-core/src/recovery.rs` 今天那一步写成「按 jsn 扫到第一条不带提交标记的记录就停」，只对一事务一条成立，一事务跨多条记录时要改，归并行线一。四条里任一不成立，恢复要么丢掉一个已经提交的事务、要么施加半个事务，而盘上没有任何痕迹。⚠️ **今天可达镜像上它恒真**：第一版一条记录一个事务、`is_commit` 全仓没有一处写 false，所以 ③ ④ 今天都不会被可达历史违反。**判别力只能靠坏镜像**，坏镜像不做这条就是恒真的检查——恒真的检查比没有检查更糟。⚠️ **它管的是条款允许的形态，不是第一版的形态**：D23（journal 的角色与格式） 已定项 7 定的是「可以跨多条，记录头带事务边界字段」，而第一版按 C310（事务切分纪律与记录数口径打架） 一条记录一个事务。I-8.8（前缀里的事务不被切开） 挡的是**一事务跨多条记录落地那一天**。⚠️ **它只判一份镜像上的记录流自洽，不判「施加的结果等于整数个事务」**：后者要拿「这次事务该有的全部效果」当 oracle，一份镜像上判不出来（形态同 I-4.3（提交原子） 那一行），归层 0 崩溃点重放。 | 已实现（2026-09-23，池级 checker `walk::check_pool_image` 的 `judge_commit_markers_per_transaction`：逐盘，一组 = 同一块盘上实例代号与事务号都相同的非 0 记录；③ 两条以上的一组里带提交标记的至多一条、且是计数器最大那条；提交标记字节每条记录都判（事务号 0 的也算），不是 0 或 1 即红；④ 一条提交标记都没有的组，后面同实例还有任何一条记录（事务号 0 的空发布也算）即红。没有 ③ 的对象也没有 ④ 的对象 ⇒ 报不适用，提交标记字节单独成立不算成立；第一版一事务一条、每条都带提交标记，层 0 每个崩溃状态都判、全部报不适用。阳性对照（一事务两条、提交标记在 jsn 最大那条，I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 都成立）与四份坏镜像（提交标记放错位置、没提交的事务后面还有号更大的记录、没提交的事务后面跟一条空发布、提交标记字节写成 2，每份只红这一条）在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，违例说明逐条标出红的是哪条判据） |
| I-8.9 | 一次发布的记录序号连续且只有末条带标志 | 同一实例、同一 `checkpoint_txg` 的记录（一次发布写出的全部记录）：「本次发布内序号」依次是 1..N、与 jsn 同步（jsn 连号时序号也连号），记录标志位 0 恰好只在序号 N 那一条上为 1，记录标志其余位为 0；序号 0、一次发布之内跳号、没有末条或多于一条末条，都是违例。只判读得出的记录，读不出的那条不判（D23（journal 的角色与格式） 已定项 4、17，用户 2026-09-24 定，`records/2026-09-24-里程碑二收尾调度.md` 第三节「序号与末条标志的不变量」） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_publish_ordinals_and_last_record_flags`；阳性对照与七份坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`） |

> ⚠️ **D23（journal 的角色与格式）的死锁 3 全靠 I-8.1（环几何够大），而 I-8.1（环几何够大） 的 checker 未实现。**
> 「写进本清单」不等于「有东西在拦」——取定长环那条论证依赖的是**拦得住**，不是**写下来**。
> 欠检查见 [checks-owed.md](checks-owed.md) C28（环几何不变量未实现）。

> ⚠️ **I-8.4（重放幂等） 是被实验逼出来的，不是设计时想到的。**
> [experiments.md](experiments.md) E24（恢复算法：先信 tail 会不会丢记录）测出：
> 全环扫描式恢复虽然不丢数据，却会把**陈旧 tail 以来的记录全部重放一遍**
> （窗口为空时仍重放满环 64 条）。没有幂等，「陈旧 tail 只是多做功」
> 就变成「重复施加已生效的记录」。

> ⚠️ **I-8.3（重放前缀严格连续） 与 I-8.1（环几何够大） 不许互相顶替。**
> E24（恢复算法：先信 tail 会不会丢记录）测到窗口撑爆环时（100 条 > 64 槽）
> **两种恢复算法一起丢**——恢复算法救不了几何。

> ⚠️ **I-8.4（重放幂等） 的射程有一条已实测的边界（E78（重放的起点），2026-09-02）**：
> 幂等管「同一条记录重复施加」，**管不到「旧记录的点名块已被合法复用」**——
> 陈旧 tail 的重放窗口远长于 I-7.4（近 K 代块未被复用）扣块的窗口，
> 窗口内记录的点名块换过内容后，施加时验证（E77（发布的持久顺序）证明它是承重步骤）会失配。
> 「陈旧 tail 只是多做功」因此只在不验证或有重放下界时成立；
> 重放下界的两条出路（根带 jsn 水位 / 盲放加尾删）见
> [checks-owed.md](checks-owed.md) C77（重放起点未定义）。

```

**出处 `.claude/kb/invariants.md:116-126`（整段抄，未转述）**

```markdown
## I-2 校验和 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红） |
| I-2.2 | 校验和算法全盘一致 | 校验和算法标识在整个文件系统内一致。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：系统配置自举头段「校验和算法标识 1」，登记表 0 = 无效、1 = CRC-32C，第一版写 1；读到 0 或未登记的码一律拒绝挂载。算法的四个参数写死在 D18（块里携带什么信息） 已定项 17 | 未实现（条款已写、检查仍欠） |
| I-2.3 | 补齐字节为零且参与校验和 | 任一数据单元（码 1）或打包记录单元（码 3，D18（块里携带什么信息） 已定项 11），其声明长度（净荷字节数；补齐起点 = 头 + 声明长度）之后到单元末尾的补齐字节**恒为 0**，且校验和 / MAC 覆盖**整个单元含补齐**（[decisions.md](decisions.md) D4（校验和位置） 已定项 2：不足一个单元的 extent 补齐到 32 KiB；码 3 由头校验和（到头末尾）与载荷校验和（头之后到单元末尾）两段合起来覆盖整个单元）。补齐字节非零、或校验和只覆盖声明长度，一律判红。索引节点（码 2）从 2026-09-13 起同样在射程内（D18（块里携带什么信息） 已定项 18 用户定案）：声明长度 = 条目区字节数，之后到节点末尾的补齐字节恒为 0，载荷 CRC 覆盖 [86 + 2 × key 宽, 16384)——起点是明文头末尾，含 29 字节预留位、条目区与其后的补齐区，口径与码 1 的 105、码 3 的 107 同（预留位 2026-09-14 从 28 加到 29，含预留位的头是 115 + 2 × key 宽）。⚠️ **加密开启时判的是解密后的明文，不是盘上字节**（2026-09-07 用户定案，还清 C189（I-2.3 在加密开启时判明文还是盘上字节没定））：补齐在盘上是 `E(0)` 而不是 0，按盘上字节判必然全红；checker 本来就要解整单元，取值同一步就有。**与 D18（块里携带什么信息） 已定项 7 的载荷 CRC 方向相反是对的**——载荷 CRC 要给无密钥一侧用，所以算盘上字节；这一条只有 checker 用，所以判明文 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节与指针头部的 MAC 16 + nonce 12（D19（块指针的结构与宽度预算） 已定项 3）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs:649` 起）另判单元头偏移 4 的格式版本 = 1（walk.rs:695，不认得就判该结构损坏、不按今天的字段表往下解）与 29 字节预留位全 0（walk.rs:730）；走读跟随的每条指针在 `judge_pointer_mac_and_nonce_are_zero`（`crates/singlefs-checker/src/image.rs:619`）判头部 MAC 16 + nonce 12 全 0，四个入口分别是码 2 节点指针（walk.rs:790）、inode 内部条目子指针（walk.rs:1440）、实例表链指针（walk.rs:1766）、extent 数据指针（walk.rs:1916），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
| I-2.5 | 位置条目按设备身份升序 | 任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17，2026-09-13 用户定案）；两条位置条目设备身份相同或逆序，一律判损坏。⚠️ **射程（2026-09-14 补）：整条 86 字节全零的指针表示「注册了但还没有根」，整条豁免升序判定**——它内部那两条位置条目都是 (设备 0, 槽 0, 校验和 0)，按字面正是「设备身份相同」那一档。今天写出全零指针的有四处：树表里 livelist 树、稀疏旁表树与 deadlist 树那三条条目（D6（快照实现模型） 已定项 2、D5（快照 / 空间记账机制） 已定项 6 与 D5（快照 / 空间记账机制） 已定项 11 各自定案的「day-1 注册进树表，根指针为零」），以及 mkfs 与暖机的根记录里那个还没发布过的中央映射树根指针。非全零的指针不享这条豁免。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |

```

**出处 `.claude/kb/invariants.md:127-159`（整段抄，未转述）**

```markdown
## I-3 空间记账 —— 类

**依赖 D5（快照 / 空间记账机制）。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**2026-09-17 起「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」）：checker 自己算，取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与层 0 不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于 F_生效 的（还在 defer 里，抬 F 还不许回收），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15「实例表单元指针照所选根」，走它施加在其上那一版（环里同实例、txg 比它小的根里最大的一条，不管在不在候选集里）的实例表链与根记录持有的分配记录树（`walk::walk_version_applied_only_by_records`，主 agent 2026-09-27 定，被攻过零轮）。⚠️ **已知缺口**：环里同实例、txg 更小的根一条都读不出时，那一版的实例表与根记录持有的分配记录树走不到，照旧漏数；「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`research/prompts/m2-rev-b2-implementer-report.md` 第七节第 2 条）。 |
| I-3.2 | 已停用·方向反了 | **此编号不再使用**——它写成「≤ 引用它的**最新**快照的代号」，方向反了：紧的约束是最**旧**的那个。旧写法被 I-3.5（引用区间的精确性） 的子判据 ② 严格蕴含，且检不出任何一类真实错误 | 不适用 |
| I-3.3 | 已停用·归属未定义 | **此编号不再使用**——「所属快照」未定义，而两种归属约定给出不同的比较符。由 I-3.6（deadlist 紧界） 取代 | 不适用 |
| I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ① `Reach(b)` 在 txg 序上**连续无洞**；② 设 R_first 为最早者，`birth(b) ≤ R_first.txg`，且若存在更早的根 R_prev 则 `birth(b) > R_prev.txg`（**双侧夹死，off-by-one 无处可藏**）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中 | 未实现 ⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
| I-3.6 | deadlist 紧界 | 任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）。判别力：抓「kill 时该走直接释放却进了 deadlist」这一类（泄漏方向） | 未实现 |
| I-3.4 | 可用空间扣待删占用 | 可用空间统计已扣除「待删除但意图未完成」占用的空间 | 未实现 |
| I-3.7 | 活快照可枚举 | 盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少） | 未实现 |
| I-3.8 | 实例表行唯一且低于挂载根 | 实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）；行的实例代号 < 挂载根实例（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2） | 已实现（2026-09-16，`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_rows`：行按实例代号唯一、每行实例代号低于最新根的实例、链指针记录在末尾；回收那一半没有输入，第一版行只增不删，删行的条件没人判） |
| I-3.9 | 释放代落在停止引用它的那一格区间里 | 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。若每一条有效根都还引用它，它不许带已释放标志（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））。环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条层 0 流的 12 个状态）按区间判，按「等于」判会在合法镜像上判红；已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。判别力：把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判） |
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：与最新根同实例、checkpoint_txg = 最新根的 txg + 1、带提交标记、根环里没有它的根的那条记录，新根段里树表指着的分配记录树（主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定，被攻过零轮；只接这一步——恢复连着施加几条记录时第二条及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
| I-3.11 | 已分配减 defer 等于最新根走读 | 逐盘：被判镜像上记账树里「已分配」那一行（记账第 1 项）减「defer 待释放」那一行（第 5 项），等于从最新有效根走读到的、这块盘上被引用的槽数。统计量的标签与行形态是 D5（快照 / 空间记账机制） 已定项 4 与已定项 10 的那几行，读的是镜像里的记账行、不读内存里的分配器；走读与 I-3.1（已分配统计对得上） 同一个走法，只取最新有效根。⚠️ 它补的是 I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 都够不着的那一种：写者把活单元记成已释放、放进 defer，已分配与空闲两个和照样对得上（alloc-basis 第三轮判决 `research/prompts/alloc-basis-r3-main-verification.md` 的 Z10）。⚠️ **判别力自证在造出来的基底上做**：E156（alloc-basis 四条岔路的代价数） 第 4 次重跑在造出来的基底 β_syn（第 5 项为 0）上再记 1 槽 defer，I-3.11（已分配减 defer 等于最新根走读） 判红、去掉「减 defer」判绿（`research/results/e156-alloc-basis-counts-2026-09-26-r2.out` 的 `E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`）。可达状态里第 5 项能不能为 0 没量到：挂载时回退（回退写行那次发布）今天没有了，同一份产物里挂着回退那一版第 5 项是 356 槽（`E7RESULT name=u13a_rollback_to_oldest_candidate overwrites=72 target_txg=52 txg_before=75 txg_after=76 instance_before=1 instance_after=1 instance_table_rows_before=0 instance_table_rows_after=0 item1_slots=374 item5_slots=356 i311=holds structure_holds=true`）。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 | 已实现（池级 checker `walk::check_pool_image`：逐盘判「defer 待释放 + 最新根走读 == 已分配」，最新根走读取 `walk_root(最新根)` 刚走完那一刻、与 I-3.1（已分配统计对得上） 同一个走法；最新根下面没有记账树时报不适用。坏镜像 `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`（盘 0 defer 行 +1 / −1 槽，只红它，I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧真被评估过且成立）；判别力自证在造出来的基底上 `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`（两盘 defer 挪回 0 之后再记一槽）；`crates/mutations.tsv` 5 条变异（判定恒真、不减第 5 项、`==` 放宽成 `≥` / `≤`、拿全部候选版本那一份走读比）；用户 2026-09-23 定采纳：alloc-basis 岔路单第 12 行（岔路 7）） |

> ⚠️ **I-3.5（引用区间的精确性）四条子判据双侧夹死 与 I-3.1（已分配统计对得上） 正交，两条都必须有。** I-3.1（已分配统计对得上） 是标量总账（已分配 == 遍历和），
> 而**归属错误对它不可见**——块被挂到错的 deadlist 上时，它仍然被恰好一处引用，总账依然平。
> 要检出归属错，必须有**按快照分箱的向量等式**。
> 这与 [decisions.md](decisions.md) D14（双轨（大小文件 / 持久临时）） 已定项 6 推导 D-3（多轨兼容可判据） 里「标量等式 → 按轨道分箱的向量等式，
> 判别力净增」是同一个形状。
>
> ⚠️ **数据丢失方向只在空间被重新分配之前可检出。** 该进 deadlist 却被释放的块，
> 一旦其空间被复用，症状就退化成 I-2.1（校验和与内容匹配） 校验和不匹配，即「损坏，原因不明」。
> **所以这类检查必须在崩溃点重放 harness 里紧跟着删除动作跑，不能只在夜间全量跑。**
>
> ⚠️ I-3.1（已分配统计对得上） 是**唯一允许全盘遍历的记账检查**——它是 checker，不是运行时路径。
> 运行时记账必须增量维护（D5（快照 / 空间记账机制）），checker 才用遍历来验证增量维护有没有出错。
> 这条对照关系是本工程验证记账正确性的核心手段。

```

**出处 `.claude/kb/invariants.md:160-177`（整段抄，未转述）**

```markdown
## I-4 事务与崩溃一致性 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-4.1 | 重放后树可遍历 | 任一崩溃点重放后，系统配置指向的树全部可完整遍历 | 未实现 |
| I-4.2 | 无被引用未提交块 | 任一崩溃点重放后，不存在被引用但未提交的块 | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`：走读到的每个被引用单元，头里的诞生代号与写序（D18（块里携带什么信息） 已定项 7 单元头字段表里的字段）按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判成已发布——**「未提交」取的就是「按那个谓词判不成已发布」**，谓词与 I-1.8（归并后版本全序） 扫描方向共用同一份 `PublishedPredicate`。打包记录类型 4 的实例表单元按 I-1.2（块头写序已发布） 那一行的例外不走谓词，不进这一条。坏镜像是把数据单元头里的诞生代号与引用它的 extent 记录里那条数据指针的诞生代号一起抬到最新根的 txg 之上——指针仍与头一致 ⇒ I-1.2（块头写序已发布） 不跟着红，分得开两条的就是这一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它。⚠️ **「任一崩溃点重放后」那半句没实现**：池级 checker 判的是手上这一份镜像、不跑恢复，判的是「被引用 ⇒ 已发布」这个态上的性质；重放之后再判一遍归层 0） |
| I-4.3 | 提交原子 | 提交是原子的：不存在"部分生效"的事务 | 未实现（它要判的是 D23（journal 的角色与格式） 已定项 7 那两个事务边界字段——事务号与提交标记——施加之后的效果。**对着一份镜像判不出来，没硬写**：「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽——同一份字节既可能是「事务整体没生效的旧态」，也可能是「生效了一半」，池级 checker 手上没有第二个输入。判它的是层 0 崩溃点重放加 oracle：`.claude/kb/milestone/02-second-txn.md` 第 486 / 487 行那两条（崩在第二条记录之前 ⇒ 整个事务不施加、文件是旧长度；把提交标记提前到第一条记录 ⇒ 恢复施加了半个事务、I-4.3（提交原子） 红）说的就是这条路。形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查）：操作断言，不进池级 checker，恒真的检查比没有检查更糟） |
| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 未实现 |
| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；判据落在 [experiments.md](experiments.md) E3（意图日志的断电语义） 判据③ | 不适用 |
| I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致（候选集的深度由 D16（发布语义） 已定项 1 的回退下界 F 定：平时整个根环，盘紧时缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内）。**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，最新根与候选集里每条更早的根各判一格：从它出发的遍历有单元对不上或读不出就红；坏镜像是 mkfs 树表单元抹成零；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态） | 未实现 |
| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在 | 未实现 |

> ⚠️ **编号只往上加，不回收、不重编。** 回收一个编号会让别处的旧引用悄悄指向另一条陈述，
> 而检索只端出其中一条，不会告诉你它换了意思（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条「矛盾比空白更糟」）。
> 退役的编号留一行说明去向，比留一个无解释的空档安全。

```

**出处 `.claude/kb/invariants.md:178-199`（整段抄，未转述）**

```markdown
## I-5 分配器 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-5.1 | 物理范围不重叠 | 不存在被两个不同引用同时占用的物理范围（除非是显式的多副本） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 引用按版判（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 1 条、`research/prompts/m2-rev-b2-implementer-report.md` 第八节）：同一版（根环里的一条根，或由记录施加出来的一版）里同一个 (设备, 槽) 被两条指针引用判红，(设备, 槽) 取自位置条目的设备 ID 与物理偏移（D19（块指针的结构与宽度预算） 已定项 4），一条指针自己的两条位置条目同落点算一个引用（那一格归 I-2.5（位置条目按设备身份升序））（`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`）；中央映射条目指的物理范围算进引用集合；别的版本走过的共享子树不重走，按缓存把它下面的落点并进这一版再比、不读盘（`walk::Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`）。坏镜像在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`。 |
| I-5.2 | 空闲统计对得上 | 空闲空间统计 == 总空间 − 已分配空间 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-5.3 | 报出的空闲都兑现得了 | `df` 报给用户的空闲 ≤ 当下可分配 + 能在有界步数内放回的扣住量（defer 窗口里、回退下界以下的释放）。运行时的等价形态：`df` 报出 s 字节空闲时写 s 字节必须成功；删掉 s 字节之后同样大小的写在有界步数内成功（[decisions.md](decisions.md) D3（空间分配） 已定项 9，用户定案）。界随 D16（发布语义） 已定项 1 的形态定。判别力：把「放回扣住量」那一步摘掉，近满盘上删一个对象再写同样大小必须由绿转红 | 未实现 ⚠️ **口径注（2026-09-13，D28（挂载期承诺量） 已定项 4）**：`df` 要扣的 checkpoint 保留池是每次发布现算的量，发布即重算，近满盘上报出的空闲会随树高变小。 |
| I-5.4 | 分配记录罩住的槽互不相交 | 任何一条有效根的分配记录树里，同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交。复用一个已回收的落点时，新记录罩住的槽上别的已回收记录要随这次分配一起删掉；留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_allocation_records_disjoint`，按回退候选集里每条根判；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant`；层 0 每个崩溃状态都判，同日在新代码上全量：第二条流 2,104,413 个状态里评估 1,842,252 个、违例 0，其余 262,161 个报不适用，第一条流 262,165 个状态里评估 4 个、违例 0（`m2-wave2-crash-verifier-report.md`「主 agent 要的两个数」）。同日立，依据 `research/prompts/m2-wave2-code-r1-main-verification.md` 第二节第 1 行：复用时跨度变了留下重叠记录，下一次可写挂载在重建分配器时 panic，而 I-5.1（物理范围不重叠） 只管树里的引用、判不出记录之间的重叠） |

> ⚠️ **I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 的等式在 zoned 布局上有一个已知缺口（[decisions.md](decisions.md) D12（目标介质） 定了多布局之后才出现）。**
> zoned 设备上一个 zone 里只要还有一块活数据就不能 reset ⇒
> **一个活快照钉住的物理空间是 zone 粒度，而它引用的块数是块粒度**。
> 于是「已分配 == 遍历所有引用得到的和」在那套布局上不成立，
> 除非把「已分配」明确定义为「被引用」，**并另立一个「不可回收」量**。
>
> 连带一条对用户可见的要求：**报给用户的「这个快照占了多少」不许被定义成引用块数之和**——
> 那个数在 zoned 上会**系统性地低于**删掉该快照真正能拿回的空间，
> 而 [pitfalls.md](pitfalls.md) 第 12 条骂的正是这类「`df` 输出对用户没有意义」。
>
> **本缺口尚未有对应条目**，补它需要先定义「不可回收空间」这个量。

```

**出处 `.claude/kb/invariants.md:200-249`（整段抄，未转述）**

```markdown
## I-6 加密不变量 —— 类

**依赖 D9（加密），17 个分项全部已定。**
**这些检查在加密未开启时应当整类跳过并显式报告为「不适用」，不许静默通过。**

<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-6.1 | nonce 不重用 | 加密开启时，全盘任意两个**已写出**的块（不论当前是否仍被引用），其 (密钥代号, nonce) 二元组不相同 | 未实现 |
| I-6.2 | 明文头字段白名单 | 加密开启时，任一单元的明文头字段集合 ⊆ 它那一类的白名单，**两张，按 D18（块里携带什么信息）已定项 11 登记表的码分**（码 1 数据单元类；码 2 / 3 元数据类；D18（块里携带什么信息）已定项 7 收口）——共同前缀两类都许：{magic / 格式版本 / flags（含单元类标签位）/ 头校验和 / **nonce 代号** / MAC}；元数据类**仅此而已**（五元组 / key 区间等身份字段一律进密文）；数据单元类另许 {fsid / 载荷 CRC}。⚠️ **2026-09-07 用户定案（D18（块里携带什么信息） 已定项 13，臂庚）按两处收窄**：① 码 1 的 {五元组 / 诞生代号 / 写序} 从「另许」里去掉，就地变密文；② **`声明长度` 从共同前缀里去掉**，三类一起变密文——它给出一个装得下的小文件的精确字节数，是收窄前泄漏面里最重的一项。字段的**位置与宽度一个都没动**，变的只是「盘上这几个字节是明文还是密文」 | 未实现 |
| I-6.3 | MAC 与密文匹配 | 加密开启时，任一被引用的块，其 MAC 与密文及关联数据匹配；**关联数据的每个字段由 checker 从权威树的 key 现算，不许从指针或明文侧读回**（需要密钥在场） | 未实现 |
| I-6.4 | 无未加密块 | 系统配置声明加密时，不存在算法类型字段标为「未加密」的数据块或元数据块 | 未实现 |
| I-6.5 | nonce 水位单调 | 记录在案的 nonce 水位 > 全盘出现过的最大 nonce（崩溃后不重用的可判定形式） | 未实现 |
| I-6.6 | MAC 长度一致 | 任一指针记录的 MAC 长度与系统配置声明的一致 | 未实现 |
| I-6.7 | 副本共用 nonce | 同一份数据的多个副本，其指针记录的 nonce 相同（nonce 未绑物理位置的可判定形式） | 未实现 |
| I-6.8 | 密文校验和匹配 | 加密开启时，任一被引用的块，其指针记录的密文校验和与盘上密文匹配（**不需要密钥**） | 未实现 |
| I-6.9 | 密文过而 MAC 败判身份错配 | 加密开启且密钥在场时，任一被引用的块，若**密文校验和通过而 MAC 失配**，判红并归类为「身份错配」——**不许归类为介质损坏，不许触发副本修复或投毒** | 未实现 |
| I-6.10 | 已停用·明文映射层不存在了 | **此编号不再使用**——2026-09-07 D9（加密） 已定项 5 改取「只做 scrub」，不设明文映射层、不设明文反向索引 ⇒ 这一格**无对象**。曾要求：加密开启且密钥在场时，checker 从权威侧重建出的明文映射层与反向索引与盘上明文副本逐条相同、差异判红不许就地修复 | 不适用 |

> ⚠️ **I-6.1（nonce 不重用） 只在「不论是否被引用」这个范围下才有判别力。** nonce 重用最危险的一批数据
> 恰恰是崩溃遗留的孤儿块——已写盘、未进树、因而不被引用。
> 把检查范围写成「被引用的块」，等于在设计上排除掉最该查的那一批。
> 因此 checker 必须两阶段：① 遍历树可达块；② **额外全盘扫描已分配物理区间**，
> 靠块的明文头识别「这是一个已写出的 singlefs 块」并取出其 nonce 纳入去重集合。
>
> ⚠️ **阶段 ② 对 D9（加密） 已定项 4（nonce 载体）提出一条硬约束：nonce 的代号部分必须放在明文头，
> 无密钥可读，不能整体埋进密文。** 埋进密文的话 checker 没有密钥就连「这块用过哪个 nonce」
> 都判断不出来，I-6.1（nonce 不重用） 的阶段 ② 与 [experiments.md](experiments.md) E5（nonce 唯一性 × 崩溃点重放） 的判据③一起失效。
> I-6.2（明文头字段白名单） 白名单里那一项因此写作「nonce 代号」而不是含糊的「序号」。
>
> ⚠️ **I-6.1（nonce 不重用） 是抗 bug 检查，不是抗攻击检查。** 阶段 ② 靠明文头 magic 枚举已写出的块，
> 而一个能写裸设备的攻击者抹掉 magic 就能让块从扫描里消失。
> 对抗场景下它给不出任何保证——[decisions.md](decisions.md) D9（加密） 已定项 6 记了适用范围。
>
> I-6.2（明文头字段白名单） 与 D18（块里携带什么信息） 已定项 3 的打架已按
> 「两张白名单、按单元类分」收口（2026-09-02，三方论证判决）；扫描器解密前先读
> 共同前缀里的类标签位分出类，再套对应那张表。**头校验和罩仍在明文的部分，MAC 罩密文**（覆盖范围见 I-2.4（头校验和覆盖范围）；加密是系统配置级全有全无），
> 加密元数据的头完整性由两者分层。仍挂在
> [checks-owed.md](checks-owed.md) C68（不变量白名单与已定条款打架） 的那一半是
> nonce 12 字节的归属三处打架与门禁比对工装。
>
> ⚠️ **I-6.2（明文头字段白名单） 的白名单是「允许出现在明文头」的清单，不是「可以拿来判定」的清单。**
> 里面每一项都只是提示：**必须在密文侧有权威副本，读者取密文侧那一份。**
> 从明文侧取期望值等于让攻击者控制判定的输入。依据见 [decisions.md](decisions.md) D9（加密） 已定项 6 的「期望值来源」一节。
>
> ⚠️ I-6.1（nonce 不重用） 与 I-6.5（nonce 水位单调） 是**唯一能把「nonce 重用」这类灾难变成可检出事实的检查**。
> nonce 重用在 ChaCha20 这类流密码下直接泄漏明文，而它不会表现为任何读写错误——
> 没有这两条检查，它永远不会被发现。验证手段见 [experiments.md](experiments.md) E5（nonce 唯一性 × 崩溃点重放）。

```

**出处 `.claude/kb/invariants.md:250-264`（整段抄，未转述）**

```markdown
## 待补

以下类别在对应决策定案前无法写出：

- 反向索引一致性（D1（数据可移动性 / 反向索引） 定案后：正向引用与反向索引必须双向可达且一致）
- 条带 / 多副本布局（D2（RAID 条带策略） 细化后）
- 快照可见性（D6（快照实现模型） 定案后）
- 索引结构自身的结构不变量，如树的有序性、平衡性（D8（核心索引结构） 定案后）
- 密钥层级与换钥（主密钥槽 80 与 KDF 标识 4 的宽度与落点 2026-09-13 由 D22（单元原子性怎么合成） 已定项 9 定死；还剩主密钥槽的内部构成与换钥不重写数据）

---


---

```

**出处 `.claude/kb/invariants.md:265-294`（整段抄，未转述）**

```markdown
## I-9 inode 树结构 —— 类

**依赖 D8（核心索引结构） 已定项 6，形态已定（码 2 内部节点 + 码 3 叶容器、记录定长 140、inode 号单调不复用），checker 未实现。**
与 I-1（块自描述） 类的分工：I-1.6（类标签登记且三处相等） 管单元自己的类标签合不合法，I-1.7（打包容器合法与判定顺序） 管一个容器自己合不合法并且任一步失败即容器级中止；
I-9（inode 树结构） 管跨单元的结构约束（谁该指向什么、号与 key 的关系）与对象级（一条记录）的判定，对象级判红只让那条记录 EIO，不进 I-1.7（打包容器合法与判定顺序） 的中止链。
**共同前置**：checker 要知道一棵树是 inode 树才能对它判这一类，而树表条目里有没有「树的种类」一段无落点（D8（核心索引结构） 已定项 8）；第一版镜像上可由打包记录类型 2 容器头里的出生树反推哪棵树是 inode 树，所以这一类今天不是恒不评估。
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-9.1 | inode 树根恒码 2 | 树表指向的 inode 树根，共同前缀偏移 6 恒为 2（索引节点），哪怕树只有一片叶或是空的（零条目、key 区间 [0, 0]）——D19（块指针的结构与宽度预算） 的指针不带被指对象的类与大小，根的类若会变，读者按 16 KiB 读 32 KiB 是静默误读 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.2 | 条目身份与子头相符 | inode 树内部节点的每条条目：类型段 ∈ {0, 2}；类型段 0 ⇒ 子单元偏移 6 = 2 且身份引用的出生树、容器号、出生代三段为 0；类型段 2 ⇒ 子单元偏移 6 = 3，子头的 (出生树, 打包记录类型, 容器号, 容器出生代) 与条目的身份引用四段逐字相等，子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 2（出生树那两处 2026-09-12 随 C297（inode 内部条目里出生树两份没有定处理） 用户定案加）；其余判该条目损坏。跨头共享时出生树的读法随 C110（跨头共享与加密后元数据类的块头一致性读法） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.3 | 内部节点消息数恒零 | inode 树内部节点的消息计数恒 0（ε = 0，D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴）；计数字段住节点内部布局（D15（格式冻结政策） 第 4 层，未冻结），那一层定了这条才可判 | 未实现，前置：树的种类 + 节点内部布局 |
| I-9.4 | 容器号不超最小 key | 打包记录类型 2 的容器内每条记录的 inode 号 ≥ 容器号；沿叶序容器号严格递增，且左容器的最大 key < 右容器号（这条逼出合并只许左吸收右 / 左从右借） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.5 | 删除队列里的号仍在树里 | `deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0（记录在回收完成前留在 inode 树里，`deleted_inodes` 只存号） | 未实现，前置：树的种类 + `deleted_inodes` 的形态（C118（`deleted_inodes` 树的形态无落点）） |
| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半没有对象，这一版没有删除（C118（`deleted_inodes` 树的形态无落点）） |
| I-9.7 | 记录三段恒零 | 打包记录类型 2 的记录偏移 108 / 112 / 120 三段（填充 4 / flags 8 / 预留 20）恒 0；非 0 判该记录 EIO，容器其余记录照读（前向兼容政策，D15（格式冻结政策） 判定表；不是损坏检出，载荷校验和照样过） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.8 | 记录 inode 等于 key | 打包记录类型 2 的记录偏移 0 的 inode 号 == 它在 inode 树里的 key；不等判红、该记录 EIO，不许静默修复 | 未实现，前置：树的种类 |
| I-9.9 | locality 三值对照 | 记录的 `locality_id` == 该对象每条 extent key 的首段；extent 树未经重建时输出红 / 绿；系统配置 `map_provenance` 标 rebuilt 时两侧同源，输出「不可判定」，不许输出「通过」；按侧判（2026-09-16）：记录那一侧是从 key 编出来的、或 key 那一侧是从记录编出来的，同样输出「不可判定」；分批重新聚簇的中间态怎么判随 C346（后台重新聚簇没有定义） 定 | 未实现 |
| I-9.10 | 对象出生代三处一致 | 记录偏移 8 的对象出生代 == 该对象每个数据单元头五元组里的对象出生代 == 该对象的墓碑记录里的对象出生代（若有）；重建后两侧同源，输出「不可判定」，与 I-9.9（locality 三值对照） 同形 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.11 | 可写克隆头有水位行 | 每个可写克隆头的树 ID 在记账里都有「inode 号水位」行（树表可枚举、记账可查）；「初值 ≥ origin 那一刻的水位」是建克隆那一刻的操作断言，转模型对拍——它的右操作数在 K 次发布后被 D5（快照 / 空间记账机制） 已定项 2 的点删丢掉，对着镜像读不到 | 未实现，前置：树的种类 |
| I-9.12 | 分隔 key 落在孩子区间之外 | inode 树内部节点的条目按分隔 key 严格递增；第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界。判的字段是内部节点条目头 8 字节那个分隔 key（D8（核心索引结构） 已定项 6：条目 120 字节 = 分隔 key 8 + 身份引用 26 + 子指针 86，「分隔 key = 该孩子创建时的最小 key，条目按分隔 key 单调递增」）；孩子的 key 区间对码 3 容器由它装的记录现算，容器头不带 key 区间 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.13 | 类型 2 容器不空 | 任一打包记录类型 2 的容器记录数 ≥ 1（空叶在重建时没有 key 范围；唯一的叶被删空时不落盘、根变成零条目） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.14 | 树表条目的诞生 txg 跨根不变 | 同一棵树（按树 ID）的树表条目，在**同一条时间线上**的任意两条有效根各自的树表里，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段）；C374（释放代与树表诞生 txg 只有验收断言盯着））。「同一条时间线上的有效根」就是回退候选集（D23（journal 的角色与格式） 已定项 14 + D16（发布语义） 已定项 1）：最新那条根本身；以及根环里 (i, T) 满足两条的根——按**最新那条根指着的实例表**，没有实例 i 的行、或有行 (i, Ti, Wi) 且 T ≤ Ti；并且 T ≥ 最新根带的回退下界 F。有行 (i, Ti, Wi) 且 T > Ti 的根在被崩溃恢复切掉的旧线上，**不与现行线上的根比**（C511（回退到无文件那一版之后诞生代怎么接） 2026-09-23 用户定案收窄射程）。候选集不足两条根，或同一条时间线上没有一棵树的条目出现在两个不同的树表单元里时，报「不适用」，不报成立——几条根指着同一个树表单元、拿同一份字节比自己是恒真判定。判别力：① 把树表条目的诞生 txg 改成跟着这次发布的 txg 走，必须红；② 管理员回退前后同一条线上记得不同，必须红（回退是向前发布，前后是同一条线）；③ 只有被切掉的那条线记得不同，不许红。推翻条件：找到一条历史，最新根的实例表把某条根判成有效，而它不是最新根的祖先（不在同一条线上）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判。2026-09-23 收窄之后同文件两条：判别力 ② 的坏镜像 `birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant`（挂着时回退到 A 之后，回退那次发布 D 的树表把 extent 树（树 11）的诞生 txg 记成 D 的 txg 9、A 那一版记 3：A 与 D 在同一条线上，只有 I-9.14（树表条目的诞生 txg 跨根不变） 红），判别力 ③ 的正向用例 `birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_is_not_compared_and_every_invariant_holds`（被切掉的线由崩溃恢复造：C 被抛弃之后只改 C 那片树表里的诞生 txg，每一条都成立）；判定在 `walk::judge_tree_table_birth_txg`） |
| I-9.15 | inode 记录的 blocks 等于 ⌈size ÷ 512⌉ | 打包记录类型 2 的每条记录，偏移 48 的 blocks == ⌈偏移 40 的 size ÷ 512⌉（D8（核心索引结构） 已定项 6：blocks 是逻辑长度的 512 字节块数，不表示分到的空间；C480（inode 记录的 blocks 怎么算全仓没有条款） 用户 2026-09-23 定）；不等判红。读者要不要也判、判了报什么没有条款 | 已实现（2026-09-24，池级 checker `walk::check_pool_image` 走 inode 叶容器时逐条判，与 I-9.7（记录三段恒零） 同一处；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：blocks 写成 64 / 5 / 7 各只红它；层 0 每个走得到 inode 叶的崩溃状态都判；变异：判定恒真、按向下取整比、写路径写回 64；里程碑「覆盖写、释放、回退与复用」收口表第 46 行） |
| I-9.16 | 树表条目按树 ID 严格升序且合发号次序 | 同一张树表（D8（核心索引结构） 已定项 8 的字段表）里，① 盘上相邻两条的树 ID 严格升序；② 树表里有的种类按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist；中央映射树不进树表）排，相邻两棵的树 ID 严格升序。两道任一道不成立即违例。依据 D8（核心索引结构） 已定项 8「排序契约」与它的 ②「八棵树的号从水位起连号发，次序照格式常量 11..18 那一组」。判别力落在 ②：① 与 checker 在树表上判的 I-1.1（块头自述逻辑地址） 那组 key 严格递增是同一个谓词，只违反 ① 的镜像两条一起红 | 已实现（2026-09-27，core `crates/singlefs-core/src/recovery.rs` 常量 `TREE_TABLE_ENTRIES_ORDERING_CONTRACT`，两道合成一个入口，`rebuild_version` 与 `walk_to_file` 在读任何一棵树之前拒；池级 checker `crates/singlefs-checker/src/walk.rs` 另写一份、违例写明哪一版根与哪两条，列进 `image.rs` 的 `IMPLEMENTED_INVARIANTS`；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`「互换两条的种类」那份只红 I-9.16（树表条目按树 ID 严格升序且合发号次序）；只违反 ① 的钉在 `tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`；实现员报告 `research/prompts/m2-rev-tree-table-ordering-implementer-report.md`） |

**不进 I-9（inode 树结构） 的**：分裂 / 合并的身份传递（左半身份不变、右半号 = 触发分裂的新记录号、出生树 = 执行分裂的树）与合并时机是操作断言，对着一个镜像判不了，走单测 / 模型对拍与崩溃点重放（C116（叶容器的分裂 / 合并纪律没有会失败的检查））；
nlink == dirent 条数的门（dirent 树存在）今天没有输入，写不成可判定形式，随 dirent 树上线时一起立。

```

**出处 `.claude/kb/decisions/13-验证路线.md:6-24`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **crash refinement 先验哪一块** | 先验「一次发布 + 一次崩溃 + 一次恢复」这个封闭单元，第一个里程碑是任一 `jsn` 连续前缀不切开发布；原候选 I-6.5（nonce 水位单调） 撤下 **状态：已定。** |
| 2 | **事务层承不承诺串行提交** | 先串行，越过交点再开并行；将来并行提交时记录按事务号顺序追加 **状态：已定。** |
| 3 | **自写参照实现（RefFS 形态）做到什么程度** | 做到极致——自己处理持久化与崩溃语义 **状态：已定。** |
| 4 | **崩溃点重放的崩溃状态集合怎么定义** | 写流按设备切段，段里每块有写的盘都被自己的屏障或 FUA 放行才关段；段内每次整写各取几态、任意组合，原地覆写多一态「新旧都读不出」；没持久放旧字节且真生成镜像；双写按设备各算 **状态：已定。** |
| 5 | **checker 与被测实现允许共享什么** | 只共享一份从 kb 生成的常量模块，其余各写一份；判定宽度由 checker 自己探测再与系统配置比对，探不到报「声明值，未探测」 **状态：已定。** |
| 6 | **三个互相独立的 oracle** | 内存模型对拍、独立解析器 + checker、独立规约执行器三个 oracle 各判一样；独立性要靠「三 oracle 合谋测试」验证，不许自称 **状态：已定。** |
| 7 | **O2（独立解析器 + checker） 判什么** | O2（独立解析器 + checker） 的定义域是单个镜像，只判集合成员；需要第二个输入的检查归记录核对器（入参含枚举器给的持久集合），故意不给编号 **状态：已定。** |
| 8 | **自研的两样** | Merkle 根链差分（相邻两代根递归比较，指针没变处不下钻）与代际增量 scrub（只遍历 birth generation 高于水位的块） **状态：已定。** |
| 9 | **崩溃点重放怎么分层** | 三层都精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore 慢慢开；夜间档是异步准入判据，先保失败可复现 **状态：已定。** |
| 10 | **真正必须精确覆盖的 N 是什么** | 提交协议结构的等价类数，不是布局套数；落地是一个封闭的提交步骤枚举加穷尽 match **状态：已定。** |
| 11 | **ignore 的节点什么时候开** | 否决按欠账轮转的账本；留下三条改管开的时机：确定性的覆盖记录、`T1`（层 1 挂钟预算） 是输入、判别力与覆盖分开检查 **状态：已定。** |
| 12 | **冲突 1：checker 即规范与校验路径不许共享工具** | checker 做遍历与扫描两个方向，对同一条不变量各自独立判，共享的只有格式解析 **状态：已定。** |
| 13 | **冲突 2：refinement 的 spec 与对拍的 model** | spec 与 model 在表述层面就写得不同；同源性要观测，≥ 5 次不一致而判 spec 错 0 次就合并、预算移给穷举 **状态：已定。** |
| 14 | **与其他决策的连锁** | 哈希与 AEAD 在形式验证里只能当 uninterpreted function；日志结构三层绑架符号执行；checker 不必解析消息语义 **状态：已定。** |
| 15 | **验证代码分两档** | harness 档（`singlefs-harness`）是单元与集成测试、随时跑；checker 档（`singlefs-checker-tier`）默认只在提交时跑快档，全量由用户要求或夜间 **状态：已定。** |

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:15-44`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **journal 的角色本身** | 取甲：每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录，祖先不延后；轴一与轴二不正交 **状态：已定。** |
| 2 | **journal 是定长环还是链式** | 取定长环；决定性理由是链式把死锁 3 的判据从静态换成动态 **状态：已定。** |
| 3 | **tail 住哪、谁是权威、什么时候持久** | 住系统配置槽，≥2 槽轮换，随每次发布写一次；恢复不许先信 tail，重放必须幂等，恢复救不了几何 **状态：已定。** |
| 4 | **记录头能不能约束在单个原子单元内** | 能；头 311 字节 = 十个字段 78 + 四笔增量 21（事务边界 9、本次发布内序号 4、反向链 4、载荷校验和 4）+ 新根段 188 + fsid 8 + MAC 16 **状态：已定。** |
| 5 | **O2（独立解析器 + checker） 到底是什么** | 一元谓词；否掉逻辑意图日志的第二条理由改写成第一条的封堵；反向判据的主语是记录核对器 **状态：已定。** |
| 6 | **目标负载优先级** | 移出 journal 这条线，见 D25（目标负载优先级）；它不阻塞 journal 的任何一项 **状态：已定。** |
| 7 | **一个事务恰不恰好产生一条 journal 记录** | 可以跨多条，记录头带事务号与提交标记；重放丢掉提交标记没出现的尾巴 **状态：已定。** |
| 8 | **记录里分辨时间线的字段** | 记录头带一条反向链（前一条记录的 hash），宽 32 位 **状态：已定。** |
| 9 | **序号与实例代号的位宽划分** | jsn = 实例代号 32 位 + 计数器 48 位，两个字段共 10 字节；代号不许回滚 **状态：已定。** |
| 10 | **`previous_hash` 覆盖哪些字段** | 覆盖前一条记录的整个记录头，其中含 jsn，因而含实例代号 **状态：已定。** |
| 11 | **反向链的 hash 算法** | 取完整 32 位 CRC32C，与密文校验和同一个函数 **状态：已定。** |
| 12 | **记录尺寸** | 4 KiB 定长，写进系统配置 **状态：已定。** |
| 13 | **载荷的完整性靠什么** | `header_csum` 罩整条记录 [0, 4096)；`payload_csum` 保留、罩点名项数组；任一不过即该记录无效 **状态：已定。** |
| 14 | **重放的下界、管理员回退、失败处置与实例切换** | 只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录；前缀口径五条；切换按一次恢复做，管理员回退是挂着时的一次向前发布 **状态：已定。** |
| 15 | **崩在记录持久之后、根槽持久之前，恢复施加什么** | 由记录重建那次发布的根：记录头带 188 字节新根段，施加一条记录 = 换掉所选根的四个字段 **状态：已定。** |
| 16 | **mkfs 与第一次可写挂载的实例代号** | mkfs 写 0（不是有效实例），第一次可写挂载取 1、写进每一份系统配置并过一道完成的屏障之后才动单元 **状态：已定。** |
| 17 | **点名项 56 字节的字段表** | 一项一个单元：位置条目 14 × 2 + 类型 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 + flags 1 **状态：已定。** |
| 18 | **在飞记录数上限与 tail 的编码** | 上限 = 环槽数 ÷ F，含义是一次恢复重放的前缀最多 N 条；坏校验和一律断链即止；tail 存计数器 **状态：已定。** |
| 19 | **空发布记录、`previous_hash`、环长** | 事务号 0 留给空发布；链按本实例逻辑前一条算；环长是 mkfs 参数，默认 768 MiB、≤ 容量 ÷ 4 **状态：已定。** |
| 20 | **记录承载什么、分配与记录谁先** | 记录承载指针层的目标态，不是逻辑意图、也不是全量物理重做；分配先于 journal **状态：已定。** |
| 21 | **记录头的两个计数字段** | jsn 每条记录全局单调、前缀判定靠它；checkpoint_txg 每个 checkpoint 一个；两者不可合并 **状态：已定。** |
| 22 | **前缀判定** | `jsn == expected`，不等即止；另有一个显式的在飞记录数上限 **状态：已定。** |
| 23 | **三个死锁各自怎么破** | 死锁 1 靠读法甲，死锁 2 靠保留池，死锁 3 靠几何不变量：最坏占用 ≤ 环大小 / F，F ≥ 2 **状态：已定。** |
| 24 | **死锁 2 与假性 ENOSPC 是两种故障** | 卡死靠保留池，reserve ≥ ckpt_cost 就够；假性 ENOSPC 只有准入治得了，保留池反而加重它 **状态：已定。** |
| 25 | **D16（发布语义）「不绕过 journal」那句取哪个读法** | 取读法甲：语义效果必须经 journal，发根不是一条 journal 追加，是覆写根环的一个槽 **状态：已定。** |
| 26 | **记录头的两条解析纪律** | 校验和覆盖范围是显式盘上常量，不由 sizeof 推；自述长度先做边界检查再用于校验和 **状态：已定。** |

```

**出处 `.claude/kb/decisions/16-发布语义.md:6-26`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **defer 窗口取多大** | 按盘回退下界的收严形态、按状态数读：候选集保留最近 4 个可退到的不同状态（去重、以根环装得下为界），可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)；F 另记进系统配置 **状态：已定。** |
| 2 | **checkpoint 的触发判据** | 时间与脏数据量两者取先到：`触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)` **状态：已定。** |
| 3 | **journal 在 D15（格式冻结政策） 的冻结分层里记第几层** | 不进四层，立成四层之外的第三个独立冻结组件；新记录格式走记录类型 + 系统配置 log-incompat 位 **状态：已定。** |
| 4 | **记录核对器那第二份 replay 实现** | 一次发布整体施加或整体不施加；第二份实现停在句法层（前缀判定 + 择根 + 续做），在合谋测试里是第四个判决器 **状态：已定。** |
| 5 | **`T_time` 与 `T_dirty` 取多少，丢失窗口的口径** | `T_time` = 5 s、越界只警告；`T_dirty` = 2 GiB、夹取成 min(字段, 环长 ÷ F)；崩溃最多丢 T_time + N_switch × (T_retry + T_redo) **状态：已定。** |
| 6 | **发布计数器** | 每次发布 checkpoint_txg 加一，fsync 触发的也是发布；根槽轮转键、记账的代、重放水位是同一个计数 **状态：已定。** |
| 7 | **发布的持久顺序** | 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障；重放施加前必须逐项验证点名单元 **状态：已定。** |
| 8 | **新实例要不要先让本实例的根落到两块盘上** | 取戊：根覆盖两块盘之前不让 fsync 返回，连推空发布至多 R = 3 次 **状态：已定。** |
| 9 | **一次空发布写不写单元** | 记账树存在时就写：空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2 重写记账行与连带的四样 **状态：已定。** |
| 10 | **发布粒度** | 攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次；发布频率只由 D16（发布语义） 定 **状态：已定。** |
| 11 | **能不能绕过 journal 直接发根** | 不能：所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏，fsync 只是提前触发一次发布 **状态：已定。** |
| 12 | **中间状态的合法性语义** | D8（核心索引结构）「中间状态必须每一个都是合法的文件系统状态」改写成「根 + journal 任意前缀，replay 之后合法」 **状态：已定。** |
| 13 | **这份 journal replay 归谁** | 归记录核对器，不归 O2（独立解析器 + checker）；它是第二次实现 replay，是 D16（发布语义） 最贵的一笔 **状态：已定。** |
| 14 | **快照发布点的位置** | 与 checkpoint 边界对齐，钉在这个 checkpoint 全部数据变更之后；同窗两个快照用 `(txg, 快照 id)` 打破并列 **状态：已定。** |
| 15 | **checkpoint C 中产生的释放什么时候进可分配集合** | C 被发布之前不得进入；两条理由，第二条按 journal 位置说，比 checkpoint 边界更严 **状态：已定。** |
| 16 | **checkpoint 与消息缓冲的摊销窗口分界** | 两者买同一样东西，差别只在窗口住内存还是住盘；E7（离线索引 harness） 的对照组钉成「有 checkpoint、无消息缓冲」 **状态：已定。** |
| 17 | **fsync 报错之后调用方能假设什么** | 报错之后这次发布可能已成也可能未成，不许假设可复用旧内存态重试；成没成由调用方自己读现行根判定 **状态：已定。** |

```
