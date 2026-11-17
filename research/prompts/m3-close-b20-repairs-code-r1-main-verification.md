# m3-close-b20-repairs-code-r1 判决：第 20 批修补的代码轮（2026-11-09）

正文 `research/prompts/_m3-close-b20-repairs-code-r1-body.md`；腿报告：辩方 `research/prompts/m3-close-b20-repairs-code-r1-sonnet-output.md`（sha256 fe2b740e…），攻方 `research/prompts/m3-close-b20-repairs-code-r1-opus-output.md`（sha256 5dd19342…）。开工快照目录：`research/prompts/m3-close-b20-repairs-code-r1-snapshot/`。

## 这一轮被判过的文件

- crates/singlefs-core/src/mount.rs：`first_txg_of_new_instance` 与删掉的 `FirstTxgOfTheNewInstance` 一族
- crates/singlefs-verification/src/mount_transparency.rs：判官按本池 fsid 筛，崩溃观察者共用的造法
- crates/singlefs-verification/src/verification_read.rs：`choose_system_configuration_of_the_pool`
- crates/singlefs-verification/tests/common_unwitnessed_publish/mod.rs：`image_with_the_newest_rotation_of_each_device_not_written`

## 一、核查

- 快照倒推（任务 11693）：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-judge -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m3-close-b20-repairs-code-r1 --out /home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-judge/tree --repository /home/fy5090/code/singlefs`，「快照 m3-close-b20-repairs-code-r1：清单点名 8 份，倒推出 8 份，sha256 对上 8 份」。
- 引文核对：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b20-repairs-code-r1-sonnet-output.md research/prompts/m3-close-b20-repairs-code-r1-opus-output.md --root /home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-judge/tree --background research/prompts/_m3-close-b20-repairs-code-r1-background.md`，「✗ 核了 5 处引文、148 处裸行号，1 处对不上，150 处没判」。对不上那一处是攻方报告第 58 行引 `mount.rs:1512-1513`：原句「再取一次是为了两次读环之间的瞬时读错（择根那一遍读到了它、这一遍没读到）不把水位拉低」主 agent 现读在 `tree_identifier_watermark_of_the_ring` 的文档注释里，行号写错、句子是真的。下面用到的行逐处现读。
- 主 agent 现读：`crates/singlefs-core/src/mount.rs` `first_txg_of_new_instance`（取环里与记录的最大值，不收所选那一版）；`tree_identifier_watermark_of_the_ring`（多取一次 max）；`crates/singlefs-core/src/system_configuration_repair.rs` `(true, None)` 那一支拒；`crates/singlefs-verification/src/mount_transparency.rs` 第 3353 行注释「这一支手里没有本池 fsid，择系统配置不筛 fsid：它判的崩溃态由一个池的写流造出，盘上没有别的池的槽」。
- 攻方用例在它的副本上复跑（任务 11691）：「test result: ok. 6 passed; 0 failed」，末行 `ATTACK_FIRST_TXG_SUMMARY cells_at_or_below_the_selected_version=3`；造法盘面（任务 11692）`ATTACK_FIXTURE tail_of_p=6 tail_on_the_crash_image=6 tail_on_the_fixture=5`。这是副本上的数；用例转正入库之后要在入库的代码上再判一次，才算修完。证红 11694 交了、等结局。
- 辩方腿自报误读了禁读清单里 `m3-close-b20-code-r2-sonnet-output.md` 前 80 行，发现后停下。它的结论这里只当线索，采纳的逐条现查，见第二节。
- 辩方列的只读核脚本与「GPU 那条两台各跑 3 次」没有复跑：前者的结论「删变异行没删多」靠 11594 的结局再核，后者归 Xid 109 那一件。

## 二、逐格判决

| 格 | 判 | 依据 |
|---|---|---|
| H1 取第一个 txg 那一遍不与所选那一版取 max | **打中（实测，产品缺陷）**：读阶段读到 D、这一遍读坏，写行 txg 等于所选那一版，违 `.claude/kb/decisions/23-journal的角色与格式.md` 第 373 行，3 格池级 checker 判 `I-3.12` 违例 | 11691 `ATTACK_FIRST_TXG only_this_read=true k=3 reads=9 mount=ok effective=6 row=6 … violated=["I-3.12"]` 等 3 行 |
| H2 判官吃的 `selected_version` 来自不筛 fsid 的恢复 | **打中（实测）**：合法的拒判成不合法，`verdict_with_the_recovered_version=Illegal` | 11691 `ATTACK_JUDGE` 行 |
| H3 只有一块盘带高世代别池槽：可写做成、只读报 `SystemConfigurationsDisagree` | **打中（实测）**，用户定收 fsid，第三节 | 11691 用例绿 |
| H4 别池槽区域数改坏时判官与产品分叉 | **打中（实测）**：`ATTACK_REGION_COUNT verdict=Illegal`。这一槽第一版造不出，与 C975（别池又布局位不认的槽没定优先次序） 同优先级；判官读法照产品改 | 11691 |
| H5 新造法的盘面要两块盘都丢一次已确认的写，不是任何崩溃态 | **打中（实测）**：`tail_on_the_fixture=5`，真崩溃态是 6；注释「就是这个样子」不实 | 11692 |
| H6 新格式化池上撕裂取号一轮，可写、只读、离线三路都走不通 | **打中（实测）**：今天的 `make_filesystem` 加今天的 `mount_writable` 真写流撕裂出来，不是旧格式。照用户判据归「我们自己代码的问题」，要支持修复 | 11691 `ATTACK_FORMATTED` 六行；用例造盘函数现读 |
| 崩溃态两支判官没加 fsid（辩方） | **打中，推的、待量**：第 3353 行的前提与同目标里手造别池槽的用例（辩方点名 `writable_mount_refusal_after_the_read_stage_counts_self_verified_slots_by_the_pool_checker.rs:127`）对不上；先写一条会红的用例再改 | 辩方报告；注释现读 |
| 判官加 fsid 修到 r2 的 A1、A2 | **成立**（辩方；攻方同），H2、H4 是另两条路 | 两份报告 |
| 取号这一形是用户定的「拒，留两轮」 | **成立**，但「离线出路也拒」是测出的后果，不是用户原话；照 H6 改 | `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2606 行 |
| B1 两块盘各是不同别池 | 轻（辩方）：恢复报 `SystemConfigurationsDisagree`，不是条款写的 `NoValidRoot`；写回 D22 | 辩方报告 |
| B2、C973 射程句 | **成立**（辩方，读码推的） | 辩方报告 |
| E205 退场没删多 | **成立到读码**（两条腿）；删少两处轻：测试文件第 92、102、414、570 行注释指着已删的名字，第 20 行的 `git show HEAD:…` 提交后失效 | 两份报告 |
| 检查单八格 | 走不到的理由：辩方「不成立」归崩溃态两支那一格，同上。不兼容旧代码：轻，同上一行。带 --item：11593、11592 抓到 22 条，11594 等结局；正文把 11066 写成 `--list-items` 是错的，实际是 `crash-case-check.py`。多机可用：Xid 109 那一条在这一批之外另修。其余成立 | 两份报告 |

## 三、用户定案（2026-11-09 弹窗）

| 事 | 用户原话 | 落点 |
|---|---|---|
| H6 | 「这个问题我们需要辨别一下，这是改代码造成的。还是测试造成的。 如果是改代码造成的，按照我们的计划，我们不需要兼容过去的格式，直接重新创建盘就好了。 如果是我们自己代码的问题，那么就要支持修复。」 | 现查归后者：离线修复支持第 0 代 |
| H3 | 「选项2  管理员需要只读」 | 只读挂载与恢复收本池 fsid |
| 变异行键作废 | 「可以作废 并且可以在本次提交的全量门禁中重跑 问题不大」 | 判法改动不为它收窄 |

## 四、被攻过零轮的

第五节的全部改法。这一轮是这一题第一轮；改完之后下一轮代码轮攻它们。

## 五、去向

| 事 | 去向 |
|---|---|
| H1：`first_txg_of_new_instance` 收所选那一版取 max | `implementation-writer`，这一批 |
| H3：只读挂载与恢复收本池 fsid，用例 | 同上 |
| H6：离线修复断言第 0 代时 tail 取 0，用例 | 同上 |
| H2：判官的 `selected_version` 走同样收 fsid 的恢复，不从被判的挂载取 | 同上（靠 H3 那一处接口） |
| H4：判官数别池槽照产品的判法 | 同上 |
| 崩溃态两支：先写会红的用例，再加 fsid | 同上 |
| H5：造法注释与用例名写明丢写；写路径那几支留不留写进 C971 射程 | 同上，kb 归写回批 |
| E205 残留注释，第 20 行 `git show` | 同上 |
| 六条攻方用例转正：断言改成修完之后的样子，配变异行 | 同上 |
| D22 已定项 8 射程：B1、H3、H6 三句；C492 | 写回批 `kb-scribe`（三语） |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 8 | 支撑 | 2026-11-09 改了：别池单盘一形、离线出路在第 0 代成立，写回批改射程 |
| D23（journal 的角色与格式） 已定项 14 | 支撑 | 2026-11-09 不受影响：第 373 行条款不动，代码改到符合它 |
