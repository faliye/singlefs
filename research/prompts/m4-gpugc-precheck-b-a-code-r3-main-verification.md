# m4-gpugc-precheck-b-a-code-r3：判决（2026-11-13）

正文 `research/prompts/_m4-gpugc-precheck-b-a-code-r3-body.md`，改动全文 `research/prompts/_m4-gpugc-precheck-b-a-code-r3-diff.md`，开工快照 `research/prompts/m4-gpugc-precheck-b-a-code-r3-snapshot/`。这是这一题的第三轮，也是最后一轮；前两轮判决 `research/prompts/m4-gpugc-precheck-b-a-code-r1-main-verification.md`、`research/prompts/m4-gpugc-precheck-b-a-code-r2-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpugc-precheck-b-a-code-r3-sonnet-output.md`，模型 `research/prompts/m4-gpugc-precheck-b-a-code-r3-sonnet-model/` | 正文与第二轮判决引的数逐行对上产物；「有效分母 1039」是原始分母、与 1082 口径不齐；探针读数都是第二轮冻结副本上量的；四处文档里三处没写被拒的是合取 7 判的前台写入类；15051 跑的树早于另一个会话最后一份补丁；检查单「类型与名字」照字面不成立 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-precheck-b-a-code-r3-opus-output.md`，模型 `research/prompts/m4-gpugc-precheck-b-a-code-r3-opus-model/` | 乙的上界与落得下那一判同一把尺、甲与合取 7 同一个数（读码）；甲在取落点被拒那一支上也判出，没有条款与用例；活意图 `R` 里的槽对算进了两判；位图克隆代价没落账；1039 没拆 |

主 agent 的核查（量的标「量的」）：

1. 快照倒推任务 15858 退 0：「清单点名 5 份，倒推出 5 份，sha256 对上 5 份」（量的）。`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-precheck-b-a-code-r3-opus-output.md research/prompts/m4-gpugc-precheck-b-a-code-r3-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3-main-check/tree --background research/prompts/_m4-gpugc-precheck-b-a-code-r3-background.md` 退 0，末行「✓ 核了 43 处引文、94 处裸行号，对上 33 处，没判 104 处（其中裸行号没有引文可比 73 处；逐处列在上面）」，没有对不上的。判决依赖的行主 agent 在冻结副本里现读：`mounted_session.rs` 第 699 行拒绝成员的文档写「合取 7 那一判判出时也可以是 `PlacementRefused`」，第 2120 行 `refusal_is_short_of_space` 把 `SpaceAdmissionRefused` 与 `PlacementRefused` 都算空间不够。
2. 攻方探针（今天的树，加了尺子比对与按 O 那一臂推过串的拆法）：模型拷到 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r3-main-check/opus-model`（逐份 sha256 与腿的目录相同），任务 15791 退 0，两份自证 `bad=0`（量的）。两份产物 `out/oracle-seeds-from-1.out`（sha256 前缀 `e9fc9d73601dbb26`）、`out/oracle-seeds-from-1001.out`（前缀 `526b4d644f9aa2d5`），判决引的字段：
   - 种子从 1 起（与第二轮第二族同一批种子）：`fired_by_slot_pairs=1039 … fired_by_conjunct_before_any_series=3643 … misrefused_slot_pairs=0 … misrefused_conjunct_before_any_series=0 remount_published_slot_pairs=187 … remount_published_conjunct_before_any_series=32 … rebuild_checked=41544 rebuild_differ=0 rebuild_panicked=0 ruler_checked=166176 ruler_bound_below_landing=0 ruler_bound_above_landing=160618 fired_by_slot_pairs_oracle_pushed=722 fired_by_conjunct_oracle_pushed=303 fired_by_slot_pairs_on_space_admission=1039 fired_by_conjunct_on_space_admission=309 fired_by_conjunct_on_placement=3334 fired_by_conjunct_on_other=0 misrefused_conjunct_on_placement=0 panics=0`
   - 种子从 1001 起：`fired_by_slot_pairs=996 … fired_by_conjunct_before_any_series=4166 … misrefused_slot_pairs=0 … misrefused_conjunct_before_any_series=0 remount_published_slot_pairs=159 … remount_published_conjunct_before_any_series=21 … rebuild_checked=41481 rebuild_differ=0 … ruler_checked=165924 ruler_bound_below_landing=0 ruler_bound_above_landing=160534 fired_by_slot_pairs_oracle_pushed=705 fired_by_conjunct_oracle_pushed=386 fired_by_slot_pairs_on_space_admission=996 fired_by_conjunct_on_space_admission=390 fired_by_conjunct_on_placement=3776 fired_by_conjunct_on_other=0 misrefused_conjunct_on_placement=0 panics=0`
3. 正推腿的两份旧探针搬到今天的树上重跑：模型拷到 `…-r3-main-check/sonnet-model`（逐份 sha256 相同），任务 15800 第 2 次尝试退 0，两族自证 `bad=0`；与第二轮产物逐字段比：第一族 `name=compare_probe_result fields_old=64 fields_new=64 differing=0`，第二族 `name=compare_probe_result fields_old=41 fields_new=41 differing=0`（量的）。产物 `out/random-history/oracle.out` sha256 前缀 `662e2261439eea3c`、`out/fill-then-fragment/oracle.out` 前缀 `b3067878663b35b7`。
4. 库单测四族在冻结副本上：任务 15801 第 2 次尝试退 0，「13 passed; 0 failed」，其中 `slot_pairs_once_everything_is_released_are_the_same_after_rebuilding_the_allocator_from_its_records ... ok`（量的）。

## 二、判决

对前两轮站住的形态：

| 条 | 读数与依据 | 判决 |
|---|---|---|
| 乙、甲在今天这棵树上（别的会话改了用户数据落点与回落之后） | 两份旧探针在今天的树上重跑，64 与 41 个字段与第二轮逐字段相同（任务 15800）；尺子比对两批种子 166176、165924 次，乙的上界小于落得下那一判读数的 0 次（任务 15791） | **站住**：第二轮引的读数搬到今天的树上照引 |
| 乙在槽对那一判上误拒 | O 那一臂推过串的格：722（判出 1039）、705（判出 996），加第一族的 1082（判出 1907），写进都是 0 | **没打中**（三份取样）。**更正第二轮**：1039 不是有效分母，有效分母是 722 |
| 甲误拒 | O 那一臂推过串的格：303（判出 3643）、386（判出 4166），写进 0 | **没打中**。**更正第二轮**：3643 是判出格数，有效分母是 303 与 386 |
| 按记录重建之后三个槽对计数 | 两批种子 41544、41481 步，不同 0 步；产品这边的用例在冻结副本上绿（任务 15801） | **站住** |
| 第 131 批四条修补 | 两腿各自逐条对冻结副本的代码 | **对得上**；文档还差一个前提，见第三节第 1 条 |

这一轮新判的（最后一轮，不再开轮）：

| 条 | 读数与依据 | 判决 |
|---|---|---|
| 甲在取落点被拒那一支上也判出（攻方 T1-4） | 判出 3643 格里 3334 格、4166 格里 3776 格的被拒是 `PlacementRefused`，这一支上误拒 0（任务 15791）；代码文档第 699 行写了，D16 已定项 10 射程没写「落点」，单测与变异行都不点这一支（两腿读码） | **形站住，缺条款与用例**：射程补写，加一条用例与一行变异（第三节第 2、3 条） |
| 四处文档里三处没写「被拒的是合取 7 判的前台写入类」（正推 T1） | 正推腿逐处列了冻结副本的行 | **打中（文字）**：第三节第 1 条 |
| 落得下那一判与乙都把活意图 `R` 里的槽对算进去，用户数据落点不取 `R`（攻方 T2） | 读码；只往白推一边偏，不会误拒；没量过偏多少 | **没有去向**：登欠账（第三节第 4 条） |
| 每次发布克隆分配器多带一张每槽一字节的位图（第一轮判「登欠账量代价」） | 两腿都查到 `checks-owed.md` 没有这一行 | **没有去向**：登欠账（第三节第 4 条） |
| 挂载续做与挂载准入的推串不经先算（正推 T2-6） | 读码 | **有去向**：D16 已定项 10 射程已写先算只罩 `accept`、`publish_user_change` 与走空三处；射程补写时点名这两处不罩 |
| C1003（被宽护卡住那条出路没量过） 那一句「可达要一次瞬时读写错」（攻方 T2） | 攻方读码：两次调用之间后台整理跑一批也走得到（推的，没量过） | 写回 C1003 时照「推的」补一句 |

检查单：

| 格 | 判决 |
|---|---|
| 类型与名字 | 产品代码成立；E180、E181 两份装置与测试里把 `SlotPairCount` 用 `.0` 拆回裸整数来读（两腿一致）。装置与测试不在代码轮的射程里，**不改**，记在这里 |
| 走不到的理由、不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、测试优先多线程、带 --item 的命令与项、新崩溃流多罩了什么 | 成立（两腿一致） |

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-core/src/allocator.rs` | **不改**：乙的增量计数、`SlotPairCount`、重建用例三轮站住 |
| `crates/singlefs-core/src/admission.rs` | **不改**：`slot_pairs_for_data_units` 换类型之后比较的两边都是槽对数 |
| `crates/singlefs-core/src/mounted_session.rs` | **要改（只改注释与测试）**：第三节第 1、2 条 |

## 三、改法

1. `mounted_session.rs` 那三处文档补上前提「被拒的是合取 7 判的前台写入类」。量过（正推腿逐处列行，主 agent 现读第 699 行一带）。只改注释。
2. 给「甲在取落点被拒那一支上判出」加一条用例与一行变异：被拒的是 `PlacementRefused`、这次调用还一串没推、全空聚簇段已少于 `N₀`、前台写入，先算判出、一串不推；变异把甲对 `PlacementRefused` 那一支关掉。量过（任务 15791：这一支 3334 与 3776 格、误拒 0）。只加测试与变异行。
3. 射程补写（kb，写回员）：D16（发布语义） 已定项 10 射程照第一轮第三节第 3 条与第二轮第三节第 5 条，另加：合取 7 那一形在空间准入被拒与取落点被拒两支上都判；挂载续做与挂载准入的推串不经先算；误拒的读数引有效分母（槽对那一判 1082、722、705，合取 7 那一判 303、386）；先算拒绝时不报卸载重挂（用户定另立一题做成单独的查询）。C1002（抬 F 先算对槽对那一判挡得少） 关到实现；C1003 补两批判别子探针的读数与「做成单独的查询」的定案。
4. 欠账新登两行（kb，写回员）：落得下那一判与乙把活意图 `R` 里的槽对算进去、偏多少没量；每次发布克隆分配器的位图代价没量。
5. 第三轮之后不再开轮（`.claude/rules/three-way-inference.md`「第三轮之后停，不开第四轮」）。第 1、2 条只动注释与测试，不再过代码轮；合入后照常跑合入后验证与变异复判。

## 交用户的

- 这一题三轮判完：乙、甲站住，误拒三份取样都是 0。
- 甲在取落点被拒那一支上判出，是它出场最多的一支（取样里约九成）；这一轮才写进条款、才有用例。
- 卸载重挂出路已照用户定的另立一题；查线路那一条是另一件，条款已写进 D26（后台整理与放置回收） 已定项 13 第 4 条。

## 这一轮被判过的文件

- crates/singlefs-core/src/allocator.rs：判了乙的上界与落得下那一判是不是同一把尺、重建之后三个计数、`SlotPairCount` 换齐，三轮站住
- crates/singlefs-core/src/admission.rs：判了 `slot_pairs_for_data_units` 换类型之后比较的语义，站住
- crates/singlefs-core/src/mounted_session.rs：判了四处文档、甲在取落点被拒那一支上判出、先算罩哪几处，要补注释里的前提与一条用例（第三节第 1、2 条）

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 10 | 支撑 | 2026-11-13 改了：射程补写乙、甲与这一轮加的几句（第三节第 3 条，写回员） |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-13 不受影响：宽护那条出路仍在 C1003，条款不动 |
| D26（后台整理与放置回收） 已定项 1 | 不影响 | 2026-11-13 不受影响：活意图 `R` 里的槽对算进两判只登欠账，条款不动 |
