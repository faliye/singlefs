# 第 30 批代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers M1 M2 M3 M4 E1 B1 B2 G1 KX1 KX2 -->

## 一、题面

第 30 批（`records/2026-10-01-里程碑三形式化接入.md#30`）照判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第三节第 1 条与用户 2026-10-05 的定案（T_time 盘上单位改毫秒）改，三份补丁，改动全文 `research/prompts/_m3-formal-code-b30-r1-diff.md`：

- 30-1：新文件 `crates/singlefs-harness/tests/model_compares_the_third_and_fourth_inode_leaf_containers.rs` 三条用例（建到第 4 片；回退到第 3 片被重写过的那一版；回退到没有第 4 片的那一版），四个变体（胶水封顶第 3 片、模型封顶第 3 片、回退只取回第 2 片、回退只丢第 3 片）各一行变异（B1、B2）。
- 30-2：池级 checker `crates/singlefs-checker/src/walk.rs` 墓碑树 / 谱系树容器子指针判 extent 偏移（E1）、I-5.4 按版本记走没走到叶；GPU 走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 同一处 E1；GPU 恢复内核 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 种类 ≥ 32 的重复逐条比、加密 / 压缩几段（MAC / nonce、算法类型、压缩码、压后长度）照 core 补 8 处；`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 文件头；坏镜像进 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，对拍进 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`。
- 30-3：`crates/singlefs-core/src/system_configuration.rs` T_time 改毫秒（`TIME_THRESHOLD_MILLISECONDS = 5000`），挂载透明 G1 期望常量（`crates/singlefs-harness/src/mount_transparency.rs`）照改，三个 harness 钉值重取。

条款：判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md`；`.claude/kb/invariants.md` I-2.4、I-5.4、I-9.16 三行；`.claude/kb/decisions/16-发布语义.md`「#### 已定项 5」；`.claude/kb/layout/01-first-txn.md` 系统配置字段表（第 149 行 T_time、第 158 行偏移说明）；用户定案原文「改成毫秒：字段表写明「单位毫秒」，默认值 5000；core 与 G1 一起改。D16 的 200 ms 写得出」。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-b30-r1/tree/`（主仓第 30 批合入后的 `crates/`、`Cargo.toml`、`Cargo.lock`，sha256 清单同目录 `crates-sha256.txt`，309 行）。树里另有删除线第 20 批（`walk.rs` 删除队列、`crash_judge_common.wgsl` 找第一个文件补 nlink 等），不在这一轮被判。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。要编就拷到自己的草稿目录。
- 合入验证 `/tmp/claude-1000/merge-and-verify/2026-10-05-2363396/`：harness 档 164 个测试二进制全绿；fmt、`--all-targets --all-features` clippy、单开 gpu / verdict-store 两种 clippy 都过。checker 档一条没跑：GPU 对拍新加的 9 条与 GPU 变异 9 行只编过（30-2 用 naga 静态校验过）；`crash_enumeration_sharded_across_processes.rs` 第 844、850、851 行三个钉值读码会因 T_time 字节变（30-3 报告第六节），等提交时重取。
- 30-2 报告：harness 档全量约 1571 次 `check_pool_image` 新旧 I-5.4 判法只在 3 个故意坏的镜像上不同；extent 内部条目子指针与实例表链指针两处加密 / 压缩判定造不出流、没有变异行；新诊断码 65、66 只当诊断。

方案按 `crates/` 今天的实现来谈。

## 三、格

### M1　30-1：第 4 片与回退用例
三条用例罩不罩得住它们要证的行为：再换一个取样点（第 5 片及以后、回退到更旧的根、回退后不建或多建）四个变体还红不红；四行变异的锚点与已有 KX1 / KX2 那几行会不会互相掩护（同一原文、不同替换文）。

### M2　30-2：E1、种类 ≥ 32、I-5.4 按版本、GPU 加密 / 压缩对齐
E1 补的那一判有没有让合法历史误红（墓碑树、谱系树在删除进行中、回退之后、多实例下）；墓碑树 / 谱系树之外还有没有走读跟随、却不判 extent 偏移的指针（`.claude/kb/invariants.md` I-2.4 那一行说每条都判）；GPU 恢复内核种类 ≥ 32 逐条比与 core 在同一盘面上结局是不是逐格相同；I-5.4 按版本在合法历史上会不会多出「部分没判」、在坏盘上会不会放过；加密 / 压缩 8 处与 core 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from` 是不是同一组字段、同一判法，有没有一处 GPU 拒而 core 收或反过来。GPU 跑不了的读码、在 CPU 侧造盘面，写「推的」。

### M3　30-3：T_time 改毫秒
core 写出与读取、G1 期望与字段表第 149 行三处是不是同一个单位、同一个值；全仓还有没有按秒解这 4 字节的地方（`crates/`、`research/` 的装置）；三个新钉值的出处站不站得住；checker 档读码判不变的那些钉值，判得对不对（池级 checker 真不读偏移 433？崩溃枚举的计数、镜像摘要还有哪些吃整份系统配置槽）。

### M4　主 agent 两处
变异表那一行只改名、锚点与替换文没动，会不会让按名字复用的判定记录（`research/scripts/crates-mutation-rows.py`）对不上；字段表第 149 行的写法与 D16 已定项 5 的定案句对不对得上。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | M1–M4 | 从判决、条款与用户定案正推每处改动做的是不是它们说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | M1–M4，重点 M2、M3 | 找反例：一段合法历史让新判法误红，或一个坏实现让它放过；在拷出的副本上造、量 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮。
- 重型测试不跑：checker 档的测试、GPU 对拍与 `crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b30-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b30-r1-<腿名>-model/`，带一份 `tools/rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。

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

### 小节清单：`.claude/kb/decisions/16-发布语义.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D16 发布语义 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D16 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：checkpoint 的触发判据取时间与脏数据量，两者取先到 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：journal 记录格式是四层之外的第三个独立冻结组件 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：一次发布整体施加或整体不施加，第二份 replay 停在句法层 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：`T_time` 与 `T_dirty` 取多少，丢失窗口的口径 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：发布计数器，四个等号写成一条 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：发布的持久顺序 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：暖机取戊，新实例的根先覆盖两块盘 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：一次空发布也写单元 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：发布粒度取攒批，不是每个事务发一次 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：没有任何操作可以绕过 journal 直接发根 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：中间状态的合法性语义改写成「根 + journal 任意前缀」 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：这份 journal replay 归记录核对器，不归 O2（独立解析器 + checker） | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：快照发布点与 checkpoint 边界对齐，钉在全部数据变更之后 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：checkpoint C 中产生的释放，C 发布之前不进可分配集合 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：checkpoint 与消息缓冲买同一样东西，E7（离线索引 harness） 的对照组钉死 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：fsync 报错之后，调用方能假设什么 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/layout/01-first-txn.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 新池新建文件写出哪些字节 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （新池新建文件写出哪些字节 标题之下、第一个下级标题之前的正文：第 2-27 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 零、盘面地图与新池新建文件的写清单 | 抄 | 默认全抄（脚本生成） |
| ## 一、mkfs 已经种下的（新池新建文件之前就在盘上） | 抄 | 默认全抄（脚本生成） |
| ## 二、数据这一侧 | 抄 | 默认全抄（脚本生成） |
| ## 三、指向数据单元的指针（住在父节点里） | 抄 | 默认全抄（脚本生成） |
| ### 三·二、中央映射条目（2026-09-06 新增，D19（块指针的结构与宽度预算） 已定项 5） | 抄 | 随父节带到 |
| ## 四、文件元数据（inode） | 抄 | 默认全抄（脚本生成） |
| ### 四·二、extent 树（2026-09-10 新增，此前这张表里没有它） | 抄 | 随父节带到 |
| ## 五、空间记账与分配 | 抄 | 默认全抄（脚本生成） |
| ## 六、journal | 抄 | 默认全抄（脚本生成） |
| ## 七、发布（根记录与根槽） | 抄 | 默认全抄（脚本生成） |
| ## 八、根槽写路径的段序列登记表（C316（提交步骤的登记位有四处且互不相同） 的登记位，2026-09-13 立） | 抄 | 默认全抄（脚本生成） |
| ## 还剩几处空白 | 抄 | 默认全抄（脚本生成） |

**出处 `.claude/kb/invariants.md:1-8`（整段抄，未转述）**

```markdown
# 不变量清单

**checker 是 invariants.md 的可执行形式。** invariants.md 每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../rules/format-evolution.md)「硬约束」）。

每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。池级 checker 对一条不变量的判定有四种：成立、不成立、不适用（必须带理由，不许静默算成立）、部分没判（`InvariantVerdict::PartiallyJudged`，`crates/singlefs-checker/src/image.rs`）。部分没判是判到的那一部分都对得上、射程里另有一部分这张镜像上判不了，说明逐条列出没判到的是什么；它不是红，也不是成立：层 0 与崩溃放量把它算成不红，层 0 计数并进「不适用」那一格，GPU 判器只记违例位、它在那一侧与成立、不适用同是 0。今天只有 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）报它。
写不成可判定形式的，说明还没想清楚，不许进 invariants.md。

```

**出处 `.claude/kb/invariants.md:9-20`（整段抄，未转述）**

```markdown
## 状态

**新池新建文件阶段（2026-09-14 起）。** 磁盘格式已落成字节：mkfs、新池新建文件的写路径与冷启动恢复在 `crates/singlefs-core`；checker 在 `crates/singlefs-checker`，与实现只共享常量模块（D13（验证路线） 已定项 5）。
池级 checker（`walk::check_pool_image`）判 74 条（`crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 的长度；状态列写「已实现」的行用 `grep -c '^| I-.*已实现' .claude/kb/invariants.md` 数，也是 74 条，其中 I-3.5（引用区间的精确性） 是部分实现：出现规则已实现（下标 65），③ ④（deadlist 那两条）未实现，I-4.4（意图只描述目标态） 在清单里（下标 73）、第一版恒报不适用；第一版 23 条，2026-09-16 里程碑「覆盖写、释放、回退与复用」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），2026-09-18 加 I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 与 I-5.4（分配记录罩住的槽互不相交），2026-09-21 加 I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号），2026-09-22 加 I-1.10（码 2 条目宽等于字段表宽）、I-9.6（水位大于两处最大号）、I-9.12（分隔 key 落在孩子区间之外）、I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块），2026-09-23 加 I-8.8（前缀里的事务不被切开） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号），2026-09-24 加 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读） 与 I-8.9（一次发布的记录序号连续且只有末条带标志），2026-09-25 加过 I-7.10（已停用·回退见证表删了） 与 I-7.11（已停用·见证择根删了）、2026-09-26 随回退见证删掉退役，同日加 I-7.12（系统配置 F 不低于同盘根上的 F），2026-09-27 加 I-1.11（映射 key 与单元头相符），同日加 I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序），2026-09-28 加 I-7.14（系统配置本盘设备号等于盘在池里的身份），2026-10-02 加里程碑三二·1 那一批 16 条（I-1.12（指针出生树等于被指单元头）、I-3.12（实例表行 T 低于引用它的根的 txg）、I-3.13（记账行代段等于这一版的 txg）、I-4.1（重放后树可遍历）、I-4.9（非最新候选根引用的单元不晚于它）、I-7.5（根槽按判定宽度对齐）、I-7.15（各盘所选系统配置槽设备数相同）、I-7.16（系统配置槽世代号与槽号的关系）、I-7.17（根槽位置等于 txg 算出的位置）、I-8.1（环几何够大）、I-8.5（tail 槽有效集非空）、I-8.10（记录新根段与同代根相同）、I-8.11（点名项与被指单元头相同）、I-8.12（同实例记录的 txg 不减）、I-9.25（树表条目的 previous_snapshot_txg 取值）、I-9.26（inode 改动计数落在出生代与容器诞生代号之间）），2026-10-04 加 I-9.27（树表条目的头 ID 取值）、I-5.5（从 mkfs 水位建树那一版的 t1 落点） 与 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），2026-10-05 加里程碑三·3 删除线那五条：下标 69 的 I-9.5（删除队列里的号仍在树里）、下标 70 的 I-3.4（可用空间扣待删占用）、下标 71 的 I-4.7（删除中 inode 有账）、下标 72 的 I-4.6（中间态其余不变量成立） 与下标 73 的 I-4.4（意图只描述目标态）），状态列写「已实现」的就是这 74 条（I-3.5（引用区间的精确性） 只算出现规则那一半，I-4.4（意图只描述目标态） 的判定恒是不适用）：除 I-4.4（意图只描述目标态） 外每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`，删除线那几份的搭建与改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`），层 0 的每个崩溃状态都跑它。
I-7.7（系统配置实例代号不低于根环） 2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 定案改成 ① ②，层 0 里 0 个状态判红；I-3.1（已分配统计对得上） 与 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 的 checker 读法 2026-09-14 由用户收尾弹窗定甲，写在各自那一行。
其余条目仍写「未实现」：它们要覆盖写、释放、回退与复用（释放、重用、残留）、journal 一侧的那一类检查、快照或加密，新池新建文件的镜像上判不出东西。

违例位是 128 位（4 个 u32 字，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 的 `POOL_CHECKER_VIOLATED_BIT_CAPACITY`），位 i 对 `IMPLEMENTED_INVARIANTS` 的第 i 条（`pool_checker_violated_bits` 读成 u128）；今天用了 74 位；`POOL_CHECKER_COVERED_INVARIANT_INDEXES` 列 74 个下标，就是 `IMPLEMENTED_INVARIANTS` 的每一条（下标 65 的 I-3.5（引用区间的精确性） 走读内核在回退候选集的版本都走完之后逐对判；66 是 I-9.27（树表条目的头 ID 取值）、67 是 I-5.5（从 mkfs 水位建树那一版的 t1 落点）、68 是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）；69 是 I-9.5（删除队列里的号仍在树里）、70 是 I-3.4（可用空间扣待删占用）、71 是 I-4.7（删除中 inode 有账）、72 是 I-4.6（中间态其余不变量成立），这四条由扫描内核的删除态扫描在回退候选集的每一版上判，读不全的版不置位；73 是 I-4.4（意图只描述目标态），第一版池级 checker 恒报不适用、内核不置位），没有「内核还没判」的那张表；I-9.6（水位大于两处最大号） 墓碑那一半也在扫描内核里判、并进下标 41 那一位；编译期断言覆盖表按下标升序、一条不缺一条不重（第 k 项是 k）（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行判定的位宽加宽与 GPU 内核补齐）。

<!-- invariant-count -->
现共 105 条在用（编号至 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），另有六条退役，编号不回收）。这一句是当前条数的权威登记位；「## 历史版本」里那几句「现共 N 条在用」记的是当时的数，不跟着改。

```

**出处 `.claude/kb/invariants.md:21-47`（整段抄，未转述）**

```markdown
## I-1 块自描述 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-1.1 | 块头自述逻辑地址 | 任一单元，其头记录的**类内身份**与实际引用一致——数据单元的身份是五元组（锚点偏移即它的逻辑地址）；索引节点是树 ID + 层级（+ 该树已定携带的 key 区间，D18（块里携带什么信息）已定项 2 口径）；打包记录单元（码 3）是（出生树 ID, 打包记录类型, 容器号, 容器出生代）四元组，「父级」按打包记录类型分——类型 1 走容器索引的身份引用（C103（容器索引无落点）），类型 2 走 inode 树内部节点条目的身份引用（I-9.2（条目身份与子头相符）），**类型 4（实例表）走根记录的物理指针，不属于任何树、不进容器索引**。**自证单元（journal 记录 / 根槽 / 系统配置槽 / 类型 4 实例表单元）显式豁免**：无父引用无树身份，自证纪律在 I-7（根环）/ I-8（journal）两类与 D22（单元原子性怎么合成）已定项 21；多层码 2 树（记账树、中央映射树）的索引节点，身份里的层级与 key 区间按引用它的父条目核：层级 = 父 − 1、区间落在父条目给的那一段、内部节点区间是子树覆盖区间；按位置寻址的两棵树（分配记录树、extent 树），节点头里的层级与 key 区间等于它的位置规定的那一层、那一段；内部条目的 key 是某个孩子那一段的起点，孩子按位置严格递增；分配记录叶里每条记录的起点与末槽都落在叶罩的那一段里；extent 上段叶条目的 inode 号、下段叶记录的单元号落在叶罩的那一段里，上段叶条目的标签只能是 0、1、2（标签 0 载荷全零、标签 1 载荷末 2 字节为零）；根之下没有空节点（这一半 checker 由实二五补）。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；`walk::Walk::walk_code_two_subtree`；坏镜像用例 `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`；`crates/singlefs-checker/src/position_addressed.rs`） |
| I-1.2 | 块头写序已发布 | 任一单元，其头记录的写序 (i, n)（D18（块里携带什么信息）已定项 7 / 11 的字段）与诞生代号 b 按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的全局已发布谓词判成已发布：i < 挂载根实例且无该实例的行 ⇒ 已发布，有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub（所选根那个实例的 T_pub = 重放之后那个根的 txg，严格介于所选根的实例与新实例之间、又不是切换照旧事务写序主人的实例写 (i, 0, 0)，切换的所选根按 D23（journal 的角色与格式） 已定项 14 取被重发的在飞 checkpoint 所基于的根；W = **属于实例 i 的**、被这次重放施加的最大事务号，切换时是被照旧的、写序属于 i 的最大事务号；同一次恢复写出的那批行里只有所选根那个实例的 W 可能非 0，同一次切换写出的那批里只有被照旧事务的写序主人的 W 可能非 0，表上允许有多行非 0，它们来自不同次的恢复或切换；2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；i == 挂载根实例 ⇒ b ≤ 挂载根的 checkpoint_txg；i > 挂载根实例 ⇒ 判损坏。**例外**：打包记录类型 4 的实例表单元不走这条谓词，它的现行版本由根记录持有的物理指针唯一确定。**判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留**——这是扫描方向独立于遍历方向判它的依据（D13（验证路线） 已定项 12 双方向要求）；2026-09-05 前它只比诞生代号与当前 txg，崩溃后 txg 重发让那条判据对同 txg 两条时间线失效（E104（扫描重建的现行版本判定）：无实例表时崩溃世界复活 6） | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`，**只判遍历方向那一半**：走读到的每个被引用单元，头里的诞生代号 b、写序的实例代号 i 与尾段（码 1 是写序的事务号 n、码 2 / 码 3 是出生序号）要与**引用它的那条指针**记的相同——不同则「头记录的写序」不是这个单元的真实身份，谓词判的就是假输入；口径同恢复路径今天的前置检查（`crates/singlefs-core/src/recovery.rs` 五处报 I-1.2（块头写序已发布） 的 `InvariantViolated` 里「出生身份与指针不符」那三处；「诞生于根之后」那两处是谓词那一半，归 I-4.2（无被引用未提交块）），checker 按字段表自己解一份。⚠️ **扫描方向那一半（「任一单元」都按谓词判、i > 挂载根实例 ⇒ 判损坏）对着一份崩溃镜像判不了，没写**：① 判不成已发布本身不是违例——本行判据自己写着那是未发布的撕裂事务垃圾或被抛弃时间线的残留；② 「i > 挂载根实例 ⇒ 判损坏」只在恢复刚发布完新根的那一刻成立，而上一次挂载取了号、写出单元、一个根都没发布就崩时，盘上合法地留着 i > 最新根实例的在飞单元，那一支今天由 I-7.7（系统配置实例代号不低于根环） ① 按各盘系统配置的实例代号兜着。谓词本身照旧在用：`PublishedPredicate` 过滤 I-1.8（归并后版本全序） 的射程、判 I-4.2（无被引用未提交块）。打包记录类型 4 的实例表单元按本行的**例外**只免谓词那一半：根记录偏移 170 的实例表指针与链上每一片，指针里的出生 txg、实例代号、尾段与被指单元头里的值相等，仍挂这一条的遍历方向（已实现：`walk::Walk` 的 `judge_birth_identity_against_the_pointer`，被引用单元与实例表链上每一片都判；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）。坏镜像是 extent 树根头里的出生序号 +1、引用它的指针一个字节不动（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它，走读一个被引用单元都读不到的状态报「不适用」） |
| I-1.3 | 块头树 ID 一致 | 任一单元，其头记录的所属树 ID 与实际引用它的树一致。中央映射树不进树表，「实际引用它的树」取根记录（或记录新根段）里中央映射树根指针带的出生树，不写死 65540：回退到无文件那一版之后再发第一个文件版本，十二棵树从水位起重新发号（D8（核心索引结构） 已定项 8 ②），映射树的号随之变（主 agent 2026-09-23 定，C511（回退到无文件那一版之后诞生代怎么接） 第 3 步实现员查出写死 15 的读法在合法的回退历史上误红）。自证单元（含类型 4 实例表单元）与 I-1.1（块头自述逻辑地址）同口径显式豁免。⚠️ 打包记录单元（码 3）与被快照共享的数据单元头里的树 ID 是出生树，跨头共享时与引用树不同，那一情形的判法随 D9（加密） 那次「绑出生快照」的重开（C110（跨头共享与加密后元数据类的块头一致性读法））。这一射程覆盖码 3 全部打包记录类型，含类型 2（inode 树的叶，D8（核心索引结构） 已定项 6）：克隆头共享 origin 的叶时，叶头的出生树是 origin | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.4 | 块头 fsid 一致 | 任一单元，其头记录的 fsid == 系统配置 fsid（D18（块里携带什么信息）已定项 7 的字段；加密开启后与 KDF 绑定冗余，接受）。scrub 半边「本卷全部单元归属指向本卷」靠它才写得出（C74（无加密期的跨卷错向写没防线）补救的另一半） journal 记录头 2026-09-14 起也带 fsid 8（系统配置 fsid 低 8 字节，D23（journal 的角色与格式） 已定项 4）：fsid 不符的记录整条不进重放前缀、按断链处理（E142（新池新建文件的干跑） 第七次跑记的 G25） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.5 | 系统配置记管道身份 | 系统配置记录写出该镜像的管道身份（实现标识 + 版本），且非空。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：自举头段「写入者身份 20」= 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32，第一版写 `singlefs-rs` / 1；实现标识全零判红 | 未实现（条款已写、检查仍欠） |
| I-1.6 | 类标签登记且三处相等 | 任一单元，共同前缀偏移 6 的单元类型标签 ∈ D18（块里携带什么信息） 已定项 11 的登记表（码 0 与未登记码即判损坏）；偏移 7 的 8 个 flag 位第一版恒 0，非 0 一律拒收（前向兼容政策，D15（格式冻结政策） 判定表）；数据单元（码 1）五元组首字节 == 偏移 6；打包记录单元（码 3）类身份段首字节 == 偏移 6 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.7 | 打包容器合法与判定顺序 | 任一打包记录单元（码 3）：记录数 × 记录宽 ≤ 声明长度且声明长度 ≤ 32768 − 136（含 29 字节预留位的头，D18（块里携带什么信息） 已定项 16；2026-09-14 预留位从 28 加到 29，`cap(136, 40) = ⌊32632 / 40⌋ = 815` 与按 135 算同值，40 是第一版墓碑记录宽，D18（块里携带什么信息） 已定项 24），**先于**载荷校验和判（D23（journal 的角色与格式） 已定项 1 的 B 条同一纪律）；判定顺序恒为头校验和 → 类身份段 → 记录宽 == 登记表为该打包记录类型登记的宽（类型 2 是 140，D8（核心索引结构） 已定项 6；类型 1（墓碑）是 40、类型 5（谱系）是 16，D18（块里携带什么信息） 已定项 24；不等判损坏，只需头字段）→ I-1.2（块头写序已发布） / I-1.3（块头树 ID 一致） / I-1.4（块头 fsid 一致） → 边界 → 载荷校验和 → 逐条记录，任一步失败即中止；打包记录类型 4 的两种 kind 记录另按 D18（块里携带什么信息） 已定项 11 的实例表字段表判，不套用「记录宽 == 登记表的宽」那一步（一片记录宽恒 88，条数上限由片容量给——D18（块里携带什么信息） 已定项 11 2026-09-13 从 64 改宽）（对象级的判定——I-9.7（记录三段恒零）、I-9.8（记录 inode 等于 key）——在这条中止链之外，只让那条记录 EIO）；容器头声明一种打包记录类型、一棵出生树；打包记录类型 ∈ 第二级登记表，未登记类型的容器其物理指针不得出现在任何回收 / 复用操作的输入集合 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；类型 1、类型 5 两种记录宽未实现，三·3 实现） |
| I-1.8 | 归并后版本全序 | 码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组（字段表在 D18（块里携带什么信息） 已定项 7（码 1）与已定项 11（码 3）；归并键取这些字段但去掉载荷校验和那一段——它正是 ①「同一组的成员载荷相同」要比的那个量，进了键 ① 就恒真）之后，同 key 的各组两两全序键不等——码 1 的键是写序，码 3 的键是 (诞生代号, 实例代号, 事务号) 三元——诞生代号打头（同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同），事务号留在末位做同一 checkpoint 内的平局破除（固定点会迭代，同一容器在一个 checkpoint 里不保证只写一次）；同一组的成员载荷相同（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 副本归并，射程 w = 2；码 2 不进这条） | 已实现（2026-09-21，池级 checker `walk::check_pool_image` 判 ① 同一组的成员载荷相同；② 同 key 的各组两两全序键不等**只判码 3**——码 1 的键停着等 C464（I-1.8 对码 1 的全序键写窄了） 定案，checker 里显式写成 `TotalOrderKey::UndecidedForDataUnit`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.9 | 清扫水位内无可读垃圾 | 清扫水位（D5（快照 / 空间记账机制） 已定项 4 第 13 项）以内不存在可读垃圾；其外的可读垃圾每个 key 都被更大写序的可见版本或可见墓碑压住（管理员回退之后的镜像不在此列，见 C615（扫盘重建对挂着时的回退没有输入），出处 `research/prompts/m3-tomb-r1-main-verification.md` 第二节；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4；扫描重建的产物未晋升期间清扫禁止，这条在重建之后失效） | 未实现 |
| I-1.10 | 码 2 条目宽等于字段表宽 | 任一码 2 索引节点，头里自述的**条目宽**（D18（块里携带什么信息） 已定项 7 的自描述三段之一：key 宽 1 + 条目数 2 + 条目宽 2，偏移表在 D18（块里携带什么信息） 已定项 18、载荷布局在 D8（核心索引结构） 已定项 11）**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）。条目宽是**盘上读来的值**，`index_node_view` 只判了它 ≥ key 宽（extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」——不判就按字段表的固定偏移切，切片当场越界（增补 3 第 6 件 panic 面普查第二族，C476（盘上内容可控时走得到的 panic 有 21 处） R1 / R3 / R4 / R13）。条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它），否则判该节点损坏（2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做）。判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目。⚠️ **中央映射树还不判**：D19（块指针的结构与宽度预算） 已定项 10 已定映射树 key 一律 27 字节（码 2 / 码 3 末尾补零）、条目一律 55 字节，但读者只拒窄于 55 的条目，`field_table_width_of_tree_kind` 不认种类 5，宽于 55 的映射节点不判损坏，账在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）；**树表条目（200 字节）两边都没写**，而走读照样按固定偏移切树表节点的条目——要不要把射程扩到它，账在 C476（盘上内容可控时走得到的 panic 有 21 处） | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::walk_tree_table_entry`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；条目宽为 0 时条目数非 0 那一条 2026-09-27 实审 A3-checker-2 落地：`index_node_view_judging_a_zero_entry_width`（`crates/singlefs-checker/src/walk.rs:747`）在解码失败理由是 `Verdict::EntryWidthZeroWithEntries`、且这棵树登记了条目字段表（`EntryFieldTableRegistration::Registered`）时报违例；树表条目与中央映射树不在射程内，只记走读失败；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
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
| I-7.9 | 回退下界 F 不高于抬 F 的上限 | **抬 F 的那一条根**（它带的 F 比同一实例里 txg 比它小的最新那条根带的高；新实例的第一条根没有同实例的前一条，不算抬——它带的是恢复算出的 F_生效；管理员回退是现行实例里的一次向前发布，带的 F 与前一条相同，不算抬；一次抬 F 推的几次空发布只有第一条算）根记录里那个回退下界 F（D22（单元原子性怎么合成） 已定项 7 的字段），按这条根 flags 里的**卸载记号**（D22（单元原子性怎么合成） 已定项 7 flags 位 0）分两支判：**带卸载记号的**，F 不高于根环里 txg 比它小、又落在两个一圈窗口的交里的最新那条根的 txg（两个窗口的取法与交为空时不判，同不带记号那一支的「之前的根」；正常卸载把 F 抬到现行那一版的 txg、不另判上限，D16（发布语义） 已定项 1「抬 F 的上限」）；**不带的**（准入抬 F），F 不高于**用它之前的根算出的** D16（发布语义） 已定项 1 的准入抬 F 上限：min(每块盘上最新的持久有效根的 txg, 第 4 新的非空持久有效根的 txg)，非空有效根不足 4 个时取最旧有效根的 txg。有效 = 按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F；「之前的根」（两支都取）取两个一圈窗口的交里 txg 比它小的那些：一个窗口锚在被判那条根的 txg 上，一个锚在镜像最新有效根的 txg 上（窗口 = txg > 锚 − 3S，3S 是环长，照 D16（发布语义） 已定项 1「一圈窗口里最旧有效根」那一行；用户 2026-10-04 定「乙，两支都取两个窗口」），交为空（镜像比被判那条根新出一圈以上、它之前的根都被盖掉）时那条根不判；只取锚在被判那条根上的窗口时，洞里留着的上一圈的旧根会顶上来、在合法镜像上判红（随机历史 230 段红 21、150 段红 9，并上锚在镜像最新有效根上的窗口之后各剩 1 段，红在带卸载记号那一支，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批 9-6 交回那一条）；取全环时攻方五批随机历史里每批 27–46 段红（判决 `research/prompts/m3-root-ring-code-r2-main-verification.md` 第五节）；上限照 D16（发布语义） 已定项 1「抬 F 的上限」那一行，窗口里挂载那一刻读不出、之后这个进程没写过的槽当作可能是一条非空有效根、F 不越过它，这些槽 checker 看不见，checker 只按镜像上读得出的根判（推的：读故障不留在盘上）。只判抬 F 的那一条，不在后来每张镜像上重算：F 抬过之后再回退，后来的根会把当初撑着 F 的非空根判成无效，现算的上限会低于 F，而 F 不回落（用户定案 2026-09-24，理由见实二报告：按「有效根不要求 txg ≥ F」的读法，固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与两个随机种子在合法状态上判红）；非空 = 它的树表里 inode 树、extent 树的根指针与它前一条有效根（按 txg 排）树表里的不同——比的是树表里这两棵树的条目，不比树表单元自己的落点（D16（发布语义） 已定项 1 表格后那段「「非空」从盘上怎么认」）；数「第 4 新的非空」时去重：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入（同一段）。⚠️ 它判的是一张镜像上 F 有没有抬过头，不判 F 该不该抬（那是准入的事，D28（挂载期承诺量） 已定项 1）。⚠️ **卸载记号盘上没有别的东西核它**：准入入口误打记号、F 又抬过头，按带记号那一支放过（第三轮判决第二节 K6：(b) 丢的正是「准入抬过头却带了卸载记号」那一格）——这一格归层 0 录制流另核「卸载记号只由卸载入口打」（C557（卸载记号只由卸载入口打没有检查）），不归这一条。⚠️ F 写进了系统配置而带新 F 的根没落盘或坏了、恢复后生效值怎么取的那一类，归层 0 与 C556（checker 与层 0 不读系统配置里的 F），不归这一条。算上限要比的有效根树表读不出时，它空不空判不了、不猜：F 不高于把这些根全算空得出的上限就成立，高于全算非空得出的上限就违例，落在两者之间这条根不判 | 已实现，两支（不带记号那一支 2026-09-24、带卸载记号那一支 2026-09-26；两支的「之前的根」2026-10-04 起都取两个一圈窗口的交）：池级 checker `walk::check_pool_image` 的 `judge_rollback_floor_raises_against_their_ceilings`，根环里每条抬 F 的根各判一格，按根记录读出的卸载记号分支（checker 读成 `UnmountMarkerView`）。**不带记号那一支**：坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（步 5 那段历史上抬到上限 11 与之后再发一版都成立、抬到 12 只红它）；固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与随机历史各档不红；层 0 `crash_enumeration_fixed_script_stream.rs` 抬过 F 的三条流要求真被评估过；`crates/mutations.tsv` 7 条变异（判法拿掉、上限取大、第 4 新改第 3 新、有效按新 F 取、不限同一实例、一条都不认、「4 个不同状态」不去重）；用户 2026-09-24 定义，里程碑「覆盖写、释放、回退与复用」收口表第 26 行。**带卸载记号那一支**：坏镜像同一文件的 `marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant`（正常卸载之后的镜像按带记号那一支判 6 ≤ 6 成立、按准入那一支会红；把那条带记号的根的 F 改成 7 只红它）；变异「checker 把每条根都读成不带卸载记号」「带记号那一支把不高于前一条根的 txg 判成低于」；记号的写读在 `crates/singlefs-core/src/root_record.rs`（写者照 `unmount_marker` 写 flags 位 0、读者拒记号以外的位），写者、读者各有一条变异（实二）。**两个一圈窗口**（用户 2026-10-04 定，被攻过零轮）：harness 用例在 `crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs`（带卸载记号的根在洞上再挂之后不许红，用例 `an_unmount_sequence_root_after_a_hole_one_revolution_later_is_not_judged_against_the_old_root_left_in_the_hole`；两个窗口里仍照红，用例 `an_unmount_sequence_root_raising_the_floor_above_the_previous_root_inside_both_windows_still_reddens_the_rollback_floor_invariant`；准入那一支的几形同文件），`crates/mutations.tsv` 里以「第9批改钉」打头、点名带卸载记号那一支的 2 行，与以「第9批」打头、点名「一圈窗口」的 4 行；改钉之后随机历史 230 段、150 段两批 0 段红（改钉簇实现员量的，输出在 `research/prompts/m3-r1-batch9-repin-evidence/window-fuzz/`）；GPU 判器的扫描内核两支同取两个窗口（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `in_the_one_revolution_window`），对拍用例与 8 行变异在 checker 档、没跑 |
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
| I-8.6 | 反向链算法 | 任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 487 字节头，`header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录反向链恒 0**（D23（journal 的角色与格式） 已定项 10 / 已定项 19，2026-09-13 用户定案，链按实例算是 2026-09-14 用户定案，头宽随 D23（journal 的角色与格式） 已定项 4 现行 487）。不等的记录不进重放前缀。jsn 的计数器为 0 的记录判违例：它没有本实例内的逻辑前一条，反向链无从定义（D23（journal 的角色与格式） 已定项 18 的落点式 `(计数器 − 1) mod 槽数 × 4096` 从计数器 1 起；用户 2026-09-27 定）。⚠️ 链按实例算，所以跨实例边界的两条记录之间**不判**这一条——实例边界由实例代号挡着（I-8.3（重放前缀严格连续)） | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，逐盘判：计数器 1 与跨实例边界的第一条判反向链恒 0，同实例前一条在盘上时判链值，前一条不在盘上不判；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 里环还空着的 4 个状态报不适用，其余每个崩溃状态都判）；计数器 0 判违例（2026-09-27，`walk.rs` 的 `ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`，用例 `a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant` 在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`） |
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
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**2026-09-17 起「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」），且落在一圈窗口里（txg 大于最新根的 txg 减 3S，3S 是根环槽数，`walk::check_pool_image` 里的 `OneRevolutionWindow`，D16（发布语义） 已定项 1「一圈窗口里最旧有效根」那一行；窗口之外的是环上有洞时洞那一槽里留着的上一圈的根）。F_生效 由 checker 自己算：取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与层 0 不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于回收门槛的（还在 defer 里，分配器还不许回收；回收门槛 = max(F_生效, 一圈窗口里最旧有效根)，窗口里一条有效根都没有时取 F_生效，D16（发布语义） 已定项 1「可再分配」那一行），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15，走那条记录新根段里的实例表指针与分配记录树根（`walk::walk_version_applied_only_by_records`；用户 2026-10-02 定新根段回到三字段，实例表指针与分配记录树根随新根段换成那次发布的）。⚠️ **没量的前提**：「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`research/prompts/m2-rev-b2-implementer-report.md` 第七节第 2 条）；量它是 C699（取号跳过改认出生代号） 的第一步。 |
| I-3.2 | 已停用·方向反了 | **此编号不再使用**——它写成「≤ 引用它的**最新**快照的代号」，方向反了：紧的约束是最**旧**的那个。旧写法被 I-3.5（引用区间的精确性） 的子判据 ② 严格蕴含，且检不出任何一类真实错误 | 不适用 |
| I-3.3 | 已停用·归属未定义 | **此编号不再使用**——「所属快照」未定义，而两种归属约定给出不同的比较符。由 I-3.6（deadlist 紧界） 取代 | 不适用 |
| I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ①② 合成一条**出现规则**：取回退候选集（I-7.4（近 K 代块未被复用） 用的那一份，加由记录施加出来、根槽从没落盘的那几版）按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 为一对，逐对判。R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判，`Reach(b)` 在 txg 序上是否连续不再单判）满足 (a) `birth(b) > R⁻.txg`；或 (b) b 落在 R 树表里某棵用户可见树（今天 extent 树、inode 树，`deleted_inodes` 树根非全零之后加它）的子树里，那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg（读不出头的单元不计入）。`birth(b)` 取 b 的单元头里的诞生代号；b 自己的头读不出时这一个落点不判，读不出归 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配）。**缺口**：R.txg ≠ R⁻.txg + 1 时这一对之间有洞，分两种处置——最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判（gap-row）；不是的，用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 `birth(b) > R⁻.txg`。R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时整对报不适用；没有一对判到任何落点时整条报不适用并带理由（同 I-4.9（非最新候选根引用的单元不晚于它） 候选集只剩一条根的写法）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中。⚠️ **射程**：缺口的来源是新实例第一条根的 txg 取根环与 journal 里见过的最大 txg 加一（`crates/singlefs-core/src/mount.rs` 取号那一步），所以写行那次发布的记录已落而根槽没落、多记录发布只落一部分之后再可写挂载，必出现缺口；另有恢复丢掉尾巴记录、根槽写失败重发跳一格（今天的代码走不到，随失败处置在后面的里程碑做）、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）。**盲区**：缺口右边那一对里，落在用户可见树子树里的新写单元（新叶）的诞生代号被改小到 R⁻.txg 以下，判不出；整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小到 R⁻.txg 以下的坏镜像同样判不出：盘上与一次合法回退逐项同形，判别子是回退那次发布写的谱系记录（D18（块里携带什么信息） 已定项 24），`crates/` 今天没有。**谱系记录落地之后**，(b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时该对不适用」，这一形与缺口条款并存（`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 9 行）。**判别力**：诞生代号改小到恰好等于 R⁻.txg 的新写单元必须红；最新根的 inode 树指回已丢的旧叶、分配记录与映射条目一起改平必须红；回退根槽被环盖掉、记录也盖掉之后的合法镜像不许红。**推翻条件**：任一合法镜像（池级 checker 与层 0 其余判定都不红）上红。 | 部分实现：出现规则（TF、β、gap、gap-row、N3）已实现，池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_the_appearance_rule_of_adjacent_candidate_versions`（`IMPLEMENTED_INVARIANTS` 下标 65；坏镜像在 `crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs`；变异在 `crates/mutations.tsv` 里「第8批 I-3.5」那几行）；③ ④（deadlist 那两条）未实现（实现进下一批）。出现规则取按树继承下界加 β 出现规则（用户 2026-10-02 定 ① 取 β）加缺口处置；缺口处置用户 2026-10-03 定 gap 加 N3（判决里的局部标号，只放过用户可见树里的单元）加 gap-row 两样都加。**被攻过几轮**：按树继承下界加 β 加整对不适用的缺口，在合法镜像上两轮没红（各一次抽样）；gap 加 N3 加 gap-row 的合成形、同 txg 几条并成一条、`deleted_inodes` 非全零才算继承、读不出头的单元不计入这三处措辞、谱系记录落地之后的区间形，都是零轮，终轮不再开轮，实现时再攻。实现挂在池级 checker 的主走读上（每版落点集合快照、用户可见树标签、压得住的判、新判定函数、位置键折叠、`IMPLEMENTED_INVARIANTS` 加一条、用例与变异），I-3.1（已分配统计对得上） 在短环上洞里的误红归下一批调查，与这一条共用同一个「候选集有洞」判法（`research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 4、5 行与第四节）。② 的上界半（`birth(b) ≤ R_first.txg`）另立 I-4.9（非最新候选根引用的单元不晚于它）（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行）。⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
| I-3.6 | deadlist 紧界 | 任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）。判别力：抓「kill 时该走直接释放却进了 deadlist」这一类（泄漏方向） | 未实现 |
| I-3.4 | 可用空间扣待删占用 | 待删中 inode 没释放的单元计在记账「已分配」里（释放只进 defer），记账标签 4（待删除但意图未完成占用）恒 0，可用空间统计因此已扣除待删占用；准入式子另扣环窗口常驻项（D28（挂载期承诺量） 已定项 1）。来源 `research/prompts/m3-delete-r3-main-verification.md` 第三节判标签 4 那一格，被攻过零轮 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_pending_deletion_occupancy_of_a_version`；坏镜像两份：标签 4 那一行的值非 0、整行缺失，登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`；GPU 判器下标 70，扫描内核判）。池级 checker 只判「标签 4 恒 0」：判的每一版里，走全了的记账树都要有标签 4 那一行且值为 0，缺行判红；另两句不在这一行判：计在「已分配」里归 I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读），可用空间已扣待删占用归 I-5.2（空闲统计对得上），即空闲加已分配等于容量；判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法；出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节） |
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
| I-4.1 | 重放后树可遍历 | 任一崩溃点重放后，所选根（重放之后那一版）指向的树全部可完整遍历。起点含介质损坏的镜像（前缀停在一条点名单元两份没都验过的记录上）报不适用。另三处报不适用：所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版）；所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布取不到 txg）；所选根之上一次发布都不施加（重放之后那一版就是所选根，可不可遍历归 I-7.2（最新根可完整遍历））——没有记录压在最新根之上的镜像上这一条因此恒报不适用、不报成立。重放用的读环按 (实例代号, 计数器) 建键，两块盘同一计数器坐着不同实例的记录时两条都留，与实现恢复同键（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2、8 行）。 | 已实现（2026-10-02，池级 checker `walk::judge_trees_of_the_replayed_version_are_traversable`：R-独，checker 自己照 D23（journal 的角色与格式） 已定项 14 的前缀规则复刻只读重放（`replay_the_next_mount_would_do`），施加出来的那一版（所选根换上末条记录新根段的七项：树表指针、中央映射树根指针、树 ID 水位、F、实例表指针、分配记录树根、卸载记号，只有 fsid 照所选根；D23（journal 的角色与格式） 已定项 15）另起一份走读，走读断了判红；在飞上限按环长 ÷ 4096 ÷ `JOURNAL_SAFETY_FACTOR` 现算。读环键 2026-10-03 改成 (实例代号, 计数器)（`records_the_replay_reads`），反向链那一关改按 (实例代号, 计数器 − 1) 找前一条（`record_back_chain_lets_it_into_the_replay_prefix`），消掉两块盘同一计数器坐着不同实例的记录时与实现分开的形态（推为第一版两盘池上不可达）；`records_as_the_recovery_reads_them` 与 `record_can_enter_a_replay_prefix` 两处同改按 (实例代号, 计数器) 建键，已做（主 agent 2026-10-03 定）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（重放六条规则各一份、两块盘同计数器不同实例两份）；层 0 里与 core 的 `effective_root` 逐状态差分今天没有能跑的入口，要先开口子，进下一批清单（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8、9 行）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 2 行与第四节用户定案） |
| I-4.2 | 无被引用未提交块 | 任一崩溃点重放后，不存在被引用但未提交的块 | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`：走读到的每个被引用单元，头里的诞生代号与写序（D18（块里携带什么信息） 已定项 7 单元头字段表里的字段）按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判成已发布——**「未提交」取的就是「按那个谓词判不成已发布」**，谓词与 I-1.8（归并后版本全序） 扫描方向共用同一份 `PublishedPredicate`。打包记录类型 4 的实例表单元按 I-1.2（块头写序已发布） 那一行的例外不走谓词，不进这一条。坏镜像是把数据单元头里的诞生代号与引用它的 extent 记录里那条数据指针的诞生代号一起抬到最新根的 txg 之上——指针仍与头一致 ⇒ I-1.2（块头写序已发布） 不跟着红，分得开两条的就是这一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它。⚠️ **「任一崩溃点重放后」那半句没实现**：池级 checker 判的是手上这一份镜像、不跑恢复，判的是「被引用 ⇒ 已发布」这个态上的性质；重放之后再判一遍归层 0） |
| I-4.3 | 提交原子 | 提交是原子的：不存在"部分生效"的事务 | 未实现（它要判的是 D23（journal 的角色与格式） 已定项 7 那两个事务边界字段——事务号与提交标记——施加之后的效果。**对着一份镜像判不出来，没硬写**：「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽——同一份字节既可能是「事务整体没生效的旧态」，也可能是「生效了一半」，池级 checker 手上没有第二个输入。判它的是层 0 崩溃点重放加 oracle：`.claude/kb/milestone/02-second-txn.md` 第 486 / 487 行那两条（崩在第二条记录之前 ⇒ 整个事务不施加、文件是旧长度；把提交标记提前到第一条记录 ⇒ 恢复施加了半个事务、I-4.3（提交原子） 红）说的就是这条路。形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查）：操作断言，不进池级 checker，恒真的检查比没有检查更糟） |
| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 已实现（2026-10-05，在清单里的第一版恒报不适用：池级 checker 在 `walk::check_pool_image` 里对它调 `not_applicable`，理由逐字是 `crates/singlefs-checker/src/image.rs` 的 `INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION`；不要求坏镜像，`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `INVARIANTS_WITHOUT_A_BAD_IMAGE` 点名；GPU 判器下标 73，内核不置位）。第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段，checker 报不适用并写理由；logged_ops 有字段表时实现（用户 2026-10-05 定，岔路单 `research/prompts/m3-delete-checker-r1-forks.md`） |
| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；判据落在 [experiments.md](experiments.md) E3（意图日志的断电语义） 判据③ | 不适用 |
| I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致（候选集的深度由 D16（发布语义） 已定项 1 的回退下界 F 定：平时整个根环，盘紧时缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内）。**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，最新根与候选集里每条更早的根各判一格：从它出发的遍历有单元对不上或读不出就红；坏镜像是 mkfs 树表单元抹成零；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态）。「其余全部不变量仍然成立」由各条在每个被判的版本上各自判，中间态的版本也在内；I-4.6（中间态其余不变量成立） 自己判的是中间态特有的一格：每条 nlink = 0 的记录，数据单元数与 size 对得上——size 为 0 时 0 或 1 个单元，size > 0 时 ⌈size ÷ 数据单元净荷容量⌉（实现 `crates/singlefs-core/src/write_request_split.rs` 第 61 行 `div_ceil(...).max(1)`）。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size`；净荷容量 checker 自己算一份（`data_unit_payload_capacity_in_bytes`）；每条 nlink = 0 的记录判一次，一条都没有时报不适用并带理由；坏镜像：第一个文件的 size 加一个净荷（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `raise_the_size_of_the_first_file_by_one_data_unit_payload`）；GPU 判器下标 72，扫描内核判；用户 2026-10-05 弹窗定「只判单元数对 size」，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第四节与岔路单 `research/prompts/m3-delete-checker-r1-forks.md`；射程只对 nlink = 0 的记录，每条记录都判的那一形不挂在这一行名下，另记欠账 C711（单元数对 size 只判了 nlink = 0 的记录）） |
| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在。「处于删除中的 inode」指两样：inode 树里 nlink = 0 的记录；extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_inodes_being_deleted_against_the_queue_and_their_data_units`；每个处于删除中的号判一次，一个都没有时报不适用并带理由；坏镜像：extent 树里有数据单元、inode 树里没有它的记录、号也不在队列里（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `drop_the_record_of_the_first_file_and_keep_its_data_units`）；GPU 判器下标 71，扫描内核判；对象取这两样，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节逐格判决表里 I-4.7（删除中 inode 有账） 对象那一格） |
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
| I-5.5 | 从 mkfs 水位建树那一版的 t1 落点 | 走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）若是从 mkfs 水位第一次建树的那一版——① V 的树表非空，且种类 1（extent 树）条目的树 ID 等于 65536；② V 的树表里每条条目的诞生 txg（条目偏移 108）都等于 V 的 `checkpoint_txg`——则 V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号等于这一版分配记录里 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、两槽都空、不在开放段里）。新池新建文件那一次是 U + 4，U 是所选系统配置偏移 417 的单元区起始槽号（默认 768 MiB 环 U = 50176，所以是 50180）；回收之后（例如 mkfs 的实例表 U、U + 1 那两槽被回收）不再是 U + 4。V 没有数据单元时不判；不满足 ①② 的版本不判。这一类含新池新建文件，也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件（用户 2026-10-04 定案，原话在决策变更史 D3（空间分配） 节）。只判 t1，t2–t12 不判（D3（空间分配） 已定项 10 射程没有「偏离即不合法」）。已知限度：这一版盘面上复算不出的几样只能近似——已释放的记录一律当占着（回收门槛复算不出：写死从 U + 4 起扫的坏实现与回收门槛放宽的坏实现两个方向都判不出，C689（建树落点判据抓不到写死从 U + 4 起扫的坏实现））；这一版自己分配的记录当空；开放段拿这一版树表单元所在的段当它（这次挂载更早开过、已经用尽的段认不出；提交内生块回落时会误排，今天的准入下够不着，C687（从 mkfs 水位建树那一版落点判不了的几格））；隔离位、抬 F 扣住位与保护期内的 R 盘上都没有，没有分配记录罩着的槽一律当空。依据 D3（空间分配） 已定项 8 射程与已定项 10 ③ ④，以及判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2 与用户 2026-10-04 对判据改法的定案（原话在决策变更史 D3（空间分配） 节）。 | 已实现（判据是 f2（按这一版分配记录现算），代码轮判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md`；池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark`：候选集里每条根与由记录施加出来的那几版各自重读树表与 extent 树判，U 取所选系统配置偏移 417；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_first_data_unit_of_a_version`，`IMPLEMENTED_INVARIANTS` 下标 67；这一版的树表读不出或条目宽不到 200 字节、种类 1 条目不止一条、extent 树读不全、有数据单元而池里某块盘上一个都没有、种类 3（分配记录树）条目不是恰好一条、分配记录树读不全，这几格报射程里判不了（没有违例时整条报不适用），该判红还是不适用欠在 C687（从 mkfs 水位建树那一版落点判不了的几格）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（数据单元从 50180 搬到 50182），合法镜像在 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`） |

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

**出处 `.claude/kb/invariants.md:278-319`（整段抄，未转述）**

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
| I-9.5 | 删除队列里的号仍在树里 | 两个方向都要成立。① `deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0（记录在回收完成前留在 inode 树里，`deleted_inodes` 只存号）；② inode 树里 nlink = 0 的每条记录，号都在同一棵树的 `deleted_inodes` 里（队列是派生态，队列 = inode 树里 nlink = 0 的号集合，D8（核心索引结构） 已定项 16 第 3 条）。**共用判法**（I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-3.4（可用空间扣待删占用） 都照这一段，不另写）：判走读到的任一版（候选集里每条根，含由记录施加出来的那几版），不只判最新根；`deleted_inodes` 与 inode 树经树表条目偏移 116 的头 ID 配对（头 ID 的取值见 I-9.27（树表条目的头 ID 取值）），配不上的版记判不了；树表、`deleted_inodes` 或 inode 树读不全的版报 `part_of_the_range_not_judged`（射程里判不了，不判红），这是今天除 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 外各条用的报法，是否改报第四种判定「部分没判」随 C701（别的判法静默跳过读不出的部分） 那一问由用户答，答了这几条随它一起改。树表指针全零、树表 0 条的版没有这几棵树，不判；树表单元相同的几版内容相同，只扫一次；没有删除的版本上 I-9.5（删除队列里的号仍在树里） 报成立（两个方向比的是队列与整棵 inode 树，干净的一版也真比过），I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立） 没有对象时报不适用并带理由 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，删除态扫描是 `judge_deletion_state_of_the_versions`；坏镜像登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，搭建与改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`，逐形的用例在 `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs`；GPU 判器下标 69，扫描内核判）。两个方向各有坏镜像；C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） 还差「队列里的号在树里 nlink ≠ 0」那一形的坏镜像；`deleted_inodes` 的形态已定在 D8（核心索引结构） 已定项 16；判法与读法的出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 |
| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部已发布的（按实例表判）墓碑记录的对象 ID（墓碑记录住墓碑树，D18（块里携带什么信息） 已定项 24；被崩溃恢复抛弃的时间线上写的墓碑不在所选根的墓碑树里）——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**两半都判**：水位 > 树内最大 key；墓碑那一半（2026-10-05 起）水位 > 所选根（最新的那条根）墓碑树里已发布的（按所选根指着的实例表判）墓碑容器里全部记录的对象 ID 最大值，函数 `largest_published_tombstone_object_identifier`，一条已发布的墓碑都没有时没有对象，墓碑树读不全报射程里判不了；坏镜像：墓碑记录的对象 ID 抬到水位（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `raise_the_tombstone_object_identifier_to_the_inode_number_watermark`）；GPU 判器内核一并判，并进下标 41 那一位 |
| I-9.7 | 记录三段恒零 | 打包记录类型 2 的记录偏移 108 / 112 / 120 三段（填充 4 / flags 8 / 预留 20）恒 0；非 0 判该记录 EIO，容器其余记录照读（前向兼容政策，D15（格式冻结政策） 判定表；不是损坏检出，载荷校验和照样过） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.8 | 记录 inode 等于 key | 打包记录类型 2 的记录偏移 0 的 inode 号 == 它在 inode 树里的 key；不等判红、该记录 EIO，不许静默修复 | 未实现，前置：树的种类 |
| I-9.9 | locality 三值对照 | 记录的 `locality_id` == 该对象每条 extent key 的首段；extent 树未经重建时输出红 / 绿；系统配置 `map_provenance` 标 rebuilt 时两侧同源，输出「不可判定」，不许输出「通过」；按侧判（2026-09-16）：记录那一侧是从 key 编出来的、或 key 那一侧是从记录编出来的，同样输出「不可判定」；分批重新聚簇的中间态怎么判随 C346（后台重新聚簇没有定义） 定 | 未实现 |
| I-9.10 | 对象出生代三处一致 | 记录偏移 8 的对象出生代 == 该对象每个数据单元头五元组里的对象出生代 == 该对象的墓碑记录里的对象出生代（若有；墓碑记录住墓碑树，D18（块里携带什么信息） 已定项 24；只对 inode 记录还在的对象判，删完之后记录不在，这一个等号无对象）；重建后两侧同源，输出「不可判定」，与 I-9.9（locality 三值对照） 同形 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；墓碑那一个等号未实现，三·3 实现） |
| I-9.11 | 可写克隆头有水位行 | 每个可写克隆头的树 ID 在记账里都有「inode 号水位」行（树表可枚举、记账可查）；「初值 ≥ origin 那一刻的水位」是建克隆那一刻的操作断言，转模型对拍——它的右操作数（建克隆那一刻的水位行）只住写出它的那条根自己的记账树里（D5（快照 / 空间记账机制） 已定项 2：每条根的记账树只装这条根那一代的行），对着镜像读不到 | 未实现，前置：树的种类 |
| I-9.12 | 分隔 key 落在孩子区间之外 | inode 树内部节点的条目按分隔 key 严格递增；第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界。判的字段是内部节点条目头 8 字节那个分隔 key（D8（核心索引结构） 已定项 6：条目 120 字节 = 分隔 key 8 + 身份引用 26 + 子指针 86，「分隔 key = 该孩子创建时的最小 key，条目按分隔 key 单调递增」）；孩子的 key 区间对码 3 容器由它装的记录现算，容器头不带 key 区间 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.13 | 类型 2 容器不空 | 任一打包记录类型 2 的容器记录数 ≥ 1（空叶在重建时没有 key 范围；唯一的叶被删空时不落盘、根变成零条目） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.14 | 树表条目的诞生 txg 跨根不变 | 同一棵树（按树 ID）的树表条目，在**同一条时间线上**的任意两条有效根各自的树表里，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段）；C374（释放代与树表诞生 txg 只有验收断言盯着））。「同一条时间线上的有效根」就是回退候选集（D23（journal 的角色与格式） 已定项 14 + D16（发布语义） 已定项 1）：最新那条根本身；以及根环里 (i, T) 满足两条的根——按**最新那条根指着的实例表**，没有实例 i 的行、或有行 (i, Ti, Wi) 且 T ≤ Ti；并且 T ≥ 最新根带的回退下界 F。有行 (i, Ti, Wi) 且 T > Ti 的根在被崩溃恢复切掉的旧线上，**不与现行线上的根比**（C511（回退到无文件那一版之后诞生代怎么接） 2026-09-23 用户定案收窄射程）。候选集不足两条根，或同一条时间线上没有一棵树的条目出现在两个不同的树表单元里时，报「不适用」，不报成立——几条根指着同一个树表单元、拿同一份字节比自己是恒真判定。判别力：① 把树表条目的诞生 txg 改成跟着这次发布的 txg 走，必须红；② 管理员回退前后同一条线上记得不同，必须红（回退是向前发布，前后是同一条线）。跨线（含被崩溃恢复抛弃的旧线）的诞生 txg 一致由 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 判，这一条不再有「只有被切掉的那条线记得不同，不许红」那一格（用户 2026-10-04 定）。推翻条件：找到一条历史，最新根的实例表把某条根判成有效，而它不是最新根的祖先（不在同一条线上）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判。2026-09-23 收窄之后同文件两条：判别力 ② 的坏镜像 `birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant`（挂着时回退到 A 之后，回退那次发布 D 的树表把 extent 树（树 65536）的诞生 txg 记成 D 的 txg 9、A 那一版记 3：A 与 D 在同一条线上，只有 I-9.14（树表条目的诞生 txg 跨根不变） 红），原判别力 ③ 的正向用例 `birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_is_not_compared_and_every_invariant_holds`（被切掉的线由崩溃恢复造：C 被抛弃之后只改 C 那片树表里的诞生 txg）要改：这张镜像上 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 会红，不能再断言每一条都成立；欠账在里程碑三开工第一轮的第 9 批（`records/2026-09-30-里程碑三开工第一轮.md`，池级 checker 簇），由实现员改它，`judge_tree_table_birth_txg` 的注释里说明判别力 ③ 的那一段跟着改；判定在 `walk::judge_tree_table_birth_txg`） |
| I-9.15 | inode 记录的 blocks 等于 ⌈size ÷ 512⌉ | 打包记录类型 2 的每条记录，偏移 48 的 blocks == ⌈偏移 40 的 size ÷ 512⌉（D8（核心索引结构） 已定项 6：blocks 是逻辑长度的 512 字节块数，不表示分到的空间；C480（inode 记录的 blocks 怎么算全仓没有条款） 用户 2026-09-23 定）；不等判红。读者要不要也判、判了报什么没有条款 | 已实现（2026-09-24，池级 checker `walk::check_pool_image` 走 inode 叶容器时逐条判，与 I-9.7（记录三段恒零） 同一处；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：blocks 写成 64 / 5 / 7 各只红它；层 0 每个走得到 inode 叶的崩溃状态都判；变异：判定恒真、按向下取整比、写路径写回 64；里程碑「覆盖写、释放、回退与复用」收口表第 46 行） |
| I-9.16 | 树表条目按树 ID 严格升序且合发号次序 | 同一张树表（D8（核心索引结构） 已定项 8 的字段表）里，① 盘上相邻两条的树 ID 严格升序；② 树表里有的种类按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops；中央映射树不进树表）排，相邻两棵的树 ID 严格升序。两道任一道不成立即违例。依据 D8（核心索引结构） 已定项 8「排序契约」与它的 ②「十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547」。判别力落在 ②：① 与 checker 在树表上判的 I-1.1（块头自述逻辑地址） 那组 key 严格递增是同一个谓词，只违反 ① 的镜像两条一起红 | 已实现（2026-09-27，core `crates/singlefs-core/src/recovery.rs` 常量 `TREE_TABLE_ENTRIES_ORDERING_CONTRACT`，两道合成一个入口，`rebuild_version` 与 `walk_to_file` 在读任何一棵树之前拒；池级 checker `crates/singlefs-checker/src/walk.rs` 另写一份、违例写明哪一版根与哪两条，列进 `image.rs` 的 `IMPLEMENTED_INVARIANTS`；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`「互换两条的种类」那份只红 I-9.16（树表条目按树 ID 严格升序且合发号次序）；只违反 ① 的钉在 `tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`；实现员报告 `research/prompts/m2-rev-tree-table-ordering-implementer-report.md`） |
| I-9.17 | 谱系树条目身份与子头相符 | 谱系树（D18（块里携带什么信息） 已定项 24）的每条条目：类型段 ∈ {0, 5}；类型段 0 ⇒ 子单元是码 2（谱系树根满长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；类型段 5 ⇒ 子单元偏移 6 = 3，子头的 (出生树, 打包记录类型, 容器号, 容器出生代) 与条目的身份引用四段逐字相等，子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 5；其余判该条目损坏。与 I-9.2（条目身份与子头相符） 同形 | 未实现（三·3 实现） |
| I-9.18 | 删除队列里的号有谱系上的墓碑 | `deleted_inodes`（D8（核心索引结构） 已定项 16）里的每个号 N，墓碑树（D18（块里携带什么信息） 已定项 24）里都有一条在谱系上的墓碑记录 (N, N 的 inode 记录偏移 8 的对象出生代, [0, 2⁶⁴ − 1])。「在谱系上」：墓碑记录的死亡 txg 落在挂载根的父链上——从挂载根的 txg 起往回走，有谱系记录 (T_rb, T_old) 的那个 txg（T_rb）的父是 T_old，其余 txg 的父是 txg − 1（谱系记录住谱系树，D18（块里携带什么信息） 已定项 24；txg − 1 那一步核心层没有显式断言，C616（谱系父链靠 txg 连号没有断言））。不按谱系过滤的写法放过改坏（`research/prompts/m3-delete-r3-main-verification.md` 第三节判新不变量写法那一格） | 未实现（今天没有实现；实现时对「队列里的号没有记录」那一形报判不了而不是红，I-9.5（删除队列里的号仍在树里） 的这一形坏镜像才只红自己，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节最后一行；第 18 批没做它） |
| I-9.19 | 谱系上的墓碑不指活对象 | 墓碑树里每条在谱系上的墓碑记录（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）指认的 (号, 对象出生代)，inode 树里有号与对象出生代都相同的记录时，那条记录的 nlink = 0 | 未实现（三·3 实现） |
| I-9.20 | 谱系上的墓碑不重号且记录合法 | 墓碑树里在谱系上的墓碑记录（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）按 (号, 对象出生代) 不重复；每条记录区间起 < 区间止，第一版只许整对象 [0, 2⁶⁴ − 1]；每个打包记录类型 1 容器的记录数 ≤ 815（⌊(32768 − 136) ÷ 40⌋）。不按谱系过滤的写法在「删 → 回退 → 再删」的合法历史上判红（`research/prompts/m3-delete-r3-main-verification.md` 第三节判新不变量写法那一格） | 未实现（三·3 实现） |
| I-9.21 | 产品镜像里非最右墓碑容器装满 | 没有用只供测试的容量旋钮的镜像上，墓碑树里容器号沿叶序严格递增（根长高之后根的条目是类型段 0、容器号恒 0，见 I-9.23（墓碑树条目身份与子头相符），所以不沿根的条目读），除最右那一个之外的每个打包记录类型 1 容器记录数 == 815（开放容器满了才关，D18（块里携带什么信息） 已定项 24） | 未实现（三·3 实现） |
| I-9.22 | 测试镜像里墓碑容器不超旋钮值 | 只供测试的容量旋钮压小了一个墓碑容器装的记录数的镜像上，墓碑树里容器号沿叶序严格递增，每个打包记录类型 1 容器记录数 ≤ 旋钮值；旋钮不落盘，checker 从镜像外拿旋钮值。按产品容量判这类镜像会在合法状态上红（`research/prompts/m3-delete-r3-main-verification.md` 第三节判容量旋钮那一格） | 未实现（三·3 实现） |
| I-9.23 | 墓碑树条目身份与子头相符 | 墓碑树（D18（块里携带什么信息） 已定项 24）每个内部条目的类型段 ∈ {0, 1}：0 ⇒ 子单元是码 2（根长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；1 ⇒ 子单元是码 3，子头 (出生树, 打包记录类型, 容器号, 容器出生代) 与身份引用四段逐字相等、子指针头部出生树 == 身份引用出生树、子容器打包记录类型 == 1。与 I-9.2（条目身份与子头相符） 同形（墓碑正推报告 `research/prompts/m3-tomb-r1-sonnet-output.md` 第 241 行 N1；根长高之后类型段 0 那一支是主 agent 按 K-R3-3 补的，被攻过零轮） | 未实现（三·3 实现） |
| I-9.24 | 删除队列至多一条 key | 任何一版里 `deleted_inodes`（D8（核心索引结构） 已定项 16）至多一条 key；每个被判的版本各判一次，不遍历版本之间的历史。依据：会话里每一次用户改动先把现行那一版的队列走空再做自己（D8（核心索引结构） 已定项 16 第 6 条的增补，用户定案），判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节「B4 + B8 队列上界与接口」一行 | 未实现（三·3 实现） |
| I-9.25 | 树表条目的 previous_snapshot_txg 取值 | 走读到的任一根（候选集里每条，含由记录施加出来的那几版）的树表里：树的种类不是 inode 树（种类码 ≠ 2）的条目，偏移 100 的 `previous_snapshot_txg` 恒 0；inode 树的条目为 0，或 1 ≤ p ≤ 这条根的 `checkpoint_txg`。依据 D5（快照 / 空间记账机制） 已定项 9（`previous_snapshot_txg` 只写在该头 inode 树那条树表条目里，其余条目写 0 且 0 = 不适用）；inode 树条目「≤ 根 txg」那一半的条款是用户补定的（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节用户定案第 2 条）。 | 已实现（2026-10-02，池级 checker `walk_tree_table_entry` 每走到一条树表条目判一次，每个走到的版本（含记录施加出来的）都走它；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，inode 树条目的 previous_snapshot_txg 压过根 txg 那一份由 2026-10-03 补；⚠️ 树表单元被几版共享时只在第一个走到它的那一版判（`visited_units`），共享它的别的候选根漏判，与条款字面「候选集里每条」不符，代码注释自认判得更松，代码改回 kb 字面，实现进下一批（主 agent 2026-10-03 定；`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2 行，推的，没造镜像）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |
| I-9.26 | inode 改动计数落在出生代与容器诞生代号之间 | 走读到的任一 inode 叶容器（码 3，打包记录类型 2）里每条记录：对象出生代（记录偏移 8）≤ 改动计数（记录偏移 88）≤ 这片容器头的诞生代号（偏移 73）。上界不是等于：容器因别的记录改动而重写时，没改的那条记录的改动计数小于容器诞生代号是合法的；下界要「创建算一次改动」，这一句条款是用户补定的。依据 D8（核心索引结构） 已定项 6 偏移 88 那一行（改动计数 = 最后一次改动所在发布的 `checkpoint_txg`）与 D18（块里携带什么信息） 已定项 9 之后的容器重写规则。 | 已实现（2026-10-02，池级 checker `walk::walk_inode_leaf_container`：每条记录判偏移 8、偏移 88 与容器头里偏移 73 的诞生代号（不是偏移 61 的容器出生代）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，下界一份（改动计数 = 对象出生代 − 1）由 2026-10-03 补；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行与第四节用户定案第 2 条） |
| I-9.27 | 树表条目的头 ID 取值 | 走读到的任一版（候选集里每条根，含由记录施加出来的那几版）的树表有条目时，逐条判条目偏移 116 的头 ID：种类码 2（inode 树）的条目等于它自己的树 ID（条目偏移 0）；种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）的条目等于这张树表里种类 2 条目的树 ID，表里有多于一条种类 2 条目时放宽成「等于其中某一条的树 ID」（收严留给多头那一批），表里没有种类 2 条目时这三种判红；种类 3、4、6、7、8、11、12 的条目等于 0；别的种类码不判。树表 0 条的版本一条也不判；条目宽不到 124 字节（头 ID 那 8 字节读不全）的树表整张不判。不拿字面 65537 比：它只是树 ID 从 mkfs 水位 65536 起发的那一版上的取值。依据 D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」与 D5（快照 / 空间记账机制） 已定项 9。 | 已实现（2026-10-04，池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_tree_table_entry_head_identifiers`，挨着 I-9.16（树表条目按树 ID 严格升序且合发号次序） 每张树表判一次、按条目计数，共享这张树表的别的版本不重判；条目宽不到 124 字节时这张树表报射程里判不了；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_tree_table_entry_head_identifiers`，`IMPLEMENTED_INVARIANTS` 下标 66；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（extent 树条目头 ID 写 0，只红它），从根环水位发树的合法历史在 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`） |
| I-9.28 | 全部可读根里同一树 ID 的诞生 txg 相同 | 根环里全部读得出且自证合法的根（不按实例表判有效与否，含被崩溃恢复抛弃的旧线上的根），任两条根各自的树表里同一树 ID（条目偏移 0）的条目，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段））。判定四种：有一处诞生 txg 对不上 → 不成立；各处都对得上，但有一条读得出、自证过的根树表指针非零而树表读不出、或条目比字段表短 → 部分没判，列出这些根（盘号、实例、txg、为什么没判）；没有一棵树出现在两个不同的树表单元里（几条根共用一个树表单元时拿同一份字节比自己，是恒真判定）→ 不适用；其余 → 成立。树表指针全零的根没有树表，不算没判到。与 I-9.14（树表条目的诞生 txg 跨根不变） 的分工：I-9.14（树表条目的诞生 txg 跨根不变） 只比同一条时间线上的根，这一条不分时间线、比全部可读根。依据：树 ID 被重发（C342（树 ID 水位在根读不出时退回去重发））时，新线上的树与老线上同一个 ID 的树诞生 txg 不同，I-9.14（树表条目的诞生 txg 跨根不变） 的同线收窄看不见这一形。推翻条件：任一合法历史（池级 checker 与层 0 其余判定都不红）上红。 | 已实现（2026-10-04，欠账来源 C342（树 ID 水位在根读不出时退回去重发）；池级 checker `walk::check_pool_image` 的 `judge_tree_identifier_birth_txg_across_every_readable_root`，四种结局；GPU 判器的扫描内核挨着 I-9.14（树表条目的诞生 txg 跨根不变） 判、只置违例位；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，合法历史、部分没判、不适用、对不上照报违例的用例在 `crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs`；`crates/mutations.tsv` 里以「第9批」「第9批改钉」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）或第四种判定的各行，与以「第 9 批 9-9 GPU 判器」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）的 3 行）。用户 2026-10-04 定新立，同时删 I-9.14（树表条目的诞生 txg 跨根不变） 判别力第 ③ 条与它的正向用例，判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第六节 S2-1 行。被攻过零轮：攻方三段合法历史上 0 红（一次抽样），被切掉旧线的镜像上红。部分没判：用户 2026-10-04 定「树表读不出算部分没判」、加第四种判定；四种结局的细则主 agent 定，零轮，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批「用户 2026-10-04 弹窗定（9-6 交回的两问）」那一条 |

**不进 I-9（inode 树结构） 的**：分裂 / 合并的身份传递（左半身份不变、右半号 = 触发分裂的新记录号、出生树 = 执行分裂的树）与合并时机是操作断言，对着一个镜像判不了，走单测 / 模型对拍与崩溃点重放（C116（叶容器的分裂 / 合并纪律没有会失败的检查））；
nlink == dirent 条数的门（dirent 树存在）今天没有输入，写不成可判定形式，随 dirent 树上线时一起立。

```

**出处 `.claude/kb/decisions/16-发布语义.md:1-4`（整段抄，未转述）**

```markdown
## D16 发布语义 —— 已定

D16（发布语义） 管发布这一格：一个新根什么时候发、一次发布按什么顺序落盘、发布计数器是哪一个数、这一批释放什么时候回可分配集合、新实例的第一个根怎么先覆盖两块盘、一次空发布写不写单元、journal 记录格式在 D15（格式冻结政策） 的冻结分层里算什么。不管的：journal 的记录字段、环几何与重放前缀在 D23（journal 的角色与格式）；根环参数、根记录与系统配置的字段表在 D22（单元原子性怎么合成）；快照的记账模型与生死边界表在 D5（快照 / 空间记账机制） 与 D6（快照实现模型）；准入不等式与挂载期承诺量在 D3（空间分配） 与 D28（挂载期承诺量）；一次发布事件内部的内存序在 `litmus/commit-publish.litmus`，那份钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。

```

**出处 `.claude/kb/decisions/16-发布语义.md:27-83`（整段抄，未转述）**

```markdown
#### 已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读

**定案**：回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算；同一个状态只数一次（去重的判法在「「非空」从盘上怎么认」那一段）。**根环容量是这句的边界**：两个状态来回交替超过根环一圈时，环里的根全是这两个状态，保留的就只有环里有的那几个，任何机制都给不出环里没有的状态。**窗口的单位是发布，不是秒**：defer 窗口保护的是「环里的根还能不能用」，普查过的消费者没有一个按秒计价。形态取 E139（按盘回退下界的收严形态） 的臂 G2S；回收门槛退回环里最旧有效根之后，装置的候选下界还没带系统配置里的 F，这一句仍不与装置里跑的一字不差，欠里记着 E139（按盘回退下界的收严形态） 要重跑：

| 项 | 定案 |
|---|---|
| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |
| 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算。**同取这个值的六处**（用户 2026-10-04 定，被攻过一轮）：分配器的回收门槛（含抬 F 那一遍）、checker 的候选过滤、checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛（max(F_生效, 环里最旧有效根)）、影子账的候选过滤、回退候选集、抬 F 的上限（含判「非空」那一遍）；六处都不再按一圈窗口滤根，窗口之外的有效根（崩在记录之后、根槽之前那一形留在槽里的旧根）也在候选集里、也压着门槛。**挂载时读不出的槽**：N = 环里读得出的最新有效根的 txg（挂载重建分配器时取它、不取 first_txg − 1；进程内每记一条根时取这条根）；读不出的槽在 txg > N − 3S（一圈，3S 是环长）里只对应一两个 txg：门槛按最小的算，根环表记最大的（零轮，只在攻方模型上量过），抬 F 那一遍里根环表挂载时读不出的那几项同样按这一句并进最旧有效根；一圈窗口这个量现在只剩这一处在用，给读不出的槽划出它能住的 txg 的范围。进程内记根时，根环表里这一项成了最小的那个，先重读那一槽：读得出、自证过、按实例表仍然有效的，照读出来的根算；读得出而自证不过的，与仍读不出同一处置，按记的最大 txg 算；读得出、自证过而按实例表被抛弃的，当没有根（用户 2026-10-03 定先重读，2026-10-04 定自证不过那一格取按记的最大 txg 算，零轮）；重读读出有效根的那一槽，算进「这个进程知道住着根的槽」（「根槽这一次读坏」那一行）。 |
| 回退下界 F | 根记录带 8 字节（D22（单元原子性怎么合成） 已定项 7，242 → 250），系统配置另记一份 8 字节（D22（单元原子性怎么合成） 已定项 9）。平时不动，窗口就是整个根环；准入不够时抬，正常卸载时抬到现行那一版的 txg |
| 抬 F 的上限 | **准入抬 F**：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根；「有效根」「非空」在环里全部的根上取；挂载那一刻读不出或自证不过、之后这个进程没写过的槽，当作可能是一条非空有效根，F 不越过它（用户 2026-10-04 定，零轮：先定只并读不出的槽，代码轮第三轮之后重定为读不出或自证不过两类都并；与「环里最旧有效根」那一行把读不出的槽并进回收门槛同一个思路）。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)。**卸载抬 F**：抬到现行那一版的 txg，不另判上限（等价于上限取最新的持久有效根，不按盘取小） |
| 根槽这一次读坏 | 算准入抬 F 的上限、挂着时管理员回退判候选（根环里有没有目标）与算 F_生效 时读根环：挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，算准入抬 F 的上限时当作可能是一条非空有效根、F 不越过它（「抬 F 的上限」那一行，用户 2026-10-04 先定只并读不出的槽、代码轮第三轮之后重定为读不出或自证不过都并，零轮），判回退候选与算 F_生效 时当没有根；挂载那一刻读得出、这个进程写过且 FUA 返回过、或进程内记根时重读读出了有效根的槽（这三类是「这个进程知道住着根的槽」），这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退、报它自己的错误成员，不按有根或没根猜（抬 F 那一半用户 2026-09-26 定，推的，被攻过零轮；回退那一半主 agent 2026-09-28 定，被攻过一轮：代码三方第三轮判决 `research/prompts/m2-closeout-code-r3-main-verification.md` 第一节判替没写的条款做了选择、回退扩写拒错或放错那一格没打中（一次抽样））。可写挂载的择根按 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续处置（D23（journal 的角色与格式） 已定项 14：系统配置见证过比所选那一版新的发布就把读阶段重读一次，仍判真拒可写）；只读挂载的择根、「环里最旧有效根」怎么处置这一格没判，见 C563（读根环一槽读坏就当没有根，别的调用方没判） |
| 生效 | F 另记进每块盘的系统配置（「抬 F 那一串」那一行写明先写后抬）；恢复后生效值 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 被抛弃时间线上的根所带 F 的最大值, 系统配置里读得出的 F 的最大值)；系统配置里读得出的取每块幸存盘两槽里全部自证过（校验和过且 fsid 与本池相同）的槽，与取号式子同一取法（D18（块里携带什么信息） 已定项 11） |
| 回退候选集 | 按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的挂着时回退取同一个集合） |
| 抬 F 那一串 | 准入与卸载共用。动盘之前在分配器的拷贝上把这一串空发布整串预演：任一次取不到落点，就一次都不发、分配器原样换回抬 F 之前那一份（`MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`）；**预演过了、第一次发布之前，先把新 F 写进每块盘的系统配置、过一道屏障**，之后才发第一条带新 F 的根；**先写那一步任何一块盘失败，这一串的根一条都不发、返回错误；已写进别的盘系统配置的新 F 留着、不回卷，F 的生效值只增不减**（主 agent 2026-09-26 定，推的，被攻过零轮）；空发布推到每块盘上都有一条带新 F 的根为止；这一串回收出来的槽在生效（每块盘上都有带新 F 的根）之前扣住、不发出去，生效之后放开（正常卸载那一格例外，见「正常卸载」那一行）；真发时落盘途中写错，交回已落盘几次（`MountError::RaiseFloorSequencePublishFailed`），那一格里扣住的槽怎么放没有条款，见 C546（抬 F 被拒时扣住的槽不退回） |
| 正常卸载 | 推一串空发布把 F 抬到现行那一版的 txg（「抬 F 那一串」照准入的走法，上限见「抬 F 的上限」那一行）；这一串回收出来的槽，只有释放代不高于「每块盘上最新有效根的 txg」最小者的那一截当场可用（预演与真发都算上），其余与准入那一串一样扣到生效（恢复择根不看 F，丢一块盘时择的是剩下那块盘上最新的根，引用它的单元不能被这一串写掉）；有一块盘一条有效根都读不出时这一截为空、整截扣到生效，这一遍读不出、自证不过的槽当没有根；卸载半途崩溃（系统配置已带新 F、带新 F 的根没落满每块盘）之后的下一次可写挂载同样只放开一截：重建分配器时当场可用的只到释放代不高于 max(min(F_生效, 「每块盘上最新有效根的 txg」最小者), 环里最旧有效根) 的槽，某块盘一条有效根都读不出时只到环里最旧有效根（挂载时读不出的槽按它能住的最小 txg 并进这个最小值，与回收门槛同一个读数），其余扣到这次挂载的暖机在每块盘上都落了根（用户 2026-10-04 定，零轮）；这一串写的根在根记录 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）；第一版几何（R = 3、区域归属 0 / 1 / 0）下空发布 2 或 3 次（现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次）；不加干净关闭标记，下次挂载照旧走恢复（D23（journal 的角色与格式） 已定项 14）；现行那一版树表 0 条（mkfs 之后只暖机、还没写过第一个文件）时一个字节都不写、F 不动、不推空发布，卸载照常做成（`crates/singlefs-core/src/mount.rs` 的 `Unmounted::NothingWrittenOnAVersionWithoutFile`） |
| 准入 | 可分配 = min(可再分配 + 活元数据 − 保留池, `df`)；准入不够、或准入放行而落点取不到时，先推空发布抬 F 再判（写行那次发布之前不推：它是新实例的第一次发布，元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC）；**可写挂载的准入不够时，写行那次发布照走切换预留，写完行之后推抬 F 的空发布、再判**；一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC；挂载处推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常（D2（RAID 条带策略） 已定项 13 那一格的例外）（用户 2026-09-26 定，被攻过零轮）；挂载续做的删除类发布被落点那一判以空间拒时同样推抬 F 再试，推满仍放不下挂载照样做成可写（D8（核心索引结构） 已定项 16 第 6 条，用户 2026-10-03 定） |
| `df` | 没用过的 + 已释放的 + 活元数据 − 保留池 − 推空最坏残留 − 滞后量；保留池 = 2 × 5 + (B − 1) × c_max = 10 + 7 c_max，残留 = 5 + (B − 2) × c_max = 5 + 6 c_max；滞后量 = 释放代 > max(第 4 新的非空有效根, 环里最旧有效根) 且不是空发布放掉的已释放块 |
| 环深下限 | 每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随之抬） |

**「非空」从盘上怎么认**：一条有效根算非空 ⟺ 它的树表里用户可见的树（inode 树、extent 树）的根指针，与它前一条有效根（按 txg 排；有效 = 按实例表判仍然有效 ∧ txg ≥ 当前的 F）树表里的不同。比的是树表里这两棵树的条目、不是树表单元自己的落点——每一次发布（连空发布）都重写树表单元（`crates/singlefs-core/src/transaction.rs`），按落点比会把空发布全算成非空。**数「第 N 新的非空有效根」与「4 个不同状态」时去重**：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入——管理员回退发出的新根与它前一条有效根不同、按「「非空」从盘上怎么认」那一判法算非空，而它的状态与更早的某一条相同，不去重就会把同一个状态数两次（第二轮 X7：来回回退之后按内容只剩 2 个不同状态）。反方向只让 F 停住，直到第 4 个状态的根离开根环。

**改新池新建文件的字节：是**：根记录加 F 8 字节；新池新建文件里 F 恒 0（mkfs 与第一次发布时非空有效根不足 4 个，上限取最旧有效根 txg 0）。系统配置加 F 8 字节，新池新建文件里写 0（D22（单元原子性怎么合成） 已定项 9）。

**连带改的**：D22（单元原子性怎么合成） 已定项 7 的根记录加 F（242 → 250）、flags 占一位卸载记号，已定项 9 的系统配置加 F 8 字节，已定项 2 的 S 下界；D23（journal 的角色与格式） 已定项 14 的回退候选集加「txg ≥ F_生效」；D3（空间分配） 已定项 9 第 2 条写明界；I-7.4（近 K 代块未被复用） 的 K 改由 F 表达；I-7.9（回退下界 F 不高于抬 F 的上限） 按卸载记号分两支。

**射程**：定的是 defer 窗口与回退下界怎么算，不定环几何、不定 c_max 的口径（c_max 是声明的最坏空发布开销，按 D28（挂载期承诺量） 已定项 4 按当时结构现算）。已知边角八样：

- **「生效」取 SysPre**：F 只住根记录、根槽不镜像（已定项 7）时，带新 F 的根坏掉 F_生效 就回落，更早的根重新进回退候选集、而它们引用的块已被合法复用——I-7.4（近 K 代块未被复用） 在这段历史上不成立（`research/prompts/m2-step45-code-r2-main-verification.md` 第四节、`research/prompts/m2-step6-checker-r1-main-verification.md` 第四节）；病根是 F 只住根记录，与生效取哪条规则无关（第三轮 K4：带 F 的根全坏那一格，今天的规则、MAX、MAX+HOLD 候选坏 12 / 140 / 12，SysPre 0）。代价：系统配置多 8 字节（动格式）；每次抬 F（准入与卸载）每块盘多一次系统配置写、加一道屏障；checker 与层 0 要跟着读系统配置里的 F——今天的 checker 按根上带的 F 判，带 F 的根全坏时会误红 I-7.4（近 K 代块未被复用） 等五条（第三轮 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。「先写后抬」是第三轮攻方提的，被攻过零轮，用户 2026-09-26 知情定。被抛弃时间线上的根带的 F 算进 F_生效（用户 2026-09-28 定「算进」，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M2：被抛弃根带着比别处高的 F 时，不算的话 I-7.12（系统配置 F 不低于同盘根上的 F） 红、回退退得到那个 F 之下，而被抛弃时间线抬 F 时已按它回收过槽；实现 `crates/singlefs-core/src/recovery.rs` 的 `effective_rollback_floor_of_the_roots_read`，checker 另算一份 `crates/singlefs-checker/src/walk.rs`）；C331（择根倒挂压过已确认的写） 取甲之后，被抛弃根带着更高 F 的盘面只在构造的镜像上有（那一形要可写挂载时一槽系统配置一时读不出，甲 拒这样的可写挂载）。与 `research/prompts/alloc-basis-r3-main-verification.md` 的 F 口径岔路一起判的那一件仍开着。
- F 回落之后第 0 代根重新进有效根、它的树表单元已被合法复用、此后每一次抬 F 都被拒到根环把它盖掉为止（`research/prompts/m2-emptypool-nonempty-r1-main-verification.md` 第一节 Z1）：前提是 F 回落，取 SysPre 之后在带 F 的根全坏那一格上不再出现（推的，第三轮没造那一形）。一次挂载里连续发布超过根环容量时挂载内不回收，checker 在合法状态上判 I-3.1（已分配统计对得上） 红（E153（账本形态与环上有洞的代价） 在 `crates/` 副本上量到），仍开着，归里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ② 行。
- **正常卸载的代价**：每次卸载多 2 或 3 次空发布（树表 0 条的一版上不推，见定案那一句），第一版几何上每次 8 个单元、两盘合计 21 次写调用、279040 字节（第一轮攻方 t10、t5a，副本上量的）；卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版。用户选时知情的代价是「停机时崩溃兜底只剩最新一条根」（按条款推的）；第二轮在卸载之后再挂、最新 j 条根坏的 75 行上恢复全读对、可写挂载全挂上，这一条没出现：F 之下的根还在环里，它们的记录能把状态重放回这一串的末尾（第二轮判决 G4）。
- **用户知情接受的代价三条**：删掉的空间要再发生 3 次改变用户可见状态的发布才回可分配集合，在那之前 `df` 不把它算空闲、按 `df` 写它是真 ENOSPC；`df` 少报 = 保留池 + 残留 = 15 + 13 c_max（c_max = 5 / 10 时 80 / 145 块）再加滞后量，而滞后量 = 4 × 最近四个窗口的释放量、**没有上界**（一窗删一个 8 块对象时平均约 34 块、最大 44 / 52，一窗删 32 个时 1044 块），删掉 s 字节 `df` 一个字节都不涨；c_max 按现算取，第一版池规模 9 块 ⇒ 保留池 73 块。
- **最少保留 4 个状态只在一次处置里根槽写失败 ≤ k_tol 次时兑现**：S = 4、c₀ = 5、3 次失败那一格违反 24 次，按字面记一次输、按 k_tol 的射程在外；全枚举下 S = 4、c₀ = 1 也违反 160 次（同为超过容忍的格）。
- **代价是选项本身的，不是这一族的缺陷**：一个被保留的状态钉住的是在它那一刻活着、后来才死的全部块，选最近 4 个就是每次删除都撞上——滞后量与「删了要等 3 次状态」都从这里来。
- **回收门槛取环里最旧有效根的代价**：崩在记录之后、根槽之前那一形留在环里的旧根要等被覆盖一圈才离开，每个洞回收门槛多压一圈（正推腿端口上 S = 16 平均 893 块对窗口门槛 583 块）；一块盘持续写不进根时扣住的单元随发布线性涨、没有上界（攻方模型上量，持续写失败 15 / 30 / 60 / 120 次时最多扣住 50 / 80 / 140 / 260）；崩在记录之后、根槽之前连崩 13 次之后挂载（S = 4），已分配 166 块、窗口门槛同一段 26 块（攻方副本上量）。数两腿各量一种口径，不是入库装置上的数；用户 2026-10-04 知情定。
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
- 回收门槛与同取窗口的另外五处退回环里最旧有效根：判决 `research/prompts/m3-reclaim-window-r1-main-verification.md` 第二节 W1、W3 各行与第六节（窗口门槛在目标形态上出局：丢一块盘之后挂载落到幸存盘上更旧的根，读出来是坏数据，计数模型全枚举；只改分配器门槛或只改 checker 都重开 I-3.1（已分配统计对得上） 误红，副本量过；六处一起退，两腿独立量到零假候选、零不可恢复，被攻过一轮）；用户定案 2026-10-04（Q1 RGall、Q2 下一轮就改），原话在变更史。E139（按盘回退下界的收严形态） 第 1 次重跑：旧门槛对照臂 G2SW_ringgate 判的行零假候选、零不可恢复，加窗口的 G2SW 有非零行；其中一部分是装置的候选下界没带系统配置里的 F 造出来的（判决 W2 行），数不当依据，以实验页为准，装置补 F 之后的第 2 次重跑欠着（欠里记着）。无实验：判决里的数在三方腿的副本与模型上量，没立实验号，不是入库装置上的数。此前取一圈窗口的三方判决（`research/prompts/m3-ring-r2-main-verification.md`、`research/prompts/m3-ring-r3-main-verification.md`）与用户 2026-10-02 的定案，经过与原话在变更史。
- 抬 F 的上限并进挂载那一刻读不出或自证不过两类槽：代码轮第三轮判决 `research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K2 的 RS3 行（一槽从挂载起读回全零、读得出而自证不过，根环表不记它；准入抬 F 的上限算成 22，越过盘上完好的非空有效根 21，池级 checker 判 I-7.9（回退下界 F 不高于抬 F 的上限） 红；设备报错那一形停在 21、0 红；副本量过，主 agent 复跑对上）与第六节 Q3 行；用户定案 2026-10-04（读不出与自证不过两类都并），原话在变更史。无实验：数在攻方副本上量，没立实验号。
- 抬 F 那一遍并进挂载时读不出的槽、进程内记根先重读、抬 F 的上限只取窗口里的根（这一半随回收门槛退回环里最旧有效根而作废）：代码轮判决 `research/prompts/m3-root-ring-code-r1-main-verification.md` 第二节 K2-e、K2-b、K2-b2 三行（攻方原型量过，主 agent 复跑 115 份输出逐字节相同）与第三节表二之后用户定案那一段（E2、B2）。
- 进程内重读读得出而自证不过按记的最大 txg 算、重读读出有效根的槽算进这个进程知道住着根的槽、抬 F 的上限把挂载那一刻读不出的槽当作可能是非空有效根：代码轮第二轮判决 `research/prompts/m3-root-ring-code-r2-main-verification.md` 第二节第 2、6 行（有效根所在的槽挂载那一刻读错、重读又读坏，实现当没有根，回收门槛越过盘上有效根，攻方模型上 12 组参数 12 组红，改法 A 12 组 0 红；抬 F 越过读不出的有效根那一形，攻方探针上 F = 22、读得出时 F = 21，推的）与第四节 RS1、RS3 两行（改法 A 的代价是从没写过的槽加挂载读错时记账多，安全方向，2 组；上限算进读不出的槽的代价是读故障时 F 抬得少一点，推的）；用户定案见同一份判决第六节，原话在变更史。无实验：数在攻方模型上量，没立实验号。
- 正常卸载那一串回收的槽只放开一截：设计轮判决 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`（续做推满之后卸载在预演里被拒，丙1 站住、零轮）；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 N1 各行（丙1 在单盘幸存时写掉被择中的旧根引用的单元，两次抽样同向打中，主 agent 复跑坐实；恢复择根不看 F）与岔路单 `research/prompts/m3-unlink-code-r3-forks.md`。用户定案 2026-10-03：「丙 1 进下一批」、单盘那一问「甲 GA（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 有一块盘没有有效根时整截扣、读不出的槽当没有根、卸载半途崩溃之后的下一次可写挂载只放开一截：代码轮判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节（N1「根环里读不出的槽、有洞」那一行是代码取的保守读法；「形 R」那一行攻方打中、主 agent 复跑坐实，`research/prompts/m3-batch14-code-r1-main-checks/`）与岔路单 `research/prompts/m3-batch14-code-r1-forks.md`。用户定案 2026-10-04 选「甲 再挂也只放开一截（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 再挂时某块盘一条有效根都读不出只放到最旧有效根（该判决写作一圈窗口里最旧有效根，随回收门槛退回环里最旧有效根）、当场放开的式子带上最旧有效根那一项：代码轮判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节（N1「再挂期间某块盘根环整段读坏」那一行攻方打中、主 agent 复跑坐实，`research/prompts/m3-batch16-code-r1-main-checks/`；窗口那一项是正推腿报的条款漏写）与岔路单 `research/prompts/m3-batch16-code-r1-forks.md`。用户定案 2026-10-04 选「甲 只放到窗口里最旧有效根（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 树表 0 条那一版上正常卸载一个字节都不写、F 不动：用户定案 2026-10-04（选乙「不推，条款照实现写」，原话在变更史），把实现里主 agent 2026-09-26 那条定案（`crates/singlefs-core/src/mount.rs` 的 `Unmounted::NothingWrittenOnAVersionWithoutFile` 的注释，推的，被攻过零轮）写进条款；代码轮判决 `research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K1 与第五节第 1 行（照字面推空发布那一读法在攻方副本上量过：模型推 2 条、对拍报 PublishCount）。无实验：数在三方腿的副本上量，没立实验号。

**欠**：C282（环里最旧根没有定义）——定义与条款写在已定项 1 里，会红的检查仍欠；C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）（与 defer 窗口取多大无关，病根在 D23（journal 的角色与格式） 已定项 14 的影子账）；C329（写行那次发布之前推抬 F 的空发布没有检查）；C563（读根环一槽读坏就当没有根，别的调用方没判）；C660（回收门槛改成一圈窗口之后按盘回退下界的装置要重跑）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:84-96`（整段抄，未转述）**

```markdown
#### 已定项 2：checkpoint 的触发判据取时间与脏数据量，两者取先到

**定案**：`触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)`。两个阈值是**系统配置声明值**，挂载时求值一次（形态与 D21（权威态与派生态的分界）「单元几何挂载时求值一次」同型）。

**射程**：定的是什么时候触发，不定两个阈值取多少（已定项 5），也不定触发之后怎么切分事务（同样在已定项 5）。已知边角一样：`.claude/rules/fs-design.md` 硬要求 5 说的分支变量是那两个阈值、它们在挂载期间恒定，`now` 与脏字节数是被比较的量、会变，不是分支变量。

**依据**：

- 无实验：`∨` 这个形态是两条失败模式互相堵出来的，没有可量的量——只按脏数据量，空闲系统上最后一批写可以无限期不落、崩溃丢窗口无界；只按时间，突发写会把一个 checkpoint 的 journal 占用撑过环几何，而 I-8.1（环几何够大） 要的是「任一事务的最坏 journal 占用」有界。那个 `∨` 式子就是全部，没有魔数以外的自由度，`.claude/rules/fs-design.md` 硬要求 1「判据显式、可计算、写下来」因此满足。可量的量是两个阈值的取值与它们的可行域，在已定项 5。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:97-117`（整段抄，未转述）**

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

**出处 `.claude/kb/decisions/16-发布语义.md:118-137`（整段抄，未转述）**

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

**出处 `.claude/kb/decisions/16-发布语义.md:138-162`（整段抄，未转述）**

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

**出处 `.claude/kb/decisions/16-发布语义.md:163-185`（整段抄，未转述）**

```markdown
#### 已定项 6：发布计数器，四个等号写成一条

**定案**：每次发布把 checkpoint_txg 加一——fsync 触发的发布也是发布，没有「小发布不记号」的例外；根槽的轮转键（D22（单元原子性怎么合成） 已定项 2 的「区域 = txg mod R」里那个 txg）、记账的代（D5（快照 / 空间记账机制） 已定项 2）、恢复重放的水位（D23（journal 的角色与格式） 已定项 14）都是这同一个计数。不另设「发布代号」字段——它与 checkpoint_txg 是同一个东西。**四个等号**：fsync = 提前发布、区域 = txg mod R、一个 txg 一个号、记账代 = checkpoint 号，四处读的是同一个数，哪个实现读岔一处就会撞根槽或读错账。

**四条骑手条款**（与定案同日采纳，不是可选项）：

1. **根记录必须带 32 位实例代号**（与 D23（journal 的角色与格式） 已定项 9 的 journal 实例代号是同一个计数），择新在 checkpoint_txg 平局时按实例代号高者赢。机制：丢盘回退 + 盘失而复得会造出两条同 txg 的自证合法根——journal 靠实例代号分辨时间线，根环此前什么都没有。字段落点在 D22（单元原子性怎么合成） 已定项 7。
2. **记账 key 的「代」段宽度 ≥ 48 位**：按本机 2785 发布/秒，32 位 17.8 天回绕。记账 key 的宽度算术自陈是假设，要按此重算。
3. **`T_time` 下界的推导（197 ms）只适用于非 fsync 触发的发布**——那步推导以「一窗多次操作共享一次发布」为前提，fsync 密集负载下窗口由 fsync 自身触发。引用已定项 5 的下界时连这句一起引；上界 5 s（用户定的丢失窗口）不受影响。
4. **记账条目的跨发布合并不再可行**（代进了 key，「后者胜」并不掉不同代的两条）⇒ 每次发布把这一代的行全写一遍就是实付值，不是上界。代价，照实记。

**射程**：定的是这个计数怎么加、四处怎么共用一个数，不定根环几何本身（D22（单元原子性怎么合成） 已定项 2）、也不定 journal 实例代号什么时候递增（D23（journal 的角色与格式） 已定项 9）。已知边角一样：这条定案当天的对抗测试实际跑在两条腿上——本地腿那一份后来被收窄后的字词损坏检查判红（`proposed proposed`，实词自复读那一类），按 `.claude/rules/three-way-inference.md`「闸判红之后那一轮作废」，**不许再拿它当「三方一致」引用**；定案本身不因此推翻，两条云端腿各自的判定性产出仍在。

**依据**：

- E48（根环几何的可行点）：轮转键就叫 txg 且逐次发布递增，9 个可行点全部建在这个读法上。
- E50（根环槽数的上下界）：源码逐字「一圈时长（秒）= 槽总数 ÷ fsync 率。每次 fsync 发一次根」——已入库的根环算术只在「每次发布 checkpoint_txg 加一」这条等式下为真，不钉它那批实测追溯性作废。
- E53（丢一整块盘之后根环还挂不挂得上）：丢盘回退 + 盘失而复得的那条历史，就是骑手条款 1（根记录带实例代号）要分辨的东西。
- E44（序号位宽的本机实测与代价）：本机 2785 发布/秒，32 位 17.8 天回绕——骑手条款 2 的「≥ 48 位」是从这个速率来的。
- 对抗测试（用户要求先攻再定）的原始输出 `c85-publication-counter-forward 那一轮的腿报告（产物在 git 里）`、`c85-publication-counter-reverse 那一轮的腿报告（产物在 git 里）`、`c85-publication-counter-local.out`；用户定案，原话在变更史。

**欠**：C85（发布计数器的四个等号没写死）（条款已写成一条，会红的检查仍欠）、C88（根环的时间线判别未实现）（骑手条款 1 的 checker 侧，反推腿那一轮找出来的可达破绽）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:186-202`（整段抄，未转述）**

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

**出处 `.claude/kb/decisions/16-发布语义.md:203-224`（整段抄，未转述）**

```markdown
#### 已定项 8：暖机取戊，新实例的根先覆盖两块盘

**定案**：新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回；做法是连推空发布，第一版几何至多 R = 3 次。**第一版实付 2 次，而且它是格式常量不是运行时谓词**：根环三个区域的设备归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7），于是 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定。登记成 `format-const` WARM_UP_EMPTY_PUBLISHES = 2 与 NEW_POOL_FILE_CREATION_TXG = 3（[layout/01-first-txn.md](../layout/01-first-txn.md) 八），条款在 D22（单元原子性怎么合成） 已定项 16 第 5 句。**后续挂载与挂载中途崩了再挂**：次数现算——写行那次发布的 txg 取 D23（journal 的角色与格式） 已定项 14 注 3 那个下界本身（max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1），之后每次空发布加一，按根环落点看落哪块盘，推到本实例的根覆盖每块盘为止，至多 R 次；取号之前的准入与取号之后推发布用同一份计划（`crates/singlefs-core/src/mount.rs` 的 `warm_up_publish_txgs`）。

**射程**：定的是新实例确认之前要做什么，不定实例切换本身的判据（D23（journal 的角色与格式） 已定项 14）、不定空发布写出多少单元（已定项 9）。四样已知边角：

- **前置 C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 的条款那一半已满足**（用户取影子账，条款写进 D23（journal 的角色与格式） 已定项 14 与 I-7.4（近 K 代块未被复用）），暖机已生效；**检查那一半仍欠**：多次挂载的录制流已有（层 0 第二条流），逐句核仍差格 1 的专门断言，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 因此仍在未还表。
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

**欠**：C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）（检查那一半）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:225-239`（整段抄，未转述）**

```markdown
#### 已定项 9：一次空发布也写单元

**定案**：记账树存在时就写——空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2（每条根的记账树只装这条根那一代的行，每次发布把这一代的行全写一遍）重写记账行，连带记账树节点、分配记录、映射条目与树表单元。第一次可写挂载的暖机时树表 0 条 ⇒ 零单元；以后的空发布按 D28（挂载期承诺量） 已定项 4 现算的 c_max 块。**树表 0 条的一版上写行的那次发布写两个单元**：实例表与一片分配记录节点，分配记录树根指针住根记录（D22（单元原子性怎么合成） 已定项 7）；被换下的上一版实例表与上一片分配记录节点记成已释放，重开时从这片节点重建这一版的账。根记录那一项只有树表 0 条、写过行的那一版写非零，mkfs 的第 0 代与带文件的一版写全 0，零单元发布照抄上一版。

**射程**：定的是空发布写不写单元、写哪几样，不定 c_max 怎么算（D28（挂载期承诺量） 已定项 4 按当时结构现算）、也不定空发布什么时候推（推抬 F 在已定项 1，暖机在已定项 8）。写行那次发布写的那片分配记录节点，单元头树 ID 写 0（实现今天的取法：那一版的树 ID 水位是 mkfs 的 65536，写分配记录树的 65538 会让 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 当场红），没有条款；它进不进中央映射也没有条款——那一版没有映射树，D19（块指针的结构与宽度预算） 已定项 8 的自举豁免三类里没有它。

**依据**：

- E148（提交固定点按两棵记录树重算）：空发布的固定点按两棵记录树重算是池规模 9 块、新池新建文件规模 4 块，「以后的空发布按现算的 c_max 块」那句的数出自这里。
- E142（新池新建文件的干跑）：第一次可写挂载的暖机那两次空发布逐字节写出来，树表 0 条时零单元。
- 用户定案 2026-09-23（写行那次发布在树表 0 条的一版上写实例表与分配记录节点两个单元、分配记录树根指针住根记录），原话在变更史；实现与用例点名在 [checks-owed.md](../checks-owed.md) 已还清表 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 那一行。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:240-255`（整段抄，未转述）**

```markdown
#### 已定项 10：发布粒度取攒批，不是每个事务发一次

**定案**：攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。

**射程**：定的是频率，不定一次发布事件内部的顺序——那在 `litmus/commit-publish.litmus`，它钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。两样已知边角：

- **净代价分布**：格式侧几乎为零（journal 本来就要有），验证侧一大笔——中间态语义改写（已定项 12）、第二份 replay 实现（已定项 4 与 13），再加 D13（验证路线） 的「Merkle 根链差分」粒度变粗：一代根里塞了 K 个事务的改动，定位精度从「哪个事务」降到「哪个 checkpoint」，而 D13（验证路线） 已写明全量档的真代价是定位成本。
- **发布语义决定「走几次」**：每事务发根 → K 批走 K 次；checkpoint → 同一窗口内的 K 批只走约一次，各批只留一条 journal 记录（小、顺序写）⇒ 分批大删除那一格的元数据写放大从 `K × M × H` 降为 `checkpoint 数 × M × H`。

**依据**：

- 无实验：定案靠两条互相独立的推理，当时没有可量的量。① 它与 D5（快照 / 空间记账机制） 已定的 ZFS birth txg 模型天然相容——ZFS 的 txg 本身就是 checkpoint（`zfs_txg_timeout` 默认 5 秒，或攒够一定脏数据量），一个 txg 批量大量操作、txg 结束才更新一次 uberblock；采纳 checkpoint 之后 birth txg 的号粒度问题自动消失，一个 txg 一个号，deadlist 的比较粒度就是 checkpoint 粒度、不必重算（未在本项目验证）。② **根是丢失写的最终见证者**：检测「盘上这块是不是它应该是的那个版本」需要一个外部见证者，块自身办不到（合法的旧版本与合法的新版本在块内层面同构，D18（块里携带什么信息） 有完整论证），而见证者必须在不同的失败域上——父节点若在同一次事件里也没写成就要祖父，递归到根 ⇒ checkpoint 发布的那个根是这条递归的终点，它的持久化必须比任何单个块更可靠（ZFS 的形态是 4 个 label × 每个 128 KiB 的 uberblock 环、且偶数与奇数 label 分两阶段写；本工程根自身的冗余形态由 D22（单元原子性怎么合成） 已定项 2 定）。频率的可量代价（两个阈值的可行域）在已定项 5。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:256-268`（整段抄，未转述）**

```markdown
#### 已定项 11：没有任何操作可以绕过 journal 直接发根

**定案**：所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏。fsync 的效果是「提前触发一次发布」，**不是另开一条日志格式、另一套恢复逻辑**。判据是：journal 是所有提交的必经之路，还是某些路径的旁路——前者是一套机制、持久化分两级，合规；只有 fsync（或只有元数据、只有小写）走 journal、别的直接发根，就是 `.claude/rules/fs-design.md` 点名的「为 fsync 单开一套日志树」。

**射程**：说的是挂载之后的操作；mkfs 种子写第 0 代根不经 journal（D13（验证路线） 已定项 1 自己列的根槽三种写者之一，mkfs 一条记录都不写），不在这句的射程里。已知边角一样：D23（journal 的角色与格式） 已定项 1 的轴一取甲压在这一条上，重开它要回答的依据里有这一条。

**依据**：

- 无实验：它是 `.claude/rules/fs-design.md`「一个事务层，所有结构共用」在发布这一格的形态，判的是机制的形状（必经之路还是旁路），不是一个能量出来的数；不写死就会退化成「为某个路径开第二套事务机制」。每次持久化的写量是另一个问题，归 D23（journal 的角色与格式） 已定项 1。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:269-281`（整段抄，未转述）**

```markdown
#### 已定项 12：中间状态的合法性语义改写成「根 + journal 任意前缀」

**定案**：D8（核心索引结构）「中间状态必须每一个都是合法的文件系统状态」改写为「**根 + journal 任意前缀，replay 之后合法**」。

**射程**：定的是合法性怎么判，不定前缀本身怎么截——前缀判定的完整口径是 D23（journal 的角色与格式） 已定项 14 的五条，缺一不可。已知边角一样：块复用那一侧的真正约束比 checkpoint 边界更严，按 journal 位置说（已定项 15 与 D22（单元原子性怎么合成） 已定项 4）。

**依据**：

- E42（一事务几条记录）：1601 个前缀里 1400 个被前缀切开（87.4%），一事务一条那一臂非法前缀恒 0——那是「任意前缀 replay 之后合法」的直接形式；跨多条而无边界字段时非法前缀恰等于被切开的前缀数，加边界字段之后 0。这条语义在模型上是可数的，而且数得出它什么时候破。
- 用户定案，原话在变更史。

**欠**：C42（残留记录冒充合法前缀）（上一条时间线留在环里的记录会被下一次恢复当作合法前缀重放）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:282-294`（整段抄，未转述）**

```markdown
#### 已定项 13：这份 journal replay 归记录核对器，不归 O2（独立解析器 + checker）

**定案**：D13（验证路线） 已定 O2（独立解析器 + checker） 是一元谓词、只判单个镜像，所以这份 journal replay 归「记录核对器」。那是第二次实现 replay，而 replay 正是崩溃语义最容易出**共同误解**的地方——D13（验证路线） 的「三 oracle 合谋测试」要防的就是这个。**这是 D16（发布语义） 最贵的一笔。**

**射程**：定的是归谁，做什么与允许共享什么在已定项 4。已知边角一样：D23（journal 的角色与格式） 轴一定甲之后「redo 为空」，最贵那一笔的大头随之消失，只剩前缀判定 + 择根 + 续做。⚠️ **「redo 为空」只对「根已持久」的发布成立**：E42（一事务几条记录） 实测 1601 个前缀里 1400 个被前缀切开（87.4%），被切开的比例与边界字段在不在无关——变的是这 1400 个非法还是合法（无边界字段臂 1400 非法，有边界字段臂 0）⇒ **恢复期重放非空是常态，不是边角。**

**依据**：

- 无实验：哪个判决器实现 replay 是验证路线的分工，没有可量的量；第二份实现的成本量级与它必须独立实现什么在已定项 4。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:295-307`（整段抄，未转述）**

```markdown
#### 已定项 14：快照发布点与 checkpoint 边界对齐，钉在全部数据变更之后

**定案**：快照发布点必须与 checkpoint 边界对齐，且位置钉死在「本 checkpoint 全部数据变更之后」，快照不许出现在 checkpoint 中间。同一 checkpoint 内建两个快照是允许的：它们捕获同一状态、其间 deadlist 为空；但 checker 里的「上一个快照」要用 `(txg, 快照 id)` 打破并列。

**射程**：定的是快照点落在哪，不定快照本身的记账模型（D5（快照 / 空间记账机制） 与 D6（快照实现模型））。已知边角一样：这条规则的可执行形态是已定项 5 那段丢失窗口口径——发布阶段等全部单元落盘，快照点就落在那之后。

**依据**：

- 无实验：理由是机制性的，没有可量的量——`birth` 的分辨率只到 checkpoint，**块无法表达「我生在本 checkpoint 里快照点之前还是之后」**；不钉死它，D5（快照 / 空间记账机制） 那张边界表里 `birth == S.txg` 与 `death == S.txg` 两行会同时失去确定答案，而它们一个是**数据丢失**、一个是**空间泄漏**。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:308-327`（整段抄，未转述）**

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

**出处 `.claude/kb/decisions/16-发布语义.md:328-340`（整段抄，未转述）**

```markdown
#### 已定项 16：checkpoint 与消息缓冲买同一样东西，E7（离线索引 harness） 的对照组钉死

**定案**：checkpoint 与 D11（索引节点要不要留消息缓冲区） 的消息缓冲区买的是同一样东西，差别只在**摊销窗口住在哪**：checkpoint 的窗口住**内存**、不进格式；消息缓冲的窗口住**盘**、必须进格式。⇒ **D11（索引节点要不要留消息缓冲区） 的边际收益 = 「跨 checkpoint 的摊销」减去「checkpoint 内摊销」**。**E7（离线索引 harness） 的对照组必须是「有 checkpoint、无消息缓冲」。**

**射程**：定的是那个对照组怎么设，不定 D11（索引节点要不要留消息缓冲区） 取哪一边——那由 D11（索引节点要不要留消息缓冲区） 自己定。已知边角一样：BetrFS 那组数字里这两块从没被拆开过（它同时有 4 MiB 节点的消息缓冲**和** 5 秒 checkpoint），**未在本项目验证**。

**依据**：

- 无实验：这是「对照组怎么设」的规定，不是实验结论；文献里没有一个对照组带 write buffer 前端，消息缓冲相对本工程现方向的边际收益，现有文献一个数字都没给，本项目也还没有可引的量。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:341-363`（整段抄，未转述）**

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

**出处 `.claude/kb/layout/01-first-txn.md:1-27`（整段抄，未转述）**

```markdown
# 新池新建文件写出哪些字节

**这是 C66（格式级空白没有分项号）欠的那张表。** 它把「正确提交一个事务」这个第一个可运行目标
（见项目 `CLAUDE.md`）要往盘上写的每一段列出来，每个字段都指向一条已定或未定的决策分项。

**它的用途有两个。** 第一是反向检查：一个字段如果指不到任何分项，说明那是一处**格式级空白**——
它挡在第一行代码前面，却不在任何决策的未定项清单里，于是门禁阶段
「每个未定项有没有判过改不改新池新建文件的字节」对它一个字也说不出来。
C66（格式级空白没有分项号）立账正是因为这种空白当时有五处。
第二是给里程碑供数：[01-first-txn.md](../milestone/01-first-txn.md) 的每一步写「盘上多了什么」，
那些字节的宽度、取值、落点都以这张表为准，里程碑不另抄一份。

**口径**：新池新建文件指「mkfs 之后写一个小文件并发布一次」。它不含快照、不含 reflink、
不含加密（D9（加密） 已定项 10 定加密不进第一版）、不含压缩、不含目录树
（第一个可运行目标不要求目录，见项目 `CLAUDE.md`）。
宽度一律是**初值**，写代码时可调；调了要回来改这张表。

**每一行都给具体的数，分两种**：「指向」列指到已定项的，宽度与取值是那条分项定的；
指到未定项的，宽度与取值是**预想**——写代码时先按它取，分项定了就改，改完在文末「历史版本」记一行。
「新池新建文件里的值」一列写的是这一次 mkfs 加一次发布在盘上留下的实际值；
写「mkfs 随机」的是每次 mkfs 都不同的量，写「预想」的是仓里没有条款定它。

**与里程碑和决策索引挂钩的形式**（门禁阶段「新池新建文件的三份文件互相挂钩」判红）：
每一节标题下面一行写它属于里程碑的哪一步；里程碑每一步写它写出的字节在这里哪一节；
凡是 [decisions.md](../decisions.md) 里判过「改新池新建文件的字节：是」的未定项，
在这里的「还剩几处空白」表和里程碑的「会碰到的决策点」里都要各占一行。

```

**出处 `.claude/kb/layout/01-first-txn.md:28-88`（整段抄，未转述）**

```markdown
## 零、盘面地图与新池新建文件的写清单

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 1 到步 5 共用。

**两块盘、同构、`physical_block_size` = 512、io_min = 512**（D2（RAID 条带策略） 已定项 9 定第一版跑 2 块盘；
512 是本机 `nvme0n1` 现查的值，D22（单元原子性怎么合成） 已定项 2 那一节逐字「同构 512 池上槽就是 512」；io_min 由 D2（RAID 条带策略） 已定项 19 记进系统配置）。
全部固定结构两盘各一份、设备内偏移相同。落点坐标全池一个：(设备身份 4 字节, 16 KiB 槽号 6 字节)，
槽号 = 设备内偏移 ÷ 16384（D3（空间分配） 已定项 7、D19（块指针的结构与宽度预算） 已定项 4）。
这一节的槽号与写清单以 E142（新池新建文件的干跑） 第二十二次跑第一段的主产物为准：`research/results/e142-new-pool-file-creation-dry-run-r22-main-2026-10-05.out`（`research/scripts/replay.sh` 第 174 行登记的那份）里 `name=width` 30 行 expected == actual，`name=write_list` 29 条写、12 个单元，窗口里 29 个写与 `crates/` 的导出按 (设备, 偏移, 长度) 配对、逐字节全等（`name=impl_bytes_equal_summary … matched=29 equal=29 unequal=0`，记账树根第 10 条已承诺预留两边都是 98 386）；分配记录树与 extent 树按 D8（核心索引结构） 已定项 14 按位置寻址。

| 设备内偏移 | 长度 | 是什么 | 16 KiB 槽号 | 指向 | 状态 |
|---|---|---|---|---|---|
| 0 | 4096 | 系统配置槽 0 | 0 | D22（单元原子性怎么合成） 已定项 8（每盘一份、2 槽轮换）；槽宽 = 格式常量 4096（2026-09-14 三方第一轮 + 用户收尾弹窗定案；此前是探测到的 `physical_block_size` 512，那条 D20（承重面：单元的原子性与自包含） 类规则是分辨率规则不是尺寸规则）；槽数恒 2、槽 i 偏移 = i × 固定结构槽距（D22（单元原子性怎么合成） 已定项 16） | 已定 |
| 4096 | 4096 | 系统配置槽 1 | 0 | 固定结构槽距 = max(4096, mkfs 时的 io_min) = 4096（D2（RAID 条带策略） 已定项 19；io_min 与槽距两个字段进系统配置并进挂载比对） | 已定 |
| 1 MiB | 3 × 32 KiB | 根环区域 0 / 1 / 2 各自的起点，区域 r 在 基址 + `r × P × chunk`，基址 = 系统配置「根环起点」字段（1 MiB）、P = 3、chunk = 1 MiB ⇒ 1 / 4 / 7 MiB，公式住设备内偏移 | 64 / 256 / 448 | D22（单元原子性怎么合成） 已定项 2 / 已定项 16 | 已定 |
| 每区域内 | 8 槽 × 512，槽距 4096 | 根槽，每区 S = 8（S 住系统配置、区间 4..16：下界 4 由 D16（发布语义） 已定项 1 的环深下限给出，上界由 E50（根环槽数的上下界） 给出）；区域内槽位 = `(txg div R) mod S` | — | D22（单元原子性怎么合成） 已定项 2 / 已定项 16；I-7.5（根槽按判定宽度对齐） | 已定 |
| — | — | 区域归属：区域 0 → 盘 0，区域 1 → 盘 1，区域 2 → 盘 0，mkfs 逐区域写进系统配置；devs < R 按 D22（单元原子性怎么合成） 已定项 14 的鸽笼下界判、不拒绝挂载 | — | D2（RAID 条带策略） 已定项 7；E87（固定结构的放置） 逐字「两盘上 R=3 必有一盘背两个区域」 | 已定（哪个区域落哪盘是 mkfs 参数） |
| 16 MiB | 768 MiB | journal 环，两盘互为镜像，记录 4096，mkfs 整环写 0；196608 个记录槽，在飞记录数上限 = 196608 ÷ 3 = 65536（语义 = 重放前缀最多 65536 条，坏校验和断链即止，D23（journal 的角色与格式） 已定项 18） | 1024 .. 50175 | D22（单元原子性怎么合成） 已定项 8；D23（journal 的角色与格式） 已定项 2 / 已定项 12 / 已定项 18 / 已定项 19（**环长是 mkfs 参数、默认 768 MiB、约束环 ≤ 设备容量 ÷ 4**，2026-09-14 用户定案改写；挂载时有效 T_dirty = min(字段, 环长 ÷ F) = 256 MiB） | 已定 |
| 784 MiB 起 | 到盘尾 | 单元区：系统配置「单元区起始槽号」= 50176（D3（空间分配） 已定项 10 ④），分配器不下探、固定结构不写分配记录；数据单元对齐 32768、节点对齐 16384（D18（块里携带什么信息） 已定项 9）；聚簇段 64 槽、开放段 = 单元区内最低的 64 槽对齐全空段（D3（空间分配） 已定项 10 ①）；**镜像 4 GiB 时单元区 = 262144 − 50176 = 211968 槽 = 3312 段**（镜像 4 GiB 是里程碑步 0 的预想，由 D23（journal 的角色与格式） 已定项 19 ③ 的「环 ≤ 设备容量 ÷ 4」逼出来——1 GiB 的镜像装不下 768 MiB 的环） | 50176 .. | D3（空间分配） 已定项 7 / 已定项 10 | 已定（镜像大小是里程碑步 0 的预想） |

**树 ID 的分配**（D8（核心索引结构） 已定项 11：从 65536 起、种类字段 2 字节取不到的最小值，与种类码错开）：extent 树 65536、inode 树 65537、分配记录树 65538、记账树 65539、中央映射树 65540、livelist 共享树 65541（D6（快照实现模型） 已定项 2）、稀疏旁表树 65542（D5（快照 / 空间记账机制） 已定项 6）、deadlist 树 65543（D5（快照 / 空间记账机制） 已定项 11，2026-09-14 用户定案 day-1 注册）、`deleted_inodes` 树 65544、墓碑树 65545、谱系树 65546、logged_ops 树 65547（D18（块里携带什么信息） 已定项 24 第 6 条）；
树 ID 水位在 mkfs 时写 65536、第一次发布后写 **65548**（十二棵；只注册前八棵时 65544；D8（核心索引结构） 已定项 8 ②）。中央映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）⇒ 树表第 1 版 **11 条**。
数据单元五元组里的树 ID 取引用它的 extent 树，即 65536（D18（块里携带什么信息） 已定项 11 登记表码 1 那一行逐字「树 ID 今天取读者树」）。

**mkfs 写出的单元与新池新建文件写出的单元，按分配顺序**（落点两盘相同——刚 mkfs 的两盘空闲图相同、同一条规则给同一个号，不是条款；每个单元两盘各一份，D2（RAID 条带策略） 已定项 10 / 已定项 6）：

| 序 | 谁写 | 单元 | 码 | 大小 | 16 KiB 槽号 | 树 ID | 哪一节 |
|---|---|---|---|---|---|---|---|
| m1 | mkfs | 实例表单元第 0 片 | 3（打包记录类型 4） | 32768 | 50176–50177 | 0（无归属） | 七 |
| m2 | mkfs | 树表单元第 0 版（0 条条目） | 2 | 16384 | 50178 | — | 七 |
| m3 | mkfs | 第 0 代根记录，三个区域各自槽 0，实例代号 0 | 16（自证） | 457 / 512 槽 | 区域 0 / 1 / 2 | — | 七 |
| m4 | mkfs | 系统配置，两槽同内容、世代号 1、实例代号 0 | 18（自证） | 489 / 4096 槽 | 0 | — | 一 |
| a1 | 第一次可写挂载取号 | 系统配置槽 0，世代号 2，实例代号 1（D23（journal 的角色与格式） 已定项 16：取号先写进每一份系统配置、全或无，之后才动单元） | 18（自证） | 489 / 4096 槽 | 0 | 两盘各一份 | 一 |
| w1 | 暖机 | 空 journal 记录 jsn (1, 1)、checkpoint_txg 1、事务号 0、不点名、新根段照 mkfs 根（D16（发布语义） 已定项 8：第一次可写挂载先连推空发布，直到本实例的根覆盖两块盘） | 17（自证） | 4096 | 环内偏移 0 | 两盘各一份 | 六 |
| w2 | 暖机 | 第 1 代根记录，区域 `1 mod 3` = 1 的槽 0，FUA；树表、实例表指针照 mkfs，映射根指针零 | 16（自证） | 457 / 512 槽 | 区域 1 | — | 七 |
| w3 | 暖机 | 系统配置槽 1，世代号 3，tail = 1 | 18（自证） | 489 / 4096 槽 | 0 | 两盘各一份 | 一 |
| w4 | 暖机 | 空 journal 记录 jsn (1, 2)、checkpoint_txg 2；反向链 750278362（= CRC32C(w1 的 487 字节头)） | 17（自证） | 4096 | 环内偏移 4096 | 两盘各一份 | 六 |
| w5 | 暖机 | 第 2 代根记录，区域 `2 mod 3` = 2 的槽 0，FUA——两个暖机根分住两块盘，暖机到此为止（区域归属 0 / 1 / 0 写死 ⇒ 暖机恒 2 次，D22（单元原子性怎么合成） 已定项 16 第 5 句） | 16（自证） | 457 / 512 槽 | 区域 2 | — | 七 |
| w6 | 暖机 | 系统配置槽 0，世代号 4，tail = 2 | 18（自证） | 489 / 4096 槽 | 0 | 两盘各一份 | 一 |
| t1 | 事务 | 数据单元（文件内容）——不在开放段 [50240, 50304) 里的最低 32768 对齐空槽对 | 1 | 32768 | 50180–50181 | 65536 | 二 |
| t2 | 事务 | extent 树上段根兼叶（1 条上段叶条目：inode 1 的那一个数据指针内联，标签 2；只有一个数据单元的文件不建下段，D8（核心索引结构） 已定项 14）——开放段起 bump，树 ID 最小 | 2 | 16384 | 50240 | 65536 | 四·二 |
| t3 | 事务 | inode 树叶容器（1 条记录）——码 3 按「数据单元」那一档取落点，起点要 32768 对齐 ⇒ 跳过 50241 | 3（打包记录类型 2） | 32768 | 50242–50243 | 65537 | 四 |
| t4 | 事务 | inode 树根（1 条条目） | 2 | 16384 | 50244 | 65537 | 四 |
| t5 | 事务 | 分配记录树叶（设备 0 那 14 条记录；罩槽 [49532, 50344)，是 W = 812 下的第 61 片，D8（核心索引结构） 已定项 14）——树内先叶后根、同层按 key 升序 | 2 | 16384 | 50245 | 65538 | 五 |
| t6 | 事务 | 分配记录树叶（设备 1 那 14 条记录；同一段槽） | 2 | 16384 | 50246 | 65538 | 五 |
| t7 | 事务 | 分配记录树层级 1 节点（设备 0；1 条内部条目，指 t5） | 2 | 16384 | 50247 | 65538 | 五 |
| t8 | 事务 | 分配记录树层级 1 节点（设备 1；1 条内部条目，指 t6） | 2 | 16384 | 50248 | 65538 | 五 |
| t9 | 事务 | 分配记录树根（层级 2，罩整个 key 空间；2 条内部条目分指 t7 / t8；两块 4 GiB 盘时树高 3，D8（核心索引结构） 已定项 14） | 2 | 16384 | 50249 | 65538 | 五 |
| t10 | 事务 | 记账树根兼叶（15 条记录） | 2 | 16384 | 50250 | 65539 | 五 |
| t11 | 事务 | 中央映射树根兼叶（10 条条目：码 1 一条 + 码 2 / 码 3 九条，D19（块指针的结构与宽度预算） 已定项 8）——倒数第二 | 2 | 16384 | 50251 | 65540 | 三·二 |
| t12 | 事务 | 树表单元第 1 版（11 条条目：extent、inode、分配记录、记账四棵有根 + livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops 七棵根指针为零）——最末 | 2 | 16384 | 50252 | — | 七 |
| t13 | 事务 | journal 记录 1 条 jsn (1, 3)、checkpoint_txg 3、事务号 1（点名 t1..t12 共 12 项，一项含两盘的位置条目）；反向链 2660362816（= CRC32C(w4 的 487 字节头)）；新根段指向 t12（50252）/ t11（50251） | 17（自证） | 4096 | 环内偏移 8192 | — | 六 |
| t14 | 事务 | 第 3 代根记录，区域 `3 mod 3` = 0 的槽 1 | 16（自证） | 457 / 512 槽 | 区域 0 | — | 七 |
| t15 | 事务 | 系统配置槽 1，世代号 5，tail = 3（根槽之后再更新，D16（发布语义） 已定项 7 的注） | 18（自证） | 489 / 4096 槽 | 0 | — | 一 |

⇒ **写请求数**：事务本身单元 12 × 2 盘 = 24（t1..t12），记录 1 × 2 盘 = 2，根槽 1，系统配置 2 ⇒ **29 条**，屏障 2 道，FUA 1 道；它前面是取号的系统配置写 2 条（a1）与暖机两次空发布各 5 条（记录 2 + 根 1 + 系统配置 2）⇒ **取号 2 条 + 暖机 10 条**，屏障与 FUA 的道数以八那一节登记的段序列为准。
（持久顺序 D16（发布语义） 已定项 7：单元与节点 → 屏障 → journal 记录 → 屏障 → 根槽 FUA；系统配置的槽轮换写在根槽之后。）
分配记录 28 条 = 14 个单元 × 2 盘（m1、m2、t1..t12 各一条；设备 0 的 14 条住 t5、设备 1 的 14 条住 t6，都落在罩槽 [49532, 50344) 的那一片叶里；分配记录树自己五个节点（t5..t9）的落点在写出前已定，D3（空间分配） 已定项 5 的提交内生块；deadlist 树 day-1 注册但没有根节点 ⇒ 不占落点、不写分配记录）。
m2（树表单元第 0 版）被 COW 之后仍被第 0 代根引用，但条目照样改写成已释放、释放代 3（D3（空间分配） 已定项 7：释放 = 放进 defer 队列那一刻；槽仍占着，要等 F_生效 抬过 3 才可再分配，D16（发布语义） 已定项 1）——2026-09-17 里程碑「覆盖写、释放、回退与复用」步 5 逼出：不释放它，txg 0 的根离开候选集之后这一槽永远占着、I-3.1（已分配统计对得上） 红。
**新池新建文件里的释放只有这一条**（被换下的 m2），墓碑、deadlist、稀疏旁表一个字节都不写。

```

**出处 `.claude/kb/layout/01-first-txn.md:89-167`（整段抄，未转述）**

```markdown
## 一、mkfs 已经种下的（新池新建文件之前就在盘上）

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 1。

| 段 | 字段 | 宽度 | 指向 | 状态 |
|---|---|---|---|---|
| 系统配置 | 每盘一份，更新走 2 槽轮换：槽世代号从 1 起每写 +1，下一次写的槽 = 世代号 mod 2，择槽取校验和过且世代号最大 | — | D22（单元原子性怎么合成） 已定项 8 / 已定项 16 | 已定 |
| 系统配置 | 槽宽 = 格式常量 4096（2026-09-14 三方第一轮主 agent 判决 + 用户收尾弹窗定案：D20（承重面：单元的原子性与自包含） 那条类规则是撕裂判定的分辨率、不是结构尺寸；正推腿判「记录的槽距」、反推腿判「常量 4096」；登记名 SYSTEM_CONFIGURATION_SLOT_BYTES <!-- format-const: SYSTEM_CONFIGURATION_SLOT_BYTES = 4096 -->）；槽距 = 4096 向上取整到 io_min 的整数倍（D2（RAID 条带策略） 已定项 19，同日定案） | 4096 | D20（承重面：单元的原子性与自包含） 推论三第 1 问自证单元那一格（2026-09-10 用户定案）；C232（系统配置槽宽：类规则给了答案，字段表说未定） 记着 D18（块里携带什么信息） 已定项 11 登记表那一行仍写「占位」 | 已定（2026-09-14 用户定案） |
| 系统配置 | 不带自己的树表单元指针，树表的权威在根记录 | 0 | D22（单元原子性怎么合成） 已定项 12 | 已定 |
| 系统配置 | 字段表（journal 几何、tail、根环参数、阈值、加密预留、单元区起点、io_min 与槽距、写入者身份、校验和算法标识、mkfs 时的 physical_block_size、扩展点声明值 N） | 489 | D22（单元原子性怎么合成） 已定项 9 / 已定项 15 | 已定（2026-09-13 用户定案，2026-09-14 用户定案加四个字段），表见「系统配置字段表」 |
| 系统配置 | 槽轮换要的世代号与整槽校验和：CRC32C 4 字节 + 28 字节零，覆盖整槽 4096 含补齐、字段按零参与 | 8 + 32 | D22（单元原子性怎么合成） 已定项 21（形态）；D18（块里携带什么信息） 已定项 17（算法与覆盖范围） | 已定 |
| 根环 | R=3 区域、素数步长放置、跨区轮转槽序、槽宽 = 探测到的 `physical_block_size` | — | D22（单元原子性怎么合成） 已定项 2 | 已定 |
| 根环 | 第 0 代根种进**全部**区域 | 3 × 457 | D22（单元原子性怎么合成） 已定项 8 | 已定 |
| 根环 | 区域内槽位 = `(txg div R) mod S`，一次 txg 只写一个槽 | — | D22（单元原子性怎么合成） 已定项 16 | 已定 |

**系统配置字段表**（D22（单元原子性怎么合成） 已定项 9 于 2026-09-13 定案、已定项 15 同日改三格：去掉间接目录单元指针 59、加单元区起始槽号 8 与 io_min / 槽距各 4；行取自 E100（系统配置的三段几何） 的 24 字段候选表、
补 E115（系统配置字段表的完备性与字节预算） 数出的缺项、按 E124（系统配置字段表按 D18 已定项 14 重算） 把 nonce 水位改成 12；主密钥槽 2026-09-13 用户定案内联进槽）：

| 段 | 字段 | 宽 | 新池新建文件里的值 | 出处 |
|---|---|---|---|---|
| 自举头 | magic | 4 | 4 字节 ASCII `SFSB`，十六进制 53 46 53 42，从偏移 0 起按此顺序排（字节串，不涉字节序）；取值出处 `crates/singlefs-core/src/system_configuration.rs` 第 22 行 `SYSTEM_CONFIGURATION_MAGIC`，checker 自写的一份 `crates/singlefs-checker/src/lib.rs` 第 192 行 | 骨架 |
| 自举头 | 格式版本 | 2 | 1 | 骨架 |
| 自举头 | feature bits（incompat / compat_ro / compat 各 256 位） | 96 | incompat 位 1 = 1（第一条纯 SSD 布局线，带 F 与卸载记号的那一版；位 0 退役），其余全 0；位 n 住第 n div 8 个字节的第 n mod 8 低位 | D15（格式冻结政策） 已定项 1「三个 bitmap 各 256 位」、已定项 4 位分配登记表（2026-09-13 用户定案）；位序 D22（单元原子性怎么合成） 已定项 13 / 已定项 17 |
| 自举头 | fsid | 16 | mkfs 参数（里程碑步 1 验收：同参数两次 mkfs 逐字节相同） | I-1.4（块头 fsid 一致） |
| 自举头 | 写入者身份（实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32） | 20 | `singlefs-rs` / 1 | I-1.5（系统配置记管道身份）；D17（实现分层与第三方管道） 债 3；D22（单元原子性怎么合成） 已定项 9（2026-09-14 用户定案） |
| 自举头 | 校验和算法标识（0 无效、1 CRC-32C） | 1 | 1 | I-2.2（校验和算法全盘一致）；C309（32 字节校验和的算法没定）；D18（块里携带什么信息） 已定项 17；D22（单元原子性怎么合成） 已定项 9（2026-09-14 用户定案） |
| 自举头 | 本盘设备号 | 4 | 0 / 1 | D19（块指针的结构与宽度预算） 已定项 4 同宽 |
| 自举头 | 设备数 devs | 4 | 2 | E115（系统配置字段表的完备性与字节预算） 缺项 8；D22（单元原子性怎么合成） 已定项 15 ③ 留 |
| 自举头 | 槽世代号 | 8 | mkfs 1，取号 2，暖机 3、4，新池新建文件发布后 5 | D22（单元原子性怎么合成） 已定项 21 / 已定项 16 |
| 自举头 | 整槽校验和 | 32 | CRC32C 4 + 28 零，覆盖整槽 | D22（单元原子性怎么合成） 已定项 21；D18（块里携带什么信息） 已定项 17 |
| 加密预留 | 系统配置自身 MAC | 16 | 全 0 | D9（加密） 已定项 8 |
| 加密预留 | nonce 水位 | 12 | 0 | I-6.5（nonce 水位单调）；E124（系统配置字段表按 D18 已定项 14 重算） |
| 加密预留 | KDF 标识 | 4 | 0 | D9（加密） 已定项 12 第 3 项；宽 4 是 D22（单元原子性怎么合成） 已定项 9（2026-09-13 用户定案） |
| 加密预留 | 加密类型 | 1 | 0（未加密；登记表在 D9（加密），D22（单元原子性怎么合成） 已定项 17） | D9（加密） 已定项 12 第 3 项 |
| 加密预留 | MAC 长度声明 | 1 | 16 | I-6.6（MAC 长度一致） |
| 加密预留 | 主密钥槽（内联进槽） | 80 | 全 0（加密关） | D22（单元原子性怎么合成） 已定项 9（2026-09-13 用户定案：内联，不走间接目录单元；80 是 E115（系统配置字段表的完备性与字节预算） 的最小可用值）；D9（加密） 已定项 12 第 3 项；内部构成随 D9（加密） 开加密时定 |
| 几何 | 节点大小 | 4 | 16384 | D8（核心索引结构） 已定项 2 |
| 几何 | 单元大小 | 4 | 32768 | D4（校验和位置） 已定项 5；D18（块里携带什么信息） 已定项 9 |
| 几何 | 落点粒度 | 4 | 16384 | D3（空间分配） 已定项 7 |
| 几何 | 位置条目宽度 | 4 | 14 | D19（块指针的结构与宽度预算） 已定项 4 |
| 几何 | mkfs 时的 `physical_block_size` | 4 | 512 | D13（验证路线） 已定项 5 的「与系统配置记录的 mkfs 时取值比对」；D12（目标介质） 已定项 4 三条比对之一；D22（单元原子性怎么合成） 已定项 9（2026-09-14 用户定案） |
| 几何 | 扩展点声明值 N | 4 | 0 | D21（权威态与派生态的分界） 已定项 8；D22（单元原子性怎么合成） 已定项 9（2026-09-14 用户定案） |
| 几何 | journal 环起点（16 KiB 槽号） | 8 | 1024 | D23（journal 的角色与格式） 已定项 2 |
| 几何 | journal 环长度（字节） | 8 | 805 306 368 <!-- format-const: JOURNAL_RING_DEFAULT_BYTES = 805306368 --> | D23（journal 的角色与格式） 已定项 2 / 已定项 19（mkfs 参数、默认 768 MiB、约束环 ≤ 设备容量 ÷ 4，2026-09-14 用户定案） |
| 几何 | journal 最大记录字节 | 4 | 4096 | D23（journal 的角色与格式） 已定项 12 |
| 几何 | journal 在飞记录数上限 | 4 | 65536（= 196608 ÷ F） | D23（journal 的角色与格式） 已定项 18 |
| 几何 | journal 最坏占用 | 8 | 268 435 456（65536 条 × 4096） | I-8.1（环几何够大）；D23（journal 的角色与格式） 已定项 19 |
| 几何 | journal 安全系数 F | 4 | 3 | I-8.1（环几何够大） |
| 几何 | 根环区域数 R | 1 | 3 | D22（单元原子性怎么合成） 已定项 2 |
| 几何 | 每区槽数 S | 1 | 8 | D22（单元原子性怎么合成） 已定项 2 |
| 几何 | 素数步长 P | 4 | 3 | D22（单元原子性怎么合成） 已定项 2 |
| 几何 | 根环 chunk | 4 | 1 048 576 <!-- format-const: ROOT_RING_CHUNK_BYTES = 1048576 --> | D22（单元原子性怎么合成） 已定项 2 |
| 几何 | 根环起点（16 KiB 槽号；区域公式的基址） | 8 | 64 | D22（单元原子性怎么合成） 已定项 16 |
| 几何 | 根环逐区域设备身份 | 12 | 0 / 1 / 0 | D2（RAID 条带策略） 已定项 7 |
| 几何 | 条带宽度上界 w_max | 1 | 4 | D2（RAID 条带策略） 已定项 6 / 已定项 18 |
| 几何 | 组大小 g | 1 | 4（运行时夹取 min(g, 可写设备数) = 2） | D2（RAID 条带策略） 已定项 8 / 已定项 18 |
| 几何 | 映射来源（重建纪元 + 权威态摘要） | 24 | 全 0 | D18（块里携带什么信息） day-1 留位 |
| 几何 | 单元区起始槽号 | 8 | 50176 | D3（空间分配） 已定项 10 ④ |
| 几何 | mkfs 时的 io_min | 4 | 512 | D2（RAID 条带策略） 已定项 19 |
| 几何 | 固定结构槽距 | 4 | 4096 | D2（RAID 条带策略） 已定项 19 |
| 可调值 | T_time | 4 | 5000（单位毫秒，即 5 秒） | D16（发布语义） 已定项 5 |
| 可调值 | T_dirty | 8 | 2 GiB（挂载时有效值 = min(2 GiB, 环长 ÷ F) = 256 MiB，D23（journal 的角色与格式） 已定项 19 ③） | D16（发布语义） 已定项 5 |
| 可调值 | 整理低水位 / 高水位 / 停止线 | 8 × 3 | 0 = 内置默认 | D26（后台整理与放置回收） 已定项 1；D22（单元原子性怎么合成） 已定项 15 ④ |
| 运行时 | journal tail（jsn 的 48 位计数器） | 8 | mkfs 0，暖机后 1、2，新池新建文件发布后 3 | D23（journal 的角色与格式） 已定项 3 / 已定项 18 |
| 运行时 | journal 实例代号 | 4 | mkfs 0，第一次可写挂载取号后 1 | D23（journal 的角色与格式） 已定项 9 / 已定项 16 |
| 运行时 | 回退下界 F | 8 | 0（新池新建文件里没有抬 F） | D16（发布语义） 已定项 1「生效」；D22（单元原子性怎么合成） 已定项 9 |

合计 **489 字节**（<!-- format-const: SYSTEM_CONFIGURATION_BYTES = 489 stale=合计 **481 -->），4096 槽内余 3607（按此前的 512 槽算只余 23），跨 1 扇区、撕裂态 0。2026-09-13 起这张表是 D22（单元原子性怎么合成） 已定项 9 与已定项 15 的定案，2026-09-14 用户定案加四个字段（写入者身份 20 + 校验和算法标识 1 + mkfs 时的 physical_block_size 4 + 扩展点声明值 N 4 = 29，452 → 481）；主密钥槽 80 字节的内部构成随 D9（加密） 开加密时定。

**系统配置字段表的偏移**：行序即盘上顺序，偏移按行累加，从槽首字节起算，整数字段一律小端（checker 读整数用 `from_le_bytes`，`crates/singlefs-checker/src/lib.rs` 第 228–241 行的 `read_u16` / `read_u32` / `read_u64`）：magic 0 / 格式版本 4 / feature bits 6 / fsid 102 / 写入者身份 118 / 校验和算法标识 138 / 本盘设备号 139 / 设备数 devs 143 / 槽世代号 147 / 整槽校验和 155 / 系统配置自身 MAC 187 / nonce 水位 203 / KDF 标识 215 / 加密类型 219 / MAC 长度声明 220 / 主密钥槽 221 / 节点大小 301 / 单元大小 305 / 落点粒度 309 / 位置条目宽度 313 / mkfs 时的 physical_block_size 317 / 扩展点声明值 N 321 / journal 环起点 325 / journal 环长度 333 / journal 最大记录字节 341 / journal 在飞记录数上限 345 / journal 最坏占用 349 / journal 安全系数 F 357 / 根环区域数 R 361 / 每区槽数 S 362 / 素数步长 P 363 / 根环 chunk 367 / 根环起点 371 / 根环逐区域设备身份 379、383、387（每区 4 字节）/ 条带宽度上界 w_max 391 / 组大小 g 392 / 映射来源 393 / 单元区起始槽号 417 / mkfs 时的 io_min 425 / 固定结构槽距 429 / T_time 433 / T_dirty 437 / 整理低水位 445、高水位 453、停止线 461 / journal tail 469 / journal 实例代号 477 / 回退下界 F 481，合计 489。checker 自己读的那几个偏移——6、102、139、143、147、155、219、317、325、333、361、362、363、367、371、379、383、387、417、429、469、477、481——现查 `crates/singlefs-checker/src/lib.rs` 第 195–203 行、第 224 行、第 278–280 行与 `crates/singlefs-checker/src/image.rs` 第 245–304 行，逐个与这里按行累加的结果相同；没有一处不一致。

**系统配置槽 0 与槽 1 的位置**：槽宽都是 4096；槽 0 在设备内偏移 0，槽 1 在设备内偏移 = 固定结构槽距（偏移 429 起那 4 字节，小端，第一版 4096）。槽距住在槽自己里，读槽 1 之前要先有一个读得出槽距的槽 0。读法照 D22（单元原子性怎么合成） 已定项 16：逐盘先读槽 0；「可用」的槽 = 自证过（magic、整槽校验和、格式版本、incompat 位、加密类型都过）且几何字段都在这一版读者收的范围里。槽 1 的偏移按这个次序取第一个有的：① 这块盘的槽 0 可用时，取它自述的槽距；② 否则取池里按 `reader.devices()` 次序第一块槽 0 可用的盘自述的槽距；③ 全池没有一个可用的槽 0 时逐档：在 4096 到 1 MiB − 4096 之间每 512 字节一档各读一槽，只收自证过、且自述的槽距正好等于所在偏移的槽，几档都收得到时取世代号最大的，世代号相同取偏移小的；读到自证过而池级字段越界的槽整池拒，incompat 位不认得的槽不可择、不核偏移。③ 这一格池级 checker 与实现已统一成逐档：checker 是 `crates/singlefs-checker/src/image.rs` 第 631 行 `system_configuration_slot_readings` 调第 583 行 `slot_one_read_at_every_slot_spacing_the_format_allows`，实现是 `crates/singlefs-core/src/recovery.rs` 第 1039 行 `choose_system_configuration` 调第 967 行 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`；GPU 判器里恢复那一份是 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 的 `slot_one_by_trying_every_spacing`，池级 checker 那一份是 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `checker_slot_one_by_reading_every_spacing`。两边在同一份镜像上一起判的用例是 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs`。E170（O3规约执行器第一版） 第 1 次重跑的执行器照同一句读（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/system_configuration.py` 的 `choose_system_configuration`、`slot_one_by_trying_every_spacing`）；第一次那一份（`research/e7-index-bench/src/bin/e170_o3_spec_executor/`）是暂代读法（收到第一个就停、不比世代号），留档不改。mkfs 写元数据之前把每块盘 [0, 根环基址) 整段写 0（D22（单元原子性怎么合成） 已定项 8 第 3 条），用过的盘上重做 mkfs 不留旧池的系统配置槽。

**根环参数一览**：R = 3、S = 8、P = 3、chunk = 1 MiB、基址 1 MiB、槽宽 512、槽距 4096、区域归属 **0 / 1 / 0**（第一版写死，不是 mkfs 参数——D2（RAID 条带策略） 已定项 7 与 D22（单元原子性怎么合成） 已定项 16 第 4 句，2026-09-14 用户定案）。
mkfs 把第 0 代根写进三个区域各自的槽 0，一共 **3 份**；第 n 次发布写区域 `n mod 3` 的槽 `(n div 3) mod 8`。
⚠️ **根槽不镜像**：一次发布只写一个槽（D2（RAID 条带策略） 已定项 15 正文逐字「一次 txg 只写一个槽」），
冗余来自另外两个区域里的前两代加 journal 的新根段（D23（journal 的角色与格式） 已定项 15）；掉一块盘最坏回退 1 代，是 E87（固定结构的放置） 钉成单测的数。
**暖机空发布恒 2 次、新池新建文件 txg 恒 3**（`WARM_UP_EMPTY_PUBLISHES` 与 `NEW_POOL_FILE_CREATION_TXG` 登记在八；条款 D22（单元原子性怎么合成） 已定项 16 第 5 句）：归属 0 / 1 / 0 下 txg 1 落盘 1、txg 2 落盘 0，两次正好覆盖两块盘。

```

**出处 `.claude/kb/layout/01-first-txn.md:168-195`（整段抄，未转述）**

```markdown
## 二、数据这一侧

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 2。

| 段 | 字段 | 宽度 | 新池新建文件里的值 | 指向 | 状态 |
|---|---|---|---|---|---|
| 数据单元 | 占盘长度恒 32768（含头） | 32768 | 槽 50180–50181，两盘各一份 | D4（校验和位置） 已定项 5 / 已定项 7 | 已定 |
| 单元头 | magic | 4 | 4 字节 ASCII `SFSU`，十六进制 53 46 53 55，从偏移 0 起按此顺序排（字节串，不涉字节序；码 1 / 2 / 3 三类单元同一个值）；取值出处 `crates/singlefs-core/src/unit.rs` 第 14 行 `UNIT_MAGIC`，checker 自写的一份 `crates/singlefs-checker/src/lib.rs` 第 194 行 | D18（块里携带什么信息） 已定项 7 | 已定 |
| 单元头 | 格式版本 | 2 | 1 | D18（块里携带什么信息） 已定项 7 | 已定 |
| 单元头 | 单元类型标签（共同前缀偏移 6；登记表的码） | 1 | 1 | D18（块里携带什么信息） 已定项 11 | 已定 |
| 单元头 | flag 位（偏移 7，第一版恒 0，非 0 判损坏） | 1 | 0 | D18（块里携带什么信息） 已定项 11 | 已定 |
| 单元头 | extent 声明长度（原点 = 含预留位的头末端 134，上界 32768 − 134） | 2 | 文件字节数 | D4（校验和位置） 已定项 6；D18（块里携带什么信息） 已定项 7 / 已定项 16 | 已定 |
| 单元头 | 头校验和：CRC32C 4 + 28 零，覆盖偏移 0 到 104，自身按 0 参与 | 32 | 算出来的 | D18（块里携带什么信息） 已定项 7 / 已定项 17；I-2.4（头校验和覆盖范围） | 已定 |
| 单元头 | 五元组 · 单元类型标签副本（偏移 42，== 偏移 6） | 1 | 1 | D18（块里携带什么信息） 已定项 3 / 已定项 11 / 已定项 18 | 已定 |
| 单元头 | 五元组 · 树 ID（偏移 43） | 8 | 65536（extent 树） | D18（块里携带什么信息） 已定项 3 / 已定项 11 | 已定 |
| 单元头 | 五元组 · 对象 ID（偏移 51） | 8 | 1（inode 号） | D18（块里携带什么信息） 已定项 3 / 已定项 11 | 已定 |
| 单元头 | 五元组 · 对象出生代（偏移 59） | 8 | 3（= 发布它的 checkpoint_txg） | D18（块里携带什么信息） 已定项 3 / 已定项 11 | 已定 |
| 单元头 | 五元组 · 锚点偏移（偏移 67） | 8 | 0 | D18（块里携带什么信息） 已定项 3 / 已定项 11 | 已定 |
| 单元头 | 诞生代号（birth txg），偏移 75 | 8 | 3（= 发布它的 checkpoint_txg，D16（发布语义） 已定项 6） | D18（块里携带什么信息） 已定项 7 | 已定 |
| 单元头 | fsid（偏移 83） | 8 | 系统配置 fsid 的低 8 字节 | D18（块里携带什么信息） 已定项 7 | 已定 |
| 单元头 | 写序（实例代号 4 + 事务号低 48 位），偏移 91 | 10 | (1, 1) | D18（块里携带什么信息） 已定项 7；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1；D23（journal 的角色与格式） 已定项 16 | 已定 |
| 单元头 | 载荷 CRC（只对码 1，偏移 101） | 4 | CRC32C，覆盖偏移 105 到 32767（含 29 字节预留位） | D18（块里携带什么信息） 已定项 7；C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据） 码 1 那一半 | 已定 |
| 单元头 | nonce 代号、MAC 与算法类型的预留位（偏移 105，算进头 ⇒ 头 134） | 12 + 16 + 1 <!-- format-const: NONCE_MAC_ALGORITHM_RESERVED_BYTES = 29 --> | 全 0 | D9（加密） 已定项 10；D18（块里携带什么信息） 已定项 14 / 已定项 16（算法类型那 1 字节 2026-09-14 用户定案加） | 已定（第一版不写值） |
| 单元净荷 | 用户数据，声明长度之后补齐到单元末尾且补齐恒为 0 | 32768 − 134 | 文件内容 + 0 | I-2.3（补齐字节为零且参与校验和）；D4（校验和位置） 已定项 4 | 已定 |
| 分配记录 | 每盘一条：key (设备身份, 槽号 50180)、value (跨度 2, 分配代 3) | 20 × 2 | 见五 | D3（空间分配） 已定项 7 / 已定项 11 | 已定 |

⚠️ 类身份段末尾 105（`format-const: DATA_UNIT_HEADER_BYTES`）是头校验和的端点；含 29 字节预留位的头是 **134** <!-- format-const: DATA_UNIT_PAYLOAD_OFFSET = 134 -->，载荷从 134 起（D18（块里携带什么信息） 已定项 16）。

```

**出处 `.claude/kb/layout/01-first-txn.md:196-247`（整段抄，未转述）**

```markdown
## 三、指向数据单元的指针（住在父节点里）

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 3（extent 树上段叶条目里标签 2 的那一份载荷）。段序与偏移是 D19（块指针的结构与宽度预算） 已定项 11 的定案，行序即盘上顺序。

| 字段（偏移） | 宽度 | 新池新建文件里的值 | 指向 | 状态 |
|---|---|---|---|---|
| MAC（0） | 16 | 全 0 | D9（加密） 已定项 2；D19（块指针的结构与宽度预算） 已定项 3（定宽留位） | 已定（第一版留位） |
| nonce（16） | 12 | 全 0 | D9（加密） 已定项 4；D19（块指针的结构与宽度预算） 已定项 3 | 已定（第一版留位） |
| 算法类型（28） | 1 | 0（未加密） | D9（加密） 已定项 12 第 4 项；D19（块指针的结构与宽度预算） 已定项 7；登记表 D22（单元原子性怎么合成） 已定项 17 | 已定（第一版留位） |
| 压缩算法码（29） | 1 | 0（不压缩；登记表在 D9（加密） 已定项 14） | D9（加密） 已定项 13 硬要求 2；D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11（2026-09-14 用户定案） | 已定（第一版留位） |
| 压后长度（30） | 2 | 0（码 0 时恒 0，非 0 判该指针损坏） | D9（加密） 已定项 13 硬要求 2；D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11（2026-09-14 用户定案） | 已定（第一版留位） |
| extent 距起点偏移（32） | 2 | 0 | D9（加密） 已定项 12 第 2 项；D4（校验和位置） 已定项 7 | 已定（**第一版恒 0**，切分是预留特性） |
| 出生树 ID（34） | 8 | 65536（第一版无跨头共享，就是引用它的树） | D19（块指针的结构与宽度预算） 已定项 7 | 已定 |
| 出生 txg（42） | 8 | 3（= 被指单元的诞生代号） | D19（块指针的结构与宽度预算） 已定项 7 | 已定 |
| 位置条目 ×2 副本（50）：设备 ID 4 + 物理偏移 6（**16 KiB 槽号**）+ 密文校验和 4，按设备身份升序 | 14 × 2 | (0, 50180, CRC32C 整单元) 与 (1, 50180, 同) | D19（块指针的结构与宽度预算） 已定项 4 / 已定项 2 / 已定项 11；D2（RAID 条带策略） 已定项 10 | 已定 |
| 子节点的校验和（Merkle） | 随算法 | 加密关闭时就是位置条目里那 4 字节 CRC32C，不另开字段；第一版自验证强度 32 位 | D4（校验和位置） 已定项 1 / 已定项 8 | 已定 |
| 映射 key 的其余段（78） | 10（指向码 1：写序）/ 8（指向码 2 / 码 3：实例代号 4 + 出生序号 4） | 写序 (1, 1)；码 2 / 码 3 的出生序号按 D19（块指针的结构与宽度预算） 已定项 9（从 0 起、树内码 2 / 码 3 共用计数） | D19（块指针的结构与宽度预算） 已定项 8 / 已定项 9 | 已定 |

⇒ 指针宽：指向码 1 的 **88** = 头部 50 + 位置条目 14 × 2（`format-const: LOC_ENTRY`）+ 写序 10；指向码 2 / 码 3 的 **86** = 头部 50 + 位置条目 14 × 2 + 实例代号 4 + 出生序号 4（D19（块指针的结构与宽度预算） 已定项 7、已定项 8、已定项 11；头部 2026-09-14 从 47 变 50，多的 3 字节是压缩算法码 1 + 压后长度 2）。 <!-- format-const: DATA_POINTER_BYTES = 88 --> <!-- format-const: NODE_POINTER_BYTES = 86 -->

**位置条目（14 字节）的内部**：行序即盘上顺序，偏移按行累加：设备 ID 0（4 字节）/ 16 KiB 槽号 4（6 字节）/ 密文校验和 10（4 字节），整数小端。一个指针里第一条副本从指针偏移 50 起、第二条从 64 起（头部 50 + 14）；journal 点名项里第一条在点名项偏移 0、第二条在 14。现查 checker：`crates/singlefs-checker/src/image.rs` 第 634–640 行 `location_at`（设备 ID 在 +0、槽号在 +4、校验和在 +10）与第 673、687 行的 `location_at(bytes, 50)` / `location_at(bytes, 64)`，`crates/singlefs-checker/src/lib.rs` 第 806–814 行点名项的 `location(0)` / `location(14)`；与这里相同。

⚠️ **2026-09-06 起这些位置条目是「提示」不是权威**（D19（块指针的结构与宽度预算） 已定项 5：
位置权威取混合）。宽度与段划分一个字节没变，变的是读到它之后的地位——
可以直接去那个落点试，但它可能过期；**释放判定一律走中央映射（本文件第三·二节），不许按提示**。

### 三·二、中央映射条目（2026-09-06 新增，D19（块指针的结构与宽度预算） 已定项 5）

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 4。

| 字段 | 宽度 | 新池新建文件里的值 | 指向 | 状态 |
|---|---|---|---|---|
| key：版本身份（码 1） | 27 | 类标签 1 + 出生树 65536 + 出生 txg 3（= 诞生代号）+ 写序 (1, 1)（与数据单元头里的写序同值） | D19（块指针的结构与宽度预算） 已定项 6 | 已定（2026-09-11 用户定案：按类复用写序键） |
| key：版本身份（码 2 / 码 3） | 27 | 类标签 + 出生树 + 出生 txg 3 + 实例代号 1 + 出生序号 = 25 字节，**末尾补零到 27**（D19（块指针的结构与宽度预算） 已定项 10，2026-09-14 用户定案改写；此前是「按类变长、不补齐」） | D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10 | 已定（2026-09-14 用户定案） |
| key 各段从引用侧哪里来 | — | 类标签由查找路径给；出生树、出生 txg、写序由指针带（三段与数据单元头同值）；指向码 2 / 码 3 的指针另带实例代号 + 出生序号 | D19（块指针的结构与宽度预算） 已定项 8 | 已定（2026-09-12 用户定案） |
| value：位置条目 ×`w` 副本 | 14 × 2 | 与三那一节的两条位置条目相同，按设备身份升序 | D19（块指针的结构与宽度预算） 已定项 4 / 已定项 5 / 已定项 11 | 已定 |
| 条目自带的 `seq` | — | 映射不走 write buffer，没有这一段（D8（核心索引结构） 已定项 10：第一版记账也直落叶） | D8（核心索引结构） 已定项 7 / 已定项 10 | 已定 |
| 映射的射程 | — | 码 1、码 2、码 3 全部进映射；映射树自己的节点、树表单元、实例表单元自举豁免，它们父指针里的位置条目是权威 | D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12 | 已定（2026-09-12 用户定案；day-1 写出 2026-09-13 重申） |
| 映射树的根 | 86 | 住根记录（第 11 个字段），不进树表；新池新建文件指 t11（槽 50251） | D19（块指针的结构与宽度预算） 已定项 11 | 已定（2026-09-13 用户定案，宽度 2026-09-14 随指针头部加压缩 3 字节） |

⇒ 条目**一律 55** = key 27 + value 28（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10，2026-09-14 用户定案回到定宽；此前码 2 / 码 3 是 53）；新池新建文件 10 条（t1 码 1 一条；t2..t10 码 2 / 码 3 九条：extent 根、inode 叶容器、inode 根、分配记录树五个节点、记账根，D19（块指针的结构与宽度预算） 已定项 8），声明长度 10 × 55 = **550**；映射树 1 个 16 KiB 节点（t11），节点头 86 + 2 × 27 + 29 = **169**（key 宽 27，条目区从 169 起）。 <!-- format-const: MAPPING_ENTRY_BYTES = 55 --> <!-- format-const: CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES = 169 -->

**它为什么进新池新建文件**：中央映射是解引用与释放判定的**唯一入口**
⇒ 新池新建文件写出的每个数据单元都要有一条条目。
E109（位置权威三臂的运行时代价） 量到 D25（目标负载优先级） 主负载（8 叶）下这一份是
**66 048 字节**（映射树叶节点 1 + 内部 3 个 × 16384 + 一个根槽 512）。
⚠️ 这个数取 key 33 字节那一档；E109（位置权威三臂的运行时代价） 同一份产物里 key 40 字节那一档
逐字也是 `map_height=4 … map_bytes=66048`（扇出 267 → 239）。D19（块指针的结构与宽度预算） 已定项 6 取按类复用写序键：映射条目一律 55 字节、映射叶扇出 296，
E134（映射 key 格式代价·槽基线与两条新臂） 算出映射树在 E109（位置权威三臂的运行时代价） 的池规模上仍是 4 层；
两份 kb 对这个数能不能引的分歧仍挂在 C253（66 048 那笔定额两份 kb 说法相反）。
新池新建文件只有 1 叶、树高 1，那笔定额在这里是 16 384。

```

**出处 `.claude/kb/layout/01-first-txn.md:248-288`（整段抄，未转述）**

```markdown
## 四、文件元数据（inode）

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 3。

D8（核心索引结构） 已定项 6（2026-09-05）定了整段：inode 树的叶是码 3 打包记录单元（打包记录类型 2），内部节点是码 2、根恒为码 2，记录定长 140。新池新建文件写一个码 2 根（t4）、一个码 3 叶（t3）、一条记录、一条水位统计量。

| 段 | 字段 | 宽度 | 新池新建文件里的值 | 指向 | 状态 |
|---|---|---|---|---|---|
| inode 树根（码 2） | 单元类型标签恒 = 2；类身份段 = key 宽 1 + 树 ID + 层级 + key 区间 [1, 1] + 诞生代号 + fsid + 写序 4（只实例代号）+ 出生序号 4 + 载荷 CRC 4 + 预留 2 + 条目数 2 + 条目宽 2（偏移表见 D18（块里携带什么信息） 已定项 18） | 头 **131** <!-- format-const: INODE_TREE_ROOT_INDEX_NODE_HEADER_BYTES = 131 --> = 86 + 2 × 8 + 29（明文头末尾 102，头校验和到 101；含预留位 29 的头 131，条目区从 131 起） | 树 ID 65537、层级 1、区间 [1, 1]、诞生代号 3、写序 1、key 宽 8、条目数 1、条目宽 120 | D8（核心索引结构） 已定项 6（根恒码 2、带 key 区间）/ 已定项 11；D18（块里携带什么信息） 已定项 7 / 已定项 12 / 已定项 16 / 已定项 18 | 已定 |
| inode 树内部节点条目 | 分隔 key | 8 | 1 | D8（核心索引结构） 已定项 6 | 已定 |
| inode 树内部节点条目 | 身份引用：出生树 8 + 打包记录类型 2（0 = 子是码 2，2 = 子是码 3）+ 容器号 8 + 容器出生代 8 | 26 | (65537, 2, 1, 3) | D8（核心索引结构） 已定项 6；字段形态是 D18（块里携带什么信息） 已定项 11 的身份引用 | 已定 |
| inode 树内部节点条目 | 子指针 | 86 | 位置条目 (0, 50242) / (1, 50242) | D19（块指针的结构与宽度预算） 已定项 4 / 7 / 8 / 11；D8（核心索引结构） 已定项 6 | 已定 |
| inode 叶容器头 | 类身份段（标签 3、出生树、类型 = 2、容器号 = 1、出生代、记录数 = 1、记录宽 = 140、诞生代号、fsid、载荷校验和、写序、出生序号）；含 29 字节预留位的头 136，记录区从 136 起 | 107 + 29 | 出生树 65537、容器号 1、容器出生代 3、记录数 1、记录宽 140、诞生代号 3、写序 (1, 1)、出生序号 0（该树该 checkpoint 第一个码 2 / 码 3 单元） | D18（块里携带什么信息） 已定项 11 / 已定项 16；容器号规则与类型 2 的启用是 D8（核心索引结构） 已定项 6；序号 D19（块指针的结构与宽度预算） 已定项 9 | 已定 |
| inode 叶容器 | 扩展点 | N = 0 | 带，紧贴单元尾部，第一版 0 字节 | D21（权威态与派生态的分界） 已定项 8 | 已定（2026-09-13 用户定案） |
| inode 记录（偏移 0） | inode | 8 | 1 | D8（核心索引结构） 已定项 6 | 已定 |
| inode 记录（偏移 8） | 对象出生代 | 8 | 3 | D8（核心索引结构） 已定项 6；理由连带 D9（加密） 已定项 6 / D18（块里携带什么信息） 已定项 10 的改写 | 已定 |
| inode 记录（偏移 16） | locality_id（第一版取 0） | 8 | 0 | D8（核心索引结构） 已定项 6 | 已定 |
| inode 记录（偏移 24） | mode / uid / gid / nlink | 16 | 0o100644 / 0 / 0 / 1 | D8（核心索引结构） 已定项 6 | 已定 |
| inode 记录（偏移 40） | size / blocks / rdev | 24 | 文件字节数 / ⌈文件字节数 ÷ 512⌉（512 字节块计；新池新建文件是 6） / 0 | D8（核心索引结构） 已定项 6 | 已定 |
| inode 记录（偏移 64） | atime / mtime / ctime 秒 | 24 | 写入时刻（发布参数，可复现） | D8（核心索引结构） 已定项 6 | 已定 |
| inode 记录（偏移 88） | 改动计数 | 8 | 3（= 发布它的 checkpoint_txg） | D8（核心索引结构） 已定项 6 | 已定 |
| inode 记录（偏移 96） | atime / mtime / ctime 纳秒 | 12 | 写入时刻的纳秒部分（今天的参数只带秒，写 0） | D8（核心索引结构） 已定项 6 | 已定 |
| inode 记录（偏移 108 / 112 / 120） | 填充 / flags / 预留，恒 0 | 4 + 8 + 20 | 0 | D8（核心索引结构） 已定项 6 | 已定 |
| 单元类型标签的值域，以及打包记录单元算哪一类 | 1（登记表；新池新建文件写码 1 / 2 / 3） | 1 | 1 / 2 / 3 | D18（块里携带什么信息） 已定项 11 | 已定 |
| 记账 | 「inode 号水位」统计量一行（key 22 + value 8 + seq 4），15 行里的第 15 行 | 34 | 值 2（下一个可用号），key (12, 65537, 0xFFFF_FFFF, 3) | D5（快照 / 空间记账机制） 已定项 4 第 12 项 / 已定项 5 / 已定项 9 / 已定项 10；D8（核心索引结构） 已定项 7 / 已定项 10 | 已定 |

**这一节仍写不出来的**：`deleted_inodes` 的形态（C118（`deleted_inodes` 树的形态无落点），新池新建文件不删东西、不写它）；记录格式与统计量登记表两个冻结组件的 spec 文件（C119（冻结组件没有 spec 文件））。

### 四·二、extent 树（2026-09-10 新增，此前这张表里没有它）

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 3。

| 段 | 字段 | 宽度 | 新池新建文件里的值 | 指向 | 状态 |
|---|---|---|---|---|---|
| extent 树节点（码 2） | 头，同 inode 树根的字段集合；**key 区间无条件带**（D18（块里携带什么信息） 已定项 2，2026-09-14 用户定案改写成全部码 2 节点都带，不再按树分） | **163** <!-- format-const: EXTENT_TREE_INDEX_NODE_HEADER_BYTES = 163 --> = 86 + 2 × 24 + 29 | 树 ID 65536、层级 0、区间 [(0, 0, 0), (0, 142, 2^64 − 1)]（上段第 0 片叶按位置罩 inode 0..142，D8（核心索引结构） 已定项 14）、key 宽 24、条目数 1、条目宽 113 | D18（块里携带什么信息） 已定项 2 / 已定项 7 / 已定项 12 / 已定项 16；D8（核心索引结构） 已定项 11 / 已定项 14 | 已定（2026-09-14 用户定案） |
| extent 上段叶条目 | key = (locality_id 8, inode 8, offset 8)，上段条目取 (0, inode, 0)（D8（核心索引结构） 已定项 14）；下段 key 的 offset 段是文件字节偏移（D8（核心索引结构） 已定项 3，2026-09-23 定），第二个数据单元起是 32634，新池新建文件不建下段 | 24 | (0, 1, 0) | D8（核心索引结构） 已定项 3 / 已定项 14 | 已定 |
| extent 上段叶条目 | 标签（0 = 没有单元、1 = 下段根指针、2 = 内联的那一个数据指针） | 1 | 2 | D8（核心索引结构） 已定项 14 | 已定 |
| extent 上段叶条目 | 载荷：指向数据单元的指针（标签 2） | 88 | 三那一节 | D19（块指针的结构与宽度预算） 已定项 4 / 已定项 7 / 已定项 8；D8（核心索引结构） 已定项 14 | 已定 |

⇒ extent 叶记录（下段叶的条目）24 + 88 = **112**（D19（块指针的结构与宽度预算） 已定项 7 / 8），新池新建文件不写：只有一个数据单元的文件不建下段（D8（核心索引结构） 已定项 14）。新池新建文件写的是上段叶条目 **113** = key 24 + 标签 1 + 载荷 88（`format-const: EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES`），1 条，树高 1，上段根兼叶（t2，落槽 50240；高度 1 的树根就是叶、不留缓冲，D11（索引节点要不要留消息缓冲区） 已定项 7）。 <!-- format-const: EXTENT_LEAF_RECORD_BYTES = 112 -->

```

**出处 `.claude/kb/layout/01-first-txn.md:289-312`（整段抄，未转述）**

```markdown
## 五、空间记账与分配

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 2 记下、步 3 落盘。

| 段 | 字段 | 宽度 | 新池新建文件里的值 | 指向 | 状态 |
|---|---|---|---|---|---|
| 分配记录 | 逐落点谓词的语义 | — | — | D3（空间分配） 已定项 1 | 已定 |
| 分配记录 | 住一棵独立 keyspace 的 btree，key = 落点、value = 跨度 + 分配代 | — | 树 ID 65538，5 个节点（t5..t9：两片叶、两个层级 1 节点、层级 2 的根；按绝对槽号按位置寻址，D8（核心索引结构） 已定项 14） | D3（空间分配） 已定项 3 / 已定项 11 | 已定 |
| 分配记录 | 落点粒度 16384；key = (设备身份 4, **16 KiB 槽号** 6)；value = (跨度段 2，最高位是已释放标志, 分配代或释放代 8) | 10 + 10 | 28 条：槽 50176（跨 2）、50178（1）、50180（2）、50240（1）、50242（2）、50244（1）、50245（1）、50246（1）、50247（1）、50248（1）、50249（1）、50250（1）、50251（1）、50252（1），每条两盘各一（设备 0 的 14 条住 t5、设备 1 的 14 条住 t6）；mkfs 写出的 50176 那条分配代 0；50178（第 0 版树表）被这次发布重写树表换下，条目改写成已释放（跨度段最高位）、释放代 3（2026-09-17 起，E142（新池新建文件的干跑） 第十次跑）；其余 3（= checkpoint_txg，D3（空间分配） 已定项 7 逐字「存完整 checkpoint 号」）。⚠️ 槽 50241 空着——t2 占 50240 之后 bump 游标停在 50241，而 t3 要 32768 对齐的两槽（D3（空间分配） 已定项 10 ⑤） | D3（空间分配） 已定项 7 / 已定项 11 | 已定 |
| 分配记录树节点 | 码 2 头 = 86 + 2 × 10 + 29 | **135** <!-- format-const: ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES = 135 --> | 五个节点都是树 ID 65538、key 宽 10：t5 / t6 层级 0、条目数 14、条目宽 20、声明长度 280、区间 [(盘, 49532), (盘, 50343)]；t7 / t8 层级 1、条目数 1、条目宽 96、区间 [(盘, 0), (盘, 137227)]（罩 812 × 169 = 137228 槽）；t9 层级 2、条目数 2、条目宽 96、声明长度 192、区间 [(0, 0), (0xFFFFFFFF, 2^48 − 1)] | D8（核心索引结构） 已定项 11 / 已定项 14；D18（块里携带什么信息） 已定项 16 | 已定 |
| 分配记录树内部条目 | key 10（孩子按位置罩的那一段的起点）+ 子指针 86 | 96 | t7：(0, 49532) → t5；t8：(1, 49532) → t6；t9：(0, 0) → t7、(1, 0) → t8 | D8（核心索引结构） 已定项 14 | 已定 |
| 记账树节点 | 走 COW，挂父指针下 | — | 树 ID 65539，1 个节点（t10） | D22（单元原子性怎么合成） 已定项 3 | 已定 |
| 记账树节点 | 节点带 key 区间（`min_key` + `max_key`） | 44 | 首末两条 key | D18（块里携带什么信息） 已定项 2 | 已定 |
| 记账树节点 | 码 2 头 = 86 + 2 × 22 + 29 | **159** <!-- format-const: ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES = 159 --> | 树 ID 65539、层级 0、key 宽 22、条目数 15、条目宽 34 | D8（核心索引结构） 已定项 11；D18（块里携带什么信息） 已定项 16 | 已定 |
| 记账 key | `(统计量标签 2, 树 ID 8, 设备 4, 代 8)`；标签 = D5（快照 / 空间记账机制） 已定项 4 的行号，不带设备维的设备段 0xFFFF_FFFF、不带树维的树 ID 0；代 = checkpoint_txg 3 | 22 | **15 行**（key 升序）：已分配字节 (1, 0, 0, 3)、(1, 0, 1, 3)；空闲字节 (2, 0, 0, 3)、(2, 0, 1, 3)；不可回收字节 (3, 0, 0, 3)、(3, 0, 1, 3)；待删除但意图未完成占用 (4, 0, 0xFFFF_FFFF, 3)；defer 队列待释放 (5, 0, 0, 3)、(5, 0, 1, 3)；已承诺预留量 (6, 0, 0xFFFF_FFFF, 3)；碎片度 runs (10, 0, 0, 3)、(10, 0, 1, 3)（2026-09-14 起带设备维）；全空聚簇段数 (11, 0, 0, 3)、(11, 0, 1, 3)；inode 号水位 (12, 65537, 0xFFFF_FFFF, 3) | D5（快照 / 空间记账机制） 已定项 5 / 已定项 8 / 已定项 9 / 已定项 10；D16（发布语义） 已定项 6 | 已定 |
| 记账条目 | value = 完整值 | 8 | 已分配 278 528 / 278 528（17 个 16 KiB 槽 × 16384，含 mkfs 的）；空闲 3 472 605 184 / 3 472 605 184（(211968 − 17) × 16384，镜像 4 GiB 的预想）；不可回收 0 / 0；待删占用 0；defer 待释放 16 384 / 16 384（被换下的第 0 版树表那 1 槽，2026-09-17 起）；已承诺预留 98 386（D3（空间分配） 已定项 2 的式子：树表有 `deleted_inodes` 条目，2 × ((2 + 0 + 1) × 16384 + 41 × 1)）；**runs 4 / 4**（单元区空闲槽的连续段，逐设备：[50179]、[50182, 50239]、[50241]、[50253, 单元区末]）；全空聚簇段数 3310 / 3310（3312 段减 [50176, 50240) 与开放段 [50240, 50304)）；inode 号水位 2 | D5（快照 / 空间记账机制） 已定项 5 / 已定项 7 / 已定项 8 | 已定（空闲与段数随镜像大小） |
| 记账条目 | `seq` 段，住 value | 4 | 一律 1（直落叶，同 key 本次发布第一次写） | D8（核心索引结构） 已定项 7 / 已定项 10 | 已定 |
| 记账条目 | 整理触发量各一条完整值（碎片度 runs、全空聚簇段数），与其余统计量同形 | 同其余记账条目 | 写（「记账 key」一格 15 行里的 4 行：碎片度 runs 与全空聚簇段数各两行，逐设备） | D26（后台整理与放置回收） 已定项 6；D5（快照 / 空间记账机制） 已定项 8 | 已定 |
| 提交内生块 | 从聚簇段里分配：开放段 [50240, 50304)，bump 只在内存，挂载后新开一段；bump 次序 = 按树 ID 升序、树内先叶后根、中央映射树倒数第二、树表单元最末（D3（空间分配） 已定项 10 ⑤，2026-09-14 用户定案） | — | t2 到 t12 依次 50240、50242–50243、50244、50245、50246、50247、50248、50249、50250、50251、50252（50241 被 t3 的 32768 对齐跳过；t5..t9 是分配记录树的五个节点，树内先叶后根、同层按 key 升序） | D3（空间分配） 已定项 5 / 已定项 10 | 已定 |
| 准入 | 读数 = 已发布 − 在飞已批；合取式只登记在 D3（空间分配） 已定项 7，第一版 `N₀` = 0 | — | 新池新建文件恒过 | D28（挂载期承诺量） 已定项 2；D3（空间分配） 已定项 12 | 已定 |
| 用户数据落点政策 | — | — | 已选设备内槽号最小、32768 对齐、两槽都空、不在开放段与 `R` 内的空槽（新池新建文件 t1@50180） | D3（空间分配） 已定项 8 / 已定项 10 | 已定（2026-09-13） |

⇒ 记账条目 **34** = key 22 + value 8 + seq 4，15 行 ⇒ 声明长度 15 × 34 = **510**；分配记录 **20** = key 10 + value 10，每片叶 14 条 ⇒ 声明长度 **280**（两片叶共 28 条）。 <!-- format-const: ACCOUNTING_ENTRY_BYTES = 34 --> <!-- format-const: ALLOCATION_RECORD_BYTES = 20 -->

```

**出处 `.claude/kb/layout/01-first-txn.md:313-362`（整段抄，未转述）**

```markdown
## 六、journal

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 5。

| 字段 | 宽度 | 新池新建文件里的值 | 指向 | 状态 |
|---|---|---|---|---|
| 记录尺寸 | 4096 | 3 条：w1 偏移 0、w4 偏移 4096、t13 偏移 8192（记录 n 落 `(计数器 − 1) mod 槽数 × 4096`），各两盘一份 | D23（journal 的角色与格式） 已定项 12 / 已定项 18 | 已定 |
| 记录头（十个字段，不含事务边界、本次发布内序号、反向链、载荷校验和、新根段、fsid、MAC 七笔增量） | 78 | 见「记录头字段表」，合计 487 | D23（journal 的角色与格式） 已定项 4 | 已定 |
| 事务边界字段（一个事务可跨多条记录）：事务号 8 + 提交标记 1 | 9 | t13 (1, 1)；空发布 (0, 1)——0 保留给不承载事务的记录 | D23（journal 的角色与格式） 已定项 7 / 已定项 19 | 已定 |
| 本次发布内序号（紧跟提交标记；无符号 32 位，从 1 起） | 4 | w1、w4、t13 都是 1（各自那次发布只有一条记录；空发布也写 1）；一次发布切成 N 条记录时依次 1..N | D23（journal 的角色与格式） 已定项 4 / 已定项 14 注 1 | 已定（2026-09-23 用户定案，被攻过零轮） |
| 反向链 hash（CRC32C） | 4 | w1 写 0（**本实例第一条**——链按实例算，2026-09-14 用户定案改写，D23（journal 的角色与格式） 已定项 19 ②）；w4 = 750278362 = CRC32C(w1 的 487 字节头，header_csum 按零参与)；t13 = 2660362816 = CRC32C(w4 的头) | D23（journal 的角色与格式） 已定项 8 / 已定项 10 / 已定项 19；I-8.6（反向链算法） | 已定 |
| 载荷校验和（覆盖点名项数组，CRC32C） | 4 | 算出来的 | D23（journal 的角色与格式） 已定项 13 | 已定 |
| 新根段：树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4 | 364 | w1 / w4：树表 50178、映射 0、水位 65536、F 0、实例表指针指 50176（两盘各一份）、分配记录树根全零、卸载记号 0；t13：树表 50252、映射 50251、水位 65548（十二棵；只注册前八棵时 65544）、F 0、后三项同 w1 / w4；卸载记号 4 字节的编码 D23（journal 的角色与格式） 没写，装置照 D22（单元原子性怎么合成） 已定项 7 根记录 flags 的编码，推的，欠 C709（journal 新根段卸载记号的编码没有条款） | D23（journal 的角色与格式） 已定项 15 | 已定（2026-09-13 用户定案，两条指针 2026-09-14 各加 3） |
| fsid（新根段之后） | 8 | 系统配置 fsid 的低 8 字节，与单元头同口径 | D23（journal 的角色与格式） 已定项 4（2026-09-14 用户定案）；I-1.4（块头 fsid 一致） | 已定 |
| MAC（fsid 之后） | 16 | 全 0 | D9（加密） 已定项 12 第 1 项（2026-09-14 射程扩到自证单元）；D23（journal 的角色与格式） 已定项 4 | 已定（第一版留位） |
| 序号 `jsn`（含在记录头十个字段的 78 内） | 10 | t13：实例代号 1、计数器 3（暖机两条占 1、2） | D23（journal 的角色与格式） 已定项 9 | 已定 |
| 点名项 | 56 | 12 项（t1..t12 各一项，一项含两盘的位置条目）；构成见「点名项字段表」；4096 − 487 = 3609 ⇒ 一条记录最多装 64 项 | D23（journal 的角色与格式） 已定项 17 | 已定 |
| 环形几何、tail | — | 环长是 mkfs 参数、默认 768 MiB、约束环 ≤ 设备容量 ÷ 4（2026-09-14 用户定案）；tail 住系统配置、存 jsn 计数器 | D23（journal 的角色与格式） 已定项 2 / 已定项 3 / 已定项 18 / 已定项 19 | 已定 |
| 环放在两块盘上互为镜像 | — | — | D22（单元原子性怎么合成） 已定项 8 | 已定 |

**记录头字段表**（78 那十个字段抄 E23（journal 几何） 的表去掉 `tail_lsn`，加七笔已定增量；
`format-const: JOURNAL_HEADER_BYTES` 登记 487，十个字段的 78 另登记为 `JOURNAL_HEADER_TEN_FIELD_BYTES`）：

| 字段 | 宽 | 新池新建文件里的值 | 出处 |
|---|---|---|---|
| magic | 4 | 4 字节 ASCII `SFSJ`，十六进制 53 46 53 4A，从偏移 0 起按此顺序排（字节串，不涉字节序）；取值出处 `crates/singlefs-core/src/journal.rs` 第 20 行 `JOURNAL_MAGIC`，checker 自写的一份 `crates/singlefs-checker/src/lib.rs` 第 695 行 | D23（journal 的角色与格式） 已定项 4 |
| 类型 | 2 | 1（普通记录；登记表 D23（journal 的角色与格式） 已定项 1，D22（单元原子性怎么合成） 已定项 17） | D23（journal 的角色与格式） 已定项 1 的 A 条 |
| 算法类型 | 1 | 0 | D9（加密） 已定项 12 第 4 项 |
| 记录标志 | 1 | 1（位 0 = 本次发布末条；w1、w4、t13 都是一次发布只有一条记录，都写 1） | D23（journal 的角色与格式） 已定项 4、17 |
| 记录长度 | 4 | 4096 | D23（journal 的角色与格式） 已定项 1 的 B 条 |
| 点名项数 | 4 | 12 | D23（journal 的角色与格式） 已定项 17 |
| `jsn`（实例代号 4 + 计数器 6） | 10 | (1, 3)（暖机两条空记录占 (1, 1)、(1, 2)） | D23（journal 的角色与格式） 已定项 9 / 已定项 16 |
| checkpoint_txg | 8 | 3（暖机 1、2 之后） | D16（发布语义） 已定项 6 / 已定项 8 |
| nonce | 12 | 全 0 | D9（加密） 预留第 1 项 |
| 头校验和（`header_csum`：CRC32C 4 + 28 零，覆盖整条记录 `[0, 4096)` 含补齐、自身按零参与——2026-09-14 用户定案写死，D23（journal 的角色与格式） 已定项 13） | 32 | 算出来的 | D23（journal 的角色与格式） 已定项 4 / 已定项 13；D18（块里携带什么信息） 已定项 17 |
| 事务号 | 8 | 1（空发布 0） | D23（journal 的角色与格式） 已定项 7 / 已定项 19 |
| 提交标记 | 1 | 1 | D23（journal 的角色与格式） 已定项 7 |
| 本次发布内序号 | 4 | 1（这次发布只有这一条） | D23（journal 的角色与格式） 已定项 4 |
| 反向链 | 4 | t13：2660362816 = CRC32C(w4 的 487 字节头)；w4：750278362 = CRC32C(w1 的 487 字节头)；w1：0（本实例第一条） | D23（journal 的角色与格式） 已定项 8 / 已定项 10 / 已定项 19 |
| 载荷校验和 | 4 | 算出来的（字段保留，仍罩点名项数组；「头落了、载荷只落一半」第一版不单独分辨，D23（journal 的角色与格式） 已定项 13） | D23（journal 的角色与格式） 已定项 13 |
| 新根段（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4） | 364 | w1 / w4：树表 50178、映射 0、水位 65536、F 0、实例表指针指 50176（两盘各一份）、分配记录树根全零、卸载记号 0；t13：树表 50252、映射 50251、水位 65548（十二棵；只注册前八棵时 65544）、F 0、后三项同 w1 / w4（卸载记号的编码是推的，欠 C709（journal 新根段卸载记号的编码没有条款）） | D23（journal 的角色与格式） 已定项 15 |
| fsid | 8 | 系统配置 fsid 的低 8 字节 | D23（journal 的角色与格式） 已定项 4（2026-09-14 用户定案） |
| MAC | 16 | 全 0 | D9（加密） 已定项 12 第 1 项；D23（journal 的角色与格式） 已定项 4（2026-09-14 用户定案） |

合计 **487 字节**，4096 记录余 3609，装 64 个点名项。

**记录头字段表的偏移**：行序即盘上顺序，偏移按行累加，从记录首字节起算，整数字段一律小端：magic 0 / 类型 4 / 算法类型 6 / 记录标志 7 / 记录长度 8 / 点名项数 12 / jsn 16（实例代号 4 字节在 16，计数器 6 字节在 20）/ checkpoint_txg 26 / nonce 34 / 头校验和 46 / 事务号 78 / 提交标记 86 / 本次发布内序号 87 / 反向链 91 / 载荷校验和 95 / 新根段 99（树表单元指针 99、中央映射树根指针 185、树 ID 水位 271、回退下界 F 279、实例表指针 287、分配记录树根指针 373、卸载记号 459）/ fsid 463 / MAC 471，到 487 为止；前十个字段是 magic 到头校验和，合计 78。现查 checker：`crates/singlefs-checker/src/lib.rs` 第 696–704 行的偏移常量与第 778–833 行读的偏移（类型 4、记录长度 8、点名项数 12、jsn 的 16 与 20、checkpoint_txg 26），逐个与这里按行累加的结果相同；没有一处不一致。

**点名项字段表（56，D23（journal 的角色与格式） 已定项 17）**：位置条目 14 × 2（偏移 0，按设备身份升序）+ 单元类型标签 1（28）+ 出生树 8（29）+ 出生 txg 8（37）+ key 尾段 10（45：码 1 写序；码 2 / 码 3 实例代号 4 + 出生序号 4 + 补零 2）+ flags 1（55）。不另带载荷 CRC（位置条目各带 4 字节校验和）；重放从点名项直接凑出映射 key。

```

**出处 `.claude/kb/layout/01-first-txn.md:363-397`（整段抄，未转述）**

```markdown
## 七、发布（根记录与根槽）

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 5（mkfs 种下的第 0 代与树表单元第 0 版归步 1）。行序即盘上顺序（D22（单元原子性怎么合成） 已定项 7 的表序，两处的字段先后逐行相同），偏移按行累加，各字段的偏移数写在 D22（单元原子性怎么合成） 已定项 7 字段表之后那一句。

| 字段 | 宽度 | 新池新建文件里的值 | 指向 | 状态 |
|---|---|---|---|---|
| magic | 4 | 4 字节 ASCII `SFSR`，十六进制 53 46 53 52，从偏移 0 起按此顺序排（字节串，不涉字节序）；取值出处 `crates/singlefs-core/src/root_record.rs` 第 11 行 `ROOT_MAGIC`，checker 自写的一份 `crates/singlefs-checker/src/lib.rs` 第 193 行 | D22（单元原子性怎么合成） 已定项 7 | 已定 |
| fsid | 16 | 系统配置同值 | D22（单元原子性怎么合成） 已定项 7；I-1.4（块头 fsid 一致） | 已定 |
| flags | 4 | 0（位 0 卸载记号；新池新建文件里没有卸载） | D22（单元原子性怎么合成） 已定项 7 | 已定 |
| 实例代号 | 4 | mkfs 0，第一次可写挂载起 1 | D22（单元原子性怎么合成） 已定项 7；D16（发布语义） 已定项 6；D23（journal 的角色与格式） 已定项 16 | 已定 |
| `checkpoint_txg` | 8 | mkfs 0，暖机 1、2，新池新建文件发布后 3 | D22（单元原子性怎么合成） 已定项 7；D16（发布语义） 已定项 6 / 已定项 8 | 已定 |
| 树表单元指针 | 86 | mkfs 指 50178，发布后指 50252 | D22（单元原子性怎么合成） 已定项 7（指针头部 50 + 位置条目 14 × 2 + 实例代号 4 + 出生序号 4；D19（块指针的结构与宽度预算） 已定项 8 的自举豁免）| 已定 |
| 树 ID 水位 | 8 | mkfs 65536，发布后 **65548**（deadlist 树 ID 65543 day-1 注册，D5（快照 / 空间记账机制） 已定项 11；`deleted_inodes` 等四棵 65544..65547 同批；只注册前八棵时 65544） | D8（核心索引结构） 已定项 8 ② / 已定项 11；D22（单元原子性怎么合成） 已定项 7 | 已定 |
| 回退下界 F | 8 | mkfs 0，发布后 0（非空有效根不足 4 个，上限取最旧有效根 txg 0） | D16（发布语义） 已定项 1；D22（单元原子性怎么合成） 已定项 7 | 已定 |
| 自证校验和 | 32 | CRC32C 4 + 28 零，覆盖整槽 512 含补齐、自身按零参与 | D22（单元原子性怎么合成） 已定项 7；D18（块里携带什么信息） 已定项 17 | 已定 |
| 实例表单元指针 | 86 | 指 50176 | D22（单元原子性怎么合成） 已定项 7；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 | 已定 |
| 中央映射树根指针 | 86 | mkfs 与暖机全 0，发布后指 50251 | D19（块指针的结构与宽度预算） 已定项 11；D22（单元原子性怎么合成） 已定项 7 | 已定（2026-09-13 用户定案） |
| 分配记录树根指针 | 86 | mkfs、暖机与新池新建文件全 0；只有树表 0 条、写过行的那一版写非零，指写行那次发布写的那一片分配记录节点 | D22（单元原子性怎么合成） 已定项 7；D16（发布语义） 已定项 9 | 已定（2026-09-23 用户定案） |
| 算法类型 | 1 | 0（未加密） | D9（加密） 已定项 12 第 1 项（2026-09-14 射程扩到自证单元）；D22（单元原子性怎么合成） 已定项 7 | 已定（第一版留位） |
| nonce | 12 | 全 0 | 同上 | 已定（第一版留位） |
| MAC | 16 | 全 0 | 同上 | 已定（第一版留位） |
| 根槽落点 | 512 槽 | mkfs：区域 0 / 1 / 2 各自槽 0；发布：区域 `txg mod 3` 的槽 `(txg div 3) mod 8`，新池新建文件（txg 3）= 区域 0 槽 1 | D22（单元原子性怎么合成） 已定项 2 / 已定项 8 / 已定项 16 | 已定 |
| 实例表单元（第一片，mkfs 写出） | 码 3 打包记录类型 4，记录数 1（唯一一条是「无下一片」的链指针记录）；含预留位的头 136 <!-- format-const: PACKED_UNIT_RECORDS_OFFSET = 136 -->，记录区从 136 起 | 记录宽 88（2026-09-13 起，C304（实例表链指针记录装不下 83 宽的指针） 收口；2026-09-14 指针从 83 变 86 之后那 3 字节预留归零，记录宽仍 88，D18（块里携带什么信息） 已定项 11）；四元组 (0, 4, 0, 0)；写序 (0, 0) | D18（块里携带什么信息） 已定项 11 / 已定项 16；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2；D23（journal 的角色与格式） 已定项 16 | 已定 |
| 树表单元自己的头 | 码 2 索引节点头 **131** <!-- format-const: TREE_TABLE_UNIT_INDEX_NODE_HEADER_BYTES = 131 -->（86 + 2 × 8 + 29，key = 树 ID、key 宽 8） | 树 ID 段写 0、层级 0、key 宽 8、条目数 11、条目宽 200、声明长度 2200 | D18（块里携带什么信息） 已定项 7 / 已定项 16；D8（核心索引结构） 已定项 11 | 已定 |
| 树表条目 | **树 ID 8**（打头，= 码 2 条目的 key）+ 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8 + **头 ID 8** + 预留 76 | 200 | 第 1 版 **11 条**（按树 ID 升序）：(树 ID 65536, extent, 种类 1, 根 50240, 头 ID 65537)、(65537, inode, 2, 根 50244, 头 ID 65537)、(65538, 分配记录, 3, 根 50249, 头 ID 0)、(65539, 记账, 4, 根 50250, 头 ID 0)、(65541, livelist, 6, 根指针零, 头 ID 0)、(65542, 稀疏旁表, 7, 根指针零, 头 ID 0)、(65543, deadlist, 8, 根指针零, 头 ID 0)、(65544, `deleted_inodes`, 9, 根指针零, 头 ID 65537)、(65545, 墓碑树, 10, 根指针零, 头 ID 65537)、(65546, 谱系树, 11, 根指针零, 头 ID 0)、(65547, logged_ops, 12, 根指针零, 头 ID 0)，十一条诞生 txg 都是 3、`previous_snapshot_txg` 0（0 = 不适用；只有 inode 树那条承载头的前驱快照，D5（快照 / 空间记账机制） 已定项 9）；树的种类的码按 D8（核心索引结构） 已定项 9 的登记表；头 ID 按 D5（快照 / 空间记账机制） 已定项 9 | D8（核心索引结构） 已定项 8 / 已定项 11 | 已定（2026-09-14 用户定案重排并加头 ID；2026-09-16 用户定案加宽到 200、预留 76） |
| 树表归谁 | — | 树表单元由根记录独占持有、住根记录的间接层，不进任何树 | D22（单元原子性怎么合成） 已定项 7 / 已定项 12；C241（树表归谁，两条用户定案说反话） 2026-09-13 已还清 | 已定 |
| 销毁可写头的第二个结构（livelist 共享树） | 200 | day-1 注册：树表第 1 版第 5 条，树的种类 6、树 ID 65541、根指针零、诞生 txg 3、头 ID 0；条目 = 头树 ID 8 + 映射 key 27 + 事件类型 1 = **36 字节一宽**，新池新建文件一条都不写 | D6（快照实现模型） 已定项 2；D19（块指针的结构与宽度预算） 已定项 10 | 已定（2026-09-14 用户定案回一宽） |
| 稀疏旁表（跨头共享块的引用计数）树 | 200 | day-1 注册：树表第 1 版第 6 条，树的种类 7、树 ID 65542、根指针零、诞生 txg 3、头 ID 0；条目 = 映射 key 27 + 计数 4 = **31 字节一宽**，新池新建文件一条都不写 | D5（快照 / 空间记账机制） 已定项 6；D19（块指针的结构与宽度预算） 已定项 10 | 已定（2026-09-14 用户定案回一宽） |
| deadlist 树 | 200 | day-1 注册：树表第 1 版第 7 条，树的种类 8、树 ID 65543、根指针零、诞生 txg 3、头 ID 0；条目形态 2026-09-16 已定（D5（快照 / 空间记账机制） 已定项 12），新池新建文件一条都不写 | D5（快照 / 空间记账机制） 已定项 11；D8（核心索引结构） 已定项 9 | 已定（2026-09-14 用户定案 day-1 注册） |
| 发布的两道屏障与持久顺序 | — | 单元与节点 → 屏障 → journal 记录 → 屏障 → 根槽 FUA | D16（发布语义） 已定项 7；E77（发布的持久顺序） | 已定 |
| 每次写至少 w=2 列 | — | 全部单元与记录两盘各一份 | D2（RAID 条带策略） 已定项 6 | 已定 |
| 系统配置槽轮换写 | 489 / 4096 | 根槽之后写槽 1，世代号 5，tail = 3（取号写槽 0 世代 2；暖机两次各写一次：槽 1 世代 3、槽 0 世代 4） | D22（单元原子性怎么合成） 已定项 8 / 已定项 21 / 已定项 16；D23（journal 的角色与格式） 已定项 3 | 已定 |

<!-- format-const: TREE_TABLE_ENTRY_BYTES = 200 stale=expect("148")|预留 24 非零|树表条目预留 24|TREE_TABLE_ENTRY_BYTES, 148|7 × 148|每层 109|109²|预留剩 24|[147] = 1 -->

```

**出处 `.claude/kb/layout/01-first-txn.md:398-424`（整段抄，未转述）**

```markdown
## 八、根槽写路径的段序列登记表（C316（提交步骤的登记位有四处且互不相同） 的登记位，2026-09-13 立）

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 1（mkfs 种根）与步 5（发布：暖机与新池新建文件）；恢复那一步不写字节，只吃这张表切出的段。

一条线的提交协议由它全部根槽写路径的录制流段序列定（D13（验证路线） 已定项 4 切段：屏障与 FUA 写切段，段内任意整写子集），等价类按这张表的同构分：段边界的位置相同、每段里出现的步骤种类**集合**相同（D17（实现分层与第三方管道） 已定项 2）；一步重复几次（设备数、单元数）是参数不进判据。别处引提交步骤一律链到这一节，不另抄。步骤种类**六种**：写单元（含码 3 容器、实例表单元）、写 journal 记录、根槽 FUA 写、系统配置槽原地覆写、屏障、**整段清零**（`zero_fill`，2026-09-22 随 mkfs 清根环与 journal 环加，用户 2026-09-19 定「录制流登记成一种新步骤、层 0 认它」）。⚠️ **mkfs 那一行的段序列不来自 E142（新池新建文件的干跑） 产物**：那份装置是 `research/` 下一份独立手写模型、不读 `crates/` 的 mkfs 实现（实验页「它答不了的」第 5 条），清零那几步它没有；那一行逐字来自活代码，由 `crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs` 的 `recorded_stream_matches_the_registered_mkfs_segment_sequence` 钉住，门禁 doc-registries 的 segment-registry 格按它比。其余各行的段序列逐字来自 E142（新池新建文件的干跑） 产物的 `name=segments` 行（`path=instance_acquisition / warm_up / transaction / post_mkfs_stream`；取号那一行不是根槽写路径，登记在这里是因为整条流按录制流切、它的两个写落在流的开头一段），单测 `registered_segment_sequences_match_every_recorded_path` 钉住它们；表与产物之间的逐字比对由门禁 doc-registries 的 segment-registry 格做（C316（提交步骤的登记位有四处且互不相同）已还清）。

⚠️ **表里取 E142（新池新建文件的干跑） 产物的各行，对的是第二十一次跑的主产物** `e142-new-pool-file-creation-dry-run-r21-main-2026-10-04.out`（`research/scripts/replay.sh` 第 174 行登记的那份；第 35–39 行与第十九次跑的主产物逐字相同）：取号、暖机、普通发布三行与整条流那一句同产物第 36–39 行逐字相同，mkfs 那一行不按产物比、产物第 35 行与钉它的用例逐字相同；普通发布与整条流里的 12 个单元是分配记录树、extent 树按位置寻址（D8（核心索引结构） 已定项 14）之后的；每次发布末尾有系统配置轮换之后那道屏障（D16（发布语义） 已定项 7），同一块盘上连着的屏障并成一道（D13（验证路线） 已定项 4）；`name=width` 30 行 expected == actual。那一份产物没有跑层 0 整轮枚举（`name=layer0 … states=not_run closed_form=16777240 skipped=true`，够判之后不跑），整条流那一句的状态数是闭式算出来的，不是枚举出来的。

| 根槽写路径 | 录制流的段序列（每段的写数与种类；「种类」那串是产物 `kinds=` 字段的原样，每段一个步骤种类多重集，段之间用 `\|` 隔开，门禁 doc-registries 的 segment-registry 格逐字比对） | 出处 | 层 0 枚举 |
|---|---|---|---|
| mkfs 种根 | [m0 清零 × 每盘（[0, 根环基址) 一段 + 根环三个区域各一段 + journal 环一段）= 10 + m1 实例表单元 × 2 盘 + m2 树表单元 × 2 盘 = 4 单元写，共 14] 屏障 [m3 根 FUA 区域 0] [m3 根 FUA 区域 1] [m3 根 FUA 区域 2] [m4 系统配置槽 × 2 盘 × 2 槽 = 4，都是世代号 1] 屏障 ⇒ `14+1+1+1+4`，25 次操作（21 次写，两道池屏障按设备记、每道两块盘各一步）、16402 个崩溃状态，种类 `[zero_fill×10,unit_write×4,barrier×2]\|[root_record_fua]\|[root_record_fua]\|[root_record_fua]\|[system_configuration_slot×4,barrier×2]` | 一；**活代码**（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs` 的 `recorded_stream_matches_the_registered_mkfs_segment_sequence` 钉住录制流，门禁 doc-registries 的 segment-registry 格按它比）。⚠️ **不按 E142（新池新建文件的干跑） 的产物比**：那份装置是 `research/` 下一份独立手写模型、不读 `crates/` 的 mkfs 实现，清零那十步它没有（C485（segment-registry 格不读活代码，段序列变了照样绿） 2026-09-22） | **不在**（G19：装置从 mkfs 之后的池起枚举；段序列由单测钉住） |
| 第一次可写挂载取号（实例代号 0 → 1，不是根槽写路径） | [a1 系统配置槽 × 2 盘，世代号 2] ⇒ `2`，2 次操作、4 个崩溃状态，种类 `[system_configuration_slot×2]`；取号两写之后一道屏障（D23（journal 的角色与格式） 已定项 16 要取号两写之后、本实例第一个非系统配置写之前一道完成了的屏障；2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：`crates/singlefs-core/src/transaction.rs` 的 `acquire_instance` 写完两份系统配置自己发一道（`CommitStep::Barrier`），首次挂载上它与暖机第一次空发布开头那道背靠背、登记的段序列不变；第二次以后的挂载靠它把取号与写行那次发布的单元写隔开（2026-09-17 按代码改写）；世代号按 D22（单元原子性怎么合成） 已定项 16 逐盘 + 1，首次挂载写出的是 2 | 一（a1）；D23（journal 的角色与格式） 已定项 16；产物 `name=segments path=instance_acquisition` | 在（第六次跑起，整条流开头那一段） |
| 空发布（暖机，第一版 2 次） | 屏障 [w1 空记录 × 2 盘] 屏障 [w2 根 FUA] [w3 系统配置槽 × 2 盘] 屏障（第二次同型，w4–w6；第一次末尾那道屏障与第二次开头那道是同一块盘上连着的屏障，并成一道，D13（验证路线） 已定项 4）⇒ 两次合起来 `2+1+2+2+1+2`，20 次操作、15 个崩溃状态，种类 `[journal_record×2,barrier×4]\|[root_record_fua]\|[system_configuration_slot×2,barrier×2]\|[journal_record×2,barrier×2]\|[root_record_fua]\|[system_configuration_slot×2,barrier×2]` | 零；D16（发布语义） 已定项 8；产物 `name=segments path=warm_up` | 在（第四次跑起） |
| 普通发布（新池新建文件） | [t1–t12 单元 × 2 盘 = 24] 屏障 [t13 记录 × 2 盘] 屏障 [t14 根 FUA] [t15 系统配置槽 × 2 盘] 屏障 ⇒ `24+2+1+2`，35 次操作（29 次写，三道池屏障按设备各记两步）、16777223 个崩溃状态，种类 `[unit_write×24,barrier×2]\|[journal_record×2,barrier×2]\|[root_record_fua]\|[system_configuration_slot×2,barrier×2]` | 零；D16（发布语义） 已定项 7；产物 `name=segments path=transaction` | 在（E142（新池新建文件的干跑） 主臂） |
| 实例切换 / 管理员回退 | 实例切换那一半 2026-09-16 已写成字节，就是「第一次之后的可写挂载（写行）」那一行（切换 = 挂载内做一次恢复再写行，D23（journal 的角色与格式） 已定项 14）；管理员回退那一半是挂着时的一次普通发布（里程碑「覆盖写、释放、回退与复用」步 4，D23（journal 的角色与格式） 已定项 14「管理员回退是挂着时的一次向前发布」）：[单元 × 2 盘] 屏障 [记录 × 2 盘，jsn 接现行那一版那一条、事务号 0] 屏障 [根 FUA] [系统配置槽 × 2 盘] 屏障（发布末尾那道，D16（发布语义） 已定项 7），孤立看与普通发布同型；段里的单元只有固定点那几种（树表单元、分配记录树、记账树、中央映射树的节点），加固定脚本里回退 D 那次发布追加谱系记录写的两个单元（D18（块里携带什么信息） 已定项 24 第 5 条）：谱系容器（码 3，槽 50352）与谱系树根（码 2，槽 50354），两块盘各一份；不写实例表单元（实例表照现行那一版的）、不写数据单元；不取号、不暖机，前后都不另接空发布。脚本钉的段序列、写数与闭式已按段模型手算钉住（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 第 426–437 行钉段序列，第 634–665 行钉 36 段、闭式 67502143、快档 77）；实六 2026-09-26 在新形态代码上跑过一次探针，核对出 191 写、9 根、29 段、快档 104（`research/prompts/m2-impl6-implementer-report.md` 第 45 行），当时在代码审阅第 19 条与 C577（系统配置没见证到的最新根，乙罩不到）之前；带 layer0 名字的目标仍没跑过（`research/prompts/m2-impl-merge-verify-1-implementer-report.md` 第 73、222 行），层 0 仍没跑（步 6） | D23（journal 的角色与格式） 已定项 14；D16（发布语义） 已定项 7；用例 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 的 `rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content`（钉只重写固定点单元、实例表照 C 的、jsn 接 C、事务号 0） | 管理员回退那一半在第二条流里，层 0 没按新形态跑过（步 6） |
| 第一次之后的可写挂载（写行） | **装置钉住**（里程碑「覆盖写、释放、回退与复用」步 3 2026-09-16 落地，孤立形状从第二条流推得）：[取号系统配置槽 × 2 盘，世代号 3] 屏障 [实例表单元（写行）+ 分配记录树 5 个节点 + 记账 + 映射 + 树表 = 9 单元 × 2 盘 = 18] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] 屏障 ⇒ 孤立看 `2+18+2+1+2`；整条流里上一次发布的 2 个系统配置槽写与取号的 2 个写之间隔着一道屏障、各自成一段，末尾的 2 个系统配置槽写过了发布末尾那道屏障（D16（发布语义） 已定项 7），不与暖机第一次的单元写同段（第二条流第 15–21 段 `2+2+18+2+1+2+16`）；用例钉的写行那一段 18 个单元写、之后暖机那一段 16 个，与这一行和「空发布（暖机，后续可写挂载的实例，写 c_max 个固定点单元）」那一行孤立看的 18、16 相符（分配记录树按位置寻址之后每次重写 5 个节点：两块盘的叶各一、层级 1 节点各一、根一，同新池新建文件的 t5..t9，D8（核心索引结构） 已定项 14）；取号之后那道屏障由取号自己发（D23（journal 的角色与格式） 已定项 16：写行那次发布有单元写，等不到空发布开头那道）；每次可写挂载都写行（实例 0 不写，第一次可写挂载要写的区间是空的），写行那次是新实例的第一次发布，之前不推抬 F 的空发布，元数据走切换预留（D18（块里携带什么信息） 已定项 11、D28（挂载期承诺量） 已定项 3；2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方） | D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 16；用例 `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` | 在（第二条流，门禁 checker-tier-layer0-replay） |
| 空发布（暖机，后续可写挂载的实例，写 c_max 个固定点单元） | **装置钉住**（步 3 落地，孤立形状从第二条流推得）：[分配记录树 5 个节点 + 记账 + 映射 + 树表 = 8 单元 × 2 盘 = 16] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] 屏障 ⇒ 孤立看 `16+2+1+2`，与首次挂载的暖机不同型：记账树已经存在，空发布也重写记账行连带分配记录树的 5 个节点、映射和树表单元（D16（发布语义） 已定项 9），这条脚本上一次空发布写 8 个单元；整条流里上一次发布的 2 个系统配置槽写过了发布末尾那道屏障、自成一段，不与这一次的单元写同段（第二条流第 20–21、24–25 段各 `2+16`，用例钉的 16 个单元写与孤立看的 16 相符）；实例 2 从 txg 5 起推 2 次（txg 6 落盘 0、txg 7 落盘 1），次数按「本实例的根覆盖全部区域盘」现算、上限 3——D16（发布语义） 已定项 8 只给第一次可写挂载定了常量 2，后续挂载的次数没有条款，实做的取法等用户定 | D16（发布语义） 已定项 8 / 已定项 9；用例 `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` | 在（第二条流，门禁 checker-tier-layer0-replay） |
| 抬 F 的空发布（D16（发布语义） 已定项 1：准入不够时先推空发布抬回退下界） | **装置钉住**（里程碑「覆盖写、释放、回退与复用」步 5 2026-09-17 落地，孤立形状从第二条流推得）：与后续挂载的暖机空发布同型——[分配记录树 5 个节点 + 记账 + 映射 + 树表 = 8 单元 × 2 盘 = 16] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] 屏障 ⇒ 孤立看 `16+2+1+2`，根记录的 F 写成目标值、记账行是回收过释放代 ≤ F 之后的账；推到每块盘上都有一条带新 F 的根为止（这条脚本上两次：txg 15 落盘 0、txg 16 落盘 1）；整条流里 txg 14 的轮换、抬 F 先写的系统配置、两次空发布的单元写各自成一段（第二条流第 56–65 段 `2+2+16+2+1+2+16+2+1+2`；用例钉的 16 个单元写与孤立看的 16 相符）；第一版只有测试的强制入口，准入不够的正常触发没做 | D16（发布语义） 已定项 1；用例 `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` | 在（第二条流，门禁 checker-tier-layer0-replay） |
| 只做过 mkfs 的池的可写挂载（取号、零单元写行、零单元暖机）再发第一个文件版本 | **装置钉住**（里程碑「覆盖写、释放、回退与复用」步 3 2026-09-17 落地，2026-09-17 用户定案允许只做过 mkfs 的池可写挂载）：重开之后取号 [系统配置槽 × 2 盘] 屏障，树表 0 条 ⇒ 写行那次发布与暖机都写零个单元，与 mkfs 同一个进程里的取号、两次暖机同型，之后第一个文件版本同新池新建文件 ⇒ 整条流 `2+2+1+2+2+1+2+24+2+1+2`，与新池新建文件那条流逐段相同、闭式 16777240（同一份用例的 `the_formatted_pool_mount_and_first_file_stream_keeps_the_new_pool_file_creation_segment_sequence` 钉住）；树表 0 条的一版上再写实例表行（第二次起的可写挂载）今天做得成：写行那次发布写实例表与一片分配记录节点两个单元（D16（发布语义） 已定项 9）；回退到树表 0 条的根只在环里还留着带文件版本的根时在写之前拒绝（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步要拿掉这道拒绝） | D16（发布语义） 已定项 9（树表 0 条时空发布写零个单元）；用例 `crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_a_formatted_pool.rs` | 与新池新建文件那条流的基线镜像、写表与段序列逐项相同（快用例钉住），不另枚举；崩溃状态上重开可写挂载不在层 0 里 |

⚠️ 发布与下一次发布之间有发布末尾那道屏障（系统配置轮换之后、发布返回之前，D16（发布语义） 已定项 7，C577（系统配置没见证到的最新根，乙罩不到） 加的）：上一次发布的 2 个系统配置槽写自成一段，不与下一次发布的单元写同段；同一块盘上连着的屏障并成一道（D13（验证路线） 已定项 4），取号两写之后那道与暖机第一次开头那道、暖机第一次末尾那道与第二次开头那道在录制流里各只记一道。新池新建文件那条流是整条流 `2+2+1+2+2+1+2+24+2+1+2`、16777240 个状态，种类 `[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]`；57 次操作，16777240 是每次写只取两态的闭式，系统配置槽写取三态时是 16777260（产物 `name=segments_three_state path=post_mkfs_stream` 那一行，`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` 的 `FULL_STATES` 同值）。mkfs 末尾有屏障，mkfs 与取号之间的接缝是切开的。段序列按整条录制流切，不按路径切，所以登记表按路径给的段序列是「孤立看」的形状，整条流那一句才是层 0 枚举吃的。

⚠️ 第二条流（里程碑「覆盖写、释放、回退与复用」步 0 的固定脚本，2026-09-26 改到正常卸载）：取号 → 暖机 × 2 → A → B → 进程退出、重开取号 → 写行 → 暖机 × 2 → C → 挂着时的一次向前发布回退到 A 的根（D，不再重开进程、不再重新取号）→ 覆盖写 × 5 → 抬 F 的空发布 × 2 → E → 正常卸载（推两次带卸载记号的空发布，txg 18 落盘 0、txg 19 落盘 1）；第二条流的段序列 `2+2+1+2+2+1+2+24+2+1+2+24+2+1+2+2+18+2+1+2+16+2+1+2+16+2+1+2+24+2+1+2+24+2+1+2+28+2+1+2+28+2+1+2+28+2+1+2+28+2+1+2+28+2+1+2+2+16+2+1+2+16+2+1+2+28+2+1+2+2+16+2+1+2+16+2+1+2`、481 次写、1678377089 个状态（19 条根槽写、78 段，状态数是每次写只取两态的闭式）；全量枚举时系统配置槽写取三态，状态数是同一份用例 `FULL_STATES_THROUGH_THE_UNMOUNT` 钉的 1678377204；平时快档按甲二枚举（D13（验证路线） 已定项 9），门禁 checker-tier-layer0-replay `--full` 在 release 下全量跑、带断点续跑，doc-registries 的 segment-registry 格核这句与用例里的数组、闭式、写数相符。没有干跑产物，装置钉住：`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 的 `prepare` 把这个数组钉死；到 C 是 `ThirdVersion` 那一臂（32 段、50724921），到 D 是 `RollbackToFirstVersion` 那一臂（36 段、67502143），两个前缀各由同一份用例里只跑准备与计数的一条钉住。前 11 段与新池新建文件那条流（整条流那一句）逐段相同，到 B 为止是 `SecondVersionOnly` 那一臂（15 段、70 次写）；每次发布末尾有屏障（D16（发布语义） 已定项 7），每次发布的 2 个系统配置槽轮换写都自成一段，不与下一次发布的单元写同段；B 的轮换与重开取号的 2 个系统配置槽写之间隔着取号写之前那道屏障，各自成段（2、2）；各次发布的单元写各自成一段：A、B、C 各 24，写行 18，写行之后两次暖机各 16，D 24，五次覆盖写与 E 各 28，抬 F 与卸载推的空发布各 16；抬 F 与卸载开头先写的系统配置与上一次的轮换各自成段（2、2）；末尾落在 txg 19 的轮换（2）。C577（系统配置没见证到的最新根，乙罩不到） 之前实六探针现算的是 `PROBE script=ReuseAfterRaisingFloorThenNormalUnmount sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 4, 18, 2, 1, 18, 2, 1, 18, 2, 1, 26, 2, 1, 22, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 4, 16, 2, 1, 18, 2, 1, 30, 2, 1, 4, 16, 2, 1, 18, 2, 1, 2] writes=477 roots=19 segments=61 closed_form=6649413746 quick_tier=232`，快档 232 个状态（C577（系统配置没见证到的最新根，乙罩不到） 之前的数，重量归提交时的崩溃验证员）。

<!-- format-const: WARM_UP_EMPTY_PUBLISHES = 2 -->
<!-- format-const: NEW_POOL_FILE_CREATION_TXG = 3 -->

```

**出处 `.claude/kb/layout/01-first-txn.md:425-437`（整段抄，未转述）**

```markdown
## 还剩几处空白

**这张表里已经没有指不到分项的字段了，也没有判「改新池新建文件的字节：是」的未定项**（2026-09-13 总审核新立的 31 条同日用户定案；D22（单元原子性怎么合成） 未定项 6 判「否」、按用户指示挂到 zoned 开线；2026-09-14 总审核第二轮新立的 D2（RAID 条带策略） 未定项 21「parity 几格」与 D5（快照 / 空间记账机制） 已定项 12「deadlist 条目的形态」也都判「否」）。
两边的清单必须相等，门禁阶段「新池新建文件的三份文件互相挂钩」判红。
⚠️ **表里第一行不是未定项，是一格已经定案的条款**：系统配置槽宽 2026-09-14 由用户要求重开、走三方论证，同日定为格式常量 4096，它不在任何决策的「未定项」一节里（D22（单元原子性怎么合成） 已定项 2 那一格写的是「**格式常量 4096**」），所以门禁那一条对不上它；留着这一行是为了不让这张表写成「（无）」——写成「无」就等于宣布这张表已经定完了。

| 空白 | 落点 | 现在 | 预想先按什么写 |
|---|---|---|---|
| 系统配置槽宽是不是「= 探测到的 `physical_block_size`」 | D22（单元原子性怎么合成） 已定项 2 的槽宽那一行 | **2026-09-14 三方第一轮 + 用户收尾弹窗定案：格式常量 4096（`research/prompts/d22-slotwidth-r1-main-verification.md`）**（主 agent 另起）。它动的是一、七两节里每一次系统配置写的槽宽（512 改成 4096）与 481 字节系统配置在槽里的余量（31 改成 3615）；根槽宽仍是探测到的 `physical_block_size`（512），槽距仍是 4096 | 一、七两节已按 4096 的系统配置槽写，`crates/singlefs-format/src/lib.rs` 的 `SYSTEM_CONFIGURATION_SLOT_BYTES` 坐实 |

**仍标预想的取值**（不是格式空白，也不再等哪一次跑：E142（新池新建文件的干跑） 第八次跑已按 4 GiB 取数，产物 `name=config` 逐字 `device_bytes=4294967296`，里程碑「新池新建文件」2026-09-14 已出口；仍标预想，是因为镜像大小全仓没有条款，账在 C323（镜像大小全仓没有条款））：镜像大小 4 GiB（决定空闲字节与全空段数；4 GiB 这个下界由 D23（journal 的角色与格式） 已定项 19 ③ 的「环 ≤ 设备容量 ÷ 4」逼出）。extent 树带不带 key 区间 2026-09-14 已定（D18（块里携带什么信息） 已定项 2：全部码 2 节点无条件带），不再是预想。
C66（格式级空白没有分项号） 立账时点的五处空白如今全部有了落点。

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
