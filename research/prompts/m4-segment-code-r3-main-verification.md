# m4-segment-code-r3 判决：里程碑四预开发整段代码轮第三轮（最终轮，主 agent，2026-10-16）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 R1 R6 R7 R8 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 F1 F2 F3 F4 F5 F6 H1 H2 H3 H4 H5 H6 J1 J2 J3 J4 J5 J6 J7 G3 G4 G5 U2 -->

被判的产品代码与前两轮同一组 20 份（冻结副本 `~/.local/share/singlefs/work/m4-segment-code-r3/repo/`，sha256 在 `m4-segment-code-r3-snapshot/crates-sha256.txt` 21 行），修复批三改过的是 `crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-reference-filesystem/src/lib.rs`。正文 `research/prompts/_m4-segment-code-r3-body.md`，背景材料 `_m4-segment-code-r3-background.md`。这是最终轮，之后不再开轮。

## 一、核查

- 快照：两条腿各自在冻结副本根上 `sha256sum -c` 21 行全 OK（两份报告开头原样）；腿跑着的时候主 agent 没改工作树。
- cite-check（以工作树为根）：攻方 `✓ 核了 20 处引文、0 处裸行号，对上 20 处，没判 0 处`；辩方 `✓ 核了 39 处引文、47 处裸行号，对上 39 处，没判 47 处（其中裸行号没有引文可比 47 处；逐处列在上面）`。
- 复跑：攻方 `rerun.sh` → `name=rerun_total same=3 differ=0`（`~/.local/share/singlefs/work/m4-segment-code-r3-main-rerun-attack.log`）；辩方的数核工具 `check-numbers.py` 主 agent 重跑（`~/.local/share/singlefs/work/m4-segment-code-r3-main-check/check-numbers.txt`），输出与腿交的 `out/check-numbers.txt` 逐字相同，末行 `name=check_total ok=59 differ=2`。辩方另把前两轮四条腿的 `rerun.sh` 在新目录各跑一遍，全部与腿交的相同（主 agent 前两轮各自复跑过同一批，结果相同，这一次不重跑）。
- 主 agent 现查：`crates/singlefs-core/src/transaction.rs` 第 7505 行 `publish_relocation_batch` 的文档注释「准入同删除类（式子那一判照判，合取 7 免）」在（攻方 Q3 第三条）。

## 二、判定与去向

| 格 | 判定 | 依据 | 去向 |
|---|---|---|---|
| Q1 前两轮站住的 | 攻方换「一次发布写到一半崩溃」的攻击面，删快照流全量 8215 态全过；唯一的红是 I-3.1 记账多的一边，`05-空间记账.md` 第 8 行用户认过的那一形，不分辨修补（一次抽样，量的）。辩方：结论都站得住，依据够不着的几处——F1「与 D23 相容」只证到可观测一层、没有一格逐字节比映射树；T7「出路」一句靠读码；T8「字面不要求补发」应写成「字面没说」；T10 读法只被辩方复核、没被攻过；T9「两次没打中」盖不到 Verus 那一半 | 两腿 | 依据的措辞照辩方改写进阶段同步的 kb 写回；T10 读法标「零轮」写进 D26 射程 |
| Q2 修复批三 | H4、H3、H2、H1 没打中（H4 2014 次整理发布无一没搬、无一超批大小，H1 144 段带回退的随机历史全 Completed；攻方，量的）；**H5 打中（判据一侧，量的）**：一版多条根时被抛弃的是同一版后一条根，H5 照取「之前那一版」把还在的快照判成没有、读它判红；RefFS 看不到被抛弃的是哪条根；今天的翻译喂不到（推的）；实现员交下来的「退路那一版模型判不合」——故障注入的比重不抽整理，冲突是哑的；「批超过 K」没造出 | 攻方；复跑 `reffs-abandoned-root.txt` 相同 | 用户 2026-10-16 定「乙：翻译交进来」→ J1（零轮） |
| Q3 清单全不全 | 两腿各列了没进任何一张表的形状：`publish_relocation_batch` 准入注释与行为不符（第三轮冻结副本同文件两处注释矛盾）；批数只数搬了单元的、用户改动之后一趟可发三次；强制进 deadlist 那一臂造得出纪元 0 的条目；瞬时读错与整理从没在对拍历史里一起出现；`history.rs` 第 543 行测试注释过时；整理搬根改变 D16（发布语义）「非空」状态；R7 欠账行、T7「记后果」行没落进 `checks-owed.md` | 两腿 | J2–J6；欠账行归阶段同步 |
| Q4 判决引的数 | 59 项对上、2 项对不上：第一轮判决 T5 与第二轮判决 R1 引的「24 格红」实际是没走完的 24 格、池级 checker 违例分布在 34 格；第二轮判决 R1 引的种子 950093「模型 9、实现 11」不在辩方腿交的 `out/seeds.txt` 里（那一行截断），辩方新跑的 `out/seed950093-ceiling.txt` 里有这两个数。前两轮判决第一节的 cite-check 数是以那一轮冻结副本加仓根现算的，不是「以工作树为根」；四条腿的 `rerun.sh --selftest` 今天全红（模板自证只认模板自带的步骤） | 辩方；主 agent 重跑 `check-numbers.py` 相同 | 两处数的更正写在这一节，前两轮判决原样不改；`rerun.sh --selftest` 归提交前门禁判，红了照门禁出路改 |

两腿分歧：没有。攻方的 H5 打中与辩方复核第二轮判决「H5 一句与 U2 相反、第三轮正文已改」说的是同一处的两面。

## 三、改法（修复批四，都被攻过零轮，不再开轮）

- **J1**（用户 2026-10-16 定「乙」）：翻译把「被抛弃的根是不是写出这一版的那一条」交进 RefFS（翻译里有 `NewVersionWrittenBy`），RefFS 按它取快照集合：是写出这一版的那一条，取之前那一版的；不是，取这一版自己那一刻的；删快照这种一个操作写多条根的取前后两份。改 `restart_after_a_recovery_that_abandoned_the_newest_root` 的签名与翻译里的调用；U2「以读出内容重新起头、不判那一步」不动。RefFS 单测钉攻方那两格（建快照后挂一次再抛弃最新根、删快照后抛弃最后一批）；变异一行证红。
- **J2**：`transaction.rs` 第 7505 行一带 `publish_relocation_batch` 的准入注释照实写，与同文件另一处说法对齐。
- **J3**：`batches_published` 只数搬了单元的那几次时，用户改动之后一趟可发三次——照 G4「签发前判、无产出就停」核 `run_compaction` 一次调用至多发几次，与 G8 甲「一次调用至多签几条、发布后至多一批」对齐；用例钉次数。
- **J4**：强制进 deadlist 那一臂造得出纪元 0 的条目——开关强制时纪元照正常路径取，或断言纪元非 0；用例钉。
- **J5**：故障注入的随机历史比重里加整理，让瞬时读错与整理在对拍历史里一起出现；`history.rs` 第 543 行过时的注释照现状改。
- **J6**：整理搬根改变 D16（发布语义）「非空」状态——核 `mount.rs` 第 3414 行一带「非空」的字节比较与用户 2026-10-13 定「搬根算非空」一致，写一条用例钉住；不一致的停下报。

## 四、交给下一批的实现员（按文件聚簇）

| 簇 | 改法 | 文件 |
|---|---|---|
| 甲四 核心与 harness | J2、J3、J4、J5、J6 | `transaction.rs`、`mounted_session.rs`、`mount.rs`（J6 只读核、要改时动）、`crates/singlefs-harness/src/history.rs` 与 harness 用例 |
| 乙四 RefFS 与翻译 | J1 | `crates/singlefs-reference-filesystem/src/lib.rs`、harness 里 RefFS 翻译那一份（`reference_filesystem_translation.rs`） |

甲四与乙四都可能碰 harness：甲四不碰 `reference_filesystem_translation.rs`，乙四不碰 `history.rs`。

## 五、交用户

| # | 题 | 状态 |
|---|---|---|
| 1 | RefFS 抛弃最新根那次的快照集合（一版多条根） | 用户 2026-10-16 定「乙：翻译交进来」；被攻过零轮 |
| 2 | 攻方要主 agent 跑的单元段全量崩溃状态（带活意图的回退 65548 态，推的 25–45 分钟；整理之后回退每形 1048588 态） | 重型，不在验证段的轻型里；跑不跑交用户 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 2 | 支撑 | 2026-10-16 不受影响：J2–J5 改的是实现与用例；T10 读法（零轮）与 Q1 改写的依据措辞归阶段同步 |
| D16（发布语义） | 不影响 | 2026-10-16 不受影响：J6 只核「搬根算非空」与已定的一致 |
| D5（快照 / 空间记账机制） 已定项 14 | 不影响 | 2026-10-16 不受影响：J1 改的是 RefFS 的允许集合 |
