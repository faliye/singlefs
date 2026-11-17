# m4-gpugc-precheck-b-a-code-r1：判决（2026-11-12）

正文 `research/prompts/_m4-gpugc-precheck-b-a-code-r1-body.md`，改动全文 `research/prompts/_m4-gpugc-precheck-b-a-code-r1-diff.md`，开工快照 `research/prompts/m4-gpugc-precheck-b-a-code-r1-snapshot/`。这是这一题（改法乙、甲写进 `crates/`）的第一轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpugc-precheck-b-a-code-r1-sonnet-output.md` | 乙的增量计数每条路都跟上；乙误拒读码找不到、A/B 没有区分力；乙少报的出路是卸载重挂、现行同样有这个缺口；甲的「推串之前」是实现员的读法、射程原文没有；串里自己的空发布可能让全空段数加一（推的）；检查单「类型与名字」「测试优先多线程」不成立 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-output.md`，模型 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/` | T2-1 甲的前提站不住（推的，待探针）；T2-2「甲误拒 0」没有底；T1-b 乙丢的卸载重挂不一定走不通；T1-c 槽对那一判造不出误拒；检查单同两格不成立 |

主 agent 的核查（量的标「量的」）：

1. 快照倒推任务 13936 退 0：「清单点名 4 份，倒推出 4 份，sha256 对上 4 份」（量的）。`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-output.md research/prompts/m4-gpugc-precheck-b-a-code-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1-main-check/tree --background research/prompts/_m4-gpugc-precheck-b-a-code-r1-background.md` 退 0，末行「✓ 核了 85 处引文、99 处裸行号，对上 57 处，没判 127 处（其中裸行号没有引文可比 70 处；逐处列在上面）」，没有对不上的；判决依赖的行主 agent 在冻结副本里现读：`mounted_session.rs` 第 6170–6174 行甲的文档写「抬 F 回收 defer 里的槽不改这个数……推了这次写也是被合取 7 拒」，第 2120–2121 行 `refusal_is_short_of_space` 注释同一句；`allocator.rs` 第 2121 行聚簇段集只在内存、重开后重来；攻方说的「第四个入口」推环预演（`mounted_session.rs` 第 1890 行）调的是 `publish_the_change_raising_the_floor_for_space`，与 `publish_user_change` 那一处（第 3838 行）同一条路，不是另一个调用点。
2. 攻方探针：主 agent 拷模型到 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1-main-check/opus-model`（逐份 sha256 与腿的目录相同），交任务 13926，第 2 次尝试退 0；`out/oracle.out` sha256 前缀 `82a71415e825c942`（量的），整行：

   `oracle PROBE_RESULT configurations=310 steps=124000 changes=145155 fired=13425 fired_by_slot_pairs=1907 fired_by_formula=11514 fired_by_mixed=0 fired_by_conjunct_before_any_series=4 fired_both_members_empty=0 fired_both_members_set=0 conjunct_fired_with_versions_published=0 misrefused_slot_pairs=0 misrefused_formula=0 misrefused_mixed=0 misrefused_conjunct_before_any_series=0 remount_published_slot_pairs=12 remount_published_formula=0 remount_published_mixed=0 remount_published_conjunct_before_any_series=2 oracle_pushed_more_series_than_the_product=7217 base_published_after_a_series=19351 conjunct_before_any_series_judged=56681 conjunct_before_any_series_found=4 series_product=31219 series_product_raised=24 series_product_mixed=0 series_product_crossed_the_floor=0 series_oracle=7300 series_oracle_raised=7 series_oracle_mixed=0 series_oracle_crossed_the_floor=0 panics=0`

   自证 `name=self_checks bad=0`（四格：甲钩子跑到过、推串钩子跑到过、O 那一臂先算关得掉、没有 panic）；`EXAMPLE MISREFUSED` 0 行，`EXAMPLE REMOUNT_PUBLISHED` 14 行（槽对那一判 12、甲 2）（量的）。
3. 崩溃点重放带 `--item` 三项，缩到展开深度 4 的任务 13913 退 1：放量三项的判器红数都是 0（`red=0`），判红是因为缩域后有段落在展开之外（`unregistered_segments_beyond_expansion`）；`truncate-to-zero-on-a-nearly-full-pool-stream` 那条用例判段报「观察者 mount_continuing_the_truncation_at_the_edge 在点 …journal_record.1 块 0 的录入段那一块不在库里」，不是断言判出的违例。照它的出路不缩域单跑这一条，交任务 13928（写判决时在跑）。这一趟缩过域，不算门禁通过。

## 二、判决

| 条 | 读数与依据 | 判决 |
|---|---|---|
| 乙在槽对那一判上误拒（正推 T1、攻方 T1-c） | 槽对那一判判出 1907 格，O 那一臂写进 0（`misrefused_slot_pairs=0`）；这一族推串之后写得进的底 19351，有区分力（量的）。两腿读码都找不到上界小于推串真放得出的路 | **这一份取样没撞上**；一份样本不撑「没打中」，登欠账 |
| 甲的前提（攻方 T2-1、正推 T2）：推了这次写也是被合取 7 拒 | 产品那一臂推的 31219 串里 24 串把全空段数推高（`series_product_raised=24`），O 那一臂 7300 串里 7 串（量的）；越过 `N₀` 的 0 串，甲误拒 0 格 | **注释写的前提不成立，打中**：`mounted_session.rs` 第 6174 行、第 2120–2121 行两处文档要改成量到的样子 |
| 甲误拒（攻方 T2-2） | 甲判了 56681 次、判出 4 次，误拒 0（量的）；分母 4 | **这一份取样没撞上，分母太小**；不撑「没打中」，登欠账 |
| 乙、甲先拒之后卸载重挂写得进（正推 T1、攻方 T1-b） | 槽对那一判判出的 1907 格里 12 格重挂之后写进，甲 4 格里 2 格（量的）；拒绝报里没有这条出路 | **打中**：先算判出放不下时，有一条走得通的出路没报。现行上界同一形丢不丢没量（正推读码说现行同样有，推的）。改法是零轮，交用户、并进 C1003 |
| 甲射程比 A/B 探针宽一形（先走空发成几步再判） | 两腿读码一致：与射程括注「走空之后才判出」同类 | **成立**，射程补写（第三节第 3 条） |
| 先乙后甲，两样同时成立只报乙 | 探针 `fired_both_members_empty=0`、`fired_both_members_set=0`（量的）；产品代码没有读这两个字段的 | **不打中**，写进射程 |
| 改 size、建快照不先判 | 两腿一致：照推那一边，只多白推一串，不误拒 | **成立** |
| 两处调用（`publish_user_change`、走空）没有真池用例 | 两腿读码：两处传的类型都对，变异表没有点这两处的行 | **打中（覆盖）**：补变异行与用例，第三节第 2 条 |
| 每次发布克隆分配器多一张每槽一字节的位图（正推 T3） | 按 1 TiB、每槽 16 KiB 每次每盘多 64 MiB（算的） | **不判违规**：`.claude/rules/fs-design.md`「记账是事务的副产品」管的是遍历，不管克隆；登欠账量代价 |

检查单：

| 格 | 判决 |
|---|---|
| 走不到的理由 | 成立（两腿逐处追过 `allocator.rs` 第 514 行那一条） |
| 类型与名字 | **不成立**（两腿一致）：槽数与槽对数共用裸 `u64`，第三节第 4 条 |
| 不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、带 --item 的命令与项、新崩溃流多罩了什么 | 成立（两腿一致） |
| 测试优先多线程 | **不成立**（两腿一致）：`allocator::tests` 两条 600 步伪随机历史没写不并行的理由，第三节第 5 条 |

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-core/src/allocator.rs` | **要改**：第三节第 4、5 条 |
| `crates/singlefs-core/src/mounted_session.rs` | **要改**：第三节第 1、2、4 条 |

## 三、改法

1. 改文档两处（`mounted_session.rs` 甲那一判的文档、`refusal_is_short_of_space` 合取 7 那一臂的注释）：推串里的空发布会换下固定点单元，可能把全空段数推高；判的是写之前那一刻的数，推高越不越过 `N₀` 不另估。量过（任务 13926：24/31219 串推高、0 串越过）。
2. 补覆盖：`publish_user_change` 那一处与走空那一处各传哪一种，各加一行变异（传反）与一条会让它红的用例；用例放得进库单测就放库单测，放不进照第 126 批真池用例那一形。量过（两腿读码一致缺）。
3. 射程补写（kb，写回员）：D16 已定项 10 射程那一句照实现员报告「要改的条款」一节的新文改，并补「先走空发成几步之后判出的同样判」「两样同时成立只报槽对那一判」两句；C1002 去向改成实现那两处、验收形写成第 126 批两条真池用例。
4. 槽对数立一个 newtype（与槽数分开），`SlotsAFloorRaiseCouldRelease`、分配器那三个字段与读它的函数一起换。量过（两腿读码一致）。
5. `allocator::tests` 两条伪随机历史用例上写一行为什么不并行（每一步读上一步的分配器状态，次序就是被测对象）。
6. **零轮，不改代码**：先算判出放不下时，C 那一圈罩着、这次挂载开过的聚簇段里放得开的槽对补得上差额，报里带上卸载重挂那条出路（攻方第二轮改法一同形）。这一形这次有了 14 格读数（任务 13926），但改法本身没被攻过、没有 A/B；写进产品代码之前先在探针里做 A/B，与 C1003 一起交用户定。

改完开第二轮，攻第 1、2、4、5 条改过之后的形态。

## 交用户的

- 第三节第 6 条：先算拒绝时带不带卸载重挂那条出路（零轮；取样里 1907 格中 12 格、4 格中 2 格重挂写得进）。
- 乙在槽对那一判上、甲的误拒都只有一份取样（甲分母 4），登欠账，要不要另立一份取样量。

## 这一轮被判过的文件

- crates/singlefs-core/src/allocator.rs：判了乙的增量计数（各路都跟上）、槽数与槽对数共用裸整数、伪随机历史用例不并行的理由，要改第三节第 4、5 条
- crates/singlefs-core/src/mounted_session.rs：判了乙的上界、甲的射程与次序、甲文档里「推了也被合取 7 拒」的前提、两处调用的覆盖、拒绝报里少的卸载重挂出路，要改第三节第 1、2、4 条

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 10 | 支撑 | 2026-11-12 改了：射程那一句要补乙的上界、甲与两句边角（第三节第 3 条，写回员） |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-12 不受影响：宽护那条出路的缺口并进 C1003，条款不动 |
