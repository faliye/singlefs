# m4-gpugc-publish-growth-r1 判决：核心层每次发布的代价随版本涨，定位、外推与改法（2026-11-06）

被判的文件（开工快照 `research/prompts/m4-gpugc-publish-growth-r1-snapshot/judged-sha256.txt`）：`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/allocation_record_tree.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs`。正文 `research/prompts/_m4-gpugc-publish-growth-r1-body.md`，背景 `research/prompts/_m4-gpugc-publish-growth-r1-background.md`。

## 一、腿与核查

| 腿 | 报告 | 结局 |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m4-gpugc-publish-growth-r1-sonnet-output.md`（sha256 `86e5812f…`），模型 `research/prompts/m4-gpugc-publish-growth-r1-sonnet-model/` | Q1–Q3 逐格有结论 |
| 攻方（Opus） | `research/prompts/m4-gpugc-publish-growth-r1-opus-output.md`（sha256 `8ca60aae…`），模型 `research/prompts/m4-gpugc-publish-growth-r1-opus-model/` | Q1–Q3 逐格有结论 |

正文第二节写「这四份核心层文件……此刻与 HEAD 相同（`git diff --stat` 输出为空）」，这一句错了（攻方腿报的，主 agent 现查坐实）：那条命令比的是暂存区，`git diff --stat HEAD` 对 `allocator.rs`、`mounted_session.rs`、`transaction.rs` 三份是「1269 insertions(+), 160 deletions(-)」。腿读的、探针测的都是此刻的工作树（与暂存区相同），判决引的行都在倒推出的快照树里核过，这一句的错不改任何一格的结论。

核查（照 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」）：
1. 开工快照倒推：任务 7817，`python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-gpugc-publish-growth-r1 --out /home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-main-check/m4-gpugc-publish-growth-r1-tree --repository /home/fy5090/code/singlefs`，退 0。
2. 引文：`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-publish-growth-r1-sonnet-output.md research/prompts/m4-gpugc-publish-growth-r1-opus-output.md --root <上面那棵树> --background research/prompts/_m4-gpugc-publish-growth-r1-background.md`，末行「✓ 核了 93 处引文、84 处裸行号，对上 74 处，没判 103 处（其中裸行号没有引文可比 55 处；逐处列在上面）」，没有一处对不上。判决要引的行主 agent 在那棵树里逐处人工核过：
   - `transaction.rs:12180`「steps_published.push(current.clone());」（删除批每一步整版深拷，探针没包）；
   - `mounted_session.rs:3304`–`:3305`「let mut copy = allocator.clone();」「let mut rehearsed = start.clone();」与 `:3339`「versions_published.push(next.clone());」（抬 F 推环那一支，探针没包）；
   - `transaction.rs:1579`–`:1596` `persist_the_publish_or_freeze_it` 文档「落盘那几步里失败……分配器换回发布之前的样子，这次发布冻结在它上面等原样重发」与 `:1595`–`:1596`「let allocator_after_the_publish = std::mem::replace(allocator, allocator_before_this_publish);」；
   - `.claude/rules/fs-design.md` 第 19 行「**线划在「谁消费这个数」，不划在「用什么机制算出来」。**」、第 29 行「⚠️ **一个量落不落在第一格，按这一格自己的判据判：「被问到那一刻有没有答案」，不按名词清单判。**」、第 33–34 行；
   - `.claude/kb/checks-owed.md` 第 265 行 C381（根已落盘之后发布失败，分配器仍退回）。
3. 复跑（模型目录拷进 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-main-check/`，在副本里交）：
   - 正推腿模型，任务 7812：「name=rerun_done differing_files=0」，九份 `status=same`（量的）。
   - 正推腿甲档只改类型的编译核，任务 7813：「name=cargo_check_exit code=101」「name=error_lines count=14」，分布 transaction.rs 8、mount.rs 5、recovery.rs 1（量的）。
   - 攻方腿模型，任务 7816：三行 `same=yes`；「name=model_ratio units=3386 owned_over_shared=555.0」「name=model_ratio units=6724 owned_over_shared=818.2」「name=model_tables_left units=6724 records=67240 two_copies_ms=0.5609 two_drops_ms=0.3382 total_ms=0.8991」（量的，量的是攻方腿自写的小模型，不是核心层本身）。
   - 缺页对照（正推腿「要主 agent 跑的命令」第 2 条，探针副本，2288MiB 档 P=2048，各三次；判法照正推腿报告第 323 行跑前写死的「无归还组的交错写窗口每次缺页降到 < 10% 基线，且 `construction_seconds` 低于基线最小值的 2/3 ⇒ 缺页是大头；缺页降了而秒数不变 ⇒ 缺页不是；缺页没降 ⇒ 变量没起作用，这趟作废」）。无归还组命令前加 `env GLIBC_TUNABLES=glibc.malloc.trim_threshold=17179869184:glibc.malloc.top_pad=1073741824`，基线不加，其余逐字相同。日志与产物在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-main-check/probe-runs/`（量的）：

     | 组 | 任务 | 台 | `construction_seconds` | `phase=interleaved_half` 秒 | 交错写八个窗口每窗口（128 次用户改动）缺页增量 | 末窗口 `clone_into_versions_published` 秒 |
     |---|---|---|---|---|---|---|
     | 无归还 | 7815 | 本机 | 141.766 | 118.804 | 19624–23083 | 2.774488 |
     | 无归还 | 7850 | 第二台 | 119.009 | 99.061 | 19623–23107 | 2.835261 |
     | 无归还 | 7854 | 本机 | 138.561 | 114.815 | 19368–23060 | 2.805875 |
     | 基线 | 7818 | 本机 | 215.444 | 187.379 | 2678484–6200830 | 9.835013 |
     | 基线 | 7853 | 本机 | 268.622 | 241.124 | 2678722–6195173 | 11.252487 |
     | 基线 | 7887 | 本机 | 220.923 | 195.777 | 2679331–6201958 | 15.586871 |

     缺页增量取日志里相邻两行 `name=probe_window` 的 `minor_faults_total` 之差（主 agent 现算）。判：无归还组每窗口缺页是基线的 0.3%–0.9%，过「< 10%」；基线最小值 215.444 的 2/3 是 143.63，无归还组三次 119.009–141.766 都在它下面，只算本机两次是 138.561–141.766，同样在下面。照跑前判法：**缺页是大头**。离线最近的一次（7815，141.766）只差 1.3%，这一格过得很窄。

## 二、判决

照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」逐格判。「量的」指上面核查第 3 条主 agent 复跑或现读的读数，「推的」指读码与算术，没量过。

### Q1　定位与外推

| 格 | 两腿 | 判决 |
|---|---|---|
| Q1.1 四格包没包对行 | 正推：部分成立，`clone_into_versions_published` 包的是整个 `TransactionOutput`，口径对齐后深拷组占 48.7%–51.5%（量的）；攻方：打中（部分），同一路径上 `transaction.rs:12180`、`mounted_session.rs:3304`–`:3305`、`:3339` 的整版深拷没包，删垫片那一段 `publish_user_change_total` 有 43.4%–56.9% 落在直接子格之外（量的） | **部分成立**。交错写那一段「深拷组约一半」成立（48.7%–51.5%）；这一格包的是整版 `TransactionOutput`，不只是单元字节（`transaction.rs:3241`、`:3242`、`:3244`、`:3305` 四个字段随 U 或 R 涨）。删垫片那一段探针漏包，那一段的拆法不成立。漏包的几行主 agent 在快照树里核过（核查第 2 条） |
| Q1.1 阳性对照 | 两腿一致：证计时，不证覆盖；对照单价只有真拷贝的 1/2.3–1/4（有缺页窗口） | **证明计时可信，证明不了探针罩住了全部深拷**。漏包的几行就是反例 |
| Q1.1 那约 15% | 正推：13.2% 量的，落在 `publish_admitted` 与固定点；攻方：点到行，中央映射树整份重装、`mapped_units` 整份重装与比对、元数据节点两次深拷 | **落在 `publish_admitted` 与准入之前的固定点**（13.2% 量的，逐行分摊推的）。甲档碰不到这一截 |
| Q1.2 「次数不涨，涨的是每次的代价」 | 正推：扫记录组基本成立、深拷组不成立；攻方：字面不成立，取落点、释放、`record_for` 每次进的次数从 56.5 / 32.9 / 41.2 涨到 89.1 / 52.6 / 71.5 | **字面不成立**（量的）。分配器查找次数随 U 约成对数涨，`carried_unit_clone` 次数 ∝ U |
| Q1.2 「总挂钟高于 P² 是缺页造成」 | 正推：推的，读数不支撑（缺页约占末窗口 22%）；攻方：能分辨，读数只撑一半 | **缺页是 P=2048 建池挂钟的大头（量的，核查第 3 条缺页对照）**：关掉 glibc 归还后建池秒数从 215.444–268.622 降到 119.009–141.766，末窗口 `clone_into_versions_published` 从 9.84–15.59 秒降到 2.77–2.84 秒，交错写每窗口缺页从 268 万–620 万降到 1.9 万–2.3 万。正推腿「约占 22%」那一估算被这组读数推翻；两腿的「单价在缺页出现之前已因缓存装不下涨了」那一半不受影响。这组读数只在 P=2048 一个点上量，**答的是「缺页占多大份额」，不答「高于 P² 的那一截次方是不是缺页造成」**；后一句仍是推的 |
| Q1.3 外推 | 两腿一致：外推越位，最大量到 P=4096，目标在外面 8.94 倍；8.8–111 h（推的） | **推的，区间 8.8–111 h**，拟合在今天的代码与 glibc 缺省设置上取点（P=1024、2048、4096 三点，`construction-timing-2288MiB-each-device-copies.out`）。「几十小时」落在区间里，但不由读数定。缺页对照之后，这一区间只对「不改核心层、不改 glibc 设置」成立 |

### Q2　改法

| 格 | 两腿 | 判决 |
|---|---|---|
| Q2.1 甲档能不能只改类型 | 正推：做得到（推的），14 处构造点 + 若干比较读取点，不是只改类型；攻方：不改语义（推的），这一格没打中 | **做得到，要改的不只是类型**：复跑任务 7813「name=error_lines count=14」（transaction.rs 8、mount.rs 5、recovery.rs 1，量的），低于正推腿自己写的推翻线 40。`PartialEq`、录制流、写盘字节、checker 读的东西不变是推的，两腿一致 |
| Q2.1 「系数小约三个量级」 | 两腿一致：不成立，只对单元字节那一份成立；整组缩 14–28 倍，整次发布约快一倍到两倍（推的） | **不成立**。甲档之后仍要照拷映射 key、分配记录、`mapped_units`，攻方模型复跑任务 7816 的比值 555.0（3386 单元）、818.2（6724 单元）量的是攻方腿自写的小模型，不是核心层。缺页对照另外说明（推的）：整版深拷每次分配、释放一整版字节是缺页的来源，甲档拿掉这份字节复制，缺页那一截跟着大半消失 |
| Q2.1 甲档之后剩下随 U 或 R 涨的项 | 正推：`publish_admitted`、扫记录组；攻方：调查员漏了六项 | **剩下的有**：`publish_admitted` 里随 U 的循环、准入之前固定点每轮整份装 `mapped_units`（`transaction.rs:9083`）并建身份索引、扫分配记录组（随 R）、中央映射树整份重装、元数据节点两次深拷。总挂钟仍 ∝ U 乘发布次数（推的） |
| Q2.2 乙档 | 两腿一致：单独做不到「每次发布不随 U 涨」，剩固定点与中央映射树；按身份有序会改 `units` 的次序与下标语义 | **不是唯一、单独也不够**（推的）。乙档里 `Arc<TransactionOutput>` 那一小块（版本列表与挂载交回值共享同一份）可以独立做 |
| Q2.3 ① 有序结构 | 两腿一致：做得到，不碰 D3（空间分配） 已定项 7 的盘上编码；攻方另报 `records()` 交切片、次序从插入序变键序，157 处调用 | **做得到（推的）**，不碰盘上格式；改次序的那一面要逐处核调用方 |
| Q2.3 ② 碰过的叶作为副产品 | 正推：做得到，买回最多（约 8.5 ms），漏一处写点即静默出错，要保留全量参照；攻方：做得到，挪 `release` 下的 `assert!` 会改失败路径 | **做得到（推的），两个前提**：全量遍历保留为 checker 一侧的参照（`.claude/rules/fs-design.md` 第 34 行「审计与被审计用同一段代码」被禁止）；`assert!` 照原失败路径留着，不挪 |
| Q2.3 ③ 分配器拷贝换回滚日志 | 正推：做得到、最难，要求「换回去」逐位相同；攻方：打中，D16（发布语义） 已定项 17 不要求「换回」，C381 恰是「根已落盘之后发布失败，分配器仍退回」，冻结重发那一支要的是发布之后的分配器 | **按调查员的写法做不到**：`transaction.rs:1579`–`:1596` 落盘失败那一支要的是发布之后那一份分配器（「let allocator_after_the_publish = std::mem::replace(allocator, allocator_before_this_publish);」），回滚日志只给得出之前那一份。调查员把 D16（发布语义） 已定项 17 读反了。要做，得另外定冻结重发那一支拿什么，与 C381 一起定 |
| Q2.4 常驻多出的那一截 | 正推：同一个根、两种持有（推的）；攻方：量不出拆法，`VmRSS` 含 glibc 留着没还的空闲堆 | **拆不开**（量的现象，推的归因）。缺页对照里无归还组末窗口常驻 1072750592–1074712576 字节、基线 970604544–971452416 字节，不还的空闲堆确实进常驻，`resident_bytes` 不能拿来拆活着的版本与留着的空闲 |

### Q3　归类

| 组 | 判决 |
|---|---|
| 扫分配记录组 | **落第一行**，两腿一致：它们在分配、释放判定与准入路径上，按 `.claude/rules/fs-design.md` 第 23 行那一行的两格「**运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定）」「**不许**，且代价不许随盘容量增长」与第 29 行「被问到那一刻有没有答案」判 |
| 深拷组 | **不落第一行**，两腿一致：深拷在准入之后（`transaction.rs:12861` 判决与准入走完、`:12970` 才进 `publish_admitted`，整版深拷在落盘之后 `mounted_session.rs:3067`），不产出被决策消费的数；第 19 行「**线划在「谁消费这个数」，不划在「用什么机制算出来」。**」按字面不罩它。它随 U 涨的代价不归这张表管，归的是发布挂钟本身 |
| 准入之前固定点每轮整份装 `mapped_units` 并建身份索引 | **落第一行（推的）**，攻方腿新报、正推腿没归类：`transaction.rs:9083` 在 `plan.resolve` 里，结果经 `transaction.rs:12721`「let settled = settle_the_allocation_record_tree(」交给准入，是在决策那一刻按 U 现算的；主 agent 在快照树里核过这两行 |

### 交用户的

- 核心层改法（甲档、乙档里的 `Arc<TransactionOutput>`、扫记录组 ①②、固定点、回滚日志）怎么排、改不改，交用户定。各档的效果都是推的；只有「缺页是 P=2048 建池挂钟的大头」与甲档的编译错数是量的。
- 甲档、②、`Arc<TransactionOutput>` 都是腿提的形态，被攻过零轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 7 | 不影响 | 2026-11-06 不受影响：扫记录组的三样改法都只改内存里的结构与次序，不碰分配记录条目的盘上编码 |
| D16（发布语义） 已定项 17 | 不影响 | 2026-11-06 不受影响：判决只指出调查员读反了它，条款本身不动；冻结重发那一支与 C381 的关系照旧挂在 C381 |
| D8（核心索引结构） | 不影响 | 2026-11-06 不受影响：背景材料只作为「不碰盘上格式」的出处引到，改法都不碰索引的盘上结构 |
