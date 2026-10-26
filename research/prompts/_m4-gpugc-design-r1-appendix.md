**出处 `.claude/kb/milestone/04-fourth-txn.md:1-10`（整段抄，未转述）**

```markdown
# 里程碑「共享与回收」：作业点、验收与次序

**中文** · [English](04-fourth-txn.en.md) · [日本語](04-fourth-txn.ja.md)

**口径**：规划文件，2026-10-01 按用户与主 agent 的对话建档，是草案。**它不要求正确**：每项开工前回来改。前提是里程碑三（[03-third-txn.md](03-third-txn.md)）的出口全部达成；里程碑三收口时少了哪一样，这份文件按缺的那一样重排。原来的项六（并行提交）搬进里程碑六「并发读写与并行提交」（[06-sixth-txn.md](06-sixth-txn.md)），排在这份与里程碑五「小文件打包」（[05-fifth-txn.md](05-fifth-txn.md)）之后（用户 2026-10-11 定：「里程碑4 和五 交换顺序就好了」，同日再定打包排在并发之前）；这份按串行提交设计。

**读法**：每项四格——「原话」照录用户口述；「仓里已有的」写这一项在仓里压着哪些条款、欠账、代码（2026-10-01 现查的，带出处）；「作业点」一张表：编号、做什么、压着的条款与欠账、验收（每条先证会红）、依赖；「开工前要先定的」列还没定、开工前要走三方或问用户的问题。凡写「推的」的是主 agent 没量过的判断。作业点的编号写成「项·点」（一·1、二·3），只在这份文件里用。

**新事务有两种**：建快照 / 删快照（项一），与 GC 搬迁（项二）。GC 不新增 `CommitStep` 成员，意图写成单调谓词（D26（后台整理与放置回收） 已定项 2），走里程碑三删除落地的那套意图机制。

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:11-30`（整段抄，未转述）**

```markdown
## 出口（用户 2026-10-01 定：共享最小形态 + GC，生命周期闭环与长期老化，只读扫描重建作并行线）

九件，全部达成才算完，每件的验收在对应项的作业点表里（出口 4 挪出，编号不改）：

| # | 出口 | 归哪一项 |
|---|---|---|
| 1 | 共享最小形态落地：建只读快照、快照下覆盖写与删文件（被快照引用的块进 deadlist、不释放）、删快照；D5（快照 / 空间记账机制） 已定项 17 的重开收口；新盘上形态登记进 `.claude/kb/layout/` 下与里程碑同号的一份（文件名以 `04-` 开头）；I-3.5（引用区间的精确性）、I-3.6（deadlist 紧界）、I-3.7（活快照可枚举） 实现；共享的崩溃流全域 0 红 | 一 |
| 2 | 自建 GC 落地：意图、分批搬迁、产出全空聚簇段、discard；D3（空间分配） 已定项 7 登记表第 7 行的 `N₀` 重开并定值；能搬被快照共享的块；C142（第一个 reflink 之后整理会造出静默陈旧读）、C139（整理停摆之后释放也发不出去）、C364（整理意图记录的字段没有逐字段对账）、C91（整理意图与增量触发量的重做漂移） 还清或改写；GC 的崩溃流全域 0 红 | 二 |
| 3 | 生命周期闭环：一条固定脚本走完分配 → 覆盖写 → 建快照 → 再覆盖写 → 删文件 → GC 搬共享块 → 删快照 → 回收 → 在搬出来的空段上再分配，崩溃放量全域 0 红 | 三 |
| 5 | 「GC 在抽象文件系统状态上是恒等变换」在里程碑三七·1 选定的工具上证出；带 GC 的模型对拍与不跑 GC 逐项相同 | 二 |
| 6 | 只读扫描重建：从单元扫出的索引与现行索引逐项比对零差异 | 四 |
| 7 | 截断落地（用户 2026-10-12 加）：单文件 truncate 到 N 字节走删除的意图与分批回收，被快照引用的尾部块进 deadlist；新盘上形态登记进 `.claude/kb/layout/` 下文件名以 `04-` 开头的一份；截断的崩溃流全域 0 红 | 六 |
| 8 | GPU 垃圾回收（用户 2026-10-19 加）：D26（后台整理与放置回收） 已定项 13 落地——挂载参数 `background_compute`（默认 `cpu`）、`background_gpu_devices`（不给用全部显卡）、`background_gpu_memory_limit`（一组参数管全部后台任务，用户 2026-10-20 定），挂着可切、下一批生效，GPU 用不上报错不降级，发布后查水位加定时兜底，写带宽上限加前台让路；GPU 路径与 CPU 参照在同一组输入上逐项相同、两种模式写出的块序列逐字节相同；C412（整理的挂钟代价没量过） 量出来、几个默认值定下；CPU 模式与 GPU 模式的对照组跑完（七·5） | 七 |
| 9 | 接手里程碑三、写着「已打进主工作区，待合入后验证」的八笔欠账与 C821（GPU 判器没有扫描重建内核）销账（用户 2026-10-21 加）：C463（一块盘的系统配置两份全废之后静默成功）、C555（两盘最新系统配置槽都坏时取号撞号）、C593（落后且缺一份单元的盘被永久拒可写）、C596（见证判不出的系统配置盘面只剩只读挂载）、C663（系统配置槽越界与不收之值两读者分类不同）、C707（系统配置判不出里 fsid 不符落哪一边没写）、C728（根槽与系统配置槽尾部字节没有条款）、C754（退回之后再判缺记录不重扫环）；C821（GPU 判器没有扫描重建内核） 的内核、对拍测试与 `crates/mutations.tsv` 的 24 行变异同样在。实现与检查已在提交 0b926bd3f 里，那一笔没跑重型六道；提交时重型六道与各登记行的验收都绿了，逐笔挪进已还清表 | 八笔在里程碑三二·4、二·5（[03-third-txn.md](03-third-txn.md)），C821（GPU 判器没有扫描重建内核） 在四；去向仍写在欠账表各行 |
| 10 | 崩溃全量：新增的跑完，另补积压，每个里程碑至少 100 亿态（用户 2026-10-22 定）。崩溃点重放与崩溃放量的全量照两段跑（录入在 CPU、判在 GPU，观察者挂全），不缩枚举域，全部 0 红。① 这一里程碑新增的崩溃流与单元键变了的块先跑；② 再从积压登记表里补跑，至少 50 亿态，按登记表的次序接着上一次停下的地方。新增与积压合起来多于 100 亿态时，这一里程碑至少跑 100 亿态，新增跑不完的那一截进积压登记表末尾；合起来不足 100 亿时全部跑完。这一里程碑立积压登记表：前面没跑的全量（全域约 3.8×10¹⁰ 态，推的：各条全量崩溃枚举用例 `#[ignore]` 理由里写的态数之和，出处 `~/.local/share/singlefs/work/m3-close-b16-observer-provenance/report.md` 第八节第 5 条），按 `.claude/gate.d/crash-point-replay-items.tsv` 的项序与崩溃放量 ITEMS 的分项序、每项态数现数，逐项登记跑到哪；表放在 checker 档 crate 里。红了照 `.claude/rules/test-script-discipline.md`「全量的红，钉回快档」钉回快档、修好再跑。 | 崩溃全量（跨里程碑） |

**挪出**：真盘掉电装置原是项五（并行线），用户 2026-10-16 定挪进里程碑九（[09-ninth-txn.md](09-ninth-txn.md)「四　真盘掉电装置」）；这份文件没有项五，项六的编号不改。长期老化（原出口 4、段 4、三·2，QEMU 真设备上开着快照与 GC 跑随机历史）用户 2026-10-17 定一起挪进里程碑九（[09-ninth-txn.md](09-ninth-txn.md)「五　长期老化」；原话「不算，真设备已经移动到里程碑9 了， 如果里程碑4有残留一起移动」）；E10（老化：结构抗老化 vs 事后整理） 的跑前登记留在二·1，出口编号不改。

**不在出口**：目录与 POSIX 面（用户定往这份文件之后的里程碑：「目录往里程碑5去做比较好」；里程碑五给了小文件打包、里程碑六给了并发；目录与 POSIX 面落在里程碑十一，用户 2026-10-12 定：「POSIX 去里程碑9，FUSE 去里程碑10，里程碑7 优化树结构和缓存，里程碑8 优化环结构和wa」）、可写克隆、加密、压缩、小数据打包容器（D27（小数据打包容器），归里程碑五 [05-fifth-txn.md](05-fifth-txn.md)）、第三方管道、扫描重建的写回（救援）、ZNS / FDP、交给盘的 FTL、FTL 模拟器（这三样的进入条件在「优化项：不在出口，进入条件已登记」一节）。

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:31-42`（整段抄，未转述）**

```markdown
## 分组与次序

| 段 | 做什么 | 为什么排在这 | 状态 |
|---|---|---|---|
| 0 接手 | 核里程碑三交出的（删除的意图机制、崩溃点覆盖表、形式化工具）；「接手里程碑三：三条接口问题」一节「三问的答案现状」逐问核对（第 2 问没有一轮答过，按「否」办；第 3 问并行那一半归里程碑六） | 第 2 问按「否」办，项一的设计（一·2 的共用释放）要先改 | 未开 |
| 1 设计 | 一·1（D5（快照 / 空间记账机制） 重开与只读快照形态）∥ 二·1（GC 设计轮，含 E10（老化：结构抗老化 vs 事后整理） 跑前登记）∥ 四·1（扫描重建题面）∥ 六·1（截断设计轮） | 四边互不依赖（六·1 里被快照引用的尾部块那一问随一·1 的形态）；二·1 里搬共享块那一问要等一·1 的形态，其余先做 | 做了（一·1 `research/prompts/m4-snapshot-r1-main-verification.md`；二·1 `research/prompts/m4-gc-r1-main-verification.md`，E10（老化：结构抗老化 vs 事后整理） 的跑前登记是 `research/prompts/e10-r1-prereg.md`；四·1 题面随 `crates/singlefs-checker/src/rebuild.rs` 写成；六·1 `research/prompts/m4-trunc-r1-main-verification.md`、`research/prompts/m4-trunc-r2-main-verification.md`） |
| 2 实现 | 一·2 → 二·2（共享先落地；GC 不涉共享的部分可与一·2 并行写）∥ 四·2 ∥ 六·2 | GC 最难的崩溃流（搬被共享的块）要先有共享才造得出来 | 核心层已合入 master（提交 0b926bd3f）：一·2 建只读快照 / 快照下覆盖写与删文件 / 删快照，二·2 整理（选段与搬迁计划、意图记录、区域扫描、`publish_relocation_batch`；产品路径在签整理意图之前停下），四·2 `rebuild.rs`，六·2 `truncate.rs`；各作业点还差什么见各作业点行与欠账表 |
| 3 覆盖 | 一·4、二·3、三·1、六·3 的流跑在崩溃放量上全域；二·4 证性质 | 出口 1、2、3、5、7 在这一段满足 | 流已入库并登记在 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行（一·4、二·3、三·1、六·3 对应的流）；全域 0 红与二·4 的性质证明还没有记录 |
| GPU GC | 七·1 量 → 七·2 设计 → 七·3 实现 → 七·4 覆盖 ∥ 七·5 对照组 | 七·1 只等二·2 的 GC 发布路径（已在 master：`transaction.rs` 的 `publish_relocation_batch`）；七·2 要七·1 的 CPU 与 I/O 拆分；出口 8 在七·4 满足 | 未开 |

**估计**：没估。

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:43-57`（整段抄，未转述）**

```markdown
## 一　共享最小形态：一个只读快照

- **原话**：用户 2026-10-01 在「共享（快照 / 克隆）的最小形态要不要排到 GC 前面或一起做」那一问选「里程碑四：共享最小形态 + GC」。
- **仓里已有的**：D5（快照 / 空间记账机制） 已定 21 项、D6（快照实现模型） 已定 10 项；D5（快照 / 空间记账机制） 已定项 17「引入同一块的第二个活引用必须先重开 D5（快照 / 空间记账机制）」，只读快照就是第二个活引用。deadlist、livelist、稀疏旁表三棵树新池里已注册进树表（`crates/singlefs-core/src/transaction.rs` 的 `FileVersionTreeIdentifiers` 里 `livelist`、`sparse_side_table`、`deadlist` 三个字段，`crates/singlefs-core/src/recovery.rs` 的 `rebuild_version_in_the_unit_area_starting_at` 读回），deadlist 已有写者（`transaction.rs` 的 `publish_create_snapshot` 建快照，快照下覆盖写与删文件把被快照引用的块记进 deadlist，`publish_snapshot_destruction_intent` 销毁快照），livelist 与稀疏旁表仍只注册、没有写者；`grep -rn "克隆" crates/singlefs-core/src` 仍只命中 `transaction.rs` 里一条 TODO 注释的 2 行（说「`crates/` 今天没有克隆路径」），没有克隆代码。I-3.6（deadlist 紧界）、I-3.7（活快照可枚举） 已实现（2026-10-15）；I-3.5（引用区间的精确性） 部分实现：出现规则（`walk.rs` 的 `judge_the_appearance_rule_of_adjacent_candidate_versions`）与 ③ ④（deadlist 那两条）已实现，余下以 `crates/singlefs-checker/invariants/05-空间记账.md` 的状态格为准。相关欠账：C144（快照谱系的区间标号没有载体）、C145（deadlist 在盘上没有落点）、C300（deadlist 条目按 key 查映射查不到没有条款）、C301（deadlist 与 livelist 的态别没写）、C336（写意图那一刻找不到快照的下一侧）。删除批与覆盖写共用的释放按快照分支：被快照引用的块进 deadlist，别的照常释放（`crates/singlefs-core/src/transaction.rs` 的 `snapshot_lifetime_verdict`）；一·2 改的是这处共用释放，不只是删除的分批回收。按关键词搜到、归属是推的（2026-10-12）：欠账 C195（销毁级联时 prev 传递只写在实验的模型修订记录里）、C302（快照那一刻的码 2 历史版本取不出）、C337（被销毁的头何时不算活引用没有条款）、C250（`previous_snapshot_txg` 的父链解析非单射）、C236（稀疏旁表被定为必做却没有任何落点）、C148（树表条目变宽把间接层门槛压到二层）、C149（根槽重发之后 birth 口径松一格）、C675（同一种树在树表里至多一条没有不变量）、C289（码 2 的出生树取哪个字段没写）、C292（码 2 身份段零消费者的理由已不成立）、C110（跨头共享与加密后元数据类的块头一致性读法）；分项 D16（发布语义） 已定项 14、D21（权威态与派生态的分界） 已定项 6；不变量清单 `crates/singlefs-checker/invariants/README.md`「待补」里没编号的「快照可见性」。别家的 deadlist 做法在 `.claude/kb/prior-art.md`「十、快照下的死块记账、搬被共享的块与 discard 时机」（未在本项目验证）。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 一·1 | 设计轮（三方）：D5（快照 / 空间记账机制） 已定项 17 的重开收口到只读快照这一形态；只读快照用到 D6（快照实现模型） 已定机制的哪几样（每头一棵树、引用计数旁表、livelist），哪几样留给可写克隆；快照在并行提交下切在哪（推的：切在一次发根的边界上，D16（发布语义） 已定项 10 的攒批发根） | D5（快照 / 空间记账机制） 已定项 13、14、17、19、21；D6（快照实现模型） 已定项 1、3；C144（快照谱系的区间标号没有载体）、C336（写意图那一刻找不到快照的下一侧） | 判决落 `research/prompts/`，定案写回 kb | 无；并行提交排在里程碑六（[06-sixth-txn.md](06-sixth-txn.md)），这里按串行设计，「快照在并行提交下切在哪」交里程碑六一·1 定形态时答 |
| 一·2 | 实现：建只读快照；快照下覆盖写与删文件，被快照引用的块进 deadlist、不释放；删快照，deadlist 按约定 B 并到更新的那一侧（D5（快照 / 空间记账机制） 已定项 16），其余释放；活快照列表可枚举 | D5（快照 / 空间记账机制） 已定项 12、13、16、19、20、21；I-3.5（引用区间的精确性）、I-3.6（deadlist 紧界）、I-3.7（活快照可枚举）；C145（deadlist 在盘上没有落点）、C300（deadlist 条目按 key 查映射查不到没有条款）、C301（deadlist 与 livelist 的态别没写）；C675（同一种树在树表里至多一条没有不变量）（2026-10-19 从里程碑三挪来，归属是推的，开工前核，仍欠的见登记行）；C865（树表条目未知 flags 位让整张树表读不下去）（2026-10-20 从里程碑三挪来：`research/prompts/m3-close-c693-r2-main-verification.md` 判对象级的丙随这一行做，仍欠的见登记行） | 三条不变量的 checker 判定各有必红世界（例：把该直接释放的块放进 deadlist，I-3.6（deadlist 紧界） 红）；模型对拍加建快照、删快照两种操作 | 一·1 |
| 一·3 | 新盘上形态登记进 `.claude/kb/layout/` 下与里程碑同号的一份（文件名以 `04-` 开头），起名照 `.claude/rules/path-moves.md`「按里程碑递增的文件放进同一个目录」 | `.claude/rules/format-evolution.md`「硬约束」：格式、`crates/singlefs-checker/invariants/`、checker 三者同步；C774（E142 装置追平退役树之后剩第 26 次登记） | 字节表每段指向一条决策分项 | 一·2 |
| 一·4 | 崩溃流：建快照、快照下覆盖写、删快照，用公共前缀模块，节点按 `<流程>.<周期>.<步>` 编码，登记 `crash-case:` 行 | D13（验证路线） 已定项 4 | 全域 0 红；每条流一个必红世界 | 一·2 |

- **开工前要先定的**：快照怎么点名（第一版没有目录；D5（快照 / 空间记账机制） 已定项 9 头 ID = 该头 inode 树的树 ID，快照的标识沿不沿用它）；欠账表里与快照相关的其余各行（C265（销毁可写头的释放边界写成了固定的克隆时刻）、C270（分叉点闸的子计数数不数克隆）、C347（祖先表读不出时克隆头认领什么没有条款） 这类管可写克隆的）逐条判归不归这个里程碑。

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:58-72`（整段抄，未转述）**

```markdown
## 二　自建 GC（D26（后台整理与放置回收） 腿一的回收线）

- **原话**：「cpu gc 肯定要做。」（2026-10-01）
- **仓里已有的**：D26（后台整理与放置回收） 已定项 1、2、3、4、5、6、7、11、12 定了触发、事务形态、准入底额、记账交互、观测点、回收分线与腿一的形态；未定项 8、9、10 只管交给盘那条线。E93（老化下的放置与碎片度） 判腿一是必需的；E10（老化：结构抗老化 vs 事后整理） 第一次跑还没出结论。代码：整理已入库——意图条目格式在 `crates/singlefs-core/src/logged_operations.rs`，选段与搬迁计划在 `compaction_plan.rs`，区域扫描在 `region_scan.rs`，发布入口是 `transaction.rs` 的 `publish_relocation_batch`，显式运行入口是 `mounted_session.rs` 的 `run_compaction`（都在 `crates/singlefs-core/src/`），discard 已进 `BlockDevice`；批大小条款没定，产品路径在签任何意图之前停下。`allocator.rs` 的 `reclaim_released_up_to` 只把已释放的落点放回空闲、不搬活数据；`crates/singlefs-core/src/system_configuration.rs` 的 `COMPACTION_WATERMARK_BUILT_IN_DEFAULT` 与 `compaction_watermarks` 让整理三条水位第一版恒 0（0 = 内置默认）。D3（空间分配） 已定项 7 登记表第 7 行：`N₀` 第一版取 1，算式 `⌈(3S − 1) × c ÷ 64⌉ + 1` 等计数模型验过再定（欠 C820（整理底额 `N₀` 的算式没有计数模型））。相关欠账：C2（无界操作做成单事务）、C91（整理意图与增量触发量的重做漂移）、C139（整理停摆之后释放也发不出去）、C142（第一个 reflink 之后整理会造出静默陈旧读）、C364（整理意图记录的字段没有逐字段对账）、C412（整理的挂钟代价没量过）。GC 意图的载体是 logged_ops 树（`research/prompts/m3-intent-r1-main-verification.md`；用户定案 K-I1：零根注册，原话在变更史，被攻过零轮）；字段表后来由判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1 在 D26（后台整理与放置回收） 已定项 2 定下（key 10 + value 34，条目 44）；`crates/singlefs-core/src/logged_operations.rs` 已有字段表的 `to_bytes` 与 `parse`，`transaction.rs` 已有写 logged_ops 条目的代码（`logged_operations_root_entries`）；删除落地的那套意图（`transaction.rs` 的 `DeletionStepOfThePublish`）只认 unlink，装不下 GC 意图；C364（整理意图记录的字段没有逐字段对账） 余下字段表与实现的逐字段对账与它会红的检查（I-4.4（意图只描述目标态） 的字段白名单判法已实现）。E10（老化：结构抗老化 vs 事后整理） 的跑前登记是 `research/prompts/e10-r1-prereg.md`（2026-10-13 写）；装置 `scripts/experiment-apparatus/e10_aging_compaction.rs` 与变异表 `scripts/mutation/tables/e10_aging_compaction.tsv`（21 行）已入库，`research/results/e10-aging-compaction-2026-10-13/` 下有 4 份分片产物，登记里写的实现臂 `crates/singlefs-checker-tier/src/bin/e10_aging_compaction_crates_arm.rs` 还没有；实验页里没有数值门槛，长期老化（挪进里程碑九「五　长期老化」）要的门槛由二·1 先登记出来。按关键词搜到、归属是推的（2026-10-12）：欠账 C122（分配记录不点删之后的条目数口径）、C133（搬迁的转发信息没有载体）、C140（意图完成不等于空段被腾出）、C141（停机判据的输入不持久）、C146（无空段时的回落政策全仓无定义）、C197（迁移的三笔写是不是一个事务没写下来）、C243（停机谓词那个量从来没有过轨迹）、C326（runs 换口径之后 E93 的数还能不能引）、C372（全空聚簇段数两份定义相反）、C373（停机谓词判真时活意图删不删没有条款）、C413（碎片度与全空段统计没有不变量）、C621（整理落地时要动的删除条款没有清单外的检查）；分项 D3（空间分配） 已定项 9、14，D5（快照 / 空间记账机制） 已定项 15。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 二·1 | 设计轮：整理意图记录的字段表（已定：D26（后台整理与放置回收） 已定项 2 的字段表，key 10 + value 34；I-4.4（意图只描述目标态） 的字段白名单判法已实现，仍欠字段表与实现的逐字段对账）；三条水位与 `N₀` 的取值（E10（老化：结构抗老化 vs 事后整理） 跑前登记）；discard 什么时候发（已定：发布返回之后、可再分配谓词为真的槽按整段发，D26（后台整理与放置回收） 未定项 9「discard 的时机」一段）；搬被共享的块怎么改中央映射（别家线索在 `.claude/kb/prior-art.md`「十、快照下的死块记账、搬被共享的块与 discard 时机」，未在本项目验证）。GC 批次在并行提交下怎么取号、堵不堵前台 fsync 不归里程碑四：里程碑六一·1 定并行形态时答（[06-sixth-txn.md](06-sixth-txn.md)） | D26（后台整理与放置回收） 已定项 1、2、3、5；D3（空间分配） 已定项 7；C364（整理意图记录的字段没有逐字段对账）、C91（整理意图与增量触发量的重做漂移）、C142（第一个 reflink 之后整理会造出静默陈旧读）、C139（整理停摆之后释放也发不出去）；C122（分配记录不点删之后的条目数口径）（2026-10-19 从里程碑三挪来，归属是推的，开工前核，仍欠的见登记行） | 判决与写回；E10（老化：结构抗老化 vs 事后整理） 跑前登记写成 | 搬共享块那一问等一·1；意图形态等里程碑三 |
| 二·2 | 实现：分三段——选段与搬迁计划写成纯函数、带 CPU 参照实现（GPU 垃圾回收在出口 8，见「七　GPU 垃圾回收」，拿这份纯函数当 CPU 参照），执行走意图分批；分配记录源侧与目的地侧各一个式子（D26（后台整理与放置回收） 已定项 5）；六个观测量（已定项 6）；discard（发布返回之后的 discard 已接上产品路径：`crates/singlefs-core/src/mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole`，用户改动的发布与整理显式入口之后各发一次；文件后端真打洞报不支持那一支没有用例，欠 C857（文件后端真打洞报不支持那一支没有用例）） | D26（后台整理与放置回收） 已定项 5、6、12；C2（无界操作做成单事务）；I-4.4（意图只描述目标态）、I-4.6（中间态其余不变量成立）；C323（镜像大小全仓没有条款）（checker 按探测容量复算统计项 10、11 那一半，归属是推的，开工前核） | 单测加变异；I-4.4（意图只描述目标态）、I-4.6（中间态其余不变量成立） 判 GC 意图有必红世界；I-3.1（已分配统计对得上）、I-7.4（近 K 代块未被复用） 在搬迁中间态成立 | 二·1；搬共享块那一支等一·2 |
| 二·3 | 崩溃流：GC 搬迁一条、搬被共享的块一条 | D13（验证路线） 已定项 4；C621（整理落地时要动的删除条款没有清单外的检查）（2026-10-19 从里程碑三挪来，归属是推的，开工前核，仍欠的见登记行） | 全域 0 红；必红世界（例：源落点在映射发布之前就被释放，判器红） | 二·2 |
| 二·4 | 性质：GC 在抽象文件系统状态上是恒等变换（搬完之后读到的内容逐字不变），在里程碑三七·1 选定的工具上证；模型对拍：随机历史穿插 GC，结果与不跑 GC 逐项相同 | D13（验证路线）；里程碑三七·1；C599（随机历史造不出被抛弃的根）、C679（卸载记号没进模型对拍）、C682（模型的映射树节点数上界只涨不落）、C830（M3 蕴涵式前提没有可满足性见证）（2026-10-19 从里程碑三挪来，归属是推的，开工前核，仍欠的见各登记行） | 证明产物入库；必出反例的世界（GC 改掉一个字节）；对拍有种子命中 | 二·2；里程碑三七·1 的工具 |

- **开工前要先定的**：GC 流的态数先估，对着崩溃放量今天的上限估（2026-10-12 现查）：一个落点至多 64 次写（`crates/singlefs-checker-tier/src/crash_judge_tables.rs` 的 `MAXIMUM_WRITES_PER_LOCATION`，组合掩码就是它）、一条流至多 2048 次写（`crash_facts.rs`）、全量展开深度 30（`scripts/crash/crash-amplification.sh` 的 `FULL_EXPAND_UP_TO`，段写数超过它报 `exhaustive=false`）、一段的状态数装进 u64（`crash.rs`）、GPU 内核展开至多 400 000 个表达式（`crash_judge_dispatch.rs`）、违例位 128 已用 92；GC 流与三·1 的闭环流哪一样撞上就先改装置；C412（整理的挂钟代价没量过） 在这个里程碑量不量；里程碑五（[05-fifth-txn.md](05-fifth-txn.md)「出口」第 3 行与段 0）要接手这里交出的「整理意图机制、区域扫描」，「区域扫描」在 D26（后台整理与放置回收） 正文里没有这个词，二·1 要定它指什么、交不交。

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:73-84`（整段抄，未转述）**

```markdown
## 三　生命周期闭环

- **原话**：「长期老化 生命周期闭环这两个可以。」（2026-10-01）
- **仓里已有的**：里程碑二固定脚本到 E 再正常卸载 1 662 648 564 态 0 红（`briefs/2026-09-28.md`「全量（2026-09-30）」一节）（2026-10-11 注：这条流后来改成 481 次写、全量枚举 1 678 377 204 个状态（`FULL_STATES_THROUGH_THE_UNMOUNT`；每次写只取两态的闭式是 1 678 377 089）；2026-10-11 全量 f 第 2 项按新流全域重判：现判 1 610 875 015 态、复用 67 502 189 态（合计恰为 1 678 377 204），0 红，见 `records/2026-10-11-崩溃放量两段与脚本审计.md`）；随机历史 `crates/singlefs-harness/src/history.rs` 的 `HistoryOperation`；QEMU 门禁 checker-tier-qemu-device-streams；E93（老化下的放置与碎片度） 的老化模型；D26（后台整理与放置回收） 已定项 6 的观测量里有 runs 与全空聚簇段数。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 三·1 | 固定脚本「生命周期闭环」：分配 → 覆盖写 → 建快照 → 再覆盖写 → 删文件 → GC 搬共享块 → 删快照 → 回收 → 在搬出来的空段上再分配；公共前缀模块与节点编码沿用 [02-second-txn.md](02-second-txn.md)「增补 4」 | D13（验证路线） 已定项 4 | 全域 0 红；必红世界 | 一·4、二·3 |

- **开工前要先定的**：随机历史加建快照、删快照、GC 三种操作之后，`KNOWN_RED_FORMS` 不新增。

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:85-117`（整段抄，未转述）**

```markdown
## 四　只读扫描重建（并行线）

- **原话**：用户 2026-10-01 在「真盘掉电核实和只读扫描重建，要不要作为里程碑四的并行线」那一问选「只读扫描重建」。
- **仓里已有的**：D21（权威态与派生态的分界） 已定项 13「救援、换树、迁移走同一条扫描重建路径」；D13（验证路线） 已定项 12 要 checker 做遍历与扫描两个方向；checker 的扫描方向按候选槽扫，读者不给候选时从单元区起点扫到盘尾（`crates/singlefs-checker/src/image.rs`、`walk.rs`），扫出来的供 I-7.8（根记录树 ID 水位不低于全池最大树 ID）、I-1.8（归并后版本全序） 与 journal 那几条判定用，另供只读扫描重建 `crates/singlefs-checker/src/rebuild.rs` 重建三张派生表（中央映射、每个 inode 的数据单元集合、inode 记录集合），与遍历方向的比对在 `walk.rs` 末尾（I-1.13（扫描重建与遍历方向的索引相同））。相关欠账：C113（扫描重建时多版单元的现行版本判定无输入）、C130（扫描重建的落点维没有任何度量）、C152（收窄后未发布号重发的扫描重建复核）、C301（deadlist 与 livelist 的态别没写）、C410（checker 两个方向共享什么没有检查）。按关键词搜到、归属是推的（2026-10-12）：欠账 C86（两种单元大小与扫描步进对不拢）、C99（打包共享单元里的记录扫描时发现不了）、C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据）、C109（哪些树的 key 死亡要写墓碑）、C615（扫盘重建对挂着时的回退没有输入）、C768（退役记录的扫描重建读侧没有消费者）；不变量 I-1.9（清扫水位内无可读垃圾）。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 四·1 | 题面：从单元扫出索引（只读，不写回），与现行索引逐项比对；多版单元的现行版本照 C113（扫描重建时多版单元的现行版本判定无输入） 定案；`walk.rs` 的扫描方向罩到哪已现查（2026-10-12，见同一节「仓里已有的」），起点只是把 checker 现有的候选槽扫描接上重建 | D21（权威态与派生态的分界） 已定项 13；D13（验证路线） 已定项 12；C410（checker 两个方向共享什么没有检查）；C464（I-1.8 对码 1 的全序键写窄了）（2026-10-19 从里程碑三挪来，归属是推的，开工前核，仍欠的见登记行） | 题面写成 | 无 |
| 四·2 | 实现与比对：固定脚本与随机历史的每个镜像上比对 | C113（扫描重建时多版单元的现行版本判定无输入）、C152（收窄后未发布号重发的扫描重建复核）；C860（扫描重建认删除中只凭扫描一侧） | 零差异；必红世界：改掉一个派生态条目，比对红；抹掉一个自描述字段，重建判损坏 | 四·1；deadlist 条目与被钉住单元的映射条目是权威态（C301（deadlist 与 livelist 的态别没写） 已定这一半，扫描不重建它们），销毁队列（派生态）第一版不重建、不比对，livelist 那一部分等 C301（deadlist 与 livelist 的态别没写） 的另一半定 |

- **开工前要先定的**：和遍历方向共享什么（只许共享格式解析，D13（验证路线） 已定项 12）。

### 四·1 题面（主 agent 2026-10-13 写，按 `crates/` 现状）

**题**：只读的扫描重建——不读根环与树表，只从单元区扫出的单元头重建「索引」，与遍历方向从最新根读出的索引逐项比对，零差异。不写回盘（救援的写回不在出口）。

**重建什么**（第一版三张派生表，都从扫出的单元头与载荷现算）：
1. 中央映射：每个已发布的码 1 / 码 3 单元 → 它的落点（设备, 槽）。单元头里有中央映射 key 的全部输入（`crates/singlefs-core/src/records.rs` 的 `:347`、`:359` 两条 key 规则读的字段），同一 key 多版按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 择现行（单元写序 (实例代号, 事务号) + 实例表已发布谓词，`crates/singlefs-checker/src/walk.rs` 的 `PublishedPredicate` 与 `scanned_content_units` 已算到这一步）。
2. 每个 inode 的数据单元集合（extent 树的叶内容）：从码 1 单元头的对象 key（`ScannedContentUnit.object_key`）归并，按 I-1.8（归并后版本全序） 的全序键取现行；与遍历方向从 extent 树读出的 (inode, 偏移) → 数据单元逐项比。
3. inode 记录集合：从 inode 叶容器（码 3，打包记录类型 2）的现行版本解出每条记录；与遍历方向从 inode 树读出的逐项比。
分配记录树、记账树是派生态（D21（权威态与派生态的分界） 已定项 5），第一版不重建、不比对（账在 C122（分配记录不点删之后的条目数口径）、C413（碎片度与全空段统计没有不变量））；deadlist 条目与被钉住单元的映射条目是权威态，不由扫描重建；销毁队列（带销毁中记号的码 14 条目）是派生态，第一版不重建、不比对；livelist（可写克隆）等 C301（deadlist 与 livelist 的态别没写） 的另一半定了再比。

**输入与禁区**：输入只有 `ImageReader`（`crates/singlefs-checker/src/image.rs`）的设备、几何、候选槽；实例表行从最新根读（已发布谓词要它，与今天 I-1.8（归并后版本全序） 那一遍同一来源，这是重建方允许读根的唯一一处，写明在函数文档里）；不读树表、不读任何树节点（码 2）。扫描方向与遍历方向共享的只有格式解析（D13（验证路线） 已定项 12）：重建代码只许调 `singlefs_format` 常量与 checker 里解单元头的函数（`unit_header_of` 一族），不许调走树的函数（`walk_*`、`IndexNodeCache`）；这一条由一个单测钉（重建模块的 `use` 清单与一张白名单逐项比，C410（checker 两个方向共享什么没有检查） 的会红形态）。

**比对与判定**：新不变量一条（编号是 I-1.13（扫描重建与遍历方向的索引相同））：「扫描重建的三张表与遍历方向读出的三张表逐项相同」，判定 Holds / Violated（列出第一处差异：表名、key、两边的值）/ NotApplicable（最新根读不出、实例表读不全）。接进 `check_pool_image` 的末尾，与别的不变量同一份 `judgements`；崩溃点重放每个状态都跑（代价：多一遍全盘扫，候选槽由 harness 给、不是真全盘）。

**必红世界**（各一个，先证会红）：① 改掉一条派生态条目——把遍历方向读到的中央映射里一条条目的落点改成另一个槽（镜像里改映射叶的那个字段），比对红在中央映射那张表；② 抹掉一个自描述字段——把一个码 1 单元头的对象 key 清零，重建判「单元损坏、不认领」，那条 key 两边数量不同，比对红；③ 共享检查：往重建模块里加一行调 `walk_*` 的代码，白名单单测红。

**射程**：固定脚本流与随机历史的每个镜像上比对（四·2 验收）；多版单元只按 P2 择，谱系记录按「不是任何现行版本的成员」认领（C615（扫盘重建对挂着时的回退没有输入））、退役记录按退役树认领（C768（退役记录的扫描重建读侧没有消费者）），这两格真判（用户 2026-10-14 定甲，岔路单 `research/prompts/m4-batch2-code-r1-forks.md` 第 1 行）；加密关着（D9（加密） 已定项 10）。

**落点**：新模块 `crates/singlefs-checker/src/rebuild.rs`（纯函数：扫出的单元 → 三张表），比对在 `walk.rs` 末尾一处调用；不变量行加进 `crates/singlefs-checker/invariants/01-块自描述.md`。要动的文件：`crates/singlefs-checker/src/rebuild.rs`（新）、`walk.rs`、`image.rs`（`IMPLEMENTED_INVARIANTS` 加一条）、`lib.rs`、`crates/singlefs-checker/invariants/01-块自描述.md`、`crates/singlefs-harness/tests/`（必红世界用例）、`crates/mutations.tsv`。

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:118-133`（整段抄，未转述）**

```markdown
## 六　截断（truncate）

- **原话**：里程碑三写「第一版只做 unlink、不带 truncate（用户 2026-09-28 定；截断到 N 字节留给下一个里程碑，同一套意图机制）」（[03-third-txn.md](03-third-txn.md)「三　删除（新事务）」）；用户 2026-10-12 定放进这个里程碑。
- **仓里已有的**：意图与分批回收由里程碑三的删除落地（D8（核心索引结构） 已定项 5）。C778（不改 size 的部分删除怎么记没有条款） 那一行转述 D18（块里携带什么信息） 已定项 10「记录粒度」：truncate / punch 这类部分删除不写区间墓碑，读法 Z 管文件变短（扫描重建时数据单元按 inode 记录的对象出生代与 size 判死活）。punch（不改 size 的部分删除）不在这一份：C778（不改 size 的部分删除怎么记没有条款） 用户 2026-10-12 定挂里程碑十一（目录与 POSIX 面）。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 六·1 | 设计轮（三方）：截断的意图目标态（「inode N：size = n，n 之后无 extent」这类，推的）；截断点落在一个数据单元中间时尾单元怎么 COW；被快照引用的尾部块进 deadlist；扫描重建怎么按读法 Z 判截掉的单元 | D8（核心索引结构） 已定项 5；D18（块里携带什么信息） 已定项 10；D5（快照 / 空间记账机制） | 判决落 `research/prompts/`，定案写回 kb；有实验才有决策 | 段 0；尾部块进 deadlist 那一问随一·1 |
| 六·2 | 实现 core：单文件 `truncate`（按 inode 号）走删除的意图与分批回收；`crates/singlefs-harness/src/model.rs` 加截断操作，RefFS 同步 | 六·1 的定案 | 每条新测试先证会红；模型对拍逐项相同 | 六·1 |
| 六·3 | checker 与崩溃流：截断中间崩溃的流接进崩溃放量；新盘上形态登记进 `04-` 那一份字节表 | D13（验证路线） 已定项 9、15 | 全域 0 红；改坏「最后一批与截断意图同事务」那一处，那条流的判器红 | 六·2 |

- **变长**：进这一份（用户 2026-10-16 定「变长也做」），size 改大、尾部留洞，上段叶条目标签 3、4 声明洞。
- **设计判决**：六·1 两轮。`research/prompts/m4-trunc-r1-main-verification.md`（K1–K12，补定 S1–S5）定意图载体、先落目标、尾单元、洞、读法 Z 加写序、checker 五条判、崩溃流三条；`research/prompts/m4-trunc-r2-main-verification.md`（R1–R8，用户定案在第六节）补截在单元边界再变长那一形：inode 记录偏移 120–135 放截断字段（txg、size），读法 Z 加第四条，新开 incompat 位 2。两轮的形态都被攻过零轮，代码轮再攻；定案写回 kb 归验证段绿了之后的阶段同步。
- **实现进度**：写段原在 m4-predev 工作树里做，已合进 master（提交 0b926bd3f；过程见 `records/2026-10-13-里程碑四预开发.md` 第四节「截断」）。m4-trunc-r1 那一份的核心层、模型与 RefFS、池级 checker、接线、GPU 判器与恢复内核、三条崩溃流已打进；m4-trunc-r2 那一份（核心层、池级 checker、模型、GPU）还没派，等另一会话的崩溃点重放全量跑完；验证段没开。钉截在边界那一形的 `every_step_image_of_the_fixed_truncation_history_is_green_under_the_pool_checker` 今天红，等 m4-trunc-r2 的实现。

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:134-150`（整段抄，未转述）**

```markdown
## 七　GPU 垃圾回收

- **原话**：用户 2026-10-19：「gpu垃圾回收需要加入。不需要讨论。」「这个不是默认开启，开启了之后就可以用。」「还有 gpu可以设置vram大小」「还可以设置哪一张显卡 不设置的话 用全部」「放入里程碑4的出口」；弹窗四问的答案与定案在 D26（后台整理与放置回收） 已定项 13，原话在变更史。
- **仓里已有的**（2026-10-19 现查）：GC 的发布入口 `publish_relocation_batch` 在 `crates/singlefs-core/src/transaction.rs`；区域扫描 `region_scan.rs`、选段与计划 `compaction_plan.rs` 是纯函数（GPU 的口子）。挂载走库调用（`crates/singlefs-core/src/mount.rs`），没有挂载参数这一层，命令行要到里程碑十的 FUSE。整理三水位住系统运行配置、第一版恒 0（`system_configuration.rs` 的 `compaction_watermarks`）。产品代码里没有 GPU 依赖：`wgpu` 只在 `crates/singlefs-checker-tier/Cargo.toml`（特性 `gpu`）。一条意图的 R 是一个聚簇段 64 槽、至多 2 MiB（`research/prompts/m4-gc-r1-main-verification.md` G1），而 E21（GPU 卸载的净收益） 量到传输地板要 ≥ 64 MiB 的批，所以 GPU 模式要攒批。C412（整理的挂钟代价没量过） 没量。本机 ZNS 不在这一节（`/sys/block/nvme0n1/queue/zoned` 为 `none`）。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 七·1 | 量：在 `crates/` 现有的 GC 路径上量一批搬迁的挂钟，拆成读写 I/O 与 CPU（解单元头、核校验和、查映射、分配记录树写放大）各几成；口径写明设备、负载、批大小（C412（整理的挂钟代价没量过） 的检查列） | C412（整理的挂钟代价没量过）；D24（后台重活能不能卸给 GPU） 已定项 3 的判据形式 | 断言钉绝对值；批大小翻倍挂钟跟着涨（判别力自证） | 二·2 的发布路径 |
| 七·6 | 发布路径按身份查：`TransactionOutput::unit`、整理读版本时逐 key 查中央映射（`version_as_the_compaction_reads_it`）、发布照抄数据单元（`carried_unit`）、分配记录树结算里的 `mapping_key_carried_from`、`file_other_than_the_first` 五处由在 Vec 上 `iter().find` 改成按身份索引，行为不变（用户 2026-10-20 定「先修四处线性查找」）；另加两处同样改：整理发批核搬迁源落点 `relocation_checked_against_the_version` 按 key 线性找角色、`mapping_locations_of_a_mapped_unit` 线性找 key（三个调用点，含快照判定那一圈进 deadlist 那一支）（用户 2026-10-21 定并进，`research/prompts/m4-publish-lookup-code-r2-main-verification.md` 第五节）；再加五处：整理发批与恢复时 `take_the_mapping_positions_over_the_location_hints` 按角色在 `units` 上线性找、`publish_admitted` 里对重写的每个角色在 `units` 上线性找两处，与随一次写的单元数平方涨的三处 `.contains`（留给快照的角色清单两处、记录点名的角色清单一处）（用户 2026-10-22 定并进，`research/prompts/m4-publish-lookup-followup-code-r1-main-verification.md` 第五节）；增长判据与变异数产品调用点上的次数，不数测试自己另建的索引 | `.claude/rules/fs-design.md`「记账是事务的副产品」；七·1 第二段的发现（`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 2026-10-20 调查员交回那一条） | 改前改后同一串操作写出的盘镜像逐字节相同；每处改法一条会红的变异；一批 CPU 随池大小的增长从平方降到接近线性（副本推翻实验：P = 4096 一批 217.47 → 62.38 ms），改完重跑 E185（GC一批搬迁的挂钟拆分） 第二段 | 七·1 第二段 |
| 七·2 | 设计轮：GPU 上算哪几段；攒批的形态（几批连起来交、还是放宽 R）与显存上限的关系；多张卡怎么分活；GPU 结果两份核比对的形态（第二份核怎么写才算独立；一张卡时两份都在这张卡上跑，几张卡时怎么分，不许要求多张卡；比对放在 GPU 上还是 CPU 上）与连续几轮不确定才交 CPU 判的 N（D26（后台整理与放置回收） 已定项 13 第 8 条）；控制接口（挂着切换）的形态；定时兜底间隔、写带宽上限、显存上限的默认值；GPU 代码住哪（推的：单开一个可选的包，core 不带它也能编，守 D24（后台重活能不能卸给 GPU） 已定项 2 第 1 条）；GPU 路径默认 WGSL 经 `wgpu`（现成，checker 档的崩溃判器在用），有更好的做法在七·2 的设计轮比过再换 | D26（后台整理与放置回收） 已定项 13；D24（后台重活能不能卸给 GPU） 已定项 2 | 判决落 `research/prompts/`，定案写回 kb；有实验才有决策 | 七·1 |
| 七·3 | 实现：三个挂载参数与挂着切换（下一批生效）；定时兜底；写带宽上限与前台 fsync 让路；GPU 路径加 CPU 参照；两份独立的 GPU 核各算一次比对，确认的交 CPU 改位图，不确定的留到下一轮，连续 N 轮不确定的交 CPU 判（D26（后台整理与放置回收） 已定项 13 第 8 条）；挂载时与运行中的报错不降级；分支能被测试强制进入、运行时看得出走了哪条（`.claude/rules/fs-design.md` 五条硬要求） | D26（后台整理与放置回收） 已定项 13；七·2 的定案 | 两条路径在同一组输入上逐项相同、对拍进快档；往 GPU 一侧塞已知故障对拍红；点名不存在的卡、上限超过那张卡的显存总量、上限个数与卡数不等各拒挂；卡上空闲显存不到上限时照常跑、只用空闲的；每张卡上 GC 占的显存峰值（攒批、缓冲、暂存区一起算）不超过它的上限，钉绝对值的断言，塞一个多分配一块缓冲的故障要红；运行中注入 GPU 出错，那一批放弃、GC 停下报错；塞一个让其中一份核在一个条目上算错的故障，那一条记状态不确定、这一批不回收不搬，下一轮去掉故障后回收；同一条连续 N 轮都算错，第 N 轮交 CPU 判、按 CPU 的判定处理；每批报的确认、不确定、交 CPU 判三个数钉绝对值；只点一张卡与点几张卡各跑一遍，判定逐项相同；模型对拍 gpu 模式与 cpu 模式读到的内容逐项相同 | 七·2 |
| 七·4 | 覆盖：GPU 模式下的 GC 搬迁流、批间切 cpu ↔ gpu 一条，接崩溃放量；两种模式写出的块序列逐字节相同 | D13（验证路线） 已定项 4 | 全域 0 红；必红世界（GPU 计划改掉一个源落点，块序列比对红） | 七·3、二·3 |
| 七·5 | 对照组（用户 2026-10-19 加）：同一负载、同一池，`background_compute=cpu` 与 `background_compute=gpu` 各跑，报 GC 挂钟、GC 占的主机 CPU 时间（GPU 模式含准备、提交、等结果、两份结果的比对与交 CPU 判的那几条）、前台 fsync 延迟；几档盘容量各跑一遍，看省下的 CPU 时间随容量怎么变 | D26（后台整理与放置回收） 已定项 13 第 7 条 | 跑前登记写死口径；钉绝对值的断言；阳性对照：CPU 一侧故意把每个确认的条目再判一遍，省下的 CPU 时间必须变少；对照组的数写进仓根 README 与当期简报（`briefs/`），三语一起改、用词照 `.claude/kb/translation-glossary.md`（用户 2026-10-19：「对照组的实验数据 要写入readme和briefs」） | 七·3 |

- **开工前要先定的**：产品里多卡的先后不读仓根 `multi-host.env`（那是测试装置的配置），按什么排要七·2 定。七·2 定控制接口时，三个挂载参数只有一组、管全部后台任务（D26（后台整理与放置回收） 已定项 13 第 1 到 3 条），一个开关一次切全部；后台任务种类与计算后端各做成封闭枚举，里程碑七的 GPU 打包搬迁接同一套，加一种任务只加一个枚举成员，不回头改这一份的接口（[07-seventh-txn.md](07-seventh-txn.md)「三　GPU 打包搬迁」）。

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:151-167`（整段抄，未转述）**

```markdown
## 八　接手里程碑三的删除与验证线欠账（2026-10-19 从里程碑三挪来，不在出口）

- **原话**：用户弹窗：不在出口字面的 79 笔「挪到各自主题的后续里程碑」（落进项八的是其中删除、崩溃点系统、形式化、待优化这几个主题的 51 笔，别的笔落各自主题的作业点）；C407（三 oracle 合谋测试的判官没进门禁）、C447（规约执行器没接进崩溃点重放与 RefFS 比对）、C409（常规层、大负载层在门禁里没有阶段）「挪到里程碑4」。
- **次序**：不进「分组与次序」那张表的段 0–3，排在段 3 之后做；八·1 到八·6 之间没有写死的依赖，各作业点的依赖列为准（推的，开工前核）。
- **作业点**：

| # | 做什么 | 压着的条款与欠账 | 验收（先证会红） | 依赖 |
|---|---|---|---|---|
| 八·1 | 删除机制的连带欠账：墓碑树、谱系与退役容器、inode 号水位几条判据的空白，各补会红的用例；回退留下的墓碑有没有预付罩着；墓碑树根长高那次发布的崩溃流；E84（打包墓碑的钉住量） 装置的容量常量 | C614（回退时墓碑树取错与删完对象无墓碑单镜像判不了）、C617（删后回退留下的墓碑没有预付罩着）、C619（墓碑树根长高那次发布没有崩溃流）、C691（打包墓碑装置把容量写死成 583）、C715（墓碑死亡 txg 的等式没有检查）、C716（树内最大 key 每版各比自己的水位）、C725（删除态扫描去重键里的 txg 没有取样点）、C769（水位台阶行的草案 ③ 判不出历史没写）、C770（退役完备两次攻方抽样都没进模型）、C772（冷读不判这一版自己那一行水位在）、C781（跨版本容器集包含没有判法）、C782（产品支只有一片容器时三判不判）、C783（容量旋钮在历史中途改变没有条款）、C784（只判最右之前 k 片的变异有限组关不死）、C785（水位行缺那一判在完整读者里轮不到先报）（从里程碑三三·3、三·4、三·5、三·6 挪来，仍欠的见各登记行） | 照各登记行「怎么拦」：要先定条款的先定（三方或交用户）；每条判据一张坏镜像、`crates/mutations.tsv` 一行，先证会红；C619（墓碑树根长高那次发布没有崩溃流） 那条流接进崩溃放量全域 0 红 | 段 0 |
| 八·2 | 崩溃点重放与 GPU 判器的判别力：已登记的流与 GPU 对拍补必红世界与变异行；快档编进来几条照实报；全量出的红钉进钉表；复用与剪枝有关掉再判的对照；记录核对器按记录自己的实例与 txg 归发布 | C588（记录核对器按写表次序错归发布）、C592（单独拿掉单元写与记录间那道屏障今天没判据会红）、C604（快档不开特性，崩溃放量与 GPU 对拍编进来零条）、C605（崩溃放量的复用与剪枝没有关掉再判的对照）、C609（全量出的红没有统一的钉表）、C623（崩溃放量取不出复用数没有棘轮）、C651（没有可用槽 0 时 checker 与实现找槽 1 读法不同）、C658（崩溃注入第三截放过丢掉的一步复活）、C720（GPU 树表偏移不重叠没有断言）、C721（记录核对器判据在不变量清单里没有对应条目）、C729（F3 的取样罩不罩得住更长历史与别的种子基）、C731（journal_tail 不逐盘）、C732（GPU 恢复内核多层树三处判点没有变异行）、C735（GPU 判器里由记录施加的版本没有对拍流与变异行）、C744（扫描内核单调用点改法的对拍流缺三类状态）、C749（F1 拒因那一格崩溃注入计数没有钉）、C752（映射现算那一拒的镜像复算较新根一臂没用例）（从里程碑三六·3、六·5 挪来，仍欠的见各登记行） | 照各登记行「怎么拦」：改坏实现或改坏判器，那条流的判器必须红，进 `crates/mutations.tsv` 或 checker 档用例；GPU 对拍新加的流先证 GPU 一侧改坏会红 | 段 0 |
| 八·3 | 崩溃点重放分层与崩溃放量调优：常规层、大负载层各立一个门禁阶段；崩溃放量入口报展开深度与分份，一律进快档的类逐类核，全量写明谁来派，观察者每态代价进入口登记表，双机导入的块只数一次；长跑的项按挂钟打进度；带一次覆盖写的崩溃点重放两臂各跑一趟 | C409（常规层、大负载层在门禁里没有阶段）（用户定挪里程碑四；负载形态等 `research/prompts/m3-close-c409-r1-forks.md` 的 G1–G4 与 E184（常规层与大负载层的代价计数） 的代价数）、C606（崩溃放量入口不写展开深度，缩过规模的全量照样退 0）、C611（长跑的项不按挂钟打进度）、C624（一律进快档的几类没逐类核在不在快档）、C645（崩溃放量全量没有人能派）、C741（带一次覆盖写的崩溃点重放没跑）、C786（按计划导入几份库同一块各数一次）、C789（观察者（J4）内联跑的代价没量就接进全量）（从里程碑三六·2、六·4、六·5 与「待优化」第 22 行挪来，仍欠的见各登记行） | C409（常规层、大负载层在门禁里没有阶段）：两层各一个门禁阶段，无对象时退 77 明说未跑；常规层的成功行报抽了几个崩溃点、分在哪几个桶，没抽到的桶逐个列名；判别力：常规层抽样数设成 0，阶段必须判红。其余照各登记行「怎么拦」 | 段 0；C409（常规层、大负载层在门禁里没有阶段） 等 m3-close-c409 下一轮判决 |
| 八·4 | 三条 oracle 的独立性：O3（独立规约执行器） 对崩溃点重放的每个状态出判定、与 RefFS 逐个比对；E171（三oracle合谋测试） 的判官接进门禁；执行器批补两条路径的态，见证判据按现行条款重判，交回值接进 O1（内存模型对拍（HashMap 理想文件系统）） | C447（规约执行器没接进崩溃点重放与 RefFS 比对）、C407（三 oracle 合谋测试的判官没进门禁）（两笔用户定挪里程碑四）、C661（规约执行器批里两条路径没有态）、C672（交回值或内存状态错而写流不变的一类）、C688（可写挂载见证判据的条款与实现对不上）、C730（第 25 批零轮形态里没被攻方打过的几条）（从里程碑三七·6 挪来，仍欠的见各登记行） | C447（规约执行器没接进崩溃点重放与 RefFS 比对）：没有对象时门禁阶段退 77 明说未实现；有对象之后不一致的落进 C191（同源性计数器只盖了一对） 那张表，喂一个「执行器放行、RefFS 判红」的状态必须判红。C407（三 oracle 合谋测试的判官没进门禁）：判官读 E171（三oracle合谋测试） 复跑新写的比对器产物，判登记集合、三列各有只有它红的、对照不失灵三格，判别力照登记行逐格改产物各红一次；重型，只在提交或用户要求时跟着复跑跑 | 段 0 |
| 八·5 | 形式化工具的欠账：门禁 checker-tier-formal-verification 那一道「被信号停了」「内存排不上」「提交时第 1 步停下」三种结局的归类、样本与频率；重放前缀核与 core 里同一概念的类型合成一处并复证；E165（恢复重放有界穷举与变异杀伤界） 里「只坏第一次读」那一族的态数 | C630（提交时形式化那一道第 1 步停下的频率没量）、C631（内存排不上时形式化那一道最坏约 12 小时才报没跑成）、C633（形式化那一道被信号停了的归类与出路文字没对齐）、C647（重放前缀核与 core 里同一概念各一个类型）、C666（只坏第一次读那一种读故障时机没枚举）（从里程碑三七·2、七·3、七·5 挪来，仍欠的见各登记行） | 照各登记行「怎么拦」：阶段层各补一份样本（假包装排不上内存、被信号停），判没跑成而不是通过；类型合成一处之后 Verus 复证仍过；E165（恢复重放有界穷举与变异杀伤界） 产物报那一族择到另一槽的态数，为 0 判红 | 段 0；二·4 要用这道门禁 |
| 八·6 | 流程、门禁与文档的待优化：提交流程里单列归档实验记录那一步；测试文件头的来历改成指向 kb，加结构判据；变更史搬迁后旧的「日期加其N」自引用逐条改指 | C602（归档实验记录只在整轮门禁里判）、C610（测试文件头还写着来历）、C591（变更史搬迁后旧的日期加其N自引用没逐条改点）（从里程碑三「待优化」第 24、34 行与登记行要新加的那一行挪来，仍欠的见各登记行） | 照各登记行「怎么拦」：各一份判别力样本，先证会红 | 无 |
| 八·7 | 2026-10-21 doc 门禁违规修复里超出那一轮的部分（用户定「超出的部分 记入里程碑4的收口」，调度记录 `records/2026-10-21-doc门禁违规修复.md`）：① 截断那一批的字段（截断条目、N_w、截断字段、incompat 位 2）没写进任何中文决策，D8（核心索引结构） 的记录字段表偏移 120 还写「预留 20」，I-4.10、I-9.39 点不出字段落点、I-4.4、I-4.6、I-9.7 只锚了一半；② E182（被抛弃根读不出的三条岔路）页 19 种 58 处、E184（常规层与大负载层的代价计数）页 6 种 7 处还用改名前的产物键名（例 `question=Q1`），「逐行相同」「diff 无输出」两句对改名后的产物不成立；③ E159 页把页内岔路标号登记成全局编号、让 J1、J4 撞号，E16 与 E158 的 G0 共用一处登记位；④ `research/prompts/m4-gc-r1-main-verification.md` 两行、`m4-snapshot-r1-main-verification.md` 一行回看格写「不受影响」没带理由；⑤ E183 第三段 H5 产物没登记进 `scripts/experiment-apparatus/replay.sh`（页上 21 行整行引文因此判不过），变更史 E184 节里有一项说的是 E183 第三段；⑥ E156 页贴的两份锚点脚本 sha256 与现文件对不上，E183 页引的单测行号旧（2660，现 6386），E181 页写 18 条自证、17 条变异行（现查 28、35）；⑦ E10、E137 的装置这一批改了、`research/results/` 里没有不比它旧的产物（用户定那一轮不修）；⑧ `scripts/kb/relabel-item.py` 调 strict 的 `load_map()`，真仓上遇到读不出首行的译文退 1；⑨ clerk 用 Bash 跑 `scripts/kb/replace-once.py` 写进了 `.claude/kb/`，越出它定义的写范围而写闸（`.claude/hooks/write-guard.sh` 只挂 Write / Edit）看不见；⑩ 门禁 doc-process-records 的 crates-adversarial-review 报 27 个产品代码文件只在判决正文提到、没列进「## 这一轮被判过的文件」（用户定那一轮先不关）；⑪ kb 译文上的违规与 doc-text 的 trilingual 四格（用户定「译文应该也一样判 这次可以豁免」）；⑫ 两份未跟踪的自证样本形态文件 `.claude/kb/decisions/01-甲.ja.md`、`03-丙.en.md` 落在真 kb 里，来历未查 | 各条点名的文件与门禁格；doc-violations 盘点表 `~/.local/share/singlefs/work/doc-violations-inventory/inventory.tsv` | 逐条：①先定分项再补不变量锚点、门禁 doc-registries 的 invariant-anchors 格绿；②③④⑤⑥ 对应的门禁格与 doc-lint 在中文原文上不再报；⑦ 复跑产物入库、evidence-in-repo 格绿；⑧ 给一个首行读不出的译文样本、脚本不退 1；⑨ 写范围在 Bash 写上会红的检查；⑩ 补判决或用户逐份豁免；⑪ 译文照中文原文改到四格与 doc-lint 绿；⑫ 查清是哪道自证漏写、删掉并补会红的检查 | 无 |
| 八·8 | 2026-10-23 门禁 code-tooling 自证覆盖格修正里没做完的部分（用户定「未做完的写入里程碑4收口」，调度记录 `records/2026-10-23-门禁自证覆盖格修正.md`）：① 门禁 harness-model-differential-and-scenarios 样本档的 `seed.record`、门禁 `checker-tier-crash-point-replay` 清场没过时的 `feature_log`，判红时改成拷到 `TMPDIR` 下再指路，只核了语法、没跑到那一支；② 门禁 doc-text 的 clock-times 格在 records 与 kb 的 67 份文件里报 6620 行钟点、时区词与时间戳，不在那一轮改过的文件里，没有认领的会话 | 门禁 harness-model-differential-and-scenarios、`checker-tier-crash-point-replay`、doc-text 的 clock-times 格 | 逐条：① 各造一个让那一步失败的输入（verdict-record 退非 0、清场回读不过），判红后出路里的路径指得到文件；② clock-times 格在真仓上绿，或逐份登记放行理由 | 无 |

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:168-178`（整段抄，未转述）**

```markdown
## 优化项：不在出口，进入条件已登记

用户 2026-10-19 定 GPU GC 进出口 8（「七　GPU 垃圾回收」），这张表只剩另外三样，另加 2026-10-20 七·1 第二段查出来的「发布不再整版照抄」一样。用户 2026-10-01 定：「gpu gc只是优化项目。 同样ZNS / FTL SSD 算法 也是特性ssd的优化项。 所以这里我们提前计划好。」这几样不进出口，GC 设计里留口子、不写占位代码（`.claude/singlefs-ai-sop/rules/code-discipline.md`「用不到的代码删掉」）。

| 优化项 | GC 里留的口子 | 进入条件 |
|---|---|---|
| 发布不再整版照抄 | 七·6 之后一批 CPU 仍随池大小线性涨：版本里装着每个单元的字节，每次发布照抄、克隆全部单元（`transaction.rs` `units: Vec<PublishedUnit>` 注释「没重写的从上一版照抄」），整理每批两次整版克隆（`advance_compaction_by_one_publish`），分配记录树每次结算三轮、每轮按全部分配记录重新分组（`settle_the_allocation_record_tree`）。改成只放这次改的单元、没改的与上一版共享，是 `TransactionOutput` 的形态改动 | 七·6 改完、E185（GC一批搬迁的挂钟拆分） 第二段重跑之后，一批 CPU 里这一层仍占大头；先开设计轮三方再实现 |
| ZNS / FDP | 聚簇段保持「段内只追加、整段回收」，与 zone reset 对得上（推的，二·1 里核） | D10（随机写 / 碎片化的真答案） 已定项 1 定 FDP / ZNS 是「有就用的加速」；D12（目标介质） 已定项 1 每种介质一个 incompat 位；有设备可测 |
| 交给盘的 FTL | 二·2 的 discard（D26（后台整理与放置回收） 已定项 7 那条「只打标记、发指令」的底座） | D26（后台整理与放置回收） 未定项 8、9、10 定了 |
| FTL 模拟器 | `research/` 下的装置，把 singlefs 的写流与 GC 搬迁喂进去量盘内部的写放大 | GC 落地之后、交给盘那条线开工之前；它的结果就是 D26（后台整理与放置回收） 未定项 8、9 的输入 |

```

**出处 `.claude/kb/milestone/04-fourth-txn.md:179-197`（整段抄，未转述）**

```markdown
## 接手里程碑三：三条接口问题

这三问原定在里程碑三三·1（删除的设计轮）开轮时放进材料；删除三轮的材料里搜不到它们（2026-10-12 现查），答案现状写在「三问的答案现状」一段：

| # | 问题 | 答「否」时里程碑四要改什么 |
|---|---|---|
| 1 | 意图记录的形态能不能装下第二个使用者：GC 那条「`R` 内不存在分配代 ≤ `c0` 且未释放的落点」（D26（后台整理与放置回收） 已定项 2） | 回头改意图的格式与 I-4.4（意图只描述目标态）、I-4.6（中间态其余不变量成立） 的判法 |
| 2 | 删除的回收路径有没有写死「删即释放」：有快照之后，被快照引用的块要进 deadlist（D5（快照 / 空间记账机制） 已定项 13、19） | 回头改删除的分批回收 |
| 3 | 后台事务（删除批次；之后是 GC 与删快照）在并行提交下怎么取号、会不会堵前台 fsync（D13（验证路线） 已定项 2；并行提交的形态归里程碑六，[06-sixth-txn.md](06-sixth-txn.md)） | GC 与删快照另立一套排序规则 |
| 4 | 观察者录入与结论的版本里拼着一个手写版本串 `OBSERVER_DEDUPLICATION_SCHEME`（`crates/singlefs-checker-tier/src/crash_amplification.rs`），与 D13（验证路线） 已定项 17「手写的版本常量一律不进键」相抵；用户定：现有的留着写明白、不经用户说不升，里程碑四收尾时删掉 | 把它从观察者版本的拼法里去掉（版本只由观察者单元键与另读的输入组成）；去掉那一次库里全部观察者块与观察者结论重录、重判一次，排在里程碑四收尾那一趟全量里 |

**三问的答案现状**（2026-10-12 现查；第 1 问一栏后来更新，第 2 问一栏「没有按快照分支」已不成立，一·2 已按快照分支）：

| # | 答案 | 出处 | 里程碑四要做的 |
|---|---|---|---|
| 1 | 能：意图分承诺类与提示类，GC 意图走 logged_ops 树；用户定案 K-I1 让它零根注册（被攻过零轮），字段表已由判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1 定在 D26（后台整理与放置回收） 已定项 2 | `research/prompts/m3-intent-r1-main-verification.md`；`crates/singlefs-core/src/transaction.rs` 已有写 logged_ops 条目的代码（`logged_operations_root_entries`） | 字段表已定，I-4.4（意图只描述目标态） 的字段白名单判法已实现，仍欠字段表与实现的逐字段对账（C364（整理意图记录的字段没有逐字段对账）） |
| 2 | 没有一轮答过；代码里删除批与覆盖写共用一处 `allocator.release`，没有按快照分支 | `crates/singlefs-core/src/transaction.rs` 的 `for released in settled.release` | 按「否」办：一·2 改这处共用释放（推的，没量过） |
| 3 | 串行下由 D8（核心索引结构） 已定项 16 罩住（删除各步取号是发布路径上的 +1）；并行那一半、GC 与删快照的取号没答 | `research/prompts/m3-conc-r1-main-verification.md` 里删除各步取号那一格；[06-sixth-txn.md](06-sixth-txn.md) 一·9 与「接里程碑四的接口」一节 | 里程碑四按串行设计，不答；里程碑六一·1 答 |

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:1-50`（整段抄，未转述）**

````markdown
## E185 GC一批搬迁的挂钟拆分 —— 第 2 次重跑已跑（占比 0.25 的几何改负载后造成了，单点核与第一、二段跑完；问题单四行都够判：第 1、2 行读这一趟，第 3、4 行沿用第 1 次重跑的产物；第三、四段够判后不跑）

**中文** · [English](185-wall-clock-breakdown-of-a-batch-of-gc-migrations.en.md) · [日本語](185-GC-一バッチ分の移動の壁時計分解.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 Q12 Q13 Q14 Q15 Q16 Q17 G1 G2 G3 G4 G5 G6 V1 V15 V16 V21 V22 V23 V26 V27 V30 V31 V33 V34 V55 V56 V94 F1 F2 F3 F4 F5 F7 F8 N1 N2 N3 N4 N5 N6 N7 N8 N9 N10 N11 N12 N13 N14 N15 H1 H2 H3 H4 H5 H6 H7 H8 R2 -->

跑前登记 `research/prompts/e185-preregistration.md`（四段：第一段 `wall-clock`、第二段 `stage-split`、第三段 `gpu-submission`、第四段 `foreground-fsync`，第四段附带、够判后不跑）。回答问题单 `research/prompts/m4-gpugc-measure-forks.md` 的四行：一批搬迁的挂钟受盘还是 CPU 限、GPU 上算 GC 的哪几段（甲 只算校验和核对 ／ 乙 加区域扫描里的解头与映射反查 ／ 丙 再加选段打分）、攒批形态（一批一交 ／ 几批攒起来一次交 ／ 放宽 R）、定时兜底间隔与写带宽上限的默认值从什么量取。被测的是 `crates/` 今天的 GC 路径 `MountedSession::run_compaction`；背景是欠账 C412（整理的挂钟代价没量过），被测条款是 D26（后台整理与放置回收） 已定项 13、已定项 1 与 D24（后台重活能不能卸给 GPU） 已定项 3 的判据形式。

正文是第 2 次重跑的结果：重跑登记 `research/prompts/e185-r2-prereg.md`（sha256 前缀 `7d74c089`，写于 2026-10-23，装置改之前、这一趟的产物一份都还没跑）。起因：第 1 次重跑里 f = 0.25 两点几何没造成，问题单第 1、2 行按字面不够判；第一段里还有一段连成一片的慢段（同一份活 CPU 涨到 1.13–2.5 倍，作废后当场重试的那条 2.316 倍被收下，登记第四节第 5 行，第 161 行）。登记改了三样：负载（改法甲：只换造池第 2 步覆盖写的次序，让占比 0.25 的几何造得出）、装置的读数（每条意图前后的校准、线程与机器状态读数，造池之后逐段打成分）、作废与重试的做法（校准比接进「机器被别的活挤」，作废后先等机器回到常态再重试，第十一节修订 R2-1 至 R2-4，原判据原样保留）；另有执行员在单测之前、产物之前写的修订一（R2-5 至 R2-9，只收严或补量，登记第 415–425 行）。Q1–Q17 每一行的算法、门槛、判定照原登记（含第十二节 12.1–12.11）一个字不改。问题单第 3、4 行第 1 次重跑已够判，这一趟不再判，沿用第 1 次重跑的产物（岔路表写明哪几份）；第三、四段这一趟不跑。第 1 次重跑的数与七·6 前后的比法（重跑登记 `research/prompts/e185-r1-prereg.md`）已挪进变更史，正文只写这一趟。

装置是入库装置 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs`（一个二进制、五个子命令：`wall-clock`、`stage-split`、`gpu-submission`、`foreground-fsync` 与这一趟新加的 `geometry-check`，只读驱动 `singlefs-core` 的公开入口，不改 `crates/` 里别的文件）。决定工作量、几何与判据门槛的常量都是本地常量，值抄自登记第五节常量表，每个子命令开跑先与 `crates/` 同名常量回比（停机条款第 1 条）；这一趟新加一个本地常量 `LOCAL_ROOT_RING_ENTRIES = 24`，回比 `singlefs_format::ROOT_RING_REGIONS × ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`（装置第 131 行、第 178–180 行），只用来打机理核对那一栏。

**现状**：

- 装置只有一版：sha256 `0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d`，13187 行，`grep -c '#\[test\]'` 对它 = 118（改前基线是第 1 次重跑第三段用的 `eadb929d…`，10226 行、87 条）。三趟产物的 `name=apparatus` 行里，编进二进制的源码哈希与起跑脚本设给装置的登记哈希都是 `0bd22743…`（停机条款第 7 条与修订 R2-8 没触发），单点核两份与第二段同形，第一段如下（第一段第 2 行）：

```text
E7RESULT name=apparatus source_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d registered_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d
```

- 单测与变异（登记第十二节第 427–431 行，量的是主 agent 在一次性编译目录里跑的，这一页没重跑）：单测 `118 passed; 0 failed`，clippy 退 0；`crates/mutations.tsv` 里 `E185 V` 起头的行现在 94 条（`grep -c 'E185 V' crates/mutations.tsv` 输出 94）：V1–V15 照原登记第九节，V16–V55 是上一轮执行员补的，V56–V94 共 39 行是这一趟补的（V56–V76 登记第九节，V77–V94 修订一 R2-9），这 39 行的轻型变异全部抓到（登记第 431 行：计数「抓到 314 条、没红 0 条、无效 0 条」，39 行都在抓到里）；整表的三个数待提交时门禁 checker-tier-crates-mutation-replay。
- 这一趟的产物（都在 `research/results/`，行数用 `grep -c ''` 数，末行都是 `name=done`）：
  - 单点核：`research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out`（125 行，sha256 `c47db8bf3a7841d65aa209c3a3fb4ba941e9c30ec68672cde1b0fc480c206e00`，末行 `E7RESULT name=done emitted=125`），对照 `research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out`（125 行，sha256 `0add58dbc58f0948d0b115d6575173494aca801550508c2d5d732ad38885a5c2`，末行 `E7RESULT name=done emitted=125`），输入清单 `research/results/e185-r2-inputs-geometry-check-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-geometry-check-2026-10-23.txt`。
  - 第一段：`research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out`（2103 行，sha256 `18191ec3a8afbb27405987f99d3e36e5ae2538d3c9ab4f2717738495e767096b`，末行 `E7RESULT name=done emitted=2103`），输入清单 `research/results/e185-r2-inputs-wall-clock-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-wall-clock-2026-10-23.txt`。
  - 第二段：`research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out`（1803 行，sha256 `f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a`，末行 `E7RESULT name=done emitted=1803`），输入清单 `research/results/e185-r2-inputs-stage-split-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-stage-split-2026-10-23.txt`。
  - 沿用的（第 3、4 行，不重算）：第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（sha256 `4041a1cac83e7d62ef8bbe1580468aa6b4e43d69d67e8e9e044dd885ab063408`）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`（sha256 `978ae7b5fe60ab8f5a87338b982e8989c4c0d0ba8f19994bfc8e1c39b3d437fd`），输入清单 `research/results/e185-r1-inputs-gpu-submission-2026-10-22.txt`、`research/results/e185-r1-inputs-2026-10-22.txt`。
  - 存档不引（第 1 次重跑与 2026-10-20 那一趟的产物，数在变更史）：`research/results/e185-gc-batch-wall-clock-split-stage-split-r1-2026-10-22.out`、`research/results/e185-r1-comparison-2026-10-22.out`（比法判决行 `output_void=false`，即那一份没作废）、`research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out`、`research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out`。
- 跑在哪：单点核、第一段、第二段在第二台先后跑，不同时跑，真盘臂的镜像放在同一块盘的 ext4 上（输入清单第 22–26 行：型号 CT2000T500SSD8，逻辑块 512，文件系统 ext4；第一段配置行 `image_directory_filesystem=ext4 image_directory_is_cleared_at_boot=false image_device=64512`），不带 `gpu` 特性编（三份输入清单里 `grep -c lookup-work-counters` 都输出 0），外层内存上限 46G、`E185_MEMORY_CAP=40G`、装置线程 31（第一段配置行 `worker_threads=31`）。三份输入清单第 1–11 行（装置、`transaction.rs`、`mounted_session.rs`、`region_scan.rs`、`compaction_plan.rs`、`unit.rs`、`checksum.rs`、`block_device.rs`、`allocator.rs`、`singlefs-format/src/lib.rs`、`Cargo.lock` 的 sha256）逐行相同（`diff` 核过），第 12 行是各自拷树后的提交号。每趟整条命令经 `scripts/crash/gpu-quiesce-run.sh` 起，清场记录原样：

```text
GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0
GPU_QUIESCE step=command result=exit-0
GPU_QUIESCE step=restore result=not-restored holders=1
GPU_QUIESCE step=exit code=0
```

  单点核那一份 `step=quiesce result=stopped-and-read-back holders=0`、`step=restore result=not-restored holders=0`；第二段那一份第一行是 `step=quiesce result=joined-and-read-back holders=1`（别的会话已登记同用卡，加入而不是新停）；三份都是 `step=command result=exit-0`、`step=exit code=0`。清场回读没过会退 78、命令不起，三趟都没有。
- 判决行（单点核两份、第一段、第二段，整行抄）：

```text
E7RESULT name=verdict segment=geometry_check constant_mismatches=0 overwrite_order=highest_full_segment_last points=2 geometry_formed_points=2
E7RESULT name=verdict segment=geometry_check constant_mismatches=0 overwrite_order=slot_order points=2 geometry_formed_points=1
E7RESULT name=verdict segment=wall_clock constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none discrimination_failures=0 clause_wall_clock_not_rising_with_the_batch_triggered=0 clause_two_cpu_paths_disagree_points=0 gc_to_sequential_write_bandwidth_maximum_ratio=0.015977 gc_to_sequential_write_bandwidth_judgement=single_run_enough stored_intents=140
E7RESULT name=verdict segment=stage_split constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none
```

  四行里取值 `false` 的字段没有，也没有 `not_run`，名字表示违例、不匹配、失败的计数（`constant_mismatches`、`constant_or_anchor_mismatches`、`stop_clause_findings`、`positive_control_failures`、`discrimination_failures`、两条失败条款的触发数）都是 0；大于 0 的整数只有 `stored_intents=140` 与 `points`/`geometry_formed_points`：旧造法对照那一份 `geometry_formed_points=1`（两点里只造成了占比 0.75 那一点）是对照的预期，它重现了第 1 次重跑的「没造成」，改法甲那一份是 `geometry_formed_points=2`。`gc_to_sequential_write_bandwidth_maximum_ratio=0.015977` 是 Q16 的值不是失败计数。
- 起跑脚本：三趟当时跑的是仓根 `e185-r2-tools/` 下的 `run-geometry-check.sh`、`run-wall-clock.sh`、`run-stage-split.sh` 与它们共用的 `common.sh`；之后照用户要求，合成了 `scripts/experiment-apparatus/e185-r2-segment-run.sh`（登记第五节「跑法」已改写，第十二节「起法」一行写明），三趟跑的是合并之前那几份，每段做的事相同（写输入清单 → 在拷过去的树里编一次装置 → 跑这一段 → 产物写进 `research/results/`）。
- `scripts/experiment-apparatus/replay.sh` 还没有 E185（GC一批搬迁的挂钟拆分） 的登记行（`grep -c E185 scripts/experiment-apparatus/replay.sh` 输出 0；照 E184（常规层与大负载层的代价计数） 的先例只登记确定性的一片），三份测量产物都带挂钟字段。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:51-127`（整段抄，未转述）**

````markdown
### 几何：f = 0.25 两点造成了（改法甲）

改法甲只换造池第 2 步覆盖写的次序（「做了什么」第 2 条），占比 0.75 的次序不变。单点核（登记第五节「单点核」，只核 P = 1024）有两份：改法甲 `research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out`，对照 `research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out`（`--overwrite-order slot-order`，第 1 次重跑那种造法）。登记三格的读数：

| 格 | 读哪 | 读到的 | 判 |
|---|---|---|---|
| 甲造成 | 改法甲那一份第 64 行 `name=geometry`、第 55 行 `name=segment_composition` | `formed=true`；那一段 `live_slots=16 slots_of_this_workload=16 other_slots=0` | 成立，停机条款第 8 条（第十条 H1）不中 |
| 占比 0.75 没被同一棵树里的改动带歪 | 两份第 123 行 `name=pool fraction=0.75 pool_data_units=1024` 的 `digests=` 整值 | 相同（下面的 `diff` 退 0） | 成立，停机条款第 9 条不中 |
| 对照：旧造法在今天的树上重现「没造成」 | 对照那一份第 3 行 `name=geometry_attempt`、第 64 行 `name=geometry` | `geometry_formed_segments=0`、`choice=none … formed=false` | 与登记推的 0 相同 |

```text
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=forced_fallback fallback_region_start_slot=53824 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
E7RESULT name=segment_composition fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 region_start_slot=53824 live_slots=16 slots_of_this_workload=16 other_slots=0 formed=true
E7RESULT name=geometry_attempt fraction=0.25 pool_data_units=1024 workload_changes=0 construction_seconds=20.697 overwritten_units=776 filler_units=0 floor_raises_pushed_by_the_session=0 release_rounds=4 every_overwritten_slot_released=true mapped_units=1056 geometry_formed_segments=1 greedy_region_start_slot=51008 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false
E7RESULT name=geometry_attempt fraction=0.25 pool_data_units=1024 workload_changes=0 construction_seconds=20.776 overwritten_units=776 filler_units=0 floor_raises_pushed_by_the_session=0 release_rounds=4 every_overwritten_slot_released=true mapped_units=1058 geometry_formed_segments=0 greedy_region_start_slot=50368 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=none fallback_region_start_slot=none formed=false reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
```

改法甲下 greedy 自己选中的那一段（第 3 行 `greedy_region_start_slot=51008 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false`）仍是装着别的单元的段，f = 0.25 的点跟占比 0.75 一样走登记好的后备（`choice=forced_fallback`，只供测试的强制选段点名那一个 f 段，第 64 行 `reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change`）。所以第一、二段量的是被强制选定的那一段，不是 greedy 自己选中的段。第一、二段八个 `name=geometry` 行（四个点 × 两段）全是 `choice=forced_fallback … formed=true`：

```text
$ grep -h 'name=geometry ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out | grep -c 'choice=forced_fallback .* formed=true'
8
```

```text
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=forced_fallback fallback_region_start_slot=53824 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
E7RESULT name=geometry fraction=0.25 pool_data_units=8192 choice=forced_fallback fallback_region_start_slot=78464 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
```

占比 0.75 的池与第 1 次重跑是不是逐字节相同，登记第五节两道，各报各的：

```text
$ diff <(awk 'NR==237||NR==515' research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
1,2c1,2
< 1024 423aa5bc004b3247287ae29cf0bd80de0508b33ec2724f654640e0475fdbc216,ec8a4ecb79383a2f66c6828c4ce64bc0234a708f10c09b7566a5d58b79200ca3
< 8192 e0910aa57c7de2b42dfd1026123435f3e84244105ef0067fb6b0aa44edc3cbb6,d177e4fc925f9c701acdd6e9a879bfff3017615db4c034e1d0c9e55ea6902e26
---
> 1024 a77b38d298bf0f30358b2011867845734bea60928bcb4b1e98b2ab13563d30c3,c43dcaf3fa8b68f1cf7b1d498569f1fa96a6220d07b428d8b5ceecb8b5ebc5d0
> 8192 7d26930931d792bf5859dfc1ceaa83a87ae4a2328ffce19af576986880642c3e,72de4e03f439421f3e56933997b33246b24351994fb457965e6d157778679ede
exit=1
$ diff <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
exit=0
$ diff <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
exit=0
```

第 1 道（与第 1 次重跑第一段第 237、515 行比）不同，第 2 道（同一棵树里：单点核两份、再加第一段第 912 行）相同。照登记第五节判法记：占比 0.75 的池与第 1 次重跑不同，同一棵树里改动没有带歪它；差在两棵树之间（核心层五份源文件与 `Cargo.lock` 的 sha256 与第 1 次重跑输入清单不同，登记第三节第 134 行）。不停，占比 0.75 的点照量，这一趟的数不与第 1 次重跑的数混用，所以岔路表与各量的判定只引这一趟。

机理核对（登记第八节，第十条 H2；只报，不是锚点，对不上不作废）：覆盖写副本落进这次造池打出的旧洞的份数，装置逐次数出来的与登记推的 `overwritten_units − 24` 并列，造成段数两条路数并列，下面两条命令从三份产物里数：

```text
$ grep -h 'name=overwrite_landing' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | grep 'status=counted' | sed -E 's/.*fraction=([0-9.]+) pool_data_units=([0-9]+) .*copies_landing_in_holes_of_this_construction=([0-9]+) .*predicted_copies_landing_in_holes=([0-9]+).*/\1 \2 \3 \4/' | sort | uniq -c
      3 0.25 1024 752 752
      2 0.25 8192 6128 6128
      3 0.75 1024 256 256
      2 0.75 8192 2048 2048
$ grep -h 'name=segment_composition_summary' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out | sed -E 's/.*fraction=([0-9.]+) pool_data_units=([0-9]+) .* formed_segments=([0-9]+) geo.*/\1 \2 \3/' | sort | uniq -c
      2 0.25 1024 1
      2 0.25 8192 1
      2 0.75 1024 3
      2 0.75 8192 3
```

三样都对上：每个点落进旧洞的副本数 = 登记的预测（752、6128、256、2048）；改法甲下 f = 0.25 的 `formed_segments` 是 1（登记预测 1，与 P 无关），f = 0.75 是 3（预测 3）；对照那一份 f = 0.25 的 `geometry_formed_segments=0`（预测 0，上面对照格）。`name=segment_composition_summary` 共 12 行（第一、二段各 4、单点核两份各 2），`counts_agree=true` 12 行、`counts_agree=false` 0 行（`grep -h 'name=segment_composition_summary' <四份产物> | grep -c 'counts_agree=true'` 输出 12，`grep -c 'counts_agree=false'` 输出 0）。记「调查员推的机理在这几个点上对上了」；这一句只说这四个点与单点核那两点，不外推到别的 f、P。

```text
E7RESULT name=segment_composition_summary fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 segments=58 formed_segments=1 geometry_formed_segments_of_the_attempt=1 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=776 copies_landing_in_holes_of_this_construction=752 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=752 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.25 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 segments=472 formed_segments=1 geometry_formed_segments_of_the_attempt=1 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.25 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=6152 copies_landing_in_holes_of_this_construction=6128 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=6128 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.75 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 segments=53 formed_segments=3 geometry_formed_segments_of_the_attempt=3 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.75 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=280 copies_landing_in_holes_of_this_construction=256 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=256 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.75 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 segments=418 formed_segments=3 geometry_formed_segments_of_the_attempt=3 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.75 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=2072 copies_landing_in_holes_of_this_construction=2048 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=2048 root_ring_entries=24
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:128-138`（整段抄，未转述）**

````markdown
### 问题单第 1 行够判的集合 S

登记第一节「第 1 行够判」的读法（设计员写的，主 agent 认下，调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 381 行）：S = 第一段里（f、P、K）三者的格，那一点 `name=geometry` 写 `formed=true`，且那一格 `name=q3` 写 `judgement=compatible`；两个 f 在 S 里各有至少两个 K 就够判；Q1、Q2 在 S 的格上报的判定就是第 1 行的读数。

```text
$ grep -c 'name=q3 .*judgement=compatible row_one_judgeable_here=true' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
12
```

第一段 `name=geometry` 四行全 `formed=true`（上面），`name=q3` 12 格全是 `judgement=compatible row_one_judgeable_here=true`（第 1879、1883、1887、1891、1895、1899、1903、1907、1911、1915、1919、1923 行），所以 S = 全部 12 格：f = 0.25 与 f = 0.75 各 6 格（P = 1024、8192 × K = 4、8、16）。第二段 12 格同一批点。F5 没触发（判决行 `clause_two_cpu_paths_disagree_points=0`），F4 没有（每格 5 轮判定一致）。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:139-191`（整段抄，未转述）**

````markdown
### 各量的判定（第 2 次重跑）

照原登记第六节各报各的判定，5 轮判定不全相同写「不稳定」。产物：第一段 = `research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out`（下文行号叫「第一段第 N 行」），第二段 = `research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out`（「第二段第 N 行」）。区间是对那一组行取最小与最大，两端都是产物里的原值。第一段 Q1、Q2 合计 24 格的判定数：

```text
$ W=research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
$ grep -E 'name=q[12] ' $W | sed -E 's/.* judgement=([a-z_]+)$/\1/' | sort | uniq -c
     12 both
     12 cpu
```

第一段 12 个（f、P、K）格，每格在 `name=q1`、`name=q2`、`name=q3`、`name=q14` 四行里各一行，四行连着排（第 1877–1880 行起，每格 4 行，到第 1924 行）。

| 量 | 读哪 | 这一趟的数 | 判定 |
|---|---|---|---|
| Q1 ρ_真盘 | 第一段 `name=q1` 12 行（第 1877、1881、1885、1889、1893、1897、1901、1905、1909、1913、1917、1921 行） | `rho_median`：P = 1024 六格 1.221978–1.493617，P = 8192 六格 8.441898–9.848870 | P = 1024 六格 `both`，P = 8192 六格 `cpu`；每格 5 轮判定全相同（`judgement_per_round`），没有 `disk`、没有 `unstable`；f = 0.25 的六格与 f = 0.75 的六格同判。随 P 翻面（第 1926 行六对全翻），不随 f（第 1925 行 `flips=0`）、不随 K（第 1927 行 `flips=0`） |
| Q2 挂钟比 | 第一段 `name=q2` 12 行（第 1878、1882、…、1922 行，每隔 4 行） | `intent_wall_ratio_median`：P = 1024 六格 1.723812–1.906832，P = 8192 六格 1.105487–1.118963 | 与 Q1 同判：P = 1024 六格 `both`，P = 8192 六格 `cpu`，每格 5 轮相同；随 P 翻面（第 1929 行 `flips=6`），不随 f（第 1928 行）、不随 K（第 1930 行）`flips=0` |
| Q3 两路对账 | 第一段 `name=q3` 12 行（第 1879、1883、…、1923 行） | 每轮 `reconciliation_per_round` 最大 0.103088（f = 0.75、P = 1024、K = 8 第 5 轮，第 1907 行）；f = 0.25 的格最大 0.055696（P = 1024、K = 16 第 2 轮，第 1887 行） | 12 格全 `compatible`、`row_one_judgeable_here=true`；F5 没触发（判决行 `clause_two_cpu_paths_disagree_points=0`）。离 20% 的门槛最近的一轮是 0.103088 |
| Q4 批大小翻倍 | 第一段 `name=q4` 12 对（第 1933–1944 行） | 每对 5 轮中位 1.015102–1.074115 | 内存臂 6 对、真盘臂 6 对全 `rises`，12 对的 `copy_bytes_double=true`、`clause_wall_clock_not_rising_with_the_batch_triggered=false`；判决行 F1 触发数 0。判据只要 > 1（且每轮 > 1），这一趟涨得不多：批大小翻倍，第一批挂钟涨 1.5%–7.4% |
| Q14 GC 单跑写带宽 | 第一段 `name=q14` 12 行 | `write_bytes_per_second_median` 7229989.048161–50509463.337147 | 不单判 |
| Q15 设备顺序写带宽 | 第一段第 1945 行 | `E7RESULT name=q15 direct_with_fdatasync_bytes_per_second_median=3161349388.1 page_cache_without_sync_bytes_per_second_median=13066869439.3 positive_control_row_four_holds=true` | 不单判 |
| Q16 带宽比 | 第一段第 1958 行 | `maximum_ratio=0.015977`（f = 0.75、P = 1024、K = 16，第 1954 行 `q16_point`） | 「单跑够」；三个旋钮都不翻（第 1959–1961 行 `flips=0`） |
| Q17 前台 fsync 延迟比 | 第四段 | 没跑 | 够判后未跑（第 4 行，Q16 ≤ 10%，原登记第十一节够判停机） |
| Q5 甲段份额 | 第二段 `name=stage` 批行（`kind` 是 `batch` 或 `final_batch`）的 `crc_share`；`name=stage_judgement` 30 行 | f = 0.75 最大 0.020433（第 947 行），f = 0.25 最大 0.006863（第 116 行） | 30 行 `crc_share_judgement=not_worth`，每行 5 轮都是 |
| Q6 乙段增量份额 | 同上的 `scan_increment_share` | f = 0.75 最大 0.006521（第 1065 行），f = 0.25 最大 0.002601（第 216 行） | 30 行 `scan_increment_share_judgement=not_worth`，每行 5 轮都是 |
| Q7 丙段增量份额 | 第二段 `name=choose_judgement` 12 行（第 248–250、869–871、1148–1150、1793–1795 行）的 `choose_share_per_batch_per_round` | f = 0.25 最大 0.028846（第 250 行），f = 0.75 最大 0.014279（第 1150 行）；签发那一次的原值 `choose_share_of_the_issue` 最大 f = 0.25 是 0.03894、f = 0.75 是 0.038388，也都低于 10% | 12 行 `choose_share_judgement=not_worth`，每行 5 轮都是。`name=stage_judgement` 里的丙段那一格 30 行都是 `not_run`，是登记第六节写明的（丙段改读 `name=choose_judgement`），不是漏判 |
| Q8 单线程吞吐 | 第二段批行 `crc_bytes_per_second`、`scan_increment_bytes_per_second`；签发行 `choose_record_bytes_per_second` | 甲 2875331797.740485–3189332554.68769 B/s；乙增量 1046057094.516405–12147544022.242819 B/s；丙 112157777.007656–892197239.242721 记录字节/秒 | 不单判，喂 Q9 |
| 饱和吞吐（Q9 的 CPU 一侧） | 第二段 `name=saturation` 108 行，`distinct_threads_holds=true` 108 行 | 16 线程：甲 48832371460.2–49032300970.9，乙 14532706506.7–37784969513.8，丙 1641866090.6–13741047928.6 B/s | 不单判 |

第二段几条数的命令与输出（`S` 是第二段产物路径）：

```text
$ S=research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
$ for f in 0.25 0.75; do for k in crc_share scan_increment_share; do echo -n "batch f=$f $k max: "; grep 'name=stage ' $S | grep -E "kind=(batch|final_batch)" | grep "fraction=$f " | grep -o " $k=[0-9.]*" | sed 's/.*=//' | sort -g | tail -1; done; done
batch f=0.25 crc_share max: 0.006863
batch f=0.25 scan_increment_share max: 0.002601
batch f=0.75 crc_share max: 0.020433
batch f=0.75 scan_increment_share max: 0.006521
$ for f in 0.25 0.75; do echo -n "choose_judgement f=$f max per_round: "; grep 'name=choose_judgement ' $S | grep "fraction=$f " | grep -o 'choose_share_per_batch_per_round=[^ ]*' | sed 's/.*=//' | tr ',' '\n' | sort -g | tail -1; done
choose_judgement f=0.25 max per_round: 0.028846
choose_judgement f=0.75 max per_round: 0.014279
$ grep 'name=stage_judgement ' $S | sed -E 's/.* (crc_share_judgement=[a-z_]+) .* (scan_increment_share_judgement=[a-z_]+) .* (choose_share_judgement=[a-z_]+)$/\1 \2 \3/' | sort | uniq -c
     30 crc_share_judgement=not_worth scan_increment_share_judgement=not_worth choose_share_judgement=not_run
$ grep 'name=choose_judgement ' $S | sed -E 's/.* (choose_share_judgement=[a-z_]+)$/\1/' | sort | uniq -c
     12 choose_share_judgement=not_worth
```

最大的那一行的第二段第 250 行（丙段）整行抄：

```text
E7RESULT name=choose_judgement fraction=0.25 pool_data_units=1024 batch_units=16 rounds=5 choose_share_per_batch_per_round=0.028676,0.028423,0.028614,0.028705,0.028846 choose_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth choose_share_judgement=not_worth
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:192-204`（整段抄，未转述）**

```markdown
### 沿用第 1 次重跑的量（问题单第 3、4 行；这一趟第三、四段不跑）

数取自第 1 次重跑第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（下文「r1 第三段第 N 行」）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`。两个范围都含 f = 0.25 的两点，那两点在第 1 次重跑里几何没造成（量的是 greedy 选中的段），这一趟没有重算。

| 量 | 读哪 | 第 1 次重跑的数 | 判定 |
|---|---|---|---|
| Q9 16 线程对 GPU | r1 第三段 `name=q9` 72 行（两张卡 × 三段 × 12 格） | `ratio` 最大 0.007746（卡 0、乙增量、f = 0.75、P = 8192、K = 8） | 72 行 `judgement=gpu_loses`，单线程版 `judgement_one_thread=gpu_loses`，`clause_sixteen_thread_form_misfits_the_gc_triggered=false`（F2 没触发）。这一趟在 f = 0.25 造成了的点上没有 Q9。Q9 算不算问题单第 2 行的翻面量，交主 agent，并进七·2 设计轮定（登记第一节、调度记录第 393 行） |
| Q10 B_批 | r1 第三段 `name=q13` 的 `batch_bytes`（读第 1 次重跑第一段 `name=call`） | f = 0.75：131072–786432 字节；f = 0.25：16384 字节（几何没造成的点） | 不单判 |
| Q11 a、Q12 b | r1 第三段第 54、136 行 | 卡 0 `E7RESULT name=q11_q12 index=0 card_valid=true intercept_seconds=0.004492 slope_seconds_per_byte=4.117289e-10 one_range_median_seconds=0.005244 largest_two_points_slope_seconds_per_byte=4.172332e-10`；卡 1 `E7RESULT name=q11_q12 index=1 card_valid=true intercept_seconds=0.010300 slope_seconds_per_byte=1.266004e-9 one_range_median_seconds=0.005393 largest_two_points_slope_seconds_per_byte=1.191920e-9` | 不单判 |
| Q13 φ 与 n* | r1 第三段 `name=q13` 56 行 | 卡 0 `fixed_overhead_share` 0.932762–0.998500、`batches_to_merge` 125–5993；卡 1 0.911860–0.997990、94–4470 | 56 行 `judgement=merge_batches`（φ ≥ 50%，「要攒批」），两张卡同判（F8 没触发）。判别力第 201 行 `E7RESULT name=discrimination quantity=Q13 lower_value=0.911860 higher_value=0.998500 moved_threshold=0.955180 lower_judgement=one_batch_per_submission higher_judgement=merging_optional holds=true` |

推的（没量）：f = 0.25 造成之后，它的 B_批 ≤ 签发那一次的 262144 字节（原登记第七节乙类第 356 行），比 f = 0.75 的小；φ = a ÷ (a + b·B_批) 随 B_批变小只会变大，翻面要 φ 落到 50% 以下，第 1 次重跑 φ 最小的那一格是 0.911860。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:205-220`（整段抄，未转述）**

````markdown
### 阳性对照、自检与失败条款

- 第一段第 1 行阳性对照四格（第 1963–1966 行）：盘臂内存盘、真盘两格 `holds=true`，CPU 臂内存盘、真盘两格 `cpu_clause_holds=true write_wait_clause_holds=true`；CPU 臂内存盘那一格原判据那一句 `wait_clause_under_the_original_criterion_holds=false`，照 12.9 只报不判。整行抄：

```text
E7RESULT name=positive_control row=1 arm_of_the_question=disk device=memory injection=sleep_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=1795862.0 wait_increment_nanoseconds=362017597.0 write_wait_increment_nanoseconds=362030328.0 cpu_change=0.036631 wait_change=-3020.110262 paired_rounds=5 cpu_increment_paired_median_nanoseconds=1701172.000000 write_wait_increment_paired_median_nanoseconds=362030518.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=cpu device=memory injection=spin_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=352496564.0 wait_increment_nanoseconds=68029.0 write_wait_increment_nanoseconds=67282.0 cpu_change=7.189988 wait_change=-0.567528 cpu_increment_over_injected=1.001411 cpu_clause_holds=true write_wait_increment_over_injected=0.000191 write_wait_clause_holds=true wait_clause_under_the_original_criterion_holds=false original_criterion_is_reported_only=true paired_rounds=5 cpu_increment_paired_median_nanoseconds=352445233.000000 write_wait_increment_paired_median_nanoseconds=67472.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=disk device=real_disk injection=sleep_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=1782904.0 wait_increment_nanoseconds=361903509.0 write_wait_increment_nanoseconds=362318961.0 cpu_change=0.033885 wait_change=7.108353 paired_rounds=5 cpu_increment_paired_median_nanoseconds=1272218.000000 write_wait_increment_paired_median_nanoseconds=362373072.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=cpu device=real_disk injection=spin_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=353173767.0 wait_increment_nanoseconds=4108344.0 write_wait_increment_nanoseconds=1711955.0 cpu_change=6.712162 wait_change=0.080694 cpu_increment_over_injected=1.003335 cpu_clause_holds=true write_wait_increment_over_injected=0.004864 write_wait_clause_holds=true wait_clause_under_the_original_criterion_holds=true original_criterion_is_reported_only=true paired_rounds=5 cpu_increment_paired_median_nanoseconds=352961316.000000 write_wait_increment_paired_median_nanoseconds=1684671.000000 holds=true
```

- 第二段第 2 行阳性对照四格（第 123–126 行）`holds=true`；两段 `name=clock_resolution_check` 都 `holds=true`（两段第 3 行）；第一段 `name=discrimination` 三行（Q1、Q2、Q16，第 1931、1932、1962 行）`holds=true`；两段 `name=anchor` 各 34 行全 `holds=true`（`grep 'name=anchor' <产物> | grep -c 'holds=true'` 两份都输出 34，`grep -vc` 输出 0）。
- 作废：第一段 `name=intent` 140 行全是 `valid=true invalid_reason=none`，第二段 60 行同；两段都没有 `name=round_invalid` 行（`grep -c 'name=round_invalid'` 输出 0），`stored_intents=140`。第 1 次重跑第一段作废过 1 轮，这一趟一轮都没作废、没重试。
- 失败条款：F1 没触发（Q4 12 对全 `rises`）；F2 这一趟不适用（Q9 沿用）；F3 打中：Q1 在 P = 1024 的六格落在 (0.5, 2)（1.221978–1.493617）、Q2 在 P = 1024 的六格落在 (1.5, 3)（1.723812–1.906832），照实记「两边都占」，是正当结果；F4 没有；F5 没触发；F7 打中：第二段 Q5、Q6、Q7 都 < 10%（最大分别 0.020433、0.006521、0.028846），照原登记记「三段都不到一批 CPU 的 10%，CPU 的大头在其余（读这一版、发布）」，交主 agent 判第 2 行的候选是否要补一个「发布那一半」的臂；F6、F8 沿用第 1 次重跑（F6：Q13 没有落在 (10%, 50%) 的格；F8：两张卡同判），这一趟没重算。
- 这一趟新加的失败条款（登记第十条）：H1（甲造不出）没触发；H2（机理对不上）没触发，见「几何」一节；H3、H4 以 F5 打中为前提，没打中，不适用；H5（轮数不足）没有，`name=round_invalid` 0 行；H6（第 1 行仍不够判）没有，S 里 f = 0.25 与 f = 0.75 各 6 格；H7（第 4 行单跑那一半翻面）没触发，`name=q16` 是 `single_run_enough`；H8（校准基线漂移）没触发，两段 `drifted=false`。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:221-240`（整段抄，未转述）**

````markdown
### 校准与机器状态读数（登记第六节新加的量，N1–N15）

校准比（N1）是测量线程每条意图前后对一块 16 MiB 缓冲连做 3 遍 CRC-32C 取中位，除以段开头 21 遍的中位；> 1.10 作废（修订 R2-1）；作废后先等机器回到常态再重试（R2-2）。其余读数只报不判。

| 读数 | 第一段 | 第二段 |
|---|---|---|
| 段开头 / 段末基线中位（N2） | 5243296.0 / 5227887.0 纳秒，末 ÷ 头 = 0.997061，`drifted=false`（第 5、2099 行） | 5224477.0 / 5222377.0 纳秒，0.999598，`drifted=false`（第 5、1799 行） |
| 校准自己的阳性对照（R2-3，加倍的活） | `ratio=2.003004 crc_holds=true holds=true`（第 6 行） | `ratio=2.003983 crc_holds=true holds=true`（第 6 行） |
| 前后两个校准比里大的那个的最大值（N1） | 1.015958；> 1.10 的意图 0 条，> 1.05 的 0 条，> 1.20 的 0 条；等常态 0 次（第 2100 行） | 1.077323；> 1.10 的 0 条，> 1.05 的 3 条，> 1.20 的 0 条；等常态 0 次（第 1800 行） |
| 追指针校准比（N15）的范围 | 1.226963–1.534854，每条意图都高于 1 | 1.279786–1.514878，同 |
| 整机其余部分占的忙比例（N10 `others_share`）最大 | 0.002502（第 4 行，段开头） | 0.001875（第 932 行） |
| 线程读数（N3–N9）最大 | 排队等 CPU 7508078 纳秒，非自愿切换 151，`major_faults` 全 0，CPU 号 4 个 | 排队等 CPU 914484 纳秒，非自愿切换 29，CPU 号 2 个 |

```text
E7RESULT name=calibration_trajectory segment=wall_clock intents=140 intents_with_a_readable_ratio=140 larger_ratio_peak=1.015958 intents_above_the_voiding_edge=0 voiding_edge=1.10 intents_above_the_lower_reported_edge=0 lower_reported_edge=1.05 intents_above_the_upper_reported_edge=0 upper_reported_edge=1.20 larger_ratio_final=0.998801 wait_longest_seconds=0 waits=0 wait_total_seconds=0
E7RESULT name=calibration_trajectory segment=stage_split intents=60 intents_with_a_readable_ratio=60 larger_ratio_peak=1.077323 intents_above_the_voiding_edge=0 voiding_edge=1.10 intents_above_the_lower_reported_edge=3 lower_reported_edge=1.05 intents_above_the_upper_reported_edge=0 upper_reported_edge=1.20 larger_ratio_final=1.002871 wait_longest_seconds=0 waits=0 wait_total_seconds=0
```

追指针校准比（N15）全部高于 1（1.23–1.53）是装置报出来的，登记只要求报、不判，这一页没查原因；它没有用来作废任何意图。N14（照原作废规则的并列判定）两段各 12 行，`differs_from_the_registered=true` 0 行（`grep -c 'differs_from_the_registered=true'` 对两份产物都输出 0）：没有一条意图因校准比被作废，R2-1 在这一趟没有动任何一格的进入与否。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:241-274`（整段抄，未转述）**

````markdown
### 问题单逐行（岔路表）

问题单 `research/prompts/m4-gpugc-measure-forks.md`（四行的「状态」一格已由主 agent 改成「够判」，调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 394 行）；够判条件照问题单原文，判定照这一趟的产物；S 的读法见上一节。

| 行 | 已够判 / 还差什么 | 这一趟读得出的 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1 盘 ／ CPU | **已够判**。问题单够判条件：至少两种批大小、两种段内活槽占比，在内存设备与真盘上各量到一批的挂钟与按阶段拆开的 CPU 时间。f = 0.25 与 f = 0.75 的几何都造成了（第一段 `name=geometry` 四行 `formed=true`），S = 12 格（`grep -c 'name=q3 .*judgement=compatible row_one_judgeable_here=true'` 输出 12），两个 f 各 6 格、K = 4、8、16 都在，Q1、Q2 逐格 5 轮判定一致 | P = 1024 六格 Q1、Q2 都判 `both`（`rho_median` 1.221978–1.493617，`intent_wall_ratio_median` 1.723812–1.906832）；P = 8192 六格都判 `cpu`（8.441898–9.848870；1.105487–1.118963）。判定随 P 翻面（第一段第 1926、1929 行 `flips=6`），不随 f、不随 K 翻（第 1925、1927、1928、1930 行 `flips=0`）；按 f、P 报，不按 K 报。F3 打中（P = 1024 两边都占）。整行见下面第一块 | 登记里没有剩下没跑的量。能动它的只有 P 之外的点：装置只收 P = 1024、8192 两个池大小，两点之间与更大的池没量。推的（没量）：翻面轴是 P，池更大时 CPU 一侧只会更重 |
| 2 GPU 算哪几段 | **已够判**（第 1 行已够判）。登记第一节「第 2 行够判」：S 里每一格第二段 `name=stage_judgement` 的甲、乙判定与 `name=choose_judgement` 的丙判定都不是 `not_run`，且 `crc_bytes`、`records_read_per_choice` 大于 0：甲、乙 30 行批判定全 `not_worth`，丙 12 行全 `not_worth`，批行 `crc_bytes` 最小 131072，`records_read_per_choice` 最小 1068（命令见「各量的判定」一节） | 甲、乙、丙三段都 `not_worth`：甲份额最大 0.020433，乙增量最大 0.006521，丙每批最大 0.028846（签发那一次的原值最大 0.03894），都不到一批 CPU 的 10%。原登记 F7 打中：CPU 的大头在其余（读这一版、发布），交主 agent 判第 2 行的候选要不要补「发布那一半」的臂。Q9 沿用第 1 次重跑 72 行 `gpu_loses`，算不算翻面量交主 agent，并进七·2 设计轮定（Q9 是登记第一节写明的一格）。整行见下面第二块 | 登记里没有剩下没跑的量。Q9 在 f = 0.25 的点上没有（第三段不跑）；它不论算不算翻面量，都与 Q5–Q7 同向（第 1 次重跑 72 行全 `gpu_loses`，这一趟三段份额没有一格越过 10%） |
| 3 攒批形态 | **已够判**（沿用第 1 次重跑，这一趟第三段不跑）。依据：第 1 次重跑第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（sha256 `4041a1ca…`）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`（`978ae7b5…`）。第三段装置 `eadb929d…` 比重跑登记写死的 `b3a50d50…` 多 3 行（`quarantine_mark`），主 agent 已认（本次登记第一节派发提示第 5 条，登记第 19 行） | 同第 1 次重跑：Q10 f = 0.75 每批 131072–786432 字节；两张卡 a、b 量到；Q13 56 行全 `merge_batches`，φ 最小 0.911860，两张卡同判：一批一交（甲）出局，选择在乙、丙之间，乙丙之间七·1 只给 n* 与每批字节，R 的语义与批间重扫由七·2 设计轮判（原登记第五节第 3 行臂表） | 没有剩下的量。翻面要 φ 落到 50% 以下，φ 最小的那一格也在 0.91 以上；f = 0.25 造成之后 B_批更小，φ 只会更大（推的，没量） |
| 4 兜底间隔与写带宽上限 | **单跑那一半已够判**（沿用第 1 次重跑 `name=q16`，这一趟第一段并列报、不重判）；第四段与 Q17 够判后未跑 | 第 1 次重跑第一段第 786 行 `E7RESULT name=q16 maximum_ratio=0.011019 judgement=single_run_enough …`；这一趟第一段第 1958 行 `maximum_ratio=0.015977 judgement=single_run_enough`，H7 没触发 | Q17 只在 Q16 > 10% 时跑，这一趟不跑 |

第一块（第 1 行的读数，第一段整行）：

```text
E7RESULT name=q1 fraction=0.25 pool_data_units=1024 batch_units=8 geometry_formed=true rounds=5 rho_per_round=1.456034,1.438745,1.464884,1.404448,1.458415 rho_median=1.456034 judgement_per_round=both,both,both,both,both judgement=both
E7RESULT name=q2 fraction=0.25 pool_data_units=1024 batch_units=8 geometry_formed=true rounds=5 intent_wall_ratio_per_round=1.772596,1.749635,1.720118,1.741935,1.760248 intent_wall_ratio_median=1.749635 per_batch_median_ratio=1.749635 judgement_per_round=both,both,both,both,both judgement=both
E7RESULT name=q3 fraction=0.25 pool_data_units=1024 batch_units=8 rounds=5 reconciliation_per_round=0.050004,0.031328,0.021439,0.016639,0.043347 judgement_per_round=compatible,compatible,compatible,compatible,compatible judgement=compatible row_one_judgeable_here=true
E7RESULT name=q1 fraction=0.75 pool_data_units=8192 batch_units=8 geometry_formed=true rounds=5 rho_per_round=9.819379,9.125596,9.851385,8.752512,9.908914 rho_median=9.819379 judgement_per_round=cpu,cpu,cpu,cpu,cpu judgement=cpu
E7RESULT name=q2 fraction=0.75 pool_data_units=8192 batch_units=8 geometry_formed=true rounds=5 intent_wall_ratio_per_round=1.108970,1.117088,1.108237,1.122220,1.114470 intent_wall_ratio_median=1.114470 per_batch_median_ratio=1.120025,1.116781,1.108746 judgement_per_round=cpu,cpu,cpu,cpu,cpu judgement=cpu
E7RESULT name=q3 fraction=0.75 pool_data_units=8192 batch_units=8 rounds=5 reconciliation_per_round=0.006159,0.006354,0.005763,0.006425,0.012056 judgement_per_round=compatible,compatible,compatible,compatible,compatible judgement=compatible row_one_judgeable_here=true
E7RESULT name=sensitivity quantity=Q1 knob=fraction pairs=6 flips=0 statement=does_not_flip_at_these_points flipped_pairs=none
E7RESULT name=sensitivity quantity=Q1 knob=pool_data_units pairs=6 flips=6 statement=flips_with_this_knob flipped_pairs=0.25:1024:4:both->0.25:8192:4:cpu,0.25:1024:8:both->0.25:8192:8:cpu,0.25:1024:16:both->0.25:8192:16:cpu,0.75:1024:4:both->0.75:8192:4:cpu,0.75:1024:8:both->0.75:8192:8:cpu,0.75:1024:16:both->0.75:8192:16:cpu
E7RESULT name=sensitivity quantity=Q2 knob=pool_data_units pairs=6 flips=6 statement=flips_with_this_knob flipped_pairs=0.25:1024:4:both->0.25:8192:4:cpu,0.25:1024:8:both->0.25:8192:8:cpu,0.25:1024:16:both->0.25:8192:16:cpu,0.75:1024:4:both->0.75:8192:4:cpu,0.75:1024:8:both->0.75:8192:8:cpu,0.75:1024:16:both->0.75:8192:16:cpu
```

第二块（第 2 行的读数，第二段整行：丙段判定、批判定的一行，份额最大的两行批行）：

```text
E7RESULT name=choose_judgement fraction=0.25 pool_data_units=1024 batch_units=4 rounds=5 choose_share_per_batch_per_round=0.014847,0.014637,0.014529,0.014800,0.014522 choose_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth choose_share_judgement=not_worth
E7RESULT name=stage_judgement fraction=0.25 pool_data_units=1024 batch_units=4 batch_index=0 rounds=5 crc_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth crc_share_judgement=not_worth scan_increment_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth scan_increment_share_judgement=not_worth choose_share_judgement_per_round=not_run,not_run,not_run,not_run,not_run choose_share_judgement=not_run
E7RESULT name=stage fraction=0.75 pool_data_units=1024 batch_units=4 round=1 call_index=1 kind=batch call_cpu_nanoseconds=12578975 crc_cpu_nanoseconds=257032.000000 crc_wall_nanoseconds=257271.000000 crc_repetitions=1 scan_cpu_nanoseconds=332311.000000 scan_increment_cpu_nanoseconds=75279.000000 choose_cpu_nanoseconds=none choose_calls=0 crc_bytes=786432 records_read_per_choice=1069 full_segments=3312 crc_share=0.020433 scan_increment_share=0.005985 choose_share_per_batch=none choose_share_of_the_issue=none crc_bytes_per_second=3059665722.555947 scan_increment_bytes_per_second=10446897541.146935 choose_record_bytes_per_second=none choose_records_per_second=none choose_full_segments_per_second=none snapshot_branch_bytes=0 snapshot_branch_note=the_pools_have_no_snapshot release_check_crc_cpu_nanoseconds=84920.000000 release_check_crc_wall_nanoseconds=85020.000000 release_check_crc_repetitions=1 release_check_crc_bytes=262144 crc_with_the_release_check_cpu_nanoseconds=341952.000000 crc_with_the_release_check_bytes=1048576
E7RESULT name=stage fraction=0.75 pool_data_units=1024 batch_units=16 round=3 call_index=1 kind=batch call_cpu_nanoseconds=13777990 crc_cpu_nanoseconds=247692.000000 crc_wall_nanoseconds=247821.000000 crc_repetitions=1 scan_cpu_nanoseconds=337532.000000 scan_increment_cpu_nanoseconds=89840.000000 choose_cpu_nanoseconds=none choose_calls=0 crc_bytes=786432 records_read_per_choice=1069 full_segments=3312 crc_share=0.017977 scan_increment_share=0.006521 choose_share_per_batch=none choose_share_of_the_issue=none crc_bytes_per_second=3175039968.993751 scan_increment_bytes_per_second=8753695458.593056 choose_record_bytes_per_second=none choose_records_per_second=none choose_full_segments_per_second=none snapshot_branch_bytes=0 snapshot_branch_note=the_pools_have_no_snapshot release_check_crc_cpu_nanoseconds=330301.000000 release_check_crc_wall_nanoseconds=330562.000000 release_check_crc_repetitions=1 release_check_crc_bytes=1048576 crc_with_the_release_check_cpu_nanoseconds=577993.000000 crc_with_the_release_check_bytes=1835008
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:275-284`（整段抄，未转述）**

```markdown
### 做了什么

1. 计时包装设备：包住内存盘（`SparseBlockDevice`）或真盘（ext4 上的 `O_DIRECT` 镜像文件，`DirectInputOutputBlockDevice`），`BlockDevice` 的每一种调用前后各读一次 `Instant` 与本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`（CPU 读数包着挂钟读数），按调用种类累计挂钟、CPU、等盘（挂钟减 CPU）、次数与字节；读按「这个计时窗口里第一次写之前 / 之后」分开，第一次写之前的读再分三栏：落在 R 那块盘 R 里的、扣掉搬迁源槽上的再读，归扫描读；这一批搬迁源的槽上（两块盘同号槽）扣掉 R 那块盘每个槽第一次读之后剩下的，归释放前读盘核（`release_check_read_bytes`，登记第十二节 12.5）；其余归别处；写按「落在这一批目的地 / 别的」分开。阳性对照的注入（每次 `write_at` 里睡 2 ms，或按线程 CPU 时钟自旋 2 ms）也在这里。
2. 造池：harness 默认参数 mkfs 两块 4 GiB 内存盘、第一个文件、可写挂载；建 P + 64 个空 inode，前 P 个各顺序写一个数据单元；整段都是这次写的段按槽序每 4 个覆盖写 3 个（f = 0.25）或 1 个（f = 0.75），混住的段这次写的单元全覆盖写掉；用多建的空 inode 把最高那一段补满；再「抬 F 到准入上限、建一个 inode」转到被覆盖写的旧落点都已释放。**覆盖写的次序（改法甲，装置 `overwrite_order_of`，第 2131 行）**：f = 0.75 仍全部按旧槽号升序；f = 0.25 改成三组依次：① 不在「整段 32 个单元都是这次写的」段里的单元（混住段的全部，与配方累计点名的），组内按旧槽号升序；② 整段里要覆盖写的单元，除槽号最大的那一整段之外，按旧槽号升序；③ 槽号最大的那一整段里要覆盖写的 24 个，按旧槽号升序。覆盖写哪些单元、写几次、写的内容不变，只换次序。机理（登记第七节「推的预测」）：每次覆盖写是一次发布、写一条根，根环 24 条根，旧落点在覆盖写那一次发布就记成已释放，覆盖写到第 25 次起每份副本都落进 24 次之前打出的洞，所以最后 24 次覆盖写打出的洞（槽号最大那一整段的 24 个）造完时还空着，那一段剩 16 个活槽、全是这次写的。
3. 几何核：在一份拷贝上可写挂载、照会话签发那一步的选段那一圈重放（`choose_region` 与「先扫」交替），看 greedy 将选哪一段；那一段全是这次写的数据单元、活槽 = 64f 就用 greedy，不是而它全是这次写的数据单元就把它们并进配方重造（至多三次），装着别的单元就走登记好的后备（只供测试的强制选段点名槽号最小的 f 段）。造池之后逐段打成分（`name=segment_composition`，N11）、造成段数两条路数并列（`name=segment_composition_summary` 的 `counts_agree`）、覆盖写副本的落点（`name=overwrite_landing`，N12）；这几行只打每个点头一次造池，第 2–5 轮的重造不打。
4. 量一条意图：可写挂载（`mount_writable_with_compaction_switches`，产品开关只把批大小换成只供测试的 `ForcedForTests(K)`），每次调用之前把会话「已经过去的发布数」设成 1000（发布预算落到 1），一次一次调 `run_compaction` 到最后一批删意图；每次调用前后读 `Instant` 与线程 CPU 时钟（挂钟读数包着 CPU 读数，与设备调用那一层反过来），返回之后在计时窗口外按发成的那一版判是签发、批还是最后一批。每条意图前后还在计时窗口外做一遍校准（N1、N15）和线程与机器读数（N3–N9，`name=intent_machine_readings`）：校准比 > 1.10 的那一条作废，作废之后先每 10 秒做一次校准、连着两次 ≤ 1.10 才重量，单次至多等 300 秒、一段里累计至多等 600 秒（修订 R2-1、R2-2）；每段开头做校准基线（N2，21 遍）与加倍活的自检（R2-3），段末再做一遍基线（R2-4）。段开头、每个取样点开头、段末各一行 `name=machine_state`（N10）。
5. 第二段的重放：调用之前取这一版的分配记录、映射条目（照会话的私有取法另写一份）、钉住与隔离的槽，取扫描读到的单元字节；三个阶段（甲 每个被读单元整单元 CRC-32C（另一个子项是释放前读盘核那几份副本的整单元 CRC-32C，副本字节在调用之前从两块盘同号槽各读一份）、乙 整个 `scan_region`、丙 签发那一步每次 `choose_region`）每次重放先不计时跑一遍热身、再计时跑一遍，5 次重放取中位；饱和吞吐 1、16、N 线程各 2 秒。丙段的判定（改动六）：每轮取那一轮有效意图签发那一行已经算出的 `choose_share_per_batch`，按原门槛（10%）判，五轮收成一行 `name=choose_judgement`（`name=stage_judgement` 行照旧打，它的丙段那一格仍是 `not_run`）。
6. 第三段（沿用，这一趟不跑）：每张卡用造池时写的一个真数据单元（32768 字节）、n 取 1 到 16384 段，`GpuCard::crc32c_of_ranges` 先丢一次再计 5 次取中位，核每段 CRC 与 CPU 相同、每次不快过 PCIe 链路峰值；最小二乘拟合 a、b，用第一段的 B_批 算 φ 与 n*，用第二段的 16 线程饱和吞吐算 D24（后台重活能不能卸给 GPU） 已定项 3 形式的比（Q9）。
7. 单点核（`geometry-check` 子命令，改动二）：只造池、核几何，打 `name=geometry_attempt`、`name=geometry`、`name=segment_composition`、`name=segment_composition_summary`、`name=overwrite_landing`、`name=pool` 几行与末行，不量；`--overwrite-order slot-order` 照第 1 次重跑那一版造（全部按旧槽号升序，不数副本落点）。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:285-315`（整段抄，未转述）**

````markdown
### 复跑（第 2 次重跑；命令照重跑登记第五节「跑法」，起法在 `scripts/experiment-apparatus/e185-r2-segment-run.sh` 文件头）

```bash
# 仓根。装置在 checker 档包里，编与单测归主 agent 带前缀跑（主工作区那一版 118 条）
cargo test --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split
# 三趟（单点核、第一段、第二段）各一条命令，主 agent 在本机仓根起，经 multi-host-run.sh --peer 在第二台跑；
# <段> 取 geometry-check、wall-clock、stage-split，<日期> 是那一趟起跑当天，产物名见上面「这一趟的产物」
bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/multi-host-run.sh --peer --memory-cap 46G --label e185-r2-<段>-<日期> --fetch research/results/<产物名>… -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r2-segment-run.sh <段>
```

`e185-r2-segment-run.sh` 在拷过去的树里做四步：写输入清单 `research/results/e185-r2-inputs-<段>-<日期>.txt` → `bash scripts/hosts/capped.sh 31 cargo build --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split`（不带 `gpu` 特性，不带 `--all-targets`、`--tests`，免得并进 `lookup-work-counters`）→ 跑这一段（装置经 `capped.sh 31` 起，环境 `E185_REGISTERED_APPARATUS_SHA256`、`E185_MEMORY_CAP=40G`）→ 产物写进 `research/results/`：

```bash
# geometry-check 段（单点核，两条）
target/release/e185_gc_batch_wall_clock_split geometry-check --point 0.25:1024 --point 0.75:1024 > research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-<日期>.out
target/release/e185_gc_batch_wall_clock_split geometry-check --point 0.25:1024 --point 0.75:1024 --overwrite-order slot-order > research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-<日期>.out
# wall-clock 段（第一段；真盘臂的镜像放在树里 target/e185-images，落在第二台那块盘的 ext4 上、不在开机清空的目录里）
target/release/e185_gc_batch_wall_clock_split wall-clock --image-directory target/e185-images > research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-<日期>.out
# stage-split 段（第二段；只用内存盘，与第一段同一台、先后跑）
target/release/e185_gc_batch_wall_clock_split stage-split > research/results/e185-gc-batch-wall-clock-split-stage-split-r2-<日期>.out
```

三趟在第二台先后跑、不同时跑；装置线程不设，取登记值 `.claude/heavy-thread-default`（跑的那一刻是 31）；产物文件名的日期是那一趟起跑当天，同一天再跑加 `-2`、`-3`，不盖已有的。三趟当时跑的是合并之前仓根的 `e185-r2-tools/run-<段>.sh`，每段做的事与上面相同。

第三段与第四段这一趟不跑。主 agent 定要 f = 0.25 造成了的点上的 Q9 时另跑第三段（条件照重跑登记第五节「第三段跑不跑」，三丁换成「主 agent 定要」；命令形态照第 1 次重跑，`--wall-clock-product`、`--stage-split-product` 换成这一趟的两份 `-r2-` 产物，带 `gpu` 特性另编）：

```bash
cargo build --release -p singlefs-checker-tier --features gpu --bin e185_gc_batch_wall_clock_split
bash scripts/crash/gpu-quiesce-run.sh -- target/release/e185_gc_batch_wall_clock_split gpu-submission --wall-clock-product research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out --stage-split-product research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out --cards 0,1 > research/results/e185-gc-batch-wall-clock-split-gpu-submission-r2-<日期>.out
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:316-330`（整段抄，未转述）**

```markdown
### 口径

- CPU 时间是本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` 两次读数之差（登记第一节，第十二节 12.2 修订；原来的 schedstat 读法对正在跑的线程约 1 ms 才前进一次，实测数在 12.2）。`wall-clock`、`stage-split`、`foreground-fsync` 开跑先在本线程上自旋 1 ms、连读这个钟，前进不到 100 次就停机退 3（产物行 `name=clock_resolution_check`）。设备调用那一层 CPU 读数包着挂钟读数，等盘 = 挂钟 − CPU，纯 CPU 的设备调用里可以是负的（差读钟的工夫）；一次 `run_compaction` 调用那一层挂钟读数包着 CPU 读数。这样嵌套，两个钟走得一样快时「挂钟 ≥ CPU + 等盘」不会因为读钟的先后被打破。
- 一批的挂钟是包住那一次 `run_compaction` 调用（含调用末尾的 discard）的 `Instant`；discard、屏障、写零各按调用种类单列。
- 真盘臂：每次测量之前把上一次改过的 4 MiB 块从起始件拷回工作件，再核整份镜像的分块摘要（每 4 MiB 一块的 SHA-256 接起来再算一次 SHA-256）与起始件相同；内存臂每轮按同一份配方重造、核两块盘的分块摘要与第一次造的相同。挂载时 journal 记录偏移的提示照抄单元区之前写过的扇区（同内存盘的答法），不在 `O_DIRECT` 镜像上把整个环逐槽读一遍；计时窗口里不照抄。
- 设备：内存臂是 harness 的稀疏内存盘；真盘臂是两个镜像文件，放在第二台一块 NVMe（型号 CT2000T500SSD8，逻辑块 512，`research/results/e185-r2-inputs-wall-clock-2026-10-23.txt` 第 22–26 行）上的 ext4 里，屏障是 `fdatasync`，结论只说「第二台那块盘上 ext4 的 `O_DIRECT` 镜像文件」（重跑登记第一节「真盘」）；原登记写的本机那块盘没用。
- 批：一次调用里搬了单元的那一次（与会话 `observations.batches_published` 同一口径）；签发单列，不算批。
- Q4 的 `rises` 只要求第一批挂钟的比 > 1（5 轮中位，且每轮 > 1）；这一趟 12 对的中位在 1.015102–1.074115，涨幅小，判据不看涨了多少。
- Q7 的「一批 CPU」取这条意图批 CPU 的中位（装置从原登记起的读法，登记第六节「丙段怎么有读数」写死）；丙段的值是签发那一次 `name=stage` 行的 `choose_share_per_batch`，每轮取那一轮有效意图的一个，所以 `name=choose_judgement` 每格只有 5 个值。
- 第一行阳性对照（登记第五节阳性对照表第 1 行，CPU 臂等盘那一句照第十二节 12.9）：取样点 f = 0.75、P = 1024、K = 8，每次 `write_at` 里睡 2 ms（盘臂）或按线程 CPU 时钟自旋 2 ms（CPU 臂）。增量一律是对照各轮的中位 − 基线各轮的中位。盘臂：整条意图等盘增量落在写调用次数 × 2 ms 的 [1.0, 1.5] 倍里、CPU 变化在 ±10% 以内（没改）。CPU 臂两句各自报：CPU 增量落在写调用次数 × 2 ms 的 ±10% 里（没改）；写调用（注入所在的那一种调用）的等盘增量的绝对值 ≤ 写调用次数 × 2 ms × 10%（12.9 的新判法，分母是注入量）。原判据「整条意图等盘相对基线的变化在 ±10% 以内」照算照报（`wait_change`、`wait_clause_under_the_original_criterion_holds`），不作判定用。对照点每一轮里，每个设备臂挨着量基线、盘臂对照、CPU 臂对照各一次，次序按轮号轮换（第 1、4 轮 基线 → 盘臂 → CPU 臂，第 2、5 轮 盘臂 → CPU 臂 → 基线，第 3 轮 CPU 臂 → 基线 → 盘臂），轮数 5；按轮号配对的增量中位另报、不判。
- 作废与停机（登记第十一节）：原登记八条作废、12.10 第 ③ 条与第 1 次重跑的「计数特性被并进来」照沿用；修订 R2-1 加第三样触发（这条意图前后的校准比有一个 > 1.10，`invalid_reason=calibration_ratio_above_the_registered_edge`），R2-2 作废后先等常态再重试，R2-3 校准自己的阳性对照不过就停（退 4），R2-4 基线漂移只打标记；停机条款第 7 条（装置变了，输入清单第一行与登记的哈希对不上，另有修订 R2-8 在装置里也核一遍）至第 10 条（校准锚点不符）与修订 R2-5、R2-7 把基线读不到、锚点不符的遍数累计超过 3 遍收严到停（退 3）；修订 R2-6 校准读不到那一条意图记 `invalid_reason=calibration_unreadable`。这一趟这些都没触发（两段判决行 `stop_clause_findings=0`，140 + 60 条意图 `invalid_reason=none`）。
- 第三段的 B_批（沿用）取第 1 次重跑第一段内存臂、第 1 轮、`kind` 为 `batch` 或 `final_batch` 那几行的 `scan_read_bytes_in_the_region`；φ 与 n* 按两张卡各自拟合的 a、b 算。
- 产物不带执行那台机器的路径：第一段 `name=configuration` 不打 `--image-directory` 的值，只打 `image_directory_filesystem`（目录所在设备在 `/proc/self/mountinfo` 里的文件系统种类）、`image_directory_is_cleared_at_boot`（tmpfs / ramfs，或真路径落在 `/tmp` 下为 `true`）与 `image_device`（设备号）；镜像建不成的报错只带文件名（登记第十二节 12.6）；输入清单里第二台的目录换成「〔第二台的目录〕」。
- 键名写全名，登记里的提问与条款编号只当索引：第二段甲、乙、丙三段的份额是 `name=stage` 的 `crc_share`、`scan_increment_share`、`choose_share_per_batch`（签发那一次另报 `choose_share_of_the_issue`，丙段判定读 `name=choose_judgement`）；Q16 是第一段判决行的 `gc_to_sequential_write_bandwidth_maximum_ratio` 与 `gc_to_sequential_write_bandwidth_judgement`；F1、F2、F5 是 `clause_wall_clock_not_rising_with_the_batch_triggered`、`clause_sixteen_thread_form_misfits_the_gc_triggered`、`clause_two_cpu_paths_disagree_points`；Q17 是第四段判决行的 `foreground_fsync_latency_judgement`。新旧对照表在登记 12.6。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:331-348`（整段抄，未转述）**

```markdown
### 它答不了的

- f = 0.25 量到的是被强制选定的那一段，不是 greedy 自己选中的段：改法甲下 greedy 选中的仍是装着别的单元的段（单点核第 3 行 `greedy_formed=false`），f = 0.25 与 f = 0.75 一样走只供测试的强制选段后备。这一页的第 1 行读数只对「被强制选定的、全是这次写的数据单元的段」成立，对 greedy 自己会选中的段没有读数。
- 改法甲造成的几何靠一条机理（根环 24 条根、旧落点在覆盖写那一次发布就记成已释放），机理核对只在四个点（f = 0.25、0.75 × P = 1024、8192）与单点核上对上；别的 f、P、别的覆盖写次序没试，也没试登记里的改法乙（快照留旧单元，装置没实现）。
- 占比 0.75 的池与第 1 次重跑不同（差在两棵树，「几何」一节），所以这一趟的数不与第 1 次重跑的数逐格比；七·6 前后的比法这一趟没有。
- 第 3、4 行沿用第 1 次重跑：第三段的 Q9–Q13 在 f = 0.25 的两点上量的是几何没造成的段（`batch_bytes=16384`、`merged_buffer_bytes=16` 不是显存占用：B_批 16384 字节不满一个 32768 字节的槽，装置把每批段数整除成 0 再算缓冲），f = 0.25 造成之后的 φ 与 n* 没量；第三段装置 `eadb929d…` 与重跑登记写死的 `b3a50d50…` 差的 3 行不在第三段路径上这一句是读 `git diff` 推的，没另量，主 agent 已认。
- Q9 照原登记算、照原样报；今天的 D26（后台整理与放置回收） 已定项 13 第 7 条写 D24（后台重活能不能卸给 GPU） 已定项 3 按挂钟比的判据不管它罩着的后台任务，Q9 在第 2 行还算不算翻面量，这一页不判，交主 agent，并进七·2 设计轮定（重跑登记第六节、第一节）。
- 第 1 行读数的 P 轴只有 1024、8192 两点（装置只收这两个池大小）：翻面发生在两点之间的哪里、更大的池上是什么，没量。
- Q4 的 `rises` 涨幅小（1.5%–7.4%）：「批大小是不是挂钟的主旋钮」这个问法这一页不下判断，F1 没触发只说明每一对都 > 1。
- 校准门槛 1.10 的依据只有本机上一个代用量（登记第四节第 11 行，zlib 的 CRC-32 在本机、负载 18 时的分布），第二台与装置自己的 CRC-32C 没量过；门槛在跑前写死，没看这一趟的数再改。这一趟没有一条意图越过它（最大 1.077323），所以它实际没动任何一格（N14 并列判定 24 行无一差异），也就不能说明门槛放在哪合适。追指针校准比（N15）每条意图都高于 1（1.23–1.53），没查原因。
- 校准只量测量线程此刻跑同一份活要多久，不排除对象自身（GC 路径）在个别意图里变慢；H3、H4 以 F5 打中为前提，这一趟没打中，两条没用上。F5 没触发也不能反推第 1 次重跑的 F5 是机器状态：这一趟的机器状态读数（`others_share` 最大 0.002502）与第 1 次重跑没有同一口径的对照读数。
- 第二段清场记录第一行是 `joined-and-read-back holders=1`：别的会话已登记同用卡，本趟加入而不是新停；计时窗口里别的会话有没有占第二台的 CPU，只能读 `name=machine_state` 的 `others_share`（最大 0.001875）与每条意图的排队等 CPU 读数，那是读数不是隔离。
- 释放前读盘核那一栏（登记第十二节 12.5）只认这一批搬迁源的槽。同一次发布对被换下的旧单元（重写的树节点这类）做的释放前读盘核走同一个函数、也在第一次写之前，落在 R 外，仍记在 `scan_read_bytes_elsewhere`：这一条是读 `transaction.rs` 推的，没量。分栏靠「区域扫描在释放前读盘核之前、扫描对 R 那块盘每个槽只读一遍」这个次序；调查员只在 f = 0.75、P = 128、K = 8 一个点上、只对 R 那块盘抓过调用方，另一块盘那一份是推的，单测端到端那一条在同一个点上钉住两块盘合计 2 × 搬的单元数 × 32768。甲段第二个子项不进 Q5、Q8、Q10 与第三段的 B_批，只报数。
- 几何后备：greedy 将选的段装着节点或打包单元时，登记只举了「多覆盖写那几段、让节点段的旧节点也被引用着」两种改法，装置只实现了前一种；竞争段装着别的单元时直接走后备、不试三次（登记第十二节 12.3 写死了这个读法），`name=geometry` 行写明原因；后一种改法没有试过。
- 第二段的饱和吞吐量的是重放（同一份输入反复跑），不是 GC 自己在多线程上跑：今天的 GC 整条路在调用线程上同步跑。
- 第一行阳性对照照 12.9 判「中位之差」，按轮配对的增量只报不判（这一趟 CPU 臂真盘格 `write_wait_increment_paired_median_nanoseconds=1684671.000000`，内存格 `67472.000000`）；交错之后配对判不判、怎么判没有定。
- 停机条款第 6 条（包含自检反过来）照登记字面判，但两个钟走得不一样快：线程 CPU 时钟不受校时调速，`Instant`（`CLOCK_MONOTONIC`）受。2026-10-19 执行员在本机用 python 量过（忙等、挂钟读数包着 CPU 读数）：200 ms 的窗口里 CPU 读数最多比挂钟多 1526 纳秒（比值最大 1.000007630），`CLOCK_MONOTONIC` 比 `CLOCK_MONOTONIC_RAW` 慢 7.036 ppm；挂钟换成 `CLOCK_MONOTONIC_RAW` 时 CPU 读数没有一次大过挂钟（200 ms 窗口最多 −194 纳秒）。几乎全在 CPU 上、又长到几百毫秒的调用（内存臂）因此可能让第 6 条触发，读钟那几十纳秒的余量盖不住；与线程 CPU 读数配对的挂钟已照 12.4 换成 `CLOCK_MONOTONIC_RAW`，这一趟两段判决行都是 `stop_clause_findings=0`，第 6 条没触发。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:349-368`（整段抄，未转述）**

```markdown
### 路径与结论登记

同一个量两条或以上互不共享代码的路径各算一遍再比对的，有五样：

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 扫描读字节：设备计数 | 计时包装记下的、第一次写之前落在 R 那块盘 R 里的读，扣掉这一批搬迁源槽上的再读（释放前读盘核，登记第十二节 12.5），长度相加 | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:3267 split_the_reads_of_a_call` | 区域 R（会话签发的那一条意图）、槽字节 16384、这一批搬迁源的槽（会话交回的 `units_relocated_by_this_publish` 的 `source`）、读的次序（扫描在释放前读盘核之前） |
| 2 | 扫描读字节：分配记录 | 调用之前那一版 R 那块盘上未释放记录的 Σ 跨度 × 16384 | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:1429 expected_scan_read_bytes` | 同一个 R、同一个槽字节常量 |
| 3 | 批数：装置分类 | 每次调用发成的那一版里 logged_ops 还有没有意图、搬了几个单元 | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:1369 kind_of_the_call` | 会话交回的那一版 |
| 4 | 批数：会话计数 | 会话 `observations.batches_published` 的增量 | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:4937 stop_findings_of` | 无（会话自己数） |
| 5 | 批数：闭式 | ⌈m ÷ K⌉ | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:1401 batches_of_one_intent` | 活槽占比定的 m |
| 6 | 一批的 CPU：真盘臂线程 CPU 时钟 | 真盘上一批的本线程 CPU | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:706 thread_cpu_nanoseconds` | 本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`（clock id 3） |
| 7 | 一批的 CPU：内存臂挂钟 | 内存盘上同一批的挂钟（等盘近 0） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:1561 cpu_reconciliation` | 同一份配方造的池；`Instant`（`CLOCK_MONOTONIC`），与路径 6 的钟走得不一样快（停机条款第 6 条那一条，口径里） |
| 8 | 造成段数：逐段成分 | 单元区里每一段（有未释放记录的段）逐段数成分，数「造成」为真的段（`name=segment_composition_summary` 的 `formed_segments`） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:2901 construction_reading_lines` | `is_the_geometry_formed`（活槽 = 64f 且全是这次写的数据单元，第 2784 行）、`composition_of_the_segment`、同一份分配记录 |
| 9 | 造成段数：几何核那条路数 | 对着可写挂载后的那一版，按造池记下的数据单元槽所在的段起点筛「造成」为真的段（`geometry_formed_segments`） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:2951 predict_the_greedy_choice` | 同路径 8：`is_the_geometry_formed`、`composition_of_the_segment`、同一份分配记录 |
| 10 | 落进旧洞的副本：逐次数 | 造池第 2 步每次覆盖写之后，看这一份新副本的槽是不是前面某次覆盖写空出来的旧槽（`copies_landing_in_holes_of_this_construction`） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:2186 landings_of_the_copies` | 覆盖写的次序（改法甲的 `overwrite_order_of`）、每次覆盖写前后的槽号 |
| 11 | 落进旧洞的副本：闭式推的预测 | `overwritten_units − 24`（24 = 根环条数） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:2210 predicted_copies_landing_in_holes` | 本地常量 `LOCAL_ROOT_RING_ENTRIES`（开跑与 `singlefs_format` 的两个常量之积回比）；机理前提：每次覆盖写一次发布、一条根，洞从第 j + 24 次发布起分得到，用户数据取最低的空槽对 |

源码落点的行号按主工作区那一份装置（sha256 前 16 位 `0bd2274383f12e88`）现查。跨路径的比对：1 与 2 由停机条款第 2 条逐批连起来（`stop_findings_of`，对不上就停）；3 与 4 由停机条款第 3 条连起来，3 与 5 只在几何造成了的点上连（同一函数）；6 与 7 是 Q3，> 20% 记第十条 F5，不停；8 与 9 由 `counts_agree` 连起来（`construction_reading_lines` 里 `formed_segments == prediction.geometry_formed_segments.len()`，单测 `construction_reading_lines_compare_the_two_counts_of_formed_segments_and_report_the_landing`，第 13124 行），对不上写 `counts_agree=false`、不停机，8 与 9 共用同一个判成造的函数，这个函数错了两边一起错，由「甲造成」那一格 `live_slots=16 slots_of_this_workload=16 other_slots=0` 的绝对值兜；10 与 11 只在 `name=overwrite_landing` 里并列打出，对不上记第十条 H2、不停机不作废，单测钉了闭式的四个值（第 12344–12347 行），没有一处断言把逐次数的与闭式的连起来，还欠这一条。取值是枚举点（两个 f、两个 P、三个 K、两个设备；8 至 11 是四个 f、P 点），理由是登记第八节只要求这几档；1 与 2 共用同一个 R，R 取错时两边一起错，由第七节「签发那一步扫 R 读到的字节」的绝对值锚点兜。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:369-377`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-10-23 不受影响：第 2 次重跑给出的是里程碑四设计轮要的读数（问题单第 1 行：池小（P = 1024）两边都占、池大（P = 8192）CPU 限；第 2 行：甲、乙、丙三段份额都不到 10%，F7 打中；第 3 行沿用第 1 次重跑 Q13 全判要攒批；第 4 行单跑够，比值 0.015977），该分项射程把「GPU 上算 GC 的哪几段」与定时兜底间隔、写带宽上限的默认值留给设计轮与 C412（整理的挂钟代价没量过），依据段是「无实验」、没引 E185（GC一批搬迁的挂钟拆分）；Q9 的用法与第 7 条是否相左，这一趟没重算（第三段沿用），Q9 算不算翻面量交主 agent，并进七·2 设计轮定；决策正文这一次没有改 |
| D26（后台整理与放置回收） 已定项 1 | 备料 | 2026-10-23 不受影响：这一趟装置照旧读它的停机谓词（R 按记录全空）判最后一批，不量三水位阈值；f = 0.25 几何这一趟用改法甲造成了，是负载次序的事，与触发判据无关；该分项依据段没引 E185（GC一批搬迁的挂钟拆分），决策正文这一次没有改 |
| D24（后台重活能不能卸给 GPU） 已定项 3 | 备料 | 2026-10-23 不受影响：Q9 沿用第 1 次重跑第三段（72 行全判 GPU 输、F2 没触发），这一趟没重算；第二段又量了 CPU 一侧的 16 线程饱和吞吐（甲 48832371460.2–49032300970.9 B/s），没有代进该分项的形式；该分项依据段没引 E185（GC一批搬迁的挂钟拆分）；Q9 算不算问题单第 2 行的翻面量交主 agent；决策正文这一次没有改 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 不影响 | 2026-10-23 不受影响：只作定义引用（GPU 结果要复核、GPU 与 CPU 两份实现）：第三段逐段对拍 CRC 的 32 行 `crcs_agree=true` 是第 1 次重跑第三段的装置作废检查，这一趟第三段不跑，不量运行时两份 GPU 核各算一次比对的复核政策，结论不变 |

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:1-6`（整段抄，未转述）**

```markdown
## D26 后台整理与放置回收 —— 半定（三项未定）

**中文** · [English](26-background-compaction-and-placement-reclamation.en.md) · [日本語](26-バックグラウンド整理と配置回収.ja.md)

D26（后台整理与放置回收） 管长期运行积累的碎片由谁解决：两条腿各是什么、整理怎么起跑怎么停、走哪套事务形态、要不要申请空间、与记账怎么交互、观测量住哪，以及回收分哪两条线。不管的：分配落点政策在 D3（空间分配） 已定项 5 / 8，准入不等式在 D28（挂载期承诺量） 已定项 1；重平衡与搬迁单位的口径在 D2（RAID 条带策略） 已定项 8 / 11；销毁快照与销毁可写头在 D6（快照实现模型）；索引节点内部布局属 D15（格式冻结政策） 第 4 层；阈值的具体数值不在这里，等 E10（老化：结构抗老化 vs 事后整理）。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:24-44`（整段抄，未转述）**

```markdown
#### 已定项 1：整理的触发判据

**定案**：三条。

1. **起跑：三个水位，任一越线即起跑。** 全空聚簇段数（物理量、带设备维）、碎片度 `F = runs / L`（runs 的现行口径是单元区空闲槽的连续段数，物理量、带设备维，D5（快照 / 空间记账机制） 已定项 4 第 10 项）、不可回收字节（带设备维）。全空聚簇段数按记录口径（段内 64 槽都没有未释放分配记录）；代码 `empty_segments` 已是记录口径（C372（全空聚簇段数两份定义相反）的代码那一半已做），与 D3（空间分配） 已定项 10 ① 同一句。三个量都已由已定项 6 定为持久，挂载即有读数。
2. **停机：删意图那一刻 `R` 按记录全空**（`R` 本地谓词）；签发前判 `R` 外非全空段（只装提交内生块的段不算非全空）的空洞数 ≥ `R` 里要搬的单元数，不够不签；一条意图完成而全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为假就不签下一条；`segs0` 仍以值写进意图记录，只作观测量。
3. **`R` 在意图期内不可作分配目的地**（回落政策的第一条）。代价是分配路径多一条合取「候选落点不在任何活整理意图的 `R` 里」；活意图数有界（整理一次推进一个批次）⇒ 这是 O(活意图数) 的点查，不是遍历，意图本身住 logged_ops 树（D8（核心索引结构） 已定项 5），分配器读得到。

**阈值数值不进格式**：等 E10（老化：结构抗老化 vs 事后整理），在那之前取保守默认、做成挂载可调。改新池新建文件的字节：否——三个量已由已定项 6 定为记账树里的统计量、新池新建文件已经在写它们；起跑与停机是判定的形式，`segs0` 住意图记录而新池新建文件不含整理意图。

**射程**：管起跑与停机的形式、以及 `R` 的排他，不管阈值的具体数值。意图达成之后有没有产出由第 2 条的 `R` 本地谓词判，全局差值只决定签不签下一条；「每 N 条意图完成之后全空段数严格增加」那条 checker 欠在 C140（意图完成不等于空段被腾出） 名下。`R` 之外无空段时分配落到哪，仓里仍无条款。「`R` 不可作分配目的地」这个状态在盘上没有独立落点，靠读活意图推出来——与 C127（钉住状态在分配记录里没有落点） 同形，但这一条有输入（意图记录在盘上）。边际收益（`Δ碎片度 ÷ Δ写字节`）这个停机形态不可用：分子取全局 `runs` 时符号在实验轨迹上全是错的，分母取「这一批整理已写的字节」是历史的函数、从盘上状态导不回来，写进意图记录就是「声明增量」，撞 D8（核心索引结构） 已定项 5 硬要求 1 逐字「意图声明目标状态，绝不声明增量」。

**依据**：

- E93（老化下的放置与碎片度）：碎片度可 O(1) 增量维护；聚簇段政策没有整理喂空段就不自持；预算最高那条整理臂的 `runs` 轨迹单调上行——边际收益停机判据的分子因此在每个采样点符号都是错的。
- 三方论证第三轮（原样输出 `research/prompts/d26-r3-{sonnet,opus,local}-output.md`）：停机判据换形态、回落政策要加一条。
- 用户定案（三水位、产出谓词、`R` 排他），原话在变更史。
- 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G4：停机改成 `R` 本地谓词，签发前判空洞、无产出就不签下一条（攻方模型上签发前判之后四个几何活锁 0，无产出就停之后全部几何活锁 0）；C372（全空聚簇段数两份定义相反） 按记录口径收口。第七节用户定案「重开：R 本地谓词 + 签发前判 + 无产出就停」，原话在变更史。

**欠**：C140（意图完成不等于空段被腾出）；C146（无空段时的回落政策全仓无定义）；C243（停机谓词那个量从来没有过轨迹）；C364（整理意图记录的字段没有逐字段对账）；C372（全空聚簇段数两份定义相反）；C373（停机谓词判真时活意图删不删没有条款）。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:45-85`（整段抄，未转述）**

```markdown
#### 已定项 2：整理的事务形态

**定案**：两半。

| 半 | 定案 |
|---|---|
| 提交步骤 | **不新增 `CommitStep` 成员**。整理是普通事务序列，无界总量走 D8（核心索引结构） 已定项 5 的意图机制——那一条逐字管「凡是**可能修改无界数量 extent** 的操作……先把意图写进 logged_ops 树，再分批推进」，整理在射程内 |
| 意图的目标态 | **「`R` 内不存在分配代 ≤ `c0` 且未释放的落点」**，`c0` = 写下这条意图那一刻的分配代。**不写「清空区域 `R`」** |

这个谓词可用的三条：**单调**——前台在 `R` 内新分配的落点，其分配代必然 > `c0` ⇒ 不违反目标态 ⇒ 前台写不再阻止意图完成；**算得出**——两个输入都在 D3（空间分配） 已定项 7 的分配记录条目里（value 是「分配代或释放代 8 + 跨度段 2」，跨度段最高位已被「已释放」借走）；**零格式位**——不新增任何盘上字段，`c0` 住意图记录。改新池新建文件的字节：否——新池新建文件不含整理步骤，枚举里加不加新成员、意图谓词怎么写，都不改它写出的字节。

**整理意图条目的字段表**（logged_ops 树的条目，读写在 `crates/singlefs-core/src/logged_operations.rs` 的 `to_bytes` 与 `parse`，小端）：

| 段 | 字段 | 宽 | 偏移 | 取值 |
|---|---|---|---|---|
| key | 种类 | 2 | 0 | 码 1 = 整理；0 与别的码保留，读者拒收 |
| key | 签发 txg | 8 | 2 | 签发这条意图的那次发布的 txg |
| value | `R` 的设备 | 4 | 10 | `R` 所在的盘 |
| value | `R` 的起始槽 | 6 | 14 | 64 的倍数，不对齐拒收 |
| value | `R` 的槽数 | 2 | 20 | 恒 64（一个聚簇段），别的值拒收 |
| value | `c0` | 8 | 22 | 签发那一刻的分配代 |
| value | `segs0` | 8 | 30 | 签发那次发布写下的记账第 11 项（全空聚簇段数，D5（快照 / 空间记账机制） 已定项 4）里 `R` 所在盘的值，只作观测量 |
| value | 补零 | 6 | 38 | 非零拒收 |

合计 44 字节：key 10、value 34 定宽，一个码 2 节点装 369 条；宽不等于 44 的条目一律拒收。
<!-- format-const: LOGGED_OPERATIONS_KEY_BYTES = 10 -->
<!-- format-const: LOGGED_OPERATIONS_VALUE_BYTES = 34 -->
<!-- format-const: LOGGED_OPERATIONS_ENTRY_BYTES = 44 -->
<!-- format-const: LOGGED_OPERATIONS_ENTRIES_PER_NODE = 369 -->

**射程**：管目标态怎么写与要不要加提交步骤。不管「达成等不等于有效」——两个聚簇段之间来回搬的历史里每一条意图都正常完成而全空段供给恒为零，那一格归已定项 1 的停机与产出判据。一批整理要改写 K+1 个已发布根，这种提交点**组合**没被崩溃点采样枚举过，归 crash harness。「清空区域 `R`」写不得的理由是它不单调：全仓没有任何一条条款说 `R` 在意图期间不可分配（已定项 1 第 3 条正是为此补的），前台写可以填回 `R` ⇒ 目标态永不达成、意图永不删。

**依据**：

- E94（搬共享落点的触碰面）：三种位置权威架构都只写树节点、根记录与既有条目，并坐实一批整理要改写 K+1 个已发布根。
- 三方论证第二轮与第三轮（原样输出 `research/prompts/d26-r2-{sonnet,opus,local}-output.md`、`research/prompts/d26-r3-{sonnet,opus,local}-output.md`）：攻击腿与找反例腿独立撞到「清空区域 `R`」不单调。
- 用户定案，原话在变更史。
- 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：logged_ops 单树、定宽联合条目，第一版只装整理意图，字段表由它定。

**欠**：C91（整理意图与增量触发量的重做漂移）；C140（意图完成不等于空段被腾出）；C364（整理意图记录的字段没有逐字段对账）。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:86-105`（整段抄，未转述）**

```markdown
#### 已定项 3：整理自己要不要申请空间

**定案**：三句。

1. **「释放空间这个操作本身不需要申请空间」作为要求原样成立**，D3（空间分配） 那条原则**不重开**——重开它会把立项时点名 btrfs 病根的那条纪律一起拆掉。不成立的是「D3（空间分配） 已定项 2 + 已定项 4 **已经兑现了它**」这句话，要重开的是 D3（空间分配） 已定项 2 的「解开自己需要多少」算法。
2. **整理不是释放路径，它走常规准入**，批次最坏占用计入「已承诺预留」。
3. **第一版就留常驻底额**，形态是准入的**第二个合取式** `(可用 ≥ 需求) ∧ (全空聚簇段数 ≥ N₀)`。**底额进不了不等式的任何一项**——它约束的是**形状**不是标量。`N₀` 取**能让棘轮不发散的最小值**，判据是「越小越好」不是「够用就行」（理由是 D3（空间分配） 已定项 13 不分 data / metadata 两类空间、全局统一分配器 ⇒ 「释放要申请空间」在这里不像 btrfs 那样致命），数值等 E10（老化：结构抗老化 vs 事后整理）。

改新池新建文件的字节：否——全空聚簇段数已由已定项 6 定为记账树里的统计量、新池新建文件已经在写它；加一个合取式是准入判定的形式，不改写出的字节。

**射程**：管整理这条路径进不进常规准入、底额要不要留与它的形状，不管 `N₀` 取多少。底额自己占的空间算准入不等式的哪一项没定——按 D28（挂载期承诺量） 已定项 1 它像「已承诺预留」那样从可用里减掉，那样又和整理批次的预留重复计一次，这一格欠一条分项。这一条不是免费的：它把「全空聚簇段数」从 `.claude/rules/fs-design.md` 第三格（后台、无戒律）拖进**第一格**（运行时决策路径，要在被问到那一刻有答案），连带欠三样——它的在飞 overlay（形态照 D28（挂载期承诺量） 已定项 2）、一条 I-3.1（已分配统计对得上） 形态的不变量（增量读数与 checker 全扫对得上）、以及设备维（已定项 6 定全空聚簇段数是物理量、带设备维，而准入不等式是逐设备求和的）。

**依据**：

- E93（老化下的放置与碎片度）：聚簇段政策没有整理喂空段就不自持，整理预算是唯一把碎片度压下来的力量——整理停摆 ⇒ 空段供给关闭 ⇒ 提交内生块散布 ⇒ 按 D3（空间分配） 已定项 5 的依据，I-8.1（环几何够大） 的最坏事务占用在全散布政策下没有可用上界 ⇒ 释放这个事务本身发不出去。这条链与 `w` 无关、第一版就成立。
- 三方论证第三轮（原样输出 `research/prompts/d26-r3-{sonnet,opus,local}-output.md`）：三腿两处独立撞到同一点，并把靶子从「重开 D3（空间分配） 的原则」改成「重开 D3（空间分配） 已定项 2 的算法」。
- 用户定案（走常规准入、第一版就留底额、底额越小越好），原话在变更史。

**欠**：C139（整理停摆之后释放也发不出去）；C90（钉住条带下的 ENOSPC 棘轮）。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:106-121`（整段抄，未转述）**

```markdown
#### 已定项 4：结构抗老化取什么形态

**定案**：格式部分——**不在节点头存组身份、不动索引节点内部布局**；放置的组只从 key 算（key 区间分组）。政策部分——聚簇段放置、可选重写半径 `R`、按 key 分组重落由 D3（空间分配） 已定项 8 与已定项 1 管，本项不再单独持有。改新池新建文件的字节：否——新池新建文件的索引节点字节不依赖本项，码 2 头那 2 字节预留照旧恒 0。

**射程**：管已登记的那两条格式臂采不采，不管「有没有第三种形态」：将来再提一种要盘上字段的抗老化形态，另立分项、另立实验，不重开本项。1 字节老化代那条路在字节上仍开着，落点是码 2 类身份段那 2 字节预留。判决对几何取样点的敏感性没有检查。

**依据**：

- E127（分裂合并之下组身份要不要存）：带分裂合并的模型里，节点数分组（要在节点头存组身份）在六个等预算格上没有一格把 `runs` 压到 key 区间分组（不要字段）的 0.8 倍以下，补齐 knob 栅格后六格全输。
- E95（老化的节点布局臂）：另一条登记过的格式臂（代际分离，节点头带老化代）按它自己的判据 5 六格没有一格过半；它的「格式 / 政策」分界是写死的标签，把同一个函数原样注册成政策臂答案行就翻面。
- E101（节点头留位的代价与表达力）：在节点头留 0 / 1 / 2 / 4 字节标签位，四档上代价一格没掉、树高不动；翻不了判定的唯一理由是节点组的组数没有上界，而节点组随 E127（分裂合并之下组身份要不要存） 退役之后那条理由消失。
- 第五轮三方论证判决 `research/prompts/d26-r5-main-verification.md`：三条腿一致；反推腿自建的最有利实现只到 0.998，并打中「同父兄弟由树结构免费给出」不成立（码 2 头无父指针、派生树头无 key 区间、搬迁走映射不改引用者）⇒ 组只从 key 算。
- 用户定案（解除「政策那一半也不切」、翻字节判定、政策落回既有政策格），原话在变更史。

**欠**：C317（判决只在一个几何取样点上量过没有检查）；C102（索引节点头没有预留区，加字段不是 compat）。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:122-155`（整段抄，未转述）**

```markdown
#### 已定项 5：整理与记账的交互

**定案**：三问的答案。

1. **搬迁不改 `birth`**（权威在 D5（快照 / 空间记账机制） 的 `birth(b)` 定义表下）。
2. **分配记录树改写量**：源侧 `⌈N/扇出⌉` 个叶 + 每层各一个内部节点（`interior += leaves.min(level_nodes)`），目的地侧散着时是 `min(N, 叶总数)`。目的地那条分配记录的分配代写单元的诞生代（不写搬迁那次发布的 txg），I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 原样成立。不随盘容量走：源侧连续时 N = 4096、16 TiB 是 13 个节点 / 212992 字节 / 2.60 倍，容量涨 256 倍只到 19 个节点，涨的 6 个全部来自树高。
3. **deadlist 条目按版本身份记**——key 取中央映射 key（D19（块指针的结构与宽度预算） 已定项 6，码 1 27 字节），不按落点记、不带位置提示，与 D5（快照 / 空间记账机制） 已定项 6 的稀疏旁表同一个 key。

**区域扫描与选段的读法**（三层是 `scan_region` → `choose_region` → `plan_batch`）：

| 问 | 读法 |
|---|---|
| 区域扫描判类的优先级 | 钉住 → 隔离 → 读头 → 类标签 → key 与位置；隔离槽（释放前核出校验和对不上）一律判隔离，不看映射 |
| 选段资格 | `R` 里至少一个能搬的单元，按 `scan_region` 的分类取 |
| 扫描整次被拒的段 | 记进本次挂载的不合格段集合，不再选；拒整次扫描的是盘上的坏记录，归 C411（分配记录的跨度段与单元类别不符没检查） 的域，整理不替它修 |
| greedy 同分 | 取起点槽号最小的段 |
| `plan_batch` 交空表 | 返回值分「搬空」与「搬不动」两种 |

改新池新建文件的字节：否——第一个可运行目标无快照 ⇒ deadlist 条目数为 0、共享落点为 0，三问的任一答案写出的字节逐字节相同；相邻的映射条目编码判「是」，那一格归 D19（块指针的结构与宽度预算） 已定项 6，不在本项。

**射程**：管搬迁对三样记账结构各做什么，另管区域扫描与选段的读法（`R` 里哪些单元能搬、选哪一段当 `R`），两半并列、互不改对方。**目的地侧的落点政策不在已定项 5 定**：没有条款保证它连续，唯一被建模过的那条政策（最低空槽优先）在整理会跑的那个状态里保证它不连续，而已定项 1 第 3 条还把全池最可能有连续空间的那一段（`R`）排除在目的地之外。三条口径注：E74（分配记录的条目数与整理开销） 的绝对值取落点粒度 32 KiB 而 D3（空间分配） 已定项 7 已定粒度 16384，且同条已把条目数口径从「已分配落点数」改成「曾分配过、尚未被覆盖的落点数」⇒ 那些数是双重旧口径；「370 倍」只属于 16 TiB 那一档（4823 / 13 = 371），1 TiB 是 319 倍、256 TiB 是 432 倍；D3（空间分配） 已定项 3 正文那句「那条 key 把整理放在了好的那一端」只对源侧成立。`birth` 是**逻辑版本**的属性 ⇒ 拿它当「这份物理副本有多新」用的消费者（D13（验证路线） 已定项 8 的代际增量 scrub）对搬迁一律不可见。

**依据**：

- E94（搬共享落点的触碰面）：按中央映射 key 记 deadlist 时搬迁触碰面恒零重写，按指针记每批重写 11–36 条；指针位置权威下搬批代价随快照数走、每批重写全部快照根。
- E74（分配记录的条目数与整理开销）：整理代价不随盘容量走，COW 放大全看落点聚不聚——源侧与目的地侧两个式子由它量出。
- 2026-09-08 三轮对抗论证与 2026-09-10 第四轮（原样输出 `research/prompts/d26-item5-r{1,2,3}-{sonnet,opus,local}-output.md`、`research/prompts/d26-item5-r4-{opus,sonnet,local}-output.md`）：① 与 ② 结案，③ 的方向与挡路条由这两轮定下。
- 用户定案随 D5（快照 / 空间记账机制） 已定项 6 收口，原话在变更史。
- 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3：目的地那条分配记录的分配代写单元的诞生代，I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 一个字不改。
- 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G2、G9：三层与选段资格「`R` 里至少一个能搬的单元」（攻方 K-b），区域扫描的分类。
- 判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S3（判类优先级、隔离槽一律判隔离，主 agent 定）、S4（扫描被拒的段不再选、`plan_batch` 交空表分两种，主 agent 定）；greedy 同分取起点槽号最小是里程碑四第 2 批 GC 纯函数实现员的读法、主 agent 认（`records/2026-10-13-里程碑四预开发.md`「三　交回与判断」）。

**欠**：C122（分配记录不点删之后的条目数口径）；C194（代际增量 scrub 看不见被搬过的副本）；C146（无空段时的回落政策全仓无定义）；C195（销毁级联时 prev 传递只写在实验的模型修订记录里）；C196（中央映射树的根住哪全仓无条款）；C197（迁移的三笔写是不是一个事务没写下来）。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:156-179`（整段抄，未转述）**

```markdown
#### 已定项 6：走没走整理的运行时观测点

**定案**：观测量六个——碎片度 `F = runs / L` 的 runs、全空聚簇段数、回落计数（分配走不进聚簇段的次数）、不可回收字节、整理进度、一批整理的搬迁量。**触发用的三个量持久化，不靠挂载后重算**：

| 量 | 住哪 | 为什么是那里 |
|---|---|---|
| runs、全空聚簇段数 | 记账树（D5（快照 / 空间记账机制） 已定项 3 的载体：「(统计量, 维度元组, 代) → 完整值」） | 幂等完整值、加统计量不改格式；D5（快照 / 空间记账机制） 已定项 4 的统计量清单因此从九个变成十一个，走它自陈的重开条款，那次重开已经执行 |
| 不可回收字节 | 记账树 | 本来就是 D5（快照 / 空间记账机制） 已定项 4 的第 3 统计量 |
| 整理进度 | 不存：删意图前对整个 `R` 全量核目标态，条目与分配记录本身就是进度 | 攻方 F4：续做点随现行回退会越过被复活的落点 |
| 回落计数、一批整理的搬迁量 | 运行时计数器 | 只供观测，不进判定，丢了不影响 |

不持久化则挂载后触发判据要么没有输入（盲跑），要么全扫一遍才有（O(容量)，撞 `.claude/rules/fs-design.md` 第一格）。改新池新建文件的字节：是——新池新建文件多写两条记账条目，[layout/01-first-txn.md](../layout/01-first-txn.md) 第五节加一行。

**射程**：管这六个量各住哪、哪几个持久化，不管阈值数值（归已定项 1）。runs 与全空聚簇段数**都是带设备维的物理量**：runs 的口径是单元区空闲槽的连续段数、逐设备一行（D5（快照 / 空间记账机制） 已定项 4 第 10 项），全空聚簇段数同样逐设备一行。这一项的触发判定引的那些 runs 数按换口径之前那条「按 key 序遍历已分配对象、数物理连续段」量的，两个口径量的不是同一个量、不能直接搬。增量 runs 在意图重做下会漂，持久化把它变成记账树的标准问题（每代一份完整值，形态同 I-3.1（已分配统计对得上）），审计路径是全扫 runs。

**依据**：

- E93（老化下的放置与碎片度）：碎片度按它给的定义可 O(1) 增量维护，且给出了 checker 侧拿来对照的全扫 runs 审计路径。
- 三方论证（原样输出 `research/prompts/d26-open-items-{dossier,forward-answer,attack-answer,local-answer}.md`）：反推腿独立指出整理进度住在意图记录里、触发量不持久化则挂载后盲跑或全扫。
- 用户定案（持久化），原话在变更史。
- 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：logged_ops 的整理意图不存续做点（攻方 F4：续做点随现行回退会越过被复活的落点），删意图前对整个 `R` 全量核目标态。

**欠**：C91（整理意图与增量触发量的重做漂移）；C141（停机判据的输入不持久）；C326（runs 换口径之后 E93 的数还能不能引）；C323（镜像大小全仓没有条款）；C413（碎片度与全空段统计没有不变量）。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:180-189`（整段抄，未转述）**

```markdown
#### 已定项 7：回收分两条线，测试发现盘的 FTL 支持 GC 时交给盘

**定案**：回收（GC）以后是两条线。先做腿一的自建 GC；以后测试发现盘的 FTL 支持 GC 时，绕过代码里的 GC，让盘自己实现 GC，本工程这边只打标记、发指令。回收走哪条线按系统配置里的 feature bit 处理，每块盘自带；用 incompat、compat_ro、compat 三档里的哪一档没定，落未定项 10。GC 的设计里要预留这条分支，这个功能以后一定会加；近期不论证、不实现、不验证。改新池新建文件的字节：否——先做自建 GC、交给盘那条线近期不实现，第一版不写这一位；位定下之后重判。

**射程**：管回收分不分线与按什么选线，不管怎么测盘的 FTL（未定项 8）、走交给盘那条线时腿一里哪几件活停下（未定项 9）、feature bit 用哪一档与发给盘的指令（未定项 10）。**与 D10（随机写 / 碎片化的真答案） 的候选 C 不是一回事**：候选 C 逐字是「NVMe FDP / ZNS 把数据放置与 GC 责任交给主机」，已定项 7 的「交给盘」方向相反——回收留在盘自己的 FTL 里，主机不做；FDP / ZNS 仍是「有就用的加速」，一个字不改。交给盘那条线在 `crates/` 里不存在；自建那条线在 `crates/singlefs-core` 里有意图条目的格式（`logged_operations.rs`）、区域扫描（`region_scan.rs`）、选段与计划（`compaction_plan.rs`，含用户改动的发布之后跑至多一批的触发 `compaction_starts_after_a_user_change`）、发布入口（`transaction.rs` 的 `publish_relocation_batch`）与显式入口 `MountedSession::run_compaction`（`mounted_session.rs`）；批大小条款没定，产品路径在签任何意图之前停下（`CompactionRunStop::BatchSizeNotDecidedByTheClauses`），只供测试的开关给出批大小才推得动。另有：`crates/singlefs-core/src/system_configuration.rs` 里是水位常量 `COMPACTION_WATERMARK_COUNT`、`COMPACTION_WATERMARK_BUILT_IN_DEFAULT`（「整理低水位 / 高水位 / 停止线三条」，第一版恒 0 = 用内置默认，写 24 字节 0），`crates/singlefs-harness/src/mount_transparency.rs` 与 `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` 只核水位的内置默认值 0，`crates/singlefs-core/src/allocator.rs` 的 `reclaim_released_up_to` 只把已释放的落点在可再分配之后放回空闲、不搬活数据，块设备 trait 有 `discard_at`（`crates/singlefs-core/src/block_device.rs`），产品路径上由 `crates/singlefs-core/src/mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole` 在用户改动的发布与整理显式入口之后调用（`discard_at(` 另在 `block_device.rs` 的单测、harness 的录制器与故障注入设备转发里命中）；discard 什么时候发见未定项 9「discard 的时机」一段。

**依据**：无实验：这一项是产品方向的政策判断，用户定近期不论证、不实现、不验证，两条线在代码里都不存在，没有可量的量；要量的那几格（怎么测 FTL、腿一停哪几件、feature bit 档位）各自落在未定项 8 / 9 / 10 上。用户原话与追问的答复在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:190-209`（整段抄，未转述）**

```markdown
#### 已定项 11：碎片的答案押在哪——两条腿叠加，设备接口降为加速

**定案**：**长期运行积累的碎片由本工程自己解决，不押在设备接口上。** 两条腿**叠加**，不是二选一：

| 腿 | 是什么 | 格式级？ |
|---|---|---|
| **一：事后整理** | 常驻、自动、用户无需知道的后台整理，外加本工程自己的放置与回收层；回收那一半分两条线（已定项 7） | 否——落 `.claude/rules/fs-design.md` 第三格（后台、可续做、非决策路径，无戒律） |
| **二：结构抗老化** | 索引结构本身在老化后仍保持 scan 局部性，从源头少产生碎片 | 形态已定为不进格式（已定项 4）⇒ 不卡 D8（核心索引结构） 的节点格式冻结 |

**NVMe FDP / ZNS 降为「有就用的加速」，不再是答案**：把碎片的答案押在设备接口上不可靠——在不支持 FDP / ZNS 的设备上等于没有答案，而 D12（目标介质） 已定不预先封顶介质套数 ⇒ 那不是边角，是主流介质。**叠加是当期定案，不是终局**：两条腿里可以有一条降级，判据用 E10（老化：结构抗老化 vs 事后整理） 自己写死的那一条（「③ 若能达到 ② 的水平，则常驻后台整理可以从承诺降级为优化」），**不许另立判据**；在那之前两条都做。改新池新建文件的字节：否——两条腿都是后台与放置政策，新池新建文件不跑它们。

**射程**：管答案押在哪、两条腿是不是叠加。腿一的形态框架在已定项 12，触发、事务形态、准入、记账交互各在已定项 1 / 2 / 3 / 5，回收那一半在已定项 7；腿二的形态在已定项 4。D10（随机写 / 碎片化的真答案） 那三条候选各自的地位写在 D10（随机写 / 碎片化的真答案） 自己那边，这里不复述。降级判据压在一个未跑的实验上，在它跑出来之前这条「叠加」没有可用的翻面观测。

**依据**：

- E93（老化下的放置与碎片度）：聚簇段政策没有整理喂空段就不自持，邻居重写有天花板而整理随预算持续走低——腿一是必需的，不是可选加速。
- 用户定案（两条腿叠加、FDP / ZNS 降为加速），原话在变更史。

**欠**：C139（整理停摆之后释放也发不出去）。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:210-231`（整段抄，未转述）**

```markdown
#### 已定项 12：腿一的形态由三条已定纪律框死

**定案**：腿一的形态不是开放问题，三条已定纪律合起来就把它定完了：

| 约束 | 出处 |
|---|---|
| 整理属第三格，**无戒律**——可以遍历、可以慢、不参与任何即时判定 | `.claude/rules/fs-design.md` 三格纪律，逐字点名「销毁快照、整理、scrub」 |
| 无界重活走「写意图 → 分批 → 可续做」 | 同一格；由 C2（无界操作做成单事务） 兜底 |
| 搬迁单位是「整个对象重落一组」，不是单条条带 | D2（RAID 条带策略） 已定项 8 的口径，可直接复用 |

**整理与重平衡是两件活，不合并**：整理管碎片，重平衡管盘间均衡（D2（RAID 条带策略） 已写明）。改新池新建文件的字节：否——三条都是既有纪律的复用，不新增任何盘上字段。

**射程**：管腿一整体的形态框架，不管每一格的具体条款——触发在已定项 1、事务形态在已定项 2、准入在已定项 3、记账交互与区域扫描、选段的读法在已定项 5、观测点在已定项 6、回收分线在已定项 7。第三格那条无戒律只覆盖整理这条后台路径；一旦某个量被拖进运行时决策路径（已定项 3 把全空聚簇段数拖进去就是一例），它就不再享受这条豁免。

**依据**：

- E74（分配记录的条目数与整理开销）：整理代价不随盘容量走，但 COW 放大差 370 倍、全看落点聚不聚——「搬迁单位是整个对象重落一组」这条复用口径承重的就是这一格。
- E93（老化下的放置与碎片度）：整理随预算持续走低、不饱和，支持把它做成常驻的后台活而不是一次性工具。
- 用户定案（腿一的形态不必再定），原话在变更史。

**欠**：C2（无界操作做成单事务）。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:232-252`（整段抄，未转述）**

```markdown
#### 已定项 13：后台任务的计算放在哪、怎么开、什么节奏

**定案**：八条。

1. **计算放在哪**：默认 CPU。挂载参数 `background_compute`（取 `cpu` 或 `gpu`，默认 `cpu`；命令行有了之后写成 `--background-compute=gpu`）选 GPU。这一组挂载参数（第 1、2、3 条）只有一份，管全部后台任务，不按任务分设：开了 `gpu`，凡有 GPU 路径的后台任务（GC 搬迁、打包搬迁，以后接上 GPU 路径的别的后台任务）都走 GPU，没有 GPU 路径的照旧在 CPU 上算。它只活在内存里：不写进系统配置、不占 feature bit（D24（后台重活能不能卸给 GPU） 已定项 2 第 1 条），每次挂载各给各的。挂着的时候可以经已挂载会话的控制接口切换（一次切全部后台任务），从下一批开始生效；一批之内不换（`.claude/rules/fs-design.md`「分支变量不许在操作中途改变」）。
2. **用哪几张卡**：挂载参数 `background_gpu_devices` 是一张卡的列表（可以点几张）；不给就用本机全部显卡。
3. **显存上限**：挂载参数 `background_gpu_memory_limit` 是一个带单位的字节数列表，按次序一张卡一个，与参与的卡逐个对应（给了 `background_gpu_devices` 就对它的列表，没给就对本机全部显卡的列表），不补齐：`background_gpu_devices` 点了三张卡，`background_gpu_memory_limit` 要么不给、要么正好给三个数，只给一个也报错；GPU 模式下全部后台任务在那张卡上合占的显存不超过它的那一个数：这个数是上限、不是预留，用得比它少没有关系（卡上此刻空闲的不到这个数也照常跑，只用空闲的那些），任何时刻都不许比它多，攒批、缓冲、暂存区一起算在里面。不给就不设上限，后台任务跑的时候卡上能用多少用多少。GPU 模式把连续几批的计算攒起来一次交给 GPU，攒多少受它管。
4. **配错与 GPU 用不上时报错，不降级**：挂载或切换那一刻，`background_gpu_memory_limit` 给了而个数与参与的卡数不等、某一个数超过那张卡的显存总量、GPU 不在、驱动不对、点名的卡不存在，就拒绝挂载（或拒绝切换）并报错；挂着的时候 GPU 出错，正在算的那一批放弃（意图留在盘上，目标态单调，已定项 2），那个后台任务停下并报错，不换回 CPU；恢复要人切回 `cpu` 或修好 GPU 再切。唯一的例外在第 8 条：连续几轮状态不确定的条目按条交 CPU 判，后台任务照旧在 GPU 上跑。
5. **什么时候起跑**：每次发布之后按已定项 1 的三水位查一次；另有定时兜底，每隔一段时间再查一次，间隔是挂载参数。两种模式起跑规则相同。
6. **起跑之后的节奏**：GC 占盘写带宽有一个上限（挂载参数）；前台有 fsync 在等时不发下一批。
7. **值不值按省下的 CPU 时间算**：GPU 模式的目的是少占主机 CPU 时间，数据量大时跑得更快只是副产物。收益是后台任务少占的主机 CPU 时间（CPU 模式下它的 CPU 时间减 GPU 模式下它的 CPU 时间，后者含准备数据、提交、等结果、两份结果的比对，以及第 8 条交 CPU 判的那几条），挂钟与 CPU 模式持平也做；D24（后台重活能不能卸给 GPU） 已定项 3 那条按挂钟比的判据不管这一项罩着的后台任务。实现之后跑一组对照：同一负载、同一池，CPU 模式与 GPU 模式各跑，报两边的 GC 挂钟、主机 CPU 时间、前台 fsync 延迟，并在几档盘容量上各跑一遍，看收益随容量怎么变。
8. **GPU 的结果怎么复核**：两份独立写的 GPU 核（不共用核函数）对同一批输入各算一次，逐条比两份的结果。两次相同的条目算确认，交 CPU 改位图（CPU 只改位图，不再抽样复核）；两次不同的条目写明状态不确定，这一批不回收、不搬，留到下一轮重算。同一个条目连续 N 轮都不确定，交 CPU 那一份实现单独判一次，按 CPU 的判定处理。每一批报确认、不确定、交 CPU 判的条目各多少，运行时看得出来。只有一张卡时两份核都在这张卡上跑，有几张卡时可以分开跑；不许要求多张卡，一张与几张都要能跑、判定相同。N 与几张卡时怎么分在里程碑四设计轮（七·2）定。

GPU 路径与 CPU 路径是两份实现（D24（后台重活能不能卸给 GPU） 已定项 2 第 3 条）：运行时的复核照第 8 条由两份 GPU 核各算一次比对，CPU 那一份管连续不确定的条目，两份实现在测试里对拍。GPU 路径默认写成 WGSL、经 `wgpu` 起（checker 档的崩溃判器已经这样用，`Cargo.lock` 里是 `wgpu 30.0.1`，不用另装 CUDA 工具链）；有更好的做法可以在设计轮比过之后换。改新池新建文件的字节：否——八条全是挂载参数与后台行为，不写进盘。

**射程**：管开关、选卡、显存、出错处理、起跑与节奏的形式。第 1 到 4 条与第 7、8 条管全部有 GPU 路径的后台任务，今天是 GC 搬迁与打包搬迁；第 5、6 条只管 GC，打包的起跑与限速在 D27（小数据打包容器） 已定项 6。D24（后台重活能不能卸给 GPU） 的三个候选仍暂缓（D24（后台重活能不能卸给 GPU） 已定项 1），重开并接上 GPU 路径时进同一组开关，值不值照哪条判据重开时定。不管：GPU 上算 GC 的哪几段（里程碑四设计轮定）、算打包的哪几段（里程碑七设计轮定）；定时兜底的间隔、写带宽上限的默认值（等 C412（整理的挂钟代价没量过） 与 E10（老化：结构抗老化 vs 事后整理））。⚠️ 第 4 条运行中出错就停 GC：停着的时候空间只减不增，与 D3（空间分配） 已定项 14「后台整理常驻、自动」并存的读法是这只在用户显式开了 GPU 时才会发生，默认 `cpu` 不受影响。⚠️ 第 8 条的两份核抓得到只有一份出的错（一份核写错、没有 ECC 时的位翻转、不确定的竞争），抓不到两份一起错的：两份共用的那几样（同一个驱动与着色器编译器、同一份输入表的准备、对规则的同一种误读）出错时两份照样一致，那一类靠 GPU 与 CPU 两份实现在测试里的对拍和崩溃点重放抓。

**依据**：无实验：这一项是开关与运行政策的用户定案，数值不在这里（等 C412（整理的挂钟代价没量过） 与 E10（老化：结构抗老化 vs 事后整理））。用户定案（2026-10-19 弹窗四问与追加的几句；2026-10-20 参数统一管全部后台任务、收益口径扩到全部后台任务；2026-10-21 两份 GPU 核各算一次比对、不确定的留到下一轮、连续不确定的交 CPU 判），原话在变更史。

**欠**：C412（整理的挂钟代价没量过）。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:263-272`（整段抄，未转述）**

```markdown
#### 未定项 8：回收两条线的切换判据

**题面**：怎么测盘的 FTL 支不支持 GC、在 mkfs 还是挂载时测、测的结果存不存盘。用户答判据的输入是「如果测试发现硬盘的ftl支持gc」，测什么、何时测没说，并定近期不论证（原话与答复全文在变更史）。

**这条分支受 `.claude/rules/fs-design.md`「五条硬要求」管**，那一节逐条是：切换判据必须显式、可计算、写下来；每条分支必须能被测试强制进入；每条分支单独验，包括崩溃点重放；分支必须可观测；分支变量不许在操作中途改变。⇒ 「盘的 FTL 支不支持 GC」这个测试结果不能在一次操作中途变。

**先做哪一条已定**：先做自建那条，交给盘那条以后加（已定项 7）；这一项要在交给盘那条线接进来之前答。

**第一期**：D12（目标介质） 已定项 5 定第一条布局线是普通 SSD，口径逐字「不要求 FDP / ZNS / 任何特殊设备接口」；普通 SSD 落哪条线取决于这一项。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:273-299`（整段抄，未转述）**

```markdown
#### 未定项 9：回收两条线的射程

**题面**：走「交给盘」那条线时，腿一里哪几件活停下。用户答「我们自己就不实现gc了。gc部分交给ftl」「我们标记，发指令就好了」，腿一里哪几件算「gc 部分」没说，并定近期不论证。

**腿一挂着的活**（按已定项索引表与连锁表列，不保证列全）：

| 活 | 出处 | 走交给盘那条线时停不停 |
|---|---|---|
| 整理：搬活数据、产出全空聚簇段 | 已定项 1（停机取 `R` 本地谓词「删意图那一刻 `R` 按记录全空」） | 未定 |
| 部分条带的惰性重落 | D2（RAID 条带策略） 已定项 11 | 未定 |

⚠️ **按最宽的读法（腿一整个停）会撞已定项 3 的依据**：那条链从「整理停摆」一路推到「释放这个事务本身发不出去」，并把 `(全空聚簇段数 ≥ N₀)` 做成了分配准入的第二个合取式；它数的是提交内生块散布之后的元数据块数与事务占用，不经过盘的 FTL，盘的 FTL 再好也不改它——这一句是主 agent 读条款的推论，未经三方。⇒ 这一项要连同已定项 3 与 D3（空间分配） 已定项 5（提交内生块的落点）一起答。

**discard 的时机**：盘的 FTL 只有收到 discard 才知道哪些块不用了。discard 什么时候发已定（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G5）：发布返回之后，「可再分配」谓词为真（已回收、没被抬 F 扣住、没被隔离、F 已生效）的槽按 64 槽对齐的整段发；块设备 trait 加一个 `discard_at`（默认实现返回「不支持」并计数）。这一项未定的只剩交给盘那条线的射程本身。

**discard 的第一版取法**（`crates/singlefs-core/src/block_device.rs` 现行）：

| 问 | 第一版 |
|---|---|
| `discard_at` 交回什么 | `Applied` 或 `Unsupported`；`Unsupported` 一个字节不动、不进录制流，只计入 `unsupported_calls` |
| 长度 0 | 交 `Applied`，不计入 `applied_calls`（判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S5，主 agent 定：长度 0 没有动作） |
| 后端 | 文件后端复用打洞，`fallocate` 报不支持打洞时交 `Unsupported`；`O_DIRECT` 后端第一版交 `Unsupported` |
| 崩溃点重放里取几态 | 照 D13（验证路线） 已定项 4 的字面：长于一个扇区、罩住的范围里原来有东西的取第三态（判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S6，主 agent 定） |
| 产品路径的调用者 | `mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole`，用户改动的发布与整理显式入口之后各调一次（装着还没清扫孤儿头的段不发） |

还开着两问，`.claude/kb/milestone/04-fourth-txn.md` 二·2 在产品路径上发 discard 时答：「一次 discard 段内部分落地」是不是第三态的另一种形态；宿主重跑（`SparseBlockDevice` 交 `Applied`）与虚机（`O_DIRECT` 后端交 `Unsupported`）的结局要不要答同一个。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:300-307`（整段抄，未转述）**

```markdown
#### 未定项 10：回收线的 feature bit 用哪一档、发给盘的指令是什么

**题面**：回收线的 feature bit 用 incompat（不认识不许挂）、compat_ro（不认识只读挂）、compat（不认识随便）哪一档，另占一位还是跟着 D12（目标介质） 已定项 1 的布局位走；发给盘的是哪几种指令。用户定近期不论证。

**档位怎么判**：`.claude/rules/fs-design.md`「一条格式分支值不值得，看它能不能把『用错了』变成『挂不上』」。主 agent 提的异议（用户据此选了「feature bit，档位留未定」）：两条线写到盘上的字节若相同、只差主机搬不搬数据与发不发 discard，不认识这一位的实现本来能正确读写这块盘，incompat 挡住它换不来安全性，该用 compat；交给盘那条线若让盘上少了自建线依赖的东西（例如已定项 3 准入要的全空聚簇段），不认识这一位的实现照自建线去写会出事，才轮到 compat_ro 或 incompat。⇒ 档位跟着未定项 9 的答案走。feature bit 在 mkfs 时写死，满足未定项 8 引的「分支变量不许在操作中途改变」。

**实现的样子**：系统配置自举头有 incompat / compat_ro / compat 三张位图、各 256 位（[layout/01-first-txn.md](../layout/01-first-txn.md)「一、mkfs 已经种下的」），第一版只有 incompat 位 0 = 1（第一条纯 SSD 布局线）；`crates/singlefs-core/src/system_configuration.rs` 只查 incompat 位图（`incompat_bits_are_mountable`），`grep -rn "compat_ro" crates --include=*.rs` 零命中。主 agent 的读法（用户没说，不当定案）：发给盘的指令里至少有 discard，理由见未定项 9 的「discard 的时机」一段。

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:308-322`（整段抄，未转述）**

```markdown
### 与其他决策的连锁

| 决策 | 连锁点 |
|---|---|
| D10（随机写 / 碎片化的真答案） | D26（后台整理与放置回收） 已定项 11 就是 D10（随机写 / 碎片化的真答案） 已定项 1 的答案：两条腿叠加、候选 C 降为加速。回收分两条线之后，自建那层不再对所有介质一视同仁；候选 C 把放置与 GC 交给主机，与已定项 7 的「交给盘」方向相反 |
| D3（空间分配） | 第 2 条那句「常驻、自动、用户无需知道的后台整理」的承诺，承接方就是腿一；已定项 3 要重开的是 D3（空间分配） 已定项 2 的「解开自己需要多少」算法，不是那条原则本身 |
| D8（核心索引结构） | 已定项 4 定不进格式 ⇒ 腿二不卡节点格式冻结；整理的无界总量走 D8（核心索引结构） 已定项 5 的意图机制 |
| D15（格式冻结政策） | 索引节点内部布局是它的第 4 层，已定项 4 判不动那一层 |
| D21（权威态与派生态的分界） | 已定项 1 已判权威态：反向映射主体在单元里，整理不等索引重建 |
| D9（加密） | 已定项 5 定了无密钥侧只做 scrub、不搬不回收 ⇒ 锁盘期整理整个停摆。这是已定案接受的边界，不是 D26（后台整理与放置回收） 的缺口 |
| D2（RAID 条带策略） | 已定项 8 的搬迁单位口径可复用（D26（后台整理与放置回收） 已定项 12）；已定项 11 把「部分条带的惰性重落」挂上腿一——这是腿一第一个具体职责 |
| D12（目标介质） | 已定项 5 定第一条布局线是纯 SSD ⇒ 第一期没有 FDP / ZNS 可依赖；「第一期的主战场」只在普通 SSD 落自建那条线时成立，落哪条线看未定项 8 |
| D5（快照 / 空间记账机制） | 已定项 5 的「birth 不改」是对它 `birth(b)` 定义句的收口；已定项 6 往它已定项 4 的统计量清单加两个（runs、全空聚簇段数），九个变十一个 |
| [layout/01-first-txn.md](../layout/01-first-txn.md) | 已定项 6 让新池新建文件多写两条记账条目，第五节加一行 |

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:1-6`（整段抄，未转述）**

```markdown
## D24 后台重活能不能卸给 GPU —— 已定

**中文** · [English](24-offloading-heavy-background-work-to-gpu.en.md) · [日本語](24-バックグラウンド重処理の-GPU-オフロード要否.ja.md)

D24（后台重活能不能卸给 GPU） 管一件事：三个后台重活（批量压缩、全盘 scrub、大规模纠删码重建）要不要卸给 GPU。后台整理（GC 搬迁与打包搬迁）的计算放 GPU 不在这三个里：用户定做，开关与运行政策在 D26（后台整理与放置回收） 已定项 13，一组参数管全部后台任务，已定项 2 的三条硬约束照样管它。不管的：CPU 侧算法怎么选在 D9（加密） 已定项 3；scrub 本身的形态在 D9（加密） 已定项 5；后台整理与回收的队列与预算在 D26（后台整理与放置回收）；三格划分本身是 `.claude/rules/fs-design.md` 的记账纪律，D24（后台重活能不能卸给 GPU） 只引用它。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:17-29`（整段抄，未转述）**

```markdown
#### 已定项 1：暂缓，不做

**定案**：**暂缓，不做**——三个候选场景（批量压缩、全盘 scrub、大规模纠删码重建）都不排期。**暂缓不等于否掉**：算力受限那一格（批量压缩）仍然开着，只是不排期。**GPU 这条线不再继续测**，D24（后台重活能不能卸给 GPU） 停在「方向存疑、两侧都没坐实」这个状态——GPU 侧还缺一个真的 AES-GCM 核函数（要 `nvcc`）。**不要把「GPU 赢」当成已定结论去支撑别的决策。**

**射程**：承重的只有「非必需」这一条——三个候选场景全在 `.claude/rules/fs-design.md` 第三格（后台、可续做、非决策路径），不做它不影响任何硬承诺，不像 D1（数据可移动性 / 反向索引） 的缩容、D3（空间分配） 的常驻整理那样是承诺过的能力。**实测那一格不在候选清单里**：判掉的是「纯扫描」，而三个候选的 CPU 侧都要在扫描之上再算，所以标题里那句「一格实测为负收益」要读成「一个与候选无关的格」，不是「候选被实测判负」——那一格的数与判法在已定项 3。

**依据**：

- 用户定案两次（这件事非必需；GPU 这条线不再继续测），原话在变更史。
- 无实验：这条暂缓承重的只有用户定案，而实测判掉的那一格是纯扫描、不在候选清单里，那一格记在已定项 3；候选三格里两格无数据（纠删码重建与批量压缩），没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:30-46`（整段抄，未转述）**

```markdown
#### 已定项 2：三条硬约束

**定案**：三条，不满足就不许做——

1. **绝不能成为必需。** 文件系统必须在没有 GPU 时完整工作、且语义完全相同。⇒ **GPU 是可选加速器，一个比特都不许进格式。**
2. **不许进前台路径。** PCIe 往返是几十微秒量级，前台读写延迟吃不起这个；它只能服务已经攒批的后台重活。
3. **它是一个新的失败域，卸出去的结果必须复核。** GPU 算错（驱动 bug、显存 ECC 缺失、算法实现分歧）不能变成静默的数据损坏 ⇒ 与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验的两条路径不许共享同一段代码」同构：**GPU 路径与 CPU 路径必须是两份实现**，两份在测试里对拍。运行时怎么复核由用到 GPU 的那条决策定：后台任务照 D26（后台整理与放置回收） 已定项 13 第 8 条，两份独立的 GPU 核各算一次比对，确认的交 CPU 改位图，连续几轮不确定的条目交 CPU 那一份判。

**射程**：第 1 条管的是格式，第 2 条管的是路径，第 3 条管的是验证，三条各自独立，满足不了任何一条就不许做。它们不定「值不值得做」，那是已定项 1 与已定项 3。第 3 条要的两份实现一份都没有（D24（后台重活能不能卸给 GPU） 暂缓，不排期）。

**依据**：

- 用户定案（GPU 是可选加速器；2026-10-21 运行时的复核改成两份独立的 GPU 核各算一次比对、连续不确定的条目交 CPU 判），原话在变更史。
- 无实验：三条都是政策与验证纪律的直接推论，没有可量的量；第 2 条引的 PCIe 往返量级是公开常识，本工程没测过前台路径上的 GPU 往返。

**欠**：无。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:47-59`（整段抄，未转述）**

```markdown
#### 已定项 3：判据取什么形式

**定案**：判据是 `CPU 16 线程耗时 ÷ (传输 + 计算 + 传回) 耗时`，小于 1 就否掉那一格。代进去的 CPU 侧数必须是**多核饱和吞吐**，不是「单核 × 核数」：线性外推与实测在 AES-GCM 上差近 4 倍（外推约 61 GB/s，实测饱和 17.4 GB/s）。GPU 侧的两个地板是**单程 56.69 GB/s**（锁页 H2D，≥64 MiB）与**来回 28.5 GB/s**（H2D + D2H，回传全部数据的场景走这一行）；CPU 侧的上限是**内存带宽墙 64.91 GB/s**。⇒ **GPU 只在「CPU 那侧的实际吞吐掉到传输地板以下」时才有机会。**

**射程**：⚠️ **「瓶颈是不是 CPU」不等价于「这个任务有没有硬件指令」**——有 AES-NI 的加密照样在 17.4 GB/s 就饱和，瓶颈仍然在 CPU 侧，**有硬件指令 ≠ 快到 GPU 追不上**。⚠️ **「CPU 够快所以 GPU 无用」这个前提在本工程不成立，不许拿它支撑任何结论**：它来自单核乘核数的线性外推，而多核实测饱和在 17.4 GB/s，差近 4 倍且方向相反——17.4 低于 GPU 的两个地板。代进去之后四个候选格各是什么：scrub（校验和 / MAC）有 AES-NI 时 CPU 侧多核饱和 17.4 GB/s（8 线程，16/32 线程 16.3/16.5；单核 3656.13 MiB/s）⇒ 按判据这一格 GPU 赢，⚠️ **两侧都没坐实，不许当已定结论用**（GPU 侧缺真的 AES-GCM 核函数）；无 AES-NI 的机器上单核 244.74 MiB/s ⇒ 16 核约 4.1 GB/s，与高端 NVMe 阵列同量级，这一档 GPU 可能有用，多核未测、饱和值只会更低；EC 重建无数据（Reed-Solomon over GF(2⁸)，重建要读全部幸存盘并算 parity，计算量随盘数 × 容量线性，这一格计算确实重）；批量压缩无数据（压缩比与吞吐的取舍，GPU 压缩库存在）。这两个数（17.4 与 244.74）来自 E6（加密算法选型），不是 D24（后台重活能不能卸给 GPU） 自己量的。⚠️ **不能读成「GPU 无用」**：计算段那一段的推论只对**带宽受限**的核函数成立，**压缩是算力受限的**，它的计算段可能远不止那一成，那一格仍然开着且仍要等 `nvcc`。⚠️ **还有一层压在上面**：三个候选场景全是盘限的后台活，盘带宽比 64.91 与 28.5 都低得多 ⇒ 整件事若是盘限的，CPU 本来就在空转，卸给 GPU 省不下任何东西——**这一层是推理不是实测**（本工程没有阵列）。⚠️ **一个未验的逃生口**：GPUDirect Storage 让 NVMe 直接 DMA 进显存、绕开主存与 PCIe 那一跳，**它会推翻「GPU 吃数据 56.69 GB/s」这个传输地板**，没测过，不许当它不存在。要定 D24（后台重活能不能卸给 GPU），先要量 EC 重建与批量压缩这两格的 CPU 上界，以及「PCIe 传输 + 核函数 + 传回」这条链路相对纯 CPU 的净收益——**数据要过一遍 PCIe，那笔搬运成本可能吃掉全部计算收益**。**本机可测**：NVIDIA RTX 5090（32 GB）+ RTX 5060 Ti（16 GB），驱动 595.84；`nvcc` 没有装（`command -v nvcc` 零命中，`/usr/local/cuda*/bin/nvcc` 也没有），那是 G24.3（计算段占比） 的前置。

**依据**：

- E21（GPU 卸载的净收益）：三条路径的吞吐（CPU 吃数据 64.91 GB/s、GPU 单程 56.69 GB/s、GPU 来回 28.5 GB/s）把判据的两个阈值钉住——GPU 光把数据搞进去就已经比 CPU 读它慢 13%，而这还没开始算；计算段用带宽受限算子给上界，显存内 1041 GB/s 而端到端（含传输）只有 50.3 GB/s ⇒ 传输占 90.3%、计算段几乎免费，对纯扫描那一格净收益 0.78×，小于 1 ⇒ GPU 输。**决定 GPU 赢不赢的是 CPU 那侧有没有对应的硬件指令，不是 GPU 有多快。**
- 用户定案（判据的正确形式是多核饱和吞吐），原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:60-83`（整段抄，未转述）**

```markdown
#### 已定项 4：重开判据

**定案**：说不出什么现象会让它重开，「暂缓」就是无限期搁置。四个闸——

<!-- doc-lint:registry name-col=2 -->

| 闸 | 简称 | 判什么 | 测得了吗 |
|---|---|---|---|
| G24.1 | 缺硬件指令 | 目标平台的 CPU **缺**对应硬件指令，使该任务的 CPU 侧吞吐低于 **50.3 GB/s**（单程场景）或 **28.5 GB/s**（要回传的场景） | 能，量一下那个任务在目标 CPU 上的吞吐即可 |
| G24.2 | 不是盘限 | 该任务**不是盘限**的——即 CPU 侧吞吐低于盘带宽，CPU 真的成了瓶颈 | 需要阵列，本工程没有 |
| G24.3 | 计算段占比 | 压缩那一格：真核函数的计算段占比 | **不能**，本机无 `nvcc`，前置是 CUDA 工具链 |
| G24.4 | GPUDirect 可用 | GPUDirect Storage 可用——它绕开主存与 PCIe 那一跳，会推翻「GPU 吃数据 56.69 GB/s」这个传输地板 | 未验 |

⚠️ **G24.1（缺硬件指令） 与 G24.2（不是盘限） 必须同时成立。** 只满足 G24.1（缺硬件指令）（CPU 没指令所以慢）而 G24.2（不是盘限） 不成立（整件事是盘限的），那 CPU 慢也无所谓——它本来就在等盘。

**射程**：四个闸是重开的充分条件，不是排期的判据；G24.3（计算段占比） 与 G24.4（GPUDirect 可用） 测不了，前置分别是 CUDA 工具链与 GPUDirect Storage，G24.2（不是盘限） 要阵列。已定项 4 定的是「什么现象会让它重开」，不定重开之后按什么次序做。

**依据**：

- E21（GPU 卸载的净收益）：G24.1（缺硬件指令） 的两个阈值（50.3 GB/s 单程、28.5 GB/s 来回）与 G24.4（GPUDirect 可用） 要推翻的那个地板（56.69 GB/s）都是它量出来的；判据 1 的形式也出自它。
- 用户定案（说不出什么现象会让它重开，暂缓就是无限期搁置），原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:84-96`（整段抄，未转述）**

```markdown
#### 已定项 5：合法性的来源

**定案**：三个候选场景全部落在 `.claude/rules/fs-design.md` 记账纪律的**第三格**（后台、可续做、非决策路径，明写「无戒律」）⇒ 引入 GPU 不与任何已定纪律冲突。**三格划分是 D24（后台重活能不能卸给 GPU） 合法性的全部依据——一旦某个场景挪进第一格（运行时决策路径），D24（后台重活能不能卸给 GPU） 对它当场失效。**

**射程**：与别的决策的连锁三条：D2（RAID 条带策略） 那边，EC 重建是 D24（后台重活能不能卸给 GPU） 最可能有收益的一格；D3（空间分配） 那边，常驻后台整理属第三格，是 GPU 的合法作用域；D9（加密） 那边，scrub 的校验和与 MAC 计算量由它决定，**无 AES-NI 那一档是 GPU 唯一有明确数据支持的入口**。已定项 5 不判这三格值不值得做，那是已定项 1 与已定项 3。

**依据**：

- `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」那张三格表：第三格逐字「后台、可续做、非决策路径（销毁快照、整理、scrub）」，明写「无戒律」。
- 无实验：一个场景落在哪一格是按那条纪律自己的判据判的（「被问到那一刻有没有答案」），没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/experiments/10-老化结构抗老化vs事后整理.md:1-24`（整段抄，未转述）**

```markdown
## E10 老化：结构抗老化 vs 事后整理 —— 部分已跑（第一段 21 片跑了 4 片，读数只当线索；剩 17 片停下，登记的三处要不要修订待定）

**中文** · [English](10-aging-structural-resistance-vs-post-hoc-compaction.en.md) · [日本語](10-老化-構造的抗老化-vs-事後デフラグ.ja.md)

**背景**：D3（空间分配） 承诺了常驻后台整理，理由是不分类导致碎片压力集中。
FAST 2017 的实测（见 [prior-art.md](../prior-art.md) 7.2）给出另一条路——
**老化可以被索引结构本身吸收掉**。两条路是替代还是叠加，决定 D3（空间分配） 已定项 14 的分量，
也是 D10（随机写 / 碎片化的真答案） 候选 B 的判据。

**做法**：照 FAST 2017 的方法论，用**连续 git checkout Linux 内核源码**做老化负载，
每隔固定轮数量一次遍历带宽与碎片度。三组对照：
① 关掉后台整理 ② 开着后台整理 ③ 关掉后台整理但索引采用抗老化形态。

**判据**：
- ③ 若能达到 ② 的水平，则 D3（空间分配） 的「常驻后台整理」可以从「承诺」降级为「优化」。
- ② 相对 ①的收益要能量出来，否则 D3（空间分配） 第 2 条那个承诺没有数据支撑。

**怎样才算失败**：① 在老化后没有可测的退化，说明老化负载不够狠或数据集太小，
**整轮作废**——老化实验最容易骗过自己的地方就是「没测出退化」被当成「不会老化」。

**前置**：文件系统、后台整理实现、一个碎片度测量工具。

**口径与产物**：第一次跑照跑前登记 `research/prompts/e10-r1-prereg.md`（2026-10-13 写，问题与判据以它为准，不是上面那套三组对照）；装置 `scripts/experiment-apparatus/e10_aging_compaction.rs`，变异表 `scripts/mutation/tables/e10_aging_compaction.tsv`（21 行）。第一段 21 片跑了 4 片，原始输出在 `research/results/e10-aging-compaction-2026-10-13/` 下：`main-s4.out`、`disk_1gib-s4.out`、`disk_1gib-s8.out`、`disk_1gib-s16.out`；剩下 17 片停下，读数只当线索。

```

**出处 `.claude/kb/experiments/10-老化结构抗老化vs事后整理.md:25-31`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 14 | 备料 | 2026-10-21 不受影响：标题状态与索引行同步成部分已跑，备料关系与结论不变（读数只当线索）；此前 2026-10-17 不受影响：这一轮只把标题括注与索引行同步，结论、数与产物不变；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D10（随机写 / 碎片化的真答案） 已定项 1 | 备料 | 2026-10-21 不受影响：标题状态与索引行同步成部分已跑，备料关系与结论不变（读数只当线索）；此前 2026-10-17 不受影响：这一轮只把标题括注与索引行同步，结论、数与产物不变；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |

```

**出处 `.claude/kb/decisions/13-验证路线.md:1-6`（整段抄，未转述）**

```markdown
## D13 验证路线 —— 已定

**中文** · [English](13-verification-route.en.md) · [日本語](13-検証ルート.ja.md)

D13（验证路线） 管本工程拿什么验正确性：三类现成办法全做——自写参照实现做差分对拍、有界穷举崩溃测试、crash refinement 形式验证（调查见 [prior-art.md](../prior-art.md) 7.4）——并且不照抄，针对本工程特有的四样（COW 每事务发新根、Merkle 自验证、整卷 AEAD 的 nonce 唯一性、没有参照实现）自己设计验证手段。它定 oracle 怎么分、自研哪两样、崩溃点重放怎么分层与抽样、崩溃状态集合怎么定义、checker 与实现共享什么。不管的：每条不变量写什么在 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)；三样验证手段怎么落地、谁挡着谁在 [verification-build.md](../verification-build.md)；提交步骤登记在哪、结构等价类怎么分、有几个，归 D17（实现分层与第三方管道） 已定项 2。

```

**出处 `.claude/kb/decisions/13-验证路线.md:29-90`（整段抄，未转述）**

```markdown
#### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」

**定案**：crash refinement 的第一个验证对象是恢复函数 `Recover` 的关系性质，最小封闭单元是「一次发布 + 一次崩溃 + 一次恢复」。四个里程碑的主语 `Recover` 指 `crates/singlefs-core` 里以 `verify_named_units = true` 调 `replay_journal` 的三处（`recover` 的 `Consult` 策略、可写挂载读阶段、只读挂载），三处的 `records` 都取 `scan_journal` 的输出；`Ignore` 与 `ConsultWithoutNamedVerification` 是测试与对照用的臂，写进可信部分；`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` 不是产品调用方。四个里程碑依次是：

| 里程碑 | 义务 | 落点 |
|---|---|---|
| **M1 全性** | `Recover` 在 D13（验证路线） 已定项 4 枚举的**每一个**崩溃状态上有定义、终止、输出一个能过 I-7.1（根槽有效集非空） 与 I-8.3（重放前缀严格连续） 的镜像 | 脚手架。**完成判据不是「证出来了」，是「M2 能在它上面写出来」**——崩溃点重放本就全量枚举（[verification-build.md](../verification-build.md)「崩溃点重放不抽样、全量枚举」），恢复卡住当场暴露，单独证它没有边际价值。取前缀、择根、扫环三截已进 Verus，都在 `crates/singlefs-replay-prefix/src/lib.rs`：取前缀核 `replay_core`（终止与不溢出在核内证到，循环都带 `decreases`；`requires` 两条——记录表按 (实例代号, 计数器) 升序、计数器有后继——由胶水 `replay_journal` 里的断言兜），择根核 `choose_root_slot`（自证过的槽里取 (`checkpoint_txg`, 实例代号) 最大、键相同取槽序最前），扫环核 `first_copy_of_each_record_key`（同一 (实例代号, 计数器) 先到先得），择根与取前缀的合判写成推论 `corollary_recovery_replays_from_the_newest_self_verified_root`；门禁 checker-tier-formal-verification 判证明，登记数 56 verified（`.claude/gate.d/verus-pins.tsv`）。E165（恢复重放有界穷举与变异杀伤界） 当轻量工具，它第 3 次重跑第二段的择根合判族在内存盘上调真 `choose_root`、`scan_journal`、`replay_journal`。择根序的四个改坏变体在合法历史 K 族里只有「取第一个有效者」红，另三种（比较不看实例代号、txg 平局取低、实例代号为主）由 Verus 必红世界（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 择根那一节）与对抗根环 KA 罩（第 3 次重跑第三段）。核之外：择系统配置（`choose_system_configuration`）与两截胶水（择根的读槽、自证校验和、fsid、区域到盘的映射、槽的遍历次序；扫环的偏移枚举、提示、解析与 CRC、盘的遍历次序）不在 Verus 里，崩溃点重放与崩溃放量的 CPU 判器在每个崩溃状态上调 `recover` 罩它们（GPU 判器用 WGSL 另写一份 `replay_journal`，验不到 core） |
| **M2 前缀不切发布** | **任一 `jsn` 连续前缀，其重放结果等于某个整数个已发布的 checkpoint 的结果——不存在被前缀切开的发布；前缀停在一次发布中间时，那次发布整体不施加**（D16（发布语义） 已定项 4「一次发布整体施加或整体不施加」）。分两层。**句法层**（对任意自证过的记录表，无前提）：施加的记录是若干整次发布，在飞上限与提交标记逐条判过，施加的计数器从链首起逐一连续（锚点读得出时从锚点加一起；读不出时首条 `checkpoint_txg` = 所选根 + 1、序号 1）；反向链与点名单元是读盘回调的结果，写成「被施加的每条记录，回调交回了真」。**语义层**（前提：盘上每条记录出自这一版的合法写侧，即 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族；同一 (实例代号, 计数器) 在全部时间线上只对应一份内容）：「结果」是根的投影加盘，不是整条根——返回的根的 (实例代号, `checkpoint_txg`) 与新根段四个字段等于所选根那条时间线上的第 j 版（j 不早于所选根），其余字段等于所选根，卸载记号除外（由记录重建的根取「不是卸载那一串写下的」）；盘上 (所选根, j] 每一版的记录读得出、点名单元两份都验过、记录条数之和不超在飞上限 | 落点是 I-8.3（重放前缀严格连续）——它的判据补句逐字就是这一条（「前缀不得停在一次发布的中间——一次发布整体施加或整体不施加」），不要在 [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) 里另立一条。句法层在 Verus 核 `replay_core` 的 `ensures`（`replay_follows_the_prefix_rules`）里，必红世界登记在 `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv`，其中「改判定」行只改可执行判定、不碰 spec；语义层靠 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族在界内判，Verus 不证。核之外：胶水（摊表、包回调、拼报告与新根）由 E165（恢复重放有界穷举与变异杀伤界） 直接调真 `replay_journal`、harness 用例与 `crates/mutations.tsv` 的变异行罩，拼根里投影之外的字段（卸载记号、实例表指针、分配记录树根）没有变异行、E165（恢复重放有界穷举与变异杀伤界） 的生成器把它们写成定值；两个读盘回调（反向链、点名单元闭包）由 E165（恢复重放有界穷举与变异杀伤界） 里真回调、内存盘的读数、`crates/mutations.tsv` 的变异行与只读挂载几种形状的读盘次数用例罩 |
| **M3 幂等与单调** | **幂等**：`Recover(Recover(c)) = Recover(c)`，两读都证——记录级（同一条记录施加两次与施加一次，得到的根相同，I-8.4（重放幂等） 的字面）与不动点（把重建出的根当所选根、同一张记录表、同一个读者、同一个开关再重放一次，返回的根不变；前提是所选根之上的记录不超在飞上限，且记录表满足 I-8.12（同实例记录的 txg 不减）：同一实例里 journal 记录的 txg 随计数器不减）；第二次因所选根 `checkpoint_txg` 顶格报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` 且一条没施加的，算相等。「就是 I-8.4（重放幂等）」不成立：I-8.4（重放幂等） 是记录级一读，不动点是另一条性质。**单调**（M2 的推论，不单独证）：串行提交（已定项 2）、D16（发布语义） 已定项 7 的持久顺序、所选根取根环里最新的有效根之下，同一次运行里 D13（验证路线） 已定项 4 的两个不带故障的崩溃状态 `c ⊆ c'`，`Recover(c)` 落到的版本不晚于 `Recover(c')` 落到的版本，版本按 (`checkpoint_txg`, 实例代号) 比，与 D22（单元原子性怎么合成） 已定项 7 的择新序同一个序；开并行提交或把域放宽到带故障时重开。「施加的事务集包含于」那一读法撤：它对理想恢复也为假 | Verus 核 `replay_core` 的 ensures 证两读：记录级与不动点（不动点的前件含 I-8.12（同实例记录的 txg 不减））；单调写成推论 `corollary_the_landed_version_does_not_go_back`，择根与写侧那两条前提写在它的 requires 里。E165（恢复重放有界穷举与变异杀伤界） 的对抗一族在界内判不动点，违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上（第 3 次重跑第一段）。checker 那一半（I-8.4（重放幂等））已实现，取在崩溃注入的二次崩溃上判、不进池级 checker 的清单：`crates/singlefs-checker-tier/src/crash_injection.rs` 的 `REPLAY_IDEMPOTENCE_INVARIANT` 与 `judge_the_remount_after_a_second_crash`。O2（独立解析器 + checker） 判不了幂等：I-4.5（已停用·需两镜像） 退役理由逐字「幂等要比对两个镜像，单镜像判不了」 |
| **M4 不少施加** | 前提与对象同 M2 的语义层。`Recover` 返回的根恰是 j\*：从所选根起每一版都完整在盘上（记录任一份自证过、点名单元两份都验过）、记录条数之和不超在飞上限的最远一版。不加这一条，M1–M3 全部证出来也排除不了「已确认的完整发布被恢复丢掉」：一条都不施加的恢复过 M1–M3 的全部字面。「不少施加」按「多出违例」读：变体多出违例才算被它挡住 | E165（恢复重放有界穷举与变异杀伤界） 的最远一版判据在界内判；Verus 的必红世界里「第一条水位之上的记录就停、一条都不施加」那一行红；M4 自己不单独证 |

**挂载透明**（独立的第四条性质，不是 M3 的第三读）：在崩溃镜像上只读恢复一次得 R1；在同一份镜像上做一个动作——只挂载（取号、写行、暖机、挂载续做与挂载自己推的抬 F），或挂载加一次发布——之后再只读恢复一次得 R2，逐字段比 R1 与 R2。比的那一版：只挂载比恢复落到的那一版，挂载加一次发布比挂载后那次发布之前的那一版。挂载途中三个相位的二次崩溃镜像 img2 上也判：C(recover(img2)) = C(v1)（v1 是 R1 落到的那一版，C 取 POSIX 内容与存在性），内容与字段两半都判；字段一半里，落回 v1 那一版自己的各字段与 R1 相同，落到这次挂载写的某一版的照逐字段表判，扫描报告不判，抬 F 第一个写起 F 不判、在它之前（含暖机）F 照「挂载没推抬 F」那一格判；抬 F 第一个写的位置按挂载交回的抬 F 那几串（准入推的与续做里推的）在挂载写表末尾的位置取，取不出（`NotLocatedInTheMountWrites`）时这一次挂载的二次崩溃整段 F 记不判。逐字段表（R2 对 R1）：

| 字段 | 判法 |
|---|---|
| POSIX 内容与存在性 | 必须相同 |
| 池身份与系统配置的不可变配置（本盘设备号除外） | 必须相同；不可变档 37 行除本盘设备号一律按原字节比 |
| 系统配置的节点大小 | 必须相同 |
| 系统配置的运行配置 36 字节（T_time、T_dirty、三水位） | R2 等于内置默认值（没有挂载选项通道，`to_slot` 总写默认） |
| 卸载记号 | R2 恒为「不是卸载写的」，R1 的记号不进比较 |
| F 与 F 生效值 | 挂载没推抬 F：R2 的 F 生效值与 R2 根里的 F 都等于 R1 的 F 生效值；推了（准入推的与续做里为空间推的都算）：R2 的 F 生效值等于挂载交回的最后一串抬到的 F |
| 实例代号 | max(各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1，两个量都从崩溃镜像读 |
| 实例表行 | 照 D18（块里携带什么信息） 已定项 11 恢复写的行 |
| `checkpoint_txg` | 只判下界：≥ max(根环全部根的 txg, 环里全部自证记录的 `checkpoint_txg`) + 1（D23（journal 的角色与格式） 已定项 14 的下界，允许跳号） |
| 根槽位置 | 不另判，由 `checkpoint_txg` 定 |
| 树 ID 水位 | 挂载不发第一个文件时相同 |
| 树表里 inode 树、extent 树、墓碑、谱系、deleted_inodes 的根指针（后三样是树表条目，种类 9、10、11，不是根记录字段） | 相同；挂载续做过时，按续做之后那一版判：等于挂载交回的续做各步里 txg 不大于 R2 那一版的最近一步的各根指针；树表单元自己的落点不比 |
| 树表种类 6、7、8、12 的根，诞生 txg、头 ID、树 ID、种类 | 必须相同（种类 6、7、8、12 的根恒全零）；诞生 txg 跨根另有 I-9.14（树表条目的诞生 txg 跨根不变），头 ID 另有 I-9.27（树表条目的头 ID 取值），单镜像判 |
| 实例表与树表的单元指针 | 不比字节，与「树表单元自己的落点不比」同 |
| 扫描报告 | 施加 0 条、水位之上 0 条 |
| 回收门槛 t 与钉槽 p（根槽偏移 428..437） | 不比：每次带单元的发布按这个进程的根环与钉现算、零单元发布照抄上一版，挂载做的发布会改它们（与 F、卸载记号同类）；单镜像性质，归池级 checker 的 I-7.18（根记录的回收门槛等于现算的门槛） 与 I-7.19（钉着的槽不住本实例写的根） 判 |
| 根槽偏移 437..466（算法、nonce、MAC） | 必须为 0；单镜像性质，归池级 checker 的 I-2.4（头校验和覆盖范围）判，挂载透明不比 |
| magic、flags 其余位、自证校验和、整槽校验和 | 不比（结构性，读槽时判） |
| 系统配置槽世代号 | 不比，交 I-7.16（系统配置槽世代号与槽号的关系） 在 R2 的镜像上判 |
| 系统配置的 journal_tail | R1 ≤ R2 ≤ R2 环里自证记录的最大计数器，挂载做完取等 |
| 系统配置的 journal 实例代号 | 挂载做完等于 R2 根的实例代号；二次崩溃时取 R1 的实例代号或新实例号两者之一 |
| 分配记录（含根记录的 `mapping_root`、`allocation_record_tree_root` 与树表种类 3、4 的根） | 不比整表：POSIX 内容相同，且池级 checker 对 R2 的镜像判绿；`mapping_root`：R2 全零 ⇔ R1 全零；`allocation_record_tree_root`：R1 带树 ⇒ R2 全零，R1 树表 0 条 ⇒ （R2 非零 ⇔ 新实例号 ≥ 2） |

系统配置的运行量（槽世代号、journal_tail、journal 实例代号）落回 v1 时照它们自己那一行判，不按 v1 相同判。

**射程**：refinement 的边际价值只在崩溃点重放枚举取不到的那几维——载荷字节、记录条数 / 段数 / txg 数（含环回绕）、上一条时间线的残留、槽数与设备数；崩溃点那一维崩溃点重放已全量枚举。「校验和过了」在模型里是一个布尔标志，**不许读成 refinement 验了完整性**——与已定项 14 里 D4（校验和位置） 那一行「不许读成『验了 Merkle』」同形，也与 D13（验证路线） 已定项 4 对 CRC32C 碰撞概率的「知情接受」是同一笔账。D13（验证路线） 已定项 4 是 refinement 左边的 `Crash`（模型层的枚举域），不是右边的 `Spec`，拿它当 spec 会让 refinement 恒真。I-6.5（nonce 水位单调） 的形式化撤下；D18（块里携带什么信息） 已定项 14 定了、且 nonce 分配规则（单调计数器 + 预支水位、水位在使用前持久）被立成一条分项之后，这一格重开——重开时它唯一的真优势是 nonce 对 POSIX 不可见，它的 spec 与 O1（内存模型对拍（HashMap 理想文件系统）） 的 model 结构上不可能同源，不受冲突 2（D13（验证路线） 已定项 13） 影响。

M1–M4 的前提与域：语义层只挂「同一 (实例代号, 计数器) 在全部时间线上只对应一份内容」，由 D23（journal 的角色与格式） 已定项 14「这一版的失败处置」的原样重发担保，根槽写失败推进 `checkpoint_txg` 再发那条路落地时重核；前提域取 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族（合法写侧写出的历史的任一落盘子集，较新的根槽可以读不出）。M2 的「其余字段等于所选根」里，由记录重建的根的卸载记号是 `NotWrittenByTheUnmountSequence`（D22（单元原子性怎么合成） 已定项 7「其余根写 0」），与所选根的记号可以不同，所以写「卸载记号除外」。M3 的单调推论只在串行提交、不带故障的崩溃状态里成立。M4 连同择根才排除「已确认的」完整发布被恢复丢掉，前提要全：不带故障的崩溃状态（已定项 4 的枚举域）、所选根按 D22（单元原子性怎么合成） 已定项 7 的择根取、链不跨实例边界（D23（journal 的角色与格式） 已定项 14 注 1）、新实例本实例写成的根覆盖两块盘之前不让任何 fsync 返回（D16（发布语义） 已定项 8 的暖机）；带故障的域里不排除——在飞上限会截掉记录完整的发布，一块盘整块掉了时点名单元少一份的那一版不施加（C519（丢一整块盘时恢复丢掉刚确认的那一版））。

挂载透明的射程：只管不带故障、可写挂载做成的崩溃状态。可写挂载被拒的状态不判透明，判拒绝本身合不合法：拒绝合法 ⇔ 拒因在 D23（journal 的角色与格式） 已定项 14 那张单子上，且照那一条的判据在崩溃镜像上自己复算成立（「读阶段判完之后系统配置又读不出」这一拒因带补写例外：D23（journal 的角色与格式） 已定项 14 的单子上这一拒只在某块盘一份自证过的系统配置都没有、又过不了 D22（单元原子性怎么合成） 已定项 8 的补写闸时成立；判官只复算「一份都没有」那一半，补写闸五件不复算，补写闸放过的盘面挂上了不拒，归崩溃点重放的观察者判（已定项 6）；判槽自证读池级 checker 的逐盘数，不用核心层的解析，实现在实现批），且盘上逐字节不变，且只读读出在允许集合里；拒因不在单子上的照崩溃注入第二截的写法报红。带故障的状态不在射程：取号那一刻有一槽读不出与 N-配置报成同一个拒因，接进带读故障的档之前要分开（C664（挂载拒绝合法复算把两种拒因报成一个））。逐字段表里树 ID 水位、两棵树的根指针、`checkpoint_txg`、扫描报告四格是推的（三方判决取辩方的式子，没量），F 那一格「推了」的一半没量；根记录的 `mapping_root`、`allocation_record_tree_root`，墓碑、谱系、deleted_inodes 三根指针，算法、nonce、MAC，系统配置的槽世代号与 journal_tail 各归哪一类，照 C655（挂载透明逐字段表有字段没归类）、C687（从 mkfs 水位建树那一版落点判不了的几格） 判决 Y1、Y2 写在逐字段表里。

**依据**：
- E42（一事务几条记录）：为了让「被切开 ⇒ 非法」成为恒等式而钉死了载荷，「被切开却碰巧合法」这一类它结构上看不见——M2 把载荷符号化补的正是这一格。
- 三方：材料 `_d13-item1-background.md`，三条腿的产出 `d13-item1-opus-forward-output.md`、`d13-item1-sonnet-reverse-output.md`、`d13-item1-local-attack-v2-output.md`；三腿一致把原候选排最后，两条云端腿各自收敛到「前缀判定与提交原子性」这道接缝，主 agent 按封闭性把择新与发布原子性收进同一个单元（判决与撤下原候选的三条门槛原文在变更史）。
- E165（恢复重放有界穷举与变异杀伤界）：在界内对真 `replay_journal` 判 M2 的语义层、M4 与 M3 两读，并量变异杀伤界；用户 2026-10-01 定它当轻量工具；C 域合法崩溃状态数按两种切法分写（甲切法是 D16（发布语义） 已定项 7 的屏障位置）；对抗一族里不动点的违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上，谓词成立那一边 0 违例（第 3 次重跑第一段）；择根合判族调真 `choose_root`、`scan_journal`、`replay_journal`，界内、合法、不带故障时已确认的完整发布不丢（第 3 次重跑第二段）；择根序的四个改坏变体在合法历史 K、K9、KW 三族界内只有取第一个有效者红，另三种在对抗根环 KA 上红（第 3 次重跑第三段）。
- 三方判决（M1–M4 的措辞与落点）：`research/prompts/m3-formal-r1-main-verification.md`、`research/prompts/m3-formal-r2-main-verification.md` 第二节、`research/prompts/m3-formal-r3-main-verification.md` 第二、三节；M3 单调推论的四步推导在 `m3-formal-r2-opus-output.md`「3.2 推的：串行提交下，j1 与 j2 推得出落到的版本单调」一节。
- 三方判决（M3 不动点前提补 I-8.12（同实例记录的 txg 不减） 与落点现状）：`research/prompts/m3-formal-code-r1-main-verification.md` 第二节不动点前件那一格（打中条款、不打代码）与第三节第 5 条；装置的前提比 Verus 的前件宽、违例全落在该谓词不成立一边的对账在调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 10 批「10-5 交回」那一条。
- 用户定案（2026-10-01 弹窗，都照推荐）：卸载记号除外、M4 射程句的前提写全、单调的版本序取 (`checkpoint_txg`, 实例代号)、txg 顶格取「报 no_successor 且一条没施加算相等」；写回等 K10 判完一起做，原话在变更史。
- 挂载透明：无实验：判法在崩溃注入里（`crates/singlefs-harness/src/mount_transparency.rs`，checker 档的崩溃注入调它），不是实验；三轮判决里攻方的数在副本装置上量，代码轮第二轮攻方在入库代码上量的第一次崩溃 147 806 态、二次崩溃 234 773 态 0 红主 agent 没复跑。
- 三方判决（挂载透明）：`research/prompts/m3-formal-r4-main-verification.md` 第二节「挂载后再恢复：模掉什么」「比的对象与射程」两行与第四节第 2、3 条（独立的第四条、比的那一版、射程）；`research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条（逐字段表、二次崩溃比 v1、拒绝合法的条件）；`research/prompts/m3-formal-r6-main-verification.md` 第二节 U5、U6 两行（txg 只判下界、拒的判据在崩溃镜像上自己复算）与「逐字段表没碰的字段」一行；`research/prompts/m3-formal-code-r2-main-verification.md` 第二节 K2 各行（实现照判、站住；暖机与挂载推的抬 F 拆段）。
- 用户定案（挂载透明加二次崩溃，2026-10-02 弹窗，原话在变更史）：`research/prompts/m3-formal-r4-main-verification.md`「用户定案（2026-10-02）」二次崩溃那一行。
- 择根序变体那一句：调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 12 批 E165（恢复重放有界穷举与变异杀伤界） 第 3 次重跑第三段判 Q2 交回那一条（主 agent 判补这一句）。
- 挂载续做算挂载的一部分、F 的界按挂载交回的抬 F 在写表末尾的位置取：用户定案 2026-10-03「续做算挂载的一部分」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 3 条（界按写流位置取，找不到不 panic）；实现 `crates/singlefs-harness/src/mount_transparency.rs`。
- 拒绝合法那一句里「读阶段判完之后系统配置又读不出」这一拒因的复算带补写例外、判槽自证读池级 checker 的逐盘数：三方判决 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第二节 Y1、Y3 两行与第三节交用户表「条文」「G4 拒绝那一臂的判槽自证」两行；用户定案 2026-10-22「改读池级 checker」，零轮，原话在变更史。

**欠**：C77（重放起点未定义）：在择新与前缀判定那道接缝上；C649（Verus 判不到的两样回调结果）：已还清（M2 语义层的反向链与点名单元仍只能写成后置条件，Verus 一侧不判，两个读盘回调改由 `crates/mutations.tsv` 的变异行罩着，见欠账表已还清表）；C650（择根与重放没有合判）：M4 的射程句要择根与重放合起来才成立；C653（扫环胶水内存随盘数涨）；C654（必红世界期望条数随求解环境漂）；C655（挂载透明逐字段表有字段没归类）：已还清（逐字段表照判决 Y1、Y2 补齐，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:91-104`（整段抄，未转述）**

```markdown
#### 已定项 2：先串行提交

**定案**：事务层先承诺串行提交——`.claude/rules/fs-design.md`「判据写不出来时，先只做一条，而且**先做慢而正确的那条**」落到实处，串行是本工程明令的默认值，不是在两条路里挑了一条。越过交点再开并行，而不是在交点之前就承诺并行。对将来并行提交的一条前置约束：记录按事务号顺序追加，事务号 N 的 fsync 在返回前必须等 < N 的事务全部已追加 ∨ 已作废 ∨ 已确定实例切换或转只读（作废是提交管道的内存状态，对并发后继可见）；串行提交下这条规则空转。

**射程**：串行的天花板在一次 checkpoint 约 1.1 万条条目（单线程合并耗时追上一次设备同步的交点，算式在 C58（串行提交撞天花板没人报警））：低于交点设备同步是瓶颈、串行不吃亏，高于交点单线程合并变成天花板，没有安全余量。已定项 2 不决定记录头的字段清单、不改变新池新建文件写出的字节，它决定性能天花板与 crash refinement 的范围。并行提交仍然会动崩溃语义：「前缀」是全序上的概念，并行下一个前缀可能同时含有完整的事务 B 与残缺的事务 A；记录头已带事务边界字段（D23（journal 的角色与格式） 已定项 7：事务号 8 + 提交标记 1）。

**依据**：
- E17（write buffer 合并上限）：单线程 sort-merge 的上限是交点的一半；按叶分区的无锁并行是越过交点之后的逃生口，但不许默认它够。
- E44（序号位宽的本机实测与代价）：本机 fsync 率是交点的另一半。
- E42（一事务几条记录）：串行下一个事务跨多条记录同样会被前缀切开——决定记录头字段清单的是「一个事务恰不恰好产生一条 journal 记录」，不是串行 / 并行。
- 用户定案，记在变更史；将来并行提交那条前置约束来自 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1。没走三方论证：这是确认一条已经写明的默认值，真正要走三方的是何时改成并行，那一步没做。

**欠**：C58（串行提交撞天花板没人报警）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:105-137`（整段抄，未转述）**

```markdown
#### 已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义

**定案**：不做只管功能的内存模型，做一个自己处理持久化与崩溃语义的参照实现（RefFS），把 RefFS 从「储备候选」升为**要做的东西**。「做到极致」取语义级：RefFS 只在内存里按 POSIX 操作走，自己定义每次 fsync 之后什么算持久、崩溃之后允许看到哪几种 POSIX 状态，没有盘上格式；带自己盘上格式、自己写盘与恢复的那一档（工程量接近再写一个文件系统）不做。RefFS 与实现不许共用崩溃点枚举的那段代码：做到极致之后「三 oracle 合谋测试」（已定项 6）防的那个共同误解，最危险的落点是「RefFS 与实现是否共享同一个崩溃语义的错误理解」。

RefFS 的来处与边界：

| 项 | 定案 |
|---|---|
| 来处 | 只照 kb 条文另写：不拷 O1（内存模型对拍（HashMap 理想文件系统）） 的代码，不读录制流，不调切段、状态计划与实现的崩溃点枚举 |
| 放哪 | 单独一个包 `crates/singlefs-reference-filesystem`：依赖闭包（dev-dependencies 也算）⊆ {它自己, `singlefs-format`}，能只用 std 就只用 std；源码不引 `singlefs_core`、`singlefs_checker`、`singlefs_harness`、`singlefs_checker_tier`（经改名的别名同样算）；`singlefs-core` 与 `singlefs-checker` 的依赖闭包里没有它。三条由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判 |
| 输入白名单 | 都由调用方交入，只收五样：每个操作的结局（成、在任何写之前拒、写了之后报错、原样重发）；崩溃落在第几个操作里（hi）与它的归属规则；回退目标 m；每次恢复读出的内容；显式发布点 |
| 第一版的操作 | 第一个文件、覆盖写、发布调用、fsync、回退 |
| 调用点 | 崩溃注入与故障注入的调用点上，RefFS 的判定与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官的全部四条判据合取，任一红即红 |

崩溃语义（串行提交、不带故障的崩溃状态）：

1. **fsync**：fsync 返回成功之后，这次 fsync 等到的那次发布带走的全部操作必须在（串行下就是调用开始之前已取号成功的全部操作，按单元计）；按状态下标读，即允许集合的下界 lo 抬到那次发布的末尾。fsync 报错时不抬 lo，报错那次发布带的操作可在可不在；之后的操作接在下一次恢复读出的状态上判。
2. **丢法：按发布边界的前缀**。崩溃落在第 hi 个操作里时，恢复允许读出的是 {S_j : j = lo，或 j 恰是某次显式发布的末尾；lo ≤ j ≤ hi}，S_j 是上一次恢复之后前 j 个操作施加出来的状态，S_0 是上一次恢复读出的内容。丢掉任意子集的那种丢法在串行提交下出局（与 D16（发布语义） 已定项 12 不相容），开并行提交时重开。
3. **抬 lo**：返回成功的发布调用、fsync、回退都抬 lo（D16（发布语义） 已定项 7：回退与 fsync 同一个返回条件）；回退算一次已确认发布，它的下标算发布末尾。
4. **回退**：回退是 RefFS 的一个操作，回退之后的内容等于整段历史里第 m 次已确认发布之后的内容；m 由调用方从实现报的回退根翻出来交入，RefFS 不判目标选得对不对。m 的编号覆盖每一个写根的入口（发布调用、回退、写行与暖机、抬 F、卸载），映到写它时的现行版本，现行版本含每次恢复读出、重新起头的那一份。
5. **崩溃时刻的归属**：崩溃落在第几个操作里，按「切开当前段的那道屏障由哪一步发出」归属，不按第一个被扣下的写；这条只给 RefFS 用，O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不动。
6. **恢复之后**：每次恢复之后，RefFS 先判读出的状态在允许集合里，再以它为起点接着走。

**射程**：只定做到什么程度与 RefFS 的形态，不定排期；不许读成「先做 RefFS 再做实现」——RefFS 是验证手段不是里程碑（`.claude/rules/format-evolution.md`「从写开始，不从读开始」）。显式发布点只取历史里的发布调用与 fsync：自动触发的攒批落地之前 RefFS 只接显式发布点，自动触发的发布不在 RefFS 射程，归已定项 1 的 M2 与记录核对器；崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步。带故障的崩溃状态（说谎的设备、丢整块盘 C519（丢一整块盘时恢复丢掉刚确认的那一版））不适用。独立性：RefFS 不读录制流、不调切段与状态计划，但它的 (lo, hi) 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官盘级下界那一条判据的入参输入同源，记在 C191（同源性计数器只盖了一对） 那一类同源账上。D17（实现分层与第三方管道） 已定项 4 据此把「入口」的可执行定义交给 RefFS 的操作面，fsync 的持久性承诺由 RefFS 的崩溃语义定义；O3（独立规约执行器） 仍是崩溃点集合的独立执行器，职责不变。RefFS 的第一版在 `crates/singlefs-reference-filesystem`，崩溃注入两截与故障注入重开的调用点已接：harness 档经翻译层 `crates/singlefs-harness/src/reference_filesystem_translation.rs` 依赖它，调用点判 RefFS 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官两票、任一红即红。

**依据**：
- 无实验：做到什么程度、RefFS 怎么来与收什么是验证投入与边界的取舍；三轮判决里攻方的数都是副本装置上量的，入库装置上还没重做（`research/prompts/m3-formal-r5-main-verification.md` 第五节第 3 条），这里不引。
- 用户定案，原话在变更史：「做到极致」（原定案）；做到语义级（2026-10-02，岔路单 `m3-formal-7-forks.md` 里 RefFS 做到哪一档那一行，是对原定案换了读法）；按发布边界的前缀、调用点合取、另起、单独一个包（2026-10-02，`research/prompts/m3-formal-r4-main-verification.md`「用户定案」）；合取收严成全四条、回退目标作输入（2026-10-02，`research/prompts/m3-formal-r5-main-verification.md`「用户定案」）；回退抬 lo、崩溃时刻按屏障归属、m 覆盖每一个写根的入口、白名单加原样重发（2026-10-02，`research/prompts/m3-formal-r6-main-verification.md`「用户定案」）。
- 三方判决：`research/prompts/m3-formal-r4-main-verification.md`（第一轮：fsync、丢法、独立性三格与六条岔路）、`research/prompts/m3-formal-r5-main-verification.md`（第二轮：fsync 合成句、按发布边界补三句、合取收严、另起与单独一个包、任意子集出局限串行）、`research/prompts/m3-formal-r6-main-verification.md`（第三轮最终轮：回退与 lo、射程两句、白名单、fsync 按状态下标读）。
- D17（实现分层与第三方管道） 已定项 4：O1（内存模型对拍（HashMap 理想文件系统））明写「不管崩溃」，定义不了 fsync 的持久性承诺，而第一个可运行目标恰恰是「正确提交一个事务」——做到极致的 RefFS 补的正是这一半。

**欠**：C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C191（同源性计数器只盖了一对）；C658（崩溃注入第三截放过丢掉的一步复活）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:138-153`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，全量与快档都枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。**同一块盘上连着的屏障并成一道**：录制流里一道屏障与这块盘上一道屏障之间没有任何一次写（别的盘的屏障不算写）时，这一道不记（`crates/singlefs-harness/src/lib.rs` 的 `push`），独立模型照同一条规矩记；中间没有写，这一段不产生崩溃状态，合并与否不改变崩溃状态集合，只改变操作数与种类串。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。崩溃点重放的扇区取 512 字节；两条崩溃点重放流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条崩溃点重放流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；崩溃点重放的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（新池新建文件的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案，2026-09-26 用户定照实现改成「关掉它所在的那一段，与前面没被屏障隔开的写同段」），记在变更史。
- 按设备切段（有一块盘没放行就不关段）、FUA 只放行它那块盘、原地覆写多取「新旧都读不出」：用户定案（代码审阅第 1、3、4 条；2026-09-27 弹窗选「A 合并，按设备记屏障」「照补，全量也跑」与「原地覆写补第三态」），第三态按字节撕是主 agent 同日定的、被攻过零轮，记在变更史；实现与判别力（只吞盘 1 一道屏障，段尾 `26,2,1,2` → `26,5`，崩溃点重放小流里记录核对器报出来）在 `m2-rev-b3a2-implementer-report.md` 第一、三节。
- 同一块盘上连着的屏障并成一道：用户 2026-09-28 定「并，跟今天的录制器」（E142（新池新建文件的干跑） 第十九次跑撞上登记的停机 S19-crates：独立模型两道都记、录制器并掉第二道，暖机与整条流四格的操作数与种类串不同、崩溃状态数相同，报告 `e142-r19-runner3-report.md` 结论段，登记 `e142-r19-prereg.md` 的 R40），原话在变更史。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/13-验证路线.md:154-170`（整段抄，未转述）**

```markdown
#### 已定项 5：只共享一份从 kb 生成的常量

**定案**：checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改（`.claude/singlefs-ai-sop/rules/machine-first.md`：重复要生成，不能手抄）。**其余一律不共享**：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空（C12（增量语义共用） 由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格定的判据，形态是 crate 粒度的依赖闭包交集、不是符号级，「格式解析与常量除外」那条例外就是这一项在用）。

- **判定宽度不许从别处拿**：I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 的判据是挂载时探测的 `physical_block_size`，它不在镜像里。checker 自己探一次，再与系统配置记录的 mkfs 时取值比对（D22（单元原子性怎么合成） 已定项 2「mkfs 按池内最大值划、挂载时逐设备复算比对」的 checker 侧形态）。**探不到时报「声明值，未探测」，不许当成探到的**——文件当设备的镜像属于这一类，崩溃点重放的绝大多数镜像都是文件，所以这条不是边角。异构池里判定宽度是一个集合不是一个数（C17（设备几何当全局参数）），checker 要复算 mkfs 那步取最大值的聚合，不然健康的异构池上会假红。
- **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。
- **CRC32C 各写一份的前提是参数钉死**（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性。

**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 code-source-discipline 的 format-constants 格按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size` 与 `io_min`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源的类型是 `crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`：声明值那一种（`DeclaredOnly`）已接到 I-7.5（根槽按判定宽度对齐） 的判定上（已实现，`decision_width_declared_by_the_chosen_system_configurations` 取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的），探到那一种（`Probed`）只有类型定义（`crates/singlefs-checker/src/lib.rs`），全仓没有构造它的代码、没有接到任何判定上；I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。

**依据**：
- 无实验：共享边界是保验证独立性的政策，没有可量的量；它的判别力靠两边各写一份的检查逐条红给人看。
- 三方对抗：材料 `_checker-sharing-vehicle-background.md`，腿的产出 `research/prompts/checker-sharing-vehicle-*`，腿况在 [verification-build.md](../verification-build.md)——正推腿从 D13（验证路线）、C12（增量语义共用） 推出「常量在例外内、其余禁止」；替代反推腿与本地腿各自构造出同一条：kb 里写错的值共享与各抄都抓不住。
- 用户定案，记在变更史。

**欠**：C17（设备几何当全局参数）；C387（常量模块手写，不是从 kb 生成）；C388（checker 的判定宽度不自己探测）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:171-201`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构；无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3） | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导；第一版用 Python 单写（E170（O3规约执行器第一版）） | 规约本身可能写错——**必须配阳性对照才有意义**；被投影的六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值（journal_tail）、一次发布内各写的先后 |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
功能正确性不只押在这三个 oracle 上：O1（内存模型对拍（HashMap 理想文件系统）） 那一行已把崩溃之后允许看到的集合交给 RefFS（已定项 3），恢复函数的 M2–M4 落在 Verus 与 E165（恢复重放有界穷举与变异杀伤界）（已定项 1），需要第二个输入的核对归记录核对器（已定项 7）；三个 oracle 之间的共同盲区靠「三 oracle 合谋测试」找。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 的形态是 `crates/singlefs-harness/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 harness-model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 是只照 kb 文字写的 Python 执行器，现行那一份是 E171（三oracle合谋测试） 第 1 次重跑用的 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`（E170（O3规约执行器第一版） 第 2 次重跑从第 1 次重跑冻结的 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` 拷出 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` 加谓词 W、T，再跟上 487 字节 journal 记录头；第 1 次重跑冻结的那一份与第一次那一份 `scripts/experiment-apparatus/e170_o3_spec_executor/` 留档）：严格跑法判得出（第一次导出的批 1、批 2 上 0 态撞「规约欠一句」），与实现逐态一致（两边全放行，不当它判得对的证据；「择到的根不是自证根里 txg 最大的那条」「③ 收到不少于 2 槽」两条路径批里 0 态，C661（规约执行器批里两条路径没有态）），坏状态表照改正的期望重判有判别力（阳性对照与盲测都判对），还没接进崩溃点重放；判一个崩溃态的做法：照规约的恢复规则（择系统配置、择根、前缀按 D23（journal 的角色与格式） 已定项 14 的五条口径）从镜像自己恢复出一版，恢复不出（整池拒或没有自证根）判不在；恢复出的这一版的投影（实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F）要等于版本表里某一版的投影，且这一版的下标在 [已确认, 已开始] 之间。版本表是基镜像上自己恢复出的现行那一版，加录制流里每一次根槽写按流序各一版，每版的投影取实现写进根槽的字节；已确认取崩溃态里落了的最晚一条写之前已返回的最晚一版（只从落了的写推），已开始取落了的写所属的最晚一版。系统配置槽只用来择槽、取根环与 journal 环的几何、判池级字段越界整池拒。所以恢复规则那一半是照 kb 文字另写的，允许集合那一半取自实现自己写出的根；RefFS（已定项 3）在 `crates/singlefs-reference-filesystem`，崩溃注入与故障注入的调用点已接；「三 oracle 合谋测试」是 E171（三oracle合谋测试），读数取自一棵冻住的树（快照摘要见实验页，与现行 `crates/` 不是同一份）：O1（内存模型对拍（HashMap 理想文件系统）） 一列是模型对拍加模型的崩溃判官，不接 RefFS，与表里 O1（内存模型对拍（HashMap 理想文件系统）） 那一行不同（崩溃之后的集合那一半 E171（三oracle合谋测试） 没用 RefFS）；O3（独立规约执行器） 一列用 E170（O3规约执行器第一版） 第 1 次重跑冻结的那一份；手造坏镜像那一族只喂 O2（独立解析器 + checker）、O3（独立规约执行器），期望出自 E170（O3规约执行器第一版） 的设计员，Q2 照登记字面判「有」（4 张全是 P8），按登记 F3 判成 E170（O3规约执行器第一版） 推期望推错、这四张不是坏镜像，真坏镜像里 O2（独立解析器 + checker）、O3（独立规约执行器） 两条全放行的 0 张；三条的源头是否独立（表第 4 列）E171（三oracle合谋测试） 只在 Q2 量过 O2（独立解析器 + checker）、O3（独立规约执行器） 一对，那一对的补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，不当独立读数，O1（内存模型对拍（HashMap 理想文件系统）） 不在 Q2 里；没有量到的不写成「互相独立」被验证过。

**三条 oracle 都不判的两样**（第一样由崩溃注入与崩溃点重放两侧判，崩溃点重放那一侧的观察者在实现批）：一是无读故障的崩溃态上，D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真（系统配置槽里的见证值超前于盘上读得出的记录）：崩溃注入第二截判拒绝合不合法时只在崩溃镜像上复算这条判据，为真就判合法（挂载透明的射程只管不带故障的状态，已定项 1）；要有判官问「这个崩溃态没注入读故障，判据凭什么为真」：崩溃注入第二、三截在没注入读故障的崩溃态上复算这条判据，为真即判拒得不合法（`crates/singlefs-checker-tier/src/crash_injection.rs` 的 `RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`，C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 已还清，见欠账表已还清表）。「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由崩溃点重放 mkfs 返回之后各条流上的观察者判：观察者按流登记，记录 mkfs 的流不挂，只记相对基镜像新出现的、一份自证过的系统配置都没有的盘，撕裂域在内，判槽自证读池级 checker 的逐盘数（`crates/singlefs-checker/src/image.rs` 的 `system_configuration_redundancy_report`），另有一条登记完整性用例判含系统配置槽写段的流都挂了它（实现在实现批）。随机历史里补写闸（D22（单元原子性怎么合成） 已定项 8）不过的那一形，由崩溃注入第二、三截的 G4 拒绝那一臂判；补写闸过了的那一形，随机历史里没人判，正确实现留不下这种盘面（两次抽样都没打中）。mkfs 返回之前的崩溃态不在观察者与 G4 拒绝那一臂的射程：mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后；G4 拒绝那一臂与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条还有一处不同形：择不到系统配置时那条的复算交 `None`、G4 按真，这时读阶段本身已拒、到不了这一格。二是一次发布内各写的先后：O2（独立解析器 + checker） 的 I-7.12（系统配置 F 不低于同盘根上的 F） 判抬 F 那一串先写系统配置这一步；变异点名的用例只判 E171（三oracle合谋测试） 第 270 行那一种（发布返回之后每个崩溃点的见证值），第 242 行点名的用例判的是失败路径冻结；`layout/01-first-txn.md` 第八节把根槽 FUA 与系统配置槽轮换登记成两段，只是登记在册；其余写序不在三条之内。

**依据**：
- E170（O3规约执行器第一版）：O3（独立规约执行器） 照「另一种语言、只照 kb 文字重新推导、配阳性对照」写得出第一版，判得动单个崩溃状态、一态够进快档；第 1 次重跑（照 2026-10-03 的 kb 改过一次的执行器）严格跑法判得出、与实现在取样的批态上一致、阳性对照有判别力（第一次跑判不出与判别力不足那两笔留在实验页）。
- G4（无读故障的崩溃态上每块盘至少一份自证过的系统配置）：代码轮判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 j2-b 立、第 29 批实现，射程与 mkfs 那一段见 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L3 与第三节第 3 条。
- 两槽全废盘面改由崩溃点重放 mkfs 返回之后各条流上的观察者判、G4 拒绝那一臂留着：三方判决 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第二节 Y2 各行与第三节（前两轮 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`、`research/prompts/m3-close-g4-rewrite-r2-main-verification.md`）；观察者的接法（按流登记、相对基镜像、登记完整性用例）是用户定案 2026-10-22「按流登记+相对基镜像」，零轮，原话在变更史。数在三方腿的模型与副本原型上量，没立实验号。
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性靠「三 oracle 合谋测试」观测（E171（三oracle合谋测试））。
- E171（三oracle合谋测试）：在一棵冻住的树上，坏实现表里有三条 oracle 全部放行的行、有只靠一条才抓到的行，O3（独立规约执行器） 在坏实现表上一行没红；手造的伪造根、伪造续接镜像里有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的（读数与限定在实验页 Q1、Q2、Q3 三行）。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 的分工句：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节另起与单独一个包那一格（两腿同向）。

**欠**：C445（模型对拍的取样点不随写路径增长）；C446（未实现的不变量没有前置登记，也没有到期）；C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C407（三 oracle 合谋测试的判官没进门禁）：E171（三oracle合谋测试） 已量过一次，没有一格门禁读它的产物；C672（交回值或内存状态错而写流不变的一类）；C679（卸载记号没进模型对拍）；C874（补写那一支进不了崩溃测试、运行时看不出）：两槽全废盘面的观察者与 G4 拒绝那一臂改读池级 checker 都还没实现

```

**出处 `.claude/kb/decisions/13-验证路线.md:202-220`（整段抄，未转述）**

```markdown
#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像

**定案**：

> **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
> 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`——持久集合是枚举器给出的这个崩溃状态里哪些写落了盘（崩溃点重放的段内整写子集）；只看盘上字节分不开「写没落、而原地内容恰好相同」与「写落了」，判复用与缺席要拿持久集合判；它与自研第 1 项「Merkle 根链差分」（已定项 8）同属二元检查。⚠️ **故意不给它编号**——O2（独立解析器 + checker） 那场混乱就是编号当称呼造成的，再加一个 O4 只会给下一次漂移多一个落脚点。⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。

**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-checker-tier/src/crash.rs`，接在崩溃点重放的每个崩溃状态上（门禁 checker-tier-crash-point-replay）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 崩溃点重放不覆盖可写挂载：崩溃点重放在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在崩溃点重放的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-checker-tier/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。

**依据**：
- 无实验：**依据是类型，不是成本**：「逐字节比对」是二元关系 `R(镜像₁, 镜像₂)`，**没有任何投入能把一元谓词变成二元关系**；I-4.5（已停用·需两镜像） 因同一条门槛退役。
- 「崩溃后镜像」是两份：随 D16（发布语义） 已定项 4 收窄问法那一轮三方论证（材料 `_d16-items134-background.md`）由用户定案，记在变更史。
- 持久集合那一样入参：用户定案，原话在变更史；判别子观测不到的实据在 `research/prompts/m2-layer0-scale-r3-main-verification.md` 第二节按扇区豁免那一行。
- 崩溃点重放不覆盖可写挂载、可写挂载与二次崩溃由崩溃注入覆盖：用户定案（代码审阅第 2 条，2026-09-27 弹窗选「崩溃注入层补可写挂载」），记在变更史；崩溃注入那一侧的实现在 `m2-rev-b3b-implementer-report.md`。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:221-244`（整段抄，未转述）**

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
- 无实验：两样都靠 COW 每事务发新根与 birth txg 的结构性质成立，都还没有实现，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C389（Merkle 根链差分零代码）；C194（代际增量 scrub 看不见被搬过的副本）；C408（代际增量 scrub 零代码）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:245-278`（整段抄，未转述）**

```markdown
#### 已定项 9：崩溃点重放要分三层，否则挂钟必炸

**定案**：

| 层 | 范围 | 触发 |
|---|---|---|
| 冒烟层 | 固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模（规模见射程），精确全量 | checker 档的全量（已定项 15）：用户要求时跑；提交时默认只跑快档 |
| 常规层 | 中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore，覆盖报告与门禁逐个列「本次未跑」 | 每次提交 |
| 大负载层 | 常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量 | 用户要求时 / 发布前 |

**剪枝加 ignore 之后敢说什么**：敢说「开着的节点在全部崩溃状态上通过」；
**不敢说**「ignore 的节点通过」。

**全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快档全绿（必要条件）。**
**里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑一趟全量；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑。**
「全量」指两样：checker-tier-crash-point-replay `--full`（门禁 checker-tier-crash-point-replay，已定项 15 的全量）与崩溃放量全量（`scripts/crash/crash-amplification.sh` 不带参数）。
「动了崩溃点重放或判器」按崩溃判定库现算：这一批改动让库里有判定块的单元键对不上、对不上的那几把键里有判器单元（判器入口 fn 的单元闭包里的函数单元）变了，就算；阶段把作废块数与变了的判器单元清单打出来，主 agent 照它弹窗，不按路径清单判。
「上一次全量」由主 agent 写进当前里程碑记录一行：日期、两样各自的结论。
**全量档必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个全量档 bug 的挂钟 ≈ `log2(N) × 一趟全量的挂钟`，一个不可复现的全量档失败价值接近 0，
甚至为负，它训练人忽略红灯。

**射程**：只有冒烟层（门禁 checker-tier-crash-point-replay）；冒烟层分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；全量带断点续跑（门禁 checker-tier-crash-point-replay `--full`），登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档，全量按已定项 15 由用户要求时跑；常规层、大负载层在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；节点怎么精确剪枝、ignore 节点什么时候开归已定项 10、11。

**依据**：
- 无实验：分层是门禁挂钟的设计约束；常规层、大负载层在门禁里的负载还没有定，E184（常规层与大负载层的代价计数） 第一段已量候选负载的规模与态数区间重叠，只备料；第二到四段（作废面、冷吞吐、两种落法的固定开销）未跑。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 平时快档用甲二：用户定案，原话在变更史；崩溃点重放规模三轮判决 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- 用户定案：「我们不是抽样 我们是剪枝 加 初期ignore」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定案：不抽样，剪枝加初期 ignore」那一行）、「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）、「1 该枚举该设计的设计完 设计全」「2 枚举不完 正是剪枝的用处」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节里材料员被门禁 doc-process-records 的 implementation-premise 格挡下、用户定『设计全、枚举不完靠剪枝、粒度合适』的那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 9 推翻。
- 用户定案（2026-10-01）：全量档不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。
- 用户定案（2026-10-01）：全量档全绿的时点是里程碑收尾与动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑，原话在变更史；交用户那一格是三方判决 `research/prompts/test-script-align-r1-main-verification.md` 第二节 t1-3。
- 三方判决 `research/prompts/test-script-align-r2-main-verification.md` 第二节 U3（「U3 t1-3 写回照字面做不了」那一行）：「全量」的两个所指、「动了崩溃点重放或判器」的路径判法、「上一次全量」的记法，「准入条件」与「异步」合成一句。

**欠**：C409（常规层、大负载层在门禁里没有阶段）；C635（上一次全量没有机器可读的标记）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:279-308`（整段抄，未转述）**

```markdown
#### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」

**定案**：常规层 精确覆盖先缩问题，再谈剪枝。崩溃点的定义域是**事务层状态机**，而 `.claude/rules/fs-design.md` 有两句顶层规则：
「一个事务层，所有结构共用」与「唯一不许多样化的是事务层」。所以布局能改的只有两类：

| 差异 | 改变的是 | 判定 |
|---|---|---|
| 节点大小、条带宽度 | 一步写多少字节、某步重复几次 | **参数**，不新增步骤种类 |
| 旋转 vs SSD | seek 代价、IO 调度 | 落在设备层，**不落在事务层协议里** |
| **zoned（ZNS）** | **能不能原地覆盖根指针** | **结构性**：只能追加、根位置漂移，且多出 zone finish/reset |

**落地手段是一个封闭枚举 + 穷尽 match**：`crates/singlefs-core/src/transaction.rs` 的 `CommitStep`，步骤种类的登记位是 [layout/01-first-txn.md](../layout/01-first-txn.md) 八的段序列登记表。
任何布局要引入新的结构性步骤**必须改这个共享定义 ⇒ 编译器强制所有引用处补分支 ⇒
这个 diff 门禁 grep 得到**，可以自动打标「本次引入了新的结构等价类，需为它新开一轮独立大负载层 重放」，
**不靠人记得申报**（按 diff 算门禁范围欠在 C8（门禁范围判不出来））。再加 newtype 隔离（`RotationalPtr` / `ZonedPtr`），
把 D12（目标介质） 已定项 2 里「不许共用」那一桶（记账事实的判定路径）变成编译期约束。
⚠️ **它有第二个职责（D17（实现分层与第三方管道） 加的）：它是第三方管道的接入点。**
协议归本工程所有，一条管道要引入新的结构性提交步骤必须往本仓发这个 diff。

**射程**：旋转与 SSD 在提交协议层面同构，由 D12（目标介质） 那一轮三方论证承担；zoned 那一行只说明它至少多出一个等价类。等价关系按录制流的段序列同构、有几个，归 D17（实现分层与第三方管道） 已定项 2。按种类串认节点够不够，第一轮判决打中（挂起，待第二、三轮攻）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节。
⚠️ **它钉住的只是「有哪些步骤」，钉不住「每一步的失败原子性宽度」**——
持久内存与对象存储改变的是每一步的失败原子性宽度，枚举接不住（缺口与补法见 D17（实现分层与第三方管道））。
`RotationalPtr` / `ZonedPtr` 这层 newtype 隔离 `crates/` 里没有（`grep -rn "RotationalPtr\|ZonedPtr" crates` 零命中）。

**依据**：
- 无实验：N 是什么由 `.claude/rules/fs-design.md` 那两句顶层规则推出，没有可量的量；等价关系怎么判、有几个的依据在 D17（实现分层与第三方管道） 已定项 2。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。

**欠**：C8（门禁范围判不出来）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:309-343`（整段抄，未转述）**

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
| T1 | 常规层 挂钟预算 | 常规层 本地挂钟预算，是输入不是输出 |

2. **`T1`（常规层 挂钟预算） 是输入，不是输出**（`.claude/rules/fs-design.md`：门禁运行时长本身是设计约束）。
   正确顺序是「先声明预算 → 用它约束一次提交能开几个 ignore 节点」，不是「量出耗时 → 推出能开多少」。
   **这一条不需要任何测量，现在就该定。**
3. **判别力必须与覆盖分开检查**：每个 ignore 节点被打开的那次提交里，额外对一份「已知坏的镜像」跑它自己的 checker，
   不报错即判红——这是 D13（验证路线）已有的阳性对照纪律的直接复用。见 C13（checker 判定失效）。

门禁里没有「常规层 崩溃点重放」这个阶段，量不出 `T1`（常规层 挂钟预算） / `C` / 每格耗时。
⚠️ **因此 `C`、`K_max`、`T1`（常规层 挂钟预算） 这三个数一个都不许现在写进 kb。**

**射程**：只管 ignore 的节点什么时候开；冒烟层全量枚举不需要 `T1`（常规层 挂钟预算）。第 2 条的「现在就该定」与末句「一个都不许现在写进 kb」说反话，记在 [verification-build.md](../verification-build.md)「待定案」第 6 问，没判。节点登记表带状态字段（开着 / ignore / 等实现），另加三条——每个 ignore 行点名一个会变的对象、那个对象变绿而节点仍 ignore 时判红、覆盖报告列 ignore 的状态数与占比：这三条被攻过零轮（`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节），不写成定案，等第二、三轮再攻。

**依据**：
- 无实验：否决账本靠的是一次调度规则的模拟验算（200 轮、取后半程稳态最大值、严格前缀贪心，三行数在变更史），没有登记成实验；留下的三条是门禁设计原则，没有可量的量。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。
- 用户定案：「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 11 推翻，「逐格判决」一节给节点登记表带状态字段的形态与三条待攻的补充条件。

**欠**：C8（门禁范围判不出来）；C13（checker 判定失效）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:344-359`（整段抄，未转述）**

```markdown
#### 已定项 12：冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具`

**定案**：**冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具` 直接打架。**
在用的不变量（条数的权威登记位在 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 文件头）若既是 checker 源码又是形式验证的 spec，就成了单点。
**可行的折中，而且本工程已经付过这笔钱**：checker 做两个方向——**遍历式**（从系统配置走下去）
与**扫描式**（全盘扫，靠「元数据块必须自描述」的块头重建），两者对同一条不变量独立判断，
共享的只有格式解析。

**射程**：管 checker 内部两个方向之间共享什么；checker 与实现之间共享什么归已定项 5。池级 checker 两个方向都有（`crates/singlefs-checker/src/walk.rs` 文件头）。

**依据**：
- 无实验：这是「checker 即规范」与校验路径独立之间的取舍，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C410（checker 两个方向共享什么没有检查）：两个方向「共享的只有格式解析」这一条没有会红的检查（`.claude/gate.d/` 里没有判它的阶段）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:360-380`（整段抄，未转述）**

```markdown
#### 已定项 13：冲突 2：crash refinement 的 spec 与模型对拍的 model

**定案**：**冲突 2：crash refinement 的 spec 与模型对拍的 model 是同一个东西的两种写法。**
合并成一份 → 省一半工作量，但**直接违反独立性要求**；各写一份 → 名义独立，
但通常是同一个人同一周写的。缓解（只降低相关性、不消除）：让两份**在表述层面就不同**——
spec 写成「对镜像的状态谓词」，model 写成「op → 状态转移」，不一致时至少能互相定位。
**同源性要被观测**：统计「spec 与 model 不一致」的事件数及每次判定哪一方错；
若累计 ≥ 5 次不一致而判定「spec 错」的次数为 0，说明两者实际同源，应把两条腿合并、预算移给穷举。
判定分三格：spec 错 / model 错 / 条款欠一句（两边各照条款的一种读法，条款没写谁对）。一次不一致先在前提成立的域里重判，还在的才记；「条款欠一句」的事件先记账，不计入「累计 ≥ 5 次」，也不计入「spec 错」，条款补了那一句之后按补的句子再判一次、归到前两格。
这里的 model 指 O1（内存模型对拍（HashMap 理想文件系统））。O3（独立规约执行器） 能对同一（历史、崩溃点）交出判定行之前（第一版 Python 执行器 E170（O3规约执行器第一版） 已有，没接进崩溃点重放，C447（规约执行器没接进崩溃点重放与 RefFS 比对）），O3（独立规约执行器） × RefFS 这一对登记为空格、不计事件；O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不当替身；没立的对报「未立」，不报绿、不计入任何一格。

**射程**：缓解只降低相关性、不消除。已定项 1 撤下的那条原候选不受这条影响，理由在已定项 1 的射程。crash refinement 的代码是 `crates/singlefs-replay-prefix/src/lib.rs` 里的 Verus 核与它的 spec（取前缀、择根、扫环三截与择根和取前缀的合判推论，覆盖已定项 1 的 M1、M2 句法层与 M3 的两读和单调推论；M2 语义层与 M4 靠 E165（恢复重放有界穷举与变异杀伤界） 在界内判）；spec 不能对具体状态求值，「spec 与 model 不一致」的事件从哪来没定，计数器没有代码（C191（同源性计数器只盖了一对））。

**依据**：
- 无实验：同源性要靠将来 spec 与 model 的不一致事件计数观测，spec 与计数器都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 第三格「条款欠一句」：三方判决 `research/prompts/m3-formal-r1-main-verification.md` 第二节「F2 同源性」一行（攻方打中，推的）、`research/prompts/m3-formal-r2-main-verification.md` 第二节第 9 行、`research/prompts/m3-formal-r3-main-verification.md` 第二节同源性计数器一行，三轮站住；无实验：计数器仍没有事件来源。
- 「model 指 O1（内存模型对拍（HashMap 理想文件系统））」与没立的对登记为空格：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节同源性计数器那一格（两腿同向的部分）；`research/prompts/m3-formal-r4-main-verification.md`「用户定案」里替身对那一行（不算替身对）。

**欠**：C191（同源性计数器只盖了一对）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:381-399`（整段抄，未转述）**

```markdown
#### 已定项 14：与其他决策的连锁

**定案**：

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） | 根链差分把 Merkle 从「要付的代价」翻转成验证杠杆；但形式验证里哈希是 uninterpreted function，**碰撞抗性完全在验证范围之外，不许读成「验了 Merkle」** |
| D8（核心索引结构） | 日志结构节点、write buffer、key cache 三层都会绑架符号执行（循环上界、滞后窗口、缓存一致性），使「Yggdrasil 一分钟内」的口径在本工程不成立 |
| D9（加密） | AEAD 在 SMT 里不可解，只能建模成 uninterpreted function，**「nonce 重用泄漏明文」这个后果验不出来**，只剩「nonce 不重复」这条纯组合性质——而这条纯组合性质恰是这条腿最值钱的产出 |
| D11（索引节点要不要留消息缓冲区） | 已定项 1 取「留」；D8（核心索引结构） 已定项 1 取幂等完整值之后 checker 的 I-3.1（已分配统计对得上） 不必解析消息语义（D11（索引节点要不要留消息缓冲区） 据此判它的代价 1 在本工程不成立），冲突 1 不因此变成「无解」 |

**射程**：连锁的是形式验证与 checker 碰到这几条决策时的边界，不改那几条决策的条款；根链差分与冲突 1 本身归已定项 8、12。

**依据**：
- 无实验：连锁是从那几条决策的已定条款推出来的边界，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:400-412`（整段抄，未转述）**

```markdown
#### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑

**定案**：`crates/` 下的验证代码按「什么时候跑」分两档，各住一个包，与池级 checker 一共三个名字：**harness 档**是 `crates/singlefs-harness`，单元测试与集成测试加上它们用的脚手架库（录制器、内存池、理想模型、随机历史、故障注入设备、场景），改了代码随时跑；包内再分轻重：debug 下单条跑到 60 秒及以上的是重，标 `#[ignore]`，单条用时看平常跑的时候量到的、不另外单独量，要跑随时跑，一律经内存包装；一条用例一个场景，测试文件按测什么起名、不按里程碑起名。**checker 档**是 `crates/singlefs-checker-tier`，崩溃态枚举引擎与记录核对器、崩溃枚举用例经判定库的两段与它的计数、发现日志与工具链身份、一份长期判定库（块键只由内容定，几类测试各自要哪些块只当索引）与两段协议、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对、实验与真设备装置二进制，以及它们的全部用例，连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档里的崩溃点重放（门禁 checker-tier-crash-point-replay）分两档，checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 不分：**快档**是两趟：`cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（`--tests` 带上全部 `test = true` 的目标，装置二进制的内联单测也在里面），再一趟带 `tests/` 里 `cfg` 点到的特性（`--features <特性> --lib --test <挂特性的测试文件>…`，特性里有 `gpu` 的这一趟经 `scripts/crash/gpu-quiesce-run.sh` 起）（门禁 checker-tier-crash-point-replay 在整轮门禁与提交时都跑这两趟，第二趟是它的 feature-gated-tests 格），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（checker-tier-crash-point-replay `--full`），提交时不默认跑，由用户要求时跑，跑法按已定项 9 分层；崩溃放量判器整套上了 GPU，罩全部池级 checker 不变量（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`）。**池级 checker** 是 `crates/singlefs-checker`，判决器库，被两档调用。依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ④ 条判，dev-dependencies 也算）；两档都要的一小段代码宁可各留一份。崩溃枚举用例一律写在 `crates/singlefs-checker-tier/tests/` 里（`scripts/crash/crash-case-check.py` 判），重型测试闸按包判：跑到 `singlefs-checker-tier` 的测试就是 checker 档。

**射程**：管 `crates/` 下代码与测试住哪个包、什么时候跑、闸按什么认、三个包怎么称呼（`.claude/rules/verification.md`「定义与名字」）；不管每条不变量判什么（[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)）、不管枚举域怎么定（已定项 4）、不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`），上游那一批已同步进副本、未发版。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。

**依据**：
- 无实验：什么时候跑、住哪个包是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
- 用户定案（2026-09-27），原话在变更史；三方 `verification-split-r1`（判决 `research/prompts/verification-split-r1-main-verification.md`）攻的是拆出 `singlefs-checker-tier` 之前的那一版，拆包之后的改动被攻过零轮。
- 用户定案（2026-10-01）：全量不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:413-428`（整段抄，未转述）**

```markdown
#### 已定项 16：按段登记的段整域装不进 u64 就分片判，态数用 1280 位定宽整数

**定案**：
- 分片：崩溃放量里按段登记的段（`.claude/gate.d/crash-point-replay-segments.tsv`，一行一段五列：项名、流名、段号、段写数、段写表摘要），整域态数装不进 u64 时按固定大小的片依次判；片大小由环境变量 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` 定，没设取 2^32 态；片起点是判定库里的续点（续点按段写表摘要与判器单元键寻址，项名、流名、段号只作显示）。整域装得进 u64 的登记段不分片，整域一次判。代码落点：`crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`、`crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`。
- 位宽：登记段的总态数、片起点、片尾与进度一律用 1280 位（10 个 u128）定宽无符号整数 `StateCount1280Bits`（`crates/singlefs-checker-tier/src/crash.rs:1640`，肢数是同一文件的 `STATE_COUNT_LIMBS`），算术全部 checked，不饱和、不截断；驱动行 `registered_segment_progress=` 的已判态数与总态数都写精确十进制，不写幂的形态。
- 登记表怎么交给引擎：引擎只经环境变量 `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` 读按段登记表（`crates/singlefs-checker-tier/src/crash_amplification.rs:14764`，在 `fn registered_segments_of_the_flow_from_the_environment` 里），没设就当一段都没登，没有别的退路。144 个单元的文件顺序写成 145 个、extent 下段长成两层的那条流（引擎流名 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels`）的单元写段（322 次单元写）与记录段（290 次 journal 记录原地写）两段都登。

**射程**：管崩溃放量引擎对按段登记的段怎么分片、态数用什么整数记、登记表从哪里读；不管哪几段该登（由用户点头，写在登记表文件头）、不管登记段的域本身怎么展开（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10）、不管每个崩溃状态怎么定义（已定项 4）。没登记的段的态数、与整条流合起来的态数仍要装得进 u64，装不下在写库之前拒跑；登记段整域越过 1280 位同样在写库之前拒跑。片大小只许 1 到 2^32：两个 GPU 内核把片内序号当 u32 加到片起点上（`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`）；轻型（`scripts/crash/crash-point-replay.sh --quick`）调用方没设片大小时设成 `LIGHT_REGISTERED_SEGMENT_SLICE_STATES`。展开上限盖不住又没登记的段，放量入口 `scripts/crash/crash-amplification.sh` 判红；崩溃点重放入口、放量入口与分项驱动开跑前核这份表再送给引擎。

**依据**：
- E161（崩溃放量的去重与分段耗时）「第 2 次重跑：取样域按 11 段的流重定（2026-10-05）」一节：段的全量态数与闭式、装置逐个数、crates 计划逐段相等，态数随段写数按幂长，撑的只是「长段整域装不进 u64」这个前提；片大小、位宽取值与登记表的读法是工程参数，用户定，它不撑。
- 用户定案，原话在变更史：分片与每片 2^32 态（`records/2026-10-11-崩溃放量两段与脚本审计.md` 第 96 行）、位宽 1280 位（同一份记录第 98、99 行）、两段都登（同一份记录第 92 行）。
- 分片与位宽是对判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10（按段登记）的补充，没经三方攻过。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:429-451`（整段抄，未转述）**

```markdown
#### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成

**定案**：
- 重型门禁、崩溃判定库与实验准入的每一份判定（崩溃状态块、快档与模型对拍的每条测试函数、crates 变异每行、广谱变异每条、形式化每条、QEMU 每档、litmus 每份、研究复跑每行、实验变异表每条、实验准入）沿不沿用，只看它的单元键：它依赖的函数单元各自去注释后的指纹，加闭包里各单元（含 mod 头）的 `#[cfg]`、`#[cfg_attr]`、`cfg!` 真引到的 cfg 原子、各带这一次目标下的真值（闭包没引到的原子不进键：带不带 `gpu` 特性编的两趟，闭包不提 `gpu` 的判定键相同、共用一份库），加工具链（含 cargo 解析后的生效构建环境与外部工具版本）、从 `Cargo.lock` 解析出的闭包真引到的依赖版本，加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）。函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。
- 一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。
- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。
- 两种排除不进键，理由不同、分开记。文件级：`scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份（算键的库自己与单元算法 `scripts/gate/lib_function_units.py`）按角色排，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键。边级：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收；现在只有内存包装 `HostLookup.query` 起 `scripts/hosts/host-resources.py` 那一条，第二台那一路只出提示，按这条边排出键。边级排除要配三格自证：这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达。另有一格「键不变而退出码变」必红。python 模块有单元进闭包时，它在 import 时执行的模块级语句一起收进来：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头。
- 每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；仓根也不是库的输入：崩溃判定库的目录由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 给时只收绝对路径，相对路径拒绝；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。
- 改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。

**射程**：管重型门禁、崩溃判定库与实验准入的复用键；不管静态门禁「只判新加行」的改动范围（`scripts/gate/changed-paths.sh`，那是判据语义）与 preflight 的 `admission: inputs-changed`；不管枚举域（已定项 4）、分层与什么时候请用户跑全量（已定项 9）。镜像判定与给定状态判定的版本是判定定义那一组（CPU 参照判器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` 的闭包）的键，不论这一块在 CPU 还是 GPU 上判的：GPU 判器是加速路径，它的准入靠与定义那一组的对拍（快档的 GPU 与 CPU 对拍用例），它的单元只进块属性当出处，不进键、不分种类；显卡驱动、卡型、几张卡怎么分活同样留作块的属性或跑的参数，不进键。

**依据**：
- 用户定案（2026-10-15），原话在变更史与 `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 用户定：提交与重跑无关，重跑只跟代码变化走」一节、「2026-10-15 批3」那几段。
- 用户定案（2026-10-17）：判定不与硬件、不与 CPU 还是 GPU 绑定，复用与剪枝不看设备；原话在变更史与 `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 叫停全量那几段。
- 主 agent 定案（2026-10-15）：仓根不进键与库、库目录只收绝对路径、cfg 成分只取闭包真引到的原子，出处同一份记录「2026-10-15 验证段第一步判红、两条定案、4-3 四批派出」定案第 1、2 条。
- 反证：E164（崩溃放量新设计的小范围功能测试） F4 量到只按函数取指纹、不连引用的常量一起收，漏判 V5 与 V11（V11 只改常量 `DATA_POINTER_BYTES`）：函数单元要连同引用的常量、类型、静态项与宏一起算。
- 盘上的数：`ledger.md`「一之二、盘上量到的」一节——只差名字的块各存各判 4100 块与 9679 块；整库一把的判法摘要一换累计多判 3 935 242 934 态，那次闭包里改动的 29 份文件有 24 份去注释后逐字相同：键带名字、按文件或整版取指纹，同一份判定会重判。
- 三方判决 `research/prompts/m3-close-reuse-closure-r1-main-verification.md`、`research/prompts/m3-close-reuse-closure-r2-main-verification.md`、`research/prompts/m3-close-reuse-closure-r3-main-verification.md`（两种排除并列、模块级语句随模块进闭包、第二台那一路只出提示按边排出键；三轮定，零轮改法交用户定）；用户定案 2026-10-22「丙加甲 要走三方」、2026-10-23 弹窗四题，原话在变更史。
- 没有腿况记录：除复用键闭包三轮（两种排除、模块级语句、第二台那一路）攻过的几处外，未经三方攻过；用户定改完代码开三轮攻方，只攻「剪枝是不是原子化的、键是不是只由函数单元组成」。

**欠**：C8（门禁范围判不出来）、C489（singlefs-core 的广谱变异对今天的代码没有结论）、C528（两条门禁路判的改动窗口不同）、C564（另外 6 条崩溃点重放全量没人跑）、C605（崩溃放量的复用与剪枝没有关掉再判的对照）、C630（提交时形式化那一道第 1 步停下的频率没量）、C635（上一次全量没有机器可读的标记）、C645（崩溃放量全量没有人能派）、C777（五个实验键在 stage-inputs.tsv 没有登记行）、C836（staged-green 挂的树不一定是被判的树）、C890（判法单元闭包少收 import 时执行的模块级语句）。

```

**出处 `.claude/kb/decisions-history.md:14611-14628`（整段抄，未转述）**

```markdown
### 2026-10-19

- 新立 D26（后台整理与放置回收） 已定项 13「GC 的计算放在哪、怎么开、什么节奏」：默认 CPU，挂载参数 `gc_compute` 可开 GPU（只在内存里，不进格式）、`gc_gpu_devices` 点名显卡（不给用全部）、`gc_gpu_memory_limit` 设显存上限，挂着可切、下一批生效；GPU 用不上时挂载与运行中都报错、不降级；起跑是发布后查三水位加定时兜底；节奏是写带宽上限加前台 fsync 让路。D24（后台重活能不能卸给 GPU） 开头射程加一句指过来。
  - **依据**：用户定案。原话：「可以 那么先加入gpu垃圾回收。gpu垃圾回收需要加入。不需要讨论。现在就是 如何配置的问题 --gc-mode=gpu 这种参数加入 还是 其他参数。 另外 后台的gpu gc 按照什么频率进行 这两个很重要。 这个不是默认开启，开启了之后就可以用。」「还有 gpu可以设置vram大小」「还可以设置哪一张显卡 不设置的话 用全部」。弹窗四问的选项：开关形态「挂载参数 + 运行中可切换」；GPU 不可用「两种情况都报错，不降级」；起跑频率「发布后查 + 定时兜底」；运行节奏「带宽预算 + 前台让路」。
  - **改前**：没有这一项；D24（后台重活能不能卸给 GPU） 开头射程只列三个后台重活。
- D26（后台整理与放置回收） 已定项 13 第 2、3、4 条改：`gc_gpu_devices` 与 `gc_gpu_memory_limit` 都是列表，上限按次序一张卡一个、与参与的卡逐个对应，个数不等或某一个数超过那张卡的显存总量就报错；上限不给就能用多少用多少。射程删去「逐卡还是合计没明说」。
  - **依据**：用户定案，原话：「gc_gpu_memory_limit 和 gc_gpu_devices 支持多个设置。但是只要设置了 数字就要对应。并且超出显卡vrm上线 直接报错。  gc_gpu_memory_limit 不给的话，gc时候能用多少用多少。」「比如 gc_gpu_devices 是1 2 3  gc_gpu_memory_limit 可以不设置 但是如果gc_gpu_memory_limit 设置了 就要写三个，不能只写一个，不支持补齐」
  - **改前**：同日上一条写成的 D26（后台整理与放置回收） 已定项 13：上限是一个数、管每张参与的卡，空闲显存不够上限就拒挂。
- D26（后台整理与放置回收） 已定项 13 加第 7 条：GPU 模式值不值按 GC 少占的主机 CPU 时间算，挂钟持平也做，D24（后台重活能不能卸给 GPU） 已定项 3 的挂钟比判据不管 GC；实现之后跑 CPU 模式与 GPU 模式的对照，几档盘容量各跑一遍。
  - **依据**：用户定案，原话：「ssd越大gpu的收益越高  我觉得这点应该是没有什么疑问的。所以 gpu这个功能是需要的。哪怕和cpu持平我觉得都很赚，因为显然省下了 cpu的时钟。 这个做完后需要做个对照组测一下。」「收益随容量涨」只对选段打分那一段成立（候选段数随容量涨），每批的扫描、校验和核对与搬迁与容量无关，这一句由对照组的容量档检验，没写成定案。
  - **改前**：同日写成的 D26（后台整理与放置回收） 已定项 13 只有六条，没有收益口径与对照组。
- D26（后台整理与放置回收） 已定项 13 第 3 条补一句：`gc_gpu_memory_limit` 的每个数是上限、不是预留，用得比它少没有关系（卡上空闲不到这个数也照常跑），任何时刻都不许比它多，攒批、缓冲、暂存区一起算。
  - **依据**：用户定案，原话：「用户设置的gpu vram是我们可以使用的最大 vram 小于没有关系，但是我们使用的不能比这个多」。
  - **改前**：同一项第 3 条只写「GC 在那张卡上占的显存不超过它的那一个数」，没说用得少行不行、算不算缓冲。
- D26（后台整理与放置回收） 已定项 13 加一句：GPU 路径默认写成 WGSL、经 `wgpu` 起，有更好的做法可以在设计轮比过之后换。
  - **依据**：用户定案，原话：「对了 gpu模式下 用wgsl 这个不用下载应该是线程的」「现成的 。 当然如果有更好的 也可以考虑」；`Cargo.lock` 里 `wgpu` 是 30.0.1，checker 档已有六份 `.wgsl`（现查）。
  - **改前**：同日写成的 D26（后台整理与放置回收） 已定项 13 没写 GPU 路径用什么写。

```

**出处 `.claude/kb/checks-owed.md:19-20`（整段抄，未转述）**

```markdown
| # | 简称 | 要拦什么 | 怎么拦（会红的形态） | 前置 | 出处 |
|---|---|---|---|---|---|
```

**出处 `.claude/kb/checks-owed.md:291-291`（整段抄，未转述）**

```markdown
| C412 | 整理的挂钟代价没量过 | **D3（空间分配） 已定项 14 是「后台整理常驻、自动、用户无需知道」这条硬承诺，而它没有任何挂钟数在背书。** D3（空间分配） 已定项 1 的依据引 E74（分配记录的条目数与整理开销），而 E74（分配记录的条目数与整理开销） 量的是整理路径重写多少个节点（载体强制付出多少），不是它要花多少墙钟；D3（空间分配） 瘦身前的正文自陈挂钟那一问仍欠实现，那句话已随 2026-09-20 的瘦身移出正文、在决策变更史里。⇒ 整理跑一轮要多久、会不会跟前台抢 I/O，一个数都没有 | 建一个计数实验或给 E74（分配记录的条目数与整理开销） 补一轮，量一批整理的挂钟，口径写明设备、负载、批大小，断言钉绝对值而不是只比相对倍数。判别力自证：把批大小翻倍，挂钟必须跟着涨 | D26（后台整理与放置回收） 的整理策略未定项 8 / 9 / 10 还开着，先定量哪一条策略；`crates/` 里没有整理（`grep -rn 整理 crates --include=*.rs` 命中 12 行、4 个文件，都是注释、断言消息与字段名里的字（系统运行配置那三个整理水位恒取内置默认 0），没有整理路径） | 2026-09-20 D3（空间分配） 瘦身现查：瘦身前那一版第 90 行自陈挂钟那一问仍欠实现，而欠账表里没有对应编号 |
```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:8-23`（整段抄，未转述）**

```markdown

**D26（后台整理与放置回收）的分项只有一套编号**，与「未定项」一节连号，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **整理的触发判据** | 三水位任一越线起跑；停机取 `R` 本地谓词「删意图那一刻 `R` 按记录全空」，签发前判空洞够不够，无产出就不签下一条；`R` 在意图期内不可作分配目的地 **状态：已定。** **进度：未实现。** |
| 2 | **整理的事务形态** | 不新增 `CommitStep` 成员；意图的目标态写成「`R` 内不存在分配代 ≤ `c0` 且未释放的落点」 **状态：已定。** **进度：未实现。** |
| 3 | **整理自己要不要申请空间** | 走常规准入；第一版就留常驻底额，做成准入的第二个合取式；底额取「尽可能小」 **状态：已定。** **进度：未实现。** |
| 4 | **结构抗老化取什么形态** | 不进格式：不存组身份、不动索引节点内部布局，组只从 key 算；政策那一半落回 D3（空间分配） 已定项 8 与已定项 1 **状态：已定。** **进度：未实现。** |
| 5 | **整理与记账的交互** | 搬迁不改 `birth`；分配记录树源侧与目的地侧各一个式子；deadlist 条目按中央映射 key 记 **状态：已定。** **进度：未实现。** |
| 6 | **走没走整理的运行时观测点** | 观测量六个，触发用的三个持久化进记账树，整理进度不存（删意图前对整个 `R` 全量核目标态），不靠挂载后盲跑或全扫 **状态：已定。** **进度：未实现。** |
| 7 | **回收分不分线** | 分两条线：先自建 GC，盘的 FTL 支持 GC 时交给盘、本工程只打标记发指令，走哪条线按 feature bit 处理 **状态：已定。** **进度：未实现。** |
| 11 | **碎片的答案押在哪** | 两条腿叠加、不是二选一；NVMe FDP / ZNS 降为「有就用的加速」，不再是答案 **状态：已定。** **进度：无对象。** |
| 12 | **腿一的形态** | 由三条已定纪律框死：第三格无戒律、无界重活走意图分批、搬迁单位是整个对象重落一组 **状态：已定。** **进度：无对象。** |
| 13 | **后台任务的计算放在哪、怎么开、什么节奏** | 默认 CPU；一组挂载参数管全部后台任务：开 GPU 则有 GPU 路径的都走 GPU，可点名显卡、设显存上限，挂着可切；结果两份核比对；用不上报错不降级；水位加定时兜底；按带宽预算跑、前台让路 **状态：已定。** **进度：未实现。** |

```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:254-262`（整段抄，未转述）**

```markdown

| # | 分项 | 状态 |
|---|---|---|
| 8 | **回收两条线的切换判据** | 怎么测盘的 FTL 支不支持 GC、在哪一刻测、测的结果存不存盘，都没说；用户定近期不论证。改新池新建文件的字节：否（2026-09-20，依据：回收与整理都在后台跑，新池新建文件不跑它们，理由见表下那一段） 动不动格式：界不定（2026-09-21，依据：题面逐字含「测的结果存不存盘」，而这一问本身还没定——存就要在盘上给它一个字段，不存就一个字节都不动） 2026-10-21 复核过，仍然开着，因为同文件新增的已定项 13 只管后台任务的计算放在哪、怎么开、什么节奏，没答怎么测盘的 FTL 支不支持 GC；用户定近期不论证。 **状态：未定。** **进度：未实现。** |
| 9 | **回收两条线的射程** | 走交给盘那条线时腿一里哪几件活停下没说；用户定近期不论证。改新池新建文件的字节：否（2026-09-20，依据：同未定项 8，停哪几件活只改后台行为） 动不动格式：不动（2026-09-21，依据：题面是「腿一里哪几件活停下」，停的是后台行为；D26（后台整理与放置回收） 已定项 6 已把三个量定为记账树里的统计量，停活不改它们的宽度与含义） 2026-10-21 复核过，仍然开着，因为同文件新增的已定项 13 只管后台任务的计算放在哪、怎么开、什么节奏，没答交给盘那条线时哪几件活停下；用户定近期不论证。 **状态：未定。** **进度：未实现。** |
| 10 | **回收线的 feature bit 用哪一档、发给盘的指令是什么** | 先记成 feature bit，档位与指令留未定；用户定近期不论证。改新池新建文件的字节：否（2026-09-20，依据：新池新建文件写出的系统配置 feature bit 三档今天全 0，加一档不改已写出的字节） 动不动格式：动（2026-09-21，依据：feature bit 就是系统配置那三张位图里的位；题面的「另占一位还是跟着 D12（目标介质） 已定项 1 的布局位走」两种答案对应不同的位分配） 2026-10-21 复核过，仍然开着，因为同文件新增的已定项 13 只管后台任务的计算放在哪、怎么开、什么节奏（它的挂载参数不占 feature bit），没答回收线占哪一档、发给盘什么指令；用户定近期不论证。 **状态：未定。** **进度：未实现。** |

未定项 8、9、10 改新池新建文件的字节：否——回收与整理都在后台跑，新池新建文件不跑它们（D12（目标介质） 已定项 5「与 D26（后台整理与放置回收） 的连锁」一段逐字「整理仍是后台的，第一个可运行目标那一轮不跑它」）；第 10 项另加一条，先做自建 GC、交给盘那条线近期不实现，第一版不写这一位，位定下之后重判。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:8-16`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **做不做** | 暂缓，不排期；暂缓不等于否掉，算力受限那一格仍开着，GPU 这条线不再继续测 **状态：已定。** **进度：未实现。** |
| 2 | **三条硬约束** | 绝不能成为必需（一个比特都不许进格式）、不许进前台路径、卸出去的结果必须复核，GPU 与 CPU 是两份实现 **状态：已定。** **进度：无对象。** |
| 3 | **判据取什么形式** | 取 CPU 侧的多核饱和吞吐与 GPU 的两个传输地板比，不是「单核 × 核数」，也不是「有没有硬件指令」 **状态：已定。** **进度：无对象。** |
| 4 | **重开判据** | 四个闸 G24.1（缺硬件指令） 到 G24.4（GPUDirect 可用），前两个必须同时成立 **状态：已定。** **进度：无对象。** |
| 5 | **合法性的来源** | 三个候选全在 `.claude/rules/fs-design.md` 第三格；某个场景挪进第一格，D24（后台重活能不能卸给 GPU） 对它当场失效 **状态：已定。** **进度：无对象。** |

```

**出处 `.claude/kb/decisions/13-验证路线.md:8-28`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **crash refinement 先验哪一块** | 先验「一次发布 + 一次崩溃 + 一次恢复」这个封闭单元，第一个里程碑是任一 `jsn` 连续前缀不切开发布；挂载透明另立第四条性质；原候选 I-6.5（nonce 水位单调） 撤下 **状态：已定。** **进度：未实现。** |
| 2 | **事务层承不承诺串行提交** | 先串行，越过交点再开并行；将来并行提交时记录按事务号顺序追加 **状态：已定。** **进度：已实现。** |
| 3 | **自写参照实现（RefFS 形态）做到什么程度** | 做到极致取语义级：只在内存里按 POSIX 操作走，自己定义持久与崩溃后允许的状态，没有盘上格式；照 kb 另起、单独一个包、只收白名单输入 **状态：已定。** **进度：已实现。** |
| 4 | **崩溃点重放的崩溃状态集合怎么定义** | 写流按设备切段，段里每块有写的盘都被自己的屏障或 FUA 放行才关段；段内每次整写各取几态、任意组合，原地覆写多一态「新旧都读不出」；没持久放旧字节且真生成镜像；双写按设备各算 **状态：已定。** **进度：已实现。** |
| 5 | **checker 与被测实现允许共享什么** | 只共享一份从 kb 生成的常量模块，其余各写一份；判定宽度由 checker 自己探测再与系统配置比对，探不到报「声明值，未探测」 **状态：已定。** **进度：已实现。** |
| 6 | **三个互相独立的 oracle** | 内存模型对拍、独立解析器 + checker、独立规约执行器三个 oracle 各判一样；独立性要靠「三 oracle 合谋测试」验证，不许自称 **状态：已定。** **进度：未实现。** |
| 7 | **O2（独立解析器 + checker） 判什么** | O2（独立解析器 + checker） 的定义域是单个镜像，只判集合成员；需要第二个输入的检查归记录核对器（入参含枚举器给的持久集合），故意不给编号 **状态：已定。** **进度：已实现。** |
| 8 | **自研的两样** | Merkle 根链差分（相邻两代根递归比较，指针没变处不下钻）与代际增量 scrub（只遍历 birth generation 高于水位的块） **状态：已定。** **进度：未实现。** |
| 9 | **崩溃点重放怎么分层** | 三层都精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore 慢慢开；里程碑收尾与判器单元有变的那批提交前弹窗请用户跑全量档，红或不跑照样提交、记未结；先保失败可复现 **状态：已定。** **进度：已实现。** |
| 10 | **真正必须精确覆盖的 N 是什么** | 提交协议结构的等价类数，不是布局套数；落地是一个封闭的提交步骤枚举加穷尽 match **状态：已定。** **进度：已实现。** |
| 11 | **ignore 的节点什么时候开** | 否决按欠账轮转的账本；留下三条改管开的时机：确定性的覆盖记录、`T1`（常规层 挂钟预算） 是输入、判别力与覆盖分开检查 **状态：已定。** **进度：无对象。** |
| 12 | **冲突 1：checker 即规范与校验路径不许共享工具** | checker 做遍历与扫描两个方向，对同一条不变量各自独立判，共享的只有格式解析 **状态：已定。** **进度：已实现。** |
| 13 | **冲突 2：refinement 的 spec 与对拍的 model** | spec 与 model 在表述层面就写得不同；同源性要观测，≥ 5 次不一致而判 spec 错 0 次就合并、预算移给穷举 **状态：已定。** **进度：无对象。** |
| 14 | **与其他决策的连锁** | 哈希与 AEAD 在形式验证里只能当 uninterpreted function；日志结构三层绑架符号执行；checker 不必解析消息语义 **状态：已定。** **进度：无对象。** |
| 15 | **验证代码分两档** | harness 档（`singlefs-harness`）是单元与集成测试、随时跑；checker 档（`singlefs-checker-tier`）默认只在提交时跑快档，全量用户要求时跑 **状态：已定。** **进度：已实现。** |
| 16 | **按段登记的段怎么分片、态数用多宽** | 登记段整域装不进 u64 就按片依次判，片大小默认 2^32 态、片起点是库里的续点；态数一律用 1280 位定宽整数、算术全 checked；登记表只经环境变量读 **状态：已定。** **进度：未实现。** |
| 17 | **判定复用的单元是什么** | 剪枝的单元是函数：一份判定沿不沿用只看它依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本；按提交或树算改动、整文件原文、文件级与包级指纹、手写版本号、路径名字进键、整道标记一律不许 **状态：已定。** **进度：未实现。** |

```
