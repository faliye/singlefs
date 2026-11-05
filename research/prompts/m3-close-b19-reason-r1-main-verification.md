# m3-close-b19-reason-r1 判决（第一轮，2026-10-30）

正文 `research/prompts/_m3-close-b19-reason-r1-body.md`。腿：正推 `research/prompts/m3-close-b19-reason-r1-sonnet-output.md`（cab01db4…，模型 `research/prompts/m3-close-b19-reason-r1-sonnet-model/`）、攻方 `research/prompts/m3-close-b19-reason-r1-opus-output.md`（b8f6b2b8…，模型 `research/prompts/m3-close-b19-reason-r1-opus-model/`）、攻方第二次抽样 `research/prompts/m3-close-b19-reason-r1-opus-s2-output.md`（fad3f855…，模型 `research/prompts/m3-close-b19-reason-r1-opus-s2-model/`；只攻 Q1 第 1 问与 Q3 窄方向，为第一次攻方那两格的「没打中」补第二次抽样）。

## 一、核查

- 开工快照倒推：`python3 scripts/agents/snapshot-tree.py m3-close-b19-reason-r1 --out ~/.local/share/singlefs/work/m3-close-b19-reason-r1-judge/tree`：「清单点名 15 份，倒推出 13 份、sha256 对上 13 份；倒推不出 2 份、对不上 0 份」。倒推不出的是 `.claude/kb/checks-owed.md` 与 `.claude/kb/decisions/16-发布语义.md`：快照之后别的会话的提交改过、快照目录里没有 diff。腿引这两份的只有 C934、C937 两行的编号与验收句、D16 已定项 1「抬 F 的上限」一行的式子，主 agent 对着此刻的文件逐处人工核，那几句字句没变。crates 七份逐份对上。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-reason-r1-sonnet-output.md research/prompts/m3-close-b19-reason-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-close-b19-reason-r1-judge/tree --background research/prompts/_m3-close-b19-reason-r1-background.md`：「核了 71 处引文、36 处裸行号，对上 54 处，没判 53 处」，判了的 0 处对不上；第二次抽样单独核：「核了 40 处引文、15 处裸行号，对上 21 处，没判 34 处」，判了的 0 处对不上。没判的是裸行号与快照外的文件，判决用到的主 agent 现读。
- 复跑：三份模型拷进 `~/.local/share/singlefs/work/m3-close-b19-reason-r1-judge/`，经 `run-with-memory-cap.sh 4G` 跑 `tools/rerun.sh --repo <仓根> --frozen <倒推的树> --work <新目录> --model <拷贝>`，三份都退 0：正推腿七步全 identical（`name=q1_scene_and_direction_identical lines=32`、`name=q2_scale1_identical lines=243`、`name=q2_scale4_identical lines=216`、`name=q2_defense_port_identical lines=112`、`name=q3_span_readings_identical lines=71`、`name=q2_summary_identical lines=153`、`name=q2_port_check_against_opus_identical lines=1`）；攻方五份 identical（`name=selftest_output_identical lines=1`、`name=q1_output_identical lines=244`、`name=q2_output_identical lines=15`、`name=q2ext_output_identical lines=12`、`name=q3_output_identical lines=41`）；第二次抽样 `same=false` 0 行。
- 现读坐实（倒推的树里）：`crates/singlefs-checker/src/walk.rs:9033`–`9037` 认 u 落了盘只看「u 与抬 F 那条根同一实例，或 u 那个实例的每一行都有 u < T」；`crates/singlefs-core/src/mount.rs:9230`–`9234` 写的行 (i, T, W) 里 T 取施加之后那一版的 txg（`selected_root_txg: effective_root.checkpoint_txg`）；`crates/singlefs-core/src/transaction.rs:1545` 的 `refuse_while_a_publish_is_frozen` 在 1835、9477、10094、12130 四处被调用；`.claude/kb/decisions/23-journal的角色与格式.md` 第 406 行「根槽写失败重发时 checkpoint_txg 推进一格再发」与第 409 行「这一版的失败处置：发布不接受失败。一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它」，`crates/singlefs-core/src/` 里 grep「推进一格再发」零命中。
- 正文自己写错的一句：节「三、格」Q1 第 2 问写「`first_txg_of_new_instance` 给它 T = u + 1 > u」。三条腿都指出 u + 1 是新实例自己第一次发布的 txg，丁比的是 u 那个旧实例的行，那一行的 T 照 `mount.rs:9232` 取施加之后那一版（T = u 或 T < u），丁报认不出。主 agent 现读坐实，题面那一句不成立。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| Q1 第 1 问　C934 的前提 | **今天的写者上成立（没打中，两次攻方抽样与正推腿一致）**：落盘任一步失败就冻结、下一次经分配器的发布先被 `refuse_while_a_publish_is_frozen` 拒、原样重发；零单元发布失败整个挂载报错、同一实例不再发下一版；`k_tol` 在核心代码里只是预算与几何，没有一段跳过失败的那一版。**这条前提靠 D23 已定项 14 第 406 行那一段还没实现撑着**：那一段一落地（根槽写失败推进 txg 再发），「记录在、根没落、同一实例接着发」就是合法历史，丁在合法盘面上误红（攻方模型 S=4 157/15706、S=8 91/28058，推的模型）。写进 D23 已定项 14 射程与 I-7.9 射程：那一段落地要同批改 I-7.9 的重建 | 三条腿 Q1 第 1 问；攻方第二次抽样模型拆掉四道（行 T 一判、恢复写行取值、挂载报错、冻结）各自必出 mismatch，复跑对上 |
| Q1 第 2 问　崩在根槽之前、换新实例 | **造得出，丁报认不出**（三腿一致）；题面「T = u + 1」不成立（见第一节） | 现读 `mount.rs:9232`、`walk.rs:9033` |
| Q1 第 2 问的方向 | **「重建上限只会不低于真上限、只漏红」不成立**（两腿各自量的）：重建是把那一槽的住户换成 u，不是多认一条；最旧有效根、每块盘最新有效根、非空去重都能往两个方向动，往低动时合法抬 F 被判红。今天的写者上缝是空的（Q1 第 1、2 问），这个方向只在 D23 第 406 行那一段落地之后才有对象，随那一段一起改 | 正推腿 S=4 高 3246、等 9024、低 1097（假设缝打开，推的模型）；攻方坏写者钻缝与合法写者误红盘面各一张 |
| Q1 第 3 问　钉槽 p | 丁不读 p，两形都钉不上，也不需要它（三腿一致） | 腿报告 |
| Q2　C934 三处简化的代价 | **第二轮报的 5.1% / 2.2% 是三处简化全放开的数**（两腿各自在同一批历史上复现：正推 5.0% / 2.1%、攻方 5.1% / 2.2%）；照实现员今天的保守取法，合法抬 F 根部分没判是 **31.7% / 54.0%（正推）、30.0% / 33.5%（攻方）**，S=4 / S=8，J=128，推的模型。三处里 A（只认一层）要和 B（u 是行 T、被抛弃、实例没有行）一起放开才起作用，C（第 0 代根）只在年轻池第一圈内。**定**：照两腿模型里全放开的那一形实现——被盖的槽在 u ≥ T、被抛弃、实例没有行、第 0 代根四种情形下两种住户都算，往下逐层认（u − 3S…）到认出或到窗口外，两种算出的结论一致才判、不一致报部分没判；CPU 与 GPU 同改。理由：用户 2026-10-29 选丁时看的代价数就是这一形的数；两种都算按构造不引入误红 | 两腿 Q2；复跑对上 |
| Q3　C937 宽方向 | **打中**（攻方最小镜像：两个一槽单元、一条跨 3 的记录、记账 2 槽，I-3.1 报部分没判、I-3.18 只说跨度、槽 1 那个活单元没有自己的记录无人提；正推腿小世界 501 个盘面同形）。病根：取宽按 (L)(P)(U) 逐判各取宽，拼出一个两种读法都不认的组合 | 两腿 Q3；复跑对上 |
| Q3　C937 窄方向 | **打中**（攻方第一次抽样没打中；第二次抽样最小镜像：一条窄记录跨 1 罩两槽单元、一条罩着全零槽 63 的未释放泄漏记录、记账 2 槽，I-3.1 取宽后部分没判、I-3.18 只红窄那一条、槽 63 头用不了各条都没对象、I-3.11、I-3.16、I-5.2、I-5.4 都成立；正推腿小世界「窄向游离记录补偿」4052 个、「记账介于记录与单元之间」226 个同类）。病根：上沿只比个数，泄漏多数的一槽与窄记录漏数的尾槽在个数上抵掉。两次抽样在这一格上不一致，按内容对齐后第二次与正推腿给的是同一件事，照「打中了」那一行当线索，主 agent 在倒推的树里现读坐实取宽那一支的上沿只比槽数（`walk.rs:10391` 起的 `allocated_statistic_lies_within_the_bounds_of_either_span_reading`，上沿那一句 10418–10423 拿两份 `slots_counted_at_the_root_reclaim_threshold` 的大者与 `walked_and_still_deferred_slots` 取小，都是个数） | 第二次抽样与正推腿 Q3 |
| Q3　`!=` 改 `>` 够不够 | **不够**：只挡宽方向，对窄方向那一形不起作用（第二次抽样量的） | 第二次抽样 |
| Q3　改法 | **定**：取宽那一支只认整份改记读法，不逐判混取；改记那一份三判之外另加逐槽两判——遍历走到的每一槽都在改记那一份 E(θ_high) 里、改记那一份 E(t) 的每一槽都在 Ω ∪ U 里（defer 之外还罩着没人走到的槽就不成立）；不成立判红。两份最小镜像做成字节级坏镜像进 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，真 checker 上两张都判 I-3.1 红（腿在副本装置上量的形照 `.claude/rules/three-way-inference.md` 在入库装置上重做一次才算）；I-3.1 部分没判的说明补上记账的已分配、两种读法的下沿上沿数（正推腿指的缺口）。CPU 与 GPU 扫描内核同改 | 两腿 Q3 |
| Q3　取宽之前就藏得住的一形 | 宽记录盖住一个没有自己记录的邻居单元、记账照单元记时，按记录自述三判就成立（正推腿小世界 209 个盘面），I-3.18 只说跨度、不提邻居没有记录。这一形与取宽无关，挂在「一个被遍历走到的单元的起点上没有自己的记录、被别的记录的跨度罩着时由哪一条判」上，登记 C944，下一轮先攻它归哪条不变量 | 正推腿 Q3 |

## 三、交用户的

无岔路。告知一件：用户 2026-10-29 选 C717 走丁时，第二轮报的代价（合法抬 F 根部分没判 5.1% / 2.2%）是三处简化全放开的数；批 19 实现员按保守取法写的丁今天在两腿模型上是 30.0%–54.0%。这一轮定照全放开那一形实现，回到用户选丁时看的数。

## 四、被攻过零轮的

Q2 三处简化放开的实现（两种都算、一致才判、逐层往下认）；Q3 只认整份改记读法加逐槽两判；I-3.1 部分没判说明补数。实现交回之后开第二轮攻这几样，连同 C944 归哪条不变量。

## 五、去向

| 编号 | 去向 |
|---|---|
| C934（I-7.9 按 journal 重建抬 F 那一刻的根环靠一条推的前提） | 挪已还清：前提在今天的写者上成立（三条腿、两次攻方抽样），三处简化的代价量过；前提失效的条件（D23 已定项 14 第 406 行那一段落地）写进 D23 已定项 14 射程与 I-7.9 射程 |
| C937（I-3.1 照 I-3.18 改记槽数取宽的条件被攻过零轮） | 留着：两个方向都打中，改法定了，实现交回后第二轮攻；去向改「里程碑三二·4（批 19 下一批实现，第二轮 m3-close-b19-reason-r2）」 |
| C944（新登记） | 一个被遍历走到的单元起点上没有自己的分配记录、被别的记录的跨度罩着时，没有一条不变量点名它（取宽之前 I-3.1 就成立、I-3.18 只说跨度）；去向里程碑三二·4，第二轮先攻归哪条 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-30 改了：射程补一句「根槽写失败推进 txg 再发那一段落地时，I-7.9 按 journal 重建抬 F 那一刻的根环所靠的前提（实例中间的一版根槽一定落了盘）失效，要同批改 I-7.9 的重建」 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-30 不受影响：这一轮只判 I-7.9 重建的前提与 I-3.1 取宽，抬 F 上限的式子本身没动 |
