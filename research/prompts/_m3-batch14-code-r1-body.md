# 删除批第 14 批合入的代码：代码轮第一轮正文（2026-10-03）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 GA GB GC U8 K1 K2 K3 -->

## 一、题面

删除批代码轮第三轮（`research/prompts/m3-unlink-code-r3-main-verification.md`）判丙1 在单盘幸存时打穿。用户 2026-10-03 弹窗定了两条：丙1 单盘那一问选「甲 GA（推荐）」，Q-F 时间字段那一问选「乙 意图带时间、三格同改」（岔路单 `research/prompts/m3-unlink-code-r3-forks.md`）。这两条连同那份判决第三节的改法，加上回扫旧树值（12 棵树、树 ID 从 65536 起）落在 `crates/` 的那一半，由第 14 批两位实现员写进 `crates/`，已合入主仓（`records/2026-10-01-删除与并发事务.md` 2026-10-03 那几条）。这一轮判两件事：代码做的是不是判决与用户定案说的；哪一格会错。GA 只在上一轮两条攻方腿的副本上量过，这是它在入库代码上第一次被攻。

改动全文在 `research/prompts/_m3-batch14-code-r1-diff.md`（两份补丁原样，加变异表合入前后的差）。冻结副本 `/tmp/claude-1000/m3-batch14-code-r1/tree/`，被判的 21 份文件的 sha256 在 `research/prompts/m3-batch14-code-r1-snapshot/crates-sha256.txt`。实现员报告 `/tmp/claude-1000/m3-batch14/p/report.md`、`/tmp/claude-1000/m3-batch14/q/report.md`，结论句只当线索，以代码为准。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定案 |
|---|---|---|---|
| N1 | GA：卸载那一格回收时，先当场放开释放代不高于「各盘最新有效根 txg 取小」的那一截，其余扣到生效；有一块盘一条有效根都读不出时整截扣到生效；准入那一格不变 | `crates/singlefs-core/src/mount.rs` 第 2343–2353 行（`reuse_of_the_reclaimed_slots` 与新枚举）、第 2368 行（`newest_valid_root_txg_on_every_device`）、第 2791 行起 `raise_the_floor_through`（第 2954 行那一臂） | D16（发布语义） 已定项 1「抬 F 那一串」「正常卸载」两行（第二次写回已写成 GA）；用户「甲 GA（推荐）」 |
| N2 | GC：单盘幸存读回用例 h2、h4 两形 | `crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs` 第 250、318 行 | 上一轮判决第三节第 2 条 |
| N3 | Q-F 乙：`UserChange::Unlink` 带 `write_time_seconds`，意图那次写进三个时间秒，各批照抄上一版记录；随机历史按步号取时间；十份 harness 测试传记录现有的时间 | `crates/singlefs-core/src/mounted_session.rs` 第 60–66 行；`crates/singlefs-core/src/transaction.rs` 第 5842–5882、5945 行；`crates/singlefs-harness/src/history.rs` | D8（核心索引结构） 已定项 6 偏移 64、88，已定项 16 第 7 条 ①b；用户「乙 意图带时间、三格同改」 |
| N4 | U8 一行变异；N2 四条注错用例（第二个单元写、记录写、根槽写、续做里抬 F 先写系统配置的第二个写） | `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs`；变异表 | 上一轮判决第三节第 5、6 条 |
| N5 | GB 与文档：`mount.rs` 写丙1 的几处改成 GA；「只差这两样」改成三样；`unlink_mount_continuation_and_the_emptied_version.rs` 那句「记录里没有 ctime」改对 | `mount.rs`；那份测试文件 | 上一轮判决第三节第 1、3、4 条 |
| N6 | 回扫旧树值落在 `crates/` 的那一半：注释、`expect` 消息、局部变量名；harness 里 `= 7` 的局部常量换成格式常量 `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES`；变异表一行说明列改名（删旧名追加新名） | `allocator.rs`、`recovery.rs`、`transaction.rs`、`mount.rs` 与 harness 几份测试 | 回扫报告 `/tmp/claude-1000/m3-writeback2/peer-sweep-12-trees-report.md` 第 3.4 节 |

**这一轮不判的**（各有去处，腿不攻）：
- 开工第一轮接手那条线（根环）7-2 与第 8 批合进主仓的改动，在冻结副本里，不属于这一批：`raise_the_floor_through` 的回收门槛 `reclaim_floor(new_floor, oldest_valid_root_in_the_window)`、抬 F 的上限、挂载时读不出的槽与重读那一支、`recovery.rs` 的系统配置见证。它们由那条线的代码轮 m3-root-ring-code-r2 判，它的攻方已打中重读那一支，改法进它的第 9 批。
- 形式化接入那条线第 16、17、18 批的改动。
- I-7.9 在根环有洞时误红：归根环那条线第 9 批。
- `recovery.rs` 的 `NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE = 10` 偏松：记欠账。

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 冻结副本取自 2026-10-03 主工作区，第 14 批 P、Q 两份补丁已经合入。合入验证 `merge-and-verify.sh`（日志 `/tmp/claude-1000/m3-batch14/merge-verify/`）：harness 档 141 个测试二进制全绿，fmt 过，clippy 过；checker 档没跑。
- 实现员 P 自报：
  - GC 两条在 GA 下绿；改回丙1 时两条都红，红在 `MappingStillUnreadable`，槽 1411、50242。
  - 原来钉丙1 结局的两条（`admission_raises_the_floor_before_refusing.rs` 现第 1477、1680 行）改成钉 GA 的结局，改回丙1 照样绿。那块盘面的交集本来就为空，它们不分辨丙1。
  - 被抛弃的根在单盘上被择中那一形没有用例钉。
  - 模型不带写入时间，比不了。
  - 「步 5」那一行变异点名的是慢档用例，没证。
- 上一轮两份攻方模型（`research/prompts/m3-unlink-code-r3-opus-model/`、`research/prompts/m3-unlink-code-r3-opus-s2-model/`）各带一份 GA 的副本改法，是对着上一轮的冻结副本写的。主 agent 在副本上复跑过第一份的窄盘网格：GA 1900 个起点，交集 0，卸载被拒 0。第一份的 `rerun.sh` 要给 `run.sh` 补执行位才跑得起来。
- 形式化接入那条线 2026-10-03 在 D16 已定项 1 射程补了一句「树表 0 条的一版上不推」（卸载那一串空发布）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N6）
逐条从用户定案、上一轮判决行或条款原文推到冻结副本里的行，再推到钉它的测试与变异行。要看的有这几处：
- N1：放开的那一截是不是「各盘最新有效根取小」以下；被抛弃的根、读不出与自证不过的槽怎么算，是不是与 D16 已定项 1 今天的原文一致；准入那一格有没有被碰；与「树表 0 条的一版上不推」那句对不对得上。
- N2：GC 两条判的是不是用户看得到的读回结局。
- N3：意图那次三个时间秒是不是调用方给的值；各批是不是照抄；①b（同一份崩溃镜像各自续做，两份盘逐字节相同）还成不成立；挂载续做里那几批没有调用方，取的是不是意图那次的值。
- N4、N5：每条新变异改坏的是不是它名字说的那一处；文档与代码对不对得上。
- N6：只改注释、消息、名字的地方，行为是不是真没变；`= 7` 换成格式常量（值 11）之后，用到它的地方结果是不是不变。

### K2　哪一格会错（N1–N6，重点 N1、N3）
在冻结副本的拷贝上攻：
- **N1 GA**：
  - 把上一轮两份攻方模型移到这份冻结副本上，照上一轮的四族放开扫：丢任一块盘之后，剩下那块盘择中的根引用的单元有没有被卸载那一串写掉；卸载在预演里被拒的格子有多少。
  - 单盘恢复择中被抛弃的根那一形：被抛弃的根不算进「各盘最新有效根」，丢盘之后剩下那块盘上最新的根恰好是被抛弃的那条时，会不会择中它、读到被写掉的单元。
  - 根环里有读不出的槽、有洞（由 journal 施加出来的那一版不写根）时，「各盘最新有效根」取得对不对。
  - 卸载—再挂循环。
- **N3 Q-F 乙**：有没有一形，同一份崩溃镜像各自续做，两份盘不逐字节相同；随机历史与层 0 固定脚本里，时间取法有没有让钉了字节的地方悄悄变了；模型不比时间留下多大的缝。
- **N2、N4–N6**：每条新用例改坏点名的那一行红不红，有没有错实现照样过。

### K3　被判的文件名单
`research/prompts/m3-batch14-code-r1-snapshot/crates-sha256.txt` 里那 21 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从用户定案、判决行与条款原文推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N1、N3 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步（单元区槽数、覆盖写次数、文件单元数、崩在哪一批、卸载里崩在哪个写之后、丢哪块盘、unlink 时机、调用方给的时间）放开扫，不写死 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮，进下一批的代码轮。
- N1 判的是用户看得到的结局：丢一块盘之后，剩下那块盘只读挂载做不做得成、整读活文件与卸载之前读出的相同不相同。池级 checker 的条名在单盘镜像上饱和，不拿它分辨 GA 与丙1、改之前那一版（上一轮判决第二节）。
- 腿提的新改法算零轮。

交付：
- 腿的报告写进 `research/prompts/m3-batch14-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句。
- 模型与脚本放 `research/prompts/m3-batch14-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号，去 kb 文件里现查，不从背景材料里数；引代码写冻结副本里的文件:行号。
- 方案按 `crates/` 今天的实现来谈。
