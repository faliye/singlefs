# 判决：里程碑「第二个事务」第二波代码改动，代码三方第二轮（2026-09-18）

<!-- doc-lint:not-numbers V1 V2 V3 V4 V5 V6 Z1 Z2 Z3 Z4 Z5 Z6 -->

正文 `research/prompts/_m2-wave2-code-r1-body.md`；背景材料 `_m2-wave2-code-r1-background.md`；代码附录 `_m2-wave2-code-r1-diff.md`；开工快照 `research/prompts/m2-wave2-code-r1-start-snapshot.sha256`（17 个文件，腿跑着时与交回后主 agent 与核查员各核一次，都 17/17 相同）。
腿：云端攻方（Opus，Z1–Z3，报告 `m2-wave2-code-r1-opus-output.md`，模型 `m2-wave2-code-r1-opus-model/`）、云端正推（Sonnet，Z4 条款那一半、Z5、Z6，`-sonnet-output.md`）、本地攻方（Z4 算术那一半，提示 `-local-attack.md`，计入的两份干净样本 `-output-s2.md`、`-output-s3.md`）。核查员 `m2-wave2-code-r1-verifier-output.md`：核 106 处，103 ✓、3 ✗，Opus 的 8 个复跑格在核查员自己的副本上逐字节相同。

## 一、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| Z1 上界准入 | **打中四处**：Z1-a 实例表不在准入里（第 370 次可写挂载取号之后 panic、每试一次再烧一个号）；Z1-b 按上界误拒真发得起来的池（实现员那两例之外另 5 处，「一路写到被拒、再重开」进吸收态）；Z1-d 复用已回收记录时跨度变了、旧记录留着，下一次可写挂载在取号之前 panic、池从此挂不上可写；Z1-f 会话里同一个上界算式误拒装得下的覆盖写。Z1-e 没打中 | 攻方探针，核查员复跑逐字节相同；主 agent 现查 `crates/singlefs-core/src/mount.rs` 第 712、838 行、`crates/singlefs-core/src/allocator.rs` 第 375、601、609 行、`crates/singlefs-format/src/lib.rs` 第 101 行 |
| Z2 失败账 | 没打中触发列（三种走法合计都等于设备一层、没被吞）。问列上打中两处：Z2-c 最后一步超级块槽失败时根已 FUA，分配器照样退回，换一条路发布再断电之后 checker 五条红、重开 `UnitUnreadable`；Z2-d 注释「份数就是失败过几次发布」与代码不符（落盘之前就失败的不记）。Z2-e 属第 20b 行已知一族，补两个细节 | 同上；`crates/singlefs-core/src/transaction.rs` 第 231、1221、1258、2111 行 |
| Z3 聚簇段登记 | **打中一处**：Z3-a 重开之后用户数据落进上一次挂载开过、仍装着环里的根引用的提交内生块的段（88 种模式 18 种中）。Z3-b / Z3-c 第一版几何下不可达 | 同上；`crates/singlefs-core/src/allocator.rs` 第 403、495、536 行 |
| Z4 两条新不变量 | 没打中。条款那一半：`crates/singlefs-checker/src/walk.rs` 的区间端点、跳过已回收记录、I-9.14（树表条目的诞生 txg 跨根不变） 只出现在一个树表单元里报不适用，三处与 `.claude/kb/invariants.md` 两行逐字对得上。算术那一半：本地腿两份样本 17 行与主 agent 按事实表算的逐格相同 | 正推腿；本地腿 s2、s3 |
| Z5 改动计数与区域表 | 67 段全部归因、21 行区域表与字节表对得上，没打中；正推腿报的「21 个区域 11 个字节不等」是 E142（第一个事务的干跑） 量 5 触发跑前登记 F1，另派诊断查清：两边写路径逐字节一致，两边喂的 3000 字节文件内容不同（装置 `(index * 7 + 3) % 251`，`crates/singlefs-harness/src/scenario.rs` 第 29–33 行 `index % 251`），违反跑前登记第 18 行「同 3000 字节内容」 | 诊断 `research/prompts/e142-r11-f1-diagnosis.md`（副本上两个方向换内容都 21/21 全等）；主 agent 现查装置第 3717 行与 `scenario.rs` |
| Z6 写回 | **打中一处**：收口表第 11 行的「还要」漏了 `.claude/kb/decisions/08-核心索引结构.md` 第 430 行「改动计数 1」。另有一处正推腿顺带报的：`walk.rs` 第 963–972 行注释还是修订前的口径（「与 I-3.9 的字面不同，等用户定案」、函数说明用旧简称） | 主 agent 现查两处属实 |

**判据写宽了一格**：Z3 触发列「用户数据落进装着提交内生块的段」按字面第一个事务就中（数据单元 50180 与 mkfs 的实例表同段）；攻方只算「上一次挂载当聚簇段开过、仍装着环里的根引用的提交内生块」的段，这一格按它的口径判。

**打中归错了判据两处**（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」）：Z1-d 字面满足 Z1 触发列，病根在步 5 的复用、不在上界；Z2-c 不满足 Z2 触发列，实质是失败路径的正确性。两处都按病根归位处置。

## 二、处置：挡不挡本轮课题

本轮课题：V1–V6 这批改动是否成立，以及里程碑「第二个事务」自己那几步的正确性。挡着的这一轮改；不挡的记下、本轮收尾时回表再判。正文跑前写死「Z1 / Z2 / Z3 / Z4 打中 ⇒ 改代码、补会红的用例与变异、记进收口表；本轮不再攻第三轮」，所以这一轮的改法**被攻过零轮**，收口表照写。

| # | 打中 | 挡不挡 | 为什么 | 这一轮做什么 |
|---|---|---|---|---|
| 1 | Z1-d 复用留下重叠的分配记录，池永久挂不上可写 | **挡** | 步 5 的复用是里程碑自己的一步，288 段历史 208 段以它收尾；checker 对重叠记录没有判定（I-5.1（物理范围不重叠） 管的是树里的引用） | 改分配器：一次分配写下的记录罩住的槽上，别的已回收记录删掉；新立一条不变量「同一块盘上分配记录罩住的槽互不重叠」、checker 实现；用例用攻方的历史（挂载 → 覆盖写 2 次，× 8，第 9 次挂载） |
| 2 | Z1-a 实例表满了取号之后 panic、每试一次烧一个号 | **挡（最小改法）** | V1 自己承诺「算不过就在任何写之前返回」，实例表是它漏算的一项；把 panic 加烧号换成取号之前拒绝 | 取号之前的准入加一项：这一版的行数 + 这次要写的行数 + 链指针 ≤ 一片的容量，不够返回新的挂载错误、一个写都不发；实例表第二片与删行不在这一轮 |
| 3 | Z2-d 注释与代码不符 | 挡（V2 自己的文字） | — | 改注释为「落盘阶段失败的发布」 |
| 4 | Z6 两处文字 | 挡（本轮写回） | — | `walk.rs` 注释改成与 `.claude/kb/invariants.md` I-3.9（释放代落在停止引用它的那一格区间里） 一致；`decisions/08-核心索引结构.md` 第 430 行按已定项 6 改成 3 |
| 5 | Z1-b / Z1-f 上界在容量边上误拒 | 不挡 | 误拒是保守的一侧，不写坏任何东西；只在分配记录树 812 条上限最后约 10 条里出现，而一个节点装满本身就是第 28 行的容量墙 | 记进收口表；「上界 / 拷贝上真分配」交用户定（攻方的改法臂 3 在扫过的格上不误拒也不漏，被攻过零轮、代价没量） |
| 6 | Z1-a 的完整改法（第二片或删行） | 不挡 | 第 370 次可写挂载之后的事，固定脚本碰不到 | 记进收口表第 28 行一族 |
| 7 | Z2-c 根已 FUA 之后失败、分配器退回 | 不挡本轮课题，但是写坏池的缺陷 | 病根在「发布失败之后实例还能不能接着写」这个语义（`transaction.rs` 第 1221 行注释引 D16（发布语义） 已定项 7「这次发布没有成立」，而恢复会选中已落盘的根），不在 V1–V6 | 另立欠账 C381，收尾时列为优先交用户定 |
| 8 | Z2-e 调用方不取、挂载失败时成功落盘的账也丢；`raise_rollback_floor` 同形 | 不挡 | 第 20b 行已知一族 | 补进第 20b 行 |
| 9 | Z3-a 重开之后用户数据落进旧聚簇段 | 不挡 | 是落点政策（D3（空间分配） 已定项 8 第 2 条）被违反，不写坏数据；第 20c 行「一个聚簇段什么时候不再算」没有条款 | 第 20c 行改写「今天可达」并附攻方的历史 |
| 10 | E142 量 5 两边文件内容不同 | 挡（E142 第十一次跑的前提） | 跑前登记第 18 行要求同内容 | 装置对齐到 `index % 251`、跑前登记追加修订、重跑，交 E142 执行员 |

攻方在造重叠的那一版上顺带看到 I-3.1（已分配统计对得上） 从第 6 次挂载起、重叠出现之前就红，它自己标「疑似第 ② 行那一族、没核」；主 agent 也没核，记进第 ② 行当线索。

## 三、被判的文件（门禁 56 号按路径认）

| 文件 | 这一轮攻了没有 |
|---|---|
| `crates/singlefs-core/src/mount.rs` | 攻了：Z1-a、Z1-b、Z1-d（`establish_instance`、取号之前的准入） |
| `crates/singlefs-core/src/transaction.rs` | 攻了：Z1-b、Z1-f、Z2 全部 |
| `crates/singlefs-core/src/allocator.rs` | 攻了：Z1-d、Z3 |
| `crates/singlefs-core/src/write_accounting.rs` | 攻了：Z2-a / Z2-b 的按种类合计与设备一层逐项比 |
| `crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/lib.rs` | 攻了：Z1-d、Z2-c 走到重开与恢复 |
| `crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs` | 攻了：Z4 两半 |
| `crates/singlefs-harness/src/first_transaction_regions.rs`、`crates/singlefs-harness/src/sha256.rs`、`crates/singlefs-harness/src/hexadecimal.rs`、`crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs`、`crates/singlefs-harness/src/lib.rs` | 攻了：Z5（区域表与字节表逐行、配对行） |
| `crates/singlefs-harness/src/scenario.rs`、`crates/singlefs-harness/src/crash.rs` | 攻了：Z5（量 5 的实装一侧）；文件内容与装置不同由诊断查出，改的是装置 |
| `crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs` | 没攻：只是跟着 `run_first_transaction` 回调多一个参数改的调用点（3 行） |
| `crates/singlefs-harness/src/bin/first_transaction_on_device.rs` | 没攻：增补 1、增补 2 第 30 行加的 `publish_writes` 行与挂载窗口这一轮的腿没碰；同一文件里 E152（按里程碑对比六家文件系统的文件性能） 加的几种模式是别的会话的改动，不在本轮 |
| `crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-format/src/lib.rs`、`crates/singlefs-harness/src/segments.rs` | 没攻：只改了 `//!` 注释里的路径（`.claude/rules/path-moves.md`） |

## 四、核查员查出的引用错

- Opus：「kb 与收口表里 grep『重叠』无命中」说宽了——`.claude/kb/` 下有 33 处「重叠」；主 agent 现查，没有一处管分配记录之间的重叠，结论不变。
- Sonnet：`walk.rs` 第 1066–1096 行那段代码块把闭包参数 `sighting` 写成 `s`，不是原文。
- 本地腿：核对表把 C2 标在 `walk.rs` 第 1100 行，实际横跨 1099–1100；交回里 s1 的哈希前缀写错（`0358bd…`，实际 `035bd6b4…`），s1 本来不计入。
三处都不改判定。

## 五、写回哪里

1. `crates/`：第二节第 1–4 行，交实现员；每一处先做成会红的用例，再改，`crates/mutations.tsv` 各留一条改回去就红的变异。
2. `.claude/kb/invariants.md`：新立「同一块盘上分配记录罩住的槽互不重叠」，条数登记位跟着改。
3. `.claude/kb/checks-owed.md`：新立 C381（根已 FUA 之后发布失败，分配器仍退回）。
4. `.claude/kb/milestone/02-second-txn.md` 收口表：第 ②、11、20b、20c、28 行按第二节补；新加 Z1-d、Z1-a（最小改法）、Z1-b / Z1-f、Z2-c 几行，标「被攻过零轮」。
5. `.claude/kb/decisions/08-核心索引结构.md` 第 430 行。
6. 背景材料与正文里开工快照的 `/tmp` 路径改指 `research/prompts/m2-wave2-code-r1-start-snapshot.sha256`（文件已拷进仓）。

## 六、这一轮之后

这一批不开第三轮。第二节第 1–4 行改完之后：层 0 全量（门禁 54 号）与 crates 变异表（59 号）归 `crash-verifier` 重跑；Z1-d 的改法要跑攻方那组 88 种模式的扫描确认不再中。第 5、7 行在本轮收尾时与收口表里其余留待的项一起回表再判。
