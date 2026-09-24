# 2026-09-23 四份补丁：代码轮第一轮判决（2026-09-24）

<!-- doc-lint:not-numbers Y1 Y2 Y3 Y4 Y5 Y6 -->

正文 `research/prompts/_m2-wave3-code-r1-body.md`，背景材料 `_m2-wave3-code-r1-background.md`，diff `_m2-wave3-code-r1-diff.md`，开工快照 `m2-wave3-code-r1-snapshot/sha256sums.txt`（36 份，00:50:44 UTC）。判决只引产物与代码，腿的结论句当线索。

被判的 `crates/` 文件（门禁 56 号按路径点名）：`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/root_record.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-harness/src/bad_disk_input.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/model_comparison.rs`。`make_filesystem.rs` 与 `allocator.rs` 在四份补丁里被引用、没有自己的 diff 块，材料员的小节清单与本地腿的覆盖表都没给它们单列一行；这里点名，覆盖由第三节 Y5 与 Y6 那两格一起判。

## 一、这一轮交了什么

| 腿 | 格 | 报告 | sha256 |
|---|---|---|---|
| 云端攻方（Opus） | Y1、Y3、Y4 | `m2-wave3-code-r1-opus-output.md`，模型 `m2-wave3-code-r1-opus-model/` | dc92dc659f4d74641ec3c393545a3692a593a33cd4de8d38a8662661d463bb55 |
| 云端正推（Sonnet） | Y2、Y5 | `m2-wave3-code-r1-sonnet-output.md` | 023fbb235bbe3f071f001491120dcf3ad6f32faaf1dbf16f940954a8cfa5c91d |
| 本地攻方 | Y6 与逐文件覆盖 | `m2-wave3-code-r1-local-attack.md` 与样本 s1、s2（干净）、void1（闸判红作废） | 见核查员报告 |
| 核查员 | 全部 | `m2-wave3-code-r1-verifier-output.md` | e2df87050dc43d1287091c552767c317250566cb824aa3e9a0dfd9c8e01f21a8 |

核查员三条腿共核 70 处：攻方 ✓20 ✗1（附录 B 贴出的六行输出次序与命令对不上，内容无误）；正推 ✓20 ✗6（`.claude/kb/decisions/22-单元原子性怎么合成.md:725` 引了三次，那份文件只有 534 行，725 是背景材料的行号，原文在第 150 行；其余是代码行区间偏差与措辞）；本地腿 ✓20 ✗2 核不动 1。

**开工快照**：`sha256sum -c` 34 OK、2 FAILED——`.claude/kb/checks-owed.md` 与 `.claude/kb/milestone/02-second-txn.md`，是主 agent 同日按用户定案写回 C495、C483 与收口表第 8、12、22、49 行改的。三条腿对这两份按行号引用零处（核查员现跑 `grep`），不因此记 ✗。

**复跑**：核查员在自己的草稿目录重新拷仓、编译、跑攻方的 `run.sh`：Y1-a 的 130 格只红 I-7.8、Y4-a 的 k = 78..80 三格，排序后与留存日志 `diff` 为空；改法 D 让 Y1-a 288 格全绿、D 之下 `checker_known_bad_images` 23 条与 `second_transaction_step_four_rollback` 11 条全绿；改法 E 让 Y4 的变异由漏变红。核查员交回时还没跑完的两段，主 agent 等进程退出之后核了（草稿目录 `/tmp/claude-1000/m2-wave3-verifier/rerun-opus/`）：`fixE-y3.log` 347 行 `k=` 判定排序后与同一目录里不带改法的 `y3-run1.log` 加 `y3-formatted.log` 逐行相同，与攻方留存的 `logs/fixE-y3.log` 也逐行相同（两个测试线程交错，只差行序）；`fixD-m356.log` 原样是 `回退之后推零单元发布到 (1, 3) 离开根环：池级 checker 一条违例都没有：[("I-7.8", Violated("根环水位最大 11，盘上出现过的最大树 ID 15"))]`，改法 D 之下变异 356（攻方报告按开工时的行号叫它，内容是「回退行那次发布的树 ID 水位不取根环里的 max」）点名的用例仍红。第四节第 1、2 条的前提都成立。

**本地腿的提示有三处代码事实写错**（核查员查出一处，主 agent 按它的办法又查出两处）。第 1 题七行的「点名断言」是本地腿自己读代码写进英文提示的，两份样本在这三行上答得一致，只因为看的是同一个错前提：

| 行 | 提示写的 | 现查（`crates/singlefs-harness/tests/`） |
|---|---|---|
| 行 2（写入时间留成 A 的） | 只给了一条比内容的断言 | `second_transaction_step_one_overwrite.rs` 的 `cold_start_reads_the_second_content_and_the_pool_checker_stays_green` 里另有一条 `assert_eq!(first_file_inode_record.write_time_seconds, FIXED_WRITE_TIME_SECONDS + 60, …)`，从只读挂载读回 inode 记录，就是钉这一条的 |
| 行 6（C481 基线档） | 「那个测试函数里找不到读分档数或分档名的断言」 | `second_transaction_supplement_three_bad_disk_input.rs` 第 145–150 行 `assert_eq!(report.tally.base_image_tiers_sampled(), EVERY_BASE_IMAGE_TIER.len(), …)`，上面两行注释点名它就是抓这条变异的 |
| 行 7（C378） | 今天回卷、变异改成不回卷 | 方向反了：`crates/mutations.tsv` 里 C378 那一行写「……把系统配置的实例代号回卷成旧号（改成回卷）」，今天是不回卷；点名的用例名就是「……leaves the new instance in the system configuration……」 |

所以本地腿第 1 题的行 2、6、7 与第 4 题 A 列作废，改由主 agent 现查判（第三节 Y6）。

## 二、跑前判据，各触发没触发

| 判据 | 触发没触发 |
|---|---|
| 1 兑现了条款：抄出整段原文，推导不引没有条款的取法 | Y2 子问 1、Y5 两问、Y3 的 ② ③ |
| 2 和条款说反话：指得出字节或可达历史 | **Y1-a**：checker 的代码与 I-7.8 自己的读法注说反话，差异在孤儿码 2 节点头的写序实例代号上 |
| 3 替没写的条款做了选择：两个都说得通、在字节或可达历史上分得开 | Y3 的 ① ④、**Y4**（Y4-a 在 k = 78..80 上把两个读法分开） |
| 4 攻方给出的历史要在副本上真跑出来 | Y1-a、Y4-a 都真跑了，核查员独立复跑坐实；Y1「水位被瞬时读错拉低」只推没跑，记推测 |
| 5 本地表 unknown 不算失败，一列填不出判据来源的行号整行作废 | 行 2、6、7 按第一节作废 |

## 三、逐格判决

### Y1　树 ID 水位：水位三样选择没被打中；checker 的 I-7.8 在合法状态上判红（和条款说反话），改

- **Y1-a**（两条流共 130 格）：第一个文件版本那次发布崩在记录落盘之前 → 可写挂载 → 新实例第一次发布取的 txg 与孤儿码 2 节点的诞生代号相等 → I-7.8 只数诞生代号不超过根环最大 txg 的节点，孤儿被数进「出现过的」，判红。`.claude/kb/invariants.md` 第 58 行 I-7.8 的 checker 读法注写的是「崩在发布之前的那个事务写出的孤儿节点不算『出现过』」，代码（`crates/singlefs-checker/src/walk.rs` 第 877 行那一格过滤）只看诞生代号、不看写序实例代号。判别子 checker 看得到：实例表行 (1, 2, …) 说实例 1 施加到 txg 2 为止，诞生代号 3 的实例 1 节点就没发布过。
- **不分辨水位补丁的三样选择**：mkfs 那条流水位恒 11、三样选择都不起作用，照样红。病根在 checker 的读法，不拿它判 Y1 的三样选择；三样选择（读不出的根拿记录顶、被抛弃时间线也算、按常量次序连号发）在这一轮没被打中。
- 其余几问：号被重发（48 格，重发的都是没发布过的号）、树表条数与「树建没建」都没打中；`publish_version` 的 `assert_eq!` 产品路径走不到（调用点全带 `Some`）；两个新错误成员「走到时盘上逐字节不变」没验，记账。
- **结论**：采纳攻方的改法 D——I-7.8 的扫描方向再排除「写序实例 i 在最新根的实例表里有非回退行 (i, Ti, ·)、Ti > 0、且诞生代号 > Ti」的码 2 节点；回退行不排除（被抛弃时间线发布过的号仍要算）。**被攻过零轮**。`.claude/kb/invariants.md` I-7.8 的 checker 读法注跟着改成这个判法。

### Y2　中央映射树根按根指针的出生树判 I-1.3：兑现了条款

- 正推抄出 I-1.3 整行（`.claude/kb/invariants.md` 第 26 行），四处判定读法（checker、只读挂载、恢复、挂载重建）逐一现查读的都是根指针头部的出生树，没有一处还用写死的 15。
- 子问 2：`.claude/rules/fs-design.md`「审计与被审计用同一段代码」管的是运行时靠遍历现算的记账量，这里读的是 O(1) 的字段，射程够不着，**推不出**。
- 正推附带的一条事实记账：拿掉常量 15 之后，中央映射树这一格的 I-1.3 是块头与指针两个字段互核，两个字段由同一次写的同一个变量落盘；两边一起写错时 I-1.3 不说话。不挡里程碑二，第五节记账。

### Y3　候选 b：② ③ 兑现，① ④ 是选择；④ 与 C533 两格没打中

- ② 根环里没有 (i, txg) 的根、③ txg ≥ 最新根带的 F：判决 Z3 与 I-3.1 读法注的字面后果。① 带提交标记且最新根的实例表里有这个实例的行、txg ≤ Ti：「已被施加过」是实现读出来的，另一个读法「下一次挂载会施加的那一版」在 Y4-a 那三格上分得开。④ 根环里有一条没被判抛弃的根、txg 比它小：条款没有，是回收门槛推出来的充分条件。
- ④ 翻面只在根环转过一圈之后，与收口表第 ② 行的已知红同一时刻，分不开；C533 那两格 17 前缀 × 7 步 119 格全绿。没打中各只是一次观测，只覆盖攻方报告「没打中的形状」那张表写的历史。

### Y4　I-3.10 只读回退候选集：替没写的条款做了选择，改

- `.claude/kb/invariants.md` 第 136 行 I-3.10 判据列写「任一未带已释放标志的分配记录」，射程 ④（2026-09-23 主 agent 补、零轮）收窄到回退候选集那几版；代码与 ④ 一致，与「任一」不一致。
- **Y4-a**：B 那次发布的分配代写错（攻方自己的分配器变异，只在 B 那一次开），录制流前缀 k = 78..80（B 的记录已落、根槽没落，恢复一定施加 B）上 I-3.10 在崩溃镜像上判成立，挂载之后才红；k ≥ 81 在崩溃镜像上就红。两个读法：(a) 今天只读「已被恢复施加过」的几版，(b) 另读「下一次挂载会施加」的那一版。(b) 判得更严。
- **结论**：采纳改法 E——I-3.10 的读集再加「同实例、txg = 最新根 txg + 1、带提交标记、根环里没有它的根」那条记录新根段里树表指着的分配记录树。**被攻过零轮**；只接一步、不复刻恢复的整条前缀规则，链上第二条及以后仍漏，第五节记账。`.claude/kb/invariants.md` I-3.10 的射程 ④ 跟着改。

### Y5　C512 根记录加分配记录树根、I-9.14 按时间线收窄：兑现了条款

- 新字段的三种取值（mkfs 全零 / 树表 0 条写行那一版非零 / 带文件的一版全零，零单元发布照抄）是 `.claude/kb/decisions/16-发布语义.md`「#### 已定项 9」与 `.claude/kb/decisions/22-单元原子性怎么合成.md`「#### 已定项 7」的字面后果，五处写入与释放判定（`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-core/src/transaction.rs` 四处）逐一对应。
- I-9.14 的收窄与 `.claude/kb/invariants.md` 那一行今天的条文逐字一致。
- **一处注释说了假话**：`crates/singlefs-core/src/transaction.rs` 第 3371 行注释说「`root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero` 钉住」，全仓搜不到这个测试。补这条用例（第四节第 3 条）。

### Y6　harness 那一组的用例与变异：七条变异都钉在点名的行为上；逐文件覆盖有四处缺变异

第 1 题七行，本地腿两份样本一致的行采信，第一节作废的三行主 agent 现查：

| 行 | 判 | 依据 |
|---|---|---|
| 1 extent 指针没换 | 同一件事 | 两份样本一致；点名用例读回内容 |
| 2 写入时间留成 A 的 | 同一件事 | 主 agent 现查：点名用例里有从只读挂载读回 `write_time_seconds` 的断言 |
| 3 反向链取错 | 同一件事，经 checker | 点名用例没有直接比反向链的断言；它逐条要求池级 checker 不判红，反向链由 I-8.6（反向链算法）判。变异红在 I-8.6 那一条上才算钉住行为，门禁 59 号只看用例红没红、不看红在哪条断言——第四节第 4 条要实现员记下这条变异红在哪条不变量上 |
| 4 取号之后没有屏障 | 同一件事 | 两份样本一致 |
| 5 层 0 按发布归属错位 | 同一件事 | 两份样本一致 |
| 6 C481 拿掉树表 0 条那一档 | 同一件事 | 主 agent 现查：第 145–150 行的 `assert_eq!` |
| 7 C378 改成回卷 | 同一件事 | 主 agent 现查：点名用例断言新号留在系统配置里 |

逐文件覆盖（第 2–14 题，本地腿两份样本一致的格；主 agent 拿 `grep -n -F` 对 `crates/mutations.tsv` 现查过）：

| 文件 | 缺什么 |
|---|---|
| `crates/singlefs-core/src/recovery.rs` | `highest_tree_identifier_watermark_in_the_ring`（读不出的根拿记录的水位顶上，C342 用户认了「取」）与 `tree_table_entry_count` 一条变异都没有 |
| `crates/singlefs-checker/src/walk.rs` | 中央映射树根按根指针出生树判的那一行（第二份补丁）没有变异；I-3.10 的判定有描述里带「I-3.10」的八行变异盯着 |
| `crates/singlefs-core/src/mounted_read.rs` | `open_pool_for_read` 改读出生树的那两处没有变异 |
| `crates/singlefs-core/src/root_record.rs` | 整个文件 0 行变异；457 字节的读写只由文件里那条 `#[test]` 往返盯着 |

`crates/singlefs-harness/src/history.rs`、`model.rs`、`model_comparison.rs` 的改动是错误成员到显示串的穷举 `match` 臂增删，编译器替它们数着，不另要变异；`crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 加一条「I-3.10」，由 checker 报告的逐条名单用例盯着。`crates/singlefs-harness/src/fault_injection.rs` 与 `bad_disk_input.rs` 的新函数是装置本身，由行 6、行 7 两条变异经它们红出来。

## 四、要落的改动（主 agent 定，不交用户）

派一个实现员在副本里做，交补丁；`crates/singlefs-checker/src/walk.rs` 另有实现员（收口表第 26 行，I-7.9、I-3.11）在改，这一份等那一份的补丁打进主工作区之后再派，从那之后的主工作区起步：

1. **改法 D**（Y1-a）：把 Y1-a 的两条流（mkfs 流 k = 44..54、回退流 k = 84..98，挂载之后五种动作）做成用例，今天的 checker 上必须红、D 之后必须绿；`crates/mutations.tsv` 加一条「把 D 的排除条件拿掉」必须让这条用例红。
2. **改法 E**（Y4-a）：用坏镜像做用例：B 那次发布的记录已落、根槽没落的崩溃镜像，把 B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1，今天判成立、E 之后判红；变异「E 那一步不读」必须让它红。
3. **补 Y5 那条用例**：`root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero`（带文件的一版根记录的分配记录树根指针全零），先证明会红。
4. **补 Y6 那四处变异**：`recovery.rs` 的 `highest_tree_identifier_watermark_in_the_ring` 不取记录的水位、`tree_table_entry_count` 恒报 0；`walk.rs` 与 `mounted_read.rs` 判映射树根时退回写死的 15（取回退流上水位不是 11 的那一版，否则等价）；`root_record.rs` 新字段的偏移错一位。每条先证明会红；等价的按 `.claude/rules/mutation-sampling.md` 换敏感取样点。另记下行 3 那条变异红在哪条不变量上。
5. `.claude/kb/invariants.md`：I-7.8 的 checker 读法注改成 D 的判法、I-3.10 的射程 ④ 改成 E 的读集，由书记员照主 agent 的规格写，与补丁同一批。

改法 D、E 都是攻方提、只在它的模型上量过，**被攻过零轮**；与这一批新补丁一起进下一轮代码三方，不为它们单开第二轮。

## 五、交用户的与记账的

| 项 | 状态 |
|---|---|
| 改法 D、E | 零轮，主 agent 采纳；进下一轮代码三方 |
| E 只接一步：恢复会连着施加几条记录时，第二条及以后那几版的分配记录 I-3.10 仍读不到 | 记账（欠账表新立一行），不挡收口表 |
| 中央映射树这一格 I-1.3 是同一次写的两个字段互核，两边一起写错时不说话 | 记账（欠账表新立一行），不挡收口表 |
| Y1 两个新错误成员走到时盘上逐字节不变没验 | 记账：锚在 `TreeIdentifierWatermarkLeavesNoRoom…` 与 `TreeTableOfTheVersionToBuildOnUnreadable` 的两行变异只钉「交回错误」，没钉「盘上不变」 |
| 本地攻方自己读代码写进提示的「点名断言」三处写错 | 记进 `records/2026-09-16-subagent拆分提案.md`：本地腿的提示事实要在派出之前由主 agent 或材料员逐条现查 |

这一轮一格都没有替用户定。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 8 | 不影响 | 2026-09-24 不受影响：水位的三样选择这一轮没被打中；Y1-a 改的是 checker 对 I-7.8 的读法，不是水位的定案 |
| D16（发布语义） 已定项 9 | 不影响 | 2026-09-24 不受影响：Y5 判兑现，定案一字不改 |
| D22（单元原子性怎么合成） 已定项 7 | 不影响 | 2026-09-24 不受影响：Y5 判兑现，字段表一字不改 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-09-24 不受影响：Y1-a 用到它「新实例第一次发布的 txg ≥ max(…) + 1」那一句当前提，实现取等号是这句允许的，不改 |
