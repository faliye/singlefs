# 根环实现与系统配置 R2：代码轮第一轮判决（2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 R2 H1 H2 H3 H4 B1 B2 E1 E2 E3 K1a RR1 RR2 RR3 RR4 RR5 RR6 RR7 RR8 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 改动全文 | `research/prompts/_m3-root-ring-code-r1-body.md`、`_m3-root-ring-code-r1-background.md`、`_m3-root-ring-code-r1-diff.md` | — |
| 开工快照 | `research/prompts/m3-root-ring-code-r1-snapshot/`（crates 31 份、kb 7 份；冻结副本 `/tmp/claude-1000/m3-root-ring-code-r1/tree/`） | `bash research/scripts/snapshot-check.sh m3-root-ring-code-r1`：「✗ 快照 m3-root-ring-code-r1：38 个文件里 3 个对不上」，三份见下 |
| 云端正推（Sonnet） | `research/prompts/m3-root-ring-code-r1-sonnet-output.md`（241 行，sha256 3b2dd816…2d76），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-root-ring-code-r1-opus-output.md`（sha256 ad94d741…4f9d），模型 `…-opus-model/` | 主 agent 复跑，见第四节 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <crates 取冻结副本、其余取主仓的拼合根> --background <背景材料>`：「✗ 核了 89 处引文，3 处对不上，0 处没判」，三处见下 |

快照对不上的三份与处置：

- `crates/singlefs-checker/src/walk.rs`：腿跑着时被别的会话在主工作区改了丁-defer 那一段（620 行差异，新函数 `deferred_slots_among_the_references_of_the_selected_roots`），冻结副本没动。两条腿都读冻结副本；只有正推腿第 162、166 行三处行号取自主工作区（cite-check：「walk.rs:5486「.entry((record.instance, *counter))」……原文在这份文件第 5412, 6366 行」，另两处同样差 74 行），三句原文在冻结副本第 5412、5435、5313 行逐字在、意思不变，照冻结副本的行号采。
- `.claude/kb/decisions/13-验证路线.md`、`.claude/kb/checks-owed.md`：形式化接入那条线的会话在腿跑着时定点改过，逐处旧串、新串与两次写后 sha256 存在 `research/prompts/m3-root-ring-code-r1-snapshot/peer-changes-formal-line.md`（sha256 855df593…81d9）。改的是 D13（验证路线） 已定项 6 一节、C407、C670 起的新行、C671、C673；两条腿只引了 `13-验证路线.md:138`（已定项 5），没引 `checks-owed.md` 的行（`grep -nE 'checks-owed\.md|13-验证路线\.md'` 两份报告只命中正推第 67 行那一处），被引的行不在被改的几处里，所以没倒推整份原样。

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 七项取值、三处装记录、头宽偏移容量与 kb 标记 | 一致；手解盘面七段与同代根逐段相同 | — | **站住**（一次抽样） |
| K1 卸载记号其余位 | H1：读者与恢复判损坏，池级 checker 收（同一盘面恢复丢记录 3、checker 把它算进前缀） | 同一件，只有校验和对得上的外来字节走得到，合法崩溃历史走不到（量过） | **打中**（主 agent 现查：core `root_record.rs:43-48` 位 0 之外非 0 交 `None`；checker `lib.rs:827-828` 只切原样字节、不判这 4 字节）：RR1 |
| K1 卸载那一条用例 | H2：变异 M-F（恢复照抄所选根的卸载记号）下 harness 档 122 个二进制 858 条全绿；补洞草稿（两次卸载空发布的根槽都撕掉）冻结副本绿、M-F 下红 | — | **打中覆盖缺口**（取样点不敏感，`.claude/rules/mutation-sampling.md` 第三类）：RR2 |
| K1 写行发布记录的两项新字段 | H3：M-A、M-B 存活，写行记录从不被施加、没有观察者 | — | **打中覆盖缺口**：I-8.10 扩到七段（用户 2026-10-03 定，进第 8 批 8-2）之后同代根在盘上时就比得到；RR3 |
| K2 挂载时读不出的槽（K2-e） | — | 打中：挂载那一次读错，根环表记成 txg h 的有效根，写到 h + 3S − 1 那一版时门槛停在 h，checker 取窗口里最旧有效根 h + 1，I-3.1 红（记账多 10 个槽）；7 组全红、对照 3 组 0；层 0 txg 23 落盘后 9 态、txg 24 落盘前 4099 态红 | **打中，判别子观测不到**（checker 读盘，挂载那一刻的读错不留在盘上）：D16（发布语义） 已定项 1 的零轮规则「门槛按最小的算，根环表记最大的」与 I-3.1 的等式合不起来；改判据或改条款，交用户（第三节表二） |
| K2 抬 F 那一遍（K2-b） | — | 打中：抬 F 的回收门槛只取读得出的根，根环表里挂载时读不出的那一项没算进去，门槛越过窗口里、F 之上的有效根 X，记账比遍历少（不安全方向）；29 组 19 组红、X 读得出的对照 20 组 0；层 0 三次发布 4108 / 4099 / 0 态红，开改法 B1 后 0 / 0 / 0 | **打中**（主 agent 现查冻结副本 `mount.rs:2732-2738` `oldest_valid_root_in_the_window` 只在 `roots` 上取）：RR4（B1，零轮） |
| K2 抬 F 的上限（K2-b2） | — | 打中：没有读故障的合法历史上准入抬 F 被拒 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`：上限要读洞里窗口外旧根的树表，窗口已许可它被复用；5 组对照臂与随机历史 8 段撞到 | **打中，条款少列一处**：D16 已定项 1「同取这个窗口的五处」不含抬 F 的上限；交用户（第三节表二） |
| K2 挂载时窗口、影子账过滤、循环等价、两份窗口边界、四处的锚 | — | 没打中；循环 170 326 组输入 0 处不同 | **站住**（一次抽样） |
| K3 R2 谓词、三处同一定义、判拒用例 | 一致；fsid 不符条款没说、没有用例；D23 已定项 14 正文还写取甲 | 一致 | **站住**；kb 措辞进写回批 RR7 |
| K3 永远只读 | 单故障没有；双故障两类有，D23 已定项 14 射程认下、欠走出来的路 | 要同一块盘两槽都坏，系统配置 489 字节在 512 字节撕裂粒度下撕不坏 | **站住**：照条款现状，不新开 |
| K3 E165 模型 | — | `e165_replay_bounded_enumeration.rs` 仍按取甲建可写挂载那道闸，不调实现、不会红 | **线索**：并进 C662 那一族的 E165 重新登记（第 8 批 8-4） |
| K4 第一个文件那一形、键、独立性 | 无分歧；H4：记录单落那一版里分配记录树根换成写行那一版的非空指针，checker 读数与正确记录相同 | 层 0 20 511 态 0 违例；键与独立性没问题 | **站住**；H4 记欠账 RR6 |
| K5 两个钉值、派生形态 | 站住：自写 CRC 复算 2660362816、4048568891；反向打三份补丁复现旧值 1733593715；67 只留在用例名与注释 | — | **站住**；E142 干跑模型仍是 311 头，进 E142 重跑（C660，第 8 批 8-4） |

## 三、打中的改法（零轮）与交用户的岔路

表一：并进第 8 批 crates

| # | 改法 | 验收 |
|---|---|---|
| RR1 | 池级 checker 解析记录时照 core：卸载记号位 0 之外有位为 1 判这条记录坏（攻方 K1a） | 正推 H1 那一盘面（卸载记号字段写 2 或 0x80000000）上 checker 与恢复收下同一批记录；补变异行证红 |
| RR2 | 补正推 H2 那条用例（两次卸载空发布的根槽都撕掉、所选根不带记号），`crates/mutations.tsv` 再加一行 M-F（照抄所选根） | M-F 红在新用例上 |
| RR3 | I-8.10 扩到七段（8-2 已排）落地时，把 M-A（写行记录 `new_instance_table` 写成上一版的）、M-B（`new_allocation_record_tree_root` 写成空）加进变异表 | 两行各红在 I-8.10 的用例上 |
| RR4 | 抬 F 那一遍取回收门槛时，把根环表里挂载时读不出的那几项按记的最大 txg 并进窗口里最旧有效根（攻方 B1，与进程内记根同一取法） | 攻方 `raise_floor_hidden_slot.rs` 的红组转绿；层 0 三次发布 0 违例；补一条 harness 用例与变异行 |
| RR5 | `crates/mutations.tsv` 第 1873 行名字改成说报读错的盘面（改钉报告第 2 条） | 门禁 mutation-tables 格绿 |

表二：交用户（零轮）

| 岔路 | 候选 | 依据 |
|---|---|---|
| K2-e：挂载时读不出的槽记最大 txg，与 I-3.1 等式合不起来 | E1 改判据：checker 的 I-3.1 丁-defer 门槛改成区间，下沿取窗口里「那一槽没住着它该住的 txg 的根」的位置的最小 txg，记账落在区间两端的遍历之间就算对得上（推的）；E2 改实现：进程内记根时，根环表那一项成了窗口里最小的那个，先重读那一槽，读得出就照读出来的算，仍读不出才按最大 txg（推的；整条命都读不出时照中）；E3 维持：I-3.1 在这一形上报不适用 | 攻方 K2-e 与改法表；主 agent 复跑 `u31-s4-9-mount.out`：「AFTER txg=23 red=["I-3.1"]」 |
| K2-b2：抬 F 的上限要读窗口外旧根的树表 | B2：上限与「非空」只在一圈窗口里的有效根上取，D16 已定项 1 的列表加第六处（推的，没实现）；维持：环上有洞且 F 低于洞里旧根时准入抬 F 拒，直到洞那一槽被盖掉或卸载抬 F | 攻方 K2-b2；主 agent 复跑 `rf-14-none.out`、`rf-15-none.out` 都记下这个拒因 |

表三：欠账与写回

| # | 去向 |
|---|---|
| RR6 | H4：记录单落、根槽从没落盘的那一版，池级 checker 判不了记录里分配记录树根的对错（I-8.10 扩到七段也比不到，没有同代根）；记进 `.claude/kb/checks-owed.md`，D8 已定项 8 若说那一项冗余就降为「无害、没有判据」 |
| RR7 | 写回批：D23 已定项 14 与 D22 已定项 8 射程里「实现还是取甲」改成 R2 的现值；I-4.1、I-3.1、I-8.6 行照 7-2 报告第十一节改现值；fsid 不符在 R2 里落哪一边条款没说，记进欠账 |
| RR8 | harness 判法 `mount_transparency.rs` 的 `newer_publish_witnessed_on_the_crash_image` 仍按取甲（已排进第 8 批）；`write_request_split.rs` 与几份用例名里的 67 不改（正推 K5-2：只在名字与注释里，断言不依赖它） |

用户 2026-10-03 弹窗定表二两条：K2-e 取 E2（进程内记根时，根环表里挂载时读不出的那一项成了窗口里最小的那个，先重读那一槽，读得出照读出来的算，仍读不出才按最大 txg；整条命都读不出那一形另记，第二轮攻）；K2-b2 取 B2（抬 F 的上限与「非空」只在一圈窗口里的有效根上取，D16（发布语义） 已定项 1 的列表加第六处，先写回条款再进第 8 批实现）。两条都是零轮。

## 四、复跑核对

主 agent 把攻方模型目录拷到会话草稿 `r1-recheck/`，经它的 `scripts/rebuild-and-run.sh`（内存包装、15 线程）复跑 build、k2e、k2b-1、k2b-2、k2b-more-1、k2b-more-2、k2b-fix-1、k1k3k4-1：115 份输出与腿的 `results/out/` 同名文件逐字节相同（两趟各报「compared=40 differ=0」「compared=75 differ=0」）。判决引的数从复跑输出里现数：K2-b 的 rf 系 20 组红 10 组、rfh 系 9 组全红，合 29 组红 19 组；X 读得出的 20 组红 0；开 B1 的 rfx 系 9 组红 0；`rfo3-17-persist.out` 那一行「AFTER_RAISE newest=31 (was 29) x_still_on_disk=Some(21) red=["I-3.1：盘 0：记账的已分配 Some(1146880)，遍历全部有效根得到 1245184"]」。层 0 枚举那几件（k2e-enum、k4-enum 与 `rfe-sw0/1`）与随机历史批 b04、b13、b15 没复跑，层 0 的态数引腿报告原行。正推腿的模型没复跑；它的 H1–H4 主 agent 只现查了代码那一半（H1 两处行号）。checker 档没跑。

## 五、被判的文件

这一轮判的 `crates/` 文件（冻结副本）：`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`、`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/journal.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/root_record.rs`、`crates/singlefs-core/src/root_ring.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-format/src/lib.rs`，以及 `crates/singlefs-harness/tests/` 下 19 份（清单在 `research/prompts/m3-root-ring-code-r1-snapshot/crates-sha256.txt`）。

第二轮攻 RR1–RR5 落地之后的样子与这一轮站住的各格（一次抽样的「没打中」要再抽一次），换攻击面：第一轮的修补本身、产生结论的方法、没被看过的地方。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-03 不受影响：K2-e、K2-b2 两条交用户，定之前不改条款；B1 照条款现有的「门槛按最小的算」补漏 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-03 不受影响：实现照 R2，正文措辞进写回批 |
| D23（journal 的角色与格式） 已定项 15 | 不影响 | 2026-10-03 不受影响：七项施加照条款，checker 卸载记号那一判照用户定案补 |
| D22（单元原子性怎么合成） 已定项 8 | 不影响 | 2026-10-03 不受影响：R2 谓词与射程逐字相同 |
