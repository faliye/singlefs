# 第 17 批代码轮第一轮正文（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 K4 M1 -->

## 一、题面

第 17 批（`records/2026-10-01-里程碑三形式化接入.md#17`）合入了 17-1 与 17-4，改动全文 `research/prompts/_m3-formal-code-b17-r1-diff.md`：

- 17-1：池级 checker 与 GPU 判器新立 I-9.27（树表条目的头 ID 取值）与 I-5.5（第一次建树那一版的用户数据落点），`IMPLEMENTED_INVARIANTS` 下标 66、67，GPU 覆盖表编译期断言改成「每个下标恰好在两张表之一里、各自升序」；高水位历史用例、头 ID 写死 65537 的坏镜像（M1）、`checker_known_bad_images.rs` 两张坏镜像、GPU 对拍坏流、`writable_mount_of_a_formatted_pool.rs` 三张不适用表各加两条；`crates/singlefs-core/src/transaction.rs` 一句注释。
- 17-4：`tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 的坏镜像互换种类时头 ID 跟着换。

条款：`.claude/kb/invariants.md` 的 I-9.27、I-5.5 两行；`.claude/kb/decisions/08-核心索引结构.md`「#### 已定项 8：树表条目的字段表」头 ID 那一句与句末关系判据；`.claude/kb/decisions/03-空间分配.md`「#### 已定项 8」射程（用户 2026-10-04 定：管「从 mkfs 水位第一次建树的那一版」）。条款判决：`research/prompts/c673-clauses-r1-main-verification.md`。

## 二、实现今天的样子（主 agent 的观测，2026-10-04）

- 合入验证：harness 档四个包全部测试二进制，17-4 之后全绿（17-4 之前红的那一条是 I-9.27 对互换种类那张坏镜像判红，判得对，改的是坏镜像造法）；fmt、clippy（`--all-targets --all-features`）退 0；checker 档、GPU、WGSL 的 naga 校验都没跑。
- 17-1 交出的补丁是实现员把开工快照上编过、证过红的那一版用 `git merge-file` 合到当时主仓上的（主仓那时 I-3.5 进了下标 65、`walk.rs` 树表走读重构过），合并之后它没再编；主 agent 合入后编过、harness 全绿。8 行变异里证了第 1、2、5 行（M1、A2c 那一形、认法 ② 不看诞生 txg），其余归提交时。
- I-5.5 几格判不了时实现报不适用：树表读不出或条目宽不到 200 字节、种类 1 条目不止一条、extent 树读不全、有数据单元而某块盘上一个都没有；多块盘落点不同时逐盘判、任一盘不等就红。I-9.27 条目宽不到 124 字节时整张表不判（照 I-9.16 先例）。
- GPU 的 I-5.5 放在扫描内核 `crash_judge_checker_scan.wgsl`（只读的 extent 树读法在那里）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　I-9.27 的 CPU 与 GPU 判法对不对
与 invariants 那一行逐句对：种类 2 自己、种类 1 / 9 / 10 对同表种类 2、其余写 0、树表 0 条不判、多条种类 2 时放宽；挂的位置（按表、第一次走到这张树表的那一版）；GPU 与 CPU 是不是同判（种类表、条目宽、偏移 116）；高水位用例与 M1 坏镜像证不证得到它。

### K2　I-5.5 的认法与判据会不会误红、漏判
认法 ①（树表非空且种类 1 的树 ID == 65536）②（每条诞生 txg 等于这一版的 `checkpoint_txg`）与判据（每盘最小数据单元槽号 == U + 4）：合法历史上（重挂之后的第一次建树、非默认环、零长文件、崩溃落在第一个文件版本中途再建一次、两遍读环不一致时 t1 被隔离）会不会认出而 t1 ≠ U + 4；坏实现（E171 第 15 行那条、t1 挪一槽）漏不漏；判不了就报不适用的那几格是不是过宽（把该判的也放过了）。

### K3　GPU 覆盖表断言的改法
「每个下标恰好在两张表之一里、各自升序」能不能让漏判（一条既不在覆盖表也不在还没判表）或重复登记编不过；与根环那条线将来删「还没判」表、回到全覆盖的断言接不接得上。

### K4　变异行与测试的判别力
8 行变异各自改的是不是它点名要证的那一处；第 3、4、6、7、8 行没证的那几行推不推得出红；17-4 改过造法之后，点名那条用例的两行旧变异（`crates/mutations.tsv` 第 1605、1809 行）仍红（实现员 prove-red 过）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3、K4 | 逐句核代码做的是不是条款说的，每格给判定与引文 |
| 云端攻方（Opus） | K2、K4，兼 K1 | 攻哪一格会错：造误红的合法镜像、漏判的坏实现、过宽的不适用；量得了的在副本上写小模型或 harness 用例量，量不了的写推的 |

## 五、跑前写死的判据

- 两腿同判且主 agent 现查坐实 ⇒ 按那一判；打中而坐实 ⇒ 改法并进下一批实现员；分歧 ⇒ 开第二轮。
- checker 档、GPU 没跑的格，腿的结论只能是「推的」，判决里照实标。

交付：腿的报告写 `research/prompts/m3-formal-code-b17-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b17-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。

### 小节清单：`.claude/kb/invariants.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 不变量清单 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （不变量清单 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 状态 | 抄 | 默认全抄（脚本生成） |
| ## I-1 块自描述 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-7 根环 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-8 journal —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-2 校验和 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-3 空间记账 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-4 事务与崩溃一致性 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-5 分配器 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-6 加密不变量 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## 待补 | 抄 | 默认全抄（脚本生成） |
| ## I-9 inode 树结构 —— 类 | 抄 | 默认全抄（脚本生成） |

### 小节清单：`.claude/kb/decisions/08-核心索引结构.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D8 核心索引结构 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D8 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：记账取幂等完整值，接受 key 数量上升 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：节点大小 16 KiB，钉成常量 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：key 布局 —— 改名与索引分开处理 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：节点大小 —— 上限由恢复时间定，靠实验取值 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：whiteout 删除 —— 接受，但批量删除必须走 intent log | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：write buffer 条目的 `seq` —— 4 字节，住 value | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：树表条目的字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：树的种类码的登记表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：第一版记账直落叶，`seq` 的取值规则 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：码 2 的载荷布局与 key 比较序 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：write buffer 前端与节点消息缓冲区的衔接 —— 一棵树只准一级 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：索引树更新做状态折叠 —— 同一次发布里同一个 key 的中间状态不落叶 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：一套 btree 实现 + 多个独立 keyspace + 两个前端，派生树可以异构 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：write buffer 条目的三条硬要求 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/03-空间分配.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D3 空间分配 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D3 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：分配记录逐落点答得出「属不属于现行版本」 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：墓碑与 livelist 的 FREE 条目由准入控制算进最坏情况 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：分配记录住一棵独立 keyspace 的 btree | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：准入不等式的权威形式 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：提交内生块从聚簇段分配 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：准入的在飞合成 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：分配记录条目的盘上编码，与分配准入合取的登记表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：用户数据块不进聚簇段，落点是已选设备内起点最小的空槽 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：不许有假性 ENOSPC，删掉的空间在有界步数内可用 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：聚簇段的定义、用户数据落点的四格、提交内生块的 bump 次序 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：分配记录 20 字节里跨度段进 value | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：分配准入合取的唯一登记位，与两条准入式的关系 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：不分 data / metadata 两类空间 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：不分类的连带承诺——后台整理常驻、自动、用户无需知道 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：空间预留取准入控制形态 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：加密开启时无密钥的一侧只读只检测 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：释放空间这个操作本身不需要申请空间 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/experiments/171-三oracle合谋测试.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E171 三oracle合谋测试 —— 已跑（s4 整表 117 行、F4 扩表 164 行与编不过 6 行的补位都已跑，读数到齐；结论三轮判决已写回） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E171 标题之下、第一个下级标题之前的正文：第 2-15 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 装置、研究一侧与变异表 | 抄 | 默认全抄（脚本生成） |
| ### s4 整表的结果（整行抄自产物） | 抄 | 默认全抄（脚本生成） |
| ### 三条全放行的 7 行 | 抄 | 默认全抄（脚本生成） |
| ### Q3 只靠一条的 11 行 | 抄 | 默认全抄（脚本生成） |
| ### s3 整表作废 | 抄 | 默认全抄（脚本生成） |
| ### F4 扩表 | 抄 | 默认全抄（脚本生成） |
| ### F4 扩表的结果（整行抄自产物） | 抄 | 默认全抄（脚本生成） |
| ### F4 新出的全放行行 | 抄 | 默认全抄（脚本生成） |
| ### 三轮判决之后的三分 | 抄 | 默认全抄（脚本生成） |
| ### F4 新出的只靠一条的 12 行 | 抄 | 默认全抄（脚本生成） |
| ### 没触达却红的两行（第 155、161 行）与未触达行里交回值不同的七行 | 抄 | 默认全抄（脚本生成） |
| ### 编不过的 6 行补位（已跑） | 抄 | 默认全抄（脚本生成） |
| ### Q2 里 O2 放行而 O3 判不在的坏镜像（整行抄自产物） | 抄 | 默认全抄（脚本生成） |
| ### 第一段的基线与对照（整行抄自 s1 产物） | 抄 | 默认全抄（脚本生成） |
| ### 对照与自检 | 抄 | 默认全抄（脚本生成） |
| ### 单测与变异 | 抄 | 默认全抄（脚本生成） |
| ### 路径与结论登记 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-10-03：建页；O3 接上；s3 整表作废；s4 整表；F4 扩表跑完；编不过的行补位跑完；结论三轮判决写回 | 不抄 | 历史节的子节 |

**出处 `.claude/kb/invariants.md:1-8`（整段抄，未转述）**

```markdown
# 不变量清单

**checker 是 invariants.md 的可执行形式。** invariants.md 每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../rules/format-evolution.md)「硬约束」）。

每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。
写不成可判定形式的，说明还没想清楚，不许进 invariants.md。

```

**出处 `.claude/kb/invariants.md:9-20`（整段抄，未转述）**

```markdown
## 状态

**新池新建文件阶段（2026-09-14 起）。** 磁盘格式已落成字节：mkfs、新池新建文件的写路径与冷启动恢复在 `crates/singlefs-core`；checker 在 `crates/singlefs-checker`，与实现只共享常量模块（D13（验证路线） 已定项 5）。
池级 checker（`walk::check_pool_image`）判 65 条（数它的命令：`grep -c '^| I-.*已实现' .claude/kb/invariants.md`，与 `crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 长度相等；第一版 23 条，2026-09-16 里程碑「覆盖写、释放、回退与复用」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），2026-09-18 加 I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 与 I-5.4（分配记录罩住的槽互不相交），2026-09-21 加 I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号），2026-09-22 加 I-1.10（码 2 条目宽等于字段表宽）、I-9.6（水位大于两处最大号）、I-9.12（分隔 key 落在孩子区间之外）、I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块），2026-09-23 加 I-8.8（前缀里的事务不被切开） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号），2026-09-24 加 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读） 与 I-8.9（一次发布的记录序号连续且只有末条带标志），2026-09-25 加过 I-7.10（已停用·回退见证表删了） 与 I-7.11（已停用·见证择根删了）、2026-09-26 随回退见证删掉退役，同日加 I-7.12（系统配置 F 不低于同盘根上的 F），2026-09-27 加 I-1.11（映射 key 与单元头相符），同日加 I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序），2026-09-28 加 I-7.14（系统配置本盘设备号等于盘在池里的身份），2026-10-02 加里程碑三二·1 那一批 16 条（I-1.12（指针出生树等于被指单元头）、I-3.12（实例表行 T 低于引用它的根的 txg）、I-3.13（记账行代段等于这一版的 txg）、I-4.1（重放后树可遍历）、I-4.9（非最新候选根引用的单元不晚于它）、I-7.5（根槽按判定宽度对齐）、I-7.15（各盘所选系统配置槽设备数相同）、I-7.16（系统配置槽世代号与槽号的关系）、I-7.17（根槽位置等于 txg 算出的位置）、I-8.1（环几何够大）、I-8.5（tail 槽有效集非空）、I-8.10（记录新根段与同代根相同）、I-8.11（点名项与被指单元头相同）、I-8.12（同实例记录的 txg 不减）、I-9.25（树表条目的 previous_snapshot_txg 取值）、I-9.26（inode 改动计数落在出生代与容器诞生代号之间））），状态列写「已实现」的就是这 65 条：每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`），层 0 的每个崩溃状态都跑它。
I-7.7（系统配置实例代号不低于根环） 2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 定案改成 ① ②，层 0 里 0 个状态判红；I-3.1（已分配统计对得上） 与 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 的 checker 读法 2026-09-14 由用户收尾弹窗定甲，写在各自那一行。
其余条目仍写「未实现」：它们要覆盖写、释放、回退与复用（释放、重用、残留）、journal 一侧的那一类检查、快照或加密，新池新建文件的镜像上判不出东西。

违例位是 128 位（4 个 u32 字，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 的 `POOL_CHECKER_VIOLATED_BIT_CAPACITY`），位 i 对 `IMPLEMENTED_INVARIANTS` 的第 i 条（`pool_checker_violated_bits` 读成 u128）；今天用了 65 位，`POOL_CHECKER_COVERED_INVARIANT_INDEXES` 列满 65 个下标，编译期断言它与 `IMPLEMENTED_INVARIANTS` 等长、按下标次序一条不缺（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行判定的位宽加宽与 GPU 内核补齐）。

<!-- invariant-count -->
现共 104 条在用（编号至 I-9.27（树表条目的头 ID 取值），另有六条退役，编号不回收）。这一句是当前条数的权威登记位；「## 历史版本」里那几句「现共 N 条在用」记的是当时的数，不跟着改。

```

**出处 `.claude/kb/invariants.md:21-47`（整段抄，未转述）**

```markdown
## I-1 块自描述 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-1.1 | 块头自述逻辑地址 | 任一单元，其头记录的**类内身份**与实际引用一致——数据单元的身份是五元组（锚点偏移即它的逻辑地址）；索引节点是树 ID + 层级（+ 该树已定携带的 key 区间，D18（块里携带什么信息）已定项 2 口径）；打包记录单元（码 3）是（出生树 ID, 打包记录类型, 容器号, 容器出生代）四元组，「父级」按打包记录类型分——类型 1 走容器索引的身份引用（C103（容器索引无落点）），类型 2 走 inode 树内部节点条目的身份引用（I-9.2（条目身份与子头相符）），**类型 4（实例表）走根记录的物理指针，不属于任何树、不进容器索引**。**自证单元（journal 记录 / 根槽 / 系统配置槽 / 类型 4 实例表单元）显式豁免**：无父引用无树身份，自证纪律在 I-7（根环）/ I-8（journal）两类与 D22（单元原子性怎么合成）已定项 21；多层码 2 树（记账树、中央映射树）的索引节点，身份里的层级与 key 区间按引用它的父条目核：层级 = 父 − 1、区间落在父条目给的那一段、内部节点区间是子树覆盖区间；按位置寻址的两棵树（分配记录树、extent 树），节点头里的层级与 key 区间等于它的位置规定的那一层、那一段；内部条目的 key 是某个孩子那一段的起点，孩子按位置严格递增；分配记录叶里每条记录的起点与末槽都落在叶罩的那一段里；extent 上段叶条目的 inode 号、下段叶记录的单元号落在叶罩的那一段里，上段叶条目的标签只能是 0、1、2（标签 0 载荷全零、标签 1 载荷末 2 字节为零）；根之下没有空节点（这一半 checker 由实二五补）。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；`walk::Walk::walk_code_two_subtree`；坏镜像用例 `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`；`crates/singlefs-checker/src/position_addressed.rs`） |
| I-1.2 | 块头写序已发布 | 任一单元，其头记录的写序 (i, n)（D18（块里携带什么信息）已定项 7 / 11 的字段）与诞生代号 b 按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的全局已发布谓词判成已发布：i < 挂载根实例且无该实例的行 ⇒ 已发布，有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub（所选根那个实例的 T_pub = 重放之后那个根的 txg，严格介于所选根的实例与新实例之间、又不是切换照旧事务写序主人的实例写 (i, 0, 0)，切换的所选根按 D23（journal 的角色与格式） 已定项 14 取被重发的在飞 checkpoint 所基于的根；W = **属于实例 i 的**、被这次重放施加的最大事务号，切换时是被照旧的、写序属于 i 的最大事务号；同一次恢复写出的那批行里只有所选根那个实例的 W 可能非 0，同一次切换写出的那批里只有被照旧事务的写序主人的 W 可能非 0，表上允许有多行非 0，它们来自不同次的恢复或切换；2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；i == 挂载根实例 ⇒ b ≤ 挂载根的 checkpoint_txg；i > 挂载根实例 ⇒ 判损坏。**例外**：打包记录类型 4 的实例表单元不走这条谓词，它的现行版本由根记录持有的物理指针唯一确定。**判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留**——这是扫描方向独立于遍历方向判它的依据（D13（验证路线） 已定项 12 双方向要求）；2026-09-05 前它只比诞生代号与当前 txg，崩溃后 txg 重发让那条判据对同 txg 两条时间线失效（E104（扫描重建的现行版本判定）：无实例表时崩溃世界复活 6） | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`，**只判遍历方向那一半**：走读到的每个被引用单元，头里的诞生代号 b、写序的实例代号 i 与尾段（码 1 是写序的事务号 n、码 2 / 码 3 是出生序号）要与**引用它的那条指针**记的相同——不同则「头记录的写序」不是这个单元的真实身份，谓词判的就是假输入；口径同恢复路径今天的前置检查（`crates/singlefs-core/src/recovery.rs` 五处报 I-1.2（块头写序已发布） 的 `InvariantViolated` 里「出生身份与指针不符」那三处；「诞生于根之后」那两处是谓词那一半，归 I-4.2（无被引用未提交块）），checker 按字段表自己解一份。⚠️ **扫描方向那一半（「任一单元」都按谓词判、i > 挂载根实例 ⇒ 判损坏）对着一份崩溃镜像判不了，没写**：① 判不成已发布本身不是违例——本行判据自己写着那是未发布的撕裂事务垃圾或被抛弃时间线的残留；② 「i > 挂载根实例 ⇒ 判损坏」只在恢复刚发布完新根的那一刻成立，而上一次挂载取了号、写出单元、一个根都没发布就崩时，盘上合法地留着 i > 最新根实例的在飞单元，那一支今天由 I-7.7（系统配置实例代号不低于根环） ① 按各盘系统配置的实例代号兜着。谓词本身照旧在用：`PublishedPredicate` 过滤 I-1.8（归并后版本全序） 的射程、判 I-4.2（无被引用未提交块）。打包记录类型 4 的实例表单元按本行的**例外**只免谓词那一半：根记录偏移 170 的实例表指针与链上每一片，指针里的出生 txg、实例代号、尾段与被指单元头里的值相等，仍挂这一条的遍历方向（已实现：`walk::Walk` 的 `judge_birth_identity_against_the_pointer`，被引用单元与实例表链上每一片都判；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）。坏镜像是 extent 树根头里的出生序号 +1、引用它的指针一个字节不动（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它，走读一个被引用单元都读不到的状态报「不适用」） |
| I-1.3 | 块头树 ID 一致 | 任一单元，其头记录的所属树 ID 与实际引用它的树一致。中央映射树不进树表，「实际引用它的树」取根记录（或记录新根段）里中央映射树根指针带的出生树，不写死 15：回退到无文件那一版之后再发第一个文件版本，八棵树从水位起重新发号（D8（核心索引结构） 已定项 8 ②），映射树的号随之变（主 agent 2026-09-23 定，C511（回退到无文件那一版之后诞生代怎么接） 第 3 步实现员查出写死 15 的读法在合法的回退历史上误红）。自证单元（含类型 4 实例表单元）与 I-1.1（块头自述逻辑地址）同口径显式豁免。⚠️ 打包记录单元（码 3）与被快照共享的数据单元头里的树 ID 是出生树，跨头共享时与引用树不同，那一情形的判法随 D9（加密） 那次「绑出生快照」的重开（C110（跨头共享与加密后元数据类的块头一致性读法））。这一射程覆盖码 3 全部打包记录类型，含类型 2（inode 树的叶，D8（核心索引结构） 已定项 6）：克隆头共享 origin 的叶时，叶头的出生树是 origin | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.4 | 块头 fsid 一致 | 任一单元，其头记录的 fsid == 系统配置 fsid（D18（块里携带什么信息）已定项 7 的字段；加密开启后与 KDF 绑定冗余，接受）。scrub 半边「本卷全部单元归属指向本卷」靠它才写得出（C74（无加密期的跨卷错向写没防线）补救的另一半） journal 记录头 2026-09-14 起也带 fsid 8（系统配置 fsid 低 8 字节，D23（journal 的角色与格式） 已定项 4）：fsid 不符的记录整条不进重放前缀、按断链处理（E142（新池新建文件的干跑） 第七次跑记的 G25） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.5 | 系统配置记管道身份 | 系统配置记录写出该镜像的管道身份（实现标识 + 版本），且非空。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：自举头段「写入者身份 20」= 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32，第一版写 `singlefs-rs` / 1；实现标识全零判红 | 未实现（条款已写、检查仍欠） |
| I-1.6 | 类标签登记且三处相等 | 任一单元，共同前缀偏移 6 的单元类型标签 ∈ D18（块里携带什么信息） 已定项 11 的登记表（码 0 与未登记码即判损坏）；偏移 7 的 8 个 flag 位第一版恒 0，非 0 一律拒收（前向兼容政策，D15（格式冻结政策） 判定表）；数据单元（码 1）五元组首字节 == 偏移 6；打包记录单元（码 3）类身份段首字节 == 偏移 6 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.7 | 打包容器合法与判定顺序 | 任一打包记录单元（码 3）：记录数 × 记录宽 ≤ 声明长度且声明长度 ≤ 32768 − 136（含 29 字节预留位的头，D18（块里携带什么信息） 已定项 16；2026-09-14 预留位从 28 加到 29，`cap(136, 40) = ⌊32632 / 40⌋ = 815` 与按 135 算同值，40 是第一版墓碑记录宽，D18（块里携带什么信息） 已定项 24），**先于**载荷校验和判（D23（journal 的角色与格式） 已定项 1 的 B 条同一纪律）；判定顺序恒为头校验和 → 类身份段 → 记录宽 == 登记表为该打包记录类型登记的宽（类型 2 是 140，D8（核心索引结构） 已定项 6；类型 1（墓碑）是 40、类型 5（谱系）是 16，D18（块里携带什么信息） 已定项 24；不等判损坏，只需头字段）→ I-1.2（块头写序已发布） / I-1.3（块头树 ID 一致） / I-1.4（块头 fsid 一致） → 边界 → 载荷校验和 → 逐条记录，任一步失败即中止；打包记录类型 4 的两种 kind 记录另按 D18（块里携带什么信息） 已定项 11 的实例表字段表判，不套用「记录宽 == 登记表的宽」那一步（一片记录宽恒 88，条数上限由片容量给——D18（块里携带什么信息） 已定项 11 2026-09-13 从 64 改宽）（对象级的判定——I-9.7（记录三段恒零）、I-9.8（记录 inode 等于 key）——在这条中止链之外，只让那条记录 EIO）；容器头声明一种打包记录类型、一棵出生树；打包记录类型 ∈ 第二级登记表，未登记类型的容器其物理指针不得出现在任何回收 / 复用操作的输入集合 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；类型 1、类型 5 两种记录宽未实现，三·3 实现） |
| I-1.8 | 归并后版本全序 | 码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组（字段表在 D18（块里携带什么信息） 已定项 7（码 1）与已定项 11（码 3）；归并键取这些字段但去掉载荷校验和那一段——它正是 ①「同一组的成员载荷相同」要比的那个量，进了键 ① 就恒真）之后，同 key 的各组两两全序键不等——码 1 的键是写序，码 3 的键是 (诞生代号, 实例代号, 事务号) 三元——诞生代号打头（同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同），事务号留在末位做同一 checkpoint 内的平局破除（固定点会迭代，同一容器在一个 checkpoint 里不保证只写一次）；同一组的成员载荷相同（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 副本归并，射程 w = 2；码 2 不进这条） | 已实现（2026-09-21，池级 checker `walk::check_pool_image` 判 ① 同一组的成员载荷相同；② 同 key 的各组两两全序键不等**只判码 3**——码 1 的键停着等 C464（I-1.8 对码 1 的全序键写窄了） 定案，checker 里显式写成 `TotalOrderKey::UndecidedForDataUnit`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.9 | 清扫水位内无可读垃圾 | 清扫水位（D5（快照 / 空间记账机制） 已定项 4 第 13 项）以内不存在可读垃圾；其外的可读垃圾每个 key 都被更大写序的可见版本或可见墓碑压住（管理员回退之后的镜像不在此列，见 C615（扫盘重建对挂着时的回退没有输入），出处 `research/prompts/m3-tomb-r1-main-verification.md` 第二节；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4；扫描重建的产物未晋升期间清扫禁止，这条在重建之后失效） | 未实现 |
| I-1.10 | 码 2 条目宽等于字段表宽 | 任一码 2 索引节点，头里自述的**条目宽**（D18（块里携带什么信息） 已定项 7 的自描述三段之一：key 宽 1 + 条目数 2 + 条目宽 2，偏移表在 D18（块里携带什么信息） 已定项 18、载荷布局在 D8（核心索引结构） 已定项 11）**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）。条目宽是**盘上读来的值**，`index_node_view` 只判了它 ≥ key 宽（extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」——不判就按字段表的固定偏移切，切片当场越界（增补 3 第 6 件 panic 面普查第二族，C476（盘上内容可控时走得到的 panic 有 21 处） R1 / R3 / R4 / R13）。条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它），否则判该节点损坏（2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做）。判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目。⚠️ **中央映射树不判**：它的条目宽由 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树） 定，那一条还开着；**树表条目（200 字节）两边都没写**，而走读照样按固定偏移切树表节点的条目——要不要把射程扩到它，账在 C476（盘上内容可控时走得到的 panic 有 21 处） | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::walk_tree_table_entry`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；条目宽为 0 时条目数非 0 那一条 2026-09-27 实审 A3-checker-2 落地：`index_node_view_judging_a_zero_entry_width`（`crates/singlefs-checker/src/walk.rs:747`）在解码失败理由是 `Verdict::EntryWidthZeroWithEntries`、且这棵树登记了条目字段表（`EntryFieldTableRegistration::Registered`）时报违例；树表条目与中央映射树不在射程内，只记走读失败；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
| I-1.11 | 映射 key 与单元头相符 | 任一中央映射条目，key 的类标签属于 D18（块里携带什么信息） 已定项 11 的登记表，且等于它的位置条目指的单元头里的类标签；key 里的出生 txg、实例代号、尾段（码 1 是写序的事务号 6 字节；码 2 / 码 3 是出生序号 4 字节，其后 2 字节补零恒 0）分别等于那个单元头里的诞生代号、写序实例代号、事务号或出生序号；key 里的出生树等于那个单元头里的出生树（码 1 / 码 3 取类身份段偏移 43，码 2 取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定）。不等判损坏。 | 已实现（2026-09-27，池级 checker `walk::check_pool_image`，判定在 `crates/singlefs-checker/src/walk.rs` 的 `walk_central_mapping_entries`；key 各段的偏移按 D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10 的字段表另写一份，不用实现的解析；判到的射程：类标签在登记表里且等于被指单元头的，出生 txg / 实例代号 / 尾段等于单元头的，码 2 / 码 3 尾段后 2 字节补零恒 0，码 1 / 码 3 的出生树等于单元头偏移 43，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不判；映射根全零（无文件的一版）时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（映射里数据单元那条 key 的出生 txg 3 → 4）与 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`。编号已落到代码：`crates/singlefs-checker/src/image.rs:38` 的常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 现在写的值就是这个编号本身，I-1.11（映射 key 与单元头相符）） |
| I-1.12 | 指针出生树等于被指单元头 | 任一被走读的指针，偏移 34 的出生树等于被指单元头里的出生树：码 1、码 3 取头偏移 43；码 2 的出生树取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定，不判（只写进状态列）。指针各段落点依据 D19（块指针的结构与宽度预算） 已定项 8（指针携带被指单元映射 key 除类标签外的各段）。inode 内部条目到叶容器那一类由 I-9.2（条目身份与子头相符） 罩住，不重判。 | 已实现（2026-10-02，池级 checker `walk::check_pool_image` 走读里的 `judge_pointer_birth_tree_against_the_unit_header`，只在两处调用：实例表链上每一片、extent 数据单元；码 1、码 3 取头偏移 43，码 2 不判，码 2 那一支不登记成「判了一部分」，那会让每份干净镜像整条报不适用；没有走到这两处指针的镜像报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C294（指针里的 key 各段与被指单元头相等没有不变量）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行） |

> I-1（块自描述） 类的适用范围是**任一单元**（数据与元数据都在内，
> D20（承重面：单元的原子性与自包含） 口径），判定字段的落点与宽度在
> [decisions.md](decisions.md) D18（块里携带什么信息） 已定项 7 的字段表（码 1 / 2）与已定项 11 的类身份段表（码 3）；
> 自证单元（journal 记录 / 根槽 / 系统配置槽）对 I-1.1（块头自述逻辑地址） 与
> I-1.3（块头树 ID 一致） 显式豁免，豁免的依据是它们的自证纪律另有落点
> （I-7（根环） / I-8（journal） 两类与 D22（单元原子性怎么合成） 已定项 21）。
> ⚠️ **豁免行是穷举的，按 D18（块里携带什么信息） 已定项 11 的登记表**：码 1 / 2 / 3 在字段表里有类身份段，码 16 / 17 / 18 在豁免行里点名，
> 其余码即判红（I-1.6（类标签登记且三处相等））——这是「穷举的显式分支」作用在不变量射程上。

```

**出处 `.claude/kb/invariants.md:48-82`（整段抄，未转述）**

```markdown
## I-7 根环 —— 类

**依赖 D22（单元原子性怎么合成），形态未定。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-7.1 | 根槽有效集非空 | 全部根槽位中，**自证校验和通过**的记录集合 S 非空 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.2 | 最新根可完整遍历 | S 中代号最大者，其 checkpoint 树根指针出发按 D4（校验和位置） 递归校验，能走完全部权威态（[decisions.md](decisions.md) D21（权威态与派生态的分界） 定义：单元 + 记账），无一处失配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.3 | 环健康性 | **环健康性**：S 中除代号最大者外，至少还存在一条更早代号的记录。不存在即判红——它意味着某次提交把上一代直接覆盖了，轮换逻辑已失效，**下一次撕裂将无路可退**。**例外只有一个**：S 全部是第 0 代（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成）已定项 8）时判绿，崩溃后回退到这个态的镜像同样判绿。⚠️ 射程：种子没被覆盖完之前，它们自己就充当「更早代号的记录」，连续两代落进同一槽的轮换 bug 要等 R × S 次发布把种子全部覆盖之后才会被这一条抓到 | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，例外（自证过的根全部是第 0 代）照判据原样实现；坏镜像建在 mkfs 刚写完那份上（三条第 0 代根的 checkpoint_txg 一起改成 1），例外那一支另有一份阳性对照；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。**「K 代」2026-09-13 起就指这个候选集**：平时回退下界 F 不动时是整个根环，盘紧时抬 F 缩到最近 4 个可退到的状态、且每块盘上最新的持久有效根都在内（D16（发布语义） 已定项 1）；原「K 是运行时策略、下限 ≥ 2」作废。[decisions.md](decisions.md) D22（单元原子性怎么合成）：**根环深度是块重用延迟的上界**（D22（单元原子性怎么合成）同时明令块复用的界不能是格式常量），不满足则环里的旧根是假的回退候选。「有效」按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的回退候选集判：(i, T) 有效 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被抛弃时间线的根不在候选集里，这只管回退目标的选择；被抛弃时间线今天只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。它们引用的块在离开根环之前同样不许重新分配、不许抹头（2026-09-13 用户定案，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账，D23（journal 的角色与格式） 已定项 14；留着的理由是第一轮判决 H6：崩溃恢复落到旧根之后，被抛弃的根引用的单元被复用、之后恢复落在它上面读不出）。⚠️ F_生效 按 D16（发布语义） 已定项 1「生效」取根上与系统配置里读得出的最大值；checker 今天只读根上带的 F，带 F 的根全坏时会在合法状态上误红这一条与别的按回退候选集判的几条（第三轮判决 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。崩溃后 defer 队列的内存态丢失，接手的是清扫准入「释放代 ≤ max(F_生效, 环里最旧有效根)」那条闸（谓词由 D16（发布语义） 已定项 1 给，2026-09-13；有效性按实例表判），保护宽度随之变 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，按每条回退候选集里更早的根各判一格：走读它引用的单元时校验和对不上或头用不了，就是它指着的块被复用或抹头；坏镜像是 mkfs 树表单元（只有第 0 代根还引用）改内容重封；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 被抛弃、还在根环里的根也判（2026-09-27，用户定「checker 现在就补被抛弃根那一半」，代码审阅第 7 条）：最新根的实例表有行时，根环里每条被判抛弃的根（有行 (i, Ti, ·) 且 T > Ti）各用一份自己的走读走一遍，只看读回来对不对——走出 I-2.1（校验和与内容匹配） 违例或走读失败就按这条根判红，判定丢掉、引用不进 I-3.1（已分配统计对得上） 的并集（`walk::judge_blocks_referenced_by_abandoned_roots`，用例在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`）；根环有读不出的槽且最新根的实例表有行时，没有违例就整条报不适用、不报成立；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形在这一半上照实红，等 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 修好转绿。 |
| I-7.5 | 根槽按判定宽度对齐 | 每个根槽的**起始偏移**是判定宽度的整数倍，判定宽度 = max(声明的 `physical_block_size`, 声明的 `io_min`)（`io_min` 为 0 视为没声明）；mkfs 在任何写之前先拒不按这个宽度对齐的几何，非常规几何由 mkfs 拒、不再是合法镜像。**逐槽判，不是只判第 0 个**——偏移半个槽的槽照样跨 2 个原子单元（E41（根环槽几何 2×4×256） 实测：对齐时风险槽数 1，偏移 256 时是 2），撕裂会连带打掉相邻的槽而没有任何东西报警 | 已实现（2026-10-02，池级 checker `judge_root_slots_aligned_to_the_decision_width`（`crates/singlefs-checker/src/image.rs`）逐槽判：判定宽度取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的（异构池取最大，`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 1 条），是声明值、注明「声明值，未探测」，探测比对仍欠 C388（checker 的判定宽度不自己探测）；mkfs 在任何写之前拒不按这个宽度对齐的几何（`crates/singlefs-core/src/make_filesystem.rs` 的 `first_slot_not_aligned_to_the_decision_width`）；异构池一份与逐槽判的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 5 行） ⚠️ **射程注（2026-09-13，D2（RAID 条带策略） 已定项 15）**：`io_min` 大于 `physical_block_size` 时按 `io_min` 判——已定项 17 收窄后根槽与系统配置槽在它的射程内。 |
| I-7.6 | 根环区域落在互不相同的盘上 | R 个区域**在 mkfs 时逐区域写下的设备身份**（D2（RAID 条带策略） 已定项 7 用户定案逐字「归属不再按 `(r × P) mod devs` 算，mkfs 时逐区域把设备身份写下来」）取满 min(R, devs) 个不同值，且没有一块盘背超过 ⌈R / devs⌉ 个区域（鸽笼下界，D22（单元原子性怎么合成） 已定项 14，2026-09-13 用户定案）：devs ≥ R 时就是两两不同；devs < R 时不拒绝挂载，容错承诺按盘数降级（E87（固定结构的放置） 量过的 2 盘 R = 3：掉盘 1 最坏 1 代、掉盘 0 最坏 2 代）。取值不满或某盘超过下界即**拒绝挂载**——名义容错 F 个失败域而实际区域挤在同一块盘上，是 E47（根环失败域的损失） 实测过的形态（`even_span` 在 18 个几何格里只有 3/18 真的落到 4 块不同盘） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.7 | 系统配置实例代号不低于根环 | 可写挂载独占打开成功的那个集合里（它至少是过半，过半是准入门槛不是写入范围），每块盘的系统配置实例代号按它两槽里全部自证过（校验和过且 fsid 与本池相同）的槽读，取号用同一个读法（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者，任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」；② 各盘的实例代号（每盘取它全部自证过的槽里最大的那个）不等时，较大者不出现在任何根记录、journal 记录、单元写序里，镜像记不下这次挂载独占打开的集合时报「不适用」，第一版两盘就是全部盘。取号没做完、全或无失败回卷期间的代号不等由 ② 判成立，不另开例外（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的取号规则）。只在 devs ≥ R 时根环才是系统配置之外的独立见证，第一版 R = 3 > devs = 2 时两路见证的失败域是同一对盘 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 用户定案改成 ① ②**：checker 读每盘两槽里全部自证过的系统配置，扫描方向加读 journal 环与三类单元头的写序；层 0 全量 262165 个状态 I-7.7（系统配置实例代号不低于根环） 违例 0（旧定义「各盘择到的相等且 ≥ 根环」下取号只落一块盘的 2 个状态判红）；坏镜像三份：没人引用的实例 2 孤儿（①）、journal 环里实例 2 的记录（①）、盘 0 最大号 2 而孤儿带 2（②） |
| I-7.8 | 根记录树 ID 水位不低于全池最大树 ID | **根环里全部根记录的「树 ID 水位」字段取 max，必须 > 该池中出现过的最大树 ID。**「出现过的」由 checker 全盘扫描算出：全部单元头的索引节点类身份段里那个树 ID（D18（块里携带什么信息） 已定项 7），并上树表条目里的树 ID；**不许与运行时共用同一段代码**（D13（验证路线） 已定项 5）。取 max 的范围是「根环里**全部**根记录」不是「回退候选集」——被抛弃时间线的根仍然承载它那一代发过的号，形态与 D23（journal 的角色与格式） 已定项 14 给 checkpoint_txg 取值那句同源。**判别力**：抓两类——① 某次发布没更新水位；② 水位被实现成「本 checkpoint 发出的最高号」而不是**累计高水位**（后者的镜像是连发 R + 1 个 checkpoint 各建一棵树，第一棵的号会随根轮出环而被重发）。⚠️ **只许写成不等式**：水位是「下一个可用号」的上界，写成等式会把正常镜像判红 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 崩溃孤儿两条流（第一个文件版本那次发布的每个前缀 → 可写挂载 → 挂载之后六种动作）在 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs`；中间实例行（T = 0）不排除的坏镜像是 `checker_known_bad_images.rs` 的 `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`（实例 2 的根槽全清零之后实例 3 写中间实例行 (2, 0, 0)，根环水位压低之后只红 I-7.8（根记录树 ID 水位不低于全池最大树 ID）；回退行那一道随回退行删掉，D18（块里携带什么信息） 已定项 11）；实现在 `walk::written_after_its_instance_was_last_applied` ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲；2026-09-24 主 agent 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 1 条补第二道，被攻过零轮）**：崩在发布之前的那个事务写出的孤儿节点不算「出现过」。扫描方向不数两类码 2 节点：① 诞生代号超过根环里最大 checkpoint_txg 的；② 写序实例 i 在最新根指着的实例表里有行 (i, Ti, ·)、Ti > 0、而诞生代号 > Ti 的——行说实例 i 只被施加到 Ti，之后写的没发布过；下一个实例第一次发布取的 txg 可以等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 取等号），只靠 ① 挡不住（第一个文件版本崩在记录落盘之前、再可写挂载的两条流上 130 格在合法状态上判红）。Ti = 0 的行（恢复写的中间实例行）不排除：中间实例发布过的号仍要算（D8（核心索引结构） 已定项 8 ②）。节点头 fsid 不是本池的照旧数。按字面数孤儿，层 0 全量 262165 个状态里 261891 个判红（水位 11 而盘上已有树 ID 11–15 的孤儿节点），见 records 2026-09-13-总审核 十一·七 ⚠️ **判法补一句（已实现：`walk::check_pool_image` 末尾在取 max 那一格之后再判最新那条根自己的水位）**：另判最新那条根的水位不低于全池最大树 ID。只取根环里全部根的 max 判不到把最新那条根的水位压低的镜像：攻方量到 checker 49 条 0 违例，而 core 的 `recover` 报 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 失败；core 重发树 11..18 的状态 checker 也 0 违例。用攻方那两份坏镜像钉住（只压低最新那条根的水位；建一次树之后经会话入口覆盖写 24 次把发树那条根轮出去、再把环里每条根的水位压到 11，只红它），被攻过零轮（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 9、10 行） |
| I-7.9 | 回退下界 F 不高于抬 F 的上限 | **抬 F 的那一条根**（它带的 F 比同一实例里 txg 比它小的最新那条根带的高；新实例的第一条根没有同实例的前一条，不算抬——它带的是恢复算出的 F_生效；管理员回退是现行实例里的一次向前发布，带的 F 与前一条相同，不算抬；一次抬 F 推的几次空发布只有第一条算）根记录里那个回退下界 F（D22（单元原子性怎么合成） 已定项 7 的字段），按这条根 flags 里的**卸载记号**（D22（单元原子性怎么合成） 已定项 7 flags 位 0）分两支判：**带卸载记号的**，F 不高于根环里 txg 比它小的最新那条根的 txg（正常卸载把 F 抬到现行那一版的 txg、不另判上限，D16（发布语义） 已定项 1「抬 F 的上限」）；**不带的**（准入抬 F），F 不高于**用它之前的根算出的** D16（发布语义） 已定项 1 的准入抬 F 上限：min(每块盘上最新的持久有效根的 txg, 第 4 新的非空持久有效根的 txg)，非空有效根不足 4 个时取最旧有效根的 txg。有效 = 按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F；「之前的根」取根环里 txg 比它小的那些。只判抬 F 的那一条，不在后来每张镜像上重算：F 抬过之后再回退，后来的根会把当初撑着 F 的非空根判成无效，现算的上限会低于 F，而 F 不回落（用户定案 2026-09-24，理由见实二报告：按「有效根不要求 txg ≥ F」的读法，固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与两个随机种子在合法状态上判红）；非空 = 它的树表里 inode 树、extent 树的根指针与它前一条有效根（按 txg 排）树表里的不同——比的是树表里这两棵树的条目，不比树表单元自己的落点（D16（发布语义） 已定项 1 表格后那段「「非空」从盘上怎么认」）；数「第 4 新的非空」时去重：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入（同一段）。⚠️ 它判的是一张镜像上 F 有没有抬过头，不判 F 该不该抬（那是准入的事，D28（挂载期承诺量） 已定项 1）。⚠️ **卸载记号盘上没有别的东西核它**：准入入口误打记号、F 又抬过头，按带记号那一支放过（第三轮判决第二节 K6：(b) 丢的正是「准入抬过头却带了卸载记号」那一格）——这一格归层 0 录制流另核「卸载记号只由卸载入口打」（C557（卸载记号只由卸载入口打没有检查）），不归这一条。⚠️ F 写进了系统配置而带新 F 的根没落盘或坏了、恢复后生效值怎么取的那一类，归层 0 与 C556（checker 与层 0 不读系统配置里的 F），不归这一条。算上限要比的有效根树表读不出时，它空不空判不了、不猜：F 不高于把这些根全算空得出的上限就成立，高于全算非空得出的上限就违例，落在两者之间这条根不判 | 已实现，两支（不带记号那一支 2026-09-24、带卸载记号那一支 2026-09-26）：池级 checker `walk::check_pool_image` 的 `judge_rollback_floor_raises_against_their_ceilings`，根环里每条抬 F 的根各判一格，按根记录读出的卸载记号分支（checker 读成 `UnmountMarkerView`）。**不带记号那一支**：坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（步 5 那段历史上抬到上限 11 与之后再发一版都成立、抬到 12 只红它）；固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与随机历史各档不红；层 0 `crash_enumeration_fixed_script_stream.rs` 抬过 F 的三条流要求真被评估过；`crates/mutations.tsv` 7 条变异（判法拿掉、上限取大、第 4 新改第 3 新、有效按新 F 取、不限同一实例、一条都不认、「4 个不同状态」不去重）；用户 2026-09-24 定义，里程碑「覆盖写、释放、回退与复用」收口表第 26 行。**带卸载记号那一支**：坏镜像同一文件的 `marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant`（正常卸载之后的镜像按带记号那一支判 6 ≤ 6 成立、按准入那一支会红；把那条带记号的根的 F 改成 7 只红它）；变异「checker 把每条根都读成不带卸载记号」「带记号那一支把不高于前一条根的 txg 判成低于」；记号的写读在 `crates/singlefs-core/src/root_record.rs`（写者照 `unmount_marker` 写 flags 位 0、读者拒记号以外的位），写者、读者各有一条变异（实二） |
| I-7.10 | 已停用·回退见证表删了 | **此编号不再使用**——2026-09-26 管理员回退改成挂着时的一次向前发布，系统配置里的回退见证表删掉（D23（journal 的角色与格式） 已定项 14、D22（单元原子性怎么合成） 已定项 9）⇒ 这一格**无对象**。曾要求：每块盘两槽里自证过的系统配置槽，槽内偏移 481 起的回退见证表解得开（条数不超过 R × S − 1、条目严格升序、目标实例代号小于新实例代号、条数之后全 0），同一个新实例代号在各盘各槽里记的回退目标相同 | 不适用 |
| I-7.11 | 已停用·见证择根删了 | **此编号不再使用**——2026-09-26 回退见证表删掉（D23（journal 的角色与格式） 已定项 14），择根不再跳过被见证抛弃的根 ⇒ 这一格**无对象**。曾要求：所选根是根环里自证过、不被回退见证表抛弃的根里 (txg, 实例代号) 最大的那一条；根环里至少一条自证过的根不被见证表抛弃；见证表里新实例代号不大于所选根实例代号的每一条，所选根指着的实例表罩得住它 | 不适用 |
| I-7.12 | 系统配置 F 不低于同盘根上的 F | 每块盘分开判：这块盘两槽里自证过（校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）的系统配置槽中回退下界 F（D22（单元原子性怎么合成） 已定项 9 的字段）的最大值，不低于这块盘根槽里每一条自证过的根带的 F（D22（单元原子性怎么合成） 已定项 7 的字段）。抬 F 那一串先把新 F 写进每块盘的系统配置、过一道屏障，才发第一条带新 F 的根（D16（发布语义） 已定项 1「抬 F 那一串」），所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里。某块盘两槽都自证不过时，那块盘不判；一块盘都判不了时整条报「不适用」。判别力：把「先写系统配置」挪到第一条根之后，崩在两者之间的状态必须红 | 已实现（2026-09-26，池级 checker `walk::check_pool_image` 的 `judge_system_configuration_floor_against_the_roots_on_each_device`：每块有自证过的系统配置槽的盘判一格，那块盘上一条根都没有时那一格成立；两槽都自证不过的盘不判，一块盘都判不了时整条报「不适用」。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant`：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12（系统配置 F 不低于同盘根上的 F） 真被评估过且成立，盘 1 两槽的 F 改回 0 只红它、红在盘 1 txg 16 那条带 11 的根上；变异「不比系统配置里的 F（每块盘恒成立）」「checker 不读系统配置里的 F（读成 0）」（实二）。条款仍是主 agent 2026-09-26 按 SysPre 的写序推的，没三方；层 0 每个崩溃状态都跑池级 checker，按判法每个状态都评估得到它（推的，层 0 没跑） |
| I-7.13 | 系统配置池级字段在读者收的范围里 | 任一自证过（magic 与整槽校验和对，`check_system_configuration_slot`）、incompat 位认得的系统配置槽（字段住 D22（单元原子性怎么合成） 已定项 9 的字段表，读者收的界在 D22（单元原子性怎么合成） 已定项 2 与已定项 16）：格式版本 = 1（`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）、加密类型 = 0（第一版恒关，`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`）、固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号（`geometry_of` 的 `fixed_structure_slot_spacing_lies_in_the_format_range` / `physical_block_size_fits_a_root_slot` / `journal_ring_bytes_lie_in_the_supported_range`）。任一盘任一槽不满足其中一项即判红：checker 报违例、不作保，且该池其余不变量一律报「不适用」，与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）。**判的字段与 core 收同一张表**：格式版本、加密类型、槽距（上界用同一槽自述的根环起点）、根槽宽、环长、单元区起始槽号（偏移 417）= 环长现算的起点且在 64 槽段边界上、journal 环起点（偏移 325）与根环起点（偏移 371）等于第一版常量，共八项（D22（单元原子性怎么合成） 已定项 16 第 1 条）。R（区域数）与 S（每区槽数）越界不在这一条里，仍归「这一槽不可择」 | 已实现（2026-09-27，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_system_configuration_values_the_reader_accepts` 在 `walk::check_pool_image` 里调用，任一盘任一槽带越界值时该池其余不变量整批报「不适用」并提前返回。**八项 checker 与 core 两边都判**：checker 解槽时判格式版本与加密类型（`crates/singlefs-checker/src/lib.rs`），其余六项在 `geometry_of` 判——先判 journal 环起点（偏移 325）= `JOURNAL_RING_START_SLOT`（1024）、根环起点（偏移 371）= `ROOT_RING_BASE_SLOT`（64）（两个起点先于槽距判：槽距上界按同一槽自述的根环起点算），再判槽距、根槽宽、环长，最后判单元区起始槽号（偏移 417）= 同一槽自述的环起点 + ⌈环长 ÷ 16384⌉（`unit_area_start_slot_is_the_slot_after_the_journal_ring`）且是 `CLUSTER_SEGMENT_SLOTS`（64）的整数倍；新加的四判各报一个 `Verdict` 成员（`JournalRingStartSlotNotTheFirstVersionConstant`、`RootRingBaseSlotNotTheFirstVersionConstant`、`UnitAreaStartNotTheSlotAfterTheJournalRing`、`UnitAreaStartOffTheClusterSegmentBoundary`），都归 `value_refused_by_this_reader`（整池不作保）；常量取自 `singlefs-format`，算式 checker 自己写。core 一侧 `crates/singlefs-core/src/system_configuration.rs` 的 `parse_slot` 读出偏移 325、371 两个字段，`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 按格式版本、加密类型、槽距（上界用同一槽自述的根环起点，`largest_fixed_structure_slot_spacing_under_the_root_ring_base`）、根槽宽、根环起点、journal 环起点、环长、单元区起点的次序判，不收的值报 `SystemConfigurationValueOutsideWhatThisReaderAccepts` 的一个成员（两个起点是 `RootRingBaseNotTheFirstVersionSlot`、`JournalRingStartNotTheFirstVersionSlot`），经 `RecoveryFailure::SystemConfigurationValueRefused` 整池拒。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：`a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`（盘 1 槽 0 加密类型改成 1、重算整槽校验和，只红 I-7.13（系统配置池级字段在读者收的范围里）、其余不变量全报不适用），`each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`（几何字段各一格，各只红 I-7.13（系统配置池级字段在读者收的范围里）、违例说明点名被拒的那个成员）。两边收同一张表由 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 钉住：新池新建文件之后的镜像上两块盘四个系统配置槽改同一个几何字段、整槽校验和重封，五格各一条用例，每格 core 的只读与可写挂载都拒成同一个 `SystemConfigurationValueRefused` 值、两块盘逐字节不变，checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、其余不变量报不适用；根环起点写 0 那一格两边点名的成员不同（core 先判槽距上界、报槽距，checker 先判根环起点、报根环起点），照两边各自的判定次序钉（用户 2026-09-27 定「读字段，不等于常量就整池拒」，`records/2026-09-24-里程碑二收尾调度.md` 第 293 行；代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节「core 与 checker 对系统配置几何字段的读法」那一格）|
| I-7.14 | 系统配置本盘设备号等于盘在池里的身份 | 每块盘两个系统配置槽里，每个自证过（magic 与整槽校验和对）、fsid 与本池相同的槽，自举头里的本盘设备号等于这块盘在池里的身份。字段住 D22（单元原子性怎么合成） 已定项 9 的系统配置字段表（自举头「本盘设备号 4」）；实现一侧是 D18（块里携带什么信息） 已定项 11 里挂载核盘表那一句：可写挂载比盘表（`crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`，`OwnDeviceNumberDiffersFromTheIdentityHandedIn`）。别的池的槽归 I-1.4（块头 fsid 一致） 判，不在这一条里 | 已实现（2026-09-28，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_own_device_number_is_the_device_identity` 在 `walk::check_pool_image` 里调用，每块盘每个自证过、fsid 属于本池的系统配置槽判一次，本盘设备号与 `ImageReader::devices` 给的这块盘的身份比。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：两块盘四个系统配置槽的本盘设备号互换，`devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant` 钉只红 I-7.14（系统配置本盘设备号等于盘在池里的身份），这份坏镜像也在 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 的清单里；变异表 `crates/mutations.tsv` 里对着这一判的三行（判定恒真、池级走读不调这一判、坏镜像写回原值）改法 A 的 `prove-red.sh` 各抓到（`research/prompts/m2-impl-r2-fixes-a-implementer-report.md` 第 77–78 行）。合法历史上不判违例：门禁 harness-model-differential-and-scenarios 随机历史那一趟 I-7.14（系统配置本盘设备号等于盘在池里的身份） 判绿 4155 次、不适用 0 次（同一报告第 189 行）；层 0 快档与全量没跑）|
| I-7.15 | 各盘所选系统配置槽设备数相同 | 各盘所选的系统配置槽（自证过、世代号最大者）里的设备数相同（D22（单元原子性怎么合成） 已定项 9 的字段）。不要求等于盘表：降级镜像（掉了一块盘）里 checker 手里的盘数小于设备数是合法的。形态抄 I-1.4（块头 fsid 一致） 的逐盘循环；core 的 `choose_system_configuration` 在几块盘的设备数对不上时报 `SystemConfigurationsDisagree`、整池挂不上，两边同表。 | 已实现（2026-10-02，池级 checker `walk::check_pool_image` 逐盘把各盘所选槽的设备数同第一块盘所选槽的比；坏镜像是盘 1 两个槽的设备数改成 3 并重封，只红它，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C466（几块盘的系统配置设备数对不上时 checker 判不出）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 7 行） |
| I-7.16 | 系统配置槽世代号与槽号的关系 | 每块盘上，任一可择的（自证过、池级字段在读者收的范围里）系统配置槽 i：世代号 ≥ 2 时世代号 mod 2 == i；两个槽都可择时世代号相差不超过 1，相等时只许都是 1（mkfs 写下的两槽世代号都是 1，D22（单元原子性怎么合成） 已定项 16）。带越界值的槽归 I-7.13（系统配置池级字段在读者收的范围里） 整池拒，不进这一条。 | 已实现（2026-10-02，池级 checker `judge_system_configuration_slot_generations_follow_the_rotation`（`crates/singlefs-checker/src/image.rs`）：奇偶一格、差值一格，差值那一格取「两槽都是 mkfs 写的 1，或相差恰为 1」；差值与奇偶各一份只红它的坏镜像，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 7 行与第四节用户定案第 1 条） |
| I-7.17 | 根槽位置等于 txg 算出的位置 | 根环里任一自证过的根所在的槽位 (区域 r, 槽 s)：`checkpoint_txg` ≥ 1 时 r == txg mod R 且 s == (txg div R) mod S；`checkpoint_txg` == 0（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成） 已定项 8）时只判 s == 0，不判 r。公式是 D22（单元原子性怎么合成） 已定项 16 的定案。根环补洞判决要的「槽位与 txg 对得上」就是这一条（`research/prompts/m3-ring-r1-main-verification.md` 第二节第 6 行）。 | 已实现（2026-10-02，池级 checker `judge_root_records_live_in_the_slot_their_txg_names`（`crates/singlefs-checker/src/image.rs`）：根环里每条自证过的根逐条判，第 0 代只判槽 0；公式同 core 的 `target_for_publish`（`crates/singlefs-core/src/root_ring.rs`）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 8 行） |

> ⚠️ **I-7.3（环健康性） 的价值在于它不需要真的崩溃一次才能发现 bug。**
> 轮换逻辑写错（例如连续两代落到同一槽位）在**下一次正常提交后的 checker 扫描**里就会被抓到，
> 而不是等到某次撕裂真的发生、发现无处可退时才暴露。
> 这与「校验和会不会红」是两类判别力：前者查机制活着没有，后者查数据对不对。

> ⚠️ **I-7（根环） 类验不了「失败域独立」这个假设。** 根环的多区域布局依赖
> 「盘首与盘尾的物理故障 / 掉电撕裂事件统计独立」，而对着镜像判不出这一条——
> 它只能验证「机制按设计跑了」，不能验证「设计的假设是真的」。
> 该假设**本机无证据**，见 [decisions.md](decisions.md) D22（单元原子性怎么合成） 已定项 2。

```

**出处 `.claude/kb/invariants.md:83-124`（整段抄，未转述）**

```markdown
## I-8 journal —— 类

**依赖 D23（journal 的角色与格式），形态已定（定长环 + tail 住系统配置槽），checker 未实现。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-8.1 | 环几何够大 | 每块盘每个自证过的系统配置槽里三个字段各占一格：journal 在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋，journal 最坏占用 = 在飞上限 × 4096，安全系数 F = `JOURNAL_SAFETY_FACTOR`（格式常量，checker 与 core 都判等于，不是下界），任一不等判红；另判一格：环里同一实例、同一 `checkpoint_txg` 的自证记录（同一次发布）不多于在飞记录数上限条。`环大小 ≥ F × 任一事务的最坏 journal 占用` 是这几格的推论，不另判。mkfs 时算、mount 时复算比对，core 挂载同判。三格任一不等时与 I-7.13（系统配置池级字段在读者收的范围里） 同口径整池不作保：checker 报这一条违例，其余不变量一律报「不适用」，与实现整池拒绝挂载一致；I-7.13（系统配置池级字段在读者收的范围里） 与这一条同时不成立时走 I-7.13（系统配置池级字段在读者收的范围里） 那一支（只红 I-7.13（系统配置池级字段在读者收的范围里），这一条报不适用）；同一次发布记录数那一格不触发整池不作保。 | 已实现（2026-10-02，池级 checker：三个字段等式在 `judge_journal_ring_geometry_fields_agree`（`crates/singlefs-checker/src/image.rs`），每块盘每个可择的系统配置槽各判三格：F 等于 `JOURNAL_SAFETY_FACTOR`，在飞上限等于环长 ÷ 4096 ÷ 该槽里的 F 取整（F 为 0 判红、不除），最坏占用等于在飞上限 × 4096（乘法溢出判红）；带越界值的槽归 I-7.13（系统配置池级字段在读者收的范围里）、不可择的槽不读几何；同一次发布的记录数不多于在飞上限那一格在 `walk::judge_records_of_one_publish_within_the_in_flight_record_limit`，取所选槽记的上限。F 那一格 2026-10-03 从「F ≥ 2」改成等于格式常量，三格不等的整池不作保那一支加在 `walk::check_pool_image` 里 I-7.13（系统配置池级字段在读者收的范围里） 那一支之后、I-8.5（tail 槽有效集非空） 之前（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 3、4 行）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（`each_ring_geometry_and_root_slot_alignment_bad_image_reddens_only_its_own_invariant`：F 取 1、2、4 与只改没被择的那一槽 F 各一份；`journal_ring_geometry_fields_that_disagree_redden_only_the_ring_geometry_invariant_and_every_other_is_not_applicable`：三格不等时其余不变量报不适用）。core 挂载同判：`crates/singlefs-core/src/recovery.rs` 的 `journal_ring_geometry_fields_recompute_from_the_ring_length`，放在 `system_configuration_values_this_reader_accepts` 的最后。在飞上限那一格复算取槽里的 F、core 取格式常量，F 不等于格式常量时两边都已因 F 那一格拒，不影响判红与否；对应的账是 C28（环几何不变量未实现）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4 行） ⚠️ **口径注（D28（挂载期承诺量） 已定项 4）**：ckpt_cost 按当时各记录树的高现算、挂载期内会长；实现上最坏占用 = 在飞上限 × 4096，由环长与 F 倒算，取的是上界、不随树高变（`research/prompts/m3-inv-a-r1-main-verification.md` 回看决策那一行）。 |
| I-8.2 | 记录头不跨原子单元 | 任一 journal 记录的头完整落在一个 `physical_block_size` 单元内。该宽度**运行时探测**，不许硬编码 | 未实现 |
| I-8.3 | 重放前缀严格连续 | 被重放的记录集合，其 `jsn` 严格连续（`jsn == expected`，断号即止；链从所选根覆盖的最后一条记录之后接，跨实例边界即停，记录水位按实例代号为主比——D23（journal 的角色与格式） 已定项 14 的注，2026-09-13），且整段落在在飞记录数上限之内。⚠️ **这一条不是前缀纪律的全部**：D23（journal 的角色与格式） 已定项 14 的前缀口径共五条，这里只落了其中三条半——**「提交标记齐全的事务才施加」与「施加前逐项验证点名单元」两条不在这条判据里**，而两者的处境不同：前者 2026-09-23 已落成 I-8.8（前缀里的事务不被切开），checker 判定与崩溃点重放那一半仍欠，记在 C37（事务与记录的对应无定义）；后者**已经有了**，只是不住这一条——它住 D16（发布语义） 已定项 7，两条故障注入自证 2026-09-14 在层 0 做出（C76（发布时序没有权威落点））。⇒ 照这一条实现恢复施加逻辑的人会漏掉那两道防线，而它们正是挡住「半个事务」与「点名单元已损坏的记录」的 | 未实现 ⚠️ **判据补一句（2026-09-13，D16（发布语义） 已定项 4）**：前缀不得停在一次发布的中间——一次发布整体施加或整体不施加。 ⚠️ **再补一句（2026-09-14，D23（journal 的角色与格式） 已定项 18 用户定案）**：「在飞记录数上限」的语义就是**这个前缀最多 N 条**（N = 环槽数 ÷ F），不是 XFS 那种「最后 N 条内算撕裂、之外算损坏」；**任一校验和不过的记录一律断链即止**——停在它之前那一条，第一版不区分撕裂与损坏、不因此拒绝挂载。 |
| I-8.4 | 重放幂等 | 任一 journal 记录被重放两次与重放一次，产生的读出内容相同（等价关系取「读出的内容相同」，不比扇区、实例代号与行） | 未实现（实现在途：取 R-换，在崩溃注入的二次崩溃上判恢复加写行那一整段；被攻过零轮，实现时先在崩溃注入上量误红；`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 2 条） |
| I-8.5 | tail 槽有效集非空 | 池里至少有一块盘的两个系统配置槽里有一个槽自证过（magic 与整槽校验和对）：tail 住这些槽（D23（journal 的角色与格式） 已定项 3），这个集合就是全部 tail 见证者。全池皆废判红（mkfs 没做完、写到系统配置之前失败的盘面也是全池皆废，照判红，故障注入的已知红清单认这一类，用户 2026-10-03 定）；单盘全废不判违例，走旁路报告：另报哪块盘少于两槽自证过、可写挂载会拒。对着只读恢复判。自证过而不可择的槽（R / S 越界、incompat 位不认得）与带越界值的槽都算自证过；I-7.13（系统配置池级字段在读者收的范围里） 带越界值整池拒、或 I-8.1（环几何够大） 三格不等整池不作保的那一支里，这一条不判、报「不适用」。 | 已实现（2026-10-02，池级 checker `judge_some_device_carries_a_self_verified_system_configuration`（`crates/singlefs-checker/src/image.rs`）：把「不自证」（magic 或整槽校验和不对）与「自证过而不可择」分成两类，后一类计入自证过（`slot_verifies_itself`），别的不变量在全池皆废时照旧报不适用；单盘全废走旁路报告 `system_configuration_redundancy_report`，不进违例通道；自证过而不可择的四槽一条都不红的一份坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（`self_verified_system_configuration_slots_no_reader_can_choose_still_witness_the_tail`）；对应的账是 C461（checker 判「挂不上的镜像」整片报不适用）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 6 行与第四节用户定案第 3 条） |
| I-8.6 | 反向链算法 | 任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 311 字节头，`header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录反向链恒 0**（D23（journal 的角色与格式） 已定项 10 / 已定项 19，2026-09-13 用户定案，链按实例算是 2026-09-14 用户定案，头宽随 D23（journal 的角色与格式） 已定项 4 现行 311）。不等的记录不进重放前缀。jsn 的计数器为 0 的记录判违例：它没有本实例内的逻辑前一条，反向链无从定义（D23（journal 的角色与格式） 已定项 18 的落点式 `(计数器 − 1) mod 槽数 × 4096` 从计数器 1 起；用户 2026-09-27 定）。⚠️ 链按实例算，所以跨实例边界的两条记录之间**不判**这一条——实例边界由实例代号挡着（I-8.3（重放前缀严格连续)） | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，逐盘判：计数器 1 与跨实例边界的第一条判反向链恒 0，同实例前一条在盘上时判链值，前一条不在盘上不判；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 里环还空着的 4 个状态报不适用，其余每个崩溃状态都判）；计数器 0 判违例（2026-09-27，`walk.rs` 的 `ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`，用例 `a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant` 在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`） |
| I-8.7 | 实例内事务号不重号 | 判环里全部自证过的记录，同一实例按 `jsn` 的计数器看：① 按计数器相邻的两条非 0 记录，事务号不减（事务号 0 是不承载事务的空发布记录，不进这个比较）；② 一个事务号的记录在计数器上连续，被别的记录（含事务号 0）隔开之后不再出现。D23（journal 的角色与格式） 已定项 7 逐字：「事务号按实例计数、从 1 起」「记录按事务号顺序追加……这是实例表行的 W 能当精确前缀的依据」；同一项定一个事务可以跨多条记录，所以相邻两条可以同号。 | 已实现（2026-09-21 立时判「非 0 事务号严格递增」；2026-09-23 按 D23（journal 的角色与格式） 已定项 7 改成不减加同号连续。池级 checker `walk::check_pool_image` 的 `judge_transaction_numbers_per_instance`：逐盘按实例判，同一实例按计数器相邻的两条非 0 记录，号不等时后一条严格大于前一条（①），号相等时两条之间不许坐着同一实例的别的记录（②，事务号 0 的也算；中间只有读不出的槽或别的实例的记录不判红）；「被别的非 0 号隔开之后又出现」一定伴着一处 ① 红，不另判；同一实例非 0 的记录不足两条时报不适用。坏镜像三份（交错两份、事务号倒退）在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，每份只红这一条，违例说明标出红的是 ① 还是 ②） |
| I-8.8 | 前缀里的事务不被切开 | 判环里全部自证过的记录：③ 同一实例同一事务号的一组记录里，**带提交标记的至多一条，且它是这一组计数器最大的那一条**；提交标记那一字节只许 0 或 1；④ **一组里一条提交标记都没有时，这个实例在它后面不许再有任何记录**（事务号 0 的空发布也算；别的实例的记录不算）：D23（journal 的角色与格式） 已定项 7 逐字写「一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加」，实例切换之后这个实例不再写任何记录。它与 I-8.7（实例内事务号不重号） 的 ① ② 合起来，保证 D23（journal 的角色与格式） 已定项 7 原话那个恢复算法「丢掉提交标记还没出现的那个事务的全部记录」**恰好丢掉、且只丢掉**被崩溃切开的那个尾巴；`crates/singlefs-core/src/recovery.rs` 今天那一步写成「按 jsn 扫到第一条不带提交标记的记录就停」，只对一事务一条成立，一事务跨多条记录时要改，归并行线一。四条里任一不成立，恢复要么丢掉一个已经提交的事务、要么施加半个事务，而盘上没有任何痕迹。⚠️ **今天可达镜像上它恒真**：第一版一条记录一个事务、`is_commit` 全仓没有一处写 false，所以 ③ ④ 今天都不会被可达历史违反。**判别力只能靠坏镜像**，坏镜像不做这条就是恒真的检查——恒真的检查比没有检查更糟。⚠️ **它管的是条款允许的形态，不是第一版的形态**：D23（journal 的角色与格式） 已定项 7 定的是「可以跨多条，记录头带事务边界字段」，而第一版按 C310（事务切分纪律与记录数口径打架） 一条记录一个事务。I-8.8（前缀里的事务不被切开） 挡的是**一事务跨多条记录落地那一天**。⚠️ **它只判一份镜像上的记录流自洽，不判「施加的结果等于整数个事务」**：后者要拿「这次事务该有的全部效果」当 oracle，一份镜像上判不出来（形态同 I-4.3（提交原子） 那一行），归层 0 崩溃点重放。 | 已实现（2026-09-23，池级 checker `walk::check_pool_image` 的 `judge_commit_markers_per_transaction`：逐盘，一组 = 同一块盘上实例代号与事务号都相同的非 0 记录；③ 两条以上的一组里带提交标记的至多一条、且是计数器最大那条；提交标记字节每条记录都判（事务号 0 的也算），不是 0 或 1 即红；④ 一条提交标记都没有的组，后面同实例还有任何一条记录（事务号 0 的空发布也算）即红。没有 ③ 的对象也没有 ④ 的对象 ⇒ 报不适用，提交标记字节单独成立不算成立；第一版一事务一条、每条都带提交标记，层 0 每个崩溃状态都判、全部报不适用。阳性对照（一事务两条、提交标记在 jsn 最大那条，I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 都成立）与四份坏镜像（提交标记放错位置、没提交的事务后面还有号更大的记录、没提交的事务后面跟一条空发布、提交标记字节写成 2，每份只红这一条）在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，违例说明逐条标出红的是哪条判据） |
| I-8.9 | 一次发布的记录序号连续且只有末条带标志 | 同一实例、同一 `checkpoint_txg` 的记录（一次发布写出的全部记录）：「本次发布内序号」依次是 1..N、与 jsn 同步（jsn 连号时序号也连号），记录标志位 0 恰好只在序号 N 那一条上为 1，记录标志其余位为 0；序号 0、一次发布之内跳号、没有末条或多于一条末条，都是违例。只判读得出的记录，读不出的那条不判（D23（journal 的角色与格式） 已定项 4、17，用户 2026-09-24 定，`records/2026-09-24-里程碑二收尾调度.md` 第三节「序号与末条标志的不变量」） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_publish_ordinals_and_last_record_flags`；阳性对照与七份坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`） |
| I-8.10 | 记录新根段与同代根相同 | 某块盘 journal 环里自证过的记录 J，带末条标志（记录标志位 0 = 1），且这块盘上同（实例代号, `checkpoint_txg`）带末条标志的记录只有这一条；根环里任一自证过、实例代号与 `checkpoint_txg` 都与 J 相同的根 R，J 的新根段四段与 R 的同名四段逐字节相同：新根段 [0..86) = R[36..122)、[86..172) = R[256..342)、[172..180) = R[122..130)、[180..188) = R[130..138)（树表单元指针、中央映射树根指针、树 ID 水位、回退下界 F，D23（journal 的角色与格式） 已定项 15）。只认带末条标志那一条；同（实例, txg）有两条以上不同内容的根不判，归 C88（根环的时间线判别未实现）；末条标志不唯一归 I-8.9（一次发布的记录序号连续且只有末条带标志）。 | 已实现（2026-10-02，池级 checker `walk::judge_new_root_segment_of_the_last_record_against_the_roots`：逐盘按 (实例, txg) 分组，带末条标志恰一条才判，与根环里同 (实例, txg) 的每条自证过的根比四段，一对都没比到报不适用；坏镜像取记录一侧，把 F 改一位并重封头校验和，另有树表指针、映射根指针、树 ID 水位各一份，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 4 行） |
| I-8.11 | 点名项与被指单元头相同 | 任一自证过的 journal 记录里任一点名项的每条位置条目：若它指着的那一份单元的整单元校验和与位置条目里的相符，则单元头里的类标签、诞生代号、key 尾段与点名项偏移 28、37、45 的同名字段逐字节相同（码 1 尾段 = 写序实例代号 4 + 事务号 6，码 2 / 码 3 尾段 = 实例代号 4 + 出生序号 4 + 补零 2，补零恒 0）；出生树（点名项偏移 29）只判码 1、码 3，等于头偏移 43，码 2 随 C289（码 2 的出生树取哪个字段没写） 不判。校验和不相符的位置条目不判（落点被合法复用之后，单元头就不是它点名的那个单元了）。 | 已实现（2026-10-02，池级 checker `walk::judge_named_entries_against_the_unit_headers`：点名项每条位置条目，所指那一份整单元校验和相符才比，不符不判；出生树只判码 1、码 3；一条都没比到报不适用；类标签、出生树、写序实例代号、key 尾段各一份只红它的坏镜像，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 2 行） |
| I-8.12 | 同实例记录的 txg 不减 | 同一块盘 journal 环里自证过的记录，同一实例代号的记录按 jsn 计数器从小到大，`checkpoint_txg` 不减。换实例后严格大于不判（设备失而复得下没核）。 | 已实现（2026-10-02，池级 checker `walk::judge_journal_record_txg_per_instance`：逐盘、逐实例按计数器看 txg 不减，换实例不判，没有可比的一对时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行） |

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

**出处 `.claude/kb/invariants.md:125-135`（整段抄，未转述）**

```markdown
## I-2 校验和 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红） |
| I-2.2 | 校验和算法全盘一致 | 校验和算法标识在整个文件系统内一致。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：系统配置自举头段「校验和算法标识 1」，登记表 0 = 无效、1 = CRC-32C，第一版写 1；读到 0 或未登记的码一律拒绝挂载。算法的四个参数写死在 D18（块里携带什么信息） 已定项 17 | 未实现（条款已写、检查仍欠） |
| I-2.3 | 补齐字节为零且参与校验和 | 任一数据单元（码 1）或打包记录单元（码 3，D18（块里携带什么信息） 已定项 11），其声明长度（净荷字节数；补齐起点 = 头 + 声明长度）之后到单元末尾的补齐字节**恒为 0**，且校验和 / MAC 覆盖**整个单元含补齐**（[decisions.md](decisions.md) D4（校验和位置） 已定项 2：不足一个单元的 extent 补齐到 32 KiB；码 3 由头校验和（到头末尾）与载荷校验和（头之后到单元末尾）两段合起来覆盖整个单元）。补齐字节非零、或校验和只覆盖声明长度，一律判红。索引节点（码 2）从 2026-09-13 起同样在射程内（D18（块里携带什么信息） 已定项 18 用户定案）：声明长度 = 条目区字节数，之后到节点末尾的补齐字节恒为 0，载荷 CRC 覆盖 [86 + 2 × key 宽, 16384)——起点是明文头末尾，含 29 字节预留位、条目区与其后的补齐区，口径与码 1 的 105、码 3 的 107 同（预留位 2026-09-14 从 28 加到 29，含预留位的头是 115 + 2 × key 宽）。⚠️ **加密开启时判的是解密后的明文，不是盘上字节**（2026-09-07 用户定案，还清 C189（I-2.3 在加密开启时判明文还是盘上字节没定））：补齐在盘上是 `E(0)` 而不是 0，按盘上字节判必然全红；checker 本来就要解整单元，取值同一步就有。**与 D18（块里携带什么信息） 已定项 7 的载荷 CRC 方向相反是对的**——载荷 CRC 要给无密钥一侧用，所以算盘上字节；这一条只有 checker 用，所以判明文 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节与指针头部的 MAC 16 + nonce 12（D19（块指针的结构与宽度预算） 已定项 3）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。指针头部的 extent 偏移 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（D19（块指针的结构与宽度预算） 已定项 3；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。根记录里的指针偏移非 0 按这条根自证不过处置：core 退回上一条根（靠 journal 重放可以回到同一版，挂载照样成功），checker 红这一条而 core 挂载成功，两边各判各的、不矛盾（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 6 行）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs:674` 起）另判单元头偏移 4 的格式版本 = 1（walk.rs:722，不认得就判该结构损坏、不按今天的字段表往下解）与 29 字节预留位全 0（walk.rs:758）；走读跟随的每条指针在 `judge_pointer_mac_and_nonce_are_zero`（`crates/singlefs-checker/src/image.rs:699`）判头部 MAC 16 + nonce 12 全 0，四个入口分别是码 2 节点指针（walk.rs:815）、inode 内部条目子指针（walk.rs:1485）、实例表链指针（walk.rs:1811）、extent 数据指针（walk.rs:1961），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`）。**extent 偏移那一半已实现**（checker 的指针判定 `judge_pointer_mac_and_nonce_are_zero` 加这 2 字节，走读跟随的每条指针都判；core 每个读盘上指针头部的读者都判 extent 偏移非 0、判该结构损坏：树表条目的树根指针、inode 内部条目的子指针、extent 叶记录的数据指针、extent 上段叶条目的下段根指针与内联数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针各一处，加原有的 journal 新根段、码 2 内部条目、根记录三处（`crates/singlefs-core/src/pointer.rs` 的 `read_judging_the_extent_offset_from`；`grep -rn 'Pointer::read_from' crates/singlefs-core/src/` 只剩 `pointer.rs` 自己的单测一处）；影子账认单元的两个读者（`allocation_record_tree::node_pointers_as_far_as_readable`、`recovery::instance_table_page_pointers_as_far_as_readable`）今天也判，已定改回不判（影子账要多隔离、不漏隔离），改动在下一批；坏镜像遍历偏移 32、33，core 每类读者各一条用例在 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`；对应的账是 C581（extent 偏移非 0 没有判据）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行与 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行） |
| I-2.5 | 位置条目按设备身份升序 | 任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17，2026-09-13 用户定案）；两条位置条目设备身份相同或逆序，一律判损坏。⚠️ **射程（2026-09-14 补）：整条 86 字节全零的指针表示「注册了但还没有根」，整条豁免升序判定**——它内部那两条位置条目都是 (设备 0, 槽 0, 校验和 0)，按字面正是「设备身份相同」那一档。今天写出全零指针的有四处：树表里 livelist 树、稀疏旁表树与 deadlist 树那三条条目（D6（快照实现模型） 已定项 2、D5（快照 / 空间记账机制） 已定项 6 与 D5（快照 / 空间记账机制） 已定项 11 各自定案的「day-1 注册进树表，根指针为零」），以及 mkfs 与暖机的根记录里那个还没发布过的中央映射树根指针。非全零的指针不享这条豁免。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |

```

**出处 `.claude/kb/invariants.md:136-170`（整段抄，未转述）**

```markdown
## I-3 空间记账 —— 类

**依赖 D5（快照 / 空间记账机制）。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**2026-09-17 起「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」）：checker 自己算，取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与层 0 不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于 F_生效 的（还在 defer 里，抬 F 还不许回收），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15「实例表单元指针照所选根」，走它施加在其上那一版（环里同实例、txg 比它小的根里最大的一条，不管在不在候选集里）的实例表链与根记录持有的分配记录树（`walk::walk_version_applied_only_by_records`，主 agent 2026-09-27 定，被攻过零轮）。⚠️ **已知缺口**：环里同实例、txg 更小的根一条都读不出时，那一版的实例表与根记录持有的分配记录树走不到，照旧漏数；「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`research/prompts/m2-rev-b2-implementer-report.md` 第七节第 2 条）。 |
| I-3.2 | 已停用·方向反了 | **此编号不再使用**——它写成「≤ 引用它的**最新**快照的代号」，方向反了：紧的约束是最**旧**的那个。旧写法被 I-3.5（引用区间的精确性） 的子判据 ② 严格蕴含，且检不出任何一类真实错误 | 不适用 |
| I-3.3 | 已停用·归属未定义 | **此编号不再使用**——「所属快照」未定义，而两种归属约定给出不同的比较符。由 I-3.6（deadlist 紧界） 取代 | 不适用 |
| I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ①② 合成一条**出现规则**：取回退候选集（I-7.4（近 K 代块未被复用） 用的那一份，加由记录施加出来、根槽从没落盘的那几版）按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 为一对，逐对判。R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判，`Reach(b)` 在 txg 序上是否连续不再单判）满足 (a) `birth(b) > R⁻.txg`；或 (b) b 落在 R 树表里某棵用户可见树（今天 extent 树、inode 树，`deleted_inodes` 树根非全零之后加它）的子树里，那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg（读不出头的单元不计入）。`birth(b)` 取 b 的单元头里的诞生代号；b 自己的头读不出时这一个落点不判，读不出归 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配）。**缺口**：R.txg ≠ R⁻.txg + 1 时这一对之间有洞，分两种处置——最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判（gap-row）；不是的，用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 `birth(b) > R⁻.txg`。R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时整对报不适用；没有一对判到任何落点时整条报不适用并带理由（同 I-4.9（非最新候选根引用的单元不晚于它） 候选集只剩一条根的写法）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中。⚠️ **射程**：缺口的来源是新实例第一条根的 txg 取根环与 journal 里见过的最大 txg 加一（`crates/singlefs-core/src/mount.rs` 取号那一步），所以写行那次发布的记录已落而根槽没落、多记录发布只落一部分之后再可写挂载，必出现缺口；另有恢复丢掉尾巴记录、根槽写失败重发跳一格、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）。**盲区**：缺口右边那一对里，落在用户可见树子树里的新写单元（新叶）的诞生代号被改小到 R⁻.txg 以下，判不出；整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小到 R⁻.txg 以下的坏镜像同样判不出：盘上与一次合法回退逐项同形，判别子是回退那次发布写的谱系记录（D18（块里携带什么信息） 已定项 24），`crates/` 今天没有。**谱系记录落地之后**，(b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时该对不适用」，这一形与缺口条款并存（`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 9 行）。**判别力**：诞生代号改小到恰好等于 R⁻.txg 的新写单元必须红；最新根的 inode 树指回已丢的旧叶、分配记录与映射条目一起改平必须红；回退根槽被环盖掉、记录也盖掉之后的合法镜像不许红。**推翻条件**：任一合法镜像（池级 checker 与层 0 其余判定都不红）上红。 | 未实现（实现进下一批）：出现规则取按树继承下界加 β 出现规则（用户 2026-10-02 定 ① 取 β）加缺口处置；缺口处置用户 2026-10-03 定 gap 加 N3（判决里的局部标号，只放过用户可见树里的单元）加 gap-row 两样都加。**被攻过几轮**：按树继承下界加 β 加整对不适用的缺口，在合法镜像上两轮没红（各一次抽样）；gap 加 N3 加 gap-row 的合成形、同 txg 几条并成一条、`deleted_inodes` 非全零才算继承、读不出头的单元不计入这三处措辞、谱系记录落地之后的区间形，都是零轮，终轮不再开轮，实现时再攻。实现挂在池级 checker 的主走读上（每版落点集合快照、用户可见树标签、压得住的判、新判定函数、位置键折叠、`IMPLEMENTED_INVARIANTS` 加一条、用例与变异），I-3.1（已分配统计对得上） 在短环上洞里的误红归下一批调查，与这一条共用同一个「候选集有洞」判法（`research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 4、5 行与第四节）。② 的上界半（`birth(b) ≤ R_first.txg`）另立 I-4.9（非最新候选根引用的单元不晚于它）（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行）。⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
| I-3.6 | deadlist 紧界 | 任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）。判别力：抓「kill 时该走直接释放却进了 deadlist」这一类（泄漏方向） | 未实现 |
| I-3.4 | 可用空间扣待删占用 | 待删中 inode 没释放的单元计在记账「已分配」里（释放只进 defer），记账标签 4（待删除但意图未完成占用）恒 0，可用空间统计因此已扣除待删占用；准入式子另扣环窗口常驻项（D28（挂载期承诺量） 已定项 1）。来源 `research/prompts/m3-delete-r3-main-verification.md` 第三节判标签 4 那一格，被攻过零轮 | 未实现 |
| I-3.7 | 活快照可枚举 | 盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少） | 未实现 |
| I-3.8 | 实例表行唯一且低于挂载根 | 实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）；行的实例代号 < 挂载根实例（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2） | 已实现（2026-09-16，`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_rows`：行按实例代号唯一、每行实例代号低于最新根的实例、链指针记录在末尾；回收那一半没有输入，第一版行只增不删，删行的条件没人判） |
| I-3.9 | 释放代落在停止引用它的那一格区间里 | 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。若每一条有效根都还引用它，它不许带已释放标志（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））。环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条层 0 流的 12 个状态）按区间判，按「等于」判会在合法镜像上判红；已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。判别力：把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判）。**判法收窄在实现途中**：今天任一候选根走不全就整条报不适用；收窄成只在比最后引用它的那条根晚的候选根都走得全时才判这一条已释放记录，攻方造出的对照格镜像（抬 F 的空发布之后再覆盖写，把最后一次空发布那条根的树表翻坏）钉进用例（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 10 行） |
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：与最新根同实例、checkpoint_txg = 最新根的 txg + 1、带提交标记、根环里没有它的根的那条记录，新根段里树表指着的分配记录树（主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定，被攻过零轮；只接这一步——恢复连着施加几条记录时第二条及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
| I-3.11 | 已分配减 defer 等于最新根走读 | 逐盘：被判镜像上记账树里「已分配」那一行（记账第 1 项）减「defer 待释放」那一行（第 5 项），等于从最新有效根走读到的、这块盘上被引用的槽数。统计量的标签与行形态是 D5（快照 / 空间记账机制） 已定项 4 与已定项 10 的那几行，读的是镜像里的记账行、不读内存里的分配器；走读与 I-3.1（已分配统计对得上） 同一个走法，只取最新有效根。⚠️ 它补的是 I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 都够不着的那一种：写者把活单元记成已释放、放进 defer，已分配与空闲两个和照样对得上（alloc-basis 第三轮判决 `research/prompts/alloc-basis-r3-main-verification.md` 的 Z10）。⚠️ **判别力自证在造出来的基底上做**：E156（alloc-basis 四条岔路的代价数） 第 4 次重跑在造出来的基底 β_syn（第 5 项为 0）上再记 1 槽 defer，I-3.11（已分配减 defer 等于最新根走读） 判红、去掉「减 defer」判绿（`research/results/e156-alloc-basis-counts-2026-09-26-r2.out` 的 `E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`）。可达状态里第 5 项能不能为 0 没量到：挂载时回退（回退写行那次发布）今天没有了，同一份产物里挂着回退那一版第 5 项是 356 槽（`E7RESULT name=u13a_rollback_to_oldest_candidate overwrites=72 target_txg=52 txg_before=75 txg_after=76 instance_before=1 instance_after=1 instance_table_rows_before=0 instance_table_rows_after=0 item1_slots=374 item5_slots=356 i311=holds structure_holds=true`）。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 | 已实现（池级 checker `walk::check_pool_image`：逐盘判「defer 待释放 + 最新根走读 == 已分配」，最新根走读取 `walk_root(最新根)` 刚走完那一刻、与 I-3.1（已分配统计对得上） 同一个走法；最新根下面没有记账树时报不适用。坏镜像 `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`（盘 0 defer 行 +1 / −1 槽，只红它，I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧真被评估过且成立）；判别力自证在造出来的基底上 `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`（两盘 defer 挪回 0 之后再记一槽）；`crates/mutations.tsv` 5 条变异（判定恒真、不减第 5 项、`==` 放宽成 `≥` / `≤`、拿全部候选版本那一份走读比）；用户 2026-09-23 定采纳：alloc-basis 岔路单第 12 行（岔路 7）） |
| I-3.12 | 实例表行 T 低于引用它的根的 txg | 根环里任一自证过的根 R，它指着的实例表的每一行 (i, T, W) 满足 T < R 的 `checkpoint_txg`；链上某一片读不出的根不判。依据 D18（块里携带什么信息） 已定项 11（恢复写的行，所选根那个实例的 T 是重放之后那个根的 txg，中间实例写 0）与 D23（journal 的角色与格式） 已定项 14（新实例第一次发布的 `checkpoint_txg` 大于环里全部根与记录的号）。射程是每条根各自指着的那张表，不是只有挂载根那一张（那一张归 I-3.8（实例表行唯一且低于挂载根））；同一实例的行在两张表里是否逐字节相同不判，条款没有这一句。 | 已实现（2026-10-02，池级 checker `walk::judge_instance_table_rows_against_the_roots_referencing_them`：每条自证过的根各沿自己的实例表链判，链上有一片读不出的根跳过，一行都没判到时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 6 行） |
| I-3.13 | 记账行代段等于这一版的 txg | 走读到的任一版（候选根，以及由记录施加出来的那几版）的记账树里，每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 `checkpoint_txg`，只判走全了的树。依据 D5（快照 / 空间记账机制） 与 D16（发布语义） 已定项 6 的「记账代 = checkpoint 号」（每条根的记账树只装这条根那一代的行）。 | 已实现（2026-10-02，池级 checker 走读里的 `walk_tree_table_entry` 对每一版的记账树判，版本含由记录施加出来的，只判走全了的树；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |

> ⚠️ **I-3.5（引用区间的精确性）四条子判据双侧夹死 与 I-3.1（已分配统计对得上） 正交，两条都必须有。** I-3.1（已分配统计对得上） 是标量总账（已分配 == 遍历和），
> 而**归属错误对它不可见**——块被挂到错的 deadlist 上时，它仍然被恰好一处引用，总账依然平。
> 要检出归属错，必须有**按快照分箱的向量等式**。
> 这与 [decisions.md](decisions.md) D14（双轨（大小文件 / 持久临时）） 已定项 6 推导 D-3（多轨兼容可判据） 里「标量等式 → 按轨道分箱的向量等式，
> 判别力净增」是同一个形状。
>
> ⚠️ **数据丢失方向只在空间被重新分配之前可检出。** 该进 deadlist 却被释放的块，
> 一旦其空间被复用，症状就退化成 I-2.1（校验和与内容匹配） 校验和不匹配，即「损坏，原因不明」。
> **所以这类检查必须在崩溃点重放 harness 里紧跟着删除动作跑，不能只在全量里跑。**
>
> ⚠️ I-3.1（已分配统计对得上） 是**唯一允许全盘遍历的记账检查**——它是 checker，不是运行时路径。
> 运行时记账必须增量维护（D5（快照 / 空间记账机制）），checker 才用遍历来验证增量维护有没有出错。
> 这条对照关系是本工程验证记账正确性的核心手段。

```

**出处 `.claude/kb/invariants.md:171-189`（整段抄，未转述）**

```markdown
## I-4 事务与崩溃一致性 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-4.1 | 重放后树可遍历 | 任一崩溃点重放后，所选根（重放之后那一版）指向的树全部可完整遍历。起点含介质损坏的镜像（前缀停在一条点名单元两份没都验过的记录上）报不适用。另三处报不适用：所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版）；所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布取不到 txg）；所选根之上一次发布都不施加（重放之后那一版就是所选根，可不可遍历归 I-7.2（最新根可完整遍历））——没有记录压在最新根之上的镜像上这一条因此恒报不适用、不报成立。重放用的读环按 (实例代号, 计数器) 建键，两块盘同一计数器坐着不同实例的记录时两条都留，与实现恢复同键（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2、8 行）。 | 已实现（2026-10-02，池级 checker `walk::judge_trees_of_the_replayed_version_are_traversable`：R-独，checker 自己照 D23（journal 的角色与格式） 已定项 14 的前缀规则复刻只读重放（`replay_the_next_mount_would_do`），施加出来的那一版（所选根换上末条记录的新根段四段，实例表指针与分配记录树根照所选根）另起一份走读，走读断了判红；在飞上限按环长 ÷ 4096 ÷ `JOURNAL_SAFETY_FACTOR` 现算。读环键 2026-10-03 改成 (实例代号, 计数器)（`records_the_replay_reads`），反向链那一关改按 (实例代号, 计数器 − 1) 找前一条（`record_back_chain_lets_it_into_the_replay_prefix`），消掉两块盘同一计数器坐着不同实例的记录时与实现分开的形态（推为第一版两盘池上不可达）；`records_as_the_recovery_reads_them` 与 `record_can_enter_a_replay_prefix` 两处同改按 (实例代号, 计数器) 建键，实现进下一批（主 agent 2026-10-03 定）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（重放六条规则各一份、两块盘同计数器不同实例两份）；层 0 里与 core 的 `effective_root` 逐状态差分今天没有能跑的入口，要先开口子，进下一批清单（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8、9 行）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 2 行与第四节用户定案） |
| I-4.2 | 无被引用未提交块 | 任一崩溃点重放后，不存在被引用但未提交的块 | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`：走读到的每个被引用单元，头里的诞生代号与写序（D18（块里携带什么信息） 已定项 7 单元头字段表里的字段）按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判成已发布——**「未提交」取的就是「按那个谓词判不成已发布」**，谓词与 I-1.8（归并后版本全序） 扫描方向共用同一份 `PublishedPredicate`。打包记录类型 4 的实例表单元按 I-1.2（块头写序已发布） 那一行的例外不走谓词，不进这一条。坏镜像是把数据单元头里的诞生代号与引用它的 extent 记录里那条数据指针的诞生代号一起抬到最新根的 txg 之上——指针仍与头一致 ⇒ I-1.2（块头写序已发布） 不跟着红，分得开两条的就是这一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它。⚠️ **「任一崩溃点重放后」那半句没实现**：池级 checker 判的是手上这一份镜像、不跑恢复，判的是「被引用 ⇒ 已发布」这个态上的性质；重放之后再判一遍归层 0） |
| I-4.3 | 提交原子 | 提交是原子的：不存在"部分生效"的事务 | 未实现（它要判的是 D23（journal 的角色与格式） 已定项 7 那两个事务边界字段——事务号与提交标记——施加之后的效果。**对着一份镜像判不出来，没硬写**：「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽——同一份字节既可能是「事务整体没生效的旧态」，也可能是「生效了一半」，池级 checker 手上没有第二个输入。判它的是层 0 崩溃点重放加 oracle：`.claude/kb/milestone/02-second-txn.md` 第 486 / 487 行那两条（崩在第二条记录之前 ⇒ 整个事务不施加、文件是旧长度；把提交标记提前到第一条记录 ⇒ 恢复施加了半个事务、I-4.3（提交原子） 红）说的就是这条路。形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查）：操作断言，不进池级 checker，恒真的检查比没有检查更糟） |
| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 未实现 |
| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；判据落在 [experiments.md](experiments.md) E3（意图日志的断电语义） 判据③ | 不适用 |
| I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致（候选集的深度由 D16（发布语义） 已定项 1 的回退下界 F 定：平时整个根环，盘紧时缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内）。**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，最新根与候选集里每条更早的根各判一格：从它出发的遍历有单元对不上或读不出就红；坏镜像是 mkfs 树表单元抹成零；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态） | 未实现 |
| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在 | 未实现 |
| I-4.9 | 非最新候选根引用的单元不晚于它 | 回退候选集里不是最新的每一条根 R，它引用的单元（按落点认，读落点上单元头里的诞生代号，头校验和过）的诞生代号 ≤ R 的 `checkpoint_txg`。最新那条根归 I-4.2（无被引用未提交块），不在这一条里。依据 D16（发布语义） 已定项 7（单元先于根落盘）。 | 已实现（2026-10-02，池级 checker `walk::judge_unit_birth_against_an_older_candidate_root` 判走读里每个被引用的单元，共享子树里别的版本先走过的单元由 `merge_placements_below_a_unit_walked_by_an_earlier_version` 补判；候选集只剩一条根时报不适用，同 I-3.9（释放代落在停止引用它的那一格区间里）；坏镜像沿 I-4.2（无被引用未提交块） 那一份的造法，映射 key 也要跟着改，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行） |

> ⚠️ **编号只往上加，不回收、不重编。** 回收一个编号会让别处的旧引用悄悄指向另一条陈述，
> 而检索只端出其中一条，不会告诉你它换了意思（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条「矛盾比空白更糟」）。
> 退役的编号留一行说明去向，比留一个无解释的空档安全。

```

**出处 `.claude/kb/invariants.md:190-212`（整段抄，未转述）**

```markdown
## I-5 分配器 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-5.1 | 物理范围不重叠 | 不存在被两个不同引用同时占用的物理范围（除非是显式的多副本） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 引用按版判（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 1 条、`research/prompts/m2-rev-b2-implementer-report.md` 第八节）：同一版（根环里的一条根，或由记录施加出来的一版）里同一个 (设备, 槽) 被两条指针引用判红，(设备, 槽) 取自位置条目的设备 ID 与物理偏移（D19（块指针的结构与宽度预算） 已定项 4），一条指针自己的两条位置条目同落点算一个引用（那一格归 I-2.5（位置条目按设备身份升序））（`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`）；中央映射条目指的物理范围算进引用集合；别的版本走过的共享子树不重走，按缓存把它下面的落点并进这一版再比、不读盘（`walk::Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`）。坏镜像在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`。 |
| I-5.2 | 空闲统计对得上 | 空闲空间统计 == 总空间 − 已分配空间 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-5.3 | 报出的空闲都兑现得了 | `df` 报给用户的空闲 ≤ 当下可分配 + 能在有界步数内放回的扣住量（defer 窗口里、回退下界以下的释放）。运行时的等价形态：`df` 报出 s 字节空闲时写 s 字节必须成功；删掉 s 字节之后同样大小的写在有界步数内成功（[decisions.md](decisions.md) D3（空间分配） 已定项 9，用户定案）。界随 D16（发布语义） 已定项 1 的形态定。判别力：把「放回扣住量」那一步摘掉，近满盘上删一个对象再写同样大小必须由绿转红 | 未实现 ⚠️ **口径注（2026-09-13，D28（挂载期承诺量） 已定项 4）**：`df` 要扣的 checkpoint 保留池是每次发布现算的量，发布即重算，近满盘上报出的空闲会随树高变小。 |
| I-5.4 | 分配记录罩住的槽互不相交 | 任何一条有效根的分配记录树里，同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交。复用一个已回收的落点时，新记录罩住的槽上别的已回收记录要随这次分配一起删掉；留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_allocation_records_disjoint`，按回退候选集里每条根判；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant`；层 0 每个崩溃状态都判，同日在新代码上全量：第二条流 2,104,413 个状态里评估 1,842,252 个、违例 0，其余 262,161 个报不适用，第一条流 262,165 个状态里评估 4 个、违例 0（`m2-wave2-crash-verifier-report.md`「主 agent 要的两个数」）。同日立，依据 `research/prompts/m2-wave2-code-r1-main-verification.md` 第二节第 1 行：复用时跨度变了留下重叠记录，下一次可写挂载在重建分配器时 panic，而 I-5.1（物理范围不重叠） 只管树里的引用、判不出记录之间的重叠） |
| I-5.5 | 从 mkfs 水位建树那一版的 t1 落点 | 走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）若是从 mkfs 水位第一次建树的那一版——① V 的树表非空，且种类 1（extent 树）条目的树 ID 等于 65536；② V 的树表里每条条目的诞生 txg（条目偏移 108）都等于 V 的 `checkpoint_txg`——则 V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号等于 U + 4，U 是所选系统配置偏移 417 的单元区起始槽号（默认 768 MiB 环 U = 50176，所以是 50180）。V 没有数据单元时不判；不满足 ①② 的版本不判。这一类含新池新建文件，也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件（用户 2026-10-04 定案，原话在决策变更史 D3（空间分配） 节）。只判 t1，t2–t12 不判（D3（空间分配） 已定项 10 射程没有「偏离即不合法」）。依据 D3（空间分配） 已定项 8 射程与已定项 10 ③ ④。 | 未实现（C673（三 oracle 合谋测试的补 oracle 改法没做） ⑥） |

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

**出处 `.claude/kb/invariants.md:213-262`（整段抄，未转述）**

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

**出处 `.claude/kb/invariants.md:263-277`（整段抄，未转述）**

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

**出处 `.claude/kb/invariants.md:278-318`（整段抄，未转述）**

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
| I-9.5 | 删除队列里的号仍在树里 | `deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0（记录在回收完成前留在 inode 树里，`deleted_inodes` 只存号） | 未实现（三·3 实现；`deleted_inodes` 的形态已定在 D8（核心索引结构） 已定项 16） |
| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID（墓碑记录住墓碑树，D18（块里携带什么信息） 已定项 24；被崩溃恢复抛弃的时间线上写的墓碑不在所选根的墓碑树里）——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**判的只有「水位 > 树内最大 key」这一半**：墓碑那一半（水位 > 墓碑树里全部记录的对象 ID 最大值）未实现，三·3 实现 |
| I-9.7 | 记录三段恒零 | 打包记录类型 2 的记录偏移 108 / 112 / 120 三段（填充 4 / flags 8 / 预留 20）恒 0；非 0 判该记录 EIO，容器其余记录照读（前向兼容政策，D15（格式冻结政策） 判定表；不是损坏检出，载荷校验和照样过） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.8 | 记录 inode 等于 key | 打包记录类型 2 的记录偏移 0 的 inode 号 == 它在 inode 树里的 key；不等判红、该记录 EIO，不许静默修复 | 未实现，前置：树的种类 |
| I-9.9 | locality 三值对照 | 记录的 `locality_id` == 该对象每条 extent key 的首段；extent 树未经重建时输出红 / 绿；系统配置 `map_provenance` 标 rebuilt 时两侧同源，输出「不可判定」，不许输出「通过」；按侧判（2026-09-16）：记录那一侧是从 key 编出来的、或 key 那一侧是从记录编出来的，同样输出「不可判定」；分批重新聚簇的中间态怎么判随 C346（后台重新聚簇没有定义） 定 | 未实现 |
| I-9.10 | 对象出生代三处一致 | 记录偏移 8 的对象出生代 == 该对象每个数据单元头五元组里的对象出生代 == 该对象的墓碑记录里的对象出生代（若有；墓碑记录住墓碑树，D18（块里携带什么信息） 已定项 24；只对 inode 记录还在的对象判，删完之后记录不在，这一个等号无对象）；重建后两侧同源，输出「不可判定」，与 I-9.9（locality 三值对照） 同形 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；墓碑那一个等号未实现，三·3 实现） |
| I-9.11 | 可写克隆头有水位行 | 每个可写克隆头的树 ID 在记账里都有「inode 号水位」行（树表可枚举、记账可查）；「初值 ≥ origin 那一刻的水位」是建克隆那一刻的操作断言，转模型对拍——它的右操作数在 K 次发布后被 D5（快照 / 空间记账机制） 已定项 2 的点删丢掉，对着镜像读不到 | 未实现，前置：树的种类 |
| I-9.12 | 分隔 key 落在孩子区间之外 | inode 树内部节点的条目按分隔 key 严格递增；第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界。判的字段是内部节点条目头 8 字节那个分隔 key（D8（核心索引结构） 已定项 6：条目 120 字节 = 分隔 key 8 + 身份引用 26 + 子指针 86，「分隔 key = 该孩子创建时的最小 key，条目按分隔 key 单调递增」）；孩子的 key 区间对码 3 容器由它装的记录现算，容器头不带 key 区间 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.13 | 类型 2 容器不空 | 任一打包记录类型 2 的容器记录数 ≥ 1（空叶在重建时没有 key 范围；唯一的叶被删空时不落盘、根变成零条目） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.14 | 树表条目的诞生 txg 跨根不变 | 同一棵树（按树 ID）的树表条目，在**同一条时间线上**的任意两条有效根各自的树表里，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段）；C374（释放代与树表诞生 txg 只有验收断言盯着））。「同一条时间线上的有效根」就是回退候选集（D23（journal 的角色与格式） 已定项 14 + D16（发布语义） 已定项 1）：最新那条根本身；以及根环里 (i, T) 满足两条的根——按**最新那条根指着的实例表**，没有实例 i 的行、或有行 (i, Ti, Wi) 且 T ≤ Ti；并且 T ≥ 最新根带的回退下界 F。有行 (i, Ti, Wi) 且 T > Ti 的根在被崩溃恢复切掉的旧线上，**不与现行线上的根比**（C511（回退到无文件那一版之后诞生代怎么接） 2026-09-23 用户定案收窄射程）。候选集不足两条根，或同一条时间线上没有一棵树的条目出现在两个不同的树表单元里时，报「不适用」，不报成立——几条根指着同一个树表单元、拿同一份字节比自己是恒真判定。判别力：① 把树表条目的诞生 txg 改成跟着这次发布的 txg 走，必须红；② 管理员回退前后同一条线上记得不同，必须红（回退是向前发布，前后是同一条线）；③ 只有被切掉的那条线记得不同，不许红。推翻条件：找到一条历史，最新根的实例表把某条根判成有效，而它不是最新根的祖先（不在同一条线上）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判。2026-09-23 收窄之后同文件两条：判别力 ② 的坏镜像 `birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant`（挂着时回退到 A 之后，回退那次发布 D 的树表把 extent 树（树 65536）的诞生 txg 记成 D 的 txg 9、A 那一版记 3：A 与 D 在同一条线上，只有 I-9.14（树表条目的诞生 txg 跨根不变） 红），判别力 ③ 的正向用例 `birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_is_not_compared_and_every_invariant_holds`（被切掉的线由崩溃恢复造：C 被抛弃之后只改 C 那片树表里的诞生 txg，每一条都成立）；判定在 `walk::judge_tree_table_birth_txg`） |
| I-9.15 | inode 记录的 blocks 等于 ⌈size ÷ 512⌉ | 打包记录类型 2 的每条记录，偏移 48 的 blocks == ⌈偏移 40 的 size ÷ 512⌉（D8（核心索引结构） 已定项 6：blocks 是逻辑长度的 512 字节块数，不表示分到的空间；C480（inode 记录的 blocks 怎么算全仓没有条款） 用户 2026-09-23 定）；不等判红。读者要不要也判、判了报什么没有条款 | 已实现（2026-09-24，池级 checker `walk::check_pool_image` 走 inode 叶容器时逐条判，与 I-9.7（记录三段恒零） 同一处；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：blocks 写成 64 / 5 / 7 各只红它；层 0 每个走得到 inode 叶的崩溃状态都判；变异：判定恒真、按向下取整比、写路径写回 64；里程碑「覆盖写、释放、回退与复用」收口表第 46 行） |
| I-9.16 | 树表条目按树 ID 严格升序且合发号次序 | 同一张树表（D8（核心索引结构） 已定项 8 的字段表）里，① 盘上相邻两条的树 ID 严格升序；② 树表里有的种类按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist；中央映射树不进树表）排，相邻两棵的树 ID 严格升序。两道任一道不成立即违例。依据 D8（核心索引结构） 已定项 8「排序契约」与它的 ②「十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547」。判别力落在 ②：① 与 checker 在树表上判的 I-1.1（块头自述逻辑地址） 那组 key 严格递增是同一个谓词，只违反 ① 的镜像两条一起红 | 已实现（2026-09-27，core `crates/singlefs-core/src/recovery.rs` 常量 `TREE_TABLE_ENTRIES_ORDERING_CONTRACT`，两道合成一个入口，`rebuild_version` 与 `walk_to_file` 在读任何一棵树之前拒；池级 checker `crates/singlefs-checker/src/walk.rs` 另写一份、违例写明哪一版根与哪两条，列进 `image.rs` 的 `IMPLEMENTED_INVARIANTS`；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`「互换两条的种类」那份只红 I-9.16（树表条目按树 ID 严格升序且合发号次序）；只违反 ① 的钉在 `tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`；实现员报告 `research/prompts/m2-rev-tree-table-ordering-implementer-report.md`） |
| I-9.17 | 谱系树条目身份与子头相符 | 谱系树（D18（块里携带什么信息） 已定项 24）的每条条目：类型段 ∈ {0, 5}；类型段 0 ⇒ 子单元是码 2（谱系树根满长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；类型段 5 ⇒ 子单元偏移 6 = 3，子头的 (出生树, 打包记录类型, 容器号, 容器出生代) 与条目的身份引用四段逐字相等，子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 5；其余判该条目损坏。与 I-9.2（条目身份与子头相符） 同形 | 未实现（三·3 实现） |
| I-9.18 | 删除队列里的号有谱系上的墓碑 | `deleted_inodes`（D8（核心索引结构） 已定项 16）里的每个号 N，墓碑树（D18（块里携带什么信息） 已定项 24）里都有一条在谱系上的墓碑记录 (N, N 的 inode 记录偏移 8 的对象出生代, [0, 2⁶⁴ − 1])。「在谱系上」：墓碑记录的死亡 txg 落在挂载根的父链上——从挂载根的 txg 起往回走，有谱系记录 (T_rb, T_old) 的那个 txg（T_rb）的父是 T_old，其余 txg 的父是 txg − 1（谱系记录住谱系树，D18（块里携带什么信息） 已定项 24；txg − 1 那一步核心层没有显式断言，C616（谱系父链靠 txg 连号没有断言））。不按谱系过滤的写法放过改坏（`research/prompts/m3-delete-r3-main-verification.md` 第三节判新不变量写法那一格） | 未实现（三·3 实现） |
| I-9.19 | 谱系上的墓碑不指活对象 | 墓碑树里每条在谱系上的墓碑记录（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）指认的 (号, 对象出生代)，inode 树里有号与对象出生代都相同的记录时，那条记录的 nlink = 0 | 未实现（三·3 实现） |
| I-9.20 | 谱系上的墓碑不重号且记录合法 | 墓碑树里在谱系上的墓碑记录（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）按 (号, 对象出生代) 不重复；每条记录区间起 < 区间止，第一版只许整对象 [0, 2⁶⁴ − 1]；每个打包记录类型 1 容器的记录数 ≤ 815（⌊(32768 − 136) ÷ 40⌋）。不按谱系过滤的写法在「删 → 回退 → 再删」的合法历史上判红（`research/prompts/m3-delete-r3-main-verification.md` 第三节判新不变量写法那一格） | 未实现（三·3 实现） |
| I-9.21 | 产品镜像里非最右墓碑容器装满 | 没有用只供测试的容量旋钮的镜像上，墓碑树里容器号沿叶序严格递增（根长高之后根的条目是类型段 0、容器号恒 0，见 I-9.23（墓碑树条目身份与子头相符），所以不沿根的条目读），除最右那一个之外的每个打包记录类型 1 容器记录数 == 815（开放容器满了才关，D18（块里携带什么信息） 已定项 24） | 未实现（三·3 实现） |
| I-9.22 | 测试镜像里墓碑容器不超旋钮值 | 只供测试的容量旋钮压小了一个墓碑容器装的记录数的镜像上，墓碑树里容器号沿叶序严格递增，每个打包记录类型 1 容器记录数 ≤ 旋钮值；旋钮不落盘，checker 从镜像外拿旋钮值。按产品容量判这类镜像会在合法状态上红（`research/prompts/m3-delete-r3-main-verification.md` 第三节判容量旋钮那一格） | 未实现（三·3 实现） |
| I-9.23 | 墓碑树条目身份与子头相符 | 墓碑树（D18（块里携带什么信息） 已定项 24）每个内部条目的类型段 ∈ {0, 1}：0 ⇒ 子单元是码 2（根长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；1 ⇒ 子单元是码 3，子头 (出生树, 打包记录类型, 容器号, 容器出生代) 与身份引用四段逐字相等、子指针头部出生树 == 身份引用出生树、子容器打包记录类型 == 1。与 I-9.2（条目身份与子头相符） 同形（墓碑正推报告 `research/prompts/m3-tomb-r1-sonnet-output.md` 第 241 行 N1；根长高之后类型段 0 那一支是主 agent 按 K-R3-3 补的，被攻过零轮） | 未实现（三·3 实现） |
| I-9.24 | 删除队列至多一条 key | 任何一版里 `deleted_inodes`（D8（核心索引结构） 已定项 16）至多一条 key；每个被判的版本各判一次，不遍历版本之间的历史。依据：会话里每一次用户改动先把现行那一版的队列走空再做自己（D8（核心索引结构） 已定项 16 第 6 条的增补，用户定案），判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节「B4 + B8 队列上界与接口」一行 | 未实现（三·3 实现） |
| I-9.25 | 树表条目的 previous_snapshot_txg 取值 | 走读到的任一根（候选集里每条，含由记录施加出来的那几版）的树表里：树的种类不是 inode 树（种类码 ≠ 2）的条目，偏移 100 的 `previous_snapshot_txg` 恒 0；inode 树的条目为 0，或 1 ≤ p ≤ 这条根的 `checkpoint_txg`。依据 D5（快照 / 空间记账机制） 已定项 9（`previous_snapshot_txg` 只写在该头 inode 树那条树表条目里，其余条目写 0 且 0 = 不适用）；inode 树条目「≤ 根 txg」那一半的条款是用户补定的（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节用户定案第 2 条）。 | 已实现（2026-10-02，池级 checker `walk_tree_table_entry` 每走到一条树表条目判一次，每个走到的版本（含记录施加出来的）都走它；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，inode 树条目的 previous_snapshot_txg 压过根 txg 那一份由 2026-10-03 补；⚠️ 树表单元被几版共享时只在第一个走到它的那一版判（`visited_units`），共享它的别的候选根漏判，与条款字面「候选集里每条」不符，代码注释自认判得更松，代码改回 kb 字面，实现进下一批（主 agent 2026-10-03 定；`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2 行，推的，没造镜像）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |
| I-9.26 | inode 改动计数落在出生代与容器诞生代号之间 | 走读到的任一 inode 叶容器（码 3，打包记录类型 2）里每条记录：对象出生代（记录偏移 8）≤ 改动计数（记录偏移 88）≤ 这片容器头的诞生代号（偏移 73）。上界不是等于：容器因别的记录改动而重写时，没改的那条记录的改动计数小于容器诞生代号是合法的；下界要「创建算一次改动」，这一句条款是用户补定的。依据 D8（核心索引结构） 已定项 6 偏移 88 那一行（改动计数 = 最后一次改动所在发布的 `checkpoint_txg`）与 D18（块里携带什么信息） 已定项 9 之后的容器重写规则。 | 已实现（2026-10-02，池级 checker `walk::walk_inode_leaf_container`：每条记录判偏移 8、偏移 88 与容器头里偏移 73 的诞生代号（不是偏移 61 的容器出生代）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，下界一份（改动计数 = 对象出生代 − 1）由 2026-10-03 补；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行与第四节用户定案第 2 条） |
| I-9.27 | 树表条目的头 ID 取值 | 走读到的任一版（候选集里每条根，含由记录施加出来的那几版）的树表有条目时，逐条判条目偏移 116 的头 ID：种类码 2（inode 树）的条目等于它自己的树 ID（条目偏移 0）；种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）的条目等于这张树表里种类 2 条目的树 ID，表里有多于一条种类 2 条目时放宽成「等于其中某一条的树 ID」（收严留给多头那一批），表里没有种类 2 条目时这三种判红；种类 3、4、6、7、8、11、12 的条目等于 0；别的种类码不判。树表 0 条的版本一条也不判。不拿字面 65537 比：它只是树 ID 从 mkfs 水位 65536 起发的那一版上的取值。依据 D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」与 D5（快照 / 空间记账机制） 已定项 9。 | 未实现（C673（三 oracle 合谋测试的补 oracle 改法没做） ⑤） |

**不进 I-9（inode 树结构） 的**：分裂 / 合并的身份传递（左半身份不变、右半号 = 触发分裂的新记录号、出生树 = 执行分裂的树）与合并时机是操作断言，对着一个镜像判不了，走单测 / 模型对拍与崩溃点重放（C116（叶容器的分裂 / 合并纪律没有会失败的检查））；
nlink == dirent 条数的门（dirent 树存在）今天没有输入，写不成可判定形式，随 dirent 树上线时一起立。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:1-4`（整段抄，未转述）**

```markdown
## D8 核心索引结构 —— 已定

D8（核心索引结构） 管核心索引长什么样：一套 btree 实现配多个独立 keyspace 与两个前端、节点大小与它的上限判据、key 布局、whiteout 删除与无界操作的分批、inode 树的形态与 inode 记录的盘上字段表、write buffer 条目的 `seq`、树表条目的字段表与树的种类码登记表、码 2 索引节点的载荷布局与 key 比较序、前端与节点消息缓冲区的衔接、同一次发布里同一个 key 的状态折叠。**不预设 B-tree**，调查见 [prior-art.md](../prior-art.md) 第五节。不管的：记账统计量的载体、key 与合并规则在 D5（快照 / 空间记账机制）；多可写头下 key 要不要加一维快照、快照的实现模型在 D6（快照实现模型）；消息缓冲区的 ε 布局轴在 D11（索引节点要不要留消息缓冲区）；单元头、单元类型码与打包记录单元的装载纪律在 D18（块里携带什么信息）；指针的字段与宽度在 D19（块指针的结构与宽度预算）；发布语义与 checkpoint 边界在 D16（发布语义）；格式冻结的层次在 D15（格式冻结政策）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:28-40`（整段抄，未转述）**

```markdown
#### 已定项 1：记账取幂等完整值，接受 key 数量上升

**定案**：**记账条目一律是「某个统计量在某一代的完整值」**，写入时整格覆盖；**不用增量 Δ**。合并规则按统计量分（D5（快照 / 空间记账机制） 已定项 3 第 2 条、登记表见 D5（快照 / 空间记账机制） 已定项 10）：非单调统计量「后者胜」（后者由 `seq` 定），单调统计量取最大值。取最大值同样幂等、可交换、可结合，本项的论证（幂等完整值胜过增量 Δ）不受影响。付出的是 key 数量上升（每个统计量 × 每一代各占一条）。

**射程**：它定的是记账条目 value 的语义；载体（记账住一棵独立 keyspace 的 btree、条目是「(统计量, 维度元组, 代) → 完整值」）在 D5（快照 / 空间记账机制）。⚠️ **key 数量上升的量还没测**：「接受」是对方向的定案，量出来之前不许拿它去推「记账占多少空间」。⚠️ **兜底**：若某一类账确实做不到幂等（本项不预判有没有这一类），那一类必须另有一条与运行时**不共享判定代码**的独立重算路径。⚠️ 它同时改变 D11（索引节点要不要留消息缓冲区） 代价 1 的成立条件——该代价的判据是「缓冲里放的是幂等完整值还是增量 Δ」，本项把答案钉在幂等那一侧 ⇒ **代价 1 在本工程不成立**；D11（索引节点要不要留消息缓冲区） 的代价 2（可重建性退化）不受影响。

**依据**：

- E14（I-3.1 的判别力：幂等完整值 vs 增量 Δ）：同一棵树同一负载注入四类语义 bug，幂等完整值形态下 I-3.1（已分配统计对得上） 全抓、零盲区，增量 Δ 形态下一条都没抓到——Δ 形态下账本身就是权威内容，checker 只能拿同一个累加函数重放同一份 Δ 日志，它在和自己比。
- 用户定案 2026-08-29：取幂等完整值，接受 key 数量上升（原话在变更史）。

**欠**：C15（write buffer 混入 Δ）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:41-56`（整段抄，未转述）**

```markdown
#### 已定项 2：节点大小 16 KiB，钉成常量

**定案**：**节点大小 16 KiB。mkfs 把 16 KiB 写进系统配置，实现按常量走，不按设备特性算**——格式里是一个常量，不是一条公式。
<!-- format-const: NODE_BYTES = 16384 -->

**射程**：⚠️ **登记 `format-const` 不等于冻结这个数**（2026-08-31 用户定案：「这个数需要可以调整」）。那条标记只保证 **kb 与实验源码写的是同一个数**，改它照常走 D8（核心索引结构） 的流程；它写进系统配置，仍然是 mkfs 参数。⚠️ **它现在只钉得住一部分源码**：门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/code-source-discipline.sh` 的 format-constants 格）的形态是 `const 名 = 整数字面量`，`e56_epsilon.rs` 的 `16 * 1024`、`e43_extension_point_budget.rs` 与 `e52_head_mechanisms.rs` 的数组形式都不认。⚠️ **16 KiB 是 CPU 缓存轴上的选择，不是设备 I/O 轴上的**：E7（离线索引 harness） 量设备 I/O 时说小节点两端都赢、方向相反，两者量的是不同资源，不矛盾。上限那一维归已定项 4；E1（节点大小 vs 可恢复性） 的恢复时间那一半一旦给出比 16 KiB 更紧的上限，本项重开。

**依据**：

- E20（指针变宽的 CPU 缓存代价）：唯一超出 L3 的工作集上 16 KiB 是最小点；同一实验里 40 字节的指针头部在 4 KiB 节点付出的相对代价远高于 16 KiB 节点，大节点吸收指针变宽；同一条曲线上 8 KiB 的反常六轮稳定且无人解释 ⇒ 在解释掉它之前不许把节点大小写成「按设备特性算」。
- E6（加密算法选型）：加密 / 校验单元的增益在 16 KiB 之后基本饱和。
- E73（节点 key 区间的扇出代价）：码 2 头带上 key 区间之后树高一格都没涨，16 KiB 不用重开。
- 用户定案 2026-08-30：16 KiB 钉成常量（原话在变更史）。

**欠**：C56（没登记的格式常量仍会漂）、C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:57-87`（整段抄，未转述）**

```markdown
#### 已定项 3：key 布局 —— 改名与索引分开处理

**定案**：**key = `(locality_id, inode, offset)`**，`locality_id` 在创建时从父目录继承，**改名时故意不更新**。offset 段是**文件字节偏移**（这个单元第一个字节在文件里的偏移，第 n 个单元从 0 起是 n × 净荷容量 32634）。

- 命名走 dirent 树（`parent, name` → inode），改名就是删一条加一条，**O(1)**
- 数据按 inode 编 key，与名字彻底解耦
- scan 局部性来自继承的 `locality_id`，改名后**局部性缓慢退化，正确性不受影响**
- 退化由 D3（空间分配） 已承诺的常驻后台整理修复，不是新增负担

**批量改名**：`mv 大目录 别处` 在这个设计下**只改一条 dirent**。三处会把它重新变成无界操作，必须一开始就堵死：

1. **`locality_id` 永远不许被改名更新。** 它是**提示，不是正确性的一部分**——可以是错的、可以过期。
2. **改名覆盖已存在的目标 = 一次删除**，必须走已定项 5（whiteout 删除）的 intent log。
3. **后台重新聚簇必须是增量、可中断、可续做的**，且随时可以停掉不做。

**推论（可当 review 判据用）**：**任何让 rename 触及后代的设计改动，一律否决。**

**inode 记录必须存一份 `locality_id`**：它既是数据 key 的首段、又永不随改名更新，而命名树的 value 只有 inode、数据单元头的五元组里也没有它，所以盘上只有 key 自己带着它——读者打开一个改过名的文件时必须有地方拿到它，否则改名之后数据只能靠扫描才找得到。落点是已定项 6（inode 记录的盘上字段表）；这个值按本项从父目录继承，而第一版没有目录，所以「没有父目录时取什么」也要一起定——第一版取 0，登记在已定项 6 的记录字段表。扫描重建时重编出来的 key 首段按 inode 记录 > 幸存 extent key > 0 给这个对象的全部 key 编同一个首段（2026-09-16 用户定案，D18（块里携带什么信息） 已定项 3 那条加字段的提议不收）。

**射程**：它管数据 key 怎么编与改名怎么不触及后代，不管命名树自己的 key（dirent 树按 `(parent_inode, name)` 编，见已定项 5）。scan 局部性不一定要住在 key 序里：可以住 key 序（全路径索引，改名是 O(子树深度)）、住分配器（把子目录的数据放到父目录附近）、住第二索引（仍然贵，且要多维护一份一致性），本工程取「出生时继承、改名不更新的 locality 前缀」（较好，改名免费）。⚠️ 「它是提示、可以是错的」准许的是**不信它的含义**，不是**不存它的位**：作为地址首段它的取值精确且定义性。⚠️ **E9（key 编码对遍历局部性的影响） 的遍历局部性数只在一个口径下成立，引用必须连口径一起引**：**交错创建**（`interleave`，老化的代理）**加上 `cache_leaves = 8`**；`locality_id` 的收益是**新树的收益**，不是稳态收益，改名强度的现实性未校准——曲线可用，绝对刻度不可用。⚠️ 全路径索引的遍历优势也量出来了，但**不许拿它去替 D9（加密） 泄漏面与 D1（数据可移动性 / 反向索引） 稳定性做判断**——E9（key 编码对遍历局部性的影响） 对那两样一个字没说。⚠️ 「每次改名平均重写约 2700 个 key」这个数**缺口径**（查不到是哪一档改名、哪个种子），标为未核实；同表里 ext4 Orlov 分配器「遍历内核树快 30%」也是未核实——kb 内无出处无口径，要用先补。⚠️ 三段 key 在 D6（快照实现模型） 取多可写头之后曾经不完整（两个可写头各写一次同一个 `(locality_id, inode, offset)` 会落在同一个 key 槽、后写覆盖先写），2026-08-30 D6（快照实现模型） 已定项 1 取「每头一棵自己的树」之后消解——每头一棵树之下三段 key 在树内唯一，**两个头不再落进同一个 key 槽**。⚠️ **nonce 那条义务仍在**：D9（加密） 已定项 4 已定 nonce 每 extent 存，**生成时仍必须保证两个可写头之间不撞**（候选形态是按头划分 nonce 空间），它不再由身份唯一性免费得到，否则 I-6.1（nonce 不重用） 从「由身份唯一性保证」变成「由生成器保证」而没人保证它；该义务是推论，未走三方论证。⚠️ 后台重新聚簇是只搬物理位置还是重编 key、重编时怎么分批、进度从哪续做，全仓没有定义。

**依据**：

- E9（key 编码对遍历局部性的影响）：同一棵目录树同一个 LRU 上四条臂对比，`locality_id` 前缀比按 inode 编 key 少发设备读，值得做；depth-first 创建时三条真实臂逐位相同，缓存装得下整棵树时四条臂全部相等、效应消失；收益随改名次数衰减到几乎为零。
- 机制论证（2026-08-25 定案时写下，原文在变更史）：全路径索引在本工程还要多付三条 BetrFS 不必付的代价——分隔 key 就是路径会把 D9（加密） 已定项 5 认定的敏感信息从一棵树扩到所有按路径编 key 的树、引用方身份是路径会让 D1（数据可移动性 / 反向索引） 的反向索引条目随改名全部失效、lifting 对 key 编码的额外要求与 D9（加密） 的 nonce 载体和 D6（快照实现模型） 的快照 ID 位置抢同一段 key 空间。
- 用户定案 2026-09-16：扫描重建时给一个对象的全部 extent key 编同一个首段（原话在变更史）。
- E157（并行线一两条条款的计数模型） 量出：定位一页的节点读取次数、树高与叶片数、换容量后字节区间还原那一格，两臂在全部取样点上逐点相等、分不开；唯一分得开的是 AAD 锚点那一格，文件字节偏移下 N 个 key 全算对、单元序号下只有 1 个算对。
- D9（加密） 已定项 6 逐字「锚点偏移 = 该 extent 首次写出时的对象内起始偏移，读者现算 `key.offset − ptr.extent_off`」，而同一条决策已定项 12 第 2 项写明 AAD 字段组成是 day-1 永久契约 ⇒ 条款本来就预设了这个单位。

**欠**：C346（后台重新聚簇没有定义）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:88-105`（整段抄，未转述）**

```markdown
#### 已定项 4：节点大小 —— 上限由恢复时间定，靠实验取值

**定案**：大节点省 seek、树更浅，但**重扫代价随节点大小线性放大**，所以**不照抄 128K–256K**。**节点大小换的是「恢复时间」，不是「数据安全」，目标线该定在恢复时间上。取值靠 E1（节点大小 vs 可恢复性） 定：在「重扫代价 ≤ 目标线」的前提下取查找代价最低的那档。** 三条约束：

1. **它是 mkfs 时的格式参数，不是挂载参数。** 自动选取只发生在 mkfs 一次。改它 = **重建索引**，不是重建文件系统（D21（权威态与派生态的分界） 已定索引是派生态、冻结第 2 层降级为本地缓存格式）。
2. **默认策略必须是确定性的、可查的**，且 mkfs 要把「为什么选了这个值」打出来。
3. **测试钉死的是「模拟设备的参数」，不是节点大小本身**——固定一组设备档位让自动规则照常跑，**连选取规则本身也一起被测**。要求只有一条：同一档位必须得到同一取值。

**射程**：⚠️ **判据不是「全盘扫描重建成功率 ≥ 目标线」**——索引坏的永久损失恒 0 ⇒ 成功率这一维与节点大小**无关**，拿它当判据夹不出任何上限。⚠️ **目标线本身仍未定**，它是一个恢复时间预算（多久之内要把被破坏的那部分重扫回来），本项自己算不出来；E1（节点大小 vs 可恢复性） 剩下那一半仍欠 mkfs、checker 与一个能定点破坏镜像的工具。⚠️ 前置：索引可从单元自描述重建——E28（明文映射层能不能只靠单元自描述重建） 实测**元数据侧成立、数据块侧今天不成立**（缺口恰等于数据块总数，欠 D18（块里携带什么信息） 已定项 3）；⚠️ **「改它 = 重建索引」取消的是定性，不是代价**：整棵树按新尺寸重建是一次满量级的全盘 I/O，只是不丢数据；而且这个取消有未满足的前置——数据块自描述补上之前，「等于重建文件系统」在数据块那一侧仍然成立。桥接依据是索引树重建与明文映射层重建**读同一份输入**（每个单元自带「我是谁、我属于谁」），⚠️ 这一步是推理，E28（明文映射层能不能只靠单元自描述重建） 只测了明文映射层那一个实例；D11（索引节点要不要留消息缓冲区） 「不能拖」的全部理由就是这半句。⚠️ **第一期只有字节主导那一端在射程内**：D12（目标介质） 已定项 5 定第一条布局线是纯 SSD ⇒ 结论不变，**支撑从两端变一端**；寻道主导那一端要等 HDD 布局线开出来才重新在射程里。⚠️ **节点大小的自动选取判据里必须含指针宽度这一项**（E20（指针变宽的 CPU 缓存代价）：大节点吸收指针变宽，方向与重扫代价相反），此前没有。⚠️ **「按设备特性取相反方向」这个理由不成立**：两端最优方向相同，都偏小；节点大小仍是 mkfs 参数，要保留按介质调档需另找依据。

**依据**：

- E29（坏一个节点的爆炸半径）：索引坏的永久损失恒为 0（D21（权威态与派生态的分界） 已定索引是派生态、丢了只是慢），数据单元坏的损失与节点大小无关，只有重扫代价随节点大小线性放大（第 1 层节点的重扫量恰等于扇出）——所以上限只能定在恢复时间上，「爆炸半径」这个说法被它纠正。
- E7（离线索引 harness）：**固定内存预算下大节点三个轴全都更差**（量的是 io/op、字节/次、点查读/次），机制是节点越大、同样的内存装得下的节点越少，命中率掉得比「每次读更多」赚回来的更快；寻道主导那一端方向相同，大节点仍差 ⇒ 「不照抄 128K–256K」在字节主导与寻道主导两端都得到支持。
- 用户定案 2026-09-06：口径改写成「上限由恢复时间定」（原话在变更史）。

**欠**：C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:106-151`（整段抄，未转述）**

```markdown
#### 已定项 5：whiteout 删除 —— 接受，但批量删除必须走 intent log

**定案**：删大目录、截断大文件会产生海量墓碑，塞进一个事务必然爆掉。**做法（bcachefs 已验证的形态）**：凡是**可能修改无界数量 extent** 的操作（truncate、collapse、删大目录），**先把意图写进 logged_ops 树，再分批推进**；崩溃后恢复时续做未完成的意图。删除中的 inode 单独进 `deleted_inodes` 树，由后台回收——用户的 `rm` 返回，不等于所有 extent 都已释放。第一版 unlink 不往 logged_ops 树写条目：意图就是 inode 记录 nlink = 0 加 `deleted_inodes` 那一条（已定项 16）。logged_ops 树在第一个文件版本那次与 `deleted_inodes` 一起零根注册，字段表、回退时取哪一版、态别留到第一个写者（整理，D26（后台整理与放置回收） 已定项 2）落地时定（C364（整理意图记录没有字段表））。意图分两类：**承诺类**（unlink；truncate、删大目录）必须续做到底，可写挂载交回之前走到空；**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载；续做入口按树分、共用发布路径。本条第 2 条与三态表、C1（意图续做做成常规路径） 的字面今天按承诺类写，整理落地时按类分开写（C621（整理落地那天要动的已写回删除条款没有清单之外的检查））。

**硬要求**：无界规模的操作一律不许做成单个事务；意图必须**幂等**；whiteout 只在节点重写合并 bset 时清除，**不许有别的清除路径**。

意图本身就是一条普通 key，骑在同一个事务层上，断电后只有三种状态：

| 断电时机 | 恢复后看到 | 怎么收场 |
|---|---|---|
| 意图提交之前 | 没有意图 | 操作从未发生，用户那边也没返回成功 |
| 意图已提交、工作做了一半 | 意图还在 | 恢复时**续做**，已做完的部分被幂等地重跑 |
| 工作做完、意图未删 | 意图还在 | 续做一遍发现无事可做，然后删掉意图 |

不可能出现「意图没了但目标状态没达成」——**删除意图必须与最后一批工作在同一个事务里提交**。

1. **意图声明目标状态，绝不声明增量。**「把文件 X 截到 N 字节」可以重复执行，「再删 1000 个 extent」不能。
2. **续做是每次恢复都跑的常规动作，不是修复手段**（否则会出现「平时不跑、真出事时才第一次跑」的代码路径）。
3. **中间状态必须每一个都是合法的文件系统状态**——「删了一半 extent 的文件」本身就得合法。
4. **一批里每个动作施加时先核它要删的东西还在**（随 D5（快照 / 空间记账机制） 已定项 12）——幂等的字面：一批按批开始时的状态为几个意图各算一份计划、施加时不复核，两个区间重叠的级联会把同一块放两次，统计量减两次、I-3.1（已分配统计对得上） 红。

**范围重建 vs 逐项修改：两条都要有，且要有显式判据**（`.claude/rules/fs-design.md`「不追求万能方案」）：

| 走法 | 代价 | 适用 |
|---|---|---|
| 逐项修改 | K × 随机访问 | 受影响的 key 稀疏散布 |
| 范围重建 | N × 顺序访问 | 受影响的 key 在 key 序上连续 |

**真正的开关不是 K/N，是「受影响的 key 连不连续」**：连续 ⇒ 范围重建可整节点丢弃或重写；不连续 ⇒ 只能逐项。**由此得到一条布局规则：让一起死的东西在 key 序上挨在一起。**

- dirent 按 `(parent_inode, name)` 编 key → 删整个目录是范围操作
- extent 按 `(inode, offset)` 编 key → 截断是范围操作
- 同一快照的 key 带同一 snapshot 前缀 → 删快照有机会变成范围操作

**这顺带从结构上解掉了 whiteout 膨胀**：范围删除直接丢弃整个节点，不用插 N 个墓碑。

**射程**：朴素的交叉点是 `K/N > 顺序代价/随机代价`（机械盘约 1%、SSD 约 10%）。⚠️ 这两个数是**推理，不是实测**，说的是**耗时**，要靠 E4（范围重建 vs 逐项修改的交叉点） 量，那一半要 QEMU。⚠️ **第一期只取 SSD 那一档**（D12（目标介质） 已定项 5：第一条布局线是纯 SSD）。⚠️ 写放大那一半量到的交叉点，自变量是「布局」与「节点大小」而不是介质，不能拿它去证伪「机械盘约 1%、SSD 约 10%」——那说的是耗时。按 `.claude/rules/fs-design.md` 的五条要求，这个分支还欠：判据写成可计算的式子、留一个能强制走指定分支的测试开关、两条分支各自跑崩溃点重放、运行时能看出走了哪条。⚠️ **切换判据里不许出现「当前填充率」这类操作中途会变的量**——判据只能用操作期间稳定的输入。**与 D3（空间分配） 的交互（容易漏）**：意图未完成期间那些空间还没真的释放，空间统计必须把「待删除但未完成」计进去，否则会出现「我删了一大堆，`df` 却不动」。

**依据**：

- E30（交叉点的写放大那一半）：**写放大那一半已量**——在写放大口径下量出条目连续与条目散布两种布局在 4 KiB / 16 KiB 节点上的交叉点，两者相差两个数量级以上——开关是布局与节点大小，不是介质。
- 用户定案 2026-09-16：加第 4 条硬要求「一批施加时先核要删的东西还在」（原话在变更史）。
- 三方 `research/prompts/m3-intent-r1-main-verification.md`：意图分承诺类与提示类（两腿独立得出，一轮）；logged_ops 树现在零根注册是用户定案 K-I1（原话在变更史；被攻过零轮）。

**欠**：C1（意图续做做成常规路径）、C2（无界操作做成单事务）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:152-223`（整段抄，未转述）**

```markdown
#### 已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用
<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 P8 P9 -->

**定案**：inode 树 = 码 2 内部节点（派生态，不留消息缓冲区，条目 120 字节、扇出 135）+ 码 3 打包记录单元做叶（打包记录类型 2，权威态单元，一叶 233 条 140 字节记录）；根恒为码 2；容器索引就是这棵树的内部节点；已发布的 (inode 号, 对象出生代) 不复用，下一个号是记账里一条取最大值合并的统计量。码 2 根装满时长一层——新根 1 个码 2、分出的右半 1 个码 2（右半没有容器身份），inode 树与墓碑树（D18（块里携带什么信息） 已定项 24）共用这一条；长高那次发布多 2 个 16 KiB 节点 × 副本数。根长高之前今天的代码在 `crates/singlefs-core/src/inode_tree.rs` 的 `MoreLeafContainersThanOneRootNodeHolds` 拒（推的单元数，零轮）。

| 部件 | 是什么 | 态别 |
|---|---|---|
| 叶 | 一个 32 KiB 码 3 打包记录单元（D18（块里携带什么信息） 已定项 11），打包记录类型 2，装 233 条 140 字节记录；自带类身份段、记录数、记录宽、载荷校验和，扫描路径上的载荷完整性证据（C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据） 对码 2 的洞）对它不存在 | 权威态（单元） |
| 内部节点 | 码 2 索引节点。头带 key 区间（min / max 各 8 字节，D18（块里携带什么信息） 已定项 2 留给 inode 树的那一格由此定「带」：理由与记账树同一条——一个节点单独捡起来能自证覆盖段，key 是 inode 号不是文件名、敏感性论证够不到）。**头是块头，落 D15（格式冻结政策） 第 1 层（冻结），与记账树节点头同一格（C102（索引节点头没有预留区，加字段不是 compat））；载荷布局（条目 120、ε = 0）落第 4 层，不永久** | 派生态，丢了从叶重建 |
| 根 | 恒为码 2 索引节点，哪怕树只有一片叶，哪怕树是空的（零条目，key 区间写 [0, 0]，0 是无效 inode 号）。理由：D19（块指针的结构与宽度预算） 的指针不带被指对象的类与大小，树表指针指向的根若类会随树长大而变，读者按 16 KiB 读 32 KiB 是静默误读 | 派生态 |
| 容器索引 | 就是这棵树的内部节点，不另立树：物理指针只住内部节点，dirent 树引用的是 inode 号。更新一条记录 = COW 叶 + 全部祖先（D23（journal 的角色与格式） 已定项 1 甲） | — |

**内部节点条目 = 分隔 key 8 + 身份引用 26 + 子指针 86 = 120 字节**（扇出按今天的头 131 算 ⌊(16384 − 131) / 120⌋ = 135）。身份引用就是 D18（块里携带什么信息） 已定项 11 给「别的树引用容器」的那 26 字节：出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8。**类型段兼作「子节点是什么」**：0 ⇒ 子节点是码 2 索引节点（容器号、出生代两段恒 0；登记表里的 0 在容器头里仍是无效，两处含义靠所在结构区分），2 ⇒ 子节点是码 3 容器（读两个 16 KiB 槽）；其余值判条目损坏。合法集合按树分：inode 树 {0, 2}（I-9.2（条目身份与子头相符））、墓碑树 {0, 1}（I-9.23（墓碑树条目身份与子头相符））、谱系树 {0, 5}（I-9.17（谱系树条目身份与子头相符）），三棵树的内部条目同一形态。读者由此知道读几个槽；码 3 那条边的 AAD 五段（标签由类型 ≠ 0 推出，其余四段照抄条目）全部来自查找路径，D9（加密） 的 A6（AAD 期望值不来自明文侧）在这条边上不需要树的种类；码 2 那条边（内部节点指内部节点）的 AAD 仍是 C107（索引节点的 AAD 身份无落点）。出生树逐条目携带，是因为克隆头（D6（快照实现模型） 已定项 1）共享 origin 的叶，同一个内部节点里会混着出生树不同的条目；D19（块指针的结构与宽度预算） 已定项 7 让子指针头部也带出生树之后，身份引用里的出生树**照旧保留**——两份并存，加密开不开都占位、不挪作他用；类型段 0 条目身份引用里的出生树恒 0，与容器号、出生代两段同形；码 3 那条边 AAD 的出生树取身份引用那份，不取子指针那份。**分隔 key** = 该孩子创建时的最小 key，条目按分隔 key 单调递增，查找取最后一个分隔 key ≤ 目标 key 的条目。
<!-- format-const: INODE_INTERNAL_ENTRY = 120 -->

**内部节点不留消息缓冲区，inode 树的 ε = 0**——这是对 D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴增补（布局轴「全部布局共用同一组 ε」不动）。分支变量是 D11（索引节点要不要留消息缓冲区） 已定项 4 那条谓词：**这棵树待下刷的消息能不能只从权威态（单元（带写序）+ 实例表 + 祖先表 + 墓碑单元）重算**。能 ⇒ 留缓冲（ε = 0.65）；不能 ⇒ 不留。inode 树不能：属性更新（mtime / nlink / mode 的新值）在到达叶之前没有权威态载体，缓冲进派生态节点就撞 D21（权威态与派生态的分界） 硬约束 6。extent / dirent / 反向索引能（E59（缓冲消息能不能只从单元重算） 的「单元 + 记账」一行三类全 0）；记账树按 D21（权威态与派生态的分界）「离线可由遍历重算」判能，但 E59（缓冲消息能不能只从单元重算） 那一列是神谕、未实测。它落 D15（格式冻结政策） 第 4 层（派生态），不是格式分支。**态别按重算性判，不按「树」这个词判**：一棵树的内部节点可从它的叶重算 ⇒ 派生态；叶若装着算不回来的值 ⇒ 权威态。记账是同一条判据的已定实例，权威记录点在 D21（权威态与派生态的分界）。

**装载纪律按记录类型分**：「一容器一个代际」与「快照发布前先关闭开放容器」只对按代际回收的打包记录类型 1（墓碑）成立；类型 2 的记录逐条死亡，容器由 B 树分裂 / 合并回收，一片叶活很多代、反复重写，快照只是共享它。类型 2 的记录死亡不写墓碑，死亡由现行叶版本体现，择现行版本压在 C113（扫描重建时多版单元的现行版本判定无输入）。

**容器身份不需要分配器**：inode 号在每一条时间线上单调（管理员回退的新根取 R_old 那一版的 inode 树、水位取 max(现行那一版内存里的, 环里读得出的根的)；崩溃恢复回到所选根的树与它的水位；两种之后发的号都大于那棵树里每一个 key）⇒ 插入永远落在最右的叶 ⇒ 分裂只发生在最右叶，**分裂点取末尾**——左半保留全部原有记录与身份，右半从触发分裂的那条新记录起，**右半容器号 = 那条新记录的 inode 号**，出生代 = 本次发布的 checkpoint_txg，**出生树 = 执行这次分裂的那棵树**（分裂是新建，与「COW 重写身份不变」是两件事）；第一个容器的号 = 第一条记录的 inode 号。内部节点同样在右端溢出、末尾分裂，右半是码 2、没有容器身份。合并：只许**左吸收右 / 左从右借**（右吸收左或右从左借会破「记录号 ≥ 容器号」），保留左半身份，右半退役走 COW 释放路径；合并时机是写路径策略（单测 / 模型对拍），不进格式。**记录数为 0 的类型 2 容器不落盘**（发布前必须合并掉；唯一的叶被删空时是不落盘、根变成零条目）。**跨头首次 COW 的重生**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P6）：新身份 = (本头树 ID, 类型 2, 原容器号, 本次发布代号)——保留容器号、只换出生树与出生代，同时给旧身份写退役记录。重生只对类型 2：类型 1 容器归写者树独占、克隆不收进自己的容器索引（D18（块里携带什么信息） 已定项 11），从不跨头 COW。两条不同的断言并存：「容器被 COW 重写时身份不变」（内容同、位置变）与「分裂后左半身份不变」（内容少了、身份不变）。类型 1 的容器号仍按 C103（容器索引无落点） 那套单调分配，两条规则按记录类型分。

**key 与 inode 号**：key = inode 号 8 字节，唯一性范围 (树 ID, inode)，位宽 64，声明生命周期创建总数上界 2^63。**已发布的 (inode 号, 对象出生代) 不复用**（D6（快照实现模型） 判据 9 的戒律）：`deleted_inodes` 后台回收窗口里复用会让新旧对象的 extent key 三段逐字相同。⚠️ **「已发布的」这四个字不是为了对齐措辞，是让现行设计自洽**：水位住记账（D5（快照 / 空间记账机制） 已定项 4 第 12 项），被抛弃 checkpoint 的那笔账随它一起丢 ⇒ **每一次普通崩溃**都会让那一代发过的 inode 号被重发。管理员回退是挂着时的一次向前发布，那次发布的 inode 号水位取 max(现行那一版内存里的, 环里读得出的根的)，已发过的号不重发（D23（journal 的角色与格式） 已定项 14）；崩溃恢复落到较旧的根时水位随所选根那棵账载入，被抛弃时间线里已发过的号会被重发，而恢复之后每一个新对象的出生代都大于环里读得出的每一条根与记录的 txg（D23（journal 的角色与格式） 已定项 14 注 3）——要几个故障才撞得上被抛弃时间线里已发布对象的出生代，崩溃恢复那一形没量过，见 C143（inode 号水位在崩溃恢复落到旧根后退回去重发）；对外暴露的对象身份 = (inode 号, 对象出生代)，凡按 inode 号认对象的结构都要带上出生代或随根回退。`deleted_inodes` 回收窗口里 extent key 三段逐字相同那一句说的是同一条时间线里的复用（旧对象已发布、它的 extent 在树里等后台回收），照旧被禁；崩溃恢复之后重发同一个号，新对象的 extent 落在所选根那棵 extent 树里，旧对象的 extent 随被抛弃的根一起不在树里——**这靠 extent 树与 `deleted_inodes` 都随所选根**；管理员回退那次发布的新根取 R_old 的 inode 树与 extent 树，`deleted_inodes` 也取 R_old 那一版的（否则 R_old 之后删掉、随 R_old 的 inode 树回来的对象会被后台回收再收一次；主 agent 2026-09-26 定，推的，被攻过零轮）；`deleted_inodes` 的形态（已定项 16）守这两条；盘上分开两者的是单元五元组里的对象出生代（D9（加密） 已定项 6）。**inode 记录在回收完成前留在 inode 树里（nlink = 0），`deleted_inodes` 只存号**。

**下一个 inode 号是记账里的一条统计量**（D5（快照 / 空间记账机制） 已定项 4 第 12 项「inode 号水位」）：key = (统计量, 树 ID, 设备段, 代)，不带设备维，value = 下一个可用号，**合并规则取最大值**（幂等、交换、结合）；条目照样带 seq（已定项 15 第 3 条是格式要求，一棵 write buffer 树一套条目格式），只是**合并结果不依赖它**。选它而不是「后者胜」的理由是崩溃后 txg 重发那一格：同一个代段里先后出现大小两个值，取最大值给大的（浪费号、不复用号）；「未发布的等于没发生」只对常规挂载路径成立，扫描重建看的是盘。「标签 → 合并规则」是盘上语义，统计量登记表立成独立冻结组件（D15（格式冻结政策））。水位行只为可写头维护、随每次发布重写（D5（快照 / 空间记账机制） 已定项 2 的 K 代点删要求每行每发布重写），只读树没有这一行；读现行值 = 取该维度元组下代号最大的那一行，空 ⇒ 从未维护过、不是 0。克隆头：新头的水位 = origin 那一刻的运行时水位（含还在 write buffer 里没落盘的那条），建克隆那次发布必须为新树写出这条水位行；两个头此后各自分配，号会相交，唯一性靠 (树 ID, inode)。

**记录字段表（定长 140，小端；对齐口径是记录内偏移，容器内第 n 条起点 136 + 140n（码 3 头 107 + nonce / MAC / 算法类型预留位 29，D18（块里携带什么信息） 已定项 16）不按 8 对齐，按非对齐读）**
<!-- format-const: INODE_RECORD_BYTES = 140 -->
<!-- format-const: INODE_LEAF_RECORDS = 233 -->

| 偏移 | 字段 | 宽 |
|---|---|---|
| 0 | inode | 8 |
| 8 | 对象出生代（创建那次发布的 checkpoint_txg；inode 号不复用后它不再承担复用判别，按五元组 / AAD 的 day-1 契约保留，扫描时与该对象每个数据单元头逐字段对照） | 8 |
| 16 | locality_id（第一版无父目录取 0） | 8 |
| 24 | mode / uid / gid / nlink，各 4 | 16 |
| 40 | size / blocks / rdev，各 8 | 24 |
| 64 | atime / mtime / ctime 秒，各 8（unlink 的意图那次三个共写调用方给的一个值，各批照抄） | 24 |
| 88 | 改动计数（最后一次改动所在发布的 checkpoint_txg；提交锚 = 叶容器头的诞生代号；同一发布窗口内多次改动只有最后一版落盘——内部节点不缓冲；unlink 的意图那次与每一批也算改动，写本次发布的 checkpoint_txg，不照抄上一版；创建算一次改动，所以改动计数不低于记录偏移 8 的对象出生代，I-9.26（inode 改动计数落在出生代与容器诞生代号之间）） | 8 |
| 96 | atime / mtime / ctime 纳秒，各 4 | 12 |
| 108 | 填充（恒 0） | 4 |
| 112 | flags（恒 0） | 8 |
| 120 | 预留（恒 0） | 20 |

**写入时间（偏移 64 起三个秒字段、偏移 96 起三个纳秒字段）取写这条记录的那一次发布的参数，不取系统时钟**：每次发布调用带一个写入时间，同一个参数两次发布逐字节相同（可复现）。三个秒字段都写参数的秒数；三个纳秒字段写参数的纳秒部分，今天的参数只带秒数，所以写 0。**新建**（新池新建文件、新建 inode）写创建那次发布的参数；**改写内容**（覆盖写、顺序写）重写这条记录时写这一次发布的参数，不照抄上一版的值；**unlink 的意图那次**写调用方随这次 unlink 给的时间，三个时间秒（atime、mtime、ctime）共一个值，各批照抄意图那次写的值；**不改这条记录内容的发布**——同一片叶里别的记录改动、这条被连带重写，回退——照抄这条记录上一版的值。

**`blocks` 是逻辑长度的 512 字节块数**：`blocks` = ⌈`size` ÷ 512⌉，按文件长度向上取整，**不表示分到的空间**，不按这个对象分到几个数据单元算。新池新建文件那条记录 3000 字节 ⇒ 6；新建、还没写过数据的 inode ⇒ 0。

不放 extent 指针（D14（双轨（大小文件 / 持久临时）） 已定项 3 内联阈值 0），btime 不进记录。**flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ 该记录 EIO（对象级，不拒整个容器）**——前向兼容政策不是损坏检出（载荷校验和照样过），落成 I-9.7（记录三段恒零），在 I-1.7（打包容器合法与判定顺序） 的中止链之外。将来给任一位 / 任一字节赋义按 D15（格式冻结政策）「改变已有字段含义」判 incompat，退役的字节永久报废。**记录内不存在 compat 扩展路径**：inode 属性都有语义依赖，D15（格式冻结政策） 判定表第一行够不着，且记录被反复重写、旧读者会把不认识的内容丢掉。预留 20 不承重：记录内任何赋义都是 incompat，宽度只决定那次要不要连记录宽一起改。**溢出路线按属性逐个判档**：无语义依赖的属性走新增 keyspace（D15（格式冻结政策） 判定表给 compat），有依赖的走 flags 位 / 预留（incompat）；属性 keyspace 今天没有落点（xattr 被推迟）。**记录宽必须等于登记表给类型 2 的宽**，不等判损坏（I-1.7（打包容器合法与判定顺序） 里在「类身份段」之后、I-1.2（块头写序已发布） 之前插一步，只需头字段）；改宽是 incompat。记录格式、码 3 类身份段对类型 2 的用法、inode 树的 key 编码是永久字节：**立成一个独立的已冻结组件**（C9（冻结后偷改格式） 按组件记），冻结时刻是第一个写出类型 2 容器的镜像；不动 D15（格式冻结政策） 的四层；类型 1 的记录格式（宽 40，D18（块里携带什么信息） 已定项 24）各自独立。

**不变量**：I-9（inode 树结构） 一类，十四条，见 [invariants.md](../invariants.md)；状态列统一带前置「树的种类可识别（已定项 8）」，第一版镜像上可由类型 2 容器头里的出生树反推哪棵树是 inode 树。nlink == dirent 条数不在这里立——它的门（dirent 树存在）今天没有输入，随 dirent 树上线时一起立。第一版的唯一文件取 inode 1（0 无效），nlink = 1，不进孤儿回收。分裂 / 合并的身份传递与合并方向以外的写路径纪律是操作断言，走单测 / 模型对拍与崩溃点重放。

**重建**：内部节点丢了，收集打包记录类型 2 的容器，按 (出生树, 容器号, 容器出生代) 分组，同身份多版的择现行 = C113（扫描重建时多版单元的现行版本判定无输入）；每叶 key 范围由记录算，内部节点的 key 区间由叶算；产物落 D18（块里携带什么信息） 已定项 5 的级 1（只读挂载），级 2 的见证者今天不存在。**extent 树的重建**：对象按 (树 ID, inode) 认、只看本头的 extent 树；这个对象的全部 extent key（幸存的与找回的）编同一个首段，取值 inode 记录 > 幸存 extent key > 0，记录完好时重建视图里的值就是记录里的值；取错只丢局部性。I-9.9（locality 三值对照） 在重建之后按侧判：一侧是从另一侧编出来的就输出「不可判定」。

**新池新建文件写**：一个码 2 根节点（头带 key 区间 [1, 1]；一条条目：key 1、类型 2、出生树、容器号 1、出生代 3、指针）；一个码 3 叶（类型 2、容器号 1、出生代 3、记录数 1、记录宽 140）；一条记录（inode 1、出生代 3、locality 0、nlink 1、改动计数 3、三段 0）；记账里一条「inode 号水位 = 2」。

**射程**：**唯一性的射程是一条已发布历史内**——每个容器号都是某条记录首次插入时的 inode 号；号在崩溃恢复落到较旧的根之后可以重发（管理员回退那次发布的水位取 max、不重发，D23（journal 的角色与格式） 已定项 14），所以同一棵树里两个容器可以同号，分开它们的是容器出生代（右半的出生代 = 触发分裂那次发布的 txg，而恢复之后新实例的每一个 txg 都大于环里读得出的每一条根与记录的 txg，D23（journal 的角色与格式） 已定项 14 注 3），同一次发布内「分裂 → 合并 → 再分裂」也造不出同身份两版。跨崩溃不成立：崩溃后 checkpoint 号会被重发，被抛弃那版与重试那版四段逐字相同、内容不同、都在盘上——这与数据单元的五元组、码 2 节点的 (树, 层级, 诞生代号) 是同一种暴露（根记录靠 D16（发布语义） 骑手条款 1 的实例代号分辨，别的单元类靠类身份段里的写序 (实例代号, 事务号) 分辨——容器不单独加实例代号，写序里那 4 字节就是它），常规路径由 D3（空间分配） 已定项 1 的谓词（分配记录）判，扫描重建路径按 C113（扫描重建时多版单元的现行版本判定无输入） 定案的 P2 / P3 规则判。⚠️ **判它安全的依据与树 ID 那条同源**：孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」），扫描重建的两道闸按写序择新。⚠️ **残留的疑点**：讲统计量合并规则那一句「『未发布的等于没发生』只对常规挂载路径成立，**扫描重建看的是盘**」读起来像是反方向的——它管的是「水位在一个代段里取最大值而不是后者胜」，与「未发布过的号可不可以重发」不是同一个命题，但两者挨得很近。⚠️ **重建的射程是单头镜像**：有克隆头时共享叶头里的出生树是 origin（I-1.3（块头树 ID 一致） 的 C110（跨头共享与加密后元数据类的块头一致性读法） 那一格），按出生树认领会把克隆头的树认残——多头镜像的归属划分要克隆祖先表（E90（树 ID 进 AAD 与跨头共享） 第二笔），落地前多头镜像的 inode 树重建没有输入。⚠️ 填充率没有下界：`⌈n/233⌉` 是新建树的值，1e6 顺序创建再删 90% 不合并是 9.98 倍空间放大；加宽记录能同时缩小爆炸半径（160 ⇒ 204 条），这笔取舍归 C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）。⚠️ **挡着「新池新建文件写」的九样，① ② 已定案，其余照录**：① 树表条目已由已定项 8 定；② 记账条目的 seq 住 value、宽 4 字节（已定项 7）；③ 统计量标签值（C71（不认识的统计量标签静默忽略））；④ 不带设备维的统计量的设备段取值、新建树的记账行初始化、读现行值的读法（C117（不带设备维的统计量的设备段没有取值））；⑤ 码 3 容器带不带扩展点（C108（扩展点归属对打包记录单元无判定））；⑥ 码 2 节点头的基础字节数（已由已定项 11 收口）；⑦ `deleted_inodes` 的形态（已定项 16）；⑧ ⑨ 记录格式与统计量登记表两个冻结组件的 spec 文件（C119（冻结组件没有 spec 文件））。⚠️ 跨头首次 COW 时容器身份要不要重生的另一半归 E90（树 ID 进 AAD 与跨头共享） 第二笔。⚠️ **`blocks` 不承担对外的 `st_blocks`**：stat 报的 `st_blocks` 怎么给（stat 时从 extent 树现算分到的单元）、SEEK_HOLE / SEEK_DATA，挪到做 POSIX 面的里程碑。`blocks` 按文件长度向上取整改新池新建文件的字节：那条记录偏移 48 的 8 字节从 64 变 6。 ⚠️ **写入时间单镜像判不出**：取值是发布参数，单个镜像里看不出参数是什么——O1（内存模型对拍（HashMap 理想文件系统）） 的模型与对拍不带写入时间，O2（独立解析器 + checker） 只判单个镜像，O3（独立规约执行器） 的投影不进 inode 树；判它的是按写路径钉写的 harness 用例（覆盖写、顺序写、建 inode、unlink 各一条），多单元顺序写、回退、同一片叶里被连带重写的别的记录三格没有用例（C677（写入时间在三格写路径上没有用例））。

**依据**：

- E98（inode 记录与 inode 树的几何）：inode 记录的盘上形态——打包进码 3 不触发 D21（权威态与派生态的分界） 硬约束 6、住索引节点触发（14 个字段里 11 个算不回来）；改动计数单靠 checkpoint_txg 排不了同窗口内两个版本，所以内部节点不缓冲、同一发布窗口只有最后一版落盘。
- E103（inode 打包路 ① 的更新代价）：路 ①（独立容器索引 + inode 树按身份引用）的读代价比容器索引并进内部节点高、还多一次单元读，而单次写不变——容器索引不另立树。
- E104（扫描重建的现行版本判定）：不认退役记录时退役容器里已死的 inode 会复活；模型打出「重生取最小 inode 号当容器号会撞车」，所以重生保留容器号、只换出生树与出生代。
- 四轮三方论证后用户定案 2026-09-05（材料 `_d8-item6-path.md`、`_d8-item6-r2-amended.md`、`_d8-item6-r3-conclusion.md`、`_d8-item6-r4-conclusion.md`、`_d8-item6-r5-landing.md`，四轮三腿输出 `research/prompts/d8-item6-r{1,2,3,4}-{sonnet,opus}-output.md` 与 `research/results/d8-item6-r{1,2,3,4}-local.out`）。
- 用户定案 2026-09-12（C297（inode 内部条目里出生树两份没有定处理），两轮三方）：身份引用里的出生树与子指针里的那份两份并存，原话在变更史。
- 用户定案 2026-09-16（C143（inode 号水位在崩溃恢复落到旧根后退回去重发），三轮三方）：唯一性收窄到 (inode 号, 对象出生代)，原话在变更史。
- 用户定案 2026-09-23：`blocks` 按文件长度向上取整、不表示分到的空间，`st_blocks` 挪到做 POSIX 面的里程碑（主 agent 的提议，C480（inode 记录的 blocks 怎么算全仓没有条款）），原话在变更史。
- 用户定案 2026-10-01：K-R3-3「写根长高」，`research/prompts/m3-delete-r3-main-verification.md` 第三节 G6-墓碑树根满（原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K3：写入时间在各写路径上的取值（两腿同，攻方在副本上量过、主 agent 复跑坐实），三条 oracle 都判不到。
- 用户定案 2026-10-03：删除步算改动（「算改动，改实现」）、时间字段取「乙 意图带时间、三格同改」，原话在变更史；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 N3 那一行（收窄掉「ctime 跟着改」没回问用户，记录偏移 64 起三格写同一个值）。

**欠**：C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）、C102（索引节点头没有预留区，加字段不是 compat）、C103（容器索引无落点）、C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据）、C106（开放容器数上界）、C107（索引节点的 AAD 身份无落点）、C110（跨头共享与加密后元数据类的块头一致性读法）、C113（扫描重建时多版单元的现行版本判定无输入）、C115（inode 属性的溢出路线无落点）、C116（叶容器的分裂 / 合并纪律没有会失败的检查）、C119（冻结组件没有 spec 文件）、C152（收窄后未发布号重发的扫描重建复核）、C347（祖先表读不出时克隆头认领什么没有条款）、C677（写入时间在三格写路径上没有用例）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:224-241`（整段抄，未转述）**

```markdown
#### 已定项 7：write buffer 条目的 `seq` —— 4 字节，住 value

**定案**：**`seq` 宽 4 字节，住 value 不住 key**，射程覆盖记账 / 反向索引 / LRU 三棵 write buffer 树，**一棵树一套条目格式、三棵同宽**。宽度定死之后因果倒过来：**`T_dirty` 的上界从此是挂载时判定**（4 字节 = 32 位窗口，按 34 字节/条对应约 136 GiB）；D16（发布语义） 已定项 5 那句「`T_dirty` 是可调值，不是格式常量」原样成立，改的是它多了一个由格式定的上界。**在 seq 里再放一份 txg 是把 key 里已有的段抄第二遍**，不许。两条骑手：

1. **反向索引与 LRU 定 key 时必须让去重窗口同样被 `T_dirty` 界住**（代进 key，或别的等效办法）。4 字节够用的依据是「同一个 key 的待去重条目必然同属一个发布窗口」，而那一条在记账树上由 D5（快照 / 空间记账机制） 已定项 5 的 key 末段是「代」推出来；那两棵树的 key 布局全仓一个字都没有，不兑现这条骑手，4 字节在它们身上没有依据。
2. **同一个 key 的跨批次插入顺序必须与 `seq` 同向**，不许留成隐含假设——btree 层同 key 是「谁后写谁赢」，不是「谁 `seq` 大谁赢」，而按叶分区的并行 flush 下两个批次的物理落地先后不受 `seq` 约束。

**射程**：它定的是条目里这一段的宽度与落点，不定别的段（记账 key 的 `(统计量类型标签, 树 ID, 设备, 代)` 四段在 D5（快照 / 空间记账机制） 已定项 5）。⚠️ 这一段不是与别段正交的：那四段在**同一个 checkpoint 内不唯一**（幂等完整值会被写很多次），去重规则「后者胜」逐字要求「后者」由条目自己携带 ⇒ 不定它，那四段合起来还不是一个定得下合并结果的 key。它**立在 D8（核心索引结构） 不立在 D5（快照 / 空间记账机制）**，因为 write buffer 覆盖「反向索引 / LRU / 记账」三类，立在记账那边会让另外两棵树各自重问一遍。⚠️ 记账条目因此从 30 字节（key 22 + value 8）变 **34**，E97（记账与分配记录的条目编码） / E71（记账 key 的条目数与宽度） / E54（记账的「代」要保留几代） / E99（write buffer 条目与 seq 的去重） 四页的数要重算。⚠️ 「inode 号水位」那条记账条目照样带 `seq`（已定项 15 第 3 条是格式要求），只是它的合并结果取最大值、不依赖 `seq`。⚠️ 取值规则（从几起、一次发布里怎么数）在已定项 10；前端与节点缓冲怎么衔接在已定项 12。

**依据**：

- E99（write buffer 条目与 seq 的去重）：量的正是同一个 key 的待去重条目在一个发布窗口内能有多少条、去重要几位——窗口需求落在 32 位以内，4 字节够。
- E17（write buffer 合并上限）：第一版漏了条目序号，两条臂与真值当场对不上，逼出「条目必须自带 `seq`」这个可判定形态；按叶分区可并行的实测同时说明并行 flush 下物理落地先后不受 `seq` 约束，骑手 2 因此必须显式写下来。
- 三条已定条款的字面冲突（三方论证，主 agent 逐条现查）：D5（快照 / 空间记账机制） 已定项 2 逐字「不用找 ⇒ 是点删不是扫描；触及的 key 数 = 统计量数」——`seq` 进 key 之后同一前缀下有多行，点删当场不成立；已定项 6 逐字「条目照样带 seq……只是**合并结果不依赖它**」——单调统计量取最大值要求两条相遇，而 `seq` 进 key 之后它们是不同的 key、根本不会相遇；D5（快照 / 空间记账机制） 已定项 5 的 key 定义本身不含 `seq`，而「后者由 `seq` 定」只有在 `seq` 不进 key 时才自洽。主 agent 补的第四条：`seq` 进 key 会让 key 不可构造——折叠后同一代只留一条，读者想查 `(标签, 树, 设备, 代)` 的值时不知道 `seq`，点查退化成前缀扫。
- 用户定案 2026-09-06：宽度取 4 字节并把因果倒过来、射程扩到三棵树、`seq` 住 value（原话在变更史）。

**欠**：C161（缓冲两级并存而衔接没定）、C162（seq 进 value 之后四个实验的数要重算）、C345（反向索引 / LRU 的陈旧检出没有时间线索）、C416（去重窗口被 T_dirty 界住那条骑手没人查）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:242-284`（整段抄，未转述）**

```markdown
#### 已定项 8：树表条目的字段表

**定案**：**树表条目合计 200 字节** = **树 ID 8**（打头）+ 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8 + **头 ID 8** + 预留 76。每层 ⌊(16384 − 131) / 200⌋ = ⌊16253 / 200⌋ = **81** 棵，二层封顶 81² = 6561 棵。⚠️ **分母的口径写死在这里**：16384 减**含预留位的码 2 头**（131 是树表单元自己的码 2 头，key 宽 8 ⇒ 115 + 16，D18（块里携带什么信息） 已定项 16 / 已定项 18；条目区起点是含预留位的头末尾，已定项 11），不减别的数。

- **① 快照进树表**，每个快照一条条目；**诞生 txg 8 字节**，口径是**快照完成（发布）那一次 checkpoint 的 txg**，不是根节点的诞生代号（那是它最后一次 COW 的号，这棵树自己算不出来）。它给 I-3.7（活快照可枚举） 一个载体。
- **② 树 ID 水位住根记录，取根环里全部根记录该字段的 max**。逐字形态：`新水位 = max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)`，**每次发布随根记录重写**。「根环里全部根记录」读作根环里全部自证过的根（被抛弃时间线上的也算），并上环里全部自证通过的 journal 记录新根段带的该字段（D23（journal 的角色与格式） 已定项 15）：根所在的槽读不出而它那条记录还在时，恢复本来就从记录把那条根重建出来，记录带的水位就是那条根的水位；journal 记录要等它对应的树写好、根落盘之后才会被绕环盖掉。根与它那条记录都读不出（双重介质故障）的那一格盖不住，记在 C342（树 ID 水位在根读不出时退回去重发）。从 mkfs 之外的水位发第一个文件版本时，十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547（extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops）。⚠️ **「累计」这两个字是承重的**：读成「本 checkpoint 发出的最高号」（不累计）时，树 ID 会随根轮出环被重发，而**那时被重发的号属于已发布的树**，D6（快照实现模型） 判据 9 正面命中 ⇒ 整条定案不成立。
- **③ `previous_snapshot_txg` 每头一份，住树表条目**（8 字节）。它是**每个头一个标量**不是每个单元一份，答的是「这个头从哪个快照分出来」，与单元自己的内容无关 ⇒ 按 D1（数据可移动性 / 反向索引） 已定项 4 的判据它进不了单元的自包段。一条树表条目上有**两个各自独立的 8 字节 txg 字段**——诞生 txg（这条自己发布的号）与 `previous_snapshot_txg`（它前驱那个快照的号），对活头条目与快照条目都有定义。
- **④ 兼容档取「树表条目自带长度」这一层，不取「索引节点每条目自带长度」那一层**：前者让旧读者**跳过**一棵不认识的树；后者让旧读者**解开**新树的节点，而**今天没有消费者**——它不是被判成本高，是被判**买不到今天要的东西**。
- **㊀ 旧写者对不认识的记账行**（限「树 ID 不认识、统计量标签认识」这一半）：**写者必须把这类行原样带到新代**（值不变、只换代号；有前端的写者把这类行当前端条目写、随发布排空到叶，第一版写者没有前端（已定项 10）、直落叶，盘上布局相同、读不错）。
- **㊁ flags 其余 15 位**：**未知位一律拒收——该条目 EIO**（对象级，不拒整个池）。**不取**「忽略未知位、继续按已知语义解释」，那是静默误读。配两条纪律：**必须被旧读者看见的改动走系统配置 incompat 位，不许塞进每条目的 flags**；**checker 不许走「跳过」**——I-3.1（已分配统计对得上） 逐字「已分配空间统计 == 实际遍历**所有**引用得到的和」无限定，checker 跳过一棵不认识的树，遍历和就短一截，判出来的是假阳性红而不是漏检，它遇到不认识的树的正确行为是**报「无法完整遍历」并停**。
- **排序契约与嵌套层数**：**条目按树 ID 升序排；嵌套层数不设格式上限**。排序取树 ID 的理由：它是五元组、记账 key、容器索引里到处用的那个键，且 ② 让它随创建序严格递增。不设上限的理由：设上限等于多一个失败模式，而那个上限还随条目宽变。
- **平局判别子取 `(诞生 txg, 树 ID)`，零字节**：D16（发布语义） 已定要用 `(txg, 快照 id)` 打破并列，而 ② 改住根记录之后树 ID **严格随创建序递增**（水位累计、只增不减，崩溃只烧号不回退——I-7.8（根记录树 ID 水位不低于全池最大树 ID) 把这一条变成可判的）⇒ `(诞生 txg, 树 ID)` 就是与创建序一致的全序，而树 ID 本来就在条目里。
- **头 ID 8 字节的取值**：inode 树条目写自己的树 ID（65537），extent 树条目写它服务的那个头（65537），`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537），谱系树写 0（全池一份），logged_ops 写 0（字段表未定，C364（整理意图记录没有字段表）），分配记录树 / 记账树 / livelist 共享树 / 稀疏旁表树 / deadlist 树写 0（无归属，与 D5（快照 / 空间记账机制） 给「全池 / 无归属」行的约定同一个数）；中央映射树不进树表。见 D5（快照 / 空间记账机制） 已定项 9。 ⚠️ 括号里的 65537 是树 ID 从 mkfs 水位 65536 起发的那一版（新池新建文件）上的取值，判据是关系、不是字面值：inode 树条目的头 ID 等于它自己的树 ID；extent 树、`deleted_inodes`、墓碑树条目的头 ID 等于同一张树表里 inode 树条目的树 ID；谱系树、logged_ops、分配记录树、记账树、livelist 共享树、稀疏旁表树、deadlist 树写 0。树 ID 不是从 mkfs 水位起发的那一版（水位已被抬过，或一个克隆头的 inode 树）上，头 ID 随这张表里 inode 树的树 ID 走，不再是 65537（D5（快照 / 空间记账机制） 已定项 9「可写头的 ID = 该头 inode 树的树 ID」）。判它的是 I-9.27（树表条目的头 ID 取值）。
- **预留 76 字节**：已知认购者按「一段字节只有一个意思」排，**逐条登记在树表条目预留的认购表里**（门禁 doc-registries 的 tree-table-reserve 格按它算和：认购合计不许超过 76，表里每条要指得到一个还开着的欠账或一条已定分项）。合计 48，余 28。

<!-- gate:tree-table-reserve total=76 -->

| 认购者 | 住哪条条目 | 宽度 | 出处 |
|---|---|---|---|
| livelist 每头计数器 | 头条目 | 8 | C271（livelist 的 FREE 在什么时刻记没有定义） |
| 第二道销毁闸的活快照计数 | 头条目 | 8 | C272（第二道销毁闸没有 O(1) 的检查法） |
| origin 链接 / 克隆时刻 | 头条目 | 8 | C271（livelist 的 FREE 在什么时刻记没有定义） |
| 快照谱系的区间标号 | 快照条目 | 16 | C144（快照谱系的区间标号没有载体） |
| `children`（子快照计数） | 快照条目 | 8 | D5（快照 / 空间记账机制） 已定项 18 |

⚠️ **inode 号水位不在表里**：C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 2026-09-16 定案「不搬水位，水位照旧住记账」（D5（快照 / 空间记账机制） 已定项 4 第 12 项）⇒ 它不认购这里的字节。

**改新池新建文件的字节：是**——树表单元 t12 的条目宽字段（200）、声明长度（11 × 200 = 2200）与 11 条条目的排布。

**射程**：⚠️ **㊀ 的射程只到「树 ID 不认识」那一半，另一半的处方是相反的**：不认识**统计量标签**那一半由 C71（不认识的统计量标签静默忽略） 管，而它逐字要求「挂载时遇到不认识的统计量标签**必须按 compat_ro 拒绝可写挂载**」——㊀ 不覆盖也不推翻它。两半分得开的机制依据：D5（快照 / 空间记账机制） 已定项 5 的 key 定宽 ⇒ 不认识标签**含义**的写者照样能按固定偏移切出各段；成立的机制是已定项 1「记账条目取幂等完整值」，一条行的值对搬运者不透明也搬得动，而写者不认识那棵树就改不了它 ⇒ 值本来就不变。不搬的代价：D5（快照 / 空间记账机制） 已定项 2「只保留最近 K 代」⇒ 没人重写的行过 K 代消失。⚠️ **暴露面今天很窄**：只有一个统计量带树维（D5（快照 / 空间记账机制） 已定项 4 第 12 项，只为可写头维护）⇒ 不是可写头的树一行都没有；再加一个带树维的统计量，不搬的丢失量就按树数长。⚠️ **㊁ 拒收不丢数据依赖 ④ 的条目自带长度**：条目 EIO 但**位置还在** ⇒ 写者知道「那儿有一棵树我读不了」⇒ 不做跨它的 deadlist 合并；没有长度字段就定位不到下一条，这条推导不成立。⚠️ **㊁ 与本工程对另外两个 flags 字段已定的政策是同一条**：D18（块里携带什么信息）「偏移 7 是 8 个 flag 位……读者遇到非 0 位一律拒收……本工程不给这 8 位留「忽略陌生位」的通道」；已定项 6「flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ **该记录 EIO**」，已落成 I-9.7（记录三段恒零）⇒ 三个 flags 字段并成一条，不用再解释它们为什么不同。⚠️ **「位」和「种类」是两根轴**：D15（格式冻结政策） 已定项 2 那条 compat（新增 keyspace 对旧读者只是看不到新树）管的是**种类**，㊁ 管的是**位**——一棵旧读者自以为认识的树被加了位，它的语义变了，那不是加法。⚠️ **嵌套层数不设上限的代价是性能悬崖**，按 `.claude/rules/fs-design.md` 硬要求 4 必须可观测：「每次发布 COW 的树表单元数 = 层数」要能在运行时读出来——走错层数不报错，没有观测手段就永远发现不了。⚠️ **① 交付的那张表不够用**：D5（快照 / 空间记账机制) 点名的「全部根的 txg 列表」写在 D6（快照实现模型） 已定项 1（每头一棵自己的树）之前，而 D6（快照实现模型） 用一个反例打穿了它的前提（主线 txg=10 建可写快照 X、在 X 里于 txg=30 写一个块、主线 txg=35 建快照 Y，按 D5（快照 / 空间记账机制） 的条件判「Y 引用它」而 Y 根本看不见它）⇒ 要的不是「每个根一个 txg 标量」，是**谱系**；诞生 txg 这 8 字节仍然非有不可（I-3.5（引用区间的精确性） 要按 txg 升序枚举全部快照根与活头、I-3.6（deadlist 紧界） 要 `prev(S).txg`），欠的是**在它之上再加一层标号**（嵌套区间标号是两个数，而且随谱系变化要回头改写，与「诞生 txg 写一次不再动」是完全不同的格式性质）。同一个前提的三个消费者里定案只兑现了 I-3.7（活快照可枚举） 一个：I-3.5（引用区间的精确性） 的可判定形式建在「全部根按 txg 排成单一全序」上，谱系成树之后那一条恒假；D18（块里携带什么信息） 的代际回收走同一个区间测试，方向是过判 ⇒ 少回收 ⇒ 泄漏。⚠️ **③ 只补了活头头部的半格**：D5（快照 / 空间记账机制） 逐字「每个可写头各持一份 `previous_snapshot_txg` **与一份 deadlist**」「建快照时把**活头的 deadlist 整个交接给新快照**，活头换一个空的」⇒ 快照条目也要能指到一份 deadlist，而 deadlist 的盘上落点全仓一处都没有。⚠️ **丙-2 唯一盖不住的那一格判为不是错误**：某个 checkpoint 分配了树 ID 42、写出带 42 的单元头，根没写成就崩了 ⇒ 水位随那个根一起丢 ⇒ 下次挂载重发 42，而盘上留着带 42 的孤儿单元；判它不构成错误的依据是孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」）、I-1.2（块头写序已发布） 逐字「判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留」、扫描重建的两道闸与孤儿回收条件都不看树 ID、孤儿与活版共用五元组本来就是常态（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 加 10 字节写序正是为此）、D6（快照实现模型） 判据 9 的理由句说的是「树里的 key 变成可见」而孤儿不在任何树里。由此 D6（快照实现模型） 判据 9 的射程按它自己的理由句收窄成「**已发布**的快照标识单调不回收」（2026-09-06 用户定案，同一次）。⚠️ **inode 号水位另给答案**：D5（快照 / 空间记账机制） 已定项 4 第 12 项逐字「**带**（只为可写头维护）」⇒ 每个可写头一个标量、条数随头数长，而根记录也是定长槽 ⇒ 丙-2 同样搬不过去；2026-09-16 定案不搬水位，把唯一性收窄到「已发布的 (inode 号, 对象出生代) 不复用」（已定项 6）。⚠️ **树表容量在两处按不同条目宽算**：E79（根记录的容量） 与 D22（单元原子性怎么合成） 已定项 7 报的每层棵数是另一个模型（它把树表条目当成「树 ID 8 + 根指针 59」= 67 字节），方向不变（间接层躲不开、超过一层再嵌一层），只是门槛比 D22（单元原子性怎么合成） 正文写的低。⚠️ 预留里那两笔已知认购（deadlist 的落点、快照谱系的嵌套区间标号）**没有顺手命名成字段**——命名等于把两个未定项一并定了，那是两个决定。排序契约（① 树 ID 严格升序，② 合发号次序）由 I-9.16（树表条目按树 ID 严格升序且合发号次序） 判：重建与冷走读在读任何一棵树之前拒，池级 checker 另判一份（实审 A3b Q2 与树表排序实现，用户 2026-09-27 定立不变量并同步）。

**依据**：

- E112（旧写者遇到不认识的树）：跑前写死判据与接受条款下六格的静默结局与搬运代价——`Skip + Carry` 在挂得上的那些里零静默结局，`Ignore` 那一格记录被按错编码解释，`Refuse` 全挂不上且与 D15（格式冻结政策） 已定项 2 的 compat 档不相容；搬运代价每代只多写两行，跑前定的作废阈值未触发。
- E113（不认识的树·补齐臂集）：臂集补成 5 flags × 3 记账、选法改成跑前写死的字典序之后，`RejectEntry` 在全部扫描点里一次都没输过，`Skip` 在第 1 关就输（隐形快照使 deadlist 合并认错邻居）、`Ignore` 输在第 2 关、攻击腿提的按位分档 `PerBit` 输在它没有第三种行为——㊁ 由此从「跳过该条目」重定为「该条目 EIO」；记账那一维取 `Carry` 不取 `Exempt`（`Exempt` 要改 D5（快照 / 空间记账机制） 已定项 2，那笔代价一个格子都没占）。
- 用户定案 2026-09-06：四问（快照进树表、树 ID 水位住根记录、`previous_snapshot_txg` 每头一份、兼容档取条目自带长度）、㊀ ㊁、排序契约与嵌套层数、平局判别子、预留取 32 字节（原话在变更史）。
- 用户定案 2026-09-23：② 的 max 也并上环里记录新根段带的水位（C342（树 ID 水位在根读不出时退回去重发） 前置那一问；C511（回退到无文件那一版之后诞生代怎么接） 第 3 步的实现员先选了这一边，用户同日认下，原话「journal 日志就是为了保证树能完整写入的」），原话在变更史；发号次序是主 agent 同日照格式常量次序定的，被攻过零轮。
- 用户定案 2026-09-14：树 ID 挪到条目最前、根指针 83 → 86、预留里切 8 字节做头 ID（原话在变更史；依据是已定项 11 定码 2 条目里 key 一律是条目的前 key 宽 个字节，树 ID 不打头就要给通用解码规则加例外）。
- 用户定案 2026-09-16：条目从 148 加宽到 200、预留 76，代价是每单元从 109 棵降到 81 棵、二层封顶从 11881 降到 6561，用户知情接受（原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K1：头 ID 读同表关系、不读字面 65537——字面读法在一段合法的高水位历史上误红 4 条，还放行头 ID 写死 65537 的坏实现（攻方在副本上量过，主 agent 复跑坐实）；判它的 I-9.27（树表条目的头 ID 取值） 按表判，树表有条目时才判。

**欠**：C144（快照谱系的区间标号没有载体）、C145（deadlist 在盘上没有落点）、C147（树 ID 水位的不变量还没有）、C148（树表条目变宽把间接层门槛压到二层）、C149（根槽重发之后 birth 口径松一格）、C156（记录编码误读没有不变量）、C157（树表容量在两处按不同条目宽算）、C158（checker 跳过不认识的树会判假阳性）、C159（E112 的臂集缺三条且天平不含挂不上的代价）、C160（记账臂的真实代价不在天平上）、C342（树 ID 水位在根读不出时退回去重发）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:285-316`（整段抄，未转述）**

```markdown
#### 已定项 9：树的种类码的登记表

**定案**：树表条目里「树的种类」那 2 字节的取值，唯一登记位是已定项 9 这一节的登记表；**新树只追加不重用码**，0 保留为「无」。

| 码 | 树 | 出处 |
|---|---|---|
| 0 | 无（树表条目不用它） | — |
| 1 | extent 树 | D4（校验和位置） 已定项 7 |
| 2 | inode 树 | 已定项 6 |
| 3 | 分配记录树 | D3（空间分配） 已定项 3 |
| 4 | 记账树 | D5（快照 / 空间记账机制） 已定项 3 |
| 5 | 中央映射树 | D19（块指针的结构与宽度预算） 已定项 5 |
| 6 | livelist 共享树 | D6（快照实现模型） 已定项 2 |
| 7 | 稀疏旁表树 | D5（快照 / 空间记账机制） 已定项 6 |
| 8 | deadlist 树 | D5（快照 / 空间记账机制） 已定项 11 |
| 9 | `deleted_inodes` 树 | 已定项 16 |
| 10 | 墓碑树 | D18（块里携带什么信息） 已定项 24 |
| 11 | 谱系树 | D18（块里携带什么信息） 已定项 24 |
| 12 | logged_ops 树 | 已定项 5 |

新池新建文件写出的 **11** 条树表条目照此表（码 1..4、6..12 各一条；中央映射树的根住根记录、不进树表，D19（块指针的结构与宽度预算） 已定项 11；码 5 照登记）；E142（新池新建文件的干跑） 的 `TREE_KIND_*` 常量是码 1..8 那八个值（码 9..12 它没有）；`crates/singlefs-core/src/records.rs` 与 checker 的 `walk.rs` 用同一组常量（码 5 不进树表，那里没有它）。

**射程**：它只登记「树的种类」这一段的取值，树 ID 是另一个量，两个量的值域不相交（树 ID ≥ 65536，取值在已定项 11）。与 D18（块里携带什么信息） 已定项 11 的单元类登记表是两张表、两个概念（用户不取合并）。码 8（deadlist）与 livelist 共享树、稀疏旁表同一个先例——都是「销毁快照时要读、而销毁不许全盘扫描」的结构，day-1 注册进树表比第一次建快照时惰性创建少一条「首次建树」路径；条目形态另立 D5（快照 / 空间记账机制） 已定项 12。**改新池新建文件的字节：是**——加码 8 那一次让树表从 6 条变 7 条、树 ID 水位从 18 变 19。

**依据**：

- 无实验：码值是取值约定，没有可量的量；登记表由用户 2026-09-13 定案（码 0..7）、2026-09-14 定案加码 8，两次都没有走三方论证、可推翻，原话在变更史。
- 码 9..12：判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节种类码那一行（两腿各自得出）；树 ID 与种类码值域不相交由用户定案 2026-10-01 取（岔路单 `research/prompts/m3-blanks-r1-forks.md` 的「种类码与树 ID 的错开」一行，原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2：新池新建文件写出的树表条目数按实现现值（11 条，`crates/singlefs-format/src/lib.rs` 的 `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES`）。

**欠**：C315（树的种类码没有登记表）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:317-328`（整段抄，未转述）**

```markdown
#### 已定项 10：第一版记账直落叶，`seq` 的取值规则

**定案**：**第一版记账直落记账树叶，不走 write buffer 前端**；D5（快照 / 空间记账机制） 已定项 3 第 3 条读成「前端是后续优化」。**`seq` = 同一个 `(标签, 树, 设备, 代)` 在一次发布里被写进树的第几次**（不是运行期那个统计量被改的次数），**从 1 起**；第一版禁边插边 COW（D19（块指针的结构与宽度预算） 已定项 12 的固定点：每个码 2 / 码 3 一个实例一个 checkpoint 只写出一次，发布末尾一次性 COW）⇒ 每个 key 每次发布只写进树一次 ⇒ **恒 1**；合并规则取最大值的统计量也照写。

**射程**：它定的是第一版走哪条路与 `seq` 怎么数，不定 `seq` 的宽度与落点（已定项 7）、也不定前端与节点缓冲的衔接（已定项 12，第一版两级都不开）。放开固定点、开前端或节点缓冲之后「恒 1」还成不成立，由已定项 13 的状态折叠接着管。

**依据**：

- 无实验：第一版走不走前端是实现排期的选择，没有可量的量；由用户 2026-09-13 在写代码前的总审核里定案，没有走三方论证、可推翻，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:329-361`（整段抄，未转述）**

```markdown
#### 已定项 11：码 2 的载荷布局与 key 比较序

**定案**：码 2 节点头自描述（预留 2 之后是条目数 2 + 条目宽 2，key 宽 1 排在 key 区间之前，偏移表见 D18（块里携带什么信息） 已定项 18），基础字节数 81 → 86；含 29 字节预留位后头 = `115 + 2 × key 宽`。条目区从含预留位的头末尾（`115 + 2 × key 宽`）起，**定宽条目连续排列**，**声明长度 = 条目数 × 条目宽**；条目之后的空闲字节恒 0 且参与载荷 CRC。载荷 CRC 覆盖 `[86 + 2 × key 宽, 16384)`（起点是明文头末尾，口径与码 1 的 105、码 3 的 107 同）。**条目里 key 一律是条目的前 key 宽 个字节**——「读条目宽 → 按条目宽切条目 → 取前 key 宽 个字节当 key」是一条对全部码 2 树都成立的通用解码规则，扫描重建路径上不用另查一张「这棵树的 key 在条目里排第几」的表。key 的全序按逐字段无符号整数、字段自左向右比较，**盘上小端存储不构成 memcmp 序**。树 ID 从 65536 起编（种类字段 2 字节取不到的最小值，两个量的值域永不相交）：extent 65536、inode 65537、分配记录 65538、记账 65539、中央映射 65540、livelist 65541、稀疏旁表 65542、deadlist 65543、`deleted_inodes` 65544、墓碑树 65545、谱系树 65546、logged_ops 65547；mkfs 时水位 65536，第一个文件版本那次之后 65548。

**改新池新建文件的字节：是**——六个码 2 单元的条目区起点（`115 + 2 × key 宽`：inode 树根 131、extent 163、分配记录 135、记账 159、映射 169、树表 131）、声明长度，以及树表条目的字段次序；树 ID 字段（树表条目里的树 ID 与头 ID、码 2 头里的树 ID、记账 key 的树 ID 段）与树 ID 水位的值也变，随树 ID 从 65536 起编。

**多层码 2 树怎么长、怎么收**（记账树、中央映射树，以及照已定项 14 仍用码 2 btree 的树）：① 一次发布里分裂新建的节点与别的单元落在同一段，整条根到叶的路径照常 COW，不加写序步骤、不加屏障；② 叶装不下时从中间切；③ 收缩只摘空节点，根只剩一个孩子时降高一层，不做「低于一半合并」与借条目；④ 分隔 key 跟着维护：插到最左分隔 key 之下时把它压低，条目在兄弟间挪动之后把右边那个孩子的分隔 key 改成它新的最小 key；逐层可判：分隔 key ≤ 孩子节点头的最小 key，且 > 左邻孩子节点头的最大 key（D18（块里携带什么信息） 已定项 18 的 key 区间）；⑤ 树高 = 根节点码 2 头里的层级 + 1。

- **一次发布里先删后插**：上一版有、这一版没有的 key 先按 key 升序删，再按 key 升序插这一版新有的；只决定形状，不决定写序（写的是最终态，已定项 13）。
- **从盘上重建的树分隔 key 坏了**：重建只核拼得成树的几样（层级、节点不空、内部条目宽、不重复引用），不核分隔 key；下一次发布按分隔 key 删不掉时规划交回，可写挂载在取号之前拒，盘上不变。遇到这种盘之后怎么办（修复、只读挂载）没定。
- **叶条目宽不由读树判**：读树只切 key，叶条目窄于字段表由解条目的一方各报各的；内部条目窄于 key + 86 由读树报。

**内部节点的条目 = 本树 key + 子指针 86**：记账树 22 + 86 = 108、中央映射树 27 + 86 = 113。分配记录树与 extent 树按已定项 14 另起按 key 空间寻址的结构，结构在三方 `m2-keyspace-r1` 判；照码 2 btree 做时它们的内部条目是 96 与 110（extent 110 是用户 2026-09-24 早先定的）。

<!-- format-const: ACCOUNTING_INTERNAL_ENTRY_BYTES = 108 -->
<!-- format-const: CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES = 113 -->

**射程**：它管码 2 节点的载荷怎么排与 key 怎么比，不管头里各字段的偏移（D18（块里携带什么信息） 已定项 18）、也不管每棵树的条目里除 key 之外还有什么（各树自己的分项）。唯一因「key 打头」需要重排的是树表条目，已按这条改成树 ID 打头（已定项 8）。「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」在中央映射条目回到定宽 55 之后不用动（D19（块指针的结构与宽度预算） 已定项 10）——条目宽字段只有一个 u16，按类两宽时这两句顶不住。两条 2026-09-14 补的定案都是用户弹窗定案，没有走三方论证，可推翻。⚠️ 分裂、收缩、分裂点、写序四样被攻过零轮，三方原型只抽了一次样；「低于一半合并」与「追加时末尾切」在崩溃历史上与选中的那两样零对零，没选的理由是用户的取舍，不是安全性。「分裂单开一次发布」出局：只改结构的发布挤掉 D16（发布语义） 已定项 1 的「最近 4 个可退到的不同状态」。

**依据**：

- E145（码 2 自描述头与映射树 key 宽的代价）：量的正是码 2 头自描述要加的那几段与映射树 key 宽的代价——加到 86 之后扇出、树高、解头那几节都不掉档，自描述值得做。
- 用户定案 2026-09-13：码 2 头自描述、基础 86、条目区起点、载荷 CRC 覆盖范围、key 比较序（原话在变更史）。
- 用户定案 2026-09-14：条目里 key 一律是前 key 宽 个字节；「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」原样成立（原话在变更史）。
- 用户定案 2026-09-24：extent 树内部条目 110 字节（原话在变更史）；E157（并行线一两条条款的计数模型） 在 110、136 两个取样点上结论都不翻面。
- 无实验：分裂、收缩、分裂点、写序四样是三方原型（`research/prompts/m2-treesplit-r1-opus-model/`）在同一批崩溃历史上零对零之后由用户选的，没立实验号；分隔 key 跟着维护是那一轮主 agent 按打中定（原型 t1.out：沿用 D8（核心索引结构） 已定项 6 的「分隔 key 创建后不变」24 格全红、跟着维护 24 格全 0），判决 `research/prompts/m2-treesplit-r1-main-verification.md`。
- 用户定案 2026-09-24：写序同段照常 COW、只摘空节点且根剩一个孩子时降高、从中间切、内部条目 = 本树 key + 子指针 86（原话在变更史）。
- `research/prompts/m2-treesplit-implementer-report.md` 第五节 B1、B5、B6。
- 用户定案 2026-10-01：树 ID 整体挪到 65536 起，与种类码的值域不相交（岔路单 `research/prompts/m3-blanks-r1-forks.md` 的「种类码与树 ID 的错开」一行，原话在变更史）。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节种类码那一行：新树的种类码取 9..12（两腿各自得出）。

**欠**：C306（码 2 头不自描述 key 宽）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:362-375`（整段抄，未转述）**

```markdown
#### 已定项 12：write buffer 前端与节点消息缓冲区的衔接 —— 一棵树只准一级

**定案**：走 write buffer 前端的三棵树（记账 / 反向索引 / LRU）**ε 恒 0、内部节点不留消息缓冲区**；这三棵树上的写**一律进前端**——含已定项 8 ㊀ 那类旧写者搬运的记账行：有前端的写者把它当前端条目写，第一版写者没有前端（已定项 10）、直落叶，盘上布局相同、读不错；**发布前把前端排空到叶**，落叶时同 key 比同一窗口的到达序；发布集的前端条目在同一次发布里排空、按事务切，不跨发布（fsync = 一次全量发布）。其余树按 D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴留不留缓冲。**第一版两级都不开**（已定项 10、D11（索引节点要不要留消息缓冲区） 已定项 7），不改新池新建文件的字节。

**检查**：前端树的写路径只暴露前端入口（类型层）；运行时断言——一个 checkpoint 里前端树叶上的每一处逻辑改动（key → 值）都对得上一条这个 checkpoint 进过叶的前端条目，窗口中途 flush 的与发布时排空的都算、实例切换重发时保留；重组、索引重建、损坏恢复重建三个例外各配一条「前后每个 key 读出来的值不变」。它只能是运行时的：盘上分不出经前端排空与直写落叶。按字面写的三种断言（只认发布时排空的、只认这一次尝试的、按物理新叶数）在合法历史上会误报（`c161-r3-model-output.txt` 的 `G3_publish_only` / `G3_attempt` / `G3_physical` 行）。

**射程**：**代价**：每次发布排空到叶的写出是「排空到根缓冲」那类形态的 3.4–10.7 倍（覆盖写负载 4.8–10.6 倍，同一产物 `part` 列的第三段），发布集的条目都得在这次发布里落叶、不能按预算分批；同一份脏页预算下前端装的条目少一个量级——`T_dirty` 按什么算没有口径。**买到的**：前端树上没有同 key 跨级并存，读的次序最短；陈旧检出（C52（陈旧当成合法））在这三棵树上没有对象。**没出局但没选的**：甲-R4（前端直落叶、节点缓冲只收不走前端的写）与乙-D（前端进根缓冲逐层下推、按位置定新旧）各只在塌缩上被打中一次，它们的陈旧检出要每条消息带发布号（与已定项 7「不许再放一份 txg」冲突），节点布局进 D15（格式冻结政策） 第 4 层。**连带**：重组时节点缓冲怎么搬（射程是全部留缓冲的树，这一项只省了前端三棵）；单调统计量的点删遇墓碑（D5（快照 / 空间记账机制） 已定项 3 第 2 条）；反向索引 / LRU 的时间线索；D11（索引节点要不要留消息缓冲区） 已定项 6 树轴的分支变量与已定项 5 的措辞。

**依据**：

- 无实验：这一格量的是设计形态在合法历史上会不会被分辨臂打中，依据是三轮三方（`research/prompts/c161-r1-main-verification.md`、`c161-r2-main-verification.md`、`c161-r3-main-verification.md` 各自第四节）——这一形在三轮试过的全部形状（乱序 flush、K 代点删、分裂与合并、部分发布、排空做到一半崩溃、删除、树高 4、索引重建、实例切换、管理员回退）上没被打中；出局的乙 + seq 全池单调两轮被打中（计数器不落盘；实例切换时计数器从所选根重新载入），还要 8 字节 seq 与落盘计数器。用户 2026-09-16 收尾弹窗定案，原话在变更史。

**欠**：C161（缓冲两级并存而衔接没定）、C344（单调统计量的点删遇墓碑没有会红的检查）、C345（反向索引 / LRU 的陈旧检出没有时间线索）、C348（T_dirty 按什么算没有口径）、C349（重组时节点消息缓冲区怎么搬没有条款）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:376-387`（整段抄，未转述）**

```markdown
#### 已定项 13：索引树更新做状态折叠 —— 同一次发布里同一个 key 的中间状态不落叶

**定案**：一棵索引树上的同一个 key 在同一次发布里被改了多次（A → B → C → D → E）时，落叶只做一次：叶里这个 key 的条目直接写成最终态 E，A–D 不写进任何节点。「同一周期」是一次发布：同一个 `checkpoint_txg`，fsync 触发的也是一次发布（D16（发布语义） 已定项 6）。**E 按同一次发布里的到达序取最后一次改动，不按值取**；E 是墓碑时，折叠的结果就是墓碑，不许把墓碑当数值与 A–D 一起取最大（C344（单调统计量的点删遇墓碑没有会红的检查） 实测这种朴素折叠四条候选 16 / 16 丢点删，`b.out`）。到达序在内存里由条目自己带着（已定项 15 第 3 条），不靠按 key 排序之后的相对位置。

**射程**：**全部索引树的全部落叶路径**。第一版的直落叶（已定项 10）之外，以后开的 write buffer 前端排空（已定项 12：落叶时同 key 比同一窗口的到达序，比出来的最后一条就是 E）、节点消息缓冲区（D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴）、边插边 COW（D3（空间分配） 已定项 5 允许、D19（块指针的结构与宽度预算） 已定项 12 第一版不取的那条实现）都照这个规矩落叶。**第一版已经满足它**：D19（块指针的结构与宽度预算） 已定项 12 的固定点让已定项 10 推出「每个 key 每次发布只写进树一次」、`seq` 恒 1；已定项 13 把这件事从固定点的推论提成独立条款，放开固定点之后照样成立，不改新池新建文件的字节。实现今天没有对象：`crates/singlefs-core/src/transaction.rs` 的 `publish_overwrite` 一次覆盖写就是一次发布（txg、事务号各加一），没有「同一次发布里几个事务改同一个 key」的路径；同一个 checkpoint 里装两个对象的用例改的是不同的 key。⚠️ **没答的两条**：① A–D 在树之外的去向不归它管——A–D 是同一个逻辑块被覆盖写几次得到的数据单元版本时，按 D16（发布语义） 已定项 2 的丢失窗口与切分纪律「一事务一单元」，A–D 各是一个事务，各有一个落了盘的数据单元与一条 journal 记录（D23（journal 的角色与格式） 已定项 7：一条记录一个事务号），这几个单元没有根引用、也不在 I-7.4（近 K 代块未被复用） 的保护里，它们要不要写、写了在同一次发布里怎么释放、记账怎么算，决策正文、欠账表、不变量清单里都没有管它的条款；② 已定项 7 的盘上 `seq` 在折叠之下恒 1（已定项 10 的「恒 1」从第一版扩到所有版本），而认 E 用的是内存里的到达序，盘上那 4 字节 `seq` 还拿不拿来当 `T_dirty` 上界的依据（已定项 7 定案里「`T_dirty` 的上界从此是挂载时判定」那一句），没答。⚠️ 「单调统计量在同一次发布里不减、所以两种取法给同一个数」是主 agent 的推论，没有检查。

**依据**：

- 无实验：用户定「这个是决策 记录 暂时不需要做和验证」，没开三方、没建实验，也没有可量的量；用户 2026-09-19 定案与同日对「同一周期 = 一次发布」的确认，原话在变更史。A–D 可以不落的三条支点是已有条款（主 agent 2026-09-19 现查）：D16（发布语义） 已定项 4 与 D23（journal 的角色与格式） 已定项 14 第五条（施加的单位是一次发布，合法前缀停在一次发布的两个事务之间时那次发布整体不施加，所以 replay 出来的只有发布边界上的状态）、D16（发布语义） 已定项 6（fsync 触发的也是一次发布，被 fsync 确认过的状态都落在发布边界上）、D16（发布语义） 已定项 14（快照发布点对齐 checkpoint 边界、钉在这个 checkpoint 全部数据变更之后，没有快照引用 A–D）。

**欠**：C383（同一次发布里同 key 的中间态折叠没有检查）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:388-441`（整段抄，未转述）**

```markdown
#### 已定项 14：一套 btree 实现 + 多个独立 keyspace + 两个前端，派生树可以异构

**定案**：**一套 btree 实现 + 多个独立 keyspace + 两个前端，管权威态的树与照分裂做的树；派生态的分配记录树与 extent 树（D21（权威态与派生态的分界） 已定项 5、9）可以另起按 key 空间寻址的结构，格式可以与码 2 btree 不同。**

| 层 | 做什么 |
|---|---|
| 独立 keyspace | 每类数据一棵树（extent / inode / dirent / 反向索引 / 空闲空间 / 记账 …），互不干扰 |
| 日志结构节点 | 节点内新 key 追加不插入，内存里定期重排；写顺序、可在只读锁下回写 |
| **write buffer** 前端 | 高频小更新（反向索引、LRU、记账）批量攒起来排序去重后批量插入 |
| **key cache** 前端 | 同一批 key 反复读写的（分配信息）走哈希表，不做全树查找 |

**两棵派生树的结构**：

- **分配记录树**：按绝对槽号按位置寻址。每片叶罩一段固定的槽 `[k × W, (k + 1) × W)`，W 取偶数（两槽数据单元永远不跨叶），且 W ≤ 叶条目容量 812（一条记录至少罩一个槽，叶永远装得下）；记录按起点槽落叶。往上各层按位置寻址，子节点罩哪一段由它在父节点里的位置算出。**没有记录的一段 = 全空闲**：那片叶不写，父节点那一格留空（用户 K1）。checker 逐节点核「这个节点罩的就是它的位置规定的那一段」与「记录的末槽不越过它所在叶的末槽」。
- **extent 树**：两段，都按位置寻址。上段按 inode 号的位置寻址；下段一个文件一棵，按数据单元号的位置寻址（叶罩 144 个单元，内部扇出 147），洞就是缺席。**只有一个数据单元的文件不建下段**：上段叶条目里直接放那个数据指针；条目带一个标签字节区分「没有单元 / 下段根指针 / 内联数据指针」（用户 K2）。
- **挂载怎么读**：分配记录树挂载时整棵读进挂载态（分配器本来就要）；extent 树按需，打开文件时按位置走下去（用户 K4）。
- **实现取值**（实二一取，主 agent 2026-09-25 定，被攻过零轮）：
  - **分配记录树**：
    - 叶宽 W = 812（取叶条目容量 (16384 − 135) ÷ 20 本身），内部条目 96 字节（key 10 + 子指针 86），扇出 169。
    <!-- format-const: ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES = 96 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169 -->
    - 层级 L 的节点罩 W × 169^L 个槽。
    - 内部节点与根只写有孩子的格：每个孩子一条内部条目（key 10 + 子指针 86），条目的 key 取这个孩子按位置罩的那一段的起点（孩子所在的盘, 那一段的第一个槽号），按 key 排，没有孩子的格不占条目；读者按这个 key 认孩子，槽号不落在孩子那一层的格点上、盘不在池里、或认出的孩子不在这个父节点名下的，这条条目不指任何孩子（`crates/singlefs-core/src/allocation_record_tree.rs` 的 `child_named_by_entry_key`）。根节点头的 key 区间罩整个 key 空间，smallest_key = (0, 0)，largest_key = (0xFFFFFFFF, 2^48 − 1)。
    - 根罩整个 key 空间、按盘分流；根层取最小的 R ≥ 1，使 Σ_盘 ⌈盘上槽数 ÷ (W × 169^(R−1))⌉ ≤ 169。4 GiB × 2 时根在第 2 层、树高 3。
  - **extent 树**：
    - 上段叶条目 113 字节 = key 24（(0, inode, 0)）+ 标签 1 + 载荷 88。标签 0 全零，只有读者认、写者不写；标签 1 是下段根的节点指针 86 字节加 2 字节零；标签 2 是那一个数据单元的数据指针 88 字节。
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES = 113 -->
    - 上段叶罩 143 个 inode，内部条目 110 字节（key 24 + 子指针 86），扇出 147。
    - 上段叶节点头的 key 区间照 D18（块里携带什么信息） 已定项 2 射程段写这片叶按位置罩的那一段，不写叶里第一条与最后一条条目的 key：闭区间，第 k 片（从 0 起）smallest_key = (0, 143k, 0)，largest_key = (0, 143k + 142, 2^64 − 1)。
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_INODES = 143 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_ENTRY_BYTES = 110 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_FANOUT = 147 -->
    - 下段叶罩 144 个单元，下段 key 的字节偏移 = 单元号 × 32634。上段与下段的内部条目，key 都取这个孩子按位置罩的那一段的起点（`crates/singlefs-core/src/extent_tree.rs` 的 `build_extent_internal_entry`）。
    <!-- format-const: EXTENT_TREE_LOWER_LEAF_DATA_UNITS = 144 -->
  - **发布路径的固定点**：猜重写集 → 在分配器拷贝上走释放（释放弄脏的叶先并进重写集）→ 取落点 → 算脏节点 → 不在集合里就并进去重来。第 64 轮还没收拢，就在任何落盘之前返回 `AllocationRecordTreeRewriteSetDidNotSettle`；64 没有依据，见新立的欠账。
  - **盘上槽数**：取盘上整槽数，盘尾不足一槽的零头不算；写侧与读侧只有一处定义（`crates/singlefs-core/src/allocator.rs` 的 `absolute_slot_count_of_device`），分配记录树根层两侧同一个数（实二六取，主 agent 2026-09-25 定，被攻过零轮；用例 `half_a_slot_past_a_root_level_boundary_leaves_the_writer_and_the_reader_on_the_same_root_level_and_every_tree_written_reads_back`）。
  - **打开文件时自报的读数**：`ExtentTreeReadsAtOpen` 数的是读了几个 extent 树节点，不是块层的设备读次数；提示过期、经中央映射回退时每跳多 2 次设备读，两个数不相等（实二六查出，主 agent 2026-09-25 认这个口径、不另加设备读数，被攻过零轮；用例 `opening_a_file_whose_lower_extent_segment_has_two_levels_reports_as_many_node_reads_as_the_block_layer_counts` 钉相等的那一格，`a_leaf_of_a_two_level_lower_extent_segment_with_a_stale_location_hint_reads_back_every_data_unit_through_the_central_mapping` 钉不等的那一格，都在 `crates/singlefs-harness/tests/tree_nodes_and_the_central_mapping_on_mounted_read.rs`）。
- 两棵树的节点照样带 D18（块里携带什么信息） 已定项 18 的码 2 头（树 ID、层级、key 区间、诞生代号），树高照 D8（核心索引结构） 已定项 11「树高 = 根节点头层级 + 1」读。

**射程**：⚠️ 异构只放给派生态、可重建的两棵树：分配记录树与 extent 树；记账树（权威态）、中央映射树、树表、实例表、inode 树照一套码 2 btree 做（D8（核心索引结构） 已定项 11 的分裂与收缩）。异构结构照样要：单个节点自描述（哪棵树、哪一层、管哪一段 key、第几代，D18（块里携带什么信息））、损坏可检出（D21（权威态与派生态的分界） 已定项 11）、每一种新写序进层 0 崩溃点、checker 另有一套判它的代码。那两棵的结构在三方 `m2-keyspace-r1` 判。⚠️ 「日志结构节点」说的是**内存里的排布**——盘上每次落盘是整单元 COW 重写，按已定项 11 的单一定宽条目数组写（声明长度恰等于条目数 × 条目宽），不是多段追加的 bset。⚠️ **write buffer 前端第一版不实装**（已定项 10：记账直落记账树叶）；前端与节点消息缓冲区怎么衔接在已定项 12，条目要满足的三条硬要求在已定项 15。索引结构可以多样、事务层不许多样那条纪律在 `.claude/rules/fs-design.md`，本项只定索引这一侧。⚠️ 叶宽 W 的具体取值、上段扇出、上段叶条目的完整字段表，条款没写到的由实现员照上面几条取、交回里写明，主 agent 定后补进这一节。⚠️ 三种新写序（长高多层时位置 0 那一串节点同一次发布写、矮下去多层时同一串一起释放、内联与下段互换）要进层 0，checker 判定随实现一起做。⚠️ 这一节的四项都被攻过零轮；「全局按位置＋内联」的崩溃历史没扫（用户定全量崩溃最后统一跑）。⚠️ 被否掉的候选：全局按 key 排（上段要分裂）、每文件一棵根住 inode 记录（要改已冻结的 inode 记录）、全局按位置不内联（一次挂载里连续建到第 14625 个小文件就假性报满，违反 D3（空间分配） 已定项 9 第 1 条）——理由在判决第三节。

**依据**：

- 无实验：一套 btree 实现配多 keyspace 的形态没有可量的量，依据是 2026-08-29 现查 `linux-6.17/fs/bcachefs/bcachefs_format.h` 的 `BCH_BTREE_IDS()` 宏（数 `x(` 行与带 `BTREE_IS_write_buffer` 的行，源码固定在 `~/code/fs-refs/`）：bcachefs 用**一套 btree 实现 + 21 棵树**，其中 **5 棵**走 write buffer（`lru` / `backpointers` / `deleted_inodes` / `rebalance_work` / `accounting`）；比「按访问模式选不同结构」好在**只有一套结构要调对**——异构方案意味着 N 套实现、N 套崩溃一致性 bug、N 套 checker。prior-art.md 的 bcachefs 那一行以这一段为准。
- 用户定案 2026-09-24：派生树可以异构（原话在变更史）；多出来的那套实现、崩溃一致性与 checker 由层 0 流与 checker 各自覆盖，不再拿「只有一套结构要调对」挡。
- 无实验：三方原型（`research/prompts/m2-keyspace-r1-opus-model/`）在同一批历史与代价模型上量过，没立实验号，选哪一个由用户定，判决 `research/prompts/m2-keyspace-r1-main-verification.md`。
- 用户定案 2026-09-24：K1、K2、K4（原话在变更史）。
- 实二一报告 `research/prompts/m2-keyspace-implementer-report.md`：实现员取值，主 agent 认，被攻过零轮。
- 实二六报告 `research/prompts/m2-e26-implementer-report.md` 第六节 Q3、Q4：盘上槽数与打开文件自报读数两条取值，主 agent 认，被攻过零轮。
- E142（新池新建文件的干跑） 第十五次跑第五段（`research/prompts/e142-r15-step5-runner-report.md` 第四节）：分配记录树内部与根节点那两格、extent 上段叶 key 区间那一格，独立装置的变体臂只有这一种读法逐字节对上 crates；这是归因，不是在装置主读法上验过，主读法按这几格改完的重跑在后面。

**欠**：C211（节点落盘形态没有正文）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:442-459`（整段抄，未转述）**

```markdown
#### 已定项 15：write buffer 条目的三条硬要求

**定案**：

1. **排序会摧毁局部性而不是创造它 ⇒ 节点缓存必须是抗扫描的**（2Q / LRU-K 之类）。按叶排序之后每次 flush 就是一次覆盖整个叶子空间的顺序扫描，而 **LRU 对「反复的、大于缓存的顺序扫描」命中率恰好为 0**。**这是硬要求，不是建议。**
2. **条目形态必须是幂等完整值，不许是增量 Δ**。记账那一棵咬得最紧——「释放了 4096 字节」的自然表达就是 Δ，处置见已定项 1 的定案。本工程因此与 bcachefs 分道：其 accounting 树是 delta 形态（`disk_accounting_format.h` 原文「updates are _deltas_」）。
3. **条目必须自带序号（seq），这是格式要求**。flush 时按 key 排序**丢弃时间序**，而去重规则是「后者胜」⇒ **「后者」必须由条目自己携带**；用不稳定排序时同 key 条目的相对次序未定义，去重结果因此不确定。

**射程**：三条管的是 write buffer 这条路上的条目与缓存策略，宽度与落点在已定项 7、取值规则在已定项 10、与节点消息缓冲区的衔接在已定项 12。⚠️ 要求 2 与要求 3 是两条不同的要求：幂等管「重复应用不出错」，序号管「谁覆盖谁是确定的」。⚠️ **已知代价**（bcachefs 原文）：write buffer 的更新无序且最终一致，flush 时按 key 排序丢弃时间序，且这个 sort-merge-sweep **本质单线程**，在多线程负载下它的效率是关键路径。这句有两种读法，实测结果相反——**线程读法**（合并的 CPU 吞吐撑不住多线程负载）**不成立**：单线程合并吞吐按每 4 KiB 用户写 3 条条目算远超盘的带宽，按叶分区还可并行；**I/O 读法**（flush 的树插入 I/O 是关键路径）**成立，且是三项里最大的一项**。⇒ **关键路径上真正卡住的是 D9（加密） 的加密，不是合并**；⚠️ E17（write buffer 合并上限） 只量了合并这一段，不含树插入与 I/O。

**依据**：

- E7（离线索引 harness）：本工程这一侧的 io/op 在缓存占树 2% / 6% / 25% 三档上逐位相同，干预证实保持同叶聚合不变、只打乱组的处理顺序就能让 25% 缓存档的读次数下降，100% 缓存档打乱前后完全相同（阴性对照）——LRU 对「反复的、大于缓存的顺序扫描」命中率恰好为 0，所以要求 1 是硬要求；同一装置量出 flush 的树插入 I/O 折算下来远大于加密与合并，I/O 读法成立。
- E14（I-3.1 的判别力：幂等完整值 vs 增量 Δ）：增量 Δ 形态下 I-3.1（已分配统计对得上） 判别力归零，幂等完整值形态下全抓——要求 2 由它钉住。
- E17（write buffer 合并上限）：第一版漏了 `seq`，两条臂与真值当场对不上，逼出要求 3 的可判定形态；同一实验的单线程合并吞吐与按叶分区的并行倍数把「sort-merge-sweep 本质单线程」的线程读法判为不成立。

**欠**：C15（write buffer 混入 Δ）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:460-488`（整段抄，未转述）**

```markdown
#### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空

**定案**：

1. **意图**：意图那次发布把 inode N 的记录写成 nlink = 0（size 不变），`deleted_inodes` 加 key = N（8 字节、无 value、不带对象出生代），往墓碑树开放容器追加一条墓碑（D18（块里携带什么信息） 已定项 24）；不立 logged_ops（K-U1 L4）。
2. **分批**：从尾部按数据单元个数分批，每批释放尾部 K 个单元、size 改成「剩下单元数 × 净荷容量」；K 产品取 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`（144，自由选取），测试由只供测试的旋钮压小；删除批只重写被截掉那一片下段叶与它的祖先（K-R3-1，extent 写法的新分支，要过 `.claude/rules/fs-design.md` 五条硬要求）；最后一批释放剩下全部单元、删 inode 记录、删 `deleted_inodes` 那条 key，同一次发布；只有这一条路，不设一次做完的快路。意图那次、每一批与续做的每一批各占一个事务号，等于本实例的下一个（与普通发布同一套编号）。
3. **建树**：`deleted_inodes` 在第一个文件版本那次发布零根注册；它是派生态：队列 = inode 树里 nlink = 0 的号集合（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V7）。
4. **读者与写入口**：`InodeRecord` 的 nlink 由 `parse` 读、`to_bytes` 写结构里的值；现造记录的入口拒「nlink = 0」与「没有这条记录」；从意图提交起只读挂载、冷走读、模型读回答「没有文件」；写入口拒 nlink = 0 的 inode（`research/prompts/m3-delete-r2-main-verification.md` 第二节第 1 行、第三节 V6）。
5. **删空**：「有树、文件集合为空」是 `RebuiltVersion` / `PreviousVersion` / `PoolVersion` 的第三个成员，不借 `WithoutFile`；判定 = 树表有条目 ∧ inode 树零条目根（key 区间 `[0, 0]`，照已定项 6）∧ extent 树零条目根（key 区间按位置规定）∧ `deleted_inodes` 根指针置零；不看墓碑树与谱系树。删完之后只能做 `publish_new_inodes`（size 0）、写行与空发布（K-U5 X1）（`research/prompts/m3-delete-r3-main-verification.md` 第三节 G5）。删空的一版上覆盖写、顺序写、unlink 拒成已有的「没有文件版本可改」那一种拒绝（`UserChangeRefused::NoFileVersionToChange`）。
6. **续做**：`mount_writable` 在写行与暖机之后、交回之前把队列走到空；不进取号前预演；会话里的队列每次从现行那一版读、不缓存；崩溃注入加 `WritableMountPhase::Continuation`（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V3）。会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己；挂载续做照旧；unlink 一次调用 = 走空 + 意图 + 全部批（与续做共用同一个走空）。会话走空时删除类发布（意图那次与分批的每一批）被落点那一判以空间拒，推一串抬 F、回收后再试；每一次被拒的删除类发布各算一次准入（一次准入最多 `PUBLISHES_PER_ADMISSION_AT_MOST` = 8 次发布，`crates/singlefs-core/src/mount.rs`）；推满仍放不下交回「抬 F 之后仍没空间」的拒绝（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`），已发成的留在盘上。挂载续做走空时删除类发布被落点那一判以空间拒，同样推抬 F 再试，预算口径与会话走空相同；推满仍放不下时挂载照样交回可写、队列留着，之后会话里每次用户改动先走空、走不动报空间不够，正常卸载照常做成，靠的是 D16（发布语义） 已定项 1「正常卸载」那一行当场可用的那一截回收出来的槽（与 D16（发布语义） 已定项 1「准入」那一行同一定案）。任何一版里 `deleted_inodes` 至多一条 key；多于一个根装得下那一支拒成类型化错误、不是空间不够。
7. **幂等的判据五条**：① 计划相等；①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同（意图那次与每一批的改动计数是本次发布的 checkpoint_txg；意图那次的三个时间秒由调用方给、共一个值，各批照抄，续做的几批不另取时间）；② 删完再续做一个写都不发；③ 每个写点截断、重开续做到底，终态逻辑内容与不崩溃的终态相同、每个崩溃态 checker 全绿；④ 每个被判状态的记账标签 4、6 等于从这一版镜像现算的值。「逻辑内容」含 inode 每条（号、对象出生代、size、nlink）、extent 单元序号集与内容校验和、`deleted_inodes` key 集、墓碑记录多重集（含死亡 txg）、谱系记录多重集、标签 12 / 4 / 6、读者的答案；不含实例号、根与单元头里的 txg、改动计数、落点、单元头的诞生代号与写序、逐盘标签 1 / 2 / 5 / 10 / 11。另加 harness 层每种发布的重写角色集合钉绝对值（零轮）（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V1；`research/prompts/m3-delete-r3-main-verification.md` 第三节 G2）。

**射程**：第一版只做单文件 unlink、按 inode 号、无目录、不带 truncate（用户 2026-09-28 定）；删完之后不再建带数据的文件（K-U5，记欠账）；崩溃流接在 `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs` 那条流后面 unlink 它的 3 单元文件（K-U6 F1），截断成 0 另立一条流。只改尾叶的 extent 分支、写入类发布按发布之后的标签判、重写角色集合钉绝对值被攻过零轮；第 6 条增补的会话里每次用户改动先走空、与任何一版至多一条 key，也被攻零轮。

**依据**：

- 无实验：数都在三方腿的副本原型上量（三份删除判决），入库装置「删除借保留量的环窗口」在量环窗口那一半、还没出量。
- 三方判决 `research/prompts/m3-delete-r1-main-verification.md`、`research/prompts/m3-delete-r2-main-verification.md`、`research/prompts/m3-delete-r3-main-verification.md`。
- 用户定案 2026-10-01：K-U1、K-U5、K-U6、K-R3-1（原话在变更史）。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节队列上界与接口那一行：回退到意图那一版会把号带回队列，只有「会话里每次用户改动先走空」与「回退之后续做」两个臂在放开用户动作的历史上队列长度与有队列时的写次数两项都守住。
- 用户定案 2026-10-01：会话里每一次用户改动先走空（岔路单 `research/prompts/m3-blanks-r1-forks.md` 的「会话里的续做点」一行，原话在变更史）。
- 用户定案 2026-10-02：会话走空被空间拒推抬 F 再试；2026-10-03：每一次被拒的删除类发布各算一次准入、挂载续做被空间拒同样推抬 F 再试、推满仍放不下照 D16（发布语义） 已定项 1 做成可写、删除类发布各占一个事务号（原话在变更史）。
- 代码轮判决 `research/prompts/m3-unlink-code-r1-main-verification.md` 第二节 K2 那一行（续做、走空与推抬 F）与第四节（挂载续做推满仍放不下的结局交用户）；实现：会话走空推抬 F 是 `crates/singlefs-core/src/mounted_session.rs` 的 `publish_the_queue_until_it_is_empty_raising_the_floor_for_space`，事务号是 `crates/singlefs-core/src/transaction.rs` 的 `publish_deletion_step`。
- 主 agent 2026-10-02 定 Q-L 照收（调度记录 `records/2026-10-01-删除与并发事务.md` 第 320 行）：删空的一版上覆盖写、顺序写、unlink 拒成已有的那一种拒绝。
- 用户定案 2026-10-03：「卸载必须做成」，续做推满仍放不下之后正常卸载靠 D16（发布语义） 已定项 1「正常卸载」那一行放开的那一截回收出来的槽做成，单盘幸存那一问取「甲 GA（推荐）」（原话在变更史）；设计轮判决 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`，代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 N1 各行。
- 用户定案 2026-10-03：删除步算改动（「算改动，改实现」），时间字段取「乙 意图带时间、三格同改」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 N3 那一行。

**欠**：C618（删完之后不能再建带数据的文件）、C619（墓碑树根长高那次发布没有崩溃流）、C614（回退时墓碑树取错与删完对象无墓碑单镜像判不了）；C118（`deleted_inodes` 树的形态无落点） 还清。

```

**出处 `.claude/kb/decisions/03-空间分配.md:1-4`（整段抄，未转述）**

```markdown
## D3 空间分配 —— 已定

D3（空间分配） 管分配器这一格：空间分不分类、预留取什么形态、分配记录的粒度与载体与盘上编码、用户数据与提交内生块的落点、聚簇段的几何、分配准入合取的唯一登记位，以及「不许有假性 ENOSPC、删掉的空间在有界步数内可用」这条硬要求。不管的：准入不等式的权威形式与准入的在飞合成挪去了 D28（挂载期承诺量） 已定项 1 / 2，已定项 4 / 6 只留指针、编号不动；整理的触发判据、停机谓词与常驻底额在 D26（后台整理与放置回收）；一次写落哪几块设备在 D2（RAID 条带策略） 已定项 8；`df` 报什么与抬 F 那道闸在 D16（发布语义） 已定项 1；快照侧的 deadlist 纪律在 D5（快照 / 空间记账机制）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:29-49`（整段抄，未转述）**

```markdown
#### 已定项 1：分配记录逐落点答得出「属不属于现行版本」

**定案**：分配记录是「某个落点此刻属不属于现行版本」这个谓词的权威记录点，不是聚合量——D5（快照 / 空间记账机制） 已定项 3 那棵记账树装的是「(统计量, 维度元组, 代) → 完整值」，它答不了单个落点。谓词由两半合成，两半都已定：

| 半 | 谁给 | 说的是 |
|---|---|---|
| 这个落点分配出去了吗 | **D3（空间分配）** 的分配记录 | 空闲的落点不属于任何版本 |
| 分出去的属不属于**现行**版本 | **D5（快照 / 空间记账机制）** 的 deadlist | 进了 deadlist 的落点只被快照引用，不在活树里 |

「权威记录点」是运行时口径，不是态别：分配记录**是派生态**，整棵树可从单元 + 记账 + 根重建，权威态仍是那三样、不扩项（权威记录在 D21（权威态与派生态的分界） 已定项 5）；运行时判这个谓词只许问它、不许现扫，而它自己丢了能重建。

**射程**：管运行时逐落点判现行版本。不管扫描重建时多版单元的现行版本判定——那一格按 C113（扫描重建时多版单元的现行版本判定无输入） 定案收成「单元写序 + 实例表的已发布谓词 + 祖先表」，读不到这个谓词；D11（索引节点要不要留消息缓冲区） 已定项 4 的重算源因此也不含分配记录。整理**策略**本身不在射程里（D26（后台整理与放置回收） 的分项），量出来的是载体强制付出多少。

**依据**：

- E59（缓冲消息能不能只从单元重算）：只扫单元时同一个 key 的多份可读副本分不出现行值（分不出的数恰好等于覆写次数），加上这个谓词之后四类消息的分歧全部归零——逐落点这一半买到的就是这个。
- E74（分配记录的条目数与整理开销）：逐落点的条目数随容量走、与记账树的量纲不同，两棵树都没建错；整理路径里没有随盘容量走的项，而 COW 重写的是整个节点、落点在 key 空间里连续还是打散差几百倍——已定项 3 那条 key 把整理放在好的那一端。
- 用户定案，原话在变更史。

**欠**：C122（分配记录不点删之后的条目数口径）；C412（整理的挂钟代价没量过）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:50-67`（整段抄，未转述）**

```markdown
#### 已定项 2：墓碑与 livelist 的 FREE 条目由准入控制算进最坏情况

**定案**：删除这个操作本身要写盘——D18（块里携带什么信息） 已定项 8 定「删除要写墓碑单元」，可写头里删块时还要往 livelist 追加一条 FREE 条目（36 字节，D6（快照实现模型） 已定项 2），而已定项 17 的原则逐字写着「COW 系统必须保证『释放空间』这个操作本身不需要申请空间」。解法不是给删除路径开口子，是把那一笔提前到写入那一刻收掉：已定项 15 的准入控制本来就要求「任何增加空间占用的操作，进门前先算**最坏情况下解开自己需要多少**」——**墓碑就是「解开自己」的一部分**，计入那个量；livelist 的 FREE 条目同样是「解开自己」的一部分，计入那个量、写入时预付。墓碑那一笔按对象记：每个活对象（nlink ≥ 1）一份墓碑记录的摊分（含墓碑树内部节点的摊分）记进已承诺预留（D5（快照 / 空间记账机制） 已定项 4 第 6 项，记账标签 6），另加全池常驻的槽。精确式子：标签 6 = 副本数 × ((2 + H + 1) × 16384 + 41 × n_live)，单位字节、按副本之和记（C375（待删占用与已承诺预留按份还是按和记没有条款））；H 是现行那一版墓碑树的内部层数（零根与只有根时 0），从墓碑树根节点码 2 头现读（先例：D28（挂载期承诺量） 已定项 4 的 ckpt_cost 读法）；41 = ⌈2 × 16384 ÷ 815⌉，每份副本、每个对象先取整再乘 n_live；n_live = 这一版 inode 树里 nlink ≥ 1 的记录数；树表里没有 `deleted_inodes` 条目的版本，标签 6 取 0。删除各批的新写由准入式子里的常驻扣项「环窗口 = (根环槽数 3 × S + 1) × 每批上界 W」兜（D28（挂载期承诺量） 已定项 1，S 是系统配置每区槽数），不进记账行。删除类发布（意图、各批、挂载里的续做批）不判式子那一判，落点那一判照判；写入类发布按发布之后的标签判；环窗口从第一个文件版本那次发布起计。⇒ 删除路径仍然不申请空间：已定项 17 的原则一个字不改，改的是「解开自己需要多少」这个量的算法。

**射程**：定的是这笔账归谁付（准入控制，写入时付），不是这笔账是多少——第一版一次 unlink 写一条 40 字节整对象墓碑（D18（块里携带什么信息） 已定项 24）；能不能批量摊、盘满时删得动删不动还没有数，要等删除路径实现、E168（删除借保留量的环窗口） 出量才算得出来。它把已定项 15「预留量动态贴着实际需求，不白占」变紧了：预留量现在与「将来会不会被删」绑在一起，墓碑那一项对永不删除的对象是纯浪费，浪费多少同样没量。删除类发布不判式子被攻过一轮；写入类发布按发布之后的标签判、环窗口从第一个文件版本起计被攻过零轮；标签 6 的式子里常驻槽数取 2 + H + 1 现读这一写法被攻过零轮。

**依据**：

- E82（准入的在飞合成）：忘掉墓碑那一笔时盘满后删除全数死锁——「解开自己」含墓碑是可执行的，不是修辞。
- E130（每头一份 livelist 过不过有界销毁）：每释放一个块要多写的那几个字节把 livelist 追加抬成「解开自己」的一部分，不预付就与已定项 17 的原则冲突。
- 用户定案（墓碑那一半与后加的 livelist 那一半），原话在变更史。
- 墓碑按活对象摊分加全池常驻一份、删除类发布不判式子、写入类发布按发布之后的标签判、环窗口从第一个文件版本起计：`research/prompts/m3-delete-r3-main-verification.md` 第三节 G1-墓碑预付、G1-删除类发布怎么判、G1-写入类发布、G1-窗口起计四行。
- 环窗口是准入式子里的常驻扣项、不进已承诺预留：用户定案 K-U4「全池一份环窗口」与 K-R3-2「常驻扣项（推荐）」，K-R3-2 改了 K-U4 的字面，两条原话在变更史。
- 标签 6 的精确式子（常驻槽数 2 + H + 1 现读、41 × n_live、按副本之和记）：`research/prompts/m3-blanks-r1-main-verification.md` 第二节墓碑预付那一行；用户定案 2026-10-01「乙 2 + 层数 + 1，现读（推荐）」，原话在变更史。
- 环窗口的根数读根环槽数 3 × S：同一份判决第二节根环里的根数那一行（一轮）。

**欠**：C260（livelist 条目的预付没算进解开自己那个量）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:68-80`（整段抄，未转述）**

```markdown
#### 已定项 3：分配记录住一棵独立 keyspace 的 btree

**定案**：载体是独立 keyspace 的 btree，key = 落点（设备身份 + 设备内偏移），value = 分配代。形态照 D5（快照 / 空间记账机制） 已定项 3 的记账树，同一套 btree 机制，不新开第二种结构；key 用落点是已定项 1「逐落点」的直接后果，`(dev, 偏移)` 与 D19（块指针的结构与宽度预算） 已定项 1 的位置条目同一个坐标系，查一个落点是点查不是扫描；加一棵 keyspace 不是 incompat 改动——D15（格式冻结政策） 已定项 2 定树表 day-1 做成开放列表，判定表把「新增一棵 keyspace」钉死为 compat。先按这个形态落地，写代码时再调整。

**射程**：载体定了之后的代价（条目数、树高、整理放大按 key 空间连不连续分出两端）由 E74（分配记录的条目数与整理开销） 量，归已定项 1。定的是载体，不是粒度：量出来若不可接受，重开的是已定项 1 的粒度，不是这一项的载体。条目的盘上编码与宽度归已定项 7，跨度段归 key 还是 value 归已定项 11。

**依据**：

- E97（记账与分配记录的条目编码）：同一坐标系下的编码与条目宽由它判，落点按 16 KiB 槽号编码时非法落点归零，坐标系必须是 16 KiB 槽号。
- 用户定案，原话在变更史。

**欠**：C122（分配记录不点删之后的条目数口径）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:81-94`（整段抄，未转述）**

````markdown
#### 已定项 4：准入不等式的权威形式

**定案**：权威正文在 D28（挂载期承诺量） 已定项 1，引这一项时引那一处，别引这里。结论句：

```
可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数 − 环窗口(d)
```

**射程**：这一格只留指针与结论句；式子逐项的口径、按不按设备求和、每一项为什么在式子里，全在 D28（挂载期承诺量） 已定项 1。编号不重排，所以这一节留着。

**依据**：无实验：这一格只是指针，权威正文与全部依据都在 D28（挂载期承诺量） 已定项 1。

**欠**：无。

````

**出处 `.claude/kb/decisions/03-空间分配.md:95-109`（整段抄，未转述）**

```markdown
#### 已定项 5：提交内生块从聚簇段分配

**定案**：提交自身产生的块——分配记录树、记账树等结构的 COW 新节点——必须从 key 空间上连续的聚簇段（bump 段）分配；用户数据块的落点不受此约束。段的几何与 bump 次序在已定项 10，段耗尽时的回落在已定项 8 第 2 条。提交内生块在固定点分配时找不到空槽（`NoFreeSlotOnAnyDevice`），在任何写之前拒绝这次发布，盘上不变；这一格兜住准入式子罩不住的分裂与聚簇段层（C545（空间准入罩不住分裂与聚簇段层））。

**射程**：它是**分配器政策不是格式位**——聚簇段的位置与长度是运行时选择，盘上没有任何字段声明它；违反它的实现写出的镜像完全合法，只是每次发布贵几百倍。⇒ 拦它只能靠运行时读数：提交路径记「本次发布的固定点收敛轮数」，超过声明上界判红（`.claude/rules/fs-design.md` 硬要求 4：走错分支是性能悬崖，悬崖不报错）。w ≥ 2 时各设备上的聚簇段要不要对齐（每个单元 w 份副本各整落一列 ⇒ w 个聚簇段各自推进）全仓没有条款，归这一项的实现。

**依据**：

- E148（提交固定点按两棵记录树重算）：两棵记录树下聚簇提交内生块仍给常数固定点，全散布时没有上界——常数上界只在聚簇政策下存在。
- E81（提交的固定点）：全部记账结构都是 COW 树，给它们写节点又产生分配，固定点循环的放大由提交内生块聚不聚簇决定——全散布时一次 fsync 拖出的元数据块数让 I-8.1（环几何够大） 的「最坏事务占用」没有可用上界，只聚簇提交内生块就断链，用户数据全随机也不影响。它给的是下界（假设发布末尾一次性 COW），边插边 COW 的实现更贵。
- 用户定案，原话在变更史。
- `research/prompts/c363b-r1-main-verification.md` 第四节第 1 条，主 agent 定，被攻过零轮。

**欠**：C83（提交固定点没人回答）；C545（空间准入罩不住分裂与聚簇段层）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:110-119`（整段抄，未转述）**

```markdown
#### 已定项 6：准入的在飞合成

**定案**：权威正文在 D28（挂载期承诺量） 已定项 2，引这一项时引那一处，别引这里。结论句：准入读数 = 已发布统计量 − 在飞已批准的预留；窗口内产生的释放不发信用；「最坏情况下解开自己需要多少」含墓碑那一笔。

**射程**：这一格只留指针与结论句；在飞 overlay 的态别、实例切换重建内存态时 overlay 怎么重算，都在 D28（挂载期承诺量） 已定项 2。编号不重排，所以这一节留着。

**依据**：无实验：这一格只是指针，权威正文与全部依据都在 D28（挂载期承诺量） 已定项 2。

**欠**：无。

```

**出处 `.claude/kb/decisions/03-空间分配.md:120-158`（整段抄，未转述）**

```markdown
#### 已定项 7：分配记录条目的盘上编码，与分配准入合取的登记表

**定案**：落点粒度取 16384 字节（一个 16 KiB 槽）；一条记录记一个存储单元。key = (设备身份 4, 16 KiB 槽号 6) 共 10 字节；value = (跨度段 2 含已释放标志, 分配代或释放代 8) 共 10 字节，代存完整 checkpoint 号。

| 段 | 宽 | 归哪边 | 依据 |
|---|---|---|---|
| 设备身份 | **4** | key | D19（块指针的结构与宽度预算） 已定项 4。**同一个坐标系**，不许各定各的（已定项 3 依据第 2 条逐字）|
| 落点 | **6**，编码为 **16 KiB 槽号** | key | 粒度由 D18（块里携带什么信息） 已定项 9 逼出来（索引节点对齐 16384、大小就是 16 KiB）；E97（记账与分配记录的条目编码） 判据 7 实测按 32 KiB 单元号编码时一半的槽表达不了、按 16384 编码时零个。6 字节槽号寻址 4 EiB / 设备 |
| 跨度段（含已释放标志位） | **2（初值）** | value（已定项 11） | 一条记录记一个单元，跨度段说它覆盖几个槽。宽度按 D19（块指针的结构与宽度预算） 已定项 3 的「定宽留位」政策取，**理由是政策不是用量**——第一版恒为 1 或 2；**最高位借作已释放标志**（0 = 仍分配、1 = 已释放），不占表达跨度值的位 |
| 分配代 / 释放代 | **8** | value | 与 D22（单元原子性怎么合成） 已定项 7 的 `checkpoint_txg` 同宽同值域，语义按跨度段的标志位取其一。E97（记账与分配记录的条目编码） 判据 3 / 8 实测：任何有限模都只是把破 key 序推迟到回绕那一刻 |

**落点释放时条目不删、不点删**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4）：改写成「已释放 + 释放代」——释放代写进那 8 字节，已释放标志位借跨度段的最高位，条目留到该落点被重新分配时覆盖。不点删的理由：树按落点排序、代住 value，按代点删要扫全树，而点删所需的按代清单正是崩溃会丢的那份内存态；条目改写与产生这次释放的 COW 在同一次发布里原子生效——分配记录是根下的 COW 树（D22（单元原子性怎么合成） 已定项 3），条目与根同一次发布，释放代 = 那次发布的 checkpoint_txg；崩溃丢掉的是没发布的那个 checkpoint，它里面的释放一条都不留痕、对应的旧版本仍被所选根引用。崩溃后 defer 队列的内存态丢失，接手的是「释放代 ≤ max(F_生效, 环里最旧有效根)」这条闸（I-7.4（近 K 代块未被复用） 的注；谓词由 D16（发布语义） 已定项 1 给，有效性按实例表判；抬 F 的机制押在这个谓词上）。

**「释放」的口径** = 分配器把落点放进 defer 队列那一刻 = 没有任何根（含快照）引用它；被快照引用的落点按 D5（快照 / 空间记账机制） 的 deadlist 纪律进 deadlist 不进 defer、快照销毁时才释放 ⇒ 释放代晚于最后一个引用它的根被销毁的那次发布。**清扫准入**两支共有的合取 = 不在分配器在飞 overlay 里 ∧ 头可读；有条目的再要求已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)（D16（发布语义） 已定项 1 的谓词；由释放的口径，此时环里没有根还引用它；单元属于哪个实例无关，当前实例自己释放的落点也要能清）；孤儿（没有条目）要求头里的写序实例 < 当前实例 ∧ 按实例表的已发布谓词判未发布（当前实例没有行，它的孤儿只能是在飞的写，交给 overlay 那一条）。

**分配准入的合取式，唯一登记位就是这张表**（已定项 12 指到这里），共七条，全部为真才许分配：

| # | 合取 | 出处与射程 |
|---|---|---|
| 1 | `可用 ≥ 需求` | D28（挂载期承诺量） 已定项 1 的九项式子 |
| 2 | 落点谓词 | 已定项 1 的「属不属于现行版本」，输入是分配记录 + deadlist |
| 3 | 候选落点不在任何活让渡区里 | D9（加密） 已定项 7 那次重开把 `\|A\|` 从「已分配」改记「已承诺预留」，而那个量不带设备维、不带树维（D5（快照 / 空间记账机制） 已定项 4 第 6 项），A 的槽在分配记录树里不再有任何一位表达 ⇒ 密钥侧分配器可以把它们再分配一次；命题 5(b) 逐字「与 I-3.1（已分配统计对得上） / I-5.1（物理范围不重叠） 共用同一趟全盘遍历」= checker，不是运行时闸。形态照 D26（后台整理与放置回收） 已定项 1 第 3 条对 `R` 的写法：靠读一个有界的活对象集合推出来，不靠盘上多一位；活让渡区同时至多一个 ⇒ O(1) 点查不是遍历。⚠️ D9（加密） 已定项 7 随已定项 5 改取「只做 scrub」而撤回（让渡区 A 无对象）⇒ 这个合取恒真，条款原样保留 |
| 4 | 条带表[候选落点] 里没有活成员 | 只对 w ≥ 3 的条带起作用：D2（RAID 条带策略） 已定项 11 规则 1 逐字「w ≥ 3 条带的成员被释放后，该格与整条带（含 parity）在修复完成前不得复用」；w = 2 的镜像没有 parity、两份一起释放，这一条恒真。代价是每次分配多一次点查（条带表按落点排，与分配记录树同一个坐标系，D2（RAID 条带策略） 已定项 14），点查的执行路径押在条带成员表的载体上（D2（RAID 条带策略） 已定项 12）|
| 5 | 不在 D26（后台整理与放置回收） 已定项 1 保护期内的 `R` | 整理意图期内那批槽不许当分配目的地 |
| 6 | 候选落点不在任何开放的聚簇段里（只管用户数据）| 已定项 10 ②；提交内生块正是从开放段 bump，不受这一条约束 |
| 7 | `全空聚簇段数 ≥ N₀` | D26（后台整理与放置回收） 已定项 3 的第二合取式；`N₀` 第一版取 0（整理未实现，整理进里程碑时重开）|

**一条记录恰好覆盖一个单元是分配器的不变量，不是编码强制的**：跨度 1 必须永远合法（索引节点就是一个 16 KiB 槽），所以 `(偶数槽, 跨度 1)` 这条记录永远写得出来。先按这个形态落地，写代码时再调整。

**射程**：占容量那个倍数不是这一项的判据——`.claude/rules/fs-design.md` 明令「比谁省不构成任何判据」，E97（记账与分配记录的条目编码） 把两条候选压到只剩这一个差别之后自己判不出高下，这一格是用户定的。条目数上界随「不点删」从「已分配落点数」变成「曾分配过、尚未被覆盖的落点数」，仍以容量 / 16 KiB 为界；E74（分配记录的条目数与整理开销） 判据 1「条目数恰好等于已分配落点数」在这个新口径下按定义为假，改口径之后要与 E97（记账与分配记录的条目编码） 一起重跑取新的绝对值，今天未跑。别处把这条条目形态引成「一条记一个单元、另带跨度段」的，取的是已定项 11 之前那版写法——「一条记录记一个存储单元」这一半今天照旧，跨度段则住 value。落点粒度这个数同时进两层冻结（位置条目在 D15（格式冻结政策） 已定项 7 的第 2 层、分配 keyspace 的 key 在第 3 层，而第 2 层要先于第 3 层冻）⇒ 它要当第 2 层的问题看；异构池（D12（目标介质） 已定项 3）让它不能挂在某一条布局线的 incompat 位上——一个布局 A 的节点里的位置条目会指向布局 B 的设备，这一格仍欠处置。合取 3 与合取 4 都不改新池新建文件写出的字节（第一版不加密、两块盘 w = 2，两条恒真）。

**依据**：

- E97（记账与分配记录的条目编码）：落点按 16 KiB 槽号编码时全部合法起点都表达得了、非法落点归零，代按有限模编码只是把破 key 序推迟到回绕；同一档上的条目数、树高与占容量逐条按条目宽线性，两条候选压到只剩占容量一个差别。
- E44（序号位宽的本机实测与代价）：本机的发布速率给出「1 字节的代多久绕完一圈」，代取 8 字节不是拍的。
- 用户定案（编码形态、第三个合取、第四个合取），原话在变更史；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4 给「不点删」。

**欠**：C122（分配记录不点删之后的条目数口径）；C127（钉住状态在分配记录里没有落点）；C411（分配记录的跨度段与单元类别不符没检查）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:159-185`（整段抄，未转述）**

```markdown
#### 已定项 8：用户数据块不进聚簇段，落点是已选设备内起点最小的空槽

**定案**：五条。

1. **第一版用户数据块不进聚簇段。落点**：设备集合由 D2（RAID 条带策略） 已定项 8 选定之后，在每一块被选中的设备上各自取该设备内**起点槽号最小、起点 32768 对齐、跨度内两槽都空、不在任何开放的聚簇段里、整跨度都过分配准入合取（七条，唯一登记位是已定项 7）的落点**；**不选设备、不轮转设备**（设备维归 D2（RAID 条带策略） 已定项 8）；保护期内的 `R`（D26（后台整理与放置回收） 已定项 1 第 3 条）已经是那七条合取里的一条，不另列。
2. **聚簇段在挂载期间对用户数据关着**：这次挂载开过的聚簇段（当前开放的那一段，以及回落之前开过的段）都不给用户数据（已定项 10 ② 的「任何开放的聚簇段」指的就是这些段）；重开之后（bump 指针与「这次挂载开过哪些段」都只住内存，挂载后新开一段，已定项 10 ①），上一次挂载开过、装着提交内生块的段对用户数据开放，混放允许。已定项 5「用户数据块的落点不受此约束」原样成立、不推广；提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`——C146（无空段时的回落政策全仓无定义） 第 ② 条（回落落到哪）由此有了政策，第 ① 条（整理的下一个目的地落回自己正在清空的段）不归这一项，归 D26（后台整理与放置回收） 已定项 1。
3. **≤ 4 KiB 档打包前的落点同第 1 条**；D27（小数据打包容器） 已定项 6 打包写出的容器也走第 1 条。
4. **只限第一版**。做不做分配器提示、提示写成什么在 D14（双轨（大小文件 / 持久临时）） 已定项 2 判，这一条是它的减数臂；第 1 条既然把第一版的落点唯一确定，任何第一版的提示都与它不能同真——D14（双轨（大小文件 / 持久临时）） 已定项 2 在第一版这一格上因此只剩「否」，那一项的重开条件之一就是重开第 1 条。
5. **底额引 D26（后台整理与放置回收） 已定项 3、不另写数**；整理的起跑与停机照 D26（后台整理与放置回收） 已定项 1。

第一版盘不等大时，各盘落点答案必须相同；小盘没有全空段而大盘有时，在动任何状态之前拒绝（发布层统一报假性 ENOSPC）。**写明这是第一版的限制、不是永久判据**，以后放开时要答的是 D2（RAID 条带策略） 已定项 2 那一半（小盘写满之后设备集合怎么选）与已定项 8 待办 ①（w ≥ 2 时各盘聚簇段要不要对齐）。

**射程**：它是**分配器政策**，与已定项 5 同一类——盘上没有字段声明它；例外是**从 mkfs 水位第一次建树的那一版**，单镜像上的认法是 ① 树表非空，且 extent 树条目的树 ID 是 65536（mkfs 种下的水位）；② 树表里每条条目的诞生 txg 都等于这一版的 `checkpoint_txg`。新池新建文件是这一类，mkfs 之后重挂、实例 1、2、3 的第一个文件只要从 mkfs 水位建树也是。这一类上那一次用户数据的落点取值（t1@50180，即单元区起始槽号 U 加 4：mkfs 的实例表占 U、U + 1，树表占 U + 2，第一个 32768 对齐的空槽对从 U + 4 起；默认 768 MiB 环 U = 50176，环长不是默认值时 U 按已定项 10 ④ 现算、t1 仍是 U + 4）登记在 [layout/01-first-txn.md](../layout/01-first-txn.md) 字节表里，算格式级，偏离它写出的这一类镜像不合法，判它的是 I-5.5（从 mkfs 水位建树那一版的 t1 落点）。其余的写（覆盖写、之后的发布、重挂之后不属于这一类的版本）上违反它的实现写出的镜像合法，拦它只能靠运行时读数（D26（后台整理与放置回收） 已定项 6 的回落计数与全空聚簇段数）。⚠️ 第 2 条混放的代价：装着用户数据的旧段要等段里的用户数据也释放才能整段空出来，提交内生块的聚簇随挂载次数退化；现象是全空聚簇段数随挂载次数单调下降、回落次数上升。**改新池新建文件的字节：是**——新池新建文件写出的字段与宽度一个都不变，用户数据那一份的落点取值（t1@50180，连同指向它的指针与分配记录条目里的取值）由第 1 条定，[layout/01-first-txn.md](../layout/01-first-txn.md) 第 67、309 行把它登记成已定的字节。两句待办：w ≥ 2 时各设备上的聚簇段要不要对齐归已定项 5 的实现；「同一次请求切出的事务按 key 升序取号」已写进 D16（发布语义） 已定项 5，运行时检查仍欠。runs 这个量今天没有条款消费它的绝对值（D26（后台整理与放置回收） 已定项 1 逐字「阈值数值不进格式：等 E10（老化：结构抗老化 vs 事后整理）」）。

**依据**：

- E151（用户数据落点的到达序与容器臂）：分得开候选的量是**提交内生块进不进聚簇段**——用户数据进 bump 段时它们绝大多数落不进，不进段时是 0（装置里提交内生块寿命一个 checkpoint，0 是下界）；它是已定项 5 用户定案的「必须」要的量，也是 D26（后台整理与放置回收） 已定项 3 第二合取式读的量的上游。同一页给出这条政策在成段重写负载与容器档上的 runs 代价，均匀随机负载打平。
- E93（老化下的放置与碎片度）：最低空槽优先这条臂在均匀随机负载上几百个 checkpoint 内就冲顶，聚簇段那条臂的回落率九成以上——「聚簇段分配」这个政策骑在整理身上；这一条不是为 runs 选的。
- 两轮三方论证的判决在 `research/prompts/d3-r2-main-verification.md`，第一轮与第二轮的经过在 [records/2026-09-10-D3项8第一轮.md](../../../records/2026-09-10-D3项8第一轮.md) 与 [records/2026-09-13-D3项8第二轮.md](../../../records/2026-09-13-D3项8第二轮.md)。
- 用户定案（在三条出路里定了庚），原话在变更史。
- 2026-09-23 三方判决 `m2-placement-falsepositive-r1-main-verification.md` 里判盘不等大落点那一格——今天的三个错误成员是对空白的第一版默认，D2（RAID 条带策略） 已定项 2 明写「不由已定项 2 覆盖」、已定项 8 在第一版几何下没有对象可执行、已定项 10 允许各副本槽号不同。
- 用户定案 2026-09-23：一个聚簇段什么时候不再算——照今天的实现写成条款：这次挂载开过的聚簇段都关着，重开之后混放允许；代码不动（里程碑「覆盖写、释放、回退与复用」收口表第 59 行），原话在变更史；有警告，见 `.claude/warnings/2026-09-23.md`。
- 用户定案 2026-10-03：用户数据落点的取值在新池新建文件上算 layout 登记的字节，原话在变更史；问题由 `research/prompts/e171-conclusion-r3-main-verification.md` 第五节第 1 行提出（D3（空间分配） 已定项 8 射程与 `layout/01-first-txn.md` 第 67、309 行对读互相矛盾）。
- 用户定案 2026-10-04：落点约束管从 mkfs 水位第一次建树的那一版（候选甲），原话在变更史；认法与判据由 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2 与第五节第 1 行给出（认法 ①②、t1 = U + 4 在 6 种几何 × 5 种长度上 24 格全判、全绿，攻方副本上量过）。

**欠**：C319（请求内单元按 key 升序发出没有条款也没有检查）；C146（无空段时的回落政策全仓无定义） 第 ① 条；C243（停机谓词那个量从来没有过轨迹）；C673（三 oracle 合谋测试的补 oracle 改法没做） ⑥（判它的 I-5.5（从 mkfs 水位建树那一版的 t1 落点） 待实现）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:186-203`（整段抄，未转述）**

```markdown
#### 已定项 9：不许有假性 ENOSPC，删掉的空间在有界步数内可用

**定案**：两条硬要求。

1. **不许有假性 ENOSPC**：只要 `df` 报出的空闲 ≥ s，写 s 字节就必须成功。文件系统可以先在内部做有界步数的工作（推发布、抬回退下界之类）再分配；只有盘真的写满（`df` 空闲 < s）才许报 ENOSPC。
2. **删掉的空间在有界步数内可用**：删掉一个 s 字节的对象之后，同样大小的写必须在有界步数内成功。这个界要写成算得出的数，不许是「最终会」。**界是 3 次改变用户可见状态的发布**：删掉的块被最近 4 个状态里更旧的 3 个钉着，要再发生 3 次这样的发布才回可分配集合；在那之前 `df` 不把它算空闲，写它按 `df` 是真 ENOSPC，第 1 条不破。文件系统自己推的空发布不算这 3 次（空发布不改用户可见状态）。

可判定形式是不变量 I-5.3（报出的空闲都兑现得了）。**它当场改变了什么**：`df` 报什么是这条定案的另一半：D16（发布语义） 已定项 1 定的 `df` 口径把被扣掉的那几截排除在空闲之外，第 1 条靠它兑现，代价是 `df` 少报、删掉 s 字节时 `df` 可能一个字节都不涨。第 2 条的「有界步数」按 D16（发布语义） 已定项 1 选定的臂计：准入不够时先推空发布抬 F，一次准入最多 B = 4 + 2 k_tol 次发布，做满仍不够才报 ENOSPC。

**射程**：只管假性 ENOSPC 与删除后的可用性，不管真 ENOSPC；配额不在内（第一版不做每快照配额）。未在真设备上验证。**界 3 是按字面判的**：D16（发布语义） 已定项 1 选定那条臂在填满之后删了再写，按它自己的 `df` 那几次都是真 ENOSPC，而按这一条第 2 条的字面它没达标；射程只到 k_tol = 2 次根槽写失败。保留池那一项不是常数——它按 D28（挂载期承诺量） 已定项 4 每次发布按当时的结构现算，第 1 条的兑现条件随树高变。

**依据**：

- E139（按盘回退下界的收严形态）：填满之后删了再写的失败窗口逐窗数出来，越过「3 次」的那几格是哪几窗有产物；一次准入的发布数实测不超过 B，跑前保留池下不卡死。
- 用户定案（逐字「我们的目标是 避免 btrfs ENOSPC， 我们绝对不能接受 ENOSPC」，在「只要第 1 条」与「两条都要」之间选了两条都要），原话在变更史。它正是 [pitfalls.md](../pitfalls.md) 第 1 条那个症状的两半，而已定项 13 的「不分类」只治了其中一个病根。

**欠**：C283（准入失败时不先推发布就报 ENOSPC）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:204-238`（整段抄，未转述）**

```markdown
#### 已定项 10：聚簇段的定义、用户数据落点的四格、提交内生块的 bump 次序

**定案**：① 聚簇段段长 64 个 16 KiB 槽；段起点 = 单元区内最低的、64 槽对齐的全空段；bump 指针只住内存，挂载后新开一段；「全空聚簇段数」= 段内 64 槽都没有未释放分配记录的段数（记账行记的就是它，`crates/singlefs-core/src/allocator.rs` 的 `empty_segments`）；分配器开段另看隔离位（影子账、D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）与抬 F 扣住位，那一个量叫「可开聚簇段」（`lowest_empty_segment` 按它找段起点）——两个量分开登记，不许互相顶替（C379（记账的全空聚簇段数与分配器能开的段两个口径） 取 B，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M3：自然历史里部分隔离的一段让两者差 1 段）。② 用户数据的候选集加一条合取「候选落点不在任何开放的聚簇段里」。③ 数据单元取最低的、起点 32768 对齐、两槽都空、整跨度都过合取的起点；索引节点是提交内生块，从开放聚簇段 bump（已定项 5），「最低的空槽」只在开放段之内说，不下探到段外。④ 系统配置几何段加「单元区起始槽号」8 字节（第一版 = journal 环末尾的下一个槽：mkfs 按环长现算写进系统配置偏移 417，起点不在 64 槽段边界上在任何写之前拒；读者择系统配置时判 417 = 现算起点，对不上整池拒；默认 768 MiB 环是 50176，实审 A3b 报告 `research/prompts/m2-rev-a3b-implementer-report.md`），分配器不下探；固定结构不写分配记录。⑤ **提交内生块从开放段 bump 的次序**：一次发布重写实例表单元时（写行那一次），**实例表单元最前**，多于一片时尾片先（第 k 片当第 k+1 片的父，照「先叶后根」）；其余按**树 ID 升序**，树内**先叶后根**、同层按 key 升序，**中央映射树倒数第二**，**树表单元最末**；码 3 打包容器按「数据单元」那一档取落点（起点 32768 对齐、两槽都空）。这个次序同时就是出生序号的发号次序（D19（块指针的结构与宽度预算） 已定项 9）。

⑤ 在新池新建文件上的落点（开放段 [50240, 50304)，bump 游标从 50240 起）：

| 序 | 单元 | 树 ID | 落点 | 出生序号 |
|---|---|---|---|---|
| t2 | extent 树根兼叶（码 2，16 KiB） | 65536 | 50240 | 0 |
| t3 | inode 树叶容器（码 3，32 KiB，起点要 32768 对齐 ⇒ 跳过 50241） | 65537 | 50242–50243 | 0 |
| t4 | inode 树根（码 2） | 65537 | 50244 | 1 |
| t5 | 分配记录树叶，设备 0（码 2） | 65538 | 50245 | 0 |
| t6 | 分配记录树叶，设备 1（码 2） | 65538 | 50246 | 1 |
| t7 | 分配记录树层级 1 节点，设备 0（码 2） | 65538 | 50247 | 2 |
| t8 | 分配记录树层级 1 节点，设备 1（码 2） | 65538 | 50248 | 3 |
| t9 | 分配记录树根，层级 2（码 2） | 65538 | 50249 | 4 |
| t10 | 记账树根兼叶（码 2） | 65539 | 50250 | 0 |
| t11 | 中央映射树根兼叶（码 2） | 65540 | 50251 | 0 |
| t12 | 树表单元（码 2） | — | 50252 | 0 |

槽 50241 是这个次序留下的一个空洞：t2 占掉 50240 之后游标停在 50241，而 t3 要 32768 对齐的两槽 ⇒ 游标进到 50242，50241 一直空着。它让新池新建文件的空闲 run 数从 3 变 4（记账第 10 项，D5（快照 / 空间记账机制） 已定项 8）。

**射程**：③ 里「索引节点取最低的空槽」只在开放段之内说——按字面读「单元区里最低的空槽」会与已定项 5 / 已定项 8「聚簇段只给提交内生块」给出两套写清单，字节表与 E142（新池新建文件的干跑） 装置取的都是开放段内那一套。⑤ 取「树 ID 升序 + 树内先叶后根」的理由：树 ID 是全仓到处在用的那个键（D8（核心索引结构） 已定项 8 逐字「条目按树 ID 升序排」），先叶后根是 COW 本来的顺序（父指针要等孩子的落点定了才写得出来）；中央映射树倒数第二是因为它要装别人的条目，树表单元最末是因为它装全部树的根指针。码 3 容器取「数据单元」那一档，依据是 D18（块里携带什么信息） 已定项 9 的对齐要求按单元大小分档、与它是不是用户数据无关，两种读法在开放段内给同一个落点，写死为条款把 C324（码 3 打包容器要不要 32768 对齐没写） 收口。**改新池新建文件的字节：是**——十一个提交内生块（默认几何）的落点整体重排，分配记录的 key、位置条目的槽号、中央映射条目的 value 全跟着变。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。 ⚠️ ⑤ 那张落点表是默认几何（768 MiB 环、两块 4 GiB 盘）上的值：提交内生块的个数随几何变（环长 1024 MiB 时 13 个），t2–t12 偏离这张表写出的镜像算不算不合法，这一版没有条款、不判（只有 t1 由已定项 8 射程判）。

**依据**：

- E151（用户数据落点的到达序与容器臂）：段长 64 槽是它的装置建模时用的几何，段长这个数不是这一项现拍的。
- E142（新池新建文件的干跑）：⑤ 那张落点表与 50241 那个空洞是它写出来的字节，字节表与它逐字对得上。
- 用户定案（① 到 ④ 按总审核的选项逐条定，⑤ 另一次弹窗定案；⑤ 里实例表最前那一句 2026-09-23 定），原话在变更史。
- `research/prompts/m2-presumed-clauses-r1-main-verification.md` K1：条款没点名实例表时，三个候选里只有「最前」不与 ⑤ 的「中央映射树倒数第二，树表单元最末」冲突（本地攻方两次一致）。此前这个次序只以字节表里那一行的行序存在，一条条款都没有——两个实现按各自顺序 bump，写出来的落点、出生序号、映射 key 全都不同，而没有任何东西会说话。
- 用户定案 2026-09-24：实例表多于一片时尾片先（实四交回的两处条款空白之一，原话在变更史）；无实验：两种排法只差落点与出生序号的次序，没有可量的代价差。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2：⑤ 那张表按实现现值改（11 个单元、树 ID 从 65536 起，落点与 [layout/01-first-txn.md](../layout/01-first-txn.md) 第 67–78 行同；攻方副本上读出提交内生块相对单元区起点 [64, 66, 68, …, 76]），提交内生块个数随几何变（同一份副本上环长 1024 MiB 时 13 个）；出生序号照 D19（块指针的结构与宽度预算） 已定项 9 逐树从 0 起数。

**欠**：C324（码 3 打包容器要不要 32768 对齐没写） 已由 ⑤ 收口。

```

**出处 `.claude/kb/decisions/03-空间分配.md:239-252`（整段抄，未转述）**

```markdown
#### 已定项 11：分配记录 20 字节里跨度段进 value

**定案**：进 value：key = (设备身份 4, 16 KiB 槽号 6) 共 10 字节，value = (跨度段 2, 分配代或释放代 8) 共 10 字节；已释放标志仍在跨度段最高位。分配记录树的码 2 头按 key 宽 10 算。

**射程**：只定跨度段归哪一边，字段本身的宽度与语义在已定项 7。**改新池新建文件的字节：是**——t5 的条目宽字段、声明长度与那批记录的排布。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。

**依据**：

- E142（新池新建文件的干跑）：t5 的条目宽、声明长度与记录排布按这一项的宽度写出来，字节表与产物逐字对得上。
- 已定项 7 逐字「条目不删、改写成已释放 + 释放代」是**原地改 value**；跨度段进 key 时标志一翻 key 就变，树里同一个落点会留下两条记录。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/03-空间分配.md:253-262`（整段抄，未转述）**

```markdown
#### 已定项 12：分配准入合取的唯一登记位，与两条准入式的关系

**定案**：分配准入的合取式只登记在已定项 7 那张表一处，共七条，别处一律指过去。D16（发布语义） 已定项 1 的 `可分配 = min(可再分配 + 活元数据 − 保留池, df)` 与 D28（挂载期承诺量） 已定项 1 的九项式子是两道串联的闸：先过 D28（挂载期承诺量） 的「可用 ≥ 需求」，再过 D16（发布语义） 的抬 F 逻辑。

**射程**：定的是登记位与两式的先后，不改任何一条合取的内容。**改新池新建文件的字节：否**——新池新建文件恒过准入，这一项挡的是准入判定本身。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。

**依据**：无实验：收到一处是 `.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条的直接要求（此前合取式有四处互不相同的写法），没有可量的量；两道闸的读法与 D16（发布语义） 已定项 1「准入不够时先推空发布抬 F」的措辞一致。用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/03-空间分配.md:263-272`（整段抄，未转述）**

```markdown
#### 已定项 13：不分 data / metadata 两类空间

**定案**：不分 data / metadata 两类空间，全局统一分配器。

**射程**：管的是分配器有没有第二个分类轴，不管落点政策（已定项 8 / 10）、不管设备维（D2（RAID 条带策略） 已定项 8）、也不管一棵树的节点与它的叶要不要分开放。它直接推出已定项 14 那条连带承诺：不分类意味着碎片化压力全压在一个分配器上。D26（后台整理与放置回收） 已定项 3 的底额「越小越好」这条口径也建在它上面——不分类之后「释放要申请空间」不像 btrfs 那样致命。

**依据**：无实验：这一格定的是不开第二个分类轴，没有可量的量。病根是 [pitfalls.md](../pitfalls.md) 第 1 条那个症状——btrfs 的两级分配加 chunk 分类僵化，`df` 还有几十 G 却报没空间、删文件也报没空间；按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「别的项目怎么做，是线索不是证据」，它只当反证与机制，不当正证。用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/03-空间分配.md:273-285`（整段抄，未转述）**

```markdown
#### 已定项 14：不分类的连带承诺——后台整理常驻、自动、用户无需知道

**定案**：选了已定项 13 的不分类，就等于承诺后台整理是**常驻的、自动的、用户无需知道的**——btrfs 靠人手动跑 `balance` 来救，那是它最大的运维恶名之一。这个承诺的承接方是 D26（后台整理与放置回收）：机制在它的已定项 1 / 2 / 3，触发判据、停机谓词与常驻底额都在那里，未定的那几项也挂在那里。

**射程**：承诺的是「不要用户操心」，不承诺整理用的是哪套算法、阈值取多少（数值等 E10（老化：结构抗老化 vs 事后整理））。两处例外要连着引：加密开启时这条承诺不成立（已定项 16）；D26（后台整理与放置回收） 已定项 7 把回收分成两条线之后，走交给盘那条线时腿一里哪几件活停下还未定（D26（后台整理与放置回收） 未定项 9），那一问要连同已定项 5 一起答。

**依据**：

- E93（老化下的放置与碎片度）：不做任何放置的那条臂在均匀随机负载上几百个 checkpoint 内就冲顶，不是缓慢退化；聚簇段那条臂的回落率九成以上，「聚簇段分配」这个政策骑在整理身上 ⇒ 没有常驻整理喂空段，落点政策本身立不住。
- 用户定案，原话在变更史。

**欠**：C243（停机谓词那个量从来没有过轨迹）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:286-298`（整段抄，未转述）**

```markdown
#### 已定项 15：空间预留取准入控制形态

**定案**：空间预留用准入控制形态，量要小。预留不是「划一块地不给用」，而是：任何增加空间占用的操作，进门前先算「**最坏情况下解开自己需要多少**」，算不过就拒绝。这样预留量动态贴着实际需求，不白占；量要小。它的可执行形式是 D28（挂载期承诺量） 已定项 1 的九项式子，读数怎么合成在 D28（挂载期承诺量） 已定项 2，登记位是已定项 7 那张七条合取表。

**射程**：定的是形态与方向（量要小、贴着需求），不定任何一个具体的数——每一项算多少在 D28（挂载期承诺量），整理的常驻底额在 D26（后台整理与放置回收） 已定项 3。「解开自己需要多少」这个量含哪几笔由已定项 2 扩写（墓碑、livelist 的 FREE 条目），扩一笔就把「不白占」紧一分。

**依据**：

- E82（准入的在飞合成）：三种直觉写法各出一类可数违例——只读已发布值同窗超卖、把窗口内释放记成可用会吃掉 defer 该扣住的块、忘了墓碑盘满后删除全数死锁 ⇒ 准入控制这个形态要配在飞合成才立得住，而不是划一块地。
- 用户定案，原话在变更史。

**欠**：C87（准入读的合成值无定义）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:299-312`（整段抄，未转述）**

```markdown
#### 已定项 16：加密开启时无密钥的一侧只读只检测

**定案**：加密开启时，无密钥的一侧**只读只检测，既不搬也不回收**。

- **不能搬**：四条已定条款不能同时为真——搬一个单元就是发出一条新条带（D2（RAID 条带策略） 已定项 6 逐字「`w ≥ 2` 是硬下界（零冗余的条带不许发出）」），新条带的每一格都要一条成员记录（D2（RAID 条带策略） 已定项 12），而成员表逐字「**必须住在单元的载荷里**」，载荷是密文（D9（加密） 已定项 11「除系统配置外全部加密，元数据留最小明文头」）⇒ 无密钥侧写不了成员表 ⇒ 搬不成。`w ≥ 2` 是硬下界 ⇒ 这条对**任何**池成立，不只是 `w ≥ 3` 的池。
- **不能回收**：记账是权威态（D21（权威态与派生态的分界））且加密，无密钥侧写不了它 ⇒「把空间标记为可分配」在锁盘期做不到。
- **检测始终可做**（指针里有不要密钥的密文校验和）——这是无密钥侧唯一保留的能力。

**射程**：代价要写明并连着引——已定项 14 那条「常驻、自动、用户无需知道的后台整理」在加密开启时**不成立**，锁盘期整理与疏散全部停摆，近满卷上的换盘与缩容同样做不成。这是知情接受的，不是遗漏。加密不进第一个可运行版本（D9（加密） 已定项 10）⇒ 第一版没有执行路径。

**依据**：无实验：不是取舍改了偏好，是「能搬」这一侧被四条已定条款证明不可达，没有可量的代价要比。用户定案（撤销原来的「能搬」），原话在变更史；权威记录在 D9（加密） 已定项 5。

**欠**：C167（明文映射层的引用全仓回扫没做）。

```

**出处 `.claude/kb/decisions/03-空间分配.md:313-327`（整段抄，未转述）**

```markdown
#### 已定项 17：释放空间这个操作本身不需要申请空间

**定案**：COW 系统必须保证「释放空间」这个操作本身不需要申请空间。btrfs 没在设计层保证这点，靠一堆启发式补了十几年还在出问题。凡是释放路径要写的字节（墓碑单元、livelist 的 FREE 条目），空间在写入那一刻就按已定项 15 的准入控制预付掉（已定项 2）。

**射程**：它是 D3（空间分配） 里最硬的一条，别的分项都要过它：已定项 2 定的是怎么在不破它的前提下容纳「删除要写盘」，已定项 9 第 2 条定的是删掉的空间多久回来。它不承诺「释放一定立刻成功」——被 defer 窗口与回退下界钉住的那几代按 D16（发布语义） 已定项 1 的口径不算空闲，那是真 ENOSPC 不是破例。删除类发布不判式子（已定项 2）是「释放空间不需要申请空间」在第一版的落法；墓碑树根满时长高（D8（核心索引结构） 已定项 6），不因结构拒删除。

**依据**：

- E130（每头一份 livelist 过不过有界销毁）：候选甲每释放一个块要多写的那几个字节逐字撞上这条原则，逼出「每批最坏空间需求有界且可在批前预留」这条判据的可执行读法。
- E82（准入的在飞合成）：忘掉墓碑那一笔的臂在盘满后删除全数死锁——这条原则被破掉时的现象是可数的。
- 删除类发布不判式子：`research/prompts/m3-delete-r3-main-verification.md` 第三节 G1-删除类发布怎么判那一行；墓碑树根满时长高：同一份判决第三节 G6-墓碑树根满那一行与用户定案 K-R3-3「写根长高（推荐）」，原话在变更史。
- 用户定案，原话在变更史。

**欠**：C260（livelist 条目的预付没算进解开自己那个量）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1-15`（整段抄，未转述）**

```markdown
## E171 三oracle合谋测试 —— 已跑（s4 整表 117 行、F4 扩表 164 行与编不过 6 行的补位都已跑，读数到齐；结论三轮判决已写回）

<!-- doc-lint:not-numbers O1 O2 O3 Q1 Q2 Q3 Q4 Y1 Y3 P1 P2 P3 P4 P5 P8 P9 N1 N2 N3 N4 N5 S1 S2 S3 S4 S5 S6 S7 S8 Z1 Z2 Z3 Z4 F1 F2 F3 F4 F5 F6 F7 X1 X2 X3 X4 X5 X6 X7 X8 X9 X10 X11 X12 X13 X14 X15 X52 X53 X54 X55 X56 X57 X58 X59 X60 X61 X62 X63 X64 X65 X66 X67 X68 X69 X70 X71 X72 X73 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 V1 V2 W1 K4 -->

跑前登记 `research/prompts/e171-preregistration.md`（含第十二节修订 1–5；Q4 的另一种读法由主 agent 在装置写之前删去，只留整表 ≤ 60 秒那一种）；问题单 `research/prompts/three-oracle-collusion-questions.md` 的 Q1（一张坏实现表在同一批历史与崩溃状态上，有没有一行三条 oracle 全放行）、Q2（手造坏镜像喂 O2、O3，有没有一张两条全放行）、Q3（拿已知被抓的坏实现逐条关掉一条 oracle，剩下的还红不红）、Q4（这道测试一次跑多久、怎么进门禁）。被测条款 D13（验证路线） 已定项 6：三个 oracle 的独立性要靠合谋测试验证，不许自称。

**现状**：坏实现表 117 行用修过的逐行驱动在同一棵冻住的树上跑完一遍（s4，两条线道、每条 2 线程），连同 Q3-核三趟与比对器；s3 那一次整表因驱动把上一行的变异带进下一行而整表作废、留档（「s3 整表作废」一节）。O3 用 E170（O3规约执行器第一版） 第 1 次重跑冻结的执行器，开头与收尾两次冻结核对都是 0 处不同。s4 的分母 D = 19 < 30，登记 F4 触发，扩表 164 行（序号 118–281）在同一份冻住的树上跑完，比对器把两半并成 281 行：分母 D = 37 ≥ 30。两趟里编不过的 6 行（第 33、59、127、163、206、243 行）当时的驱动没按登记 5.2 补位；驱动改成编不过之后补位，补位那一趟在同一份冻住的树上跑完（六行换上的都是未触达的行），比对器把 117 + 164 + 补位并成 `…-s4-f4-substitutes-comparator.out`：分母仍 D = 37，Q1、Q3 名单与补位之前逐字同（「编不过的 6 行补位（已跑）」一节）。读数到齐之后开了三轮结论判决（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`，第三轮是最终轮），判这批读数撑不撑 D13（验证路线） 已定项 6：真盲区候选收成第 15、23、42、242、269、270、271 行，别的全放行行与未触达行里交回值不同的行归另几类（「三轮判决之后的三分」一节）；三分与读数一样只属于 s4 那一棵冻住的树。

| 行 | 判定 | 依据（产物） | 照字面之外要一起读的 |
|---|---|---|---|
| Q1 | 有：甲、乙各 12 行三条全放行（s4 的 7 行加扩表的第 242、269、270、271、277 行），丙 13 行（另加第 233 行）；分母 D = 37（补位之前与之后相同） | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` 的 `name=q1_alpha`、`name=q1_beta`、`name=q1_gamma`（`verdict=yes_a_row_is_quiet_on_every_oracle`）、`name=q1_denominator`（`counted_D=37 … enough_to_judge_none_side=true`）；补位之前的 `…-s4-f4-comparator.out` 里 Q1 三种读法的名单行逐字同 | 照登记第六节 Q1-甲「≥ 1 判『有』」，与分母无关；D ≥ 30 只说「没有」那一边的门槛够了，这一批里「有」已成立。s4 那 7 行的读数在 F4 产物里没变（第 1–117 行的结果行逐字节相同）；扩表新出的 5 行与第 233 行（丙）、批 2 的第 265 行的读数在「F4 新出的全放行行」一节。三轮判决之后的三分在「三轮判决之后的三分」一节：真盲区候选 7 行（第 15、23、42、242、269、270、271 行），另几类 5 行（第 27、112、277 行靠记录重建的恢复，第 34 行分配政策，第 92 行条款在字节表与里程碑验收），没有一行还记「对这一批等价」；第 233 行在丙里全放行（甲乙里 O1-崩溃判官抓到），第 265 行批 2 是取样点不敏感。O3 在 37 行上一行都没红（`name=q1_column_red_rows … O3甲=0 O3乙=0`）。补位之后分母仍 37，「有」没翻 |
| Q2 | 有（照登记字面）：甲、乙各 4 张两条全放行，全是 P8 | 同一份产物的 `name=q2_all_quiet`（`table_rows=107 all_quiet_rows=4 verdict=yes_a_bad_image_passes_both`）、`name=f3_p_rows_judged_in_by_o3`、`name=q2_negative_bases`；O2 放行而 O3 判不在的数在 `name=q2_family` 的 P 族与盲测 B 族两行（`o3_only_strict=12`、`o3_only_strict=17`，整行抄在「Q2 里 O2 放行而 O3 判不在的坏镜像」一节） | 4 张都是 E170（O3规约执行器第一版） 设计员推的「不在」而 O3 判「在」的 P8 行，F3 触发；主 agent 照 F3 判成期望推错、这四张不是坏镜像（判决记在 D13（验证路线） 已定项 6 的射程）。Q2 的数与第二段 `…-s2-comparator.out` 同出一批输入，s4 比对器重算一遍；F4 产物里 Q2、F3 与对照的结果行与 s4 那一份逐字节相同。几何两点（`q2_geometry_unstable_kinds=2`）：翻面的是流一 P2、P5 两种补丁的 O2 一列（底态 F 红、Mk 不红），O3 两个底态都判「不在」，所以这两种补丁的「两条全放行」两点都不成立、不翻；全放行的 P8 两个底态都是 `same_on_both_points`。另有 O2 放行而 O3 判不在的坏镜像 29 张（P 族 12 张加盲测 B 族 17 张，12 + 17）：写「O2 放行而 O3 判不在」，不写「O3 独有」，Q2 只比 O2 与 O3 一对，O1 与 RefFS 没在坏镜像上跑；这 29 张是结构自洽的伪造根、伪造续接镜像，补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，细节在「它答不了的」 |
| Q3 | 有一行只靠一条：甲、乙各 23 行（只靠 O1 的 32、45、108、115、172、233、237、249、250、281；只靠 O2 的 8、65、86、87、90、99、102、135、234、238、245、258、265） | 补位之后的 `…-s4-f4-substitutes-comparator.out` 的 `name=q3_alpha`、`name=q3_beta`（补位之前的 `…-s4-f4-comparator.out` 同名行逐字同）；Q3-核 9 行 `name=q3_core_row` 全是 `differences_outside_the_closed_column=0`（Q3-核取第一刀的行，扩表没有第一刀） | Q3-核 9 趟 0 处不同，读法 9 成立、F5 不触发。第 102、233、265 行批 1 与批 2 红的列不同，都是取样点不敏感（批 2 没取到红的那几段或那一类态），记「不稳定」。s4 那 11 行另外两条为什么看不到在「Q3 只靠一条的 11 行」一节，扩表新出的 12 行在「F4 新出的只靠一条的 12 行」一节 |
| Q4 | 只能进全量：117 行整表两条线道 5 569.6 秒，扩表 164 行另一趟 7 607.9 秒，门槛 60 秒 | `e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out` 的两行 `name=table source=table-driver.out`（`rows=117 table_wall_ns=5569579167726`、`rows=164 table_wall_ns=7607936234181`） | 281 行分两趟跑，两趟挂钟相加约 13 177.5 秒（两个数相加，不是一趟量的）；各行之和 11 114.4 + 15 191.9 秒是单线道的估计（没单独跑单线道）。编不过的 6 行补位另一趟 1 277.9 秒（`…-s4-f4-substitutes-rows-timing.out` 的 `name=table source=substitution-driver.out … rows=6 table_wall_ns=1277937171946`），三趟相加约 14 455.5 秒，也是三个数相加。Q3-核每趟三行约 114–130 秒 |

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:16-31`（整段抄，未转述）**

```markdown
### 装置、研究一侧与变异表

| 件 | 位置 | 说明 |
|---|---|---|
| 入库装置 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs` | 模式 `baseline`、`row`、`known-bad-images`，开关 `--positive-control`、`--disable-oracle`；只读、只驱动、只观测，不改 core、checker、harness、format、replay-prefix；内联单测 31 条（`grep -c '#\[test\]'`） |
| 研究一侧 | `research/e7-index-bench/src/bin/e171_three_oracle_collusion/`（Python，目录里没有 `main.rs`，cargo 不认它当 bin） | 挑法脚本（登记 5.2，sha256 `4032a0e0…`，原样抄）、117 行的表 `bad-implementations-117.tsv`（登记 5.9 照名字抄出）、F4 扩表 `bad-implementations-expansion-164.tsv` 与造它的 `expansion_table.py`、`row_driver.py`、`comparator.py` 与它的几个模块、冻结执行器的接线 `frozen_o3.py`、判坏镜像的 `o3_bad_image_judge.py`、四个单测文件、变异驱动 |
| 变异 | 研究一侧 `research/mutations/e171_three_oracle_collusion.tsv` 67 行；装置一侧 `crates/mutations.tsv` 末尾 15 行（名字以「E171 装置」起头，`grep -c '^E171 装置' crates/mutations.tsv`） | 研究一侧经本目录的 `mutation_driver.py` 跑（登记修订 1 第 1 行）；装置一侧是 checker 档，三个数归提交时的门禁 checker-tier-crates-mutation-replay |
| 复跑 | `research/scripts/replay.sh` 里 E171 十一行登记（`grep -c '^E171.@driver' research/scripts/replay.sh`） | 见「复跑」一节 |
| 产物 | `research/results/` 下 `e171-three-oracle-collusion-2026-10-03-` 起头的 46 份（`ls research/results \| grep -c '^e171-three-oracle-collusion-2026-10-03-'`） | 逐份在「复跑」一节的产物表里 |

- 流的驱动：两条流（新池新建文件、固定脚本到 E 再正常卸载）照抄 E170（O3规约执行器第一版） 导出装置的两条流驱动、取样三批与中性格式的写出（不 `#[path]` 引它），改三处：驱动里的 API 调用不 `.expect`，结局（成功、错误成员、panic）记进 `calls.tsv` 的 `ok` / `err` 列与 `call-outcomes.tsv`；跟着每次调用推理想模型，记 O1-对拍（模型的答案与实现的结局比，`judge_and_advance`）；每个取样态算 O1-崩溃判官（`crash_recovery_disagreement`）与 O2（`check_pool_image`），写 `oracle-verdicts.tsv`。`baseline` 回比本地常量与 crates 的现算值（S1，42 条 `name=s1` 行都是 `holds=true`；`grep -c 'name=s1 ' …-baseline-run1.out` 与 `grep -c 'name=s1 .*holds=false'` 现数）；`row` 不回比，只取批 1 与批 2。
- S8 的三处调用在装置里的行号（`grep -n` 现取）：`newest_persisted_root` 第 1883 行、`observed_read_back_after_a_crash` 第 1885 行、`crash_recovery_disagreement` 第 1907 行，都在 `crash_judge_of_the_state`（第 1875 行）与它转调的 `crash_judge_verdict` 里；O1-对拍的 `judge_and_advance` 在第 544 行。
- 已知坏镜像族：`crates/singlefs-harness/tests/checker_known_bad_images.rs`（sha256 `e8543908…`）的导入、常量、辅助函数与 `known_bad_images` 照抄进装置里的一个模块（第 7–1294 行，加上 `known_bad_images` 用到而散在第 1295 行之后的 12 个常量与辅助函数）；`known-bad-images` 模式核条数 41、条目名与原文件逐条相同、干净镜像摘要等于 `build_pool` 的（S4）。
- 逐行驱动：一次跑先拍一张仓快照（`Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`，237 个文件，总摘要写在驱动的标准输出第一行），每条线道从快照同步出自己的树与编译目录，改一处、`cargo build --release`、装置 `row` 经 `run-with-memory-cap.sh`（限时 1800 秒）、还原；表里的行照名字取、核锚点，不行按补位规则换同组下一行（至多三次）。s4 与 F4 两趟时驱动只在编之前的失败上补位；第八段起编不过之后也补位（换上的行编不过再换，编之前的失败与编不过合起来至多三次），组里不含 E171 装置自己的变异行，`row-result.tsv` 记 `substituted_from` 与每一个试过而没用上的 `failed_attempt_<序>`；开关 `--keep-compile-failures-as-they-are` 只给 `replay.sh` 复跑 s4、F4 两份产物用，补位那一份产物不带它。


```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:32-79`（整段抄，未转述）**

````markdown
### s4 整表的结果（整行抄自产物）

`research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out`（2119 行，末行 `E7RESULT name=done emitted=2119`）。冻结核对、阳性对照四条 oracle 各两趟（基线与空变异）、Q1 的分母与三种读法、7 行全放行（甲）、Q3 两种读法、Q3-核 9 趟与判决行：

```
E7RESULT name=config segment=s4 o3=frozen_executor baseline_run=baseline-run1 rows_root=rows-s4 controls=8 disabled_runs=2 q3_core_comparisons=3
E7RESULT name=freeze_check phase=start reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=positive_control oracle=O1-对拍 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O1-对拍 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=baseline red_units=2 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=empty_mutation red_units=2 red=true
E7RESULT name=positive_control oracle=O2 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O2 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O3 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O3 run=empty_mutation red_units=1 red=true
E7RESULT name=q1_denominator rows_run=117 counted_D=19 untouched=93 only_in_the_ignored_pass=0 compile_failed=2 anchor_not_unique=0 not_found=0 self_crash=3 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=false
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=5 rows_run=117 more_than_half=false
E7RESULT name=q1_column_red_rows counted_rows=19 O1=5 O1-对拍=5 O1-崩溃判官=1 O2=8 O3甲=0 O3乙=0
E7RESULT name=untouched_but_red rows=0 row_numbers=none
E7RESULT name=q1_alpha counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_quiet_row reading=q1_alpha row=15 row_name=实审_A2c_第_15_条：用户数据落点不从这张图的单元区起点起扫 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=22/22 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=23 row_name=并行线三_C116：分裂出来的右半容器号不是触发分裂那条新记录的_inode_号 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=26/26 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=27 row_name=实二十：记录头读者把末条标志位_0_读反（带标志的读成不是末条） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=34 row_name=步_5：抬_F_生效之后不放开扣住的槽 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/498 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=42 row_name=实二（回退改形态：SysPre_与_B1，D16_已定项_1_/_7、D22_已定项_7_/_9、I-7.9_/_I-7.12、C556_/_C557）：卸载把_F_抬到现行那一版带的_F（原地不动）而不是它的_txg o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/537 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=92 row_name=步_1_验收第_4_条：inode_记录的写入时间留成_A_的（覆盖写照抄上一版的写入时间） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/61 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=112 row_name=步_3：前缀跨实例边界（把别的实例的记录也接上） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_beta counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_gamma counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=19 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle=none
E7RESULT name=q3_alpha caught_rows=12 only_O1=32,45,108,115 only_O2=8,65,86,87,90,99,102 rows_whose_red_columns_differ_between_batch_1_and_2=102
E7RESULT name=q3_beta caught_rows=12 only_O1=32,45,108,115 only_O2=8,65,86,87,90,99,102 rows_whose_red_columns_differ_between_batch_1_and_2=102
E7RESULT name=q3_core_row closing=O1 row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O1 row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O1 row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=no_executor row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=q3_core_row closing=no_executor row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=q3_core_row closing=no_executor row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=freeze_check phase=end reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=8 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 q3_core_runs=9 q3_core_runs_with_differences=0 rows_untouched_but_red=0 negative_bases_with_o2_red=0 negative_bases_with_o3_red=0 known_bad_rows_with_o2_quiet=0 o3_freeze_differences=0 o3_digest_mismatch_states=0 o3_dump_problem_lines=0 o3_image_digest_mismatches=0 rows_with_o3_freeze_lines_not_the_reference=0 rows_with_the_o3_executor_failed=0 a2_p_rows_not_expected_not_in=0 f3_p_rows_judged_in_by_o3=4 q2_geometry_unstable_kinds=2 q1_alpha_all_quiet_rows=7 q1_beta_all_quiet_rows=7 q1_gamma_all_quiet_rows=7 q1_o3_strict_reading_unreachable=0 q2_all_quiet_alpha=4 q2_all_quiet_beta=4 q3_alpha_rows_held_by_a_single_oracle=11 q3_beta_rows_held_by_a_single_oracle=11 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 rows_compiled_without_the_rebuild_check=0
```

分族、分类的计数（一条命令从产物里数，原样输出）：`grep -o 'name=row row=[0-9]* .* category=[a-z_]*' research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out | sed -E 's/.*category=//' | sort | uniq -c` 输出 `2 compile_failed`、`19 counted`、`3 self_crash`、`93 untouched`（四行）。不进分母的 5 行：第 6、7、85 行实现自崩，第 33、59 行编不过。登记 5.2 写「一行编不过……换同一文件组内下标 j + 1」，s3、s4 那一版逐行驱动只在编之前的失败（找不到、名字不唯一、锚点不唯一）上补位，第 33、59 行没有补位；驱动在第八段改成编不过之后也补位，这两行连同扩表编不过的 4 行的补位那一趟已跑（「编不过的 6 行补位（已跑）」一节）。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:80-95`（整段抄，未转述）**

```markdown
### 三条全放行的 7 行

读数取 s4 的导出目录（`research/target/e171-dumps/rows-s4/row-<序号>/dump/`，与 `baseline-run1` 逐条 op、逐态比；草稿脚本 `quiet_rows_s4_fua.py`、`returned_check.py` 只读导出目录，不入库）。三分照 `.claude/rules/mutation-sampling.md` 的口径落到这一批上：**对这一批等价**＝两条流、批 1 批 2 的每个态连同实现在态上的恢复都在条款允许的集合里（写依据；会分开的历史写在旁边，没跑）；**取样点不敏感**＝两条流里有条款不许的态，取样规则没取到；**真盲区**＝取到的态里有条款不许的，三条都放行。「镜像不同」数的是 `states.tsv` 里镜像摘要与基线那一态不同的态；「择到新版」数的是其中实现看 journal 那一遍恢复择到的根 txg 不小于「第一处不同之后写出的第一条根」的态。三条各自看什么：O1-对拍比每次调用之后实现交回的 `ObservedRoot`（`crates/singlefs-harness/src/model.rs` 第 589 行：根的 key、journal 计数器、根带的 F、文件内容、实例表、每块盘的分配记录），O1-崩溃判官比实现恢复择到的根与读回内容；O2 判单个镜像的结构；O3 的允许集合 A(c) 由这一行自己写出的根槽写构成（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py` 第 3 行），一版只投影根的六样（`root_ring.py` 第 22 行：实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F），判的是它自己从镜像恢复出的那一版在不在 A(c) 里（`executor_core.py` 第 127 行）。下表末列的三分是三轮结论判决之后的现值（旧三分列的改动写在「历史版本」一节），各行归属的全貌在「三轮判决之后的三分」一节。

| 行（变异） | 和基线不同在哪 | 落在哪种状态里 | O1 / O2 / O3 看不看得到 | 三分 |
|---|---|---|---|---|
| 15（用户数据落点不从这张图的单元区起点起扫） | 两条流第 22 条 op 起：用户数据单元两块盘各 32 KiB 落点 822149120 → 823132160，内容逐字节相同；指向它的 16 KiB 节点里指针与校验和随之变；同一次发布的十一个提交内生块（t2–t12）也整体后移 64 槽，开放段 [50240, 50304) 整段后移到 [50304, 50368)：设备 0 上 12 条写换了落点（第 22 条 op 50180 → 50240，第 24 条 50240 → 50304，第 26 条 50242 → 50306，……，第 44 条 50252 → 50316；两条流相同，`research/prompts/e171-conclusion-r3-opus-model/out/row15-offsets.out` 第 1、2 行 `name=row15_first_publish`）；流二第 4 次调用起返回点后移 4 条 op（`mount_writable` 92–179 → 92–183），共 555 条 op 不同；调用结局同；看 journal 那一遍恢复逐态同 | 镜像不同：流一批 1 20 态、批 2 7 态，流二批 1 364 态、批 2 119 态；择到新版：流一批 1 12 态，流二批 1 356 态、批 2 112 态 | O1：`ObservedRoot` 不带落点，读回同，看不到；O2：落点在单元区里、记账自洽，判绿；O3：投影不带落点，「在」 | **真盲区候选**：依据是 D3（空间分配） 已定项 10 ⑤ 的 t2–t12 落点：「⑤ 在新池新建文件上的落点（开放段 [50240, 50304)，bump 游标从 50240 起）」（`.claude/kb/decisions/03-空间分配.md` 第 207 行）、射程「改新池新建文件的字节：是」「七个提交内生块的落点整体重排」（第 221 行）、字节表「t2 到 t12 依次 50240、50242–50243、50244、…」（`.claude/kb/layout/01-first-txn.md` 第 307 行）；变异下用户数据落到 50240、十一个提交内生块整体挪 64 槽，不在 ⑤ 钉的落点上，三条都放行。另一条依据（用户 2026-10-03 定案之后才有）：D3（空间分配） 已定项 8 射程「新池新建文件是例外：那一次用户数据的落点取值（t1@50180）登记在 [layout/01-first-txn.md](../layout/01-first-txn.md) 字节表里，算格式级，偏离它写出的新池新建文件镜像不合法」（`.claude/kb/decisions/03-空间分配.md` 第 171 行），变异把 t1 从 50180 挪到 50240，本身就不合法；三轮判决定去留时还没有这一条，只靠 ⑤。会分开的历史（推的）：单元区刚好够用的池，跳过开头一段之后发布在模型说该成的地方报空间不够，O1-对拍会红 |
| 23（分裂出来的右半容器号不是触发分裂那条新记录的 inode 号） | 两条流第 26 条 op 起：第一个文件那次发布写的 inode 叶（码 3，两块盘各 32 KiB）偏移 53 一字节 0x01 → 0x00（容器号 1 → 0），头校验和随之变；同一次发布第 28 条 op（16 KiB 节点）偏移 52、60、131、149 等处 0x01 → 0x00；之后重写 inode 树的每次发布都带着（流二 155 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流一批 1 20 态、批 2 7 态，流二批 1 364 态、批 2 119 态；择到第一个文件那一版（txg 3）及之后：流一批 1 12 态，流二批 1 356 态、批 2 112 态 | O1：不读盘，`ObservedRoot` 里没有容器号，读回同；O2：I-9.4（容器号不超最小 key）只要「inode 号 ≥ 容器号、沿叶序严格递增」，一片叶时 1 ≥ 0 成立，判绿；O3：投影不进 inode 树，「在」 | **真盲区候选**：D8（核心索引结构） 已定项 6 定案写「**新池新建文件写**：一个码 2 根节点（头带 key 区间 [1, 1]；一条条目：key 1、类型 2、出生树、容器号 1、出生代 3、指针）；一个码 3 叶（类型 2、容器号 1、出生代 3、记录数 1、记录宽 140）」（`.claude/kb/decisions/08-核心索引结构.md` 第 203 行），射程写「每个容器号都是某条记录首次插入时的 inode 号」（第 205 行）；容器号 0 不是任何一条记录的 inode 号。择到新版的那些态不在条款写得出的镜像里，三条都放行。会让 O2 红的取样点（推的，没跑）：一片叶装满之后的分裂，右半容器号 0 小于左半，I-9.4 的「沿叶序严格递增」判红 |
| 27（记录头读者把末条标志位 0 读反） | 写流与调用结局全同；只在看 journal 那一遍恢复上不同：流一批 1 6 态、流二批 1 51 态（批 2 0 态）里，基线施加 1 条记录、择到新的一版，变异把带末条标志的记录读成「后面还有」、施加 0 条、停在前一版；读回内容不同的 3 / 30 态（例：流一第 41 态基线 `1,3 file_read e8ca4bf8…`，变异 `1,2 no_file`） | 发布写到一半的态：被丢掉的那一版的调用没返回、它的根槽没落（`returned_check.py` 57 态全是 `call_returned=False its_root_slot_landed=False`） | O1-崩溃判官：前一版是模型提交过的、不比盘上已持久的最新根槽旧、读回与那一版相同，判绿是对的；O1-对拍：调用不经恢复；O2、O3：镜像与基线逐字节相同，各自不看实现的恢复 | **另一类：三条不判实现恢复里靠记录重建的那一段**。态本身合法（调用没返回，新旧两版都在允许集合里），错在实现的恢复少施加一条记录，违反 D13（验证路线） 已定项 1 的 M4「`Recover` 返回的根恰是 j\*」与 D23（journal 的角色与格式） 已定项 15「由记录重建那次发布的根」、已定项 14「恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录」；O1-崩溃判官的判据（模型提交过的某一版、不比盘上已持久的最新根旧）比 M4 弱，O2 判恢复之前的镜像，O3 不跑实现的恢复；M4 的落点是 E165 与 Verus，不是这三条。会分开的取样点（推的，没跑）：一次已经返回、提交点只在 journal 记录里（根槽还没写）的发布之后的态，变异会丢掉一次已返回的发布；两条流的发布都在返回之前写根槽，这样的态不存在。那时 O1-崩溃判官的「不许比盘上已持久的最新根槽旧」也挡不住（根槽没写），会不会三条都放行没量过 |
| 34（抬 F 生效之后不放开扣住的槽） | 流一无不同；流二第 498 条 op 起（抬 F 之后那次 `publish`）：用户数据单元落点 822083584 → 822411264，内容相同；指向它的节点与分配记录随之变（51 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流二批 1 68 态、批 2 21 态；择到 txg 17 及之后的根：批 1 60 态、批 2 14 态 | 同第 15 行：O1 不比落点，O2 判绿（记账自洽），O3「在」 | **另一类：分配政策，镜像合法**：抬 F 之后那次发布换了一个空闲槽、没复用回收的槽，每个态的分配记录与记账自洽；D3（空间分配） 已定项 8 射程「其余的写（覆盖写、之后的发布、重挂之后）上违反它的实现写出的镜像合法，拦它只能靠运行时读数」（`.claude/kb/decisions/03-空间分配.md` 第 171 行；新池新建文件那一格例外，这一行的改动在流二抬 F 之后那次发布上，不在其中），三条按设计都不判分配政策。会分开的历史（推的）：抬 F 之后空间只够复用扣住那几槽的池，发布报空间不够而模型说该成，O1-对拍会红；这一行在 `crates/mutations.tsv` 点名的测试 `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` 直接断言复用 |
| 42（卸载把 F 抬到现行那一版带的 F 而不是它的 txg） | 流一无不同；流二第 537 条 op 起（卸载那次调用的第一条写：两块盘的系统配置槽，偏移 481 一字节 0x11 → 0x0b，即 F 17 → 11，连同校验和），之后卸载推的空发布写的根与节点里同样是 11（24 条 op 不同）；卸载照常 `ok`；恢复逐态同 | 镜像不同：流二批 1 48 态、批 2 14 态，全持久态在内；择到卸载写出的根（txg ≥ 18）：批 1 32 态、批 2 7 态 | O1：卸载没有模型的答（登记读法 6），这次调用不对拍、写落在卸载里的态崩溃判官记不适用，看不到（若有卸载的答，`ObservedRoot` 带根的 F，会比出 11 对 17，推的）；O2：系统配置的 F 与根带的 F 都是 11，I-7.12（系统配置 F 不低于同盘根上的 F）成立，单个镜像不知道该是 17；O3：投影带 F，但 A(c) 就是这一行自己写的 F = 11 的根，「在」 | **真盲区候选**：D16（发布语义） 已定项 1 定案的字段表「正常卸载」一行写「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md` 第 42 行），射程「正常卸载的代价」那一条写「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」（第 57 行）。变异下卸载返回之后的全持久态 F = 11，回退候选集里留着 txg 11–16 的根，不在条款允许的镜像里，三条都放行。O1 那一半是登记读法 6 写明的已知缺口（模型没有卸载的答） |
| 92（inode 记录的写入时间留成 A 的） | 流一无不同；流二第 61 条 op 起（第二次 `publish`，覆盖写那次的 inode 叶，两块盘各 32 KiB）：叶内偏移 200、208、216 三个字节 0x3c → 0x00（第 0 条记录起点 136，+64 / +72 / +80 是 atime、mtime、ctime 秒：基线 60，变异留 A 的 0），头里偏移 10–13（头校验和）与 89–92 另有不同；之后重写这片叶的发布都带着（139 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流二批 1 344 态、批 2 112 态；择到覆盖写那一版（txg 4）及之后：批 1 336 态、批 2 105 态 | O1：`ObservedFile` 只有内容，`model.rs` 与 `model_comparison.rs` 里 `write_time` 零命中，D13（验证路线） 已定项 6 表 O1 行判「内容、大小、目录结构」，看不到；O2：单个镜像不知道这次发布的写入时间参数，判绿；O3：投影不进 inode 树，「在」 | **另一类：条款在字节表与里程碑验收，决策正文欠一句**：D8（核心索引结构） 已定项 6 的字段表只定了偏移 64「atime / mtime / ctime 秒」的位置；「写入时间是发布参数」写在 `.claude/kb/milestone/01-first-txn.md` 第 103 行与 `02-second-txn.md` 第 103 行，字节表写「写入时刻（mkfs 参数，可复现）」（`.claude/kb/layout/01-first-txn.md` 第 267 行），验收写「inode 记录的写入时间留成 A 的 ⇒ 判红」（`02-second-txn.md` 第 113 行），决策正文里没找到取值的一句。择到新版的态（写入时间 0、这次发布的参数是 60）不合验收，三条按设计看不见（没有 API 交回写入时间），不当独立性失败 |
| 112（前缀跨实例边界） | 写流与调用结局全同；看 journal 那一遍恢复在两条流各 3 态（批 1 第 9–11 态，段 1：暖机的第一条写——新实例 1 的 journal 记录——落了、它的根槽没落）不同：基线择 `0,0`、施加 0 条，变异把实例 1 的记录接到实例 0 的根上、施加 1 条、得到 `1,1`；读回都没有文件 | 暖机写到一半的态（`returned_check.py` 6 态全是 `call_returned=False`） | O1-崩溃判官：`1,1` 是模型提交过的一版、不比已持久的最新根槽旧、读回同，判绿是对的；O2、O3：镜像与基线逐字节相同 | **另一类：三条不判实现恢复里靠记录重建的那一段（多施加）**：暖机没返回，`0,0` 与 `1,1` 都在允许集合里，错在实现的恢复把别的实例的记录也接上，违反 D23（journal 的角色与格式） 已定项 14 注 1「前缀规则不跨实例边界」（`.claude/kb/decisions/23-journal的角色与格式.md` 第 369 行），依据只有这一条注；这一行的变异已在门禁 checker-tier-formal-verification 的红例钉表里（`.claude/gate.d/e165-pinned-red-examples.tsv` 第 26 行），由 E165 的有界穷举判。会分开的取样点（推的，没跑）：环里留着别的实例、不在所选根那条时间线上的记录（崩溃恢复抛弃了根之后的新实例），接进来之后恢复落到模型没提交过的一版，O1-崩溃判官会红；两条流里没有这种态 |

第 102 行（抬 F 先写系统配置那一步不带新 F）是批 2 多出来的那一行全放行（`name=q1_alpha … rows_that_differ_from_batch_1=102`）：流二第 440 条 op（抬 F 先写的系统配置槽，偏移 481 一字节 0x0b → 0x00）与第 537 条 op（卸载先写的系统配置槽，0x11 → 0x0b）不同，4 条 op；O2 在批 1 判红 12 态，都是 I-7.12（系统配置 F 不低于同盘根上的 F），在第 60、73 段各 6 态（`name=trajectory row=102 stream=fixed_script_through_e_then_unmount oracle=O2 total_red_states=12 peak=6 positive_segments=2 of_segments=79 …`）。批 2 在流二只取 17 段（7、11、16、20、24、28、32、36、40、44、48、52、57、61、65、70、74），第 60、73 段一个态都没取（`awk -F'\t' '$3==60\|\|$3==73' states.tsv` 只出批 1 的 8 + 8 态）：**取样点不敏感**，会分开的取样点就是第 60、73 段的任一态。这一格两批翻面，记「不稳定」；Q1、Q3 的结论字在批 2 上不翻（批 2 全放行 8 行仍 ≥ 1；只靠一条的 10 行仍 ≥ 1）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:96-115`（整段抄，未转述）**

```markdown
### Q3 只靠一条的 11 行

只写够不到的对象与没判的字段，不判结论。读数取 s4 的导出目录（草稿脚本 `caught_rows_s4.py`，不入库）与产物的 `name=trajectory` 行；段为正的数是「红态的段数 / 这条流的段数」。

| 行（变异） | 那一条红在哪 | 另外两条为什么看不到 |
|---|---|---|
| 32（可写挂载不写实例表的行，实例表照抄） | O1-对拍：流二第 4 次调用 `mount_writable` 判 `disagrees:这一版的实例表`（模型要有上一个实例那一行，实现交回空表），模型停在这次调用，之后流二 324 态（批 1）崩溃判官记不适用 | O2：这一行 0 态红（批 1 流二 397 态都绿），实现了的不变量里没有一条拿根的实例代号去对实例表的行（由读数推的，没逐条查 invariants.md）；O3：投影里没有实例表，A(c) 是这一行自己写的根，「在」 |
| 45（挂载那一刻就读不出的槽也重读、仍坏就拒） | O1-对拍：流二第 12 次调用 `raise_floor` 交回 `err:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，模型说该成 | O2、O3：被拒之前的写流与基线逐条相同（流二停在第 440 条 op，0 条 op 不同），取到的镜像都是基线那一批的前缀；被拒的调用不留镜像，单镜像的判官没有对象可判 |
| 108（码 2 读者补齐字节从最后一个载荷字节算起） | O1-对拍：两条流第 2 次调用 `publish` 判 `disagrees:这一版的文件内容`（胶水解不开实现交回的单元：「补齐字节非零」）；流二第 4 次调用 `mount_writable` 交回 `err:Recovery` | 错在实现的读者一侧，写出的字节与基线相同（流一 0 条 op 不同，流二停在第 92 条 op 之前也 0 条不同）；O2 用自己的解析器读裸字节、不调 core 的读者，O3 用 Python 自己的读者，两边看到的镜像与基线逐字节相同。层 0 那份手写版本表的 oracle 与记录核对器红（`layer0_or_record_checker_red=true`，不是三条之一） |
| 115（取前缀核锚点筛选不看 txg） | O1-崩溃判官：批 1 流一 32 态、流二 52 态「走读失败：没择到根」（实现恢复报 `RootPublishCarriesMoreThanOneLastRecordFlag…`），段为正 8/12、12/16；O1-对拍：流二第 4 次调用 `mount_writable` 交回 `err:Recovery`，模型说该成 | 写出的字节与基线相同（流一 0 条 op 不同）；O2 判镜像结构、O3 用它自己的前缀规则从镜像恢复，都不看实现的恢复，看到的镜像与基线逐字节相同。层 0 那份 oracle 与记录核对器红 |
| 8（空闲不随分配减） | O2：I-5.2（空闲统计对得上）批 1 流一 9 态、流二 353 态，段为正 2/12、69/79；批 2 流二 112 态 | O1：`ObservedRoot` 里没有记账行，调用都成、内容都对；O3：投影不进记账树，「在」 |
| 65（blocks 向下取整） | O2：I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）批 1 流一 9 态、流二 353 态 | O1：`model.rs` 与 `model_comparison.rs` 里 `blocks` 零命中，读回内容同；O3：投影不进 inode 树 |
| 86（写系统配置槽时 F 那 8 字节写成 0） | O2：I-7.12（系统配置 F 不低于同盘根上的 F）批 1 流二 97 态，段为正 19/79；批 2 28 态 | O1：比的是根带的 F（`ObservedRoot` 的 `rollback_floor`），不比系统配置槽里的 F；O3：投影的 F 取自根，系统配置槽只拿来择几何，A(c) 是这一行自己写的根，「在」 |
| 87（core 那份在飞上限把 F 多算 1） | O2：I-8.1（环几何够大）批 1 流一 51 态、流二 385 态，I-7.13（系统配置池级字段在读者收的范围里）流一 2 态、流二 12 态；段为正 12/12、79/79 | O1：不比系统配置槽的字段；O3：同第 86 行，投影里没有在飞上限 |
| 90（新池新建文件的改动计数写回 1） | O2：I-9.26（inode 改动计数落在出生代与容器诞生代号之间）批 1 流一 9 态、流二 240 态 | O1：不比 inode 记录的改动计数；O3：投影不进 inode 树 |
| 99（defer 行写 0） | O2：I-3.11（已分配减 defer 等于最新根走读）批 1 流一 9 态、流二 353 态 | O1：不比记账行；O3：投影不进记账树 |
| 102（抬 F 先写系统配置那一步不带新 F） | O2：I-7.12（系统配置 F 不低于同盘根上的 F）批 1 流二 12 态，第 60、73 段各 6 态；批 2 不红（取样点不敏感，见「三条全放行的 7 行」一节末） | O1：`raise_floor` 的答比的是写出的根（根带的 F 对），不比系统配置槽；O3：同第 86 行 |

O3 在 s4 计入分母的 19 行上一行都没红（`name=q1_column_red_rows … O3甲=0 O3乙=0`），而它在每行两条流的 450 个批 1 态上都给出了严格判定（`o3_judged_states_strict=450`，`name=q1_o3_coverage … rows_with_no_strict_judgement=0`）。由执行器的写法读出来的一条（读码，没另跑）：A(c) 只由这一行自己写出的根槽写构成，投影只有根的六样，所以 O3 只在「实现自己的恢复规则与条款不同、或一次发布在崩溃之后半施加」这一类上红；实现把某个字段一致地写错，O3 照样判「在」。O3 在坏镜像那一族（Q2）上红得多：P 行 26 张红 22、盲测 40 张全红、已知坏镜像 41 张严格红 28（`name=q2_family` 四行）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:116-119`（整段抄，未转述）**

```markdown
### s3 整表作废

s3 那一次整表（输出根 `research/target/e171-dumps/rows-s3/`）两条线道各自跨行复用编译目录；还原改过的文件用 `copy2`、同步用 `rsync -a`，都把修改时间带回快照那一刻，cargo 按修改时间判新旧，线道 1 跑过第 112 行（replay-prefix 的变异）之后、跑第 114 行之前的 45 行，编出来的装置都带着第 112 行的变异。登记修订 5 照实记了这 45 行、cargo 1.98.0 上的机理实测与驱动的改法；s3 整表、s3 的 Q3-核各趟与 s3 比对器产物不再当 Q1、Q3、Q4 的读数。留档不删：`research/results/` 下的 `e171-three-oracle-collusion-2026-10-03-s3-comparator.out`、`…-s3-rows-timing.out`、`…-s3-clean-check-comparator.out`（第五段对第 2、27、42、65 行的干净复核），与 `research/target/e171-dumps/rows-s3*`、`rows-s5-clean-check`。s3 读到的「Q1 全放行 40 行、分母 52」里 38 行是被带着跑的；s4 在修过的驱动上重跑全部 117 行，判决行 `rows_compiled_without_the_rebuild_check=0`。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:120-128`（整段抄，未转述）**

```markdown
### F4 扩表

登记第十节 F4：「触达行太少，Q1 的『没有』不够判：进第三段扩表（5.2），扩完仍不够记『不够判』，不记『没有』」，触发观测「Q1-分母 D < 30」；s4 的 D = 19。扩表照登记 5.2「扩表」：步长 4 的挑法比步长 8 多出来的 164 行（与步长 8 重叠的 56 行不重跑），按挑法脚本的输出顺序接在 117 后面编号 118–281、刀号 3。

- 行文件 `research/e7-index-bench/src/bin/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv`（sha256 `f1bd082e…`），由同一目录的 `expansion_table.py` 起挑法脚本两次（步长 8、步长 4）造出；表取 `crates/mutations.tsv` 去掉 E171 装置自己的变异行（与 A6、A7 同一个去法）。在 s4 冻住的树的那一份与今天仓里的那一份上造出来逐字节相同；按文件：admission 5、allocation_record_tree 2、allocator 12、block_device 2、code_two_tree 3、extent_tree 2、inode_tree 1、instance_table 2、journal 2、make_filesystem 6、mount 44、mounted_read 3、mounted_session 2、records 2、recovery 28、root_record 1、system_configuration 3、transaction 34、unit 2、replay-prefix 8。登记 5.9 只钉了 117 行的行文 sha256，扩表各行的行文摘要取造表那一刻的表。
- 逐行驱动加 `--other-selected-rows`：补位时跳过 117 行与扩表两张表里的每一个名字（登记 5.2「扩表与补位都不重挑已跑过的行」）。
- 比对器 `--rows-root` 可以给两份（117 行那份与扩表那份，同一个行号在两份里都有就拒），`--expansion-rows` 给扩表时多两行结果：`name=f4_expansion`（扩表与挑法脚本在 `--mutations-table` 那张表上现算的逐行相同、164 行、序号连续、刀号 3、与 117 行不重名）与 `name=q1_denominator_by_table`（分母按表分开再报）；Q1 的分母与三种读法、Q3 都按并起来的 281 行算。拿 s4 的输入重跑改过的比对器（只给 rows-s4），E7RESULT 行与 `…-s4-comparator.out` 逐字节相同（`cmp` 无输出）。
- 扩表那一趟在 s4 那一份冻住的树上跑（快照摘要 `d1586465…`，与 s4 五趟驱动的相同），两半才能并成同一批；树不重新冻。主 agent 照第七段交回的脚本跑完七步（空变异一行对 `baseline-run1` 未触达、扩表与挑法脚本在冻住的树的表上现算的逐字节相同、这一次两趟驱动与 s4 的快照摘要只有一种），扩表那一趟两条线道 7 607.9 秒。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:129-161`（整段抄，未转述）**

````markdown
### F4 扩表的结果（整行抄自产物）

`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out`（4698 行，末行 `E7RESULT name=done emitted=4698`；耗时行在 `…-s4-f4-rows-timing.out`，291 行）。并表、分母、三种读法、Q3、判决行，与扩表新出的全放行行（甲 5 行、丙多出的第 233 行）：

```
E7RESULT name=config segment=s4-f4 o3=frozen_executor baseline_run=baseline-run1 rows_root=rows-s4+rows-s4-f4 controls=8 disabled_runs=2 q3_core_comparisons=3
E7RESULT name=freeze_check phase=start reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=f4_expansion rows=164 expected=164 first_number=118 last_number=281 numbers_are_consecutive=true slice_3_rows=164 names_shared_with_the_117_rows=0 found_once_in_today_table=164 equal_to_the_selection_script_on_today_table=true
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=232 only_in_the_ignored_pass=0 compile_failed=6 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=12 rows_run=281 more_than_half=false
E7RESULT name=q1_column_red_rows counted_rows=37 O1=12 O1-对拍=11 O1-崩溃判官=4 O2=15 O3甲=0 O3乙=0
E7RESULT name=untouched_but_red rows=2 row_numbers=155,161
E7RESULT name=q1_alpha counted_rows=37 all_quiet_rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,233,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_beta counted_rows=37 all_quiet_rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,233,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_gamma counted_rows=37 all_quiet_rows=13 row_numbers=15,23,27,34,42,92,112,233,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=37 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle=none
E7RESULT name=q3_alpha caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=q3_beta caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=q1_denominator_by_table table=step_8_table rows_run=117 counted_D=19 counted_row_numbers=8,11,15,23,27,32,34,42,45,65,86,87,90,92,99,102,108,112,115
E7RESULT name=q1_denominator_by_table table=step_4_expansion rows_run=164 counted_D=18 counted_row_numbers=135,172,233,234,237,238,242,244,245,249,250,258,265,269,270,271,277,281
E7RESULT name=freeze_check phase=end reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=8 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 q3_core_runs=9 q3_core_runs_with_differences=0 rows_untouched_but_red=2 negative_bases_with_o2_red=0 negative_bases_with_o3_red=0 known_bad_rows_with_o2_quiet=0 o3_freeze_differences=0 o3_digest_mismatch_states=0 o3_dump_problem_lines=0 o3_image_digest_mismatches=0 rows_with_o3_freeze_lines_not_the_reference=0 rows_with_the_o3_executor_failed=0 a2_p_rows_not_expected_not_in=0 f3_p_rows_judged_in_by_o3=4 q2_geometry_unstable_kinds=2 q1_alpha_all_quiet_rows=12 q1_beta_all_quiet_rows=12 q1_gamma_all_quiet_rows=13 q1_o3_strict_reading_unreachable=0 q2_all_quiet_alpha=4 q2_all_quiet_beta=4 q3_alpha_rows_held_by_a_single_oracle=23 q3_beta_rows_held_by_a_single_oracle=23 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 rows_compiled_without_the_rebuild_check=0
E7RESULT name=q1_quiet_row reading=q1_alpha row=242 row_name=增补_3_第_4_件（增补_2_收口表第_40_行_/_C381_的前提）：发布的持久顺序把系统配置槽轮换挪到根槽_FUA_写之前，最后一步失败时根就不会已经落盘 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=516 o3_judged_states_provisional=516 first_operation_that_differs_from_the_baseline=8/8 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=269 row_name=C554_乙-配置续：见证值取成槽世代号（mkfs_之后第一次取号写的_tail_不再是_0，新池新建文件的字节变了） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=0/0 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=270 row_name=C577：发布在系统配置轮换之后不过屏障就返回（树表_0_条的一版上返回之后见证值可以落后） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=390 o3_judged_states_provisional=390 first_operation_that_differs_from_the_baseline=20/20 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=271 row_name=D8_已定项_8_头_ID：deleted_inodes_的树表条目头_ID_退回写_0（不写它服务的_inode_树的树_ID） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=44/44 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=277 row_name=取前缀核单测：施加一次发布之后新根段不换（停在所选根的） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_gamma row=233 row_name=由记录重建的根不照抄所选根的文件系统标识（代码轮_m3-formal-code-r1_K3，R3-1「其余字段等于所选根」）：改写全零 o1_comparison_red_calls=0 o1_crash_judge_red_states=45 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=true
```

分类的计数（一条命令从产物里数，原样输出）：`grep -o 'name=row row=[0-9]* .* category=[a-z_]*' research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out | sed -E 's/.*category=//' | sort | uniq -c` 输出 `6 compile_failed`、`37 counted`、`6 self_crash`、`232 untouched`（四行）。不进分母的 12 行：第 6、7、85、125、235、272 行实现自崩，第 33、59、127、163、206、243 行编不过（这一份是补位之前的读数，补位那一趟在「编不过的 6 行补位（已跑）」一节）。第 1–117 行的结果行（`name=row`、`row_touch`、`row_matrix`、`trajectory`、`row_o3`、`row_s5`）在 F4 产物与 s4 产物里逐字节相同（1819 行，`cmp` 无输出），7 行全放行的 `name=q1_quiet_row` 三种读法逐行相同：s4 那 7 行的三分不重做。Q2、F3 与对照的结果行两份也逐字节相同。拿第八段改过的比对器重跑 F4 与 s4 的输入，E7RESULT 行与两份产物各自逐字节相同。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:162-176`（整段抄，未转述）**

```markdown
### F4 新出的全放行行

读数取 F4 的导出目录（`research/target/e171-dumps/rows-s4-f4/row-<序号>/dump/`，与 `baseline-run1` 逐条 op、逐态比）；草稿脚本 `quiet_rows_f4.py`（第七段 `quiet_rows_s4_fua.py` 改输出根）、`extra_checks_f4.py`、`order_by_segment_f4.py`（都只读导出目录，不入库）。三分的口径与「三条全放行的 7 行」一节相同。顺序类的两行（第 242、270 行）写流的条数或次序变了，逐态与基线比没有意义，改用一条判法在这一行自己的态上数「条款不许的态」，同一判法在基线上数一遍都是 0（判法分得出）：切段用比对器自己的 `baseline_checks.segments_of_the_write_stream`（FUA 写放行它那块盘、屏障放行它那块盘）。

| 行（变异） | 和基线不同在哪 | 条款不许的态（批 1 / 批 2） | O1 / O2 / O3 看不看得到 | 三分 |
|---|---|---|---|---|
| 242（发布的持久顺序把系统配置槽轮换挪到根槽 FUA 写之前） | 两条流第 8 条 op 起：每次取号、暖机、发布末尾的「根槽 FUA → 两块盘的系统配置槽 → 屏障」换成「两块盘的系统配置槽 → 根槽 FUA → 屏障」，写的内容逐条相同、次序变了（流一 9 条、流二 57 条 op 按下标比不同）；基线里根槽 FUA 自成一段，变异把它与系统配置写切进同一段（流一 8 段里 3 段、流二 59 段里 19 段，基线 0 段）；调用结局同 | 同一段里系统配置写已落、根槽 FUA 没落全：流一 21 / 3 态，流二 133 / 19 态；基线 0 | O1-崩溃判官：根槽没落，恢复择到前一版，前一版是模型提交过的、不比已持久的最新根槽旧，判绿；O1-对拍：调用结局同；O2：这一行 0 态红（单个镜像里系统配置槽比根早一次发布，没逐条查 invariants.md 有没有管这一样的）；O3：A(c) 由这一行自己写的根构成，恢复落到的版本在里面，「在」 | **真盲区候选**：D16（发布语义） 已定项 7 定案「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」「系统配置槽在根槽持久之后再更新」（`.claude/kb/decisions/16-发布语义.md` 第 181 行）；系统配置先于它见证的根落盘的态，照条款的写序造不出来，三条都放行 |
| 269（见证值取成槽世代号） | 流一第 0、1 条 op（mkfs 之后第一次取号写的两块盘系统配置槽）偏移 469 一字节 0x00 → 0x01，连同校验和（155–158）；流二另有第 92、93 条 op（可写挂载取号那一写）0x04 → 0x06。偏移 469 是 journal tail（`crates/singlefs-checker/src/lib.rs` 第 200 行 `SYSTEM_CONFIGURATION_TAIL_OFFSET`）；调用结局同；看 journal 那一遍恢复逐态同 | 镜像与基线不同的态：流一 31 / 0，流二 78 / 14（全持久态相同：之后的发布把那一槽写成正确的 tail） | O1：`ObservedRoot` 不带系统配置，恢复不先信 tail（D23（journal 的角色与格式） 已定项 3），读回同；O2：读了 tail（`crates/singlefs-checker/src/lib.rs` 第 285 行），没有一条不变量判它的值；O3：执行器只在锚点断言里读 tail（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/executor_main.py` 第 180 行，`anchor_assertions` 里），只核全持久态，判定不用它 | **真盲区候选**：D23（journal 的角色与格式） 已定项 14 写「取号那一写……改写取号那一刻按同一取法读到的见证值 c_见证」（全部自证过的槽 journal_tail 取最大）、「mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：新池新建文件的字节不变」（`.claude/kb/decisions/23-journal的角色与格式.md` 第 392 行）；tail 写成 1 的系统配置槽不在条款写得出的镜像里，三条都放行 |
| 270（发布在系统配置轮换之后不过屏障就返回） | 每次发布末尾那道屏障没了：流一 57 → 53 条 op、流二 595 → 565 条，之后的返回点整体前移；写的内容同；系统配置轮换与下一次调用的第一批写切进同一段 | 一次调用的系统配置写没落全、之后的调用的写已落：流一 7 / 6 态，流二 70 / 59 态；基线 0 | O1-崩溃判官：那次发布的根槽 FUA 已落，恢复择到它，判绿；O1-对拍：调用结局同；O2：系统配置落后根一次发布在单个镜像里合法（I-7.12 只比 F）；O3：「在」 | **真盲区候选**：D16（发布语义） 已定项 7 定案「根槽（FUA）→ 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回」（第 181 行），射程「屏障口径」写明系统配置轮换之后那一道屏障是「发布返回前加屏障」（第 186 行）；返回之后系统配置还没落、后面的写先落的态照条款造不出来，三条都放行 |
| 271（deleted_inodes 的树表条目头 ID 写 0） | 两条流第 44、45 条 op（树表单元，两块盘各 16 KiB）偏移 1647、1649 两字节 0x01 → 0x00（头 ID 65537 → 0），连同单元校验和；指向它的节点（第 48、49 条 op）与根槽（第 52 条 op）的指针与校验和随之变；流二之后重写树表的发布都带着（85 条 op 不同）；调用结局同；恢复逐态同 | 择到新版（根 txg 不小于第一处不同之后写出的第一条根）：流一 12 / 0 态，流二 356 / 112 态 | O1：不读树表；O2：树表条目只解树 ID、种类、诞生 txg（`crates/singlefs-checker/src/walk.rs` 第 2953、2955 行），头 ID 不判；O3：投影只带树表单元指针，A(c) 是这一行自己写的根，「在」 | **真盲区候选**：D8（核心索引结构） 已定项 8 定案「头 ID 8 字节的取值：……`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537）」（`.claude/kb/decisions/08-核心索引结构.md` 第 250 行）；择到新版的态里头 ID 是 0，三条都放行 |
| 277（取前缀施加一次发布之后新根段不换） | 写流与调用结局全同；看 journal 那一遍恢复在流一批 1 6 态、流二批 1 51 态（批 2 0 态）不同：基线施加 1 条记录、落到新的一版，变异同样施加 1 条、却停在所选根那一版（例：流一第 41 态基线 `1,3 file_read e8ca4bf8…`，变异 `1,2 no_file`） | 0：57 态里被丢掉的那一版的调用都没返回、它的根槽都没落（`extra_checks_f4.py`：全是 `call_returned=False its_root_slot_landed=False`） | O1-崩溃判官：前一版是模型提交过的、不比已持久的最新根槽旧、读回与那一版相同，判绿是对的；O2、O3：镜像与基线逐字节相同 | **另一类：三条不判实现恢复里靠记录重建的那一段**：调用没返回，新旧两版都在「根 + journal 任意前缀」（D16（发布语义） 已定项 12）里，态本身合法；错在实现的恢复施加记录之后新根段不换，违反 D13（验证路线） 已定项 1 的 M4 与 D23（journal 的角色与格式） 已定项 15（同第 27 行），三条不判；M4 的落点是 E165 与 Verus。会分开的历史（推的，没跑）：一次已返回的发布之后根槽读不出、恢复要靠 journal 重放追上那一版（D16（发布语义） 已定项 7 射程「同一实例里退一代，靠 journal 重放追得上」），变异会停在前一版；两条流里没有读坏根槽的态 |

- **第 233 行（由记录重建的根不照抄所选根的文件系统标识，改写全零），只在丙里全放行**：写流与调用结局全同；看 journal 那一遍恢复在施加了记录的态上走读失败（`InvariantViolated { invariant: "I-1.4", detail: "树根 fsid 不符" }`），流一批 1 3 态、流二批 1 48 态（其中 6 态写落在卸载里、崩溃判官记不适用）。甲、乙里 O1-崩溃判官判红（流一 3 态、流二 42 态，段为正 2/12、28/79），只靠 O1；丙只取 O1-对拍，调用不经恢复，三条全放行。在丙里是**真盲区候选**：实现在合法的崩溃镜像上恢复不出任何一版，D13（验证路线） 已定项 1 的 M2 写重放结果的根「其余字段等于所选根」（`.claude/kb/decisions/13-验证路线.md` 第 32 行）；O2 判恢复之前的镜像（与基线逐字节相同），O3 跑的是它自己的恢复。层 0 那份 oracle 与记录核对器在这一行上红（`layer0_or_record_checker_red=true`，不是三条之一）。批 2 不红：两条流批 2 一个「基线施加了记录」的态都没有（`extra_checks_f4.py` 输出 `b2_base_records_applied_gt0: 0`），**取样点不敏感**，会分开的取样点是任一个施加了记录的态；这一格两批翻面，记「不稳定」。
- **第 265 行（抬 F 那一串不先写系统配置），批 2 多出来的全放行**：只有流二第 440 条 op 起不同（抬 F 先写系统配置那一步整步没了，149 条 op 按下标比不同）；O2 在批 1 判红 12 态，都是 I-7.12（系统配置 F 不低于同盘根上的 F），在第 59、71 段各 6 态（`name=trajectory row=265 stream=fixed_script_through_e_then_unmount oracle=O2 total_red_states=12 peak=6 positive_segments=2 of_segments=77 …`）。批 2 在流二取的段是 7、11、16、20、24、28、32、36、40、44、48、52、56、60、64、68、72，没有第 59、71 段（`extra_checks_f4.py` 输出 `batch2_states_in_segments_59_71=0`）：**取样点不敏感**，同第 102 行；记「不稳定」。条款是 D16（发布语义） 已定项 7 定案「抬 F 那一串……先把新 F 写进每块盘的系统配置、过一道屏障」。Q1、Q3 的结论字在批 2 上不翻（批 2 全放行 15 行，只靠一条的行仍 ≥ 1）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:177-193`（整段抄，未转述）**

```markdown
### 三轮判决之后的三分

三轮结论判决（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`；第三轮是最终轮，以它第三节「三轮之后的结论」与第五节为准）把这批读数重新三分。口径：**真盲区候选**＝这一批取到的态在已定条款下不合法、三条都放行（E171 s4 树）；**另几类**＝三条按设计不判、或错不在态上。执行员原判「对这一批等价」的五行（第 15、27、34、112、277 行）一行都不再是等价。

| 类 | 行 | 一句话依据（各行条款原文在上面两节的行里） | 三轮里的翻动 |
|---|---|---|---|
| 真盲区候选 | 第 15 行 | D3（空间分配） 已定项 10 ⑤ 的 t2–t12 落点：变异下十一个提交内生块整体挪 64 槽（`research/prompts/e171-conclusion-r3-opus-model/out/row15-offsets.out`），⑤ 与 `layout/01-first-txn.md` 第 307 行钉的是这些落点 | 第一轮从「对这一批等价」挪进来（D3 已定项 10 ③）；第二轮维持，依据写 ⑤ 落点表与 `layout/01-first-txn.md` 第 309 行的 t1@50180；第三轮留名单、依据换成 ⑤ 的 t2–t12 落点（主 agent 现查 ⑤ 表 t2@50240 与第 220 行「改新池新建文件的字节：是」，复跑 `row15-offsets.out` 同） |
| 真盲区候选 | 第 23、42、271 行 | 第 23 行 D8（核心索引结构） 已定项 6 的容器号；第 42 行 D16（发布语义） 已定项 1 卸载抬 F；第 271 行 D8（核心索引结构） 已定项 8 头 ID | 三轮都在名单 |
| 真盲区候选 | 第 242、269、270 行 | 第 242、270 行 D16（发布语义） 已定项 7 的写序；第 269 行 D23（journal 的角色与格式） 已定项 14 的见证值 | 第一轮被改出名单（按已定项 12 合法、写序义务无人判），第二轮翻回，第三轮没被打中 |
| 另一类 | 第 27、112、277 行 | 三条不判实现恢复里靠记录重建的那一段（D13（验证路线） 已定项 1 的 M4、D23（journal 的角色与格式） 已定项 14 注 1 与已定项 15）；第 112 行已在 `.claude/gate.d/e165-pinned-red-examples.tsv` 里 | 第一轮采正推改归这一类，第二轮站住、措辞收窄，第三轮没变 |
| 另一类 | 第 34 行 | 分配政策，镜像合法：D3（空间分配） 已定项 8 射程 | 第一轮与第 15 行一起挪进真盲区候选，第二轮翻回，第三轮维持 |
| 另一类 | 第 92 行 | 条款在字节表与里程碑验收，决策正文欠一句 | 第一轮归「条款没定且不可见」不进名单，第二轮改理由，第三轮维持 |
| 另一类（零轮） | 第 110、162、174 行 | 运行时读数（写账、分支名），三条按设计都不判，与第 34 行同类 | 第三轮新冒出：攻方量交回值时发现（「没触达却红的两行与未触达行里交回值不同的七行」一节） |
| 另一类（零轮） | 第 49、252 行 | 交回值或内存状态错、写流不变，要更长的历史才分得开 | 同上，第三轮新冒出 |

第 155、161 行是已知的那两行（O1-对拍判红），不在这几类里；第 233 行（丙里全放行、甲乙里 O1-崩溃判官抓到）、第 265 行（批 2 取样点不敏感）的读法在「F4 新出的全放行行」一节，第 102 行（批 2 取样点不敏感）在「三条全放行的 7 行」一节末，三轮没有改。「另一类（零轮）」两行的意思是这一类是第三轮才冒出来的，没有再开轮，进欠账，不改登记的触达定义。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:194-214`（整段抄，未转述）**

```markdown
### F4 新出的只靠一条的 12 行

只写够不到的对象与没判的字段，不判结论。读数取 F4 的导出目录（草稿脚本 `caught_rows_f4.py`，第七段 `caught_rows_s4.py` 改输出根，不入库）与产物的 `name=trajectory`、`name=row_matrix` 行；段为正的数是「红态的段数 / 这条流的段数」。

| 行（变异） | 那一条红在哪 | 另外两条为什么看不到 |
|---|---|---|
| 172（交进来的盘数等于 w 的下限也拒） | O1-对拍：流二第 4 次调用 `mount_writable` 交回 `err:WritableDeviceCountBelowTheStripeWidthLowerBound`，模型说该成；流二停在第 92 条 op | 被拒之前的写流与基线逐条相同（0 条 op 不同），取到的镜像都是基线那一批的前缀；被拒的调用不留镜像，O2、O3 没有对象可判（同第 45 行） |
| 233（由记录重建的根不照抄所选根的 fsid） | O1-崩溃判官：批 1 流一 3 态、流二 42 态走读失败（I-1.4 树根 fsid 不符），段为正 2/12、28/79；批 2 0 态 | 错在实现的恢复，写流与基线逐字节相同；O2 判恢复之前的镜像，O3 跑它自己的恢复。批 1 与批 2 红的列不同：批 2 没有施加了记录的态，取样点不敏感（「F4 新出的全放行行」一节） |
| 237（解系统配置槽时根环起点读错了位置） | O1-崩溃判官：批 1 流一 53 态、流二 73 态（段为正 12/12、16/16），批 2 7、14 态，「没择到根」（`SystemConfigurationValueRefused … RootRingBaseNotTheFirstVersionSlot`）；O1-对拍：流二第 4 次调用 `mount_writable` 交回 `err:Recovery` | 错在实现的读者一侧，写出的字节与基线相同（0 条 op 不同）；O2、O3 各用自己的解析器读系统配置，看到的镜像与基线逐字节相同 |
| 249（一个数据单元的发布也切成两条记录） | O1-对拍：两条流第 2 次调用 `publish` 判 `disagrees:jsn`（模型说那一版的 jsn 3，实现 4），模型停下 | 两条记录合起来仍是一次完整的发布：O2 在这一行 0 态红（没逐条查 invariants.md 有没有限一次发布几条记录的）；O3 自己重放记录，落到的版本在 A(c) 里，投影不带 jsn |
| 250（数据单元的净荷字节倒序写） | O1-对拍：两条流第 2 次调用 `publish` 判 `disagrees:这一版的文件内容`（读回 3000 字节、内容倒序） | 单元的校验和按写下去的字节算，结构自洽：D13（验证路线） 已定项 6 表 O2 行「验不到」一栏写的「一个块被覆盖成另一份合法内容」；O3 的投影不带文件内容，不拿写入的内容去比 |
| 281（择根取第一个自证过的槽就不再换） | O1-崩溃判官：批 1 流一 41 态、流二 61 态（段为正 9/12、13/16），批 2 7、14 态，「走到实例 0 第 0 代根」（比盘上已持久的最新根槽旧）；O1-对拍：流二第 4 次调用 `mount_writable` 交回 `err:NewerStateStillUnreadableAfterOneReread` | 错在实现的择根，写流与基线逐字节相同（0 条 op 不同）；O2 判镜像结构，O3 用自己的择根规则恢复，看到的镜像与基线相同 |
| 135（复活 defer 里那一条时 defer 槽数不减） | O2：I-3.11（已分配减 defer 等于最新根走读）批 1 流二 225 态，段为正 44/79；批 2 70 态 | O1：`ObservedRoot`（`crates/singlefs-harness/src/model.rs` 第 594 行）没有记账行；O3：投影不进记账树 |
| 234（根记录写者不照卸载记号写 flags） | O2：I-7.9（回退下界 F 不高于抬 F 的上限，按卸载记号分两支）批 1 流二 29 态，第 73 段起段为正 6/79；批 2 7 态 | O1：写落在卸载那次调用里，模型没有卸载的答（登记读法 6），对拍不比、崩溃判官记不适用，`ObservedRoot` 也不带根的 flags；O3：投影不带卸载记号（`root_ring.py` 第 22 行起的说明：「卸载记号与根记录其余字段不进投影」） |
| 238（改动计数留 1） | O2：I-9.26（inode 改动计数落在出生代与容器诞生代号之间）批 1 流二 333 态，段为正 65/79；批 2 105 态 | O1：不比 inode 记录的改动计数；O3：投影不进 inode 树 |
| 245（覆盖写的反向链取上一版记录自己的反向链） | O2：I-8.6（反向链不等的记录不进重放）批 1 流二 336 态，段为正 67/79；批 2 105 态 | O1：`ObservedRoot` 只带 journal 计数器，不带反向链；O3：它的前缀规则核反向链（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/replay_prefix.py` 第 188 行，不等就停），这一行严格、暂代都判「在」；停下之后落到的那一版仍在 A(c) 里是推的（根槽已落、不靠重放），没逐态查它停在哪 |
| 258（不释放旧落点） | O2：I-3.1、I-3.11 批 1 流二 333 态，段为正 65/79；批 2 105 态 | O1：`ObservedUnitAllocationRecords` 只带这一版每个角色的单元的分配记录（`model.rs` 第 638 行），旧落点释没释放不在里面；O3：投影不进分配记录树与记账树 |
| 265（抬 F 那一串不先写系统配置） | O2：I-7.12 批 1 流二 12 态（第 59、71 段各 6 态）；批 2 0 态 | O1：比的是根带的 F，不比系统配置槽；O3：投影的 F 取自根，A(c) 是这一行自己写的根。批 1 与批 2 红的列不同：批 2 没取第 59、71 段，取样点不敏感（「F4 新出的全放行行」一节） |

O3 在 F4 计入分母的 37 行上一行都没红（`name=q1_column_red_rows … O3甲=0 O3乙=0`），每行两条流的批 1 态都给了严格判定（`name=q1_o3_coverage … rows_with_no_strict_judgement=0`）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:215-229`（整段抄，未转述）**

````markdown
### 没触达却红的两行（第 155、161 行）与未触达行里交回值不同的七行

`name=untouched_but_red rows=2 row_numbers=155,161`：两行的变异是同一处（`crates/singlefs-core/src/mount.rs` 里抬 F 上限 `newest_on_every_device.min(fourth_newest)` 改成 `.max(…)`，两行原文只差行首空白），触达三样都不同（`name=row_touch row=155 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false … touched=false`），O1-对拍在流二第 12 次调用 `raise_floor` 判 `disagrees:抬 F 的上限`（`call-outcomes.tsv`：模型「上限 Some(11)（D16（发布语义） 已定项 1）」，实现「实现报上限 13」），批 1、批 2 都红；O2、O3 不红。

- 不是装置、比对器或取样的缺陷：比对器的触达照登记第一节读法 4 字面算（「任一次调用的结局（成功 / 哪一种错误）」，`collusion_matrix.py` 的 `outcome_words` 只比调用名与结局列），这一次调用两边都是 `ok`；O1-对拍比的是成功的调用交回的值（这里是抬 F 的上限），写流不变是因为这一次请求的 F 不超过 11，上限放宽没起作用。O1-对拍按调用算，与取样点无关，两批都红。
- 缺口在登记读法 4 的触达定义：它把「成功的调用交回的值」排在三样之外，却写「三样都相同的行……在这一批上与正常实现逐字节同行为，哪条 oracle 都不该红」；这两行交回值不同，那句前提不成立。装置的 `call-outcomes.tsv` 只在对拍判出不同时记两边的值，成功的调用交回什么没有单独记，入库装置没有这一列。触达定义没改（产物跑过之后不改登记），分母仍 37。
- 别的未触达行有没有同类差异：攻方（第三轮）在 s4 树的副本上量了（副本探针，s4 树，入库装置没有这一列）。探针照流二的驱动在内存稀疏盘上跑一遍，记每次成功调用交回值的 `{:?}` 文本摘要、末了的分配器与现行那一版；基线与 E171 导出的 595 条 op 逐条相同；再把未触达的行逐行打进副本、重编重跑、还原。跑了 232 行（238 行去掉 `make_filesystem.rs` 的 6 行，mkfs 不是一次调用，探针不记它的交回值，第 149–154 行没跑），7 行不同：第 49、110、155、161、162、174、252 行。第 155、161 行就是上面那两行（这把尺子在同一次第 12 次调用 `raise_floor` 上抓到它们，上限 11 → 13，是现成的阳性对照）；另 5 行 O1-对拍不比那几个字段所以不红：第 49 行可写挂载交回的见证值 4 → 3，第 110 行写账的结构种类，第 162 行影子账分支名，第 174 行抬 F 交回的写账，第 252 行内存里的分配器少了根槽登记（第 49、252 行交回值或分配器不同，写流不变）。汇总行整行抄自攻方 `research/prompts/e171-conclusion-r3-opus-model/out/return-rows.out` 第 239 行：

```
name=return_summary hidden_difference=7 identical=225 no_output=6
```

  主 agent 复跑（`tools/rerun.sh … all`）这 7 行逐字同；`return-rows.out` 只差变异表第 28–31 行那四条 `name=return_row`（复跑是 `status=no_output`、汇总 `hidden_difference=7 identical=221 no_output=10`，攻方是 `identical`）：攻方在决定去掉 `make_filesystem.rs` 的行之前已跑过这四行，`rerun.sh` 的 `awk` 把它们滤掉，复跑就不再跑（`research/prompts/e171-conclusion-r3-main-verification.md` 第四节）。害处没量：第 49、252 行推的是要更长的历史才分得开。
- 对其余读数的影响：两行不进分母，也不进任何 Q1、Q3 名单。照「触达」把它们算进去，D = 39，两行都只有 O1 红，Q3 只靠 O1 的行多两行（第 155、161 行），Q1 三种读法的全放行行不变。把 7 行交回值不同的行与第 125 行（「它答不了的」）都算进来，D 由 37 变 45（推的，算术，没在入库装置上跑），那 5 行 O1 不红，会成为新的全放行行。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:230-257`（整段抄，未转述）**

````markdown
### 编不过的 6 行补位（已跑）

登记第一节读法 5「替换之后编不过（`.claude/rules/mutation-sampling.md` 第八类，按 5.2 的补位规则换行）」；s4 与 F4 那一版驱动只在编之前的失败上补位，第 33、59、127、163、206、243 行编不过之后照原样记编不过、没有补位。第八段把驱动改成编不过之后也补位（换同一文件组里下标 j + 1、两张表里都没有、这一次跑里别的补位也没用过的那一行，换上的行编之前照样核锚点，编不过再换，编之前的失败与编不过合起来一行至多三次；组里不含 E171 装置自己的变异行），加 `--rerun-compile-failed-from`（只跑几个输出根里结局是编不过的行）；比对器加 `--substitution-rows-root`（补位那一趟的行换掉原来编不过的同号行，别的情形拒），多出 `name=substitution` 与 `name=substitution_summary` 两种结果行。

- 补位那一趟在 s4 那一份冻住的树上跑（快照摘要与 s4 五趟的相同，空变异一行对 `baseline-run1` 未触达），只跑两份输出根里结局是编不过的行（`rerun_compile_failed rows=33,59,127,163,206,243`），两条线道、每条 2 线程，六步都退 0，补位 6 行挂钟 1 277.9 秒；比对器把 117 + 164 + 补位并成 `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out`（4795 行，末行 `E7RESULT name=done emitted=4795`），耗时行在 `…-s4-f4-substitutes-rows-timing.out`（292 行）。
- 补位的六行（整行抄自 `…-s4-f4-substitutes-comparator.out` 第 4625–4630 行）、补位汇总（第 4631 行）与分母（第 4521 行）：

```
E7RESULT name=substitution row=33 original_name=步_4：回退候选集的_F_用最新根自己带的_F_而不是_F_生效 original_status=compile_failed substituted_by=步_4：影子账把被抛弃根账里已释放的落点也隔离 failed_attempts=1 attempts=compile_failed:步_4：回退候选集的_F_用最新根自己带的_F_而不是_F_生效 category=untouched
E7RESULT name=substitution row=59 original_name=C483_②：挂载态打开时_inode_树根只按位置提示读（不经映射回退） original_status=compile_failed substituted_by=C483_②：挂载态打开时_inode_叶容器只按位置提示读（不经映射回退） failed_attempts=1 attempts=compile_failed:C483_②：挂载态打开时_inode_树根只按位置提示读（不经映射回退） category=untouched
E7RESULT name=substitution row=127 original_name=增补_2：回落只看已分配位（发出影子账隔离的槽） original_status=compile_failed substituted_by=增补_2_第_20c_行：用户数据只排除当前那一个开放段（回落把开放段置空之后，那一段对用户数据开放） failed_attempts=2 attempts=compile_failed:增补_2：回落只看已分配位（发出影子账隔离的槽）|compile_failed:增补_2：回落只看已分配位（抬_F_的空发布发出扣住的槽） category=untouched
E7RESULT name=substitution row=163 original_name=C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷） original_status=compile_failed substituted_by=D8_已定项_8_②：可写挂载写行那次的树_ID_水位只沿所选那一版带，不取根环与环里记录的_max（原为_C511_第_3_步回退到暖机根那一格；回退改成向前发布之后由崩溃恢复丢掉一次第一个文件版本那一形护） failed_attempts=1 attempts=compile_failed:C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷） category=untouched
E7RESULT name=substitution row=206 original_name=步_6：恢复从系统配置_tail_起逐条验证记录的点名单元、失配即中止（E78_的自我中止形态） original_status=compile_failed substituted_by=增补_3_第_4_件：一块盘两个系统配置槽都无效就整体_return_Err（每盘一份的冗余当场作废，D22_已定项_8_第_1_条） failed_attempts=1 attempts=compile_failed:步_6：恢复从系统配置_tail_起逐条验证记录的点名单元、失配即中止（E78_的自我中止形态） category=untouched
E7RESULT name=substitution row=243 original_name=并行线三_C116：叶容器被_COW_重写时身份跟着这次发布换出生代 original_status=compile_failed substituted_by=普查_R2：释放判定路径把「映射条目切不动」报成「不在映射」（两件要分流的事并成一个成员） failed_attempts=1 attempts=compile_failed:并行线三_C116：叶容器被_COW_重写时身份跟着这次发布换出生代 category=untouched
E7RESULT name=substitution_summary rows_that_did_not_compile=6 substituted_rows=6 still_not_compiled=0 row_numbers=33,59,127,163,206,243
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=238 only_in_the_ignored_pass=0 compile_failed=0 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
```

- 六行换上的都是 `category=untouched`（第 127 行前一个换上的行也编不过，换了第二次）；分母 D = 37 没变：`compile_failed` 6 → 0、`untouched` 232 → 238，`name=f6_table_failures` 的 `rows_not_counted_for_the_table` 12 → 6（只剩六行实现自崩）。补位产物比补位之前的 F4 产物多出来的行只有这六行的 `name=row`、`row_touch`、`row_matrix`、`row_s5`、`row_o3` 与 `trajectory`，加 `name=substitution` 六行与 `name=substitution_summary`，改了的行是 `name=config`、`name=q1_denominator`、`name=f6_table_failures` 与 `name=done`。
- Q1、Q3 名单与补位之前逐字同（自己 diff 两份产物核）：

```
cd research/results; A=e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out; B=e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out; P='name=(q1_alpha|q1_beta|q1_gamma|q1_quiet_row|q1_column_red_rows|q3_alpha|q3_beta|untouched_but_red|q1_denominator_by_table|q1_delta|q1_o3_coverage) '; diff <(grep -E "$P" $A) <(grep -E "$P" $B); echo "diff_rc=$?"; grep -cE "$P" $A; grep -cE "$P" $B
```

  输出 `diff_rc=0`、`48`、`48`；`name=q2_`、`name=f3_`、`name=verdict`、`name=f4_expansion`、`name=q1_denominator_by_table`、`name=positive_control`、`name=freeze_check`、`name=q3_core_row` 开头的行两份也逐行同（同一条命令把 `P` 换成 `'name=(q2_|f3_|verdict|f4_expansion|q1_denominator_by_table|positive_control|freeze_check|q3_core_row)'`，输出 `diff_rc=0`、`171`、`171`）。
- 拿改过的比对器（不给补位根）重跑 s4 与 F4 的输入，E7RESULT 行与两份产物各自逐字节相同；拿一个手造的补位根（一行换成第 34 行的导出、一行仍编不过）空跑过一遍，分母与名单照换过的行算、两种新结果行照写（草稿，不入库）。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:258-286`（整段抄，未转述）**

````markdown
### Q2 里 O2 放行而 O3 判不在的坏镜像（整行抄自产物）

`…-s4-f4-substitutes-comparator.out` 第 4759、4760 行（补位之前的 `…-s4-f4-comparator.out` 第 4662、4663 行同两行，`sed -n '4662,4663p'` 与 `sed -n '4759,4760p'` 各自喂 `md5sum` 都是 `07b56b24014dac5d9eb543ef15384aa9`）：

```
E7RESULT name=q2_family family=P rows=26 o2_red=10 o2_quiet=16 o3_strict_red=22 o3_provisional_red=22 o2_only_strict=0 o3_only_strict=12 both_red_strict=10 o2_only_provisional=0 o3_only_provisional=12 both_red_provisional=10 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:2,new:3,new:5,new:7,new:8,new:9,new:10,new:11,new:12,fix:2,fix:3,fix:5,fix:8,fix:9,fix:11,fix:12
E7RESULT name=q2_family family=B rows=40 o2_red=23 o2_quiet=17 o3_strict_red=40 o3_provisional_red=40 o2_only_strict=0 o3_only_strict=17 both_red_strict=23 o2_only_provisional=0 o3_only_provisional=17 both_red_provisional=23 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:18,new:19,new:20,new:23,new:26,new:28,new:30,new:32,new:33,new:35,fix:21,fix:22,fix:23,fix:24,fix:25,fix:32,fix:35
```

12 + 17 = 29 张是 O2 放行而 O3 严格判「不在」。从 `name=q2_row` 逐行数同一批、按补丁种类分（命令与原样输出；`B` 是上面那份产物）：

```
grep 'name=q2_row family=[PB] ' B | grep ' o2=green ' | grep -c 'o3_strict=not_in'
29
for f in P B; do echo "== $f"; grep "name=q2_row family=$f " B | grep ' o2=green ' | grep 'o3_strict=not_in' | sed -E 's/.*bad_name=([^ ]*) .*/\1/' | sed -E 's/^B[0-9]+:(P[0-9]+):.*/\1/; s/^(P[0-9]+)-.*/\1/' | sort | uniq -c; done
== P
      1 P2
      4 P3
      4 P4
      1 P5
      2 P9
== B
      4 P2
     12 P3
      1 P5
```

P 族 12 张是 5 种补丁（P2、P3、P4、P5、P9）的实例，盲测 B 族 17 张是 3 型（P2、P3、P5）的随机实例；全是结构自洽的伪造根、伪造续接镜像。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:287-389`（整段抄，未转述）**

````markdown
### 第一段的基线与对照（整行抄自 s1 产物）

`research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out`（基线第一趟；两趟的 E7RESULT 行逐字节相同，`diff` 无输出）：

```
E7RESULT name=stream stream=new_pool_file_creation ending=completed exported=true not_exported_why=- operations=57 writes=41 segments=11 tearable_writes=8 calls=3 full_states=16777260 quick_tier_states=46 sampled_states=82 model_stopped_at_call=none
E7RESULT name=batch stream=new_pool_file_creation batch=1 states=53 segments_touched=12 states_with_a_torn_write=20 o1_crash_judge_red=0 o1_crash_judge_green=53 o1_crash_judge_not_applicable=0 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=new_pool_file_creation batch=2 states=7 segments_touched=1 states_with_a_torn_write=0 o1_crash_judge_red=0 o1_crash_judge_green=7 o1_crash_judge_not_applicable=0 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=stream stream=fixed_script_through_e_then_unmount ending=completed exported=true not_exported_why=- operations=595 writes=477 segments=78 tearable_writes=46 calls=15 full_states=1662648564 quick_tier_states=278 sampled_states=890 model_stopped_at_call=14
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=1 states=397 segments_touched=79 states_with_a_torn_write=115 o1_crash_judge_red=0 o1_crash_judge_green=349 o1_crash_judge_not_applicable=48 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=2 states=119 segments_touched=17 states_with_a_torn_write=0 o1_crash_judge_red=0 o1_crash_judge_green=105 o1_crash_judge_not_applicable=14 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0
```

`research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out`，基线与对照（Z1 两趟 26 个文件逐字节相同、S2 与 E170（O3规约执行器第一版） 导出装置 18 项逐字节相同、S5 两个数比对器自己现算、A3 基线 0 红）：

```
E7RESULT name=a4 table_rows=117 expected=117 per_file_matches_the_registration=true found_once_in_today_table=117 line_unchanged_since_the_registration=117
E7RESULT name=a5 first_slice_rows=24 expected=24 distinct_files=24
E7RESULT name=a6 step_8_rows=117 step_4_rows=220 expected_step_4=220 overlap=56 expected_overlap=56 step_4_only=164 expected_step_4_only=164 step_8_names_equal_the_registered_table=true
E7RESULT name=a7 implementation_rows=844 expected=844 files=24 expected_files=24
E7RESULT name=a3_baseline_reds o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0
E7RESULT name=s5_batch_1_states stream=new_pool_file_creation recomputed=53 device=53 equal_counts=true equal_ordinals=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=1 recomputed=53 device=53 equal=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=2 recomputed=7 device=7 equal=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=3 recomputed=22 device=22 equal=true
E7RESULT name=s5_batch_1_states stream=fixed_script_through_e_then_unmount recomputed=397 device=397 equal_counts=true equal_ordinals=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=1 recomputed=349 device=349 equal=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=2 recomputed=105 device=105 equal=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=3 recomputed=330 device=330 equal=true
E7RESULT name=z1_summary files_compared=26 files_differing=0
E7RESULT name=s2_summary files_compared=18 files_differing=0
```

阳性对照（登记 5.6，每条 oracle 在基线与空变异各一趟）与关一条的开关自检：

```
E7RESULT name=positive_control oracle=O1-对拍 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O1-对拍 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=baseline red_units=2 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=empty_mutation red_units=2 red=true
E7RESULT name=positive_control oracle=O2 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O2 run=empty_mutation red_units=1 red=true
E7RESULT name=q3_core closed=O1 run=disable-O1 differences_outside_the_closed_column=0 where=none
E7RESULT name=q3_core closed=O2 run=disable-O2 differences_outside_the_closed_column=0 where=none
```

三行试跑的分类、触达三样与 Q1 的分母：

```
E7RESULT name=row row=0 row_name=空变异（C318_实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒_0（影子账隔离的单元又不进不等式）） category=untouched substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=0 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false ignored_pass_differs=false touched=false
E7RESULT name=row row=1 row_name=C318_实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒_0（影子账隔离的单元又不进不等式） category=untouched substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=1 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false ignored_pass_differs=false touched=false
E7RESULT name=row row=6 row_name=K1：分配记录树按槽号找叶时叶宽算成两倍（记录进了不罩它的叶，按位置寻址的核判出） category=self_crash substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=6 write_stream_differs=true call_outcomes_differ=true recovery_report_differs=false ignored_pass_differs=false touched=true
E7RESULT name=q1_denominator rows_run=3 counted_D=0 untouched=2 only_in_the_ignored_pass=0 compile_failed=0 anchor_not_unique=0 not_found=0 self_crash=1 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=false
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=1 rows_run=3 more_than_half=false
E7RESULT name=q1_o1_and_o2_both_quiet counted_rows=0 rows=0 row_numbers=none o3=not_run
E7RESULT name=q1_o1_and_o2_both_quiet_batch_2 rows=0 rows_that_differ_from_batch_1=none
E7RESULT name=q1_column_red_rows counted_rows=0 O1=0 O1-对拍=0 O1-崩溃判官=0 O2=0
E7RESULT name=untouched_but_red rows=0 row_numbers=none
```

Q2 的 O2 一列（O3 两列、盲测族没跑）：

```
E7RESULT name=q2_family family=P rows=26 o2_red=10 o2_quiet=16 o2_quiet_rows=new:2,new:3,new:5,new:7,new:8,new:9,new:10,new:11,new:12,fix:2,fix:3,fix:5,fix:8,fix:9,fix:11,fix:12
E7RESULT name=q2_family family=N rows=10 o2_red=2 o2_quiet=8 o2_quiet_rows=new:14,new:15,new:16,new:17,fix:14,fix:15,fix:16,fix:17
E7RESULT name=q2_family family=K rows=41 o2_red=41 o2_quiet=0 o2_quiet_rows=none
E7RESULT name=q2_blind_family rows=0 expected_when_run=40 status=not_run why=seed_needs_the_frozen_executor_s7
E7RESULT name=q2_negative_bases bases=8 expected=8 o2_red_bases=0 red_base_list=none
E7RESULT name=a9_a10_a8 p_rows=26 p_rows_expected=26 n_rows=10 n_rows_expected=10 known_bad_rows=41 known_bad_rows_expected=41 blind_rows=0 q2_table_rows_when_the_blind_family_exists=107
E7RESULT name=q2_all_quiet status=not_run why=o3_frozen_version_not_available_s7 o2_quiet_rows_total=16
```

判决行（比对器自己的那一行；O3 一律 `not_run`）：

```
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=6 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 rows_untouched_but_red=0 negative_bases_with_o2_red=0 known_bad_rows_with_o2_quiet=0 o3_columns=not_run q1_alpha=not_run q1_beta=not_run q2_all_quiet=not_run q3_alpha=not_run
```

`research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out`（第一刀第 6 行：分配记录树按槽号找叶时叶宽算成两倍，实现在第一次 `publish` 里 panic，装置记「实现自崩」、这条流不导出）：

```
E7RESULT name=stream stream=new_pool_file_creation ending=implementation_crashed:call_2_publish:分配记录（盘DeviceIdentity(0)槽SlotNumber(50176)跨2）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过 exported=false not_exported_why=implementation_crashed_in_a_call:call_2_publish:分配记录（盘_DeviceIdentity(0)_槽_SlotNumber(50176)_跨_2）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过 operations=22 writes=12 segments=7 tearable_writes=0 calls=3 full_states=0 quick_tier_states=0 sampled_states=0 model_stopped_at_call=2
E7RESULT name=call stream=new_pool_file_creation call=2 label=publish first_operation=22 return_point=22 status=panic o1_comparison=not_modelled
E7RESULT name=stream stream=fixed_script_through_e_then_unmount ending=implementation_crashed:call_2_publish:分配记录（盘DeviceIdentity(0)槽SlotNumber(50176)跨2）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过 exported=false not_exported_why=implementation_crashed_in_a_call:call_2_publish:分配记录（盘_DeviceIdentity(0)_槽_SlotNumber(50176)_跨_2）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过 operations=22 writes=12 segments=7 tearable_writes=0 calls=3 full_states=0 quick_tier_states=0 sampled_states=0 model_stopped_at_call=2
E7RESULT name=call stream=fixed_script_through_e_then_unmount call=2 label=publish first_operation=22 return_point=22 status=panic o1_comparison=not_modelled
```

`research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out`（41 条已知坏镜像在流一全持久态上：补丁叠回与直接改坏判红的不变量集合相等、目标不变量都红、条目名与原文件逐条相同）：

```
E7RESULT name=s4 check=known_bad_clean_image_digest_equals_build_pool local="9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034" crates="9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034" holds=true
E7RESULT name=s4 check=known_bad_clean_image_digest_equals_the_full_state_of_the_dump local=Some("9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034") crates=Some("9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034") holds=true
E7RESULT name=s4 check=known_bad_test_file_sha256_is_the_copied_one local="e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820" crates="e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820" holds=true
E7RESULT name=s4 check=known_bad_entry_count_here local=41 crates=41 holds=true
E7RESULT name=s4 check=known_bad_entry_count_in_the_original_file local=41 crates=41 holds=true
E7RESULT name=s4 check=known_bad_entry_names_equal_the_original_file_in_order local=["I-7.4","I-4.8","I-1.1","I-1.3","I-1.4","I-1.4","I-1.6","I-1.7","I-1.10","I-2.1","I-2.3","I-2.4","I-2.5","I-3.1","I-5.1","I-5.2","I-7.1","I-7.2","I-7.6","I-7.13","I-7.14","I-7.7","I-7.7","I-7.7","I-7.8","I-9.1","I-9.2","I-9.4","I-9.7","I-9.10","I-9.15","I-9.16","I-3.8","I-8.6","I-1.8","I-1.2","I-1.11","I-4.2","I-9.6","I-9.12","I-9.13"] crates=["I-7.4","I-4.8","I-1.1","I-1.3","I-1.4","I-1.4","I-1.6","I-1.7","I-1.10","I-2.1","I-2.3","I-2.4","I-2.5","I-3.1","I-5.1","I-5.2","I-7.1","I-7.2","I-7.6","I-7.13","I-7.14","I-7.7","I-7.7","I-7.7","I-7.8","I-9.1","I-9.2","I-9.4","I-9.7","I-9.10","I-9.15","I-9.16","I-3.8","I-8.6","I-1.8","I-1.2","I-1.11","I-4.2","I-9.6","I-9.12","I-9.13"] holds=true
E7RESULT name=verdict known_bad_entries_failing_s4=0 e170_bad_rows_failing_s3_or_s4=0
```

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:390-396`（整段抄，未转述）**

```markdown
### 对照与自检

- 阴性（登记 5.6）：基线两趟、三批的每一态与每一次调用上，O1-对拍、O1-崩溃判官、O2 判红都是 0 处（A3，`name=a3_baseline_reds`）；空变异（拿表里第一行、把替换文换成原文本身，走完整条逐行驱动）判未触达、四列不红（`name=row row=0`、`name=row_touch row=0`、`name=row_matrix row=0`）；8 张不打补丁的底态（两条流的 F、Mk、Mk⁺、Mk⁻）O2 不红（`name=q2_negative_bases`）。O3（第二段起）：冻结的执行器在基线两条流批 1、批 2 的 576 态上判红 0 处，s4 产物原行 `E7RESULT name=a3_o3_baseline states=576 strict_in=576 strict_not_in=0 strict_spec_gap=0 strict_error=0 provisional_in=576 provisional_not_in=0 digest_mismatch_states=0 dump_problem_lines=0 base_unrecoverable_streams=none complete=true freeze_lines_equal_the_reference=true`；8 张底态 O3 甲乙都不红（`name=q2_negative_bases … o3_strict_red_bases=0 o3_provisional_red_bases=0`）。s4 整表开跑前，同一棵冻住的树上的空变异一行对 `baseline-run1` 未触达（主 agent 跑的整表脚本第 3 步：`write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false touched=false`）。
- 阳性（装置一侧三条，登记 5.6）：O1-对拍在流二第一次 `publish` 的观测里把那一版文件内容的末字节翻一位，对拍判出一处；O1-崩溃判官在每条流批 1 第一个「实现读回了文件」的态把读回内容末字节翻一位，每条流各一处、合计 2 处（登记字面是「第一个」，装置按每条流各翻一次，对照要的「红」没变）；O2 在流一全持久态上叠已知坏镜像族第一条，1 处。基线一趟与空变异一趟各跑一遍，红都一路带到 `oracle-verdicts.tsv` 与 `call-outcomes.tsv`（比对器的 `name=positive_control` 六行）。O3 的阳性对照（第二段起）：流一全持久态叠 P1 补丁交冻结的执行器，基线与空变异两趟各判红 1 处（s4 产物 `name=positive_control oracle=O3` 两行）。翻过的那次对拍让模型停在那次调用，此后的调用不比、写落在它之后的态 O1-崩溃判官记「不适用」，所以 O1-对拍那一趟的 `o1_crash_judge_red_states` 是 0。
- 停机条款：S1 本地常量与 crates 回比 42 条全过；S2 两条流的八个文件与 `impl-verdicts.tsv`（去掉耗时列）逐字节等于 E170（O3规约执行器第一版） 导出装置在同一棵树上 `dump` 写出的；S3 O2 直接调 `check_pool_image` 与 `judge_crash_image` 内那一遍判红的不变量集合在基线每个取样态上相同（`o2_matches_judgement_false=0`），在 76 行坏状态上与 E170（O3规约执行器第一版） `bad-impl-verdicts.tsv` 池级 checker 那一列逐行相同；S4 已知坏镜像族 41 条、条目名与原文件逐条相同、干净镜像摘要等于 `build_pool` 的、补丁叠回与直接改坏判红集合相等、目标不变量都红；S5 比对器自己现算的两个数（批 1 的态数与序号集合、O1-崩溃判官适用的态数）与装置的计数相等；S6 没触发：E170（O3规约执行器第一版） 导出装置在今天的树上 `dump` 退 0、36 条 `holds=true`、0 条 `holds=false`；S7 在第一段触发（O3 列不跑），第二段起 O3 用登记修订 2 抄下冻结行的那一版，s4 比对器开头与收尾两次冻结核对都是 0 处不同；S8 的三处调用在「装置、研究一侧与变异表」一节给了行号。逐行那一趟的 S5（登记修订 3、4）在 s4 117 行上 0 行不过：判决行 `rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0`，`name=q1_denominator … device_stop=0`。
- 被测的是跑那一刻的树：s4 的空变异、117 行与 Q3-核三趟从同一份冻住的树拍快照（五趟驱动的 `snapshot_digest` 只有一种，`d15864653cd83cedddc4902b6598d5db9cf590a62e6cbb8a54bed6661e4700cb`）；开跑前树核对拿它与第六段读码时的清单逐文件比，只差一个别的 bin（`e168_delete_ring_window_borrow.rs`，不挡）。第一段的基线、对照与试跑用的是 `crates/` 在跑那一刻的状态；E170（O3规约执行器第一版） 导出装置一侧用的是 `git show :crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` 取出的暂存版本（sha256 `8c526293…`，登记第三节钉的那一份）在草稿目录里编出来的，它的 `dump` 与 `bad-states` 输出与工作区里 E170（O3规约执行器第一版） 第 1 次重跑改过的版本逐字节相同（`cmp` 无输出），所以复跑驱动直接用工作区里的版本。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:397-402`（整段抄，未转述）**

```markdown
### 单测与变异

- Python 一侧单测 110 条（研究一侧目录里 `grep -c '    def test_' test_*.py` 四个文件 32 + 29 + 24 + 25），`python3 -m unittest test_collusion_matrix test_collusion_pipeline test_o3_columns test_o3_comparison` 连跑 5 轮每轮 `Ran 110 tests` / `OK`（手造数据、确定性，5 轮只说明没有隐藏状态）：矩阵（触达三样、O3 两列的读法、全放行谓词、关一条、门槛、轨迹）、切段与取样公式现算、导出目录读回、坏实现表（锚点、补位、空变异）、挑法脚本的行数、冻结核对、坏镜像的 O3 判法、逐行驱动的编译记录与重编核对（真 `rsync -a` 与 `copy2`、假 cargo）、F4 扩表（挑法脚本两次的差集与编号、比对器的逐行核对、两个输出根的合并与重号拒绝、分母按表分开、补位跳过两张表的名字）、编不过之后的补位（手造组 r0–r5、表里有 r0 与 r2、装置行插在中间：编不过依次换到 r1、r3、r4，第四次补不上记编不过；换上的行锚点不过与编不过合算三次；空变异不补位；组里没有装置行；驱动在同一个行目录里重编换上的行、`compile-failed-1.log` 与 `failed_attempt_1` 同号、复跑开关不补位；比对器只拿补位行换掉编不过的同号行、别的情形拒、补位行的结果行写换上的名字）。每个会被变异弄坏的谓词都配了钉绝对值的断言（例：扩表在手造表上必须是 `(118, a4)`、`(119, a12)`、`(120, b2)` 三行；补位的整行 `name=substitution row=3 … substituted_by=c_换 failed_attempts=1 …` 写死）。
- Python 一侧变异 67 条，表 `research/mutations/e171_three_oracle_collusion.tsv`，跑法 `mutation_driver.py`（登记第九节写「经 `research/scripts/mutate.sh` 跑」，那条脚本只起 cargo test，起不了 Python 的单测，登记修订 1 第 1 行记了这一处）：`e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out` 第 137 行 `E7RESULT name=verdict mutations=67 caught=67 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`，抓到 / 无效 / 没红 = 67 / 0 / 0（结局符号逐个数：✅ 67、⏭ 0、❌ 0、⏱ 0、💥 0、⚠️ 0、🧱 0）。X60–X74 是编不过补位那一批（编不过不补位、不计次数、组里算进装置行、换上的行锚点不过时结局不记编不过、补位那一趟领了不是编不过的行、不记试过的名字、编不过之后不换行直接跑装置、比对器不核原行是编不过、不核补位自的名字、结果行写原名、合计把补不上的算换上、行分析不套补位、日志与试过的名字不同号、复跑开关不起作用、补位那一趟同一行在两个根里不拒）。同名的 `…-r6.out` 是同一批的前 13 条加进来时的全表跑（65 条，65 / 0 / 0），之后补了 X73、X74，留档。按「没红的三类」想过而没加的一条：编不过之后的补位从表里原名往下找、不从编不过的那一行往下找——补位跳过表里与已用过的名字，原名与编不过那一行之间的每个名字都在这两样里，两种找法在所有输入上同值，是等价变异，没加进表，等价性写成一条留档测试（`test_walking_from_the_name_in_the_table_or_from_the_last_failed_name_gives_the_same_substitute`，手造组上逐个起点核）；加了这条测试之后变异表重跑一遍，输出与 `-r7.out` 逐字节相同。X52–X59 是 F4 扩表那一批（扩表不去掉重叠行、从 117 编号、把装置自己的变异行算进挑法、两个输出根同号不拒、补位不跳过别的表、扩表核对只比行数、分母按表对调、只读第一个输出根），每条都由 `F4ExpansionTests` 里对应的那条单测抓到。同名的 `…-r4.out` 是同一张表的一次中途被停的跑（外面套的 `timeout 230` 在第 8 条变异时停了它，61 行、没有判决行），不算数、留档。更早的 `…-s1-comparator-mutations.out`、`…-s2-comparator-mutations.out`、`…-s3-comparator-mutations.out` 与 `…-s3-comparator-mutations-2026-10-03.out`、`-r2.out`、`-r3.out` 是表在 9、26、37、40、44、44 条时的全表跑，各自全抓。
- 装置内联单测 31 条（`grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`）：编过（`research/scripts/compile-then-swap.py` 的 `cargo build` 与 `cargo test --no-run`），没跑（checker 档，归提交）。装置一侧变异 15 行追加在 `crates/mutations.tsv` 末尾（名字以「E171 装置」起头），三个数待提交时门禁 checker-tier-crates-mutation-replay。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:403-415`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 批 1 的态数（装置） | 取样三批的公式作用在层 0 枚举表给的每段序号区间上，再加撕裂扫描；枚举点，不是全量 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1346` `sample_ordinals` | `layer0_enumeration_tables` 的段区间与哪几次写取三态（来自 `crates/singlefs-checker-tier/src/crash.rs`） |
| 2 | 批 1 的态数（比对器现算） | 从 `ops.tsv` 按切段规则切段、`torn.tsv` 的写序号当三态写，每段状态数 = 各写取几态的乘积减 1，再按同一套取样公式现算序号集合 | `research/e7-index-bench/src/bin/e171_three_oracle_collusion/baseline_checks.py:175` `recompute_first_batch` | 取样公式（E170（O3规约执行器第一版） 登记 5.4 的文字）；`torn.tsv` 与 `ops.tsv`（装置从 crates 的枚举表与写表写出）：两条路径共用「哪几次写取三态」 |
| 3 | O1-崩溃判官适用的态数（装置） | 一态里持久或撕裂的写属于的最后一次调用不早于模型停下的那次调用，这一态不适用 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1920` `crash_judge_is_applicable` | 调用表的返回点与写表的流序下标 |
| 4 | O1-崩溃判官适用的态数（比对器现算） | 从 `calls.tsv`、`ops.tsv`、`states.tsv` 的落地向量数写落在 `unmount` 那次调用里的态 | `research/e7-index-bench/src/bin/e171_three_oracle_collusion/baseline_checks.py:184` `recompute_crash_judge_applicable` | `calls.tsv` 的返回点（装置写出）；模型只停在 `unmount` 这一条读法（登记第一节读法 6） |
| 5 | O2 在坏状态行上的判定（装置） | 从中性格式按落地向量重建 `MemoryPool`、叠 E170（O3规约执行器第一版） 写的补丁、调 `check_pool_image` | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:4646` `e170_bad_rows_on_both_streams` | 池级 checker 本身、E170（O3规约执行器第一版） 写的 `bad-patches.bin` 与落地向量 |
| 6 | O2 在坏状态行上的判定（E170（O3规约执行器第一版） 导出装置） | `CrashImage` 上把补丁接在写表末尾当持久的系统配置槽写，调 `judge_crash_image` 取池级 checker 一项 | `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:2994` `observe_bad_row` | 池级 checker 本身；同一批补丁字节与落地向量 |

路径 1 与 2、3 与 4 在产物里逐格比（`name=s5_batch_1_states`、`name=s5_o1_applicable` 八行 `equal=true`），路径 5 与 6 逐行比（`name=e170_bad_row` 76 行 `o2_equals_e170=true`）；这几处比对写在产物里，没有 `assert` 把两条路径直接连起来，还欠这一条。共用项里最要紧的是池级 checker 本身（路径 5、6 同一份 `check_pool_image`）：它错了两条一起错，所以这一格不能当 O2 判得对的证据；路径 1、2 共用「哪几次写取三态」，它错了两份态数一起错。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:416-535`（整段抄，未转述）**

````markdown
### 复跑

装置与 E170（O3规约执行器第一版） 导出装置都是 checker 档的入库装置，经内存包装起；研究一侧是 Python。`research/scripts/replay.sh` 里 E171 的十一行登记，各带自己的临时目录，产物只留 E7RESULT 行、exact 比：

| 驱动函数 | 产物 | 跑什么 |
|---|---|---|
| `driver_e171_s1_baseline` | `…-s1-baseline-run1.out` | 装置 `baseline` 一趟 |
| `driver_e171_s1_known_bad_images` | `…-s1-known-bad-images.out` | 基线、E170 导出装置 `bad-states`（占位种子）、装置 `known-bad-images` |
| `driver_e171_s2_bad_states_export`、`driver_e171_s2_known_bad_images`、`driver_e171_s2_images_o3`、`driver_e171_s2_o3_control_baseline` | `…-s2-bad-states-export.out`、`…-s2-known-bad-images.out`、`…-s2-images-o3.out`、`…-s2-o3-control-baseline.out` | 坏镜像三族在冻结种子下的造法、O2 与冻结执行器的判定、O3 阳性对照 |
| `driver_e171_s2_comparator` | `…-s2-comparator.out` | 第二段比对器那一串（九行试跑）；比对器在第三至六段改过判决行与 config 行，这份今天复跑对不上，没改指 |
| `driver_e171_s3_comparator_mutations` | `…-s3-comparator-mutations-2026-10-03-r7.out` | 研究一侧 67 条变异，`mutation_driver.py`，约一分钟 |
| `driver_e171_s4_comparator` | `…-s4-comparator.out` | 第二段那一串前置、修过的逐行驱动跑全部 117 行（两条线道、每条 2 线程，一份冻住的树）、Q3-核三趟、比对器；推的约两小时，没真跑过这个驱动函数（重型，归提交时的全部实验复跑）。117 行的读数读今天的 `crates/`，实现、O2、O1、装置或那 117 行的行文变了产物就变。整表那一步带 `--keep-compile-failures-as-they-are`：这份产物是编不过不补位的那一版驱动跑的 |
| `driver_e171_s4_f4_comparator` | `…-s4-f4-comparator.out` | 先整条跑 `driver_e171_s4_comparator`，再在它那一份冻住的树与工作根上跑扩表 164 行（同样带 `--keep-compile-failures-as-they-are`），比对器并两半、带扩表、段名 s4-f4；推的约四小时，没真跑过（`bash -n` 过；比对器那一半用同一组参数在已有导出上重跑过、与产物逐字节相同） |
| `driver_e171_s4_f4_substitutes_comparator` | `…-s4-f4-substitutes-comparator.out` | 先整条跑 `driver_e171_s4_f4_comparator`（它的草稿目录留着 s4、F4 的导出与冻住的树），再在同一份冻住的树与工作根上只跑两份输出根里结局是编不过的行（`--rerun-compile-failed-from`，补位跳过两张表的全部名字，两条线道、每条 2 线程，不带 `--keep-compile-failures-as-they-are`），比对器并 117 + 164 + 补位、带 `--substitution-rows-root`、段名 s4-f4-substitutes；推的约四个半小时（s4 与 F4 那一串约四小时，加补位 1 277.9 秒），没真跑过（`bash -n` 过；各步参数照补位那一趟实际跑的命令逐条对过，没有重跑驱动或比对器）。读今天的 `crates/`，实现、O2、O1、装置或那六行的行文变了产物就变 |

拿 s4 的输入（已有的导出目录与驱动输出）重跑今天的比对器，E7RESULT 行与 `…-s4-comparator.out` 逐字节相同（`cmp` 无输出）：比对器这一半复得出；装置与驱动那一半要整表重跑才核得到。第一段几样的单条命令：

```
bash research/scripts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- baseline --output-directory <新目录>
bash research/scripts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- bad-states --dump-directory <基线导出目录的一份拷贝> --seed-hex 0000000000000000
bash research/scripts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- known-bad-images --dump-directory <上一步那份拷贝> --output-directory <新目录> --e170-bad-rows
cd research/e7-index-bench/src/bin/e171_three_oracle_collusion && python3 mutation_driver.py --table ../../../../mutations/e171_three_oracle_collusion.tsv --scratch <草稿目录> --output <新文件>
```

第一段比对器那份（`…-s1-comparator.out`，第一段的 `driver_e171_s1_comparator` 已由 s2 那一份取代、不再登记）要先跑齐基线两趟、E170（O3规约执行器第一版） 导出装置的 `dump`、关一条两趟、阳性对照六趟（基线三趟，空变异三趟经 `row_driver.py --empty-mutation --positive-control <名>`）、逐行驱动三行（`row_driver.py --empty-mutation --rows 1,6`），再 `python3 comparator.py …`，大约十几趟装置（每趟约一分钟），驱动函数把这一串写死；比对器的 config 行只写目录的名字，所以复跑目录的名字要与驱动里的相同。比对器的 A4–A7 读今天的 `crates/mutations.tsv`：表里别处加了核心实现的行或改了那 117 行，这几行会变，不是装置坏了。占位种子 `0000000000000000` 造出的盲测行 E171（三oracle合谋测试） 不用；E170（O3规约执行器第一版） 在第 1 次重跑里加了第二套坏状态表（`bad-states-second-run`），`bad-states` 仍是第一套，两个版本的导出装置在第一套上的输出逐字节相同（「对照与自检」一节最后一条）。

复跑驱动的自检：基线、已知坏镜像、变异驱动三个驱动函数照 `replay.sh` 的口径在草稿目录里跑过一遍，输出与入库产物逐字节相同（`cmp` 无输出）；比对器那一个驱动函数没有整条真跑（十几趟装置，前台跑不完），它的胶水（各步命令行、目录的名字、传给比对器的参数）用「装置、E170（O3规约执行器第一版） 导出装置与逐行驱动换成从已有导出目录拷」的桩跑过，比对器照真的跑，输出与入库产物逐字节相同；各步命令本身在做产物时逐条真跑过。整条真跑由提交时的复跑做。

产物（都在 `research/results/` 下，名字前缀 `e171-three-oracle-collusion-2026-10-03-s1-` 之后接）：

| 文件名（前缀之后） | 内容 |
|---|---|
| `baseline-run1.out`、`baseline-run2.out` | 基线两趟的 E7RESULT 行（两份逐字节相同） |
| `baseline-timing.out` | 两趟的 E7TIMING 行（每批每段耗时、每条流驱动、判、写各花多久） |
| `known-bad-images.out` | 装置 `known-bad-images` 模式：41 条已知坏镜像与 E170（O3规约执行器第一版） 的 76 行坏状态上的 O2 |
| `comparator.out` | 比对器：登记锚点、基线与对照、关一条、三行试跑、Q1 分母、Q2 的 O2 一列 |
| `rows-timing.out` | 逐行驱动与比对器的 E7TIMING 行（Q4 单行数） |
| `comparator-mutations.out` | Python 一侧 9 条变异 |
| `e170-pinned-dump.out` | E170 暂存版本导出装置的 `dump`（S2 的对照） |
| `e170-bad-states-placeholder-seed.out` | 同一版本的 `bad-states`（P、N 行的出处，盲测行不用） |
| `positive-control-o1-comparison.out`、`positive-control-o1-crash-judge.out`、`positive-control-o2.out` | 三条阳性对照在基线那一趟的装置输出 |
| `disable-o1.out`、`disable-o2.out` | 关一条的开关自检（基线上各一趟）的装置输出 |
| `row-0-device.out`、`row-1-device.out`、`row-6-device.out` | 三行试跑的装置输出（row-0 是空变异） |

全名（`ls research/results | grep '^e171-three-oracle-collusion-2026-10-03-s1-'`，17 份）：

```
research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run2.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator-mutations.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-disable-o1.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-disable-o2.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-e170-bad-states-placeholder-seed.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-e170-pinned-dump.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-comparison.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-crash-judge.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-positive-control-o2.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-row-0-device.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-row-1-device.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-rows-timing.out
```

第二段起的产物（`ls research/results | grep -E '^e171-three-oracle-collusion-2026-10-03-s[234]-'`，29 份，名字前缀 `e171-three-oracle-collusion-2026-10-03-` 之后接）：

| 文件名（前缀之后） | 内容 | 现在算不算数 |
|---|---|---|
| `s2-bad-states-export.out`、`s2-known-bad-images.out`、`s2-images-o3.out`、`s2-o3-control-baseline.out`、`s2-o3-baseline.out` | 坏镜像三族的造法与 O2、O3 判定、O3 阳性对照、冻结执行器在基线上（A3 的 O3 一份） | 算：s4 比对器读的就是这几份 |
| `s2-comparator.out`、`s2-rows-timing.out` | 第二段比对器（Q2 全表、九行试跑）与它的耗时 | Q2 的数同 s4；九行试跑由 s4 整表取代 |
| `s2-comparator-mutations.out` | 变异表 26 条时的全表跑 | 历史 |
| `s3-baseline-after-the-fix.out`、`s3-row-0-device.out` | 修订 3 修过取号、暖机记红之后的装置基线与空变异一行（与修之前逐字节相同） | 算：修订 3 的实核 |
| `s3-k4c-attack-row-device.out`、`s3-k4c-check-driver.out` | 修订 3：取号恒拒那一行经逐行驱动跑一行（O1 红） | 算：修订 3 的实核 |
| `s3-comparator.out`、`s3-rows-timing.out` | s3 整表 117 行的比对器与耗时 | 作废留档（「s3 整表作废」一节） |
| `s3-clean-check-comparator.out` | 第五段在干净工作根上复核第 2、27、42、65 行 | 作废的佐证，留档 |
| `s3-comparator-mutations.out`、`s3-comparator-mutations-2026-10-03.out`、`-r2.out`、`-r3.out` | 变异表 37、40、44、44 条时的全表跑 | 历史 |
| `s3-comparator-mutations-2026-10-03-r4.out` | 52 条那一次中途被停（61 行、没有判决行） | 不算数，留档 |
| `s3-comparator-mutations-2026-10-03-r5.out` | 变异表 52 条的全表跑，52 / 0 / 0 | 历史 |
| `s3-comparator-mutations-2026-10-03-r6.out` | 变异表 65 条的全表跑（X60–X72 加进来之后），65 / 0 / 0 | 历史 |
| `s3-comparator-mutations-2026-10-03-r7.out` | 变异表 67 条的全表跑，67 / 0 / 0 | 算：复跑登记指它 |
| `s4-comparator.out`、`s4-rows-timing.out` | s4 整表 117 行、Q3-核三趟、比对器与耗时 | 算：117 行那一半的读数（F4 产物里第 1–117 行的结果行与它逐字节相同） |
| `s4-f4-comparator.out`、`s4-f4-rows-timing.out` | F4 扩表 164 行与 s4 的 117 行并表的比对器与耗时 | 算：补位之前的读数（复跑登记里带 `--keep-compile-failures-as-they-are` 的那一份） |
| `s4-f4-substitutes-comparator.out`、`s4-f4-substitutes-rows-timing.out` | 117 + 164 行并上编不过 6 行的补位之后的比对器与耗时（补位那一趟 1 277.9 秒） | 算：补位之后的 Q1、Q3、Q4 读数（Q1、Q3 名单与补位之前逐字同），三轮判决引的产物行号取自这一份 |

全名：

```
research/results/e171-three-oracle-collusion-2026-10-03-s2-bad-states-export.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-comparator-mutations.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-images-o3.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-known-bad-images.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-o3-baseline.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-o3-control-baseline.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-baseline-after-the-fix.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-clean-check-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r2.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r3.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r4.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r5.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r6.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-k4c-attack-row-device.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-k4c-check-driver.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-row-0-device.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-rows-timing.out
```

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:536-557`（整段抄，未转述）**

```markdown
### 它答不了的

- Q1 的「有」是照字面判的（≥ 1 行）；12 行（丙 13 行）里哪几行算真坏，三轮结论判决给了三分（「三轮判决之后的三分」一节）：真盲区候选 7 行（第 15、23、42、242、269、270、271 行），状态不合法所依的条款句写在「三条全放行的 7 行」「F4 新出的全放行行」两节；另几类 5 行里第 92 行那一句在字节表与里程碑验收、不在决策正文。三分只是「这一批取到的态」上的读法，各行会分开的历史多数是推的、没跑。分母 D = 37 补位之前与之后相同（补位的六行都未触达），Q1 的「有」没翻。
- 281 行里 238 行未触达（补位之后；补位之前 232 行，另 6 行编不过）：这两条流的历史只走到实现的一小部分（取号、暖机、发布、可写挂载、管理员回退、抬 F、正常卸载各一两次，没有分裂、没有删除、没有空间紧的池），未触达只说那一处改动的写流在这两条流上没变，不说它被哪条 oracle 罩着；其中 7 行改动走到了、只是没写到盘上（第 49、110、155、161、162、174、252 行，「没触达却红的两行与未触达行里交回值不同的七行」一节）。三轮之后没有一行全放行行还记「对这一批等价」。
- 触达的定义（登记读法 4）不比成功的调用交回的值，这一条没改（登记不改），分母仍 37：第 155、161 行抬 F 的上限交回 13（模型 11）、写流与结局都同，记未触达，而 O1-对拍判红；攻方在 s4 树的副本上量了 232 行未触达行的成功调用交回值，7 行不同（第 49、110、155、161、162、174、252 行），汇总行 `name=return_summary hidden_difference=7 identical=225 no_output=6`（副本探针，s4 树，入库装置没有这一列；整行与来历在「没触达却红的两行与未触达行里交回值不同的七行」一节）；mkfs 的 6 行（第 149–154 行）没量。
- 编不过的 6 行的补位那一趟已跑（「编不过的 6 行补位（已跑）」一节）；s4、F4 两份产物是补位之前的驱动跑的，复跑它们的登记行带 `--keep-compile-failures-as-they-are`、照旧不补位。
- 顺序类的条款（第 242、270 行）是在层 0 的切段模型上数的：「条款不许的态」靠切段规则（FUA 写只放行它自己那块盘、屏障放行它那块盘）分得出，真机上一次 FUA 写与随后的写怎么排，不在这一批里。
- O3 在 37 行上一行都没红：它的 A(c) 由实现自己写出的根构成，抓的是「恢复落到的那一版不在 [ack, begun] 里」这一类，实现一致地写错的字段（容器号、写入时间、F 的取值、记账行）它照样判「在」。这是读执行器代码读出来的，没有另造一行只让 O3 红的坏实现去核；坏镜像那一族里 O3 红得多（Q2）。
- 三行试跑（第一段）不是表的抽样，s4 整表取代了它们当 Q1 的读数；第 1 行未触达只说那一处改动在这两条流的历史上没走到。
- 基线上 O1-崩溃判官有一部分态不适用：流二最后一次调用 `unmount` 模型没有答，写落在这次调用里的态不判；基线批 1 流二 397 态里 48 个、批 2 的 119 态里 14 个不适用（`o1_crash_judge_not_applicable`），这些态上 O1 一列是空的。流一没有不适用的态。
- 阳性对照 O1-崩溃判官每条流各翻一次（合计 2 处），登记字面是「批 1 里第一个」；O2 的对照只在流一的全持久态上叠第一条已知坏镜像，没有别的取样点。对照证明管子把红带到矩阵，不证明每条 oracle 对每种改坏都分得出。
- Q2 的 O2 一列取自两条路径（路径 5、6）；P 行的补丁字节与落地向量是 E170（O3规约执行器第一版） 导出装置写的，期望是 E170（O3规约执行器第一版） 设计员推的（登记 A2、F3）。Q2 全放行的 4 张全是 P8，那一判归 F3。41 条已知坏镜像是 O2 自己的回归语料，O2 41 条全红是构造的结果，不是 O2 独立性的证据。
- 逐行驱动与装置读跑那一刻的 `crates/`：s4 五趟驱动是同一份冻住的树，但 `crates/` 里别处在变，整表跑出来的数只说那一份树。
- 坏镜像上实现的恢复读回什么（登记第一节读法 11，附带）、旧种子盲测族的回归都没跑。Q1-丁（三条全放行而记录核对器或层 0 手写版本表的 oracle 红的行）跑了：F4 的 12 行全放行（甲、乙）里一行都没有（`name=q1_delta … =none`）；比对器只按甲乙算这一格，丙多出的第 233 行上层 0 那份 oracle 与记录核对器红（`name=q1_quiet_row reading=q1_gamma row=233 … layer0_or_record_checker_red=true`）；层 0 那份 oracle 与记录核对器另在只靠 O1 的第 108、115 行上红。
- 装置内联单测没跑（checker 档），装置一侧变异 15 行的三个数没有；Python 一侧的变异只证明比对器与驱动的谓词会红，不证明它们读的装置输出是对的（装置输出由两趟基线、S2、S5 与对照另行钉）。
- E171 的 O1 一列是模型对拍加模型的崩溃判官，不接 RefFS（登记把崩溃判官算进 O1，另写本实验不用 RefFS），与 D13（验证路线） 已定项 6 表 O1 行把崩溃之后允许看到的集合归 RefFS 不同；两者的下界也不同（盘上已持久的最新根，对最后一次返回成功的发布）。「已确认而丢了」那一类在 E171 的四列里只有 O3 判，没有在 37 行上跑过 RefFS（读码读出来的）。RefFS 的数只在三轮的第二轮判决里（`research/prompts/e171-conclusion-r2-main-verification.md` 第二节 R6 一格，读数在 `research/prompts/e171-conclusion-r2-opus-model/out/judge-summary.out`，攻方副本探针，s4 树 core，RefFS 直调 crate、崩溃落点手给、没经翻译层）：暖机那一形 O3 判「不在」27 态，RefFS 判绿的也是这 27 态（27 / 27）；带内容的同形写流 O3 判「不在」的 27 态 RefFS 全红，RefFS 红的共 36 态，另 9 态 RefFS 红而 O3 判在（两边归属崩溃时刻的规矩不同，推的）。这几个数不是 E171 装置读数。
- Q2 里 O2 放行而 O3 判不在的 29 张（P 族 12、盲测 B 族 17，「Q2 里 O2 放行而 O3 判不在的坏镜像」一节）全是结构自洽的伪造根、伪造续接镜像，不是坏实现；补丁与期望同出 E170（O3规约执行器第一版） 的改法代码（E170 页「它答不了的」），源头独立在 Q2 只量过 O2、O3 一对，不当独立读数，O1 不在 Q2 里；流一 P2、P5 两种补丁的 O2 一列随底态翻面。这 29 张的复跑读今天的 `crates/`，不还原 s4 的读数：基线流与 s4 树不同，数不保证仍是 12 与 17。D13 依据段只指到本页的这个数。
- 这一页的行级读数（全放行、只靠一条、未触达的名单与每行的态）只属于快照摘要那一棵冻住的树：它与今天的 `crates/` 不是同一份，页里的名单不能直接当今天实现的名单；`replay.sh` 的 E171 复跑行读今天的 `crates/`，复跑一次出的是今天那棵树的读数。入库装置上除 Q2 的 29 张之外，下面几个数都不能原样重做（装置没有挂载模式、没有 RefFS 列、panic 的流不导出）。
- 第 125 行是自崩的 6 行之一：登记读法 5 把 panic 记自崩、不进分母，D = 37。它在 panic 之前有 13 次调用可判：副本探针在 s4 树上驱动、打第 125 行变异，第 4–13 次调用的全持久态 O2 判 I-3.11 红，第 14 次（`unmount`）panic（`research/prompts/e171-conclusion-r2-opus-model/out/drive-s4-m125.tsv`），装置对 panic 整条流不导出。把 panic 与报错同样截断照判，计入分母 D = 38（改口径的预演，不是装置读数，37 与 38 都写）；若触达也把成功调用的交回值算进去，含第 155、161 行，D = 40（37 加第 125、155、161 行）；7 行交回值不同的行都算进来、含第 125 行，D = 45（推的，算术）。
- 谓词 W（正确实现的可写挂载会把这一态读成「见证过的更新一版读不出」）在第 242、269 行取到的崩溃态上为真的态各 15、63 个，基线 0 / 576；s4 树 core 上两次可写挂载的见证类拒 61 个（第 242 行 9、第 269 行 52）。这些数是攻方副本脚本（W）与副本探针（挂载）在 s4 树上量的，入库装置没有 W、也没有挂载模式。今天的 core 读 s4 写出的镜像大面积 `refused:Undecidable`（基线流一 44 / 60、流二 500 / 516）：格式变了（journal 头 311 → 487、新根段 188 → 364、点名项 67 → 64），旧记录在新解析器下读不出，不是回归；所以 W 的 15 / 63 与 61 个见证拒只对 s4 core 成立，在今天的 core 上没有对照意义。
- 用户数据落点的取值算不算 layout 登记的字节，这一页不判；三轮判决交用户（`research/prompts/e171-conclusion-r3-main-verification.md` 第五节第 1 行，零轮），用户 2026-10-03 定「算」：D3（空间分配） 已定项 8 射程改成新池新建文件上的落点取值（t1@50180）算格式级，检查欠在 C673（三 oracle 合谋测试的补 oracle 改法没做） ⑥。它不改这一页的三分：第 15 行的依据是 ⑤ 的 t2–t12 落点；第 34 行的落点改动在流二抬 F 之后那次发布上，不是新池新建文件，仍归分配政策。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:558-576`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 6 | 支撑 | 2026-10-03 改了：E171 触发了定案里「若三者全部放行，说明存在共同盲区，回头补 oracle」那一支（Q1 在坏实现表上读到三者全部放行的行），也支撑「独立性要被验证，不许自称」一句；表第 4 列「独立性来自哪里」E171 只在 Q2 量过 O2、O3 一对，那一对的补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，不当独立读数，O1 不在 Q2 里，所以不支撑也不推翻「三个互相独立」；该分项依据段与本行同一次改动里引回本页（`research/prompts/e171-conclusion-r3-main-verification.md` 第二节 K1 第 1 条） |
| D13（验证路线） 已定项 1 | 不影响 | 2026-10-03 不受影响：只拿 M2 重放结果的根「其余字段等于所选根」当第 233 行（丙）实现恢复结果合不合法的判据，M4「`Recover` 返回的根恰是 j\*」当第 27、112、277 行（三条不判实现恢复里靠记录重建的那一段）的判据；实验不量这一项 |
| D13（验证路线） 已定项 7 | 备料 | 2026-10-03 不受影响：记录核对器与层 0 手写版本表的 oracle 不是三条之一，比对器给它们留了附带列（Q1-丁）；F4 的 12 行全放行（甲乙）里它们一行都没红，在丙多出的第 233 行与只靠 O1 的第 108、115 行上红；已定项 7 的定案句没被动 |
| D13（验证路线） 已定项 15 | 备料 | 2026-10-03 不受影响：117 行整表两条线道 5 569.6 秒、扩表 164 行另一趟 7 607.9 秒，Q4 照登记判「只能进全量」；已定项 15 取的「debug 下单条跑到 60 秒及以上算重」那一档与「checker 档默认只在提交时跑」的定案句没被动，合谋测试进哪一档由主 agent 照 Q4 定 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-03 不受影响：只拿它定案里「新池新建文件写」那一句的容器号 1 与射程里「每个容器号都是某条记录首次插入时的 inode 号」当第 23 行状态合不合法的判据；字段表只定了写入时间的位置（偏移 64）、不定取值，决策正文欠一句，第 92 行归另一类（条款在字节表与里程碑验收）；实验不量这一项 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-03 不受影响：只拿它字段表「正常卸载」一行与射程「卸载之后 F = 现行那一版的 txg」当第 42 行状态合不合法的判据；实验不量这一项 |
| D16（发布语义） 已定项 7 | 不影响 | 2026-10-03 不受影响：只拿定案的持久顺序（根槽 FUA → 系统配置槽 → 屏障、系统配置轮换持久之后才返回、抬 F 那一串先写系统配置）当第 242、270、265 行状态合不合法的判据，射程「同一实例里退一代，靠 journal 重放追得上」当第 277 行会分开的历史；实验不量这一项 |
| D16（发布语义） 已定项 12 | 不影响 | 2026-10-03 不受影响：只拿「根 + journal 任意前缀」当第 277 行新旧两版都合法的依据；实验不量这一项 |
| D8（核心索引结构） 已定项 8 | 不影响 | 2026-10-03 不受影响：只拿定案「头 ID 8 字节的取值」里 `deleted_inodes` 写 inode 树的树 ID 那一句当第 271 行的判据；实验不量这一项 |
| D23（journal 的角色与格式） 已定项 3 | 不影响 | 2026-10-03 不受影响：只拿「恢复不许先信 tail」说明第 269 行 tail 写错而恢复逐态相同；实验不量这一项 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-03 不受影响：只拿取号那一写写见证值 c_见证、mkfs 之后第一次取号写 0 那一句当第 269 行的判据，同一条见证判据当第 242 行「态不合法」的第二块依据，注 1「前缀规则不跨实例边界」当第 112 行的判据；实验不量这一项 |
| D23（journal 的角色与格式） 已定项 15 | 不影响 | 2026-10-03 不受影响：只拿「由记录重建那次发布的根」当第 27、277 行的判据；实验不量这一项 |
| D3（空间分配） 已定项 8 | 不影响 | 2026-10-03 不受影响：只拿射程「其余的写上违反它的实现写出的镜像合法」当第 34 行归分配政策的判据（第 34 行的落点改动在流二抬 F 之后那次发布上，不在新池新建文件里）；射程与 `layout/01-first-txn.md` 第 67、309 行的不一致由三轮判决交用户，用户同日定新池新建文件的落点取值算格式级，那一格不碰这一页的读数；实验不量这一项 |
| D3（空间分配） 已定项 10 | 不影响 | 2026-10-03 不受影响：只拿 ⑤ 的 t2–t12 落点当第 15 行的判据；实验不量这一项 |

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:6-27`（整段抄，未转述）**

```markdown

**D8（核心索引结构）的分项只有一套编号**，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **记账怎么表达成幂等完整值** | 记账条目一律是某个统计量在某一代的完整值、整格覆盖，不用增量 Δ，接受 key 数量上升 **状态：已定。** |
| 2 | **节点大小取多少** | 16 KiB：mkfs 写进系统配置，实现按常量走，不按设备特性算 **状态：已定。** |
| 3 | **key 布局** | key = `(locality_id, inode, offset)`，`locality_id` 创建时从父目录继承、改名时故意不更新 **状态：已定。** |
| 4 | **节点大小的上限由什么定** | 上限由恢复时间定，不由可恢复性定；取值在「重扫代价 ≤ 目标线」下取查找代价最低的那档，靠实验 **状态：已定。** |
| 5 | **whiteout 删除** | 接受墓碑；可能修改无界数量 extent 的操作先写意图再分批推进；第一版 unlink 的意图住 `deleted_inodes`（已定项 16） **状态：已定。** |
| 6 | **inode 记录的盘上字段表** | inode 树的叶是码 3 打包记录单元（类型 2，一叶 233 条定长 140 字节记录），内部节点是码 2、条目 120 字节、不留缓冲；已发布的 (inode 号, 对象出生代) 不复用 **状态：已定。** |
| 7 | **write buffer 条目自带的 `seq`** | 宽 4 字节、住 value 不住 key，射程覆盖记账 / 反向索引 / LRU 三棵树，`T_dirty` 的上界因此是挂载时判定 **状态：已定。** |
| 8 | **树表条目：树 ID 的分配与编码、树的种类、根指针以外还有什么** | 条目合计 200 字节、树 ID 8 打头；快照进树表；树 ID 水位取根环全部根记录该字段的 max；未知 flags 位拒收该条目 **状态：已定。** |
| 9 | **树的种类码的登记表** | 码 0 无、1..8 是八棵既有的树、9..12 是 deleted_inodes、墓碑树、谱系树、logged_ops；新树只追加不重用码 **状态：已定。** |
| 10 | **第一版记账走不走 write buffer 前端；`seq` 的取值规则** | 第一版记账直落记账树叶、不走前端；`seq` = 同一个 key 在一次发布里被写进树的第几次，从 1 起，第一版恒 1 **状态：已定。** |
| 11 | **码 2 索引节点的载荷布局、种类到条目宽的表、key 比较序** | 码 2 头自描述、基础 86 字节，条目区从含预留位的头末尾起定宽连续排列，key 是条目的前 key 宽 个字节 **状态：已定。** |
| 12 | **write buffer 前端与节点消息缓冲区的衔接** | 一棵树只准一级：走前端的三棵树 ε 恒 0、写一律进前端、发布前排空到叶；第一版两级都不开 **状态：已定。** |
| 13 | **索引树更新做不做状态折叠** | 同一次发布里同一个 key 改了多次时落叶只做一次、直接写最终态 E，中间态不写进任何节点，E 按到达序认 **状态：已定。** |
| 14 | **索引的总体结构** | 一套 btree 实现 + 多个独立 keyspace + 两个前端（write buffer、key cache）；派生态的分配记录树与 extent 树可以另起按 key 空间寻址的结构 **状态：已定。** |
| 15 | **write buffer 条目的三条硬要求** | 节点缓存必须抗扫描；条目必须是幂等完整值、不许是增量 Δ；条目必须自带序号 `seq`，这是格式要求 **状态：已定。** |
| 16 | **第一版 unlink 怎么删** | 第一版 unlink：只立 `deleted_inodes`，从尾部按 144 单元分批，挂载里续做到空 **状态：已定。** |

```

**出处 `.claude/kb/decisions/03-空间分配.md:6-28`（整段抄，未转述）**

```markdown

D3（空间分配）的分项只有一套编号，与「未定项」一节连号，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **分配记录的粒度** | 逐落点答得出「这个落点属不属于现行版本」，不是聚合量 **状态：已定。** |
| 2 | **墓碑单元与「释放空间不需要申请空间」怎么共存** | 墓碑按活对象写入时预付；删除各批由准入式子里的常驻环窗口兜，删除类发布不判式子 **状态：已定。** |
| 3 | **分配记录的载体形态** | 住一棵独立 keyspace 的 btree，key 是落点（设备身份 + 设备内偏移），value 是分配代 **状态：已定。** |
| 4 | **准入不等式的权威形式** | 权威正文在 D28（挂载期承诺量） 已定项 1，这一格只留结论句与指针 **状态：已定。** |
| 5 | **提交内生块的落点** | 提交自身产生的块从聚簇段分配；用户数据块的落点不受此约束 **状态：已定。** |
| 6 | **准入的在飞合成** | 权威正文在 D28（挂载期承诺量） 已定项 2，这一格只留结论句与指针 **状态：已定。** |
| 7 | **分配记录条目的盘上编码与宽度** | 落点粒度 16384；key =（设备身份 4, 槽号 6）；value =（跨度段 2, 代 8）；释放时改写不点删 **状态：已定。** |
| 8 | **用户数据块的默认落点政策，以及段耗尽时的回落** | 第一版不进聚簇段，取已选设备内起点最小、整跨度过准入合取的空槽；段耗尽回落同一规则 **状态：已定。** |
| 9 | **不许有假性 ENOSPC，删掉的空间在有界步数内可用** | `df` 报出 s 字节空闲就写得进 s 字节；删掉 s 字节之后同样大小的写在有界步数内成功 **状态：已定。** |
| 10 | **聚簇段的定义、用户数据落点的四格、提交内生块的 bump 次序** | 段长 64 槽、起点是最低的 64 槽对齐全空段；bump 次序按树 ID 升序、树内先叶后根 **状态：已定。** |
| 11 | **分配记录 20 字节里跨度段归 key 还是 value** | 进 value：key 10 字节、value 10 字节；已释放标志仍借跨度段最高位 **状态：已定。** |
| 12 | **分配准入合取的唯一登记位，与两条准入式的关系** | 合取式只登记在已定项 7 一处、共七条；两条准入式是串联的两道闸 **状态：已定。** |
| 13 | **分不分 data / metadata 两类空间** | 不分，全局统一分配器 **状态：已定。** |
| 14 | **不分类带出来的连带承诺** | 后台整理必须常驻、自动、用户无需知道；承接方是 D26（后台整理与放置回收） **状态：已定。** |
| 15 | **空间预留取什么形态** | 取准入控制：任何增加空间占用的操作进门前先算最坏情况下解开自己需要多少，算不过就拒绝 **状态：已定。** |
| 16 | **加密开启时无密钥的一侧能做什么** | 只读只检测，既不搬也不回收；常驻自动整理那条承诺在加密开启时不成立 **状态：已定。** |
| 17 | **释放空间这个操作本身不需要申请空间** | COW 系统必须保证它；释放路径要用的空间在写入那一刻预付掉 **状态：已定。** |

```
