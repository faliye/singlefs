# 里程碑四预开发整段：代码轮第二轮正文（2026-10-16）

<!-- doc-lint:not-numbers R1 R2 R3 R4 R5 R6 R7 R8 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 S2 S3 S7 S8 G1 G3 G4 G5 G9 M4 -->

## 一、题面

第一轮判决 `research/prompts/m4-segment-code-r1-main-verification.md` 打中 T5、T4、T6、T11，缺 T2 强制开关、T9 对拍输入域，改法 F1–F10 都被攻过零轮。两批修复已打进工作树 `~/.local/share/singlefs/work/m4-predev`（修复批一甲乙丙、修复批二甲二丙二丁与 u32 加宽参考补丁；交回与主 agent 的判定在 `records/2026-10-13-里程碑四预开发.md` 第三节对应的行）。修复批一合入后验证全绿（`✓ 测试全绿（218 个测试二进制，共 2327 条测试）`，`~/.local/share/singlefs/work/m4-verify-logs/merge-verify-fix-r1.log`）；修复批二的合入后验证、崩溃与变异轻型还没跑（等卡与内存），它们的红归验证段，不算这一轮打中。

这一轮攻两样：第一轮的修补本身（F1–F6 落在产品代码里的那几处与实现员、主 agent 取的零轮读法），以及第一轮没打中的格再抽一次样（第一轮攻方只抽了一次，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」）。

被判的产品代码与第一轮相同的 20 份（sha256 在 `research/prompts/m4-segment-code-r2-snapshot/crates-sha256.txt`），修复批改到的是 `crates/singlefs-core/src/mount.rs`、`transaction.rs`、`mounted_session.rs`、`compaction_plan.rs`、`crates/singlefs-reference-filesystem/src/lib.rs`，改动全文 `research/prompts/_m4-segment-code-r2-diff.md`（相对第一轮冻结副本）。决策：D26（后台整理与放置回收）、D5（快照 / 空间记账机制）、D8（核心索引结构）、D16（发布语义）。

## 二、实现今天的样子（主 agent 的观测，2026-10-16）

- 冻结副本 `~/.local/share/singlefs/work/m4-segment-code-r2/repo/`（工作树打完两批修复之后的 `crates/`、`Cargo.toml`、`Cargo.lock`）。
- 修复批的读法（实现员报告与主 agent 判定，都被攻过零轮）：
  - F1（`~/.local/share/singlefs/work/m4-fix-r1-core-writer/patch/report.md`）：判决字面「中央映射树取目标版的」走不通（那几个节点在现行版的账里多半已释放），改成映射树节点照现行版 COW、条目位置与目标版不同的 key 所在叶连同到根整条重写，走整理一批同一条路；比重另开一组 `COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`。主 agent 定：回退不动 logged_ops，活意图与暂停态照旧、不重新排他。
  - F2：腾空 R 的那一批同发布删意图；删意图核不过照这一批不删意图再发（只供测试的开关 `RegionCheckOfTheIntentDeletion` 强制进入，`~/.local/share/singlefs/work/m4-fix-r2-core-writer/patch/report.md`）；这一支发出的那一版没搬单元而意图照抄，模型 `answer_compaction_publish` 的 `(false, true, …)` 臂判不合（条款没写）；「最后一批删意图核不过、照抄再发」同形一支没有强制用例。
  - F3：签发那次发布先算账，`segs0` 与记账第 11 项不等就换成记账的值重算再发，分配器上的活意图一并换。
  - F4：只供测试的寿命判定开关挂在 `PoolWriter` 上。
  - F5：文件集合为空时建快照放开（核心、RefFS、模型三处）。
  - F6（`~/.local/share/singlefs/work/m4-fix-r1-reffs-model-writer/patch/report.md`）：恢复后快照集合按全部对上的允许状态取，公开接口换成 `allowed_snapshot_sets()` / `allowed_reads_of_the_snapshot()`；实现员自补一处读法：实现做了的操作与读快照读出的内容排除说不通的快照集合。
- 第一轮判决里交下来、还开着的：计划里的单元都被这次发布顺带重写、一个没搬而 R 还不空的那一形（条款没写、没有取样点）；T5 多出来的重写越不越过 `ckpt_cost`、零个全空段、合法镜像触发 `TwoRecordsMovableUnderOneMappingKey` 三个取样点没造出；T8 少发 discard 不补（记欠账）；T10 坏 logged_ops 条目拒可写挂载（主 agent 定不改）；T7 产品路径上整理不起跑而合取 7 开着（用户定维持）。
- 已知、归验证段或下一批、报出来不算打中的：GPU 判器内核里与池级 checker「孩子一律挂父单元」同形的那一处还没跟上（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 挂父单元那一段）；段登记表重登的段 1 摘要待崩溃轻型核；主仓那份形式化门禁点不中 `compaction_plan_proofs` 子模块（合主仓前处理）；变异轻型还没判清的行。后果超出这里写的范围时算打中。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### R1　回退之后的映射（F1）
`mount.rs` 造回退那一版的上一版与 `transaction.rs` 里 F1 接的重写路径：只重写位置与目标版不同的 key 所在叶到根，这一取法在这几种形状下映射与释放集、复活集是否一致——目标版之后有过多批整理、整理搬的单元又被覆盖写过、回退目标落在意图半路、两块盘不对称、映射树只有一片叶；回退不动 logged_ops 时，目标版早于签发、现行版带活意图，回退之后 R 排他与意图指的 R 还对不对得上，暂停的意图之后能不能再起；多出来的重写过不过准入。

### R2　腾空 R 的那一批与删意图（F2）
`transaction.rs` / `mounted_session.rs` 的删意图判定、`RegionCheckOfTheIntentDeletion`：「这一批之后 R 按记录全空」怎么判、核不过那一支在产品路径上真走不到吗；模型 `(false, true, …)` 臂判不合的那一形是实现错还是模型错（对判决 G4「目标态到达与最后一批同发布删」）；同形另一支；`batches_published` 的计数。

### R3　签发那次发布的 `segs0`（F3）
签发先算账、不等就换值重算再发：两次算账之间分配器与意图的状态换得干不干净，重算会不会让准入、根槽、`B_gc` 多算一次；`segs0` 与 G1 字段表定义句、停机谓词「全局差值」逐字对。

### R4　强制开关（F4）
寿命判定开关挂在 `PoolWriter` 上：平常路径有没有一条能碰到它；强制立即释放与强制进 deadlist 两臂与 S3 的判定是不是只差判定那一处、别的副作用同不同。

### R5　空文件集合上的快照（F5）
核心、RefFS、模型三处：建快照之后再写文件、读快照读到空、删快照、回退拒、上限 10 数不数它、建快照那次发布崩在中途，三处的允许结局是否同口径。

### R6　RefFS 的快照允许集合（F6）
`crates/singlefs-reference-filesystem/src/lib.rs` 改过的允许集合与实现员自补的那一处排除读法：会不会把一个合法的崩溃结局排除掉（核心层真能落到、RefFS 判不允许），或把不合法的放进来；新公开接口在 harness 翻译里的每一处用法。

### R7　第一轮交下来的开着的形状
计划里的单元都被顺带重写、一个没搬而 R 还不空；多出来的重写越过 `ckpt_cost`；零个全空段；合法镜像触发 `TwoRecordsMovableUnderOneMappingKey`。能造出取样点就造，造不出写清楚为什么。

### R8　第一轮没打中的格再抽一次样
第一轮的 T1（建快照）、T2（释放分流与 deadlist）、T3（删快照）、T7（R 排他与合取 7）、T9（区域扫描与计划）、T10（恢复读侧）照第一轮正文 `research/prompts/_m4-segment-code-r1-body.md` 第三节同名格的题再攻，换攻击面：不重复第一轮攻方报告 `research/prompts/m4-segment-code-r1-opus-output.md` 里已写的取样办法与种子段。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | R1–R6 与第一轮判决全部判定 | 复核第一轮判决：打中的四格是不是真的错、F1–F6 改到的地方够不够得着打中的那一形、F1 不照判决字面取的那一法是否仍守住 G3「映射是唯一权威」；不改代码的 T7、T8、T10 三处判定站不站得住；每条修补是不是同样打中别的替代改法；有替被判出局一方辩护的就辩 |
| 云端攻方（Opus） | R1–R8 | 在冻结副本的拷贝上造反例，攻修补本身与零轮读法；R8 换攻击面再抽一次样；每个打中给可复现的用例或变异（改哪一行、哪条断言红）；已知清单里的不算打中，后果超出清单写的范围时算 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷编译目录当种子之前先 `find crates -type f -exec touch {} +`。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；用自己的编译目录。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU，不碰 `crates/singlefs-checker-tier/`。
