# m4-gpugc-precheck-b-a-code-r2：判决（2026-11-12）

正文 `research/prompts/_m4-gpugc-precheck-b-a-code-r2-body.md`，改动全文 `research/prompts/_m4-gpugc-precheck-b-a-code-r2-diff.md`，开工快照 `research/prompts/m4-gpugc-precheck-b-a-code-r2-snapshot/`。这是这一题的第二轮；第一轮判决 `research/prompts/m4-gpugc-precheck-b-a-code-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端辩方（Sonnet） | `research/prompts/m4-gpugc-precheck-b-a-code-r2-sonnet-output.md`，模型 `research/prompts/m4-gpugc-precheck-b-a-code-r2-sonnet-model/` | 第一轮「底 19351」口径宽了、「先乙后甲不打中」引的两个 0 是结构上恒为 0 的计数、「越过 `N₀` 的 0 串」没报分母；「甲的前提不成立」只够得着注释有缺口；拒绝报少出路不是乙甲带来的；检查单八格成立 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-output.md`，模型 `research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-model/` | 第一轮改法第 1 条漏改 `mounted_session.rs:686`–`688`，另三处同前提（`:658`、`:2937`、`:6828`）；判决只用了推高的 24 串；留 `u64` 的那个读数与 `admission.rs` 是同一个量两种表示，「类型与名字」不成立；其余成立 |

主 agent 的核查（量的标「量的」）：

1. 快照倒推任务 14767 退 1：「清单点名 6 份，倒推出 5 份、sha256 对上 5 份；倒推不出 1 份」。倒推不出的是 `.claude/kb/checks-owed.md`（别的会话在快照之后改过，HEAD 也不是快照那一份）；腿引它的两处都是第 607 行 C1003（被宽护卡住那条出路没量过），主 agent 核这一行与 C1002（抬 F 先算对槽对那一判挡得少）那一行此刻的内容与快照时抄进附录的逐字节相同（sha256 前缀 `9ad60a97cf7c`、`6360bf561b76` 两边一样）。`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-output.md research/prompts/m4-gpugc-precheck-b-a-code-r2-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2-main-check/tree --background research/prompts/_m4-gpugc-precheck-b-a-code-r2-background.md` 退 0，末行「✓ 核了 31 处引文、136 处裸行号，对上 23 处，没判 144 处（其中裸行号没有引文可比 69 处；逐处列在上面）」，没有对不上的。判决依赖的行主 agent 在冻结副本里现读：`mounted_session.rs` 第 686–688 行仍写「推了也写不进的写不花根环的槽与写盘」，第 658、2937、6828 行都把这个成员写成「推抬 F 放得出的全放出来也放不下」而没提甲那一形；`admission.rs` 第 980 行 `pub slot_pairs_for_data_units: u64`。
2. 辩方探针：模型拷到 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2-main-check/sonnet-model`（逐份 sha256 与腿的目录相同），任务 14740 第 4 次尝试退 0，`out/oracle.out` sha256 前缀 `32c69feb2c70caaa`，自证 `name=self_checks bad=0`（量的）。`PROBE_RESULT` 前 30 个字段与第一轮任务 13926 的逐字段相同（310 个配置，同一批种子）。这一轮新拆出的（整行在产物里，下面抄计数行）：
   - `oracle COUNT fired by=slot-pairs oracle-series=0 825`、`oracle COUNT fired by=slot-pairs oracle-series=1 1082`
   - `oracle COUNT fired by=slot-pairs old-bound=fires-by-slot-pairs 620`、`oracle COUNT fired by=slot-pairs old-bound=pushed-then-no-space-wide-protection-not 1243`、`oracle COUNT fired by=slot-pairs old-bound=fires-by-conjunct-before-any-series 44`
   - `oracle COUNT remount published by=slot-pairs old-bound=pushed-then-no-space-wide-protection-not 10`、`oracle COUNT remount published by=slot-pairs old-bound=fires-by-conjunct-before-any-series 2`
   - `oracle COUNT fired by=slot-pairs count-below-floor=yes 62`
   - `oracle COUNT conjunct-first refused 413`、`oracle COUNT conjunct-first forced=NoSpaceAfterRaisingTheFloor 413`，`conjunct_first_forced_published=0`
   - `oracle COUNT series arm=product empty-segments=lowered 6846`、`raised 24`、`same 24349`；`series_product_short_before=0`、`series_oracle_short_before=59 series_oracle_short_before_raised=0`、`series_forced_short_before=362 series_forced_short_before_raised=0`
   - `oracle COUNT configs fired by=conjunct-before-any-series 2`
3. 攻方探针（换一族：先填满再打碎，全空段压到 0）：模型拷到 `…-r2-main-check/opus-model`（逐份 sha256 相同）。头一次交的 14743 在第二台往共享工作根拷仓副本时卡在读写等待上（管任务服务的会话量到是那条 NFS 导出同步写），主 agent 取消，把核查目录里那份 `tools/rerun.sh` 的仓副本改拷到任务所在机器的本地临时目录（只换副本放哪，探针源码没动；腿交来的原样留在 `…-r2-main-check/opus-rerun.sh.as-delivered`），重交 14746 退 0，`out/oracle.out` sha256 前缀 `30a9b0ce5b288e41`，自证 `bad=0`（量的），整行：

   `oracle PROBE_RESULT configurations=155 steps=38750 changes=41544 fired=4994 fired_by_slot_pairs=1039 fired_by_formula=312 fired_by_mixed=0 fired_by_conjunct_before_any_series=3643 conjunct_fired_with_zero_empty_segments=3643 conjunct_fired_with_empty_segments_above_zero=0 fired_both_members_empty=0 fired_both_members_set=0 conjunct_fired_with_versions_published=0 misrefused_slot_pairs=0 misrefused_formula=0 misrefused_mixed=0 misrefused_conjunct_before_any_series=0 remount_published_slot_pairs=187 remount_published_formula=0 remount_published_mixed=0 remount_published_conjunct_before_any_series=32 oracle_pushed_more_series_than_the_product=1181 base_published_after_a_series=3920 conjunct_before_any_series_judged=14407 conjunct_before_any_series_found=3643 series_product=5651 series_product_raised=4 series_product_lowered=963 series_product_mixed=0 series_product_crossed_the_floor=0 series_oracle=1188 series_oracle_raised=1 series_oracle_lowered=56 series_oracle_mixed=0 series_oracle_crossed_the_floor=0 steps_with_zero_empty_segments=18410 fills=8 rebuild_checked=41544 rebuild_differ=0 rebuild_panicked=0 panics=0`
4. 这一批在主工作区上的验证：合入后验证 14552「测试全绿（2 个测试二进制，共 13 条测试；跑了 2 项（2 个包）；按条复用：现判 13、复用 440、拒算 0）」、fmt 过、build 过、clippy default 绿；这一批改到的两个包的全特性 clippy 14703 退 0（量的）。

## 二、判决

对第一轮判决的复核（辩方腿 T2，主 agent 照第一节的读数逐条核）：

| 第一轮的结论 | 这一轮的读数 | 判决 |
|---|---|---|
| 乙在槽对那一判上误拒「这一份取样没撞上」，引底 19351 | 19351 是整族的底；槽对那一判判出的 1907 格里 O 那一臂真推了至少一串的是 1082 格、写进 0；第二份互不相同的一族 1039 格、写进 0（量的） | **更正分母**：有效分母是 1082 与 1039，不是 19351。两份样本都没撞上，记「没打中」 |
| 甲误拒「分母太小」（4 格） | 第二族甲判出 3643 格、O 那一臂写进 0；第一族 4 格落在 2 个配置里（量的） | **记「没打中」**：两份样本都是 0，第二份分母 3643 |
| 甲文档的前提不成立（打中） | 产品那一臂 31219 串：压低 6846、推高 24、不变 24349；起步时全空段已少于 `N₀` 的串，O 那一臂 59 串、强推一臂 362 串，推高 0 串；合取 7 自己先拒的 413 格强推之后写进 0（量的） | **收窄**：打中的是「注释给的理由不全」（只讲了回收 defer，没讲换下固定点单元），不是「推了能写进」。第一轮改过的两处文档与这些读数相容，不用再改措辞 |
| 「越过 `N₀` 的 0 串」 | 产品那一臂起步就少于 `N₀` 的串是 0（它那时不推），这个 0 在产品臂上是结构的；有分母的是上一行的 59 与 362（量的） | **更正**：引 0/421，不引产品臂的 0 |
| 拒绝报里少一条卸载重挂出路（打中） | 第一族那 12 格换回老上界：10 格是推一串后 `NoSpaceAfterRaisingTheFloor`、宽护那一报不出，2 格由甲判出；老上界下同样不报出路。第二族槽对那一判 187 格、甲 32 格重挂后写进（量的） | **打中的是先算这一类拒绝本来就不报这条出路，不是乙、甲带来的**；并进 C1003，交用户（第三节第 6 条） |
| 先乙后甲「不打中」，引 `fired_both_members_*` 两个 0 | 那两个计数是构造上恒为 0 的；槽对那一判判出而全空段同时少于 `N₀` 的是 62 格，报的都是 `devices_short`（量的）。产品代码没有读这两个字段分支的（两腿读码一致） | **结论留着，依据更正**：两样同时成立时只报槽对那一判，这一形在取样里有 62 格，写进射程 |

这一轮新判的：

| 条 | 读数与依据 | 判决 |
|---|---|---|
| 第一轮改法第 1 条漏改（攻方 T1-2） | 主 agent 现读冻结副本第 686–688 行、第 658、2937、6828 行 | **打中**：四处文档都没把甲那一形说对，第三节第 1 条 |
| 留 `u64` 的 `unblocked_slot_pairs_outside_the_cluster_segments` 与 `admission.rs` 的 `slot_pairs_for_data_units`（攻方 T1-3，辩方判留着站得住） | 两腿读码一致：产品里没有把槽对数与槽数混着比或加的地方；同一个量此刻一半带类型一半不带 | **照字面打中**（检查单「类型与名字」）：换齐，第三节第 2 条 |
| 两条新真池用例钉的形 | 两腿各自读码：各在点名的变异行下红在用例自己的那一句；走空那条没断言这一步之前没推过串、没发成版 | **成立**；走空那条补两句断言，第三节第 3 条 |
| 重建分配器之后三个计数 | 第二族每一步按记录重建比一次：`rebuild_checked=41544 rebuild_differ=0`（量的）；产品这边没有用例钉 | **成立**；补一条小用例，第三节第 4 条 |
| `accept` 的分类、`devices_short` 为空时的调用方、`SlotPairCount` 的求和与算术 | 两腿读码一致，攻方试了 8 形造不出错分 | **成立** |

检查单：

| 格 | 判决 |
|---|---|
| 类型与名字 | **不成立**（攻方照字面；辩方判成立）：采攻方，第三节第 2 条 |
| 走不到的理由、不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、测试优先多线程、带 --item 的命令与项、新崩溃流多罩了什么 | 成立（两腿一致） |

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-core/src/allocator.rs` | **要改**：第三节第 2、4 条 |
| `crates/singlefs-core/src/mounted_session.rs` | **要改**：第三节第 1 条 |

## 三、改法

1. `mounted_session.rs` 四处文档（结构体 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 的文档、`UserChangeRefused` 那个成员的文档、第 2937 行与第 6828 行两处方法文档）把两形分开写：槽对与式子那两判是推抬 F 放得出的全放出来也放不下；合取 7 那一形是这次调用还一串没推、写之前那一刻全空聚簇段已少于 `N₀`，不推。删掉「推了也写不进的写不花根环的槽与写盘」当作合取 7 那一形的理由。量过（主 agent 现读四处；任务 14740、14746）。
2. `unblocked_slot_pairs_outside_the_cluster_segments` 的返回值与 `admission.rs` 的 `PlacementRoomOnOneDevice::slot_pairs_for_data_units` 换成 `SlotPairCount`，读它们的地方（准入里的比较、实验装置 E180、E181 两份源码、验证包测试）跟着换。量过（两腿读码；E180、E181 的实验页状态写着读数产物还没跑出，没有留存产物要重出）。
3. 走空那条真池用例 `a_walk_step_refused_for_space_while_empty_cluster_segments_are_below_the_floor_is_pushed_for` 补断言：报里这一步之前 `versions_published` 与不该有的串是空的（具体断哪几个字段由实现员照用例造的池定，推的）。
4. `allocator::tests` 补一条：一段有分配、释放、回收的历史之后 `rebuild_from_records`，三个槽对计数与重建之前相等。量过（任务 14746 `rebuild_differ=0`）。
5. 射程补写（kb，写回员，三轮判完做）：照第一轮第三节第 3 条，另加这一轮更正过的读数口径（有效分母 1082 与 1039、3643；0/421；两样同时成立只报槽对那一判的 62 格）。
6. **被攻过零次，不改代码**：先算判出放不下时报里带不带卸载重挂那条出路。这一轮量到它不是乙、甲带来的（老上界下同样不报），两族里重挂写得进的是 12、2 与 187、32 格；改法本身没被攻过、没有 A/B，与 C1003 一起交用户。

改完开第三轮（最后一轮），只攻前两轮站住的形态。

## 交用户的

- 第三节第 6 条：先算拒绝时带不带卸载重挂那条出路。
- 乙、甲的误拒各有两份互不相同的取样、都是 0，记「没打中」，第一轮登的那条欠账可以关。

## 这一轮被判过的文件

- crates/singlefs-core/src/allocator.rs：判了 `SlotPairCount` 换得全不全（一个读数留着裸整数）、重建之后三个计数有没有用例钉，要改第三节第 2、4 条
- crates/singlefs-core/src/mounted_session.rs：判了第一轮改的两处文档与读数相容、另四处文档还把合取 7 那一形说成放不下，要改第三节第 1 条

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 10 | 支撑 | 2026-11-12 改了：射程补写的内容加这一轮更正过的读数口径（第三节第 5 条，写回员） |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-12 不受影响：卸载重挂那条出路并进 C1003，条款不动 |
